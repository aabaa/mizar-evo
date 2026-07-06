use super::*;
use crate::{
    certificate_parser::{
        ClauseRefNamespace, ClauseTautologyPolicy, DerivedFact, FinalGoalNamespace, FinalGoalRef,
        Fingerprint, GeneratedClause, KernelProfileRecord, ParsedCertificateTestParts,
        RequiredProofStatus, ResolutionStep, SubstitutionEntry, SymbolManifestEntry,
        VariableManifestEntry,
    },
    clause::{
        Atom, ClauseForm, Literal, Polarity, SymbolId, SymbolKey, SymbolKind, TautologyPolicy,
        Term, VariableId,
    },
    formula_evidence::{
        Formula, FormulaEvidenceParseContext, FormulaSourceClass, GoalPolarity,
        IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID, ImportedStatementProjection,
        SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
        canonical_imported_statement_projection_payload, parse_formula_evidence,
    },
    substitution_checker::{Replacement, SubstitutionPayload, SubstitutionPayloadEntry, TermPath},
};

#[test]
fn sat_backed_kernel_evidence_accepts_only_unsat_wrapper_result() {
    let target = formula_target(7);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let premise = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &premise)],
        goal_item(20, &premise),
    );

    let context = formula_evidence_context_with_identity(&target_vc, &parsed);

    let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, Some(&context)));

    assert_eq!(result.status(), KernelCheckStatus::Accepted);
    assert!(result.sat_check_report().is_some());
    assert!(result.checked_resolution_steps().is_empty());
    assert!(result.used_axioms().is_empty());
    assert!(result.rejections().is_empty());
}

#[test]
fn sat_backed_kernel_evidence_instantiates_formula_substitution_before_sat_check() {
    let target = formula_target(8);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let source = formula_atom_with_variable(1, 1);
    let instantiated = formula_atom_with_variable(1, 2);
    let parsed = parsed_formula_evidence_with_substitutions(
        &target,
        vec![variable_item(1), variable_item(2)],
        vec![formula_item(1, 10, &source)],
        vec![formula_substitution_item(2, 1, 11, 1, &var(2))],
        goal_item(20, &instantiated),
    );
    let context = formula_evidence_context_with_identity(&target_vc, &parsed);
    let mut input = evidence_input(&target_vc, &parsed, Some(&context));
    input.limits.max_report_records = 1;

    let result = check_kernel_evidence(input);

    assert_eq!(result.status(), KernelCheckStatus::Accepted);
    assert!(result.sat_check_report().is_some());
    assert!(result.checked_substitutions().is_empty());
    assert!(result.rejections().is_empty());
}

#[test]
fn sat_backed_kernel_evidence_rejects_satisfiable_goal() {
    let target = formula_target(7);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let parsed = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &formula_atom(1))],
        goal_item(20, &formula_atom(2)),
    );

    let context = formula_evidence_context_with_identity(&target_vc, &parsed);

    let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, Some(&context)));

    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert_eq!(result.rejections().len(), 1);
    assert_eq!(
        result.rejections()[0].detail(),
        RejectionDetail::InvalidSatRefutation
    );
    assert_eq!(
        result.rejections()[0].location().field_path,
        Some("sat_checker.satisfiable")
    );
    assert!(result.sat_check_report().is_none());
}

#[test]
fn sat_backed_kernel_evidence_binds_goal_polarity_to_check_kind() {
    let target = formula_target(10);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let not_goal = Formula::Not(Box::new(goal.clone()));

    let proof_obligation = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &goal)],
        goal_item_with_polarity(20, GoalPolarity::AssertFalseForRefutation, &goal),
    );
    let consistency_check = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &not_goal)],
        goal_item_with_polarity(20, GoalPolarity::AssertTrueForConsistency, &goal),
    );

    let proof_context = formula_evidence_context_with_identity(&target_vc, &proof_obligation);
    let consistency_context =
        formula_evidence_context_with_identity(&target_vc, &consistency_check);

    let accepted_proof = check_kernel_evidence(evidence_input_with_check_kind(
        &target_vc,
        &proof_obligation,
        Some(&proof_context),
        KernelEvidenceCheckKind::ProofObligation,
    ));
    assert_eq!(accepted_proof.status(), KernelCheckStatus::Accepted);
    assert_eq!(
        accepted_proof.evidence_check_kind(),
        Some(KernelEvidenceCheckKind::ProofObligation)
    );

    let accepted_consistency = check_kernel_evidence(evidence_input_with_check_kind(
        &target_vc,
        &consistency_check,
        Some(&consistency_context),
        KernelEvidenceCheckKind::ConsistencyCheck,
    ));
    assert_eq!(accepted_consistency.status(), KernelCheckStatus::Accepted);
    assert_eq!(
        accepted_consistency.evidence_check_kind(),
        Some(KernelEvidenceCheckKind::ConsistencyCheck)
    );

    let proof_polarity_mismatch = check_kernel_evidence(evidence_input_with_check_kind(
        &target_vc,
        &consistency_check,
        None,
        KernelEvidenceCheckKind::ProofObligation,
    ));
    assert_goal_polarity_mismatch(&proof_polarity_mismatch);

    let consistency_polarity_mismatch = check_kernel_evidence(evidence_input_with_check_kind(
        &target_vc,
        &proof_obligation,
        None,
        KernelEvidenceCheckKind::ConsistencyCheck,
    ));
    assert_goal_polarity_mismatch(&consistency_polarity_mismatch);
}

#[test]
fn sat_backed_kernel_evidence_rejects_goal_polarity_before_context_and_sat() {
    let target = formula_target(11);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let not_goal = Formula::Not(Box::new(goal.clone()));
    let parsed = parsed_formula_evidence(
        &target,
        vec![imported_formula_item(1, 10, &not_goal)],
        goal_item_with_polarity(20, GoalPolarity::AssertTrueForConsistency, &goal),
    );

    let result = check_kernel_evidence(evidence_input_with_check_kind(
        &target_vc,
        &parsed,
        None,
        KernelEvidenceCheckKind::ProofObligation,
    ));

    assert_goal_polarity_mismatch(&result);
    assert!(result.checked_imports().is_empty());
    assert!(result.sat_check_report().is_none());
}

#[test]
fn sat_backed_kernel_evidence_rejects_target_context_mismatch() {
    let target = formula_target(7);
    let other_target = TargetVcFingerprint::from_certificate_fingerprint(&formula_target(8));
    let premise = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &premise)],
        goal_item(20, &premise),
    );

    let result = check_kernel_evidence(evidence_input(&other_target, &parsed, None));

    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        result.rejections()[0].detail(),
        RejectionDetail::ContextMismatch
    );
    assert_eq!(
        result.rejections()[0].location().field_path,
        Some("target_vc")
    );
}

#[test]
fn sat_backed_kernel_evidence_checks_imported_formula_context() {
    let target = formula_target(7);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let premise = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![imported_formula_item(1, 10, &premise)],
        goal_item(20, &premise),
    );
    let context = formula_evidence_context(
        formula_imported_fact(5, &premise, AcceptedProofStatus::KernelVerified),
        ImportedFactNamespace::ImportedAxiom,
    );

    let accepted = check_kernel_evidence(evidence_input(&target_vc, &parsed, Some(&context)));

    assert_eq!(accepted.status(), KernelCheckStatus::Accepted);
    assert_eq!(accepted.checked_imports().len(), 1);
    assert_ne!(
        accepted.checked_imports()[0].statement_fingerprint,
        formula_fingerprint(&premise)
    );
    assert_eq!(accepted.used_axioms().len(), 1);
    assert_eq!(accepted.used_axioms()[0].imported_fact_id, 5);

    let mut report_limited_input = evidence_input(&target_vc, &parsed, Some(&context));
    report_limited_input.limits.max_report_records = 2;
    let report_limited = check_kernel_evidence(report_limited_input);
    assert_eq!(report_limited.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        report_limited.rejections()[0].detail(),
        RejectionDetail::ResourceExhaustion
    );
    assert_eq!(
        report_limited.rejections()[0].location().field_path,
        Some("checker_limits.max_report_records")
    );

    let missing_context = check_kernel_evidence(evidence_input(&target_vc, &parsed, None));
    assert_eq!(missing_context.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        missing_context.rejections()[0].detail(),
        RejectionDetail::MissingProvenance
    );

    let wrong_identity_context = formula_evidence_context(
        formula_imported_fact_with_identity(
            5,
            b"other-pkg",
            b"module",
            b"ITEM",
            &premise,
            AcceptedProofStatus::KernelVerified,
        ),
        ImportedFactNamespace::ImportedAxiom,
    );
    let wrong_identity = check_kernel_evidence(evidence_input(
        &target_vc,
        &parsed,
        Some(&wrong_identity_context),
    ));
    assert_eq!(wrong_identity.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        wrong_identity.rejections()[0].detail(),
        RejectionDetail::UnresolvedSymbol
    );
    assert_eq!(
        wrong_identity.rejections()[0].location().field_path,
        Some("formula.imported_source")
    );

    let mut stale_projection =
        formula_imported_fact(5, &premise, AcceptedProofStatus::KernelVerified);
    stale_projection.statement_projection.payload =
        canonical_imported_statement_projection_payload(
            &stale_projection.statement_projection.statement_fingerprint,
            &formula_fingerprint(&Formula::Not(Box::new(premise.clone()))),
        )
        .expect("canonical stale imported statement projection");
    stale_projection.statement_projection.formula_fingerprint =
        formula_fingerprint(&Formula::Not(Box::new(premise.clone())));
    let stale_projection_context =
        formula_evidence_context(stale_projection, ImportedFactNamespace::ImportedAxiom);
    let stale_projection_result = check_kernel_evidence(evidence_input(
        &target_vc,
        &parsed,
        Some(&stale_projection_context),
    ));
    assert_eq!(
        stale_projection_result.status(),
        KernelCheckStatus::Rejected
    );
    assert_eq!(
        stale_projection_result.rejections()[0].detail(),
        RejectionDetail::UnresolvedSymbol
    );
    assert_eq!(
        stale_projection_result.rejections()[0]
            .location()
            .field_path,
        Some("formula.imported_statement_projection")
    );

    let mut payload_only_mismatch =
        formula_imported_fact(5, &premise, AcceptedProofStatus::KernelVerified);
    payload_only_mismatch.statement_projection.payload = b"not-canonical".to_vec();
    assert_imported_projection_rejects(&target_vc, &parsed, payload_only_mismatch);

    let mut unsupported_statement_projection =
        formula_imported_fact(5, &premise, AcceptedProofStatus::KernelVerified);
    unsupported_statement_projection
        .statement_projection
        .statement_fingerprint = Fingerprint::new(99, b"unsupported-statement".to_vec());
    assert_imported_projection_rejects(&target_vc, &parsed, unsupported_statement_projection);

    let mut empty_formula_projection =
        formula_imported_fact(5, &premise, AcceptedProofStatus::KernelVerified);
    let empty_formula_fingerprint =
        Fingerprint::new(SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID, vec![]);
    empty_formula_projection
        .statement_projection
        .formula_fingerprint = empty_formula_fingerprint;
    assert_imported_projection_rejects(&target_vc, &parsed, empty_formula_projection);

    let mut statement_projection_mismatch =
        formula_imported_fact(5, &premise, AcceptedProofStatus::KernelVerified);
    let other_statement = Fingerprint::new(
        IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID,
        b"other-statement".to_vec(),
    );
    statement_projection_mismatch
        .statement_projection
        .statement_fingerprint = other_statement;
    assert_imported_projection_rejects(&target_vc, &parsed, statement_projection_mismatch);

    let ambiguous_context = formula_evidence_context_entries(
        vec![
            formula_imported_fact(5, &premise, AcceptedProofStatus::KernelVerified),
            formula_imported_fact(6, &premise, AcceptedProofStatus::KernelVerified),
        ],
        Vec::new(),
    );
    let ambiguous = check_kernel_evidence(evidence_input(
        &target_vc,
        &parsed,
        Some(&ambiguous_context),
    ));
    assert_eq!(ambiguous.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        ambiguous.rejections()[0].detail(),
        RejectionDetail::UnresolvedSymbol
    );
    assert_eq!(
        ambiguous.rejections()[0].location().imported_fact_id,
        Some(5)
    );

    let weak_context = formula_evidence_context(
        formula_imported_fact(5, &premise, AcceptedProofStatus::DischargedBuiltin),
        ImportedFactNamespace::ImportedAxiom,
    );
    let weak = check_kernel_evidence(evidence_input(&target_vc, &parsed, Some(&weak_context)));
    assert_eq!(weak.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        weak.rejections()[0].detail(),
        RejectionDetail::UnresolvedSymbol
    );

    let external_parsed = parsed_formula_evidence(
        &target,
        vec![imported_formula_item_with_required_status(
            1,
            10,
            &premise,
            RequiredProofStatus::ExternallyAttestedPolicyPermitted,
        )],
        goal_item(20, &premise),
    );
    let external_context = formula_evidence_context(
        formula_imported_fact(
            5,
            &premise,
            AcceptedProofStatus::ExternallyAttestedPolicyPermitted,
        ),
        ImportedFactNamespace::ImportedAxiom,
    );
    let denied_external = check_kernel_evidence(evidence_input(
        &target_vc,
        &external_parsed,
        Some(&external_context),
    ));
    assert_eq!(denied_external.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        denied_external.rejections()[0].detail(),
        RejectionDetail::UnresolvedSymbol
    );

    let mut allowed_external_input =
        evidence_input(&target_vc, &external_parsed, Some(&external_context));
    allowed_external_input
        .policy
        .imported_fact_policy
        .allow_externally_attested = true;
    let allowed_external = check_kernel_evidence(allowed_external_input);
    assert_eq!(allowed_external.status(), KernelCheckStatus::Accepted);
    assert!(allowed_external.policy_taint());
    assert!(allowed_external.checked_imports()[0].policy_taint);
}

#[test]
fn sat_backed_kernel_evidence_checks_context_identity_for_non_imported_sources() {
    let target = formula_target(12);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let cited = formula_atom(2);
    let parsed = parsed_formula_evidence(
        &target,
        vec![
            formula_item(1, 10, &goal),
            cited_formula_item(2, 11, 2, &cited),
            generated_formula_item(3, 12, 7, &goal),
        ],
        goal_item(20, &goal),
    );
    let context = formula_evidence_context_with_identity_payload(context_identity_payload(
        &target_vc,
        vec![
            KernelContextIdentityEntry::new(
                KernelContextIdentitySource::GeneratedVcFact { vc_fact_id: 7 },
                3,
                formula_fingerprint(&goal),
                KernelFormulaProducerRef::Generated(KernelVcGeneratedFormulaId::new(3)),
            ),
            KernelContextIdentityEntry::new(
                KernelContextIdentitySource::LocalHypothesis {
                    local_context_id: 1,
                },
                1,
                formula_fingerprint(&goal),
                KernelFormulaProducerRef::Core(CoreFormulaId::new(1)),
            ),
            KernelContextIdentityEntry::new(
                KernelContextIdentitySource::CitedPremise {
                    local_context_id: 2,
                },
                2,
                formula_fingerprint(&cited),
                KernelFormulaProducerRef::Core(CoreFormulaId::new(2)),
            ),
        ],
    ));

    let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, Some(&context)));

    assert_eq!(result.status(), KernelCheckStatus::Accepted);
    assert!(result.checked_imports().is_empty());
    assert!(result.used_axioms().is_empty());
}

#[test]
fn sat_backed_kernel_evidence_rejects_context_identity_formula_id_mismatch() {
    let target = formula_target(13);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![cited_formula_item(2, 11, 2, &goal)],
        goal_item(20, &goal),
    );
    let context = formula_evidence_context_with_identity_payload(context_identity_payload(
        &target_vc,
        vec![KernelContextIdentityEntry::new(
            KernelContextIdentitySource::CitedPremise {
                local_context_id: 2,
            },
            99,
            formula_fingerprint(&goal),
            KernelFormulaProducerRef::Core(CoreFormulaId::new(99)),
        )],
    ));

    let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, Some(&context)));

    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        result.rejections()[0].detail(),
        RejectionDetail::MissingProvenance
    );
    assert_eq!(
        result.rejections()[0].location().field_path,
        Some("formula.context_identity")
    );
}

#[test]
fn sat_backed_kernel_evidence_requires_context_identity_for_local_sources() {
    let target = formula_target(13);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &goal)],
        goal_item(20, &goal),
    );
    let import_only_context = formula_evidence_context_entries(Vec::new(), Vec::new());

    let missing_context = check_kernel_evidence(evidence_input(&target_vc, &parsed, None));
    assert_eq!(missing_context.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        missing_context.rejections()[0].detail(),
        RejectionDetail::MissingProvenance
    );
    assert_eq!(
        missing_context.rejections()[0].location().field_path,
        Some("formula_context")
    );

    let missing_identity = check_kernel_evidence(evidence_input(
        &target_vc,
        &parsed,
        Some(&import_only_context),
    ));
    assert_eq!(missing_identity.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        missing_identity.rejections()[0].detail(),
        RejectionDetail::MissingProvenance
    );
    assert_eq!(
        missing_identity.rejections()[0].location().field_path,
        Some("formula_context.context_identity")
    );
}

#[test]
fn sat_backed_kernel_evidence_exempts_policy_bounded_builtin_from_context_identity() {
    let target = formula_target(17);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![policy_bounded_builtin_formula_item(
            1,
            10,
            b"builtin-tautology",
            &goal,
        )],
        goal_item(20, &goal),
    );

    let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, None));

    assert_eq!(result.status(), KernelCheckStatus::Accepted);
    assert!(result.rejections().is_empty());
    assert!(result.used_axioms().is_empty());
}

