//! Snapshot-scoped IR identity tables and output lineage.
//!
//! This module is specified in
//! [`identity.md`](../../../../doc/design/mizar-ir/en/identity.md).

use std::{collections::HashMap, error::Error, fmt, sync::Mutex};

use mizar_session::{BuildSnapshotId, Hash, SourceId};

const MODULE_ID_DOMAIN: &str = "mizar-ir/module-id/v1";
const ITEM_ID_DOMAIN: &str = "mizar-ir/item-id/v1";
const EXPR_ID_DOMAIN: &str = "mizar-ir/expr-id/v1";
const VC_ID_DOMAIN: &str = "mizar-ir/vc-id/v1";
const PHASE_OUTPUT_ID_DOMAIN: &str = "mizar-ir/phase-output-id/v1";
const SOURCE_HASH_DOMAIN: &str = "mizar-ir/session-source-hash/v1";
const CONTENT_HASH_DOMAIN: &str = "mizar-ir/output-content-hash/v1";
const SIDE_TABLE_HASH_DOMAIN: &str = "mizar-ir/output-side-table-hash/v1";
const NAMED_INPUT_HASH_DOMAIN: &str = "mizar-ir/named-input-hash/v1";

/// Snapshot-scoped module identity owned by `mizar-ir`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct ModuleId(Hash);

/// Snapshot-scoped item identity owned by `mizar-ir`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct ItemId(Hash);

/// Snapshot-scoped expression identity owned by `mizar-ir`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct ExprId(Hash);

/// Snapshot-scoped VC ordering identity owned by `mizar-ir`.
///
/// A `VcId` is not a proof-reuse identity and is never proof evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct VcId(Hash);

/// Snapshot-scoped sealed phase-output identity owned by `mizar-ir`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, std::hash::Hash)]
pub struct PhaseOutputId(Hash);

/// Pipeline phase label used for output identity.
#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
pub struct PipelinePhase(String);

/// Phase-local work-unit label used for output identity.
#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
pub struct WorkUnit(String);

/// Runtime output kind tag used to keep typed handles from aliasing.
#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
pub struct OutputKind(String);

/// Named non-output hash declared by a phase producer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedInputHash {
    /// Stable producer-local input name.
    pub name: String,
    /// Hash domain for the digest.
    pub domain: String,
    /// Digest value.
    pub digest: Hash,
}

/// Input for module id assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleIdentityInput {
    /// Snapshot that scopes the id.
    pub snapshot: BuildSnapshotId,
    /// Stable package identity.
    pub package_id: String,
    /// Canonical module path.
    pub module_path: String,
    /// Optional session-owned source id carried with the module.
    ///
    /// This id is consumed metadata, not a canonical hash input, because
    /// `mizar-session` intentionally keeps `SourceId` non-persistable.
    pub source_id: Option<SourceId>,
    /// Exact source hash.
    pub source_hash: Hash,
}

/// Input for item id assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemIdentityInput {
    /// Snapshot that scopes the id.
    pub snapshot: BuildSnapshotId,
    /// Owning module id.
    pub module: ModuleId,
    /// Producer-owned item kind.
    pub item_kind: String,
    /// Producer-owned normalized origin key.
    pub origin_key: String,
    /// Producer-owned declaration-order key.
    pub declaration_order_key: String,
}

/// Input for expression id assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprIdentityInput {
    /// Snapshot that scopes the id.
    pub snapshot: BuildSnapshotId,
    /// Owning module id.
    pub module: ModuleId,
    /// Optional owning item id.
    pub item: Option<ItemId>,
    /// Producer-owned expression kind.
    pub expression_kind: String,
    /// Producer-owned expression path key.
    pub producer_path_key: String,
}

/// Input for VC id assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VcIdentityInput {
    /// Snapshot that scopes the id.
    pub snapshot: BuildSnapshotId,
    /// Owning module id.
    pub module: ModuleId,
    /// Optional owning item id.
    pub item: Option<ItemId>,
    /// Producer-owned obligation order key.
    pub obligation_order_key: String,
    /// Optional canonical VC fingerprint from the VC owner.
    pub canonical_vc_fingerprint: Option<Hash>,
}

/// Input for sealed phase-output id assignment and lineage registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputIdentityInput {
    /// Snapshot that scopes the output.
    pub snapshot: BuildSnapshotId,
    /// Producing phase.
    pub phase: PipelinePhase,
    /// Producing work unit.
    pub work_unit: WorkUnit,
    /// Runtime output kind tag.
    pub output_kind: OutputKind,
    /// Semantic content hash of the sealed output.
    pub content_hash: Hash,
    /// Side-table hash of source maps, diagnostics, explanations, and docs.
    pub side_table_hash: Hash,
    /// Parent phase outputs consumed by this producer.
    pub parents: Vec<PhaseOutputId>,
    /// Non-output input hashes declared by this producer.
    pub named_input_hashes: Vec<NamedInputHash>,
}

/// Registered phase-output lineage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseOutputLineage {
    /// Output id.
    pub output: PhaseOutputId,
    /// Snapshot that owns the output.
    pub snapshot: BuildSnapshotId,
    /// Producing phase.
    pub phase: PipelinePhase,
    /// Producing work unit.
    pub work_unit: WorkUnit,
    /// Runtime output kind tag.
    pub output_kind: OutputKind,
    /// Parent output ids in canonical order.
    pub parents: Vec<PhaseOutputId>,
    /// Non-output input hashes in canonical order.
    pub named_input_hashes: Vec<NamedInputHash>,
    /// Semantic content hash.
    pub content_hash: Hash,
    /// Side-table hash.
    pub side_table_hash: Hash,
}

/// Result of registering phase-output lineage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhaseOutputRegistration {
    /// Registered lineage.
    pub(crate) lineage: PhaseOutputLineage,
    /// Whether registration inserted new registry state.
    pub(crate) status: PhaseOutputRegistrationStatus,
}

/// Phase-output registration status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub(crate) enum PhaseOutputRegistrationStatus {
    /// The lineage was newly inserted.
    Inserted,
    /// The same lineage was already registered.
    Existing,
}

