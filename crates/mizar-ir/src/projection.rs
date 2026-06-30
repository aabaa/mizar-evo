//! Artifact projection from sealed IR handles to stable external schemas.
//!
//! This module is specified in
//! [`projection.md`](../../../../doc/design/mizar-ir/en/projection.md).

use std::{collections::HashSet, error::Error, fmt, sync::Arc};

use mizar_artifact::{
    module_summary::{ModuleSummaryIdentity, SourceRangeSummary},
    proof_witness::{EvidenceKind, ProofStatus, ProofWitnessRef},
    registration_summary::ArtifactHashRef,
    verified_artifact::{
        ArtifactDiagnostic, BuildProvenance, DependencyArtifactHash, DiagnosticRelated,
        ExpressionMetadata, ObligationMetadata, OverloadMetadata, VerifiedArtifact,
        VerifiedArtifactError, VerifiedExport, current_schema_version, write_verified_artifact,
    },
};
use mizar_session::{BuildSnapshotId, Hash};

use crate::{
    identity::PhaseOutputId,
    publisher::{PhaseOutputPublisher, PublishError},
    storage::{AnyPhaseOutputRef, StorageError},
};

/// Projection service over sealed IR handles.
#[derive(Debug)]
pub struct ArtifactProjectionService {
    publisher: Arc<PhaseOutputPublisher>,
}

/// Input for projecting one module into an unpublished verified-artifact draft.
pub struct ProjectVerifiedArtifactInput {
    /// Target current snapshot.
    pub snapshot: BuildSnapshotId,
    /// Sealed outputs that justify this projection.
    pub required_outputs: Vec<AnyPhaseOutputRef>,
    /// Stable module identity.
    pub module: ModuleSummaryIdentity,
    /// Package- or workspace-relative source file path.
    pub source_file: String,
    /// Exact source text hash.
    pub source_hash: Hash,
    /// Optional local timestamp excluded from stable hashes.
    pub verified_at: Option<String>,
    /// Externally visible declarations and signatures.
    pub exports: Vec<VerifiedExport>,
    /// Stable source-shaped expression metadata.
    pub expressions: Vec<ExpressionMetadata>,
    /// Verification obligation metadata.
    pub obligations: Vec<ObligationMetadata>,
    /// References to published proof witness payloads.
    pub proof_witnesses: Vec<ProofWitnessRef>,
    /// Stable projected diagnostics.
    pub diagnostics: Vec<ArtifactDiagnostic>,
    /// Build provenance envelope.
    pub provenance: BuildProvenance,
}

/// Unpublished artifact candidate produced by `mizar-ir`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedArtifactDraft {
    /// Snapshot for which this draft is current.
    pub snapshot: BuildSnapshotId,
    /// Sealed output ids used as projection inputs.
    pub required_outputs: Vec<PhaseOutputId>,
    /// Stable artifact schema value. It is not published by `mizar-ir`.
    pub artifact: VerifiedArtifact,
    /// Downstream integrations deliberately not stubbed by this crate.
    pub external_dependency_gaps: Vec<ProjectionExternalDependencyGap>,
}

/// Downstream integration that remains outside `mizar-ir`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
#[non_exhaustive]
pub enum ProjectionExternalDependencyGap {
    /// Real driver build sessions are owned outside `mizar-ir`.
    DriverSession,
    /// Real diagnostic registry/renderer integration is not present here.
    DiagnosticsIntegration,
    /// Real producer typed projection payloads are not exposed yet.
    ProducerProjectionPayload,
    /// Artifact publication and manifest transactions are owned by phase 15.
    ArtifactPublicationToken,
}

/// Projection failure. No variant returns a partial draft.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProjectionError {
    /// No sealed output was supplied as projection input.
    MissingRequiredOutput,
    /// Snapshot is not eligible for current/package projection.
    Publisher {
        /// Publisher error.
        error: Box<PublishError>,
    },
    /// A required output belongs to a different snapshot.
    OutputSnapshotMismatch {
        /// Target snapshot.
        snapshot: BuildSnapshotId,
        /// Output id.
        output: PhaseOutputId,
        /// Output snapshot.
        output_snapshot: BuildSnapshotId,
    },
    /// Storage rejected a required output.
    Storage {
        /// Storage error.
        error: Box<StorageError>,
    },
    /// The same required output was supplied more than once.
    DuplicateRequiredOutput {
        /// Duplicate output id.
        output: PhaseOutputId,
    },
    /// Projected data attempted to carry raw internal IR or storage state.
    RawInternalLeak {
        /// Stable field path where the marker was found.
        field: String,
        /// Forbidden marker.
        marker: &'static str,
    },
    /// `mizar-artifact` rejected the projected schema value.
    Artifact {
        /// Artifact schema error.
        error: Box<VerifiedArtifactError>,
    },
}

impl ArtifactProjectionService {
    /// Creates a projection service over a publisher.
    pub fn new(publisher: Arc<PhaseOutputPublisher>) -> Self {
        Self { publisher }
    }

    /// Returns the wrapped publisher.
    pub fn publisher(&self) -> &Arc<PhaseOutputPublisher> {
        &self.publisher
    }

    /// Projects sealed current-snapshot outputs into an unpublished draft.
    pub fn project_module(
        &self,
        input: ProjectVerifiedArtifactInput,
    ) -> Result<VerifiedArtifactDraft, ProjectionError> {
        if input.required_outputs.is_empty() {
            return Err(ProjectionError::MissingRequiredOutput);
        }
        self.publisher
            .validate_current_snapshot(input.snapshot)
            .map_err(|error| ProjectionError::Publisher {
                error: Box::new(error),
            })?;

        let required_outputs =
            self.validate_required_outputs(input.snapshot, &input.required_outputs)?;

        let mut artifact = VerifiedArtifact {
            schema_version: current_schema_version(),
            module: input.module,
            source_file: input.source_file,
            source_hash: input.source_hash,
            verified_at: input.verified_at,
            interface_hash: zero_hash(),
            implementation_hash: zero_hash(),
            exports: input.exports,
            expressions: input.expressions,
            obligations: input.obligations,
            proof_witnesses: input.proof_witnesses,
            diagnostics: input.diagnostics,
            provenance: input.provenance,
        };
        normalize_artifact_order(&mut artifact);
        reject_raw_internal_leakage(&artifact)?;
        artifact
            .refresh_hashes()
            .map_err(|error| ProjectionError::Artifact {
                error: Box::new(error),
            })?;
        write_verified_artifact(&artifact).map_err(|error| ProjectionError::Artifact {
            error: Box::new(error),
        })?;
        let revalidated_outputs =
            self.validate_required_outputs(input.snapshot, &input.required_outputs)?;
        debug_assert_eq!(required_outputs, revalidated_outputs);

        Ok(VerifiedArtifactDraft {
            snapshot: input.snapshot,
            required_outputs,
            artifact,
            external_dependency_gaps: default_external_dependency_gaps(),
        })
    }

