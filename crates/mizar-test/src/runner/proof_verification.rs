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

fn source_algorithm_vc_output(
    workspace_root: &Path,
    case: &TestCase,
) -> Result<(Option<mizar_core::core_ir::CoreIr>, VcSet), String> {
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
    let checked =
        mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols)?;
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
        use mizar_core::control_flow::{ControlFlowStatementPlacement, build_control_flow_ir};
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
                if let CoreAlgorithmStmtKind::Snapshot { name, captures } = &statement.kind {
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
        if sealed.len() != 2
            || captured.len() != 2
            || captured[0] == captured[1]
            || checked.snapshots().values().next().map(|(name, _)| name) != Some(*name)
            || !sealed.iter().zip(captured.iter()).all(|(binding, var)| {
                checked
                    .bindings()
                    .bindings()
                    .get(*binding)
                    .is_some_and(|binding| {
                        flow.locals.iter().any(|(_, local)| {
                            local.binder.var == *var
                                && local.binder.source.anchor
                                    == mizar_core::core_ir::CoreSourceAnchor::SourceRange(
                                        binding.declaration_range,
                                    )
                        })
                    })
            })
            || locals.len() != captured.len()
            || !locals.iter().zip(captured.iter()).all(|(local, var)| {
                flow.locals
                    .get(*local)
                    .is_some_and(|local| local.binder.var == *var)
            })
            || flow.contexts.get(*context).is_none_or(|context| {
                !locals
                    .iter()
                    .all(|local| context.definitely_initialized.contains(local))
            })
            || flow.ghost_effects.ghost_assignment_effects.len() != 1
        {
            return Err("snapshot capture correspondence differed".into());
        }
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
        &GenerationSchemaVersion::new(format!("mizar-vc-generation-step5c14-{profile}-v1")),
        &VcSchemaVersion::new(format!("mizar-vc-vcset-step5c14-{profile}-v1")),
    )?;
    Ok((Some(core), vcs))
}

