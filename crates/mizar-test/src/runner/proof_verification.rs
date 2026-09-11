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
    use mizar_vc::generator::{
        CoreGenerationCandidateSet, CoreGenerationInput, VcNormalizationInput,
    };
    use mizar_vc::vc_ir::{SeedIntakeTable, VcModuleRef};
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
        &typed, &scope, symbols, &labels, &resolved,
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
    let core = mizar_core::elaborator::lower_source_theorem_skeletons(&checked)?;
    let flow = mizar_core::control_flow::build_control_flow_ir(&core);
    let handoff = mizar_core::control_flow::build_obligation_seed_handoff(&core, &flow);
    let intake = SeedIntakeTable::try_from_handoff(&handoff).map_err(|error| error.to_string())?;
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
        source: ast.source_id,
        candidates: &candidates,
    })
    .map_err(|error| error.to_string())?;
    Ok((classes, Some(vcs)))
}

pub(super) fn is_active_proof_verification(case: &TestCase) -> bool {
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
        if !is_active_proof_verification(case)
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
    // Task-180 is the single snapshot-backed proof case.  Its source and
    // snapshot identities stay fixed when later snapshot-free proof routes
    // are admitted before it in corpus order.
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
                            snapshot_id(ordinal)
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
        )
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
                &typed,
                &scope,
                &resolver.env,
                &labels,
                &genuine,
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
                &typed,
                &scope,
                &resolver.env,
                &labels,
                &substituted,
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
                snapshot_id(777)
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
