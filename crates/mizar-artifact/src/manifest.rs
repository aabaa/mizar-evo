//! Package artifact manifest schema, validating reader, and transaction writer.
//!
//! The schema is specified in
//! [manifest.md](../../../../doc/design/mizar-artifact/en/manifest.md).

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::{self, Write as _},
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use mizar_session::Hash;

use crate::{
    module_summary::{
        MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummaryError, ModuleSummaryIdentity,
        ModuleSummaryReadOptions, SOURCE_HASH_CONSTRUCTION, read_module_summary,
    },
    registration_summary::{
        ArtifactHashClass, ArtifactHashRef, REGISTRATION_SUMMARY_SCHEMA_FAMILY,
        RegistrationSummaryError, RegistrationSummaryReadOptions, read_registration_summary,
    },
    store::{
        ARTIFACT_HASH_CONSTRUCTION, CanonicalHashDomain, CanonicalJson, CanonicalJsonError,
        MinorVersionPolicy, PublishedArtifactPath, PublishedArtifactReadOptions,
        PublishedArtifactWrite, PublishedPathError, SchemaVersion, SchemaVersionError,
        SchemaVersionSupport, StoreIoError, StoreIoOperation, artifact_hash_domain,
        canonical_json_bytes, read_published_artifact, write_published_artifact,
    },
    verified_artifact::{
        VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifactError, VerifiedArtifactReadOptions,
        artifact_hash_excluded_paths, read_verified_artifact,
    },
};

/// Manifest schema family.
pub const MANIFEST_SCHEMA_FAMILY: &str = "mizar-artifact/manifest";

/// Standard package manifest path relative to the artifact root.
pub const ARTIFACT_MANIFEST_PATH: &str = "artifact-manifest.json";

/// Stable package-level publication manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactManifest {
    /// Schema version read from or written to the manifest.
    pub schema_version: SchemaVersion,
    /// Stable package identity.
    pub package: PackageIdentity,
    /// Package-relative artifact root label.
    pub artifact_root: String,
    /// Producer-owned lockfile or lock projection hash.
    pub lockfile_hash: ArtifactHashRef,
    /// Toolchain identifier.
    pub toolchain: String,
    /// Language edition identifier.
    pub language_edition: String,
    /// Producer-owned verifier configuration hash.
    pub verifier_config_hash: ArtifactHashRef,
    /// Published module artifacts reachable from this manifest.
    pub modules: Vec<ModuleArtifactEntry>,
    /// Deliberately exposed development artifacts.
    pub development_artifacts: Vec<DevelopmentArtifactEntry>,
    /// Manifest generation policy metadata.
    pub provenance: ManifestProvenance,
}

/// Package identity stored in the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageIdentity {
    /// Stable package id.
    pub package_id: String,
    /// Package version when one is available.
    pub package_version: Option<String>,
    /// Lockfile identity when one is available.
    pub lockfile_identity: Option<String>,
}

/// Manifest provenance metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestProvenance {
    /// Producer name/version.
    pub generated_by: String,
    /// Publication policy label.
    pub manifest_policy: String,
    /// Transaction format label.
    pub transaction_format: String,
}

/// One module artifact entry in the package manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleArtifactEntry {
    /// Stable package/module identity.
    pub module: ModuleSummaryIdentity,
    /// Package- or workspace-relative source path.
    pub source_file: String,
    /// Exact source text hash.
    pub source_hash: Hash,
    /// Verified artifact path relative to the artifact root.
    pub artifact_file: String,
    /// Store-level verified artifact hash.
    pub artifact_hash: ArtifactHashRef,
    /// Verified artifact interface hash.
    pub interface_hash: ArtifactHashRef,
    /// Verified artifact implementation hash.
    pub implementation_hash: ArtifactHashRef,
    /// Optional module-summary path.
    pub module_summary_file: Option<String>,
    /// Optional module-summary store hash.
    pub module_summary_hash: Option<ArtifactHashRef>,
    /// Optional module-summary interface hash.
    pub module_summary_interface_hash: Option<ArtifactHashRef>,
    /// Optional registration-summary path.
    pub registration_summary_file: Option<String>,
    /// Optional registration-summary store hash.
    pub registration_summary_hash: Option<ArtifactHashRef>,
    /// Optional registration-summary interface hash.
    pub registration_interface_hash: Option<ArtifactHashRef>,
    /// Proof witness entries reachable for this module.
    pub proof_witnesses: Vec<ManifestProofWitnessEntry>,
    /// Optional diagnostics payload hash.
    pub diagnostics_hash: Option<ArtifactHashRef>,
}

/// Manifest-side proof witness reachability entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestProofWitnessEntry {
    /// Stable proof obligation id.
    pub obligation_id: String,
    /// Producer-owned obligation fingerprint.
    pub obligation_fingerprint: ArtifactHashRef,
    /// Witness payload path relative to the artifact root.
    pub witness_path: String,
    /// Producer-owned witness artifact hash.
    pub witness_artifact_hash: ArtifactHashRef,
}

/// Optional development artifact entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevelopmentArtifactEntry {
    /// Producer-owned development artifact kind.
    pub kind: String,
    /// Development artifact path relative to the artifact root.
    pub path: String,
    /// Optional producer-owned artifact hash.
    pub artifact_hash: Option<ArtifactHashRef>,
    /// Optional producer-owned diagnostic hash.
    pub diagnostic_hash: Option<ArtifactHashRef>,
    /// Optional related module identity.
    pub related_module: Option<ModuleSummaryIdentity>,
}

/// Additional validation requested while reading a manifest JSON value.
#[derive(Debug, Clone, Copy, Default)]
pub struct ArtifactManifestReadOptions<'a> {
    /// Manifest path to include in schema-version diagnostics.
    pub artifact_path: Option<&'a str>,
}

/// Additional validation requested while reading `artifact-manifest.json`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ManifestFileReadOptions {
    /// Validate module artifact, summary, and witness references from disk.
    pub validate_references: bool,
    /// Check witness file reachability when validating references.
    pub validate_witness_files: bool,
    /// Check development artifact reachability from disk.
    pub validate_development_artifacts: bool,
    /// Expected manifest file hash.
    pub expected_manifest_hash: Option<Hash>,
}

/// Completed manifest read result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedManifestRead {
    /// Manifest path.
    pub path: PublishedArtifactPath,
    /// Parsed manifest.
    pub manifest: ArtifactManifest,
    /// Store-level manifest hash.
    pub artifact_hash: Hash,
}

/// Opaque freshness hook supplied by the build coordinator.
pub trait ManifestFreshnessCheck {
    /// Returns whether the transaction guard still names the active snapshot.
    fn is_fresh(&self, freshness_guard: Option<&str>) -> bool;
}

impl<F> ManifestFreshnessCheck for F
where
    F: Fn(Option<&str>) -> bool,
{
    fn is_fresh(&self, freshness_guard: Option<&str>) -> bool {
        self(freshness_guard)
    }
}

/// Manifest transaction commit options.
#[derive(Default)]
pub struct ManifestCommitOptions<'a> {
    /// Opaque freshness check owned by the caller.
    pub freshness_check: Option<&'a dyn ManifestFreshnessCheck>,
}

/// In-progress manifest publication transaction.
#[derive(Debug, Clone)]
pub struct ManifestTransaction {
    artifact_root: PathBuf,
    seed_manifest: ArtifactManifest,
    base_manifest_hash: Option<Hash>,
    freshness_guard: Option<String>,
    staged_modules: BTreeMap<ModuleSummaryIdentity, ModuleArtifactEntry>,
}

/// Successful manifest transaction commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestCommit {
    /// Published manifest.
    pub manifest: ArtifactManifest,
    /// Store-level write result for `artifact-manifest.json`.
    pub write: PublishedArtifactWrite,
}

