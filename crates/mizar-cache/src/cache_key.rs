//! Canonical internal cache key construction.
//!
//! The pure-projection contract is specified in the
//! [cache-key design spec](../../../doc/design/mizar-cache/en/cache_key.md).

use std::{collections::BTreeMap, error::Error, fmt};

use mizar_session::{Edition, Hash, ModulePath, NormalizedPath, PackageId};

/// Current cache-key schema version.
pub const CACHE_KEY_SCHEMA_VERSION: &str = "mizar-cache/cache-key-schema/v1";
/// Domain used for final cache-key hashes.
pub const CACHE_KEY_HASH_DOMAIN: &str = "mizar-cache/cache-key/v1";

/// A versioned pipeline phase for a cache key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PipelinePhase(String);

/// Phase-local unit identity for a cache key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkUnit(String);

/// Cache/schema version string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion(String);

/// Build/proof policy fingerprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PolicyFingerprint(Hash);

/// Named hash with a domain-separated digest.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedHash {
    /// Stable name within the containing collection.
    pub name: String,
    /// Hash domain.
    pub domain: String,
    /// Hash digest.
    pub digest: Hash,
}

/// Hash of a dependency artifact or dependency-facing summary.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyHash {
    /// Dependency class such as manifest, interface, implementation, or lockfile.
    pub dependency_kind: String,
    /// Dependency package id.
    pub package_id: String,
    /// Dependency module path when applicable.
    pub module_path: String,
    /// Dependency-local name.
    pub name: String,
    /// Hash domain.
    pub domain: String,
    /// Hash digest.
    pub digest: Hash,
}

/// Hash of a dependency slice.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencySliceHash {
    /// Slice class.
    pub slice_kind: String,
    /// Slice owner identity.
    pub owner: String,
    /// Slice-local name.
    pub name: String,
    /// Hash domain.
    pub domain: String,
    /// Hash digest.
    pub digest: Hash,
}

/// Named schema version that affects key interpretation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedSchemaVersion {
    /// Schema family.
    pub schema_family: String,
    /// Schema-local name.
    pub name: String,
    /// Version string.
    pub version: SchemaVersion,
}

/// Source identity for source-backed work units.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceIdentity {
    /// Package id.
    pub package_id: PackageId,
    /// Module path.
    pub module_path: ModulePath,
    /// Normalized source path.
    pub normalized_source_path: NormalizedPath,
    /// Source content hash.
    pub source_hash: Hash,
    /// Language edition.
    pub language_edition: Edition,
}

/// Completeness state of the dependency footprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum FootprintCompleteness {
    /// All required dependency information is available.
    Complete,
    /// The work unit cannot be cached because dependency information is missing.
    IncompleteUncacheable,
}

/// Dependency artifact availability validation input.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyArtifactAvailability {
    /// Package id.
    pub package_id: String,
    /// Module path.
    pub module_path: String,
    /// Artifact kind.
    pub artifact_kind: String,
    /// Artifact path or stable manifest entry key.
    pub artifact_path: String,
    /// Hash domain.
    pub domain: String,
    /// Expected hash digest.
    pub digest: Hash,
}

/// Named compatibility field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CompatibilityField {
    /// Compatibility family or component.
    pub family: String,
    /// Field name.
    pub field_name: String,
    /// Field value.
    pub value: String,
}

/// Proof-reuse evidence identity used for validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProofReuseEvidenceIdentity {
    /// Obligation anchor fingerprint.
    pub obligation_anchor_fingerprint: Hash,
    /// Evidence kind.
    pub evidence_kind: String,
    /// Witness or deterministic-discharge hash domain.
    pub witness_or_discharge_domain: String,
    /// Witness or deterministic-discharge hash digest.
    pub witness_or_discharge_digest: Hash,
}

/// Diagnostic reference used only for miss explanations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticRefHash {
    /// Diagnostic reference kind.
    pub diagnostic_ref_kind: String,
    /// Diagnostic reference hash.
    pub diagnostic_ref_hash: Hash,
}

/// Cache validation inputs that must be compared after a key match.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheValidationInputs {
    /// Cache schema compatibility token.
    pub cache_schema_compatibility: SchemaVersion,
    /// Toolchain compatibility fields.
    pub toolchain_compatibility: Vec<CompatibilityField>,
    /// Dependency artifacts that must exist with matching hashes.
    pub dependency_artifacts: Vec<DependencyArtifactAvailability>,
    /// Dependency footprint completeness.
    pub footprint_completeness: FootprintCompleteness,
    /// Explicit uncacheable marker.
    pub uncacheable: bool,
    /// Verifier/proof policy compatibility fields.
    pub policy_compatibility: Vec<CompatibilityField>,
    /// Canonical VC fingerprint when applicable.
    pub canonical_vc_fingerprint: Option<Hash>,
    /// Local-context fingerprint when applicable.
    pub local_context_fingerprint: Option<Hash>,
    /// Dependency-slice validation fingerprints.
    pub dependency_slice_fingerprints: Vec<DependencySliceHash>,
    /// Obligation anchor fingerprint when applicable.
    pub obligation_anchor_fingerprint: Option<Hash>,
    /// Selected proof witness hash for trusted witness reuse.
    pub selected_proof_witness_hash: Option<NamedHash>,
    /// Deterministic discharge hash for built-in discharge reuse.
    pub deterministic_discharge_hash: Option<NamedHash>,
    /// Proof-reuse metadata schema versions.
    pub proof_reuse_schema_versions: Vec<NamedSchemaVersion>,
    /// Stable validation hash exported by proof metadata.
    pub proof_reuse_validation_hash: Option<NamedHash>,
    /// Class-aware proof evidence identities.
    pub proof_reuse_evidence_identities: Vec<ProofReuseEvidenceIdentity>,
    /// Diagnostic-only refs used for miss explanations.
    pub diagnostic_refs: Vec<DiagnosticRefHash>,
}

/// Complete cache key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// Cache key and record schema version.
    pub cache_schema_version: SchemaVersion,
    /// Pipeline phase.
    pub phase: PipelinePhase,
    /// Phase-local work unit.
    pub work_unit: WorkUnit,
    /// Optional source identity.
    pub source_identity: Option<SourceIdentity>,
    /// Direct input hashes.
    pub input_hashes: Vec<NamedHash>,
    /// Dependency hashes.
    pub dependency_hashes: Vec<DependencyHash>,
    /// Dependency slice hashes.
    pub dependency_slices: Vec<DependencySliceHash>,
    /// Build/verifier config hash.
    pub config_hash: Hash,
    /// Schema versions affecting interpretation.
    pub schema_versions: Vec<NamedSchemaVersion>,
    /// Active policy fingerprint.
    pub policy_fingerprint: PolicyFingerprint,
    /// Validation inputs for lookup/reuse.
    pub validation_inputs: CacheValidationInputs,
    /// Domain-separated final hash.
    pub final_hash: Hash,
}

/// Request consumed by `CacheKeyBuilder`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKeyRequest {
    /// Cache key schema version.
    pub cache_schema_version: SchemaVersion,
    /// Pipeline phase.
    pub phase: PipelinePhase,
    /// Phase-local work unit.
    pub work_unit: WorkUnit,
    /// Optional source identity.
    pub source_identity: Option<SourceIdentity>,
    /// Direct input hashes.
    pub input_hashes: Vec<NamedHash>,
    /// Dependency hashes.
    pub dependency_hashes: Vec<DependencyHash>,
    /// Dependency slice hashes.
    pub dependency_slices: Vec<DependencySliceHash>,
    /// Build/verifier config hash.
    pub config_hash: Hash,
    /// Schema versions affecting interpretation.
    pub schema_versions: Vec<NamedSchemaVersion>,
    /// Active policy fingerprint.
    pub policy_fingerprint: PolicyFingerprint,
    /// Validation inputs for lookup/reuse.
    pub validation_inputs: CacheValidationInputs,
}

