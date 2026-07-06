use super::*;
use mizar_kernel::{
    certificate_parser::{
        ClauseTautologyPolicy, Fingerprint, KernelProfileRecord, RequiredProofStatus,
    },
    checker::{
        AcceptedProofStatus, FormulaEvidenceContext, FormulaImportedFactEvidence,
        ImportedFactContextLimits, KernelCheckPolicy, KernelCheckStatus, KernelEvidenceCheckInput,
        KernelEvidenceCheckKind, KernelEvidenceCheckLimits, check_kernel_evidence,
    },
    clause::{Atom, SymbolId, SymbolKey, SymbolKind},
    formula_evidence::{
        Formula, FormulaEvidenceParseContext, FormulaSourceClass, GoalPolarity,
        IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID, ImportedStatementProjection,
        ParsedKernelEvidence, SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
        canonical_imported_statement_projection_payload, parse_formula_evidence,
    },
    rejection::TargetVcFingerprint,
};
use mizar_proof::{
    policy::{KernelEvidenceOrigin, KernelPolicyInput, VerifierPolicy},
    selection::{
        CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, VcProofSelection,
        merge_artifact_proof_selections, select_winner,
    },
    status::{
        ObligationAnchor, ProofObligationIdentity, ProofReuseDependencyCompatibility,
        ProofStatusProjectionInput, project_status,
    },
};
use mizar_vc::vc_ir::VcId;

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
        accepted_goal_polarity: class
            .is_trusted()
            .then(|| SUPPORTED_ACCEPTED_GOAL_POLARITY.to_owned()),
        trusted_used_axioms_hash: class.is_trusted().then(|| hash(7)),
        selected_candidate_provenance_hash: Some(hash(8)),
        selection_reason: "trusted-winner".to_owned(),
        tie_break_key_hash: hash(9),
        dependency_compatibility: Some(compatibility(10)),
        proof_reuse_validation_hash: hash(11),
        cache_reuse_predicate_complete: class.is_trusted(),
    }
}

#[test]
fn status_reuse_metadata_conversion_carries_accepted_goal_polarity() {
    let projection = projected_kernel_status();
    let metadata = projection.reuse_metadata();
    assert_eq!(
        metadata
            .accepted_goal_polarity()
            .map(|polarity| polarity.as_str()),
        Some(SUPPORTED_ACCEPTED_GOAL_POLARITY)
    );

    let snapshot = ProofReuseMetadataSnapshot::from(metadata);
    assert_eq!(snapshot.selected_class, ProofWinnerClass::KernelVerified);
    assert_eq!(
        snapshot.accepted_goal_polarity.as_deref(),
        Some(SUPPORTED_ACCEPTED_GOAL_POLARITY)
    );
    assert_eq!(snapshot.selected_proof_witness_hash, Some(hash(50)));
    assert!(snapshot.cache_reuse_predicate_complete);
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
            "accepted goal polarity",
            ProofReuseMissReason::AcceptedGoalPolarityMissing,
            Box::new(|request| request.cached.accepted_goal_polarity = None),
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
fn accepted_goal_polarity_is_required_for_trusted_reuse() {
    for class in [
        ProofWinnerClass::KernelVerified,
        ProofWinnerClass::DischargedBuiltin,
    ] {
        let mut request = make_request(class);
        request.cached.accepted_goal_polarity = None;
        assert_eq!(
            miss_reason(&request),
            ProofReuseMissReason::AcceptedGoalPolarityMissing,
            "{class:?} cached pre-audit polarity"
        );

        let mut request = make_request(class);
        request.current.accepted_goal_polarity = None;
        assert_eq!(
            miss_reason(&request),
            ProofReuseMissReason::AcceptedGoalPolarityMissing,
            "{class:?} current polarity"
        );
    }
}

#[test]
fn unsupported_accepted_goal_polarity_misses() {
    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.cached.accepted_goal_polarity = Some("future-polarity".to_owned());
    request.current.accepted_goal_polarity = Some("future-polarity".to_owned());
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::UnsupportedAcceptedGoalPolarity
    );

    let mut request = make_request(ProofWinnerClass::DischargedBuiltin);
    request.current.accepted_goal_polarity = Some("future-polarity".to_owned());
    request.cached.accepted_goal_polarity = Some("future-polarity".to_owned());
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::UnsupportedAcceptedGoalPolarity
    );
}

#[test]
fn accepted_goal_polarity_mismatch_misses() {
    let mut request = make_request(ProofWinnerClass::KernelVerified);
    request.cached.accepted_goal_polarity = Some("future-polarity".to_owned());
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::AcceptedGoalPolarityMismatch
    );

    let mut request = make_request(ProofWinnerClass::DischargedBuiltin);
    request.current.accepted_goal_polarity = Some("future-polarity".to_owned());
    assert_eq!(
        miss_reason(&request),
        ProofReuseMissReason::AcceptedGoalPolarityMismatch
    );
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