#[test]
fn sat_backed_kernel_evidence_rejects_stale_context_identity_payloads() {
    let target = formula_target(14);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &goal)],
        goal_item(20, &goal),
    );

    let wrong_target_identity = context_identity_payload(
        &TargetVcFingerprint::from_certificate_fingerprint(&formula_target(99)),
        context_identity_entries_for(&parsed),
    );
    let wrong_target_context =
        formula_evidence_context_with_identity_payload(wrong_target_identity);
    let wrong_target = check_kernel_evidence(evidence_input(
        &target_vc,
        &parsed,
        Some(&wrong_target_context),
    ));
    assert_eq!(wrong_target.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        wrong_target.rejections()[0].location().field_path,
        Some("formula_context.context_identity.target_vc")
    );

    let mut stale_hash_identity =
        context_identity_payload(&target_vc, context_identity_entries_for(&parsed));
    stale_hash_identity.context_identity_hash = Hash::from_bytes([9; Hash::BYTE_LEN]);
    let stale_hash_context = formula_evidence_context_with_identity_payload(stale_hash_identity);
    let stale_hash = check_kernel_evidence(evidence_input(
        &target_vc,
        &parsed,
        Some(&stale_hash_context),
    ));
    assert_eq!(stale_hash.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        stale_hash.rejections()[0].location().field_path,
        Some("formula_context.context_identity.hash")
    );

    let mut wrong_row_entries = context_identity_entries_for(&parsed);
    wrong_row_entries[0] = KernelContextIdentityEntry::new(
        KernelContextIdentitySource::LocalHypothesis {
            local_context_id: 2,
        },
        1,
        formula_fingerprint(&goal),
        KernelFormulaProducerRef::Core(CoreFormulaId::new(1)),
    );
    let wrong_row_context = formula_evidence_context_with_identity_payload(
        context_identity_payload(&target_vc, wrong_row_entries),
    );
    let wrong_row = check_kernel_evidence(evidence_input(
        &target_vc,
        &parsed,
        Some(&wrong_row_context),
    ));
    assert_eq!(wrong_row.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        wrong_row.rejections()[0].location().field_path,
        Some("formula.context_identity")
    );

    let mut duplicate_entries = context_identity_entries_for(&parsed);
    duplicate_entries.push(duplicate_entries[0].clone());
    let duplicate_context = formula_evidence_context_with_identity_payload(
        context_identity_payload(&target_vc, duplicate_entries),
    );
    let duplicate = check_kernel_evidence(evidence_input(
        &target_vc,
        &parsed,
        Some(&duplicate_context),
    ));
    assert_eq!(duplicate.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        duplicate.rejections()[0].location().field_path,
        Some("formula.context_identity")
    );
}

#[test]
fn sat_backed_kernel_evidence_rejects_goal_labeled_as_local_hypothesis_without_context_row() {
    let target = formula_target(15);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &goal)],
        goal_item(20, &goal),
    );
    let immutable_context_rows = vec![KernelContextIdentityEntry::new(
        KernelContextIdentitySource::LocalHypothesis {
            local_context_id: 1,
        },
        1,
        formula_fingerprint(&formula_atom(2)),
        KernelFormulaProducerRef::Core(CoreFormulaId::new(1)),
    )];
    let context = formula_evidence_context_with_identity_payload(context_identity_payload(
        &target_vc,
        immutable_context_rows,
    ));

    let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, Some(&context)));

    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        result.rejections()[0].detail(),
        RejectionDetail::MissingProvenance
    );
    assert_eq!(
        result.rejections()[0].location().field_path,
        Some("formula.context_identity")
    );
}

#[test]
fn sat_backed_kernel_evidence_limits_context_identity_entries_before_sat() {
    let target = formula_target(16);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let goal = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &goal)],
        goal_item(20, &goal),
    );
    let context = formula_evidence_context_with_identity(&target_vc, &parsed);
    let mut input = evidence_input(&target_vc, &parsed, Some(&context));
    input.limits.formula_context.max_context_identity_entries = 0;

    let result = check_kernel_evidence(input);

    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        result.rejections()[0].detail(),
        RejectionDetail::ResourceExhaustion
    );
    assert_eq!(
        result.rejections()[0].location().field_path,
        Some("formula_context.context_identity.entries")
    );
}

#[test]
fn context_identity_hash_uses_task_28_golden_line_grammar() {
    let target = TargetVcFingerprint::new(18, vec![0xab, 0xcd]);
    let canonical = Hash::from_bytes([0x42; Hash::BYTE_LEN]);
    let entries = vec![
        KernelContextIdentityEntry::new(
            KernelContextIdentitySource::GeneratedVcFact { vc_fact_id: 7 },
            3,
            Fingerprint::new(2, vec![0x30]),
            KernelFormulaProducerRef::Generated(KernelVcGeneratedFormulaId::new(9)),
        ),
        KernelContextIdentityEntry::new(
            KernelContextIdentitySource::CitedPremise {
                local_context_id: 2,
            },
            2,
            Fingerprint::new(2, vec![0x11]),
            KernelFormulaProducerRef::Core(CoreFormulaId::new(5)),
        ),
        KernelContextIdentityEntry::new(
            KernelContextIdentitySource::LocalHypothesis {
                local_context_id: 1,
            },
            1,
            Fingerprint::new(2, vec![0x10, 0x20]),
            KernelFormulaProducerRef::Core(CoreFormulaId::new(4)),
        ),
    ];
    let provisional =
        KernelContextIdentityPayload::new(target, canonical, Hash::from_bytes([0; 32]), entries);

    let hash_input = context_identity_hash_input(&provisional);

    assert_eq!(
        String::from_utf8(hash_input).expect("utf8"),
        "vc-kernel-context-identity-v1\n\
schema-version=1\n\
target-vc=18:abcd\n\
canonical-evidence-hash=4242424242424242424242424242424242424242424242424242424242424242\n\
[entries]\n\
source=LocalHypothesis { local_context_id: 1 }; formula-id=1; fingerprint=2:1020; producer=Core(CoreFormulaId(4))\n\
source=CitedPremise { local_context_id: 2 }; formula-id=2; fingerprint=2:11; producer=Core(CoreFormulaId(5))\n\
source=GeneratedVcFact { vc_fact_id: 7 }; formula-id=3; fingerprint=2:30; producer=Generated(VcGeneratedFormulaId(9))\n"
    );
    assert_eq!(
        recompute_context_identity_hash(&provisional),
        Hash::from_bytes([
            0xe6, 0xec, 0x91, 0x89, 0x9b, 0x20, 0x41, 0xe4, 0x02, 0x4f, 0xf8, 0x9a, 0xfe, 0x66,
            0x6f, 0xbf, 0xab, 0x9c, 0x11, 0x76, 0xde, 0x13, 0x86, 0x76, 0xbb, 0x8c, 0xb5, 0x4b,
            0x03, 0xdd, 0x0a, 0xb7,
        ])
    );
}

#[test]
fn sat_backed_kernel_evidence_batch_preserves_equal_target_order() {
    let target = formula_target(7);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let accepted_premise = formula_atom(1);
    let accepted = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &accepted_premise)],
        goal_item(20, &accepted_premise),
    );
    let rejected = parsed_formula_evidence(
        &target,
        vec![formula_item(1, 10, &formula_atom(1))],
        goal_item(20, &formula_atom(2)),
    );
    let rejected_context = formula_evidence_context_with_identity(&target_vc, &rejected);
    let accepted_context = formula_evidence_context_with_identity(&target_vc, &accepted);

    let results = check_kernel_evidence_batch(&[
        evidence_input(&target_vc, &rejected, Some(&rejected_context)),
        evidence_input(&target_vc, &accepted, Some(&accepted_context)),
    ]);

    assert_eq!(results[0].status(), KernelCheckStatus::Rejected);
    assert_eq!(results[1].status(), KernelCheckStatus::Accepted);
}

#[test]
fn valid_cluster_and_reduction_trace_replays_in_trace_order() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();
    let mut cluster = cluster_step(2, CheckedFactRef::ImportedAxiom(1));
    cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
    let mut reduction = reduction_step(4, CheckedFactRef::TraceStep(2));
    reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
    reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![cluster.clone()],
        vec![reduction.clone()],
        cluster_limits(),
    )
    .expect("cluster trace context");

    let report = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect("valid cluster trace");

    assert_eq!(report.checked_cluster_steps().len(), 1);
    assert_eq!(report.checked_cluster_steps()[0].cluster_trace_step_id, 2);
    assert_eq!(report.checked_reduction_steps().len(), 1);
    assert_eq!(report.checked_reduction_steps()[0].reduction_step_id, 4);
}

#[test]
fn cluster_trace_missing_context_or_provenance_is_missing_provenance() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();

    let no_request = replay_cluster_trace(cluster_input_with_requested(
        &target,
        &facts,
        None,
        Vec::new(),
    ))
    .expect("empty request does not require context");
    assert!(no_request.checked_cluster_steps().is_empty());

    let missing = replay_cluster_trace(cluster_input(&target, &facts, None))
        .expect_err("missing cluster trace context");
    assert_eq!(missing.detail(), RejectionDetail::MissingProvenance);
    assert_eq!(missing.location().field_path, Some("cluster_trace_context"));

    let mut cluster = cluster_step(1, CheckedFactRef::ImportedAxiom(1));
    cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
    let context = ClusterTraceContext::new(
        Some(Vec::new()),
        vec![cluster],
        Vec::new(),
        cluster_limits(),
    )
    .expect("context");
    let missing_provenance = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("missing provenance");
    assert_eq!(
        missing_provenance.detail(),
        RejectionDetail::MissingProvenance
    );
}

#[test]
fn cluster_trace_rejects_hidden_or_future_dependencies_and_mutated_facts() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();

    let mut future_dependency = cluster_step(2, CheckedFactRef::TraceStep(3));
    future_dependency.generated_fact_fingerprint =
        expected_cluster_fact_fingerprint(&future_dependency);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![future_dependency],
        Vec::new(),
        cluster_limits(),
    )
    .expect("context");
    let dependency_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("future dependency");
    assert_eq!(
        dependency_error.detail(),
        RejectionDetail::InvalidClusterTrace
    );

    let mut missing_transitive = cluster_step(4, CheckedFactRef::TraceStep(2));
    missing_transitive.generated_fact_fingerprint =
        expected_cluster_fact_fingerprint(&missing_transitive);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![missing_transitive],
        Vec::new(),
        cluster_limits(),
    )
    .expect("context");
    let missing_transitive_error = replay_cluster_trace(cluster_input_with_requested(
        &target,
        &facts,
        Some(&context),
        vec![4],
    ))
    .expect_err("missing transitive dependency");
    assert_eq!(
        missing_transitive_error.detail(),
        RejectionDetail::InvalidClusterTrace
    );
    assert_eq!(
        missing_transitive_error.location().cluster_trace_step_id,
        Some(4)
    );
    assert_eq!(
        missing_transitive_error.location().field_path,
        Some("dependency")
    );

    let mut mutated_fact = cluster_step(2, CheckedFactRef::ImportedAxiom(1));
    mutated_fact.generated_fact_fingerprint = b"wrong".to_vec();
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![mutated_fact],
        Vec::new(),
        cluster_limits(),
    )
    .expect("context");
    let fact_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("mutated generated fact");
    assert_eq!(fact_error.detail(), RejectionDetail::InvalidClusterTrace);
    assert_eq!(
        fact_error.location().field_path,
        Some("generated_fact_fingerprint")
    );
}

#[test]
fn reduction_trace_rejects_guard_strategy_and_result_mismatches() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();

    let mut missing_guard = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    missing_guard.required_guard_ids = vec![1, 2];
    missing_guard.strategy_audit_key = expected_strategy_audit_key(&missing_guard);
    missing_guard.result_fingerprint = expected_reduction_result_fingerprint(&missing_guard);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        Vec::new(),
        vec![missing_guard],
        cluster_limits(),
    )
    .expect("context");
    let guard_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("missing guard");
    assert_eq!(guard_error.detail(), RejectionDetail::InvalidClusterTrace);
    assert_eq!(guard_error.location().field_path, Some("discharged_guards"));

    let mut bad_audit = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    bad_audit.strategy_audit_key = b"bad-audit".to_vec();
    bad_audit.result_fingerprint = expected_reduction_result_fingerprint(&bad_audit);
    let context =
        ClusterTraceContext::new(Some(vec![1]), Vec::new(), vec![bad_audit], cluster_limits())
            .expect("context");
    let audit_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("strategy audit mismatch");
    assert_eq!(
        audit_error.location().field_path,
        Some("strategy_audit_key")
    );

    let mut bad_result = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    bad_result.strategy_audit_key = expected_strategy_audit_key(&bad_result);
    bad_result.result_fingerprint = b"bad-result".to_vec();
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        Vec::new(),
        vec![bad_result],
        cluster_limits(),
    )
    .expect("context");
    let result_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("result mismatch");
    assert_eq!(
        result_error.location().field_path,
        Some("result_fingerprint")
    );
}

#[test]
fn cluster_trace_context_is_bounded_sorted_and_unique() {
    let mut first = cluster_step(1, CheckedFactRef::ImportedAxiom(1));
    first.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&first);
    let mut second = cluster_step(3, CheckedFactRef::GeneratedClause(7));
    second.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&second);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![second.clone(), first.clone()],
        Vec::new(),
        cluster_limits(),
    )
    .expect("context canonicalizes order");
    assert_eq!(
        context
            .cluster_steps()
            .iter()
            .map(|step| step.cluster_trace_step_id)
            .collect::<Vec<_>>(),
        [1, 3]
    );

    let duplicate = ClusterTraceContext::new(
        Some(vec![1]),
        vec![first],
        vec![reduction_step(1, CheckedFactRef::ImportedAxiom(1))],
        cluster_limits(),
    )
    .expect_err("cross namespace duplicate");
    assert_eq!(
        duplicate,
        ClusterTraceContextError::DuplicateTraceStep { step_id: 1 }
    );

    let over_budget = ClusterTraceContext::new(
        Some(vec![1]),
        vec![second],
        Vec::new(),
        ClusterTraceReplayLimits {
            max_cluster_steps: 0,
            ..cluster_limits()
        },
    )
    .expect_err("cluster count limit");
    assert_eq!(
        over_budget,
        ClusterTraceContextError::TraceStepCountExceeded { max: 0, actual: 1 }
    );
}

#[test]
fn cluster_trace_replays_only_requested_steps_and_dependencies() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();
    let mut requested = cluster_step(1, CheckedFactRef::ImportedAxiom(1));
    requested.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&requested);
    let mut unused_malformed = cluster_step(3, CheckedFactRef::ImportedAxiom(1));
    unused_malformed.generated_fact_fingerprint = b"wrong".to_vec();
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![unused_malformed, requested],
        Vec::new(),
        cluster_limits(),
    )
    .expect("context");

    let report = replay_cluster_trace(cluster_input_with_requested(
        &target,
        &facts,
        Some(&context),
        vec![1],
    ))
    .expect("unrequested malformed evidence is ignored");

    assert_eq!(report.checked_cluster_steps().len(), 1);
    assert_eq!(report.checked_cluster_steps()[0].cluster_trace_step_id, 1);
}

#[test]
fn cluster_trace_closes_requested_dependencies_in_global_order() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();
    let mut base_cluster = cluster_step(2, CheckedFactRef::ImportedAxiom(1));
    base_cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&base_cluster);
    let mut reduction = reduction_step(4, CheckedFactRef::TraceStep(2));
    reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
    reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
    let mut requested_cluster = cluster_step(6, CheckedFactRef::TraceStep(4));
    requested_cluster.generated_fact_fingerprint =
        expected_cluster_fact_fingerprint(&requested_cluster);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![requested_cluster, base_cluster],
        vec![reduction],
        cluster_limits(),
    )
    .expect("context");

    let report = replay_cluster_trace(cluster_input_with_requested(
        &target,
        &facts,
        Some(&context),
        vec![6],
    ))
    .expect("transitive dependencies replay before requested id");

    assert_eq!(
        report
            .checked_cluster_steps()
            .iter()
            .map(|step| step.cluster_trace_step_id)
            .collect::<Vec<_>>(),
        [2, 6]
    );
    assert_eq!(
        report
            .checked_reduction_steps()
            .iter()
            .map(|step| step.reduction_step_id)
            .collect::<Vec<_>>(),
        [4]
    );
}

#[test]
fn cluster_trace_rejects_unchecked_base_fact_dependencies() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();

    let mut missing_import = cluster_step(1, CheckedFactRef::ImportedAxiom(99));
    missing_import.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&missing_import);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![missing_import],
        Vec::new(),
        cluster_limits(),
    )
    .expect("context");
    let import_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("unchecked imported fact");
    assert_eq!(import_error.detail(), RejectionDetail::InvalidClusterTrace);
    assert_eq!(import_error.location().cluster_trace_step_id, Some(1));

    let mut missing_generated = cluster_step(2, CheckedFactRef::GeneratedClause(99));
    missing_generated.generated_fact_fingerprint =
        expected_cluster_fact_fingerprint(&missing_generated);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![missing_generated],
        Vec::new(),
        cluster_limits(),
    )
    .expect("context");
    let generated_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("unchecked generated clause");
    assert_eq!(
        generated_error.detail(),
        RejectionDetail::InvalidClusterTrace
    );
    assert_eq!(generated_error.location().cluster_trace_step_id, Some(2));
}

