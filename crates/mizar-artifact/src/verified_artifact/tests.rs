use std::collections::BTreeMap;

use mizar_session::Hash;

use super::{
    ArtifactDiagnostic, BuildProvenance, DependencyArtifactHash, DiagnosticRelated,
    DiagnosticSeverity, ExportProofStatus, ExportVisibility, ExpressionMetadata,
    ObligationMetadata, ObligationStatus, OverloadMetadata, VERIFIED_ARTIFACT_SCHEMA_FAMILY,
    VerifiedArtifact, VerifiedArtifactError, VerifiedArtifactReadOptions, VerifiedExport,
    artifact_hash_excluded_paths, current_schema_version, implementation_hash_input_json,
    interface_hash_input_json, read_verified_artifact, verified_artifact_hash_string,
    verified_artifact_json, write_verified_artifact,
};
use crate::{
    module_summary::{ModuleSummaryIdentity, SourceRangeSummary},
    proof_witness::{
        EvidenceKind, KernelAcceptanceMetadata, ProofStatus as WitnessProofStatus, ProofWitnessRef,
    },
    registration_summary::{ArtifactHashClass, ArtifactHashRef},
    store::{
        CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersion, SchemaVersionError,
        artifact_hash_domain, canonical_json_string,
    },
};

#[test]
fn verified_artifact_round_trips_through_canonical_json() {
    let artifact = sample_artifact();
    let json = verified_artifact_json(&artifact).expect("canonical verified artifact JSON");
    let bytes = write_verified_artifact(&artifact).expect("canonical verified artifact bytes");

    assert_eq!(bytes, canonical_json_string(&json).into_bytes());
    assert_eq!(
        read_verified_artifact(
            &json,
            VerifiedArtifactReadOptions {
                expected_module: Some(&artifact.module),
                expected_interface_hash: Some(artifact.interface_hash),
                expected_implementation_hash: Some(artifact.implementation_hash),
                ..VerifiedArtifactReadOptions::default()
            }
        )
        .expect("valid verified artifact"),
        artifact
    );
}

#[test]
fn verified_artifact_writer_and_hash_inputs_are_deterministic_for_identical_inputs() {
    let artifact = sample_artifact();
    let excluded_paths = artifact_hash_excluded_paths();
    let artifact_domain =
        artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version);
    let first_json = verified_artifact_json(&artifact).expect("first canonical JSON");
    let first_bytes = write_verified_artifact(&artifact).expect("first canonical bytes");
    let first_artifact_hash = artifact_domain.hash(&first_json, &excluded_paths);
    let first_interface_input =
        interface_hash_input_json(&artifact).expect("first interface hash input");
    let first_implementation_input =
        implementation_hash_input_json(&artifact).expect("first implementation hash input");
    let first_interface_hash = artifact
        .compute_interface_hash()
        .expect("first interface hash");
    let first_implementation_hash = artifact
        .compute_implementation_hash()
        .expect("first implementation hash");

    for _ in 0..3 {
        let json = verified_artifact_json(&artifact).expect("repeated canonical JSON");
        assert_eq!(json, first_json);
        assert_eq!(canonical_json_string(&json).into_bytes(), first_bytes);
        assert_eq!(
            write_verified_artifact(&artifact).expect("repeated canonical bytes"),
            first_bytes
        );
        assert_eq!(
            artifact_domain.hash(&json, &excluded_paths),
            first_artifact_hash
        );
        assert_eq!(
            interface_hash_input_json(&artifact).expect("repeated interface hash input"),
            first_interface_input
        );
        assert_eq!(
            implementation_hash_input_json(&artifact).expect("repeated implementation hash input"),
            first_implementation_input
        );
        assert_eq!(
            artifact
                .compute_interface_hash()
                .expect("repeated interface hash"),
            first_interface_hash
        );
        assert_eq!(
            artifact
                .compute_implementation_hash()
                .expect("repeated implementation hash"),
            first_implementation_hash
        );
    }
}

#[test]
fn public_hash_input_helpers_match_computed_hashes() {
    let artifact = sample_artifact();
    let interface_input = interface_hash_input_json(&artifact).expect("interface hash input");
    let implementation_input =
        implementation_hash_input_json(&artifact).expect("implementation hash input");
    let interface_domain = CanonicalHashDomain::new(
        HashClass::Interface,
        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
        artifact.schema_version,
    );
    let implementation_domain = CanonicalHashDomain::new(
        HashClass::Implementation,
        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
        artifact.schema_version,
    );

    assert_eq!(
        interface_domain.hash(&interface_input, &[]),
        artifact.compute_interface_hash().expect("interface hash")
    );
    assert_eq!(
        implementation_domain.hash(&implementation_input, &[]),
        artifact
            .compute_implementation_hash()
            .expect("implementation hash")
    );
    assert_ne!(
        interface_domain.hash(&interface_input, &[]),
        implementation_domain.hash(&implementation_input, &[])
    );
}

