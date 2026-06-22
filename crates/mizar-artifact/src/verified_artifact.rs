//! Published `VerifiedArtifact` schema, canonical writer, and validating reader.
//!
//! The schema is specified in
//! [verified_artifact.md](../../../../doc/design/mizar-artifact/en/verified_artifact.md).

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use mizar_session::Hash;

use crate::{
    module_summary::{ModuleSummaryIdentity, SOURCE_HASH_CONSTRUCTION, SourceRangeSummary},
    proof_witness::{
        EvidenceKind, ProofStatus as WitnessProofStatus, ProofWitnessError,
        ProofWitnessReadOptions, ProofWitnessRef, proof_witness_ref_json, read_proof_witness_ref,
    },
    registration_summary::{ArtifactHashClass, ArtifactHashRef},
    store::{
        ARTIFACT_HASH_CONSTRUCTION, CanonicalHashDomain, CanonicalJson, CanonicalJsonError,
        HashClass, MinorVersionPolicy, SchemaVersion, SchemaVersionError, SchemaVersionSupport,
        canonical_json_bytes,
    },
};

/// Schema family used by verified artifact projections.
pub const VERIFIED_ARTIFACT_SCHEMA_FAMILY: &str = "mizar-artifact/verified-artifact";

/// Stable published projection for one verified source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedArtifact {
    /// Schema version read from or written to the artifact.
    pub schema_version: SchemaVersion,
    /// Stable package/module identity.
    pub module: ModuleSummaryIdentity,
    /// Package- or workspace-relative source path.
    pub source_file: String,
    /// Exact source text hash.
    pub source_hash: Hash,
    /// Optional user-visible local timestamp, excluded from stable hashes.
    pub verified_at: Option<String>,
    /// Recomputed importer-visible interface hash.
    pub interface_hash: Hash,
    /// Recomputed full stable projection hash.
    pub implementation_hash: Hash,
    /// Externally visible declarations and signatures.
    pub exports: Vec<VerifiedExport>,
    /// Stable source-shaped expression metadata.
    pub expressions: Vec<ExpressionMetadata>,
    /// Verification obligations projected from VC/proof phases.
    pub obligations: Vec<ObligationMetadata>,
    /// References to published proof witness payloads.
    pub proof_witnesses: Vec<ProofWitnessRef>,
    /// Stable projected diagnostics.
    pub diagnostics: Vec<ArtifactDiagnostic>,
    /// Build provenance envelope.
    pub provenance: BuildProvenance,
}

/// One externally visible export in a verified artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedExport {
    /// Stable origin id for this exported surface element.
    pub origin_id: String,
    /// Fully qualified exported name.
    pub fully_qualified_name: String,
    /// Exported namespace path.
    pub namespace_path: Vec<String>,
    /// Export visibility.
    pub visibility: ExportVisibility,
    /// Producer-owned export kind.
    pub export_kind: String,
    /// Diagnostic/navigation source range.
    pub source_range: SourceRangeSummary,
    /// Rendered importer-visible signature.
    pub rendered_signature: String,
    /// Producer-owned dependency-facing export fingerprint.
    pub interface_fingerprint: ArtifactHashRef,
    /// Projected proof status when importer-visible.
    pub proof_status: Option<ExportProofStatus>,
    /// Optional documentation or diagnostic payload reference.
    pub documentation_ref: Option<ArtifactHashRef>,
}

/// Export visibility represented in a verified artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ExportVisibility {
    /// Public export.
    Public,
    /// Re-exported item.
    Reexported,
}

/// Importer-visible proof status for an export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ExportProofStatus {
    /// The exported proof obligation is accepted.
    Accepted,
    /// The exported proof obligation is not accepted.
    NotAccepted,
    /// The export does not require proof acceptance.
    NotRequired,
}

/// Stable source-shaped expression metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionMetadata {
    /// Deterministic expression id.
    pub expression_id: String,
    /// Diagnostic/navigation source range.
    pub source_range: SourceRangeSummary,
    /// Producer-owned expression kind.
    pub expression_kind: String,
    /// Stable rendered surface text.
    pub rendered_surface: String,
    /// Stable rendered inferred type when available.
    pub inferred_type: Option<String>,
    /// Stable rendered resolved symbol when available.
    pub resolved_symbol: Option<String>,
    /// Stable summaries of inserted coercions.
    pub inserted_coercions: Vec<String>,
    /// Stable rendered active thesis when available.
    pub active_thesis: Option<String>,
    /// Optional stable overload resolution summary.
    pub overload_resolution: Option<OverloadMetadata>,
}

/// Stable overload resolution metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadMetadata {
    /// Root overloaded symbol.
    pub root_symbol: String,
    /// Selected candidate summary.
    pub selected_candidate: String,
    /// Active refinement summaries.
    pub active_refinements: Vec<String>,
    /// Optional coercion summary.
    pub coercion_summary: Option<String>,
}

/// Verification obligation metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationMetadata {
    /// Stable proof obligation id.
    pub obligation_id: String,
    /// Best-effort cross-edit identity.
    pub obligation_anchor: Option<String>,
    /// Owning export origin id when one exists.
    pub owner_origin_id: Option<String>,
    /// Diagnostic/navigation source range.
    pub source_range: SourceRangeSummary,
    /// Producer-owned obligation kind.
    pub obligation_kind: String,
    /// Stable rendered statement summary.
    pub statement_summary: String,
    /// Composite proof-reuse input fingerprint.
    pub obligation_fingerprint: ArtifactHashRef,
    /// VC semantic fingerprint.
    pub vc_fingerprint: ArtifactHashRef,
    /// Local proof context fingerprint.
    pub local_context_fingerprint: ArtifactHashRef,
    /// Dependency slice fingerprint.
    pub dependency_slice_fingerprint: ArtifactHashRef,
    /// Verifier policy fingerprint.
    pub verifier_policy_fingerprint: ArtifactHashRef,
    /// Projected obligation status.
    pub status: ObligationStatus,
    /// Accepted witness obligation id when a trusted witness is required.
    pub accepted_witness_obligation_id: Option<String>,
    /// Optional deterministic no-witness discharge hash for not-required entries.
    pub deterministic_discharge_hash: Option<ArtifactHashRef>,
    /// Optional diagnostic payload reference.
    pub diagnostic_ref: Option<ArtifactHashRef>,
}

/// Verification status for an obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ObligationStatus {
    /// Accepted by a trusted witness reference.
    Accepted,
    /// Still open.
    Open,
    /// Rejected.
    Rejected,
    /// Externally attested, not kernel accepted.
    ExternallyAttested,
    /// No proof obligation is required.
    NotRequired,
}

/// Stable projected diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactDiagnostic {
    /// Stable diagnostic id.
    pub diagnostic_id: String,
    /// Stable diagnostic code.
    pub code: String,
    /// Diagnostic severity.
    pub severity: DiagnosticSeverity,
    /// Optional primary diagnostic range.
    pub primary_range: Option<SourceRangeSummary>,
    /// Stable message key.
    pub message_key: String,
    /// Human-readable rendered message.
    pub rendered_message: String,
    /// Related diagnostic locations.
    pub related: Vec<DiagnosticRelated>,
    /// Optional structured explanation payload reference.
    pub explanation_ref: Option<ArtifactHashRef>,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum DiagnosticSeverity {
    /// Error diagnostic.
    Error,
    /// Warning diagnostic.
    Warning,
    /// Informational diagnostic.
    Info,
    /// Hint diagnostic.
    Hint,
}

/// One related diagnostic location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticRelated {
    /// Related source range.
    pub source_range: SourceRangeSummary,
    /// Stable message key.
    pub message_key: String,
    /// Human-readable rendered message.
    pub rendered_message: String,
}

/// Build provenance envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildProvenance {
    /// Toolchain identifier.
    pub toolchain: String,
    /// Language edition identifier.
    pub language_edition: String,
    /// Lockfile hash reference.
    pub lockfile_hash: ArtifactHashRef,
    /// Verifier configuration hash reference.
    pub verifier_config_hash: ArtifactHashRef,
    /// Dependency artifact hashes that affected this artifact.
    pub dependency_artifact_hashes: Vec<DependencyArtifactHash>,
    /// Optional opaque cache key, excluded from stable hashes.
    pub cache_key: Option<String>,
}

/// Dependency artifact hashes recorded in provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyArtifactHash {
    /// Dependency module identity.
    pub module: ModuleSummaryIdentity,
    /// Dependency interface hash.
    pub interface_hash: ArtifactHashRef,
    /// Optional dependency implementation hash.
    pub implementation_hash: Option<ArtifactHashRef>,
    /// Optional dependency artifact byte hash.
    pub artifact_hash: Option<ArtifactHashRef>,
}

/// Additional validation requested by a caller while reading a verified artifact.
#[derive(Debug, Clone, Copy, Default)]
pub struct VerifiedArtifactReadOptions<'a> {
    /// Artifact path to include in schema-version diagnostics.
    pub artifact_path: Option<&'a str>,
    /// Expected module identity from the manifest or import request.
    pub expected_module: Option<&'a ModuleSummaryIdentity>,
    /// Expected interface hash from the manifest or import request.
    pub expected_interface_hash: Option<Hash>,
    /// Expected implementation hash from the manifest or import request.
    pub expected_implementation_hash: Option<Hash>,
}

/// Errors produced by the verified artifact schema.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VerifiedArtifactError {
    /// Canonical JSON object construction failed.
    CanonicalJson(CanonicalJsonError),
    /// Schema-version compatibility failed.
    SchemaVersion(SchemaVersionError),
    /// A nested proof witness reference is invalid.
    ProofWitness {
        /// Path of the nested proof witness reference.
        path: String,
        /// Underlying proof witness error.
        error: ProofWitnessError,
    },
    /// A required field is missing.
    MissingField { path: String },
    /// An unknown field was present.
    UnknownField { path: String, field: String },
    /// A field had the wrong JSON type.
    UnexpectedType {
        path: String,
        expected: &'static str,
    },
    /// A field value failed schema validation.
    InvalidField { path: String, reason: String },
    /// A serialized hash string is malformed or has the wrong domain.
    InvalidHash { path: String, reason: String },
    /// A collection is not in canonical order.
    UnsortedCollection { path: String },
    /// A collection contains a duplicate identity key.
    DuplicateEntry { path: String, key: String },
    /// The stored interface hash does not match the recomputed projection hash.
    InterfaceHashMismatch { expected: String, actual: String },
    /// The stored implementation hash does not match the recomputed projection hash.
    ImplementationHashMismatch { expected: String, actual: String },
    /// The caller-provided expected interface hash does not match the artifact.
    ExpectedInterfaceHashMismatch { expected: String, actual: String },
    /// The caller-provided expected implementation hash does not match the artifact.
    ExpectedImplementationHashMismatch { expected: String, actual: String },
    /// The caller-provided expected module identity does not match the artifact.
    ModuleIdentityMismatch { expected: String, actual: String },
    /// An obligation and proof witness reference are not consistent.
    WitnessReferenceMismatch { path: String, reason: String },
}

/// Returns the current verified artifact schema version.
pub const fn current_schema_version() -> SchemaVersion {
    SchemaVersion::new(1, 0)
}

/// Returns the supported verified artifact schema-version range.
pub fn schema_version_support() -> SchemaVersionSupport {
    SchemaVersionSupport::new(
        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
        current_schema_version().major(),
        current_schema_version().minor(),
        MinorVersionPolicy::UpToSupported,
    )
}

/// Serializes a verified artifact to canonical UTF-8 JSON bytes.
pub fn write_verified_artifact(
    artifact: &VerifiedArtifact,
) -> Result<Vec<u8>, VerifiedArtifactError> {
    verified_artifact_json(artifact).map(|json| canonical_json_bytes(&json))
}

/// Builds the canonical JSON value for a verified artifact.
pub fn verified_artifact_json(
    artifact: &VerifiedArtifact,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    validate_artifact(artifact)?;
    verified_artifact_json_unchecked(artifact)
}

/// Reads and validates a verified artifact from a canonical JSON value.
pub fn read_verified_artifact(
    value: &CanonicalJson,
    options: VerifiedArtifactReadOptions<'_>,
) -> Result<VerifiedArtifact, VerifiedArtifactError> {
    let fields = expect_object(value, "$")?;
    let schema_version = read_schema_version(fields, options.artifact_path)?;
    reject_unknown_fields(
        fields,
        &[
            "schema_version",
            "module",
            "source_file",
            "source_hash",
            "verified_at",
            "interface_hash",
            "implementation_hash",
            "exports",
            "expressions",
            "obligations",
            "proof_witnesses",
            "diagnostics",
            "provenance",
        ],
        "$",
    )?;

    let artifact = VerifiedArtifact {
        schema_version,
        module: read_identity(required_field(fields, "module", "$")?, "$.module")?,
        source_file: read_required_string(fields, "source_file", "$")?,
        source_hash: read_source_hash(
            required_field(fields, "source_hash", "$")?,
            "$.source_hash",
        )?,
        verified_at: read_optional_verified_at(fields, "verified_at", "$")?,
        interface_hash: read_verified_artifact_hash(
            required_field(fields, "interface_hash", "$")?,
            "$.interface_hash",
            ArtifactHashClass::Interface,
            schema_version,
        )?,
        implementation_hash: read_verified_artifact_hash(
            required_field(fields, "implementation_hash", "$")?,
            "$.implementation_hash",
            ArtifactHashClass::Implementation,
            schema_version,
        )?,
        exports: read_exports(required_field(fields, "exports", "$")?, "$.exports")?,
        expressions: read_expressions(
            required_field(fields, "expressions", "$")?,
            "$.expressions",
        )?,
        obligations: read_obligations(
            required_field(fields, "obligations", "$")?,
            "$.obligations",
        )?,
        proof_witnesses: read_proof_witnesses(
            required_field(fields, "proof_witnesses", "$")?,
            "$.proof_witnesses",
        )?,
        diagnostics: read_diagnostics(
            required_field(fields, "diagnostics", "$")?,
            "$.diagnostics",
        )?,
        provenance: read_provenance(required_field(fields, "provenance", "$")?, "$.provenance")?,
    };

    validate_artifact_shape(&artifact)?;
    let interface_hash = artifact.compute_interface_hash()?;
    if interface_hash != artifact.interface_hash {
        return Err(VerifiedArtifactError::InterfaceHashMismatch {
            expected: verified_artifact_hash_string(
                ArtifactHashClass::Interface,
                schema_version,
                interface_hash,
            ),
            actual: verified_artifact_hash_string(
                ArtifactHashClass::Interface,
                schema_version,
                artifact.interface_hash,
            ),
        });
    }
    let implementation_hash = artifact.compute_implementation_hash()?;
    if implementation_hash != artifact.implementation_hash {
        return Err(VerifiedArtifactError::ImplementationHashMismatch {
            expected: verified_artifact_hash_string(
                ArtifactHashClass::Implementation,
                schema_version,
                implementation_hash,
            ),
            actual: verified_artifact_hash_string(
                ArtifactHashClass::Implementation,
                schema_version,
                artifact.implementation_hash,
            ),
        });
    }

    if let Some(expected_module) = options.expected_module
        && expected_module != &artifact.module
    {
        return Err(VerifiedArtifactError::ModuleIdentityMismatch {
            expected: identity_display(expected_module),
            actual: identity_display(&artifact.module),
        });
    }

    if let Some(expected_hash) = options.expected_interface_hash
        && expected_hash != artifact.interface_hash
    {
        return Err(VerifiedArtifactError::ExpectedInterfaceHashMismatch {
            expected: verified_artifact_hash_string(
                ArtifactHashClass::Interface,
                schema_version,
                expected_hash,
            ),
            actual: verified_artifact_hash_string(
                ArtifactHashClass::Interface,
                schema_version,
                artifact.interface_hash,
            ),
        });
    }

    if let Some(expected_hash) = options.expected_implementation_hash
        && expected_hash != artifact.implementation_hash
    {
        return Err(VerifiedArtifactError::ExpectedImplementationHashMismatch {
            expected: verified_artifact_hash_string(
                ArtifactHashClass::Implementation,
                schema_version,
                expected_hash,
            ),
            actual: verified_artifact_hash_string(
                ArtifactHashClass::Implementation,
                schema_version,
                artifact.implementation_hash,
            ),
        });
    }

    Ok(artifact)
}

