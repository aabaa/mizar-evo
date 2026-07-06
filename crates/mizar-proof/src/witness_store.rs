//! Proof witness draft staging and manifest-gated publication references.
//!
//! This module does not accept proofs, run proof search, call ATP backends,
//! call the kernel, query caches, or write artifact manifests. It prepares
//! artifact witness-reference candidates for already-selected trusted evidence
//! and validates publication only when the artifact boundary supplies committed
//! manifest reachability.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use mizar_artifact::{
    manifest::{ManifestProofWitnessEntry, ModuleArtifactEntry},
    proof_witness::{
        EvidenceKind, KernelAcceptanceMetadata, ProofStatus as ArtifactProofStatus,
        ProofWitnessError, ProofWitnessRef, current_schema_version as current_witness_ref_version,
        write_proof_witness_ref,
    },
    registration_summary::{ArtifactHashClass, ArtifactHashRef},
    store::{CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersion},
};
use mizar_session::Hash;

use crate::{
    policy::{KernelEvidenceOrigin, PolicyFingerprint},
    selection::{CandidateSourceId, ProofWinnerClass, TrustedKernelEvidence},
    status::{ObligationAnchor, ProjectedProofStatus, ProofStatusProjection, TrustedUsedAxiomsRef},
};

const WITNESS_PAYLOAD_HASH_DOMAIN: &str = "mizar-proof/witness-payload/v1";

/// Payload schema identity for producer-owned witness bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofWitnessPayloadSchema {
    family: String,
    version: SchemaVersion,
    canonical_bytes_required: bool,
}

impl ProofWitnessPayloadSchema {
    /// Creates a non-empty payload schema identity.
    pub fn new(
        family: impl Into<String>,
        version: SchemaVersion,
    ) -> Result<Self, ProofWitnessStoreError> {
        Self::with_canonical_bytes_required(family, version, false)
    }

    /// Creates a payload schema identity and records whether the producer
    /// promises canonical bytes.
    pub fn with_canonical_bytes_required(
        family: impl Into<String>,
        version: SchemaVersion,
        canonical_bytes_required: bool,
    ) -> Result<Self, ProofWitnessStoreError> {
        let family = family.into();
        validate_payload_schema_family(&family)?;
        Ok(Self {
            family,
            version,
            canonical_bytes_required,
        })
    }

    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }

    #[must_use]
    pub const fn version(&self) -> SchemaVersion {
        self.version
    }

    #[must_use]
    pub const fn canonical_bytes_required(&self) -> bool {
        self.canonical_bytes_required
    }
}

/// Diagnostics and reuse-validation provenance carried with staged witnesses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofWitnessProvenance {
    build_snapshot_fingerprint: Hash,
    producer_identity: String,
    selected_candidate_id: CandidateSourceId,
    kernel_evidence_origin: KernelEvidenceOrigin,
    target_vc_fingerprint: Hash,
    dependency_slice_fingerprint: Hash,
    dependency_artifact_fingerprint: Hash,
    verifier_policy_fingerprint: Hash,
    checker_schema_version: SchemaVersion,
    evidence_schema_version: SchemaVersion,
    accepted_result_hash: Option<Hash>,
    trusted_used_axioms_ref: Option<TrustedUsedAxiomsRef>,
    advisory_backend_ref: Option<Hash>,
}

impl ProofWitnessProvenance {
    /// Creates provenance metadata for a staged witness.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        build_snapshot_fingerprint: Hash,
        producer_identity: impl Into<String>,
        selected_candidate_id: CandidateSourceId,
        kernel_evidence_origin: KernelEvidenceOrigin,
        target_vc_fingerprint: Hash,
        dependency_slice_fingerprint: Hash,
        dependency_artifact_fingerprint: Hash,
        verifier_policy_fingerprint: Hash,
        checker_schema_version: SchemaVersion,
        evidence_schema_version: SchemaVersion,
    ) -> Result<Self, ProofWitnessStoreError> {
        let producer_identity = producer_identity.into();
        if producer_identity.is_empty() {
            return Err(ProofWitnessStoreError::EmptyProducerIdentity);
        }
        Ok(Self {
            build_snapshot_fingerprint,
            producer_identity,
            selected_candidate_id,
            kernel_evidence_origin,
            target_vc_fingerprint,
            dependency_slice_fingerprint,
            dependency_artifact_fingerprint,
            verifier_policy_fingerprint,
            checker_schema_version,
            evidence_schema_version,
            accepted_result_hash: None,
            trusted_used_axioms_ref: None,
            advisory_backend_ref: None,
        })
    }

    #[must_use]
    pub const fn with_accepted_result_hash(mut self, hash: Hash) -> Self {
        self.accepted_result_hash = Some(hash);
        self
    }

    #[must_use]
    pub fn with_trusted_used_axioms_ref(mut self, reference: &TrustedUsedAxiomsRef) -> Self {
        self.trusted_used_axioms_ref = Some(reference.clone());
        self
    }

    #[must_use]
    pub const fn with_advisory_backend_ref(mut self, hash: Hash) -> Self {
        self.advisory_backend_ref = Some(hash);
        self
    }

    #[must_use]
    pub const fn build_snapshot_fingerprint(&self) -> Hash {
        self.build_snapshot_fingerprint
    }

    #[must_use]
    pub fn producer_identity(&self) -> &str {
        &self.producer_identity
    }

    #[must_use]
    pub const fn selected_candidate_id(&self) -> &CandidateSourceId {
        &self.selected_candidate_id
    }

    #[must_use]
    pub const fn kernel_evidence_origin(&self) -> KernelEvidenceOrigin {
        self.kernel_evidence_origin
    }

    #[must_use]
    pub const fn target_vc_fingerprint(&self) -> Hash {
        self.target_vc_fingerprint
    }

    #[must_use]
    pub const fn dependency_slice_fingerprint(&self) -> Hash {
        self.dependency_slice_fingerprint
    }

    #[must_use]
    pub const fn dependency_artifact_fingerprint(&self) -> Hash {
        self.dependency_artifact_fingerprint
    }

    #[must_use]
    pub const fn verifier_policy_fingerprint(&self) -> Hash {
        self.verifier_policy_fingerprint
    }

    #[must_use]
    pub const fn checker_schema_version(&self) -> SchemaVersion {
        self.checker_schema_version
    }

    #[must_use]
    pub const fn evidence_schema_version(&self) -> SchemaVersion {
        self.evidence_schema_version
    }

    #[must_use]
    pub const fn accepted_result_hash(&self) -> Option<Hash> {
        self.accepted_result_hash
    }

    #[must_use]
    pub fn trusted_used_axioms_ref(&self) -> Option<&TrustedUsedAxiomsRef> {
        self.trusted_used_axioms_ref.as_ref()
    }

    #[must_use]
    pub fn trusted_used_axiom_ref_hash(&self) -> Option<Hash> {
        self.trusted_used_axioms_ref
            .as_ref()
            .map(TrustedUsedAxiomsRef::used_axioms_hash)
    }

    #[must_use]
    pub const fn advisory_backend_ref(&self) -> Option<Hash> {
        self.advisory_backend_ref
    }
}

