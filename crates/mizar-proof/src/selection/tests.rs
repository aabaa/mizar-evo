use crate::policy::{
    ExternalEvidenceMode, KernelEvidenceOrigin, PolicyAssumptionMode, PolicyCandidate,
    PolicyDiagnosticCategory, ProofPolicyEvaluator,
};
use mizar_kernel::rejection::{
    RejectionCategory, RejectionDetail, RejectionLocation, RejectionRecord, TargetVcFingerprint,
};

use super::*;

#[test]
fn candidate_source_id_rejects_empty_id() {
    assert_eq!(
        CandidateSourceId::new(""),
        Err(SelectionInputError::EmptyCandidateSourceId)
    );
    assert_eq!(
        CandidateSourceId::new("stable-candidate")
            .expect("stable id")
            .as_str(),
        "stable-candidate"
    );
}

#[test]
fn winner_order_prefers_trusted_then_policy_classes() {
    let policy = VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
        .with_policy_assumption(PolicyAssumptionMode::Record);
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let set = base_set(policy).with_candidates([
        candidate("rejected", rejected_policy_decision()).with_evidence_hash(hash(6)),
        candidate(
            "open",
            evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
        )
        .with_evidence_hash(hash(5)),
        candidate(
            "assumed",
            evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
        )
        .with_evidence_hash(hash(4)),
        candidate(
            "external",
            evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
        )
        .with_evidence_hash(hash(3)),
        trusted_candidate(
            "builtin",
            CandidatePolicyClass::DischargedBuiltin,
            KernelEvidenceOrigin::BuiltinDischarge,
            hash(22),
        )
        .with_deterministic_discharge_hash(hash(2)),
        trusted_candidate(
            "kernel",
            CandidatePolicyClass::KernelVerified,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            hash(1),
        ),
    ]);

    let selection = select_winner(&set);

    assert_eq!(selection.selected_class(), ProofWinnerClass::KernelVerified);
    assert_eq!(
        selection
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("kernel")
    );
    assert!(selection.trusted_used_axioms_available());
}

#[test]
fn winner_order_exercises_adjacent_boundaries() {
    let policy = VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
        .with_policy_assumption(PolicyAssumptionMode::Record);
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let builtin = trusted_candidate(
        "builtin",
        CandidatePolicyClass::DischargedBuiltin,
        KernelEvidenceOrigin::BuiltinDischarge,
        hash(21),
    )
    .with_deterministic_discharge_hash(hash(22));
    let external = candidate(
        "external",
        evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
    )
    .with_evidence_hash(hash(23));
    let assumed = candidate(
        "assumed",
        evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
    )
    .with_evidence_hash(hash(24));
    let open = candidate(
        "open",
        evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
    )
    .with_evidence_hash(hash(25));
    let rejected = candidate("rejected", rejected_policy_decision()).with_evidence_hash(hash(26));

    assert_selected(
        &base_set(policy.clone()).with_candidates([
            external.clone(),
            assumed.clone(),
            open.clone(),
            rejected.clone(),
            builtin,
        ]),
        ProofWinnerClass::DischargedBuiltin,
        "builtin",
    );
    assert_selected(
        &base_set(policy.clone()).with_candidates([
            assumed.clone(),
            open.clone(),
            rejected.clone(),
            external,
        ]),
        ProofWinnerClass::PolicyPermittedExternal,
        "external",
    );
    assert_selected(
        &base_set(policy.clone()).with_candidates([open.clone(), rejected.clone(), assumed]),
        ProofWinnerClass::PolicyAssumed,
        "assumed",
    );
    assert_selected(
        &base_set(policy.clone()).with_candidates([rejected.clone(), open]),
        ProofWinnerClass::PolicyOpen,
        "open",
    );
    assert_selected(
        &base_set(policy).with_candidates([rejected]),
        ProofWinnerClass::Rejected,
        "rejected",
    );
}

