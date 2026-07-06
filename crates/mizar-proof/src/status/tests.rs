use crate::{
    policy::{
        AcceptedGoalPolarity, CandidatePolicyClass, ExternalEvidenceMode, KernelPolicyInput,
        PolicyCandidate, PolicyDecision, PolicyDiagnostic, PolicyDiagnosticCategory,
        PolicyReasonCode, ProofPolicyEvaluator,
    },
    selection::{
        ProofEvidenceCandidate, ProofEvidenceSet, VcProofSelection,
        merge_artifact_proof_selections, select_winner,
    },
};
use mizar_kernel::{
    certificate_parser::{
        ClauseTautologyPolicy, Fingerprint, KernelProfileRecord, RequiredProofStatus,
    },
    checker::{
        AcceptedProofStatus, FormulaEvidenceContext, FormulaImportedFactEvidence,
        ImportedFactContextLimits, ImportedFactNamespace, KernelCheckPolicy, KernelCheckStatus,
        KernelEvidenceCheckInput, KernelEvidenceCheckKind, KernelEvidenceCheckLimits,
        check_kernel_evidence,
    },
    clause::{Atom, SymbolId, SymbolKey, SymbolKind},
    formula_evidence::{
        Formula, FormulaEvidenceParseContext, FormulaSourceClass, GoalPolarity,
        IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID, ImportedStatementProjection,
        ParsedKernelEvidence, SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
        canonical_imported_statement_projection_payload, parse_formula_evidence,
    },
    rejection::{RejectionCategory, RejectionDetail, TargetVcFingerprint},
};

use super::*;

#[test]
fn obligation_identity_rejects_empty_fields() {
    assert_eq!(
        ObligationAnchor::new(""),
        Err(StatusProjectionError::EmptyObligationAnchor)
    );
    assert_eq!(
        ProofObligationIdentity::new(
            "",
            ObligationAnchor::new("anchor-0").expect("valid anchor"),
            hash(20),
            hash(21),
            hash(22),
            hash(23),
        ),
        Err(StatusProjectionError::EmptyObligationId)
    );
}

#[test]
fn projects_selection_classes_without_status_collapse() {
    let kernel = project(selection_for_kernel(true), VerifierPolicy::release());
    assert_eq!(kernel.projected_status(), ProjectedProofStatus::Accepted);
    assert_eq!(
        kernel.artifact_publication(),
        ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Accepted)
    );
    assert_eq!(
        kernel.accepted_witness_obligation_id(),
        Some("obligation-0")
    );
    assert_eq!(
        kernel.reuse_metadata().accepted_goal_polarity(),
        Some(AcceptedGoalPolarity::AssertFalseForRefutation)
    );
    assert!(kernel.reuse_metadata().trusted_used_axioms_hash().is_none());

    let builtin = project(selection_for_builtin(), VerifierPolicy::release());
    assert_eq!(builtin.projected_status(), ProjectedProofStatus::Accepted);
    assert_eq!(
        builtin.artifact_publication(),
        ArtifactStatusPublication::ExternalDependencyGap(
            ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported
        )
    );
    assert_eq!(
        builtin.reuse_metadata().deterministic_discharge_hash(),
        Some(hash(11))
    );
    assert_eq!(
        builtin.reuse_metadata().accepted_goal_polarity(),
        Some(AcceptedGoalPolarity::AssertFalseForRefutation)
    );
    assert_eq!(builtin.accepted_witness_obligation_id(), None);

    let external = project(selection_for_external(), external_policy());
    assert_eq!(
        external.projected_status(),
        ProjectedProofStatus::ExternallyAttested
    );
    assert_eq!(
        external.artifact_publication(),
        ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::ExternallyAttested)
    );
    assert_eq!(external.reuse_metadata().accepted_goal_polarity(), None);

    let assumed = project(selection_for_assumed(), assumed_policy());
    assert_eq!(
        assumed.projected_status(),
        ProjectedProofStatus::PolicyAssumed
    );
    assert_eq!(
        assumed.artifact_publication(),
        ArtifactStatusPublication::ExternalDependencyGap(
            ArtifactPublicationGap::PolicyAssumedStatusUnsupported
        )
    );
    assert_eq!(assumed.reuse_metadata().accepted_goal_polarity(), None);

    let open = project(selection_for_open(), VerifierPolicy::interactive());
    assert_eq!(open.projected_status(), ProjectedProofStatus::Open);
    assert_eq!(
        open.artifact_publication(),
        ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Open)
    );
    assert_eq!(open.reuse_metadata().accepted_goal_polarity(), None);

    let rejected = project(selection_for_rejected(), VerifierPolicy::release());
    assert_eq!(rejected.projected_status(), ProjectedProofStatus::Rejected);
    assert_eq!(
        rejected.artifact_publication(),
        ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
    );
    assert_eq!(rejected.reuse_metadata().accepted_goal_polarity(), None);
}

#[test]
fn no_selectable_evidence_projects_by_open_policy() {
    let release_policy = VerifierPolicy::release();
    let release = project(
        selection_for_no_selectable(release_policy.clone()),
        release_policy,
    );
    assert_eq!(release.projected_status(), ProjectedProofStatus::Rejected);
    assert_eq!(
        release.artifact_publication(),
        ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
    );

    let interactive_policy = VerifierPolicy::interactive();
    let interactive = project(
        selection_for_no_selectable(interactive_policy.clone()),
        interactive_policy,
    );
    assert_eq!(interactive.projected_status(), ProjectedProofStatus::Open);
    assert_eq!(
        interactive.artifact_publication(),
        ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Open)
    );

    let development_policy = VerifierPolicy::development();
    let development = project(
        selection_for_no_selectable(development_policy.clone()),
        development_policy,
    );
    assert_eq!(
        development.projected_status(),
        ProjectedProofStatus::Rejected
    );
    assert_eq!(
        development.artifact_publication(),
        ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
    );
}

#[test]
fn kernel_verified_artifact_publication_requires_witness_metadata() {
    let projection = project(selection_for_kernel(false), VerifierPolicy::release());

    assert_eq!(
        projection.projected_status(),
        ProjectedProofStatus::Accepted
    );
    assert_eq!(
        projection.artifact_publication(),
        ArtifactStatusPublication::ExternalDependencyGap(
            ArtifactPublicationGap::MissingKernelVerifiedWitness
        )
    );
    assert_eq!(projection.accepted_witness_obligation_id(), None);
}

#[test]
fn trusted_used_axioms_require_matching_trusted_selection() {
    let trusted = TrustedUsedAxiomsRef::for_test(hash(10), hash(90), 2);
    let projection = project_input(selection_for_kernel(true), VerifierPolicy::release())
        .with_trusted_used_axioms(trusted.clone());
    let projection = project_status(projection).expect("matching trusted axioms project");

    assert_eq!(projection.trusted_used_axioms(), Some(&trusted));
    assert_eq!(
        projection.reuse_metadata().trusted_used_axioms_hash(),
        Some(hash(90))
    );

    let external = project_input(selection_for_external(), external_policy())
        .with_trusted_used_axioms(trusted);
    assert_eq!(
        project_status(external),
        Err(
            StatusProjectionError::TrustedUsedAxiomsForNonTrustedStatus {
                selected_class: ProofWinnerClass::PolicyPermittedExternal,
            }
        )
    );
}

