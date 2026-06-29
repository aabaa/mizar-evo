use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn key_builder_is_deterministic_and_sorts_all_vectors() {
    let mut first = request();
    first.input_hashes.push(first.input_hashes[0].clone());
    first
        .dependency_hashes
        .push(first.dependency_hashes[0].clone());
    first
        .dependency_slices
        .push(first.dependency_slices[0].clone());
    first.schema_versions.push(first.schema_versions[0].clone());
    first
        .validation_inputs
        .toolchain_compatibility
        .push(first.validation_inputs.toolchain_compatibility[0].clone());
    first
        .validation_inputs
        .dependency_artifacts
        .push(first.validation_inputs.dependency_artifacts[0].clone());
    first
        .validation_inputs
        .policy_compatibility
        .push(first.validation_inputs.policy_compatibility[0].clone());
    first
        .validation_inputs
        .dependency_slice_fingerprints
        .push(first.validation_inputs.dependency_slice_fingerprints[0].clone());
    first
        .validation_inputs
        .proof_reuse_schema_versions
        .push(first.validation_inputs.proof_reuse_schema_versions[0].clone());
    first
        .validation_inputs
        .proof_reuse_evidence_identities
        .push(first.validation_inputs.proof_reuse_evidence_identities[0].clone());
    first
        .validation_inputs
        .diagnostic_refs
        .push(first.validation_inputs.diagnostic_refs[0].clone());
    first.input_hashes.reverse();
    first.dependency_hashes.reverse();
    first.dependency_slices.reverse();
    first.schema_versions.reverse();
    first.validation_inputs.toolchain_compatibility.reverse();
    first.validation_inputs.dependency_artifacts.reverse();
    first.validation_inputs.policy_compatibility.reverse();
    first
        .validation_inputs
        .dependency_slice_fingerprints
        .reverse();
    first
        .validation_inputs
        .proof_reuse_schema_versions
        .reverse();
    first
        .validation_inputs
        .proof_reuse_evidence_identities
        .reverse();
    first.validation_inputs.diagnostic_refs.reverse();

    let first = cacheable(first);
    let second = cacheable(request());

    assert_eq!(first.final_hash, second.final_hash);
    assert_eq!(first.input_hashes, second.input_hashes);
    assert_eq!(first.dependency_hashes, second.dependency_hashes);
    assert_eq!(first.dependency_slices, second.dependency_slices);
    assert_eq!(first.schema_versions, second.schema_versions);
    assert_eq!(
        first.validation_inputs.toolchain_compatibility,
        second.validation_inputs.toolchain_compatibility
    );
    assert_eq!(
        first.validation_inputs.dependency_artifacts,
        second.validation_inputs.dependency_artifacts
    );
    assert_eq!(
        first.validation_inputs.policy_compatibility,
        second.validation_inputs.policy_compatibility
    );
    assert_eq!(
        first.validation_inputs.dependency_slice_fingerprints,
        second.validation_inputs.dependency_slice_fingerprints
    );
    assert_eq!(
        first.validation_inputs.proof_reuse_schema_versions,
        second.validation_inputs.proof_reuse_schema_versions
    );
    assert_eq!(
        first.validation_inputs.proof_reuse_evidence_identities,
        second.validation_inputs.proof_reuse_evidence_identities
    );
    assert_eq!(
        first.validation_inputs.diagnostic_refs,
        second.validation_inputs.diagnostic_refs
    );
}

