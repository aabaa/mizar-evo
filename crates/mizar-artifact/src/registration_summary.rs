//! Published `RegistrationSummary` schema, canonical writer, and validating reader.
//!
//! The schema is specified in
//! [registration_summary.md](../../../../doc/design/mizar-artifact/en/registration_summary.md).

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
    str::FromStr,
};

use mizar_session::Hash;

use crate::{
    module_summary::{ModuleSummaryIdentity, SOURCE_HASH_CONSTRUCTION, SourceRangeSummary},
    store::{
        ARTIFACT_HASH_CONSTRUCTION, CanonicalHashDomain, CanonicalJson, CanonicalJsonError,
        HashClass, MinorVersionPolicy, SchemaVersion, SchemaVersionError, SchemaVersionSupport,
        canonical_json_bytes,
    },
};

/// Schema family used by all registration summary artifacts.
pub const REGISTRATION_SUMMARY_SCHEMA_FAMILY: &str = "mizar-artifact/registration-summary";

/// Dependency-facing published registration summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationSummary {
    /// Schema version read from or written to the artifact.
    pub schema_version: SchemaVersion,
    /// Stable package/module identity.
    pub module: ModuleSummaryIdentity,
    /// Exact source text hash for stale-artifact diagnostics.
    pub source_hash: Hash,
    /// Recomputed dependency-facing registration interface hash.
    pub registration_interface_hash: Hash,
    /// Activated public registrations visible to importers.
    pub activated_registrations: Vec<ActivatedRegistrationSummary>,
    /// Hash-addressed references to published resolution traces.
    pub trace_artifacts: Vec<RegistrationTraceArtifactRef>,
    /// Dependency registration summaries that affected this projection.
    pub dependency_registrations: Vec<DependencyRegistrationRef>,
}

/// One activated registration exported to importers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivatedRegistrationSummary {
    /// Stable registration origin id.
    pub origin_id: String,
    /// Source label or stable generated label.
    pub label: Option<String>,
    /// Registration kind.
    pub registration_kind: RegistrationKind,
    /// Export visibility. Task 7 publishes only public registrations.
    pub visibility: RegistrationVisibility,
    /// Exported namespace path.
    pub namespace_path: Vec<String>,
    /// Module that declared the registration.
    pub source_module: ModuleSummaryIdentity,
    /// Canonical checker trigger key.
    pub trigger_key: String,
    /// Normalized trigger pattern projection.
    pub normalized_pattern: RegistrationPatternSummary,
    /// Generated contribution projection.
    pub generated_contribution: RegistrationContributionSummary,
    /// Projected proof status. Task 7 publishes only accepted registrations.
    pub accepted_status: RegistrationAcceptedStatus,
    /// Verifier-policy fingerprint that made this registration visible.
    pub verifier_policy_fingerprint: ArtifactHashRef,
    /// Resolution trace ids required for replay or diagnostics.
    pub trace_ids: Vec<String>,
    /// Diagnostic/navigation source range.
    pub source_range: Option<SourceRangeSummary>,
}

/// Normalized registration trigger pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationPatternSummary {
    /// Producer-owned semantic pattern fingerprint.
    pub fingerprint: ArtifactHashRef,
    /// Referenced type head when applicable.
    pub type_head: Option<String>,
    /// Referenced attribute when applicable.
    pub attribute: Option<String>,
    /// Referenced functor when applicable.
    pub functor: Option<String>,
    /// Referenced term head when applicable.
    pub term_head: Option<String>,
    /// Pattern parameters in canonical producer order.
    pub parameters: Vec<String>,
    /// Guard fingerprints in canonical producer order.
    pub guards: Vec<ArtifactHashRef>,
}

/// Generated registration contribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationContributionSummary {
    /// Contribution kind.
    pub kind: RegistrationContributionKind,
    /// Stable human-readable summary of the generated contribution.
    pub summary: String,
    /// Producer-owned semantic contribution fingerprint.
    pub fingerprint: ArtifactHashRef,
}

/// Hash-addressed reference to a published resolution trace artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationTraceArtifactRef {
    /// Stable trace id used by registrations.
    pub trace_id: String,
    /// Trace kind owned by the resolution-trace schema.
    pub trace_kind: RegistrationTraceKind,
    /// Package-relative artifact path.
    pub artifact_path: String,
    /// Published trace file byte hash.
    pub artifact_hash: ArtifactHashRef,
    /// Semantic replay hash that participates in registration compatibility.
    pub trace_replay_hash: ArtifactHashRef,
    /// Optional diagnostic payload hash.
    pub diagnostic_hash: Option<ArtifactHashRef>,
    /// Activated registration origin ids that name this trace id.
    pub used_by_registration_origin_ids: Vec<String>,
}

/// Dependency registration summary hash that affected this summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyRegistrationRef {
    /// Dependency module identity.
    pub module: ModuleSummaryIdentity,
    /// Dependency registration interface hash.
    pub registration_interface_hash: Hash,
}

/// Artifact-framed hash classes used by producer-owned hash references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ArtifactHashClass {
    /// Dependency-facing exported interface hash.
    Interface,
    /// Full stable published projection hash.
    Implementation,
    /// Projected diagnostics and explanation-handle hash.
    Diagnostic,
    /// Published artifact equivalence hash.
    Artifact,
}

/// Producer-owned artifact-framed hash reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactHashRef {
    /// Hash class.
    pub class: ArtifactHashClass,
    /// Producer-owned schema family.
    pub schema_family: String,
    /// Producer-owned schema version.
    pub schema_version: SchemaVersion,
    /// Digest bytes.
    pub digest: Hash,
}

/// Registration kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RegistrationKind {
    /// Existential registration.
    Existential,
    /// Conditional cluster registration.
    Conditional,
    /// Functorial cluster registration.
    Functorial,
    /// Reduction registration.
    Reduction,
}

/// Export visibility represented in this dependency-facing artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RegistrationVisibility {
    /// Public registration.
    Public,
}

/// Projected registration proof status represented in this artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RegistrationAcceptedStatus {
    /// The registration obligations were accepted by the configured verifier policy.
    Accepted,
}

/// Generated contribution kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RegistrationContributionKind {
    /// Produced existence fact.
    ExistenceFact,
    /// Produced attribute fact.
    AttributeFact,
    /// Produced functorial result fact.
    FunctorialResult,
    /// Produced reduction rule.
    ReductionRule,
}

/// Resolution trace kind referenced by a registration summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum RegistrationTraceKind {
    /// Cluster expansion trace.
    Cluster,
    /// Reduction strategy trace.
    Reduction,
}

/// Additional validation requested by a caller while reading a summary.
#[derive(Debug, Clone, Copy, Default)]
pub struct RegistrationSummaryReadOptions<'a> {
    /// Artifact path to include in schema-version diagnostics.
    pub artifact_path: Option<&'a str>,
    /// Expected module identity from the manifest or import request.
    pub expected_module: Option<&'a ModuleSummaryIdentity>,
    /// Expected registration interface hash from the manifest or import request.
    pub expected_registration_interface_hash: Option<Hash>,
    /// Referenced trace artifacts supplied by the caller for hash validation.
    pub supplied_trace_artifacts: &'a [SuppliedTraceArtifactRef<'a>],
}

/// Hashes observed for a referenced trace artifact supplied by the caller.
#[derive(Debug, Clone, Copy)]
pub struct SuppliedTraceArtifactRef<'a> {
    /// Trace id.
    pub trace_id: &'a str,
    /// Published trace file byte hash observed by the caller.
    pub artifact_hash: &'a ArtifactHashRef,
    /// Semantic trace replay hash observed by the caller.
    pub trace_replay_hash: &'a ArtifactHashRef,
    /// Optional diagnostic payload hash observed by the caller.
    pub diagnostic_hash: Option<&'a ArtifactHashRef>,
}

