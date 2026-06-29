use super::*;
use mizar_artifact::{
    module_summary::ModuleSummaryIdentity,
    proof_witness::{EvidenceKind, ProofStatus as ArtifactProofStatus, proof_witness_ref_json},
    store::SchemaVersion,
};
use mizar_kernel::checker::KernelCheckStatus;
use mizar_vc::vc_ir::VcId;

use crate::{
    policy::{KernelPolicyInput, VerifierPolicy},
    selection::{
        ArtifactProofSelection, ProofEvidenceCandidate, ProofEvidenceSet, VcProofSelection,
        merge_artifact_proof_selections, select_winner,
    },
    status::{ProofObligationIdentity, ProofStatusProjectionInput, project_status},
};

#[test]
fn stages_kernel_verified_candidate_before_commit() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref");

    assert_eq!(staged.selected_class(), ProofWinnerClass::KernelVerified);
    assert_eq!(reference.proof_status, ArtifactProofStatus::KernelVerified);
    assert_eq!(
        reference.evidence_kind,
        EvidenceKind::FormulaSubstitutionKernelEvidence
    );
    assert_eq!(reference.witness_path, "proof-witnesses/obl-1.json");
    assert_eq!(
        &reference.witness_artifact_hash,
        staged.staged_payload_hash()
    );
    assert_eq!(
        reference.kernel_acceptance,
        kernel_acceptance_metadata(9, hash(2))
    );
    proof_witness_ref_json(reference).expect("artifact ref candidate validates");
}

#[test]
fn publish_requires_committed_reachability_proof() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref")
        .clone();
    let proof = publication_proof(
        &staged,
        vec![reference.clone()],
        vec![manifest_entry(&reference)],
    )
    .expect("coverage token is internally consistent");

    let published = publish_ref(&staged, &proof).expect("published after committed proof");
    assert_eq!(published.reference(), &reference);
    assert_eq!(
        published.committed_main_artifact_hash(),
        proof.committed_main_artifact_hash()
    );
    assert_eq!(
        published.provenance().build_snapshot_fingerprint(),
        staged.provenance().build_snapshot_fingerprint()
    );
}

#[test]
fn witness_stage_and_publish_refs_ignore_candidate_arrival_order() {
    let first = witness_roundtrip_from_candidate_order(["candidate-1", "candidate-later"]);
    let second = witness_roundtrip_from_candidate_order(["candidate-later", "candidate-1"]);

    assert_eq!(first, second);
    assert_eq!(
        first
            .0
            .artifact_reference_candidate()
            .expect("staged witness ref exists"),
        second
            .0
            .artifact_reference_candidate()
            .expect("staged witness ref exists")
    );
}

#[test]
fn publish_before_manifest_reference_fails() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref")
        .clone();
    let proof = publication_proof(&staged, vec![reference], Vec::new());

    assert!(matches!(
        proof,
        Err(ProofWitnessStoreError::ManifestCoverageMismatch { .. })
    ));
}

#[test]
fn publish_rejects_missing_committed_artifact_reference() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let committed = different_reference(&staged, 7);
    let proof = publication_proof(
        &staged,
        vec![committed.clone()],
        vec![manifest_entry(&committed)],
    )
    .expect("coverage token is internally consistent");

    assert!(matches!(
        publish_ref(&staged, &proof),
        Err(ProofWitnessStoreError::MissingCommittedWitnessReference { .. })
    ));
}

#[test]
fn publish_rejects_stale_build_snapshot() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref")
        .clone();
    let proof = CommittedWitnessPublicationProof::for_test(
        module_entry(vec![manifest_entry(&reference)], 1),
        hash(99),
        vec![reference.clone()],
    )
    .expect("coverage token is internally consistent");

    assert_eq!(
        publish_ref(&staged, &proof),
        Err(ProofWitnessStoreError::StaleBuildSnapshot)
    );
}

