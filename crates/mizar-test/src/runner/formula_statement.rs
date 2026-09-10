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

const STEP5C8_CASES: [(&str, Stage, Option<&str>); 7] = [
    (
        "pass_formula_statement_connective_precedence_001",
        Stage::FormulaStatement,
        None,
    ),
    (
        "fail_type_elaboration_formula_unbound_free_variable_001",
        Stage::TypeElaboration,
        Some("formulas.free_variable_unbound"),
    ),
    (
        "fail_parse_only_iff_unparenthesized_chain_001",
        Stage::ParseOnly,
        Some("formulas.iff.unparenthesized_chain"),
    ),
    (
        "pass_formula_statement_iff_parenthesized_001",
        Stage::FormulaStatement,
        None,
    ),
    (
        "pass_formula_statement_is_type_assertion_001",
        Stage::FormulaStatement,
        None,
    ),
    (
        "pass_formula_statement_existential_multi_witness_001",
        Stage::FormulaStatement,
        None,
    ),
    (
        "pass_formula_statement_nested_quantifier_st_holds_001",
        Stage::FormulaStatement,
        None,
    ),
];

pub(super) fn is_step5c8_candidate(case: &TestCase) -> bool {
    STEP5C8_CASES.iter().any(|(id, _, _)| {
        case.id.0 == *id
            || case
                .source_path
                .file_name()
                .is_some_and(|name| name == format!("{id}.miz").as_str())
            || case
                .expectation_path
                .file_name()
                .is_some_and(|name| name == format!("{id}.expect.toml").as_str())
    })
}

pub(super) fn step5c8_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    let Some((id, stage, key)) = STEP5C8_CASES.iter().find(|(id, _, _)| case.id.0 == *id) else {
        return false;
    };
    let outcome = if key.is_some() {
        ExpectedOutcome::Fail
    } else {
        ExpectedOutcome::Pass
    };
    let directory = if key.is_some() { "fail" } else { "pass" };
    let source = format!("tests/miz/{directory}/formulas/{id}.miz");
    let sidecar = Path::new(&source).with_extension("expect.toml");
    let phase = match stage {
        Stage::ParseOnly => PipelinePhase::Parse,
        Stage::TypeElaboration => PipelinePhase::Resolve,
        _ => PipelinePhase::StatementCheck,
    };
    case.expectation.id == case.id
        && case.source_path.ends_with(&source)
        && case.expectation_path.ends_with(&sidecar)
        && case.expectation.source == Path::new(&source).file_name().unwrap()
        && case.expectation.stage == *stage
        && case.expectation.expected_phase == Some(phase)
        && case.expectation.expected_outcome == outcome
        && case.expectation.stable_detail_key.as_deref() == *key
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.snapshots.is_none()
        && case.expectation.tags == [format!("active_{}", stage.as_str())]
        && root.is_none_or(|root| {
            workspace_relative_source(root, &case.source_path).as_deref() == Some(source.as_str())
                && workspace_relative_source(root, &case.expectation_path)
                    .is_some_and(|actual| Path::new(&actual) == sidecar)
        })
}

const EXACT_FORMULA_STATEMENT_CASES: [(&str, &str, ExpectedOutcome); 7] = [
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
    (
        "pass_formula_statement_attr_negated_chain_assertion_001",
        "tests/miz/pass/attributes/pass_formula_statement_attr_negated_chain_assertion_001.miz",
        ExpectedOutcome::Pass,
    ),
];

pub(super) fn is_active_formula_statement(workspace_root: &Path, case: &TestCase) -> bool {
    if is_step5c8_candidate(case) {
        return case.expectation.stage == Stage::FormulaStatement
            && step5c8_admitted(Some(workspace_root), case);
    }
    exact_formula_statement_case(workspace_root, case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_FORMULA_STATEMENT_TAG]
        && case.expectation.stage == Stage::FormulaStatement
        && case.expectation.expected_phase == Some(PipelinePhase::StatementCheck)
        && case
            .source_path
            .extension()
            .is_some_and(|extension| extension == "miz")
}

