//! Published verified module artifact schema.
//!
//! Design authority: [verified_artifact.md](../../../../doc/design/mizar-artifact/en/verified_artifact.md).

use crate::module_summary::ExportedItem;
use crate::proof_witness::ProofWitnessRef;
use crate::store::{
    ArtifactError, CanonicalJson, canonical_hash, check_schema_version, ensure_sorted_by,
    ensure_unique_by, expect_object, hash_json, optional_string, optional_string_json, parse_json,
    reject_raw_ir_fields, reject_unknown_fields, required_array, required_hash, required_string,
    required_u64, validate_portable_path_text,
};
use mizar_session::Hash;

/// Current `VerifiedArtifact` schema version.
pub const VERIFIED_ARTIFACT_SCHEMA_VERSION: &str = "mizar-artifact/verified-artifact/v1";

const SCHEMA: &str = "VerifiedArtifact";
const SUPPORTED: &[&str] = &[VERIFIED_ARTIFACT_SCHEMA_VERSION];

/// Export projection used by `VerifiedArtifact`.
pub type ArtifactExport = ExportedItem;

/// Stable content reference used for diagnostic explanation previews.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRef {
    /// Portable path.
    pub path: String,
    /// Referenced content hash.
    pub content_hash: Hash,
    /// Referenced schema version.
    pub schema_version: String,
}

/// Stable metadata key/value field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataField {
    /// Stable metadata key.
    pub key: String,
    /// Stable rendered metadata value.
    pub value: String,
}

/// Stable expression metadata projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionMetadata {
    /// Stable expression id.
    pub expression_id: String,
    /// Source order used for diagnostics and display.
    pub source_order: u64,
    /// Expression kind.
    pub kind: String,
    /// Rendered type or fact summary.
    pub rendered_type: String,
    /// Stable metadata fields.
    pub metadata: Vec<MetadataField>,
}

/// Stable obligation metadata projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationMetadata {
    /// Stable obligation id.
    pub obligation_id: String,
    /// Source order used for artifact display.
    pub source_order: u64,
    /// Obligation kind.
    pub kind: String,
    /// Projected proof status.
    pub proof_status: String,
    /// Optional witness id for accepted evidence.
    pub witness_id: Option<String>,
}

/// Projected diagnostic with stable ordering and optional explanation ref.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectedDiagnostic {
    /// Source order assigned by the producer.
    pub source_order: u64,
    /// Stable diagnostic code.
    pub code: String,
    /// Stable severity spelling.
    pub severity: String,
    /// User-facing message.
    pub message: String,
    /// Optional explanation preview reference.
    pub explanation_ref: Option<ArtifactRef>,
}

/// Dependency artifact hash recorded in provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyArtifactHash {
    /// Dependency artifact identifier.
    pub artifact: String,
    /// Dependency artifact content hash.
    pub content_hash: Hash,
}

/// Build provenance attached to a verified artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildProvenance {
    /// Toolchain identity.
    pub toolchain: String,
    /// Language edition.
    pub language_edition: String,
    /// Lockfile hash.
    pub lockfile_hash: Hash,
    /// Verifier configuration hash.
    pub verifier_config_hash: Hash,
    /// Dependency artifact hashes.
    pub dependency_artifact_hashes: Vec<DependencyArtifactHash>,
    /// Build cache key or equivalent reproducibility key.
    pub cache_key: Hash,
    /// Local run id excluded from reproducibility hashes.
    pub local_run_id: Option<String>,
}

/// Stable published projection for one verified module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedArtifact {
    /// Schema version.
    pub schema_version: String,
    /// Portable source file path.
    pub source_file: String,
    /// Source content hash.
    pub source_hash: Hash,
    /// Local verification timestamp, excluded from canonical hashes.
    pub verified_at: Option<String>,
    /// Module path.
    pub module_path: String,
    /// Importer-visible interface hash.
    pub interface_hash: Hash,
    /// Stable implementation projection hash.
    pub implementation_hash: Hash,
    /// Export projections.
    pub exports: Vec<ArtifactExport>,
    /// Expression metadata.
    pub expressions: Vec<ExpressionMetadata>,
    /// Obligation metadata.
    pub obligations: Vec<ObligationMetadata>,
    /// External proof witness references.
    pub proof_witnesses: Vec<ProofWitnessRef>,
    /// Projected diagnostics.
    pub diagnostics: Vec<ProjectedDiagnostic>,
    /// Reproducibility provenance.
    pub provenance: BuildProvenance,
}