#[test]
fn stage_hash_is_stable_and_changes_with_trusted_inputs() {
    let first = stage(kernel_draft()).expect("kernel draft stages");
    let second = stage(kernel_draft()).expect("same draft stages");
    assert_eq!(first.staged_payload_hash(), second.staged_payload_hash());

    let changed_witness_path =
        stage(kernel_draft().with_witness_path_for_test("proof-witnesses/renamed.json"))
            .expect("changed witness path stages");
    assert_eq!(
        first.staged_payload_hash(),
        changed_witness_path.staged_payload_hash()
    );

    let changed_build_snapshot = stage(kernel_draft().with_build_snapshot_for_test(hash(47)))
        .expect("changed build snapshot stages");
    assert_eq!(
        first.staged_payload_hash(),
        changed_build_snapshot.staged_payload_hash()
    );

    let changed_backend_ref = stage(kernel_draft().with_advisory_backend_ref_for_test(hash(48)))
        .expect("changed advisory backend ref stages");
    assert_eq!(
        first.staged_payload_hash(),
        changed_backend_ref.staged_payload_hash()
    );

    let changed_payload =
        stage(kernel_draft().with_payload_bytes_for_test(b"different payload".to_vec()))
            .expect("changed payload stages");
    assert_ne!(
        first.staged_payload_hash(),
        changed_payload.staged_payload_hash()
    );

    let changed_evidence = stage(kernel_draft().with_selected_evidence_hash_for_test(hash(44)))
        .expect("changed evidence stages");
    assert_ne!(
        first.staged_payload_hash(),
        changed_evidence.staged_payload_hash()
    );

    let changed_policy = stage(kernel_draft().with_verifier_policy_for_test(hash(45)))
        .expect("changed policy stages");
    assert_ne!(
        first.staged_payload_hash(),
        changed_policy.staged_payload_hash()
    );

    let changed_obligation = stage(kernel_draft().with_obligation_fingerprint_for_test(46))
        .expect("changed obligation stages");
    assert_ne!(
        first.staged_payload_hash(),
        changed_obligation.staged_payload_hash()
    );

    let changed_schema = stage(kernel_draft().with_payload_schema_version_for_test(2))
        .expect("changed schema stages");
    assert_ne!(
        first.staged_payload_hash(),
        changed_schema.staged_payload_hash()
    );
}

#[test]
fn stage_rejects_path_escape() {
    let draft = kernel_draft().with_witness_path_for_test("proof-witnesses/../evil.json");
    assert!(matches!(
        stage(draft),
        Err(ProofWitnessStoreError::InvalidWitnessPath { .. })
    ));
}

#[test]
fn draft_rejects_empty_required_canonical_payload_bytes() {
    let obligation_fingerprint = artifact_ref(ArtifactHashClass::Interface, "obligation", 1);
    let anchor = anchor();
    let projection = status_projection(
        "obl-1",
        anchor.clone(),
        &obligation_fingerprint,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(2),
        None,
    );
    let payload_schema = ProofWitnessPayloadSchema::with_canonical_bytes_required(
        "mizar-proof/test-witness-payload",
        SchemaVersion::new(1, 0),
        true,
    )
    .expect("schema");

    assert!(matches!(
        ProofWitnessDraft::new(
            "obl-1",
            anchor,
            obligation_fingerprint,
            &projection,
            trusted_metadata(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2), 9),
            payload_schema,
            Vec::<u8>::new(),
            "proof-witnesses/empty.json",
            provenance(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2)),
        ),
        Err(ProofWitnessStoreError::NonCanonicalPayloadBytes { .. })
    ));
}

#[test]
fn draft_requires_matching_status_projection() {
    let obligation_fingerprint = artifact_ref(ArtifactHashClass::Interface, "obligation", 1);
    let anchor = anchor();
    let projection = status_projection(
        "obl-1",
        anchor.clone(),
        &obligation_fingerprint,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(99),
        None,
    );

    assert!(matches!(
        ProofWitnessDraft::new(
            "obl-1",
            anchor,
            obligation_fingerprint,
            &projection,
            trusted_metadata(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2), 9),
            payload_schema(),
            b"canonical witness payload".to_vec(),
            "proof-witnesses/obl-1.json",
            provenance(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2)),
        ),
        Err(ProofWitnessStoreError::StatusProjectionMismatch {
            field: "selected_evidence_hash",
            ..
        })
    ));
}

#[test]
fn draft_rejects_inconsistent_provenance_origin() {
    let obligation_fingerprint = artifact_ref(ArtifactHashClass::Interface, "obligation", 1);
    let anchor = anchor();
    let projection = status_projection(
        "obl-1",
        anchor.clone(),
        &obligation_fingerprint,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(2),
        None,
    );

    assert!(matches!(
        ProofWitnessDraft::new(
            "obl-1",
            anchor,
            obligation_fingerprint,
            &projection,
            trusted_metadata(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2), 9),
            payload_schema(),
            b"canonical witness payload".to_vec(),
            "proof-witnesses/obl-1.json",
            provenance(KernelEvidenceOrigin::BuiltinDischarge, hash(2)),
        ),
        Err(ProofWitnessStoreError::StatusProjectionMismatch {
            field: "kernel_evidence_origin",
            ..
        })
    ));
}