/// Errors produced by the manifest schema and transaction manager.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ManifestError {
    /// Canonical JSON object construction failed.
    CanonicalJson(CanonicalJsonError),
    /// Schema-version compatibility failed.
    SchemaVersion(SchemaVersionError),
    /// Store I/O or canonical JSON file validation failed.
    StoreIo(StoreIoError),
    /// A referenced verified artifact was invalid.
    VerifiedArtifact {
        /// Referenced artifact path.
        path: String,
        /// Underlying verified artifact error.
        error: Box<VerifiedArtifactError>,
    },
    /// A referenced module summary was invalid.
    ModuleSummary {
        /// Referenced artifact path.
        path: String,
        /// Underlying module summary error.
        error: Box<ModuleSummaryError>,
    },
    /// A referenced registration summary was invalid.
    RegistrationSummary {
        /// Referenced artifact path.
        path: String,
        /// Underlying registration summary error.
        error: Box<RegistrationSummaryError>,
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
    /// An optional file/hash/interface group was only partially present.
    PartialOptionalGroup { path: String, fields: String },
    /// A manifest field disagrees with a referenced artifact field.
    ReferencedArtifactMismatch {
        path: String,
        field: String,
        expected: String,
        actual: String,
    },
    /// The manifest proof witness entries do not exactly cover the artifact.
    WitnessCoverageMismatch { module: String, reason: String },
    /// A staged module was repeated with different content.
    ConflictingStagedModule { module: String },
    /// The transaction's base manifest hash no longer matches the final manifest.
    BaseManifestHashMismatch {
        expected: Option<String>,
        actual: Option<String>,
    },
    /// The caller-supplied freshness hook rejected this transaction.
    ObsoleteSnapshot { freshness_guard: Option<String> },
    /// The manifest seed does not match the current manifest metadata.
    ManifestSeedMismatch { field: String },
}

/// Returns the current manifest schema version.
pub const fn current_schema_version() -> SchemaVersion {
    SchemaVersion::new(1, 0)
}

/// Returns the supported manifest schema-version range.
pub fn schema_version_support() -> SchemaVersionSupport {
    SchemaVersionSupport::new(
        MANIFEST_SCHEMA_FAMILY,
        current_schema_version().major(),
        current_schema_version().minor(),
        MinorVersionPolicy::UpToSupported,
    )
}

/// Returns the store-level hash domain for manifests.
pub fn manifest_hash_domain(schema_version: SchemaVersion) -> CanonicalHashDomain {
    artifact_hash_domain(MANIFEST_SCHEMA_FAMILY, schema_version)
}

/// Returns the standard manifest path.
pub fn artifact_manifest_path() -> PublishedArtifactPath {
    PublishedArtifactPath::new(ARTIFACT_MANIFEST_PATH).expect("standard manifest path is valid")
}

/// Serializes a manifest to canonical UTF-8 JSON bytes.
pub fn write_artifact_manifest(manifest: &ArtifactManifest) -> Result<Vec<u8>, ManifestError> {
    artifact_manifest_json(manifest).map(|json| canonical_json_bytes(&json))
}

/// Builds the canonical JSON value for a manifest.
pub fn artifact_manifest_json(manifest: &ArtifactManifest) -> Result<CanonicalJson, ManifestError> {
    validate_manifest_shape(manifest, false)?;
    artifact_manifest_json_unchecked(manifest)
}

/// Reads and validates a manifest from a canonical JSON value.
pub fn read_artifact_manifest(
    value: &CanonicalJson,
    options: ArtifactManifestReadOptions<'_>,
) -> Result<ArtifactManifest, ManifestError> {
    let fields = expect_object(value, "$")?;
    let schema_version = read_schema_version(fields, options.artifact_path)?;
    reject_unknown_fields(
        fields,
        &[
            "schema_version",
            "package",
            "artifact_root",
            "lockfile_hash",
            "toolchain",
            "language_edition",
            "verifier_config_hash",
            "modules",
            "development_artifacts",
            "provenance",
        ],
        "$",
    )?;

    let manifest = ArtifactManifest {
        schema_version,
        package: read_package_identity(required_field(fields, "package", "$")?, "$.package")?,
        artifact_root: read_required_string(fields, "artifact_root", "$")?,
        lockfile_hash: read_required_artifact_hash_ref(
            fields,
            "lockfile_hash",
            "$",
            ArtifactHashClass::Artifact,
        )?,
        toolchain: read_required_string(fields, "toolchain", "$")?,
        language_edition: read_required_string(fields, "language_edition", "$")?,
        verifier_config_hash: read_required_artifact_hash_ref(
            fields,
            "verifier_config_hash",
            "$",
            ArtifactHashClass::Interface,
        )?,
        modules: read_module_entries(required_field(fields, "modules", "$")?, "$.modules")?,
        development_artifacts: read_development_entries(
            required_field(fields, "development_artifacts", "$")?,
            "$.development_artifacts",
        )?,
        provenance: read_manifest_provenance(
            required_field(fields, "provenance", "$")?,
            "$.provenance",
        )?,
    };

    validate_manifest_shape(&manifest, true)?;
    Ok(manifest)
}

/// Atomically writes `artifact-manifest.json`.
pub fn write_manifest_file(
    artifact_root: impl AsRef<Path>,
    manifest: &ArtifactManifest,
) -> Result<PublishedArtifactWrite, ManifestError> {
    let json = artifact_manifest_json(manifest)?;
    let domain = manifest_hash_domain(manifest.schema_version);
    write_published_artifact(
        artifact_root,
        &artifact_manifest_path(),
        &json,
        &domain,
        &[],
    )
    .map_err(ManifestError::StoreIo)
}

/// Reads `artifact-manifest.json` and optionally validates referenced files.
pub fn read_manifest_file(
    artifact_root: impl AsRef<Path>,
    options: ManifestFileReadOptions,
) -> Result<PublishedManifestRead, ManifestError> {
    let path = artifact_manifest_path();
    let read = read_published_artifact(
        artifact_root.as_ref(),
        &path,
        PublishedArtifactReadOptions::default(),
    )
    .map_err(ManifestError::StoreIo)?;
    let manifest = read_artifact_manifest(
        &read.value,
        ArtifactManifestReadOptions {
            artifact_path: Some(path.as_str()),
        },
    )?;
    let artifact_hash = manifest_hash_domain(manifest.schema_version).hash(&read.value, &[]);
    if let Some(expected) = options.expected_manifest_hash
        && expected != artifact_hash
    {
        return Err(ManifestError::StoreIo(StoreIoError::ArtifactHashMismatch {
            path: path.as_str().to_owned(),
            expected,
            actual: artifact_hash,
        }));
    }
    if options.validate_references {
        validate_manifest_references(artifact_root.as_ref(), &manifest, options)?;
    } else if options.validate_development_artifacts {
        validate_development_artifact_reachability(artifact_root.as_ref(), &manifest)?;
    }
    Ok(PublishedManifestRead {
        path,
        manifest,
        artifact_hash,
    })
}

impl ManifestTransaction {
    /// Begins a manifest transaction against the current final manifest, if any.
    pub fn begin(
        artifact_root: impl AsRef<Path>,
        seed_manifest: ArtifactManifest,
        freshness_guard: Option<String>,
    ) -> Result<Self, ManifestError> {
        validate_manifest_shape(&seed_manifest, false)?;
        let artifact_root = artifact_root.as_ref().to_path_buf();
        let base_manifest_hash =
            match read_manifest_file(&artifact_root, ManifestFileReadOptions::default()) {
                Ok(current) => {
                    ensure_seed_matches_current(&seed_manifest, &current.manifest)?;
                    Some(current.artifact_hash)
                }
                Err(error) if is_missing_manifest(&error) => None,
                Err(error) => return Err(error),
            };

        Ok(Self {
            artifact_root,
            seed_manifest,
            base_manifest_hash,
            freshness_guard,
            staged_modules: BTreeMap::new(),
        })
    }

    /// Returns the manifest hash observed when the transaction began.
    pub const fn base_manifest_hash(&self) -> Option<Hash> {
        self.base_manifest_hash
    }

    /// Returns the caller-supplied freshness guard.
    pub fn freshness_guard(&self) -> Option<&str> {
        self.freshness_guard.as_deref()
    }

    /// Stages one module entry for the transaction.
    pub fn stage_module(&mut self, entry: ModuleArtifactEntry) -> Result<(), ManifestError> {
        validate_module_entry_shape(&entry, "$.modules[]", false)?;
        let key = entry.module.clone();
        if let Some(existing) = self.staged_modules.get(&key) {
            let existing_json = module_entry_json(existing)?;
            let new_json = module_entry_json(&entry)?;
            if existing_json != new_json {
                return Err(ManifestError::ConflictingStagedModule {
                    module: identity_display(&key),
                });
            }
            return Ok(());
        }
        self.staged_modules.insert(key, entry);
        Ok(())
    }

