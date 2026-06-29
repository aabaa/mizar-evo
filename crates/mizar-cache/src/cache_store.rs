//! Internal cache record storage and fail-closed lookup validation.
//!
//! The design is specified in
//! [cache_store.md](../../../doc/design/mizar-cache/en/cache_store.md).

use std::{
    collections::BTreeMap,
    error::Error,
    fmt, fs,
    fs::OpenOptions,
    io,
    io::Write,
    path::{Path, PathBuf},
};

use mizar_artifact::store::{CanonicalJson, canonical_json_bytes};
use mizar_session::Hash;

use crate::cache_key::{
    CACHE_KEY_SCHEMA_VERSION, CacheKey, CacheKeyBuildOutcome, CompatibilityField,
    DependencyArtifactAvailability, DependencyHash, DependencySliceHash, DiagnosticRefHash,
    FootprintCompleteness, NamedHash, NamedSchemaVersion, ProofReuseEvidenceIdentity,
    SchemaVersion, SourceIdentity,
};

/// Current on-disk cache record schema version.
pub const CACHE_RECORD_SCHEMA_VERSION: &str = "mizar-cache/cache-record-schema/v1";
/// Magic bytes for the cache record binary envelope.
pub const CACHE_RECORD_MAGIC: &[u8] = b"MIZAR-CACHE-RECORD\0";

const RECORD_FORMAT_VERSION: u32 = 1;
const OUTPUT_HASH_DOMAIN: &str = "mizar-cache/cache-record-output/v1";
const RECORD_EXTENSION: &str = "mcr";
const TEMP_FILE_ATTEMPTS: usize = 32;

/// Root for internal cache records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStoreRoot {
    root: PathBuf,
}

/// Header for a reusable cache record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheRecordHeader {
    /// On-disk cache record schema version.
    pub cache_record_schema_version: SchemaVersion,
    /// Canonical key that produced the record.
    pub key: CacheKey,
    /// Toolchain identity and compatibility fields for the producer.
    pub produced_by: Vec<CompatibilityField>,
    /// Output descriptor.
    pub output: CacheOutputDescriptor,
    /// Explicit uncacheable marker.
    pub uncacheable: bool,
    /// Diagnostic-only references for miss explanations.
    pub diagnostic_refs: Vec<DiagnosticRefHash>,
}

/// Cached output descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheOutputDescriptor {
    /// Output bytes are stored inline in the record envelope.
    Inline {
        /// Hash of the canonical output bytes.
        output_hash: Hash,
        /// Output byte length.
        byte_len: u64,
    },
    /// Output bytes are stored in the later task-9 blob store.
    Blob {
        /// Content-addressed blob reference.
        blob: CacheBlobRef,
        /// Hash of the canonical output bytes.
        output_hash: Hash,
        /// Output byte length.
        byte_len: u64,
    },
}

/// Reference to a content-addressed blob.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CacheBlobRef {
    /// Hash family such as `blake3`.
    pub hash_family: String,
    /// Digest encoded as lowercase hexadecimal.
    pub digest: String,
}

/// Cache record with inline output bytes for task 8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheRecord {
    /// Record header.
    pub header: CacheRecordHeader,
    /// Inline output bytes. Blob-backed records have empty inline output.
    pub output: Vec<u8>,
}

/// Result of cache lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheLookupOutcome {
    /// A validated hit.
    Hit(Box<CacheRecord>),
    /// A fail-closed miss.
    Miss(CacheMiss),
}

/// Result of inserting a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheInsertOutcome {
    /// A new record was written.
    Inserted,
    /// An identical record already existed.
    AlreadyPresent,
    /// The record was explicitly uncacheable or incomplete.
    RejectedUncacheable,
}

/// Fail-closed cache miss reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheMiss {
    /// No record exists at the expected key path.
    NotFound,
    /// A schema version is unknown or incompatible.
    UnknownSchema,
    /// Producing toolchain compatibility is unknown or incompatible.
    UnknownToolchain,
    /// Dependency footprint is incomplete.
    IncompleteFootprint,
    /// Dependency footprint completeness or schema is unsupported.
    UnsupportedFootprint,
    /// The key or record is explicitly uncacheable.
    Uncacheable,
    /// A dependency artifact is missing or mismatched.
    DependencyUnavailable,
    /// Verifier/proof policy is incompatible.
    PolicyIncompatible,
    /// Proof-reuse validation data is absent or invalid.
    ProofReuseInvalid,
    /// The record envelope, header, payload, or output hash is corrupt.
    CorruptRecord,
}

/// Errors that prevent insertion.
#[derive(Debug)]
#[non_exhaustive]
pub enum CacheStoreError {
    /// Filesystem I/O failed.
    Io {
        /// Operation name.
        operation: &'static str,
        /// Path involved in the operation.
        path: PathBuf,
        /// Source error.
        source: io::Error,
    },
    /// The record is not valid for insertion.
    InvalidRecord {
        /// Fail-closed reason.
        reason: CacheMiss,
    },
    /// A different record already exists for the same validated key.
    DivergentRecord {
        /// Existing record path.
        path: PathBuf,
    },
}

impl CacheStoreRoot {
    /// Creates a cache store rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the cache root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the deterministic record path for `key`.
    pub fn record_path(&self, key: &CacheKey) -> PathBuf {
        self.root
            .join("records")
            .join(path_component_for(key.phase.as_str()))
            .join(format!("{}.{}", hash_hex(key.final_hash), RECORD_EXTENSION))
    }

    /// Looks up an already-built key.
    pub fn lookup(&self, key: &CacheKey) -> CacheLookupOutcome {
        if key.cache_schema_version.as_str() != CACHE_KEY_SCHEMA_VERSION {
            return CacheLookupOutcome::Miss(CacheMiss::UnknownSchema);
        }
        if let Some(miss) = footprint_miss(key.validation_inputs.footprint_completeness) {
            return CacheLookupOutcome::Miss(miss);
        }
        if key.validation_inputs.uncacheable {
            return CacheLookupOutcome::Miss(CacheMiss::Uncacheable);
        }

        let path = self.record_path(key);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return CacheLookupOutcome::Miss(CacheMiss::NotFound);
            }
            Err(_) => {
                return CacheLookupOutcome::Miss(CacheMiss::CorruptRecord);
            }
        };

        let (decoded, output) = match decode_record_bytes(&bytes) {
            Ok(record) => record,
            Err(miss) => return CacheLookupOutcome::Miss(miss),
        };

        match validate_decoded_record(&self.root, key, decoded, output) {
            Ok(record) => CacheLookupOutcome::Hit(Box::new(record)),
            Err(miss) => CacheLookupOutcome::Miss(miss),
        }
    }

    /// Looks up a key-builder outcome, rejecting uncacheable/no-key outcomes before disk access.
    pub fn lookup_key_outcome(&self, outcome: &CacheKeyBuildOutcome) -> CacheLookupOutcome {
        match outcome {
            CacheKeyBuildOutcome::Cacheable(key) => self.lookup(key),
            CacheKeyBuildOutcome::Uncacheable(_) => {
                CacheLookupOutcome::Miss(CacheMiss::Uncacheable)
            }
            CacheKeyBuildOutcome::NoKey(_) => CacheLookupOutcome::Miss(CacheMiss::UnknownSchema),
        }
    }

    /// Inserts a complete inline record.
    pub fn insert(&self, record: &CacheRecord) -> Result<CacheInsertOutcome, CacheStoreError> {
        if let Some(miss) = insertion_rejection(record) {
            return match miss {
                CacheMiss::Uncacheable | CacheMiss::IncompleteFootprint => {
                    Ok(CacheInsertOutcome::RejectedUncacheable)
                }
                reason => Err(CacheStoreError::InvalidRecord { reason }),
            };
        }

        let bytes = encode_record(record);
        let (decoded, output) = decode_record_bytes(&bytes)
            .map_err(|reason| CacheStoreError::InvalidRecord { reason })?;
        validate_decoded_record(&self.root, &record.header.key, decoded, output)
            .map_err(|reason| CacheStoreError::InvalidRecord { reason })?;

        let path = self.record_path(&record.header.key);
        if let Ok(existing) = fs::read(&path) {
            if existing == bytes {
                return Ok(CacheInsertOutcome::AlreadyPresent);
            }
            return Err(CacheStoreError::DivergentRecord { path });
        }

        let parent = path
            .parent()
            .expect("record path always has parent directories");
        fs::create_dir_all(parent).map_err(|source| CacheStoreError::Io {
            operation: "create_dir_all",
            path: parent.to_path_buf(),
            source,
        })?;

        let temporary_dir = self.root.join("tmp");
        fs::create_dir_all(&temporary_dir).map_err(|source| CacheStoreError::Io {
            operation: "create_dir_all",
            path: temporary_dir.clone(),
            source,
        })?;

        self.write_via_unique_temp(&path, &temporary_dir, &bytes, record)
    }

    fn write_via_unique_temp(
        &self,
        path: &Path,
        temporary_dir: &Path,
        bytes: &[u8],
        record: &CacheRecord,
    ) -> Result<CacheInsertOutcome, CacheStoreError> {
        for attempt in 0..TEMP_FILE_ATTEMPTS {
            let temporary = temporary_dir.join(format!(
                "{}-{}-{attempt}.tmp",
                path_component_for(record.header.key.phase.as_str()),
                hash_hex(record.header.key.final_hash)
            ));
            let mut file = match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => file,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(source) => {
                    return Err(CacheStoreError::Io {
                        operation: "create_new",
                        path: temporary,
                        source,
                    });
                }
            };
            file.write_all(bytes)
                .map_err(|source| CacheStoreError::Io {
                    operation: "write",
                    path: temporary.clone(),
                    source,
                })?;
            file.sync_all().map_err(|source| CacheStoreError::Io {
                operation: "sync_all",
                path: temporary.clone(),
                source,
            })?;
            drop(file);

            match fs::hard_link(&temporary, path) {
                Ok(()) => {
                    let _ = fs::remove_file(&temporary);
                    return Ok(CacheInsertOutcome::Inserted);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    let _ = fs::remove_file(&temporary);
                    if let Ok(existing) = fs::read(path)
                        && existing == bytes
                    {
                        return Ok(CacheInsertOutcome::AlreadyPresent);
                    }
                    return Err(CacheStoreError::DivergentRecord {
                        path: path.to_path_buf(),
                    });
                }
                Err(source) => {
                    let _ = fs::remove_file(&temporary);
                    return Err(CacheStoreError::Io {
                        operation: "hard_link",
                        path: path.to_path_buf(),
                        source,
                    });
                }
            }
        }

        Err(CacheStoreError::DivergentRecord {
            path: path.to_path_buf(),
        })
    }
}

