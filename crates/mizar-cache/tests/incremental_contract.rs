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
        NamedSchemaVersion, PipelinePhase, PolicyFingerprint, ProofReuseEvidenceIdentity,
        SchemaVersion, WorkUnit,
    },
    cache_store::{
        CacheInsertOutcome, CacheLookupOutcome, CacheMiss, CacheRecord, CacheStoreError,
        CacheStoreRoot,
    },
    dependency_fingerprint::{
        DEPENDENCY_FINGERPRINT_SCHEMA_VERSION, DependencyFingerprint, DependencyFootprint,
        DependencyFootprintBuildOutcome, DependencyFootprintBuilder,
        DependencyFootprintCompleteness, DependencyFootprintRequest, DependencySliceFingerprint,
        FingerprintIdentity, FingerprintTargetKind, FootprintOwner, ProofReuseValidationInput,
        ProofReuseValidationState,
    },
    proof_reuse::{
        PROOF_REUSE_SCHEMA_VERSION, ProofReuseDependencyCompatibilitySnapshot,
        ProofReuseMetadataSnapshot, ProofReuseMissReason, ProofReuseValidationEnvironment,
        ProofReuseValidationOutcome, ProofReuseValidationRequest, ProofReuseValidator,
    },
};
use mizar_proof::selection::ProofWinnerClass;
use mizar_session::Hash;

#[test]
fn trusted_incremental_contract_requires_complete_cross_module_validation() {
    let world = TestWorld::new("trusted-contract");
    let footprint = reusable_footprint(DependencyFootprintCompleteness::Complete);
    let slice = footprint.slices[0].as_cache_key_slice();
    let key = cacheable_key(&world, |request| {
        request.dependency_slices = vec![slice.clone()];
        request.validation_inputs.dependency_slice_fingerprints = vec![slice];
        request.validation_inputs.footprint_completeness = FootprintCompleteness::Complete;
    });
    let store = CacheStoreRoot::new(world.cache_root.clone());
    let record = CacheRecord::new_inline(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        b"verified incremental output".to_vec(),
    );

    assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Hit(Box::new(record.clone()))
    );

    let proof = proof_request_with_environment(
        ProofWinnerClass::KernelVerified,
        proof_environment_from_key(&key),
    );
    assert_contract_surfaces_agree(&key, &footprint, &proof);
    let ProofReuseValidationOutcome::Hit(hit) = ProofReuseValidator::validate(&proof) else {
        panic!("kernel-verified proof metadata should validate");
    };
    assert_eq!(hit.selected_class, ProofWinnerClass::KernelVerified);
    assert_eq!(hit.witness_or_discharge_hash, hash(20));
    assert_eq!(hit.proof_reuse_validation_hash, hash(11));
}

#[test]
fn missing_or_unknown_incremental_inputs_fail_closed_before_reuse() {
    let world = TestWorld::new("fail-closed-inputs");
    let store = CacheStoreRoot::new(world.cache_root.clone());

    let incomplete_footprint = uncacheable_footprint(|request| {
        request.requested_completeness = DependencyFootprintCompleteness::IncompleteUncacheable;
    });
    let incomplete_key = uncacheable_key(&world, |request| {
        request.validation_inputs.footprint_completeness =
            FootprintCompleteness::IncompleteUncacheable;
        request.validation_inputs.uncacheable = incomplete_footprint.uncacheable;
    });
    assert_eq!(
        store.lookup(&incomplete_key),
        CacheLookupOutcome::Miss(CacheMiss::IncompleteFootprint)
    );

    let explicit_uncacheable_key = uncacheable_key(&world, |request| {
        request.validation_inputs.uncacheable = true;
    });
    assert_eq!(
        store.lookup(&explicit_uncacheable_key),
        CacheLookupOutcome::Miss(CacheMiss::Uncacheable)
    );

    let unsupported_schema = CacheKeyBuilder::new(base_key_request(&world, |request| {
        request.cache_schema_version = SchemaVersion::new("mizar-cache/cache-key-schema/future");
    }))
    .build();
    assert_eq!(
        store.lookup_key_outcome(&unsupported_schema),
        CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
    );

    assert_insert_rejected(&store, &world, CacheMiss::UnknownToolchain, |request| {
        request.validation_inputs.toolchain_compatibility[0].value = "unknown".to_owned()
    });
    assert_insert_rejected(&store, &world, CacheMiss::PolicyIncompatible, |request| {
        request.validation_inputs.policy_compatibility[0].value = "incompatible".to_owned();
    });

    let artifact_key = cacheable_key(&world, |_| {});
    let record = CacheRecord::new_inline(
        artifact_key.clone(),
        artifact_key
            .validation_inputs
            .toolchain_compatibility
            .clone(),
        b"artifact-sensitive output".to_vec(),
    );
    assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
    fs::write(&world.dependency_artifact_path, b"changed artifact bytes")
        .expect("overwrite dependency artifact");
    assert_eq!(
        store.lookup(&artifact_key),
        CacheLookupOutcome::Miss(CacheMiss::DependencyUnavailable)
    );
}