/// Kernel-derived trusted evidence plus artifact witness metadata anchored to
/// that evidence.
///
/// `mizar-artifact` owns the published metadata shape, but this wrapper keeps
/// the metadata bound to the accepted proof-obligation kernel evidence hash
/// before witness staging can turn it into a `ProofWitnessRef`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedKernelWitnessMetadata {
    trusted_kernel_evidence: TrustedKernelEvidence,
    kernel_evidence_origin: KernelEvidenceOrigin,
    kernel_acceptance: KernelAcceptanceMetadata,
}

impl TrustedKernelWitnessMetadata {
    /// Creates witness metadata from a kernel-derived policy input and matching
    /// artifact kernel-acceptance projection.
    ///
    /// This test-only constructor stands in for the future kernel/artifact
    /// boundary token. Production callers cannot synthesize trusted
    /// `KernelAcceptanceMetadata` inside `mizar-proof`.
    #[cfg(test)]
    fn from_kernel_policy_input(
        input: &crate::policy::KernelPolicyInput,
        kernel_acceptance: KernelAcceptanceMetadata,
    ) -> Result<Self, ProofWitnessStoreError> {
        let trusted_kernel_evidence = TrustedKernelEvidence::from_policy_input(input)
            .ok_or(ProofWitnessStoreError::KernelEvidenceNotTrusted)?;
        validate_kernel_acceptance_metadata(&kernel_acceptance)?;
        if kernel_acceptance.accepted_result_hash.digest
            != trusted_kernel_evidence.accepted_evidence_hash()
        {
            return Err(ProofWitnessStoreError::AcceptedEvidenceHashMismatch {
                expected: trusted_kernel_evidence.accepted_evidence_hash(),
                actual: kernel_acceptance.accepted_result_hash.digest,
            });
        }
        Ok(Self {
            trusted_kernel_evidence,
            kernel_evidence_origin: input.origin(),
            kernel_acceptance,
        })
    }

    #[must_use]
    pub const fn trusted_kernel_evidence(&self) -> &TrustedKernelEvidence {
        &self.trusted_kernel_evidence
    }

    #[must_use]
    pub const fn kernel_acceptance(&self) -> &KernelAcceptanceMetadata {
        &self.kernel_acceptance
    }

    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.trusted_kernel_evidence.selected_class()
    }

    #[must_use]
    pub const fn kernel_evidence_origin(&self) -> KernelEvidenceOrigin {
        self.kernel_evidence_origin
    }

    #[must_use]
    pub const fn accepted_evidence_hash(&self) -> Hash {
        self.trusted_kernel_evidence.accepted_evidence_hash()
    }
}

/// Producer-owned draft consumed before artifact commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofWitnessDraft {
    obligation_id: String,
    obligation_anchor: ObligationAnchor,
    obligation_fingerprint: ArtifactHashRef,
    selected_class: ProofWinnerClass,
    selected_evidence_hash: Hash,
    expected_selected_proof_witness_hash: Option<Hash>,
    payload_schema: ProofWitnessPayloadSchema,
    payload_bytes: Vec<u8>,
    witness_path: String,
    kernel_acceptance: KernelAcceptanceMetadata,
    provenance: ProofWitnessProvenance,
}