impl ArtifactRef {
    /// Creates a stable artifact reference.
    pub fn new(
        path: impl Into<String>,
        content_hash: Hash,
        schema_version: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            content_hash,
            schema_version: schema_version.into(),
        }
    }

    /// Validates referenced bytes against the recorded hash.
    pub fn validate_content(&self, content: &str) -> Result<(), ArtifactError> {
        let actual = crate::store::content_hash_str(content);
        if actual == self.content_hash {
            Ok(())
        } else {
            Err(ArtifactError::HashMismatch {
                expected: self.content_hash,
                actual,
            })
        }
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("content_hash", hash_json(self.content_hash)),
            ("path", CanonicalJson::string(&self.path)),
            (
                "schema_version",
                CanonicalJson::string(&self.schema_version),
            ),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(object, SCHEMA, &["content_hash", "path", "schema_version"])?;
        let path = required_string(object, SCHEMA, "path")?;
        validate_portable_path_text(&path)?;
        Ok(Self::new(
            path,
            required_hash(object, SCHEMA, "content_hash")?,
            required_string(object, SCHEMA, "schema_version")?,
        ))
    }
}

impl MetadataField {
    /// Creates a stable metadata field and rejects raw-IR shaped keys.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Result<Self, ArtifactError> {
        let key = key.into();
        if is_forbidden_metadata_key(&key) {
            return Err(ArtifactError::RawIrPayload { field: key });
        }
        Ok(Self {
            key,
            value: value.into(),
        })
    }

    fn sort_key(&self) -> String {
        self.key.clone()
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("key", CanonicalJson::string(&self.key)),
            ("value", CanonicalJson::string(&self.value)),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(object, SCHEMA, &["key", "value"])?;
        Self::new(
            required_string(object, SCHEMA, "key")?,
            required_string(object, SCHEMA, "value")?,
        )
    }
}

impl ExpressionMetadata {
    /// Creates stable expression metadata.
    pub fn new(
        expression_id: impl Into<String>,
        source_order: u64,
        kind: impl Into<String>,
        rendered_type: impl Into<String>,
        metadata: Vec<MetadataField>,
    ) -> Self {
        let mut expression = Self {
            expression_id: expression_id.into(),
            source_order,
            kind: kind.into(),
            rendered_type: rendered_type.into(),
            metadata,
        };
        expression.metadata.sort_by_key(MetadataField::sort_key);
        expression
    }

    fn sort_key(&self) -> String {
        format!("{}\u{1f}{:020}", self.expression_id, self.source_order)
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("expression_id", CanonicalJson::string(&self.expression_id)),
            ("kind", CanonicalJson::string(&self.kind)),
            (
                "metadata",
                CanonicalJson::array(
                    self.metadata
                        .iter()
                        .map(MetadataField::to_json_value)
                        .collect(),
                ),
            ),
            ("rendered_type", CanonicalJson::string(&self.rendered_type)),
            ("source_order", CanonicalJson::Integer(self.source_order)),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "expression_id",
                "kind",
                "metadata",
                "rendered_type",
                "source_order",
            ],
        )?;
        let metadata = required_array(object, SCHEMA, "metadata")?
            .iter()
            .map(MetadataField::from_json_value)
            .collect::<Result<Vec<_>, _>>()?;
        let expression = Self {
            expression_id: required_string(object, SCHEMA, "expression_id")?,
            source_order: required_u64(object, SCHEMA, "source_order")?,
            kind: required_string(object, SCHEMA, "kind")?,
            rendered_type: required_string(object, SCHEMA, "rendered_type")?,
            metadata,
        };
        ensure_sorted_by(
            &expression.metadata,
            SCHEMA,
            "metadata",
            MetadataField::sort_key,
        )?;
        Ok(expression)
    }
}

