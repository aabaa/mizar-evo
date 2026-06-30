//! Immutable phase-output storage and typed handles.
//!
//! This module is specified in
//! [`storage.md`](../../../../doc/design/mizar-ir/en/storage.md).

use std::{
    any::{Any, TypeId, type_name},
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use mizar_session::{BuildSnapshotId, Hash};

use crate::identity::{
    IdentityError, OutputKind, PhaseOutputId, PhaseOutputLineage, PipelinePhase, WorkUnit,
};

/// Default canonical payload size above which outputs spill to blobs.
pub const DEFAULT_BLOB_SPILL_THRESHOLD: usize = 64 * 1024;

/// Storage schema version for a phase-output payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
pub struct SchemaVersion(u32);

/// Storage slot id. Slot ids are process-local and never stable identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct OutputSlotId(u64);

/// Storage generation used to reject stale handles if a future task reuses
/// storage internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct StorageGeneration(u64);

/// Content-addressed internal blob id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct ContentBlobId(Hash);

/// Storage placement policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePolicy {
    /// Canonical payload bytes above this threshold spill to blobs.
    pub blob_spill_threshold: usize,
}

/// Payload placement recorded on a sealed handle.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StoragePlacement {
    /// Payload is stored as an in-memory typed value.
    Resident,
    /// Payload canonical bytes are stored in an internal content-addressed blob.
    Blob {
        /// Blob id.
        blob: ContentBlobId,
        /// Canonical byte length.
        len: usize,
    },
}

/// Side-table record stored beside a sealed output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SideTableRecord {
    /// Side-table domain, such as `source-map` or `diagnostic`.
    pub domain: String,
    /// Producer-local stable key.
    pub key: String,
    /// Digest of the side-table payload.
    pub digest: Hash,
}

/// Side tables attached to one sealed phase output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IrSideTables {
    /// Source-map and source-range records.
    pub source_maps: Vec<SideTableRecord>,
    /// Diagnostic records or diagnostic identifiers.
    pub diagnostics: Vec<SideTableRecord>,
    /// Explanation request references.
    pub explanation_refs: Vec<SideTableRecord>,
    /// Documentation attachment identifiers.
    pub documentation_attachments: Vec<SideTableRecord>,
}

/// Input for allocating a private pending output slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocateSlotInput {
    /// Snapshot that owns the output.
    pub snapshot: BuildSnapshotId,
    /// Producing phase.
    pub phase: PipelinePhase,
    /// Producing work unit.
    pub work_unit: WorkUnit,
    /// Runtime output kind tag.
    pub output_kind: OutputKind,
    /// Payload schema version.
    pub schema_version: SchemaVersion,
}

/// A producer-private pending output slot.
///
/// Cloning this value does not clone storage. It only copies the capability to
/// attempt sealing the same pending slot; storage still accepts exactly one
/// seal.
#[derive(Debug)]
pub struct PendingOutputSlot<T> {
    id: OutputSlotId,
    snapshot: BuildSnapshotId,
    phase: PipelinePhase,
    work_unit: WorkUnit,
    output_kind: OutputKind,
    schema_version: SchemaVersion,
    generation: StorageGeneration,
    marker: PhantomData<fn() -> T>,
}

/// Input for sealing a pending output slot.
#[derive(Debug)]
pub struct SealOutputInput<T> {
    /// Pending slot allocated for this output.
    pub slot: PendingOutputSlot<T>,
    /// Registered output lineage.
    pub lineage: PhaseOutputLineage,
    /// Side tables to store beside the payload.
    pub side_tables: IrSideTables,
    /// Complete immutable payload.
    pub payload: T,
}

/// Input for sealing a pending output from canonical bytes.
pub struct SealBlobOutputInput<T> {
    /// Pending slot allocated for this output.
    pub slot: PendingOutputSlot<T>,
    /// Registered output lineage.
    pub lineage: PhaseOutputLineage,
    /// Side tables to store beside the payload.
    pub side_tables: IrSideTables,
    /// Canonical payload bytes used for blob placement and content addressing.
    pub canonical_bytes: Vec<u8>,
    /// Decoder for reconstructing the typed payload from canonical bytes.
    pub decode: BlobDecoder<T>,
}

/// Input for sealing a complete payload with canonical bytes.
pub struct SealCanonicalOutputInput<T> {
    /// Pending slot allocated for this output.
    pub slot: PendingOutputSlot<T>,
    /// Registered output lineage.
    pub lineage: PhaseOutputLineage,
    /// Side tables to store beside the payload.
    pub side_tables: IrSideTables,
    /// Complete immutable payload.
    pub payload: T,
    /// Canonical payload bytes used for blob placement and content addressing.
    pub canonical_bytes: Vec<u8>,
    /// Decoder for reconstructing the typed payload from blob-backed bytes.
    pub decode: BlobDecoder<T>,
}

/// Typed decoder for blob-backed payloads.
pub struct BlobDecoder<T> {
    decode: TypedBlobDecoder<T>,
}

/// Error returned by a producer-supplied blob decoder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobDecodeError {
    message: String,
}

/// Explicit owner retaining a sealed output across collection.
#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
pub struct RetainOwner(String);

/// Input for a collection pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectInput {
    /// Snapshot to collect.
    pub snapshot: BuildSnapshotId,
    /// Outputs protected by caller-owned current roots or active readers.
    pub protected_outputs: Vec<PhaseOutputId>,
}

/// Summary returned by collection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollectionSummary {
    /// Sealed outputs dropped.
    pub outputs_dropped: usize,
    /// Abandoned pending slots dropped.
    pub abandoned_slots_dropped: usize,
    /// Internal blobs dropped after their last slot disappeared.
    pub blobs_dropped: usize,
    /// Outputs skipped because they still have retain owners.
    pub retained_outputs: usize,
    /// Outputs skipped because the caller protected them.
    pub protected_outputs: usize,
}

/// Erased immutable phase-output handle.
///
/// This handle carries storage and lineage metadata but no typed payload
/// capability. Use [`IrStorageService::typed_handle`] to recover a typed handle
/// after runtime kind and Rust type validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnyPhaseOutputRef {
    lineage: PhaseOutputLineage,
    schema_version: SchemaVersion,
    generation: StorageGeneration,
    placement: StoragePlacement,
}

/// Typed immutable phase-output handle.
///
/// The type parameter is a compile-time expectation. Storage validates the
/// runtime kind tag and stored Rust type before returning a payload.
#[derive(Debug)]
pub struct PhaseOutputRef<T> {
    inner: AnyPhaseOutputRef,
    marker: PhantomData<fn() -> T>,
}

/// In-memory storage service for sealed `mizar-ir` phase outputs.
#[derive(Debug, Default)]
pub struct IrStorageService {
    policy: StoragePolicy,
    state: Mutex<StorageState>,
}

#[derive(Debug, Default)]
struct StorageState {
    next_slot_id: u64,
    slots: HashMap<OutputSlotId, SlotRecord>,
    outputs: HashMap<PhaseOutputId, OutputSlotId>,
    output_generations: HashMap<PhaseOutputId, u64>,
    blobs: HashMap<ContentBlobId, Arc<Vec<u8>>>,
    collected_outputs: HashSet<PhaseOutputId>,
}

#[derive(Debug)]
struct SlotRecord {
    snapshot: BuildSnapshotId,
    phase: PipelinePhase,
    work_unit: WorkUnit,
    output_kind: OutputKind,
    schema_version: SchemaVersion,
    generation: StorageGeneration,
    rust_type: TypeId,
    rust_type_name: &'static str,
    retain_owners: HashSet<RetainOwner>,
    state: SlotState,
}

#[derive(Debug)]
enum SlotState {
    Pending,
    Abandoned,
    Sealed(Box<SealedSlot>),
}

#[derive(Debug)]
struct SealedSlot {
    handle: AnyPhaseOutputRef,
    payload: PayloadStorage,
    side_tables: Arc<IrSideTables>,
}

#[derive(Clone)]
enum PayloadStorage {
    Resident(Arc<dyn Any + Send + Sync>),
    Blob {
        blob: ContentBlobId,
        decoder: ErasedBlobDecoder,
    },
}

type ErasedBlobDecoder =
    Arc<dyn Fn(&[u8]) -> Result<Arc<dyn Any + Send + Sync>, BlobDecodeError> + Send + Sync>;
type TypedBlobDecoder<T> = Arc<dyn Fn(&[u8]) -> Result<T, BlobDecodeError> + Send + Sync>;

enum PendingPayload {
    Resident(Arc<dyn Any + Send + Sync>),
    BlobCandidate {
        canonical_bytes: Vec<u8>,
        decode: ErasedBlobDecoder,
    },
}

