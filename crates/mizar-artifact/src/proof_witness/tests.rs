use super::{
    EvidenceKind, KernelAcceptanceMetadata, ProofStatus, ProofWitnessError,
    ProofWitnessReadOptions, ProofWitnessRef, current_schema_version, proof_witness_ref_json,
    read_proof_witness_ref, write_proof_witness_ref,
};
use crate::{
    registration_summary::{ArtifactHashClass, ArtifactHashRef},
    store::{CanonicalJson, SchemaVersion, SchemaVersionError, canonical_json_string},
};
use mizar_session::Hash;

#[test]
fn proof_witness_ref_round_trips_through_canonical_json() {
    let reference = sample_reference();
    let json = proof_witness_ref_json(&reference).expect("canonical proof witness JSON");
    let bytes = write_proof_witness_ref(&reference).expect("canonical proof witness bytes");

    assert_eq!(bytes, canonical_json_string(&json).into_bytes());
    assert_eq!(
        read_proof_witness_ref(&json, ProofWitnessReadOptions::default())
            .expect("valid proof witness reference"),
        reference
    );
}

#[test]
fn proof_witness_ref_writer_is_deterministic_for_identical_inputs() {
    let reference = sample_reference();
    let first_json = proof_witness_ref_json(&reference).expect("first canonical JSON");
    let first_bytes = write_proof_witness_ref(&reference).expect("first canonical bytes");

    for _ in 0..3 {
        let json = proof_witness_ref_json(&reference).expect("repeated canonical JSON");
        assert_eq!(json, first_json);
        assert_eq!(canonical_json_string(&json).into_bytes(), first_bytes);
        assert_eq!(
            write_proof_witness_ref(&reference).expect("repeated canonical bytes"),
            first_bytes
        );
    }
}

#[test]
fn supplied_witness_artifact_hash_mismatch_is_rejected() {
    let reference = sample_reference();
    let json = proof_witness_ref_json(&reference).expect("canonical JSON");
    assert!(
        read_proof_witness_ref(
            &json,
            ProofWitnessReadOptions {
                expected_witness_artifact_hash: Some(&reference.witness_artifact_hash),
                ..ProofWitnessReadOptions::default()
            },
        )
        .is_ok()
    );

    let wrong_hash = hash_ref(
        ArtifactHashClass::Artifact,
        "mizar-kernel/formula-evidence-witness",
        88,
    );
    assert!(matches!(
        read_proof_witness_ref(
            &json,
            ProofWitnessReadOptions {
                expected_witness_artifact_hash: Some(&wrong_hash),
                ..ProofWitnessReadOptions::default()
            },
        ),
        Err(ProofWitnessError::WitnessArtifactHashMismatch { .. })
    ));
}

#[test]
fn incompatible_version_reads_fail_cleanly() {
    let reference = sample_reference();
    let mut json = proof_witness_ref_json(&reference).expect("canonical JSON");
    replace_object_field(&mut json, "schema_version", CanonicalJson::string("3.0"));

    assert!(matches!(
        read_proof_witness_ref(
            &json,
            ProofWitnessReadOptions {
                artifact_path: Some("build/proof-witness-ref.json"),
                ..ProofWitnessReadOptions::default()
            },
        ),
        Err(ProofWitnessError::SchemaVersion(
            SchemaVersionError::MajorMismatch { .. }
        ))
    ));

    let mut missing = proof_witness_ref_json(&reference).expect("canonical JSON");
    remove_object_field(&mut missing, "schema_version");
    assert!(matches!(
        read_proof_witness_ref(&missing, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::SchemaVersion(
            SchemaVersionError::Missing { .. }
        ))
    ));

    let mut malformed = proof_witness_ref_json(&reference).expect("canonical JSON");
    replace_object_field(
        &mut malformed,
        "schema_version",
        CanonicalJson::string("bad"),
    );
    assert!(matches!(
        read_proof_witness_ref(&malformed, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::SchemaVersion(
            SchemaVersionError::Malformed { .. }
        ))
    ));

    let mut legacy_v1 = proof_witness_ref_json(&reference).expect("canonical JSON");
    replace_object_field(
        &mut legacy_v1,
        "schema_version",
        CanonicalJson::string("1.0"),
    );
    replace_nested_object_field(
        &mut legacy_v1,
        "kernel_acceptance",
        "certificate_format",
        CanonicalJson::string("tptp-tstp/v1"),
    );
    replace_nested_object_field(
        &mut legacy_v1,
        "kernel_acceptance",
        "used_axioms_hash",
        hash_ref_json(ArtifactHashClass::Diagnostic, "mizar-proof/used-axioms", 55),
    );
    assert!(matches!(
        read_proof_witness_ref(&legacy_v1, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::SchemaVersion(
            SchemaVersionError::MajorMismatch { .. }
        ))
    ));

    let mut newer_minor = proof_witness_ref_json(&reference).expect("canonical JSON");
    replace_object_field(
        &mut newer_minor,
        "schema_version",
        CanonicalJson::string("2.1"),
    );
    assert!(matches!(
        read_proof_witness_ref(&newer_minor, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::SchemaVersion(
            SchemaVersionError::MinorTooNew { .. }
        ))
    ));
}