#[test]
fn draft_rejects_unprojected_trusted_used_axioms_provenance() {
    let trusted_used_axioms = TrustedUsedAxiomsRef::for_test(hash(2), hash(90), 2);
    let obligation_fingerprint = artifact_ref(ArtifactHashClass::Interface, "obligation", 1);
    let anchor = anchor();
    let projection = status_projection(
        "obl-1",
        anchor.clone(),
        &obligation_fingerprint,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(2),
        None,
    );

    assert_eq!(
        ProofWitnessDraft::new(
            "obl-1",
            anchor,
            obligation_fingerprint,
            &projection,
            trusted_metadata(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2), 9),
            payload_schema(),
            b"canonical witness payload".to_vec(),
            "proof-witnesses/obl-1.json",
            provenance(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2))
                .with_trusted_used_axioms_ref(&trusted_used_axioms),
        ),
        Err(
            ProofWitnessStoreError::TrustedUsedAxiomsProjectionMismatch {
                expected: None,
                actual: Some(hash(90)),
            }
        )
    );
}

#[test]
fn draft_rejects_mismatched_selected_proof_witness_hash() {
    let obligation_fingerprint = artifact_ref(ArtifactHashClass::Interface, "obligation", 1);
    let anchor = anchor();
    let projection = status_projection_with_selected_witness_hash(
        "obl-1",
        anchor.clone(),
        &obligation_fingerprint,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(2),
        hash(99),
    );

    assert!(matches!(
        ProofWitnessDraft::new(
            "obl-1",
            anchor,
            obligation_fingerprint,
            &projection,
            trusted_metadata(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2), 9),
            payload_schema(),
            b"canonical witness payload".to_vec(),
            "proof-witnesses/obl-1.json",
            provenance(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2)),
        ),
        Err(ProofWitnessStoreError::StatusProjectionMismatch {
            field: "selected_proof_witness_hash",
            ..
        })
    ));
}

#[test]
fn unaccepted_or_policy_tainted_kernel_input_cannot_create_trusted_witness_metadata() {
    let cases = [
        KernelPolicyInput::for_test(
            KernelCheckStatus::Rejected,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            false,
            Some(hash(2)),
        ),
        KernelPolicyInput::for_test(
            KernelCheckStatus::Accepted,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            true,
            Some(hash(2)),
        ),
        KernelPolicyInput::for_test(
            KernelCheckStatus::Accepted,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            false,
            None,
        ),
    ];

    for input in cases {
        assert_eq!(
            TrustedKernelWitnessMetadata::from_kernel_policy_input(
                &input,
                kernel_acceptance_metadata(9, hash(2)),
            ),
            Err(ProofWitnessStoreError::KernelEvidenceNotTrusted),
            "only accepted, untainted kernel inputs with accepted evidence hashes create witness metadata"
        );
    }
}

#[test]
fn trusted_witness_metadata_rejects_mismatched_accepted_result_hash() {
    let input = KernelPolicyInput::for_test(
        KernelCheckStatus::Accepted,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        false,
        Some(hash(2)),
    );

    assert!(matches!(
        TrustedKernelWitnessMetadata::from_kernel_policy_input(
            &input,
            kernel_acceptance_metadata(9, hash(99)),
        ),
        Err(ProofWitnessStoreError::AcceptedEvidenceHashMismatch { .. })
    ));
}

#[test]
fn discharged_builtin_stages_internal_hash_but_does_not_publish_ref() {
    let staged = stage(discharged_builtin_draft()).expect("builtin draft stages internally");

    assert_eq!(staged.selected_class(), ProofWinnerClass::DischargedBuiltin);
    assert!(staged.artifact_reference_candidate().is_none());
    assert_eq!(
        publish_ref(
            &staged,
            &CommittedWitnessPublicationProof::for_test(
                module_entry(Vec::new(), 1),
                staged.provenance().build_snapshot_fingerprint(),
                Vec::new(),
            )
            .expect("empty coverage token is valid")
        ),
        Err(ProofWitnessStoreError::UnsupportedWitnessPublication {
            selected_class: ProofWinnerClass::DischargedBuiltin,
            gap: "WITNESS10-G001: mizar-artifact ProofWitnessRef 2.0 has no discharged_builtin witness status/evidence pair",
        })
    );
}

#[test]
fn publication_proof_rejects_duplicate_manifest_reference() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref")
        .clone();
    let entry = manifest_entry(&reference);

    assert!(matches!(
        publication_proof(&staged, vec![reference], vec![entry.clone(), entry]),
        Err(ProofWitnessStoreError::DuplicateManifestWitnessReference { .. })
    ));
}