#[test]
fn reduction_guards_match_exactly_and_dependencies_are_checked() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();

    let mut order_insensitive = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    order_insensitive.required_guard_ids = vec![2, 1];
    order_insensitive.discharged_guards = vec![
        GuardEvidence {
            guard_id: 1,
            source_fact_ref: CheckedFactRef::ImportedAxiom(1),
            checked_dependency_ref: CheckedFactRef::GeneratedClause(7),
        },
        GuardEvidence {
            guard_id: 2,
            source_fact_ref: CheckedFactRef::GeneratedClause(7),
            checked_dependency_ref: CheckedFactRef::ImportedAxiom(1),
        },
    ];
    order_insensitive.strategy_audit_key = expected_strategy_audit_key(&order_insensitive);
    order_insensitive.result_fingerprint =
        expected_reduction_result_fingerprint(&order_insensitive);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        Vec::new(),
        vec![order_insensitive],
        cluster_limits(),
    )
    .expect("context");
    replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect("guard ids match independent of evidence order");

    let mut extra_guard = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    extra_guard.discharged_guards.push(GuardEvidence {
        guard_id: 2,
        source_fact_ref: CheckedFactRef::ImportedAxiom(1),
        checked_dependency_ref: CheckedFactRef::ImportedAxiom(1),
    });
    extra_guard.strategy_audit_key = expected_strategy_audit_key(&extra_guard);
    extra_guard.result_fingerprint = expected_reduction_result_fingerprint(&extra_guard);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        Vec::new(),
        vec![extra_guard],
        cluster_limits(),
    )
    .expect("context");
    let extra_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("extra guard id");
    assert_eq!(extra_error.location().field_path, Some("discharged_guards"));

    let mut duplicate_guard = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    duplicate_guard.required_guard_ids = vec![1, 1];
    duplicate_guard.discharged_guards.push(GuardEvidence {
        guard_id: 1,
        source_fact_ref: CheckedFactRef::ImportedAxiom(1),
        checked_dependency_ref: CheckedFactRef::ImportedAxiom(1),
    });
    duplicate_guard.strategy_audit_key = expected_strategy_audit_key(&duplicate_guard);
    duplicate_guard.result_fingerprint = expected_reduction_result_fingerprint(&duplicate_guard);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        Vec::new(),
        vec![duplicate_guard],
        cluster_limits(),
    )
    .expect("context");
    let duplicate_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("duplicate guard id");
    assert_eq!(
        duplicate_error.location().field_path,
        Some("discharged_guards")
    );

    let mut bad_source = reduction_step(1, CheckedFactRef::ImportedAxiom(99));
    bad_source.strategy_audit_key = expected_strategy_audit_key(&bad_source);
    bad_source.result_fingerprint = expected_reduction_result_fingerprint(&bad_source);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        Vec::new(),
        vec![bad_source],
        cluster_limits(),
    )
    .expect("context");
    let source_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("unchecked guard source");
    assert_eq!(
        source_error.location().field_path,
        Some("guard.source_fact_ref")
    );

    let mut bad_checked = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    bad_checked.discharged_guards[0].checked_dependency_ref = CheckedFactRef::GeneratedClause(99);
    bad_checked.strategy_audit_key = expected_strategy_audit_key(&bad_checked);
    bad_checked.result_fingerprint = expected_reduction_result_fingerprint(&bad_checked);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        Vec::new(),
        vec![bad_checked],
        cluster_limits(),
    )
    .expect("context");
    let checked_error = replay_cluster_trace(cluster_input(&target, &facts, Some(&context)))
        .expect_err("unchecked guard dependency");
    assert_eq!(
        checked_error.location().field_path,
        Some("guard.checked_dependency_ref")
    );
}

#[test]
fn unused_context_entries_are_ignored_after_bounded_construction() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();
    let mut requested = cluster_step(1, CheckedFactRef::ImportedAxiom(1));
    requested.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&requested);
    let unused_malformed_reduction = reduction_step(3, CheckedFactRef::ImportedAxiom(99));
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![requested],
        vec![unused_malformed_reduction],
        cluster_limits(),
    )
    .expect("context");
    let mut input = cluster_input_with_requested(&target, &facts, Some(&context), vec![1]);
    input.limits.max_reduction_steps = 0;

    let report = replay_cluster_trace(input).expect("unused reduction is ignored");

    assert_eq!(report.checked_cluster_steps().len(), 1);
    assert!(report.checked_reduction_steps().is_empty());
}

#[test]
fn cluster_trace_context_rejects_duplicates_and_canonicalizes_reductions() {
    let duplicate_cluster = ClusterTraceContext::new(
        Some(vec![1]),
        vec![
            cluster_step(1, CheckedFactRef::ImportedAxiom(1)),
            cluster_step(1, CheckedFactRef::GeneratedClause(7)),
        ],
        Vec::new(),
        cluster_limits(),
    )
    .expect_err("duplicate cluster id");
    assert_eq!(
        duplicate_cluster,
        ClusterTraceContextError::DuplicateClusterStep { step_id: 1 }
    );

    let duplicate_reduction = ClusterTraceContext::new(
        Some(vec![1]),
        Vec::new(),
        vec![
            reduction_step(1, CheckedFactRef::ImportedAxiom(1)),
            reduction_step(1, CheckedFactRef::GeneratedClause(7)),
        ],
        cluster_limits(),
    )
    .expect_err("duplicate reduction id");
    assert_eq!(
        duplicate_reduction,
        ClusterTraceContextError::DuplicateReductionStep { step_id: 1 }
    );

    let base_duplicate = CheckedFactContext::new(vec![1, 1], Vec::new(), Vec::new())
        .expect_err("duplicate imported base fact");
    assert_eq!(
        base_duplicate,
        ClusterTraceContextError::DuplicateBaseFact {
            namespace: BaseFactNamespace::ImportedAxiom,
            id: 1
        }
    );

    let mut first = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    first.strategy_audit_key = expected_strategy_audit_key(&first);
    first.result_fingerprint = expected_reduction_result_fingerprint(&first);
    let mut second = reduction_step(3, CheckedFactRef::ImportedAxiom(1));
    second.strategy_audit_key = expected_strategy_audit_key(&second);
    second.result_fingerprint = expected_reduction_result_fingerprint(&second);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        Vec::new(),
        vec![second, first],
        cluster_limits(),
    )
    .expect("reduction context canonicalizes order");
    assert_eq!(
        context
            .reduction_steps()
            .iter()
            .map(|step| step.reduction_step_id)
            .collect::<Vec<_>>(),
        [1, 3]
    );
}

#[test]
fn cluster_trace_runtime_limits_are_resource_exhaustion() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();
    let mut cluster = cluster_step(1, CheckedFactRef::ImportedAxiom(1));
    cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
    let context =
        ClusterTraceContext::new(Some(vec![1]), vec![cluster], Vec::new(), cluster_limits())
            .expect("context");
    let mut input = cluster_input(&target, &facts, Some(&context));
    input.limits.max_trace_field_bytes = 1;

    let error = replay_cluster_trace(input).expect_err("field byte limit");

    assert_eq!(error.detail(), RejectionDetail::ResourceExhaustion);

    let mut reduction = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    reduction.required_guard_ids = vec![1];
    reduction.discharged_guards.clear();
    reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
    reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
    let context =
        ClusterTraceContext::new(Some(vec![1]), Vec::new(), vec![reduction], cluster_limits())
            .expect("reduction context");
    let mut input = cluster_input(&target, &facts, Some(&context));
    input.limits.max_guard_evidence = 0;

    let guard_limit = replay_cluster_trace(input).expect_err("guard count limit");

    assert_eq!(guard_limit.detail(), RejectionDetail::ResourceExhaustion);
    assert_eq!(
        guard_limit.location().field_path,
        Some("required_guard_ids")
    );

    let empty_context =
        ClusterTraceContext::new(Some(vec![1]), Vec::new(), Vec::new(), cluster_limits())
            .expect("empty context");
    let mut requested_over_budget =
        cluster_input_with_requested(&target, &facts, Some(&empty_context), vec![1, 2]);
    requested_over_budget.limits.max_trace_steps = 1;
    let requested_error =
        replay_cluster_trace(requested_over_budget).expect_err("requested id count limit");
    assert_eq!(
        requested_error.detail(),
        RejectionDetail::ResourceExhaustion
    );
    assert_eq!(
        requested_error.location().field_path,
        Some("requested_trace_steps")
    );

    let mut dependency = cluster_step(1, CheckedFactRef::ImportedAxiom(1));
    dependency.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&dependency);
    let mut requested = cluster_step(2, CheckedFactRef::TraceStep(1));
    requested.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&requested);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![dependency, requested],
        Vec::new(),
        cluster_limits(),
    )
    .expect("context");
    let mut closure_limited =
        cluster_input_with_requested(&target, &facts, Some(&context), vec![2]);
    closure_limited.limits.max_trace_steps = 1;
    let closure_error = replay_cluster_trace(closure_limited).expect_err("closure count limit");
    assert_eq!(closure_error.detail(), RejectionDetail::ResourceExhaustion);
    assert_eq!(
        closure_error.location().field_path,
        Some("requested_trace_steps")
    );

    let mut reduction = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
    reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
    let context =
        ClusterTraceContext::new(Some(vec![1]), Vec::new(), vec![reduction], cluster_limits())
            .expect("context");
    let mut runtime_limited = cluster_input(&target, &facts, Some(&context));
    runtime_limited.limits.max_reduction_steps = 0;
    let runtime_error = replay_cluster_trace(runtime_limited).expect_err("runtime context limit");
    assert_eq!(runtime_error.detail(), RejectionDetail::ResourceExhaustion);
    assert_eq!(
        runtime_error.location().field_path,
        Some("cluster_trace_context.reduction_steps")
    );

    let mut reduction = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
    reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
    let context =
        ClusterTraceContext::new(Some(vec![1]), Vec::new(), vec![reduction], cluster_limits())
            .expect("context");
    let mut binding_limited = cluster_input(&target, &facts, Some(&context));
    binding_limited.limits.max_reduction_bindings = 0;
    let binding_error = replay_cluster_trace(binding_limited).expect_err("binding count limit");
    assert_eq!(binding_error.detail(), RejectionDetail::ResourceExhaustion);
    assert_eq!(binding_error.location().field_path, Some("substitution"));

    let mut cluster = cluster_step(1, CheckedFactRef::ImportedAxiom(1));
    cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
    let context =
        ClusterTraceContext::new(Some(vec![1]), vec![cluster], Vec::new(), cluster_limits())
            .expect("context");
    let mut commitment_limited = cluster_input(&target, &facts, Some(&context));
    commitment_limited.limits.max_commitment_bytes = 1;
    let commitment_error =
        replay_cluster_trace(commitment_limited).expect_err("commitment byte limit");
    assert_eq!(
        commitment_error.detail(),
        RejectionDetail::ResourceExhaustion
    );
    assert_eq!(
        commitment_error.location().field_path,
        Some("generated_fact_fingerprint")
    );
}

#[test]
fn kernel_service_rejects_legacy_certificate_without_migration_audit_policy() {
    let (certificate, context) = resolution_service_fixture(vec![42]);
    let target = TargetVcFingerprint::from_certificate_fingerprint(&certificate.target_vc);
    let input = KernelCheckInput {
        target_vc_fingerprint: &target,
        certificate: &certificate,
        imported_fact_context: Some(&context),
        substitution_context: None,
        cluster_trace_context: None,
        requested_cluster_trace_steps: &[],
        policy: KernelCheckPolicy::default(),
        limits: service_limits(),
    };

    let result = check_kernel_certificate(input);

    assert_service_rejection(
        result,
        RejectionCategory::CertificateRejection,
        RejectionDetail::UnsupportedCertificateFormat,
        RejectionLocation::new().with_field_path("policy.allow_legacy_certificate_audit"),
    );
}

#[test]
fn kernel_service_reports_legacy_audit_pipeline_without_acceptance() {
    let (certificate, context) = resolution_service_fixture(vec![42]);
    let target = TargetVcFingerprint::from_certificate_fingerprint(&certificate.target_vc);
    let mut cluster = cluster_step(1, CheckedFactRef::ImportedAxiom(1));
    cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
    let cluster_context =
        ClusterTraceContext::new(Some(vec![7]), vec![cluster], Vec::new(), cluster_limits())
            .expect("cluster context");

    let result = check_kernel_certificate(service_input(
        &target,
        &certificate,
        &context,
        Some(&cluster_context),
        &[1],
        KernelCheckPolicy::default(),
        service_limits(),
    ));

    assert_legacy_audit_result(
        &result,
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );
    assert_eq!(result.checked_imports().len(), 1);
    assert!(result.checked_substitutions().is_empty());
    assert_eq!(result.checked_resolution_steps().len(), 1);
    assert_eq!(result.checked_cluster_steps().len(), 1);
    assert!(result.checked_derived_facts().is_empty());
    assert!(!result.policy_taint());

    let (mut extra_import, _) = resolution_service_fixture(vec![42]);
    let unused_clause = ordinary(vec![neg_p()]);
    let unused = imported_ref(
        9,
        b"pkg",
        b"mod",
        b"unused",
        clause_fingerprint(&unused_clause),
        RequiredProofStatus::KernelVerified,
    );
    extra_import.imported_theorems.push(unused.clone());
    let extra_context = ImportedFactContext::new(
        Some(vec![1]),
        vec![evidence(
            &extra_import.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            ordinary(vec![neg_p()]),
        )],
        vec![evidence(
            &unused,
            AcceptedProofStatus::KernelVerified,
            unused_clause,
        )],
        context_limits(),
    )
    .expect("extra context");
    let target = TargetVcFingerprint::from_certificate_fingerprint(&extra_import.target_vc);
    let extra_result = check_kernel_certificate(service_input(
        &target,
        &extra_import,
        &extra_context,
        None,
        &[],
        KernelCheckPolicy::default(),
        service_limits(),
    ));
    assert_legacy_audit_result(
        &extra_result,
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );
    assert!(extra_result.used_axioms().is_empty());
}

#[test]
fn kernel_service_rejects_legacy_tautology_marker_final_goal_in_audit_mode() {
    let (certificate, context) = legacy_tautology_service_fixture(vec![42]);
    let target = TargetVcFingerprint::from_certificate_fingerprint(&certificate.target_vc);

    let result = check_kernel_certificate(service_input(
        &target,
        &certificate,
        &context,
        None,
        &[],
        KernelCheckPolicy::default(),
        service_limits(),
    ));

    assert_service_rejection(
        result,
        RejectionCategory::KernelRejection,
        RejectionDetail::InvalidSatProof,
        RejectionLocation::new().with_final_goal(),
    );
}

#[test]
fn kernel_service_preserves_import_namespaces_and_checks_substitutions() {
    let (mut certificate, _) = resolution_service_fixture(vec![42]);
    certificate.substitutions = vec![simple_substitution(1, var(1), var(2))];
    let theorem_clause = ordinary(vec![pos_p()]);
    let theorem = imported_ref(
        1,
        b"pkg",
        b"mod",
        b"theorem",
        clause_fingerprint(&theorem_clause),
        RequiredProofStatus::KernelVerified,
    );
    certificate.imported_theorems.push(theorem.clone());
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            ordinary(vec![neg_p()]),
        )],
        vec![evidence(
            &theorem,
            AcceptedProofStatus::KernelVerified,
            theorem_clause,
        )],
        context_limits(),
    )
    .expect("same numeric ids are allowed across imported namespaces");
    let substitution_context = simple_substitution_context(1, var(2));
    let mut cluster = cluster_step(1, CheckedFactRef::ImportedTheorem(1));
    cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
    let cluster_context =
        ClusterTraceContext::new(Some(vec![7]), vec![cluster], Vec::new(), cluster_limits())
            .expect("cluster context");
    let target = TargetVcFingerprint::from_certificate_fingerprint(&certificate.target_vc);

    let result = check_kernel_certificate(service_input_with_substitutions(
        &target,
        &certificate,
        &context,
        Some(&substitution_context),
        Some(&cluster_context),
        &[1],
        service_limits(),
    ));

    assert_legacy_audit_result(
        &result,
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );
    assert_eq!(result.checked_imports().len(), 2);
    assert_eq!(result.checked_substitutions().len(), 1);
    assert_eq!(result.checked_substitutions()[0].substitution_id, 1);
    assert_eq!(result.checked_cluster_steps().len(), 1);
    assert!(result.used_axioms().is_empty());

    let missing_substitution_context = check_kernel_certificate(service_input(
        &target,
        &certificate,
        &context,
        None,
        &[],
        KernelCheckPolicy::default(),
        service_limits(),
    ));
    assert_eq!(
        missing_substitution_context.rejections()[0].detail(),
        RejectionDetail::MissingProvenance
    );
    assert_eq!(
        missing_substitution_context.rejections()[0]
            .location()
            .field_path,
        Some("substitution_context")
    );
    assert_eq!(missing_substitution_context.rejections().len(), 1);
}