#[test]
fn every_semantic_field_changes_final_hash() {
    let base = key_for(request()).final_hash;
    let cases = vec![
        mutate(|request| request.phase = PipelinePhase::new("proof")),
        mutate(|request| request.work_unit = WorkUnit::new("module:beta")),
        mutate(|request| request.source_identity = None),
        mutate(|request| {
            request
                .source_identity
                .as_mut()
                .expect("source identity")
                .package_id = PackageId::new("other-pkg");
        }),
        mutate(|request| {
            request
                .source_identity
                .as_mut()
                .expect("source identity")
                .module_path = ModulePath::new("pkg.beta");
        }),
        mutate(|request| {
            request
                .source_identity
                .as_mut()
                .expect("source identity")
                .normalized_source_path = normalized_path("src/beta.miz");
        }),
        mutate(|request| {
            request
                .source_identity
                .as_mut()
                .expect("source identity")
                .source_hash = hash(99);
        }),
        mutate(|request| {
            request
                .source_identity
                .as_mut()
                .expect("source identity")
                .language_edition = Edition::new("2027");
        }),
        mutate(|request| request.input_hashes[0].name = "source-v2".to_owned()),
        mutate(|request| request.input_hashes[0].domain = "domain/source-v2".to_owned()),
        mutate(|request| request.input_hashes[0].digest = hash(100)),
        mutate(|request| request.dependency_hashes[0].dependency_kind = "summary".to_owned()),
        mutate(|request| request.dependency_hashes[0].package_id = "dep2".to_owned()),
        mutate(|request| request.dependency_hashes[0].module_path = "dep.beta".to_owned()),
        mutate(|request| request.dependency_hashes[0].name = "artifact-v2".to_owned()),
        mutate(|request| request.dependency_hashes[0].domain = "domain/summary".to_owned()),
        mutate(|request| request.dependency_hashes[0].digest = hash(101)),
        mutate(|request| request.dependency_slices[0].slice_kind = "definition".to_owned()),
        mutate(|request| request.dependency_slices[0].owner = "pkg.alpha::D1".to_owned()),
        mutate(|request| request.dependency_slices[0].name = "expanded".to_owned()),
        mutate(|request| request.dependency_slices[0].domain = "domain/definition".to_owned()),
        mutate(|request| request.dependency_slices[0].digest = hash(102)),
        mutate(|request| request.config_hash = hash(103)),
        mutate(|request| request.schema_versions[0].schema_family = "vc-ir-v2".to_owned()),
        mutate(|request| request.schema_versions[0].name = "output-v2".to_owned()),
        mutate(|request| request.schema_versions[0].version = SchemaVersion::new("schema/v2")),
        mutate(|request| request.policy_fingerprint = PolicyFingerprint::new(hash(104))),
        mutate(|request| {
            request.validation_inputs.cache_schema_compatibility =
                SchemaVersion::new("mizar-cache/cache-key-schema/v1+compat");
        }),
        mutate(|request| request.validation_inputs.canonical_vc_fingerprint = Some(hash(105))),
        mutate(|request| request.validation_inputs.local_context_fingerprint = Some(hash(106))),
        mutate(|request| {
            request.validation_inputs.dependency_artifacts[0].package_id = "dep2".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.dependency_artifacts[0].module_path = "dep.beta".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.dependency_artifacts[0].artifact_kind = "summary".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.dependency_artifacts[0].artifact_path =
                "build/dep.beta.summary.json".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.dependency_artifacts[0].domain =
                "artifact-summary".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.dependency_artifacts[0].digest = hash(107);
        }),
        mutate(|request| {
            request.validation_inputs.footprint_completeness =
                FootprintCompleteness::IncompleteUncacheable;
        }),
        mutate(|request| request.validation_inputs.uncacheable = true),
        mutate(|request| {
            request.validation_inputs.policy_compatibility[0].family = "proof-policy-v2".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.policy_compatibility[0].field_name =
                "allow_external".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.policy_compatibility[0].value = "false".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.dependency_slice_fingerprints[0].slice_kind =
                "proof".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.dependency_slice_fingerprints[0].owner =
                "pkg.alpha::T2".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.dependency_slice_fingerprints[0].name = "premise".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.dependency_slice_fingerprints[0].domain =
                "domain/proof".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.dependency_slice_fingerprints[0].digest = hash(108);
        }),
        mutate(|request| {
            request.validation_inputs.obligation_anchor_fingerprint = Some(hash(109));
        }),
        mutate(|request| {
            request
                .validation_inputs
                .selected_proof_witness_hash
                .as_mut()
                .expect("selected witness")
                .name = "selected-witness-v2".to_owned();
        }),
        mutate(|request| {
            request
                .validation_inputs
                .selected_proof_witness_hash
                .as_mut()
                .expect("selected witness")
                .domain = "domain/selected-witness-v2".to_owned();
        }),
        mutate(|request| {
            request
                .validation_inputs
                .selected_proof_witness_hash
                .as_mut()
                .expect("selected witness")
                .digest = hash(110);
        }),
        mutate(|request| {
            request
                .validation_inputs
                .deterministic_discharge_hash
                .as_mut()
                .expect("deterministic discharge")
                .name = "deterministic-discharge-v2".to_owned();
        }),
        mutate(|request| {
            request
                .validation_inputs
                .deterministic_discharge_hash
                .as_mut()
                .expect("deterministic discharge")
                .domain = "domain/deterministic-discharge-v2".to_owned();
        }),
        mutate(|request| {
            request
                .validation_inputs
                .deterministic_discharge_hash
                .as_mut()
                .expect("deterministic discharge")
                .digest = hash(111);
        }),
        mutate(|request| {
            request.validation_inputs.proof_reuse_schema_versions[0].schema_family =
                "proof-reuse-v2".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.proof_reuse_schema_versions[0].name =
                "metadata-v2".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.proof_reuse_schema_versions[0].version =
                SchemaVersion::new("2.0");
        }),
        mutate(|request| {
            request
                .validation_inputs
                .proof_reuse_validation_hash
                .as_mut()
                .expect("proof reuse validation")
                .name = "proof-reuse-validation-v2".to_owned();
        }),
        mutate(|request| {
            request
                .validation_inputs
                .proof_reuse_validation_hash
                .as_mut()
                .expect("proof reuse validation")
                .domain = "domain/proof-reuse-validation-v2".to_owned();
        }),
        mutate(|request| {
            request
                .validation_inputs
                .proof_reuse_validation_hash
                .as_mut()
                .expect("proof reuse validation")
                .digest = hash(112);
        }),
        mutate(|request| {
            request.validation_inputs.proof_reuse_evidence_identities[0]
                .obligation_anchor_fingerprint = hash(115);
        }),
        mutate(|request| {
            request.validation_inputs.proof_reuse_evidence_identities[0].evidence_kind =
                "KernelVerified".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.proof_reuse_evidence_identities[0]
                .witness_or_discharge_domain = "witness-v2".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.proof_reuse_evidence_identities[0]
                .witness_or_discharge_digest = hash(113);
        }),
        mutate(|request| {
            request.validation_inputs.diagnostic_refs[0].diagnostic_ref_kind =
                "other_explanation".to_owned();
        }),
        mutate(|request| {
            request.validation_inputs.diagnostic_refs[0].diagnostic_ref_hash = hash(114);
        }),
        mutate(|request| {
            request
                .validation_inputs
                .toolchain_compatibility
                .push(CompatibilityField {
                    family: "rustc".to_owned(),
                    field_name: "version".to_owned(),
                    value: "1.99".to_owned(),
                });
        }),
    ];

    for changed in cases {
        assert_ne!(base, key_for(changed).final_hash);
    }
}