#[test]
fn proof_reuse_requires_each_architecture_22_validation_field() {
    type Mutation = Box<dyn Fn(&mut ProofReuseValidationRequest)>;

    let cases: Vec<(&str, ProofReuseMissReason, Mutation)> = vec![
        (
            "obligation anchor",
            ProofReuseMissReason::ObligationAnchorMismatch,
            Box::new(|request| request.cached.obligation_anchor = "stale-anchor".to_owned()),
        ),
        (
            "canonical vc fingerprint",
            ProofReuseMissReason::CanonicalVcFingerprintMismatch,
            Box::new(|request| request.cached.canonical_vc_fingerprint = hash(31)),
        ),
        (
            "local context fingerprint",
            ProofReuseMissReason::LocalContextFingerprintMismatch,
            Box::new(|request| request.cached.local_context_fingerprint = hash(32)),
        ),
        (
            "dependency slice fingerprint",
            ProofReuseMissReason::DependencySliceFingerprintMismatch,
            Box::new(|request| request.cached.dependency_slice_fingerprint = hash(33)),
        ),
        (
            "policy fingerprint",
            ProofReuseMissReason::PolicyMismatch,
            Box::new(|request| request.cached.policy_fingerprint = hash(34)),
        ),
        (
            "selected witness hash",
            ProofReuseMissReason::SelectedWitnessHashMismatch,
            Box::new(|request| request.cached.selected_proof_witness_hash = Some(hash(35))),
        ),
        (
            "deterministic discharge hash",
            ProofReuseMissReason::DeterministicDischargeHashMismatch,
            Box::new(|request| {
                request.current = proof_snapshot(ProofWinnerClass::DischargedBuiltin);
                request.cached = request.current.clone();
                request.cached.deterministic_discharge_hash = Some(hash(36));
            }),
        ),
        (
            "proof reuse validation hash",
            ProofReuseMissReason::ProofReuseValidationHashMismatch,
            Box::new(|request| request.cached.proof_reuse_validation_hash = hash(37)),
        ),
        (
            "dependency artifact fingerprint",
            ProofReuseMissReason::DependencyArtifactMismatch,
            Box::new(|request| {
                request
                    .cached
                    .dependency_compatibility
                    .as_mut()
                    .expect("dependency compatibility")
                    .dependency_artifact_fingerprint = hash(38);
            }),
        ),
    ];

    for (label, expected, mutate) in cases {
        let mut request = proof_request(ProofWinnerClass::KernelVerified);
        mutate(&mut request);
        let ProofReuseValidationOutcome::Miss(miss) = ProofReuseValidator::validate(&request)
        else {
            panic!("{label} should fail closed");
        };
        assert_eq!(miss.reason, expected, "{label}");
    }
}

