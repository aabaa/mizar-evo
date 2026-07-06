use std::{
    fs, io,
    path::{Path, PathBuf},
};

use mizar_artifact::store::SchemaVersion as ArtifactSchemaVersion;
use mizar_cache::{
    cache_key::{
        CACHE_KEY_SCHEMA_VERSION, CacheKey, CacheKeyBuildOutcome, CacheKeyBuilder, CacheKeyRequest,
        CacheValidationInputs, CompatibilityField, DependencyArtifactAvailability, DependencyHash,
        DependencySliceHash, DiagnosticRefHash, FootprintCompleteness, NamedHash,
        NamedSchemaVersion, PolicyFingerprint, SchemaVersion, WorkUnit,
    },
    cache_store::{
        CacheInsertOutcome, CacheLookupOutcome, CacheMiss, CacheOutputDescriptor, CacheRecord,
        CacheStoreRoot,
    },
    proof_reuse::{
        PROOF_REUSE_SCHEMA_VERSION, ProofReuseDependencyCompatibilitySnapshot,
        ProofReuseMetadataSnapshot, ProofReuseMissReason, ProofReuseValidationEnvironment,
        ProofReuseValidationOutcome, ProofReuseValidationRequest, ProofReuseValidator,
        SUPPORTED_ACCEPTED_GOAL_POLARITY,
    },
};
use mizar_proof::selection::ProofWinnerClass;
use mizar_session::Hash;

#[test]
fn cache_key_projection_is_independent_of_input_vector_order() {
    let key = cache_key_with_order(VectorOrder::Forward);
    let shuffled = cache_key_with_order(VectorOrder::Reverse);

    assert_eq!(key, shuffled);
    assert_eq!(
        key.input_hashes
            .iter()
            .map(|input| input.name.as_str())
            .collect::<Vec<_>>(),
        ["source", "manifest"]
    );
    assert_eq!(
        key.validation_inputs
            .diagnostic_refs
            .iter()
            .map(|diagnostic| diagnostic.diagnostic_ref_kind.as_str())
            .collect::<Vec<_>>(),
        ["key", "proof"]
    );
}

#[test]
fn cache_store_deletion_changes_only_lookup_availability() {
    let root = temp_cache_root("store-deletion");
    remove_dir_if_exists(&root);
    let store = CacheStoreRoot::new(root.clone());
    let produced_by = vec![compatibility("toolchain", "mizar-cache", "task-16")];
    let key = cache_key_with_order(VectorOrder::Forward);

    let inline = CacheRecord::new_inline(
        key.clone(),
        produced_by.clone(),
        b"deterministic inline output".to_vec(),
    );
    let inline_again = CacheRecord::new_inline(
        key.clone(),
        produced_by.clone(),
        b"deterministic inline output".to_vec(),
    );
    assert_eq!(inline, inline_again);
    assert_eq!(store.insert(&inline).unwrap(), CacheInsertOutcome::Inserted);
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Hit(Box::new(inline.clone()))
    );

    fs::remove_file(store.record_path(&key)).unwrap();
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::NotFound)
    );
    assert_eq!(store.insert(&inline).unwrap(), CacheInsertOutcome::Inserted);
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Hit(Box::new(inline_again))
    );

    let blob_key = cache_key_with_work_unit("task-16/blob-record");
    let blob_output =
        b"deterministic blob output large enough to live behind a blob descriptor".to_vec();
    let blob_record =
        CacheRecord::new_blob(blob_key.clone(), produced_by.clone(), blob_output.clone());
    let blob_record_again = CacheRecord::new_blob(blob_key.clone(), produced_by, blob_output);
    assert_eq!(blob_record, blob_record_again);
    let blob_ref = match &blob_record.header.output {
        CacheOutputDescriptor::Blob { blob, .. } => blob.clone(),
        _ => panic!("expected blob-backed record"),
    };
    assert_eq!(
        store.insert(&blob_record).unwrap(),
        CacheInsertOutcome::Inserted
    );
    assert_eq!(
        store.lookup(&blob_key),
        CacheLookupOutcome::Hit(Box::new(blob_record.clone()))
    );

    fs::remove_file(store.blob_path(&blob_ref)).unwrap();
    assert_eq!(
        store.lookup(&blob_key),
        CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
    );
    assert_eq!(
        store.insert(&blob_record).unwrap(),
        CacheInsertOutcome::AlreadyPresent
    );
    assert_eq!(
        store.lookup(&blob_key),
        CacheLookupOutcome::Hit(Box::new(blob_record_again))
    );

    remove_dir_if_exists(&root);
}

