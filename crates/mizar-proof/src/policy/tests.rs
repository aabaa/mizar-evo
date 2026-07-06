use crate::selection::{
    CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, TrustedKernelEvidence,
    select_winner,
};

use super::*;

#[test]
fn explicit_kernel_origin_controls_trusted_class() {
    let evaluator = ProofPolicyEvaluator::new(VerifierPolicy::release());

    assert_eq!(
        evaluator.candidate_class(&PolicyCandidate::KernelResult(kernel_input_for_test(
            KernelCheckStatus::Accepted,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            false
        ))),
        CandidatePolicyClass::KernelVerified
    );
    assert_eq!(
        kernel_input_for_test(
            KernelCheckStatus::Accepted,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            false
        )
        .accepted_goal_polarity(),
        Some(AcceptedGoalPolarity::AssertFalseForRefutation)
    );
    assert_eq!(
        evaluator.candidate_class(&PolicyCandidate::KernelResult(kernel_input_for_test(
            KernelCheckStatus::Accepted,
            KernelEvidenceOrigin::BuiltinDischarge,
            false
        ))),
        CandidatePolicyClass::DischargedBuiltin
    );
    assert_eq!(
        evaluator.candidate_class(&PolicyCandidate::KernelResult(kernel_input_for_test(
            KernelCheckStatus::Accepted,
            KernelEvidenceOrigin::KernelPrimitive,
            false
        ))),
        CandidatePolicyClass::DischargedBuiltin
    );
    let rejected =
        evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(kernel_input_for_test(
            KernelCheckStatus::Rejected,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            false,
        )));
    assert_eq!(rejected.class, CandidatePolicyClass::KernelRejected);
    assert_eq!(rejected.diagnostic, None);
    assert!(
        rejected.kernel_rejections.is_empty(),
        "test fixture has no real kernel rejection records"
    );
}

#[test]
fn policy_tainted_kernel_results_do_not_become_trusted() {
    for origin in [
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        KernelEvidenceOrigin::BuiltinDischarge,
        KernelEvidenceOrigin::KernelPrimitive,
    ] {
        assert_policy_tainted_admission(
            VerifierPolicy::release(),
            origin,
            CandidatePolicyClass::RejectedByPolicy,
            ExternalEvidencePublicationStatus::RejectedByPolicy,
            PolicyDiagnosticCategory::PolicyRejection,
            PolicyReasonCode::ExternalEvidenceRejected,
        );
        assert_policy_tainted_admission(
            VerifierPolicy::development(),
            origin,
            CandidatePolicyClass::ExternallyAttested,
            ExternalEvidencePublicationStatus::ExternallyAttestedDevelopment,
            PolicyDiagnosticCategory::PolicyOpen,
            PolicyReasonCode::ExternalEvidenceRecorded,
        );
        assert_policy_tainted_admission(
            VerifierPolicy::development()
                .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner),
            origin,
            CandidatePolicyClass::ExternallyAttested,
            ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted,
            PolicyDiagnosticCategory::PolicyOpen,
            PolicyReasonCode::ExternalEvidencePolicyPermitted,
        );
    }
}

#[test]
fn accepted_consistency_kernel_results_are_diagnostic_only() {
    for policy in [
        VerifierPolicy::release(),
        VerifierPolicy::development()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner),
    ] {
        for policy_taint in [false, true] {
            let evaluator = ProofPolicyEvaluator::new(policy.clone());
            let input = KernelPolicyInput::for_test_with_check_kind(
                KernelCheckStatus::Accepted,
                KernelEvidenceOrigin::AtpFormulaSubstitution,
                Some(KernelEvidenceCheckKind::ConsistencyCheck),
                policy_taint,
                Some(hash(7)),
            );
            let candidate = PolicyCandidate::KernelResult(input.clone());

            assert_eq!(input.accepted_evidence_hash(), None);
            assert_eq!(input.accepted_goal_polarity(), None);
            assert_eq!(TrustedKernelEvidence::from_policy_input(&input), None);
            assert_eq!(
                evaluator.candidate_class(&candidate),
                CandidatePolicyClass::DiagnosticOnly
            );
            assert_eq!(evaluator.best_possible_early_stop_class(&candidate), None);

            let decision = evaluator.evaluate_candidate(&candidate);
            assert_eq!(decision.class, CandidatePolicyClass::DiagnosticOnly);
            assert_eq!(decision.diagnostic, None);
            assert!(decision.kernel_rejections.is_empty());
            assert!(decision.external_admission.is_none());
        }
    }
}

