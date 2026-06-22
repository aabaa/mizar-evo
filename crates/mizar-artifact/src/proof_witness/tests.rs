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
        "mizar-artifact/proof-witness-payload",
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
    replace_object_field(&mut json, "schema_version", CanonicalJson::string("2.0"));

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

    let mut newer_minor = proof_witness_ref_json(&reference).expect("canonical JSON");
    replace_object_field(
        &mut newer_minor,
        "schema_version",
        CanonicalJson::string("1.1"),
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
            "mizar-artifact/proof-witness-payload",
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

    let mut wrong_accepted_result_class = json.clone();
    replace_nested_object_field(
        &mut wrong_accepted_result_class,
        "kernel_acceptance",
        "accepted_result_hash",
        hash_ref_json(ArtifactHashClass::Artifact, "mizar-kernel/result", 92),
    );
    assert!(matches!(
        read_proof_witness_ref(
            &wrong_accepted_result_class,
            ProofWitnessReadOptions::default()
        ),
        Err(ProofWitnessError::InvalidHash { path, .. })
            if path == "$.kernel_acceptance.accepted_result_hash"
    ));

    let mut wrong_used_axioms_class = json;
    replace_nested_object_field(
        &mut wrong_used_axioms_class,
        "kernel_acceptance",
        "used_axioms_hash",
        hash_ref_json(ArtifactHashClass::Interface, "mizar-proof/used-axioms", 93),
    );
    assert!(matches!(
        read_proof_witness_ref(&wrong_used_axioms_class, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::InvalidHash { path, .. })
            if path == "$.kernel_acceptance.used_axioms_hash"
    ));
}

#[test]
fn status_evidence_and_certificate_format_matrix_is_validated() {
    let mut valid_builtin = sample_reference();
    valid_builtin.proof_status = ProofStatus::DischargedBuiltin;
    valid_builtin.evidence_kind = EvidenceKind::BuiltinCertificate;
    valid_builtin.kernel_acceptance.certificate_format = Some("mizar-builtin-cert/v1".to_owned());
    valid_builtin.kernel_acceptance.accepted_result_hash =
        hash_ref(ArtifactHashClass::Interface, "mizar-kernel/result", 51);
    let valid_builtin_json =
        proof_witness_ref_json(&valid_builtin).expect("valid builtin witness ref");
    assert!(
        read_proof_witness_ref(&valid_builtin_json, ProofWitnessReadOptions::default()).is_ok()
    );

    let mut valid_primitive = sample_reference();
    valid_primitive.proof_status = ProofStatus::DischargedBuiltin;
    valid_primitive.evidence_kind = EvidenceKind::KernelPrimitive;
    valid_primitive.kernel_acceptance.certificate_format = None;
    valid_primitive.kernel_acceptance.accepted_result_hash =
        hash_ref(ArtifactHashClass::Interface, "mizar-kernel/result", 52);
    let valid_primitive_json =
        proof_witness_ref_json(&valid_primitive).expect("valid primitive witness ref");
    assert!(
        read_proof_witness_ref(&valid_primitive_json, ProofWitnessReadOptions::default()).is_ok()
    );

    let mut invalid_status = sample_reference();
    invalid_status.proof_status = ProofStatus::KernelVerified;
    invalid_status.evidence_kind = EvidenceKind::BuiltinCertificate;
    assert!(matches!(
        proof_witness_ref_json(&invalid_status),
        Err(ProofWitnessError::InvalidStatusEvidence { .. })
    ));
    let mut invalid_status_json =
        proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
    replace_object_field(
        &mut invalid_status_json,
        "evidence_kind",
        CanonicalJson::string("builtin_certificate"),
    );
    assert!(matches!(
        read_proof_witness_ref(&invalid_status_json, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::InvalidStatusEvidence { .. })
    ));

    let mut missing_certificate_format = sample_reference();
    missing_certificate_format
        .kernel_acceptance
        .certificate_format = None;
    assert!(matches!(
        proof_witness_ref_json(&missing_certificate_format),
        Err(ProofWitnessError::InvalidField { path, .. })
            if path == "$.kernel_acceptance.certificate_format"
    ));
    let mut missing_certificate_format_json =
        proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
    replace_nested_object_field(
        &mut missing_certificate_format_json,
        "kernel_acceptance",
        "certificate_format",
        CanonicalJson::null(),
    );
    assert!(matches!(
        read_proof_witness_ref(
            &missing_certificate_format_json,
            ProofWitnessReadOptions::default()
        ),
        Err(ProofWitnessError::InvalidField { path, .. })
            if path == "$.kernel_acceptance.certificate_format"
    ));

    let mut primitive_with_format = valid_primitive;
    primitive_with_format.kernel_acceptance.certificate_format =
        Some("kernel-primitive/v1".to_owned());
    assert!(matches!(
        proof_witness_ref_json(&primitive_with_format),
        Err(ProofWitnessError::InvalidField { path, .. })
            if path == "$.kernel_acceptance.certificate_format"
    ));
    let mut primitive_with_format_json = valid_primitive_json;
    replace_nested_object_field(
        &mut primitive_with_format_json,
        "kernel_acceptance",
        "certificate_format",
        CanonicalJson::string("kernel-primitive/v1"),
    );
    assert!(matches!(
        read_proof_witness_ref(&primitive_with_format_json, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::InvalidField { path, .. })
            if path == "$.kernel_acceptance.certificate_format"
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

    let mut missing_certificate_format = json.clone();
    remove_nested_object_field(
        &mut missing_certificate_format,
        "kernel_acceptance",
        "certificate_format",
    );
    assert!(matches!(
        read_proof_witness_ref(
            &missing_certificate_format,
            ProofWitnessReadOptions::default()
        ),
        Err(ProofWitnessError::MissingField { path })
            if path == "$.kernel_acceptance.certificate_format"
    ));

    let mut missing_used_axioms_hash = json.clone();
    remove_nested_object_field(
        &mut missing_used_axioms_hash,
        "kernel_acceptance",
        "used_axioms_hash",
    );
    assert!(matches!(
        read_proof_witness_ref(&missing_used_axioms_hash, ProofWitnessReadOptions::default()),
        Err(ProofWitnessError::MissingField { path })
            if path == "$.kernel_acceptance.used_axioms_hash"
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
        evidence_kind: EvidenceKind::AtpCertificate,
        witness_path: "proof-witnesses/hidden/obligation-1.json".to_owned(),
        witness_artifact_hash: hash_ref(
            ArtifactHashClass::Artifact,
            "mizar-artifact/proof-witness-payload",
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
            certificate_format: Some("tptp-tstp/v1".to_owned()),
            accepted_result_hash: hash_ref(ArtifactHashClass::Interface, "mizar-kernel/result", 5),
            used_axioms_hash: Some(hash_ref(
                ArtifactHashClass::Diagnostic,
                "mizar-proof/used-axioms",
                6,
            )),
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