#[test]
fn incompatible_version_reads_fail_cleanly() {
    let mut json = sample_json();
    set_field(&mut json, "schema_version", CanonicalJson::string("1.1"));

    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::SchemaVersion(
            SchemaVersionError::MinorTooNew { .. }
        ))
    ));

    let mut json = sample_json();
    object_mut(&mut json).remove("schema_version");
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::SchemaVersion(
            SchemaVersionError::Missing { .. }
        ))
    ));

    let mut json = sample_json();
    set_field(
        &mut json,
        "schema_version",
        CanonicalJson::string("not-a-version"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::SchemaVersion(
            SchemaVersionError::Malformed { .. }
        ))
    ));

    let mut json = sample_json();
    set_field(&mut json, "schema_version", CanonicalJson::string("2.0"));
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::SchemaVersion(
            SchemaVersionError::MajorMismatch { .. }
        ))
    ));
}

#[test]
fn reader_rejects_source_ranges_paths_and_timestamps() {
    let mut json = sample_json();
    let export = array_field_mut(&mut json, "exports").first_mut().unwrap();
    set_nested_range(export, "source_range", 50, 10);
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidField { path, .. })
            if path == "$.exports[0].source_range"
    ));

    let mut json = sample_json();
    set_field(
        &mut json,
        "source_file",
        CanonicalJson::string("../article.miz"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidField { path, .. })
            if path == "$.source_file"
    ));

    let mut json = sample_json();
    set_field(
        &mut json,
        "verified_at",
        CanonicalJson::string("2026-06-22T14:03:05.1Z"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidField { path, .. })
            if path == "$.verified_at"
    ));
}

#[test]
fn reader_rejects_invalid_hash_domains_and_checks_hash_participation() {
    let artifact = sample_artifact();

    let mut json = sample_json();
    set_field(
        &mut json,
        "interface_hash",
        CanonicalJson::string(verified_artifact_hash_string(
            ArtifactHashClass::Diagnostic,
            artifact.schema_version,
            artifact.interface_hash,
        )),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidHash { path, .. })
            if path == "$.interface_hash"
    ));

    let mut json = sample_json();
    let export = array_field_mut(&mut json, "exports").first_mut().unwrap();
    set_object_field(
        export,
        "interface_fingerprint",
        CanonicalJson::string(
            hash_ref(ArtifactHashClass::Diagnostic, "mizar-doc/section", 88)
                .to_artifact_hash_string(),
        ),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidHash { path, .. })
            if path == "$.exports[0].interface_fingerprint"
    ));

    let mut json = sample_json();
    let export = array_field_mut(&mut json, "exports").first_mut().unwrap();
    set_object_field(
        export,
        "rendered_signature",
        CanonicalJson::string("changed importer signature"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InterfaceHashMismatch { .. })
    ));

    let mut json = sample_json();
    let expression = array_field_mut(&mut json, "expressions")
        .first_mut()
        .unwrap();
    set_object_field(
        expression,
        "rendered_surface",
        CanonicalJson::string("changed local surface"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
    ));
}

#[test]
fn reader_rejects_hash_domain_mismatches_across_schema_fields() {
    let artifact = sample_artifact();

    let mut json = sample_json();
    set_field(
        &mut json,
        "interface_hash",
        CanonicalJson::string(artifact_framed_hash_string_for_test(
            "mizar-artifact/other-schema",
            ArtifactHashClass::Interface,
            artifact.schema_version,
            artifact.interface_hash,
        )),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidHash { path, .. })
            if path == "$.interface_hash"
    ));

    let mut json = sample_json();
    set_field(
        &mut json,
        "implementation_hash",
        CanonicalJson::string(verified_artifact_hash_string(
            ArtifactHashClass::Interface,
            artifact.schema_version,
            artifact.implementation_hash,
        )),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidHash { path, .. })
            if path == "$.implementation_hash"
    ));

    let mut json = sample_json();
    set_field(
        &mut json,
        "source_hash",
        CanonicalJson::string(
            hash_ref(
                ArtifactHashClass::Artifact,
                "mizar-artifact/wrong-source",
                91,
            )
            .to_artifact_hash_string(),
        ),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidHash { path, .. })
            if path == "$.source_hash"
    ));

    let mut json = sample_json();
    let obligation = array_field_mut(&mut json, "obligations")
        .first_mut()
        .unwrap();
    set_object_field(
        obligation,
        "local_context_fingerprint",
        CanonicalJson::string(
            hash_ref(ArtifactHashClass::Artifact, "mizar-vc/local-context", 92)
                .to_artifact_hash_string(),
        ),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidHash { path, .. })
            if path == "$.obligations[0].local_context_fingerprint"
    ));

    let mut json = sample_json();
    let diagnostic = array_field_mut(&mut json, "diagnostics")
        .first_mut()
        .unwrap();
    set_object_field(
        diagnostic,
        "explanation_ref",
        CanonicalJson::string(
            hash_ref(
                ArtifactHashClass::Interface,
                "mizar-diagnostics/explanation",
                93,
            )
            .to_artifact_hash_string(),
        ),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidHash { path, .. })
            if path == "$.diagnostics[0].explanation_ref"
    ));

    let mut json = sample_json();
    set_object_field(
        object_field_mut(&mut json, "provenance"),
        "lockfile_hash",
        CanonicalJson::string(
            hash_ref(ArtifactHashClass::Interface, "mizar-build/lockfile", 94)
                .to_artifact_hash_string(),
        ),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidHash { path, .. })
            if path == "$.provenance.lockfile_hash"
    ));

    let mut json = sample_json();
    let dependency = array_object_field_mut(
        object_field_mut(&mut json, "provenance"),
        "dependency_artifact_hashes",
    )
    .first_mut()
    .unwrap();
    set_object_field(
        dependency,
        "implementation_hash",
        CanonicalJson::string(
            hash_ref(
                ArtifactHashClass::Interface,
                "mizar-artifact/verified-artifact",
                95,
            )
            .to_artifact_hash_string(),
        ),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidHash { path, .. })
            if path == "$.provenance.dependency_artifact_hashes[0].implementation_hash"
    ));
}

