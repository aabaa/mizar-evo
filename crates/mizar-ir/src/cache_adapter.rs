//! Cache-record adapter for sealed IR handles.
//!
//! This module is specified in
//! [`cache_adapter.md`](../../../../doc/design/mizar-ir/en/cache_adapter.md).

use std::{
    error::Error,
    fmt,
    sync::{Arc, Mutex},
};

use mizar_cache::{
    cache_key::{CacheKey, CompatibilityField},
    cache_store::{CacheLookupOutcome, CacheMiss, CacheRecord},
};
use mizar_session::{BuildSnapshotId, Hash};

use crate::{
    identity::{NamedInputHash, OutputKind, PhaseOutputId, PipelinePhase, WorkUnit},
    publisher::{
        OutputOrigin, ParentHashSummary, PhaseOutputPublisher, PublicationTarget, PublishError,
        PublishOutputInput, canonical_parent_summaries, content_hash_from_parent_summaries,
        side_table_hash,
    },
    storage::{
        AnyPhaseOutputRef, BlobDecodeError, BlobDecoder, IrSideTables, PhaseOutputRef,
        SchemaVersion, SideTableRecord, StorageError,
    },
};

const PAYLOAD_MAGIC: &[u8] = b"MIZAR-IR-CACHE-PAYLOAD\0";
const PAYLOAD_FORMAT_VERSION: u32 = 1;

/// Cache adapter over a phase-output publisher.
#[derive(Debug)]
pub struct IrCacheAdapter {
    publisher: Arc<PhaseOutputPublisher>,
    state: Mutex<CacheAdapterState>,
}

#[derive(Debug, Default)]
struct CacheAdapterState {
    successful_rehydrations: u64,
}

/// Cacheability classification consumed by `mizar-ir`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheAdapterCacheability {
    /// The cache owner has supplied a complete, cacheable context.
    Cacheable,
    /// The output is valid IR but should not be cached.
    Skip,
    /// This output kind/schema is not supported by the adapter.
    Incompatible,
}

/// Input for converting a sealed handle to a cache record.
pub struct EncodeCacheRecordInput<T> {
    /// Sealed output handle.
    pub handle: PhaseOutputRef<T>,
    /// Cache key supplied by `mizar-cache`.
    pub cache_key: CacheKey,
    /// Toolchain/producer compatibility fields supplied by the cache owner.
    pub produced_by: Vec<CompatibilityField>,
    /// Canonical payload bytes for this output.
    pub canonical_payload: Vec<u8>,
    /// Cacheability decision from the producer/cache boundary.
    pub cacheability: CacheAdapterCacheability,
}

/// Result of cache-record encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EncodeCacheRecordOutcome {
    /// A cache record was produced for the cache owner.
    Encoded(Box<CacheRecord>),
    /// Encoding was skipped fail-closed.
    Skipped(CacheAdapterMiss),
}

/// Input for rehydrating a validated cache hit.
pub struct RehydrateCacheHitInput<T> {
    /// Lookup result from `mizar-cache`.
    pub lookup: CacheLookupOutcome,
    /// Target current snapshot.
    pub snapshot: BuildSnapshotId,
    /// Target phase.
    pub phase: PipelinePhase,
    /// Target work unit.
    pub work_unit: WorkUnit,
    /// Target output kind.
    pub output_kind: OutputKind,
    /// Target schema version.
    pub schema_version: SchemaVersion,
    /// Current-snapshot parent handles.
    pub parents: Vec<AnyPhaseOutputRef>,
    /// Current-snapshot named non-output input hashes.
    pub named_input_hashes: Vec<NamedInputHash>,
    /// Decoder for the canonical payload bytes.
    pub decode: BlobDecoder<T>,
}

/// Result of rehydrating a cache hit.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheRehydrateOutcome<T> {
    /// A new sealed current-snapshot handle was created.
    Rehydrated(Box<PhaseOutputRef<T>>),
    /// Cache data failed closed before handle exposure.
    Miss(CacheAdapterMiss),
}