#[test]
fn schedules_only_kernel_checkable_evidence() {
    let evaluator = ProofPolicyEvaluator::new(VerifierPolicy::release());

    for candidate in [
        PolicyCandidate::UncheckedFormulaSubstitution {
            encoded_problem_matches: true,
        },
        PolicyCandidate::UncheckedBuiltinDischarge {
            has_stable_kernel_representation: true,
        },
        PolicyCandidate::KernelPrimitive {
            allowed_by_policy: true,
        },
    ] {
        let decision = evaluator.evaluate_candidate(&candidate);
        assert_eq!(decision.class, CandidatePolicyClass::KernelCheckable);
        assert!(decision.can_schedule_kernel_check);
    }

    for candidate in [
        PolicyCandidate::UncheckedFormulaSubstitution {
            encoded_problem_matches: false,
        },
        PolicyCandidate::UncheckedBuiltinDischarge {
            has_stable_kernel_representation: false,
        },
        PolicyCandidate::KernelPrimitive {
            allowed_by_policy: false,
        },
        PolicyCandidate::ExternallyAttested,
        PolicyCandidate::CacheRecord,
        PolicyCandidate::BackendReportedUsedAxioms,
    ] {
        assert!(!evaluator.can_schedule_kernel_check(&candidate));
    }
}

#[test]
fn rejected_kernel_checkable_inputs_have_stable_reasons() {
    let release = ProofPolicyEvaluator::new(VerifierPolicy::release());
    assert_policy_rejection(
        release.evaluate_candidate(&PolicyCandidate::UncheckedFormulaSubstitution {
            encoded_problem_matches: false,
        }),
        PolicyReasonCode::KernelEvidenceTargetMismatch,
    );
    assert_policy_rejection(
        release.evaluate_candidate(&PolicyCandidate::UncheckedBuiltinDischarge {
            has_stable_kernel_representation: false,
        }),
        PolicyReasonCode::MissingBuiltinKernelRepresentation,
    );
    assert_policy_rejection(
        release.evaluate_candidate(&PolicyCandidate::KernelPrimitive {
            allowed_by_policy: false,
        }),
        PolicyReasonCode::KernelPrimitiveNotAllowed,
    );

    let no_formula = ProofPolicyEvaluator::new(
        VerifierPolicy::release()
            .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]),
    );
    assert_policy_rejection(
        no_formula.evaluate_candidate(&PolicyCandidate::UncheckedFormulaSubstitution {
            encoded_problem_matches: true,
        }),
        PolicyReasonCode::KernelEvidenceFormatDisabled,
    );

    let no_builtin = ProofPolicyEvaluator::new(
        VerifierPolicy::release()
            .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
    );
    assert_policy_rejection(
        no_builtin.evaluate_candidate(&PolicyCandidate::UncheckedBuiltinDischarge {
            has_stable_kernel_representation: true,
        }),
        PolicyReasonCode::KernelEvidenceFormatDisabled,
    );
    assert_policy_rejection(
        no_builtin.evaluate_candidate(&PolicyCandidate::KernelPrimitive {
            allowed_by_policy: true,
        }),
        PolicyReasonCode::KernelEvidenceFormatDisabled,
    );
}