#[test]
fn non_trusted_statuses_keep_trusted_used_axioms_absent() {
    let no_selectable_policy = VerifierPolicy::release();
    let cases = [
        (
            selection_for_external(),
            external_policy(),
            ProofWinnerClass::PolicyPermittedExternal,
        ),
        (
            selection_for_assumed(),
            assumed_policy(),
            ProofWinnerClass::PolicyAssumed,
        ),
        (
            selection_for_open(),
            VerifierPolicy::interactive(),
            ProofWinnerClass::PolicyOpen,
        ),
        (
            selection_for_rejected(),
            VerifierPolicy::release(),
            ProofWinnerClass::Rejected,
        ),
        (
            selection_for_no_selectable(no_selectable_policy.clone()),
            no_selectable_policy,
            ProofWinnerClass::NoSelectableEvidence,
        ),
    ];

    for (selection, policy, selected_class) in cases {
        let normal = project_status(project_input(selection.clone(), policy.clone()))
            .expect("non-trusted projection succeeds without trusted axioms");
        assert_eq!(normal.selected_class(), selected_class);
        assert!(normal.trusted_used_axioms().is_none());
        assert!(normal.reuse_metadata().trusted_used_axioms_hash().is_none());

        let forced = project_input(selection, policy)
            .with_trusted_used_axioms(TrustedUsedAxiomsRef::for_test(hash(10), hash(90), 2));
        assert_eq!(
            project_status(forced),
            Err(StatusProjectionError::TrustedUsedAxiomsForNonTrustedStatus { selected_class })
        );
    }
}

#[test]
fn trusted_used_axioms_reject_mismatched_kernel_evidence_hash() {
    let trusted = TrustedUsedAxiomsRef::for_test(hash(12), hash(90), 2);
    let input = project_input(selection_for_kernel(true), VerifierPolicy::release())
        .with_trusted_used_axioms(trusted);

    assert_eq!(
        project_status(input),
        Err(StatusProjectionError::TrustedUsedAxiomsEvidenceMismatch {
            selected_evidence_hash: Some(hash(10)),
            accepted_evidence_hash: hash(12),
        })
    );
}

#[test]
fn trusted_used_axioms_from_kernel_result_accepts_public_kernel_result() {
    let result = accepted_kernel_result();
    let trusted = TrustedUsedAxiomsRef::from_kernel_result(
        &result,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
    )
    .expect("accepted proof-obligation kernel result yields trusted used axioms ref");
    let expected_input = KernelPolicyInput::from_kernel_result(
        &result,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
    );

    assert_eq!(trusted.used_axiom_count(), 1);
    assert_eq!(
        trusted.accepted_evidence_hash(),
        expected_input
            .accepted_evidence_hash()
            .expect("accepted evidence hash")
    );
    assert_eq!(
        trusted.used_axioms_hash(),
        hash_used_axioms(result.used_axioms())
    );
}

#[test]
fn trusted_used_axioms_from_kernel_result_rejects_untrusted_kernel_results() {
    let rejected = rejected_kernel_result();
    assert_eq!(
        TrustedUsedAxiomsRef::from_kernel_result(
            &rejected,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
        ),
        Err(TrustedUsedAxiomsError::KernelResultNotAccepted)
    );

    let policy_tainted = policy_tainted_kernel_result();
    assert_eq!(policy_tainted.status(), KernelCheckStatus::Accepted);
    assert!(policy_tainted.policy_taint());
    assert_eq!(
        TrustedUsedAxiomsRef::from_kernel_result(
            &policy_tainted,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
        ),
        Err(TrustedUsedAxiomsError::KernelResultPolicyTainted)
    );

    let consistency_check = accepted_consistency_kernel_result();
    assert_eq!(consistency_check.status(), KernelCheckStatus::Accepted);
    assert_eq!(
        consistency_check.evidence_check_kind(),
        Some(KernelEvidenceCheckKind::ConsistencyCheck)
    );
    let consistency_input = KernelPolicyInput::from_kernel_result(
        &consistency_check,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
    );
    assert_eq!(consistency_input.accepted_evidence_hash(), None);
    assert_eq!(
        TrustedUsedAxiomsRef::from_kernel_result(
            &consistency_check,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
        ),
        Err(TrustedUsedAxiomsError::MissingAcceptedEvidenceHash)
    );
}

#[test]
fn trusted_used_axioms_reject_missing_accepted_evidence_hash() {
    let input = KernelPolicyInput::for_test(
        KernelCheckStatus::Accepted,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        false,
        None,
    );

    assert_eq!(
        trusted_used_axioms_from_kernel_policy_input(&input, &[]),
        Err(TrustedUsedAxiomsError::MissingAcceptedEvidenceHash)
    );
}

#[test]
fn projection_rejects_active_policy_mismatch() {
    let selection = selection_for_external();
    let active_policy = VerifierPolicy::release();
    let input = project_input(selection, active_policy.clone());

    assert_eq!(
        project_status(input),
        Err(StatusProjectionError::PolicyFingerprintMismatch {
            selected_policy_fingerprint: external_policy().policy_fingerprint(),
            active_policy_fingerprint: active_policy.policy_fingerprint(),
        })
    );
}

#[test]
fn reuse_metadata_exports_architecture_22_identity() {
    let explanation = ExplanationRef::new(hash(70));
    let projection = project_status(
        project_input(selection_for_external(), external_policy())
            .with_dependency_compatibility(dependency_compatibility(40))
            .with_explanation_ref(explanation),
    )
    .expect("projection succeeds");
    let reuse = projection.reuse_metadata();
    let evidence_identity = reuse.proof_evidence_identity();

    assert_eq!(
        reuse.projected_status(),
        ProjectedProofStatus::ExternallyAttested
    );
    assert_eq!(reuse.obligation_anchor().as_str(), "anchor-0");
    assert_eq!(reuse.obligation_fingerprint(), hash(20));
    assert_eq!(reuse.vc_fingerprint(), hash(21));
    assert_eq!(reuse.local_context_fingerprint(), hash(22));
    assert_eq!(reuse.dependency_slice_fingerprint(), hash(23));
    assert_eq!(
        reuse.policy_fingerprint(),
        external_policy().policy_fingerprint()
    );
    assert_eq!(reuse.selected_evidence_hash(), Some(hash(30)));
    assert_eq!(reuse.accepted_goal_polarity(), None);
    assert_eq!(reuse.explanation_ref(), Some(explanation));
    assert_eq!(
        reuse.external_admission_status(),
        Some(ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted)
    );
    assert_eq!(
        reuse.dependency_compatibility(),
        Some(dependency_compatibility(40))
    );
    assert_eq!(
        evidence_identity
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("external")
    );
    assert_eq!(
        evidence_identity.selected_candidate_provenance_hash(),
        Some(hash(33))
    );
    assert_eq!(evidence_identity.accepted_goal_polarity(), None);
    assert_eq!(
        evidence_identity.selection_reason(),
        "deterministic_winner_class"
    );
    assert_eq!(
        evidence_identity.tie_break_key_hash(),
        reuse.tie_break_key_hash()
    );
    assert_eq!(
        reuse.proof_reuse_validation_hash(),
        hash_status_reuse_metadata(reuse)
    );
    assert!(!reuse.cache_reuse_predicate_complete());
}