#[test]
fn proof_reuse_validation_is_deterministic_and_never_promotes_external_evidence() {
    let hit = proof_request(ProofWinnerClass::KernelVerified, VectorOrder::Forward);
    let hit_reordered = proof_request(ProofWinnerClass::KernelVerified, VectorOrder::Reverse);

    assert_eq!(
        ProofReuseValidator::validate(&hit),
        ProofReuseValidator::validate(&hit_reordered)
    );
    let ProofReuseValidationOutcome::Hit(hit) = ProofReuseValidator::validate(&hit) else {
        panic!("kernel-verified proof metadata should validate");
    };
    assert_eq!(
        hit.diagnostic_refs
            .iter()
            .map(|diagnostic| diagnostic.diagnostic_ref_hash)
            .collect::<Vec<_>>(),
        [hash(41), hash(42)]
    );

    let mut miss = proof_request(ProofWinnerClass::KernelVerified, VectorOrder::Forward);
    miss.cached.obligation_anchor = "different-anchor".to_owned();
    let mut miss_reordered = miss.clone();
    miss_reordered.diagnostic_refs.reverse();
    assert_eq!(
        ProofReuseValidator::validate(&miss),
        ProofReuseValidator::validate(&miss_reordered)
    );
    let ProofReuseValidationOutcome::Miss(anchor_miss) = ProofReuseValidator::validate(&miss)
    else {
        panic!("obligation-anchor mismatch must fail closed");
    };
    assert_eq!(
        anchor_miss.reason,
        ProofReuseMissReason::ObligationAnchorMismatch
    );

    let external = proof_request(
        ProofWinnerClass::PolicyPermittedExternal,
        VectorOrder::Forward,
    );
    let ProofReuseValidationOutcome::Miss(external_miss) = ProofReuseValidator::validate(&external)
    else {
        panic!("externally attested evidence must not become a trusted reuse hit");
    };
    assert_eq!(external_miss.reason, ProofReuseMissReason::NonReusableClass);
}

#[derive(Clone, Copy)]
enum VectorOrder {
    Forward,
    Reverse,
}

fn cache_key_with_order(order: VectorOrder) -> CacheKey {
    cache_key_from_request(cache_key_request(
        order,
        WorkUnit::new("task-16/key-record"),
    ))
}

fn cache_key_with_work_unit(work_unit: &str) -> CacheKey {
    cache_key_from_request(cache_key_request(
        VectorOrder::Forward,
        WorkUnit::new(work_unit),
    ))
}

fn cache_key_from_request(request: CacheKeyRequest) -> CacheKey {
    match CacheKeyBuilder::new(request).build() {
        CacheKeyBuildOutcome::Cacheable(key) => key,
        CacheKeyBuildOutcome::Uncacheable(key) => panic!("expected cacheable key, got {key:?}"),
        CacheKeyBuildOutcome::NoKey(rejection) => {
            panic!("expected cacheable key, got rejection {rejection:?}")
        }
        _ => panic!("expected cacheable key, got unknown non-exhaustive outcome"),
    }
}

