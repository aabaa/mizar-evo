//! Phase-output publisher for sealed IR handles.
//!
//! This module is specified in
//! [`publisher.md`](../../../../doc/design/mizar-ir/en/publisher.md).

use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    sync::{Arc, Mutex},
};

use mizar_session::{BuildSnapshotId, Hash};

use crate::{
    identity::{
        IdentityError, NamedInputHash, OutputIdentityInput, OutputKind, PhaseOutputId,
        PhaseOutputRegistrationStatus, PipelinePhase, SnapshotHandleRegistry, WorkUnit,
    },
    storage::{
        AnyPhaseOutputRef, BlobDecoder, IrSideTables, IrStorageService, PendingOutputSlot,
        PhaseOutputRef, SchemaVersion, SealCanonicalOutputInput, SideTableRecord, StorageError,
        StorageGeneration,
    },
};

const CONTENT_HASH_DOMAIN: &str = "mizar-ir/publisher-content-hash/v1";
const SIDE_TABLE_HASH_DOMAIN: &str = "mizar-ir/publisher-side-table-hash/v1";
const NAMED_INPUT_HASH_DOMAIN: &str = "mizar-ir/publisher-named-input-hash/v1";

/// Hash summary for a sealed parent output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub(crate) struct ParentHashSummary {
    /// Parent content hash.
    pub(crate) content_hash: Hash,
    /// Parent side-table hash.
    pub(crate) side_table_hash: Hash,
}

/// Origin of a publish request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
#[non_exhaustive]
pub enum OutputOrigin {
    /// Output comes from a package/current source build.
    PackageSource,
    /// Output comes from a retained stale snapshot and is internal-only.
    RetainedStaleSnapshot,
    /// Output has already been validated and rehydrated into the current
    /// snapshot by a later cache adapter.
    ValidatedCacheInput,
    /// Output comes from an editor/open-buffer-only source.
    OpenBuffer,
}

/// Publication target requested by a producer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
#[non_exhaustive]
pub enum PublicationTarget {
    /// Current/package-visible build result.
    CurrentPackage,
    /// Internal retained output only.
    InternalOnly,
}

/// Explicit crate-local allowed work-unit context.
#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
pub struct AllowedWorkUnit {
    /// Producing phase.
    pub phase: PipelinePhase,
    /// Runtime output kind.
    pub output_kind: OutputKind,
    /// Work unit allowed for that phase/kind.
    pub work_unit: WorkUnit,
}

/// Input for publishing a complete output.
pub struct PublishOutputInput<T> {
    /// Pending storage slot.
    pub slot: PendingOutputSlot<T>,
    /// Target snapshot.
    pub snapshot: BuildSnapshotId,
    /// Producing phase.
    pub phase: PipelinePhase,
    /// Producing work unit.
    pub work_unit: WorkUnit,
    /// Runtime output kind.
    pub output_kind: OutputKind,
    /// Payload schema version.
    pub schema_version: SchemaVersion,
    /// Complete payload.
    pub payload: T,
    /// Canonical payload bytes used for hashing and storage placement.
    pub canonical_payload: Option<Vec<u8>>,
    /// Decoder for canonical payload bytes.
    pub decode: BlobDecoder<T>,
    /// Parent sealed outputs.
    pub parents: Vec<AnyPhaseOutputRef>,
    /// Producer-declared non-output input hashes.
    pub named_input_hashes: Vec<NamedInputHash>,
    /// Side tables to attach.
    pub side_tables: IrSideTables,
    /// Origin classification.
    pub origin: OutputOrigin,
    /// Requested publication target.
    pub target: PublicationTarget,
}

/// Narrow publisher API used by phase services.
#[derive(Debug)]
pub struct PhaseOutputPublisher {
    storage: Arc<IrStorageService>,
    registry: Arc<SnapshotHandleRegistry>,
    state: Mutex<PublisherState>,
}

#[derive(Debug, Default)]
struct PublisherState {
    current_snapshots: HashSet<BuildSnapshotId>,
    obsolete_snapshots: HashSet<BuildSnapshotId>,
    superseded_by: HashMap<BuildSnapshotId, BuildSnapshotId>,
    current_outputs: HashSet<CurrentOutputRoot>,
    allowed_work_units: HashSet<AllowedWorkUnit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
struct CurrentOutputRoot {
    output: PhaseOutputId,
    generation: StorageGeneration,
}

/// Errors reported by the phase-output publisher.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PublishError {
    /// Snapshot was not registered as current/known to this publisher.
    UnknownSnapshot {
        /// Unknown snapshot.
        snapshot: BuildSnapshotId,
    },
    /// Snapshot is obsolete for current publication.
    ObsoleteSnapshot {
        /// Obsolete snapshot.
        snapshot: BuildSnapshotId,
    },
    /// Snapshot replacement tried to replace a snapshot with itself.
    InvalidSnapshotReplacement {
        /// Snapshot supplied as both old and new.
        snapshot: BuildSnapshotId,
    },
    /// Snapshot has already been superseded and cannot become current again.
    SupersededSnapshot {
        /// Superseded snapshot.
        snapshot: BuildSnapshotId,
        /// Replacement snapshot.
        replacement: BuildSnapshotId,
    },
    /// Open-buffer output was requested as current/package output.
    OpenBufferOutput {
        /// Snapshot.
        snapshot: BuildSnapshotId,
    },
    /// Retained stale snapshot output was requested as current/package output.
    RetainedStaleSnapshotOutput {
        /// Snapshot.
        snapshot: BuildSnapshotId,
    },
    /// Work unit is not allowed by explicit crate-local context.
    WorkUnitNotAllowed {
        /// Producing phase.
        phase: PipelinePhase,
        /// Work unit.
        work_unit: WorkUnit,
        /// Output kind.
        output_kind: OutputKind,
    },
    /// Parent handle belongs to a different snapshot.
    ParentSnapshotMismatch {
        /// Publish snapshot.
        snapshot: BuildSnapshotId,
        /// Parent output id.
        parent: PhaseOutputId,
        /// Parent snapshot.
        parent_snapshot: BuildSnapshotId,
    },
    /// Output handle belongs to a different snapshot.
    OutputSnapshotMismatch {
        /// Publish snapshot.
        snapshot: BuildSnapshotId,
        /// Output id.
        output: PhaseOutputId,
        /// Output snapshot.
        output_snapshot: BuildSnapshotId,
    },
    /// Output is sealed but was not published as a current/package result.
    OutputNotCurrentPackage {
        /// Current snapshot.
        snapshot: BuildSnapshotId,
        /// Output id.
        output: PhaseOutputId,
    },
    /// Canonical payload bytes were missing.
    MissingCanonicalPayload,
    /// Invalid side-table record.
    InvalidSideTable {
        /// Field path.
        field: &'static str,
    },
    /// Identity registration failed.
    Identity {
        /// Identity error.
        error: Box<IdentityError>,
    },
    /// Storage sealing failed.
    Storage {
        /// Storage error.
        error: Box<StorageError>,
    },
}

