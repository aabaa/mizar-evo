//! Published `ProofWitnessRef` schema, canonical writer, and validating reader.
//!
//! The schema is specified in
//! [proof_witness.md](../../../../doc/design/mizar-artifact/en/proof_witness.md).

use std::{collections::BTreeMap, error::Error, fmt};

use crate::{
    registration_summary::{ArtifactHashClass, ArtifactHashRef},
    store::{
        CanonicalJson, CanonicalJsonError, MinorVersionPolicy, SchemaVersion, SchemaVersionError,
        SchemaVersionSupport, canonical_json_bytes,
    },
};

/// Schema family used by proof witness reference artifacts.
pub const PROOF_WITNESS_REF_SCHEMA_FAMILY: &str = "mizar-artifact/proof-witness-ref";

/// Stable reference to an external proof witness payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofWitnessRef {
    /// Schema version read from or written to the artifact.
    pub schema_version: SchemaVersion,
    /// Stable proof obligation id.
    pub obligation_id: String,
    /// Producer-owned obligation fingerprint.
    pub obligation_fingerprint: ArtifactHashRef,
    /// Accepted proof status projected by proof/kernel phases.
    pub proof_status: ProofStatus,
    /// Accepted evidence class.
    pub evidence_kind: EvidenceKind,
    /// Package-artifact-root-relative witness path.
    pub witness_path: String,
    /// Published witness payload byte hash.
    pub witness_artifact_hash: ArtifactHashRef,
    /// Kernel acceptance metadata projected into artifact readers.
    pub kernel_acceptance: KernelAcceptanceMetadata,
}

/// Kernel acceptance metadata for a proof witness reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelAcceptanceMetadata {
    /// Kernel profile fingerprint used by the accepting run.
    pub kernel_profile_fingerprint: ArtifactHashRef,
    /// Verifier policy fingerprint used by the accepting run.
    pub verifier_policy_fingerprint: ArtifactHashRef,
    /// Checker schema version visible to the kernel/proof producer.
    pub checker_schema_version: SchemaVersion,
    /// Producer-owned certificate format when one exists.
    pub certificate_format: Option<String>,
    /// Producer-owned accepted result hash.
    pub accepted_result_hash: ArtifactHashRef,
    /// Optional diagnostic hash for used axiom citations.
    pub used_axioms_hash: Option<ArtifactHashRef>,
}

/// Accepted proof status represented in a proof witness reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ProofStatus {
    /// An ATP certificate was accepted by the minimum kernel.
    KernelVerified,
    /// A built-in certificate or allowed primitive discharged the obligation.
    DischargedBuiltin,
}

/// Accepted evidence class represented in a proof witness reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum EvidenceKind {
    /// ATP certificate payload.
    AtpCertificate,
    /// Built-in certificate payload.
    BuiltinCertificate,
    /// Allowed kernel primitive without a certificate payload format.
    KernelPrimitive,
}

/// Additional validation requested by a caller while reading a witness reference.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProofWitnessReadOptions<'a> {
    /// Artifact path to include in schema-version diagnostics.
    pub artifact_path: Option<&'a str>,
    /// Expected witness artifact hash from a manifest or observed payload.
    pub expected_witness_artifact_hash: Option<&'a ArtifactHashRef>,
}

/// Errors produced by the proof witness reference schema.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofWitnessError {
    /// Canonical JSON object construction failed.
    CanonicalJson(CanonicalJsonError),
    /// Schema-version compatibility failed.
    SchemaVersion(SchemaVersionError),
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
    /// The status/evidence pair is unsupported.
    InvalidStatusEvidence {
        proof_status: &'static str,
        evidence_kind: &'static str,
    },
    /// The caller-provided witness artifact hash does not match the reference.
    WitnessArtifactHashMismatch { expected: String, actual: String },
}

/// Returns the current proof witness reference schema version.
pub const fn current_schema_version() -> SchemaVersion {
    SchemaVersion::new(1, 0)
}

/// Returns the supported proof witness reference schema-version range.
pub fn schema_version_support() -> SchemaVersionSupport {
    SchemaVersionSupport::new(
        PROOF_WITNESS_REF_SCHEMA_FAMILY,
        current_schema_version().major(),
        current_schema_version().minor(),
        MinorVersionPolicy::UpToSupported,
    )
}

/// Serializes a proof witness reference to canonical UTF-8 JSON bytes.
pub fn write_proof_witness_ref(reference: &ProofWitnessRef) -> Result<Vec<u8>, ProofWitnessError> {
    proof_witness_ref_json(reference).map(|json| canonical_json_bytes(&json))
}

/// Builds the canonical JSON value for a proof witness reference.
pub fn proof_witness_ref_json(
    reference: &ProofWitnessRef,
) -> Result<CanonicalJson, ProofWitnessError> {
    validate_reference(reference)?;
    proof_witness_ref_json_unchecked(reference)
}