/// Pure cache-key builder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKeyBuilder {
    request: CacheKeyRequest,
}

/// Result of cache-key construction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheKeyBuildOutcome {
    /// A cacheable key.
    Cacheable(CacheKey),
    /// A canonical key that must be treated as a miss.
    Uncacheable(CacheKey),
    /// No key can be produced.
    NoKey(CacheKeyBuildRejection),
}

/// Reasons key construction can reject a request.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheKeyBuildRejection {
    /// The request used an unsupported cache-key schema.
    UnsupportedCacheKeySchema {
        /// Actual schema.
        actual: SchemaVersion,
    },
    /// A canonical collection key appeared with conflicting payloads.
    ConflictingDuplicate {
        /// Collection name.
        collection: &'static str,
        /// Canonical key display.
        key: String,
    },
    /// A proof-related reuse hash was incomplete.
    IncompleteProofReuseEvidence {
        /// Field name.
        field: &'static str,
    },
    /// A required identity field was empty.
    MissingRequiredIdentity {
        /// Field name.
        field: &'static str,
    },
}

impl PipelinePhase {
    /// Creates a pipeline phase identifier.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the phase string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl WorkUnit {
    /// Creates a work-unit identifier.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the work-unit string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl SchemaVersion {
    /// Creates a schema version string.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the schema version string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::new(CACHE_KEY_SCHEMA_VERSION)
    }
}

impl PolicyFingerprint {
    /// Creates a policy fingerprint.
    pub const fn new(hash: Hash) -> Self {
        Self(hash)
    }

    /// Returns the fingerprint hash.
    pub const fn hash(self) -> Hash {
        self.0
    }
}

impl CacheKeyBuilder {
    /// Creates a cache-key builder from a complete request.
    pub fn new(request: CacheKeyRequest) -> Self {
        Self { request }
    }

    /// Builds a cache-key outcome.
    pub fn build(self) -> CacheKeyBuildOutcome {
        let mut request = self.request;
        if request.cache_schema_version.as_str() != CACHE_KEY_SCHEMA_VERSION {
            return CacheKeyBuildOutcome::NoKey(
                CacheKeyBuildRejection::UnsupportedCacheKeySchema {
                    actual: request.cache_schema_version,
                },
            );
        }

        if let Err(rejection) =
            validate_required_identities(&request).and_then(|()| canonicalize_request(&mut request))
        {
            return CacheKeyBuildOutcome::NoKey(rejection);
        }

        if request.validation_inputs.footprint_completeness
            == FootprintCompleteness::IncompleteUncacheable
        {
            request.validation_inputs.uncacheable = true;
        }
        if has_missing_required_validation_input(&request) {
            request.validation_inputs.uncacheable = true;
        }

        let uncacheable = request.validation_inputs.uncacheable;
        let final_hash = final_hash_for_request(&request);

        let key = CacheKey {
            cache_schema_version: request.cache_schema_version,
            phase: request.phase,
            work_unit: request.work_unit,
            source_identity: request.source_identity,
            input_hashes: request.input_hashes,
            dependency_hashes: request.dependency_hashes,
            dependency_slices: request.dependency_slices,
            config_hash: request.config_hash,
            schema_versions: request.schema_versions,
            policy_fingerprint: request.policy_fingerprint,
            validation_inputs: request.validation_inputs,
            final_hash,
        };

        if uncacheable {
            CacheKeyBuildOutcome::Uncacheable(key)
        } else {
            CacheKeyBuildOutcome::Cacheable(key)
        }
    }
}

impl fmt::Display for CacheKeyBuildRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedCacheKeySchema { actual } => {
                write!(f, "unsupported cache key schema `{}`", actual.as_str())
            }
            Self::ConflictingDuplicate { collection, key } => {
                write!(f, "conflicting duplicate in {collection} for `{key}`")
            }
            Self::IncompleteProofReuseEvidence { field } => {
                write!(f, "incomplete proof reuse evidence field `{field}`")
            }
            Self::MissingRequiredIdentity { field } => {
                write!(f, "missing required cache key identity field `{field}`")
            }
        }
    }
}

impl Error for CacheKeyBuildRejection {}

fn validate_required_identities(request: &CacheKeyRequest) -> Result<(), CacheKeyBuildRejection> {
    reject_empty("phase", request.phase.as_str())?;
    reject_empty("work_unit", request.work_unit.as_str())?;
    reject_empty(
        "validation_inputs.cache_schema_compatibility",
        request
            .validation_inputs
            .cache_schema_compatibility
            .as_str(),
    )?;

    if let Some(source) = &request.source_identity {
        reject_empty("source_identity.package_id", source.package_id.as_str())?;
        reject_empty("source_identity.module_path", source.module_path.as_str())?;
        reject_empty(
            "source_identity.normalized_source_path",
            source.normalized_source_path.as_str(),
        )?;
        reject_empty(
            "source_identity.language_edition",
            source.language_edition.as_str(),
        )?;
    }

    for value in &request.input_hashes {
        reject_empty("input_hashes.name", &value.name)?;
        reject_empty("input_hashes.domain", &value.domain)?;
    }
    for value in &request.dependency_hashes {
        reject_empty("dependency_hashes.dependency_kind", &value.dependency_kind)?;
        reject_empty("dependency_hashes.package_id", &value.package_id)?;
        reject_empty("dependency_hashes.name", &value.name)?;
        reject_empty("dependency_hashes.domain", &value.domain)?;
    }
    for value in &request.dependency_slices {
        validate_dependency_slice_identity("dependency_slices", value)?;
    }
    for value in &request.schema_versions {
        validate_schema_version_identity("schema_versions", value)?;
    }
    validate_validation_input_identities(&request.validation_inputs)?;
    Ok(())
}

fn validate_validation_input_identities(
    validation: &CacheValidationInputs,
) -> Result<(), CacheKeyBuildRejection> {
    for value in &validation.toolchain_compatibility {
        validate_compatibility_field_identity("toolchain_compatibility", value)?;
    }
    for value in &validation.dependency_artifacts {
        reject_empty("dependency_artifacts.package_id", &value.package_id)?;
        reject_empty("dependency_artifacts.artifact_kind", &value.artifact_kind)?;
        reject_empty("dependency_artifacts.artifact_path", &value.artifact_path)?;
        reject_empty("dependency_artifacts.domain", &value.domain)?;
    }
    for value in &validation.policy_compatibility {
        validate_compatibility_field_identity("policy_compatibility", value)?;
    }
    for value in &validation.dependency_slice_fingerprints {
        validate_dependency_slice_identity("dependency_slice_fingerprints", value)?;
    }
    if let Some(value) = &validation.selected_proof_witness_hash {
        reject_empty("selected_proof_witness_hash.name", &value.name)?;
        reject_empty("selected_proof_witness_hash.domain", &value.domain)?;
    }
    if let Some(value) = &validation.deterministic_discharge_hash {
        reject_empty("deterministic_discharge_hash.name", &value.name)?;
        reject_empty("deterministic_discharge_hash.domain", &value.domain)?;
    }
    for value in &validation.proof_reuse_schema_versions {
        validate_schema_version_identity("proof_reuse_schema_versions", value)?;
    }
    if let Some(value) = &validation.proof_reuse_validation_hash {
        reject_empty("proof_reuse_validation_hash.name", &value.name)?;
        reject_empty("proof_reuse_validation_hash.domain", &value.domain)?;
    }
    for value in &validation.proof_reuse_evidence_identities {
        reject_empty(
            "proof_reuse_evidence_identities.evidence_kind",
            &value.evidence_kind,
        )?;
        reject_empty(
            "proof_reuse_evidence_identities.witness_or_discharge_domain",
            &value.witness_or_discharge_domain,
        )?;
    }
    for value in &validation.diagnostic_refs {
        reject_empty(
            "diagnostic_refs.diagnostic_ref_kind",
            &value.diagnostic_ref_kind,
        )?;
    }
    Ok(())
}

