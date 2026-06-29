//! Artifact- and diagnostics-facing proof status projection.
//!
//! This module projects already-selected proof outcomes. It does not run ATP
//! backends, call the kernel, solve SAT problems, query caches, stage
//! witnesses, write artifact manifests, or accept proofs.

use std::{error::Error, fmt};

use mizar_artifact::store::SchemaVersion;
use mizar_kernel::checker::{KernelCheckResult, KernelCheckStatus, UsedAxiom};
use mizar_session::Hash;
use mizar_vc::vc_ir::VcId;

use crate::{
    policy::{
        ExternalEvidencePublicationStatus, KernelEvidenceOrigin, KernelPolicyInput,
        OpenObligationMode, PolicyFingerprint, VerifierPolicy,
    },
    selection::{
        ArtifactProofSelection, CandidateSourceId, DiagnosticRef, ProofWinnerClass,
        ProofWitnessPublication,
    },
};

const USED_AXIOMS_HASH_DOMAIN: &str = "mizar-proof-trusted-used-axioms-v1";
const PROOF_REUSE_VALIDATION_HASH_DOMAIN: &str = "mizar-proof-reuse-validation-v1";

/// Stable source identity for proof-reuse candidates across edits.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
pub struct ObligationAnchor(String);

impl ObligationAnchor {
    /// Creates a non-empty obligation anchor.
    pub fn new(anchor: impl Into<String>) -> Result<Self, StatusProjectionError> {
        let anchor = anchor.into();
        if anchor.is_empty() {
            return Err(StatusProjectionError::EmptyObligationAnchor);
        }
        Ok(Self(anchor))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObligationAnchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Stable proof-obligation identity supplied by the VC/artifact producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofObligationIdentity {
    obligation_id: String,
    obligation_anchor: ObligationAnchor,
    obligation_fingerprint: Hash,
    vc_fingerprint: Hash,
    local_context_fingerprint: Hash,
    dependency_slice_fingerprint: Hash,
}

impl ProofObligationIdentity {
    pub fn new(
        obligation_id: impl Into<String>,
        obligation_anchor: ObligationAnchor,
        obligation_fingerprint: Hash,
        vc_fingerprint: Hash,
        local_context_fingerprint: Hash,
        dependency_slice_fingerprint: Hash,
    ) -> Result<Self, StatusProjectionError> {
        let obligation_id = obligation_id.into();
        if obligation_id.is_empty() {
            return Err(StatusProjectionError::EmptyObligationId);
        }
        Ok(Self {
            obligation_id,
            obligation_anchor,
            obligation_fingerprint,
            vc_fingerprint,
            local_context_fingerprint,
            dependency_slice_fingerprint,
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
    pub const fn obligation_fingerprint(&self) -> Hash {
        self.obligation_fingerprint
    }

    #[must_use]
    pub const fn vc_fingerprint(&self) -> Hash {
        self.vc_fingerprint
    }

    #[must_use]
    pub const fn local_context_fingerprint(&self) -> Hash {
        self.local_context_fingerprint
    }

    #[must_use]
    pub const fn dependency_slice_fingerprint(&self) -> Hash {
        self.dependency_slice_fingerprint
    }
}

/// Stable diagnostics-owned explanation reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq, std::hash::Hash)]
pub struct ExplanationRef(Hash);

impl ExplanationRef {
    #[must_use]
    pub const fn new(hash: Hash) -> Self {
        Self(hash)
    }

    #[must_use]
    pub const fn hash(self) -> Hash {
        self.0
    }
}

/// Trusted used-axiom reference derived from an accepted kernel result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedUsedAxiomsRef {
    accepted_evidence_hash: Hash,
    used_axioms_hash: Hash,
    used_axiom_count: usize,
}

impl TrustedUsedAxiomsRef {
    pub fn from_kernel_result(
        result: &KernelCheckResult,
        origin: KernelEvidenceOrigin,
    ) -> Result<Self, TrustedUsedAxiomsError> {
        trusted_used_axioms_from_kernel_policy_input(
            &KernelPolicyInput::from_kernel_result(result, origin),
            result.used_axioms(),
        )
    }

    #[must_use]
    pub const fn accepted_evidence_hash(&self) -> Hash {
        self.accepted_evidence_hash
    }

    #[must_use]
    pub const fn used_axioms_hash(&self) -> Hash {
        self.used_axioms_hash
    }

    #[must_use]
    pub const fn used_axiom_count(&self) -> usize {
        self.used_axiom_count
    }

    #[cfg(test)]
    pub(crate) const fn for_test(
        accepted_evidence_hash: Hash,
        used_axioms_hash: Hash,
        used_axiom_count: usize,
    ) -> Self {
        Self {
            accepted_evidence_hash,
            used_axioms_hash,
            used_axiom_count,
        }
    }
}

/// Error while deriving trusted used-axiom metadata from kernel output.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TrustedUsedAxiomsError {
    KernelResultNotAccepted,
    KernelResultPolicyTainted,
    MissingAcceptedEvidenceHash,
}

impl fmt::Display for TrustedUsedAxiomsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KernelResultNotAccepted => f.write_str("kernel result is not accepted"),
            Self::KernelResultPolicyTainted => f.write_str("kernel result is policy-tainted"),
            Self::MissingAcceptedEvidenceHash => {
                f.write_str("accepted kernel evidence hash is unavailable")
            }
        }
    }
}

impl Error for TrustedUsedAxiomsError {}

fn trusted_used_axioms_from_kernel_policy_input(
    input: &KernelPolicyInput,
    used_axioms: &[UsedAxiom],
) -> Result<TrustedUsedAxiomsRef, TrustedUsedAxiomsError> {
    if input.status() != KernelCheckStatus::Accepted {
        return Err(TrustedUsedAxiomsError::KernelResultNotAccepted);
    }
    if input.policy_taint() {
        return Err(TrustedUsedAxiomsError::KernelResultPolicyTainted);
    }

    let accepted_evidence_hash = input
        .accepted_evidence_hash()
        .ok_or(TrustedUsedAxiomsError::MissingAcceptedEvidenceHash)?;

    Ok(TrustedUsedAxiomsRef {
        accepted_evidence_hash,
        used_axioms_hash: hash_used_axioms(used_axioms),
        used_axiom_count: used_axioms.len(),
    })
}

/// Input to proof status projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofStatusProjectionInput {
    selection: ArtifactProofSelection,
    policy: VerifierPolicy,
    identity: ProofObligationIdentity,
    dependency_compatibility: Option<ProofReuseDependencyCompatibility>,
    trusted_used_axioms: Option<TrustedUsedAxiomsRef>,
    explanation_ref: Option<ExplanationRef>,
}

impl ProofStatusProjectionInput {
    #[must_use]
    pub const fn new(
        selection: ArtifactProofSelection,
        policy: VerifierPolicy,
        identity: ProofObligationIdentity,
    ) -> Self {
        Self {
            selection,
            policy,
            identity,
            dependency_compatibility: None,
            trusted_used_axioms: None,
            explanation_ref: None,
        }
    }

    #[must_use]
    pub fn with_dependency_compatibility(
        mut self,
        compatibility: ProofReuseDependencyCompatibility,
    ) -> Self {
        self.dependency_compatibility = Some(compatibility);
        self
    }

    #[must_use]
    pub fn with_trusted_used_axioms(mut self, trusted: TrustedUsedAxiomsRef) -> Self {
        self.trusted_used_axioms = Some(trusted);
        self
    }

    #[must_use]
    pub const fn with_explanation_ref(mut self, explanation_ref: ExplanationRef) -> Self {
        self.explanation_ref = Some(explanation_ref);
        self
    }
}

/// Internal projected proof status before final artifact publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ProjectedProofStatus {
    Accepted,
    ExternallyAttested,
    PolicyAssumed,
    Open,
    Rejected,
    NotRequired,
}

impl ProjectedProofStatus {
    #[must_use]
    pub const fn is_trusted(self) -> bool {
        matches!(self, Self::Accepted)
    }
}

/// Obligation status names currently representable by `mizar-artifact`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum CurrentArtifactObligationStatus {
    Accepted,
    Open,
    Rejected,
    ExternallyAttested,
    NotRequired,
}

/// Why a projected status cannot be published into the current artifact schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ArtifactPublicationGap {
    MissingKernelVerifiedWitness,
    DischargedBuiltinWitnessUnsupported,
    PolicyAssumedStatusUnsupported,
}

/// Publication availability for the current artifact schema.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, std::hash::Hash)]
#[non_exhaustive]
pub enum ArtifactStatusPublication {
    Publishable(CurrentArtifactObligationStatus),
    ExternalDependencyGap(ArtifactPublicationGap),
}

impl ArtifactStatusPublication {
    #[must_use]
    pub const fn status(self) -> Option<CurrentArtifactObligationStatus> {
        match self {
            Self::Publishable(status) => Some(status),
            Self::ExternalDependencyGap(_) => None,
        }
    }