#[test]
fn dependency_footprint_projects_missing_and_external_proof_metadata_to_miss() {
    for state in [
        ProofReuseValidationState::Mismatched,
        ProofReuseValidationState::Missing,
        ProofReuseValidationState::ExternalOnly,
        ProofReuseValidationState::UnsupportedEvidenceKind("new-proof-class".to_owned()),
    ] {
        let footprint = uncacheable_footprint(|request| {
            request.proof_reuse_validation[0].state = state.clone();
        });
        assert_eq!(
            footprint.completeness,
            DependencyFootprintCompleteness::IncompleteUncacheable
        );
        assert!(footprint.uncacheable);
    }

    let unknown_compatibility = uncacheable_footprint(|request| {
        request.compatibility_fields[0].value = "unknown".to_owned();
    });
    assert_eq!(
        unknown_compatibility.completeness,
        DependencyFootprintCompleteness::IncompleteUncacheable
    );
    assert!(
        unknown_compatibility
            .unknown_markers
            .iter()
            .any(|marker| marker.family == "compatibility:toolchain")
    );

    match DependencyFootprintBuilder::new(base_footprint_request(|request| {
        request.schema_version = SchemaVersion::new("mizar-cache/dependency-fingerprint/future");
    }))
    .build()
    {
        DependencyFootprintBuildOutcome::NoFootprint(_) => {}
        other => panic!("unknown dependency-footprint schema must produce no footprint: {other:?}"),
    }
}

#[test]
fn cache_deletion_and_diagnostic_order_are_non_semantic() {
    let world = TestWorld::new("deletion-nonsemantic");
    let store = CacheStoreRoot::new(world.cache_root.clone());
    let key = cacheable_key(&world, |_| {});
    let record = CacheRecord::new_inline(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        b"reusable output".to_vec(),
    );
    assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
    let proof_hit = ProofReuseValidator::validate(&proof_request(ProofWinnerClass::KernelVerified));

    fs::remove_file(store.record_path(&key)).expect("remove record");
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::NotFound)
    );
    assert_eq!(
        ProofReuseValidator::validate(&proof_request(ProofWinnerClass::KernelVerified)),
        proof_hit
    );
    assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Hit(Box::new(record.clone()))
    );

    let mut reordered = proof_request(ProofWinnerClass::KernelVerified);
    reordered.diagnostic_refs.reverse();
    assert_eq!(ProofReuseValidator::validate(&reordered), proof_hit);
}

#[test]
fn externally_attested_evidence_never_becomes_trusted_reuse() {
    let external = proof_request(ProofWinnerClass::PolicyPermittedExternal);
    let ProofReuseValidationOutcome::Miss(external_miss) = ProofReuseValidator::validate(&external)
    else {
        panic!("externally attested metadata must not validate as a reuse hit");
    };
    assert_eq!(external_miss.reason, ProofReuseMissReason::NonReusableClass);

    let mut synthesized = proof_request(ProofWinnerClass::PolicyPermittedExternal);
    synthesized.cached.trusted_used_axioms_hash = Some(hash(90));
    let ProofReuseValidationOutcome::Miss(synthesized_miss) =
        ProofReuseValidator::validate(&synthesized)
    else {
        panic!("external metadata with trusted axiom refs must fail closed");
    };
    assert_eq!(
        synthesized_miss.reason,
        ProofReuseMissReason::SynthesizedTrustedAxiomSetReference
    );

    let world = TestWorld::new("external-evidence");
    let key = uncacheable_key(&world, |request| {
        request.validation_inputs.proof_reuse_evidence_identities[0].evidence_kind =
            "policy_permitted_external".to_owned();
    });
    let store = CacheStoreRoot::new(world.cache_root.clone());
    assert_eq!(
        store.lookup(&key),
        CacheLookupOutcome::Miss(CacheMiss::Uncacheable)
    );
}

struct TestWorld {
    cache_root: PathBuf,
    dependency_artifact_path: PathBuf,
    dependency_artifact_domain: String,
}