#[test]
fn shuffled_candidate_arrival_never_changes_the_winner() {
    let policy = VerifierPolicy::release();
    let left = trusted_candidate(
        "preferred",
        CandidatePolicyClass::KernelVerified,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(2),
    )
    .with_backend_profile_priority(0)
    .with_evidence_format_priority(0)
    .with_provenance_hash(hash(9));
    let right = trusted_candidate(
        "later",
        CandidatePolicyClass::KernelVerified,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(1),
    )
    .with_backend_profile_priority(1)
    .with_evidence_format_priority(0)
    .with_provenance_hash(hash(8));

    let first =
        select_winner(&base_set(policy.clone()).with_candidates([left.clone(), right.clone()]));
    let second = select_winner(&base_set(policy).with_candidates([right, left]));

    assert_eq!(
        first.selected_candidate_id(),
        second.selected_candidate_id()
    );
    assert_eq!(
        first.selected_candidate_id().map(CandidateSourceId::as_str),
        Some("preferred")
    );
    assert_eq!(
        first.reuse_metadata().tie_break_key_hash(),
        second.reuse_metadata().tie_break_key_hash()
    );
    assert_eq!(first.reuse_metadata(), second.reuse_metadata());
    assert_eq!(
        first.reuse_metadata().selected_candidate_provenance_hash(),
        Some(hash(9))
    );
    assert_eq!(
        first.reuse_metadata().selection_reason(),
        "deterministic_winner_class"
    );
}

#[test]
fn tie_break_uses_present_hashes_and_priority_sentinels() {
    let policy = VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner);
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let external = evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested);
    let with_present_hash = candidate("with-hash", external.clone()).with_evidence_hash(hash(9));
    let missing_hash = candidate("missing-hash", external.clone());
    let explicit_priority = candidate("explicit-priority", external)
        .with_backend_profile_priority(1)
        .with_evidence_hash(hash(10));

    let selection = select_winner(&base_set(policy).with_candidates([
        missing_hash,
        with_present_hash,
        explicit_priority,
    ]));

    assert_eq!(
        selection
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("explicit-priority")
    );
    assert_eq!(
        selection.reuse_metadata().selected_evidence_hash(),
        Some(hash(10))
    );
}

#[test]
fn tie_break_covers_format_hash_provenance_and_source_id() {
    let explicit_format = trusted_candidate(
        "explicit-format",
        CandidatePolicyClass::KernelVerified,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(31),
    )
    .with_backend_profile_priority(0)
    .with_evidence_format_priority(0);
    let missing_format = trusted_candidate(
        "missing-format",
        CandidatePolicyClass::KernelVerified,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(30),
    )
    .with_backend_profile_priority(0);
    assert_selected(
        &base_set(VerifierPolicy::release()).with_candidates([missing_format, explicit_format]),
        ProofWinnerClass::KernelVerified,
        "explicit-format",
    );

    let policy = VerifierPolicy::interactive();
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let open_decision = evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation);
    let present_hash =
        candidate("present-hash", open_decision.clone()).with_evidence_hash(hash(40));
    let missing_hash = candidate("missing-hash", open_decision.clone());
    assert_selected(
        &base_set(policy.clone()).with_candidates([missing_hash, present_hash]),
        ProofWinnerClass::PolicyOpen,
        "present-hash",
    );

    let provenance_low = candidate("prov-low", open_decision.clone())
        .with_evidence_hash(hash(41))
        .with_provenance_hash(hash(1));
    let provenance_high = candidate("prov-high", open_decision.clone())
        .with_evidence_hash(hash(41))
        .with_provenance_hash(hash(2));
    assert_selected(
        &base_set(policy.clone()).with_candidates([provenance_high, provenance_low]),
        ProofWinnerClass::PolicyOpen,
        "prov-low",
    );

    let source_a = candidate("source-a", open_decision.clone()).with_evidence_hash(hash(42));
    let source_b = candidate("source-b", open_decision).with_evidence_hash(hash(42));
    assert_selected(
        &base_set(policy).with_candidates([source_b, source_a]),
        ProofWinnerClass::PolicyOpen,
        "source-a",
    );
}