    #[must_use]
    pub const fn gap(self) -> Option<ArtifactPublicationGap> {
        match self {
            Self::Publishable(_) => None,
            Self::ExternalDependencyGap(gap) => Some(gap),
        }
    }
}

/// Dependency artifact and schema compatibility metadata for proof reuse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProofReuseDependencyCompatibility {
    dependency_artifact_fingerprint: Hash,
    dependency_schema_version: SchemaVersion,
    proof_reuse_schema_version: SchemaVersion,
}

impl ProofReuseDependencyCompatibility {
    #[must_use]
    pub const fn new(
        dependency_artifact_fingerprint: Hash,
        dependency_schema_version: SchemaVersion,
        proof_reuse_schema_version: SchemaVersion,
    ) -> Self {
        Self {
            dependency_artifact_fingerprint,
            dependency_schema_version,
            proof_reuse_schema_version,
        }
    }

    #[must_use]
    pub const fn dependency_artifact_fingerprint(self) -> Hash {
        self.dependency_artifact_fingerprint
    }

    #[must_use]
    pub const fn dependency_schema_version(self) -> SchemaVersion {
        self.dependency_schema_version
    }

    #[must_use]
    pub const fn proof_reuse_schema_version(self) -> SchemaVersion {
        self.proof_reuse_schema_version
    }
}

/// Stable identity for the proof evidence selected for possible reuse.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofEvidenceReuseIdentity {
    selected_candidate_id: Option<CandidateSourceId>,
    selected_candidate_provenance_hash: Option<Hash>,
    selected_evidence_hash: Option<Hash>,
    selected_proof_witness_hash: Option<Hash>,
    deterministic_discharge_hash: Option<Hash>,
    tie_break_key_hash: Hash,
    selection_reason: &'static str,
}

impl ProofEvidenceReuseIdentity {
    #[must_use]
    pub fn selected_candidate_id(&self) -> Option<&CandidateSourceId> {
        self.selected_candidate_id.as_ref()
    }

    #[must_use]
    pub const fn selected_candidate_provenance_hash(&self) -> Option<Hash> {
        self.selected_candidate_provenance_hash
    }

    #[must_use]
    pub const fn selected_evidence_hash(&self) -> Option<Hash> {
        self.selected_evidence_hash
    }

    #[must_use]
    pub const fn selected_proof_witness_hash(&self) -> Option<Hash> {
        self.selected_proof_witness_hash
    }

    #[must_use]
    pub const fn deterministic_discharge_hash(&self) -> Option<Hash> {
        self.deterministic_discharge_hash
    }

    #[must_use]
    pub const fn tie_break_key_hash(&self) -> Hash {
        self.tie_break_key_hash
    }

    #[must_use]
    pub const fn selection_reason(&self) -> &'static str {
        self.selection_reason
    }
}

/// Stable proof-reuse metadata emitted by status projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatusReuseMetadata {
    selected_class: ProofWinnerClass,
    projected_status: ProjectedProofStatus,
    selected_candidate_id: Option<CandidateSourceId>,
    obligation_anchor: ObligationAnchor,
    obligation_fingerprint: Hash,
    vc_fingerprint: Hash,
    local_context_fingerprint: Hash,
    dependency_slice_fingerprint: Hash,
    policy_fingerprint: PolicyFingerprint,
    selected_evidence_hash: Option<Hash>,
    selected_proof_witness_hash: Option<Hash>,
    deterministic_discharge_hash: Option<Hash>,
    trusted_used_axioms_hash: Option<Hash>,
    external_admission_status: Option<ExternalEvidencePublicationStatus>,
    selected_candidate_provenance_hash: Option<Hash>,
    selection_reason: &'static str,
    tie_break_key_hash: Hash,
    dependency_compatibility: Option<ProofReuseDependencyCompatibility>,
    explanation_ref: Option<ExplanationRef>,
    diagnostic_result_id: Option<Hash>,
    proof_reuse_validation_hash: Hash,
}

impl StatusReuseMetadata {
    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.selected_class
    }

    #[must_use]
    pub const fn projected_status(&self) -> ProjectedProofStatus {
        self.projected_status
    }

    #[must_use]
    pub fn selected_candidate_id(&self) -> Option<&CandidateSourceId> {
        self.selected_candidate_id.as_ref()
    }

    #[must_use]
    pub const fn obligation_anchor(&self) -> &ObligationAnchor {
        &self.obligation_anchor
    }

    #[must_use]
    pub const fn obligation_fingerprint(&self) -> Hash {
        self.obligation_fingerprint
    }

    #[must_use]
    pub const fn vc_fingerprint(&self) -> Hash {
        self.vc_fingerprint
    }

    #[must_use]
    pub const fn local_context_fingerprint(&self) -> Hash {
        self.local_context_fingerprint
    }

    #[must_use]
    pub const fn dependency_slice_fingerprint(&self) -> Hash {
        self.dependency_slice_fingerprint
    }

    #[must_use]
    pub const fn policy_fingerprint(&self) -> PolicyFingerprint {
        self.policy_fingerprint
    }

    #[must_use]
    pub const fn selected_evidence_hash(&self) -> Option<Hash> {
        self.selected_evidence_hash
    }

    #[must_use]
    pub const fn selected_proof_witness_hash(&self) -> Option<Hash> {
        self.selected_proof_witness_hash
    }

    #[must_use]
    pub const fn deterministic_discharge_hash(&self) -> Option<Hash> {
        self.deterministic_discharge_hash
    }

    #[must_use]
    pub const fn trusted_used_axioms_hash(&self) -> Option<Hash> {
        self.trusted_used_axioms_hash
    }

    #[must_use]
    pub const fn external_admission_status(&self) -> Option<ExternalEvidencePublicationStatus> {
        self.external_admission_status
    }

    #[must_use]
    pub const fn selected_candidate_provenance_hash(&self) -> Option<Hash> {
        self.selected_candidate_provenance_hash
    }

    #[must_use]
    pub const fn selection_reason(&self) -> &'static str {
        self.selection_reason
    }

    #[must_use]
    pub const fn tie_break_key_hash(&self) -> Hash {
        self.tie_break_key_hash
    }

    #[must_use]
    pub const fn dependency_compatibility(&self) -> Option<ProofReuseDependencyCompatibility> {
        self.dependency_compatibility
    }

    #[must_use]
    pub const fn explanation_ref(&self) -> Option<ExplanationRef> {
        self.explanation_ref
    }

    #[must_use]
    pub const fn diagnostic_result_id(&self) -> Option<Hash> {
        self.diagnostic_result_id
    }

    #[must_use]
    pub fn proof_evidence_identity(&self) -> ProofEvidenceReuseIdentity {
        ProofEvidenceReuseIdentity {
            selected_candidate_id: self.selected_candidate_id.clone(),
            selected_candidate_provenance_hash: self.selected_candidate_provenance_hash,
            selected_evidence_hash: self.selected_evidence_hash,
            selected_proof_witness_hash: self.selected_proof_witness_hash,
            deterministic_discharge_hash: self.deterministic_discharge_hash,
            tie_break_key_hash: self.tie_break_key_hash,
            selection_reason: self.selection_reason,
        }
    }

    #[must_use]
    pub const fn proof_reuse_validation_hash(&self) -> Hash {
        self.proof_reuse_validation_hash
    }

    #[must_use]
    pub const fn cache_reuse_predicate_complete(&self) -> bool {
        if self.dependency_compatibility.is_none() {
            return false;
        }

        match self.selected_class {
            ProofWinnerClass::KernelVerified => self.selected_proof_witness_hash.is_some(),
            ProofWinnerClass::DischargedBuiltin => self.deterministic_discharge_hash.is_some(),
            ProofWinnerClass::PolicyPermittedExternal
            | ProofWinnerClass::PolicyAssumed
            | ProofWinnerClass::PolicyOpen
            | ProofWinnerClass::Rejected
            | ProofWinnerClass::NoSelectableEvidence => false,
        }
    }
}

/// Final status projection result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofStatusProjection {
    vc: VcId,
    obligation_id: String,
    selected_class: ProofWinnerClass,
    projected_status: ProjectedProofStatus,
    artifact_publication: ArtifactStatusPublication,
    accepted_witness_obligation_id: Option<String>,
    trusted_used_axioms: Option<TrustedUsedAxiomsRef>,
    diagnostic_refs: Vec<DiagnosticRef>,
    explanation_ref: Option<ExplanationRef>,
    reuse_metadata: StatusReuseMetadata,
}

impl ProofStatusProjection {
    #[must_use]
    pub const fn vc(&self) -> VcId {
        self.vc
    }

    #[must_use]
    pub fn obligation_id(&self) -> &str {
        &self.obligation_id
    }

    #[must_use]
    pub const fn selected_class(&self) -> ProofWinnerClass {
        self.selected_class
    }

    #[must_use]
    pub const fn projected_status(&self) -> ProjectedProofStatus {
        self.projected_status
    }

