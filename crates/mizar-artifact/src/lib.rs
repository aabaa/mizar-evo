//! Stable artifact schemas, canonical bytes, and manifest publication.
//!
//! `mizar-artifact` owns published projection schemas and file publication
//! helpers. It does not own cache trust, producer phase execution, or proof
//! acceptance.

/// Manifest schema and manifest transaction helpers.
pub mod manifest;

/// Resolver-facing module summary schema.
pub mod module_summary;

/// Proof witness reference schema.
pub mod proof_witness;

/// Checker-facing registration summary schema.
pub mod registration_summary;

/// Canonical JSON, hash helpers, and artifact store writes.
pub mod store;

/// Published verified module artifact schema.
pub mod verified_artifact;

pub use manifest::{ArtifactManifest, ManifestTransaction, ModuleArtifactEntry, ProofWitnessEntry};
pub use module_summary::{ExportedItem, ExportedLabel, ImportedInterface, ModuleSummary};
pub use proof_witness::ProofWitnessRef;
pub use registration_summary::{ClusterTraceRef, ExportedRegistration, RegistrationSummary};
pub use store::{
    ArtifactError, ArtifactStore, CanonicalJson, StoredBlob, canonical_hash, content_hash_str,
    hash_from_hex, hash_to_hex,
};
pub use verified_artifact::{
    ArtifactExport, ArtifactRef, BuildProvenance, DependencyArtifactHash, ExpressionMetadata,
    MetadataField, ObligationMetadata, ProjectedDiagnostic, VerifiedArtifact,
};