fn projected_kernel_status() -> mizar_proof::status::ProofStatusProjection {
    let kernel_result = accepted_kernel_result();
    let policy_input = KernelPolicyInput::from_kernel_result(
        &kernel_result,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
    );
    let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
        CandidateSourceId::new("kernel").expect("stable candidate id"),
        &policy_input,
    )
    .expect("accepted proof-obligation kernel input is trusted")
    .with_selected_proof_witness_hash(hash(50))
    .with_provenance_hash(hash(34));
    let proof_set =
        ProofEvidenceSet::new(b"obligation".to_vec(), hash(100), VerifierPolicy::release())
            .with_candidates([candidate]);
    let selection = merge_artifact_proof_selections(
        [VcProofSelection::new(
            VcId::new(0),
            select_winner(&proof_set),
        )],
        [],
    )
    .expect("artifact selection")
    .into_iter()
    .next()
    .expect("one artifact selection");
    let identity = ProofObligationIdentity::new(
        "obligation-0",
        ObligationAnchor::new("anchor-0").expect("valid anchor"),
        hash(20),
        hash(21),
        hash(22),
        hash(23),
    )
    .expect("valid identity");
    let dependency_compatibility =
        ProofReuseDependencyCompatibility::new(hash(24), version(1, 0), version(2, 0));

    project_status(
        ProofStatusProjectionInput::new(selection, VerifierPolicy::release(), identity)
            .with_dependency_compatibility(dependency_compatibility),
    )
    .expect("status projection")
}

fn accepted_kernel_result() -> mizar_kernel::checker::KernelCheckResult {
    let target = formula_target(7);
    let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
    let premise = formula_atom(1);
    let parsed = parsed_formula_evidence(
        &target,
        vec![imported_formula_item(
            1,
            10,
            &premise,
            RequiredProofStatus::KernelVerified,
        )],
        goal_item(20, &premise),
    );
    let context = formula_evidence_context(formula_imported_fact(
        5,
        &premise,
        AcceptedProofStatus::KernelVerified,
    ));

    let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, Some(&context)));
    assert_eq!(result.status(), KernelCheckStatus::Accepted);
    assert!(!result.policy_taint());
    result
}

fn formula_target(tag: u8) -> Fingerprint {
    Fingerprint::new(9, vec![tag])
}

fn formula_profile() -> KernelProfileRecord {
    KernelProfileRecord::v1(7, ClauseTautologyPolicy::Reject)
}

fn formula_atom(symbol_id: u32) -> Formula {
    Formula::Atom(Atom::with_arity(
        SymbolKey {
            kind: SymbolKind::Predicate,
            id: SymbolId(symbol_id),
        },
        0,
        Vec::new(),
    ))
}

fn formula_fingerprint(formula: &Formula) -> Fingerprint {
    Fingerprint::new(
        SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
        formula
            .canonical_hash_input()
            .expect("test formula has canonical bytes"),
    )
}

struct FormulaFixture {
    bytes: Vec<u8>,
    provenance_id: u32,
    fingerprint: Fingerprint,
}

fn imported_formula_item(
    formula_id: u32,
    provenance_id: u32,
    formula: &Formula,
    required_status: RequiredProofStatus,
) -> FormulaFixture {
    let fingerprint = formula_fingerprint(formula);
    let statement_fingerprint = imported_statement_fingerprint();
    let statement_projection = imported_statement_projection(&statement_fingerprint, &fingerprint);
    let mut bytes = Vec::new();
    put_u32(formula_id, &mut bytes);
    bytes.push(FormulaSourceClass::AcceptedImportedAxiom.tag());
    put_fingerprint(&fingerprint, &mut bytes);
    put_u32(provenance_id, &mut bytes);
    put_bytes(b"pkg", &mut bytes);
    put_bytes(b"module", &mut bytes);
    put_bytes(b"ITEM", &mut bytes);
    put_fingerprint(&statement_fingerprint, &mut bytes);
    bytes.push(required_status_tag(required_status));
    put_fingerprint(&statement_projection.statement_fingerprint, &mut bytes);
    put_fingerprint(&statement_projection.formula_fingerprint, &mut bytes);
    put_bytes(&statement_projection.payload, &mut bytes);
    put_formula(formula, &mut bytes);
    FormulaFixture {
        bytes,
        provenance_id,
        fingerprint,
    }
}