#[test]
fn proof_reuse_validation_hash_changes_or_invalidates_for_exported_components() {
    let projection = project_status(
        project_input(selection_for_kernel(true), VerifierPolicy::release())
            .with_dependency_compatibility(dependency_compatibility(40))
            .with_explanation_ref(ExplanationRef::new(hash(70))),
    )
    .expect("projection succeeds");
    let base = projection.reuse_metadata().clone();
    assert!(base.cache_reuse_predicate_complete());

    assert_hash_changed(&base, "selected_class", |metadata| {
        metadata.selected_class = ProofWinnerClass::PolicyAssumed;
    });
    assert_hash_changed(&base, "projected_status", |metadata| {
        metadata.projected_status = ProjectedProofStatus::Open;
    });
    assert_hash_changed(&base, "selected_candidate_id", |metadata| {
        metadata.selected_candidate_id = Some(CandidateSourceId::new("other").expect("stable id"));
    });
    assert_hash_changed(&base, "obligation_anchor", |metadata| {
        metadata.obligation_anchor = ObligationAnchor::new("anchor-1").expect("stable anchor");
    });
    assert_hash_changed(&base, "obligation_fingerprint", |metadata| {
        metadata.obligation_fingerprint = hash(41);
    });
    assert_hash_changed(&base, "vc_fingerprint", |metadata| {
        metadata.vc_fingerprint = hash(42);
    });
    assert_hash_changed(&base, "local_context_fingerprint", |metadata| {
        metadata.local_context_fingerprint = hash(43);
    });
    assert_hash_changed(&base, "dependency_slice_fingerprint", |metadata| {
        metadata.dependency_slice_fingerprint = hash(44);
    });
    assert_hash_changed(&base, "policy_fingerprint", |metadata| {
        metadata.policy_fingerprint = external_policy().policy_fingerprint();
    });
    assert_hash_changed(&base, "selected_evidence_hash", |metadata| {
        metadata.selected_evidence_hash = Some(hash(45));
    });
    assert_hash_changed(&base, "selected_proof_witness_hash", |metadata| {
        metadata.selected_proof_witness_hash = Some(hash(46));
    });
    assert_hash_changed(&base, "deterministic_discharge_hash", |metadata| {
        metadata.deterministic_discharge_hash = Some(hash(47));
    });
    assert_hash_changed(&base, "accepted_goal_polarity", |metadata| {
        metadata.accepted_goal_polarity = None;
    });
    assert_hash_changed(&base, "trusted_used_axioms_hash", |metadata| {
        metadata.trusted_used_axioms_hash = Some(hash(48));
    });
    assert_hash_changed(&base, "external_admission_status", |metadata| {
        metadata.external_admission_status =
            Some(ExternalEvidencePublicationStatus::ExternallyAttestedDevelopment);
    });
    assert_hash_changed(&base, "selected_candidate_provenance_hash", |metadata| {
        metadata.selected_candidate_provenance_hash = Some(hash(49));
    });
    assert_hash_changed(&base, "selection_reason", |metadata| {
        metadata.selection_reason = "primary_rejected_diagnostic";
    });
    assert_hash_changed(&base, "tie_break_key_hash", |metadata| {
        metadata.tie_break_key_hash = hash(50);
    });
    assert_hash_changed(&base, "dependency_artifact_fingerprint", |metadata| {
        metadata.dependency_compatibility = Some(ProofReuseDependencyCompatibility::new(
            hash(51),
            SchemaVersion::new(1, 40),
            SchemaVersion::new(2, 40),
        ));
    });
    assert_hash_changed(&base, "dependency_schema_version", |metadata| {
        metadata.dependency_compatibility = Some(ProofReuseDependencyCompatibility::new(
            hash(40),
            SchemaVersion::new(1, 41),
            SchemaVersion::new(2, 40),
        ));
    });
    assert_hash_changed(&base, "proof_reuse_schema_version", |metadata| {
        metadata.dependency_compatibility = Some(ProofReuseDependencyCompatibility::new(
            hash(40),
            SchemaVersion::new(1, 40),
            SchemaVersion::new(2, 41),
        ));
    });
    assert_hash_changed(&base, "explanation_ref", |metadata| {
        metadata.explanation_ref = Some(ExplanationRef::new(hash(52)));
    });
    assert_hash_changed(&base, "diagnostic_result_id", |metadata| {
        metadata.diagnostic_result_id = Some(hash(53));
    });

    let mut missing_dependency_compatibility = base.clone();
    missing_dependency_compatibility.dependency_compatibility = None;
    assert!(!missing_dependency_compatibility.cache_reuse_predicate_complete());
    assert_ne!(
        hash_status_reuse_metadata(&missing_dependency_compatibility),
        base.proof_reuse_validation_hash()
    );
}

#[test]
fn externally_attested_reuse_metadata_never_upgrades_trust() {
    let projection = project_status(
        project_input(selection_for_external(), external_policy())
            .with_dependency_compatibility(dependency_compatibility(40)),
    )
    .expect("projection succeeds");
    let reuse = projection.reuse_metadata();

    assert_eq!(
        projection.projected_status(),
        ProjectedProofStatus::ExternallyAttested
    );
    assert_eq!(
        reuse.selected_class(),
        ProofWinnerClass::PolicyPermittedExternal
    );
    assert!(!projection.projected_status().is_trusted());
    assert!(!reuse.projected_status().is_trusted());
    assert!(!reuse.cache_reuse_predicate_complete());
    assert_eq!(projection.trusted_used_axioms(), None);
    assert_eq!(reuse.trusted_used_axioms_hash(), None);
    assert_eq!(
        reuse.external_admission_status(),
        Some(ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted)
    );
    assert_eq!(
        reuse.proof_evidence_identity().selected_evidence_hash(),
        Some(hash(30))
    );
    assert_eq!(projection.accepted_witness_obligation_id(), None);
}

