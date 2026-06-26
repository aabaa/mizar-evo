use super::*;

fn target(byte: u8) -> TargetVcFingerprint {
    TargetVcFingerprint::new(1, vec![byte])
}

fn record(
    target_byte: u8,
    category: RejectionCategory,
    detail: RejectionDetail,
    location: RejectionLocation,
) -> RejectionRecord {
    RejectionRecord::new(target(target_byte), category, detail, location)
        .expect("valid rejection record")
}

#[test]
fn stable_keys_and_category_detail_pairs_are_explicit() {
    let details = [
        (
            RejectionDetail::UnsupportedCertificateFormat,
            "unsupported_certificate_format",
            RejectionCategory::CertificateRejection,
        ),
        (
            RejectionDetail::ContextMismatch,
            "context_mismatch",
            RejectionCategory::CertificateRejection,
        ),
        (
            RejectionDetail::MalformedCertificate,
            "malformed_certificate",
            RejectionCategory::CertificateRejection,
        ),
        (
            RejectionDetail::MalformedWitnessData,
            "malformed_witness_data",
            RejectionCategory::CertificateRejection,
        ),
        (
            RejectionDetail::MissingProvenance,
            "missing_provenance",
            RejectionCategory::KernelRejection,
        ),
        (
            RejectionDetail::ResourceExhaustion,
            "resource_exhaustion",
            RejectionCategory::CertificateRejection,
        ),
        (
            RejectionDetail::InvalidSubstitution,
            "invalid_substitution",
            RejectionCategory::KernelRejection,
        ),
        (
            RejectionDetail::InvalidSatProof,
            "invalid_sat_proof",
            RejectionCategory::KernelRejection,
        ),
        (
            RejectionDetail::InvalidSatRefutation,
            "invalid_sat_refutation",
            RejectionCategory::KernelRejection,
        ),
        (
            RejectionDetail::InvalidClusterTrace,
            "invalid_cluster_trace",
            RejectionCategory::KernelRejection,
        ),
        (
            RejectionDetail::UnresolvedSymbol,
            "unresolved_symbol",
            RejectionCategory::KernelRejection,
        ),
        (
            RejectionDetail::Timeout,
            "timeout",
            RejectionCategory::KernelRejection,
        ),
    ];

    assert_eq!(
        RejectionCategory::CertificateRejection.stable_key(),
        "certificate_rejection"
    );
    assert_eq!(
        RejectionCategory::KernelRejection.stable_key(),
        "kernel_rejection"
    );
    for (detail, key, category) in details {
        assert_eq!(detail.stable_key(), key);
        assert!(detail.is_allowed_for(category));
        assert!(!detail.is_acceptance());
    }
    assert!(
        RejectionDetail::ResourceExhaustion.is_allowed_for(RejectionCategory::CertificateRejection)
    );
    assert!(RejectionDetail::ResourceExhaustion.is_allowed_for(RejectionCategory::KernelRejection));
}

#[test]
fn invalid_category_detail_mappings_are_rejected() {
    for detail in [
        RejectionDetail::UnsupportedCertificateFormat,
        RejectionDetail::ContextMismatch,
        RejectionDetail::MalformedCertificate,
        RejectionDetail::MalformedWitnessData,
    ] {
        assert_eq!(
            RejectionRecord::new(
                target(1),
                RejectionCategory::KernelRejection,
                detail,
                RejectionLocation::new()
            ),
            Err(RejectionRecordError::InvalidCategoryDetail {
                category: RejectionCategory::KernelRejection,
                detail,
            })
        );
    }

    for detail in [
        RejectionDetail::MissingProvenance,
        RejectionDetail::InvalidSubstitution,
        RejectionDetail::InvalidSatProof,
        RejectionDetail::InvalidSatRefutation,
        RejectionDetail::InvalidClusterTrace,
        RejectionDetail::UnresolvedSymbol,
        RejectionDetail::Timeout,
    ] {
        assert_eq!(
            RejectionRecord::new(
                target(1),
                RejectionCategory::CertificateRejection,
                detail,
                RejectionLocation::new()
            ),
            Err(RejectionRecordError::InvalidCategoryDetail {
                category: RejectionCategory::CertificateRejection,
                detail,
            })
        );
    }
}

