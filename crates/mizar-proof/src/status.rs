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
mod tests;
