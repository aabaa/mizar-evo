use super::*;

type RequestMutation = Box<dyn Fn(&mut ProofReuseValidationRequest)>;
type MismatchCase = (&'static str, ProofReuseMissReason, RequestMutation);

fn hash(byte: u8) -> Hash {
    Hash::from_bytes([byte; Hash::BYTE_LEN])
}

fn version(major: u16, minor: u16) -> ArtifactSchemaVersion {
    ArtifactSchemaVersion::new(major, minor)
}

fn compatibility(seed: u8) -> ProofReuseDependencyCompatibilitySnapshot {
    ProofReuseDependencyCompatibilitySnapshot {
        dependency_artifact_fingerprint: hash(seed),
        dependency_schema_version: version(1, 0),
        proof_reuse_schema_version: version(2, 0),
    }
}

fn cacheable_environment() -> ProofReuseValidationEnvironment {
    ProofReuseValidationEnvironment {
        cache_schema_version: SchemaVersion::new(CACHE_KEY_SCHEMA_VERSION),
        proof_reuse_schema_versions: vec![NamedSchemaVersion {
            schema_family: "mizar-cache/proof-reuse".to_owned(),
            name: "validator".to_owned(),
            version: SchemaVersion::new(PROOF_REUSE_SCHEMA_VERSION),
        }],
        toolchain_compatibility: vec![CompatibilityField {
            family: "toolchain".to_owned(),
            field_name: "mizar-cache-proof-reuse".to_owned(),
            value: "current".to_owned(),
        }],
        policy_compatibility: vec![CompatibilityField {
            family: "verifier-policy".to_owned(),
            field_name: "proof-reuse-policy".to_owned(),
            value: "current".to_owned(),
        }],
        dependency_artifacts: vec![DependencyArtifactAvailability {
            package_id: "pkg".to_owned(),
            module_path: "module".to_owned(),
            artifact_kind: "proof-reuse-dependency".to_owned(),
            artifact_path: "artifact/proof-reuse-dependency".to_owned(),
            domain: "mizar-cache/proof-reuse-dependency-artifact/v1".to_owned(),
            digest: hash(1),
        }],
        footprint_completeness: FootprintCompleteness::Complete,
        uncacheable: false,
    }
}

fn snapshot(class: ProofWinnerClass) -> ProofReuseMetadataSnapshot {
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
        selection_reason: "trusted-winner".to_owned(),
        tie_break_key_hash: hash(9),
        dependency_compatibility: Some(compatibility(10)),
        proof_reuse_validation_hash: hash(11),
        cache_reuse_predicate_complete: class.is_trusted(),
    }
}

fn make_request(class: ProofWinnerClass) -> ProofReuseValidationRequest {
    let current = snapshot(class);
    ProofReuseValidationRequest {
        cached: current.clone(),
        current,
        environment: cacheable_environment(),
        diagnostic_refs: vec![
            DiagnosticRefHash {
                diagnostic_ref_kind: "proof".to_owned(),
                diagnostic_ref_hash: hash(42),
            },
            DiagnosticRefHash {
                diagnostic_ref_kind: "proof".to_owned(),
                diagnostic_ref_hash: hash(41),
            },
        ],
    }
}

fn hit(request: &ProofReuseValidationRequest) -> ProofReuseValidationHit {
    match ProofReuseValidator::validate(request) {
        ProofReuseValidationOutcome::Hit(hit) => hit,
        ProofReuseValidationOutcome::Miss(miss) => panic!("expected hit, got {miss:?}"),
    }
}

fn miss_reason(request: &ProofReuseValidationRequest) -> ProofReuseMissReason {
    match ProofReuseValidator::validate(request) {
        ProofReuseValidationOutcome::Hit(hit) => panic!("expected miss, got {hit:?}"),
        ProofReuseValidationOutcome::Miss(miss) => miss.reason,
    }
}

#[test]
fn matching_kernel_verified_metadata_validates() {
    let request = make_request(ProofWinnerClass::KernelVerified);
    let hit = hit(&request);

    assert_eq!(hit.selected_class, ProofWinnerClass::KernelVerified);
    assert_eq!(hit.witness_or_discharge_hash, hash(20));
    assert_eq!(hit.proof_reuse_validation_hash, hash(11));
    assert_eq!(hit.diagnostic_refs[0].diagnostic_ref_hash, hash(41));
    assert_eq!(hit.diagnostic_refs[1].diagnostic_ref_hash, hash(42));
}

