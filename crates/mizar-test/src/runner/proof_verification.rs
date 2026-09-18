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

const STEP5C14_VC_CASES: [(&str, &str, &str, &str, &str, &str); 6] = [
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
    let failure = domain == "algorithms.assertions";
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
        && (!matches!(
            domain,
            "algorithms.claim"
                | "algorithms.assertions"
                | "algorithms.computation"
                | "algorithms.ghost"
        ) || case.expectation.schema_version == 1
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
            == failure.then_some("algorithms.assert.unprovable")
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
        if is_step5c14_return_candidate(case)
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
            if case.expectation.domain == "algorithms.assertions"
                && mizar_vc::discharge::failed_source_algorithm_assertion(
                    core.as_ref().ok_or("assertion Core missing")?,
                    &first,
                )?
                .is_none()
            {
                return Err("algorithms.assert.unprovable was not observed".into());
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

    fn step5c14_computation_core(text: &str) -> Result<mizar_core::core_ir::CoreIr, String> {
        use mizar_checker::type_checker::SourceVariableSemanticsChecker;
        use mizar_resolve::{
            labels::{LabelResolver, ProofLabelSourceCollector},
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
        let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
            &source,
            &typed,
            &scope,
            &resolver.env,
            &labels,
            &resolved,
            None,
        )?;
        mizar_core::elaborator::lower_source_theorem_skeletons(&check, None)
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
}
