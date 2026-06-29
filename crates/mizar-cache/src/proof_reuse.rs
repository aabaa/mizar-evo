//! Cache-side proof-reuse validation.
//!
//! The contract is specified in
//! [proof_reuse.md](../../../doc/design/mizar-cache/en/proof_reuse.md).

use mizar_artifact::store::SchemaVersion as ArtifactSchemaVersion;
use mizar_proof::{selection::ProofWinnerClass, status::StatusReuseMetadata};
use mizar_session::Hash;

use crate::cache_key::{
    CACHE_KEY_SCHEMA_VERSION, CompatibilityField, DependencyArtifactAvailability,
    DiagnosticRefHash, FootprintCompleteness, NamedSchemaVersion, SchemaVersion,
};

/// Current cache-side proof-reuse validation schema.
pub const PROOF_REUSE_SCHEMA_VERSION: &str = "mizar-cache/proof-reuse-schema/v1";

/// Cache-side snapshot of proof metadata exported by `mizar-proof`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofReuseMetadataSnapshot {
    /// Selected class exported by the proof owner.
    pub selected_class: ProofWinnerClass,
    /// Stable selected candidate id, when present.
    pub selected_candidate_id: Option<String>,
    /// Stable obligation anchor.
    pub obligation_anchor: String,
    /// Obligation fingerprint.
    pub obligation_fingerprint: Hash,
    /// Canonical VC fingerprint.
    pub canonical_vc_fingerprint: Hash,
    /// Canonical local-context fingerprint.
    pub local_context_fingerprint: Hash,
    /// Dependency-slice fingerprint.
    pub dependency_slice_fingerprint: Hash,
    /// Active proof-policy fingerprint.
    pub policy_fingerprint: Hash,
    /// Selected evidence hash.
    pub selected_evidence_hash: Option<Hash>,
    /// Selected proof witness hash for trusted kernel evidence.
    pub selected_proof_witness_hash: Option<Hash>,
    /// Deterministic discharge hash for built-in discharge evidence.
    pub deterministic_discharge_hash: Option<Hash>,
    /// Trusted used-axioms reference hash exported by proof status, if any.
    pub trusted_used_axioms_hash: Option<Hash>,
    /// Producer-owned selected candidate provenance hash, if any.
    pub selected_candidate_provenance_hash: Option<Hash>,
    /// Stable selection reason exported by `mizar-proof`.
    pub selection_reason: String,
    /// Stable tie-break key hash exported by `mizar-proof`.
    pub tie_break_key_hash: Hash,
    /// Dependency artifact and proof-reuse schema compatibility metadata.
    pub dependency_compatibility: Option<ProofReuseDependencyCompatibilitySnapshot>,
    /// Stable proof-reuse validation hash exported by `mizar-proof`.
    pub proof_reuse_validation_hash: Hash,
    /// Upstream class-aware completeness predicate exported by `mizar-proof`.
    pub cache_reuse_predicate_complete: bool,
}

/// Cache-side copy of proof dependency compatibility metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProofReuseDependencyCompatibilitySnapshot {
    /// Dependency artifact fingerprint exported by proof status.
    pub dependency_artifact_fingerprint: Hash,
    /// Dependency schema version exported by proof status.
    pub dependency_schema_version: ArtifactSchemaVersion,
    /// Proof-reuse schema version exported by proof status.
    pub proof_reuse_schema_version: ArtifactSchemaVersion,
}

/// Fail-closed cache-side guards that are not proof authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofReuseValidationEnvironment {
    /// Cache-key schema used by the current lookup.
    pub cache_schema_version: SchemaVersion,
    /// Cache-side proof-reuse schema versions affecting interpretation.
    pub proof_reuse_schema_versions: Vec<NamedSchemaVersion>,
    /// Toolchain compatibility fields for the current lookup.
    pub toolchain_compatibility: Vec<CompatibilityField>,
    /// Verifier/proof policy compatibility fields for the current lookup.
    pub policy_compatibility: Vec<CompatibilityField>,
    /// Dependency artifact availability/hash metadata supplied by the cache boundary.
    pub dependency_artifacts: Vec<DependencyArtifactAvailability>,
    /// Dependency footprint completeness for the current lookup.
    pub footprint_completeness: FootprintCompleteness,
    /// Explicit uncacheable marker.
    pub uncacheable: bool,
}