impl CacheRecordHeader {
    /// Creates an inline-output header.
    pub fn new_inline(key: CacheKey, produced_by: Vec<CompatibilityField>, output: &[u8]) -> Self {
        Self {
            cache_record_schema_version: SchemaVersion::new(CACHE_RECORD_SCHEMA_VERSION),
            key,
            produced_by,
            output: CacheOutputDescriptor::inline(output),
            uncacheable: false,
            diagnostic_refs: Vec::new(),
        }
    }
}

impl CacheOutputDescriptor {
    /// Creates an inline descriptor for `output`.
    pub fn inline(output: &[u8]) -> Self {
        Self::Inline {
            output_hash: output_hash(output),
            byte_len: output.len() as u64,
        }
    }

    fn output_hash(&self) -> Hash {
        match self {
            Self::Inline { output_hash, .. } | Self::Blob { output_hash, .. } => *output_hash,
        }
    }

    fn byte_len(&self) -> u64 {
        match self {
            Self::Inline { byte_len, .. } | Self::Blob { byte_len, .. } => *byte_len,
        }
    }
}

impl CacheRecord {
    /// Creates an inline cache record.
    pub fn new_inline(
        key: CacheKey,
        produced_by: Vec<CompatibilityField>,
        output: impl Into<Vec<u8>>,
    ) -> Self {
        let output = output.into();
        Self {
            header: CacheRecordHeader::new_inline(key, produced_by, &output),
            output,
        }
    }
}

impl fmt::Display for CacheStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "cache store {operation} failed for `{}`: {source}",
                path.display()
            ),
            Self::InvalidRecord { reason } => {
                write!(formatter, "cache record is not insertable: {reason:?}")
            }
            Self::DivergentRecord { path } => write!(
                formatter,
                "divergent cache record already exists at `{}`",
                path.display()
            ),
        }
    }
}

impl Error for CacheStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidRecord { .. } | Self::DivergentRecord { .. } => None,
        }
    }
}

fn insertion_rejection(record: &CacheRecord) -> Option<CacheMiss> {
    if record.header.cache_record_schema_version.as_str() != CACHE_RECORD_SCHEMA_VERSION
        || record.header.key.cache_schema_version.as_str() != CACHE_KEY_SCHEMA_VERSION
    {
        return Some(CacheMiss::UnknownSchema);
    }
    if record.header.uncacheable || record.header.key.validation_inputs.uncacheable {
        return Some(CacheMiss::Uncacheable);
    }
    if let Some(miss) = footprint_miss(record.header.key.validation_inputs.footprint_completeness) {
        return Some(miss);
    }
    if schema_incompatible(&record.header.key) {
        return Some(CacheMiss::UnknownSchema);
    }
    if unknown_compatibility(&record.header.produced_by)
        || unknown_compatibility(&record.header.key.validation_inputs.toolchain_compatibility)
    {
        return Some(CacheMiss::UnknownToolchain);
    }
    if unknown_compatibility(&record.header.key.validation_inputs.policy_compatibility) {
        return Some(CacheMiss::PolicyIncompatible);
    }
    if proof_reuse_invalid(&record.header.key) {
        return Some(CacheMiss::ProofReuseInvalid);
    }
    if !matches!(record.header.output, CacheOutputDescriptor::Inline { .. }) {
        return Some(CacheMiss::CorruptRecord);
    }
    if record.header.output.byte_len() != record.output.len() as u64
        || record.header.output.output_hash() != output_hash(&record.output)
    {
        return Some(CacheMiss::CorruptRecord);
    }
    None
}

fn validate_decoded_record(
    root: &Path,
    requested_key: &CacheKey,
    decoded: DecodedHeader,
    output: Vec<u8>,
) -> Result<CacheRecord, CacheMiss> {
    if decoded.cache_record_schema_version != CACHE_RECORD_SCHEMA_VERSION
        || decoded.cache_key_schema_version != requested_key.cache_schema_version.as_str()
        || requested_key.cache_schema_version.as_str() != CACHE_KEY_SCHEMA_VERSION
    {
        return Err(CacheMiss::UnknownSchema);
    }
    if decoded.key_hash != requested_key.final_hash
        || decoded.phase != requested_key.phase.as_str()
        || decoded.work_unit != requested_key.work_unit.as_str()
        || decoded.key_json != cache_key_json(requested_key)
    {
        return Err(CacheMiss::CorruptRecord);
    }
    if decoded.uncacheable {
        return Err(CacheMiss::Uncacheable);
    }
    if let Some(miss) = footprint_miss(requested_key.validation_inputs.footprint_completeness) {
        return Err(miss);
    }
    if schema_incompatible(requested_key) {
        return Err(CacheMiss::UnknownSchema);
    }
    if dependency_unavailable(root, requested_key) {
        return Err(CacheMiss::DependencyUnavailable);
    }
    if unknown_compatibility(&decoded.produced_by)
        || unknown_compatibility(&requested_key.validation_inputs.toolchain_compatibility)
    {
        return Err(CacheMiss::UnknownToolchain);
    }
    if unknown_compatibility(&requested_key.validation_inputs.policy_compatibility) {
        return Err(CacheMiss::PolicyIncompatible);
    }
    if proof_reuse_invalid(requested_key) {
        return Err(CacheMiss::ProofReuseInvalid);
    }

    let output_descriptor = match decoded.output {
        DecodedOutput::Inline {
            output_hash,
            byte_len,
        } => CacheOutputDescriptor::Inline {
            output_hash,
            byte_len,
        },
        DecodedOutput::Blob => return Err(CacheMiss::CorruptRecord),
    };

    if output_descriptor.byte_len() != output.len() as u64
        || output_descriptor.output_hash() != output_hash(&output)
    {
        return Err(CacheMiss::CorruptRecord);
    }

    Ok(CacheRecord {
        header: CacheRecordHeader {
            cache_record_schema_version: SchemaVersion::new(CACHE_RECORD_SCHEMA_VERSION),
            key: requested_key.clone(),
            produced_by: decoded.produced_by,
            output: output_descriptor,
            uncacheable: false,
            diagnostic_refs: decoded.diagnostic_refs,
        },
        output,
    })
}