impl ProofWitnessDraft {
    /// Creates a proof witness draft for a trusted selection.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        obligation_id: impl Into<String>,
        obligation_anchor: ObligationAnchor,
        obligation_fingerprint: ArtifactHashRef,
        status_projection: &ProofStatusProjection,
        trusted_kernel_metadata: TrustedKernelWitnessMetadata,
        payload_schema: ProofWitnessPayloadSchema,
        payload_bytes: impl Into<Vec<u8>>,
        witness_path: impl Into<String>,
        provenance: ProofWitnessProvenance,
    ) -> Result<Self, ProofWitnessStoreError> {
        let obligation_id = obligation_id.into();
        if obligation_id.is_empty() {
            return Err(ProofWitnessStoreError::EmptyObligationId);
        }
        let selected_class = trusted_kernel_metadata.selected_class();
        let selected_evidence_hash = trusted_kernel_metadata.accepted_evidence_hash();
        match selected_class {
            ProofWinnerClass::KernelVerified | ProofWinnerClass::DischargedBuiltin => {}
            ProofWinnerClass::PolicyPermittedExternal
            | ProofWinnerClass::PolicyAssumed
            | ProofWinnerClass::PolicyOpen
            | ProofWinnerClass::Rejected
            | ProofWinnerClass::NoSelectableEvidence => {
                return Err(ProofWitnessStoreError::UnsupportedWitnessClass { selected_class });
            }
        }
        require_hash_class(
            &obligation_fingerprint,
            ArtifactHashClass::Interface,
            "obligation_fingerprint",
        )?;
        validate_status_projection(
            status_projection,
            &obligation_id,
            &obligation_anchor,
            &obligation_fingerprint,
            &trusted_kernel_metadata,
            &provenance,
        )?;
        let expected_selected_proof_witness_hash = status_projection
            .reuse_metadata()
            .selected_proof_witness_hash();
        let payload_bytes = payload_bytes.into();
        if payload_schema.canonical_bytes_required() && payload_bytes.is_empty() {
            return Err(ProofWitnessStoreError::NonCanonicalPayloadBytes {
                reason: "canonical witness payload bytes must be non-empty",
            });
        }
        let witness_path = witness_path.into();
        validate_witness_path(&witness_path)?;
        let staged_payload_hash = witness_payload_hash_ref_from_inputs(
            &obligation_fingerprint,
            selected_evidence_hash,
            &payload_schema,
            &payload_bytes,
            provenance.verifier_policy_fingerprint(),
        );
        validate_selected_proof_witness_hash(
            expected_selected_proof_witness_hash,
            &staged_payload_hash,
        )?;
        Ok(Self {
            obligation_id,
            obligation_anchor,
            obligation_fingerprint,
            selected_class,
            selected_evidence_hash,
            expected_selected_proof_witness_hash,
            payload_schema,
            payload_bytes,
            witness_path,
            kernel_acceptance: trusted_kernel_metadata.kernel_acceptance,
            provenance,
        })
    }

    #[must_use]
    pub fn obligation_id(&self) -> &str {
        &self.obligation_id
    }

    #[must_use]
    pub const fn obligation_anchor(&self) -> &ObligationAnchor {
        &self.obligation_anchor
    }

    #[must_use]
    pub const fn obligation_fingerprint(&self) -> &ArtifactHashRef {
        &self.obligation_fingerprint
    }

    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.selected_class
    }

    #[must_use]
    pub const fn selected_evidence_hash(&self) -> Hash {
        self.selected_evidence_hash
    }

    #[must_use]
    pub const fn payload_schema(&self) -> &ProofWitnessPayloadSchema {
        &self.payload_schema
    }

    #[must_use]
    pub fn payload_bytes(&self) -> &[u8] {
        &self.payload_bytes
    }

    #[must_use]
    pub fn witness_path(&self) -> &str {
        &self.witness_path
    }

    #[must_use]
    pub const fn kernel_acceptance(&self) -> &KernelAcceptanceMetadata {
        &self.kernel_acceptance
    }

    #[must_use]
    pub const fn provenance(&self) -> &ProofWitnessProvenance {
        &self.provenance
    }
}

/// Staged witness metadata returned before artifact commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofWitnessStagedRef {
    obligation_id: String,
    obligation_anchor: ObligationAnchor,
    obligation_fingerprint: ArtifactHashRef,
    selected_class: ProofWinnerClass,
    selected_evidence_hash: Hash,
    payload_schema: ProofWitnessPayloadSchema,
    witness_path: String,
    staged_payload_hash: ArtifactHashRef,
    artifact_reference_candidate: Option<ProofWitnessRef>,
    provenance: ProofWitnessProvenance,
}

impl ProofWitnessStagedRef {
    #[must_use]
    pub fn obligation_id(&self) -> &str {
        &self.obligation_id
    }

    #[must_use]
    pub const fn obligation_anchor(&self) -> &ObligationAnchor {
        &self.obligation_anchor
    }

    #[must_use]
    pub const fn obligation_fingerprint(&self) -> &ArtifactHashRef {
        &self.obligation_fingerprint
    }

    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.selected_class
    }

    #[must_use]
    pub const fn selected_evidence_hash(&self) -> Hash {
        self.selected_evidence_hash
    }

    #[must_use]
    pub const fn payload_schema(&self) -> &ProofWitnessPayloadSchema {
        &self.payload_schema
    }

    #[must_use]
    pub fn witness_path(&self) -> &str {
        &self.witness_path
    }

    #[must_use]
    pub const fn staged_payload_hash(&self) -> &ArtifactHashRef {
        &self.staged_payload_hash
    }

    #[must_use]
    pub const fn artifact_reference_candidate(&self) -> Option<&ProofWitnessRef> {
        self.artifact_reference_candidate.as_ref()
    }

    #[must_use]
    pub const fn provenance(&self) -> &ProofWitnessProvenance {
        &self.provenance
    }
}

/// Artifact-owned proof that a staged witness is reachable from a committed
/// module artifact and manifest entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommittedWitnessPublicationProof {
    committed_main_artifact_hash: ArtifactHashRef,
    build_snapshot_fingerprint: Hash,
    verified_artifact_witnesses: Vec<ProofWitnessRef>,
    module_entry: ModuleArtifactEntry,
}

impl CommittedWitnessPublicationProof {
    /// Test-only constructor standing in for the future artifact-owned commit
    /// boundary token.
    ///
    /// Production code intentionally has no public constructor here:
    /// `mizar-proof` must not treat caller-assembled matching tuples as
    /// committed manifest reachability.
    #[cfg(test)]
    fn for_test(
        module_entry: ModuleArtifactEntry,
        build_snapshot_fingerprint: Hash,
        verified_artifact_witnesses: Vec<ProofWitnessRef>,
    ) -> Result<Self, ProofWitnessStoreError> {
        require_hash_class(
            &module_entry.artifact_hash,
            ArtifactHashClass::Artifact,
            "module_entry.artifact_hash",
        )?;
        validate_exact_witness_coverage(
            &verified_artifact_witnesses,
            &module_entry.proof_witnesses,
        )?;
        Ok(Self {
            build_snapshot_fingerprint,
            verified_artifact_witnesses,
            committed_main_artifact_hash: module_entry.artifact_hash.clone(),
            module_entry,
        })
    }

    #[must_use]
    pub const fn committed_main_artifact_hash(&self) -> &ArtifactHashRef {
        &self.committed_main_artifact_hash
    }

    #[must_use]
    pub const fn build_snapshot_fingerprint(&self) -> Hash {
        self.build_snapshot_fingerprint
    }

    #[must_use]
    pub fn verified_artifact_witnesses(&self) -> &[ProofWitnessRef] {
        &self.verified_artifact_witnesses
    }

    #[must_use]
    pub fn manifest_witnesses(&self) -> &[ManifestProofWitnessEntry] {
        &self.module_entry.proof_witnesses
    }

    #[must_use]
    pub const fn module_entry(&self) -> &ModuleArtifactEntry {
        &self.module_entry
    }
}

/// Published witness reference after committed manifest reachability is proven.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofWitnessPublishedRef {
    reference: ProofWitnessRef,
    provenance: ProofWitnessProvenance,
    committed_main_artifact_hash: ArtifactHashRef,
}