#[test]
fn parser_conversion_preserves_target_fallback_and_location() {
    for (parser_detail, shared_detail) in [
        (
            CertificateRejectionDetail::UnsupportedCertificateFormat,
            RejectionDetail::UnsupportedCertificateFormat,
        ),
        (
            CertificateRejectionDetail::ContextMismatch,
            RejectionDetail::ContextMismatch,
        ),
        (
            CertificateRejectionDetail::MalformedCertificate,
            RejectionDetail::MalformedCertificate,
        ),
        (
            CertificateRejectionDetail::ResourceExhaustion,
            RejectionDetail::ResourceExhaustion,
        ),
    ] {
        let parser_error = CertificateParseError {
            category: FailureCategory::CertificateRejection,
            detail: parser_detail,
            location: CertificateParseLocation {
                byte_offset: 42,
                section_tag: Some(SectionTag::VariableManifest),
                item_index: Some(7),
                field_path: Some("target_vc"),
            },
        };
        let expected_target = target(9);

        let record =
            RejectionRecord::from_certificate_parse_error(expected_target.clone(), parser_error)
                .expect("parser detail maps to shared rejection");

        assert_eq!(record.target_vc_fingerprint(), &expected_target);
        assert_eq!(record.category(), RejectionCategory::CertificateRejection);
        assert_eq!(record.detail(), shared_detail);
        assert_ne!(record.detail(), RejectionDetail::Timeout);
        assert_eq!(record.location().certificate_byte_offset, Some(42));
        assert_eq!(
            record.location().section_tag,
            Some(SectionTag::VariableManifest)
        );
        assert_eq!(record.location().item_index, Some(7));
        assert_eq!(record.location().field_path, Some("target_vc"));
        assert!(!record.is_acceptance());
    }
}

#[test]
fn checker_locations_cover_all_evidence_fields() {
    let location = RejectionLocation::new()
        .with_resolution_step_id(1)
        .with_substitution_id(2)
        .with_clause_ref(ClauseRef::new(ClauseRefNamespace::GeneratedClause, 3))
        .with_imported_fact_id(4)
        .with_cluster_trace_step_id(5)
        .with_reduction_step_id(6)
        .with_derived_fact_id(7)
        .with_final_goal();

    assert_eq!(location.resolution_step_id, Some(1));
    assert_eq!(location.substitution_id, Some(2));
    assert_eq!(
        location.clause_ref,
        Some(ClauseRef::new(ClauseRefNamespace::GeneratedClause, 3))
    );
    assert_eq!(location.imported_fact_id, Some(4));
    assert_eq!(location.cluster_trace_step_id, Some(5));
    assert_eq!(location.reduction_step_id, Some(6));
    assert_eq!(location.derived_fact_id, Some(7));
    assert!(location.final_goal);
}

#[test]
fn checker_owner_mappings_use_expected_details() {
    assert!(
        RejectionRecord::new(
            target(1),
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSatProof,
            RejectionLocation::new().with_resolution_step_id(1),
        )
        .is_ok()
    );
    assert!(
        RejectionRecord::new(
            target(1),
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSatRefutation,
            RejectionLocation::new().with_field_path("sat_encoding"),
        )
        .is_ok()
    );
    assert!(
        RejectionRecord::new(
            target(1),
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSubstitution,
            RejectionLocation::new().with_substitution_id(1),
        )
        .is_ok()
    );
    assert!(
        RejectionRecord::new(
            target(1),
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidClusterTrace,
            RejectionLocation::new().with_cluster_trace_step_id(1),
        )
        .is_ok()
    );
    assert!(
        RejectionRecord::new(
            target(1),
            RejectionCategory::KernelRejection,
            RejectionDetail::UnresolvedSymbol,
            RejectionLocation::new().with_imported_fact_id(1),
        )
        .is_ok()
    );
    assert!(
        RejectionRecord::new(
            target(1),
            RejectionCategory::KernelRejection,
            RejectionDetail::MissingProvenance,
            RejectionLocation::new(),
        )
        .is_ok()
    );
    assert!(
        RejectionRecord::new(
            target(1),
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedWitnessData,
            RejectionLocation::new(),
        )
        .is_ok()
    );
}