#[test]
fn publication_proof_rejects_duplicate_artifact_reference() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref")
        .clone();

    assert!(matches!(
        publication_proof(
            &staged,
            vec![reference.clone(), reference.clone()],
            vec![manifest_entry(&reference)]
        ),
        Err(ProofWitnessStoreError::DuplicateArtifactWitnessReference { .. })
    ));
}

#[test]
fn publication_proof_rejects_manifest_tuple_mismatch() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref")
        .clone();
    let mut mismatched = manifest_entry(&reference);
    mismatched.witness_artifact_hash = artifact_ref(ArtifactHashClass::Artifact, "mismatch", 8);

    assert!(matches!(
        publication_proof(&staged, vec![reference], vec![mismatched]),
        Err(ProofWitnessStoreError::ManifestCoverageMismatch { .. })
    ));
}

#[test]
fn publication_proof_rejects_manifest_path_escape() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref")
        .clone();
    let mut escaped = manifest_entry(&reference);
    escaped.witness_path = "proof-witnesses/../escape.json".to_owned();

    assert!(matches!(
        publication_proof(&staged, Vec::new(), vec![escaped]),
        Err(ProofWitnessStoreError::InvalidWitnessPath { .. })
    ));
}

#[test]
fn publication_proof_rejects_wrong_module_artifact_hash_class() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref")
        .clone();
    let mut module_entry = module_entry(vec![manifest_entry(&reference)], 1);
    module_entry.artifact_hash =
        artifact_ref(ArtifactHashClass::Interface, "wrong-artifact-class", 8);

    assert!(matches!(
        CommittedWitnessPublicationProof::for_test(
            module_entry,
            staged.provenance().build_snapshot_fingerprint(),
            vec![reference],
        ),
        Err(ProofWitnessStoreError::HashClassMismatch {
            field: "module_entry.artifact_hash",
            expected: ArtifactHashClass::Artifact,
            actual: ArtifactHashClass::Interface,
        })
    ));
}

#[test]
fn publication_proof_rejects_wrong_manifest_hash_classes() {
    let staged = stage(kernel_draft()).expect("kernel draft stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref")
        .clone();

    let mut wrong_obligation = manifest_entry(&reference);
    wrong_obligation.obligation_fingerprint =
        artifact_ref(ArtifactHashClass::Artifact, "wrong-obligation-class", 9);
    assert!(matches!(
        publication_proof(&staged, Vec::new(), vec![wrong_obligation]),
        Err(ProofWitnessStoreError::HashClassMismatch {
            field: "manifest_obligation_fingerprint",
            expected: ArtifactHashClass::Interface,
            actual: ArtifactHashClass::Artifact,
        })
    ));

    let mut wrong_witness = manifest_entry(&reference);
    wrong_witness.witness_artifact_hash =
        artifact_ref(ArtifactHashClass::Interface, "wrong-witness-class", 10);
    assert!(matches!(
        publication_proof(&staged, Vec::new(), vec![wrong_witness]),
        Err(ProofWitnessStoreError::HashClassMismatch {
            field: "manifest_witness_artifact_hash",
            expected: ArtifactHashClass::Artifact,
            actual: ArtifactHashClass::Interface,
        })
    ));
}

#[test]
fn payload_schema_family_rejects_malformed_segments() {
    assert!(matches!(
        ProofWitnessPayloadSchema::new("bad//family", SchemaVersion::new(1, 0)),
        Err(ProofWitnessStoreError::MalformedPayloadSchemaFamily { .. })
    ));
    assert!(matches!(
        ProofWitnessPayloadSchema::new("bad:family", SchemaVersion::new(1, 0)),
        Err(ProofWitnessStoreError::MalformedPayloadSchemaFamily { .. })
    ));
}

fn publication_proof(
    staged: &ProofWitnessStagedRef,
    artifact_witnesses: Vec<ProofWitnessRef>,
    manifest_witnesses: Vec<ManifestProofWitnessEntry>,
) -> Result<CommittedWitnessPublicationProof, ProofWitnessStoreError> {
    CommittedWitnessPublicationProof::for_test(
        module_entry(manifest_witnesses, 1),
        staged.provenance().build_snapshot_fingerprint(),
        artifact_witnesses,
    )
}

