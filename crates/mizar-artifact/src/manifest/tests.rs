use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    module_summary::{
        DependencyInterfaceRef, ExportedLabelSummary, ExportedSymbolSummary,
        LexicalContributionSummary, ModuleLexicalSummary, ModuleReexportSummary, ModuleSummary,
        ProofStatusSummary, SourceRangeSummary,
        current_schema_version as module_summary_schema_version, module_summary_json,
    },
    proof_witness::{
        EvidenceKind, KernelAcceptanceMetadata, ProofStatus as WitnessProofStatus, ProofWitnessRef,
        current_schema_version as proof_witness_schema_version,
    },
    registration_summary::{
        ActivatedRegistrationSummary, DependencyRegistrationRef, RegistrationAcceptedStatus,
        RegistrationContributionKind, RegistrationContributionSummary, RegistrationKind,
        RegistrationPatternSummary, RegistrationSummary, RegistrationTraceArtifactRef,
        RegistrationTraceKind, RegistrationVisibility,
        current_schema_version as registration_summary_schema_version, registration_summary_json,
    },
    store::{artifact_hash_domain, canonical_json_string, write_published_artifact},
    verified_artifact::{
        ArtifactDiagnostic, BuildProvenance, DependencyArtifactHash, DiagnosticRelated,
        DiagnosticSeverity, ExportProofStatus, ExportVisibility, ExpressionMetadata,
        ObligationMetadata, ObligationStatus, OverloadMetadata, VerifiedArtifact, VerifiedExport,
        verified_artifact_json,
    },
};

use super::*;

static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(1);

#[test]
fn manifest_round_trips_through_sorted_canonical_json() {
    let module_a = identity("pkg", "articles/a", "2026");
    let module_b = identity("pkg", "articles/b", "2026");
    let mut manifest = sample_manifest();
    manifest.modules = vec![
        fake_entry(module_b.clone(), "artifacts/b.json", 20),
        fake_entry(module_a.clone(), "artifacts/a.json", 10),
    ];
    manifest.development_artifacts = vec![
        development_entry("trace", "dev/z.json", Some(module_b)),
        development_entry("trace", "dev/a.json", Some(module_a.clone())),
    ];

    let json = artifact_manifest_json(&manifest).expect("canonical manifest JSON");
    let text = canonical_json_string(&json);
    assert!(
        text.find("\"module_path\":\"articles/a\"")
            .expect("module a")
            < text
                .find("\"module_path\":\"articles/b\"")
                .expect("module b")
    );

    let read = read_artifact_manifest(&json, ArtifactManifestReadOptions::default())
        .expect("read sorted manifest");
    assert_eq!(read.modules[0].module, module_a);
    assert_eq!(read.development_artifacts[0].path, "dev/a.json");
    assert_eq!(read.development_artifacts[1].path, "dev/z.json");
}

#[test]
fn manifest_writer_is_deterministic_for_identical_inputs() {
    let module_a = identity("pkg", "articles/a", "2026");
    let module_b = identity("pkg", "articles/b", "2026");
    let mut manifest = sample_manifest();
    manifest.modules = vec![
        fake_entry(module_b.clone(), "artifacts/b.json", 20),
        fake_entry(module_a.clone(), "artifacts/a.json", 10),
    ];
    manifest.development_artifacts = vec![
        development_entry("trace", "dev/z.json", Some(module_b)),
        development_entry("trace", "dev/a.json", Some(module_a)),
    ];
    let domain = manifest_hash_domain(manifest.schema_version);
    let first_json = artifact_manifest_json(&manifest).expect("first canonical manifest JSON");
    let first_bytes = write_artifact_manifest(&manifest).expect("first canonical bytes");
    let first_hash = domain.hash(&first_json, &[]);

    for _ in 0..3 {
        let json = artifact_manifest_json(&manifest).expect("repeated canonical manifest JSON");
        assert_eq!(json, first_json);
        assert_eq!(canonical_json_string(&json).into_bytes(), first_bytes);
        assert_eq!(
            write_artifact_manifest(&manifest).expect("repeated canonical bytes"),
            first_bytes
        );
        assert_eq!(domain.hash(&json, &[]), first_hash);
    }
}

#[test]
fn reader_rejects_unsorted_and_duplicate_manifest_collections() {
    let module_a = identity("pkg", "articles/a", "2026");
    let module_b = identity("pkg", "articles/b", "2026");
    let entry_a = fake_entry(module_a.clone(), "artifacts/a.json", 10);
    let entry_b = fake_entry(module_b, "artifacts/b.json", 20);
    let manifest = sample_manifest();
    let unsorted = manifest_json_with_modules(&manifest, [&entry_b, &entry_a]);
    let error = read_artifact_manifest(&unsorted, ArtifactManifestReadOptions::default())
        .expect_err("unsorted modules must fail");
    assert!(matches!(
        error,
        ManifestError::UnsortedCollection { path } if path == "$.modules"
    ));

    let duplicate = manifest_json_with_modules(&manifest, [&entry_a, &entry_a]);
    let error = read_artifact_manifest(&duplicate, ArtifactManifestReadOptions::default())
        .expect_err("duplicate modules must fail");
    assert!(matches!(
        error,
        ManifestError::DuplicateEntry { path, .. } if path == "$.modules"
    ));

    let witness_a = manifest_witness("obl-a", 30);
    let witness_b = manifest_witness("obl-b", 31);
    let mut entry = entry_a.clone();
    entry.proof_witnesses = vec![witness_b.clone(), witness_a.clone()];
    let unsorted = manifest_json_with_modules(&manifest, [&entry]);
    let error = read_artifact_manifest(&unsorted, ArtifactManifestReadOptions::default())
        .expect_err("unsorted proof witnesses must fail");
    assert!(matches!(
        error,
        ManifestError::UnsortedCollection { path } if path == "$.modules[0].proof_witnesses"
    ));

    let mut entry = entry_a.clone();
    entry.proof_witnesses = vec![witness_a.clone(), witness_a];
    let duplicate = manifest_json_with_modules(&manifest, [&entry]);
    let error = read_artifact_manifest(&duplicate, ArtifactManifestReadOptions::default())
        .expect_err("duplicate proof witnesses must fail");
    assert!(matches!(
        error,
        ManifestError::DuplicateEntry { path, .. } if path == "$.modules[0].proof_witnesses"
    ));

    let mut unsorted_development = sample_manifest();
    unsorted_development.development_artifacts = vec![
        development_entry("trace", "dev/z.json", None),
        development_entry("trace", "dev/a.json", None),
    ];
    let unsorted = manifest_json_with_development_artifacts(&unsorted_development);
    let error = read_artifact_manifest(&unsorted, ArtifactManifestReadOptions::default())
        .expect_err("unsorted development artifacts must fail");
    assert!(matches!(
        error,
        ManifestError::UnsortedCollection { path } if path == "$.development_artifacts"
    ));

    let mut duplicate_development = sample_manifest();
    duplicate_development.development_artifacts = vec![
        development_entry("trace", "dev/a.json", None),
        development_entry("trace", "dev/a.json", Some(module_a)),
    ];
    let duplicate = manifest_json_with_development_artifacts(&duplicate_development);
    let error = read_artifact_manifest(&duplicate, ArtifactManifestReadOptions::default())
        .expect_err("duplicate development artifacts must fail");
    assert!(matches!(
        error,
        ManifestError::DuplicateEntry { path, .. } if path == "$.development_artifacts"
    ));
}

#[test]
fn manifest_commit_publishes_old_or_new_and_ignores_orphans() {
    let root = TestArtifactRoot::new();
    let module_a = identity("pkg", "articles/a", "2026");
    let module_b = identity("pkg", "articles/b", "2026");
    let artifact_a = sample_verified_artifact(module_a.clone(), "articles/a.miz", hash(1), false);
    let artifact_b = sample_verified_artifact(module_b.clone(), "articles/b.miz", hash(2), false);
    let entry_a = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact_a);
    let entry_b = publish_verified_artifact(root.path(), "artifacts/b.json", &artifact_b);
    let mut initial = sample_manifest();
    initial.modules = vec![entry_a];
    write_manifest_file(root.path(), &initial).expect("initial manifest");

    fs::write(
        root.path()
            .join(".mizar-artifact-tmp-artifact-manifest.json"),
        b"{\"not\":\"a manifest\"}\n",
    )
    .expect("orphan temporary-like file");

    let mut transaction =
        ManifestTransaction::begin(root.path(), sample_manifest(), Some("snap-1".to_owned()))
            .expect("begin");
    transaction.stage_module(entry_b).expect("stage module b");
    let before = read_manifest_file(root.path(), ManifestFileReadOptions::default())
        .expect("read before commit");
    assert_eq!(before.manifest.modules.len(), 1);

    let freshness = |guard: Option<&str>| guard == Some("snap-1");
    let commit = transaction
        .commit(ManifestCommitOptions {
            freshness_check: Some(&freshness),
        })
        .expect("commit");
    assert_eq!(commit.manifest.modules.len(), 2);
    let after = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect("read after commit");
    assert_eq!(after.manifest.modules.len(), 2);
}