impl ProofWitnessPublishedRef {
    #[must_use]
    pub const fn reference(&self) -> &ProofWitnessRef {
        &self.reference
    }

    #[must_use]
    pub const fn provenance(&self) -> &ProofWitnessProvenance {
        &self.provenance
    }

    #[must_use]
    pub const fn committed_main_artifact_hash(&self) -> &ArtifactHashRef {
        &self.committed_main_artifact_hash
    }
}

/// Witness-store failures and deferred downstream gaps.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofWitnessStoreError {
    /// The producer supplied an empty proof obligation id.
    EmptyObligationId,
    /// The payload schema family is empty.
    EmptyPayloadSchemaFamily,
    /// The payload schema family has malformed segments or characters.
    MalformedPayloadSchemaFamily { reason: &'static str },
    /// The producer identity is empty.
    EmptyProducerIdentity,
    /// The selected class cannot produce trusted witness material.
    UnsupportedWitnessClass { selected_class: ProofWinnerClass },
    /// The kernel policy input did not represent trusted accepted evidence.
    KernelEvidenceNotTrusted,
    /// The artifact metadata is not bound to the same accepted proof-obligation
    /// kernel evidence.
    AcceptedEvidenceHashMismatch { expected: Hash, actual: Hash },
    /// The selected status projection does not authorize this witness draft.
    StatusProjectionMismatch {
        field: &'static str,
        reason: &'static str,
    },
    /// The trusted used-axiom provenance disagrees with status projection.
    TrustedUsedAxiomsProjectionMismatch {
        expected: Option<Hash>,
        actual: Option<Hash>,
    },
    /// Current artifact schema cannot publish this trusted witness class.
    UnsupportedWitnessPublication {
        selected_class: ProofWinnerClass,
        gap: &'static str,
    },
    /// A hash reference used the wrong artifact hash class.
    HashClassMismatch {
        field: &'static str,
        expected: ArtifactHashClass,
        actual: ArtifactHashClass,
    },
    /// A path escaped or failed to live below `proof-witnesses/`.
    InvalidWitnessPath { reason: String },
    /// A schema that requires canonical bytes received invalid payload bytes.
    NonCanonicalPayloadBytes { reason: &'static str },
    /// The artifact proof token came from a stale build snapshot.
    StaleBuildSnapshot,
    /// No unpublished artifact reference candidate exists for the staged item.
    MissingArtifactReferenceCandidate { selected_class: ProofWinnerClass },
    /// The committed artifact did not contain the staged witness reference.
    MissingCommittedWitnessReference { obligation_id: String },
    /// The committed manifest entry did not match the staged witness tuple.
    MissingCommittedManifestReference { obligation_id: String },
    /// Manifest witness entries do not exactly cover artifact witnesses.
    ManifestCoverageMismatch { reason: String },
    /// The manifest has duplicate witness entries for one identity.
    DuplicateManifestWitnessReference { obligation_id: String },
    /// The verified artifact has duplicate witness references for one identity.
    DuplicateArtifactWitnessReference { obligation_id: String },
    /// Artifact `ProofWitnessRef` schema validation failed.
    ProofWitness(ProofWitnessError),
}

impl fmt::Display for ProofWitnessStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyObligationId => formatter.write_str("proof witness obligation id is empty"),
            Self::EmptyPayloadSchemaFamily => {
                formatter.write_str("proof witness payload schema family is empty")
            }
            Self::MalformedPayloadSchemaFamily { reason } => {
                write!(
                    formatter,
                    "malformed proof witness payload schema family: {reason}"
                )
            }
            Self::EmptyProducerIdentity => {
                formatter.write_str("proof witness producer identity is empty")
            }
            Self::UnsupportedWitnessClass { selected_class } => write!(
                formatter,
                "winner class `{selected_class:?}` cannot produce trusted proof witness material"
            ),
            Self::KernelEvidenceNotTrusted => {
                formatter.write_str("kernel evidence is not accepted trusted evidence")
            }
            Self::AcceptedEvidenceHashMismatch { expected, actual } => write!(
                formatter,
                "kernel acceptance metadata hash mismatch: expected `{}`, got `{}`",
                lower_hex(expected.as_bytes()),
                lower_hex(actual.as_bytes())
            ),
            Self::StatusProjectionMismatch { field, reason } => write!(
                formatter,
                "status projection field `{field}` does not authorize proof witness draft: {reason}"
            ),
            Self::TrustedUsedAxiomsProjectionMismatch { expected, actual } => write!(
                formatter,
                "trusted used_axioms provenance mismatch: expected `{}`, got `{}`",
                optional_hash_hex(*expected),
                optional_hash_hex(*actual)
            ),
            Self::UnsupportedWitnessPublication {
                selected_class,
                gap,
            } => write!(
                formatter,
                "winner class `{selected_class:?}` cannot publish a proof witness: {gap}"
            ),
            Self::HashClassMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "hash field `{field}` has class `{actual:?}`, expected `{expected:?}`"
            ),
            Self::InvalidWitnessPath { reason } => {
                write!(formatter, "invalid proof witness path: {reason}")
            }
            Self::NonCanonicalPayloadBytes { reason } => {
                write!(
                    formatter,
                    "non-canonical proof witness payload bytes: {reason}"
                )
            }
            Self::StaleBuildSnapshot => {
                formatter.write_str("committed witness proof belongs to a different build snapshot")
            }
            Self::MissingArtifactReferenceCandidate { selected_class } => write!(
                formatter,
                "staged winner class `{selected_class:?}` has no artifact reference candidate"
            ),
            Self::MissingCommittedWitnessReference { obligation_id } => write!(
                formatter,
                "committed VerifiedArtifact does not contain witness `{obligation_id}`"
            ),
            Self::MissingCommittedManifestReference { obligation_id } => write!(
                formatter,
                "committed manifest does not contain witness `{obligation_id}`"
            ),
            Self::ManifestCoverageMismatch { reason } => {
                write!(formatter, "manifest witness coverage mismatch: {reason}")
            }
            Self::DuplicateManifestWitnessReference { obligation_id } => write!(
                formatter,
                "duplicate manifest witness reference for obligation `{obligation_id}`"
            ),
            Self::DuplicateArtifactWitnessReference { obligation_id } => write!(
                formatter,
                "duplicate artifact witness reference for obligation `{obligation_id}`"
            ),
            Self::ProofWitness(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for ProofWitnessStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProofWitness(error) => Some(error),
            Self::EmptyObligationId
            | Self::EmptyPayloadSchemaFamily
            | Self::MalformedPayloadSchemaFamily { .. }
            | Self::EmptyProducerIdentity
            | Self::UnsupportedWitnessClass { .. }
            | Self::KernelEvidenceNotTrusted
            | Self::AcceptedEvidenceHashMismatch { .. }
            | Self::StatusProjectionMismatch { .. }
            | Self::TrustedUsedAxiomsProjectionMismatch { .. }
            | Self::UnsupportedWitnessPublication { .. }
            | Self::HashClassMismatch { .. }
            | Self::InvalidWitnessPath { .. }
            | Self::NonCanonicalPayloadBytes { .. }
            | Self::StaleBuildSnapshot
            | Self::MissingArtifactReferenceCandidate { .. }
            | Self::MissingCommittedWitnessReference { .. }
            | Self::MissingCommittedManifestReference { .. }
            | Self::ManifestCoverageMismatch { .. }
            | Self::DuplicateManifestWitnessReference { .. }
            | Self::DuplicateArtifactWitnessReference { .. } => None,
        }
    }
}