    /// Commits the manifest transaction through the store's atomic writer.
    pub fn commit(
        self,
        options: ManifestCommitOptions<'_>,
    ) -> Result<ManifestCommit, ManifestError> {
        let current =
            match read_manifest_file(&self.artifact_root, ManifestFileReadOptions::default()) {
                Ok(current) => Some(current),
                Err(error) if is_missing_manifest(&error) => None,
                Err(error) => return Err(error),
            };
        let current_hash = current.as_ref().map(|read| read.artifact_hash);
        if current_hash != self.base_manifest_hash {
            return Err(ManifestError::BaseManifestHashMismatch {
                expected: self.base_manifest_hash.map(hash_string),
                actual: current_hash.map(hash_string),
            });
        }
        if let Some(freshness_check) = options.freshness_check
            && !freshness_check.is_fresh(self.freshness_guard.as_deref())
        {
            return Err(ManifestError::ObsoleteSnapshot {
                freshness_guard: self.freshness_guard,
            });
        }

        let mut manifest = current
            .map(|read| read.manifest)
            .unwrap_or_else(|| self.seed_manifest.clone());
        for staged in self.staged_modules.into_values() {
            upsert_module_entry(&mut manifest.modules, staged);
        }
        sort_manifest_entries(&mut manifest);
        validate_manifest_shape(&manifest, true)?;
        validate_manifest_references(
            &self.artifact_root,
            &manifest,
            ManifestFileReadOptions {
                validate_references: true,
                validate_witness_files: true,
                validate_development_artifacts: true,
                expected_manifest_hash: None,
            },
        )?;

        let write = write_manifest_file(&self.artifact_root, &manifest)?;
        Ok(ManifestCommit { manifest, write })
    }
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CanonicalJson(error) => write!(formatter, "{error}"),
            Self::SchemaVersion(error) => write!(formatter, "{error}"),
            Self::StoreIo(error) => write!(formatter, "{error}"),
            Self::VerifiedArtifact { path, error } => {
                write!(formatter, "invalid verified artifact `{path}`: {error}")
            }
            Self::ModuleSummary { path, error } => {
                write!(formatter, "invalid module summary `{path}`: {error}")
            }
            Self::RegistrationSummary { path, error } => {
                write!(formatter, "invalid registration summary `{path}`: {error}")
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
            Self::PartialOptionalGroup { path, fields } => {
                write!(
                    formatter,
                    "optional field group `{path}` is partial: {fields}"
                )
            }
            Self::ReferencedArtifactMismatch {
                path,
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "referenced artifact `{path}` field `{field}` mismatch: expected `{expected}`, got `{actual}`"
            ),
            Self::WitnessCoverageMismatch { module, reason } => {
                write!(
                    formatter,
                    "proof witness coverage mismatch for `{module}`: {reason}"
                )
            }
            Self::ConflictingStagedModule { module } => {
                write!(
                    formatter,
                    "staged module `{module}` has conflicting content"
                )
            }
            Self::BaseManifestHashMismatch { expected, actual } => write!(
                formatter,
                "manifest base hash mismatch: expected {}, got {}",
                optional_hash_display(expected.as_deref()),
                optional_hash_display(actual.as_deref())
            ),
            Self::ObsoleteSnapshot { freshness_guard } => write!(
                formatter,
                "manifest transaction freshness guard `{}` is obsolete",
                freshness_guard.as_deref().unwrap_or("<none>")
            ),
            Self::ManifestSeedMismatch { field } => {
                write!(
                    formatter,
                    "manifest seed field `{field}` differs from current manifest"
                )
            }
        }
    }
}

impl Error for ManifestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalJson(error) => Some(error),
            Self::SchemaVersion(error) => Some(error),
            Self::StoreIo(error) => Some(error),
            Self::VerifiedArtifact { error, .. } => Some(error),
            Self::ModuleSummary { error, .. } => Some(error),
            Self::RegistrationSummary { error, .. } => Some(error),
            _ => None,
        }
    }
}

fn artifact_manifest_json_unchecked(
    manifest: &ArtifactManifest,
) -> Result<CanonicalJson, ManifestError> {
    let mut modules = manifest.modules.clone();
    modules.sort_by(|left, right| left.module.cmp(&right.module));
    for module in &mut modules {
        sort_proof_witnesses(&mut module.proof_witnesses);
    }
    let mut development_artifacts = manifest.development_artifacts.clone();
    development_artifacts.sort_by(development_entry_order);

    json_object([
        (
            "schema_version",
            CanonicalJson::string(manifest.schema_version.to_string()),
        ),
        ("package", package_identity_json(&manifest.package)?),
        (
            "artifact_root",
            CanonicalJson::string(&manifest.artifact_root),
        ),
        (
            "lockfile_hash",
            CanonicalJson::string(manifest.lockfile_hash.to_artifact_hash_string()),
        ),
        ("toolchain", CanonicalJson::string(&manifest.toolchain)),
        (
            "language_edition",
            CanonicalJson::string(&manifest.language_edition),
        ),
        (
            "verifier_config_hash",
            CanonicalJson::string(manifest.verifier_config_hash.to_artifact_hash_string()),
        ),
        (
            "modules",
            CanonicalJson::array(
                modules
                    .iter()
                    .map(module_entry_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "development_artifacts",
            CanonicalJson::array(
                development_artifacts
                    .iter()
                    .map(development_entry_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "provenance",
            manifest_provenance_json(&manifest.provenance)?,
        ),
    ])
}

fn package_identity_json(identity: &PackageIdentity) -> Result<CanonicalJson, ManifestError> {
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
    ])
}

fn manifest_provenance_json(
    provenance: &ManifestProvenance,
) -> Result<CanonicalJson, ManifestError> {
    json_object([
        (
            "generated_by",
            CanonicalJson::string(&provenance.generated_by),
        ),
        (
            "manifest_policy",
            CanonicalJson::string(&provenance.manifest_policy),
        ),
        (
            "transaction_format",
            CanonicalJson::string(&provenance.transaction_format),
        ),
    ])
}

fn module_entry_json(entry: &ModuleArtifactEntry) -> Result<CanonicalJson, ManifestError> {
    let mut proof_witnesses = entry.proof_witnesses.clone();
    sort_proof_witnesses(&mut proof_witnesses);
    json_object([
        ("module", identity_json(&entry.module)?),
        ("source_file", CanonicalJson::string(&entry.source_file)),
        (
            "source_hash",
            CanonicalJson::string(source_hash_string(entry.source_hash)),
        ),
        ("artifact_file", CanonicalJson::string(&entry.artifact_file)),
        (
            "artifact_hash",
            CanonicalJson::string(entry.artifact_hash.to_artifact_hash_string()),
        ),
        (
            "interface_hash",
            CanonicalJson::string(entry.interface_hash.to_artifact_hash_string()),
        ),
        (
            "implementation_hash",
            CanonicalJson::string(entry.implementation_hash.to_artifact_hash_string()),
        ),
        (
            "module_summary_file",
            optional_string_json(entry.module_summary_file.as_deref()),
        ),
        (
            "module_summary_hash",
            optional_artifact_hash_json(entry.module_summary_hash.as_ref()),
        ),
        (
            "module_summary_interface_hash",
            optional_artifact_hash_json(entry.module_summary_interface_hash.as_ref()),
        ),
        (
            "registration_summary_file",
            optional_string_json(entry.registration_summary_file.as_deref()),
        ),
        (
            "registration_summary_hash",
            optional_artifact_hash_json(entry.registration_summary_hash.as_ref()),
        ),
        (
            "registration_interface_hash",
            optional_artifact_hash_json(entry.registration_interface_hash.as_ref()),
        ),
        (
            "proof_witnesses",
            CanonicalJson::array(
                proof_witnesses
                    .iter()
                    .map(proof_witness_entry_json)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        ),
        (
            "diagnostics_hash",
            optional_artifact_hash_json(entry.diagnostics_hash.as_ref()),
        ),
    ])
}

fn proof_witness_entry_json(
    entry: &ManifestProofWitnessEntry,
) -> Result<CanonicalJson, ManifestError> {
    json_object([
        ("obligation_id", CanonicalJson::string(&entry.obligation_id)),
        (
            "obligation_fingerprint",
            CanonicalJson::string(entry.obligation_fingerprint.to_artifact_hash_string()),
        ),
        ("witness_path", CanonicalJson::string(&entry.witness_path)),
        (
            "witness_artifact_hash",
            CanonicalJson::string(entry.witness_artifact_hash.to_artifact_hash_string()),
        ),
    ])
}

fn development_entry_json(
    entry: &DevelopmentArtifactEntry,
) -> Result<CanonicalJson, ManifestError> {
    json_object([
        ("kind", CanonicalJson::string(&entry.kind)),
        ("path", CanonicalJson::string(&entry.path)),
        (
            "artifact_hash",
            optional_artifact_hash_json(entry.artifact_hash.as_ref()),
        ),
        (
            "diagnostic_hash",
            optional_artifact_hash_json(entry.diagnostic_hash.as_ref()),
        ),
        (
            "related_module",
            optional_identity_json(entry.related_module.as_ref())?,
        ),
    ])
}

fn identity_json(identity: &ModuleSummaryIdentity) -> Result<CanonicalJson, ManifestError> {
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

fn optional_identity_json(
    identity: Option<&ModuleSummaryIdentity>,
) -> Result<CanonicalJson, ManifestError> {
    identity.map_or(Ok(CanonicalJson::Null), identity_json)
}

fn optional_string_json(value: Option<&str>) -> CanonicalJson {
    value.map_or(CanonicalJson::Null, CanonicalJson::string)
}

fn optional_artifact_hash_json(value: Option<&ArtifactHashRef>) -> CanonicalJson {
    value.map_or(CanonicalJson::Null, |hash_ref| {
        CanonicalJson::string(hash_ref.to_artifact_hash_string())
    })
}

fn read_schema_version(
    fields: &BTreeMap<String, CanonicalJson>,
    artifact_path: Option<&str>,
) -> Result<SchemaVersion, ManifestError> {
    let Some(value) = fields.get("schema_version") else {
        return Err(ManifestError::SchemaVersion(
            schema_version_support()
                .check(None)
                .expect_err("missing version must fail"),
        ));
    };
    let CanonicalJson::String(value) = value else {
        return Err(ManifestError::UnexpectedType {
            path: "$.schema_version".to_owned(),
            expected: "a schema-version string",
        });
    };
    let support = schema_version_support();
    let checked = match artifact_path {
        Some(path) => support.check_at_path(Some(value), path),
        None => support.check(Some(value)),
    };
    checked.map_err(ManifestError::SchemaVersion)
}

fn read_package_identity(
    value: &CanonicalJson,
    path: &str,
) -> Result<PackageIdentity, ManifestError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &["package_id", "package_version", "lockfile_identity"],
        path,
    )?;
    Ok(PackageIdentity {
        package_id: read_required_string(fields, "package_id", path)?,
        package_version: read_optional_string(fields, "package_version", path)?,
        lockfile_identity: read_optional_string(fields, "lockfile_identity", path)?,
    })
}

fn read_manifest_provenance(
    value: &CanonicalJson,
    path: &str,
) -> Result<ManifestProvenance, ManifestError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &["generated_by", "manifest_policy", "transaction_format"],
        path,
    )?;
    Ok(ManifestProvenance {
        generated_by: read_required_string(fields, "generated_by", path)?,
        manifest_policy: read_required_string(fields, "manifest_policy", path)?,
        transaction_format: read_required_string(fields, "transaction_format", path)?,
    })
}