#[test]
fn commit_returns_manifest_in_canonical_order_after_merge() {
    let root = TestArtifactRoot::new();
    let module_a = identity("pkg", "articles/a", "2026");
    let module_b = identity("pkg", "articles/b", "2026");
    let artifact_a = sample_verified_artifact(module_a.clone(), "articles/a.miz", hash(1), false);
    let artifact_b = sample_verified_artifact(module_b.clone(), "articles/b.miz", hash(2), false);
    let entry_a = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact_a);
    let entry_b = publish_verified_artifact(root.path(), "artifacts/b.json", &artifact_b);
    let mut initial = sample_manifest();
    initial.modules = vec![entry_b];
    write_manifest_file(root.path(), &initial).expect("initial manifest");

    let mut transaction =
        ManifestTransaction::begin(root.path(), sample_manifest(), None).expect("begin");
    transaction.stage_module(entry_a).expect("stage module a");
    let commit = transaction
        .commit(ManifestCommitOptions::default())
        .expect("commit");

    assert_eq!(commit.manifest.modules[0].module, module_a);
    assert_eq!(commit.manifest.modules[1].module, module_b);
}

#[test]
fn replayed_identical_commit_is_idempotent() {
    let root = TestArtifactRoot::new();
    let module = identity("pkg", "articles/a", "2026");
    let artifact = sample_verified_artifact(module, "articles/a.miz", hash(1), false);
    let entry = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact);

    let mut first =
        ManifestTransaction::begin(root.path(), sample_manifest(), None).expect("begin first");
    first.stage_module(entry.clone()).expect("stage first");
    let first_commit = first
        .commit(ManifestCommitOptions::default())
        .expect("first commit");
    let first_text =
        fs::read_to_string(root.path().join(ARTIFACT_MANIFEST_PATH)).expect("first manifest text");

    let mut replay =
        ManifestTransaction::begin(root.path(), sample_manifest(), None).expect("begin replay");
    replay.stage_module(entry).expect("stage replay");
    let replay_commit = replay
        .commit(ManifestCommitOptions::default())
        .expect("replay commit");
    let replay_text =
        fs::read_to_string(root.path().join(ARTIFACT_MANIFEST_PATH)).expect("replay manifest text");

    assert_eq!(
        first_commit.write.artifact_hash,
        replay_commit.write.artifact_hash
    );
    assert_eq!(first_text, replay_text);
}

#[test]
fn manifest_transaction_output_is_deterministic_for_identical_inputs() {
    let module = identity("pkg", "articles/a", "2026");
    let artifact = sample_verified_artifact(module, "articles/a.miz", hash(1), false);

    let root_a = TestArtifactRoot::new();
    let root_b = TestArtifactRoot::new();
    let entry_a = publish_verified_artifact(root_a.path(), "artifacts/a.json", &artifact);
    let entry_b = publish_verified_artifact(root_b.path(), "artifacts/a.json", &artifact);
    assert_eq!(entry_a, entry_b);

    let mut transaction_a =
        ManifestTransaction::begin(root_a.path(), sample_manifest(), None).expect("begin a");
    transaction_a
        .stage_module(entry_a)
        .expect("stage module into first transaction");
    let commit_a = transaction_a
        .commit(ManifestCommitOptions::default())
        .expect("commit a");
    let manifest_text_a = fs::read_to_string(root_a.path().join(ARTIFACT_MANIFEST_PATH))
        .expect("first manifest text");

    let mut transaction_b =
        ManifestTransaction::begin(root_b.path(), sample_manifest(), None).expect("begin b");
    transaction_b
        .stage_module(entry_b)
        .expect("stage module into second transaction");
    let commit_b = transaction_b
        .commit(ManifestCommitOptions::default())
        .expect("commit b");
    let manifest_text_b = fs::read_to_string(root_b.path().join(ARTIFACT_MANIFEST_PATH))
        .expect("second manifest text");

    assert_eq!(commit_a.manifest, commit_b.manifest);
    assert_eq!(commit_a.write.artifact_hash, commit_b.write.artifact_hash);
    assert_eq!(
        write_artifact_manifest(&commit_a.manifest).expect("first committed manifest bytes"),
        write_artifact_manifest(&commit_b.manifest).expect("second committed manifest bytes")
    );
    assert_eq!(manifest_text_a, manifest_text_b);
}

#[test]
fn commit_rejects_changed_base_manifest_hash() {
    let root = TestArtifactRoot::new();
    let module_a = identity("pkg", "articles/a", "2026");
    let module_b = identity("pkg", "articles/b", "2026");
    let artifact_a = sample_verified_artifact(module_a, "articles/a.miz", hash(1), false);
    let artifact_b = sample_verified_artifact(module_b, "articles/b.miz", hash(2), false);
    let entry_a = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact_a);
    let entry_b = publish_verified_artifact(root.path(), "artifacts/b.json", &artifact_b);

    let mut transaction =
        ManifestTransaction::begin(root.path(), sample_manifest(), None).expect("begin");
    transaction
        .stage_module(entry_a.clone())
        .expect("stage stale transaction");

    let mut intervening =
        ManifestTransaction::begin(root.path(), sample_manifest(), None).expect("begin other");
    intervening
        .stage_module(entry_b)
        .expect("stage other transaction");
    intervening
        .commit(ManifestCommitOptions::default())
        .expect("other commit");

    let error = transaction
        .commit(ManifestCommitOptions::default())
        .expect_err("stale base hash must fail");
    assert!(matches!(
        error,
        ManifestError::BaseManifestHashMismatch { .. }
    ));

    let final_manifest =
        read_manifest_file(root.path(), ManifestFileReadOptions::default()).expect("final");
    assert_eq!(final_manifest.manifest.modules.len(), 1);
    assert_ne!(final_manifest.manifest.modules[0].module, entry_a.module);
}

#[test]
fn referenced_verified_artifact_hash_honors_local_field_exclusions() {
    let root = TestArtifactRoot::new();
    let module = identity("pkg", "articles/a", "2026");
    let artifact = sample_verified_artifact(module, "articles/a.miz", hash(1), false);
    let entry = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact);
    let mut changed = artifact.clone();
    changed.verified_at = Some("2026-06-23T00:00:00Z".to_owned());
    changed.provenance.cache_key = Some("cache-key-2".to_owned());
    let changed_entry = publish_verified_artifact(root.path(), "artifacts/a.json", &changed);
    assert_eq!(entry.artifact_hash, changed_entry.artifact_hash);

    let mut manifest = sample_manifest();
    manifest.modules = vec![entry];
    write_manifest_file(root.path(), &manifest).expect("write manifest");
    read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect("manifest validation must use verified artifact exclusions");
}

#[test]
fn referenced_validation_rejects_artifact_hash_and_source_mismatch() {
    let root = TestArtifactRoot::new();
    let module = identity("pkg", "articles/a", "2026");
    let artifact = sample_verified_artifact(module, "articles/a.miz", hash(1), false);
    let mut entry = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact);
    entry.artifact_hash.digest = hash(99);
    let mut manifest = sample_manifest();
    manifest.modules = vec![entry];
    write_manifest_file(root.path(), &manifest).expect("write manifest with bad hash");
    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("artifact hash mismatch must fail");
    assert!(matches!(
        error,
        ManifestError::StoreIo(StoreIoError::ArtifactHashMismatch { .. })
    ));

    let root = TestArtifactRoot::new();
    let module = identity("pkg", "articles/a", "2026");
    let artifact = sample_verified_artifact(module, "articles/a.miz", hash(1), false);
    let mut entry = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact);
    entry.source_hash = hash(88);
    let mut manifest = sample_manifest();
    manifest.modules = vec![entry];
    write_manifest_file(root.path(), &manifest).expect("write manifest with bad source hash");
    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("source hash mismatch must fail");
    assert!(matches!(
        error,
        ManifestError::ReferencedArtifactMismatch { field, .. } if field == "source_hash"
    ));
}