fn module_entry(proof_witnesses: Vec<ManifestProofWitnessEntry>, seed: u8) -> ModuleArtifactEntry {
    ModuleArtifactEntry {
        module: module_identity(),
        source_file: "src/article.miz".to_owned(),
        source_hash: hash(seed.wrapping_add(1)),
        artifact_file: "artifacts/article.verified.json".to_owned(),
        artifact_hash: artifact_ref(ArtifactHashClass::Artifact, "main-artifact", seed),
        interface_hash: artifact_ref(ArtifactHashClass::Interface, "interface", seed),
        implementation_hash: artifact_ref(
            ArtifactHashClass::Implementation,
            "implementation",
            seed,
        ),
        module_summary_file: None,
        module_summary_hash: None,
        module_summary_interface_hash: None,
        registration_summary_file: None,
        registration_summary_hash: None,
        registration_interface_hash: None,
        proof_witnesses,
        diagnostics_hash: None,
    }
}

fn module_identity() -> ModuleSummaryIdentity {
    ModuleSummaryIdentity {
        package_id: "mml".to_owned(),
        package_version: Some("1.0.0".to_owned()),
        lockfile_identity: Some("lock".to_owned()),
        module_path: "article".to_owned(),
        language_edition: "2026".to_owned(),
    }
}

fn manifest_entry(reference: &ProofWitnessRef) -> ManifestProofWitnessEntry {
    ManifestProofWitnessEntry {
        obligation_id: reference.obligation_id.clone(),
        obligation_fingerprint: reference.obligation_fingerprint.clone(),
        witness_path: reference.witness_path.clone(),
        witness_artifact_hash: reference.witness_artifact_hash.clone(),
    }
}

fn different_reference(staged: &ProofWitnessStagedRef, seed: u8) -> ProofWitnessRef {
    ProofWitnessRef {
        schema_version: current_witness_ref_version(),
        obligation_id: "other-obligation".to_owned(),
        obligation_fingerprint: artifact_ref(ArtifactHashClass::Interface, "other", seed),
        proof_status: ArtifactProofStatus::KernelVerified,
        evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
        witness_path: "proof-witnesses/other.json".to_owned(),
        witness_artifact_hash: staged.staged_payload_hash().clone(),
        kernel_acceptance: kernel_acceptance_metadata(seed, hash(seed)),
    }
}

fn kernel_draft() -> ProofWitnessDraft {
    let obligation_fingerprint = artifact_ref(ArtifactHashClass::Interface, "obligation", 1);
    let anchor = anchor();
    let projection = status_projection(
        "obl-1",
        anchor.clone(),
        &obligation_fingerprint,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        hash(2),
        None,
    );
    ProofWitnessDraft::new(
        "obl-1",
        anchor,
        obligation_fingerprint,
        &projection,
        trusted_metadata(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2), 9),
        payload_schema(),
        b"canonical witness payload".to_vec(),
        "proof-witnesses/obl-1.json",
        provenance(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2)),
    )
    .expect("valid kernel draft")
}

fn discharged_builtin_draft() -> ProofWitnessDraft {
    let obligation_fingerprint = artifact_ref(ArtifactHashClass::Interface, "obligation", 3);
    let anchor = anchor();
    let projection = status_projection(
        "obl-builtin",
        anchor.clone(),
        &obligation_fingerprint,
        KernelEvidenceOrigin::BuiltinDischarge,
        hash(4),
        None,
    );
    ProofWitnessDraft::new(
        "obl-builtin",
        anchor,
        obligation_fingerprint,
        &projection,
        trusted_metadata(KernelEvidenceOrigin::BuiltinDischarge, hash(4), 10),
        payload_schema(),
        b"builtin internal payload".to_vec(),
        "proof-witnesses/obl-builtin.json",
        provenance(KernelEvidenceOrigin::BuiltinDischarge, hash(4)),
    )
    .expect("valid builtin draft")
}

fn witness_roundtrip_from_candidate_order(
    order: [&'static str; 2],
) -> (ProofWitnessStagedRef, ProofWitnessPublishedRef) {
    let obligation_fingerprint = artifact_ref(ArtifactHashClass::Interface, "obligation", 1);
    let anchor = anchor();
    let projection = status_projection_from_candidate_order(
        "obl-1",
        anchor.clone(),
        &obligation_fingerprint,
        KernelEvidenceOrigin::AtpFormulaSubstitution,
        order,
    );
    let draft = ProofWitnessDraft::new(
        "obl-1",
        anchor,
        obligation_fingerprint,
        &projection,
        trusted_metadata(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2), 9),
        payload_schema(),
        b"canonical witness payload".to_vec(),
        "proof-witnesses/obl-1.json",
        provenance(KernelEvidenceOrigin::AtpFormulaSubstitution, hash(2)),
    )
    .expect("valid deterministic witness draft");
    let staged = stage(draft).expect("deterministic witness stages");
    let reference = staged
        .artifact_reference_candidate()
        .expect("kernel witness creates unpublished artifact ref")
        .clone();
    let proof = publication_proof(
        &staged,
        vec![reference.clone()],
        vec![manifest_entry(&reference)],
    )
    .expect("coverage token is internally consistent");
    let published = publish_ref(&staged, &proof).expect("published witness ref");
    (staged, published)
}