fn validate_dependency_slice_identity(
    field: &'static str,
    value: &DependencySliceHash,
) -> Result<(), CacheKeyBuildRejection> {
    reject_empty(field, &value.slice_kind)?;
    reject_empty(field, &value.owner)?;
    reject_empty(field, &value.name)?;
    reject_empty(field, &value.domain)
}

fn validate_schema_version_identity(
    field: &'static str,
    value: &NamedSchemaVersion,
) -> Result<(), CacheKeyBuildRejection> {
    reject_empty(field, &value.schema_family)?;
    reject_empty(field, &value.name)?;
    reject_empty(field, value.version.as_str())
}

fn validate_compatibility_field_identity(
    field: &'static str,
    value: &CompatibilityField,
) -> Result<(), CacheKeyBuildRejection> {
    reject_empty(field, &value.family)?;
    reject_empty(field, &value.field_name)
}

fn reject_empty(field: &'static str, value: &str) -> Result<(), CacheKeyBuildRejection> {
    if value.trim().is_empty() {
        return Err(CacheKeyBuildRejection::MissingRequiredIdentity { field });
    }
    Ok(())
}

fn has_missing_required_validation_input(request: &CacheKeyRequest) -> bool {
    let validation = &request.validation_inputs;
    let proof_reuse_related = is_proof_reuse_related(validation);
    let vc_related = matches!(request.phase.as_str(), "vc" | "proof" | "atp")
        || proof_reuse_related
        || validation.canonical_vc_fingerprint.is_some()
        || validation.local_context_fingerprint.is_some()
        || !validation.dependency_slice_fingerprints.is_empty();

    validation.toolchain_compatibility.is_empty()
        || validation.policy_compatibility.is_empty()
        || request.schema_versions.is_empty()
        || (vc_related
            && (validation.canonical_vc_fingerprint.is_none()
                || validation.local_context_fingerprint.is_none()
                || validation.dependency_slice_fingerprints.is_empty()))
        || (proof_reuse_related
            && (validation.obligation_anchor_fingerprint.is_none()
                || validation.proof_reuse_schema_versions.is_empty()
                || validation.proof_reuse_validation_hash.is_none()
                || proof_evidence_requires_missing_hash(validation)))
}

fn is_proof_reuse_related(validation: &CacheValidationInputs) -> bool {
    !validation.proof_reuse_evidence_identities.is_empty()
        || validation.selected_proof_witness_hash.is_some()
        || validation.deterministic_discharge_hash.is_some()
        || validation.proof_reuse_validation_hash.is_some()
        || !validation.proof_reuse_schema_versions.is_empty()
}

fn proof_evidence_requires_missing_hash(validation: &CacheValidationInputs) -> bool {
    validation
        .proof_reuse_evidence_identities
        .iter()
        .any(
            |identity| match proof_evidence_kind(&identity.evidence_kind) {
                ProofEvidenceKind::KernelVerified => {
                    validation.selected_proof_witness_hash.is_none()
                }
                ProofEvidenceKind::DischargedBuiltin => {
                    validation.deterministic_discharge_hash.is_none()
                }
                ProofEvidenceKind::Unknown => true,
            },
        )
}

fn proof_evidence_kind(value: &str) -> ProofEvidenceKind {
    match value {
        "kernel_verified" | "KernelVerified" | "kernel-verified" => {
            ProofEvidenceKind::KernelVerified
        }
        "discharged_builtin" | "DischargedBuiltin" | "discharged-builtin" => {
            ProofEvidenceKind::DischargedBuiltin
        }
        _ => ProofEvidenceKind::Unknown,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProofEvidenceKind {
    KernelVerified,
    DischargedBuiltin,
    Unknown,
}

fn canonicalize_request(request: &mut CacheKeyRequest) -> Result<(), CacheKeyBuildRejection> {
    canonicalize_by_keys(
        &mut request.input_hashes,
        "input_hashes",
        NamedHash::duplicate_identity_key,
        NamedHash::canonical_sort_key,
    )?;
    canonicalize_by_keys(
        &mut request.dependency_hashes,
        "dependency_hashes",
        DependencyHash::duplicate_identity_key,
        DependencyHash::canonical_sort_key,
    )?;
    canonicalize_by_keys(
        &mut request.dependency_slices,
        "dependency_slices",
        DependencySliceHash::duplicate_identity_key,
        DependencySliceHash::canonical_sort_key,
    )?;
    canonicalize_by_keys(
        &mut request.schema_versions,
        "schema_versions",
        NamedSchemaVersion::duplicate_identity_key,
        NamedSchemaVersion::canonical_sort_key,
    )?;
    canonicalize_validation_inputs(&mut request.validation_inputs)?;
    Ok(())
}

fn canonicalize_validation_inputs(
    validation: &mut CacheValidationInputs,
) -> Result<(), CacheKeyBuildRejection> {
    canonicalize_by_keys(
        &mut validation.toolchain_compatibility,
        "toolchain_compatibility",
        CompatibilityField::duplicate_identity_key,
        CompatibilityField::canonical_sort_key,
    )?;
    canonicalize_by_keys(
        &mut validation.dependency_artifacts,
        "dependency_artifacts",
        DependencyArtifactAvailability::duplicate_identity_key,
        DependencyArtifactAvailability::canonical_sort_key,
    )?;
    canonicalize_by_keys(
        &mut validation.policy_compatibility,
        "policy_compatibility",
        CompatibilityField::duplicate_identity_key,
        CompatibilityField::canonical_sort_key,
    )?;
    canonicalize_by_keys(
        &mut validation.dependency_slice_fingerprints,
        "dependency_slice_fingerprints",
        DependencySliceHash::duplicate_identity_key,
        DependencySliceHash::canonical_sort_key,
    )?;
    canonicalize_by_keys(
        &mut validation.proof_reuse_schema_versions,
        "proof_reuse_schema_versions",
        NamedSchemaVersion::duplicate_identity_key,
        NamedSchemaVersion::canonical_sort_key,
    )?;
    canonicalize_by_keys(
        &mut validation.proof_reuse_evidence_identities,
        "proof_reuse_evidence_identities",
        ProofReuseEvidenceIdentity::duplicate_identity_key,
        ProofReuseEvidenceIdentity::canonical_sort_key,
    )?;
    canonicalize_by_keys(
        &mut validation.diagnostic_refs,
        "diagnostic_refs",
        DiagnosticRefHash::duplicate_identity_key,
        DiagnosticRefHash::canonical_sort_key,
    )?;

    Ok(())
}

fn canonicalize_by_keys<T, I, S>(
    values: &mut Vec<T>,
    collection: &'static str,
    identity_key_fn: impl Fn(&T) -> I,
    sort_key_fn: impl Fn(&T) -> S,
) -> Result<(), CacheKeyBuildRejection>
where
    T: Clone + Eq,
    I: Ord + fmt::Debug,
    S: Ord,
{
    let mut by_key = BTreeMap::<I, T>::new();
    for value in values.drain(..) {
        let key = identity_key_fn(&value);
        match by_key.get(&key) {
            Some(existing) if existing != &value => {
                return Err(CacheKeyBuildRejection::ConflictingDuplicate {
                    collection,
                    key: format!("{key:?}"),
                });
            }
            Some(_) => {}
            None => {
                by_key.insert(key, value);
            }
        }
    }
    values.extend(by_key.into_values());
    values.sort_by_key(sort_key_fn);
    Ok(())
}

fn final_hash_for_request(request: &CacheKeyRequest) -> Hash {
    let mut hasher = stable_hasher("final");
    write_schema_version_field(
        &mut hasher,
        "cache_schema_version",
        &request.cache_schema_version,
    );
    write_str_field(&mut hasher, "phase", request.phase.as_str());
    write_str_field(&mut hasher, "work_unit", request.work_unit.as_str());
    write_source_identity(&mut hasher, request.source_identity.as_ref());
    write_named_hashes(&mut hasher, "input_hashes", &request.input_hashes);
    write_dependency_hashes(&mut hasher, "dependency_hashes", &request.dependency_hashes);
    write_dependency_slice_hashes(&mut hasher, "dependency_slices", &request.dependency_slices);
    write_hash_field(
        &mut hasher,
        "config_hash",
        "mizar-cache/cache-key/config-hash/v1",
        request.config_hash,
    );
    write_schema_versions(&mut hasher, "schema_versions", &request.schema_versions);
    write_hash_field(
        &mut hasher,
        "policy_fingerprint",
        "mizar-cache/cache-key/policy-fingerprint/v1",
        request.policy_fingerprint.hash(),
    );
    write_validation_inputs(&mut hasher, &request.validation_inputs);
    finish_hash(hasher)
}

fn stable_hasher(label: &str) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    write_str(&mut hasher, CACHE_KEY_HASH_DOMAIN);
    write_str(&mut hasher, label);
    hasher
}

fn write_source_identity(hasher: &mut blake3::Hasher, source: Option<&SourceIdentity>) {
    write_field_tag(hasher, "source_identity");
    match source {
        Some(source) => {
            hasher.update(&[1]);
            write_str_field(hasher, "package_id", source.package_id.as_str());
            write_str_field(hasher, "module_path", source.module_path.as_str());
            write_str_field(
                hasher,
                "normalized_source_path",
                source.normalized_source_path.as_str(),
            );
            write_hash_field(
                hasher,
                "source_hash",
                "mizar-cache/cache-key/source-hash/v1",
                source.source_hash,
            );
            write_str_field(hasher, "language_edition", source.language_edition.as_str());
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

fn write_named_hashes(hasher: &mut blake3::Hasher, field: &str, values: &[NamedHash]) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_named_hash(hasher, value);
    }
}

fn write_named_hash(hasher: &mut blake3::Hasher, value: &NamedHash) {
    write_field_tag(hasher, "NamedHash");
    write_str_field(hasher, "name", &value.name);
    write_str_field(hasher, "domain", &value.domain);
    write_hash_field(hasher, "digest", &value.domain, value.digest);
}

fn write_dependency_hashes(hasher: &mut blake3::Hasher, field: &str, values: &[DependencyHash]) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "DependencyHash");
        write_str_field(hasher, "dependency_kind", &value.dependency_kind);
        write_str_field(hasher, "package_id", &value.package_id);
        write_str_field(hasher, "module_path", &value.module_path);
        write_str_field(hasher, "name", &value.name);
        write_str_field(hasher, "domain", &value.domain);
        write_hash_field(hasher, "digest", &value.domain, value.digest);
    }
}