fn footprint_miss(value: FootprintCompleteness) -> Option<CacheMiss> {
    match value {
        FootprintCompleteness::Complete | FootprintCompleteness::ConservativeComplete => None,
        FootprintCompleteness::IncompleteUncacheable => Some(CacheMiss::IncompleteFootprint),
        FootprintCompleteness::Unsupported => Some(CacheMiss::UnsupportedFootprint),
    }
}

fn schema_incompatible(key: &CacheKey) -> bool {
    unknown_value(key.validation_inputs.cache_schema_compatibility.as_str())
        || key
            .schema_versions
            .iter()
            .any(|schema| unknown_value(schema.version.as_str()))
        || key
            .validation_inputs
            .proof_reuse_schema_versions
            .iter()
            .any(|schema| unknown_value(schema.version.as_str()))
}

fn dependency_unavailable(root: &Path, key: &CacheKey) -> bool {
    key.validation_inputs
        .dependency_artifacts
        .iter()
        .any(|artifact| {
            unknown_value(&artifact.artifact_path)
                || unknown_value(&artifact.domain)
                || fs::read(dependency_artifact_path(root, artifact))
                    .map(|bytes| {
                        dependency_artifact_hash(&artifact.domain, &bytes) != artifact.digest
                    })
                    .unwrap_or(true)
        })
}

fn dependency_artifact_path(root: &Path, artifact: &DependencyArtifactAvailability) -> PathBuf {
    let path = Path::new(&artifact.artifact_path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.parent().unwrap_or(root).join(path)
    }
}

fn unknown_compatibility(fields: &[CompatibilityField]) -> bool {
    fields.is_empty()
        || fields.iter().any(|field| {
            field.family.trim().is_empty()
                || field.field_name.trim().is_empty()
                || unknown_value(&field.value)
        })
}

fn unknown_value(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "" | "unknown" | "unsupported" | "incompatible" | "missing" | "opaque"
    )
}

fn proof_reuse_invalid(key: &CacheKey) -> bool {
    let validation = &key.validation_inputs;
    let proof_phase = matches!(key.phase.as_str(), "proof" | "atp");
    let proof_related = proof_phase
        || !validation.proof_reuse_evidence_identities.is_empty()
        || validation.selected_proof_witness_hash.is_some()
        || validation.deterministic_discharge_hash.is_some()
        || validation.proof_reuse_validation_hash.is_some()
        || !validation.proof_reuse_schema_versions.is_empty();

    let vc_related = matches!(key.phase.as_str(), "vc" | "proof" | "atp")
        || proof_related
        || validation.canonical_vc_fingerprint.is_some()
        || validation.local_context_fingerprint.is_some()
        || !validation.dependency_slice_fingerprints.is_empty();

    if vc_related
        && (validation.canonical_vc_fingerprint.is_none()
            || validation.local_context_fingerprint.is_none()
            || validation.dependency_slice_fingerprints.is_empty())
    {
        return true;
    }

    if proof_related
        && (validation.obligation_anchor_fingerprint.is_none()
            || validation.proof_reuse_schema_versions.is_empty()
            || validation.proof_reuse_validation_hash.is_none()
            || validation.proof_reuse_evidence_identities.is_empty())
    {
        return true;
    }

    validation
        .proof_reuse_evidence_identities
        .iter()
        .any(|identity| match identity.evidence_kind.as_str() {
            "kernel_verified" | "KernelVerified" | "kernel-verified" => {
                validation.selected_proof_witness_hash.is_none()
            }
            "discharged_builtin" | "DischargedBuiltin" | "discharged-builtin" => {
                validation.deterministic_discharge_hash.is_none()
            }
            _ => true,
        })
}

fn encode_record(record: &CacheRecord) -> Vec<u8> {
    let header = canonical_json_bytes(&header_json(&record.header));
    let mut encoded = Vec::with_capacity(
        CACHE_RECORD_MAGIC.len() + 4 + 8 + header.len() + 8 + record.output.len(),
    );
    encoded.extend_from_slice(CACHE_RECORD_MAGIC);
    encoded.extend_from_slice(&RECORD_FORMAT_VERSION.to_le_bytes());
    encoded.extend_from_slice(&(header.len() as u64).to_le_bytes());
    encoded.extend_from_slice(&header);
    encoded.extend_from_slice(&(record.output.len() as u64).to_le_bytes());
    encoded.extend_from_slice(&record.output);
    encoded
}

fn decode_record_bytes(bytes: &[u8]) -> Result<(DecodedHeader, Vec<u8>), CacheMiss> {
    let mut cursor = 0usize;
    take_exact(bytes, &mut cursor, CACHE_RECORD_MAGIC.len())
        .filter(|actual| *actual == CACHE_RECORD_MAGIC)
        .ok_or(CacheMiss::CorruptRecord)?;
    let version = read_u32(bytes, &mut cursor).ok_or(CacheMiss::CorruptRecord)?;
    if version != RECORD_FORMAT_VERSION {
        return Err(CacheMiss::UnknownSchema);
    }
    let header_len = read_u64(bytes, &mut cursor).ok_or(CacheMiss::CorruptRecord)? as usize;
    let header_bytes =
        take_exact(bytes, &mut cursor, header_len).ok_or(CacheMiss::CorruptRecord)?;
    let payload_len = read_u64(bytes, &mut cursor).ok_or(CacheMiss::CorruptRecord)? as usize;
    let payload = take_exact(bytes, &mut cursor, payload_len).ok_or(CacheMiss::CorruptRecord)?;
    if cursor != bytes.len() {
        return Err(CacheMiss::CorruptRecord);
    }

    let header_json = parse_canonical_json(header_bytes)?;
    let decoded = decoded_header_from_json(header_json)?;
    Ok((decoded, payload.to_vec()))
}

