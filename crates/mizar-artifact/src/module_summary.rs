//! Published `ModuleSummary` schema, canonical writer, and validating reader.
//!
//! The schema is specified in
//! [module_summary.md](../../../../doc/design/mizar-artifact/en/module_summary.md).

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use mizar_session::Hash;

use crate::store::{
    ARTIFACT_HASH_CONSTRUCTION, CanonicalHashDomain, CanonicalJson, CanonicalJsonError, HashClass,
    MinorVersionPolicy, SchemaVersion, SchemaVersionError, SchemaVersionSupport,
    canonical_json_bytes,
};

/// Schema family used by all module summary artifacts.
pub const MODULE_SUMMARY_SCHEMA_FAMILY: &str = "mizar-artifact/module-summary";
/// Construction label used for source text hashes in this schema.
pub const SOURCE_HASH_CONSTRUCTION: &str = "mizar-session/hash-text/v1";

const INTERFACE_HASH_CLASS: &str = "interface";

/// Dependency-facing published module summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSummary {
    /// Schema version read from or written to the artifact.
    pub schema_version: SchemaVersion,
    /// Stable package/module identity.
    pub module: ModuleSummaryIdentity,
    /// Exact source text hash for stale-artifact diagnostics.
    pub source_hash: Hash,
    /// Recomputed dependency-facing interface hash.
    pub interface_hash: Hash,
    /// Exported symbol projection.
    pub exported_symbols: Vec<ExportedSymbolSummary>,
    /// Exported label projection.
    pub exported_labels: Vec<ExportedLabelSummary>,
    /// Exported lexical projection.
    pub lexical_summary: ModuleLexicalSummary,
    /// Exported forwarding relationships.
    pub reexports: Vec<ModuleReexportSummary>,
    /// Dependency interface hashes that affected this summary.
    pub dependency_interfaces: Vec<DependencyInterfaceRef>,
}

/// Stable package/module identity visible to downstream tools.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleSummaryIdentity {
    /// Stable package id.
    pub package_id: String,
    /// Package version when one is available.
    pub package_version: Option<String>,
    /// Lockfile identity when one is available.
    pub lockfile_identity: Option<String>,
    /// Canonical module path.
    pub module_path: String,
    /// Language edition used to interpret the module.
    pub language_edition: String,
}

/// Byte range used for diagnostics and navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceRangeSummary {
    /// Start byte offset.
    pub start_byte: u64,
    /// End byte offset.
    pub end_byte: u64,
}

/// Exported symbol projection consumed by importers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedSymbolSummary {
    /// Stable origin id for this exported surface element.
    pub origin_id: String,
    /// Fully qualified exported name.
    pub fully_qualified_name: String,
    /// Exported namespace path.
    pub namespace_path: Vec<String>,
    /// Exported visibility identifier.
    pub visibility: String,
    /// Declaration kind identifier.
    pub declaration_kind: String,
    /// Diagnostic/navigation source range.
    pub source_range: SourceRangeSummary,
    /// Rendered importer-visible signature or statement.
    pub rendered_signature: String,
    /// Per-element interface fingerprint.
    pub interface_fingerprint: Hash,
    /// Projected proof status when importer-visible.
    pub proof_status: Option<ProofStatusSummary>,
}

/// Stable projected proof status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ProofStatusSummary {
    /// The exported proof obligation is accepted.
    Accepted,
    /// The exported proof obligation is not accepted.
    NotAccepted,
    /// This exported surface element does not require proof acceptance.
    NotRequired,
}

/// Exported label projection consumed by importers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedLabelSummary {
    /// Stable label origin id.
    pub origin_id: String,
    /// Label text.
    pub label: String,
    /// Fully qualified owner item.
    pub owner_fully_qualified_name: String,
    /// Exported visibility identifier.
    pub visibility: String,
    /// Diagnostic/navigation source range.
    pub source_range: SourceRangeSummary,
    /// Label target kind identifier.
    pub target_kind: String,
}

/// Exported lexical projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleLexicalSummary {
    /// Producer-owned lexical schema version.
    pub schema_version: String,
    /// Optional dependency-facing lexical fingerprint.
    pub fingerprint: Option<Hash>,
    /// Exported lexical contributions.
    pub contributions: Vec<LexicalContributionSummary>,
}

/// One exported lexical contribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalContributionSummary {
    /// Contribution kind identifier.
    pub kind: String,
    /// Canonical lexical key.
    pub key: String,
    /// Canonical producer payload.
    pub payload: String,
}

/// Exported forwarding relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleReexportSummary {
    /// Re-export target module.
    pub target_module: ModuleSummaryIdentity,
    /// Re-export target item origin id for item-level re-exports.
    pub target_item_origin_id: Option<String>,
    /// Name exported through the forwarding relationship.
    pub exported_name: Option<String>,
    /// Stable provenance id for diagnostics.
    pub provenance_origin_id: Option<String>,
}

/// Dependency summary hash that affected this summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyInterfaceRef {
    /// Dependency module identity.
    pub module: ModuleSummaryIdentity,
    /// Dependency module interface hash.
    pub interface_hash: Hash,
}

/// Additional validation requested by a caller while reading a summary.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModuleSummaryReadOptions<'a> {
    /// Artifact path to include in schema-version diagnostics.
    pub artifact_path: Option<&'a str>,
    /// Expected module identity from the manifest or import request.
    pub expected_module: Option<&'a ModuleSummaryIdentity>,
    /// Expected interface hash from the manifest or import request.
    pub expected_interface_hash: Option<Hash>,
}

/// Errors produced by the module summary schema.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModuleSummaryError {
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
    /// The stored interface hash does not match the recomputed projection hash.
    InterfaceHashMismatch { expected: String, actual: String },
    /// The caller-provided expected interface hash does not match the summary.
    ExpectedInterfaceHashMismatch { expected: String, actual: String },
    /// The caller-provided expected module identity does not match the summary.
    ModuleIdentityMismatch { expected: String, actual: String },
}

/// Returns the current module summary schema version.
pub const fn current_schema_version() -> SchemaVersion {
    SchemaVersion::new(1, 0)
}

/// Returns the supported module summary schema-version range.
pub fn schema_version_support() -> SchemaVersionSupport {
    SchemaVersionSupport::new(
        MODULE_SUMMARY_SCHEMA_FAMILY,
        current_schema_version().major(),
        current_schema_version().minor(),
        MinorVersionPolicy::UpToSupported,
    )
}

/// Serializes a module summary to canonical UTF-8 JSON bytes.
pub fn write_module_summary(summary: &ModuleSummary) -> Result<Vec<u8>, ModuleSummaryError> {
    module_summary_json(summary).map(|json| canonical_json_bytes(&json))
}

/// Builds the canonical JSON value for a module summary.
pub fn module_summary_json(summary: &ModuleSummary) -> Result<CanonicalJson, ModuleSummaryError> {
    validate_summary(summary)?;
    module_summary_json_unchecked(summary)
}

/// Reads and validates a module summary from a canonical JSON value.
pub fn read_module_summary(
    value: &CanonicalJson,
    options: ModuleSummaryReadOptions<'_>,
) -> Result<ModuleSummary, ModuleSummaryError> {
    let fields = expect_object(value, "$")?;
    let schema_version = read_schema_version(fields, options.artifact_path)?;
    reject_unknown_fields(
        fields,
        &[
            "schema_version",
            "module",
            "source_hash",
            "interface_hash",
            "exported_symbols",
            "exported_labels",
            "lexical_summary",
            "reexports",
            "dependency_interfaces",
        ],
        "$",
    )?;

    let summary = ModuleSummary {
        schema_version,
        module: read_identity(required_field(fields, "module", "$")?, "$.module")?,
        source_hash: read_source_hash(
            required_field(fields, "source_hash", "$")?,
            "$.source_hash",
        )?,
        interface_hash: read_interface_hash(
            required_field(fields, "interface_hash", "$")?,
            "$.interface_hash",
            schema_version,
        )?,
        exported_symbols: read_exported_symbols(
            required_field(fields, "exported_symbols", "$")?,
            "$.exported_symbols",
            schema_version,
        )?,
        exported_labels: read_exported_labels(
            required_field(fields, "exported_labels", "$")?,
            "$.exported_labels",
        )?,
        lexical_summary: read_lexical_summary(
            required_field(fields, "lexical_summary", "$")?,
            "$.lexical_summary",
            schema_version,
        )?,
        reexports: read_reexports(required_field(fields, "reexports", "$")?, "$.reexports")?,
        dependency_interfaces: read_dependency_interfaces(
            required_field(fields, "dependency_interfaces", "$")?,
            "$.dependency_interfaces",
            schema_version,
        )?,
    };

    validate_summary_shape(&summary)?;
    let recomputed = summary.compute_interface_hash()?;
    if recomputed != summary.interface_hash {
        return Err(ModuleSummaryError::InterfaceHashMismatch {
            expected: interface_hash_string(schema_version, recomputed),
            actual: interface_hash_string(schema_version, summary.interface_hash),
        });
    }

    if let Some(expected_module) = options.expected_module
        && expected_module != &summary.module
    {
        return Err(ModuleSummaryError::ModuleIdentityMismatch {
            expected: identity_display(expected_module),
            actual: identity_display(&summary.module),
        });
    }

    if let Some(expected_hash) = options.expected_interface_hash
        && expected_hash != summary.interface_hash
    {
        return Err(ModuleSummaryError::ExpectedInterfaceHashMismatch {
            expected: interface_hash_string(schema_version, expected_hash),
            actual: interface_hash_string(schema_version, summary.interface_hash),
        });
    }

    Ok(summary)
}

