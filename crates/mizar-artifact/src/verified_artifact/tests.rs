use std::collections::BTreeMap;

use mizar_session::Hash;

use super::{
    ArtifactDiagnostic, BuildProvenance, DependencyArtifactHash, DiagnosticRelated,
    DiagnosticSeverity, ExportProofStatus, ExportVisibility, ExpressionMetadata,
    ObligationMetadata, ObligationStatus, OverloadMetadata, StatementMetadata,
    VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifact, VerifiedArtifactError,
    VerifiedArtifactReadOptions, VerifiedExport, artifact_hash_excluded_paths,
    builtin_verified_artifact_json, current_schema_version, implementation_hash_input_json,
    interface_hash_input_json, read_builtin_verified_artifact, read_verified_artifact,
    read_verified_artifact_for_version, verified_artifact_hash_string, verified_artifact_json,
    write_verified_artifact,
};
use crate::{
    module_summary::{ModuleSummaryIdentity, SourceRangeSummary},
    proof_witness::{
        EvidenceKind, KernelAcceptanceMetadata, ProofStatus as WitnessProofStatus, ProofWitnessRef,
        current_schema_version as proof_witness_schema_version,
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
    set_field(&mut json, "schema_version", CanonicalJson::string("1.2"));

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

    for (field, value) in [
        ("evidence_schema_version", CanonicalJson::string("1.1")),
        (
            "target_binding_hash",
            CanonicalJson::string(
                hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-vc/kernel-target-binding",
                    96,
                )
                .to_artifact_hash_string(),
            ),
        ),
        (
            "formula_evidence_hash",
            CanonicalJson::string(
                hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/formula-evidence",
                    97,
                )
                .to_artifact_hash_string(),
            ),
        ),
        (
            "substitution_evidence_hash",
            CanonicalJson::string(
                hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/substitution-evidence",
                    98,
                )
                .to_artifact_hash_string(),
            ),
        ),
        (
            "provenance_hash",
            CanonicalJson::string(
                hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/evidence-provenance",
                    99,
                )
                .to_artifact_hash_string(),
            ),
        ),
        (
            "formula_context_hash",
            CanonicalJson::string(
                hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/formula-context",
                    100,
                )
                .to_artifact_hash_string(),
            ),
        ),
        (
            "accepted_result_hash",
            CanonicalJson::string(
                hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/accepted-result",
                    101,
                )
                .to_artifact_hash_string(),
            ),
        ),
    ] {
        let mut json = sample_json();
        let witness = array_field_mut(&mut json, "proof_witnesses")
            .first_mut()
            .unwrap();
        set_object_field(object_field_mut(witness, "kernel_acceptance"), field, value);
        assert!(
            matches!(
                read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
                Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
            ),
            "{field}"
        );
    }

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
    let domain = artifact_hash_domain(
        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
        sample_artifact().schema_version,
    );
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
    let schema_version = SchemaVersion::new(1, 0);
    let witness_schema_version = proof_witness_schema_version();
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
        statements: Vec::new(),
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
            schema_version: witness_schema_version,
            obligation_id: "obl-1".to_owned(),
            obligation_fingerprint,
            proof_status: WitnessProofStatus::KernelVerified,
            evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
            witness_path: "proof-witnesses/hidden/obl-1.json".to_owned(),
            witness_artifact_hash: hash_ref(
                ArtifactHashClass::Artifact,
                "mizar-kernel/formula-evidence-witness",
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
                evidence_schema_version: schema_version,
                target_binding_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-vc/kernel-target-binding",
                    33,
                ),
                formula_evidence_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/formula-evidence",
                    34,
                ),
                substitution_evidence_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/substitution-evidence",
                    35,
                ),
                provenance_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/evidence-provenance",
                    36,
                ),
                formula_context_hash: Some(hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/formula-context",
                    37,
                )),
                accepted_result_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/accepted-result",
                    38,
                ),
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
    extra_witness.witness_artifact_hash = hash_ref(
        ArtifactHashClass::Artifact,
        "mizar-kernel/formula-evidence-witness",
        111,
    );

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