enum PayloadRead {
    Resident(Arc<dyn Any + Send + Sync>),
    Blob {
        output: PhaseOutputId,
        schema_version: SchemaVersion,
        blob: ContentBlobId,
        bytes: Arc<Vec<u8>>,
        decoder: ErasedBlobDecoder,
    },
}

impl fmt::Debug for PayloadStorage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resident(_) => formatter.write_str("Resident(..)"),
            Self::Blob { blob, .. } => formatter.debug_struct("Blob").field("blob", blob).finish(),
        }
    }
}

/// Errors reported by IR storage.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StorageError {
    /// The slot is unknown to this storage service.
    UnknownSlot {
        /// Missing slot id.
        slot: OutputSlotId,
    },
    /// The phase output is unknown to this storage service.
    UnknownOutput {
        /// Missing output id.
        output: PhaseOutputId,
    },
    /// The output exists but has not been sealed.
    UnsealedOutput {
        /// Pending slot id.
        slot: OutputSlotId,
    },
    /// The pending slot was abandoned after a failed seal.
    AbandonedOutput {
        /// Abandoned slot id.
        slot: OutputSlotId,
    },
    /// The pending slot has already been sealed.
    AlreadySealed {
        /// Slot id.
        slot: OutputSlotId,
    },
    /// Another sealed slot already owns this output id.
    OutputAlreadyStored {
        /// Output id.
        output: PhaseOutputId,
    },
    /// The pending slot metadata does not match the output lineage.
    MetadataMismatch {
        /// Mismatched metadata field.
        field: &'static str,
    },
    /// The output lineage no longer matches its canonical identity.
    InvalidLineage {
        /// Output id carried by the lineage.
        output: PhaseOutputId,
        /// Identity validation error.
        error: Box<IdentityError>,
    },
    /// The handle or erased handle has the wrong runtime/Rust type.
    TypeMismatch {
        /// Expected Rust type name.
        expected: &'static str,
        /// Stored Rust type name.
        actual: &'static str,
    },
    /// The output was collected.
    CollectedOutput {
        /// Collected output id.
        output: PhaseOutputId,
    },
    /// The handle generation is stale.
    StaleHandle {
        /// Output id from the stale handle.
        output: PhaseOutputId,
    },
    /// A blob-backed payload failed validation or decode.
    CorruptBlob {
        /// Output id.
        output: PhaseOutputId,
    },
}

impl IrStorageService {
    /// Creates an empty storage service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty storage service with a custom policy.
    pub fn with_policy(policy: StoragePolicy) -> Self {
        Self {
            policy,
            state: Mutex::default(),
        }
    }

    /// Allocates a producer-private pending slot.
    pub fn allocate<T>(&self, input: AllocateSlotInput) -> PendingOutputSlot<T>
    where
        T: Send + Sync + 'static,
    {
        let mut state = self.state.lock().expect("IR storage mutex poisoned");
        let slot = OutputSlotId(state.next_slot_id);
        state.next_slot_id += 1;
        let generation = StorageGeneration(0);
        state.slots.insert(
            slot,
            SlotRecord {
                snapshot: input.snapshot,
                phase: input.phase.clone(),
                work_unit: input.work_unit.clone(),
                output_kind: input.output_kind.clone(),
                schema_version: input.schema_version,
                generation,
                rust_type: TypeId::of::<T>(),
                rust_type_name: type_name::<T>(),
                retain_owners: HashSet::new(),
                state: SlotState::Pending,
            },
        );

        PendingOutputSlot {
            id: slot,
            snapshot: input.snapshot,
            phase: input.phase,
            work_unit: input.work_unit,
            output_kind: input.output_kind,
            schema_version: input.schema_version,
            generation,
            marker: PhantomData,
        }
    }

    /// Seals a pending slot exactly once and returns a typed handle.
    pub fn seal<T>(&self, input: SealOutputInput<T>) -> Result<PhaseOutputRef<T>, StorageError>
    where
        T: Send + Sync + 'static,
    {
        self.seal_with_payload(
            input.slot,
            input.lineage,
            input.side_tables,
            PendingPayload::Resident(Arc::new(input.payload)),
        )
    }

    /// Seals a pending slot from canonical bytes, spilling to a blob when the
    /// configured threshold is exceeded.
    pub fn seal_blob<T>(
        &self,
        input: SealBlobOutputInput<T>,
    ) -> Result<PhaseOutputRef<T>, StorageError>
    where
        T: Send + Sync + 'static,
    {
        let output = input.lineage.output;
        let decode = input.decode.into_erased();
        if input.canonical_bytes.len() <= self.policy.blob_spill_threshold {
            self.validate_pending_before_decode(&input.slot, &input.lineage)?;
            let payload = run_decoder(&decode, &input.canonical_bytes)
                .map_err(|_| StorageError::CorruptBlob { output });
            let payload = match payload {
                Ok(payload) => payload,
                Err(error) => {
                    self.abandon_pending_slot(input.slot.id);
                    return Err(error);
                }
            };
            return self.seal_with_payload(
                input.slot,
                input.lineage,
                input.side_tables,
                PendingPayload::Resident(payload),
            );
        }

        self.seal_with_payload(
            input.slot,
            input.lineage,
            input.side_tables,
            PendingPayload::BlobCandidate {
                canonical_bytes: input.canonical_bytes,
                decode,
            },
        )
    }

    /// Seals a complete payload while using canonical bytes for placement.
    pub fn seal_canonical<T>(
        &self,
        input: SealCanonicalOutputInput<T>,
    ) -> Result<PhaseOutputRef<T>, StorageError>
    where
        T: Send + Sync + 'static,
    {
        if input.canonical_bytes.len() <= self.policy.blob_spill_threshold {
            return self.seal_with_payload(
                input.slot,
                input.lineage,
                input.side_tables,
                PendingPayload::Resident(Arc::new(input.payload)),
            );
        }

        self.seal_with_payload(
            input.slot,
            input.lineage,
            input.side_tables,
            PendingPayload::BlobCandidate {
                canonical_bytes: input.canonical_bytes,
                decode: input.decode.into_erased(),
            },
        )
    }

    fn seal_with_payload<T>(
        &self,
        slot: PendingOutputSlot<T>,
        lineage: PhaseOutputLineage,
        side_tables: IrSideTables,
        payload: PendingPayload,
    ) -> Result<PhaseOutputRef<T>, StorageError>
    where
        T: Send + Sync + 'static,
    {
        let mut state = self.state.lock().expect("IR storage mutex poisoned");
        let slot_id = slot.id;
        let output = lineage.output;

        let existing_slot = state.outputs.get(&output).copied();
        let mut record = state
            .slots
            .remove(&slot_id)
            .ok_or(StorageError::UnknownSlot { slot: slot_id })?;
        match record.state {
            SlotState::Pending => {}
            SlotState::Abandoned => {
                state.slots.insert(slot_id, record);
                return Err(StorageError::AbandonedOutput { slot: slot_id });
            }
            SlotState::Sealed(_) => {
                state.slots.insert(slot_id, record);
                return Err(StorageError::AlreadySealed { slot: slot_id });
            }
        }

        if let Some(existing_slot) = existing_slot
            && existing_slot != slot_id
        {
            record.state = SlotState::Abandoned;
            state.slots.insert(slot_id, record);
            return Err(StorageError::OutputAlreadyStored { output });
        }

        if let Err(error) = reject_stale_pending(&slot, &record, output) {
            record.state = SlotState::Abandoned;
            state.slots.insert(slot_id, record);
            return Err(error);
        }
        if let Err(error) = reject_lineage_mismatch(&slot, &lineage) {
            record.state = SlotState::Abandoned;
            state.slots.insert(slot_id, record);
            return Err(error);
        }
        if let Err(error) = lineage.validate_identity() {
            record.state = SlotState::Abandoned;
            state.slots.insert(slot_id, record);
            return Err(StorageError::InvalidLineage {
                output,
                error: Box::new(error),
            });
        }

        let (payload, placement) = build_payload_storage(&mut state, slot.schema_version, payload);
        let generation = StorageGeneration(*state.output_generations.entry(output).or_insert(0));
        record.generation = generation;
        let handle = AnyPhaseOutputRef {
            lineage,
            schema_version: slot.schema_version,
            generation,
            placement,
        };
        let typed = PhaseOutputRef {
            inner: handle.clone(),
            marker: PhantomData,
        };
        record.state = SlotState::Sealed(Box::new(SealedSlot {
            handle,
            payload,
            side_tables: Arc::new(side_tables),
        }));
        state.outputs.insert(output, slot_id);
        state.slots.insert(slot_id, record);

        Ok(typed)
    }