fn read_identity(
    value: &CanonicalJson,
    path: &str,
) -> Result<ModuleSummaryIdentity, ManifestError> {
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
    Ok(ModuleSummaryIdentity {
        package_id: read_required_string(fields, "package_id", path)?,
        package_version: read_optional_string(fields, "package_version", path)?,
        lockfile_identity: read_optional_string(fields, "lockfile_identity", path)?,
        module_path: read_required_string(fields, "module_path", path)?,
        language_edition: read_required_string(fields, "language_edition", path)?,
    })
}

fn read_module_entries(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<ModuleArtifactEntry>, ManifestError> {
    let CanonicalJson::Array(values) = value else {
        return Err(ManifestError::UnexpectedType {
            path: path.to_owned(),
            expected: "an array",
        });
    };
    values
        .iter()
        .enumerate()
        .map(|(index, value)| read_module_entry(value, &array_path(path, index)))
        .collect()
}

fn read_module_entry(
    value: &CanonicalJson,
    path: &str,
) -> Result<ModuleArtifactEntry, ManifestError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "module",
            "source_file",
            "source_hash",
            "artifact_file",
            "artifact_hash",
            "interface_hash",
            "implementation_hash",
            "module_summary_file",
            "module_summary_hash",
            "module_summary_interface_hash",
            "registration_summary_file",
            "registration_summary_hash",
            "registration_interface_hash",
            "proof_witnesses",
            "diagnostics_hash",
        ],
        path,
    )?;

    Ok(ModuleArtifactEntry {
        module: read_identity(
            required_field(fields, "module", path)?,
            &field_path(path, "module"),
        )?,
        source_file: read_required_string(fields, "source_file", path)?,
        source_hash: read_source_hash(
            required_field(fields, "source_hash", path)?,
            &field_path(path, "source_hash"),
        )?,
        artifact_file: read_required_string(fields, "artifact_file", path)?,
        artifact_hash: read_required_artifact_hash_ref(
            fields,
            "artifact_hash",
            path,
            ArtifactHashClass::Artifact,
        )?,
        interface_hash: read_required_artifact_hash_ref(
            fields,
            "interface_hash",
            path,
            ArtifactHashClass::Interface,
        )?,
        implementation_hash: read_required_artifact_hash_ref(
            fields,
            "implementation_hash",
            path,
            ArtifactHashClass::Implementation,
        )?,
        module_summary_file: read_optional_string(fields, "module_summary_file", path)?,
        module_summary_hash: read_optional_artifact_hash_ref(
            fields,
            "module_summary_hash",
            path,
            ArtifactHashClass::Artifact,
        )?,
        module_summary_interface_hash: read_optional_artifact_hash_ref(
            fields,
            "module_summary_interface_hash",
            path,
            ArtifactHashClass::Interface,
        )?,
        registration_summary_file: read_optional_string(fields, "registration_summary_file", path)?,
        registration_summary_hash: read_optional_artifact_hash_ref(
            fields,
            "registration_summary_hash",
            path,
            ArtifactHashClass::Artifact,
        )?,
        registration_interface_hash: read_optional_artifact_hash_ref(
            fields,
            "registration_interface_hash",
            path,
            ArtifactHashClass::Interface,
        )?,
        proof_witnesses: read_proof_witness_entries(
            required_field(fields, "proof_witnesses", path)?,
            &field_path(path, "proof_witnesses"),
        )?,
        diagnostics_hash: read_optional_artifact_hash_ref(
            fields,
            "diagnostics_hash",
            path,
            ArtifactHashClass::Diagnostic,
        )?,
    })
}

fn read_proof_witness_entries(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<ManifestProofWitnessEntry>, ManifestError> {
    let CanonicalJson::Array(values) = value else {
        return Err(ManifestError::UnexpectedType {
            path: path.to_owned(),
            expected: "an array",
        });
    };
    values
        .iter()
        .enumerate()
        .map(|(index, value)| read_proof_witness_entry(value, &array_path(path, index)))
        .collect()
}

fn read_proof_witness_entry(
    value: &CanonicalJson,
    path: &str,
) -> Result<ManifestProofWitnessEntry, ManifestError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "obligation_id",
            "obligation_fingerprint",
            "witness_path",
            "witness_artifact_hash",
        ],
        path,
    )?;
    Ok(ManifestProofWitnessEntry {
        obligation_id: read_required_string(fields, "obligation_id", path)?,
        obligation_fingerprint: read_required_artifact_hash_ref(
            fields,
            "obligation_fingerprint",
            path,
            ArtifactHashClass::Interface,
        )?,
        witness_path: read_required_string(fields, "witness_path", path)?,
        witness_artifact_hash: read_required_artifact_hash_ref(
            fields,
            "witness_artifact_hash",
            path,
            ArtifactHashClass::Artifact,
        )?,
    })
}