/// Request for validating one upstream-selected cached proof-reuse candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofReuseValidationRequest {
    /// Current proof metadata exported by `mizar-proof`.
    pub current: ProofReuseMetadataSnapshot,
    /// Cached proof metadata being tested for reuse.
    pub cached: ProofReuseMetadataSnapshot,
    /// Cache-side fail-closed guards.
    pub environment: ProofReuseValidationEnvironment,
    /// Diagnostic-only references for explaining the decision.
    pub diagnostic_refs: Vec<DiagnosticRefHash>,
}

/// Result of proof-reuse validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofReuseValidationOutcome {
    /// The cached candidate is compatible with current inputs.
    Hit(ProofReuseValidationHit),
    /// The cached candidate must be treated as a miss.
    Miss(ProofReuseValidationMiss),
}

/// Successful cache-side proof-reuse validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofReuseValidationHit {
    /// Reused class, unchanged from `mizar-proof`.
    pub selected_class: ProofWinnerClass,
    /// Matched selected witness or deterministic discharge hash.
    pub witness_or_discharge_hash: Hash,
    /// Matched proof-reuse validation hash.
    pub proof_reuse_validation_hash: Hash,
    /// Canonical diagnostic references, for explanation only.
    pub diagnostic_refs: Vec<DiagnosticRefHash>,
}

/// Fail-closed cache-side proof-reuse miss.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofReuseValidationMiss {
    /// Miss reason. This is diagnostic-only.
    pub reason: ProofReuseMissReason,
    /// Canonical diagnostic references, for explanation only.
    pub diagnostic_refs: Vec<DiagnosticRefHash>,
}

/// Diagnostic-only reason a proof-reuse candidate missed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofReuseMissReason {
    /// Cache or proof-reuse schema compatibility is unsupported.
    UnsupportedSchema,
    /// Toolchain compatibility is unknown.
    UnknownToolchain,
    /// Dependency footprint is incomplete.
    IncompleteFootprint,
    /// Dependency footprint schema or completeness is unsupported.
    UnsupportedFootprint,
    /// Explicit uncacheable marker is present.
    Uncacheable,
    /// Cached and current selected proof classes differ.
    SelectedClassMismatch,
    /// The selected class is not a complete reusable proof class.
    NonReusableClass,
    /// Upstream metadata reports an incomplete reuse predicate.
    IncompleteUpstreamPredicate,
    /// Non-trusted metadata attempted to carry trusted used-axiom references.
    SynthesizedTrustedAxiomSetReference,
    /// Obligation anchor mismatch.
    ObligationAnchorMismatch,
    /// Obligation fingerprint mismatch.
    ObligationFingerprintMismatch,
    /// Canonical VC fingerprint mismatch.
    CanonicalVcFingerprintMismatch,
    /// Local-context fingerprint mismatch.
    LocalContextFingerprintMismatch,
    /// Dependency-slice fingerprint mismatch.
    DependencySliceFingerprintMismatch,
    /// Policy fingerprint mismatch.
    PolicyMismatch,
    /// Policy compatibility is unknown or incompatible.
    PolicyCompatibilityMismatch,
    /// Selected candidate identity mismatch.
    SelectedCandidateMismatch,
    /// Selected candidate identity is missing for a reusable trusted class.
    SelectedCandidateMissing,
    /// Selected evidence hash mismatch.
    SelectedEvidenceHashMismatch,
    /// Selected candidate provenance hash mismatch.
    SelectedCandidateProvenanceMismatch,
    /// Stable selection reason mismatch.
    SelectionReasonMismatch,
    /// Stable tie-break hash mismatch.
    TieBreakKeyMismatch,
    /// Required selected witness hash is missing.
    SelectedWitnessHashMissing,
    /// Selected witness hash is not valid for the selected class.
    UnexpectedSelectedWitnessHash,
    /// Selected witness hash mismatch.
    SelectedWitnessHashMismatch,
    /// Required deterministic discharge hash is missing.
    DeterministicDischargeHashMissing,
    /// Deterministic discharge hash is not valid for the selected class.
    UnexpectedDeterministicDischargeHash,
    /// Deterministic discharge hash mismatch.
    DeterministicDischargeHashMismatch,
    /// Trusted used-axioms reference hash mismatch.
    TrustedAxiomSetReferenceMismatch,
    /// Dependency artifact fingerprint mismatch.
    DependencyArtifactMismatch,
    /// Dependency or proof-reuse schema version mismatch.
    SchemaVersionMismatch,
    /// Proof-reuse validation hash mismatch.
    ProofReuseValidationHashMismatch,
}