#[test]
fn conflicting_duplicate_identity_rejects_each_canonical_collection() {
    let mut cases = Vec::new();

    let mut conflicting = request();
    conflicting.input_hashes.push(NamedHash {
        name: conflicting.input_hashes[0].name.clone(),
        domain: conflicting.input_hashes[0].domain.clone(),
        digest: hash(250),
    });
    cases.push((conflicting, "input_hashes"));

    let mut conflicting = request();
    conflicting.dependency_hashes[0].digest = hash(251);
    conflicting
        .dependency_hashes
        .push(request().dependency_hashes[0].clone());
    cases.push((conflicting, "dependency_hashes"));

    let mut conflicting = request();
    conflicting.dependency_slices[0].digest = hash(252);
    conflicting
        .dependency_slices
        .push(request().dependency_slices[0].clone());
    cases.push((conflicting, "dependency_slices"));

    let mut conflicting = request();
    conflicting.schema_versions[0].version = SchemaVersion::new("9.9");
    conflicting
        .schema_versions
        .push(request().schema_versions[0].clone());
    cases.push((conflicting, "schema_versions"));

    let mut conflicting = request();
    conflicting.validation_inputs.toolchain_compatibility[0].value = "other".to_owned();
    conflicting
        .validation_inputs
        .toolchain_compatibility
        .push(request().validation_inputs.toolchain_compatibility[0].clone());
    cases.push((conflicting, "toolchain_compatibility"));

    let mut conflicting = request();
    conflicting.validation_inputs.dependency_artifacts[0].digest = hash(253);
    conflicting
        .validation_inputs
        .dependency_artifacts
        .push(request().validation_inputs.dependency_artifacts[0].clone());
    cases.push((conflicting, "dependency_artifacts"));

    let mut conflicting = request();
    conflicting.validation_inputs.policy_compatibility[0].value = "other".to_owned();
    conflicting
        .validation_inputs
        .policy_compatibility
        .push(request().validation_inputs.policy_compatibility[0].clone());
    cases.push((conflicting, "policy_compatibility"));

    let mut conflicting = request();
    conflicting.validation_inputs.dependency_slice_fingerprints[0].digest = hash(254);
    conflicting
        .validation_inputs
        .dependency_slice_fingerprints
        .push(request().validation_inputs.dependency_slice_fingerprints[0].clone());
    cases.push((conflicting, "dependency_slice_fingerprints"));

    let mut conflicting = request();
    conflicting.validation_inputs.proof_reuse_schema_versions[0].version =
        SchemaVersion::new("9.9");
    conflicting
        .validation_inputs
        .proof_reuse_schema_versions
        .push(request().validation_inputs.proof_reuse_schema_versions[0].clone());
    cases.push((conflicting, "proof_reuse_schema_versions"));

    let mut conflicting = request();
    conflicting
        .validation_inputs
        .proof_reuse_evidence_identities[0]
        .witness_or_discharge_digest = hash(255);
    conflicting
        .validation_inputs
        .proof_reuse_evidence_identities
        .push(request().validation_inputs.proof_reuse_evidence_identities[0].clone());
    cases.push((conflicting, "proof_reuse_evidence_identities"));

    for (request, collection) in cases {
        assert!(matches!(
            CacheKeyBuilder::new(request).build(),
            CacheKeyBuildOutcome::NoKey(CacheKeyBuildRejection::ConflictingDuplicate {
                collection: actual,
                ..
            }) if actual == collection
        ));
    }
}