impl PhaseOutputPublisher {
    /// Creates a publisher over shared storage and identity registry.
    pub fn new(storage: Arc<IrStorageService>, registry: Arc<SnapshotHandleRegistry>) -> Self {
        Self {
            storage,
            registry,
            state: Mutex::default(),
        }
    }

    /// Returns the shared storage service.
    pub fn storage(&self) -> &Arc<IrStorageService> {
        &self.storage
    }

    /// Returns the shared identity registry.
    pub fn registry(&self) -> &Arc<SnapshotHandleRegistry> {
        &self.registry
    }

    /// Registers a snapshot as current/known for publication.
    pub fn register_current_snapshot(&self, snapshot: BuildSnapshotId) {
        let mut state = self.state.lock().expect("publisher mutex poisoned");
        self.registry.register_snapshot(snapshot);
        state.current_snapshots.insert(snapshot);
        if !state.superseded_by.contains_key(&snapshot) {
            state.obsolete_snapshots.remove(&snapshot);
        }
    }

    /// Marks a snapshot obsolete for current publication.
    pub fn mark_obsolete(&self, snapshot: BuildSnapshotId) -> Result<(), PublishError> {
        let mut state = self.state.lock().expect("publisher mutex poisoned");
        if !state.current_snapshots.contains(&snapshot) {
            return Err(PublishError::UnknownSnapshot { snapshot });
        }
        state.obsolete_snapshots.insert(snapshot);
        Ok(())
    }

    /// Atomically replaces one current snapshot with another.
    ///
    /// The old snapshot remains known so retained handles can be read and cache
    /// encoded while storage keeps them alive. It is also permanently marked
    /// obsolete for current/package publication unless a future owner creates a
    /// distinct replacement snapshot.
    pub fn replace_current_snapshot(
        &self,
        old_snapshot: BuildSnapshotId,
        new_snapshot: BuildSnapshotId,
    ) -> Result<(), PublishError> {
        if old_snapshot == new_snapshot {
            return Err(PublishError::InvalidSnapshotReplacement {
                snapshot: old_snapshot,
            });
        }

        let mut state = self.state.lock().expect("publisher mutex poisoned");
        validate_snapshot_state(
            &state,
            old_snapshot,
            OutputOrigin::PackageSource,
            PublicationTarget::CurrentPackage,
        )?;
        if let Some(replacement) = state.superseded_by.get(&new_snapshot).copied() {
            return Err(PublishError::SupersededSnapshot {
                snapshot: new_snapshot,
                replacement,
            });
        }

        self.registry.register_snapshot(new_snapshot);
        state.current_snapshots.insert(new_snapshot);
        state.obsolete_snapshots.remove(&new_snapshot);
        state.obsolete_snapshots.insert(old_snapshot);
        state.superseded_by.insert(old_snapshot, new_snapshot);
        Ok(())
    }

    /// Validates that a snapshot is eligible for current/package projection.
    pub fn validate_current_snapshot(&self, snapshot: BuildSnapshotId) -> Result<(), PublishError> {
        let state = self.state.lock().expect("publisher mutex poisoned");
        validate_snapshot_state(
            &state,
            snapshot,
            OutputOrigin::PackageSource,
            PublicationTarget::CurrentPackage,
        )
    }

    /// Validates that a sealed output was published as a current/package result.
    pub fn validate_current_output(
        &self,
        snapshot: BuildSnapshotId,
        output: &AnyPhaseOutputRef,
    ) -> Result<(), PublishError> {
        let state = self.state.lock().expect("publisher mutex poisoned");
        validate_snapshot_state(
            &state,
            snapshot,
            OutputOrigin::PackageSource,
            PublicationTarget::CurrentPackage,
        )?;
        if output.snapshot() != snapshot {
            return Err(PublishError::OutputSnapshotMismatch {
                snapshot,
                output: output.output(),
                output_snapshot: output.snapshot(),
            });
        }
        let root = CurrentOutputRoot {
            output: output.output(),
            generation: output.generation(),
        };
        if !state.current_outputs.contains(&root) {
            return Err(PublishError::OutputNotCurrentPackage {
                snapshot,
                output: output.output(),
            });
        }
        Ok(())
    }

    /// Registers an explicit allowed work-unit context.
    pub fn allow_work_unit(&self, allowed: AllowedWorkUnit) {
        self.state
            .lock()
            .expect("publisher mutex poisoned")
            .allowed_work_units
            .insert(allowed);
    }

    /// Allocates a storage slot for a future published output.
    pub fn allocate<T>(
        &self,
        snapshot: BuildSnapshotId,
        phase: PipelinePhase,
        work_unit: WorkUnit,
        output_kind: OutputKind,
        schema_version: SchemaVersion,
    ) -> PendingOutputSlot<T>
    where
        T: Send + Sync + 'static,
    {
        self.storage.allocate(crate::storage::AllocateSlotInput {
            snapshot,
            phase,
            work_unit,
            output_kind,
            schema_version,
        })
    }