#[test]
fn statement_profile_exact_fields_round_trip_and_row_normalization() {
    assert_eq!(current_schema_version(), SchemaVersion::new(1, 1));
    let artifact = sample_statement_artifact();
    let json = verified_artifact_json(&artifact).unwrap();
    let CanonicalJson::Object(fields) = &json else {
        panic!("object")
    };
    assert_eq!(
        fields.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "diagnostics",
            "exports",
            "expressions",
            "implementation_hash",
            "interface_hash",
            "module",
            "obligations",
            "proof_witnesses",
            "provenance",
            "schema_version",
            "source_file",
            "source_hash",
            "statements",
            "verified_at",
        ]
    );
    let expected_row = CanonicalJson::object([
        ("item", CanonicalJson::string("Hidden.Th1")),
        (
            "source_range",
            CanonicalJson::object([
                ("start_byte", CanonicalJson::integer(10)),
                ("end_byte", CanonicalJson::integer(30)),
            ])
            .unwrap(),
        ),
        (
            "elaborated",
            CanonicalJson::string("for x being set holds x = x"),
        ),
        (
            "reserve_binders",
            CanonicalJson::array([CanonicalJson::string("z"), CanonicalJson::string("a")]),
        ),
        (
            "resolved_symbols",
            CanonicalJson::object([
                ("z", CanonicalJson::string("Base.z")),
                ("a", CanonicalJson::string("Base.a")),
            ])
            .unwrap(),
        ),
    ])
    .unwrap();
    let CanonicalJson::Array(rows) = fields.get("statements").unwrap() else {
        panic!("array")
    };
    assert_eq!(rows[0], expected_row);
    assert_eq!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()).unwrap(),
        artifact
    );
    assert_eq!(
        write_verified_artifact(&artifact).unwrap(),
        canonical_json_string(&json).into_bytes()
    );

    let mut reordered = artifact.clone();
    reordered.statements.reverse();
    reordered.statements[1].resolved_symbols = [
        ("a".to_owned(), "Base.a".to_owned()),
        ("z".to_owned(), "Base.z".to_owned()),
    ]
    .into_iter()
    .collect();
    assert_eq!(verified_artifact_json(&reordered).unwrap(), json);
    assert_eq!(
        reordered.compute_implementation_hash().unwrap(),
        artifact.implementation_hash
    );
    let mut unordered_json = json.clone();
    array_field_mut(&mut unordered_json, "statements").reverse();
    let decoded =
        read_verified_artifact(&unordered_json, VerifiedArtifactReadOptions::default()).unwrap();
    assert_eq!(decoded, artifact);
    assert_eq!(verified_artifact_json(&decoded).unwrap(), json);
    assert_eq!(decoded.statements[0].reserve_binders, ["z", "a"]);
}