/// Reads and validates a proof witness reference from a canonical JSON value.
pub fn read_proof_witness_ref(
    value: &CanonicalJson,
    options: ProofWitnessReadOptions<'_>,
) -> Result<ProofWitnessRef, ProofWitnessError> {
    let fields = expect_object(value, "$")?;
    let schema_version = read_schema_version(fields, options.artifact_path)?;
    reject_unknown_fields(
        fields,
        &[
            "schema_version",
            "obligation_id",
            "obligation_fingerprint",
            "proof_status",
            "evidence_kind",
            "witness_path",
            "witness_artifact_hash",
            "kernel_acceptance",
        ],
        "$",
    )?;

    let reference = ProofWitnessRef {
        schema_version,
        obligation_id: read_required_string(fields, "obligation_id", "$")?,
        obligation_fingerprint: read_required_artifact_hash_ref(
            fields,
            "obligation_fingerprint",
            "$",
            ArtifactHashClass::Interface,
        )?,
        proof_status: read_proof_status(fields, "proof_status", "$")?,
        evidence_kind: read_evidence_kind(fields, "evidence_kind", "$")?,
        witness_path: read_required_string(fields, "witness_path", "$")?,
        witness_artifact_hash: read_required_artifact_hash_ref(
            fields,
            "witness_artifact_hash",
            "$",
            ArtifactHashClass::Artifact,
        )?,
        kernel_acceptance: read_kernel_acceptance(
            required_field(fields, "kernel_acceptance", "$")?,
            "$.kernel_acceptance",
        )?,
    };

    validate_reference(&reference)?;
    if let Some(expected_hash) = options.expected_witness_artifact_hash
        && expected_hash != &reference.witness_artifact_hash
    {
        return Err(ProofWitnessError::WitnessArtifactHashMismatch {
            expected: artifact_hash_ref_string(expected_hash),
            actual: artifact_hash_ref_string(&reference.witness_artifact_hash),
        });
    }

    Ok(reference)
}

impl ProofStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::KernelVerified => "kernel_verified",
            Self::DischargedBuiltin => "discharged_builtin",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "kernel_verified" => Some(Self::KernelVerified),
            "discharged_builtin" => Some(Self::DischargedBuiltin),
            _ => None,
        }
    }
}

impl EvidenceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::AtpCertificate => "atp_certificate",
            Self::BuiltinCertificate => "builtin_certificate",
            Self::KernelPrimitive => "kernel_primitive",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "atp_certificate" => Some(Self::AtpCertificate),
            "builtin_certificate" => Some(Self::BuiltinCertificate),
            "kernel_primitive" => Some(Self::KernelPrimitive),
            _ => None,
        }
    }
}

impl fmt::Display for ProofWitnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CanonicalJson(error) => write!(formatter, "{error}"),
            Self::SchemaVersion(error) => write!(formatter, "{error}"),
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
            Self::InvalidStatusEvidence {
                proof_status,
                evidence_kind,
            } => write!(
                formatter,
                "unsupported proof witness status/evidence combination `{proof_status}` / \
                 `{evidence_kind}`"
            ),
            Self::WitnessArtifactHashMismatch { expected, actual } => write!(
                formatter,
                "proof witness artifact hash mismatch: expected `{expected}`, got `{actual}`"
            ),
        }
    }
}