    fn validate_required_outputs(
        &self,
        snapshot: BuildSnapshotId,
        outputs: &[AnyPhaseOutputRef],
    ) -> Result<Vec<PhaseOutputId>, ProjectionError> {
        let mut ids = Vec::with_capacity(outputs.len());
        let mut seen = HashSet::with_capacity(outputs.len());
        for output in outputs {
            if output.snapshot() != snapshot {
                return Err(ProjectionError::OutputSnapshotMismatch {
                    snapshot,
                    output: output.output(),
                    output_snapshot: output.snapshot(),
                });
            }
            if !seen.insert(output.output()) {
                return Err(ProjectionError::DuplicateRequiredOutput {
                    output: output.output(),
                });
            }
            self.publisher
                .validate_current_output(snapshot, output)
                .map_err(|error| ProjectionError::Publisher {
                    error: Box::new(error),
                })?;
            self.publisher
                .storage()
                .validate_handle(output)
                .map_err(|error| ProjectionError::Storage {
                    error: Box::new(error),
                })?;
            ids.push(output.output());
        }
        ids.sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
        Ok(ids)
    }
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRequiredOutput => {
                formatter.write_str("artifact projection requires at least one sealed output")
            }
            Self::Publisher { error } => write!(formatter, "projection currentness error: {error}"),
            Self::OutputSnapshotMismatch {
                snapshot,
                output,
                output_snapshot,
            } => write!(
                formatter,
                "projection output `{output:?}` from `{output_snapshot:?}` cannot be used for `{snapshot:?}`"
            ),
            Self::Storage { error } => write!(formatter, "projection storage error: {error}"),
            Self::DuplicateRequiredOutput { output } => {
                write!(
                    formatter,
                    "duplicate projection required output `{output:?}`"
                )
            }
            Self::RawInternalLeak { field, marker } => write!(
                formatter,
                "projected artifact field `{field}` contains raw internal marker `{marker}`"
            ),
            Self::Artifact { error } => {
                write!(formatter, "artifact schema rejected projection: {error}")
            }
        }
    }
}

impl Error for ProjectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Publisher { error } => Some(error),
            Self::Storage { error } => Some(error),
            Self::Artifact { error } => Some(error),
            Self::MissingRequiredOutput
            | Self::OutputSnapshotMismatch { .. }
            | Self::DuplicateRequiredOutput { .. }
            | Self::RawInternalLeak { .. } => None,
        }
    }
}

fn default_external_dependency_gaps() -> Vec<ProjectionExternalDependencyGap> {
    vec![
        ProjectionExternalDependencyGap::DriverSession,
        ProjectionExternalDependencyGap::DiagnosticsIntegration,
        ProjectionExternalDependencyGap::ProducerProjectionPayload,
        ProjectionExternalDependencyGap::ArtifactPublicationToken,
    ]
}

fn zero_hash() -> Hash {
    Hash::from_bytes([0; Hash::BYTE_LEN])
}

fn normalize_artifact_order(artifact: &mut VerifiedArtifact) {
    artifact.exports.sort_by_key(export_sort_key);
    artifact
        .expressions
        .sort_by_key(|expression| (expression.expression_id.clone(), expression.source_range));
    artifact
        .obligations
        .sort_by_key(|obligation| (obligation.obligation_id.clone(), obligation.source_range));
    artifact.proof_witnesses.sort_by_key(proof_witness_sort_key);
    artifact.diagnostics.sort_by_key(diagnostic_sort_key);
    for diagnostic in &mut artifact.diagnostics {
        diagnostic.related.sort_by_key(diagnostic_related_sort_key);
    }
    artifact
        .provenance
        .dependency_artifact_hashes
        .sort_by_key(|dependency| dependency.module.clone());
}

fn export_sort_key(export: &VerifiedExport) -> (String, String, String, SourceRangeSummary) {
    (
        export.origin_id.clone(),
        export.fully_qualified_name.clone(),
        export.export_kind.clone(),
        export.source_range,
    )
}

fn proof_witness_sort_key(witness: &ProofWitnessRef) -> (String, String, u8, u8, String, String) {
    (
        witness.obligation_id.clone(),
        witness.obligation_fingerprint.to_artifact_hash_string(),
        proof_status_rank(witness.proof_status),
        evidence_kind_rank(witness.evidence_kind),
        witness.witness_path.clone(),
        witness.witness_artifact_hash.to_artifact_hash_string(),
    )
}

fn proof_status_rank(value: ProofStatus) -> u8 {
    match value {
        ProofStatus::KernelVerified => 0,
        ProofStatus::DischargedBuiltin => 1,
        _ => u8::MAX,
    }
}

fn evidence_kind_rank(value: EvidenceKind) -> u8 {
    match value {
        EvidenceKind::FormulaSubstitutionKernelEvidence => 0,
        EvidenceKind::AtpCertificate => 1,
        EvidenceKind::BuiltinCertificate => 2,
        EvidenceKind::KernelPrimitive => 3,
        _ => u8::MAX,
    }
}

fn diagnostic_sort_key(
    diagnostic: &ArtifactDiagnostic,
) -> (String, String, Option<SourceRangeSummary>) {
    (
        diagnostic.diagnostic_id.clone(),
        diagnostic.code.clone(),
        diagnostic.primary_range,
    )
}

fn diagnostic_related_sort_key(
    related: &DiagnosticRelated,
) -> (SourceRangeSummary, String, String) {
    (
        related.source_range,
        related.message_key.clone(),
        related.rendered_message.clone(),
    )
}