impl VerifiedArtifact {
    /// Computes the importer-visible interface hash for this artifact.
    pub fn compute_interface_hash(&self) -> Result<Hash, VerifiedArtifactError> {
        let projection = interface_projection_json(self)?;
        let domain = CanonicalHashDomain::new(
            HashClass::Interface,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            self.schema_version,
        );
        Ok(domain.hash(&projection, &[]))
    }

    /// Computes the full stable implementation hash for this artifact.
    pub fn compute_implementation_hash(&self) -> Result<Hash, VerifiedArtifactError> {
        let projection = implementation_projection_json(self)?;
        let domain = CanonicalHashDomain::new(
            HashClass::Implementation,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            self.schema_version,
        );
        Ok(domain.hash(&projection, &[]))
    }

    /// Recomputes and stores both top-level hashes.
    pub fn refresh_hashes(&mut self) -> Result<(Hash, Hash), VerifiedArtifactError> {
        let interface_hash = self.compute_interface_hash()?;
        self.interface_hash = interface_hash;
        let implementation_hash = self.compute_implementation_hash()?;
        self.implementation_hash = implementation_hash;
        Ok((interface_hash, implementation_hash))
    }
}

impl ExportVisibility {
    fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Reexported => "reexported",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "public" => Some(Self::Public),
            "reexported" => Some(Self::Reexported),
            _ => None,
        }
    }
}

impl ExportProofStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::NotAccepted => "not_accepted",
            Self::NotRequired => "not_required",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "accepted" => Some(Self::Accepted),
            "not_accepted" => Some(Self::NotAccepted),
            "not_required" => Some(Self::NotRequired),
            _ => None,
        }
    }
}

impl ObligationStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Open => "open",
            Self::Rejected => "rejected",
            Self::ExternallyAttested => "externally_attested",
            Self::NotRequired => "not_required",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "accepted" => Some(Self::Accepted),
            "open" => Some(Self::Open),
            "rejected" => Some(Self::Rejected),
            "externally_attested" => Some(Self::ExternallyAttested),
            "not_required" => Some(Self::NotRequired),
            _ => None,
        }
    }
}

impl DiagnosticSeverity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
            Self::Hint => "hint",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "error" => Some(Self::Error),
            "warning" => Some(Self::Warning),
            "info" => Some(Self::Info),
            "hint" => Some(Self::Hint),
            _ => None,
        }
    }
}

impl fmt::Display for VerifiedArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CanonicalJson(error) => write!(formatter, "{error}"),
            Self::SchemaVersion(error) => write!(formatter, "{error}"),
            Self::ProofWitness { path, error } => {
                write!(
                    formatter,
                    "invalid proof witness reference `{path}`: {error}"
                )
            }
            Self::MissingField { path } => write!(formatter, "missing required field `{path}`"),
            Self::UnknownField { path, field } => {
                write!(formatter, "unknown field `{field}` in object `{path}`")
            }
            Self::UnexpectedType { path, expected } => {
                write!(formatter, "field `{path}` must be {expected}")
            }
            Self::InvalidField { path, reason } => {
                write!(formatter, "invalid field `{path}`: {reason}")
            }
            Self::InvalidHash { path, reason } => {
                write!(formatter, "invalid hash field `{path}`: {reason}")
            }
            Self::UnsortedCollection { path } => {
                write!(formatter, "collection `{path}` is not in canonical order")
            }
            Self::DuplicateEntry { path, key } => {
                write!(
                    formatter,
                    "collection `{path}` contains duplicate key `{key}`"
                )
            }
            Self::InterfaceHashMismatch { expected, actual } => write!(
                formatter,
                "verified artifact interface_hash mismatch: expected `{expected}`, got `{actual}`"
            ),
            Self::ImplementationHashMismatch { expected, actual } => write!(
                formatter,
                "verified artifact implementation_hash mismatch: expected `{expected}`, got `{actual}`"
            ),
            Self::ExpectedInterfaceHashMismatch { expected, actual } => write!(
                formatter,
                "verified artifact expected interface hash `{expected}` does not match `{actual}`"
            ),
            Self::ExpectedImplementationHashMismatch { expected, actual } => write!(
                formatter,
                "verified artifact expected implementation hash `{expected}` does not match `{actual}`"
            ),
            Self::ModuleIdentityMismatch { expected, actual } => {
                write!(
                    formatter,
                    "verified artifact expected module `{expected}` does not match `{actual}`"
                )
            }
            Self::WitnessReferenceMismatch { path, reason } => {
                write!(formatter, "invalid witness reference `{path}`: {reason}")
            }
        }
    }
}

impl Error for VerifiedArtifactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalJson(error) => Some(error),
            Self::SchemaVersion(error) => Some(error),
            Self::ProofWitness { error, .. } => Some(error),
            _ => None,
        }
    }
}

impl From<CanonicalJsonError> for VerifiedArtifactError {
    fn from(error: CanonicalJsonError) -> Self {
        Self::CanonicalJson(error)
    }
}

impl From<SchemaVersionError> for VerifiedArtifactError {
    fn from(error: SchemaVersionError) -> Self {
        Self::SchemaVersion(error)
    }
}

fn validate_artifact(artifact: &VerifiedArtifact) -> Result<(), VerifiedArtifactError> {
    validate_artifact_shape(artifact)?;
    let interface_hash = artifact.compute_interface_hash()?;
    if interface_hash != artifact.interface_hash {
        return Err(VerifiedArtifactError::InterfaceHashMismatch {
            expected: verified_artifact_hash_string(
                ArtifactHashClass::Interface,
                artifact.schema_version,
                interface_hash,
            ),
            actual: verified_artifact_hash_string(
                ArtifactHashClass::Interface,
                artifact.schema_version,
                artifact.interface_hash,
            ),
        });
    }
    let implementation_hash = artifact.compute_implementation_hash()?;
    if implementation_hash != artifact.implementation_hash {
        return Err(VerifiedArtifactError::ImplementationHashMismatch {
            expected: verified_artifact_hash_string(
                ArtifactHashClass::Implementation,
                artifact.schema_version,
                implementation_hash,
            ),
            actual: verified_artifact_hash_string(
                ArtifactHashClass::Implementation,
                artifact.schema_version,
                artifact.implementation_hash,
            ),
        });
    }
    Ok(())
}

fn validate_artifact_shape(artifact: &VerifiedArtifact) -> Result<(), VerifiedArtifactError> {
    schema_version_support().check(Some(&artifact.schema_version.to_string()))?;
    validate_identity(&artifact.module, "$.module")?;
    validate_source_file(&artifact.source_file, "$.source_file")?;
    validate_optional_verified_at(artifact.verified_at.as_deref(), "$.verified_at")?;
    validate_exports(&artifact.exports, "$.exports")?;
    validate_expressions(&artifact.expressions, "$.expressions")?;
    validate_obligations(&artifact.obligations, "$.obligations")?;
    validate_proof_witnesses(&artifact.proof_witnesses, "$.proof_witnesses")?;
    validate_diagnostics(&artifact.diagnostics, "$.diagnostics")?;
    validate_provenance(&artifact.provenance, "$.provenance")?;
    validate_witness_consistency(artifact)?;
    Ok(())
}