/// Snapshot-scoped identity and lineage registry.
#[derive(Debug, Default)]
pub struct SnapshotHandleRegistry {
    state: Mutex<RegistryState>,
}

#[derive(Debug, Default)]
struct RegistryState {
    snapshots: HashMap<BuildSnapshotId, SnapshotState>,
    outputs: HashMap<PhaseOutputId, PhaseOutputLineage>,
}

#[derive(Debug, Default)]
struct SnapshotState {
    modules: HashMap<ModuleId, CanonicalIdentityRecord>,
    module_keys: HashMap<CanonicalIdentityKey, ModuleId>,
    items: HashMap<ItemId, CanonicalIdentityRecord>,
    item_keys: HashMap<CanonicalIdentityKey, ItemId>,
    item_modules: HashMap<ItemId, ModuleId>,
    expressions: HashMap<ExprId, CanonicalIdentityRecord>,
    expression_keys: HashMap<CanonicalIdentityKey, ExprId>,
    vcs: HashMap<VcId, CanonicalIdentityRecord>,
    vc_keys: HashMap<CanonicalIdentityKey, VcId>,
    output_keys: HashMap<CanonicalIdentityKey, PhaseOutputId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CanonicalIdentityRecord {
    family: &'static str,
    fields: Vec<CanonicalField>,
}

#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
struct CanonicalIdentityKey {
    family: &'static str,
    fields: Vec<CanonicalField>,
}

#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
struct CanonicalField {
    name: &'static str,
    value: CanonicalValue,
}

#[derive(Debug, Clone, PartialEq, Eq, std::hash::Hash)]
enum CanonicalValue {
    Hash { domain: &'static str, value: Hash },
    Id(Hash),
    OptionalHash(Option<Hash>),
    String(String),
}

/// Errors reported by identity registration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum IdentityError {
    /// The snapshot has not been registered with this registry.
    UnknownSnapshot {
        /// Missing snapshot.
        snapshot: BuildSnapshotId,
    },
    /// A required producer-owned identity field was empty.
    MissingRequiredField {
        /// Field path.
        field: &'static str,
    },
    /// A canonical duplicate key had conflicting payload.
    ConflictingDuplicate {
        /// Collection name.
        collection: &'static str,
        /// Duplicate key.
        key: String,
    },
    /// A parent output is unknown to the registry.
    UnknownParentOutput {
        /// Parent output id.
        parent: PhaseOutputId,
    },
    /// A parent output belongs to an incompatible snapshot.
    IncompatibleSnapshotParent {
        /// Output snapshot.
        snapshot: BuildSnapshotId,
        /// Parent output id.
        parent: PhaseOutputId,
        /// Parent snapshot.
        parent_snapshot: BuildSnapshotId,
    },
    /// The owning module id was not registered in this snapshot.
    UnknownModuleId {
        /// Missing module id.
        module: ModuleId,
    },
    /// The owning item id was not registered in this snapshot.
    UnknownItemId {
        /// Missing item id.
        item: ItemId,
    },
    /// The owning item id belongs to a different module in this snapshot.
    ItemModuleMismatch {
        /// Module supplied by the caller.
        module: ModuleId,
        /// Item supplied by the caller.
        item: ItemId,
        /// Module recorded for the item.
        item_module: ModuleId,
    },
    /// A previously registered id was observed with different canonical fields.
    IdentityCollision {
        /// Identity family.
        family: &'static str,
    },
    /// A previously registered output id was observed with different lineage.
    OutputCollision {
        /// Output id.
        output: PhaseOutputId,
    },
    /// A lineage record no longer matches the canonical output identity it
    /// carries.
    LineageIdentityMismatch {
        /// Output id carried by the lineage.
        output: PhaseOutputId,
    },
}

impl SnapshotHandleRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a snapshot as known to `mizar-ir`.
    pub fn register_snapshot(&self, snapshot: BuildSnapshotId) {
        self.state
            .lock()
            .expect("identity registry mutex poisoned")
            .snapshots
            .entry(snapshot)
            .or_default();
    }

    /// Assigns a deterministic module id.
    pub fn module_id(&self, input: ModuleIdentityInput) -> Result<ModuleId, IdentityError> {
        reject_empty("module.package_id", &input.package_id)?;
        reject_empty("module.module_path", &input.module_path)?;

        let key_fields = vec![
            field_string("package_id", input.package_id.clone()),
            field_string("module_path", input.module_path.clone()),
        ];
        let fields = vec![
            field_string("package_id", input.package_id.clone()),
            field_string("module_path", input.module_path.clone()),
            field_hash("source_hash", SOURCE_HASH_DOMAIN, input.source_hash),
        ];
        let id = ModuleId(hash_identity(MODULE_ID_DOMAIN, input.snapshot, &fields)?);
        let key = canonical_key(MODULE_ID_DOMAIN, key_fields);
        let record = CanonicalIdentityRecord {
            family: MODULE_ID_DOMAIN,
            fields,
        };

        let mut state = self.state.lock().expect("identity registry mutex poisoned");
        let snapshot = snapshot_state_mut(&mut state, input.snapshot)?;
        insert_identity_with_key(
            &mut snapshot.module_keys,
            &mut snapshot.modules,
            "modules",
            key,
            id,
            record,
        )?;
        Ok(id)
    }

    /// Assigns a deterministic item id.
    pub fn item_id(&self, input: ItemIdentityInput) -> Result<ItemId, IdentityError> {
        reject_empty("item.item_kind", &input.item_kind)?;
        reject_empty("item.origin_key", &input.origin_key)?;
        reject_empty("item.declaration_order_key", &input.declaration_order_key)?;

        let fields = vec![
            field_id("module", input.module.hash()),
            field_string("item_kind", input.item_kind.clone()),
            field_string("origin_key", input.origin_key.clone()),
            field_string("declaration_order_key", input.declaration_order_key.clone()),
        ];
        let id = ItemId(hash_identity(ITEM_ID_DOMAIN, input.snapshot, &fields)?);
        let key = canonical_key(ITEM_ID_DOMAIN, fields.clone());
        let record = CanonicalIdentityRecord {
            family: ITEM_ID_DOMAIN,
            fields,
        };

        let mut state = self.state.lock().expect("identity registry mutex poisoned");
        let snapshot = snapshot_state_mut(&mut state, input.snapshot)?;
        reject_unknown_module(snapshot, input.module)?;
        insert_identity_with_key(
            &mut snapshot.item_keys,
            &mut snapshot.items,
            "items",
            key,
            id,
            record,
        )?;
        snapshot.item_modules.insert(id, input.module);
        Ok(id)
    }

