use std::fs;
use std::path::Path;

use mizar_core::elaborator::{SourceStructureCoreNormalizer, SourceStructureCoreReceipt};
use mizar_vc::generator::{ExactTask180VcInput, generate_exact_task180_vc};
use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion, VcSet};

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::{ExpectedOutcome, PipelinePhase};
use crate::harness::{TestCase, TestPlan};
use crate::staged_model::Stage;

use super::import_fixtures::augment_type_elaboration_import_summaries;
use super::shared::{resolver_symbol_collection, run_frontend, snapshot_id};
use super::syntax_smoke::workspace_relative_source;
use super::type_elaboration::source_structure_semantics::source_structure_semantics_output;
use super::type_elaboration::{source_contradiction_core_ir, step5c4_mode_sethood_is_unprovable};
use super::{ProofVerificationCaseResult, ProofVerificationCaseStatus};

const ACTIVE_PROOF_VERIFICATION_TAG: &str = "active_proof_verification";
const EXACT_TASK180_CASE_ID: &str = "pass_proof_verification_contradiction_formula_constant_001";
pub(in crate::runner) const STEP5C2_PROOF_CASES: [(&str, &str); 2] = [
    (
        "pass_proof_verification_struct_constructor_access_001",
        "tests/miz/pass/structures/pass_proof_verification_struct_constructor_access_001.miz",
    ),
    (
        "pass_proof_verification_struct_with_update_001",
        "tests/miz/pass/structures/pass_proof_verification_struct_with_update_001.miz",
    ),
];
const STEP5C4_PROOF_CASE: (&str, &str) = (
    "fail_proof_verification_mode_sethood_unprovable_001",
    "tests/miz/fail/modes/fail_proof_verification_mode_sethood_unprovable_001.miz",
);
const STEP5C4_PROOF_DETAIL_KEY: &str = "modes.sethood.unprovable";
const STEP5C11_PROOF_CASES: [(&str, &str, &str, &str, &str); 2] = [
    (
        "fail_proof_verification_functorial_false_coherence_001",
        "tests/miz/fail/clusters/fail_proof_verification_functorial_false_coherence_001.miz",
        "clusters.functorial",
        "spec.en.17.clusters.functorial.false_coherence",
        "clusters.functorial.false_coherence",
    ),
    (
        "fail_proof_verification_reduce_false_reducibility_001",
        "tests/miz/fail/clusters/fail_proof_verification_reduce_false_reducibility_001.miz",
        "clusters.reduction",
        "spec.en.17.clusters.reduce.false_reducibility",
        "clusters.reduce.false_reducibility",
    ),
];

pub(super) fn is_step5c11_proof_candidate(case: &TestCase) -> bool {
    STEP5C11_PROOF_CASES.iter().any(|(id, source, ..)| {
        case.id.0 == *id
            || case.expectation.id.0 == *id
            || case.source_path.file_name() == Path::new(source).file_name()
            || case.expectation_path.file_name()
                == Path::new(source).with_extension("expect.toml").file_name()
    })
}

pub(super) fn step5c11_proof_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    let Some((index, (_, source, domain, spec_ref, detail))) = STEP5C11_PROOF_CASES
        .iter()
        .enumerate()
        .find(|(_, (id, ..))| case.id.0 == *id)
    else {
        return false;
    };
    let reduction = index == 1;
    let snapshot = format!("snapshots/vc/{}.vc_ir.snap", case.id.0);
    let refs = case.expectation.spec_refs.iter().map(|id| id.0.as_str());
    case.expectation.id == case.id
        && case.source_path.ends_with(source)
        && case
            .expectation_path
            .ends_with(Path::new(source).with_extension("expect.toml"))
        && root.is_none_or(|root| {
            workspace_relative_source(root, &case.source_path).as_deref() == Some(*source)
                && workspace_relative_source(root, &case.expectation_path).is_some_and(|path| {
                    Path::new(&path) == Path::new(source).with_extension("expect.toml")
                })
        })
        && case.expectation.source == Path::new(source).file_name().unwrap()
        && case.expectation.schema_version == 1
        && case.expectation.profiles.as_slice() == ["fast"]
        && case.expectation.kind == crate::expectation::TestKind::Fail
        && case.expectation.stage == Stage::ProofVerification
        && case.expectation.expected_phase == Some(PipelinePhase::Verification)
        && case.expectation.expected_outcome == ExpectedOutcome::Fail
        && case.expectation.domain == *domain
        && case.expectation.failure_category.as_deref() == Some("proof_failure")
        && case.expectation.stable_detail_key.as_deref() == Some(*detail)
        && if reduction {
            refs.eq([
                *spec_ref,
                "spec.en.mizar_vc.vc_ir.reduce_false_reducibility_snapshot",
            ]) && case.expectation.snapshots.as_deref() == Some(Path::new(&snapshot))
        } else {
            refs.eq([*spec_ref]) && case.expectation.snapshots.is_none()
        }
        && case.expectation.rejection_reason.is_none()
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.declaration_symbol_payloads.is_empty()
        && case.expectation.ast_profile.is_none()
        && case.expectation.snapshot_profiles.is_empty()
        && case.expectation.tokens.is_empty()
        && case.expectation.origin.is_none()
        && case.expectation.architecture22.is_none()
        && case.expectation.tags.as_slice() == [ACTIVE_PROOF_VERIFICATION_TAG]
}

const STEP5C5_PHRASE_ID: &str = "pass_proof_verification_pred_phrase_identifier_001";
const STEP5C5_PHRASE_SOURCE: &str =
    "tests/miz/pass/predicates/pass_proof_verification_pred_phrase_identifier_001.miz";

const STEP5C5_SYMBOLIC_ID: &str = "pass_proof_verification_pred_symbolic_infix_001";
const STEP5C5_MEANS_ID: &str = "pass_proof_verification_func_means_prefix_001";
const STEP5C5_EQUALS_ID: &str = "pass_proof_verification_func_equals_infix_operator_001";
const STEP5C5_PREDICATE_CASES: [(&str, &str, &str, &str, &str); 4] = [
    (
        STEP5C5_PHRASE_ID,
        STEP5C5_PHRASE_SOURCE,
        "predicates.phrase_definition",
        "spec.en.09.predicates.definition.phrase",
        "spec.en.mizar_vc.vc_ir.phrase_predicate_snapshot",
    ),
    (
        STEP5C5_SYMBOLIC_ID,
        "tests/miz/pass/predicates/pass_proof_verification_pred_symbolic_infix_001.miz",
        "predicates.symbolic_definition",
        "spec.en.09.predicates.definition.symbolic",
        "spec.en.mizar_vc.vc_ir.symbolic_predicate_snapshot",
    ),
    (
        STEP5C5_EQUALS_ID,
        "tests/miz/pass/functors/pass_proof_verification_func_equals_infix_operator_001.miz",
        "functors.equals_definition",
        "spec.en.10.functors.equals.definitional_unfolding",
        "spec.en.mizar_vc.vc_ir.equals_functor_snapshot",
    ),
    (
        STEP5C5_MEANS_ID,
        "tests/miz/pass/functors/pass_proof_verification_func_means_prefix_001.miz",
        "functors.means_definition",
        "spec.en.10.functors.means.definitional_unfolding",
        "spec.en.mizar_vc.vc_ir.means_functor_snapshot",
    ),
];

pub(super) fn is_step5c5_phrase_candidate(case: &TestCase) -> bool {
    STEP5C5_PREDICATE_CASES.iter().any(|(id, source, ..)| {
        case.id.0 == *id
            || case.expectation.id.0 == *id
            || case.source_path.file_name() == Path::new(source).file_name()
            || case.expectation_path.file_name()
                == Path::new(source).with_extension("expect.toml").file_name()
    })
}

fn step5c5_phrase_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    let Some((id, source, domain, semantic_ref, snapshot_ref)) = STEP5C5_PREDICATE_CASES
        .iter()
        .find(|(id, ..)| case.id.0 == *id)
    else {
        return false;
    };
    case.expectation.id == case.id
        && case.source_path.ends_with(source)
        && case
            .expectation_path
            .ends_with(Path::new(source).with_extension("expect.toml"))
        && root.is_none_or(|root| {
            workspace_relative_source(root, &case.source_path).as_deref() == Some(*source)
                && workspace_relative_source(root, &case.expectation_path).is_some_and(|path| {
                    Path::new(&path) == Path::new(source).with_extension("expect.toml")
                })
        })
        && case.expectation.source == Path::new(source).file_name().unwrap()
        && case.expectation.schema_version == 1
        && case.expectation.profiles.as_slice() == ["fast"]
        && case.expectation.kind == crate::expectation::TestKind::Pass
        && case.expectation.stage == Stage::ProofVerification
        && case.expectation.expected_phase == Some(PipelinePhase::VcGeneration)
        && case.expectation.expected_outcome == ExpectedOutcome::Pass
        && case.expectation.domain == *domain
        && case
            .expectation
            .spec_refs
            .iter()
            .map(|id| id.0.as_str())
            .eq([*semantic_ref, *snapshot_ref])
        && case.expectation.snapshots.as_deref()
            == Some(Path::new(&format!("snapshots/vc/{id}.vc_ir.snap")))
        && case.expectation.tags.as_slice() == [ACTIVE_PROOF_VERIFICATION_TAG]
        && case.expectation.failure_category.is_none()
        && case.expectation.stable_detail_key.is_none()
        && case.expectation.rejection_reason.is_none()
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.declaration_symbol_payloads.is_empty()
        && case.expectation.ast_profile.is_none()
        && case.expectation.snapshot_profiles.is_empty()
        && case.expectation.tokens.is_empty()
        && case.expectation.origin.is_none()
        && case.expectation.architecture22.is_none()
}

fn phrase_theorem_output(
    root: &Path,
    case: &TestCase,
    frontend: super::shared::FrontendRun,
) -> Result<(mizar_core::core_ir::CoreIr, VcSet), String> {
    use mizar_checker::type_checker::{SourceVariableSemanticsChecker, TermFormulaChecker};
    use mizar_resolve::labels::{LabelResolver, ProofLabelSourceCollector};
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    let ast = frontend.ast.clone().ok_or("phrase source AST missing")?;
    let (source, neutral, symbols) = super::source_registration_inputs(root, case, frontend)?;
    let (_, _, typed) =
        TermFormulaChecker::check_source_predicate_statements(&source, &symbols, &neutral)?;
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(&ast, source.module(), &symbols),
    )
    .map_err(|error| format!("{error:?}"))?;
    let owner = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .ok_or("phrase theorem owner missing")?;
    let namespace = mizar_resolve::env::NamespacePath::new(source.module().path().as_str());
    let labels = ProofLabelSourceCollector::new(
        &ast,
        source.module(),
        namespace.clone(),
        owner.contribution(),
        &source,
    )
    .and_then(|collector| collector.collect_with_theorem_owners(&symbols))
    .map_err(|error| error.to_string())?;
    let resolved = LabelResolver::new(labels.projections()).resolve(
        source.module(),
        &namespace,
        labels.references(),
    );
    let checked = SourceVariableSemanticsChecker::check_theorem_skeletons(
        &source, &typed, &scope, &symbols, &labels, &resolved, None,
    )?;
    let core = mizar_core::elaborator::lower_source_theorem_skeletons(&checked, None)?;
    let vcs = generate_core_vcs(&core, snapshot_id(0))?;
    Ok((core, vcs))
}

const STEP5C14_VC_CASES: [(&str, &str, &str, &str, &str, &str); 8] = [
    (
        "pass_proof_verification_computation_justification_001",
        "tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.miz",
        "algorithms.computation",
        "spec.en.20.algorithms.computation.justification",
        "spec.en.mizar_vc.vc_ir.computation_request_snapshot",
        "snapshots/vc/pass_proof_verification_computation_justification_001.vc_ir.snap",
    ),
    (
        "fail_proof_verification_algorithm_assert_unprovable_001",
        "tests/miz/fail/algorithms/fail_proof_verification_algorithm_assert_unprovable_001.miz",
        "algorithms.assertions",
        "spec.en.20.algorithms.state.var_const_assert",
        "spec.en.mizar_vc.vc_ir.algorithm_assert_failure_snapshot",
        "snapshots/vc/fail_proof_verification_algorithm_assert_unprovable_001.vc_ir.snap",
    ),
    (
        "fail_proof_verification_algorithm_ensures_unprovable_001",
        "tests/miz/fail/algorithms/fail_proof_verification_algorithm_ensures_unprovable_001.miz",
        "algorithms.contracts",
        "spec.en.20.algorithms.contracts.ensures",
        "spec.en.mizar_vc.vc_ir.algorithm_ensures_failure_snapshot",
        "snapshots/vc/fail_proof_verification_algorithm_ensures_unprovable_001.vc_ir.snap",
    ),
    (
        "pass_proof_verification_algorithm_ensures_return_001",
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz",
        "algorithms.contracts",
        "spec.en.20.algorithms.contracts.ensures",
        "spec.en.mizar_vc.vc_ir.algorithm_ensures_return_snapshot",
        "snapshots/vc/pass_proof_verification_algorithm_ensures_return_001.vc_ir.snap",
    ),
    (
        "pass_proof_verification_algorithm_ghost_snapshot_001",
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_ghost_snapshot_001.miz",
        "algorithms.ghost",
        "spec.en.20.algorithms.ghost.snapshot",
        "spec.en.mizar_vc.vc_ir.algorithm_ghost_snapshot",
        "snapshots/vc/pass_proof_verification_algorithm_ghost_snapshot_001.vc_ir.snap",
    ),
    (
        "pass_proof_verification_algorithm_while_invariant_001",
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_while_invariant_001.miz",
        "algorithms.loops",
        "spec.en.20.algorithms.loops.while_invariant",
        "spec.en.mizar_vc.vc_ir.algorithm_while_invariant",
        "snapshots/vc/pass_proof_verification_algorithm_while_invariant_001.vc_ir.snap",
    ),
    (
        "pass_proof_verification_algorithm_var_const_assert_001",
        "tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz",
        "algorithms.state",
        "spec.en.20.algorithms.state.var_const_assert",
        "spec.en.mizar_vc.vc_ir.algorithm_var_const_assert_snapshot",
        "snapshots/vc/pass_proof_verification_algorithm_var_const_assert_001.vc_ir.snap",
    ),
    (
        "pass_proof_verification_claim_block_theorem_001",
        "tests/miz/pass/algorithms/pass_proof_verification_claim_block_theorem_001.miz",
        "algorithms.claim",
        "spec.en.20.algorithms.claim.block",
        "spec.en.mizar_vc.vc_ir.algorithm_void_claim_snapshot",
        "snapshots/vc/pass_proof_verification_claim_block_theorem_001.vc_ir.snap",
    ),
];

pub(super) fn is_step5c14_return_candidate(case: &TestCase) -> bool {
    STEP5C14_VC_CASES.iter().any(|(id, source, ..)| {
        case.id.0 == *id
            || case.expectation.id.0 == *id
            || case.source_path.file_name() == Path::new(source).file_name()
            || case.expectation_path.file_name()
                == Path::new(source).with_extension("expect.toml").file_name()
    })
}

pub(super) fn step5c14_return_admitted(root: Option<&Path>, case: &TestCase) -> bool {
    let Some(&(_, source, domain, spec_ref, snapshot_ref, snapshot)) =
        STEP5C14_VC_CASES.iter().find(|(id, ..)| case.id.0 == *id)
    else {
        return false;
    };
    let failure = source.starts_with("tests/miz/fail/");
    case.expectation.id == case.id
        && case.source_path.ends_with(source)
        && case
            .expectation_path
            .ends_with(Path::new(source).with_extension("expect.toml"))
        && root.is_none_or(|root| {
            workspace_relative_source(root, &case.source_path).as_deref() == Some(source)
                && workspace_relative_source(root, &case.expectation_path).is_some_and(|path| {
                    Path::new(&path) == Path::new(source).with_extension("expect.toml")
                })
        })
        && case.expectation.source == Path::new(source).file_name().unwrap()
        && (!failure
            && !matches!(
                domain,
                "algorithms.claim"
                    | "algorithms.assertions"
                    | "algorithms.computation"
                    | "algorithms.ghost"
            )
            || case.expectation.schema_version == 1
                && case.expectation.profiles.as_slice() == ["fast"]
                && case.expectation.ast_profile.is_none()
                && case.expectation.snapshot_profiles.is_empty()
                && case.expectation.tokens.is_empty()
                && case.expectation.origin.is_none()
                && case.expectation.architecture22.is_none())
        && case.expectation.kind
            == if failure {
                crate::expectation::TestKind::Fail
            } else {
                crate::expectation::TestKind::Pass
            }
        && case.expectation.stage == Stage::ProofVerification
        && case.expectation.domain == domain
        && case.expectation.expected_phase
            == Some(if failure {
                PipelinePhase::Verification
            } else {
                PipelinePhase::VcGeneration
            })
        && case.expectation.expected_outcome
            == if failure {
                ExpectedOutcome::Fail
            } else {
                ExpectedOutcome::Pass
            }
        && case.expectation.failure_category.as_deref() == failure.then_some("proof_failure")
        && case.expectation.stable_detail_key.as_deref()
            == failure.then_some(if domain == "algorithms.assertions" {
                "algorithms.assert.unprovable"
            } else {
                "algorithms.ensures.unprovable"
            })
        && case.expectation.rejection_reason.is_none()
        && case.expectation.diagnostic_codes.is_empty()
        && case.expectation.diagnostic_payloads.is_empty()
        && case.expectation.declaration_symbol_payloads.is_empty()
        && case.expectation.snapshots.as_deref() == Some(Path::new(snapshot))
        && case.expectation.tags.as_slice() == [ACTIVE_PROOF_VERIFICATION_TAG]
        && case
            .expectation
            .spec_refs
            .iter()
            .map(|id| id.0.as_str())
            .eq([spec_ref, snapshot_ref])
}

const GENERATION_SCHEMA: &str = "mizar-vc-generation-task31-v1";
const VC_SCHEMA: &str = "mizar-vc-vcset-task31-v1";
const STEP5C7_PROOF_CASES: [(&str, &str); 2] = [
    (
        "pass_proof_verification_term_comprehension_guarded_001",
        "tests/miz/pass/terms/pass_proof_verification_term_comprehension_guarded_001.miz",
    ),
    (
        "pass_proof_verification_term_set_enumeration_membership_001",
        "tests/miz/pass/terms/pass_proof_verification_term_set_enumeration_membership_001.miz",
    ),
];

fn step5c7_proof_candidate(case: &TestCase) -> bool {
    STEP5C7_PROOF_CASES.iter().any(|(id, source)| {
        case.id.0 == *id
            || case.source_path.file_name() == Path::new(source).file_name()
            || case.expectation_path.file_name()
                == Path::new(source).with_extension("expect.toml").file_name()
    })
}

fn step5c7_proof_row(case: &TestCase) -> Option<&'static str> {
    STEP5C7_PROOF_CASES
        .iter()
        .find(|(id, source)| {
            case.id.0 == *id
                && case.expectation.id == case.id
                && case.source_path.ends_with(source)
                && case
                    .expectation_path
                    .ends_with(Path::new(source).with_extension("expect.toml"))
                && case.expectation.source == Path::new(source).file_name().unwrap()
                && case.expectation.stage == Stage::ProofVerification
                && case.expectation.expected_phase == Some(PipelinePhase::VcGeneration)
                && case.expectation.expected_outcome == ExpectedOutcome::Pass
                && case.expectation.stable_detail_key.is_none()
                && case.expectation.diagnostic_codes.is_empty()
                && case.expectation.diagnostic_payloads.is_empty()
                && case.expectation.snapshots.is_none()
                && case.expectation.tags.as_slice() == [ACTIVE_PROOF_VERIFICATION_TAG]
        })
        .map(|(_, source)| *source)
}

fn step5c7_proof_workspace_member(root: &Path, case: &TestCase) -> bool {
    step5c7_proof_row(case).is_some_and(|source| {
        workspace_relative_source(root, &case.source_path).is_some_and(|actual| actual == source)
            && workspace_relative_source(root, &case.expectation_path).is_some_and(|actual| {
                Path::new(&actual) == Path::new(source).with_extension("expect.toml")
            })
    })
}

fn normalize_term_ast(
    ast: &mizar_syntax::SurfaceAst,
    module: &mizar_resolve::resolved_ast::ModuleId,
    symbols: &mizar_resolve::env::SymbolEnv,
) -> Result<usize, String> {
    use mizar_checker::type_checker::SourceVariableSemanticsChecker;
    use mizar_resolve::labels::{LabelResolver, ProofLabelSourceCollector};
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    let scope = SourceVariableScopeResolver::resolve_occurrences(SourceVariableScopeInput::new(
        ast, module, symbols,
    ))
    .map_err(|error| format!("membership scope: {error:?}"))?;
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let typed = super::type_elaboration::step5c7_membership_typed_ast(
        ast, module, symbols, &scope, &bindings,
    )?;
    let arena = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, module)
        .map_err(|error| error.to_string())?;
    let owner = symbols
        .symbols()
        .iter()
        .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
        .ok_or("membership theorem owner missing")?;
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
    let checked = mizar_checker::source_set_term::SourceSetTermProducer::check_membership_proof(
        &typed, &scope, &bindings, &labels, &resolved, symbols,
    )?;
    mizar_core::elaborator::normalize_source_membership_proof(&checked)
}

pub(super) fn theorem_ast_output(
    ast: &mizar_syntax::SurfaceAst,
    module: &mizar_resolve::resolved_ast::ModuleId,
    symbols: &mizar_resolve::env::SymbolEnv,
    phase: PipelinePhase,
    snapshot: mizar_session::BuildSnapshotId,
    algorithm: Option<&mizar_checker::type_checker::SourceAlgorithmCheck<'_>>,
) -> Result<
    (
        Vec<mizar_proof::policy::CandidatePolicyClass>,
        Option<VcSet>,
    ),
    String,
> {
    use mizar_checker::{
        source_statement::SourceTheoremStatus, type_checker::SourceVariableSemanticsChecker,
    };
    use mizar_proof::policy::{
        CandidatePolicyClass, PolicyCandidate, ProofPolicyEvaluator, VerifierPolicy,
    };
    use mizar_resolve::labels::{LabelResolver, ProofLabelSourceCollector};
    use mizar_resolve::names::{SourceVariableScopeInput, SourceVariableScopeResolver};
    let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(ast, module, symbols),
    )
    .map_err(|error| format!("theorem scope: {error:?}"))?;
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let typed = super::type_elaboration::step5c8_formula_typed_ast(
        ast, module, symbols, &scope, &bindings, true,
    )?;
    let arena = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(ast, module)
        .map_err(|error| error.to_string())?;
    let owner = symbols
        .symbols()
        .iter()
        .find(|entry| {
            matches!(
                entry.kind(),
                mizar_resolve::env::SymbolKind::Theorem | mizar_resolve::env::SymbolKind::Lemma
            )
        })
        .ok_or("theorem owner missing")?;
    let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
    let labels = ProofLabelSourceCollector::new(
        ast,
        module,
        namespace.clone(),
        owner.contribution(),
        &arena,
    )
    .and_then(|collector| collector.collect_with_theorem_owners(symbols))
    .map_err(|error| error.to_string())?;
    let resolved =
        LabelResolver::new(labels.projections()).resolve(module, &namespace, labels.references());
    let checked = SourceVariableSemanticsChecker::check_theorem_skeletons(
        &arena, &typed, &scope, symbols, &labels, &resolved, algorithm,
    )?;
    let policy = ProofPolicyEvaluator::new(VerifierPolicy::development());
    let mut classes = Vec::new();
    for (owner, _) in checked.owners() {
        let (candidate, expected) = match owner.status {
            SourceTheoremStatus::Unmodified => continue,
            SourceTheoremStatus::Open => (
                PolicyCandidate::OpenObligation,
                CandidatePolicyClass::OpenAllowed,
            ),
            SourceTheoremStatus::Assumed => (
                PolicyCandidate::PolicyAssumption,
                CandidatePolicyClass::AssumedByPolicy,
            ),
            _ => return Err("unsupported theorem status".into()),
        };
        let decision = policy.evaluate_candidate(&candidate);
        if decision.class != expected
            || decision.can_schedule_kernel_check
            || decision.kernel_evidence_check_kind.is_some()
        {
            return Err("theorem status crossed proof-policy boundary".into());
        }
        classes.push(decision.class);
    }
    match phase {
        PipelinePhase::Resolve | PipelinePhase::StatementCheck => return Ok((classes, None)),
        PipelinePhase::VcGeneration if classes.is_empty() => {}
        _ => return Err("unsupported theorem phase/status".into()),
    }
    let core = mizar_core::elaborator::lower_source_theorem_skeletons(&checked, algorithm)?;
    let vcs = if algorithm.is_some() {
        mizar_vc::generator::generate_source_void_claim(
            &core,
            snapshot,
            &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-claim-v1"),
            &VcSchemaVersion::new("mizar-vc-vcset-step5c14-claim-v1"),
        )?
    } else if core.proof_nodes().iter().any(|(_, node)| {
        matches!(
            node.kind,
            mizar_core::core_ir::CoreProofNodeKind::ComputationGoal { .. }
        )
    }) {
        mizar_vc::generator::generate_source_computation_request(
            &core,
            snapshot,
            &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-computation-v1"),
            &VcSchemaVersion::new("mizar-vc-vcset-step5c14-computation-v1"),
        )?
    } else {
        generate_core_vcs(&core, snapshot)?
    };
    Ok((classes, Some(vcs)))
}

pub(super) fn generate_core_vcs(
    core: &mizar_core::core_ir::CoreIr,
    snapshot: mizar_session::BuildSnapshotId,
) -> Result<VcSet, String> {
    use mizar_vc::generator::{
        CoreGenerationCandidateSet, CoreGenerationInput, VcNormalizationInput,
    };
    use mizar_vc::vc_ir::{SeedIntakeTable, VcModuleRef};
    let flow = mizar_core::control_flow::build_control_flow_ir(core);
    let handoff = mizar_core::control_flow::build_obligation_seed_handoff(core, &flow);
    let intake = SeedIntakeTable::try_from_handoff(&handoff).map_err(|error| error.to_string())?;
    let module = core.module_id();
    let package = module.package().as_str();
    let path = module.path().as_str();
    let candidates = CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
        schema_version: &GenerationSchemaVersion::new(GENERATION_SCHEMA),
        module: &VcModuleRef::new(format!(
            "package={}:{};module={}:{}",
            package.len(),
            package,
            path.len(),
            path
        )),
        intake: &intake,
        handoff: &handoff,
        flow_output: Some(&flow),
    })
    .map_err(|error| error.to_string())?;
    let vcs = CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
        schema_version: &VcSchemaVersion::new(VC_SCHEMA),
        snapshot,
        source: core.source_id(),
        candidates: &candidates,
    })
    .map_err(|error| error.to_string())?;
    Ok(vcs)
}

pub(super) fn is_active_proof_verification(case: &TestCase) -> bool {
    if super::type_elaboration::is_step5c5_predicate_duplicate_candidate(case)
        || super::type_elaboration::is_step5c5_argument_candidate(case)
        || super::formula_statement::is_step5c5_negated_candidate(case)
        || super::type_elaboration::is_step5c3_argument_candidate(case)
        || super::type_elaboration::is_step5c4_dependent_candidate(case)
        || super::type_elaboration::is_step5c6_alias_candidate(case)
        || super::type_elaboration::is_step5c7_candidate(case)
    {
        return false;
    }
    if is_step5c5_phrase_candidate(case) {
        return step5c5_phrase_admitted(None, case);
    }
    if is_step5c14_return_candidate(case) {
        return step5c14_return_admitted(None, case);
    }
    if super::is_step5c14_static_candidate(case)
        || super::is_step5c13_overload_candidate(case)
        || super::is_step5c12_candidate(case)
    {
        return false;
    }
    if is_step5c11_proof_candidate(case) {
        return step5c11_proof_admitted(None, case);
    }
    if super::is_step5c11_registration_candidate(case) {
        return false;
    }
    if super::parse_only::is_step5c11_parse_candidate(case) {
        return false;
    }
    if super::formula_statement::is_step5c9_candidate(case)
        || super::formula_statement::is_step5c10_candidate(case)
    {
        return case.expectation.stage == Stage::ProofVerification
            && super::formula_statement::step5_formula_admitted(None, case);
    }
    let task180 = case.id.0 == EXACT_TASK180_CASE_ID
        && active_tag_count(case) == 1
        && case.expectation.stage == Stage::ProofVerification
        && case.expectation.expected_phase == Some(PipelinePhase::VcGeneration)
        && case.expectation.expected_outcome == ExpectedOutcome::Pass
        && case.expectation.snapshots.is_some();
    let step5c2 = step5c2_proof_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_PROOF_VERIFICATION_TAG]
        && case.expectation.snapshots.is_none();
    let step5c4 = step5c4_proof_case(case).is_some()
        && case.expectation.tags.as_slice() == [ACTIVE_PROOF_VERIFICATION_TAG]
        && case.expectation.snapshots.is_none();
    (task180 || step5c2 || step5c4 || step5c7_proof_row(case).is_some())
        && (!step5c7_proof_candidate(case) || step5c7_proof_row(case).is_some())
        && case
            .source_path
            .extension()
            .is_some_and(|extension| extension == "miz")
}

pub(super) fn validate_active_proof_verification_tags(
    workspace_root: &Path,
    plan: &TestPlan,
) -> Vec<ValidationDiagnostic> {
    let reserved_cases = plan
        .cases
        .iter()
        .filter(|case| {
            case.id.0 == EXACT_TASK180_CASE_ID
                || is_step5c2_proof_id(case)
                || case.id.0 == STEP5C4_PROOF_CASE.0
                || step5c7_proof_candidate(case)
                || is_step5c11_proof_candidate(case)
                || is_step5c14_return_candidate(case)
                || is_step5c5_phrase_candidate(case)
                || case
                    .expectation
                    .tags
                    .iter()
                    .any(|tag| tag == ACTIVE_PROOF_VERIFICATION_TAG)
        })
        .collect::<Vec<_>>();
    let mut diagnostics =
        super::formula_statement::validate_step5_formula_admission(workspace_root, plan);
    for case in reserved_cases {
        if is_step5c5_phrase_candidate(case) && !step5c5_phrase_admitted(Some(workspace_root), case)
            || is_step5c14_return_candidate(case)
                && !step5c14_return_admitted(Some(workspace_root), case)
            || is_step5c11_proof_candidate(case)
                && !step5c11_proof_admitted(Some(workspace_root), case)
            || !is_active_proof_verification(case)
            || is_step5c2_proof_id(case) && !is_step5c2_proof_workspace_member(workspace_root, case)
            || step5c4_proof_case(case).is_some()
                && !is_step5c4_proof_workspace_member(workspace_root, case)
            || step5c7_proof_candidate(case)
                && !step5c7_proof_workspace_member(workspace_root, case)
        {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "proof_verification",
                "E-PROOF-VERIFICATION-ACTIVE-GATE",
                format!("proof_verification.active_gate.{}", case.id.0),
                "proof verification admits only exact .miz pass expectations: Task-180 requires its VcIr snapshot and Step 5C.2 structure cases require no snapshot",
            ));
        }
        if !case.expectation.diagnostic_codes.is_empty() {
            diagnostics.push(ValidationDiagnostic::error(
                &case.expectation_path,
                "proof_verification",
                "E-PROOF-VERIFICATION-PUBLIC-DIAGNOSTIC-CODES",
                format!("proof_verification.public_codes.{}", case.id.0),
                "active_proof_verification cases must keep diagnostic_codes empty until public proof diagnostic codes are specified",
            ));
        }
    }
    if STEP5C2_PROOF_CASES
        .iter()
        .any(|(_, source)| workspace_root.join(source).is_file())
    {
        for (id, source) in STEP5C2_PROOF_CASES {
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
                    "proof_verification",
                    "E-PROOF-VERIFICATION-STEP5C2-INVENTORY",
                    format!("proof_verification.step5c2_inventory.{id}"),
                    format!("Step 5C.2 proof row `{id}` must occur exactly once; found {count}"),
                ));
            }
        }
    }
    if workspace_root.join(STEP5C4_PROOF_CASE.1).is_file() {
        let count = plan
            .cases
            .iter()
            .filter(|case| {
                case.id.0 == STEP5C4_PROOF_CASE.0
                    && workspace_relative_source(workspace_root, &case.source_path)
                        .is_some_and(|actual| actual == STEP5C4_PROOF_CASE.1)
            })
            .count();
        if count != 1 {
            diagnostics.push(ValidationDiagnostic::error(
                Path::new(STEP5C4_PROOF_CASE.1),
                "proof_verification",
                "E-PROOF-VERIFICATION-STEP5C4-INVENTORY",
                format!(
                    "proof_verification.step5c4_inventory.{}",
                    STEP5C4_PROOF_CASE.0
                ),
                format!(
                    "Step 5C.4 proof row `{}` must occur exactly once; found {count}",
                    STEP5C4_PROOF_CASE.0
                ),
            ));
        }
    }
    if STEP5C7_PROOF_CASES
        .iter()
        .any(|(_, source)| workspace_root.join(source).is_file())
    {
        for (id, source) in STEP5C7_PROOF_CASES {
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
                    "proof_verification",
                    "E-PROOF-VERIFICATION-STEP5C7-INVENTORY",
                    format!("proof_verification.step5c7_inventory.{id}"),
                    format!("Step 5C.7 proof row must occur once; found {count}"),
                ));
            }
        }
    }
    for (id, source, ..) in STEP5C11_PROOF_CASES {
        if workspace_root
            .join("tests/coverage/step5_activation_map.tsv")
            .is_file()
            || workspace_root.join(source).is_file()
        {
            let count = plan
                .cases
                .iter()
                .filter(|case| {
                    case.id.0 == id && step5c11_proof_admitted(Some(workspace_root), case)
                })
                .count();
            if count != 1 {
                diagnostics.push(ValidationDiagnostic::error(
                    Path::new(source),
                    "proof_verification",
                    "E-PROOF-VERIFICATION-STEP5C11-INVENTORY",
                    "proof_verification.step5c11_inventory",
                    format!(
                        "each Step 5C.11 correctness row must occur exactly once; found {count}"
                    ),
                ));
            }
        }
    }
    if workspace_root
        .join("tests/coverage/step5_activation_map.tsv")
        .is_file()
        || plan.cases.iter().any(is_step5c14_return_candidate)
    {
        for (id, source, ..) in STEP5C14_VC_CASES {
            let count = plan
                .cases
                .iter()
                .filter(|case| {
                    case.id.0 == id && step5c14_return_admitted(Some(workspace_root), case)
                })
                .count();
            if count != 1 {
                diagnostics.push(ValidationDiagnostic::error(
                    Path::new(source), "proof_verification",
                    "E-PROOF-VERIFICATION-STEP5C14-INVENTORY",
                    "proof_verification.step5c14_return_inventory",
                    format!("algorithm VC row {id} must occur exactly once with its authenticated snapshot; found {count}"),
                ));
            }
        }
    }
    if workspace_root
        .join("tests/coverage/step5_activation_map.tsv")
        .is_file()
        || plan.cases.iter().any(is_step5c5_phrase_candidate)
    {
        for (id, source, ..) in STEP5C5_PREDICATE_CASES {
            let count = plan
                .cases
                .iter()
                .filter(|case| {
                    case.id.0 == id && step5c5_phrase_admitted(Some(workspace_root), case)
                })
                .count();
            if count != 1 {
                diagnostics.push(ValidationDiagnostic::error(
                    Path::new(source),
                    "proof_verification",
                    "E-PROOF-VERIFICATION-STEP5C5-INVENTORY",
                    "proof_verification.step5c5_phrase_inventory",
                    format!("predicate VC row {id} must occur exactly once; found {count}"),
                ));
            }
        }
    }
    diagnostics
}

fn is_step5c2_proof_id(case: &TestCase) -> bool {
    STEP5C2_PROOF_CASES.iter().any(|(id, _)| case.id.0 == *id)
}

fn step5c2_proof_case(case: &TestCase) -> Option<(&'static str, &'static str)> {
    STEP5C2_PROOF_CASES.iter().copied().find(|(id, source)| {
        case.id.0 == *id
            && case.source_path.ends_with(source)
            && case.expectation.stage == Stage::ProofVerification
            && case.expectation.expected_phase == Some(PipelinePhase::VcGeneration)
            && case.expectation.expected_outcome == ExpectedOutcome::Pass
    })
}

fn is_step5c2_proof_workspace_member(workspace_root: &Path, case: &TestCase) -> bool {
    step5c2_proof_case(case).is_some_and(|(_, source)| {
        workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|actual| actual == source)
    })
}

fn step5c4_proof_case(case: &TestCase) -> Option<(&'static str, &'static str)> {
    (case.id.0 == STEP5C4_PROOF_CASE.0
        && case.source_path.ends_with(STEP5C4_PROOF_CASE.1)
        && case.expectation.stage == Stage::ProofVerification
        && case.expectation.expected_phase == Some(PipelinePhase::Verification)
        && case.expectation.expected_outcome == ExpectedOutcome::Fail
        && case.expectation.stable_detail_key.as_deref() == Some(STEP5C4_PROOF_DETAIL_KEY)
        && case.expectation.diagnostic_payloads.is_empty())
    .then_some(STEP5C4_PROOF_CASE)
}

fn is_step5c4_proof_workspace_member(workspace_root: &Path, case: &TestCase) -> bool {
    step5c4_proof_case(case).is_some_and(|(_, source)| {
        workspace_relative_source(workspace_root, &case.source_path)
            .is_some_and(|actual| actual == source)
    })
}

pub(super) fn run_proof_verification_case(
    workspace_root: &Path,
    tests_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> ProofVerificationCaseResult {
    if is_step5c5_phrase_candidate(case) {
        let check = || -> Result<(), String> {
            if !step5c5_phrase_admitted(Some(workspace_root), case) {
                return Err("invalid phrase predicate admission".into());
            }
            let build = || {
                phrase_theorem_output(workspace_root, case, run_frontend(workspace_root, case, 0)?)
            };
            let first = build()?;
            let second = build()?;
            if first != second {
                return Err("phrase source-to-Core/VC rerun differed".into());
            }
            let expected =
                fs::read_to_string(tests_root.join(case.expectation.snapshots.as_ref().unwrap()))
                    .map_err(|error| format!("phrase VC snapshot could not be read: {error}"))?;
            if first.1.debug_text() != expected {
                return Err("phrase VC snapshot differed".into());
            }
            Ok(())
        };
        let failure = check().err();
        return ProofVerificationCaseResult {
            id: case.id.clone(),
            expectation_path: case.expectation_path.clone(),
            status: if failure.is_none() {
                ProofVerificationCaseStatus::Passed
            } else {
                ProofVerificationCaseStatus::Failed
            },
            failure,
        };
    }
    if is_step5c14_return_candidate(case) {
        let check = || -> Result<(), String> {
            if !step5c14_return_admitted(Some(workspace_root), case) {
                return Err("invalid algorithm contract admission".into());
            }
            // Snapshot identity is independent of the active corpus ordering.
            let build = || -> Result<(Option<mizar_core::core_ir::CoreIr>, VcSet), String> {
                let frontend = run_frontend(workspace_root, case, 0)?;
                let computation = (case.expectation.domain == "algorithms.computation")
                    .then(|| frontend.ast.clone())
                    .flatten();
                let claim = (case.expectation.domain == "algorithms.claim")
                    .then(|| frontend.ast.clone())
                    .flatten();
                let (source, typed, symbols) =
                    super::source_registration_inputs(workspace_root, case, frontend)?;
                if let Some(ast) = computation {
                    return theorem_ast_output(
                        &ast,
                        source.module(),
                        &symbols,
                        PipelinePhase::VcGeneration,
                        snapshot_id(0),
                        None,
                    )?
                    .1
                    .map(|vcs| (None, vcs))
                    .ok_or_else(|| "computation VC missing".into());
                }
                let checked = mizar_checker::type_checker::check_source_algorithm_types(
                    &source, &typed, &symbols,
                )?;
                if let Some(ast) = claim {
                    return theorem_ast_output(
                        &ast,
                        source.module(),
                        &symbols,
                        PipelinePhase::VcGeneration,
                        snapshot_id(0),
                        Some(&checked),
                    )?
                    .1
                    .map(|vcs| (None, vcs))
                    .ok_or_else(|| "claim theorem VC missing".into());
                }
                let core = mizar_core::elaborator::lower_source_algorithms(&checked)?;
                if case.expectation.domain == "algorithms.ghost" {
                    use mizar_core::control_flow::{
                        ControlFlowStatementPlacement, build_control_flow_ir,
                    };
                    use mizar_core::core_ir::CoreAlgorithmStmtKind;
                    let flow_output = build_control_flow_ir(&core);
                    let (_, flow) = flow_output
                        .flows
                        .iter()
                        .next()
                        .ok_or("snapshot flow missing")?;
                    let captures = checked.snapshots().values().collect::<Vec<_>>();
                    let [(_, sealed)] = captures.as_slice() else {
                        return Err("snapshot seal missing".into());
                    };
                    let statements = core
                        .algorithm_statements()
                        .iter()
                        .filter_map(|(id, statement)| {
                            if let CoreAlgorithmStmtKind::Snapshot { name, captures } =
                                &statement.kind
                            {
                                Some((id, name, captures))
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let [(id, name, captured)] = statements.as_slice() else {
                        return Err("snapshot statement missing".into());
                    };
                    let Some(ControlFlowStatementPlacement::Snapshot {
                        context,
                        captures: locals,
                        ..
                    }) = flow.source_map.statement_placements.get(id)
                    else {
                        return Err("snapshot placement missing".into());
                    };
                    if sealed.len() != 2 || captured.len() != 2 || captured[0] == captured[1]
                        || checked.snapshots().values().next().map(|(name, _)| name) != Some(*name)
                        || !sealed.iter().zip(captured.iter()).all(|(binding, var)| {
                            checked.bindings().bindings().get(*binding).is_some_and(|binding|
                                flow.locals.iter().any(|(_, local)| local.binder.var == *var
                                    && local.binder.source.anchor == mizar_core::core_ir::CoreSourceAnchor::SourceRange(binding.declaration_range)))
                        })
                        || locals.len() != captured.len()
                        || !locals.iter().zip(captured.iter()).all(|(local, var)| flow.locals.get(*local).is_some_and(|local| local.binder.var == *var))
                        || flow.contexts.get(*context).is_none_or(|context| !locals.iter().all(|local| context.definitely_initialized.contains(local)))
                        || flow.ghost_effects.ghost_assignment_effects.len() != 1
                    { return Err("snapshot capture correspondence differed".into()); }
                }

                let profile = if case.expectation.domain == "algorithms.assertions" {
                    "assert-failure"
                } else if case.expectation.domain == "algorithms.ghost" {
                    "ghost-snapshot"
                } else if case.expectation.domain == "algorithms.state" {
                    "state"
                } else {
                    "return"
                };
                let vcs = mizar_vc::generator::generate_source_algorithm_postconditions(
                    &core,
                    snapshot_id(0),
                    &GenerationSchemaVersion::new(format!(
                        "mizar-vc-generation-step5c14-{profile}-v1"
                    )),
                    &VcSchemaVersion::new(format!("mizar-vc-vcset-step5c14-{profile}-v1")),
                )?;
                Ok((Some(core), vcs))
            };
            let (core, first) = build()?;
            let (replayed_core, second) = build()?;
            if core
                .as_ref()
                .map(mizar_core::control_flow::build_control_flow_ir)
                != replayed_core
                    .as_ref()
                    .map(mizar_core::control_flow::build_control_flow_ir)
            {
                return Err("algorithm CFG rerun was nondeterministic".into());
            }
            if core != replayed_core || first != second || first.debug_text() != second.debug_text()
            {
                return Err("algorithm source-to-VC rerun was nondeterministic".into());
            }
            let expected =
                fs::read_to_string(tests_root.join(case.expectation.snapshots.as_ref().unwrap()))
                    .map_err(|error| format!("algorithm VC snapshot could not be read: {error}"))?;
            if first.debug_text() != expected {
                return Err("algorithm VC snapshot differed".into());
            }
            if case.expectation.expected_outcome == ExpectedOutcome::Fail {
                let failed = mizar_vc::discharge::failed_source_algorithm_assertion(
                    core.as_ref().ok_or("algorithm Core missing")?,
                    &first,
                )?
                .ok_or("expected algorithm failure was not observed")?;
                let kind = if case.expectation.domain == "algorithms.assertions" {
                    mizar_vc::vc_ir::VcKind::AlgorithmAssertion
                } else {
                    mizar_vc::vc_ir::VcKind::AlgorithmPostcondition
                };
                if !first
                    .vcs()
                    .iter()
                    .any(|vc| vc.id == failed && vc.kind == kind)
                {
                    return Err("algorithm failure referred to a different obligation".into());
                }
            }
            Ok(())
        };
        let failure = check().err();
        return ProofVerificationCaseResult {
            id: case.id.clone(),
            expectation_path: case.expectation_path.clone(),
            status: if failure.is_none() {
                ProofVerificationCaseStatus::Passed
            } else {
                ProofVerificationCaseStatus::Failed
            },
            failure,
        };
    }
    if is_step5c11_proof_candidate(case) {
        let check = || -> Result<(), String> {
            if !step5c11_proof_admitted(Some(workspace_root), case) {
                return Err("invalid Step 5C.11 proof admission".into());
            }
            let reduction = case.id.0 == STEP5C11_PROOF_CASES[1].0;
            let ordinal = if reduction { 0 } else { ordinal };
            let build = || -> Result<(_, VcSet), String> {
                let (source, nodes, symbols) = super::source_registration_inputs(
                    workspace_root,
                    case,
                    run_frontend(workspace_root, case, ordinal)?,
                )?;
                let checked =
                    mizar_checker::registration_resolution::check_source_registration_intake(
                        &source, &nodes, &symbols,
                    )?;
                let core = mizar_core::elaborator::lower_source_functorial_registration(&checked)?;
                let vcs = generate_core_vcs(&core, snapshot_id(ordinal))?;
                Ok((core, vcs))
            };
            let (core, vcs) = build()?;
            if reduction {
                let replay = build()?;
                if core != replay.0 || vcs != replay.1 || vcs.debug_text() != replay.1.debug_text()
                {
                    return Err("reduction source-to-VC rerun was nondeterministic".into());
                }
                let expected = fs::read_to_string(
                    tests_root.join(case.expectation.snapshots.as_ref().unwrap()),
                )
                .map_err(|error| format!("reduction VC snapshot could not be read: {error}"))?;
                if vcs.debug_text() != expected {
                    return Err("reduction VC snapshot differed".into());
                }
            }
            if mizar_vc::discharge::failed_functorial_coherence(&core, &vcs)?.is_none() {
                return Err("registration did not produce the expected correctness failure".into());
            }
            Ok(())
        };
        let failure = check().err();
        return ProofVerificationCaseResult {
            id: case.id.clone(),
            expectation_path: case.expectation_path.clone(),
            status: if failure.is_none() {
                ProofVerificationCaseStatus::Passed
            } else {
                ProofVerificationCaseStatus::Failed
            },
            failure,
        };
    }
    if super::formula_statement::is_step5c10_candidate(case) {
        let check = || -> Result<(), String> {
            if !is_active_proof_verification(case)
                || !super::formula_statement::step5_formula_admitted(Some(workspace_root), case)
            {
                return Err("invalid Step 5C.10 proof admission".into());
            }
            let frontend = run_frontend(workspace_root, case, ordinal)?;
            if !frontend.diagnostics.is_empty() {
                return Err("theorem frontend diagnostics".into());
            }
            let ast = frontend.ast.ok_or("theorem source has no AST")?;
            let resolver = resolver_symbol_collection(workspace_root, case, &ast);
            if !resolver.detail_keys.is_empty() {
                return Err("theorem resolver diagnostics".into());
            }
            let phase = case
                .expectation
                .expected_phase
                .ok_or("theorem phase missing")?;
            let (classes, vcs) = theorem_ast_output(
                &ast,
                &resolver.module,
                &resolver.env,
                phase,
                snapshot_id(ordinal),
                None,
            )?;
            match (phase, vcs) {
                (PipelinePhase::VcGeneration, Some(vcs))
                    if !vcs.vcs().is_empty() && classes.is_empty() =>
                {
                    Ok(())
                }
                (PipelinePhase::StatementCheck, None) if !classes.is_empty() => Ok(()),
                _ => Err("theorem phase produced no matching output".into()),
            }
        };
        let failure = check().err();
        return ProofVerificationCaseResult {
            id: case.id.clone(),
            expectation_path: case.expectation_path.clone(),
            status: if failure.is_none() {
                ProofVerificationCaseStatus::Passed
            } else {
                ProofVerificationCaseStatus::Failed
            },
            failure,
        };
    }
    if super::formula_statement::is_step5c9_candidate(case) {
        let check = || -> Result<(), String> {
            if !is_active_proof_verification(case)
                || !super::formula_statement::step5_formula_admitted(Some(workspace_root), case)
            {
                return Err("invalid Step 5C.9 proof admission".to_owned());
            }
            let frontend = run_frontend(workspace_root, case, ordinal)?;
            if !frontend.diagnostics.is_empty() {
                return Err("case-completeness frontend diagnostics".to_owned());
            }
            let ast = frontend.ast.ok_or("case-completeness source has no AST")?;
            let resolver = resolver_symbol_collection(workspace_root, case, &ast);
            if !resolver.detail_keys.is_empty() {
                return Err("case-completeness resolver diagnostics".to_owned());
            }
            if super::formula_statement::check_formula_ast_with_organization(
                &ast,
                &resolver.module,
                &resolver.env,
                true,
            )? {
                return Err("expected an incomplete source-derived case split".to_owned());
            }
            Ok(())
        };
        let failure = check().err();
        return ProofVerificationCaseResult {
            id: case.id.clone(),
            expectation_path: case.expectation_path.clone(),
            status: if failure.is_none() {
                ProofVerificationCaseStatus::Passed
            } else {
                ProofVerificationCaseStatus::Failed
            },
            failure,
        };
    }
    if step5c7_proof_candidate(case) {
        let check = || -> Result<(), String> {
            if !step5c7_proof_workspace_member(workspace_root, case) {
                return Err("invalid Step 5C.7 proof admission".to_owned());
            }
            let frontend = run_frontend(workspace_root, case, ordinal)?;
            if !frontend.diagnostics.is_empty() {
                return Err("membership frontend diagnostics".to_owned());
            }
            let ast = frontend.ast.ok_or("membership source has no AST")?;
            let resolver = resolver_symbol_collection(workspace_root, case, &ast);
            if !resolver.detail_keys.is_empty() {
                return Err("membership resolver diagnostics".to_owned());
            }
            let first = normalize_term_ast(&ast, &resolver.module, &resolver.env)?;
            let second = normalize_term_ast(&ast, &resolver.module, &resolver.env)?;
            if first != 2 || second != first {
                return Err("membership obligations not discharged deterministically".to_owned());
            }
            Ok(())
        };
        let failure = check().err();
        return ProofVerificationCaseResult {
            id: case.id.clone(),
            expectation_path: case.expectation_path.clone(),
            status: if failure.is_none() {
                ProofVerificationCaseStatus::Passed
            } else {
                ProofVerificationCaseStatus::Failed
            },
            failure,
        };
    }
    if is_step5c4_proof_workspace_member(workspace_root, case) {
        let frontend = run_frontend(workspace_root, case, ordinal);
        let failure = match frontend {
            Ok(frontend) if frontend.diagnostics.is_empty() => {
                let Some(ast) = frontend.ast else {
                    return ProofVerificationCaseResult {
                        id: case.id.clone(),
                        expectation_path: case.expectation_path.clone(),
                        status: ProofVerificationCaseStatus::Failed,
                        failure: Some("Step 5C.4 proof source produced no AST".to_owned()),
                    };
                };
                let resolver = resolver_symbol_collection(workspace_root, case, &ast);
                if !resolver.detail_keys.is_empty() {
                    Some("Step 5C.4 proof source produced resolver diagnostics".to_owned())
                } else {
                    step5c4_mode_sethood_is_unprovable(
                        &ast,
                        &resolver.module,
                        &resolver.shells,
                        &resolver.env,
                    )
                    .err()
                }
            }
            Ok(_) => Some("Step 5C.4 proof source produced frontend diagnostics".to_owned()),
            Err(error) => Some(error),
        };
        return ProofVerificationCaseResult {
            id: case.id.clone(),
            expectation_path: case.expectation_path.clone(),
            status: if failure.is_none() {
                ProofVerificationCaseStatus::Passed
            } else {
                ProofVerificationCaseStatus::Failed
            },
            failure,
        };
    }
    if is_step5c2_proof_workspace_member(workspace_root, case) {
        let first = normalize_structure_case(workspace_root, case, ordinal);
        let second = normalize_structure_case(workspace_root, case, ordinal);
        let failure = match (first, second) {
            (Ok(first), Ok(second)) => {
                if first != second {
                    Some("Step 5C.2 structure normalization rerun was nondeterministic".to_owned())
                } else if !first.has_zero_residual_vcs() || first.residual_vc_count() != 0 {
                    Some("Step 5C.2 structure normalization retained residual VCs".to_owned())
                } else {
                    None
                }
            }
            (Err(error), _) | (_, Err(error)) => Some(error),
        };
        return ProofVerificationCaseResult {
            id: case.id.clone(),
            expectation_path: case.expectation_path.clone(),
            status: if failure.is_none() {
                ProofVerificationCaseStatus::Passed
            } else {
                ProofVerificationCaseStatus::Failed
            },
            failure,
        };
    }

    let first = generate_case_vc(workspace_root, case, ordinal);
    let second = generate_case_vc(workspace_root, case, ordinal);
    let failure = match (first, second) {
        (Ok(first), Ok(second)) => {
            if first != second || first.debug_text() != second.debug_text() {
                Some("exact Task-180 source-to-VC rerun was nondeterministic".to_owned())
            } else {
                compare_vc_snapshot(tests_root, case.expectation.snapshots.as_deref(), &first)
            }
        }
        (Err(error), _) | (_, Err(error)) => Some(error),
    };
    ProofVerificationCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status: if failure.is_none() {
            ProofVerificationCaseStatus::Passed
        } else {
            ProofVerificationCaseStatus::Failed
        },
        failure,
    }
}

pub(in crate::runner) fn normalize_structure_case(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> Result<SourceStructureCoreReceipt, String> {
    if !is_step5c2_proof_workspace_member(workspace_root, case) {
        return Err("case is not an exact Step 5C.2 proof workspace member".to_owned());
    }
    let frontend = run_frontend(workspace_root, case, ordinal)?;
    if !frontend.diagnostics.is_empty() {
        return Err("Step 5C.2 proof source produced frontend diagnostics".to_owned());
    }
    let ast = frontend
        .ast
        .ok_or_else(|| "Step 5C.2 proof source produced no AST".to_owned())?;
    let resolver = resolver_symbol_collection(workspace_root, case, &ast);
    if resolver
        .detail_keys
        .iter()
        .any(|key| key != "declaration_symbol.symbol.duplicate_declaration")
    {
        return Err("Step 5C.2 proof source produced resolver diagnostics".to_owned());
    }
    let output = source_structure_semantics_output(&ast, resolver.module, &resolver.env)?;
    SourceStructureCoreNormalizer::normalize(&output)
        .map_err(|error| format!("Step 5C.2 Core normalization failed: {error}"))
}

fn active_tag_count(case: &TestCase) -> usize {
    case.expectation
        .tags
        .iter()
        .filter(|tag| tag.as_str() == ACTIVE_PROOF_VERIFICATION_TAG)
        .count()
}

pub(in crate::runner) fn generate_case_vc(
    workspace_root: &Path,
    case: &TestCase,
    _ordinal: usize,
) -> Result<VcSet, String> {
    // Task-180 source and snapshot identities stay fixed when later proof
    // routes are admitted before it in corpus order.
    let ordinal = 0;
    let frontend = run_frontend(workspace_root, case, ordinal)?;
    if !frontend.diagnostics.is_empty() {
        return Err("exact Task-180 source produced frontend diagnostics".to_owned());
    }
    let ast = frontend
        .ast
        .ok_or_else(|| "exact Task-180 source produced no AST".to_owned())?;
    let resolver = resolver_symbol_collection(workspace_root, case, &ast);
    if !resolver.detail_keys.is_empty() {
        return Err("exact Task-180 source produced resolver diagnostics".to_owned());
    }
    let symbols = augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    let core = source_contradiction_core_ir(&ast, resolver.module, &symbols)?;
    generate_exact_task180_vc(ExactTask180VcInput {
        core: &core,
        snapshot: snapshot_id(ordinal),
        generation_schema_version: &GenerationSchemaVersion::new(GENERATION_SCHEMA),
        vc_schema_version: &VcSchemaVersion::new(VC_SCHEMA),
    })
    .map_err(|error| error.to_string())
}

pub(in crate::runner) fn compare_vc_snapshot(
    tests_root: &Path,
    snapshot_path: Option<&Path>,
    vc_set: &VcSet,
) -> Option<String> {
    let Some(snapshot_path) = snapshot_path else {
        return Some("exact Task-180 VcIr snapshot path is absent".to_owned());
    };
    let expected = match fs::read_to_string(tests_root.join(snapshot_path)) {
        Ok(expected) => expected,
        Err(error) => {
            return Some(format!(
                "could not read exact Task-180 VcIr snapshot `{}`: {error}",
                snapshot_path.display()
            ));
        }
    };
    let actual = vc_set.debug_text();
    if expected == actual {
        None
    } else {
        Some(format!(
            "exact Task-180 VcIr snapshot `{}` differed (expected {} bytes, got {} bytes)",
            snapshot_path.display(),
            expected.len(),
            actual.len()
        ))
    }
}

pub(super) fn proof_verification_failure_diagnostic(
    case: &TestCase,
    result: &ProofVerificationCaseResult,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        &case.expectation_path,
        "proof_verification",
        "E-PROOF-VERIFICATION-CASE",
        format!("proof_verification.{}", case.id.0),
        result
            .failure
            .clone()
            .unwrap_or_else(|| "proof-verification case failed".to_owned()),
    )
}

#[cfg(test)]
mod term_proof_tests {
    use super::*;

    fn config() -> crate::harness::DiscoveryConfig {
        crate::harness::DiscoveryConfig {
            workspace_root: Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap(),
            tests_root: "tests".into(),
            manifest_path: "tests/coverage/spec_trace.toml".into(),
            profile: crate::harness::TestProfile::Fast,
            validation_mode: crate::harness::ValidationMode::Development,
        }
    }

    fn parse(source: &str) -> mizar_syntax::SurfaceAst {
        use mizar_frontend::{
            orchestration::Frontend,
            parsing::MizarParserSeam,
            source::{FrontendSourceLoader, SourceUnitRequest},
        };
        use mizar_session::{
            DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
            SourceInput, SourceOriginInput,
        };
        let temporary = std::process::Command::new("mktemp")
            .arg("-d")
            .output()
            .unwrap();
        assert!(temporary.status.success());
        let root = std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
        fs::create_dir(root.join("src")).unwrap();
        let path = root.join("src/membership.miz");
        fs::write(&path, source).unwrap();
        let output = Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(&root)),
            super::super::ParseOnlyImportProvider,
            MizarParserSeam,
        )
        .run(
            SourceUnitRequest {
                snapshot: snapshot_id(777),
                input: SourceInput {
                    package_id: PackageId::new("membership-test"),
                    module_path: ModulePath::new("membership"),
                    normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path: path.clone() },
                },
            },
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();
        fs::remove_file(path).unwrap();
        fs::remove_dir(root.join("src")).unwrap();
        fs::remove_dir(root).unwrap();
        // Exercise the checker even when frontend diagnostics would already reject.
        output.ast.unwrap()
    }

    #[test]
    fn step5c5_equals_preserves_rhs_scopes_and_two_open_goals() {
        use mizar_core::core_ir::*;
        fn term(core: &CoreIr, id: CoreTermId) -> String {
            match &core.terms().get(id).unwrap().kind {
                CoreTermKind::Var(var) => format!("v{}", var.index()),
                CoreTermKind::Apply { functor, args } => {
                    assert_eq!(functor, &core.definitions().iter().next().unwrap().1.symbol);
                    format!(
                        "F({})",
                        args.iter()
                            .map(|id| term(core, *id))
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                }
                other => panic!("unexpected term {other:?}"),
            }
        }
        fn formula(core: &CoreIr, id: CoreFormulaId) -> String {
            match &core.formulas().get(id).unwrap().kind {
                CoreFormulaKind::Equals { left, right } => {
                    format!("{}={}", term(core, *left), term(core, *right))
                }
                CoreFormulaKind::TypePred { subject, ty } => {
                    format!("{}({})", ty.as_str(), term(core, *subject))
                }
                CoreFormulaKind::Forall { binders, body } => format!(
                    "forall[{}]({})",
                    binders
                        .iter()
                        .map(|b| format!(
                            "v{}:{}",
                            b.var.index(),
                            formula(core, b.ty_guard.unwrap())
                        ))
                        .collect::<Vec<_>>()
                        .join(","),
                    formula(core, *body)
                ),
                other => panic!("unexpected formula {other:?}"),
            }
        }
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_EQUALS_ID)
            .unwrap();
        let text = fs::read_to_string(&case.source_path).unwrap();
        let validate = |core: &CoreIr,
                        vcs: &VcSet,
                        source: &str,
                        rhs: usize,
                        swap_args: bool,
                        swap_equality: bool| {
            assert_eq!(
                (
                    core.definitions().len(),
                    core.proofs().len(),
                    core.proof_nodes().len(),
                    core.obligation_seeds().len()
                ),
                (1, 1, 3, 2)
            );
            let (definition_id, definition) = core.definitions().iter().next().unwrap();
            assert_eq!(definition.expansion, ExpansionPolicy::Opaque);
            assert_eq!(
                definition
                    .params
                    .iter()
                    .map(|b| b.var.index())
                    .collect::<Vec<_>>(),
                [4, 5]
            );
            assert_eq!(
                definition
                    .params
                    .iter()
                    .map(|b| formula(core, b.ty_guard.unwrap()))
                    .collect::<Vec<_>>(),
                ["set(v4)", "set(v5)"]
            );
            let DefinitionBody::Term(body) = definition.body else {
                panic!("actual RHS term");
            };
            assert_eq!(term(core, body), format!("v{rhs}"));
            assert_eq!(definition.correctness.len(), 1);
            let seed_id = definition.correctness[0];
            let seed = core.obligation_seeds().get(seed_id).unwrap();
            assert_eq!(seed.kind, ObligationSeedKind::DefinitionCorrectness);
            assert_eq!(seed.status, ObligationSeedStatus::Active);
            assert_eq!(seed.owner, definition.owner.anchor_item());
            assert!(seed.context.is_empty());
            assert_eq!(
                formula(core, seed.goal.unwrap()),
                format!("forall[v4:set(v4),v5:set(v5)](set(v{rhs}))")
            );
            assert!(
                seed.core_refs
                    .contains(&CoreNodeRef::Definition(definition_id))
            );
            assert!(seed.core_refs.contains(&CoreNodeRef::Term(body)));
            assert!(
                seed.core_refs
                    .contains(&CoreNodeRef::Formula(seed.goal.unwrap()))
            );
            let CoreFormulaKind::Forall {
                binders,
                body: consequent,
            } = &core.formulas().get(seed.goal.unwrap()).unwrap().kind
            else {
                unreachable!()
            };
            for id in binders
                .iter()
                .map(|binder| binder.ty_guard.unwrap())
                .chain(std::iter::once(*consequent))
            {
                assert!(seed.core_refs.contains(&CoreNodeRef::Formula(id)));
                let CoreFormulaKind::TypePred { subject, .. } =
                    core.formulas().get(id).unwrap().kind
                else {
                    unreachable!()
                };
                assert!(seed.core_refs.contains(&CoreNodeRef::Term(subject)));
            }
            let CoreSourceAnchor::SourceRange(result_span) =
                core.formulas().get(*consequent).unwrap().source.anchor
            else {
                panic!("declared result source");
            };
            assert_eq!(&source[result_span.start..result_span.end], "set");
            let CoreSourceAnchor::SourceRange(rhs_span) =
                core.terms().get(body).unwrap().source.anchor
            else {
                panic!("RHS source");
            };
            assert_eq!(
                &source[rhs_span.start..rhs_span.end],
                definition.params[rhs - 4].source_name.as_deref().unwrap()
            );
            let CoreSourceAnchor::SourceRange(span) = seed.source.anchor else {
                panic!("coherence source");
            };
            assert_eq!(&source[span.start..span.end], "coherence;");
            let (_, proof) = core.proofs().iter().next().unwrap();
            assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
            let goal = |first: usize, second: usize| {
                let call = if swap_args {
                    format!("F(v{second},v{first})")
                } else {
                    format!("F(v{first},v{second})")
                };
                if swap_equality {
                    format!("v{first}={call}")
                } else {
                    format!("{call}=v{first}")
                }
            };
            assert_eq!(
                formula(core, proof.proposition),
                format!("forall[v0:set(v0),v1:set(v1)]({})", goal(0, 1))
            );
            let mut current = proof.root;
            let mut guards = Vec::new();
            for expected in [2, 3] {
                let CoreProofNodeKind::IntroduceBinder { binder, child } =
                    &core.proof_nodes().get(current).unwrap().kind
                else {
                    panic!("local binder");
                };
                assert_eq!(binder.var.index(), expected);
                assert_eq!(
                    formula(core, binder.ty_guard.unwrap()),
                    format!("set(v{expected})")
                );
                guards.push(binder.ty_guard.unwrap());
                current = *child;
            }
            let CoreProofNodeKind::TerminalGoal {
                obligation,
                citations,
            } = &core.proof_nodes().get(current).unwrap().kind
            else {
                panic!("terminal");
            };
            assert!(citations.is_empty());
            let terminal = core.obligation_seeds().get(*obligation).unwrap();
            assert_eq!(terminal.kind, ObligationSeedKind::TheoremProof);
            assert_eq!(terminal.status, ObligationSeedStatus::Active);
            assert_eq!(terminal.owner, proof.item);
            assert_eq!(terminal.context, guards);
            assert_eq!(formula(core, terminal.goal.unwrap()), goal(2, 3));
            assert_ne!(terminal.local_path, seed.local_path);
            assert_eq!(
                core.terms()
                    .iter()
                    .filter_map(|(_, row)| if let CoreTermKind::Var(var) = row.kind {
                        Some(var.index())
                    } else {
                        None
                    })
                    .collect::<std::collections::BTreeSet<_>>(),
                (0..6).collect()
            );
            for (id, row) in core.terms().iter() {
                assert_eq!(core.source_map().term_sources.get(&id), Some(&row.source));
            }
            for (id, row) in core.formulas().iter() {
                assert_eq!(
                    core.source_map().formula_sources.get(&id),
                    Some(&row.source)
                );
            }
            assert_eq!(vcs.vcs().len(), 2);
            assert!(
                vcs.vcs()
                    .iter()
                    .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
            );
            for (id, seed) in core.obligation_seeds().iter() {
                assert_eq!(
                    core.source_map().obligation_sources.get(&id),
                    Some(&seed.source)
                );
                let vc = vcs
                    .vcs()
                    .iter()
                    .find(|vc| vc.goal == mizar_vc::vc_ir::VcFormulaRef::Core(seed.goal.unwrap()))
                    .unwrap();
                assert_eq!(
                    vc.kind,
                    if seed.kind == ObligationSeedKind::DefinitionCorrectness {
                        mizar_vc::vc_ir::VcKind::DefinitionCorrectness
                    } else {
                        mizar_vc::vc_ir::VcKind::TheoremProofStep
                    }
                );
            }
        };
        let sources = [
            (text.clone(), 4, false, false),
            (text.replace("equals X", "equals Y"), 5, false, false),
            (
                text.replace("FirstOfDef", "DifferentDefinition")
                    .replace("FirstOf1", "DifferentTheorem")
                    .replace("+", "*"),
                4,
                false,
                false,
            ),
            (
                text.replace("let X, Y be set;\n  func", "let A, B be set;\n  func")
                    .replace("X \\+\\ Y -> set equals X", "A \\+\\ B -> set equals A"),
                4,
                false,
                false,
            ),
            (
                text.replace(
                    "for X, Y being set holds X \\+\\ Y = X",
                    "for C, D being set holds C \\+\\ D = C",
                ),
                4,
                false,
                false,
            ),
            (
                text.replace(
                    "proof\n  let X, Y be set;\n  thus X \\+\\ Y = X",
                    "proof\n  let E, F be set;\n  thus E \\+\\ F = E",
                ),
                4,
                false,
                false,
            ),
            (
                text.replace("holds X \\+\\ Y = X", "holds Y \\+\\ X = X")
                    .replace("thus X \\+\\ Y = X", "thus Y \\+\\ X = X"),
                4,
                true,
                false,
            ),
            (
                text.replace("holds X \\+\\ Y = X", "holds X = X \\+\\ Y")
                    .replace("thus X \\+\\ Y = X", "thus X = X \\+\\ Y"),
                4,
                false,
                true,
            ),
        ];
        for (source, rhs, swap_args, swap_equality) in sources {
            let build = || {
                phrase_theorem_output(
                    &config.workspace_root,
                    case,
                    super::super::formula_statement::step5c8_test_frontend(&source),
                )
                .unwrap()
            };
            let (core, vcs) = build();
            assert_eq!((core.clone(), vcs.clone()), build());
            validate(&core, &vcs, &source, rhs, swap_args, swap_equality);
            if source == text {
                for mutation in 0..8 {
                    let mut parts = CoreIrParts {
                        source_id: core.source_id(),
                        module_id: core.module_id().clone(),
                        items: core.items().clone(),
                        terms: core.terms().clone(),
                        formulas: core.formulas().clone(),
                        definitions: core.definitions().clone(),
                        proofs: core.proofs().clone(),
                        proof_nodes: core.proof_nodes().clone(),
                        algorithms: core.algorithms().clone(),
                        algorithm_statements: core.algorithm_statements().clone(),
                        generated: core.generated().clone(),
                        obligation_seeds: core.obligation_seeds().clone(),
                        source_map: core.source_map().clone(),
                        diagnostics: core.diagnostics().clone(),
                    };
                    let (definition_id, definition) = core.definitions().iter().next().unwrap();
                    let seed = definition.correctness[0];
                    let goal = core.obligation_seeds().get(seed).unwrap().goal.unwrap();
                    let CoreFormulaKind::Forall {
                        body: consequent, ..
                    } = core.formulas().get(goal).unwrap().kind
                    else {
                        unreachable!()
                    };
                    let (call_id, _) = core
                        .terms()
                        .iter()
                        .find(|(_, row)| matches!(row.kind, CoreTermKind::Apply { .. }))
                        .unwrap();
                    match mutation {
                        0 => {
                            parts.definitions.get_mut(definition_id).unwrap().expansion =
                                ExpansionPolicy::Transparent
                        }
                        1 => parts
                            .definitions
                            .get_mut(definition_id)
                            .unwrap()
                            .correctness
                            .clear(),
                        2 => parts
                            .definitions
                            .get_mut(definition_id)
                            .unwrap()
                            .correctness
                            .push(seed),
                        3 => {
                            parts.definitions.get_mut(definition_id).unwrap().params[1].ty_guard =
                                None
                        }
                        4 => {
                            let CoreFormulaKind::TypePred { subject, .. } =
                                &mut parts.formulas.get_mut(consequent).unwrap().kind
                            else {
                                unreachable!()
                            };
                            *subject = call_id;
                        }
                        5 => {
                            parts.definitions.get_mut(definition_id).unwrap().params[1].var =
                                definition.params[0].var
                        }
                        6 => {
                            let CoreTermKind::Apply { args, .. } =
                                &mut parts.terms.get_mut(call_id).unwrap().kind
                            else {
                                unreachable!()
                            };
                            args.swap(0, 1);
                        }
                        7 => {
                            let (terminal, _) = core
                                .obligation_seeds()
                                .iter()
                                .find(|(_, row)| row.kind == ObligationSeedKind::TheoremProof)
                                .unwrap();
                            parts
                                .obligation_seeds
                                .get_mut(terminal)
                                .unwrap()
                                .context
                                .push(goal);
                        }
                        _ => unreachable!(),
                    }
                    // Detached Core validation is structural; source agreement is checked by the graph oracle.
                    let hostile = CoreIr::try_new(parts).unwrap_or_else(|error| {
                        panic!("structurally valid graph mutation {mutation}: {error:?}")
                    });
                    let changed_vcs =
                        generate_core_vcs(&hostile, snapshot_id(0)).unwrap_or_else(|error| {
                            panic!("generatable graph mutation {mutation}: {error}")
                        });
                    assert!(
                        std::panic::catch_unwind(|| validate(
                            &hostile,
                            &changed_vcs,
                            &source,
                            rhs,
                            swap_args,
                            swap_equality
                        ))
                        .is_err(),
                        "undetected graph mutation {mutation}"
                    );
                }
            }
            if source == text {
                let (_, actual) = phrase_theorem_output(
                    &config.workspace_root,
                    case,
                    run_frontend(&config.workspace_root, case, 0).unwrap(),
                )
                .unwrap();
                assert_eq!(fs::read_to_string(config.workspace_root.join("tests/snapshots/vc/pass_proof_verification_func_equals_infix_operator_001.vc_ir.snap")).unwrap(), actual.debug_text());
            }
        }
    }

    #[test]
    fn step5c5_equals_authenticates_infix_association_payload() {
        use mizar_checker::type_checker::TermFormulaChecker;
        use mizar_syntax::ast::{
            SurfaceAstBuilder, SurfaceNodeKind as K, SurfaceOperatorAssociativity,
        };
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_EQUALS_ID)
            .unwrap();
        let text = fs::read_to_string(&case.source_path).unwrap();
        let ast = super::super::formula_statement::step5c8_test_frontend(&text)
            .ast
            .unwrap();
        let targets = ast
            .nodes()
            .iter()
            .enumerate()
            .filter_map(|(index, node)| matches!(node.kind, K::InfixExpression(_)).then_some(index))
            .collect::<Vec<_>>();
        assert_eq!(targets.len(), 2);
        for target in targets {
            for association in [
                SurfaceOperatorAssociativity::Right,
                SurfaceOperatorAssociativity::NonAssociative,
            ] {
                let mut builder = SurfaceAstBuilder::new(ast.source_id);
                let mut rebuilt = Vec::new();
                for (index, node) in ast.nodes().iter().enumerate() {
                    let children = node.children.iter().map(|id| rebuilt[id.index()]).collect();
                    let mut kind = node.kind.clone();
                    if index == target {
                        let K::InfixExpression(operator) = &mut kind else {
                            unreachable!()
                        };
                        assert_eq!(operator.associativity, SurfaceOperatorAssociativity::Left);
                        operator.associativity = association;
                    }
                    rebuilt.push(match kind {
                        K::Token(token) => builder.add_token(token.kind, token.text, node.range),
                        kind => builder.add_node(kind, node.range, children),
                    });
                }
                let changed = builder.finish(Some(rebuilt[ast.root().unwrap().index()]), None);
                let mut frontend = super::super::formula_statement::step5c8_test_frontend(&text);
                assert!(frontend.diagnostics.is_empty());
                frontend.ast = Some(changed);
                // Both source and neutral receipts share the hostile payload, so equality cannot reject it.
                let (source, nodes, symbols) = super::super::source_registration_inputs(
                    &config.workspace_root,
                    case,
                    frontend,
                )
                .unwrap();
                mizar_resolve::symbols::validate_source_symbol_env(&source, &symbols).unwrap();
                assert_eq!(
                    TermFormulaChecker::check_source_predicate_statements(
                        &source, &symbols, &nodes
                    )
                    .unwrap_err(),
                    "predicates.statement.unsupported_source_types"
                );
            }
        }
    }

    #[test]
    fn step5c5_equals_requires_exact_source_types_clause_and_operator() {
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_EQUALS_ID)
            .unwrap();
        let text = fs::read_to_string(&case.source_path).unwrap();
        for (index, changed) in [
            text.replace("X \\+\\ Y ->", "Y \\+\\ X ->"),
            text.replace("equals X", "equals Unknown"),
            text.replace("equals X", "equals it"),
            text.replace("-> set", "-> object"),
            text.replace("let X, Y be set;\n  func", "let X, Y be object;\n  func"),
            text.replace("for X, Y being set", "for X, Y being object"),
            text.replace("proof\n  let X, Y be set", "proof\n  let X, Y be object"),
            text.replace("holds X \\+\\ Y", "holds Unknown \\+\\ Y"),
            text.replace("thus X \\+\\ Y", "thus X \\+\\ Unknown"),
            text.replace("let X, Y be set;\n  func", "let X, Y, Z be set;\n  func"),
            text.replace("for X, Y being set", "for X, Y, Z being set"),
            text.replace("  coherence;\n", ""),
            text.replace("  coherence;", "  coherence; coherence;"),
            text.replace("  coherence;", "  existence;"),
            text.replace("  coherence;", "  coherence proof thus X = X; end;"),
            text.replace("  coherence;", "  symmetry; coherence;"),
            text.replace("func FirstOfDef", "redefine func FirstOfDef"),
            text.replace(r#""\\+\\""#, r#""\\*\\""#),
            text.replace("left, 80", "right, 80"),
            text.replace("left, 80", "left, 90"),
            text.replace("left, 80", "left, \"80\""),
            text.replace("infix_operator", "prefix_operator"),
            text.lines()
                .filter(|line| !line.starts_with("infix_operator"))
                .collect::<Vec<_>>()
                .join("\n"),
            text.replace(
                "\ntheorem",
                "\ninfix_operator(\"\\\\+\\\\\", left, 80);\ntheorem",
            ),
            text.replace("holds X \\+\\ Y = X", "holds X \\+\\ Y = X & X = X"),
            text.replace("thus X \\+\\ Y = X;", "thus X \\+\\ Y = X by FirstOfDef;"),
        ]
        .into_iter()
        .enumerate()
        {
            assert_ne!(changed, text, "unchanged control {index}");
            assert!(
                phrase_theorem_output(
                    &config.workspace_root,
                    case,
                    super::super::formula_statement::step5c8_test_frontend(&changed)
                )
                .is_err(),
                "accepted source mutation {index}"
            );
        }
    }

    #[test]
    fn step5c5_means_preserves_opaque_body_and_three_open_goals() {
        use mizar_core::core_ir::*;
        fn term(core: &CoreIr, id: CoreTermId) -> String {
            match &core.terms().get(id).unwrap().kind {
                CoreTermKind::Var(var) => format!("v{}", var.index()),
                CoreTermKind::Apply { functor, args } => {
                    assert_eq!(functor, &core.definitions().iter().next().unwrap().1.symbol);
                    format!(
                        "F({})",
                        args.iter()
                            .map(|id| term(core, *id))
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                }
                other => panic!("unexpected term {other:?}"),
            }
        }
        fn formula(core: &CoreIr, id: CoreFormulaId) -> String {
            match &core.formulas().get(id).unwrap().kind {
                CoreFormulaKind::Equals { left, right } => {
                    format!("{}={}", term(core, *left), term(core, *right))
                }
                CoreFormulaKind::TypePred { subject, ty } => {
                    format!("{}({})", ty.as_str(), term(core, *subject))
                }
                CoreFormulaKind::And(items) => format!(
                    "and({})",
                    items
                        .iter()
                        .map(|id| formula(core, *id))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                CoreFormulaKind::Implies {
                    premise,
                    conclusion,
                } => format!(
                    "implies({},{})",
                    formula(core, *premise),
                    formula(core, *conclusion)
                ),
                CoreFormulaKind::Forall { binders, body }
                | CoreFormulaKind::Exists { binders, body } => {
                    let quantifier = if matches!(
                        core.formulas().get(id).unwrap().kind,
                        CoreFormulaKind::Forall { .. }
                    ) {
                        "forall"
                    } else {
                        "exists"
                    };
                    format!(
                        "{quantifier}[{}]({})",
                        binders
                            .iter()
                            .map(|binder| format!(
                                "v{}:{}",
                                binder.var.index(),
                                formula(core, binder.ty_guard.unwrap())
                            ))
                            .collect::<Vec<_>>()
                            .join(","),
                        formula(core, *body)
                    )
                }
                other => panic!("unexpected formula {other:?}"),
            }
        }
        fn backrefs(core: &CoreIr, seed: &ObligationSeed, root: CoreFormulaId) {
            let mut pending = vec![CoreNodeRef::Formula(root)];
            let mut visited = std::collections::BTreeSet::new();
            while let Some(reference) = pending.pop() {
                if !visited.insert(reference.clone()) {
                    continue;
                }
                assert!(seed.core_refs.contains(&reference), "missing {reference:?}");
                match reference {
                    CoreNodeRef::Term(id) => {
                        if let CoreTermKind::Apply { args, .. } =
                            &core.terms().get(id).unwrap().kind
                        {
                            pending.extend(args.iter().copied().map(CoreNodeRef::Term));
                        }
                    }
                    CoreNodeRef::Formula(id) => match &core.formulas().get(id).unwrap().kind {
                        CoreFormulaKind::Equals { left, right } => {
                            pending.extend([CoreNodeRef::Term(*left), CoreNodeRef::Term(*right)])
                        }
                        CoreFormulaKind::TypePred { subject, .. } => {
                            pending.push(CoreNodeRef::Term(*subject))
                        }
                        CoreFormulaKind::And(items) => {
                            pending.extend(items.iter().copied().map(CoreNodeRef::Formula))
                        }
                        CoreFormulaKind::Implies {
                            premise,
                            conclusion,
                        } => pending.extend([
                            CoreNodeRef::Formula(*premise),
                            CoreNodeRef::Formula(*conclusion),
                        ]),
                        CoreFormulaKind::Forall { binders, body }
                        | CoreFormulaKind::Exists { binders, body } => {
                            pending.push(CoreNodeRef::Formula(*body));
                            pending.extend(
                                binders
                                    .iter()
                                    .filter_map(|binder| binder.ty_guard)
                                    .map(CoreNodeRef::Formula),
                            );
                        }
                        other => panic!("unexpected {other:?}"),
                    },
                    _ => unreachable!(),
                }
            }
        }
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_MEANS_ID)
            .unwrap();
        let text = fs::read_to_string(&case.source_path).unwrap();
        let validate = |core: &CoreIr,
                        vcs: &VcSet,
                        source: &str,
                        swapped_body: bool,
                        swapped_goal: bool| {
            assert_eq!(core.definitions().len(), 1);
            assert_eq!(core.proofs().len(), 1);
            assert_eq!(core.proof_nodes().len(), 2);
            assert_eq!(core.obligation_seeds().len(), 3);
            let (definition_id, definition) = core.definitions().iter().next().unwrap();
            assert_eq!(definition.expansion, ExpansionPolicy::Opaque);
            assert_eq!(definition.params.len(), 1);
            assert_eq!(definition.correctness.len(), 2);
            let DefinitionBody::Formula(body) = definition.body else {
                panic!("formula body")
            };
            assert_eq!(
                formula(core, body),
                if swapped_body { "v2=F(v2)" } else { "F(v2)=v2" }
            );
            assert_eq!(
                formula(core, definition.params[0].ty_guard.unwrap()),
                "set(v2)"
            );
            let relation = |result: usize| {
                if swapped_body {
                    format!("v2=v{result}")
                } else {
                    format!("v{result}=v2")
                }
            };
            let expected = [
                format!("forall[v2:set(v2)](exists[v3:set(v3)]({}))", relation(3)),
                format!(
                    "forall[v2:set(v2)](forall[v4:set(v4),v5:set(v5)](implies(and({},{}),v4=v5)))",
                    relation(4),
                    relation(5)
                ),
            ];
            for (index, seed_id) in definition.correctness.iter().enumerate() {
                let seed = core.obligation_seeds().get(*seed_id).unwrap();
                assert_eq!(seed.kind, ObligationSeedKind::DefinitionCorrectness);
                assert_eq!(seed.status, ObligationSeedStatus::Active);
                assert_eq!(seed.owner, definition.owner.anchor_item());
                assert!(seed.context.is_empty());
                assert_eq!(formula(core, seed.goal.unwrap()), expected[index]);
                assert!(
                    seed.core_refs
                        .contains(&CoreNodeRef::Definition(definition_id))
                );
                assert!(seed.core_refs.contains(&CoreNodeRef::Formula(body)));
                assert!(
                    seed.core_refs
                        .contains(&CoreNodeRef::Formula(seed.goal.unwrap()))
                );
                backrefs(core, seed, body);
                backrefs(core, seed, seed.goal.unwrap());
                assert!(!seed.source.provenance.is_empty());
                let CoreSourceAnchor::SourceRange(span) = seed.source.anchor else {
                    panic!("source clause")
                };
                assert_eq!(
                    &source[span.start..span.end],
                    ["existence;", "uniqueness;"][index]
                );
                assert_eq!(
                    core.source_map().obligation_sources.get(seed_id),
                    Some(&seed.source)
                );
            }
            assert_ne!(
                core.obligation_seeds()
                    .get(definition.correctness[0])
                    .unwrap()
                    .local_path,
                core.obligation_seeds()
                    .get(definition.correctness[1])
                    .unwrap()
                    .local_path
            );
            let (_, proof) = core.proofs().iter().next().unwrap();
            assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
            assert_eq!(
                formula(core, proof.proposition),
                if swapped_goal {
                    "forall[v0:set(v0)](v0=F(v0))"
                } else {
                    "forall[v0:set(v0)](F(v0)=v0)"
                }
            );
            let CoreProofNodeKind::IntroduceBinder { binder, child } =
                &core.proof_nodes().get(proof.root).unwrap().kind
            else {
                panic!("local binder")
            };
            let CoreProofNodeKind::TerminalGoal {
                obligation,
                citations,
            } = &core.proof_nodes().get(*child).unwrap().kind
            else {
                panic!("terminal")
            };
            assert!(citations.is_empty());
            let seed = core.obligation_seeds().get(*obligation).unwrap();
            assert_eq!(seed.kind, ObligationSeedKind::TheoremProof);
            assert_eq!(seed.status, ObligationSeedStatus::Active);
            assert_eq!(seed.owner, proof.item);
            assert_eq!(seed.context, [binder.ty_guard.unwrap()]);
            assert_eq!(formula(core, seed.context[0]), "set(v1)");
            assert_eq!(
                formula(core, seed.goal.unwrap()),
                if swapped_goal { "v1=F(v1)" } else { "F(v1)=v1" }
            );
            let variables = core
                .terms()
                .iter()
                .filter_map(|(_, row)| {
                    if let CoreTermKind::Var(var) = row.kind {
                        Some(var)
                    } else {
                        None
                    }
                })
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(variables, (0..6).map(CoreVarId::new).collect());
            for (id, row) in core.terms().iter() {
                assert_eq!(core.source_map().term_sources.get(&id), Some(&row.source));
                if matches!(row.kind, CoreTermKind::Var(var) if var.index() >= 3) {
                    let CoreSourceAnchor::SourceRange(span) = row.source.anchor else {
                        panic!("It origin")
                    };
                    assert_eq!(&source[span.start..span.end], "it");
                    assert!(
                        row.source
                            .provenance
                            .iter()
                            .any(|provenance| provenance.phase == CoreProvenancePhase::Generated)
                    );
                }
            }
            for (id, row) in core.formulas().iter() {
                assert_eq!(
                    core.source_map().formula_sources.get(&id),
                    Some(&row.source)
                );
            }

            assert_eq!(vcs.vcs().len(), 3);
            assert_eq!(
                vcs.vcs()
                    .iter()
                    .filter(|vc| vc.kind == mizar_vc::vc_ir::VcKind::DefinitionCorrectness)
                    .count(),
                2
            );
            assert_eq!(
                vcs.vcs()
                    .iter()
                    .filter(|vc| vc.kind == mizar_vc::vc_ir::VcKind::TheoremProofStep)
                    .count(),
                1
            );
            assert!(
                vcs.vcs()
                    .iter()
                    .all(|vc| vc.status == mizar_vc::vc_ir::VcStatus::Open)
            );
            for (_, seed) in core.obligation_seeds().iter() {
                assert_eq!(
                    vcs.vcs()
                        .iter()
                        .filter(
                            |vc| vc.goal == mizar_vc::vc_ir::VcFormulaRef::Core(seed.goal.unwrap())
                        )
                        .count(),
                    1
                );
            }
        };
        let sources = [
            (text.clone(), false, false),
            (text.replace("means it = X", "means X = it"), true, false),
            (
                text.replace("holds choicebox X = X", "holds X = choicebox X")
                    .replace("thus choicebox X = X", "thus X = choicebox X"),
                false,
                true,
            ),
            (
                text.replace("choicebox", "parcel")
                    .replace("ChoiceBoxDef", "ParcelDef")
                    .replace("ChoiceBox1", "Parcel1"),
                false,
                false,
            ),
            (
                text.replace("let X be set;\n  func", "let Formal be set;\n  func")
                    .replace(
                        "choicebox X -> set means it = X",
                        "choicebox Formal -> set means it = Formal",
                    ),
                false,
                false,
            ),
            (
                text.replace(
                    "for X being set holds choicebox X = X",
                    "for Header being set holds choicebox Header = Header",
                ),
                false,
                false,
            ),
            (
                text.replace(
                    "proof\n  let X be set;\n  thus choicebox X = X",
                    "proof\n  let Local be set;\n  thus choicebox Local = Local",
                ),
                false,
                false,
            ),
        ];
        for (source, swapped_body, swapped_goal) in sources {
            let build = || {
                phrase_theorem_output(
                    &config.workspace_root,
                    case,
                    super::super::formula_statement::step5c8_test_frontend(&source),
                )
                .unwrap()
            };
            let (core, vcs) = build();
            assert_eq!((core.clone(), vcs.clone()), build());
            validate(&core, &vcs, &source, swapped_body, swapped_goal);
            if source == text {
                for mutation in 0..7 {
                    let mut parts = CoreIrParts {
                        source_id: core.source_id(),
                        module_id: core.module_id().clone(),
                        items: core.items().clone(),
                        terms: core.terms().clone(),
                        formulas: core.formulas().clone(),
                        definitions: core.definitions().clone(),
                        proofs: core.proofs().clone(),
                        proof_nodes: core.proof_nodes().clone(),
                        algorithms: core.algorithms().clone(),
                        algorithm_statements: core.algorithm_statements().clone(),
                        generated: core.generated().clone(),
                        obligation_seeds: core.obligation_seeds().clone(),
                        source_map: core.source_map().clone(),
                        diagnostics: core.diagnostics().clone(),
                    };
                    let (definition_id, definition) = core.definitions().iter().next().unwrap();
                    let DefinitionBody::Formula(body) = definition.body else {
                        unreachable!()
                    };
                    match mutation {
                        0 => {
                            parts.definitions.get_mut(definition_id).unwrap().expansion =
                                ExpansionPolicy::Transparent
                        }
                        1 | 2 => {
                            let removed = definition.correctness[mutation - 1];
                            let remap = |id: ObligationSeedId| {
                                ObligationSeedId::new(
                                    id.index() - usize::from(id.index() > removed.index()),
                                )
                            };
                            parts.obligation_seeds = ObligationSeedTable::new();
                            for (id, row) in core.obligation_seeds().iter() {
                                if id == removed {
                                    continue;
                                }
                                let mut row = row.clone();
                                row.core_refs.retain(|reference| {
                                    *reference != CoreNodeRef::ObligationSeed(removed)
                                });
                                for reference in &mut row.core_refs {
                                    if let CoreNodeRef::ObligationSeed(id) = reference {
                                        *id = remap(*id);
                                    }
                                }
                                parts.obligation_seeds.insert(row);
                            }
                            let owned = &mut parts
                                .definitions
                                .get_mut(definition_id)
                                .unwrap()
                                .correctness;
                            owned.retain(|id| *id != removed);
                            for id in owned {
                                *id = remap(*id);
                            }
                            for (_, row) in parts.proof_nodes.iter_mut() {
                                if let CoreProofNodeKind::TerminalGoal { obligation, .. } =
                                    &mut row.kind
                                {
                                    *obligation = remap(*obligation);
                                }
                            }
                            parts.source_map.obligation_sources = parts
                                .obligation_seeds
                                .iter()
                                .map(|(id, row)| (id, row.source.clone()))
                                .collect();
                        }
                        3 => {
                            let (_, seed) = parts
                                .obligation_seeds
                                .iter_mut()
                                .find(|(_, row)| row.kind == ObligationSeedKind::TheoremProof)
                                .unwrap();
                            seed.context.push(body);
                            seed.core_refs.push(CoreNodeRef::Formula(body));
                        }
                        4 | 5 => {
                            let from = CoreVarId::new(if mutation == 4 { 3 } else { 1 });
                            let into = CoreVarId::new(2);
                            for (_, row) in parts.terms.iter_mut() {
                                if row.kind == CoreTermKind::Var(from) {
                                    row.kind = CoreTermKind::Var(into);
                                }
                            }
                            for (_, row) in parts.formulas.iter_mut() {
                                if let CoreFormulaKind::Forall { binders, .. }
                                | CoreFormulaKind::Exists { binders, .. } = &mut row.kind
                                {
                                    for binder in binders {
                                        if binder.var == from {
                                            binder.var = into;
                                        }
                                    }
                                }
                            }
                            for (_, row) in parts.proof_nodes.iter_mut() {
                                if let CoreProofNodeKind::IntroduceBinder { binder, .. } =
                                    &mut row.kind
                                    && binder.var == from
                                {
                                    binder.var = into;
                                }
                            }
                        }
                        6 => {
                            let CoreFormulaKind::Equals { left, right } =
                                &mut parts.formulas.get_mut(body).unwrap().kind
                            else {
                                unreachable!()
                            };
                            std::mem::swap(left, right);
                        }
                        _ => unreachable!(),
                    }
                    let hostile = CoreIr::try_new(parts)
                        .unwrap_or_else(|error| panic!("structural mutation {mutation}: {error}"));
                    let hostile_vcs = generate_core_vcs(&hostile, snapshot_id(0)).unwrap();
                    // Detached Core validation is structural; this source contract must still detect the forgery.
                    assert!(
                        std::panic::catch_unwind(|| validate(
                            &hostile,
                            &hostile_vcs,
                            &source,
                            false,
                            false
                        ))
                        .is_err(),
                        "undetected mutation {mutation}"
                    );
                }
            }
            if source == text {
                let (_, actual) = phrase_theorem_output(
                    &config.workspace_root,
                    case,
                    run_frontend(&config.workspace_root, case, 0).unwrap(),
                )
                .unwrap();
                assert_eq!(fs::read_to_string(config.workspace_root.join("tests/snapshots/vc/pass_proof_verification_func_means_prefix_001.vc_ir.snap")).unwrap(), actual.debug_text());
            }
        }
    }

    #[test]
    fn step5c5_means_and_equals_authenticate_application_and_clause_receipts() {
        use mizar_checker::{source_application::*, source_term::*, type_checker::*, typed_ast::*};
        use mizar_resolve::{labels::*, names::*};
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        for id in [STEP5C5_MEANS_ID, STEP5C5_EQUALS_ID] {
            let equals = id == STEP5C5_EQUALS_ID;
            let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
            let frontend = run_frontend(&config.workspace_root, case, 0).unwrap();
            let ast = frontend.ast.clone().unwrap();
            let (source, neutral, symbols) =
                super::super::source_registration_inputs(&config.workspace_root, case, frontend)
                    .unwrap();
            let (bindings, _, typed) =
                TermFormulaChecker::check_source_predicate_statements(&source, &symbols, &neutral)
                    .unwrap();
            let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(&ast, source.module(), &symbols),
            )
            .unwrap();
            assert_eq!(scope.bindings().len(), if equals { 4 } else { 2 });
            assert_eq!(bindings.bindings().len(), if equals { 6 } else { 3 });
            assert!(typed.initial_obligations().is_empty());
            let owner = symbols
                .symbols()
                .iter()
                .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
                .unwrap();
            let namespace = mizar_resolve::env::NamespacePath::new(source.module().path().as_str());
            let labels = ProofLabelSourceCollector::new(
                &ast,
                source.module(),
                namespace.clone(),
                owner.contribution(),
                &source,
            )
            .unwrap()
            .collect_with_theorem_owners(&symbols)
            .unwrap();
            let resolved = LabelResolver::new(labels.projections()).resolve(
                source.module(),
                &namespace,
                labels.references(),
            );
            let check = |typed: &TypedAst| {
                SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &source, typed, &scope, &symbols, &labels, &resolved, None,
                )
                .map(|_| ())
            };
            assert!(check(&typed).is_ok());
            let primary = typed.source_term().unwrap();
            let application = typed.source_application().unwrap();
            let application_input = SourceFunctorApplicationHandoffInput {
                source_id: typed.source_id(),
                module_id: typed.module_id().clone(),
                wrappers: Vec::new(),
                applications: application
                    .applications()
                    .iter()
                    .map(|(_, row)| SourceFunctorApplicationInput {
                        site: row.site().clone(),
                        source_range: row.source_range(),
                        source_ordinal: row.source_ordinal(),
                        context: row.context(),
                        recovery: row.recovery(),
                        spelling: row.spelling().into(),
                        kind: row.kind(),
                        form: row.form(),
                        head_ordinal: row.head_ordinal(),
                        head: row.head().clone(),
                    })
                    .collect(),
                candidates: application
                    .candidates()
                    .iter()
                    .map(|(_, row)| SourceFunctorCandidateInput {
                        application: row.application(),
                        ordinal: row.ordinal(),
                        symbol: row.symbol().clone(),
                        contribution: row.contribution(),
                    })
                    .collect(),
                arguments: application
                    .arguments()
                    .iter()
                    .map(|(_, row)| SourceFunctorArgumentInput {
                        application: row.application(),
                        ordinal: row.ordinal(),
                        target: row.target(),
                    })
                    .collect(),
                type_requests: application
                    .type_requests()
                    .iter()
                    .map(|(_, row)| SourceFunctorTypeRequestInput {
                        application: row.application(),
                        candidate: row.candidate(),
                        request_ordinal: row.request_ordinal(),
                        kind: row.kind(),
                    })
                    .collect(),
            };
            assert_eq!(
                SourceFunctorApplicationProducer::build(
                    application_input.clone(),
                    &symbols,
                    &bindings,
                    primary,
                    typed.nodes()
                )
                .unwrap(),
                *application
            );
            for mutation in 0..5 {
                let mut input = application_input.clone();
                let expected = match mutation {
                    0 => {
                        input.candidates[0].symbol = owner.symbol().clone();
                        SourceFunctorApplicationError::InvalidCandidate {
                            candidate: SourceFunctorCandidateId::new(0),
                        }
                    }
                    1 => {
                        input.arguments[0].ordinal = 1;
                        SourceFunctorApplicationError::ReorderedArgument {
                            argument: SourceFunctorArgumentId::new(0),
                        }
                    }
                    2 => {
                        input.type_requests[1].candidate = Some(SourceFunctorCandidateId::new(0));
                        SourceFunctorApplicationError::InvalidTypeRequest {
                            request: SourceFunctorTypeRequestId::new(1),
                        }
                    }
                    3 => {
                        input.applications[0].form = if equals {
                            SourceFunctorApplicationForm::Prefix
                        } else {
                            SourceFunctorApplicationForm::Infix
                        };
                        SourceFunctorApplicationError::InvalidForm {
                            application: SourceFunctorApplicationId::new(0),
                        }
                    }
                    4 => {
                        input.arguments[0].target = SourceFunctorArgumentTarget::Primary(
                            SourcePrimaryTermId::new(if equals { 4 } else { 3 }),
                        );
                        if equals {
                            SourceFunctorApplicationError::DuplicateArgumentTarget {
                                argument: SourceFunctorArgumentId::new(0),
                            }
                        } else {
                            SourceFunctorApplicationError::InvalidArgument {
                                argument: SourceFunctorArgumentId::new(0),
                            }
                        }
                    }
                    _ => unreachable!(),
                };
                assert_eq!(
                    SourceFunctorApplicationProducer::build(
                        input,
                        &symbols,
                        &bindings,
                        primary,
                        typed.nodes()
                    )
                    .unwrap_err(),
                    expected
                );
            }
            let mut primary_input = SourcePrimaryTermHandoffInput {
                source_id: typed.source_id(),
                module_id: typed.module_id().clone(),
                numeric_type_requests: Vec::new(),
                terms: primary
                    .terms()
                    .iter()
                    .map(|(_, row)| SourcePrimaryTermInput {
                        site: row.site().clone(),
                        source_range: row.source_range(),
                        source_ordinal: row.source_ordinal(),
                        context: row.context(),
                        recovery: row.recovery(),
                        spelling: row.spelling().into(),
                        kind: row.kind(),
                        role: row.role(),
                        parent: row.parent(),
                    })
                    .collect(),
                references: primary
                    .references()
                    .iter()
                    .map(|(_, row)| SourcePrimaryTermReferenceInput {
                        term: row.term(),
                        binding: row.binding(),
                        role: row.role(),
                    })
                    .collect(),
            };
            let mut wrong_formal = primary_input.clone();
            wrong_formal.references[0].binding = bindings.bindings().iter().nth(1).unwrap().0;
            assert_eq!(
                SourcePrimaryTermProducer::build(wrong_formal, &bindings, typed.nodes())
                    .unwrap_err(),
                SourcePrimaryTermError::InvalidReference {
                    reference: SourcePrimaryTermReferenceId::new(0)
                }
            );
            primary_input.terms[0].role = if equals {
                SourcePrimaryTermRole::CurrentDefinitionResult
            } else {
                SourcePrimaryTermRole::Value
            };
            assert_eq!(
                SourcePrimaryTermProducer::build(primary_input, &bindings, typed.nodes())
                    .unwrap_err(),
                SourcePrimaryTermError::InvalidTerm {
                    term: SourcePrimaryTermId::new(0)
                }
            );
            let clauses = source
                .arena()
                .iter()
                .filter(|(_, node)| {
                    matches!(
                        node.kind(),
                        mizar_syntax::SurfaceNodeKind::CorrectnessCondition
                    )
                })
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            for mutation in 0..6 {
                let mut nodes = typed
                    .nodes()
                    .iter()
                    .map(|(_, row)| row.clone())
                    .collect::<Vec<_>>();
                let first = clauses[0];
                let second = if equals {
                    source
                        .arena()
                        .iter()
                        .find(|(_, node)| {
                            matches!(
                                node.kind(),
                                mizar_syntax::SurfaceNodeKind::DefinitionParameter
                            )
                        })
                        .unwrap()
                        .0
                } else {
                    clauses[1]
                };
                match mutation {
                    0 if equals => nodes[first.index()].children.reverse(),
                    0 => nodes[first.index()].children = nodes[second.index()].children.clone(),
                    1 => nodes[first.index()].anchor = nodes[second.index()].anchor.clone(),
                    2 => nodes[first.index()].resolved_node = Some(second),
                    3 => {
                        nodes[primary
                            .terms()
                            .iter()
                            .next()
                            .unwrap()
                            .1
                            .site()
                            .node()
                            .index()]
                        .resolved_node = Some(source.arena().root())
                    }
                    4 => {
                        let id = source
                            .arena()
                            .iter()
                            .find(|(_, node)| {
                                matches!(
                                    node.kind(),
                                    mizar_syntax::SurfaceNodeKind::DefinitionParameter
                                )
                            })
                            .unwrap()
                            .0;
                        nodes[id.index()].children.reverse();
                    }
                    5 => {
                        let definition = source
                            .arena()
                            .iter()
                            .find(|(_, node)| {
                                matches!(
                                    node.kind(),
                                    mizar_syntax::SurfaceNodeKind::FunctorDefinition
                                )
                            })
                            .unwrap()
                            .1;
                        let ty = definition.children()[5];
                        let head = source.arena().node(ty).unwrap().children()[0];
                        let token = source.arena().node(head).unwrap().children()[0];
                        nodes[token.index()].kind =
                            r#"Token(SurfaceToken { kind: ReservedWord, text: "object" })"#.into();
                    }
                    _ => unreachable!(),
                }
                let changed = TypedAst::try_new(TypedAstParts {
                    source_id: typed.source_id(),
                    module_id: typed.module_id().clone(),
                    resolved_root: Some(source.arena().root()),
                    source_context: None,
                    source_type: None,
                    source_attribute: None,
                    nodes: TypedArena::try_new(typed.nodes().root(), nodes).unwrap(),
                    contexts: LocalTypeContextTable::new(),
                    types: TypeTable::new(),
                    facts: TypeFactTable::new(),
                    coercions: CoercionTable::new(),
                    initial_obligations: InitialObligationTable::new(),
                    diagnostics: TypeDiagnosticTable::new(),
                })
                .unwrap()
                .with_source_term(primary.clone())
                .unwrap()
                .with_source_application(application.clone())
                .unwrap()
                .with_source_atomic_formula(typed.source_atomic_formula().unwrap().clone())
                .unwrap();
                assert_eq!(check(&changed).unwrap_err(), "theorems.source.invalid");
            }
            let text = fs::read_to_string(&case.source_path).unwrap();
            for foreign in [
                if equals {
                    text.replace("+", "*")
                } else {
                    text.replace("choicebox", "foreignbox")
                },
                if equals {
                    text.replace("equals X", "equals Y")
                } else {
                    text.replace("means it = X", "means X = it")
                },
            ] {
                let (other, nodes, env) = super::super::source_registration_inputs(
                    &config.workspace_root,
                    case,
                    super::super::formula_statement::step5c8_test_frontend(&foreign),
                )
                .unwrap();
                let (_, _, other_typed) =
                    TermFormulaChecker::check_source_predicate_statements(&other, &env, &nodes)
                        .unwrap();
                assert!(check(&other_typed).is_err());
            }
        }
    }

    #[test]
    fn step5c5_means_source_types_and_correctness_clauses() {
        use mizar_checker::{source_term::*, type_checker::*};
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_MEANS_ID)
            .unwrap();
        let text = fs::read_to_string(&case.source_path).unwrap();
        let check = |text: &str| {
            let (source, neutral, symbols) = super::super::source_registration_inputs(
                &config.workspace_root,
                case,
                super::super::formula_statement::step5c8_test_frontend(text),
            )?;
            TermFormulaChecker::check_source_predicate_statements(&source, &symbols, &neutral)
        };
        for renamed in [
            text.clone(),
            text.replace("choicebox", "parcel")
                .replace("ChoiceBoxDef", "ParcelDef")
                .replace("ChoiceBox1", "Parcel1"),
            text.replace("let X be set;\n  func", "let Formal be set;\n  func")
                .replace(
                    "choicebox X -> set means it = X",
                    "choicebox Formal -> set means it = Formal",
                ),
            text.replace(
                "for X being set holds choicebox X = X",
                "for Header being set holds choicebox Header = Header",
            ),
            text.replace(
                "proof\n  let X be set;\n  thus choicebox X = X",
                "proof\n  let Local be set;\n  thus choicebox Local = Local",
            ),
            text.replace("means it = X", "means X = it"),
            text.replace("holds choicebox X = X", "holds X = choicebox X")
                .replace("thus choicebox X = X", "thus X = choicebox X"),
        ] {
            let (bindings, inference, typed) =
                check(&renamed).unwrap_or_else(|error| panic!("{error}: {renamed}"));
            assert_eq!(bindings.bindings().len(), 3);
            assert_eq!(inference.terms().len(), 8);
            assert_eq!(inference.formulas().len(), 3);
            assert!(inference.facts().is_empty());
            assert!(inference.diagnostics().is_empty());
            let primary = typed.source_term().unwrap();
            assert_eq!(primary.terms().len(), 6);
            assert_eq!(primary.references().len(), 5);
            let it = primary
                .terms()
                .iter()
                .find(|(_, term)| term.kind() == SourcePrimaryTermKind::It)
                .unwrap();
            assert_eq!(it.1.role(), SourcePrimaryTermRole::CurrentDefinitionResult);
            assert!(
                !primary
                    .references()
                    .iter()
                    .any(|(_, reference)| reference.term() == it.0)
            );
            let applications = typed.source_application().unwrap();
            assert_eq!(applications.applications().len(), 2);
            assert_eq!(applications.arguments().len(), 2);
            assert_eq!(applications.candidates().len(), 2);
            assert!(typed.initial_obligations().is_empty());
        }
        for changed in [
            text.replace("means it = X", "means X = X"),
            text.replace("means it = X", "means it = Unknown"),
            text.replace("means it = X", "means it = it"),
            text.replace("means it = X", "means not it = X"),
            text.replace("means it = X", "means it = X & X = X"),
            text.replace("-> set", "-> object"),
            text.replace("for X being set", "for X being object"),
            text.replace("thus choicebox X = X", "thus choicebox Missing = X"),
            text.replace("choicebox X ->", "X choicebox X ->"),
            text.replace("  existence;\n", ""),
            text.replace("  uniqueness;\n", ""),
            text.replace("existence;\n  uniqueness;", "uniqueness;\n  existence;"),
            text.replace("existence;", "existence; existence;"),
            text.replace("existence;", "existence by ChoiceBox1;"),
            text.replace("uniqueness;", "uniqueness; commutativity;"),
            text.replace(
                "thus choicebox X = X;",
                "thus choicebox X = X by ChoiceBox1;",
            ),
        ] {
            assert_ne!(changed, text);
            assert!(check(&changed).is_err(), "accepted {changed}");
        }
    }

    #[test]
    fn step5c5_phrase_preserves_guarded_definition_and_open_goal() {
        use mizar_core::core_ir::*;
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        for case in plan
            .cases
            .iter()
            .filter(|case| [STEP5C5_PHRASE_ID, STEP5C5_SYMBOLIC_ID].contains(&case.id.0.as_str()))
        {
            let symbolic = case.id.0 == STEP5C5_SYMBOLIC_ID;
            let build = || {
                phrase_theorem_output(
                    &config.workspace_root,
                    case,
                    run_frontend(&config.workspace_root, case, 0).unwrap(),
                )
                .unwrap()
            };
            let (core, vcs) = build();
            assert_eq!((core.clone(), vcs.clone()), build());
            assert_eq!(core.items().len(), 2);
            assert_eq!(core.definitions().len(), 1);
            assert_eq!(core.proofs().len(), 1);
            assert_eq!(core.proof_nodes().len(), 2);
            assert_eq!(core.obligation_seeds().len(), 1);
            let (_, definition) = core.definitions().iter().next().unwrap();
            assert_eq!(definition.params.len(), 2);
            assert_eq!(definition.expansion, ExpansionPolicy::Transparent);
            assert!(definition.correctness.is_empty());
            let DefinitionBody::Formula(equality) = definition.body else {
                panic!("definition body")
            };
            let variable = |id| match core.terms().get(id).unwrap().kind {
                CoreTermKind::Var(var) => var,
                _ => panic!("variable term"),
            };
            let nested = if symbolic {
                let CoreFormulaKind::Forall { binders, body } =
                    &core.formulas().get(equality).unwrap().kind
                else {
                    panic!("quantified membership body")
                };
                assert_eq!(binders.len(), 1);
                let nested = &binders[0];
                let CoreFormulaKind::TypePred { subject, ty } =
                    &core.formulas().get(nested.ty_guard.unwrap()).unwrap().kind
                else {
                    panic!("object guard")
                };
                assert_eq!(ty.as_str(), "object");
                assert_eq!(variable(*subject), nested.var);
                let CoreFormulaKind::Implies {
                    premise,
                    conclusion,
                } = core.formulas().get(*body).unwrap().kind
                else {
                    panic!("source membership implication")
                };
                for (formula, container) in [
                    (premise, definition.params[0].var),
                    (conclusion, definition.params[1].var),
                ] {
                    let CoreFormulaKind::Membership { element, set } =
                        core.formulas().get(formula).unwrap().kind
                    else {
                        panic!("primitive membership")
                    };
                    assert_eq!(variable(element), nested.var);
                    assert_eq!(variable(set), container);
                    assert_ne!(
                        core.terms().get(element).unwrap().source,
                        core.terms().get(set).unwrap().source
                    );
                    for source in [
                        &core.formulas().get(formula).unwrap().source,
                        &core.terms().get(element).unwrap().source,
                        &core.terms().get(set).unwrap().source,
                    ] {
                        assert!(
                            source
                                .provenance
                                .iter()
                                .any(|row| row.key.as_str().starts_with("checker/predicate/"))
                        );
                    }
                }
                Some(nested.var)
            } else {
                let CoreFormulaKind::Equals { left, right } =
                    core.formulas().get(equality).unwrap().kind
                else {
                    panic!("ordered equality body")
                };
                assert_eq!(
                    [variable(left), variable(right)],
                    [definition.params[0].var, definition.params[1].var]
                );
                assert_ne!(
                    core.terms().get(left).unwrap().source,
                    core.terms().get(right).unwrap().source
                );
                for source in [
                    &core.formulas().get(equality).unwrap().source,
                    &core.terms().get(left).unwrap().source,
                    &core.terms().get(right).unwrap().source,
                ] {
                    assert!(
                        source
                            .provenance
                            .iter()
                            .any(|row| row.key.as_str().starts_with("checker/predicate/"))
                    );
                    assert!(
                        !source
                            .provenance
                            .iter()
                            .any(|row| row.key.as_str().starts_with("checker/theorem/"))
                    );
                }
                None
            };

            let (_, proof) = core.proofs().iter().next().unwrap();
            assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
            let CoreFormulaKind::Forall {
                binders: header,
                body: header_goal,
            } = &core.formulas().get(proof.proposition).unwrap().kind
            else {
                panic!("universal theorem")
            };
            let CoreProofNodeKind::IntroduceBinder {
                binder: local,
                child,
            } = &core.proof_nodes().get(proof.root).unwrap().kind
            else {
                panic!("local binder")
            };
            let CoreProofNodeKind::TerminalGoal {
                obligation,
                citations,
            } = &core.proof_nodes().get(*child).unwrap().kind
            else {
                panic!("terminal goal")
            };
            assert!(citations.is_empty());
            let seed = core.obligation_seeds().get(*obligation).unwrap();
            assert_eq!(seed.status, ObligationSeedStatus::Active);
            assert_eq!(seed.kind, ObligationSeedKind::TheoremProof);
            assert_eq!(seed.owner, proof.item);
            assert_eq!(seed.context.len(), 2);
            assert_eq!(seed.context[0], local.ty_guard.unwrap());
            assert_eq!(
                [
                    definition.params[0].var,
                    definition.params[1].var,
                    header[0].var,
                    local.var
                ]
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
                4
            );
            if let Some(nested) = nested {
                assert!(
                    ![
                        definition.params[0].var,
                        definition.params[1].var,
                        header[0].var,
                        local.var
                    ]
                    .contains(&nested)
                );
            }
            for (formula, var) in [
                (*header_goal, header[0].var),
                (seed.goal.unwrap(), local.var),
            ] {
                let CoreFormulaKind::Atom { predicate, args } =
                    &core.formulas().get(formula).unwrap().kind
                else {
                    panic!("actual predicate goal")
                };
                assert_eq!(predicate, &definition.symbol);
                assert_eq!(
                    args.iter().map(|id| variable(*id)).collect::<Vec<_>>(),
                    [var, var]
                );
                assert_ne!(
                    core.terms().get(args[0]).unwrap().source,
                    core.terms().get(args[1]).unwrap().source
                );
            }
            let CoreFormulaKind::Forall {
                binders: formals,
                body: biconditional,
            } = &core.formulas().get(seed.context[1]).unwrap().kind
            else {
                panic!("guarded definition")
            };
            assert_eq!(
                formals.iter().map(|binder| binder.var).collect::<Vec<_>>(),
                definition
                    .params
                    .iter()
                    .map(|binder| binder.var)
                    .collect::<Vec<_>>()
            );
            for binder in formals.iter().chain(header).chain([local]) {
                let CoreFormulaKind::TypePred { subject, ty } =
                    &core.formulas().get(binder.ty_guard.unwrap()).unwrap().kind
                else {
                    panic!("set guard")
                };
                assert_eq!(ty.as_str(), "set");
                assert_eq!(variable(*subject), binder.var);
            }
            let CoreFormulaKind::Iff {
                left: atom,
                right: body,
            } = core.formulas().get(*biconditional).unwrap().kind
            else {
                panic!("biconditional")
            };
            assert_eq!(body, equality);
            let CoreFormulaKind::Atom { predicate, args } =
                &core.formulas().get(atom).unwrap().kind
            else {
                panic!("definition atom")
            };
            assert_eq!(predicate, &definition.symbol);
            assert_eq!(
                args.iter().map(|id| variable(*id)).collect::<Vec<_>>(),
                formals.iter().map(|binder| binder.var).collect::<Vec<_>>()
            );
            let [vc] = vcs.vcs() else { panic!("one VC") };
            assert_eq!(vc.kind, mizar_vc::vc_ir::VcKind::TheoremProofStep);
            assert_eq!(vc.status, mizar_vc::vc_ir::VcStatus::Open);
            assert_eq!(
                vc.goal,
                mizar_vc::vc_ir::VcFormulaRef::Core(seed.goal.unwrap())
            );
            assert_eq!(
                vc.local_context
                    .entries()
                    .iter()
                    .filter_map(|entry| entry.formula)
                    .collect::<Vec<_>>(),
                seed.context
                    .iter()
                    .copied()
                    .map(mizar_vc::vc_ir::VcFormulaRef::Core)
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                fs::read_to_string(
                    config
                        .workspace_root
                        .join("tests")
                        .join(case.expectation.snapshots.as_ref().unwrap())
                )
                .unwrap(),
                vcs.debug_text()
            );
            let result = run_proof_verification_case(
                &config.workspace_root,
                &config.workspace_root.join("tests"),
                case,
                333,
            );
            assert_eq!(
                result.status,
                ProofVerificationCaseStatus::Passed,
                "{:?}",
                result.failure
            );
        }
    }

    #[test]
    fn step5c5_symbolic_source_operands_and_scopes_are_exact() {
        use mizar_core::core_ir::*;
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_SYMBOLIC_ID)
            .unwrap();
        let text = fs::read_to_string(&case.source_path).unwrap();
        let check = |text: &str| {
            phrase_theorem_output(
                &config.workspace_root,
                case,
                super::super::formula_statement::step5c8_test_frontend(text),
            )
        };
        for renamed in [
            text.clone(),
            text.replace("SubsetLikeDef", "OtherDefinition"),
            text.replace("SubsetRefl1", "OtherTheorem"),
            text.replace("<<=", ">>="),
            text.replace("X, Y", "First, Y")
                .replace("X <<= Y means", "First <<= Y means")
                .replace("z in X", "z in First"),
            text.replace("X, Y", "X, Second")
                .replace("X <<= Y means", "X <<= Second means")
                .replace("z in Y", "z in Second"),
            text.replace(
                "for z being object st z in X holds z in Y",
                "for Element being object st Element in X holds Element in Y",
            ),
            text.replace(
                "for X being set holds X <<= X",
                "for Header being set holds Header <<= Header",
            ),
            text.replace("let X be set;", "let Local be set;")
                .replace("thus X <<= X", "thus Local <<= Local"),
            text.replace(
                "for X being set holds X <<= X",
                "for z being set holds z <<= z",
            )
            .replace("let X be set;", "let z be set;")
            .replace("thus X <<= X", "thus z <<= z"),
        ] {
            assert!(
                check(&renamed).is_ok(),
                "{renamed}: {:?}",
                check(&renamed).err()
            );
        }
        let baseline = check(&text).unwrap();
        for (changed, indices) in [
            (
                text.replace("z in X holds z in Y", "z in Y holds z in X"),
                [1, 0],
            ),
            (
                text.replace("z in X holds z in Y", "z in X holds z in X"),
                [0, 0],
            ),
            (
                text.replace(
                    "for z being object st z in X holds z in Y",
                    "for X being object st X in Y holds X in Y",
                ),
                [1, 1],
            ),
        ] {
            let output = check(&changed).unwrap();
            assert_ne!(output.0, baseline.0);
            assert_eq!(output, check(&changed).unwrap());
            let core = &output.0;
            let (_, definition) = core.definitions().iter().next().unwrap();
            assert_eq!(definition.params.len(), 2);
            assert_ne!(definition.params[0].var, definition.params[1].var);
            assert!(
                definition
                    .params
                    .iter()
                    .all(|binder| binder.ty_guard.is_some())
            );
            let DefinitionBody::Formula(body) = definition.body else {
                panic!()
            };
            let CoreFormulaKind::Forall { binders, body } =
                &core.formulas().get(body).unwrap().kind
            else {
                panic!()
            };
            assert!(binders[0].ty_guard.is_some());
            assert!(
                definition
                    .params
                    .iter()
                    .all(|formal| formal.var != binders[0].var)
            );
            let mut variables = definition
                .params
                .iter()
                .map(|binder| binder.var)
                .collect::<std::collections::BTreeSet<_>>();
            variables.insert(binders[0].var);
            for (_, proof_node) in core.proof_nodes().iter() {
                if let CoreProofNodeKind::IntroduceBinder { binder, .. } = &proof_node.kind {
                    variables.insert(binder.var);
                }
            }
            for (_, formula) in core.formulas().iter() {
                if let CoreFormulaKind::Forall { binders, .. } = &formula.kind {
                    variables.extend(binders.iter().map(|binder| binder.var));
                }
            }
            assert_eq!(variables.len(), 5);
            let CoreFormulaKind::Implies {
                premise,
                conclusion,
            } = core.formulas().get(*body).unwrap().kind
            else {
                panic!()
            };
            for (formula, index) in [premise, conclusion].into_iter().zip(indices) {
                let CoreFormulaKind::Membership { element, set } =
                    core.formulas().get(formula).unwrap().kind
                else {
                    panic!()
                };
                assert_eq!(
                    core.terms().get(element).unwrap().kind,
                    CoreTermKind::Var(binders[0].var)
                );
                assert_eq!(
                    core.terms().get(set).unwrap().kind,
                    CoreTermKind::Var(definition.params[index].var)
                );
            }
        }
        for changed in [
            text.replace("z in Y", "z in Missing"),
            text.replace("z in Y", "z in z"),
            text.replace(
                "for z being object st z in X holds z in Y",
                "for X being object st X in X holds X in Y",
            ),
            text.replace("for z being object", "for z being set"),
            text.replace("holds z in Y", "holds not z in Y"),
            text.replace("holds z in Y", "holds z in Y & z in X"),
            text.replace("holds z in Y", "holds for w being object holds w in Y"),
            text.replace("thus X <<= X;", "thus X <<= X by SubsetRefl1;"),
            text.replace("thus X <<= X", "thus not X <<= X"),
            text.replace("z in Y;", "z in Y; symmetry;"),
            text.replace("let X be set", "let X be object"),
        ] {
            assert_ne!(changed, text);
            assert!(check(&changed).is_err(), "accepted {changed}");
        }
    }

    #[test]
    fn step5c5_phrase_source_renaming_and_near_misses() {
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C5_PHRASE_ID)
            .unwrap();
        let text = fs::read_to_string(&case.source_path).unwrap();
        let check = |text: &str| {
            phrase_theorem_output(
                &config.workspace_root,
                case,
                super::super::formula_statement::step5c8_test_frontend(text),
            )
        };
        for renamed in [
            text.clone(),
            text.replace("LikesDef", "PredicateLabel"),
            text.replace("LikesSelf1", "TheoremLabel"),
            text.replace("likes", "admires"),
            text.replace("X, Y", "First, Y")
                .replace("X likes Y means X = Y", "First likes Y means First = Y"),
            text.replace("X, Y", "X, Second")
                .replace("X likes Y means X = Y", "X likes Second means X = Second"),
            text.replace(
                "for X being set holds X likes X",
                "for Header being set holds Header likes Header",
            ),
            text.replace("let X be set;", "let Local be set;")
                .replace("thus X likes X", "thus Local likes Local"),
        ] {
            assert!(
                check(&renamed).is_ok(),
                "{renamed}: {:?}",
                check(&renamed).err()
            );
        }
        for changed in [
            text.replace("likes", "~"),
            text.replace("means X = Y", "means Y = X"),
            text.replace("means X = Y", "means X = X"),
            text.replace("means X = Y", "means not X = Y"),
            text.replace("thus X likes X", "thus X does not likes X"),
            text.replace("holds X likes X", "holds X does not likes X"),
            text.replace("thus X likes X", "thus X admires X"),
            text.replace("thus X likes X", "thus X likes Y"),
            text.replace("for X being set", "for X being object"),
            text.replace("let X be set", "let X be object"),
            text.replace("let X be set", "let X be non empty set"),
            text.replace("let X be set;", "let X be set; assume A: X = X;"),
            text.replace("thus X likes X;", "thus X likes X by LikesSelf1;"),
            text.replace("means X = Y;", "means X = Y; symmetry;"),
            text.replace("X, Y", "X, X"),
            text.replace(
                "end;\n\ntheorem",
                "end;\ndefinition let A,B be set; pred Extra: A loves B means A = B; end;\ntheorem",
            ),
            text.replace("thus X likes X;", "thus X likes X; thus X likes X;"),
            text.replace("end;", ""),
        ] {
            assert_ne!(changed, text, "ineffective mutation");
            assert!(check(&changed).is_err(), "accepted {changed}");
        }
    }

    #[test]
    fn step5c5_phrase_authenticates_receipts_and_binding_correspondence() {
        use mizar_checker::{
            type_checker::{SourceVariableSemanticsChecker, TermFormulaChecker},
            typed_ast::*,
        };
        use mizar_resolve::{
            labels::{LabelResolver, ProofLabelSourceCollector},
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        };
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        for case in plan
            .cases
            .iter()
            .filter(|case| [STEP5C5_PHRASE_ID, STEP5C5_SYMBOLIC_ID].contains(&case.id.0.as_str()))
        {
            let symbolic = case.id.0 == STEP5C5_SYMBOLIC_ID;
            let frontend = run_frontend(&config.workspace_root, case, 0).unwrap();
            let ast = frontend.ast.clone().unwrap();
            let (source, neutral, symbols) =
                super::super::source_registration_inputs(&config.workspace_root, case, frontend)
                    .unwrap();
            let (bindings, inference, typed) =
                TermFormulaChecker::check_source_predicate_statements(&source, &symbols, &neutral)
                    .unwrap();
            assert_eq!(bindings.bindings().len(), if symbolic { 5 } else { 4 });
            assert_eq!(
                bindings
                    .bindings()
                    .iter()
                    .map(|(_, row)| (row.declaration_range.start, row.declaration_range.end))
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                if symbolic { 5 } else { 4 }
            );
            assert_eq!(inference.terms().len(), if symbolic { 8 } else { 6 });
            assert!(
                inference
                    .type_entries()
                    .iter()
                    .all(|(_, entry)| entry.status == TypeStatus::Known)
            );
            let primary = typed.source_term().unwrap();
            let atomic = typed.source_atomic_formula().unwrap();
            assert_eq!(primary.terms().len(), if symbolic { 8 } else { 6 });
            assert_eq!(
                primary
                    .terms()
                    .iter()
                    .map(|(_, term)| (term.source_range().start, term.source_range().end))
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                if symbolic { 8 } else { 6 }
            );
            assert_eq!(atomic.candidates().len(), 2);
            for (formula, row) in atomic.formulas().iter() {
                use mizar_checker::source_atomic_formula::{
                    SourceAtomicEdgeRole, SourceAtomicFormulaKind, SourceAtomicRequestKind,
                };
                if row.kind() == SourceAtomicFormulaKind::Membership {
                    let requests = atomic
                        .requests()
                        .iter()
                        .filter(|(_, request)| request.formula() == formula)
                        .map(|(_, request)| request)
                        .collect::<Vec<_>>();
                    assert_eq!(requests.len(), 1);
                    assert_eq!(requests[0].ordinal(), 0);
                    assert_eq!(
                        requests[0].kind(),
                        SourceAtomicRequestKind::OperandExpectedType
                    );
                    let edge = atomic.edges().get(requests[0].edge().unwrap()).unwrap();
                    assert_eq!(edge.ordinal(), 1);
                    assert_eq!(edge.role(), SourceAtomicEdgeRole::BuiltinRightOperand);
                }
            }
            assert_eq!(atomic.predicate_segments().len(), 2);
            let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                SourceVariableScopeInput::new(&ast, source.module(), &symbols),
            )
            .unwrap();
            assert_eq!(scope.bindings().len(), 2);
            let owner = symbols
                .symbols()
                .iter()
                .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
                .unwrap();
            let namespace = mizar_resolve::env::NamespacePath::new(source.module().path().as_str());
            let labels = ProofLabelSourceCollector::new(
                &ast,
                source.module(),
                namespace.clone(),
                owner.contribution(),
                &source,
            )
            .unwrap()
            .collect_with_theorem_owners(&symbols)
            .unwrap();
            let resolved = LabelResolver::new(labels.projections()).resolve(
                source.module(),
                &namespace,
                labels.references(),
            );
            let check = |typed: &TypedAst| {
                SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &source, typed, &scope, &symbols, &labels, &resolved, None,
                )
                .map(|_| ())
            };
            assert!(check(&typed).is_ok());
            for (_, entry) in inference.type_entries().iter() {
                let TypeEntryActual::Known(ty) = entry.actual else {
                    panic!("known set type")
                };
                let head = &inference.normalized_types().get(ty).unwrap().head;
                assert!(
                    head == &mizar_checker::type_checker::TypeHeadRef::BuiltinSet
                        || symbolic
                            && head == &mizar_checker::type_checker::TypeHeadRef::BuiltinObject
                );
            }
            let predicate = symbols
                .symbols()
                .iter()
                .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Predicate)
                .unwrap();
            assert!(
                atomic
                    .candidates()
                    .iter()
                    .all(|(_, candidate)| candidate.symbol() == predicate.symbol())
            );
            let assemble = |nodes, primary: &mizar_checker::source_term::SourcePrimaryTermHandoff, atomic: &mizar_checker::source_atomic_formula::SourceAtomicFormulaHandoff| {
            TypedAst::try_new(TypedAstParts {
                source_id: typed.source_id(), module_id: typed.module_id().clone(), resolved_root: Some(source.arena().root()),
                source_context: None, source_type: None, source_attribute: None, nodes,
                contexts: LocalTypeContextTable::new(), types: TypeTable::new(), facts: TypeFactTable::new(),
                coercions: CoercionTable::new(), initial_obligations: InitialObligationTable::new(), diagnostics: TypeDiagnosticTable::new(),
            }).and_then(|ast| ast.with_source_term(primary.clone())).and_then(|ast| ast.with_source_atomic_formula(atomic.clone()))
        };

            let target = atomic
                .predicate_heads()
                .iter()
                .next()
                .unwrap()
                .1
                .site()
                .node();
            for mutation in 0..5 {
                let mut nodes = typed
                    .nodes()
                    .iter()
                    .map(|(_, node)| node.clone())
                    .collect::<Vec<_>>();
                match mutation {
                    0 => nodes[target.index()].kind = "PredicateHead".into(),
                    1 => nodes[target.index()].children.clear(),
                    2 => nodes[target.index()].recovery = NodeRecoveryState::Recovered,
                    3 => nodes[target.index()].anchor = nodes[0].anchor.clone(),
                    4 => nodes[target.index()].resolved_node = Some(source.arena().root()),
                    _ => unreachable!(),
                }
                let changed = TypedArena::try_new(typed.nodes().root(), nodes).unwrap();
                match assemble(changed, primary, atomic) {
                    Ok(changed) => assert!(check(&changed).is_err(), "mutation {mutation}"),
                    Err(error) => assert_eq!(
                        error,
                        TypedAstError::InvalidSourceAtomicFormula,
                        "mutation {mutation}"
                    ),
                }
            }
            if symbolic {
                let quantifiers = typed
                    .nodes()
                    .iter()
                    .filter(|(_, node)| node.kind.as_str() == "QuantifiedFormula(Universal)")
                    .map(|(id, _)| id)
                    .collect::<Vec<_>>();
                let nested = quantifiers[0];
                let segment = typed.nodes().node(nested).unwrap().children[1];
                let declaration = typed.nodes().node(segment).unwrap().children[0];
                let parent = typed
                    .nodes()
                    .iter()
                    .find(|(_, node)| node.children.contains(&nested))
                    .unwrap()
                    .0;
                let membership = atomic.formulas().iter().find(|(_, row)| row.kind() == mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Membership).unwrap().1.site().node();
                for mutation in 0..5 {
                    let mut nodes = typed
                        .nodes()
                        .iter()
                        .map(|(_, node)| node.clone())
                        .collect::<Vec<_>>();
                    match mutation {
                        0 => nodes[nested.index()].children.swap(3, 5),
                        1 => nodes[declaration.index()].anchor = nodes[0].anchor.clone(),
                        2 => nodes[nested.index()].resolved_node = Some(source.arena().root()),
                        3 => {
                            let child = nodes[parent.index()]
                                .children
                                .iter_mut()
                                .find(|id| **id == nested)
                                .unwrap();
                            *child = quantifiers[1];
                        }
                        4 => nodes[membership.index()].children.swap(0, 2),
                        _ => unreachable!(),
                    }
                    let changed = assemble(
                        TypedArena::try_new(typed.nodes().root(), nodes).unwrap(),
                        primary,
                        atomic,
                    )
                    .expect("structurally valid hostile graph reaches replay seal");
                    assert_eq!(
                        check(&changed),
                        Err("theorems.source.invalid".to_owned()),
                        "nested mutation {mutation}"
                    );
                }
            }
            let text = fs::read_to_string(&case.source_path).unwrap();
            // Graft independently valid semantic receipts onto the unchanged original
            // graph: this isolates candidate-symbol and operand-binding authentication.
            let head = if symbolic { "<<=" } else { "likes" };
            let renamed_head = if symbolic { ">>=" } else { "hates" };
            let theorem = if symbolic {
                "SubsetRefl1"
            } else {
                "LikesSelf1"
            };
            let swapped = if symbolic {
                text.replace("z in X holds z in Y", "z in Y holds z in X")
            } else {
                text.replace("means X = Y", "means Y = X")
            };
            for text in [text.replace(head, renamed_head), swapped] {
                let (foreign, nodes, env) = super::super::source_registration_inputs(
                    &config.workspace_root,
                    case,
                    super::super::formula_statement::step5c8_test_frontend(&text),
                )
                .unwrap();
                let (_, _, foreign_typed) =
                    TermFormulaChecker::check_source_predicate_statements(&foreign, &env, &nodes)
                        .unwrap();
                let grafted = assemble(
                    typed.nodes().clone(),
                    foreign_typed.source_term().unwrap(),
                    foreign_typed.source_atomic_formula().unwrap(),
                )
                .unwrap();
                assert!(check(&grafted).is_err());
            }
            for changed in [
                text.replace(head, renamed_head)
                    .replace(theorem, "OtherOwner"),
                text.replace(theorem, "OtherOwner"),
                text.replace("let X be set;", "let Z be set;")
                    .replace(&format!("thus X {head} X"), &format!("thus Z {head} Z"))
                    .replace(theorem, "OtherOwner"),
            ] {
                let frontend = super::super::formula_statement::step5c8_test_frontend(&changed);
                let ast = frontend.ast.clone().unwrap();
                let (foreign, nodes, env) = super::super::source_registration_inputs(
                    &config.workspace_root,
                    case,
                    frontend,
                )
                .unwrap();
                assert!(
                    TermFormulaChecker::check_source_predicate_statements(&source, &env, &neutral)
                        .is_err()
                );
                assert!(
                    TermFormulaChecker::check_source_predicate_statements(
                        &source, &symbols, &nodes
                    )
                    .is_err()
                );
                let (_, _, foreign_typed) =
                    TermFormulaChecker::check_source_predicate_statements(&foreign, &env, &nodes)
                        .unwrap();
                assert!(check(&foreign_typed).is_err());
                let foreign_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
                    SourceVariableScopeInput::new(&ast, foreign.module(), &env),
                )
                .unwrap();
                let owner = env
                    .symbols()
                    .iter()
                    .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
                    .unwrap();
                let foreign_labels = ProofLabelSourceCollector::new(
                    &ast,
                    foreign.module(),
                    namespace.clone(),
                    owner.contribution(),
                    &foreign,
                )
                .unwrap()
                .collect_with_theorem_owners(&env)
                .unwrap();
                let result = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &source,
                    &typed,
                    &foreign_scope,
                    &symbols,
                    &foreign_labels,
                    &resolved,
                    None,
                );
                assert!(result.is_err(), "foreign ownership admitted: {changed}");
            }
        }
    }

    #[test]
    fn step5c5_phrase_metadata_inventory_and_cross_stage_aliases() {
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        for case in plan.cases.iter().filter(|case| {
            [
                STEP5C5_PHRASE_ID,
                STEP5C5_SYMBOLIC_ID,
                STEP5C5_MEANS_ID,
                STEP5C5_EQUALS_ID,
            ]
            .contains(&case.id.0.as_str())
        }) {
            assert!(step5c5_phrase_admitted(Some(&config.workspace_root), case));
            assert!(
                crate::expectation::validate_expectation_path(
                    &case.expectation_path,
                    &case.expectation,
                    &config.workspace_root.join("tests")
                )
                .is_empty()
            );
            for mutation in 0..28 {
                let mut changed = case.clone();
                match mutation {
                    0 => changed.id.0.push('_'),
                    1 => changed.expectation.id.0.push('_'),
                    2 => changed.source_path = changed.source_path.with_file_name("other.miz"),
                    3 => {
                        changed.expectation_path =
                            changed.expectation_path.with_file_name("other.expect.toml")
                    }
                    4 => changed.expectation.source = "other.miz".into(),
                    5 => changed.expectation.domain.push('_'),
                    6 => changed.expectation.expected_phase = Some(PipelinePhase::Verification),
                    7 => changed.expectation.expected_outcome = ExpectedOutcome::Fail,
                    8 => changed.expectation.spec_refs.reverse(),
                    9 => changed.expectation.spec_refs.pop().map(|_| ()).unwrap(),
                    10 => changed
                        .expectation
                        .tags
                        .push(ACTIVE_PROOF_VERIFICATION_TAG.into()),
                    11 => changed.expectation.snapshots = None,
                    12 => changed.expectation.snapshots = Some("snapshots/vc/other.snap".into()),
                    13 => changed.expectation.stable_detail_key = Some("unrelated".into()),
                    14 => changed.expectation.failure_category = Some("proof_failure".into()),
                    15 => changed.expectation.schema_version += 1,
                    16 => changed.expectation.profiles.push("other".into()),
                    17 => changed.expectation.kind = crate::expectation::TestKind::Fail,
                    18 => changed.expectation.rejection_reason = Some("other".into()),
                    19 => changed
                        .expectation
                        .diagnostic_codes
                        .push("E-UNRELATED".into()),
                    20 => changed
                        .expectation
                        .diagnostic_payloads
                        .push("unrelated".into()),
                    21 => changed
                        .expectation
                        .declaration_symbol_payloads
                        .push("unrelated".into()),
                    22 => changed.expectation.ast_profile = Some("unrelated".into()),
                    23 => changed
                        .expectation
                        .snapshot_profiles
                        .push("unrelated".into()),
                    24 => {
                        changed.expectation.tokens = plan
                            .cases
                            .iter()
                            .find(|row| !row.expectation.tokens.is_empty())
                            .unwrap()
                            .expectation
                            .tokens
                            .clone()
                    }
                    25 => {
                        changed.expectation.origin = Some(crate::expectation::OriginMetadata {
                            schema_version: 1,
                            kind: changed.expectation.kind,
                            generator: "test".into(),
                            generator_version: "1".into(),
                            seed: "0".into(),
                            profile: "fast".into(),
                            expected_outcome: changed.expectation.expected_outcome,
                            minimized: false,
                            original_failure_category: None,
                        })
                    }
                    26 => {
                        changed.expectation.architecture22 =
                            Some(crate::expectation::Architecture22Metadata {
                                scenarios: vec!["cache_hit_miss_timing".into()],
                                equivalence_class: Some("observable_outputs_equal".into()),
                                gate: crate::expectation::Architecture22Gate::Planned,
                            })
                    }
                    27 => changed
                        .expectation
                        .spec_refs
                        .push(changed.expectation.spec_refs[0].clone()),
                    _ => unreachable!(),
                }
                assert!(is_step5c5_phrase_candidate(&changed));
                assert!(
                    !step5c5_phrase_admitted(Some(&config.workspace_root), &changed),
                    "mutation {mutation}"
                );
                if !matches!(mutation, 0 | 2 | 11) {
                    assert!(
                        crate::expectation::validate_expectation_path(
                            &changed.expectation_path,
                            &changed.expectation,
                            &config.workspace_root.join("tests")
                        )
                        .iter()
                        .any(|diagnostic| diagnostic.code.0 == "E-EXPECT-SNAPSHOT-SCOPE"),
                        "snapshot metadata mutation {mutation}"
                    );
                }
            }
            for (stage, tag, phase) in [
                (Stage::ParseOnly, "active_parse_only", PipelinePhase::Parse),
                (
                    Stage::DeclarationSymbol,
                    "active_declaration_symbol",
                    PipelinePhase::Resolve,
                ),
                (
                    Stage::TypeElaboration,
                    "active_type_elaboration",
                    PipelinePhase::TypeCheck,
                ),
                (
                    Stage::FormulaStatement,
                    "active_formula_statement",
                    PipelinePhase::StatementCheck,
                ),
                (
                    Stage::AdvancedSemantics,
                    "active_advanced_semantics",
                    PipelinePhase::ClusterResolution,
                ),
            ] {
                for alias in 0..4 {
                    let mut changed = case.clone();
                    changed.expectation.stage = stage;
                    changed.expectation.tags = vec![tag.into()];
                    changed.expectation.expected_phase = Some(phase);
                    if alias > 0 {
                        changed.id.0 = "alias".into();
                    }
                    if alias > 1 {
                        changed.expectation.id.0 = "alias".into();
                    }
                    if alias > 2 {
                        changed.source_path = changed.source_path.with_file_name("alias.miz");
                    }
                    assert!(is_step5c5_phrase_candidate(&changed));
                    assert!(!is_active_proof_verification(&changed));
                    assert!(!super::super::step5c11_registration_admitted(
                        &config.workspace_root,
                        &changed
                    ));
                    assert!(!super::super::step5c13_overload_admitted(
                        &config.workspace_root,
                        &changed
                    ));
                    assert!(!super::super::is_active_parse_only(&changed));
                    assert!(!super::super::is_active_declaration_symbol(&changed));
                    assert!(!super::super::is_active_type_elaboration(&changed));
                    assert!(
                        !super::super::formula_statement::is_active_formula_statement(
                            &config.workspace_root,
                            &changed
                        )
                    );
                }
            }
            for duplicate in [false, true] {
                let mut changed = plan.clone();
                if duplicate {
                    changed.cases.push(case.clone());
                } else {
                    changed.cases.retain(|row| row.id != case.id);
                }
                assert!(
                    validate_active_proof_verification_tags(&config.workspace_root, &changed)
                        .iter()
                        .any(|diagnostic| diagnostic.code.0
                            == "E-PROOF-VERIFICATION-STEP5C5-INVENTORY")
                );
            }
        }
    }

    #[test]
    fn step5c5_phrase_downstream_structural_controls_and_baselines() {
        use mizar_core::core_ir::*;
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        for case in plan
            .cases
            .iter()
            .filter(|case| [STEP5C5_PHRASE_ID, STEP5C5_SYMBOLIC_ID].contains(&case.id.0.as_str()))
        {
            let (core, vcs) = phrase_theorem_output(
                &config.workspace_root,
                case,
                run_frontend(&config.workspace_root, case, 0).unwrap(),
            )
            .unwrap();
            let (seed_id, seed) = core.obligation_seeds().iter().next().unwrap();
            let (proof_id, proof) = core.proofs().iter().next().unwrap();
            let (definition_id, definition) = core.definitions().iter().next().unwrap();
            assert!(
                seed.core_refs
                    .contains(&CoreNodeRef::Definition(definition_id))
            );
            assert!(
                seed.core_refs
                    .contains(&CoreNodeRef::Item(definition.owner.anchor_item()))
            );
            assert!(
                core.terms()
                    .iter()
                    .all(|(id, _)| seed.core_refs.contains(&CoreNodeRef::Term(id)))
            );
            assert!(
                core.formulas()
                    .iter()
                    .all(|(id, _)| seed.core_refs.contains(&CoreNodeRef::Formula(id)))
            );
            for mutation in 0..11 {
                let mut parts = CoreIrParts {
                    source_id: core.source_id(),
                    module_id: core.module_id().clone(),
                    items: core.items().clone(),
                    terms: core.terms().clone(),
                    formulas: core.formulas().clone(),
                    definitions: core.definitions().clone(),
                    proofs: core.proofs().clone(),
                    proof_nodes: core.proof_nodes().clone(),
                    algorithms: core.algorithms().clone(),
                    algorithm_statements: core.algorithm_statements().clone(),
                    generated: core.generated().clone(),
                    obligation_seeds: core.obligation_seeds().clone(),
                    source_map: core.source_map().clone(),
                    diagnostics: core.diagnostics().clone(),
                };
                let changed = parts.obligation_seeds.get_mut(seed_id).unwrap();
                match mutation {
                    0 => {
                        changed.context.remove(0);
                    }
                    1 => changed.context.push(seed.goal.unwrap()),
                    2 => changed.context.reverse(),
                    3 => changed.goal = Some(proof.proposition),
                    4 => changed.local_path = "".into(),
                    5 => changed.status = ObligationSeedStatus::Skipped,
                    6 => changed.owner = definition.owner.anchor_item(),
                    7 => changed.core_refs.clear(),
                    8 => parts.proofs.get_mut(proof_id).unwrap().status = CoreProofStatus::Open,
                    9 => {
                        parts.definitions.get_mut(definition_id).unwrap().owner =
                            CoreDefinitionOwner::for_item(proof.item)
                    }
                    10 => {
                        let CoreProofNodeKind::IntroduceBinder { binder, .. } =
                            &mut parts.proof_nodes.get_mut(proof.root).unwrap().kind
                        else {
                            panic!()
                        };
                        binder.var = definition.params[0].var;
                    }
                    _ => unreachable!(),
                }
                // Core-only fields such as Pending status need complete Core comparison:
                // generic seed intake does not authenticate a detached graph as source.
                let changed = CoreIr::try_new(parts)
                    .unwrap_or_else(|error| panic!("Core mutation {mutation}: {error}"));
                let actual = generate_core_vcs(&changed, snapshot_id(0))
                    .unwrap_or_else(|error| panic!("VC mutation {mutation}: {error}"));
                assert_ne!(
                    (changed, actual),
                    (core.clone(), vcs.clone()),
                    "mutation {mutation}"
                );
            }
            let temporary = std::process::Command::new("mktemp")
                .arg("-d")
                .output()
                .unwrap();
            assert!(temporary.status.success());
            let root =
                std::path::PathBuf::from(String::from_utf8(temporary.stdout).unwrap().trim());
            let path = root.join(case.expectation.snapshots.as_ref().unwrap());
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            for contents in [None, Some("corrupt VC baseline")] {
                if let Some(contents) = contents {
                    fs::write(&path, contents).unwrap();
                }
                let run = run_proof_verification_case(&config.workspace_root, &root, case, 0);
                assert_eq!(run.status, ProofVerificationCaseStatus::Failed);
                assert!(run.failure.unwrap().contains("snapshot"));
            }
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn step5c10_mapped_sources_reach_frozen_phases() {
        use mizar_proof::policy::CandidatePolicyClass;
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        for (ordinal, (id, _, key)) in super::super::formula_statement::STEP5C10_CASES
            .into_iter()
            .enumerate()
        {
            let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
            let output = run_frontend(&config.workspace_root, case, ordinal).unwrap();
            assert!(
                output.diagnostics.is_empty(),
                "{id}: {:?}",
                output.diagnostics
            );
            let ast = output.ast.unwrap();
            let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
            assert!(
                resolver.detail_keys.is_empty(),
                "{id}: {:?}",
                resolver.detail_keys
            );
            let phase = case.expectation.expected_phase.unwrap();
            let result = theorem_ast_output(
                &ast,
                &resolver.module,
                &resolver.env,
                phase,
                snapshot_id(ordinal),
                None,
            );
            if let Some(key) = key {
                assert_eq!(result.unwrap_err(), key, "{id}");
                continue;
            }
            let (classes, vcs) = result.unwrap_or_else(|error| panic!("{id}: {error}"));
            match phase {
                PipelinePhase::StatementCheck => {
                    assert_eq!(
                        classes,
                        [
                            CandidatePolicyClass::OpenAllowed,
                            CandidatePolicyClass::AssumedByPolicy
                        ]
                    );
                    assert!(vcs.is_none());
                }
                PipelinePhase::VcGeneration => {
                    assert!(classes.is_empty());
                    let vcs = vcs.unwrap();
                    assert_eq!(vcs.source(), ast.source_id);
                    assert_eq!(vcs.snapshot(), snapshot_id(ordinal));
                    assert_eq!(vcs.vcs().len(), 2);
                    let mut context_sizes = Vec::new();
                    for vc in vcs.vcs() {
                        assert_eq!(vc.status, mizar_vc::vc_ir::VcStatus::Open);
                        assert!(
                            vc.local_context.entries().iter().all(
                                |entry| entry.formula.is_some() && !entry.provenance.is_empty()
                            )
                        );
                        assert!(!vc.provenance.is_empty());
                        context_sizes.push(vc.local_context.entries().len());
                    }
                    context_sizes.sort();
                    assert_eq!(context_sizes, [1, 2]);
                    assert_eq!(
                        theorem_ast_output(
                            &ast,
                            &resolver.module,
                            &resolver.env,
                            phase,
                            snapshot_id(ordinal),
                            None,
                        )
                        .unwrap()
                        .1,
                        Some(vcs)
                    );
                }
                _ => panic!("unexpected success phase"),
            }
            let run = run_proof_verification_case(
                &config.workspace_root,
                &config.workspace_root.join("tests"),
                case,
                ordinal,
            );
            assert_eq!(
                run.status,
                super::super::ProofVerificationCaseStatus::Passed,
                "{:?}",
                run.failure
            );
        }
    }

    fn check_theorem_source(
        source: &str,
        phase: PipelinePhase,
    ) -> Result<
        (
            Vec<mizar_proof::policy::CandidatePolicyClass>,
            Option<VcSet>,
        ),
        String,
    > {
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::super::formula_statement::STEP5C10_CASES[0].0)
            .unwrap();
        let ast = parse(source);
        let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            return Err(format!("resolver: {:?}", resolver.detail_keys));
        }
        theorem_ast_output(
            &ast,
            &resolver.module,
            &resolver.env,
            phase,
            snapshot_id(777),
            None,
        )
    }

    fn with_step5c14_theorem_check<T>(
        text: &str,
        consume: impl FnOnce(&mizar_checker::type_checker::SourceTheoremCheck<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        use mizar_resolve::resolved_ast::SurfaceResolvedArena;
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::super::formula_statement::STEP5C10_CASES[0].0)
            .unwrap();
        let ast = parse(text);
        let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            return Err(format!("{:?}", resolver.detail_keys));
        }
        let source =
            SurfaceResolvedArena::lower(&ast, &resolver.module).map_err(|e| e.to_string())?;
        with_source_theorem_check(&ast, &source, &resolver.env, consume)
    }

    fn with_source_theorem_check<T>(
        ast: &mizar_syntax::SurfaceAst,
        source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
        symbols: &mizar_resolve::env::SymbolEnv,
        consume: impl FnOnce(&mizar_checker::type_checker::SourceTheoremCheck<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        use mizar_checker::type_checker::SourceVariableSemanticsChecker;
        use mizar_resolve::{
            labels::{LabelResolver, ProofLabelSourceCollector},
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        };
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(ast, source.module(), symbols),
        )
        .map_err(|e| format!("{e:?}"))?;
        let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        let typed = super::super::type_elaboration::step5c8_formula_typed_ast(
            ast,
            source.module(),
            symbols,
            &scope,
            &bindings,
            true,
        )?;
        let owner = symbols
            .symbols()
            .iter()
            .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
            .or_else(|| {
                symbols
                    .symbols()
                    .iter()
                    .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Lemma)
            })
            .ok_or("no theorem")?;
        let namespace = mizar_resolve::env::NamespacePath::new(source.module().path().as_str());
        let labels = ProofLabelSourceCollector::new(
            ast,
            source.module(),
            namespace.clone(),
            owner.contribution(),
            source,
        )
        .and_then(|collector| collector.collect_with_theorem_owners(symbols))
        .map_err(|e| e.to_string())?;
        let resolved = LabelResolver::new(labels.projections()).resolve(
            source.module(),
            &namespace,
            labels.references(),
        );
        let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
            source, &typed, &scope, symbols, &labels, &resolved, None,
        )?;
        consume(&check)
    }

    fn step5c14_computation_core(text: &str) -> Result<mizar_core::core_ir::CoreIr, String> {
        with_step5c14_theorem_check(text, |check| {
            mizar_core::elaborator::lower_source_theorem_skeletons(check, None)
        })
    }

    #[test]
    fn step5c14_computation_rejects_stale_and_mutated_receipts() -> Result<(), String> {
        use mizar_checker::{type_checker::SourceVariableSemanticsChecker, typed_ast::*};
        use mizar_resolve::{
            labels::{LabelResolver, ProofLabelSourceCollector},
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
            resolved_ast::SurfaceResolvedArena,
        };
        let text = "theorem T: 0 = 0 by computation(steps: 8);";
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::super::formula_statement::STEP5C10_CASES[0].0)
            .unwrap();
        let ast = parse(text);
        let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            return Err(format!("{:?}", resolver.detail_keys));
        }
        let source =
            SurfaceResolvedArena::lower(&ast, &resolver.module).map_err(|e| e.to_string())?;
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&ast, &resolver.module, &resolver.env),
        )
        .map_err(|e| format!("{e:?}"))?;
        let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        let typed = super::super::type_elaboration::step5c8_formula_typed_ast(
            &ast,
            &resolver.module,
            &resolver.env,
            &scope,
            &bindings,
            true,
        )?;
        let owner = resolver
            .env
            .symbols()
            .iter()
            .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
            .ok_or("no theorem")?;
        let namespace = mizar_resolve::env::NamespacePath::new(resolver.module.path().as_str());
        let labels = ProofLabelSourceCollector::new(
            &ast,
            &resolver.module,
            namespace.clone(),
            owner.contribution(),
            &source,
        )
        .and_then(|collector| collector.collect_with_theorem_owners(&resolver.env))
        .map_err(|e| e.to_string())?;
        let resolved = LabelResolver::new(labels.projections()).resolve(
            &resolver.module,
            &namespace,
            labels.references(),
        );

        for changed in [
            text.replace("steps: 8", "steps: 9"),
            text.replace("T:", "U:"),
            text.replace("0 = 0", "0 = 1"),
        ] {
            let changed_ast = parse(&changed);
            let changed_source =
                SurfaceResolvedArena::lower(&changed_ast, &resolver.module).unwrap();
            assert!(
                SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &changed_source,
                    &typed,
                    &scope,
                    &resolver.env,
                    &labels,
                    &resolved,
                    None
                )
                .is_err()
            );
        }
        let option = typed
            .nodes()
            .iter()
            .find_map(|(id, node)| (node.kind.as_str() == "ComputationOption").then_some(id))
            .unwrap();
        for mutation in 0..5 {
            let mut nodes = typed
                .nodes()
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            match mutation {
                0 => nodes[option.index()].children.clear(),
                1 => nodes[option.index()].recovery = NodeRecoveryState::Recovered,
                2 => nodes[option.index()].kind = "TermReference".into(),
                3 => {
                    nodes[option.index()].anchor =
                        mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
                            source_id: typed.source_id(),
                            start: 0,
                            end: 1,
                        })
                }
                4 => nodes[option.index()].resolved_node = Some(source.arena().root()),
                _ => unreachable!(),
            }
            let nodes = TypedArena::try_new(typed.nodes().root(), nodes).unwrap();
            let changed = {
                TypedAst::try_new(TypedAstParts {
                    source_id: typed.source_id(),
                    module_id: typed.module_id().clone(),
                    resolved_root: None,
                    source_context: None,
                    source_type: None,
                    source_attribute: None,
                    nodes,
                    contexts: LocalTypeContextTable::new(),
                    types: TypeTable::new(),
                    facts: TypeFactTable::new(),
                    coercions: CoercionTable::new(),
                    initial_obligations: InitialObligationTable::new(),
                    diagnostics: TypeDiagnosticTable::new(),
                })
            }
            .and_then(|typed_ast| typed_ast.with_source_term(typed.source_term().unwrap().clone()))
            .and_then(|typed_ast| {
                typed_ast.with_source_atomic_formula(typed.source_atomic_formula().unwrap().clone())
            })
            .unwrap_or_else(|error| {
                panic!("mutation {mutation} must construct a typed receipt: {error}")
            });
            {
                assert!(
                    SourceVariableSemanticsChecker::check_theorem_skeletons(
                        &source,
                        &changed,
                        &scope,
                        &resolver.env,
                        &labels,
                        &resolved,
                        None
                    )
                    .is_err(),
                    "mutation {mutation}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn step5c14_computation_preserves_source_request_without_execution() {
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == "pass_proof_verification_computation_justification_001")
            .unwrap();
        let frontend = run_frontend(&config.workspace_root, case, 0).unwrap();
        let ast = frontend.ast.clone().unwrap();
        let (source, _, symbols) =
            super::super::source_registration_inputs(&config.workspace_root, case, frontend)
                .unwrap();
        let vcs = theorem_ast_output(
            &ast,
            source.module(),
            &symbols,
            PipelinePhase::VcGeneration,
            snapshot_id(0),
            None,
        )
        .unwrap()
        .1
        .unwrap();
        assert_eq!(fs::read_to_string(config.workspace_root.join("tests/snapshots/vc/pass_proof_verification_computation_justification_001.vc_ir.snap")).unwrap(), vcs.debug_text());
        assert_eq!(
            vcs,
            theorem_ast_output(
                &ast,
                source.module(),
                &symbols,
                PipelinePhase::VcGeneration,
                snapshot_id(0),
                None
            )
            .unwrap()
            .1
            .unwrap()
        );
        use mizar_core::{binder_normalization::*, control_flow::*, core_ir::*};
        let original = "theorem Comp1: 0 = 0 by computation(steps: 8);";
        let baseline = step5c14_computation_core(original).unwrap();
        for digits in ["8", "9", "0", "0008", "18446744073709551616000000000000000"] {
            let source = original.replace("steps: 8", &format!("steps: {digits}"));
            let core = step5c14_computation_core(&source).unwrap();
            assert_eq!(core, step5c14_computation_core(&source).unwrap());
            assert_eq!(
                core.debug_text(),
                step5c14_computation_core(&source).unwrap().debug_text()
            );
            if digits != "8" {
                assert_ne!(core, baseline);
            }
            let (_, proof) = core.proofs().iter().next().unwrap();
            assert_eq!(proof.status, CoreProofStatus::PendingAutomaticProof);
            let node = core.proof_nodes().get(proof.root).unwrap();
            let CoreProofNodeKind::ComputationGoal { obligation, steps } = &node.kind else {
                panic!("request missing");
            };
            assert_eq!(steps, digits);
            let seed = core.obligation_seeds().get(*obligation).unwrap();
            assert_eq!(seed.status, ObligationSeedStatus::Active);
            assert!(seed.context.is_empty());
            let CoreSourceAnchor::SourceRange(request) = node.source.anchor else {
                panic!();
            };
            assert_eq!(
                &source[request.start..request.end],
                format!("by computation(steps: {digits})")
            );
            let formula = core.formulas().get(proof.proposition).unwrap();
            let CoreFormulaKind::Equals { left, right } = formula.kind else {
                panic!();
            };
            assert_ne!(left, right);
            for term in [left, right] {
                assert_eq!(
                    core.terms().get(term).unwrap().kind,
                    CoreTermKind::Numeral("0".into())
                );
                assert!(
                    normalize_core_term(&core, term, &BinderContext::new(), &formula.source)
                        .is_err()
                );
            }
            assert_ne!(
                core.terms().get(left).unwrap().source,
                core.terms().get(right).unwrap().source
            );
            assert!(
                normalize_core_formula(
                    &core,
                    proof.proposition,
                    &BinderContext::new(),
                    &formula.source
                )
                .is_err()
            );
            let flow = build_control_flow_ir(&core);
            let handoff = build_obligation_seed_handoff(&core, &flow);
            assert_eq!(
                handoff.entries.iter().next().unwrap().1.seed.status,
                ObligationSeedStatus::Deferred
            );
            assert!(
                generate_core_vcs(&core, snapshot_id(777))
                    .unwrap()
                    .vcs()
                    .is_empty()
            );
            let vcs = check_theorem_source(&source, PipelinePhase::VcGeneration)
                .unwrap()
                .1
                .unwrap();
            assert_eq!(
                vcs,
                check_theorem_source(&source, PipelinePhase::VcGeneration)
                    .unwrap()
                    .1
                    .unwrap()
            );
            let [vc] = vcs.vcs() else {
                panic!("not one VC");
            };
            assert_eq!(vc.status, mizar_vc::vc_ir::VcStatus::Open);
            assert_eq!(
                vc.goal,
                mizar_vc::vc_ir::VcFormulaRef::Core(proof.proposition)
            );
            assert_eq!(
                vc.proof_hint.as_ref().unwrap().computation,
                Some(mizar_vc::vc_ir::ComputationHint::SymbolicRequest(
                    mizar_vc::vc_ir::ProofHintKey::new(format!("by-computation(steps:{digits})"))
                ))
            );
            assert_eq!(vc.source.primary, node.source);
            assert_eq!(vc.source.related.len(), 4);
            assert!(vcs.canonical_vc_fingerprint(vc.id).is_none());
            let slices = mizar_vc::dependency_slice::try_compute_dependency_slices(
                mizar_vc::dependency_slice::DependencySliceInput {
                    vc_set: &vcs,
                    discharge_output: None,
                },
            )
            .unwrap();
            let slice = slices.slice_for(vc.id).unwrap();
            assert!(slice.requires_cache_miss());
            assert!(slice.unknowns().iter().any(|unknown| unknown.family()
                == mizar_vc::dependency_slice::DependencyUnknownFamily::Computation));
            assert!(format!("{vcs:?}").contains(&format!("by-computation(steps:{digits})")));
        }
        let renamed = step5c14_computation_core(&original.replace("Comp1", "Other")).unwrap();
        assert_ne!(baseline.items(), renamed.items());
        assert_ne!(baseline.debug_text(), renamed.debug_text());
    }

    #[test]
    fn step5c14_computation_core_mutations_fail_closed() {
        use mizar_core::core_ir::*;
        let core = step5c14_computation_core("theorem T: 0 = 0 by computation(steps: 8);").unwrap();
        let (proof_id, proof) = core.proofs().iter().next().unwrap();
        let formula = proof.proposition;
        let node = proof.root;
        let CoreProofNodeKind::ComputationGoal { obligation, .. } =
            core.proof_nodes().get(node).unwrap().kind
        else {
            panic!();
        };
        let CoreFormulaKind::Equals { left, right } = core.formulas().get(formula).unwrap().kind
        else {
            panic!();
        };
        let generate = |core: &CoreIr| {
            mizar_vc::generator::generate_source_computation_request(
                core,
                snapshot_id(777),
                &GenerationSchemaVersion::new("test"),
                &VcSchemaVersion::new("test"),
            )
        };
        for mutation in 0..20 {
            let mut parts = CoreIrParts {
                source_id: core.source_id(),
                module_id: core.module_id().clone(),
                items: core.items().clone(),
                terms: core.terms().clone(),
                formulas: core.formulas().clone(),
                definitions: core.definitions().clone(),
                proofs: core.proofs().clone(),
                proof_nodes: core.proof_nodes().clone(),
                algorithms: core.algorithms().clone(),
                algorithm_statements: core.algorithm_statements().clone(),
                generated: core.generated().clone(),
                obligation_seeds: core.obligation_seeds().clone(),
                source_map: core.source_map().clone(),
                diagnostics: core.diagnostics().clone(),
            };
            match mutation {
                0 => {
                    parts.proof_nodes.get_mut(node).unwrap().kind =
                        CoreProofNodeKind::TerminalGoal {
                            obligation,
                            citations: Vec::new(),
                        }
                }
                1 => {
                    parts.formulas.get_mut(formula).unwrap().kind =
                        CoreFormulaKind::Equals { left, right: left }
                }
                2 => parts.terms.get_mut(right).unwrap().kind = CoreTermKind::Numeral("1".into()),
                3 => parts
                    .obligation_seeds
                    .get_mut(obligation)
                    .unwrap()
                    .context
                    .push(formula),
                4 => {
                    parts.obligation_seeds.get_mut(obligation).unwrap().label =
                        Some(CoreLabelRef::new("fake"))
                }
                5 => parts
                    .items
                    .get_mut(proof.item)
                    .unwrap()
                    .dependencies
                    .push(proof.item),
                6 => parts.proofs.get_mut(proof_id).unwrap().status = CoreProofStatus::Open,
                7 => parts.items.get_mut(proof.item).unwrap().kind = CoreItemKind::Lemma,
                8 => parts
                    .obligation_seeds
                    .get_mut(obligation)
                    .unwrap()
                    .core_refs
                    .push(CoreNodeRef::Term(left)),
                9 => parts
                    .obligation_seeds
                    .get_mut(obligation)
                    .unwrap()
                    .core_refs
                    .retain(|reference| *reference != CoreNodeRef::Proof(proof_id)),
                10 => {
                    parts
                        .obligation_seeds
                        .get_mut(obligation)
                        .unwrap()
                        .semantic_origin = NormalizedSemanticOrigin::new("fake")
                }
                11 => {
                    parts
                        .obligation_seeds
                        .get_mut(obligation)
                        .unwrap()
                        .local_path = LocalProofOrProgramPath::new("fake")
                }
                12 => {
                    parts.terms.get_mut(left).unwrap().source =
                        parts.terms.get(right).unwrap().source.clone();
                    parts
                        .source_map
                        .term_sources
                        .insert(left, parts.terms.get(left).unwrap().source.clone());
                }
                13 => {
                    parts
                        .formulas
                        .get_mut(formula)
                        .unwrap()
                        .source
                        .provenance
                        .clear();
                    parts
                        .source_map
                        .formula_sources
                        .insert(formula, parts.formulas.get(formula).unwrap().source.clone());
                }
                14 => {
                    let term = parts.terms.get(left).unwrap().clone();
                    let id = parts.terms.insert(term.clone());
                    parts.source_map.term_sources.insert(id, term.source);
                }
                15 => parts
                    .obligation_seeds
                    .get_mut(obligation)
                    .unwrap()
                    .provenance
                    .clear(),
                16 => parts
                    .proofs
                    .get_mut(proof_id)
                    .unwrap()
                    .source
                    .provenance
                    .clear(),
                17 => {
                    parts.proof_nodes.get_mut(node).unwrap().kind = CoreProofNodeKind::Sequence {
                        children: Vec::new(),
                    }
                }
                18 => parts.obligation_seeds.get_mut(obligation).unwrap().goal = None,
                19 => {
                    parts.proof_nodes.get_mut(node).unwrap().kind =
                        CoreProofNodeKind::ComputationGoal {
                            obligation,
                            steps: "9".into(),
                        }
                }
                _ => unreachable!(),
            }
            match CoreIr::try_new(parts) {
                Err(_) => assert_ne!(mutation, 19),
                Ok(mutated) if mutation == 19 => {
                    let vcs = generate(&mutated).unwrap();
                    assert_ne!(vcs, generate(&core).unwrap());
                    assert!(vcs.debug_text().contains("by-computation(steps:9)"));
                }
                Ok(mutated) => assert!(generate(&mutated).is_err(), "accepted mutation {mutation}"),
            }
        }
    }

    #[test]
    fn step5c14_computation_rejects_unsupported_source_profiles() {
        for source in [
            "theorem T: 1 = 1 by computation(steps: 8);",
            "theorem T: 0 = 1 by computation(steps: 8);",
            "theorem T: 0 = 0 by computation;",
            "theorem T: 0 = 0 by computation();",
            "theorem T: 0 = 0 by computation(timeout: 8);",
            "theorem T: 0 = 0 by computation(steps: 8, steps: 9);",
            "theorem T: 0 = 0 by computation(steps: 8, nest: 1);",
            "theorem T: 0 = 0 by computation(steps 8);",
            "theorem T: 0 = 0 by (steps: 8);",
            "theorem T: 0 = 0 proof thus 0 = 0 by computation(steps: 8); end;",
            "open theorem T: 0 = 0 by computation(steps: 8);",
            "theorem T: 0 = 0 by computation(steps: 8); theorem U: 0 = 0 by computation(steps: 8);",
        ] {
            assert!(
                step5c14_computation_core(source).is_err(),
                "accepted {source}"
            );
        }
    }

    #[test]
    fn step5c10_skeleton_errors_require_the_actual_thesis_and_repair() {
        let valid = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        assert!(check_theorem_source(valid, PipelinePhase::VcGeneration).is_ok());
        assert!(
            check_theorem_source(
                &valid.replace(
                    "proof let X be set; thus X = X;",
                    "proof let Y be set; thus Y = Y;"
                ),
                PipelinePhase::VcGeneration
            )
            .is_ok()
        );
        for (source, key) in [
            (
                valid.replace("thus X = X;", "assume A: X = X; thus X = X;"),
                "theorems.skeleton.assumption_without_antecedent",
            ),
            (
                valid.replace("thus X = X;", "thus not X = X;"),
                "theorems.skeleton.conclusion_mismatch",
            ),
            (
                valid.replace("thus X = X;", ""),
                "theorems.skeleton.incomplete_proof",
            ),
        ] {
            assert_eq!(
                check_theorem_source(&source, PipelinePhase::StatementCheck).unwrap_err(),
                key
            );
        }
        for source in [
            valid.replace("let X be set", "let X be object"),
            valid.replace("thus", "hence"),
            valid.replace("thus X = X;", "thus X = X; thus X = X;"),
            valid.replace("for X being set holds", "for X being set st X = X holds"),
            valid.replace("thus X = X", "thus Y = Y"),
        ] {
            assert!(
                check_theorem_source(&source, PipelinePhase::StatementCheck).is_err(),
                "{source}"
            );
        }
    }

    #[test]
    fn step5c10_citations_are_prior_clean_owners_not_names() {
        let source = include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        );
        let renamed = source
            .replace("Lem1", "Earlier")
            .replace("UseLem1", "Later")
            .replace("X", "Bound");
        assert_eq!(
            check_theorem_source(&renamed, PipelinePhase::VcGeneration)
                .unwrap()
                .1
                .unwrap()
                .vcs()
                .len(),
            2
        );
        for source in [
            source.replace("by Lem1", "by Missing"),
            source.replace("by Lem1", "by UseLem1"),
            source.replace("thus X = X;", "thus X = X by UseLem1;"),
        ] {
            assert_eq!(
                check_theorem_source(&source, PipelinePhase::Resolve).unwrap_err(),
                "theorems.reference.unknown_label",
                "{source}"
            );
        }
        for source in [
            source.replace("by Lem1", "by Lem1, Lem1"),
            source.replace("lemma Lem1", "assumed lemma Lem1"),
            source
                .replacen("being set", "being object", 1)
                .replacen("be set", "be object", 1),
        ] {
            assert!(
                check_theorem_source(&source, PipelinePhase::VcGeneration).is_err(),
                "{source}"
            );
        }
    }

    #[test]
    fn step5c10_status_projection_never_supplies_a_clean_dependency() {
        use mizar_proof::policy::{
            CandidatePolicyClass, PolicyCandidate, ProofPolicyEvaluator, VerifierPolicy,
        };
        let source = "open theorem O: for X being set holds X = X; assumed theorem A: for X being set holds X = X;";
        let (classes, vcs) = check_theorem_source(source, PipelinePhase::StatementCheck).unwrap();
        assert_eq!(
            classes,
            [
                CandidatePolicyClass::OpenAllowed,
                CandidatePolicyClass::AssumedByPolicy
            ]
        );
        assert!(vcs.is_none());
        assert!(check_theorem_source(source, PipelinePhase::VcGeneration).is_err());
        for status in ["open", "assumed"] {
            let dependency = format!(
                "{status} theorem A: for X being set holds X = X; theorem T: for X being set holds X = X proof let X be set; thus X = X by A; end;"
            );
            assert!(check_theorem_source(&dependency, PipelinePhase::StatementCheck).is_err());
            let justified = format!(
                "{status} theorem T: for X being set holds X = X proof let X be set; thus X = X; end;"
            );
            assert!(check_theorem_source(&justified, PipelinePhase::StatementCheck).is_err());
        }
        for policy in [VerifierPolicy::development(), VerifierPolicy::release()] {
            let evaluator = ProofPolicyEvaluator::new(policy);
            for candidate in [
                PolicyCandidate::OpenObligation,
                PolicyCandidate::PolicyAssumption,
            ] {
                let decision = evaluator.evaluate_candidate(&candidate);
                assert!(!decision.can_schedule_kernel_check);
                assert!(decision.kernel_evidence_check_kind.is_none());
                assert!(!matches!(
                    decision.class,
                    CandidatePolicyClass::KernelVerified
                        | CandidatePolicyClass::DischargedBuiltin
                        | CandidatePolicyClass::KernelCheckable
                ));
            }
        }
    }

    #[test]
    fn step5c10_theorem_receipts_reject_substituted_resolution() {
        use mizar_checker::type_checker::SourceVariableSemanticsChecker;
        use mizar_resolve::{
            env::NamespacePath,
            labels::{
                LabelProjection, LabelProjectionData, LabelResolver, ProofLabelSourceCollector,
            },
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
            resolved_ast::SurfaceResolvedArena,
        };
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::super::formula_statement::STEP5C10_CASES[0].0)
            .unwrap();
        let source = fs::read_to_string(&case.source_path)
            .unwrap()
            .replace("by Lem1", "by Missing");
        let ast = parse(&source);
        let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
        let scope = SourceVariableScopeResolver::resolve_proof_occurrences(
            SourceVariableScopeInput::new(&ast, &resolver.module, &resolver.env),
        )
        .unwrap();
        let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
        let typed = super::super::type_elaboration::step5c8_formula_typed_ast(
            &ast,
            &resolver.module,
            &resolver.env,
            &scope,
            &bindings,
            true,
        )
        .unwrap();
        let arena = SurfaceResolvedArena::lower(&ast, &resolver.module).unwrap();
        let namespace = NamespacePath::new(resolver.module.path().as_str());
        let labels = ProofLabelSourceCollector::new(
            &ast,
            &resolver.module,
            namespace.clone(),
            resolver.env.symbols().iter().next().unwrap().contribution(),
            &arena,
        )
        .unwrap()
        .collect_with_theorem_owners(&resolver.env)
        .unwrap();
        let genuine = LabelResolver::new(labels.projections()).resolve(
            &resolver.module,
            &namespace,
            labels.references(),
        );
        assert_eq!(
            SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena,
                &typed,
                &scope,
                &resolver.env,
                &labels,
                &genuine,
                None,
            )
            .unwrap_err(),
            "theorems.reference.unknown_label"
        );
        let owner = labels
            .projections()
            .iter()
            .find(|label| label.primary_spelling() == "Lem1")
            .unwrap();
        let renamed = LabelProjection::current_module(
            LabelProjectionData {
                origin_path: owner.origin_path().clone(),
                module: owner.module().clone(),
                namespace: owner.namespace().clone(),
                primary_spelling: "Missing".into(),
                kind: owner.kind(),
                declaration_range: owner.declaration_range(),
                origin: owner.origin().clone(),
                contribution: owner.contribution(),
            },
            match owner.source() {
                mizar_resolve::labels::LabelProjectionSource::CurrentModule {
                    visible_after_ordinal,
                    ..
                } => *visible_after_ordinal,
                _ => panic!("expected local theorem label"),
            },
        );
        let substituted = LabelResolver::new(&[renamed]).resolve(
            &resolver.module,
            &namespace,
            labels.references(),
        );
        assert!(!substituted.has_unresolved());
        assert_eq!(
            SourceVariableSemanticsChecker::check_theorem_skeletons(
                &arena,
                &typed,
                &scope,
                &resolver.env,
                &labels,
                &substituted,
                None,
            )
            .unwrap_err(),
            "theorems.source.invalid"
        );
    }

    #[test]
    fn step5c10_theorem_receipts_reject_foreign_source_identity() {
        use mizar_session::SessionIdAllocator;
        let config = config();
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == super::super::formula_statement::STEP5C10_CASES[0].0)
            .unwrap();
        let mut ast = parse(&fs::read_to_string(&case.source_path).unwrap());
        let resolver = resolver_symbol_collection(&config.workspace_root, case, &ast);
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(777)).unwrap();
        ast.source_id = ids.next_source_id(snapshot_id(777)).unwrap();
        assert!(
            theorem_ast_output(
                &ast,
                &resolver.module,
                &resolver.env,
                PipelinePhase::VcGeneration,
                snapshot_id(777),
                None,
            )
            .is_err()
        );
    }

    #[test]
    fn step5c7_proof_corpus() {
        let config = config();
        let root = &config.workspace_root;
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let cases = plan
            .cases
            .iter()
            .filter(|case| step5c7_proof_candidate(case))
            .collect::<Vec<_>>();
        assert_eq!(cases.len(), 2);
        for case in cases {
            let result = run_proof_verification_case(root, &root.join("tests"), case, 1);
            assert_eq!(
                result.status,
                ProofVerificationCaseStatus::Passed,
                "{:?}",
                result.failure
            );
        }
    }

    #[test]
    fn step5c7_membership_proof_mutations_are_not_credit() {
        let config = config();
        let root = &config.workspace_root;
        let plan = crate::harness::build_test_plan(&config).unwrap();
        for (id, path) in STEP5C7_PROOF_CASES {
            let case = plan.cases.iter().find(|case| case.id.0 == id).unwrap();
            let source = fs::read_to_string(root.join(path)).unwrap();
            let check = |text: &str| {
                let ast = parse(text);
                let symbols = resolver_symbol_collection(root, case, &ast);
                normalize_term_ast(&ast, &symbols.module, &symbols.env)
            };
            // Renaming is semantically irrelevant, including proof-local alpha names.
            let renamed = source
                .replace("A1", "Premise")
                .replace("CompMember1", "Guarded")
                .replace("EnumMember1", "Listed");
            assert_eq!(check(&renamed), Ok(2));
            if id.contains("comprehension") {
                for (label, changed) in [
                    ("missing citation", source.replace(" by A1", "")),
                    ("wrong citation", source.replace("by A1", "by Missing")),
                    ("missing guard", source.replace(" : z in A", "")),
                    (
                        "non-set bound",
                        source.replace("reserve A for set", "reserve A for object"),
                    ),
                    (
                        "shadowed set reservation",
                        source.replace(
                            "reserve A for set;",
                            "reserve A for set;\nreserve A for object;",
                        ),
                    ),
                    ("dependent bound", source.replace("z in A", "z in z")),
                    ("wrong mapper", source.replace("{ z where", "{ x where")),
                    (
                        "wrong proof premise",
                        source.replace("A1: x in A", "A1: A in A"),
                    ),
                    (
                        "altered conclusion",
                        source.replace("thus x in", "thus A in"),
                    ),
                    ("unjustified premise", source.replace(" st x in A", "")),
                ] {
                    assert!(check(&changed).is_err(), "{label} was accepted");
                }
                let alpha = source
                    .replace("let x be", "let u be")
                    .replace("A1: x in", "A1: u in")
                    .replace("thus x in", "thus u in");
                assert_eq!(check(&alpha), Ok(2));
            } else {
                for changed in [
                    source.replace("thus x in", "thus y in"),
                    source.replace("{x, y}", "{y}"),
                    source.replace("thus x in {x, y}", "thus x in {x}"),
                    source.replace("let x, y be object", "let x, y be set"),
                ] {
                    assert!(
                        check(&changed).is_err(),
                        "altered enumeration proof accepted: {changed}"
                    );
                }
            }
        }
    }

    #[test]
    fn step5c7_citation_results_cannot_be_substituted() {
        use mizar_checker::{
            source_set_term::SourceSetTermProducer, type_checker::SourceVariableSemanticsChecker,
        };
        use mizar_resolve::{
            labels::{
                LabelReferenceCandidate, LabelReferenceScope, LabelResolver,
                ProofLabelSourceCollector,
            },
            names::{SourceVariableScopeInput, SourceVariableScopeResolver},
            resolved_ast::{ReferenceSite, SurfaceResolvedArena},
        };
        let config = config();
        let root = &config.workspace_root;
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
            .unwrap();
        let source = fs::read_to_string(root.join(STEP5C7_PROOF_CASES[0].1)).unwrap();
        for citation in ["A1", "Missing"] {
            let ast = parse(&source.replace("by A1", &format!("by {citation}")));
            let symbols = resolver_symbol_collection(root, case, &ast);
            let module = &symbols.module;
            let scope = SourceVariableScopeResolver::resolve_occurrences(
                SourceVariableScopeInput::new(&ast, module, &symbols.env),
            )
            .unwrap();
            let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
            let typed = super::super::type_elaboration::step5c7_membership_typed_ast(
                &ast,
                module,
                &symbols.env,
                &scope,
                &bindings,
            )
            .unwrap();
            let arena = SurfaceResolvedArena::lower(&ast, module).unwrap();
            let owner = symbols
                .env
                .symbols()
                .iter()
                .find(|entry| entry.kind() == mizar_resolve::env::SymbolKind::Theorem)
                .unwrap();
            let namespace = mizar_resolve::env::NamespacePath::new(module.path().as_str());
            let labels = ProofLabelSourceCollector::new(
                &ast,
                module,
                namespace.clone(),
                owner.contribution(),
                &arena,
            )
            .unwrap()
            .collect_with_let_conditions()
            .unwrap();
            let resolve = |references: &[LabelReferenceCandidate]| {
                LabelResolver::new(labels.projections()).resolve(module, &namespace, references)
            };
            let check = |resolved| {
                SourceSetTermProducer::check_membership_proof(
                    &typed,
                    &scope,
                    &bindings,
                    &labels,
                    resolved,
                    &symbols.env,
                )
                .is_ok()
            };
            let original = resolve(labels.references());
            assert_eq!(check(&original), citation == "A1");
            let reference = &labels.references()[0];
            let LabelReferenceScope::Unqualified { proof_scope } = reference.scope() else {
                panic!("local citation")
            };
            let forged = LabelReferenceCandidate::unqualified_citation(
                ReferenceSite::new(reference.site().node(), reference.site().range(), "A1"),
                if citation == "A1" {
                    labels.projections()[0].origin().clone()
                } else {
                    reference.origin().clone()
                },
                reference.ordinal(),
                proof_scope.clone(),
            );
            let substituted = resolve(&[forged]);
            assert!(!substituted.has_unresolved());
            assert!(
                !check(&substituted),
                "substituted site/origin for {citation}"
            );
        }
    }

    #[test]
    fn step5c7_proof_admission_is_exact() {
        let config = config();
        let root = &config.workspace_root;
        let mut plan = crate::harness::build_test_plan(&config).unwrap();
        let diagnostics = validate_active_proof_verification_tags(root, &plan);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let original = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
            .unwrap()
            .clone();
        for mutate in [
            |case: &mut TestCase| case.expectation.tags.clear(),
            |case: &mut TestCase| case.expectation.tags.push("extra".into()),
            |case: &mut TestCase| case.expectation.expected_phase = Some(PipelinePhase::TypeCheck),
            |case: &mut TestCase| case.expectation.expected_outcome = ExpectedOutcome::Fail,
            |case: &mut TestCase| case.expectation.stable_detail_key = Some("forged".into()),
            |case: &mut TestCase| case.expectation.source = "wrong.miz".into(),
            |case: &mut TestCase| case.id.0 = "unlisted".into(),
            |case: &mut TestCase| case.expectation.id.0 = "forged".into(),
            |case: &mut TestCase| case.expectation_path.set_file_name("wrong.expect.toml"),
        ] {
            let mut altered = original.clone();
            mutate(&mut altered);
            assert!(!step5c7_proof_workspace_member(root, &altered));
            assert_eq!(
                run_proof_verification_case(root, &root.join("tests"), &altered, 0).status,
                ProofVerificationCaseStatus::Failed
            );
        }
        let mut wrong_root = original.clone();
        wrong_root.source_path = root.join("alias").join(STEP5C7_PROOF_CASES[0].1);
        assert!(!step5c7_proof_workspace_member(root, &wrong_root));
        plan.cases.push(original);
        assert!(!validate_active_proof_verification_tags(root, &plan).is_empty());
        plan.cases
            .retain(|case| case.id.0 != STEP5C7_PROOF_CASES[1].0);
        assert!(
            validate_active_proof_verification_tags(root, &plan)
                .iter()
                .any(|diagnostic| diagnostic.detail_key.ends_with(STEP5C7_PROOF_CASES[1].0))
        );
    }

    #[test]
    fn step5c7_qua_checks_both_source_scopes_and_rejects_unrelated_errors() {
        let config = config();
        let root = &config.workspace_root;
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == "fail_type_elaboration_term_qua_invalid_narrowing_001")
            .unwrap();
        let source = fs::read_to_string(&case.source_path).unwrap();
        let keys = |source: &str| {
            let ast = parse(source);
            let symbols = resolver_symbol_collection(root, case, &ast);
            if !symbols.detail_keys.is_empty() {
                return symbols.detail_keys;
            }
            super::super::type_elaboration::step5c7_term_detail_keys(
                &ast,
                &symbols.module,
                &symbols.env,
            )
        };
        let invalid = vec!["terms.qua.invalid_narrowing".to_owned()];
        assert_eq!(keys(&source), invalid);
        let legal = source.replace("qua QuaBox", "qua object");
        assert!(keys(&legal).is_empty());
        assert!(keys(&source.replace("qua QuaBox", "qua set")).is_empty());
        assert_eq!(
            keys(&source.replacen("qua QuaBox", "qua object", 1)),
            invalid
        );
        assert_eq!(
            keys(&legal.replacen("qua object", "qua QuaBox", 1)),
            invalid
        );
        let renamed = source
            .replace("QuaBox", "Crate")
            .replace("field d", "field payload")
            .replace("X", "Y");
        assert_eq!(keys(&renamed), invalid);
        assert!(keys(&renamed.replace("qua Crate", "qua object")).is_empty());
        let (_, theorem) = source.split_once("theorem").unwrap();
        let definition = source.split_once("theorem").unwrap().0;
        for changed in [
            source.replace("qua QuaBox", "qua Unknown"),
            source.replacen("X qua", "Z qua", 1),
            source.replace("being set", "being object"),
            source.replace("be set", "be object"),
            source.replace("field d -> set;", "field d -> object;"),
            source.replace("field d -> set;", "field d -> set; field d -> set;"),
            source.replace("field d -> set;", "field d -> ;"),
            source.replace("struct QuaBox where", "struct QuaBox -> Missing where"),
            source.replace("qua QuaBox", "qua empty QuaBox"),
            source.replace(
                "struct QuaBox where\n    field d -> set;\n  end;",
                "mode QuaBox is set;",
            ),
            format!("theorem{theorem}{definition}"),
            source.replace("thus (X qua QuaBox)", "thus (Z qua object)"),
        ] {
            assert_ne!(keys(&changed), invalid, "{changed}");
        }
        let ast = parse(&source);
        let symbols = resolver_symbol_collection(root, case, &ast);
        let mut pending = vec![ast.root().unwrap()];
        let mut reachable = std::collections::BTreeSet::new();
        while let Some(id) = pending.pop() {
            assert!(
                reachable.insert(id),
                "duplicate structural parent for {id:?}"
            );
            let node = ast.node(id).unwrap();
            pending.extend(node.children.iter().copied().filter(|child| {
                node.kind != mizar_syntax::SurfaceNodeKind::Root
                    || !matches!(
                        ast.node(*child).unwrap().kind,
                        mizar_syntax::SurfaceNodeKind::Token(_)
                    )
            }));
        }
        assert_eq!(reachable.len(), ast.nodes().len(), "orphan source nodes");
        let scope = mizar_resolve::names::SourceVariableScopeResolver::resolve_occurrences(
            mizar_resolve::names::SourceVariableScopeInput::new(
                &ast,
                &symbols.module,
                &symbols.env,
            ),
        )
        .unwrap();
        let bindings = scope
            .references()
            .iter()
            .map(|reference| reference.binding())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(bindings.len(), 2);
        let qua_sites = ast
            .nodes()
            .iter()
            .filter(|node| node.kind == mizar_syntax::SurfaceNodeKind::QuaExpression)
            .map(|node| node.range)
            .collect::<Vec<_>>();
        assert_eq!(qua_sites.len(), 2);
        assert_ne!(qua_sites[0], qua_sites[1]);
        let renamed_ast = parse(&renamed);
        let stale = super::super::type_elaboration::step5c7_term_detail_keys(
            &renamed_ast,
            &symbols.module,
            &symbols.env,
        );
        assert_ne!(stale, invalid);
        let mut foreign_ast = ast.clone();
        use mizar_session::SessionIdAllocator;
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(777)).unwrap();
        foreign_ast.source_id = ids.next_source_id(snapshot_id(777)).unwrap();
        assert_ne!(
            super::super::type_elaboration::step5c7_term_detail_keys(
                &foreign_ast,
                &symbols.module,
                &symbols.env
            ),
            invalid
        );
    }

    #[test]
    fn step5c7_term_evidence_is_source_derived() {
        let config = config();
        let root = &config.workspace_root;
        let plan = crate::harness::build_test_plan(&config).unwrap();
        let case = plan
            .cases
            .iter()
            .find(|case| case.id.0 == STEP5C7_PROOF_CASES[0].0)
            .unwrap();
        let keys = |source: &str| {
            let ast = parse(source);
            let symbols = resolver_symbol_collection(root, case, &ast);
            super::super::type_elaboration::step5c7_term_detail_keys(
                &ast,
                &symbols.module,
                &symbols.env,
            )
        };
        let qua = fs::read_to_string(
            root.join("tests/miz/pass/terms/pass_type_elaboration_term_qua_widening_001.miz"),
        )
        .unwrap();
        assert!(keys(&qua).is_empty());
        assert!(
            !keys(
                &qua.replace("being set", "being object")
                    .replace("be set", "be object")
                    .replace("qua object", "qua set")
            )
            .is_empty()
        );
        let wrong_base = "theorem T: for x being object for y being set holds (x qua set) = y proof let x be object; let y be set; thus (x qua set) = y; end;";
        assert!(!keys(wrong_base).is_empty());
        let choice = fs::read_to_string(
            root.join("tests/miz/fail/terms/fail_type_elaboration_term_choice_uninhabited_001.miz"),
        )
        .unwrap();
        let mixed = choice.replacen(
            "the emptyish set = the emptyish set",
            "the set = the emptyish set",
            1,
        );
        assert_eq!(keys(&mixed), ["terms.choice.missing_inhabitation"]);
        assert_eq!(
            keys(&choice.replace("emptyish", "barren")),
            ["terms.choice.missing_inhabitation"]
        );
        assert!(!keys("theorem T: 1 = 1 proof thus 1 = 1; end;").is_empty());
        let mut ast = parse(&fs::read_to_string(&case.source_path).unwrap());
        let symbols = resolver_symbol_collection(root, case, &ast);
        use mizar_session::SessionIdAllocator;
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(777)).unwrap();
        ast.source_id = ids.next_source_id(snapshot_id(777)).unwrap();
        assert!(normalize_term_ast(&ast, &symbols.module, &symbols.env).is_err());
    }

    #[test]
    fn source_theorem_local_handoff_exact_local_projection() {
        use mizar_core::core_ir::{CoreFormulaKind, CoreProofNodeKind, CoreTermKind};
        use mizar_kernel::{
            clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
            formula_evidence::Formula,
        };
        use mizar_vc::{kernel_evidence_handoff::*, vc_ir::VcFormulaRef};
        let guard_wire = [1, 4, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0];
        let goal_wire = [
            1, 3, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
        ];
        let guard_fingerprint = Formula::Atom(Atom::new(
            SymbolKey::new(SymbolKind::BuiltinRelation, 1),
            vec![Term::Variable(VariableId(0))],
        ))
        .canonical_hash_input()
        .unwrap();
        let goal_fingerprint = Formula::Atom(Atom::new(
            SymbolKey::new(SymbolKind::Equality, 0),
            vec![Term::Variable(VariableId(0)), Term::Variable(VariableId(0))],
        ))
        .canonical_hash_input()
        .unwrap();
        for source in [
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "theorem Renamed: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;",
        ] {
            let core = step5c14_computation_core(source).unwrap();
            let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
            let vc = &vcs.vcs()[0];
            let proof = core.proofs().iter().next().unwrap().1;
            let CoreFormulaKind::Forall { binders, .. } =
                &core.formulas().get(proof.proposition).unwrap().kind
            else {
                panic!()
            };
            let CoreProofNodeKind::IntroduceBinder { binder, .. } =
                &core.proof_nodes().get(proof.root).unwrap().kind
            else {
                panic!()
            };
            assert_ne!(binders[0].var, binder.var);
            assert_eq!(binder.role.as_str(), "proof-let");
            let local_guard = binder.ty_guard.unwrap();
            let CoreFormulaKind::TypePred { subject, ty } =
                &core.formulas().get(local_guard).unwrap().kind
            else {
                panic!()
            };
            assert_eq!(ty.as_str(), "set");
            assert_eq!(
                core.terms().get(*subject).unwrap().kind,
                CoreTermKind::Var(binder.var)
            );
            let VcFormulaRef::Core(goal) = vc.goal else {
                panic!()
            };
            let CoreFormulaKind::Equals { left, right } = core.formulas().get(goal).unwrap().kind
            else {
                panic!()
            };
            for term in [left, right] {
                assert_eq!(
                    core.terms().get(term).unwrap().kind,
                    CoreTermKind::Var(binder.var)
                );
            }
            let handoff = build_source_theorem_kernel_handoff(&core, &vcs, vc.id).unwrap();
            let envelope = handoff.canonical_evidence();
            assert_eq!(
                envelope.symbol_manifest(),
                [
                    KernelManifestEntry {
                        entry_id: 0,
                        payload: vec![3, 0, 0, 0, 0]
                    },
                    KernelManifestEntry {
                        entry_id: 1,
                        payload: vec![4, 0, 0, 0, 1]
                    },
                ]
            );
            assert_eq!(
                envelope.variable_manifest(),
                [KernelManifestEntry {
                    entry_id: 0,
                    payload: vec![0, 0, 0, 0]
                }]
            );
            assert_eq!(envelope.formula_evidence().len(), 1);
            let guard = &envelope.formula_evidence()[0];
            assert_eq!(
                guard.source(),
                &KernelFormulaSource::LocalHypothesis {
                    local_context_id: 1
                }
            );
            assert_eq!(guard.formula_bytes(), guard_wire);
            assert_eq!(
                guard.formula_fingerprint(),
                &KernelEvidenceFingerprint {
                    algorithm_id: 2,
                    digest: guard_fingerprint.clone()
                }
            );
            assert_eq!(
                guard.producer_formula_ref(),
                Some(VcFormulaRef::Core(local_guard))
            );
            assert_eq!(handoff.context_identity().entries().len(), 1);
            let context = &handoff.context_identity().entries()[0];
            assert_eq!(
                context.source(),
                KernelContextIdentitySource::LocalHypothesis {
                    local_context_id: 1
                }
            );
            assert_eq!(
                context.producer_formula_ref(),
                VcFormulaRef::Core(local_guard)
            );
            assert_eq!(context.formula_fingerprint(), guard.formula_fingerprint());
            assert_eq!(envelope.final_goal().formula_bytes, goal_wire);
            assert_eq!(
                envelope.final_goal().formula_fingerprint,
                KernelEvidenceFingerprint {
                    algorithm_id: 2,
                    digest: goal_fingerprint.clone()
                }
            );
            assert_eq!(
                envelope.final_goal().polarity,
                KernelGoalPolarity::AssertFalseForRefutation
            );
            assert!(envelope.substitutions().is_empty());
            assert!(handoff.formula_context_requirements().is_none());
            assert!(handoff.context_identity_is_consistent());
            assert!(handoff.targets_vc(&vcs, vc.id).unwrap());
        }
    }

    #[test]
    fn source_theorem_local_handoff_generic_equality_and_kernel_nonacceptance() {
        use mizar_kernel::{
            certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
            checker::{
                FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
                KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
                KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
                KernelEvidenceCheckLimits, KernelFormulaProducerRef, check_kernel_evidence,
            },
            clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
            formula_evidence::{Formula, FormulaEvidenceParseContext, parse_formula_evidence},
            rejection::{RejectionDetail, TargetVcFingerprint},
        };
        use mizar_vc::kernel_evidence_handoff::KernelContextIdentitySource as SourceContext;
        use mizar_vc::{kernel_evidence_handoff::*, vc_ir::VcFormulaRef};
        let source = "theorem T: for X being set holds X = X proof let Y be set; thus Y = Y; end;";
        let core = step5c14_computation_core(source).unwrap();
        let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
        let original = (core.clone(), vcs.clone());
        let vc = &vcs.vcs()[0];
        let (_, item) = core.items().iter().next().unwrap();
        let (_, proof) = core.proofs().iter().next().unwrap();
        let (_, seed) = core.obligation_seeds().iter().next().unwrap();
        let provenance = format!(
            "source-theorem-local-handoff-v1;source={:?};module={:?};item={:?};proof={:?};seed={:?};accounting={:?}",
            core.source_id(), core.module_id(), item, proof, seed, vcs.seed_accounting()
        ).into_bytes();
        let symbols = [
            KernelManifestEntry {
                entry_id: 0,
                payload: vec![3, 0, 0, 0, 0],
            },
            KernelManifestEntry {
                entry_id: 1,
                payload: vec![4, 0, 0, 0, 1],
            },
        ];
        let variables = [KernelManifestEntry {
            entry_id: 0,
            payload: vec![0, 0, 0, 0],
        }];
        let formulas = [
            (
                vc.local_context.entries()[0].formula.unwrap(),
                vec![1, 4, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0],
                Formula::Atom(Atom::new(
                    SymbolKey::new(SymbolKind::BuiltinRelation, 1),
                    vec![Term::Variable(VariableId(0))],
                )),
            ),
            (
                vc.goal,
                vec![
                    1, 3, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 2, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0,
                ],
                Formula::Atom(Atom::new(
                    SymbolKey::new(SymbolKind::Equality, 0),
                    vec![Term::Variable(VariableId(0)), Term::Variable(VariableId(0))],
                )),
            ),
        ]
        .into_iter()
        .map(
            |(formula_ref, formula_bytes, formula)| KernelFormulaPayload {
                formula_ref,
                projection: KernelFormulaProjection {
                    formula_fingerprint: KernelEvidenceFingerprint {
                        algorithm_id: 2,
                        digest: formula.canonical_hash_input().unwrap(),
                    },
                    formula_bytes,
                    provenance_payload: provenance.clone(),
                },
            },
        )
        .collect::<Vec<_>>();
        let expected = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
            vc_set: &vcs,
            vc: vc.id,
            goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
            kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
            symbol_manifest: &symbols,
            variable_manifest: &variables,
            formula_payloads: &formulas,
            imported_formula_payloads: &[],
            substitutions: &[],
            formula_context: None,
            discharge_output: None,
        })
        .unwrap();
        let handoff = build_source_theorem_kernel_handoff(&core, &vcs, vc.id).unwrap();
        assert_eq!(handoff, expected);
        assert_eq!(
            handoff,
            build_source_theorem_kernel_handoff(&core, &vcs, vc.id).unwrap()
        );
        for row in handoff.canonical_evidence().provenance() {
            assert_eq!(row.payload, provenance);
        }
        let fresh = generate_core_vcs(&core, snapshot_id(778)).unwrap();
        let fresh_handoff = build_source_theorem_kernel_handoff(&core, &fresh, vc.id).unwrap();
        assert_ne!(handoff.canonical_hash(), fresh_handoff.canonical_hash());
        assert!(!handoff.targets_vc(&fresh, vc.id).unwrap());
        let bytes = handoff.encode_local_evidence(None).unwrap();
        assert_eq!(&bytes[..26], b"MIZAR_KERNEL_EVIDENCE\0\0\x01\0\x01");
        let fingerprint = handoff.canonical_evidence().target_vc();
        let parsed = parse_formula_evidence(
            &bytes,
            &FormulaEvidenceParseContext::v1(
                Fingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
                KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
            ),
        )
        .unwrap();
        assert!(parsed.equality_instances().is_empty());
        let target = TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone());
        let entries = handoff
            .context_identity()
            .entries()
            .iter()
            .map(|entry| {
                let VcFormulaRef::Core(formula) = entry.producer_formula_ref() else {
                    panic!()
                };
                let SourceContext::LocalHypothesis { local_context_id } = entry.source() else {
                    panic!()
                };
                KernelContextIdentityEntry::new(
                    KernelContextIdentitySource::LocalHypothesis { local_context_id },
                    entry.formula_id(),
                    Fingerprint::new(
                        entry.formula_fingerprint().algorithm_id,
                        entry.formula_fingerprint().digest.clone(),
                    ),
                    KernelFormulaProducerRef::Core(formula),
                )
            })
            .collect();
        let context = FormulaEvidenceContext::with_context_identity(
            Some(handoff.context_identity_hash().as_bytes().to_vec()),
            vec![],
            vec![],
            Some(KernelContextIdentityPayload::new(
                target.clone(),
                handoff.canonical_hash(),
                handoff.context_identity_hash(),
                entries,
            )),
            ImportedFactContextLimits::default(),
        )
        .unwrap();
        let result = check_kernel_evidence(KernelEvidenceCheckInput {
            target_vc_fingerprint: &target,
            evidence: &parsed,
            formula_context: Some(&context),
            check_kind: KernelEvidenceCheckKind::ProofObligation,
            policy: KernelCheckPolicy::default(),
            limits: KernelEvidenceCheckLimits::default(),
        });
        assert_eq!(result.status(), KernelCheckStatus::Rejected);
        assert_eq!(result.rejections().len(), 1);
        let rejection = &result.rejections()[0];
        assert_eq!(rejection.detail(), RejectionDetail::InvalidSatRefutation);
        assert_eq!(
            rejection.location().field_path,
            Some("sat_checker.satisfiable")
        );
        assert!(!result.policy_taint());
        assert_eq!((core, vcs), original);
    }

    #[test]
    fn source_theorem_local_handoff_refuses_foreign_and_unsupported_graphs() {
        use mizar_core::core_ir::CoreItemKind;
        use mizar_vc::{
            discharge::{DischargeInput, DischargePolicy, try_discharge},
            kernel_evidence_handoff::build_source_theorem_kernel_handoff,
            vc_ir::*,
        };
        let source = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let core = step5c14_computation_core(source).unwrap();
        let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
        let selected = vcs.vcs()[0].id;
        let invalid = "theorem.handoff.invalid_projection";
        assert_eq!(
            build_source_theorem_kernel_handoff(&core, &vcs, VcId::new(1)).unwrap_err(),
            invalid
        );
        let foreign = step5c14_computation_core(&source.replace("T:", "Different:")).unwrap();
        assert!(build_source_theorem_kernel_handoff(&foreign, &vcs, selected).is_err());
        let lemma = step5c14_computation_core(&source.replacen("theorem", "lemma", 1)).unwrap();
        assert_eq!(lemma.items().len(), 1);
        assert_eq!(
            lemma.items().iter().next().unwrap().1.kind,
            CoreItemKind::Lemma
        );
        let lemma_vcs = generate_core_vcs(&lemma, snapshot_id(777)).unwrap();
        assert_eq!(lemma_vcs.vcs().len(), 1);
        assert_eq!(
            build_source_theorem_kernel_handoff(&lemma, &lemma_vcs, lemma_vcs.vcs()[0].id)
                .unwrap_err(),
            invalid
        );
        for unsupported in [
            source.replace("set", "object"),
            format!("{source} {}", source.replace("T:", "U:")),
        ] {
            let actual = step5c14_computation_core(&unsupported).unwrap();
            let actual_vcs = generate_core_vcs(&actual, snapshot_id(777)).unwrap();
            assert!(!actual_vcs.vcs().is_empty(), "{unsupported}");
            for vc in actual_vcs.vcs() {
                assert_eq!(
                    build_source_theorem_kernel_handoff(&actual, &actual_vcs, vc.id).unwrap_err(),
                    invalid,
                    "{unsupported}"
                );
            }
        }
        let computation =
            step5c14_computation_core("theorem C: 0 = 0 by computation(steps: 8);").unwrap();
        let computation_vcs = mizar_vc::generator::generate_source_computation_request(
            &computation,
            snapshot_id(777),
            &GenerationSchemaVersion::new("mizar-vc-generation-step5c14-computation-v1"),
            &VcSchemaVersion::new("mizar-vc-vcset-step5c14-computation-v1"),
        )
        .unwrap();
        assert_eq!(computation_vcs.vcs().len(), 1);
        assert_eq!(computation_vcs.vcs()[0].status, VcStatus::Open);
        assert_eq!(
            build_source_theorem_kernel_handoff(
                &computation,
                &computation_vcs,
                computation_vcs.vcs()[0].id
            )
            .unwrap_err(),
            invalid
        );
        let cited = include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        );
        let cited_core = step5c14_computation_core(cited).unwrap();
        let cited_vcs = generate_core_vcs(&cited_core, snapshot_id(777)).unwrap();
        assert_eq!(cited_vcs.vcs().len(), 2);
        assert_eq!(
            check_theorem_source(cited, PipelinePhase::VcGeneration)
                .unwrap()
                .1
                .unwrap(),
            generate_core_vcs(&cited_core, snapshot_id(777)).unwrap()
        );
        assert!(cited_vcs.vcs()[1].local_context.entries().len() > 1);
        for vc in cited_vcs.vcs() {
            assert_eq!(
                build_source_theorem_kernel_handoff(&cited_core, &cited_vcs, vc.id).unwrap_err(),
                invalid
            );
        }
        let discharge = try_discharge(DischargeInput {
            vc_set: &vcs,
            policy: &DischargePolicy::default(),
        })
        .unwrap();
        assert_ne!(discharge.vc_set(), &vcs);
        assert_eq!(
            build_source_theorem_kernel_handoff(&core, discharge.vc_set(), selected).unwrap_err(),
            invalid
        );
        for mutation in 0..5 {
            let mut parts = VcSetParts {
                schema_version: vcs.schema_version().clone(),
                snapshot: vcs.snapshot(),
                source: vcs.source(),
                module: vcs.module().clone(),
                generated_formulas: vcs.generated_formulas().to_vec(),
                vcs: vcs.vcs().to_vec(),
                seed_accounting: vcs.seed_accounting().to_vec(),
            };
            match mutation {
                0 => parts.vcs[0].status = VcStatus::NeedsAtp,
                1 => parts.vcs[0].provenance.clear(),
                2 => {
                    let primary = parts.vcs[0].source.primary.clone();
                    parts.vcs[0].source.related.push(primary);
                }
                3 => parts.vcs[0].premises.clear(),
                4 => parts.module = VcModuleRef::new("foreign-module"),
                _ => unreachable!(),
            }
            let changed = VcSet::try_new(parts).unwrap();
            assert_ne!(
                changed, vcs,
                "mutation {mutation} must change the whole graph"
            );
            assert_eq!(
                build_source_theorem_kernel_handoff(&core, &changed, selected).unwrap_err(),
                invalid,
                "mutation {mutation}"
            );
        }
        assert!(build_source_theorem_kernel_handoff(&core, &vcs, selected).is_ok());
    }

    #[test]
    fn source_theorem_variable_reflexivity_names_reach_normal_kernel() {
        use mizar_atp::backend::{
            encode_closed_choice_reflexivity_names, encode_local_variable_reflexivity_names,
        };
        use mizar_kernel::{
            certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
            checker::{
                FormulaEvidenceContext, ImportedFactContextLimits, KernelCheckPolicy,
                KernelCheckStatus, KernelContextIdentityEntry, KernelContextIdentityPayload,
                KernelContextIdentitySource, KernelEvidenceCheckInput, KernelEvidenceCheckKind,
                KernelEvidenceCheckLimits, KernelFormulaProducerRef, check_kernel_evidence,
            },
            formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
            rejection::{RejectionDetail, TargetVcFingerprint},
        };
        use mizar_vc::{kernel_evidence_handoff::build_source_theorem_kernel_handoff, vc_ir::*};
        for source in [
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
            "theorem Renamed: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;",
        ] {
            let core = step5c14_computation_core(source).unwrap();
            let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
            let original = (core.clone(), vcs.clone());
            let vc = &vcs.vcs()[0];
            assert_eq!(vc.status, VcStatus::Open);
            let handoff = build_source_theorem_kernel_handoff(&core, &vcs, vc.id).unwrap();
            let before = handoff.clone();
            let fingerprint = handoff.canonical_evidence().target_vc();
            let target =
                TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone());
            let mut parse_context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
                KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
            );
            let v1 = handoff.encode_local_evidence(None).unwrap();
            let parsed = parse_formula_evidence(&v1, &parse_context).unwrap();
            let original_parsed = parsed.clone();
            let names = encode_local_variable_reflexivity_names(&parsed).unwrap();
            assert_eq!(names, [vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0]]);
            assert_eq!(
                names,
                encode_local_variable_reflexivity_names(&parsed).unwrap()
            );
            assert_eq!(parsed, original_parsed);
            assert!(
                encode_closed_choice_reflexivity_names(&parsed)
                    .unwrap()
                    .is_empty()
            );
            let v2 = handoff.encode_local_evidence(Some(&names)).unwrap();
            parse_context.accepted_schema_versions.insert(2);
            let named = parse_formula_evidence(&v2, &parse_context).unwrap();
            assert_eq!(named.equality_instances().len(), 1);
            let foreign_vcs = generate_core_vcs(&core, snapshot_id(778)).unwrap();
            let foreign = build_source_theorem_kernel_handoff(&core, &foreign_vcs, vc.id).unwrap();
            let contexts = [&handoff, &foreign].map(|binding| {
                let entries = binding.context_identity().entries().iter().map(|entry| {
                    let VcFormulaRef::Core(formula) = entry.producer_formula_ref() else {
                        panic!()
                    };
                    let mizar_vc::kernel_evidence_handoff::KernelContextIdentitySource::LocalHypothesis { local_context_id } = entry.source() else {
                        panic!()
                    };
                    assert_eq!(local_context_id, 1);
                    KernelContextIdentityEntry::new(
                        KernelContextIdentitySource::LocalHypothesis { local_context_id },
                        entry.formula_id(),
                        Fingerprint::new(entry.formula_fingerprint().algorithm_id, entry.formula_fingerprint().digest.clone()),
                        KernelFormulaProducerRef::Core(formula),
                    )
                }).collect();
                let fingerprint = binding.canonical_evidence().target_vc();
                FormulaEvidenceContext::with_context_identity(
                    Some(binding.context_identity_hash().as_bytes().to_vec()),
                    vec![], vec![],
                    Some(KernelContextIdentityPayload::new(
                        TargetVcFingerprint::new(fingerprint.algorithm_id, fingerprint.digest.clone()),
                        binding.canonical_hash(), binding.context_identity_hash(), entries,
                    )),
                    ImportedFactContextLimits::default(),
                ).unwrap()
            });
            let foreign_fingerprint = foreign.canonical_evidence().target_vc();
            let foreign_target = TargetVcFingerprint::new(
                foreign_fingerprint.algorithm_id,
                foreign_fingerprint.digest.clone(),
            );
            assert_ne!(target, foreign_target);
            for (evidence, target, context, accepted) in [
                (&named, &target, &contexts[0], true),
                (&parsed, &target, &contexts[0], false),
                (&named, &target, &contexts[1], false),
                (&named, &foreign_target, &contexts[0], false),
            ] {
                let result = check_kernel_evidence(KernelEvidenceCheckInput {
                    target_vc_fingerprint: target,
                    evidence,
                    formula_context: Some(context),
                    check_kind: KernelEvidenceCheckKind::ProofObligation,
                    policy: KernelCheckPolicy::default(),
                    limits: KernelEvidenceCheckLimits::default(),
                });
                assert_eq!(
                    result.status(),
                    if accepted {
                        KernelCheckStatus::Accepted
                    } else {
                        KernelCheckStatus::Rejected
                    }
                );
                assert_eq!(
                    result.evidence_check_kind(),
                    Some(KernelEvidenceCheckKind::ProofObligation)
                );
                assert!(!result.policy_taint());
                assert!(result.used_axioms().is_empty());
                if std::ptr::eq(evidence, &parsed) {
                    assert_eq!(
                        result.rejections()[0].detail(),
                        RejectionDetail::InvalidSatRefutation
                    );
                }
            }
            assert_eq!(handoff, before);
            assert_eq!(handoff.encode_local_evidence(None).unwrap(), v1);
            assert_eq!((core, vcs), original);
        }
    }

    #[test]
    fn source_theorem_variable_reflexivity_untrusted_parsed_profiles() {
        use mizar_atp::backend::encode_local_variable_reflexivity_names;
        use mizar_kernel::{
            certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
            clause::{Atom, SymbolKey, SymbolKind, Term, VariableId},
            formula_evidence::{
                Formula, FormulaEvidenceParseContext, GoalPolarity, parse_formula_evidence,
            },
        };
        use mizar_vc::kernel_evidence_handoff::*;
        let core = step5c14_computation_core(
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;",
        )
        .unwrap();
        let vcs = generate_core_vcs(&core, snapshot_id(777)).unwrap();
        let vc = &vcs.vcs()[0];
        let equality = SymbolKey::new(SymbolKind::Equality, 7);
        let variable = Term::Variable(VariableId(11));
        let goal = Formula::Atom(Atom::new(
            equality,
            vec![variable.clone(), variable.clone()],
        ));
        let goal_wire = vec![
            1, 3, 0, 0, 0, 7, 0, 0, 0, 2, 0, 0, 0, 2, 1, 0, 0, 0, 11, 1, 0, 0, 0, 11,
        ];
        // These raw projections exercise only the untrusted encoder, never source acceptance.
        for case in 0..9 {
            let mut formula = goal.clone();
            let mut wire = goal_wire.clone();
            match case {
                1 => {
                    formula = Formula::Atom(Atom::new(
                        SymbolKey::new(SymbolKind::Predicate, 8),
                        vec![variable.clone(), variable.clone()],
                    ));
                    wire[1] = 1;
                    wire[5] = 8;
                }
                2 => {
                    formula = Formula::Not(Box::new(formula));
                    wire.insert(0, 2);
                }
                3 => {
                    formula = Formula::Atom(Atom::new(
                        equality,
                        vec![variable.clone(), Term::Variable(VariableId(12))],
                    ));
                    wire[23] = 12;
                }
                4 => {
                    let ground = Term::Application {
                        symbol: SymbolKey::new(SymbolKind::FunctorPredicate, 9),
                        arguments: vec![],
                    };
                    formula = Formula::Atom(Atom::new(equality, vec![ground.clone(), ground]));
                    wire.truncate(14);
                    wire.extend([2, 2, 0, 0, 0, 9, 0, 0, 0, 0, 2, 2, 0, 0, 0, 9, 0, 0, 0, 0]);
                }
                8 => {
                    formula = Formula::Atom(Atom::new(equality, vec![variable.clone(); 3]));
                    wire[9] = 3;
                    wire[13] = 3;
                    wire.extend([1, 0, 0, 0, 11]);
                }
                _ => {}
            }
            let formulas =
                [vc.local_context.entries()[0].formula.unwrap(), vc.goal].map(|formula_ref| {
                    KernelFormulaPayload {
                        formula_ref,
                        projection: KernelFormulaProjection {
                            formula_fingerprint: KernelEvidenceFingerprint {
                                algorithm_id: 2,
                                digest: formula.canonical_hash_input().unwrap(),
                            },
                            formula_bytes: wire.clone(),
                            provenance_payload: b"untrusted-encoder-input".to_vec(),
                        },
                    }
                });
            let substitutions = if case == 5 {
                vec![KernelSubstitutionPayload {
                    substitution_id: 1,
                    source_formula: formulas[0].formula_ref,
                    binder_context_encoding: vec![
                        0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 0, 0, 0, 0,
                    ],
                    payload: vec![
                        0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 1, 0, 0, 0, 11, 1,
                    ],
                    freshness_witnesses: vec![],
                    free_variable_constraints: vec![],
                    provenance_payload: b"untrusted-substitution".to_vec(),
                }]
            } else {
                vec![]
            };
            let symbols = [(8, 1), (9, 2), (7, 3)].map(|(id, kind)| KernelManifestEntry {
                entry_id: kind as u32,
                payload: vec![kind, 0, 0, 0, id],
            });
            let variables = [11, 12].map(|id| KernelManifestEntry {
                entry_id: id,
                payload: id.to_be_bytes().to_vec(),
            });
            let handoff = build_kernel_evidence_handoff(KernelEvidenceHandoffInput {
                vc_set: &vcs,
                vc: vc.id,
                goal_polarity: KernelGoalPolarity::AssertFalseForRefutation,
                kernel_profile: KernelEvidenceProfile::v1(1, KernelClauseTautologyPolicy::Reject),
                symbol_manifest: &symbols,
                variable_manifest: &variables,
                formula_payloads: &formulas,
                imported_formula_payloads: &[],
                substitutions: &substitutions,
                formula_context: None,
                discharge_output: None,
            })
            .unwrap();
            let mut bytes = handoff
                .encode_local_evidence(if case == 6 { Some(&[]) } else { None })
                .unwrap();
            if case == 7 {
                // The ordinary writer's final-goal frame is last; change only its polarity byte.
                let goal_size =
                    1 + 1 + 4 + formula.canonical_hash_input().unwrap().len() + 4 + wire.len();
                let polarity_offset = bytes.len() - goal_size;
                assert_eq!(bytes[polarity_offset], 1);
                bytes[polarity_offset] = 2;
            }
            let target = handoff.canonical_evidence().target_vc();
            let mut context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
            );
            context.accepted_schema_versions.insert(2);
            let parsed = parse_formula_evidence(&bytes, &context).unwrap();
            if case == 8 {
                let Formula::Atom(atom) = &parsed.final_goal().formula else {
                    panic!()
                };
                assert_eq!(atom.arguments.len(), 3);
                assert_eq!(atom.arguments, vec![variable.clone(); 3]);
            }
            if case == 0 {
                assert_eq!(
                    encode_local_variable_reflexivity_names(&parsed).unwrap(),
                    [vec![0, 0, 0, 1, 1, 0, 0, 0, 7, 1, 0, 0, 0, 11]]
                );
            } else {
                assert_eq!(
                    encode_local_variable_reflexivity_names(&parsed).unwrap_err(),
                    "unsupported local-variable reflexivity input",
                    "case {case}"
                );
            }
            if case == 5 {
                assert_eq!(parsed.substitutions().len(), 1);
            }
            if case == 6 {
                assert_eq!(parsed.schema_version(), 2);
            }
            if case == 7 {
                assert_eq!(
                    parsed.final_goal().polarity,
                    GoalPolarity::AssertTrueForConsistency
                );
            }
        }
    }

    #[test]
    fn source_theorem_formula_payloads_preserve_actual_locals_and_content() {
        use mizar_core::{
            binder_normalization::{
                BinderContext, CanonicalFormulaKind, CanonicalTermKind, CanonicalVar,
                NormalizedVarClass, NormalizedVarSort, canonical_formula, normalize_core_formula,
            },
            core_ir::{CoreFormulaKind, CoreProofNodeKind},
            elaborator::{lower_source_theorem_skeletons, source_theorem_formula_payloads},
        };
        use mizar_vc::vc_ir::VcFormulaRef;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let mut baseline = None;
        for source in [
            original.to_owned(),
            "theorem Renamed: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;".to_owned(),
            format!(":: leading trivia\n{}\n", original.replace(" proof ", "\nproof\n")),
        ] {
            let (core, payloads) = with_step5c14_theorem_check(&source, |check| {
                let core = lower_source_theorem_skeletons(check, None)?;
                let before = core.clone();
                let proof = core.proofs().iter().next().unwrap().1;
                let CoreFormulaKind::Forall { binders, .. } =
                    &core.formulas().get(proof.proposition).unwrap().kind
                else {
                    panic!()
                };
                let CoreProofNodeKind::IntroduceBinder { binder: local, .. } =
                    &core.proof_nodes().get(proof.root).unwrap().kind
                else {
                    panic!()
                };
                assert_ne!(binders[0].var, local.var);
                assert_eq!(binders[0].role.as_str(), "quantifier");
                assert_eq!(local.role.as_str(), "proof-let");
                let guard = local.ty_guard.unwrap();
                let goal = core.obligation_seeds().iter().next().unwrap().1.goal.unwrap();
                let vcs = generate_core_vcs(&core, snapshot_id(777))?;
                assert_eq!(vcs.vcs().len(), 1);
                assert_eq!(vcs.vcs()[0].goal, VcFormulaRef::Core(goal));
                assert_eq!(
                    vcs.vcs()[0].local_context.entries()[0].formula,
                    Some(VcFormulaRef::Core(guard))
                );
                let mut context = BinderContext::new();
                for binder in [&binders[0], local] {
                    context.declare_variable(
                        binder.var,
                        NormalizedVarClass::Free,
                        binder.role.clone(),
                        NormalizedVarSort::Term,
                    );
                }
                for (id, var) in [(guard, local.var), (binders[0].ty_guard.unwrap(), binders[0].var)] {
                    let row = core.formulas().get(id).unwrap();
                    let normalized = normalize_core_formula(&core, id, &context, &row.source)
                        .map_err(|e| format!("{e:?}"))?;
                    assert_eq!(normalized.free_variables, [var].into_iter().collect());
                    let canonical = canonical_formula(&normalized, &context, &row.source)
                        .map_err(|e| format!("{e:?}"))?;
                    let CanonicalFormulaKind::TypePred { subject, .. } = canonical.kind() else {
                        panic!()
                    };
                    assert_eq!(subject.kind(), &CanonicalTermKind::Var(CanonicalVar::Free(var)));
                }
                let mut prefix = b"mizar-core/source-theorem-formula/v1\0".to_vec();
                for value in [
                    core.module_id().package().as_str(),
                    core.module_id().path().as_str(),
                    "proof-let",
                ] {
                    prefix.extend_from_slice(&u32::try_from(value.len()).unwrap().to_be_bytes());
                    prefix.extend_from_slice(value.as_bytes());
                }
                let mut guard_bytes = prefix.clone();
                guard_bytes.extend_from_slice(&[3, 0, 0, 0, 3, b's', b'e', b't', 2, 0, 0, 0, 0]);
                let mut goal_bytes = prefix;
                goal_bytes.extend_from_slice(&[2, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0]);
                let expected = [(guard, guard_bytes.clone()), (goal, goal_bytes.clone())]
                    .into_iter()
                    .collect();
                for roots in [[guard, goal], [goal, guard], [guard, goal]] {
                    assert_eq!(source_theorem_formula_payloads(check, &core, &roots)?, expected);
                }
                for (id, bytes) in [(guard, guard_bytes.clone()), (goal, goal_bytes.clone())] {
                    assert_eq!(
                        source_theorem_formula_payloads(check, &core, &[id])?,
                        [(id, bytes)].into_iter().collect()
                    );
                }
                assert!(source_theorem_formula_payloads(check, &core, &[])?.is_empty());
                assert_eq!(core, before);
                Ok((core, [guard_bytes, goal_bytes]))
            }).unwrap();
            if let Some((previous, bytes)) = &baseline {
                assert_ne!(&core, previous);
                assert_eq!(&payloads, bytes);
            } else {
                baseline = Some((core, payloads));
            }
        }
    }

    #[test]
    fn source_theorem_formula_payloads_refuse_foreign_roots_and_profiles() {
        use mizar_core::{
            core_ir::*,
            elaborator::{lower_source_theorem_skeletons, source_theorem_formula_payloads},
        };
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let invalid = "unsupported source theorem formula payload";
        for source in [
            original.to_owned(),
            "open theorem O: for X being set holds X = X;".to_owned(),
            original.replacen("theorem", "lemma", 1),
            original.replace("set", "object"),
            format!("{original} {}", original.replace("T:", "U:")),
            "theorem C: 0 = 0 by computation(steps: 8);".to_owned(),
            include_str!(
                "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
            ).to_owned(),
        ] {
            with_step5c14_theorem_check(&source, |check| {
                if source.starts_with("open theorem") {
                    let lower_error = lower_source_theorem_skeletons(check, None).unwrap_err();
                    assert_eq!(lower_error, "theorems.core.status_unsupported");
                    let foreign = step5c14_computation_core(
                        "theorem C: 0 = 0 by computation(steps: 8);",
                    )?;
                    assert!(foreign.proof_nodes().iter().any(|(_, node)| {
                        matches!(node.kind, CoreProofNodeKind::ComputationGoal { .. })
                    }));
                    let roots = foreign.obligation_seeds().iter().flat_map(|(_, seed)| {
                        seed.context.iter().copied().chain(seed.goal)
                    }).collect::<Vec<_>>();
                    assert!(!roots.is_empty());
                    for requested in [roots.as_slice(), &[]] {
                        assert_eq!(
                            source_theorem_formula_payloads(check, &foreign, requested).unwrap_err(),
                            lower_error
                        );
                    }
                    return Ok(());
                }
                let core = lower_source_theorem_skeletons(check, None)?;
                let before = core.clone();
                let roots = core.obligation_seeds().iter().flat_map(|(_, seed)| {
                    seed.context.iter().copied().chain(seed.goal)
                }).collect::<Vec<_>>();
                assert!(!roots.is_empty());
                if source != original {
                    if source.contains("lemma T:") {
                        assert_eq!(core.items().iter().next().unwrap().1.kind, CoreItemKind::Lemma);
                    }
                    if source.contains("being object") {
                        assert_eq!(core.items().len(), 1);
                        assert_eq!(core.proof_nodes().len(), 2);
                        assert_eq!(roots.len(), 2);
                        let CoreFormulaKind::TypePred { ty, .. } =
                            &core.formulas().get(roots[0]).unwrap().kind
                        else {
                            panic!()
                        };
                        assert_eq!(ty.as_str(), "object");
                        assert_eq!(
                            source_theorem_formula_payloads(check, &core, &roots[1..]).unwrap_err(),
                            invalid
                        );
                    }
                    assert_eq!(source_theorem_formula_payloads(check, &core, &roots).unwrap_err(), invalid);
                    assert_eq!(source_theorem_formula_payloads(check, &core, &[]).unwrap_err(), invalid);
                } else {
                    let proof = core.proofs().iter().next().unwrap().1;
                    let CoreFormulaKind::Forall { binders, body } =
                        &core.formulas().get(proof.proposition).unwrap().kind
                    else {
                        panic!()
                    };
                    for refused in [
                        vec![roots[0], roots[0]],
                        vec![CoreFormulaId::new(usize::MAX)],
                        vec![proof.proposition],
                        vec![binders[0].ty_guard.unwrap()],
                        vec![*body],
                        vec![roots[0], *body],
                    ] {
                        assert_eq!(source_theorem_formula_payloads(check, &core, &refused).unwrap_err(), invalid);
                    }
                    let foreign = step5c14_computation_core(&source.replace("T:", "Different:"))?;
                    for requested in [roots.as_slice(), &[]] {
                        assert_eq!(source_theorem_formula_payloads(check, &foreign, requested).unwrap_err(), invalid);
                    }
                    let mut parts = CoreIrParts {
                        source_id: core.source_id(),
                        module_id: core.module_id().clone(),
                        items: core.items().clone(),
                        terms: core.terms().clone(),
                        formulas: core.formulas().clone(),
                        definitions: core.definitions().clone(),
                        proofs: core.proofs().clone(),
                        proof_nodes: core.proof_nodes().clone(),
                        algorithms: core.algorithms().clone(),
                        algorithm_statements: core.algorithm_statements().clone(),
                        generated: core.generated().clone(),
                        obligation_seeds: core.obligation_seeds().clone(),
                        source_map: core.source_map().clone(),
                        diagnostics: core.diagnostics().clone(),
                    };
                    let CoreFormulaKind::Equals { left, .. } =
                        core.formulas().get(roots[1]).unwrap().kind
                    else {
                        panic!()
                    };
                    parts.terms.get_mut(left).unwrap().kind = CoreTermKind::Var(binders[0].var);
                    let mutated = CoreIr::try_new(parts).unwrap();
                    assert_ne!(mutated, core);
                    for requested in [roots.as_slice(), &[]] {
                        assert_eq!(source_theorem_formula_payloads(check, &mutated, requested).unwrap_err(), invalid);
                    }
                }
                assert_eq!(core, before);
                Ok(())
            }).unwrap();
        }
    }

    #[test]
    fn source_theorem_canonical_components_exact_content_and_edits() {
        use mizar_core::elaborator::{
            lower_source_theorem_skeletons, source_theorem_formula_payloads,
        };
        use mizar_vc::{
            generator::{CoreGenerationCandidateSet, CoreGenerationInput, VcNormalizationInput},
            vc_ir::*,
        };
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        // Independent frozen grammar vectors prepared before this API, not producer output.
        let vectors = [
            (
                VC_SCHEMA.to_owned(),
                GENERATION_SCHEMA.to_owned(),
                [
                    "0b1995287247ab14546781d55ed0bf31d69fb5059a99cb121ef58294dcdc466f",
                    "803374a93f4ee93612cf149c760849274ad5b2c9b3996c93fd38a84ecee63874",
                    "73e14c7080ab133d7635e954d0776de272ddd550a92395f04546f46796db5766",
                ],
            ),
            (
                format!("{VC_SCHEMA}-changed"),
                GENERATION_SCHEMA.to_owned(),
                [
                    "d1dacac4b96f867781b298e90217f090dcefa8c2821906b8c5cd4bb71761bf1e",
                    "f769d4420f35013b4fe2493ea72271f39985b5cc58a87b87d61c2498c8ed6eb4",
                    "d95ff07e4cfc0ee6ffc3d2804e3069c58e8cff06ecb4db7bb601b1ae0b1e62ee",
                ],
            ),
            (
                VC_SCHEMA.to_owned(),
                format!("{GENERATION_SCHEMA}-changed"),
                [
                    "4ba3de656ac4560c67537c459027a63d47b39e41067cdf1455b237901337e65f",
                    "4c17538a0e5331db9a12a7c9aeddf615bfc4cbfda6e707886c0287c2f0261183",
                    "e778cc5f716882654fd614030c761f56654537fe6ba00d4726efef5eff2ffad1",
                ],
            ),
        ];
        for source in [
            original.to_owned(),
            "theorem T: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;".to_owned(),
            format!(":: leading trivia\n{}\n", original.replace(" proof ", "\nproof\n")),
            original.replace("T:", "U:"),
        ] {
            with_step5c14_theorem_check(&source, |check| {
                let core = lower_source_theorem_skeletons(check, None)?;
                let core_before = core.clone();
                let seed = core.obligation_seeds().iter().next().unwrap().1;
                let roots = [seed.context[0], seed.goal.unwrap()];
                let formulas = source_theorem_formula_payloads(check, &core, &roots)?;
                let formulas_before = formulas.clone();
                let vcs = generate_core_vcs(&core, snapshot_id(777))?;
                let vcs_before = vcs.clone();
                let vc = &vcs.vcs()[0];
                let anchor = vc.anchor.clone();
                assert!(!anchor.is_complete());
                assert_eq!(vcs.canonical_vc_fingerprint(vc.id), None);
                assert_eq!(vcs.local_context_fingerprint(vc.id), None);
                let baseline = vcs.source_theorem_canonical_components(vc.id, &core, &formulas)?;
                assert_eq!(
                    baseline,
                    vcs.source_theorem_canonical_components(vc.id, &core, &formulas)?
                );
                let actual = [baseline.0.hash(), baseline.1, baseline.2.hash()].map(|hash| {
                    hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()
                });
                assert_eq!(actual, vectors[0].2);
                let fresh = generate_core_vcs(&core, snapshot_id(778))?;
                assert_ne!(fresh.snapshot(), vcs.snapshot());
                assert_eq!(
                    fresh.source_theorem_canonical_components(fresh.vcs()[0].id, &core, &formulas)?,
                    baseline
                );
                if source == original {
                    for (vc_schema, generation_schema, expected) in &vectors[1..] {
                        // Regenerate candidates and normalize with the actual alternate schemas.
                        let flow = mizar_core::control_flow::build_control_flow_ir(&core);
                        let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&core, &flow);
                        let intake = SeedIntakeTable::try_from_handoff(&handoff)
                            .map_err(|error| error.to_string())?;
                        let package = core.module_id().package().as_str();
                        let path = core.module_id().path().as_str();
                        let candidates = CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
                            schema_version: &GenerationSchemaVersion::new(generation_schema.clone()),
                            module: &VcModuleRef::new(format!(
                                "package={}:{};module={}:{}", package.len(), package, path.len(), path
                            )),
                            intake: &intake,
                            handoff: &handoff,
                            flow_output: Some(&flow),
                        }).map_err(|error| error.to_string())?;
                        let changed = CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
                            schema_version: &VcSchemaVersion::new(vc_schema.clone()),
                            snapshot: vcs.snapshot(),
                            source: core.source_id(),
                            candidates: &candidates,
                        }).map_err(|error| error.to_string())?;
                        let components = changed.source_theorem_canonical_components(
                            changed.vcs()[0].id, &core, &formulas
                        )?;
                        let hashes = [components.0.hash(), components.1, components.2.hash()];
                        for (hash, original) in hashes.iter().zip([
                            baseline.0.hash(), baseline.1, baseline.2.hash()
                        ]) {
                            assert_ne!(*hash, original);
                        }
                        let actual = hashes.map(|hash| {
                            hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()
                        });
                        assert_eq!(&actual, expected);
                    }
                }
                assert_eq!(vc.anchor, anchor);
                assert_eq!(core, core_before);
                assert_eq!(formulas, formulas_before);
                assert_eq!(vcs, vcs_before);
                Ok(())
            }).unwrap();
        }
    }

    #[test]
    fn source_theorem_canonical_components_refuse_maps_and_whole_inputs() {
        use mizar_core::{
            core_ir::*,
            elaborator::{lower_source_theorem_skeletons, source_theorem_formula_payloads},
        };
        use mizar_vc::{kernel_evidence_handoff::build_source_theorem_kernel_handoff, vc_ir::*};
        let source = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        with_step5c14_theorem_check(source, |check| {
            let core = lower_source_theorem_skeletons(check, None)?;
            let core_before = core.clone();
            let seed = core.obligation_seeds().iter().next().unwrap().1;
            let guard = seed.context[0];
            let goal = seed.goal.unwrap();
            let formulas = source_theorem_formula_payloads(check, &core, &[guard, goal])?;
            let formulas_before = formulas.clone();
            let vcs = generate_core_vcs(&core, snapshot_id(777))?;
            let vcs_before = vcs.clone();
            let selected = vcs.vcs()[0].id;
            let proof = core.proofs().iter().next().unwrap().1;
            let CoreFormulaKind::Forall { binders, .. } =
                &core.formulas().get(proof.proposition).unwrap().kind
            else {
                panic!()
            };
            let original_guard = binders[0].ty_guard.unwrap();
            assert_ne!(original_guard, guard);
            let invalid = "unsupported source theorem canonical components";
            for mutation in 0..7 {
                let mut changed = formulas.clone();
                match mutation {
                    0 => changed.clear(),
                    1 => {
                        changed.remove(&guard);
                    }
                    2 => {
                        changed.remove(&goal);
                    }
                    3 => {
                        changed.get_mut(&guard).unwrap().clear();
                    }
                    4 => {
                        changed.get_mut(&goal).unwrap().clear();
                    }
                    5 => {
                        changed.insert(original_guard, formulas[&guard].clone());
                    }
                    6 => {
                        let bytes = changed.remove(&guard).unwrap();
                        changed.insert(original_guard, bytes);
                    }
                    _ => unreachable!(),
                }
                assert_eq!(
                    vcs.source_theorem_canonical_components(selected, &core, &changed)
                        .unwrap_err(),
                    invalid,
                    "map mutation {mutation}"
                );
            }
            let foreign = step5c14_computation_core(&source.replace("T:", "Different:"))?;
            let foreign_vcs = generate_core_vcs(&foreign, vcs.snapshot())?;
            for (actual_core, actual_vcs, id) in [
                (&core, &vcs, VcId::new(usize::MAX)),
                (&foreign, &vcs, selected),
                (&core, &foreign_vcs, selected),
            ] {
                let upstream =
                    build_source_theorem_kernel_handoff(actual_core, actual_vcs, id).unwrap_err();
                for map in [&formulas, &std::collections::BTreeMap::new()] {
                    assert_eq!(
                        actual_vcs
                            .source_theorem_canonical_components(id, actual_core, map)
                            .unwrap_err(),
                        upstream
                    );
                }
            }
            for mutation in 0..10 {
                let mut parts = VcSetParts {
                    schema_version: vcs.schema_version().clone(),
                    snapshot: vcs.snapshot(),
                    source: vcs.source(),
                    module: vcs.module().clone(),
                    generated_formulas: vcs.generated_formulas().to_vec(),
                    vcs: vcs.vcs().to_vec(),
                    seed_accounting: vcs.seed_accounting().to_vec(),
                };
                let mut entries = parts.vcs[0].local_context.entries().to_vec();
                let mut policies = Vec::new();
                match mutation {
                    0 => parts.vcs[0].status = VcStatus::NeedsAtp,
                    1 => entries[0].kind = ContextEntryKind::CurrentThesis,
                    2 => entries[0].formula = Some(VcFormulaRef::Core(original_guard)),
                    3 => entries[0].provenance.clear(),
                    4 => {
                        entries[0].provenance[0].core.as_mut().unwrap().key =
                            "foreign-checker-role".into()
                    }
                    5 => entries[0].sort_key = CanonicalSortKey::new("foreign-source-ordinal"),
                    6 => policies.push(VerifierPolicyInput {
                        sort_key: CanonicalSortKey::new("foreign-policy"),
                        key: PolicyKey::new("foreign-policy"),
                        value: PolicyValue::new("enabled"),
                    }),
                    7 => parts.vcs[0].premises.clear(),
                    8 => {
                        parts.vcs[0].proof_hint = Some(ProofHint {
                            citations: Vec::new(),
                            unfold_requests: Vec::new(),
                            premise_restrictions: Vec::new(),
                            solver: None,
                            max_axioms: None,
                            timeout: None,
                            computation: None,
                            provenance: Vec::new(),
                        })
                    }
                    9 => parts.vcs[0].provenance.clear(),
                    _ => unreachable!(),
                }
                parts.vcs[0].local_context = LocalContext::try_new(entries, policies).unwrap();
                let changed = VcSet::try_new(parts).unwrap();
                assert_ne!(changed, vcs, "whole-input mutation {mutation}");
                let upstream =
                    build_source_theorem_kernel_handoff(&core, &changed, selected).unwrap_err();
                assert_eq!(
                    changed
                        .source_theorem_canonical_components(selected, &core, &formulas)
                        .unwrap_err(),
                    upstream,
                    "whole-input mutation {mutation}"
                );
            }
            // Explicitly malformed, source-unbacked negative: change only the seed provenance key.
            // Original Core56 bytes below are negative operands, not authenticated to this Core.
            let mut parts = CoreIrParts {
                source_id: core.source_id(),
                module_id: core.module_id().clone(),
                items: core.items().clone(),
                terms: core.terms().clone(),
                formulas: core.formulas().clone(),
                definitions: core.definitions().clone(),
                proofs: core.proofs().clone(),
                proof_nodes: core.proof_nodes().clone(),
                algorithms: core.algorithms().clone(),
                algorithm_statements: core.algorithm_statements().clone(),
                generated: core.generated().clone(),
                obligation_seeds: core.obligation_seeds().clone(),
                source_map: core.source_map().clone(),
                diagnostics: core.diagnostics().clone(),
            };
            let seed_id = core.obligation_seeds().iter().next().unwrap().0;
            let provenance = &mut parts.obligation_seeds.get_mut(seed_id).unwrap().provenance;
            assert_eq!(provenance.len(), 1);
            assert_eq!(provenance[0].phase, CoreProvenancePhase::Checker);
            provenance[0].key = "checker/theorem/foreign/skeleton/terminal".into();
            let malformed = CoreIr::try_new(parts).unwrap();
            assert_ne!(malformed, core);
            assert_eq!(malformed.source_map(), core.source_map());
            let malformed_vcs = generate_core_vcs(&malformed, vcs.snapshot())?;
            let id = malformed_vcs.vcs()[0].id;
            assert!(build_source_theorem_kernel_handoff(&malformed, &malformed_vcs, id).is_ok());
            assert_eq!(
                malformed_vcs
                    .source_theorem_canonical_components(id, &malformed, &formulas)
                    .unwrap_err(),
                invalid
            );
            assert_eq!(core, core_before);
            assert_eq!(formulas, formulas_before);
            assert_eq!(vcs, vcs_before);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn source_theorem_canonical_inputs_match_genuine_content_and_full_slices() {
        use mizar_core::elaborator::{
            lower_source_theorem_skeletons, source_theorem_formula_payloads,
        };
        use mizar_proof::status::source_theorem_canonical_inputs;
        use mizar_vc::{
            dependency_slice::{
                DependencySliceCompleteness, DependencySliceInput, try_compute_dependency_slices,
            },
            generator::{CoreGenerationCandidateSet, CoreGenerationInput, VcNormalizationInput},
            vc_ir::*,
        };
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        // Independent fixed-schema literals frozen before the facade implementation.
        let expected = [
            "770f0ab609b0f647adef9a5d0a96b8ca81c755bed1fa0c91e4bd9021e5625563",
            "e2753e241cf06d7bbc31e1886154a430469c93bcae47a724e4a21a77e46e45d9",
            "2d1ad8e890a278757aef06df254c0035e637e14ad636fa53e887fddb72db8aaa",
        ];
        for source in [
            original.to_owned(),
            "theorem T: for Bound being set holds Bound = Bound proof let Local be set; thus Local = Local; end;".to_owned(),
            format!(":: leading trivia\n{}\n", original.replace(" proof ", "\nproof\n")),
            original.replace("T:", "U:"),
        ] {
            with_step5c14_theorem_check(&source, |check| {
                let check_before = format!("{check:?}");
                let owners_before = check.owners().to_vec();
                assert_eq!(check.owners().len(), 1);
                assert_eq!(check.owners()[0].0.spelling, if source.contains("U:") { "U" } else { "T" });
                let core = lower_source_theorem_skeletons(check, None)?;
                let roots = core.obligation_seeds().iter().flat_map(|(_, seed)| {
                    seed.context.iter().copied().chain(seed.goal)
                }).collect::<Vec<_>>();
                let formulas = source_theorem_formula_payloads(check, &core, &roots)?;
                for ordinal in [777, 778] {
                    let snapshot = snapshot_id(ordinal);
                    // Independently generate the original graph, without relabeling a container.
                    let flow = mizar_core::control_flow::build_control_flow_ir(&core);
                    let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&core, &flow);
                    let intake = SeedIntakeTable::try_from_handoff(&handoff)
                        .map_err(|error| error.to_string())?;
                    let package = core.module_id().package().as_str();
                    let path = core.module_id().path().as_str();
                    let candidates = CoreGenerationCandidateSet::try_from_seed_intake(CoreGenerationInput {
                        schema_version: &GenerationSchemaVersion::new("source-theorem-local-v1"),
                        module: &VcModuleRef::new(format!(
                            "package={}:{};module={}:{}", package.len(), package, path.len(), path
                        )),
                        intake: &intake,
                        handoff: &handoff,
                        flow_output: Some(&flow),
                    }).map_err(|error| error.to_string())?;
                    let vcs = CoreGenerationCandidateSet::try_normalize(VcNormalizationInput {
                        schema_version: &VcSchemaVersion::new("vc-v1"),
                        snapshot,
                        source: core.source_id(),
                        candidates: &candidates,
                    }).map_err(|error| error.to_string())?;
                    assert_eq!(vcs.snapshot(), snapshot);
                    assert_eq!(vcs.vcs().len(), 1);
                    let vc = &vcs.vcs()[0];
                    let anchor_before = vc.anchor.clone();
                    assert_eq!(vc.status, VcStatus::Open);
                    assert_eq!(vc.kind, VcKind::TheoremProofStep);
                    assert!(!vc.anchor.is_complete());
                    assert_eq!(vcs.canonical_vc_fingerprint(vc.id), None);
                    assert_eq!(vcs.local_context_fingerprint(vc.id), None);
                    let components = vcs.source_theorem_canonical_components(vc.id, &core, &formulas)?;
                    let slices = try_compute_dependency_slices(DependencySliceInput {
                        vc_set: &vcs,
                        discharge_output: None,
                    }).map_err(|error| error.to_string())?;
                    let slice = slices.slice_for(vc.id).unwrap();
                    let actual = source_theorem_canonical_inputs(check, snapshot, vc.id)?;
                    assert_eq!((actual.0, actual.1, actual.2), components);
                    let hashes = [actual.0.hash(), actual.1, actual.2.hash()].map(|hash| {
                        hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()
                    });
                    assert_eq!(hashes, expected);
                    assert_eq!(&actual.3, slice);
                    assert_eq!(actual.3.vc(), slice.vc());
                    assert_eq!(actual.3.kind(), slice.kind());
                    assert_eq!(actual.3.status(), slice.status());
                    assert_eq!(actual.3.entries(), slice.entries());
                    assert_eq!(actual.3.unknowns(), slice.unknowns());
                    assert_eq!(actual.3.completeness(), slice.completeness());
                    assert_eq!(actual.3.fingerprint(), slice.fingerprint());
                    assert_eq!(actual.3.kind(), &VcKind::TheoremProofStep);
                    assert_eq!(actual.3.status(), &VcStatus::Open);
                    assert!(!actual.3.entries().is_empty());
                    assert!(!actual.3.unknowns().is_empty());
                    assert_eq!(actual.3.completeness(), DependencySliceCompleteness::IncompleteUncacheable);
                    assert!(!actual.3.is_complete());
                    assert!(actual.3.requires_cache_miss());
                    assert_eq!(actual, source_theorem_canonical_inputs(check, snapshot, vc.id)?);
                    assert_eq!(vc.anchor, anchor_before);
                }
                assert_eq!(check.owners(), owners_before);
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            }).unwrap();
        }
    }

    #[test]
    fn source_theorem_canonical_inputs_preserve_upstream_errors_before_selection() {
        use mizar_core::{
            core_ir::{CoreItemKind, CoreProofNodeKind},
            elaborator::{lower_source_theorem_skeletons, source_theorem_formula_payloads},
        };
        use mizar_proof::status::source_theorem_canonical_inputs;
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let absent = VcId::new(usize::MAX);
        with_step5c14_theorem_check(original, |check| {
            let check_before = format!("{check:?}");
            let core = lower_source_theorem_skeletons(check, None)?;
            let roots = core
                .obligation_seeds()
                .iter()
                .flat_map(|(_, seed)| seed.context.iter().copied().chain(seed.goal))
                .collect::<Vec<_>>();
            let formulas = source_theorem_formula_payloads(check, &core, &roots)?;
            // VC69 checks target absence before schema/replay, so the old generator is sufficient.
            let vcs = generate_core_vcs(&core, snapshot_id(777))?;
            let expected = vcs
                .source_theorem_canonical_components(absent, &core, &formulas)
                .unwrap_err();
            assert_eq!(
                source_theorem_canonical_inputs(check, snapshot_id(777), absent).unwrap_err(),
                expected
            );
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        for source in [
            "open theorem O: for X being set holds X = X;".to_owned(),
            original.replacen("theorem", "lemma", 1),
            "theorem C: 0 = 0 by computation(steps: 8);".to_owned(),
        ] {
            with_step5c14_theorem_check(&source, |check| {
                let check_before = format!("{check:?}");
                let expected = if source.starts_with("open theorem") {
                    let error = lower_source_theorem_skeletons(check, None).unwrap_err();
                    assert_eq!(error, "theorems.core.status_unsupported");
                    error
                } else {
                    let core = lower_source_theorem_skeletons(check, None)?;
                    if source.starts_with("lemma") {
                        assert_eq!(
                            core.items().iter().next().unwrap().1.kind,
                            CoreItemKind::Lemma
                        );
                    } else {
                        assert!(
                            core.obligation_seeds()
                                .iter()
                                .all(|(_, seed)| seed.context.is_empty())
                        );
                        assert!(core.proof_nodes().iter().any(|(_, node)| {
                            matches!(node.kind, CoreProofNodeKind::ComputationGoal { .. })
                        }));
                    }
                    let roots = core
                        .obligation_seeds()
                        .iter()
                        .flat_map(|(_, seed)| seed.context.iter().copied().chain(seed.goal))
                        .collect::<Vec<_>>();
                    assert!(!roots.is_empty());
                    source_theorem_formula_payloads(check, &core, &roots).unwrap_err()
                };
                for selected in [VcId::new(0), absent] {
                    assert_eq!(
                        source_theorem_canonical_inputs(check, snapshot_id(777), selected)
                            .unwrap_err(),
                        expected
                    );
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
        }
    }

    #[test]
    fn source_theorem_obligation_identity_exact_genuine_fields_and_slices() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_artifact::store::{CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersion};
        use mizar_proof::status::{
            source_theorem_canonical_inputs as canonical,
            source_theorem_obligation_identity as identity,
        };
        use mizar_vc::{dependency_slice::DependencySliceCompleteness, vc_ir::*};
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let expected = [
            "5a352e84b26834eb3cc0e54edd522604c625a953b3ba47f927cd762001c116c9",
            "8ba4633c3ed51f7af5c2744adba8c89fe6fb65da7fb01b506305fe4afbc884db",
            "d87df55e0d47933114a69f77e96d84c426c5b30369eb451834358f1c6b4b08a1",
        ];
        // Complete Resolver46 bytes and candidate literals frozen before this implementation.
        let expected_syntax = concat!(
            "6d697a61722d7265736f6c76652f736f757263652d7468656f72656d2d73796e7461782f7631000060000000",
            "0600020065000000077468656f72656d000000000002006400000001540000000000020066000000013a0000",
            "0000002c000000010037000000040002006500000003666f7200000000003800000003000200640000000158",
            "0000000000020065000000056265696e67000000000015000000010019000000010002006500000003736574",
            "000000000002006500000005686f6c647300000000002d00000003001c00000001001d000000010002006400",
            "000001580000000000020066000000013d00000000001c00000001001d000000010002006400000001580000",
            "0000006200000004000200650000000570726f6f6600000000003d0000000300020065000000036c65740000",
            "0000003e00000003000200640000000158000000000002006500000002626500000000001500000001001900",
            "00000100020065000000037365740000000000020066000000013b0000000000540000000300020065000000",
            "047468757300000000004000000001002c00000001002d00000003001c00000001001d000000010002006400",
            "000001580000000000020066000000013d00000000001c00000001001d000000010002006400000001580000",
            "000000020066000000013b000000000002006500000003656e640000000000020066000000013b00000000",
        ).as_bytes().chunks_exact(2).map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()
        }).collect::<Vec<_>>();
        for (label, digest) in [
            (
                "T",
                "9135ff7f3f377b4585f23b8677c68aa131190b4d10864bf4ecc4c623cb098d7a",
            ),
            (
                "U",
                "9135ff7f3f377b4511b939a5737fde9431190b4d10864bf48c80a6b4e18f3ff3",
            ),
        ] {
            let text = original.replace("theorem T:", &format!("theorem {label}:"));
            let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let symbols = collected.env();
            let ast = frontend.ast.as_ref().unwrap();
            let before = (source.clone(), collected.env().clone());
            with_source_theorem_check(ast, &source, symbols, |check| {
                let check_before = format!("{check:?}");
                let owner = format!(concat!(
                    "{{\"module_path\":\"owners\",\"namespace\":\"owners\",\"package_id\":\"owner-test\",",
                    "\"role\":\"theorem\",\"spelling\":\"{}\",\"status\":\"unmodified\",\"visibility\":\"public\"}}\n"
                ), label).into_bytes();
                let mut syntax = expected_syntax.clone();
                assert_eq!(syntax.len(), 527);
                assert_eq!(syntax[72], b'T');
                syntax[72] = label.as_bytes()[0];
                let syntax_rows = mizar_resolve::symbols::source_theorem_syntax_payloads_from_arena(&source, symbols)?;
                assert_eq!(syntax_rows.len(), 1);
                assert_eq!(syntax_rows.get(&check.owners()[0].0.symbol), Some(&syntax));
                let anchor = format!("source-theorem-candidate-anchor-v1:{digest}");
                for ordinal in [777, 778] {
                    let snapshot = snapshot_id(ordinal);
                    // Request the unchanged old producer FIRST; its entire real slice is the operand.
                    let old = canonical(check, snapshot, VcId::new(0))?;
                    let hashes = [old.0.hash(), old.1, old.2.hash(), old.3.fingerprint().hash()].map(|hash| {
                        hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()
                    });
                    assert_eq!(hashes[..3], expected);
                    let obligation = CanonicalJson::object([
                        ("obligation_anchor", CanonicalJson::string(&anchor)),
                        ("vc_fingerprint", CanonicalJson::string(expected[0])),
                        ("canonical_goal_hash", CanonicalJson::string(expected[1])),
                        ("local_context_fingerprint", CanonicalJson::string(expected[2])),
                        ("vc_local_dependency_slice_fingerprint", CanonicalJson::string(&hashes[3])),
                    ]).map_err(|error| error.to_string())?;
                    let fingerprint = CanonicalHashDomain::new(
                        HashClass::Interface, "mizar-proof/source-theorem-original-obligation", SchemaVersion::new(1, 0),
                    ).hash(&obligation, &[]);
                    let actual = identity(&source, check, symbols, snapshot, VcId::new(0))?;
                    assert_eq!(actual.0.obligation_id(), anchor);
                    assert_eq!(actual.0.obligation_anchor().as_str(), anchor);
                    assert_eq!(actual.0.obligation_fingerprint(), fingerprint);
                    assert_eq!(actual.0.vc_fingerprint(), old.0.hash());
                    assert_eq!(actual.0.local_context_fingerprint(), old.2.hash());
                    assert_eq!(actual.0.dependency_slice_fingerprint(), old.3.fingerprint().hash());
                    assert_eq!(actual.1, old.3);
                    assert_eq!(actual.1.vc(), VcId::new(0));
                    assert_eq!(actual.1.kind(), &VcKind::TheoremProofStep);
                    assert_eq!(actual.1.status(), &VcStatus::Open);
                    assert_eq!(actual.1.entries(), old.3.entries());
                    assert_eq!(actual.1.unknowns(), old.3.unknowns());
                    assert_eq!(actual.1.fingerprint(), old.3.fingerprint());
                    assert!(!actual.1.entries().is_empty());
                    assert!(!actual.1.unknowns().is_empty());
                    assert_eq!(actual.1.completeness(), DependencySliceCompleteness::IncompleteUncacheable);
                    assert!(!actual.1.is_complete());
                    assert!(actual.1.requires_cache_miss());
                    let raw = source_theorem_candidate_anchor_hash(&owner, &syntax, old.0, old.1, old.2)?;
                    assert_eq!(raw.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>(), digest);
                    for (owner_bytes, syntax_bytes) in [(&[][..], syntax.as_slice()), (owner.as_slice(), &[][..])] {
                        assert_eq!(source_theorem_candidate_anchor_hash(owner_bytes, syntax_bytes, old.0, old.1, old.2).unwrap_err(),
                            "unsupported source theorem candidate anchor");
                    }
                    assert_eq!(actual, identity(&source, check, symbols, snapshot, VcId::new(0))?);
                    assert_eq!(old, canonical(check, snapshot, VcId::new(0))?);
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            }).unwrap();
            assert_eq!((source, collected.env().clone()), before);
        }
    }

    #[test]
    fn source_theorem_obligation_identity_neutral_allocation_and_participating_edits() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_proof::status::{
            source_theorem_canonical_inputs as canonical,
            source_theorem_obligation_identity as identity,
        };
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let expected = "source-theorem-candidate-anchor-v1:9135ff7f3f377b4585f23b8677c68aa131190b4d10864bf4ecc4c623cb098d7a";
        let mut prior = Vec::new();
        let mut original_source = None;
        for (text, package, module, skip, neutral) in [
            (original.to_owned(), "owner-test", "owners", 0, true),
            (
                format!(
                    ":: 日本語 λ\n{}\n",
                    original.replace(" proof ", "\nproof\n")
                ),
                "owner-test",
                "owners",
                0,
                true,
            ),
            (original.to_owned(), "owner-test", "owners", 7, true),
            (original.to_owned(), "other-package", "owners", 0, false),
            (original.to_owned(), "owner-test", "other_module", 0, false),
            (original.to_owned(), "owner-test", "nested.owners", 0, false),
            (
                original.replace("theorem T:", "theorem U:"),
                "owner-test",
                "owners",
                0,
                false,
            ),
            (
                original.replace(
                    "for X being set holds X = X",
                    "for Bound being set holds Bound = Bound",
                ),
                "owner-test",
                "owners",
                0,
                false,
            ),
            (
                original.replace(
                    "let X be set; thus X = X",
                    "let Local be set; thus Local = Local",
                ),
                "owner-test",
                "owners",
                0,
                false,
            ),
        ] {
            let (frontend, source, collected) = inputs(&text, package, module, skip);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let symbols = collected.env();
            let ast = frontend.ast.as_ref().unwrap();
            let before = (source.clone(), collected.env().clone(), prior.clone());
            if skip == 7 {
                assert_ne!(Some(source.source_id()), original_source);
            } else if original_source.is_none() {
                original_source = Some(source.source_id());
            }
            with_source_theorem_check(ast, &source, symbols, |check| {
                let check_before = format!("{check:?}");
                for ordinal in [777, 778] {
                    let snapshot = snapshot_id(ordinal);
                    let old = canonical(check, snapshot, VcId::new(0))?;
                    let actual = identity(&source, check, symbols, snapshot, VcId::new(0))?;
                    if neutral {
                        assert_eq!(actual.0.obligation_anchor().as_str(), expected);
                    } else {
                        assert_ne!(actual.0.obligation_anchor().as_str(), expected);
                    }
                    assert_eq!(actual.1, old.3);
                    assert_eq!(actual.0.vc_fingerprint(), old.0.hash());
                    assert_eq!(actual.0.local_context_fingerprint(), old.2.hash());
                    assert_eq!(
                        actual.0.dependency_slice_fingerprint(),
                        old.3.fingerprint().hash()
                    );
                    assert_eq!(old, canonical(check, snapshot, VcId::new(0))?);
                    prior.push(actual);
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((&source, collected.env()), (&before.0, &before.1));
            assert_eq!(&prior[..before.2.len()], before.2);
        }
    }

    #[test]
    fn source_theorem_obligation_identity_preserve_upstream_errors_and_inputs() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_checker::type_checker::SourceVariableSemanticsChecker;
        use mizar_proof::status::{
            source_theorem_canonical_inputs as canonical,
            source_theorem_obligation_identity as identity,
        };
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let snapshot = snapshot_id(777);
        let absent = VcId::new(usize::MAX);
        for text in [
            original.to_owned(),
            original.replacen("theorem", "lemma", 1),
            "open theorem O: for X being set holds X = X;".to_owned(),
            format!(
                "{original}\n{}",
                original.replace("theorem T:", "theorem U:")
            ),
        ] {
            let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let symbols = collected.env();
            let ast = frontend.ast.as_ref().unwrap();
            let before = (source.clone(), collected.env().clone());
            with_source_theorem_check(ast, &source, symbols, |check| {
                let check_before = format!("{check:?}");
                if text == original {
                    let old = canonical(check, snapshot, VcId::new(0))?;
                    let prior = identity(&source, check, symbols, snapshot, VcId::new(0))?;
                    assert_eq!(
                        identity(&source, check, symbols, snapshot, absent).unwrap_err(),
                        canonical(check, snapshot, absent).unwrap_err()
                    );
                    for text in [
                        original.replace("theorem T:", "theorem Foreign:"),
                        format!(
                            "{original}\n{}",
                            original.replace("theorem T:", "theorem U:")
                        ),
                        "theorem T: for X being set holds X = ;".to_owned(),
                    ] {
                        let (other_frontend, other_source, other) =
                            inputs(&text, "owner-test", "owners", 0);
                        assert_eq!(
                            !other_frontend.diagnostics.is_empty(),
                            text.ends_with("= ;")
                        );
                        let other_before = (other_source.clone(), other.env().clone());
                        for (current, symbols) in [
                            (&other_source, other.env()),
                            (&source, other.env()),
                            (&other_source, collected.env()),
                        ] {
                            let expected = SourceVariableSemanticsChecker::check_theorem_skeletons(
                                current,
                                check.typed_ast(),
                                check.scope(),
                                symbols,
                                check.labels(),
                                check.resolved_labels(),
                                None,
                            )
                            .unwrap_err();
                            for selected in [VcId::new(0), absent] {
                                assert_eq!(
                                    identity(current, check, symbols, snapshot, selected)
                                        .unwrap_err(),
                                    expected
                                );
                            }
                        }
                        assert_eq!((other_source, other.env().clone()), other_before);
                    }
                    assert_eq!(old, canonical(check, snapshot, VcId::new(0))?);
                    assert_eq!(
                        prior,
                        identity(&source, check, symbols, snapshot, VcId::new(0))?
                    );
                } else {
                    for selected in [VcId::new(0), absent] {
                        let expected = canonical(check, snapshot, selected).unwrap_err();
                        assert_eq!(
                            identity(&source, check, symbols, snapshot, selected).unwrap_err(),
                            expected
                        );
                    }
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((source, collected.env().clone()), before);
        }
    }

    #[test]
    fn source_theorem_reflexivity_checks_genuine_original_targets() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_core::{
            control_flow::{build_control_flow_ir, build_obligation_seed_handoff},
            elaborator::lower_source_theorem_skeletons as lower,
        };
        use mizar_kernel::{
            checker::{KernelCheckStatus, KernelEvidenceCheckKind},
            rejection::TargetVcFingerprint,
        };
        use mizar_proof::{
            policy::{CandidatePolicyClass, VerifierPolicy},
            status::{
                check_source_theorem_reflexivity as verify,
                source_theorem_canonical_inputs as canonical,
                source_theorem_obligation_identity as identity,
            },
        };
        use mizar_vc::{
            generator::{
                CoreGenerationCandidateSet as Candidates, CoreGenerationInput as Generation,
                VcNormalizationInput as Normalization, generate_source_theorem_vcs as generate,
            },
            kernel_evidence_handoff::build_source_theorem_kernel_handoff as project,
            vc_ir::*,
        };
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let mut original_source = None;
        let mut prior = Vec::new();
        for (text, package, module, skip) in [
            (original.to_owned(), "owner-test", "owners", 0),
            (
                original.replace(
                    "for X being set holds X = X",
                    "for Bound being set holds Bound = Bound",
                ),
                "owner-test",
                "owners",
                0,
            ),
            (
                original.replace(
                    "let X be set; thus X = X",
                    "let Local be set; thus Local = Local",
                ),
                "owner-test",
                "owners",
                0,
            ),
            (
                format!(":: 日本語 λ\n{original}\n"),
                "owner-test",
                "owners",
                0,
            ),
            (original.to_owned(), "owner-test", "owners", 7),
            (original.to_owned(), "other-package", "owners", 0),
            (original.to_owned(), "owner-test", "nested.owners", 0),
        ] {
            let (frontend, source, collected) = inputs(&text, package, module, skip);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let ast = frontend.ast.as_ref().unwrap();
            let symbols = collected.env();
            let before = (source.clone(), symbols.clone(), prior.clone());
            if skip == 7 {
                assert_ne!(Some(source.source_id()), original_source);
            } else if original_source.is_none() {
                original_source = Some(source.source_id());
            }
            with_source_theorem_check(ast, &source, symbols, |check| {
                let check_before = format!("{check:?}");
                let core = lower(check, None)?;
                let generation_schema = GenerationSchemaVersion::new("source-theorem-local-v1");
                let vc_schema = VcSchemaVersion::new("vc-v1");
                for ordinal in [777, 778] {
                    let snapshot = snapshot_id(ordinal);
                    // Independent original generic construction, with the Proof schemas.
                    let flow = build_control_flow_ir(&core);
                    let handoff = build_obligation_seed_handoff(&core, &flow);
                    let intake = SeedIntakeTable::try_from_handoff(&handoff)
                        .map_err(|error| error.to_string())?;
                    let package = core.module_id().package().as_str();
                    let path = core.module_id().path().as_str();
                    let module = VcModuleRef::new(format!(
                        "package={}:{};module={}:{}",
                        package.len(),
                        package,
                        path.len(),
                        path
                    ));
                    let candidates = Candidates::try_from_seed_intake(Generation {
                        schema_version: &generation_schema,
                        module: &module,
                        intake: &intake,
                        handoff: &handoff,
                        flow_output: Some(&flow),
                    })
                    .map_err(|error| error.to_string())?;
                    let expected = Candidates::try_normalize(Normalization {
                        schema_version: &vc_schema,
                        snapshot,
                        source: core.source_id(),
                        candidates: &candidates,
                    })
                    .map_err(|error| error.to_string())?;
                    let graph = generate(&core, snapshot, &generation_schema, &vc_schema)?;
                    assert_eq!(graph, expected);
                    assert_eq!(graph.vcs().len(), 1);
                    let vc = &expected.vcs()[0];
                    assert_eq!(vc.status, VcStatus::Open);
                    let original_handoff = project(&core, &expected, vc.id)?;
                    let target = original_handoff.canonical_evidence().target_vc();
                    let target =
                        TargetVcFingerprint::new(target.algorithm_id, target.digest.clone());
                    let policy = VerifierPolicy::default();
                    let old = canonical(check, snapshot, vc.id)?;
                    let old_identity = identity(&source, check, symbols, snapshot, vc.id)?;
                    assert_eq!(old_identity.1, old.3);
                    let immutable = (
                        core.clone(),
                        graph.clone(),
                        expected.clone(),
                        policy.clone(),
                    );
                    let actual = verify(&source, check, symbols, snapshot, vc.id, &policy)?;
                    assert_eq!(actual.0.status(), KernelCheckStatus::Accepted);
                    assert_eq!(actual.0.target_vc_fingerprint(), &target);
                    assert_eq!(
                        actual.0.evidence_check_kind(),
                        Some(KernelEvidenceCheckKind::ProofObligation)
                    );
                    assert!(!actual.0.policy_taint());
                    assert!(actual.0.used_axioms().is_empty());
                    assert_eq!(actual.1.class, CandidatePolicyClass::DischargedBuiltin);
                    assert_eq!(
                        actual,
                        verify(&source, check, symbols, snapshot, vc.id, &policy)?
                    );
                    assert_eq!(old, canonical(check, snapshot, vc.id)?);
                    assert_eq!(
                        old_identity,
                        identity(&source, check, symbols, snapshot, vc.id)?
                    );
                    assert_eq!(
                        (&core, &graph, &expected, &policy),
                        (&immutable.0, &immutable.1, &immutable.2, &immutable.3)
                    );
                    prior.push(actual);
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((&source, symbols), (&before.0, &before.1));
            assert_eq!(&prior[..before.2.len()], before.2);
        }
    }

    #[test]
    fn source_theorem_reflexivity_enforces_actual_builtin_policy_formats() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_kernel::checker::KernelCheckStatus;
        use mizar_proof::{
            policy::{CandidatePolicyClass, KernelEvidenceFormat, VerifierPolicy},
            status::check_source_theorem_reflexivity as verify,
        };
        use mizar_vc::vc_ir::VcId;
        let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let (frontend, source, collected) = inputs(text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let symbols = collected.env();
        let before = (source.clone(), symbols.clone());
        with_source_theorem_check(ast, &source, symbols, |check| {
            let check_before = format!("{check:?}");
            for policy in [
                VerifierPolicy::default(),
                VerifierPolicy::release().with_require_kernel_certificates(true),
                VerifierPolicy::development()
                    .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]),
                VerifierPolicy::default().with_kernel_evidence_formats([]),
                VerifierPolicy::release()
                    .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
            ] {
                let before = policy.clone();
                let actual = verify(
                    &source,
                    check,
                    symbols,
                    snapshot_id(777),
                    VcId::new(0),
                    &policy,
                );
                if policy
                    .kernel_evidence_formats()
                    .contains(&KernelEvidenceFormat::BuiltinKernelEvidence)
                {
                    let (result, decision) = actual?;
                    assert_eq!(result.status(), KernelCheckStatus::Accepted);
                    assert_eq!(decision.class, CandidatePolicyClass::DischargedBuiltin);
                } else {
                    assert_eq!(actual.unwrap_err(), "theorem.proof.kernel_policy_rejected");
                }
                assert_eq!(policy, before);
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, symbols.clone()), before);
    }

    #[test]
    fn source_theorem_reflexivity_preserves_upstream_errors_and_inputs() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_checker::type_checker::SourceVariableSemanticsChecker as Checker;
        use mizar_core::elaborator::lower_source_theorem_skeletons as lower;
        use mizar_proof::{
            policy::VerifierPolicy, status::check_source_theorem_reflexivity as verify,
        };
        use mizar_vc::{
            kernel_evidence_handoff::build_source_theorem_kernel_handoff as project, vc_ir::VcId,
        };
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let snapshot = snapshot_id(777);
        let absent = VcId::new(usize::MAX);
        let disabled = VerifierPolicy::default().with_kernel_evidence_formats([]);
        let policy_before = disabled.clone();
        let cited = include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        );
        // The unchanged alternate-schema generator supplies only VC69 error oracles here;
        // no alternate-schema graph supplies an expected successful target.
        for text in [
            original.to_owned(),
            original.replacen("theorem", "lemma", 1),
            "open theorem O: for X being set holds X = X;".to_owned(),
            format!(
                "{original}\n{}",
                original.replace("theorem T:", "theorem U:")
            ),
            cited.to_owned(),
        ] {
            let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let ast = frontend.ast.as_ref().unwrap();
            let symbols = collected.env();
            let before = (source.clone(), symbols.clone());
            with_source_theorem_check(ast, &source, symbols, |check| {
                let check_before = format!("{check:?}");
                if text == original {
                    let policy = VerifierPolicy::default();
                    let prior = verify(&source, check, symbols, snapshot, VcId::new(0), &policy)?;
                    let prior_before = prior.clone();
                    let core = lower(check, None)?;
                    let vcs = generate_core_vcs(&core, snapshot)?;
                    assert!(!vcs.vcs().is_empty());
                    let expected = project(&core, &vcs, absent).unwrap_err();
                    assert_eq!(
                        verify(&source, check, symbols, snapshot, absent, &disabled).unwrap_err(),
                        expected
                    );
                    for text in [
                        original.replace("theorem T:", "theorem Foreign:"),
                        format!(
                            "{original}\n{}",
                            original.replace("theorem T:", "theorem U:")
                        ),
                        "theorem T: for X being set holds X = ;".to_owned(),
                    ] {
                        let (other_frontend, other_source, other) =
                            inputs(&text, "owner-test", "owners", 0);
                        assert_eq!(
                            !other_frontend.diagnostics.is_empty(),
                            text.ends_with("= ;")
                        );
                        let other_before = (other_source.clone(), other.env().clone());
                        for (current, symbols) in [
                            (&other_source, other.env()),
                            (&source, other.env()),
                            (&other_source, collected.env()),
                        ] {
                            let expected = Checker::check_theorem_skeletons(
                                current,
                                check.typed_ast(),
                                check.scope(),
                                symbols,
                                check.labels(),
                                check.resolved_labels(),
                                None,
                            )
                            .unwrap_err();
                            for selected in [VcId::new(0), absent] {
                                assert_eq!(
                                    verify(current, check, symbols, snapshot, selected, &disabled)
                                        .unwrap_err(),
                                    expected
                                );
                            }
                        }
                        assert_eq!((other_source, other.env().clone()), other_before);
                    }
                    assert_eq!(prior, prior_before);
                    assert_eq!(
                        prior,
                        verify(&source, check, symbols, snapshot, VcId::new(0), &policy)?
                    );
                } else {
                    let core = lower(check, None);
                    let expected = match &core {
                        Err(error) => error.clone(),
                        Ok(core) => {
                            let vcs = generate_core_vcs(core, snapshot)?;
                            assert!(!vcs.vcs().is_empty());
                            assert!(vcs.vc(VcId::new(0)).is_some());
                            project(core, &vcs, VcId::new(0)).unwrap_err()
                        }
                    };
                    assert_eq!(
                        verify(&source, check, symbols, snapshot, VcId::new(0), &disabled)
                            .unwrap_err(),
                        expected
                    );
                    let expected = match &core {
                        Err(error) => error.clone(),
                        Ok(core) => {
                            project(core, &generate_core_vcs(core, snapshot)?, absent).unwrap_err()
                        }
                    };
                    assert_eq!(
                        verify(&source, check, symbols, snapshot, absent, &disabled).unwrap_err(),
                        expected
                    );
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((source, symbols.clone()), before);
        }
        assert_eq!(disabled, policy_before);
    }

    #[test]
    fn source_theorem_reflexivity_selection_matches_fresh_actual_evidence() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_kernel::checker::{KernelCheckStatus as Status, KernelEvidenceCheckKind as Kind};
        use mizar_proof::{
            policy::{
                CandidatePolicyClass as Class, KernelEvidenceFormat as Format,
                KernelEvidenceOrigin as Origin, KernelPolicyInput as Input,
                VerifierPolicy as Policy,
            },
            selection::{
                CandidateSourceId as Id, ProofEvidenceCandidate as Candidate,
                ProofEvidenceSet as Set, ProofWinnerClass as Winner,
                ProofWitnessPublication as Publication, select_winner,
            },
            status::{
                check_source_theorem_reflexivity as verify,
                select_source_theorem_reflexivity as select,
            },
        };
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let mut original_source = None;
        let mut prior = Vec::new();
        let bound = original.replace(
            "for X being set holds X = X",
            "for Bound being set holds Bound = Bound",
        );
        let local = original.replace(
            "let X be set; thus X = X",
            "let Local be set; thus Local = Local",
        );
        let trivia = format!(":: 日本語 λ\n{original}\n");
        for (index, (text, package, module, skip)) in [
            (original.to_owned(), "owner-test", "owners", 0),
            (bound, "owner-test", "owners", 0),
            (local, "owner-test", "owners", 0),
            (trivia, "owner-test", "owners", 0),
            (original.to_owned(), "owner-test", "owners", 7),
            (original.to_owned(), "other-package", "owners", 0),
            (original.to_owned(), "owner-test", "nested.owners", 0),
        ]
        .into_iter()
        .enumerate()
        {
            let (frontend, s, collected) = inputs(&text, package, module, skip);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (s.clone(), env.clone(), prior.clone());
            if skip == 7 {
                assert_ne!(Some(s.source_id()), original_source);
            } else if original_source.is_none() {
                original_source = Some(s.source_id());
            }
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &s, env, |check| {
                let check_before = format!("{check:?}");
                for (policy_index, policy) in [
                    Policy::default(),
                    Policy::release().with_require_kernel_certificates(true),
                    Policy::development()
                        .with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                    Policy::default().with_profile_id("theorem-selection"),
                    Policy::default().with_schema_version(2),
                    Policy::default().with_checker_schema_version(2),
                ]
                .into_iter()
                .take(if index == 0 { 6 } else { 1 })
                .enumerate()
                {
                    let snap = snapshot_id(777 + usize::from(index != 0 || policy_index != 0));
                    let vc = VcId::new(0);
                    let policy_before = policy.clone();
                    let raw = verify(&s, check, env, snap, vc, &policy)?;
                    let raw_before = raw.clone();
                    assert_eq!(raw.0.status(), Status::Accepted);
                    assert_eq!(raw.0.evidence_check_kind(), Some(Kind::ProofObligation));
                    assert!(!raw.0.policy_taint());
                    assert!(raw.0.used_axioms().is_empty());
                    assert_eq!(raw.1.class, Class::DischargedBuiltin);
                    let target = raw.0.target_vc_fingerprint().sort_bytes();
                    let hex: String = target.iter().map(|byte| format!("{byte:02x}")).collect();
                    let id = Id::new(format!("source-theorem-reflexivity-v1:{hex}")).unwrap();
                    let input = Input::from_kernel_result(&raw.0, Origin::BuiltinDischarge);
                    let candidate =
                        Candidate::from_trusted_kernel_input(id.clone(), &input).unwrap();
                    let expected = select_winner(
                        &Set::new(target, None, policy.clone()).with_candidates([candidate]),
                    );
                    let chosen = select(&s, check, env, snap, vc, &policy)?;
                    let chosen_before = chosen.clone();
                    assert_eq!(chosen, expected);
                    assert_eq!(chosen.selected_candidate_id(), Some(&id));
                    assert_eq!(chosen.selected_class(), Winner::DischargedBuiltin);
                    assert!(chosen.trusted_used_axioms_available());
                    let meta = chosen.reuse_metadata();
                    assert_eq!(meta.policy_fingerprint(), policy.policy_fingerprint());
                    let hash = input.accepted_evidence_hash().unwrap();
                    let polarity = input.accepted_goal_polarity().unwrap();
                    assert_eq!(meta.selected_evidence_hash(), Some(hash));
                    assert_eq!(meta.accepted_goal_polarity(), Some(polarity));
                    assert_eq!(meta.encoded_problem_hash(), None);
                    assert_eq!(meta.selected_candidate_provenance_hash(), None);
                    assert_eq!(meta.deterministic_discharge_hash(), None);
                    assert_eq!(meta.selected_proof_witness_hash(), None);
                    assert_eq!(
                        meta.proof_witness_publication(),
                        Publication::ExternalDependencyGap
                    );
                    assert_eq!(chosen, select(&s, check, env, snap, vc, &policy)?);
                    assert_eq!(raw, verify(&s, check, env, snap, vc, &policy)?);
                    assert_eq!(
                        (&raw, &chosen, &policy),
                        (&raw_before, &chosen_before, &policy_before)
                    );
                    prior.push((raw, chosen));
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((&s, env), (&before.0, &before.1));
            assert_eq!(&prior[..before.2.len()], before.2);
        }
    }

    #[test]
    fn source_theorem_reflexivity_selection_preserves_actual_failures() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_proof::{
            policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
            status::{
                check_source_theorem_reflexivity as verify,
                select_source_theorem_reflexivity as select,
            },
        };
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let snap = snapshot_id(777);
        let absent = VcId::new(usize::MAX);
        let policies = [
            Policy::default(),
            Policy::default().with_kernel_evidence_formats([]),
            Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
        ];
        let policies_before = policies.clone();
        let cited = include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        );
        for text in [
            original.to_owned(),
            original.replacen("theorem", "lemma", 1),
            "open theorem O: for X being set holds X = X;".to_owned(),
            format!(
                "{original}\n{}",
                original.replace("theorem T:", "theorem U:")
            ),
            cited.to_owned(),
        ] {
            let (frontend, s, collected) = inputs(&text, "owner-test", "owners", 0);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (s.clone(), env.clone());
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &s, env, |check| {
                let check_before = format!("{check:?}");
                for (index, policy) in policies.iter().enumerate() {
                    for vc in [VcId::new(0), absent] {
                        if text == original && index == 0 && vc != absent {
                            continue;
                        }
                        let expected = verify(&s, check, env, snap, vc, policy).unwrap_err();
                        assert_eq!(
                            select(&s, check, env, snap, vc, policy).unwrap_err(),
                            expected
                        );
                    }
                }
                if text == original {
                    let vc = VcId::new(0);
                    let policy = &policies[0];
                    let prior = (
                        verify(&s, check, env, snap, vc, policy)?,
                        select(&s, check, env, snap, vc, policy)?,
                    );
                    let prior_before = prior.clone();
                    for text in [
                        original.replace("theorem T:", "theorem Foreign:"),
                        format!(
                            "{original}\n{}",
                            original.replace("theorem T:", "theorem U:")
                        ),
                        "theorem T: for X being set holds X = ;".to_owned(),
                    ] {
                        let (other_frontend, other_source, other) =
                            inputs(&text, "owner-test", "owners", 0);
                        assert_eq!(
                            !other_frontend.diagnostics.is_empty(),
                            text.ends_with("= ;")
                        );
                        let other_before = (other_source.clone(), other.env().clone());
                        for (current, env) in [
                            (&other_source, other.env()),
                            (&s, other.env()),
                            (&other_source, collected.env()),
                        ] {
                            for vc in [VcId::new(0), absent] {
                                let expected = verify(current, check, env, snap, vc, &policies[1])
                                    .unwrap_err();
                                assert_eq!(
                                    select(current, check, env, snap, vc, &policies[1])
                                        .unwrap_err(),
                                    expected
                                );
                            }
                        }
                        assert_eq!((other_source, other.env().clone()), other_before);
                    }
                    assert_eq!(prior, prior_before);
                    assert_eq!(
                        prior,
                        (
                            verify(&s, check, env, snap, vc, policy)?,
                            select(&s, check, env, snap, vc, policy)?
                        )
                    );
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((s, env.clone()), before);
        }
        assert_eq!(policies, policies_before);
    }

    #[test]
    fn source_theorem_reflexivity_status_matches_existing_projection_and_full_slice() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_proof::{
            policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
            selection::{
                ProofWinnerClass as Winner, VcProofSelection,
                merge_artifact_proof_selections as merge,
            },
            status::{
                ArtifactPublicationGap as Gap, ArtifactStatusPublication as Publication,
                ProjectedProofStatus as Status, ProofStatusProjectionInput as Input,
                project_source_theorem_reflexivity_status as project, project_status,
                select_source_theorem_reflexivity as select,
                source_theorem_obligation_identity as identity,
            },
        };
        use mizar_vc::{
            dependency_slice::DependencySliceCompleteness as Completeness,
            vc_ir::{VcId, VcKind, VcStatus},
        };
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let bound = original.replace(
            "for X being set holds X = X",
            "for Bound being set holds Bound = Bound",
        );
        let local = original.replace(
            "let X be set; thus X = X",
            "let Local be set; thus Local = Local",
        );
        let trivia = format!(":: 日本語 λ\n{original}\n");
        let mut original_source = None;
        let mut prior = Vec::new();
        for (index, (text, package, module, skip)) in [
            (original.to_owned(), "owner-test", "owners", 0),
            (bound, "owner-test", "owners", 0),
            (local, "owner-test", "owners", 0),
            (trivia, "owner-test", "owners", 0),
            (original.to_owned(), "owner-test", "owners", 7),
            (original.to_owned(), "other-package", "owners", 0),
            (original.to_owned(), "owner-test", "nested.owners", 0),
        ]
        .into_iter()
        .enumerate()
        {
            let (frontend, source, collected) = inputs(&text, package, module, skip);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (source.clone(), env.clone(), prior.clone());
            if skip == 7 {
                assert_ne!(Some(source.source_id()), original_source);
            } else if original_source.is_none() {
                original_source = Some(source.source_id());
            }
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
                let check_before = format!("{check:?}");
                for (policy_index, policy) in [
                    Policy::default(),
                    Policy::release().with_require_kernel_certificates(true),
                    Policy::development()
                        .with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                    Policy::default().with_profile_id("theorem-status"),
                    Policy::default().with_schema_version(2),
                    Policy::default().with_checker_schema_version(2),
                ]
                .into_iter()
                .take(if index == 0 { 6 } else { 1 })
                .enumerate()
                {
                    let snap = snapshot_id(777 + usize::from(index != 0 || policy_index != 0));
                    let vc = VcId::new(0);
                    let policy_before = policy.clone();
                    let original = identity(&source, check, env, snap, vc)?;
                    let chosen = select(&source, check, env, snap, vc, &policy)?;
                    let inputs_before = (original.clone(), chosen.clone());
                    assert_eq!(chosen.selected_class(), Winner::DischargedBuiltin);
                    assert!(chosen.trusted_used_axioms_available());
                    let [merged] = <[_; 1]>::try_from(
                        merge([], [VcProofSelection::new(vc, chosen.clone())]).unwrap(),
                    )
                    .unwrap();
                    let expected =
                        project_status(Input::new(merged, policy.clone(), original.0.clone()))
                            .unwrap();
                    let actual = project(&source, check, env, snap, vc, &policy)?;
                    let actual_before = actual.clone();
                    assert_eq!(actual.0, expected);
                    assert_eq!(actual.0.vc(), vc);
                    assert_eq!(actual.0.obligation_id(), original.0.obligation_id());
                    assert_eq!(actual.0.selected_class(), Winner::DischargedBuiltin);
                    assert_eq!(actual.0.projected_status(), Status::Accepted);
                    assert_eq!(
                        actual.0.artifact_publication(),
                        Publication::ExternalDependencyGap(
                            Gap::DischargedBuiltinWitnessUnsupported
                        )
                    );
                    assert_eq!(actual.0.accepted_witness_obligation_id(), None);
                    assert_eq!(actual.0.trusted_used_axioms(), None);
                    assert_eq!(actual.0.explanation_ref(), None);
                    let meta = actual.0.reuse_metadata();
                    let id = &original.0;
                    let selected = chosen.reuse_metadata();
                    assert_eq!(meta.obligation_anchor(), id.obligation_anchor());
                    assert_eq!(meta.obligation_fingerprint(), id.obligation_fingerprint());
                    assert_eq!(meta.vc_fingerprint(), id.vc_fingerprint());
                    assert_eq!(
                        meta.local_context_fingerprint(),
                        id.local_context_fingerprint()
                    );
                    assert_eq!(
                        meta.dependency_slice_fingerprint(),
                        id.dependency_slice_fingerprint()
                    );
                    assert_eq!(meta.policy_fingerprint(), policy.policy_fingerprint());
                    assert_eq!(meta.selected_candidate_id(), chosen.selected_candidate_id());
                    assert_eq!(
                        meta.selected_evidence_hash(),
                        selected.selected_evidence_hash()
                    );
                    assert!(meta.selected_evidence_hash().is_some());
                    assert_eq!(
                        meta.accepted_goal_polarity(),
                        selected.accepted_goal_polarity()
                    );
                    assert!(meta.accepted_goal_polarity().is_some());
                    assert_eq!(meta.trusted_used_axioms_hash(), None);
                    assert_eq!(meta.selected_proof_witness_hash(), None);
                    assert_eq!(meta.deterministic_discharge_hash(), None);
                    assert_eq!(meta.selected_candidate_provenance_hash(), None);
                    assert_eq!(meta.dependency_compatibility(), None);
                    assert_eq!(meta.explanation_ref(), None);
                    assert!(!meta.cache_reuse_predicate_complete());
                    assert_eq!(actual.1, original.1);
                    assert_eq!(actual.1.vc(), vc);
                    assert_eq!(actual.1.kind(), &VcKind::TheoremProofStep);
                    assert_eq!(actual.1.status(), &VcStatus::Open);
                    assert_eq!(actual.1.entries(), original.1.entries());
                    assert!(!actual.1.entries().is_empty());
                    assert_eq!(actual.1.unknowns(), original.1.unknowns());
                    assert!(!actual.1.unknowns().is_empty());
                    assert_eq!(actual.1.completeness(), Completeness::IncompleteUncacheable);
                    assert_eq!(actual.1.fingerprint(), original.1.fingerprint());
                    assert_eq!(
                        actual.1.fingerprint().hash(),
                        meta.dependency_slice_fingerprint()
                    );
                    assert!(actual.1.requires_cache_miss());
                    assert_eq!(actual, project(&source, check, env, snap, vc, &policy)?);
                    assert_eq!(original, identity(&source, check, env, snap, vc)?);
                    assert_eq!(chosen, select(&source, check, env, snap, vc, &policy)?);
                    assert_eq!((&original, &chosen), (&inputs_before.0, &inputs_before.1));
                    assert_eq!((&actual, &policy), (&actual_before, &policy_before));
                    prior.push(actual);
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((&source, env), (&before.0, &before.1));
            assert_eq!(&prior[..before.2.len()], before.2);
        }
    }

    #[test]
    fn source_theorem_reflexivity_status_preserves_actual_error_order() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_proof::{
            policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
            status::{
                project_source_theorem_reflexivity_status as project,
                select_source_theorem_reflexivity as select,
                source_theorem_obligation_identity as identity,
            },
        };
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let snap = snapshot_id(777);
        let absent = VcId::new(usize::MAX);
        let policies = [
            Policy::default(),
            Policy::default().with_kernel_evidence_formats([]),
            Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
        ];
        let policies_before = policies.clone();
        let cited = include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        );
        for (case, text) in [
            original.to_owned(),
            original.replacen("theorem", "lemma", 1),
            "open theorem O: for X being set holds X = X;".to_owned(),
            format!(
                "{original}\n{}",
                original.replace("theorem T:", "theorem U:")
            ),
            cited.to_owned(),
        ]
        .into_iter()
        .enumerate()
        {
            let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (source.clone(), env.clone());
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
                let check_before = format!("{check:?}");
                for (policy_index, policy) in policies.iter().enumerate() {
                    for vc in [VcId::new(0), absent] {
                        let actual_identity = identity(&source, check, env, snap, vc);
                        let expected = match actual_identity {
                            Err(error) => {
                                let selection_error =
                                    select(&source, check, env, snap, vc, policy).unwrap_err();
                                if case == 3 {
                                    assert_eq!(error, "unsupported source theorem formula payload");
                                    assert_eq!(
                                        selection_error,
                                        "theorem.handoff.invalid_projection"
                                    );
                                    assert_ne!(error, selection_error);
                                }
                                error
                            }
                            Ok(original) => {
                                assert_eq!(case, 0);
                                assert_eq!(vc, VcId::new(0));
                                assert!(original.1.requires_cache_miss());
                                if policy_index == 0 {
                                    continue;
                                }
                                let error =
                                    select(&source, check, env, snap, vc, policy).unwrap_err();
                                assert_eq!(original, identity(&source, check, env, snap, vc)?);
                                error
                            }
                        };
                        assert_eq!(
                            project(&source, check, env, snap, vc, policy).unwrap_err(),
                            expected
                        );
                    }
                }
                if case == 0 {
                    let vc = VcId::new(0);
                    let original_identity = identity(&source, check, env, snap, vc)?;
                    let prior = project(&source, check, env, snap, vc, &policies[0])?;
                    let prior_before = prior.clone();
                    for text in [
                        original.replace("theorem T:", "theorem Foreign:"),
                        format!(
                            "{original}\n{}",
                            original.replace("theorem T:", "theorem U:")
                        ),
                        "theorem T: for X being set holds X = ;".to_owned(),
                    ] {
                        let (other_frontend, other_source, other) =
                            inputs(&text, "owner-test", "owners", 0);
                        assert_eq!(
                            !other_frontend.diagnostics.is_empty(),
                            text.ends_with("= ;")
                        );
                        let other_before = (other_source.clone(), other.env().clone());
                        for (current, symbols) in [
                            (&other_source, other.env()),
                            (&source, other.env()),
                            (&other_source, env),
                        ] {
                            for selected in [vc, absent] {
                                let expected =
                                    identity(current, check, symbols, snap, selected).unwrap_err();
                                select(current, check, symbols, snap, selected, &policies[1])
                                    .unwrap_err();
                                assert_eq!(
                                    project(current, check, symbols, snap, selected, &policies[1])
                                        .unwrap_err(),
                                    expected
                                );
                            }
                        }
                        assert_eq!((other_source, other.env().clone()), other_before);
                    }
                    assert_eq!(original_identity, identity(&source, check, env, snap, vc)?);
                    assert_eq!(prior, project(&source, check, env, snap, vc, &policies[0])?);
                    assert_eq!(prior, prior_before);
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((source, env.clone()), before);
        }
        assert_eq!(policies, policies_before);
    }

    #[test]
    fn source_theorem_reflexivity_metadata_matches_actual_wire_and_fields() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_artifact::{
            proof_witness::KernelAcceptanceMetadata,
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
            store::SchemaVersion,
        };
        use mizar_core::elaborator::lower_source_theorem_skeletons as lower;
        use mizar_kernel::{
            certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
            checker::{KernelCheckStatus as Status, KernelEvidenceCheckKind as Kind},
            formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        };
        use mizar_proof::{
            policy::{
                AcceptedGoalPolarity, CandidatePolicyClass as Class,
                KernelEvidenceFormat as Format, KernelEvidenceOrigin as Origin, KernelPolicyInput,
                VerifierPolicy as Policy,
            },
            status::{
                check_source_theorem_reflexivity as verify,
                check_source_theorem_reflexivity_metadata as metadata,
                project_source_theorem_reflexivity_status as status,
                select_source_theorem_reflexivity as select,
            },
        };
        use mizar_vc::{
            generator::generate_source_theorem_vcs as generate,
            kernel_evidence_handoff::{
                KernelClauseTautologyPolicy, build_source_theorem_kernel_handoff as project,
            },
            vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion, VcStatus},
        };
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let bound = original.replace(
            "for X being set holds X = X",
            "for Bound being set holds Bound = Bound",
        );
        let local = original.replace(
            "let X be set; thus X = X",
            "let Local be set; thus Local = Local",
        );
        let trivia = format!(":: 日本語 λ\n{original}\n");
        let mut original_source = None;
        let mut prior = Vec::new();
        for (index, (text, package, module, skip)) in [
            (original.to_owned(), "owner-test", "owners", 0),
            (bound, "owner-test", "owners", 0),
            (local, "owner-test", "owners", 0),
            (trivia, "owner-test", "owners", 0),
            (original.to_owned(), "owner-test", "owners", 7),
            (original.to_owned(), "other-package", "owners", 0),
            (original.to_owned(), "owner-test", "nested.owners", 0),
        ]
        .into_iter()
        .enumerate()
        {
            let (frontend, source, collected) = inputs(&text, package, module, skip);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (source.clone(), env.clone(), prior.clone());
            if skip == 7 {
                assert_ne!(Some(source.source_id()), original_source);
            } else if original_source.is_none() {
                original_source = Some(source.source_id());
            }
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
                let check_before = format!("{check:?}");
                for policy in [
                    Policy::default(),
                    Policy::release().with_require_kernel_certificates(true),
                    Policy::development()
                        .with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                    Policy::default().with_profile_id("theorem-metadata"),
                    Policy::default().with_schema_version(2),
                    Policy::default().with_checker_schema_version(2),
                ]
                .into_iter()
                .take(if index == 0 { 6 } else { 1 })
                {
                    for ordinal in [777, 778] {
                        let snap = snapshot_id(ordinal);
                        let vc = VcId::new(0);
                        let policy_before = policy.clone();
                        let raw = verify(&source, check, env, snap, vc, &policy)?;
                        let raw_before = raw.clone();
                        let core = lower(check, None)?;
                        let graph = generate(
                            &core,
                            snap,
                            &GenerationSchemaVersion::new("source-theorem-local-v1"),
                            &VcSchemaVersion::new("vc-v1"),
                        )?;
                        assert_eq!(graph.vcs()[0].status, VcStatus::Open);
                        let handoff = project(&core, &graph, vc)?;
                        let envelope = handoff.canonical_evidence();
                        let target = envelope.target_vc();
                        let profile = envelope.kernel_profile();
                        let tautology = match profile.clause_tautology_policy {
                            KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                            KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                            _ => panic!("unsupported actual theorem profile"),
                        };
                        let mut context = FormulaEvidenceParseContext::v1(
                            Fingerprint::new(target.algorithm_id, target.digest.clone()),
                            KernelProfileRecord::v1(profile.profile_id, tautology),
                        );
                        let v1 = handoff.encode_local_evidence(None)?;
                        let parsed = parse_formula_evidence(&v1, &context).unwrap();
                        assert_eq!(parsed.schema_version(), 1);
                        let names =
                            mizar_atp::backend::encode_local_variable_reflexivity_names(&parsed)?;
                        assert!(!names.is_empty());
                        let wire = handoff.encode_local_evidence(Some(&names))?;
                        assert_ne!(wire, v1);
                        context.accepted_schema_versions.insert(2);
                        let named = parse_formula_evidence(&wire, &context).unwrap();
                        assert_eq!(named.schema_version(), 2);
                        assert_eq!(named.equality_instances().len(), names.len());
                        for name in &names {
                            assert!(wire.windows(name.len()).any(|bytes| bytes == name));
                        }
                        let components = handoff.local_evidence_component_hashes()?;
                        let input =
                            KernelPolicyInput::from_kernel_result(&raw.0, Origin::BuiltinDischarge);
                        assert_eq!(raw.0.status(), Status::Accepted);
                        assert_eq!(raw.0.evidence_check_kind(), Some(Kind::ProofObligation));
                        assert!(!raw.0.policy_taint());
                        assert_eq!(raw.1.class, Class::DischargedBuiltin);
                        assert_eq!(
                            input.accepted_goal_polarity(),
                            Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                        );
                        let accepted = input.accepted_evidence_hash().unwrap();
                        let families = [
                            "mizar-vc/local-evidence-component/profile",
                            "mizar-vc/local-evidence-component/formula",
                            "mizar-vc/local-evidence-component/substitutions",
                            "mizar-vc/local-evidence-component/provenance",
                            "mizar-proof/verifier-policy",
                            "mizar-vc/kernel-evidence-handoff-canonical",
                            "mizar-proof/kernel-accepted-evidence",
                        ];
                        let digests = [
                            components[0],
                            components[1],
                            components[2],
                            components[3],
                            policy.policy_fingerprint().hash(),
                            handoff.canonical_hash(),
                            accepted,
                        ];
                        let [
                            profile,
                            formula,
                            substitutions,
                            provenance,
                            verifier,
                            target,
                            result,
                        ] = std::array::from_fn(|i| ArtifactHashRef {
                            class: ArtifactHashClass::Interface,
                            schema_family: families[i].into(),
                            schema_version: SchemaVersion::new(
                                if i == 4 { policy.schema_version() } else { 1 },
                                0,
                            ),
                            digest: digests[i],
                        });
                        let expected = KernelAcceptanceMetadata {
                            kernel_profile_fingerprint: profile,
                            verifier_policy_fingerprint: verifier,
                            checker_schema_version: SchemaVersion::new(
                                policy.checker_schema_version(),
                                0,
                            ),
                            evidence_schema_version: SchemaVersion::new(named.schema_version(), 0),
                            target_binding_hash: target,
                            formula_evidence_hash: formula,
                            substitution_evidence_hash: substitutions,
                            provenance_hash: provenance,
                            formula_context_hash: None,
                            accepted_result_hash: result,
                        };
                        let chosen = select(&source, check, env, snap, vc, &policy)?;
                        let projected = status(&source, check, env, snap, vc, &policy)?;
                        let old_before = (chosen.clone(), projected.clone());
                        let actual = metadata(&source, check, env, snap, vc, &policy)?;
                        assert_eq!((&actual.0, &actual.1), (&raw.0, &raw.1));
                        assert_eq!(actual.2.as_ref(), Some(&expected));
                        assert_eq!(actual.3, wire);
                        assert_eq!(parse_formula_evidence(&actual.3, &context).unwrap(), named);
                        assert_eq!(
                            chosen.reuse_metadata().selected_evidence_hash(),
                            Some(accepted)
                        );
                        assert!(projected.1.requires_cache_miss());
                        assert!(
                            !projected
                                .0
                                .reuse_metadata()
                                .cache_reuse_predicate_complete()
                        );
                        assert_eq!(actual, metadata(&source, check, env, snap, vc, &policy)?);
                        assert_eq!(raw, verify(&source, check, env, snap, vc, &policy)?);
                        assert_eq!(chosen, select(&source, check, env, snap, vc, &policy)?);
                        assert_eq!(projected, status(&source, check, env, snap, vc, &policy)?);
                        assert_eq!((chosen, projected), old_before);
                        assert_eq!(raw, raw_before);
                        assert_eq!(policy, policy_before);
                        prior.push(actual);
                    }
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((&source, env), (&before.0, &before.1));
            assert_eq!(&prior[..before.2.len()], before.2);
        }
    }

    #[test]
    fn source_theorem_reflexivity_metadata_preserves_compatibility_errors() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_proof::{
            policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
            status::{
                check_source_theorem_reflexivity as verify,
                check_source_theorem_reflexivity_metadata as metadata,
            },
        };
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let snap = snapshot_id(777);
        let absent = VcId::new(usize::MAX);
        let policies = [
            Policy::default(),
            Policy::default().with_kernel_evidence_formats([]),
            Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
        ];
        let policies_before = policies.clone();
        let cited = include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        );
        for text in [
            original.to_owned(),
            original.replacen("theorem", "lemma", 1),
            "open theorem O: for X being set holds X = X;".to_owned(),
            format!(
                "{original}\n{}",
                original.replace("theorem T:", "theorem U:")
            ),
            cited.to_owned(),
        ] {
            let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (source.clone(), env.clone());
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
                let check_before = format!("{check:?}");
                for (index, policy) in policies.iter().enumerate() {
                    for vc in [VcId::new(0), absent] {
                        if text == original && index == 0 && vc != absent {
                            continue;
                        }
                        let expected = verify(&source, check, env, snap, vc, policy).unwrap_err();
                        assert_eq!(
                            metadata(&source, check, env, snap, vc, policy).unwrap_err(),
                            expected
                        );
                    }
                }
                if text == original {
                    let vc = VcId::new(0);
                    let policy = &policies[0];
                    let prior = (
                        verify(&source, check, env, snap, vc, policy)?,
                        metadata(&source, check, env, snap, vc, policy)?,
                    );
                    let prior_before = prior.clone();
                    for text in [
                        original.replace("theorem T:", "theorem Foreign:"),
                        format!(
                            "{original}\n{}",
                            original.replace("theorem T:", "theorem U:")
                        ),
                        "theorem T: for X being set holds X = ;".to_owned(),
                    ] {
                        let (other_frontend, other_source, other) =
                            inputs(&text, "owner-test", "owners", 0);
                        assert_eq!(
                            !other_frontend.diagnostics.is_empty(),
                            text.ends_with("= ;")
                        );
                        let other_before = (other_source.clone(), other.env().clone());
                        for (current, symbols) in [
                            (&other_source, other.env()),
                            (&source, other.env()),
                            (&other_source, env),
                        ] {
                            for policy in &policies {
                                for selected in [vc, absent] {
                                    let expected =
                                        verify(current, check, symbols, snap, selected, policy)
                                            .unwrap_err();
                                    assert_eq!(
                                        metadata(current, check, symbols, snap, selected, policy)
                                            .unwrap_err(),
                                        expected
                                    );
                                }
                            }
                        }
                        assert_eq!((other_source, other.env().clone()), other_before);
                    }
                    assert_eq!(prior, prior_before);
                    assert_eq!(
                        prior,
                        (
                            verify(&source, check, env, snap, vc, policy)?,
                            metadata(&source, check, env, snap, vc, policy)?,
                        )
                    );
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((source, env.clone()), before);
        }
        assert_eq!(policies, policies_before);
    }
    #[test]
    fn source_theorem_reflexivity_witness_matches_original_identity_and_checked_wire() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_artifact::{
            proof_witness::{
                EvidenceKind, ProofStatus, ProofWitnessReadOptions, ProofWitnessRef,
                builtin_proof_witness_ref_json, read_builtin_proof_witness_ref,
            },
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
            store::{CanonicalHashDomain as Domain, CanonicalJson as J, HashClass, SchemaVersion},
        };
        use mizar_core::elaborator::lower_source_theorem_skeletons as lower;
        use mizar_kernel::{
            certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
            checker::{KernelCheckStatus as Status, KernelEvidenceCheckKind as Kind},
            formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        };
        use mizar_proof::selection::{
            CandidateSourceId as Id, ProofEvidenceCandidate as Candidate, ProofEvidenceSet as Set,
            ProofWinnerClass, ProofWitnessPublication, select_winner,
        };
        use mizar_proof::{
            policy::{
                AcceptedGoalPolarity, CandidatePolicyClass as Class,
                KernelEvidenceFormat as Format, KernelEvidenceOrigin as Origin, KernelPolicyInput,
                VerifierPolicy as Policy,
            },
            status::{
                check_source_theorem_reflexivity as verify,
                check_source_theorem_reflexivity_metadata as metadata,
                project_source_theorem_reflexivity_status as status,
                project_source_theorem_reflexivity_witness as witness,
                select_source_theorem_reflexivity as select,
                source_theorem_canonical_inputs as canonical,
                source_theorem_obligation_identity as identity,
            },
        };
        use mizar_vc::{
            generator::generate_source_theorem_vcs as generate,
            kernel_evidence_handoff::{
                KernelClauseTautologyPolicy, build_source_theorem_kernel_handoff as project,
            },
            vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion, VcStatus},
        };
        let id = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let bound = id.replace(
            "for X being set holds X = X",
            "for Bound being set holds Bound = Bound",
        );
        let local = id.replace(
            "let X be set; thus X = X",
            "let Local be set; thus Local = Local",
        );
        let trivia = format!(":: 日本語 λ\n{id}\n");
        let mut original_source = None;
        let mut prior = Vec::new();
        for (index, (text, package, module, skip)) in [
            (id.to_owned(), "owner-test", "owners", 0),
            (bound, "owner-test", "owners", 0),
            (local, "owner-test", "owners", 0),
            (trivia, "owner-test", "owners", 0),
            (id.to_owned(), "owner-test", "owners", 7),
            (id.to_owned(), "other-package", "owners", 0),
            (id.to_owned(), "owner-test", "nested.owners", 0),
        ]
        .into_iter()
        .enumerate()
        {
            let (frontend, s, collected) = inputs(&text, package, module, skip);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let e = collected.env();
            let before = (s.clone(), e.clone(), prior.clone());
            if skip == 7 {
                assert_ne!(Some(s.source_id()), original_source);
            } else if original_source.is_none() {
                original_source = Some(s.source_id());
            }
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &s, e, |c| {
                let check_before = format!("{c:?}");
                for p in [
                    Policy::default(),
                    Policy::release().with_require_kernel_certificates(true),
                    Policy::development()
                        .with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                    Policy::default().with_profile_id("theorem-metadata"),
                    Policy::default().with_schema_version(2),
                    Policy::default().with_checker_schema_version(2),
                ]
                .into_iter()
                .take(if index == 0 { 6 } else { 1 })
                {
                    for ordinal in [777, 778] {
                        let sn = snapshot_id(ordinal);
                        let vc = VcId::new(0);
                        let policy_before = p.clone();
                        let id = identity(&s, c, e, sn, vc)?;
                        let content = canonical(c, sn, vc)?;
                        let r = verify(&s, c, e, sn, vc, &p)?;
                        let old = metadata(&s, c, e, sn, vc, &p)?;
                        assert_eq!((&old.0, &old.1), (&r.0, &r.1));
                        let core = lower(c, None)?;
                        let graph = generate(
                            &core,
                            sn,
                            &GenerationSchemaVersion::new("source-theorem-local-v1"),
                            &VcSchemaVersion::new("vc-v1"),
                        )?;
                        assert_eq!(graph.vcs()[0].status, VcStatus::Open);
                        let handoff = project(&core, &graph, vc)?;
                        let envelope = handoff.canonical_evidence();
                        let target = envelope.target_vc();
                        let profile = envelope.kernel_profile();
                        let tautology = match profile.clause_tautology_policy {
                            KernelClauseTautologyPolicy::Reject => ClauseTautologyPolicy::Reject,
                            KernelClauseTautologyPolicy::Marker => ClauseTautologyPolicy::Marker,
                            _ => panic!("unsupported actual theorem profile"),
                        };
                        let mut context = FormulaEvidenceParseContext::v1(
                            Fingerprint::new(target.algorithm_id, target.digest.clone()),
                            KernelProfileRecord::v1(profile.profile_id, tautology),
                        );
                        let v1 = handoff.encode_local_evidence(None)?;
                        let parsed = parse_formula_evidence(&v1, &context).unwrap();
                        assert_eq!(parsed.schema_version(), 1);
                        let names =
                            mizar_atp::backend::encode_local_variable_reflexivity_names(&parsed)?;
                        assert!(!names.is_empty());
                        let wire = handoff.encode_local_evidence(Some(&names))?;
                        assert_ne!(wire, v1);
                        context.accepted_schema_versions.insert(2);
                        let named = parse_formula_evidence(&wire, &context).unwrap();
                        assert_eq!(named.schema_version(), 2);
                        assert_eq!(named.equality_instances().len(), names.len());
                        for name in &names {
                            assert!(wire.windows(name.len()).any(|bytes| bytes == name));
                        }
                        let input =
                            KernelPolicyInput::from_kernel_result(&r.0, Origin::BuiltinDischarge);
                        assert_eq!(r.0.status(), Status::Accepted);
                        assert_eq!(r.0.evidence_check_kind(), Some(Kind::ProofObligation));
                        assert!(!r.0.policy_taint());
                        assert_eq!(r.1.class, Class::DischargedBuiltin);
                        assert_eq!(
                            input.accepted_goal_polarity(),
                            Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                        );
                        let accepted = input.accepted_evidence_hash().unwrap();
                        let m = old.2.clone().unwrap();
                        assert_eq!(m.accepted_result_hash.digest, accepted);
                        assert_eq!(m.target_binding_hash.digest, handoff.canonical_hash());
                        assert_eq!(
                            m.checker_schema_version,
                            SchemaVersion::new(p.checker_schema_version(), 0)
                        );
                        assert_eq!(
                            m.evidence_schema_version,
                            SchemaVersion::new(named.schema_version(), 0)
                        );
                        assert_eq!(m.formula_context_hash, None);
                        let choice = select(&s, c, e, sn, vc, &p)?;
                        let state = status(&s, c, e, sn, vc, &p)?;
                        assert_eq!(id.0.obligation_id(), id.0.obligation_anchor().as_str());
                        assert_eq!(id.0.vc_fingerprint(), content.0.hash());
                        assert_eq!(id.0.local_context_fingerprint(), content.2.hash());
                        assert_eq!(
                            id.0.dependency_slice_fingerprint(),
                            content.3.fingerprint().hash()
                        );
                        assert_eq!(id.1, content.3);
                        assert!(!id.1.entries().is_empty());
                        assert!(!id.1.unknowns().is_empty());
                        assert!(!id.1.is_complete());
                        assert!(id.1.requires_cache_miss());
                        let checked_target = mizar_kernel::rejection::TargetVcFingerprint::new(
                            target.algorithm_id,
                            target.digest.clone(),
                        );
                        let target = r.0.target_vc_fingerprint().sort_bytes();
                        let target_hex: String =
                            target.iter().map(|b| format!("{b:02x}")).collect();
                        let candidate_id =
                            Id::new(format!("source-theorem-reflexivity-v1:{target_hex}")).unwrap();
                        let candidate =
                            Candidate::from_trusted_kernel_input(candidate_id, &input).unwrap();
                        let expected_selection = select_winner(
                            &Set::new(target, None, p.clone()).with_candidates([candidate]),
                        );
                        assert_eq!(choice, expected_selection);
                        assert_eq!(choice.selected_class(), ProofWinnerClass::DischargedBuiltin);
                        let meta = choice.reuse_metadata();
                        assert_eq!(
                            meta.accepted_goal_polarity(),
                            input.accepted_goal_polarity()
                        );
                        assert_eq!(meta.encoded_problem_hash(), None);
                        assert_eq!(meta.selected_candidate_provenance_hash(), None);
                        assert_eq!(meta.deterministic_discharge_hash(), None);
                        assert_eq!(meta.selected_proof_witness_hash(), None);
                        assert_eq!(
                            meta.proof_witness_publication(),
                            ProofWitnessPublication::ExternalDependencyGap
                        );
                        let payload_family =
                            "mizar-proof/source-theorem-reflexivity-witness-payload";
                        let wire_hex: String = wire.iter().map(|b| format!("{b:02x}")).collect();
                        let payload_hash = ArtifactHashRef {
                            class: ArtifactHashClass::Artifact,
                            schema_family: payload_family.into(),
                            schema_version: SchemaVersion::new(1, 0),
                            digest: Domain::new(
                                HashClass::Artifact,
                                payload_family,
                                SchemaVersion::new(1, 0),
                            )
                            .hash(&J::string(wire_hex), &[]),
                        };
                        let identity_hashes = [
                            id.0.obligation_fingerprint(),
                            handoff.context_identity_hash(),
                        ]
                        .map(|h| {
                            h.as_bytes()
                                .iter()
                                .map(|b| format!("{b:02x}"))
                                .collect::<String>()
                        });
                        let [original_hash, local_context] = identity_hashes;
                        let keys = [
                            "original_obligation_fingerprint",
                            "checked_target_bytes",
                            "local_context_identity_hash",
                            "kernel_profile_fingerprint",
                            "verifier_policy_fingerprint",
                            "checker_schema_version",
                            "evidence_schema_version",
                            "target_binding_hash",
                            "formula_evidence_hash",
                            "substitution_evidence_hash",
                            "provenance_hash",
                            "formula_context_hash",
                            "accepted_result_hash",
                            "accepted_goal_polarity",
                            "witness_artifact_hash",
                        ];
                        let values = [
                            original_hash,
                            target_hex,
                            local_context,
                            m.kernel_profile_fingerprint.to_artifact_hash_string(),
                            m.verifier_policy_fingerprint.to_artifact_hash_string(),
                            m.checker_schema_version.to_string(),
                            m.evidence_schema_version.to_string(),
                            m.target_binding_hash.to_artifact_hash_string(),
                            m.formula_evidence_hash.to_artifact_hash_string(),
                            m.substitution_evidence_hash.to_artifact_hash_string(),
                            m.provenance_hash.to_artifact_hash_string(),
                            m.accepted_result_hash.to_artifact_hash_string(),
                            input.accepted_goal_polarity().unwrap().as_str().into(),
                            payload_hash.to_artifact_hash_string(),
                        ]
                        .map(J::string);
                        let mut values = values.into_iter().collect::<Vec<_>>();
                        values.insert(11, J::null());
                        let object = J::object(keys.into_iter().zip(values)).unwrap();
                        let family = "mizar-proof/source-theorem-reflexivity-obligation";
                        let expected_ref = ProofWitnessRef {
                            schema_version: SchemaVersion::new(3, 0),
                            obligation_id: id.0.obligation_id().into(),
                            obligation_fingerprint: ArtifactHashRef {
                                class: ArtifactHashClass::Interface,
                                schema_family: family.into(),
                                schema_version: SchemaVersion::new(1, 0),
                                digest: Domain::new(
                                    HashClass::Interface,
                                    family,
                                    SchemaVersion::new(1, 0),
                                )
                                .hash(&object, &[]),
                            },
                            proof_status: ProofStatus::DischargedBuiltin,
                            evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
                            witness_path: format!(
                                "proof-witnesses/source-theorem-reflexivity-v1/{}.bin",
                                payload_hash
                                    .digest
                                    .as_bytes()
                                    .iter()
                                    .map(|b| format!("{b:02x}"))
                                    .collect::<String>()
                            ),
                            witness_artifact_hash: payload_hash,
                            kernel_acceptance: m.clone(),
                        };
                        assert_ne!(
                            expected_ref.obligation_fingerprint.digest,
                            id.0.obligation_fingerprint()
                        );
                        assert_ne!(handoff.context_identity_hash(), handoff.canonical_hash());
                        assert_eq!(r.0.target_vc_fingerprint(), &checked_target);
                        assert_eq!(old.3, wire);
                        let actual = witness(&s, c, e, sn, vc, &p)?;
                        assert_eq!(actual, (expected_selection, expected_ref, wire.clone()));
                        let json = builtin_proof_witness_ref_json(&actual.1).unwrap();
                        assert_eq!(
                            read_builtin_proof_witness_ref(
                                &json,
                                ProofWitnessReadOptions::default()
                            )
                            .unwrap(),
                            actual.1
                        );
                        assert_eq!(actual, witness(&s, c, e, sn, vc, &p)?);
                        assert_eq!(id, identity(&s, c, e, sn, vc)?);
                        assert_eq!(old, metadata(&s, c, e, sn, vc, &p)?);
                        assert_eq!(r, verify(&s, c, e, sn, vc, &p)?);
                        assert_eq!(choice, select(&s, c, e, sn, vc, &p)?);
                        assert_eq!(state, status(&s, c, e, sn, vc, &p)?);
                        assert_eq!(state.1, id.1);
                        assert!(!state.0.reuse_metadata().cache_reuse_predicate_complete());
                        assert_eq!(p, policy_before);
                        prior.push(actual);
                    }
                }
                assert_eq!(format!("{c:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((&s, e), (&before.0, &before.1));
            assert_eq!(&prior[..before.2.len()], before.2);
        }
    }
    #[test]
    fn source_theorem_reflexivity_witness_hashes_bind_all_participants() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_artifact::{
            registration_summary::{ArtifactHashClass, ArtifactHashRef},
            store::{CanonicalHashDomain as Domain, CanonicalJson as J, HashClass, SchemaVersion},
        };
        use mizar_kernel::{
            certificate_parser::{ClauseTautologyPolicy, Fingerprint, KernelProfileRecord},
            formula_evidence::{FormulaEvidenceParseContext, parse_formula_evidence},
        };
        use mizar_proof::{
            policy::{KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
            status::{
                check_source_theorem_reflexivity_metadata as metadata,
                project_source_theorem_reflexivity_witness as witness,
                source_theorem_obligation_identity as identity,
            },
        };
        use mizar_vc::{
            generator::generate_source_theorem_vcs,
            kernel_evidence_handoff::build_source_theorem_kernel_handoff,
            vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
        };
        let text = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let (frontend, s, collected) = inputs(text, "owner-test", "owners", 0);
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &s, collected.env(), |c| {
            let sn = snapshot_id(777);
            let vc = VcId::new(0);
            let p = VerifierPolicy::default();
            let id = identity(&s, c, collected.env(), sn, vc)?;
            let old = metadata(&s, c, collected.env(), sn, vc, &p)?;
            let accepted = old.2.as_ref().unwrap();
            let core = mizar_core::elaborator::lower_source_theorem_skeletons(c, None)?;
            let graph = generate_source_theorem_vcs(
                &core,
                sn,
                &GenerationSchemaVersion::new("source-theorem-local-v1"),
                &VcSchemaVersion::new("vc-v1"),
            )?;
            let handoff = build_source_theorem_kernel_handoff(&core, &graph, vc)?;
            let input = KernelPolicyInput::from_kernel_result(
                &old.0,
                KernelEvidenceOrigin::BuiltinDischarge,
            );
            let payload_family = "mizar-proof/source-theorem-reflexivity-witness-payload";
            let payload_domain = Domain::new(
                HashClass::Artifact,
                payload_family,
                SchemaVersion::new(1, 0),
            );
            let wire_hex: String = old.3.iter().map(|b| format!("{b:02x}")).collect();
            let payload = ArtifactHashRef {
                class: ArtifactHashClass::Artifact,
                schema_family: payload_family.into(),
                schema_version: SchemaVersion::new(1, 0),
                digest: payload_domain.hash(&J::string(wire_hex), &[]),
            };
            let identity_hashes = [
                id.0.obligation_fingerprint(),
                handoff.context_identity_hash(),
            ]
            .map(|h| {
                h.as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            });
            let [original_hash, local_context] = identity_hashes;
            let keys = [
                "original_obligation_fingerprint",
                "checked_target_bytes",
                "local_context_identity_hash",
                "kernel_profile_fingerprint",
                "verifier_policy_fingerprint",
                "checker_schema_version",
                "evidence_schema_version",
                "target_binding_hash",
                "formula_evidence_hash",
                "substitution_evidence_hash",
                "provenance_hash",
                "formula_context_hash",
                "accepted_result_hash",
                "accepted_goal_polarity",
                "witness_artifact_hash",
            ];
            let values = [
                original_hash,
                old.0
                    .target_vc_fingerprint()
                    .sort_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect(),
                local_context,
                accepted
                    .kernel_profile_fingerprint
                    .to_artifact_hash_string(),
                accepted
                    .verifier_policy_fingerprint
                    .to_artifact_hash_string(),
                accepted.checker_schema_version.to_string(),
                accepted.evidence_schema_version.to_string(),
                accepted.target_binding_hash.to_artifact_hash_string(),
                accepted.formula_evidence_hash.to_artifact_hash_string(),
                accepted
                    .substitution_evidence_hash
                    .to_artifact_hash_string(),
                accepted.provenance_hash.to_artifact_hash_string(),
                accepted.accepted_result_hash.to_artifact_hash_string(),
                input.accepted_goal_polarity().unwrap().as_str().into(),
                payload.to_artifact_hash_string(),
            ]
            .map(J::string);
            let mut values = values.into_iter().collect::<Vec<_>>();
            values.insert(11, J::null());
            let object = J::object(keys.into_iter().zip(values)).unwrap();
            let domain = Domain::new(
                HashClass::Interface,
                "mizar-proof/source-theorem-reflexivity-obligation",
                SchemaVersion::new(1, 0),
            );
            let m = domain.hash(&object, &[]);
            let actual = witness(&s, c, collected.env(), sn, vc, &p)?;
            assert_eq!(actual.1.witness_artifact_hash, payload);
            assert_eq!(actual.1.obligation_fingerprint.digest, m);
            assert_eq!(actual.2, old.3);
            let J::Object(fields) = &object else {
                panic!("object");
            };
            assert_eq!(fields.len(), 15);
            // Mutations exercise only the structural hash grammar, never a forged Kernel result.
            for (key, value) in fields {
                let mut changed = fields.clone();
                changed.insert(
                    key.clone(),
                    match value {
                        J::String(text) => J::string(format!("{text}0")),
                        J::Null => J::string("non-null"),
                        _ => panic!("unexpected participant type"),
                    },
                );
                assert_ne!(domain.hash(&J::Object(changed), &[]), m, "{key}");
            }
            assert_eq!(accepted.formula_context_hash, None);
            assert_ne!(
                handoff.context_identity_hash(),
                accepted.target_binding_hash.digest
            );
            assert_ne!(
                handoff.context_identity_hash(),
                id.0.local_context_fingerprint()
            );
            let v1 = handoff.encode_local_evidence(None)?;
            assert_ne!(old.3, v1);
            let v1_hex: String = v1.iter().map(|b| format!("{b:02x}")).collect();
            assert_ne!(payload_domain.hash(&J::string(v1_hex), &[]), payload.digest);
            let envelope = handoff.canonical_evidence();
            let target = envelope.target_vc();
            let profile = envelope.kernel_profile();
            assert_eq!(
                profile.clause_tautology_policy,
                mizar_vc::kernel_evidence_handoff::KernelClauseTautologyPolicy::Reject
            );
            let context = FormulaEvidenceParseContext::v1(
                Fingerprint::new(target.algorithm_id, target.digest.clone()),
                KernelProfileRecord::v1(profile.profile_id, ClauseTautologyPolicy::Reject),
            );
            let parsed = parse_formula_evidence(&v1, &context).unwrap();
            let mut names = mizar_atp::backend::encode_local_variable_reflexivity_names(&parsed)?;
            assert_eq!(handoff.encode_local_evidence(Some(&names))?, old.3);
            assert!(!names.is_empty());
            names[0][0] ^= 1;
            let changed = handoff.encode_local_evidence(Some(&names))?;
            assert_ne!(changed, old.3);
            let hex: String = changed.iter().map(|b| format!("{b:02x}")).collect();
            assert_ne!(payload_domain.hash(&J::string(hex), &[]), payload.digest);
            let mut changed = old.3.clone();
            changed.push(0);
            let hex: String = changed.iter().map(|b| format!("{b:02x}")).collect();
            assert_ne!(payload_domain.hash(&J::string(hex), &[]), payload.digest);
            Ok(())
        })
        .unwrap();
    }
    #[test]
    fn source_theorem_reflexivity_witness_preserves_identity_first_errors() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_proof::{
            policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
            status::{
                check_source_theorem_reflexivity as verify,
                check_source_theorem_reflexivity_metadata as metadata,
                project_source_theorem_reflexivity_witness as witness,
                source_theorem_obligation_identity as identity,
            },
        };
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let snap = snapshot_id(777);
        let absent = VcId::new(usize::MAX);
        let policies = [
            Policy::default(),
            Policy::default().with_kernel_evidence_formats([]),
            Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
        ];
        let policies_before = policies.clone();
        let cited = include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        );
        for text in [
            original.to_owned(),
            original.replacen("theorem", "lemma", 1),
            "open theorem O: for X being set holds X = X;".to_owned(),
            format!(
                "{original}\n{}",
                original.replace("theorem T:", "theorem U:")
            ),
            cited.to_owned(),
        ] {
            let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (source.clone(), env.clone());
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
                let check_before = format!("{check:?}");
                for (index, policy) in policies.iter().enumerate() {
                    for vc in [VcId::new(0), absent] {
                        if text == original && index == 0 && vc != absent {
                            continue;
                        }
                        let check_error =
                            metadata(&source, check, env, snap, vc, policy).unwrap_err();
                        let expected = match identity(&source, check, env, snap, vc) {
                            Err(_) => identity(&source, check, env, snap, vc).unwrap_err(),
                            Ok(_) => check_error,
                        };
                        assert_eq!(
                            witness(&source, check, env, snap, vc, policy).unwrap_err(),
                            expected
                        );
                    }
                }
                if text == original {
                    let vc = VcId::new(0);
                    let policy = &policies[0];
                    let prior = (
                        verify(&source, check, env, snap, vc, policy)?,
                        metadata(&source, check, env, snap, vc, policy)?,
                        witness(&source, check, env, snap, vc, policy)?,
                        identity(&source, check, env, snap, vc)?,
                    );
                    let prior_before = prior.clone();
                    for text in [
                        original.replace("theorem T:", "theorem Foreign:"),
                        format!(
                            "{original}\n{}",
                            original.replace("theorem T:", "theorem U:")
                        ),
                        "theorem T: for X being set holds X = ;".to_owned(),
                    ] {
                        let (other_frontend, other_source, other) =
                            inputs(&text, "owner-test", "owners", 0);
                        assert_eq!(
                            !other_frontend.diagnostics.is_empty(),
                            text.ends_with("= ;")
                        );
                        let other_before = (other_source.clone(), other.env().clone());
                        for (current, symbols) in [
                            (&other_source, other.env()),
                            (&source, other.env()),
                            (&other_source, env),
                        ] {
                            for policy in &policies {
                                for selected in [vc, absent] {
                                    let expected =
                                        identity(current, check, symbols, snap, selected)
                                            .unwrap_err();
                                    metadata(current, check, symbols, snap, selected, policy)
                                        .unwrap_err();
                                    assert_eq!(
                                        witness(current, check, symbols, snap, selected, policy)
                                            .unwrap_err(),
                                        expected
                                    );
                                }
                            }
                        }
                        assert_eq!((other_source, other.env().clone()), other_before);
                    }
                    assert_eq!(prior, prior_before);
                    assert_eq!(
                        prior,
                        (
                            verify(&source, check, env, snap, vc, policy)?,
                            metadata(&source, check, env, snap, vc, policy)?,
                            witness(&source, check, env, snap, vc, policy)?,
                            identity(&source, check, env, snap, vc)?,
                        )
                    );
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((source, env.clone()), before);
        }
        assert_eq!(policies, policies_before);
    }

    #[test]
    fn source_theorem_reflexivity_used_axioms_status_matches_real_kernel_and_original_slice() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_kernel::checker::{KernelCheckStatus, KernelEvidenceCheckKind};
        use mizar_proof::{
            policy::{
                AcceptedGoalPolarity, CandidatePolicyClass, KernelEvidenceFormat as Format,
                KernelEvidenceOrigin as Origin, KernelPolicyInput, VerifierPolicy as Policy,
            },
            selection::{
                ProofWinnerClass as Winner, VcProofSelection,
                merge_artifact_proof_selections as merge,
            },
            status::{
                ArtifactPublicationGap as Gap, ArtifactStatusPublication as Publication,
                ProjectedProofStatus as Status, ProofStatusProjectionInput as Input,
                TrustedUsedAxiomsRef, check_source_theorem_reflexivity as verify,
                project_source_theorem_reflexivity_status as legacy,
                project_source_theorem_reflexivity_used_axioms_status as project, project_status,
                select_source_theorem_reflexivity as select,
                source_theorem_obligation_identity as identity,
            },
        };
        use mizar_vc::{
            dependency_slice::DependencySliceCompleteness as Completeness,
            vc_ir::{VcId, VcStatus},
        };
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let bound = original.replace(
            "for X being set holds X = X",
            "for Bound being set holds Bound = Bound",
        );
        let local = original.replace(
            "let X be set; thus X = X",
            "let Local be set; thus Local = Local",
        );
        let trivia = format!(":: 日本語 λ\n{original}\n");
        let mut original_source = None;
        let mut cases = 0;
        for (index, (text, package, module, skip)) in [
            (original.to_owned(), "owner-test", "owners", 0),
            (bound, "owner-test", "owners", 0),
            (local, "owner-test", "owners", 0),
            (trivia, "owner-test", "owners", 0),
            (original.to_owned(), "owner-test", "owners", 7),
            (original.to_owned(), "other-package", "owners", 0),
            (original.to_owned(), "owner-test", "nested.owners", 0),
        ]
        .into_iter()
        .enumerate()
        {
            let (frontend, source, collected) = inputs(&text, package, module, skip);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (source.clone(), env.clone());
            if skip == 7 {
                assert_ne!(Some(source.source_id()), original_source);
            } else if original_source.is_none() {
                original_source = Some(source.source_id());
            }
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
                let check_before = format!("{check:?}");
                for policy in [
                    Policy::default(),
                    Policy::release().with_require_kernel_certificates(true),
                    Policy::development()
                        .with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                    Policy::default().with_profile_id("theorem-used-axioms"),
                    Policy::default().with_schema_version(2),
                    Policy::default().with_checker_schema_version(2),
                ]
                .into_iter()
                .take(if index == 0 { 6 } else { 1 })
                {
                    for ordinal in [777, 778] {
                        let snap = snapshot_id(ordinal);
                        let vc = VcId::new(0);
                        let policy_before = policy.clone();
                        let original = identity(&source, check, env, snap, vc)?;
                        let origin = Origin::BuiltinDischarge;
                        let checked = verify(&source, check, env, snap, vc, &policy)?;
                        let chosen = select(&source, check, env, snap, vc, &policy)?;
                        let old = legacy(&source, check, env, snap, vc, &policy)?;
                        let input = KernelPolicyInput::from_kernel_result(&checked.0, origin);
                        let reference =
                            TrustedUsedAxiomsRef::from_kernel_result(&checked.0, origin).unwrap();
                        assert_eq!(checked.0.status(), KernelCheckStatus::Accepted);
                        assert!(!checked.0.policy_taint());
                        assert_eq!(
                            checked.0.evidence_check_kind(),
                            Some(KernelEvidenceCheckKind::ProofObligation)
                        );
                        assert_eq!(checked.1.class, CandidatePolicyClass::DischargedBuiltin);
                        assert_eq!(reference.used_axiom_count(), checked.0.used_axioms().len());
                        assert!(checked.0.used_axioms().is_empty());
                        assert_eq!(
                            Some(reference.accepted_evidence_hash()),
                            input.accepted_evidence_hash()
                        );
                        assert_eq!(chosen.selected_class(), Winner::DischargedBuiltin);
                        assert!(chosen.trusted_used_axioms_available());
                        let [merged] = <[_; 1]>::try_from(
                            merge([], [VcProofSelection::new(vc, chosen.clone())]).unwrap(),
                        )
                        .unwrap();
                        let expected_legacy = project_status(Input::new(
                            merged.clone(),
                            policy.clone(),
                            original.0.clone(),
                        ))
                        .unwrap();
                        let expected = project_status(
                            Input::new(merged, policy.clone(), original.0.clone())
                                .with_trusted_used_axioms(reference.clone()),
                        )
                        .unwrap();
                        assert_eq!(old, (expected_legacy, original.1.clone()));
                        assert_eq!(old.0.trusted_used_axioms(), None);
                        assert_eq!(old.0.reuse_metadata().trusted_used_axioms_hash(), None);
                        let actual = project(&source, check, env, snap, vc, &policy)?;
                        assert_eq!(actual, (expected, original.1.clone()));
                        assert_eq!(actual.0.vc(), vc);
                        assert_eq!(actual.0.obligation_id(), original.0.obligation_id());
                        assert_eq!(actual.0.selected_class(), Winner::DischargedBuiltin);
                        assert_eq!(actual.0.projected_status(), Status::Accepted);
                        assert_eq!(
                            actual.0.artifact_publication(),
                            Publication::ExternalDependencyGap(
                                Gap::DischargedBuiltinWitnessUnsupported
                            )
                        );
                        assert_eq!(actual.0.trusted_used_axioms(), Some(&reference));
                        assert_eq!(actual.0.accepted_witness_obligation_id(), None);
                        assert_eq!(actual.0.explanation_ref(), None);
                        let meta = actual.0.reuse_metadata();
                        let id = &original.0;
                        assert_eq!(meta.obligation_anchor(), id.obligation_anchor());
                        assert_eq!(meta.obligation_fingerprint(), id.obligation_fingerprint());
                        assert_eq!(meta.vc_fingerprint(), id.vc_fingerprint());
                        assert_eq!(
                            meta.local_context_fingerprint(),
                            id.local_context_fingerprint()
                        );
                        assert_eq!(
                            meta.dependency_slice_fingerprint(),
                            id.dependency_slice_fingerprint()
                        );
                        assert_eq!(meta.policy_fingerprint(), policy.policy_fingerprint());
                        assert_eq!(meta.selected_candidate_id(), chosen.selected_candidate_id());
                        assert_eq!(
                            meta.selected_evidence_hash(),
                            Some(reference.accepted_evidence_hash())
                        );
                        assert_eq!(
                            meta.selected_evidence_hash(),
                            chosen.reuse_metadata().selected_evidence_hash()
                        );
                        assert_eq!(
                            meta.accepted_goal_polarity(),
                            Some(AcceptedGoalPolarity::AssertFalseForRefutation)
                        );
                        assert_eq!(
                            meta.accepted_goal_polarity(),
                            input.accepted_goal_polarity()
                        );
                        assert_eq!(
                            meta.trusted_used_axioms_hash(),
                            Some(reference.used_axioms_hash())
                        );
                        assert_eq!(meta.selected_proof_witness_hash(), None);
                        assert_eq!(meta.deterministic_discharge_hash(), None);
                        assert_eq!(meta.selected_candidate_provenance_hash(), None);
                        assert_eq!(meta.dependency_compatibility(), None);
                        assert_eq!(meta.explanation_ref(), None);
                        assert!(!meta.cache_reuse_predicate_complete());
                        assert_eq!(actual.1.status(), &VcStatus::Open);
                        assert!(!actual.1.entries().is_empty());
                        assert!(!actual.1.unknowns().is_empty());
                        assert_eq!(actual.1.completeness(), Completeness::IncompleteUncacheable);
                        assert_eq!(
                            actual.1.fingerprint().hash(),
                            meta.dependency_slice_fingerprint()
                        );
                        assert!(actual.1.requires_cache_miss());
                        assert_eq!(actual, project(&source, check, env, snap, vc, &policy)?);
                        assert_eq!(original, identity(&source, check, env, snap, vc)?);
                        assert_eq!(checked, verify(&source, check, env, snap, vc, &policy)?);
                        assert_eq!(chosen, select(&source, check, env, snap, vc, &policy)?);
                        assert_eq!(old, legacy(&source, check, env, snap, vc, &policy)?);
                        assert_eq!(policy, policy_before);
                        cases += 1;
                    }
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((&source, env), (&before.0, &before.1));
        }
        assert_eq!(cases, 24);
    }

    #[test]
    fn source_theorem_reflexivity_used_axioms_status_preserves_identity_first_errors() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_proof::{
            policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
            status::{
                check_source_theorem_reflexivity as verify,
                project_source_theorem_reflexivity_status as legacy,
                project_source_theorem_reflexivity_used_axioms_status as project,
                select_source_theorem_reflexivity as select,
                source_theorem_obligation_identity as identity,
            },
        };
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let snap = snapshot_id(777);
        let absent = VcId::new(usize::MAX);
        let policies = [
            Policy::default(),
            Policy::default().with_kernel_evidence_formats([]),
            Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
        ];
        let policies_before = policies.clone();
        let cited = include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        );
        for (case, text) in [
            original.to_owned(),
            original.replacen("theorem", "lemma", 1),
            "open theorem O: for X being set holds X = X;".to_owned(),
            format!(
                "{original}\n{}",
                original.replace("theorem T:", "theorem U:")
            ),
            cited.to_owned(),
        ]
        .into_iter()
        .enumerate()
        {
            let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (source.clone(), env.clone());
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
                let check_before = format!("{check:?}");
                let vc = VcId::new(0);
                let policy = &policies[0];
                let prior = if case == 0 {
                    Some((
                        identity(&source, check, env, snap, vc)?,
                        verify(&source, check, env, snap, vc, policy)?,
                        select(&source, check, env, snap, vc, policy)?,
                        legacy(&source, check, env, snap, vc, policy)?,
                        project(&source, check, env, snap, vc, policy)?,
                    ))
                } else {
                    None
                };
                for (index, policy) in policies.iter().enumerate() {
                    for selected in [vc, absent] {
                        if case == 0 && index == 0 && selected == vc {
                            continue;
                        }
                        let expected = match identity(&source, check, env, snap, selected) {
                            Err(_) => {
                                let error =
                                    identity(&source, check, env, snap, selected).unwrap_err();
                                let check_error =
                                    verify(&source, check, env, snap, selected, policy)
                                        .unwrap_err();
                                if case == 3 {
                                    assert_eq!(error, "unsupported source theorem formula payload");
                                    assert_eq!(check_error, "theorem.handoff.invalid_projection");
                                    assert_ne!(error, check_error);
                                }
                                error
                            }
                            Ok(original) => {
                                let check_error =
                                    verify(&source, check, env, snap, selected, policy)
                                        .unwrap_err();
                                assert_eq!((case, selected), (0, vc));
                                assert_eq!(check_error, "theorem.proof.kernel_policy_rejected");
                                assert_eq!(
                                    original,
                                    identity(&source, check, env, snap, selected)?
                                );
                                check_error
                            }
                        };
                        assert_eq!(
                            project(&source, check, env, snap, selected, policy).unwrap_err(),
                            expected
                        );
                    }
                }
                if let Some(prior) = prior {
                    for text in [
                        original.replace("theorem T:", "theorem Foreign:"),
                        format!(
                            "{original}\n{}",
                            original.replace("theorem T:", "theorem U:")
                        ),
                        "theorem T: for X being set holds X = ;".to_owned(),
                    ] {
                        let (other_frontend, other_source, other) =
                            inputs(&text, "owner-test", "owners", 0);
                        assert_eq!(
                            !other_frontend.diagnostics.is_empty(),
                            text.ends_with("= ;")
                        );
                        let other_before = (other_source.clone(), other.env().clone());
                        for (current, symbols) in [
                            (&other_source, other.env()),
                            (&source, other.env()),
                            (&other_source, env),
                        ] {
                            for policy in &policies {
                                for selected in [vc, absent] {
                                    let expected =
                                        identity(current, check, symbols, snap, selected)
                                            .unwrap_err();
                                    verify(current, check, symbols, snap, selected, policy)
                                        .unwrap_err();
                                    assert_eq!(
                                        project(current, check, symbols, snap, selected, policy)
                                            .unwrap_err(),
                                        expected
                                    );
                                }
                            }
                        }
                        assert_eq!((other_source, other.env().clone()), other_before);
                    }
                    assert_eq!(
                        prior,
                        (
                            identity(&source, check, env, snap, vc)?,
                            verify(&source, check, env, snap, vc, policy)?,
                            select(&source, check, env, snap, vc, policy)?,
                            legacy(&source, check, env, snap, vc, policy)?,
                            project(&source, check, env, snap, vc, policy)?,
                        )
                    );
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((source, env.clone()), before);
        }
        assert_eq!(policies, policies_before);
    }

    #[test]
    fn source_theorem_reflexivity_trusted_witness_metadata_matches_fresh_kernel() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_kernel::checker::{KernelCheckStatus as Status, KernelEvidenceCheckKind as Kind};
        use mizar_proof::{
            policy::{
                AcceptedGoalPolarity as Polarity, CandidatePolicyClass as Class,
                KernelEvidenceFormat as Format, KernelEvidenceOrigin as Origin, KernelPolicyInput,
                VerifierPolicy as Policy,
            },
            selection::{ProofWinnerClass as Winner, TrustedKernelEvidence},
            status::{
                check_source_theorem_reflexivity_metadata as metadata,
                source_theorem_reflexivity_trusted_witness_metadata as trusted,
            },
        };
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let mut original_source = None;
        for (index, (text, package, module, skip)) in [
            (original.to_owned(), "owner-test", "owners", 0),
            (
                original.replace(
                    "for X being set holds X = X",
                    "for Bound being set holds Bound = Bound",
                ),
                "owner-test",
                "owners",
                0,
            ),
            (
                original.replace(
                    "let X be set; thus X = X",
                    "let Local be set; thus Local = Local",
                ),
                "owner-test",
                "owners",
                0,
            ),
            (
                format!(":: 日本語 λ\n{original}\n"),
                "owner-test",
                "owners",
                0,
            ),
            (original.to_owned(), "owner-test", "owners", 7),
            (original.to_owned(), "other-package", "owners", 0),
            (original.to_owned(), "owner-test", "nested.owners", 0),
        ]
        .into_iter()
        .enumerate()
        {
            let (frontend, source, collected) = inputs(&text, package, module, skip);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (source.clone(), env.clone());
            if skip == 7 {
                assert_ne!(Some(source.source_id()), original_source);
            } else if original_source.is_none() {
                original_source = Some(source.source_id());
            }
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
                let check_before = format!("{check:?}");
                for policy in [
                    Policy::default(),
                    Policy::release().with_require_kernel_certificates(true),
                    Policy::development()
                        .with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
                    Policy::default().with_profile_id("theorem-trusted-metadata"),
                    Policy::default().with_schema_version(2),
                    Policy::default().with_checker_schema_version(2),
                ]
                .into_iter()
                .take(if index == 0 { 6 } else { 1 })
                {
                    let policy_before = policy.clone();
                    for ordinal in [777, 778] {
                        let snap = snapshot_id(ordinal);
                        let vc = VcId::new(0);
                        let origin = Origin::BuiltinDischarge;
                        let (result, decision, acceptance, _wire) =
                            metadata(&source, check, env, snap, vc, &policy)?;
                        assert_eq!(result.status(), Status::Accepted);
                        assert_eq!(result.evidence_check_kind(), Some(Kind::ProofObligation));
                        assert!(!result.policy_taint());
                        assert_eq!(decision.class, Class::DischargedBuiltin);
                        let acceptance = acceptance.unwrap();
                        let input = KernelPolicyInput::from_kernel_result(&result, origin);
                        let evidence = TrustedKernelEvidence::from_policy_input(&input).unwrap();
                        let actual = trusted(&source, check, env, snap, vc, &policy)?;
                        assert_eq!(actual.kernel_acceptance(), &acceptance);
                        assert_eq!(actual.trusted_kernel_evidence(), &evidence);
                        assert_eq!(actual.kernel_evidence_origin(), origin);
                        assert_eq!(actual.selected_class(), Winner::DischargedBuiltin);
                        assert_eq!(
                            Some(actual.accepted_evidence_hash()),
                            input.accepted_evidence_hash()
                        );
                        assert_eq!(
                            actual.accepted_evidence_hash(),
                            acceptance.accepted_result_hash.digest
                        );
                        assert_eq!(
                            actual.trusted_kernel_evidence().accepted_goal_polarity(),
                            Polarity::AssertFalseForRefutation
                        );
                        assert_eq!(
                            Some(actual.trusted_kernel_evidence().accepted_goal_polarity()),
                            input.accepted_goal_polarity()
                        );
                        assert_eq!(actual, trusted(&source, check, env, snap, vc, &policy)?);
                        assert_eq!(policy, policy_before);
                    }
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((&source, env), (&before.0, &before.1));
        }
    }

    #[test]
    fn source_theorem_reflexivity_trusted_witness_metadata_preserves_upstream_errors() {
        use super::super::tests::source_registration_owner_inputs as inputs;
        use mizar_proof::{
            policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
            status::{
                check_source_theorem_reflexivity_metadata as metadata,
                source_theorem_reflexivity_trusted_witness_metadata as trusted,
            },
        };
        use mizar_vc::vc_ir::VcId;
        let original =
            "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
        let snap = snapshot_id(777);
        let vc = VcId::new(0);
        let absent = VcId::new(usize::MAX);
        let policies = [
            Policy::default(),
            Policy::default().with_kernel_evidence_formats([]),
            Policy::release().with_kernel_evidence_formats([Format::FormulaSubstitution]),
        ];
        let policies_before = policies.clone();
        let cited = include_str!(
            "../../../../tests/miz/pass/theorems/pass_proof_verification_lemma_reference_001.miz"
        );
        for (case, text) in [
            original.to_owned(),
            original.replacen("theorem", "lemma", 1),
            "open theorem O: for X being set holds X = X;".to_owned(),
            format!(
                "{original}\n{}",
                original.replace("theorem T:", "theorem U:")
            ),
            cited.to_owned(),
        ]
        .into_iter()
        .enumerate()
        {
            let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
            assert!(frontend.diagnostics.is_empty());
            assert!(collected.diagnostics().is_empty());
            let env = collected.env();
            let before = (source.clone(), env.clone());
            with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
                let check_before = format!("{check:?}");
                let prior = if case == 0 {
                    Some(trusted(&source, check, env, snap, vc, &policies[0])?)
                } else {
                    None
                };
                for (index, policy) in policies
                    .iter()
                    .take(if case == 0 { 3 } else { 1 })
                    .enumerate()
                {
                    for selected in [vc, absent].into_iter().take(if case == 0 { 2 } else { 1 }) {
                        if case == 0 && index == 0 && selected == vc {
                            continue;
                        }
                        let expected =
                            metadata(&source, check, env, snap, selected, policy).unwrap_err();
                        assert_eq!(
                            trusted(&source, check, env, snap, selected, policy).unwrap_err(),
                            expected
                        );
                    }
                }
                if let Some(prior) = prior {
                    for text in [
                        original.replace("theorem T:", "theorem Foreign:"),
                        format!(
                            "{original}\n{}",
                            original.replace("theorem T:", "theorem U:")
                        ),
                        "theorem T: for X being set holds X = ;".to_owned(),
                    ] {
                        let (other_frontend, other_source, other) =
                            inputs(&text, "owner-test", "owners", 0);
                        assert_eq!(
                            !other_frontend.diagnostics.is_empty(),
                            text.ends_with("= ;")
                        );
                        let other_before = (other_source.clone(), other.env().clone());
                        for (current, symbols) in [
                            (&other_source, other.env()),
                            (&source, other.env()),
                            (&other_source, env),
                        ] {
                            let policy = &policies[0];
                            let expected =
                                metadata(current, check, symbols, snap, vc, policy).unwrap_err();
                            assert_eq!(
                                trusted(current, check, symbols, snap, vc, policy).unwrap_err(),
                                expected
                            );
                        }
                        assert_eq!((other_source, other.env().clone()), other_before);
                    }
                    assert_eq!(prior, trusted(&source, check, env, snap, vc, &policies[0])?);
                }
                assert_eq!(format!("{check:?}"), check_before);
                Ok(())
            })
            .unwrap();
            assert_eq!((source, env.clone()), before);
        }
        assert_eq!(policies, policies_before);
    }
}