#[test]
fn unsupported_schema_empty_identities_and_incomplete_proof_reuse_fail_closed() {
    let mut unsupported = request();
    unsupported.cache_schema_version = SchemaVersion::new("mizar-cache/cache-key-schema/v99");
    assert!(matches!(
        CacheKeyBuilder::new(unsupported).build(),
        CacheKeyBuildOutcome::NoKey(CacheKeyBuildRejection::UnsupportedCacheKeySchema { .. })
    ));

    let mut empty_phase = request();
    empty_phase.phase = PipelinePhase::new("");
    assert!(matches!(
        CacheKeyBuilder::new(empty_phase).build(),
        CacheKeyBuildOutcome::NoKey(CacheKeyBuildRejection::MissingRequiredIdentity {
            field: "phase"
        })
    ));

    let mut empty_name = request();
    empty_name.input_hashes[0].name.clear();
    assert!(matches!(
        CacheKeyBuilder::new(empty_name).build(),
        CacheKeyBuildOutcome::NoKey(CacheKeyBuildRejection::MissingRequiredIdentity {
            field: "input_hashes.name"
        })
    ));

    let mut missing_witness = request();
    missing_witness
        .validation_inputs
        .selected_proof_witness_hash = None;
    assert!(uncacheable(missing_witness).validation_inputs.uncacheable);

    let mut missing_discharge = request();
    missing_discharge
        .validation_inputs
        .deterministic_discharge_hash = None;
    assert!(uncacheable(missing_discharge).validation_inputs.uncacheable);

    let mut camel_case_witness = request();
    camel_case_witness
        .validation_inputs
        .proof_reuse_evidence_identities[0]
        .evidence_kind = "KernelVerified".to_owned();
    camel_case_witness
        .validation_inputs
        .selected_proof_witness_hash = None;
    assert!(
        uncacheable(camel_case_witness)
            .validation_inputs
            .uncacheable
    );

    let mut missing_vc = request();
    missing_vc.validation_inputs.canonical_vc_fingerprint = None;
    assert!(uncacheable(missing_vc).validation_inputs.uncacheable);

    let mut unknown_evidence = request();
    unknown_evidence
        .validation_inputs
        .proof_reuse_evidence_identities[0]
        .evidence_kind = "externally_attested".to_owned();
    assert!(uncacheable(unknown_evidence).validation_inputs.uncacheable);
}