#[test]
fn statement_fields_participate_in_implementation_and_artifact_but_not_interface_identity() {
    let artifact = sample_statement_artifact();
    let json = verified_artifact_json(&artifact).unwrap();
    let domain = artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version);
    let artifact_hash = domain.hash(&json, &artifact_hash_excluded_paths());
    for case in 0..8 {
        let mut changed = artifact.clone();
        let row = &mut changed.statements[0];
        match case {
            0 => row.item.push_str("Changed"),
            1 => row.elaborated.push(' '),
            2 => row.source_range.start_byte += 1,
            3 => row.source_range.end_byte += 1,
            4 => row.reserve_binders.reverse(),
            5 => row.reserve_binders.push("b".to_owned()),
            6 => {
                row.resolved_symbols
                    .insert("a".to_owned(), "Other.a".to_owned());
            }
            7 => {
                row.resolved_symbols
                    .insert("b".to_owned(), "Base.b".to_owned());
            }
            _ => unreachable!(),
        }
        changed.refresh_hashes().unwrap();
        assert_eq!(
            changed.interface_hash, artifact.interface_hash,
            "case {case}"
        );
        assert_ne!(
            changed.implementation_hash, artifact.implementation_hash,
            "case {case}"
        );
        assert_ne!(
            domain.hash(
                &verified_artifact_json(&changed).unwrap(),
                &artifact_hash_excluded_paths()
            ),
            artifact_hash,
            "case {case}"
        );
    }
    let mut local = artifact.clone();
    local.verified_at = None;
    local.provenance.cache_key = Some("other-local-cache".to_owned());
    local.refresh_hashes().unwrap();
    assert_eq!(local.interface_hash, artifact.interface_hash);
    assert_eq!(local.implementation_hash, artifact.implementation_hash);
    assert_eq!(
        domain.hash(
            &verified_artifact_json(&local).unwrap(),
            &artifact_hash_excluded_paths()
        ),
        artifact_hash
    );

    let mut empty = artifact;
    empty.statements.clear();
    empty.refresh_hashes().unwrap();
    let json = verified_artifact_json(&empty).unwrap();
    assert_eq!(
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default()).unwrap(),
        empty
    );
    assert_eq!(
        object_mut(&mut json.clone()).get("statements"),
        Some(&CanonicalJson::array([]))
    );
}

#[test]
fn statement_profile_rejects_missing_unknown_and_malformed_fields() {
    let original = verified_artifact_json(&sample_statement_artifact()).unwrap();
    let mut missing = original.clone();
    object_mut(&mut missing).remove("statements");
    assert!(
        matches!(read_verified_artifact(&missing, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::MissingField { path }) if path == "$.statements")
    );
    for field in [
        "item",
        "source_range",
        "elaborated",
        "reserve_binders",
        "resolved_symbols",
    ] {
        let mut json = original.clone();
        object_mut(&mut array_field_mut(&mut json, "statements")[0]).remove(field);
        assert!(
            matches!(read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::MissingField { path }) if path == format!("$.statements[0].{field}")),
            "{field}"
        );
    }
    for (field, value) in [
        ("item", CanonicalJson::Null),
        ("item", CanonicalJson::string("")),
        ("source_range", CanonicalJson::Null),
        ("elaborated", CanonicalJson::integer(1)),
        ("elaborated", CanonicalJson::string("")),
        ("reserve_binders", CanonicalJson::Null),
        (
            "reserve_binders",
            CanonicalJson::array([CanonicalJson::integer(1)]),
        ),
        (
            "reserve_binders",
            CanonicalJson::array([CanonicalJson::string("")]),
        ),
        ("resolved_symbols", CanonicalJson::array([])),
        (
            "resolved_symbols",
            CanonicalJson::object([("a", CanonicalJson::integer(1))]).unwrap(),
        ),
        (
            "resolved_symbols",
            CanonicalJson::object([("a", CanonicalJson::string(""))]).unwrap(),
        ),
        (
            "resolved_symbols",
            CanonicalJson::object([("", CanonicalJson::string("Base.a"))]).unwrap(),
        ),
    ] {
        let mut json = original.clone();
        set_field(
            &mut array_field_mut(&mut json, "statements")[0],
            field,
            value,
        );
        assert!(
            matches!(read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::UnexpectedType { path, .. } | VerifiedArtifactError::InvalidField { path, .. })
            if path.starts_with(&format!("$.statements[0].{field}"))),
            "{field}"
        );
    }
    for value in [
        CanonicalJson::Null,
        CanonicalJson::object([] as [(&str, CanonicalJson); 0]).unwrap(),
    ] {
        let mut json = original.clone();
        set_field(&mut json, "statements", value);
        assert!(
            matches!(read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::UnexpectedType { path, .. }) if path == "$.statements")
        );
    }
    let mut json = original.clone();
    array_field_mut(&mut json, "statements")[0] = CanonicalJson::Null;
    assert!(
        matches!(read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::UnexpectedType { path, .. }) if path == "$.statements[0]")
    );
    let mut json = original.clone();
    set_field(
        &mut array_field_mut(&mut json, "statements")[0],
        "proof_status",
        CanonicalJson::string("accepted"),
    );
    assert!(
        matches!(read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::UnknownField { path, field }) if path == "$.statements[0]" && field == "proof_status")
    );
    let mut json = original.clone();
    let row = &mut array_field_mut(&mut json, "statements")[0];
    set_field(
        object_field_mut(row, "source_range"),
        "arena",
        CanonicalJson::integer(3),
    );
    assert!(
        matches!(read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::UnknownField { path, .. }) if path == "$.statements[0].source_range")
    );
    let mut json = original.clone();
    set_nested_range(
        &mut array_field_mut(&mut json, "statements")[0],
        "source_range",
        30,
        10,
    );
    assert!(
        matches!(read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::InvalidField { path, .. }) if path == "$.statements[0].source_range")
    );
    let mut json = original.clone();
    duplicate_array_item(array_field_mut(&mut json, "statements"), 0);
    assert!(
        matches!(read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::DuplicateEntry { path, .. }) if path == "$.statements")
    );
    let mut json = original;
    set_field(
        &mut array_field_mut(&mut json, "statements")[0],
        "reserve_binders",
        CanonicalJson::array([CanonicalJson::string("x"), CanonicalJson::string("x")]),
    );
    assert!(
        matches!(read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::DuplicateEntry { path, .. }) if path == "$.statements[0].reserve_binders")
    );
}