#[test]
fn cache_reuse_predicate_completeness_is_class_aware() {
    let kernel_with_witness = project_status(
        project_input(selection_for_kernel(true), VerifierPolicy::release())
            .with_dependency_compatibility(dependency_compatibility(40)),
    )
    .expect("projection succeeds");
    assert!(
        kernel_with_witness
            .reuse_metadata()
            .cache_reuse_predicate_complete()
    );

    let kernel_without_witness = project_status(
        project_input(selection_for_kernel(false), VerifierPolicy::release())
            .with_dependency_compatibility(dependency_compatibility(40)),
    )
    .expect("projection succeeds");
    assert!(
        !kernel_without_witness
            .reuse_metadata()
            .cache_reuse_predicate_complete()
    );

    let builtin = project_status(
        project_input(selection_for_builtin(), VerifierPolicy::release())
            .with_dependency_compatibility(dependency_compatibility(40)),
    )
    .expect("projection succeeds");
    assert!(builtin.reuse_metadata().cache_reuse_predicate_complete());

    let external = project_status(
        project_input(selection_for_external(), external_policy())
            .with_dependency_compatibility(dependency_compatibility(40)),
    )
    .expect("projection succeeds");
    assert!(!external.reuse_metadata().cache_reuse_predicate_complete());
}

#[test]
fn reuse_metadata_covers_trusted_rejected_and_no_selectable_outcomes() {
    let kernel = project(selection_for_kernel(true), VerifierPolicy::release());
    let kernel_reuse = kernel.reuse_metadata();
    assert_eq!(
        kernel_reuse.selected_class(),
        ProofWinnerClass::KernelVerified
    );
    assert_eq!(
        kernel_reuse.projected_status(),
        ProjectedProofStatus::Accepted
    );
    assert_eq!(kernel_reuse.selected_evidence_hash(), Some(hash(10)));
    assert_eq!(kernel_reuse.selected_proof_witness_hash(), Some(hash(50)));
    assert_eq!(kernel_reuse.deterministic_discharge_hash(), None);
    assert_eq!(
        kernel_reuse.accepted_goal_polarity(),
        Some(AcceptedGoalPolarity::AssertFalseForRefutation)
    );
    assert_eq!(
        kernel_reuse
            .proof_evidence_identity()
            .accepted_goal_polarity(),
        Some(AcceptedGoalPolarity::AssertFalseForRefutation)
    );
    assert_eq!(kernel_reuse.diagnostic_result_id(), None);

    let builtin = project(selection_for_builtin(), VerifierPolicy::release());
    let builtin_reuse = builtin.reuse_metadata();
    assert_eq!(
        builtin_reuse.selected_class(),
        ProofWinnerClass::DischargedBuiltin
    );
    assert_eq!(
        builtin_reuse.projected_status(),
        ProjectedProofStatus::Accepted
    );
    assert_eq!(builtin_reuse.selected_evidence_hash(), Some(hash(10)));
    assert_eq!(builtin_reuse.selected_proof_witness_hash(), None);
    assert_eq!(builtin_reuse.deterministic_discharge_hash(), Some(hash(11)));
    assert_eq!(
        builtin_reuse.accepted_goal_polarity(),
        Some(AcceptedGoalPolarity::AssertFalseForRefutation)
    );
    assert_eq!(builtin_reuse.diagnostic_result_id(), None);

    let rejected = project(selection_for_rejected(), VerifierPolicy::release());
    let rejected_reuse = rejected.reuse_metadata();
    assert_eq!(rejected_reuse.selected_class(), ProofWinnerClass::Rejected);
    assert_eq!(
        rejected_reuse.projected_status(),
        ProjectedProofStatus::Rejected
    );
    assert_eq!(rejected_reuse.selected_evidence_hash(), None);
    assert_eq!(rejected_reuse.accepted_goal_polarity(), None);
    assert!(rejected_reuse.selected_candidate_id().is_some());
    assert_eq!(rejected_reuse.diagnostic_result_id(), None);

    let release_policy = VerifierPolicy::release();
    let no_selectable = project(
        selection_for_no_selectable(release_policy.clone()),
        release_policy,
    );
    let no_selectable_reuse = no_selectable.reuse_metadata();
    assert_eq!(
        no_selectable_reuse.selected_class(),
        ProofWinnerClass::NoSelectableEvidence
    );
    assert_eq!(
        no_selectable_reuse.projected_status(),
        ProjectedProofStatus::Rejected
    );
    assert_eq!(no_selectable_reuse.selected_evidence_hash(), None);
    assert_eq!(no_selectable_reuse.accepted_goal_polarity(), None);
    assert!(no_selectable_reuse.selected_candidate_id().is_none());
    assert!(no_selectable_reuse.diagnostic_result_id().is_some());
}

#[test]
fn corrected_terminal_kernel_rejections_do_not_fall_back_to_policy_open() {
    for case in terminal_kernel_rejection_cases() {
        let policy = VerifierPolicy::interactive();
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        let decision =
            evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(case.input.clone()));

        assert_eq!(decision.class, CandidatePolicyClass::KernelRejected);
        assert_eq!(decision.kernel_rejections.len(), 1, "{}", case.label);
        assert_eq!(decision.kernel_rejections[0].category(), case.category);
        assert_eq!(decision.kernel_rejections[0].detail(), case.detail);

        let rejected = candidate(case.label, decision).with_evidence_hash(hash(case.hash_tag));
        let open = candidate(
            "open",
            evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
        )
        .with_evidence_hash(hash(240));
        let selection = artifact_selection(
            [VcProofSelection::new(
                VcId::new(0),
                select_winner(&base_set(policy.clone()).with_candidates([open, rejected])),
            )],
            [],
        );

        assert_eq!(selection.selected_class(), ProofWinnerClass::Rejected);
        assert_eq!(
            selection
                .selection()
                .selected_candidate_id()
                .map(CandidateSourceId::as_str),
            Some(case.label)
        );

        let projection = project(selection, policy);
        assert_eq!(
            projection.projected_status(),
            ProjectedProofStatus::Rejected
        );
        assert_eq!(
            projection.artifact_publication(),
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
        );
        assert_eq!(projection.reuse_metadata().accepted_goal_polarity(), None);
    }
}

#[test]
fn terminal_kernel_rejections_only_displace_policy_open() {
    let case = terminal_kernel_rejection_cases()
        .into_iter()
        .next()
        .expect("terminal case");

    let interactive = VerifierPolicy::interactive();
    let evaluator = ProofPolicyEvaluator::new(interactive.clone());
    let terminal = candidate(
        case.label,
        evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(case.input)),
    )
    .with_evidence_hash(hash(case.hash_tag));
    let trusted = trusted_candidate(
        "trusted",
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(10),
    );

    let trusted_selection = artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(
                &base_set(interactive.clone()).with_candidates([terminal.clone(), trusted]),
            ),
        )],
        [],
    );
    assert_eq!(
        trusted_selection.selected_class(),
        ProofWinnerClass::KernelVerified
    );

    let external_policy = external_policy();
    let external_evaluator = ProofPolicyEvaluator::new(external_policy.clone());
    let external_selection = artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(
                &base_set(external_policy).with_candidates([
                    terminal.clone(),
                    candidate(
                        "external",
                        external_evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
                    )
                    .with_evidence_hash(hash(30)),
                ]),
            ),
        )],
        [],
    );
    assert_eq!(
        external_selection.selected_class(),
        ProofWinnerClass::PolicyPermittedExternal
    );

    let assumed_policy = assumed_policy();
    let assumed_evaluator = ProofPolicyEvaluator::new(assumed_policy.clone());
    let assumed_selection = artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(
                &base_set(assumed_policy).with_candidates([
                    terminal,
                    candidate(
                        "assumed",
                        assumed_evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
                    )
                    .with_evidence_hash(hash(31)),
                ]),
            ),
        )],
        [],
    );
    assert_eq!(
        assumed_selection.selected_class(),
        ProofWinnerClass::PolicyAssumed
    );
}

