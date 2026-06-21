use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use mizar_frontend::lexical_env::{
    ExportRank, ExportedOperatorAssociativity, ExportedOperatorFixity, ExportedOperatorMetadata,
    ExportedSymbolShape, FrontendLexicalEnvironmentError, LexicalEnvironmentRequest,
    LexicalSummaryFingerprint, LexicalSummaryProvider, ModuleId, ModuleLexicalSummary,
    ResolvedImport, ResolvedImportEntry, ResolvedImports, SymbolId, UserSymbolArity,
    UserSymbolKind,
};
use mizar_frontend::orchestration::{DiagnosticCode, Frontend, FrontendDiagnostic};
use mizar_frontend::parsing::MizarParserSeam;
use mizar_frontend::source::{FrontendSourceLoader, SourceUnitRequest};
use mizar_resolve::declarations::DeclarationShellCollector;
use mizar_resolve::env::NamespacePath;
use mizar_resolve::resolved_ast::ModuleId as ResolverModuleId;
use mizar_resolve::symbols::{
    SignatureProjectionExtractor, SymbolCollector, SymbolDiagnostic, SymbolDiagnosticClass,
};
use mizar_session::{
    BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
    SourceInput, SourceOriginInput, normalize_path,
};
use mizar_syntax::SurfaceAst;

use crate::diagnostic::{ValidationDiagnostic, ValidationSeverity};
use crate::expectation::{ExpectedOutcome, PipelinePhase};
use crate::harness::{DiscoveryConfig, HarnessError, TestCase, TestPlan, build_test_plan};
use crate::path_rules::absolute_from;
use crate::staged_model::Stage;