#[test]
fn matching_discharged_builtin_metadata_validates() {
    let request = make_request(ProofWinnerClass::DischargedBuiltin);
    let hit = hit(&request);

    assert_eq!(hit.selected_class, ProofWinnerClass::DischargedBuiltin);
    assert_eq!(hit.witness_or_discharge_hash, hash(21));
    assert_eq!(hit.proof_reuse_validation_hash, hash(11));
}

#[test]
fn each_metadata_mismatch_blocks_reuse() {
    let cases: Vec<MismatchCase> = vec![
        (
            "obligation anchor",
            ProofReuseMissReason::ObligationAnchorMismatch,
            Box::new(|request| request.cached.obligation_anchor = "anchor-2".to_owned()),
        ),
        (
            "obligation fingerprint",
            ProofReuseMissReason::ObligationFingerprintMismatch,
            Box::new(|request| request.cached.obligation_fingerprint = hash(31)),
        ),
        (
            "canonical vc fingerprint",
            ProofReuseMissReason::CanonicalVcFingerprintMismatch,
            Box::new(|request| request.cached.canonical_vc_fingerprint = hash(32)),
        ),
        (
            "local context fingerprint",
            ProofReuseMissReason::LocalContextFingerprintMismatch,
            Box::new(|request| request.cached.local_context_fingerprint = hash(33)),
        ),
        (
            "dependency slice fingerprint",
            ProofReuseMissReason::DependencySliceFingerprintMismatch,
            Box::new(|request| request.cached.dependency_slice_fingerprint = hash(34)),
        ),
        (
            "policy fingerprint",
            ProofReuseMissReason::PolicyMismatch,
            Box::new(|request| request.cached.policy_fingerprint = hash(35)),
        ),
        (
            "selected candidate",
            ProofReuseMissReason::SelectedCandidateMismatch,
            Box::new(|request| request.cached.selected_candidate_id = Some("other".to_owned())),
        ),
        (
            "selected evidence hash",
            ProofReuseMissReason::SelectedEvidenceHashMismatch,
            Box::new(|request| request.cached.selected_evidence_hash = Some(hash(36))),
        ),
        (
            "selected candidate provenance",
            ProofReuseMissReason::SelectedCandidateProvenanceMismatch,
            Box::new(|request| {
                request.cached.selected_candidate_provenance_hash = Some(hash(37));
            }),
        ),
        (
            "selection reason",
            ProofReuseMissReason::SelectionReasonMismatch,
            Box::new(|request| request.cached.selection_reason = "other".to_owned()),
        ),
        (
            "tie-break key",
            ProofReuseMissReason::TieBreakKeyMismatch,
            Box::new(|request| request.cached.tie_break_key_hash = hash(38)),
        ),
        (
            "trusted used-axioms reference hash",
            ProofReuseMissReason::TrustedAxiomSetReferenceMismatch,
            Box::new(|request| request.cached.trusted_used_axioms_hash = Some(hash(39))),
        ),
        (
            "selected witness hash",
            ProofReuseMissReason::SelectedWitnessHashMismatch,
            Box::new(|request| request.cached.selected_proof_witness_hash = Some(hash(40))),
        ),
        (
            "proof-reuse validation hash",
            ProofReuseMissReason::ProofReuseValidationHashMismatch,
            Box::new(|request| request.cached.proof_reuse_validation_hash = hash(41)),
        ),
        (
            "dependency artifact",
            ProofReuseMissReason::DependencyArtifactMismatch,
            Box::new(|request| {
                request
                    .cached
                    .dependency_compatibility
                    .as_mut()
                    .expect("compatibility")
                    .dependency_artifact_fingerprint = hash(42);
            }),
        ),
        (
            "dependency schema",
            ProofReuseMissReason::SchemaVersionMismatch,
            Box::new(|request| {
                request
                    .cached
                    .dependency_compatibility
                    .as_mut()
                    .expect("compatibility")
                    .dependency_schema_version = version(9, 0);
            }),
        ),
        (
            "proof-reuse schema",
            ProofReuseMissReason::SchemaVersionMismatch,
            Box::new(|request| {
                request
                    .cached
                    .dependency_compatibility
                    .as_mut()
                    .expect("compatibility")
                    .proof_reuse_schema_version = version(9, 0);
            }),
        ),
        (
            "selected class",
            ProofReuseMissReason::SelectedClassMismatch,
            Box::new(|request| request.cached.selected_class = ProofWinnerClass::PolicyOpen),
        ),
    ];

    for (label, expected, mutate) in cases {
        let mut request = make_request(ProofWinnerClass::KernelVerified);
        mutate(&mut request);
        assert_eq!(miss_reason(&request), expected, "{label}");
    }
}