fn source_registration_vc_output(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> Result<(mizar_core::core_ir::CoreIr, VcSet), String> {
    let (source, nodes, symbols) = super::source_registration_inputs(
        workspace_root,
        case,
        run_frontend(workspace_root, case, ordinal)?,
    )?;
    let checked = mizar_checker::registration_resolution::check_source_registration_intake(
        &source, &nodes, &symbols,
    )?;
    let core = mizar_core::elaborator::lower_source_functorial_registration(&checked)?;
    let vcs = generate_core_vcs(&core, snapshot_id(ordinal))?;
    Ok((core, vcs))
}

#[cfg(test)]
pub(in crate::runner) fn metamorphic_proof_observation(
    root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> Result<(Vec<String>, Vec<mizar_session::SourceRange>), String> {
    use mizar_core::core_ir::CoreSourceAnchor;
    if !is_active_proof_verification(case) {
        return Err("inactive proof case".into());
    }
    let unavailable = |value: String, route: &str| {
        (
            vec![value, format!("ranges=unavailable:{route}")],
            Vec::new(),
        )
    };
    let mut observation = Vec::new();
    let vcs = if is_step5c5_phrase_candidate(case) {
        if !step5c5_phrase_admitted(Some(root), case) {
            return Err("invalid phrase predicate admission".into());
        }
        phrase_theorem_output(root, case, run_frontend(root, case, 0)?)?.1
    } else if is_step5c14_return_candidate(case) {
        if !step5c14_return_admitted(Some(root), case) {
            return Err("invalid algorithm contract admission".into());
        }
        let (core, vcs) = source_algorithm_vc_output(root, case)?;
        if matches!(
            case.expectation.domain.as_str(),
            "algorithms.assertions" | "algorithms.contracts"
        ) {
            let core = core.as_ref().ok_or("algorithm Core missing")?;
            if let Some(failed) =
                mizar_vc::discharge::failed_source_algorithm_assertion(core, &vcs)?
            {
                let failed = vcs.vc(failed).ok_or("algorithm failure VC missing")?;
                let detail = match failed.kind {
                    mizar_vc::vc_ir::VcKind::AlgorithmAssertion => "algorithms.assert.unprovable",
                    mizar_vc::vc_ir::VcKind::AlgorithmPostcondition => {
                        "algorithms.ensures.unprovable"
                    }
                    _ => return Err("algorithm failure referred to a different obligation".into()),
                };
                observation.push(format!("fail/verification/proof_failure/{detail}"));
            }
        }
        vcs
    } else if is_step5c11_proof_candidate(case) {
        if !step5c11_proof_admitted(Some(root), case) {
            return Err("invalid Step 5C.11 proof admission".into());
        }
        let ordinal = if case.id.0 == STEP5C11_PROOF_CASES[1].0 {
            0
        } else {
            ordinal
        };
        let (core, vcs) = source_registration_vc_output(root, case, ordinal)?;
        if mizar_vc::discharge::failed_functorial_coherence(&core, &vcs)?.is_none() {
            return Err("registration did not produce the expected correctness failure".into());
        }
        let detail = if case.id.0 == STEP5C11_PROOF_CASES[1].0 {
            "clusters.reduce.false_reducibility"
        } else {
            "clusters.functorial.false_coherence"
        };
        observation.push(format!("fail/verification/proof_failure/{detail}"));
        vcs
    } else if is_step5c2_proof_workspace_member(root, case) {
        let receipt = normalize_structure_case(root, case, ordinal)?;
        if !receipt.has_zero_residual_vcs() || receipt.residual_vc_count() != 0 {
            return Err("structure normalization retained residual VCs".into());
        }
        return Ok(unavailable(
            format!("pass/vc_generation/normalized={}", receipt.equality_count()),
            "structure-receipt",
        ));
    } else if case.id.0 == EXACT_TASK180_CASE_ID {
        generate_case_vc(root, case, ordinal)?
    } else {
        let frontend = run_frontend(root, case, ordinal)?;
        if !frontend.diagnostics.is_empty() {
            return Err("proof frontend diagnostics".into());
        }
        let ast = frontend.ast.ok_or("proof source has no AST")?;
        let resolver = resolver_symbol_collection(root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            return Err("proof resolver diagnostics".into());
        }
        if super::formula_statement::is_step5c10_candidate(case) {
            if !super::formula_statement::step5_formula_admitted(Some(root), case) {
                return Err("invalid Step 5C.10 proof admission".into());
            }
            let (classes, vcs) = theorem_ast_output(
                &ast,
                &resolver.module,
                &resolver.env,
                case.expectation
                    .expected_phase
                    .ok_or("theorem phase missing")?,
                snapshot_id(ordinal),
                None,
            )?;
            match vcs {
                Some(vcs) if classes.is_empty() => vcs,
                None if !classes.is_empty() => {
                    return Ok(unavailable(
                        format!("pass/statement_check/policy={classes:?}"),
                        "theorem-policy",
                    ));
                }
                _ => return Err("theorem phase produced no matching output".into()),
            }
        } else if super::formula_statement::is_step5c9_candidate(case) {
            if !super::formula_statement::step5_formula_admitted(Some(root), case) {
                return Err("invalid Step 5C.9 proof admission".into());
            }
            if super::formula_statement::check_formula_ast_with_organization(
                &ast,
                &resolver.module,
                &resolver.env,
                true,
            )? {
                return Err("expected an incomplete source-derived case split".into());
            }
            return Ok(unavailable(
                "fail/verification/proof_failure/theorems.per_cases.incomplete_case_split".into(),
                "case-completeness",
            ));
        } else if step5c7_proof_workspace_member(root, case) {
            let count = normalize_term_ast(&ast, &resolver.module, &resolver.env)?;
            if count != 2 {
                return Err("membership obligations not discharged".into());
            }
            return Ok(unavailable(
                format!("pass/vc_generation/normalized={count}"),
                "membership-normalization",
            ));
        } else if is_step5c4_proof_workspace_member(root, case) {
            step5c4_mode_sethood_is_unprovable(
                &ast,
                &resolver.module,
                &resolver.shells,
                &resolver.env,
            )?;
            return Ok(unavailable(
                format!("fail/verification/proof_failure/{STEP5C4_PROOF_DETAIL_KEY}"),
                "sethood-observation",
            ));
        } else {
            return Err("unhandled active proof case".into());
        }
    };
    if vcs.vcs().is_empty() {
        return Err("proof route generated no VCs".into());
    }
    if observation.is_empty() {
        observation.push("pass/vc_generation".into());
    }
    // VC ordinals and identity hashes may change with source bytes. Pair retained
    // locations by the stable semantic owner/path/kind instead of those ordinals.
    let mut ordered = vcs
        .vcs()
        .iter()
        .map(|vc| {
            (
                format!(
                    "{:?}/{:?}/{:?}",
                    vc.anchor.owner, vc.anchor.local_path, vc.kind
                ),
                vc,
            )
        })
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.0.cmp(&right.0));
    if ordered.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err("ambiguous semantic VC range pairing".into());
    }
    let mut ranges = Vec::new();
    for (key, vc) in ordered {
        use mizar_vc::vc_ir::VcStatus;
        let status = match &vc.status {
            VcStatus::Open => "open".into(),
            VcStatus::NeedsAtp => "needs_atp".into(),
            VcStatus::Discharged { evidence } => format!("discharged/{:?}", evidence.rule),
            VcStatus::PolicyOpen { policy } => format!("policy_open/{policy:?}"),
            VcStatus::AssumedByPolicy { policy, .. } => format!("assumed/{policy:?}"),
            VcStatus::SkippedDueToInvalidInput { reason } => format!("invalid/{reason:?}"),
            VcStatus::DeferredExternal { reason } => format!("deferred/{reason:?}"),
            _ => return Err("VC status requires a semantic diagnostic observation".into()),
        };
        observation.push(format!("vc={key}/{status}"));
        for source in std::iter::once(&vc.source.primary).chain(&vc.source.related) {
            match &source.anchor {
                CoreSourceAnchor::SourceRange(range) => {
                    observation.push("source:range".into());
                    ranges.push(*range);
                }
                anchor => observation.push(format!("source:{anchor:?}")),
            }
        }
        observation.push(format!(
            "anchor_range_present={}",
            vc.anchor.source_range.is_some()
        ));
        if let Some(range) = vc.anchor.source_range {
            ranges.push(range);
        }
    }
    if ranges.is_empty() {
        observation.push("ranges=unavailable:vc-output".into());
    }
    Ok((observation, ranges))
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
            let (core, first) = source_algorithm_vc_output(workspace_root, case)?;
            let (replayed_core, second) = source_algorithm_vc_output(workspace_root, case)?;
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
            let (core, vcs) = source_registration_vc_output(workspace_root, case, ordinal)?;
            if reduction {
                let replay = source_registration_vc_output(workspace_root, case, ordinal)?;
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
            if case.id.0 == "pass_proof_verification_term_set_enumeration_membership_001" {
                super::native_conversion_handoff(&ast, &resolver.module, &resolver.env)?
                    .check_witness_goals()
                    .map_err(|_| "native enumeration goal does not match source".to_owned())?;
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
mod term_proof_tests;