#[test]
fn implementation_hash_participates_in_stable_obligation_witness_diagnostic_and_provenance_fields()
{
    let mut json = sample_json();
    let obligation = array_field_mut(&mut json, "obligations")
        .last_mut()
        .unwrap();
    set_object_field(
        obligation,
        "statement_summary",
        CanonicalJson::string("changed stable obligation statement"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
    ));

    let mut json = sample_json();
    let witness = array_field_mut(&mut json, "proof_witnesses")
        .first_mut()
        .unwrap();
    set_object_field(
        witness,
        "witness_path",
        CanonicalJson::string("proof-witnesses/hidden/changed.json"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
    ));

    let mut json = sample_json();
    let diagnostic = array_field_mut(&mut json, "diagnostics")
        .first_mut()
        .unwrap();
    set_object_field(
        diagnostic,
        "rendered_message",
        CanonicalJson::string("changed diagnostic message"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
    ));

    let mut json = sample_json();
    set_object_field(
        object_field_mut(&mut json, "provenance"),
        "toolchain",
        CanonicalJson::string("changed-toolchain"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
    ));
}

#[test]
fn verified_at_and_cache_key_are_hash_excluded() {
    let original = sample_artifact();
    let mut json = sample_json();
    set_field(
        &mut json,
        "verified_at",
        CanonicalJson::string("2026-06-23T00:00:00Z"),
    );
    set_object_field(
        object_field_mut(&mut json, "provenance"),
        "cache_key",
        CanonicalJson::string("new-cache-key"),
    );

    let read = read_verified_artifact(&json, VerifiedArtifactReadOptions::default())
        .expect("hash-excluded metadata change remains valid");
    assert_eq!(read.interface_hash, original.interface_hash);
    assert_eq!(read.implementation_hash, original.implementation_hash);
    assert_eq!(read.verified_at.as_deref(), Some("2026-06-23T00:00:00Z"));
    assert_eq!(read.provenance.cache_key.as_deref(), Some("new-cache-key"));
}

#[test]
fn artifact_hash_exclusions_cover_local_provenance_fields() {
    let original = sample_json();
    let domain = artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, current_schema_version());
    let excluded_paths = artifact_hash_excluded_paths();
    let original_hash = domain.hash(&original, &excluded_paths);

    let mut local_changed = original.clone();
    set_field(
        &mut local_changed,
        "verified_at",
        CanonicalJson::string("2026-06-23T00:00:00Z"),
    );
    set_object_field(
        object_field_mut(&mut local_changed, "provenance"),
        "cache_key",
        CanonicalJson::string("new-cache-key"),
    );
    assert_eq!(domain.hash(&local_changed, &excluded_paths), original_hash);

    for (field, mutate) in [
        (
            "provenance.toolchain",
            mutate_provenance_toolchain as fn(&mut CanonicalJson),
        ),
        (
            "provenance.language_edition",
            mutate_provenance_language_edition,
        ),
        ("provenance.lockfile_hash", mutate_provenance_lockfile_hash),
        (
            "provenance.verifier_config_hash",
            mutate_provenance_verifier_config_hash,
        ),
        (
            "provenance.dependency_artifact_hashes[].module",
            mutate_dependency_module,
        ),
        (
            "provenance.dependency_artifact_hashes[].interface_hash",
            mutate_dependency_interface_hash,
        ),
        (
            "provenance.dependency_artifact_hashes[].implementation_hash",
            mutate_dependency_implementation_hash,
        ),
        (
            "provenance.dependency_artifact_hashes[].artifact_hash",
            mutate_dependency_artifact_hash,
        ),
    ] {
        let mut stable_changed = original.clone();
        mutate(&mut stable_changed);
        assert_ne!(
            domain.hash(&stable_changed, &excluded_paths),
            original_hash,
            "{field} must participate in the artifact hash"
        );
    }
}

#[test]
fn interface_hash_excludes_local_only_projection_fields() {
    let original = sample_artifact();
    let mut artifact = original.clone();
    artifact.source_file = "articles/renamed-local-path.miz".to_owned();
    artifact.source_hash = hash(96);
    artifact.exports[0].source_range = range(100, 120);
    artifact.exports[0].documentation_ref = Some(hash_ref(
        ArtifactHashClass::Diagnostic,
        "mizar-doc/section",
        97,
    ));
    artifact.expressions[0].rendered_surface = "changed local expression".to_owned();
    artifact.obligations[1].statement_summary = "changed local obligation".to_owned();
    artifact.proof_witnesses[0].witness_path =
        "proof-witnesses/hidden/local-changed.json".to_owned();
    artifact.diagnostics[0].rendered_message = "changed local diagnostic".to_owned();
    artifact.provenance.toolchain = "changed-toolchain".to_owned();
    artifact.provenance.lockfile_hash =
        hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 98);
    artifact.provenance.dependency_artifact_hashes[0].implementation_hash = Some(hash_ref(
        ArtifactHashClass::Implementation,
        "mizar-artifact/verified-artifact",
        99,
    ));
    artifact.refresh_hashes().expect("refresh hashes");

    assert_eq!(artifact.interface_hash, original.interface_hash);
    assert_ne!(artifact.implementation_hash, original.implementation_hash);
}