    /// Assigns a deterministic expression id.
    pub fn expr_id(&self, input: ExprIdentityInput) -> Result<ExprId, IdentityError> {
        reject_empty("expr.expression_kind", &input.expression_kind)?;
        reject_empty("expr.producer_path_key", &input.producer_path_key)?;

        let fields = vec![
            field_id("module", input.module.hash()),
            field_optional_id("item", input.item.map(ItemId::hash)),
            field_string("expression_kind", input.expression_kind.clone()),
            field_string("producer_path_key", input.producer_path_key.clone()),
        ];
        let id = ExprId(hash_identity(EXPR_ID_DOMAIN, input.snapshot, &fields)?);
        let key = canonical_key(EXPR_ID_DOMAIN, fields.clone());
        let record = CanonicalIdentityRecord {
            family: EXPR_ID_DOMAIN,
            fields,
        };

        let mut state = self.state.lock().expect("identity registry mutex poisoned");
        let snapshot = snapshot_state_mut(&mut state, input.snapshot)?;
        reject_unknown_module(snapshot, input.module)?;
        if let Some(item) = input.item {
            reject_item_belongs_to_module(snapshot, input.module, item)?;
        }
        insert_identity_with_key(
            &mut snapshot.expression_keys,
            &mut snapshot.expressions,
            "expressions",
            key,
            id,
            record,
        )?;
        Ok(id)
    }

    /// Assigns a deterministic VC id.
    pub fn vc_id(&self, input: VcIdentityInput) -> Result<VcId, IdentityError> {
        reject_empty("vc.obligation_order_key", &input.obligation_order_key)?;

        let key_fields = vec![
            field_id("module", input.module.hash()),
            field_optional_id("item", input.item.map(ItemId::hash)),
            field_string("obligation_order_key", input.obligation_order_key.clone()),
        ];
        let fields = vec![
            field_id("module", input.module.hash()),
            field_optional_id("item", input.item.map(ItemId::hash)),
            field_string("obligation_order_key", input.obligation_order_key.clone()),
            field_optional_hash("canonical_vc_fingerprint", input.canonical_vc_fingerprint),
        ];
        let id = VcId(hash_identity(VC_ID_DOMAIN, input.snapshot, &fields)?);
        let key = canonical_key(VC_ID_DOMAIN, key_fields);
        let record = CanonicalIdentityRecord {
            family: VC_ID_DOMAIN,
            fields,
        };

        let mut state = self.state.lock().expect("identity registry mutex poisoned");
        let snapshot = snapshot_state_mut(&mut state, input.snapshot)?;
        reject_unknown_module(snapshot, input.module)?;
        if let Some(item) = input.item {
            reject_item_belongs_to_module(snapshot, input.module, item)?;
        }
        insert_identity_with_key(
            &mut snapshot.vc_keys,
            &mut snapshot.vcs,
            "vcs",
            key,
            id,
            record,
        )?;
        Ok(id)
    }

    /// Registers a sealed phase output and returns its deterministic id.
    pub fn register_output(
        &self,
        input: OutputIdentityInput,
    ) -> Result<PhaseOutputLineage, IdentityError> {
        Ok(self.register_output_with_status(input)?.lineage)
    }

    /// Registers phase-output lineage and reports whether it was newly
    /// inserted.
    pub(crate) fn register_output_with_status(
        &self,
        input: OutputIdentityInput,
    ) -> Result<PhaseOutputRegistration, IdentityError> {
        let lineage = derive_phase_output_lineage(input)?;
        let output = lineage.output;
        let key =
            phase_output_duplicate_key(&lineage.phase, &lineage.work_unit, &lineage.output_kind);

        let mut state = self.state.lock().expect("identity registry mutex poisoned");
        if !state.snapshots.contains_key(&lineage.snapshot) {
            return Err(IdentityError::UnknownSnapshot {
                snapshot: lineage.snapshot,
            });
        }
        for parent in &lineage.parents {
            let Some(parent_lineage) = state.outputs.get(parent) else {
                return Err(IdentityError::UnknownParentOutput { parent: *parent });
            };
            if parent_lineage.snapshot != lineage.snapshot {
                return Err(IdentityError::IncompatibleSnapshotParent {
                    snapshot: lineage.snapshot,
                    parent: *parent,
                    parent_snapshot: parent_lineage.snapshot,
                });
            }
        }
        let existing_for_key = state
            .snapshots
            .get(&lineage.snapshot)
            .expect("snapshot was checked above")
            .output_keys
            .get(&key)
            .copied();
        if let Some(existing) = existing_for_key
            && existing != output
        {
            return Err(IdentityError::ConflictingDuplicate {
                collection: "phase_outputs",
                key: key_display(&key),
            });
        }

        match state.outputs.get(&output) {
            Some(existing) if existing == &lineage => Ok(PhaseOutputRegistration {
                lineage: existing.clone(),
                status: PhaseOutputRegistrationStatus::Existing,
            }),
            Some(_) => Err(IdentityError::OutputCollision { output }),
            None => {
                state.outputs.insert(output, lineage.clone());
                state
                    .snapshots
                    .get_mut(&lineage.snapshot)
                    .expect("snapshot was checked above")
                    .output_keys
                    .insert(key, output);
                Ok(PhaseOutputRegistration {
                    lineage,
                    status: PhaseOutputRegistrationStatus::Inserted,
                })
            }
        }
    }