#[test]
fn reader_rejects_invalid_hash_domains_and_digests() {
    let reference = sample_reference();
    let json = proof_witness_ref_json(&reference).expect("canonical JSON");
    let digest = "11".repeat(Hash::BYTE_LEN);

    for bad_hash in [
        format!("mizar-artifact/other-framed-hash/v1:interface:mizar-artifact/proof:1.0:{digest}"),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:artifact:mizar-artifact/proof:1.0:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact//proof:1.0:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/proof:bad:{digest}"
        ),
        format!(
            "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/proof:1.0:{}",
            "GG".repeat(Hash::BYTE_LEN)
        ),
    ] {
        let mut bad_json = json.clone();
        replace_object_field(
            &mut bad_json,
            "obligation_fingerprint",
            CanonicalJson::string(bad_hash),
        );

        assert!(matches!(
            read_proof_witness_ref(&bad_json, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::InvalidHash { path, .. })
                if path == "$.obligation_fingerprint"
        ));
    }
}

#[test]
fn reader_rejects_wrong_hash_classes_on_each_hash_field() {
    let reference = sample_reference();
    let json = proof_witness_ref_json(&reference).expect("canonical JSON");

    let mut wrong_witness_hash_class = json.clone();
    replace_object_field(
        &mut wrong_witness_hash_class,
        "witness_artifact_hash",
        hash_ref_json(
            ArtifactHashClass::Interface,
            "mizar-kernel/formula-evidence-witness",
            91,
        ),
    );
    assert!(matches!(
        read_proof_witness_ref(
            &wrong_witness_hash_class,
            ProofWitnessReadOptions::default()
        ),
        Err(ProofWitnessError::InvalidHash { path, .. })
            if path == "$.witness_artifact_hash"
    ));

    for (field, family, seed) in [
        ("kernel_profile_fingerprint", "mizar-kernel/profile", 92),
        (
            "verifier_policy_fingerprint",
            "mizar-proof/verifier-policy",
            93,
        ),
        ("target_binding_hash", "mizar-vc/kernel-target-binding", 94),
        ("formula_evidence_hash", "mizar-kernel/formula-evidence", 95),
        (
            "substitution_evidence_hash",
            "mizar-kernel/substitution-evidence",
            96,
        ),
        ("provenance_hash", "mizar-kernel/evidence-provenance", 97),
        ("formula_context_hash", "mizar-kernel/formula-context", 98),
        ("accepted_result_hash", "mizar-kernel/accepted-result", 99),
    ] {
        let mut wrong_class = json.clone();
        replace_nested_object_field(
            &mut wrong_class,
            "kernel_acceptance",
            field,
            hash_ref_json(ArtifactHashClass::Artifact, family, seed),
        );
        let expected_path = format!("$.kernel_acceptance.{field}");
        assert!(
            matches!(
                read_proof_witness_ref(&wrong_class, ProofWitnessReadOptions::default()),
                Err(ProofWitnessError::InvalidHash { path, .. }) if path == expected_path
            ),
            "{field}"
        );
    }
}