#[test]
fn public_hash_input_helpers_sort_participating_collections() {
    let baseline = sample_artifact_with_extra_ordering_items();
    let mut unsorted = baseline.clone();
    unsorted.exports.reverse();
    unsorted.expressions.reverse();
    unsorted.obligations.reverse();
    unsorted.proof_witnesses.reverse();
    unsorted
        .diagnostics
        .iter_mut()
        .find(|diagnostic| diagnostic.related.len() > 1)
        .expect("diagnostic with related entries")
        .related
        .reverse();
    unsorted.diagnostics.reverse();
    unsorted.provenance.dependency_artifact_hashes.reverse();

    assert_eq!(
        interface_hash_input_json(&unsorted).expect("unsorted interface input"),
        interface_hash_input_json(&baseline).expect("baseline interface input")
    );
    assert_eq!(
        implementation_hash_input_json(&unsorted).expect("unsorted implementation input"),
        implementation_hash_input_json(&baseline).expect("baseline implementation input")
    );
    assert_eq!(
        unsorted
            .compute_interface_hash()
            .expect("unsorted interface hash"),
        baseline
            .compute_interface_hash()
            .expect("baseline interface hash")
    );
    assert_eq!(
        unsorted
            .compute_implementation_hash()
            .expect("unsorted implementation hash"),
        baseline
            .compute_implementation_hash()
            .expect("baseline implementation hash")
    );
}

#[test]
fn implementation_only_edits_do_not_change_interface_hash_input() {
    let baseline = sample_artifact();
    let mut changed = baseline.clone();
    changed.source_file = "articles/renamed-local-path.miz".to_owned();
    changed.source_hash = hash(106);
    changed.exports[0].source_range = range(110, 120);
    changed.exports[0].documentation_ref = Some(hash_ref(
        ArtifactHashClass::Diagnostic,
        "mizar-doc/section",
        107,
    ));
    changed.expressions[0].rendered_surface = "changed local expression".to_owned();
    changed.obligations[1].statement_summary = "changed local obligation".to_owned();
    changed.proof_witnesses[0].witness_path =
        "proof-witnesses/hidden/local-changed-task16.json".to_owned();
    changed.diagnostics[0].rendered_message = "changed local diagnostic".to_owned();
    changed.provenance.toolchain = "changed-toolchain".to_owned();
    changed.provenance.language_edition = "2027".to_owned();
    changed.provenance.lockfile_hash =
        hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 108);
    changed.provenance.verifier_config_hash = hash_ref(
        ArtifactHashClass::Interface,
        "mizar-build/verifier-config",
        109,
    );
    changed.provenance.dependency_artifact_hashes[0].implementation_hash = Some(hash_ref(
        ArtifactHashClass::Implementation,
        "mizar-artifact/verified-artifact",
        110,
    ));
    changed.provenance.dependency_artifact_hashes[0].artifact_hash = Some(hash_ref(
        ArtifactHashClass::Artifact,
        "mizar-artifact/file",
        111,
    ));

    assert_eq!(
        interface_hash_input_json(&changed).expect("changed interface input"),
        interface_hash_input_json(&baseline).expect("baseline interface input")
    );
    assert_ne!(
        implementation_hash_input_json(&changed).expect("changed implementation input"),
        implementation_hash_input_json(&baseline).expect("baseline implementation input")
    );
    assert_eq!(
        changed
            .compute_interface_hash()
            .expect("changed interface hash"),
        baseline
            .compute_interface_hash()
            .expect("baseline interface hash")
    );
    assert_ne!(
        changed
            .compute_implementation_hash()
            .expect("changed implementation hash"),
        baseline
            .compute_implementation_hash()
            .expect("baseline implementation hash")
    );
}

#[test]
fn reader_rejects_inconsistent_witness_references() {
    let mut json = sample_json();
    let obligation = array_field_mut(&mut json, "obligations")
        .first_mut()
        .unwrap();
    set_object_field(
        obligation,
        "obligation_fingerprint",
        CanonicalJson::string(
            hash_ref(ArtifactHashClass::Interface, "mizar-proof/obligation", 89)
                .to_artifact_hash_string(),
        ),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
            if path == "$.obligations[0].obligation_fingerprint"
    ));

    let mut json = sample_json();
    let obligation = array_field_mut(&mut json, "obligations")
        .first_mut()
        .unwrap();
    set_object_field(
        obligation,
        "accepted_witness_obligation_id",
        CanonicalJson::string("other-obligation"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
            if path == "$.obligations[0].accepted_witness_obligation_id"
    ));

    let mut json = sample_json();
    let obligation = array_field_mut(&mut json, "obligations")
        .first_mut()
        .unwrap();
    set_object_field(
        obligation,
        "verifier_policy_fingerprint",
        CanonicalJson::string(
            hash_ref(ArtifactHashClass::Interface, "mizar-proof/policy", 90)
                .to_artifact_hash_string(),
        ),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
            if path == "$.obligations[0].verifier_policy_fingerprint"
    ));
}