#[test]
fn kernel_service_treats_checked_generated_clause_ids_as_a_base_set() {
    let (mut certificate, context) = resolution_service_fixture(vec![42]);
    certificate.resolution_trace.push(resolution_step(
        2,
        clause_ref(ClauseRefNamespace::ImportedAxiom, 1),
        clause_ref(ClauseRefNamespace::GeneratedClause, 2),
        neg_p(),
        clause_ref(ClauseRefNamespace::GeneratedClause, 3),
    ));
    let mut cluster = cluster_step(1, CheckedFactRef::GeneratedClause(3));
    cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
    let cluster_context =
        ClusterTraceContext::new(Some(vec![7]), vec![cluster], Vec::new(), cluster_limits())
            .expect("cluster context");
    let target = TargetVcFingerprint::from_certificate_fingerprint(&certificate.target_vc);

    let result = check_kernel_certificate(service_input(
        &target,
        &certificate,
        &context,
        Some(&cluster_context),
        &[1],
        KernelCheckPolicy::default(),
        service_limits(),
    ));

    assert_legacy_audit_result(
        &result,
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );
    assert_eq!(result.checked_resolution_steps().len(), 2);
    assert_eq!(result.checked_cluster_steps().len(), 1);
}

#[test]
fn kernel_service_rejects_target_final_goal_and_derived_fact_gaps() {
    let (certificate, context) = resolution_service_fixture(vec![42]);
    let wrong_target = TargetVcFingerprint::new(1, vec![99]);
    let target_error = check_kernel_certificate(service_input(
        &wrong_target,
        &certificate,
        &context,
        None,
        &[],
        KernelCheckPolicy::default(),
        service_limits(),
    ));
    assert_eq!(target_error.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        target_error.rejections()[0].detail(),
        RejectionDetail::ContextMismatch
    );
    assert_eq!(
        target_error.rejections()[0].category(),
        RejectionCategory::CertificateRejection
    );

    let (mut unchecked_final, context) = resolution_service_fixture(vec![42]);
    unchecked_final.final_goal = FinalGoalRef {
        namespace: FinalGoalNamespace::GeneratedClause,
        id: 99,
    };
    let target = TargetVcFingerprint::from_certificate_fingerprint(&unchecked_final.target_vc);
    let final_error = check_kernel_certificate(service_input(
        &target,
        &unchecked_final,
        &context,
        None,
        &[],
        KernelCheckPolicy::default(),
        service_limits(),
    ));
    assert_eq!(final_error.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        final_error.rejections()[0].detail(),
        RejectionDetail::InvalidSatProof
    );
    assert!(final_error.rejections()[0].location().final_goal);

    let (mut unchecked_present_final, context) = resolution_service_fixture(vec![42]);
    unchecked_present_final.final_goal = FinalGoalRef {
        namespace: FinalGoalNamespace::GeneratedClause,
        id: 2,
    };
    let target =
        TargetVcFingerprint::from_certificate_fingerprint(&unchecked_present_final.target_vc);
    let unchecked_present_error = check_kernel_certificate(service_input(
        &target,
        &unchecked_present_final,
        &context,
        None,
        &[],
        KernelCheckPolicy::default(),
        service_limits(),
    ));
    assert_eq!(
        unchecked_present_error.rejections()[0].detail(),
        RejectionDetail::InvalidSatProof
    );
    assert!(
        unchecked_present_error.rejections()[0]
            .location()
            .final_goal
    );

    let (mut derived, context) = resolution_service_fixture(vec![42]);
    derived.derived_facts.push(DerivedFact {
        derived_fact_id: 1,
        source: clause_ref(ClauseRefNamespace::ResolutionStep, 1),
        payload: b"unsupported".to_vec(),
    });
    let target = TargetVcFingerprint::from_certificate_fingerprint(&derived.target_vc);
    let derived_error = check_kernel_certificate(service_input(
        &target,
        &derived,
        &context,
        None,
        &[],
        KernelCheckPolicy::default(),
        service_limits(),
    ));
    assert_eq!(
        derived_error.rejections()[0].detail(),
        RejectionDetail::InvalidSatProof
    );
    assert_eq!(
        derived_error.rejections()[0].location().derived_fact_id,
        Some(1)
    );

    let (mut derived_goal, context) = resolution_service_fixture(vec![42]);
    derived_goal.final_goal = FinalGoalRef {
        namespace: FinalGoalNamespace::DerivedFact,
        id: 7,
    };
    let target = TargetVcFingerprint::from_certificate_fingerprint(&derived_goal.target_vc);
    let derived_goal_error = check_kernel_certificate(service_input(
        &target,
        &derived_goal,
        &context,
        None,
        &[],
        KernelCheckPolicy::default(),
        service_limits(),
    ));
    assert_eq!(
        derived_goal_error.rejections()[0].detail(),
        RejectionDetail::InvalidSatProof
    );
    assert!(derived_goal_error.rejections()[0].location().final_goal);

    let (mut over_derived_limit, context) = resolution_service_fixture(vec![42]);
    over_derived_limit.derived_facts.push(DerivedFact {
        derived_fact_id: 8,
        source: clause_ref(ClauseRefNamespace::ResolutionStep, 1),
        payload: b"unsupported".to_vec(),
    });
    let target = TargetVcFingerprint::from_certificate_fingerprint(&over_derived_limit.target_vc);
    let mut derived_limits = service_limits();
    derived_limits.max_derived_facts = 0;
    let derived_limit_error = check_kernel_certificate(service_input(
        &target,
        &over_derived_limit,
        &context,
        None,
        &[],
        KernelCheckPolicy::default(),
        derived_limits,
    ));
    assert_eq!(
        derived_limit_error.rejections()[0].detail(),
        RejectionDetail::ResourceExhaustion
    );
    assert_eq!(
        derived_limit_error.rejections()[0].location().field_path,
        Some("derived_facts")
    );
}