#[test]
fn target_context_mismatch_does_not_displace_policy_open() {
    let policy = VerifierPolicy::interactive();
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let input = KernelPolicyInput::from_kernel_result(
        &target_context_mismatch_kernel_result(),
        KernelEvidenceOrigin::AtpFormulaSubstitution,
    );
    let decision = evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(input));

    assert_eq!(decision.class, CandidatePolicyClass::KernelRejected);
    assert_eq!(decision.kernel_rejections.len(), 1);
    assert_eq!(
        decision.kernel_rejections[0].detail(),
        RejectionDetail::ContextMismatch
    );
    assert_eq!(
        decision.kernel_rejections[0].location().field_path,
        Some("target_vc")
    );

    let selection = artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(
                &base_set(policy).with_candidates([
                    candidate("target-context-mismatch", decision).with_evidence_hash(hash(234)),
                    candidate(
                        "open",
                        evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
                    )
                    .with_evidence_hash(hash(240)),
                ]),
            ),
        )],
        [],
    );

    assert_eq!(selection.selected_class(), ProofWinnerClass::PolicyOpen);
    assert_eq!(
        selection
            .selection()
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("open")
    );
}

#[test]
fn consistency_goal_polarity_mismatch_does_not_displace_policy_open() {
    let policy = VerifierPolicy::interactive();
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    let input = KernelPolicyInput::from_kernel_result(
        &consistency_goal_polarity_mismatch_kernel_result(),
        KernelEvidenceOrigin::AtpFormulaSubstitution,
    );
    assert_eq!(
        input.evidence_check_kind(),
        Some(KernelEvidenceCheckKind::ConsistencyCheck)
    );
    let decision = evaluator.evaluate_candidate(&PolicyCandidate::KernelResult(input));

    assert_eq!(decision.class, CandidatePolicyClass::KernelRejected);
    assert_eq!(
        decision.kernel_evidence_check_kind,
        Some(KernelEvidenceCheckKind::ConsistencyCheck)
    );
    assert_eq!(decision.kernel_rejections.len(), 1);
    assert_eq!(
        decision.kernel_rejections[0].detail(),
        RejectionDetail::ContextMismatch
    );
    assert!(decision.kernel_rejections[0].location().final_goal);
    assert_eq!(
        decision.kernel_rejections[0].location().field_path,
        Some("final_goal.polarity")
    );

    let selection = artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(
                &base_set(policy).with_candidates([
                    candidate("consistency-polarity-mismatch", decision)
                        .with_evidence_hash(hash(235)),
                    candidate(
                        "open",
                        evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
                    )
                    .with_evidence_hash(hash(240)),
                ]),
            ),
        )],
        [],
    );

    assert_eq!(selection.selected_class(), ProofWinnerClass::PolicyOpen);
    assert_eq!(
        selection
            .selection()
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("open")
    );
}

fn project(selection: ArtifactProofSelection, policy: VerifierPolicy) -> ProofStatusProjection {
    project_status(project_input(selection, policy)).expect("projection succeeds")
}

fn project_input(
    selection: ArtifactProofSelection,
    policy: VerifierPolicy,
) -> ProofStatusProjectionInput {
    ProofStatusProjectionInput::new(selection, policy, identity())
}

fn dependency_compatibility(tag: u8) -> ProofReuseDependencyCompatibility {
    ProofReuseDependencyCompatibility::new(
        hash(tag),
        SchemaVersion::new(1, u16::from(tag)),
        SchemaVersion::new(2, u16::from(tag)),
    )
}

fn assert_hash_changed(
    base: &StatusReuseMetadata,
    label: &str,
    mutate: impl FnOnce(&mut StatusReuseMetadata),
) {
    let mut changed = base.clone();
    mutate(&mut changed);
    assert_ne!(
        hash_status_reuse_metadata(&changed),
        base.proof_reuse_validation_hash(),
        "{label} should participate in proof reuse validation hash"
    );
}

fn identity() -> ProofObligationIdentity {
    ProofObligationIdentity::new(
        "obligation-0",
        ObligationAnchor::new("anchor-0").expect("valid anchor"),
        hash(20),
        hash(21),
        hash(22),
        hash(23),
    )
    .expect("valid obligation identity")
}

fn selection_for_kernel(with_witness: bool) -> ArtifactProofSelection {
    let mut candidate = trusted_candidate(
        "kernel",
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(10),
    )
    .with_provenance_hash(hash(34));
    if with_witness {
        candidate = candidate.with_selected_proof_witness_hash(hash(50));
    }
    artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(&base_set(VerifierPolicy::release()).with_candidates([candidate])),
        )],
        [],
    )
}

fn selection_for_builtin() -> ArtifactProofSelection {
    artifact_selection(
        [],
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(
                &base_set(VerifierPolicy::release()).with_candidates([trusted_candidate(
                    "builtin",
                    KernelEvidenceOrigin::BuiltinDischarge,
                    hash(10),
                )
                .with_deterministic_discharge_hash(hash(11))]),
            ),
        )],
    )
}

fn selection_for_external() -> ArtifactProofSelection {
    let policy = external_policy();
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(
                &base_set(policy).with_candidates([candidate(
                    "external",
                    evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
                )
                .with_evidence_hash(hash(30))
                .with_provenance_hash(hash(33))]),
            ),
        )],
        [],
    )
}

fn selection_for_assumed() -> ArtifactProofSelection {
    let policy = assumed_policy();
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(
                &base_set(policy).with_candidates([candidate(
                    "assumed",
                    evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
                )
                .with_evidence_hash(hash(31))]),
            ),
        )],
        [],
    )
}

fn selection_for_open() -> ArtifactProofSelection {
    let policy = VerifierPolicy::interactive();
    let evaluator = ProofPolicyEvaluator::new(policy.clone());
    artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(
                &base_set(policy).with_candidates([candidate(
                    "open",
                    evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
                )
                .with_evidence_hash(hash(32))]),
            ),
        )],
        [],
    )
}

fn selection_for_rejected() -> ArtifactProofSelection {
    artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(
                &base_set(VerifierPolicy::release())
                    .with_candidates([candidate("rejected", rejected_policy_decision())]),
            ),
        )],
        [],
    )
}

fn selection_for_no_selectable(policy: VerifierPolicy) -> ArtifactProofSelection {
    artifact_selection(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(&base_set(policy)),
        )],
        [],
    )
}