    fn validate_pending_before_decode<T>(
        &self,
        slot: &PendingOutputSlot<T>,
        lineage: &PhaseOutputLineage,
    ) -> Result<(), StorageError>
    where
        T: Send + Sync + 'static,
    {
        let mut state = self.state.lock().expect("IR storage mutex poisoned");
        let output = lineage.output;
        let existing_slot = state.outputs.get(&output).copied();
        let record = state
            .slots
            .get_mut(&slot.id)
            .ok_or(StorageError::UnknownSlot { slot: slot.id })?;
        match record.state {
            SlotState::Pending => {}
            SlotState::Abandoned => return Err(StorageError::AbandonedOutput { slot: slot.id }),
            SlotState::Sealed(_) => return Err(StorageError::AlreadySealed { slot: slot.id }),
        }
        if let Some(existing_slot) = existing_slot
            && existing_slot != slot.id
        {
            record.state = SlotState::Abandoned;
            return Err(StorageError::OutputAlreadyStored { output });
        }
        if let Err(error) = reject_stale_pending(slot, record, output) {
            record.state = SlotState::Abandoned;
            return Err(error);
        }
        if let Err(error) = reject_lineage_mismatch(slot, lineage) {
            record.state = SlotState::Abandoned;
            return Err(error);
        }
        if let Err(error) = lineage.validate_identity() {
            record.state = SlotState::Abandoned;
            return Err(StorageError::InvalidLineage {
                output,
                error: Box::new(error),
            });
        }
        Ok(())
    }

    pub(crate) fn abandon<T>(&self, slot: PendingOutputSlot<T>) -> bool {
        self.abandon_pending_slot(slot.id)
    }

    fn abandon_pending_slot(&self, slot: OutputSlotId) -> bool {
        let mut state = self.state.lock().expect("IR storage mutex poisoned");
        if let Some(record) = state.slots.get_mut(&slot)
            && matches!(record.state, SlotState::Pending)
        {
            record.state = SlotState::Abandoned;
            return true;
        }
        false
    }

    /// Reads a sealed payload through a typed handle.
    pub fn get<T>(&self, handle: &PhaseOutputRef<T>) -> Result<Arc<T>, StorageError>
    where
        T: Send + Sync + 'static,
    {
        let read = {
            let state = self.state.lock().expect("IR storage mutex poisoned");
            let record = sealed_record_for_handle(&state, handle.any())?;
            reject_rust_type::<T>(record)?;
            let SlotState::Sealed(sealed) = &record.state else {
                return Err(StorageError::UnsealedOutput {
                    slot: slot_for_handle(&state, handle.any())?,
                });
            };
            payload_read_request(
                &state,
                handle.output(),
                record.schema_version,
                &sealed.payload,
            )?
        };
        let payload = read.resolve()?;

        payload
            .downcast::<T>()
            .map_err(|_| StorageError::TypeMismatch {
                expected: type_name::<T>(),
                actual: "unknown",
            })
    }

    /// Returns side tables attached to a sealed typed handle.
    pub fn side_tables<T>(
        &self,
        handle: &PhaseOutputRef<T>,
    ) -> Result<Arc<IrSideTables>, StorageError>
    where
        T: Send + Sync + 'static,
    {
        self.side_tables_by_ref(handle.any())
    }

    /// Returns side tables attached to an erased sealed handle.
    pub fn side_tables_by_ref(
        &self,
        handle: &AnyPhaseOutputRef,
    ) -> Result<Arc<IrSideTables>, StorageError> {
        let state = self.state.lock().expect("IR storage mutex poisoned");
        let record = sealed_record_for_handle(&state, handle)?;
        let SlotState::Sealed(sealed) = &record.state else {
            return Err(StorageError::UnsealedOutput {
                slot: slot_for_handle(&state, handle)?,
            });
        };
        Ok(sealed.side_tables.clone())
    }

    /// Validates that an erased handle still names a sealed output in this
    /// storage service.
    pub fn validate_handle(&self, handle: &AnyPhaseOutputRef) -> Result<(), StorageError> {
        let state = self.state.lock().expect("IR storage mutex poisoned");
        let record = sealed_record_for_handle(&state, handle)?;
        if !matches!(record.state, SlotState::Sealed(_)) {
            return Err(StorageError::UnsealedOutput {
                slot: slot_for_handle(&state, handle)?,
            });
        }
        Ok(())
    }

    /// Validates an erased handle for the requested type and output kind.
    pub fn typed_handle<T>(
        &self,
        handle: &AnyPhaseOutputRef,
        expected_kind: &OutputKind,
    ) -> Result<PhaseOutputRef<T>, StorageError>
    where
        T: Send + Sync + 'static,
    {
        let state = self.state.lock().expect("IR storage mutex poisoned");
        let record = sealed_record_for_handle(&state, handle)?;
        if &record.output_kind != expected_kind {
            return Err(StorageError::TypeMismatch {
                expected: type_name::<T>(),
                actual: record.rust_type_name,
            });
        }
        reject_rust_type::<T>(record)?;
        Ok(PhaseOutputRef {
            inner: handle.clone(),
            marker: PhantomData,
        })
    }

    /// Retains a sealed output for an explicit owner.
    pub fn retain(&self, output: PhaseOutputId, owner: RetainOwner) -> Result<(), StorageError> {
        let mut state = self.state.lock().expect("IR storage mutex poisoned");
        let slot = slot_for_output(&state, output)?;
        let record = state
            .slots
            .get_mut(&slot)
            .ok_or(StorageError::UnknownOutput { output })?;
        if !matches!(record.state, SlotState::Sealed(_)) {
            return Err(StorageError::UnsealedOutput { slot });
        }
        record.retain_owners.insert(owner);
        Ok(())
    }

    /// Releases a retain owner from a sealed output.
    pub fn release(
        &self,
        output: PhaseOutputId,
        owner: &RetainOwner,
    ) -> Result<bool, StorageError> {
        let mut state = self.state.lock().expect("IR storage mutex poisoned");
        let slot = slot_for_output(&state, output)?;
        let record = state
            .slots
            .get_mut(&slot)
            .ok_or(StorageError::UnknownOutput { output })?;
        Ok(record.retain_owners.remove(owner))
    }

    /// Collects unretained outputs for one snapshot.
    pub fn collect(&self, input: CollectInput) -> CollectionSummary {
        let mut state = self.state.lock().expect("IR storage mutex poisoned");
        let protected = input.protected_outputs.into_iter().collect::<HashSet<_>>();
        let mut summary = CollectionSummary::default();
        let mut drop_slots = Vec::new();

        for (slot, record) in &state.slots {
            if record.snapshot != input.snapshot {
                continue;
            }
            match &record.state {
                SlotState::Pending => {}
                SlotState::Abandoned => drop_slots.push(*slot),
                SlotState::Sealed(sealed) => {
                    let output = sealed.handle.output();
                    if !record.retain_owners.is_empty() {
                        summary.retained_outputs += 1;
                    } else if protected.contains(&output) {
                        summary.protected_outputs += 1;
                    } else {
                        drop_slots.push(*slot);
                    }
                }
            }
        }

        for slot in drop_slots {
            let Some(record) = state.slots.remove(&slot) else {
                continue;
            };
            match record.state {
                SlotState::Pending => {}
                SlotState::Abandoned => {
                    summary.abandoned_slots_dropped += 1;
                }
                SlotState::Sealed(sealed) => {
                    let output = sealed.handle.output();
                    let next_generation = sealed.handle.generation().get() + 1;
                    state
                        .output_generations
                        .entry(output)
                        .and_modify(|generation| *generation = (*generation).max(next_generation))
                        .or_insert(next_generation);
                    state.outputs.remove(&output);
                    state.collected_outputs.insert(output);
                    summary.outputs_dropped += 1;
                    if let PayloadStorage::Blob { blob, .. } = sealed.payload
                        && !blob_is_referenced(&state, blob)
                        && state.blobs.remove(&blob).is_some()
                    {
                        summary.blobs_dropped += 1;
                    }
                }
            }
        }

        summary
    }

    #[cfg(test)]
    fn get_pending_for_test<T>(&self, slot: &PendingOutputSlot<T>) -> Result<Arc<T>, StorageError>
    where
        T: Send + Sync + 'static,
    {
        let state = self.state.lock().expect("IR storage mutex poisoned");
        let record = state
            .slots
            .get(&slot.id)
            .ok_or(StorageError::UnknownSlot { slot: slot.id })?;
        if matches!(record.state, SlotState::Pending) {
            return Err(StorageError::UnsealedOutput { slot: slot.id });
        }
        if matches!(record.state, SlotState::Abandoned) {
            return Err(StorageError::AbandonedOutput { slot: slot.id });
        }
        Err(StorageError::AlreadySealed { slot: slot.id })
    }
}

