use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use mizar_checker::binding_env::{
    BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
    BindingContextOwner, BindingContextRecovery, BindingContextTable, BindingDiagnosticTable,
    BindingDraft, BindingEnv, BindingEnvParts, BindingId, BindingKind, BindingRecoveryState,
    BindingStatus, BindingTable, BindingTypeSite, CapturedFreeVariables,
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
    DeclarationChecker, DeclarationCheckingOutput, DeclarationContextInput, DeclarationInput,
    DeclarationKind, DeclarationStatus, ReservedDefaultPayload, TypeExpressionInput, TypeHeadInput,
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
use mizar_resolve::env::{NamespacePath, SymbolEnv};
use mizar_resolve::resolved_ast::ModuleId as ResolverModuleId;
use mizar_resolve::symbols::{
    SignatureProjectionExtractor, SymbolCollector, SymbolDiagnostic, SymbolDiagnosticClass,
};
use mizar_session::{
    BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
    SourceAnchor, SourceInput, SourceOriginInput, SourceRange, normalize_path,
};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeKind, SurfaceTokenKind};

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
const SOURCE_RESERVE_MODULE_CONTEXT: BindingContextId = BindingContextId::new(0);

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

fn declaration_symbol_detail_keys(
    workspace_root: &Path,
    case: &TestCase,
    output: FrontendRun,
) -> Vec<String> {
    let frontend_diagnostic_keys = frontend_detail_keys(case, &output.diagnostics);
    if !frontend_diagnostic_keys.is_empty() {
        return frontend_diagnostic_keys;
    }

    let Some(ast) = output.ast else {
        return vec!["declaration_symbol.no_ast".to_owned()];
    };
    resolver_symbol_collection(workspace_root, case, &ast).detail_keys
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

    source_type_elaboration_detail_keys(&ast, resolver.module, &resolver.env)
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

fn source_type_elaboration_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Vec<String> {
    let Ok(source_reserve) = extract_builtin_source_reserve_declarations(ast) else {
        return vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()];
    };
    let handoff = match assemble_source_reserve_checker_handoff(
        ast.source_id,
        module,
        symbols,
        &source_reserve,
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
    if let Err(error) = assert_source_reserve_handoff(&handoff, &source_reserve) {
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
    if assert_source_reserve_core_context_readiness(&handoff, &source_reserve).is_err() {
        return vec!["type_elaboration.core.context_invalid".to_owned()];
    }
    Vec::new()
}

#[derive(Debug, Clone)]
struct SourceReserveBridge {
    bindings: Vec<SourceReserveBinding>,
    source_range: SourceRange,
}

impl SourceReserveBridge {
    fn root_node(&self) -> TypedNodeId {
        TypedNodeId::new(self.bindings.len() * 2)
    }

    fn type_node(&self, index: usize) -> TypedNodeId {
        TypedNodeId::new(index * 2)
    }

    fn declaration_node(&self, index: usize) -> TypedNodeId {
        TypedNodeId::new(index * 2 + 1)
    }
}

#[derive(Debug, Clone)]
struct SourceReserveBinding {
    spelling: String,
    binding_range: SourceRange,
    type_range: SourceRange,
    type_spelling: String,
    type_head: TypeHeadInput,
}

#[derive(Debug)]
struct SourceReserveHandoff {
    binding_env: BindingEnv,
    declarations: DeclarationCheckingOutput,
    typed_ast: TypedAst,
    resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone)]
struct SourceTypeExpression {
    range: SourceRange,
    spelling: String,
    head: TypeHeadInput,
}

fn extract_builtin_source_reserve_declarations(
    ast: &SurfaceAst,
) -> Result<SourceReserveBridge, ()> {
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
            bindings.extend(extract_builtin_reserve_segment(ast, segment)?);
        }
    }

    if bindings.is_empty() {
        return Err(());
    }
    Ok(SourceReserveBridge {
        bindings,
        source_range: source_range.expect("reserve_items was non-empty"),
    })
}

fn is_supported_builtin_reserve_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::Token(_)
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::ReserveItem
            | SurfaceNodeKind::ReserveSegment
            | SurfaceNodeKind::TypeExpression
            | SurfaceNodeKind::TypeHead
    )
}

fn extract_builtin_reserve_segment(
    ast: &SurfaceAst,
    segment: &SurfaceNode,
) -> Result<Vec<SourceReserveBinding>, ()> {
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
                type_expression = Some(extract_builtin_source_type_expression(ast, child)?);
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
        .map(|(spelling, binding_range)| SourceReserveBinding {
            spelling,
            binding_range,
            type_range: type_expression.range,
            type_spelling: type_expression.spelling.clone(),
            type_head: type_expression.head.clone(),
        })
        .collect())
}