#[test]
fn referenced_validation_rejects_verified_artifact_schema_version_mismatch() {
    let root = TestArtifactRoot::new();
    let module = identity("pkg", "articles/a", "2026");
    let artifact = sample_verified_artifact(module, "articles/a.miz", hash(1), false);
    let mut entry = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact);
    let wrong_version = SchemaVersion::new(1, 1);
    let json = verified_artifact_json(&artifact).expect("verified artifact JSON");
    let excluded = artifact_hash_excluded_paths();
    entry.artifact_hash.schema_version = wrong_version;
    entry.artifact_hash.digest =
        artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, wrong_version).hash(&json, &excluded);
    entry.interface_hash.schema_version = wrong_version;
    entry.implementation_hash.schema_version = wrong_version;
    let mut manifest = sample_manifest();
    manifest.modules = vec![entry];
    write_manifest_file(root.path(), &manifest)
        .expect("write manifest with wrong referenced version");
    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("referenced schema version mismatch must fail");
    assert!(matches!(
        error,
        ManifestError::ReferencedArtifactMismatch { field, .. } if field == "schema_version"
    ));
}

#[test]
fn manifest_requires_exact_witness_coverage() {
    let root = TestArtifactRoot::new();
    let module = identity("pkg", "articles/a", "2026");
    let artifact = sample_verified_artifact(module, "articles/a.miz", hash(1), true);
    let mut entry = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact);
    entry.proof_witnesses.clear();
    let mut manifest = sample_manifest();
    manifest.modules = vec![entry];
    write_manifest_file(root.path(), &manifest).expect("write manifest");
    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("missing witness entry must fail");
    assert!(matches!(
        error,
        ManifestError::WitnessCoverageMismatch { .. }
    ));
}

#[test]
fn witness_file_reachability_is_checked_when_requested() {
    let root = TestArtifactRoot::new();
    let module = identity("pkg", "articles/a", "2026");
    let artifact = sample_verified_artifact(module, "articles/a.miz", hash(1), true);
    let entry = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact);
    let witness_path = entry.proof_witnesses[0].witness_path.clone();
    let mut manifest = sample_manifest();
    manifest.modules = vec![entry];
    write_manifest_file(root.path(), &manifest).expect("write manifest");

    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            validate_witness_files: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("missing witness file must fail when validation is requested");
    assert!(matches!(
        error,
        ManifestError::StoreIo(StoreIoError::Io {
            kind: io::ErrorKind::NotFound,
            ..
        })
    ));

    write_plain_file(
        root.path(),
        &witness_path,
        b"producer-owned witness payload",
    );
    read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            validate_witness_files: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect("existing witness file is reachable");
}

#[test]
fn manifest_validates_module_and_registration_summary_sidecars() {
    let root = TestArtifactRoot::new();
    let module = identity("pkg", "articles/a", "2026");
    let source_hash = hash(1);
    let artifact = sample_verified_artifact(module.clone(), "articles/a.miz", source_hash, false);
    let mut entry = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact);
    let module_summary = sample_module_summary(module.clone(), source_hash);
    let module_summary_write =
        publish_module_summary(root.path(), "summaries/module-a.json", &module_summary);
    let registration_summary = sample_registration_summary(module.clone(), source_hash);
    let registration_summary_write = publish_registration_summary(
        root.path(),
        "summaries/registration-a.json",
        &registration_summary,
    );
    attach_module_summary(&mut entry, "summaries/module-a.json", module_summary_write);
    attach_registration_summary(
        &mut entry,
        "summaries/registration-a.json",
        registration_summary_write,
    );
    let mut manifest = sample_manifest();
    manifest.modules = vec![entry.clone()];
    write_manifest_file(root.path(), &manifest).expect("write manifest");
    read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect("valid sidecars are accepted");

    let mut bad_module_hash = manifest.clone();
    bad_module_hash.modules[0]
        .module_summary_hash
        .as_mut()
        .expect("module summary hash")
        .digest = hash(77);
    write_manifest_file(root.path(), &bad_module_hash).expect("bad module hash manifest");
    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("module summary hash mismatch must fail");
    assert!(matches!(
        error,
        ManifestError::StoreIo(StoreIoError::ArtifactHashMismatch { .. })
    ));

    write_manifest_file(root.path(), &manifest).expect("restore manifest");
    let mut bad_registration_interface = manifest.clone();
    bad_registration_interface.modules[0]
        .registration_interface_hash
        .as_mut()
        .expect("registration interface hash")
        .digest = hash(78);
    write_manifest_file(root.path(), &bad_registration_interface)
        .expect("bad registration interface manifest");
    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("registration interface mismatch must fail");
    assert!(matches!(error, ManifestError::RegistrationSummary { .. }));

    write_manifest_file(root.path(), &manifest).expect("restore manifest");
    let mut bad_source = sample_module_summary(module, hash(99));
    bad_source
        .refresh_interface_hash()
        .expect("refresh bad source summary");
    let bad_source_write =
        publish_module_summary(root.path(), "summaries/module-a.json", &bad_source);
    let mut bad_source_manifest = manifest.clone();
    attach_module_summary(
        &mut bad_source_manifest.modules[0],
        "summaries/module-a.json",
        bad_source_write,
    );
    write_manifest_file(root.path(), &bad_source_manifest).expect("bad source manifest");
    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("module summary source mismatch must fail");
    assert!(matches!(
        error,
        ManifestError::ReferencedArtifactMismatch { field, .. } if field == "source_hash"
    ));

    let wrong_version = SchemaVersion::new(1, 1);
    let mut bad_module_version = bad_source_manifest.clone();
    bad_module_version.modules[0]
        .module_summary_hash
        .as_mut()
        .expect("module summary hash")
        .schema_version = wrong_version;
    bad_module_version.modules[0]
        .module_summary_hash
        .as_mut()
        .expect("module summary hash")
        .digest = module_summary_artifact_hash(&bad_source, wrong_version);
    bad_module_version.modules[0]
        .module_summary_interface_hash
        .as_mut()
        .expect("module summary interface hash")
        .schema_version = wrong_version;
    write_manifest_file(root.path(), &bad_module_version)
        .expect("wrong module summary version manifest");
    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("module summary schema version mismatch must fail");
    assert!(matches!(
        error,
        ManifestError::ReferencedArtifactMismatch { field, .. } if field == "schema_version"
    ));

    publish_module_summary(root.path(), "summaries/module-a.json", &module_summary);
    write_manifest_file(root.path(), &manifest).expect("restore manifest");
    let mut bad_registration_version = manifest.clone();
    bad_registration_version.modules[0]
        .registration_summary_hash
        .as_mut()
        .expect("registration summary hash")
        .schema_version = wrong_version;
    bad_registration_version.modules[0]
        .registration_summary_hash
        .as_mut()
        .expect("registration summary hash")
        .digest = registration_summary_artifact_hash(&registration_summary, wrong_version);
    bad_registration_version.modules[0]
        .registration_interface_hash
        .as_mut()
        .expect("registration interface hash")
        .schema_version = wrong_version;
    write_manifest_file(root.path(), &bad_registration_version)
        .expect("wrong registration summary version manifest");
    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("registration summary schema version mismatch must fail");
    assert!(matches!(
        error,
        ManifestError::ReferencedArtifactMismatch { field, .. } if field == "schema_version"
    ));
}

#[test]
fn manifest_rejects_partial_summaries_and_invalid_development_hash_choices() {
    let mut entry = fake_entry(identity("pkg", "articles/a", "2026"), "artifacts/a.json", 1);
    entry.module_summary_file = Some("summaries/a.json".to_owned());
    let mut manifest = sample_manifest();
    manifest.modules = vec![entry];
    let error = artifact_manifest_json(&manifest).expect_err("partial summary group");
    assert!(matches!(error, ManifestError::PartialOptionalGroup { .. }));

    let mut manifest = sample_manifest();
    manifest.development_artifacts = vec![DevelopmentArtifactEntry {
        kind: "trace".to_owned(),
        path: "dev/trace.json".to_owned(),
        artifact_hash: None,
        diagnostic_hash: None,
        related_module: None,
    }];
    let error = artifact_manifest_json(&manifest).expect_err("missing development hash");
    assert!(matches!(error, ManifestError::InvalidField { .. }));
}

#[test]
fn commit_rejects_obsolete_freshness_guard() {
    let root = TestArtifactRoot::new();
    let module = identity("pkg", "articles/a", "2026");
    let artifact = sample_verified_artifact(module, "articles/a.miz", hash(1), false);
    let entry = publish_verified_artifact(root.path(), "artifacts/a.json", &artifact);
    let mut transaction =
        ManifestTransaction::begin(root.path(), sample_manifest(), Some("snap-old".to_owned()))
            .expect("begin");
    transaction.stage_module(entry).expect("stage");

    let freshness = |guard: Option<&str>| guard == Some("snap-current");
    let error = transaction
        .commit(ManifestCommitOptions {
            freshness_check: Some(&freshness),
        })
        .expect_err("obsolete transaction must fail");
    assert!(matches!(error, ManifestError::ObsoleteSnapshot { .. }));
}

