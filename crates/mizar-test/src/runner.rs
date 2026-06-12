use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use mizar_frontend::lexical_env::{
    FrontendLexicalEnvironmentError, LexicalEnvironmentRequest, LexicalSummaryProvider,
    ResolvedImports,
};
use mizar_frontend::orchestration::{DiagnosticCode, Frontend, FrontendDiagnostic};
use mizar_frontend::parsing::MizarParserSeam;
use mizar_frontend::source::{FrontendSourceLoader, SourceUnitRequest};
use mizar_session::{
    BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
    SourceInput, SourceOriginInput, normalize_path,
};

use crate::diagnostic::{ValidationDiagnostic, ValidationSeverity};
use crate::expectation::{ExpectedOutcome, PipelinePhase};
use crate::harness::{DiscoveryConfig, HarnessError, TestCase, TestPlan, build_test_plan};
use crate::path_rules::absolute_from;
use crate::staged_model::Stage;

const ACTIVE_PARSE_ONLY_TAG: &str = "active_parse_only";
const ALLOW_FRONTEND_RECOVERY_DIAGNOSTICS_TAG: &str = "allow_frontend_recovery_diagnostics";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOnlyRunReport {
    pub results: Vec<ParseOnlyCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOnlyCaseResult {
    pub id: crate::expectation::TestCaseId,
    pub expectation_path: PathBuf,
    pub status: ParseOnlyCaseStatus,
    pub actual_diagnostic_codes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseOnlyCaseStatus {
    Passed,
    Failed,
}

impl ParseOnlyRunReport {
    pub fn passed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == ParseOnlyCaseStatus::Passed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == ParseOnlyCaseStatus::Failed)
            .count()
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ValidationSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == ValidationSeverity::Warning)
            .count()
    }
}

pub fn run_parse_only_corpus(config: &DiscoveryConfig) -> Result<ParseOnlyRunReport, HarnessError> {
    let workspace_root = normalized_workspace_root(config)?;
    let plan = build_test_plan(config)?;
    let mut diagnostics = plan.diagnostics.clone();
    if plan.error_count() > 0 {
        return Ok(ParseOnlyRunReport {
            results: Vec::new(),
            diagnostics,
        });
    }
    diagnostics.extend(validate_active_parse_only_tags(&plan));

    let mut results = Vec::new();
    for (ordinal, case) in active_parse_only_cases(&plan).enumerate() {
        let result = run_parse_only_case(&workspace_root, case, ordinal);
        if result.status == ParseOnlyCaseStatus::Failed {
            diagnostics.push(parse_only_failure_diagnostic(case, &result));
        }
        results.push(result);
    }
    diagnostics.sort();

    Ok(ParseOnlyRunReport {
        results,
        diagnostics,
    })
}

pub fn active_parse_only_cases(plan: &TestPlan) -> impl Iterator<Item = &TestCase> {
    plan.cases.iter().filter(|case| is_active_parse_only(case))
}

fn is_active_parse_only(case: &TestCase) -> bool {
    has_active_parse_only_tag(case)
        && case.expectation.stage == Stage::ParseOnly
        && case.expectation.expected_phase == Some(PipelinePhase::Parse)
        && matches!(
            case.expectation.expected_outcome,
            ExpectedOutcome::Pass | ExpectedOutcome::Fail
        )
        && case
            .source_path
            .extension()
            .is_some_and(|extension| extension == "miz")
}

fn has_active_parse_only_tag(case: &TestCase) -> bool {
    case.expectation
        .tags
        .iter()
        .any(|tag| tag == ACTIVE_PARSE_ONLY_TAG)
}

fn validate_active_parse_only_tags(plan: &TestPlan) -> Vec<ValidationDiagnostic> {
    plan.cases
        .iter()
        .filter(|case| has_active_parse_only_tag(case) && !is_active_parse_only(case))
        .map(|case| {
            ValidationDiagnostic::error(
                &case.expectation_path,
                "parse_only",
                "E-PARSE-ONLY-ACTIVE-GATE",
                format!("parse_only.active_gate.{}", case.id.0),
                "active_parse_only cases must be .miz pass/fail expectations at stage parse_only and phase parse",
            )
        })
        .collect()
}

fn run_parse_only_case(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> ParseOnlyCaseResult {
    let output = run_frontend(workspace_root, case, ordinal);
    let (has_ast, actual_diagnostic_codes) = match output {
        Ok(output) => (
            output.has_ast,
            assertion_diagnostic_codes(case, &output.diagnostics),
        ),
        Err(error) => (false, vec![frontend_error_code(&error)]),
    };
    let expected_diagnostic_codes = &case.expectation.diagnostic_codes;
    let status = match case.expectation.expected_outcome {
        ExpectedOutcome::Pass
            if has_ast && actual_diagnostic_codes == *expected_diagnostic_codes =>
        {
            ParseOnlyCaseStatus::Passed
        }
        ExpectedOutcome::Fail if actual_diagnostic_codes == *expected_diagnostic_codes => {
            ParseOnlyCaseStatus::Passed
        }
        _ => ParseOnlyCaseStatus::Failed,
    };

    ParseOnlyCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status,
        actual_diagnostic_codes,
    }
}

