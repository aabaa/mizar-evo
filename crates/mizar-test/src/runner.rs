use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use mizar_checker::binding_env::{
    BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
    BindingContextOwner, BindingContextRecovery, BindingContextTable, BindingDiagnosticTable,
    BindingEnv, BindingEnvParts, BindingId, BindingKind, BindingStatus, BindingTable,
    BindingTypeSite,
};
use mizar_checker::cluster_trace::ClusterFactTable;
use mizar_checker::overload_resolution::{
    CandidateViabilityInput, CandidateViabilityOutput, OverloadCandidateInput,
    OverloadCollectionOutput, OverloadSelectionOutput, OverloadSiteInput,
    OverloadSiteResolutionInput, SpecificityComparisonInput, SpecificityGraphOutput,
    TemplateExpansionOutput,
};
use mizar_checker::resolved_typed_ast::{
    ExprId, ExpressionMetadataInput, ResolvedNodeKindHint, ResolvedNodeKindHintKind,
    ResolvedTypedAst, ResolvedTypedAstInputs, ResolvedTypedNodeId, ResolvedTypedNodeKind,
    SourceNodeRole,
};
use mizar_checker::type_checker::{
    AttributeInput, AttributePolarity, DeclarationCheckingOutput, DeclarationKind,
    DeclarationStatus, FormulaInput, FormulaKind, ModeExpansion, SourceReserveBindingInput,
    SourceReserveDeclarationBridge, TermFormulaChecker, TermInput, TermKind, TypeExpressionInput,
    TypeHeadInput,
};
use mizar_checker::typed_ast::{
    CoercionTable, InitialObligationTable, LocalTypeContextId, NodeRecoveryState, TypeEntryId,
    TypeStatus, TypeTable, TypedArenaBuilder, TypedAst, TypedAstParts, TypedNode, TypedNodeId,
    TypedNodeLinks, TypedSiteRef, TypingState,
};
use mizar_core::{
    binder_normalization::{NormalizedVarClass, NormalizedVarSort},
    core_ir::{CoreSourceRef, CoreVarId, CoreVarRole},
    elaborator::{
        CheckerOwnedProvenance, CoreBinderSeed, CoreContextInput, CoreVariableSeed,
        ResolvedTypedAstSummary, prepare_core_context,
    },
};
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
use mizar_resolve::env::{
    ContributionKind, DefinitionKind, ExportStatus, NamespacePath, SymbolEntry, SymbolEnv,
    SymbolEnvIndexes, SymbolKind, Visibility,
};
use mizar_resolve::resolved_ast::{
    FullyQualifiedName, LocalSymbolId, ModuleId as ResolverModuleId, SemanticOrigin,
    SymbolId as ResolverSymbolId,
};
use mizar_resolve::symbols::{
    SignatureProjectionExtractor, SymbolCollector, SymbolDiagnostic, SymbolDiagnosticClass,
};
use mizar_session::{
    BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
    SourceAnchor, SourceInput, SourceOriginInput, SourceRange, normalize_path,
};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind, SurfaceTokenKind};

use crate::diagnostic::{ValidationDiagnostic, ValidationSeverity};
use crate::expectation::{ExpectedOutcome, PipelinePhase};
use crate::harness::{DiscoveryConfig, HarnessError, TestCase, TestPlan, build_test_plan};
use crate::path_rules::absolute_from;
use crate::staged_model::Stage;

const ACTIVE_PARSE_ONLY_TAG: &str = "active_parse_only";
const ACTIVE_DECLARATION_SYMBOL_TAG: &str = "active_declaration_symbol";
const ACTIVE_TYPE_ELABORATION_TAG: &str = "active_type_elaboration";
const ALLOW_FRONTEND_RECOVERY_DIAGNOSTICS_TAG: &str = "allow_frontend_recovery_diagnostics";
const TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY: &str =
    "type_elaboration.external_dependency.ast_payload_extraction";

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
#[non_exhaustive]
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
    pub actual_payload_keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeclarationSymbolCaseStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeElaborationRunReport {
    pub results: Vec<TypeElaborationCaseResult>,
    pub diagnostics: Vec<ValidationDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeElaborationCaseResult {
    pub id: crate::expectation::TestCaseId,
    pub expectation_path: PathBuf,
    pub status: TypeElaborationCaseStatus,
    pub actual_detail_keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TypeElaborationCaseStatus {
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

impl TypeElaborationRunReport {
    pub fn passed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == TypeElaborationCaseStatus::Passed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == TypeElaborationCaseStatus::Failed)
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

pub fn run_type_elaboration_corpus(
    config: &DiscoveryConfig,
) -> Result<TypeElaborationRunReport, HarnessError> {
    let workspace_root = normalized_workspace_root(config)?;
    let plan = build_test_plan(config)?;
    let mut diagnostics = plan.diagnostics.clone();
    if plan.error_count() > 0 {
        return Ok(TypeElaborationRunReport {
            results: Vec::new(),
            diagnostics,
        });
    }
    diagnostics.extend(validate_active_type_elaboration_tags(&plan));

    let mut results = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let result = run_type_elaboration_case(&workspace_root, case, ordinal);
        if result.status == TypeElaborationCaseStatus::Failed {
            diagnostics.push(type_elaboration_failure_diagnostic(case, &result));
        }
        results.push(result);
    }
    diagnostics.sort();

    Ok(TypeElaborationRunReport {
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

pub fn active_type_elaboration_cases(plan: &TestPlan) -> impl Iterator<Item = &TestCase> {
    plan.cases
        .iter()
        .filter(|case| is_active_type_elaboration(case))
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

fn is_active_type_elaboration(case: &TestCase) -> bool {
    has_active_type_elaboration_tag(case)
        && case.expectation.stage == Stage::TypeElaboration
        && case.expectation.expected_phase == Some(PipelinePhase::TypeCheck)
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

fn has_active_type_elaboration_tag(case: &TestCase) -> bool {
    case.expectation
        .tags
        .iter()
        .any(|tag| tag == ACTIVE_TYPE_ELABORATION_TAG)
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

fn validate_active_type_elaboration_tags(plan: &TestPlan) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    for case in plan
        .cases
        .iter()
        .filter(|case| has_active_type_elaboration_tag(case))
    {
        if !is_active_type_elaboration(case) {
            diagnostics.push(
                ValidationDiagnostic::error(
                    &case.expectation_path,
                    "type_elaboration",
                    "E-TYPE-ELABORATION-ACTIVE-GATE",
                    format!("type_elaboration.active_gate.{}", case.id.0),
                    "active_type_elaboration cases must be .miz pass/fail expectations at stage type_elaboration and phase type_check",
                ),
            );
        }
        if !case.expectation.diagnostic_codes.is_empty() {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "type_elaboration",
                "E-TYPE-ELABORATION-PUBLIC-DIAGNOSTIC-CODES",
                format!("type_elaboration.public_codes.{}", case.id.0),
                "active_type_elaboration cases must keep diagnostic_codes empty until public checker diagnostic codes are specified; use diagnostic_payloads or stable_detail_key for internal detail keys",
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
    let actual = match output {
        Ok(output) => declaration_symbol_observation(workspace_root, case, output),
        Err(error) => DeclarationSymbolObservation {
            detail_keys: vec![format!("frontend_error:{error}")],
            payload_keys: Vec::new(),
        },
    };
    let expected_detail_keys = expected_declaration_symbol_detail_keys(case);
    let expected_payload_keys = expected_declaration_symbol_payload_keys(case);
    let status = match case.expectation.expected_outcome {
        ExpectedOutcome::Pass
            if actual.detail_keys.is_empty()
                && (case.expectation.declaration_symbol_payloads.is_empty()
                    || actual.payload_keys == expected_payload_keys) =>
        {
            DeclarationSymbolCaseStatus::Passed
        }
        ExpectedOutcome::Fail if actual.detail_keys == expected_detail_keys => {
            DeclarationSymbolCaseStatus::Passed
        }
        _ => DeclarationSymbolCaseStatus::Failed,
    };

    DeclarationSymbolCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status,
        actual_detail_keys: actual.detail_keys,
        actual_payload_keys: actual.payload_keys,
    }
}

fn run_type_elaboration_case(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> TypeElaborationCaseResult {
    let output = run_frontend(workspace_root, case, ordinal);
    let actual_detail_keys = match output {
        Ok(output) => type_elaboration_detail_keys(workspace_root, case, output),
        Err(error) => vec![format!("frontend_error:{error}")],
    };
    let expected_detail_keys = expected_type_elaboration_detail_keys(case);
    let status = match case.expectation.expected_outcome {
        ExpectedOutcome::Pass if actual_detail_keys.is_empty() => TypeElaborationCaseStatus::Passed,
        ExpectedOutcome::Fail if actual_detail_keys == expected_detail_keys => {
            TypeElaborationCaseStatus::Passed
        }
        _ => TypeElaborationCaseStatus::Failed,
    };

    TypeElaborationCaseResult {
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DeclarationSymbolObservation {
    detail_keys: Vec<String>,
    payload_keys: Vec<String>,
}

fn declaration_symbol_observation(
    workspace_root: &Path,
    case: &TestCase,
    output: FrontendRun,
) -> DeclarationSymbolObservation {
    let frontend_diagnostic_keys = frontend_detail_keys(case, &output.diagnostics);
    if !frontend_diagnostic_keys.is_empty() {
        return DeclarationSymbolObservation {
            detail_keys: frontend_diagnostic_keys,
            payload_keys: Vec::new(),
        };
    }

    let Some(ast) = output.ast else {
        return DeclarationSymbolObservation {
            detail_keys: vec!["declaration_symbol.no_ast".to_owned()],
            payload_keys: Vec::new(),
        };
    };
    let resolver = resolver_symbol_collection(workspace_root, case, &ast);
    let payload_keys = if resolver.detail_keys.is_empty() {
        declaration_symbol_payload_keys(&resolver.env)
    } else {
        Vec::new()
    };
    DeclarationSymbolObservation {
        detail_keys: resolver.detail_keys,
        payload_keys,
    }
}

fn type_elaboration_detail_keys(
    workspace_root: &Path,
    case: &TestCase,
    output: FrontendRun,
) -> Vec<String> {
    let frontend_diagnostic_keys = frontend_detail_keys(case, &output.diagnostics);
    if !frontend_diagnostic_keys.is_empty() {
        return frontend_diagnostic_keys
            .into_iter()
            .map(|key| format!("type_elaboration.lower_stage.{key}"))
            .collect();
    }

    let Some(ast) = output.ast else {
        return vec!["type_elaboration.lower_stage.declaration_symbol.no_ast".to_owned()];
    };
    let resolver = resolver_symbol_collection(workspace_root, case, &ast);
    if !resolver.detail_keys.is_empty() {
        return resolver
            .detail_keys
            .into_iter()
            .map(|key| format!("type_elaboration.lower_stage.{key}"))
            .collect();
    }

    let symbols = augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    source_type_elaboration_detail_keys(&ast, resolver.module, &symbols)
}

fn frontend_detail_keys(case: &TestCase, diagnostics: &[FrontendDiagnostic]) -> Vec<String> {
    assertion_diagnostic_codes(case, diagnostics)
        .into_iter()
        .map(|code| format!("frontend:{code}"))
        .collect()
}

#[derive(Debug)]
struct ResolverSymbolCollection {
    module: ResolverModuleId,
    env: SymbolEnv,
    detail_keys: Vec<String>,
}

fn resolver_symbol_collection(
    workspace_root: &Path,
    case: &TestCase,
    ast: &SurfaceAst,
) -> ResolverSymbolCollection {
    let module = resolver_module_id(workspace_root, &case.source_path);
    let namespace = NamespacePath::new(module.path().as_str());
    let shells = DeclarationShellCollector::new(ast, &module).collect();
    let projections = SignatureProjectionExtractor::new(ast, &shells, namespace).extract();
    let result = SymbolCollector::new(ast.source_id, &module, &shells, &projections).collect();

    let detail_keys = result
        .diagnostics()
        .iter()
        .map(symbol_diagnostic_detail_key)
        .collect();
    ResolverSymbolCollection {
        module,
        env: result.into_env(),
        detail_keys,
    }
}

fn augment_type_elaboration_import_summaries(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: SymbolEnv,
) -> SymbolEnv {
    let imported_modules = type_elaboration_imported_fixture_modules(ast, module);
    if imported_modules.is_empty() {
        return symbols;
    }
    let mut indexes = clone_symbol_env_indexes(&symbols);
    for (imported_module, anchor) in imported_modules {
        let frontend_module = ModuleId::new(imported_module.path().as_str());
        let exported_symbols = parse_only_fixture_symbols(&frontend_module);
        if exported_symbols.is_empty() {
            continue;
        }
        let contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource {
                source_id: ast.source_id,
            },
            SourceAnchor::Range(anchor),
        );
        for (ordinal, exported) in exported_symbols.iter().enumerate() {
            if !matches!(
                (exported.kind, exported.spelling.as_str()),
                (UserSymbolKind::Attribute, "empty")
                    | (UserSymbolKind::Attribute, "TypeCaseAttr")
                    | (UserSymbolKind::Mode, "TypeCaseMode")
                    | (UserSymbolKind::Structure, "R")
                    | (UserSymbolKind::Structure, "TypeCaseStruct")
            ) {
                continue;
            }
            let Some(kind) = resolver_symbol_kind(exported.kind) else {
                continue;
            };
            let symbol = ResolverSymbolId::new(
                imported_module.clone(),
                LocalSymbolId::new(format!("summary:{}:{ordinal}", exported.symbol_id.as_str())),
                FullyQualifiedName::new(format!(
                    "{}::{}#{}",
                    imported_module.path().as_str(),
                    exported.spelling,
                    ordinal
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol.clone(),
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    exported.spelling.clone(),
                    SemanticOrigin::new(
                        ast.source_id,
                        imported_module.clone(),
                        SourceAnchor::Range(anchor),
                        vec![ordinal as u32],
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
            indexes.contributions.add_symbol(contribution, symbol);
        }
    }
    SymbolEnv::new(module.clone(), indexes)
}

fn clone_symbol_env_indexes(symbols: &SymbolEnv) -> SymbolEnvIndexes {
    SymbolEnvIndexes {
        imports: symbols.imports().clone(),
        exports: symbols.exports().clone(),
        symbols: symbols.symbols().clone(),
        labels: symbols.labels().clone(),
        definitions: symbols.definitions().clone(),
        overloads: symbols.overloads().clone(),
        registrations: symbols.registrations().clone(),
        lexical_summaries: symbols.lexical_summaries().clone(),
        namespace_graph: symbols.namespace_graph().clone(),
        declaration_dependencies: symbols.declaration_dependencies().clone(),
        contributions: symbols.contributions().clone(),
        module_summaries: symbols.module_summaries().clone(),
    }
}

fn type_elaboration_imported_fixture_modules(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
) -> Vec<(ResolverModuleId, SourceRange)> {
    let mut modules = Vec::new();
    for node in ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ImportAliasDecl))
    {
        let Some(module_path) = node
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .find(|child| matches!(child.kind, SurfaceNodeKind::ModulePath))
        else {
            continue;
        };
        let Ok(spelling) = module_path_spelling(ast, module_path) else {
            continue;
        };
        let frontend_module = ModuleId::new(spelling.as_str());
        if parse_only_fixture_symbols(&frontend_module).is_empty() {
            continue;
        }
        let imported_module =
            ResolverModuleId::new(module.package().clone(), ModulePath::new(spelling.as_str()));
        if modules
            .iter()
            .any(|(existing, _)| existing == &imported_module)
        {
            continue;
        }
        modules.push((imported_module, module_path.range));
    }
    modules
}

fn module_path_spelling(ast: &SurfaceAst, node: &SurfaceNode) -> Result<String, ()> {
    if !matches!(node.kind, SurfaceNodeKind::ModulePath) || node.children.is_empty() {
        return Err(());
    }
    let mut segments = Vec::new();
    for child_id in &node.children {
        let child = ast.node(*child_id).ok_or(())?;
        if !matches!(child.kind, SurfaceNodeKind::PathSegment) || child.children.len() != 1 {
            continue;
        }
        let token = ast
            .node(child.children[0])
            .and_then(SurfaceNode::token_text)
            .ok_or(())?;
        segments.push(token.to_owned());
    }
    if segments.is_empty() {
        return Err(());
    }
    Ok(segments.join("."))
}

fn resolver_symbol_kind(kind: UserSymbolKind) -> Option<SymbolKind> {
    match kind {
        UserSymbolKind::Functor => Some(SymbolKind::Functor),
        UserSymbolKind::Predicate => Some(SymbolKind::Predicate),
        UserSymbolKind::Mode => Some(SymbolKind::Mode),
        UserSymbolKind::Attribute => Some(SymbolKind::Attribute),
        UserSymbolKind::Structure => Some(SymbolKind::Structure),
        UserSymbolKind::Selector => Some(SymbolKind::Selector),
        UserSymbolKind::Constructor => None,
        _ => None,
    }
}

fn source_type_elaboration_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Vec<String> {
    if let Some(keys) = source_builtin_binary_term_formula_detail_keys(ast, module.clone(), symbols)
    {
        return keys;
    }
    let Ok(source_reserve) =
        extract_builtin_source_reserve_declarations(ast, module.clone(), symbols)
    else {
        return vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()];
    };
    let handoff = match assemble_source_reserve_checker_handoff(
        symbols,
        &source_reserve.bridge,
        source_reserve.mode_expansions.clone(),
    ) {
        Ok(handoff) => handoff,
        Err(_) => return vec!["type_elaboration.checker.typed_ast_invalid".to_owned()],
    };
    if !handoff.declarations.diagnostics().is_empty() {
        let mut keys = handoff
            .declarations
            .diagnostics()
            .canonical_iter()
            .map(|(_, diagnostic)| format!("type_elaboration.checker.{}", diagnostic.message_key))
            .collect::<Vec<_>>();
        keys.sort();
        keys.dedup();
        return keys;
    }
    if let Err(error) = assert_source_reserve_handoff(&handoff, &source_reserve.bridge) {
        let detail_key = match error.as_str() {
            "resolved source reserve count mismatch" => {
                "type_elaboration.checker.resolved_typed_ast_count_mismatch"
            }
            "resolved typed AST produced diagnostics" => {
                "type_elaboration.checker.resolved_typed_ast_diagnostics"
            }
            _ => "type_elaboration.checker.resolved_typed_ast_invalid",
        };
        return vec![detail_key.to_owned()];
    }
    if assert_source_reserve_core_summary_readiness(&handoff).is_err() {
        return vec!["type_elaboration.core.resolved_typed_ast_summary_invalid".to_owned()];
    }
    if assert_source_reserve_core_context_readiness(&handoff, &source_reserve.bridge).is_err() {
        return vec!["type_elaboration.core.context_invalid".to_owned()];
    }
    Vec::new()
}

#[derive(Debug, Clone, Copy)]
struct SourceBuiltinBinaryTermFormulaConfig {
    label: &'static str,
    operator: &'static str,
    left: &'static str,
    right: &'static str,
    formula_kind: FormulaKind,
}

const SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS: &[SourceBuiltinBinaryTermFormulaConfig] = &[
    SourceBuiltinBinaryTermFormulaConfig {
        label: "TermFormulaPayloadBoundary",
        operator: "=",
        left: "1",
        right: "1",
        formula_kind: FormulaKind::Equality,
    },
    SourceBuiltinBinaryTermFormulaConfig {
        label: "BuiltinInequalityPayloadBoundary",
        operator: "<>",
        left: "1",
        right: "2",
        formula_kind: FormulaKind::Inequality,
    },
];

#[derive(Debug, Clone)]
struct SourceBuiltinBinaryTermFormula {
    formula_site: TypedSiteRef,
    formula_range: SourceRange,
    formula_kind: FormulaKind,
    left_site: TypedSiteRef,
    left_range: SourceRange,
    right_site: TypedSiteRef,
    right_range: SourceRange,
}

fn source_builtin_binary_term_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_builtin_binary_term_formula(ast)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [
            TermInput::new(
                payload.left_site.clone(),
                context,
                payload.left_range,
                TermKind::Numeral,
            ),
            TermInput::new(
                payload.right_site.clone(),
                context,
                payload.right_range,
                TermKind::Numeral,
            ),
        ],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            payload.formula_kind,
        )
        .with_terms(vec![payload.left_site, payload.right_site])],
    );
    let mut keys = output
        .diagnostics()
        .canonical_iter()
        .map(|(_, diagnostic)| format!("type_elaboration.checker.{}", diagnostic.message_key))
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    Some(keys)
}

fn extract_source_builtin_binary_term_formula(
    ast: &SurfaceAst,
) -> Option<SourceBuiltinBinaryTermFormula> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_builtin_binary_theorem_bridge_node(node))
    {
        return None;
    }
    let theorem_items = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
    let [(_, theorem)] = theorem_items.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, theorem) {
        return None;
    }
    let theorem_tokens = direct_token_texts(ast, theorem);
    let config = SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS
        .iter()
        .copied()
        .find(|config| theorem_tokens.iter().any(|token| token == config.label))?;

    let theorem_structural_children = structural_child_ids(ast, theorem);
    let formula_expressions = theorem_structural_children
        .iter()
        .copied()
        .filter(|id| {
            ast.node(*id)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::FormulaExpression))
        })
        .collect::<Vec<_>>();
    if formula_expressions.len() != 1
        || theorem_structural_children
            .iter()
            .any(|child| !formula_expressions.contains(child))
    {
        return None;
    }
    let formula_expression = ast.node(formula_expressions[0])?;
    let formula_children = structural_child_ids(ast, formula_expression);
    let [formula_id] = formula_children.as_slice() else {
        return None;
    };
    let formula = ast.node(*formula_id)?;
    let operator_tokens = direct_token_texts(ast, formula);
    if !matches!(formula.kind, SurfaceNodeKind::BuiltinPredicateApplication)
        || subtree_has_recovery(ast, formula)
        || operator_tokens.len() != 1
        || operator_tokens[0] != config.operator
    {
        return None;
    }

    let predicate_structural_children = structural_child_ids(ast, formula);
    let term_expressions = predicate_structural_children
        .iter()
        .copied()
        .filter(|id| {
            ast.node(*id)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
        })
        .collect::<Vec<_>>();
    if term_expressions.len() != 2
        || predicate_structural_children
            .iter()
            .any(|child| !term_expressions.contains(child))
    {
        return None;
    }

    let left = exact_numeral_term_operand(ast, term_expressions[0], config.left)?;
    let right = exact_numeral_term_operand(ast, term_expressions[1], config.right)?;
    Some(SourceBuiltinBinaryTermFormula {
        formula_site: surface_site(*formula_id),
        formula_range: formula.range,
        formula_kind: config.formula_kind,
        left_site: surface_site(left.0),
        left_range: left.1,
        right_site: surface_site(right.0),
        right_range: right.1,
    })
}

fn exact_numeral_term_operand(
    ast: &SurfaceAst,
    term_expression_id: SurfaceNodeId,
    expected_spelling: &str,
) -> Option<(SurfaceNodeId, SourceRange)> {
    let term_expression = ast.node(term_expression_id)?;
    if !matches!(term_expression.kind, SurfaceNodeKind::TermExpression)
        || subtree_has_recovery(ast, term_expression)
    {
        return None;
    }
    let term_children = structural_child_ids(ast, term_expression);
    let [term_id] = term_children.as_slice() else {
        return None;
    };
    let term = ast.node(*term_id)?;
    if matches!(term.kind, SurfaceNodeKind::NumeralTerm)
        && direct_token_texts(ast, term).as_slice() == [expected_spelling]
        && structural_child_ids(ast, term).is_empty()
    {
        Some((*term_id, term.range))
    } else {
        None
    }
}

fn structural_child_ids(ast: &SurfaceAst, node: &SurfaceNode) -> Vec<SurfaceNodeId> {
    node.children
        .iter()
        .copied()
        .filter(|child| {
            ast.node(*child)
                .is_some_and(|child_node| !matches!(child_node.kind, SurfaceNodeKind::Token(_)))
        })
        .collect()
}

fn is_supported_builtin_binary_theorem_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::BuiltinPredicateApplication
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::Token(_)
    )
}

fn direct_token_texts(ast: &SurfaceAst, node: &SurfaceNode) -> Vec<String> {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter_map(SurfaceNode::token_text)
        .map(str::to_owned)
        .collect()
}

fn surface_site(id: SurfaceNodeId) -> TypedSiteRef {
    TypedSiteRef::Node(TypedNodeId::new(id.index()))
}

fn source_module_binding_env(
    ast: &SurfaceAst,
    module: ResolverModuleId,
) -> Result<BindingEnv, mizar_checker::binding_env::BindingEnvError> {
    let mut contexts = BindingContextTable::new();
    let context = contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: Vec::new(),
        visible_bindings: Vec::new(),
        recovery: BindingContextRecovery::Normal,
    });
    debug_assert_eq!(context, BindingContextId::new(0));
    BindingEnv::try_new(BindingEnvParts {
        source_id: ast.source_id,
        module_id: module,
        contexts,
        bindings: BindingTable::new(),
        diagnostics: BindingDiagnosticTable::new(),
    })
}