    #[must_use]
    pub const fn artifact_publication(&self) -> ArtifactStatusPublication {
        self.artifact_publication
    }

    #[must_use]
    pub fn accepted_witness_obligation_id(&self) -> Option<&str> {
        self.accepted_witness_obligation_id.as_deref()
    }

    #[must_use]
    pub const fn trusted_used_axioms(&self) -> Option<&TrustedUsedAxiomsRef> {
        self.trusted_used_axioms.as_ref()
    }

    #[must_use]
    pub fn diagnostic_refs(&self) -> &[DiagnosticRef] {
        &self.diagnostic_refs
    }

    #[must_use]
    pub const fn explanation_ref(&self) -> Option<ExplanationRef> {
        self.explanation_ref
    }

    #[must_use]
    pub const fn reuse_metadata(&self) -> &StatusReuseMetadata {
        &self.reuse_metadata
    }
}

/// Projection error.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StatusProjectionError {
    EmptyObligationId,
    EmptyObligationAnchor,
    TrustedUsedAxiomsForNonTrustedStatus {
        selected_class: ProofWinnerClass,
    },
    TrustedUsedAxiomsUnavailable {
        selected_class: ProofWinnerClass,
    },
    TrustedUsedAxiomsEvidenceMismatch {
        selected_evidence_hash: Option<Hash>,
        accepted_evidence_hash: Hash,
    },
    PolicyFingerprintMismatch {
        selected_policy_fingerprint: PolicyFingerprint,
        active_policy_fingerprint: PolicyFingerprint,
    },
}

impl fmt::Display for StatusProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyObligationId => f.write_str("obligation id must be non-empty"),
            Self::EmptyObligationAnchor => f.write_str("obligation anchor must be non-empty"),
            Self::TrustedUsedAxiomsForNonTrustedStatus { selected_class } => write!(
                f,
                "trusted used_axioms cannot be attached to non-trusted class {selected_class:?}"
            ),
            Self::TrustedUsedAxiomsUnavailable { selected_class } => write!(
                f,
                "selected class {selected_class:?} did not report trusted used_axioms availability"
            ),
            Self::TrustedUsedAxiomsEvidenceMismatch {
                selected_evidence_hash,
                accepted_evidence_hash,
            } => write!(
                f,
                "trusted used_axioms evidence hash {accepted_evidence_hash:?} does not match selected evidence {selected_evidence_hash:?}"
            ),
            Self::PolicyFingerprintMismatch {
                selected_policy_fingerprint,
                active_policy_fingerprint,
            } => write!(
                f,
                "selected policy fingerprint {selected_policy_fingerprint:?} does not match active policy fingerprint {active_policy_fingerprint:?}"
            ),
        }
    }
}

impl Error for StatusProjectionError {}

/// Projects one selected proof outcome into status metadata.
pub fn project_status(
    input: ProofStatusProjectionInput,
) -> Result<ProofStatusProjection, StatusProjectionError> {
    let selected_policy_fingerprint = input
        .selection
        .selection()
        .reuse_metadata()
        .policy_fingerprint();
    let active_policy_fingerprint = input.policy.policy_fingerprint();
    if selected_policy_fingerprint != active_policy_fingerprint {
        return Err(StatusProjectionError::PolicyFingerprintMismatch {
            selected_policy_fingerprint,
            active_policy_fingerprint,
        });
    }

    let selected_class = input.selection.selected_class();
    let projected_status = projected_status(selected_class, &input.policy);
    let artifact_publication = artifact_publication(&input.selection, projected_status);
    let trusted_used_axioms =
        validated_trusted_used_axioms(&input.selection, selected_class, input.trusted_used_axioms)?;
    let accepted_witness_obligation_id = matches!(
        artifact_publication,
        ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Accepted)
    )
    .then(|| input.identity.obligation_id.clone());
    let selection_metadata = input.selection.selection().reuse_metadata();
    let trusted_used_axioms_hash = trusted_used_axioms
        .as_ref()
        .map(TrustedUsedAxiomsRef::used_axioms_hash);

    let mut reuse_metadata = StatusReuseMetadata {
        selected_class,
        projected_status,
        selected_candidate_id: input.selection.selection().selected_candidate_id().cloned(),
        obligation_anchor: input.identity.obligation_anchor.clone(),
        obligation_fingerprint: input.identity.obligation_fingerprint,
        vc_fingerprint: input.identity.vc_fingerprint,
        local_context_fingerprint: input.identity.local_context_fingerprint,
        dependency_slice_fingerprint: input.identity.dependency_slice_fingerprint,
        policy_fingerprint: input.policy.policy_fingerprint(),
        selected_evidence_hash: selection_metadata.selected_evidence_hash(),
        selected_proof_witness_hash: selection_metadata.selected_proof_witness_hash(),
        deterministic_discharge_hash: selection_metadata.deterministic_discharge_hash(),
        trusted_used_axioms_hash,
        external_admission_status: selection_metadata.external_admission_status(),
        selected_candidate_provenance_hash: selection_metadata.selected_candidate_provenance_hash(),
        selection_reason: selection_metadata.selection_reason(),
        tie_break_key_hash: selection_metadata.tie_break_key_hash(),
        dependency_compatibility: input.dependency_compatibility,
        explanation_ref: input.explanation_ref,
        diagnostic_result_id: input.selection.selection().diagnostic_result_id(),
        proof_reuse_validation_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
    };
    reuse_metadata.proof_reuse_validation_hash = hash_status_reuse_metadata(&reuse_metadata);

    Ok(ProofStatusProjection {
        vc: input.selection.vc(),
        obligation_id: input.identity.obligation_id,
        selected_class,
        projected_status,
        artifact_publication,
        accepted_witness_obligation_id,
        trusted_used_axioms,
        diagnostic_refs: input
            .selection
            .selection()
            .ordered_diagnostic_refs()
            .to_vec(),
        explanation_ref: input.explanation_ref,
        reuse_metadata,
    })
}

fn projected_status(
    selected_class: ProofWinnerClass,
    policy: &VerifierPolicy,
) -> ProjectedProofStatus {
    match selected_class {
        ProofWinnerClass::KernelVerified | ProofWinnerClass::DischargedBuiltin => {
            ProjectedProofStatus::Accepted
        }
        ProofWinnerClass::PolicyPermittedExternal => ProjectedProofStatus::ExternallyAttested,
        ProofWinnerClass::PolicyAssumed => ProjectedProofStatus::PolicyAssumed,
        ProofWinnerClass::PolicyOpen => ProjectedProofStatus::Open,
        ProofWinnerClass::Rejected => ProjectedProofStatus::Rejected,
        ProofWinnerClass::NoSelectableEvidence => match policy.open_obligation() {
            OpenObligationMode::AllowPolicyOpen => ProjectedProofStatus::Open,
            OpenObligationMode::Reject | OpenObligationMode::RecordDiagnostic => {
                ProjectedProofStatus::Rejected
            }
        },
    }
}

fn artifact_publication(
    selection: &ArtifactProofSelection,
    projected_status: ProjectedProofStatus,
) -> ArtifactStatusPublication {
    match selection.selected_class() {
        ProofWinnerClass::KernelVerified => {
            let metadata = selection.selection().reuse_metadata();
            if metadata.proof_witness_publication() == ProofWitnessPublication::Available
                && metadata.selected_proof_witness_hash().is_some()
            {
                ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Accepted)
            } else {
                ArtifactStatusPublication::ExternalDependencyGap(
                    ArtifactPublicationGap::MissingKernelVerifiedWitness,
                )
            }
        }
        ProofWinnerClass::DischargedBuiltin => ArtifactStatusPublication::ExternalDependencyGap(
            ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported,
        ),
        ProofWinnerClass::PolicyAssumed => ArtifactStatusPublication::ExternalDependencyGap(
            ArtifactPublicationGap::PolicyAssumedStatusUnsupported,
        ),
        ProofWinnerClass::PolicyPermittedExternal => ArtifactStatusPublication::Publishable(
            CurrentArtifactObligationStatus::ExternallyAttested,
        ),
        ProofWinnerClass::PolicyOpen => {
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Open)
        }
        ProofWinnerClass::Rejected => {
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
        }
        ProofWinnerClass::NoSelectableEvidence => match projected_status {
            ProjectedProofStatus::Open => {
                ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Open)
            }
            ProjectedProofStatus::Rejected => {
                ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
            }
            ProjectedProofStatus::Accepted
            | ProjectedProofStatus::ExternallyAttested
            | ProjectedProofStatus::PolicyAssumed
            | ProjectedProofStatus::NotRequired => {
                unreachable!("no-selectable evidence projects only to open or rejected statuses")
            }
        },
    }
}