#[test]
fn kernel_service_soundness_fail_corpus_rejects_single_mutations() {
    let (import_certificate, _) = resolution_service_fixture(vec![42]);
    let import_target =
        TargetVcFingerprint::from_certificate_fingerprint(&import_certificate.target_vc);
    let bad_import = imported_ref(
        1,
        b"bad-pkg",
        b"mod",
        b"axiom",
        import_certificate.imported_axioms[0]
            .statement_fingerprint
            .clone(),
        RequiredProofStatus::KernelVerified,
    );
    let bad_import_context = ImportedFactContext::new(
        Some(vec![1]),
        vec![evidence(
            &bad_import,
            AcceptedProofStatus::KernelVerified,
            ordinary(vec![neg_p()]),
        )],
        Vec::new(),
        context_limits(),
    )
    .expect("mutated imported context");
    assert_service_rejection(
        check_kernel_certificate(service_input(
            &import_target,
            &import_certificate,
            &bad_import_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::UnresolvedSymbol,
        RejectionLocation::new()
            .with_imported_fact_id(1)
            .with_field_path("package_id"),
    );

    let (mut substitution_certificate, substitution_context_base) =
        resolution_service_fixture(vec![42]);
    substitution_certificate.substitutions = vec![simple_substitution(1, var(1), var(1))];
    let substitution_evidence = simple_substitution_context(1, var(2));
    let substitution_target =
        TargetVcFingerprint::from_certificate_fingerprint(&substitution_certificate.target_vc);
    assert_service_rejection(
        check_kernel_certificate(service_input_with_substitutions(
            &substitution_target,
            &substitution_certificate,
            &substitution_context_base,
            Some(&substitution_evidence),
            None,
            &[],
            service_limits(),
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::InvalidSubstitution,
        RejectionLocation::new()
            .with_substitution_id(1)
            .with_field_path("target_term"),
    );

    let (mut resolution_certificate, resolution_context) = resolution_service_fixture(vec![42]);
    resolution_certificate.resolution_trace[0].pivot_literal = pos_p();
    let resolution_target =
        TargetVcFingerprint::from_certificate_fingerprint(&resolution_certificate.target_vc);
    assert_service_rejection(
        check_kernel_certificate(service_input(
            &resolution_target,
            &resolution_certificate,
            &resolution_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::InvalidSatProof,
        RejectionLocation::new()
            .with_resolution_step_id(1)
            .with_clause_ref(crate::rejection::ClauseRef::new(
                crate::rejection::ClauseRefNamespace::ImportedAxiom,
                1,
            ))
            .with_field_path("pivot_literal"),
    );

    let (cluster_certificate, cluster_context_base) = resolution_service_fixture(vec![42]);
    let cluster_target =
        TargetVcFingerprint::from_certificate_fingerprint(&cluster_certificate.target_vc);
    let mut mutated_cluster = cluster_step(1, CheckedFactRef::ImportedAxiom(1));
    mutated_cluster.generated_fact_fingerprint = b"wrong".to_vec();
    let bad_cluster_context = ClusterTraceContext::new(
        Some(vec![7]),
        vec![mutated_cluster],
        Vec::new(),
        cluster_limits(),
    )
    .expect("mutated cluster context");
    assert_service_rejection(
        check_kernel_certificate(service_input(
            &cluster_target,
            &cluster_certificate,
            &cluster_context_base,
            Some(&bad_cluster_context),
            &[1],
            KernelCheckPolicy::default(),
            service_limits(),
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::InvalidClusterTrace,
        RejectionLocation::new()
            .with_cluster_trace_step_id(1)
            .with_field_path("generated_fact_fingerprint"),
    );

    let (reduction_certificate, reduction_context_base) = resolution_service_fixture(vec![42]);
    let reduction_target =
        TargetVcFingerprint::from_certificate_fingerprint(&reduction_certificate.target_vc);
    let mut mutated_reduction = reduction_step(1, CheckedFactRef::ImportedAxiom(1));
    mutated_reduction.strategy_audit_key = expected_strategy_audit_key(&mutated_reduction);
    mutated_reduction.result_fingerprint = b"wrong".to_vec();
    let bad_reduction_context = ClusterTraceContext::new(
        Some(vec![7]),
        Vec::new(),
        vec![mutated_reduction],
        cluster_limits(),
    )
    .expect("mutated reduction context");
    assert_service_rejection(
        check_kernel_certificate(service_input(
            &reduction_target,
            &reduction_certificate,
            &reduction_context_base,
            Some(&bad_reduction_context),
            &[1],
            KernelCheckPolicy::default(),
            service_limits(),
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::InvalidClusterTrace,
        RejectionLocation::new()
            .with_reduction_step_id(1)
            .with_field_path("result_fingerprint"),
    );

    let (mut final_goal_certificate, final_goal_context) = resolution_service_fixture(vec![42]);
    final_goal_certificate.final_goal = FinalGoalRef {
        namespace: FinalGoalNamespace::GeneratedClause,
        id: 2,
    };
    let final_goal_target =
        TargetVcFingerprint::from_certificate_fingerprint(&final_goal_certificate.target_vc);
    assert_service_rejection(
        check_kernel_certificate(service_input(
            &final_goal_target,
            &final_goal_certificate,
            &final_goal_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::InvalidSatProof,
        RejectionLocation::new().with_final_goal(),
    );

    let (mut derived_certificate, derived_context) = resolution_service_fixture(vec![42]);
    derived_certificate.derived_facts.push(DerivedFact {
        derived_fact_id: 1,
        source: clause_ref(ClauseRefNamespace::ResolutionStep, 1),
        payload: b"unsupported".to_vec(),
    });
    let derived_target =
        TargetVcFingerprint::from_certificate_fingerprint(&derived_certificate.target_vc);
    assert_service_rejection(
        check_kernel_certificate(service_input(
            &derived_target,
            &derived_certificate,
            &derived_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::InvalidSatProof,
        RejectionLocation::new()
            .with_derived_fact_id(1)
            .with_field_path("payload"),
    );

    let (timeout_certificate, timeout_context) = resolution_service_fixture(vec![42]);
    let timeout_target =
        TargetVcFingerprint::from_certificate_fingerprint(&timeout_certificate.target_vc);
    let mut timeout_limits = service_limits();
    timeout_limits.max_pipeline_steps = 0;
    assert_service_rejection(
        check_kernel_certificate(service_input(
            &timeout_target,
            &timeout_certificate,
            &timeout_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            timeout_limits,
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::Timeout,
        RejectionLocation::new().with_field_path("target_vc"),
    );

    let (resource_certificate, resource_context) = resolution_service_fixture(vec![42]);
    let resource_target =
        TargetVcFingerprint::from_certificate_fingerprint(&resource_certificate.target_vc);
    let mut resource_limits = service_limits();
    resource_limits.max_report_records = 0;
    assert_service_rejection(
        check_kernel_certificate(service_input(
            &resource_target,
            &resource_certificate,
            &resource_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            resource_limits,
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::ResourceExhaustion,
        RejectionLocation::new().with_field_path("checker_limits.max_report_records"),
    );
}

#[test]
fn kernel_service_results_are_deterministic_under_repetition_and_permutation() {
    let (mut certificate, _) = resolution_service_fixture(vec![42]);
    certificate.substitutions = vec![simple_substitution(1, var(1), var(2))];
    let extra_clause = ordinary(vec![pos_p()]);
    let extra_import = imported_ref(
        2,
        b"pkg",
        b"mod",
        b"extra",
        clause_fingerprint(&extra_clause),
        RequiredProofStatus::KernelVerified,
    );
    certificate.imported_axioms.push(extra_import.clone());

    let primary_evidence = evidence(
        &certificate.imported_axioms[0],
        AcceptedProofStatus::KernelVerified,
        ordinary(vec![neg_p()]),
    );
    let extra_evidence = evidence(
        &extra_import,
        AcceptedProofStatus::KernelVerified,
        extra_clause,
    );
    let imported_context = ImportedFactContext::new(
        Some(vec![1]),
        vec![extra_evidence.clone(), primary_evidence.clone()],
        Vec::new(),
        context_limits(),
    )
    .expect("permuted imported context");
    let imported_context_permuted = ImportedFactContext::new(
        Some(vec![1]),
        vec![primary_evidence, extra_evidence],
        Vec::new(),
        context_limits(),
    )
    .expect("canonical imported context");
    let substitution_context = simple_substitution_context(1, var(2));

    let mut cluster_two = cluster_step(2, CheckedFactRef::ImportedAxiom(1));
    cluster_two.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster_two);
    let mut cluster_three = cluster_step(3, CheckedFactRef::GeneratedClause(3));
    cluster_three.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster_three);
    let mut reduction_four = reduction_step(4, CheckedFactRef::TraceStep(2));
    reduction_four.strategy_audit_key = expected_strategy_audit_key(&reduction_four);
    reduction_four.result_fingerprint = expected_reduction_result_fingerprint(&reduction_four);
    let cluster_context = ClusterTraceContext::new(
        Some(vec![7]),
        vec![cluster_three.clone(), cluster_two.clone()],
        vec![reduction_four.clone()],
        cluster_limits(),
    )
    .expect("permuted cluster context");
    let cluster_context_permuted = ClusterTraceContext::new(
        Some(vec![7]),
        vec![cluster_two, cluster_three],
        vec![reduction_four],
        cluster_limits(),
    )
    .expect("canonical cluster context");
    let requested = [4, 3, 2];
    let requested_permuted = [2, 4, 3];
    let target = TargetVcFingerprint::from_certificate_fingerprint(&certificate.target_vc);

    let accepted = check_kernel_certificate(service_input_with_substitutions(
        &target,
        &certificate,
        &imported_context,
        Some(&substitution_context),
        Some(&cluster_context),
        &requested,
        service_limits(),
    ));
    let repeated = check_kernel_certificate(service_input_with_substitutions(
        &target,
        &certificate,
        &imported_context,
        Some(&substitution_context),
        Some(&cluster_context),
        &requested,
        service_limits(),
    ));
    let permuted = check_kernel_certificate(service_input_with_substitutions(
        &target,
        &certificate,
        &imported_context_permuted,
        Some(&substitution_context),
        Some(&cluster_context_permuted),
        &requested_permuted,
        service_limits(),
    ));

    assert_legacy_audit_result(
        &accepted,
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );
    assert_eq!(accepted, repeated);
    assert_eq!(accepted, permuted);
    assert_eq!(
        accepted
            .checked_cluster_steps()
            .iter()
            .map(|step| step.cluster_trace_step_id)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    assert_eq!(accepted.checked_reduction_steps()[0].reduction_step_id, 4);

    let wrong_target = TargetVcFingerprint::new(1, vec![99]);
    let rejected = check_kernel_certificate(service_input_with_substitutions(
        &wrong_target,
        &certificate,
        &imported_context,
        Some(&substitution_context),
        Some(&cluster_context),
        &requested,
        service_limits(),
    ));
    let rejected_again = check_kernel_certificate(service_input_with_substitutions(
        &wrong_target,
        &certificate,
        &imported_context,
        Some(&substitution_context),
        Some(&cluster_context),
        &requested,
        service_limits(),
    ));
    assert_eq!(rejected, rejected_again);
    assert_service_rejection(
        rejected,
        RejectionCategory::CertificateRejection,
        RejectionDetail::ContextMismatch,
        RejectionLocation::new().with_field_path("target_vc"),
    );
}

#[test]
fn kernel_service_batches_are_deterministic_for_distinct_targets_and_ties() {
    let (first_certificate, first_context) = resolution_service_fixture(vec![1]);
    let (second_certificate, second_context) = resolution_service_fixture(vec![2]);
    let (third_certificate, third_context) = resolution_service_fixture(vec![3]);
    let first_target =
        TargetVcFingerprint::from_certificate_fingerprint(&first_certificate.target_vc);
    let second_target =
        TargetVcFingerprint::from_certificate_fingerprint(&second_certificate.target_vc);
    let third_target =
        TargetVcFingerprint::from_certificate_fingerprint(&third_certificate.target_vc);
    let first_permutation = vec![
        service_input(
            &third_target,
            &third_certificate,
            &third_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
        service_input(
            &first_target,
            &first_certificate,
            &first_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
        service_input(
            &second_target,
            &second_certificate,
            &second_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
    ];
    let second_permutation = vec![
        service_input(
            &second_target,
            &second_certificate,
            &second_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
        service_input(
            &third_target,
            &third_certificate,
            &third_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
        service_input(
            &first_target,
            &first_certificate,
            &first_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
    ];

    let first_results = check_kernel_batch(&first_permutation);
    let second_results = check_kernel_batch(&second_permutation);

    assert_eq!(first_results, second_results);
    assert_eq!(
        first_results
            .iter()
            .map(|result| result.target_vc_fingerprint().digest.as_slice())
            .collect::<Vec<_>>(),
        [&[1][..], &[2][..], &[3][..]]
    );

    let (tie_first, tie_first_context) = resolution_service_fixture_with_final(
        vec![9],
        FinalGoalRef {
            namespace: FinalGoalNamespace::GeneratedClause,
            id: 3,
        },
    );
    let (tie_second, tie_second_context) = resolution_service_fixture_with_final(
        vec![9],
        FinalGoalRef {
            namespace: FinalGoalNamespace::ResolutionStep,
            id: 1,
        },
    );
    let (before_ties, before_ties_context) = resolution_service_fixture(vec![8]);
    let tie_first_target = TargetVcFingerprint::from_certificate_fingerprint(&tie_first.target_vc);
    let tie_second_target =
        TargetVcFingerprint::from_certificate_fingerprint(&tie_second.target_vc);
    let before_ties_target =
        TargetVcFingerprint::from_certificate_fingerprint(&before_ties.target_vc);
    let tie_inputs = vec![
        service_input(
            &tie_first_target,
            &tie_first,
            &tie_first_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
        service_input(
            &before_ties_target,
            &before_ties,
            &before_ties_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
        service_input(
            &tie_second_target,
            &tie_second,
            &tie_second_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
    ];
    let reversed_tie_inputs = vec![
        service_input(
            &tie_second_target,
            &tie_second,
            &tie_second_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
        service_input(
            &before_ties_target,
            &before_ties,
            &before_ties_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
        service_input(
            &tie_first_target,
            &tie_first,
            &tie_first_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
    ];

    let tie_results = check_kernel_batch(&tie_inputs);
    let reversed_tie_results = check_kernel_batch(&reversed_tie_inputs);

    assert_eq!(tie_results[0].target_vc_fingerprint().digest, vec![8]);
    assert_legacy_audit_result(
        &tie_results[1],
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );
    assert_legacy_audit_result(
        &tie_results[2],
        legacy_audit_location(FinalGoalNamespace::ResolutionStep, 1),
    );
    assert_legacy_audit_result(
        &reversed_tie_results[1],
        legacy_audit_location(FinalGoalNamespace::ResolutionStep, 1),
    );
    assert_legacy_audit_result(
        &reversed_tie_results[2],
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );
    assert_ne!(tie_results, reversed_tie_results);
}

#[test]
fn kernel_service_replay_cost_budgets_are_exact_and_stable() {
    let target = TargetVcFingerprint::new(1, vec![42]);
    let facts = checked_fact_context();
    let mut cluster = cluster_step(2, CheckedFactRef::ImportedAxiom(1));
    cluster.generated_fact_fingerprint = expected_cluster_fact_fingerprint(&cluster);
    let mut reduction = reduction_step(4, CheckedFactRef::TraceStep(2));
    reduction.strategy_audit_key = expected_strategy_audit_key(&reduction);
    reduction.result_fingerprint = expected_reduction_result_fingerprint(&reduction);
    let context = ClusterTraceContext::new(
        Some(vec![1]),
        vec![cluster],
        vec![reduction],
        cluster_limits(),
    )
    .expect("cluster trace context");
    let requested = [4];
    let mut exact_trace_limits = cluster_limits();
    exact_trace_limits.max_trace_steps = 2;
    exact_trace_limits.max_cluster_steps = 1;
    exact_trace_limits.max_reduction_steps = 1;
    let exact_report = replay_cluster_trace(ClusterTraceReplayInput {
        target_vc_fingerprint: &target,
        checked_fact_context: &facts,
        cluster_trace_context: Some(&context),
        requested_trace_steps: &requested,
        limits: exact_trace_limits,
    })
    .expect("exact trace budget passes");
    assert_eq!(exact_report.checked_cluster_steps().len(), 1);
    assert_eq!(exact_report.checked_reduction_steps().len(), 1);

    let mut trace_limited = exact_trace_limits;
    trace_limited.max_trace_steps = 1;
    let trace_error = replay_cluster_trace(ClusterTraceReplayInput {
        target_vc_fingerprint: &target,
        checked_fact_context: &facts,
        cluster_trace_context: Some(&context),
        requested_trace_steps: &requested,
        limits: trace_limited,
    })
    .expect_err("trace closure budget rejects");
    assert_rejection_record(
        &trace_error,
        RejectionCategory::KernelRejection,
        RejectionDetail::ResourceExhaustion,
        RejectionLocation::new().with_field_path("requested_trace_steps"),
    );

    let mut cluster_limited = exact_trace_limits;
    cluster_limited.max_cluster_steps = 0;
    let cluster_error = replay_cluster_trace(ClusterTraceReplayInput {
        target_vc_fingerprint: &target,
        checked_fact_context: &facts,
        cluster_trace_context: Some(&context),
        requested_trace_steps: &requested,
        limits: cluster_limited,
    })
    .expect_err("cluster count budget rejects");
    assert_rejection_record(
        &cluster_error,
        RejectionCategory::KernelRejection,
        RejectionDetail::ResourceExhaustion,
        RejectionLocation::new().with_field_path("cluster_trace_context.cluster_steps"),
    );

    let mut reduction_limited = exact_trace_limits;
    reduction_limited.max_reduction_steps = 0;
    let reduction_error = replay_cluster_trace(ClusterTraceReplayInput {
        target_vc_fingerprint: &target,
        checked_fact_context: &facts,
        cluster_trace_context: Some(&context),
        requested_trace_steps: &requested,
        limits: reduction_limited,
    })
    .expect_err("reduction count budget rejects");
    assert_rejection_record(
        &reduction_error,
        RejectionCategory::KernelRejection,
        RejectionDetail::ResourceExhaustion,
        RejectionLocation::new().with_field_path("cluster_trace_context.reduction_steps"),
    );

    let (certificate, imported_context) = resolution_service_fixture(vec![11]);
    let service_target = TargetVcFingerprint::from_certificate_fingerprint(&certificate.target_vc);
    let mut exact_pipeline_limits = service_limits();
    exact_pipeline_limits.max_pipeline_steps = 7;
    let exact_pipeline = check_kernel_certificate(service_input(
        &service_target,
        &certificate,
        &imported_context,
        None,
        &[],
        KernelCheckPolicy::default(),
        exact_pipeline_limits,
    ));
    assert_legacy_audit_result(
        &exact_pipeline,
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );

    let mut short_pipeline_limits = exact_pipeline_limits;
    short_pipeline_limits.max_pipeline_steps = 6;
    assert_service_rejection(
        check_kernel_certificate(service_input(
            &service_target,
            &certificate,
            &imported_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            short_pipeline_limits,
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::Timeout,
        RejectionLocation::new().with_field_path("final_goal"),
    );

    let mut exact_report_limits = service_limits();
    exact_report_limits.max_report_records = 3;
    let exact_report = check_kernel_certificate(service_input(
        &service_target,
        &certificate,
        &imported_context,
        None,
        &[],
        KernelCheckPolicy::default(),
        exact_report_limits,
    ));
    assert_legacy_audit_result(
        &exact_report,
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );

    let mut short_report_limits = exact_report_limits;
    short_report_limits.max_report_records = 2;
    assert_service_rejection(
        check_kernel_certificate(service_input(
            &service_target,
            &certificate,
            &imported_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            short_report_limits,
        )),
        RejectionCategory::KernelRejection,
        RejectionDetail::ResourceExhaustion,
        RejectionLocation::new().with_field_path("checker_limits.max_report_records"),
    );
}

#[test]
fn kernel_service_orders_batches_by_target_then_input_order() {
    let (later_first, later_first_context) = resolution_service_fixture_with_final(
        vec![2],
        FinalGoalRef {
            namespace: FinalGoalNamespace::GeneratedClause,
            id: 3,
        },
    );
    let (earlier, earlier_context) = resolution_service_fixture(vec![1]);
    let (later_second, later_second_context) = resolution_service_fixture_with_final(
        vec![2],
        FinalGoalRef {
            namespace: FinalGoalNamespace::ResolutionStep,
            id: 1,
        },
    );
    let later_first_target =
        TargetVcFingerprint::from_certificate_fingerprint(&later_first.target_vc);
    let earlier_target = TargetVcFingerprint::from_certificate_fingerprint(&earlier.target_vc);
    let later_second_target =
        TargetVcFingerprint::from_certificate_fingerprint(&later_second.target_vc);
    let inputs = vec![
        service_input(
            &later_first_target,
            &later_first,
            &later_first_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
        service_input(
            &earlier_target,
            &earlier,
            &earlier_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
        service_input(
            &later_second_target,
            &later_second,
            &later_second_context,
            None,
            &[],
            KernelCheckPolicy::default(),
            service_limits(),
        ),
    ];

    let results = check_kernel_batch(&inputs);

    assert_eq!(results[0].target_vc_fingerprint().digest, vec![1]);
    assert_eq!(results[1].target_vc_fingerprint().digest, vec![2]);
    assert_legacy_audit_result(
        &results[1],
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );
    assert_eq!(results[2].target_vc_fingerprint().digest, vec![2]);
    assert_legacy_audit_result(
        &results[2],
        legacy_audit_location(FinalGoalNamespace::ResolutionStep, 1),
    );
}

#[test]
fn kernel_service_propagates_policy_taint_timeout_and_resource_limits() {
    let (certificate, context) = resolution_service_fixture_with_status(
        vec![42],
        RequiredProofStatus::ExternallyAttestedPolicyPermitted,
        AcceptedProofStatus::ExternallyAttestedPolicyPermitted,
    );
    let target = TargetVcFingerprint::from_certificate_fingerprint(&certificate.target_vc);
    let result = check_kernel_certificate(service_input(
        &target,
        &certificate,
        &context,
        None,
        &[],
        KernelCheckPolicy {
            imported_fact_policy: ImportedFactPolicy {
                allow_externally_attested: true,
            },
            ..KernelCheckPolicy::default()
        },
        service_limits(),
    ));
    assert_legacy_audit_result(
        &result,
        legacy_audit_location(FinalGoalNamespace::GeneratedClause, 3),
    );
    assert!(result.policy_taint());
    assert_eq!(
        result.checked_imports()[0].accepted_proof_status,
        AcceptedProofStatus::ExternallyAttestedPolicyPermitted
    );

    let mut timeout_limits = service_limits();
    timeout_limits.max_pipeline_steps = 0;
    let timeout = check_kernel_certificate(service_input(
        &target,
        &certificate,
        &context,
        None,
        &[],
        KernelCheckPolicy {
            imported_fact_policy: ImportedFactPolicy {
                allow_externally_attested: true,
            },
            ..KernelCheckPolicy::default()
        },
        timeout_limits,
    ));
    assert_eq!(timeout.rejections()[0].detail(), RejectionDetail::Timeout);
    assert_eq!(
        timeout.rejections()[0].location().field_path,
        Some("target_vc")
    );

    let mut later_timeout_limits = service_limits();
    later_timeout_limits.max_pipeline_steps = 4;
    let later_timeout = check_kernel_certificate(service_input(
        &target,
        &certificate,
        &context,
        None,
        &[],
        KernelCheckPolicy {
            imported_fact_policy: ImportedFactPolicy {
                allow_externally_attested: true,
            },
            ..KernelCheckPolicy::default()
        },
        later_timeout_limits,
    ));
    assert_eq!(
        later_timeout.rejections()[0].detail(),
        RejectionDetail::Timeout
    );
    assert_eq!(
        later_timeout.rejections()[0].location().field_path,
        Some("cluster_trace_context")
    );

    let mut resource_limits = service_limits();
    resource_limits.max_report_records = 0;
    let resource = check_kernel_certificate(service_input(
        &target,
        &certificate,
        &context,
        None,
        &[],
        KernelCheckPolicy {
            imported_fact_policy: ImportedFactPolicy {
                allow_externally_attested: true,
            },
            ..KernelCheckPolicy::default()
        },
        resource_limits,
    ));
    assert_eq!(
        resource.rejections()[0].detail(),
        RejectionDetail::ResourceExhaustion
    );
    assert_eq!(
        resource.rejections()[0].location().field_path,
        Some("checker_limits.max_report_records")
    );
}

#[test]
fn valid_imports_build_resolution_context_and_policy_taint() {
    let axiom_clause = ordinary(vec![neg_p()]);
    let theorem_clause = ordinary(vec![pos_q()]);
    let certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            clause_fingerprint(&axiom_clause),
            RequiredProofStatus::DischargedBuiltin,
        )],
        vec![imported_ref(
            2,
            b"pkg",
            b"mod",
            b"theorem",
            clause_fingerprint(&theorem_clause),
            RequiredProofStatus::ExternallyAttestedPolicyPermitted,
        )],
    );
    let context = ImportedFactContext::new(
        Some(vec![9]),
        vec![evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            axiom_clause.clone(),
        )],
        vec![evidence(
            &certificate.imported_theorems[0],
            AcceptedProofStatus::ExternallyAttestedPolicyPermitted,
            theorem_clause.clone(),
        )],
        context_limits(),
    )
    .expect("context");

    let report = check_imported_facts(input_with_policy(
        &certificate,
        Some(&context),
        ImportedFactPolicy {
            allow_externally_attested: true,
        },
        limits(),
    ))
    .expect("valid imports");

    assert_eq!(report.checked_imports().len(), 2);
    assert_eq!(
        report.checked_imports()[0].namespace,
        ImportedFactNamespace::ImportedAxiom
    );
    assert_eq!(
        report.checked_imports()[1].namespace,
        ImportedFactNamespace::ImportedTheorem
    );
    assert!(report.checked_imports()[1].policy_taint);
    assert!(report.policy_taint());
    assert_eq!(
        report.imported_clause_context().imported_axiom_clauses()[0].clause,
        axiom_clause
    );
    assert_eq!(
        report.imported_clause_context().imported_theorem_clauses()[0].clause,
        theorem_clause
    );
}

#[test]
fn missing_context_or_provenance_is_missing_provenance() {
    let clause = ordinary(vec![neg_p()]);
    let certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            clause_fingerprint(&clause),
            RequiredProofStatus::KernelVerified,
        )],
        Vec::new(),
    );

    let missing =
        check_imported_facts(input(&certificate, None, limits())).expect_err("missing context");
    assert_eq!(missing.detail(), RejectionDetail::MissingProvenance);
    assert_eq!(missing.location().field_path, Some("imported_fact_context"));

    let context = ImportedFactContext::new(
        Some(Vec::new()),
        vec![evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            clause,
        )],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let missing_provenance = check_imported_facts(input(&certificate, Some(&context), limits()))
        .expect_err("empty provenance");
    assert_eq!(
        missing_provenance.detail(),
        RejectionDetail::MissingProvenance
    );
}

#[test]
fn identity_status_and_missing_evidence_fail_as_unresolved_symbol() {
    let clause = ordinary(vec![neg_p()]);
    let certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            clause_fingerprint(&clause),
            RequiredProofStatus::KernelVerified,
        )],
        Vec::new(),
    );

    let context = ImportedFactContext::new(Some(vec![1]), Vec::new(), Vec::new(), context_limits())
        .expect("context");
    let missing_evidence = check_imported_facts(input(&certificate, Some(&context), limits()))
        .expect_err("missing evidence");
    assert_eq!(missing_evidence.detail(), RejectionDetail::UnresolvedSymbol);
    assert_eq!(missing_evidence.location().imported_fact_id, Some(1));

    let mut wrong_identity = evidence(
        &certificate.imported_axioms[0],
        AcceptedProofStatus::KernelVerified,
        clause.clone(),
    );
    wrong_identity.package_id = b"other".to_vec();
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![wrong_identity],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let identity_error = check_imported_facts(input(&certificate, Some(&context), limits()))
        .expect_err("identity mismatch");
    assert_eq!(identity_error.detail(), RejectionDetail::UnresolvedSymbol);
    assert_eq!(identity_error.location().field_path, Some("package_id"));

    let mut wrong_statement = evidence(
        &certificate.imported_axioms[0],
        AcceptedProofStatus::KernelVerified,
        clause.clone(),
    );
    wrong_statement.statement_fingerprint = Fingerprint::new(1, vec![9, 9, 9]);
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![wrong_statement],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let statement_error = check_imported_facts(input(&certificate, Some(&context), limits()))
        .expect_err("evidence statement fingerprint mismatch");
    assert_eq!(statement_error.detail(), RejectionDetail::UnresolvedSymbol);
    assert_eq!(
        statement_error.location().field_path,
        Some("statement_fingerprint")
    );

    let builtin_status = evidence(
        &certificate.imported_axioms[0],
        AcceptedProofStatus::DischargedBuiltin,
        clause.clone(),
    );
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![builtin_status],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let builtin_status_error = check_imported_facts(input(&certificate, Some(&context), limits()))
        .expect_err("builtin status is weaker than kernel verified");
    assert_eq!(
        builtin_status_error.detail(),
        RejectionDetail::UnresolvedSymbol
    );
    assert_eq!(
        builtin_status_error.location().field_path,
        Some("required_proof_status")
    );

    let weak_status = evidence(
        &certificate.imported_axioms[0],
        AcceptedProofStatus::ExternallyAttestedPolicyPermitted,
        clause.clone(),
    );
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![weak_status],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let status_error = check_imported_facts(input(&certificate, Some(&context), limits()))
        .expect_err("status mismatch");
    assert_eq!(status_error.detail(), RejectionDetail::UnresolvedSymbol);
    assert_eq!(
        status_error.location().field_path,
        Some("required_proof_status")
    );

    let external_certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            clause_fingerprint(&clause),
            RequiredProofStatus::ExternallyAttestedPolicyPermitted,
        )],
        Vec::new(),
    );
    let external = evidence(
        &external_certificate.imported_axioms[0],
        AcceptedProofStatus::ExternallyAttestedPolicyPermitted,
        clause,
    );
    let context =
        ImportedFactContext::new(Some(vec![1]), vec![external], Vec::new(), context_limits())
            .expect("context");
    let policy_error = check_imported_facts(input(&external_certificate, Some(&context), limits()))
        .expect_err("external attestation is disabled by policy");
    assert_eq!(policy_error.detail(), RejectionDetail::UnresolvedSymbol);
    assert_eq!(
        policy_error.location().field_path,
        Some("required_proof_status")
    );
}

#[test]
fn imported_clause_fingerprint_binding_is_checked_before_replay() {
    let clause = ordinary(vec![neg_p()]);
    let certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            clause_fingerprint(&clause),
            RequiredProofStatus::KernelVerified,
        )],
        Vec::new(),
    );

    let mut wrong_normalized = evidence(
        &certificate.imported_axioms[0],
        AcceptedProofStatus::KernelVerified,
        clause.clone(),
    );
    wrong_normalized.normalized_clause_fingerprint = Fingerprint::new(1, vec![99]);
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![wrong_normalized],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let normalized_error = check_imported_facts(input(&certificate, Some(&context), limits()))
        .expect_err("normalized fingerprint mismatch");
    assert_eq!(
        normalized_error.location().field_path,
        Some("normalized_clause_fingerprint")
    );
    assert_eq!(normalized_error.detail(), RejectionDetail::UnresolvedSymbol);

    let wrong_statement = Fingerprint::new(1, vec![7, 7, 7]);
    let mismatched_certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            wrong_statement.clone(),
            RequiredProofStatus::KernelVerified,
        )],
        Vec::new(),
    );
    let mut mismatched_evidence = evidence(
        &mismatched_certificate.imported_axioms[0],
        AcceptedProofStatus::KernelVerified,
        clause,
    );
    mismatched_evidence.normalized_clause_fingerprint = Fingerprint::new(
        1,
        mismatched_evidence.clause.canonical_hash_input().unwrap(),
    );
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![mismatched_evidence],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let statement_error =
        check_imported_facts(input(&mismatched_certificate, Some(&context), limits()))
            .expect_err("statement fingerprint must bind to clause content");
    assert_eq!(
        statement_error.location().field_path,
        Some("statement_fingerprint")
    );

    let unsupported_certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            Fingerprint::new(9, clause_fingerprint(&ordinary(vec![neg_p()])).digest),
            RequiredProofStatus::KernelVerified,
        )],
        Vec::new(),
    );
    let unsupported = evidence(
        &unsupported_certificate.imported_axioms[0],
        AcceptedProofStatus::KernelVerified,
        ordinary(vec![neg_p()]),
    );
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![unsupported],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let unsupported_error =
        check_imported_facts(input(&unsupported_certificate, Some(&context), limits()))
            .expect_err("unsupported fingerprint algorithm");
    assert_eq!(
        unsupported_error.location().field_path,
        Some("statement_fingerprint.algorithm_id")
    );

    let mut unsupported_evidence_algorithm = evidence(
        &certificate.imported_axioms[0],
        AcceptedProofStatus::KernelVerified,
        ordinary(vec![neg_p()]),
    );
    unsupported_evidence_algorithm
        .normalized_clause_fingerprint
        .algorithm_id = 9;
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![unsupported_evidence_algorithm],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let unsupported_evidence_error =
        check_imported_facts(input(&certificate, Some(&context), limits()))
            .expect_err("unsupported evidence fingerprint algorithm");
    assert_eq!(
        unsupported_evidence_error.location().field_path,
        Some("normalized_clause_fingerprint.algorithm_id")
    );
}

#[test]
fn imported_clause_profile_manifest_and_resource_limits_are_checked() {
    let clause = ordinary(vec![neg_p()]);
    let certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            clause_fingerprint(&clause),
            RequiredProofStatus::KernelVerified,
        )],
        Vec::new(),
    );

    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            wrong_profile_clause(),
        )],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let profile_error = check_imported_facts(input(&certificate, Some(&context), limits()))
        .expect_err("profile mismatch");
    assert_eq!(profile_error.detail(), RejectionDetail::MissingProvenance);
    assert_eq!(profile_error.location().field_path, Some("clause.profile"));

    let unknown_symbol = unknown_symbol_clause();
    let unknown_symbol_fingerprint = clause_fingerprint(&unknown_symbol);
    let unknown_certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            unknown_symbol_fingerprint,
            RequiredProofStatus::KernelVerified,
        )],
        Vec::new(),
    );
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![evidence(
            &unknown_certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            unknown_symbol,
        )],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let manifest_error =
        check_imported_facts(input(&unknown_certificate, Some(&context), limits()))
            .expect_err("manifest mismatch");
    assert_eq!(manifest_error.detail(), RejectionDetail::MissingProvenance);
    assert_eq!(manifest_error.location().field_path, Some("clause"));

    let variable_clause = variable_clause(99);
    let variable_certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            clause_fingerprint(&variable_clause),
            RequiredProofStatus::KernelVerified,
        )],
        Vec::new(),
    );
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![evidence(
            &variable_certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            variable_clause,
        )],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let variable_error =
        check_imported_facts(input(&variable_certificate, Some(&context), limits()))
            .expect_err("variable manifest mismatch");
    assert_eq!(variable_error.detail(), RejectionDetail::MissingProvenance);
    assert_eq!(variable_error.location().field_path, Some("clause"));

    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![evidence(
            &certificate.imported_axioms[0],
            AcceptedProofStatus::KernelVerified,
            clause,
        )],
        Vec::new(),
        context_limits(),
    )
    .expect("context");
    let mut tiny_limits = limits();
    tiny_limits.max_imported_clause_literals = 0;
    let resource_error = check_imported_facts(input(&certificate, Some(&context), tiny_limits))
        .expect_err("literal limit");
    assert_eq!(resource_error.detail(), RejectionDetail::ResourceExhaustion);
}