#[test]
fn statement_writer_validates_rows_and_legacy_nonempty_vectors() {
    for case in 0..9 {
        let mut artifact = sample_statement_artifact();
        let row = &mut artifact.statements[0];
        match case {
            0 => row.item.clear(),
            1 => row.elaborated.clear(),
            2 => row.source_range = range(30, 10),
            3 => row.reserve_binders.push(String::new()),
            4 => row.reserve_binders.push("z".to_owned()),
            5 => {
                row.resolved_symbols
                    .insert(String::new(), "Base.a".to_owned());
            }
            6 => {
                row.resolved_symbols.insert("a".to_owned(), String::new());
            }
            7 => artifact.statements.push(artifact.statements[0].clone()),
            8 => artifact.statements[1].source_range = artifact.statements[0].source_range,
            _ => unreachable!(),
        }
        if case == 8 {
            artifact.refresh_hashes().unwrap();
            assert!(
                verified_artifact_json(&artifact).is_ok(),
                "distinct items can share ranges"
            );
        } else {
            assert!(
                matches!(
                    verified_artifact_json(&artifact),
                    Err(VerifiedArtifactError::InvalidField { .. }
                        | VerifiedArtifactError::DuplicateEntry { .. })
                ),
                "case {case}"
            );
        }
    }
    for version in [SchemaVersion::new(1, 0), SchemaVersion::new(2, 0)] {
        let mut artifact = sample_statement_artifact();
        artifact.schema_version = version;
        let result = if version.major() == 2 {
            builtin_verified_artifact_json(&artifact)
        } else {
            verified_artifact_json(&artifact)
        };
        assert!(
            matches!(result, Err(VerifiedArtifactError::InvalidField { path, .. }) if path == "$.statements")
        );
        assert!(
            matches!(artifact.compute_implementation_hash(), Err(VerifiedArtifactError::InvalidField { path, .. }) if path == "$.statements")
        );
    }
}