#[test]
fn conflicting_duplicate_candidate_ids_are_reported_deterministically() {
    let policy = VerifierPolicy::interactive();
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let first = candidate(
        "duplicate",
        evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
    )
    .with_evidence_hash(hash(50));
    let second = candidate(
        "duplicate",
        evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
    )
    .with_evidence_hash(hash(51));

    let conflict_only =
        select_winner(&base_set(policy.clone()).with_candidates([first.clone(), second.clone()]));
    assert_eq!(conflict_only.selected_class(), ProofWinnerClass::Rejected);
    assert_eq!(
        conflict_only
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("duplicate")
    );
    assert!(!conflict_only.ordered_diagnostic_refs().is_empty());

    let trusted = trusted_candidate(
        "trusted",
        CandidatePolicyClass::KernelVerified,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(52),
    );
    let with_trusted = select_winner(&base_set(policy).with_candidates([first, trusted, second]));
    assert_eq!(
        with_trusted.selected_class(),
        ProofWinnerClass::KernelVerified
    );
    assert_eq!(
        with_trusted
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("trusted")
    );
    assert!(!with_trusted.ordered_diagnostic_refs().is_empty());
}

#[test]
fn external_and_policy_assumption_do_not_win_when_kernel_certificates_are_required() {
    let policy = VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
        .with_require_kernel_certificates(true)
        .with_policy_assumption(PolicyAssumptionMode::Record);
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let external = candidate(
        "external",
        evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
    )
    .with_evidence_hash(hash(1));
    let assumption = candidate(
        "assumption",
        evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
    )
    .with_evidence_hash(hash(2));

    let selection = select_winner(&base_set(policy).with_candidates([external, assumption]));

    assert_eq!(selection.selected_class(), ProofWinnerClass::Rejected);
    assert_eq!(
        selection
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("assumption")
    );
    assert!(!selection.trusted_used_axioms_available());
}

#[test]
fn pending_kernel_checkable_blocks_non_trusted_winners() {
    let policy = VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
        .with_policy_assumption(PolicyAssumptionMode::Record);
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let external = candidate(
        "external",
        evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
    )
    .with_evidence_hash(hash(61));
    let open = candidate(
        "open",
        evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
    )
    .with_evidence_hash(hash(62));
    let kernel_checkable = candidate(
        "kernel-checkable",
        evaluator.evaluate_candidate(&PolicyCandidate::UncheckedFormulaSubstitution {
            encoded_problem_matches: true,
        }),
    )
    .with_evidence_hash(hash(63));

    let selection =
        select_winner(&base_set(policy).with_candidates([external, open, kernel_checkable]));

    assert_eq!(selection.selected_class(), ProofWinnerClass::Rejected);
    assert_eq!(
        selection
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("kernel-checkable")
    );
}

#[test]
fn diagnostic_only_and_empty_sets_have_no_selected_winner() {
    let empty = select_winner(&base_set(VerifierPolicy::development()));
    assert_eq!(
        empty.selected_class(),
        ProofWinnerClass::NoSelectableEvidence
    );
    assert!(empty.selected_candidate_id().is_none());
    assert!(empty.diagnostic_result_id().is_some());
    assert!(!empty.trusted_used_axioms_available());

    let evaluator = ProofPolicyEvaluator::new(VerifierPolicy::development());
    let diagnostic = candidate(
        "diagnostic",
        evaluator.evaluate_candidate(&PolicyCandidate::BackendDiagnostic),
    )
    .with_diagnostic_ref(DiagnosticRef::new(hash(44)));
    let selection =
        select_winner(&base_set(evaluator.policy().clone()).with_candidates([diagnostic]));

    assert_eq!(
        selection.selected_class(),
        ProofWinnerClass::NoSelectableEvidence
    );
    assert!(selection.selected_candidate_id().is_none());
    assert_eq!(
        selection.ordered_diagnostic_refs(),
        &[DiagnosticRef::new(hash(44))]
    );
}