#[test]
fn formula_substitution_evidence_matrix_and_legacy_certificates_are_validated() {
    let valid_json = proof_witness_ref_json(&sample_reference()).expect("valid witness ref");
    assert!(read_proof_witness_ref(&valid_json, ProofWitnessReadOptions::default()).is_ok());

    let mut invalid_status_json =
        proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
    replace_object_field(
        &mut invalid_status_json,
        "evidence_kind",
        CanonicalJson::string("atp_certificate"),
    );
    assert!(matches!(
        read_proof_witness_ref(&invalid_status_json, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::InvalidStatusEvidence { .. })
    ));

    for (field, value) in [
        ("proof_status", "discharged_builtin"),
        ("evidence_kind", "builtin_certificate"),
        ("evidence_kind", "kernel_primitive"),
    ] {
        let mut legacy_enum_json =
            proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
        replace_object_field(&mut legacy_enum_json, field, CanonicalJson::string(value));
        assert!(
            matches!(
                read_proof_witness_ref(&legacy_enum_json, ProofWitnessReadOptions::default()),
                Err(ProofWitnessError::InvalidStatusEvidence { .. })
            ),
            "{field}={value}"
        );
    }

    let legacy_mutations: [fn(&mut ProofWitnessRef); 3] = [
        |reference: &mut ProofWitnessRef| reference.evidence_kind = EvidenceKind::AtpCertificate,
        |reference: &mut ProofWitnessRef| {
            reference.proof_status = ProofStatus::DischargedBuiltin;
            reference.evidence_kind = EvidenceKind::BuiltinCertificate;
        },
        |reference: &mut ProofWitnessRef| {
            reference.proof_status = ProofStatus::DischargedBuiltin;
            reference.evidence_kind = EvidenceKind::KernelPrimitive;
        },
    ];
    for mutate in legacy_mutations {
        let mut legacy_reference = sample_reference();
        mutate(&mut legacy_reference);
        assert!(matches!(
            proof_witness_ref_json(&legacy_reference),
            Err(ProofWitnessError::InvalidStatusEvidence { .. })
        ));
    }

    for field in [
        "status",
        "evidence",
        "backend_method",
        "resolution_trace",
        "smt_proof_object",
        "instantiated_formulas",
        "sat_problem",
    ] {
        let mut legacy_payload_json =
            proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
        replace_object_field(
            &mut legacy_payload_json,
            field,
            CanonicalJson::string("not trusted witness content"),
        );
        assert!(
            matches!(
                read_proof_witness_ref(&legacy_payload_json, ProofWitnessReadOptions::default()),
                Err(ProofWitnessError::UnknownField { path, field: unknown })
                    if path == "$" && unknown == field
            ),
            "{field}"
        );
    }

    let mut legacy_certificate_format_json =
        proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
    replace_nested_object_field(
        &mut legacy_certificate_format_json,
        "kernel_acceptance",
        "certificate_format",
        CanonicalJson::string("tptp-tstp/v1"),
    );
    assert!(matches!(
        read_proof_witness_ref(
            &legacy_certificate_format_json,
            ProofWitnessReadOptions::default()
        ),
        Err(ProofWitnessError::UnknownField { path, field })
            if path == "$.kernel_acceptance" && field == "certificate_format"
    ));

    let mut legacy_backend_log_json =
        proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
    replace_nested_object_field(
        &mut legacy_backend_log_json,
        "kernel_acceptance",
        "backend_log_hash",
        hash_ref_json(ArtifactHashClass::Diagnostic, "mizar-atp/backend-log", 51),
    );
    assert!(matches!(
        read_proof_witness_ref(&legacy_backend_log_json, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::UnknownField { path, field })
            if path == "$.kernel_acceptance" && field == "backend_log_hash"
    ));

    let mut legacy_used_axioms_json =
        proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
    replace_nested_object_field(
        &mut legacy_used_axioms_json,
        "kernel_acceptance",
        "used_axioms_hash",
        hash_ref_json(ArtifactHashClass::Diagnostic, "mizar-proof/used-axioms", 52),
    );
    assert!(matches!(
        read_proof_witness_ref(&legacy_used_axioms_json, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::UnknownField { path, field })
            if path == "$.kernel_acceptance" && field == "used_axioms_hash"
    ));

    let mut legacy_resolution_trace_json =
        proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
    replace_nested_object_field(
        &mut legacy_resolution_trace_json,
        "kernel_acceptance",
        "resolution_trace_hash",
        hash_ref_json(
            ArtifactHashClass::Diagnostic,
            "mizar-atp/resolution-trace",
            53,
        ),
    );
    assert!(matches!(
        read_proof_witness_ref(
            &legacy_resolution_trace_json,
            ProofWitnessReadOptions::default()
        ),
        Err(ProofWitnessError::UnknownField { path, field })
            if path == "$.kernel_acceptance" && field == "resolution_trace_hash"
    ));
}