impl ObligationMetadata {
    /// Creates stable obligation metadata.
    pub fn new(
        obligation_id: impl Into<String>,
        source_order: u64,
        kind: impl Into<String>,
        proof_status: impl Into<String>,
        witness_id: Option<String>,
    ) -> Self {
        Self {
            obligation_id: obligation_id.into(),
            source_order,
            kind: kind.into(),
            proof_status: proof_status.into(),
            witness_id,
        }
    }

    fn sort_key(&self) -> String {
        format!("{}\u{1f}{:020}", self.obligation_id, self.source_order)
    }

    fn interface_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("kind", CanonicalJson::string(&self.kind)),
            ("obligation_id", CanonicalJson::string(&self.obligation_id)),
            ("proof_status", CanonicalJson::string(&self.proof_status)),
        ])
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("kind", CanonicalJson::string(&self.kind)),
            ("obligation_id", CanonicalJson::string(&self.obligation_id)),
            ("proof_status", CanonicalJson::string(&self.proof_status)),
            ("source_order", CanonicalJson::Integer(self.source_order)),
            ("witness_id", optional_string_json(&self.witness_id)),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "kind",
                "obligation_id",
                "proof_status",
                "source_order",
                "witness_id",
            ],
        )?;
        Ok(Self::new(
            required_string(object, SCHEMA, "obligation_id")?,
            required_u64(object, SCHEMA, "source_order")?,
            required_string(object, SCHEMA, "kind")?,
            required_string(object, SCHEMA, "proof_status")?,
            optional_string(object, SCHEMA, "witness_id")?,
        ))
    }
}

impl ProjectedDiagnostic {
    /// Creates a projected diagnostic.
    pub fn new(
        source_order: u64,
        code: impl Into<String>,
        severity: impl Into<String>,
        message: impl Into<String>,
        explanation_ref: Option<ArtifactRef>,
    ) -> Self {
        Self {
            source_order,
            code: code.into(),
            severity: severity.into(),
            message: message.into(),
            explanation_ref,
        }
    }

    fn sort_key(&self) -> String {
        format!(
            "{:020}\u{1f}{}\u{1f}{}",
            self.source_order, self.code, self.message
        )
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("code", CanonicalJson::string(&self.code)),
            (
                "explanation_ref",
                self.explanation_ref
                    .as_ref()
                    .map_or(CanonicalJson::Null, ArtifactRef::to_json_value),
            ),
            ("message", CanonicalJson::string(&self.message)),
            ("severity", CanonicalJson::string(&self.severity)),
            ("source_order", CanonicalJson::Integer(self.source_order)),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "code",
                "explanation_ref",
                "message",
                "severity",
                "source_order",
            ],
        )?;
        let explanation_ref = match object.get("explanation_ref") {
            Some(CanonicalJson::Null) | None => None,
            Some(value) => Some(ArtifactRef::from_json_value(value)?),
        };
        Ok(Self::new(
            required_u64(object, SCHEMA, "source_order")?,
            required_string(object, SCHEMA, "code")?,
            required_string(object, SCHEMA, "severity")?,
            required_string(object, SCHEMA, "message")?,
            explanation_ref,
        ))
    }
}

impl DependencyArtifactHash {
    /// Creates a dependency artifact hash record.
    pub fn new(artifact: impl Into<String>, content_hash: Hash) -> Self {
        Self {
            artifact: artifact.into(),
            content_hash,
        }
    }

    fn sort_key(&self) -> String {
        format!(
            "{}\u{1f}{}",
            self.artifact,
            crate::store::hash_to_hex(self.content_hash)
        )
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("artifact", CanonicalJson::string(&self.artifact)),
            ("content_hash", hash_json(self.content_hash)),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(object, SCHEMA, &["artifact", "content_hash"])?;
        Ok(Self::new(
            required_string(object, SCHEMA, "artifact")?,
            required_hash(object, SCHEMA, "content_hash")?,
        ))
    }
}