impl Error for ProofWitnessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalJson(error) => Some(error),
            Self::SchemaVersion(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CanonicalJsonError> for ProofWitnessError {
    fn from(error: CanonicalJsonError) -> Self {
        Self::CanonicalJson(error)
    }
}

impl From<SchemaVersionError> for ProofWitnessError {
    fn from(error: SchemaVersionError) -> Self {
        Self::SchemaVersion(error)
    }
}

fn validate_reference(reference: &ProofWitnessRef) -> Result<(), ProofWitnessError> {
    schema_version_support().check(Some(&reference.schema_version.to_string()))?;
    validate_non_empty(&reference.obligation_id, "$.obligation_id")?;
    validate_artifact_hash_ref(
        &reference.obligation_fingerprint,
        "$.obligation_fingerprint",
        ArtifactHashClass::Interface,
    )?;
    validate_artifact_hash_ref(
        &reference.witness_artifact_hash,
        "$.witness_artifact_hash",
        ArtifactHashClass::Artifact,
    )?;
    validate_witness_path(&reference.witness_path, "$.witness_path")?;
    validate_kernel_acceptance(&reference.kernel_acceptance, "$.kernel_acceptance")?;
    validate_status_evidence(reference.proof_status, reference.evidence_kind)?;
    validate_certificate_format(reference)?;
    Ok(())
}

fn validate_kernel_acceptance(
    metadata: &KernelAcceptanceMetadata,
    path: &str,
) -> Result<(), ProofWitnessError> {
    validate_artifact_hash_ref(
        &metadata.kernel_profile_fingerprint,
        &field_path(path, "kernel_profile_fingerprint"),
        ArtifactHashClass::Interface,
    )?;
    validate_artifact_hash_ref(
        &metadata.verifier_policy_fingerprint,
        &field_path(path, "verifier_policy_fingerprint"),
        ArtifactHashClass::Interface,
    )?;
    validate_optional_non_empty(
        metadata.certificate_format.as_deref(),
        &field_path(path, "certificate_format"),
    )?;
    validate_artifact_hash_ref(
        &metadata.accepted_result_hash,
        &field_path(path, "accepted_result_hash"),
        ArtifactHashClass::Interface,
    )?;
    if let Some(used_axioms_hash) = &metadata.used_axioms_hash {
        validate_artifact_hash_ref(
            used_axioms_hash,
            &field_path(path, "used_axioms_hash"),
            ArtifactHashClass::Diagnostic,
        )?;
    }
    Ok(())
}

fn validate_status_evidence(
    proof_status: ProofStatus,
    evidence_kind: EvidenceKind,
) -> Result<(), ProofWitnessError> {
    let valid = matches!(
        (proof_status, evidence_kind),
        (ProofStatus::KernelVerified, EvidenceKind::AtpCertificate)
            | (
                ProofStatus::DischargedBuiltin,
                EvidenceKind::BuiltinCertificate
            )
            | (
                ProofStatus::DischargedBuiltin,
                EvidenceKind::KernelPrimitive
            )
    );
    if !valid {
        return Err(ProofWitnessError::InvalidStatusEvidence {
            proof_status: proof_status.as_str(),
            evidence_kind: evidence_kind.as_str(),
        });
    }
    Ok(())
}

fn validate_certificate_format(reference: &ProofWitnessRef) -> Result<(), ProofWitnessError> {
    match (
        reference.evidence_kind,
        reference.kernel_acceptance.certificate_format.as_deref(),
    ) {
        (EvidenceKind::AtpCertificate | EvidenceKind::BuiltinCertificate, Some(_)) => Ok(()),
        (EvidenceKind::KernelPrimitive, None) => Ok(()),
        (EvidenceKind::AtpCertificate | EvidenceKind::BuiltinCertificate, None) => {
            Err(ProofWitnessError::InvalidField {
                path: "$.kernel_acceptance.certificate_format".to_owned(),
                reason: "certificate evidence requires a non-empty certificate format".to_owned(),
            })
        }
        (EvidenceKind::KernelPrimitive, Some(_)) => Err(ProofWitnessError::InvalidField {
            path: "$.kernel_acceptance.certificate_format".to_owned(),
            reason: "kernel primitive evidence must use null certificate format".to_owned(),
        }),
    }
}

fn validate_witness_path(value: &str, path: &str) -> Result<(), ProofWitnessError> {
    validate_non_empty(value, path)?;
    if value.starts_with('/') {
        return Err(ProofWitnessError::InvalidField {
            path: path.to_owned(),
            reason: "witness path must be relative".to_owned(),
        });
    }
    if value.contains('\\') {
        return Err(ProofWitnessError::InvalidField {
            path: path.to_owned(),
            reason: "witness path must use / separators".to_owned(),
        });
    }
    let Some(rest) = value.strip_prefix("proof-witnesses/") else {
        return Err(ProofWitnessError::InvalidField {
            path: path.to_owned(),
            reason: "witness path must start with proof-witnesses/".to_owned(),
        });
    };
    if rest.is_empty() {
        return Err(ProofWitnessError::InvalidField {
            path: path.to_owned(),
            reason: "witness path must contain a child segment".to_owned(),
        });
    }
    for segment in value.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(ProofWitnessError::InvalidField {
                path: path.to_owned(),
                reason: "witness path segments must not be empty, . or ..".to_owned(),
            });
        }
    }
    Ok(())
}

fn proof_witness_ref_json_unchecked(
    reference: &ProofWitnessRef,
) -> Result<CanonicalJson, ProofWitnessError> {
    json_object([
        (
            "schema_version",
            CanonicalJson::string(reference.schema_version.to_string()),
        ),
        (
            "obligation_id",
            CanonicalJson::string(&reference.obligation_id),
        ),
        (
            "obligation_fingerprint",
            CanonicalJson::string(artifact_hash_ref_string(&reference.obligation_fingerprint)),
        ),
        (
            "proof_status",
            CanonicalJson::string(reference.proof_status.as_str()),
        ),
        (
            "evidence_kind",
            CanonicalJson::string(reference.evidence_kind.as_str()),
        ),
        (
            "witness_path",
            CanonicalJson::string(&reference.witness_path),
        ),
        (
            "witness_artifact_hash",
            CanonicalJson::string(artifact_hash_ref_string(&reference.witness_artifact_hash)),
        ),
        (
            "kernel_acceptance",
            kernel_acceptance_json(&reference.kernel_acceptance)?,
        ),
    ])
}

fn kernel_acceptance_json(
    metadata: &KernelAcceptanceMetadata,
) -> Result<CanonicalJson, ProofWitnessError> {
    json_object([
        (
            "kernel_profile_fingerprint",
            CanonicalJson::string(artifact_hash_ref_string(
                &metadata.kernel_profile_fingerprint,
            )),
        ),
        (
            "verifier_policy_fingerprint",
            CanonicalJson::string(artifact_hash_ref_string(
                &metadata.verifier_policy_fingerprint,
            )),
        ),
        (
            "checker_schema_version",
            CanonicalJson::string(metadata.checker_schema_version.to_string()),
        ),
        (
            "certificate_format",
            optional_string_json(metadata.certificate_format.as_deref()),
        ),
        (
            "accepted_result_hash",
            CanonicalJson::string(artifact_hash_ref_string(&metadata.accepted_result_hash)),
        ),
        (
            "used_axioms_hash",
            optional_artifact_hash_json(metadata.used_axioms_hash.as_ref()),
        ),
    ])
}