/// Errors produced by the registration summary schema.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegistrationSummaryError {
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
    /// A collection is not in canonical order.
    UnsortedCollection { path: String },
    /// A collection contains a duplicate identity key.
    DuplicateEntry { path: String, key: String },
    /// The stored registration interface hash does not match the recomputed hash.
    RegistrationInterfaceHashMismatch { expected: String, actual: String },
    /// The caller-provided expected registration interface hash does not match.
    ExpectedRegistrationInterfaceHashMismatch { expected: String, actual: String },
    /// The caller-provided expected module identity does not match the summary.
    ModuleIdentityMismatch { expected: String, actual: String },
    /// Trace ids and used-by relationships are not bidirectionally consistent.
    TraceReferenceMismatch { path: String, reason: String },
    /// A caller-supplied trace artifact does not match the summary reference.
    SuppliedTraceArtifactMismatch {
        trace_id: String,
        field: &'static str,
        expected: String,
        actual: String,
    },
    /// A caller-supplied trace artifact does not correspond to this summary.
    UnknownSuppliedTraceArtifact { trace_id: String },
}

/// Returns the current registration summary schema version.
pub const fn current_schema_version() -> SchemaVersion {
    SchemaVersion::new(1, 0)
}

/// Returns the supported registration summary schema-version range.
pub fn schema_version_support() -> SchemaVersionSupport {
    SchemaVersionSupport::new(
        REGISTRATION_SUMMARY_SCHEMA_FAMILY,
        current_schema_version().major(),
        current_schema_version().minor(),
        MinorVersionPolicy::UpToSupported,
    )
}

/// Serializes a registration summary to canonical UTF-8 JSON bytes.
pub fn write_registration_summary(
    summary: &RegistrationSummary,
) -> Result<Vec<u8>, RegistrationSummaryError> {
    registration_summary_json(summary).map(|json| canonical_json_bytes(&json))
}

/// Builds the canonical JSON value for a registration summary.
pub fn registration_summary_json(
    summary: &RegistrationSummary,
) -> Result<CanonicalJson, RegistrationSummaryError> {
    validate_summary(summary)?;
    registration_summary_json_unchecked(summary)
}

/// Reads and validates a registration summary from a canonical JSON value.
pub fn read_registration_summary(
    value: &CanonicalJson,
    options: RegistrationSummaryReadOptions<'_>,
) -> Result<RegistrationSummary, RegistrationSummaryError> {
    let fields = expect_object(value, "$")?;
    let schema_version = read_schema_version(fields, options.artifact_path)?;
    reject_unknown_fields(
        fields,
        &[
            "schema_version",
            "module",
            "source_hash",
            "registration_interface_hash",
            "activated_registrations",
            "trace_artifacts",
            "dependency_registrations",
        ],
        "$",
    )?;

    let summary = RegistrationSummary {
        schema_version,
        module: read_identity(required_field(fields, "module", "$")?, "$.module")?,
        source_hash: read_source_hash(
            required_field(fields, "source_hash", "$")?,
            "$.source_hash",
        )?,
        registration_interface_hash: read_registration_interface_hash(
            required_field(fields, "registration_interface_hash", "$")?,
            "$.registration_interface_hash",
            schema_version,
        )?,
        activated_registrations: read_activated_registrations(
            required_field(fields, "activated_registrations", "$")?,
            "$.activated_registrations",
            schema_version,
        )?,
        trace_artifacts: read_trace_artifacts(
            required_field(fields, "trace_artifacts", "$")?,
            "$.trace_artifacts",
        )?,
        dependency_registrations: read_dependency_registrations(
            required_field(fields, "dependency_registrations", "$")?,
            "$.dependency_registrations",
            schema_version,
        )?,
    };

    validate_summary_shape(&summary)?;
    let recomputed = summary.compute_registration_interface_hash()?;
    if recomputed != summary.registration_interface_hash {
        return Err(
            RegistrationSummaryError::RegistrationInterfaceHashMismatch {
                expected: registration_interface_hash_string(schema_version, recomputed),
                actual: registration_interface_hash_string(
                    schema_version,
                    summary.registration_interface_hash,
                ),
            },
        );
    }

    if let Some(expected_module) = options.expected_module
        && expected_module != &summary.module
    {
        return Err(RegistrationSummaryError::ModuleIdentityMismatch {
            expected: identity_display(expected_module),
            actual: identity_display(&summary.module),
        });
    }

    if let Some(expected_hash) = options.expected_registration_interface_hash
        && expected_hash != summary.registration_interface_hash
    {
        return Err(
            RegistrationSummaryError::ExpectedRegistrationInterfaceHashMismatch {
                expected: registration_interface_hash_string(schema_version, expected_hash),
                actual: registration_interface_hash_string(
                    schema_version,
                    summary.registration_interface_hash,
                ),
            },
        );
    }

    validate_supplied_trace_artifacts(&summary, options.supplied_trace_artifacts)?;

    Ok(summary)
}

impl RegistrationSummary {
    /// Computes the dependency-facing registration interface hash for this summary.
    pub fn compute_registration_interface_hash(&self) -> Result<Hash, RegistrationSummaryError> {
        let projection = registration_interface_projection_json(self)?;
        let domain = CanonicalHashDomain::new(
            HashClass::Interface,
            REGISTRATION_SUMMARY_SCHEMA_FAMILY,
            self.schema_version,
        );
        Ok(domain.hash(&projection, &[]))
    }

    /// Recomputes and stores the dependency-facing registration interface hash.
    pub fn refresh_registration_interface_hash(
        &mut self,
    ) -> Result<Hash, RegistrationSummaryError> {
        let hash = self.compute_registration_interface_hash()?;
        self.registration_interface_hash = hash;
        Ok(hash)
    }
}

impl ArtifactHashRef {
    /// Builds a producer-owned artifact-framed hash reference.
    pub fn new(
        class: ArtifactHashClass,
        schema_family: impl Into<String>,
        schema_version: SchemaVersion,
        digest: Hash,
    ) -> Self {
        Self {
            class,
            schema_family: schema_family.into(),
            schema_version,
            digest,
        }
    }

    /// Returns the canonical artifact-framed hash string.
    pub fn to_artifact_hash_string(&self) -> String {
        artifact_hash_ref_string(self)
    }
}

impl ArtifactHashClass {
    fn as_str(self) -> &'static str {
        match self {
            Self::Interface => "interface",
            Self::Implementation => "implementation",
            Self::Diagnostic => "diagnostic",
            Self::Artifact => "artifact",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "interface" => Some(Self::Interface),
            "implementation" => Some(Self::Implementation),
            "diagnostic" => Some(Self::Diagnostic),
            "artifact" => Some(Self::Artifact),
            _ => None,
        }
    }
}

impl RegistrationKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Existential => "existential",
            Self::Conditional => "conditional",
            Self::Functorial => "functorial",
            Self::Reduction => "reduction",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "existential" => Some(Self::Existential),
            "conditional" => Some(Self::Conditional),
            "functorial" => Some(Self::Functorial),
            "reduction" => Some(Self::Reduction),
            _ => None,
        }
    }
}

impl RegistrationVisibility {
    fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "public" => Some(Self::Public),
            _ => None,
        }
    }
}

impl RegistrationAcceptedStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "accepted" => Some(Self::Accepted),
            _ => None,
        }
    }
}