pub(super) fn validate_step5c8_admission(
    workspace_root: &Path,
    plan: &TestPlan,
) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = Vec::new();
    for (id, _, key) in STEP5C8_CASES {
        let directory = if key.is_some() { "fail" } else { "pass" };
        let source = format!("tests/miz/{directory}/formulas/{id}.miz");
        if workspace_root.join(&source).is_file()
            && plan
                .cases
                .iter()
                .filter(|case| case.id.0 == id && step5c8_admitted(Some(workspace_root), case))
                .count()
                != 1
        {
            diagnostics.push(ValidationDiagnostic::error(
                Path::new(&source),
                "formula_statement",
                "E-FORMULAS-STEP5C8-INVENTORY",
                format!("formulas.step5c8_inventory.{id}"),
                "Step 5C.8 requires exactly one authenticated mapped row",
            ));
        }
    }
    for case in plan.cases.iter().filter(|case| is_step5c8_candidate(case)) {
        if !step5c8_admitted(Some(workspace_root), case) {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "formula_statement",
                "E-FORMULAS-STEP5C8-ADMISSION",
                format!("formulas.step5c8_admission.{}", case.id.0),
                "Step 5C.8 metadata or workspace path does not match its mapped row",
            ));
        }
    }
    diagnostics
}

pub(super) fn validate_active_formula_statement_tags(
    workspace_root: &Path,
    plan: &TestPlan,
) -> Vec<ValidationDiagnostic> {
    let mut diagnostics = validate_step5c8_admission(workspace_root, plan);
    for case in plan.cases.iter().filter(|case| {
        !is_step5c8_candidate(case)
            && (active_tag_count(case) > 0
                || EXACT_FORMULA_STATEMENT_CASES
                    .iter()
                    .any(|(id, _, _)| case.id.0 == *id))
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
    if is_step5c8_candidate(case) {
        if !step5c8_admitted(Some(workspace_root), case) {
            return vec!["formulas.invalid_admission".to_owned()];
        }
        let semantics = check_formula_ast(&ast, &resolver.module, &resolver.env)
            .err()
            .into_iter()
            .collect();
        return reconcile_frontend_and_semantics(frontend_keys, semantics);
    }
    if case.id.0 == "pass_formula_statement_attr_negated_chain_assertion_001"
        && is_active_formula_statement(workspace_root, case)
    {
        let keys = super::type_elaboration::source_attribute_semantics_detail_keys(
            &ast,
            resolver.module.clone(),
            &resolver.env,
        );
        return reconcile_frontend_and_semantics(frontend_keys, keys);
    }
    let semantics = source_variable_semantics_detail_keys(&ast, &resolver.module, &resolver.env);
    reconcile_frontend_and_semantics(frontend_keys, semantics)
}

fn check_formula_ast(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
) -> Result<(), String> {
    use mizar_resolve::labels::{LabelResolver, ProofLabelSourceCollector};
    let scope = SourceVariableScopeResolver::resolve_occurrences(SourceVariableScopeInput::new(
        ast, module, symbols,
    ))
    .map_err(|error| format!("formulas.scope:{error:?}"))?;
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let typed = super::type_elaboration::step5c8_formula_typed_ast(
        ast, module, symbols, &scope, &bindings,
    )?;
    let arena = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, module)
        .map_err(|error| error.to_string())?;
    let owner = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .ok_or("formulas.theorem_owner_missing")?;
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let labels = ProofLabelSourceCollector::new(
        ast,
        module,
        namespace.clone(),
        owner.contribution(),
        &arena,
    )
    .and_then(|collector| collector.collect_with_let_conditions())
    .map_err(|error| error.to_string())?;
    let resolved =
        LabelResolver::new(labels.projections()).resolve(module, &namespace, labels.references());
    SourceVariableSemanticsChecker::check_formula_statements(
        &typed, &scope, symbols, &labels, &resolved,
    )
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
pub(super) fn step5c8_test_frontend(source: &str) -> FrontendRun {
    use mizar_frontend::{
        orchestration::Frontend,
        parsing::MizarParserSeam,
        source::{FrontendSourceLoader, SourceUnitRequest},
    };
    use mizar_session::{
        DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId, SourceInput,
        SourceOriginInput,
    };
    let temporary = std::process::Command::new("mktemp")
        .arg("-d")
        .output()
        .unwrap();
    assert!(temporary.status.success());
    let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
    std::fs::create_dir(root.join("src")).unwrap();
    let path = root.join("src/formula_mutation.miz");
    std::fs::write(&path, source).unwrap();
    let output = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
        super::ParseOnlyImportProvider,
        MizarParserSeam,
    )
    .run(
        SourceUnitRequest {
            snapshot: super::shared::snapshot_id(5800),
            input: SourceInput {
                package_id: PackageId::new("formula-mutation"),
                module_path: ModulePath::new("formula_mutation"),
                normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                edition: Edition::new("2026"),
                origin: SourceOriginInput::Disk { path: path.clone() },
            },
        },
        &InMemorySessionIdAllocator::new(),
    )
    .unwrap();
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(root.join("src")).unwrap();
    std::fs::remove_dir(root).unwrap();
    FrontendRun {
        source_text: source.into(),
        ast: output.ast,
        ast_snapshot: None,
        diagnostics: output.diagnostics,
    }
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
    fn step5c8_admission_rejects_metadata_and_cross_stage_fallback() {
        let root = workspace_root();
        let plan = build_test_plan(&config()).unwrap();
        for (id, _, _) in super::STEP5C8_CASES {
            let original = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
            assert!(super::step5c8_admitted(Some(&root), original), "{id}");
            let mutations: &[fn(&mut crate::harness::TestCase)] = &[
                |case| case.id.0.push_str("_forged"),
                |case| case.expectation.id.0.push_str("_forged"),
                |case| case.source_path = case.source_path.with_file_name("wrong.miz"),
                |case| {
                    case.expectation_path =
                        case.expectation_path.with_file_name("wrong.expect.toml")
                },
                |case| case.expectation.source = "wrong.miz".into(),
                |case| case.expectation.stage = crate::staged_model::Stage::ProofVerification,
                |case| case.expectation.expected_phase = Some(PipelinePhase::TypeCheck),
                |case| case.expectation.expected_outcome = ExpectedOutcome::MetadataOnly,
                |case| case.expectation.stable_detail_key = Some("wrong".into()),
                |case| case.expectation.diagnostic_codes.push("E-FORGED".into()),
                |case| case.expectation.tags.clear(),
                |case| case.expectation.tags.push(case.expectation.tags[0].clone()),
            ];
            for mutate in mutations {
                let mut case = original.clone();
                mutate(&mut case);
                assert!(super::is_step5c8_candidate(&case));
                assert!(!super::step5c8_admitted(Some(&root), &case), "{case:?}");
                assert!(!is_active_formula_statement(&root, &case));
                assert!(!super::super::is_active_parse_only(&case));
                assert!(!super::super::is_active_type_elaboration(&case));
            }
            let mut wrong_root = original.clone();
            wrong_root.source_path = root
                .join("alias")
                .join(original.source_path.strip_prefix(&root).unwrap());
            assert!(!super::step5c8_admitted(Some(&root), &wrong_root));
            let mut missing = plan.clone();
            missing.cases.retain(|case| case.id.0 != id);
            assert!(
                validate_active_formula_statement_tags(&root, &missing)
                    .iter()
                    .any(|d| d.detail_key == format!("formulas.step5c8_inventory.{id}"))
            );
            let mut duplicate = plan.clone();
            duplicate.cases.push(original.clone());
            assert!(
                validate_active_formula_statement_tags(&root, &duplicate)
                    .iter()
                    .any(|d| d.detail_key == format!("formulas.step5c8_inventory.{id}"))
            );
        }
    }

    fn check_formula_source(source: &str) -> Result<(), String> {
        let root = workspace_root();
        let plan = build_test_plan(&config()).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::STEP5C8_CASES[0].0)
            .unwrap();
        let output = super::step5c8_test_frontend(source);
        let ast = output.ast.unwrap();
        let resolver = super::resolver_symbol_collection(&root, case, &ast);
        assert!(
            resolver.detail_keys.is_empty(),
            "{:?}",
            resolver.detail_keys
        );
        super::check_formula_ast(&ast, &resolver.module, &resolver.env)
    }

    #[test]
    fn step5c8_formula_structure_and_builtin_types_are_checked() {
        for formula in [
            "x = x or not x = x & x = x",
            "(x = x iff x = x)",
            "x is set",
            "x is set & x = x",
        ] {
            let source = format!(
                "theorem F: for x being set holds {formula} proof let x be set; thus {formula}; end;"
            );
            assert_eq!(check_formula_source(&source), Ok(()), "{source}");
        }
        for source in [
            "theorem F: for x, y being set holds x = x & x <> y proof let x, y be set; thus x = x & y <> x; end;",
            "theorem F: for x being set holds x = x iff x = x iff x = x proof let x be set; thus x = x iff x = x iff x = x; end;",
            "theorem F: for x being object holds x is set proof let x be object; thus x is set; end;",
            "theorem F: for x being object holds x in x proof let x be object; thus x in x; end;",
            "theorem F: for x being set holds x = x or not x = x & x = x proof let x be set; thus (x = x or not x = x) & x = x; end;",
            "theorem F: for x being set holds (x = x iff x = x) proof let x be set; thus x = x & x = x; end;",
            "theorem F: for x being object holds x = x proof let x be set; thus x = x; end;",
        ] {
            assert!(check_formula_source(source).is_err(), "{source}");
        }
    }

    #[test]
    fn step5c8_witnesses_keep_order_types_and_substitution() {
        let source = "theorem F: for a, b being object ex x, y being object st x = a & y = b proof let a, b be object; take a, b; thus a = a & b = b; end;";
        assert_eq!(check_formula_source(source), Ok(()));
        assert_eq!(
            check_formula_source(
                "theorem F: for a being set ex x being object st x = a proof let b be set; take b; thus b = b; end;"
            ),
            Ok(())
        );
        for replacement in ["take b, a;", "take a;", "take a, b, a;", "take k = a, b;"] {
            assert!(
                check_formula_source(&source.replace("take a, b;", replacement)).is_err(),
                "{replacement}"
            );
        }
        assert!(check_formula_source("theorem F: for a being object ex x being set st x = a proof let a be object; take a; thus a = a; end;").is_err());
    }

    #[test]
    fn step5c8_restrictions_and_real_citations_are_preserved() {
        let source = "theorem F: for X being set st X = X for y being object st y in X holds y in X proof let X be set such that A1: X = X; let y be object such that A2: y in X; thus y in X by A2; end;";
        assert_eq!(check_formula_source(source), Ok(()));
        assert!(
            check_formula_source(
                &source
                    .replace("st y in X", "st y = y")
                    .replace("A2: y in X", "A2: y = y")
            )
            .is_err()
        );
        for (from, to) in [
            ("A2: y in X", "A2: y = y"),
            (" by A2", " by Missing"),
            (" by A2", " by A1"),
            (" such that A2: y in X", ""),
        ] {
            assert!(
                check_formula_source(&source.replace(from, to)).is_err(),
                "{to}"
            );
        }
    }

    #[test]
    fn step5c8_free_variable_key_requires_resolver_failure() {
        let root = workspace_root();
        let plan = build_test_plan(&config()).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::STEP5C8_CASES[1].0)
            .unwrap();
        for (source, expected) in [
            (
                "theorem F: for x being set holds x = Missing;",
                vec!["formulas.free_variable_unbound".to_owned()],
            ),
            ("theorem F: for x being set holds x = x;", Vec::new()),
        ] {
            let keys = super::super::type_elaboration_detail_keys(
                &root,
                case,
                super::step5c8_test_frontend(source),
                &mut None,
            );
            assert_eq!(keys, expected);
        }
    }

    #[test]
    fn exact_inventory_has_seven_unique_id_source_pairs() {
        assert_eq!(EXACT_FORMULA_STATEMENT_CASES.len(), 7);
        assert_eq!(
            EXACT_FORMULA_STATEMENT_CASES
                .iter()
                .map(|(id, source, _)| (*id, *source))
                .collect::<BTreeSet<_>>()
                .len(),
            7
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
    fn corpus_executes_exact_seven_and_preserves_checker_keys() {
        let report = super::super::run_formula_statement_corpus(&config()).unwrap();
        assert_eq!(report.results.len(), 12);
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
