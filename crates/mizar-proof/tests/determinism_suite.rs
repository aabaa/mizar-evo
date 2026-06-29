use std::collections::BTreeMap;

use mizar_proof::{
    policy::{
        CandidatePolicyClass, ExternalEvidenceMode, ExternalEvidencePublicationStatus,
        OpenObligationMode, PolicyAssumptionMode, PolicyCandidate, PolicyDecision,
        PolicyDiagnostic, PolicyDiagnosticCategory, PolicyReasonCode, PortfolioEarlyStopClass,
        ProofPolicyEvaluator, VerifierPolicy,
    },
    selection::{
        CandidateSourceId, DiagnosticRef, ProofEvidenceCandidate, ProofEvidenceSet, ProofSelection,
        VcProofSelection, merge_artifact_proof_selections, select_winner,
    },
    status::{
        ExplanationRef, ObligationAnchor, ProofObligationIdentity, ProofStatusProjection,
        ProofStatusProjectionInput, project_status,
    },
};
use mizar_session::Hash;
use mizar_vc::vc_ir::VcId;

#[test]
fn policy_classification_and_normalization_ignore_candidate_order() {
    let policy = verifier_policy();
    let candidates = policy_candidates();
    let expected = policy_observations(&policy, candidates.iter().cloned());

    for permutation in permutations(candidates.len()) {
        let shuffled = permutation
            .into_iter()
            .map(|index| candidates[index].clone());
        assert_eq!(policy_observations(&policy, shuffled), expected);
    }
}

#[test]
fn selection_and_status_outputs_ignore_candidate_order() {
    let policy = verifier_policy();
    let candidates = selection_candidates(&policy);
    let expected_selection = select_ordered(policy.clone(), candidates.iter().cloned());
    let expected_projection = project_ordered(policy.clone(), expected_selection.clone());

    for permutation in permutations(candidates.len()) {
        let shuffled = permutation
            .into_iter()
            .map(|index| candidates[index].clone());
        let selection = select_ordered(policy.clone(), shuffled);
        assert_eq!(selection, expected_selection);

        let projection = project_ordered(policy.clone(), selection);
        assert_eq!(projection, expected_projection);
        assert_eq!(
            projection.reuse_metadata(),
            expected_projection.reuse_metadata()
        );
    }
}

fn policy_observations(
    policy: &VerifierPolicy,
    candidates: impl IntoIterator<Item = (&'static str, PolicyCandidate)>,
) -> BTreeMap<&'static str, PolicyObservation> {
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    candidates
        .into_iter()
        .map(|(id, candidate)| {
            let decision = evaluator.evaluate_candidate(&candidate);
            (
                id,
                PolicyObservation {
                    class: decision.class,
                    can_schedule_kernel_check: decision.can_schedule_kernel_check,
                    diagnostic: decision
                        .diagnostic
                        .as_ref()
                        .map(|diagnostic| (diagnostic.category, diagnostic.reason)),
                    external_status: decision
                        .external_admission
                        .as_ref()
                        .map(|admission| admission.publication_status()),
                    early_stop_class: evaluator.best_possible_early_stop_class(&candidate),
                },
            )
        })
        .collect()
}

fn select_ordered(
    policy: VerifierPolicy,
    candidates: impl IntoIterator<Item = ProofEvidenceCandidate>,
) -> ProofSelection {
    select_winner(
        &ProofEvidenceSet::new(b"obligation".to_vec(), hash(100), policy)
            .with_candidates(candidates),
    )
}

fn project_ordered(policy: VerifierPolicy, selection: ProofSelection) -> ProofStatusProjection {
    let mut merged =
        merge_artifact_proof_selections([VcProofSelection::new(VcId::new(7), selection)], [])
            .expect("portfolio selection is valid");
    assert_eq!(merged.len(), 1);
    project_status(
        ProofStatusProjectionInput::new(merged.remove(0), policy, obligation_identity())
            .with_explanation_ref(ExplanationRef::new(hash(90))),
    )
    .expect("status projection succeeds")
}

fn policy_candidates() -> [(&'static str, PolicyCandidate); 7] {
    [
        (
            "formula",
            PolicyCandidate::UncheckedFormulaSubstitution {
                encoded_problem_matches: true,
            },
        ),
        ("external", PolicyCandidate::ExternallyAttested),
        ("assumption", PolicyCandidate::PolicyAssumption),
        ("open", PolicyCandidate::OpenObligation),
        ("legacy", PolicyCandidate::LegacyReplay),
        ("diagnostic", PolicyCandidate::BackendDiagnostic),
        ("cache", PolicyCandidate::CacheRecord),
    ]
}

fn selection_candidates(policy: &VerifierPolicy) -> Vec<ProofEvidenceCandidate> {
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    vec![
        candidate(
            "external",
            evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
        )
        .with_backend_profile_priority(1)
        .with_evidence_hash(hash(10)),
        candidate(
            "assumed",
            evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
        )
        .with_backend_profile_priority(0)
        .with_evidence_hash(hash(11)),
        candidate(
            "open",
            evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
        )
        .with_backend_profile_priority(0)
        .with_evidence_hash(hash(12))
        .with_diagnostic_ref(DiagnosticRef::new(hash(70))),
        candidate("rejected", rejected_policy_decision())
            .with_backend_profile_priority(0)
            .with_evidence_hash(hash(13))
            .with_diagnostic_ref(DiagnosticRef::new(hash(71))),
    ]
}

fn verifier_policy() -> VerifierPolicy {
    VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
        .with_policy_assumption(PolicyAssumptionMode::Record)
        .with_open_obligation(OpenObligationMode::AllowPolicyOpen)
}

fn obligation_identity() -> ProofObligationIdentity {
    ProofObligationIdentity::new(
        "obligation-7",
        ObligationAnchor::new("anchor-7").expect("anchor"),
        hash(20),
        hash(21),
        hash(22),
        hash(23),
    )
    .expect("proof obligation identity")
}

fn candidate(id: &str, decision: PolicyDecision) -> ProofEvidenceCandidate {
    ProofEvidenceCandidate::new(CandidateSourceId::new(id).expect("source id"), decision)
}

fn rejected_policy_decision() -> PolicyDecision {
    PolicyDecision {
        class: CandidatePolicyClass::RejectedByPolicy,
        can_schedule_kernel_check: false,
        diagnostic: Some(PolicyDiagnostic::new(
            PolicyDiagnosticCategory::PolicyRejection,
            PolicyReasonCode::LegacyReplayRejected,
        )),
        kernel_rejections: Vec::new(),
        external_admission: None,
    }
}

fn permutations(len: usize) -> Vec<Vec<usize>> {
    let forward = (0..len).collect::<Vec<_>>();
    let reverse = (0..len).rev().collect::<Vec<_>>();
    let rotated = (0..len).map(|index| (index + 2) % len).collect::<Vec<_>>();
    vec![forward, reverse, rotated]
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PolicyObservation {
    class: CandidatePolicyClass,
    can_schedule_kernel_check: bool,
    diagnostic: Option<(PolicyDiagnosticCategory, PolicyReasonCode)>,
    external_status: Option<ExternalEvidencePublicationStatus>,
    early_stop_class: Option<PortfolioEarlyStopClass>,
}
