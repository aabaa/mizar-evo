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
    /// All required dependency families are covered, but some slices are coarse.
    ConservativeComplete,
    /// Footprint completeness or schema is unsupported by this cache version.
    Unsupported,
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
            FootprintCompleteness::ConservativeComplete => "conservative_complete",
            FootprintCompleteness::Unsupported => "unsupported",
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
mod tests;