fn read_development_entries(
    value: &CanonicalJson,
    path: &str,
) -> Result<Vec<DevelopmentArtifactEntry>, ManifestError> {
    let CanonicalJson::Array(values) = value else {
        return Err(ManifestError::UnexpectedType {
            path: path.to_owned(),
            expected: "an array",
        });
    };
    values
        .iter()
        .enumerate()
        .map(|(index, value)| read_development_entry(value, &array_path(path, index)))
        .collect()
}

fn read_development_entry(
    value: &CanonicalJson,
    path: &str,
) -> Result<DevelopmentArtifactEntry, ManifestError> {
    let fields = expect_object(value, path)?;
    reject_unknown_fields(
        fields,
        &[
            "kind",
            "path",
            "artifact_hash",
            "diagnostic_hash",
            "related_module",
        ],
        path,
    )?;
    Ok(DevelopmentArtifactEntry {
        kind: read_required_string(fields, "kind", path)?,
        path: read_required_string(fields, "path", path)?,
        artifact_hash: read_optional_artifact_hash_ref(
            fields,
            "artifact_hash",
            path,
            ArtifactHashClass::Artifact,
        )?,
        diagnostic_hash: read_optional_artifact_hash_ref(
            fields,
            "diagnostic_hash",
            path,
            ArtifactHashClass::Diagnostic,
        )?,
        related_module: read_optional_identity(fields, "related_module", path)?,
    })
}

fn validate_manifest_shape(
    manifest: &ArtifactManifest,
    strict_order: bool,
) -> Result<(), ManifestError> {
    schema_version_support()
        .check(Some(&manifest.schema_version.to_string()))
        .map_err(ManifestError::SchemaVersion)?;
    validate_package_identity(&manifest.package, "$.package")?;
    validate_published_path_string(&manifest.artifact_root, "$.artifact_root")?;
    validate_artifact_hash_ref(
        &manifest.lockfile_hash,
        "$.lockfile_hash",
        ArtifactHashClass::Artifact,
        None,
        None,
    )?;
    validate_non_empty(&manifest.toolchain, "$.toolchain")?;
    validate_non_empty(&manifest.language_edition, "$.language_edition")?;
    validate_artifact_hash_ref(
        &manifest.verifier_config_hash,
        "$.verifier_config_hash",
        ArtifactHashClass::Interface,
        None,
        None,
    )?;
    validate_modules(&manifest.modules, "$.modules", strict_order)?;
    validate_development_entries(
        &manifest.development_artifacts,
        "$.development_artifacts",
        strict_order,
    )?;
    validate_manifest_provenance(&manifest.provenance, "$.provenance")
}

fn validate_package_identity(identity: &PackageIdentity, path: &str) -> Result<(), ManifestError> {
    validate_non_empty(&identity.package_id, &field_path(path, "package_id"))?;
    validate_optional_non_empty(
        identity.package_version.as_deref(),
        &field_path(path, "package_version"),
    )?;
    validate_optional_non_empty(
        identity.lockfile_identity.as_deref(),
        &field_path(path, "lockfile_identity"),
    )
}

fn validate_manifest_provenance(
    provenance: &ManifestProvenance,
    path: &str,
) -> Result<(), ManifestError> {
    validate_non_empty(&provenance.generated_by, &field_path(path, "generated_by"))?;
    validate_non_empty(
        &provenance.manifest_policy,
        &field_path(path, "manifest_policy"),
    )?;
    validate_non_empty(
        &provenance.transaction_format,
        &field_path(path, "transaction_format"),
    )
}

fn validate_modules(
    modules: &[ModuleArtifactEntry],
    path: &str,
    strict_order: bool,
) -> Result<(), ManifestError> {
    let mut seen = BTreeSet::new();
    let mut previous: Option<&ModuleSummaryIdentity> = None;
    for (index, entry) in modules.iter().enumerate() {
        let item_path = array_path(path, index);
        if !seen.insert(entry.module.clone()) {
            return Err(ManifestError::DuplicateEntry {
                path: path.to_owned(),
                key: identity_display(&entry.module),
            });
        }
        if strict_order
            && let Some(previous) = previous
            && previous > &entry.module
        {
            return Err(ManifestError::UnsortedCollection {
                path: path.to_owned(),
            });
        }
        previous = Some(&entry.module);
        validate_module_entry_shape(entry, &item_path, strict_order)?;
    }
    Ok(())
}

fn validate_module_entry_shape(
    entry: &ModuleArtifactEntry,
    path: &str,
    strict_order: bool,
) -> Result<(), ManifestError> {
    validate_identity(&entry.module, &field_path(path, "module"))?;
    validate_published_path_string(&entry.source_file, &field_path(path, "source_file"))?;
    validate_published_path_string(&entry.artifact_file, &field_path(path, "artifact_file"))?;
    validate_artifact_hash_ref(
        &entry.artifact_hash,
        &field_path(path, "artifact_hash"),
        ArtifactHashClass::Artifact,
        Some(VERIFIED_ARTIFACT_SCHEMA_FAMILY),
        Some(entry.artifact_hash.schema_version),
    )?;
    validate_artifact_hash_ref(
        &entry.interface_hash,
        &field_path(path, "interface_hash"),
        ArtifactHashClass::Interface,
        Some(VERIFIED_ARTIFACT_SCHEMA_FAMILY),
        Some(entry.artifact_hash.schema_version),
    )?;
    validate_artifact_hash_ref(
        &entry.implementation_hash,
        &field_path(path, "implementation_hash"),
        ArtifactHashClass::Implementation,
        Some(VERIFIED_ARTIFACT_SCHEMA_FAMILY),
        Some(entry.artifact_hash.schema_version),
    )?;
    validate_optional_summary_group(
        path,
        "module_summary",
        entry.module_summary_file.as_deref(),
        entry.module_summary_hash.as_ref(),
        entry.module_summary_interface_hash.as_ref(),
        MODULE_SUMMARY_SCHEMA_FAMILY,
    )?;
    validate_optional_summary_group(
        path,
        "registration_summary",
        entry.registration_summary_file.as_deref(),
        entry.registration_summary_hash.as_ref(),
        entry.registration_interface_hash.as_ref(),
        REGISTRATION_SUMMARY_SCHEMA_FAMILY,
    )?;
    validate_proof_witness_entries(
        &entry.proof_witnesses,
        &field_path(path, "proof_witnesses"),
        strict_order,
    )?;
    if let Some(hash) = &entry.diagnostics_hash {
        validate_artifact_hash_ref(
            hash,
            &field_path(path, "diagnostics_hash"),
            ArtifactHashClass::Diagnostic,
            None,
            None,
        )?;
    }
    Ok(())
}

fn validate_optional_summary_group(
    path: &str,
    label: &str,
    file: Option<&str>,
    artifact_hash: Option<&ArtifactHashRef>,
    interface_hash: Option<&ArtifactHashRef>,
    schema_family: &str,
) -> Result<(), ManifestError> {
    match (file, artifact_hash, interface_hash) {
        (None, None, None) => Ok(()),
        (Some(file), Some(artifact_hash), Some(interface_hash)) => {
            validate_published_path_string(file, &field_path(path, &format!("{label}_file")))?;
            validate_artifact_hash_ref(
                artifact_hash,
                &field_path(path, &format!("{label}_hash")),
                ArtifactHashClass::Artifact,
                Some(schema_family),
                Some(artifact_hash.schema_version),
            )?;
            validate_artifact_hash_ref(
                interface_hash,
                &field_path(path, &format!("{label}_interface_hash")),
                ArtifactHashClass::Interface,
                Some(schema_family),
                Some(artifact_hash.schema_version),
            )
        }
        _ => Err(ManifestError::PartialOptionalGroup {
            path: field_path(path, label),
            fields: format!("{label}_file, {label}_hash, {label}_interface_hash"),
        }),
    }
}