fn write_dependency_slice_hashes(
    hasher: &mut blake3::Hasher,
    field: &str,
    values: &[DependencySliceHash],
) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_dependency_slice_hash(hasher, value);
    }
}

fn write_dependency_slice_hash(hasher: &mut blake3::Hasher, value: &DependencySliceHash) {
    write_field_tag(hasher, "DependencySliceHash");
    write_str_field(hasher, "slice_kind", &value.slice_kind);
    write_str_field(hasher, "owner", &value.owner);
    write_str_field(hasher, "name", &value.name);
    write_str_field(hasher, "domain", &value.domain);
    write_hash_field(hasher, "digest", &value.domain, value.digest);
}

fn write_schema_versions(hasher: &mut blake3::Hasher, field: &str, values: &[NamedSchemaVersion]) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "NamedSchemaVersion");
        write_str_field(hasher, "schema_family", &value.schema_family);
        write_str_field(hasher, "name", &value.name);
        write_schema_version_field(hasher, "version", &value.version);
    }
}

fn write_schema_version_field(hasher: &mut blake3::Hasher, field: &str, value: &SchemaVersion) {
    write_field_tag(hasher, field);
    write_str(hasher, value.as_str());
}

fn write_validation_inputs(hasher: &mut blake3::Hasher, value: &CacheValidationInputs) {
    write_field_tag(hasher, "validation_inputs");
    write_schema_version_field(
        hasher,
        "cache_schema_compatibility",
        &value.cache_schema_compatibility,
    );
    write_compatibility_fields(
        hasher,
        "toolchain_compatibility",
        &value.toolchain_compatibility,
    );
    write_dependency_artifacts(hasher, "dependency_artifacts", &value.dependency_artifacts);
    write_str_field(
        hasher,
        "footprint_completeness",
        match value.footprint_completeness {
            FootprintCompleteness::Complete => "complete",
            FootprintCompleteness::IncompleteUncacheable => "incomplete_uncacheable",
        },
    );
    write_field_tag(hasher, "uncacheable");
    hasher.update(&[u8::from(value.uncacheable)]);
    write_compatibility_fields(hasher, "policy_compatibility", &value.policy_compatibility);
    write_optional_hash(
        hasher,
        "canonical_vc_fingerprint",
        "mizar-cache/cache-key/canonical-vc-fingerprint/v1",
        value.canonical_vc_fingerprint,
    );
    write_optional_hash(
        hasher,
        "local_context_fingerprint",
        "mizar-cache/cache-key/local-context-fingerprint/v1",
        value.local_context_fingerprint,
    );
    write_dependency_slice_hashes(
        hasher,
        "dependency_slice_fingerprints",
        &value.dependency_slice_fingerprints,
    );
    write_optional_hash(
        hasher,
        "obligation_anchor_fingerprint",
        "mizar-cache/cache-key/obligation-anchor-fingerprint/v1",
        value.obligation_anchor_fingerprint,
    );
    write_optional_named_hash(
        hasher,
        "selected_proof_witness_hash",
        value.selected_proof_witness_hash.as_ref(),
    );
    write_optional_named_hash(
        hasher,
        "deterministic_discharge_hash",
        value.deterministic_discharge_hash.as_ref(),
    );
    write_schema_versions(
        hasher,
        "proof_reuse_schema_versions",
        &value.proof_reuse_schema_versions,
    );
    write_optional_named_hash(
        hasher,
        "proof_reuse_validation_hash",
        value.proof_reuse_validation_hash.as_ref(),
    );
    write_proof_reuse_evidence_identities(
        hasher,
        "proof_reuse_evidence_identities",
        &value.proof_reuse_evidence_identities,
    );
    write_diagnostic_refs(hasher, "diagnostic_refs", &value.diagnostic_refs);
}

fn write_compatibility_fields(
    hasher: &mut blake3::Hasher,
    field: &str,
    values: &[CompatibilityField],
) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "CompatibilityField");
        write_str_field(hasher, "family", &value.family);
        write_str_field(hasher, "field_name", &value.field_name);
        write_str_field(hasher, "value", &value.value);
    }
}

fn write_dependency_artifacts(
    hasher: &mut blake3::Hasher,
    field: &str,
    values: &[DependencyArtifactAvailability],
) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "DependencyArtifactAvailability");
        write_str_field(hasher, "package_id", &value.package_id);
        write_str_field(hasher, "module_path", &value.module_path);
        write_str_field(hasher, "artifact_kind", &value.artifact_kind);
        write_str_field(hasher, "artifact_path", &value.artifact_path);
        write_str_field(hasher, "domain", &value.domain);
        write_hash_field(hasher, "digest", &value.domain, value.digest);
    }
}

fn write_proof_reuse_evidence_identities(
    hasher: &mut blake3::Hasher,
    field: &str,
    values: &[ProofReuseEvidenceIdentity],
) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "ProofReuseEvidenceIdentity");
        write_hash_field(
            hasher,
            "obligation_anchor_fingerprint",
            "mizar-cache/cache-key/proof-evidence-obligation-anchor/v1",
            value.obligation_anchor_fingerprint,
        );
        write_str_field(hasher, "evidence_kind", &value.evidence_kind);
        write_str_field(
            hasher,
            "witness_or_discharge_domain",
            &value.witness_or_discharge_domain,
        );
        write_hash_field(
            hasher,
            "witness_or_discharge_digest",
            &value.witness_or_discharge_domain,
            value.witness_or_discharge_digest,
        );
    }
}