#[test]
fn duplicate_context_ids_are_rejected_and_unused_malformed_entries_ignored() {
    let clause = ordinary(vec![neg_p()]);
    let certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            clause_fingerprint(&clause),
            RequiredProofStatus::KernelVerified,
        )],
        Vec::new(),
    );
    let first = evidence(
        &certificate.imported_axioms[0],
        AcceptedProofStatus::KernelVerified,
        clause.clone(),
    );
    let duplicate = first.clone();
    let duplicate_error = ImportedFactContext::new(
        Some(vec![1]),
        vec![first, duplicate],
        Vec::new(),
        context_limits(),
    )
    .expect_err("duplicate context id");
    assert_eq!(
        duplicate_error,
        ImportedFactContextError::DuplicateImportedFact {
            namespace: ImportedFactNamespace::ImportedAxiom,
            imported_fact_id: 1
        }
    );

    let mut unused_malformed = evidence(
        &imported_ref(
            99,
            b"pkg",
            b"mod",
            b"unused",
            Fingerprint::new(1, vec![99]),
            RequiredProofStatus::KernelVerified,
        ),
        AcceptedProofStatus::KernelVerified,
        wrong_profile_clause(),
    );
    unused_malformed.normalized_clause_fingerprint = Fingerprint::new(1, vec![99]);
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![
            unused_malformed,
            evidence(
                &certificate.imported_axioms[0],
                AcceptedProofStatus::KernelVerified,
                clause,
            ),
        ],
        Vec::new(),
        context_limits(),
    )
    .expect("context canonicalizes order");

    check_imported_facts(input(&certificate, Some(&context), limits()))
        .expect("unused malformed context entry is ignored");
}

#[test]
fn context_and_reports_are_canonical_under_shuffled_evidence() {
    let first_clause = ordinary(vec![neg_p()]);
    let second_clause = ordinary(vec![pos_q()]);
    let certificate = make_certificate(
        vec![
            imported_ref(
                1,
                b"pkg",
                b"mod",
                b"first",
                clause_fingerprint(&first_clause),
                RequiredProofStatus::KernelVerified,
            ),
            imported_ref(
                2,
                b"pkg",
                b"mod",
                b"second",
                clause_fingerprint(&second_clause),
                RequiredProofStatus::KernelVerified,
            ),
        ],
        Vec::new(),
    );
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![
            evidence(
                &certificate.imported_axioms[1],
                AcceptedProofStatus::KernelVerified,
                second_clause,
            ),
            evidence(
                &certificate.imported_axioms[0],
                AcceptedProofStatus::KernelVerified,
                first_clause,
            ),
        ],
        Vec::new(),
        context_limits(),
    )
    .expect("context canonicalizes evidence order");

    assert_eq!(
        context
            .imported_axioms()
            .iter()
            .map(|entry| entry.imported_fact_id)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let report = check_imported_facts(input(&certificate, Some(&context), limits()))
        .expect("valid shuffled context");

    assert_eq!(
        report
            .checked_imports()
            .iter()
            .map(|entry| entry.imported_fact_id)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(
        report
            .imported_clause_context()
            .imported_axiom_clauses()
            .iter()
            .map(|entry| entry.imported_fact_id)
            .collect::<Vec<_>>(),
        [1, 2]
    );
}

#[test]
fn context_constructor_rejects_entry_count_before_sorting() {
    let clause = ordinary(vec![neg_p()]);
    let imported = imported_ref(
        1,
        b"pkg",
        b"mod",
        b"axiom",
        clause_fingerprint(&clause),
        RequiredProofStatus::KernelVerified,
    );

    let error = ImportedFactContext::new(
        Some(vec![1]),
        vec![evidence(
            &imported,
            AcceptedProofStatus::KernelVerified,
            clause,
        )],
        Vec::new(),
        ImportedFactContextLimits {
            max_imported_context_entries: 0,
            max_context_identity_entries: usize::MAX,
        },
    )
    .expect_err("context entry count limit");

    assert_eq!(
        error,
        ImportedFactContextError::ImportedFactCountExceeded { max: 0, actual: 1 }
    );
}

#[test]
fn formula_context_constructor_rejects_context_identity_entry_count() {
    let target = TargetVcFingerprint::new(19, vec![0x19]);
    let formula = formula_atom(1);
    let identity = context_identity_payload(
        &target,
        vec![KernelContextIdentityEntry::new(
            KernelContextIdentitySource::LocalHypothesis {
                local_context_id: 1,
            },
            1,
            formula_fingerprint(&formula),
            KernelFormulaProducerRef::Core(CoreFormulaId::new(1)),
        )],
    );

    let error = FormulaEvidenceContext::with_context_identity(
        Some(vec![1]),
        Vec::new(),
        Vec::new(),
        Some(identity),
        ImportedFactContextLimits {
            max_imported_context_entries: usize::MAX,
            max_context_identity_entries: 0,
        },
    )
    .expect_err("context identity entry count limit");

    assert_eq!(
        error,
        ImportedFactContextError::ContextIdentityCountExceeded { max: 0, actual: 1 }
    );
}

#[test]
fn imported_fact_count_limit_rejects_before_context_lookup() {
    let clause = ordinary(vec![neg_p()]);
    let certificate = make_certificate(
        vec![imported_ref(
            1,
            b"pkg",
            b"mod",
            b"axiom",
            clause_fingerprint(&clause),
            RequiredProofStatus::KernelVerified,
        )],
        Vec::new(),
    );
    let mut tiny_limits = limits();
    tiny_limits.max_imported_facts = 0;

    let error = check_imported_facts(input(&certificate, None, tiny_limits))
        .expect_err("count limit should fire before missing context");

    assert_eq!(error.detail(), RejectionDetail::ResourceExhaustion);
    assert_eq!(
        error.location().field_path,
        Some("imported_fact_context.imported_fact_count")
    );
}

fn assert_rejection_record(
    record: &RejectionRecord,
    category: RejectionCategory,
    detail: RejectionDetail,
    location: RejectionLocation,
) {
    assert_eq!(record.category(), category);
    assert_eq!(record.category().stable_key(), category.stable_key());
    assert_eq!(record.detail(), detail);
    assert_eq!(record.stable_detail_key(), detail.stable_key());
    assert_eq!(record.location(), &location);
}

fn assert_service_rejection(
    result: KernelCheckResult,
    category: RejectionCategory,
    detail: RejectionDetail,
    location: RejectionLocation,
) {
    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert!(result.checked_imports().is_empty());
    assert!(result.checked_substitutions().is_empty());
    assert!(result.checked_resolution_steps().is_empty());
    assert!(result.checked_cluster_steps().is_empty());
    assert!(result.checked_reduction_steps().is_empty());
    assert!(result.checked_derived_facts().is_empty());
    assert!(result.final_goal().is_none());
    assert!(result.used_axioms().is_empty());
    assert_eq!(result.rejections().len(), 1);
    assert_rejection_record(&result.rejections()[0], category, detail, location);
}

fn assert_legacy_audit_result(result: &KernelCheckResult, location: RejectionLocation) {
    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert!(result.final_goal().is_none());
    assert!(result.used_axioms().is_empty());
    assert_eq!(result.rejections().len(), 1);
    assert_rejection_record(
        &result.rejections()[0],
        RejectionCategory::CertificateRejection,
        RejectionDetail::UnsupportedCertificateFormat,
        location,
    );
}

fn legacy_audit_location(namespace: FinalGoalNamespace, id: u32) -> RejectionLocation {
    let location = RejectionLocation::new().with_final_goal();
    match namespace {
        FinalGoalNamespace::GeneratedClause => {
            location.with_clause_ref(crate::rejection::ClauseRef::new(
                crate::rejection::ClauseRefNamespace::GeneratedClause,
                id,
            ))
        }
        FinalGoalNamespace::ResolutionStep => {
            location.with_clause_ref(crate::rejection::ClauseRef::new(
                crate::rejection::ClauseRefNamespace::ResolutionStep,
                id,
            ))
        }
        FinalGoalNamespace::DerivedFact => location.with_derived_fact_id(id),
    }
}

fn service_input<'a>(
    target: &'a TargetVcFingerprint,
    certificate: &'a ParsedCertificate,
    context: &'a ImportedFactContext,
    cluster_context: Option<&'a ClusterTraceContext>,
    requested_cluster_trace_steps: &'a [u32],
    policy: KernelCheckPolicy,
    limits: KernelCheckLimits,
) -> KernelCheckInput<'a> {
    KernelCheckInput {
        target_vc_fingerprint: target,
        certificate,
        imported_fact_context: Some(context),
        substitution_context: None,
        cluster_trace_context: cluster_context,
        requested_cluster_trace_steps,
        policy: legacy_audit_policy(policy),
        limits,
    }
}

fn service_input_with_substitutions<'a>(
    target: &'a TargetVcFingerprint,
    certificate: &'a ParsedCertificate,
    context: &'a ImportedFactContext,
    substitution_context: Option<&'a SubstitutionContext>,
    cluster_context: Option<&'a ClusterTraceContext>,
    requested_cluster_trace_steps: &'a [u32],
    limits: KernelCheckLimits,
) -> KernelCheckInput<'a> {
    KernelCheckInput {
        target_vc_fingerprint: target,
        certificate,
        imported_fact_context: Some(context),
        substitution_context,
        cluster_trace_context: cluster_context,
        requested_cluster_trace_steps,
        policy: legacy_audit_policy(KernelCheckPolicy::default()),
        limits,
    }
}

fn legacy_audit_policy(mut policy: KernelCheckPolicy) -> KernelCheckPolicy {
    policy.allow_legacy_certificate_audit = true;
    policy
}

fn service_limits() -> KernelCheckLimits {
    KernelCheckLimits {
        imported_facts: limits(),
        substitutions: SubstitutionReplayLimits {
            max_substitutions: 8,
            max_binder_context_bytes: 128,
            max_binder_frames: 8,
            max_freshness_witnesses: 8,
            max_free_variable_constraints: 8,
            max_term_encoding_bytes: 4096,
            max_term_recursion_depth: 16,
            max_alpha_renames: 8,
            max_payload_replacements: 8,
            max_term_path_segments: 8,
            max_avoided_variables: 8,
            max_capture_set_variables: 8,
        },
        resolution: ResolutionReplayLimits {
            max_checked_steps: 8,
            max_parent_literals: 8,
            max_resolvent_literals: 8,
            max_resolvent_canonical_bytes: 4096,
            max_term_encoding_bytes: 4096,
            max_term_recursion_depth: 16,
        },
        cluster_trace: cluster_limits(),
        max_pipeline_steps: 16,
        max_derived_facts: 8,
        max_report_records: 64,
    }
}

fn resolution_service_fixture(target_digest: Vec<u8>) -> (ParsedCertificate, ImportedFactContext) {
    resolution_service_fixture_with_final(
        target_digest,
        FinalGoalRef {
            namespace: FinalGoalNamespace::GeneratedClause,
            id: 3,
        },
    )
}