fn reject_raw_internal_leakage(artifact: &VerifiedArtifact) -> Result<(), ProjectionError> {
    scan_module_identity("$.module", &artifact.module)?;
    scan_string("$.source_file", &artifact.source_file)?;
    scan_optional_string("$.verified_at", artifact.verified_at.as_deref())?;
    for (index, export) in artifact.exports.iter().enumerate() {
        scan_export(&format!("$.exports[{index}]"), export)?;
    }
    for (index, expression) in artifact.expressions.iter().enumerate() {
        scan_expression(&format!("$.expressions[{index}]"), expression)?;
    }
    for (index, obligation) in artifact.obligations.iter().enumerate() {
        scan_obligation(&format!("$.obligations[{index}]"), obligation)?;
    }
    for (index, witness) in artifact.proof_witnesses.iter().enumerate() {
        scan_witness(&format!("$.proof_witnesses[{index}]"), witness)?;
    }
    for (index, diagnostic) in artifact.diagnostics.iter().enumerate() {
        scan_diagnostic(&format!("$.diagnostics[{index}]"), diagnostic)?;
    }
    scan_provenance("$.provenance", &artifact.provenance)
}

fn scan_module_identity(
    path: &str,
    identity: &ModuleSummaryIdentity,
) -> Result<(), ProjectionError> {
    scan_string(&field_path(path, "package_id"), &identity.package_id)?;
    scan_optional_string(
        &field_path(path, "package_version"),
        identity.package_version.as_deref(),
    )?;
    scan_optional_string(
        &field_path(path, "lockfile_identity"),
        identity.lockfile_identity.as_deref(),
    )?;
    scan_string(&field_path(path, "module_path"), &identity.module_path)?;
    scan_string(
        &field_path(path, "language_edition"),
        &identity.language_edition,
    )
}

fn scan_export(path: &str, export: &VerifiedExport) -> Result<(), ProjectionError> {
    scan_string(&field_path(path, "origin_id"), &export.origin_id)?;
    scan_string(
        &field_path(path, "fully_qualified_name"),
        &export.fully_qualified_name,
    )?;
    scan_string_array(&field_path(path, "namespace_path"), &export.namespace_path)?;
    scan_string(&field_path(path, "export_kind"), &export.export_kind)?;
    scan_string(
        &field_path(path, "rendered_signature"),
        &export.rendered_signature,
    )?;
    scan_hash_ref(
        &field_path(path, "interface_fingerprint"),
        &export.interface_fingerprint,
    )?;
    scan_optional_hash_ref(
        &field_path(path, "documentation_ref"),
        export.documentation_ref.as_ref(),
    )
}

fn scan_expression(path: &str, expression: &ExpressionMetadata) -> Result<(), ProjectionError> {
    scan_string(
        &field_path(path, "expression_id"),
        &expression.expression_id,
    )?;
    scan_string(
        &field_path(path, "expression_kind"),
        &expression.expression_kind,
    )?;
    scan_string(
        &field_path(path, "rendered_surface"),
        &expression.rendered_surface,
    )?;
    scan_optional_string(
        &field_path(path, "inferred_type"),
        expression.inferred_type.as_deref(),
    )?;
    scan_optional_string(
        &field_path(path, "resolved_symbol"),
        expression.resolved_symbol.as_deref(),
    )?;
    scan_string_array(
        &field_path(path, "inserted_coercions"),
        &expression.inserted_coercions,
    )?;
    scan_optional_string(
        &field_path(path, "active_thesis"),
        expression.active_thesis.as_deref(),
    )?;
    if let Some(overload) = &expression.overload_resolution {
        scan_overload(&field_path(path, "overload_resolution"), overload)?;
    }
    Ok(())
}

fn scan_overload(path: &str, overload: &OverloadMetadata) -> Result<(), ProjectionError> {
    scan_string(&field_path(path, "root_symbol"), &overload.root_symbol)?;
    scan_string(
        &field_path(path, "selected_candidate"),
        &overload.selected_candidate,
    )?;
    scan_string_array(
        &field_path(path, "active_refinements"),
        &overload.active_refinements,
    )?;
    scan_optional_string(
        &field_path(path, "coercion_summary"),
        overload.coercion_summary.as_deref(),
    )
}

fn scan_obligation(path: &str, obligation: &ObligationMetadata) -> Result<(), ProjectionError> {
    scan_string(
        &field_path(path, "obligation_id"),
        &obligation.obligation_id,
    )?;
    scan_optional_string(
        &field_path(path, "obligation_anchor"),
        obligation.obligation_anchor.as_deref(),
    )?;
    scan_optional_string(
        &field_path(path, "owner_origin_id"),
        obligation.owner_origin_id.as_deref(),
    )?;
    scan_string(
        &field_path(path, "obligation_kind"),
        &obligation.obligation_kind,
    )?;
    scan_string(
        &field_path(path, "statement_summary"),
        &obligation.statement_summary,
    )?;
    scan_hash_ref(
        &field_path(path, "obligation_fingerprint"),
        &obligation.obligation_fingerprint,
    )?;
    scan_hash_ref(
        &field_path(path, "vc_fingerprint"),
        &obligation.vc_fingerprint,
    )?;
    scan_hash_ref(
        &field_path(path, "local_context_fingerprint"),
        &obligation.local_context_fingerprint,
    )?;
    scan_hash_ref(
        &field_path(path, "dependency_slice_fingerprint"),
        &obligation.dependency_slice_fingerprint,
    )?;
    scan_hash_ref(
        &field_path(path, "verifier_policy_fingerprint"),
        &obligation.verifier_policy_fingerprint,
    )?;
    scan_optional_string(
        &field_path(path, "accepted_witness_obligation_id"),
        obligation.accepted_witness_obligation_id.as_deref(),
    )?;
    scan_optional_hash_ref(
        &field_path(path, "deterministic_discharge_hash"),
        obligation.deterministic_discharge_hash.as_ref(),
    )?;
    scan_optional_hash_ref(
        &field_path(path, "diagnostic_ref"),
        obligation.diagnostic_ref.as_ref(),
    )
}