#[test]
fn reader_rejects_proof_authority_status_boundary_violations() {
    let mut json = sample_json();
    array_field_mut(&mut json, "proof_witnesses").clear();
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
            if path == "$.obligations[0].accepted_witness_obligation_id"
    ));

    let mut json = sample_json();
    let obligation = array_field_mut(&mut json, "obligations")
        .first_mut()
        .unwrap();
    set_object_field(
        obligation,
        "deterministic_discharge_hash",
        CanonicalJson::string(
            hash_ref(ArtifactHashClass::Interface, "mizar-proof/discharge", 100)
                .to_artifact_hash_string(),
        ),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
            if path == "$.obligations[0].deterministic_discharge_hash"
    ));

    for status in ["open", "rejected", "externally_attested"] {
        let mut json = sample_json();
        let obligation = array_field_mut(&mut json, "obligations")
            .last_mut()
            .unwrap();
        set_object_field(obligation, "status", CanonicalJson::string(status));
        set_object_field(
            obligation,
            "accepted_witness_obligation_id",
            CanonicalJson::string("obl-2"),
        );
        set_object_field(
            obligation,
            "deterministic_discharge_hash",
            CanonicalJson::null(),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
                if path == "$.obligations[1].accepted_witness_obligation_id"
        ));
    }

    let mut json = sample_json();
    let obligation = array_field_mut(&mut json, "obligations")
        .last_mut()
        .unwrap();
    set_object_field(obligation, "status", CanonicalJson::string("open"));
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
            if path == "$.obligations[1].deterministic_discharge_hash"
    ));
}

#[test]
fn reader_rejects_extra_orphan_and_non_accepted_proof_witnesses() {
    let mut json = sample_json();
    let extra = array_field_mut(&mut json, "proof_witnesses")[0].clone();
    let witnesses = array_field_mut(&mut json, "proof_witnesses");
    witnesses.push(extra);
    let extra_witness = witnesses.last_mut().unwrap();
    set_object_field(
        extra_witness,
        "obligation_id",
        CanonicalJson::string("zzz-missing-obligation"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
            if path == "$.proof_witnesses[1].obligation_id"
    ));

    let mut json = sample_json();
    let extra = array_field_mut(&mut json, "proof_witnesses")[0].clone();
    let witnesses = array_field_mut(&mut json, "proof_witnesses");
    witnesses.push(extra);
    let extra_witness = witnesses.last_mut().unwrap();
    set_object_field(
        extra_witness,
        "obligation_id",
        CanonicalJson::string("obl-2"),
    );
    set_object_field(
        extra_witness,
        "obligation_fingerprint",
        CanonicalJson::string(
            hash_ref(ArtifactHashClass::Interface, "mizar-proof/obligation", 25)
                .to_artifact_hash_string(),
        ),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
            if path == "$.proof_witnesses[1].obligation_id"
    ));
}

#[test]
fn reader_rejects_raw_ir_and_ownership_boundary_fields() {
    let mut json = sample_json();
    let expression = array_field_mut(&mut json, "expressions")
        .first_mut()
        .unwrap();
    set_object_field(
        expression,
        "resolved_typed_ast",
        CanonicalJson::string("raw checker dump"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::UnknownField { path, field })
            if path == "$.expressions[0]" && field == "resolved_typed_ast"
    ));

    let mut json = sample_json();
    set_field(
        &mut json,
        "scheduler_state",
        CanonicalJson::string("not-owned-here"),
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::UnknownField { path, field })
            if path == "$" && field == "scheduler_state"
    ));
}

#[test]
fn writer_sorts_diagnostics_and_reader_rejects_unsorted_arrays() {
    let mut artifact = sample_artifact();
    artifact.diagnostics.reverse();
    artifact.diagnostics[0].related.reverse();
    artifact.refresh_hashes().expect("refresh hashes");

    let json = verified_artifact_json(&artifact).expect("writer sorts diagnostics");
    read_verified_artifact(&json, VerifiedArtifactReadOptions::default())
        .expect("writer output is reader-valid");

    let mut unsorted = sample_json();
    array_field_mut(&mut unsorted, "diagnostics").reverse();
    assert!(matches!(
        read_verified_artifact(&unsorted, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::UnsortedCollection { path })
            if path == "$.diagnostics"
    ));

    let mut unsorted_related = sample_json();
    let diagnostic = array_field_mut(&mut unsorted_related, "diagnostics")
        .first_mut()
        .unwrap();
    array_object_field_mut(diagnostic, "related").reverse();
    assert!(matches!(
        read_verified_artifact(&unsorted_related, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::UnsortedCollection { path })
            if path == "$.diagnostics[0].related"
    ));
}

#[test]
fn reader_rejects_duplicate_identity_keys_at_collection_boundaries() {
    for (field, expected_path) in [
        ("exports", "$.exports"),
        ("expressions", "$.expressions"),
        ("obligations", "$.obligations"),
        ("proof_witnesses", "$.proof_witnesses"),
        ("diagnostics", "$.diagnostics"),
    ] {
        let mut json = sample_json();
        duplicate_array_item(array_field_mut(&mut json, field), 0);
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::DuplicateEntry { path, .. })
                if path == expected_path
        ));
    }

    let mut json = sample_json();
    let diagnostic = array_field_mut(&mut json, "diagnostics")
        .first_mut()
        .unwrap();
    duplicate_array_item(array_object_field_mut(diagnostic, "related"), 0);
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::DuplicateEntry { path, .. })
            if path == "$.diagnostics[0].related"
    ));

    let mut json = sample_json();
    duplicate_array_item(
        array_object_field_mut(
            object_field_mut(&mut json, "provenance"),
            "dependency_artifact_hashes",
        ),
        0,
    );
    assert!(matches!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::DuplicateEntry { path, .. })
            if path == "$.provenance.dependency_artifact_hashes"
    ));
}