#[test]
fn rejected_candidates_use_diagnostic_order_not_arrival() {
    let kernel_rejected = candidate(
        "kernel-rejected",
        PolicyDecision {
            class: CandidatePolicyClass::KernelRejected,
            can_schedule_kernel_check: false,
            diagnostic: None,
            kernel_rejections: Vec::new(),
            external_admission: None,
        },
    )
    .with_evidence_hash(hash(9));
    let policy_rejected =
        candidate("policy-rejected", rejected_policy_decision()).with_evidence_hash(hash(1));

    let left = select_winner(
        &base_set(VerifierPolicy::release())
            .with_candidates([policy_rejected.clone(), kernel_rejected.clone()]),
    );
    let right = select_winner(
        &base_set(VerifierPolicy::release()).with_candidates([kernel_rejected, policy_rejected]),
    );

    assert_eq!(left.selected_class(), ProofWinnerClass::Rejected);
    assert_eq!(left.selected_candidate_id(), right.selected_candidate_id());
    assert_eq!(
        left.selected_candidate_id().map(CandidateSourceId::as_str),
        Some("kernel-rejected")
    );
}

#[test]
fn rejected_order_covers_source_category_hash_and_id_ties() {
    let evidence_rejected = candidate(
        "evidence-rejected",
        rejected_policy_decision_with_reason(PolicyReasonCode::KernelEvidenceFormatDisabled),
    )
    .with_evidence_hash(hash(70));
    let policy_rejected =
        candidate("policy-rejected", rejected_policy_decision()).with_evidence_hash(hash(69));
    let invalid_rejected = candidate(
        "invalid-rejected",
        PolicyDecision {
            class: CandidatePolicyClass::KernelCheckable,
            can_schedule_kernel_check: true,
            diagnostic: None,
            kernel_rejections: Vec::new(),
            external_admission: None,
        },
    )
    .with_evidence_hash(hash(68));

    assert_selected(
        &base_set(VerifierPolicy::release())
            .with_candidates([policy_rejected.clone(), evidence_rejected.clone()]),
        ProofWinnerClass::Rejected,
        "evidence-rejected",
    );
    assert_selected(
        &base_set(VerifierPolicy::release())
            .with_candidates([invalid_rejected.clone(), policy_rejected]),
        ProofWinnerClass::Rejected,
        "policy-rejected",
    );

    let certificate_rejected = candidate(
        "certificate-rejected",
        kernel_rejected_decision_with_record(rejection_record(
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedCertificate,
        )),
    );
    let kernel_rejected = candidate(
        "kernel-rejected",
        kernel_rejected_decision_with_record(rejection_record(
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSubstitution,
        )),
    );
    assert_selected(
        &base_set(VerifierPolicy::release())
            .with_candidates([kernel_rejected, certificate_rejected]),
        ProofWinnerClass::Rejected,
        "certificate-rejected",
    );

    let missing_hash = candidate("missing-hash", rejected_policy_decision());
    let present_hash =
        candidate("present-hash", rejected_policy_decision()).with_evidence_hash(hash(1));
    assert_selected(
        &base_set(VerifierPolicy::release()).with_candidates([missing_hash, present_hash]),
        ProofWinnerClass::Rejected,
        "present-hash",
    );

    let source_a = candidate("source-a", rejected_policy_decision()).with_evidence_hash(hash(2));
    let source_b = candidate("source-b", rejected_policy_decision()).with_evidence_hash(hash(2));
    assert_selected(
        &base_set(VerifierPolicy::release()).with_candidates([source_b, source_a]),
        ProofWinnerClass::Rejected,
        "source-a",
    );
}