#[test]
fn development_artifact_reachability_is_checked_without_hash_recomputation() {
    let root = TestArtifactRoot::new();
    let mut manifest = sample_manifest();
    manifest.development_artifacts = vec![development_entry("trace", "dev/missing.json", None)];
    write_manifest_file(root.path(), &manifest).expect("write manifest");
    let error = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_development_artifacts: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect_err("missing development artifact must fail");
    assert!(matches!(
        error,
        ManifestError::StoreIo(StoreIoError::Io {
            kind: io::ErrorKind::NotFound,
            ..
        })
    ));

    write_plain_file(
        root.path(),
        "dev/missing.json",
        b"producer-owned development payload",
    );
    read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_development_artifacts: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .expect("development artifact existence is enough until producer hash schemas land");
}

fn sample_manifest() -> ArtifactManifest {
    ArtifactManifest {
        schema_version: current_schema_version(),
        package: PackageIdentity {
            package_id: "pkg".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("lock".to_owned()),
        },
        artifact_root: "target/mizar-artifacts".to_owned(),
        lockfile_hash: hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 1),
        toolchain: "mizar-evo-test".to_owned(),
        language_edition: "2026".to_owned(),
        verifier_config_hash: hash_ref(
            ArtifactHashClass::Interface,
            "mizar-build/verifier-config",
            2,
        ),
        modules: Vec::new(),
        development_artifacts: Vec::new(),
        provenance: ManifestProvenance {
            generated_by: "mizar-artifact-test".to_owned(),
            manifest_policy: "test-policy".to_owned(),
            transaction_format: "manifest-transaction-v1".to_owned(),
        },
    }
}

fn fake_entry(module: ModuleSummaryIdentity, artifact_file: &str, seed: u8) -> ModuleArtifactEntry {
    ModuleArtifactEntry {
        module,
        source_file: "articles/source.miz".to_owned(),
        source_hash: hash(seed),
        artifact_file: artifact_file.to_owned(),
        artifact_hash: hash_ref(
            ArtifactHashClass::Artifact,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            seed,
        ),
        interface_hash: hash_ref(
            ArtifactHashClass::Interface,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            seed + 1,
        ),
        implementation_hash: hash_ref(
            ArtifactHashClass::Implementation,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            seed + 2,
        ),
        module_summary_file: None,
        module_summary_hash: None,
        module_summary_interface_hash: None,
        registration_summary_file: None,
        registration_summary_hash: None,
        registration_interface_hash: None,
        proof_witnesses: Vec::new(),
        diagnostics_hash: None,
    }
}

fn development_entry(
    kind: &str,
    path: &str,
    related_module: Option<ModuleSummaryIdentity>,
) -> DevelopmentArtifactEntry {
    DevelopmentArtifactEntry {
        kind: kind.to_owned(),
        path: path.to_owned(),
        artifact_hash: Some(hash_ref(ArtifactHashClass::Artifact, "mizar-dev/trace", 50)),
        diagnostic_hash: None,
        related_module,
    }
}

fn manifest_json_with_modules<'a>(
    manifest: &ArtifactManifest,
    modules: impl IntoIterator<Item = &'a ModuleArtifactEntry>,
) -> CanonicalJson {
    json_object([
        (
            "schema_version",
            CanonicalJson::string(manifest.schema_version.to_string()),
        ),
        (
            "package",
            package_identity_json(&manifest.package).expect("package JSON"),
        ),
        (
            "artifact_root",
            CanonicalJson::string(&manifest.artifact_root),
        ),
        (
            "lockfile_hash",
            CanonicalJson::string(manifest.lockfile_hash.to_artifact_hash_string()),
        ),
        ("toolchain", CanonicalJson::string(&manifest.toolchain)),
        (
            "language_edition",
            CanonicalJson::string(&manifest.language_edition),
        ),
        (
            "verifier_config_hash",
            CanonicalJson::string(manifest.verifier_config_hash.to_artifact_hash_string()),
        ),
        (
            "modules",
            CanonicalJson::array(
                modules
                    .into_iter()
                    .map(module_entry_json_preserving_proof_order)
                    .collect::<Result<Vec<_>, _>>()
                    .expect("module entry JSON"),
            ),
        ),
        ("development_artifacts", CanonicalJson::array([])),
        (
            "provenance",
            manifest_provenance_json(&manifest.provenance).expect("provenance JSON"),
        ),
    ])
    .expect("manifest JSON")
}

fn module_entry_json_preserving_proof_order(
    entry: &ModuleArtifactEntry,
) -> Result<CanonicalJson, ManifestError> {
    let mut json = module_entry_json(entry)?;
    let CanonicalJson::Object(fields) = &mut json else {
        unreachable!("module entry JSON is an object");
    };
    fields.insert(
        "proof_witnesses".to_owned(),
        CanonicalJson::array(
            entry
                .proof_witnesses
                .iter()
                .map(proof_witness_entry_json)
                .collect::<Result<Vec<_>, _>>()?,
        ),
    );
    Ok(json)
}

fn manifest_json_with_development_artifacts(manifest: &ArtifactManifest) -> CanonicalJson {
    json_object([
        (
            "schema_version",
            CanonicalJson::string(manifest.schema_version.to_string()),
        ),
        (
            "package",
            package_identity_json(&manifest.package).expect("package JSON"),
        ),
        (
            "artifact_root",
            CanonicalJson::string(&manifest.artifact_root),
        ),
        (
            "lockfile_hash",
            CanonicalJson::string(manifest.lockfile_hash.to_artifact_hash_string()),
        ),
        ("toolchain", CanonicalJson::string(&manifest.toolchain)),
        (
            "language_edition",
            CanonicalJson::string(&manifest.language_edition),
        ),
        (
            "verifier_config_hash",
            CanonicalJson::string(manifest.verifier_config_hash.to_artifact_hash_string()),
        ),
        ("modules", CanonicalJson::array([])),
        (
            "development_artifacts",
            CanonicalJson::array(
                manifest
                    .development_artifacts
                    .iter()
                    .map(development_entry_json)
                    .collect::<Result<Vec<_>, _>>()
                    .expect("development JSON"),
            ),
        ),
        (
            "provenance",
            manifest_provenance_json(&manifest.provenance).expect("provenance JSON"),
        ),
    ])
    .expect("manifest JSON")
}

fn sample_verified_artifact(
    module: ModuleSummaryIdentity,
    source_file: &str,
    source_hash: Hash,
    with_witness: bool,
) -> VerifiedArtifact {
    let schema_version = SchemaVersion::new(1, 0);
    let witness_schema_version = proof_witness_schema_version();
    let mut obligations = Vec::new();
    let mut proof_witnesses = Vec::new();
    let verifier_policy = hash_ref(ArtifactHashClass::Interface, "mizar-proof/policy", 21);
    let obligation_fingerprint =
        hash_ref(ArtifactHashClass::Interface, "mizar-proof/obligation", 20);
    if with_witness {
        obligations.push(ObligationMetadata {
            obligation_id: "obl-1".to_owned(),
            obligation_anchor: Some("A.Th1.proof".to_owned()),
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
        });
        proof_witnesses.push(ProofWitnessRef {
            schema_version: witness_schema_version,
            obligation_id: "obl-1".to_owned(),
            obligation_fingerprint,
            proof_status: WitnessProofStatus::KernelVerified,
            evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
            witness_path: "proof-witnesses/a/obl-1.json".to_owned(),
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
                verifier_policy_fingerprint: verifier_policy.clone(),
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
                formula_context_hash: None,
                accepted_result_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/accepted-result",
                    37,
                ),
            },
        });
    }

    let mut artifact = VerifiedArtifact {
        schema_version,
        module,
        source_file: source_file.to_owned(),
        source_hash,
        verified_at: Some("2026-06-22T14:03:05Z".to_owned()),
        interface_hash: hash(0),
        implementation_hash: hash(0),
        exports: vec![VerifiedExport {
            origin_id: "export-1".to_owned(),
            fully_qualified_name: "A.Th1".to_owned(),
            namespace_path: vec!["A".to_owned()],
            visibility: ExportVisibility::Public,
            export_kind: "theorem".to_owned(),
            source_range: range(10, 20),
            rendered_signature: "for x holds x = x".to_owned(),
            interface_fingerprint: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-checker/export",
                2,
            ),
            proof_status: Some(if with_witness {
                ExportProofStatus::Accepted
            } else {
                ExportProofStatus::NotRequired
            }),
            documentation_ref: None,
        }],
        statements: Vec::new(),
        expressions: vec![ExpressionMetadata {
            expression_id: "expr-1".to_owned(),
            source_range: range(11, 12),
            expression_kind: "term".to_owned(),
            rendered_surface: "x".to_owned(),
            inferred_type: Some("object".to_owned()),
            resolved_symbol: None,
            inserted_coercions: Vec::new(),
            active_thesis: None,
            overload_resolution: Some(OverloadMetadata {
                root_symbol: "equals".to_owned(),
                selected_candidate: "builtin.eq".to_owned(),
                active_refinements: vec!["object".to_owned()],
                coercion_summary: None,
            }),
        }],
        obligations,
        proof_witnesses,
        diagnostics: vec![ArtifactDiagnostic {
            diagnostic_id: "diag-1".to_owned(),
            code: "MZ1001".to_owned(),
            severity: DiagnosticSeverity::Info,
            primary_range: None,
            message_key: "artifact.note".to_owned(),
            rendered_message: "artifact note".to_owned(),
            related: vec![DiagnosticRelated {
                source_range: range(1, 2),
                message_key: "context.first".to_owned(),
                rendered_message: "first context".to_owned(),
            }],
            explanation_ref: None,
        }],
        provenance: BuildProvenance {
            captured_source: None,
            toolchain: "mizar-evo-test".to_owned(),
            language_edition: "2026".to_owned(),
            lockfile_hash: hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 36),
            verifier_config_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-build/verifier-config",
                37,
            ),
            dependency_artifact_hashes: vec![DependencyArtifactHash {
                module: identity("dep", "articles/base", "2026"),
                interface_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-artifact/module-summary",
                    38,
                ),
                implementation_hash: None,
                artifact_hash: None,
            }],
            cache_key: Some("cache-key-1".to_owned()),
        },
    };
    artifact.refresh_hashes().expect("sample hashes");
    artifact
}

