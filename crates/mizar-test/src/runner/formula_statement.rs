use std::path::Path;

use mizar_checker::type_checker::{SourceVariableSemanticsChecker, SourceVariableSemanticsInput};
use mizar_resolve::env::SymbolEnv;
use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
use mizar_resolve::resolved_ast::ModuleId;
use mizar_syntax::SurfaceAst;

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::{ExpectedOutcome, PipelinePhase};
use crate::harness::{TestCase, TestPlan};
use crate::staged_model::Stage;

use super::shared::{FrontendRun, frontend_detail_keys, resolver_symbol_collection, run_frontend};
use super::syntax_smoke::workspace_relative_source;
use super::{FormulaStatementCaseResult, FormulaStatementCaseStatus};

const ACTIVE_FORMULA_STATEMENT_TAG: &str = "active_formula_statement";
const DUPLICATE_BINDING_FRONTEND_KEY: &str = "frontend:lexing:ScopeSkeleton(DuplicateBindingName)";
const DUPLICATE_GENERALIZATION_KEY: &str = "variables.let.duplicate_generalization";

const EXACT_FORMULA_STATEMENT_CASES: [(&str, &str, ExpectedOutcome); 6] = [
    (
        "pass_formula_statement_deffunc_defpred_local_001",
        "tests/miz/pass/variables/pass_formula_statement_deffunc_defpred_local_001.miz",
        ExpectedOutcome::Pass,
    ),
    (
        "pass_formula_statement_let_such_that_assumption_001",
        "tests/miz/pass/variables/pass_formula_statement_let_such_that_assumption_001.miz",
        ExpectedOutcome::Pass,
    ),
    (
        "pass_formula_statement_set_local_constant_take_001",
        "tests/miz/pass/variables/pass_formula_statement_set_local_constant_take_001.miz",
        ExpectedOutcome::Pass,
    ),
    (
        "pass_formula_statement_reconsider_builtin_widening_001",
        "tests/miz/pass/variables/pass_formula_statement_reconsider_builtin_widening_001.miz",
        ExpectedOutcome::Pass,
    ),
    (
        "fail_formula_statement_duplicate_generalization_001",
        "tests/miz/fail/variables/fail_formula_statement_duplicate_generalization_001.miz",
        ExpectedOutcome::Fail,
    ),
    (
        "fail_formula_statement_take_non_existential_thesis_001",
        "tests/miz/fail/variables/fail_formula_statement_take_non_existential_thesis_001.miz",
        ExpectedOutcome::Fail,
    ),
];

pub(super) fn is_active_formula_statement(workspace_root: &Path, case: &TestCase) -> bool {
    exact_formula_statement_case(workspace_root, case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_FORMULA_STATEMENT_TAG]
        && case.expectation.stage == Stage::FormulaStatement
        && case.expectation.expected_phase == Some(PipelinePhase::StatementCheck)
        && case
            .source_path
            .extension()
            .is_some_and(|extension| extension == "miz")
}

pub(super) fn validate_active_formula_statement_tags(
    workspace_root: &Path,
    plan: &TestPlan,
) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    for case in plan.cases.iter().filter(|case| {
        active_tag_count(case) > 0
            || EXACT_FORMULA_STATEMENT_CASES
                .iter()
                .any(|(id, _, _)| case.id.0 == *id)
    }) {
        if !is_active_formula_statement(workspace_root, case) {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "formula_statement",
                "E-FORMULA-STATEMENT-ACTIVE-GATE",
                format!("formula_statement.active_gate.{}", case.id.0),
                "Step 5C.1 formula-statement admission requires one exact id/source row, one active_formula_statement tag, a .miz pass/fail expectation, stage formula_statement, and phase statement_check",
            ));
        }
        if !case.expectation.diagnostic_codes.is_empty() {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "formula_statement",
                "E-FORMULA-STATEMENT-PUBLIC-DIAGNOSTIC-CODES",
                format!("formula_statement.public_codes.{}", case.id.0),
                "active_formula_statement cases must keep diagnostic_codes empty; use stable_detail_key for the frozen internal detail key",
            ));
        }
    }
    for (id, source, _) in EXACT_FORMULA_STATEMENT_CASES {
        let count = plan
            .cases
            .iter()
            .filter(|case| {
                case.id.0 == id
                    && workspace_relative_source(workspace_root, &case.source_path)
                        .is_some_and(|actual| actual == source)
            })
            .count();
        if count != 1 {
            diagnostics.push(ValidationDiagnostic::error(
                Path::new(source),
                "formula_statement",
                "E-FORMULA-STATEMENT-EXACT-INVENTORY",
                format!("formula_statement.inventory.{id}"),
                format!(
                    "Step 5C.1 formula-statement row `{id}` must occur exactly once; found {count}"
                ),
            ));
        }
    }
    diagnostics
}