#[test]
fn trusted_classes_require_kernel_derived_marker() {
    let spoofed = candidate(
        "spoofed",
        PolicyDecision {
            class: CandidatePolicyClass::KernelVerified,
            can_schedule_kernel_check: false,
            diagnostic: None,
            kernel_rejections: Vec::new(),
            external_admission: None,
        },
    );

    let selection = select_winner(&base_set(VerifierPolicy::release()).with_candidates([spoofed]));

    assert_eq!(selection.selected_class(), ProofWinnerClass::Rejected);
    assert_eq!(
        selection
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("spoofed")
    );
    assert!(!selection.trusted_used_axioms_available());

    let mismatched_kernel_input = KernelPolicyInput::for_test(
        KernelCheckStatus::Accepted,
        KernelEvidenceOrigin::BuiltinDischarge,
        false,
        Some(hash(80)),
    );
    let mismatched = candidate(
        "mismatched",
        PolicyDecision {
            class: CandidatePolicyClass::DischargedBuiltin,
            can_schedule_kernel_check: false,
            diagnostic: None,
            kernel_rejections: Vec::new(),
            external_admission: None,
        },
    )
    .with_evidence_hash(hash(81))
    .with_trusted_kernel_evidence(
        TrustedKernelEvidence::from_policy_input(&mismatched_kernel_input)
            .expect("accepted input with evidence hash"),
    );

    let mismatched_selection =
        select_winner(&base_set(VerifierPolicy::release()).with_candidates([mismatched]));
    assert_eq!(
        mismatched_selection.selected_class(),
        ProofWinnerClass::Rejected
    );
    assert!(!mismatched_selection.trusted_used_axioms_available());
}

#[test]
fn discharged_builtin_trust_and_witness_gap_are_explicit() {
    let builtin = trusted_candidate(
        "builtin",
        CandidatePolicyClass::DischargedBuiltin,
        KernelEvidenceOrigin::BuiltinDischarge,
        hash(90),
    )
    .with_deterministic_discharge_hash(hash(91))
    .with_selected_proof_witness_hash(hash(92));

    let selection = select_winner(&base_set(VerifierPolicy::release()).with_candidates([builtin]));

    assert_eq!(
        selection.selected_class(),
        ProofWinnerClass::DischargedBuiltin
    );
    assert!(selection.trusted_used_axioms_available());
    assert_eq!(
        selection.reuse_metadata().selected_evidence_hash(),
        Some(hash(90))
    );
    assert_eq!(
        selection.reuse_metadata().deterministic_discharge_hash(),
        Some(hash(91))
    );
    assert_eq!(
        selection.reuse_metadata().selected_proof_witness_hash(),
        None
    );
    assert_eq!(
        selection.reuse_metadata().proof_witness_publication(),
        ProofWitnessPublication::ExternalDependencyGap
    );
}

#[test]
fn non_trusted_winners_never_report_trusted_used_axioms() {
    let external_policy = VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner);
    let external_evaluator = ProofPolicyEvaluator::new(external_policy.clone());
    let external = candidate(
        "external",
        external_evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
    )
    .with_evidence_hash(hash(101));
    let external_selection = select_winner(&base_set(external_policy).with_candidates([external]));
    assert_eq!(
        external_selection.selected_class(),
        ProofWinnerClass::PolicyPermittedExternal
    );
    assert!(!external_selection.trusted_used_axioms_available());

    let assumed_policy = VerifierPolicy::development()
        .with_external_evidence(ExternalEvidenceMode::Reject)
        .with_policy_assumption(PolicyAssumptionMode::Record);
    let assumed_evaluator = ProofPolicyEvaluator::new(assumed_policy.clone());
    let assumed = candidate(
        "assumed",
        assumed_evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
    )
    .with_evidence_hash(hash(102));
    let assumed_selection = select_winner(&base_set(assumed_policy).with_candidates([assumed]));
    assert_eq!(
        assumed_selection.selected_class(),
        ProofWinnerClass::PolicyAssumed
    );
    assert!(!assumed_selection.trusted_used_axioms_available());

    let open_policy = VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::Reject)
        .with_policy_assumption(PolicyAssumptionMode::Reject);
    let open_evaluator = ProofPolicyEvaluator::new(open_policy.clone());
    let open = candidate(
        "open",
        open_evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
    )
    .with_evidence_hash(hash(103));
    let open_selection = select_winner(&base_set(open_policy).with_candidates([open]));
    assert_eq!(
        open_selection.selected_class(),
        ProofWinnerClass::PolicyOpen
    );
    assert!(!open_selection.trusted_used_axioms_available());
}