#[test]
fn classifies_non_kernel_policy_and_diagnostic_inputs() {
    let release = ProofPolicyEvaluator::new(VerifierPolicy::release());
    let development = ProofPolicyEvaluator::new(VerifierPolicy::development());
    let interactive = ProofPolicyEvaluator::new(VerifierPolicy::interactive());

    assert_eq!(
        release.candidate_class(&PolicyCandidate::ExternallyAttested),
        CandidatePolicyClass::RejectedByPolicy
    );
    assert_eq!(
        development.candidate_class(&PolicyCandidate::ExternallyAttested),
        CandidatePolicyClass::ExternallyAttested
    );
    assert_eq!(
        release.candidate_class(&PolicyCandidate::OpenObligation),
        CandidatePolicyClass::RejectedByPolicy
    );
    assert_eq!(
        interactive.candidate_class(&PolicyCandidate::OpenObligation),
        CandidatePolicyClass::OpenAllowed
    );
    assert_eq!(
        release.candidate_class(&PolicyCandidate::PolicyAssumption),
        CandidatePolicyClass::RejectedByPolicy
    );
    assert_eq!(
        development.candidate_class(&PolicyCandidate::PolicyAssumption),
        CandidatePolicyClass::AssumedByPolicy
    );

    for candidate in [
        PolicyCandidate::BackendDiagnostic,
        PolicyCandidate::BackendProofPayload(BackendProofPayloadKind::BackendProofMethod),
        PolicyCandidate::BackendProofPayload(BackendProofPayloadKind::ResolutionTrace),
        PolicyCandidate::BackendProofPayload(BackendProofPayloadKind::SmtProofObject),
        PolicyCandidate::BackendProofPayload(BackendProofPayloadKind::TstpTrace),
        PolicyCandidate::BackendProofPayload(BackendProofPayloadKind::UnsatCore),
        PolicyCandidate::BackendReportedUsedAxioms,
        PolicyCandidate::CacheRecord,
        PolicyCandidate::Counterexample,
        PolicyCandidate::TimingRecord,
        PolicyCandidate::UnsupportedProofPayload,
    ] {
        assert_eq!(
            development.candidate_class(&candidate),
            CandidatePolicyClass::DiagnosticOnly
        );
    }

    assert_policy_rejection(
        development.evaluate_candidate(&PolicyCandidate::LegacyReplay),
        PolicyReasonCode::LegacyReplayRejected,
    );
}

#[test]
fn externally_attested_admission_matrix_is_policy_driven() {
    let cases = [
        external_case(ExternalEvidenceMode::Reject, false, BuildMode::Release),
        external_case(ExternalEvidenceMode::Reject, false, BuildMode::Development),
        external_case(ExternalEvidenceMode::Reject, false, BuildMode::Interactive),
        external_case(ExternalEvidenceMode::Reject, true, BuildMode::Release),
        external_case(ExternalEvidenceMode::Reject, true, BuildMode::Development),
        external_case(ExternalEvidenceMode::Reject, true, BuildMode::Interactive),
        external_case(
            ExternalEvidenceMode::RecordDevelopment,
            false,
            BuildMode::Release,
        ),
        external_case(
            ExternalEvidenceMode::RecordDevelopment,
            false,
            BuildMode::Development,
        ),
        external_case(
            ExternalEvidenceMode::RecordDevelopment,
            false,
            BuildMode::Interactive,
        ),
        external_case(
            ExternalEvidenceMode::RecordDevelopment,
            true,
            BuildMode::Release,
        ),
        external_case(
            ExternalEvidenceMode::RecordDevelopment,
            true,
            BuildMode::Development,
        ),
        external_case(
            ExternalEvidenceMode::RecordDevelopment,
            true,
            BuildMode::Interactive,
        ),
        external_case(
            ExternalEvidenceMode::PermitNonTrustedWinner,
            false,
            BuildMode::Release,
        ),
        external_case(
            ExternalEvidenceMode::PermitNonTrustedWinner,
            false,
            BuildMode::Development,
        ),
        external_case(
            ExternalEvidenceMode::PermitNonTrustedWinner,
            false,
            BuildMode::Interactive,
        ),
        external_case(
            ExternalEvidenceMode::PermitNonTrustedWinner,
            true,
            BuildMode::Release,
        ),
        external_case(
            ExternalEvidenceMode::PermitNonTrustedWinner,
            true,
            BuildMode::Development,
        ),
        external_case(
            ExternalEvidenceMode::PermitNonTrustedWinner,
            true,
            BuildMode::Interactive,
        ),
    ];

    for case in cases {
        assert_external_admission(case);
    }
}