const ACTIVE_PARSE_ONLY_TAG: &str = "active_parse_only";
const ACTIVE_DECLARATION_SYMBOL_TAG: &str = "active_declaration_symbol";
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
    pub snapshot_failure: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseOnlyCaseStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationSymbolRunReport {
    pub results: Vec<DeclarationSymbolCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationSymbolCaseResult {
    pub id: crate::expectation::TestCaseId,
    pub expectation_path: PathBuf,
    pub status: DeclarationSymbolCaseStatus,
    pub actual_detail_keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationSymbolCaseStatus {
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

impl DeclarationSymbolRunReport {
    pub fn passed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == DeclarationSymbolCaseStatus::Passed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == DeclarationSymbolCaseStatus::Failed)
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
    let tests_root = normalized_tests_root(&workspace_root, config);
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
        let result = run_parse_only_case(&workspace_root, &tests_root, case, ordinal);
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

pub fn run_declaration_symbol_corpus(
    config: &DiscoveryConfig,
) -> Result<DeclarationSymbolRunReport, HarnessError> {
    let workspace_root = normalized_workspace_root(config)?;
    let plan = build_test_plan(config)?;
    let mut diagnostics = plan.diagnostics.clone();
    if plan.error_count() > 0 {
        return Ok(DeclarationSymbolRunReport {
            results: Vec::new(),
            diagnostics,
        });
    }
    diagnostics.extend(validate_active_declaration_symbol_tags(&plan));

    let mut results = Vec::new();
    for (ordinal, case) in active_declaration_symbol_cases(&plan).enumerate() {
        let result = run_declaration_symbol_case(&workspace_root, case, ordinal);
        if result.status == DeclarationSymbolCaseStatus::Failed {
            diagnostics.push(declaration_symbol_failure_diagnostic(case, &result));
        }
        results.push(result);
    }
    diagnostics.sort();

    Ok(DeclarationSymbolRunReport {
        results,
        diagnostics,
    })
}

pub fn active_parse_only_cases(plan: &TestPlan) -> impl Iterator<Item = &TestCase> {
    plan.cases.iter().filter(|case| is_active_parse_only(case))
}

pub fn active_declaration_symbol_cases(plan: &TestPlan) -> impl Iterator<Item = &TestCase> {
    plan.cases
        .iter()
        .filter(|case| is_active_declaration_symbol(case))
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

fn is_active_declaration_symbol(case: &TestCase) -> bool {
    has_active_declaration_symbol_tag(case)
        && case.expectation.stage == Stage::DeclarationSymbol
        && case.expectation.expected_phase == Some(PipelinePhase::Resolve)
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

fn has_active_declaration_symbol_tag(case: &TestCase) -> bool {
    case.expectation
        .tags
        .iter()
        .any(|tag| tag == ACTIVE_DECLARATION_SYMBOL_TAG)
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

fn validate_active_declaration_symbol_tags(plan: &TestPlan) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    for case in plan
        .cases
        .iter()
        .filter(|case| has_active_declaration_symbol_tag(case))
    {
        if !is_active_declaration_symbol(case) {
            diagnostics.push(
                ValidationDiagnostic::error(
                    &case.expectation_path,
                    "declaration_symbol",
                    "E-DECLARATION-SYMBOL-ACTIVE-GATE",
                    format!("declaration_symbol.active_gate.{}", case.id.0),
                    "active_declaration_symbol cases must be .miz pass/fail expectations at stage declaration_symbol and phase resolve",
                ),
            );
        }
        if !case.expectation.diagnostic_codes.is_empty() {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "declaration_symbol",
                "E-DECLARATION-SYMBOL-PUBLIC-DIAGNOSTIC-CODES",
                format!("declaration_symbol.public_codes.{}", case.id.0),
                "active_declaration_symbol cases must keep diagnostic_codes empty until public resolver diagnostic codes are specified; use diagnostic_payloads or stable_detail_key for internal detail keys",
            ));
        }
    }
    diagnostics
}

fn run_parse_only_case(
    workspace_root: &Path,
    tests_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> ParseOnlyCaseResult {
    let output = run_frontend(workspace_root, case, ordinal);
    let (has_ast, actual_diagnostic_codes, ast_snapshot) = match output {
        Ok(output) => (
            output.ast.is_some(),
            assertion_diagnostic_codes(case, &output.diagnostics),
            output.ast_snapshot,
        ),
        Err(error) => (false, vec![frontend_error_code(&error)], None),
    };
    let expected_diagnostic_codes = &case.expectation.diagnostic_codes;
    let diagnostic_status = match case.expectation.expected_outcome {
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
    let snapshot_failure = if diagnostic_status == ParseOnlyCaseStatus::Passed {
        case.expectation
            .snapshots
            .as_ref()
            .and_then(|snapshot_path| {
                compare_surface_ast_snapshot(tests_root, snapshot_path, ast_snapshot.as_deref())
            })
    } else {
        None
    };
    let status = if snapshot_failure.is_some() {
        ParseOnlyCaseStatus::Failed
    } else {
        diagnostic_status
    };

    ParseOnlyCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status,
        actual_diagnostic_codes,
        snapshot_failure,
    }
}

fn run_declaration_symbol_case(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> DeclarationSymbolCaseResult {
    let output = run_frontend(workspace_root, case, ordinal);
    let actual_detail_keys = match output {
        Ok(output) => declaration_symbol_detail_keys(workspace_root, case, output),
        Err(error) => vec![format!("frontend_error:{error}")],
    };
    let expected_detail_keys = expected_declaration_symbol_detail_keys(case);
    let status = match case.expectation.expected_outcome {
        ExpectedOutcome::Pass if actual_detail_keys.is_empty() => {
            DeclarationSymbolCaseStatus::Passed
        }
        ExpectedOutcome::Fail if actual_detail_keys == expected_detail_keys => {
            DeclarationSymbolCaseStatus::Passed
        }
        _ => DeclarationSymbolCaseStatus::Failed,
    };

    DeclarationSymbolCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status,
        actual_detail_keys,
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
        ParseOnlyImportProvider,
        MizarParserSeam,
    );
    let ids = InMemorySessionIdAllocator::new();
    let output = frontend
        .run(prepared.request.clone(), &ids)
        .map_err(|error| error.to_string())?;
    let ast_snapshot = output.ast.as_ref().map(|ast| ast.snapshot_text());
    Ok(FrontendRun {
        ast: output.ast,
        ast_snapshot,
        diagnostics: output.diagnostics,
    })
}

#[derive(Debug)]
struct FrontendRun {
    ast: Option<SurfaceAst>,
    ast_snapshot: Option<String>,
    diagnostics: Vec<FrontendDiagnostic>,
}

fn declaration_symbol_detail_keys(
    workspace_root: &Path,
    case: &TestCase,
    output: FrontendRun,
) -> Vec<String> {
    let frontend_diagnostic_keys = assertion_diagnostic_codes(case, &output.diagnostics)
        .into_iter()
        .map(|code| format!("frontend:{code}"))
        .collect::<Vec<_>>();
    if !frontend_diagnostic_keys.is_empty() {
        return frontend_diagnostic_keys;
    }

    let Some(ast) = output.ast else {
        return vec!["declaration_symbol.no_ast".to_owned()];
    };
    let module = resolver_module_id(workspace_root, &case.source_path);
    let namespace = NamespacePath::new(module.path().as_str());
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let projections = SignatureProjectionExtractor::new(&ast, &shells, namespace).extract();
    let result = SymbolCollector::new(ast.source_id, &module, &shells, &projections).collect();

    result
        .diagnostics()
        .iter()
        .map(symbol_diagnostic_detail_key)
        .collect()
}

fn resolver_module_id(workspace_root: &Path, source_path: &Path) -> ResolverModuleId {
    ResolverModuleId::new(
        PackageId::new("mizar-test-corpus"),
        ModulePath::new(module_path(workspace_root, source_path)),
    )
}

fn symbol_diagnostic_detail_key(diagnostic: &SymbolDiagnostic) -> String {
    format!(
        "declaration_symbol.symbol.{}",
        symbol_diagnostic_class_key(diagnostic.class())
    )
}

const fn symbol_diagnostic_class_key(class: SymbolDiagnosticClass) -> &'static str {
    match class {
        SymbolDiagnosticClass::MissingShell => "missing_shell",
        SymbolDiagnosticClass::ContextOnlyShell => "context_only_shell",
        SymbolDiagnosticClass::DuplicateDeclaration => "duplicate_declaration",
        SymbolDiagnosticClass::IllegalOverloadGroup => "illegal_overload_group",
        _ => "unknown",
    }
}

fn expected_declaration_symbol_detail_keys(case: &TestCase) -> Vec<String> {
    if !case.expectation.diagnostic_payloads.is_empty() {
        return case.expectation.diagnostic_payloads.clone();
    }
    case.expectation.stable_detail_key.iter().cloned().collect()
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
        .map_err(|error| format!("failed to create corpus temp package: {error}"))?;
    fs::write(&temp_source, text)
        .map_err(|error| format!("failed to write corpus temp source: {error}"))?;
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
        "mizar-test-corpus-{}-{ordinal}-{nanos}",
        std::process::id()
    ))
}

fn normalized_workspace_root(config: &DiscoveryConfig) -> Result<PathBuf, HarnessError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        HarnessError::Infrastructure(format!("failed to read current directory: {error}"))
    })?;
    Ok(absolute_from(&current_dir, &config.workspace_root))
}