#[test]
fn statement_version_bounds_and_legacy_field_priority_remain_closed() {
    let json = verified_artifact_json(&sample_statement_artifact()).unwrap();
    assert!(matches!(
        read_verified_artifact_for_version(
            &json,
            VerifiedArtifactReadOptions::default(),
            SchemaVersion::new(1, 0)
        ),
        Err(VerifiedArtifactError::SchemaVersion(
            SchemaVersionError::MinorTooNew { .. }
        ))
    ));
    for version in ["1.2", "2.1", "3.0"] {
        let mut value = json.clone();
        set_field(&mut value, "schema_version", CanonicalJson::string(version));
        set_field(&mut value, "aaa_unknown", CanonicalJson::Null);
        assert!(matches!(
            read_verified_artifact(&value, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::SchemaVersion(_))
        ));
        assert!(matches!(
            read_builtin_verified_artifact(&value, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::SchemaVersion(_))
        ));
    }
    for version in [SchemaVersion::new(1, 0), SchemaVersion::new(2, 0)] {
        let mut artifact = sample_artifact();
        artifact.schema_version = version;
        artifact.refresh_hashes().unwrap();
        let mut json = if version.major() == 2 {
            builtin_verified_artifact_json(&artifact).unwrap()
        } else {
            verified_artifact_json(&artifact).unwrap()
        };
        assert!(!object_mut(&mut json).contains_key("statements"));
        let decoded = if version.major() == 2 {
            read_builtin_verified_artifact(&json, VerifiedArtifactReadOptions::default()).unwrap()
        } else {
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()).unwrap()
        };
        assert_eq!(decoded, artifact);
        set_field(&mut json, "statements", CanonicalJson::array([]));
        object_mut(&mut json).remove("module");
        let result = if version.major() == 2 {
            read_builtin_verified_artifact(&json, VerifiedArtifactReadOptions::default())
        } else {
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default())
        };
        assert!(
            matches!(result, Err(VerifiedArtifactError::UnknownField { path, field }) if path == "$" && field == "statements")
        );
    }
    let mut malformed = json.clone();
    object_mut(&mut malformed).remove("source_file");
    set_field(&mut malformed, "statements", CanonicalJson::Null);
    assert!(
        matches!(read_verified_artifact(&malformed, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::MissingField { path }) if path == "$.source_file")
    );
    let mut malformed = json;
    object_mut(&mut malformed).remove("provenance");
    object_mut(&mut malformed).remove("statements");
    assert!(
        matches!(read_verified_artifact(&malformed, VerifiedArtifactReadOptions::default()),
        Err(VerifiedArtifactError::MissingField { path }) if path == "$.provenance")
    );
}

fn sample_statement_artifact() -> VerifiedArtifact {
    let mut artifact = sample_artifact();
    artifact.schema_version = SchemaVersion::new(1, 1);
    artifact.statements = vec![
        StatementMetadata {
            item: "Hidden.Th1".to_owned(),
            source_range: range(10, 30),
            elaborated: "for x being set holds x = x".to_owned(),
            reserve_binders: vec!["z".to_owned(), "a".to_owned()],
            resolved_symbols: [
                ("z".to_owned(), "Base.z".to_owned()),
                ("a".to_owned(), "Base.a".to_owned()),
            ]
            .into_iter()
            .collect(),
        },
        StatementMetadata {
            item: "Hidden.Th2".to_owned(),
            source_range: range(40, 60),
            elaborated: "for y being set holds y = y".to_owned(),
            reserve_binders: Vec::new(),
            resolved_symbols: BTreeMap::new(),
        },
    ];
    artifact.refresh_hashes().unwrap();
    artifact
}