fn cache_key_request(order: VectorOrder, work_unit: WorkUnit) -> CacheKeyRequest {
    let mut input_hashes = vec![
        named_hash("manifest", "mizar-cache/test/input/v1", 1),
        named_hash("source", "mizar-cache/test/input/v1", 2),
    ];
    let mut dependency_hashes = vec![
        dependency_hash("interface", "dep-a", "module-a", "summary", 3),
        dependency_hash("implementation", "dep-b", "module-b", "body", 4),
    ];
    let mut dependency_slices = vec![
        dependency_slice("visible-interface", "dep-a::module-a", "theorem-a", 5),
        dependency_slice("visible-interface", "dep-b::module-b", "theorem-b", 6),
    ];
    let mut schema_versions = vec![
        schema_version(
            "mizar-cache",
            "record",
            "mizar-cache/cache-record-schema/v1",
        ),
        schema_version("mizar-cache", "key", CACHE_KEY_SCHEMA_VERSION),
    ];
    let mut toolchain = vec![
        compatibility("toolchain", "rustc", "stable"),
        compatibility("toolchain", "mizar-cache", "task-16"),
    ];
    let mut policy = vec![
        compatibility("policy", "verifier", "deterministic"),
        compatibility("policy", "proof-authority", "kernel-only"),
    ];
    let mut diagnostics = vec![diagnostic("proof", 42), diagnostic("key", 41)];

    if matches!(order, VectorOrder::Reverse) {
        input_hashes.reverse();
        dependency_hashes.reverse();
        dependency_slices.reverse();
        schema_versions.reverse();
        toolchain.reverse();
        policy.reverse();
        diagnostics.reverse();
    }

    CacheKeyRequest {
        cache_schema_version: SchemaVersion::new(CACHE_KEY_SCHEMA_VERSION),
        phase: mizar_cache::cache_key::PipelinePhase::new("parse"),
        work_unit,
        source_identity: None,
        input_hashes,
        dependency_hashes,
        dependency_slices,
        config_hash: hash(7),
        schema_versions,
        policy_fingerprint: PolicyFingerprint::new(hash(8)),
        validation_inputs: CacheValidationInputs {
            cache_schema_compatibility: SchemaVersion::new(CACHE_KEY_SCHEMA_VERSION),
            toolchain_compatibility: toolchain,
            dependency_artifacts: Vec::new(),
            footprint_completeness: FootprintCompleteness::Complete,
            uncacheable: false,
            policy_compatibility: policy,
            canonical_vc_fingerprint: None,
            local_context_fingerprint: None,
            dependency_slice_fingerprints: Vec::new(),
            obligation_anchor_fingerprint: None,
            selected_proof_witness_hash: None,
            deterministic_discharge_hash: None,
            proof_reuse_schema_versions: Vec::new(),
            proof_reuse_validation_hash: None,
            proof_reuse_evidence_identities: Vec::new(),
            diagnostic_refs: diagnostics,
        },
    }
}

fn proof_request(
    class: ProofWinnerClass,
    diagnostic_order: VectorOrder,
) -> ProofReuseValidationRequest {
    let current = proof_snapshot(class);
    let mut diagnostic_refs = vec![diagnostic("proof", 42), diagnostic("proof", 41)];
    if matches!(diagnostic_order, VectorOrder::Reverse) {
        diagnostic_refs.reverse();
    }

    ProofReuseValidationRequest {
        cached: current.clone(),
        current,
        environment: proof_environment(),
        diagnostic_refs,
    }
}

fn proof_snapshot(class: ProofWinnerClass) -> ProofReuseMetadataSnapshot {
    let (selected_proof_witness_hash, deterministic_discharge_hash) = match class {
        ProofWinnerClass::KernelVerified => (Some(hash(20)), None),
        ProofWinnerClass::DischargedBuiltin => (None, Some(hash(21))),
        ProofWinnerClass::PolicyPermittedExternal
        | ProofWinnerClass::PolicyAssumed
        | ProofWinnerClass::PolicyOpen
        | ProofWinnerClass::Rejected
        | ProofWinnerClass::NoSelectableEvidence => (None, None),
        _ => (None, None),
    };

    ProofReuseMetadataSnapshot {
        selected_class: class,
        selected_candidate_id: Some("candidate-1".to_owned()),
        obligation_anchor: "anchor-1".to_owned(),
        obligation_fingerprint: hash(1),
        canonical_vc_fingerprint: hash(2),
        local_context_fingerprint: hash(3),
        dependency_slice_fingerprint: hash(4),
        policy_fingerprint: hash(5),
        selected_evidence_hash: Some(hash(6)),
        selected_proof_witness_hash,
        deterministic_discharge_hash,
        accepted_goal_polarity: class
            .is_trusted()
            .then(|| SUPPORTED_ACCEPTED_GOAL_POLARITY.to_owned()),
        trusted_used_axioms_hash: class.is_trusted().then(|| hash(7)),
        selected_candidate_provenance_hash: Some(hash(8)),
        selection_reason: "proof-owner-selected-winner".to_owned(),
        tie_break_key_hash: hash(9),
        dependency_compatibility: Some(ProofReuseDependencyCompatibilitySnapshot {
            dependency_artifact_fingerprint: hash(10),
            dependency_schema_version: ArtifactSchemaVersion::new(1, 0),
            proof_reuse_schema_version: ArtifactSchemaVersion::new(2, 0),
        }),
        proof_reuse_validation_hash: hash(11),
        cache_reuse_predicate_complete: class.is_trusted(),
    }
}