#[test]
fn uncacheable_marker_and_incomplete_footprint_produce_miss_outcome() {
    let mut explicit = request();
    explicit.validation_inputs.uncacheable = true;
    let explicit = uncacheable(explicit);
    assert!(explicit.validation_inputs.uncacheable);

    let mut incomplete = request();
    incomplete.validation_inputs.footprint_completeness =
        FootprintCompleteness::IncompleteUncacheable;
    let incomplete = uncacheable(incomplete);
    assert!(incomplete.validation_inputs.uncacheable);
    assert_eq!(
        incomplete.validation_inputs.footprint_completeness,
        FootprintCompleteness::IncompleteUncacheable
    );
}

#[test]
fn diagnostic_refs_participate_only_when_supplied_and_nondeterministic_inputs_are_absent() {
    let base = cacheable(request()).final_hash;
    let mut changed = request();
    changed
        .validation_inputs
        .diagnostic_refs
        .push(DiagnosticRefHash {
            diagnostic_ref_kind: "cache_miss_explanation".to_owned(),
            diagnostic_ref_hash: hash(99),
        });
    assert_ne!(
        base,
        key_for(changed).final_hash,
        "diagnostic refs are hashed only when explicitly supplied as miss explanation refs"
    );

    let same_again = request();
    assert_eq!(base, cacheable(same_again).final_hash);
}

fn mutate(mut f: impl FnMut(&mut CacheKeyRequest)) -> CacheKeyRequest {
    let mut request = request();
    f(&mut request);
    request
}

fn cacheable(request: CacheKeyRequest) -> CacheKey {
    match CacheKeyBuilder::new(request).build() {
        CacheKeyBuildOutcome::Cacheable(key) => key,
        other => panic!("expected cacheable key, got {other:?}"),
    }
}

fn uncacheable(request: CacheKeyRequest) -> CacheKey {
    match CacheKeyBuilder::new(request).build() {
        CacheKeyBuildOutcome::Uncacheable(key) => key,
        other => panic!("expected uncacheable key, got {other:?}"),
    }
}

fn key_for(request: CacheKeyRequest) -> CacheKey {
    match CacheKeyBuilder::new(request).build() {
        CacheKeyBuildOutcome::Cacheable(key) | CacheKeyBuildOutcome::Uncacheable(key) => key,
        other => panic!("expected key-bearing outcome, got {other:?}"),
    }
}

fn request() -> CacheKeyRequest {
    CacheKeyRequest {
        cache_schema_version: SchemaVersion::default(),
        phase: PipelinePhase::new("vc"),
        work_unit: WorkUnit::new("module:alpha#vc:1"),
        source_identity: Some(SourceIdentity {
            package_id: PackageId::new("pkg"),
            module_path: ModulePath::new("pkg.alpha"),
            normalized_source_path: normalized_path("src/alpha.miz"),
            source_hash: hash(1),
            language_edition: Edition::new("2026"),
        }),
        input_hashes: vec![named("source", 1), named("core-ir", 2)],
        dependency_hashes: vec![
            dependency_hash("interface", "dep", "dep.alpha", "artifact", 3),
            dependency_hash("manifest", "dep", "", "manifest", 4),
        ],
        dependency_slices: vec![
            slice_hash("theorem", "pkg.alpha::T1", "used", 5),
            slice_hash("cluster", "pkg.alpha::C1", "used", 6),
        ],
        config_hash: hash(7),
        schema_versions: vec![
            schema("vc-ir", "output", "1.0"),
            schema("proof-reuse", "metadata", "1.0"),
        ],
        policy_fingerprint: PolicyFingerprint::new(hash(8)),
        validation_inputs: validation_inputs(),
    }
}