/// Stateless proof-reuse validator.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProofReuseValidator;

impl ProofReuseValidator {
    /// Validates a cached proof-reuse candidate against current metadata.
    pub fn validate(request: &ProofReuseValidationRequest) -> ProofReuseValidationOutcome {
        validate_proof_reuse(request)
    }
}

impl Default for ProofReuseValidationEnvironment {
    fn default() -> Self {
        Self {
            cache_schema_version: SchemaVersion::new(CACHE_KEY_SCHEMA_VERSION),
            proof_reuse_schema_versions: Vec::new(),
            toolchain_compatibility: Vec::new(),
            policy_compatibility: Vec::new(),
            dependency_artifacts: Vec::new(),
            footprint_completeness: FootprintCompleteness::IncompleteUncacheable,
            uncacheable: true,
        }
    }
}

impl From<mizar_proof::status::ProofReuseDependencyCompatibility>
    for ProofReuseDependencyCompatibilitySnapshot
{
    fn from(value: mizar_proof::status::ProofReuseDependencyCompatibility) -> Self {
        Self {
            dependency_artifact_fingerprint: value.dependency_artifact_fingerprint(),
            dependency_schema_version: value.dependency_schema_version(),
            proof_reuse_schema_version: value.proof_reuse_schema_version(),
        }
    }
}

impl From<&StatusReuseMetadata> for ProofReuseMetadataSnapshot {
    fn from(metadata: &StatusReuseMetadata) -> Self {
        Self {
            selected_class: metadata.selected_class(),
            selected_candidate_id: metadata
                .selected_candidate_id()
                .map(|candidate| candidate.as_str().to_owned()),
            obligation_anchor: metadata.obligation_anchor().as_str().to_owned(),
            obligation_fingerprint: metadata.obligation_fingerprint(),
            canonical_vc_fingerprint: metadata.vc_fingerprint(),
            local_context_fingerprint: metadata.local_context_fingerprint(),
            dependency_slice_fingerprint: metadata.dependency_slice_fingerprint(),
            policy_fingerprint: metadata.policy_fingerprint().hash(),
            selected_evidence_hash: metadata.selected_evidence_hash(),
            selected_proof_witness_hash: metadata.selected_proof_witness_hash(),
            deterministic_discharge_hash: metadata.deterministic_discharge_hash(),
            trusted_used_axioms_hash: metadata.trusted_used_axioms_hash(),
            selected_candidate_provenance_hash: metadata.selected_candidate_provenance_hash(),
            selection_reason: metadata.selection_reason().to_owned(),
            tie_break_key_hash: metadata.tie_break_key_hash(),
            dependency_compatibility: metadata.dependency_compatibility().map(Into::into),
            proof_reuse_validation_hash: metadata.proof_reuse_validation_hash(),
            cache_reuse_predicate_complete: metadata.cache_reuse_predicate_complete(),
        }
    }
}