impl BuildProvenance {
    /// Creates provenance and sorts dependency artifact hashes.
    pub fn new(
        toolchain: impl Into<String>,
        language_edition: impl Into<String>,
        lockfile_hash: Hash,
        verifier_config_hash: Hash,
        dependency_artifact_hashes: Vec<DependencyArtifactHash>,
        cache_key: Hash,
        local_run_id: Option<String>,
    ) -> Self {
        let mut provenance = Self {
            toolchain: toolchain.into(),
            language_edition: language_edition.into(),
            lockfile_hash,
            verifier_config_hash,
            dependency_artifact_hashes,
            cache_key,
            local_run_id,
        };
        provenance
            .dependency_artifact_hashes
            .sort_by_key(DependencyArtifactHash::sort_key);
        provenance
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("cache_key", hash_json(self.cache_key)),
            (
                "dependency_artifact_hashes",
                CanonicalJson::array(
                    self.dependency_artifact_hashes
                        .iter()
                        .map(DependencyArtifactHash::to_json_value)
                        .collect(),
                ),
            ),
            (
                "language_edition",
                CanonicalJson::string(&self.language_edition),
            ),
            ("local_run_id", optional_string_json(&self.local_run_id)),
            ("lockfile_hash", hash_json(self.lockfile_hash)),
            ("toolchain", CanonicalJson::string(&self.toolchain)),
            ("verifier_config_hash", hash_json(self.verifier_config_hash)),
        ])
    }

    fn hash_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("cache_key", hash_json(self.cache_key)),
            (
                "dependency_artifact_hashes",
                CanonicalJson::array(
                    self.dependency_artifact_hashes
                        .iter()
                        .map(DependencyArtifactHash::to_json_value)
                        .collect(),
                ),
            ),
            (
                "language_edition",
                CanonicalJson::string(&self.language_edition),
            ),
            ("lockfile_hash", hash_json(self.lockfile_hash)),
            ("toolchain", CanonicalJson::string(&self.toolchain)),
            ("verifier_config_hash", hash_json(self.verifier_config_hash)),
        ])
    }

    fn from_json_value(value: &CanonicalJson) -> Result<Self, ArtifactError> {
        let object = expect_object(value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "cache_key",
                "dependency_artifact_hashes",
                "language_edition",
                "local_run_id",
                "lockfile_hash",
                "toolchain",
                "verifier_config_hash",
            ],
        )?;
        let dependency_artifact_hashes =
            required_array(object, SCHEMA, "dependency_artifact_hashes")?
                .iter()
                .map(DependencyArtifactHash::from_json_value)
                .collect::<Result<Vec<_>, _>>()?;
        let provenance = Self {
            toolchain: required_string(object, SCHEMA, "toolchain")?,
            language_edition: required_string(object, SCHEMA, "language_edition")?,
            lockfile_hash: required_hash(object, SCHEMA, "lockfile_hash")?,
            verifier_config_hash: required_hash(object, SCHEMA, "verifier_config_hash")?,
            dependency_artifact_hashes,
            cache_key: required_hash(object, SCHEMA, "cache_key")?,
            local_run_id: optional_string(object, SCHEMA, "local_run_id")?,
        };
        ensure_sorted_by(
            &provenance.dependency_artifact_hashes,
            SCHEMA,
            "dependency_artifact_hashes",
            DependencyArtifactHash::sort_key,
        )?;
        Ok(provenance)
    }
}