fn publish_verified_artifact(
    root: &Path,
    path: &str,
    artifact: &VerifiedArtifact,
) -> ModuleArtifactEntry {
    let json = verified_artifact_json(artifact).expect("verified artifact JSON");
    let domain = artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version);
    let excluded = artifact_hash_excluded_paths();
    let published_path = PublishedArtifactPath::new(path).expect("published path");
    let write = write_published_artifact(root, &published_path, &json, &domain, &excluded)
        .expect("write verified artifact");
    ModuleArtifactEntry {
        module: artifact.module.clone(),
        source_file: artifact.source_file.clone(),
        source_hash: artifact.source_hash,
        artifact_file: path.to_owned(),
        artifact_hash: ArtifactHashRef::new(
            ArtifactHashClass::Artifact,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            artifact.schema_version,
            write.artifact_hash,
        ),
        interface_hash: ArtifactHashRef::new(
            ArtifactHashClass::Interface,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            artifact.schema_version,
            artifact.interface_hash,
        ),
        implementation_hash: ArtifactHashRef::new(
            ArtifactHashClass::Implementation,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            artifact.schema_version,
            artifact.implementation_hash,
        ),
        module_summary_file: None,
        module_summary_hash: None,
        module_summary_interface_hash: None,
        registration_summary_file: None,
        registration_summary_hash: None,
        registration_interface_hash: None,
        proof_witnesses: artifact
            .proof_witnesses
            .iter()
            .map(|witness| ManifestProofWitnessEntry {
                obligation_id: witness.obligation_id.clone(),
                obligation_fingerprint: witness.obligation_fingerprint.clone(),
                witness_path: witness.witness_path.clone(),
                witness_artifact_hash: witness.witness_artifact_hash.clone(),
            })
            .collect(),
        diagnostics_hash: None,
    }
}

fn sample_module_summary(module: ModuleSummaryIdentity, source_hash: Hash) -> ModuleSummary {
    let mut summary = ModuleSummary {
        schema_version: module_summary_schema_version(),
        module: module.clone(),
        source_hash,
        interface_hash: hash(0),
        exported_symbols: vec![ExportedSymbolSummary {
            origin_id: "symbol:a".to_owned(),
            fully_qualified_name: "A.Th1".to_owned(),
            namespace_path: vec!["A".to_owned()],
            visibility: "public".to_owned(),
            declaration_kind: "theorem".to_owned(),
            source_range: range(10, 20),
            rendered_signature: "theorem A.Th1: x = x".to_owned(),
            interface_fingerprint: hash(101),
            proof_status: Some(ProofStatusSummary::Accepted),
        }],
        exported_labels: vec![ExportedLabelSummary {
            origin_id: "label:a".to_owned(),
            label: "A1".to_owned(),
            owner_fully_qualified_name: "A.Th1".to_owned(),
            visibility: "public".to_owned(),
            source_range: range(12, 13),
            target_kind: "theorem".to_owned(),
        }],
        lexical_summary: ModuleLexicalSummary {
            schema_version: "lexer-summary-v1".to_owned(),
            fingerprint: Some(hash(102)),
            contributions: vec![LexicalContributionSummary {
                kind: "notation".to_owned(),
                key: "alpha".to_owned(),
                payload: "infix alpha".to_owned(),
            }],
        },
        reexports: vec![ModuleReexportSummary {
            target_module: module,
            target_item_origin_id: Some("symbol:a".to_owned()),
            exported_name: Some("A.Th1".to_owned()),
            provenance_origin_id: Some("reexport:a".to_owned()),
        }],
        dependency_interfaces: vec![DependencyInterfaceRef {
            module: identity("dep", "articles/base", "2026"),
            interface_hash: hash(103),
        }],
    };
    summary.refresh_interface_hash().expect("summary hash");
    summary
}

fn publish_module_summary(
    root: &Path,
    path: &str,
    summary: &ModuleSummary,
) -> PublishedArtifactWrite {
    let json = module_summary_json(summary).expect("module summary JSON");
    let domain = artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary.schema_version);
    let published_path = PublishedArtifactPath::new(path).expect("module summary path");
    write_published_artifact(root, &published_path, &json, &domain, &[])
        .expect("write module summary")
}

fn module_summary_artifact_hash(summary: &ModuleSummary, schema_version: SchemaVersion) -> Hash {
    let json = module_summary_json(summary).expect("module summary JSON");
    artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, schema_version).hash(&json, &[])
}

fn attach_module_summary(
    entry: &mut ModuleArtifactEntry,
    path: &str,
    write: PublishedArtifactWrite,
) {
    entry.module_summary_file = Some(path.to_owned());
    entry.module_summary_hash = Some(ArtifactHashRef::new(
        ArtifactHashClass::Artifact,
        MODULE_SUMMARY_SCHEMA_FAMILY,
        module_summary_schema_version(),
        write.artifact_hash,
    ));
    entry.module_summary_interface_hash = Some(ArtifactHashRef::new(
        ArtifactHashClass::Interface,
        MODULE_SUMMARY_SCHEMA_FAMILY,
        module_summary_schema_version(),
        sample_module_summary(entry.module.clone(), entry.source_hash).interface_hash,
    ));
}

fn sample_registration_summary(
    module: ModuleSummaryIdentity,
    source_hash: Hash,
) -> RegistrationSummary {
    let mut summary = RegistrationSummary {
        schema_version: registration_summary_schema_version(),
        module: module.clone(),
        source_hash,
        registration_interface_hash: hash(0),
        activated_registrations: vec![ActivatedRegistrationSummary {
            origin_id: "registration:a".to_owned(),
            label: Some("label:a".to_owned()),
            registration_kind: RegistrationKind::Conditional,
            visibility: RegistrationVisibility::Public,
            namespace_path: vec!["A".to_owned()],
            source_module: module,
            trigger_key: "alpha".to_owned(),
            normalized_pattern: RegistrationPatternSummary {
                fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-artifact/checker", 111),
                type_head: Some("A.Type".to_owned()),
                attribute: Some("non_empty".to_owned()),
                functor: None,
                term_head: None,
                parameters: vec!["T".to_owned()],
                guards: vec![hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-artifact/checker",
                    112,
                )],
            },
            generated_contribution: RegistrationContributionSummary {
                kind: RegistrationContributionKind::AttributeFact,
                summary: "generated alpha contribution".to_owned(),
                fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-artifact/checker", 113),
            },
            accepted_status: RegistrationAcceptedStatus::Accepted,
            verifier_policy_fingerprint: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-artifact/verifier-policy",
                114,
            ),
            trace_ids: vec!["trace:a".to_owned()],
            source_range: Some(range(10, 20)),
        }],
        trace_artifacts: vec![RegistrationTraceArtifactRef {
            trace_id: "trace:a".to_owned(),
            trace_kind: RegistrationTraceKind::Cluster,
            artifact_path: "traces/a.json".to_owned(),
            artifact_hash: hash_ref(
                ArtifactHashClass::Artifact,
                "mizar-artifact/resolution-trace",
                115,
            ),
            trace_replay_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-artifact/resolution-trace",
                116,
            ),
            diagnostic_hash: Some(hash_ref(
                ArtifactHashClass::Diagnostic,
                "mizar-artifact/resolution-trace",
                117,
            )),
            used_by_registration_origin_ids: vec!["registration:a".to_owned()],
        }],
        dependency_registrations: vec![DependencyRegistrationRef {
            module: identity("dep", "articles/base", "2026"),
            registration_interface_hash: hash(118),
        }],
    };
    summary
        .refresh_registration_interface_hash()
        .expect("registration summary hash");
    summary
}