#[test]
fn forged_external_admission_cannot_override_active_policy_mode() {
    let forged_external = candidate(
        "forged-external",
        PolicyDecision {
            class: CandidatePolicyClass::ExternallyAttested,
            can_schedule_kernel_check: false,
            diagnostic: None,
            kernel_rejections: Vec::new(),
            external_admission: Some(crate::policy::ExternalEvidenceAdmission::new(
                true,
                true,
                ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted,
                None,
            )),
        },
    )
    .with_evidence_hash(hash(110));

    let selection =
        select_winner(&base_set(VerifierPolicy::development()).with_candidates([forged_external]));

    assert_eq!(
        selection.selected_class(),
        ProofWinnerClass::NoSelectableEvidence
    );
    assert!(selection.selected_candidate_id().is_none());
}

#[test]
fn artifact_merge_emits_canonical_vc_order() {
    let merged = merge_artifact_proof_selections(
        [
            VcProofSelection::new(VcId::new(2), selection_for_open("vc2")),
            VcProofSelection::new(VcId::new(0), selection_for_open("vc0")),
        ],
        [VcProofSelection::new(
            VcId::new(1),
            selection_for_builtin("vc1"),
        )],
    )
    .expect("merge succeeds");

    assert_eq!(
        merged
            .iter()
            .map(ArtifactProofSelection::vc)
            .collect::<Vec<_>>(),
        vec![VcId::new(0), VcId::new(1), VcId::new(2)]
    );
}

#[test]
fn artifact_merge_preserves_trusted_class_precedence() {
    let builtin_over_external = merge_artifact_proof_selections(
        [VcProofSelection::new(
            VcId::new(0),
            selection_for_external("external"),
        )],
        [VcProofSelection::new(
            VcId::new(0),
            selection_for_builtin("builtin"),
        )],
    )
    .expect("merge succeeds");
    assert_eq!(
        builtin_over_external[0].source(),
        ProofSelectionSource::BuiltinDischarge
    );
    assert_eq!(
        builtin_over_external[0].selected_class(),
        ProofWinnerClass::DischargedBuiltin
    );

    let kernel_over_builtin = merge_artifact_proof_selections(
        [VcProofSelection::new(
            VcId::new(0),
            selection_for_kernel("kernel"),
        )],
        [VcProofSelection::new(
            VcId::new(0),
            selection_for_builtin("builtin"),
        )],
    )
    .expect("merge succeeds");
    assert_eq!(
        kernel_over_builtin[0].source(),
        ProofSelectionSource::Portfolio
    );
    assert_eq!(
        kernel_over_builtin[0].selected_class(),
        ProofWinnerClass::KernelVerified
    );
}

#[test]
fn artifact_merge_uses_tie_break_hash_before_source_rank() {
    let first = selection_for_rejected("same-class-a");
    let second = selection_for_rejected("same-class-b");
    assert_ne!(tie_hash_bytes(&first), tie_hash_bytes(&second));

    let (lower_hash, higher_hash) = if tie_hash_bytes(&first) < tie_hash_bytes(&second) {
        (first, second)
    } else {
        (second, first)
    };
    let expected_candidate_id = lower_hash.selected_candidate_id().cloned();

    let merged = merge_artifact_proof_selections(
        [VcProofSelection::new(VcId::new(0), higher_hash)],
        [VcProofSelection::new(VcId::new(0), lower_hash)],
    )
    .expect("merge succeeds");

    assert_eq!(merged[0].source(), ProofSelectionSource::BuiltinDischarge);
    assert_eq!(merged[0].selected_class(), ProofWinnerClass::Rejected);
    assert_eq!(
        merged[0].selection().selected_candidate_id(),
        expected_candidate_id.as_ref()
    );
}

#[test]
fn artifact_merge_uses_source_rank_for_equal_tie_break_hash() {
    let shared = selection_for_rejected("same-class-shared");

    let merged = merge_artifact_proof_selections(
        [VcProofSelection::new(VcId::new(0), shared.clone())],
        [VcProofSelection::new(VcId::new(0), shared)],
    )
    .expect("merge succeeds");

    assert_eq!(merged[0].source(), ProofSelectionSource::Portfolio);
    assert_eq!(merged[0].selected_class(), ProofWinnerClass::Rejected);
}