fn validated_trusted_used_axioms(
    selection: &ArtifactProofSelection,
    selected_class: ProofWinnerClass,
    trusted_used_axioms: Option<TrustedUsedAxiomsRef>,
) -> Result<Option<TrustedUsedAxiomsRef>, StatusProjectionError> {
    let Some(trusted_used_axioms) = trusted_used_axioms else {
        return Ok(None);
    };

    if !selected_class.is_trusted() {
        return Err(StatusProjectionError::TrustedUsedAxiomsForNonTrustedStatus { selected_class });
    }
    if !selection.selection().trusted_used_axioms_available() {
        return Err(StatusProjectionError::TrustedUsedAxiomsUnavailable { selected_class });
    }

    let selected_evidence_hash = selection
        .selection()
        .reuse_metadata()
        .selected_evidence_hash();
    if selected_evidence_hash != Some(trusted_used_axioms.accepted_evidence_hash()) {
        return Err(StatusProjectionError::TrustedUsedAxiomsEvidenceMismatch {
            selected_evidence_hash,
            accepted_evidence_hash: trusted_used_axioms.accepted_evidence_hash(),
        });
    }

    Ok(Some(trusted_used_axioms))
}

fn hash_status_reuse_metadata(metadata: &StatusReuseMetadata) -> Hash {
    let mut hash = StableHasher::new(PROOF_REUSE_VALIDATION_HASH_DOMAIN);
    hash.field_str(
        "selected_class",
        proof_winner_class_key(metadata.selected_class),
    );
    hash.field_str(
        "projected_status",
        projected_proof_status_key(metadata.projected_status),
    );
    hash.field_optional_str(
        "selected_candidate_id",
        metadata
            .selected_candidate_id
            .as_ref()
            .map(CandidateSourceId::as_str),
    );
    hash.field_str("obligation_anchor", metadata.obligation_anchor.as_str());
    hash.field_hash("obligation_fingerprint", metadata.obligation_fingerprint);
    hash.field_hash("vc_fingerprint", metadata.vc_fingerprint);
    hash.field_hash(
        "local_context_fingerprint",
        metadata.local_context_fingerprint,
    );
    hash.field_hash(
        "dependency_slice_fingerprint",
        metadata.dependency_slice_fingerprint,
    );
    hash.field_hash("policy_fingerprint", metadata.policy_fingerprint.hash());
    hash.field_optional_hash("selected_evidence_hash", metadata.selected_evidence_hash);
    hash.field_optional_hash(
        "selected_proof_witness_hash",
        metadata.selected_proof_witness_hash,
    );
    hash.field_optional_hash(
        "deterministic_discharge_hash",
        metadata.deterministic_discharge_hash,
    );
    hash.field_optional_hash(
        "trusted_used_axioms_hash",
        metadata.trusted_used_axioms_hash,
    );
    hash.field_optional_str(
        "external_admission_status",
        metadata
            .external_admission_status
            .map(external_publication_status_key),
    );
    hash.field_optional_hash(
        "selected_candidate_provenance_hash",
        metadata.selected_candidate_provenance_hash,
    );
    hash.field_str("selection_reason", metadata.selection_reason);
    hash.field_hash("tie_break_key_hash", metadata.tie_break_key_hash);
    hash_dependency_compatibility(&mut hash, metadata.dependency_compatibility);
    hash.field_optional_hash(
        "explanation_ref",
        metadata.explanation_ref.map(ExplanationRef::hash),
    );
    hash.field_optional_hash("diagnostic_result_id", metadata.diagnostic_result_id);
    hash.finalize()
}

fn hash_dependency_compatibility(
    hash: &mut StableHasher,
    compatibility: Option<ProofReuseDependencyCompatibility>,
) {
    hash.field_bool("dependency_compatibility_present", compatibility.is_some());
    if let Some(compatibility) = compatibility {
        hash.field_hash(
            "dependency_artifact_fingerprint",
            compatibility.dependency_artifact_fingerprint(),
        );
        hash_schema_version(
            hash,
            "dependency_schema_version",
            compatibility.dependency_schema_version(),
        );
        hash_schema_version(
            hash,
            "proof_reuse_schema_version",
            compatibility.proof_reuse_schema_version(),
        );
    }
}

fn hash_schema_version(hash: &mut StableHasher, label: &str, version: SchemaVersion) {
    hash.field_u16(&format!("{label}_major"), version.major());
    hash.field_u16(&format!("{label}_minor"), version.minor());
}

fn proof_winner_class_key(class: ProofWinnerClass) -> &'static str {
    match class {
        ProofWinnerClass::KernelVerified => "kernel_verified",
        ProofWinnerClass::DischargedBuiltin => "discharged_builtin",
        ProofWinnerClass::PolicyPermittedExternal => "policy_permitted_external",
        ProofWinnerClass::PolicyAssumed => "policy_assumed",
        ProofWinnerClass::PolicyOpen => "policy_open",
        ProofWinnerClass::Rejected => "rejected",
        ProofWinnerClass::NoSelectableEvidence => "no_selectable_evidence",
    }
}

fn projected_proof_status_key(status: ProjectedProofStatus) -> &'static str {
    match status {
        ProjectedProofStatus::Accepted => "accepted",
        ProjectedProofStatus::ExternallyAttested => "externally_attested",
        ProjectedProofStatus::PolicyAssumed => "policy_assumed",
        ProjectedProofStatus::Open => "open",
        ProjectedProofStatus::Rejected => "rejected",
        ProjectedProofStatus::NotRequired => "not_required",
    }
}

fn external_publication_status_key(status: ExternalEvidencePublicationStatus) -> &'static str {
    match status {
        ExternalEvidencePublicationStatus::RejectedByPolicy => "rejected_by_policy",
        ExternalEvidencePublicationStatus::ExternallyAttestedDevelopment => {
            "externally_attested_development"
        }
        ExternalEvidencePublicationStatus::ExternallyAttestedOpenDiagnostic => {
            "externally_attested_open_diagnostic"
        }
        ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted => {
            "externally_attested_policy_permitted"
        }
    }
}

fn hash_used_axioms(used_axioms: &[UsedAxiom]) -> Hash {
    let mut hash = StableHasher::new(USED_AXIOMS_HASH_DOMAIN);
    hash.field_u64("used_axiom_count", used_axioms.len() as u64);
    for axiom in used_axioms {
        hash.field_str("namespace", imported_fact_namespace_key(axiom.namespace));
        hash.field_u32("imported_fact_id", axiom.imported_fact_id);
        hash.field_u8(
            "statement_fingerprint_algorithm",
            axiom.statement_fingerprint.algorithm_id,
        );
        hash.field_bytes(
            "statement_fingerprint_digest",
            &axiom.statement_fingerprint.digest,
        );
    }
    hash.finalize()
}

fn imported_fact_namespace_key(
    namespace: mizar_kernel::checker::ImportedFactNamespace,
) -> &'static str {
    match namespace {
        mizar_kernel::checker::ImportedFactNamespace::ImportedAxiom => "imported_axiom",
        mizar_kernel::checker::ImportedFactNamespace::ImportedTheorem => "imported_theorem",
        _ => "unknown",
    }
}

struct StableHasher {
    lanes: [u64; 4],
    length: u64,
}

impl StableHasher {
    fn new(domain: &str) -> Self {
        let mut hasher = Self {
            lanes: [
                0x6d_69_7a_61_72_2d_70_72,
                0x6f_6f_66_2d_73_74_61_74,
                0x75_73_2d_61_78_69_6f,
                0x6d_73_2d_68_61_73_68,
            ],
            length: 0,
        };
        hasher.field_str("domain", domain);
        hasher
    }

    fn field_str(&mut self, label: &str, value: &str) {
        self.field_bytes(label, value.as_bytes());
    }

    fn field_bool(&mut self, label: &str, value: bool) {
        self.field_bytes(label, &[u8::from(value)]);
    }

    fn field_u8(&mut self, label: &str, value: u8) {
        self.field_bytes(label, &[value]);
    }

    fn field_u16(&mut self, label: &str, value: u16) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_u32(&mut self, label: &str, value: u32) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_u64(&mut self, label: &str, value: u64) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_hash(&mut self, label: &str, value: Hash) {
        self.field_bytes(label, value.as_bytes());
    }

    fn field_optional_hash(&mut self, label: &str, value: Option<Hash>) {
        match value {
            Some(value) => {
                self.field_u8(&format!("{label}_tag"), 0);
                self.field_hash(label, value);
            }
            None => self.field_u8(&format!("{label}_tag"), 1),
        }
    }

    fn field_optional_str(&mut self, label: &str, value: Option<&str>) {
        match value {
            Some(value) => {
                self.field_u8(&format!("{label}_tag"), 0);
                self.field_str(label, value);
            }
            None => self.field_u8(&format!("{label}_tag"), 1),
        }
    }

    fn field_bytes(&mut self, label: &str, value: &[u8]) {
        self.feed_bytes(&(label.len() as u64).to_le_bytes());
        self.feed_bytes(label.as_bytes());
        self.feed_bytes(&(value.len() as u64).to_le_bytes());
        self.feed_bytes(value);
    }