fn artifact_selection(
    portfolio: impl IntoIterator<Item = VcProofSelection>,
    builtin_discharge: impl IntoIterator<Item = VcProofSelection>,
) -> ArtifactProofSelection {
    let mut merged = merge_artifact_proof_selections(portfolio, builtin_discharge)
        .expect("valid artifact selection");
    assert_eq!(merged.len(), 1);
    merged.remove(0)
}

fn base_set(policy: VerifierPolicy) -> ProofEvidenceSet {
    ProofEvidenceSet::new(b"obligation".to_vec(), hash(100), policy)
}

fn candidate(id: &str, decision: PolicyDecision) -> ProofEvidenceCandidate {
    ProofEvidenceCandidate::new(CandidateSourceId::new(id).expect("stable id"), decision)
}

fn trusted_candidate(
    id: &str,
    origin: KernelEvidenceOrigin,
    evidence_hash: Hash,
) -> ProofEvidenceCandidate {
    let kernel_input = KernelPolicyInput::for_test(
        KernelCheckStatus::Accepted,
        origin,
        false,
        Some(evidence_hash),
    );
    ProofEvidenceCandidate::from_trusted_kernel_input(
        CandidateSourceId::new(id).expect("stable id"),
        &kernel_input,
    )
    .expect("accepted proof-obligation kernel input is trusted evidence")
}

fn rejected_policy_decision() -> PolicyDecision {
    PolicyDecision {
        class: CandidatePolicyClass::RejectedByPolicy,
        can_schedule_kernel_check: false,
        diagnostic: Some(PolicyDiagnostic::new(
            PolicyDiagnosticCategory::PolicyRejection,
            PolicyReasonCode::OpenObligationRejected,
        )),
        kernel_rejections: Vec::new(),
        kernel_evidence_check_kind: None,
        external_admission: None,
    }
}

fn external_policy() -> VerifierPolicy {
    VerifierPolicy::interactive()
        .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
}

fn assumed_policy() -> VerifierPolicy {
    VerifierPolicy::development()
}

fn accepted_kernel_result() -> KernelCheckResult {
    let target = formula_target(7);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let premise = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![imported_formula_item(
            1,
            10,
            &premise,
            RequiredProofStatus::KernelVerified,
        )],
        goal_item(20, &premise),
    );
    let context = formula_evidence_context(
        formula_imported_fact(5, &premise, AcceptedProofStatus::KernelVerified),
        ImportedFactNamespace::ImportedAxiom,
    );

    let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, Some(&context)));
    assert_eq!(result.status(), KernelCheckStatus::Accepted);
    assert!(!result.policy_taint());
    result
}

fn accepted_consistency_kernel_result() -> KernelCheckResult {
    let target = formula_target(10);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let not_goal = Formula::Not(Box::new(goal.clone()));
    let parsed = parsed_formula_evidence(
        &target,
        vec![imported_formula_item(
            1,
            10,
            &not_goal,
            RequiredProofStatus::KernelVerified,
        )],
        goal_item_with_polarity(20, GoalPolarity::AssertTrueForConsistency, &goal),
    );
    let context = formula_evidence_context(
        formula_imported_fact(5, &not_goal, AcceptedProofStatus::KernelVerified),
        ImportedFactNamespace::ImportedAxiom,
    );

    let result = check_kernel_evidence(evidence_input_with_check_kind(
        &target_vc,
        &parsed,
        Some(&context),
        KernelEvidenceCheckKind::ConsistencyCheck,
    ));
    assert_eq!(result.status(), KernelCheckStatus::Accepted);
    assert!(!result.policy_taint());
    result
}

struct TerminalKernelRejectionCase {
    label: &'static str,
    input: KernelPolicyInput,
    category: RejectionCategory,
    detail: RejectionDetail,
    hash_tag: u8,
}

fn terminal_kernel_rejection_cases() -> Vec<TerminalKernelRejectionCase> {
    vec![
        terminal_case(
            "invalid-sat-refutation",
            invalid_sat_refutation_kernel_result(),
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSatRefutation,
            230,
        ),
        terminal_case(
            "context-mismatch",
            goal_polarity_mismatch_kernel_result(),
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            RejectionCategory::CertificateRejection,
            RejectionDetail::ContextMismatch,
            231,
        ),
        terminal_case(
            "missing-provenance",
            rejected_kernel_result(),
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            RejectionCategory::KernelRejection,
            RejectionDetail::MissingProvenance,
            232,
        ),
    ]
}

fn terminal_case(
    label: &'static str,
    result: KernelCheckResult,
    origin: KernelEvidenceOrigin,
    category: RejectionCategory,
    detail: RejectionDetail,
    hash_tag: u8,
) -> TerminalKernelRejectionCase {
    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert_eq!(result.rejections().len(), 1, "{label}");
    assert_eq!(result.rejections()[0].category(), category, "{label}");
    assert_eq!(result.rejections()[0].detail(), detail, "{label}");
    TerminalKernelRejectionCase {
        label,
        input: KernelPolicyInput::from_kernel_result(&result, origin),
        category,
        detail,
        hash_tag,
    }
}

fn invalid_sat_refutation_kernel_result() -> KernelCheckResult {
    let target = formula_target(11);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let parsed = parsed_formula_evidence(&target, Vec::new(), goal_item(20, &goal));

    check_kernel_evidence(evidence_input(&target_vc, &parsed, None))
}

fn goal_polarity_mismatch_kernel_result() -> KernelCheckResult {
    let target = formula_target(12);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        Vec::new(),
        goal_item_with_polarity(20, GoalPolarity::AssertTrueForConsistency, &goal),
    );

    check_kernel_evidence(evidence_input(&target_vc, &parsed, None))
}

fn target_context_mismatch_kernel_result() -> KernelCheckResult {
    let target = formula_target(13);
    let target_vc = TargetVcFingerprint::new(1, vec![99]);
    let goal = formula_atom(1);
    let parsed = parsed_formula_evidence(&target, Vec::new(), goal_item(20, &goal));

    check_kernel_evidence(evidence_input(&target_vc, &parsed, None))
}

fn consistency_goal_polarity_mismatch_kernel_result() -> KernelCheckResult {
    let target = formula_target(14);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let parsed = parsed_formula_evidence(&target, Vec::new(), goal_item(20, &goal));

    check_kernel_evidence(evidence_input_with_check_kind(
        &target_vc,
        &parsed,
        None,
        KernelEvidenceCheckKind::ConsistencyCheck,
    ))
}

fn rejected_kernel_result() -> KernelCheckResult {
    let target = formula_target(8);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let parsed = parsed_formula_evidence(
        &target,
        vec![local_formula_item(1, 10, &formula_atom(1))],
        goal_item(20, &formula_atom(2)),
    );

    let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, None));
    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    result
}