    /// Removes lineage only if it still exactly matches a newly inserted
    /// registration being rolled back.
    pub(crate) fn rollback_inserted_output(&self, lineage: &PhaseOutputLineage) -> bool {
        let key =
            phase_output_duplicate_key(&lineage.phase, &lineage.work_unit, &lineage.output_kind);
        let mut state = self.state.lock().expect("identity registry mutex poisoned");
        if state.outputs.get(&lineage.output) != Some(lineage) {
            return false;
        }
        state.outputs.remove(&lineage.output);
        if let Some(snapshot) = state.snapshots.get_mut(&lineage.snapshot)
            && snapshot.output_keys.get(&key) == Some(&lineage.output)
        {
            snapshot.output_keys.remove(&key);
        }
        true
    }

    /// Returns registered lineage for a phase output.
    pub fn output_lineage(&self, output: PhaseOutputId) -> Option<PhaseOutputLineage> {
        self.state
            .lock()
            .expect("identity registry mutex poisoned")
            .outputs
            .get(&output)
            .cloned()
    }
}

impl ModuleId {
    /// Returns the opaque hash backing this id.
    pub const fn hash(self) -> Hash {
        self.0
    }
}

impl ItemId {
    /// Returns the opaque hash backing this id.
    pub const fn hash(self) -> Hash {
        self.0
    }
}

impl ExprId {
    /// Returns the opaque hash backing this id.
    pub const fn hash(self) -> Hash {
        self.0
    }
}

impl VcId {
    /// Returns the opaque hash backing this id.
    pub const fn hash(self) -> Hash {
        self.0
    }
}

impl PhaseOutputId {
    /// Returns the opaque hash backing this id.
    pub const fn hash(self) -> Hash {
        self.0
    }
}

impl PhaseOutputLineage {
    /// Derives canonical output lineage without registering it.
    pub fn from_input(input: OutputIdentityInput) -> Result<Self, IdentityError> {
        derive_phase_output_lineage(input)
    }

    /// Validates that this lineage still matches its canonical
    /// `PhaseOutputId`.
    pub fn validate_identity(&self) -> Result<(), IdentityError> {
        let expected = derive_phase_output_lineage(OutputIdentityInput {
            snapshot: self.snapshot,
            phase: self.phase.clone(),
            work_unit: self.work_unit.clone(),
            output_kind: self.output_kind.clone(),
            content_hash: self.content_hash,
            side_table_hash: self.side_table_hash,
            parents: self.parents.clone(),
            named_input_hashes: self.named_input_hashes.clone(),
        })?;
        if expected == *self {
            Ok(())
        } else {
            Err(IdentityError::LineageIdentityMismatch {
                output: self.output,
            })
        }
    }
}

impl PipelinePhase {
    /// Creates a pipeline phase label.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the phase label.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl WorkUnit {
    /// Creates a work-unit label.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the work-unit label.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl OutputKind {
    /// Creates an output kind tag.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the output kind tag.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSnapshot { snapshot } => {
                write!(formatter, "unknown IR snapshot `{snapshot:?}`")
            }
            Self::MissingRequiredField { field } => {
                write!(formatter, "missing required identity field `{field}`")
            }
            Self::ConflictingDuplicate { collection, key } => {
                write!(
                    formatter,
                    "conflicting duplicate identity key `{key}` in `{collection}`"
                )
            }
            Self::UnknownParentOutput { parent } => {
                write!(formatter, "unknown parent output `{parent:?}`")
            }
            Self::IncompatibleSnapshotParent {
                snapshot,
                parent,
                parent_snapshot,
            } => {
                write!(
                    formatter,
                    "parent output `{parent:?}` from snapshot `{parent_snapshot:?}` \
                     cannot be reused as a current output for snapshot `{snapshot:?}`"
                )
            }
            Self::UnknownModuleId { module } => {
                write!(formatter, "unknown module id `{module:?}` for snapshot")
            }
            Self::UnknownItemId { item } => {
                write!(formatter, "unknown item id `{item:?}` for snapshot")
            }
            Self::ItemModuleMismatch {
                module,
                item,
                item_module,
            } => {
                write!(
                    formatter,
                    "item id `{item:?}` belongs to module `{item_module:?}`, not `{module:?}`"
                )
            }
            Self::IdentityCollision { family } => {
                write!(
                    formatter,
                    "identity hash collision or conflicting registration in `{family}`"
                )
            }
            Self::OutputCollision { output } => {
                write!(
                    formatter,
                    "phase output hash collision or conflicting lineage for `{output:?}`"
                )
            }
            Self::LineageIdentityMismatch { output } => {
                write!(
                    formatter,
                    "phase output lineage `{output:?}` does not match its canonical identity"
                )
            }
        }
    }
}

impl Error for IdentityError {}

fn snapshot_state_mut(
    state: &mut RegistryState,
    snapshot: BuildSnapshotId,
) -> Result<&mut SnapshotState, IdentityError> {
    state
        .snapshots
        .get_mut(&snapshot)
        .ok_or(IdentityError::UnknownSnapshot { snapshot })
}

fn reject_unknown_module(snapshot: &SnapshotState, module: ModuleId) -> Result<(), IdentityError> {
    if snapshot.modules.contains_key(&module) {
        Ok(())
    } else {
        Err(IdentityError::UnknownModuleId { module })
    }
}

fn reject_unknown_item(snapshot: &SnapshotState, item: ItemId) -> Result<(), IdentityError> {
    if snapshot.items.contains_key(&item) {
        Ok(())
    } else {
        Err(IdentityError::UnknownItemId { item })
    }
}

fn reject_item_belongs_to_module(
    snapshot: &SnapshotState,
    module: ModuleId,
    item: ItemId,
) -> Result<(), IdentityError> {
    reject_unknown_item(snapshot, item)?;
    let item_module = snapshot
        .item_modules
        .get(&item)
        .copied()
        .expect("item module is recorded with item registration");
    if item_module == module {
        Ok(())
    } else {
        Err(IdentityError::ItemModuleMismatch {
            module,
            item,
            item_module,
        })
    }
}

fn insert_identity<T: Eq + std::hash::Hash + Copy>(
    table: &mut HashMap<T, CanonicalIdentityRecord>,
    id: T,
    record: CanonicalIdentityRecord,
) -> Result<(), IdentityError> {
    match table.get(&id) {
        Some(existing) if existing == &record => Ok(()),
        Some(existing) => Err(IdentityError::IdentityCollision {
            family: existing.family,
        }),
        None => {
            table.insert(id, record);
            Ok(())
        }
    }
}