fn write_diagnostic_refs(hasher: &mut blake3::Hasher, field: &str, values: &[DiagnosticRefHash]) {
    write_field_tag(hasher, field);
    write_len(hasher, values.len());
    for value in values {
        write_field_tag(hasher, "DiagnosticRefHash");
        write_str_field(hasher, "diagnostic_ref_kind", &value.diagnostic_ref_kind);
        write_hash_field(
            hasher,
            "diagnostic_ref_hash",
            "mizar-cache/cache-key/diagnostic-ref/v1",
            value.diagnostic_ref_hash,
        );
    }
}

fn write_optional_named_hash(hasher: &mut blake3::Hasher, field: &str, value: Option<&NamedHash>) {
    write_field_tag(hasher, field);
    match value {
        Some(value) => {
            hasher.update(&[1]);
            write_named_hash(hasher, value);
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

fn write_optional_hash(
    hasher: &mut blake3::Hasher,
    field: &str,
    domain: &str,
    value: Option<Hash>,
) {
    write_field_tag(hasher, field);
    match value {
        Some(value) => {
            hasher.update(&[1]);
            write_hash(hasher, domain, value);
        }
        None => {
            hasher.update(&[0]);
        }
    }
}

fn write_str_field(hasher: &mut blake3::Hasher, field: &str, value: &str) {
    write_field_tag(hasher, field);
    write_str(hasher, value);
}

fn write_hash_field(hasher: &mut blake3::Hasher, field: &str, domain: &str, value: Hash) {
    write_field_tag(hasher, field);
    write_hash(hasher, domain, value);
}

fn write_field_tag(hasher: &mut blake3::Hasher, field: &str) {
    write_str(hasher, field);
}

fn write_str(hasher: &mut blake3::Hasher, value: &str) {
    write_len(hasher, value.len());
    hasher.update(value.as_bytes());
}

fn write_hash(hasher: &mut blake3::Hasher, domain: &str, value: Hash) {
    write_str(hasher, domain);
    hasher.update(value.as_bytes());
}

fn write_len(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

fn finish_hash(hasher: blake3::Hasher) -> Hash {
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

impl NamedHash {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        canonical_key_parts([self.name.as_bytes(), self.domain.as_bytes()])
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.name.as_bytes(),
            self.domain.as_bytes(),
            self.digest.as_bytes(),
        ])
    }
}

impl DependencyHash {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.dependency_kind.as_bytes(),
            self.package_id.as_bytes(),
            self.module_path.as_bytes(),
            self.name.as_bytes(),
            self.domain.as_bytes(),
        ])
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.dependency_kind.as_bytes(),
            self.package_id.as_bytes(),
            self.module_path.as_bytes(),
            self.name.as_bytes(),
            self.domain.as_bytes(),
            self.digest.as_bytes(),
        ])
    }
}

impl DependencySliceHash {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.slice_kind.as_bytes(),
            self.owner.as_bytes(),
            self.name.as_bytes(),
            self.domain.as_bytes(),
        ])
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.slice_kind.as_bytes(),
            self.owner.as_bytes(),
            self.name.as_bytes(),
            self.domain.as_bytes(),
            self.digest.as_bytes(),
        ])
    }
}

impl NamedSchemaVersion {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        canonical_key_parts([self.schema_family.as_bytes(), self.name.as_bytes()])
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.schema_family.as_bytes(),
            self.name.as_bytes(),
            self.version.as_str().as_bytes(),
        ])
    }
}

impl CompatibilityField {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        canonical_key_parts([self.family.as_bytes(), self.field_name.as_bytes()])
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.family.as_bytes(),
            self.field_name.as_bytes(),
            self.value.as_bytes(),
        ])
    }
}

impl DependencyArtifactAvailability {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.package_id.as_bytes(),
            self.module_path.as_bytes(),
            self.artifact_kind.as_bytes(),
            self.artifact_path.as_bytes(),
            self.domain.as_bytes(),
        ])
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.package_id.as_bytes(),
            self.module_path.as_bytes(),
            self.artifact_kind.as_bytes(),
            self.artifact_path.as_bytes(),
            self.domain.as_bytes(),
            self.digest.as_bytes(),
        ])
    }
}

impl ProofReuseEvidenceIdentity {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.obligation_anchor_fingerprint.as_bytes(),
            self.evidence_kind.as_bytes(),
            self.witness_or_discharge_domain.as_bytes(),
        ])
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.obligation_anchor_fingerprint.as_bytes(),
            self.evidence_kind.as_bytes(),
            self.witness_or_discharge_domain.as_bytes(),
            self.witness_or_discharge_digest.as_bytes(),
        ])
    }
}

impl DiagnosticRefHash {
    fn duplicate_identity_key(&self) -> Vec<u8> {
        self.canonical_sort_key()
    }

    fn canonical_sort_key(&self) -> Vec<u8> {
        canonical_key_parts([
            self.diagnostic_ref_kind.as_bytes(),
            self.diagnostic_ref_hash.as_bytes(),
        ])
    }
}