fn run_frontend(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> Result<FrontendRun, String> {
    let prepared = prepare_source_package(workspace_root, case, ordinal)?;
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&prepared.package_root)),
        EmptyProvider,
        MizarParserSeam,
    );
    let ids = InMemorySessionIdAllocator::new();
    let output = frontend
        .run(prepared.request.clone(), &ids)
        .map_err(|error| error.to_string())?;
    Ok(FrontendRun {
        has_ast: output.ast.is_some(),
        diagnostics: output.diagnostics,
    })
}

#[derive(Debug)]
struct FrontendRun {
    has_ast: bool,
    diagnostics: Vec<FrontendDiagnostic>,
}

fn prepare_source_package(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> Result<PreparedSource, String> {
    let source_path = &case.source_path;
    let text = fs::read_to_string(source_path)
        .map_err(|error| format!("failed to read source `{}`: {error}", source_path.display()))?;
    let package_root = temp_package_root(ordinal);
    let temp_source = package_root
        .join("src")
        .join(source_path.file_name().unwrap_or_default());
    fs::create_dir_all(temp_source.parent().unwrap_or(&package_root))
        .map_err(|error| format!("failed to create parse-only temp package: {error}"))?;
    fs::write(&temp_source, text)
        .map_err(|error| format!("failed to write parse-only temp source: {error}"))?;
    let normalized_path = normalize_path(&package_root, &temp_source)
        .map_err(|error| format!("failed to normalize temp source path: {error}"))?;

    Ok(PreparedSource {
        package_root: package_root.clone(),
        request: SourceUnitRequest {
            snapshot: snapshot_id(ordinal),
            input: SourceInput {
                package_id: PackageId::new("mizar-test-corpus"),
                module_path: ModulePath::new(module_path(workspace_root, source_path)),
                normalized_path,
                edition: Edition::new("2026"),
                origin: SourceOriginInput::Disk { path: temp_source },
            },
        },
    })
}

#[derive(Debug)]
struct PreparedSource {
    package_root: PathBuf,
    request: SourceUnitRequest,
}

impl Drop for PreparedSource {
    fn drop(&mut self) {
        match fs::remove_dir_all(&self.package_root) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => {}
        }
    }
}

fn temp_package_root(ordinal: usize) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    std::env::temp_dir().join(format!(
        "mizar-test-parse-only-{}-{ordinal}-{nanos}",
        std::process::id()
    ))
}

fn normalized_workspace_root(config: &DiscoveryConfig) -> Result<PathBuf, HarnessError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        HarnessError::Infrastructure(format!("failed to read current directory: {error}"))
    })?;
    Ok(absolute_from(&current_dir, &config.workspace_root))
}

fn module_path(workspace_root: &Path, source_path: &Path) -> String {
    source_path
        .strip_prefix(workspace_root)
        .unwrap_or(source_path)
        .with_extension("")
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>()
        .join(".")
}

fn snapshot_id(ordinal: usize) -> BuildSnapshotId {
    BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{:064x}",
        ordinal + 1
    ))
    .expect("static parse-only snapshot id should be valid")
}

fn frontend_diagnostic_code(diagnostic: &FrontendDiagnostic) -> String {
    match &diagnostic.code {
        DiagnosticCode::SourceLoad => "source_load".to_owned(),
        DiagnosticCode::Preprocess(kind) => format!("preprocess:{kind:?}"),
        DiagnosticCode::LexicalEnvironment(code) => {
            format!("lexical_environment:{code:?}")
        }
        DiagnosticCode::Lexing(kind) => format!("lexing:{kind:?}"),
        DiagnosticCode::Syntax(code) => code.to_string(),
        _ => "frontend_diagnostic".to_owned(),
    }
}

fn assertion_diagnostic_codes(case: &TestCase, diagnostics: &[FrontendDiagnostic]) -> Vec<String> {
    let syntax_codes = diagnostics
        .iter()
        .filter_map(|diagnostic| match &diagnostic.code {
            DiagnosticCode::Syntax(code) => Some(code.to_string()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let has_non_syntax = diagnostics
        .iter()
        .any(|diagnostic| !matches!(diagnostic.code, DiagnosticCode::Syntax(_)));
    if !syntax_codes.is_empty()
        && (!has_non_syntax
            || case
                .expectation
                .tags
                .iter()
                .any(|tag| tag == ALLOW_FRONTEND_RECOVERY_DIAGNOSTICS_TAG))
    {
        syntax_codes
    } else {
        diagnostics.iter().map(frontend_diagnostic_code).collect()
    }
}

fn frontend_error_code(error: &str) -> String {
    format!("frontend_error:{error}")
}

fn parse_only_failure_diagnostic(
    case: &TestCase,
    result: &ParseOnlyCaseResult,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        &case.expectation_path,
        "parse_only",
        "E-PARSE-ONLY-ASSERT",
        format!("parse_only.{}", case.id.0),
        format!(
            "parse-only case `{}` expected diagnostics {:?} but got {:?}",
            case.id.0, case.expectation.diagnostic_codes, result.actual_diagnostic_codes
        ),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EmptyProvider;

impl LexicalSummaryProvider for EmptyProvider {
    fn resolve_imports(
        &self,
        _request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
        Ok(ResolvedImports {
            imports: Vec::new(),
            summaries: Vec::new(),
            diagnostics: Vec::new(),
        })
    }
}

impl fmt::Display for ParseOnlyCaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        })
    }
}