    fn feed_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            let lane = self.length as usize % self.lanes.len();
            let mixed = self.length.rotate_left((lane as u32) + 7);
            self.lanes[lane] ^= u64::from(*byte)
                .wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_add(mixed);
            self.lanes[lane] = self.lanes[lane]
                .rotate_left(13 + lane as u32)
                .wrapping_mul(0x1000_0000_01b3);
            self.length = self.length.wrapping_add(1);
        }
    }

    fn finalize(mut self) -> Hash {
        self.lanes[0] ^= self.length;
        self.lanes[1] ^= self.length.rotate_left(17);
        self.lanes[2] ^= self.lanes[0].rotate_left(9);
        self.lanes[3] ^= self.lanes[1].rotate_left(13);

        let mut bytes = [0_u8; Hash::BYTE_LEN];
        for (chunk, lane) in bytes.chunks_exact_mut(8).zip(self.lanes) {
            chunk.copy_from_slice(&lane.to_be_bytes());
        }
        Hash::from_bytes(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        policy::{
            CandidatePolicyClass, ExternalEvidenceMode, KernelPolicyInput, PolicyCandidate,
            PolicyDecision, PolicyDiagnostic, PolicyDiagnosticCategory, PolicyReasonCode,
            ProofPolicyEvaluator,
        },
        selection::{
            ProofEvidenceCandidate, ProofEvidenceSet, VcProofSelection,
            merge_artifact_proof_selections, select_winner,
        },
    };
    use mizar_kernel::{
        certificate_parser::{
            ClauseTautologyPolicy, Fingerprint, KernelProfileRecord, RequiredProofStatus,
        },
        checker::{
            AcceptedProofStatus, FormulaEvidenceContext, FormulaImportedFactEvidence,
            ImportedFactContextLimits, ImportedFactNamespace, KernelCheckPolicy, KernelCheckStatus,
            KernelEvidenceCheckInput, KernelEvidenceCheckLimits, check_kernel_evidence,
        },
        clause::{Atom, SymbolId, SymbolKey, SymbolKind},
        formula_evidence::{
            Formula, FormulaEvidenceParseContext, FormulaSourceClass, GoalPolarity,
            ParsedKernelEvidence, SUPPORTED_FORMULA_FINGERPRINT_ALGORITHM_ID,
            parse_formula_evidence,
        },
        rejection::TargetVcFingerprint,
    };

    use super::*;

    #[test]
    fn obligation_identity_rejects_empty_fields() {
        assert_eq!(
            ObligationAnchor::new(""),
            Err(StatusProjectionError::EmptyObligationAnchor)
        );
        assert_eq!(
            ProofObligationIdentity::new(
                "",
                ObligationAnchor::new("anchor-0").expect("valid anchor"),
                hash(20),
                hash(21),
                hash(22),
                hash(23),
            ),
            Err(StatusProjectionError::EmptyObligationId)
        );
    }

    #[test]
    fn projects_selection_classes_without_status_collapse() {
        let kernel = project(selection_for_kernel(true), VerifierPolicy::release());
        assert_eq!(kernel.projected_status(), ProjectedProofStatus::Accepted);
        assert_eq!(
            kernel.artifact_publication(),
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Accepted)
        );
        assert_eq!(
            kernel.accepted_witness_obligation_id(),
            Some("obligation-0")
        );
        assert!(kernel.reuse_metadata().trusted_used_axioms_hash().is_none());

        let builtin = project(selection_for_builtin(), VerifierPolicy::release());
        assert_eq!(builtin.projected_status(), ProjectedProofStatus::Accepted);
        assert_eq!(
            builtin.artifact_publication(),
            ArtifactStatusPublication::ExternalDependencyGap(
                ArtifactPublicationGap::DischargedBuiltinWitnessUnsupported
            )
        );
        assert_eq!(
            builtin.reuse_metadata().deterministic_discharge_hash(),
            Some(hash(11))
        );
        assert_eq!(builtin.accepted_witness_obligation_id(), None);

        let external = project(selection_for_external(), external_policy());
        assert_eq!(
            external.projected_status(),
            ProjectedProofStatus::ExternallyAttested
        );
        assert_eq!(
            external.artifact_publication(),
            ArtifactStatusPublication::Publishable(
                CurrentArtifactObligationStatus::ExternallyAttested
            )
        );

        let assumed = project(selection_for_assumed(), assumed_policy());
        assert_eq!(
            assumed.projected_status(),
            ProjectedProofStatus::PolicyAssumed
        );
        assert_eq!(
            assumed.artifact_publication(),
            ArtifactStatusPublication::ExternalDependencyGap(
                ArtifactPublicationGap::PolicyAssumedStatusUnsupported
            )
        );

        let open = project(selection_for_open(), VerifierPolicy::interactive());
        assert_eq!(open.projected_status(), ProjectedProofStatus::Open);
        assert_eq!(
            open.artifact_publication(),
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Open)
        );

        let rejected = project(selection_for_rejected(), VerifierPolicy::release());
        assert_eq!(rejected.projected_status(), ProjectedProofStatus::Rejected);
        assert_eq!(
            rejected.artifact_publication(),
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
        );
    }

    #[test]
    fn no_selectable_evidence_projects_by_open_policy() {
        let release_policy = VerifierPolicy::release();
        let release = project(
            selection_for_no_selectable(release_policy.clone()),
            release_policy,
        );
        assert_eq!(release.projected_status(), ProjectedProofStatus::Rejected);
        assert_eq!(
            release.artifact_publication(),
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
        );

        let interactive_policy = VerifierPolicy::interactive();
        let interactive = project(
            selection_for_no_selectable(interactive_policy.clone()),
            interactive_policy,
        );
        assert_eq!(interactive.projected_status(), ProjectedProofStatus::Open);
        assert_eq!(
            interactive.artifact_publication(),
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Open)
        );

        let development_policy = VerifierPolicy::development();
        let development = project(
            selection_for_no_selectable(development_policy.clone()),
            development_policy,
        );
        assert_eq!(
            development.projected_status(),
            ProjectedProofStatus::Rejected
        );
        assert_eq!(
            development.artifact_publication(),
            ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Rejected)
        );
    }

    #[test]
    fn kernel_verified_artifact_publication_requires_witness_metadata() {
        let projection = project(selection_for_kernel(false), VerifierPolicy::release());

        assert_eq!(
            projection.projected_status(),
            ProjectedProofStatus::Accepted
        );
        assert_eq!(
            projection.artifact_publication(),
            ArtifactStatusPublication::ExternalDependencyGap(
                ArtifactPublicationGap::MissingKernelVerifiedWitness
            )
        );
        assert_eq!(projection.accepted_witness_obligation_id(), None);
    }

    #[test]
    fn trusted_used_axioms_require_matching_trusted_selection() {
        let trusted = TrustedUsedAxiomsRef::for_test(hash(10), hash(90), 2);
        let projection = project_input(selection_for_kernel(true), VerifierPolicy::release())
            .with_trusted_used_axioms(trusted.clone());
        let projection = project_status(projection).expect("matching trusted axioms project");

        assert_eq!(projection.trusted_used_axioms(), Some(&trusted));
        assert_eq!(
            projection.reuse_metadata().trusted_used_axioms_hash(),
            Some(hash(90))
        );

        let external = project_input(selection_for_external(), external_policy())
            .with_trusted_used_axioms(trusted);
        assert_eq!(
            project_status(external),
            Err(
                StatusProjectionError::TrustedUsedAxiomsForNonTrustedStatus {
                    selected_class: ProofWinnerClass::PolicyPermittedExternal,
                }
            )
        );
    }

    #[test]
    fn non_trusted_statuses_keep_trusted_used_axioms_absent() {
        let no_selectable_policy = VerifierPolicy::release();
        let cases = [
            (
                selection_for_external(),
                external_policy(),
                ProofWinnerClass::PolicyPermittedExternal,
            ),
            (
                selection_for_assumed(),
                assumed_policy(),
                ProofWinnerClass::PolicyAssumed,
            ),
            (
                selection_for_open(),
                VerifierPolicy::interactive(),
                ProofWinnerClass::PolicyOpen,
            ),
            (
                selection_for_rejected(),
                VerifierPolicy::release(),
                ProofWinnerClass::Rejected,
            ),
            (
                selection_for_no_selectable(no_selectable_policy.clone()),
                no_selectable_policy,
                ProofWinnerClass::NoSelectableEvidence,
            ),
        ];

        for (selection, policy, selected_class) in cases {
            let normal = project_status(project_input(selection.clone(), policy.clone()))
                .expect("non-trusted projection succeeds without trusted axioms");
            assert_eq!(normal.selected_class(), selected_class);
            assert!(normal.trusted_used_axioms().is_none());
            assert!(normal.reuse_metadata().trusted_used_axioms_hash().is_none());

            let forced = project_input(selection, policy)
                .with_trusted_used_axioms(TrustedUsedAxiomsRef::for_test(hash(10), hash(90), 2));
            assert_eq!(
                project_status(forced),
                Err(StatusProjectionError::TrustedUsedAxiomsForNonTrustedStatus { selected_class })
            );
        }
    }

    #[test]
    fn trusted_used_axioms_reject_mismatched_kernel_evidence_hash() {
        let trusted = TrustedUsedAxiomsRef::for_test(hash(12), hash(90), 2);
        let input = project_input(selection_for_kernel(true), VerifierPolicy::release())
            .with_trusted_used_axioms(trusted);

        assert_eq!(
            project_status(input),
            Err(StatusProjectionError::TrustedUsedAxiomsEvidenceMismatch {
                selected_evidence_hash: Some(hash(10)),
                accepted_evidence_hash: hash(12),
            })
        );
    }

    #[test]
    fn trusted_used_axioms_from_kernel_result_accepts_public_kernel_result() {
        let result = accepted_kernel_result();
        let trusted = TrustedUsedAxiomsRef::from_kernel_result(
            &result,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
        )
        .expect("accepted kernel result yields trusted used axioms ref");
        let expected_input = KernelPolicyInput::from_kernel_result(
            &result,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
        );

        assert_eq!(trusted.used_axiom_count(), 0);
        assert_eq!(
            trusted.accepted_evidence_hash(),
            expected_input
                .accepted_evidence_hash()
                .expect("accepted evidence hash")
        );
        assert_eq!(
            trusted.used_axioms_hash(),
            hash_used_axioms(result.used_axioms())
        );
    }

    #[test]
    fn trusted_used_axioms_from_kernel_result_rejects_untrusted_kernel_results() {
        let rejected = rejected_kernel_result();
        assert_eq!(
            TrustedUsedAxiomsRef::from_kernel_result(
                &rejected,
                KernelEvidenceOrigin::AtpFormulaSubstitution,
            ),
            Err(TrustedUsedAxiomsError::KernelResultNotAccepted)
        );

        let policy_tainted = policy_tainted_kernel_result();
        assert_eq!(policy_tainted.status(), KernelCheckStatus::Accepted);
        assert!(policy_tainted.policy_taint());
        assert_eq!(
            TrustedUsedAxiomsRef::from_kernel_result(
                &policy_tainted,
                KernelEvidenceOrigin::AtpFormulaSubstitution,
            ),
            Err(TrustedUsedAxiomsError::KernelResultPolicyTainted)
        );
    }

    #[test]
    fn trusted_used_axioms_reject_missing_accepted_evidence_hash() {
        let input = KernelPolicyInput::for_test(
            KernelCheckStatus::Accepted,
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            false,
            None,
        );

        assert_eq!(
            trusted_used_axioms_from_kernel_policy_input(&input, &[]),
            Err(TrustedUsedAxiomsError::MissingAcceptedEvidenceHash)
        );
    }

    #[test]
    fn projection_rejects_active_policy_mismatch() {
        let selection = selection_for_external();
        let active_policy = VerifierPolicy::release();
        let input = project_input(selection, active_policy.clone());

        assert_eq!(
            project_status(input),
            Err(StatusProjectionError::PolicyFingerprintMismatch {
                selected_policy_fingerprint: external_policy().policy_fingerprint(),
                active_policy_fingerprint: active_policy.policy_fingerprint(),
            })
        );
    }

    #[test]
    fn reuse_metadata_exports_architecture_22_identity() {
        let explanation = ExplanationRef::new(hash(70));
        let projection = project_status(
            project_input(selection_for_external(), external_policy())
                .with_dependency_compatibility(dependency_compatibility(40))
                .with_explanation_ref(explanation),
        )
        .expect("projection succeeds");
        let reuse = projection.reuse_metadata();
        let evidence_identity = reuse.proof_evidence_identity();

        assert_eq!(
            reuse.projected_status(),
            ProjectedProofStatus::ExternallyAttested
        );
        assert_eq!(reuse.obligation_anchor().as_str(), "anchor-0");
        assert_eq!(reuse.obligation_fingerprint(), hash(20));
        assert_eq!(reuse.vc_fingerprint(), hash(21));
        assert_eq!(reuse.local_context_fingerprint(), hash(22));
        assert_eq!(reuse.dependency_slice_fingerprint(), hash(23));
        assert_eq!(
            reuse.policy_fingerprint(),
            external_policy().policy_fingerprint()
        );
        assert_eq!(reuse.selected_evidence_hash(), Some(hash(30)));
        assert_eq!(reuse.explanation_ref(), Some(explanation));
        assert_eq!(
            reuse.external_admission_status(),
            Some(ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted)
        );
        assert_eq!(
            reuse.dependency_compatibility(),
            Some(dependency_compatibility(40))
        );
        assert_eq!(
            evidence_identity
                .selected_candidate_id()
                .map(CandidateSourceId::as_str),
            Some("external")
        );
        assert_eq!(
            evidence_identity.selected_candidate_provenance_hash(),
            Some(hash(33))
        );
        assert_eq!(
            evidence_identity.selection_reason(),
            "deterministic_winner_class"
        );
        assert_eq!(
            evidence_identity.tie_break_key_hash(),
            reuse.tie_break_key_hash()
        );
        assert_eq!(
            reuse.proof_reuse_validation_hash(),
            hash_status_reuse_metadata(reuse)
        );
        assert!(!reuse.cache_reuse_predicate_complete());
    }

    #[test]
    fn proof_reuse_validation_hash_changes_or_invalidates_for_exported_components() {
        let projection = project_status(
            project_input(selection_for_kernel(true), VerifierPolicy::release())
                .with_dependency_compatibility(dependency_compatibility(40))
                .with_explanation_ref(ExplanationRef::new(hash(70))),
        )
        .expect("projection succeeds");
        let base = projection.reuse_metadata().clone();
        assert!(base.cache_reuse_predicate_complete());

        assert_hash_changed(&base, "selected_class", |metadata| {
            metadata.selected_class = ProofWinnerClass::PolicyAssumed;
        });
        assert_hash_changed(&base, "projected_status", |metadata| {
            metadata.projected_status = ProjectedProofStatus::Open;
        });
        assert_hash_changed(&base, "selected_candidate_id", |metadata| {
            metadata.selected_candidate_id =
                Some(CandidateSourceId::new("other").expect("stable id"));
        });
        assert_hash_changed(&base, "obligation_anchor", |metadata| {
            metadata.obligation_anchor = ObligationAnchor::new("anchor-1").expect("stable anchor");
        });
        assert_hash_changed(&base, "obligation_fingerprint", |metadata| {
            metadata.obligation_fingerprint = hash(41);
        });
        assert_hash_changed(&base, "vc_fingerprint", |metadata| {
            metadata.vc_fingerprint = hash(42);
        });
        assert_hash_changed(&base, "local_context_fingerprint", |metadata| {
            metadata.local_context_fingerprint = hash(43);
        });
        assert_hash_changed(&base, "dependency_slice_fingerprint", |metadata| {
            metadata.dependency_slice_fingerprint = hash(44);
        });
        assert_hash_changed(&base, "policy_fingerprint", |metadata| {
            metadata.policy_fingerprint = external_policy().policy_fingerprint();
        });
        assert_hash_changed(&base, "selected_evidence_hash", |metadata| {
            metadata.selected_evidence_hash = Some(hash(45));
        });
        assert_hash_changed(&base, "selected_proof_witness_hash", |metadata| {
            metadata.selected_proof_witness_hash = Some(hash(46));
        });
        assert_hash_changed(&base, "deterministic_discharge_hash", |metadata| {
            metadata.deterministic_discharge_hash = Some(hash(47));
        });
        assert_hash_changed(&base, "trusted_used_axioms_hash", |metadata| {
            metadata.trusted_used_axioms_hash = Some(hash(48));
        });
        assert_hash_changed(&base, "external_admission_status", |metadata| {
            metadata.external_admission_status =
                Some(ExternalEvidencePublicationStatus::ExternallyAttestedDevelopment);
        });
        assert_hash_changed(&base, "selected_candidate_provenance_hash", |metadata| {
            metadata.selected_candidate_provenance_hash = Some(hash(49));
        });
        assert_hash_changed(&base, "selection_reason", |metadata| {
            metadata.selection_reason = "primary_rejected_diagnostic";
        });
        assert_hash_changed(&base, "tie_break_key_hash", |metadata| {
            metadata.tie_break_key_hash = hash(50);
        });
        assert_hash_changed(&base, "dependency_artifact_fingerprint", |metadata| {
            metadata.dependency_compatibility = Some(ProofReuseDependencyCompatibility::new(
                hash(51),
                SchemaVersion::new(1, 40),
                SchemaVersion::new(2, 40),
            ));
        });
        assert_hash_changed(&base, "dependency_schema_version", |metadata| {
            metadata.dependency_compatibility = Some(ProofReuseDependencyCompatibility::new(
                hash(40),
                SchemaVersion::new(1, 41),
                SchemaVersion::new(2, 40),
            ));
        });
        assert_hash_changed(&base, "proof_reuse_schema_version", |metadata| {
            metadata.dependency_compatibility = Some(ProofReuseDependencyCompatibility::new(
                hash(40),
                SchemaVersion::new(1, 40),
                SchemaVersion::new(2, 41),
            ));
        });
        assert_hash_changed(&base, "explanation_ref", |metadata| {
            metadata.explanation_ref = Some(ExplanationRef::new(hash(52)));
        });
        assert_hash_changed(&base, "diagnostic_result_id", |metadata| {
            metadata.diagnostic_result_id = Some(hash(53));
        });

        let mut missing_dependency_compatibility = base.clone();
        missing_dependency_compatibility.dependency_compatibility = None;
        assert!(!missing_dependency_compatibility.cache_reuse_predicate_complete());
        assert_ne!(
            hash_status_reuse_metadata(&missing_dependency_compatibility),
            base.proof_reuse_validation_hash()
        );
    }

    #[test]
    fn externally_attested_reuse_metadata_never_upgrades_trust() {
        let projection = project_status(
            project_input(selection_for_external(), external_policy())
                .with_dependency_compatibility(dependency_compatibility(40)),
        )
        .expect("projection succeeds");
        let reuse = projection.reuse_metadata();

        assert_eq!(
            projection.projected_status(),
            ProjectedProofStatus::ExternallyAttested
        );
        assert_eq!(
            reuse.selected_class(),
            ProofWinnerClass::PolicyPermittedExternal
        );
        assert!(!projection.projected_status().is_trusted());
        assert!(!reuse.projected_status().is_trusted());
        assert!(!reuse.cache_reuse_predicate_complete());
        assert_eq!(projection.trusted_used_axioms(), None);
        assert_eq!(reuse.trusted_used_axioms_hash(), None);
        assert_eq!(
            reuse.external_admission_status(),
            Some(ExternalEvidencePublicationStatus::ExternallyAttestedPolicyPermitted)
        );
        assert_eq!(
            reuse.proof_evidence_identity().selected_evidence_hash(),
            Some(hash(30))
        );
        assert_eq!(projection.accepted_witness_obligation_id(), None);
    }

    #[test]
    fn cache_reuse_predicate_completeness_is_class_aware() {
        let kernel_with_witness = project_status(
            project_input(selection_for_kernel(true), VerifierPolicy::release())
                .with_dependency_compatibility(dependency_compatibility(40)),
        )
        .expect("projection succeeds");
        assert!(
            kernel_with_witness
                .reuse_metadata()
                .cache_reuse_predicate_complete()
        );

        let kernel_without_witness = project_status(
            project_input(selection_for_kernel(false), VerifierPolicy::release())
                .with_dependency_compatibility(dependency_compatibility(40)),
        )
        .expect("projection succeeds");
        assert!(
            !kernel_without_witness
                .reuse_metadata()
                .cache_reuse_predicate_complete()
        );

        let builtin = project_status(
            project_input(selection_for_builtin(), VerifierPolicy::release())
                .with_dependency_compatibility(dependency_compatibility(40)),
        )
        .expect("projection succeeds");
        assert!(builtin.reuse_metadata().cache_reuse_predicate_complete());

        let external = project_status(
            project_input(selection_for_external(), external_policy())
                .with_dependency_compatibility(dependency_compatibility(40)),
        )
        .expect("projection succeeds");
        assert!(!external.reuse_metadata().cache_reuse_predicate_complete());
    }

    #[test]
    fn reuse_metadata_covers_trusted_rejected_and_no_selectable_outcomes() {
        let kernel = project(selection_for_kernel(true), VerifierPolicy::release());
        let kernel_reuse = kernel.reuse_metadata();
        assert_eq!(
            kernel_reuse.selected_class(),
            ProofWinnerClass::KernelVerified
        );
        assert_eq!(
            kernel_reuse.projected_status(),
            ProjectedProofStatus::Accepted
        );
        assert_eq!(kernel_reuse.selected_evidence_hash(), Some(hash(10)));
        assert_eq!(kernel_reuse.selected_proof_witness_hash(), Some(hash(50)));
        assert_eq!(kernel_reuse.deterministic_discharge_hash(), None);
        assert_eq!(kernel_reuse.diagnostic_result_id(), None);

        let builtin = project(selection_for_builtin(), VerifierPolicy::release());
        let builtin_reuse = builtin.reuse_metadata();
        assert_eq!(
            builtin_reuse.selected_class(),
            ProofWinnerClass::DischargedBuiltin
        );
        assert_eq!(
            builtin_reuse.projected_status(),
            ProjectedProofStatus::Accepted
        );
        assert_eq!(builtin_reuse.selected_evidence_hash(), Some(hash(10)));
        assert_eq!(builtin_reuse.selected_proof_witness_hash(), None);
        assert_eq!(builtin_reuse.deterministic_discharge_hash(), Some(hash(11)));
        assert_eq!(builtin_reuse.diagnostic_result_id(), None);

        let rejected = project(selection_for_rejected(), VerifierPolicy::release());
        let rejected_reuse = rejected.reuse_metadata();
        assert_eq!(rejected_reuse.selected_class(), ProofWinnerClass::Rejected);
        assert_eq!(
            rejected_reuse.projected_status(),
            ProjectedProofStatus::Rejected
        );
        assert_eq!(rejected_reuse.selected_evidence_hash(), None);
        assert!(rejected_reuse.selected_candidate_id().is_some());
        assert_eq!(rejected_reuse.diagnostic_result_id(), None);

        let release_policy = VerifierPolicy::release();
        let no_selectable = project(
            selection_for_no_selectable(release_policy.clone()),
            release_policy,
        );
        let no_selectable_reuse = no_selectable.reuse_metadata();
        assert_eq!(
            no_selectable_reuse.selected_class(),
            ProofWinnerClass::NoSelectableEvidence
        );
        assert_eq!(
            no_selectable_reuse.projected_status(),
            ProjectedProofStatus::Rejected
        );
        assert_eq!(no_selectable_reuse.selected_evidence_hash(), None);
        assert!(no_selectable_reuse.selected_candidate_id().is_none());
        assert!(no_selectable_reuse.diagnostic_result_id().is_some());
    }

    fn project(selection: ArtifactProofSelection, policy: VerifierPolicy) -> ProofStatusProjection {
        project_status(project_input(selection, policy)).expect("projection succeeds")
    }

    fn project_input(
        selection: ArtifactProofSelection,
        policy: VerifierPolicy,
    ) -> ProofStatusProjectionInput {
        ProofStatusProjectionInput::new(selection, policy, identity())
    }

    fn dependency_compatibility(tag: u8) -> ProofReuseDependencyCompatibility {
        ProofReuseDependencyCompatibility::new(
            hash(tag),
            SchemaVersion::new(1, u16::from(tag)),
            SchemaVersion::new(2, u16::from(tag)),
        )
    }

    fn assert_hash_changed(
        base: &StatusReuseMetadata,
        label: &str,
        mutate: impl FnOnce(&mut StatusReuseMetadata),
    ) {
        let mut changed = base.clone();
        mutate(&mut changed);
        assert_ne!(
            hash_status_reuse_metadata(&changed),
            base.proof_reuse_validation_hash(),
            "{label} should participate in proof reuse validation hash"
        );
    }

    fn identity() -> ProofObligationIdentity {
        ProofObligationIdentity::new(
            "obligation-0",
            ObligationAnchor::new("anchor-0").expect("valid anchor"),
            hash(20),
            hash(21),
            hash(22),
            hash(23),
        )
        .expect("valid obligation identity")
    }

    fn selection_for_kernel(with_witness: bool) -> ArtifactProofSelection {
        let mut candidate = trusted_candidate(
            "kernel",
            KernelEvidenceOrigin::AtpFormulaSubstitution,
            hash(10),
        )
        .with_provenance_hash(hash(34));
        if with_witness {
            candidate = candidate.with_selected_proof_witness_hash(hash(50));
        }
        artifact_selection(
            [VcProofSelection::new(
                VcId::new(0),
                select_winner(&base_set(VerifierPolicy::release()).with_candidates([candidate])),
            )],
            [],
        )
    }

    fn selection_for_builtin() -> ArtifactProofSelection {
        artifact_selection(
            [],
            [VcProofSelection::new(
                VcId::new(0),
                select_winner(
                    &base_set(VerifierPolicy::release()).with_candidates([trusted_candidate(
                        "builtin",
                        KernelEvidenceOrigin::BuiltinDischarge,
                        hash(10),
                    )
                    .with_deterministic_discharge_hash(hash(11))]),
                ),
            )],
        )
    }

    fn selection_for_external() -> ArtifactProofSelection {
        let policy = external_policy();
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        artifact_selection(
            [VcProofSelection::new(
                VcId::new(0),
                select_winner(
                    &base_set(policy).with_candidates([candidate(
                        "external",
                        evaluator.evaluate_candidate(&PolicyCandidate::ExternallyAttested),
                    )
                    .with_evidence_hash(hash(30))
                    .with_provenance_hash(hash(33))]),
                ),
            )],
            [],
        )
    }

    fn selection_for_assumed() -> ArtifactProofSelection {
        let policy = assumed_policy();
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        artifact_selection(
            [VcProofSelection::new(
                VcId::new(0),
                select_winner(
                    &base_set(policy).with_candidates([candidate(
                        "assumed",
                        evaluator.evaluate_candidate(&PolicyCandidate::PolicyAssumption),
                    )
                    .with_evidence_hash(hash(31))]),
                ),
            )],
            [],
        )
    }

    fn selection_for_open() -> ArtifactProofSelection {
        let policy = VerifierPolicy::interactive();
        let evaluator = ProofPolicyEvaluator::new(policy.clone());
        artifact_selection(
            [VcProofSelection::new(
                VcId::new(0),
                select_winner(
                    &base_set(policy).with_candidates([candidate(
                        "open",
                        evaluator.evaluate_candidate(&PolicyCandidate::OpenObligation),
                    )
                    .with_evidence_hash(hash(32))]),
                ),
            )],
            [],
        )
    }

    fn selection_for_rejected() -> ArtifactProofSelection {
        artifact_selection(
            [VcProofSelection::new(
                VcId::new(0),
                select_winner(
                    &base_set(VerifierPolicy::release())
                        .with_candidates([candidate("rejected", rejected_policy_decision())]),
                ),
            )],
            [],
        )
    }

    fn selection_for_no_selectable(policy: VerifierPolicy) -> ArtifactProofSelection {
        artifact_selection(
            [VcProofSelection::new(
                VcId::new(0),
                select_winner(&base_set(policy)),
            )],
            [],
        )
    }

    fn artifact_selection(
        portfolio: impl IntoIterator<Item = VcProofSelection>,
        builtin_discharge: impl IntoIterator<Item = VcProofSelection>,
    ) -> ArtifactProofSelection {
        let mut merged = merge_artifact_proof_selections(portfolio, builtin_discharge)
            .expect("valid artifact selection");
        assert_eq!(merged.len(), 1);
        merged.remove(0)
    }

    fn base_set(policy: VerifierPolicy) -> ProofEvidenceSet {
        ProofEvidenceSet::new(b"obligation".to_vec(), hash(100), policy)
    }

    fn candidate(id: &str, decision: PolicyDecision) -> ProofEvidenceCandidate {
        ProofEvidenceCandidate::new(CandidateSourceId::new(id).expect("stable id"), decision)
    }

    fn trusted_candidate(
        id: &str,
        origin: KernelEvidenceOrigin,
        evidence_hash: Hash,
    ) -> ProofEvidenceCandidate {
        let kernel_input = KernelPolicyInput::for_test(
            KernelCheckStatus::Accepted,
            origin,
            false,
            Some(evidence_hash),
        );
        ProofEvidenceCandidate::from_trusted_kernel_input(
            CandidateSourceId::new(id).expect("stable id"),
            &kernel_input,
        )
        .expect("accepted kernel input is trusted evidence")
    }

    fn rejected_policy_decision() -> PolicyDecision {
        PolicyDecision {
            class: CandidatePolicyClass::RejectedByPolicy,
            can_schedule_kernel_check: false,
            diagnostic: Some(PolicyDiagnostic::new(
                PolicyDiagnosticCategory::PolicyRejection,
                PolicyReasonCode::OpenObligationRejected,
            )),
            kernel_rejections: Vec::new(),
            external_admission: None,
        }
    }

    fn external_policy() -> VerifierPolicy {
        VerifierPolicy::interactive()
            .with_external_evidence(ExternalEvidenceMode::PermitNonTrustedWinner)
    }

    fn assumed_policy() -> VerifierPolicy {
        VerifierPolicy::development()
    }

    fn accepted_kernel_result() -> KernelCheckResult {
        let target = formula_target(7);
        let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
        let premise = formula_atom(1);
        let parsed = parsed_formula_evidence(
            &target,
            vec![local_formula_item(1, 10, &premise)],
            goal_item(20, &premise),
        );

        let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, None));
        assert_eq!(result.status(), KernelCheckStatus::Accepted);
        assert!(!result.policy_taint());
        result
    }

    fn rejected_kernel_result() -> KernelCheckResult {
        let target = formula_target(8);
        let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
        let parsed = parsed_formula_evidence(
            &target,
            vec![local_formula_item(1, 10, &formula_atom(1))],
            goal_item(20, &formula_atom(2)),
        );

        let result = check_kernel_evidence(evidence_input(&target_vc, &parsed, None));
        assert_eq!(result.status(), KernelCheckStatus::Rejected);
        result
    }

    fn policy_tainted_kernel_result() -> KernelCheckResult {
        let target = formula_target(9);
        let target_vc = TargetVcFingerprint::from_certificate_fingerprint(&target);
        let premise = formula_atom(1);
        let parsed = parsed_formula_evidence(
            &target,
            vec![imported_formula_item(
                1,
                10,
                &premise,
                RequiredProofStatus::ExternallyAttestedPolicyPermitted,
            )],
            goal_item(20, &premise),
        );
        let context = formula_evidence_context(
            formula_imported_fact(
                5,
                &premise,
                AcceptedProofStatus::ExternallyAttestedPolicyPermitted,
            ),
            ImportedFactNamespace::ImportedAxiom,
        );
        let mut input = evidence_input(&target_vc, &parsed, Some(&context));
        input.policy = KernelCheckPolicy {
            imported_fact_policy: mizar_kernel::checker::ImportedFactPolicy {
                allow_externally_attested: true,
            },
            ..KernelCheckPolicy::default()
        };

        let result = check_kernel_evidence(input);
        assert_eq!(result.status(), KernelCheckStatus::Accepted);
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

    fn local_formula_item(
        formula_id: u32,
        provenance_id: u32,
        formula: &Formula,
    ) -> FormulaFixture {
        let fingerprint = formula_fingerprint(formula);
        let mut bytes = Vec::new();
        put_u32(formula_id, &mut bytes);
        bytes.push(FormulaSourceClass::LocalHypothesis.tag());
        put_fingerprint(&fingerprint, &mut bytes);
        put_u32(provenance_id, &mut bytes);
        put_u32(1, &mut bytes);
        put_formula(formula, &mut bytes);
        FormulaFixture {
            bytes,
            provenance_id,
            fingerprint,
        }
    }

    fn imported_formula_item(
        formula_id: u32,
        provenance_id: u32,
        formula: &Formula,
        required_status: RequiredProofStatus,
    ) -> FormulaFixture {
        let fingerprint = formula_fingerprint(formula);
        let mut bytes = Vec::new();
        put_u32(formula_id, &mut bytes);
        bytes.push(FormulaSourceClass::AcceptedImportedAxiom.tag());
        put_fingerprint(&fingerprint, &mut bytes);
        put_u32(provenance_id, &mut bytes);
        put_bytes(b"pkg", &mut bytes);
        put_bytes(b"module", &mut bytes);
        put_bytes(b"ITEM", &mut bytes);
        put_fingerprint(&fingerprint, &mut bytes);
        bytes.push(required_status_tag(required_status));
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
            policy: KernelCheckPolicy::default(),
            limits: KernelEvidenceCheckLimits::default(),
        }
    }

    fn formula_evidence_context(
        imported: FormulaImportedFactEvidence,
        namespace: ImportedFactNamespace,
    ) -> FormulaEvidenceContext {
        let (axioms, theorems) = match namespace {
            ImportedFactNamespace::ImportedAxiom => (vec![imported], Vec::new()),
            ImportedFactNamespace::ImportedTheorem => (Vec::new(), vec![imported]),
            _ => panic!("unknown imported fact namespace in status projection test fixture"),
        };
        FormulaEvidenceContext::new(
            Some(vec![1]),
            axioms,
            theorems,
            ImportedFactContextLimits::default(),
        )
        .expect("formula evidence context")
    }

    fn formula_imported_fact(
        imported_fact_id: u32,
        formula: &Formula,
        accepted_proof_status: AcceptedProofStatus,
    ) -> FormulaImportedFactEvidence {
        FormulaImportedFactEvidence {
            imported_fact_id,
            package_id: b"pkg".to_vec(),
            module_path: b"module".to_vec(),
            exported_item_id: b"ITEM".to_vec(),
            statement_fingerprint: formula_fingerprint(formula),
            accepted_proof_status,
        }
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

    fn provenance_item(
        target: &Fingerprint,
        provenance_id: u32,
        fingerprint: &Fingerprint,
    ) -> Vec<u8> {
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
            _ => panic!("unknown formula variant in status projection test fixture"),
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
            _ => panic!("unknown symbol kind in status projection test fixture"),
        }
    }

    fn required_status_tag(status: RequiredProofStatus) -> u8 {
        match status {
            RequiredProofStatus::KernelVerified => 1,
            RequiredProofStatus::DischargedBuiltin => 2,
            RequiredProofStatus::ExternallyAttestedPolicyPermitted => 3,
            _ => panic!("unknown required proof status in status projection test fixture"),
        }
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }
}