fn validate_proof_witness_entries(
    entries: &[ManifestProofWitnessEntry],
    path: &str,
    strict_order: bool,
) -> Result<(), ManifestError> {
    let mut seen = BTreeSet::new();
    let mut previous: Option<(String, String, String)> = None;
    for (index, entry) in entries.iter().enumerate() {
        let item_path = array_path(path, index);
        if !seen.insert(entry.obligation_id.clone()) {
            return Err(ManifestError::DuplicateEntry {
                path: path.to_owned(),
                key: entry.obligation_id.clone(),
            });
        }
        let key = proof_witness_order_key(entry);
        if strict_order
            && let Some(previous) = previous.as_ref()
            && previous > &key
        {
            return Err(ManifestError::UnsortedCollection {
                path: path.to_owned(),
            });
        }
        previous = Some(key);
        validate_non_empty(
            &entry.obligation_id,
            &field_path(&item_path, "obligation_id"),
        )?;
        validate_artifact_hash_ref(
            &entry.obligation_fingerprint,
            &field_path(&item_path, "obligation_fingerprint"),
            ArtifactHashClass::Interface,
            None,
            None,
        )?;
        validate_published_path_string(
            &entry.witness_path,
            &field_path(&item_path, "witness_path"),
        )?;
        validate_artifact_hash_ref(
            &entry.witness_artifact_hash,
            &field_path(&item_path, "witness_artifact_hash"),
            ArtifactHashClass::Artifact,
            None,
            None,
        )?;
    }
    Ok(())
}

fn validate_development_entries(
    entries: &[DevelopmentArtifactEntry],
    path: &str,
    strict_order: bool,
) -> Result<(), ManifestError> {
    let mut seen = BTreeSet::new();
    let mut previous: Option<(String, String, Option<ModuleSummaryIdentity>)> = None;
    for (index, entry) in entries.iter().enumerate() {
        let item_path = array_path(path, index);
        let duplicate_key = (entry.kind.clone(), entry.path.clone());
        if !seen.insert(duplicate_key.clone()) {
            return Err(ManifestError::DuplicateEntry {
                path: path.to_owned(),
                key: format!("{}:{}", duplicate_key.0, duplicate_key.1),
            });
        }
        let order_key = development_entry_order_key(entry);
        if strict_order
            && let Some(previous) = previous.as_ref()
            && previous > &order_key
        {
            return Err(ManifestError::UnsortedCollection {
                path: path.to_owned(),
            });
        }
        previous = Some(order_key);
        validate_non_empty(&entry.kind, &field_path(&item_path, "kind"))?;
        validate_published_path_string(&entry.path, &field_path(&item_path, "path"))?;
        match (&entry.artifact_hash, &entry.diagnostic_hash) {
            (Some(artifact_hash), None) => validate_artifact_hash_ref(
                artifact_hash,
                &field_path(&item_path, "artifact_hash"),
                ArtifactHashClass::Artifact,
                None,
                None,
            )?,
            (None, Some(diagnostic_hash)) => validate_artifact_hash_ref(
                diagnostic_hash,
                &field_path(&item_path, "diagnostic_hash"),
                ArtifactHashClass::Diagnostic,
                None,
                None,
            )?,
            _ => {
                return Err(ManifestError::InvalidField {
                    path: item_path,
                    reason: "exactly one of artifact_hash or diagnostic_hash must be non-null"
                        .to_owned(),
                });
            }
        }
        if let Some(module) = &entry.related_module {
            validate_identity(module, &field_path(&item_path, "related_module"))?;
        }
    }
    Ok(())
}

fn validate_identity(identity: &ModuleSummaryIdentity, path: &str) -> Result<(), ManifestError> {
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

fn validate_non_empty(value: &str, path: &str) -> Result<(), ManifestError> {
    if value.is_empty() {
        return Err(ManifestError::InvalidField {
            path: path.to_owned(),
            reason: "must be non-empty".to_owned(),
        });
    }
    Ok(())
}

fn validate_optional_non_empty(value: Option<&str>, path: &str) -> Result<(), ManifestError> {
    if let Some(value) = value {
        validate_non_empty(value, path)?;
    }
    Ok(())
}

fn validate_published_path_string(path: &str, field: &str) -> Result<(), ManifestError> {
    PublishedArtifactPath::new(path)
        .map(|_| ())
        .map_err(|error| match error {
            StoreIoError::Path { reason, .. } => ManifestError::InvalidField {
                path: field.to_owned(),
                reason: reason.to_string(),
            },
            other => ManifestError::StoreIo(other),
        })
}

fn validate_artifact_hash_ref(
    hash_ref: &ArtifactHashRef,
    path: &str,
    expected_class: ArtifactHashClass,
    expected_family: Option<&str>,
    expected_version: Option<SchemaVersion>,
) -> Result<(), ManifestError> {
    if hash_ref.class != expected_class {
        return Err(ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: format!(
                "wrong artifact hash class, expected {}",
                artifact_hash_class_str(expected_class)
            ),
        });
    }
    validate_schema_family(&hash_ref.schema_family, path)?;
    if let Some(expected_family) = expected_family
        && hash_ref.schema_family != expected_family
    {
        return Err(ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong schema family".to_owned(),
        });
    }
    if let Some(expected_version) = expected_version
        && hash_ref.schema_version != expected_version
    {
        return Err(ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong schema version".to_owned(),
        });
    }
    Ok(())
}

fn validate_manifest_references(
    artifact_root: &Path,
    manifest: &ArtifactManifest,
    options: ManifestFileReadOptions,
) -> Result<(), ManifestError> {
    for entry in &manifest.modules {
        validate_module_references(artifact_root, entry, options.validate_witness_files)?;
    }
    if options.validate_development_artifacts {
        validate_development_artifact_reachability(artifact_root, manifest)?;
    }
    Ok(())
}

fn validate_module_references(
    artifact_root: &Path,
    entry: &ModuleArtifactEntry,
    validate_witness_files: bool,
) -> Result<(), ManifestError> {
    let artifact_path =
        PublishedArtifactPath::new(&entry.artifact_file).map_err(ManifestError::StoreIo)?;
    let domain = artifact_hash_domain(
        VERIFIED_ARTIFACT_SCHEMA_FAMILY,
        entry.artifact_hash.schema_version,
    );
    let excluded_paths = artifact_hash_excluded_paths();
    let read = read_published_artifact(
        artifact_root,
        &artifact_path,
        PublishedArtifactReadOptions {
            artifact_hash_domain: Some(&domain),
            hash_excluded_paths: &excluded_paths,
            expected_artifact_hash: Some(entry.artifact_hash.digest),
        },
    )
    .map_err(ManifestError::StoreIo)?;
    let artifact = read_verified_artifact(
        &read.value,
        VerifiedArtifactReadOptions {
            artifact_path: Some(artifact_path.as_str()),
            expected_module: Some(&entry.module),
            expected_interface_hash: Some(entry.interface_hash.digest),
            expected_implementation_hash: Some(entry.implementation_hash.digest),
        },
    )
    .map_err(|error| ManifestError::VerifiedArtifact {
        path: entry.artifact_file.clone(),
        error: Box::new(error),
    })?;
    if artifact.schema_version != entry.artifact_hash.schema_version {
        return Err(ManifestError::ReferencedArtifactMismatch {
            path: entry.artifact_file.clone(),
            field: "schema_version".to_owned(),
            expected: entry.artifact_hash.schema_version.to_string(),
            actual: artifact.schema_version.to_string(),
        });
    }

    if artifact.source_file != entry.source_file {
        return Err(ManifestError::ReferencedArtifactMismatch {
            path: entry.artifact_file.clone(),
            field: "source_file".to_owned(),
            expected: entry.source_file.clone(),
            actual: artifact.source_file,
        });
    }
    if artifact.source_hash != entry.source_hash {
        return Err(ManifestError::ReferencedArtifactMismatch {
            path: entry.artifact_file.clone(),
            field: "source_hash".to_owned(),
            expected: source_hash_string(entry.source_hash),
            actual: source_hash_string(artifact.source_hash),
        });
    }

    validate_witness_coverage(entry, &artifact.proof_witnesses)?;
    if validate_witness_files {
        for witness in &entry.proof_witnesses {
            let path = PublishedArtifactPath::new(&witness.witness_path)
                .map_err(ManifestError::StoreIo)?;
            ensure_reachable_file(artifact_root, &path)?;
        }
    }
    validate_optional_module_summary(artifact_root, entry)?;
    validate_optional_registration_summary(artifact_root, entry)
}