fn extract_builtin_source_type_expression(
    ast: &SurfaceAst,
    node: &SurfaceNode,
) -> Result<SourceTypeExpression, ()> {
    if subtree_has_recovery(ast, node) || node.children.len() != 1 {
        return Err(());
    }
    let head = ast.node(node.children[0]).ok_or(())?;
    if !matches!(head.kind, SurfaceNodeKind::TypeHead) || head.children.len() != 1 {
        return Err(());
    }
    let token = ast.node(head.children[0]).and_then(SurfaceNode::token_text);
    let head = match token {
        Some("set") => TypeHeadInput::BuiltinSet,
        Some("object") => TypeHeadInput::BuiltinObject,
        _ => return Err(()),
    };
    Ok(SourceTypeExpression {
        range: node.range,
        spelling: token.expect("builtin token matched").to_owned(),
        head,
    })
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
    source_id: mizar_session::SourceId,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
    source_reserve: &SourceReserveBridge,
) -> Result<SourceReserveHandoff, String> {
    let binding_env =
        assemble_source_reserve_binding_env(source_id, module.clone(), source_reserve)?;
    let context_inputs = vec![DeclarationContextInput::new(
        SOURCE_RESERVE_MODULE_CONTEXT,
        TypedSiteRef::Node(source_reserve.root_node()),
        source_reserve.source_range,
    )];
    let declaration_inputs = source_reserve
        .bindings
        .iter()
        .enumerate()
        .map(|(index, binding)| {
            let type_site = TypedSiteRef::Node(source_reserve.type_node(index));
            DeclarationInput::new(
                BindingId::new(index),
                SOURCE_RESERVE_MODULE_CONTEXT,
                TypedSiteRef::Node(source_reserve.declaration_node(index)),
                binding.binding_range,
                DeclarationKind::ReservedVariable,
            )
            .with_type_expression(TypeExpressionInput::new(
                type_site.clone(),
                binding.type_range,
                binding.type_spelling.clone(),
                binding.type_head.clone(),
            ))
            .with_reserved_default(ReservedDefaultPayload::new(type_site, false))
        })
        .collect::<Vec<_>>();
    let declarations = DeclarationChecker::default().check(
        symbols,
        &binding_env,
        context_inputs,
        declaration_inputs,
    );
    let typed_ast =
        assemble_source_reserve_typed_ast(source_id, module, source_reserve, &declarations)?;
    let resolved = assemble_source_reserve_resolved_typed_ast(&typed_ast, source_reserve)
        .map_err(|error| error.to_string())?;

    Ok(SourceReserveHandoff {
        binding_env,
        declarations,
        typed_ast,
        resolved,
    })
}