#[test]
fn early_stop_requires_observed_selectable_class() {
    let release = ProofPolicyEvaluator::new(VerifierPolicy::release());

    let missing = release.portfolio_early_stop_decision(&PortfolioEarlyStopInput::new(
        None,
        [PortfolioEarlyStopClass::KernelVerified],
    ));
    assert!(!missing.may_stop());
    assert_eq!(
        missing.reason(),
        PortfolioEarlyStopReason::NoObservedCandidate
    );
    assert_eq!(missing.observed_best_class(), None);
    assert_eq!(missing.blocking_pending_class(), None);

    let unselectable = release.portfolio_early_stop_decision(&PortfolioEarlyStopInput::new(
        Some(PortfolioEarlyStopClass::PolicyPermittedExternal),
        [],
    ));
    assert!(!unselectable.may_stop());
    assert_eq!(
        unselectable.reason(),
        PortfolioEarlyStopReason::ObservedClassNotSelectable
    );
    assert_eq!(
        unselectable.observed_best_class(),
        Some(PortfolioEarlyStopClass::PolicyPermittedExternal)
    );
    assert_eq!(unselectable.blocking_pending_class(), None);
}

#[test]
fn early_stop_blocks_equal_or_higher_pending_classes() {
    let release = ProofPolicyEvaluator::new(VerifierPolicy::release());
    let equal = release.portfolio_early_stop_decision(&PortfolioEarlyStopInput::new(
        Some(PortfolioEarlyStopClass::KernelVerified),
        [PortfolioEarlyStopClass::KernelVerified],
    ));
    assert!(!equal.may_stop());
    assert_eq!(
        equal.reason(),
        PortfolioEarlyStopReason::BlockedByEqualClass
    );
    assert_eq!(
        equal.blocking_pending_class(),
        Some(PortfolioEarlyStopClass::KernelVerified)
    );

    let interactive = ProofPolicyEvaluator::new(
        VerifierPolicy::interactive().with_policy_assumption(PolicyAssumptionMode::Record),
    );
    let higher = interactive.portfolio_early_stop_decision(&PortfolioEarlyStopInput::new(
        Some(PortfolioEarlyStopClass::PolicyOpen),
        [
            PortfolioEarlyStopClass::PolicyAssumed,
            PortfolioEarlyStopClass::KernelVerified,
        ],
    ));
    assert!(!higher.may_stop());
    assert_eq!(
        higher.reason(),
        PortfolioEarlyStopReason::BlockedByHigherClass
    );
    assert_eq!(
        higher.blocking_pending_class(),
        Some(PortfolioEarlyStopClass::KernelVerified)
    );
}

#[test]
fn early_stop_allows_when_pending_classes_cannot_displace() {
    let release = ProofPolicyEvaluator::new(VerifierPolicy::release());
    let trusted = release.portfolio_early_stop_decision(&PortfolioEarlyStopInput::new(
        Some(PortfolioEarlyStopClass::KernelVerified),
        [
            PortfolioEarlyStopClass::PolicyPermittedExternal,
            PortfolioEarlyStopClass::PolicyAssumed,
            PortfolioEarlyStopClass::PolicyOpen,
        ],
    ));
    assert!(trusted.may_stop());
    assert_eq!(
        trusted.reason(),
        PortfolioEarlyStopReason::NoDisplacingPendingClass
    );
    assert_eq!(
        trusted.observed_best_class(),
        Some(PortfolioEarlyStopClass::KernelVerified)
    );
    assert_eq!(trusted.blocking_pending_class(), None);

    let interactive = ProofPolicyEvaluator::new(VerifierPolicy::interactive());
    let open = interactive.portfolio_early_stop_decision(&PortfolioEarlyStopInput::new(
        Some(PortfolioEarlyStopClass::PolicyAssumed),
        [PortfolioEarlyStopClass::PolicyOpen],
    ));
    assert!(open.may_stop());
    assert_eq!(
        open.reason(),
        PortfolioEarlyStopReason::NoDisplacingPendingClass
    );
}