pub(super) fn run_formula_statement_case(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> FormulaStatementCaseResult {
    let actual_detail_keys = match run_frontend(workspace_root, case, ordinal) {
        Ok(output) => formula_statement_detail_keys(workspace_root, case, output),
        Err(error) => vec![format!("frontend_error:{error}")],
    };
    let expected_detail_keys = expected_detail_keys(case);
    let status = match case.expectation.expected_outcome {
        ExpectedOutcome::Pass if actual_detail_keys.is_empty() => {
            FormulaStatementCaseStatus::Passed
        }
        ExpectedOutcome::Fail if actual_detail_keys == expected_detail_keys => {
            FormulaStatementCaseStatus::Passed
        }
        _ => FormulaStatementCaseStatus::Failed,
    };
    FormulaStatementCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status,
        actual_detail_keys,
    }
}

pub(super) fn formula_statement_failure_diagnostic(
    case: &TestCase,
    result: &FormulaStatementCaseResult,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        &case.expectation_path,
        "formula_statement",
        "E-FORMULA-STATEMENT-ASSERT",
        format!("formula_statement.{}", case.id.0),
        format!(
            "formula-statement case `{}` expected detail keys {:?} but got {:?}",
            case.id.0,
            expected_detail_keys(case),
            result.actual_detail_keys
        ),
    )
}

fn formula_statement_detail_keys(
    workspace_root: &Path,
    case: &TestCase,
    output: FrontendRun,
) -> Vec<String> {
    let frontend_keys = frontend_detail_keys(case, &output.diagnostics);
    let Some(ast) = output.ast else {
        return if frontend_keys.is_empty() {
            vec!["formula_statement.lower_stage.no_ast".to_owned()]
        } else {
            lower_stage_keys(frontend_keys)
        };
    };
    let resolver = resolver_symbol_collection(workspace_root, case, &ast);
    if !resolver.detail_keys.is_empty() {
        return resolver
            .detail_keys
            .into_iter()
            .map(|key| format!("formula_statement.lower_stage.{key}"))
            .collect();
    }
    let semantics = source_variable_semantics_detail_keys(&ast, &resolver.module, &resolver.env);
    reconcile_frontend_and_semantics(frontend_keys, semantics)
}

fn lower_stage_keys(keys: Vec<String>) -> Vec<String> {
    keys.into_iter()
        .map(|key| format!("formula_statement.lower_stage.{key}"))
        .collect()
}

fn reconcile_frontend_and_semantics(
    frontend_keys: Vec<String>,
    semantics: Vec<String>,
) -> Vec<String> {
    if semantics.as_slice() == [DUPLICATE_GENERALIZATION_KEY] {
        if frontend_keys.as_slice() == [DUPLICATE_BINDING_FRONTEND_KEY] {
            return semantics;
        }
        if frontend_keys.is_empty() {
            return vec![
                "formula_statement.lower_stage.missing_duplicate_binding_diagnostic".to_owned(),
            ];
        }
    }
    if frontend_keys.is_empty() {
        semantics
    } else {
        lower_stage_keys(frontend_keys)
    }
}