    /// Publishes a complete output and returns a sealed typed handle.
    pub fn publish<T>(
        &self,
        input: PublishOutputInput<T>,
    ) -> Result<PhaseOutputRef<T>, PublishError>
    where
        T: Send + Sync + 'static,
    {
        let slot_for_abandon = input.slot.clone();
        let mut state = self.state.lock().expect("publisher mutex poisoned");
        if let Err(error) =
            validate_snapshot_state(&state, input.snapshot, input.origin, input.target)
        {
            self.storage.abandon(slot_for_abandon);
            return Err(error);
        }
        if let Err(error) =
            validate_allowed_work_unit(&state, &input.phase, &input.work_unit, &input.output_kind)
        {
            self.storage.abandon(slot_for_abandon);
            return Err(error);
        }
        if let Err(error) = validate_slot_metadata(
            &input.slot,
            input.snapshot,
            &input.phase,
            &input.work_unit,
            &input.output_kind,
            input.schema_version,
        ) {
            self.storage.abandon(slot_for_abandon);
            return Err(error);
        }
        if let Err(error) = validate_side_tables(&input.side_tables) {
            self.storage.abandon(slot_for_abandon);
            return Err(error);
        }

        let Some(canonical_payload) = input.canonical_payload.clone() else {
            self.storage.abandon(slot_for_abandon);
            return Err(PublishError::MissingCanonicalPayload);
        };
        let mut parents = input.parents.clone();
        if let Err(error) = self.validate_parent_handles(input.snapshot, &parents) {
            self.storage.abandon(slot_for_abandon);
            return Err(error);
        }
        sort_parent_refs(&mut parents);
        let content_hash = match content_hash(
            &canonical_payload,
            &parents,
            input.named_input_hashes.clone(),
        ) {
            Ok(hash) => hash,
            Err(error) => {
                self.storage.abandon(slot_for_abandon);
                return Err(error);
            }
        };
        let side_table_hash = match side_table_hash(&input.side_tables) {
            Ok(hash) => hash,
            Err(error) => {
                self.storage.abandon(slot_for_abandon);
                return Err(error);
            }
        };
        let parent_ids = parents.iter().map(AnyPhaseOutputRef::output).collect();

        let registration = match self
            .registry
            .register_output_with_status(OutputIdentityInput {
                snapshot: input.snapshot,
                phase: input.phase,
                work_unit: input.work_unit,
                output_kind: input.output_kind,
                content_hash,
                side_table_hash,
                parents: parent_ids,
                named_input_hashes: input.named_input_hashes,
            }) {
            Ok(registration) => registration,
            Err(error) => {
                self.storage.abandon(slot_for_abandon);
                return Err(PublishError::Identity {
                    error: Box::new(error),
                });
            }
        };
        let lineage = registration.lineage.clone();
        let output = lineage.output;
        let publishes_current = input.target == PublicationTarget::CurrentPackage;

        match self.storage.seal_canonical(SealCanonicalOutputInput {
            slot: input.slot,
            lineage: lineage.clone(),
            side_tables: input.side_tables,
            payload: input.payload,
            canonical_bytes: canonical_payload,
            decode: input.decode,
        }) {
            Ok(handle) => {
                if publishes_current {
                    state.current_outputs.insert(CurrentOutputRoot {
                        output,
                        generation: handle.any().generation(),
                    });
                }
                Ok(handle)
            }
            Err(error) => {
                if registration.status == PhaseOutputRegistrationStatus::Inserted {
                    self.registry.rollback_inserted_output(&lineage);
                }
                Err(PublishError::Storage {
                    error: Box::new(error),
                })
            }
        }
    }

    fn validate_parent_handles(
        &self,
        snapshot: BuildSnapshotId,
        parents: &[AnyPhaseOutputRef],
    ) -> Result<(), PublishError> {
        for parent in parents {
            if parent.snapshot() != snapshot {
                return Err(PublishError::ParentSnapshotMismatch {
                    snapshot,
                    parent: parent.output(),
                    parent_snapshot: parent.snapshot(),
                });
            }
            self.storage
                .validate_handle(parent)
                .map_err(|error| PublishError::Storage {
                    error: Box::new(error),
                })?;
        }
        Ok(())
    }
}

impl AllowedWorkUnit {
    /// Creates an allowed work-unit record.
    pub fn new(phase: PipelinePhase, output_kind: OutputKind, work_unit: WorkUnit) -> Self {
        Self {
            phase,
            output_kind,
            work_unit,
        }
    }
}

impl From<&AnyPhaseOutputRef> for ParentHashSummary {
    fn from(value: &AnyPhaseOutputRef) -> Self {
        Self {
            content_hash: value.content_hash(),
            side_table_hash: value.side_table_hash(),
        }
    }
}

impl fmt::Display for PublishError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSnapshot { snapshot } => {
                write!(formatter, "unknown publisher snapshot `{snapshot:?}`")
            }
            Self::ObsoleteSnapshot { snapshot } => {
                write!(
                    formatter,
                    "snapshot `{snapshot:?}` is obsolete for current publication"
                )
            }
            Self::InvalidSnapshotReplacement { snapshot } => {
                write!(formatter, "snapshot `{snapshot:?}` cannot replace itself")
            }
            Self::SupersededSnapshot {
                snapshot,
                replacement,
            } => {
                write!(
                    formatter,
                    "snapshot `{snapshot:?}` was already superseded by `{replacement:?}`"
                )
            }
            Self::OpenBufferOutput { snapshot } => {
                write!(
                    formatter,
                    "open-buffer output for `{snapshot:?}` cannot publish as current/package output"
                )
            }
            Self::RetainedStaleSnapshotOutput { snapshot } => {
                write!(
                    formatter,
                    "retained stale output for `{snapshot:?}` cannot publish as current/package output"
                )
            }
            Self::WorkUnitNotAllowed {
                phase,
                work_unit,
                output_kind,
            } => {
                write!(
                    formatter,
                    "work unit `{}` is not allowed for phase `{}` and output kind `{}`",
                    work_unit.as_str(),
                    phase.as_str(),
                    output_kind.as_str()
                )
            }
            Self::ParentSnapshotMismatch {
                snapshot,
                parent,
                parent_snapshot,
            } => {
                write!(
                    formatter,
                    "parent `{parent:?}` from `{parent_snapshot:?}` cannot be used for `{snapshot:?}`"
                )
            }
            Self::OutputSnapshotMismatch {
                snapshot,
                output,
                output_snapshot,
            } => {
                write!(
                    formatter,
                    "output `{output:?}` from `{output_snapshot:?}` cannot be used for `{snapshot:?}`"
                )
            }
            Self::OutputNotCurrentPackage { snapshot, output } => {
                write!(
                    formatter,
                    "output `{output:?}` was not published as current/package output for `{snapshot:?}`"
                )
            }
            Self::MissingCanonicalPayload => formatter.write_str("missing canonical payload bytes"),
            Self::InvalidSideTable { field } => {
                write!(formatter, "invalid publisher side-table field `{field}`")
            }
            Self::Identity { error } => write!(formatter, "publisher identity error: {error}"),
            Self::Storage { error } => write!(formatter, "publisher storage error: {error}"),
        }
    }
}