#[test]
fn deterministic_ordering_uses_documented_tie_breakers() {
    let mut records = [
        record(
            2,
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedCertificate,
            RejectionLocation::new().with_certificate_byte_offset(1),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSubstitution,
            RejectionLocation::new().with_substitution_id(1),
        ),
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedCertificate,
            RejectionLocation::new()
                .with_certificate_byte_offset(1)
                .with_section_tag(SectionTag::GeneratedClauses)
                .with_item_index(1)
                .with_field_path("b"),
        ),
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::UnsupportedCertificateFormat,
            RejectionLocation::new()
                .with_certificate_byte_offset(1)
                .with_section_tag(SectionTag::GeneratedClauses)
                .with_item_index(1)
                .with_field_path("a"),
        ),
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::ContextMismatch,
            RejectionLocation::new().with_certificate_byte_offset(2),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidClusterTrace,
            RejectionLocation::new().with_cluster_trace_step_id(1),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidClusterTrace,
            RejectionLocation::new().with_reduction_step_id(1),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSatProof,
            RejectionLocation::new().with_derived_fact_id(1),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSatProof,
            RejectionLocation::new().with_final_goal(),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSatProof,
            RejectionLocation::new()
                .with_clause_ref(ClauseRef::new(ClauseRefNamespace::GeneratedClause, 1)),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::UnresolvedSymbol,
            RejectionLocation::new()
                .with_clause_ref(ClauseRef::new(ClauseRefNamespace::ImportedTheorem, 1)),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::UnresolvedSymbol,
            RejectionLocation::new()
                .with_clause_ref(ClauseRef::new(ClauseRefNamespace::ImportedAxiom, 1)),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::UnresolvedSymbol,
            RejectionLocation::new().with_imported_fact_id(1),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSatProof,
            RejectionLocation::new().with_resolution_step_id(1),
        ),
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::InvalidSatProof,
            RejectionLocation::new()
                .with_clause_ref(ClauseRef::new(ClauseRefNamespace::ResolutionStep, 1)),
        ),
    ];

    records.sort();

    assert_eq!(
        records[0].detail(),
        RejectionDetail::UnsupportedCertificateFormat
    );
    assert_eq!(records[1].detail(), RejectionDetail::MalformedCertificate);
    assert_eq!(records[2].detail(), RejectionDetail::ContextMismatch);
    assert_eq!(records[3].detail(), RejectionDetail::UnresolvedSymbol);
    assert_eq!(records[3].location().imported_fact_id, Some(1));
    assert_eq!(
        records[4]
            .location()
            .clause_ref
            .expect("imported axiom")
            .namespace,
        ClauseRefNamespace::ImportedAxiom
    );
    assert_eq!(
        records[5]
            .location()
            .clause_ref
            .expect("imported theorem")
            .namespace,
        ClauseRefNamespace::ImportedTheorem
    );
    assert_eq!(records[6].detail(), RejectionDetail::InvalidSatProof);
    assert_eq!(records[6].location().clause_ref.expect("clause ref").id, 1);
    assert_eq!(records[7].detail(), RejectionDetail::InvalidSatProof);
    assert_eq!(records[7].location().resolution_step_id, Some(1));
    assert_eq!(
        records[8]
            .location()
            .clause_ref
            .expect("resolution ref")
            .namespace,
        ClauseRefNamespace::ResolutionStep
    );
    assert_eq!(records[9].detail(), RejectionDetail::InvalidSubstitution);
    assert_eq!(records[10].detail(), RejectionDetail::InvalidClusterTrace);
    assert_eq!(records[10].location().cluster_trace_step_id, Some(1));
    assert_eq!(records[11].detail(), RejectionDetail::InvalidClusterTrace);
    assert_eq!(records[11].location().reduction_step_id, Some(1));
    assert_eq!(records[12].location().derived_fact_id, Some(1));
    assert!(records[13].location().final_goal);
    assert_eq!(records[14].target_vc_fingerprint(), &target(2));
}