#[test]
fn early_stop_candidate_class_normalization_is_policy_driven() {
    let release = ProofPolicyEvaluator::new(VerifierPolicy::release());
    assert_eq!(
        release.best_possible_early_stop_class(&PolicyCandidate::KernelResult(
            kernel_input_for_test(
                KernelCheckStatus::Accepted,
                KernelEvidenceOrigin::AtpFormulaSubstitution,
                false,
            )
        )),
        Some(PortfolioEarlyStopClass::KernelVerified)
    );
    assert_eq!(
        release.best_possible_early_stop_class(&PolicyCandidate::KernelResult(
            kernel_input_for_test(
                KernelCheckStatus::Accepted,
                KernelEvidenceOrigin::BuiltinDischarge,
                false,
            )
        )),
        Some(PortfolioEarlyStopClass::DischargedBuiltin)
    );
    assert_eq!(
        release.best_possible_early_stop_class(&PolicyCandidate::UncheckedFormulaSubstitution {
            encoded_problem_matches: true,
        }),
        Some(PortfolioEarlyStopClass::KernelVerified)
    );
    assert_eq!(
        release.best_possible_early_stop_class(&PolicyCandidate::UncheckedBuiltinDischarge {
            has_stable_kernel_representation: true,
        }),
        Some(PortfolioEarlyStopClass::DischargedBuiltin)
    );

    for candidate in [
        PolicyCandidate::ExternallyAttested,
        PolicyCandidate::BackendDiagnostic,
        PolicyCandidate::BackendReportedUsedAxioms,
        PolicyCandidate::BackendProofPayload(BackendProofPayloadKind::ResolutionTrace),
        PolicyCandidate::CacheRecord,
        PolicyCandidate::TimingRecord,
    ] {
        assert_eq!(release.best_possible_early_stop_class(&candidate), None);
    }

    let external_permitted = ProofPolicyEvaluator::new(
        VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner),
    );
    assert_eq!(
        external_permitted.best_possible_early_stop_class(&PolicyCandidate::ExternallyAttested),
        Some(PortfolioEarlyStopClass::PolicyPermittedExternal)
    );
    assert_eq!(
        external_permitted.best_possible_early_stop_class(&PolicyCandidate::KernelResult(
            kernel_input_for_test(
                KernelCheckStatus::Accepted,
                KernelEvidenceOrigin::AtpFormulaSubstitution,
                true,
            )
        )),
        Some(PortfolioEarlyStopClass::PolicyPermittedExternal)
    );

    let development = ProofPolicyEvaluator::new(VerifierPolicy::development());
    assert_eq!(
        development.best_possible_early_stop_class(&PolicyCandidate::PolicyAssumption),
        Some(PortfolioEarlyStopClass::PolicyAssumed)
    );
    assert_eq!(
        ProofPolicyEvaluator::new(VerifierPolicy::interactive())
            .best_possible_early_stop_class(&PolicyCandidate::OpenObligation),
        Some(PortfolioEarlyStopClass::PolicyOpen)
    );

    let no_formula = ProofPolicyEvaluator::new(
        VerifierPolicy::release()
            .with_kernel_evidence_formats([KernelEvidenceFormat::BuiltinKernelEvidence]),
    );
    assert_eq!(
        no_formula.best_possible_early_stop_class(&PolicyCandidate::UncheckedFormulaSubstitution {
            encoded_problem_matches: true,
        }),
        None
    );
}

#[test]
fn early_stop_allowed_matches_full_selection_result() {
    let policy = VerifierPolicy::release();
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let observed = trusted_selection_candidate(
        "observed",
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(1),
    )
    .with_backend_profile_priority(0)
    .with_evidence_format_priority(0);
    let lower_pending = selection_candidate(
        "external",
        evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
    )
    .with_evidence_hash(hash(2));

    let decision = evaluator.portfolio_early_stop_decision(&PortfolioEarlyStopInput::new(
        Some(PortfolioEarlyStopClass::KernelVerified),
        [PortfolioEarlyStopClass::PolicyPermittedExternal],
    ));
    assert!(decision.may_stop());

    let observed_only = select_winner(&selection_set(policy.clone(), [observed.clone()]));
    let full_run = select_winner(&selection_set(policy, [observed, lower_pending]));
    assert_eq!(
        observed_only.selected_candidate_id(),
        full_run.selected_candidate_id()
    );
    assert_eq!(
        full_run
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("observed")
    );
}