impl From<ProofWitnessError> for ProofWitnessStoreError {
    fn from(error: ProofWitnessError) -> Self {
        Self::ProofWitness(error)
    }
}

/// Stages a producer-owned witness draft before artifact commit.
pub fn stage(draft: ProofWitnessDraft) -> Result<ProofWitnessStagedRef, ProofWitnessStoreError> {
    validate_witness_path(&draft.witness_path)?;
    require_hash_class(
        &draft.obligation_fingerprint,
        ArtifactHashClass::Interface,
        "obligation_fingerprint",
    )?;
    validate_kernel_acceptance_metadata(&draft.kernel_acceptance)?;
    if draft.kernel_acceptance.accepted_result_hash.digest != draft.selected_evidence_hash {
        return Err(ProofWitnessStoreError::AcceptedEvidenceHashMismatch {
            expected: draft.selected_evidence_hash,
            actual: draft.kernel_acceptance.accepted_result_hash.digest,
        });
    }
    validate_draft_internal_consistency(&draft)?;
    let staged_payload_hash = witness_payload_hash_ref(&draft);
    validate_selected_proof_witness_hash(
        draft.expected_selected_proof_witness_hash,
        &staged_payload_hash,
    )?;
    let artifact_reference_candidate = match draft.selected_class {
        ProofWinnerClass::KernelVerified => {
            let reference = ProofWitnessRef {
                schema_version: current_witness_ref_version(),
                obligation_id: draft.obligation_id.clone(),
                obligation_fingerprint: draft.obligation_fingerprint.clone(),
                proof_status: ArtifactProofStatus::KernelVerified,
                evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
                witness_path: draft.witness_path.clone(),
                witness_artifact_hash: staged_payload_hash.clone(),
                kernel_acceptance: draft.kernel_acceptance.clone(),
            };
            write_proof_witness_ref(&reference)?;
            Some(reference)
        }
        ProofWinnerClass::DischargedBuiltin => None,
        ProofWinnerClass::PolicyPermittedExternal
        | ProofWinnerClass::PolicyAssumed
        | ProofWinnerClass::PolicyOpen
        | ProofWinnerClass::Rejected
        | ProofWinnerClass::NoSelectableEvidence => {
            return Err(ProofWitnessStoreError::UnsupportedWitnessClass {
                selected_class: draft.selected_class,
            });
        }
    };

    Ok(ProofWitnessStagedRef {
        obligation_id: draft.obligation_id,
        obligation_anchor: draft.obligation_anchor,
        obligation_fingerprint: draft.obligation_fingerprint,
        selected_class: draft.selected_class,
        selected_evidence_hash: draft.selected_evidence_hash,
        payload_schema: draft.payload_schema,
        witness_path: draft.witness_path,
        staged_payload_hash,
        artifact_reference_candidate,
        provenance: draft.provenance,
    })
}

/// Publishes a staged witness only after committed artifact reachability is
/// supplied by the artifact boundary.
pub fn publish_ref(
    staged: &ProofWitnessStagedRef,
    proof: &CommittedWitnessPublicationProof,
) -> Result<ProofWitnessPublishedRef, ProofWitnessStoreError> {
    if staged.selected_class == ProofWinnerClass::DischargedBuiltin {
        return Err(ProofWitnessStoreError::UnsupportedWitnessPublication {
            selected_class: staged.selected_class,
            gap: "WITNESS10-G001: mizar-artifact ProofWitnessRef 2.0 has no discharged_builtin witness status/evidence pair",
        });
    }
    if staged.provenance.build_snapshot_fingerprint() != proof.build_snapshot_fingerprint {
        return Err(ProofWitnessStoreError::StaleBuildSnapshot);
    }
    let Some(reference) = staged.artifact_reference_candidate.as_ref() else {
        return Err(ProofWitnessStoreError::MissingArtifactReferenceCandidate {
            selected_class: staged.selected_class,
        });
    };
    validate_exact_witness_coverage(
        &proof.verified_artifact_witnesses,
        &proof.module_entry.proof_witnesses,
    )?;
    if !proof
        .verified_artifact_witnesses
        .iter()
        .any(|item| item == reference)
    {
        return Err(ProofWitnessStoreError::MissingCommittedWitnessReference {
            obligation_id: staged.obligation_id.clone(),
        });
    }
    if !proof
        .module_entry
        .proof_witnesses
        .iter()
        .any(|entry| manifest_entry_matches_ref(entry, reference))
    {
        return Err(ProofWitnessStoreError::MissingCommittedManifestReference {
            obligation_id: staged.obligation_id.clone(),
        });
    }
    Ok(ProofWitnessPublishedRef {
        reference: reference.clone(),
        provenance: staged.provenance.clone(),
        committed_main_artifact_hash: proof.committed_main_artifact_hash.clone(),
    })
}