fn normalized_tests_root(workspace_root: &Path, config: &DiscoveryConfig) -> PathBuf {
    absolute_from(workspace_root, &config.tests_root)
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
    if let Some(snapshot_failure) = &result.snapshot_failure {
        return ValidationDiagnostic::error(
            &case.expectation_path,
            "parse_only",
            "E-PARSE-ONLY-SNAPSHOT",
            format!("parse_only.snapshot.{}", case.id.0),
            format!("parse-only case `{}` {snapshot_failure}", case.id.0),
        );
    }
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

fn declaration_symbol_failure_diagnostic(
    case: &TestCase,
    result: &DeclarationSymbolCaseResult,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        &case.expectation_path,
        "declaration_symbol",
        "E-DECLARATION-SYMBOL-ASSERT",
        format!("declaration_symbol.{}", case.id.0),
        format!(
            "declaration-symbol case `{}` expected detail keys {:?} but got {:?}",
            case.id.0,
            expected_declaration_symbol_detail_keys(case),
            result.actual_detail_keys
        ),
    )
}

fn compare_surface_ast_snapshot(
    tests_root: &Path,
    snapshot_path: &Path,
    actual: Option<&str>,
) -> Option<String> {
    let Some(actual) = actual else {
        return Some(format!(
            "requested SurfaceAst snapshot `{}` but the parser produced no AST",
            snapshot_path.display()
        ));
    };
    let full_path = tests_root.join(snapshot_path);
    let expected = match fs::read_to_string(&full_path) {
        Ok(expected) => expected,
        Err(error) => {
            return Some(format!(
                "could not read SurfaceAst snapshot `{}`: {error}",
                snapshot_path.display()
            ));
        }
    };
    if expected == actual {
        None
    } else {
        Some(format!(
            "SurfaceAst snapshot `{}` differed (expected {} bytes, got {} bytes)",
            snapshot_path.display(),
            expected.len(),
            actual.len()
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParseOnlyImportProvider;

impl LexicalSummaryProvider for ParseOnlyImportProvider {
    fn resolve_imports(
        &self,
        request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
        let mut imports = Vec::new();
        let mut summaries = Vec::new();
        let mut seen_modules = BTreeSet::new();

        for (stub_ordinal, stub) in request.import_stubs.iter().enumerate() {
            let module_id = ModuleId::new(stub.path.spelling.as_ref());
            imports.push(ResolvedImportEntry {
                stub_ordinal,
                stub_span: stub.span,
                import: ResolvedImport {
                    module_id: module_id.clone(),
                },
            });

            if seen_modules.insert(module_id.clone()) {
                summaries.push(ModuleLexicalSummary {
                    exported_symbols: parse_only_fixture_symbols(&module_id),
                    module_id,
                    fingerprint: LexicalSummaryFingerprint::new((stub_ordinal as u64) + 1),
                });
            }
        }

        Ok(ResolvedImports {
            imports,
            summaries,
            diagnostics: Vec::new(),
        })
    }
}

fn parse_only_fixture_symbols(module_id: &ModuleId) -> Vec<ExportedSymbolShape> {
    if module_id.as_str() != "parser.type_fixtures" {
        return Vec::new();
    }
    [
        (
            "empty",
            UserSymbolKind::Attribute,
            UserSymbolArity::exact(0),
            None,
        ),
        (
            "T",
            UserSymbolKind::Mode,
            UserSymbolArity::at_least(0),
            None,
        ),
        (
            "R",
            UserSymbolKind::Structure,
            UserSymbolArity::at_least(0),
            None,
        ),
        (
            "TypeCaseAttr",
            UserSymbolKind::Attribute,
            UserSymbolArity::exact(0),
            None,
        ),
        (
            "TypeCaseMode",
            UserSymbolKind::Mode,
            UserSymbolArity::at_least(0),
            None,
        ),
        (
            "TypeCaseStruct",
            UserSymbolKind::Structure,
            UserSymbolArity::at_least(0),
            None,
        ),
        (
            "divides",
            UserSymbolKind::Predicate,
            UserSymbolArity::exact(2),
            None,
        ),
        (
            "<=",
            UserSymbolKind::Predicate,
            UserSymbolArity::exact(2),
            None,
        ),
        (
            "~",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(1),
            Some(ExportedOperatorMetadata {
                fixity: ExportedOperatorFixity::Prefix,
                precedence: 70,
            }),
        ),
        (
            "!",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(1),
            Some(ExportedOperatorMetadata {
                fixity: ExportedOperatorFixity::Postfix,
                precedence: 90,
            }),
        ),
        (
            "|.",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(1),
            None,
        ),
        (
            ".|",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(1),
            None,
        ),
        (
            "++",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(2),
            Some(ExportedOperatorMetadata {
                fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
                precedence: 10,
            }),
        ),
        (
            "**",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(2),
            Some(ExportedOperatorMetadata {
                fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Right),
                precedence: 20,
            }),
        ),
        (
            "%%",
            UserSymbolKind::Functor,
            UserSymbolArity::exact(2),
            Some(ExportedOperatorMetadata {
                fixity: ExportedOperatorFixity::Infix(
                    ExportedOperatorAssociativity::NonAssociative,
                ),
                precedence: 10,
            }),
        ),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(rank, (spelling, kind, arity, operator))| ExportedSymbolShape {
            spelling: spelling.to_owned(),
            symbol_id: SymbolId::new(format!("{}#parse-only#{spelling}", module_id.as_str())),
            source_module: module_id.clone(),
            export_rank: ExportRank::new(rank as u32),
            kind,
            arity,
            operator,
        },
    )
    .collect()
}

impl fmt::Display for ParseOnlyCaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        })
    }
}