trait DraftTestExt {
    fn with_payload_bytes_for_test(self, bytes: Vec<u8>) -> Self;
    fn with_selected_evidence_hash_for_test(self, hash: Hash) -> Self;
    fn with_witness_path_for_test(self, path: &'static str) -> Self;
    fn with_build_snapshot_for_test(self, hash: Hash) -> Self;
    fn with_advisory_backend_ref_for_test(self, hash: Hash) -> Self;
    fn with_verifier_policy_for_test(self, hash: Hash) -> Self;
    fn with_obligation_fingerprint_for_test(self, seed: u8) -> Self;
    fn with_payload_schema_version_for_test(self, minor: u16) -> Self;
}

impl DraftTestExt for ProofWitnessDraft {
    fn with_payload_bytes_for_test(mut self, bytes: Vec<u8>) -> Self {
        self.payload_bytes = bytes;
        self
    }

    fn with_selected_evidence_hash_for_test(mut self, hash: Hash) -> Self {
        self.selected_evidence_hash = hash;
        self.kernel_acceptance.accepted_result_hash =
            artifact_ref_with_digest(ArtifactHashClass::Interface, "accepted", hash);
        self.provenance.accepted_result_hash = Some(hash);
        self
    }

    fn with_witness_path_for_test(mut self, path: &'static str) -> Self {
        self.witness_path = path.to_owned();
        self
    }

    fn with_build_snapshot_for_test(mut self, hash: Hash) -> Self {
        self.provenance.build_snapshot_fingerprint = hash;
        self
    }

    fn with_advisory_backend_ref_for_test(mut self, hash: Hash) -> Self {
        self.provenance.advisory_backend_ref = Some(hash);
        self
    }

    fn with_verifier_policy_for_test(mut self, hash: Hash) -> Self {
        self.provenance.verifier_policy_fingerprint = hash;
        self.kernel_acceptance.verifier_policy_fingerprint =
            artifact_ref_with_digest(ArtifactHashClass::Interface, "policy", hash);
        self
    }

    fn with_obligation_fingerprint_for_test(mut self, seed: u8) -> Self {
        self.obligation_fingerprint =
            artifact_ref(ArtifactHashClass::Interface, "obligation-changed", seed);
        self
    }