fn sample_json() -> CanonicalJson {
    verified_artifact_json(&sample_artifact()).expect("sample verified artifact JSON")
}

fn sample_artifact() -> VerifiedArtifact {
    let schema_version = current_schema_version();
    let module = identity("pkg", "articles/hidden", "2026");
    let dependency_module = identity("dep", "articles/base", "2026");
    let obligation_fingerprint =
        hash_ref(ArtifactHashClass::Interface, "mizar-proof/obligation", 20);
    let verifier_policy = hash_ref(ArtifactHashClass::Interface, "mizar-proof/policy", 21);

    let mut artifact = VerifiedArtifact {
        schema_version,
        module: module.clone(),
        source_file: "articles/hidden.miz".to_owned(),
        source_hash: hash(1),
        verified_at: Some("2026-06-22T14:03:05Z".to_owned()),
        interface_hash: hash(0),
        implementation_hash: hash(0),
        exports: vec![
            VerifiedExport {
                origin_id: "export-1".to_owned(),
                fully_qualified_name: "Hidden.Th1".to_owned(),
                namespace_path: vec!["Hidden".to_owned()],
                visibility: ExportVisibility::Public,
                export_kind: "theorem".to_owned(),
                source_range: range(10, 20),
                rendered_signature: "for x holds x = x".to_owned(),
                interface_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-checker/export",
                    2,
                ),
                proof_status: Some(ExportProofStatus::Accepted),
                documentation_ref: Some(hash_ref(
                    ArtifactHashClass::Diagnostic,
                    "mizar-doc/section",
                    3,
                )),
            },
            VerifiedExport {
                origin_id: "export-2".to_owned(),
                fully_qualified_name: "Hidden.Def1".to_owned(),
                namespace_path: vec!["Hidden".to_owned()],
                visibility: ExportVisibility::Reexported,
                export_kind: "definition".to_owned(),
                source_range: range(30, 44),
                rendered_signature: "func f -> object".to_owned(),
                interface_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-checker/export",
                    4,
                ),
                proof_status: Some(ExportProofStatus::NotRequired),
                documentation_ref: None,
            },
        ],
        expressions: vec![
            ExpressionMetadata {
                expression_id: "expr-1".to_owned(),
                source_range: range(11, 12),
                expression_kind: "term".to_owned(),
                rendered_surface: "x".to_owned(),
                inferred_type: Some("object".to_owned()),
                resolved_symbol: Some("Hidden.x".to_owned()),
                inserted_coercions: vec!["object-coercion".to_owned()],
                active_thesis: Some("x = x".to_owned()),
                overload_resolution: Some(OverloadMetadata {
                    root_symbol: "equals".to_owned(),
                    selected_candidate: "builtin.eq".to_owned(),
                    active_refinements: vec!["object".to_owned()],
                    coercion_summary: Some("identity".to_owned()),
                }),
            },
            ExpressionMetadata {
                expression_id: "expr-2".to_owned(),
                source_range: range(34, 39),
                expression_kind: "definition_head".to_owned(),
                rendered_surface: "f".to_owned(),
                inferred_type: None,
                resolved_symbol: None,
                inserted_coercions: Vec::new(),
                active_thesis: None,
                overload_resolution: None,
            },
        ],
        obligations: vec![
            ObligationMetadata {
                obligation_id: "obl-1".to_owned(),
                obligation_anchor: Some("Hidden.Th1.proof".to_owned()),
                owner_origin_id: Some("export-1".to_owned()),
                source_range: range(12, 18),
                obligation_kind: "theorem_body".to_owned(),
                statement_summary: "x = x".to_owned(),
                obligation_fingerprint: obligation_fingerprint.clone(),
                vc_fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-vc/vc", 22),
                local_context_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-vc/local-context",
                    23,
                ),
                dependency_slice_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-vc/dependency-slice",
                    24,
                ),
                verifier_policy_fingerprint: verifier_policy.clone(),
                status: ObligationStatus::Accepted,
                accepted_witness_obligation_id: Some("obl-1".to_owned()),
                deterministic_discharge_hash: None,
                diagnostic_ref: None,
            },
            ObligationMetadata {
                obligation_id: "obl-2".to_owned(),
                obligation_anchor: None,
                owner_origin_id: Some("export-2".to_owned()),
                source_range: range(30, 44),
                obligation_kind: "definition_totality".to_owned(),
                statement_summary: "definition requires no proof".to_owned(),
                obligation_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-proof/obligation",
                    25,
                ),
                vc_fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-vc/vc", 26),
                local_context_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-vc/local-context",
                    27,
                ),
                dependency_slice_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-vc/dependency-slice",
                    28,
                ),
                verifier_policy_fingerprint: verifier_policy.clone(),
                status: ObligationStatus::NotRequired,
                accepted_witness_obligation_id: None,
                deterministic_discharge_hash: Some(hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-proof/discharge",
                    29,
                )),
                diagnostic_ref: Some(hash_ref(
                    ArtifactHashClass::Diagnostic,
                    "mizar-diagnostics/obligation",
                    30,
                )),
            },
        ],
        proof_witnesses: vec![ProofWitnessRef {
            schema_version,
            obligation_id: "obl-1".to_owned(),
            obligation_fingerprint,
            proof_status: WitnessProofStatus::KernelVerified,
            evidence_kind: EvidenceKind::AtpCertificate,
            witness_path: "proof-witnesses/hidden/obl-1.json".to_owned(),
            witness_artifact_hash: hash_ref(
                ArtifactHashClass::Artifact,
                "mizar-proof/witness-file",
                31,
            ),
            kernel_acceptance: KernelAcceptanceMetadata {
                kernel_profile_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/profile",
                    32,
                ),
                verifier_policy_fingerprint: verifier_policy,
                checker_schema_version: schema_version,
                certificate_format: Some("atp-cert-v1".to_owned()),
                accepted_result_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/accepted-result",
                    33,
                ),
                used_axioms_hash: Some(hash_ref(
                    ArtifactHashClass::Diagnostic,
                    "mizar-kernel/used-axioms",
                    34,
                )),
            },
        }],
        diagnostics: vec![
            ArtifactDiagnostic {
                diagnostic_id: "diag-1".to_owned(),
                code: "MZ1001".to_owned(),
                severity: DiagnosticSeverity::Warning,
                primary_range: Some(range(12, 18)),
                message_key: "proof.used_simplification".to_owned(),
                rendered_message: "simplification used".to_owned(),
                related: vec![
                    DiagnosticRelated {
                        source_range: range(1, 2),
                        message_key: "context.first".to_owned(),
                        rendered_message: "first context".to_owned(),
                    },
                    DiagnosticRelated {
                        source_range: range(3, 4),
                        message_key: "context.second".to_owned(),
                        rendered_message: "second context".to_owned(),
                    },
                ],
                explanation_ref: Some(hash_ref(
                    ArtifactHashClass::Diagnostic,
                    "mizar-diagnostics/explanation",
                    35,
                )),
            },
            ArtifactDiagnostic {
                diagnostic_id: "diag-2".to_owned(),
                code: "MZ2001".to_owned(),
                severity: DiagnosticSeverity::Info,
                primary_range: None,
                message_key: "artifact.note".to_owned(),
                rendered_message: "artifact note".to_owned(),
                related: Vec::new(),
                explanation_ref: None,
            },
        ],
        provenance: BuildProvenance {
            toolchain: "mizar-evo-test".to_owned(),
            language_edition: "2026".to_owned(),
            lockfile_hash: hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 36),
            verifier_config_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-build/verifier-config",
                37,
            ),
            dependency_artifact_hashes: vec![DependencyArtifactHash {
                module: dependency_module,
                interface_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-artifact/module-summary",
                    38,
                ),
                implementation_hash: Some(hash_ref(
                    ArtifactHashClass::Implementation,
                    "mizar-artifact/verified-artifact",
                    39,
                )),
                artifact_hash: Some(hash_ref(
                    ArtifactHashClass::Artifact,
                    "mizar-artifact/file",
                    40,
                )),
            }],
            cache_key: Some("cache-key-1".to_owned()),
        },
    };
    artifact.refresh_hashes().expect("sample hashes");
    artifact
}