fn witness_payload_hash_ref(draft: &ProofWitnessDraft) -> ArtifactHashRef {
    witness_payload_hash_ref_from_inputs(
        &draft.obligation_fingerprint,
        draft.selected_evidence_hash(),
        &draft.payload_schema,
        draft.payload_bytes(),
        draft.provenance.verifier_policy_fingerprint(),
    )
}

fn witness_payload_hash_ref_from_inputs(
    obligation_fingerprint: &ArtifactHashRef,
    selected_evidence_hash: Hash,
    payload_schema: &ProofWitnessPayloadSchema,
    payload_bytes: &[u8],
    verifier_policy_fingerprint: Hash,
) -> ArtifactHashRef {
    let value = witness_payload_hash_json_from_inputs(
        obligation_fingerprint,
        selected_evidence_hash,
        payload_schema,
        payload_bytes,
        verifier_policy_fingerprint,
    );
    let domain = CanonicalHashDomain::new(
        HashClass::Artifact,
        payload_schema.family(),
        payload_schema.version(),
    );
    ArtifactHashRef::new(
        ArtifactHashClass::Artifact,
        payload_schema.family(),
        payload_schema.version(),
        domain.hash(&value, &[]),
    )
}

fn witness_payload_hash_json_from_inputs(
    obligation_fingerprint: &ArtifactHashRef,
    selected_evidence_hash: Hash,
    payload_schema: &ProofWitnessPayloadSchema,
    payload_bytes: &[u8],
    verifier_policy_fingerprint: Hash,
) -> CanonicalJson {
    let mut object = BTreeMap::new();
    object.insert(
        "hash_domain".to_owned(),
        CanonicalJson::String(WITNESS_PAYLOAD_HASH_DOMAIN.to_owned()),
    );
    object.insert(
        "obligation_fingerprint".to_owned(),
        CanonicalJson::String(obligation_fingerprint.to_artifact_hash_string()),
    );
    object.insert(
        "payload_bytes_hex".to_owned(),
        CanonicalJson::String(lower_hex(payload_bytes)),
    );
    object.insert(
        "payload_schema_family".to_owned(),
        CanonicalJson::String(payload_schema.family().to_owned()),
    );
    object.insert(
        "payload_schema_version".to_owned(),
        CanonicalJson::String(payload_schema.version().to_string()),
    );
    object.insert(
        "selected_evidence_hash".to_owned(),
        CanonicalJson::String(lower_hex(selected_evidence_hash.as_bytes())),
    );
    object.insert(
        "verifier_policy_fingerprint".to_owned(),
        CanonicalJson::String(lower_hex(verifier_policy_fingerprint.as_bytes())),
    );
    CanonicalJson::Object(object)
}

fn validate_selected_proof_witness_hash(
    expected: Option<Hash>,
    staged_payload_hash: &ArtifactHashRef,
) -> Result<(), ProofWitnessStoreError> {
    if let Some(expected) = expected
        && expected != staged_payload_hash.digest
    {
        return Err(status_projection_mismatch(
            "selected_proof_witness_hash",
            "projection witness hash must match staged payload hash",
        ));
    }
    Ok(())
}

fn validate_status_projection(
    projection: &ProofStatusProjection,
    obligation_id: &str,
    obligation_anchor: &ObligationAnchor,
    obligation_fingerprint: &ArtifactHashRef,
    trusted_kernel_metadata: &TrustedKernelWitnessMetadata,
    provenance: &ProofWitnessProvenance,
) -> Result<(), ProofWitnessStoreError> {
    let selected_class = trusted_kernel_metadata.selected_class();
    let selected_evidence_hash = trusted_kernel_metadata.accepted_evidence_hash();
    let reuse = projection.reuse_metadata();

    if projection.projected_status() != ProjectedProofStatus::Accepted {
        return Err(status_projection_mismatch(
            "projected_status",
            "trusted witness drafts require accepted status projection",
        ));
    }
    if projection.selected_class() != selected_class {
        return Err(status_projection_mismatch(
            "selected_class",
            "trusted kernel metadata class must match deterministic selection",
        ));
    }
    if projection.obligation_id() != obligation_id {
        return Err(status_projection_mismatch(
            "obligation_id",
            "draft obligation id must match status projection",
        ));
    }
    if reuse.obligation_anchor() != obligation_anchor {
        return Err(status_projection_mismatch(
            "obligation_anchor",
            "draft obligation anchor must match status projection",
        ));
    }
    if reuse.obligation_fingerprint() != obligation_fingerprint.digest {
        return Err(status_projection_mismatch(
            "obligation_fingerprint",
            "draft obligation fingerprint must match status projection",
        ));
    }
    if reuse.selected_evidence_hash() != Some(selected_evidence_hash) {
        return Err(status_projection_mismatch(
            "selected_evidence_hash",
            "trusted metadata evidence hash must match selected evidence",
        ));
    }
    if reuse.vc_fingerprint() != provenance.target_vc_fingerprint() {
        return Err(status_projection_mismatch(
            "target_vc_fingerprint",
            "provenance target VC fingerprint must match status projection",
        ));
    }
    if reuse.dependency_slice_fingerprint() != provenance.dependency_slice_fingerprint() {
        return Err(status_projection_mismatch(
            "dependency_slice_fingerprint",
            "provenance dependency slice fingerprint must match status projection",
        ));
    }
    if reuse.selected_candidate_id() != Some(provenance.selected_candidate_id()) {
        return Err(status_projection_mismatch(
            "selected_candidate_id",
            "provenance candidate id must match deterministic selection",
        ));
    }
    if trusted_kernel_metadata.kernel_evidence_origin() != provenance.kernel_evidence_origin() {
        return Err(status_projection_mismatch(
            "kernel_evidence_origin",
            "provenance origin must match trusted kernel evidence origin",
        ));
    }

    validate_policy_fingerprint(
        reuse.policy_fingerprint(),
        provenance.verifier_policy_fingerprint(),
        "provenance.verifier_policy_fingerprint",
    )?;
    validate_policy_fingerprint(
        reuse.policy_fingerprint(),
        trusted_kernel_metadata
            .kernel_acceptance()
            .verifier_policy_fingerprint
            .digest,
        "kernel_acceptance.verifier_policy_fingerprint",
    )?;

    let kernel_acceptance = trusted_kernel_metadata.kernel_acceptance();
    if kernel_acceptance.checker_schema_version != provenance.checker_schema_version() {
        return Err(status_projection_mismatch(
            "checker_schema_version",
            "provenance checker schema must match kernel acceptance metadata",
        ));
    }
    if kernel_acceptance.evidence_schema_version != provenance.evidence_schema_version() {
        return Err(status_projection_mismatch(
            "evidence_schema_version",
            "provenance evidence schema must match kernel acceptance metadata",
        ));
    }
    if let Some(accepted_result_hash) = provenance.accepted_result_hash()
        && accepted_result_hash != selected_evidence_hash
    {
        return Err(ProofWitnessStoreError::AcceptedEvidenceHashMismatch {
            expected: selected_evidence_hash,
            actual: accepted_result_hash,
        });
    }
    if let Some(reference) = provenance.trusted_used_axioms_ref()
        && reference.accepted_evidence_hash() != selected_evidence_hash
    {
        return Err(ProofWitnessStoreError::AcceptedEvidenceHashMismatch {
            expected: selected_evidence_hash,
            actual: reference.accepted_evidence_hash(),
        });
    }

    let expected_used_axioms = reuse.trusted_used_axioms_hash();
    let actual_used_axioms = provenance.trusted_used_axiom_ref_hash();
    if expected_used_axioms != actual_used_axioms {
        return Err(
            ProofWitnessStoreError::TrustedUsedAxiomsProjectionMismatch {
                expected: expected_used_axioms,
                actual: actual_used_axioms,
            },
        );
    }

    Ok(())
}