impl TestWorld {
    fn new(label: &str) -> Self {
        let root = temp_root(label);
        remove_dir_if_exists(&root);
        let cache_root = root.join("cache");
        let dependency_artifact_path = root.join("artifacts/dep.mizir.json");
        fs::create_dir_all(
            dependency_artifact_path
                .parent()
                .expect("dependency artifact path has parent"),
        )
        .expect("create dependency artifact parent");
        fs::write(&dependency_artifact_path, dependency_artifact_bytes())
            .expect("write dependency artifact");
        Self {
            cache_root,
            dependency_artifact_path,
            dependency_artifact_domain: "mizar-cache/task-20/dependency-artifact/v1".to_owned(),
        }
    }
}

impl Drop for TestWorld {
    fn drop(&mut self) {
        if let Some(root) = self.cache_root.parent() {
            let _ = fs::remove_dir_all(root);
        }
    }
}

fn assert_insert_rejected(
    store: &CacheStoreRoot,
    world: &TestWorld,
    expected: CacheMiss,
    mutate: impl FnOnce(&mut CacheKeyRequest),
) {
    let key = cacheable_key(world, mutate);
    let record = CacheRecord::new_inline(
        key.clone(),
        key.validation_inputs.toolchain_compatibility.clone(),
        b"rejected output".to_vec(),
    );
    match store.insert(&record) {
        Err(CacheStoreError::InvalidRecord { reason }) => assert_eq!(reason, expected),
        other => panic!("expected invalid record {expected:?}, got {other:?}"),
    }
}

fn cacheable_key(world: &TestWorld, mutate: impl FnOnce(&mut CacheKeyRequest)) -> CacheKey {
    match CacheKeyBuilder::new(base_key_request(world, mutate)).build() {
        CacheKeyBuildOutcome::Cacheable(key) => key,
        other => panic!("expected cacheable key, got {other:?}"),
    }
}

fn uncacheable_key(world: &TestWorld, mutate: impl FnOnce(&mut CacheKeyRequest)) -> CacheKey {
    match CacheKeyBuilder::new(base_key_request(world, mutate)).build() {
        CacheKeyBuildOutcome::Uncacheable(key) => key,
        other => panic!("expected uncacheable key, got {other:?}"),
    }
}