#[test]
fn early_stop_allowed_matches_full_selection_with_selectable_lower_class() {
    let policy = VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
        .with_policy_assumption(PolicyAssumptionMode::Record);
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let observed = selection_candidate(
        "external",
        evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
    )
    .with_evidence_hash(hash(11));
    let lower_pending = selection_candidate(
        "assumed",
        evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
    )
    .with_evidence_hash(hash(12));

    let decision = evaluator.portfolio_early_stop_decision(&PortfolioEarlyStopInput::new(
        Some(PortfolioEarlyStopClass::PolicyPermittedExternal),
        [PortfolioEarlyStopClass::PolicyAssumed],
    ));
    assert!(decision.may_stop());

    let observed_only = select_winner(&selection_set(policy.clone(), [observed.clone()]));
    let full_run = select_winner(&selection_set(policy, [observed, lower_pending]));
    assert_eq!(
        observed_only.selected_candidate_id(),
        full_run.selected_candidate_id()
    );
    assert_eq!(
        full_run
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("external")
    );
}

#[test]
fn equal_pending_class_blocks_when_full_selection_could_change() {
    let policy = VerifierPolicy::release();
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let observed = trusted_selection_candidate(
        "observed",
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(5),
    )
    .with_backend_profile_priority(1)
    .with_evidence_format_priority(0);
    let equal_pending = trusted_selection_candidate(
        "pending",
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(4),
    )
    .with_backend_profile_priority(0)
    .with_evidence_format_priority(0);

    let decision = evaluator.portfolio_early_stop_decision(&PortfolioEarlyStopInput::new(
        Some(PortfolioEarlyStopClass::KernelVerified),
        [PortfolioEarlyStopClass::KernelVerified],
    ));
    assert!(!decision.may_stop());
    assert_eq!(
        decision.reason(),
        PortfolioEarlyStopReason::BlockedByEqualClass
    );

    let observed_only = select_winner(&selection_set(policy.clone(), [observed.clone()]));
    let full_run = select_winner(&selection_set(policy, [observed, equal_pending]));
    assert_eq!(
        observed_only
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("observed")
    );
    assert_eq!(
        full_run
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("pending")
    );
}

#[test]
fn fingerprint_changes_for_each_policy_relevant_setting() {
    let base = VerifierPolicy::release();
    let base_fingerprint = base.policy_fingerprint();

    for changed in [
        base.clone().with_schema_version(POLICY_SCHEMA_VERSION + 1),
        base.clone().with_profile_id("release-alt"),
        base.clone().with_build_mode(BuildMode::Development),
        base.clone().with_require_kernel_certificates(false),
        base.clone()
            .with_external_evidence(ExternalEvidenceMode::RecordDevelopment),
        base.clone()
            .with_open_obligation(OpenObligationMode::RecordDiagnostic),
        base.clone()
            .with_policy_assumption(PolicyAssumptionMode::Record),
        base.clone()
            .with_kernel_evidence_formats([KernelEvidenceFormat::FormulaSubstitution]),
        base.clone()
            .with_checker_schema_version(DEFAULT_CHECKER_SCHEMA_VERSION + 1),
    ] {
        assert_ne!(changed.policy_fingerprint(), base_fingerprint);
    }
}

#[test]
fn fingerprint_sorts_kernel_evidence_formats_and_ignores_candidate_order() {
    let left = VerifierPolicy::release().with_kernel_evidence_formats([
        KernelEvidenceFormat::BuiltinKernelEvidence,
        KernelEvidenceFormat::FormulaSubstitution,
    ]);
    let right = VerifierPolicy::release().with_kernel_evidence_formats([
        KernelEvidenceFormat::FormulaSubstitution,
        KernelEvidenceFormat::BuiltinKernelEvidence,
    ]);
    let evaluator = ProofPolicyEvaluator::new(left);
    let initial = evaluator.policy_fingerprint();

    assert_eq!(initial, right.policy_fingerprint());
    assert_eq!(initial.as_bytes().len(), Hash::BYTE_LEN);
    assert_eq!(initial.to_lower_hex().len(), Hash::BYTE_LEN * 2);

    for candidate in [
        PolicyCandidate::CacheRecord,
        PolicyCandidate::UncheckedFormulaSubstitution {
            encoded_problem_matches: true,
        },
        PolicyCandidate::TimingRecord,
    ] {
        let _ = evaluator.evaluate_candidate(&candidate);
    }
    assert_eq!(evaluator.policy_fingerprint(), initial);
}