fn resolution_service_fixture_with_final(
    target_digest: Vec<u8>,
    final_goal: FinalGoalRef,
) -> (ParsedCertificate, ImportedFactContext) {
    resolution_service_fixture_with_status_and_final(
        target_digest,
        RequiredProofStatus::KernelVerified,
        AcceptedProofStatus::KernelVerified,
        final_goal,
    )
}

fn resolution_service_fixture_with_status(
    target_digest: Vec<u8>,
    required_proof_status: RequiredProofStatus,
    accepted_proof_status: AcceptedProofStatus,
) -> (ParsedCertificate, ImportedFactContext) {
    resolution_service_fixture_with_status_and_final(
        target_digest,
        required_proof_status,
        accepted_proof_status,
        FinalGoalRef {
            namespace: FinalGoalNamespace::GeneratedClause,
            id: 3,
        },
    )
}

fn resolution_service_fixture_with_status_and_final(
    target_digest: Vec<u8>,
    required_proof_status: RequiredProofStatus,
    accepted_proof_status: AcceptedProofStatus,
    final_goal: FinalGoalRef,
) -> (ParsedCertificate, ImportedFactContext) {
    let imported_clause = ordinary(vec![neg_p()]);
    let imported = imported_ref(
        1,
        b"pkg",
        b"mod",
        b"axiom",
        clause_fingerprint(&imported_clause),
        required_proof_status,
    );
    let certificate = ParsedCertificate::new_for_kernel_tests(ParsedCertificateTestParts {
        schema_version: 1,
        encoding_version: 1,
        kernel_profile: KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        target_vc: Fingerprint::new(1, target_digest.clone()),
        symbol_manifest: vec![SymbolManifestEntry { symbol: p_symbol() }],
        variable_manifest: vec![
            VariableManifestEntry {
                variable_id: VariableId(1),
            },
            VariableManifestEntry {
                variable_id: VariableId(2),
            },
        ],
        imported_axioms: vec![imported.clone()],
        imported_theorems: Vec::new(),
        generated_clauses: vec![
            generated_clause(2, ordinary(vec![pos_p()])),
            generated_clause(3, empty_clause()),
        ],
        substitutions: Vec::new(),
        resolution_trace: vec![resolution_step(
            1,
            clause_ref(ClauseRefNamespace::ImportedAxiom, 1),
            clause_ref(ClauseRefNamespace::GeneratedClause, 2),
            neg_p(),
            clause_ref(ClauseRefNamespace::GeneratedClause, 3),
        )],
        derived_facts: Vec::new(),
        final_goal,
        canonical_hash_input: {
            let mut bytes = target_digest;
            bytes.extend_from_slice(&final_goal.id.to_be_bytes());
            bytes
        },
    });
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![evidence(&imported, accepted_proof_status, imported_clause)],
        Vec::new(),
        context_limits(),
    )
    .expect("imported fact context");
    (certificate, context)
}

fn legacy_tautology_service_fixture(
    target_digest: Vec<u8>,
) -> (ParsedCertificate, ImportedFactContext) {
    let imported_clause = ordinary_with_context(vec![neg_p(), pos_q()], marker_context());
    let imported = imported_ref(
        1,
        b"pkg",
        b"mod",
        b"tautology",
        clause_fingerprint(&imported_clause),
        RequiredProofStatus::KernelVerified,
    );
    let certificate = ParsedCertificate::new_for_kernel_tests(ParsedCertificateTestParts {
        schema_version: 1,
        encoding_version: 1,
        kernel_profile: KernelProfileRecord::v1(1, ClauseTautologyPolicy::Marker),
        target_vc: Fingerprint::new(1, target_digest.clone()),
        symbol_manifest: vec![
            SymbolManifestEntry { symbol: p_symbol() },
            SymbolManifestEntry { symbol: q_symbol() },
        ],
        variable_manifest: vec![VariableManifestEntry {
            variable_id: VariableId(1),
        }],
        imported_axioms: vec![imported.clone()],
        imported_theorems: Vec::new(),
        generated_clauses: vec![
            generated_clause(
                2,
                ordinary_with_context(vec![neg_q(), pos_p()], marker_context()),
            ),
            generated_clause(3, tautology_marker_clause()),
        ],
        substitutions: Vec::new(),
        resolution_trace: vec![resolution_step(
            1,
            clause_ref(ClauseRefNamespace::ImportedAxiom, 1),
            clause_ref(ClauseRefNamespace::GeneratedClause, 2),
            neg_p(),
            clause_ref(ClauseRefNamespace::GeneratedClause, 3),
        )],
        derived_facts: Vec::new(),
        final_goal: FinalGoalRef {
            namespace: FinalGoalNamespace::GeneratedClause,
            id: 3,
        },
        canonical_hash_input: {
            let mut bytes = target_digest;
            bytes.extend_from_slice(&3u32.to_be_bytes());
            bytes
        },
    });
    let context = ImportedFactContext::new(
        Some(vec![1]),
        vec![evidence(
            &imported,
            AcceptedProofStatus::KernelVerified,
            imported_clause,
        )],
        Vec::new(),
        context_limits(),
    )
    .expect("imported fact context");
    (certificate, context)
}

fn generated_clause(clause_id: u32, clause: Clause) -> GeneratedClause {
    GeneratedClause { clause_id, clause }
}

fn resolution_step(
    step_id: u32,
    parent_a: ClauseRef,
    parent_b: ClauseRef,
    pivot_literal: Literal,
    generated_clause: ClauseRef,
) -> ResolutionStep {
    ResolutionStep {
        step_id,
        parent_a,
        parent_b,
        pivot_literal,
        generated_clause,
    }
}

fn simple_substitution(
    substitution_id: u32,
    source_term: Term,
    target_term: Term,
) -> SubstitutionEntry {
    SubstitutionEntry {
        substitution_id,
        source_term,
        target_term,
        binder_context_encoding: service_binder_context(Vec::new(), vec![1, 2], Vec::new()),
        freshness_witness_refs: Vec::new(),
        free_variable_constraint_refs: Vec::new(),
    }
}

fn simple_substitution_context(substitution_id: u32, actual_term: Term) -> SubstitutionContext {
    SubstitutionContext::new(
        Some(vec![7]),
        vec![SubstitutionPayloadEntry::new(
            substitution_id,
            SubstitutionPayload::new(
                substitution_id,
                1,
                TermPath::root(),
                vec![Replacement::new(VariableId(1), actual_term, 1)],
            ),
        )],
        Vec::new(),
        Vec::new(),
    )
    .expect("valid substitution context")
}

fn service_binder_context(
    frames: Vec<(u32, u32, u32, u8)>,
    free_variables: Vec<u32>,
    schematic_variables: Vec<u32>,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&1u16.to_be_bytes());
    bytes.extend_from_slice(&(frames.len() as u32).to_be_bytes());
    for (binder_id, canonical_index, variable_id, binder_role) in frames {
        bytes.extend_from_slice(&binder_id.to_be_bytes());
        bytes.extend_from_slice(&canonical_index.to_be_bytes());
        bytes.extend_from_slice(&variable_id.to_be_bytes());
        bytes.push(binder_role);
    }
    bytes.extend_from_slice(&(free_variables.len() as u32).to_be_bytes());
    for variable in free_variables {
        bytes.extend_from_slice(&variable.to_be_bytes());
    }
    bytes.extend_from_slice(&(schematic_variables.len() as u32).to_be_bytes());
    for variable in schematic_variables {
        bytes.extend_from_slice(&variable.to_be_bytes());
    }
    bytes
}

const fn clause_ref(namespace: ClauseRefNamespace, id: u32) -> ClauseRef {
    ClauseRef { namespace, id }
}

fn cluster_input<'a>(
    target: &'a TargetVcFingerprint,
    facts: &'a CheckedFactContext,
    context: Option<&'a ClusterTraceContext>,
) -> ClusterTraceReplayInput<'a> {
    let requested = context.map_or_else(
        || vec![1],
        |context| {
            let mut ids: Vec<u32> = context
                .cluster_steps()
                .iter()
                .map(|step| step.cluster_trace_step_id)
                .chain(
                    context
                        .reduction_steps()
                        .iter()
                        .map(|step| step.reduction_step_id),
                )
                .collect();
            ids.sort_unstable();
            ids
        },
    );
    cluster_input_with_requested(target, facts, context, requested)
}

fn cluster_input_with_requested<'a>(
    target: &'a TargetVcFingerprint,
    facts: &'a CheckedFactContext,
    context: Option<&'a ClusterTraceContext>,
    requested: Vec<u32>,
) -> ClusterTraceReplayInput<'a> {
    ClusterTraceReplayInput {
        target_vc_fingerprint: target,
        checked_fact_context: facts,
        cluster_trace_context: context,
        requested_trace_steps: Box::leak(requested.into_boxed_slice()),
        limits: cluster_limits(),
    }
}

fn checked_fact_context() -> CheckedFactContext {
    CheckedFactContext::new(vec![1], Vec::new(), vec![7]).expect("checked fact context")
}

fn cluster_limits() -> ClusterTraceReplayLimits {
    ClusterTraceReplayLimits {
        max_cluster_steps: 8,
        max_reduction_steps: 8,
        max_trace_steps: 16,
        max_guard_evidence: 8,
        max_reduction_bindings: 8,
        max_trace_field_bytes: 4096,
        max_commitment_bytes: 8192,
    }
}

fn cluster_step(cluster_trace_step_id: u32, dependency: CheckedFactRef) -> ClusterStepEvidence {
    ClusterStepEvidence {
        cluster_trace_step_id,
        source_type: b"type:T".to_vec(),
        applied_cluster: b"cluster:C".to_vec(),
        generated_attribute: b"attr:A".to_vec(),
        generated_type: b"type:T+A".to_vec(),
        dependency,
        generated_fact_fingerprint: Vec::new(),
    }
}

fn reduction_step(
    reduction_step_id: u32,
    guard_dependency: CheckedFactRef,
) -> ReductionStepEvidence {
    ReductionStepEvidence {
        reduction_step_id,
        applied_reduction: b"reduction:R".to_vec(),
        rule_fqn: b"pkg::module::R".to_vec(),
        enclosing_term_before: b"term:before:R".to_vec(),
        redex_path: b"path:0.1".to_vec(),
        source_redex: b"term:redex:R".to_vec(),
        target_term: b"term:target:R".to_vec(),
        substitution: vec![ReductionBindingEvidence {
            variable: b"x".to_vec(),
            replacement: b"replacement".to_vec(),
        }],
        required_guard_ids: vec![1],
        discharged_guards: vec![GuardEvidence {
            guard_id: 1,
            source_fact_ref: guard_dependency,
            checked_dependency_ref: CheckedFactRef::ImportedAxiom(1),
        }],
        rule_view: b"fingerprint:R".to_vec(),
        selection_key: b"selection:R".to_vec(),
        strategy_audit_key: Vec::new(),
        result_fingerprint: Vec::new(),
    }
}

fn input<'a>(
    certificate: &'a ParsedCertificate,
    context: Option<&'a ImportedFactContext>,
    limits: ImportedFactCheckLimits,
) -> ImportedFactCheckInput<'a> {
    input_with_policy(certificate, context, ImportedFactPolicy::default(), limits)
}

fn input_with_policy<'a>(
    certificate: &'a ParsedCertificate,
    context: Option<&'a ImportedFactContext>,
    policy: ImportedFactPolicy,
    limits: ImportedFactCheckLimits,
) -> ImportedFactCheckInput<'a> {
    ImportedFactCheckInput {
        target_vc_fingerprint: Box::leak(Box::new(TargetVcFingerprint::new(1, vec![42]))),
        certificate,
        imported_fact_context: context,
        policy,
        limits,
    }
}

fn make_certificate(
    imported_axioms: Vec<ImportedFactRef>,
    imported_theorems: Vec<ImportedFactRef>,
) -> ParsedCertificate {
    ParsedCertificate::new_for_kernel_tests(ParsedCertificateTestParts {
        schema_version: 1,
        encoding_version: 1,
        kernel_profile: KernelProfileRecord::v1(1, ClauseTautologyPolicy::Reject),
        target_vc: Fingerprint::new(1, vec![42]),
        symbol_manifest: vec![
            SymbolManifestEntry { symbol: p_symbol() },
            SymbolManifestEntry { symbol: q_symbol() },
        ],
        variable_manifest: vec![VariableManifestEntry {
            variable_id: VariableId(1),
        }],
        imported_axioms,
        imported_theorems,
        generated_clauses: Vec::new(),
        substitutions: Vec::new(),
        resolution_trace: Vec::new(),
        derived_facts: Vec::new(),
        final_goal: FinalGoalRef {
            namespace: FinalGoalNamespace::GeneratedClause,
            id: 0,
        },
        canonical_hash_input: vec![1, 2, 3],
    })
}

fn imported_ref(
    imported_fact_id: u32,
    package_id: &[u8],
    module_path: &[u8],
    exported_item_id: &[u8],
    statement_fingerprint: Fingerprint,
    required_proof_status: RequiredProofStatus,
) -> ImportedFactRef {
    ImportedFactRef {
        imported_fact_id,
        package_id: package_id.to_vec(),
        module_path: module_path.to_vec(),
        exported_item_id: exported_item_id.to_vec(),
        statement_fingerprint,
        required_proof_status,
    }
}

fn evidence(
    imported: &ImportedFactRef,
    accepted_proof_status: AcceptedProofStatus,
    clause: Clause,
) -> ImportedFactEvidence {
    let normalized_clause_fingerprint = clause_fingerprint(&clause);
    ImportedFactEvidence {
        imported_fact_id: imported.imported_fact_id,
        package_id: imported.package_id.clone(),
        module_path: imported.module_path.clone(),
        exported_item_id: imported.exported_item_id.clone(),
        statement_fingerprint: imported.statement_fingerprint.clone(),
        accepted_proof_status,
        normalized_clause_fingerprint,
        clause,
    }
}

fn clause_fingerprint(clause: &Clause) -> Fingerprint {
    Fingerprint::new(
        1,
        clause
            .canonical_hash_input()
            .expect("test clause canonical hash input"),
    )
}

fn limits() -> ImportedFactCheckLimits {
    ImportedFactCheckLimits {
        max_imported_facts: 16,
        max_imported_context_entries: 16,
        max_imported_clause_literals: 8,
        max_imported_clause_canonical_bytes: 4096,
        max_imported_term_encoding_bytes: 4096,
        max_imported_term_recursion_depth: 16,
    }
}

fn context_limits() -> ImportedFactContextLimits {
    ImportedFactContextLimits {
        max_imported_context_entries: 16,
        max_context_identity_entries: 16,
    }
}

fn ordinary(literals: Vec<Literal>) -> Clause {
    ordinary_with_context(literals, base_context())
}

fn ordinary_with_context(literals: Vec<Literal>, context: ClauseValidationContext) -> Clause {
    Clause::from_canonical_parts(ClauseForm::Ordinary, literals, &context).expect("ordinary clause")
}

fn empty_clause() -> Clause {
    Clause::from_canonical_parts(ClauseForm::Empty, Vec::new(), &base_context())
        .expect("empty clause")
}

fn tautology_marker_clause() -> Clause {
    Clause::from_canonical_parts(ClauseForm::Tautology, Vec::new(), &marker_context())
        .expect("tautology marker")
}

fn var(id: u32) -> Term {
    Term::Variable(VariableId(id))
}

fn wrong_profile_clause() -> Clause {
    let context = ClauseValidationContext::new(ClauseProfile::new(1, 2, TautologyPolicy::Reject))
        .with_known_symbol(p_symbol())
        .with_canonical_variable(VariableId(1))
        .with_limits(8, 4096)
        .with_max_term_recursion_depth(16);
    Clause::from_canonical_parts(ClauseForm::Ordinary, vec![neg_p()], &context)
        .expect("wrong profile clause")
}

fn unknown_symbol_clause() -> Clause {
    let symbol = SymbolKey {
        kind: SymbolKind::Predicate,
        id: SymbolId(99),
    };
    let context = ClauseValidationContext::new(ClauseProfile::new(1, 1, TautologyPolicy::Reject))
        .with_known_symbol(symbol)
        .with_canonical_variable(VariableId(1))
        .with_limits(8, 4096)
        .with_max_term_recursion_depth(16);
    Clause::from_canonical_parts(
        ClauseForm::Ordinary,
        vec![Literal::new(
            Polarity::Negative,
            Atom::new(symbol, Vec::new()),
        )],
        &context,
    )
    .expect("unknown symbol clause")
}

fn variable_clause(variable_id: u32) -> Clause {
    let context = ClauseValidationContext::new(ClauseProfile::new(1, 1, TautologyPolicy::Reject))
        .with_known_symbol(p_symbol())
        .with_canonical_variable(VariableId(variable_id))
        .with_limits(8, 4096)
        .with_max_term_recursion_depth(16);
    Clause::from_canonical_parts(
        ClauseForm::Ordinary,
        vec![Literal::new(
            Polarity::Negative,
            Atom::new(p_symbol(), vec![Term::Variable(VariableId(variable_id))]),
        )],
        &context,
    )
    .expect("variable clause")
}

fn base_context() -> ClauseValidationContext {
    context_with_policy(TautologyPolicy::Reject)
}

fn marker_context() -> ClauseValidationContext {
    context_with_policy(TautologyPolicy::Marker)
}

fn context_with_policy(tautology_policy: TautologyPolicy) -> ClauseValidationContext {
    ClauseValidationContext::new(ClauseProfile::new(1, 1, tautology_policy))
        .with_known_symbol(p_symbol())
        .with_known_symbol(q_symbol())
        .with_canonical_variable(VariableId(1))
        .with_limits(8, 4096)
        .with_max_term_recursion_depth(16)
}