fn validate_witness_coverage(
    entry: &ModuleArtifactEntry,
    witnesses: &[crate::proof_witness::ProofWitnessRef],
) -> Result<(), ManifestError> {
    let mut expected = witnesses
        .iter()
        .map(|witness| {
            (
                witness.obligation_id.clone(),
                witness.obligation_fingerprint.clone(),
                witness.witness_path.clone(),
                witness.witness_artifact_hash.clone(),
            )
        })
        .collect::<Vec<_>>();
    let mut actual = entry
        .proof_witnesses
        .iter()
        .map(|witness| {
            (
                witness.obligation_id.clone(),
                witness.obligation_fingerprint.clone(),
                witness.witness_path.clone(),
                witness.witness_artifact_hash.clone(),
            )
        })
        .collect::<Vec<_>>();
    expected.sort_by_key(witness_tuple_key);
    actual.sort_by_key(witness_tuple_key);
    if expected != actual {
        return Err(ManifestError::WitnessCoverageMismatch {
            module: identity_display(&entry.module),
            reason:
                "manifest proof_witnesses do not exactly match VerifiedArtifact.proof_witnesses"
                    .to_owned(),
        });
    }
    Ok(())
}

fn validate_optional_module_summary(
    artifact_root: &Path,
    entry: &ModuleArtifactEntry,
) -> Result<(), ManifestError> {
    let (Some(file), Some(artifact_hash), Some(interface_hash)) = (
        &entry.module_summary_file,
        &entry.module_summary_hash,
        &entry.module_summary_interface_hash,
    ) else {
        return Ok(());
    };
    let path = PublishedArtifactPath::new(file).map_err(ManifestError::StoreIo)?;
    let domain = artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, artifact_hash.schema_version);
    let read = read_published_artifact(
        artifact_root,
        &path,
        PublishedArtifactReadOptions {
            artifact_hash_domain: Some(&domain),
            hash_excluded_paths: &[],
            expected_artifact_hash: Some(artifact_hash.digest),
        },
    )
    .map_err(ManifestError::StoreIo)?;
    let summary = read_module_summary(
        &read.value,
        ModuleSummaryReadOptions {
            artifact_path: Some(path.as_str()),
            expected_module: Some(&entry.module),
            expected_interface_hash: Some(interface_hash.digest),
        },
    )
    .map_err(|error| ManifestError::ModuleSummary {
        path: file.clone(),
        error: Box::new(error),
    })?;
    if summary.schema_version != artifact_hash.schema_version {
        return Err(ManifestError::ReferencedArtifactMismatch {
            path: file.clone(),
            field: "schema_version".to_owned(),
            expected: artifact_hash.schema_version.to_string(),
            actual: summary.schema_version.to_string(),
        });
    }
    if summary.source_hash != entry.source_hash {
        return Err(ManifestError::ReferencedArtifactMismatch {
            path: file.clone(),
            field: "source_hash".to_owned(),
            expected: source_hash_string(entry.source_hash),
            actual: source_hash_string(summary.source_hash),
        });
    }
    Ok(())
}

fn validate_optional_registration_summary(
    artifact_root: &Path,
    entry: &ModuleArtifactEntry,
) -> Result<(), ManifestError> {
    let (Some(file), Some(artifact_hash), Some(interface_hash)) = (
        &entry.registration_summary_file,
        &entry.registration_summary_hash,
        &entry.registration_interface_hash,
    ) else {
        return Ok(());
    };
    let path = PublishedArtifactPath::new(file).map_err(ManifestError::StoreIo)?;
    let domain = artifact_hash_domain(
        REGISTRATION_SUMMARY_SCHEMA_FAMILY,
        artifact_hash.schema_version,
    );
    let read = read_published_artifact(
        artifact_root,
        &path,
        PublishedArtifactReadOptions {
            artifact_hash_domain: Some(&domain),
            hash_excluded_paths: &[],
            expected_artifact_hash: Some(artifact_hash.digest),
        },
    )
    .map_err(ManifestError::StoreIo)?;
    let summary = read_registration_summary(
        &read.value,
        RegistrationSummaryReadOptions {
            artifact_path: Some(path.as_str()),
            expected_module: Some(&entry.module),
            expected_registration_interface_hash: Some(interface_hash.digest),
            supplied_trace_artifacts: &[],
        },
    )
    .map_err(|error| ManifestError::RegistrationSummary {
        path: file.clone(),
        error: Box::new(error),
    })?;
    if summary.schema_version != artifact_hash.schema_version {
        return Err(ManifestError::ReferencedArtifactMismatch {
            path: file.clone(),
            field: "schema_version".to_owned(),
            expected: artifact_hash.schema_version.to_string(),
            actual: summary.schema_version.to_string(),
        });
    }
    if summary.source_hash != entry.source_hash {
        return Err(ManifestError::ReferencedArtifactMismatch {
            path: file.clone(),
            field: "source_hash".to_owned(),
            expected: source_hash_string(entry.source_hash),
            actual: source_hash_string(summary.source_hash),
        });
    }
    Ok(())
}

fn validate_development_artifact_reachability(
    artifact_root: &Path,
    manifest: &ArtifactManifest,
) -> Result<(), ManifestError> {
    for entry in &manifest.development_artifacts {
        let path = PublishedArtifactPath::new(&entry.path).map_err(ManifestError::StoreIo)?;
        ensure_reachable_file(artifact_root, &path)?;
    }
    Ok(())
}

fn ensure_reachable_file(
    artifact_root: &Path,
    path: &PublishedArtifactPath,
) -> Result<(), ManifestError> {
    let root = fs::canonicalize(artifact_root).map_err(|error| {
        ManifestError::StoreIo(store_io_error(
            path.as_str(),
            StoreIoOperation::Canonicalize,
            error,
        ))
    })?;
    let mut final_path = root.clone();
    for segment in path.as_str().split('/') {
        final_path.push(segment);
    }
    let metadata = fs::symlink_metadata(&final_path).map_err(|error| {
        ManifestError::StoreIo(store_io_error(path.as_str(), StoreIoOperation::Read, error))
    })?;
    if metadata.file_type().is_symlink() {
        return Err(ManifestError::StoreIo(StoreIoError::Path {
            path: path.as_str().to_owned(),
            reason: PublishedPathError::SymlinkEscape,
        }));
    }
    if !metadata.is_file() {
        return Err(ManifestError::InvalidField {
            path: path.as_str().to_owned(),
            reason: "path must reference a file".to_owned(),
        });
    }
    let canonical = fs::canonicalize(&final_path).map_err(|error| {
        ManifestError::StoreIo(store_io_error(
            path.as_str(),
            StoreIoOperation::Canonicalize,
            error,
        ))
    })?;
    if !canonical.starts_with(&root) {
        return Err(ManifestError::StoreIo(StoreIoError::Path {
            path: path.as_str().to_owned(),
            reason: PublishedPathError::RootEscape,
        }));
    }
    Ok(())
}

fn ensure_seed_matches_current(
    seed: &ArtifactManifest,
    current: &ArtifactManifest,
) -> Result<(), ManifestError> {
    if seed.package != current.package {
        return Err(ManifestError::ManifestSeedMismatch {
            field: "package".to_owned(),
        });
    }
    if seed.artifact_root != current.artifact_root {
        return Err(ManifestError::ManifestSeedMismatch {
            field: "artifact_root".to_owned(),
        });
    }
    if seed.toolchain != current.toolchain {
        return Err(ManifestError::ManifestSeedMismatch {
            field: "toolchain".to_owned(),
        });
    }
    if seed.language_edition != current.language_edition {
        return Err(ManifestError::ManifestSeedMismatch {
            field: "language_edition".to_owned(),
        });
    }
    Ok(())
}

fn upsert_module_entry(entries: &mut Vec<ModuleArtifactEntry>, staged: ModuleArtifactEntry) {
    if let Some(existing) = entries
        .iter_mut()
        .find(|entry| entry.module == staged.module)
    {
        *existing = staged;
    } else {
        entries.push(staged);
    }
}

fn sort_manifest_entries(manifest: &mut ArtifactManifest) {
    manifest
        .modules
        .sort_by(|left, right| left.module.cmp(&right.module));
    for module in &mut manifest.modules {
        sort_proof_witnesses(&mut module.proof_witnesses);
    }
    manifest
        .development_artifacts
        .sort_by(development_entry_order);
}

fn is_missing_manifest(error: &ManifestError) -> bool {
    matches!(
        error,
        ManifestError::StoreIo(StoreIoError::Io {
            kind: io::ErrorKind::NotFound,
            ..
        })
    )
}

fn json_object<I, K>(fields: I) -> Result<CanonicalJson, ManifestError>
where
    I: IntoIterator<Item = (K, CanonicalJson)>,
    K: Into<String>,
{
    CanonicalJson::object(fields).map_err(ManifestError::CanonicalJson)
}