impl SchemaVersion {
    /// Creates a schema version.
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the numeric schema version.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl ContentBlobId {
    /// Returns the opaque hash backing this blob id.
    pub const fn hash(self) -> Hash {
        self.0
    }
}

impl Default for StoragePolicy {
    fn default() -> Self {
        Self {
            blob_spill_threshold: DEFAULT_BLOB_SPILL_THRESHOLD,
        }
    }
}

impl StoragePolicy {
    /// Creates a storage policy with a custom blob spill threshold.
    pub const fn with_blob_spill_threshold(blob_spill_threshold: usize) -> Self {
        Self {
            blob_spill_threshold,
        }
    }
}

impl<T> BlobDecoder<T> {
    /// Creates a typed blob decoder.
    pub fn new(
        decode: impl Fn(&[u8]) -> Result<T, BlobDecodeError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            decode: Arc::new(decode),
        }
    }

    /// Decodes canonical bytes into a typed payload.
    pub fn decode(&self, bytes: &[u8]) -> Result<T, BlobDecodeError> {
        (self.decode)(bytes)
    }

    fn into_erased(self) -> ErasedBlobDecoder
    where
        T: Send + Sync + 'static,
    {
        let decode = self.decode;
        Arc::new(move |bytes| {
            let value = decode(bytes)?;
            Ok(Arc::new(value) as Arc<dyn Any + Send + Sync>)
        })
    }
}

impl<T> Clone for BlobDecoder<T> {
    fn clone(&self) -> Self {
        Self {
            decode: self.decode.clone(),
        }
    }
}

impl BlobDecodeError {
    /// Creates a blob decode error message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BlobDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for BlobDecodeError {}

impl RetainOwner {
    /// Creates a retain owner label.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the owner label.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl OutputSlotId {
    /// Returns the process-local slot number.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl StorageGeneration {
    /// Returns the process-local generation number.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl SideTableRecord {
    /// Creates a side-table record.
    pub fn new(domain: impl Into<String>, key: impl Into<String>, digest: Hash) -> Self {
        Self {
            domain: domain.into(),
            key: key.into(),
            digest,
        }
    }
}

impl<T> Clone for PendingOutputSlot<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            snapshot: self.snapshot,
            phase: self.phase.clone(),
            work_unit: self.work_unit.clone(),
            output_kind: self.output_kind.clone(),
            schema_version: self.schema_version,
            generation: self.generation,
            marker: PhantomData,
        }
    }
}

impl<T> PendingOutputSlot<T> {
    /// Returns the process-local slot id.
    pub const fn slot(&self) -> OutputSlotId {
        self.id
    }

    /// Returns the owning snapshot.
    pub const fn snapshot(&self) -> BuildSnapshotId {
        self.snapshot
    }

    /// Returns the producing phase.
    pub fn phase(&self) -> &PipelinePhase {
        &self.phase
    }

    /// Returns the producing work unit.
    pub fn work_unit(&self) -> &WorkUnit {
        &self.work_unit
    }

    /// Returns the runtime output kind.
    pub fn output_kind(&self) -> &OutputKind {
        &self.output_kind
    }

    /// Returns the schema version.
    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    /// Returns the storage generation.
    pub const fn generation(&self) -> StorageGeneration {
        self.generation
    }
}

impl AnyPhaseOutputRef {
    /// Returns the output id.
    pub const fn output(&self) -> PhaseOutputId {
        self.lineage.output
    }

    /// Returns the owning snapshot.
    pub const fn snapshot(&self) -> BuildSnapshotId {
        self.lineage.snapshot
    }

    /// Returns the producing phase.
    pub fn phase(&self) -> &PipelinePhase {
        &self.lineage.phase
    }

    /// Returns the producing work unit.
    pub fn work_unit(&self) -> &WorkUnit {
        &self.lineage.work_unit
    }

    /// Returns the runtime output kind.
    pub fn output_kind(&self) -> &OutputKind {
        &self.lineage.output_kind
    }

    /// Returns the content hash.
    pub const fn content_hash(&self) -> Hash {
        self.lineage.content_hash
    }

    /// Returns the side-table hash.
    pub const fn side_table_hash(&self) -> Hash {
        self.lineage.side_table_hash
    }

    /// Returns the schema version.
    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    /// Returns the storage generation.
    pub const fn generation(&self) -> StorageGeneration {
        self.generation
    }

    /// Returns the payload placement.
    pub const fn placement(&self) -> &StoragePlacement {
        &self.placement
    }

    /// Returns output lineage.
    pub const fn lineage(&self) -> &PhaseOutputLineage {
        &self.lineage
    }
}

impl<T> Clone for PhaseOutputRef<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> PartialEq for PhaseOutputRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for PhaseOutputRef<T> {}

impl<T> PhaseOutputRef<T> {
    /// Erases the payload type.
    pub fn erase(&self) -> AnyPhaseOutputRef {
        self.inner.clone()
    }

    /// Borrows the erased handle.
    pub const fn any(&self) -> &AnyPhaseOutputRef {
        &self.inner
    }

    /// Returns the output id.
    pub const fn output(&self) -> PhaseOutputId {
        self.inner.output()
    }

    /// Returns the owning snapshot.
    pub const fn snapshot(&self) -> BuildSnapshotId {
        self.inner.snapshot()
    }

    /// Returns the producing phase.
    pub fn phase(&self) -> &PipelinePhase {
        self.inner.phase()
    }

    /// Returns the producing work unit.
    pub fn work_unit(&self) -> &WorkUnit {
        self.inner.work_unit()
    }

    /// Returns the runtime output kind.
    pub fn output_kind(&self) -> &OutputKind {
        self.inner.output_kind()
    }

    /// Returns the content hash.
    pub const fn content_hash(&self) -> Hash {
        self.inner.content_hash()
    }

    /// Returns the side-table hash.
    pub const fn side_table_hash(&self) -> Hash {
        self.inner.side_table_hash()
    }

    /// Returns the schema version.
    pub const fn schema_version(&self) -> SchemaVersion {
        self.inner.schema_version()
    }

    /// Returns the payload placement.
    pub const fn placement(&self) -> &StoragePlacement {
        self.inner.placement()
    }

    /// Returns output lineage.
    pub const fn lineage(&self) -> &PhaseOutputLineage {
        self.inner.lineage()
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSlot { slot } => write!(formatter, "unknown IR storage slot `{slot:?}`"),
            Self::UnknownOutput { output } => {
                write!(formatter, "unknown IR phase output `{output:?}`")
            }
            Self::UnsealedOutput { slot } => {
                write!(formatter, "IR storage slot `{slot:?}` has not been sealed")
            }
            Self::AbandonedOutput { slot } => {
                write!(formatter, "IR storage slot `{slot:?}` was abandoned")
            }
            Self::AlreadySealed { slot } => {
                write!(formatter, "IR storage slot `{slot:?}` is already sealed")
            }
            Self::OutputAlreadyStored { output } => {
                write!(formatter, "IR phase output `{output:?}` is already stored")
            }
            Self::MetadataMismatch { field } => {
                write!(formatter, "IR storage metadata mismatch for `{field}`")
            }
            Self::InvalidLineage { output, error } => {
                write!(
                    formatter,
                    "IR storage rejected invalid lineage for `{output:?}`: {error}"
                )
            }
            Self::TypeMismatch { expected, actual } => {
                write!(
                    formatter,
                    "IR storage type mismatch: expected `{expected}`, found `{actual}`"
                )
            }
            Self::CollectedOutput { output } => {
                write!(formatter, "IR phase output `{output:?}` was collected")
            }
            Self::StaleHandle { output } => {
                write!(
                    formatter,
                    "IR phase output handle `{output:?}` has a stale storage generation"
                )
            }
            Self::CorruptBlob { output } => {
                write!(formatter, "IR phase output `{output:?}` has a corrupt blob")
            }
        }
    }
}

impl Error for StorageError {}

fn reject_stale_pending<T>(
    slot: &PendingOutputSlot<T>,
    record: &SlotRecord,
    output: PhaseOutputId,
) -> Result<(), StorageError>
where
    T: 'static,
{
    if record.generation != slot.generation {
        return Err(StorageError::StaleHandle { output });
    }
    if record.rust_type != TypeId::of::<T>() {
        return Err(StorageError::TypeMismatch {
            expected: type_name::<T>(),
            actual: record.rust_type_name,
        });
    }
    Ok(())
}