#[test]
fn legacy_profiles_keep_independently_mapped_bytes_and_hash_projections() {
    for version in [SchemaVersion::new(1, 0), SchemaVersion::new(2, 0)] {
        let mut artifact = sample_artifact();
        artifact.schema_version = version;
        artifact.exports.clear();
        artifact.expressions.clear();
        artifact.obligations.clear();
        artifact.proof_witnesses.clear();
        artifact.diagnostics.clear();
        artifact.provenance.dependency_artifact_hashes.clear();
        let module = CanonicalJson::object([
            ("package_id", CanonicalJson::string("pkg")),
            ("package_version", CanonicalJson::string("1.0.0")),
            ("lockfile_identity", CanonicalJson::string("lock")),
            ("module_path", CanonicalJson::string("articles/hidden")),
            ("language_edition", CanonicalJson::string("2026")),
        ])
        .unwrap();
        let expected_interface = CanonicalJson::object([
            ("schema_version", CanonicalJson::string(version.to_string())),
            ("module", module.clone()),
            ("exports", CanonicalJson::array([])),
            ("dependency_artifact_hashes", CanonicalJson::array([])),
        ])
        .unwrap();
        let provenance = CanonicalJson::object([
            ("toolchain", CanonicalJson::string("mizar-evo-test")),
            ("language_edition", CanonicalJson::string("2026")),
            (
                "lockfile_hash",
                CanonicalJson::string(artifact.provenance.lockfile_hash.to_artifact_hash_string()),
            ),
            (
                "verifier_config_hash",
                CanonicalJson::string(
                    artifact
                        .provenance
                        .verifier_config_hash
                        .to_artifact_hash_string(),
                ),
            ),
            ("dependency_artifact_hashes", CanonicalJson::array([])),
        ])
        .unwrap();
        let expected_implementation = CanonicalJson::object([
            ("schema_version", CanonicalJson::string(version.to_string())),
            ("module", module),
            ("source_file", CanonicalJson::string("articles/hidden.miz")),
            (
                "source_hash",
                CanonicalJson::string(format!("mizar-session/hash-text/v1:{}", "01".repeat(32))),
            ),
            ("exports", CanonicalJson::array([])),
            ("expressions", CanonicalJson::array([])),
            ("obligations", CanonicalJson::array([])),
            ("proof_witnesses", CanonicalJson::array([])),
            ("diagnostics", CanonicalJson::array([])),
            ("provenance", provenance),
        ])
        .unwrap();
        let interface_hash = CanonicalHashDomain::new(
            HashClass::Interface,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            version,
        )
        .hash(&expected_interface, &[]);
        let implementation_hash = CanonicalHashDomain::new(
            HashClass::Implementation,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            version,
        )
        .hash(&expected_implementation, &[]);
        artifact.refresh_hashes().unwrap();
        assert_eq!(artifact.interface_hash, interface_hash);
        assert_eq!(artifact.implementation_hash, implementation_hash);
        assert_eq!(
            interface_hash_input_json(&artifact).unwrap(),
            expected_interface
        );
        assert_eq!(
            implementation_hash_input_json(&artifact).unwrap(),
            expected_implementation
        );
        let mut expected = expected_implementation;
        set_field(
            &mut expected,
            "interface_hash",
            CanonicalJson::string(artifact_framed_hash_string_for_test(
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                ArtifactHashClass::Interface,
                version,
                interface_hash,
            )),
        );
        set_field(
            &mut expected,
            "implementation_hash",
            CanonicalJson::string(artifact_framed_hash_string_for_test(
                VERIFIED_ARTIFACT_SCHEMA_FAMILY,
                ArtifactHashClass::Implementation,
                version,
                implementation_hash,
            )),
        );
        set_field(
            &mut expected,
            "verified_at",
            CanonicalJson::string("2026-06-22T14:03:05Z"),
        );
        set_field(
            object_field_mut(&mut expected, "provenance"),
            "cache_key",
            CanonicalJson::string("cache-key-1"),
        );
        let actual = if version.major() == 2 {
            builtin_verified_artifact_json(&artifact).unwrap()
        } else {
            verified_artifact_json(&artifact).unwrap()
        };
        assert_eq!(actual, expected);
        assert_eq!(
            canonical_json_string(&actual).into_bytes(),
            canonical_json_string(&expected).into_bytes()
        );
        let domain = artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, version);
        assert_eq!(
            domain.hash(&actual, &artifact_hash_excluded_paths()),
            domain.hash(&expected, &artifact_hash_excluded_paths())
        );
    }
}