fn kernel_input_for_test(
    status: KernelCheckStatus,
    origin: KernelEvidenceOrigin,
    policy_taint: bool,
) -> KernelPolicyInput {
    KernelPolicyInput::for_test(status, origin, policy_taint, None)
}

fn selection_set(
    policy: VerifierPolicy,
    candidates: impl IntoIterator<Item = ProofEvidenceCandidate>,
) -> ProofEvidenceSet {
    ProofEvidenceSet::new(b"obligation".to_vec(), hash(100), policy).with_candidates(candidates)
}

fn selection_candidate(id: &str, decision: PolicyDecision) -> ProofEvidenceCandidate {
    ProofEvidenceCandidate::new(CandidateSourceId::new(id).expect("stable id"), decision)
}

fn trusted_selection_candidate(
    id: &str,
    origin: KernelEvidenceOrigin,
    evidence_hash: Hash,
) -> ProofEvidenceCandidate {
    let input = KernelPolicyInput::for_test(
        KernelCheckStatus::Accepted,
        origin,
        false,
        Some(evidence_hash),
    );
    ProofEvidenceCandidate::from_trusted_kernel_input(
        CandidateSourceId::new(id).expect("stable id"),
        &input,
    )
    .expect("accepted proof-obligation kernel input is trusted evidence")
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn assert_policy_rejection(decision: PolicyDecision, reason: PolicyReasonCode) {
    assert_eq!(decision.class, CandidatePolicyClass::RejectedByPolicy);
    assert!(!decision.can_schedule_kernel_check);
    assert!(decision.kernel_rejections.is_empty());
    assert!(decision.external_admission.is_none());
    assert_eq!(
        decision.diagnostic,
        Some(PolicyDiagnostic::new(
            PolicyDiagnosticCategory::PolicyRejection,
            reason
        ))
    );
}

fn assert_policy_tainted_admission(
    policy: VerifierPolicy,
    origin: KernelEvidenceOrigin,
    expected_class: CandidatePolicyClass,
    expected_status: ExternalEvidencePublicationStatus,
    expected_category: PolicyDiagnosticCategory,
    expected_reason: PolicyReasonCode,
) {
    let evaluator = ProofPolicyEvaluator::new(policy);
    let input = kernel_input_for_test(KernelCheckStatus::Accepted, origin, true);
    assert_eq!(input.accepted_goal_polarity(), None);
    assert_eq!(TrustedKernelEvidence::from_policy_input(&input), None);
    let decision = evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(input));
    let expected_diagnostic = Some(PolicyDiagnostic::new(expected_category, expected_reason));

    assert_eq!(decision.class, expected_class);
    assert!(!decision.can_schedule_kernel_check);
    assert!(decision.kernel_rejections.is_empty());
    assert_eq!(decision.diagnostic, expected_diagnostic);

    let admission = decision
        .external_admission
        .expect("policy-tainted kernel input carries external admission");
    assert_eq!(evaluator.external_evidence_admission(), admission);
    assert_eq!(admission.policy_class(), expected_class);
    assert_eq!(admission.publication_status(), expected_status);
    assert_eq!(admission.diagnostic().cloned(), expected_diagnostic);
    assert!(!admission.trusted_used_axioms_allowed());
}

fn assert_external_admission(case: ExternalAdmissionCase) {
    let evaluator = ProofPolicyEvaluator::new(
        VerifierPolicy::release()
            .with_external_evidence(case.external_evidence)
            .with_require_kernel_certificates(case.require_kernel_certificates)
            .with_build_mode(case.build_mode),
    );
    let decision = evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested);
    let expected_diagnostic = Some(PolicyDiagnostic::new(
        case.expected_category,
        case.expected_reason,
    ));

    assert_eq!(decision.class, case.expected_class);
    assert!(!decision.can_schedule_kernel_check);
    assert!(decision.kernel_rejections.is_empty());
    assert_eq!(decision.diagnostic, expected_diagnostic);

    let admission = decision
        .external_admission
        .expect("external candidate carries admission result");
    assert_eq!(evaluator.external_evidence_admission(), admission);
    assert_eq!(
        admission.record_as_development_evidence(),
        case.expected_record
    );
    assert_eq!(admission.may_win_selection(), case.expected_may_win);
    assert_eq!(admission.publication_status(), case.expected_status);
    assert_eq!(admission.diagnostic().cloned(), expected_diagnostic);
    assert!(!admission.trusted_used_axioms_allowed());
}