fn verified_artifact_json_unchecked(
    artifact: &VerifiedArtifact,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        (
            "schema_version",
            CanonicalJson::string(artifact.schema_version.to_string()),
        ),
        ("module", identity_json(&artifact.module)?),
        ("source_file", CanonicalJson::string(&artifact.source_file)),
        (
            "source_hash",
            CanonicalJson::string(source_hash_string(artifact.source_hash)),
        ),
        (
            "verified_at",
            optional_string_json(artifact.verified_at.as_deref()),
        ),
        (
            "interface_hash",
            CanonicalJson::string(verified_artifact_hash_string(
                ArtifactHashClass::Interface,
                artifact.schema_version,
                artifact.interface_hash,
            )),
        ),
        (
            "implementation_hash",
            CanonicalJson::string(verified_artifact_hash_string(
                ArtifactHashClass::Implementation,
                artifact.schema_version,
                artifact.implementation_hash,
            )),
        ),
        (
            "exports",
            CanonicalJson::array(
                sorted_exports(&artifact.exports)
                    .into_iter()
                    .map(export_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "expressions",
            CanonicalJson::array(
                sorted_expressions(&artifact.expressions)
                    .into_iter()
                    .map(expression_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "obligations",
            CanonicalJson::array(
                sorted_obligations(&artifact.obligations)
                    .into_iter()
                    .map(obligation_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "proof_witnesses",
            CanonicalJson::array(
                sorted_proof_witnesses(&artifact.proof_witnesses)
                    .into_iter()
                    .enumerate()
                    .map(|(index, witness)| {
                        proof_witness_json(witness, &array_path("$.proof_witnesses", index))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "diagnostics",
            CanonicalJson::array(
                sorted_diagnostics(&artifact.diagnostics)
                    .into_iter()
                    .map(diagnostic_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        ("provenance", provenance_json(&artifact.provenance)?),
    ])
}

fn interface_projection_json(
    artifact: &VerifiedArtifact,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        (
            "schema_version",
            CanonicalJson::string(artifact.schema_version.to_string()),
        ),
        ("module", identity_json(&artifact.module)?),
        (
            "exports",
            CanonicalJson::array(
                sorted_exports(&artifact.exports)
                    .into_iter()
                    .map(export_interface_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "dependency_artifact_hashes",
            CanonicalJson::array(
                sorted_dependency_artifact_hashes(&artifact.provenance.dependency_artifact_hashes)
                    .into_iter()
                    .map(dependency_interface_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
    ])
}

fn implementation_projection_json(
    artifact: &VerifiedArtifact,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        (
            "schema_version",
            CanonicalJson::string(artifact.schema_version.to_string()),
        ),
        ("module", identity_json(&artifact.module)?),
        ("source_file", CanonicalJson::string(&artifact.source_file)),
        (
            "source_hash",
            CanonicalJson::string(source_hash_string(artifact.source_hash)),
        ),
        (
            "exports",
            CanonicalJson::array(
                sorted_exports(&artifact.exports)
                    .into_iter()
                    .map(export_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "expressions",
            CanonicalJson::array(
                sorted_expressions(&artifact.expressions)
                    .into_iter()
                    .map(expression_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "obligations",
            CanonicalJson::array(
                sorted_obligations(&artifact.obligations)
                    .into_iter()
                    .map(obligation_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "proof_witnesses",
            CanonicalJson::array(
                sorted_proof_witnesses(&artifact.proof_witnesses)
                    .into_iter()
                    .enumerate()
                    .map(|(index, witness)| {
                        proof_witness_json(witness, &array_path("$.proof_witnesses", index))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "diagnostics",
            CanonicalJson::array(
                sorted_diagnostics(&artifact.diagnostics)
                    .into_iter()
                    .map(diagnostic_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "provenance",
            provenance_implementation_json(&artifact.provenance)?,
        ),
    ])
}

fn identity_json(identity: &ModuleSummaryIdentity) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        ("package_id", CanonicalJson::string(&identity.package_id)),
        (
            "package_version",
            optional_string_json(identity.package_version.as_deref()),
        ),
        (
            "lockfile_identity",
            optional_string_json(identity.lockfile_identity.as_deref()),
        ),
        ("module_path", CanonicalJson::string(&identity.module_path)),
        (
            "language_edition",
            CanonicalJson::string(&identity.language_edition),
        ),
    ])
}

fn source_range_json(range: SourceRangeSummary) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        (
            "start_byte",
            CanonicalJson::integer(integer_from_u64(
                range.start_byte,
                "source_range.start_byte",
            )?),
        ),
        (
            "end_byte",
            CanonicalJson::integer(integer_from_u64(range.end_byte, "source_range.end_byte")?),
        ),
    ])
}

fn optional_source_range_json(
    range: Option<SourceRangeSummary>,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    range.map_or(Ok(CanonicalJson::null()), source_range_json)
}

fn export_json(export: &VerifiedExport) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        ("origin_id", CanonicalJson::string(&export.origin_id)),
        (
            "fully_qualified_name",
            CanonicalJson::string(&export.fully_qualified_name),
        ),
        ("namespace_path", string_array_json(&export.namespace_path)),
        (
            "visibility",
            CanonicalJson::string(export.visibility.as_str()),
        ),
        ("export_kind", CanonicalJson::string(&export.export_kind)),
        ("source_range", source_range_json(export.source_range)?),
        (
            "rendered_signature",
            CanonicalJson::string(&export.rendered_signature),
        ),
        (
            "interface_fingerprint",
            CanonicalJson::string(export.interface_fingerprint.to_artifact_hash_string()),
        ),
        (
            "proof_status",
            optional_string_json(export.proof_status.map(ExportProofStatus::as_str)),
        ),
        (
            "documentation_ref",
            optional_artifact_hash_json(export.documentation_ref.as_ref()),
        ),
    ])
}

fn export_interface_json(export: &VerifiedExport) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        ("origin_id", CanonicalJson::string(&export.origin_id)),
        (
            "fully_qualified_name",
            CanonicalJson::string(&export.fully_qualified_name),
        ),
        ("namespace_path", string_array_json(&export.namespace_path)),
        (
            "visibility",
            CanonicalJson::string(export.visibility.as_str()),
        ),
        ("export_kind", CanonicalJson::string(&export.export_kind)),
        (
            "rendered_signature",
            CanonicalJson::string(&export.rendered_signature),
        ),
        (
            "interface_fingerprint",
            CanonicalJson::string(export.interface_fingerprint.to_artifact_hash_string()),
        ),
        (
            "proof_status",
            optional_string_json(export.proof_status.map(ExportProofStatus::as_str)),
        ),
    ])
}

fn expression_json(
    expression: &ExpressionMetadata,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        (
            "expression_id",
            CanonicalJson::string(&expression.expression_id),
        ),
        ("source_range", source_range_json(expression.source_range)?),
        (
            "expression_kind",
            CanonicalJson::string(&expression.expression_kind),
        ),
        (
            "rendered_surface",
            CanonicalJson::string(&expression.rendered_surface),
        ),
        (
            "inferred_type",
            optional_string_json(expression.inferred_type.as_deref()),
        ),
        (
            "resolved_symbol",
            optional_string_json(expression.resolved_symbol.as_deref()),
        ),
        (
            "inserted_coercions",
            string_array_json(&expression.inserted_coercions),
        ),
        (
            "active_thesis",
            optional_string_json(expression.active_thesis.as_deref()),
        ),
        (
            "overload_resolution",
            expression
                .overload_resolution
                .as_ref()
                .map_or(Ok(CanonicalJson::null()), overload_json)?,
        ),
    ])
}

fn overload_json(overload: &OverloadMetadata) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        ("root_symbol", CanonicalJson::string(&overload.root_symbol)),
        (
            "selected_candidate",
            CanonicalJson::string(&overload.selected_candidate),
        ),
        (
            "active_refinements",
            string_array_json(&overload.active_refinements),
        ),
        (
            "coercion_summary",
            optional_string_json(overload.coercion_summary.as_deref()),
        ),
    ])
}

fn obligation_json(
    obligation: &ObligationMetadata,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        (
            "obligation_id",
            CanonicalJson::string(&obligation.obligation_id),
        ),
        (
            "obligation_anchor",
            optional_string_json(obligation.obligation_anchor.as_deref()),
        ),
        (
            "owner_origin_id",
            optional_string_json(obligation.owner_origin_id.as_deref()),
        ),
        ("source_range", source_range_json(obligation.source_range)?),
        (
            "obligation_kind",
            CanonicalJson::string(&obligation.obligation_kind),
        ),
        (
            "statement_summary",
            CanonicalJson::string(&obligation.statement_summary),
        ),
        (
            "obligation_fingerprint",
            CanonicalJson::string(obligation.obligation_fingerprint.to_artifact_hash_string()),
        ),
        (
            "vc_fingerprint",
            CanonicalJson::string(obligation.vc_fingerprint.to_artifact_hash_string()),
        ),
        (
            "local_context_fingerprint",
            CanonicalJson::string(
                obligation
                    .local_context_fingerprint
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "dependency_slice_fingerprint",
            CanonicalJson::string(
                obligation
                    .dependency_slice_fingerprint
                    .to_artifact_hash_string(),
            ),
        ),
        (
            "verifier_policy_fingerprint",
            CanonicalJson::string(
                obligation
                    .verifier_policy_fingerprint
                    .to_artifact_hash_string(),
            ),
        ),
        ("status", CanonicalJson::string(obligation.status.as_str())),
        (
            "accepted_witness_obligation_id",
            optional_string_json(obligation.accepted_witness_obligation_id.as_deref()),
        ),
        (
            "deterministic_discharge_hash",
            optional_artifact_hash_json(obligation.deterministic_discharge_hash.as_ref()),
        ),
        (
            "diagnostic_ref",
            optional_artifact_hash_json(obligation.diagnostic_ref.as_ref()),
        ),
    ])
}

fn proof_witness_json(
    witness: &ProofWitnessRef,
    path: &str,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    proof_witness_ref_json(witness).map_err(|error| VerifiedArtifactError::ProofWitness {
        path: path.to_owned(),
        error,
    })
}

fn diagnostic_json(
    diagnostic: &ArtifactDiagnostic,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        (
            "diagnostic_id",
            CanonicalJson::string(&diagnostic.diagnostic_id),
        ),
        ("code", CanonicalJson::string(&diagnostic.code)),
        (
            "severity",
            CanonicalJson::string(diagnostic.severity.as_str()),
        ),
        (
            "primary_range",
            optional_source_range_json(diagnostic.primary_range)?,
        ),
        (
            "message_key",
            CanonicalJson::string(&diagnostic.message_key),
        ),
        (
            "rendered_message",
            CanonicalJson::string(&diagnostic.rendered_message),
        ),
        (
            "related",
            CanonicalJson::array(
                sorted_diagnostic_related(&diagnostic.related)
                    .into_iter()
                    .map(diagnostic_related_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "explanation_ref",
            optional_artifact_hash_json(diagnostic.explanation_ref.as_ref()),
        ),
    ])
}

fn diagnostic_related_json(
    related: &DiagnosticRelated,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        ("source_range", source_range_json(related.source_range)?),
        ("message_key", CanonicalJson::string(&related.message_key)),
        (
            "rendered_message",
            CanonicalJson::string(&related.rendered_message),
        ),
    ])
}

fn provenance_json(provenance: &BuildProvenance) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        ("toolchain", CanonicalJson::string(&provenance.toolchain)),
        (
            "language_edition",
            CanonicalJson::string(&provenance.language_edition),
        ),
        (
            "lockfile_hash",
            CanonicalJson::string(provenance.lockfile_hash.to_artifact_hash_string()),
        ),
        (
            "verifier_config_hash",
            CanonicalJson::string(provenance.verifier_config_hash.to_artifact_hash_string()),
        ),
        (
            "dependency_artifact_hashes",
            CanonicalJson::array(
                sorted_dependency_artifact_hashes(&provenance.dependency_artifact_hashes)
                    .into_iter()
                    .map(dependency_artifact_hash_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "cache_key",
            optional_string_json(provenance.cache_key.as_deref()),
        ),
    ])
}

fn provenance_implementation_json(
    provenance: &BuildProvenance,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        ("toolchain", CanonicalJson::string(&provenance.toolchain)),
        (
            "language_edition",
            CanonicalJson::string(&provenance.language_edition),
        ),
        (
            "lockfile_hash",
            CanonicalJson::string(provenance.lockfile_hash.to_artifact_hash_string()),
        ),
        (
            "verifier_config_hash",
            CanonicalJson::string(provenance.verifier_config_hash.to_artifact_hash_string()),
        ),
        (
            "dependency_artifact_hashes",
            CanonicalJson::array(
                sorted_dependency_artifact_hashes(&provenance.dependency_artifact_hashes)
                    .into_iter()
                    .map(dependency_artifact_hash_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
    ])
}

fn dependency_artifact_hash_json(
    dependency: &DependencyArtifactHash,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        ("module", identity_json(&dependency.module)?),
        (
            "interface_hash",
            CanonicalJson::string(dependency.interface_hash.to_artifact_hash_string()),
        ),
        (
            "implementation_hash",
            optional_artifact_hash_json(dependency.implementation_hash.as_ref()),
        ),
        (
            "artifact_hash",
            optional_artifact_hash_json(dependency.artifact_hash.as_ref()),
        ),
    ])
}

fn dependency_interface_json(
    dependency: &DependencyArtifactHash,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    json_object([
        ("module", identity_json(&dependency.module)?),
        (
            "interface_hash",
            CanonicalJson::string(dependency.interface_hash.to_artifact_hash_string()),
        ),
    ])
}

fn json_object(
    fields: impl IntoIterator<Item = (&'static str, CanonicalJson)>,
) -> Result<CanonicalJson, VerifiedArtifactError> {
    CanonicalJson::object(fields).map_err(Into::into)
}

fn optional_string_json(value: Option<&str>) -> CanonicalJson {
    value.map_or_else(CanonicalJson::null, CanonicalJson::string)
}

fn optional_artifact_hash_json(value: Option<&ArtifactHashRef>) -> CanonicalJson {
    value.map_or_else(CanonicalJson::null, |hash_ref| {
        CanonicalJson::string(hash_ref.to_artifact_hash_string())
    })
}

fn string_array_json(values: &[String]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(CanonicalJson::string))
}

fn read_schema_version(
    fields: &BTreeMap<String, CanonicalJson>,
    artifact_path: Option<&str>,
) -> Result<SchemaVersion, VerifiedArtifactError> {
    let value = fields.get("schema_version");
    let version = match value {
        Some(CanonicalJson::String(version)) => Some(version.as_str()),
        Some(_) => {
            return Err(VerifiedArtifactError::UnexpectedType {
                path: "$.schema_version".to_owned(),
                expected: "a schema-version string",
            });
        }
        None => None,
    };
    let support = schema_version_support();
    if let Some(path) = artifact_path {
        support.check_at_path(version, path).map_err(Into::into)
    } else {
        support.check(version).map_err(Into::into)
    }
}

fn read_identity(
    value: &CanonicalJson,
    path: &str,
) -> Result<ModuleSummaryIdentity, VerifiedArtifactError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "package_id",
            "package_version",
            "lockfile_identity",
            "module_path",
            "language_edition",
        ],
        path,
    )?;
    let identity = ModuleSummaryIdentity {
        package_id: read_required_string(fields, "package_id", path)?,
        package_version: read_optional_string(fields, "package_version", path)?,
        lockfile_identity: read_optional_string(fields, "lockfile_identity", path)?,
        module_path: read_required_string(fields, "module_path", path)?,
        language_edition: read_required_string(fields, "language_edition", path)?,
    };
    validate_identity(&identity, path)?;
    Ok(identity)
}

fn read_source_range(
    value: &CanonicalJson,
    path: &str,
) -> Result<SourceRangeSummary, VerifiedArtifactError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(fields, &["start_byte", "end_byte"], path)?;
    let range = SourceRangeSummary {
        start_byte: read_non_negative_u64(fields, "start_byte", path)?,
        end_byte: read_non_negative_u64(fields, "end_byte", path)?,
    };
    validate_source_range(range, path)?;
    Ok(range)
}

fn read_optional_source_range(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<Option<SourceRangeSummary>, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    match value {
        CanonicalJson::Null => Ok(None),
        _ => read_source_range(value, &path).map(Some),
    }
}

fn read_exports(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<VerifiedExport>, VerifiedArtifactError> {
    let values = expect_array(value, path)?;
    let exports = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_export(value, &array_path(path, index)))
        .collect::<Result<Vec<_>, _>>()?;
    validate_exports(&exports, path)?;
    ensure_sorted_by_key(&exports, export_sort_key, path)?;
    Ok(exports)
}

fn read_export(value: &CanonicalJson, path: &str) -> Result<VerifiedExport, VerifiedArtifactError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "origin_id",
            "fully_qualified_name",
            "namespace_path",
            "visibility",
            "export_kind",
            "source_range",
            "rendered_signature",
            "interface_fingerprint",
            "proof_status",
            "documentation_ref",
        ],
        path,
    )?;
    let export = VerifiedExport {
        origin_id: read_required_string(fields, "origin_id", path)?,
        fully_qualified_name: read_required_string(fields, "fully_qualified_name", path)?,
        namespace_path: read_string_array(
            required_field(fields, "namespace_path", path)?,
            &field_path(path, "namespace_path"),
        )?,
        visibility: read_export_visibility(fields, "visibility", path)?,
        export_kind: read_required_string(fields, "export_kind", path)?,
        source_range: read_source_range(
            required_field(fields, "source_range", path)?,
            &field_path(path, "source_range"),
        )?,
        rendered_signature: read_required_string(fields, "rendered_signature", path)?,
        interface_fingerprint: read_required_artifact_hash_ref(
            fields,
            "interface_fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        proof_status: read_optional_export_proof_status(fields, "proof_status", path)?,
        documentation_ref: read_optional_artifact_hash_ref(
            fields,
            "documentation_ref",
            path,
            ArtifactHashClass::Diagnostic,
        )?,
    };
    validate_export(&export, path)?;
    Ok(export)
}

fn read_expressions(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<ExpressionMetadata>, VerifiedArtifactError> {
    let values = expect_array(value, path)?;
    let expressions = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_expression(value, &array_path(path, index)))
        .collect::<Result<Vec<_>, _>>()?;
    validate_expressions(&expressions, path)?;
    ensure_sorted_by_key(&expressions, expression_sort_key, path)?;
    Ok(expressions)
}

fn read_expression(
    value: &CanonicalJson,
    path: &str,
) -> Result<ExpressionMetadata, VerifiedArtifactError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "expression_id",
            "source_range",
            "expression_kind",
            "rendered_surface",
            "inferred_type",
            "resolved_symbol",
            "inserted_coercions",
            "active_thesis",
            "overload_resolution",
        ],
        path,
    )?;
    let expression = ExpressionMetadata {
        expression_id: read_required_string(fields, "expression_id", path)?,
        source_range: read_source_range(
            required_field(fields, "source_range", path)?,
            &field_path(path, "source_range"),
        )?,
        expression_kind: read_required_string(fields, "expression_kind", path)?,
        rendered_surface: read_required_string(fields, "rendered_surface", path)?,
        inferred_type: read_optional_string(fields, "inferred_type", path)?,
        resolved_symbol: read_optional_string(fields, "resolved_symbol", path)?,
        inserted_coercions: read_string_array(
            required_field(fields, "inserted_coercions", path)?,
            &field_path(path, "inserted_coercions"),
        )?,
        active_thesis: read_optional_string(fields, "active_thesis", path)?,
        overload_resolution: read_optional_overload(fields, "overload_resolution", path)?,
    };
    validate_expression(&expression, path)?;
    Ok(expression)
}

fn read_optional_overload(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<Option<OverloadMetadata>, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    match value {
        CanonicalJson::Null => Ok(None),
        _ => read_overload(value, &path).map(Some),
    }
}

fn read_overload(
    value: &CanonicalJson,
    path: &str,
) -> Result<OverloadMetadata, VerifiedArtifactError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "root_symbol",
            "selected_candidate",
            "active_refinements",
            "coercion_summary",
        ],
        path,
    )?;
    let overload = OverloadMetadata {
        root_symbol: read_required_string(fields, "root_symbol", path)?,
        selected_candidate: read_required_string(fields, "selected_candidate", path)?,
        active_refinements: read_string_array(
            required_field(fields, "active_refinements", path)?,
            &field_path(path, "active_refinements"),
        )?,
        coercion_summary: read_optional_string(fields, "coercion_summary", path)?,
    };
    validate_overload(&overload, path)?;
    Ok(overload)
}