    fn with_payload_schema_version_for_test(mut self, minor: u16) -> Self {
        self.payload_schema = ProofWitnessPayloadSchema::new(
            "mizar-proof/test-witness-payload",
            SchemaVersion::new(1, minor),
        )
        .expect("schema");
        self
    }
}

fn payload_schema() -> ProofWitnessPayloadSchema {
    ProofWitnessPayloadSchema::new("mizar-proof/test-witness-payload", SchemaVersion::new(1, 0))
        .expect("schema")
}

fn provenance(origin: KernelEvidenceOrigin, accepted_hash: Hash) -> ProofWitnessProvenance {
    provenance_for_candidate(origin, accepted_hash, candidate_id())
}

fn provenance_for_candidate(
    origin: KernelEvidenceOrigin,
    accepted_hash: Hash,
    selected_candidate_id: CandidateSourceId,
) -> ProofWitnessProvenance {
    ProofWitnessProvenance::new(
        hash(20),
        "mizar-proof-test",
        selected_candidate_id,
        origin,
        hash(21),
        hash(22),
        hash(23),
        policy_fingerprint_hash(),
        SchemaVersion::new(1, 0),
        SchemaVersion::new(1, 0),
    )
    .expect("provenance")
    .with_accepted_result_hash(accepted_hash)
    .with_advisory_backend_ref(hash(27))
}

fn anchor() -> ObligationAnchor {
    ObligationAnchor::new("anchor-1").expect("anchor")
}

fn trusted_metadata(
    origin: KernelEvidenceOrigin,
    accepted_hash: Hash,
    metadata_seed: u8,
) -> TrustedKernelWitnessMetadata {
    let input = KernelPolicyInput::for_test(
        KernelCheckStatus::Accepted,
        origin,
        false,
        Some(accepted_hash),
    );
    TrustedKernelWitnessMetadata::from_kernel_policy_input(
        &input,
        kernel_acceptance_metadata(metadata_seed, accepted_hash),
    )
    .expect("trusted witness metadata")
}

fn status_projection(
    obligation_id: &'static str,
    obligation_anchor: ObligationAnchor,
    obligation_fingerprint: &ArtifactHashRef,
    origin: KernelEvidenceOrigin,
    accepted_hash: Hash,
    trusted_used_axioms: Option<TrustedUsedAxiomsRef>,
) -> ProofStatusProjection {
    status_projection_with_options(
        obligation_id,
        obligation_anchor,
        obligation_fingerprint,
        origin,
        accepted_hash,
        trusted_used_axioms,
        None,
    )
}

fn status_projection_with_selected_witness_hash(
    obligation_id: &'static str,
    obligation_anchor: ObligationAnchor,
    obligation_fingerprint: &ArtifactHashRef,
    origin: KernelEvidenceOrigin,
    accepted_hash: Hash,
    selected_witness_hash: Hash,
) -> ProofStatusProjection {
    status_projection_with_options(
        obligation_id,
        obligation_anchor,
        obligation_fingerprint,
        origin,
        accepted_hash,
        None,
        Some(selected_witness_hash),
    )
}

fn status_projection_with_options(
    obligation_id: &'static str,
    obligation_anchor: ObligationAnchor,
    obligation_fingerprint: &ArtifactHashRef,
    origin: KernelEvidenceOrigin,
    accepted_hash: Hash,
    trusted_used_axioms: Option<TrustedUsedAxiomsRef>,
    selected_witness_hash: Option<Hash>,
) -> ProofStatusProjection {
    let policy = VerifierPolicy::release();
    let selection =
        artifact_selection_for_origin(origin, accepted_hash, selected_witness_hash, policy.clone());
    let identity = ProofObligationIdentity::new(
        obligation_id,
        obligation_anchor,
        obligation_fingerprint.digest,
        hash(21),
        hash(28),
        hash(22),
    )
    .expect("valid proof obligation identity");
    let mut input = ProofStatusProjectionInput::new(selection, policy, identity);
    if let Some(trusted_used_axioms) = trusted_used_axioms {
        input = input.with_trusted_used_axioms(trusted_used_axioms);
    }
    project_status(input).expect("trusted projection succeeds")
}

fn status_projection_from_candidate_order(
    obligation_id: &'static str,
    obligation_anchor: ObligationAnchor,
    obligation_fingerprint: &ArtifactHashRef,
    origin: KernelEvidenceOrigin,
    order: [&'static str; 2],
) -> ProofStatusProjection {
    let policy = VerifierPolicy::release();
    let selection = artifact_selection_for_origin_with_order(origin, order, policy.clone());
    let identity = ProofObligationIdentity::new(
        obligation_id,
        obligation_anchor,
        obligation_fingerprint.digest,
        hash(21),
        hash(28),
        hash(22),
    )
    .expect("valid proof obligation identity");
    project_status(ProofStatusProjectionInput::new(selection, policy, identity))
        .expect("trusted projection succeeds")
}

fn artifact_selection_for_origin(
    origin: KernelEvidenceOrigin,
    accepted_hash: Hash,
    selected_witness_hash: Option<Hash>,
    policy: VerifierPolicy,
) -> ArtifactProofSelection {
    let mut candidate = trusted_candidate(origin, accepted_hash);
    if let Some(selected_witness_hash) = selected_witness_hash {
        candidate = candidate.with_selected_proof_witness_hash(selected_witness_hash);
    }
    let selection = select_winner(
        &ProofEvidenceSet::new(b"witness-obligation".to_vec(), hash(100), policy)
            .with_candidates([candidate]),
    );
    let vc_selection = VcProofSelection::new(VcId::new(0), selection);
    let mut merged = match origin {
        KernelEvidenceOrigin::AtpFormulaSubstitution => {
            merge_artifact_proof_selections([vc_selection], [])
        }
        KernelEvidenceOrigin::BuiltinDischarge | KernelEvidenceOrigin::KernelPrimitive => {
            merge_artifact_proof_selections([], [vc_selection])
        }
    }
    .expect("artifact selection merge succeeds");
    assert_eq!(merged.len(), 1);
    merged.remove(0)
}

fn artifact_selection_for_origin_with_order(
    origin: KernelEvidenceOrigin,
    order: [&'static str; 2],
    policy: VerifierPolicy,
) -> ArtifactProofSelection {
    let candidates = order.into_iter().map(|id| match id {
        "candidate-1" => trusted_candidate_with_id(id, origin, hash(2))
            .with_backend_profile_priority(0)
            .with_evidence_format_priority(0),
        "candidate-later" => trusted_candidate_with_id(id, origin, hash(3))
            .with_backend_profile_priority(1)
            .with_evidence_format_priority(0),
        _ => panic!("unknown candidate id in determinism fixture"),
    });
    let selection = select_winner(
        &ProofEvidenceSet::new(b"witness-obligation".to_vec(), hash(100), policy)
            .with_candidates(candidates),
    );
    assert_eq!(
        selection
            .selected_candidate_id()
            .map(CandidateSourceId::as_str),
        Some("candidate-1")
    );
    let vc_selection = VcProofSelection::new(VcId::new(0), selection);
    let mut merged = match origin {
        KernelEvidenceOrigin::AtpFormulaSubstitution => {
            merge_artifact_proof_selections([vc_selection], [])
        }
        KernelEvidenceOrigin::BuiltinDischarge | KernelEvidenceOrigin::KernelPrimitive => {
            merge_artifact_proof_selections([], [vc_selection])
        }
    }
    .expect("artifact selection merge succeeds");
    assert_eq!(merged.len(), 1);
    merged.remove(0)
}

fn trusted_candidate(origin: KernelEvidenceOrigin, accepted_hash: Hash) -> ProofEvidenceCandidate {
    trusted_candidate_with_id("candidate-1", origin, accepted_hash)
}

fn trusted_candidate_with_id(
    id: &'static str,
    origin: KernelEvidenceOrigin,
    accepted_hash: Hash,
) -> ProofEvidenceCandidate {
    let input = KernelPolicyInput::for_test(
        KernelCheckStatus::Accepted,
        origin,
        false,
        Some(accepted_hash),
    );
    ProofEvidenceCandidate::from_trusted_kernel_input(candidate_id_named(id), &input)
        .expect("accepted kernel policy input is trusted")
}

fn candidate_id() -> CandidateSourceId {
    candidate_id_named("candidate-1")
}

fn candidate_id_named(id: &'static str) -> CandidateSourceId {
    CandidateSourceId::new(id).expect("candidate id")
}

fn policy_fingerprint_hash() -> Hash {
    VerifierPolicy::release().policy_fingerprint().hash()
}

fn kernel_acceptance_metadata(seed: u8, accepted_hash: Hash) -> KernelAcceptanceMetadata {
    KernelAcceptanceMetadata {
        kernel_profile_fingerprint: artifact_ref(ArtifactHashClass::Interface, "kernel", seed),
        verifier_policy_fingerprint: artifact_ref_with_digest(
            ArtifactHashClass::Interface,
            "policy",
            policy_fingerprint_hash(),
        ),
        checker_schema_version: SchemaVersion::new(1, 0),
        evidence_schema_version: SchemaVersion::new(1, 0),
        target_binding_hash: artifact_ref(
            ArtifactHashClass::Interface,
            "target",
            seed.wrapping_add(2),
        ),
        formula_evidence_hash: artifact_ref(
            ArtifactHashClass::Interface,
            "formula",
            seed.wrapping_add(3),
        ),
        substitution_evidence_hash: artifact_ref(
            ArtifactHashClass::Interface,
            "subst",
            seed.wrapping_add(4),
        ),
        provenance_hash: artifact_ref(ArtifactHashClass::Interface, "prov", seed.wrapping_add(5)),
        formula_context_hash: Some(artifact_ref(
            ArtifactHashClass::Interface,
            "ctx",
            seed.wrapping_add(6),
        )),
        accepted_result_hash: artifact_ref_with_digest(
            ArtifactHashClass::Interface,
            "accepted",
            accepted_hash,
        ),
    }
}

fn artifact_ref(class: ArtifactHashClass, family: &'static str, seed: u8) -> ArtifactHashRef {
    artifact_ref_with_digest(class, family, hash(seed))
}

fn artifact_ref_with_digest(
    class: ArtifactHashClass,
    family: &'static str,
    digest: Hash,
) -> ArtifactHashRef {
    ArtifactHashRef::new(
        class,
        format!("mizar-proof/test-{family}"),
        SchemaVersion::new(1, 0),
        digest,
    )
}

fn hash(seed: u8) -> Hash {
    let mut bytes = [0; Hash::BYTE_LEN];
    bytes[0] = seed;
    Hash::from_bytes(bytes)
}