/// Fail-closed cache-adapter miss reason.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CacheAdapterMiss {
    /// Miss supplied by `mizar-cache`.
    Cache(CacheMiss),
    /// The caller asked the adapter to skip encoding.
    Skipped,
    /// Unsupported output kind or schema.
    Incompatible,
    /// Cache payload bytes are malformed.
    CorruptRecord,
    /// Canonical payload bytes do not match the recorded content hash.
    PayloadHashMismatch,
    /// Side-table records do not match the recorded side-table hash.
    SideTableHashMismatch,
    /// Current parent handles do not match the cache record parent summaries.
    ParentMismatch,
    /// A parent lineage was missing while encoding a sealed output.
    MissingParentLineage {
        /// Missing parent output id.
        parent: PhaseOutputId,
    },
    /// Storage rejected an input before handle exposure.
    Storage {
        /// Storage error.
        error: Box<StorageError>,
    },
    /// Publisher rejected the validated cache input.
    Publisher {
        /// Publisher error.
        error: Box<PublishError>,
    },
    /// Cache bytes could not be decoded into the requested typed payload.
    Decode {
        /// Decoder message.
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CachedIrRecordPayload {
    output_kind: OutputKind,
    schema_version: SchemaVersion,
    content_hash: Hash,
    side_table_hash: Hash,
    parent_summaries: Vec<ParentHashSummary>,
    named_input_hashes: Vec<NamedInputHash>,
    side_tables: IrSideTables,
    canonical_payload: Vec<u8>,
}

impl IrCacheAdapter {
    /// Creates a cache adapter over a publisher.
    pub fn new(publisher: Arc<PhaseOutputPublisher>) -> Self {
        Self {
            publisher,
            state: Mutex::default(),
        }
    }

    /// Returns the wrapped publisher.
    pub fn publisher(&self) -> &Arc<PhaseOutputPublisher> {
        &self.publisher
    }

    /// Returns the number of successful rehydrations in this process.
    pub fn successful_rehydrations(&self) -> u64 {
        self.state
            .lock()
            .expect("cache adapter mutex poisoned")
            .successful_rehydrations
    }

    /// Encodes a sealed output as an internal cache record.
    pub fn encode<T>(&self, input: EncodeCacheRecordInput<T>) -> EncodeCacheRecordOutcome
    where
        T: Send + Sync + 'static,
    {
        match input.cacheability {
            CacheAdapterCacheability::Cacheable => {}
            CacheAdapterCacheability::Skip => {
                return EncodeCacheRecordOutcome::Skipped(CacheAdapterMiss::Skipped);
            }
            CacheAdapterCacheability::Incompatible => {
                return EncodeCacheRecordOutcome::Skipped(CacheAdapterMiss::Incompatible);
            }
        }

        if let Err(error) = self.publisher.storage().validate_handle(input.handle.any()) {
            return EncodeCacheRecordOutcome::Skipped(CacheAdapterMiss::Storage {
                error: Box::new(error),
            });
        }

        let side_tables = match self.publisher.storage().side_tables(&input.handle) {
            Ok(side_tables) => (*side_tables).clone(),
            Err(error) => {
                return EncodeCacheRecordOutcome::Skipped(CacheAdapterMiss::Storage {
                    error: Box::new(error),
                });
            }
        };

        let payload =
            match self.payload_from_handle(&input.handle, input.canonical_payload, side_tables) {
                Ok(payload) => payload,
                Err(miss) => return EncodeCacheRecordOutcome::Skipped(miss),
            };
        let bytes = encode_cached_payload(&payload);
        EncodeCacheRecordOutcome::Encoded(Box::new(CacheRecord::new_inline(
            input.cache_key,
            input.produced_by,
            bytes,
        )))
    }

    /// Rehydrates a sealed current-snapshot handle from a validated cache hit.
    pub fn rehydrate<T>(&self, input: RehydrateCacheHitInput<T>) -> CacheRehydrateOutcome<T>
    where
        T: Send + Sync + 'static,
    {
        let record = match input.lookup {
            CacheLookupOutcome::Hit(record) => record,
            CacheLookupOutcome::Miss(miss) => {
                return CacheRehydrateOutcome::Miss(CacheAdapterMiss::Cache(miss));
            }
            _ => return CacheRehydrateOutcome::Miss(CacheAdapterMiss::Incompatible),
        };

        let cached = match decode_cached_payload(&record.output) {
            Ok(payload) => payload,
            Err(()) => return CacheRehydrateOutcome::Miss(CacheAdapterMiss::CorruptRecord),
        };
        if cached.output_kind != input.output_kind || cached.schema_version != input.schema_version
        {
            return CacheRehydrateOutcome::Miss(CacheAdapterMiss::Incompatible);
        }

        if let Err(miss) =
            self.validate_rehydration_payload(input.snapshot, &cached, &input.parents)
        {
            return CacheRehydrateOutcome::Miss(miss);
        }

        let parent_summaries = canonical_parent_summaries(&input.parents);
        if parent_summaries != cached.parent_summaries {
            return CacheRehydrateOutcome::Miss(CacheAdapterMiss::ParentMismatch);
        }

        if !content_hash_matches(
            &cached.canonical_payload,
            &cached.parent_summaries,
            cached.named_input_hashes.clone(),
            cached.content_hash,
        ) || !content_hash_matches(
            &cached.canonical_payload,
            &parent_summaries,
            input.named_input_hashes.clone(),
            cached.content_hash,
        ) {
            return CacheRehydrateOutcome::Miss(CacheAdapterMiss::PayloadHashMismatch);
        }

        let canonical_payload = cached.canonical_payload;
        let payload = match decode_payload(&input.decode, &canonical_payload) {
            Ok(payload) => payload,
            Err(error) => {
                return CacheRehydrateOutcome::Miss(CacheAdapterMiss::Decode {
                    message: error.message().to_owned(),
                });
            }
        };
        let slot = self.publisher.allocate::<T>(
            input.snapshot,
            input.phase.clone(),
            input.work_unit.clone(),
            input.output_kind.clone(),
            input.schema_version,
        );

        match self.publisher.publish(PublishOutputInput {
            slot,
            snapshot: input.snapshot,
            phase: input.phase,
            work_unit: input.work_unit,
            output_kind: input.output_kind,
            schema_version: input.schema_version,
            payload,
            canonical_payload: Some(canonical_payload),
            decode: input.decode,
            parents: input.parents,
            named_input_hashes: input.named_input_hashes,
            side_tables: cached.side_tables,
            origin: OutputOrigin::ValidatedCacheInput,
            target: PublicationTarget::CurrentPackage,
        }) {
            Ok(handle) => {
                self.state
                    .lock()
                    .expect("cache adapter mutex poisoned")
                    .successful_rehydrations += 1;
                CacheRehydrateOutcome::Rehydrated(Box::new(handle))
            }
            Err(error) => CacheRehydrateOutcome::Miss(CacheAdapterMiss::Publisher {
                error: Box::new(error),
            }),
        }
    }

    fn payload_from_handle<T>(
        &self,
        handle: &PhaseOutputRef<T>,
        canonical_payload: Vec<u8>,
        side_tables: IrSideTables,
    ) -> Result<CachedIrRecordPayload, CacheAdapterMiss> {
        let mut parent_summaries = Vec::new();
        for parent in &handle.lineage().parents {
            let Some(lineage) = self.publisher.registry().output_lineage(*parent) else {
                return Err(CacheAdapterMiss::MissingParentLineage { parent: *parent });
            };
            parent_summaries.push(ParentHashSummary {
                content_hash: lineage.content_hash,
                side_table_hash: lineage.side_table_hash,
            });
        }
        sort_parent_summaries_by_hash(&mut parent_summaries);

        if !content_hash_matches(
            &canonical_payload,
            &parent_summaries,
            handle.lineage().named_input_hashes.clone(),
            handle.content_hash(),
        ) {
            return Err(CacheAdapterMiss::PayloadHashMismatch);
        }

        let actual_side_table_hash =
            side_table_hash(&side_tables).map_err(|_| CacheAdapterMiss::SideTableHashMismatch)?;
        if actual_side_table_hash != handle.side_table_hash() {
            return Err(CacheAdapterMiss::SideTableHashMismatch);
        }

        Ok(CachedIrRecordPayload {
            output_kind: handle.output_kind().clone(),
            schema_version: handle.schema_version(),
            content_hash: handle.content_hash(),
            side_table_hash: handle.side_table_hash(),
            parent_summaries,
            named_input_hashes: handle.lineage().named_input_hashes.clone(),
            side_tables,
            canonical_payload,
        })
    }

    fn validate_rehydration_payload(
        &self,
        snapshot: BuildSnapshotId,
        cached: &CachedIrRecordPayload,
        parents: &[AnyPhaseOutputRef],
    ) -> Result<(), CacheAdapterMiss> {
        let actual_side_table_hash = side_table_hash(&cached.side_tables)
            .map_err(|_| CacheAdapterMiss::SideTableHashMismatch)?;
        if actual_side_table_hash != cached.side_table_hash {
            return Err(CacheAdapterMiss::SideTableHashMismatch);
        }
        for parent in parents {
            if parent.snapshot() != snapshot {
                return Err(CacheAdapterMiss::ParentMismatch);
            }
            self.publisher
                .storage()
                .validate_handle(parent)
                .map_err(|error| CacheAdapterMiss::Storage {
                    error: Box::new(error),
                })?;
        }
        Ok(())
    }
}

impl fmt::Display for CacheAdapterMiss {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cache(miss) => write!(formatter, "cache miss: {miss:?}"),
            Self::Skipped => formatter.write_str("cache adapter skipped encoding"),
            Self::Incompatible => formatter.write_str("cache adapter output is incompatible"),
            Self::CorruptRecord => formatter.write_str("cache adapter record is corrupt"),
            Self::PayloadHashMismatch => formatter.write_str("cache adapter payload hash mismatch"),
            Self::SideTableHashMismatch => {
                formatter.write_str("cache adapter side-table hash mismatch")
            }
            Self::ParentMismatch => formatter.write_str("cache adapter parent mismatch"),
            Self::MissingParentLineage { parent } => {
                write!(
                    formatter,
                    "cache adapter missing parent lineage `{parent:?}`"
                )
            }
            Self::Storage { error } => write!(formatter, "cache adapter storage miss: {error}"),
            Self::Publisher { error } => {
                write!(formatter, "cache adapter publisher miss: {error}")
            }
            Self::Decode { message } => {
                write!(formatter, "cache adapter payload decode failed: {message}")
            }
        }
    }
}