fn sample_artifact_with_extra_ordering_items() -> VerifiedArtifact {
    let mut artifact = sample_artifact();
    let mut extra_obligation = artifact.obligations[0].clone();
    extra_obligation.obligation_id = "obl-0".to_owned();
    extra_obligation.source_range = range(6, 9);
    extra_obligation.accepted_witness_obligation_id = Some("obl-0".to_owned());
    extra_obligation.obligation_fingerprint =
        hash_ref(ArtifactHashClass::Interface, "mizar-proof/obligation", 110);

    let mut extra_witness = artifact.proof_witnesses[0].clone();
    extra_witness.obligation_id = "obl-0".to_owned();
    extra_witness.obligation_fingerprint = extra_obligation.obligation_fingerprint.clone();
    extra_witness.witness_path = "proof-witnesses/hidden/obl-0.json".to_owned();
    extra_witness.witness_artifact_hash =
        hash_ref(ArtifactHashClass::Artifact, "mizar-proof/witness-file", 111);

    artifact.obligations.push(extra_obligation);
    artifact.proof_witnesses.push(extra_witness);
    artifact
        .provenance
        .dependency_artifact_hashes
        .push(DependencyArtifactHash {
            module: identity("dep", "articles/alpha", "2026"),
            interface_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-artifact/module-summary",
                112,
            ),
            implementation_hash: Some(hash_ref(
                ArtifactHashClass::Implementation,
                "mizar-artifact/verified-artifact",
                113,
            )),
            artifact_hash: Some(hash_ref(
                ArtifactHashClass::Artifact,
                "mizar-artifact/file",
                114,
            )),
        });
    artifact
}

fn identity(package_id: &str, module_path: &str, edition: &str) -> ModuleSummaryIdentity {
    ModuleSummaryIdentity {
        package_id: package_id.to_owned(),
        package_version: Some("1.0.0".to_owned()),
        lockfile_identity: Some("lock".to_owned()),
        module_path: module_path.to_owned(),
        language_edition: edition.to_owned(),
    }
}