impl ModuleSummary {
    /// Computes the dependency-facing interface hash for this summary.
    pub fn compute_interface_hash(&self) -> Result<Hash, ModuleSummaryError> {
        let projection = interface_projection_json(self)?;
        let domain = CanonicalHashDomain::new(
            HashClass::Interface,
            MODULE_SUMMARY_SCHEMA_FAMILY,
            self.schema_version,
        );
        Ok(domain.hash(&projection, &[]))
    }

    /// Recomputes and stores the dependency-facing interface hash.
    pub fn refresh_interface_hash(&mut self) -> Result<Hash, ModuleSummaryError> {
        let hash = self.compute_interface_hash()?;
        self.interface_hash = hash;
        Ok(hash)
    }
}

impl ProofStatusSummary {
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

impl fmt::Display for ModuleSummaryError {
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
            Self::InterfaceHashMismatch { expected, actual } => {
                write!(
                    formatter,
                    "module summary interface_hash mismatch: expected `{expected}`, got `{actual}`"
                )
            }
            Self::ExpectedInterfaceHashMismatch { expected, actual } => {
                write!(
                    formatter,
                    "module summary expected interface hash `{expected}` does not match `{actual}`"
                )
            }
            Self::ModuleIdentityMismatch { expected, actual } => {
                write!(
                    formatter,
                    "module summary expected module `{expected}` does not match `{actual}`"
                )
            }
        }
    }
}

impl Error for ModuleSummaryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalJson(error) => Some(error),
            Self::SchemaVersion(error) => Some(error),
            _ => None,
        }
    }
}

impl From<CanonicalJsonError> for ModuleSummaryError {
    fn from(error: CanonicalJsonError) -> Self {
        Self::CanonicalJson(error)
    }
}

impl From<SchemaVersionError> for ModuleSummaryError {
    fn from(error: SchemaVersionError) -> Self {
        Self::SchemaVersion(error)
    }
}

fn validate_summary(summary: &ModuleSummary) -> Result<(), ModuleSummaryError> {
    validate_summary_shape(summary)?;
    let recomputed = summary.compute_interface_hash()?;
    if recomputed != summary.interface_hash {
        return Err(ModuleSummaryError::InterfaceHashMismatch {
            expected: interface_hash_string(summary.schema_version, recomputed),
            actual: interface_hash_string(summary.schema_version, summary.interface_hash),
        });
    }
    Ok(())
}

fn validate_summary_shape(summary: &ModuleSummary) -> Result<(), ModuleSummaryError> {
    schema_version_support().check(Some(&summary.schema_version.to_string()))?;
    validate_identity(&summary.module, "$.module")?;
    validate_exported_symbols(&summary.exported_symbols, "$.exported_symbols")?;
    validate_exported_labels(&summary.exported_labels, "$.exported_labels")?;
    validate_lexical_summary(&summary.lexical_summary, "$.lexical_summary")?;
    validate_reexports(&summary.reexports, "$.reexports")?;
    validate_dependency_interfaces(&summary.dependency_interfaces, "$.dependency_interfaces")?;
    Ok(())
}