impl RegistrationContributionKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::ExistenceFact => "existence_fact",
            Self::AttributeFact => "attribute_fact",
            Self::FunctorialResult => "functorial_result",
            Self::ReductionRule => "reduction_rule",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "existence_fact" => Some(Self::ExistenceFact),
            "attribute_fact" => Some(Self::AttributeFact),
            "functorial_result" => Some(Self::FunctorialResult),
            "reduction_rule" => Some(Self::ReductionRule),
            _ => None,
        }
    }
}

impl RegistrationTraceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Cluster => "cluster",
            Self::Reduction => "reduction",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "cluster" => Some(Self::Cluster),
            "reduction" => Some(Self::Reduction),
            _ => None,
        }
    }
}

impl fmt::Display for RegistrationSummaryError {
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
            Self::UnsortedCollection { path } => {
                write!(formatter, "collection `{path}` is not in canonical order")
            }
            Self::DuplicateEntry { path, key } => {
                write!(
                    formatter,
                    "collection `{path}` contains duplicate key `{key}`"
                )
            }
            Self::RegistrationInterfaceHashMismatch { expected, actual } => {
                write!(
                    formatter,
                    "registration summary registration_interface_hash mismatch: expected \
                     `{expected}`, got `{actual}`"
                )
            }
            Self::ExpectedRegistrationInterfaceHashMismatch { expected, actual } => {
                write!(
                    formatter,
                    "registration summary expected interface hash `{expected}` does not match \
                     `{actual}`"
                )
            }
            Self::ModuleIdentityMismatch { expected, actual } => {
                write!(
                    formatter,
                    "registration summary expected module `{expected}` does not match `{actual}`"
                )
            }
            Self::TraceReferenceMismatch { path, reason } => {
                write!(formatter, "invalid trace reference `{path}`: {reason}")
            }
            Self::SuppliedTraceArtifactMismatch {
                trace_id,
                field,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "supplied trace artifact `{trace_id}` field `{field}` expected `{expected}` \
                     but got `{actual}`"
                )
            }
            Self::UnknownSuppliedTraceArtifact { trace_id } => {
                write!(
                    formatter,
                    "supplied trace artifact `{trace_id}` is not referenced by the summary"
                )
            }
        }
    }
}