impl Error for CacheAdapterMiss {}

fn content_hash_matches(
    canonical_payload: &[u8],
    parent_summaries: &[ParentHashSummary],
    named_input_hashes: Vec<NamedInputHash>,
    expected: Hash,
) -> bool {
    content_hash_from_parent_summaries(canonical_payload, parent_summaries, named_input_hashes)
        .is_ok_and(|actual| actual == expected)
}

fn decode_payload<T>(decode: &BlobDecoder<T>, bytes: &[u8]) -> Result<T, BlobDecodeError> {
    decode.decode(bytes)
}

fn sort_parent_summaries_by_hash(values: &mut [ParentHashSummary]) {
    values.sort_by(|left, right| {
        left.content_hash
            .as_bytes()
            .cmp(right.content_hash.as_bytes())
            .then_with(|| {
                left.side_table_hash
                    .as_bytes()
                    .cmp(right.side_table_hash.as_bytes())
            })
    });
}

fn encode_cached_payload(payload: &CachedIrRecordPayload) -> Vec<u8> {
    let mut writer = PayloadWriter::default();
    writer.bytes(PAYLOAD_MAGIC);
    writer.u32(PAYLOAD_FORMAT_VERSION);
    writer.string(payload.output_kind.as_str());
    writer.u32(payload.schema_version.get());
    writer.hash(payload.content_hash);
    writer.hash(payload.side_table_hash);
    writer.parent_summaries(&payload.parent_summaries);
    writer.named_inputs(&payload.named_input_hashes);
    writer.side_tables(&payload.side_tables);
    writer.byte_vec(&payload.canonical_payload);
    writer.finish()
}

fn decode_cached_payload(bytes: &[u8]) -> Result<CachedIrRecordPayload, ()> {
    let mut reader = PayloadReader::new(bytes);
    reader.expect_bytes(PAYLOAD_MAGIC)?;
    if reader.u32()? != PAYLOAD_FORMAT_VERSION {
        return Err(());
    }
    let output_kind = OutputKind::new(reader.string()?);
    let schema_version = SchemaVersion::new(reader.u32()?);
    let content_hash = reader.hash()?;
    let side_table_hash = reader.hash()?;
    let parent_summaries = reader.parent_summaries()?;
    let named_input_hashes = reader.named_inputs()?;
    let side_tables = reader.side_tables()?;
    let canonical_payload = reader.byte_vec()?;
    if !reader.is_empty() {
        return Err(());
    }
    Ok(CachedIrRecordPayload {
        output_kind,
        schema_version,
        content_hash,
        side_table_hash,
        parent_summaries,
        named_input_hashes,
        side_tables,
        canonical_payload,
    })
}

#[derive(Default)]
struct PayloadWriter {
    bytes: Vec<u8>,
}