fn scan_witness(path: &str, witness: &ProofWitnessRef) -> Result<(), ProjectionError> {
    scan_string(&field_path(path, "obligation_id"), &witness.obligation_id)?;
    scan_hash_ref(
        &field_path(path, "obligation_fingerprint"),
        &witness.obligation_fingerprint,
    )?;
    scan_string(&field_path(path, "witness_path"), &witness.witness_path)?;
    scan_hash_ref(
        &field_path(path, "witness_artifact_hash"),
        &witness.witness_artifact_hash,
    )?;
    let acceptance = &witness.kernel_acceptance;
    scan_hash_ref(
        &field_path(path, "kernel_profile_fingerprint"),
        &acceptance.kernel_profile_fingerprint,
    )?;
    scan_hash_ref(
        &field_path(path, "verifier_policy_fingerprint"),
        &acceptance.verifier_policy_fingerprint,
    )?;
    scan_hash_ref(
        &field_path(path, "target_binding_hash"),
        &acceptance.target_binding_hash,
    )?;
    scan_hash_ref(
        &field_path(path, "formula_evidence_hash"),
        &acceptance.formula_evidence_hash,
    )?;
    scan_hash_ref(
        &field_path(path, "substitution_evidence_hash"),
        &acceptance.substitution_evidence_hash,
    )?;
    scan_hash_ref(
        &field_path(path, "provenance_hash"),
        &acceptance.provenance_hash,
    )?;
    scan_optional_hash_ref(
        &field_path(path, "formula_context_hash"),
        acceptance.formula_context_hash.as_ref(),
    )?;
    scan_hash_ref(
        &field_path(path, "accepted_result_hash"),
        &acceptance.accepted_result_hash,
    )
}

fn scan_diagnostic(path: &str, diagnostic: &ArtifactDiagnostic) -> Result<(), ProjectionError> {
    scan_string(
        &field_path(path, "diagnostic_id"),
        &diagnostic.diagnostic_id,
    )?;
    scan_string(&field_path(path, "code"), &diagnostic.code)?;
    scan_string(&field_path(path, "message_key"), &diagnostic.message_key)?;
    scan_string(
        &field_path(path, "rendered_message"),
        &diagnostic.rendered_message,
    )?;
    for (index, related) in diagnostic.related.iter().enumerate() {
        scan_string(
            &field_path(
                &array_path(&field_path(path, "related"), index),
                "message_key",
            ),
            &related.message_key,
        )?;
        scan_string(
            &field_path(
                &array_path(&field_path(path, "related"), index),
                "rendered_message",
            ),
            &related.rendered_message,
        )?;
    }
    scan_optional_hash_ref(
        &field_path(path, "explanation_ref"),
        diagnostic.explanation_ref.as_ref(),
    )
}

fn scan_provenance(path: &str, provenance: &BuildProvenance) -> Result<(), ProjectionError> {
    scan_string(&field_path(path, "toolchain"), &provenance.toolchain)?;
    scan_string(
        &field_path(path, "language_edition"),
        &provenance.language_edition,
    )?;
    scan_hash_ref(
        &field_path(path, "lockfile_hash"),
        &provenance.lockfile_hash,
    )?;
    scan_hash_ref(
        &field_path(path, "verifier_config_hash"),
        &provenance.verifier_config_hash,
    )?;
    scan_optional_string(
        &field_path(path, "cache_key"),
        provenance.cache_key.as_deref(),
    )?;
    for (index, dependency) in provenance.dependency_artifact_hashes.iter().enumerate() {
        scan_dependency(
            &array_path(&field_path(path, "dependency_artifact_hashes"), index),
            dependency,
        )?;
    }
    Ok(())
}

fn scan_dependency(path: &str, dependency: &DependencyArtifactHash) -> Result<(), ProjectionError> {
    scan_module_identity(&field_path(path, "module"), &dependency.module)?;
    scan_hash_ref(
        &field_path(path, "interface_hash"),
        &dependency.interface_hash,
    )?;
    scan_optional_hash_ref(
        &field_path(path, "implementation_hash"),
        dependency.implementation_hash.as_ref(),
    )?;
    scan_optional_hash_ref(
        &field_path(path, "artifact_hash"),
        dependency.artifact_hash.as_ref(),
    )
}

fn scan_string_array(path: &str, values: &[String]) -> Result<(), ProjectionError> {
    for (index, value) in values.iter().enumerate() {
        scan_string(&array_path(path, index), value)?;
    }
    Ok(())
}

fn scan_optional_string(path: &str, value: Option<&str>) -> Result<(), ProjectionError> {
    if let Some(value) = value {
        scan_string(path, value)?;
    }
    Ok(())
}

fn scan_optional_hash_ref(
    path: &str,
    value: Option<&ArtifactHashRef>,
) -> Result<(), ProjectionError> {
    if let Some(value) = value {
        scan_hash_ref(path, value)?;
    }
    Ok(())
}

fn scan_hash_ref(path: &str, value: &ArtifactHashRef) -> Result<(), ProjectionError> {
    let schema_family_path = field_path(path, "schema_family");
    if let Some(marker) = forbidden_schema_family_marker(&value.schema_family) {
        return Err(ProjectionError::RawInternalLeak {
            field: schema_family_path,
            marker,
        });
    }
    Ok(())
}

fn scan_string(path: &str, value: &str) -> Result<(), ProjectionError> {
    if let Some(marker) = forbidden_text_marker(value) {
        return Err(ProjectionError::RawInternalLeak {
            field: path.to_owned(),
            marker,
        });
    }
    Ok(())
}

fn forbidden_text_marker(value: &str) -> Option<&'static str> {
    RAW_TEXT_MARKERS
        .iter()
        .copied()
        .find(|marker| value.contains(marker))
}

fn forbidden_schema_family_marker(value: &str) -> Option<&'static str> {
    RAW_SCHEMA_FAMILY_MARKERS
        .iter()
        .copied()
        .find(|marker| value.contains(marker))
}

const RAW_TEXT_MARKERS: &[&str] = &[
    "raw SurfaceAst",
    "SurfaceAst {",
    "SurfaceAst(",
    "SurfaceAst dump",
    "raw TypedAst",
    "TypedAst {",
    "TypedAst(",
    "TypedAst dump",
    "raw ResolvedTypedAst",
    "ResolvedTypedAst {",
    "ResolvedTypedAst(",
    "ResolvedTypedAst dump",
    "raw CoreIr",
    "CoreIr {",
    "CoreIr(",
    "CoreIr dump",
    "raw ControlFlowIr",
    "ControlFlowIr {",
    "ControlFlowIr(",
    "ControlFlowIr dump",
    "raw VcIr",
    "VcIr {",
    "VcIr(",
    "VcIr dump",
    "raw AtpProblem",
    "AtpProblem {",
    "AtpProblem(",
    "AtpProblem dump",
    "kernel-internal",
    "kernel internal proof state",
    "raw kernel state",
    "proof witness payload bytes",
    "inline witness bytes",
    "raw checker dump",
    "PhaseOutputRef<",
    "OutputSlotId",
    "StorageGeneration",
    "ContentBlobId",
    "storage slot",
];