fn goal_item(provenance_id: u32, formula: &Formula) -> FormulaFixture {
    let fingerprint = formula_fingerprint(formula);
    let mut bytes = Vec::new();
    bytes.push(GoalPolarity::AssertFalseForRefutation.tag());
    put_fingerprint(&fingerprint, &mut bytes);
    put_u32(provenance_id, &mut bytes);
    put_formula(formula, &mut bytes);
    FormulaFixture {
        bytes,
        provenance_id,
        fingerprint,
    }
}

fn parsed_formula_evidence(
    target: &Fingerprint,
    formulas: Vec<FormulaFixture>,
    goal: FormulaFixture,
) -> ParsedKernelEvidence {
    let bytes = formula_evidence_bytes(target, formulas, goal);
    parse_formula_evidence(
        &bytes,
        &FormulaEvidenceParseContext::v1(target.clone(), formula_profile()),
    )
    .expect("formula evidence parses")
}

fn evidence_input<'a>(
    target: &'a TargetVcFingerprint,
    evidence: &'a ParsedKernelEvidence,
    formula_context: Option<&'a FormulaEvidenceContext>,
) -> KernelEvidenceCheckInput<'a> {
    KernelEvidenceCheckInput {
        target_vc_fingerprint: target,
        evidence,
        formula_context,
        check_kind: KernelEvidenceCheckKind::ProofObligation,
        policy: KernelCheckPolicy::default(),
        limits: KernelEvidenceCheckLimits::default(),
    }
}

fn formula_evidence_context(imported: FormulaImportedFactEvidence) -> FormulaEvidenceContext {
    FormulaEvidenceContext::new(
        Some(vec![1]),
        vec![imported],
        Vec::new(),
        ImportedFactContextLimits::default(),
    )
    .expect("formula evidence context")
}

fn formula_imported_fact(
    imported_fact_id: u32,
    formula: &Formula,
    accepted_proof_status: AcceptedProofStatus,
) -> FormulaImportedFactEvidence {
    let formula_fingerprint = formula_fingerprint(formula);
    let statement_fingerprint = imported_statement_fingerprint();
    let statement_projection =
        imported_statement_projection(&statement_fingerprint, &formula_fingerprint);
    FormulaImportedFactEvidence {
        imported_fact_id,
        package_id: b"pkg".to_vec(),
        module_path: b"module".to_vec(),
        exported_item_id: b"ITEM".to_vec(),
        statement_fingerprint,
        accepted_proof_status,
        statement_projection,
    }
}

fn imported_statement_projection(
    statement_fingerprint: &Fingerprint,
    formula_fingerprint: &Fingerprint,
) -> ImportedStatementProjection {
    ImportedStatementProjection {
        statement_fingerprint: statement_fingerprint.clone(),
        formula_fingerprint: formula_fingerprint.clone(),
        payload: canonical_imported_statement_projection_payload(
            statement_fingerprint,
            formula_fingerprint,
        )
        .expect("canonical imported statement projection payload"),
    }
}

fn imported_statement_fingerprint() -> Fingerprint {
    Fingerprint::new(
        IMPORTED_STATEMENT_FINGERPRINT_ALGORITHM_ID,
        b"imported-statement".to_vec(),
    )
}

fn formula_evidence_bytes(
    target: &Fingerprint,
    formulas: Vec<FormulaFixture>,
    goal: FormulaFixture,
) -> Vec<u8> {
    let mut provenance = Vec::new();
    for formula in &formulas {
        provenance.push(provenance_item(
            target,
            formula.provenance_id,
            &formula.fingerprint,
        ));
    }
    provenance.push(provenance_item(
        target,
        goal.provenance_id,
        &goal.fingerprint,
    ));
    formula_envelope(
        target,
        vec![
            (
                FormulaEvidenceSectionTag::SymbolManifest,
                formula_symbol_items(),
            ),
            (FormulaEvidenceSectionTag::VariableManifest, Vec::new()),
            (
                FormulaEvidenceSectionTag::Formulas,
                formulas.into_iter().map(|formula| formula.bytes).collect(),
            ),
            (FormulaEvidenceSectionTag::Substitutions, Vec::new()),
            (FormulaEvidenceSectionTag::Provenance, provenance),
            (FormulaEvidenceSectionTag::FinalGoal, vec![goal.bytes]),
        ],
    )
}

#[derive(Clone, Copy)]
enum FormulaEvidenceSectionTag {
    SymbolManifest,
    VariableManifest,
    Formulas,
    Substitutions,
    Provenance,
    FinalGoal,
}

impl FormulaEvidenceSectionTag {
    const fn byte(self) -> u8 {
        match self {
            Self::SymbolManifest => 1,
            Self::VariableManifest => 2,
            Self::Formulas => 3,
            Self::Substitutions => 4,
            Self::Provenance => 5,
            Self::FinalGoal => 6,
        }
    }
}