impl VerifiedArtifact {
    /// Creates a verified artifact and computes interface/implementation hashes.
    #[allow(clippy::too_many_arguments)] // reason: constructor mirrors stable verified artifact fields
    pub fn new(
        source_file: impl Into<String>,
        source_hash: Hash,
        verified_at: Option<String>,
        module_path: impl Into<String>,
        exports: Vec<ArtifactExport>,
        expressions: Vec<ExpressionMetadata>,
        obligations: Vec<ObligationMetadata>,
        proof_witnesses: Vec<ProofWitnessRef>,
        diagnostics: Vec<ProjectedDiagnostic>,
        provenance: BuildProvenance,
    ) -> Self {
        let mut artifact = Self {
            schema_version: VERIFIED_ARTIFACT_SCHEMA_VERSION.to_owned(),
            source_file: source_file.into(),
            source_hash,
            verified_at,
            module_path: module_path.into(),
            interface_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
            implementation_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
            exports,
            expressions,
            obligations,
            proof_witnesses,
            diagnostics,
            provenance,
        };
        artifact.sort_canonical();
        artifact.interface_hash = artifact.compute_interface_hash();
        artifact.implementation_hash = artifact.compute_implementation_hash();
        artifact
    }

    /// Serializes this artifact as canonical JSON.
    pub fn to_json_string(&self) -> String {
        self.to_json_value().to_canonical_string()
    }