/// Validates a cached proof-reuse candidate against current metadata.
pub fn validate_proof_reuse(request: &ProofReuseValidationRequest) -> ProofReuseValidationOutcome {
    let diagnostic_refs = canonical_diagnostic_refs(request.diagnostic_refs.clone());
    let current = &request.current;
    let cached = &request.cached;

    if schema_incompatible(&request.environment) {
        return miss(ProofReuseMissReason::UnsupportedSchema, diagnostic_refs);
    }
    if unknown_toolchain(&request.environment.toolchain_compatibility) {
        return miss(ProofReuseMissReason::UnknownToolchain, diagnostic_refs);
    }
    if unknown_compatibility(&request.environment.policy_compatibility) {
        return miss(
            ProofReuseMissReason::PolicyCompatibilityMismatch,
            diagnostic_refs,
        );
    }
    if dependency_artifacts_unavailable(&request.environment.dependency_artifacts) {
        return miss(
            ProofReuseMissReason::DependencyArtifactMismatch,
            diagnostic_refs,
        );
    }
    match request.environment.footprint_completeness {
        FootprintCompleteness::Complete | FootprintCompleteness::ConservativeComplete => {}
        FootprintCompleteness::IncompleteUncacheable => {
            return miss(ProofReuseMissReason::IncompleteFootprint, diagnostic_refs);
        }
        FootprintCompleteness::Unsupported => {
            return miss(ProofReuseMissReason::UnsupportedFootprint, diagnostic_refs);
        }
    }
    if request.environment.uncacheable {
        return miss(ProofReuseMissReason::Uncacheable, diagnostic_refs);
    }

    if current.selected_class != cached.selected_class {
        return miss(ProofReuseMissReason::SelectedClassMismatch, diagnostic_refs);
    }
    if synthesized_trusted_used_axioms(current) || synthesized_trusted_used_axioms(cached) {
        return miss(
            ProofReuseMissReason::SynthesizedTrustedAxiomSetReference,
            diagnostic_refs,
        );
    }
    if !is_reusable_class(current.selected_class) {
        return miss(ProofReuseMissReason::NonReusableClass, diagnostic_refs);
    }

    if current.obligation_anchor != cached.obligation_anchor {
        return miss(
            ProofReuseMissReason::ObligationAnchorMismatch,
            diagnostic_refs,
        );
    }
    if current.obligation_fingerprint != cached.obligation_fingerprint {
        return miss(
            ProofReuseMissReason::ObligationFingerprintMismatch,
            diagnostic_refs,
        );
    }
    if current.canonical_vc_fingerprint != cached.canonical_vc_fingerprint {
        return miss(
            ProofReuseMissReason::CanonicalVcFingerprintMismatch,
            diagnostic_refs,
        );
    }
    if current.local_context_fingerprint != cached.local_context_fingerprint {
        return miss(
            ProofReuseMissReason::LocalContextFingerprintMismatch,
            diagnostic_refs,
        );
    }
    if current.dependency_slice_fingerprint != cached.dependency_slice_fingerprint {
        return miss(
            ProofReuseMissReason::DependencySliceFingerprintMismatch,
            diagnostic_refs,
        );
    }
    if current.policy_fingerprint != cached.policy_fingerprint {
        return miss(ProofReuseMissReason::PolicyMismatch, diagnostic_refs);
    }
    if current.selected_candidate_id != cached.selected_candidate_id {
        return miss(
            ProofReuseMissReason::SelectedCandidateMismatch,
            diagnostic_refs,
        );
    }
    if current.selected_candidate_id.is_none() {
        return miss(
            ProofReuseMissReason::SelectedCandidateMissing,
            diagnostic_refs,
        );
    }
    if current.selected_evidence_hash != cached.selected_evidence_hash {
        return miss(
            ProofReuseMissReason::SelectedEvidenceHashMismatch,
            diagnostic_refs,
        );
    }
    if current.selected_evidence_hash.is_none() {
        return miss(
            ProofReuseMissReason::SelectedEvidenceHashMismatch,
            diagnostic_refs,
        );
    }
    if current.selected_candidate_provenance_hash != cached.selected_candidate_provenance_hash {
        return miss(
            ProofReuseMissReason::SelectedCandidateProvenanceMismatch,
            diagnostic_refs,
        );
    }
    if current.selection_reason != cached.selection_reason {
        return miss(
            ProofReuseMissReason::SelectionReasonMismatch,
            diagnostic_refs,
        );
    }
    if current.tie_break_key_hash != cached.tie_break_key_hash {
        return miss(ProofReuseMissReason::TieBreakKeyMismatch, diagnostic_refs);
    }
    if current.trusted_used_axioms_hash != cached.trusted_used_axioms_hash {
        return miss(
            ProofReuseMissReason::TrustedAxiomSetReferenceMismatch,
            diagnostic_refs,
        );
    }

    let witness_or_discharge_hash = match current.selected_class {
        ProofWinnerClass::KernelVerified => {
            let Some(current_witness) = current.selected_proof_witness_hash else {
                return miss(
                    ProofReuseMissReason::SelectedWitnessHashMissing,
                    diagnostic_refs,
                );
            };
            let Some(cached_witness) = cached.selected_proof_witness_hash else {
                return miss(
                    ProofReuseMissReason::SelectedWitnessHashMissing,
                    diagnostic_refs,
                );
            };
            if current_witness != cached_witness {
                return miss(
                    ProofReuseMissReason::SelectedWitnessHashMismatch,
                    diagnostic_refs,
                );
            }
            if current.deterministic_discharge_hash.is_some()
                || cached.deterministic_discharge_hash.is_some()
            {
                return miss(
                    ProofReuseMissReason::UnexpectedDeterministicDischargeHash,
                    diagnostic_refs,
                );
            }
            current_witness
        }
        ProofWinnerClass::DischargedBuiltin => {
            let Some(current_discharge) = current.deterministic_discharge_hash else {
                return miss(
                    ProofReuseMissReason::DeterministicDischargeHashMissing,
                    diagnostic_refs,
                );
            };
            let Some(cached_discharge) = cached.deterministic_discharge_hash else {
                return miss(
                    ProofReuseMissReason::DeterministicDischargeHashMissing,
                    diagnostic_refs,
                );
            };
            if current_discharge != cached_discharge {
                return miss(
                    ProofReuseMissReason::DeterministicDischargeHashMismatch,
                    diagnostic_refs,
                );
            }
            if current.selected_proof_witness_hash.is_some()
                || cached.selected_proof_witness_hash.is_some()
            {
                return miss(
                    ProofReuseMissReason::UnexpectedSelectedWitnessHash,
                    diagnostic_refs,
                );
            }
            current_discharge
        }
        ProofWinnerClass::PolicyPermittedExternal
        | ProofWinnerClass::PolicyAssumed
        | ProofWinnerClass::PolicyOpen
        | ProofWinnerClass::Rejected
        | ProofWinnerClass::NoSelectableEvidence => {
            return miss(ProofReuseMissReason::NonReusableClass, diagnostic_refs);
        }
        _ => {
            return miss(ProofReuseMissReason::NonReusableClass, diagnostic_refs);
        }
    };

    if !current.cache_reuse_predicate_complete || !cached.cache_reuse_predicate_complete {
        return miss(
            ProofReuseMissReason::IncompleteUpstreamPredicate,
            diagnostic_refs,
        );
    }
    if let Some(reason) = dependency_compatibility_miss(
        current.dependency_compatibility,
        cached.dependency_compatibility,
    ) {
        return miss(reason, diagnostic_refs);
    }
    if current.proof_reuse_validation_hash != cached.proof_reuse_validation_hash {
        return miss(
            ProofReuseMissReason::ProofReuseValidationHashMismatch,
            diagnostic_refs,
        );
    }

    ProofReuseValidationOutcome::Hit(ProofReuseValidationHit {
        selected_class: current.selected_class,
        witness_or_discharge_hash,
        proof_reuse_validation_hash: current.proof_reuse_validation_hash,
        diagnostic_refs,
    })
}