impl Error for PublishError {}

fn validate_snapshot_state(
    state: &PublisherState,
    snapshot: BuildSnapshotId,
    origin: OutputOrigin,
    target: PublicationTarget,
) -> Result<(), PublishError> {
    if !state.current_snapshots.contains(&snapshot) {
        return Err(PublishError::UnknownSnapshot { snapshot });
    }
    if target == PublicationTarget::CurrentPackage {
        if state.obsolete_snapshots.contains(&snapshot) {
            return Err(PublishError::ObsoleteSnapshot { snapshot });
        }
        match origin {
            OutputOrigin::OpenBuffer => return Err(PublishError::OpenBufferOutput { snapshot }),
            OutputOrigin::RetainedStaleSnapshot => {
                return Err(PublishError::RetainedStaleSnapshotOutput { snapshot });
            }
            OutputOrigin::PackageSource | OutputOrigin::ValidatedCacheInput => {}
        }
    }
    Ok(())
}

fn validate_allowed_work_unit(
    state: &PublisherState,
    phase: &PipelinePhase,
    work_unit: &WorkUnit,
    output_kind: &OutputKind,
) -> Result<(), PublishError> {
    let allowed = AllowedWorkUnit {
        phase: phase.clone(),
        output_kind: output_kind.clone(),
        work_unit: work_unit.clone(),
    };
    if state.allowed_work_units.contains(&allowed) {
        Ok(())
    } else {
        Err(PublishError::WorkUnitNotAllowed {
            phase: phase.clone(),
            work_unit: work_unit.clone(),
            output_kind: output_kind.clone(),
        })
    }
}

fn validate_side_tables(side_tables: &IrSideTables) -> Result<(), PublishError> {
    for (field, values) in [
        ("source_maps", &side_tables.source_maps),
        ("diagnostics", &side_tables.diagnostics),
        ("explanation_refs", &side_tables.explanation_refs),
        (
            "documentation_attachments",
            &side_tables.documentation_attachments,
        ),
    ] {
        for value in values {
            if value.domain.trim().is_empty() || value.key.trim().is_empty() {
                return Err(PublishError::InvalidSideTable { field });
            }
        }
    }
    Ok(())
}

fn validate_slot_metadata<T>(
    slot: &PendingOutputSlot<T>,
    snapshot: BuildSnapshotId,
    phase: &PipelinePhase,
    work_unit: &WorkUnit,
    output_kind: &OutputKind,
    schema_version: SchemaVersion,
) -> Result<(), PublishError> {
    let mismatch = slot.snapshot() != snapshot
        || slot.phase() != phase
        || slot.work_unit() != work_unit
        || slot.output_kind() != output_kind
        || slot.schema_version() != schema_version;
    if mismatch {
        Err(PublishError::Storage {
            error: Box::new(StorageError::MetadataMismatch {
                field: "publisher_slot",
            }),
        })
    } else {
        Ok(())
    }
}

fn sort_parent_refs(parents: &mut [AnyPhaseOutputRef]) {
    parents.sort_by(|left, right| {
        left.content_hash()
            .as_bytes()
            .cmp(right.content_hash().as_bytes())
            .then_with(|| {
                left.side_table_hash()
                    .as_bytes()
                    .cmp(right.side_table_hash().as_bytes())
            })
    });
}

fn content_hash(
    canonical_payload: &[u8],
    parents: &[AnyPhaseOutputRef],
    named_input_hashes: Vec<NamedInputHash>,
) -> Result<Hash, PublishError> {
    let parents = parents
        .iter()
        .map(ParentHashSummary::from)
        .collect::<Vec<_>>();
    content_hash_from_parent_summaries(canonical_payload, &parents, named_input_hashes)
}

pub(crate) fn content_hash_from_parent_summaries(
    canonical_payload: &[u8],
    parents: &[ParentHashSummary],
    named_input_hashes: Vec<NamedInputHash>,
) -> Result<Hash, PublishError> {
    let mut named_input_hashes = canonicalize_named_inputs(named_input_hashes)?;
    let mut parents = parents.to_vec();
    sort_parent_summaries(&mut parents);
    let mut hasher = stable_hasher(CONTENT_HASH_DOMAIN);
    write_bytes(&mut hasher, canonical_payload);
    write_len(&mut hasher, parents.len());
    for parent in parents {
        write_hash(&mut hasher, "parent_content", parent.content_hash);
    }
    write_len(&mut hasher, named_input_hashes.len());
    for named in named_input_hashes.drain(..) {
        write_str(&mut hasher, &named.name);
        write_str(&mut hasher, &named.domain);
        write_hash(&mut hasher, NAMED_INPUT_HASH_DOMAIN, named.digest);
    }
    Ok(finish_hash(hasher))
}

pub(crate) fn side_table_hash(side_tables: &IrSideTables) -> Result<Hash, PublishError> {
    validate_side_tables(side_tables)?;
    let mut hasher = stable_hasher(SIDE_TABLE_HASH_DOMAIN);
    for (category, values) in [
        ("source_maps", &side_tables.source_maps),
        ("diagnostics", &side_tables.diagnostics),
        ("explanation_refs", &side_tables.explanation_refs),
        (
            "documentation_attachments",
            &side_tables.documentation_attachments,
        ),
    ] {
        let mut values = values.clone();
        values.sort_by(|left, right| side_table_key(left).cmp(&side_table_key(right)));
        write_str(&mut hasher, category);
        write_len(&mut hasher, values.len());
        for value in values {
            write_str(&mut hasher, &value.domain);
            write_str(&mut hasher, &value.key);
            write_hash(&mut hasher, "side_table_digest", value.digest);
        }
    }
    Ok(finish_hash(hasher))
}