fn base_key_request(
    world: &TestWorld,
    mutate: impl FnOnce(&mut CacheKeyRequest),
) -> CacheKeyRequest {
    let dependency_slice = dependency_slice("vc", "pkg::module#anchor-1", "obligation-1", 4);
    let mut request = CacheKeyRequest {
        cache_schema_version: SchemaVersion::new(CACHE_KEY_SCHEMA_VERSION),
        phase: PipelinePhase::new("proof"),
        work_unit: WorkUnit::new("task-20/proof-reuse"),
        source_identity: None,
        input_hashes: vec![named_hash("source", "mizar-cache/task-20/input/v1", 1)],
        dependency_hashes: vec![dependency_hash(
            "interface",
            "dep",
            "dep.module",
            "summary",
            2,
        )],
        dependency_slices: vec![dependency_slice.clone()],
        config_hash: hash(3),
        schema_versions: vec![
            schema_version("mizar-cache", "key", CACHE_KEY_SCHEMA_VERSION),
            schema_version(
                "mizar-cache",
                "dependency-footprint",
                DEPENDENCY_FINGERPRINT_SCHEMA_VERSION,
            ),
            schema_version(
                "mizar-cache/proof-reuse",
                "validator",
                PROOF_REUSE_SCHEMA_VERSION,
            ),
        ],
        policy_fingerprint: PolicyFingerprint::new(hash(5)),
        validation_inputs: CacheValidationInputs {
            cache_schema_compatibility: SchemaVersion::new(CACHE_KEY_SCHEMA_VERSION),
            toolchain_compatibility: vec![compatibility("toolchain", "mizar-cache", "current")],
            dependency_artifacts: vec![DependencyArtifactAvailability {
                package_id: "dep".to_owned(),
                module_path: "dep.module".to_owned(),
                artifact_kind: "mizir".to_owned(),
                artifact_path: world
                    .dependency_artifact_path
                    .to_string_lossy()
                    .into_owned(),
                domain: world.dependency_artifact_domain.clone(),
                digest: dependency_artifact_hash(
                    &world.dependency_artifact_domain,
                    dependency_artifact_bytes(),
                ),
            }],
            footprint_completeness: FootprintCompleteness::Complete,
            uncacheable: false,
            policy_compatibility: vec![compatibility(
                "verifier-policy",
                "proof-authority",
                "kernel-only",
            )],
            canonical_vc_fingerprint: Some(hash(2)),
            local_context_fingerprint: Some(hash(3)),
            dependency_slice_fingerprints: vec![dependency_slice],
            obligation_anchor_fingerprint: Some(hash(1)),
            selected_proof_witness_hash: Some(named_hash(
                "selected-witness",
                "mizar-cache/task-20/witness/v1",
                20,
            )),
            deterministic_discharge_hash: None,
            proof_reuse_schema_versions: vec![schema_version(
                "mizar-cache/proof-reuse",
                "validator",
                PROOF_REUSE_SCHEMA_VERSION,
            )],
            proof_reuse_validation_hash: Some(named_hash(
                "proof-reuse-validation",
                "mizar-cache/task-20/proof-validation/v1",
                11,
            )),
            proof_reuse_evidence_identities: vec![ProofReuseEvidenceIdentity {
                obligation_anchor_fingerprint: hash(1),
                evidence_kind: "kernel_verified".to_owned(),
                witness_or_discharge_domain: "mizar-cache/task-20/witness/v1".to_owned(),
                witness_or_discharge_digest: hash(20),
            }],
            diagnostic_refs: vec![diagnostic("proof", 42), diagnostic("cache", 41)],
        },
    };
    mutate(&mut request);
    request
}

fn reusable_footprint(completeness: DependencyFootprintCompleteness) -> DependencyFootprint {
    match DependencyFootprintBuilder::new(base_footprint_request(|request| {
        request.requested_completeness = completeness;
    }))
    .build()
    {
        DependencyFootprintBuildOutcome::Reusable(footprint) => footprint,
        other => panic!("expected reusable footprint, got {other:?}"),
    }
}

fn uncacheable_footprint(
    mutate: impl FnOnce(&mut DependencyFootprintRequest),
) -> DependencyFootprint {
    match DependencyFootprintBuilder::new(base_footprint_request(mutate)).build() {
        DependencyFootprintBuildOutcome::Uncacheable(footprint) => footprint,
        other => panic!("expected uncacheable footprint, got {other:?}"),
    }
}