fn publish_registration_summary(
    root: &Path,
    path: &str,
    summary: &RegistrationSummary,
) -> PublishedArtifactWrite {
    let json = registration_summary_json(summary).expect("registration summary JSON");
    let domain = artifact_hash_domain(REGISTRATION_SUMMARY_SCHEMA_FAMILY, summary.schema_version);
    let published_path = PublishedArtifactPath::new(path).expect("registration summary path");
    write_published_artifact(root, &published_path, &json, &domain, &[])
        .expect("write registration summary")
}

fn registration_summary_artifact_hash(
    summary: &RegistrationSummary,
    schema_version: SchemaVersion,
) -> Hash {
    let json = registration_summary_json(summary).expect("registration summary JSON");
    artifact_hash_domain(REGISTRATION_SUMMARY_SCHEMA_FAMILY, schema_version).hash(&json, &[])
}

fn attach_registration_summary(
    entry: &mut ModuleArtifactEntry,
    path: &str,
    write: PublishedArtifactWrite,
) {
    entry.registration_summary_file = Some(path.to_owned());
    entry.registration_summary_hash = Some(ArtifactHashRef::new(
        ArtifactHashClass::Artifact,
        REGISTRATION_SUMMARY_SCHEMA_FAMILY,
        registration_summary_schema_version(),
        write.artifact_hash,
    ));
    entry.registration_interface_hash = Some(ArtifactHashRef::new(
        ArtifactHashClass::Interface,
        REGISTRATION_SUMMARY_SCHEMA_FAMILY,
        registration_summary_schema_version(),
        sample_registration_summary(entry.module.clone(), entry.source_hash)
            .registration_interface_hash,
    ));
}

fn manifest_witness(obligation_id: &str, seed: u8) -> ManifestProofWitnessEntry {
    ManifestProofWitnessEntry {
        obligation_id: obligation_id.to_owned(),
        obligation_fingerprint: hash_ref(
            ArtifactHashClass::Interface,
            "mizar-proof/obligation",
            seed,
        ),
        witness_path: format!("proof-witnesses/a/{obligation_id}.json"),
        witness_artifact_hash: hash_ref(
            ArtifactHashClass::Artifact,
            "mizar-kernel/formula-evidence-witness",
            seed + 10,
        ),
    }
}

fn write_plain_file(root: &Path, path: &str, bytes: &[u8]) {
    let full_path = root.join(path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("parent directory");
    }
    fs::write(full_path, bytes).expect("plain file");
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

struct TestArtifactRoot {
    path: PathBuf,
}

impl TestArtifactRoot {
    fn new() -> Self {
        let path = Self::fresh_path();
        if path.exists() {
            fs::remove_dir_all(&path).expect("remove stale test root");
        }
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn fresh_path() -> PathBuf {
        let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "mizar-artifact-manifest-test-{}-{counter}",
            std::process::id()
        ))
    }
}

impl Drop for TestArtifactRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn commit_result_retains_actual_transaction_values() {
    let root = TestArtifactRoot::new();
    let seed = sample_manifest();
    let commit = ManifestTransaction::begin(root.path(), seed.clone(), None)
        .expect("begin structural transaction")
        .commit(ManifestCommitOptions::default())
        .expect("commit structural transaction");
    let read = read_manifest_file(root.path(), ManifestFileReadOptions::default())
        .expect("read committed manifest");
    assert_eq!(read.manifest, seed);
    let expected_write = PublishedArtifactWrite {
        path: read.path,
        artifact_hash: read.artifact_hash,
    };
    assert_eq!(commit.manifest(), &seed);
    assert_eq!(commit.write(), &expected_write);

    fs::write(root.path().join(ARTIFACT_MANIFEST_PATH), b"{").expect("corrupt committed manifest");
    assert!(read_manifest_file(root.path(), ManifestFileReadOptions::default()).is_err());
    assert_eq!(commit.manifest(), &seed);
    assert_eq!(commit.write(), &expected_write);
    assert_eq!(commit.into_manifest(), seed);
}

fn q1_manifest_fixture(
    text: &str,
    available: bool,
    with_witness: bool,
) -> crate::comment_projection::CommentedArtifact {
    let source_hash = mizar_session::hash_text(text);
    let mut artifact = sample_verified_artifact(
        identity("pkg", "articles/a", "2026"),
        "articles/a.miz",
        source_hash,
        with_witness,
    );
    artifact.schema_version = SchemaVersion::new(1, 2);
    artifact.exports[0].source_range = range(0, 0);
    artifact.expressions[0].source_range = range(0, 0);
    artifact.diagnostics[0].related[0].source_range = range(0, 0);
    for obligation in &mut artifact.obligations {
        obligation.source_range = range(0, 0);
    }
    artifact.provenance.captured_source = Some((
        mizar_session::SourceId::from_nonportable_provenance_str(
            "mizar-session-source-nonportable-v1:0000000000000001",
        )
        .unwrap(),
        mizar_session::SourceOrigin::Disk,
        source_hash,
    ));
    artifact.refresh_hashes().unwrap();
    let whole = CanonicalJson::array([
        CanonicalJson::integer(0),
        CanonicalJson::integer(text.len() as i64),
    ]);
    let comments = if available {
        CanonicalJson::object([
            ("original", CanonicalJson::string(text)),
            ("loaded", CanonicalJson::string(text)),
            ("lexical", CanonicalJson::string("")),
            (
                "loading",
                CanonicalJson::array([CanonicalJson::array([
                    CanonicalJson::string("original"),
                    whole.clone(),
                    whole.clone(),
                ])]),
            ),
            (
                "preprocess",
                CanonicalJson::array(if text.is_empty() {
                    vec![]
                } else {
                    vec![CanonicalJson::array([
                        CanonicalJson::string("removed"),
                        CanonicalJson::Null,
                        whole.clone(),
                        CanonicalJson::string("documentation"),
                    ])]
                }),
            ),
            (
                "comments",
                CanonicalJson::array(if text.is_empty() {
                    vec![]
                } else {
                    vec![CanonicalJson::array([
                        CanonicalJson::string("documentation"),
                        whole,
                        CanonicalJson::string(text),
                    ])]
                }),
            ),
            ("whitespace", CanonicalJson::array([])),
            ("landmarks", CanonicalJson::array([])),
        ])
        .unwrap()
    } else {
        CanonicalJson::Null
    };
    crate::comment_projection::CommentedArtifact { artifact, comments }
}

fn q1_manifest_publish(
    root: &Path,
    path: &str,
    value: &crate::comment_projection::CommentedArtifact,
) -> ModuleArtifactEntry {
    use crate::comment_projection::{
        COMMENTED_ARTIFACT_SCHEMA_FAMILY, COMMENTED_ARTIFACT_SCHEMA_VERSION,
        artifact_hash_excluded_paths, commented_artifact_json,
    };
    let mut entry = publish_verified_artifact(root, path, &value.artifact);
    let published = write_published_artifact(
        root,
        &PublishedArtifactPath::new(path).unwrap(),
        &commented_artifact_json(value).unwrap(),
        &artifact_hash_domain(
            COMMENTED_ARTIFACT_SCHEMA_FAMILY,
            COMMENTED_ARTIFACT_SCHEMA_VERSION,
        ),
        &artifact_hash_excluded_paths(),
    )
    .unwrap();
    for (reference, digest) in [
        (&mut entry.artifact_hash, published.artifact_hash),
        (
            &mut entry.interface_hash,
            value.compute_interface_hash().unwrap(),
        ),
        (
            &mut entry.implementation_hash,
            value.compute_implementation_hash().unwrap(),
        ),
    ] {
        reference.schema_family = COMMENTED_ARTIFACT_SCHEMA_FAMILY.to_owned();
        reference.schema_version = COMMENTED_ARTIFACT_SCHEMA_VERSION;
        reference.digest = digest;
    }
    entry
}