fn json_object(
    fields: impl IntoIterator<Item = (&'static str, CanonicalJson)>,
) -> Result<CanonicalJson, ProofWitnessError> {
    CanonicalJson::object(fields).map_err(Into::into)
}

fn optional_string_json(value: Option<&str>) -> CanonicalJson {
    value.map_or_else(CanonicalJson::null, CanonicalJson::string)
}

fn optional_artifact_hash_json(value: Option<&ArtifactHashRef>) -> CanonicalJson {
    value.map_or_else(CanonicalJson::null, |hash_ref| {
        CanonicalJson::string(artifact_hash_ref_string(hash_ref))
    })
}

fn read_schema_version(
    fields: &BTreeMap<String, CanonicalJson>,
    artifact_path: Option<&str>,
) -> Result<SchemaVersion, ProofWitnessError> {
    let value = fields.get("schema_version");
    let version = match value {
        Some(CanonicalJson::String(version)) => Some(version.as_str()),
        Some(_) => {
            return Err(ProofWitnessError::UnexpectedType {
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

fn read_kernel_acceptance(
    value: &CanonicalJson,
    path: &str,
) -> Result<KernelAcceptanceMetadata, ProofWitnessError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "kernel_profile_fingerprint",
            "verifier_policy_fingerprint",
            "checker_schema_version",
            "certificate_format",
            "accepted_result_hash",
            "used_axioms_hash",
        ],
        path,
    )?;
    let metadata = KernelAcceptanceMetadata {
        kernel_profile_fingerprint: read_required_artifact_hash_ref(
            fields,
            "kernel_profile_fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        verifier_policy_fingerprint: read_required_artifact_hash_ref(
            fields,
            "verifier_policy_fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        checker_schema_version: read_required_schema_version(
            fields,
            "checker_schema_version",
            path,
        )?,
        certificate_format: read_optional_string(fields, "certificate_format", path)?,
        accepted_result_hash: read_required_artifact_hash_ref(
            fields,
            "accepted_result_hash",
            path,
            ArtifactHashClass::Interface,
        )?,
        used_axioms_hash: read_optional_artifact_hash_ref(
            fields,
            "used_axioms_hash",
            path,
            ArtifactHashClass::Diagnostic,
        )?,
    };
    validate_kernel_acceptance(&metadata, path)?;
    Ok(metadata)
}

fn read_proof_status(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<ProofStatus, ProofWitnessError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(ProofWitnessError::UnexpectedType {
            path,
            expected: "a proof-status string",
        });
    };
    ProofStatus::from_str(value).ok_or_else(|| ProofWitnessError::InvalidField {
        path,
        reason: "unknown proof status".to_owned(),
    })
}

fn read_evidence_kind(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<EvidenceKind, ProofWitnessError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(ProofWitnessError::UnexpectedType {
            path,
            expected: "an evidence-kind string",
        });
    };
    EvidenceKind::from_str(value).ok_or_else(|| ProofWitnessError::InvalidField {
        path,
        reason: "unknown evidence kind".to_owned(),
    })
}

fn read_required_artifact_hash_ref(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<ArtifactHashRef, ProofWitnessError> {
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
) -> Result<Option<ArtifactHashRef>, ProofWitnessError> {
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
) -> Result<ArtifactHashRef, ProofWitnessError> {
    let CanonicalJson::String(value) = value else {
        return Err(ProofWitnessError::UnexpectedType {
            path: path.to_owned(),
            expected: "an artifact-framed hash string",
        });
    };
    validate_non_empty(value, path)?;
    let hash_ref = parse_artifact_hash_ref_string(value, path)?;
    if hash_ref.class != expected_class {
        return Err(ProofWitnessError::InvalidHash {
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

fn read_required_schema_version(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<SchemaVersion, ProofWitnessError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(ProofWitnessError::UnexpectedType {
            path,
            expected: "a schema-version string",
        });
    };
    validate_non_empty(value, &path)?;
    value
        .parse::<SchemaVersion>()
        .map_err(|_| ProofWitnessError::InvalidField {
            path,
            reason: "malformed schema version".to_owned(),
        })
}

fn expect_object<'a>(
    value: &'a CanonicalJson,
    path: &str,
) -> Result<&'a BTreeMap<String, CanonicalJson>, ProofWitnessError> {
    let CanonicalJson::Object(fields) = value else {
        return Err(ProofWitnessError::UnexpectedType {
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
) -> Result<&'a CanonicalJson, ProofWitnessError> {
    fields
        .get(field)
        .ok_or_else(|| ProofWitnessError::MissingField {
            path: field_path(path, field),
        })
}

fn reject_unknown_fields(
    fields: &BTreeMap<String, CanonicalJson>,
    allowed: &[&str],
    path: &str,
) -> Result<(), ProofWitnessError> {
    for field in fields.keys() {
        if !allowed.contains(&field.as_str()) {
            return Err(ProofWitnessError::UnknownField {
                path: path.to_owned(),
                field: field.clone(),
            });
        }
    }
    Ok(())
}

fn read_required_string(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<String, ProofWitnessError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(ProofWitnessError::UnexpectedType {
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
) -> Result<Option<String>, ProofWitnessError> {
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
        _ => Err(ProofWitnessError::UnexpectedType {
            path,
            expected: "a string or null",
        }),
    }
}

fn validate_non_empty(value: &str, path: &str) -> Result<(), ProofWitnessError> {
    if value.is_empty() {
        return Err(ProofWitnessError::InvalidField {
            path: path.to_owned(),
            reason: "must not be empty".to_owned(),
        });
    }
    Ok(())
}

fn validate_optional_non_empty(value: Option<&str>, path: &str) -> Result<(), ProofWitnessError> {
    if matches!(value, Some("")) {
        return Err(ProofWitnessError::InvalidField {
            path: path.to_owned(),
            reason: "must be null or a non-empty string".to_owned(),
        });
    }
    Ok(())
}

fn validate_artifact_hash_ref(
    hash_ref: &ArtifactHashRef,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<(), ProofWitnessError> {
    if hash_ref.class != expected_class {
        return Err(ProofWitnessError::InvalidHash {
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

fn validate_schema_family(value: &str, path: &str) -> Result<(), ProofWitnessError> {
    if value.is_empty() {
        return Err(ProofWitnessError::InvalidHash {
            path: path.to_owned(),
            reason: "schema family must not be empty".to_owned(),
        });
    }
    for segment in value.split('/') {
        if segment.is_empty() {
            return Err(ProofWitnessError::InvalidHash {
                path: path.to_owned(),
                reason: "schema family segments must not be empty".to_owned(),
            });
        }
        if !segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(ProofWitnessError::InvalidHash {
                path: path.to_owned(),
                reason: "schema family contains invalid characters".to_owned(),
            });
        }
    }
    Ok(())
}

fn artifact_hash_ref_string(hash_ref: &ArtifactHashRef) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        crate::store::ARTIFACT_HASH_CONSTRUCTION,
        artifact_hash_class_string(hash_ref.class),
        hash_ref.schema_family,
        hash_ref.schema_version,
        lower_hex_hash(hash_ref.digest)
    )
}

fn parse_artifact_hash_ref_string(
    value: &str,
    path: &str,
) -> Result<ArtifactHashRef, ProofWitnessError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 5 {
        return Err(ProofWitnessError::InvalidHash {
            path: path.to_owned(),
            reason: "expected construction:class:schema_family:schema_version:digest".to_owned(),
        });
    }
    if parts[0] != crate::store::ARTIFACT_HASH_CONSTRUCTION {
        return Err(ProofWitnessError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong artifact hash construction label".to_owned(),
        });
    }
    let class =
        artifact_hash_class_from_str(parts[1]).ok_or_else(|| ProofWitnessError::InvalidHash {
            path: path.to_owned(),
            reason: "unknown artifact hash class".to_owned(),
        })?;
    validate_schema_family(parts[2], path)?;
    let schema_version =
        parts[3]
            .parse::<SchemaVersion>()
            .map_err(|_| ProofWitnessError::InvalidHash {
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

fn parse_lower_hex_hash(hex: &str, path: &str) -> Result<mizar_session::Hash, ProofWitnessError> {
    if hex.len() != mizar_session::Hash::BYTE_LEN * 2 {
        return Err(ProofWitnessError::InvalidHash {
            path: path.to_owned(),
            reason: "digest must be 64 lowercase hexadecimal characters".to_owned(),
        });
    }
    let mut bytes = [0; mizar_session::Hash::BYTE_LEN];
    for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
        let high = parse_lower_hex_nibble(pair[0], path)?;
        let low = parse_lower_hex_nibble(pair[1], path)?;
        bytes[index] = (high << 4) | low;
    }
    Ok(mizar_session::Hash::from_bytes(bytes))
}

fn parse_lower_hex_nibble(byte: u8, path: &str) -> Result<u8, ProofWitnessError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(ProofWitnessError::InvalidHash {
            path: path.to_owned(),
            reason: "digest must use lowercase hexadecimal".to_owned(),
        }),
    }
}

fn lower_hex_hash(hash: mizar_session::Hash) -> String {
    let mut encoded = String::with_capacity(mizar_session::Hash::BYTE_LEN * 2);
    for byte in hash.as_bytes() {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn field_path(path: &str, field: &str) -> String {
    format!("{path}.{field}")
}

#[cfg(test)]
mod tests {
    use super::{
        EvidenceKind, KernelAcceptanceMetadata, ProofStatus, ProofWitnessError,
        ProofWitnessReadOptions, ProofWitnessRef, current_schema_version, proof_witness_ref_json,
        read_proof_witness_ref, write_proof_witness_ref,
    };
    use crate::{
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{CanonicalJson, SchemaVersion, SchemaVersionError, canonical_json_string},
    };
    use mizar_session::Hash;

    #[test]
    fn proof_witness_ref_round_trips_through_canonical_json() {
        let reference = sample_reference();
        let json = proof_witness_ref_json(&reference).expect("canonical proof witness JSON");
        let bytes = write_proof_witness_ref(&reference).expect("canonical proof witness bytes");

        assert_eq!(bytes, canonical_json_string(&json).into_bytes());
        assert_eq!(
            read_proof_witness_ref(&json, ProofWitnessReadOptions::default())
                .expect("valid proof witness reference"),
            reference
        );
    }

    #[test]
    fn supplied_witness_artifact_hash_mismatch_is_rejected() {
        let reference = sample_reference();
        let json = proof_witness_ref_json(&reference).expect("canonical JSON");
        assert!(
            read_proof_witness_ref(
                &json,
                ProofWitnessReadOptions {
                    expected_witness_artifact_hash: Some(&reference.witness_artifact_hash),
                    ..ProofWitnessReadOptions::default()
                },
            )
            .is_ok()
        );

        let wrong_hash = hash_ref(
            ArtifactHashClass::Artifact,
            "mizar-artifact/proof-witness-payload",
            88,
        );
        assert!(matches!(
            read_proof_witness_ref(
                &json,
                ProofWitnessReadOptions {
                    expected_witness_artifact_hash: Some(&wrong_hash),
                    ..ProofWitnessReadOptions::default()
                },
            ),
            Err(ProofWitnessError::WitnessArtifactHashMismatch { .. })
        ));
    }

    #[test]
    fn incompatible_version_reads_fail_cleanly() {
        let reference = sample_reference();
        let mut json = proof_witness_ref_json(&reference).expect("canonical JSON");
        replace_object_field(&mut json, "schema_version", CanonicalJson::string("2.0"));

        assert!(matches!(
            read_proof_witness_ref(
                &json,
                ProofWitnessReadOptions {
                    artifact_path: Some("build/proof-witness-ref.json"),
                    ..ProofWitnessReadOptions::default()
                },
            ),
            Err(ProofWitnessError::SchemaVersion(
                SchemaVersionError::MajorMismatch { .. }
            ))
        ));

        let mut missing = proof_witness_ref_json(&reference).expect("canonical JSON");
        remove_object_field(&mut missing, "schema_version");
        assert!(matches!(
            read_proof_witness_ref(&missing, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::SchemaVersion(
                SchemaVersionError::Missing { .. }
            ))
        ));

        let mut malformed = proof_witness_ref_json(&reference).expect("canonical JSON");
        replace_object_field(
            &mut malformed,
            "schema_version",
            CanonicalJson::string("bad"),
        );
        assert!(matches!(
            read_proof_witness_ref(&malformed, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::SchemaVersion(
                SchemaVersionError::Malformed { .. }
            ))
        ));

        let mut newer_minor = proof_witness_ref_json(&reference).expect("canonical JSON");
        replace_object_field(
            &mut newer_minor,
            "schema_version",
            CanonicalJson::string("1.1"),
        );
        assert!(matches!(
            read_proof_witness_ref(&newer_minor, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::SchemaVersion(
                SchemaVersionError::MinorTooNew { .. }
            ))
        ));
    }

    #[test]
    fn reader_rejects_invalid_hash_domains_and_digests() {
        let reference = sample_reference();
        let json = proof_witness_ref_json(&reference).expect("canonical JSON");
        let digest = "11".repeat(Hash::BYTE_LEN);

        for bad_hash in [
            format!(
                "mizar-artifact/other-framed-hash/v1:interface:mizar-artifact/proof:1.0:{digest}"
            ),
            format!(
                "mizar-artifact/artifact-framed-hash-text/v1:artifact:mizar-artifact/proof:1.0:{digest}"
            ),
            format!(
                "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact//proof:1.0:{digest}"
            ),
            format!(
                "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/proof:bad:{digest}"
            ),
            format!(
                "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/proof:1.0:{}",
                "GG".repeat(Hash::BYTE_LEN)
            ),
        ] {
            let mut bad_json = json.clone();
            replace_object_field(
                &mut bad_json,
                "obligation_fingerprint",
                CanonicalJson::string(bad_hash),
            );

            assert!(matches!(
                read_proof_witness_ref(&bad_json, ProofWitnessReadOptions::default()),
                Err(ProofWitnessError::InvalidHash { path, .. })
                    if path == "$.obligation_fingerprint"
            ));
        }
    }

    #[test]
    fn reader_rejects_wrong_hash_classes_on_each_hash_field() {
        let reference = sample_reference();
        let json = proof_witness_ref_json(&reference).expect("canonical JSON");

        let mut wrong_witness_hash_class = json.clone();
        replace_object_field(
            &mut wrong_witness_hash_class,
            "witness_artifact_hash",
            hash_ref_json(
                ArtifactHashClass::Interface,
                "mizar-artifact/proof-witness-payload",
                91,
            ),
        );
        assert!(matches!(
            read_proof_witness_ref(
                &wrong_witness_hash_class,
                ProofWitnessReadOptions::default()
            ),
            Err(ProofWitnessError::InvalidHash { path, .. })
                if path == "$.witness_artifact_hash"
        ));

        let mut wrong_accepted_result_class = json.clone();
        replace_nested_object_field(
            &mut wrong_accepted_result_class,
            "kernel_acceptance",
            "accepted_result_hash",
            hash_ref_json(ArtifactHashClass::Artifact, "mizar-kernel/result", 92),
        );
        assert!(matches!(
            read_proof_witness_ref(
                &wrong_accepted_result_class,
                ProofWitnessReadOptions::default()
            ),
            Err(ProofWitnessError::InvalidHash { path, .. })
                if path == "$.kernel_acceptance.accepted_result_hash"
        ));

        let mut wrong_used_axioms_class = json;
        replace_nested_object_field(
            &mut wrong_used_axioms_class,
            "kernel_acceptance",
            "used_axioms_hash",
            hash_ref_json(ArtifactHashClass::Interface, "mizar-proof/used-axioms", 93),
        );
        assert!(matches!(
            read_proof_witness_ref(&wrong_used_axioms_class, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::InvalidHash { path, .. })
                if path == "$.kernel_acceptance.used_axioms_hash"
        ));
    }

    #[test]
    fn status_evidence_and_certificate_format_matrix_is_validated() {
        let mut valid_builtin = sample_reference();
        valid_builtin.proof_status = ProofStatus::DischargedBuiltin;
        valid_builtin.evidence_kind = EvidenceKind::BuiltinCertificate;
        valid_builtin.kernel_acceptance.certificate_format =
            Some("mizar-builtin-cert/v1".to_owned());
        valid_builtin.kernel_acceptance.accepted_result_hash =
            hash_ref(ArtifactHashClass::Interface, "mizar-kernel/result", 51);
        let valid_builtin_json =
            proof_witness_ref_json(&valid_builtin).expect("valid builtin witness ref");
        assert!(
            read_proof_witness_ref(&valid_builtin_json, ProofWitnessReadOptions::default()).is_ok()
        );

        let mut valid_primitive = sample_reference();
        valid_primitive.proof_status = ProofStatus::DischargedBuiltin;
        valid_primitive.evidence_kind = EvidenceKind::KernelPrimitive;
        valid_primitive.kernel_acceptance.certificate_format = None;
        valid_primitive.kernel_acceptance.accepted_result_hash =
            hash_ref(ArtifactHashClass::Interface, "mizar-kernel/result", 52);
        let valid_primitive_json =
            proof_witness_ref_json(&valid_primitive).expect("valid primitive witness ref");
        assert!(
            read_proof_witness_ref(&valid_primitive_json, ProofWitnessReadOptions::default())
                .is_ok()
        );

        let mut invalid_status = sample_reference();
        invalid_status.proof_status = ProofStatus::KernelVerified;
        invalid_status.evidence_kind = EvidenceKind::BuiltinCertificate;
        assert!(matches!(
            proof_witness_ref_json(&invalid_status),
            Err(ProofWitnessError::InvalidStatusEvidence { .. })
        ));
        let mut invalid_status_json =
            proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
        replace_object_field(
            &mut invalid_status_json,
            "evidence_kind",
            CanonicalJson::string("builtin_certificate"),
        );
        assert!(matches!(
            read_proof_witness_ref(&invalid_status_json, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::InvalidStatusEvidence { .. })
        ));

        let mut missing_certificate_format = sample_reference();
        missing_certificate_format
            .kernel_acceptance
            .certificate_format = None;
        assert!(matches!(
            proof_witness_ref_json(&missing_certificate_format),
            Err(ProofWitnessError::InvalidField { path, .. })
                if path == "$.kernel_acceptance.certificate_format"
        ));
        let mut missing_certificate_format_json =
            proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
        replace_nested_object_field(
            &mut missing_certificate_format_json,
            "kernel_acceptance",
            "certificate_format",
            CanonicalJson::null(),
        );
        assert!(matches!(
            read_proof_witness_ref(
                &missing_certificate_format_json,
                ProofWitnessReadOptions::default()
            ),
            Err(ProofWitnessError::InvalidField { path, .. })
                if path == "$.kernel_acceptance.certificate_format"
        ));

        let mut primitive_with_format = valid_primitive;
        primitive_with_format.kernel_acceptance.certificate_format =
            Some("kernel-primitive/v1".to_owned());
        assert!(matches!(
            proof_witness_ref_json(&primitive_with_format),
            Err(ProofWitnessError::InvalidField { path, .. })
                if path == "$.kernel_acceptance.certificate_format"
        ));
        let mut primitive_with_format_json = valid_primitive_json;
        replace_nested_object_field(
            &mut primitive_with_format_json,
            "kernel_acceptance",
            "certificate_format",
            CanonicalJson::string("kernel-primitive/v1"),
        );
        assert!(matches!(
            read_proof_witness_ref(&primitive_with_format_json, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::InvalidField { path, .. })
                if path == "$.kernel_acceptance.certificate_format"
        ));
    }

    #[test]
    fn reader_rejects_unsafe_witness_paths() {
        let json = proof_witness_ref_json(&sample_reference()).expect("canonical JSON");
        for path_value in [
            "",
            "/proof-witnesses/cert.json",
            "other/cert.json",
            "proof-witnesses/",
            "proof-witnesses//cert.json",
            "proof-witnesses/./cert.json",
            "proof-witnesses/../cert.json",
            "proof-witnesses/a/..\\escape.json",
        ] {
            let mut reference = sample_reference();
            reference.witness_path = path_value.to_owned();
            assert!(
                matches!(
                    proof_witness_ref_json(&reference),
                    Err(ProofWitnessError::InvalidField { path, .. }) if path == "$.witness_path"
                ),
                "{path_value}"
            );

            let mut bad_json = json.clone();
            replace_object_field(
                &mut bad_json,
                "witness_path",
                CanonicalJson::string(path_value),
            );
            assert!(
                matches!(
                    read_proof_witness_ref(&bad_json, ProofWitnessReadOptions::default()),
                    Err(ProofWitnessError::InvalidField { path, .. }) if path == "$.witness_path"
                ),
                "{path_value}"
            );
        }
    }

    #[test]
    fn reader_rejects_missing_unknown_and_empty_fields() {
        let reference = sample_reference();
        let json = proof_witness_ref_json(&reference).expect("canonical JSON");

        let mut missing = json.clone();
        remove_object_field(&mut missing, "witness_artifact_hash");
        assert!(matches!(
            read_proof_witness_ref(&missing, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::MissingField { path }) if path == "$.witness_artifact_hash"
        ));

        let mut unknown = json.clone();
        replace_object_field(&mut unknown, "cache_record", CanonicalJson::bool(true));
        assert!(matches!(
            read_proof_witness_ref(&unknown, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::UnknownField { path, field })
                if path == "$" && field == "cache_record"
        ));

        let mut missing_nested = json.clone();
        remove_nested_object_field(
            &mut missing_nested,
            "kernel_acceptance",
            "accepted_result_hash",
        );
        assert!(matches!(
            read_proof_witness_ref(&missing_nested, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::MissingField { path })
                if path == "$.kernel_acceptance.accepted_result_hash"
        ));

        let mut missing_certificate_format = json.clone();
        remove_nested_object_field(
            &mut missing_certificate_format,
            "kernel_acceptance",
            "certificate_format",
        );
        assert!(matches!(
            read_proof_witness_ref(
                &missing_certificate_format,
                ProofWitnessReadOptions::default()
            ),
            Err(ProofWitnessError::MissingField { path })
                if path == "$.kernel_acceptance.certificate_format"
        ));

        let mut missing_used_axioms_hash = json.clone();
        remove_nested_object_field(
            &mut missing_used_axioms_hash,
            "kernel_acceptance",
            "used_axioms_hash",
        );
        assert!(matches!(
            read_proof_witness_ref(&missing_used_axioms_hash, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::MissingField { path })
                if path == "$.kernel_acceptance.used_axioms_hash"
        ));

        let mut unknown_nested = json.clone();
        replace_nested_object_field(
            &mut unknown_nested,
            "kernel_acceptance",
            "proof_authority",
            CanonicalJson::bool(true),
        );
        assert!(matches!(
            read_proof_witness_ref(&unknown_nested, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::UnknownField { path, field })
                if path == "$.kernel_acceptance" && field == "proof_authority"
        ));

        let mut empty_id = json;
        replace_object_field(&mut empty_id, "obligation_id", CanonicalJson::string(""));
        assert!(matches!(
            read_proof_witness_ref(&empty_id, ProofWitnessReadOptions::default()),
            Err(ProofWitnessError::InvalidField { path, .. }) if path == "$.obligation_id"
        ));
    }

    fn sample_reference() -> ProofWitnessRef {
        ProofWitnessRef {
            schema_version: current_schema_version(),
            obligation_id: "obligation:hidden:1".to_owned(),
            obligation_fingerprint: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-proof/obligation",
                1,
            ),
            proof_status: ProofStatus::KernelVerified,
            evidence_kind: EvidenceKind::AtpCertificate,
            witness_path: "proof-witnesses/hidden/obligation-1.json".to_owned(),
            witness_artifact_hash: hash_ref(
                ArtifactHashClass::Artifact,
                "mizar-artifact/proof-witness-payload",
                2,
            ),
            kernel_acceptance: KernelAcceptanceMetadata {
                kernel_profile_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/profile",
                    3,
                ),
                verifier_policy_fingerprint: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-proof/verifier-policy",
                    4,
                ),
                checker_schema_version: SchemaVersion::new(1, 0),
                certificate_format: Some("tptp-tstp/v1".to_owned()),
                accepted_result_hash: hash_ref(
                    ArtifactHashClass::Interface,
                    "mizar-kernel/result",
                    5,
                ),
                used_axioms_hash: Some(hash_ref(
                    ArtifactHashClass::Diagnostic,
                    "mizar-proof/used-axioms",
                    6,
                )),
            },
        }
    }

    fn hash_ref(class: ArtifactHashClass, schema_family: &str, seed: u8) -> ArtifactHashRef {
        ArtifactHashRef::new(class, schema_family, SchemaVersion::new(1, 0), hash(seed))
    }

    fn hash_ref_json(class: ArtifactHashClass, schema_family: &str, seed: u8) -> CanonicalJson {
        CanonicalJson::string(hash_ref(class, schema_family, seed).to_artifact_hash_string())
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }

    fn replace_object_field(value: &mut CanonicalJson, field: &str, replacement: CanonicalJson) {
        let CanonicalJson::Object(fields) = value else {
            panic!("expected object");
        };
        fields.insert(field.to_owned(), replacement);
    }

    fn remove_object_field(value: &mut CanonicalJson, field: &str) {
        let CanonicalJson::Object(fields) = value else {
            panic!("expected object");
        };
        fields.remove(field).expect("object field");
    }

    fn replace_nested_object_field(
        value: &mut CanonicalJson,
        parent: &str,
        field: &str,
        replacement: CanonicalJson,
    ) {
        replace_object_field(object_field_mut(value, parent), field, replacement);
    }

    fn remove_nested_object_field(value: &mut CanonicalJson, parent: &str, field: &str) {
        remove_object_field(object_field_mut(value, parent), field);
    }

    fn object_field_mut<'a>(value: &'a mut CanonicalJson, field: &str) -> &'a mut CanonicalJson {
        let CanonicalJson::Object(fields) = value else {
            panic!("expected object");
        };
        fields.get_mut(field).expect("object field")
    }
}