fn expect_object<'a>(
    value: &'a CanonicalJson,
    path: &str,
) -> Result<&'a BTreeMap<String, CanonicalJson>, ManifestError> {
    match value {
        CanonicalJson::Object(fields) => Ok(fields),
        _ => Err(ManifestError::UnexpectedType {
            path: path.to_owned(),
            expected: "an object",
        }),
    }
}

fn required_field<'a>(
    fields: &'a BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<&'a CanonicalJson, ManifestError> {
    fields
        .get(field)
        .ok_or_else(|| ManifestError::MissingField {
            path: field_path(path, field),
        })
}

fn reject_unknown_fields(
    fields: &BTreeMap<String, CanonicalJson>,
    allowed: &[&str],
    path: &str,
) -> Result<(), ManifestError> {
    for key in fields.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(ManifestError::UnknownField {
                path: path.to_owned(),
                field: key.clone(),
            });
        }
    }
    Ok(())
}

fn read_required_string(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<String, ManifestError> {
    let path = field_path(path, field);
    match required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )? {
        CanonicalJson::String(value) => Ok(value.clone()),
        _ => Err(ManifestError::UnexpectedType {
            path,
            expected: "a string",
        }),
    }
}

fn read_optional_string(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<Option<String>, ManifestError> {
    let path = field_path(path, field);
    match required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )? {
        CanonicalJson::Null => Ok(None),
        CanonicalJson::String(value) => Ok(Some(value.clone())),
        _ => Err(ManifestError::UnexpectedType {
            path,
            expected: "a string or null",
        }),
    }
}

fn read_optional_identity(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
) -> Result<Option<ModuleSummaryIdentity>, ManifestError> {
    let path = field_path(path, field);
    match required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )? {
        CanonicalJson::Null => Ok(None),
        value => read_identity(value, &path).map(Some),
    }
}

fn read_required_artifact_hash_ref(
    fields: &BTreeMap<String, CanonicalJson>,
    field: &str,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<ArtifactHashRef, ManifestError> {
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
) -> Result<Option<ArtifactHashRef>, ManifestError> {
    let path = field_path(path, field);
    match required_field(
        fields,
        field,
        path.rsplit_once('.').map_or("$", |(base, _)| base),
    )? {
        CanonicalJson::Null => Ok(None),
        value => read_artifact_hash_ref(value, &path, expected_class).map(Some),
    }
}

fn read_artifact_hash_ref(
    value: &CanonicalJson,
    path: &str,
    expected_class: ArtifactHashClass,
) -> Result<ArtifactHashRef, ManifestError> {
    let CanonicalJson::String(value) = value else {
        return Err(ManifestError::UnexpectedType {
            path: path.to_owned(),
            expected: "an artifact-framed hash string",
        });
    };
    let hash_ref = parse_artifact_hash_ref_string(value, path)?;
    validate_artifact_hash_ref(&hash_ref, path, expected_class, None, None)?;
    Ok(hash_ref)
}

fn read_source_hash(value: &CanonicalJson, path: &str) -> Result<Hash, ManifestError> {
    let CanonicalJson::String(value) = value else {
        return Err(ManifestError::UnexpectedType {
            path: path.to_owned(),
            expected: "a source hash string",
        });
    };
    parse_source_hash_string(value, path)
}

fn parse_artifact_hash_ref_string(
    value: &str,
    path: &str,
) -> Result<ArtifactHashRef, ManifestError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 5 {
        return Err(ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: "expected construction:class:schema_family:schema_version:digest".to_owned(),
        });
    }
    if parts[0] != ARTIFACT_HASH_CONSTRUCTION {
        return Err(ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong artifact hash construction label".to_owned(),
        });
    }
    let class =
        artifact_hash_class_from_str(parts[1]).ok_or_else(|| ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: "unknown artifact hash class".to_owned(),
        })?;
    validate_schema_family(parts[2], path)?;
    let schema_version =
        SchemaVersion::from_str(parts[3]).map_err(|_| ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: "malformed schema version".to_owned(),
        })?;
    let digest = parse_lower_hex_hash(parts[4], path)?;
    Ok(ArtifactHashRef::new(
        class,
        parts[2].to_owned(),
        schema_version,
        digest,
    ))
}

fn parse_source_hash_string(value: &str, path: &str) -> Result<Hash, ManifestError> {
    let Some(hex) = value
        .strip_prefix(SOURCE_HASH_CONSTRUCTION)
        .and_then(|rest| rest.strip_prefix(':'))
    else {
        return Err(ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: "wrong source hash construction label".to_owned(),
        });
    };
    parse_lower_hex_hash(hex, path)
}

fn parse_lower_hex_hash(hex: &str, path: &str) -> Result<Hash, ManifestError> {
    if hex.len() != Hash::BYTE_LEN * 2 {
        return Err(ManifestError::InvalidHash {
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

fn parse_lower_hex_nibble(byte: u8, path: &str) -> Result<u8, ManifestError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: "digest must use lowercase hexadecimal".to_owned(),
        }),
    }
}

fn validate_schema_family(value: &str, path: &str) -> Result<(), ManifestError> {
    if value.is_empty() {
        return Err(ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: "schema family must be non-empty".to_owned(),
        });
    }
    if value.contains(':') || value.chars().any(char::is_whitespace) {
        return Err(ManifestError::InvalidHash {
            path: path.to_owned(),
            reason: "schema family must not contain ':' or whitespace".to_owned(),
        });
    }
    Ok(())
}

fn source_hash_string(hash: Hash) -> String {
    format!("{}:{}", SOURCE_HASH_CONSTRUCTION, lower_hex_hash(hash))
}

fn hash_string(hash: Hash) -> String {
    lower_hex_hash(hash)
}

fn lower_hex_hash(hash: Hash) -> String {
    let mut output = String::with_capacity(Hash::BYTE_LEN * 2);
    for byte in hash.as_bytes() {
        write!(&mut output, "{byte:02x}").expect("writing to string cannot fail");
    }
    output
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

fn artifact_hash_class_str(class: ArtifactHashClass) -> &'static str {
    match class {
        ArtifactHashClass::Interface => "interface",
        ArtifactHashClass::Implementation => "implementation",
        ArtifactHashClass::Diagnostic => "diagnostic",
        ArtifactHashClass::Artifact => "artifact",
    }
}

fn sort_proof_witnesses(entries: &mut [ManifestProofWitnessEntry]) {
    entries.sort_by_key(proof_witness_order_key);
}

fn proof_witness_order_key(entry: &ManifestProofWitnessEntry) -> (String, String, String) {
    (
        entry.obligation_id.clone(),
        entry.obligation_fingerprint.to_artifact_hash_string(),
        entry.witness_path.clone(),
    )
}

fn development_entry_order(
    left: &DevelopmentArtifactEntry,
    right: &DevelopmentArtifactEntry,
) -> std::cmp::Ordering {
    development_entry_order_key(left).cmp(&development_entry_order_key(right))
}

fn development_entry_order_key(
    entry: &DevelopmentArtifactEntry,
) -> (String, String, Option<ModuleSummaryIdentity>) {
    (
        entry.kind.clone(),
        entry.path.clone(),
        entry.related_module.clone(),
    )
}

fn witness_tuple_key(
    tuple: &(String, ArtifactHashRef, String, ArtifactHashRef),
) -> (String, String, String) {
    (
        tuple.0.clone(),
        tuple.1.to_artifact_hash_string(),
        tuple.2.clone(),
    )
}

fn identity_display(identity: &ModuleSummaryIdentity) -> String {
    format!(
        "{}:{}:{}",
        identity.package_id, identity.module_path, identity.language_edition
    )
}

fn optional_hash_display(value: Option<&str>) -> &str {
    value.unwrap_or("<none>")
}

fn field_path(path: &str, field: &str) -> String {
    if path == "$" {
        format!("$.{field}")
    } else {
        format!("{path}.{field}")
    }
}

fn array_path(path: &str, index: usize) -> String {
    format!("{path}[{index}]")
}

fn store_io_error(path: &str, operation: StoreIoOperation, error: io::Error) -> StoreIoError {
    StoreIoError::Io {
        path: path.to_owned(),
        operation,
        kind: error.kind(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests;