fn take_exact<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Option<&'a [u8]> {
    let end = cursor.checked_add(len)?;
    let value = bytes.get(*cursor..end)?;
    *cursor = end;
    Some(value)
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Option<u32> {
    let mut buffer = [0; 4];
    buffer.copy_from_slice(take_exact(bytes, cursor, 4)?);
    Some(u32::from_le_bytes(buffer))
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Option<u64> {
    let mut buffer = [0; 8];
    buffer.copy_from_slice(take_exact(bytes, cursor, 8)?);
    Some(u64::from_le_bytes(buffer))
}

#[derive(Debug)]
struct DecodedHeader {
    cache_record_schema_version: String,
    cache_key_schema_version: String,
    key_json: CanonicalJson,
    key_hash: Hash,
    phase: String,
    work_unit: String,
    produced_by: Vec<CompatibilityField>,
    output: DecodedOutput,
    uncacheable: bool,
    diagnostic_refs: Vec<DiagnosticRefHash>,
}

#[derive(Debug)]
enum DecodedOutput {
    Inline { output_hash: Hash, byte_len: u64 },
    Blob,
}

fn decoded_header_from_json(value: CanonicalJson) -> Result<DecodedHeader, CacheMiss> {
    let header_object = object(&value)?;
    let output_object = object(field(header_object, "output")?)?;
    let output_hash = hash_field(output_object, "output_hash")?;
    let byte_len = string_field(output_object, "byte_len")?
        .parse()
        .map_err(|_| CacheMiss::CorruptRecord)?;
    let output = match string_field(output_object, "kind")? {
        "inline" => DecodedOutput::Inline {
            output_hash,
            byte_len,
        },
        "blob" => DecodedOutput::Blob,
        _ => return Err(CacheMiss::CorruptRecord),
    };

    Ok(DecodedHeader {
        cache_record_schema_version: string_field(header_object, "cache_record_schema_version")?
            .to_owned(),
        cache_key_schema_version: string_field(header_object, "cache_key_schema_version")?
            .to_owned(),
        key_json: field(header_object, "key")?.clone(),
        key_hash: hash_field(header_object, "key_hash")?,
        phase: string_field(header_object, "phase")?.to_owned(),
        work_unit: string_field(header_object, "work_unit")?.to_owned(),
        produced_by: compatibility_fields(field(header_object, "produced_by")?)?,
        output,
        uncacheable: bool_field(header_object, "uncacheable")?,
        diagnostic_refs: diagnostic_refs_from_json(field(header_object, "diagnostic_refs")?)?,
    })
}

fn header_json(header: &CacheRecordHeader) -> CanonicalJson {
    json_object([
        (
            "cache_key_schema_version",
            CanonicalJson::string(header.key.cache_schema_version.as_str()),
        ),
        (
            "cache_record_schema_version",
            CanonicalJson::string(header.cache_record_schema_version.as_str()),
        ),
        ("key", cache_key_json(&header.key)),
        (
            "diagnostic_refs",
            diagnostic_refs_json(&header.diagnostic_refs),
        ),
        (
            "key_hash",
            CanonicalJson::string(hash_hex(header.key.final_hash)),
        ),
        ("output", output_json(&header.output)),
        ("phase", CanonicalJson::string(header.key.phase.as_str())),
        ("produced_by", compatibility_json(&header.produced_by)),
        ("uncacheable", CanonicalJson::bool(header.uncacheable)),
        (
            "work_unit",
            CanonicalJson::string(header.key.work_unit.as_str()),
        ),
    ])
}

fn output_json(output: &CacheOutputDescriptor) -> CanonicalJson {
    match output {
        CacheOutputDescriptor::Inline {
            output_hash,
            byte_len,
        } => json_object([
            ("byte_len", CanonicalJson::string(byte_len.to_string())),
            ("kind", CanonicalJson::string("inline")),
            ("output_hash", CanonicalJson::string(hash_hex(*output_hash))),
        ]),
        CacheOutputDescriptor::Blob {
            blob,
            output_hash,
            byte_len,
        } => json_object([
            ("blob_digest", CanonicalJson::string(&blob.digest)),
            ("byte_len", CanonicalJson::string(byte_len.to_string())),
            ("hash_family", CanonicalJson::string(&blob.hash_family)),
            ("kind", CanonicalJson::string("blob")),
            ("output_hash", CanonicalJson::string(hash_hex(*output_hash))),
        ]),
    }
}

fn cache_key_json(key: &CacheKey) -> CanonicalJson {
    json_object([
        (
            "cache_schema_version",
            CanonicalJson::string(key.cache_schema_version.as_str()),
        ),
        (
            "config_hash",
            CanonicalJson::string(hash_hex(key.config_hash)),
        ),
        (
            "dependency_hashes",
            dependency_hashes_json(&key.dependency_hashes),
        ),
        (
            "dependency_slices",
            dependency_slices_json(&key.dependency_slices),
        ),
        (
            "final_hash",
            CanonicalJson::string(hash_hex(key.final_hash)),
        ),
        ("input_hashes", named_hashes_json(&key.input_hashes)),
        ("phase", CanonicalJson::string(key.phase.as_str())),
        (
            "policy_fingerprint",
            CanonicalJson::string(hash_hex(key.policy_fingerprint.hash())),
        ),
        (
            "schema_versions",
            schema_versions_json(&key.schema_versions),
        ),
        (
            "source_identity",
            source_identity_json(key.source_identity.as_ref()),
        ),
        ("validation_inputs", validation_inputs_json(key)),
        ("work_unit", CanonicalJson::string(key.work_unit.as_str())),
    ])
}

fn source_identity_json(value: Option<&SourceIdentity>) -> CanonicalJson {
    value.map_or_else(CanonicalJson::null, |source| {
        json_object([
            (
                "language_edition",
                CanonicalJson::string(source.language_edition.as_str()),
            ),
            (
                "module_path",
                CanonicalJson::string(source.module_path.as_str()),
            ),
            (
                "normalized_source_path",
                CanonicalJson::string(source.normalized_source_path.as_str()),
            ),
            (
                "package_id",
                CanonicalJson::string(source.package_id.as_str()),
            ),
            (
                "source_hash",
                CanonicalJson::string(hash_hex(source.source_hash)),
            ),
        ])
    })
}

fn validation_inputs_json(key: &CacheKey) -> CanonicalJson {
    let validation = &key.validation_inputs;
    json_object([
        (
            "cache_schema_compatibility",
            CanonicalJson::string(validation.cache_schema_compatibility.as_str()),
        ),
        (
            "canonical_vc_fingerprint",
            optional_hash_json(validation.canonical_vc_fingerprint),
        ),
        (
            "dependency_artifacts",
            dependency_artifacts_json(&validation.dependency_artifacts),
        ),
        (
            "dependency_slice_fingerprints",
            dependency_slices_json(&validation.dependency_slice_fingerprints),
        ),
        (
            "deterministic_discharge_hash",
            optional_named_hash_json(validation.deterministic_discharge_hash.as_ref()),
        ),
        (
            "diagnostic_refs",
            diagnostic_refs_json(&validation.diagnostic_refs),
        ),
        (
            "footprint_completeness",
            CanonicalJson::string(match validation.footprint_completeness {
                FootprintCompleteness::Complete => "complete",
                FootprintCompleteness::ConservativeComplete => "conservative_complete",
                FootprintCompleteness::Unsupported => "unsupported",
                FootprintCompleteness::IncompleteUncacheable => "incomplete_uncacheable",
            }),
        ),
        (
            "local_context_fingerprint",
            optional_hash_json(validation.local_context_fingerprint),
        ),
        (
            "obligation_anchor_fingerprint",
            optional_hash_json(validation.obligation_anchor_fingerprint),
        ),
        (
            "policy_compatibility",
            compatibility_json(&validation.policy_compatibility),
        ),
        (
            "proof_reuse_evidence_identities",
            proof_evidence_json(&validation.proof_reuse_evidence_identities),
        ),
        (
            "proof_reuse_schema_versions",
            schema_versions_json(&validation.proof_reuse_schema_versions),
        ),
        (
            "proof_reuse_validation_hash",
            optional_named_hash_json(validation.proof_reuse_validation_hash.as_ref()),
        ),
        (
            "selected_proof_witness_hash",
            optional_named_hash_json(validation.selected_proof_witness_hash.as_ref()),
        ),
        (
            "toolchain_compatibility",
            compatibility_json(&validation.toolchain_compatibility),
        ),
        ("uncacheable", CanonicalJson::bool(validation.uncacheable)),
    ])
}

fn named_hashes_json(values: &[NamedHash]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(named_hash_json))
}

fn named_hash_json(value: &NamedHash) -> CanonicalJson {
    json_object([
        ("digest", CanonicalJson::string(hash_hex(value.digest))),
        ("domain", CanonicalJson::string(&value.domain)),
        ("name", CanonicalJson::string(&value.name)),
    ])
}

fn optional_named_hash_json(value: Option<&NamedHash>) -> CanonicalJson {
    value.map_or_else(CanonicalJson::null, named_hash_json)
}

fn dependency_hashes_json(values: &[DependencyHash]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(|value| {
        json_object([
            (
                "dependency_kind",
                CanonicalJson::string(&value.dependency_kind),
            ),
            ("digest", CanonicalJson::string(hash_hex(value.digest))),
            ("domain", CanonicalJson::string(&value.domain)),
            ("module_path", CanonicalJson::string(&value.module_path)),
            ("name", CanonicalJson::string(&value.name)),
            ("package_id", CanonicalJson::string(&value.package_id)),
        ])
    }))
}

fn dependency_slices_json(values: &[DependencySliceHash]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(|value| {
        json_object([
            ("digest", CanonicalJson::string(hash_hex(value.digest))),
            ("domain", CanonicalJson::string(&value.domain)),
            ("name", CanonicalJson::string(&value.name)),
            ("owner", CanonicalJson::string(&value.owner)),
            ("slice_kind", CanonicalJson::string(&value.slice_kind)),
        ])
    }))
}

fn schema_versions_json(values: &[NamedSchemaVersion]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(|value| {
        json_object([
            ("name", CanonicalJson::string(&value.name)),
            ("schema_family", CanonicalJson::string(&value.schema_family)),
            ("version", CanonicalJson::string(value.version.as_str())),
        ])
    }))
}

fn dependency_artifacts_json(values: &[DependencyArtifactAvailability]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(|value| {
        json_object([
            ("artifact_kind", CanonicalJson::string(&value.artifact_kind)),
            ("artifact_path", CanonicalJson::string(&value.artifact_path)),
            ("digest", CanonicalJson::string(hash_hex(value.digest))),
            ("domain", CanonicalJson::string(&value.domain)),
            ("module_path", CanonicalJson::string(&value.module_path)),
            ("package_id", CanonicalJson::string(&value.package_id)),
        ])
    }))
}