fn assert_contract_surfaces_agree(
    key: &CacheKey,
    footprint: &DependencyFootprint,
    proof: &ProofReuseValidationRequest,
) {
    let current = &proof.current;
    let witness = current
        .selected_proof_witness_hash
        .expect("trusted proof fixture has witness hash");
    let key_witness = key
        .validation_inputs
        .selected_proof_witness_hash
        .as_ref()
        .expect("key carries selected witness hash");
    let key_validation = key
        .validation_inputs
        .proof_reuse_validation_hash
        .as_ref()
        .expect("key carries proof-reuse validation hash");
    let footprint_validation = &footprint.proof_reuse_validation[0];
    let footprint_validation_hash = footprint_validation
        .validation_hash
        .as_ref()
        .expect("footprint carries proof-reuse validation hash");
    let footprint_witness = footprint_validation
        .witness_or_discharge_hash
        .as_ref()
        .expect("footprint carries witness hash");

    assert_eq!(
        proof.environment.cache_schema_version,
        key.cache_schema_version
    );
    assert_eq!(
        proof.environment.cache_schema_version,
        key.validation_inputs.cache_schema_compatibility
    );
    assert_eq!(
        proof.environment.proof_reuse_schema_versions,
        key.validation_inputs.proof_reuse_schema_versions
    );
    assert_eq!(
        proof.environment.toolchain_compatibility,
        key.validation_inputs.toolchain_compatibility
    );
    assert_eq!(
        proof.environment.policy_compatibility,
        key.validation_inputs.policy_compatibility
    );
    assert_eq!(
        proof.environment.dependency_artifacts,
        key.validation_inputs.dependency_artifacts
    );
    assert_eq!(
        proof.environment.footprint_completeness,
        key.validation_inputs.footprint_completeness
    );
    assert_eq!(
        proof.environment.uncacheable,
        key.validation_inputs.uncacheable
    );

    assert_eq!(
        key.validation_inputs.obligation_anchor_fingerprint,
        Some(current.obligation_fingerprint)
    );
    assert_eq!(
        key.validation_inputs.canonical_vc_fingerprint,
        Some(current.canonical_vc_fingerprint)
    );
    assert_eq!(
        key.validation_inputs.local_context_fingerprint,
        Some(current.local_context_fingerprint)
    );
    assert_eq!(key.policy_fingerprint.hash(), current.policy_fingerprint);
    assert_eq!(key_witness.digest, witness);
    assert_eq!(key_validation.digest, current.proof_reuse_validation_hash);

    let key_slice = &key.validation_inputs.dependency_slice_fingerprints[0];
    let footprint_slice = footprint.slices[0].as_cache_key_slice();
    assert_eq!(key_slice, &footprint_slice);
    assert_eq!(key_slice.digest, current.dependency_slice_fingerprint);
    assert_eq!(key.dependency_slices, vec![footprint_slice]);

    let evidence = &key.validation_inputs.proof_reuse_evidence_identities[0];
    assert_eq!(
        evidence.obligation_anchor_fingerprint,
        current.obligation_fingerprint
    );
    assert_eq!(evidence.witness_or_discharge_digest, witness);

    assert_eq!(
        footprint_validation.state,
        ProofReuseValidationState::Complete
    );
    assert_eq!(
        footprint_validation_hash.digest,
        current.proof_reuse_validation_hash
    );
    assert_eq!(footprint_witness.digest, witness);
}

fn base_footprint_request(
    mutate: impl FnOnce(&mut DependencyFootprintRequest),
) -> DependencyFootprintRequest {
    let mut request = DependencyFootprintRequest {
        schema_version: SchemaVersion::new(DEPENDENCY_FINGERPRINT_SCHEMA_VERSION),
        owner: FootprintOwner {
            package_id: "pkg".to_owned(),
            module_path: "module".to_owned(),
            origin_id: Some("theorem-1".to_owned()),
            language_edition: Some("2025".to_owned()),
            lockfile_identity: Some("lockfile-1".to_owned()),
        },
        phase: PipelinePhase::new("proof"),
        fingerprints: vec![
            dependency_fingerprint(
                FingerprintTargetKind::Source,
                "source",
                "mizar-cache/task-20/source/v1",
                1,
            ),
            dependency_fingerprint(
                FingerprintTargetKind::PolicyToolchain,
                "policy-toolchain",
                "mizar-cache/task-20/policy/v1",
                2,
            ),
        ],
        slices: vec![DependencySliceFingerprint {
            slice_kind: "vc".to_owned(),
            owner: "pkg::module#anchor-1".to_owned(),
            name: "obligation-1".to_owned(),
            domain: "mizar-cache/task-20/dependency-slice/v1".to_owned(),
            digest: hash(4),
            completeness: DependencyFootprintCompleteness::Complete,
        }],
        compatibility_fields: vec![compatibility("toolchain", "mizar-cache", "current")],
        proof_reuse_validation: vec![ProofReuseValidationInput {
            name: "obligation-1".to_owned(),
            state: ProofReuseValidationState::Complete,
            validation_hash: Some(named_hash(
                "proof-reuse-validation",
                "mizar-cache/task-20/proof-validation/v1",
                11,
            )),
            witness_or_discharge_hash: Some(named_hash(
                "selected-witness",
                "mizar-cache/task-20/witness/v1",
                20,
            )),
            metadata_schema_versions: vec![schema_version(
                "mizar-proof",
                "status-reuse",
                "mizar-proof/status-reuse/v1",
            )],
        }],
        unknown_markers: Vec::new(),
        requested_completeness: DependencyFootprintCompleteness::Complete,
        uncacheable: false,
    };
    mutate(&mut request);
    request
}