fn read_obligations(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<ObligationMetadata>, VerifiedArtifactError> {
    let values = expect_array(value, path)?;
    let obligations = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_obligation(value, &array_path(path, index)))
        .collect::<Result<Vec<_>, _>>()?;
    validate_obligations(&obligations, path)?;
    ensure_sorted_by_key(&obligations, obligation_sort_key, path)?;
    Ok(obligations)
}

fn read_obligation(
    value: &CanonicalJson,
    path: &str,
) -> Result<ObligationMetadata, VerifiedArtifactError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "obligation_id",
            "obligation_anchor",
            "owner_origin_id",
            "source_range",
            "obligation_kind",
            "statement_summary",
            "obligation_fingerprint",
            "vc_fingerprint",
            "local_context_fingerprint",
            "dependency_slice_fingerprint",
            "verifier_policy_fingerprint",
            "status",
            "accepted_witness_obligation_id",
            "deterministic_discharge_hash",
            "diagnostic_ref",
        ],
        path,
    )?;
    let obligation = ObligationMetadata {
        obligation_id: read_required_string(fields, "obligation_id", path)?,
        obligation_anchor: read_optional_string(fields, "obligation_anchor", path)?,
        owner_origin_id: read_optional_string(fields, "owner_origin_id", path)?,
        source_range: read_source_range(
            required_field(fields, "source_range", path)?,
            &field_path(path, "source_range"),
        )?,
        obligation_kind: read_required_string(fields, "obligation_kind", path)?,
        statement_summary: read_required_string(fields, "statement_summary", path)?,
        obligation_fingerprint: read_required_artifact_hash_ref(
            fields,
            "obligation_fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        vc_fingerprint: read_required_artifact_hash_ref(
            fields,
            "vc_fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        local_context_fingerprint: read_required_artifact_hash_ref(
            fields,
            "local_context_fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        dependency_slice_fingerprint: read_required_artifact_hash_ref(
            fields,
            "dependency_slice_fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        verifier_policy_fingerprint: read_required_artifact_hash_ref(
            fields,
            "verifier_policy_fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        status: read_obligation_status(fields, "status", path)?,
        accepted_witness_obligation_id: read_optional_string(
            fields,
            "accepted_witness_obligation_id",
            path,
        )?,
        deterministic_discharge_hash: read_optional_artifact_hash_ref(
            fields,
            "deterministic_discharge_hash",
            path,
            ArtifactHashClass::Interface,
        )?,
        diagnostic_ref: read_optional_artifact_hash_ref(
            fields,
            "diagnostic_ref",
            path,
            ArtifactHashClass::Diagnostic,
        )?,
    };
    validate_obligation(&obligation, path)?;
    Ok(obligation)
}

fn read_proof_witnesses(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<ProofWitnessRef>, VerifiedArtifactError> {
    let values = expect_array(value, path)?;
    let witnesses = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let item_path = array_path(path, index);
            read_proof_witness_ref(value, ProofWitnessReadOptions::default()).map_err(|error| {
                VerifiedArtifactError::ProofWitness {
                    path: item_path,
                    error,
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_proof_witnesses(&witnesses, path)?;
    ensure_sorted_by_key(&witnesses, proof_witness_sort_key, path)?;
    Ok(witnesses)
}

fn read_diagnostics(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<ArtifactDiagnostic>, VerifiedArtifactError> {
    let values = expect_array(value, path)?;
    let diagnostics = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_diagnostic(value, &array_path(path, index)))
        .collect::<Result<Vec<_>, _>>()?;
    validate_diagnostics(&diagnostics, path)?;
    ensure_sorted_by_key(&diagnostics, diagnostic_sort_key, path)?;
    Ok(diagnostics)
}

fn read_diagnostic(
    value: &CanonicalJson,
    path: &str,
) -> Result<ArtifactDiagnostic, VerifiedArtifactError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "diagnostic_id",
            "code",
            "severity",
            "primary_range",
            "message_key",
            "rendered_message",
            "related",
            "explanation_ref",
        ],
        path,
    )?;
    let diagnostic = ArtifactDiagnostic {
        diagnostic_id: read_required_string(fields, "diagnostic_id", path)?,
        code: read_required_string(fields, "code", path)?,
        severity: read_diagnostic_severity(fields, "severity", path)?,
        primary_range: read_optional_source_range(fields, "primary_range", path)?,
        message_key: read_required_string(fields, "message_key", path)?,
        rendered_message: read_required_string(fields, "rendered_message", path)?,
        related: read_diagnostic_related(
            required_field(fields, "related", path)?,
            &field_path(path, "related"),
        )?,
        explanation_ref: read_optional_artifact_hash_ref(
            fields,
            "explanation_ref",
            path,
            ArtifactHashClass::Diagnostic,
        )?,
    };
    validate_diagnostic(&diagnostic, path)?;
    Ok(diagnostic)
}

fn read_diagnostic_related(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<DiagnosticRelated>, VerifiedArtifactError> {
    let values = expect_array(value, path)?;
    let related = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_related(value, &array_path(path, index)))
        .collect::<Result<Vec<_>, _>>()?;
    validate_diagnostic_related(&related, path)?;
    ensure_sorted_by_key(&related, diagnostic_related_sort_key, path)?;
    Ok(related)
}

fn read_related(
    value: &CanonicalJson,
    path: &str,
) -> Result<DiagnosticRelated, VerifiedArtifactError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &["source_range", "message_key", "rendered_message"],
        path,
    )?;
    let related = DiagnosticRelated {
        source_range: read_source_range(
            required_field(fields, "source_range", path)?,
            &field_path(path, "source_range"),
        )?,
        message_key: read_required_string(fields, "message_key", path)?,
        rendered_message: read_required_string(fields, "rendered_message", path)?,
    };
    validate_related(&related, path)?;
    Ok(related)
}

fn read_provenance(
    value: &CanonicalJson,
    path: &str,
) -> Result<BuildProvenance, VerifiedArtifactError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "toolchain",
            "language_edition",
            "lockfile_hash",
            "verifier_config_hash",
            "dependency_artifact_hashes",
            "cache_key",
        ],
        path,
    )?;
    let provenance = BuildProvenance {
        toolchain: read_required_string(fields, "toolchain", path)?,
        language_edition: read_required_string(fields, "language_edition", path)?,
        lockfile_hash: read_required_artifact_hash_ref(
            fields,
            "lockfile_hash",
            path,
            ArtifactHashClass::Artifact,
        )?,
        verifier_config_hash: read_required_artifact_hash_ref(
            fields,
            "verifier_config_hash",
            path,
            ArtifactHashClass::Interface,
        )?,
        dependency_artifact_hashes: read_dependency_artifact_hashes(
            required_field(fields, "dependency_artifact_hashes", path)?,
            &field_path(path, "dependency_artifact_hashes"),
        )?,
        cache_key: read_optional_string(fields, "cache_key", path)?,
    };
    validate_provenance(&provenance, path)?;
    Ok(provenance)
}

fn read_dependency_artifact_hashes(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<DependencyArtifactHash>, VerifiedArtifactError> {
    let values = expect_array(value, path)?;
    let dependencies = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_dependency_artifact_hash(value, &array_path(path, index)))
        .collect::<Result<Vec<_>, _>>()?;
    validate_dependency_artifact_hashes(&dependencies, path)?;
    ensure_sorted_by_key(&dependencies, dependency_artifact_hash_sort_key, path)?;
    Ok(dependencies)
}

fn read_dependency_artifact_hash(
    value: &CanonicalJson,
    path: &str,
) -> Result<DependencyArtifactHash, VerifiedArtifactError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "module",
            "interface_hash",
            "implementation_hash",
            "artifact_hash",
        ],
        path,
    )?;
    let dependency = DependencyArtifactHash {
        module: read_identity(
            required_field(fields, "module", path)?,
            &field_path(path, "module"),
        )?,
        interface_hash: read_required_artifact_hash_ref(
            fields,
            "interface_hash",
            path,
            ArtifactHashClass::Interface,
        )?,
        implementation_hash: read_optional_artifact_hash_ref(
            fields,
            "implementation_hash",
            path,
            ArtifactHashClass::Implementation,
        )?,
        artifact_hash: read_optional_artifact_hash_ref(
            fields,
            "artifact_hash",
            path,
            ArtifactHashClass::Artifact,
        )?,
    };
    validate_dependency_artifact_hash(&dependency, path)?;
    Ok(dependency)
}

fn read_export_visibility(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<ExportVisibility, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(VerifiedArtifactError::UnexpectedType {
            path,
            expected: "an export-visibility string",
        });
    };
    ExportVisibility::from_str(value).ok_or_else(|| VerifiedArtifactError::InvalidField {
        path,
        reason: "unknown export visibility".to_owned(),
    })
}

fn read_optional_export_proof_status(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<Option<ExportProofStatus>, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    match value {
        CanonicalJson::Null => Ok(None),
        CanonicalJson::String(value) => {
            ExportProofStatus::from_str(value).map(Some).ok_or_else(|| {
                VerifiedArtifactError::InvalidField {
                    path,
                    reason: "unknown export proof status".to_owned(),
                }
            })
        }
        _ => Err(VerifiedArtifactError::UnexpectedType {
            path,
            expected: "an export proof-status string or null",
        }),
    }
}

fn read_obligation_status(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<ObligationStatus, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(VerifiedArtifactError::UnexpectedType {
            path,
            expected: "an obligation-status string",
        });
    };
    ObligationStatus::from_str(value).ok_or_else(|| VerifiedArtifactError::InvalidField {
        path,
        reason: "unknown obligation status".to_owned(),
    })
}

fn read_diagnostic_severity(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<DiagnosticSeverity, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(VerifiedArtifactError::UnexpectedType {
            path,
            expected: "a diagnostic-severity string",
        });
    };
    DiagnosticSeverity::from_str(value).ok_or_else(|| VerifiedArtifactError::InvalidField {
        path,
        reason: "unknown diagnostic severity".to_owned(),
    })
}

fn read_optional_verified_at(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<Option<String>, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    match value {
        CanonicalJson::Null => Ok(None),
        CanonicalJson::String(value) => {
            validate_verified_at(value, &path)?;
            Ok(Some(value.clone()))
        }
        _ => Err(VerifiedArtifactError::UnexpectedType {
            path,
            expected: "a UTC RFC3339 timestamp string or null",
        }),
    }
}

fn validate_identity(
    identity: &ModuleSummaryIdentity,
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    validate_non_empty(&identity.package_id, &field_path(path, "package_id"))?;
    validate_optional_non_empty(
        identity.package_version.as_deref(),
        &field_path(path, "package_version"),
    )?;
    validate_optional_non_empty(
        identity.lockfile_identity.as_deref(),
        &field_path(path, "lockfile_identity"),
    )?;
    validate_non_empty(&identity.module_path, &field_path(path, "module_path"))?;
    validate_non_empty(
        &identity.language_edition,
        &field_path(path, "language_edition"),
    )
}