impl fmt::Display for DeclarationSymbolCaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ParseOnlyImportProvider;
    use mizar_frontend::lexical_env::{
        ExportedOperatorAssociativity, ExportedOperatorFixity, ExportedOperatorMetadata,
        LexicalEnvironmentRequest, LexicalSummaryProvider, UserSymbolKind,
    };
    use mizar_frontend::preprocess::{ImportStub, ImportStubPath};
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
        SourceRange,
    };
    use std::sync::Arc;

    #[test]
    fn parse_only_provider_resolves_every_stub_and_deduplicates_fixture_summaries() {
        let source_id = source_id(90);
        let stubs = vec![
            import_stub(source_id, "alpha", 0, 5),
            import_stub(source_id, "alpha", 7, 12),
            import_stub(source_id, "parser.type_fixtures", 14, 34),
        ];
        let request = LexicalEnvironmentRequest {
            source_id,
            import_stubs: &stubs,
            edition: Edition::new("2026"),
        };

        let resolved = ParseOnlyImportProvider
            .resolve_imports(&request)
            .expect("parse-only provider should not fail");

        assert_eq!(resolved.imports.len(), 3);
        assert_eq!(
            resolved
                .imports
                .iter()
                .map(|entry| (
                    entry.stub_ordinal,
                    entry.stub_span,
                    entry.import.module_id.as_str()
                ))
                .collect::<Vec<_>>(),
            vec![
                (0, range(source_id, 0, 5), "alpha"),
                (1, range(source_id, 7, 12), "alpha"),
                (2, range(source_id, 14, 34), "parser.type_fixtures"),
            ]
        );
        assert_eq!(resolved.summaries.len(), 2);
        assert_eq!(
            resolved
                .summaries
                .iter()
                .map(|summary| (
                    summary.module_id.as_str(),
                    summary.exported_symbols.len(),
                    summary.fingerprint.get()
                ))
                .collect::<Vec<_>>(),
            vec![("alpha", 0, 1), ("parser.type_fixtures", 15, 3)]
        );
        assert_eq!(
            resolved.summaries[1]
                .exported_symbols
                .iter()
                .map(|symbol| (symbol.spelling.as_str(), symbol.kind, symbol.operator))
                .collect::<Vec<_>>(),
            vec![
                ("empty", UserSymbolKind::Attribute, None),
                ("T", UserSymbolKind::Mode, None),
                ("R", UserSymbolKind::Structure, None),
                ("TypeCaseAttr", UserSymbolKind::Attribute, None),
                ("TypeCaseMode", UserSymbolKind::Mode, None),
                ("TypeCaseStruct", UserSymbolKind::Structure, None),
                ("divides", UserSymbolKind::Predicate, None),
                ("<=", UserSymbolKind::Predicate, None),
                (
                    "~",
                    UserSymbolKind::Functor,
                    Some(ExportedOperatorMetadata {
                        fixity: ExportedOperatorFixity::Prefix,
                        precedence: 70,
                    }),
                ),
                (
                    "!",
                    UserSymbolKind::Functor,
                    Some(ExportedOperatorMetadata {
                        fixity: ExportedOperatorFixity::Postfix,
                        precedence: 90,
                    }),
                ),
                ("|.", UserSymbolKind::Functor, None),
                (".|", UserSymbolKind::Functor, None),
                (
                    "++",
                    UserSymbolKind::Functor,
                    Some(ExportedOperatorMetadata {
                        fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left,),
                        precedence: 10,
                    }),
                ),
                (
                    "**",
                    UserSymbolKind::Functor,
                    Some(ExportedOperatorMetadata {
                        fixity: ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Right,),
                        precedence: 20,
                    }),
                ),
                (
                    "%%",
                    UserSymbolKind::Functor,
                    Some(ExportedOperatorMetadata {
                        fixity: ExportedOperatorFixity::Infix(
                            ExportedOperatorAssociativity::NonAssociative,
                        ),
                        precedence: 10,
                    }),
                ),
            ]
        );
        assert!(resolved.diagnostics.is_empty());
    }

    fn import_stub(source_id: SourceId, spelling: &str, start: usize, end: usize) -> ImportStub {
        let span = range(source_id, start, end);
        ImportStub {
            path: ImportStubPath {
                spelling: Arc::from(spelling),
                relative: None,
                components: vec![Arc::from(spelling)],
                source_segments: vec![span],
                span,
            },
            alias: None,
            span,
        }
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn source_id(byte: u8) -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(byte))
            .unwrap()
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
}