#[test]
fn discharged_builtin_hash_mismatches_and_missing_hashes_miss() {
    let mut request = make_request(ProofWinnerClass::DischargedBuiltin);
    request.cached.deterministic_discharge_hash = Some(hash(44));
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::DeterministicDischargeHashMismatch
    );

    let mut request = make_request(ProofWinnerClass::DischargedBuiltin);
    request.cached.deterministic_discharge_hash = None;
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::DeterministicDischargeHashMissing
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.cached.selected_proof_witness_hash = None;
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::SelectedWitnessHashMissing
    );

    let mut request = make_request(ProofWinnerClass::DischargedBuiltin);
    request.cached.selected_proof_witness_hash = Some(hash(45));
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::UnexpectedSelectedWitnessHash
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.cached.deterministic_discharge_hash = Some(hash(46));
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::UnexpectedDeterministicDischargeHash
    );
}

#[test]
fn missing_required_metadata_fields_miss() {
    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.cached.selected_candidate_id = None;
    request.current.selected_candidate_id = None;
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::SelectedCandidateMissing
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.cached.selected_evidence_hash = None;
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::SelectedEvidenceHashMismatch
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.cached.trusted_used_axioms_hash = None;
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::TrustedAxiomSetReferenceMismatch
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.cached.dependency_compatibility = None;
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::IncompleteUpstreamPredicate
    );
}

#[test]
fn environment_fail_closed_markers_miss() {
    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment = ProofReuseValidationEnvironment::default();
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::UnsupportedSchema
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.footprint_completeness = FootprintCompleteness::IncompleteUncacheable;
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::IncompleteFootprint
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.footprint_completeness = FootprintCompleteness::Unsupported;
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::UnsupportedFootprint
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.cache_schema_version = SchemaVersion::new("unknown");
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::UnsupportedSchema
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.proof_reuse_schema_versions[0].version = SchemaVersion::new("unknown");
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::UnsupportedSchema
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.toolchain_compatibility[0].value = "unknown".to_owned();
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::UnknownToolchain
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.policy_compatibility[0].value = "incompatible".to_owned();
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::PolicyCompatibilityMismatch
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.dependency_artifacts.clear();
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::DependencyArtifactMismatch
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.dependency_artifacts[0].domain = "unknown".to_owned();
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::DependencyArtifactMismatch
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.dependency_artifacts[0].package_id = "missing".to_owned();
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::DependencyArtifactMismatch
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.dependency_artifacts[0]
        .module_path
        .clear();
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::DependencyArtifactMismatch
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.dependency_artifacts[0].artifact_kind = "unsupported".to_owned();
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::DependencyArtifactMismatch
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.environment.uncacheable = true;
    assert_eq!(miss_reason(&request), ProofReuseMissReason::Uncacheable);
}

#[test]
fn non_trusted_and_synthesized_metadata_never_become_hits() {
    for class in [
        ProofWinnerClass::PolicyPermittedExternal,
        ProofWinnerClass::PolicyAssumed,
        ProofWinnerClass::PolicyOpen,
        ProofWinnerClass::Rejected,
        ProofWinnerClass::NoSelectableEvidence,
    ] {
        let request = make_request(class);
        assert_eq!(
            miss_reason(&request),
            ProofReuseMissReason::NonReusableClass
        );

        let mut request = make_request(class);
        request.cached.trusted_used_axioms_hash = Some(hash(90));
        assert_eq!(
            miss_reason(&request),
            ProofReuseMissReason::SynthesizedTrustedAxiomSetReference,
            "{class:?}"
        );
    }
}

#[test]
fn upstream_completeness_predicate_is_honored() {
    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.current.cache_reuse_predicate_complete = false;
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::IncompleteUpstreamPredicate
    );

    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.cached.dependency_compatibility = None;
    request.cached.cache_reuse_predicate_complete = false;
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::IncompleteUpstreamPredicate
    );
}

#[test]
fn diagnostic_order_is_not_validation_input() {
    let mut first = make_request(ProofWinnerClass::KernelVerified);
    first.diagnostic_refs.push(DiagnosticRefHash {
        diagnostic_ref_kind: "cache".to_owned(),
        diagnostic_ref_hash: hash(40),
    });
    let mut second = first.clone();
    second.diagnostic_refs.reverse();

    let first_hit = hit(&first);
    let second_hit = hit(&second);

    assert_eq!(first_hit.selected_class, second_hit.selected_class);
    assert_eq!(
        first_hit.witness_or_discharge_hash,
        second_hit.witness_or_discharge_hash
    );
    assert_eq!(first_hit.diagnostic_refs, second_hit.diagnostic_refs);
}