fn validate_source_file(value: &str, path: &str) -> Result<(), VerifiedArtifactError> {
    validate_non_empty(value, path)?;
    if value.starts_with('/') {
        return Err(VerifiedArtifactError::InvalidField {
            path: path.to_owned(),
            reason: "source_file must be relative".to_owned(),
        });
    }
    if value.contains('\\') {
        return Err(VerifiedArtifactError::InvalidField {
            path: path.to_owned(),
            reason: "source_file must use / separators".to_owned(),
        });
    }
    for segment in value.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(VerifiedArtifactError::InvalidField {
                path: path.to_owned(),
                reason: "source_file segments must not be empty, . or ..".to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_source_range(
    range: SourceRangeSummary,
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    if range.start_byte > range.end_byte {
        return Err(VerifiedArtifactError::InvalidField {
            path: path.to_owned(),
            reason: "start_byte must not be greater than end_byte".to_owned(),
        });
    }
    integer_from_u64(range.start_byte, &field_path(path, "start_byte"))?;
    integer_from_u64(range.end_byte, &field_path(path, "end_byte"))?;
    Ok(())
}

fn validate_optional_verified_at(
    value: Option<&str>,
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    if let Some(value) = value {
        validate_verified_at(value, path)?;
    }
    Ok(())
}

fn validate_verified_at(value: &str, path: &str) -> Result<(), VerifiedArtifactError> {
    if value.len() != "2026-06-22T14:03:05Z".len() {
        return Err(invalid_verified_at(path));
    }
    let bytes = value.as_bytes();
    let punctuation = [
        (4, b'-'),
        (7, b'-'),
        (10, b'T'),
        (13, b':'),
        (16, b':'),
        (19, b'Z'),
    ];
    for (index, expected) in punctuation {
        if bytes.get(index).copied() != Some(expected) {
            return Err(invalid_verified_at(path));
        }
    }
    for index in [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !bytes[index].is_ascii_digit() {
            return Err(invalid_verified_at(path));
        }
    }
    let year = parse_fixed_u16(&value[0..4], path)?;
    let month = parse_fixed_u8(&value[5..7], path)?;
    let day = parse_fixed_u8(&value[8..10], path)?;
    let hour = parse_fixed_u8(&value[11..13], path)?;
    let minute = parse_fixed_u8(&value[14..16], path)?;
    let second = parse_fixed_u8(&value[17..19], path)?;
    if month == 0
        || month > 12
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err(invalid_verified_at(path));
    }
    Ok(())
}

fn invalid_verified_at(path: &str) -> VerifiedArtifactError {
    VerifiedArtifactError::InvalidField {
        path: path.to_owned(),
        reason: "verified_at must be an RFC3339 UTC timestamp with whole-second precision"
            .to_owned(),
    }
}

fn parse_fixed_u16(value: &str, path: &str) -> Result<u16, VerifiedArtifactError> {
    value.parse::<u16>().map_err(|_| invalid_verified_at(path))
}

fn parse_fixed_u8(value: &str, path: &str) -> Result<u8, VerifiedArtifactError> {
    value.parse::<u8>().map_err(|_| invalid_verified_at(path))
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn validate_exports(exports: &[VerifiedExport], path: &str) -> Result<(), VerifiedArtifactError> {
    ensure_no_duplicate_keys(exports, |export| export.origin_id.clone(), path)?;
    for (index, export) in exports.iter().enumerate() {
        validate_export(export, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_export(export: &VerifiedExport, path: &str) -> Result<(), VerifiedArtifactError> {
    validate_non_empty(&export.origin_id, &field_path(path, "origin_id"))?;
    validate_non_empty(
        &export.fully_qualified_name,
        &field_path(path, "fully_qualified_name"),
    )?;
    validate_string_array(&export.namespace_path, &field_path(path, "namespace_path"))?;
    validate_non_empty(&export.export_kind, &field_path(path, "export_kind"))?;
    validate_source_range(export.source_range, &field_path(path, "source_range"))?;
    validate_non_empty(
        &export.rendered_signature,
        &field_path(path, "rendered_signature"),
    )?;
    validate_artifact_hash_ref(
        &export.interface_fingerprint,
        &field_path(path, "interface_fingerprint"),
        ArtifactHashClass::Interface,
    )?;
    if let Some(documentation_ref) = &export.documentation_ref {
        validate_artifact_hash_ref(
            documentation_ref,
            &field_path(path, "documentation_ref"),
            ArtifactHashClass::Diagnostic,
        )?;
    }
    Ok(())
}

fn validate_expressions(
    expressions: &[ExpressionMetadata],
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    ensure_no_duplicate_keys(
        expressions,
        |expression| expression.expression_id.clone(),
        path,
    )?;
    for (index, expression) in expressions.iter().enumerate() {
        validate_expression(expression, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_expression(
    expression: &ExpressionMetadata,
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    validate_non_empty(
        &expression.expression_id,
        &field_path(path, "expression_id"),
    )?;
    validate_source_range(expression.source_range, &field_path(path, "source_range"))?;
    validate_non_empty(
        &expression.expression_kind,
        &field_path(path, "expression_kind"),
    )?;
    validate_non_empty(
        &expression.rendered_surface,
        &field_path(path, "rendered_surface"),
    )?;
    validate_optional_non_empty(
        expression.inferred_type.as_deref(),
        &field_path(path, "inferred_type"),
    )?;
    validate_optional_non_empty(
        expression.resolved_symbol.as_deref(),
        &field_path(path, "resolved_symbol"),
    )?;
    validate_string_array(
        &expression.inserted_coercions,
        &field_path(path, "inserted_coercions"),
    )?;
    validate_optional_non_empty(
        expression.active_thesis.as_deref(),
        &field_path(path, "active_thesis"),
    )?;
    if let Some(overload) = &expression.overload_resolution {
        validate_overload(overload, &field_path(path, "overload_resolution"))?;
    }
    Ok(())
}

fn validate_overload(overload: &OverloadMetadata, path: &str) -> Result<(), VerifiedArtifactError> {
    validate_non_empty(&overload.root_symbol, &field_path(path, "root_symbol"))?;
    validate_non_empty(
        &overload.selected_candidate,
        &field_path(path, "selected_candidate"),
    )?;
    validate_string_array(
        &overload.active_refinements,
        &field_path(path, "active_refinements"),
    )?;
    validate_optional_non_empty(
        overload.coercion_summary.as_deref(),
        &field_path(path, "coercion_summary"),
    )
}

fn validate_obligations(
    obligations: &[ObligationMetadata],
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    ensure_no_duplicate_keys(
        obligations,
        |obligation| obligation.obligation_id.clone(),
        path,
    )?;
    for (index, obligation) in obligations.iter().enumerate() {
        validate_obligation(obligation, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_obligation(
    obligation: &ObligationMetadata,
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    validate_non_empty(
        &obligation.obligation_id,
        &field_path(path, "obligation_id"),
    )?;
    validate_optional_non_empty(
        obligation.obligation_anchor.as_deref(),
        &field_path(path, "obligation_anchor"),
    )?;
    validate_optional_non_empty(
        obligation.owner_origin_id.as_deref(),
        &field_path(path, "owner_origin_id"),
    )?;
    validate_source_range(obligation.source_range, &field_path(path, "source_range"))?;
    validate_non_empty(
        &obligation.obligation_kind,
        &field_path(path, "obligation_kind"),
    )?;
    validate_non_empty(
        &obligation.statement_summary,
        &field_path(path, "statement_summary"),
    )?;
    for (field, hash_ref) in [
        ("obligation_fingerprint", &obligation.obligation_fingerprint),
        ("vc_fingerprint", &obligation.vc_fingerprint),
        (
            "local_context_fingerprint",
            &obligation.local_context_fingerprint,
        ),
        (
            "dependency_slice_fingerprint",
            &obligation.dependency_slice_fingerprint,
        ),
        (
            "verifier_policy_fingerprint",
            &obligation.verifier_policy_fingerprint,
        ),
    ] {
        validate_artifact_hash_ref(
            hash_ref,
            &field_path(path, field),
            ArtifactHashClass::Interface,
        )?;
    }
    validate_optional_non_empty(
        obligation.accepted_witness_obligation_id.as_deref(),
        &field_path(path, "accepted_witness_obligation_id"),
    )?;
    if let Some(discharge_hash) = &obligation.deterministic_discharge_hash {
        validate_artifact_hash_ref(
            discharge_hash,
            &field_path(path, "deterministic_discharge_hash"),
            ArtifactHashClass::Interface,
        )?;
    }
    if let Some(diagnostic_ref) = &obligation.diagnostic_ref {
        validate_artifact_hash_ref(
            diagnostic_ref,
            &field_path(path, "diagnostic_ref"),
            ArtifactHashClass::Diagnostic,
        )?;
    }
    validate_obligation_status_shape(obligation, path)
}

fn validate_obligation_status_shape(
    obligation: &ObligationMetadata,
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    match obligation.status {
        ObligationStatus::Accepted => {
            if obligation.accepted_witness_obligation_id.as_deref()
                != Some(obligation.obligation_id.as_str())
            {
                return Err(VerifiedArtifactError::WitnessReferenceMismatch {
                    path: field_path(path, "accepted_witness_obligation_id"),
                    reason: "accepted obligation must name its own obligation_id".to_owned(),
                });
            }
            if obligation.deterministic_discharge_hash.is_some() {
                return Err(VerifiedArtifactError::WitnessReferenceMismatch {
                    path: field_path(path, "deterministic_discharge_hash"),
                    reason: "accepted obligation must not use deterministic discharge hash"
                        .to_owned(),
                });
            }
        }
        ObligationStatus::NotRequired => {
            if obligation.accepted_witness_obligation_id.is_some() {
                return Err(VerifiedArtifactError::WitnessReferenceMismatch {
                    path: field_path(path, "accepted_witness_obligation_id"),
                    reason: "not-required obligation must not name a witness".to_owned(),
                });
            }
        }
        ObligationStatus::Open
        | ObligationStatus::Rejected
        | ObligationStatus::ExternallyAttested => {
            if obligation.accepted_witness_obligation_id.is_some() {
                return Err(VerifiedArtifactError::WitnessReferenceMismatch {
                    path: field_path(path, "accepted_witness_obligation_id"),
                    reason: "untrusted obligation status must not name a witness".to_owned(),
                });
            }
            if obligation.deterministic_discharge_hash.is_some() {
                return Err(VerifiedArtifactError::WitnessReferenceMismatch {
                    path: field_path(path, "deterministic_discharge_hash"),
                    reason: "deterministic discharge hash is allowed only for not_required"
                        .to_owned(),
                });
            }
        }
    }
    Ok(())
}

fn validate_proof_witnesses(
    witnesses: &[ProofWitnessRef],
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    ensure_no_duplicate_keys(witnesses, |witness| witness.obligation_id.clone(), path)?;
    for (index, witness) in witnesses.iter().enumerate() {
        proof_witness_ref_json(witness).map_err(|error| VerifiedArtifactError::ProofWitness {
            path: array_path(path, index),
            error,
        })?;
    }
    Ok(())
}

fn validate_diagnostics(
    diagnostics: &[ArtifactDiagnostic],
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    ensure_no_duplicate_keys(
        diagnostics,
        |diagnostic| diagnostic.diagnostic_id.clone(),
        path,
    )?;
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        validate_diagnostic(diagnostic, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_diagnostic(
    diagnostic: &ArtifactDiagnostic,
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    validate_non_empty(
        &diagnostic.diagnostic_id,
        &field_path(path, "diagnostic_id"),
    )?;
    validate_non_empty(&diagnostic.code, &field_path(path, "code"))?;
    if let Some(range) = diagnostic.primary_range {
        validate_source_range(range, &field_path(path, "primary_range"))?;
    }
    validate_non_empty(&diagnostic.message_key, &field_path(path, "message_key"))?;
    validate_non_empty(
        &diagnostic.rendered_message,
        &field_path(path, "rendered_message"),
    )?;
    validate_diagnostic_related(&diagnostic.related, &field_path(path, "related"))?;
    if let Some(explanation_ref) = &diagnostic.explanation_ref {
        validate_artifact_hash_ref(
            explanation_ref,
            &field_path(path, "explanation_ref"),
            ArtifactHashClass::Diagnostic,
        )?;
    }
    Ok(())
}

fn validate_diagnostic_related(
    related: &[DiagnosticRelated],
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    ensure_no_duplicate_keys(related, diagnostic_related_identity_key, path)?;
    for (index, related) in related.iter().enumerate() {
        validate_related(related, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_related(related: &DiagnosticRelated, path: &str) -> Result<(), VerifiedArtifactError> {
    validate_source_range(related.source_range, &field_path(path, "source_range"))?;
    validate_non_empty(&related.message_key, &field_path(path, "message_key"))?;
    validate_non_empty(
        &related.rendered_message,
        &field_path(path, "rendered_message"),
    )
}

fn validate_provenance(
    provenance: &BuildProvenance,
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    validate_non_empty(&provenance.toolchain, &field_path(path, "toolchain"))?;
    validate_non_empty(
        &provenance.language_edition,
        &field_path(path, "language_edition"),
    )?;
    validate_artifact_hash_ref(
        &provenance.lockfile_hash,
        &field_path(path, "lockfile_hash"),
        ArtifactHashClass::Artifact,
    )?;
    validate_artifact_hash_ref(
        &provenance.verifier_config_hash,
        &field_path(path, "verifier_config_hash"),
        ArtifactHashClass::Interface,
    )?;
    validate_dependency_artifact_hashes(
        &provenance.dependency_artifact_hashes,
        &field_path(path, "dependency_artifact_hashes"),
    )?;
    validate_optional_non_empty(
        provenance.cache_key.as_deref(),
        &field_path(path, "cache_key"),
    )
}

fn validate_dependency_artifact_hashes(
    dependencies: &[DependencyArtifactHash],
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    ensure_no_duplicate_keys(dependencies, |dependency| dependency.module.clone(), path)?;
    for (index, dependency) in dependencies.iter().enumerate() {
        validate_dependency_artifact_hash(dependency, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_dependency_artifact_hash(
    dependency: &DependencyArtifactHash,
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    validate_identity(&dependency.module, &field_path(path, "module"))?;
    validate_artifact_hash_ref(
        &dependency.interface_hash,
        &field_path(path, "interface_hash"),
        ArtifactHashClass::Interface,
    )?;
    if let Some(implementation_hash) = &dependency.implementation_hash {
        validate_artifact_hash_ref(
            implementation_hash,
            &field_path(path, "implementation_hash"),
            ArtifactHashClass::Implementation,
        )?;
    }
    if let Some(artifact_hash) = &dependency.artifact_hash {
        validate_artifact_hash_ref(
            artifact_hash,
            &field_path(path, "artifact_hash"),
            ArtifactHashClass::Artifact,
        )?;
    }
    Ok(())
}

fn validate_witness_consistency(artifact: &VerifiedArtifact) -> Result<(), VerifiedArtifactError> {
    let witnesses = artifact
        .proof_witnesses
        .iter()
        .map(|witness| (witness.obligation_id.as_str(), witness))
        .collect::<BTreeMap<_, _>>();
    let obligations = artifact
        .obligations
        .iter()
        .map(|obligation| (obligation.obligation_id.as_str(), obligation))
        .collect::<BTreeMap<_, _>>();

    for (index, obligation) in artifact.obligations.iter().enumerate() {
        if obligation.status != ObligationStatus::Accepted {
            continue;
        }
        let path = array_path("$.obligations", index);
        let Some(witness) = witnesses.get(obligation.obligation_id.as_str()) else {
            return Err(VerifiedArtifactError::WitnessReferenceMismatch {
                path: field_path(&path, "accepted_witness_obligation_id"),
                reason: "accepted obligation must resolve to exactly one proof witness reference"
                    .to_owned(),
            });
        };
        if witness.obligation_fingerprint != obligation.obligation_fingerprint {
            return Err(VerifiedArtifactError::WitnessReferenceMismatch {
                path: field_path(&path, "obligation_fingerprint"),
                reason: "accepted witness obligation_fingerprint mismatch".to_owned(),
            });
        }
        if witness.kernel_acceptance.verifier_policy_fingerprint
            != obligation.verifier_policy_fingerprint
        {
            return Err(VerifiedArtifactError::WitnessReferenceMismatch {
                path: field_path(&path, "verifier_policy_fingerprint"),
                reason: "accepted witness verifier policy fingerprint mismatch".to_owned(),
            });
        }
    }

    for (index, witness) in artifact.proof_witnesses.iter().enumerate() {
        let path = array_path("$.proof_witnesses", index);
        let Some(obligation) = obligations.get(witness.obligation_id.as_str()) else {
            return Err(VerifiedArtifactError::WitnessReferenceMismatch {
                path: field_path(&path, "obligation_id"),
                reason: "proof witness must reference an accepted obligation in this artifact"
                    .to_owned(),
            });
        };
        if obligation.status != ObligationStatus::Accepted {
            return Err(VerifiedArtifactError::WitnessReferenceMismatch {
                path: field_path(&path, "obligation_id"),
                reason: "proof witness is trusted only for accepted obligations".to_owned(),
            });
        }
    }
    Ok(())
}

fn expect_object<'a>(
    value: &'a CanonicalJson,
    path: &str,
) -> Result<&'a BTreeMap<String, CanonicalJson>, VerifiedArtifactError> {
    let CanonicalJson::Object(fields) = value else {
        return Err(VerifiedArtifactError::UnexpectedType {
            path: path.to_owned(),
            expected: "an object",
        });
    };
    Ok(fields)
}

fn required_field<'a>(
    fields: &'a BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<&'a CanonicalJson, VerifiedArtifactError> {
    fields
        .get(field)
        .ok_or_else(|| VerifiedArtifactError::MissingField {
            path: field_path(path, field),
        })
}

fn reject_unknown_fields(
    fields: &BTreeMap<String, CanonicalJson>,
    allowed: &[&str],
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    for field in fields.keys() {
        if !allowed.contains(&field.as_str()) {
            return Err(VerifiedArtifactError::UnknownField {
                path: path.to_owned(),
                field: field.clone(),
            });
        }
    }
    Ok(())
}

fn expect_array<'a>(
    value: &'a CanonicalJson,
    path: &str,
) -> Result<&'a [CanonicalJson], VerifiedArtifactError> {
    let CanonicalJson::Array(values) = value else {
        return Err(VerifiedArtifactError::UnexpectedType {
            path: path.to_owned(),
            expected: "an array",
        });
    };
    Ok(values)
}

fn read_required_string(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<String, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(VerifiedArtifactError::UnexpectedType {
            path,
            expected: "a string",
        });
    };
    validate_non_empty(value, &path)?;
    Ok(value.clone())
}

fn read_optional_string(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<Option<String>, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    match value {
        CanonicalJson::Null => Ok(None),
        CanonicalJson::String(value) => {
            validate_non_empty(value, &path)?;
            Ok(Some(value.clone()))
        }
        _ => Err(VerifiedArtifactError::UnexpectedType {
            path,
            expected: "a string or null",
        }),
    }
}

fn read_string_array(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<String>, VerifiedArtifactError> {
    expect_array(value, path)?
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let path = array_path(path, index);
            let CanonicalJson::String(value) = value else {
                return Err(VerifiedArtifactError::UnexpectedType {
                    path,
                    expected: "a string",
                });
            };
            validate_non_empty(value, &path)?;
            Ok(value.clone())
        })
        .collect()
}

fn read_non_negative_u64(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<u64, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::Integer(value) = value else {
        return Err(VerifiedArtifactError::UnexpectedType {
            path,
            expected: "a non-negative integer",
        });
    };
    u64::try_from(*value).map_err(|_| VerifiedArtifactError::InvalidField {
        path,
        reason: "must be non-negative".to_owned(),
    })
}

fn read_required_artifact_hash_ref(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<ArtifactHashRef, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    read_artifact_hash_ref(value, &path, expected_class)
}

fn read_optional_artifact_hash_ref(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<Option<ArtifactHashRef>, VerifiedArtifactError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    match value {
        CanonicalJson::Null => Ok(None),
        _ => read_artifact_hash_ref(value, &path, expected_class).map(Some),
    }
}

fn read_artifact_hash_ref(
    value: &CanonicalJson,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<ArtifactHashRef, VerifiedArtifactError> {
    let CanonicalJson::String(value) = value else {
        return Err(VerifiedArtifactError::UnexpectedType {
            path: path.to_owned(),
            expected: "an artifact-framed hash string",
        });
    };
    validate_non_empty(value, path)?;
    let hash_ref = parse_artifact_hash_ref_string(value, path)?;
    if hash_ref.class != expected_class {
        return Err(VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: format!(
                "wrong artifact hash class: expected `{}`, got `{}`",
                artifact_hash_class_string(expected_class),
                artifact_hash_class_string(hash_ref.class)
            ),
        });
    }
    Ok(hash_ref)
}

fn read_source_hash(value: &CanonicalJson, path: &str) -> Result<Hash, VerifiedArtifactError> {
    let CanonicalJson::String(value) = value else {
        return Err(VerifiedArtifactError::UnexpectedType {
            path: path.to_owned(),
            expected: "a source hash string",
        });
    };
    parse_source_hash_string(value, path)
}

fn read_verified_artifact_hash(
    value: &CanonicalJson,
    path: &str,
    expected_class: ArtifactHashClass,
    schema_version: SchemaVersion,
) -> Result<Hash, VerifiedArtifactError> {
    let hash_ref = read_artifact_hash_ref(value, path, expected_class)?;
    if hash_ref.schema_family != VERIFIED_ARTIFACT_SCHEMA_FAMILY {
        return Err(VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong schema family".to_owned(),
        });
    }
    if hash_ref.schema_version != schema_version {
        return Err(VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong schema version".to_owned(),
        });
    }
    Ok(hash_ref.digest)
}

fn validate_non_empty(value: &str, path: &str) -> Result<(), VerifiedArtifactError> {
    if value.is_empty() {
        return Err(VerifiedArtifactError::InvalidField {
            path: path.to_owned(),
            reason: "must not be empty".to_owned(),
        });
    }
    Ok(())
}

fn validate_optional_non_empty(
    value: Option<&str>,
    path: &str,
) -> Result<(), VerifiedArtifactError> {
    if matches!(value, Some("")) {
        return Err(VerifiedArtifactError::InvalidField {
            path: path.to_owned(),
            reason: "must be null or a non-empty string".to_owned(),
        });
    }
    Ok(())
}

fn validate_string_array(values: &[String], path: &str) -> Result<(), VerifiedArtifactError> {
    for (index, value) in values.iter().enumerate() {
        validate_non_empty(value, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_artifact_hash_ref(
    hash_ref: &ArtifactHashRef,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<(), VerifiedArtifactError> {
    if hash_ref.class != expected_class {
        return Err(VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: format!(
                "wrong artifact hash class: expected `{}`, got `{}`",
                artifact_hash_class_string(expected_class),
                artifact_hash_class_string(hash_ref.class)
            ),
        });
    }
    validate_schema_family(&hash_ref.schema_family, path)?;
    Ok(())
}

fn validate_schema_family(value: &str, path: &str) -> Result<(), VerifiedArtifactError> {
    if value.is_empty() {
        return Err(VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: "schema family must not be empty".to_owned(),
        });
    }
    for segment in value.split('/') {
        if segment.is_empty() {
            return Err(VerifiedArtifactError::InvalidHash {
                path: path.to_owned(),
                reason: "schema family segments must not be empty".to_owned(),
            });
        }
        if !segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(VerifiedArtifactError::InvalidHash {
                path: path.to_owned(),
                reason: "schema family contains invalid characters".to_owned(),
            });
        }
    }
    Ok(())
}

fn integer_from_u64(value: u64, path: &str) -> Result<i64, VerifiedArtifactError> {
    i64::try_from(value).map_err(|_| VerifiedArtifactError::InvalidField {
        path: path.to_owned(),
        reason: "integer exceeds canonical JSON range".to_owned(),
    })
}

fn ensure_sorted_by_key<T, K, F>(
    values: &[T],
    key: F,
    path: &str,
) -> Result<(), VerifiedArtifactError>
where
    K: Ord,
    F: Fn(&T) -> K,
{
    for pair in values.windows(2) {
        if key(&pair[0]) > key(&pair[1]) {
            return Err(VerifiedArtifactError::UnsortedCollection {
                path: path.to_owned(),
            });
        }
    }
    Ok(())
}

fn ensure_no_duplicate_keys<T, K, F>(
    values: &[T],
    key: F,
    path: &str,
) -> Result<(), VerifiedArtifactError>
where
    K: Clone + Ord + fmt::Debug,
    F: Fn(&T) -> K,
{
    let mut seen = BTreeSet::new();
    for value in values {
        let key = key(value);
        if !seen.insert(key.clone()) {
            return Err(VerifiedArtifactError::DuplicateEntry {
                path: path.to_owned(),
                key: format!("{key:?}"),
            });
        }
    }
    Ok(())
}

fn sorted_exports(exports: &[VerifiedExport]) -> Vec<&VerifiedExport> {
    let mut sorted = exports.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|export| export_sort_key(export));
    sorted
}

fn export_sort_key(export: &VerifiedExport) -> (String, String, String, SourceRangeSummary) {
    (
        export.origin_id.clone(),
        export.fully_qualified_name.clone(),
        export.export_kind.clone(),
        export.source_range,
    )
}

fn sorted_expressions(expressions: &[ExpressionMetadata]) -> Vec<&ExpressionMetadata> {
    let mut sorted = expressions.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|expression| expression_sort_key(expression));
    sorted
}

fn expression_sort_key(expression: &ExpressionMetadata) -> (String, SourceRangeSummary) {
    (expression.expression_id.clone(), expression.source_range)
}

fn sorted_obligations(obligations: &[ObligationMetadata]) -> Vec<&ObligationMetadata> {
    let mut sorted = obligations.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|obligation| obligation_sort_key(obligation));
    sorted
}

fn obligation_sort_key(obligation: &ObligationMetadata) -> (String, SourceRangeSummary) {
    (obligation.obligation_id.clone(), obligation.source_range)
}

fn sorted_proof_witnesses(witnesses: &[ProofWitnessRef]) -> Vec<&ProofWitnessRef> {
    let mut sorted = witnesses.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|witness| proof_witness_sort_key(witness));
    sorted
}

fn proof_witness_sort_key(
    witness: &ProofWitnessRef,
) -> (String, String, &'static str, &'static str, String, String) {
    (
        witness.obligation_id.clone(),
        witness.obligation_fingerprint.to_artifact_hash_string(),
        witness_proof_status_string(witness.proof_status),
        witness_evidence_kind_string(witness.evidence_kind),
        witness.witness_path.clone(),
        witness.witness_artifact_hash.to_artifact_hash_string(),
    )
}

fn sorted_diagnostics(diagnostics: &[ArtifactDiagnostic]) -> Vec<&ArtifactDiagnostic> {
    let mut sorted = diagnostics.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|diagnostic| diagnostic_sort_key(diagnostic));
    sorted
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

fn sorted_diagnostic_related(related: &[DiagnosticRelated]) -> Vec<&DiagnosticRelated> {
    let mut sorted = related.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|related| diagnostic_related_sort_key(related));
    sorted
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

fn diagnostic_related_identity_key(
    related: &DiagnosticRelated,
) -> (SourceRangeSummary, String, String) {
    diagnostic_related_sort_key(related)
}

fn sorted_dependency_artifact_hashes(
    dependencies: &[DependencyArtifactHash],
) -> Vec<&DependencyArtifactHash> {
    let mut sorted = dependencies.iter().collect::<Vec<_>>();
    sorted.sort_by_key(|dependency| dependency_artifact_hash_sort_key(dependency));
    sorted
}

fn dependency_artifact_hash_sort_key(dependency: &DependencyArtifactHash) -> ModuleSummaryIdentity {
    dependency.module.clone()
}

fn source_hash_string(hash: Hash) -> String {
    format!("{}:{}", SOURCE_HASH_CONSTRUCTION, lower_hex_hash(hash))
}

fn verified_artifact_hash_string(
    class: ArtifactHashClass,
    schema_version: SchemaVersion,
    hash: Hash,
) -> String {
    artifact_framed_hash_string(
        artifact_hash_class_string(class),
        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
        schema_version,
        hash,
    )
}

fn artifact_framed_hash_string(
    class: &str,
    schema_family: &str,
    schema_version: SchemaVersion,
    hash: Hash,
) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        ARTIFACT_HASH_CONSTRUCTION,
        class,
        schema_family,
        schema_version,
        lower_hex_hash(hash)
    )
}

fn parse_source_hash_string(value: &str, path: &str) -> Result<Hash, VerifiedArtifactError> {
    let Some(hex) = value
        .strip_prefix(SOURCE_HASH_CONSTRUCTION)
        .and_then(|rest| rest.strip_prefix(':'))
    else {
        return Err(VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong source hash construction label".to_owned(),
        });
    };
    parse_lower_hex_hash(hex, path)
}

fn parse_artifact_hash_ref_string(
    value: &str,
    path: &str,
) -> Result<ArtifactHashRef, VerifiedArtifactError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 5 {
        return Err(VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: "expected construction:class:schema_family:schema_version:digest".to_owned(),
        });
    }
    if parts[0] != ARTIFACT_HASH_CONSTRUCTION {
        return Err(VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong artifact hash construction label".to_owned(),
        });
    }
    let class = artifact_hash_class_from_str(parts[1]).ok_or_else(|| {
        VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: "unknown artifact hash class".to_owned(),
        }
    })?;
    validate_schema_family(parts[2], path)?;
    let schema_version =
        parts[3]
            .parse::<SchemaVersion>()
            .map_err(|_| VerifiedArtifactError::InvalidHash {
                path: path.to_owned(),
                reason: "malformed schema version".to_owned(),
            })?;
    let digest = parse_lower_hex_hash(parts[4], path)?;
    Ok(ArtifactHashRef {
        class,
        schema_family: parts[2].to_owned(),
        schema_version,
        digest,
    })
}

fn artifact_hash_class_string(class: ArtifactHashClass) -> &'static str {
    match class {
        ArtifactHashClass::Interface => "interface",
        ArtifactHashClass::Implementation => "implementation",
        ArtifactHashClass::Diagnostic => "diagnostic",
        ArtifactHashClass::Artifact => "artifact",
    }
}

fn artifact_hash_class_from_str(value: &str) -> Option<ArtifactHashClass> {
    match value {
        "interface" => Some(ArtifactHashClass::Interface),
        "implementation" => Some(ArtifactHashClass::Implementation),
        "diagnostic" => Some(ArtifactHashClass::Diagnostic),
        "artifact" => Some(ArtifactHashClass::Artifact),
        _ => None,
    }
}

fn parse_lower_hex_hash(hex: &str, path: &str) -> Result<Hash, VerifiedArtifactError> {
    if hex.len() != Hash::BYTE_LEN * 2 {
        return Err(VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: "digest must be 64 lowercase hexadecimal characters".to_owned(),
        });
    }
    let mut bytes = [0; Hash::BYTE_LEN];
    for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
        let high = parse_lower_hex_nibble(pair[0], path)?;
        let low = parse_lower_hex_nibble(pair[1], path)?;
        bytes[index] = (high << 4) | low;
    }
    Ok(Hash::from_bytes(bytes))
}