pub(crate) fn canonical_parent_summaries(parents: &[AnyPhaseOutputRef]) -> Vec<ParentHashSummary> {
    let mut summaries = parents
        .iter()
        .map(ParentHashSummary::from)
        .collect::<Vec<_>>();
    sort_parent_summaries(&mut summaries);
    summaries
}

fn sort_parent_summaries(parents: &mut [ParentHashSummary]) {
    parents.sort_by(|left, right| {
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

fn canonicalize_named_inputs(
    mut values: Vec<NamedInputHash>,
) -> Result<Vec<NamedInputHash>, PublishError> {
    values.sort_by(|left, right| {
        named_input_key(left)
            .cmp(&named_input_key(right))
            .then_with(|| left.digest.as_bytes().cmp(right.digest.as_bytes()))
    });

    let mut result: Vec<NamedInputHash> = Vec::new();
    for value in values {
        if value.name.trim().is_empty() || value.domain.trim().is_empty() {
            return Err(PublishError::InvalidSideTable {
                field: "named_input_hashes",
            });
        }
        if let Some(existing) = result
            .iter()
            .find(|existing| named_input_key(existing) == named_input_key(&value))
        {
            if existing.digest != value.digest {
                return Err(PublishError::InvalidSideTable {
                    field: "named_input_hashes",
                });
            }
            continue;
        }
        result.push(value);
    }
    Ok(result)
}

fn named_input_key(value: &NamedInputHash) -> (&str, &str) {
    (&value.name, &value.domain)
}

fn side_table_key(value: &SideTableRecord) -> (&str, &str, &[u8; Hash::BYTE_LEN]) {
    (&value.domain, &value.key, value.digest.as_bytes())
}

fn stable_hasher(domain: &str) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    write_str(&mut hasher, domain);
    hasher
}

fn write_hash(hasher: &mut blake3::Hasher, domain: &str, value: Hash) {
    write_str(hasher, domain);
    hasher.update(value.as_bytes());
}

fn write_bytes(hasher: &mut blake3::Hasher, value: &[u8]) {
    write_len(hasher, value.len());
    hasher.update(value);
}

fn write_str(hasher: &mut blake3::Hasher, value: &str) {
    write_bytes(hasher, value.as_bytes());
}

fn write_len(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

fn finish_hash(hasher: blake3::Hasher) -> Hash {
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn output_kind() -> OutputKind {
        OutputKind::new("ResolvedAst")
    }

    fn work_unit(value: &str) -> WorkUnit {
        WorkUnit::new(value)
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

    fn publisher(snapshot: BuildSnapshotId) -> PhaseOutputPublisher {
        let publisher = PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        );
        publisher.register_current_snapshot(snapshot);
        publisher.allow_work_unit(AllowedWorkUnit::new(
            phase(),
            output_kind(),
            work_unit("unit"),
        ));
        publisher.allow_work_unit(AllowedWorkUnit::new(
            phase(),
            output_kind(),
            work_unit("parent"),
        ));
        publisher.allow_work_unit(AllowedWorkUnit::new(
            phase(),
            output_kind(),
            work_unit("child"),
        ));
        publisher
    }

    fn publish_text(
        publisher: &PhaseOutputPublisher,
        snapshot: BuildSnapshotId,
        unit: &str,
        payload: &str,
        parents: Vec<AnyPhaseOutputRef>,
        side_tables: IrSideTables,
        mode: (OutputOrigin, PublicationTarget),
    ) -> Result<PhaseOutputRef<String>, PublishError> {
        publisher.publish(PublishOutputInput {
            slot: publisher.allocate(
                snapshot,
                phase(),
                work_unit(unit),
                output_kind(),
                SchemaVersion::new(1),
            ),
            snapshot,
            phase: phase(),
            work_unit: work_unit(unit),
            output_kind: output_kind(),
            schema_version: SchemaVersion::new(1),
            payload: payload.to_owned(),
            canonical_payload: Some(payload.as_bytes().to_vec()),
            decode: BlobDecoder::new(|bytes| {
                String::from_utf8(bytes.to_vec())
                    .map_err(|error| crate::storage::BlobDecodeError::new(error.to_string()))
            }),
            parents,
            named_input_hashes: vec![named(1)],
            side_tables,
            origin: mode.0,
            target: mode.1,
        })
    }

    #[test]
    fn hashes_are_deterministic_for_identical_inputs() {
        let snapshot = snapshot(1);
        let left = publisher(snapshot);
        let right = publisher(snapshot);

        let left_handle = publish_text(
            &left,
            snapshot,
            "unit",
            "payload",
            Vec::new(),
            side_tables(10),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("left publish succeeds");
        let right_handle = publish_text(
            &right,
            snapshot,
            "unit",
            "payload",
            Vec::new(),
            side_tables(10),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("right publish succeeds");

        assert_eq!(left_handle.content_hash(), right_handle.content_hash());
        assert_eq!(
            left_handle.side_table_hash(),
            right_handle.side_table_hash()
        );
        assert_eq!(left_handle.output(), right_handle.output());
    }

    #[test]
    fn parent_and_named_inputs_are_canonicalized_from_sealed_handles() {
        let snapshot = snapshot(2);
        let left = publisher(snapshot);
        let right = publisher(snapshot);

        let left_parent = publish_text(
            &left,
            snapshot,
            "parent",
            "parent",
            Vec::new(),
            side_tables(20),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("left parent publishes");
        let right_parent = publish_text(
            &right,
            snapshot,
            "parent",
            "parent",
            Vec::new(),
            side_tables(20),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("right parent publishes");
        let left_child = publish_text(
            &left,
            snapshot,
            "child",
            "child",
            vec![left_parent.erase()],
            side_tables(21),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("left child publishes");
        let right_child = publish_text(
            &right,
            snapshot,
            "child",
            "child",
            vec![right_parent.erase()],
            side_tables(21),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("right child publishes");

        assert_eq!(left_child.content_hash(), right_child.content_hash());
        assert_eq!(left_child.lineage().parents, right_child.lineage().parents);
    }

    #[test]
    fn multi_parent_inputs_are_canonicalized_from_sealed_handles() {
        let snapshot = snapshot(12);
        let left = publisher(snapshot);
        let right = publisher(snapshot);
        for publisher in [&left, &right] {
            publisher.allow_work_unit(AllowedWorkUnit::new(
                phase(),
                output_kind(),
                work_unit("parent-a"),
            ));
            publisher.allow_work_unit(AllowedWorkUnit::new(
                phase(),
                output_kind(),
                work_unit("parent-b"),
            ));
        }

        let left_a = publish_text(
            &left,
            snapshot,
            "parent-a",
            "parent-a",
            Vec::new(),
            side_tables(52),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("left parent a publishes");
        let left_b = publish_text(
            &left,
            snapshot,
            "parent-b",
            "parent-b",
            Vec::new(),
            side_tables(53),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("left parent b publishes");
        let right_a = publish_text(
            &right,
            snapshot,
            "parent-a",
            "parent-a",
            Vec::new(),
            side_tables(52),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("right parent a publishes");
        let right_b = publish_text(
            &right,
            snapshot,
            "parent-b",
            "parent-b",
            Vec::new(),
            side_tables(53),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("right parent b publishes");

        let left_child = publish_text(
            &left,
            snapshot,
            "child",
            "child",
            vec![left_a.erase(), left_b.erase()],
            side_tables(54),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("left child publishes");
        let right_child = publish_text(
            &right,
            snapshot,
            "child",
            "child",
            vec![right_b.erase(), right_a.erase()],
            side_tables(54),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("right child publishes");

        assert_eq!(left_child.content_hash(), right_child.content_hash());
        assert_eq!(left_child.lineage().parents, right_child.lineage().parents);
    }

    #[test]
    fn parent_output_id_does_not_enter_child_content_hash() {
        let snapshot = snapshot(10);
        let left = publisher(snapshot);
        let right = publisher(snapshot);

        let left_parent = publish_text(
            &left,
            snapshot,
            "parent",
            "same-semantic-parent",
            Vec::new(),
            side_tables(50),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("left parent publishes");
        let right_parent = publish_text(
            &right,
            snapshot,
            "parent",
            "same-semantic-parent",
            Vec::new(),
            side_tables(60),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("right parent publishes");
        assert_eq!(left_parent.content_hash(), right_parent.content_hash());
        assert_ne!(left_parent.output(), right_parent.output());

        let left_child = publish_text(
            &left,
            snapshot,
            "child",
            "child",
            vec![left_parent.erase()],
            side_tables(51),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("left child publishes");
        let right_child = publish_text(
            &right,
            snapshot,
            "child",
            "child",
            vec![right_parent.erase()],
            side_tables(51),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("right child publishes");

        assert_eq!(left_child.content_hash(), right_child.content_hash());
        assert_ne!(left_child.output(), right_child.output());
    }

    #[test]
    fn named_inputs_and_side_tables_are_canonicalized() {
        let left_named = vec![
            NamedInputHash {
                name: "b".to_owned(),
                domain: "test".to_owned(),
                digest: hash(70),
            },
            NamedInputHash {
                name: "a".to_owned(),
                domain: "test".to_owned(),
                digest: hash(71),
            },
            NamedInputHash {
                name: "a".to_owned(),
                domain: "test".to_owned(),
                digest: hash(71),
            },
        ];
        let right_named = vec![
            NamedInputHash {
                name: "a".to_owned(),
                domain: "test".to_owned(),
                digest: hash(71),
            },
            NamedInputHash {
                name: "b".to_owned(),
                domain: "test".to_owned(),
                digest: hash(70),
            },
        ];
        assert_eq!(
            content_hash(b"payload", &[], left_named).expect("left hash derives"),
            content_hash(b"payload", &[], right_named).expect("right hash derives")
        );
        let conflict = content_hash(
            b"payload",
            &[],
            vec![
                NamedInputHash {
                    name: "a".to_owned(),
                    domain: "test".to_owned(),
                    digest: hash(71),
                },
                NamedInputHash {
                    name: "a".to_owned(),
                    domain: "test".to_owned(),
                    digest: hash(72),
                },
            ],
        )
        .expect_err("conflicting duplicate named input is rejected");
        assert!(matches!(conflict, PublishError::InvalidSideTable { .. }));

        let left_tables = IrSideTables {
            diagnostics: vec![
                SideTableRecord::new("diagnostic", "B", hash(73)),
                SideTableRecord::new("diagnostic", "A", hash(74)),
            ],
            ..IrSideTables::default()
        };
        let right_tables = IrSideTables {
            diagnostics: vec![
                SideTableRecord::new("diagnostic", "A", hash(74)),
                SideTableRecord::new("diagnostic", "B", hash(73)),
            ],
            ..IrSideTables::default()
        };
        assert_eq!(
            side_table_hash(&left_tables).expect("left side-table hash derives"),
            side_table_hash(&right_tables).expect("right side-table hash derives")
        );
    }

    #[test]
    fn rejects_wrong_obsolete_open_and_stale_publication() {
        let current_snapshot = snapshot(3);
        let publisher = publisher(current_snapshot);
        let unknown_snapshot = snapshot(4);

        let unknown = publish_text(
            &publisher,
            unknown_snapshot,
            "unit",
            "payload",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect_err("unknown snapshot is rejected");
        assert!(matches!(unknown, PublishError::UnknownSnapshot { .. }));

        let open = publish_text(
            &publisher,
            current_snapshot,
            "unit",
            "payload",
            Vec::new(),
            IrSideTables::default(),
            (OutputOrigin::OpenBuffer, PublicationTarget::CurrentPackage),
        )
        .expect_err("open-buffer current publication is rejected");
        assert!(matches!(open, PublishError::OpenBufferOutput { .. }));

        let stale = publish_text(
            &publisher,
            current_snapshot,
            "unit",
            "payload",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::RetainedStaleSnapshot,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect_err("retained stale current publication is rejected");
        assert!(matches!(
            stale,
            PublishError::RetainedStaleSnapshotOutput { .. }
        ));

        publisher
            .mark_obsolete(current_snapshot)
            .expect("snapshot can be marked obsolete");
        let obsolete = publish_text(
            &publisher,
            current_snapshot,
            "unit",
            "payload",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect_err("obsolete current publication is rejected");
        assert!(matches!(obsolete, PublishError::ObsoleteSnapshot { .. }));
    }

    #[test]
    fn snapshot_replacement_makes_old_outputs_stale_but_retained_until_release() {
        let old_snapshot = snapshot(31);
        let new_snapshot = snapshot(32);
        let publisher = publisher(old_snapshot);
        let old_output = publish_text(
            &publisher,
            old_snapshot,
            "unit",
            "old",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("old snapshot output publishes");
        let owner = crate::storage::RetainOwner::new("stale-lsp-snapshot");
        publisher
            .storage()
            .retain(old_output.output(), owner.clone())
            .expect("old output can be retained");

        publisher
            .replace_current_snapshot(old_snapshot, new_snapshot)
            .expect("snapshot replacement succeeds");

        assert!(matches!(
            publisher.validate_current_snapshot(old_snapshot),
            Err(PublishError::ObsoleteSnapshot { snapshot }) if snapshot == old_snapshot
        ));
        publisher
            .validate_current_snapshot(new_snapshot)
            .expect("new snapshot is current");
        assert!(matches!(
            publisher.validate_current_output(old_snapshot, old_output.any()),
            Err(PublishError::ObsoleteSnapshot { snapshot }) if snapshot == old_snapshot
        ));

        let republish = publish_text(
            &publisher,
            old_snapshot,
            "unit",
            "old-again",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect_err("superseded snapshot cannot publish current output");
        assert!(matches!(
            republish,
            PublishError::ObsoleteSnapshot { snapshot } if snapshot == old_snapshot
        ));

        let retained = publisher.storage().collect(crate::storage::CollectInput {
            snapshot: old_snapshot,
            protected_outputs: Vec::new(),
        });
        assert_eq!(retained.retained_outputs, 1);
        assert_eq!(
            &*publisher
                .storage()
                .get(&old_output)
                .expect("retained stale output remains readable"),
            "old"
        );

        publisher
            .storage()
            .release(old_output.output(), &owner)
            .expect("release succeeds");
        let released = publisher.storage().collect(crate::storage::CollectInput {
            snapshot: old_snapshot,
            protected_outputs: Vec::new(),
        });
        assert_eq!(released.outputs_dropped, 1);
        assert!(matches!(
            publisher.storage().get(&old_output),
            Err(StorageError::CollectedOutput { output }) if output == old_output.output()
        ));
    }

    #[test]
    fn registering_current_snapshot_does_not_revive_superseded_snapshot() {
        let old_snapshot = snapshot(33);
        let new_snapshot = snapshot(34);
        let publisher = publisher(old_snapshot);
        publisher
            .replace_current_snapshot(old_snapshot, new_snapshot)
            .expect("snapshot replacement succeeds");

        publisher.register_current_snapshot(old_snapshot);
        let error = publish_text(
            &publisher,
            old_snapshot,
            "unit",
            "revived",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect_err("superseded snapshot cannot be revived as current");

        assert!(matches!(
            error,
            PublishError::ObsoleteSnapshot { snapshot } if snapshot == old_snapshot
        ));
    }

    #[test]
    fn rejects_invalid_snapshot_replacement_requests() {
        let current_snapshot = snapshot(35);
        let publisher = publisher(current_snapshot);
        let self_replacement = publisher
            .replace_current_snapshot(current_snapshot, current_snapshot)
            .expect_err("self replacement is rejected");
        assert!(matches!(
            self_replacement,
            PublishError::InvalidSnapshotReplacement { snapshot } if snapshot == current_snapshot
        ));

        let unknown = snapshot(36);
        let unknown_replacement = publisher
            .replace_current_snapshot(unknown, snapshot(37))
            .expect_err("unknown old snapshot is rejected");
        assert!(matches!(
            unknown_replacement,
            PublishError::UnknownSnapshot { snapshot } if snapshot == unknown
        ));

        let first_replacement = snapshot(38);
        let second_replacement = snapshot(39);
        publisher
            .replace_current_snapshot(current_snapshot, first_replacement)
            .expect("first replacement succeeds");
        let reused_old_snapshot = publisher
            .replace_current_snapshot(first_replacement, current_snapshot)
            .expect_err("superseded snapshot cannot become a new current target");
        assert!(matches!(
            reused_old_snapshot,
            PublishError::SupersededSnapshot {
                snapshot,
                replacement
            } if snapshot == current_snapshot && replacement == first_replacement
        ));

        publisher
            .replace_current_snapshot(first_replacement, second_replacement)
            .expect("current replacement chain may advance with a fresh snapshot");
    }

    #[test]
    fn rejects_unallowed_work_unit_before_publish() {
        let snapshot = snapshot(13);
        let publisher = publisher(snapshot);
        let error = publish_text(
            &publisher,
            snapshot,
            "not-allowed",
            "payload",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect_err("unallowed work unit is rejected");

        assert!(matches!(error, PublishError::WorkUnitNotAllowed { .. }));
    }

    #[test]
    fn rejects_parent_from_different_snapshot_before_cache_adapter_exists() {
        let current = snapshot(5);
        let old = snapshot(6);
        let publisher = publisher(current);
        publisher.register_current_snapshot(old);
        publisher.allow_work_unit(AllowedWorkUnit::new(
            phase(),
            output_kind(),
            work_unit("old-parent"),
        ));
        let parent = publish_text(
            &publisher,
            old,
            "old-parent",
            "parent",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("old parent publishes");
        let error = publish_text(
            &publisher,
            current,
            "child",
            "child",
            vec![parent.erase()],
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect_err("cross-snapshot parent is rejected");

        assert!(matches!(error, PublishError::ParentSnapshotMismatch { .. }));
    }

    #[test]
    fn rejects_parent_handle_from_foreign_storage_before_hashing() {
        let snapshot = snapshot(14);
        let local = publisher(snapshot);
        let foreign = publisher(snapshot);
        let parent = publish_text(
            &foreign,
            snapshot,
            "parent",
            "parent",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("foreign parent publishes");

        let error = publish_text(
            &local,
            snapshot,
            "child",
            "child",
            vec![parent.erase()],
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect_err("foreign parent handle is rejected");

        match error {
            PublishError::Storage { error } => {
                assert!(matches!(*error, StorageError::UnknownOutput { .. }));
            }
            other => panic!("expected storage ownership rejection, got {other:?}"),
        }
    }

    #[test]
    fn rejects_collected_parent_handle_before_hashing() {
        let snapshot = snapshot(15);
        let publisher = publisher(snapshot);
        let parent = publish_text(
            &publisher,
            snapshot,
            "parent",
            "parent",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("parent publishes");
        let summary = publisher.storage().collect(crate::storage::CollectInput {
            snapshot,
            protected_outputs: Vec::new(),
        });
        assert_eq!(summary.outputs_dropped, 1);

        let error = publish_text(
            &publisher,
            snapshot,
            "child",
            "child",
            vec![parent.erase()],
            IrSideTables::default(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect_err("collected parent handle is rejected");

        match error {
            PublishError::Storage { error } => {
                assert!(matches!(*error, StorageError::CollectedOutput { .. }));
            }
            other => panic!("expected collected parent rejection, got {other:?}"),
        }
    }

    #[test]
    fn metadata_mismatch_and_failed_publish_do_not_expose_partial_output() {
        let snapshot = snapshot(7);
        let publisher = publisher(snapshot);
        let expected_registry = SnapshotHandleRegistry::new();
        expected_registry.register_snapshot(snapshot);
        let expected = expected_registry
            .register_output(OutputIdentityInput {
                snapshot,
                phase: phase(),
                work_unit: work_unit("child"),
                output_kind: output_kind(),
                content_hash: content_hash(b"payload", &[], vec![named(1)])
                    .expect("content hash is derived"),
                side_table_hash: side_table_hash(&IrSideTables::default())
                    .expect("side-table hash is derived"),
                parents: Vec::new(),
                named_input_hashes: vec![named(1)],
            })
            .expect("expected output id is derivable");
        let slot = publisher.allocate::<String>(
            snapshot,
            phase(),
            work_unit("parent"),
            output_kind(),
            SchemaVersion::new(1),
        );

        let error = publisher
            .publish(PublishOutputInput {
                slot,
                snapshot,
                phase: phase(),
                work_unit: work_unit("child"),
                output_kind: output_kind(),
                schema_version: SchemaVersion::new(1),
                payload: "payload".to_owned(),
                canonical_payload: Some(b"payload".to_vec()),
                decode: BlobDecoder::new(|bytes| {
                    String::from_utf8(bytes.to_vec())
                        .map_err(|error| crate::storage::BlobDecodeError::new(error.to_string()))
                }),
                parents: Vec::new(),
                named_input_hashes: vec![named(1)],
                side_tables: IrSideTables::default(),
                origin: OutputOrigin::PackageSource,
                target: PublicationTarget::CurrentPackage,
            })
            .expect_err("slot/work-unit mismatch fails");

        assert!(matches!(error, PublishError::Storage { .. }));
        assert!(
            publisher
                .registry()
                .output_lineage(expected.output)
                .is_none()
        );
    }

    #[test]
    fn storage_failure_after_registration_rolls_back_lineage() {
        let snapshot = snapshot(11);
        let publisher = publisher(snapshot);
        let foreign_storage = IrStorageService::new();
        let foreign_slot = foreign_storage.allocate::<String>(crate::storage::AllocateSlotInput {
            snapshot,
            phase: phase(),
            work_unit: work_unit("unit"),
            output_kind: output_kind(),
            schema_version: SchemaVersion::new(1),
        });
        let expected_registry = SnapshotHandleRegistry::new();
        expected_registry.register_snapshot(snapshot);
        let expected = expected_registry
            .register_output(OutputIdentityInput {
                snapshot,
                phase: phase(),
                work_unit: work_unit("unit"),
                output_kind: output_kind(),
                content_hash: content_hash(b"payload", &[], vec![named(1)])
                    .expect("content hash is derived"),
                side_table_hash: side_table_hash(&IrSideTables::default())
                    .expect("side-table hash is derived"),
                parents: Vec::new(),
                named_input_hashes: vec![named(1)],
            })
            .expect("expected output id is derivable");

        let error = publisher
            .publish(PublishOutputInput {
                slot: foreign_slot,
                snapshot,
                phase: phase(),
                work_unit: work_unit("unit"),
                output_kind: output_kind(),
                schema_version: SchemaVersion::new(1),
                payload: "payload".to_owned(),
                canonical_payload: Some(b"payload".to_vec()),
                decode: BlobDecoder::new(|bytes| {
                    String::from_utf8(bytes.to_vec())
                        .map_err(|error| crate::storage::BlobDecodeError::new(error.to_string()))
                }),
                parents: Vec::new(),
                named_input_hashes: vec![named(1)],
                side_tables: IrSideTables::default(),
                origin: OutputOrigin::PackageSource,
                target: PublicationTarget::CurrentPackage,
            })
            .expect_err("foreign storage slot fails after lineage registration");

        assert!(matches!(error, PublishError::Storage { .. }));
        assert!(
            publisher
                .registry()
                .output_lineage(expected.output)
                .is_none()
        );
    }

    #[test]
    fn side_tables_round_trip_from_published_handle() {
        let snapshot = snapshot(8);
        let publisher = publisher(snapshot);
        let side_tables = side_tables(40);
        let handle = publish_text(
            &publisher,
            snapshot,
            "unit",
            "payload",
            Vec::new(),
            side_tables.clone(),
            (
                OutputOrigin::PackageSource,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("publish succeeds");

        assert_eq!(
            &*publisher
                .storage()
                .side_tables(&handle)
                .expect("side tables round-trip"),
            &side_tables
        );
    }

    #[test]
    fn publisher_handles_do_not_carry_proof_cache_or_trust_authority() {
        let snapshot = snapshot(9);
        let publisher = publisher(snapshot);
        let handle = publish_text(
            &publisher,
            snapshot,
            "unit",
            "proof=true cache_key=abc dependency_fingerprint=abc trusted=true",
            Vec::new(),
            IrSideTables::default(),
            (
                OutputOrigin::ValidatedCacheInput,
                PublicationTarget::CurrentPackage,
            ),
        )
        .expect("cache-shaped payload can publish only as opaque IR");

        let debug = format!("{:?}", handle.erase());
        for forbidden in [
            "cache_key",
            "dependency_fingerprint",
            "trusted=true",
            "proof=true",
        ] {
            assert!(
                !debug.contains(forbidden),
                "publisher handle metadata must not expose `{forbidden}` authority"
            );
        }
    }
}