fn validate_policy_fingerprint(
    expected: PolicyFingerprint,
    actual: Hash,
    field: &'static str,
) -> Result<(), ProofWitnessStoreError> {
    if expected.hash() != actual {
        return Err(status_projection_mismatch(
            field,
            "policy fingerprint must match status projection",
        ));
    }
    Ok(())
}

fn validate_draft_internal_consistency(
    draft: &ProofWitnessDraft,
) -> Result<(), ProofWitnessStoreError> {
    if !class_allows_origin(
        draft.selected_class,
        draft.provenance.kernel_evidence_origin(),
    ) {
        return Err(status_projection_mismatch(
            "kernel_evidence_origin",
            "draft provenance origin must match selected trusted class",
        ));
    }
    if draft.kernel_acceptance.verifier_policy_fingerprint.digest
        != draft.provenance.verifier_policy_fingerprint()
    {
        return Err(status_projection_mismatch(
            "verifier_policy_fingerprint",
            "draft provenance policy fingerprint must match kernel acceptance metadata",
        ));
    }
    if draft.kernel_acceptance.checker_schema_version != draft.provenance.checker_schema_version() {
        return Err(status_projection_mismatch(
            "checker_schema_version",
            "draft provenance checker schema must match kernel acceptance metadata",
        ));
    }
    if draft.kernel_acceptance.evidence_schema_version != draft.provenance.evidence_schema_version()
    {
        return Err(status_projection_mismatch(
            "evidence_schema_version",
            "draft provenance evidence schema must match kernel acceptance metadata",
        ));
    }
    if let Some(accepted_result_hash) = draft.provenance.accepted_result_hash()
        && accepted_result_hash != draft.selected_evidence_hash
    {
        return Err(ProofWitnessStoreError::AcceptedEvidenceHashMismatch {
            expected: draft.selected_evidence_hash,
            actual: accepted_result_hash,
        });
    }
    if let Some(reference) = draft.provenance.trusted_used_axioms_ref()
        && reference.accepted_evidence_hash() != draft.selected_evidence_hash
    {
        return Err(ProofWitnessStoreError::AcceptedEvidenceHashMismatch {
            expected: draft.selected_evidence_hash,
            actual: reference.accepted_evidence_hash(),
        });
    }
    Ok(())
}

fn class_allows_origin(selected_class: ProofWinnerClass, origin: KernelEvidenceOrigin) -> bool {
    matches!(
        (selected_class, origin),
        (
            ProofWinnerClass::KernelVerified,
            KernelEvidenceOrigin::AtpFormulaSubstitution
        ) | (
            ProofWinnerClass::DischargedBuiltin,
            KernelEvidenceOrigin::BuiltinDischarge | KernelEvidenceOrigin::KernelPrimitive
        )
    )
}

fn status_projection_mismatch(field: &'static str, reason: &'static str) -> ProofWitnessStoreError {
    ProofWitnessStoreError::StatusProjectionMismatch { field, reason }
}

fn validate_exact_witness_coverage(
    artifact_witnesses: &[ProofWitnessRef],
    manifest_witnesses: &[ManifestProofWitnessEntry],
) -> Result<(), ProofWitnessStoreError> {
    let mut artifact_keys = BTreeSet::new();
    let mut artifact_obligation_ids = BTreeSet::new();
    for witness in artifact_witnesses {
        write_proof_witness_ref(witness)?;
        if !artifact_obligation_ids.insert(witness.obligation_id.clone()) {
            return Err(ProofWitnessStoreError::DuplicateArtifactWitnessReference {
                obligation_id: witness.obligation_id.clone(),
            });
        }
        let key = witness_key(witness);
        if !artifact_keys.insert(key) {
            return Err(ProofWitnessStoreError::DuplicateArtifactWitnessReference {
                obligation_id: witness.obligation_id.clone(),
            });
        }
    }

    let mut manifest_keys = BTreeSet::new();
    let mut manifest_obligation_ids = BTreeSet::new();
    for entry in manifest_witnesses {
        validate_manifest_witness_entry(entry)?;
        if !manifest_obligation_ids.insert(entry.obligation_id.clone()) {
            return Err(ProofWitnessStoreError::DuplicateManifestWitnessReference {
                obligation_id: entry.obligation_id.clone(),
            });
        }
        let key = manifest_key(entry);
        if !manifest_keys.insert(key) {
            return Err(ProofWitnessStoreError::DuplicateManifestWitnessReference {
                obligation_id: entry.obligation_id.clone(),
            });
        }
    }

    if artifact_keys != manifest_keys {
        return Err(ProofWitnessStoreError::ManifestCoverageMismatch {
            reason: "manifest proof_witnesses must exactly cover VerifiedArtifact.proof_witnesses"
                .to_owned(),
        });
    }
    Ok(())
}