#[derive(Debug)]
struct SourceReserveHandoff {
    binding_env: BindingEnv,
    declarations: DeclarationCheckingOutput,
    typed_ast: TypedAst,
    resolved: ResolvedTypedAst,
}

#[derive(Debug)]
struct SourceReserveExtraction {
    bridge: SourceReserveDeclarationBridge,
    mode_expansions: BTreeMap<ResolverSymbolId, ModeExpansion>,
}

#[cfg(test)]
impl SourceReserveExtraction {
    fn bindings(&self) -> &[SourceReserveBindingInput] {
        self.bridge.bindings()
    }

    fn module_id(&self) -> &ResolverModuleId {
        self.bridge.module_id()
    }

    fn module_context(&self) -> mizar_checker::binding_env::BindingContextId {
        self.bridge.module_context()
    }

    fn type_node(&self, index: usize) -> mizar_checker::typed_ast::TypedNodeId {
        self.bridge.type_node(index)
    }

    fn declaration_node(&self, index: usize) -> mizar_checker::typed_ast::TypedNodeId {
        self.bridge.declaration_node(index)
    }

    #[cfg(test)]
    fn mode_expansions(&self) -> &BTreeMap<ResolverSymbolId, ModeExpansion> {
        &self.mode_expansions
    }
}

#[derive(Debug, Clone)]
struct SourceTypeExpression {
    range: SourceRange,
    spelling: String,
    head: TypeHeadInput,
    attributes: Vec<AttributeInput>,
}

#[derive(Debug, Clone)]
struct SourceModeExpansionCandidate {
    definition_range: SourceRange,
    expansion: ModeExpansion,
    dependencies: Vec<ResolverSymbolId>,
}

#[derive(Debug, Clone)]
struct SourceModeExpansionEntry {
    definition_range: SourceRange,
    expansion: ModeExpansion,
    chain_edges_to_terminal: usize,
    chain_terminal_is_safe_builtin: bool,
}

#[derive(Debug, Clone, Copy)]
struct SourceModeExpansionTraversalBudget {
    mode_definition_count: usize,
}

impl SourceModeExpansionTraversalBudget {
    fn from_ast(ast: &SurfaceAst) -> Self {
        Self {
            mode_definition_count: surface_nodes_with_kind(ast, SurfaceNodeKind::ModeDefinition)
                .len(),
        }
    }

    fn permits_depth(self, depth: usize) -> bool {
        depth < self.mode_definition_count
    }
}

fn extract_builtin_source_reserve_declarations(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<SourceReserveExtraction, ()> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_builtin_reserve_bridge_node(node))
    {
        return Err(());
    }
    let reserve_items = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ReserveItem))
        .collect::<Vec<_>>();
    if reserve_items.is_empty() {
        return Err(());
    }

    let mut bindings = Vec::new();
    let mut source_range = None;
    for item in reserve_items {
        if subtree_has_recovery(ast, item) {
            return Err(());
        }
        source_range = Some(merge_optional_range(source_range, item.range));
        let segments = item
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .filter(|child| matches!(child.kind, SurfaceNodeKind::ReserveSegment))
            .collect::<Vec<_>>();
        if segments.is_empty() {
            return Err(());
        }
        for segment in segments {
            bindings.extend(extract_builtin_reserve_segment(
                ast, segment, &module, symbols,
            )?);
        }
    }

    if bindings.is_empty() {
        return Err(());
    }
    let bridge = SourceReserveDeclarationBridge::new(
        ast.source_id,
        module.clone(),
        source_range.expect("reserve_items was non-empty"),
        bindings,
    )
    .map_err(|_| ())?;
    let mode_expansions = extract_source_local_mode_expansions(ast, &module, symbols, &bridge);
    Ok(SourceReserveExtraction {
        bridge,
        mode_expansions,
    })
}

fn extract_source_local_mode_expansions(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
    bridge: &SourceReserveDeclarationBridge,
) -> BTreeMap<ResolverSymbolId, ModeExpansion> {
    let attributed_mode_heads = bridge
        .bindings()
        .iter()
        .filter(|binding| !binding.type_attributes.is_empty())
        .filter_map(|binding| match &binding.type_head {
            TypeHeadInput::Symbol(symbol)
                if source_reserve_symbol_head_kind(symbols, module, symbol)
                    == Some(SymbolKind::Mode) =>
            {
                Some(symbol.clone())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let bare_mode_heads = bridge
        .bindings()
        .iter()
        .filter(|binding| binding.type_attributes.is_empty())
        .filter_map(|binding| match &binding.type_head {
            TypeHeadInput::Symbol(symbol)
                if source_reserve_symbol_head_kind(symbols, module, symbol)
                    == Some(SymbolKind::Mode) =>
            {
                Some(symbol.clone())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mixed_mode_heads = attributed_mode_heads
        .intersection(&bare_mode_heads)
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut blocked_attributed_mode_heads = mixed_mode_heads;
    let traversal_budget = SourceModeExpansionTraversalBudget::from_ast(ast);
    let dependency_collector = AttributedModeDependencyCollector {
        ast,
        module,
        symbols,
        bridge,
        attributed_mode_heads: &attributed_mode_heads,
        traversal_budget,
    };
    for binding in bridge.bindings() {
        if !binding.type_attributes.is_empty() {
            continue;
        }
        let TypeHeadInput::Symbol(symbol) = &binding.type_head else {
            continue;
        };
        if source_reserve_symbol_head_kind(symbols, module, symbol) != Some(SymbolKind::Mode) {
            continue;
        }
        let mut visiting = BTreeSet::new();
        dependency_collector.collect(
            symbol,
            binding.type_range,
            &mut visiting,
            &mut blocked_attributed_mode_heads,
            0,
        );
    }

    let mut expansions = BTreeMap::new();
    let extractor = SourceLocalModeExpansionExtractor {
        ast,
        module,
        symbols,
        bridge,
        attributed_mode_heads: &attributed_mode_heads,
        blocked_attributed_mode_heads: &blocked_attributed_mode_heads,
        traversal_budget,
    };
    for binding in bridge.bindings() {
        let TypeHeadInput::Symbol(symbol) = &binding.type_head else {
            continue;
        };
        if blocked_attributed_mode_heads.contains(symbol)
            || expansions.contains_key(symbol)
            || source_reserve_symbol_head_kind(symbols, module, symbol) != Some(SymbolKind::Mode)
        {
            continue;
        }
        let mut visiting = BTreeSet::new();
        let _ = extractor.insert(
            symbol,
            binding.type_range,
            &mut visiting,
            &mut expansions,
            0,
            attributed_mode_heads.contains(symbol),
        );
    }
    expansions
        .into_iter()
        .map(|(symbol, entry)| (symbol, entry.expansion))
        .collect()
}

struct SourceLocalModeExpansionExtractor<'a> {
    ast: &'a SurfaceAst,
    module: &'a ResolverModuleId,
    symbols: &'a SymbolEnv,
    bridge: &'a SourceReserveDeclarationBridge,
    attributed_mode_heads: &'a BTreeSet<ResolverSymbolId>,
    blocked_attributed_mode_heads: &'a BTreeSet<ResolverSymbolId>,
    traversal_budget: SourceModeExpansionTraversalBudget,
}

impl SourceLocalModeExpansionExtractor<'_> {
    fn insert(
        &self,
        symbol: &ResolverSymbolId,
        reserve_type_range: SourceRange,
        visiting: &mut BTreeSet<ResolverSymbolId>,
        expansions: &mut BTreeMap<ResolverSymbolId, SourceModeExpansionEntry>,
        depth: usize,
        root_is_attributed: bool,
    ) -> bool {
        if !self.traversal_budget.permits_depth(depth) {
            return false;
        }
        if let Some(entry) = expansions.get(symbol) {
            if depth >= 1 && entry.definition_range.end > reserve_type_range.start {
                return false;
            }
            return depth < 1
                || mode_expansion_can_feed_chain_dependency(
                    &entry.expansion,
                    entry.chain_edges_to_terminal,
                    entry.chain_terminal_is_safe_builtin,
                    depth,
                    root_is_attributed,
                    self.symbols,
                    self.module,
                );
        }
        let is_attributed_root = depth == 0 && root_is_attributed;
        if !visiting.insert(symbol.clone())
            || self.blocked_attributed_mode_heads.contains(symbol)
            || (depth > 0 && self.attributed_mode_heads.contains(symbol))
            || source_reserve_symbol_head_kind(self.symbols, self.module, symbol)
                != Some(SymbolKind::Mode)
        {
            return false;
        }
        let Some(candidate) = extract_source_local_mode_expansion(
            self.ast,
            self.module,
            self.symbols,
            symbol,
            reserve_type_range,
            self.bridge,
        ) else {
            visiting.remove(symbol);
            return false;
        };
        if is_attributed_root
            && !mode_expansion_is_supported_attributed_root(
                &candidate.expansion,
                &candidate.dependencies,
                self.symbols,
                self.module,
            )
        {
            visiting.remove(symbol);
            return false;
        }
        if depth >= 1
            && !mode_expansion_candidate_can_feed_chain_dependency(
                &candidate,
                depth,
                root_is_attributed,
                self.symbols,
                self.module,
            )
        {
            visiting.remove(symbol);
            return false;
        }
        for dependency in &candidate.dependencies {
            if self.attributed_mode_heads.contains(dependency)
                || !self.insert(
                    dependency,
                    candidate.definition_range,
                    visiting,
                    expansions,
                    depth + 1,
                    root_is_attributed,
                )
            {
                visiting.remove(symbol);
                return false;
            }
        }
        let (chain_edges_to_terminal, chain_terminal_is_safe_builtin) =
            mode_expansion_chain_metadata(&candidate, expansions);
        expansions.insert(
            symbol.clone(),
            SourceModeExpansionEntry {
                definition_range: candidate.definition_range,
                expansion: candidate.expansion,
                chain_edges_to_terminal,
                chain_terminal_is_safe_builtin,
            },
        );
        visiting.remove(symbol);
        true
    }
}

struct AttributedModeDependencyCollector<'a> {
    ast: &'a SurfaceAst,
    module: &'a ResolverModuleId,
    symbols: &'a SymbolEnv,
    bridge: &'a SourceReserveDeclarationBridge,
    attributed_mode_heads: &'a BTreeSet<ResolverSymbolId>,
    traversal_budget: SourceModeExpansionTraversalBudget,
}

impl AttributedModeDependencyCollector<'_> {
    fn collect(
        &self,
        symbol: &ResolverSymbolId,
        reserve_type_range: SourceRange,
        visiting: &mut BTreeSet<ResolverSymbolId>,
        blocked: &mut BTreeSet<ResolverSymbolId>,
        depth: usize,
    ) {
        if !self.traversal_budget.permits_depth(depth) {
            return;
        }
        if !visiting.insert(symbol.clone()) {
            return;
        }
        if let Some(candidate) = extract_source_local_mode_expansion(
            self.ast,
            self.module,
            self.symbols,
            symbol,
            reserve_type_range,
            self.bridge,
        ) {
            for dependency in &candidate.dependencies {
                if self.attributed_mode_heads.contains(dependency) {
                    blocked.insert(dependency.clone());
                } else {
                    self.collect(
                        dependency,
                        candidate.definition_range,
                        visiting,
                        blocked,
                        depth + 1,
                    );
                }
            }
        }
        visiting.remove(symbol);
    }
}

fn mode_expansion_is_safe_chain_terminal(expansion: &ModeExpansion) -> bool {
    expansion.attributes.is_empty()
        && expansion.radix.attributes.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject
        )
}

fn mode_expansion_is_chain_dependency_terminal(
    expansion: &ModeExpansion,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    depth: usize,
) -> bool {
    mode_expansion_is_safe_chain_terminal(expansion)
        || (depth == 1
            && mode_expansion_is_direct_structure_rhs_terminal(expansion, symbols, module))
        || (depth == 1 && mode_expansion_is_direct_attributed_builtin_rhs_terminal(expansion))
}

fn mode_expansion_candidate_can_feed_chain_dependency(
    candidate: &SourceModeExpansionCandidate,
    depth: usize,
    root_is_attributed: bool,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    if mode_expansion_is_chain_dependency_terminal(&candidate.expansion, symbols, module, depth) {
        return candidate.dependencies.is_empty();
    }
    !root_is_attributed
        && candidate.dependencies.len() == 1
        && mode_expansion_is_bare_local_mode_dependency(
            &candidate.expansion,
            &candidate.dependencies[0],
            symbols,
            module,
        )
}

fn mode_expansion_can_feed_chain_dependency(
    expansion: &ModeExpansion,
    _chain_edges_to_terminal: usize,
    chain_terminal_is_safe_builtin: bool,
    depth: usize,
    root_is_attributed: bool,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    mode_expansion_is_chain_dependency_terminal(expansion, symbols, module, depth)
        || (!root_is_attributed
            && chain_terminal_is_safe_builtin
            && mode_expansion_is_bare_local_mode_head(expansion, symbols, module))
}

fn mode_expansion_chain_metadata(
    candidate: &SourceModeExpansionCandidate,
    expansions: &BTreeMap<ResolverSymbolId, SourceModeExpansionEntry>,
) -> (usize, bool) {
    if candidate.dependencies.is_empty() {
        return (
            0,
            mode_expansion_is_safe_chain_terminal(&candidate.expansion),
        );
    }
    candidate
        .dependencies
        .iter()
        .filter_map(|dependency| expansions.get(dependency))
        .map(|entry| {
            (
                entry.chain_edges_to_terminal.saturating_add(1),
                entry.chain_terminal_is_safe_builtin,
            )
        })
        .max_by_key(|(chain_edges_to_terminal, _)| *chain_edges_to_terminal)
        .unwrap_or((usize::MAX, false))
}

fn mode_expansion_is_bare_local_mode_dependency(
    expansion: &ModeExpansion,
    dependency: &ResolverSymbolId,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    expansion.attributes.is_empty()
        && expansion.radix.attributes.is_empty()
        && expansion.radix.args.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::Symbol(ref radix)
                if radix == dependency
                    && source_reserve_symbol_head_kind(symbols, module, radix)
                        == Some(SymbolKind::Mode)
        )
}

fn mode_expansion_is_bare_local_mode_head(
    expansion: &ModeExpansion,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    expansion.attributes.is_empty()
        && expansion.radix.attributes.is_empty()
        && expansion.radix.args.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::Symbol(ref radix)
                if source_reserve_symbol_head_kind(symbols, module, radix)
                    == Some(SymbolKind::Mode)
        )
}

fn mode_expansion_is_supported_attributed_root(
    expansion: &ModeExpansion,
    dependencies: &[ResolverSymbolId],
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    if dependencies.is_empty() {
        return mode_expansion_is_safe_chain_terminal(expansion)
            || mode_expansion_is_direct_structure_rhs_terminal(expansion, symbols, module)
            || mode_expansion_is_direct_attributed_builtin_rhs_terminal(expansion);
    }
    dependencies.len() == 1
        && expansion.attributes.is_empty()
        && expansion.radix.attributes.is_empty()
        && expansion.radix.args.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::Symbol(ref dependency) if dependency == &dependencies[0]
        )
}

fn mode_expansion_is_direct_structure_rhs_terminal(
    expansion: &ModeExpansion,
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
) -> bool {
    expansion.attributes.is_empty()
        && expansion.radix.attributes.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::Symbol(ref radix)
                if source_reserve_symbol_head_kind(symbols, module, radix)
                    == Some(SymbolKind::Structure)
        )
}

fn mode_expansion_is_direct_attributed_builtin_rhs_terminal(expansion: &ModeExpansion) -> bool {
    !expansion.attributes.is_empty()
        && expansion
            .attributes
            .iter()
            .all(|attribute| attribute.args.is_empty())
        && expansion.radix.attributes.is_empty()
        && expansion.radix.args.is_empty()
        && matches!(
            expansion.radix.head,
            TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject
        )
}

fn extract_source_local_mode_expansion(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
    symbol: &ResolverSymbolId,
    use_site_range: SourceRange,
    bridge: &SourceReserveDeclarationBridge,
) -> Option<SourceModeExpansionCandidate> {
    if symbol.module() != module {
        return None;
    }
    let spelling = source_mode_symbol_spelling(symbol)?;
    let definitions = surface_nodes_with_kind(ast, SurfaceNodeKind::ModeDefinition)
        .into_iter()
        .filter(|(id, node)| {
            !subtree_has_recovery(ast, node)
                && node.range.end <= use_site_range.start
                && !mode_definition_has_local_context(ast, *id)
                && mode_definition_pattern_spelling(ast, node).as_deref() == Some(spelling)
        })
        .collect::<Vec<_>>();
    let [(_, definition)] = definitions.as_slice() else {
        return None;
    };
    let rhs = extract_source_mode_rhs(ast, definition, module, symbols)?;
    if matches!(
        rhs.head,
        TypeHeadInput::Symbol(ref radix)
            if source_reserve_symbol_head_kind(symbols, module, radix) == Some(SymbolKind::Structure)
                && !local_structure_definition_precedes(
                    ast,
                    module,
                    symbols,
                    radix,
                    definition.range.start,
                )
    ) {
        return None;
    }
    let dependencies = match &rhs.head {
        TypeHeadInput::Symbol(dependency)
            if source_reserve_symbol_head_kind(symbols, module, dependency)
                == Some(SymbolKind::Mode) =>
        {
            vec![dependency.clone()]
        }
        _ => Vec::new(),
    };
    Some(SourceModeExpansionCandidate {
        definition_range: definition.range,
        expansion: ModeExpansion::new(
            TypeExpressionInput::new(
                TypedSiteRef::Node(bridge.root_node()),
                rhs.range,
                rhs.spelling,
                rhs.head,
            ),
            rhs.attributes,
        ),
        dependencies,
    })
}

fn source_mode_symbol_spelling(symbol: &ResolverSymbolId) -> Option<&str> {
    source_local_symbol_spelling(symbol)
}

fn source_local_symbol_spelling(symbol: &ResolverSymbolId) -> Option<&str> {
    let spelling = symbol.local().as_str();
    let name = spelling.strip_prefix("name=").unwrap_or_else(|| {
        spelling
            .split_once(":name=")
            .map(|(_, name)| name)
            .unwrap_or(spelling)
    });
    let name = name.split_once(':').map_or(name, |(name, _)| name);
    let mut slash_parts = name.split('/');
    let first = slash_parts.next();
    let second = slash_parts.next();
    let third = slash_parts.next();
    let name = match (first, second, third) {
        (Some(_), Some(spelling), Some(_)) => spelling,
        (Some(spelling), Some(_), None) => spelling,
        _ => name,
    };
    (!name.is_empty()).then_some(name)
}

fn local_structure_definition_precedes(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
    symbol: &ResolverSymbolId,
    before: usize,
) -> bool {
    if symbol.module() != module
        || source_reserve_symbol_head_kind(symbols, module, symbol) != Some(SymbolKind::Structure)
    {
        return false;
    }
    let Some(spelling) = source_local_symbol_spelling(symbol) else {
        return false;
    };
    let definitions = surface_nodes_with_kind(ast, SurfaceNodeKind::StructureDefinition)
        .into_iter()
        .filter(|(_, node)| {
            !subtree_has_recovery(ast, node)
                && node.range.end <= before
                && structure_definition_pattern_spelling(ast, node).as_deref() == Some(spelling)
        })
        .collect::<Vec<_>>();
    matches!(definitions.as_slice(), [(_, _)])
}

fn mode_definition_pattern_spelling(ast: &SurfaceAst, node: &SurfaceNode) -> Option<String> {
    if !matches!(node.kind, SurfaceNodeKind::ModeDefinition) {
        return None;
    }
    let pattern = node
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .find(|child| matches!(child.kind, SurfaceNodeKind::ModePattern))?;
    if pattern.children.len() != 1 {
        return None;
    }
    let token_node = ast.node(pattern.children[0])?;
    match &token_node.kind {
        SurfaceNodeKind::Token(token) if token.kind == SurfaceTokenKind::Identifier => {
            Some(token.text.to_string())
        }
        _ => None,
    }
}

fn structure_definition_pattern_spelling(ast: &SurfaceAst, node: &SurfaceNode) -> Option<String> {
    if !matches!(node.kind, SurfaceNodeKind::StructureDefinition) {
        return None;
    }
    let pattern = node
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .find(|child| matches!(child.kind, SurfaceNodeKind::StructurePattern))?;
    if pattern.children.len() != 1 {
        return None;
    }
    let token_node = ast.node(pattern.children[0])?;
    match &token_node.kind {
        SurfaceNodeKind::Token(token) if token.kind == SurfaceTokenKind::Identifier => {
            Some(token.text.to_string())
        }
        _ => None,
    }
}

fn extract_source_mode_rhs(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceTypeExpression> {
    let rhs_nodes = node
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter(|child| matches!(child.kind, SurfaceNodeKind::TypeExpression))
        .collect::<Vec<_>>();
    let [rhs] = rhs_nodes.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, rhs) {
        return None;
    }
    let rhs = extract_builtin_source_type_expression(ast, rhs, module, symbols).ok()?;
    if !rhs.attributes.is_empty()
        && !matches!(
            rhs.head,
            TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject
        )
    {
        return None;
    }
    Some(rhs)
}

fn surface_nodes_with_kind(
    ast: &SurfaceAst,
    kind: SurfaceNodeKind,
) -> Vec<(SurfaceNodeId, &SurfaceNode)> {
    let mut output = Vec::new();
    if let Some(root) = ast.root() {
        collect_surface_nodes_with_kind(ast, root, &kind, &mut output);
    }
    output
}

fn collect_surface_nodes_with_kind<'a>(
    ast: &'a SurfaceAst,
    id: SurfaceNodeId,
    kind: &SurfaceNodeKind,
    output: &mut Vec<(SurfaceNodeId, &'a SurfaceNode)>,
) {
    let Some(node) = ast.node(id) else {
        return;
    };
    if &node.kind == kind {
        output.push((id, node));
    }
    for child in &node.children {
        collect_surface_nodes_with_kind(ast, *child, kind, output);
    }
}

fn mode_definition_has_local_context(ast: &SurfaceAst, mode_id: SurfaceNodeId) -> bool {
    let Some(block_id) = containing_definition_block(ast, mode_id) else {
        return false;
    };
    ast.node(block_id)
        .is_some_and(|block| subtree_has_definition_local_context(ast, block, mode_id))
}

fn containing_definition_block(ast: &SurfaceAst, target: SurfaceNodeId) -> Option<SurfaceNodeId> {
    surface_nodes_with_kind(ast, SurfaceNodeKind::DefinitionBlockItem)
        .into_iter()
        .filter(|(id, _)| subtree_contains_node(ast, *id, target))
        .min_by_key(|(_, node)| node.range.end.saturating_sub(node.range.start))
        .map(|(id, _)| id)
}

fn subtree_contains_node(ast: &SurfaceAst, current: SurfaceNodeId, target: SurfaceNodeId) -> bool {
    if current == target {
        return true;
    }
    ast.node(current).is_some_and(|node| {
        node.children
            .iter()
            .any(|child| subtree_contains_node(ast, *child, target))
    })
}

fn subtree_has_definition_local_context(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    mode_id: SurfaceNodeId,
) -> bool {
    for child_id in &node.children {
        if *child_id == mode_id {
            continue;
        }
        let Some(child) = ast.node(*child_id) else {
            continue;
        };
        if matches!(
            child.kind,
            SurfaceNodeKind::DefinitionParameter
                | SurfaceNodeKind::AssumptionStatement
                | SurfaceNodeKind::GivenStatement
        ) || subtree_has_definition_local_context(ast, child, mode_id)
        {
            return true;
        }
    }
    false
}

fn is_supported_builtin_reserve_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::Token(_)
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::ImportItem
            | SurfaceNodeKind::ImportAliasDecl
            | SurfaceNodeKind::ModulePath
            | SurfaceNodeKind::PathSegment
            | SurfaceNodeKind::DefinitionBlockItem
            | SurfaceNodeKind::DefinitionParameter
            | SurfaceNodeKind::QualifiedVariableSegment
            | SurfaceNodeKind::AttributeDefinition
            | SurfaceNodeKind::AttributePattern
            | SurfaceNodeKind::ModeDefinition
            | SurfaceNodeKind::ModePattern
            | SurfaceNodeKind::StructureDefinition
            | SurfaceNodeKind::StructurePattern
            | SurfaceNodeKind::StructureField
            | SurfaceNodeKind::FormulaDefiniens
            | SurfaceNodeKind::FormulaCase
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::FormulaConstant(_)
            | SurfaceNodeKind::ReserveItem
            | SurfaceNodeKind::ReserveSegment
            | SurfaceNodeKind::TypeExpression
            | SurfaceNodeKind::TypeArguments
            | SurfaceNodeKind::AttributeChain
            | SurfaceNodeKind::AttributeRef
            | SurfaceNodeKind::QualifiedSymbol
            | SurfaceNodeKind::TypeHead
            | SurfaceNodeKind::ErrorRecovery(_)
    )
}