fn insert_identity_with_key<T: Eq + std::hash::Hash + Copy + PartialEq>(
    key_table: &mut HashMap<CanonicalIdentityKey, T>,
    table: &mut HashMap<T, CanonicalIdentityRecord>,
    collection: &'static str,
    key: CanonicalIdentityKey,
    id: T,
    record: CanonicalIdentityRecord,
) -> Result<(), IdentityError> {
    if let Some(existing) = key_table.get(&key).copied()
        && existing != id
    {
        return Err(IdentityError::ConflictingDuplicate {
            collection,
            key: key_display(&key),
        });
    }

    insert_identity(table, id, record)?;
    key_table.insert(key, id);
    Ok(())
}

fn canonical_key(family: &'static str, fields: Vec<CanonicalField>) -> CanonicalIdentityKey {
    CanonicalIdentityKey { family, fields }
}

fn key_display(key: &CanonicalIdentityKey) -> String {
    let fields = key
        .fields
        .iter()
        .map(|field| format!("{}={}", field.name, value_display(&field.value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{}:{fields}", key.family)
}

fn value_display(value: &CanonicalValue) -> String {
    match value {
        CanonicalValue::Hash { domain, value } => {
            format!("{domain}:{}", hash_hex(*value))
        }
        CanonicalValue::Id(value) => hash_hex(*value),
        CanonicalValue::OptionalHash(Some(value)) => hash_hex(*value),
        CanonicalValue::OptionalHash(None) => "none".to_owned(),
        CanonicalValue::String(value) => value.clone(),
    }
}

fn hash_hex(hash: Hash) -> String {
    hash.as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn canonicalize_named_input_hashes(
    mut values: Vec<NamedInputHash>,
) -> Result<Vec<NamedInputHash>, IdentityError> {
    values.sort_by(|left, right| {
        named_input_key(left)
            .cmp(&named_input_key(right))
            .then_with(|| left.digest.as_bytes().cmp(right.digest.as_bytes()))
    });

    let mut result: Vec<NamedInputHash> = Vec::new();
    for value in values {
        reject_empty("named_input_hashes.name", &value.name)?;
        reject_empty("named_input_hashes.domain", &value.domain)?;
        if let Some(existing) = result
            .iter()
            .find(|existing| named_input_key(existing) == named_input_key(&value))
        {
            if existing.digest != value.digest {
                return Err(IdentityError::ConflictingDuplicate {
                    collection: "named_input_hashes",
                    key: format!("{}:{}", value.name, value.domain),
                });
            }
            continue;
        }
        result.push(value);
    }

    Ok(result)
}

fn derive_phase_output_lineage(
    input: OutputIdentityInput,
) -> Result<PhaseOutputLineage, IdentityError> {
    reject_empty("output.phase", input.phase.as_str())?;
    reject_empty("output.work_unit", input.work_unit.as_str())?;
    reject_empty("output.output_kind", input.output_kind.as_str())?;

    let mut parents = input.parents;
    sort_phase_output_ids(&mut parents);
    parents.dedup();
    let named_input_hashes = canonicalize_named_input_hashes(input.named_input_hashes)?;

    let mut fields = vec![
        field_string("phase", input.phase.as_str().to_owned()),
        field_string("work_unit", input.work_unit.as_str().to_owned()),
        field_string("output_kind", input.output_kind.as_str().to_owned()),
        field_hash("content_hash", CONTENT_HASH_DOMAIN, input.content_hash),
        field_hash(
            "side_table_hash",
            SIDE_TABLE_HASH_DOMAIN,
            input.side_table_hash,
        ),
    ];
    for parent in &parents {
        fields.push(field_id("parent", parent.hash()));
    }
    for value in &named_input_hashes {
        fields.push(field_string("named_input_name", value.name.clone()));
        fields.push(field_string("named_input_domain", value.domain.clone()));
        fields.push(field_hash(
            "named_input_digest",
            NAMED_INPUT_HASH_DOMAIN,
            value.digest,
        ));
    }

    Ok(PhaseOutputLineage {
        output: PhaseOutputId(hash_identity(
            PHASE_OUTPUT_ID_DOMAIN,
            input.snapshot,
            &fields,
        )?),
        snapshot: input.snapshot,
        phase: input.phase,
        work_unit: input.work_unit,
        output_kind: input.output_kind,
        parents,
        named_input_hashes,
        content_hash: input.content_hash,
        side_table_hash: input.side_table_hash,
    })
}

fn phase_output_duplicate_key(
    phase: &PipelinePhase,
    work_unit: &WorkUnit,
    output_kind: &OutputKind,
) -> CanonicalIdentityKey {
    canonical_key(
        PHASE_OUTPUT_ID_DOMAIN,
        vec![
            field_string("phase", phase.as_str().to_owned()),
            field_string("work_unit", work_unit.as_str().to_owned()),
            field_string("output_kind", output_kind.as_str().to_owned()),
        ],
    )
}

fn named_input_key(value: &NamedInputHash) -> (&str, &str) {
    (&value.name, &value.domain)
}

fn hash_identity(
    domain: &'static str,
    snapshot: BuildSnapshotId,
    fields: &[CanonicalField],
) -> Result<Hash, IdentityError> {
    let snapshot_string = snapshot
        .to_published_schema_string()
        .map_err(|_| IdentityError::UnknownSnapshot { snapshot })?;
    let mut hasher = stable_hasher(domain);
    write_str_field(&mut hasher, "snapshot", &snapshot_string);
    write_len(&mut hasher, fields.len());
    for field in fields {
        write_str_field(&mut hasher, "field_name", field.name);
        write_canonical_value(&mut hasher, &field.value);
    }
    Ok(finish_hash(hasher))
}

fn stable_hasher(domain: &str) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    write_str(&mut hasher, domain);
    hasher
}

fn write_canonical_value(hasher: &mut blake3::Hasher, value: &CanonicalValue) {
    match value {
        CanonicalValue::Hash { domain, value } => {
            write_str(hasher, "hash");
            write_hash(hasher, domain, *value);
        }
        CanonicalValue::Id(value) => {
            write_str(hasher, "id");
            write_hash(hasher, "mizar-ir/id-hash/v1", *value);
        }
        CanonicalValue::OptionalHash(value) => {
            write_str(hasher, "optional-hash");
            match value {
                Some(value) => {
                    hasher.update(&[1]);
                    write_hash(hasher, "mizar-ir/optional-id-hash/v1", *value);
                }
                None => {
                    hasher.update(&[0]);
                }
            }
        }
        CanonicalValue::String(value) => {
            write_str(hasher, "string");
            write_str(hasher, value);
        }
    }
}

fn write_str_field(hasher: &mut blake3::Hasher, field: &str, value: &str) {
    write_str(hasher, field);
    write_str(hasher, value);
}

fn write_hash(hasher: &mut blake3::Hasher, domain: &str, value: Hash) {
    write_str(hasher, domain);
    hasher.update(value.as_bytes());
}

fn write_str(hasher: &mut blake3::Hasher, value: &str) {
    write_len(hasher, value.len());
    hasher.update(value.as_bytes());
}

fn write_len(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

fn finish_hash(hasher: blake3::Hasher) -> Hash {
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

fn field_string(name: &'static str, value: String) -> CanonicalField {
    CanonicalField {
        name,
        value: CanonicalValue::String(value),
    }
}

fn field_id(name: &'static str, value: Hash) -> CanonicalField {
    CanonicalField {
        name,
        value: CanonicalValue::Id(value),
    }
}

fn field_optional_id(name: &'static str, value: Option<Hash>) -> CanonicalField {
    CanonicalField {
        name,
        value: CanonicalValue::OptionalHash(value),
    }
}

fn field_optional_hash(name: &'static str, value: Option<Hash>) -> CanonicalField {
    CanonicalField {
        name,
        value: CanonicalValue::OptionalHash(value),
    }
}

fn field_hash(name: &'static str, domain: &'static str, value: Hash) -> CanonicalField {
    CanonicalField {
        name,
        value: CanonicalValue::Hash { domain, value },
    }
}

fn reject_empty(field: &'static str, value: &str) -> Result<(), IdentityError> {
    if value.trim().is_empty() {
        return Err(IdentityError::MissingRequiredField { field });
    }
    Ok(())
}

fn sort_phase_output_ids(values: &mut [PhaseOutputId]) {
    values.sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }

    fn snapshot(seed: u8) -> BuildSnapshotId {
        let mut bytes = [0; Hash::BYTE_LEN];
        bytes[0] = seed;
        let serialized = format!(
            "mizar-session-build-snapshot-v1:{}",
            bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        BuildSnapshotId::from_published_schema_str(&serialized).expect("test snapshot id is valid")
    }

    fn registered_registry(snapshot: BuildSnapshotId) -> SnapshotHandleRegistry {
        let registry = SnapshotHandleRegistry::new();
        registry.register_snapshot(snapshot);
        registry
    }

    fn module_input(snapshot: BuildSnapshotId) -> ModuleIdentityInput {
        ModuleIdentityInput {
            snapshot,
            package_id: "pkg".to_owned(),
            module_path: "Main".to_owned(),
            source_id: None,
            source_hash: hash(10),
        }
    }

    fn output_input(
        snapshot: BuildSnapshotId,
        work_unit: &str,
        parents: Vec<PhaseOutputId>,
        named_input_hashes: Vec<NamedInputHash>,
    ) -> OutputIdentityInput {
        OutputIdentityInput {
            snapshot,
            phase: PipelinePhase::new("resolve"),
            work_unit: WorkUnit::new(work_unit),
            output_kind: OutputKind::new("ResolvedAst"),
            content_hash: hash(20),
            side_table_hash: hash(21),
            parents,
            named_input_hashes,
        }
    }

    #[test]
    fn id_assignment_is_deterministic_for_identical_inputs() {
        let snapshot = snapshot(1);
        let left = registered_registry(snapshot);
        let right = registered_registry(snapshot);

        let left_module = left
            .module_id(module_input(snapshot))
            .expect("module id is assigned");
        let right_module = right
            .module_id(module_input(snapshot))
            .expect("module id is assigned");

        assert_eq!(left_module, right_module);

        let left_item = left
            .item_id(ItemIdentityInput {
                snapshot,
                module: left_module,
                item_kind: "theorem".to_owned(),
                origin_key: "Main.Th1".to_owned(),
                declaration_order_key: "0001".to_owned(),
            })
            .expect("item id is assigned");
        let right_item = right
            .item_id(ItemIdentityInput {
                snapshot,
                module: right_module,
                item_kind: "theorem".to_owned(),
                origin_key: "Main.Th1".to_owned(),
                declaration_order_key: "0001".to_owned(),
            })
            .expect("item id is assigned");

        assert_eq!(left_item, right_item);

        let left_expr = left
            .expr_id(ExprIdentityInput {
                snapshot,
                module: left_module,
                item: Some(left_item),
                expression_kind: "formula".to_owned(),
                producer_path_key: "proof/0/thesis".to_owned(),
            })
            .expect("expr id is assigned");
        let right_expr = right
            .expr_id(ExprIdentityInput {
                snapshot,
                module: right_module,
                item: Some(right_item),
                expression_kind: "formula".to_owned(),
                producer_path_key: "proof/0/thesis".to_owned(),
            })
            .expect("expr id is assigned");

        assert_eq!(left_expr, right_expr);

        let left_vc = left
            .vc_id(VcIdentityInput {
                snapshot,
                module: left_module,
                item: Some(left_item),
                obligation_order_key: "vc/0001".to_owned(),
                canonical_vc_fingerprint: Some(hash(30)),
            })
            .expect("vc id is assigned");
        let right_vc = right
            .vc_id(VcIdentityInput {
                snapshot,
                module: right_module,
                item: Some(right_item),
                obligation_order_key: "vc/0001".to_owned(),
                canonical_vc_fingerprint: Some(hash(30)),
            })
            .expect("vc id is assigned");

        assert_eq!(left_vc, right_vc);
    }

    #[test]
    fn module_id_does_not_hash_session_source_id() {
        let snapshot = snapshot(16);
        let registry = registered_registry(snapshot);

        let without_source = registry
            .module_id(module_input(snapshot))
            .expect("module without source id is assigned");
        let with_source = registry
            .module_id(ModuleIdentityInput {
                source_id: Some(test_source_id(42)),
                ..module_input(snapshot)
            })
            .expect("source id metadata does not affect module id");

        assert_eq!(without_source, with_source);
    }

    #[test]
    fn conflicting_duplicate_named_inputs_are_rejected() {
        let snapshot = snapshot(2);
        let registry = registered_registry(snapshot);

        let error = registry
            .register_output(output_input(
                snapshot,
                "unit",
                Vec::new(),
                vec![
                    NamedInputHash {
                        name: "source".to_owned(),
                        domain: "domain".to_owned(),
                        digest: hash(1),
                    },
                    NamedInputHash {
                        name: "source".to_owned(),
                        domain: "domain".to_owned(),
                        digest: hash(2),
                    },
                ],
            ))
            .expect_err("conflicting duplicate must be rejected");

        assert_eq!(
            error,
            IdentityError::ConflictingDuplicate {
                collection: "named_input_hashes",
                key: "source:domain".to_owned(),
            }
        );
    }

    #[test]
    fn conflicting_duplicate_module_key_is_rejected() {
        let snapshot = snapshot(13);
        let registry = registered_registry(snapshot);

        registry
            .module_id(module_input(snapshot))
            .expect("first module id is assigned");
        let mut changed = module_input(snapshot);
        changed.source_hash = hash(99);

        let error = registry
            .module_id(changed)
            .expect_err("same module key with different source hash must fail");

        assert!(matches!(
            error,
            IdentityError::ConflictingDuplicate {
                collection: "modules",
                ..
            }
        ));
    }

    #[test]
    fn conflicting_duplicate_phase_output_key_is_rejected() {
        let snapshot = snapshot(14);
        let registry = registered_registry(snapshot);

        registry
            .register_output(output_input(snapshot, "same", Vec::new(), Vec::new()))
            .expect("first output is registered");
        let mut changed = output_input(snapshot, "same", Vec::new(), Vec::new());
        changed.content_hash = hash(88);

        let error = registry
            .register_output(changed)
            .expect_err("same phase output key with different content must fail");

        assert!(matches!(
            error,
            IdentityError::ConflictingDuplicate {
                collection: "phase_outputs",
                ..
            }
        ));
    }

    #[test]
    fn conflicting_duplicate_vc_key_is_rejected() {
        let snapshot = snapshot(17);
        let registry = registered_registry(snapshot);
        let module = registry
            .module_id(module_input(snapshot))
            .expect("module is registered");

        registry
            .vc_id(VcIdentityInput {
                snapshot,
                module,
                item: None,
                obligation_order_key: "vc/0001".to_owned(),
                canonical_vc_fingerprint: Some(hash(1)),
            })
            .expect("first vc is registered");
        let error = registry
            .vc_id(VcIdentityInput {
                snapshot,
                module,
                item: None,
                obligation_order_key: "vc/0001".to_owned(),
                canonical_vc_fingerprint: Some(hash(2)),
            })
            .expect_err("same vc key with different fingerprint must fail");

        assert!(matches!(
            error,
            IdentityError::ConflictingDuplicate {
                collection: "vcs",
                ..
            }
        ));
    }

    #[test]
    fn phase_output_id_assignment_is_deterministic_for_identical_inputs() {
        let snapshot = snapshot(12);
        let left = registered_registry(snapshot);
        let right = registered_registry(snapshot);

        let left_output = left
            .register_output(output_input(
                snapshot,
                "same",
                Vec::new(),
                vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: "domain".to_owned(),
                    digest: hash(3),
                }],
            ))
            .expect("left output is registered");
        let right_output = right
            .register_output(output_input(
                snapshot,
                "same",
                Vec::new(),
                vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: "domain".to_owned(),
                    digest: hash(3),
                }],
            ))
            .expect("right output is registered");

        assert_eq!(left_output.output, right_output.output);
        assert_eq!(left_output, right_output);
    }

    #[test]
    fn incompatible_snapshot_parent_reuse_is_rejected() {
        let old_snapshot = snapshot(3);
        let new_snapshot = snapshot(4);
        let registry = SnapshotHandleRegistry::new();
        registry.register_snapshot(old_snapshot);
        registry.register_snapshot(new_snapshot);

        let old_output = registry
            .register_output(output_input(old_snapshot, "old", Vec::new(), Vec::new()))
            .expect("old output is registered");
        let error = registry
            .register_output(output_input(
                new_snapshot,
                "new",
                vec![old_output.output],
                Vec::new(),
            ))
            .expect_err("cross-snapshot parent reuse must fail");

        assert_eq!(
            error,
            IdentityError::IncompatibleSnapshotParent {
                snapshot: new_snapshot,
                parent: old_output.output,
                parent_snapshot: old_snapshot,
            }
        );
    }

    #[test]
    fn incompatible_snapshot_owner_reuse_is_rejected() {
        let old_snapshot = snapshot(8);
        let new_snapshot = snapshot(9);
        let registry = SnapshotHandleRegistry::new();
        registry.register_snapshot(old_snapshot);
        registry.register_snapshot(new_snapshot);

        let old_module = registry
            .module_id(module_input(old_snapshot))
            .expect("old module is registered");

        let error = registry
            .item_id(ItemIdentityInput {
                snapshot: new_snapshot,
                module: old_module,
                item_kind: "theorem".to_owned(),
                origin_key: "Main.Th1".to_owned(),
                declaration_order_key: "0001".to_owned(),
            })
            .expect_err("module from old snapshot must not be reused");

        assert_eq!(error, IdentityError::UnknownModuleId { module: old_module });
    }

    #[test]
    fn incompatible_snapshot_item_owner_reuse_is_rejected() {
        let old_snapshot = snapshot(18);
        let new_snapshot = snapshot(19);
        let registry = SnapshotHandleRegistry::new();
        registry.register_snapshot(old_snapshot);
        registry.register_snapshot(new_snapshot);

        let old_module = registry
            .module_id(module_input(old_snapshot))
            .expect("old module is registered");
        let old_item = registry
            .item_id(ItemIdentityInput {
                snapshot: old_snapshot,
                module: old_module,
                item_kind: "theorem".to_owned(),
                origin_key: "Main.Th1".to_owned(),
                declaration_order_key: "0001".to_owned(),
            })
            .expect("old item is registered");
        let new_module = registry
            .module_id(module_input(new_snapshot))
            .expect("new module is registered");

        let expr_error = registry
            .expr_id(ExprIdentityInput {
                snapshot: new_snapshot,
                module: new_module,
                item: Some(old_item),
                expression_kind: "formula".to_owned(),
                producer_path_key: "proof/0/thesis".to_owned(),
            })
            .expect_err("old item cannot be reused for new expression");
        let vc_error = registry
            .vc_id(VcIdentityInput {
                snapshot: new_snapshot,
                module: new_module,
                item: Some(old_item),
                obligation_order_key: "vc/0001".to_owned(),
                canonical_vc_fingerprint: Some(hash(30)),
            })
            .expect_err("old item cannot be reused for new vc");

        assert_eq!(expr_error, IdentityError::UnknownItemId { item: old_item });
        assert_eq!(vc_error, IdentityError::UnknownItemId { item: old_item });
    }

    #[test]
    fn mismatched_item_module_owner_is_rejected() {
        let snapshot = snapshot(15);
        let registry = registered_registry(snapshot);
        let first_module = registry
            .module_id(module_input(snapshot))
            .expect("first module is registered");
        let second_module = registry
            .module_id(ModuleIdentityInput {
                snapshot,
                package_id: "pkg".to_owned(),
                module_path: "Other".to_owned(),
                source_id: None,
                source_hash: hash(11),
            })
            .expect("second module is registered");
        let item = registry
            .item_id(ItemIdentityInput {
                snapshot,
                module: first_module,
                item_kind: "theorem".to_owned(),
                origin_key: "Main.Th1".to_owned(),
                declaration_order_key: "0001".to_owned(),
            })
            .expect("item is registered under first module");

        let error = registry
            .expr_id(ExprIdentityInput {
                snapshot,
                module: second_module,
                item: Some(item),
                expression_kind: "formula".to_owned(),
                producer_path_key: "proof/0/thesis".to_owned(),
            })
            .expect_err("item cannot be paired with another module");
        let vc_error = registry
            .vc_id(VcIdentityInput {
                snapshot,
                module: second_module,
                item: Some(item),
                obligation_order_key: "vc/0001".to_owned(),
                canonical_vc_fingerprint: Some(hash(30)),
            })
            .expect_err("item cannot be paired with another module for a vc");

        assert_eq!(
            error,
            IdentityError::ItemModuleMismatch {
                module: second_module,
                item,
                item_module: first_module,
            }
        );
        assert_eq!(
            vc_error,
            IdentityError::ItemModuleMismatch {
                module: second_module,
                item,
                item_module: first_module,
            }
        );
    }

    #[test]
    fn derived_output_lineage_round_trips_in_canonical_order() {
        let snapshot = snapshot(5);
        let registry = registered_registry(snapshot);

        let parent_a = registry
            .register_output(output_input(snapshot, "a", Vec::new(), Vec::new()))
            .expect("parent a is registered");
        let parent_b = registry
            .register_output(output_input(snapshot, "b", Vec::new(), Vec::new()))
            .expect("parent b is registered");
        let child = registry
            .register_output(output_input(
                snapshot,
                "child",
                vec![parent_b.output, parent_a.output],
                vec![
                    NamedInputHash {
                        name: "z".to_owned(),
                        domain: "domain".to_owned(),
                        digest: hash(6),
                    },
                    NamedInputHash {
                        name: "a".to_owned(),
                        domain: "domain".to_owned(),
                        digest: hash(7),
                    },
                ],
            ))
            .expect("child output is registered");

        let round_trip = registry
            .output_lineage(child.output)
            .expect("lineage round-trips");
        assert_eq!(round_trip, child);
        assert_eq!(round_trip.parents.len(), 2);
        assert_eq!(round_trip.named_input_hashes[0].name, "a");
        assert_eq!(round_trip.named_input_hashes[1].name, "z");
    }

    #[test]
    fn ir_local_ids_are_not_proof_reuse_authority() {
        let old_snapshot = snapshot(6);
        let new_snapshot = snapshot(7);
        let registry = SnapshotHandleRegistry::new();
        registry.register_snapshot(old_snapshot);
        registry.register_snapshot(new_snapshot);

        let old_module = registry
            .module_id(module_input(old_snapshot))
            .expect("old module id is assigned");
        let new_module = registry
            .module_id(module_input(new_snapshot))
            .expect("new module id is assigned");
        let old_vc = registry
            .vc_id(VcIdentityInput {
                snapshot: old_snapshot,
                module: old_module,
                item: None,
                obligation_order_key: "same-source-shaped-obligation".to_owned(),
                canonical_vc_fingerprint: Some(hash(40)),
            })
            .expect("old vc id is assigned");
        let new_vc = registry
            .vc_id(VcIdentityInput {
                snapshot: new_snapshot,
                module: new_module,
                item: None,
                obligation_order_key: "same-source-shaped-obligation".to_owned(),
                canonical_vc_fingerprint: Some(hash(40)),
            })
            .expect("new vc id is assigned");

        assert_ne!(
            old_vc, new_vc,
            "snapshot-scoped VC ids must not be reused as cross-edit proof identity"
        );

        let old_output = registry
            .register_output(output_input(
                old_snapshot,
                "old-vc-output",
                Vec::new(),
                Vec::new(),
            ))
            .expect("old output is registered");
        assert!(matches!(
            registry.register_output(output_input(
                new_snapshot,
                "new-vc-output",
                vec![old_output.output],
                Vec::new(),
            )),
            Err(IdentityError::IncompatibleSnapshotParent { .. })
        ));
    }

    fn test_source_id(seed: u8) -> SourceId {
        let snapshot = snapshot(seed);
        let allocator = mizar_session::InMemorySessionIdAllocator::new();
        mizar_session::SessionIdAllocator::next_source_id(&allocator, snapshot)
            .expect("test source id is allocated")
    }
}