fn validate_manifest_witness_entry(
    entry: &ManifestProofWitnessEntry,
) -> Result<(), ProofWitnessStoreError> {
    if entry.obligation_id.is_empty() {
        return Err(ProofWitnessStoreError::EmptyObligationId);
    }
    require_hash_class(
        &entry.obligation_fingerprint,
        ArtifactHashClass::Interface,
        "manifest_obligation_fingerprint",
    )?;
    require_hash_class(
        &entry.witness_artifact_hash,
        ArtifactHashClass::Artifact,
        "manifest_witness_artifact_hash",
    )?;
    validate_witness_path(&entry.witness_path)?;
    Ok(())
}

fn validate_kernel_acceptance_metadata(
    metadata: &KernelAcceptanceMetadata,
) -> Result<(), ProofWitnessStoreError> {
    require_hash_class(
        &metadata.kernel_profile_fingerprint,
        ArtifactHashClass::Interface,
        "kernel_acceptance.kernel_profile_fingerprint",
    )?;
    require_hash_class(
        &metadata.verifier_policy_fingerprint,
        ArtifactHashClass::Interface,
        "kernel_acceptance.verifier_policy_fingerprint",
    )?;
    require_hash_class(
        &metadata.target_binding_hash,
        ArtifactHashClass::Interface,
        "kernel_acceptance.target_binding_hash",
    )?;
    require_hash_class(
        &metadata.formula_evidence_hash,
        ArtifactHashClass::Interface,
        "kernel_acceptance.formula_evidence_hash",
    )?;
    require_hash_class(
        &metadata.substitution_evidence_hash,
        ArtifactHashClass::Interface,
        "kernel_acceptance.substitution_evidence_hash",
    )?;
    require_hash_class(
        &metadata.provenance_hash,
        ArtifactHashClass::Interface,
        "kernel_acceptance.provenance_hash",
    )?;
    if let Some(formula_context_hash) = &metadata.formula_context_hash {
        require_hash_class(
            formula_context_hash,
            ArtifactHashClass::Interface,
            "kernel_acceptance.formula_context_hash",
        )?;
    }
    require_hash_class(
        &metadata.accepted_result_hash,
        ArtifactHashClass::Interface,
        "kernel_acceptance.accepted_result_hash",
    )?;
    Ok(())
}

fn manifest_entry_matches_ref(
    entry: &ManifestProofWitnessEntry,
    reference: &ProofWitnessRef,
) -> bool {
    entry.obligation_id == reference.obligation_id
        && entry.obligation_fingerprint == reference.obligation_fingerprint
        && entry.witness_path == reference.witness_path
        && entry.witness_artifact_hash == reference.witness_artifact_hash
}

fn witness_key(reference: &ProofWitnessRef) -> WitnessKey {
    WitnessKey {
        obligation_id: reference.obligation_id.clone(),
        obligation_fingerprint: reference.obligation_fingerprint.to_artifact_hash_string(),
        witness_path: reference.witness_path.clone(),
        witness_artifact_hash: reference.witness_artifact_hash.to_artifact_hash_string(),
    }
}

fn manifest_key(entry: &ManifestProofWitnessEntry) -> WitnessKey {
    WitnessKey {
        obligation_id: entry.obligation_id.clone(),
        obligation_fingerprint: entry.obligation_fingerprint.to_artifact_hash_string(),
        witness_path: entry.witness_path.clone(),
        witness_artifact_hash: entry.witness_artifact_hash.to_artifact_hash_string(),
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct WitnessKey {
    obligation_id: String,
    obligation_fingerprint: String,
    witness_path: String,
    witness_artifact_hash: String,
}

fn require_hash_class(
    hash: &ArtifactHashRef,
    expected: ArtifactHashClass,
    field: &'static str,
) -> Result<(), ProofWitnessStoreError> {
    if hash.class != expected {
        return Err(ProofWitnessStoreError::HashClassMismatch {
            field,
            expected,
            actual: hash.class,
        });
    }
    Ok(())
}

fn validate_payload_schema_family(family: &str) -> Result<(), ProofWitnessStoreError> {
    if family.is_empty() {
        return Err(ProofWitnessStoreError::EmptyPayloadSchemaFamily);
    }
    for segment in family.split('/') {
        if segment.is_empty() {
            return Err(ProofWitnessStoreError::MalformedPayloadSchemaFamily {
                reason: "schema family segments must not be empty",
            });
        }
        if !segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(ProofWitnessStoreError::MalformedPayloadSchemaFamily {
                reason: "schema family contains invalid characters",
            });
        }
    }
    Ok(())
}

fn validate_witness_path(path: &str) -> Result<(), ProofWitnessStoreError> {
    if !path.starts_with("proof-witnesses/") {
        return Err(ProofWitnessStoreError::InvalidWitnessPath {
            reason: "path must start with proof-witnesses/".to_owned(),
        });
    }
    let child = &path["proof-witnesses/".len()..];
    if child.is_empty() {
        return Err(ProofWitnessStoreError::InvalidWitnessPath {
            reason: "path must contain a child segment after proof-witnesses/".to_owned(),
        });
    }
    if path.starts_with('/') || path.contains('\\') {
        return Err(ProofWitnessStoreError::InvalidWitnessPath {
            reason: "path must be artifact-root relative and use / separators".to_owned(),
        });
    }
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." || segment.contains(':') {
            return Err(ProofWitnessStoreError::InvalidWitnessPath {
                reason: "path must not contain empty, ., .., or drive-prefix segments".to_owned(),
            });
        }
    }
    Ok(())
}

fn lower_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("writing to string cannot fail");
    }
    encoded
}

fn optional_hash_hex(hash: Option<Hash>) -> String {
    hash.map(|hash| lower_hex(hash.as_bytes()))
        .unwrap_or_else(|| "<none>".to_owned())
}

#[cfg(test)]
mod tests;