fn extract_builtin_reserve_segment(
    ast: &SurfaceAst,
    segment: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<Vec<SourceReserveBindingInput>, ()> {
    if subtree_has_recovery(ast, segment) {
        return Err(());
    }

    let mut identifiers = Vec::new();
    let mut saw_for = false;
    let mut type_expression = None;
    for child_id in &segment.children {
        let child = ast.node(*child_id).ok_or(())?;
        match &child.kind {
            SurfaceNodeKind::Token(token)
                if token.kind == SurfaceTokenKind::ReservedWord
                    && token.text.as_ref() == "for"
                    && !saw_for =>
            {
                saw_for = true;
            }
            SurfaceNodeKind::Token(token)
                if token.kind == SurfaceTokenKind::Identifier && !saw_for =>
            {
                let spelling = token.text.to_string();
                identifiers.push((spelling, child.range));
            }
            SurfaceNodeKind::Token(token)
                if token.kind == SurfaceTokenKind::ReservedSymbol
                    && token.text.as_ref() == ","
                    && !saw_for => {}
            SurfaceNodeKind::TypeExpression if saw_for && type_expression.is_none() => {
                type_expression = Some(extract_builtin_source_type_expression(
                    ast, child, module, symbols,
                )?);
            }
            _ => return Err(()),
        }
    }

    if !saw_for || identifiers.is_empty() {
        return Err(());
    }
    let type_expression = type_expression.ok_or(())?;
    Ok(identifiers
        .into_iter()
        .map(|(spelling, binding_range)| {
            SourceReserveBindingInput::new(
                spelling,
                binding_range,
                type_expression.range,
                type_expression.spelling.clone(),
                type_expression.head.clone(),
            )
            .with_type_attributes(type_expression.attributes.clone())
        })
        .collect())
}

fn extract_builtin_source_type_expression(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<SourceTypeExpression, ()> {
    if subtree_has_recovery(ast, node) || node.children.is_empty() || node.children.len() > 2 {
        return Err(());
    }
    let (attribute_node, head_id) = match node.children.as_slice() {
        [head] => (None, *head),
        [attributes, head] => (Some(ast.node(*attributes).ok_or(())?), *head),
        _ => return Err(()),
    };
    let attributes = match attribute_node {
        Some(attribute_node) => {
            extract_builtin_source_attributes(ast, attribute_node, module, symbols)?
        }
        None => Vec::new(),
    };
    let head =
        extract_source_reserve_type_head(ast, ast.node(head_id).ok_or(())?, module, symbols)?;
    if !attributes.is_empty()
        && !is_supported_attributed_source_reserve_head(symbols, module, &head)
    {
        return Err(());
    }
    if attributes.iter().any(|attribute| {
        is_imported_fixture_reserve_attribute(symbols, module, &attribute.symbol)
            && !supported_imported_fixture_reserve_attribute_use(symbols, module, &head, attribute)
    }) {
        return Err(());
    }
    Ok(SourceTypeExpression {
        range: node.range,
        spelling: source_text_from_children(ast, node).ok_or(())?,
        head,
        attributes,
    })
}

fn extract_source_reserve_type_head(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<TypeHeadInput, ()> {
    if !matches!(node.kind, SurfaceNodeKind::TypeHead) || node.children.len() != 1 {
        return Err(());
    }
    let child = ast.node(node.children[0]).ok_or(())?;
    if let Some(token) = child.token_text() {
        return match token {
            "set" => Ok(TypeHeadInput::BuiltinSet),
            "object" => Ok(TypeHeadInput::BuiltinObject),
            _ => Err(()),
        };
    }
    if matches!(child.kind, SurfaceNodeKind::QualifiedSymbol) {
        let spelling = qualified_symbol_spelling(ast, child)?;
        let symbol = resolve_visible_type_head(symbols, module, &spelling)?;
        return Ok(TypeHeadInput::Symbol(symbol));
    }
    Err(())
}

fn is_supported_attributed_source_reserve_head(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    head: &TypeHeadInput,
) -> bool {
    match head {
        TypeHeadInput::BuiltinSet | TypeHeadInput::BuiltinObject => true,
        TypeHeadInput::Symbol(symbol) => {
            source_reserve_symbol_head_kind(symbols, module, symbol).is_some()
        }
        _ => false,
    }
}

fn supported_source_reserve_type_head_kind(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> Option<SymbolKind> {
    let entry = symbols.symbols().get(symbol)?;
    if !matches!(entry.kind(), SymbolKind::Mode | SymbolKind::Structure) {
        return None;
    }
    let contribution = symbols.contributions().get(entry.contribution())?;
    if symbol.module() == module
        && contribution.module() == module
        && matches!(contribution.kind(), ContributionKind::LocalSource { .. })
    {
        return Some(entry.kind());
    }
    if symbol.module() != module
        && matches!(entry.kind(), SymbolKind::Mode)
        && contribution.module() == symbol.module()
        && matches!(contribution.kind(), ContributionKind::ImportedSource { .. })
        && symbol.module().path().as_str() == "parser.type_fixtures"
        && entry.primary_spelling() == "TypeCaseMode"
    {
        return Some(entry.kind());
    }
    if symbol.module() != module
        && matches!(entry.kind(), SymbolKind::Structure)
        && contribution.module() == symbol.module()
        && matches!(contribution.kind(), ContributionKind::ImportedSource { .. })
        && symbol.module().path().as_str() == "parser.type_fixtures"
        && matches!(entry.primary_spelling(), "R" | "TypeCaseStruct")
    {
        return Some(entry.kind());
    }
    None
}

fn source_reserve_symbol_head_kind(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> Option<SymbolKind> {
    if symbol.module() != module {
        return None;
    }
    let entry = symbols.symbols().get(symbol)?;
    if !matches!(entry.kind(), SymbolKind::Mode | SymbolKind::Structure) {
        return None;
    }
    let contribution = symbols.contributions().get(entry.contribution())?;
    if contribution.module() != module
        || !matches!(contribution.kind(), ContributionKind::LocalSource { .. })
    {
        return None;
    }
    Some(entry.kind())
}

fn extract_builtin_source_attributes(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<Vec<AttributeInput>, ()> {
    if !matches!(node.kind, SurfaceNodeKind::AttributeChain) || node.children.is_empty() {
        return Err(());
    }
    node.children
        .iter()
        .map(|child_id| {
            let child = ast.node(*child_id).ok_or(())?;
            extract_builtin_source_attribute(ast, child, module, symbols)
        })
        .collect()
}

fn extract_builtin_source_attribute(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<AttributeInput, ()> {
    if !matches!(node.kind, SurfaceNodeKind::AttributeRef) {
        return Err(());
    }
    let mut polarity = AttributePolarity::Positive;
    let mut symbol_spelling = None;
    for child_id in &node.children {
        let child = ast.node(*child_id).ok_or(())?;
        match &child.kind {
            SurfaceNodeKind::Token(token)
                if token.kind == SurfaceTokenKind::ReservedWord
                    && token.text.as_ref() == "non"
                    && symbol_spelling.is_none()
                    && polarity == AttributePolarity::Positive =>
            {
                polarity = AttributePolarity::Negative;
            }
            SurfaceNodeKind::QualifiedSymbol if symbol_spelling.is_none() => {
                symbol_spelling = Some(qualified_symbol_spelling(ast, child)?);
            }
            _ => return Err(()),
        }
    }
    let spelling = symbol_spelling.ok_or(())?;
    let symbol = resolve_visible_attribute(symbols, module, &spelling)?;
    Ok(AttributeInput::new(
        symbol,
        polarity,
        node.range,
        source_text_from_children(ast, node).ok_or(())?,
    ))
}

fn qualified_symbol_spelling(ast: &SurfaceAst, node: &SurfaceNode) -> Result<String, ()> {
    if !matches!(node.kind, SurfaceNodeKind::QualifiedSymbol) || node.children.is_empty() {
        return Err(());
    }
    let mut segments = Vec::new();
    for child_id in &node.children {
        let child = ast.node(*child_id).ok_or(())?;
        if !matches!(child.kind, SurfaceNodeKind::PathSegment) || child.children.len() != 1 {
            return Err(());
        }
        let token = ast
            .node(child.children[0])
            .and_then(SurfaceNode::token_text)
            .ok_or(())?;
        segments.push(token.to_owned());
    }
    Ok(segments.join("."))
}

fn resolve_visible_attribute(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    spelling: &str,
) -> Result<mizar_resolve::resolved_ast::SymbolId, ()> {
    let namespace = NamespacePath::new(module.path().as_str());
    let mut local_candidates = Vec::new();
    let mut imported_candidates = Vec::new();
    for entry in symbols
        .symbols()
        .visible_candidates(&namespace, spelling)
        .into_iter()
        .filter(|entry| matches!(entry.kind(), SymbolKind::Attribute))
        .filter(|entry| supported_source_reserve_attribute(symbols, module, entry.symbol()))
    {
        let symbol = entry.symbol().clone();
        if symbol.module() == module {
            local_candidates.push(symbol);
        } else {
            imported_candidates.push(symbol);
        }
    }
    match local_candidates.as_slice() {
        [symbol] => Ok(symbol.clone()),
        [] => match imported_candidates.as_slice() {
            [symbol] => Ok(symbol.clone()),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

fn resolve_visible_type_head(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    spelling: &str,
) -> Result<mizar_resolve::resolved_ast::SymbolId, ()> {
    let namespace = NamespacePath::new(module.path().as_str());
    let mut local_candidates = Vec::new();
    let mut imported_candidates = Vec::new();
    for entry in symbols
        .symbols()
        .visible_candidates(&namespace, spelling)
        .into_iter()
        .filter(|entry| matches!(entry.kind(), SymbolKind::Mode | SymbolKind::Structure))
        .filter(|entry| {
            supported_source_reserve_type_head_kind(symbols, module, entry.symbol()).is_some()
        })
    {
        let symbol = entry.symbol().clone();
        if symbol.module() == module {
            local_candidates.push(symbol);
        } else {
            imported_candidates.push(symbol);
        }
    }
    match local_candidates.as_slice() {
        [symbol] => Ok(symbol.clone()),
        [] => match imported_candidates.as_slice() {
            [symbol] => Ok(symbol.clone()),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

fn supported_source_reserve_attribute(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> bool {
    let Some(entry) = symbols.symbols().get(symbol) else {
        return false;
    };
    if !matches!(entry.kind(), SymbolKind::Attribute) {
        return false;
    }
    let Some(contribution) = symbols.contributions().get(entry.contribution()) else {
        return false;
    };
    let local_source_attribute = symbol.module() == module
        && contribution.module() == module
        && matches!(contribution.kind(), ContributionKind::LocalSource { .. });
    let imported_fixture_attribute = is_imported_fixture_reserve_attribute(symbols, module, symbol);
    local_source_attribute || imported_fixture_attribute
}

fn is_imported_fixture_reserve_attribute(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> bool {
    imported_fixture_reserve_attribute_spelling(symbols, module, symbol).is_some()
}

fn supported_imported_fixture_reserve_attribute_use(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    head: &TypeHeadInput,
    attribute: &AttributeInput,
) -> bool {
    match imported_fixture_reserve_attribute_spelling(symbols, module, &attribute.symbol) {
        Some("TypeCaseAttr") => matches!(head, TypeHeadInput::BuiltinSet),
        Some("empty") => {
            matches!(head, TypeHeadInput::BuiltinSet)
                && attribute.polarity == AttributePolarity::Negative
        }
        _ => false,
    }
}

fn imported_fixture_reserve_attribute_spelling<'a>(
    symbols: &'a SymbolEnv,
    module: &ResolverModuleId,
    symbol: &mizar_resolve::resolved_ast::SymbolId,
) -> Option<&'a str> {
    let entry = symbols.symbols().get(symbol)?;
    let contribution = symbols.contributions().get(entry.contribution())?;
    if symbol.module() != module
        && contribution.module() == symbol.module()
        && matches!(contribution.kind(), ContributionKind::ImportedSource { .. })
        && symbol.module().path().as_str() == "parser.type_fixtures"
        && matches!(entry.kind(), SymbolKind::Attribute)
        && matches!(entry.primary_spelling(), "TypeCaseAttr" | "empty")
    {
        Some(entry.primary_spelling())
    } else {
        None
    }
}

fn source_text_from_children(ast: &SurfaceAst, node: &SurfaceNode) -> Option<String> {
    let mut tokens = Vec::new();
    collect_token_text(ast, node, &mut tokens)?;
    Some(tokens.join(" "))
}

fn collect_token_text<'a>(
    ast: &'a SurfaceAst,
    node: &'a SurfaceNode,
    output: &mut Vec<&'a str>,
) -> Option<()> {
    if let Some(text) = node.token_text() {
        output.push(text);
        return Some(());
    }
    for child_id in &node.children {
        collect_token_text(ast, ast.node(*child_id)?, output)?;
    }
    Some(())
}

fn merge_optional_range(left: Option<SourceRange>, right: SourceRange) -> SourceRange {
    match left {
        Some(left) => SourceRange {
            source_id: left.source_id,
            start: left.start.min(right.start),
            end: left.end.max(right.end),
        },
        None => right,
    }
}

fn subtree_has_recovery(ast: &SurfaceAst, node: &SurfaceNode) -> bool {
    node.recovered
        || node
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .any(|child| subtree_has_recovery(ast, child))
}

fn assemble_source_reserve_checker_handoff(
    symbols: &SymbolEnv,
    source_reserve: &SourceReserveDeclarationBridge,
    mode_expansions: BTreeMap<ResolverSymbolId, ModeExpansion>,
) -> Result<SourceReserveHandoff, String> {
    let (binding_env, declarations) = source_reserve
        .check_with_mode_expansions(symbols, mode_expansions)
        .map_err(|error| error.to_string())?
        .into_parts();
    let typed_ast = assemble_source_reserve_typed_ast(source_reserve, &declarations)?;
    let resolved = assemble_source_reserve_resolved_typed_ast(&typed_ast, source_reserve)
        .map_err(|error| error.to_string())?;

    Ok(SourceReserveHandoff {
        binding_env,
        declarations,
        typed_ast,
        resolved,
    })
}

fn assemble_source_reserve_resolved_typed_ast(
    typed_ast: &TypedAst,
    source_reserve: &SourceReserveDeclarationBridge,
) -> Result<ResolvedTypedAst, String> {
    let cluster_facts = ClusterFactTable::new();
    let overload_collection = OverloadCollectionOutput::collect(
        Vec::<OverloadSiteInput>::new(),
        Vec::<OverloadCandidateInput>::new(),
    );
    let template_expansion = TemplateExpansionOutput::expand(&overload_collection);
    let viability = CandidateViabilityOutput::filter(
        &template_expansion,
        Vec::<CandidateViabilityInput>::new(),
    );
    let specificity =
        SpecificityGraphOutput::build(&viability, Vec::<SpecificityComparisonInput>::new());
    let overload_selection =
        OverloadSelectionOutput::resolve(&specificity, Vec::<OverloadSiteResolutionInput>::new());
    let expressions = source_reserve
        .bindings()
        .iter()
        .enumerate()
        .map(|(index, _)| ExpressionMetadataInput {
            expr: ExprId::new(format!("source.reserve.declaration.{index}")),
            typed_site: TypedSiteRef::Node(source_reserve.declaration_node(index)),
            local_context: Some(LocalTypeContextId::new(0)),
            cluster_facts: Vec::new(),
        })
        .collect();
    let mut node_hints = Vec::new();
    for (index, _) in source_reserve.bindings().iter().enumerate() {
        node_hints.push(ResolvedNodeKindHint {
            typed_node: source_reserve.type_node(index),
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.reserve.type_expression"),
            },
        });
        node_hints.push(ResolvedNodeKindHint {
            typed_node: source_reserve.declaration_node(index),
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.reserve.declaration"),
            },
        });
    }
    node_hints.push(ResolvedNodeKindHint {
        typed_node: source_reserve.root_node(),
        kind: ResolvedNodeKindHintKind::SourcePreserved {
            role: SourceNodeRole::new("source.reserve.module"),
        },
    });

    ResolvedTypedAst::assemble(ResolvedTypedAstInputs {
        typed_ast,
        cluster_facts: &cluster_facts,
        overload_collection: &overload_collection,
        template_expansion: &template_expansion,
        viability: &viability,
        specificity: &specificity,
        overload_selection: &overload_selection,
        expressions,
        node_hints,
    })
    .map_err(|error| error.to_string())
}

fn assert_source_reserve_handoff(
    handoff: &SourceReserveHandoff,
    source_reserve: &SourceReserveDeclarationBridge,
) -> Result<(), String> {
    let expected_bindings = source_reserve.bindings().len();
    let expected_nodes = expected_bindings * 2 + 1;
    if handoff.resolved.nodes().len() != expected_nodes
        || handoff.resolved.expr_metadata().len() != expected_bindings
        || handoff.declarations.declarations().len() != expected_bindings
    {
        return Err("resolved source reserve count mismatch".to_owned());
    }
    let module_context = handoff
        .binding_env
        .contexts()
        .get(source_reserve.module_context())
        .ok_or_else(|| "missing source reserve module binding context".to_owned())?;
    let expected_binding_ids = (0..expected_bindings)
        .map(BindingId::new)
        .collect::<Vec<_>>();
    if module_context.bindings != expected_binding_ids
        || module_context.visible_bindings != expected_binding_ids
    {
        return Err("source reserve module binding context mismatch".to_owned());
    }
    if handoff.declarations.contexts().len() != 1
        || handoff
            .declarations
            .contexts()
            .get(LocalTypeContextId::new(0))
            .is_none()
    {
        return Err("source reserve local context missing".to_owned());
    }

    for (index, source_binding) in source_reserve.bindings().iter().enumerate() {
        let binding = handoff
            .binding_env
            .bindings()
            .get(BindingId::new(index))
            .ok_or_else(|| format!("missing source reserve binding {index}"))?;
        if binding.spelling != source_binding.spelling
            || binding.kind != BindingKind::ReservedVariable
            || binding.owner_context != source_reserve.module_context()
            || binding.declaration_range != source_binding.binding_range
            || binding.visible_after_ordinal != index
            || binding.type_site != BindingTypeSite::Source(source_binding.type_range)
            || binding.status != BindingStatus::Reserved
        {
            return Err(format!("source reserve binding {index} metadata mismatch"));
        }
        match &binding.identity {
            BinderIdentity::ReservedVariable {
                spelling,
                declaration_range,
            } if spelling == &source_binding.spelling
                && *declaration_range == source_binding.binding_range => {}
            _ => {
                return Err(format!("source reserve binding {index} identity mismatch"));
            }
        }

        let type_node_id = source_reserve.type_node(index);
        let declaration_node_id = source_reserve.declaration_node(index);
        let type_node = handoff
            .resolved
            .nodes()
            .node(ResolvedTypedNodeId::new(type_node_id.index()))
            .ok_or_else(|| format!("missing resolved type node {index}"))?;
        if type_node.source_range != source_binding.type_range {
            return Err(format!("resolved type node {index} source range mismatch"));
        }
        match &type_node.kind {
            ResolvedTypedNodeKind::SourcePreserved { role }
                if role.as_str() == "source.reserve.type_expression" => {}
            _ => return Err(format!("resolved type node {index} source role mismatch")),
        }
        if type_node.final_type.is_none() {
            return Err(format!(
                "resolved type node {index} is missing a final type"
            ));
        }

        let declaration = handoff
            .declarations
            .declarations()
            .iter()
            .map(|(_, declaration)| declaration)
            .find(|declaration| declaration.binding == BindingId::new(index))
            .ok_or_else(|| format!("missing checked declaration {index}"))?;
        if declaration.site != TypedSiteRef::Node(declaration_node_id)
            || declaration.type_site != Some(TypedSiteRef::Node(type_node_id))
            || declaration.type_entry.is_none()
            || declaration.kind != DeclarationKind::ReservedVariable
            || declaration.status != DeclarationStatus::Checked
            || !declaration.deferred.is_empty()
        {
            return Err(format!("checked declaration {index} site mismatch"));
        }
        let typed_declaration = handoff
            .typed_ast
            .nodes()
            .node(declaration_node_id)
            .ok_or_else(|| format!("missing typed declaration node {index}"))?;
        if typed_declaration.links.type_entry != declaration.type_entry
            || typed_declaration.links.context != Some(LocalTypeContextId::new(0))
        {
            return Err(format!("typed declaration node {index} links mismatch"));
        }
        let declaration_node = handoff
            .resolved
            .nodes()
            .node(ResolvedTypedNodeId::new(declaration_node_id.index()))
            .ok_or_else(|| format!("missing resolved declaration node {index}"))?;
        if declaration_node.source_range != source_binding.binding_range {
            return Err(format!(
                "resolved declaration node {index} source range mismatch"
            ));
        }
        match &declaration_node.kind {
            ResolvedTypedNodeKind::SourcePreserved { role }
                if role.as_str() == "source.reserve.declaration" => {}
            _ => return Err(format!("resolved declaration node {index} role mismatch")),
        }
        if declaration_node.final_type.is_none() {
            return Err(format!(
                "resolved declaration node {index} is missing a final type"
            ));
        }
        let expr = ExprId::new(format!("source.reserve.declaration.{index}"));
        let metadata = handoff
            .resolved
            .expr_metadata()
            .get_by_expr(&expr)
            .ok_or_else(|| format!("missing expression metadata {}", expr.as_str()))?;
        if metadata.source_range != source_binding.binding_range {
            return Err(format!(
                "expression metadata {} source range mismatch",
                expr.as_str()
            ));
        }
        if metadata.final_type.is_none() {
            return Err(format!(
                "expression metadata {} is missing a final type",
                expr.as_str()
            ));
        }
    }
    if !handoff.resolved.diagnostics().is_empty() {
        return Err("resolved typed AST produced diagnostics".to_owned());
    }
    Ok(())
}

fn assert_source_reserve_core_summary_readiness(
    handoff: &SourceReserveHandoff,
) -> Result<(), String> {
    let summary = ResolvedTypedAstSummary::from_ast(&handoff.resolved);
    if summary.source_id() != handoff.resolved.source_id() {
        return Err("resolved typed AST summary source mismatch".to_owned());
    }
    if summary.module_id() != handoff.resolved.module_id() {
        return Err("resolved typed AST summary module mismatch".to_owned());
    }
    if !summary.checker_sites().is_empty() {
        return Err("resolved typed AST summary produced checker sites".to_owned());
    }
    Ok(())
}

fn assert_source_reserve_core_context_readiness(
    handoff: &SourceReserveHandoff,
    source_reserve: &SourceReserveDeclarationBridge,
) -> Result<(), String> {
    let summary = ResolvedTypedAstSummary::from_ast(&handoff.resolved);
    let mut input = CoreContextInput::new(summary);

    for (index, source_binding) in source_reserve.bindings().iter().enumerate() {
        let binding_id = BindingId::new(index);
        let binding = handoff
            .binding_env
            .bindings()
            .get(binding_id)
            .ok_or_else(|| format!("missing source reserve binding {index}"))?;
        if binding.kind != BindingKind::ReservedVariable
            || binding.declaration_range != source_binding.binding_range
            || binding.status != BindingStatus::Reserved
        {
            return Err(format!("source reserve binding {index} is not core-ready"));
        }

        let var = CoreVarId::new(binding_id.index());
        let provenance = CheckerOwnedProvenance::checker(format!("source.reserve.binding.{index}"));
        let source = CoreSourceRef::direct(binding.declaration_range)
            .with_provenance(provenance.as_slice().to_vec());
        input.variable_seeds.push(CoreVariableSeed::new(
            var,
            NormalizedVarClass::Free,
            CoreVarRole::new("reserved-variable"),
            NormalizedVarSort::Term,
            provenance.clone(),
        ));
        input
            .binder_seeds
            .push(CoreBinderSeed::new(var, source, provenance));
    }

    let context = prepare_core_context(input).map_err(|error| error.to_string())?;
    if context.source_id() != handoff.resolved.source_id() {
        return Err("core context source mismatch".to_owned());
    }
    if context.module_id() != handoff.resolved.module_id() {
        return Err("core context module mismatch".to_owned());
    }
    if !context.item_registry().items().is_empty()
        || !context.diagnostics().is_empty()
        || !context.worklist().entries().is_empty()
    {
        return Err("core context promoted unsupported work".to_owned());
    }
    if context.binder_sources().iter().count() != source_reserve.bindings().len()
        || context.binder_context().free_variables.len() != source_reserve.bindings().len()
    {
        return Err("core context binding count mismatch".to_owned());
    }

    for (index, source_binding) in source_reserve.bindings().iter().enumerate() {
        let var = CoreVarId::new(index);
        let binder_source = context
            .binder_sources()
            .get(var)
            .ok_or_else(|| format!("missing core binder source {index}"))?;
        if binder_source.source.anchor != CoreSourceRef::direct(source_binding.binding_range).anchor
        {
            return Err(format!("core binder source {index} range mismatch"));
        }
        if binder_source.provenance.as_slice().is_empty() {
            return Err(format!("core binder source {index} provenance missing"));
        }
        if context.binder_context().variable_roles.get(&var)
            != Some(&CoreVarRole::new("reserved-variable"))
            || context.binder_context().variable_sorts.get(&var) != Some(&NormalizedVarSort::Term)
            || !matches!(context.binder_type_facts().get(&var), Some(facts) if facts.is_empty())
        {
            return Err(format!("core binder {index} metadata mismatch"));
        }
    }

    Ok(())
}

#[cfg(test)]
fn assemble_source_checker_handoff(
    symbols: &SymbolEnv,
    source_reserve: &SourceReserveExtraction,
) -> Result<SourceReserveHandoff, String> {
    let handoff = assemble_source_reserve_checker_handoff(
        symbols,
        &source_reserve.bridge,
        source_reserve.mode_expansions.clone(),
    )?;
    assert_source_reserve_handoff(&handoff, &source_reserve.bridge)?;
    assert_source_reserve_core_summary_readiness(&handoff)?;
    assert_source_reserve_core_context_readiness(&handoff, &source_reserve.bridge)?;
    Ok(handoff)
}

fn assemble_source_reserve_typed_ast(
    source_reserve: &SourceReserveDeclarationBridge,
    output: &DeclarationCheckingOutput,
) -> Result<TypedAst, String> {
    if source_reserve.bindings().is_empty() {
        return Err("source reserve bridge produced no bindings".to_owned());
    }
    let declarations_by_binding = output
        .declarations()
        .iter()
        .map(|(_, declaration)| (declaration.binding, declaration))
        .collect::<BTreeMap<_, _>>();
    let mut builder = TypedArenaBuilder::new();
    let mut declaration_nodes = Vec::new();
    for (index, source_binding) in source_reserve.bindings().iter().enumerate() {
        let type_node_id = source_reserve.type_node(index);
        let type_site = TypedSiteRef::Node(type_node_id);
        let type_entry = type_entry_for_site(output.type_entries(), &type_site);
        let pushed = builder
            .push(
                TypedNode::new(
                    "source.reserve.type_expression",
                    SourceAnchor::Range(source_binding.type_range),
                )
                .with_recovery(NodeRecoveryState::Normal)
                .with_typing(typing_for_type_entry(output.type_entries(), type_entry))
                .with_links(TypedNodeLinks {
                    type_entry,
                    ..TypedNodeLinks::default()
                }),
            )
            .map_err(|error| error.to_string())?;
        if pushed != type_node_id {
            return Err(format!("source reserve type node {index} id mismatch"));
        }

        let declaration = declarations_by_binding
            .get(&BindingId::new(index))
            .ok_or_else(|| format!("missing checked source reserve declaration {index}"))?;
        let declaration_node_id = source_reserve.declaration_node(index);
        let declaration_type_entry = declaration.type_entry;
        let pushed = builder
            .push(
                TypedNode::new(
                    "source.reserve.declaration",
                    SourceAnchor::Range(source_binding.binding_range),
                )
                .with_children(vec![type_node_id])
                .with_recovery(NodeRecoveryState::Normal)
                .with_typing(typing_for_type_entry(
                    output.type_entries(),
                    declaration_type_entry,
                ))
                .with_links(TypedNodeLinks {
                    context: Some(LocalTypeContextId::new(0)),
                    type_entry: declaration_type_entry,
                    facts: declaration.facts.clone(),
                    ..TypedNodeLinks::default()
                }),
            )
            .map_err(|error| error.to_string())?;
        if pushed != declaration_node_id {
            return Err(format!(
                "source reserve declaration node {index} id mismatch"
            ));
        }
        declaration_nodes.push(declaration_node_id);
    }

    let root = builder
        .push(
            TypedNode::new(
                "source.reserve.module",
                SourceAnchor::Range(source_reserve.source_range()),
            )
            .with_children(declaration_nodes)
            .with_recovery(NodeRecoveryState::Normal)
            .with_typing(TypingState::Successful)
            .with_links(TypedNodeLinks {
                context: Some(LocalTypeContextId::new(0)),
                ..TypedNodeLinks::default()
            }),
        )
        .map_err(|error| error.to_string())?;
    let nodes = builder
        .finish(Some(root))
        .map_err(|error| error.to_string())?;
    TypedAst::try_new(TypedAstParts {
        source_id: source_reserve.source_id(),
        module_id: source_reserve.module_id().clone(),
        resolved_root: None,
        nodes,
        contexts: output.contexts().clone(),
        types: output.type_entries().clone(),
        facts: output.facts().clone(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: output.diagnostics().clone(),
    })
    .map_err(|error| error.to_string())
}

fn type_entry_for_site(types: &TypeTable, site: &TypedSiteRef) -> Option<TypeEntryId> {
    types
        .iter()
        .find_map(|(entry_id, entry)| (&entry.owner == site).then_some(entry_id))
}

fn typing_for_type_entry(types: &TypeTable, type_entry: Option<TypeEntryId>) -> TypingState {
    type_entry
        .and_then(|entry_id| types.get(entry_id))
        .map_or(TypingState::Unknown, |entry| match entry.status {
            TypeStatus::Known => TypingState::Successful,
            TypeStatus::Assumed => TypingState::Assumed,
            TypeStatus::Unknown => TypingState::Unknown,
            TypeStatus::Error => TypingState::Error,
            TypeStatus::Skipped => TypingState::Skipped,
            _ => TypingState::Unknown,
        })
}

fn resolver_module_id(workspace_root: &Path, source_path: &Path) -> ResolverModuleId {
    ResolverModuleId::new(
        PackageId::new("mizar-test-corpus"),
        ModulePath::new(module_path(workspace_root, source_path)),
    )
}

fn symbol_diagnostic_detail_key(diagnostic: &SymbolDiagnostic) -> String {
    match diagnostic.class() {
        SymbolDiagnosticClass::SameSignatureReturnConflict => {
            "declaration_symbol.signature.same_signature_return_conflict".to_owned()
        }
        class => format!(
            "declaration_symbol.symbol.{}",
            symbol_diagnostic_class_key(class)
        ),
    }
}

fn declaration_symbol_payload_keys(env: &SymbolEnv) -> Vec<String> {
    let mut payloads = Vec::new();
    for symbol in env.symbols().iter() {
        let spelling = declaration_symbol_payload_component(symbol.primary_spelling());
        payloads.push(format!(
            "declaration_symbol.symbol.kind.{spelling}.{}",
            symbol_kind_payload_key(symbol.kind())
        ));
        payloads.push(format!(
            "declaration_symbol.symbol.visibility.{spelling}.{}",
            visibility_payload_key(symbol.visibility())
        ));
        payloads.push(format!(
            "declaration_symbol.symbol.export.{spelling}.{}",
            export_status_payload_key(symbol.export_status())
        ));
        if let Some(definition) = env.definitions().by_symbol(symbol.symbol()) {
            payloads.push(format!(
                "declaration_symbol.definition.kind.{spelling}.{}",
                definition_kind_payload_key(definition.kind())
            ));
            payloads.push(format!(
                "declaration_symbol.definition.visibility.{spelling}.{}",
                visibility_payload_key(definition.visibility())
            ));
        }
    }
    payloads.sort();
    payloads
}

fn declaration_symbol_payload_component(value: &str) -> String {
    let mut escaped = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
            escaped.push(byte as char);
        } else {
            escaped.push('%');
            escaped.push(hex_digit(byte >> 4));
            escaped.push(hex_digit(byte & 0x0f));
        }
    }
    escaped
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'A' + (value - 10)) as char,
        _ => '?',
    }
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

const fn symbol_kind_payload_key(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Predicate => "predicate",
        SymbolKind::Functor => "functor",
        SymbolKind::Mode => "mode",
        SymbolKind::Attribute => "attribute",
        SymbolKind::Structure => "structure",
        SymbolKind::Selector => "selector",
        SymbolKind::Registration => "registration",
        SymbolKind::Theorem => "theorem",
        SymbolKind::Lemma => "lemma",
        SymbolKind::Algorithm => "algorithm",
        SymbolKind::Scheme => "scheme",
        SymbolKind::Template => "template",
        SymbolKind::Synonym => "synonym",
        SymbolKind::Antonym => "antonym",
        SymbolKind::Redefinition => "redefinition",
        SymbolKind::Builtin => "builtin",
        _ => "unknown",
    }
}

const fn definition_kind_payload_key(kind: DefinitionKind) -> &'static str {
    match kind {
        DefinitionKind::Predicate => "predicate",
        DefinitionKind::Functor => "functor",
        DefinitionKind::Mode => "mode",
        DefinitionKind::Attribute => "attribute",
        DefinitionKind::Structure => "structure",
        DefinitionKind::Registration => "registration",
        DefinitionKind::Theorem => "theorem",
        DefinitionKind::Lemma => "lemma",
        DefinitionKind::Algorithm => "algorithm",
        DefinitionKind::Scheme => "scheme",
        DefinitionKind::Template => "template",
        DefinitionKind::Synonym => "synonym",
        DefinitionKind::Antonym => "antonym",
        DefinitionKind::Redefinition => "redefinition",
        DefinitionKind::Selector => "selector",
        _ => "unknown",
    }
}

const fn visibility_payload_key(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "private",
        Visibility::Public => "public",
        _ => "unknown",
    }
}

const fn export_status_payload_key(status: ExportStatus) -> &'static str {
    match status {
        ExportStatus::LocalOnly => "local_only",
        ExportStatus::Exported => "exported",
        ExportStatus::ReExported => "re_exported",
        _ => "unknown",
    }
}

fn expected_declaration_symbol_detail_keys(case: &TestCase) -> Vec<String> {
    if !case.expectation.diagnostic_payloads.is_empty() {
        return case.expectation.diagnostic_payloads.clone();
    }
    case.expectation.stable_detail_key.iter().cloned().collect()
}

fn expected_declaration_symbol_payload_keys(case: &TestCase) -> Vec<String> {
    let mut payloads = case.expectation.declaration_symbol_payloads.clone();
    payloads.sort();
    payloads
}

fn expected_type_elaboration_detail_keys(case: &TestCase) -> Vec<String> {
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
            "declaration-symbol case `{}` expected detail keys {:?} but got {:?}; expected payload keys {:?} but got {:?}",
            case.id.0,
            expected_declaration_symbol_detail_keys(case),
            result.actual_detail_keys,
            expected_declaration_symbol_payload_keys(case),
            result.actual_payload_keys
        ),
    )
}

fn type_elaboration_failure_diagnostic(
    case: &TestCase,
    result: &TypeElaborationCaseResult,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        &case.expectation_path,
        "type_elaboration",
        "E-TYPE-ELABORATION-ASSERT",
        format!("type_elaboration.{}", case.id.0),
        format!(
            "type-elaboration case `{}` expected detail keys {:?} but got {:?}",
            case.id.0,
            expected_type_elaboration_detail_keys(case),
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

impl fmt::Display for TypeElaborationCaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ParseOnlyImportProvider, TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY,
        assemble_source_checker_handoff, extract_builtin_source_reserve_declarations,
        resolve_visible_attribute, resolve_visible_type_head, source_type_elaboration_detail_keys,
    };
    use mizar_checker::binding_env::BindingId;
    use mizar_checker::resolved_typed_ast::{ResolvedTypedNodeId, ResolvedTypedNodeKind};
    use mizar_checker::type_checker::{AttributePolarity, TypeHeadInput};
    use mizar_checker::typed_ast::LocalTypeContextId;
    use mizar_core::elaborator::ResolvedTypedAstSummary;
    use mizar_frontend::lexical_env::{
        ExportedOperatorAssociativity, ExportedOperatorFixity, ExportedOperatorMetadata,
        LexicalEnvironmentRequest, LexicalSummaryProvider, UserSymbolKind,
    };
    use mizar_frontend::preprocess::{ImportStub, ImportStubPath};
    use mizar_resolve::env::{
        ContributionKind, ExportStatus, NamespacePath, SymbolEntry, SymbolEnv, SymbolEnvIndexes,
        SymbolKind, Visibility,
    };
    use mizar_resolve::resolved_ast::{
        FullyQualifiedName, LocalSymbolId, ModuleId as ResolverModuleId, SemanticOrigin,
        SymbolId as ResolverSymbolId,
    };
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SourceAnchor, SourceId, SourceRange,
    };
    use mizar_syntax::recovery::SyntaxRecoveryKind;
    use mizar_syntax::{
        SurfaceAst, SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceNodeKind, SurfaceTokenKind,
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

    #[test]
    fn source_reserve_extractor_preserves_builtin_declarations_and_rejects_gaps() {
        let builtin_source_id = source_id(91);
        let ast = reserve_ast(
            builtin_source_id,
            vec![
                reserve_item(vec!["x", "y"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["z"], ReserveTypeShape::Builtin("object")),
            ],
        );
        let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));

        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let source_reserve =
            extract_builtin_source_reserve_declarations(&ast, module.clone(), &symbols)
                .expect("builtin reserve declarations should extract");

        assert_eq!(
            source_reserve
                .bindings()
                .iter()
                .map(|binding| (
                    binding.spelling.as_str(),
                    binding.type_spelling.as_str(),
                    &binding.type_head,
                    binding.type_range
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    "x",
                    "set",
                    &TypeHeadInput::BuiltinSet,
                    range(builtin_source_id, 18, 21),
                ),
                (
                    "y",
                    "set",
                    &TypeHeadInput::BuiltinSet,
                    range(builtin_source_id, 18, 21),
                ),
                (
                    "z",
                    "object",
                    &TypeHeadInput::BuiltinObject,
                    range(builtin_source_id, 38, 44),
                ),
            ]
        );

        let non_builtin = reserve_ast(
            source_id(92),
            vec![reserve_item(vec!["x"], ReserveTypeShape::NonBuiltin("T"))],
        );

        assert!(
            extract_builtin_source_reserve_declarations(&non_builtin, module.clone(), &symbols)
                .is_err(),
            "non-builtin type heads must stay on the external gap"
        );

        let attributed_symbols = source_symbol_env(module.clone());
        let unsupported = reserve_ast(
            source_id(93),
            vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
        );

        let attributed = extract_builtin_source_reserve_declarations(
            &unsupported,
            module.clone(),
            &attributed_symbols,
        )
        .expect("attribute-bearing builtin reserve type should extract");
        assert_eq!(attributed.bindings().len(), 1);
        assert_eq!(attributed.bindings()[0].type_attributes.len(), 1);
        assert_eq!(
            attributed.bindings()[0].type_attributes[0].polarity,
            AttributePolarity::Negative
        );

        let local_mode = reserve_ast(
            source_id(94),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("Mode"),
            )],
        );
        let mode_symbols = source_mode_symbol_env(module.clone());
        let local_mode_reserve =
            extract_builtin_source_reserve_declarations(&local_mode, module.clone(), &mode_symbols)
                .expect("same-module mode reserve type should extract");
        assert_eq!(local_mode_reserve.bindings().len(), 1);
        assert!(matches!(
            local_mode_reserve.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));

        let attributed_mode = reserve_ast(
            source_id(194),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
            )],
        );
        let mode_attribute_symbols = source_mode_attribute_symbol_env(module.clone());
        let attributed_mode_reserve = extract_builtin_source_reserve_declarations(
            &attributed_mode,
            mode_attribute_symbols.module_id().clone(),
            &mode_attribute_symbols,
        )
        .expect("same-module attributed mode reserve type should extract");
        assert_eq!(attributed_mode_reserve.bindings().len(), 1);
        assert_eq!(
            attributed_mode_reserve.bindings()[0].type_attributes.len(),
            1
        );
        assert_eq!(
            attributed_mode_reserve.bindings()[0].type_attributes[0].polarity,
            AttributePolarity::Negative
        );
        assert!(matches!(
            attributed_mode_reserve.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));

        let local_structure = reserve_ast(
            source_id(195),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
        );
        let structure_symbols = source_structure_symbol_env(local_mode_reserve.module_id().clone());
        let local_structure_reserve = extract_builtin_source_reserve_declarations(
            &local_structure,
            structure_symbols.module_id().clone(),
            &structure_symbols,
        )
        .expect("same-module structure reserve type should extract");
        assert_eq!(local_structure_reserve.bindings().len(), 1);
        assert!(matches!(
            local_structure_reserve.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));

        let attributed_structure = reserve_ast(
            source_id(196),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbol("Struct"),
            )],
        );
        let structure_attribute_symbols =
            source_structure_attribute_symbol_env(local_structure_reserve.module_id().clone());
        let attributed_structure_reserve = extract_builtin_source_reserve_declarations(
            &attributed_structure,
            structure_attribute_symbols.module_id().clone(),
            &structure_attribute_symbols,
        )
        .expect("same-module attributed structure reserve type should extract");
        assert_eq!(attributed_structure_reserve.bindings().len(), 1);
        assert_eq!(
            attributed_structure_reserve.bindings()[0]
                .type_attributes
                .len(),
            1
        );
        assert!(matches!(
            attributed_structure_reserve.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));
    }

    #[test]
    fn source_reserve_extractor_preserves_local_mode_expansion_chain_payloads() {
        let source = source_id(199);
        let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let symbols = source_mode_chain_symbol_env(module.clone());
        let ast = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("B"),
            )],
        );

        let extraction =
            extract_builtin_source_reserve_declarations(&ast, module.clone(), &symbols)
                .expect("mode chain reserve should extract");
        let a = resolve_visible_type_head(&symbols, &module, "A").unwrap();
        let b = resolve_visible_type_head(&symbols, &module, "B").unwrap();
        assert_eq!(extraction.mode_expansions().len(), 2);
        assert!(matches!(
            extraction
                .mode_expansions()
                .get(&a)
                .map(|expansion| &expansion.radix.head),
            Some(TypeHeadInput::BuiltinSet)
        ));
        assert_eq!(
            extraction
                .mode_expansions()
                .get(&b)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(a.clone()))
        );

        let attributed_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let attributed_extraction = extract_builtin_source_reserve_declarations(
            &attributed_dependency,
            module.clone(),
            &symbols,
        )
        .expect("attributed dependency reserve should still extract");
        assert!(
            attributed_extraction.mode_expansions().is_empty(),
            "any attributed reserve use in a local-mode chain withholds the chain expansion"
        );

        let attributed_root_symbols = source_mode_attribute_symbol_env(module.clone());
        let attributed_root = mode_chain_reserve_ast(
            source,
            [mode_definition("Mode", ReserveTypeShape::Builtin("set"))],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
            )],
        );
        let attributed_root_extraction = extract_builtin_source_reserve_declarations(
            &attributed_root,
            module.clone(),
            &attributed_root_symbols,
        )
        .expect("single attributed local-mode reserve should still extract");
        let attributed_root_mode =
            resolve_visible_type_head(&attributed_root_symbols, &module, "Mode").unwrap();
        assert_eq!(attributed_root_extraction.mode_expansions().len(), 1);
        assert!(matches!(
            attributed_root_extraction
                .mode_expansions()
                .get(&attributed_root_mode)
                .map(|expansion| &expansion.radix.head),
            Some(TypeHeadInput::BuiltinSet)
        ));

        let mixed_attributed_root = mode_chain_reserve_ast(
            source,
            [mode_definition("Mode", ReserveTypeShape::Builtin("set"))],
            vec![
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("Mode")),
                reserve_item(
                    vec!["y"],
                    ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
                ),
            ],
        );
        let mixed_attributed_root_extraction = extract_builtin_source_reserve_declarations(
            &mixed_attributed_root,
            module.clone(),
            &attributed_root_symbols,
        )
        .expect("mixed local-mode reserve should still extract");
        assert!(
            mixed_attributed_root_extraction
                .mode_expansions()
                .is_empty(),
            "mixed bare/attributed uses of the same local mode still withhold expansion"
        );

        let structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let structure_rhs_symbols = source_local_symbols_env(
            structure_rhs_module.clone(),
            &[
                ("Struct", SymbolKind::Structure),
                ("Mode", SymbolKind::Mode),
            ],
        );
        let structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [mode_definition(
                "Mode",
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("Mode"),
            )],
        );
        let structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &structure_rhs,
            structure_rhs_module.clone(),
            &structure_rhs_symbols,
        )
        .expect("mode with structure RHS reserve should extract");
        let structure_rhs_mode =
            resolve_visible_type_head(&structure_rhs_symbols, &structure_rhs_module, "Mode")
                .unwrap();
        let structure_rhs_struct =
            resolve_visible_type_head(&structure_rhs_symbols, &structure_rhs_module, "Struct")
                .unwrap();
        assert_eq!(structure_rhs_extraction.mode_expansions().len(), 1);
        assert_eq!(
            structure_rhs_extraction
                .mode_expansions()
                .get(&structure_rhs_mode)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(structure_rhs_struct))
        );

        let attributed_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let attributed_structure_rhs_symbols = source_local_symbols_env(
            attributed_structure_rhs_module.clone(),
            &[
                ("empty", SymbolKind::Attribute),
                ("Struct", SymbolKind::Structure),
                ("Mode", SymbolKind::Mode),
            ],
        );
        let attributed_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [mode_definition(
                "Mode",
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
            )],
        );
        let attributed_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &attributed_structure_rhs,
            attributed_structure_rhs_module.clone(),
            &attributed_structure_rhs_symbols,
        )
        .expect("attributed mode with structure RHS reserve should extract");
        let attributed_structure_rhs_mode = resolve_visible_type_head(
            &attributed_structure_rhs_symbols,
            &attributed_structure_rhs_module,
            "Mode",
        )
        .unwrap();
        let attributed_structure_rhs_struct = resolve_visible_type_head(
            &attributed_structure_rhs_symbols,
            &attributed_structure_rhs_module,
            "Struct",
        )
        .unwrap();
        assert_eq!(
            attributed_structure_rhs_extraction.mode_expansions().len(),
            1
        );
        assert_eq!(
            attributed_structure_rhs_extraction
                .mode_expansions()
                .get(&attributed_structure_rhs_mode)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(attributed_structure_rhs_struct))
        );

        let mixed_attributed_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [mode_definition(
                "Mode",
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("Mode")),
                reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
                ),
            ],
        );
        let mixed_attributed_structure_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &mixed_attributed_structure_rhs,
                attributed_structure_rhs_module.clone(),
                &attributed_structure_rhs_symbols,
            )
            .expect("mixed attributed structure-RHS reserve should still extract");
        assert!(
            mixed_attributed_structure_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "mixed bare/attributed uses still withhold direct structure-RHS expansion"
        );

        let attributed_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let attributed_rhs_symbols =
            source_mode_attribute_symbol_env(attributed_rhs_module.clone());
        let attributed_rhs = mode_chain_reserve_ast(
            source,
            [mode_definition("Mode", ReserveTypeShape::AttributedSet)],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("Mode"),
            )],
        );
        let attributed_rhs_extraction = extract_builtin_source_reserve_declarations(
            &attributed_rhs,
            attributed_rhs_module.clone(),
            &attributed_rhs_symbols,
        )
        .expect("mode with attributed builtin RHS reserve should extract");
        let attributed_rhs_mode =
            resolve_visible_type_head(&attributed_rhs_symbols, &attributed_rhs_module, "Mode")
                .unwrap();
        assert_eq!(attributed_rhs_extraction.mode_expansions().len(), 1);
        let attributed_rhs_expansion = attributed_rhs_extraction
            .mode_expansions()
            .get(&attributed_rhs_mode)
            .expect("attributed RHS mode expansion should be present");
        assert!(matches!(
            attributed_rhs_expansion.radix.head,
            TypeHeadInput::BuiltinSet
        ));
        assert_eq!(attributed_rhs_expansion.attributes.len(), 1);
        assert_eq!(
            attributed_rhs_expansion.attributes[0].polarity,
            AttributePolarity::Negative
        );

        let attributed_root_attributed_rhs = mode_chain_reserve_ast(
            source,
            [mode_definition("Mode", ReserveTypeShape::AttributedSet)],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
            )],
        );
        let attributed_root_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_root_attributed_rhs,
                attributed_rhs_module.clone(),
                &attributed_rhs_symbols,
            )
            .expect("attributed mode with attributed builtin RHS reserve should extract");
        assert_eq!(
            attributed_root_attributed_rhs_extraction.bindings()[0]
                .type_attributes
                .len(),
            1
        );
        assert_eq!(
            attributed_root_attributed_rhs_extraction
                .mode_expansions()
                .len(),
            1
        );
        let attributed_root_attributed_rhs_expansion = attributed_root_attributed_rhs_extraction
            .mode_expansions()
            .get(&attributed_rhs_mode)
            .expect("attributed-root attributed RHS expansion should be present");
        assert!(matches!(
            attributed_root_attributed_rhs_expansion.radix.head,
            TypeHeadInput::BuiltinSet
        ));
        assert_eq!(attributed_root_attributed_rhs_expansion.attributes.len(), 1);

        let attributed_object_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let attributed_object_rhs_symbols = source_local_symbols_env(
            attributed_object_rhs_module.clone(),
            &[
                ("empty", SymbolKind::Attribute),
                ("ObjectMode", SymbolKind::Mode),
            ],
        );
        let attributed_root_attributed_object_rhs = mode_chain_reserve_ast(
            source,
            [mode_definition(
                "ObjectMode",
                ReserveTypeShape::AttributedObject,
            )],
            vec![reserve_item(
                vec!["w"],
                ReserveTypeShape::AttributedQualifiedSymbol("ObjectMode"),
            )],
        );
        let attributed_root_attributed_object_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_root_attributed_object_rhs,
                attributed_object_rhs_module.clone(),
                &attributed_object_rhs_symbols,
            )
            .expect("attributed mode with attributed object RHS reserve should extract");
        let attributed_object_rhs_mode = resolve_visible_type_head(
            &attributed_object_rhs_symbols,
            &attributed_object_rhs_module,
            "ObjectMode",
        )
        .unwrap();
        let attributed_root_attributed_object_rhs_expansion =
            attributed_root_attributed_object_rhs_extraction
                .mode_expansions()
                .get(&attributed_object_rhs_mode)
                .expect("attributed-root attributed object RHS expansion should be present");
        assert!(matches!(
            attributed_root_attributed_object_rhs_expansion.radix.head,
            TypeHeadInput::BuiltinObject
        ));
        assert_eq!(
            attributed_root_attributed_object_rhs_expansion
                .attributes
                .len(),
            1
        );

        let mixed_attributed_root_attributed_rhs = mode_chain_reserve_ast(
            source,
            [mode_definition("Mode", ReserveTypeShape::AttributedSet)],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("Mode")),
                reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
                ),
            ],
        );
        let mixed_attributed_root_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &mixed_attributed_root_attributed_rhs,
                attributed_rhs_module.clone(),
                &attributed_rhs_symbols,
            )
            .expect("mixed attributed-root attributed RHS reserve should still extract");
        assert!(
            mixed_attributed_root_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "mixed bare/attributed uses still withhold direct attributed-RHS expansion"
        );

        let attributed_chain_attributed_rhs_symbols = source_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
        );
        let attributed_chain_attributed_rhs_module =
            attributed_chain_attributed_rhs_symbols.module_id().clone();
        let attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("attributed chain ending in attributed RHS should still extract");
        let attributed_chain_attributed_b = resolve_visible_type_head(
            &attributed_chain_attributed_rhs_symbols,
            &attributed_chain_attributed_rhs_module,
            "B",
        )
        .unwrap();
        let attributed_chain_attributed_a = resolve_visible_type_head(
            &attributed_chain_attributed_rhs_symbols,
            &attributed_chain_attributed_rhs_module,
            "A",
        )
        .unwrap();
        assert_eq!(
            attributed_chain_attributed_rhs_extraction.bindings()[0]
                .type_attributes
                .len(),
            1
        );
        assert_eq!(
            attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .len(),
            2
        );
        let attributed_chain_attributed_b_expansion = attributed_chain_attributed_rhs_extraction
            .mode_expansions()
            .get(&attributed_chain_attributed_b)
            .expect("terminal attributed RHS expansion should be present");
        assert!(matches!(
            attributed_chain_attributed_b_expansion.radix.head,
            TypeHeadInput::BuiltinSet
        ));
        assert_eq!(attributed_chain_attributed_b_expansion.attributes.len(), 1);
        assert_eq!(
            attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_attributed_a)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(
                attributed_chain_attributed_b.clone()
            ))
        );

        let attributed_chain_attributed_object_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedObject),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_attributed_object_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_chain_attributed_object_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("attributed chain ending in attributed object RHS should still extract");
        let attributed_chain_attributed_object_b_expansion =
            attributed_chain_attributed_object_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_attributed_b)
                .expect("terminal attributed object RHS expansion should be present");
        assert!(matches!(
            attributed_chain_attributed_object_b_expansion.radix.head,
            TypeHeadInput::BuiltinObject
        ));
        assert_eq!(
            attributed_chain_attributed_object_b_expansion
                .attributes
                .len(),
            1
        );
        assert!(
            attributed_chain_attributed_object_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_attributed_a)
        );

        let mixed_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let mixed_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &mixed_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("mixed attributed-RHS chain reserve should still extract");
        assert!(
            mixed_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds mixed bare/attributed uses of the attributed root"
        );

        let attributed_dependency_for_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::AttributedQualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let attributed_dependency_for_attributed_rhs_chain_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_dependency_for_attributed_rhs_chain,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("attributed dependency attributed-RHS chain reserve should still extract");
        assert!(
            !attributed_dependency_for_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_attributed_a),
            "Task66 withholds attributed dependency modes"
        );

        let deeper_attributed_chain_attributed_rhs_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("empty", SymbolKind::Attribute),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
            ],
        );
        let deeper_attributed_chain_attributed_rhs_module =
            deeper_attributed_chain_attributed_rhs_symbols
                .module_id()
                .clone();
        let deeper_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("C")),
            ],
        );
        let deeper_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &deeper_attributed_chain_attributed_rhs,
                deeper_attributed_chain_attributed_rhs_module.clone(),
                &deeper_attributed_chain_attributed_rhs_symbols,
            )
            .expect("deeper attributed-RHS attributed chain reserve should still extract");
        let deeper_attributed_chain_attributed_c = resolve_visible_type_head(
            &deeper_attributed_chain_attributed_rhs_symbols,
            &deeper_attributed_chain_attributed_rhs_module,
            "C",
        )
        .unwrap();
        assert!(
            !deeper_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_chain_attributed_c),
            "Task66 remains capped at one dependency edge"
        );

        let duplicate_root_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let duplicate_root_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_root_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("duplicate-root attributed RHS chain reserve should still extract");
        assert!(
            duplicate_root_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 requires a unique root mode definition"
        );

        let duplicate_terminal_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let duplicate_terminal_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_terminal_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("duplicate-terminal attributed RHS chain reserve should still extract");
        assert!(
            duplicate_terminal_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 requires a unique terminal mode definition"
        );

        let forward_terminal_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("B", ReserveTypeShape::AttributedSet),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let forward_terminal_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &forward_terminal_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("forward terminal attributed RHS chain reserve should still extract");
        assert!(
            forward_terminal_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 requires source order B -> A -> reserve"
        );

        let attribute_args_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSetWithAttributeArgs),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attribute_args_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attribute_args_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("attribute-argument attributed RHS chain reserve should still extract");
        assert!(
            attribute_args_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds terminal attributed RHS attribute arguments"
        );

        let argument_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbolWithArgs("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let argument_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &argument_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("argument-bearing attributed RHS chain reserve should still extract");
        assert!(
            argument_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds argument-bearing dependency RHSs"
        );

        let contextual_root_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                contextual_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let contextual_root_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_root_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("contextual root attributed RHS chain reserve should still extract");
        assert!(
            contextual_root_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds contextual root definitions"
        );

        let contextual_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                contextual_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let contextual_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("contextual attributed RHS chain reserve should still extract");
        assert!(
            contextual_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds contextual dependency definitions"
        );

        let parameterized_root_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                parameterized_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let parameterized_root_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &parameterized_root_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("parameterized root attributed RHS chain reserve should still extract");
        assert!(
            parameterized_root_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds parameterized root definitions"
        );

        let parameterized_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                parameterized_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let parameterized_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &parameterized_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("parameterized attributed RHS chain reserve should still extract");
        assert!(
            parameterized_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds parameterized dependency definitions"
        );

        let recovered_root_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                recovered_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let recovered_root_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &recovered_root_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("recovered root attributed RHS chain reserve should still extract");
        assert!(
            recovered_root_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds recovered root definitions"
        );

        let recovered_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                recovered_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let recovered_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &recovered_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("recovered attributed RHS chain reserve should still extract");
        assert!(
            recovered_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds recovered dependency definitions"
        );

        let imported_attribute_attributed_chain_attributed_rhs_symbols =
            imported_attribute_mode_chain_symbol_env(ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("bridge"),
            ));
        let imported_attribute_attributed_chain_attributed_rhs_module =
            imported_attribute_attributed_chain_attributed_rhs_symbols
                .module_id()
                .clone();
        let imported_attribute_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &imported_attribute_attributed_chain_attributed_rhs,
                imported_attribute_attributed_chain_attributed_rhs_module,
                &imported_attribute_attributed_chain_attributed_rhs_symbols,
            )
            .is_err(),
            "Task66 withholds imported reserve-head or terminal RHS attributes"
        );

        let ambiguous_attribute_attributed_chain_attributed_rhs_symbols =
            ambiguous_attribute_mode_chain_symbol_env(ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("bridge"),
            ));
        let ambiguous_attribute_attributed_chain_attributed_rhs_module =
            ambiguous_attribute_attributed_chain_attributed_rhs_symbols
                .module_id()
                .clone();
        let ambiguous_attribute_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &ambiguous_attribute_attributed_chain_attributed_rhs,
                ambiguous_attribute_attributed_chain_attributed_rhs_module,
                &ambiguous_attribute_attributed_chain_attributed_rhs_symbols,
            )
            .is_err(),
            "Task66 withholds ambiguous reserve-head or terminal RHS attributes"
        );

        let imported_attributed_chain_attributed_rhs_root_symbols = imported_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            "A",
        );
        let imported_attributed_chain_attributed_rhs_root_module =
            imported_attributed_chain_attributed_rhs_root_symbols
                .module_id()
                .clone();
        let imported_attributed_chain_attributed_rhs_root = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &imported_attributed_chain_attributed_rhs_root,
                imported_attributed_chain_attributed_rhs_root_module,
                &imported_attributed_chain_attributed_rhs_root_symbols,
            )
            .is_err(),
            "Task66 withholds imported attributed roots"
        );

        let imported_attributed_chain_attributed_rhs_dependency_symbols =
            imported_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "B",
            );
        let imported_attributed_chain_attributed_rhs_dependency_module =
            imported_attributed_chain_attributed_rhs_dependency_symbols
                .module_id()
                .clone();
        let imported_attributed_chain_attributed_rhs_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let imported_attributed_chain_attributed_rhs_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &imported_attributed_chain_attributed_rhs_dependency,
                imported_attributed_chain_attributed_rhs_dependency_module,
                &imported_attributed_chain_attributed_rhs_dependency_symbols,
            )
            .expect("imported dependency attributed RHS chain reserve should still extract");
        assert!(
            imported_attributed_chain_attributed_rhs_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds imported dependency modes"
        );

        let ambiguous_attributed_chain_attributed_rhs_root_symbols =
            ambiguous_attributed_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "A",
            );
        let ambiguous_attributed_chain_attributed_rhs_root_module =
            ambiguous_attributed_chain_attributed_rhs_root_symbols
                .module_id()
                .clone();
        let ambiguous_attributed_chain_attributed_rhs_root = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &ambiguous_attributed_chain_attributed_rhs_root,
                ambiguous_attributed_chain_attributed_rhs_root_module,
                &ambiguous_attributed_chain_attributed_rhs_root_symbols,
            )
            .is_err(),
            "Task66 withholds ambiguous attributed roots"
        );

        let ambiguous_attributed_chain_attributed_rhs_dependency_symbols =
            ambiguous_attributed_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "B",
            );
        let ambiguous_attributed_chain_attributed_rhs_dependency_module =
            ambiguous_attributed_chain_attributed_rhs_dependency_symbols
                .module_id()
                .clone();
        let ambiguous_attributed_chain_attributed_rhs_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let ambiguous_attributed_chain_attributed_rhs_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &ambiguous_attributed_chain_attributed_rhs_dependency,
                ambiguous_attributed_chain_attributed_rhs_dependency_module,
                &ambiguous_attributed_chain_attributed_rhs_dependency_symbols,
            )
            .expect("ambiguous dependency attributed RHS chain reserve should still extract");
        assert!(
            ambiguous_attributed_chain_attributed_rhs_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds ambiguous dependency modes"
        );

        let chain_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let chain_structure_rhs_symbols = source_local_symbols_env(
            chain_structure_rhs_module.clone(),
            &[
                ("Struct", SymbolKind::Structure),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
            ],
        );
        let chain_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let chain_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &chain_structure_rhs,
            chain_structure_rhs_module.clone(),
            &chain_structure_rhs_symbols,
        )
        .expect("mode chain ending in structure RHS reserve should still extract");
        let chain_structure_b = resolve_visible_type_head(
            &chain_structure_rhs_symbols,
            &chain_structure_rhs_module,
            "B",
        )
        .unwrap();
        let chain_structure_a = resolve_visible_type_head(
            &chain_structure_rhs_symbols,
            &chain_structure_rhs_module,
            "A",
        )
        .unwrap();
        let chain_structure_struct = resolve_visible_type_head(
            &chain_structure_rhs_symbols,
            &chain_structure_rhs_module,
            "Struct",
        )
        .unwrap();
        assert_eq!(chain_structure_rhs_extraction.mode_expansions().len(), 2);
        assert_eq!(
            chain_structure_rhs_extraction
                .mode_expansions()
                .get(&chain_structure_b)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(chain_structure_struct))
        );
        assert_eq!(
            chain_structure_rhs_extraction
                .mode_expansions()
                .get(&chain_structure_a)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(chain_structure_b))
        );

        let attributed_chain_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let attributed_chain_structure_rhs_symbols = source_local_symbols_env(
            attributed_chain_structure_rhs_module.clone(),
            &[
                ("empty", SymbolKind::Attribute),
                ("Struct", SymbolKind::Structure),
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
            ],
        );
        let attributed_chain_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_structure_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_chain_structure_rhs,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("attributed mode chain ending in structure RHS should still extract");
        let attributed_chain_structure_b = resolve_visible_type_head(
            &attributed_chain_structure_rhs_symbols,
            &attributed_chain_structure_rhs_module,
            "B",
        )
        .unwrap();
        let attributed_chain_structure_a = resolve_visible_type_head(
            &attributed_chain_structure_rhs_symbols,
            &attributed_chain_structure_rhs_module,
            "A",
        )
        .unwrap();
        let attributed_chain_structure_struct = resolve_visible_type_head(
            &attributed_chain_structure_rhs_symbols,
            &attributed_chain_structure_rhs_module,
            "Struct",
        )
        .unwrap();
        assert_eq!(
            attributed_chain_structure_rhs_extraction.bindings()[0]
                .type_attributes
                .len(),
            1
        );
        assert_eq!(
            attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .len(),
            2
        );
        assert_eq!(
            attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_structure_b)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(attributed_chain_structure_struct))
        );
        assert_eq!(
            attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_structure_a)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(attributed_chain_structure_b.clone()))
        );

        let cached_attributed_chain_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let cached_attributed_chain_structure_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &cached_attributed_chain_structure_rhs,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("cached attributed structure-RHS chain reserve should still extract");
        assert!(
            cached_attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_structure_b)
        );
        assert!(
            cached_attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_structure_a),
            "cached direct structure-RHS payload may feed the attributed one-edge chain"
        );

        let mixed_attributed_chain_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let mixed_attributed_chain_structure_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &mixed_attributed_chain_structure_rhs,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("mixed attributed structure-RHS chain reserve should still extract");
        assert!(
            mixed_attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "mixed bare/attributed uses of the attributed structure-chain root withhold expansion"
        );

        let attributed_dependency_for_structure_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::AttributedQualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let attributed_dependency_for_structure_chain_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_dependency_for_structure_chain,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("attributed dependency structure-RHS chain reserve should still extract");
        assert!(
            !attributed_dependency_for_structure_chain_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_structure_a),
            "an attributed dependency mode stays outside the attributed-root structure chain"
        );

        let deeper_attributed_chain_structure_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("empty", SymbolKind::Attribute),
                ("Struct", SymbolKind::Structure),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
            ],
        );
        let deeper_attributed_chain_structure_module = deeper_attributed_chain_structure_symbols
            .module_id()
            .clone();
        let deeper_attributed_chain_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("C")),
            ],
        );
        let deeper_attributed_chain_structure_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &deeper_attributed_chain_structure_rhs,
                deeper_attributed_chain_structure_module.clone(),
                &deeper_attributed_chain_structure_symbols,
            )
            .expect("deeper attributed structure-RHS chain reserve should still extract");
        let deeper_attributed_chain_structure_c = resolve_visible_type_head(
            &deeper_attributed_chain_structure_symbols,
            &deeper_attributed_chain_structure_module,
            "C",
        )
        .unwrap();
        assert!(
            !deeper_attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_chain_structure_c),
            "attributed local-mode structure chains remain capped at one dependency edge"
        );

        let duplicate_attributed_chain_structure_dependency =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let duplicate_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("duplicate dependency structure-RHS chain reserve should still extract");
        assert!(
            duplicate_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 requires a unique dependency mode definition"
        );

        let duplicate_attributed_chain_structure_root = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let duplicate_attributed_chain_structure_root_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_attributed_chain_structure_root,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("duplicate root structure-RHS chain reserve should still extract");
        assert!(
            duplicate_attributed_chain_structure_root_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 requires a unique attributed root mode definition"
        );

        let duplicate_attributed_chain_structure_definition =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct", "Struct"],
                [
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let duplicate_attributed_chain_structure_definition_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_attributed_chain_structure_definition,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("duplicate structure definition reserve should still extract");
        assert!(
            duplicate_attributed_chain_structure_definition_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 requires a unique terminal structure definition"
        );

        let forward_attributed_chain_structure_dependency = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let forward_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &forward_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("forward dependency structure-RHS chain reserve should still extract");
        assert!(
            forward_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 requires the dependency definition to precede the attributed root definition"
        );

        let forward_attributed_chain_structure_terminal =
            mode_chain_reserve_ast_with_trailing_structures(
                source,
                [
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                ["Struct"],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let forward_attributed_chain_structure_terminal_extraction =
            extract_builtin_source_reserve_declarations(
                &forward_attributed_chain_structure_terminal,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("forward terminal structure reserve should still extract");
        assert!(
            forward_attributed_chain_structure_terminal_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 requires the structure definition to precede the terminal mode definition"
        );

        let contextual_attributed_chain_structure_dependency =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    contextual_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let contextual_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("contextual dependency structure-RHS chain reserve should still extract");
        assert!(
            contextual_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds contextual dependency mode definitions"
        );

        let contextual_attributed_chain_structure_root = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                contextual_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let contextual_attributed_chain_structure_root_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_attributed_chain_structure_root,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("contextual root structure-RHS chain reserve should still extract");
        assert!(
            contextual_attributed_chain_structure_root_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds contextual attributed root mode definitions"
        );

        let parameterized_attributed_chain_structure_dependency =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    parameterized_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let parameterized_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &parameterized_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("parameterized dependency structure-RHS chain reserve should still extract");
        assert!(
            parameterized_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds parameterized dependency mode definitions"
        );

        let recovered_attributed_chain_structure_dependency =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    recovered_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let recovered_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &recovered_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("recovered dependency structure-RHS chain reserve should still extract");
        assert!(
            recovered_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds recovered dependency mode definitions"
        );

        let argument_attributed_chain_structure_terminal = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbolWithArgs("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let argument_attributed_chain_structure_terminal_extraction =
            extract_builtin_source_reserve_declarations(
                &argument_attributed_chain_structure_terminal,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("argument-bearing terminal structure reserve should still extract");
        assert!(
            argument_attributed_chain_structure_terminal_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds argument-bearing terminal structure RHSs"
        );

        let argument_attributed_chain_structure_dependency = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbolWithArgs("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let argument_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &argument_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("argument-bearing dependency reserve should still extract");
        assert!(
            !argument_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_structure_a),
            "Task65 withholds argument-bearing dependency RHSs"
        );

        let reserve_attribute_args_attributed_chain_structure =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbolWithAttributeArgs("A"),
                )],
            );
        assert!(
            extract_builtin_source_reserve_declarations(
                &reserve_attribute_args_attributed_chain_structure,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .is_err(),
            "Task65 withholds reserve-head attribute arguments"
        );

        let imported_attributed_chain_structure_root_symbols =
            imported_structure_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "A",
            );
        let imported_attributed_chain_structure_root_module =
            imported_attributed_chain_structure_root_symbols
                .module_id()
                .clone();
        let imported_attributed_chain_structure_root = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &imported_attributed_chain_structure_root,
                imported_attributed_chain_structure_root_module,
                &imported_attributed_chain_structure_root_symbols,
            )
            .is_err(),
            "Task65 withholds imported attributed roots"
        );

        let imported_attributed_chain_structure_dependency_symbols =
            imported_structure_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "B",
            );
        let imported_attributed_chain_structure_dependency_module =
            imported_attributed_chain_structure_dependency_symbols
                .module_id()
                .clone();
        let imported_attributed_chain_structure_dependency = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let imported_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &imported_attributed_chain_structure_dependency,
                imported_attributed_chain_structure_dependency_module,
                &imported_attributed_chain_structure_dependency_symbols,
            )
            .expect("imported dependency structure-RHS chain reserve should still extract");
        assert!(
            imported_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds imported dependency modes"
        );

        let imported_attributed_chain_structure_terminal_symbols =
            imported_structure_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "Struct",
            );
        let imported_attributed_chain_structure_terminal_module =
            imported_attributed_chain_structure_terminal_symbols
                .module_id()
                .clone();
        let imported_attributed_chain_structure_terminal = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let imported_attributed_chain_structure_terminal_extraction =
            extract_builtin_source_reserve_declarations(
                &imported_attributed_chain_structure_terminal,
                imported_attributed_chain_structure_terminal_module,
                &imported_attributed_chain_structure_terminal_symbols,
            )
            .expect("imported terminal structure chain reserve should still extract");
        assert!(
            imported_attributed_chain_structure_terminal_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds imported terminal structures"
        );

        let ambiguous_attributed_chain_structure_root_symbols =
            ambiguous_attributed_structure_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "A",
            );
        let ambiguous_attributed_chain_structure_root_module =
            ambiguous_attributed_chain_structure_root_symbols
                .module_id()
                .clone();
        let ambiguous_attributed_chain_structure_root = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &ambiguous_attributed_chain_structure_root,
                ambiguous_attributed_chain_structure_root_module,
                &ambiguous_attributed_chain_structure_root_symbols,
            )
            .is_err(),
            "Task65 withholds ambiguous attributed roots"
        );

        let ambiguous_attributed_chain_structure_dependency_symbols =
            ambiguous_attributed_structure_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "B",
            );
        let ambiguous_attributed_chain_structure_dependency_module =
            ambiguous_attributed_chain_structure_dependency_symbols
                .module_id()
                .clone();
        let ambiguous_attributed_chain_structure_dependency =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let ambiguous_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &ambiguous_attributed_chain_structure_dependency,
                ambiguous_attributed_chain_structure_dependency_module,
                &ambiguous_attributed_chain_structure_dependency_symbols,
            )
            .expect("ambiguous dependency structure-RHS chain reserve should still extract");
        assert!(
            ambiguous_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds ambiguous dependency modes"
        );

        let ambiguous_attributed_chain_structure_terminal_symbols =
            ambiguous_attributed_structure_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "Struct",
            );
        let ambiguous_attributed_chain_structure_terminal_module =
            ambiguous_attributed_chain_structure_terminal_symbols
                .module_id()
                .clone();
        let ambiguous_attributed_chain_structure_terminal = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let ambiguous_attributed_chain_structure_terminal_extraction =
            extract_builtin_source_reserve_declarations(
                &ambiguous_attributed_chain_structure_terminal,
                ambiguous_attributed_chain_structure_terminal_module,
                &ambiguous_attributed_chain_structure_terminal_symbols,
            )
            .expect("ambiguous terminal structure chain reserve should still extract");
        assert!(
            ambiguous_attributed_chain_structure_terminal_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds ambiguous terminal structures"
        );

        let attributed_chain_bare_rhs_symbols = source_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
        );
        let attributed_chain_bare_rhs_module =
            attributed_chain_bare_rhs_symbols.module_id().clone();
        let attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_bare_rhs_extraction = extract_builtin_source_reserve_declarations(
            &attributed_chain_bare_rhs,
            attributed_chain_bare_rhs_module.clone(),
            &attributed_chain_bare_rhs_symbols,
        )
        .expect("attributed mode chain ending in bare builtin RHS should extract");
        let attributed_chain_bare_b = resolve_visible_type_head(
            &attributed_chain_bare_rhs_symbols,
            &attributed_chain_bare_rhs_module,
            "B",
        )
        .unwrap();
        let attributed_chain_bare_a = resolve_visible_type_head(
            &attributed_chain_bare_rhs_symbols,
            &attributed_chain_bare_rhs_module,
            "A",
        )
        .unwrap();
        assert_eq!(
            attributed_chain_bare_rhs_extraction.bindings()[0]
                .type_attributes
                .len(),
            1
        );
        assert_eq!(
            attributed_chain_bare_rhs_extraction.mode_expansions().len(),
            2
        );
        assert!(matches!(
            attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_bare_b)
                .map(|expansion| &expansion.radix.head),
            Some(TypeHeadInput::BuiltinSet)
        ));
        assert_eq!(
            attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_bare_a)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(attributed_chain_bare_b.clone()))
        );

        let attributed_chain_object_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("object")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_object_rhs_extraction = extract_builtin_source_reserve_declarations(
            &attributed_chain_object_rhs,
            attributed_chain_bare_rhs_module.clone(),
            &attributed_chain_bare_rhs_symbols,
        )
        .expect("attributed mode chain ending in object RHS should extract");
        assert!(matches!(
            attributed_chain_object_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_bare_b)
                .map(|expansion| &expansion.radix.head),
            Some(TypeHeadInput::BuiltinObject)
        ));
        assert!(
            attributed_chain_object_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_bare_a)
        );

        let cached_attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let cached_attributed_chain_bare_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &cached_attributed_chain_bare_rhs,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("cached attributed bare-RHS chain reserve should still extract");
        assert!(
            cached_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_bare_b)
        );
        assert!(
            cached_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_bare_a),
            "cached bare-builtin terminal payload may feed the attributed one-edge chain"
        );

        let deeper_attributed_chain_bare_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("empty", SymbolKind::Attribute),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
            ],
        );
        let deeper_attributed_chain_bare_module =
            deeper_attributed_chain_bare_symbols.module_id().clone();
        let deeper_attributed_chain_bare = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("C")),
            ],
        );
        let deeper_attributed_chain_bare_extraction = extract_builtin_source_reserve_declarations(
            &deeper_attributed_chain_bare,
            deeper_attributed_chain_bare_module.clone(),
            &deeper_attributed_chain_bare_symbols,
        )
        .expect("deeper attributed bare-RHS chain reserve should still extract");
        let deeper_attributed_chain_c = resolve_visible_type_head(
            &deeper_attributed_chain_bare_symbols,
            &deeper_attributed_chain_bare_module,
            "C",
        )
        .unwrap();
        assert!(
            !deeper_attributed_chain_bare_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_chain_c),
            "attributed local-mode chains remain capped at one dependency edge"
        );

        let mixed_attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let mixed_attributed_chain_bare_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &mixed_attributed_chain_bare_rhs,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("mixed attributed bare-RHS chain reserve should still extract");
        assert!(
            mixed_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "mixed bare/attributed uses of the attributed chain root withhold expansion"
        );

        let attributed_dependency_for_attributed_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::AttributedQualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let attributed_dependency_for_attributed_chain_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_dependency_for_attributed_chain,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("attributed dependency chain reserve should still extract");
        assert!(
            !attributed_dependency_for_attributed_chain_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_bare_a),
            "an attributed dependency mode stays outside the attributed-root chain slice"
        );

        let attributed_chain_with_mode_args = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbolWithArgs("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_with_mode_args_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_chain_with_mode_args,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("argument-bearing attributed bare-RHS chain reserve should still extract");
        assert!(
            attributed_chain_with_mode_args_extraction
                .mode_expansions()
                .is_empty(),
            "argument-bearing mode dependencies remain outside Task64"
        );

        let contextual_attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                contextual_mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let contextual_attributed_chain_bare_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_attributed_chain_bare_rhs,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("contextual attributed bare-RHS chain reserve should still extract");
        assert!(
            contextual_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "definition-local context keeps attributed bare-RHS chains withheld"
        );

        let parameterized_attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                parameterized_mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let parameterized_attributed_chain_bare_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &parameterized_attributed_chain_bare_rhs,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("parameterized attributed bare-RHS chain reserve should still extract");
        assert!(
            parameterized_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "parameterized definitions remain outside Task64"
        );

        let recovered_attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                recovered_mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let recovered_attributed_chain_bare_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &recovered_attributed_chain_bare_rhs,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("recovered attributed bare-RHS chain reserve should still extract");
        assert!(
            recovered_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "recovered definitions remain outside Task64"
        );

        let contextual_attributed_chain_head = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                contextual_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let contextual_attributed_chain_head_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_attributed_chain_head,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("contextual attributed chain head reserve should still extract");
        assert!(
            contextual_attributed_chain_head_extraction
                .mode_expansions()
                .is_empty(),
            "definition-local context on the attributed root keeps Task64 withheld"
        );

        let parameterized_attributed_chain_head = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                parameterized_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let parameterized_attributed_chain_head_extraction =
            extract_builtin_source_reserve_declarations(
                &parameterized_attributed_chain_head,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("parameterized attributed chain head reserve should still extract");
        assert!(
            parameterized_attributed_chain_head_extraction
                .mode_expansions()
                .is_empty(),
            "parameterized attributed root definitions remain outside Task64"
        );

        let recovered_attributed_chain_head = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                recovered_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let recovered_attributed_chain_head_extraction =
            extract_builtin_source_reserve_declarations(
                &recovered_attributed_chain_head,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("recovered attributed chain head reserve should still extract");
        assert!(
            recovered_attributed_chain_head_extraction
                .mode_expansions()
                .is_empty(),
            "recovered attributed root definitions remain outside Task64"
        );

        let duplicate_attributed_chain_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::Builtin("object")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let duplicate_attributed_chain_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_attributed_chain_dependency,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("duplicate dependency definition reserve should still extract");
        assert!(
            duplicate_attributed_chain_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task64 requires a unique dependency mode definition"
        );

        let duplicate_attributed_chain_head = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let duplicate_attributed_chain_head_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_attributed_chain_head,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("duplicate attributed root definition reserve should still extract");
        assert!(
            duplicate_attributed_chain_head_extraction
                .mode_expansions()
                .is_empty(),
            "Task64 requires a unique attributed root mode definition"
        );

        let forward_attributed_chain_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("B", ReserveTypeShape::Builtin("set")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let forward_attributed_chain_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &forward_attributed_chain_dependency,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("forward dependency attributed chain reserve should still extract");
        assert!(
            forward_attributed_chain_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task64 requires the dependency definition to precede the attributed root definition"
        );

        let reserve_attribute_args_attributed_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbolWithAttributeArgs("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &reserve_attribute_args_attributed_chain,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .is_err(),
            "reserve-head attribute arguments remain outside Task64"
        );

        let imported_root_symbols = imported_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            "A",
        );
        let imported_root_module = imported_root_symbols.module_id().clone();
        let imported_root_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &imported_root_chain,
                imported_root_module,
                &imported_root_symbols,
            )
            .is_err(),
            "imported attributed roots remain outside Task64"
        );

        let imported_dependency_symbols = imported_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            "B",
        );
        let imported_dependency_module = imported_dependency_symbols.module_id().clone();
        let imported_dependency_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let imported_dependency_extraction = extract_builtin_source_reserve_declarations(
            &imported_dependency_chain,
            imported_dependency_module,
            &imported_dependency_symbols,
        )
        .expect("imported dependency attributed chain reserve should still extract");
        assert!(
            imported_dependency_extraction.mode_expansions().is_empty(),
            "imported dependencies remain outside Task64"
        );

        let ambiguous_root_symbols = ambiguous_attributed_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            "A",
        );
        let ambiguous_root_module = ambiguous_root_symbols.module_id().clone();
        let ambiguous_root_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &ambiguous_root_chain,
                ambiguous_root_module,
                &ambiguous_root_symbols,
            )
            .is_err(),
            "ambiguous attributed roots remain outside Task64"
        );

        let ambiguous_dependency_symbols = ambiguous_attributed_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            "B",
        );
        let ambiguous_dependency_module = ambiguous_dependency_symbols.module_id().clone();
        let ambiguous_dependency_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let ambiguous_dependency_extraction = extract_builtin_source_reserve_declarations(
            &ambiguous_dependency_chain,
            ambiguous_dependency_module,
            &ambiguous_dependency_symbols,
        )
        .expect("ambiguous dependency attributed chain reserve should still extract");
        assert!(
            ambiguous_dependency_extraction.mode_expansions().is_empty(),
            "ambiguous dependencies remain outside Task64"
        );

        let chain_attributed_rhs_symbols = source_mode_chain_symbol_env(ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("bridge"),
        ));
        let chain_attributed_rhs_module = chain_attributed_rhs_symbols.module_id().clone();
        let chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let chain_attributed_rhs_extraction = extract_builtin_source_reserve_declarations(
            &chain_attributed_rhs,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("mode chain ending in attributed RHS reserve should still extract");
        let chain_attributed_b = resolve_visible_type_head(
            &chain_attributed_rhs_symbols,
            &chain_attributed_rhs_module,
            "B",
        )
        .unwrap();
        let chain_attributed_a = resolve_visible_type_head(
            &chain_attributed_rhs_symbols,
            &chain_attributed_rhs_module,
            "A",
        )
        .unwrap();
        assert_eq!(chain_attributed_rhs_extraction.mode_expansions().len(), 2);
        let chain_attributed_b_expansion = chain_attributed_rhs_extraction
            .mode_expansions()
            .get(&chain_attributed_b)
            .expect("terminal attributed RHS expansion should be present");
        assert!(matches!(
            chain_attributed_b_expansion.radix.head,
            TypeHeadInput::BuiltinSet
        ));
        assert_eq!(chain_attributed_b_expansion.attributes.len(), 1);
        assert_eq!(
            chain_attributed_b_expansion.attributes[0].polarity,
            AttributePolarity::Negative
        );
        assert_eq!(
            chain_attributed_rhs_extraction
                .mode_expansions()
                .get(&chain_attributed_a)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(chain_attributed_b.clone()))
        );

        let chain_attributed_object_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedObject),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let chain_attributed_object_rhs_extraction = extract_builtin_source_reserve_declarations(
            &chain_attributed_object_rhs,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("mode chain ending in attributed object RHS reserve should still extract");
        let chain_attributed_object_b_expansion = chain_attributed_object_rhs_extraction
            .mode_expansions()
            .get(&chain_attributed_b)
            .expect("terminal attributed object RHS expansion should be present");
        assert!(matches!(
            chain_attributed_object_b_expansion.radix.head,
            TypeHeadInput::BuiltinObject
        ));
        assert_eq!(chain_attributed_object_b_expansion.attributes.len(), 1);
        assert!(
            chain_attributed_object_rhs_extraction
                .mode_expansions()
                .contains_key(&chain_attributed_a)
        );

        let cached_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("A")),
            ],
        );
        let cached_attributed_rhs_chain_extraction = extract_builtin_source_reserve_declarations(
            &cached_attributed_rhs_chain,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("cached attributed RHS chain reserve should still extract");
        assert_eq!(
            cached_attributed_rhs_chain_extraction
                .mode_expansions()
                .len(),
            2
        );
        assert!(
            cached_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&chain_attributed_b)
        );
        assert!(
            cached_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&chain_attributed_a)
        );

        let deeper_attributed_rhs_chain_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("empty", SymbolKind::Attribute),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
            ],
        );
        let deeper_attributed_rhs_chain_module =
            deeper_attributed_rhs_chain_symbols.module_id().clone();
        let deeper_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("C")),
            ],
        );
        let deeper_attributed_rhs_chain_extraction = extract_builtin_source_reserve_declarations(
            &deeper_attributed_rhs_chain,
            deeper_attributed_rhs_chain_module.clone(),
            &deeper_attributed_rhs_chain_symbols,
        )
        .expect("deeper attributed RHS chain reserve should still extract");
        let deeper_attributed_b = resolve_visible_type_head(
            &deeper_attributed_rhs_chain_symbols,
            &deeper_attributed_rhs_chain_module,
            "B",
        )
        .unwrap();
        let deeper_attributed_a = resolve_visible_type_head(
            &deeper_attributed_rhs_chain_symbols,
            &deeper_attributed_rhs_chain_module,
            "A",
        )
        .unwrap();
        let deeper_attributed_c = resolve_visible_type_head(
            &deeper_attributed_rhs_chain_symbols,
            &deeper_attributed_rhs_chain_module,
            "C",
        )
        .unwrap();
        assert!(
            deeper_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_b)
        );
        assert!(
            deeper_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_a)
        );
        assert!(
            !deeper_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_c),
            "deeper attributed RHS chains remain outside the one-edge bridge"
        );

        let attributed_rhs_with_attribute_args = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSetWithAttributeArgs),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let attributed_rhs_with_attribute_args_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_rhs_with_attribute_args,
                chain_attributed_rhs_module.clone(),
                &chain_attributed_rhs_symbols,
            )
            .expect("attribute-argument attributed RHS chain reserve should still extract");
        assert!(
            attributed_rhs_with_attribute_args_extraction
                .mode_expansions()
                .is_empty(),
            "attribute arguments on the terminal attributed RHS remain outside Task63"
        );

        let attributed_rhs_with_mode_args = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbolWithArgs("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let attributed_rhs_with_mode_args_extraction = extract_builtin_source_reserve_declarations(
            &attributed_rhs_with_mode_args,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("argument-bearing attributed RHS chain reserve should still extract");
        assert!(
            attributed_rhs_with_mode_args_extraction
                .mode_expansions()
                .is_empty(),
            "argument-bearing mode dependencies remain outside Task63"
        );

        let contextual_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                contextual_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let contextual_attributed_rhs_extraction = extract_builtin_source_reserve_declarations(
            &contextual_attributed_rhs_chain,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("contextual attributed RHS chain reserve should still extract");
        assert!(
            contextual_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "definition-local context keeps attributed-RHS chain expansions withheld"
        );

        let parameterized_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                parameterized_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let parameterized_attributed_rhs_extraction = extract_builtin_source_reserve_declarations(
            &parameterized_attributed_rhs_chain,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("parameterized attributed RHS chain reserve should still extract");
        assert!(
            parameterized_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "parameterized definitions remain outside Task63"
        );

        let recovered_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                recovered_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let recovered_attributed_rhs_extraction = extract_builtin_source_reserve_declarations(
            &recovered_attributed_rhs_chain,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("recovered attributed RHS chain reserve should still extract");
        assert!(
            recovered_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "recovered definitions remain outside Task63"
        );

        let imported_attribute_chain_symbols = imported_attribute_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
        );
        let imported_attribute_chain_module = imported_attribute_chain_symbols.module_id().clone();
        let imported_attribute_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let imported_attribute_chain_extraction = extract_builtin_source_reserve_declarations(
            &imported_attribute_chain,
            imported_attribute_chain_module,
            &imported_attribute_chain_symbols,
        )
        .expect("imported-attribute attributed RHS chain reserve should still extract");
        assert!(
            imported_attribute_chain_extraction
                .mode_expansions()
                .is_empty(),
            "imported terminal RHS attributes remain outside Task63"
        );

        let ambiguous_attribute_chain_symbols = ambiguous_attribute_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
        );
        let ambiguous_attribute_chain_module =
            ambiguous_attribute_chain_symbols.module_id().clone();
        let ambiguous_attribute_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let ambiguous_attribute_chain_extraction = extract_builtin_source_reserve_declarations(
            &ambiguous_attribute_chain,
            ambiguous_attribute_chain_module,
            &ambiguous_attribute_chain_symbols,
        )
        .expect("ambiguous-attribute attributed RHS chain reserve should still extract");
        assert!(
            ambiguous_attribute_chain_extraction
                .mode_expansions()
                .is_empty(),
            "ambiguous terminal RHS attributes remain outside Task63"
        );

        let forward_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let forward_structure_rhs_symbols = source_local_symbols_env(
            forward_structure_rhs_module.clone(),
            &[
                ("Struct", SymbolKind::Structure),
                ("Mode", SymbolKind::Mode),
            ],
        );
        let forward_structure_rhs = mode_chain_reserve_ast_with_trailing_structures(
            source,
            [mode_definition(
                "Mode",
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
            ["Struct"],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("Mode"),
            )],
        );
        let forward_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &forward_structure_rhs,
            forward_structure_rhs_module,
            &forward_structure_rhs_symbols,
        )
        .expect("forward structure RHS reserve should still extract");
        assert!(
            forward_structure_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "structure RHS expansion payloads require the structure definition to precede the mode definition"
        );

        let forward_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
                mode_definition("A", ReserveTypeShape::Builtin("set")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("B"),
            )],
        );
        let forward_extraction =
            extract_builtin_source_reserve_declarations(&forward_dependency, module, &symbols)
                .expect("forward dependency reserve should still extract");
        assert!(
            forward_extraction.mode_expansions().is_empty(),
            "mode expansion chains require each dependency definition to precede its use"
        );

        let three_edge_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
                ("D", SymbolKind::Mode),
            ],
        );
        let three_edge_module = three_edge_symbols.module_id().clone();
        let three_edge_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("D", ReserveTypeShape::QualifiedSymbol("C")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("D"),
            )],
        );
        let three_edge_extraction = extract_builtin_source_reserve_declarations(
            &three_edge_chain,
            three_edge_module.clone(),
            &three_edge_symbols,
        )
        .expect("three-edge chain reserve should extract");
        let three_edge_a =
            resolve_visible_type_head(&three_edge_symbols, &three_edge_module, "A").unwrap();
        let three_edge_b =
            resolve_visible_type_head(&three_edge_symbols, &three_edge_module, "B").unwrap();
        let three_edge_c =
            resolve_visible_type_head(&three_edge_symbols, &three_edge_module, "C").unwrap();
        let three_edge_d =
            resolve_visible_type_head(&three_edge_symbols, &three_edge_module, "D").unwrap();
        assert_eq!(three_edge_extraction.mode_expansions().len(), 4);
        assert!(
            three_edge_extraction
                .mode_expansions()
                .contains_key(&three_edge_a)
        );
        assert!(
            three_edge_extraction
                .mode_expansions()
                .contains_key(&three_edge_b)
        );
        assert!(
            three_edge_extraction
                .mode_expansions()
                .contains_key(&three_edge_c)
        );
        assert!(
            three_edge_extraction
                .mode_expansions()
                .contains_key(&three_edge_d),
            "task 73 admits three local-mode dependency edges beyond the reserve head"
        );

        let four_edge_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
                ("D", SymbolKind::Mode),
                ("E", SymbolKind::Mode),
            ],
        );
        let four_edge_module = four_edge_symbols.module_id().clone();
        let four_edge_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("D", ReserveTypeShape::QualifiedSymbol("C")),
                mode_definition("E", ReserveTypeShape::QualifiedSymbol("D")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("E"),
            )],
        );
        let four_edge_extraction = extract_builtin_source_reserve_declarations(
            &four_edge_chain,
            four_edge_module.clone(),
            &four_edge_symbols,
        )
        .expect("four-edge chain reserve should extract");
        let four_edge_a =
            resolve_visible_type_head(&four_edge_symbols, &four_edge_module, "A").unwrap();
        let four_edge_b =
            resolve_visible_type_head(&four_edge_symbols, &four_edge_module, "B").unwrap();
        let four_edge_c =
            resolve_visible_type_head(&four_edge_symbols, &four_edge_module, "C").unwrap();
        let four_edge_d =
            resolve_visible_type_head(&four_edge_symbols, &four_edge_module, "D").unwrap();
        let four_edge_e =
            resolve_visible_type_head(&four_edge_symbols, &four_edge_module, "E").unwrap();
        assert_eq!(four_edge_extraction.mode_expansions().len(), 5);
        for symbol in [
            &four_edge_a,
            &four_edge_b,
            &four_edge_c,
            &four_edge_d,
            &four_edge_e,
        ] {
            assert!(
                four_edge_extraction.mode_expansions().contains_key(symbol),
                "task 74 admits structurally valid bare builtin-terminal local-mode chains"
            );
        }

        let long_chain_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
                ("D", SymbolKind::Mode),
                ("E", SymbolKind::Mode),
                ("F", SymbolKind::Mode),
                ("G", SymbolKind::Mode),
            ],
        );
        let long_chain_module = long_chain_symbols.module_id().clone();
        let long_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("D", ReserveTypeShape::QualifiedSymbol("C")),
                mode_definition("E", ReserveTypeShape::QualifiedSymbol("D")),
                mode_definition("F", ReserveTypeShape::QualifiedSymbol("E")),
                mode_definition("G", ReserveTypeShape::QualifiedSymbol("F")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("G"),
            )],
        );
        let long_chain_extraction = extract_builtin_source_reserve_declarations(
            &long_chain,
            long_chain_module.clone(),
            &long_chain_symbols,
        )
        .expect("long chain reserve should extract");
        let long_chain_g =
            resolve_visible_type_head(&long_chain_symbols, &long_chain_module, "G").unwrap();
        assert_eq!(long_chain_extraction.mode_expansions().len(), 7);
        assert!(
            long_chain_extraction
                .mode_expansions()
                .contains_key(&long_chain_g),
            "task 74 is structural and not capped at four local-mode dependency edges"
        );

        let cached_deeper_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let cached_deeper_symbols = source_local_symbols_env(
            cached_deeper_module.clone(),
            &[
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
                ("D", SymbolKind::Mode),
                ("E", SymbolKind::Mode),
            ],
        );
        let cached_deeper_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("D", ReserveTypeShape::QualifiedSymbol("C")),
                mode_definition("E", ReserveTypeShape::QualifiedSymbol("D")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("D")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("E")),
            ],
        );
        let cached_deeper_extraction = extract_builtin_source_reserve_declarations(
            &cached_deeper_chain,
            cached_deeper_module.clone(),
            &cached_deeper_symbols,
        )
        .expect("cached deeper chain reserve should still extract");
        let cached_a =
            resolve_visible_type_head(&cached_deeper_symbols, &cached_deeper_module, "A").unwrap();
        let cached_b =
            resolve_visible_type_head(&cached_deeper_symbols, &cached_deeper_module, "B").unwrap();
        let cached_c =
            resolve_visible_type_head(&cached_deeper_symbols, &cached_deeper_module, "C").unwrap();
        let cached_d =
            resolve_visible_type_head(&cached_deeper_symbols, &cached_deeper_module, "D").unwrap();
        let cached_e =
            resolve_visible_type_head(&cached_deeper_symbols, &cached_deeper_module, "E").unwrap();
        assert!(
            cached_deeper_extraction
                .mode_expansions()
                .contains_key(&cached_a)
        );
        assert!(
            cached_deeper_extraction
                .mode_expansions()
                .contains_key(&cached_b)
        );
        assert!(
            cached_deeper_extraction
                .mode_expansions()
                .contains_key(&cached_c)
        );
        assert!(
            cached_deeper_extraction
                .mode_expansions()
                .contains_key(&cached_d),
            "cached three-edge expansion payloads remain available for the supported reserve"
        );
        assert!(
            cached_deeper_extraction
                .mode_expansions()
                .contains_key(&cached_e),
            "cached three-edge expansion payloads may feed a structurally valid four-edge chain"
        );

        let cached_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let cached_structure_rhs_symbols = source_local_symbols_env(
            cached_structure_rhs_module.clone(),
            &[
                ("Struct", SymbolKind::Structure),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
            ],
        );
        let cached_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("A")),
            ],
        );
        let cached_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &cached_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("cached structure RHS chain reserve should still extract");
        let cached_structure_b = resolve_visible_type_head(
            &cached_structure_rhs_symbols,
            &cached_structure_rhs_module,
            "B",
        )
        .unwrap();
        let cached_structure_a = resolve_visible_type_head(
            &cached_structure_rhs_symbols,
            &cached_structure_rhs_module,
            "A",
        )
        .unwrap();
        assert!(
            cached_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_structure_b)
        );
        assert!(
            cached_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_structure_a),
            "cached direct structure-RHS expansion should feed the one-edge structure-RHS chain"
        );

        let cached_forward_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("A")),
            ],
        );
        let cached_forward_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &cached_forward_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("cached forward structure-RHS chain reserve should still extract");
        assert!(
            cached_forward_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_structure_b)
        );
        assert!(
            !cached_forward_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_structure_a),
            "cached direct structure-RHS payloads must still prove the dependency definition precedes the dependent mode"
        );

        let cached_deeper_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let cached_deeper_structure_rhs_symbols = source_local_symbols_env(
            cached_deeper_structure_rhs_module.clone(),
            &[
                ("Struct", SymbolKind::Structure),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
            ],
        );
        let cached_deeper_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["w"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("C")),
            ],
        );
        let cached_deeper_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &cached_deeper_structure_rhs_chain,
            cached_deeper_structure_rhs_module.clone(),
            &cached_deeper_structure_rhs_symbols,
        )
        .expect("cached deeper structure-RHS chain reserve should still extract");
        let cached_deeper_structure_b = resolve_visible_type_head(
            &cached_deeper_structure_rhs_symbols,
            &cached_deeper_structure_rhs_module,
            "B",
        )
        .unwrap();
        let cached_deeper_structure_a = resolve_visible_type_head(
            &cached_deeper_structure_rhs_symbols,
            &cached_deeper_structure_rhs_module,
            "A",
        )
        .unwrap();
        let cached_deeper_structure_c = resolve_visible_type_head(
            &cached_deeper_structure_rhs_symbols,
            &cached_deeper_structure_rhs_module,
            "C",
        )
        .unwrap();
        assert!(
            cached_deeper_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_deeper_structure_b)
        );
        assert!(
            cached_deeper_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_deeper_structure_a)
        );
        assert!(
            !cached_deeper_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_deeper_structure_c),
            "cached one-edge structure-RHS payloads must not let a deeper chain bypass the cap"
        );

        let duplicate_mode_definition = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let duplicate_mode_extraction = extract_builtin_source_reserve_declarations(
            &duplicate_mode_definition,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("duplicate mode definition reserve should still extract");
        assert!(
            duplicate_mode_extraction.mode_expansions().is_empty(),
            "structure-RHS chains require a unique preceding mode definition for every mode head"
        );

        let duplicate_structure_definition = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct", "Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let duplicate_structure_extraction = extract_builtin_source_reserve_declarations(
            &duplicate_structure_definition,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("duplicate structure definition reserve should still extract");
        assert!(
            duplicate_structure_extraction.mode_expansions().is_empty(),
            "structure-RHS chains require a unique preceding structure definition for the terminal structure head"
        );

        let contextual_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                contextual_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let contextual_structure_extraction = extract_builtin_source_reserve_declarations(
            &contextual_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("contextual structure-RHS chain reserve should still extract");
        assert!(
            contextual_structure_extraction.mode_expansions().is_empty(),
            "definition-local context keeps structure-RHS chain expansions withheld"
        );

        let parameterized_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                parameterized_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let parameterized_structure_extraction = extract_builtin_source_reserve_declarations(
            &parameterized_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("parameterized structure-RHS chain reserve should still extract");
        assert!(
            parameterized_structure_extraction
                .mode_expansions()
                .is_empty(),
            "parameterized mode definitions remain outside the source-derived structure-RHS chain slice"
        );

        let recovered_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                recovered_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let recovered_structure_extraction = extract_builtin_source_reserve_declarations(
            &recovered_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("recovered structure-RHS chain reserve should still extract");
        assert!(
            recovered_structure_extraction.mode_expansions().is_empty(),
            "recovered mode definitions remain outside the source-derived structure-RHS chain slice"
        );

        let argument_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbolWithArgs("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let argument_structure_extraction = extract_builtin_source_reserve_declarations(
            &argument_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("argument-bearing structure-RHS chain reserve should still extract");
        assert!(
            argument_structure_extraction.mode_expansions().is_empty(),
            "argument-bearing RHS symbols remain outside the source-derived structure-RHS chain slice"
        );

        let cyclic_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[("A", SymbolKind::Mode)],
        );
        let cyclic_module = cyclic_symbols.module_id().clone();
        let cyclic_chain = mode_chain_reserve_ast(
            source,
            [mode_definition("A", ReserveTypeShape::QualifiedSymbol("A"))],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let cyclic_extraction = extract_builtin_source_reserve_declarations(
            &cyclic_chain,
            cyclic_module,
            &cyclic_symbols,
        )
        .expect("cyclic chain reserve should still extract");
        assert!(
            cyclic_extraction.mode_expansions().is_empty(),
            "cyclic local-mode RHS dependencies must not produce partial chain expansions"
        );

        let ambiguous_symbols = ambiguous_mode_chain_symbol_env(ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("bridge"),
        ));
        let ambiguous_module = ambiguous_symbols.module_id().clone();
        let ambiguous_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("B"),
            )],
        );
        let ambiguous_extraction = extract_builtin_source_reserve_declarations(
            &ambiguous_dependency,
            ambiguous_module,
            &ambiguous_symbols,
        )
        .expect("ambiguous dependency reserve should still extract");
        assert!(
            ambiguous_extraction.mode_expansions().is_empty(),
            "ambiguous local-mode RHS dependencies must not produce partial chain expansions"
        );
    }

    #[test]
    fn source_reserve_bridge_assembles_declaration_checked_resolved_typed_ast_handoff() {
        let source_id = source_id(94);
        let ast = reserve_ast(
            source_id,
            vec![
                reserve_item(vec!["x", "y"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["z"], ReserveTypeShape::Builtin("object")),
            ],
        );
        let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let source_reserve =
            extract_builtin_source_reserve_declarations(&ast, module.clone(), &symbols)
                .expect("builtin reserve declarations should extract");

        let handoff = assemble_source_checker_handoff(&symbols, &source_reserve)
            .expect("source-derived checker handoff should reach ResolvedTypedAst");
        let resolved = &handoff.resolved;

        assert_eq!(handoff.binding_env.bindings().len(), 3);
        let module_context = handoff
            .binding_env
            .contexts()
            .get(source_reserve.module_context())
            .expect("module binding context should exist");
        assert_eq!(
            module_context.bindings,
            vec![BindingId::new(0), BindingId::new(1), BindingId::new(2)]
        );
        assert_eq!(module_context.visible_bindings, module_context.bindings);
        assert_eq!(handoff.declarations.declarations().len(), 3);
        assert_eq!(handoff.declarations.contexts().len(), 1);
        assert!(handoff.declarations.diagnostics().is_empty());
        assert_eq!(handoff.typed_ast.contexts().len(), 1);
        assert_eq!(resolved.nodes().len(), 7);
        assert_eq!(resolved.expr_metadata().len(), 3);
        assert!(resolved.diagnostics().is_empty());
        let summary = ResolvedTypedAstSummary::from_ast(resolved);
        assert_eq!(summary.source_id(), source_id);
        assert_eq!(summary.module_id(), resolved.module_id());
        assert!(
            summary.checker_sites().is_empty(),
            "successful reserve-only source payload should be summary-readable without checker recovery sites"
        );
        assert_ne!(source_reserve.type_node(0), source_reserve.type_node(1));
        assert_eq!(
            source_reserve.bindings()[0].type_range,
            source_reserve.bindings()[1].type_range
        );
        for index in 0..source_reserve.bindings().len() {
            let type_node = resolved
                .nodes()
                .node(ResolvedTypedNodeId::new(
                    source_reserve.type_node(index).index(),
                ))
                .expect("resolved type node should be present");
            match &type_node.kind {
                ResolvedTypedNodeKind::SourcePreserved { role } => {
                    assert_eq!(role.as_str(), "source.reserve.type_expression");
                }
                other => panic!("unexpected resolved type node kind: {other:?}"),
            }
            assert!(type_node.final_type.is_some());
            let declaration_node = resolved
                .nodes()
                .node(ResolvedTypedNodeId::new(
                    source_reserve.declaration_node(index).index(),
                ))
                .expect("resolved declaration node should be present");
            match &declaration_node.kind {
                ResolvedTypedNodeKind::SourcePreserved { role } => {
                    assert_eq!(role.as_str(), "source.reserve.declaration");
                }
                other => panic!("unexpected resolved declaration node kind: {other:?}"),
            }
            assert_eq!(declaration_node.children.len(), 1);
            assert!(declaration_node.final_type.is_some());
            let expr = mizar_checker::resolved_typed_ast::ExprId::new(format!(
                "source.reserve.declaration.{index}"
            ));
            let metadata = resolved
                .expr_metadata()
                .get_by_expr(&expr)
                .expect("expression metadata should be present");
            assert!(metadata.final_type.is_some());
            assert_eq!(metadata.local_context, Some(LocalTypeContextId::new(0)));
        }
        assert!(resolved.debug_text().contains("source.reserve.declaration"));
        assert!(
            resolved
                .debug_text()
                .contains("source.reserve.type_expression")
        );
    }

    #[test]
    fn source_reserve_bridge_reports_gap_or_evidence_detail_for_unsupported_shapes() {
        let source_id = source_id(95);
        let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let symbols = source_symbol_env(module.clone());
        let non_builtin = reserve_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::NonBuiltin("T"))],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&non_builtin, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let equality_theorem =
            builtin_equality_theorem_ast(source_id, "TermFormulaPayloadBoundary", "1", "1");
        assert_eq!(
            source_type_elaboration_detail_keys(&equality_theorem, module.clone(), &symbols),
            vec![
                "type_elaboration.checker.checker.formula.term.partial".to_owned(),
                "type_elaboration.checker.checker.term.external.numeric_type_payload".to_owned(),
            ]
        );
        let inequality_theorem = builtin_binary_theorem_ast(
            source_id,
            "BuiltinInequalityPayloadBoundary",
            "1",
            "<>",
            "2",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&inequality_theorem, module.clone(), &symbols),
            vec![
                "type_elaboration.checker.checker.formula.term.partial".to_owned(),
                "type_elaboration.checker.checker.term.external.numeric_type_payload".to_owned(),
            ]
        );
        let other_label_theorem =
            builtin_equality_theorem_ast(source_id, "OtherPayloadBoundary", "1", "1");
        assert_eq!(
            source_type_elaboration_detail_keys(&other_label_theorem, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_literal_theorem =
            builtin_equality_theorem_ast(source_id, "TermFormulaPayloadBoundary", "1", "2");
        assert_eq!(
            source_type_elaboration_detail_keys(&other_literal_theorem, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_inequality_label_theorem =
            builtin_binary_theorem_ast(source_id, "OtherPayloadBoundary", "1", "<>", "2");
        assert_eq!(
            source_type_elaboration_detail_keys(
                &other_inequality_label_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_inequality_literal_theorem = builtin_binary_theorem_ast(
            source_id,
            "BuiltinInequalityPayloadBoundary",
            "1",
            "<>",
            "1",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &other_inequality_literal_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let other_operator_theorem = builtin_binary_theorem_ast(
            source_id,
            "BuiltinInequalityPayloadBoundary",
            "1",
            "=",
            "2",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&other_operator_theorem, module.clone(), &symbols),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let mixed_reserve_and_inequality_theorem = reserve_then_builtin_binary_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            "BuiltinInequalityPayloadBoundary",
            "1",
            "<>",
            "2",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &mixed_reserve_and_inequality_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let mixed_reserve_and_theorem = reserve_then_builtin_equality_theorem_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::Builtin("set"))],
            "TermFormulaPayloadBoundary",
            "1",
            "1",
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &mixed_reserve_and_theorem,
                module.clone(),
                &symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let mixed = reserve_ast(
            source_id,
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["y"], ReserveTypeShape::AttributedSet),
            ],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&mixed, module, &symbols),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let attributed = reserve_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&attributed, symbols.module_id().clone(), &symbols),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let imported_symbols = imported_attribute_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed,
                imported_symbols.module_id().clone(),
                &imported_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let imported_fixture_attribute_symbols =
            imported_fixture_attribute_symbol_env(symbols.module_id().clone());
        let imported_fixture_attribute = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedSetWithNamedAttribute("TypeCaseAttr"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_attribute,
                imported_fixture_attribute_symbols.module_id().clone(),
                &imported_fixture_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );
        let imported_fixture_attribute_object = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedObjectWithNamedAttribute("TypeCaseAttr"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_attribute_object,
                imported_fixture_attribute_symbols.module_id().clone(),
                &imported_fixture_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_attribute_local_structure_symbols =
            local_structure_and_imported_fixture_attribute_symbol_env(
                symbols.module_id().clone(),
                "Struct",
                "TypeCaseAttr",
            );
        let imported_fixture_attribute_local_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbolWithNamedAttribute(
                    "TypeCaseAttr",
                    "Struct",
                ),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_attribute_local_structure,
                imported_fixture_attribute_local_structure_symbols
                    .module_id()
                    .clone(),
                &imported_fixture_attribute_local_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let imported_fixture_empty_attribute_symbols = imported_parser_fixture_symbol_env(
            symbols.module_id().clone(),
            "empty",
            SymbolKind::Attribute,
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );
        let imported_fixture_positive_empty_attribute = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedSetWithNamedAttribute("empty"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_positive_empty_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_empty_object_attribute = reserve_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedObject)],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_empty_object_attribute,
                imported_fixture_empty_attribute_symbols.module_id().clone(),
                &imported_fixture_empty_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
        let imported_fixture_empty_local_structure_symbols =
            local_structure_and_imported_fixture_attribute_symbol_env(
                symbols.module_id().clone(),
                "Struct",
                "empty",
            );
        let imported_fixture_empty_local_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbol("Struct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_empty_local_structure,
                imported_fixture_empty_local_structure_symbols
                    .module_id()
                    .clone(),
                &imported_fixture_empty_local_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let shadowed_imported_attribute_symbols =
            local_and_imported_attribute_symbol_env(symbols.module_id().clone(), "TypeCaseAttr");
        let resolved_shadowed_attribute = resolve_visible_attribute(
            &shadowed_imported_attribute_symbols,
            shadowed_imported_attribute_symbols.module_id(),
            "TypeCaseAttr",
        )
        .expect("a local attribute should shadow an imported attribute of the same spelling");
        assert_eq!(
            resolved_shadowed_attribute.module(),
            shadowed_imported_attribute_symbols.module_id()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_attribute,
                shadowed_imported_attribute_symbols.module_id().clone(),
                &shadowed_imported_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let local_mode = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("Mode"),
            )],
        );
        let mode_symbols = source_mode_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_mode,
                mode_symbols.module_id().clone(),
                &mode_symbols
            ),
            vec![
                "type_elaboration.checker.checker.type.external.mode_expansion_payload".to_owned(),
                "type_elaboration.checker.checker.type.recovery".to_owned(),
            ]
        );

        let local_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
        );
        let structure_symbols = source_structure_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_structure,
                structure_symbols.module_id().clone(),
                &structure_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let imported_mode_symbols = imported_mode_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_mode,
                imported_mode_symbols.module_id().clone(),
                &imported_mode_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let shadowed_imported_mode_symbols =
            local_and_imported_mode_symbol_env(symbols.module_id().clone(), "TypeCaseMode");
        let shadowed_imported_mode = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("TypeCaseMode"),
            )],
        );
        let resolved_shadowed_head = resolve_visible_type_head(
            &shadowed_imported_mode_symbols,
            shadowed_imported_mode_symbols.module_id(),
            "TypeCaseMode",
        )
        .expect("a local mode should shadow an imported mode of the same spelling");
        assert_eq!(
            resolved_shadowed_head.module(),
            shadowed_imported_mode_symbols.module_id()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &shadowed_imported_mode,
                shadowed_imported_mode_symbols.module_id().clone(),
                &shadowed_imported_mode_symbols
            ),
            vec![
                "type_elaboration.checker.checker.type.external.mode_expansion_payload".to_owned(),
                "type_elaboration.checker.checker.type.recovery".to_owned(),
            ]
        );

        let imported_structure_symbols = imported_structure_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_structure,
                imported_structure_symbols.module_id().clone(),
                &imported_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let imported_fixture_structure_symbols =
            imported_fixture_structure_symbol_env(symbols.module_id().clone());
        let imported_fixture_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("R"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_structure,
                imported_fixture_structure_symbols.module_id().clone(),
                &imported_fixture_structure_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let shadowed_imported_structure_symbols =
            local_and_imported_structure_symbol_env(symbols.module_id().clone(), "R");
        let resolved_shadowed_structure_head = resolve_visible_type_head(
            &shadowed_imported_structure_symbols,
            shadowed_imported_structure_symbols.module_id(),
            "R",
        )
        .expect("a local structure should shadow an imported structure of the same spelling");
        assert_eq!(
            resolved_shadowed_structure_head.module(),
            shadowed_imported_structure_symbols.module_id()
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_structure,
                shadowed_imported_structure_symbols.module_id().clone(),
                &shadowed_imported_structure_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let imported_fixture_type_case_struct_symbols = imported_parser_fixture_symbol_env(
            symbols.module_id().clone(),
            "TypeCaseStruct",
            SymbolKind::Structure,
        );
        let imported_fixture_type_case_struct = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("TypeCaseStruct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &imported_fixture_type_case_struct,
                imported_fixture_type_case_struct_symbols
                    .module_id()
                    .clone(),
                &imported_fixture_type_case_struct_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        let ambiguous_mode_symbols = ambiguous_mode_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_mode,
                ambiguous_mode_symbols.module_id().clone(),
                &ambiguous_mode_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let ambiguous_structure_symbols =
            ambiguous_structure_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &local_structure,
                ambiguous_structure_symbols.module_id().clone(),
                &ambiguous_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let mode_with_args = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbolWithArgs("Mode"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &mode_with_args,
                mode_symbols.module_id().clone(),
                &mode_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let structure_with_args = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbolWithArgs("Struct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &structure_with_args,
                structure_symbols.module_id().clone(),
                &structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let mode_attribute_symbols = source_mode_attribute_symbol_env(symbols.module_id().clone());
        let attributed_mode = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_mode,
                mode_attribute_symbols.module_id().clone(),
                &mode_attribute_symbols
            ),
            vec![
                "type_elaboration.checker.checker.type.external.mode_expansion_payload".to_owned(),
                "type_elaboration.checker.checker.type.recovery".to_owned(),
            ]
        );

        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_mode,
                mode_symbols.module_id().clone(),
                &mode_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let imported_attribute_mode_symbols =
            imported_attribute_mode_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_mode,
                imported_attribute_mode_symbols.module_id().clone(),
                &imported_attribute_mode_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let structure_attribute_symbols =
            source_structure_attribute_symbol_env(symbols.module_id().clone());
        let attributed_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbol("Struct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_structure,
                structure_attribute_symbols.module_id().clone(),
                &structure_attribute_symbols
            ),
            vec!["type_elaboration.checker.checker.declaration.deferred.evidence_query".to_owned()]
        );

        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_structure,
                structure_symbols.module_id().clone(),
                &structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let imported_attribute_structure_symbols =
            imported_attribute_structure_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_structure,
                imported_attribute_structure_symbols.module_id().clone(),
                &imported_attribute_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let ambiguous_attribute_structure_symbols =
            ambiguous_attribute_structure_symbol_env(symbols.module_id().clone());
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_structure,
                ambiguous_attribute_structure_symbols.module_id().clone(),
                &ambiguous_attribute_structure_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let attributed_structure_with_attribute_args = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbolWithAttributeArgs("Struct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_structure_with_attribute_args,
                structure_attribute_symbols.module_id().clone(),
                &structure_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let attributed_mode_with_attribute_args = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbolWithAttributeArgs("Mode"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &attributed_mode_with_attribute_args,
                mode_attribute_symbols.module_id().clone(),
                &mode_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let qualified_attribute_mode = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedAttributeQualifiedSymbol("Mode"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &qualified_attribute_mode,
                mode_attribute_symbols.module_id().clone(),
                &mode_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );

        let qualified_attribute_structure = reserve_ast(
            source_id,
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedAttributeQualifiedSymbol("Struct"),
            )],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(
                &qualified_attribute_structure,
                structure_attribute_symbols.module_id().clone(),
                &structure_attribute_symbols
            ),
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
    }

    fn source_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_local_symbol_env(module, "empty", SymbolKind::Attribute)
    }

    fn source_mode_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_local_symbol_env(module, "Mode", SymbolKind::Mode)
    }

    fn source_mode_chain_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_local_symbols_env(
            module,
            &[
                ("empty", SymbolKind::Attribute),
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
            ],
        )
    }

    fn source_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_local_symbol_env(module, "Struct", SymbolKind::Structure)
    }

    fn source_mode_attribute_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_symbol_pair_env(module, "Mode", SymbolKind::Mode)
    }

    fn source_structure_attribute_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        source_symbol_pair_env(module, "Struct", SymbolKind::Structure)
    }

    fn imported_attribute_mode_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_attribute_symbol_head_env(module, "Mode", SymbolKind::Mode)
    }

    fn imported_attribute_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_attribute_symbol_head_env(module, "Struct", SymbolKind::Structure)
    }

    fn imported_attribute_symbol_head_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(197);
        let imported_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        let local_symbol = ResolverSymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("{kind:?}/{spelling}/0")),
            FullyQualifiedName::new(format!("{}::{spelling}/0", module.path().as_str())),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                local_symbol,
                kind,
                NamespacePath::new(module.path().as_str()),
                spelling,
                SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(range(source, 0, 1)),
                    Vec::new(),
                ),
                local_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let attribute = ResolverSymbolId::new(
            imported_module.clone(),
            LocalSymbolId::new("Attribute/empty/0"),
            FullyQualifiedName::new(format!("{}::empty/0", imported_module.path().as_str())),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                attribute,
                SymbolKind::Attribute,
                NamespacePath::new(module.path().as_str()),
                "empty",
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 1, 2)),
                    Vec::new(),
                ),
                imported_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_attribute_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        let source = source_id(198);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 3)),
        );
        for (ordinal, (spelling, kind)) in [
            ("Struct", SymbolKind::Structure),
            ("empty", SymbolKind::Attribute),
            ("empty", SymbolKind::Attribute),
        ]
        .into_iter()
        .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_attribute_mode_chain_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        let source = source_id(200);
        let imported_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 2)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 2, 3)),
        );
        for (ordinal, spelling) in ["A", "B"].into_iter().enumerate() {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("Mode/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!("{}::{spelling}/0", module.path().as_str())),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    SymbolKind::Mode,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    local_contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        let attribute = ResolverSymbolId::new(
            imported_module.clone(),
            LocalSymbolId::new("Attribute/empty/0"),
            FullyQualifiedName::new(format!("{}::empty/0", imported_module.path().as_str())),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                attribute,
                SymbolKind::Attribute,
                NamespacePath::new(module.path().as_str()),
                "empty",
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 2, 3)),
                    Vec::new(),
                ),
                imported_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_attribute_mode_chain_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        let source = source_id(201);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 4)),
        );
        for (ordinal, (spelling, kind)) in [
            ("empty", SymbolKind::Attribute),
            ("empty", SymbolKind::Attribute),
            ("A", SymbolKind::Mode),
            ("B", SymbolKind::Mode),
        ]
        .into_iter()
        .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_attributed_mode_chain_symbol_env(
        module: ResolverModuleId,
        ambiguous_mode: &'static str,
    ) -> SymbolEnv {
        let source = source_id(186);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 5)),
        );
        for (ordinal, (spelling, kind)) in [
            ("empty", SymbolKind::Attribute),
            ("A", SymbolKind::Mode),
            ("B", SymbolKind::Mode),
        ]
        .into_iter()
        .chain(std::iter::once((ambiguous_mode, SymbolKind::Mode)))
        .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_attributed_structure_chain_symbol_env(
        module: ResolverModuleId,
        ambiguous_spelling: &'static str,
    ) -> SymbolEnv {
        let source = source_id(188);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 6)),
        );
        let ambiguous_kind = if ambiguous_spelling == "Struct" {
            SymbolKind::Structure
        } else {
            SymbolKind::Mode
        };
        for (ordinal, (spelling, kind)) in [
            ("empty", SymbolKind::Attribute),
            ("A", SymbolKind::Mode),
            ("B", SymbolKind::Mode),
            ("Struct", SymbolKind::Structure),
        ]
        .into_iter()
        .chain(std::iter::once((ambiguous_spelling, ambiguous_kind)))
        .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn source_symbol_pair_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(193);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 10)),
        );
        for (ordinal, (spelling, kind)) in [("empty", SymbolKind::Attribute), (spelling, kind)]
            .into_iter()
            .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/0")),
                FullyQualifiedName::new(format!("{}::{spelling}/0", module.path().as_str())),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_mode_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        ambiguous_symbol_env(module, "Mode", SymbolKind::Mode)
    }

    fn ambiguous_mode_chain_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        let source = source_id(189);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 5)),
        );
        for (ordinal, spelling) in ["A", "A", "B"].into_iter().enumerate() {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("Mode/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    SymbolKind::Mode,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn ambiguous_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        ambiguous_symbol_env(module, "Struct", SymbolKind::Structure)
    }

    fn ambiguous_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(192);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 5)),
        );
        for ordinal in 0..2 {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn source_local_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        source_local_symbols_env(module, &[(spelling, kind)])
    }

    fn source_local_symbols_env(
        module: ResolverModuleId,
        symbols: &[(&'static str, SymbolKind)],
    ) -> SymbolEnv {
        let source = source_id(190);
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        for (ordinal, (spelling, kind)) in symbols.iter().copied().enumerate() {
            let symbol = ResolverSymbolId::new(
                module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/0")),
                FullyQualifiedName::new(format!("{}::{spelling}/0", module.path().as_str())),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        module.clone(),
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            )
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_attribute_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_symbol_env(module, "empty", SymbolKind::Attribute)
    }

    fn imported_fixture_attribute_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_parser_fixture_symbol_env(module, "TypeCaseAttr", SymbolKind::Attribute)
    }

    fn imported_mode_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_symbol_env(module, "Mode", SymbolKind::Mode)
    }

    fn local_and_imported_mode_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
    ) -> SymbolEnv {
        local_and_imported_parser_fixture_symbol_env(module, spelling, SymbolKind::Mode)
    }

    fn local_and_imported_structure_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
    ) -> SymbolEnv {
        local_and_imported_parser_fixture_symbol_env(module, spelling, SymbolKind::Structure)
    }

    fn local_and_imported_attribute_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
    ) -> SymbolEnv {
        local_and_imported_parser_fixture_symbol_env(module, spelling, SymbolKind::Attribute)
    }

    fn local_structure_and_imported_fixture_attribute_symbol_env(
        module: ResolverModuleId,
        structure_spelling: &'static str,
        attribute_spelling: &'static str,
    ) -> SymbolEnv {
        let source = source_id(199);
        let imported_module = ResolverModuleId::new(
            module.package().clone(),
            ModulePath::new("parser.type_fixtures"),
        );
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        let structure = ResolverSymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("Structure/{structure_spelling}/0")),
            FullyQualifiedName::new(format!(
                "{}::{structure_spelling}/0",
                module.path().as_str()
            )),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                structure,
                SymbolKind::Structure,
                NamespacePath::new(module.path().as_str()),
                structure_spelling,
                SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(range(source, 0, 1)),
                    Vec::new(),
                ),
                local_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        let attribute = ResolverSymbolId::new(
            imported_module.clone(),
            LocalSymbolId::new(format!("Attribute/{attribute_spelling}/0")),
            FullyQualifiedName::new(format!(
                "{}::{attribute_spelling}/0",
                imported_module.path().as_str()
            )),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                attribute,
                SymbolKind::Attribute,
                NamespacePath::new(module.path().as_str()),
                attribute_spelling,
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 1, 2)),
                    Vec::new(),
                ),
                imported_contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        SymbolEnv::new(module, indexes)
    }

    fn local_and_imported_parser_fixture_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(198);
        let imported_module = ResolverModuleId::new(
            module.package().clone(),
            ModulePath::new("parser.type_fixtures"),
        );
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        for (ordinal, (symbol_module, contribution)) in [
            (module.clone(), local_contribution),
            (imported_module, imported_contribution),
        ]
        .into_iter()
        .enumerate()
        {
            let symbol = ResolverSymbolId::new(
                symbol_module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!(
                    "{}::{spelling}/{ordinal}",
                    symbol_module.path().as_str()
                )),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        symbol_module,
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_mode_chain_symbol_env(
        module: ResolverModuleId,
        imported_mode: &'static str,
    ) -> SymbolEnv {
        let source = source_id(185);
        let imported_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 2)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 2, 3)),
        );
        for (ordinal, (spelling, kind)) in [
            ("empty", SymbolKind::Attribute),
            ("A", SymbolKind::Mode),
            ("B", SymbolKind::Mode),
        ]
        .into_iter()
        .enumerate()
        {
            let is_imported = kind == SymbolKind::Mode && spelling == imported_mode;
            let symbol_module = if is_imported {
                imported_module.clone()
            } else {
                module.clone()
            };
            let contribution = if is_imported {
                imported_contribution
            } else {
                local_contribution
            };
            let symbol = ResolverSymbolId::new(
                symbol_module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!("{}::{spelling}/0", symbol_module.path().as_str())),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        symbol_module,
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_structure_mode_chain_symbol_env(
        module: ResolverModuleId,
        imported_spelling: &'static str,
    ) -> SymbolEnv {
        let source = source_id(187);
        let imported_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let local_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 2)),
        );
        let imported_contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 2, 3)),
        );
        for (ordinal, (spelling, kind)) in [
            ("empty", SymbolKind::Attribute),
            ("A", SymbolKind::Mode),
            ("B", SymbolKind::Mode),
            ("Struct", SymbolKind::Structure),
        ]
        .into_iter()
        .enumerate()
        {
            let is_imported = spelling == imported_spelling;
            let symbol_module = if is_imported {
                imported_module.clone()
            } else {
                module.clone()
            };
            let contribution = if is_imported {
                imported_contribution
            } else {
                local_contribution
            };
            let symbol = ResolverSymbolId::new(
                symbol_module.clone(),
                LocalSymbolId::new(format!("{kind:?}/{spelling}/{ordinal}")),
                FullyQualifiedName::new(format!("{}::{spelling}/0", symbol_module.path().as_str())),
            );
            indexes.symbols.insert(
                SymbolEntry::new(
                    symbol,
                    kind,
                    NamespacePath::new(module.path().as_str()),
                    spelling,
                    SemanticOrigin::new(
                        source,
                        symbol_module,
                        SourceAnchor::Range(range(source, ordinal, ordinal + 1)),
                        Vec::new(),
                    ),
                    contribution,
                )
                .with_visibility(Visibility::Public)
                .with_export_status(ExportStatus::Exported),
            );
        }
        SymbolEnv::new(module, indexes)
    }

    fn imported_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_symbol_env(module, "Struct", SymbolKind::Structure)
    }

    fn imported_fixture_structure_symbol_env(module: ResolverModuleId) -> SymbolEnv {
        imported_parser_fixture_symbol_env(module, "R", SymbolKind::Structure)
    }

    fn imported_parser_fixture_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(199);
        let imported_module = ResolverModuleId::new(
            module.package().clone(),
            ModulePath::new("parser.type_fixtures"),
        );
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let symbol = ResolverSymbolId::new(
            imported_module.clone(),
            LocalSymbolId::new(format!("{kind:?}/{spelling}/0")),
            FullyQualifiedName::new(format!("{}::{spelling}/0", imported_module.path().as_str())),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                symbol,
                kind,
                NamespacePath::new(module.path().as_str()),
                spelling,
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 0, 1)),
                    Vec::new(),
                ),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        SymbolEnv::new(module, indexes)
    }

    fn imported_symbol_env(
        module: ResolverModuleId,
        spelling: &'static str,
        kind: SymbolKind,
    ) -> SymbolEnv {
        let source = source_id(191);
        let imported_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("imported"));
        let mut indexes = SymbolEnvIndexes::default();
        let contribution = indexes.contributions.insert(
            imported_module.clone(),
            ContributionKind::ImportedSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let symbol = ResolverSymbolId::new(
            imported_module.clone(),
            LocalSymbolId::new(format!("{kind:?}/{spelling}/0")),
            FullyQualifiedName::new(format!("{}::{spelling}/0", imported_module.path().as_str())),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                symbol,
                kind,
                NamespacePath::new(module.path().as_str()),
                spelling,
                SemanticOrigin::new(
                    source,
                    imported_module,
                    SourceAnchor::Range(range(source, 0, 1)),
                    Vec::new(),
                ),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        SymbolEnv::new(module, indexes)
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

    #[derive(Debug, Clone)]
    struct ReserveItemSpec {
        names: Vec<&'static str>,
        type_shape: ReserveTypeShape,
    }

    #[derive(Debug, Clone, Copy)]
    struct ModeDefinitionSpec {
        pattern: &'static str,
        rhs_shape: ReserveTypeShape,
        local_context: bool,
        parameterized_pattern: bool,
        recovered: bool,
    }

    #[derive(Debug, Clone, Copy)]
    enum ReserveTypeShape {
        Builtin(&'static str),
        NonBuiltin(&'static str),
        QualifiedSymbol(&'static str),
        QualifiedSymbolWithArgs(&'static str),
        AttributedSetWithNamedAttribute(&'static str),
        AttributedObjectWithNamedAttribute(&'static str),
        AttributedQualifiedSymbolWithNamedAttribute(&'static str, &'static str),
        AttributedQualifiedSymbol(&'static str),
        AttributedQualifiedSymbolWithAttributeArgs(&'static str),
        QualifiedAttributeQualifiedSymbol(&'static str),
        AttributedSet,
        AttributedSetWithAttributeArgs,
        AttributedObject,
    }

    fn reserve_item(names: Vec<&'static str>, type_shape: ReserveTypeShape) -> ReserveItemSpec {
        ReserveItemSpec { names, type_shape }
    }

    const fn mode_definition(
        pattern: &'static str,
        rhs_shape: ReserveTypeShape,
    ) -> ModeDefinitionSpec {
        ModeDefinitionSpec {
            pattern,
            rhs_shape,
            local_context: false,
            parameterized_pattern: false,
            recovered: false,
        }
    }

    const fn contextual_mode_definition(
        pattern: &'static str,
        rhs_shape: ReserveTypeShape,
    ) -> ModeDefinitionSpec {
        ModeDefinitionSpec {
            pattern,
            rhs_shape,
            local_context: true,
            parameterized_pattern: false,
            recovered: false,
        }
    }

    const fn parameterized_mode_definition(
        pattern: &'static str,
        rhs_shape: ReserveTypeShape,
    ) -> ModeDefinitionSpec {
        ModeDefinitionSpec {
            pattern,
            rhs_shape,
            local_context: false,
            parameterized_pattern: true,
            recovered: false,
        }
    }

    const fn recovered_mode_definition(
        pattern: &'static str,
        rhs_shape: ReserveTypeShape,
    ) -> ModeDefinitionSpec {
        ModeDefinitionSpec {
            pattern,
            rhs_shape,
            local_context: false,
            parameterized_pattern: false,
            recovered: true,
        }
    }

    fn mode_chain_reserve_ast(
        source_id: SourceId,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        mode_chain_reserve_ast_with_order(source_id, [], modes, [], items)
    }

    fn mode_chain_reserve_ast_with_structures(
        source_id: SourceId,
        structures: impl IntoIterator<Item = &'static str>,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        mode_chain_reserve_ast_with_order(source_id, structures, modes, [], items)
    }

    fn mode_chain_reserve_ast_with_trailing_structures(
        source_id: SourceId,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        structures: impl IntoIterator<Item = &'static str>,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        mode_chain_reserve_ast_with_order(source_id, [], modes, structures, items)
    }

    fn mode_chain_reserve_ast_with_order(
        source_id: SourceId,
        leading_structures: impl IntoIterator<Item = &'static str>,
        modes: impl IntoIterator<Item = ModeDefinitionSpec>,
        trailing_structures: impl IntoIterator<Item = &'static str>,
        items: Vec<ReserveItemSpec>,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut root_children = Vec::new();
        let mut offset = 0;
        for structure in leading_structures {
            root_children.push(add_structure_definition_item(
                &mut builder,
                source_id,
                &mut offset,
                structure,
            ));
        }
        for mode in modes {
            root_children.push(add_mode_definition_item(
                &mut builder,
                source_id,
                &mut offset,
                mode,
            ));
        }
        for structure in trailing_structures {
            root_children.push(add_structure_definition_item(
                &mut builder,
                source_id,
                &mut offset,
                structure,
            ));
        }
        root_children.extend(add_reserve_items(
            &mut builder,
            source_id,
            &mut offset,
            items,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn reserve_ast(source_id: SourceId, items: Vec<ReserveItemSpec>) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn builtin_equality_theorem_ast(
        source_id: SourceId,
        label: &str,
        left: &str,
        right: &str,
    ) -> SurfaceAst {
        builtin_binary_theorem_ast(source_id, label, left, "=", right)
    }

    fn builtin_binary_theorem_ast(
        source_id: SourceId,
        label: &str,
        left: &str,
        operator: &str,
        right: &str,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let theorem = add_builtin_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            label,
            left,
            operator,
            right,
        );
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            vec![theorem],
        );
        builder.finish(Some(root), None)
    }

    fn reserve_then_builtin_equality_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        label: &str,
        left: &str,
        right: &str,
    ) -> SurfaceAst {
        reserve_then_builtin_binary_theorem_ast(source_id, items, label, left, "=", right)
    }

    fn reserve_then_builtin_binary_theorem_ast(
        source_id: SourceId,
        items: Vec<ReserveItemSpec>,
        label: &str,
        left: &str,
        operator: &str,
        right: &str,
    ) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut offset = 0;
        let mut root_children = add_reserve_items(&mut builder, source_id, &mut offset, items);
        root_children.push(add_builtin_binary_theorem_item(
            &mut builder,
            source_id,
            &mut offset,
            label,
            left,
            operator,
            right,
        ));
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
    }

    fn add_builtin_binary_theorem_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        label: &str,
        left: &str,
        operator: &str,
        right: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let theorem = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "theorem",
        );
        let label_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            label,
        );
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let formula_start = *offset;
        let left_term = add_numeral_term_expression(builder, source_id, offset, left);
        let operator = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            operator,
        );
        let right_term = add_numeral_term_expression(builder, source_id, offset, right);
        let formula_end = builder
            .node_range(right_term)
            .expect("just-created right term should exist")
            .end;
        let formula = builder.add_node(
            SurfaceNodeKind::BuiltinPredicateApplication,
            range(source_id, formula_start, formula_end),
            vec![left_term, operator, right_term],
        );
        let formula_expression = builder.add_node(
            SurfaceNodeKind::FormulaExpression,
            range(source_id, formula_start, formula_end),
            vec![formula],
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::TheoremItem,
            range(source_id, start, end),
            vec![theorem, label_token, colon, formula_expression, semicolon],
        )
    }

    fn add_numeral_term_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        spelling: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Numeral,
            spelling,
        );
        let end = builder
            .node_range(token)
            .expect("just-created numeral token should exist")
            .end;
        let numeral = builder.add_node(
            SurfaceNodeKind::NumeralTerm,
            range(source_id, start, end),
            vec![token],
        );
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            range(source_id, start, end),
            vec![numeral],
        )
    }

    fn add_reserve_items(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        items: Vec<ReserveItemSpec>,
    ) -> Vec<SurfaceBuilderNodeId> {
        let mut root_children = Vec::new();
        for item in items {
            let item_start = *offset;
            let reserve = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "reserve",
            );
            let segment_start = *offset;
            let mut segment_children = Vec::new();
            for (index, name) in item.names.iter().enumerate() {
                segment_children.push(add_token(
                    builder,
                    source_id,
                    offset,
                    SurfaceTokenKind::Identifier,
                    name,
                ));
                if index + 1 != item.names.len() {
                    segment_children.push(add_token(
                        builder,
                        source_id,
                        offset,
                        SurfaceTokenKind::ReservedSymbol,
                        ",",
                    ));
                }
            }
            segment_children.push(add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "for",
            ));
            let type_expression =
                add_reserve_type_expression(builder, source_id, offset, item.type_shape);
            segment_children.push(type_expression);
            let segment_end = builder
                .node_range(type_expression)
                .expect("just-created type expression should exist")
                .end;
            let segment = builder.add_node(
                SurfaceNodeKind::ReserveSegment,
                range(source_id, segment_start, segment_end),
                segment_children,
            );
            let semicolon = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                ";",
            );
            let item_end = builder
                .node_range(semicolon)
                .expect("just-created semicolon should exist")
                .end;
            let reserve_item = builder.add_node(
                SurfaceNodeKind::ReserveItem,
                range(source_id, item_start, item_end),
                vec![reserve, segment, semicolon],
            );
            root_children.push(reserve_item);
        }
        root_children
    }

    fn add_mode_definition_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        mode: ModeDefinitionSpec,
    ) -> SurfaceBuilderNodeId {
        let item_start = *offset;
        let definition = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "definition",
        );
        let mut block_children = vec![definition];
        if mode.local_context {
            block_children.push(add_definition_parameter(builder, source_id, offset));
        }
        let mode_start = *offset;
        let mode_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "mode",
        );
        let label = format!("{}Def", mode.pattern);
        let label_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            &label,
        );
        let colon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ":",
        );
        let pattern_start = *offset;
        let pattern_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            mode.pattern,
        );
        let mut pattern_children = vec![pattern_token];
        let mut pattern_end = pattern_start + mode.pattern.len();
        if mode.parameterized_pattern {
            let of = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "of",
            );
            let arg = add_simple_type_expression(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "set",
            );
            pattern_end = builder
                .node_range(arg)
                .expect("just-created pattern argument should exist")
                .end;
            let pattern_args = builder.add_node(
                SurfaceNodeKind::TypeArguments,
                range(
                    source_id,
                    pattern_start + mode.pattern.len() + 1,
                    pattern_end,
                ),
                vec![of, arg],
            );
            pattern_children.push(pattern_args);
        }
        let pattern = builder.add_node(
            SurfaceNodeKind::ModePattern,
            range(source_id, pattern_start, pattern_end),
            pattern_children,
        );
        let is = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "is",
        );
        let rhs = add_reserve_type_expression(builder, source_id, offset, mode.rhs_shape);
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let mode_end = builder
            .node_range(semicolon)
            .expect("just-created semicolon should exist")
            .end;
        let mut mode_definition_children = vec![mode_token, label_token, colon, pattern, is, rhs];
        if mode.recovered {
            let recovery = builder.add_recovery(
                SyntaxRecoveryKind::MissingTerm,
                range(source_id, mode_end, mode_end),
                Vec::new(),
            );
            mode_definition_children.push(recovery);
        }
        mode_definition_children.push(semicolon);
        let mode_definition = builder.add_node(
            SurfaceNodeKind::ModeDefinition,
            range(source_id, mode_start, mode_end),
            mode_definition_children,
        );
        let end = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "end",
        );
        let block_semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let item_end = builder
            .node_range(block_semicolon)
            .expect("just-created block semicolon should exist")
            .end;
        block_children.extend([mode_definition, end, block_semicolon]);
        builder.add_node(
            SurfaceNodeKind::DefinitionBlockItem,
            range(source_id, item_start, item_end),
            block_children,
        )
    }

    fn add_definition_parameter(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
    ) -> SurfaceBuilderNodeId {
        let parameter_start = *offset;
        let let_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "let",
        );
        let name = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            "x",
        );
        let be = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "be",
        );
        let ty = add_simple_type_expression(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "set",
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let parameter_end = builder
            .node_range(semicolon)
            .expect("just-created definition parameter semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::DefinitionParameter,
            range(source_id, parameter_start, parameter_end),
            vec![let_token, name, be, ty, semicolon],
        )
    }

    fn add_structure_definition_item(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        structure: &'static str,
    ) -> SurfaceBuilderNodeId {
        let item_start = *offset;
        let definition = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "definition",
        );
        let structure_start = *offset;
        let struct_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "struct",
        );
        let pattern_start = *offset;
        let pattern_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            structure,
        );
        let pattern = builder.add_node(
            SurfaceNodeKind::StructurePattern,
            range(source_id, pattern_start, pattern_start + structure.len()),
            vec![pattern_token],
        );
        let where_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "where",
        );
        let field = add_structure_field(builder, source_id, offset);
        let end = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "end",
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let structure_end = builder
            .node_range(semicolon)
            .expect("just-created structure semicolon should exist")
            .end;
        let structure_definition = builder.add_node(
            SurfaceNodeKind::StructureDefinition,
            range(source_id, structure_start, structure_end),
            vec![struct_token, pattern, where_token, field, end, semicolon],
        );
        let block_end = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "end",
        );
        let block_semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let item_end = builder
            .node_range(block_semicolon)
            .expect("just-created block semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::DefinitionBlockItem,
            range(source_id, item_start, item_end),
            vec![definition, structure_definition, block_end, block_semicolon],
        )
    }

    fn add_structure_field(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
    ) -> SurfaceBuilderNodeId {
        let field_start = *offset;
        let field = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "field",
        );
        let name = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::Identifier,
            "carrier",
        );
        let arrow = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            "->",
        );
        let field_type = add_simple_type_expression(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "set",
        );
        let semicolon = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedSymbol,
            ";",
        );
        let field_end = builder
            .node_range(semicolon)
            .expect("just-created field semicolon should exist")
            .end;
        builder.add_node(
            SurfaceNodeKind::StructureField,
            range(source_id, field_start, field_end),
            vec![field, name, arrow, field_type, semicolon],
        )
    }

    fn add_reserve_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        shape: ReserveTypeShape,
    ) -> SurfaceBuilderNodeId {
        match shape {
            ReserveTypeShape::Builtin(head) => add_simple_type_expression(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                head,
            ),
            ReserveTypeShape::NonBuiltin(head) => add_simple_type_expression(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::UserSymbol,
                head,
            ),
            ReserveTypeShape::QualifiedSymbol(head) => {
                add_qualified_type_expression(builder, source_id, offset, head, false)
            }
            ReserveTypeShape::QualifiedSymbolWithArgs(head) => {
                add_qualified_type_expression(builder, source_id, offset, head, true)
            }
            ReserveTypeShape::AttributedSetWithNamedAttribute(attribute) => {
                attributed_type_expression_with_attribute(
                    builder, source_id, offset, attribute, "set", false, false,
                )
            }
            ReserveTypeShape::AttributedObjectWithNamedAttribute(attribute) => {
                attributed_type_expression_with_attribute(
                    builder, source_id, offset, attribute, "object", false, false,
                )
            }
            ReserveTypeShape::AttributedQualifiedSymbolWithNamedAttribute(attribute, head) => {
                add_attributed_qualified_type_expression_with_attribute(
                    builder, source_id, offset, attribute, head, false, false,
                )
            }
            ReserveTypeShape::AttributedQualifiedSymbol(head) => {
                add_attributed_qualified_type_expression(
                    builder, source_id, offset, head, false, false,
                )
            }
            ReserveTypeShape::AttributedQualifiedSymbolWithAttributeArgs(head) => {
                add_attributed_qualified_type_expression(
                    builder, source_id, offset, head, true, false,
                )
            }
            ReserveTypeShape::QualifiedAttributeQualifiedSymbol(head) => {
                add_attributed_qualified_type_expression(
                    builder, source_id, offset, head, false, true,
                )
            }
            ReserveTypeShape::AttributedSet => {
                attributed_type_expression(builder, source_id, offset, "set", false)
            }
            ReserveTypeShape::AttributedSetWithAttributeArgs => {
                attributed_type_expression(builder, source_id, offset, "set", true)
            }
            ReserveTypeShape::AttributedObject => {
                attributed_type_expression(builder, source_id, offset, "object", false)
            }
        }
    }

    fn add_simple_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        token_kind: SurfaceTokenKind,
        head: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let token = add_token(builder, source_id, offset, token_kind, head);
        let type_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, start, start + head.len()),
            vec![token],
        );
        builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, start, start + head.len()),
            vec![type_head],
        )
    }

    fn add_qualified_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        head: &str,
        with_args: bool,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let (type_head, end) = add_qualified_type_head(builder, source_id, offset, head, with_args);
        builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, start, end),
            vec![type_head],
        )
    }

    fn add_qualified_type_head(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        head: &str,
        with_args: bool,
    ) -> (SurfaceBuilderNodeId, usize) {
        let start = *offset;
        let head_start = *offset;
        let token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::UserSymbol,
            head,
        );
        let segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, head_start, head_start + head.len()),
            vec![token],
        );
        let symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, head_start, head_start + head.len()),
            vec![segment],
        );
        let mut type_head_children = vec![symbol];
        let mut end = head_start + head.len();
        if with_args {
            let of = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "of",
            );
            let arg = add_simple_type_expression(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "set",
            );
            end = builder
                .node_range(arg)
                .expect("just-created type argument should exist")
                .end;
            let type_args = builder.add_node(
                SurfaceNodeKind::TypeArguments,
                range(source_id, head_start + head.len() + 1, end),
                vec![of, arg],
            );
            type_head_children.push(type_args);
        }
        let type_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, start, end),
            type_head_children,
        );
        (type_head, end)
    }

    fn attributed_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        head: &str,
        with_attribute_args: bool,
    ) -> SurfaceBuilderNodeId {
        attributed_type_expression_with_attribute(
            builder,
            source_id,
            offset,
            "empty",
            head,
            with_attribute_args,
            true,
        )
    }

    fn attributed_type_expression_with_attribute(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        attribute: &str,
        head: &str,
        with_attribute_args: bool,
        include_non: bool,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let mut attribute_children = Vec::new();
        if include_non {
            attribute_children.push(add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "non",
            ));
        }
        let attribute_start = *offset;
        let attribute_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::UserSymbol,
            attribute,
        );
        let attribute_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(
                source_id,
                attribute_start,
                attribute_start + attribute.len(),
            ),
            vec![attribute_token],
        );
        let attribute_symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(
                source_id,
                attribute_start,
                attribute_start + attribute.len(),
            ),
            vec![attribute_segment],
        );
        let mut attribute_end = attribute_start + attribute.len();
        attribute_children.push(attribute_symbol);
        if with_attribute_args {
            let open = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                "(",
            );
            let arg = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                "x",
            );
            let close = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                ")",
            );
            attribute_end = builder
                .node_range(close)
                .expect("just-created attribute argument close should exist")
                .end;
            attribute_children.extend([open, arg, close]);
        }
        let head_start = *offset;
        let head_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            head,
        );
        let attribute = builder.add_node(
            SurfaceNodeKind::AttributeRef,
            range(source_id, start, attribute_end),
            attribute_children,
        );
        let attribute_chain = builder.add_node(
            SurfaceNodeKind::AttributeChain,
            range(source_id, start, attribute_end),
            vec![attribute],
        );
        let type_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, head_start, head_start + head.len()),
            vec![head_token],
        );
        builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, start, head_start + head.len()),
            vec![attribute_chain, type_head],
        )
    }

    fn add_attributed_qualified_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        head: &str,
        with_attribute_args: bool,
        qualified_attribute: bool,
    ) -> SurfaceBuilderNodeId {
        add_attributed_qualified_type_expression_with_attribute(
            builder,
            source_id,
            offset,
            "empty",
            head,
            with_attribute_args,
            qualified_attribute,
        )
    }

    fn add_attributed_qualified_type_expression_with_attribute(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        attribute_name: &str,
        head: &str,
        with_attribute_args: bool,
        qualified_attribute: bool,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let mut attribute_children = Vec::new();
        let include_non = attribute_name == "empty";
        if include_non {
            attribute_children.push(add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedWord,
                "non",
            ));
        }
        let attribute_symbol = add_attribute_symbol(
            builder,
            source_id,
            offset,
            attribute_name,
            qualified_attribute,
        );
        let mut attribute_end = builder
            .node_range(attribute_symbol)
            .expect("just-created attribute symbol should exist")
            .end;
        attribute_children.push(attribute_symbol);
        if with_attribute_args {
            let open = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                "(",
            );
            let arg = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::Identifier,
                "x",
            );
            let close = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                ")",
            );
            attribute_end = builder
                .node_range(close)
                .expect("just-created attribute argument close should exist")
                .end;
            attribute_children.extend([open, arg, close]);
        }
        let attribute = builder.add_node(
            SurfaceNodeKind::AttributeRef,
            range(source_id, start, attribute_end),
            attribute_children,
        );
        let attribute_chain = builder.add_node(
            SurfaceNodeKind::AttributeChain,
            range(source_id, start, attribute_end),
            vec![attribute],
        );
        let (head_node, end) = add_qualified_type_head(builder, source_id, offset, head, false);
        builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, start, end),
            vec![attribute_chain, head_node],
        )
    }

    fn add_attribute_symbol(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        attribute: &str,
        qualified: bool,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let mut children = Vec::new();
        if qualified {
            let qualifier = "Struct";
            let qualifier_start = *offset;
            let qualifier_token = add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::UserSymbol,
                qualifier,
            );
            children.push(builder.add_node(
                SurfaceNodeKind::PathSegment,
                range(
                    source_id,
                    qualifier_start,
                    qualifier_start + qualifier.len(),
                ),
                vec![qualifier_token],
            ));
            children.push(add_token(
                builder,
                source_id,
                offset,
                SurfaceTokenKind::ReservedSymbol,
                ".",
            ));
        }
        let attribute_start = *offset;
        let attribute_token = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::UserSymbol,
            attribute,
        );
        let attribute_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(
                source_id,
                attribute_start,
                attribute_start + attribute.len(),
            ),
            vec![attribute_token],
        );
        children.push(attribute_segment);
        builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, start, attribute_start + attribute.len()),
            children,
        )
    }

    fn add_token(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
        kind: SurfaceTokenKind,
        text: &str,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let end = start + text.len();
        let token = builder.add_token(kind, text, range(source_id, start, end));
        *offset = end + 1;
        token
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