fn external_case(
    external_evidence: ExternalEvidenceMode,
    require_kernel_certificates: bool,
    build_mode: BuildMode,
) -> ExternalAdmissionCase {
    let (
        expected_class,
        expected_record,
        expected_may_win,
        expected_status,
        expected_category,
        expected_reason,
    ) = match external_evidence {
        ExternalEvidenceMode::Reject => (
            CandidatePolicyClass::RejectedByPolicy,
            false,
            false,
            ExternalEvidencePublicationStatus::RejectedByPolicy,
            PolicyDiagnosticCategory::PolicyRejection,
            PolicyReasonCode::ExternalEvidenceRejected,
        ),
        ExternalEvidenceMode::RecordDevelopment if require_kernel_certificates => {
            external_require_kernel_case(build_mode)
        }
        ExternalEvidenceMode::RecordDevelopment => external_record_case(build_mode),
        ExternalEvidenceMode::PermitNonTrustedWinner if require_kernel_certificates => {
            external_require_kernel_case(build_mode)
        }
        ExternalEvidenceMode::PermitNonTrustedWinner => (
            CandidatePolicyClass::ExternallyAttested,
            true,
            true,
            ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted,
            PolicyDiagnosticCategory::PolicyOpen,
            PolicyReasonCode::ExternalEvidencePolicyPermitted,
        ),
    };

    ExternalAdmissionCase {
        external_evidence,
        require_kernel_certificates,
        build_mode,
        expected_class,
        expected_record,
        expected_may_win,
        expected_status,
        expected_category,
        expected_reason,
    }
}

fn external_require_kernel_case(
    build_mode: BuildMode,
) -> (
    CandidatePolicyClass,
    bool,
    bool,
    ExternalEvidencePublicationStatus,
    PolicyDiagnosticCategory,
    PolicyReasonCode,
) {
    match build_mode {
        BuildMode::Interactive => (
            CandidatePolicyClass::ExternallyAttested,
            true,
            false,
            ExternalEvidencePublicationStatus::ExternallyAttestedOpenDiagnostic,
            PolicyDiagnosticCategory::PolicyOpen,
            PolicyReasonCode::ExternalEvidenceRecorded,
        ),
        BuildMode::Release | BuildMode::Development => (
            CandidatePolicyClass::RejectedByPolicy,
            true,
            false,
            ExternalEvidencePublicationStatus::RejectedByPolicy,
            PolicyDiagnosticCategory::PolicyRejection,
            PolicyReasonCode::ExternalEvidenceRequiresKernelCertificate,
        ),
    }
}

fn external_record_case(
    build_mode: BuildMode,
) -> (
    CandidatePolicyClass,
    bool,
    bool,
    ExternalEvidencePublicationStatus,
    PolicyDiagnosticCategory,
    PolicyReasonCode,
) {
    match build_mode {
        BuildMode::Interactive => (
            CandidatePolicyClass::ExternallyAttested,
            true,
            false,
            ExternalEvidencePublicationStatus::ExternallyAttestedOpenDiagnostic,
            PolicyDiagnosticCategory::PolicyOpen,
            PolicyReasonCode::ExternalEvidenceRecorded,
        ),
        BuildMode::Release | BuildMode::Development => (
            CandidatePolicyClass::ExternallyAttested,
            true,
            false,
            ExternalEvidencePublicationStatus::ExternallyAttestedDevelopment,
            PolicyDiagnosticCategory::PolicyOpen,
            PolicyReasonCode::ExternalEvidenceRecorded,
        ),
    }
}

struct ExternalAdmissionCase {
    external_evidence: ExternalEvidenceMode,
    require_kernel_certificates: bool,
    build_mode: BuildMode,
    expected_class: CandidatePolicyClass,
    expected_record: bool,
    expected_may_win: bool,
    expected_status: ExternalEvidencePublicationStatus,
    expected_category: PolicyDiagnosticCategory,
    expected_reason: PolicyReasonCode,
}