    /// Reads and validates a verified artifact from JSON text.
    pub fn from_json_str(input: &str) -> Result<Self, ArtifactError> {
        let value = parse_json(input)?;
        reject_raw_ir_fields(&value)?;
        let object = expect_object(&value, SCHEMA)?;
        reject_unknown_fields(
            object,
            SCHEMA,
            &[
                "schema_version",
                "source_file",
                "source_hash",
                "verified_at",
                "module_path",
                "interface_hash",
                "implementation_hash",
                "exports",
                "expressions",
                "obligations",
                "proof_witnesses",
                "diagnostics",
                "provenance",
            ],
        )?;
        let schema_version = check_schema_version(object, SCHEMA, SUPPORTED)?;
        let source_file = required_string(object, SCHEMA, "source_file")?;
        validate_portable_path_text(&source_file)?;
        let mut artifact = Self {
            schema_version,
            source_file,
            source_hash: required_hash(object, SCHEMA, "source_hash")?,
            verified_at: optional_string(object, SCHEMA, "verified_at")?,
            module_path: required_string(object, SCHEMA, "module_path")?,
            interface_hash: required_hash(object, SCHEMA, "interface_hash")?,
            implementation_hash: required_hash(object, SCHEMA, "implementation_hash")?,
            exports: required_array(object, SCHEMA, "exports")?
                .iter()
                .map(ArtifactExport::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
            expressions: required_array(object, SCHEMA, "expressions")?
                .iter()
                .map(ExpressionMetadata::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
            obligations: required_array(object, SCHEMA, "obligations")?
                .iter()
                .map(ObligationMetadata::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
            proof_witnesses: required_array(object, SCHEMA, "proof_witnesses")?
                .iter()
                .map(ProofWitnessRef::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
            diagnostics: required_array(object, SCHEMA, "diagnostics")?
                .iter()
                .map(ProjectedDiagnostic::from_json_value)
                .collect::<Result<Vec<_>, _>>()?,
            provenance: BuildProvenance::from_json_value(object.get("provenance").ok_or(
                ArtifactError::MissingField {
                    schema: SCHEMA,
                    field: "provenance",
                },
            )?)?,
        };
        artifact.validate_order_and_uniqueness()?;
        let computed_interface = artifact.compute_interface_hash();
        if computed_interface != artifact.interface_hash {
            return Err(ArtifactError::HashMismatch {
                expected: artifact.interface_hash,
                actual: computed_interface,
            });
        }
        let computed_implementation = artifact.compute_implementation_hash();
        if computed_implementation != artifact.implementation_hash {
            return Err(ArtifactError::HashMismatch {
                expected: artifact.implementation_hash,
                actual: computed_implementation,
            });
        }
        artifact.sort_canonical();
        Ok(artifact)
    }

    /// Computes a stable diagnostic hash for the diagnostic projection.
    pub fn diagnostics_hash(&self) -> Hash {
        canonical_hash("verified-artifact/diagnostics/v1", &self.diagnostics_json())
    }

    fn to_json_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("diagnostics", self.diagnostics_json()),
            ("exports", self.exports_json()),
            ("expressions", self.expressions_json()),
            ("implementation_hash", hash_json(self.implementation_hash)),
            ("interface_hash", hash_json(self.interface_hash)),
            ("module_path", CanonicalJson::string(&self.module_path)),
            ("obligations", self.obligations_json()),
            ("proof_witnesses", self.witnesses_json()),
            ("provenance", self.provenance.to_json_value()),
            (
                "schema_version",
                CanonicalJson::string(&self.schema_version),
            ),
            ("source_file", CanonicalJson::string(&self.source_file)),
            ("source_hash", hash_json(self.source_hash)),
            ("verified_at", optional_string_json(&self.verified_at)),
        ])
    }

    fn interface_hash_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            (
                "dependency_artifact_hashes",
                CanonicalJson::array(
                    self.provenance
                        .dependency_artifact_hashes
                        .iter()
                        .map(DependencyArtifactHash::to_json_value)
                        .collect(),
                ),
            ),
            ("exports", self.exports_json()),
            ("module_path", CanonicalJson::string(&self.module_path)),
            (
                "obligations",
                CanonicalJson::array(
                    self.obligations
                        .iter()
                        .map(ObligationMetadata::interface_json_value)
                        .collect(),
                ),
            ),
            (
                "schema_version",
                CanonicalJson::string(&self.schema_version),
            ),
        ])
    }

    fn implementation_hash_value(&self) -> CanonicalJson {
        CanonicalJson::object([
            ("diagnostics", self.diagnostics_json()),
            ("exports", self.exports_json()),
            ("expressions", self.expressions_json()),
            ("interface_hash", hash_json(self.interface_hash)),
            ("module_path", CanonicalJson::string(&self.module_path)),
            ("obligations", self.obligations_json()),
            ("proof_witnesses", self.witnesses_json()),
            ("provenance", self.provenance.hash_json_value()),
            (
                "schema_version",
                CanonicalJson::string(&self.schema_version),
            ),
            ("source_file", CanonicalJson::string(&self.source_file)),
            ("source_hash", hash_json(self.source_hash)),
        ])
    }

    fn compute_interface_hash(&self) -> Hash {
        canonical_hash(
            "verified-artifact/interface/v1",
            &self.interface_hash_value(),
        )
    }

    fn compute_implementation_hash(&self) -> Hash {
        canonical_hash(
            "verified-artifact/implementation/v1",
            &self.implementation_hash_value(),
        )
    }

    fn exports_json(&self) -> CanonicalJson {
        CanonicalJson::array(
            self.exports
                .iter()
                .map(ArtifactExport::to_json_value)
                .collect(),
        )
    }

    fn expressions_json(&self) -> CanonicalJson {
        CanonicalJson::array(
            self.expressions
                .iter()
                .map(ExpressionMetadata::to_json_value)
                .collect(),
        )
    }

    fn obligations_json(&self) -> CanonicalJson {
        CanonicalJson::array(
            self.obligations
                .iter()
                .map(ObligationMetadata::to_json_value)
                .collect(),
        )
    }

    fn witnesses_json(&self) -> CanonicalJson {
        CanonicalJson::array(
            self.proof_witnesses
                .iter()
                .map(ProofWitnessRef::to_json_value)
                .collect(),
        )
    }

    fn diagnostics_json(&self) -> CanonicalJson {
        CanonicalJson::array(
            self.diagnostics
                .iter()
                .map(ProjectedDiagnostic::to_json_value)
                .collect(),
        )
    }

    fn sort_canonical(&mut self) {
        self.exports.sort_by_key(ArtifactExport::sort_key);
        self.expressions.sort_by_key(ExpressionMetadata::sort_key);
        self.obligations.sort_by_key(ObligationMetadata::sort_key);
        self.proof_witnesses.sort_by_key(ProofWitnessRef::sort_key);
        self.diagnostics.sort_by_key(ProjectedDiagnostic::sort_key);
        self.provenance
            .dependency_artifact_hashes
            .sort_by_key(DependencyArtifactHash::sort_key);
    }

    fn validate_order_and_uniqueness(&self) -> Result<(), ArtifactError> {
        ensure_sorted_by(&self.exports, SCHEMA, "exports", ArtifactExport::sort_key)?;
        ensure_sorted_by(
            &self.expressions,
            SCHEMA,
            "expressions",
            ExpressionMetadata::sort_key,
        )?;
        ensure_sorted_by(
            &self.obligations,
            SCHEMA,
            "obligations",
            ObligationMetadata::sort_key,
        )?;
        ensure_sorted_by(
            &self.proof_witnesses,
            SCHEMA,
            "proof_witnesses",
            ProofWitnessRef::sort_key,
        )?;
        ensure_sorted_by(
            &self.diagnostics,
            SCHEMA,
            "diagnostics",
            ProjectedDiagnostic::sort_key,
        )?;
        ensure_unique_by(&self.exports, SCHEMA, ArtifactExport::sort_key)?;
        ensure_unique_by(&self.expressions, SCHEMA, ExpressionMetadata::sort_key)?;
        ensure_unique_by(&self.obligations, SCHEMA, ObligationMetadata::sort_key)?;
        ensure_unique_by(&self.proof_witnesses, SCHEMA, ProofWitnessRef::sort_key)?;
        ensure_unique_by(&self.diagnostics, SCHEMA, ProjectedDiagnostic::sort_key)
    }
}