fn reject_lineage_mismatch<T>(
    slot: &PendingOutputSlot<T>,
    lineage: &PhaseOutputLineage,
) -> Result<(), StorageError> {
    if slot.snapshot != lineage.snapshot {
        return Err(StorageError::MetadataMismatch { field: "snapshot" });
    }
    if slot.phase != lineage.phase {
        return Err(StorageError::MetadataMismatch { field: "phase" });
    }
    if slot.work_unit != lineage.work_unit {
        return Err(StorageError::MetadataMismatch { field: "work_unit" });
    }
    if slot.output_kind != lineage.output_kind {
        return Err(StorageError::MetadataMismatch {
            field: "output_kind",
        });
    }
    Ok(())
}

fn sealed_record_for_handle<'a>(
    state: &'a StorageState,
    handle: &AnyPhaseOutputRef,
) -> Result<&'a SlotRecord, StorageError> {
    let slot = slot_for_handle(state, handle)?;
    let record = state.slots.get(&slot).ok_or(StorageError::UnknownOutput {
        output: handle.output(),
    })?;
    if record.generation != handle.generation {
        return Err(collected_or_stale_error(state, handle));
    }
    if record.snapshot != handle.snapshot() {
        return Err(StorageError::StaleHandle {
            output: handle.output(),
        });
    }
    if record.phase != *handle.phase()
        || record.work_unit != *handle.work_unit()
        || record.output_kind != *handle.output_kind()
        || record.schema_version != handle.schema_version
    {
        return Err(StorageError::StaleHandle {
            output: handle.output(),
        });
    }
    if let SlotState::Sealed(sealed) = &record.state
        && sealed.handle != *handle
    {
        return Err(collected_or_stale_error(state, handle));
    }
    Ok(record)
}

fn slot_for_handle(
    state: &StorageState,
    handle: &AnyPhaseOutputRef,
) -> Result<OutputSlotId, StorageError> {
    slot_for_output(state, handle.output())
}

fn slot_for_output(
    state: &StorageState,
    output: PhaseOutputId,
) -> Result<OutputSlotId, StorageError> {
    if let Some(slot) = state.outputs.get(&output).copied() {
        Ok(slot)
    } else if state.collected_outputs.contains(&output) {
        Err(StorageError::CollectedOutput { output })
    } else {
        Err(StorageError::UnknownOutput { output })
    }
}

fn collected_or_stale_error(state: &StorageState, handle: &AnyPhaseOutputRef) -> StorageError {
    if state
        .output_generations
        .get(&handle.output())
        .is_some_and(|generation| handle.generation().get() < *generation)
        || state.collected_outputs.contains(&handle.output())
    {
        StorageError::CollectedOutput {
            output: handle.output(),
        }
    } else {
        StorageError::StaleHandle {
            output: handle.output(),
        }
    }
}

fn reject_rust_type<T>(record: &SlotRecord) -> Result<(), StorageError>
where
    T: Send + Sync + 'static,
{
    if record.rust_type == TypeId::of::<T>() {
        Ok(())
    } else {
        Err(StorageError::TypeMismatch {
            expected: type_name::<T>(),
            actual: record.rust_type_name,
        })
    }
}

fn build_payload_storage(
    state: &mut StorageState,
    schema_version: SchemaVersion,
    payload: PendingPayload,
) -> (PayloadStorage, StoragePlacement) {
    match payload {
        PendingPayload::Resident(value) => {
            (PayloadStorage::Resident(value), StoragePlacement::Resident)
        }
        PendingPayload::BlobCandidate {
            canonical_bytes,
            decode,
        } => {
            let blob = content_blob_id(schema_version, &canonical_bytes);
            let len = canonical_bytes.len();
            state
                .blobs
                .entry(blob)
                .or_insert_with(|| Arc::new(canonical_bytes));
            (
                PayloadStorage::Blob {
                    blob,
                    decoder: decode,
                },
                StoragePlacement::Blob { blob, len },
            )
        }
    }
}

fn payload_read_request(
    state: &StorageState,
    output: PhaseOutputId,
    schema_version: SchemaVersion,
    payload: &PayloadStorage,
) -> Result<PayloadRead, StorageError> {
    match payload {
        PayloadStorage::Resident(value) => Ok(PayloadRead::Resident(value.clone())),
        PayloadStorage::Blob { blob, decoder } => {
            let bytes = state
                .blobs
                .get(blob)
                .ok_or(StorageError::CorruptBlob { output })?;
            Ok(PayloadRead::Blob {
                output,
                schema_version,
                blob: *blob,
                bytes: bytes.clone(),
                decoder: decoder.clone(),
            })
        }
    }
}

impl PayloadRead {
    fn resolve(self) -> Result<Arc<dyn Any + Send + Sync>, StorageError> {
        match self {
            Self::Resident(value) => Ok(value),
            Self::Blob {
                output,
                schema_version,
                blob,
                bytes,
                decoder,
            } => {
                if content_blob_id(schema_version, bytes.as_slice()) != blob {
                    return Err(StorageError::CorruptBlob { output });
                }
                run_decoder(&decoder, bytes.as_slice())
                    .map_err(|_| StorageError::CorruptBlob { output })
            }
        }
    }
}

fn run_decoder(
    decoder: &ErasedBlobDecoder,
    bytes: &[u8],
) -> Result<Arc<dyn Any + Send + Sync>, BlobDecodeError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| decoder(bytes)))
        .map_err(|_| BlobDecodeError::new("blob decoder panicked"))?
}

fn content_blob_id(schema_version: SchemaVersion, bytes: &[u8]) -> ContentBlobId {
    let mut hasher = blake3::Hasher::new();
    write_blob_field(&mut hasher, "mizar-ir/content-blob/v1");
    hasher.update(&schema_version.get().to_le_bytes());
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
    ContentBlobId(Hash::from_bytes(*hasher.finalize().as_bytes()))
}

fn write_blob_field(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value.as_bytes());
}