#[test]
fn reader_rejects_unsafe_witness_paths() {
    let json = proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
    for path_value in [
        "",
        "/proof-witnesses/cert.json",
        "other/cert.json",
        "proof-witnesses/",
        "proof-witnesses//cert.json",
        "proof-witnesses/./cert.json",
        "proof-witnesses/../cert.json",
        "proof-witnesses/a/..\\escape.json",
    ] {
        let mut reference = sample_reference();
        reference.witness_path = path_value.to_owned();
        assert!(
            matches!(
                proof_witness_ref_json(&reference),
                Err(ProofWitnessError::InvalidField { path, .. }) if path == "$.witness_path"
            ),
            "{path_value}"
        );

        let mut bad_json = json.clone();
        replace_object_field(
            &mut bad_json,
            "witness_path",
            CanonicalJson::string(path_value),
        );
        assert!(
            matches!(
                read_proof_witness_ref(&bad_json, ProofWitnessReadOptions::default()),
                Err(ProofWitnessError::InvalidField { path, .. }) if path == "$.witness_path"
            ),
            "{path_value}"
        );
    }
}

#[test]
fn reader_rejects_missing_unknown_and_empty_fields() {
    let reference = sample_reference();
    let json = proof_witness_ref_json(&reference).expect("canonical JSON");

    let mut missing = json.clone();
    remove_object_field(&mut missing, "witness_artifact_hash");
    assert!(matches!(
        read_proof_witness_ref(&missing, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::MissingField { path }) if path == "$.witness_artifact_hash"
    ));

    let mut unknown = json.clone();
    replace_object_field(&mut unknown, "cache_record", CanonicalJson::bool(true));
    assert!(matches!(
        read_proof_witness_ref(&unknown, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::UnknownField { path, field })
            if path == "$" && field == "cache_record"
    ));

    let mut missing_nested = json.clone();
    remove_nested_object_field(
        &mut missing_nested,
        "kernel_acceptance",
        "accepted_result_hash",
    );
    assert!(matches!(
        read_proof_witness_ref(&missing_nested, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::MissingField { path })
            if path == "$.kernel_acceptance.accepted_result_hash"
    ));

    let mut missing_target_binding_hash = json.clone();
    remove_nested_object_field(
        &mut missing_target_binding_hash,
        "kernel_acceptance",
        "target_binding_hash",
    );
    assert!(matches!(
        read_proof_witness_ref(
            &missing_target_binding_hash,
            ProofWitnessReadOptions::default()
        ),
        Err(ProofWitnessError::MissingField { path })
            if path == "$.kernel_acceptance.target_binding_hash"
    ));

    let mut missing_formula_context_hash = json.clone();
    remove_nested_object_field(
        &mut missing_formula_context_hash,
        "kernel_acceptance",
        "formula_context_hash",
    );
    assert!(matches!(
        read_proof_witness_ref(
            &missing_formula_context_hash,
            ProofWitnessReadOptions::default()
        ),
        Err(ProofWitnessError::MissingField { path })
            if path == "$.kernel_acceptance.formula_context_hash"
    ));

    let mut unknown_nested = json.clone();
    replace_nested_object_field(
        &mut unknown_nested,
        "kernel_acceptance",
        "proof_authority",
        CanonicalJson::bool(true),
    );
    assert!(matches!(
        read_proof_witness_ref(&unknown_nested, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::UnknownField { path, field })
            if path == "$.kernel_acceptance" && field == "proof_authority"
    ));

    let mut empty_id = json;
    replace_object_field(&mut empty_id, "obligation_id", CanonicalJson::string(""));
    assert!(matches!(
        read_proof_witness_ref(&empty_id, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::InvalidField { path, .. }) if path == "$.obligation_id"
    ));
}