pub(super) fn source_variable_semantics_detail_keys(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
) -> Vec<String> {
    let resolved = match SourceVariableScopeResolver::resolve(SourceVariableScopeInput::new(
        ast, module, symbols,
    )) {
        Ok(resolved) => resolved,
        Err(error) => {
            let detail_key = error.detail_key().unwrap_or(match error {
                mizar_resolve::names::SourceVariableScopeError::SourceMismatch => {
                    "variables.source.source_mismatch"
                }
                mizar_resolve::names::SourceVariableScopeError::ModuleMismatch => {
                    "variables.source.module_mismatch"
                }
                mizar_resolve::names::SourceVariableScopeError::RecoveredSyntax => {
                    "variables.source.recovered_syntax"
                }
                mizar_resolve::names::SourceVariableScopeError::UnresolvedReference => {
                    "variables.source.unresolved_reference"
                }
                mizar_resolve::names::SourceVariableScopeError::ArityMismatch => {
                    "variables.source.arity_mismatch"
                }
                _ => "variables.source.invalid_shape",
            });
            return vec![detail_key.to_owned()];
        }
    };
    SourceVariableSemanticsChecker::check(SourceVariableSemanticsInput::new(&resolved))
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.detail_key().to_owned())
        .collect()
}

fn exact_formula_statement_case(
    workspace_root: &Path,
    case: &TestCase,
) -> Option<(&'static str, &'static str, ExpectedOutcome)> {
    EXACT_FORMULA_STATEMENT_CASES
        .iter()
        .copied()
        .find(|(id, source, outcome)| {
            case.id.0 == *id
                && workspace_relative_source(workspace_root, &case.source_path)
                    .is_some_and(|actual| actual == *source)
                && case.expectation.expected_outcome == *outcome
        })
}

fn active_tag_count(case: &TestCase) -> usize {
    case.expectation
        .tags
        .iter()
        .filter(|tag| tag.as_str() == ACTIVE_FORMULA_STATEMENT_TAG)
        .count()
}

