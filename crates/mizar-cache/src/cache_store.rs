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
    sync::atomic::{AtomicU64, Ordering},
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
/// Supported content-addressed blob hash family.
pub const CACHE_BLOB_HASH_FAMILY: &str = "blake3";

const RECORD_FORMAT_VERSION: u32 = 1;
const OUTPUT_HASH_DOMAIN: &str = "mizar-cache/cache-record-output/v1";
const RECORD_EXTENSION: &str = "mcr";
const TEMP_FILE_ATTEMPTS: usize = 32;
static TEMP_FILE_NONCE: AtomicU64 = AtomicU64::new(0);

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
    /// Output bytes are stored in the content-addressed blob store.
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

/// Cache record with validated output bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheRecord {
    /// Record header.
    pub header: CacheRecordHeader,
    /// Validated output bytes. On-disk blob records encode an empty payload.
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
    /// A different record or blob already exists for the same validated identity.
    DivergentRecord {
        /// Existing record or blob path.
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

    /// Returns the deterministic blob path for `blob`.
    pub fn blob_path(&self, blob: &CacheBlobRef) -> PathBuf {
        self.root
            .join("blobs")
            .join(path_component_for(&blob.hash_family))
            .join(path_component_for(&blob.digest))
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

        if matches!(record.header.output, CacheOutputDescriptor::Blob { .. }) {
            let written_blob = self.write_blob(&record.output)?;
            if Some(&written_blob) != record.header.output.blob_ref() {
                return Err(CacheStoreError::InvalidRecord {
                    reason: CacheMiss::CorruptRecord,
                });
            }
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

    /// Writes `bytes` into the content-addressed blob store and returns its reference.
    pub fn write_blob(&self, bytes: &[u8]) -> Result<CacheBlobRef, CacheStoreError> {
        let blob = CacheBlobRef::for_output(bytes);
        let path = self.blob_path(&blob);
        if let Ok(existing) = fs::read(&path) {
            if existing == bytes {
                return Ok(blob);
            }
            return Err(CacheStoreError::DivergentRecord { path });
        }

        let parent = path
            .parent()
            .expect("blob path always has parent directories");
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

        self.write_blob_via_unique_temp(&blob, &path, &temporary_dir, bytes)?;
        Ok(blob)
    }

    fn write_via_unique_temp(
        &self,
        path: &Path,
        temporary_dir: &Path,
        bytes: &[u8],
        record: &CacheRecord,
    ) -> Result<CacheInsertOutcome, CacheStoreError> {
        for attempt in 0..TEMP_FILE_ATTEMPTS {
            let nonce = temporary_nonce(bytes);
            let temporary = temporary_dir.join(format!(
                "{}-{}-{nonce}-{attempt}.tmp",
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

    fn write_blob_via_unique_temp(
        &self,
        blob: &CacheBlobRef,
        path: &Path,
        temporary_dir: &Path,
        bytes: &[u8],
    ) -> Result<(), CacheStoreError> {
        for attempt in 0..TEMP_FILE_ATTEMPTS {
            if let Ok(existing) = fs::read(path)
                && existing == bytes
            {
                return Ok(());
            }
            let nonce = temporary_nonce(bytes);
            let temporary = temporary_dir.join(format!(
                "{}-{}-{nonce}-{attempt}.blob.tmp",
                path_component_for(&blob.hash_family),
                blob.digest
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

            if !blob_matches_bytes(blob, bytes)
                || fs::read(&temporary)
                    .map(|written| written != bytes || !blob_matches_bytes(blob, &written))
                    .unwrap_or(true)
            {
                let _ = fs::remove_file(&temporary);
                return Err(CacheStoreError::InvalidRecord {
                    reason: CacheMiss::CorruptRecord,
                });
            }

            match fs::hard_link(&temporary, path) {
                Ok(()) => {
                    let _ = fs::remove_file(&temporary);
                    return Ok(());
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    let _ = fs::remove_file(&temporary);
                    if let Ok(existing) = fs::read(path)
                        && existing == bytes
                    {
                        return Ok(());
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

        if let Ok(existing) = fs::read(path)
            && existing == bytes
        {
            return Ok(());
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

    /// Creates a blob-output header.
    pub fn new_blob(key: CacheKey, produced_by: Vec<CompatibilityField>, output: &[u8]) -> Self {
        Self {
            cache_record_schema_version: SchemaVersion::new(CACHE_RECORD_SCHEMA_VERSION),
            key,
            produced_by,
            output: CacheOutputDescriptor::blob(output),
            uncacheable: false,
            diagnostic_refs: Vec::new(),
        }
    }
}

impl CacheBlobRef {
    /// Creates the supported content-addressed blob reference for `output`.
    pub fn for_output(output: &[u8]) -> Self {
        Self {
            hash_family: CACHE_BLOB_HASH_FAMILY.to_owned(),
            digest: hash_hex(output_hash(output)),
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

    /// Creates a blob descriptor for `output`.
    pub fn blob(output: &[u8]) -> Self {
        Self::Blob {
            blob: CacheBlobRef::for_output(output),
            output_hash: output_hash(output),
            byte_len: output.len() as u64,
        }
    }

    fn blob_ref(&self) -> Option<&CacheBlobRef> {
        match self {
            Self::Inline { .. } => None,
            Self::Blob { blob, .. } => Some(blob),
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

    /// Creates a blob-backed cache record.
    pub fn new_blob(
        key: CacheKey,
        produced_by: Vec<CompatibilityField>,
        output: impl Into<Vec<u8>>,
    ) -> Self {
        let output = output.into();
        Self {
            header: CacheRecordHeader::new_blob(key, produced_by, &output),
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
                "divergent cache record or blob already exists at `{}`",
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
    match &record.header.output {
        CacheOutputDescriptor::Inline { .. } => {
            if record.header.output.byte_len() != record.output.len() as u64
                || record.header.output.output_hash() != output_hash(&record.output)
            {
                return Some(CacheMiss::CorruptRecord);
            }
        }
        CacheOutputDescriptor::Blob {
            blob,
            output_hash: expected_output_hash,
            byte_len,
        } => {
            if !blob_ref_supported(blob)
                || blob.digest != hash_hex(*expected_output_hash)
                || *byte_len != record.output.len() as u64
                || *expected_output_hash != output_hash(&record.output)
            {
                return Some(CacheMiss::CorruptRecord);
            }
        }
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

    let (output_descriptor, output) = match decoded.output {
        DecodedOutput::Inline {
            output_hash,
            byte_len,
        } => (
            CacheOutputDescriptor::Inline {
                output_hash,
                byte_len,
            },
            output,
        ),
        DecodedOutput::Blob {
            blob,
            output_hash: expected_output_hash,
            byte_len,
        } => {
            if !output.is_empty() {
                return Err(CacheMiss::CorruptRecord);
            }
            let bytes = read_blob(root, &blob, expected_output_hash, byte_len)?;
            (
                CacheOutputDescriptor::Blob {
                    blob,
                    output_hash: expected_output_hash,
                    byte_len,
                },
                bytes,
            )
        }
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

fn read_blob(
    root: &Path,
    blob: &CacheBlobRef,
    expected_output_hash: Hash,
    expected_byte_len: u64,
) -> Result<Vec<u8>, CacheMiss> {
    if !blob_ref_supported(blob) {
        return Err(CacheMiss::UnknownSchema);
    }
    if blob.digest != hash_hex(expected_output_hash) {
        return Err(CacheMiss::CorruptRecord);
    }
    let bytes = fs::read(blob_path(root, blob)).map_err(|_| CacheMiss::CorruptRecord)?;
    if bytes.len() as u64 != expected_byte_len || output_hash(&bytes) != expected_output_hash {
        return Err(CacheMiss::CorruptRecord);
    }
    Ok(bytes)
}

fn blob_path(root: &Path, blob: &CacheBlobRef) -> PathBuf {
    root.join("blobs")
        .join(path_component_for(&blob.hash_family))
        .join(path_component_for(&blob.digest))
}

fn blob_ref_supported(blob: &CacheBlobRef) -> bool {
    blob.hash_family == CACHE_BLOB_HASH_FAMILY && hash_from_hex(&blob.digest).is_some()
}

fn blob_matches_bytes(blob: &CacheBlobRef, bytes: &[u8]) -> bool {
    blob_ref_supported(blob) && blob.digest == hash_hex(output_hash(bytes))
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
    let payload = record_payload(record);
    let mut encoded =
        Vec::with_capacity(CACHE_RECORD_MAGIC.len() + 4 + 8 + header.len() + 8 + payload.len());
    encoded.extend_from_slice(CACHE_RECORD_MAGIC);
    encoded.extend_from_slice(&RECORD_FORMAT_VERSION.to_le_bytes());
    encoded.extend_from_slice(&(header.len() as u64).to_le_bytes());
    encoded.extend_from_slice(&header);
    encoded.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    encoded.extend_from_slice(payload);
    encoded
}

fn record_payload(record: &CacheRecord) -> &[u8] {
    match record.header.output {
        CacheOutputDescriptor::Inline { .. } => &record.output,
        CacheOutputDescriptor::Blob { .. } => &[],
    }
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
    Inline {
        output_hash: Hash,
        byte_len: u64,
    },
    Blob {
        blob: CacheBlobRef,
        output_hash: Hash,
        byte_len: u64,
    },
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
        "blob" => DecodedOutput::Blob {
            blob: CacheBlobRef {
                hash_family: string_field(output_object, "hash_family")?.to_owned(),
                digest: string_field(output_object, "blob_digest")?.to_owned(),
            },
            output_hash,
            byte_len,
        },
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

fn temporary_nonce(bytes: &[u8]) -> String {
    format!(
        "{:016x}-{:016x}",
        TEMP_FILE_NONCE.fetch_add(1, Ordering::Relaxed),
        bytes.as_ptr() as usize
    )
}

fn path_component_for(value: &str) -> String {
    let mut component = String::new();
    for byte in value.bytes() {
        match byte {
            b'.' if value != "." && value != ".." => component.push('.'),
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' => {
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
mod tests;