fn compatibility_json(values: &[CompatibilityField]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(|value| {
        json_object([
            ("family", CanonicalJson::string(&value.family)),
            ("field_name", CanonicalJson::string(&value.field_name)),
            ("value", CanonicalJson::string(&value.value)),
        ])
    }))
}

fn proof_evidence_json(values: &[ProofReuseEvidenceIdentity]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(|value| {
        json_object([
            ("evidence_kind", CanonicalJson::string(&value.evidence_kind)),
            (
                "obligation_anchor_fingerprint",
                CanonicalJson::string(hash_hex(value.obligation_anchor_fingerprint)),
            ),
            (
                "witness_or_discharge_digest",
                CanonicalJson::string(hash_hex(value.witness_or_discharge_digest)),
            ),
            (
                "witness_or_discharge_domain",
                CanonicalJson::string(&value.witness_or_discharge_domain),
            ),
        ])
    }))
}

fn diagnostic_refs_json(values: &[DiagnosticRefHash]) -> CanonicalJson {
    CanonicalJson::array(values.iter().map(|value| {
        json_object([
            (
                "diagnostic_ref_hash",
                CanonicalJson::string(hash_hex(value.diagnostic_ref_hash)),
            ),
            (
                "diagnostic_ref_kind",
                CanonicalJson::string(&value.diagnostic_ref_kind),
            ),
        ])
    }))
}

fn diagnostic_refs_from_json(value: &CanonicalJson) -> Result<Vec<DiagnosticRefHash>, CacheMiss> {
    let CanonicalJson::Array(values) = value else {
        return Err(CacheMiss::CorruptRecord);
    };
    values
        .iter()
        .map(|value| {
            let object = object(value)?;
            Ok(DiagnosticRefHash {
                diagnostic_ref_kind: string_field(object, "diagnostic_ref_kind")?.to_owned(),
                diagnostic_ref_hash: hash_field(object, "diagnostic_ref_hash")?,
            })
        })
        .collect()
}

fn optional_hash_json(value: Option<Hash>) -> CanonicalJson {
    value.map_or_else(CanonicalJson::null, |hash| {
        CanonicalJson::string(hash_hex(hash))
    })
}

fn json_object<const N: usize>(fields: [(&'static str, CanonicalJson); N]) -> CanonicalJson {
    CanonicalJson::object(fields).expect("static object keys are unique")
}

fn object(value: &CanonicalJson) -> Result<&BTreeMap<String, CanonicalJson>, CacheMiss> {
    match value {
        CanonicalJson::Object(object) => Ok(object),
        _ => Err(CacheMiss::CorruptRecord),
    }
}

fn field<'a>(
    object: &'a BTreeMap<String, CanonicalJson>,
    key: &str,
) -> Result<&'a CanonicalJson, CacheMiss> {
    object.get(key).ok_or(CacheMiss::CorruptRecord)
}

fn string_field<'a>(
    object: &'a BTreeMap<String, CanonicalJson>,
    key: &str,
) -> Result<&'a str, CacheMiss> {
    match field(object, key)? {
        CanonicalJson::String(value) => Ok(value),
        _ => Err(CacheMiss::CorruptRecord),
    }
}

fn bool_field(object: &BTreeMap<String, CanonicalJson>, key: &str) -> Result<bool, CacheMiss> {
    match field(object, key)? {
        CanonicalJson::Bool(value) => Ok(*value),
        _ => Err(CacheMiss::CorruptRecord),
    }
}

fn hash_field(object: &BTreeMap<String, CanonicalJson>, key: &str) -> Result<Hash, CacheMiss> {
    hash_from_hex(string_field(object, key)?).ok_or(CacheMiss::CorruptRecord)
}

fn compatibility_fields(value: &CanonicalJson) -> Result<Vec<CompatibilityField>, CacheMiss> {
    let CanonicalJson::Array(values) = value else {
        return Err(CacheMiss::CorruptRecord);
    };
    values
        .iter()
        .map(|value| {
            let object = object(value)?;
            Ok(CompatibilityField {
                family: string_field(object, "family")?.to_owned(),
                field_name: string_field(object, "field_name")?.to_owned(),
                value: string_field(object, "value")?.to_owned(),
            })
        })
        .collect()
}

fn parse_canonical_json(bytes: &[u8]) -> Result<CanonicalJson, CacheMiss> {
    let text = std::str::from_utf8(bytes).map_err(|_| CacheMiss::CorruptRecord)?;
    let mut parser = JsonParser::new(text);
    let value = parser.parse_artifact()?;
    if canonical_json_bytes(&value) != bytes {
        return Err(CacheMiss::CorruptRecord);
    }
    Ok(value)
}

struct JsonParser<'a> {
    text: &'a str,
    position: usize,
}

impl<'a> JsonParser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, position: 0 }
    }

    fn parse_artifact(&mut self) -> Result<CanonicalJson, CacheMiss> {
        let value = self.parse_value()?;
        self.consume_byte(b'\n')?;
        if self.position != self.text.len() {
            return Err(CacheMiss::CorruptRecord);
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<CanonicalJson, CacheMiss> {
        match self.peek_byte() {
            Some(b'n') => self.parse_literal("null", CanonicalJson::Null),
            Some(b't') => self.parse_literal("true", CanonicalJson::Bool(true)),
            Some(b'f') => self.parse_literal("false", CanonicalJson::Bool(false)),
            Some(b'"') => self.parse_string().map(CanonicalJson::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'-' | b'0'..=b'9') => self.parse_integer(),
            _ => Err(CacheMiss::CorruptRecord),
        }
    }

    fn parse_literal(
        &mut self,
        literal: &str,
        value: CanonicalJson,
    ) -> Result<CanonicalJson, CacheMiss> {
        if self.text[self.position..].starts_with(literal) {
            self.position += literal.len();
            Ok(value)
        } else {
            Err(CacheMiss::CorruptRecord)
        }
    }

    fn parse_array(&mut self) -> Result<CanonicalJson, CacheMiss> {
        self.consume_byte(b'[')?;
        let mut values = Vec::new();
        if self.try_consume_byte(b']') {
            return Ok(CanonicalJson::Array(values));
        }
        loop {
            values.push(self.parse_value()?);
            if self.try_consume_byte(b']') {
                return Ok(CanonicalJson::Array(values));
            }
            self.consume_byte(b',')?;
        }
    }

    fn parse_object(&mut self) -> Result<CanonicalJson, CacheMiss> {
        self.consume_byte(b'{')?;
        let mut fields = BTreeMap::new();
        if self.try_consume_byte(b'}') {
            return Ok(CanonicalJson::Object(fields));
        }
        loop {
            let key = self.parse_string()?;
            self.consume_byte(b':')?;
            let value = self.parse_value()?;
            if fields.insert(key, value).is_some() {
                return Err(CacheMiss::CorruptRecord);
            }
            if self.try_consume_byte(b'}') {
                return Ok(CanonicalJson::Object(fields));
            }
            self.consume_byte(b',')?;
        }
    }

    fn parse_integer(&mut self) -> Result<CanonicalJson, CacheMiss> {
        let start = self.position;
        if self.try_consume_byte(b'-') && !matches!(self.peek_byte(), Some(b'0'..=b'9')) {
            return Err(CacheMiss::CorruptRecord);
        }
        if self.try_consume_byte(b'0') {
            if matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                return Err(CacheMiss::CorruptRecord);
            }
        } else {
            self.consume_digit()?;
            while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                self.position += 1;
            }
        }
        let value = self.text[start..self.position]
            .parse()
            .map_err(|_| CacheMiss::CorruptRecord)?;
        Ok(CanonicalJson::Integer(value))
    }

    fn parse_string(&mut self) -> Result<String, CacheMiss> {
        self.consume_byte(b'"')?;
        let mut value = String::new();
        loop {
            let Some(byte) = self.peek_byte() else {
                return Err(CacheMiss::CorruptRecord);
            };
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(value);
                }
                b'\\' => {
                    self.position += 1;
                    value.push(self.parse_escape()?);
                }
                0x00..=0x1f => return Err(CacheMiss::CorruptRecord),
                _ => {
                    let character = self.text[self.position..]
                        .chars()
                        .next()
                        .ok_or(CacheMiss::CorruptRecord)?;
                    value.push(character);
                    self.position += character.len_utf8();
                }
            }
        }
    }

    fn parse_escape(&mut self) -> Result<char, CacheMiss> {
        let byte = self.peek_byte().ok_or(CacheMiss::CorruptRecord)?;
        self.position += 1;
        match byte {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'b' => Ok('\u{08}'),
            b'f' => Ok('\u{0c}'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'u' => self.parse_control_escape(),
            _ => Err(CacheMiss::CorruptRecord),
        }
    }

    fn parse_control_escape(&mut self) -> Result<char, CacheMiss> {
        let prefix = self
            .text
            .get(self.position..self.position + 2)
            .ok_or(CacheMiss::CorruptRecord)?;
        if prefix != "00" {
            return Err(CacheMiss::CorruptRecord);
        }
        self.position += 2;
        let high = self.parse_hex_digit()?;
        let low = self.parse_hex_digit()?;
        char::from_u32((high << 4) | low).ok_or(CacheMiss::CorruptRecord)
    }

    fn parse_hex_digit(&mut self) -> Result<u32, CacheMiss> {
        let byte = self.peek_byte().ok_or(CacheMiss::CorruptRecord)?;
        self.position += 1;
        match byte {
            b'0'..=b'9' => Ok((byte - b'0') as u32),
            b'a'..=b'f' => Ok((byte - b'a' + 10) as u32),
            _ => Err(CacheMiss::CorruptRecord),
        }
    }

    fn consume_digit(&mut self) -> Result<(), CacheMiss> {
        match self.peek_byte() {
            Some(b'0'..=b'9') => {
                self.position += 1;
                Ok(())
            }
            _ => Err(CacheMiss::CorruptRecord),
        }
    }

    fn consume_byte(&mut self, expected: u8) -> Result<(), CacheMiss> {
        if self.try_consume_byte(expected) {
            Ok(())
        } else {
            Err(CacheMiss::CorruptRecord)
        }
    }

    fn try_consume_byte(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.position).copied()
    }
}