fn parse_lower_hex_nibble(byte: u8, path: &str) -> Result<u8, VerifiedArtifactError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(VerifiedArtifactError::InvalidHash {
            path: path.to_owned(),
            reason: "digest must use lowercase hexadecimal".to_owned(),
        }),
    }
}

fn lower_hex_hash(hash: Hash) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(Hash::BYTE_LEN * 2);
    for byte in hash.as_bytes() {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn witness_proof_status_string(status: WitnessProofStatus) -> &'static str {
    match status {
        WitnessProofStatus::KernelVerified => "kernel_verified",
        WitnessProofStatus::DischargedBuiltin => "discharged_builtin",
    }
}

fn witness_evidence_kind_string(kind: EvidenceKind) -> &'static str {
    match kind {
        EvidenceKind::AtpCertificate => "atp_certificate",
        EvidenceKind::BuiltinCertificate => "builtin_certificate",
        EvidenceKind::KernelPrimitive => "kernel_primitive",
    }
}

fn identity_display(identity: &ModuleSummaryIdentity) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        identity.package_id,
        identity.package_version.as_deref().unwrap_or("<none>"),
        identity.lockfile_identity.as_deref().unwrap_or("<none>"),
        identity.module_path,
        identity.language_edition
    )
}

fn field_path(path: &str, field: &str) -> String {
    format!("{path}.{field}")
}