fn is_forbidden_metadata_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    [
        "raw_ir",
        "surface_ast",
        "resolved_ast",
        "typed_ast",
        "core_ir",
        "control_flow_ir",
        "vc_ir",
        "atp_problem",
        "kernel_state",
    ]
    .iter()
    .any(|forbidden| normalized.contains(forbidden))
}

#[cfg(test)]
mod tests {
    use super::{
        ArtifactRef, BuildProvenance, DependencyArtifactHash, ExpressionMetadata, MetadataField,
        ObligationMetadata, ProjectedDiagnostic, VerifiedArtifact,
    };
    use crate::module_summary::ExportedItem;
    use crate::proof_witness::ProofWitnessRef;
    use crate::store::{ArtifactError, content_hash_str};

    #[test]
    fn verified_artifact_round_trips() {
        let artifact = fixture(
            Some("2026-06-22T00:00:00Z".to_owned()),
            Some("run-a".to_owned()),
        );
        let parsed = VerifiedArtifact::from_json_str(&artifact.to_json_string()).unwrap();

        assert_eq!(parsed, artifact);
        assert_eq!(artifact.diagnostics_hash(), parsed.diagnostics_hash());
    }

    #[test]
    fn local_fields_do_not_affect_hashes() {
        let first = fixture(Some("time-a".to_owned()), Some("run-a".to_owned()));
        let second = fixture(Some("time-b".to_owned()), Some("run-b".to_owned()));

        assert_eq!(first.interface_hash, second.interface_hash);
        assert_eq!(first.implementation_hash, second.implementation_hash);
    }

    #[test]
    fn raw_ir_payloads_are_rejected() {
        let artifact = fixture(None, None);
        let json = artifact
            .to_json_string()
            .replace("\"module_path\"", "\"raw_ir\"");

        assert!(matches!(
            VerifiedArtifact::from_json_str(&json),
            Err(ArtifactError::RawIrPayload { .. })
        ));
        assert!(matches!(
            MetadataField::new("raw_ir_arena", "opaque"),
            Err(ArtifactError::RawIrPayload { .. })
        ));
    }

    #[test]
    fn projected_diagnostics_keep_stable_order() {
        let mut artifact = fixture(None, None);
        artifact.diagnostics.reverse();

        assert!(matches!(
            VerifiedArtifact::from_json_str(&artifact.to_json_string()),
            Err(ArtifactError::InvalidField {
                field: "diagnostics",
                ..
            })
        ));
    }

    #[test]
    fn tampered_implementation_hash_and_invalid_explanation_path_are_rejected() {
        let mut tampered = fixture(None, None);
        tampered.implementation_hash = content_hash_str("wrong implementation");
        assert!(matches!(
            VerifiedArtifact::from_json_str(&tampered.to_json_string()),
            Err(ArtifactError::HashMismatch { .. })
        ));

        let invalid_path = fixture(None, None)
            .to_json_string()
            .replace("explanations/alpha/W0001.json", "../W0001.json");
        assert!(matches!(
            VerifiedArtifact::from_json_str(&invalid_path),
            Err(ArtifactError::InvalidPath { .. })
        ));
    }