fn dependency_fingerprint(
    target: FingerprintTargetKind,
    target_name: &str,
    value_domain: &str,
    seed: u8,
) -> DependencyFingerprint {
    DependencyFingerprint {
        target,
        identity: FingerprintIdentity {
            package_id: "pkg".to_owned(),
            module_path: "module".to_owned(),
            origin_id: Some("theorem-1".to_owned()),
            target_name: target_name.to_owned(),
            schema_family: value_domain.to_owned(),
            language_edition: Some("2025".to_owned()),
            lockfile_identity: Some("lockfile-1".to_owned()),
        },
        value_domain: value_domain.to_owned(),
        value_hash: hash(seed),
        schema_version: SchemaVersion::new("mizar-cache/task-20/schema/v1"),
        importer_visible: false,
    }
}

fn proof_request(class: ProofWinnerClass) -> ProofReuseValidationRequest {
    proof_request_with_environment(class, proof_environment())
}

fn proof_request_with_environment(
    class: ProofWinnerClass,
    environment: ProofReuseValidationEnvironment,
) -> ProofReuseValidationRequest {
    let current = proof_snapshot(class);
    ProofReuseValidationRequest {
        cached: current.clone(),
        current,
        environment,
        diagnostic_refs: vec![diagnostic("proof", 42), diagnostic("cache", 41)],
    }
}

fn proof_environment_from_key(key: &CacheKey) -> ProofReuseValidationEnvironment {
    ProofReuseValidationEnvironment {
        cache_schema_version: key.cache_schema_version.clone(),
        proof_reuse_schema_versions: key.validation_inputs.proof_reuse_schema_versions.clone(),
        toolchain_compatibility: key.validation_inputs.toolchain_compatibility.clone(),
        policy_compatibility: key.validation_inputs.policy_compatibility.clone(),
        dependency_artifacts: key.validation_inputs.dependency_artifacts.clone(),
        footprint_completeness: key.validation_inputs.footprint_completeness,
        uncacheable: key.validation_inputs.uncacheable,
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
            "proof-authority",
            "kernel-only",
        )],
        dependency_artifacts: vec![DependencyArtifactAvailability {
            package_id: "dep".to_owned(),
            module_path: "dep.module".to_owned(),
            artifact_kind: "mizir".to_owned(),
            artifact_path: "artifact/dep.mizir.json".to_owned(),
            domain: "mizar-cache/task-20/dependency-artifact/v1".to_owned(),
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
        domain: "mizar-cache/task-20/dependency/v1".to_owned(),
        digest: hash(seed),
    }
}

fn dependency_slice(slice_kind: &str, owner: &str, name: &str, seed: u8) -> DependencySliceHash {
    DependencySliceHash {
        slice_kind: slice_kind.to_owned(),
        owner: owner.to_owned(),
        name: name.to_owned(),
        domain: "mizar-cache/task-20/dependency-slice/v1".to_owned(),
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

fn dependency_artifact_bytes() -> &'static [u8] {
    br#"{"interface":"task-20"}"#
}

fn dependency_artifact_hash(domain: &str, bytes: &[u8]) -> Hash {
    let mut hasher = blake3::Hasher::new();
    write_hash_part(&mut hasher, domain.as_bytes());
    write_hash_part(&mut hasher, bytes);
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

fn write_hash_part(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "mizar-cache-task-20-{label}-{}",
        std::process::id()
    ))
}

fn remove_dir_if_exists(path: &Path) {
    match fs::remove_dir_all(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to remove {}: {error}", path.display()),
    }
}