fn expected_detail_keys(case: &TestCase) -> Vec<String> {
    case.expectation.stable_detail_key.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use crate::expectation::{ExpectedOutcome, PipelinePhase};
    use crate::harness::{DiscoveryConfig, TestProfile, ValidationMode, build_test_plan};

    use super::{
        DUPLICATE_BINDING_FRONTEND_KEY, DUPLICATE_GENERALIZATION_KEY,
        EXACT_FORMULA_STATEMENT_CASES, is_active_formula_statement,
        reconcile_frontend_and_semantics, validate_active_formula_statement_tags,
    };

    #[test]
    fn exact_inventory_has_six_unique_id_source_pairs() {
        assert_eq!(EXACT_FORMULA_STATEMENT_CASES.len(), 6);
        assert_eq!(
            EXACT_FORMULA_STATEMENT_CASES
                .iter()
                .map(|(id, source, _)| (*id, *source))
                .collect::<BTreeSet<_>>()
                .len(),
            6
        );
    }

    #[test]
    fn exact_admission_rejects_tag_phase_outcome_and_source_drift() {
        let plan = build_test_plan(&config()).unwrap();
        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == EXACT_FORMULA_STATEMENT_CASES[0].0)
            .unwrap();
        let root = workspace_root();
        assert!(is_active_formula_statement(&root, original));

        let mut case = original.clone();
        case.expectation
            .tags
            .push("active_formula_statement".to_owned());
        assert!(!is_active_formula_statement(&root, &case));
        let mut case = original.clone();
        case.expectation
            .tags
            .push("allow_frontend_recovery_diagnostics".to_owned());
        assert!(!is_active_formula_statement(&root, &case));
        let mut case = original.clone();
        case.expectation.expected_phase = Some(PipelinePhase::TypeCheck);
        assert!(!is_active_formula_statement(&root, &case));
        let mut case = original.clone();
        case.expectation.expected_outcome = ExpectedOutcome::Fail;
        assert!(!is_active_formula_statement(&root, &case));
        let mut case = original.clone();
        case.source_path = workspace_root().join("tests/miz/pass/variables/not-the-case.miz");
        assert!(!is_active_formula_statement(&root, &case));
        let mut case = original.clone();
        case.source_path = root.join(
            "alias/tests/miz/pass/variables/pass_formula_statement_deffunc_defpred_local_001.miz",
        );
        assert!(!is_active_formula_statement(&root, &case));
        let mut case = original.clone();
        case.id.0 = "pass_formula_statement_unlisted_extra_001".to_owned();
        case.source_path =
            root.join("tests/miz/pass/variables/pass_formula_statement_unlisted_extra_001.miz");
        assert!(!is_active_formula_statement(&root, &case));
        let mut plan = plan;
        plan.cases.push(case);
        assert!(
            validate_active_formula_statement_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic
                    .detail_key
                    .ends_with("pass_formula_statement_unlisted_extra_001"))
        );
    }

    #[test]
    fn exact_inventory_rejects_missing_duplicate_and_public_codes() {
        let root = workspace_root();
        let mut plan = build_test_plan(&config()).unwrap();
        assert!(validate_active_formula_statement_tags(&root, &plan).is_empty());

        let duplicate = plan
            .cases
            .iter()
            .find(|case| case.id.0 == EXACT_FORMULA_STATEMENT_CASES[0].0)
            .unwrap()
            .clone();
        plan.cases.push(duplicate);
        assert!(
            validate_active_formula_statement_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.code.0 == "E-FORMULA-STATEMENT-EXACT-INVENTORY")
        );

        plan.cases
            .retain(|case| case.id.0 != EXACT_FORMULA_STATEMENT_CASES[1].0);
        assert!(
            validate_active_formula_statement_tags(&root, &plan)
                .iter()
                .any(|diagnostic| diagnostic
                    .detail_key
                    .ends_with(EXACT_FORMULA_STATEMENT_CASES[1].0))
        );

        let case = plan
            .cases
            .iter_mut()
            .find(|case| case.id.0 == EXACT_FORMULA_STATEMENT_CASES[2].0)
            .unwrap();
        case.expectation
            .diagnostic_codes
            .push("E-FORBIDDEN".to_owned());
        assert!(
            validate_active_formula_statement_tags(&root, &plan)
                .iter()
                .any(|diagnostic| {
                    diagnostic.code.0 == "E-FORMULA-STATEMENT-PUBLIC-DIAGNOSTIC-CODES"
                })
        );
    }

    #[test]
    fn corpus_executes_exact_six_and_preserves_checker_keys() {
        let report = super::super::run_formula_statement_corpus(&config()).unwrap();
        assert_eq!(report.results.len(), 6);
        assert_eq!(report.error_count(), 0, "{:?}", report.diagnostics);
        assert!(
            report.results.iter().all(|result| {
                result.status == super::super::FormulaStatementCaseStatus::Passed
            })
        );
        let actual = report
            .results
            .iter()
            .filter(|result| !result.actual_detail_keys.is_empty())
            .map(|result| (result.id.0.as_str(), result.actual_detail_keys.clone()))
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            vec![
                (
                    "fail_formula_statement_duplicate_generalization_001",
                    vec!["variables.let.duplicate_generalization".to_owned()],
                ),
                (
                    "fail_formula_statement_take_non_existential_thesis_001",
                    vec!["variables.take.non_existential_thesis".to_owned()],
                ),
            ]
        );
    }

    #[test]
    fn duplicate_handshake_rejects_missing_extra_and_different_frontend_evidence() {
        let semantic = || vec![DUPLICATE_GENERALIZATION_KEY.to_owned()];
        assert_eq!(
            reconcile_frontend_and_semantics(
                vec![DUPLICATE_BINDING_FRONTEND_KEY.to_owned()],
                semantic(),
            ),
            semantic()
        );
        assert_ne!(
            reconcile_frontend_and_semantics(Vec::new(), semantic()),
            semantic()
        );
        assert_ne!(
            reconcile_frontend_and_semantics(
                vec![
                    DUPLICATE_BINDING_FRONTEND_KEY.to_owned(),
                    "frontend:extra".to_owned(),
                ],
                semantic(),
            ),
            semantic()
        );
        assert_ne!(
            reconcile_frontend_and_semantics(vec!["frontend:different".to_owned()], semantic(),),
            semantic()
        );
    }

    fn config() -> DiscoveryConfig {
        DiscoveryConfig {
            workspace_root: workspace_root(),
            tests_root: PathBuf::from("tests"),
            manifest_path: PathBuf::from("tests/coverage/spec_trace.toml"),
            profile: TestProfile::Fast,
            validation_mode: ValidationMode::Metadata,
        }
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }
}