fn formula_envelope(
    target: &Fingerprint,
    sections: Vec<(FormulaEvidenceSectionTag, Vec<Vec<u8>>)>,
) -> Vec<u8> {
    let mut payloads = Vec::new();
    let mut directory = Vec::new();
    let mut offset = 0_u32;
    for (section, items) in &sections {
        let mut section_payload = Vec::new();
        for item in items {
            section_payload.push(section.byte());
            section_payload.push(1);
            put_len(item.len(), &mut section_payload);
            section_payload.extend_from_slice(item);
        }
        let length = u32::try_from(section_payload.len()).expect("section length fits");
        directory.push((*section, items.len() as u32, offset, length));
        offset = offset.checked_add(length).expect("payload offset fits");
        payloads.push(section_payload);
    }

    let mut bytes = Vec::from(b"MIZAR_KERNEL_EVIDENCE\0".as_slice());
    put_u16(1, &mut bytes);
    put_u16(1, &mut bytes);
    put_formula_profile(&mut bytes);
    put_fingerprint(target, &mut bytes);
    put_u32(sections.len() as u32, &mut bytes);
    for (section, count, payload_offset, payload_length) in directory {
        bytes.push(section.byte());
        put_u32(count, &mut bytes);
        put_u32(payload_offset, &mut bytes);
        put_u32(payload_length, &mut bytes);
    }
    for payload in payloads {
        bytes.extend(payload);
    }
    bytes
}

fn formula_symbol_items() -> Vec<Vec<u8>> {
    [1_u32, 2_u32]
        .into_iter()
        .map(|id| {
            let mut item = Vec::new();
            item.push(symbol_kind_tag(SymbolKind::Predicate));
            put_u32(id, &mut item);
            item
        })
        .collect()
}

fn provenance_item(target: &Fingerprint, provenance_id: u32, fingerprint: &Fingerprint) -> Vec<u8> {
    let mut item = Vec::new();
    put_u32(provenance_id, &mut item);
    put_fingerprint(target, &mut item);
    put_fingerprint(fingerprint, &mut item);
    put_bytes(b"producer-payload", &mut item);
    item
}

fn put_formula(formula: &Formula, bytes: &mut Vec<u8>) {
    match formula {
        Formula::Atom(atom) => {
            bytes.push(1);
            put_atom(atom, bytes);
        }
        Formula::Not(child) => {
            bytes.push(2);
            put_formula(child, bytes);
        }
        Formula::And(children) => {
            bytes.push(3);
            put_u32(children.len() as u32, bytes);
            for child in children {
                put_formula(child, bytes);
            }
        }
        Formula::Or(children) => {
            bytes.push(4);
            put_u32(children.len() as u32, bytes);
            for child in children {
                put_formula(child, bytes);
            }
        }
        _ => panic!("unknown formula variant in proof-reuse test fixture"),
    }
}

fn put_atom(atom: &Atom, bytes: &mut Vec<u8>) {
    bytes.push(symbol_kind_tag(atom.symbol.kind));
    put_u32(atom.symbol.id.0, bytes);
    put_u32(atom.arity, bytes);
    put_u32(0, bytes);
}

fn put_formula_profile(bytes: &mut Vec<u8>) {
    let profile = formula_profile();
    put_u16(profile.profile_id, bytes);
    put_u16(profile.clause_schema_version, bytes);
    put_u16(profile.clause_encoding_version, bytes);
    bytes.push(profile.clause_tautology_policy.tag());
    bytes.push(profile.certificate_hash_input_algorithm.tag());
}

fn put_fingerprint(fingerprint: &Fingerprint, bytes: &mut Vec<u8>) {
    bytes.push(fingerprint.algorithm_id);
    put_bytes(&fingerprint.digest, bytes);
}

fn put_bytes(payload: &[u8], bytes: &mut Vec<u8>) {
    put_len(payload.len(), bytes);
    bytes.extend_from_slice(payload);
}

fn put_len(len: usize, bytes: &mut Vec<u8>) {
    put_u32(u32::try_from(len).expect("length fits"), bytes);
}

fn put_u16(value: u16, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn put_u32(value: u32, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn symbol_kind_tag(kind: SymbolKind) -> u8 {
    match kind {
        SymbolKind::Predicate => 1,
        SymbolKind::FunctorPredicate => 2,
        SymbolKind::Equality => 3,
        SymbolKind::BuiltinRelation => 4,
        _ => panic!("unknown symbol kind in proof-reuse test fixture"),
    }
}

fn required_status_tag(status: RequiredProofStatus) -> u8 {
    match status {
        RequiredProofStatus::KernelVerified => 1,
        RequiredProofStatus::DischargedBuiltin => 2,
        RequiredProofStatus::ExternallyAttestedPolicyPermitted => 3,
        _ => panic!("unknown required proof status in proof-reuse test fixture"),
    }
}