fn policy_tainted_kernel_result() -> KernelCheckResult {
    let target = formula_target(9);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let premise = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![imported_formula_item(
            1,
            10,
            &premise,
            RequiredProofStatus::ExternallyAttestedPolicyPermitted,
        )],
        goal_item(20, &premise),
    );
    let context = formula_evidence_context(
        formula_imported_fact(
            5,
            &premise,
            AcceptedProofStatus::ExternallyAttestedPolicyPermitted,
        ),
        ImportedFactNamespace::ImportedAxiom,
    );
    let mut input = evidence_input(&target_vc, &parsed, Some(&context));
    input.policy = KernelCheckPolicy {
        imported_fact_policy: mizar_kernel::checker::ImportedFactPolicy {
            allow_externally_attested: true,
        },
        ..KernelCheckPolicy::default()
    };

    let result = check_kernel_evidence(input);
    assert_eq!(result.status(), KernelCheckStatus::Accepted);
    result
}

fn formula_target(tag: u8) -> Fingerprint {
    Fingerprint::new(9, vec![tag])
}

fn formula_profile() -> KernelProfileRecord {
    KernelProfileRecord::v1(7, ClauseTautologyPolicy::Reject)
}

fn formula_atom(symbol_id: u32) -> Formula {
    Formula::Atom(Atom::with_arity(
        SymbolKey {
            kind: SymbolKind::Predicate,
            id: SymbolId(symbol_id),
        },
        0,
        Vec::new(),
    ))
}

fn formula_fingerprint(formula: &Formula) -> Fingerprint {
    Fingerprint::new(
        SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
        formula
            .canonical_hash_input()
            .expect("test formula has canonical bytes"),
    )
}

struct FormulaFixture {
    bytes: Vec<u8>,
    provenance_id: u32,
    fingerprint: Fingerprint,
}

fn local_formula_item(formula_id: u32, provenance_id: u32, formula: &Formula) -> FormulaFixture {
    let fingerprint = formula_fingerprint(formula);
    let mut bytes = Vec::new();
    put_u32(formula_id, &mut bytes);
    bytes.push(FormulaSourceClass::LocalHypothesis.tag());
    put_fingerprint(&fingerprint, &mut bytes);
    put_u32(provenance_id, &mut bytes);
    put_u32(1, &mut bytes);
    put_formula(formula, &mut bytes);
    FormulaFixture {
        bytes,
        provenance_id,
        fingerprint,
    }
}

fn imported_formula_item(
    formula_id: u32,
    provenance_id: u32,
    formula: &Formula,
    required_status: RequiredProofStatus,
) -> FormulaFixture {
    let fingerprint = formula_fingerprint(formula);
    let statement_fingerprint = imported_statement_fingerprint();
    let statement_projection = imported_statement_projection(&statement_fingerprint, &fingerprint);
    let mut bytes = Vec::new();
    put_u32(formula_id, &mut bytes);
    bytes.push(FormulaSourceClass::AcceptedImportedAxiom.tag());
    put_fingerprint(&fingerprint, &mut bytes);
    put_u32(provenance_id, &mut bytes);
    put_bytes(b"pkg", &mut bytes);
    put_bytes(b"module", &mut bytes);
    put_bytes(b"ITEM", &mut bytes);
    put_fingerprint(&statement_fingerprint, &mut bytes);
    bytes.push(required_status_tag(required_status));
    put_fingerprint(&statement_projection.statement_fingerprint, &mut bytes);
    put_fingerprint(&statement_projection.formula_fingerprint, &mut bytes);
    put_bytes(&statement_projection.payload, &mut bytes);
    put_formula(formula, &mut bytes);
    FormulaFixture {
        bytes,
        provenance_id,
        fingerprint,
    }
}

fn goal_item(provenance_id: u32, formula: &Formula) -> FormulaFixture {
    goal_item_with_polarity(
        provenance_id,
        GoalPolarity::AssertFalseForRefutation,
        formula,
    )
}

fn goal_item_with_polarity(
    provenance_id: u32,
    polarity: GoalPolarity,
    formula: &Formula,
) -> FormulaFixture {
    let fingerprint = formula_fingerprint(formula);
    let mut bytes = Vec::new();
    bytes.push(polarity.tag());
    put_fingerprint(&fingerprint, &mut bytes);
    put_u32(provenance_id, &mut bytes);
    put_formula(formula, &mut bytes);
    FormulaFixture {
        bytes,
        provenance_id,
        fingerprint,
    }
}

fn parsed_formula_evidence(
    target: &Fingerprint,
    formulas: Vec<FormulaFixture>,
    goal: FormulaFixture,
) -> ParsedKernelEvidence {
    let bytes = formula_evidence_bytes(target, formulas, goal);
    parse_formula_evidence(
        &bytes,
        &FormulaEvidenceParseContext::v1(target.clone(), formula_profile()),
    )
    .expect("formula evidence parses")
}

fn evidence_input<'a>(
    target: &'a TargetVcFingerprint,
    evidence: &'a ParsedKernelEvidence,
    formula_context: Option<&'a FormulaEvidenceContext>,
) -> KernelEvidenceCheckInput<'a> {
    evidence_input_with_check_kind(
        target,
        evidence,
        formula_context,
        KernelEvidenceCheckKind::ProofObligation,
    )
}

fn evidence_input_with_check_kind<'a>(
    target: &'a TargetVcFingerprint,
    evidence: &'a ParsedKernelEvidence,
    formula_context: Option<&'a FormulaEvidenceContext>,
    check_kind: KernelEvidenceCheckKind,
) -> KernelEvidenceCheckInput<'a> {
    KernelEvidenceCheckInput {
        target_vc_fingerprint: target,
        evidence,
        formula_context,
        check_kind,
        policy: KernelCheckPolicy::default(),
        limits: KernelEvidenceCheckLimits::default(),
    }
}

fn formula_evidence_context(
    imported: FormulaImportedFactEvidence,
    namespace: ImportedFactNamespace,
) -> FormulaEvidenceContext {
    let (axioms, theorems) = match namespace {
        ImportedFactNamespace::ImportedAxiom => (vec![imported], Vec::new()),
        ImportedFactNamespace::ImportedTheorem => (Vec::new(), vec![imported]),
        _ => panic!("unknown imported fact namespace in status projection test fixture"),
    };
    FormulaEvidenceContext::new(
        Some(vec![1]),
        axioms,
        theorems,
        ImportedFactContextLimits::default(),
    )
    .expect("formula evidence context")
}

fn formula_imported_fact(
    imported_fact_id: u32,
    formula: &Formula,
    accepted_proof_status: AcceptedProofStatus,
) -> FormulaImportedFactEvidence {
    let formula_fingerprint = formula_fingerprint(formula);
    let statement_fingerprint = imported_statement_fingerprint();
    let statement_projection =
        imported_statement_projection(&statement_fingerprint, &formula_fingerprint);
    FormulaImportedFactEvidence {
        imported_fact_id,
        package_id: b"pkg".to_vec(),
        module_path: b"module".to_vec(),
        exported_item_id: b"ITEM".to_vec(),
        statement_fingerprint,
        accepted_proof_status,
        statement_projection,
    }
}

