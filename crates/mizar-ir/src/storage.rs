//! Immutable phase-output storage and typed handles.
//!
//! This module is specified in
//! [`storage.md`](../../../../doc/design/mizar-ir/en/storage.md).

use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    error::Error,
    fmt,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use mizar_session::{BuildSnapshotId, Hash};

use crate::identity::{
    IdentityError, OutputKind, PhaseOutputId, PhaseOutputLineage, PipelinePhase, WorkUnit,
};

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
    state: Mutex<StorageState>,
}

#[derive(Debug, Default)]
struct StorageState {
    next_slot_id: u64,
    slots: HashMap<OutputSlotId, SlotRecord>,
    outputs: HashMap<PhaseOutputId, OutputSlotId>,
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
    payload: Arc<dyn Any + Send + Sync>,
    side_tables: Arc<IrSideTables>,
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
        let mut state = self.state.lock().expect("IR storage mutex poisoned");
        let slot_id = input.slot.id;
        let output = input.lineage.output;

        let existing_slot = state.outputs.get(&output).copied();
        let record = state
            .slots
            .get_mut(&slot_id)
            .ok_or(StorageError::UnknownSlot { slot: slot_id })?;
        match record.state {
            SlotState::Pending => {}
            SlotState::Abandoned => return Err(StorageError::AbandonedOutput { slot: slot_id }),
            SlotState::Sealed(_) => return Err(StorageError::AlreadySealed { slot: slot_id }),
        }

        if let Some(existing_slot) = existing_slot
            && existing_slot != slot_id
        {
            record.state = SlotState::Abandoned;
            return Err(StorageError::OutputAlreadyStored { output });
        }

        if let Err(error) = reject_stale_pending(&input.slot, record, output) {
            record.state = SlotState::Abandoned;
            return Err(error);
        }
        if let Err(error) = reject_lineage_mismatch(&input.slot, &input.lineage) {
            record.state = SlotState::Abandoned;
            return Err(error);
        }
        if let Err(error) = input.lineage.validate_identity() {
            record.state = SlotState::Abandoned;
            return Err(StorageError::InvalidLineage {
                output,
                error: Box::new(error),
            });
        }

        let handle = AnyPhaseOutputRef {
            lineage: input.lineage,
            schema_version: input.slot.schema_version,
            generation: input.slot.generation,
        };
        let typed = PhaseOutputRef {
            inner: handle.clone(),
            marker: PhantomData,
        };
        record.state = SlotState::Sealed(Box::new(SealedSlot {
            handle,
            payload: Arc::new(input.payload),
            side_tables: Arc::new(input.side_tables),
        }));
        state.outputs.insert(output, slot_id);

        Ok(typed)
    }

    /// Reads a sealed payload through a typed handle.
    pub fn get<T>(&self, handle: &PhaseOutputRef<T>) -> Result<Arc<T>, StorageError>
    where
        T: Send + Sync + 'static,
    {
        let payload = {
            let state = self.state.lock().expect("IR storage mutex poisoned");
            let record = sealed_record_for_handle(&state, handle.any())?;
            reject_rust_type::<T>(record)?;
            let SlotState::Sealed(sealed) = &record.state else {
                return Err(StorageError::UnsealedOutput {
                    slot: slot_for_handle(&state, handle.any())?,
                });
            };
            sealed.payload.clone()
        };

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
        return Err(StorageError::StaleHandle {
            output: handle.output(),
        });
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
        return Err(StorageError::StaleHandle {
            output: handle.output(),
        });
    }
    Ok(record)
}

fn slot_for_handle(
    state: &StorageState,
    handle: &AnyPhaseOutputRef,
) -> Result<OutputSlotId, StorageError> {
    state
        .outputs
        .get(&handle.output())
        .copied()
        .ok_or(StorageError::UnknownOutput {
            output: handle.output(),
        })
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
}