#[test]
fn deterministic_ordering_isolates_section_item_and_detail_ties() {
    let mut category_records = [
        record(
            1,
            RejectionCategory::KernelRejection,
            RejectionDetail::ResourceExhaustion,
            RejectionLocation::new(),
        ),
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::ResourceExhaustion,
            RejectionLocation::new(),
        ),
    ];

    category_records.sort();

    assert_eq!(
        category_records[0].category(),
        RejectionCategory::CertificateRejection
    );
    assert_eq!(
        category_records[1].category(),
        RejectionCategory::KernelRejection
    );

    let mut byte_offset_records = [
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedCertificate,
            RejectionLocation::new()
                .with_certificate_byte_offset(2)
                .with_section_tag(SectionTag::SymbolManifest)
                .with_item_index(1)
                .with_field_path("same"),
        ),
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedCertificate,
            RejectionLocation::new()
                .with_certificate_byte_offset(1)
                .with_section_tag(SectionTag::SymbolManifest)
                .with_item_index(1)
                .with_field_path("same"),
        ),
    ];

    byte_offset_records.sort();

    assert_eq!(
        byte_offset_records[0].location().certificate_byte_offset,
        Some(1)
    );
    assert_eq!(
        byte_offset_records[1].location().certificate_byte_offset,
        Some(2)
    );

    let mut section_records = [
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedCertificate,
            RejectionLocation::new()
                .with_certificate_byte_offset(1)
                .with_section_tag(SectionTag::FinalGoal)
                .with_item_index(1)
                .with_field_path("same"),
        ),
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedCertificate,
            RejectionLocation::new()
                .with_certificate_byte_offset(1)
                .with_section_tag(SectionTag::SymbolManifest)
                .with_item_index(1)
                .with_field_path("same"),
        ),
    ];

    section_records.sort();

    assert_eq!(
        section_records[0].location().section_tag,
        Some(SectionTag::SymbolManifest)
    );
    assert_eq!(
        section_records[1].location().section_tag,
        Some(SectionTag::FinalGoal)
    );

    let mut records = [
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedCertificate,
            RejectionLocation::new()
                .with_certificate_byte_offset(1)
                .with_section_tag(SectionTag::FinalGoal),
        ),
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedCertificate,
            RejectionLocation::new()
                .with_certificate_byte_offset(1)
                .with_section_tag(SectionTag::SymbolManifest)
                .with_item_index(2),
        ),
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::ContextMismatch,
            RejectionLocation::new()
                .with_certificate_byte_offset(1)
                .with_section_tag(SectionTag::SymbolManifest)
                .with_item_index(1),
        ),
        record(
            1,
            RejectionCategory::CertificateRejection,
            RejectionDetail::MalformedCertificate,
            RejectionLocation::new()
                .with_certificate_byte_offset(1)
                .with_section_tag(SectionTag::SymbolManifest)
                .with_item_index(1),
        ),
    ];

    records.sort();

    assert_eq!(records[0].detail(), RejectionDetail::ContextMismatch);
    assert_eq!(records[0].location().item_index, Some(1));
    assert_eq!(records[1].detail(), RejectionDetail::MalformedCertificate);
    assert_eq!(records[1].location().item_index, Some(1));
    assert_eq!(records[2].location().item_index, Some(2));
    assert_eq!(
        records[3].location().section_tag,
        Some(SectionTag::FinalGoal)
    );
}

#[test]
fn target_sort_bytes_use_fixed_width_length() {
    assert_eq!(
        TargetVcFingerprint::new(2, vec![3, 4]).sort_bytes(),
        vec![2, 0, 0, 0, 2, 3, 4]
    );
}

#[test]
fn public_enums_are_forward_compatible() {
    let source = include_str!("../rejection.rs");
    for enum_name in [
        "RejectionCategory",
        "RejectionDetail",
        "ClauseRefNamespace",
        "RejectionRecordError",
    ] {
        assert!(
            source.contains(&format!("#[non_exhaustive]\npub enum {enum_name}")),
            "{enum_name} must be #[non_exhaustive]"
        );
    }
}