fn module_summary_json_unchecked(
    summary: &ModuleSummary,
) -> Result<CanonicalJson, ModuleSummaryError> {
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
            "interface_hash",
            CanonicalJson::string(interface_hash_string(
                summary.schema_version,
                summary.interface_hash,
            )),
        ),
        (
            "exported_symbols",
            CanonicalJson::array(
                sorted_exported_symbols(&summary.exported_symbols)
                    .into_iter()
                    .map(|symbol| exported_symbol_json(symbol, summary.schema_version))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "exported_labels",
            CanonicalJson::array(
                sorted_exported_labels(&summary.exported_labels)
                    .into_iter()
                    .map(exported_label_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "lexical_summary",
            lexical_summary_json(&summary.lexical_summary, summary.schema_version)?,
        ),
        (
            "reexports",
            CanonicalJson::array(
                sorted_reexports(&summary.reexports)
                    .into_iter()
                    .map(reexport_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "dependency_interfaces",
            CanonicalJson::array(
                sorted_dependency_interfaces(&summary.dependency_interfaces)
                    .into_iter()
                    .map(|dependency| dependency_interface_json(dependency, summary.schema_version))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
    ])
}

fn interface_projection_json(summary: &ModuleSummary) -> Result<CanonicalJson, ModuleSummaryError> {
    json_object([
        (
            "schema_version",
            CanonicalJson::string(summary.schema_version.to_string()),
        ),
        ("module", identity_json(&summary.module)?),
        (
            "exported_symbols",
            CanonicalJson::array(
                sorted_exported_symbols_for_interface(&summary.exported_symbols)
                    .into_iter()
                    .map(|symbol| exported_symbol_interface_json(symbol, summary.schema_version))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "exported_labels",
            CanonicalJson::array(
                sorted_exported_labels_for_interface(&summary.exported_labels)
                    .into_iter()
                    .map(exported_label_interface_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "lexical_summary",
            lexical_summary_json(&summary.lexical_summary, summary.schema_version)?,
        ),
        (
            "reexports",
            CanonicalJson::array(
                sorted_reexports(&summary.reexports)
                    .into_iter()
                    .map(reexport_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "dependency_interfaces",
            CanonicalJson::array(
                sorted_dependency_interfaces(&summary.dependency_interfaces)
                    .into_iter()
                    .map(|dependency| dependency_interface_json(dependency, summary.schema_version))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
    ])
}

fn identity_json(identity: &ModuleSummaryIdentity) -> Result<CanonicalJson, ModuleSummaryError> {
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

fn source_range_json(range: SourceRangeSummary) -> Result<CanonicalJson, ModuleSummaryError> {
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

fn exported_symbol_json(
    symbol: &ExportedSymbolSummary,
    schema_version: SchemaVersion,
) -> Result<CanonicalJson, ModuleSummaryError> {
    json_object([
        ("origin_id", CanonicalJson::string(&symbol.origin_id)),
        (
            "fully_qualified_name",
            CanonicalJson::string(&symbol.fully_qualified_name),
        ),
        ("namespace_path", string_array_json(&symbol.namespace_path)),
        ("visibility", CanonicalJson::string(&symbol.visibility)),
        (
            "declaration_kind",
            CanonicalJson::string(&symbol.declaration_kind),
        ),
        ("source_range", source_range_json(symbol.source_range)?),
        (
            "rendered_signature",
            CanonicalJson::string(&symbol.rendered_signature),
        ),
        (
            "interface_fingerprint",
            CanonicalJson::string(interface_hash_string(
                schema_version,
                symbol.interface_fingerprint,
            )),
        ),
        (
            "proof_status",
            optional_string_json(symbol.proof_status.map(ProofStatusSummary::as_str)),
        ),
    ])
}

fn exported_symbol_interface_json(
    symbol: &ExportedSymbolSummary,
    schema_version: SchemaVersion,
) -> Result<CanonicalJson, ModuleSummaryError> {
    json_object([
        ("origin_id", CanonicalJson::string(&symbol.origin_id)),
        (
            "fully_qualified_name",
            CanonicalJson::string(&symbol.fully_qualified_name),
        ),
        ("namespace_path", string_array_json(&symbol.namespace_path)),
        ("visibility", CanonicalJson::string(&symbol.visibility)),
        (
            "declaration_kind",
            CanonicalJson::string(&symbol.declaration_kind),
        ),
        (
            "rendered_signature",
            CanonicalJson::string(&symbol.rendered_signature),
        ),
        (
            "interface_fingerprint",
            CanonicalJson::string(interface_hash_string(
                schema_version,
                symbol.interface_fingerprint,
            )),
        ),
        (
            "proof_status",
            optional_string_json(symbol.proof_status.map(ProofStatusSummary::as_str)),
        ),
    ])
}

fn exported_label_json(label: &ExportedLabelSummary) -> Result<CanonicalJson, ModuleSummaryError> {
    json_object([
        ("origin_id", CanonicalJson::string(&label.origin_id)),
        ("label", CanonicalJson::string(&label.label)),
        (
            "owner_fully_qualified_name",
            CanonicalJson::string(&label.owner_fully_qualified_name),
        ),
        ("visibility", CanonicalJson::string(&label.visibility)),
        ("source_range", source_range_json(label.source_range)?),
        ("target_kind", CanonicalJson::string(&label.target_kind)),
    ])
}

fn exported_label_interface_json(
    label: &ExportedLabelSummary,
) -> Result<CanonicalJson, ModuleSummaryError> {
    json_object([
        ("origin_id", CanonicalJson::string(&label.origin_id)),
        ("label", CanonicalJson::string(&label.label)),
        (
            "owner_fully_qualified_name",
            CanonicalJson::string(&label.owner_fully_qualified_name),
        ),
        ("visibility", CanonicalJson::string(&label.visibility)),
        ("target_kind", CanonicalJson::string(&label.target_kind)),
    ])
}

fn lexical_summary_json(
    summary: &ModuleLexicalSummary,
    schema_version: SchemaVersion,
) -> Result<CanonicalJson, ModuleSummaryError> {
    json_object([
        (
            "schema_version",
            CanonicalJson::string(&summary.schema_version),
        ),
        (
            "fingerprint",
            optional_hash_json(summary.fingerprint, schema_version),
        ),
        (
            "contributions",
            CanonicalJson::array(
                sorted_lexical_contributions(&summary.contributions)
                    .into_iter()
                    .map(lexical_contribution_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
    ])
}

fn lexical_contribution_json(
    contribution: &LexicalContributionSummary,
) -> Result<CanonicalJson, ModuleSummaryError> {
    json_object([
        ("kind", CanonicalJson::string(&contribution.kind)),
        ("key", CanonicalJson::string(&contribution.key)),
        ("payload", CanonicalJson::string(&contribution.payload)),
    ])
}

fn reexport_json(reexport: &ModuleReexportSummary) -> Result<CanonicalJson, ModuleSummaryError> {
    json_object([
        ("target_module", identity_json(&reexport.target_module)?),
        (
            "target_item_origin_id",
            optional_string_json(reexport.target_item_origin_id.as_deref()),
        ),
        (
            "exported_name",
            optional_string_json(reexport.exported_name.as_deref()),
        ),
        (
            "provenance_origin_id",
            optional_string_json(reexport.provenance_origin_id.as_deref()),
        ),
    ])
}

fn dependency_interface_json(
    dependency: &DependencyInterfaceRef,
    schema_version: SchemaVersion,
) -> Result<CanonicalJson, ModuleSummaryError> {
    json_object([
        ("module", identity_json(&dependency.module)?),
        (
            "interface_hash",
            CanonicalJson::string(interface_hash_string(
                schema_version,
                dependency.interface_hash,
            )),
        ),
    ])
}

fn json_object(
    fields: impl IntoIterator<Item = (&'static str, CanonicalJson)>,
) -> Result<CanonicalJson, ModuleSummaryError> {
    CanonicalJson::object(fields).map_err(Into::into)
}

fn optional_string_json(value: Option<&str>) -> CanonicalJson {
    value.map_or_else(CanonicalJson::null, CanonicalJson::string)
}

fn optional_hash_json(value: Option<Hash>, schema_version: SchemaVersion) -> CanonicalJson {
    value.map_or_else(CanonicalJson::null, |hash| {
        CanonicalJson::string(interface_hash_string(schema_version, hash))
    })
}

fn string_array_json(values: &[String]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(CanonicalJson::string))
}

fn read_schema_version(
    fields: &BTreeMap<String, CanonicalJson>,
    artifact_path: Option<&str>,
) -> Result<SchemaVersion, ModuleSummaryError> {
    let value = fields.get("schema_version");
    let version = match value {
        Some(CanonicalJson::String(version)) => Some(version.as_str()),
        Some(_) => {
            return Err(ModuleSummaryError::UnexpectedType {
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
) -> Result<ModuleSummaryIdentity, ModuleSummaryError> {
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
) -> Result<SourceRangeSummary, ModuleSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(fields, &["start_byte", "end_byte"], path)?;
    let range = SourceRangeSummary {
        start_byte: read_non_negative_u64(fields, "start_byte", path)?,
        end_byte: read_non_negative_u64(fields, "end_byte", path)?,
    };
    validate_source_range(range, path)?;
    Ok(range)
}

fn read_exported_symbols(
    value: &CanonicalJson,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<Vec<ExportedSymbolSummary>, ModuleSummaryError> {
    let values = expect_array(value, path)?;
    let symbols = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_exported_symbol(value, &array_path(path, index), schema_version))
        .collect::<Result<Vec<_>, _>>()?;
    validate_exported_symbols(&symbols, path)?;
    ensure_sorted(&symbols, exported_symbol_sort_key, path)?;
    Ok(symbols)
}

fn read_exported_symbol(
    value: &CanonicalJson,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<ExportedSymbolSummary, ModuleSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "origin_id",
            "fully_qualified_name",
            "namespace_path",
            "visibility",
            "declaration_kind",
            "source_range",
            "rendered_signature",
            "interface_fingerprint",
            "proof_status",
        ],
        path,
    )?;
    let symbol = ExportedSymbolSummary {
        origin_id: read_required_string(fields, "origin_id", path)?,
        fully_qualified_name: read_required_string(fields, "fully_qualified_name", path)?,
        namespace_path: read_string_array(
            required_field(fields, "namespace_path", path)?,
            &field_path(path, "namespace_path"),
        )?,
        visibility: read_required_string(fields, "visibility", path)?,
        declaration_kind: read_required_string(fields, "declaration_kind", path)?,
        source_range: read_source_range(
            required_field(fields, "source_range", path)?,
            &field_path(path, "source_range"),
        )?,
        rendered_signature: read_required_string(fields, "rendered_signature", path)?,
        interface_fingerprint: read_interface_hash(
            required_field(fields, "interface_fingerprint", path)?,
            &field_path(path, "interface_fingerprint"),
            schema_version,
        )?,
        proof_status: read_optional_proof_status(fields, "proof_status", path)?,
    };
    validate_exported_symbol(&symbol, path)?;
    Ok(symbol)
}

fn read_exported_labels(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<ExportedLabelSummary>, ModuleSummaryError> {
    let values = expect_array(value, path)?;
    let labels = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_exported_label(value, &array_path(path, index)))
        .collect::<Result<Vec<_>, _>>()?;
    validate_exported_labels(&labels, path)?;
    ensure_sorted(&labels, exported_label_sort_key, path)?;
    Ok(labels)
}

fn read_exported_label(
    value: &CanonicalJson,
    path: &str,
) -> Result<ExportedLabelSummary, ModuleSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "origin_id",
            "label",
            "owner_fully_qualified_name",
            "visibility",
            "source_range",
            "target_kind",
        ],
        path,
    )?;
    let label = ExportedLabelSummary {
        origin_id: read_required_string(fields, "origin_id", path)?,
        label: read_required_string(fields, "label", path)?,
        owner_fully_qualified_name: read_required_string(
            fields,
            "owner_fully_qualified_name",
            path,
        )?,
        visibility: read_required_string(fields, "visibility", path)?,
        source_range: read_source_range(
            required_field(fields, "source_range", path)?,
            &field_path(path, "source_range"),
        )?,
        target_kind: read_required_string(fields, "target_kind", path)?,
    };
    validate_exported_label(&label, path)?;
    Ok(label)
}

fn read_lexical_summary(
    value: &CanonicalJson,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<ModuleLexicalSummary, ModuleSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &["schema_version", "fingerprint", "contributions"],
        path,
    )?;
    let summary = ModuleLexicalSummary {
        schema_version: read_required_string(fields, "schema_version", path)?,
        fingerprint: read_optional_interface_hash(fields, "fingerprint", path, schema_version)?,
        contributions: read_lexical_contributions(
            required_field(fields, "contributions", path)?,
            &field_path(path, "contributions"),
        )?,
    };
    validate_lexical_summary(&summary, path)?;
    Ok(summary)
}

fn read_lexical_contributions(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<LexicalContributionSummary>, ModuleSummaryError> {
    let values = expect_array(value, path)?;
    let contributions = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_lexical_contribution(value, &array_path(path, index)))
        .collect::<Result<Vec<_>, _>>()?;
    validate_lexical_contributions(&contributions, path)?;
    ensure_sorted(&contributions, lexical_contribution_sort_key, path)?;
    Ok(contributions)
}

fn read_lexical_contribution(
    value: &CanonicalJson,
    path: &str,
) -> Result<LexicalContributionSummary, ModuleSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(fields, &["kind", "key", "payload"], path)?;
    let contribution = LexicalContributionSummary {
        kind: read_required_string(fields, "kind", path)?,
        key: read_required_string(fields, "key", path)?,
        payload: read_required_string(fields, "payload", path)?,
    };
    validate_lexical_contribution(&contribution, path)?;
    Ok(contribution)
}

fn read_reexports(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<ModuleReexportSummary>, ModuleSummaryError> {
    let values = expect_array(value, path)?;
    let reexports = values
        .iter()
        .enumerate()
        .map(|(index, value)| read_reexport(value, &array_path(path, index)))
        .collect::<Result<Vec<_>, _>>()?;
    validate_reexports(&reexports, path)?;
    ensure_sorted(&reexports, reexport_sort_key, path)?;
    Ok(reexports)
}

fn read_reexport(
    value: &CanonicalJson,
    path: &str,
) -> Result<ModuleReexportSummary, ModuleSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "target_module",
            "target_item_origin_id",
            "exported_name",
            "provenance_origin_id",
        ],
        path,
    )?;
    let reexport = ModuleReexportSummary {
        target_module: read_identity(
            required_field(fields, "target_module", path)?,
            &field_path(path, "target_module"),
        )?,
        target_item_origin_id: read_optional_string(fields, "target_item_origin_id", path)?,
        exported_name: read_optional_string(fields, "exported_name", path)?,
        provenance_origin_id: read_optional_string(fields, "provenance_origin_id", path)?,
    };
    validate_reexport(&reexport, path)?;
    Ok(reexport)
}

fn read_dependency_interfaces(
    value: &CanonicalJson,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<Vec<DependencyInterfaceRef>, ModuleSummaryError> {
    let values = expect_array(value, path)?;
    let dependencies = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            read_dependency_interface(value, &array_path(path, index), schema_version)
        })
        .collect::<Result<Vec<_>, _>>()?;
    validate_dependency_interfaces(&dependencies, path)?;
    ensure_sorted(&dependencies, dependency_interface_sort_key, path)?;
    Ok(dependencies)
}

fn read_dependency_interface(
    value: &CanonicalJson,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<DependencyInterfaceRef, ModuleSummaryError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(fields, &["module", "interface_hash"], path)?;
    Ok(DependencyInterfaceRef {
        module: read_identity(
            required_field(fields, "module", path)?,
            &field_path(path, "module"),
        )?,
        interface_hash: read_interface_hash(
            required_field(fields, "interface_hash", path)?,
            &field_path(path, "interface_hash"),
            schema_version,
        )?,
    })
}

fn validate_identity(
    identity: &ModuleSummaryIdentity,
    path: &str,
) -> Result<(), ModuleSummaryError> {
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

fn validate_source_range(range: SourceRangeSummary, path: &str) -> Result<(), ModuleSummaryError> {
    integer_from_u64(range.start_byte, &field_path(path, "start_byte"))?;
    integer_from_u64(range.end_byte, &field_path(path, "end_byte"))?;
    if range.start_byte > range.end_byte {
        return Err(ModuleSummaryError::InvalidField {
            path: path.to_owned(),
            reason: "start_byte must be less than or equal to end_byte".to_owned(),
        });
    }
    Ok(())
}

fn validate_exported_symbols(
    symbols: &[ExportedSymbolSummary],
    path: &str,
) -> Result<(), ModuleSummaryError> {
    ensure_no_duplicate_keys(symbols, exported_symbol_identity_key, path)?;
    for (index, symbol) in symbols.iter().enumerate() {
        validate_exported_symbol(symbol, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_exported_symbol(
    symbol: &ExportedSymbolSummary,
    path: &str,
) -> Result<(), ModuleSummaryError> {
    validate_non_empty(&symbol.origin_id, &field_path(path, "origin_id"))?;
    validate_non_empty(
        &symbol.fully_qualified_name,
        &field_path(path, "fully_qualified_name"),
    )?;
    validate_string_array(&symbol.namespace_path, &field_path(path, "namespace_path"))?;
    validate_non_empty(&symbol.visibility, &field_path(path, "visibility"))?;
    validate_non_empty(
        &symbol.declaration_kind,
        &field_path(path, "declaration_kind"),
    )?;
    validate_source_range(symbol.source_range, &field_path(path, "source_range"))?;
    validate_non_empty(
        &symbol.rendered_signature,
        &field_path(path, "rendered_signature"),
    )
}

fn validate_exported_labels(
    labels: &[ExportedLabelSummary],
    path: &str,
) -> Result<(), ModuleSummaryError> {
    ensure_no_duplicate_keys(labels, exported_label_identity_key, path)?;
    for (index, label) in labels.iter().enumerate() {
        validate_exported_label(label, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_exported_label(
    label: &ExportedLabelSummary,
    path: &str,
) -> Result<(), ModuleSummaryError> {
    validate_non_empty(&label.origin_id, &field_path(path, "origin_id"))?;
    validate_non_empty(&label.label, &field_path(path, "label"))?;
    validate_non_empty(
        &label.owner_fully_qualified_name,
        &field_path(path, "owner_fully_qualified_name"),
    )?;
    validate_non_empty(&label.visibility, &field_path(path, "visibility"))?;
    validate_source_range(label.source_range, &field_path(path, "source_range"))?;
    validate_non_empty(&label.target_kind, &field_path(path, "target_kind"))
}

fn validate_lexical_summary(
    summary: &ModuleLexicalSummary,
    path: &str,
) -> Result<(), ModuleSummaryError> {
    validate_non_empty(&summary.schema_version, &field_path(path, "schema_version"))?;
    validate_lexical_contributions(&summary.contributions, &field_path(path, "contributions"))
}

fn validate_lexical_contributions(
    contributions: &[LexicalContributionSummary],
    path: &str,
) -> Result<(), ModuleSummaryError> {
    ensure_no_duplicate_keys(contributions, lexical_contribution_sort_key, path)?;
    for (index, contribution) in contributions.iter().enumerate() {
        validate_lexical_contribution(contribution, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_lexical_contribution(
    contribution: &LexicalContributionSummary,
    path: &str,
) -> Result<(), ModuleSummaryError> {
    validate_non_empty(&contribution.kind, &field_path(path, "kind"))?;
    validate_non_empty(&contribution.key, &field_path(path, "key"))?;
    validate_non_empty(&contribution.payload, &field_path(path, "payload"))
}

fn validate_reexports(
    reexports: &[ModuleReexportSummary],
    path: &str,
) -> Result<(), ModuleSummaryError> {
    ensure_no_duplicate_keys(reexports, reexport_sort_key, path)?;
    for (index, reexport) in reexports.iter().enumerate() {
        validate_reexport(reexport, &array_path(path, index))?;
    }
    Ok(())
}

fn validate_reexport(
    reexport: &ModuleReexportSummary,
    path: &str,
) -> Result<(), ModuleSummaryError> {
    validate_identity(&reexport.target_module, &field_path(path, "target_module"))?;
    validate_optional_non_empty(
        reexport.target_item_origin_id.as_deref(),
        &field_path(path, "target_item_origin_id"),
    )?;
    validate_optional_non_empty(
        reexport.exported_name.as_deref(),
        &field_path(path, "exported_name"),
    )?;
    validate_optional_non_empty(
        reexport.provenance_origin_id.as_deref(),
        &field_path(path, "provenance_origin_id"),
    )
}

fn validate_dependency_interfaces(
    dependencies: &[DependencyInterfaceRef],
    path: &str,
) -> Result<(), ModuleSummaryError> {
    ensure_no_duplicate_keys(dependencies, dependency_module_identity_key, path)?;
    for (index, dependency) in dependencies.iter().enumerate() {
        validate_identity(
            &dependency.module,
            &field_path(&array_path(path, index), "module"),
        )?;
    }
    Ok(())
}

fn validate_non_empty(value: &str, path: &str) -> Result<(), ModuleSummaryError> {
    if value.is_empty() {
        return Err(ModuleSummaryError::InvalidField {
            path: path.to_owned(),
            reason: "must not be empty".to_owned(),
        });
    }
    Ok(())
}

fn validate_optional_non_empty(value: Option<&str>, path: &str) -> Result<(), ModuleSummaryError> {
    if matches!(value, Some("")) {
        return Err(ModuleSummaryError::InvalidField {
            path: path.to_owned(),
            reason: "must be null or a non-empty string".to_owned(),
        });
    }
    Ok(())
}

fn validate_string_array(values: &[String], path: &str) -> Result<(), ModuleSummaryError> {
    for (index, value) in values.iter().enumerate() {
        validate_non_empty(value, &array_path(path, index))?;
    }
    Ok(())
}

fn sorted_exported_symbols(symbols: &[ExportedSymbolSummary]) -> Vec<&ExportedSymbolSummary> {
    let mut symbols = symbols.iter().collect::<Vec<_>>();
    symbols.sort_by_key(|symbol| exported_symbol_sort_key(symbol));
    symbols
}

fn sorted_exported_symbols_for_interface(
    symbols: &[ExportedSymbolSummary],
) -> Vec<&ExportedSymbolSummary> {
    let mut symbols = symbols.iter().collect::<Vec<_>>();
    symbols.sort_by_key(|symbol| exported_symbol_interface_sort_key(symbol));
    symbols
}

fn sorted_exported_labels(labels: &[ExportedLabelSummary]) -> Vec<&ExportedLabelSummary> {
    let mut labels = labels.iter().collect::<Vec<_>>();
    labels.sort_by_key(|label| exported_label_sort_key(label));
    labels
}

fn sorted_exported_labels_for_interface(
    labels: &[ExportedLabelSummary],
) -> Vec<&ExportedLabelSummary> {
    let mut labels = labels.iter().collect::<Vec<_>>();
    labels.sort_by_key(|label| exported_label_interface_sort_key(label));
    labels
}

fn sorted_lexical_contributions(
    contributions: &[LexicalContributionSummary],
) -> Vec<&LexicalContributionSummary> {
    let mut contributions = contributions.iter().collect::<Vec<_>>();
    contributions.sort_by_key(|contribution| lexical_contribution_sort_key(contribution));
    contributions
}

fn sorted_reexports(reexports: &[ModuleReexportSummary]) -> Vec<&ModuleReexportSummary> {
    let mut reexports = reexports.iter().collect::<Vec<_>>();
    reexports.sort_by_key(|reexport| reexport_sort_key(reexport));
    reexports
}

fn sorted_dependency_interfaces(
    dependencies: &[DependencyInterfaceRef],
) -> Vec<&DependencyInterfaceRef> {
    let mut dependencies = dependencies.iter().collect::<Vec<_>>();
    dependencies.sort_by_key(|dependency| dependency_interface_sort_key(dependency));
    dependencies
}

type IdentityKey = (String, Option<String>, Option<String>, String, String);
type ExportedSymbolSortKey = (
    String,
    String,
    Vec<String>,
    String,
    String,
    u64,
    u64,
    String,
    String,
    Option<String>,
);
type ExportedSymbolInterfaceSortKey = (
    String,
    String,
    Vec<String>,
    String,
    String,
    String,
    String,
    Option<String>,
);
type ExportedLabelSortKey = (String, String, String, u64, u64, String, String);
type ExportedLabelInterfaceSortKey = (String, String, String, String, String);
type LexicalContributionSortKey = (String, String, String);
type ReexportSortKey = (IdentityKey, Option<String>, Option<String>, Option<String>);
type DependencyInterfaceSortKey = (IdentityKey, String);

fn identity_key(identity: &ModuleSummaryIdentity) -> IdentityKey {
    (
        identity.package_id.clone(),
        identity.package_version.clone(),
        identity.lockfile_identity.clone(),
        identity.module_path.clone(),
        identity.language_edition.clone(),
    )
}

fn exported_symbol_identity_key(symbol: &ExportedSymbolSummary) -> (String, String) {
    (
        symbol.fully_qualified_name.clone(),
        symbol.origin_id.clone(),
    )
}

fn exported_symbol_sort_key(symbol: &ExportedSymbolSummary) -> ExportedSymbolSortKey {
    (
        symbol.fully_qualified_name.clone(),
        symbol.origin_id.clone(),
        symbol.namespace_path.clone(),
        symbol.visibility.clone(),
        symbol.declaration_kind.clone(),
        symbol.source_range.start_byte,
        symbol.source_range.end_byte,
        symbol.rendered_signature.clone(),
        lower_hex_hash(symbol.interface_fingerprint),
        symbol
            .proof_status
            .map(ProofStatusSummary::as_str)
            .map(str::to_owned),
    )
}

fn exported_symbol_interface_sort_key(
    symbol: &ExportedSymbolSummary,
) -> ExportedSymbolInterfaceSortKey {
    (
        symbol.fully_qualified_name.clone(),
        symbol.origin_id.clone(),
        symbol.namespace_path.clone(),
        symbol.visibility.clone(),
        symbol.declaration_kind.clone(),
        symbol.rendered_signature.clone(),
        lower_hex_hash(symbol.interface_fingerprint),
        symbol
            .proof_status
            .map(ProofStatusSummary::as_str)
            .map(str::to_owned),
    )
}

fn exported_label_identity_key(label: &ExportedLabelSummary) -> (String, String, String) {
    (
        label.label.clone(),
        label.owner_fully_qualified_name.clone(),
        label.origin_id.clone(),
    )
}

fn exported_label_sort_key(label: &ExportedLabelSummary) -> ExportedLabelSortKey {
    (
        label.label.clone(),
        label.owner_fully_qualified_name.clone(),
        label.origin_id.clone(),
        label.source_range.start_byte,
        label.source_range.end_byte,
        label.visibility.clone(),
        label.target_kind.clone(),
    )
}

fn exported_label_interface_sort_key(
    label: &ExportedLabelSummary,
) -> ExportedLabelInterfaceSortKey {
    (
        label.label.clone(),
        label.owner_fully_qualified_name.clone(),
        label.origin_id.clone(),
        label.visibility.clone(),
        label.target_kind.clone(),
    )
}

fn lexical_contribution_sort_key(
    contribution: &LexicalContributionSummary,
) -> LexicalContributionSortKey {
    (
        contribution.kind.clone(),
        contribution.key.clone(),
        contribution.payload.clone(),
    )
}

fn reexport_sort_key(reexport: &ModuleReexportSummary) -> ReexportSortKey {
    (
        identity_key(&reexport.target_module),
        reexport.target_item_origin_id.clone(),
        reexport.exported_name.clone(),
        reexport.provenance_origin_id.clone(),
    )
}

fn dependency_module_identity_key(dependency: &DependencyInterfaceRef) -> IdentityKey {
    identity_key(&dependency.module)
}

fn dependency_interface_sort_key(
    dependency: &DependencyInterfaceRef,
) -> DependencyInterfaceSortKey {
    (
        identity_key(&dependency.module),
        lower_hex_hash(dependency.interface_hash),
    )
}

fn ensure_sorted<T, K, F>(items: &[T], mut key: F, path: &str) -> Result<(), ModuleSummaryError>
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
            return Err(ModuleSummaryError::UnsortedCollection {
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
) -> Result<(), ModuleSummaryError>
where
    K: Ord + fmt::Debug,
    F: FnMut(&T) -> K,
{
    let mut seen = BTreeSet::new();
    for item in items {
        let key = key(item);
        if seen.contains(&key) {
            return Err(ModuleSummaryError::DuplicateEntry {
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
) -> Result<&'a BTreeMap<String, CanonicalJson>, ModuleSummaryError> {
    let CanonicalJson::Object(fields) = value else {
        return Err(ModuleSummaryError::UnexpectedType {
            path: path.to_owned(),
            expected: "an object",
        });
    };
    Ok(fields)
}

fn expect_array<'a>(
    value: &'a CanonicalJson,
    path: &str,
) -> Result<&'a [CanonicalJson], ModuleSummaryError> {
    let CanonicalJson::Array(values) = value else {
        return Err(ModuleSummaryError::UnexpectedType {
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
) -> Result<&'a CanonicalJson, ModuleSummaryError> {
    fields
        .get(field)
        .ok_or_else(|| ModuleSummaryError::MissingField {
            path: field_path(path, field),
        })
}

fn reject_unknown_fields(
    fields: &BTreeMap<String, CanonicalJson>,
    allowed: &[&str],
    path: &str,
) -> Result<(), ModuleSummaryError> {
    for field in fields.keys() {
        if !allowed.contains(&field.as_str()) {
            return Err(ModuleSummaryError::UnknownField {
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
) -> Result<String, ModuleSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::String(value) = value else {
        return Err(ModuleSummaryError::UnexpectedType {
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
) -> Result<Option<String>, ModuleSummaryError> {
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
        _ => Err(ModuleSummaryError::UnexpectedType {
            path,
            expected: "a string or null",
        }),
    }
}

fn read_string_array(value: &CanonicalJson, path: &str) -> Result<Vec<String>, ModuleSummaryError> {
    let values = expect_array(value, path)?;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let path = array_path(path, index);
            let CanonicalJson::String(value) = value else {
                return Err(ModuleSummaryError::UnexpectedType {
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
) -> Result<u64, ModuleSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    let CanonicalJson::Integer(value) = value else {
        return Err(ModuleSummaryError::UnexpectedType {
            path,
            expected: "a non-negative integer",
        });
    };
    u64::try_from(*value).map_err(|_| ModuleSummaryError::InvalidField {
        path,
        reason: "must be non-negative".to_owned(),
    })
}

fn read_optional_proof_status(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<Option<ProofStatusSummary>, ModuleSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    match value {
        CanonicalJson::Null => Ok(None),
        CanonicalJson::String(value) => {
            ProofStatusSummary::from_str(value)
                .map(Some)
                .ok_or_else(|| ModuleSummaryError::InvalidField {
                    path,
                    reason: "unknown proof status".to_owned(),
                })
        }
        _ => Err(ModuleSummaryError::UnexpectedType {
            path,
            expected: "a proof-status string or null",
        }),
    }
}

fn read_optional_interface_hash(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<Option<Hash>, ModuleSummaryError> {
    let path = field_path(path, field);
    let value = required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )?;
    match value {
        CanonicalJson::Null => Ok(None),
        _ => read_interface_hash(value, &path, schema_version).map(Some),
    }
}

fn read_source_hash(value: &CanonicalJson, path: &str) -> Result<Hash, ModuleSummaryError> {
    let CanonicalJson::String(value) = value else {
        return Err(ModuleSummaryError::UnexpectedType {
            path: path.to_owned(),
            expected: "a source hash string",
        });
    };
    parse_source_hash_string(value, path)
}

fn read_interface_hash(
    value: &CanonicalJson,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<Hash, ModuleSummaryError> {
    let CanonicalJson::String(value) = value else {
        return Err(ModuleSummaryError::UnexpectedType {
            path: path.to_owned(),
            expected: "an interface hash string",
        });
    };
    parse_interface_hash_string(value, path, schema_version)
}

fn source_hash_string(hash: Hash) -> String {
    format!("{}:{}", SOURCE_HASH_CONSTRUCTION, lower_hex_hash(hash))
}

fn interface_hash_string(schema_version: SchemaVersion, hash: Hash) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        ARTIFACT_HASH_CONSTRUCTION,
        INTERFACE_HASH_CLASS,
        MODULE_SUMMARY_SCHEMA_FAMILY,
        schema_version,
        lower_hex_hash(hash)
    )
}

fn parse_source_hash_string(value: &str, path: &str) -> Result<Hash, ModuleSummaryError> {
    let Some(hex) = value
        .strip_prefix(SOURCE_HASH_CONSTRUCTION)
        .and_then(|rest| rest.strip_prefix(':'))
    else {
        return Err(ModuleSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong source hash construction label".to_owned(),
        });
    };
    parse_lower_hex_hash(hex, path)
}

fn parse_interface_hash_string(
    value: &str,
    path: &str,
    schema_version: SchemaVersion,
) -> Result<Hash, ModuleSummaryError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 5 {
        return Err(ModuleSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "expected construction:class:schema_family:schema_version:digest".to_owned(),
        });
    }
    if parts[0] != ARTIFACT_HASH_CONSTRUCTION {
        return Err(ModuleSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong artifact hash construction label".to_owned(),
        });
    }
    if parts[1] != INTERFACE_HASH_CLASS {
        return Err(ModuleSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong artifact hash class".to_owned(),
        });
    }
    if parts[2] != MODULE_SUMMARY_SCHEMA_FAMILY {
        return Err(ModuleSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong schema family".to_owned(),
        });
    }
    if parts[3] != schema_version.to_string() {
        return Err(ModuleSummaryError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong schema version".to_owned(),
        });
    }
    parse_lower_hex_hash(parts[4], path)
}

fn parse_lower_hex_hash(hex: &str, path: &str) -> Result<Hash, ModuleSummaryError> {
    if hex.len() != Hash::BYTE_LEN * 2 {
        return Err(ModuleSummaryError::InvalidHash {
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

fn parse_lower_hex_nibble(byte: u8, path: &str) -> Result<u8, ModuleSummaryError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(ModuleSummaryError::InvalidHash {
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

fn integer_from_u64(value: u64, path: &str) -> Result<i64, ModuleSummaryError> {
    i64::try_from(value).map_err(|_| ModuleSummaryError::InvalidField {
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
mod tests {
    use super::{
        DependencyInterfaceRef, ExportedLabelSummary, ExportedSymbolSummary,
        LexicalContributionSummary, MODULE_SUMMARY_SCHEMA_FAMILY, ModuleLexicalSummary,
        ModuleReexportSummary, ModuleSummary, ModuleSummaryError, ModuleSummaryIdentity,
        ModuleSummaryReadOptions, ProofStatusSummary, SourceRangeSummary, current_schema_version,
        interface_hash_string, interface_projection_json, module_summary_json,
        module_summary_json_unchecked, read_module_summary, write_module_summary,
    };
    use crate::store::{
        CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersionError, canonical_json_string,
    };
    use mizar_session::Hash;

    #[test]
    fn module_summary_round_trips_through_canonical_json() {
        let summary = sample_summary();
        let json = module_summary_json(&summary).expect("canonical module summary JSON");
        let bytes = write_module_summary(&summary).expect("canonical module summary bytes");

        assert_eq!(bytes, canonical_json_string(&json).into_bytes());
        assert_eq!(
            read_module_summary(&json, ModuleSummaryReadOptions::default()).expect("valid summary"),
            summary
        );
    }

    #[test]
    fn module_summary_writer_and_hash_are_deterministic_for_identical_inputs() {
        let summary = sample_summary();
        let first_json = module_summary_json(&summary).expect("first canonical JSON");
        let first_bytes = write_module_summary(&summary).expect("first canonical bytes");
        let first_projection =
            interface_projection_json(&summary).expect("first interface projection");
        let first_hash = summary.compute_interface_hash().expect("first hash");

        for _ in 0..3 {
            let json = module_summary_json(&summary).expect("repeated canonical JSON");
            assert_eq!(json, first_json);
            assert_eq!(canonical_json_string(&json).into_bytes(), first_bytes);
            assert_eq!(
                write_module_summary(&summary).expect("repeated canonical bytes"),
                first_bytes
            );
            assert_eq!(
                interface_projection_json(&summary).expect("repeated interface projection"),
                first_projection
            );
            assert_eq!(
                summary.compute_interface_hash().expect("repeated hash"),
                first_hash
            );
        }
    }

    #[test]
    fn writer_sorts_collections_and_reader_rejects_unsorted_arrays() {
        let mut summary = sample_summary();
        summary.exported_symbols.reverse();
        summary.exported_labels.reverse();
        summary.lexical_summary.contributions.reverse();
        summary.reexports.reverse();
        summary.dependency_interfaces.reverse();
        summary.refresh_interface_hash().expect("refresh hash");

        let json = module_summary_json(&summary).expect("writer sorts collections");
        let text = canonical_json_string(&json);
        assert!(
            text.find("Alpha.one").expect("alpha symbol")
                < text.find("Zeta.one").expect("zeta symbol")
        );
        assert!(
            text.find("\"label\":\"A1\"").expect("label A")
                < text.find("\"label\":\"Z1\"").expect("label Z")
        );

        let mut unsorted = json.clone();
        reverse_array_field(&mut unsorted, "exported_symbols");
        assert!(matches!(
            read_module_summary(&unsorted, ModuleSummaryReadOptions::default()),
            Err(ModuleSummaryError::UnsortedCollection { path }) if path == "$.exported_symbols"
        ));
    }

    #[test]
    fn interface_hash_ignores_source_metadata() {
        let mut baseline = sample_summary();
        let baseline_hash = baseline
            .refresh_interface_hash()
            .expect("baseline interface hash");
        let mut changed_metadata = baseline.clone();
        changed_metadata.source_hash = hash(99);
        changed_metadata.exported_symbols[0].source_range = SourceRangeSummary {
            start_byte: 100,
            end_byte: 120,
        };
        changed_metadata.exported_labels[0].source_range = SourceRangeSummary {
            start_byte: 200,
            end_byte: 202,
        };

        assert_eq!(
            changed_metadata
                .refresh_interface_hash()
                .expect("metadata-only interface hash"),
            baseline_hash
        );
    }

    #[test]
    fn interface_hash_uses_declared_module_summary_domain() {
        let summary = sample_summary();
        let projection = interface_projection_json(&summary).expect("interface projection");
        let expected = CanonicalHashDomain::new(
            HashClass::Interface,
            MODULE_SUMMARY_SCHEMA_FAMILY,
            current_schema_version(),
        )
        .hash(&projection, &[]);

        assert_eq!(
            summary.compute_interface_hash().expect("summary hash"),
            expected
        );
        assert_ne!(
            CanonicalHashDomain::new(
                HashClass::Artifact,
                MODULE_SUMMARY_SCHEMA_FAMILY,
                current_schema_version(),
            )
            .hash(&projection, &[]),
            expected
        );
        assert_ne!(
            CanonicalHashDomain::new(
                HashClass::Interface,
                "mizar-artifact/other-summary",
                current_schema_version(),
            )
            .hash(&projection, &[]),
            expected
        );
    }

    #[test]
    fn interface_hash_changes_for_importer_visible_projection_changes() {
        assert_interface_hash_changes("module identity", |summary| {
            summary.module.module_path = "Renamed.Hidden".to_owned();
        });
        assert_interface_hash_changes("symbol signature", |summary| {
            summary.exported_symbols[0].rendered_signature = "theorem Alpha.one: x = z".to_owned();
        });
        assert_interface_hash_changes("symbol interface fingerprint", |summary| {
            summary.exported_symbols[0].interface_fingerprint = hash(42);
        });
        assert_interface_hash_changes("proof status", |summary| {
            summary.exported_symbols[0].proof_status = Some(ProofStatusSummary::NotAccepted);
        });
        assert_interface_hash_changes("exported label", |summary| {
            summary.exported_labels[0].target_kind = "scheme".to_owned();
        });
        assert_interface_hash_changes("lexical fingerprint", |summary| {
            summary.lexical_summary.fingerprint = Some(hash(43));
        });
        assert_interface_hash_changes("lexical contribution", |summary| {
            summary.lexical_summary.contributions[0].payload = "infix alpha changed".to_owned();
        });
        assert_interface_hash_changes("reexport", |summary| {
            summary.reexports[0].exported_name = Some("Changed.Alpha".to_owned());
        });
        assert_interface_hash_changes("dependency interface", |summary| {
            summary.dependency_interfaces[0].interface_hash = hash(44);
        });
    }

    #[test]
    fn interface_hash_changes_for_exported_interface_changes() {
        let mut baseline = sample_summary();
        let baseline_hash = baseline.refresh_interface_hash().expect("baseline hash");
        let mut changed = baseline.clone();
        changed.exported_symbols[0].rendered_signature = "theorem Alpha.one: x = z".to_owned();

        assert_ne!(
            changed.refresh_interface_hash().expect("changed hash"),
            baseline_hash
        );
    }

    #[test]
    fn incompatible_version_reads_fail_cleanly() {
        let summary = sample_summary();
        let mut json = module_summary_json(&summary).expect("canonical JSON");
        replace_object_field(&mut json, "schema_version", CanonicalJson::string("2.0"));

        assert!(matches!(
            read_module_summary(
                &json,
                ModuleSummaryReadOptions {
                    artifact_path: Some("build/dep.mizir.json"),
                    ..ModuleSummaryReadOptions::default()
                },
            ),
            Err(ModuleSummaryError::SchemaVersion(
                SchemaVersionError::MajorMismatch { .. }
            ))
        ));
    }

    #[test]
    fn reader_rejects_hash_and_module_mismatches() {
        let summary = sample_summary();
        let json = module_summary_json(&summary).expect("canonical JSON");
        let other_module = ModuleSummaryIdentity {
            package_id: "other-package".to_owned(),
            ..summary.module.clone()
        };
        assert!(matches!(
            read_module_summary(
                &json,
                ModuleSummaryReadOptions {
                    expected_module: Some(&other_module),
                    ..ModuleSummaryReadOptions::default()
                }
            ),
            Err(ModuleSummaryError::ModuleIdentityMismatch { .. })
        ));

        assert!(matches!(
            read_module_summary(
                &json,
                ModuleSummaryReadOptions {
                    expected_interface_hash: Some(hash(88)),
                    ..ModuleSummaryReadOptions::default()
                }
            ),
            Err(ModuleSummaryError::ExpectedInterfaceHashMismatch { .. })
        ));

        let mut bad_hash_json = json;
        replace_object_field(
            &mut bad_hash_json,
            "interface_hash",
            CanonicalJson::string(interface_hash_string(current_schema_version(), hash(77))),
        );
        assert!(matches!(
            read_module_summary(&bad_hash_json, ModuleSummaryReadOptions::default()),
            Err(ModuleSummaryError::InterfaceHashMismatch { .. })
        ));
    }

    #[test]
    fn reader_rejects_wrong_hash_construction() {
        let summary = sample_summary();
        let mut json = module_summary_json(&summary).expect("canonical JSON");
        replace_object_field(
            &mut json,
            "interface_hash",
            CanonicalJson::string(format!(
                "mizar-artifact/artifact-framed-hash-text/v1:artifact:mizar-artifact/module-summary:1.0:{}",
                "11".repeat(Hash::BYTE_LEN)
            )),
        );

        assert!(matches!(
            read_module_summary(&json, ModuleSummaryReadOptions::default()),
            Err(ModuleSummaryError::InvalidHash { path, .. }) if path == "$.interface_hash"
        ));
    }

    #[test]
    fn reader_rejects_hash_string_domain_and_digest_mismatches() {
        let summary = sample_summary();
        let json = module_summary_json(&summary).expect("canonical JSON");
        let digest = "11".repeat(Hash::BYTE_LEN);

        for bad_hash in [
            format!(
                "mizar-artifact/other-framed-hash/v1:interface:mizar-artifact/module-summary:1.0:{digest}"
            ),
            format!(
                "mizar-artifact/artifact-framed-hash-text/v1:artifact:mizar-artifact/module-summary:1.0:{digest}"
            ),
            format!(
                "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/other-summary:1.0:{digest}"
            ),
            format!(
                "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/module-summary:1.1:{digest}"
            ),
            format!(
                "mizar-artifact/artifact-framed-hash-text/v1:interface:mizar-artifact/module-summary:1.0:{}",
                "gg".repeat(Hash::BYTE_LEN)
            ),
        ] {
            let mut bad_json = json.clone();
            replace_object_field(
                &mut bad_json,
                "interface_hash",
                CanonicalJson::string(bad_hash),
            );

            assert!(matches!(
                read_module_summary(&bad_json, ModuleSummaryReadOptions::default()),
                Err(ModuleSummaryError::InvalidHash { path, .. }) if path == "$.interface_hash"
            ));
        }

        let mut wrong_source_hash = json;
        replace_object_field(
            &mut wrong_source_hash,
            "source_hash",
            CanonicalJson::string(format!("wrong-source-hash/v1:{digest}")),
        );

        assert!(matches!(
            read_module_summary(&wrong_source_hash, ModuleSummaryReadOptions::default()),
            Err(ModuleSummaryError::InvalidHash { path, .. }) if path == "$.source_hash"
        ));
    }

    #[test]
    fn reader_rejects_unsorted_collections() {
        let summary = sample_summary();
        let json = module_summary_json(&summary).expect("canonical JSON");

        assert_unsorted_collection_rejected(
            json.clone(),
            &["exported_symbols"],
            "$.exported_symbols",
        );
        assert_unsorted_collection_rejected(
            json.clone(),
            &["exported_labels"],
            "$.exported_labels",
        );
        assert_unsorted_collection_rejected(
            json.clone(),
            &["lexical_summary", "contributions"],
            "$.lexical_summary.contributions",
        );
        assert_unsorted_collection_rejected(json.clone(), &["reexports"], "$.reexports");
        assert_unsorted_collection_rejected(
            json,
            &["dependency_interfaces"],
            "$.dependency_interfaces",
        );
    }

    #[test]
    fn reader_rejects_duplicate_collection_keys() {
        let summary = sample_summary();
        let json = module_summary_json(&summary).expect("canonical JSON");

        assert_duplicate_collection_rejected(
            json.clone(),
            &["exported_symbols"],
            "$.exported_symbols",
        );
        assert_duplicate_collection_rejected(
            json.clone(),
            &["exported_labels"],
            "$.exported_labels",
        );
        assert_duplicate_collection_rejected(
            json.clone(),
            &["lexical_summary", "contributions"],
            "$.lexical_summary.contributions",
        );
        assert_duplicate_collection_rejected(json.clone(), &["reexports"], "$.reexports");
        assert_duplicate_collection_rejected(
            json,
            &["dependency_interfaces"],
            "$.dependency_interfaces",
        );
    }

    #[test]
    fn reader_rejects_subset_duplicate_keys() {
        let mut duplicate_label_key = sample_summary();
        duplicate_label_key.exported_labels[1].label =
            duplicate_label_key.exported_labels[0].label.clone();
        duplicate_label_key.exported_labels[1].owner_fully_qualified_name = duplicate_label_key
            .exported_labels[0]
            .owner_fully_qualified_name
            .clone();
        duplicate_label_key.exported_labels[1].origin_id =
            duplicate_label_key.exported_labels[0].origin_id.clone();
        duplicate_label_key.exported_labels[1].visibility = "protected".to_owned();
        duplicate_label_key.exported_labels[1].target_kind = "scheme".to_owned();
        duplicate_label_key
            .refresh_interface_hash()
            .expect("duplicate label hash");
        let duplicate_label_json =
            module_summary_json_unchecked(&duplicate_label_key).expect("unchecked duplicate JSON");

        assert!(matches!(
            read_module_summary(&duplicate_label_json, ModuleSummaryReadOptions::default()),
            Err(ModuleSummaryError::DuplicateEntry { path, .. }) if path == "$.exported_labels"
        ));

        let mut duplicate_dependency_key = sample_summary();
        duplicate_dependency_key.dependency_interfaces[1].module = duplicate_dependency_key
            .dependency_interfaces[0]
            .module
            .clone();
        duplicate_dependency_key.dependency_interfaces[1].interface_hash = hash(77);
        duplicate_dependency_key
            .refresh_interface_hash()
            .expect("duplicate dependency hash");
        let duplicate_dependency_json = module_summary_json_unchecked(&duplicate_dependency_key)
            .expect("unchecked duplicate JSON");

        assert!(matches!(
            read_module_summary(&duplicate_dependency_json, ModuleSummaryReadOptions::default()),
            Err(ModuleSummaryError::DuplicateEntry { path, .. }) if path == "$.dependency_interfaces"
        ));
    }

    #[test]
    fn reader_rejects_duplicate_identity_keys() {
        let mut summary = sample_summary();
        summary.exported_symbols[1].fully_qualified_name =
            summary.exported_symbols[0].fully_qualified_name.clone();
        summary.exported_symbols[1].origin_id = summary.exported_symbols[0].origin_id.clone();
        summary.refresh_interface_hash().expect("refresh hash");

        assert!(matches!(
            module_summary_json(&summary),
            Err(ModuleSummaryError::DuplicateEntry { path, .. }) if path == "$.exported_symbols"
        ));
    }

    fn sample_summary() -> ModuleSummary {
        let module = identity("base", "Hidden");
        let dep_alpha = identity("dep-alpha", "AlphaDep");
        let dep_zeta = identity("dep-zeta", "ZetaDep");
        let mut summary = ModuleSummary {
            schema_version: current_schema_version(),
            module: module.clone(),
            source_hash: hash(1),
            interface_hash: hash(0),
            exported_symbols: vec![
                symbol("Zeta.one", "symbol:zeta", 20, "theorem Zeta.one: x = x", 2),
                symbol(
                    "Alpha.one",
                    "symbol:alpha",
                    0,
                    "theorem Alpha.one: x = x",
                    3,
                ),
            ],
            exported_labels: vec![label("Z1", "label:zeta", 25), label("A1", "label:alpha", 5)],
            lexical_summary: ModuleLexicalSummary {
                schema_version: "lexer-summary-v1".to_owned(),
                fingerprint: Some(hash(4)),
                contributions: vec![
                    LexicalContributionSummary {
                        kind: "notation".to_owned(),
                        key: "zeta".to_owned(),
                        payload: "infix zeta".to_owned(),
                    },
                    LexicalContributionSummary {
                        kind: "notation".to_owned(),
                        key: "alpha".to_owned(),
                        payload: "infix alpha".to_owned(),
                    },
                ],
            },
            reexports: vec![
                ModuleReexportSummary {
                    target_module: dep_zeta.clone(),
                    target_item_origin_id: Some("dep:zeta".to_owned()),
                    exported_name: Some("Dep.Zeta".to_owned()),
                    provenance_origin_id: Some("reexport:zeta".to_owned()),
                },
                ModuleReexportSummary {
                    target_module: module,
                    target_item_origin_id: Some("base:alpha".to_owned()),
                    exported_name: Some("Base.Alpha".to_owned()),
                    provenance_origin_id: Some("reexport:alpha".to_owned()),
                },
            ],
            dependency_interfaces: vec![
                DependencyInterfaceRef {
                    module: dep_zeta,
                    interface_hash: hash(5),
                },
                DependencyInterfaceRef {
                    module: dep_alpha,
                    interface_hash: hash(6),
                },
            ],
        };
        summary.refresh_interface_hash().expect("sample hash");
        module_summary_json(&summary).expect("sample is canonical");
        read_module_summary(
            &module_summary_json(&summary).expect("sample JSON"),
            ModuleSummaryReadOptions::default(),
        )
        .expect("sample round-trip")
    }

    fn identity(package_id: &str, module_path: &str) -> ModuleSummaryIdentity {
        ModuleSummaryIdentity {
            package_id: package_id.to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("lock:fixture".to_owned()),
            module_path: module_path.to_owned(),
            language_edition: "2026".to_owned(),
        }
    }

    fn symbol(
        fully_qualified_name: &str,
        origin_id: &str,
        start_byte: u64,
        rendered_signature: &str,
        fingerprint_seed: u8,
    ) -> ExportedSymbolSummary {
        ExportedSymbolSummary {
            origin_id: origin_id.to_owned(),
            fully_qualified_name: fully_qualified_name.to_owned(),
            namespace_path: vec!["Hidden".to_owned()],
            visibility: "public".to_owned(),
            declaration_kind: "theorem".to_owned(),
            source_range: SourceRangeSummary {
                start_byte,
                end_byte: start_byte + 10,
            },
            rendered_signature: rendered_signature.to_owned(),
            interface_fingerprint: hash(fingerprint_seed),
            proof_status: Some(ProofStatusSummary::Accepted),
        }
    }

    fn label(label: &str, origin_id: &str, start_byte: u64) -> ExportedLabelSummary {
        ExportedLabelSummary {
            origin_id: origin_id.to_owned(),
            label: label.to_owned(),
            owner_fully_qualified_name: "Hidden.owner".to_owned(),
            visibility: "public".to_owned(),
            source_range: SourceRangeSummary {
                start_byte,
                end_byte: start_byte + 2,
            },
            target_kind: "theorem".to_owned(),
        }
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }

    fn assert_interface_hash_changes(name: &str, mutate: impl FnOnce(&mut ModuleSummary)) {
        let baseline = sample_summary();
        let baseline_hash = baseline.interface_hash;
        let mut changed = baseline.clone();
        mutate(&mut changed);

        assert_ne!(
            changed
                .refresh_interface_hash()
                .unwrap_or_else(|error| panic!("{name}: {error}")),
            baseline_hash,
            "{name}"
        );
    }

    fn assert_unsorted_collection_rejected(
        mut json: CanonicalJson,
        fields: &[&str],
        expected_path: &str,
    ) {
        reverse_array_at(&mut json, fields);

        assert!(matches!(
            read_module_summary(&json, ModuleSummaryReadOptions::default()),
            Err(ModuleSummaryError::UnsortedCollection { path }) if path == expected_path
        ));
    }

    fn assert_duplicate_collection_rejected(
        mut json: CanonicalJson,
        fields: &[&str],
        expected_path: &str,
    ) {
        duplicate_first_array_item_at(&mut json, fields);

        assert!(matches!(
            read_module_summary(&json, ModuleSummaryReadOptions::default()),
            Err(ModuleSummaryError::DuplicateEntry { path, .. }) if path == expected_path
        ));
    }

    fn replace_object_field(value: &mut CanonicalJson, field: &str, replacement: CanonicalJson) {
        let CanonicalJson::Object(fields) = value else {
            panic!("expected object");
        };
        fields.insert(field.to_owned(), replacement);
    }

    fn reverse_array_field(value: &mut CanonicalJson, field: &str) {
        reverse_array_at(value, &[field]);
    }

    fn reverse_array_at(value: &mut CanonicalJson, fields: &[&str]) {
        let values = array_at_mut(value, fields);
        values.reverse();
    }

    fn duplicate_first_array_item_at(value: &mut CanonicalJson, fields: &[&str]) {
        let values = array_at_mut(value, fields);
        assert!(
            values.len() >= 2,
            "fixture array must contain at least two entries"
        );
        values[1] = values[0].clone();
    }

    fn array_at_mut<'a>(
        value: &'a mut CanonicalJson,
        fields: &[&str],
    ) -> &'a mut Vec<CanonicalJson> {
        let (last, parents) = fields.split_last().expect("field path must be non-empty");
        let mut current = value;
        for field in parents {
            current = object_field_mut(current, field);
        }
        let CanonicalJson::Array(values) = object_field_mut(current, last) else {
            panic!("expected array");
        };
        values
    }

    fn object_field_mut<'a>(value: &'a mut CanonicalJson, field: &str) -> &'a mut CanonicalJson {
        let CanonicalJson::Object(fields) = value else {
            panic!("expected object");
        };
        fields.get_mut(field).expect("object field")
    }
}