fn q1_manifest_validate(
    root: &Path,
    entry: ModuleArtifactEntry,
) -> Result<PublishedManifestRead, ManifestError> {
    let mut manifest = sample_manifest();
    manifest.modules = vec![entry];
    write_manifest_file(root, &manifest)?;
    read_manifest_file(
        root,
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
}

fn q1_manifest_set(value: &mut CanonicalJson, path: &[&str], replacement: CanonicalJson) {
    let CanonicalJson::Object(fields) = value else {
        panic!("object field")
    };
    if path.len() == 1 {
        fields.insert(path[0].to_owned(), replacement);
    } else {
        q1_manifest_set(fields.get_mut(path[0]).unwrap(), &path[1..], replacement);
    }
}

#[test]
fn q1_manifest_complete_projection_round_trips_through_transaction() {
    use crate::comment_projection::{
        commented_artifact_json, read_commented_artifact, write_commented_artifact,
    };
    let root = TestArtifactRoot::new();
    let value = q1_manifest_fixture("::: retained β", true, false);
    let json = commented_artifact_json(&value).unwrap();
    let decoded = read_commented_artifact(&json, VerifiedArtifactReadOptions::default()).unwrap();
    assert_eq!(decoded, value);
    let entry = q1_manifest_publish(root.path(), "artifacts/commented.json", &decoded);
    assert_eq!(
        entry.artifact_hash.digest,
        value.compute_artifact_hash().unwrap()
    );
    assert_ne!(entry.interface_hash.digest, value.artifact.interface_hash);
    assert_ne!(
        entry.implementation_hash.digest,
        value.artifact.implementation_hash
    );
    assert_eq!(
        fs::read(root.path().join(&entry.artifact_file)).unwrap(),
        write_commented_artifact(&value).unwrap()
    );
    let mut transaction = ManifestTransaction::begin(root.path(), sample_manifest(), None).unwrap();
    transaction.stage_module(entry.clone()).unwrap();
    transaction
        .commit(ManifestCommitOptions::default())
        .unwrap();
    let read = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .unwrap();
    assert_eq!(read.manifest.schema_version, SchemaVersion::new(1, 0));
    assert_eq!(read.manifest.modules, vec![entry]);
    let retained = crate::store::parse_canonical_json_bytes(
        &fs::read(root.path().join("artifacts/commented.json")).unwrap(),
        "artifacts/commented.json",
    )
    .unwrap();
    assert_eq!(
        read_commented_artifact(&retained, VerifiedArtifactReadOptions::default())
            .unwrap()
            .comments,
        value.comments
    );
}

#[test]
fn q1_manifest_availability_determinism_and_inherited_exclusions() {
    use crate::comment_projection::{artifact_hash_excluded_paths, write_commented_artifact};
    let root = TestArtifactRoot::new();
    let unavailable = q1_manifest_fixture("", false, false);
    let available = q1_manifest_fixture("", true, false);
    assert_eq!(
        unavailable.compute_interface_hash().unwrap(),
        available.compute_interface_hash().unwrap()
    );
    assert_ne!(
        unavailable.compute_implementation_hash().unwrap(),
        available.compute_implementation_hash().unwrap()
    );
    assert_ne!(
        unavailable.compute_artifact_hash().unwrap(),
        available.compute_artifact_hash().unwrap()
    );
    for value in [&unavailable, &available] {
        let first = q1_manifest_publish(root.path(), "artifacts/commented.json", value);
        let bytes = fs::read(root.path().join(&first.artifact_file)).unwrap();
        assert_eq!(bytes, write_commented_artifact(value).unwrap());
        assert_eq!(
            first,
            q1_manifest_publish(root.path(), "artifacts/commented.json", value)
        );
        assert_eq!(
            bytes,
            fs::read(root.path().join(&first.artifact_file)).unwrap()
        );
        q1_manifest_validate(root.path(), first).unwrap();
    }
    let mut local = available.clone();
    local.artifact.verified_at = Some("2026-11-18T00:00:00Z".to_owned());
    local.artifact.provenance.cache_key = Some("another-local-cache".to_owned());
    local
        .artifact
        .provenance
        .captured_source
        .as_mut()
        .unwrap()
        .0 = mizar_session::SourceId::from_nonportable_provenance_str(
        "mizar-session-source-nonportable-v1:0000000000000002",
    )
    .unwrap();
    assert_ne!(
        write_commented_artifact(&local).unwrap(),
        write_commented_artifact(&available).unwrap()
    );
    assert_eq!(
        local.compute_interface_hash().unwrap(),
        available.compute_interface_hash().unwrap()
    );
    assert_eq!(
        local.compute_implementation_hash().unwrap(),
        available.compute_implementation_hash().unwrap()
    );
    let original_entry = q1_manifest_publish(root.path(), "artifacts/commented.json", &available);
    assert_eq!(
        original_entry,
        q1_manifest_publish(root.path(), "artifacts/commented.json", &local)
    );
    q1_manifest_validate(root.path(), original_entry).unwrap();
    assert_eq!(
        artifact_hash_excluded_paths(),
        vec![
            crate::store::FieldPath::new(["artifact", "verified_at"]).unwrap(),
            crate::store::FieldPath::new(["artifact", "provenance", "cache_key"]).unwrap(),
            crate::store::FieldPath::new([
                "artifact",
                "provenance",
                "captured_source",
                "source_id"
            ])
            .unwrap(),
        ]
    );
    let legacy = sample_verified_artifact(
        identity("pkg", "articles/a", "2026"),
        "articles/a.miz",
        hash(1),
        false,
    );
    let legacy_entry = publish_verified_artifact(root.path(), "artifacts/legacy.json", &legacy);
    q1_manifest_validate(root.path(), legacy_entry).unwrap();
}

#[test]
fn q1_manifest_refuses_mixed_and_unknown_hash_domains() {
    use crate::comment_projection::COMMENTED_ARTIFACT_SCHEMA_FAMILY;
    let root = TestArtifactRoot::new();
    let entry = q1_manifest_publish(
        root.path(),
        "artifacts/commented.json",
        &q1_manifest_fixture("::: domains", true, false),
    );
    q1_manifest_validate(root.path(), entry.clone()).unwrap();
    for case in 0..8 {
        let mut bad = entry.clone();
        match case {
            0 => bad.interface_hash.schema_family = VERIFIED_ARTIFACT_SCHEMA_FAMILY.to_owned(),
            1 => bad.implementation_hash.schema_family = VERIFIED_ARTIFACT_SCHEMA_FAMILY.to_owned(),
            2 => bad.interface_hash.schema_version = SchemaVersion::new(1, 2),
            3 => bad.implementation_hash.schema_version = SchemaVersion::new(1, 2),
            4 => bad.artifact_hash.class = ArtifactHashClass::Implementation,
            5 => bad.interface_hash.class = ArtifactHashClass::Artifact,
            6 => bad.implementation_hash.class = ArtifactHashClass::Interface,
            _ => bad.artifact_hash.schema_family = "unknown/artifact".to_owned(),
        }
        let json = manifest_json_with_modules(&sample_manifest(), [&bad]);
        assert!(
            read_artifact_manifest(&json, ArtifactManifestReadOptions::default()).is_err(),
            "domain case {case}"
        );
    }
    for version in [SchemaVersion::new(1, 1), SchemaVersion::new(2, 0)] {
        let mut bad = entry.clone();
        for reference in [
            &mut bad.artifact_hash,
            &mut bad.interface_hash,
            &mut bad.implementation_hash,
        ] {
            assert_eq!(reference.schema_family, COMMENTED_ARTIFACT_SCHEMA_FAMILY);
            reference.schema_version = version;
        }
        let json = manifest_json_with_modules(&sample_manifest(), [&bad]);
        assert!(read_artifact_manifest(&json, ArtifactManifestReadOptions::default()).is_err());
    }
    q1_manifest_validate(root.path(), entry).unwrap();
}

#[test]
fn q1_manifest_rehashed_malformed_payloads_never_fall_back() {
    use crate::comment_projection::{
        COMMENTED_ARTIFACT_SCHEMA_FAMILY, COMMENTED_ARTIFACT_SCHEMA_VERSION,
        artifact_hash_excluded_paths, commented_artifact_json,
    };
    let root = TestArtifactRoot::new();
    let value = q1_manifest_fixture("::: malformed", true, false);
    let entry = q1_manifest_publish(root.path(), "artifacts/commented.json", &value);
    for case in 0..6 {
        let mut json = commented_artifact_json(&value).unwrap();
        match case {
            0 => q1_manifest_set(
                &mut json,
                &["comments", "loading"],
                CanonicalJson::array([]),
            ),
            1 => q1_manifest_set(
                &mut json,
                &["artifact", "schema_version"],
                CanonicalJson::string("1.1"),
            ),
            2 => q1_manifest_set(
                &mut json,
                &["artifact", "interface_hash"],
                CanonicalJson::string("invalid"),
            ),
            3 => q1_manifest_set(&mut json, &["schema_version"], CanonicalJson::string("1.1")),
            4 => q1_manifest_set(
                &mut json,
                &["comments"],
                CanonicalJson::object([] as [(&str, CanonicalJson); 0]).unwrap(),
            ),
            _ => q1_manifest_set(
                &mut json,
                &["artifact", "provenance", "captured_source", "source_hash"],
                CanonicalJson::string("invalid"),
            ),
        }
        let write = write_published_artifact(
            root.path(),
            &PublishedArtifactPath::new(&entry.artifact_file).unwrap(),
            &json,
            &artifact_hash_domain(
                COMMENTED_ARTIFACT_SCHEMA_FAMILY,
                COMMENTED_ARTIFACT_SCHEMA_VERSION,
            ),
            &artifact_hash_excluded_paths(),
        )
        .unwrap();
        let mut bad = entry.clone();
        bad.artifact_hash.digest = write.artifact_hash;
        assert!(
            matches!(
                q1_manifest_validate(root.path(), bad),
                Err(ManifestError::VerifiedArtifact { .. })
            ),
            "payload case {case}"
        );
    }
    let entry = q1_manifest_publish(root.path(), "artifacts/commented.json", &value);
    for interface in [true, false] {
        let mut bad = entry.clone();
        if interface {
            bad.interface_hash.digest = hash(88);
        } else {
            bad.implementation_hash.digest = hash(89);
        }
        let error = q1_manifest_validate(root.path(), bad).unwrap_err();
        assert!(
            matches!(error, ManifestError::VerifiedArtifact { error, .. }
            if matches!(*error, crate::verified_artifact::VerifiedArtifactError::ExpectedInterfaceHashMismatch { .. }
                | crate::verified_artifact::VerifiedArtifactError::ExpectedImplementationHashMismatch { .. }))
        );
    }
    q1_manifest_validate(root.path(), entry).unwrap();
}

#[test]
fn q1_manifest_inner_identity_witnesses_and_sidecars_are_validated() {
    let root = TestArtifactRoot::new();
    let value = q1_manifest_fixture("::: witnesses and summaries", true, true);
    let mut entry = q1_manifest_publish(root.path(), "artifacts/commented.json", &value);
    let module_summary = sample_module_summary(entry.module.clone(), entry.source_hash);
    let registration_summary = sample_registration_summary(entry.module.clone(), entry.source_hash);
    attach_module_summary(
        &mut entry,
        "summaries/module.json",
        publish_module_summary(root.path(), "summaries/module.json", &module_summary),
    );
    attach_registration_summary(
        &mut entry,
        "summaries/registration.json",
        publish_registration_summary(
            root.path(),
            "summaries/registration.json",
            &registration_summary,
        ),
    );
    q1_manifest_validate(root.path(), entry.clone()).unwrap();
    for case in 0..8 {
        let mut bad = entry.clone();
        match case {
            0 => bad.module.package_id = "another-package".to_owned(),
            1 => bad.module.package_version = Some("2.0.0".to_owned()),
            2 => bad.module.lockfile_identity = Some("another-lock".to_owned()),
            3 => bad.module.module_path = "articles/other".to_owned(),
            4 => bad.module.language_edition = "2027".to_owned(),
            5 => bad.source_file = "different/a.miz".to_owned(),
            6 => bad.source_hash = hash(87),
            _ => bad.proof_witnesses.clear(),
        }
        let error = q1_manifest_validate(root.path(), bad).unwrap_err();
        assert!(
            match case {
                0..=4 => matches!(error, ManifestError::VerifiedArtifact { error, .. }
                    if matches!(*error, VerifiedArtifactError::ModuleIdentityMismatch { .. })),
                5..=6 =>
                    matches!(error, ManifestError::ReferencedArtifactMismatch { path, field, .. }
                    if path == entry.artifact_file
                        && field == if case == 5 { "source_file" } else { "source_hash" }),
                _ => matches!(error, ManifestError::WitnessCoverageMismatch { .. }),
            },
            "binding case {case}"
        );
    }
    for case in 0..4 {
        let mut bad = entry.clone();
        match case {
            0 => bad.proof_witnesses[0].obligation_fingerprint.digest = hash(77),
            1 => bad.proof_witnesses[0].witness_path = "proof-witnesses/wrong.json".to_owned(),
            2 => bad.proof_witnesses[0].witness_artifact_hash.digest = hash(78),
            _ => bad.proof_witnesses[0].obligation_id = "other-obligation".to_owned(),
        }
        assert!(matches!(
            q1_manifest_validate(root.path(), bad),
            Err(ManifestError::WitnessCoverageMismatch { .. })
        ));
    }
    q1_manifest_validate(root.path(), entry.clone()).unwrap();
    let options = ManifestFileReadOptions {
        validate_references: true,
        validate_witness_files: true,
        ..ManifestFileReadOptions::default()
    };
    assert!(read_manifest_file(root.path(), options).is_err());
    write_plain_file(
        root.path(),
        &entry.proof_witnesses[0].witness_path,
        b"producer-owned witness",
    );
    read_manifest_file(root.path(), options).unwrap();
    for registration in [false, true] {
        let mut bad = entry.clone();
        if registration {
            bad.registration_interface_hash.as_mut().unwrap().digest = hash(79);
        } else {
            bad.module_summary_interface_hash.as_mut().unwrap().digest = hash(80);
        }
        assert!(q1_manifest_validate(root.path(), bad).is_err());
    }
    q1_manifest_validate(root.path(), entry).unwrap();
}

#[test]
fn q1_manifest_comment_changes_and_failed_transactions_preserve_publication() {
    let root = TestArtifactRoot::new();
    let original = q1_manifest_fixture("::: before", true, false);
    let changed = q1_manifest_fixture("::: after β", true, false);
    let entry = q1_manifest_publish(root.path(), "artifacts/published.json", &original);
    let replacement = q1_manifest_publish(root.path(), "artifacts/orphan.json", &changed);
    assert_eq!(entry.interface_hash, replacement.interface_hash);
    assert_ne!(entry.implementation_hash, replacement.implementation_hash);
    assert_ne!(entry.artifact_hash, replacement.artifact_hash);
    q1_manifest_validate(root.path(), entry.clone()).unwrap();
    let before = fs::read(root.path().join(ARTIFACT_MANIFEST_PATH)).unwrap();
    let mut stale = entry.clone();
    stale.artifact_file = replacement.artifact_file.clone();
    assert!(q1_manifest_validate(root.path(), stale).is_err());
    q1_manifest_validate(root.path(), entry.clone()).unwrap();
    let mut bad = replacement.clone();
    bad.artifact_hash.digest = hash(81);
    let mut transaction = ManifestTransaction::begin(root.path(), sample_manifest(), None).unwrap();
    transaction.stage_module(bad).unwrap();
    assert!(
        transaction
            .commit(ManifestCommitOptions::default())
            .is_err()
    );
    assert_eq!(
        fs::read(root.path().join(ARTIFACT_MANIFEST_PATH)).unwrap(),
        before
    );
    let mut transaction =
        ManifestTransaction::begin(root.path(), sample_manifest(), Some("old".to_owned())).unwrap();
    transaction.stage_module(replacement.clone()).unwrap();
    let freshness = |guard: Option<&str>| guard == Some("current");
    assert!(matches!(
        transaction.commit(ManifestCommitOptions {
            freshness_check: Some(&freshness)
        }),
        Err(ManifestError::ObsoleteSnapshot { .. })
    ));
    assert_eq!(
        fs::read(root.path().join(ARTIFACT_MANIFEST_PATH)).unwrap(),
        before
    );
    let mut stale_base = ManifestTransaction::begin(root.path(), sample_manifest(), None).unwrap();
    stale_base.stage_module(replacement.clone()).unwrap();
    let mut intervening = sample_manifest();
    intervening.modules = vec![entry.clone()];
    intervening.development_artifacts =
        vec![development_entry("trace", "dev/intervening.json", None)];
    write_plain_file(root.path(), "dev/intervening.json", b"trace");
    write_manifest_file(root.path(), &intervening).unwrap();
    let intervening_bytes = fs::read(root.path().join(ARTIFACT_MANIFEST_PATH)).unwrap();
    assert!(matches!(
        stale_base.commit(ManifestCommitOptions::default()),
        Err(ManifestError::BaseManifestHashMismatch { .. })
    ));
    assert_eq!(
        fs::read(root.path().join(ARTIFACT_MANIFEST_PATH)).unwrap(),
        intervening_bytes
    );
    let read = read_manifest_file(
        root.path(),
        ManifestFileReadOptions {
            validate_references: true,
            ..ManifestFileReadOptions::default()
        },
    )
    .unwrap();
    assert_eq!(read.manifest.modules, vec![entry]);
    assert!(root.path().join(&replacement.artifact_file).is_file());
    fs::write(root.path().join(&replacement.artifact_file), b"tampered").unwrap();
    assert!(q1_manifest_validate(root.path(), replacement).is_err());
}