fn miss(
    reason: ProofReuseMissReason,
    diagnostic_refs: Vec<DiagnosticRefHash>,
) -> ProofReuseValidationOutcome {
    ProofReuseValidationOutcome::Miss(ProofReuseValidationMiss {
        reason,
        diagnostic_refs,
    })
}

fn is_reusable_class(class: ProofWinnerClass) -> bool {
    matches!(
        class,
        ProofWinnerClass::KernelVerified | ProofWinnerClass::DischargedBuiltin
    )
}

fn synthesized_trusted_used_axioms(metadata: &ProofReuseMetadataSnapshot) -> bool {
    !metadata.selected_class.is_trusted() && metadata.trusted_used_axioms_hash.is_some()
}

fn schema_incompatible(environment: &ProofReuseValidationEnvironment) -> bool {
    environment.cache_schema_version.as_str() != CACHE_KEY_SCHEMA_VERSION
        || environment.proof_reuse_schema_versions.is_empty()
        || !environment
            .proof_reuse_schema_versions
            .iter()
            .any(|schema| {
                schema.schema_family == "mizar-cache/proof-reuse"
                    && schema.name == "validator"
                    && schema.version.as_str() == PROOF_REUSE_SCHEMA_VERSION
            })
        || environment
            .proof_reuse_schema_versions
            .iter()
            .any(|schema| {
                schema.schema_family.trim().is_empty()
                    || schema.name.trim().is_empty()
                    || unknown_value(schema.version.as_str())
            })
}