fn validation_inputs() -> CacheValidationInputs {
    CacheValidationInputs {
        cache_schema_compatibility: SchemaVersion::default(),
        toolchain_compatibility: vec![
            CompatibilityField {
                family: "mizar".to_owned(),
                field_name: "version".to_owned(),
                value: "dev".to_owned(),
            },
            CompatibilityField {
                family: "rust".to_owned(),
                field_name: "target".to_owned(),
                value: "test".to_owned(),
            },
        ],
        dependency_artifacts: vec![
            DependencyArtifactAvailability {
                package_id: "dep".to_owned(),
                module_path: "dep.alpha".to_owned(),
                artifact_kind: "mizir".to_owned(),
                artifact_path: "build/dep.alpha.mizir.json".to_owned(),
                domain: "artifact".to_owned(),
                digest: hash(9),
            },
            DependencyArtifactAvailability {
                package_id: "dep".to_owned(),
                module_path: "".to_owned(),
                artifact_kind: "manifest".to_owned(),
                artifact_path: "build/artifact-manifest.json".to_owned(),
                domain: "manifest".to_owned(),
                digest: hash(10),
            },
        ],
        footprint_completeness: FootprintCompleteness::Complete,
        uncacheable: false,
        policy_compatibility: vec![
            CompatibilityField {
                family: "proof-policy".to_owned(),
                field_name: "require_kernel_certificates".to_owned(),
                value: "true".to_owned(),
            },
            CompatibilityField {
                family: "reuse-policy".to_owned(),
                field_name: "allow_deterministic_discharge".to_owned(),
                value: "true".to_owned(),
            },
        ],
        canonical_vc_fingerprint: Some(hash(11)),
        local_context_fingerprint: Some(hash(12)),
        dependency_slice_fingerprints: vec![
            slice_hash("vc", "pkg.alpha::T1", "dependency", 13),
            slice_hash("context", "pkg.alpha::T1", "local", 14),
        ],
        obligation_anchor_fingerprint: Some(hash(15)),
        selected_proof_witness_hash: Some(named("selected-witness", 16)),
        deterministic_discharge_hash: Some(named("deterministic-discharge", 17)),
        proof_reuse_schema_versions: vec![
            schema("proof-reuse", "metadata", "1.0"),
            schema("proof-reuse", "status", "1.0"),
        ],
        proof_reuse_validation_hash: Some(named("proof-reuse-validation", 18)),
        proof_reuse_evidence_identities: vec![
            ProofReuseEvidenceIdentity {
                obligation_anchor_fingerprint: hash(15),
                evidence_kind: "kernel_verified".to_owned(),
                witness_or_discharge_domain: "witness".to_owned(),
                witness_or_discharge_digest: hash(16),
            },
            ProofReuseEvidenceIdentity {
                obligation_anchor_fingerprint: hash(15),
                evidence_kind: "discharged_builtin".to_owned(),
                witness_or_discharge_domain: "discharge".to_owned(),
                witness_or_discharge_digest: hash(17),
            },
        ],
        diagnostic_refs: vec![
            DiagnosticRefHash {
                diagnostic_ref_kind: "cache_miss_explanation".to_owned(),
                diagnostic_ref_hash: hash(19),
            },
            DiagnosticRefHash {
                diagnostic_ref_kind: "compatibility_trace".to_owned(),
                diagnostic_ref_hash: hash(20),
            },
        ],
    }
}

fn named(name: &str, seed: u8) -> NamedHash {
    NamedHash {
        name: name.to_owned(),
        domain: format!("domain/{name}"),
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
        domain: format!("domain/{dependency_kind}"),
        digest: hash(seed),
    }
}

fn slice_hash(slice_kind: &str, owner: &str, name: &str, seed: u8) -> DependencySliceHash {
    DependencySliceHash {
        slice_kind: slice_kind.to_owned(),
        owner: owner.to_owned(),
        name: name.to_owned(),
        domain: format!("domain/{slice_kind}"),
        digest: hash(seed),
    }
}

fn schema(family: &str, name: &str, version: &str) -> NamedSchemaVersion {
    NamedSchemaVersion {
        schema_family: family.to_owned(),
        name: name.to_owned(),
        version: SchemaVersion::new(version),
    }
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn normalized_path(value: &str) -> NormalizedPath {
    static NEXT_TEST_PATH: AtomicUsize = AtomicUsize::new(0);

    let suffix = NEXT_TEST_PATH.fetch_add(1, Ordering::Relaxed);
    let package_root = std::env::temp_dir().join(format!(
        "mizar-cache-key-test-{}-{suffix}",
        std::process::id()
    ));
    let source_path = package_root.join(value);
    std::fs::create_dir_all(source_path.parent().expect("test path has parent"))
        .expect("create test source dir");
    std::fs::write(&source_path, b"environ\nbegin\n").expect("write test source");

    let normalized =
        mizar_session::normalize_path(&package_root, &source_path).expect("test normalized path");
    std::fs::remove_dir_all(&package_root).expect("remove test source dir");
    normalized
}