    #[test]
    fn interface_hash_includes_dependency_summary_hashes_and_source_path_is_portable() {
        let first = fixture(None, None);
        let mut second = fixture(None, None);
        second.provenance.dependency_artifact_hashes = vec![DependencyArtifactHash::new(
            "registration-summary",
            content_hash_str("changed registration summary"),
        )];
        second.interface_hash = second.compute_interface_hash();
        second.implementation_hash = second.compute_implementation_hash();

        assert_ne!(first.interface_hash, second.interface_hash);

        let invalid_source = first
            .to_json_string()
            .replace("src/alpha.miz", "../alpha.miz");
        assert!(matches!(
            VerifiedArtifact::from_json_str(&invalid_source),
            Err(ArtifactError::InvalidPath { .. })
        ));
    }

    #[test]
    fn nested_repeated_fields_must_arrive_in_canonical_order() {
        let mut metadata = fixture(None, None);
        metadata.expressions[0].metadata = vec![
            MetadataField::new("zeta", "z").unwrap(),
            MetadataField::new("alpha", "a").unwrap(),
        ];
        assert!(matches!(
            VerifiedArtifact::from_json_str(&metadata.to_json_string()),
            Err(ArtifactError::InvalidField {
                field: "metadata",
                ..
            })
        ));

        let mut provenance = fixture(None, None);
        provenance.provenance.dependency_artifact_hashes = vec![
            DependencyArtifactHash::new("zeta", content_hash_str("z")),
            DependencyArtifactHash::new("alpha", content_hash_str("a")),
        ];
        provenance.interface_hash = provenance.compute_interface_hash();
        provenance.implementation_hash = provenance.compute_implementation_hash();
        assert!(matches!(
            VerifiedArtifact::from_json_str(&provenance.to_json_string()),
            Err(ArtifactError::InvalidField {
                field: "dependency_artifact_hashes",
                ..
            })
        ));
    }

    fn fixture(verified_at: Option<String>, local_run_id: Option<String>) -> VerifiedArtifact {
        let witness = ProofWitnessRef::from_content(
            "wit-1",
            "vc-1",
            "proof-witnesses/alpha/vc-1.json",
            "{\"proof\":true}",
            "atp_certificate",
            "kernel_verified",
            "kernel-profile-v1",
            "mizar-proof",
            vec!["A".to_owned()],
        );
        VerifiedArtifact::new(
            "src/alpha.miz",
            content_hash_str("source"),
            verified_at,
            "alpha",
            vec![ExportedItem::new(
                "theorem",
                "pkg::alpha::Th1",
                "Th1",
                "public",
                "for x holds x = x",
                None,
            )],
            vec![ExpressionMetadata::new(
                "expr-1",
                10,
                "term",
                "set",
                vec![MetadataField::new("hover", "x: set").unwrap()],
            )],
            vec![ObligationMetadata::new(
                "vc-1",
                20,
                "theorem",
                "kernel_verified",
                Some("wit-1".to_owned()),
            )],
            vec![witness],
            vec![
                ProjectedDiagnostic::new(
                    30,
                    "W0001",
                    "warning",
                    "diagnostic",
                    Some(ArtifactRef::new(
                        "explanations/alpha/W0001.json",
                        content_hash_str("{\"explain\":true}"),
                        "diag-explain-v1",
                    )),
                ),
                ProjectedDiagnostic::new(31, "W0002", "warning", "second diagnostic", None),
            ],
            BuildProvenance::new(
                "mizar-evo 0.1",
                "2024",
                content_hash_str("lock"),
                content_hash_str("config"),
                vec![DependencyArtifactHash::new(
                    "dep/hidden.mizir.json",
                    content_hash_str("dep"),
                )],
                content_hash_str("cache-key"),
                local_run_id,
            ),
        )
    }
}