fn unknown_toolchain(fields: &[CompatibilityField]) -> bool {
    unknown_compatibility(fields)
}

fn unknown_compatibility(fields: &[CompatibilityField]) -> bool {
    fields.is_empty()
        || fields.iter().any(|field| {
            field.family.trim().is_empty()
                || field.field_name.trim().is_empty()
                || unknown_value(&field.value)
        })
}

fn unknown_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "unknown" | "unsupported" | "incompatible" | "missing" | "opaque"
    )
}

fn dependency_artifacts_unavailable(artifacts: &[DependencyArtifactAvailability]) -> bool {
    artifacts.is_empty()
        || artifacts.iter().any(|artifact| {
            unknown_value(&artifact.package_id)
                || unknown_value(&artifact.module_path)
                || unknown_value(&artifact.artifact_kind)
                || unknown_value(&artifact.artifact_path)
                || unknown_value(&artifact.domain)
        })
}

fn dependency_compatibility_miss(
    current: Option<ProofReuseDependencyCompatibilitySnapshot>,
    cached: Option<ProofReuseDependencyCompatibilitySnapshot>,
) -> Option<ProofReuseMissReason> {
    let (Some(current), Some(cached)) = (current, cached) else {
        return Some(ProofReuseMissReason::IncompleteUpstreamPredicate);
    };

    if current.dependency_artifact_fingerprint != cached.dependency_artifact_fingerprint {
        return Some(ProofReuseMissReason::DependencyArtifactMismatch);
    }
    if current.dependency_schema_version != cached.dependency_schema_version
        || current.proof_reuse_schema_version != cached.proof_reuse_schema_version
    {
        return Some(ProofReuseMissReason::SchemaVersionMismatch);
    }
    None
}

fn canonical_diagnostic_refs(mut refs: Vec<DiagnosticRefHash>) -> Vec<DiagnosticRefHash> {
    refs.sort_by(|left, right| {
        left.diagnostic_ref_kind
            .cmp(&right.diagnostic_ref_kind)
            .then_with(|| {
                left.diagnostic_ref_hash
                    .as_bytes()
                    .cmp(right.diagnostic_ref_hash.as_bytes())
            })
    });
    refs.dedup();
    refs
}

#[cfg(test)]
mod tests;