impl Error for RegistrationSummaryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalJson(error) => Some(error),
            Self::SchemaVersion(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CanonicalJsonError> for RegistrationSummaryError {
    fn from(error: CanonicalJsonError) -> Self {
        Self::CanonicalJson(error)
    }
}

impl From<SchemaVersionError> for RegistrationSummaryError {
    fn from(error: SchemaVersionError) -> Self {
        Self::SchemaVersion(error)
    }
}

fn validate_summary(summary: &RegistrationSummary) -> Result<(), RegistrationSummaryError> {
    validate_summary_shape(summary)?;
    let recomputed = summary.compute_registration_interface_hash()?;
    if recomputed != summary.registration_interface_hash {
        return Err(
            RegistrationSummaryError::RegistrationInterfaceHashMismatch {
                expected: registration_interface_hash_string(summary.schema_version, recomputed),
                actual: registration_interface_hash_string(
                    summary.schema_version,
                    summary.registration_interface_hash,
                ),
            },
        );
    }
    Ok(())
}

fn validate_summary_shape(summary: &RegistrationSummary) -> Result<(), RegistrationSummaryError> {
    schema_version_support().check(Some(&summary.schema_version.to_string()))?;
    validate_identity(&summary.module, "$.module")?;
    validate_activated_registrations(
        &summary.activated_registrations,
        "$.activated_registrations",
    )?;
    validate_trace_artifacts(&summary.trace_artifacts, "$.trace_artifacts")?;
    validate_dependency_registrations(
        &summary.dependency_registrations,
        "$.dependency_registrations",
    )?;
    validate_trace_reference_consistency(summary)?;
    Ok(())
}

fn registration_summary_json_unchecked(
    summary: &RegistrationSummary,
) -> Result<CanonicalJson, RegistrationSummaryError> {
    json_object([
        (
            "schema_version",
            CanonicalJson::string(summary.schema_version.to_string()),
        ),
        ("module", identity_json(&summary.module)?),
        (
            "source_hash",
            CanonicalJson::string(source_hash_string(summary.source_hash)),
        ),
        (
            "registration_interface_hash",
            CanonicalJson::string(registration_interface_hash_string(
                summary.schema_version,
                summary.registration_interface_hash,
            )),
        ),
        (
            "activated_registrations",
            CanonicalJson::array(
                sorted_activated_registrations(&summary.activated_registrations)
                    .into_iter()
                    .map(activated_registration_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "trace_artifacts",
            CanonicalJson::array(
                sorted_trace_artifacts(&summary.trace_artifacts)
                    .into_iter()
                    .map(trace_artifact_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "dependency_registrations",
            CanonicalJson::array(
                sorted_dependency_registrations(&summary.dependency_registrations)
                    .into_iter()
                    .map(|dependency| {
                        dependency_registration_json(dependency, summary.schema_version)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
    ])
}

fn registration_interface_projection_json(
    summary: &RegistrationSummary,
) -> Result<CanonicalJson, RegistrationSummaryError> {
    json_object([
        (
            "schema_version",
            CanonicalJson::string(summary.schema_version.to_string()),
        ),
        ("module", identity_json(&summary.module)?),
        (
            "activated_registrations",
            CanonicalJson::array(
                sorted_activated_registrations(&summary.activated_registrations)
                    .into_iter()
                    .map(activated_registration_interface_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "trace_artifacts",
            CanonicalJson::array(
                sorted_trace_artifacts(&summary.trace_artifacts)
                    .into_iter()
                    .map(trace_artifact_interface_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "dependency_registrations",
            CanonicalJson::array(
                sorted_dependency_registrations(&summary.dependency_registrations)
                    .into_iter()
                    .map(|dependency| {
                        dependency_registration_json(dependency, summary.schema_version)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
    ])
}

fn identity_json(
    identity: &ModuleSummaryIdentity,
) -> Result<CanonicalJson, RegistrationSummaryError> {
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

fn source_range_json(range: SourceRangeSummary) -> Result<CanonicalJson, RegistrationSummaryError> {
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
) -> Result<CanonicalJson, RegistrationSummaryError> {
    range.map_or(Ok(CanonicalJson::null()), source_range_json)
}

fn activated_registration_json(
    registration: &ActivatedRegistrationSummary,
) -> Result<CanonicalJson, RegistrationSummaryError> {
    json_object([
        ("origin_id", CanonicalJson::string(&registration.origin_id)),
        ("label", optional_string_json(registration.label.as_deref())),
        (
            "registration_kind",
            CanonicalJson::string(registration.registration_kind.as_str()),
        ),
        (
            "visibility",
            CanonicalJson::string(registration.visibility.as_str()),
        ),
        (
            "namespace_path",
            string_array_json(&registration.namespace_path),
        ),
        ("source_module", identity_json(&registration.source_module)?),
        (
            "trigger_key",
            CanonicalJson::string(&registration.trigger_key),
        ),
        (
            "normalized_pattern",
            registration_pattern_json(&registration.normalized_pattern)?,
        ),
        (
            "generated_contribution",
            registration_contribution_json(&registration.generated_contribution)?,
        ),
        (
            "accepted_status",
            CanonicalJson::string(registration.accepted_status.as_str()),
        ),
        (
            "verifier_policy_fingerprint",
            CanonicalJson::string(artifact_hash_ref_string(
                &registration.verifier_policy_fingerprint,
            )),
        ),
        (
            "trace_ids",
            CanonicalJson::array(
                sorted_strings(&registration.trace_ids)
                    .into_iter()
                    .map(CanonicalJson::string),
            ),
        ),
        (
            "source_range",
            optional_source_range_json(registration.source_range)?,
        ),
    ])
}

fn activated_registration_interface_json(
    registration: &ActivatedRegistrationSummary,
) -> Result<CanonicalJson, RegistrationSummaryError> {
    json_object([
        ("origin_id", CanonicalJson::string(&registration.origin_id)),
        ("label", optional_string_json(registration.label.as_deref())),
        (
            "registration_kind",
            CanonicalJson::string(registration.registration_kind.as_str()),
        ),
        (
            "visibility",
            CanonicalJson::string(registration.visibility.as_str()),
        ),
        (
            "namespace_path",
            string_array_json(&registration.namespace_path),
        ),
        ("source_module", identity_json(&registration.source_module)?),
        (
            "trigger_key",
            CanonicalJson::string(&registration.trigger_key),
        ),
        (
            "normalized_pattern",
            registration_pattern_json(&registration.normalized_pattern)?,
        ),
        (
            "generated_contribution",
            registration_contribution_json(&registration.generated_contribution)?,
        ),
        (
            "accepted_status",
            CanonicalJson::string(registration.accepted_status.as_str()),
        ),
        (
            "verifier_policy_fingerprint",
            CanonicalJson::string(artifact_hash_ref_string(
                &registration.verifier_policy_fingerprint,
            )),
        ),
        (
            "trace_ids",
            CanonicalJson::array(
                sorted_strings(&registration.trace_ids)
                    .into_iter()
                    .map(CanonicalJson::string),
            ),
        ),
    ])
}

fn registration_pattern_json(
    pattern: &RegistrationPatternSummary,
) -> Result<CanonicalJson, RegistrationSummaryError> {
    json_object([
        (
            "fingerprint",
            CanonicalJson::string(artifact_hash_ref_string(&pattern.fingerprint)),
        ),
        (
            "type_head",
            optional_string_json(pattern.type_head.as_deref()),
        ),
        (
            "attribute",
            optional_string_json(pattern.attribute.as_deref()),
        ),
        ("functor", optional_string_json(pattern.functor.as_deref())),
        (
            "term_head",
            optional_string_json(pattern.term_head.as_deref()),
        ),
        ("parameters", string_array_json(&pattern.parameters)),
        (
            "guards",
            CanonicalJson::array(
                pattern
                    .guards
                    .iter()
                    .map(|guard| CanonicalJson::string(artifact_hash_ref_string(guard))),
            ),
        ),
    ])
}

fn registration_contribution_json(
    contribution: &RegistrationContributionSummary,
) -> Result<CanonicalJson, RegistrationSummaryError> {
    json_object([
        ("kind", CanonicalJson::string(contribution.kind.as_str())),
        ("summary", CanonicalJson::string(&contribution.summary)),
        (
            "fingerprint",
            CanonicalJson::string(artifact_hash_ref_string(&contribution.fingerprint)),
        ),
    ])
}

fn trace_artifact_json(
    trace: &RegistrationTraceArtifactRef,
) -> Result<CanonicalJson, RegistrationSummaryError> {
    json_object([
        ("trace_id", CanonicalJson::string(&trace.trace_id)),
        (
            "trace_kind",
            CanonicalJson::string(trace.trace_kind.as_str()),
        ),
        ("artifact_path", CanonicalJson::string(&trace.artifact_path)),
        (
            "artifact_hash",
            CanonicalJson::string(artifact_hash_ref_string(&trace.artifact_hash)),
        ),
        (
            "trace_replay_hash",
            CanonicalJson::string(artifact_hash_ref_string(&trace.trace_replay_hash)),
        ),
        (
            "diagnostic_hash",
            optional_artifact_hash_json(trace.diagnostic_hash.as_ref()),
        ),
        (
            "used_by_registration_origin_ids",
            CanonicalJson::array(
                sorted_strings(&trace.used_by_registration_origin_ids)
                    .into_iter()
                    .map(CanonicalJson::string),
            ),
        ),
    ])
}

fn trace_artifact_interface_json(
    trace: &RegistrationTraceArtifactRef,
) -> Result<CanonicalJson, RegistrationSummaryError> {
    json_object([
        ("trace_id", CanonicalJson::string(&trace.trace_id)),
        (
            "trace_kind",
            CanonicalJson::string(trace.trace_kind.as_str()),
        ),
        (
            "trace_replay_hash",
            CanonicalJson::string(artifact_hash_ref_string(&trace.trace_replay_hash)),
        ),
    ])
}

fn dependency_registration_json(
    dependency: &DependencyRegistrationRef,
    schema_version: SchemaVersion,
) -> Result<CanonicalJson, RegistrationSummaryError> {
    json_object([
        ("module", identity_json(&dependency.module)?),
        (
            "registration_interface_hash",
            CanonicalJson::string(registration_interface_hash_string(
                schema_version,
                dependency.registration_interface_hash,
            )),
        ),
    ])
}

fn json_object(
    fields: impl IntoIterator<Item = (&'static str, CanonicalJson)>,
) -> Result<CanonicalJson, RegistrationSummaryError> {
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

fn string_array_json(values: &[String]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(CanonicalJson::string))
}

fn read_schema_version(
    fields: &BTreeMap<String, CanonicalJson>,
    artifact_path: Option<&str>,
) -> Result<SchemaVersion, RegistrationSummaryError> {
    let value = fields.get("schema_version");
    let version = match value {
        Some(CanonicalJson::String(version)) => Some(version.as_str()),
        Some(_) => {
            return Err(RegistrationSummaryError::UnexpectedType {
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
) -> Result<ModuleSummaryIdentity, RegistrationSummaryError> {
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
) -> Result<SourceRangeSummary, RegistrationSummaryError> {
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
) -> Result<Option<SourceRangeSummary>, RegistrationSummaryError> {
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

fn read_activated_registrations(
    value: &CanonicalJson,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<Vec<ActivatedRegistrationSummary>, RegistrationSummaryError> {
    let values = expect_array(value, path)?;
    let registrations = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            read_activated_registration(value, &array_path(path, index), schema_version)
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_activated_registrations(&registrations, path)?;
    ensure_sorted(&registrations, activated_registration_sort_key, path)?;
    Ok(registrations)
}

fn read_activated_registration(
    value: &CanonicalJson,
    path: &str,
    _schema_version: SchemaVersion,
) -> Result<ActivatedRegistrationSummary, RegistrationSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "origin_id",
            "label",
            "registration_kind",
            "visibility",
            "namespace_path",
            "source_module",
            "trigger_key",
            "normalized_pattern",
            "generated_contribution",
            "accepted_status",
            "verifier_policy_fingerprint",
            "trace_ids",
            "source_range",
        ],
        path,
    )?;
    let registration = ActivatedRegistrationSummary {
        origin_id: read_required_string(fields, "origin_id", path)?,
        label: read_optional_string(fields, "label", path)?,
        registration_kind: read_registration_kind(fields, "registration_kind", path)?,
        visibility: read_visibility(fields, "visibility", path)?,
        namespace_path: read_string_array(
            required_field(fields, "namespace_path", path)?,
            &field_path(path, "namespace_path"),
        )?,
        source_module: read_identity(
            required_field(fields, "source_module", path)?,
            &field_path(path, "source_module"),
        )?,
        trigger_key: read_required_string(fields, "trigger_key", path)?,
        normalized_pattern: read_registration_pattern(
            required_field(fields, "normalized_pattern", path)?,
            &field_path(path, "normalized_pattern"),
        )?,
        generated_contribution: read_registration_contribution(
            required_field(fields, "generated_contribution", path)?,
            &field_path(path, "generated_contribution"),
        )?,
        accepted_status: read_accepted_status(fields, "accepted_status", path)?,
        verifier_policy_fingerprint: read_required_artifact_hash_ref(
            fields,
            "verifier_policy_fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        trace_ids: read_string_array(
            required_field(fields, "trace_ids", path)?,
            &field_path(path, "trace_ids"),
        )?,
        source_range: read_optional_source_range(fields, "source_range", path)?,
    };
    validate_activated_registration(&registration, path)?;
    ensure_sorted(
        &registration.trace_ids,
        |value| value.clone(),
        &field_path(path, "trace_ids"),
    )?;
    Ok(registration)
}

fn read_registration_pattern(
    value: &CanonicalJson,
    path: &str,
) -> Result<RegistrationPatternSummary, RegistrationSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "fingerprint",
            "type_head",
            "attribute",
            "functor",
            "term_head",
            "parameters",
            "guards",
        ],
        path,
    )?;
    let pattern = RegistrationPatternSummary {
        fingerprint: read_required_artifact_hash_ref(
            fields,
            "fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        type_head: read_optional_string(fields, "type_head", path)?,
        attribute: read_optional_string(fields, "attribute", path)?,
        functor: read_optional_string(fields, "functor", path)?,
        term_head: read_optional_string(fields, "term_head", path)?,
        parameters: read_string_array(
            required_field(fields, "parameters", path)?,
            &field_path(path, "parameters"),
        )?,
        guards: read_artifact_hash_array(
            required_field(fields, "guards", path)?,
            &field_path(path, "guards"),
            ArtifactHashClass::Interface,
        )?,
    };
    validate_registration_pattern(&pattern, path)?;
    Ok(pattern)
}

fn read_registration_contribution(
    value: &CanonicalJson,
    path: &str,
) -> Result<RegistrationContributionSummary, RegistrationSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(fields, &["kind", "summary", "fingerprint"], path)?;
    let contribution = RegistrationContributionSummary {
        kind: read_contribution_kind(fields, "kind", path)?,
        summary: read_required_string(fields, "summary", path)?,
        fingerprint: read_required_artifact_hash_ref(
            fields,
            "fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
    };
    validate_registration_contribution(&contribution, path)?;
    Ok(contribution)
}

fn read_trace_artifacts(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<RegistrationTraceArtifactRef>, RegistrationSummaryError> {
    let values = expect_array(value, path)?;
    let traces = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_trace_artifact(value, &array_path(path, index)))
        .collect::<Result<Vec<_>, _>>()?;
    validate_trace_artifacts(&traces, path)?;
    ensure_sorted(&traces, trace_artifact_sort_key, path)?;
    Ok(traces)
}

fn read_trace_artifact(
    value: &CanonicalJson,
    path: &str,
) -> Result<RegistrationTraceArtifactRef, RegistrationSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "trace_id",
            "trace_kind",
            "artifact_path",
            "artifact_hash",
            "trace_replay_hash",
            "diagnostic_hash",
            "used_by_registration_origin_ids",
        ],
        path,
    )?;
    let trace = RegistrationTraceArtifactRef {
        trace_id: read_required_string(fields, "trace_id", path)?,
        trace_kind: read_trace_kind(fields, "trace_kind", path)?,
        artifact_path: read_required_string(fields, "artifact_path", path)?,
        artifact_hash: read_required_artifact_hash_ref(
            fields,
            "artifact_hash",
            path,
            ArtifactHashClass::Artifact,
        )?,
        trace_replay_hash: read_required_artifact_hash_ref(
            fields,
            "trace_replay_hash",
            path,
            ArtifactHashClass::Interface,
        )?,
        diagnostic_hash: read_optional_artifact_hash_ref(
            fields,
            "diagnostic_hash",
            path,
            ArtifactHashClass::Diagnostic,
        )?,
        used_by_registration_origin_ids: read_string_array(
            required_field(fields, "used_by_registration_origin_ids", path)?,
            &field_path(path, "used_by_registration_origin_ids"),
        )?,
    };
    validate_trace_artifact(&trace, path)?;
    ensure_sorted(
        &trace.used_by_registration_origin_ids,
        |value| value.clone(),
        &field_path(path, "used_by_registration_origin_ids"),
    )?;
    Ok(trace)
}

fn read_dependency_registrations(
    value: &CanonicalJson,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<Vec<DependencyRegistrationRef>, RegistrationSummaryError> {
    let values = expect_array(value, path)?;
    let dependencies = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            read_dependency_registration(value, &array_path(path, index), schema_version)
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_dependency_registrations(&dependencies, path)?;
    ensure_sorted(&dependencies, dependency_registration_sort_key, path)?;
    Ok(dependencies)
}

fn read_dependency_registration(
    value: &CanonicalJson,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<DependencyRegistrationRef, RegistrationSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(fields, &["module", "registration_interface_hash"], path)?;
    let dependency = DependencyRegistrationRef {
        module: read_identity(
            required_field(fields, "module", path)?,
            &field_path(path, "module"),
        )?,
        registration_interface_hash: read_registration_interface_hash(
            required_field(fields, "registration_interface_hash", path)?,
            &field_path(path, "registration_interface_hash"),
            schema_version,
        )?,
    };
    validate_dependency_registration(&dependency, path)?;
    Ok(dependency)
}

fn read_registration_kind(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<RegistrationKind, RegistrationSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path,
            expected: "a registration-kind string",
        });
    };
    RegistrationKind::from_str(value).ok_or_else(|| RegistrationSummaryError::InvalidField {
        path,
        reason: "unknown registration kind".to_owned(),
    })
}

fn read_visibility(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<RegistrationVisibility, RegistrationSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path,
            expected: "a visibility string",
        });
    };
    RegistrationVisibility::from_str(value).ok_or_else(|| RegistrationSummaryError::InvalidField {
        path,
        reason: "registration summary publishes only public registrations".to_owned(),
    })
}

fn read_accepted_status(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<RegistrationAcceptedStatus, RegistrationSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path,
            expected: "an accepted-status string",
        });
    };
    RegistrationAcceptedStatus::from_str(value).ok_or_else(|| {
        RegistrationSummaryError::InvalidField {
            path,
            reason: "registration summary publishes only accepted registrations".to_owned(),
        }
    })
}

fn read_contribution_kind(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<RegistrationContributionKind, RegistrationSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path,
            expected: "a contribution-kind string",
        });
    };
    RegistrationContributionKind::from_str(value).ok_or_else(|| {
        RegistrationSummaryError::InvalidField {
            path,
            reason: "unknown contribution kind".to_owned(),
        }
    })
}

fn read_trace_kind(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<RegistrationTraceKind, RegistrationSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path,
            expected: "a trace-kind string",
        });
    };
    RegistrationTraceKind::from_str(value).ok_or_else(|| RegistrationSummaryError::InvalidField {
        path,
        reason: "unknown trace kind".to_owned(),
    })
}

fn read_required_artifact_hash_ref(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<ArtifactHashRef, RegistrationSummaryError> {
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
) -> Result<Option<ArtifactHashRef>, RegistrationSummaryError> {
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

fn read_artifact_hash_array(
    value: &CanonicalJson,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<Vec<ArtifactHashRef>, RegistrationSummaryError> {
    let values = expect_array(value, path)?;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            read_artifact_hash_ref(value, &array_path(path, index), expected_class)
        })
        .collect()
}

fn read_artifact_hash_ref(
    value: &CanonicalJson,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<ArtifactHashRef, RegistrationSummaryError> {
    let CanonicalJson::String(value) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path: path.to_owned(),
            expected: "an artifact-framed hash string",
        });
    };
    let hash_ref = parse_artifact_hash_ref_string(value, path)?;
    if hash_ref.class != expected_class {
        return Err(RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: format!(
                "wrong artifact hash class: expected `{}`, got `{}`",
                expected_class.as_str(),
                hash_ref.class.as_str()
            ),
        });
    }
    Ok(hash_ref)
}

fn read_source_hash(value: &CanonicalJson, path: &str) -> Result<Hash, RegistrationSummaryError> {
    let CanonicalJson::String(value) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path: path.to_owned(),
            expected: "a source hash string",
        });
    };
    parse_source_hash_string(value, path)
}