#[test]
fn artifact_merge_preserves_non_trusted_outcomes() {
    let merged = merge_artifact_proof_selections(
        [
            VcProofSelection::new(VcId::new(0), selection_for_external("external")),
            VcProofSelection::new(VcId::new(1), selection_for_assumed("assumed")),
            VcProofSelection::new(VcId::new(2), selection_for_open("open")),
            VcProofSelection::new(VcId::new(3), selection_for_rejected("rejected")),
            VcProofSelection::new(VcId::new(4), selection_for_no_selectable()),
        ],
        [],
    )
    .expect("merge succeeds");

    assert_eq!(
        merged
            .iter()
            .map(ArtifactProofSelection::selected_class)
            .collect::<Vec<_>>(),
        vec![
            ProofWinnerClass::PolicyPermittedExternal,
            ProofWinnerClass::PolicyAssumed,
            ProofWinnerClass::PolicyOpen,
            ProofWinnerClass::Rejected,
            ProofWinnerClass::NoSelectableEvidence,
        ]
    );
    assert!(
        merged
            .iter()
            .all(|selection| !selection.selection().trusted_used_axioms_available())
    );
}

#[test]
fn artifact_merge_rejects_duplicate_source_for_one_vc() {
    let portfolio_error = merge_artifact_proof_selections(
        [
            VcProofSelection::new(VcId::new(0), selection_for_open("left")),
            VcProofSelection::new(VcId::new(0), selection_for_open("right")),
        ],
        [],
    )
    .expect_err("duplicate portfolio selection is rejected");

    assert_eq!(
        portfolio_error,
        ArtifactProofSelectionError::DuplicateSelection {
            vc: VcId::new(0),
            source: ProofSelectionSource::Portfolio,
        }
    );

    let builtin_error = merge_artifact_proof_selections(
        [],
        [
            VcProofSelection::new(VcId::new(0), selection_for_builtin("left")),
            VcProofSelection::new(VcId::new(0), selection_for_builtin("right")),
        ],
    )
    .expect_err("duplicate built-in discharge selection is rejected");

    assert_eq!(
        builtin_error,
        ArtifactProofSelectionError::DuplicateSelection {
            vc: VcId::new(0),
            source: ProofSelectionSource::BuiltinDischarge,
        }
    );
}

#[test]
fn artifact_merge_rejects_invalid_source_class_pairs() {
    let portfolio_error = merge_artifact_proof_selections(
        [VcProofSelection::new(
            VcId::new(0),
            selection_for_builtin("builtin"),
        )],
        [],
    )
    .expect_err("portfolio cannot publish built-in discharge selections");

    assert_eq!(
        portfolio_error,
        ArtifactProofSelectionError::InvalidSourceClass {
            vc: VcId::new(0),
            source: ProofSelectionSource::Portfolio,
            selected_class: ProofWinnerClass::DischargedBuiltin,
        }
    );

    let builtin_error = merge_artifact_proof_selections(
        [],
        [VcProofSelection::new(
            VcId::new(0),
            selection_for_external("external"),
        )],
    )
    .expect_err("built-in discharge source cannot publish external selections");

    assert_eq!(
        builtin_error,
        ArtifactProofSelectionError::InvalidSourceClass {
            vc: VcId::new(0),
            source: ProofSelectionSource::BuiltinDischarge,
            selected_class: ProofWinnerClass::PolicyPermittedExternal,
        }
    );
}

fn base_set(policy: VerifierPolicy) -> ProofEvidenceSet {
    ProofEvidenceSet::new(b"obligation".to_vec(), hash(100), policy)
}

fn selection_for_kernel(id: &str) -> ProofSelection {
    select_winner(
        &base_set(VerifierPolicy::release()).with_candidates([trusted_candidate(
            id,
            CandidatePolicyClass::KernelVerified,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            hash(120),
        )]),
    )
}