fn assemble_source_reserve_binding_env(
    source_id: mizar_session::SourceId,
    module: ResolverModuleId,
    source_reserve: &SourceReserveBridge,
) -> Result<BindingEnv, String> {
    let binding_ids = (0..source_reserve.bindings.len())
        .map(BindingId::new)
        .collect::<Vec<_>>();
    let mut contexts = BindingContextTable::new();
    contexts.insert(BindingContextDraft {
        owner: BindingContextOwner::Module,
        parent: None,
        layer: BindingContextLayer::Module,
        lexical_scope: None,
        bindings: binding_ids.clone(),
        visible_bindings: binding_ids,
        recovery: BindingContextRecovery::Normal,
    });

    let mut bindings = BindingTable::new();
    for (index, binding) in source_reserve.bindings.iter().enumerate() {
        bindings.insert(BindingDraft {
            spelling: binding.spelling.clone(),
            kind: BindingKind::ReservedVariable,
            identity: BinderIdentity::ReservedVariable {
                spelling: binding.spelling.clone(),
                declaration_range: binding.binding_range,
            },
            owner_context: SOURCE_RESERVE_MODULE_CONTEXT,
            declaration_range: binding.binding_range,
            visible_after_ordinal: index,
            type_site: BindingTypeSite::Source(binding.type_range),
            status: BindingStatus::Reserved,
            captured: CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: BindingRecoveryState::Normal,
        });
    }

    BindingEnv::try_new(BindingEnvParts {
        source_id,
        module_id: module,
        contexts,
        bindings,
        diagnostics: BindingDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())
}

fn assemble_source_reserve_resolved_typed_ast(
    typed_ast: &TypedAst,
    source_reserve: &SourceReserveBridge,
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
        .bindings
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
    for (index, _) in source_reserve.bindings.iter().enumerate() {
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
    source_reserve: &SourceReserveBridge,
) -> Result<(), String> {
    let expected_bindings = source_reserve.bindings.len();
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
        .get(SOURCE_RESERVE_MODULE_CONTEXT)
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

    for (index, source_binding) in source_reserve.bindings.iter().enumerate() {
        let binding = handoff
            .binding_env
            .bindings()
            .get(BindingId::new(index))
            .ok_or_else(|| format!("missing source reserve binding {index}"))?;
        if binding.spelling != source_binding.spelling
            || binding.kind != BindingKind::ReservedVariable
            || binding.owner_context != SOURCE_RESERVE_MODULE_CONTEXT
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
    source_reserve: &SourceReserveBridge,
) -> Result<(), String> {
    let summary = ResolvedTypedAstSummary::from_ast(&handoff.resolved);
    let mut input = CoreContextInput::new(summary);

    for (index, source_binding) in source_reserve.bindings.iter().enumerate() {
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
    if context.binder_sources().iter().count() != source_reserve.bindings.len()
        || context.binder_context().free_variables.len() != source_reserve.bindings.len()
    {
        return Err("core context binding count mismatch".to_owned());
    }

    for (index, source_binding) in source_reserve.bindings.iter().enumerate() {
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
    source_id: mizar_session::SourceId,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
    source_reserve: &SourceReserveBridge,
) -> Result<SourceReserveHandoff, String> {
    let handoff =
        assemble_source_reserve_checker_handoff(source_id, module, symbols, source_reserve)?;
    assert_source_reserve_handoff(&handoff, source_reserve)?;
    assert_source_reserve_core_summary_readiness(&handoff)?;
    assert_source_reserve_core_context_readiness(&handoff, source_reserve)?;
    Ok(handoff)
}

fn assemble_source_reserve_typed_ast(
    source_id: mizar_session::SourceId,
    module: ResolverModuleId,
    source_reserve: &SourceReserveBridge,
    output: &DeclarationCheckingOutput,
) -> Result<TypedAst, String> {
    if source_reserve.bindings.is_empty() {
        return Err("source reserve bridge produced no bindings".to_owned());
    }
    let declarations_by_binding = output
        .declarations()
        .iter()
        .map(|(_, declaration)| (declaration.binding, declaration))
        .collect::<BTreeMap<_, _>>();
    let mut builder = TypedArenaBuilder::new();
    let mut declaration_nodes = Vec::new();
    for (index, source_binding) in source_reserve.bindings.iter().enumerate() {
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
                SourceAnchor::Range(source_reserve.source_range),
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
        source_id,
        module_id: module,
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
            "declaration-symbol case `{}` expected detail keys {:?} but got {:?}",
            case.id.0,
            expected_declaration_symbol_detail_keys(case),
            result.actual_detail_keys
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
        source_type_elaboration_detail_keys,
    };
    use mizar_checker::binding_env::BindingId;
    use mizar_checker::resolved_typed_ast::{ResolvedTypedNodeId, ResolvedTypedNodeKind};
    use mizar_checker::type_checker::TypeHeadInput;
    use mizar_checker::typed_ast::LocalTypeContextId;
    use mizar_core::elaborator::ResolvedTypedAstSummary;
    use mizar_frontend::lexical_env::{
        ExportedOperatorAssociativity, ExportedOperatorFixity, ExportedOperatorMetadata,
        LexicalEnvironmentRequest, LexicalSummaryProvider, UserSymbolKind,
    };
    use mizar_frontend::preprocess::{ImportStub, ImportStubPath};
    use mizar_resolve::env::{SymbolEnv, SymbolEnvIndexes};
    use mizar_resolve::resolved_ast::ModuleId as ResolverModuleId;
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SourceId, SourceRange,
    };
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

        let source_reserve = extract_builtin_source_reserve_declarations(&ast)
            .expect("builtin reserve declarations should extract");

        assert_eq!(
            source_reserve
                .bindings
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
            extract_builtin_source_reserve_declarations(&non_builtin).is_err(),
            "non-builtin type heads must stay on the external gap"
        );

        let unsupported = reserve_ast(
            source_id(93),
            vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
        );

        assert!(
            extract_builtin_source_reserve_declarations(&unsupported).is_err(),
            "attribute-bearing type expressions must stay on the external gap"
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
        let source_reserve = extract_builtin_source_reserve_declarations(&ast)
            .expect("builtin reserve declarations should extract");
        let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());

        let handoff = assemble_source_checker_handoff(source_id, module, &symbols, &source_reserve)
            .expect("source-derived checker handoff should reach ResolvedTypedAst");
        let resolved = &handoff.resolved;

        assert_eq!(handoff.binding_env.bindings().len(), 3);
        let module_context = handoff
            .binding_env
            .contexts()
            .get(super::SOURCE_RESERVE_MODULE_CONTEXT)
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
            source_reserve.bindings[0].type_range,
            source_reserve.bindings[1].type_range
        );
        for index in 0..source_reserve.bindings.len() {
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
    fn source_reserve_bridge_reports_external_gap_detail_for_unsupported_and_mixed_shapes() {
        let source_id = source_id(95);
        let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let non_builtin = reserve_ast(
            source_id,
            vec![reserve_item(vec!["x"], ReserveTypeShape::NonBuiltin("T"))],
        );
        assert_eq!(
            source_type_elaboration_detail_keys(&non_builtin, module.clone(), &symbols),
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
            vec![TYPE_ELABORATION_PAYLOAD_EXTRACTION_GAP_KEY.to_owned()]
        );
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
    enum ReserveTypeShape {
        Builtin(&'static str),
        NonBuiltin(&'static str),
        AttributedSet,
    }

    fn reserve_item(names: Vec<&'static str>, type_shape: ReserveTypeShape) -> ReserveItemSpec {
        ReserveItemSpec { names, type_shape }
    }

    fn reserve_ast(source_id: SourceId, items: Vec<ReserveItemSpec>) -> SurfaceAst {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let mut root_children = Vec::new();
        let mut offset = 0;
        for item in items {
            let item_start = offset;
            let reserve = add_token(
                &mut builder,
                source_id,
                &mut offset,
                SurfaceTokenKind::ReservedWord,
                "reserve",
            );
            let segment_start = offset;
            let mut segment_children = Vec::new();
            for (index, name) in item.names.iter().enumerate() {
                segment_children.push(add_token(
                    &mut builder,
                    source_id,
                    &mut offset,
                    SurfaceTokenKind::Identifier,
                    name,
                ));
                if index + 1 != item.names.len() {
                    segment_children.push(add_token(
                        &mut builder,
                        source_id,
                        &mut offset,
                        SurfaceTokenKind::ReservedSymbol,
                        ",",
                    ));
                }
            }
            segment_children.push(add_token(
                &mut builder,
                source_id,
                &mut offset,
                SurfaceTokenKind::ReservedWord,
                "for",
            ));
            let type_expression =
                add_reserve_type_expression(&mut builder, source_id, &mut offset, item.type_shape);
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
                &mut builder,
                source_id,
                &mut offset,
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
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, offset.saturating_sub(2)),
            root_children,
        );
        builder.finish(Some(root), None)
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
            ReserveTypeShape::AttributedSet => {
                attributed_type_expression(builder, source_id, offset)
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

    fn attributed_type_expression(
        builder: &mut SurfaceAstBuilder,
        source_id: SourceId,
        offset: &mut usize,
    ) -> SurfaceBuilderNodeId {
        let start = *offset;
        let non = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "non",
        );
        let empty_start = *offset;
        let empty = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::UserSymbol,
            "empty",
        );
        let set_start = *offset;
        let set = add_token(
            builder,
            source_id,
            offset,
            SurfaceTokenKind::ReservedWord,
            "set",
        );
        let empty_segment = builder.add_node(
            SurfaceNodeKind::PathSegment,
            range(source_id, empty_start, empty_start + "empty".len()),
            vec![empty],
        );
        let empty_symbol = builder.add_node(
            SurfaceNodeKind::QualifiedSymbol,
            range(source_id, empty_start, empty_start + "empty".len()),
            vec![empty_segment],
        );
        let attribute = builder.add_node(
            SurfaceNodeKind::AttributeRef,
            range(source_id, start, empty_start + "empty".len()),
            vec![non, empty_symbol],
        );
        let attribute_chain = builder.add_node(
            SurfaceNodeKind::AttributeChain,
            range(source_id, start, empty_start + "empty".len()),
            vec![attribute],
        );
        let type_head = builder.add_node(
            SurfaceNodeKind::TypeHead,
            range(source_id, set_start, set_start + "set".len()),
            vec![set],
        );
        builder.add_node(
            SurfaceNodeKind::TypeExpression,
            range(source_id, start, set_start + "set".len()),
            vec![attribute_chain, type_head],
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