fn sample_reference() -> ProofWitnessRef {
    ProofWitnessRef {
        schema_version: current_schema_version(),
        obligation_id: "obligation:hidden:1".to_owned(),
        obligation_fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-proof/obligation", 1),
        proof_status: ProofStatus::KernelVerified,
        evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
        witness_path: "proof-witnesses/hidden/obligation-1.json".to_owned(),
        witness_artifact_hash: hash_ref(
            ArtifactHashClass::Artifact,
            "mizar-kernel/formula-evidence-witness",
            2,
        ),
        kernel_acceptance: KernelAcceptanceMetadata {
            kernel_profile_fingerprint: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-kernel/profile",
                3,
            ),
            verifier_policy_fingerprint: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-proof/verifier-policy",
                4,
            ),
            checker_schema_version: SchemaVersion::new(1, 0),
            evidence_schema_version: SchemaVersion::new(1, 0),
            target_binding_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-vc/kernel-target-binding",
                5,
            ),
            formula_evidence_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-kernel/formula-evidence",
                6,
            ),
            substitution_evidence_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-kernel/substitution-evidence",
                7,
            ),
            provenance_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-kernel/evidence-provenance",
                8,
            ),
            formula_context_hash: Some(hash_ref(
                ArtifactHashClass::Interface,
                "mizar-kernel/formula-context",
                9,
            )),
            accepted_result_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-kernel/accepted-result",
                10,
            ),
        },
    }
}

fn hash_ref(class: ArtifactHashClass, schema_family: &str, seed: u8) -> ArtifactHashRef {
    ArtifactHashRef::new(class, schema_family, SchemaVersion::new(1, 0), hash(seed))
}

fn hash_ref_json(class: ArtifactHashClass, schema_family: &str, seed: u8) -> CanonicalJson {
    CanonicalJson::string(hash_ref(class, schema_family, seed).to_artifact_hash_string())
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn replace_object_field(value: &mut CanonicalJson, field: &str, replacement: CanonicalJson) {
    let CanonicalJson::Object(fields) = value else {
        panic!("expected object");
    };
    fields.insert(field.to_owned(), replacement);
}

fn remove_object_field(value: &mut CanonicalJson, field: &str) {
    let CanonicalJson::Object(fields) = value else {
        panic!("expected object");
    };
    fields.remove(field).expect("object field");
}

fn replace_nested_object_field(
    value: &mut CanonicalJson,
    parent: &str,
    field: &str,
    replacement: CanonicalJson,
) {
    replace_object_field(object_field_mut(value, parent), field, replacement);
}

fn remove_nested_object_field(value: &mut CanonicalJson, parent: &str, field: &str) {
    remove_object_field(object_field_mut(value, parent), field);
}

fn object_field_mut<'a>(value: &'a mut CanonicalJson, field: &str) -> &'a mut CanonicalJson {
    let CanonicalJson::Object(fields) = value else {
        panic!("expected object");
    };
    fields.get_mut(field).expect("object field")
}