fn read_registration_interface_hash(
    value: &CanonicalJson,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<Hash, RegistrationSummaryError> {
    let CanonicalJson::String(value) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path: path.to_owned(),
            expected: "a registration interface hash string",
        });
    };
    parse_registration_interface_hash_string(value, path, schema_version)
}

fn validate_identity(
    identity: &ModuleSummaryIdentity,
    path: &str,
) -> Result<(), RegistrationSummaryError> {
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

fn validate_source_range(
    range: SourceRangeSummary,
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    if range.start_byte > range.end_byte {
        return Err(RegistrationSummaryError::InvalidField {
            path: path.to_owned(),
            reason: "start_byte must not be greater than end_byte".to_owned(),
        });
    }
    integer_from_u64(range.start_byte, &field_path(path, "start_byte"))?;
    integer_from_u64(range.end_byte, &field_path(path, "end_byte"))?;
    Ok(())
}

fn validate_activated_registrations(
    registrations: &[ActivatedRegistrationSummary],
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    ensure_no_duplicate_keys(registrations, activated_registration_origin_key, path)?;
    for (index, registration) in registrations.iter().enumerate() {
        validate_activated_registration(registration, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_activated_registration(
    registration: &ActivatedRegistrationSummary,
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    validate_non_empty(&registration.origin_id, &field_path(path, "origin_id"))?;
    validate_optional_non_empty(registration.label.as_deref(), &field_path(path, "label"))?;
    validate_string_array(
        &registration.namespace_path,
        &field_path(path, "namespace_path"),
    )?;
    validate_identity(
        &registration.source_module,
        &field_path(path, "source_module"),
    )?;
    validate_non_empty(&registration.trigger_key, &field_path(path, "trigger_key"))?;
    validate_registration_pattern(
        &registration.normalized_pattern,
        &field_path(path, "normalized_pattern"),
    )?;
    validate_registration_contribution(
        &registration.generated_contribution,
        &field_path(path, "generated_contribution"),
    )?;
    validate_artifact_hash_ref(
        &registration.verifier_policy_fingerprint,
        &field_path(path, "verifier_policy_fingerprint"),
        ArtifactHashClass::Interface,
    )?;
    validate_string_array(&registration.trace_ids, &field_path(path, "trace_ids"))?;
    ensure_no_duplicate_keys(
        &registration.trace_ids,
        |value| value.clone(),
        &field_path(path, "trace_ids"),
    )?;
    if let Some(range) = registration.source_range {
        validate_source_range(range, &field_path(path, "source_range"))?;
    }
    Ok(())
}

fn validate_registration_pattern(
    pattern: &RegistrationPatternSummary,
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    validate_artifact_hash_ref(
        &pattern.fingerprint,
        &field_path(path, "fingerprint"),
        ArtifactHashClass::Interface,
    )?;
    validate_optional_non_empty(pattern.type_head.as_deref(), &field_path(path, "type_head"))?;
    validate_optional_non_empty(pattern.attribute.as_deref(), &field_path(path, "attribute"))?;
    validate_optional_non_empty(pattern.functor.as_deref(), &field_path(path, "functor"))?;
    validate_optional_non_empty(pattern.term_head.as_deref(), &field_path(path, "term_head"))?;
    validate_string_array(&pattern.parameters, &field_path(path, "parameters"))?;
    for (index, guard) in pattern.guards.iter().enumerate() {
        validate_artifact_hash_ref(
            guard,
            &array_path(&field_path(path, "guards"), index),
            ArtifactHashClass::Interface,
        )?;
    }
    Ok(())
}

fn validate_registration_contribution(
    contribution: &RegistrationContributionSummary,
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    validate_non_empty(&contribution.summary, &field_path(path, "summary"))?;
    validate_artifact_hash_ref(
        &contribution.fingerprint,
        &field_path(path, "fingerprint"),
        ArtifactHashClass::Interface,
    )
}

fn validate_trace_artifacts(
    traces: &[RegistrationTraceArtifactRef],
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    ensure_no_duplicate_keys(traces, trace_artifact_identity_key, path)?;
    for (index, trace) in traces.iter().enumerate() {
        validate_trace_artifact(trace, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_trace_artifact(
    trace: &RegistrationTraceArtifactRef,
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    validate_non_empty(&trace.trace_id, &field_path(path, "trace_id"))?;
    validate_non_empty(&trace.artifact_path, &field_path(path, "artifact_path"))?;
    validate_artifact_hash_ref(
        &trace.artifact_hash,
        &field_path(path, "artifact_hash"),
        ArtifactHashClass::Artifact,
    )?;
    validate_artifact_hash_ref(
        &trace.trace_replay_hash,
        &field_path(path, "trace_replay_hash"),
        ArtifactHashClass::Interface,
    )?;
    if let Some(diagnostic_hash) = &trace.diagnostic_hash {
        validate_artifact_hash_ref(
            diagnostic_hash,
            &field_path(path, "diagnostic_hash"),
            ArtifactHashClass::Diagnostic,
        )?;
    }
    validate_string_array(
        &trace.used_by_registration_origin_ids,
        &field_path(path, "used_by_registration_origin_ids"),
    )?;
    ensure_no_duplicate_keys(
        &trace.used_by_registration_origin_ids,
        |value| value.clone(),
        &field_path(path, "used_by_registration_origin_ids"),
    )
}

fn validate_dependency_registrations(
    dependencies: &[DependencyRegistrationRef],
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    ensure_no_duplicate_keys(dependencies, dependency_module_identity_key, path)?;
    for (index, dependency) in dependencies.iter().enumerate() {
        validate_dependency_registration(dependency, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_dependency_registration(
    dependency: &DependencyRegistrationRef,
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    validate_identity(&dependency.module, &field_path(path, "module"))
}

fn validate_artifact_hash_ref(
    hash_ref: &ArtifactHashRef,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<(), RegistrationSummaryError> {
    if hash_ref.class != expected_class {
        return Err(RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: format!(
                "wrong artifact hash class: expected `{}`, got `{}`",
                expected_class.as_str(),
                hash_ref.class.as_str()
            ),
        });
    }
    validate_schema_family(&hash_ref.schema_family, path)?;
    Ok(())
}

fn validate_schema_family(value: &str, path: &str) -> Result<(), RegistrationSummaryError> {
    if value.is_empty() {
        return Err(RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "schema family must not be empty".to_owned(),
        });
    }
    for segment in value.split('/') {
        if segment.is_empty() {
            return Err(RegistrationSummaryError::InvalidHash {
                path: path.to_owned(),
                reason: "schema family segments must not be empty".to_owned(),
            });
        }
        if !segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(RegistrationSummaryError::InvalidHash {
                path: path.to_owned(),
                reason: "schema family contains invalid characters".to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_trace_reference_consistency(
    summary: &RegistrationSummary,
) -> Result<(), RegistrationSummaryError> {
    let mut expected_used_by = BTreeMap::<String, BTreeSet<String>>::new();
    for registration in &summary.activated_registrations {
        for trace_id in &registration.trace_ids {
            expected_used_by
                .entry(trace_id.clone())
                .or_default()
                .insert(registration.origin_id.clone());
        }
    }

    let trace_ids = summary
        .trace_artifacts
        .iter()
        .map(|trace| trace.trace_id.clone())
        .collect::<BTreeSet<_>>();
    let expected_trace_ids = expected_used_by.keys().cloned().collect::<BTreeSet<_>>();
    if trace_ids != expected_trace_ids {
        return Err(RegistrationSummaryError::TraceReferenceMismatch {
            path: "$.trace_artifacts".to_owned(),
            reason: "trace ids must exactly match activated registration trace_ids".to_owned(),
        });
    }

    for trace in &summary.trace_artifacts {
        let actual = trace
            .used_by_registration_origin_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let expected = expected_used_by
            .get(&trace.trace_id)
            .cloned()
            .unwrap_or_default();
        if actual != expected {
            return Err(RegistrationSummaryError::TraceReferenceMismatch {
                path: format!(
                    "$.trace_artifacts[{}].used_by_registration_origin_ids",
                    trace.trace_id
                ),
                reason: "used_by set must exactly match registrations naming this trace id"
                    .to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_supplied_trace_artifacts(
    summary: &RegistrationSummary,
    supplied: &[SuppliedTraceArtifactRef<'_>],
) -> Result<(), RegistrationSummaryError> {
    let traces = summary
        .trace_artifacts
        .iter()
        .map(|trace| (trace.trace_id.as_str(), trace))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    for supplied_trace in supplied {
        if !seen.insert(supplied_trace.trace_id) {
            return Err(RegistrationSummaryError::DuplicateEntry {
                path: "supplied_trace_artifacts".to_owned(),
                key: supplied_trace.trace_id.to_owned(),
            });
        }
        let Some(trace) = traces.get(supplied_trace.trace_id) else {
            return Err(RegistrationSummaryError::UnknownSuppliedTraceArtifact {
                trace_id: supplied_trace.trace_id.to_owned(),
            });
        };
        validate_supplied_trace_field(
            supplied_trace.trace_id,
            "artifact_hash",
            &trace.artifact_hash,
            supplied_trace.artifact_hash,
        )?;
        validate_supplied_trace_field(
            supplied_trace.trace_id,
            "trace_replay_hash",
            &trace.trace_replay_hash,
            supplied_trace.trace_replay_hash,
        )?;
        match (&trace.diagnostic_hash, supplied_trace.diagnostic_hash) {
            (Some(expected), Some(actual)) => validate_supplied_trace_field(
                supplied_trace.trace_id,
                "diagnostic_hash",
                expected,
                actual,
            )?,
            (None, None) => {}
            (expected, actual) => {
                return Err(RegistrationSummaryError::SuppliedTraceArtifactMismatch {
                    trace_id: supplied_trace.trace_id.to_owned(),
                    field: "diagnostic_hash",
                    expected: expected
                        .as_ref()
                        .map_or_else(|| "null".to_owned(), artifact_hash_ref_string),
                    actual: actual.map_or_else(|| "null".to_owned(), artifact_hash_ref_string),
                });
            }
        }
    }
    Ok(())
}

fn validate_supplied_trace_field(
    trace_id: &str,
    field: &'static str,
    expected: &ArtifactHashRef,
    actual: &ArtifactHashRef,
) -> Result<(), RegistrationSummaryError> {
    if expected != actual {
        return Err(RegistrationSummaryError::SuppliedTraceArtifactMismatch {
            trace_id: trace_id.to_owned(),
            field,
            expected: artifact_hash_ref_string(expected),
            actual: artifact_hash_ref_string(actual),
        });
    }
    Ok(())
}

fn validate_non_empty(value: &str, path: &str) -> Result<(), RegistrationSummaryError> {
    if value.is_empty() {
        return Err(RegistrationSummaryError::InvalidField {
            path: path.to_owned(),
            reason: "must not be empty".to_owned(),
        });
    }
    Ok(())
}

fn validate_optional_non_empty(
    value: Option<&str>,
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    if matches!(value, Some("")) {
        return Err(RegistrationSummaryError::InvalidField {
            path: path.to_owned(),
            reason: "must be null or a non-empty string".to_owned(),
        });
    }
    Ok(())
}

fn validate_string_array(values: &[String], path: &str) -> Result<(), RegistrationSummaryError> {
    for (index, value) in values.iter().enumerate() {
        validate_non_empty(value, &array_path(path, index))?;
    }
    Ok(())
}

fn sorted_activated_registrations(
    registrations: &[ActivatedRegistrationSummary],
) -> Vec<&ActivatedRegistrationSummary> {
    let mut registrations = registrations.iter().collect::<Vec<_>>();
    registrations.sort_by_key(|registration| activated_registration_sort_key(registration));
    registrations
}

fn sorted_trace_artifacts(
    traces: &[RegistrationTraceArtifactRef],
) -> Vec<&RegistrationTraceArtifactRef> {
    let mut traces = traces.iter().collect::<Vec<_>>();
    traces.sort_by_key(|trace| trace_artifact_sort_key(trace));
    traces
}

fn sorted_dependency_registrations(
    dependencies: &[DependencyRegistrationRef],
) -> Vec<&DependencyRegistrationRef> {
    let mut dependencies = dependencies.iter().collect::<Vec<_>>();
    dependencies.sort_by_key(|dependency| dependency_registration_sort_key(dependency));
    dependencies
}

fn sorted_strings(values: &[String]) -> Vec<&String> {
    let mut values = values.iter().collect::<Vec<_>>();
    values.sort();
    values
}

type IdentityKey = (String, Option<String>, Option<String>, String, String);
type ActivatedRegistrationSortKey = (
    String,
    String,
    String,
    Option<String>,
    String,
    String,
    String,
);
type TraceArtifactSortKey = (String, String, String, String, String);
type DependencyRegistrationSortKey = (IdentityKey, String);

fn identity_key(identity: &ModuleSummaryIdentity) -> IdentityKey {
    (
        identity.package_id.clone(),
        identity.package_version.clone(),
        identity.lockfile_identity.clone(),
        identity.module_path.clone(),
        identity.language_edition.clone(),
    )
}

fn activated_registration_origin_key(registration: &ActivatedRegistrationSummary) -> String {
    registration.origin_id.clone()
}

fn activated_registration_sort_key(
    registration: &ActivatedRegistrationSummary,
) -> ActivatedRegistrationSortKey {
    (
        registration.registration_kind.as_str().to_owned(),
        registration.trigger_key.clone(),
        registration.origin_id.clone(),
        registration.label.clone(),
        artifact_hash_ref_string(&registration.normalized_pattern.fingerprint),
        artifact_hash_ref_string(&registration.generated_contribution.fingerprint),
        registration.accepted_status.as_str().to_owned(),
    )
}

fn trace_artifact_identity_key(trace: &RegistrationTraceArtifactRef) -> String {
    trace.trace_id.clone()
}

fn trace_artifact_sort_key(trace: &RegistrationTraceArtifactRef) -> TraceArtifactSortKey {
    (
        trace.trace_kind.as_str().to_owned(),
        trace.trace_id.clone(),
        trace.artifact_path.clone(),
        artifact_hash_ref_string(&trace.artifact_hash),
        artifact_hash_ref_string(&trace.trace_replay_hash),
    )
}

fn dependency_module_identity_key(dependency: &DependencyRegistrationRef) -> IdentityKey {
    identity_key(&dependency.module)
}

fn dependency_registration_sort_key(
    dependency: &DependencyRegistrationRef,
) -> DependencyRegistrationSortKey {
    (
        identity_key(&dependency.module),
        lower_hex_hash(dependency.registration_interface_hash),
    )
}

fn ensure_sorted<T, K, F>(
    items: &[T],
    mut key: F,
    path: &str,
) -> Result<(), RegistrationSummaryError>
where
    K: Ord,
    F: FnMut(&T) -> K,
{
    let mut previous = None;
    for item in items {
        let current = key(item);
        if previous
            .as_ref()
            .is_some_and(|previous| previous > &current)
        {
            return Err(RegistrationSummaryError::UnsortedCollection {
                path: path.to_owned(),
            });
        }
        previous = Some(current);
    }
    Ok(())
}

fn ensure_no_duplicate_keys<T, K, F>(
    items: &[T],
    mut key: F,
    path: &str,
) -> Result<(), RegistrationSummaryError>
where
    K: Ord + fmt::Debug,
    F: FnMut(&T) -> K,
{
    let mut seen = BTreeSet::new();
    for item in items {
        let key = key(item);
        if seen.contains(&key) {
            return Err(RegistrationSummaryError::DuplicateEntry {
                path: path.to_owned(),
                key: format!("{key:?}"),
            });
        }
        seen.insert(key);
    }
    Ok(())
}

fn expect_object<'a>(
    value: &'a CanonicalJson,
    path: &str,
) -> Result<&'a BTreeMap<String, CanonicalJson>, RegistrationSummaryError> {
    let CanonicalJson::Object(fields) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path: path.to_owned(),
            expected: "an object",
        });
    };
    Ok(fields)
}

fn expect_array<'a>(
    value: &'a CanonicalJson,
    path: &str,
) -> Result<&'a [CanonicalJson], RegistrationSummaryError> {
    let CanonicalJson::Array(values) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path: path.to_owned(),
            expected: "an array",
        });
    };
    Ok(values)
}

fn required_field<'a>(
    fields: &'a BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<&'a CanonicalJson, RegistrationSummaryError> {
    fields
        .get(field)
        .ok_or_else(|| RegistrationSummaryError::MissingField {
            path: field_path(path, field),
        })
}

fn reject_unknown_fields(
    fields: &BTreeMap<String, CanonicalJson>,
    allowed: &[&str],
    path: &str,
) -> Result<(), RegistrationSummaryError> {
    for field in fields.keys() {
        if !allowed.contains(&field.as_str()) {
            return Err(RegistrationSummaryError::UnknownField {
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
) -> Result<String, RegistrationSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
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
) -> Result<Option<String>, RegistrationSummaryError> {
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
        _ => Err(RegistrationSummaryError::UnexpectedType {
            path,
            expected: "a string or null",
        }),
    }
}

fn read_string_array(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<String>, RegistrationSummaryError> {
    let values = expect_array(value, path)?;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let path = array_path(path, index);
            let CanonicalJson::String(value) = value else {
                return Err(RegistrationSummaryError::UnexpectedType {
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
) -> Result<u64, RegistrationSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::Integer(value) = value else {
        return Err(RegistrationSummaryError::UnexpectedType {
            path,
            expected: "a non-negative integer",
        });
    };
    u64::try_from(*value).map_err(|_| RegistrationSummaryError::InvalidField {
        path,
        reason: "must be non-negative".to_owned(),
    })
}

fn source_hash_string(hash: Hash) -> String {
    format!("{}:{}", SOURCE_HASH_CONSTRUCTION, lower_hex_hash(hash))
}

fn registration_interface_hash_string(schema_version: SchemaVersion, hash: Hash) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        ARTIFACT_HASH_CONSTRUCTION,
        ArtifactHashClass::Interface.as_str(),
        REGISTRATION_SUMMARY_SCHEMA_FAMILY,
        schema_version,
        lower_hex_hash(hash)
    )
}

fn artifact_hash_ref_string(hash_ref: &ArtifactHashRef) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        ARTIFACT_HASH_CONSTRUCTION,
        hash_ref.class.as_str(),
        hash_ref.schema_family,
        hash_ref.schema_version,
        lower_hex_hash(hash_ref.digest)
    )
}

fn parse_source_hash_string(value: &str, path: &str) -> Result<Hash, RegistrationSummaryError> {
    let Some(hex) = value
        .strip_prefix(SOURCE_HASH_CONSTRUCTION)
        .and_then(|rest| rest.strip_prefix(':'))
    else {
        return Err(RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong source hash construction label".to_owned(),
        });
    };
    parse_lower_hex_hash(hex, path)
}

fn parse_registration_interface_hash_string(
    value: &str,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<Hash, RegistrationSummaryError> {
    let hash_ref = parse_artifact_hash_ref_string(value, path)?;
    if hash_ref.class != ArtifactHashClass::Interface {
        return Err(RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong artifact hash class".to_owned(),
        });
    }
    if hash_ref.schema_family != REGISTRATION_SUMMARY_SCHEMA_FAMILY {
        return Err(RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong schema family".to_owned(),
        });
    }
    if hash_ref.schema_version != schema_version {
        return Err(RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong schema version".to_owned(),
        });
    }
    Ok(hash_ref.digest)
}

fn parse_artifact_hash_ref_string(
    value: &str,
    path: &str,
) -> Result<ArtifactHashRef, RegistrationSummaryError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 5 {
        return Err(RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "expected construction:class:schema_family:schema_version:digest".to_owned(),
        });
    }
    if parts[0] != ARTIFACT_HASH_CONSTRUCTION {
        return Err(RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong artifact hash construction label".to_owned(),
        });
    }
    let class = ArtifactHashClass::from_str(parts[1]).ok_or_else(|| {
        RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "unknown artifact hash class".to_owned(),
        }
    })?;
    validate_schema_family(parts[2], path)?;
    let schema_version =
        SchemaVersion::from_str(parts[3]).map_err(|_| RegistrationSummaryError::InvalidHash {
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

fn parse_lower_hex_hash(hex: &str, path: &str) -> Result<Hash, RegistrationSummaryError> {
    if hex.len() != Hash::BYTE_LEN * 2 {
        return Err(RegistrationSummaryError::InvalidHash {
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

fn parse_lower_hex_nibble(byte: u8, path: &str) -> Result<u8, RegistrationSummaryError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(RegistrationSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "digest must use lowercase hexadecimal".to_owned(),
        }),
    }
}

fn lower_hex_hash(hash: Hash) -> String {
    let mut encoded = String::with_capacity(Hash::BYTE_LEN * 2);
    for byte in hash.as_bytes() {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn integer_from_u64(value: u64, path: &str) -> Result<i64, RegistrationSummaryError> {
    i64::try_from(value).map_err(|_| RegistrationSummaryError::InvalidField {
        path: path.to_owned(),
        reason: "value exceeds canonical JSON integer range".to_owned(),
    })
}

fn field_path(path: &str, field: &str) -> String {
    format!("{path}.{field}")
}

fn array_path(path: &str, index: usize) -> String {
    format!("{path}[{index}]")
}

fn identity_display(identity: &ModuleSummaryIdentity) -> String {
    format!(
        "{}:{}:{}",
        identity.package_id, identity.module_path, identity.language_edition
    )
}

#[cfg(test)]
mod tests;