fn neg_p() -> Literal {
    Literal::new(Polarity::Negative, Atom::new(p_symbol(), Vec::new()))
}

fn pos_p() -> Literal {
    Literal::new(Polarity::Positive, Atom::new(p_symbol(), Vec::new()))
}

fn pos_q() -> Literal {
    Literal::new(Polarity::Positive, Atom::new(q_symbol(), Vec::new()))
}

fn neg_q() -> Literal {
    Literal::new(Polarity::Negative, Atom::new(q_symbol(), Vec::new()))
}

const fn p_symbol() -> SymbolKey {
    SymbolKey {
        kind: SymbolKind::Predicate,
        id: SymbolId(1),
    }
}

const fn q_symbol() -> SymbolKey {
    SymbolKey {
        kind: SymbolKind::Predicate,
        id: SymbolId(2),
    }
}

const FORMULA_EVIDENCE_DOMAIN_SEPARATOR: &[u8] = b"MIZAR_KERNEL_EVIDENCE\0";

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

fn formula_atom_with_variable(symbol_id: u32, variable_id: u32) -> Formula {
    Formula::Atom(Atom::with_arity(
        SymbolKey {
            kind: SymbolKind::Predicate,
            id: SymbolId(symbol_id),
        },
        1,
        vec![Term::Variable(VariableId(variable_id))],
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

fn parsed_formula_evidence_with_substitutions(
    target: &Fingerprint,
    variables: Vec<Vec<u8>>,
    formulas: Vec<Vec<u8>>,
    substitutions: Vec<Vec<u8>>,
    goal: Vec<u8>,
) -> ParsedKernelEvidence {
    let bytes = formula_evidence_bytes_with_parts(target, variables, formulas, substitutions, goal);
    parse_formula_evidence(
        &bytes,
        &FormulaEvidenceParseContext::v1(target.clone(), formula_profile()),
    )
    .expect("formula evidence with substitutions parses")
}

fn parsed_formula_evidence(
    target: &Fingerprint,
    formulas: Vec<Vec<u8>>,
    goal: Vec<u8>,
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
    };
    formula_evidence_context_entries(axioms, theorems)
}

fn formula_evidence_context_entries(
    axioms: Vec<FormulaImportedFactEvidence>,
    theorems: Vec<FormulaImportedFactEvidence>,
) -> FormulaEvidenceContext {
    FormulaEvidenceContext::new(
        Some(vec![1]),
        axioms,
        theorems,
        ImportedFactContextLimits::default(),
    )
    .expect("formula evidence context")
}

fn formula_evidence_context_with_identity(
    target: &TargetVcFingerprint,
    evidence: &ParsedKernelEvidence,
) -> FormulaEvidenceContext {
    formula_evidence_context_with_identity_payload(context_identity_payload(
        target,
        context_identity_entries_for(evidence),
    ))
}

fn formula_evidence_context_with_identity_payload(
    identity: KernelContextIdentityPayload,
) -> FormulaEvidenceContext {
    FormulaEvidenceContext::with_context_identity(
        Some(vec![1]),
        Vec::new(),
        Vec::new(),
        Some(identity),
        ImportedFactContextLimits::default(),
    )
    .expect("formula evidence context with identity")
}

fn context_identity_payload(
    target: &TargetVcFingerprint,
    entries: Vec<KernelContextIdentityEntry>,
) -> KernelContextIdentityPayload {
    let canonical_handoff_hash = Hash::from_bytes([0x42; Hash::BYTE_LEN]);
    let provisional = KernelContextIdentityPayload::new(
        target.clone(),
        canonical_handoff_hash,
        Hash::from_bytes([0; Hash::BYTE_LEN]),
        entries,
    );
    let hash = recompute_context_identity_hash(&provisional);
    KernelContextIdentityPayload::new(
        target.clone(),
        canonical_handoff_hash,
        hash,
        provisional.entries().to_vec(),
    )
}

fn context_identity_entries_for(
    evidence: &ParsedKernelEvidence,
) -> Vec<KernelContextIdentityEntry> {
    evidence
        .formulas()
        .iter()
        .filter_map(|formula| {
            let source = context_identity_source(&formula.source)?;
            Some(KernelContextIdentityEntry::new(
                source,
                formula.formula_id,
                formula.formula_fingerprint.clone(),
                default_producer_ref(source, formula.formula_id),
            ))
        })
        .collect()
}

fn default_producer_ref(
    source: KernelContextIdentitySource,
    formula_id: u32,
) -> KernelFormulaProducerRef {
    match source {
        KernelContextIdentitySource::GeneratedVcFact { vc_fact_id } => {
            KernelFormulaProducerRef::Generated(KernelVcGeneratedFormulaId::new(
                vc_fact_id as usize,
            ))
        }
        KernelContextIdentitySource::LocalHypothesis { .. }
        | KernelContextIdentitySource::CitedPremise { .. } => {
            KernelFormulaProducerRef::Core(CoreFormulaId::new(formula_id as usize))
        }
    }
}

fn assert_imported_projection_rejects(
    target_vc: &TargetVcFingerprint,
    parsed: &crate::formula_evidence::ParsedKernelEvidence,
    fact: FormulaImportedFactEvidence,
) {
    let context = formula_evidence_context(fact, ImportedFactNamespace::ImportedAxiom);
    let result = check_kernel_evidence(evidence_input(target_vc, parsed, Some(&context)));
    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert_eq!(
        result.rejections()[0].detail(),
        RejectionDetail::UnresolvedSymbol
    );
    assert_eq!(
        result.rejections()[0].location().field_path,
        Some("formula.imported_statement_projection")
    );
}

fn formula_imported_fact(
    imported_fact_id: u32,
    formula: &Formula,
    accepted_proof_status: AcceptedProofStatus,
) -> FormulaImportedFactEvidence {
    formula_imported_fact_with_identity(
        imported_fact_id,
        b"pkg",
        b"module",
        b"ITEM",
        formula,
        accepted_proof_status,
    )
}

fn formula_imported_fact_with_identity(
    imported_fact_id: u32,
    package_id: &[u8],
    module_path: &[u8],
    exported_item_id: &[u8],
    formula: &Formula,
    accepted_proof_status: AcceptedProofStatus,
) -> FormulaImportedFactEvidence {
    let formula_fingerprint = formula_fingerprint(formula);
    let statement_fingerprint = imported_statement_fingerprint();
    let statement_projection =
        imported_statement_projection(&statement_fingerprint, &formula_fingerprint);
    FormulaImportedFactEvidence {
        imported_fact_id,
        package_id: package_id.to_vec(),
        module_path: module_path.to_vec(),
        exported_item_id: exported_item_id.to_vec(),
        statement_fingerprint,
        accepted_proof_status,
        statement_projection,
    }
}

fn formula_evidence_bytes(target: &Fingerprint, formulas: Vec<Vec<u8>>, goal: Vec<u8>) -> Vec<u8> {
    formula_evidence_bytes_with_parts(target, Vec::new(), formulas, Vec::new(), goal)
}

fn formula_evidence_bytes_with_parts(
    target: &Fingerprint,
    variables: Vec<Vec<u8>>,
    formulas: Vec<Vec<u8>>,
    substitutions: Vec<Vec<u8>>,
    goal: Vec<u8>,
) -> Vec<u8> {
    let mut provenance = Vec::new();
    for formula in &formulas {
        provenance.push(provenance_item(
            target,
            formula_provenance_id(formula),
            &formula_fingerprint_from_item(formula),
        ));
    }
    for substitution in &substitutions {
        let source_formula_id = substitution_source_formula_id(substitution);
        let source_formula = formulas
            .iter()
            .find(|formula| formula_id_from_item(formula) == source_formula_id)
            .expect("substitution source formula fixture exists");
        provenance.push(provenance_item(
            target,
            substitution_provenance_id(substitution),
            &formula_fingerprint_from_item(source_formula),
        ));
    }
    provenance.push(provenance_item(
        target,
        goal_provenance_id(&goal),
        &goal_fingerprint_from_item(&goal),
    ));
    formula_envelope(
        target,
        vec![
            (
                FormulaEvidenceSectionTag::SymbolManifest,
                formula_symbol_items(),
            ),
            (FormulaEvidenceSectionTag::VariableManifest, variables),
            (FormulaEvidenceSectionTag::Formulas, formulas),
            (FormulaEvidenceSectionTag::Substitutions, substitutions),
            (FormulaEvidenceSectionTag::Provenance, provenance),
            (FormulaEvidenceSectionTag::FinalGoal, vec![goal]),
        ],
    )
}

fn formula_envelope(
    target: &Fingerprint,
    sections: Vec<(FormulaEvidenceSectionTag, Vec<Vec<u8>>)>,
) -> Vec<u8> {
    let mut payloads = Vec::new();
    let mut directory = Vec::new();
    let mut offset = 0u32;
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

    let mut bytes = Vec::from(FORMULA_EVIDENCE_DOMAIN_SEPARATOR);
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
    [1u32, 2u32]
        .into_iter()
        .map(|id| {
            let mut item = Vec::new();
            item.push(symbol_kind_tag(SymbolKind::Predicate));
            put_u32(id, &mut item);
            item
        })
        .collect()
}

fn formula_item(formula_id: u32, provenance_id: u32, formula: &Formula) -> Vec<u8> {
    non_imported_formula_item(
        FormulaSourceClass::LocalHypothesis,
        formula_id,
        provenance_id,
        1,
        formula,
    )
}

fn cited_formula_item(
    formula_id: u32,
    provenance_id: u32,
    local_context_id: u32,
    formula: &Formula,
) -> Vec<u8> {
    non_imported_formula_item(
        FormulaSourceClass::CitedPremise,
        formula_id,
        provenance_id,
        local_context_id,
        formula,
    )
}

fn generated_formula_item(
    formula_id: u32,
    provenance_id: u32,
    vc_fact_id: u32,
    formula: &Formula,
) -> Vec<u8> {
    non_imported_formula_item(
        FormulaSourceClass::GeneratedVcFact,
        formula_id,
        provenance_id,
        vc_fact_id,
        formula,
    )
}

fn non_imported_formula_item(
    source_class: FormulaSourceClass,
    formula_id: u32,
    provenance_id: u32,
    source_id: u32,
    formula: &Formula,
) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(source_class.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_u32(source_id, &mut item);
    put_formula(formula, &mut item);
    item
}

fn policy_bounded_builtin_formula_item(
    formula_id: u32,
    provenance_id: u32,
    built_in_id: &[u8],
    formula: &Formula,
) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(FormulaSourceClass::PolicyBoundedBuiltin.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_bytes(built_in_id, &mut item);
    put_formula(formula, &mut item);
    item
}

fn imported_formula_item(formula_id: u32, provenance_id: u32, formula: &Formula) -> Vec<u8> {
    imported_formula_item_with_required_status(
        formula_id,
        provenance_id,
        formula,
        RequiredProofStatus::KernelVerified,
    )
}

fn imported_formula_item_with_required_status(
    formula_id: u32,
    provenance_id: u32,
    formula: &Formula,
    required_status: RequiredProofStatus,
) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let statement_fingerprint = imported_statement_fingerprint();
    let statement_projection = imported_statement_projection(&statement_fingerprint, &fingerprint);
    let mut item = Vec::new();
    put_u32(formula_id, &mut item);
    item.push(FormulaSourceClass::AcceptedImportedAxiom.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_bytes(b"pkg", &mut item);
    put_bytes(b"module", &mut item);
    put_bytes(b"ITEM", &mut item);
    put_fingerprint(&statement_fingerprint, &mut item);
    item.push(required_status_tag(required_status));
    put_fingerprint(&statement_projection.statement_fingerprint, &mut item);
    put_fingerprint(&statement_projection.formula_fingerprint, &mut item);
    put_bytes(&statement_projection.payload, &mut item);
    put_formula(formula, &mut item);
    item
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

fn variable_item(variable_id: u32) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(variable_id, &mut item);
    item
}

fn formula_substitution_item(
    substitution_id: u32,
    source_formula_id: u32,
    provenance_id: u32,
    formal_variable_id: u32,
    actual_term: &Term,
) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(substitution_id, &mut item);
    put_u32(source_formula_id, &mut item);
    put_u32(provenance_id, &mut item);
    put_bytes(&empty_binder_context(), &mut item);
    put_u32(substitution_id, &mut item);
    item.push(1);
    put_term_path(&TermPath::root(), &mut item);
    put_u32(1, &mut item);
    put_u32(formal_variable_id, &mut item);
    put_term(actual_term, &mut item);
    item.push(1);
    put_u32(0, &mut item);
    put_u32(0, &mut item);
    item
}

fn goal_item(provenance_id: u32, formula: &Formula) -> Vec<u8> {
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
) -> Vec<u8> {
    let fingerprint = formula_fingerprint(formula);
    let mut item = Vec::new();
    item.push(polarity.tag());
    put_fingerprint(&fingerprint, &mut item);
    put_u32(provenance_id, &mut item);
    put_formula(formula, &mut item);
    item
}

fn assert_goal_polarity_mismatch(result: &KernelCheckResult) {
    assert_eq!(result.status(), KernelCheckStatus::Rejected);
    assert_eq!(result.rejections().len(), 1);
    assert_eq!(
        result.rejections()[0].category(),
        RejectionCategory::CertificateRejection
    );
    assert_eq!(
        result.rejections()[0].detail(),
        RejectionDetail::ContextMismatch
    );
    assert!(result.rejections()[0].location().final_goal);
    assert_eq!(
        result.rejections()[0].location().field_path,
        Some("final_goal.polarity")
    );
}

fn provenance_item(target: &Fingerprint, provenance_id: u32, fingerprint: &Fingerprint) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(provenance_id, &mut item);
    put_fingerprint(target, &mut item);
    put_fingerprint(fingerprint, &mut item);
    put_bytes(b"producer-payload", &mut item);
    item
}

fn formula_provenance_id(item: &[u8]) -> u32 {
    u32::from_be_bytes([
        item[10 + fingerprint_len(item)],
        item[11 + fingerprint_len(item)],
        item[12 + fingerprint_len(item)],
        item[13 + fingerprint_len(item)],
    ])
}

fn formula_id_from_item(item: &[u8]) -> u32 {
    u32::from_be_bytes([item[0], item[1], item[2], item[3]])
}

fn substitution_source_formula_id(item: &[u8]) -> u32 {
    u32::from_be_bytes([item[4], item[5], item[6], item[7]])
}

fn substitution_provenance_id(item: &[u8]) -> u32 {
    u32::from_be_bytes([item[8], item[9], item[10], item[11]])
}

fn goal_provenance_id(item: &[u8]) -> u32 {
    let start = 1 + fingerprint_item_len(&item[1..]);
    u32::from_be_bytes([
        item[start],
        item[start + 1],
        item[start + 2],
        item[start + 3],
    ])
}

fn goal_fingerprint_from_item(item: &[u8]) -> Fingerprint {
    fingerprint_from_slice(&item[1..])
}

fn formula_fingerprint_from_item(item: &[u8]) -> Fingerprint {
    fingerprint_from_slice(&item[5..])
}

fn fingerprint_len(item: &[u8]) -> usize {
    u32::from_be_bytes([item[6], item[7], item[8], item[9]]) as usize
}

fn fingerprint_from_slice(bytes: &[u8]) -> Fingerprint {
    let algorithm_id = bytes[0];
    let len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
    Fingerprint::new(algorithm_id, bytes[5..5 + len].to_vec())
}

fn fingerprint_item_len(bytes: &[u8]) -> usize {
    5 + u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize
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
    }
}

fn put_atom(atom: &Atom, bytes: &mut Vec<u8>) {
    bytes.push(symbol_kind_tag(atom.symbol.kind));
    put_u32(atom.symbol.id.0, bytes);
    put_u32(atom.arity, bytes);
    put_u32(atom.arguments.len() as u32, bytes);
    for argument in &atom.arguments {
        put_term(argument, bytes);
    }
}

fn put_term(term: &Term, bytes: &mut Vec<u8>) {
    match term {
        Term::Variable(variable) => {
            bytes.push(1);
            put_u32(variable.0, bytes);
        }
        Term::Application { symbol, arguments } => {
            bytes.push(2);
            bytes.push(symbol_kind_tag(symbol.kind));
            put_u32(symbol.id.0, bytes);
            put_u32(arguments.len() as u32, bytes);
            for argument in arguments {
                put_term(argument, bytes);
            }
        }
        Term::BinderNormalized { binder_id, body } => {
            bytes.push(3);
            put_u32(*binder_id, bytes);
            put_term(body, bytes);
        }
        Term::Malformed => panic!("malformed term fixture"),
    }
}

fn empty_binder_context() -> Vec<u8> {
    let mut bytes = Vec::new();
    put_u16(1, &mut bytes);
    put_u32(0, &mut bytes);
    put_u32(0, &mut bytes);
    put_u32(0, &mut bytes);
    bytes
}

fn put_term_path(path: &TermPath, bytes: &mut Vec<u8>) {
    put_u32(path.segments.len() as u32, bytes);
    for segment in &path.segments {
        bytes.push(segment.edge_kind);
        put_u32(segment.child_index, bytes);
    }
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
    }
}

fn required_status_tag(status: RequiredProofStatus) -> u8 {
    match status {
        RequiredProofStatus::KernelVerified => 1,
        RequiredProofStatus::DischargedBuiltin => 2,
        RequiredProofStatus::ExternallyAttestedPolicyPermitted => 3,
    }
}