impl PayloadWriter {
    fn finish(self) -> Vec<u8> {
        self.bytes
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn len(&mut self, value: usize) {
        self.u64(value as u64);
    }

    fn string(&mut self, value: &str) {
        self.byte_vec(value.as_bytes());
    }

    fn hash(&mut self, value: Hash) {
        self.bytes(value.as_bytes());
    }

    fn byte_vec(&mut self, value: &[u8]) {
        self.len(value.len());
        self.bytes(value);
    }

    fn parent_summaries(&mut self, values: &[ParentHashSummary]) {
        self.len(values.len());
        for value in values {
            self.hash(value.content_hash);
            self.hash(value.side_table_hash);
        }
    }

    fn named_inputs(&mut self, values: &[NamedInputHash]) {
        self.len(values.len());
        for value in values {
            self.string(&value.name);
            self.string(&value.domain);
            self.hash(value.digest);
        }
    }

    fn side_tables(&mut self, value: &IrSideTables) {
        self.side_table_records(&value.source_maps);
        self.side_table_records(&value.diagnostics);
        self.side_table_records(&value.explanation_refs);
        self.side_table_records(&value.documentation_attachments);
    }

    fn side_table_records(&mut self, values: &[SideTableRecord]) {
        self.len(values.len());
        for value in values {
            self.string(&value.domain);
            self.string(&value.key);
            self.hash(value.digest);
        }
    }
}

struct PayloadReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> PayloadReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), ()> {
        let actual = self.take(expected.len())?;
        if actual == expected { Ok(()) } else { Err(()) }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], ()> {
        let end = self.offset.checked_add(len).ok_or(())?;
        if end > self.bytes.len() {
            return Err(());
        }
        let result = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(result)
    }

    fn u32(&mut self) -> Result<u32, ()> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().map_err(|_| ())?))
    }

    fn u64(&mut self) -> Result<u64, ()> {
        let bytes = self.take(8)?;
        Ok(u64::from_le_bytes(bytes.try_into().map_err(|_| ())?))
    }

    fn len(&mut self) -> Result<usize, ()> {
        self.u64()?.try_into().map_err(|_| ())
    }

    fn bounded_count(&mut self, min_bytes_per_entry: usize) -> Result<usize, ()> {
        let count = self.len()?;
        if min_bytes_per_entry == 0 || count > self.remaining() / min_bytes_per_entry {
            return Err(());
        }
        Ok(count)
    }

    fn string(&mut self) -> Result<String, ()> {
        String::from_utf8(self.byte_vec()?).map_err(|_| ())
    }

    fn hash(&mut self) -> Result<Hash, ()> {
        let bytes = self.take(Hash::BYTE_LEN)?;
        Ok(Hash::from_bytes(bytes.try_into().map_err(|_| ())?))
    }

    fn byte_vec(&mut self) -> Result<Vec<u8>, ()> {
        let len = self.len()?;
        Ok(self.take(len)?.to_vec())
    }

    fn parent_summaries(&mut self) -> Result<Vec<ParentHashSummary>, ()> {
        let count = self.bounded_count(Hash::BYTE_LEN * 2)?;
        let mut result = Vec::new();
        result.try_reserve(count).map_err(|_| ())?;
        for _ in 0..count {
            result.push(ParentHashSummary {
                content_hash: self.hash()?,
                side_table_hash: self.hash()?,
            });
        }
        Ok(result)
    }

    fn named_inputs(&mut self) -> Result<Vec<NamedInputHash>, ()> {
        let count = self.bounded_count(8 + 8 + Hash::BYTE_LEN)?;
        let mut result = Vec::new();
        result.try_reserve(count).map_err(|_| ())?;
        for _ in 0..count {
            result.push(NamedInputHash {
                name: self.string()?,
                domain: self.string()?,
                digest: self.hash()?,
            });
        }
        Ok(result)
    }

    fn side_tables(&mut self) -> Result<IrSideTables, ()> {
        Ok(IrSideTables {
            source_maps: self.side_table_records()?,
            diagnostics: self.side_table_records()?,
            explanation_refs: self.side_table_records()?,
            documentation_attachments: self.side_table_records()?,
        })
    }

    fn side_table_records(&mut self) -> Result<Vec<SideTableRecord>, ()> {
        let count = self.bounded_count(8 + 8 + Hash::BYTE_LEN)?;
        let mut result = Vec::new();
        result.try_reserve(count).map_err(|_| ())?;
        for _ in 0..count {
            result.push(SideTableRecord {
                domain: self.string()?,
                key: self.string()?,
                digest: self.hash()?,
            });
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_cache::cache_key::{
        CACHE_KEY_SCHEMA_VERSION, CacheKeyBuildOutcome, CacheKeyBuilder, CacheKeyRequest,
        CacheValidationInputs, FootprintCompleteness, NamedSchemaVersion,
        PipelinePhase as CachePipelinePhase, PolicyFingerprint,
        SchemaVersion as CacheSchemaVersion, WorkUnit as CacheWorkUnit,
    };

    use crate::{
        identity::{OutputIdentityInput, PhaseOutputLineage, SnapshotHandleRegistry},
        publisher::AllowedWorkUnit,
        storage::{BlobDecodeError, CollectInput, IrStorageService, StorageError},
    };

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }

    fn snapshot(seed: u8) -> BuildSnapshotId {
        let hex = [seed; Hash::BYTE_LEN]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .expect("test snapshot id is valid")
    }

    fn phase() -> PipelinePhase {
        PipelinePhase::new("resolve")
    }

    fn work_unit() -> WorkUnit {
        WorkUnit::new("unit")
    }

    fn parent_work_unit() -> WorkUnit {
        WorkUnit::new("parent")
    }

    fn output_kind() -> OutputKind {
        OutputKind::new("ResolvedAst")
    }

    fn schema() -> SchemaVersion {
        SchemaVersion::new(1)
    }

    fn named(seed: u8) -> NamedInputHash {
        NamedInputHash {
            name: "source".to_owned(),
            domain: "test".to_owned(),
            digest: hash(seed),
        }
    }

    fn side_tables(seed: u8) -> IrSideTables {
        IrSideTables {
            source_maps: vec![SideTableRecord::new("source-map", "Main", hash(seed))],
            diagnostics: vec![SideTableRecord::new("diagnostic", "D001", hash(seed + 1))],
            explanation_refs: Vec::new(),
            documentation_attachments: Vec::new(),
        }
    }

    fn string_decoder() -> BlobDecoder<String> {
        BlobDecoder::new(|bytes| {
            String::from_utf8(bytes.to_vec())
                .map_err(|error| BlobDecodeError::new(error.to_string()))
        })
    }

    fn publisher(snapshot: BuildSnapshotId) -> Arc<PhaseOutputPublisher> {
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        publisher.register_current_snapshot(snapshot);
        publisher.allow_work_unit(AllowedWorkUnit::new(phase(), output_kind(), work_unit()));
        publisher.allow_work_unit(AllowedWorkUnit::new(
            phase(),
            output_kind(),
            parent_work_unit(),
        ));
        publisher
    }

    fn publish_text(
        publisher: &PhaseOutputPublisher,
        snapshot: BuildSnapshotId,
        payload: &str,
        side_tables: IrSideTables,
    ) -> PhaseOutputRef<String> {
        publish_text_with(
            publisher,
            snapshot,
            work_unit(),
            payload,
            Vec::new(),
            side_tables,
        )
    }

    fn publish_text_with(
        publisher: &PhaseOutputPublisher,
        snapshot: BuildSnapshotId,
        work_unit: WorkUnit,
        payload: &str,
        parents: Vec<AnyPhaseOutputRef>,
        side_tables: IrSideTables,
    ) -> PhaseOutputRef<String> {
        publisher
            .publish(PublishOutputInput {
                slot: publisher.allocate(
                    snapshot,
                    phase(),
                    work_unit.clone(),
                    output_kind(),
                    schema(),
                ),
                snapshot,
                phase: phase(),
                work_unit,
                output_kind: output_kind(),
                schema_version: schema(),
                payload: payload.to_owned(),
                canonical_payload: Some(payload.as_bytes().to_vec()),
                decode: string_decoder(),
                parents,
                named_input_hashes: vec![named(1)],
                side_tables,
                origin: OutputOrigin::PackageSource,
                target: PublicationTarget::CurrentPackage,
            })
            .expect("publish succeeds")
    }

    fn cache_key(seed: u8) -> CacheKey {
        let version = CacheSchemaVersion::new(CACHE_KEY_SCHEMA_VERSION);
        let request = CacheKeyRequest {
            cache_schema_version: version.clone(),
            phase: CachePipelinePhase::new("resolve"),
            work_unit: CacheWorkUnit::new(format!("unit-{seed}")),
            source_identity: None,
            input_hashes: Vec::new(),
            dependency_hashes: Vec::new(),
            dependency_slices: Vec::new(),
            config_hash: hash(seed),
            schema_versions: vec![NamedSchemaVersion {
                schema_family: "mizar-ir".to_owned(),
                name: "cache-adapter-payload".to_owned(),
                version: CacheSchemaVersion::new("mizar-ir/cache-adapter-payload/v1"),
            }],
            policy_fingerprint: PolicyFingerprint::new(hash(seed + 1)),
            validation_inputs: CacheValidationInputs {
                cache_schema_compatibility: version,
                toolchain_compatibility: vec![compat("toolchain", "rust", "test")],
                dependency_artifacts: Vec::new(),
                footprint_completeness: FootprintCompleteness::Complete,
                uncacheable: false,
                policy_compatibility: vec![compat("policy", "verifier", "test")],
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
            },
        };
        match CacheKeyBuilder::new(request).build() {
            CacheKeyBuildOutcome::Cacheable(key) => key,
            other => panic!("expected cacheable key, got {other:?}"),
        }
    }

    fn compat(family: &str, field_name: &str, value: &str) -> CompatibilityField {
        CompatibilityField {
            family: family.to_owned(),
            field_name: field_name.to_owned(),
            value: value.to_owned(),
        }
    }

    fn produced_by() -> Vec<CompatibilityField> {
        vec![compat("toolchain", "rust", "test")]
    }

    fn encode_record(
        adapter: &IrCacheAdapter,
        handle: &PhaseOutputRef<String>,
        payload: &str,
    ) -> CacheRecord {
        match adapter.encode(EncodeCacheRecordInput {
            handle: handle.clone(),
            cache_key: cache_key(10),
            produced_by: produced_by(),
            canonical_payload: payload.as_bytes().to_vec(),
            cacheability: CacheAdapterCacheability::Cacheable,
        }) {
            EncodeCacheRecordOutcome::Encoded(record) => *record,
            other => panic!("expected encoded record, got {other:?}"),
        }
    }

    fn rehydrate(
        adapter: &IrCacheAdapter,
        record: CacheRecord,
        snapshot: BuildSnapshotId,
    ) -> CacheRehydrateOutcome<String> {
        rehydrate_with_parents(adapter, record, snapshot, Vec::new())
    }

    fn rehydrate_with_parents(
        adapter: &IrCacheAdapter,
        record: CacheRecord,
        snapshot: BuildSnapshotId,
        parents: Vec<AnyPhaseOutputRef>,
    ) -> CacheRehydrateOutcome<String> {
        adapter.rehydrate(RehydrateCacheHitInput {
            lookup: CacheLookupOutcome::Hit(Box::new(record)),
            snapshot,
            phase: phase(),
            work_unit: work_unit(),
            output_kind: output_kind(),
            schema_version: schema(),
            parents,
            named_input_hashes: vec![named(1)],
            decode: string_decoder(),
        })
    }

    fn expected_lineage(
        snapshot: BuildSnapshotId,
        content_hash: Hash,
        side_table_hash: Hash,
    ) -> PhaseOutputLineage {
        expected_lineage_with_parents(snapshot, content_hash, side_table_hash, Vec::new())
    }

    fn expected_lineage_with_parents(
        snapshot: BuildSnapshotId,
        content_hash: Hash,
        side_table_hash: Hash,
        parents: Vec<PhaseOutputId>,
    ) -> PhaseOutputLineage {
        PhaseOutputLineage::from_input(OutputIdentityInput {
            snapshot,
            phase: phase(),
            work_unit: work_unit(),
            output_kind: output_kind(),
            content_hash,
            side_table_hash,
            parents,
            named_input_hashes: vec![named(1)],
        })
        .expect("expected lineage derives")
    }

    #[test]
    fn round_trip_rehydrates_validated_hit_into_current_snapshot() {
        let old_snapshot = snapshot(1);
        let current_snapshot = snapshot(2);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher.clone());
        let original = publish_text(&source_publisher, old_snapshot, "payload", side_tables(20));
        let record = encode_record(&source_adapter, &original, "payload");

        let CacheRehydrateOutcome::Rehydrated(rehydrated) =
            rehydrate(&target_adapter, record, current_snapshot)
        else {
            panic!("validated hit should rehydrate");
        };

        assert_eq!(rehydrated.snapshot(), current_snapshot);
        assert_ne!(rehydrated.output(), original.output());
        assert_eq!(rehydrated.content_hash(), original.content_hash());
        assert_eq!(rehydrated.side_table_hash(), original.side_table_hash());
        assert_eq!(
            &*target_publisher
                .storage()
                .get(&rehydrated)
                .expect("payload is readable"),
            "payload"
        );
        assert_eq!(
            &*target_publisher
                .storage()
                .side_tables(&rehydrated)
                .expect("side tables are readable"),
            &side_tables(20)
        );
        assert_eq!(target_adapter.successful_rehydrations(), 1);
    }

    #[test]
    fn parented_rehydration_uses_current_snapshot_parent_handles() {
        let old_snapshot = snapshot(13);
        let current_snapshot = snapshot(14);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher.clone());
        let old_parent = publish_text_with(
            &source_publisher,
            old_snapshot,
            parent_work_unit(),
            "parent",
            Vec::new(),
            side_tables(80),
        );
        let old_child = publish_text_with(
            &source_publisher,
            old_snapshot,
            work_unit(),
            "child",
            vec![old_parent.erase()],
            side_tables(81),
        );
        let current_parent = publish_text_with(
            &target_publisher,
            current_snapshot,
            parent_work_unit(),
            "parent",
            Vec::new(),
            side_tables(80),
        );
        let record = encode_record(&source_adapter, &old_child, "child");

        let CacheRehydrateOutcome::Rehydrated(rehydrated) = rehydrate_with_parents(
            &target_adapter,
            record,
            current_snapshot,
            vec![current_parent.erase()],
        ) else {
            panic!("validated parented hit should rehydrate");
        };

        assert_eq!(rehydrated.snapshot(), current_snapshot);
        assert_ne!(rehydrated.output(), old_child.output());
        assert_eq!(rehydrated.content_hash(), old_child.content_hash());
        assert_eq!(rehydrated.lineage().parents, vec![current_parent.output()]);
    }

    #[test]
    fn foreign_parent_handle_misses_before_current_rehydration() {
        let old_snapshot = snapshot(15);
        let current_snapshot = snapshot(16);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher.clone());
        let old_parent = publish_text_with(
            &source_publisher,
            old_snapshot,
            parent_work_unit(),
            "parent",
            Vec::new(),
            side_tables(82),
        );
        let old_child = publish_text_with(
            &source_publisher,
            old_snapshot,
            work_unit(),
            "child",
            vec![old_parent.erase()],
            side_tables(83),
        );
        let record = encode_record(&source_adapter, &old_child, "child");
        let cached = decode_cached_payload(&record.output).expect("payload decodes");
        let expected = expected_lineage(
            current_snapshot,
            cached.content_hash,
            cached.side_table_hash,
        );

        let outcome = rehydrate_with_parents(
            &target_adapter,
            record,
            current_snapshot,
            vec![old_parent.erase()],
        );

        assert!(matches!(
            outcome,
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::ParentMismatch)
        ));
        assert!(
            target_publisher
                .registry()
                .output_lineage(expected.output)
                .is_none()
        );
    }

    #[test]
    fn collected_parent_handle_misses_before_current_rehydration() {
        let old_snapshot = snapshot(25);
        let current_snapshot = snapshot(26);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher.clone());
        let old_parent = publish_text_with(
            &source_publisher,
            old_snapshot,
            parent_work_unit(),
            "parent",
            Vec::new(),
            side_tables(84),
        );
        let old_child = publish_text_with(
            &source_publisher,
            old_snapshot,
            work_unit(),
            "child",
            vec![old_parent.erase()],
            side_tables(85),
        );
        let current_parent = publish_text_with(
            &target_publisher,
            current_snapshot,
            parent_work_unit(),
            "parent",
            Vec::new(),
            side_tables(84),
        );
        let parent_ref = current_parent.erase();
        let record = encode_record(&source_adapter, &old_child, "child");
        let cached = decode_cached_payload(&record.output).expect("payload decodes");
        let expected = expected_lineage_with_parents(
            current_snapshot,
            cached.content_hash,
            cached.side_table_hash,
            vec![current_parent.output()],
        );
        target_publisher.storage().collect(CollectInput {
            snapshot: current_snapshot,
            protected_outputs: Vec::new(),
        });

        let outcome =
            rehydrate_with_parents(&target_adapter, record, current_snapshot, vec![parent_ref]);

        assert!(matches!(
            outcome,
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::Storage { error })
                if matches!(*error, StorageError::CollectedOutput { output } if output == current_parent.output())
        ));
        assert_eq!(target_adapter.successful_rehydrations(), 0);
        assert!(
            target_publisher
                .registry()
                .output_lineage(expected.output)
                .is_none()
        );
    }

    #[test]
    fn stale_generation_parent_handle_after_collection_misses_before_current_rehydration() {
        let old_snapshot = snapshot(27);
        let current_snapshot = snapshot(28);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher.clone());
        let old_parent = publish_text_with(
            &source_publisher,
            old_snapshot,
            parent_work_unit(),
            "parent",
            Vec::new(),
            side_tables(86),
        );
        let old_child = publish_text_with(
            &source_publisher,
            old_snapshot,
            work_unit(),
            "child",
            vec![old_parent.erase()],
            side_tables(87),
        );
        let current_parent = publish_text_with(
            &target_publisher,
            current_snapshot,
            parent_work_unit(),
            "parent",
            Vec::new(),
            side_tables(86),
        );
        let stale_parent_ref = current_parent.erase();
        target_publisher.storage().collect(CollectInput {
            snapshot: current_snapshot,
            protected_outputs: Vec::new(),
        });
        let resealed_parent = publish_text_with(
            &target_publisher,
            current_snapshot,
            parent_work_unit(),
            "parent",
            Vec::new(),
            side_tables(86),
        );
        assert_eq!(current_parent.output(), resealed_parent.output());
        assert_ne!(
            current_parent.any().generation(),
            resealed_parent.any().generation()
        );
        let record = encode_record(&source_adapter, &old_child, "child");
        let cached = decode_cached_payload(&record.output).expect("payload decodes");
        let expected = expected_lineage_with_parents(
            current_snapshot,
            cached.content_hash,
            cached.side_table_hash,
            vec![current_parent.output()],
        );

        let outcome = rehydrate_with_parents(
            &target_adapter,
            record,
            current_snapshot,
            vec![stale_parent_ref],
        );

        assert!(matches!(
            outcome,
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::Storage { error })
                if matches!(*error, StorageError::CollectedOutput { output } if output == current_parent.output())
        ));
        assert_eq!(target_adapter.successful_rehydrations(), 0);
        assert!(
            target_publisher
                .registry()
                .output_lineage(expected.output)
                .is_none()
        );
    }

    #[test]
    fn miss_states_do_not_create_handles_before_validation() {
        let snapshot = snapshot(3);
        let publisher = publisher(snapshot);
        let adapter = IrCacheAdapter::new(publisher);
        for miss in [
            CacheMiss::NotFound,
            CacheMiss::UnknownSchema,
            CacheMiss::UnknownToolchain,
            CacheMiss::IncompleteFootprint,
            CacheMiss::UnsupportedFootprint,
            CacheMiss::Uncacheable,
            CacheMiss::DependencyUnavailable,
            CacheMiss::PolicyIncompatible,
            CacheMiss::ProofReuseInvalid,
            CacheMiss::CorruptRecord,
        ] {
            let outcome = adapter.rehydrate::<String>(RehydrateCacheHitInput {
                lookup: CacheLookupOutcome::Miss(miss),
                snapshot,
                phase: phase(),
                work_unit: work_unit(),
                output_kind: output_kind(),
                schema_version: schema(),
                parents: Vec::new(),
                named_input_hashes: vec![named(1)],
                decode: string_decoder(),
            });
            assert!(matches!(
                outcome,
                CacheRehydrateOutcome::Miss(CacheAdapterMiss::Cache(_))
            ));
        }
        assert_eq!(adapter.successful_rehydrations(), 0);
    }

    #[test]
    fn malformed_and_incompatible_validated_hits_miss_before_sealing() {
        let old_snapshot = snapshot(17);
        let current_snapshot = snapshot(18);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher.clone());
        let original = publish_text(&source_publisher, old_snapshot, "payload", side_tables(24));
        let record = encode_record(&source_adapter, &original, "payload");
        let cached = decode_cached_payload(&record.output).expect("payload decodes");
        let expected = expected_lineage(
            current_snapshot,
            cached.content_hash,
            cached.side_table_hash,
        );

        let mut corrupt = record.clone();
        corrupt.output = b"not-a-mizar-ir-cache-payload".to_vec();
        assert!(matches!(
            rehydrate(&target_adapter, corrupt, current_snapshot),
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::CorruptRecord)
        ));

        let mut wrong_kind = record.clone();
        let mut payload = decode_cached_payload(&wrong_kind.output).expect("payload decodes");
        payload.output_kind = OutputKind::new("OtherKind");
        wrong_kind.output = encode_cached_payload(&payload);
        assert!(matches!(
            rehydrate(&target_adapter, wrong_kind, current_snapshot),
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::Incompatible)
        ));

        let mut wrong_schema = record;
        let mut payload = decode_cached_payload(&wrong_schema.output).expect("payload decodes");
        payload.schema_version = SchemaVersion::new(99);
        wrong_schema.output = encode_cached_payload(&payload);
        assert!(matches!(
            rehydrate(&target_adapter, wrong_schema, current_snapshot),
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::Incompatible)
        ));

        assert!(
            target_publisher
                .registry()
                .output_lineage(expected.output)
                .is_none()
        );
    }

    #[test]
    fn malformed_payload_lengths_fail_closed_without_large_allocation() {
        let old_snapshot = snapshot(21);
        let current_snapshot = snapshot(22);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher.clone());
        let original = publish_text(&source_publisher, old_snapshot, "payload", side_tables(26));
        let mut record = encode_record(&source_adapter, &original, "payload");
        let cached = decode_cached_payload(&record.output).expect("payload decodes");
        let expected = expected_lineage(
            current_snapshot,
            cached.content_hash,
            cached.side_table_hash,
        );
        let mut bytes = Vec::new();
        bytes.extend_from_slice(PAYLOAD_MAGIC);
        bytes.extend_from_slice(&PAYLOAD_FORMAT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&(u64::MAX).to_le_bytes());
        record.output = bytes;

        let outcome = rehydrate(&target_adapter, record, current_snapshot);

        assert!(matches!(
            outcome,
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::CorruptRecord)
        ));
        assert!(
            target_publisher
                .registry()
                .output_lineage(expected.output)
                .is_none()
        );
    }

    #[test]
    fn tampered_payload_misses_without_sealing_or_lineage() {
        let old_snapshot = snapshot(4);
        let current_snapshot = snapshot(5);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher.clone());
        let original = publish_text(&source_publisher, old_snapshot, "payload", side_tables(30));
        let mut record = encode_record(&source_adapter, &original, "payload");
        let mut cached = decode_cached_payload(&record.output).expect("payload decodes");
        cached.canonical_payload = b"tampered".to_vec();
        record.output = encode_cached_payload(&cached);
        let expected = expected_lineage(
            current_snapshot,
            cached.content_hash,
            cached.side_table_hash,
        );

        let outcome = rehydrate(&target_adapter, record, current_snapshot);

        assert!(matches!(
            outcome,
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::PayloadHashMismatch)
        ));
        assert!(
            target_publisher
                .registry()
                .output_lineage(expected.output)
                .is_none()
        );
    }

    #[test]
    fn tampered_hash_fields_miss_without_sealing_or_lineage() {
        let old_snapshot = snapshot(19);
        let current_snapshot = snapshot(20);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher.clone());
        let original = publish_text(&source_publisher, old_snapshot, "payload", side_tables(42));

        let mut content_record = encode_record(&source_adapter, &original, "payload");
        let mut cached = decode_cached_payload(&content_record.output).expect("payload decodes");
        cached.content_hash = hash(222);
        content_record.output = encode_cached_payload(&cached);
        let expected_content = expected_lineage(
            current_snapshot,
            cached.content_hash,
            cached.side_table_hash,
        );
        assert!(matches!(
            rehydrate(&target_adapter, content_record, current_snapshot),
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::PayloadHashMismatch)
        ));
        assert!(
            target_publisher
                .registry()
                .output_lineage(expected_content.output)
                .is_none()
        );

        let mut side_record = encode_record(&source_adapter, &original, "payload");
        let mut cached = decode_cached_payload(&side_record.output).expect("payload decodes");
        cached.side_table_hash = hash(223);
        side_record.output = encode_cached_payload(&cached);
        let expected_side = expected_lineage(
            current_snapshot,
            cached.content_hash,
            cached.side_table_hash,
        );
        assert!(matches!(
            rehydrate(&target_adapter, side_record, current_snapshot),
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::SideTableHashMismatch)
        ));
        assert!(
            target_publisher
                .registry()
                .output_lineage(expected_side.output)
                .is_none()
        );
    }

    #[test]
    fn tampered_side_tables_miss_without_sealing_or_lineage() {
        let old_snapshot = snapshot(6);
        let current_snapshot = snapshot(7);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher.clone());
        let original = publish_text(&source_publisher, old_snapshot, "payload", side_tables(40));
        let mut record = encode_record(&source_adapter, &original, "payload");
        let mut cached = decode_cached_payload(&record.output).expect("payload decodes");
        cached
            .side_tables
            .diagnostics
            .push(SideTableRecord::new("diagnostic", "D999", hash(99)));
        record.output = encode_cached_payload(&cached);
        let expected = expected_lineage(
            current_snapshot,
            cached.content_hash,
            cached.side_table_hash,
        );

        let outcome = rehydrate(&target_adapter, record, current_snapshot);

        assert!(matches!(
            outcome,
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::SideTableHashMismatch)
        ));
        assert!(
            target_publisher
                .registry()
                .output_lineage(expected.output)
                .is_none()
        );
    }

    #[test]
    fn encode_skips_uncacheable_and_incompatible_records() {
        let snapshot = snapshot(8);
        let publisher = publisher(snapshot);
        let adapter = IrCacheAdapter::new(publisher.clone());
        let handle = publish_text(&publisher, snapshot, "payload", side_tables(50));

        for cacheability in [
            CacheAdapterCacheability::Skip,
            CacheAdapterCacheability::Incompatible,
        ] {
            let outcome = adapter.encode(EncodeCacheRecordInput {
                handle: handle.clone(),
                cache_key: cache_key(20),
                produced_by: produced_by(),
                canonical_payload: b"payload".to_vec(),
                cacheability,
            });
            assert!(matches!(outcome, EncodeCacheRecordOutcome::Skipped(_)));
        }
    }

    #[test]
    fn superseded_snapshot_output_can_still_encode_as_cache_input() {
        let old_snapshot = snapshot(24);
        let new_snapshot = snapshot(25);
        let publisher = publisher(old_snapshot);
        let adapter = IrCacheAdapter::new(publisher.clone());
        let handle = publish_text(&publisher, old_snapshot, "payload", side_tables(80));

        publisher
            .replace_current_snapshot(old_snapshot, new_snapshot)
            .expect("snapshot replacement succeeds");
        assert!(matches!(
            publisher.validate_current_output(old_snapshot, handle.any()),
            Err(PublishError::ObsoleteSnapshot { snapshot }) if snapshot == old_snapshot
        ));

        let outcome = adapter.encode(EncodeCacheRecordInput {
            handle,
            cache_key: cache_key(21),
            produced_by: produced_by(),
            canonical_payload: b"payload".to_vec(),
            cacheability: CacheAdapterCacheability::Cacheable,
        });
        assert!(matches!(outcome, EncodeCacheRecordOutcome::Encoded(_)));
    }

    #[test]
    fn decode_failure_misses_without_rehydration() {
        let old_snapshot = snapshot(9);
        let current_snapshot = snapshot(10);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher);
        let original = publish_text(&source_publisher, old_snapshot, "payload", side_tables(60));
        let record = encode_record(&source_adapter, &original, "payload");

        let outcome = target_adapter.rehydrate(RehydrateCacheHitInput {
            lookup: CacheLookupOutcome::Hit(Box::new(record)),
            snapshot: current_snapshot,
            phase: phase(),
            work_unit: work_unit(),
            output_kind: output_kind(),
            schema_version: schema(),
            parents: Vec::new(),
            named_input_hashes: vec![named(1)],
            decode: BlobDecoder::<String>::new(|_| Err(BlobDecodeError::new("decode failed"))),
        });

        assert!(matches!(
            outcome,
            CacheRehydrateOutcome::Miss(CacheAdapterMiss::Decode { .. })
        ));
        assert_eq!(target_adapter.successful_rehydrations(), 0);
    }

    #[test]
    fn rehydrated_handles_do_not_carry_cache_or_proof_authority() {
        let old_snapshot = snapshot(11);
        let current_snapshot = snapshot(12);
        let source_publisher = publisher(old_snapshot);
        let target_publisher = publisher(current_snapshot);
        let source_adapter = IrCacheAdapter::new(source_publisher.clone());
        let target_adapter = IrCacheAdapter::new(target_publisher);
        let payload = "proof=true cache_key=abc dependency_fingerprint=abc trusted=true kernel_acceptance=true";
        let original = publish_text(&source_publisher, old_snapshot, payload, side_tables(70));
        let record = encode_record(&source_adapter, &original, payload);

        let CacheRehydrateOutcome::Rehydrated(rehydrated) =
            rehydrate(&target_adapter, record, current_snapshot)
        else {
            panic!("validated hit should rehydrate");
        };
        assert_eq!(rehydrated.phase(), &phase());
        assert_eq!(rehydrated.work_unit(), &work_unit());
        assert_eq!(rehydrated.output_kind(), &output_kind());
        assert_eq!(rehydrated.schema_version(), schema());
        assert_eq!(rehydrated.lineage().parents, Vec::new());
        assert_eq!(rehydrated.lineage().named_input_hashes, vec![named(1)]);

        let debug = format!("{:?} {:?}", rehydrated.erase(), rehydrated.lineage());
        for forbidden in [
            "cache_key",
            "dependency_fingerprint",
            "trusted=true",
            "proof=true",
            "kernel_acceptance",
        ] {
            assert!(
                !debug.contains(forbidden),
                "rehydrated handle metadata must not expose `{forbidden}` authority"
            );
        }
    }
}