fn imported_statement_projection(
    statement_fingerprint: &Fingerprint,
    formula_fingerprint: &Fingerprint,
) -> ImportedStatementProjection {
    ImportedStatementProjection {
        statement_fingerprint: statement_fingerprint.clone(),
        formula_fingerprint: formula_fingerprint.clone(),
        payload: canonical_imported_statement_projection_payload(
            statement_fingerprint,
            formula_fingerprint,
        )
        .expect("canonical imported statement projection payload"),
    }
}

fn imported_statement_fingerprint() -> Fingerprint {
    Fingerprint::new(
        IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID,
        b"imported-statement".to_vec(),
    )
}

fn formula_evidence_bytes(
    target: &Fingerprint,
    formulas: Vec<FormulaFixture>,
    goal: FormulaFixture,
) -> Vec<u8> {
    let mut provenance = Vec::new();
    for formula in &formulas {
        provenance.push(provenance_item(
            target,
            formula.provenance_id,
            &formula.fingerprint,
        ));
    }
    provenance.push(provenance_item(
        target,
        goal.provenance_id,
        &goal.fingerprint,
    ));
    formula_envelope(
        target,
        vec![
            (
                FormulaEvidenceSectionTag::SymbolManifest,
                formula_symbol_items(),
            ),
            (FormulaEvidenceSectionTag::VariableManifest, Vec::new()),
            (
                FormulaEvidenceSectionTag::Formulas,
                formulas.into_iter().map(|formula| formula.bytes).collect(),
            ),
            (FormulaEvidenceSectionTag::Substitutions, Vec::new()),
            (FormulaEvidenceSectionTag::Provenance, provenance),
            (FormulaEvidenceSectionTag::FinalGoal, vec![goal.bytes]),
        ],
    )
}

#[derive(Clone, Copy)]
enum FormulaEvidenceSectionTag {
    SymbolManifest,
    VariableManifest,
    Formulas,
    Substitutions,
    Provenance,
    FinalGoal,
}

impl FormulaEvidenceSectionTag {
    const fn byte(self) -> u8 {
        match self {
            Self::SymbolManifest => 1,
            Self::VariableManifest => 2,
            Self::Formulas => 3,
            Self::Substitutions => 4,
            Self::Provenance => 5,
            Self::FinalGoal => 6,
        }
    }
}

fn formula_envelope(
    target: &Fingerprint,
    sections: Vec<(FormulaEvidenceSectionTag, Vec<Vec<u8>>)>,
) -> Vec<u8> {
    let mut payloads = Vec::new();
    let mut directory = Vec::new();
    let mut offset = 0_u32;
    for (section, items) in &sections {
        let mut section_payload = Vec::new();
        for item in items {
            section_payload.push(section.byte());
            section_payload.push(1);
            put_len(item.len(), &mut section_payload);
            section_payload.extend_from_slice(item);
        }
        let length = u32::try_from(section_payload.len()).expect("section length fits");
        directory.push((*section, items.len() as u32, offset, length));
        offset = offset.checked_add(length).expect("payload offset fits");
        payloads.push(section_payload);
    }

    let mut bytes = Vec::from(b"MIZAR_KERNEL_EVIDENCE\0".as_slice());
    put_u16(1, &mut bytes);
    put_u16(1, &mut bytes);
    put_formula_profile(&mut bytes);
    put_fingerprint(target, &mut bytes);
    put_u32(sections.len() as u32, &mut bytes);
    for (section, count, payload_offset, payload_length) in directory {
        bytes.push(section.byte());
        put_u32(count, &mut bytes);
        put_u32(payload_offset, &mut bytes);
        put_u32(payload_length, &mut bytes);
    }
    for payload in payloads {
        bytes.extend(payload);
    }
    bytes
}

fn formula_symbol_items() -> Vec<Vec<u8>> {
    [1_u32, 2_u32]
        .into_iter()
        .map(|id| {
            let mut item = Vec::new();
            item.push(symbol_kind_tag(SymbolKind::Predicate));
            put_u32(id, &mut item);
            item
        })
        .collect()
}

fn provenance_item(target: &Fingerprint, provenance_id: u32, fingerprint: &Fingerprint) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(provenance_id, &mut item);
    put_fingerprint(target, &mut item);
    put_fingerprint(fingerprint, &mut item);
    put_bytes(b"producer-payload", &mut item);
    item
}

fn put_formula(formula: &Formula, bytes: &mut Vec<u8>) {
    match formula {
        Formula::Atom(atom) => {
            bytes.push(1);
            put_atom(atom, bytes);
        }
        Formula::Not(child) => {
            bytes.push(2);
            put_formula(child, bytes);
        }
        Formula::And(children) => {
            bytes.push(3);
            put_u32(children.len() as u32, bytes);
            for child in children {
                put_formula(child, bytes);
            }
        }
        Formula::Or(children) => {
            bytes.push(4);
            put_u32(children.len() as u32, bytes);
            for child in children {
                put_formula(child, bytes);
            }
        }
        _ => panic!("unknown formula variant in status projection test fixture"),
    }
}

fn put_atom(atom: &Atom, bytes: &mut Vec<u8>) {
    bytes.push(symbol_kind_tag(atom.symbol.kind));
    put_u32(atom.symbol.id.0, bytes);
    put_u32(atom.arity, bytes);
    put_u32(0, bytes);
}

fn put_formula_profile(bytes: &mut Vec<u8>) {
    let profile = formula_profile();
    put_u16(profile.profile_id, bytes);
    put_u16(profile.clause_schema_version, bytes);
    put_u16(profile.clause_encoding_version, bytes);
    bytes.push(profile.clause_tautology_policy.tag());
    bytes.push(profile.certificate_hash_input_algorithm.tag());
}

fn put_fingerprint(fingerprint: &Fingerprint, bytes: &mut Vec<u8>) {
    bytes.push(fingerprint.algorithm_id);
    put_bytes(&fingerprint.digest, bytes);
}

fn put_bytes(payload: &[u8], bytes: &mut Vec<u8>) {
    put_len(payload.len(), bytes);
    bytes.extend_from_slice(payload);
}

fn put_len(len: usize, bytes: &mut Vec<u8>) {
    put_u32(u32::try_from(len).expect("length fits"), bytes);
}

fn put_u16(value: u16, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn put_u32(value: u32, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn symbol_kind_tag(kind: SymbolKind) -> u8 {
    match kind {
        SymbolKind::Predicate => 1,
        SymbolKind::FunctorPredicate => 2,
        SymbolKind::Equality => 3,
        SymbolKind::BuiltinRelation => 4,
        _ => panic!("unknown symbol kind in status projection test fixture"),
    }
}

fn required_status_tag(status: RequiredProofStatus) -> u8 {
    match status {
        RequiredProofStatus::KernelVerified => 1,
        RequiredProofStatus::DischargedBuiltin => 2,
        RequiredProofStatus::ExternallyAttestedPolicyPermitted => 3,
        _ => panic!("unknown required proof status in status projection test fixture"),
    }
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}