fn array_path(path: &str, index: usize) -> String {
    format!("{path}[{index}]")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use mizar_session::Hash;

    use super::{
        ArtifactDiagnostic, BuildProvenance, DependencyArtifactHash, DiagnosticRelated,
        DiagnosticSeverity, ExportProofStatus, ExportVisibility, ExpressionMetadata,
        ObligationMetadata, ObligationStatus, OverloadMetadata, VerifiedArtifact,
        VerifiedArtifactError, VerifiedArtifactReadOptions, VerifiedExport, current_schema_version,
        read_verified_artifact, verified_artifact_hash_string, verified_artifact_json,
        write_verified_artifact,
    };
    use crate::{
        module_summary::{ModuleSummaryIdentity, SourceRangeSummary},
        proof_witness::{
            EvidenceKind, KernelAcceptanceMetadata, ProofStatus as WitnessProofStatus,
            ProofWitnessRef,
        },
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalJson, SchemaVersion, SchemaVersionError, canonical_json_string},
    };

    #[test]
    fn verified_artifact_round_trips_through_canonical_json() {
        let artifact = sample_artifact();
        let json = verified_artifact_json(&artifact).expect("canonical verified artifact JSON");
        let bytes = write_verified_artifact(&artifact).expect("canonical verified artifact bytes");

        assert_eq!(bytes, canonical_json_string(&json).into_bytes());
        assert_eq!(
            read_verified_artifact(
                &json,
                VerifiedArtifactReadOptions {
                    expected_module: Some(&artifact.module),
                    expected_interface_hash: Some(artifact.interface_hash),
                    expected_implementation_hash: Some(artifact.implementation_hash),
                    ..VerifiedArtifactReadOptions::default()
                }
            )
            .expect("valid verified artifact"),
            artifact
        );
    }

    #[test]
    fn incompatible_version_reads_fail_cleanly() {
        let mut json = sample_json();
        set_field(&mut json, "schema_version", CanonicalJson::string("1.1"));

        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::SchemaVersion(
                SchemaVersionError::MinorTooNew { .. }
            ))
        ));

        let mut json = sample_json();
        object_mut(&mut json).remove("schema_version");
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::SchemaVersion(
                SchemaVersionError::Missing { .. }
            ))
        ));

        let mut json = sample_json();
        set_field(
            &mut json,
            "schema_version",
            CanonicalJson::string("not-a-version"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::SchemaVersion(
                SchemaVersionError::Malformed { .. }
            ))
        ));

        let mut json = sample_json();
        set_field(&mut json, "schema_version", CanonicalJson::string("2.0"));
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::SchemaVersion(
                SchemaVersionError::MajorMismatch { .. }
            ))
        ));
    }

    #[test]
    fn reader_rejects_source_ranges_paths_and_timestamps() {
        let mut json = sample_json();
        let export = array_field_mut(&mut json, "exports").first_mut().unwrap();
        set_nested_range(export, "source_range", 50, 10);
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidField { path, .. })
                if path == "$.exports[0].source_range"
        ));

        let mut json = sample_json();
        set_field(
            &mut json,
            "source_file",
            CanonicalJson::string("../article.miz"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidField { path, .. })
                if path == "$.source_file"
        ));

        let mut json = sample_json();
        set_field(
            &mut json,
            "verified_at",
            CanonicalJson::string("2026-06-22T14:03:05.1Z"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidField { path, .. })
                if path == "$.verified_at"
        ));
    }

    #[test]
    fn reader_rejects_invalid_hash_domains_and_checks_hash_participation() {
        let artifact = sample_artifact();

        let mut json = sample_json();
        set_field(
            &mut json,
            "interface_hash",
            CanonicalJson::string(verified_artifact_hash_string(
                ArtifactHashClass::Diagnostic,
                artifact.schema_version,
                artifact.interface_hash,
            )),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidHash { path, .. })
                if path == "$.interface_hash"
        ));

        let mut json = sample_json();
        let export = array_field_mut(&mut json, "exports").first_mut().unwrap();
        set_object_field(
            export,
            "interface_fingerprint",
            CanonicalJson::string(
                hash_ref(ArtifactHashClass::Diagnostic, "mizar-doc/section", 88)
                    .to_artifact_hash_string(),
            ),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidHash { path, .. })
                if path == "$.exports[0].interface_fingerprint"
        ));

        let mut json = sample_json();
        let export = array_field_mut(&mut json, "exports").first_mut().unwrap();
        set_object_field(
            export,
            "rendered_signature",
            CanonicalJson::string("changed importer signature"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InterfaceHashMismatch { .. })
        ));

        let mut json = sample_json();
        let expression = array_field_mut(&mut json, "expressions")
            .first_mut()
            .unwrap();
        set_object_field(
            expression,
            "rendered_surface",
            CanonicalJson::string("changed local surface"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
        ));
    }

    #[test]
    fn reader_rejects_hash_domain_mismatches_across_schema_fields() {
        let artifact = sample_artifact();

        let mut json = sample_json();
        set_field(
            &mut json,
            "interface_hash",
            CanonicalJson::string(artifact_framed_hash_string_for_test(
                "mizar-artifact/other-schema",
                ArtifactHashClass::Interface,
                artifact.schema_version,
                artifact.interface_hash,
            )),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidHash { path, .. })
                if path == "$.interface_hash"
        ));

        let mut json = sample_json();
        set_field(
            &mut json,
            "implementation_hash",
            CanonicalJson::string(verified_artifact_hash_string(
                ArtifactHashClass::Interface,
                artifact.schema_version,
                artifact.implementation_hash,
            )),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidHash { path, .. })
                if path == "$.implementation_hash"
        ));

        let mut json = sample_json();
        set_field(
            &mut json,
            "source_hash",
            CanonicalJson::string(
                hash_ref(
                    ArtifactHashClass::Artifact,
                    "mizar-artifact/wrong-source",
                    91,
                )
                .to_artifact_hash_string(),
            ),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidHash { path, .. })
                if path == "$.source_hash"
        ));

        let mut json = sample_json();
        let obligation = array_field_mut(&mut json, "obligations")
            .first_mut()
            .unwrap();
        set_object_field(
            obligation,
            "local_context_fingerprint",
            CanonicalJson::string(
                hash_ref(ArtifactHashClass::Artifact, "mizar-vc/local-context", 92)
                    .to_artifact_hash_string(),
            ),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidHash { path, .. })
                if path == "$.obligations[0].local_context_fingerprint"
        ));

        let mut json = sample_json();
        let diagnostic = array_field_mut(&mut json, "diagnostics")
            .first_mut()
            .unwrap();
        set_object_field(
            diagnostic,
            "explanation_ref",
            CanonicalJson::string(
                hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-diagnostics/explanation",
                    93,
                )
                .to_artifact_hash_string(),
            ),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidHash { path, .. })
                if path == "$.diagnostics[0].explanation_ref"
        ));

        let mut json = sample_json();
        set_object_field(
            object_field_mut(&mut json, "provenance"),
            "lockfile_hash",
            CanonicalJson::string(
                hash_ref(ArtifactHashClass::Interface, "mizar-build/lockfile", 94)
                    .to_artifact_hash_string(),
            ),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidHash { path, .. })
                if path == "$.provenance.lockfile_hash"
        ));

        let mut json = sample_json();
        let dependency = array_object_field_mut(
            object_field_mut(&mut json, "provenance"),
            "dependency_artifact_hashes",
        )
        .first_mut()
        .unwrap();
        set_object_field(
            dependency,
            "implementation_hash",
            CanonicalJson::string(
                hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-artifact/verified-artifact",
                    95,
                )
                .to_artifact_hash_string(),
            ),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::InvalidHash { path, .. })
                if path == "$.provenance.dependency_artifact_hashes[0].implementation_hash"
        ));
    }

    #[test]
    fn implementation_hash_participates_in_stable_obligation_witness_diagnostic_and_provenance_fields()
     {
        let mut json = sample_json();
        let obligation = array_field_mut(&mut json, "obligations")
            .last_mut()
            .unwrap();
        set_object_field(
            obligation,
            "statement_summary",
            CanonicalJson::string("changed stable obligation statement"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
        ));

        let mut json = sample_json();
        let witness = array_field_mut(&mut json, "proof_witnesses")
            .first_mut()
            .unwrap();
        set_object_field(
            witness,
            "witness_path",
            CanonicalJson::string("proof-witnesses/hidden/changed.json"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
        ));

        let mut json = sample_json();
        let diagnostic = array_field_mut(&mut json, "diagnostics")
            .first_mut()
            .unwrap();
        set_object_field(
            diagnostic,
            "rendered_message",
            CanonicalJson::string("changed diagnostic message"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
        ));

        let mut json = sample_json();
        set_object_field(
            object_field_mut(&mut json, "provenance"),
            "toolchain",
            CanonicalJson::string("changed-toolchain"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::ImplementationHashMismatch { .. })
        ));
    }

    #[test]
    fn verified_at_and_cache_key_are_hash_excluded() {
        let original = sample_artifact();
        let mut json = sample_json();
        set_field(
            &mut json,
            "verified_at",
            CanonicalJson::string("2026-06-23T00:00:00Z"),
        );
        set_object_field(
            object_field_mut(&mut json, "provenance"),
            "cache_key",
            CanonicalJson::string("new-cache-key"),
        );

        let read = read_verified_artifact(&json, VerifiedArtifactReadOptions::default())
            .expect("hash-excluded metadata change remains valid");
        assert_eq!(read.interface_hash, original.interface_hash);
        assert_eq!(read.implementation_hash, original.implementation_hash);
        assert_eq!(read.verified_at.as_deref(), Some("2026-06-23T00:00:00Z"));
        assert_eq!(read.provenance.cache_key.as_deref(), Some("new-cache-key"));
    }

    #[test]
    fn interface_hash_excludes_local_only_projection_fields() {
        let original = sample_artifact();
        let mut artifact = original.clone();
        artifact.source_file = "articles/renamed-local-path.miz".to_owned();
        artifact.source_hash = hash(96);
        artifact.exports[0].source_range = range(100, 120);
        artifact.exports[0].documentation_ref = Some(hash_ref(
            ArtifactHashClass::Diagnostic,
            "mizar-doc/section",
            97,
        ));
        artifact.expressions[0].rendered_surface = "changed local expression".to_owned();
        artifact.obligations[1].statement_summary = "changed local obligation".to_owned();
        artifact.proof_witnesses[0].witness_path =
            "proof-witnesses/hidden/local-changed.json".to_owned();
        artifact.diagnostics[0].rendered_message = "changed local diagnostic".to_owned();
        artifact.provenance.toolchain = "changed-toolchain".to_owned();
        artifact.provenance.lockfile_hash =
            hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 98);
        artifact.provenance.dependency_artifact_hashes[0].implementation_hash = Some(hash_ref(
            ArtifactHashClass::Implementation,
            "mizar-artifact/verified-artifact",
            99,
        ));
        artifact.refresh_hashes().expect("refresh hashes");

        assert_eq!(artifact.interface_hash, original.interface_hash);
        assert_ne!(artifact.implementation_hash, original.implementation_hash);
    }

    #[test]
    fn reader_rejects_inconsistent_witness_references() {
        let mut json = sample_json();
        let obligation = array_field_mut(&mut json, "obligations")
            .first_mut()
            .unwrap();
        set_object_field(
            obligation,
            "obligation_fingerprint",
            CanonicalJson::string(
                hash_ref(ArtifactHashClass::Interface, "mizar-proof/obligation", 89)
                    .to_artifact_hash_string(),
            ),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
                if path == "$.obligations[0].obligation_fingerprint"
        ));

        let mut json = sample_json();
        let obligation = array_field_mut(&mut json, "obligations")
            .first_mut()
            .unwrap();
        set_object_field(
            obligation,
            "accepted_witness_obligation_id",
            CanonicalJson::string("other-obligation"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
                if path == "$.obligations[0].accepted_witness_obligation_id"
        ));

        let mut json = sample_json();
        let obligation = array_field_mut(&mut json, "obligations")
            .first_mut()
            .unwrap();
        set_object_field(
            obligation,
            "verifier_policy_fingerprint",
            CanonicalJson::string(
                hash_ref(ArtifactHashClass::Interface, "mizar-proof/policy", 90)
                    .to_artifact_hash_string(),
            ),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
                if path == "$.obligations[0].verifier_policy_fingerprint"
        ));
    }

    #[test]
    fn reader_rejects_proof_authority_status_boundary_violations() {
        let mut json = sample_json();
        array_field_mut(&mut json, "proof_witnesses").clear();
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
                if path == "$.obligations[0].accepted_witness_obligation_id"
        ));

        let mut json = sample_json();
        let obligation = array_field_mut(&mut json, "obligations")
            .first_mut()
            .unwrap();
        set_object_field(
            obligation,
            "deterministic_discharge_hash",
            CanonicalJson::string(
                hash_ref(ArtifactHashClass::Interface, "mizar-proof/discharge", 100)
                    .to_artifact_hash_string(),
            ),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
                if path == "$.obligations[0].deterministic_discharge_hash"
        ));

        for status in ["open", "rejected", "externally_attested"] {
            let mut json = sample_json();
            let obligation = array_field_mut(&mut json, "obligations")
                .last_mut()
                .unwrap();
            set_object_field(obligation, "status", CanonicalJson::string(status));
            set_object_field(
                obligation,
                "accepted_witness_obligation_id",
                CanonicalJson::string("obl-2"),
            );
            set_object_field(
                obligation,
                "deterministic_discharge_hash",
                CanonicalJson::null(),
            );
            assert!(matches!(
                read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
                Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
                    if path == "$.obligations[1].accepted_witness_obligation_id"
            ));
        }

        let mut json = sample_json();
        let obligation = array_field_mut(&mut json, "obligations")
            .last_mut()
            .unwrap();
        set_object_field(obligation, "status", CanonicalJson::string("open"));
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
                if path == "$.obligations[1].deterministic_discharge_hash"
        ));
    }

    #[test]
    fn reader_rejects_extra_orphan_and_non_accepted_proof_witnesses() {
        let mut json = sample_json();
        let extra = array_field_mut(&mut json, "proof_witnesses")[0].clone();
        let witnesses = array_field_mut(&mut json, "proof_witnesses");
        witnesses.push(extra);
        let extra_witness = witnesses.last_mut().unwrap();
        set_object_field(
            extra_witness,
            "obligation_id",
            CanonicalJson::string("zzz-missing-obligation"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
                if path == "$.proof_witnesses[1].obligation_id"
        ));

        let mut json = sample_json();
        let extra = array_field_mut(&mut json, "proof_witnesses")[0].clone();
        let witnesses = array_field_mut(&mut json, "proof_witnesses");
        witnesses.push(extra);
        let extra_witness = witnesses.last_mut().unwrap();
        set_object_field(
            extra_witness,
            "obligation_id",
            CanonicalJson::string("obl-2"),
        );
        set_object_field(
            extra_witness,
            "obligation_fingerprint",
            CanonicalJson::string(
                hash_ref(ArtifactHashClass::Interface, "mizar-proof/obligation", 25)
                    .to_artifact_hash_string(),
            ),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::WitnessReferenceMismatch { path, .. })
                if path == "$.proof_witnesses[1].obligation_id"
        ));
    }

    #[test]
    fn reader_rejects_raw_ir_and_ownership_boundary_fields() {
        let mut json = sample_json();
        let expression = array_field_mut(&mut json, "expressions")
            .first_mut()
            .unwrap();
        set_object_field(
            expression,
            "resolved_typed_ast",
            CanonicalJson::string("raw checker dump"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::UnknownField { path, field })
                if path == "$.expressions[0]" && field == "resolved_typed_ast"
        ));

        let mut json = sample_json();
        set_field(
            &mut json,
            "scheduler_state",
            CanonicalJson::string("not-owned-here"),
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::UnknownField { path, field })
                if path == "$" && field == "scheduler_state"
        ));
    }

    #[test]
    fn writer_sorts_diagnostics_and_reader_rejects_unsorted_arrays() {
        let mut artifact = sample_artifact();
        artifact.diagnostics.reverse();
        artifact.diagnostics[0].related.reverse();
        artifact.refresh_hashes().expect("refresh hashes");

        let json = verified_artifact_json(&artifact).expect("writer sorts diagnostics");
        read_verified_artifact(&json, VerifiedArtifactReadOptions::default())
            .expect("writer output is reader-valid");

        let mut unsorted = sample_json();
        array_field_mut(&mut unsorted, "diagnostics").reverse();
        assert!(matches!(
            read_verified_artifact(&unsorted, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::UnsortedCollection { path })
                if path == "$.diagnostics"
        ));

        let mut unsorted_related = sample_json();
        let diagnostic = array_field_mut(&mut unsorted_related, "diagnostics")
            .first_mut()
            .unwrap();
        array_object_field_mut(diagnostic, "related").reverse();
        assert!(matches!(
            read_verified_artifact(&unsorted_related, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::UnsortedCollection { path })
                if path == "$.diagnostics[0].related"
        ));
    }

    #[test]
    fn reader_rejects_duplicate_identity_keys_at_collection_boundaries() {
        for (field, expected_path) in [
            ("exports", "$.exports"),
            ("expressions", "$.expressions"),
            ("obligations", "$.obligations"),
            ("proof_witnesses", "$.proof_witnesses"),
            ("diagnostics", "$.diagnostics"),
        ] {
            let mut json = sample_json();
            duplicate_array_item(array_field_mut(&mut json, field), 0);
            assert!(matches!(
                read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
                Err(VerifiedArtifactError::DuplicateEntry { path, .. })
                    if path == expected_path
            ));
        }

        let mut json = sample_json();
        let diagnostic = array_field_mut(&mut json, "diagnostics")
            .first_mut()
            .unwrap();
        duplicate_array_item(array_object_field_mut(diagnostic, "related"), 0);
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::DuplicateEntry { path, .. })
                if path == "$.diagnostics[0].related"
        ));

        let mut json = sample_json();
        duplicate_array_item(
            array_object_field_mut(
                object_field_mut(&mut json, "provenance"),
                "dependency_artifact_hashes",
            ),
            0,
        );
        assert!(matches!(
            read_verified_artifact(&json, VerifiedArtifactReadOptions::default()),
            Err(VerifiedArtifactError::DuplicateEntry { path, .. })
                if path == "$.provenance.dependency_artifact_hashes"
        ));
    }

    fn sample_json() -> CanonicalJson {
        verified_artifact_json(&sample_artifact()).expect("sample verified artifact JSON")
    }

    fn sample_artifact() -> VerifiedArtifact {
        let schema_version = current_schema_version();
        let module = identity("pkg", "articles/hidden", "2026");
        let dependency_module = identity("dep", "articles/base", "2026");
        let obligation_fingerprint =
            hash_ref(ArtifactHashClass::Interface, "mizar-proof/obligation", 20);
        let verifier_policy = hash_ref(ArtifactHashClass::Interface, "mizar-proof/policy", 21);

        let mut artifact = VerifiedArtifact {
            schema_version,
            module: module.clone(),
            source_file: "articles/hidden.miz".to_owned(),
            source_hash: hash(1),
            verified_at: Some("2026-06-22T14:03:05Z".to_owned()),
            interface_hash: hash(0),
            implementation_hash: hash(0),
            exports: vec![
                VerifiedExport {
                    origin_id: "export-1".to_owned(),
                    fully_qualified_name: "Hidden.Th1".to_owned(),
                    namespace_path: vec!["Hidden".to_owned()],
                    visibility: ExportVisibility::Public,
                    export_kind: "theorem".to_owned(),
                    source_range: range(10, 20),
                    rendered_signature: "for x holds x = x".to_owned(),
                    interface_fingerprint: hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-checker/export",
                        2,
                    ),
                    proof_status: Some(ExportProofStatus::Accepted),
                    documentation_ref: Some(hash_ref(
                        ArtifactHashClass::Diagnostic,
                        "mizar-doc/section",
                        3,
                    )),
                },
                VerifiedExport {
                    origin_id: "export-2".to_owned(),
                    fully_qualified_name: "Hidden.Def1".to_owned(),
                    namespace_path: vec!["Hidden".to_owned()],
                    visibility: ExportVisibility::Reexported,
                    export_kind: "definition".to_owned(),
                    source_range: range(30, 44),
                    rendered_signature: "func f -> object".to_owned(),
                    interface_fingerprint: hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-checker/export",
                        4,
                    ),
                    proof_status: Some(ExportProofStatus::NotRequired),
                    documentation_ref: None,
                },
            ],
            expressions: vec![
                ExpressionMetadata {
                    expression_id: "expr-1".to_owned(),
                    source_range: range(11, 12),
                    expression_kind: "term".to_owned(),
                    rendered_surface: "x".to_owned(),
                    inferred_type: Some("object".to_owned()),
                    resolved_symbol: Some("Hidden.x".to_owned()),
                    inserted_coercions: vec!["object-coercion".to_owned()],
                    active_thesis: Some("x = x".to_owned()),
                    overload_resolution: Some(OverloadMetadata {
                        root_symbol: "equals".to_owned(),
                        selected_candidate: "builtin.eq".to_owned(),
                        active_refinements: vec!["object".to_owned()],
                        coercion_summary: Some("identity".to_owned()),
                    }),
                },
                ExpressionMetadata {
                    expression_id: "expr-2".to_owned(),
                    source_range: range(34, 39),
                    expression_kind: "definition_head".to_owned(),
                    rendered_surface: "f".to_owned(),
                    inferred_type: None,
                    resolved_symbol: None,
                    inserted_coercions: Vec::new(),
                    active_thesis: None,
                    overload_resolution: None,
                },
            ],
            obligations: vec![
                ObligationMetadata {
                    obligation_id: "obl-1".to_owned(),
                    obligation_anchor: Some("Hidden.Th1.proof".to_owned()),
                    owner_origin_id: Some("export-1".to_owned()),
                    source_range: range(12, 18),
                    obligation_kind: "theorem_body".to_owned(),
                    statement_summary: "x = x".to_owned(),
                    obligation_fingerprint: obligation_fingerprint.clone(),
                    vc_fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-vc/vc", 22),
                    local_context_fingerprint: hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-vc/local-context",
                        23,
                    ),
                    dependency_slice_fingerprint: hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-vc/dependency-slice",
                        24,
                    ),
                    verifier_policy_fingerprint: verifier_policy.clone(),
                    status: ObligationStatus::Accepted,
                    accepted_witness_obligation_id: Some("obl-1".to_owned()),
                    deterministic_discharge_hash: None,
                    diagnostic_ref: None,
                },
                ObligationMetadata {
                    obligation_id: "obl-2".to_owned(),
                    obligation_anchor: None,
                    owner_origin_id: Some("export-2".to_owned()),
                    source_range: range(30, 44),
                    obligation_kind: "definition_totality".to_owned(),
                    statement_summary: "definition requires no proof".to_owned(),
                    obligation_fingerprint: hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-proof/obligation",
                        25,
                    ),
                    vc_fingerprint: hash_ref(ArtifactHashClass::Interface, "mizar-vc/vc", 26),
                    local_context_fingerprint: hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-vc/local-context",
                        27,
                    ),
                    dependency_slice_fingerprint: hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-vc/dependency-slice",
                        28,
                    ),
                    verifier_policy_fingerprint: verifier_policy.clone(),
                    status: ObligationStatus::NotRequired,
                    accepted_witness_obligation_id: None,
                    deterministic_discharge_hash: Some(hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-proof/discharge",
                        29,
                    )),
                    diagnostic_ref: Some(hash_ref(
                        ArtifactHashClass::Diagnostic,
                        "mizar-diagnostics/obligation",
                        30,
                    )),
                },
            ],
            proof_witnesses: vec![ProofWitnessRef {
                schema_version,
                obligation_id: "obl-1".to_owned(),
                obligation_fingerprint,
                proof_status: WitnessProofStatus::KernelVerified,
                evidence_kind: EvidenceKind::AtpCertificate,
                witness_path: "proof-witnesses/hidden/obl-1.json".to_owned(),
                witness_artifact_hash: hash_ref(
                    ArtifactHashClass::Artifact,
                    "mizar-proof/witness-file",
                    31,
                ),
                kernel_acceptance: KernelAcceptanceMetadata {
                    kernel_profile_fingerprint: hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-kernel/profile",
                        32,
                    ),
                    verifier_policy_fingerprint: verifier_policy,
                    checker_schema_version: schema_version,
                    certificate_format: Some("atp-cert-v1".to_owned()),
                    accepted_result_hash: hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-kernel/accepted-result",
                        33,
                    ),
                    used_axioms_hash: Some(hash_ref(
                        ArtifactHashClass::Diagnostic,
                        "mizar-kernel/used-axioms",
                        34,
                    )),
                },
            }],
            diagnostics: vec![
                ArtifactDiagnostic {
                    diagnostic_id: "diag-1".to_owned(),
                    code: "MZ1001".to_owned(),
                    severity: DiagnosticSeverity::Warning,
                    primary_range: Some(range(12, 18)),
                    message_key: "proof.used_simplification".to_owned(),
                    rendered_message: "simplification used".to_owned(),
                    related: vec![
                        DiagnosticRelated {
                            source_range: range(1, 2),
                            message_key: "context.first".to_owned(),
                            rendered_message: "first context".to_owned(),
                        },
                        DiagnosticRelated {
                            source_range: range(3, 4),
                            message_key: "context.second".to_owned(),
                            rendered_message: "second context".to_owned(),
                        },
                    ],
                    explanation_ref: Some(hash_ref(
                        ArtifactHashClass::Diagnostic,
                        "mizar-diagnostics/explanation",
                        35,
                    )),
                },
                ArtifactDiagnostic {
                    diagnostic_id: "diag-2".to_owned(),
                    code: "MZ2001".to_owned(),
                    severity: DiagnosticSeverity::Info,
                    primary_range: None,
                    message_key: "artifact.note".to_owned(),
                    rendered_message: "artifact note".to_owned(),
                    related: Vec::new(),
                    explanation_ref: None,
                },
            ],
            provenance: BuildProvenance {
                toolchain: "mizar-evo-test".to_owned(),
                language_edition: "2026".to_owned(),
                lockfile_hash: hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 36),
                verifier_config_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-build/verifier-config",
                    37,
                ),
                dependency_artifact_hashes: vec![DependencyArtifactHash {
                    module: dependency_module,
                    interface_hash: hash_ref(
                        ArtifactHashClass::Interface,
                        "mizar-artifact/module-summary",
                        38,
                    ),
                    implementation_hash: Some(hash_ref(
                        ArtifactHashClass::Implementation,
                        "mizar-artifact/verified-artifact",
                        39,
                    )),
                    artifact_hash: Some(hash_ref(
                        ArtifactHashClass::Artifact,
                        "mizar-artifact/file",
                        40,
                    )),
                }],
                cache_key: Some("cache-key-1".to_owned()),
            },
        };
        artifact.refresh_hashes().expect("sample hashes");
        artifact
    }

    fn identity(package_id: &str, module_path: &str, edition: &str) -> ModuleSummaryIdentity {
        ModuleSummaryIdentity {
            package_id: package_id.to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("lock".to_owned()),
            module_path: module_path.to_owned(),
            language_edition: edition.to_owned(),
        }
    }

    fn range(start_byte: u64, end_byte: u64) -> SourceRangeSummary {
        SourceRangeSummary {
            start_byte,
            end_byte,
        }
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }

    fn hash_ref(class: ArtifactHashClass, family: &str, seed: u8) -> ArtifactHashRef {
        ArtifactHashRef::new(class, family, SchemaVersion::new(1, 0), hash(seed))
    }

    fn artifact_framed_hash_string_for_test(
        family: &str,
        class: ArtifactHashClass,
        schema_version: SchemaVersion,
        digest: Hash,
    ) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            crate::store::ARTIFACT_HASH_CONSTRUCTION,
            super::artifact_hash_class_string(class),
            family,
            schema_version,
            super::lower_hex_hash(digest)
        )
    }

    fn set_field(json: &mut CanonicalJson, field: &str, value: CanonicalJson) {
        object_mut(json).insert(field.to_owned(), value);
    }

    fn set_object_field(json: &mut CanonicalJson, field: &str, value: CanonicalJson) {
        object_mut(json).insert(field.to_owned(), value);
    }

    fn set_nested_range(json: &mut CanonicalJson, field: &str, start: i64, end: i64) {
        let range = object_field_mut(json, field);
        let fields = object_mut(range);
        fields.insert("start_byte".to_owned(), CanonicalJson::integer(start));
        fields.insert("end_byte".to_owned(), CanonicalJson::integer(end));
    }

    fn object_field_mut<'a>(json: &'a mut CanonicalJson, field: &str) -> &'a mut CanonicalJson {
        object_mut(json).get_mut(field).expect("object field")
    }

    fn array_field_mut<'a>(json: &'a mut CanonicalJson, field: &str) -> &'a mut Vec<CanonicalJson> {
        array_mut(object_field_mut(json, field))
    }

    fn array_object_field_mut<'a>(
        json: &'a mut CanonicalJson,
        field: &str,
    ) -> &'a mut Vec<CanonicalJson> {
        array_mut(object_field_mut(json, field))
    }

    fn object_mut(json: &mut CanonicalJson) -> &mut BTreeMap<String, CanonicalJson> {
        let CanonicalJson::Object(fields) = json else {
            panic!("expected object");
        };
        fields
    }

    fn array_mut(json: &mut CanonicalJson) -> &mut Vec<CanonicalJson> {
        let CanonicalJson::Array(values) = json else {
            panic!("expected array");
        };
        values
    }

    fn duplicate_array_item(values: &mut Vec<CanonicalJson>, index: usize) {
        let duplicate = values[index].clone();
        values.insert(index + 1, duplicate);
    }
}