fn selection_for_builtin(id: &str) -> ProofSelection {
    select_winner(
        &base_set(VerifierPolicy::release()).with_candidates([trusted_candidate(
            id,
            CandidatePolicyClass::DischargedBuiltin,
            KernelEvidenceOrigin::BuiltinDischarge,
            hash(121),
        )
        .with_deterministic_discharge_hash(hash(122))]),
    )
}

fn selection_for_external(id: &str) -> ProofSelection {
    let policy = VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner);
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    select_winner(
        &base_set(policy).with_candidates([candidate(
            id,
            evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
        )
        .with_evidence_hash(hash(123))]),
    )
}

fn selection_for_assumed(id: &str) -> ProofSelection {
    let policy = VerifierPolicy::development()
        .with_external_evidence(ExternalEvidenceMode::Reject)
        .with_policy_assumption(PolicyAssumptionMode::Record);
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    select_winner(
        &base_set(policy).with_candidates([candidate(
            id,
            evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
        )
        .with_evidence_hash(hash(125))]),
    )
}

fn selection_for_open(id: &str) -> ProofSelection {
    let policy = VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::Reject)
        .with_policy_assumption(PolicyAssumptionMode::Reject);
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    select_winner(
        &base_set(policy).with_candidates([candidate(
            id,
            evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
        )
        .with_evidence_hash(hash(124))]),
    )
}

fn selection_for_rejected(id: &str) -> ProofSelection {
    select_winner(
        &base_set(VerifierPolicy::release())
            .with_candidates([candidate(id, rejected_policy_decision())]),
    )
}

fn selection_for_no_selectable() -> ProofSelection {
    select_winner(&base_set(VerifierPolicy::development()))
}

fn tie_hash_bytes(selection: &ProofSelection) -> [u8; Hash::BYTE_LEN] {
    *selection.reuse_metadata().tie_break_key_hash().as_bytes()
}

fn candidate(id: &str, decision: PolicyDecision) -> ProofEvidenceCandidate {
    ProofEvidenceCandidate::new(CandidateSourceId::new(id).expect("stable id"), decision)
}

fn trusted_candidate(
    id: &str,
    class: CandidatePolicyClass,
    origin: KernelEvidenceOrigin,
    evidence_hash: Hash,
) -> ProofEvidenceCandidate {
    let kernel_input = KernelPolicyInput::for_test(
        KernelCheckStatus::Accepted,
        origin,
        false,
        Some(evidence_hash),
    );
    let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
        CandidateSourceId::new(id).expect("stable id"),
        &kernel_input,
    )
    .expect("accepted kernel input is trusted evidence");
    assert_eq!(candidate.decision().class, class);
    candidate
}

fn rejected_policy_decision() -> PolicyDecision {
    rejected_policy_decision_with_reason(PolicyReasonCode::OpenObligationRejected)
}

fn rejected_policy_decision_with_reason(reason: PolicyReasonCode) -> PolicyDecision {
    PolicyDecision {
        class: CandidatePolicyClass::RejectedByPolicy,
        can_schedule_kernel_check: false,
        diagnostic: Some(PolicyDiagnostic::new(
            PolicyDiagnosticCategory::PolicyRejection,
            reason,
        )),
        kernel_rejections: Vec::new(),
        external_admission: None,
    }
}

fn kernel_rejected_decision_with_record(record: RejectionRecord) -> PolicyDecision {
    PolicyDecision {
        class: CandidatePolicyClass::KernelRejected,
        can_schedule_kernel_check: false,
        diagnostic: None,
        kernel_rejections: vec![record],
        external_admission: None,
    }
}

fn rejection_record(category: RejectionCategory, detail: RejectionDetail) -> RejectionRecord {
    RejectionRecord::new(
        TargetVcFingerprint::new(1, vec![1, 2, 3]),
        category,
        detail,
        RejectionLocation::new(),
    )
    .expect("valid rejection record")
}

fn assert_selected(
    evidence_set: &ProofEvidenceSet,
    expected_class: ProofWinnerClass,
    expected_id: &str,
) {
    let selection = select_winner(evidence_set);
    assert_eq!(selection.selected_class(), expected_class);
    assert_eq!(
        selection
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some(expected_id)
    );
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}