fn proof_environment() -> ProofReuseValidationEnvironment {
    ProofReuseValidationEnvironment {
        cache_schema_version: SchemaVersion::new(CACHE_KEY_SCHEMA_VERSION),
        proof_reuse_schema_versions: vec![schema_version(
            "mizar-cache/proof-reuse",
            "validator",
            PROOF_REUSE_SCHEMA_VERSION,
        )],
        toolchain_compatibility: vec![compatibility(
            "toolchain",
            "mizar-cache-proof-reuse",
            "current",
        )],
        policy_compatibility: vec![compatibility(
            "verifier-policy",
            "proof-reuse-policy",
            "current",
        )],
        dependency_artifacts: vec![DependencyArtifactAvailability {
            package_id: "pkg".to_owned(),
            module_path: "module".to_owned(),
            artifact_kind: "proof-reuse-dependency".to_owned(),
            artifact_path: "artifact/proof-reuse-dependency".to_owned(),
            domain: "mizar-cache/proof-reuse-dependency-artifact/v1".to_owned(),
            digest: hash(12),
        }],
        footprint_completeness: FootprintCompleteness::Complete,
        uncacheable: false,
    }
}

fn named_hash(name: &str, domain: &str, seed: u8) -> NamedHash {
    NamedHash {
        name: name.to_owned(),
        domain: domain.to_owned(),
        digest: hash(seed),
    }
}

fn dependency_hash(
    dependency_kind: &str,
    package_id: &str,
    module_path: &str,
    name: &str,
    seed: u8,
) -> DependencyHash {
    DependencyHash {
        dependency_kind: dependency_kind.to_owned(),
        package_id: package_id.to_owned(),
        module_path: module_path.to_owned(),
        name: name.to_owned(),
        domain: "mizar-cache/test/dependency/v1".to_owned(),
        digest: hash(seed),
    }
}

fn dependency_slice(slice_kind: &str, owner: &str, name: &str, seed: u8) -> DependencySliceHash {
    DependencySliceHash {
        slice_kind: slice_kind.to_owned(),
        owner: owner.to_owned(),
        name: name.to_owned(),
        domain: "mizar-cache/test/dependency-slice/v1".to_owned(),
        digest: hash(seed),
    }
}

fn schema_version(family: &str, name: &str, version: &str) -> NamedSchemaVersion {
    NamedSchemaVersion {
        schema_family: family.to_owned(),
        name: name.to_owned(),
        version: SchemaVersion::new(version),
    }
}

fn compatibility(family: &str, field_name: &str, value: &str) -> CompatibilityField {
    CompatibilityField {
        family: family.to_owned(),
        field_name: field_name.to_owned(),
        value: value.to_owned(),
    }
}

fn diagnostic(kind: &str, seed: u8) -> DiagnosticRefHash {
    DiagnosticRefHash {
        diagnostic_ref_kind: kind.to_owned(),
        diagnostic_ref_hash: hash(seed),
    }
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn temp_cache_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("mizar-cache-{label}-{}", std::process::id()))
}

fn remove_dir_if_exists(path: &Path) {
    match fs::remove_dir_all(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to remove {}: {error}", path.display()),
    }
}