fn output_hash(bytes: &[u8]) -> Hash {
    let mut hasher = blake3::Hasher::new();
    write_hash_part(&mut hasher, OUTPUT_HASH_DOMAIN.as_bytes());
    write_hash_part(&mut hasher, bytes);
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

fn dependency_artifact_hash(domain: &str, bytes: &[u8]) -> Hash {
    let mut hasher = blake3::Hasher::new();
    write_hash_part(&mut hasher, domain.as_bytes());
    write_hash_part(&mut hasher, bytes);
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

fn write_hash_part(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn path_component_for(value: &str) -> String {
    let mut component = String::new();
    for byte in value.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'.' | b'_' | b'-' => {
                component.push(byte as char);
            }
            _ => {
                component.push('%');
                push_hex_byte(&mut component, byte);
            }
        }
    }
    component
}

fn hash_hex(hash: Hash) -> String {
    let mut output = String::with_capacity(Hash::BYTE_LEN * 2);
    for byte in hash.as_bytes() {
        push_hex_byte(&mut output, *byte);
    }
    output
}

fn push_hex_byte(output: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    output.push(HEX[(byte >> 4) as usize] as char);
    output.push(HEX[(byte & 0xf) as usize] as char);
}

fn hash_from_hex(value: &str) -> Option<Hash> {
    if value.len() != Hash::BYTE_LEN * 2 {
        return None;
    }
    let mut bytes = [0; Hash::BYTE_LEN];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = (hex_digit(pair[0])? << 4) | hex_digit(pair[1])?;
    }
    Some(Hash::from_bytes(bytes))
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;
    use crate::cache_key::{
        CacheKeyBuilder, CacheKeyRequest, DependencyArtifactAvailability, DependencyHash,
        FootprintCompleteness, PipelinePhase, PolicyFingerprint, ProofReuseEvidenceIdentity,
        WorkUnit,
    };

    #[test]
    fn record_store_round_trips_inline_output() {
        let root = test_root("round_trip");
        let store = CacheStoreRoot::new(&root);
        let key = key(FootprintCompleteness::Complete, "dev", false);
        let record = CacheRecord::new_inline(
            key.clone(),
            key.validation_inputs.toolchain_compatibility.clone(),
            b"cached output".to_vec(),
        );

        assert_eq!(
            store.insert(&record).expect("insert"),
            CacheInsertOutcome::Inserted
        );
        assert_eq!(
            store.insert(&record).expect("repeat insert"),
            CacheInsertOutcome::AlreadyPresent
        );

        let CacheLookupOutcome::Hit(hit) = store.lookup(&key) else {
            panic!("expected cache hit");
        };
        assert_eq!(hit.header.key, key);
        assert_eq!(hit.output, b"cached output");
        cleanup(root);
    }

    #[test]
    fn conservative_complete_footprint_is_reusable() {
        let root = test_root("conservative_complete");
        let store = CacheStoreRoot::new(&root);
        let key = key(FootprintCompleteness::ConservativeComplete, "dev", false);
        let record = CacheRecord::new_inline(
            key.clone(),
            key.validation_inputs.toolchain_compatibility.clone(),
            b"coarse but complete".to_vec(),
        );

        assert_eq!(
            store.insert(&record).expect("insert"),
            CacheInsertOutcome::Inserted
        );
        assert!(matches!(store.lookup(&key), CacheLookupOutcome::Hit(_)));
        cleanup(root);
    }

    #[test]
    fn incompatible_headers_miss_instead_of_erroring() {
        let root = test_root("incompatible_headers");
        let store = CacheStoreRoot::new(&root);
        let key = key(FootprintCompleteness::Complete, "dev", false);

        let mut unknown_schema = CacheRecord::new_inline(
            key.clone(),
            key.validation_inputs.toolchain_compatibility.clone(),
            b"schema".to_vec(),
        );
        unknown_schema.header.cache_record_schema_version = SchemaVersion::new("future");
        write_raw(&store, &key, encode_record(&unknown_schema));
        assert_eq!(
            store.lookup(&key),
            CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
        );

        let mut unknown_toolchain = CacheRecord::new_inline(
            key.clone(),
            vec![CompatibilityField {
                family: "mizar".to_owned(),
                field_name: "version".to_owned(),
                value: "unknown".to_owned(),
            }],
            b"toolchain".to_vec(),
        );
        unknown_toolchain.header.cache_record_schema_version =
            SchemaVersion::new(CACHE_RECORD_SCHEMA_VERSION);
        write_raw(&store, &key, encode_record(&unknown_toolchain));
        assert_eq!(
            store.lookup(&key),
            CacheLookupOutcome::Miss(CacheMiss::UnknownToolchain)
        );

        let mut key_schema_record = CacheRecord::new_inline(
            key.clone(),
            key.validation_inputs.toolchain_compatibility.clone(),
            b"key schema".to_vec(),
        );
        key_schema_record.header.key.cache_schema_version = SchemaVersion::new("future");
        write_raw(&store, &key, encode_record(&key_schema_record));
        assert_eq!(
            store.lookup(&key),
            CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
        );

        let mut stale_requested_key = key.clone();
        stale_requested_key.cache_schema_version = SchemaVersion::new("future");
        let stale_requested_record = CacheRecord::new_inline(
            stale_requested_key.clone(),
            stale_requested_key
                .validation_inputs
                .toolchain_compatibility
                .clone(),
            b"stale key".to_vec(),
        );
        write_raw(
            &store,
            &stale_requested_key,
            encode_record(&stale_requested_record),
        );
        assert_eq!(
            store.lookup(&stale_requested_key),
            CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
        );

        let mut policy_key = key.clone();
        policy_key.validation_inputs.policy_compatibility[0].value = "incompatible".to_owned();
        let policy_record = CacheRecord::new_inline(
            policy_key.clone(),
            policy_key.validation_inputs.toolchain_compatibility.clone(),
            b"policy".to_vec(),
        );
        write_raw(&store, &policy_key, encode_record(&policy_record));
        assert_eq!(
            store.lookup(&policy_key),
            CacheLookupOutcome::Miss(CacheMiss::PolicyIncompatible)
        );

        let mut output_schema_key = key.clone();
        output_schema_key.schema_versions[0].version = SchemaVersion::new("unsupported");
        let output_schema_record = CacheRecord::new_inline(
            output_schema_key.clone(),
            output_schema_key
                .validation_inputs
                .toolchain_compatibility
                .clone(),
            b"output schema".to_vec(),
        );
        write_raw(
            &store,
            &output_schema_key,
            encode_record(&output_schema_record),
        );
        assert_eq!(
            store.lookup(&output_schema_key),
            CacheLookupOutcome::Miss(CacheMiss::UnknownSchema)
        );
        cleanup(root);
    }

    #[test]
    fn uncacheable_and_incomplete_records_miss_without_disk_trust() {
        let root = test_root("uncacheable");
        let store = CacheStoreRoot::new(&root);
        let base_key = key(FootprintCompleteness::Complete, "dev", false);
        let mut record = CacheRecord::new_inline(
            base_key.clone(),
            base_key.validation_inputs.toolchain_compatibility.clone(),
            b"uncacheable".to_vec(),
        );
        record.header.uncacheable = true;
        write_raw(&store, &base_key, encode_record(&record));
        assert_eq!(
            store.lookup(&base_key),
            CacheLookupOutcome::Miss(CacheMiss::Uncacheable)
        );

        let incomplete = key(FootprintCompleteness::IncompleteUncacheable, "dev", false);
        assert_eq!(
            store.lookup(&incomplete),
            CacheLookupOutcome::Miss(CacheMiss::IncompleteFootprint)
        );

        let unsupported = key(FootprintCompleteness::Unsupported, "dev", false);
        assert_eq!(
            store.lookup(&unsupported),
            CacheLookupOutcome::Miss(CacheMiss::UnsupportedFootprint)
        );

        let outcome = CacheKeyBuilder::new(request(
            FootprintCompleteness::IncompleteUncacheable,
            "dev",
            false,
        ))
        .build();
        assert_eq!(
            store.lookup_key_outcome(&outcome),
            CacheLookupOutcome::Miss(CacheMiss::Uncacheable)
        );
        cleanup(root);
    }

    #[test]
    fn missing_dependency_and_proof_validation_inputs_miss_fail_closed() {
        let root = test_root("missing_validation");
        let store = CacheStoreRoot::new(&root);

        let mut dependency_key = key(FootprintCompleteness::Complete, "dev", false);
        dependency_key.validation_inputs.dependency_artifacts.push(
            DependencyArtifactAvailability {
                package_id: "dep".to_owned(),
                module_path: "dep.alpha".to_owned(),
                artifact_kind: "mizir".to_owned(),
                artifact_path: "build/missing.mizir.json".to_owned(),
                domain: "artifact".to_owned(),
                digest: hash(80),
            },
        );
        let dependency_record = CacheRecord::new_inline(
            dependency_key.clone(),
            dependency_key
                .validation_inputs
                .toolchain_compatibility
                .clone(),
            b"dependency".to_vec(),
        );
        write_raw(&store, &dependency_key, encode_record(&dependency_record));
        assert_eq!(
            store.lookup(&dependency_key),
            CacheLookupOutcome::Miss(CacheMiss::DependencyUnavailable)
        );

        let mut mismatched_dependency_key = key(FootprintCompleteness::Complete, "dev", true);
        let mismatched_artifact_path = format!(
            "{}/deps/mismatch.mizir.json",
            root.file_name()
                .and_then(|name| name.to_str())
                .expect("test root has utf-8 file name")
        );
        let mismatched_artifact_on_disk = root.join("deps/mismatch.mizir.json");
        fs::create_dir_all(
            mismatched_artifact_on_disk
                .parent()
                .expect("artifact path has parent"),
        )
        .expect("mkdir dependency artifact parent");
        fs::write(&mismatched_artifact_on_disk, b"actual artifact bytes")
            .expect("write mismatched dependency artifact");
        mismatched_dependency_key
            .validation_inputs
            .dependency_artifacts
            .push(DependencyArtifactAvailability {
                package_id: "dep".to_owned(),
                module_path: "dep.beta".to_owned(),
                artifact_kind: "mizir".to_owned(),
                artifact_path: mismatched_artifact_path,
                domain: "artifact".to_owned(),
                digest: dependency_artifact_hash("artifact", b"expected artifact bytes"),
            });
        let mismatched_dependency_record = CacheRecord::new_inline(
            mismatched_dependency_key.clone(),
            mismatched_dependency_key
                .validation_inputs
                .toolchain_compatibility
                .clone(),
            b"mismatched dependency".to_vec(),
        );
        write_raw(
            &store,
            &mismatched_dependency_key,
            encode_record(&mismatched_dependency_record),
        );
        assert_eq!(
            store.lookup(&mismatched_dependency_key),
            CacheLookupOutcome::Miss(CacheMiss::DependencyUnavailable)
        );

        let mut proof_key = key(FootprintCompleteness::Complete, "dev", true);
        proof_key.validation_inputs.obligation_anchor_fingerprint = Some(hash(81));
        proof_key.validation_inputs.proof_reuse_schema_versions = vec![NamedSchemaVersion {
            schema_family: "proof-reuse".to_owned(),
            name: "metadata".to_owned(),
            version: SchemaVersion::new("1.0"),
        }];
        proof_key
            .validation_inputs
            .proof_reuse_evidence_identities
            .push(ProofReuseEvidenceIdentity {
                obligation_anchor_fingerprint: hash(81),
                evidence_kind: "kernel_verified".to_owned(),
                witness_or_discharge_domain: "witness".to_owned(),
                witness_or_discharge_digest: hash(82),
            });
        proof_key.validation_inputs.selected_proof_witness_hash = Some(named("witness", 82));
        proof_key.validation_inputs.proof_reuse_validation_hash = None;
        let proof_record = CacheRecord::new_inline(
            proof_key.clone(),
            proof_key.validation_inputs.toolchain_compatibility.clone(),
            b"proof".to_vec(),
        );
        write_raw(&store, &proof_key, encode_record(&proof_record));
        assert_eq!(
            store.lookup(&proof_key),
            CacheLookupOutcome::Miss(CacheMiss::ProofReuseInvalid)
        );

        let mut proof_phase_key = key(FootprintCompleteness::Complete, "dev", false);
        proof_phase_key.phase = PipelinePhase::new("proof");
        let proof_phase_record = CacheRecord::new_inline(
            proof_phase_key.clone(),
            proof_phase_key
                .validation_inputs
                .toolchain_compatibility
                .clone(),
            b"proof phase".to_vec(),
        );
        write_raw(&store, &proof_phase_key, encode_record(&proof_phase_record));
        assert_eq!(
            store.lookup(&proof_phase_key),
            CacheLookupOutcome::Miss(CacheMiss::ProofReuseInvalid)
        );

        let mut proof_phase_with_vc_key = key(FootprintCompleteness::Complete, "dev", true);
        proof_phase_with_vc_key.phase = PipelinePhase::new("proof");
        proof_phase_with_vc_key
            .validation_inputs
            .canonical_vc_fingerprint = Some(hash(83));
        proof_phase_with_vc_key
            .validation_inputs
            .local_context_fingerprint = Some(hash(84));
        proof_phase_with_vc_key
            .validation_inputs
            .dependency_slice_fingerprints
            .push(DependencySliceHash {
                slice_kind: "theorem".to_owned(),
                owner: "dep.alpha".to_owned(),
                name: "T1".to_owned(),
                domain: "domain/dependency-slice".to_owned(),
                digest: hash(85),
            });
        let proof_phase_with_vc_record = CacheRecord::new_inline(
            proof_phase_with_vc_key.clone(),
            proof_phase_with_vc_key
                .validation_inputs
                .toolchain_compatibility
                .clone(),
            b"proof phase with vc".to_vec(),
        );
        write_raw(
            &store,
            &proof_phase_with_vc_key,
            encode_record(&proof_phase_with_vc_record),
        );
        assert_eq!(
            store.lookup(&proof_phase_with_vc_key),
            CacheLookupOutcome::Miss(CacheMiss::ProofReuseInvalid)
        );
        cleanup(root);
    }

    #[test]
    fn corrupted_records_and_output_hash_mismatches_are_misses() {
        let root = test_root("corrupt");
        let store = CacheStoreRoot::new(&root);
        let key = key(FootprintCompleteness::Complete, "dev", false);
        write_raw(&store, &key, b"not a record".to_vec());
        assert_eq!(
            store.lookup(&key),
            CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
        );

        write_raw(&store, &key, encode_raw_record(b"{}\n", b"payload"));
        assert_eq!(
            store.lookup(&key),
            CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
        );

        write_raw(
            &store,
            &key,
            encode_raw_record(b"{\"a\":1,\"a\":2}\n", b"payload"),
        );
        assert_eq!(
            store.lookup(&key),
            CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
        );

        let record = CacheRecord::new_inline(
            key.clone(),
            key.validation_inputs.toolchain_compatibility.clone(),
            b"payload".to_vec(),
        );
        let mut bytes = encode_record(&record);
        let last = bytes.last_mut().expect("encoded record has payload byte");
        *last ^= 0xff;
        write_raw(&store, &key, bytes);
        assert_eq!(
            store.lookup(&key),
            CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
        );

        let mut truncated = encode_record(&record);
        truncated.pop();
        write_raw(&store, &key, truncated);
        assert_eq!(
            store.lookup(&key),
            CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
        );
        cleanup(root);
    }

    #[test]
    fn exact_key_path_still_validates_embedded_key_and_diagnostic_refs() {
        let root = test_root("embedded_key");
        let store = CacheStoreRoot::new(&root);
        let requested = key(FootprintCompleteness::Complete, "dev", false);
        let embedded = key(FootprintCompleteness::Complete, "dev", true);
        let mismatched = CacheRecord::new_inline(
            embedded.clone(),
            embedded.validation_inputs.toolchain_compatibility.clone(),
            b"wrong key".to_vec(),
        );
        write_raw(&store, &requested, encode_record(&mismatched));
        assert_eq!(
            store.lookup(&requested),
            CacheLookupOutcome::Miss(CacheMiss::CorruptRecord)
        );
        fs::remove_file(store.record_path(&requested)).expect("remove mismatched raw record");

        let mut record = CacheRecord::new_inline(
            requested.clone(),
            requested.validation_inputs.toolchain_compatibility.clone(),
            b"diagnostic".to_vec(),
        );
        record.header.diagnostic_refs.push(DiagnosticRefHash {
            diagnostic_ref_kind: "cache_miss_explanation".to_owned(),
            diagnostic_ref_hash: hash(90),
        });
        store.insert(&record).expect("insert diagnostic record");
        let CacheLookupOutcome::Hit(hit) = store.lookup(&requested) else {
            panic!("expected hit");
        };
        assert_eq!(hit.header.diagnostic_refs, record.header.diagnostic_refs);
        cleanup(root);
    }

    #[test]
    fn record_write_order_does_not_change_lookup_result() {
        let root = test_root("write_order");
        let store = CacheStoreRoot::new(&root);
        let first = key(FootprintCompleteness::Complete, "dev", false);
        let second = key(FootprintCompleteness::Complete, "dev", true);
        let first_record = CacheRecord::new_inline(
            first.clone(),
            first.validation_inputs.toolchain_compatibility.clone(),
            b"first".to_vec(),
        );
        let second_record = CacheRecord::new_inline(
            second.clone(),
            second.validation_inputs.toolchain_compatibility.clone(),
            b"second".to_vec(),
        );

        store.insert(&second_record).expect("insert second");
        store.insert(&first_record).expect("insert first");

        let CacheLookupOutcome::Hit(hit) = store.lookup(&first) else {
            panic!("expected first hit");
        };
        assert_eq!(hit.output, b"first");
        let CacheLookupOutcome::Hit(hit) = store.lookup(&second) else {
            panic!("expected second hit");
        };
        assert_eq!(hit.output, b"second");
        cleanup(root);
    }

    #[test]
    fn divergent_same_key_insert_loses_without_overwriting_existing_record() {
        let root = test_root("divergent_insert");
        let store = CacheStoreRoot::new(&root);
        let key = key(FootprintCompleteness::Complete, "dev", false);
        let original = CacheRecord::new_inline(
            key.clone(),
            key.validation_inputs.toolchain_compatibility.clone(),
            b"original".to_vec(),
        );
        let divergent = CacheRecord::new_inline(
            key.clone(),
            key.validation_inputs.toolchain_compatibility.clone(),
            b"divergent".to_vec(),
        );

        assert_eq!(
            store.insert(&original).expect("insert original"),
            CacheInsertOutcome::Inserted
        );
        assert!(matches!(
            store.insert(&divergent),
            Err(CacheStoreError::DivergentRecord { .. })
        ));

        let CacheLookupOutcome::Hit(hit) = store.lookup(&key) else {
            panic!("expected original hit");
        };
        assert_eq!(hit.output, b"original");
        cleanup(root);
    }

    fn key(
        completeness: FootprintCompleteness,
        toolchain_value: &str,
        alternate: bool,
    ) -> CacheKey {
        match CacheKeyBuilder::new(request(completeness, toolchain_value, alternate)).build() {
            CacheKeyBuildOutcome::Cacheable(key) | CacheKeyBuildOutcome::Uncacheable(key) => key,
            CacheKeyBuildOutcome::NoKey(rejection) => panic!("{rejection}"),
        }
    }

    fn request(
        completeness: FootprintCompleteness,
        toolchain_value: &str,
        alternate: bool,
    ) -> CacheKeyRequest {
        CacheKeyRequest {
            cache_schema_version: SchemaVersion::default(),
            phase: PipelinePhase::new("resolve"),
            work_unit: WorkUnit::new(if alternate {
                "module:alpha#resolve:2"
            } else {
                "module:alpha#resolve:1"
            }),
            source_identity: None,
            input_hashes: vec![named("source", if alternate { 42 } else { 2 })],
            dependency_hashes: vec![DependencyHash {
                dependency_kind: "interface".to_owned(),
                package_id: "dep".to_owned(),
                module_path: "dep.alpha".to_owned(),
                name: "artifact".to_owned(),
                domain: "domain/interface".to_owned(),
                digest: hash(3),
            }],
            dependency_slices: Vec::new(),
            config_hash: hash(4),
            schema_versions: vec![NamedSchemaVersion {
                schema_family: "resolve".to_owned(),
                name: "output".to_owned(),
                version: SchemaVersion::new("1.0"),
            }],
            policy_fingerprint: PolicyFingerprint::new(hash(5)),
            validation_inputs: validation_inputs(completeness, toolchain_value),
        }
    }

    fn validation_inputs(
        completeness: FootprintCompleteness,
        toolchain_value: &str,
    ) -> crate::cache_key::CacheValidationInputs {
        crate::cache_key::CacheValidationInputs {
            cache_schema_compatibility: SchemaVersion::default(),
            toolchain_compatibility: vec![CompatibilityField {
                family: "mizar".to_owned(),
                field_name: "version".to_owned(),
                value: toolchain_value.to_owned(),
            }],
            dependency_artifacts: Vec::new(),
            footprint_completeness: completeness,
            uncacheable: false,
            policy_compatibility: vec![CompatibilityField {
                family: "proof-policy".to_owned(),
                field_name: "require_kernel_certificates".to_owned(),
                value: "true".to_owned(),
            }],
            canonical_vc_fingerprint: None,
            local_context_fingerprint: None,
            dependency_slice_fingerprints: Vec::new(),
            obligation_anchor_fingerprint: None,
            selected_proof_witness_hash: None,
            deterministic_discharge_hash: None,
            proof_reuse_schema_versions: Vec::new(),
            proof_reuse_validation_hash: None,
            proof_reuse_evidence_identities: Vec::new(),
            diagnostic_refs: Vec::new(),
        }
    }

    fn named(name: &str, seed: u8) -> NamedHash {
        NamedHash {
            name: name.to_owned(),
            domain: format!("domain/{name}"),
            digest: hash(seed),
        }
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }

    fn write_raw(store: &CacheStoreRoot, key: &CacheKey, bytes: Vec<u8>) {
        let path = store.record_path(key);
        fs::create_dir_all(path.parent().expect("record path has parent")).expect("mkdir");
        fs::write(path, bytes).expect("write raw record");
    }

    fn encode_raw_record(header: &[u8], payload: &[u8]) -> Vec<u8> {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(CACHE_RECORD_MAGIC);
        encoded.extend_from_slice(&RECORD_FORMAT_VERSION.to_le_bytes());
        encoded.extend_from_slice(&(header.len() as u64).to_le_bytes());
        encoded.extend_from_slice(header);
        encoded.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        encoded.extend_from_slice(payload);
        encoded
    }

    fn test_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("mizar-cache-store-{name}-{}", std::process::id()));
        cleanup(root.clone());
        root
    }

    fn cleanup(root: PathBuf) {
        let _ = fs::remove_dir_all(root);
    }
}