fn blob_is_referenced(state: &StorageState, blob: ContentBlobId) -> bool {
    state.slots.values().any(|record| {
        matches!(
            &record.state,
            SlotState::Sealed(sealed)
                if matches!(&sealed.payload, PayloadStorage::Blob { blob: value, .. } if *value == blob)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{NamedInputHash, OutputIdentityInput, SnapshotHandleRegistry};

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

    fn work_unit(name: &str) -> WorkUnit {
        WorkUnit::new(name)
    }

    fn output_kind() -> OutputKind {
        OutputKind::new("ResolvedAst")
    }

    fn allocation(snapshot: BuildSnapshotId, work_unit: &str) -> AllocateSlotInput {
        AllocateSlotInput {
            snapshot,
            phase: phase(),
            work_unit: self::work_unit(work_unit),
            output_kind: output_kind(),
            schema_version: SchemaVersion::new(1),
        }
    }

    fn lineage(snapshot: BuildSnapshotId, work_unit: &str, content_seed: u8) -> PhaseOutputLineage {
        let registry = SnapshotHandleRegistry::new();
        registry.register_snapshot(snapshot);
        registry
            .register_output(OutputIdentityInput {
                snapshot,
                phase: phase(),
                work_unit: self::work_unit(work_unit),
                output_kind: output_kind(),
                content_hash: hash(content_seed),
                side_table_hash: hash(80),
                parents: Vec::new(),
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: "test".to_owned(),
                    digest: hash(81),
                }],
            })
            .expect("output lineage is registered")
    }

    #[test]
    fn access_before_seal_fails() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(1);
        let pending = storage.allocate::<String>(allocation(snapshot, "unit"));

        let error = storage
            .get_pending_for_test(&pending)
            .expect_err("pending outputs are not readable");

        assert_eq!(
            error,
            StorageError::UnsealedOutput {
                slot: pending.slot(),
            }
        );
    }

    #[test]
    fn double_seal_is_rejected_and_first_value_remains() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(2);
        let pending = storage.allocate::<String>(allocation(snapshot, "unit"));
        let first = storage
            .seal(SealOutputInput {
                slot: pending.clone(),
                lineage: lineage(snapshot, "unit", 10),
                side_tables: IrSideTables::default(),
                payload: "first".to_owned(),
            })
            .expect("first seal succeeds");

        let error = storage
            .seal(SealOutputInput {
                slot: pending.clone(),
                lineage: lineage(snapshot, "unit", 10),
                side_tables: IrSideTables::default(),
                payload: "second".to_owned(),
            })
            .expect_err("second seal must fail");

        assert_eq!(
            error,
            StorageError::AlreadySealed {
                slot: pending.slot(),
            }
        );
        assert_eq!(
            &*storage.get(&first).expect("first payload remains"),
            "first"
        );
    }

    #[test]
    fn handle_typing_round_trips_payload_and_metadata() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(3);
        let pending = storage.allocate::<Vec<u8>>(allocation(snapshot, "unit"));
        let lineage = lineage(snapshot, "unit", 11);
        let handle = storage
            .seal(SealOutputInput {
                slot: pending,
                lineage: lineage.clone(),
                side_tables: IrSideTables::default(),
                payload: vec![1, 2, 3],
            })
            .expect("seal succeeds");

        assert_eq!(
            *storage.get(&handle).expect("payload round-trips"),
            vec![1, 2, 3]
        );
        assert_eq!(handle.output(), lineage.output);
        assert_eq!(handle.snapshot(), snapshot);
        assert_eq!(handle.lineage(), &lineage);
        assert_eq!(handle.schema_version(), SchemaVersion::new(1));
    }

    #[test]
    fn erased_handle_rejects_wrong_type() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(4);
        let pending = storage.allocate::<String>(allocation(snapshot, "unit"));
        let handle = storage
            .seal(SealOutputInput {
                slot: pending,
                lineage: lineage(snapshot, "unit", 12),
                side_tables: IrSideTables::default(),
                payload: "text".to_owned(),
            })
            .expect("seal succeeds");
        let erased = handle.erase();

        let error = storage
            .typed_handle::<Vec<u8>>(&erased, &output_kind())
            .expect_err("wrong Rust payload type must fail");

        assert!(matches!(error, StorageError::TypeMismatch { .. }));
    }

    #[test]
    fn wrong_output_kind_is_rejected_before_typed_handle_recovery() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(5);
        let pending = storage.allocate::<String>(allocation(snapshot, "unit"));
        let handle = storage
            .seal(SealOutputInput {
                slot: pending,
                lineage: lineage(snapshot, "unit", 13),
                side_tables: IrSideTables::default(),
                payload: "text".to_owned(),
            })
            .expect("seal succeeds");

        let error = storage
            .typed_handle::<String>(&handle.erase(), &OutputKind::new("TypedAst"))
            .expect_err("wrong runtime kind must fail");

        assert!(matches!(error, StorageError::TypeMismatch { .. }));
    }

    #[test]
    fn seal_rejects_lineage_metadata_mismatch() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(6);
        let pending = storage.allocate::<String>(allocation(snapshot, "allocated"));

        let error = storage
            .seal(SealOutputInput {
                slot: pending,
                lineage: lineage(snapshot, "different", 14),
                side_tables: IrSideTables::default(),
                payload: "text".to_owned(),
            })
            .expect_err("lineage must match pending slot metadata");

        assert_eq!(error, StorageError::MetadataMismatch { field: "work_unit" });
    }

    #[test]
    fn failed_first_seal_abandons_pending_slot() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(9);
        let pending = storage.allocate::<String>(allocation(snapshot, "allocated"));

        let error = storage
            .seal(SealOutputInput {
                slot: pending.clone(),
                lineage: lineage(snapshot, "different", 17),
                side_tables: IrSideTables::default(),
                payload: "wrong".to_owned(),
            })
            .expect_err("lineage mismatch fails the first seal");
        assert_eq!(error, StorageError::MetadataMismatch { field: "work_unit" });

        let abandoned = storage
            .seal(SealOutputInput {
                slot: pending.clone(),
                lineage: lineage(snapshot, "allocated", 18),
                side_tables: IrSideTables::default(),
                payload: "correct".to_owned(),
            })
            .expect_err("abandoned pending slot must not be reused");

        assert_eq!(
            abandoned,
            StorageError::AbandonedOutput {
                slot: pending.slot(),
            }
        );
    }

    #[test]
    fn seal_rejects_lineage_that_no_longer_matches_output_id() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(10);
        let pending = storage.allocate::<String>(allocation(snapshot, "unit"));
        let mut mutated = lineage(snapshot, "unit", 19);
        mutated.content_hash = hash(200);

        let error = storage
            .seal(SealOutputInput {
                slot: pending.clone(),
                lineage: mutated,
                side_tables: IrSideTables::default(),
                payload: "tampered".to_owned(),
            })
            .expect_err("mutated lineage must fail identity validation");

        assert!(matches!(error, StorageError::InvalidLineage { .. }));
        assert_eq!(
            storage
                .seal(SealOutputInput {
                    slot: pending.clone(),
                    lineage: lineage(snapshot, "unit", 20),
                    side_tables: IrSideTables::default(),
                    payload: "correct".to_owned(),
                })
                .expect_err("failed seal abandons the slot"),
            StorageError::AbandonedOutput {
                slot: pending.slot(),
            }
        );
    }

    #[test]
    fn sealed_side_tables_round_trip() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(7);
        let pending = storage.allocate::<String>(allocation(snapshot, "unit"));
        let side_tables = IrSideTables {
            source_maps: vec![SideTableRecord::new("source-map", "Main", hash(90))],
            diagnostics: vec![SideTableRecord::new("diagnostic", "D001", hash(91))],
            explanation_refs: Vec::new(),
            documentation_attachments: Vec::new(),
        };
        let handle = storage
            .seal(SealOutputInput {
                slot: pending,
                lineage: lineage(snapshot, "unit", 15),
                side_tables: side_tables.clone(),
                payload: "text".to_owned(),
            })
            .expect("seal succeeds");

        assert_eq!(
            &*storage
                .side_tables(&handle)
                .expect("side tables round-trip"),
            &side_tables
        );
    }

    #[test]
    fn storage_handles_are_not_proof_or_trust_authority() {
        #[derive(Debug)]
        struct ProofShapedPayload {
            accepted: bool,
            trusted: bool,
            kernel_policy: &'static str,
        }

        let storage = IrStorageService::new();
        let snapshot = snapshot(8);
        let pending = storage.allocate::<ProofShapedPayload>(allocation(snapshot, "unit"));
        let handle = storage
            .seal(SealOutputInput {
                slot: pending,
                lineage: lineage(snapshot, "unit", 16),
                side_tables: IrSideTables::default(),
                payload: ProofShapedPayload {
                    accepted: true,
                    trusted: true,
                    kernel_policy: "strict",
                },
            })
            .expect("proof-shaped payload can be stored as opaque IR");
        let erased = handle.erase();
        let stored = storage
            .get(&handle)
            .expect("typed payload remains an internal value");

        assert!(stored.accepted);
        assert!(stored.trusted);
        assert_eq!(stored.kernel_policy, "strict");

        let handle_metadata = format!("{erased:?}");
        for forbidden in ["accepted", "trusted", "kernel", "policy"] {
            assert!(
                !handle_metadata.contains(forbidden),
                "storage handle metadata must not expose or promote `{forbidden}` authority"
            );
        }
    }

    #[test]
    fn blob_spill_round_trips_by_content_hash() {
        let storage = IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(3));
        let snapshot = snapshot(11);
        let payload = vec![1, 2, 3, 4];
        let handle = storage
            .seal_blob(SealBlobOutputInput {
                slot: storage.allocate::<Vec<u8>>(allocation(snapshot, "blob")),
                lineage: lineage(snapshot, "blob", 21),
                side_tables: IrSideTables::default(),
                canonical_bytes: payload.clone(),
                decode: BlobDecoder::new(|bytes| Ok(bytes.to_vec())),
            })
            .expect("blob-backed seal succeeds");

        let StoragePlacement::Blob { blob, len } = handle.placement() else {
            panic!("payload over threshold must be blob-backed");
        };

        assert_eq!(*len, payload.len());
        assert_eq!(*blob, content_blob_id(SchemaVersion::new(1), &payload));
        assert_eq!(
            *storage.get(&handle).expect("blob payload decodes"),
            payload
        );
    }

    #[test]
    fn blob_decode_failures_are_fail_closed() {
        let snapshot = snapshot(14);
        let below_threshold =
            IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(usize::MAX));
        let seal_error = below_threshold
            .seal_blob(SealBlobOutputInput::<String> {
                slot: below_threshold.allocate::<String>(allocation(snapshot, "below")),
                lineage: lineage(snapshot, "below", 27),
                side_tables: IrSideTables::default(),
                canonical_bytes: b"not-decodable".to_vec(),
                decode: BlobDecoder::new(|_| Err(BlobDecodeError::new("decode failed"))),
            })
            .expect_err("below-threshold decode failures fail during seal");
        assert!(matches!(seal_error, StorageError::CorruptBlob { .. }));

        let blob_backed =
            IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(0));
        let handle = blob_backed
            .seal_blob(SealBlobOutputInput::<String> {
                slot: blob_backed.allocate::<String>(allocation(snapshot, "above")),
                lineage: lineage(snapshot, "above", 28),
                side_tables: IrSideTables::default(),
                canonical_bytes: b"stored-but-not-decodable".to_vec(),
                decode: BlobDecoder::new(|_| Err(BlobDecodeError::new("decode failed"))),
            })
            .expect("above-threshold seal stores the blob before decode");

        assert!(matches!(
            blob_backed.get(&handle),
            Err(StorageError::CorruptBlob { output }) if output == handle.output()
        ));
    }

    #[test]
    fn blob_decoder_panic_is_fail_closed_without_poisoning_storage() {
        let storage = IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(0));
        let snapshot = snapshot(20);
        let handle = storage
            .seal_blob(SealBlobOutputInput::<String> {
                slot: storage.allocate::<String>(allocation(snapshot, "panic")),
                lineage: lineage(snapshot, "panic", 36),
                side_tables: IrSideTables::default(),
                canonical_bytes: b"panic".to_vec(),
                decode: BlobDecoder::new(|_| panic!("decoder panic")),
            })
            .expect("blob-backed seal does not run the decoder");

        assert!(matches!(
            storage.get(&handle),
            Err(StorageError::CorruptBlob { output }) if output == handle.output()
        ));

        let after = storage
            .seal(SealOutputInput {
                slot: storage.allocate::<String>(allocation(snapshot, "after-panic")),
                lineage: lineage(snapshot, "after-panic", 37),
                side_tables: IrSideTables::default(),
                payload: "after".to_owned(),
            })
            .expect("storage mutex was not poisoned by decoder panic");
        assert_eq!(&*storage.get(&after).expect("storage still works"), "after");
    }

    #[test]
    fn blob_decoder_can_reenter_storage_without_deadlock() {
        let storage = Arc::new(IrStorageService::with_policy(
            StoragePolicy::with_blob_spill_threshold(0),
        ));
        let snapshot = snapshot(21);
        let anchor = storage
            .seal(SealOutputInput {
                slot: storage.allocate::<String>(allocation(snapshot, "anchor")),
                lineage: lineage(snapshot, "anchor", 38),
                side_tables: IrSideTables::default(),
                payload: "anchor".to_owned(),
            })
            .expect("anchor output seals");
        let reentrant_storage = storage.clone();
        let reentrant_anchor = anchor.clone();
        let blob = storage
            .seal_blob(SealBlobOutputInput {
                slot: storage.allocate::<Vec<u8>>(allocation(snapshot, "reentrant")),
                lineage: lineage(snapshot, "reentrant", 39),
                side_tables: IrSideTables::default(),
                canonical_bytes: vec![7, 7, 7, 7],
                decode: BlobDecoder::new(move |bytes| {
                    let anchor = reentrant_storage
                        .get(&reentrant_anchor)
                        .map_err(|error| BlobDecodeError::new(error.to_string()))?;
                    assert_eq!(&*anchor, "anchor");
                    Ok(bytes.to_vec())
                }),
            })
            .expect("blob output seals");

        assert_eq!(
            *storage.get(&blob).expect("reentrant decoder succeeds"),
            vec![7, 7, 7, 7]
        );
    }

    #[test]
    fn below_threshold_blob_seal_checks_slot_state_before_decoder() {
        let storage = IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(100));
        let snapshot = snapshot(23);
        let pending = storage.allocate::<Vec<u8>>(allocation(snapshot, "unit"));
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let first_calls = calls.clone();
        storage
            .seal_blob(SealBlobOutputInput {
                slot: pending.clone(),
                lineage: lineage(snapshot, "unit", 41),
                side_tables: IrSideTables::default(),
                canonical_bytes: vec![1, 2, 3],
                decode: BlobDecoder::new(move |bytes| {
                    first_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(bytes.to_vec())
                }),
            })
            .expect("first below-threshold seal succeeds");

        let second_calls = calls.clone();
        let error = storage
            .seal_blob(SealBlobOutputInput {
                slot: pending,
                lineage: lineage(snapshot, "unit", 41),
                side_tables: IrSideTables::default(),
                canonical_bytes: vec![1, 2, 3],
                decode: BlobDecoder::new(move |bytes| {
                    second_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(bytes.to_vec())
                }),
            })
            .expect_err("double seal fails before decoder execution");

        assert!(matches!(error, StorageError::AlreadySealed { .. }));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn below_threshold_blob_seal_checks_unknown_slot_before_decoder() {
        let storage = IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(100));
        let foreign = IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(100));
        let snapshot = snapshot(24);
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let decoder_calls = calls.clone();
        let error = storage
            .seal_blob(SealBlobOutputInput {
                slot: foreign.allocate::<Vec<u8>>(allocation(snapshot, "foreign")),
                lineage: lineage(snapshot, "foreign", 42),
                side_tables: IrSideTables::default(),
                canonical_bytes: vec![1, 2, 3],
                decode: BlobDecoder::new(move |bytes| {
                    decoder_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(bytes.to_vec())
                }),
            })
            .expect_err("foreign pending slot fails before decoder execution");

        assert!(matches!(error, StorageError::UnknownSlot { .. }));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn below_threshold_blob_seal_checks_duplicate_output_before_decoder() {
        let storage = IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(100));
        let snapshot = snapshot(25);
        let same_lineage = lineage(snapshot, "same", 43);
        storage
            .seal(SealOutputInput {
                slot: storage.allocate::<String>(allocation(snapshot, "same")),
                lineage: same_lineage.clone(),
                side_tables: IrSideTables::default(),
                payload: "first".to_owned(),
            })
            .expect("first output seals");
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let decoder_calls = calls.clone();
        let error = storage
            .seal_blob(SealBlobOutputInput {
                slot: storage.allocate::<Vec<u8>>(allocation(snapshot, "same")),
                lineage: same_lineage,
                side_tables: IrSideTables::default(),
                canonical_bytes: vec![1, 2, 3],
                decode: BlobDecoder::new(move |bytes| {
                    decoder_calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(bytes.to_vec())
                }),
            })
            .expect_err("duplicate output fails before decoder execution");

        assert!(matches!(error, StorageError::OutputAlreadyStored { .. }));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn default_threshold_spills_only_payloads_over_threshold() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(15);
        let at_threshold = vec![1; DEFAULT_BLOB_SPILL_THRESHOLD];
        let over_threshold = vec![2; DEFAULT_BLOB_SPILL_THRESHOLD + 1];

        let resident = storage
            .seal_blob(SealBlobOutputInput {
                slot: storage.allocate::<Vec<u8>>(allocation(snapshot, "at-threshold")),
                lineage: lineage(snapshot, "at-threshold", 29),
                side_tables: IrSideTables::default(),
                canonical_bytes: at_threshold.clone(),
                decode: BlobDecoder::new(|bytes| Ok(bytes.to_vec())),
            })
            .expect("threshold-sized payload remains resident");
        let spilled = storage
            .seal_blob(SealBlobOutputInput {
                slot: storage.allocate::<Vec<u8>>(allocation(snapshot, "over-threshold")),
                lineage: lineage(snapshot, "over-threshold", 30),
                side_tables: IrSideTables::default(),
                canonical_bytes: over_threshold.clone(),
                decode: BlobDecoder::new(|bytes| Ok(bytes.to_vec())),
            })
            .expect("payload over threshold spills");

        assert!(matches!(resident.placement(), StoragePlacement::Resident));
        assert!(matches!(spilled.placement(), StoragePlacement::Blob { .. }));
        assert_eq!(
            *storage.get(&resident).expect("resident payload decodes"),
            at_threshold
        );
        assert_eq!(
            *storage.get(&spilled).expect("spilled payload decodes"),
            over_threshold
        );
    }

    #[test]
    fn collect_drops_only_unretained_and_unprotected_outputs() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(12);
        let retained = storage
            .seal(SealOutputInput {
                slot: storage.allocate::<String>(allocation(snapshot, "retained")),
                lineage: lineage(snapshot, "retained", 22),
                side_tables: IrSideTables::default(),
                payload: "retained".to_owned(),
            })
            .expect("retained output seals");
        let protected = storage
            .seal(SealOutputInput {
                slot: storage.allocate::<String>(allocation(snapshot, "protected")),
                lineage: lineage(snapshot, "protected", 23),
                side_tables: IrSideTables::default(),
                payload: "protected".to_owned(),
            })
            .expect("protected output seals");
        let dropped = storage
            .seal(SealOutputInput {
                slot: storage.allocate::<String>(allocation(snapshot, "dropped")),
                lineage: lineage(snapshot, "dropped", 24),
                side_tables: IrSideTables::default(),
                payload: "dropped".to_owned(),
            })
            .expect("dropped output seals");
        let owner = RetainOwner::new("lsp-snapshot");
        storage
            .retain(retained.output(), owner.clone())
            .expect("retain succeeds");

        let summary = storage.collect(CollectInput {
            snapshot,
            protected_outputs: vec![protected.output()],
        });

        assert_eq!(
            summary,
            CollectionSummary {
                outputs_dropped: 1,
                abandoned_slots_dropped: 0,
                blobs_dropped: 0,
                retained_outputs: 1,
                protected_outputs: 1,
            }
        );
        assert!(matches!(
            storage.get(&dropped),
            Err(StorageError::CollectedOutput { output }) if output == dropped.output()
        ));
        assert_eq!(
            &*storage.get(&retained).expect("retained output survives"),
            "retained"
        );
        assert_eq!(
            &*storage.get(&protected).expect("protected output survives"),
            "protected"
        );

        assert!(
            storage
                .release(retained.output(), &owner)
                .expect("release succeeds")
        );
        let cleanup = storage.collect(CollectInput {
            snapshot,
            protected_outputs: Vec::new(),
        });
        assert_eq!(cleanup.outputs_dropped, 2);
    }

    #[test]
    fn retained_old_snapshot_output_survives_until_release() {
        let storage = IrStorageService::new();
        let old_snapshot = snapshot(16);
        let new_snapshot = snapshot(17);
        let old = storage
            .seal(SealOutputInput {
                slot: storage.allocate::<String>(allocation(old_snapshot, "old")),
                lineage: lineage(old_snapshot, "old", 31),
                side_tables: IrSideTables::default(),
                payload: "old".to_owned(),
            })
            .expect("old snapshot output seals");
        let _new = storage
            .seal(SealOutputInput {
                slot: storage.allocate::<String>(allocation(new_snapshot, "new")),
                lineage: lineage(new_snapshot, "new", 32),
                side_tables: IrSideTables::default(),
                payload: "new".to_owned(),
            })
            .expect("new snapshot output seals");
        let owner = RetainOwner::new("stale-lsp-snapshot");
        storage
            .retain(old.output(), owner.clone())
            .expect("old output is retained");

        let retained_summary = storage.collect(CollectInput {
            snapshot: old_snapshot,
            protected_outputs: Vec::new(),
        });
        assert_eq!(retained_summary.retained_outputs, 1);
        assert_eq!(
            &*storage
                .get(&old)
                .expect("retained old output survives collection"),
            "old"
        );

        storage
            .release(old.output(), &owner)
            .expect("release succeeds");
        let released_summary = storage.collect(CollectInput {
            snapshot: old_snapshot,
            protected_outputs: Vec::new(),
        });
        assert_eq!(released_summary.outputs_dropped, 1);
        assert!(matches!(
            storage.get(&old),
            Err(StorageError::CollectedOutput { output }) if output == old.output()
        ));
    }

    #[test]
    fn collected_handle_does_not_resurrect_after_same_output_is_resealed() {
        let storage = IrStorageService::new();
        let snapshot = snapshot(22);
        let same_lineage = lineage(snapshot, "same-output", 40);
        let old = storage
            .seal(SealOutputInput {
                slot: storage.allocate::<String>(allocation(snapshot, "same-output")),
                lineage: same_lineage.clone(),
                side_tables: IrSideTables::default(),
                payload: "old".to_owned(),
            })
            .expect("old output seals");
        let summary = storage.collect(CollectInput {
            snapshot,
            protected_outputs: Vec::new(),
        });
        assert_eq!(summary.outputs_dropped, 1);
        assert!(matches!(
            storage.get(&old),
            Err(StorageError::CollectedOutput { output }) if output == old.output()
        ));

        let new = storage
            .seal(SealOutputInput {
                slot: storage.allocate::<String>(allocation(snapshot, "same-output")),
                lineage: same_lineage,
                side_tables: IrSideTables::default(),
                payload: "new".to_owned(),
            })
            .expect("same output id may be explicitly resealed as a new generation");

        assert_eq!(old.output(), new.output());
        assert_ne!(old.any().generation(), new.any().generation());
        assert!(matches!(
            storage.get(&old),
            Err(StorageError::CollectedOutput { output }) if output == old.output()
        ));
        assert_eq!(&*storage.get(&new).expect("new handle is readable"), "new");
    }

    #[test]
    fn shared_blob_survives_while_any_output_references_it() {
        let storage = IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(0));
        let snapshot = snapshot(18);
        let bytes = vec![4, 5, 6, 7];
        let first = storage
            .seal_blob(SealBlobOutputInput {
                slot: storage.allocate::<Vec<u8>>(allocation(snapshot, "first")),
                lineage: lineage(snapshot, "first", 33),
                side_tables: IrSideTables::default(),
                canonical_bytes: bytes.clone(),
                decode: BlobDecoder::new(|bytes| Ok(bytes.to_vec())),
            })
            .expect("first blob output seals");
        let second = storage
            .seal_blob(SealBlobOutputInput {
                slot: storage.allocate::<Vec<u8>>(allocation(snapshot, "second")),
                lineage: lineage(snapshot, "second", 34),
                side_tables: IrSideTables::default(),
                canonical_bytes: bytes.clone(),
                decode: BlobDecoder::new(|bytes| Ok(bytes.to_vec())),
            })
            .expect("second blob output seals");
        let owner = RetainOwner::new("cache-writer");
        storage
            .retain(second.output(), owner)
            .expect("second output is retained");

        let summary = storage.collect(CollectInput {
            snapshot,
            protected_outputs: Vec::new(),
        });

        assert_eq!(summary.outputs_dropped, 1);
        assert_eq!(summary.blobs_dropped, 0);
        assert!(matches!(
            storage.get(&first),
            Err(StorageError::CollectedOutput { output }) if output == first.output()
        ));
        assert_eq!(
            *storage.get(&second).expect("shared blob remains readable"),
            bytes
        );
    }

    #[test]
    fn blob_handles_and_collection_do_not_create_cache_or_trust_authority() {
        #[derive(Debug)]
        struct CacheProofPayload {
            cache_hit: bool,
            trusted: bool,
            proof_policy: &'static str,
        }

        let storage = IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(0));
        let snapshot = snapshot(19);
        let handle = storage
            .seal_blob(SealBlobOutputInput {
                slot: storage.allocate::<CacheProofPayload>(allocation(snapshot, "proofish")),
                lineage: lineage(snapshot, "proofish", 35),
                side_tables: IrSideTables::default(),
                canonical_bytes: b"cache-hit,trusted,strict".to_vec(),
                decode: BlobDecoder::new(|_| {
                    Ok(CacheProofPayload {
                        cache_hit: true,
                        trusted: true,
                        proof_policy: "strict",
                    })
                }),
            })
            .expect("proof/cache-shaped blob can be stored as opaque IR");
        let stored = storage.get(&handle).expect("blob payload decodes");
        assert!(stored.cache_hit);
        assert!(stored.trusted);
        assert_eq!(stored.proof_policy, "strict");

        let handle_debug = format!("{:?}", handle.erase());
        let placement_debug = format!("{:?}", handle.placement());
        let summary_debug = format!(
            "{:?}",
            storage.collect(CollectInput {
                snapshot,
                protected_outputs: vec![handle.output()],
            })
        );
        for forbidden in ["cache_hit", "trusted", "proof_policy", "strict"] {
            assert!(
                !handle_debug.contains(forbidden)
                    && !placement_debug.contains(forbidden)
                    && !summary_debug.contains(forbidden),
                "blob handle/placement/collection metadata must not expose `{forbidden}` authority"
            );
        }
    }

    #[test]
    fn collect_is_idempotent_and_drops_abandoned_slots() {
        let storage = IrStorageService::with_policy(StoragePolicy::with_blob_spill_threshold(0));
        let snapshot = snapshot(13);
        let blob = storage
            .seal_blob(SealBlobOutputInput {
                slot: storage.allocate::<Vec<u8>>(allocation(snapshot, "blob")),
                lineage: lineage(snapshot, "blob", 25),
                side_tables: IrSideTables::default(),
                canonical_bytes: vec![9, 9, 9],
                decode: BlobDecoder::new(|bytes| Ok(bytes.to_vec())),
            })
            .expect("blob output seals");
        let pending = storage.allocate::<String>(allocation(snapshot, "bad"));
        assert!(matches!(
            storage.seal(SealOutputInput {
                slot: pending,
                lineage: lineage(snapshot, "different", 26),
                side_tables: IrSideTables::default(),
                payload: "bad".to_owned(),
            }),
            Err(StorageError::MetadataMismatch { .. })
        ));

        let first = storage.collect(CollectInput {
            snapshot,
            protected_outputs: Vec::new(),
        });
        let second = storage.collect(CollectInput {
            snapshot,
            protected_outputs: Vec::new(),
        });

        assert_eq!(first.outputs_dropped, 1);
        assert_eq!(first.abandoned_slots_dropped, 1);
        assert_eq!(first.blobs_dropped, 1);
        assert_eq!(second, CollectionSummary::default());
        assert!(matches!(
            storage.get(&blob),
            Err(StorageError::CollectedOutput { output }) if output == blob.output()
        ));
    }
}