fn canonical_key_parts<const N: usize>(parts: [&[u8]; N]) -> Vec<u8> {
    let mut key = Vec::new();
    for part in parts {
        key.extend_from_slice(&(part.len() as u64).to_le_bytes());
        key.extend_from_slice(part);
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn key_builder_is_deterministic_and_sorts_all_vectors() {
        let mut first = request();
        first.input_hashes.push(first.input_hashes[0].clone());
        first
            .dependency_hashes
            .push(first.dependency_hashes[0].clone());
        first
            .dependency_slices
            .push(first.dependency_slices[0].clone());
        first.schema_versions.push(first.schema_versions[0].clone());
        first
            .validation_inputs
            .toolchain_compatibility
            .push(first.validation_inputs.toolchain_compatibility[0].clone());
        first
            .validation_inputs
            .dependency_artifacts
            .push(first.validation_inputs.dependency_artifacts[0].clone());
        first
            .validation_inputs
            .policy_compatibility
            .push(first.validation_inputs.policy_compatibility[0].clone());
        first
            .validation_inputs
            .dependency_slice_fingerprints
            .push(first.validation_inputs.dependency_slice_fingerprints[0].clone());
        first
            .validation_inputs
            .proof_reuse_schema_versions
            .push(first.validation_inputs.proof_reuse_schema_versions[0].clone());
        first
            .validation_inputs
            .proof_reuse_evidence_identities
            .push(first.validation_inputs.proof_reuse_evidence_identities[0].clone());
        first
            .validation_inputs
            .diagnostic_refs
            .push(first.validation_inputs.diagnostic_refs[0].clone());
        first.input_hashes.reverse();
        first.dependency_hashes.reverse();
        first.dependency_slices.reverse();
        first.schema_versions.reverse();
        first.validation_inputs.toolchain_compatibility.reverse();
        first.validation_inputs.dependency_artifacts.reverse();
        first.validation_inputs.policy_compatibility.reverse();
        first
            .validation_inputs
            .dependency_slice_fingerprints
            .reverse();
        first
            .validation_inputs
            .proof_reuse_schema_versions
            .reverse();
        first
            .validation_inputs
            .proof_reuse_evidence_identities
            .reverse();
        first.validation_inputs.diagnostic_refs.reverse();

        let first = cacheable(first);
        let second = cacheable(request());

        assert_eq!(first.final_hash, second.final_hash);
        assert_eq!(first.input_hashes, second.input_hashes);
        assert_eq!(first.dependency_hashes, second.dependency_hashes);
        assert_eq!(first.dependency_slices, second.dependency_slices);
        assert_eq!(first.schema_versions, second.schema_versions);
        assert_eq!(
            first.validation_inputs.toolchain_compatibility,
            second.validation_inputs.toolchain_compatibility
        );
        assert_eq!(
            first.validation_inputs.dependency_artifacts,
            second.validation_inputs.dependency_artifacts
        );
        assert_eq!(
            first.validation_inputs.policy_compatibility,
            second.validation_inputs.policy_compatibility
        );
        assert_eq!(
            first.validation_inputs.dependency_slice_fingerprints,
            second.validation_inputs.dependency_slice_fingerprints
        );
        assert_eq!(
            first.validation_inputs.proof_reuse_schema_versions,
            second.validation_inputs.proof_reuse_schema_versions
        );
        assert_eq!(
            first.validation_inputs.proof_reuse_evidence_identities,
            second.validation_inputs.proof_reuse_evidence_identities
        );
        assert_eq!(
            first.validation_inputs.diagnostic_refs,
            second.validation_inputs.diagnostic_refs
        );
    }

    #[test]
    fn every_semantic_field_changes_final_hash() {
        let base = key_for(request()).final_hash;
        let cases = vec![
            mutate(|request| request.phase = PipelinePhase::new("proof")),
            mutate(|request| request.work_unit = WorkUnit::new("module:beta")),
            mutate(|request| request.source_identity = None),
            mutate(|request| {
                request
                    .source_identity
                    .as_mut()
                    .expect("source identity")
                    .package_id = PackageId::new("other-pkg");
            }),
            mutate(|request| {
                request
                    .source_identity
                    .as_mut()
                    .expect("source identity")
                    .module_path = ModulePath::new("pkg.beta");
            }),
            mutate(|request| {
                request
                    .source_identity
                    .as_mut()
                    .expect("source identity")
                    .normalized_source_path = normalized_path("src/beta.miz");
            }),
            mutate(|request| {
                request
                    .source_identity
                    .as_mut()
                    .expect("source identity")
                    .source_hash = hash(99);
            }),
            mutate(|request| {
                request
                    .source_identity
                    .as_mut()
                    .expect("source identity")
                    .language_edition = Edition::new("2027");
            }),
            mutate(|request| request.input_hashes[0].name = "source-v2".to_owned()),
            mutate(|request| request.input_hashes[0].domain = "domain/source-v2".to_owned()),
            mutate(|request| request.input_hashes[0].digest = hash(100)),
            mutate(|request| request.dependency_hashes[0].dependency_kind = "summary".to_owned()),
            mutate(|request| request.dependency_hashes[0].package_id = "dep2".to_owned()),
            mutate(|request| request.dependency_hashes[0].module_path = "dep.beta".to_owned()),
            mutate(|request| request.dependency_hashes[0].name = "artifact-v2".to_owned()),
            mutate(|request| request.dependency_hashes[0].domain = "domain/summary".to_owned()),
            mutate(|request| request.dependency_hashes[0].digest = hash(101)),
            mutate(|request| request.dependency_slices[0].slice_kind = "definition".to_owned()),
            mutate(|request| request.dependency_slices[0].owner = "pkg.alpha::D1".to_owned()),
            mutate(|request| request.dependency_slices[0].name = "expanded".to_owned()),
            mutate(|request| request.dependency_slices[0].domain = "domain/definition".to_owned()),
            mutate(|request| request.dependency_slices[0].digest = hash(102)),
            mutate(|request| request.config_hash = hash(103)),
            mutate(|request| request.schema_versions[0].schema_family = "vc-ir-v2".to_owned()),
            mutate(|request| request.schema_versions[0].name = "output-v2".to_owned()),
            mutate(|request| request.schema_versions[0].version = SchemaVersion::new("schema/v2")),
            mutate(|request| request.policy_fingerprint = PolicyFingerprint::new(hash(104))),
            mutate(|request| {
                request.validation_inputs.cache_schema_compatibility =
                    SchemaVersion::new("mizar-cache/cache-key-schema/v1+compat");
            }),
            mutate(|request| request.validation_inputs.canonical_vc_fingerprint = Some(hash(105))),
            mutate(|request| request.validation_inputs.local_context_fingerprint = Some(hash(106))),
            mutate(|request| {
                request.validation_inputs.dependency_artifacts[0].package_id = "dep2".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.dependency_artifacts[0].module_path =
                    "dep.beta".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.dependency_artifacts[0].artifact_kind =
                    "summary".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.dependency_artifacts[0].artifact_path =
                    "build/dep.beta.summary.json".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.dependency_artifacts[0].domain =
                    "artifact-summary".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.dependency_artifacts[0].digest = hash(107);
            }),
            mutate(|request| {
                request.validation_inputs.footprint_completeness =
                    FootprintCompleteness::IncompleteUncacheable;
            }),
            mutate(|request| request.validation_inputs.uncacheable = true),
            mutate(|request| {
                request.validation_inputs.policy_compatibility[0].family =
                    "proof-policy-v2".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.policy_compatibility[0].field_name =
                    "allow_external".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.policy_compatibility[0].value = "false".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.dependency_slice_fingerprints[0].slice_kind =
                    "proof".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.dependency_slice_fingerprints[0].owner =
                    "pkg.alpha::T2".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.dependency_slice_fingerprints[0].name =
                    "premise".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.dependency_slice_fingerprints[0].domain =
                    "domain/proof".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.dependency_slice_fingerprints[0].digest = hash(108);
            }),
            mutate(|request| {
                request.validation_inputs.obligation_anchor_fingerprint = Some(hash(109));
            }),
            mutate(|request| {
                request
                    .validation_inputs
                    .selected_proof_witness_hash
                    .as_mut()
                    .expect("selected witness")
                    .name = "selected-witness-v2".to_owned();
            }),
            mutate(|request| {
                request
                    .validation_inputs
                    .selected_proof_witness_hash
                    .as_mut()
                    .expect("selected witness")
                    .domain = "domain/selected-witness-v2".to_owned();
            }),
            mutate(|request| {
                request
                    .validation_inputs
                    .selected_proof_witness_hash
                    .as_mut()
                    .expect("selected witness")
                    .digest = hash(110);
            }),
            mutate(|request| {
                request
                    .validation_inputs
                    .deterministic_discharge_hash
                    .as_mut()
                    .expect("deterministic discharge")
                    .name = "deterministic-discharge-v2".to_owned();
            }),
            mutate(|request| {
                request
                    .validation_inputs
                    .deterministic_discharge_hash
                    .as_mut()
                    .expect("deterministic discharge")
                    .domain = "domain/deterministic-discharge-v2".to_owned();
            }),
            mutate(|request| {
                request
                    .validation_inputs
                    .deterministic_discharge_hash
                    .as_mut()
                    .expect("deterministic discharge")
                    .digest = hash(111);
            }),
            mutate(|request| {
                request.validation_inputs.proof_reuse_schema_versions[0].schema_family =
                    "proof-reuse-v2".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.proof_reuse_schema_versions[0].name =
                    "metadata-v2".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.proof_reuse_schema_versions[0].version =
                    SchemaVersion::new("2.0");
            }),
            mutate(|request| {
                request
                    .validation_inputs
                    .proof_reuse_validation_hash
                    .as_mut()
                    .expect("proof reuse validation")
                    .name = "proof-reuse-validation-v2".to_owned();
            }),
            mutate(|request| {
                request
                    .validation_inputs
                    .proof_reuse_validation_hash
                    .as_mut()
                    .expect("proof reuse validation")
                    .domain = "domain/proof-reuse-validation-v2".to_owned();
            }),
            mutate(|request| {
                request
                    .validation_inputs
                    .proof_reuse_validation_hash
                    .as_mut()
                    .expect("proof reuse validation")
                    .digest = hash(112);
            }),
            mutate(|request| {
                request.validation_inputs.proof_reuse_evidence_identities[0]
                    .obligation_anchor_fingerprint = hash(115);
            }),
            mutate(|request| {
                request.validation_inputs.proof_reuse_evidence_identities[0].evidence_kind =
                    "KernelVerified".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.proof_reuse_evidence_identities[0]
                    .witness_or_discharge_domain = "witness-v2".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.proof_reuse_evidence_identities[0]
                    .witness_or_discharge_digest = hash(113);
            }),
            mutate(|request| {
                request.validation_inputs.diagnostic_refs[0].diagnostic_ref_kind =
                    "other_explanation".to_owned();
            }),
            mutate(|request| {
                request.validation_inputs.diagnostic_refs[0].diagnostic_ref_hash = hash(114);
            }),
            mutate(|request| {
                request
                    .validation_inputs
                    .toolchain_compatibility
                    .push(CompatibilityField {
                        family: "rustc".to_owned(),
                        field_name: "version".to_owned(),
                        value: "1.99".to_owned(),
                    });
            }),
        ];

        for changed in cases {
            assert_ne!(base, key_for(changed).final_hash);
        }
    }

    #[test]
    fn conflicting_duplicate_identity_rejects_each_canonical_collection() {
        let mut cases = Vec::new();

        let mut conflicting = request();
        conflicting.input_hashes.push(NamedHash {
            name: conflicting.input_hashes[0].name.clone(),
            domain: conflicting.input_hashes[0].domain.clone(),
            digest: hash(250),
        });
        cases.push((conflicting, "input_hashes"));

        let mut conflicting = request();
        conflicting.dependency_hashes[0].digest = hash(251);
        conflicting
            .dependency_hashes
            .push(request().dependency_hashes[0].clone());
        cases.push((conflicting, "dependency_hashes"));

        let mut conflicting = request();
        conflicting.dependency_slices[0].digest = hash(252);
        conflicting
            .dependency_slices
            .push(request().dependency_slices[0].clone());
        cases.push((conflicting, "dependency_slices"));

        let mut conflicting = request();
        conflicting.schema_versions[0].version = SchemaVersion::new("9.9");
        conflicting
            .schema_versions
            .push(request().schema_versions[0].clone());
        cases.push((conflicting, "schema_versions"));

        let mut conflicting = request();
        conflicting.validation_inputs.toolchain_compatibility[0].value = "other".to_owned();
        conflicting
            .validation_inputs
            .toolchain_compatibility
            .push(request().validation_inputs.toolchain_compatibility[0].clone());
        cases.push((conflicting, "toolchain_compatibility"));

        let mut conflicting = request();
        conflicting.validation_inputs.dependency_artifacts[0].digest = hash(253);
        conflicting
            .validation_inputs
            .dependency_artifacts
            .push(request().validation_inputs.dependency_artifacts[0].clone());
        cases.push((conflicting, "dependency_artifacts"));

        let mut conflicting = request();
        conflicting.validation_inputs.policy_compatibility[0].value = "other".to_owned();
        conflicting
            .validation_inputs
            .policy_compatibility
            .push(request().validation_inputs.policy_compatibility[0].clone());
        cases.push((conflicting, "policy_compatibility"));

        let mut conflicting = request();
        conflicting.validation_inputs.dependency_slice_fingerprints[0].digest = hash(254);
        conflicting
            .validation_inputs
            .dependency_slice_fingerprints
            .push(request().validation_inputs.dependency_slice_fingerprints[0].clone());
        cases.push((conflicting, "dependency_slice_fingerprints"));

        let mut conflicting = request();
        conflicting.validation_inputs.proof_reuse_schema_versions[0].version =
            SchemaVersion::new("9.9");
        conflicting
            .validation_inputs
            .proof_reuse_schema_versions
            .push(request().validation_inputs.proof_reuse_schema_versions[0].clone());
        cases.push((conflicting, "proof_reuse_schema_versions"));

        let mut conflicting = request();
        conflicting
            .validation_inputs
            .proof_reuse_evidence_identities[0]
            .witness_or_discharge_digest = hash(255);
        conflicting
            .validation_inputs
            .proof_reuse_evidence_identities
            .push(request().validation_inputs.proof_reuse_evidence_identities[0].clone());
        cases.push((conflicting, "proof_reuse_evidence_identities"));

        for (request, collection) in cases {
            assert!(matches!(
                CacheKeyBuilder::new(request).build(),
                CacheKeyBuildOutcome::NoKey(CacheKeyBuildRejection::ConflictingDuplicate {
                    collection: actual,
                    ..
                }) if actual == collection
            ));
        }
    }

    #[test]
    fn unsupported_schema_empty_identities_and_incomplete_proof_reuse_fail_closed() {
        let mut unsupported = request();
        unsupported.cache_schema_version = SchemaVersion::new("mizar-cache/cache-key-schema/v99");
        assert!(matches!(
            CacheKeyBuilder::new(unsupported).build(),
            CacheKeyBuildOutcome::NoKey(CacheKeyBuildRejection::UnsupportedCacheKeySchema { .. })
        ));

        let mut empty_phase = request();
        empty_phase.phase = PipelinePhase::new("");
        assert!(matches!(
            CacheKeyBuilder::new(empty_phase).build(),
            CacheKeyBuildOutcome::NoKey(CacheKeyBuildRejection::MissingRequiredIdentity {
                field: "phase"
            })
        ));

        let mut empty_name = request();
        empty_name.input_hashes[0].name.clear();
        assert!(matches!(
            CacheKeyBuilder::new(empty_name).build(),
            CacheKeyBuildOutcome::NoKey(CacheKeyBuildRejection::MissingRequiredIdentity {
                field: "input_hashes.name"
            })
        ));

        let mut missing_witness = request();
        missing_witness
            .validation_inputs
            .selected_proof_witness_hash = None;
        assert!(uncacheable(missing_witness).validation_inputs.uncacheable);

        let mut missing_discharge = request();
        missing_discharge
            .validation_inputs
            .deterministic_discharge_hash = None;
        assert!(uncacheable(missing_discharge).validation_inputs.uncacheable);

        let mut camel_case_witness = request();
        camel_case_witness
            .validation_inputs
            .proof_reuse_evidence_identities[0]
            .evidence_kind = "KernelVerified".to_owned();
        camel_case_witness
            .validation_inputs
            .selected_proof_witness_hash = None;
        assert!(
            uncacheable(camel_case_witness)
                .validation_inputs
                .uncacheable
        );

        let mut missing_vc = request();
        missing_vc.validation_inputs.canonical_vc_fingerprint = None;
        assert!(uncacheable(missing_vc).validation_inputs.uncacheable);

        let mut unknown_evidence = request();
        unknown_evidence
            .validation_inputs
            .proof_reuse_evidence_identities[0]
            .evidence_kind = "externally_attested".to_owned();
        assert!(uncacheable(unknown_evidence).validation_inputs.uncacheable);
    }

    #[test]
    fn uncacheable_marker_and_incomplete_footprint_produce_miss_outcome() {
        let mut explicit = request();
        explicit.validation_inputs.uncacheable = true;
        let explicit = uncacheable(explicit);
        assert!(explicit.validation_inputs.uncacheable);

        let mut incomplete = request();
        incomplete.validation_inputs.footprint_completeness =
            FootprintCompleteness::IncompleteUncacheable;
        let incomplete = uncacheable(incomplete);
        assert!(incomplete.validation_inputs.uncacheable);
        assert_eq!(
            incomplete.validation_inputs.footprint_completeness,
            FootprintCompleteness::IncompleteUncacheable
        );
    }

    #[test]
    fn diagnostic_refs_participate_only_when_supplied_and_nondeterministic_inputs_are_absent() {
        let base = cacheable(request()).final_hash;
        let mut changed = request();
        changed
            .validation_inputs
            .diagnostic_refs
            .push(DiagnosticRefHash {
                diagnostic_ref_kind: "cache_miss_explanation".to_owned(),
                diagnostic_ref_hash: hash(99),
            });
        assert_ne!(
            base,
            key_for(changed).final_hash,
            "diagnostic refs are hashed only when explicitly supplied as miss explanation refs"
        );

        let same_again = request();
        assert_eq!(base, cacheable(same_again).final_hash);
    }

    fn mutate(mut f: impl FnMut(&mut CacheKeyRequest)) -> CacheKeyRequest {
        let mut request = request();
        f(&mut request);
        request
    }

    fn cacheable(request: CacheKeyRequest) -> CacheKey {
        match CacheKeyBuilder::new(request).build() {
            CacheKeyBuildOutcome::Cacheable(key) => key,
            other => panic!("expected cacheable key, got {other:?}"),
        }
    }

    fn uncacheable(request: CacheKeyRequest) -> CacheKey {
        match CacheKeyBuilder::new(request).build() {
            CacheKeyBuildOutcome::Uncacheable(key) => key,
            other => panic!("expected uncacheable key, got {other:?}"),
        }
    }

    fn key_for(request: CacheKeyRequest) -> CacheKey {
        match CacheKeyBuilder::new(request).build() {
            CacheKeyBuildOutcome::Cacheable(key) | CacheKeyBuildOutcome::Uncacheable(key) => key,
            other => panic!("expected key-bearing outcome, got {other:?}"),
        }
    }

    fn request() -> CacheKeyRequest {
        CacheKeyRequest {
            cache_schema_version: SchemaVersion::default(),
            phase: PipelinePhase::new("vc"),
            work_unit: WorkUnit::new("module:alpha#vc:1"),
            source_identity: Some(SourceIdentity {
                package_id: PackageId::new("pkg"),
                module_path: ModulePath::new("pkg.alpha"),
                normalized_source_path: normalized_path("src/alpha.miz"),
                source_hash: hash(1),
                language_edition: Edition::new("2026"),
            }),
            input_hashes: vec![named("source", 1), named("core-ir", 2)],
            dependency_hashes: vec![
                dependency_hash("interface", "dep", "dep.alpha", "artifact", 3),
                dependency_hash("manifest", "dep", "", "manifest", 4),
            ],
            dependency_slices: vec![
                slice_hash("theorem", "pkg.alpha::T1", "used", 5),
                slice_hash("cluster", "pkg.alpha::C1", "used", 6),
            ],
            config_hash: hash(7),
            schema_versions: vec![
                schema("vc-ir", "output", "1.0"),
                schema("proof-reuse", "metadata", "1.0"),
            ],
            policy_fingerprint: PolicyFingerprint::new(hash(8)),
            validation_inputs: validation_inputs(),
        }
    }

    fn validation_inputs() -> CacheValidationInputs {
        CacheValidationInputs {
            cache_schema_compatibility: SchemaVersion::default(),
            toolchain_compatibility: vec![
                CompatibilityField {
                    family: "mizar".to_owned(),
                    field_name: "version".to_owned(),
                    value: "dev".to_owned(),
                },
                CompatibilityField {
                    family: "rust".to_owned(),
                    field_name: "target".to_owned(),
                    value: "test".to_owned(),
                },
            ],
            dependency_artifacts: vec![
                DependencyArtifactAvailability {
                    package_id: "dep".to_owned(),
                    module_path: "dep.alpha".to_owned(),
                    artifact_kind: "mizir".to_owned(),
                    artifact_path: "build/dep.alpha.mizir.json".to_owned(),
                    domain: "artifact".to_owned(),
                    digest: hash(9),
                },
                DependencyArtifactAvailability {
                    package_id: "dep".to_owned(),
                    module_path: "".to_owned(),
                    artifact_kind: "manifest".to_owned(),
                    artifact_path: "build/artifact-manifest.json".to_owned(),
                    domain: "manifest".to_owned(),
                    digest: hash(10),
                },
            ],
            footprint_completeness: FootprintCompleteness::Complete,
            uncacheable: false,
            policy_compatibility: vec![
                CompatibilityField {
                    family: "proof-policy".to_owned(),
                    field_name: "require_kernel_certificates".to_owned(),
                    value: "true".to_owned(),
                },
                CompatibilityField {
                    family: "reuse-policy".to_owned(),
                    field_name: "allow_deterministic_discharge".to_owned(),
                    value: "true".to_owned(),
                },
            ],
            canonical_vc_fingerprint: Some(hash(11)),
            local_context_fingerprint: Some(hash(12)),
            dependency_slice_fingerprints: vec![
                slice_hash("vc", "pkg.alpha::T1", "dependency", 13),
                slice_hash("context", "pkg.alpha::T1", "local", 14),
            ],
            obligation_anchor_fingerprint: Some(hash(15)),
            selected_proof_witness_hash: Some(named("selected-witness", 16)),
            deterministic_discharge_hash: Some(named("deterministic-discharge", 17)),
            proof_reuse_schema_versions: vec![
                schema("proof-reuse", "metadata", "1.0"),
                schema("proof-reuse", "status", "1.0"),
            ],
            proof_reuse_validation_hash: Some(named("proof-reuse-validation", 18)),
            proof_reuse_evidence_identities: vec![
                ProofReuseEvidenceIdentity {
                    obligation_anchor_fingerprint: hash(15),
                    evidence_kind: "kernel_verified".to_owned(),
                    witness_or_discharge_domain: "witness".to_owned(),
                    witness_or_discharge_digest: hash(16),
                },
                ProofReuseEvidenceIdentity {
                    obligation_anchor_fingerprint: hash(15),
                    evidence_kind: "discharged_builtin".to_owned(),
                    witness_or_discharge_domain: "discharge".to_owned(),
                    witness_or_discharge_digest: hash(17),
                },
            ],
            diagnostic_refs: vec![
                DiagnosticRefHash {
                    diagnostic_ref_kind: "cache_miss_explanation".to_owned(),
                    diagnostic_ref_hash: hash(19),
                },
                DiagnosticRefHash {
                    diagnostic_ref_kind: "compatibility_trace".to_owned(),
                    diagnostic_ref_hash: hash(20),
                },
            ],
        }
    }

    fn named(name: &str, seed: u8) -> NamedHash {
        NamedHash {
            name: name.to_owned(),
            domain: format!("domain/{name}"),
            digest: hash(seed),
        }
    }

    fn dependency_hash(
        dependency_kind: &str,
        package_id: &str,
        module_path: &str,
        name: &str,
        seed: u8,
    ) -> DependencyHash {
        DependencyHash {
            dependency_kind: dependency_kind.to_owned(),
            package_id: package_id.to_owned(),
            module_path: module_path.to_owned(),
            name: name.to_owned(),
            domain: format!("domain/{dependency_kind}"),
            digest: hash(seed),
        }
    }

    fn slice_hash(slice_kind: &str, owner: &str, name: &str, seed: u8) -> DependencySliceHash {
        DependencySliceHash {
            slice_kind: slice_kind.to_owned(),
            owner: owner.to_owned(),
            name: name.to_owned(),
            domain: format!("domain/{slice_kind}"),
            digest: hash(seed),
        }
    }

    fn schema(family: &str, name: &str, version: &str) -> NamedSchemaVersion {
        NamedSchemaVersion {
            schema_family: family.to_owned(),
            name: name.to_owned(),
            version: SchemaVersion::new(version),
        }
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }

    fn normalized_path(value: &str) -> NormalizedPath {
        static NEXT_TEST_PATH: AtomicUsize = AtomicUsize::new(0);

        let suffix = NEXT_TEST_PATH.fetch_add(1, Ordering::Relaxed);
        let package_root = std::env::temp_dir().join(format!(
            "mizar-cache-key-test-{}-{suffix}",
            std::process::id()
        ));
        let source_path = package_root.join(value);
        std::fs::create_dir_all(source_path.parent().expect("test path has parent"))
            .expect("create test source dir");
        std::fs::write(&source_path, b"environ\nbegin\n").expect("write test source");

        let normalized = mizar_session::normalize_path(&package_root, &source_path)
            .expect("test normalized path");
        std::fs::remove_dir_all(&package_root).expect("remove test source dir");
        normalized
    }
}