const RAW_SCHEMA_FAMILY_MARKERS: &[&str] = &[
    "SurfaceAst",
    "TypedAst",
    "ResolvedTypedAst",
    "CoreIr",
    "ControlFlowIr",
    "VcIr",
    "AtpProblem",
    "kernel-internal",
    "OutputSlotId",
    "StorageGeneration",
    "ContentBlobId",
    "storage-slot",
];

fn field_path(base: &str, field: &str) -> String {
    format!("{base}.{field}")
}

fn array_path(base: &str, index: usize) -> String {
    format!("{base}[{index}]")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mizar_artifact::{
        module_summary::{ModuleSummaryIdentity, SourceRangeSummary},
        proof_witness::{
            KernelAcceptanceMetadata, current_schema_version as proof_witness_schema_version,
        },
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::SchemaVersion as ArtifactSchemaVersion,
        verified_artifact::{
            ArtifactDiagnostic, BuildProvenance, DependencyArtifactHash, DiagnosticRelated,
            DiagnosticSeverity, ExportProofStatus, ExportVisibility, ExpressionMetadata,
            ObligationMetadata, ObligationStatus, OverloadMetadata, VerifiedArtifactReadOptions,
            VerifiedExport, read_verified_artifact, verified_artifact_json,
        },
    };
    use mizar_session::{BuildSnapshotId, Hash};

    use super::*;
    use crate::{
        identity::{OutputKind, PipelinePhase, WorkUnit},
        publisher::{
            AllowedWorkUnit, OutputOrigin, PublicationTarget, PublishError, PublishOutputInput,
        },
        storage::{BlobDecodeError, BlobDecoder, CollectInput, IrStorageService, SchemaVersion},
    };

    #[test]
    fn projects_verified_artifact_draft_with_canonical_schema() {
        let snapshot = snapshot(1);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        let service = ArtifactProjectionService::new(publisher);

        let draft = service
            .project_module(project_input(snapshot, handle.erase()))
            .expect("projection succeeds");

        assert_eq!(draft.snapshot, snapshot);
        assert_eq!(draft.required_outputs, vec![handle.output()]);
        assert_eq!(
            draft.external_dependency_gaps,
            vec![
                ProjectionExternalDependencyGap::DriverSession,
                ProjectionExternalDependencyGap::DiagnosticsIntegration,
                ProjectionExternalDependencyGap::ProducerProjectionPayload,
                ProjectionExternalDependencyGap::ArtifactPublicationToken,
            ]
        );
        assert_eq!(draft.artifact.exports[0].origin_id, "alpha");
        assert_eq!(draft.artifact.exports[1].origin_id, "zeta");
        assert_eq!(draft.artifact.diagnostics[0].diagnostic_id, "D001");
        assert_eq!(
            draft.artifact.provenance.dependency_artifact_hashes[0]
                .module
                .module_path,
            "dep.alpha"
        );

        let json = verified_artifact_json(&draft.artifact).expect("artifact json validates");
        let read = read_verified_artifact(&json, VerifiedArtifactReadOptions::default())
            .expect("artifact reader accepts projected draft");
        assert_eq!(read, draft.artifact);
    }

    #[test]
    fn accepted_obligations_and_witnesses_are_canonicalized_and_schema_valid() {
        let snapshot = snapshot(9);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        let service = ArtifactProjectionService::new(publisher);
        let mut input = project_input(snapshot, handle.erase());
        let zeta = accepted_obligation("obl-zeta", 60);
        let alpha = accepted_obligation("obl-alpha", 50);
        input.obligations = vec![zeta.clone(), alpha.clone()];
        input.proof_witnesses = vec![witness_for(&zeta, 80), witness_for(&alpha, 70)];

        let draft = service
            .project_module(input)
            .expect("accepted witnesses project through schema");

        assert_eq!(draft.artifact.obligations[0].obligation_id, "obl-alpha");
        assert_eq!(draft.artifact.obligations[1].obligation_id, "obl-zeta");
        assert_eq!(draft.artifact.proof_witnesses[0].obligation_id, "obl-alpha");
        assert_eq!(draft.artifact.proof_witnesses[1].obligation_id, "obl-zeta");
        let json = verified_artifact_json(&draft.artifact).expect("artifact json validates");
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default())
            .expect("artifact reader accepts accepted witness projection");
    }

    #[test]
    fn raw_ir_leakage_categories_are_rejected_before_draft_returns() {
        let snapshot = snapshot(2);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        let service = ArtifactProjectionService::new(publisher);
        for marker in [
            "raw SurfaceAst",
            "raw TypedAst",
            "raw CoreIr",
            "raw ControlFlowIr",
            "raw VcIr",
            "raw AtpProblem",
            "ResolvedTypedAst dump",
            "kernel-internal",
            "proof witness payload bytes",
            "inline witness bytes",
            "raw checker dump",
            "PhaseOutputRef<",
            "OutputSlotId",
            "StorageGeneration",
            "ContentBlobId",
            "storage slot",
        ] {
            let mut input = project_input(snapshot, handle.erase());
            input.exports[0].rendered_signature = format!("{marker} marker");

            let error = service
                .project_module(input)
                .expect_err("raw IR marker is rejected");
            assert!(matches!(
                error,
                ProjectionError::RawInternalLeak { marker: found, .. }
                    if found == marker || marker.contains(found) || found.contains(marker)
            ));
        }
    }

    #[test]
    fn stable_human_text_may_name_ir_layers_without_raw_dump_shape() {
        let snapshot = snapshot(14);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        let service = ArtifactProjectionService::new(publisher);
        let mut input = project_input(snapshot, handle.erase());
        input.exports[0].rendered_signature =
            "The CoreIr chapter documents internal lowering".to_owned();

        service
            .project_module(input)
            .expect("plain human text mentioning CoreIr is not raw IR leakage");
    }

    #[test]
    fn raw_ir_leakage_in_hash_ref_schema_families_is_rejected_before_draft_returns() {
        let snapshot = snapshot(10);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        let service = ArtifactProjectionService::new(publisher);
        let mut input = project_input(snapshot, handle.erase());
        input.exports[0].interface_fingerprint =
            hash_ref(ArtifactHashClass::Interface, "CoreIr", 11);

        let error = service
            .project_module(input)
            .expect_err("raw IR marker in hash ref schema family is rejected");
        assert!(matches!(
            error,
            ProjectionError::RawInternalLeak {
                marker: "CoreIr",
                ..
            }
        ));

        let mut input = project_input(snapshot, handle.erase());
        input.provenance.lockfile_hash =
            hash_ref(ArtifactHashClass::Artifact, "kernel-internal", 12);
        let error = service
            .project_module(input)
            .expect_err("kernel marker in provenance hash ref is rejected");
        assert!(matches!(
            error,
            ProjectionError::RawInternalLeak {
                marker: "kernel-internal",
                ..
            }
        ));
    }

    #[test]
    fn wrong_snapshot_output_is_rejected() {
        let current = snapshot(3);
        let old = snapshot(4);
        let publisher = publisher(current);
        publisher.register_current_snapshot(old);
        let old_handle = publish_fixture(&publisher, old, "old");
        let service = ArtifactProjectionService::new(publisher);

        let error = service
            .project_module(project_input(current, old_handle.erase()))
            .expect_err("old output cannot project for current snapshot");
        assert!(matches!(
            error,
            ProjectionError::OutputSnapshotMismatch { snapshot, output_snapshot, .. }
                if snapshot == current && output_snapshot == old
        ));
    }

    #[test]
    fn internal_only_open_output_is_rejected_before_draft_returns() {
        let snapshot = snapshot(11);
        let publisher = publisher(snapshot);
        let handle = publish_fixture_with(
            &publisher,
            snapshot,
            "open",
            OutputOrigin::OpenBuffer,
            PublicationTarget::InternalOnly,
        );
        let service = ArtifactProjectionService::new(publisher);

        let error = service
            .project_module(project_input(snapshot, handle.erase()))
            .expect_err("open-buffer internal output cannot project");
        match error {
            ProjectionError::Publisher { error } => assert!(matches!(
                *error,
                PublishError::OutputNotCurrentPackage { snapshot: rejected, .. } if rejected == snapshot
            )),
            other => panic!("expected publisher rejection, got {other:?}"),
        }
    }

    #[test]
    fn internal_only_reseal_of_collected_current_output_is_rejected() {
        let snapshot = snapshot(15);
        let publisher = publisher(snapshot);
        let old = publish_fixture(&publisher, snapshot, "module");
        publisher.storage().collect(CollectInput {
            snapshot,
            protected_outputs: Vec::new(),
        });
        let resealed_internal = publish_fixture_with(
            &publisher,
            snapshot,
            "module",
            OutputOrigin::OpenBuffer,
            PublicationTarget::InternalOnly,
        );
        assert_eq!(old.output(), resealed_internal.output());
        assert_ne!(old.any().generation(), resealed_internal.any().generation());
        let service = ArtifactProjectionService::new(publisher);

        let error = service
            .project_module(project_input(snapshot, resealed_internal.erase()))
            .expect_err("internal-only reseal must not inherit current root from old generation");
        match error {
            ProjectionError::Publisher { error } => assert!(matches!(
                *error,
                PublishError::OutputNotCurrentPackage { snapshot: rejected, .. } if rejected == snapshot
            )),
            other => panic!("expected publisher rejection, got {other:?}"),
        }
    }

    #[test]
    fn duplicate_required_outputs_fail_projection() {
        let snapshot = snapshot(12);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        let service = ArtifactProjectionService::new(publisher);
        let mut input = project_input(snapshot, handle.erase());
        input.required_outputs.push(handle.erase());

        assert!(matches!(
            service.project_module(input),
            Err(ProjectionError::DuplicateRequiredOutput { .. })
        ));
    }

    #[test]
    fn duplicate_projected_artifact_ids_fail_projection() {
        let snapshot = snapshot(13);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        let service = ArtifactProjectionService::new(publisher);
        let mut input = project_input(snapshot, handle.erase());
        input.exports[1].origin_id = input.exports[0].origin_id.clone();

        assert!(matches!(
            service.project_module(input),
            Err(ProjectionError::Artifact { .. })
        ));
    }

    #[test]
    fn obsolete_snapshot_is_rejected_as_current_projection() {
        let snapshot = snapshot(5);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        publisher
            .mark_obsolete(snapshot)
            .expect("snapshot marked obsolete");
        let service = ArtifactProjectionService::new(publisher);

        let error = service
            .project_module(project_input(snapshot, handle.erase()))
            .expect_err("obsolete snapshot cannot publish current draft");
        assert!(matches!(error, ProjectionError::Publisher { .. }));
    }

    #[test]
    fn unsealed_outputs_cannot_enter_projection_input() {
        let source = include_str!("projection.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("projection source has production section");

        assert!(production_source.contains("pub required_outputs: Vec<AnyPhaseOutputRef>"));
        assert!(
            !production_source.contains("PendingOutputSlot"),
            "projection input must not expose unsealed pending slots"
        );
    }

    #[test]
    fn stale_generation_handles_are_storage_private_and_validated_by_storage_gate() {
        let source = include_str!("projection.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("projection source has production section");

        assert!(production_source.contains("pub required_outputs: Vec<AnyPhaseOutputRef>"));
        assert!(
            production_source.contains(".validate_handle(output)"),
            "projection must use storage validation so stale-generation handles fail closed"
        );
        assert!(
            !production_source.contains("StorageGeneration(")
                && !production_source.contains("StorageGeneration::"),
            "projection must not construct or reinterpret stale storage generations"
        );
    }

    #[test]
    fn collected_output_is_rejected_before_artifact_build() {
        let snapshot = snapshot(6);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        let erased = handle.erase();
        publisher.storage().collect(CollectInput {
            snapshot,
            protected_outputs: Vec::new(),
        });
        let service = ArtifactProjectionService::new(publisher);

        let error = service
            .project_module(project_input(snapshot, erased))
            .expect_err("collected handle cannot project");
        assert!(matches!(error, ProjectionError::Storage { .. }));
    }

    #[test]
    fn witness_consistency_is_delegated_without_proof_authority() {
        let snapshot = snapshot(7);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        let service = ArtifactProjectionService::new(publisher);
        let mut input = project_input(snapshot, handle.erase());
        input.obligations[0].status = ObligationStatus::Accepted;
        input.obligations[0].accepted_witness_obligation_id = Some("obl-zeta".to_owned());
        input.obligations[0].deterministic_discharge_hash = None;

        let error = service
            .project_module(input)
            .expect_err("artifact schema rejects accepted obligation with no witness");
        assert!(matches!(error, ProjectionError::Artifact { .. }));
    }

    #[test]
    fn missing_required_output_is_rejected() {
        let snapshot = snapshot(8);
        let publisher = publisher(snapshot);
        let handle = publish_fixture(&publisher, snapshot, "module");
        let service = ArtifactProjectionService::new(publisher);
        let mut input = project_input(snapshot, handle.erase());
        input.required_outputs.clear();

        assert!(matches!(
            service.project_module(input),
            Err(ProjectionError::MissingRequiredOutput)
        ));
    }

    #[test]
    fn projection_does_not_expose_cache_or_proof_authority_markers() {
        let source = include_str!("projection.rs");
        let production_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("projection source has production section");
        for forbidden in [
            "CacheKeyBuilder",
            "CacheKey",
            "CacheRecord",
            "CacheLookupOutcome",
            "CacheMiss",
            "DependencyFingerprint",
            "mizar_cache",
            "mizar_build",
            "mizar-build",
            "trusted_status",
            "kernel_acceptance_decision",
            "mizar_driver",
        ] {
            assert!(
                !production_source.contains(forbidden),
                "projection must not introduce authority marker {forbidden}"
            );
        }
    }

    fn publisher(snapshot: BuildSnapshotId) -> Arc<PhaseOutputPublisher> {
        let storage = Arc::new(IrStorageService::new());
        let registry = Arc::new(crate::identity::SnapshotHandleRegistry::new());
        let publisher = Arc::new(PhaseOutputPublisher::new(storage, registry));
        publisher.register_current_snapshot(snapshot);
        for work_unit in ["module", "old", "open"] {
            publisher.allow_work_unit(AllowedWorkUnit::new(
                phase(),
                output_kind(),
                WorkUnit::new(work_unit),
            ));
        }
        publisher
    }

    fn publish_fixture(
        publisher: &Arc<PhaseOutputPublisher>,
        snapshot: BuildSnapshotId,
        work_unit: &str,
    ) -> crate::storage::PhaseOutputRef<String> {
        publish_fixture_with(
            publisher,
            snapshot,
            work_unit,
            OutputOrigin::PackageSource,
            PublicationTarget::CurrentPackage,
        )
    }

    fn publish_fixture_with(
        publisher: &Arc<PhaseOutputPublisher>,
        snapshot: BuildSnapshotId,
        work_unit: &str,
        origin: OutputOrigin,
        target: PublicationTarget,
    ) -> crate::storage::PhaseOutputRef<String> {
        let bytes = format!("payload:{work_unit}").into_bytes();
        let slot = publisher.allocate::<String>(
            snapshot,
            phase(),
            WorkUnit::new(work_unit),
            output_kind(),
            storage_schema(),
        );
        publisher
            .publish(PublishOutputInput {
                slot,
                snapshot,
                phase: phase(),
                work_unit: WorkUnit::new(work_unit),
                output_kind: output_kind(),
                schema_version: storage_schema(),
                payload: String::from_utf8(bytes.clone()).expect("fixture bytes are utf8"),
                canonical_payload: Some(bytes),
                decode: BlobDecoder::new(|bytes| {
                    String::from_utf8(bytes.to_vec())
                        .map_err(|error| BlobDecodeError::new(error.to_string()))
                }),
                parents: Vec::new(),
                named_input_hashes: Vec::new(),
                side_tables: Default::default(),
                origin,
                target,
            })
            .expect("fixture output publishes")
    }

    fn project_input(
        snapshot: BuildSnapshotId,
        output: AnyPhaseOutputRef,
    ) -> ProjectVerifiedArtifactInput {
        ProjectVerifiedArtifactInput {
            snapshot,
            required_outputs: vec![output],
            module: module("main"),
            source_file: "src/main.miz".to_owned(),
            source_hash: hash(1),
            verified_at: None,
            exports: vec![export("zeta", 40), export("alpha", 10)],
            expressions: vec![expression("expr-zeta", 30), expression("expr-alpha", 20)],
            obligations: vec![obligation("obl-zeta", ObligationStatus::NotRequired, 50)],
            proof_witnesses: Vec::new(),
            diagnostics: vec![diagnostic("D010", 80), diagnostic("D001", 70)],
            provenance: provenance(),
        }
    }

    fn export(origin: &str, seed: u8) -> VerifiedExport {
        VerifiedExport {
            origin_id: origin.to_owned(),
            fully_qualified_name: format!("Mizar.Test::{origin}"),
            namespace_path: vec!["Mizar".to_owned(), "Test".to_owned()],
            visibility: ExportVisibility::Public,
            export_kind: "theorem".to_owned(),
            source_range: range(seed),
            rendered_signature: format!("theorem {origin}: x = x"),
            interface_fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-ir/export", seed),
            proof_status: Some(ExportProofStatus::NotRequired),
            documentation_ref: Some(hash_ref(
                ArtifactHashClass::Diagnostic,
                "mizar-doc/section",
                seed + 1,
            )),
        }
    }

    fn expression(id: &str, seed: u8) -> ExpressionMetadata {
        ExpressionMetadata {
            expression_id: id.to_owned(),
            source_range: range(seed),
            expression_kind: "term".to_owned(),
            rendered_surface: format!("{id} surface"),
            inferred_type: Some("set".to_owned()),
            resolved_symbol: Some("Mizar.Test::x".to_owned()),
            inserted_coercions: vec!["identity-coercion".to_owned()],
            active_thesis: Some("x = x".to_owned()),
            overload_resolution: Some(OverloadMetadata {
                root_symbol: "equals".to_owned(),
                selected_candidate: "set-equality".to_owned(),
                active_refinements: vec!["set".to_owned()],
                coercion_summary: None,
            }),
        }
    }

    fn obligation(id: &str, status: ObligationStatus, seed: u8) -> ObligationMetadata {
        ObligationMetadata {
            obligation_id: id.to_owned(),
            obligation_anchor: Some(format!("{id}:anchor")),
            owner_origin_id: Some("zeta".to_owned()),
            source_range: range(seed),
            obligation_kind: "theorem-body".to_owned(),
            statement_summary: "x = x".to_owned(),
            obligation_fingerprint: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-proof/obligation",
                seed,
            ),
            vc_fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-vc/vc", seed + 1),
            local_context_fingerprint: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-vc/local-context",
                seed + 2,
            ),
            dependency_slice_fingerprint: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-proof/dependency-slice",
                seed + 3,
            ),
            verifier_policy_fingerprint: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-proof/policy",
                seed + 4,
            ),
            status,
            accepted_witness_obligation_id: None,
            deterministic_discharge_hash: Some(hash_ref(
                ArtifactHashClass::Interface,
                "mizar-proof/no-witness-discharge",
                seed + 5,
            )),
            diagnostic_ref: Some(hash_ref(
                ArtifactHashClass::Diagnostic,
                "mizar-diagnostic/obligation",
                seed + 6,
            )),
        }
    }

    fn accepted_obligation(id: &str, seed: u8) -> ObligationMetadata {
        let mut obligation = obligation(id, ObligationStatus::Accepted, seed);
        obligation.accepted_witness_obligation_id = Some(id.to_owned());
        obligation.deterministic_discharge_hash = None;
        obligation
    }

    fn witness_for(obligation: &ObligationMetadata, seed: u8) -> ProofWitnessRef {
        ProofWitnessRef {
            schema_version: proof_witness_schema_version(),
            obligation_id: obligation.obligation_id.clone(),
            obligation_fingerprint: obligation.obligation_fingerprint.clone(),
            proof_status: ProofStatus::KernelVerified,
            evidence_kind: EvidenceKind::FormulaSubstitutionKernelEvidence,
            witness_path: format!("proof-witnesses/{}.json", obligation.obligation_id),
            witness_artifact_hash: hash_ref(
                ArtifactHashClass::Artifact,
                "mizar-proof/witness",
                seed,
            ),
            kernel_acceptance: KernelAcceptanceMetadata {
                kernel_profile_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/profile",
                    seed + 1,
                ),
                verifier_policy_fingerprint: obligation.verifier_policy_fingerprint.clone(),
                checker_schema_version: ArtifactSchemaVersion::new(1, 0),
                evidence_schema_version: ArtifactSchemaVersion::new(1, 0),
                target_binding_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/target-binding",
                    seed + 2,
                ),
                formula_evidence_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/formula",
                    seed + 3,
                ),
                substitution_evidence_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/substitution",
                    seed + 4,
                ),
                provenance_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/provenance",
                    seed + 5,
                ),
                formula_context_hash: Some(hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/formula-context",
                    seed + 6,
                )),
                accepted_result_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/result",
                    seed + 7,
                ),
            },
        }
    }

    fn diagnostic(id: &str, seed: u8) -> ArtifactDiagnostic {
        ArtifactDiagnostic {
            diagnostic_id: id.to_owned(),
            code: format!("MIZ{seed:03}"),
            severity: DiagnosticSeverity::Info,
            primary_range: Some(range(seed)),
            message_key: format!("diagnostic.{id}"),
            rendered_message: format!("diagnostic {id}"),
            related: vec![DiagnosticRelated {
                source_range: range(seed + 1),
                message_key: format!("diagnostic.{id}.related"),
                rendered_message: format!("related {id}"),
            }],
            explanation_ref: Some(hash_ref(
                ArtifactHashClass::Diagnostic,
                "mizar-diagnostic/explanation",
                seed + 2,
            )),
        }
    }

    fn provenance() -> BuildProvenance {
        BuildProvenance {
            toolchain: "mizar-2026.1".to_owned(),
            language_edition: "2026".to_owned(),
            lockfile_hash: hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 90),
            verifier_config_hash: hash_ref(ArtifactHashClass::Interface, "mizar-proof/policy", 91),
            dependency_artifact_hashes: vec![
                dependency("dep.zeta", 93),
                dependency("dep.alpha", 92),
            ],
            cache_key: Some("upstream-cache-key-only".to_owned()),
        }
    }

    fn dependency(module_path: &str, seed: u8) -> DependencyArtifactHash {
        DependencyArtifactHash {
            module: module(module_path),
            interface_hash: hash_ref(ArtifactHashClass::Interface, "mizar-artifact/module", seed),
            implementation_hash: Some(hash_ref(
                ArtifactHashClass::Implementation,
                "mizar-artifact/module",
                seed + 1,
            )),
            artifact_hash: Some(hash_ref(
                ArtifactHashClass::Artifact,
                "mizar-artifact/file",
                seed + 2,
            )),
        }
    }

    fn module(module_path: &str) -> ModuleSummaryIdentity {
        ModuleSummaryIdentity {
            package_id: "mml".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("lock:fixture".to_owned()),
            module_path: module_path.to_owned(),
            language_edition: "2026".to_owned(),
        }
    }

    fn hash_ref(class: ArtifactHashClass, family: &str, seed: u8) -> ArtifactHashRef {
        ArtifactHashRef::new(class, family, ArtifactSchemaVersion::new(1, 0), hash(seed))
    }

    fn range(seed: u8) -> SourceRangeSummary {
        SourceRangeSummary {
            start_byte: u64::from(seed),
            end_byte: u64::from(seed) + 3,
        }
    }

    fn phase() -> PipelinePhase {
        PipelinePhase::new("projection-fixture")
    }

    fn output_kind() -> OutputKind {
        OutputKind::new("projection-fixture-output")
    }

    fn storage_schema() -> SchemaVersion {
        SchemaVersion::new(1)
    }

    fn snapshot(seed: u8) -> BuildSnapshotId {
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            hex_byte(seed)
        ))
        .expect("test snapshot id is valid")
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }

    fn hex_byte(seed: u8) -> String {
        format!("{seed:02x}").repeat(Hash::BYTE_LEN)
    }
}