fn range(start_byte: u64, end_byte: u64) -> SourceRangeSummary {
    SourceRangeSummary {
        start_byte,
        end_byte,
    }
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn hash_ref(class: ArtifactHashClass, family: &str, seed: u8) -> ArtifactHashRef {
    ArtifactHashRef::new(class, family, SchemaVersion::new(1, 0), hash(seed))
}

fn artifact_framed_hash_string_for_test(
    family: &str,
    class: ArtifactHashClass,
    schema_version: SchemaVersion,
    digest: Hash,
) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        crate::store::ARTIFACT_HASH_CONSTRUCTION,
        super::artifact_hash_class_string(class),
        family,
        schema_version,
        super::lower_hex_hash(digest)
    )
}

fn set_field(json: &mut CanonicalJson, field: &str, value: CanonicalJson) {
    object_mut(json).insert(field.to_owned(), value);
}

fn set_object_field(json: &mut CanonicalJson, field: &str, value: CanonicalJson) {
    object_mut(json).insert(field.to_owned(), value);
}

fn set_nested_range(json: &mut CanonicalJson, field: &str, start: i64, end: i64) {
    let range = object_field_mut(json, field);
    let fields = object_mut(range);
    fields.insert("start_byte".to_owned(), CanonicalJson::integer(start));
    fields.insert("end_byte".to_owned(), CanonicalJson::integer(end));
}

fn object_field_mut<'a>(json: &'a mut CanonicalJson, field: &str) -> &'a mut CanonicalJson {
    object_mut(json).get_mut(field).expect("object field")
}

fn array_field_mut<'a>(json: &'a mut CanonicalJson, field: &str) -> &'a mut Vec<CanonicalJson> {
    array_mut(object_field_mut(json, field))
}

fn array_object_field_mut<'a>(
    json: &'a mut CanonicalJson,
    field: &str,
) -> &'a mut Vec<CanonicalJson> {
    array_mut(object_field_mut(json, field))
}

fn object_mut(json: &mut CanonicalJson) -> &mut BTreeMap<String, CanonicalJson> {
    let CanonicalJson::Object(fields) = json else {
        panic!("expected object");
    };
    fields
}

fn array_mut(json: &mut CanonicalJson) -> &mut Vec<CanonicalJson> {
    let CanonicalJson::Array(values) = json else {
        panic!("expected array");
    };
    values
}

fn provenance_field_mut(json: &mut CanonicalJson) -> &mut CanonicalJson {
    object_field_mut(json, "provenance")
}

fn first_dependency_hash_mut(json: &mut CanonicalJson) -> &mut CanonicalJson {
    array_object_field_mut(provenance_field_mut(json), "dependency_artifact_hashes")
        .first_mut()
        .expect("dependency artifact hash entry")
}

fn mutate_provenance_toolchain(json: &mut CanonicalJson) {
    set_object_field(
        provenance_field_mut(json),
        "toolchain",
        CanonicalJson::string("changed-toolchain"),
    );
}

fn mutate_provenance_language_edition(json: &mut CanonicalJson) {
    set_object_field(
        provenance_field_mut(json),
        "language_edition",
        CanonicalJson::string("2027"),
    );
}

fn mutate_provenance_lockfile_hash(json: &mut CanonicalJson) {
    set_object_field(
        provenance_field_mut(json),
        "lockfile_hash",
        CanonicalJson::string(
            hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 101)
                .to_artifact_hash_string(),
        ),
    );
}

fn mutate_provenance_verifier_config_hash(json: &mut CanonicalJson) {
    set_object_field(
        provenance_field_mut(json),
        "verifier_config_hash",
        CanonicalJson::string(
            hash_ref(
                ArtifactHashClass::Interface,
                "mizar-build/verifier-config",
                102,
            )
            .to_artifact_hash_string(),
        ),
    );
}

fn mutate_dependency_module(json: &mut CanonicalJson) {
    set_object_field(
        object_field_mut(first_dependency_hash_mut(json), "module"),
        "module_path",
        CanonicalJson::string("articles/changed-base"),
    );
}

fn mutate_dependency_interface_hash(json: &mut CanonicalJson) {
    set_object_field(
        first_dependency_hash_mut(json),
        "interface_hash",
        CanonicalJson::string(
            hash_ref(
                ArtifactHashClass::Interface,
                "mizar-artifact/module-summary",
                103,
            )
            .to_artifact_hash_string(),
        ),
    );
}

fn mutate_dependency_implementation_hash(json: &mut CanonicalJson) {
    set_object_field(
        first_dependency_hash_mut(json),
        "implementation_hash",
        CanonicalJson::string(
            hash_ref(
                ArtifactHashClass::Implementation,
                "mizar-artifact/verified-artifact",
                104,
            )
            .to_artifact_hash_string(),
        ),
    );
}

fn mutate_dependency_artifact_hash(json: &mut CanonicalJson) {
    set_object_field(
        first_dependency_hash_mut(json),
        "artifact_hash",
        CanonicalJson::string(
            hash_ref(ArtifactHashClass::Artifact, "mizar-artifact/file", 105)
                .to_artifact_hash_string(),
        ),
    );
}

fn duplicate_array_item(values: &mut Vec<CanonicalJson>, index: usize) {
    let duplicate = values[index].clone();
    values.insert(index + 1, duplicate);
}
