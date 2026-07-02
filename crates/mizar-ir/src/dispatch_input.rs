//! Phase input identities and sealed parent handles for scheduler dispatch.
//!
//! This module is specified in
//! [`dispatch_input.md`](../../../../doc/design/mizar-ir/en/dispatch_input.md).

use std::{collections::HashSet, error::Error, fmt};

use mizar_session::{BuildSnapshotId, Hash};

use crate::{
    identity::PhaseOutputId,
    publisher::{PhaseOutputPublisher, PublishError},
    storage::{AnyPhaseOutputRef, IrStorageService, StorageError},
};

/// Deterministic phase input identities consumed by registry query boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseInputIdentities {
    input_hash: Hash,
    dependency_hashes: Vec<Hash>,
    parent_output_hashes: Vec<Hash>,
}

/// Sealed parent output validated before scheduler-selected dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedParentOutputHandle {
    handle: AnyPhaseOutputRef,
}

/// Complete IR-owned input bundle for one scheduler-selected phase dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseDispatchInputBundle {
    snapshot: BuildSnapshotId,
    identities: PhaseInputIdentities,
    parent_outputs: Vec<SealedParentOutputHandle>,
}

/// Scheduler-selected task plus the dispatch snapshot.
#[derive(Debug, Clone, Copy)]
pub struct PhaseDispatchInputRequest<'a, Task: ?Sized> {
    task: &'a Task,
    snapshot: BuildSnapshotId,
}

/// Provider seam used by downstream front doors for their task type.
pub trait PhaseDispatchInputProvider<Task: ?Sized> {
    /// Returns the IR-owned dispatch bundle for a selected task.
    fn dispatch_input_for_task(
        &self,
        request: PhaseDispatchInputRequest<'_, Task>,
    ) -> Result<Option<PhaseDispatchInputBundle>, DispatchInputError>;
}

/// Errors reported while validating dispatch inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DispatchInputError {
    /// A returned dispatch bundle belongs to a different snapshot.
    DispatchSnapshotMismatch {
        /// Expected scheduler dispatch snapshot.
        expected: BuildSnapshotId,
        /// Bundle snapshot.
        actual: BuildSnapshotId,
    },
    /// A parent handle belongs to a different snapshot than the dispatch input.
    ParentSnapshotMismatch {
        /// Dispatch snapshot.
        snapshot: BuildSnapshotId,
        /// Parent output id.
        parent: PhaseOutputId,
        /// Parent snapshot.
        parent_snapshot: BuildSnapshotId,
    },
    /// The same parent output was supplied more than once.
    DuplicateParentOutput {
        /// Duplicate parent output id.
        parent: PhaseOutputId,
    },
    /// Publisher currentness validation failed.
    Publisher {
        /// Publisher error.
        error: Box<PublishError>,
    },
    /// Storage sealing validation failed.
    Storage {
        /// Storage error.
        error: Box<StorageError>,
    },
}

impl PhaseInputIdentities {
    /// Creates identities for a dispatch with no parent output handles.
    pub fn without_parent_outputs(input_hash: Hash, dependency_hashes: Vec<Hash>) -> Self {
        Self::from_canonical_parts(input_hash, dependency_hashes, Vec::new())
    }

    fn from_parent_outputs(
        input_hash: Hash,
        dependency_hashes: Vec<Hash>,
        parent_outputs: &[SealedParentOutputHandle],
    ) -> Self {
        let parent_hashes = parent_outputs
            .iter()
            .map(SealedParentOutputHandle::identity_hash)
            .collect::<Vec<_>>();
        Self::from_canonical_parts(input_hash, dependency_hashes, parent_hashes)
    }

    fn from_canonical_parts(
        input_hash: Hash,
        mut dependency_hashes: Vec<Hash>,
        mut parent_output_hashes: Vec<Hash>,
    ) -> Self {
        dependency_hashes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        parent_output_hashes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        Self {
            input_hash,
            dependency_hashes,
            parent_output_hashes,
        }
    }

    /// Returns the owner-supplied non-parent phase input hash.
    pub const fn input_hash(&self) -> Hash {
        self.input_hash
    }

    /// Returns canonical non-output dependency hashes.
    pub fn dependency_hashes(&self) -> &[Hash] {
        &self.dependency_hashes
    }

    /// Returns canonical parent output identity hashes.
    pub fn parent_output_hashes(&self) -> &[Hash] {
        &self.parent_output_hashes
    }
}

impl<'a, Task: ?Sized> PhaseDispatchInputRequest<'a, Task> {
    /// Creates a dispatch input request.
    pub const fn new(task: &'a Task, snapshot: BuildSnapshotId) -> Self {
        Self { task, snapshot }
    }

    /// Returns the scheduler-selected downstream task.
    pub const fn task(&self) -> &'a Task {
        self.task
    }

    /// Returns the dispatch snapshot.
    pub const fn snapshot(&self) -> BuildSnapshotId {
        self.snapshot
    }
}

impl SealedParentOutputHandle {
    /// Validates a sealed current/package output for dispatch.
    pub fn from_current_output(
        publisher: &PhaseOutputPublisher,
        snapshot: BuildSnapshotId,
        handle: AnyPhaseOutputRef,
    ) -> Result<Self, DispatchInputError> {
        reject_parent_snapshot(snapshot, &handle)?;
        publisher
            .storage()
            .validate_handle(&handle)
            .map_err(|error| DispatchInputError::Storage {
                error: Box::new(error),
            })?;
        publisher
            .validate_current_output(snapshot, &handle)
            .map_err(|error| DispatchInputError::Publisher {
                error: Box::new(error),
            })?;
        Ok(Self { handle })
    }

    /// Validates a sealed handle already rehydrated into the dispatch snapshot.
    pub fn from_validated_rehydrated_output(
        storage: &IrStorageService,
        snapshot: BuildSnapshotId,
        handle: AnyPhaseOutputRef,
    ) -> Result<Self, DispatchInputError> {
        reject_parent_snapshot(snapshot, &handle)?;
        storage
            .validate_handle(&handle)
            .map_err(|error| DispatchInputError::Storage {
                error: Box::new(error),
            })?;
        Ok(Self { handle })
    }

    /// Returns the erased sealed output handle.
    pub const fn as_output_ref(&self) -> &AnyPhaseOutputRef {
        &self.handle
    }

    /// Consumes this wrapper and returns the erased sealed output handle.
    pub fn into_output_ref(self) -> AnyPhaseOutputRef {
        self.handle
    }

    /// Returns the parent output id.
    pub const fn output(&self) -> PhaseOutputId {
        self.handle.output()
    }

    /// Returns the parent output snapshot.
    pub const fn snapshot(&self) -> BuildSnapshotId {
        self.handle.snapshot()
    }

    /// Returns the identity hash used in `PhaseInputIdentities`.
    pub const fn identity_hash(&self) -> Hash {
        self.output().hash()
    }
}

impl PhaseDispatchInputBundle {
    /// Creates a dispatch bundle and derives parent identity hashes from
    /// validated sealed parent handles.
    pub fn new(
        snapshot: BuildSnapshotId,
        input_hash: Hash,
        dependency_hashes: Vec<Hash>,
        mut parent_outputs: Vec<SealedParentOutputHandle>,
    ) -> Result<Self, DispatchInputError> {
        reject_bundle_parent_snapshots(snapshot, &parent_outputs)?;
        parent_outputs.sort_by(|left, right| {
            left.identity_hash()
                .as_bytes()
                .cmp(right.identity_hash().as_bytes())
        });
        reject_duplicate_parents(&parent_outputs)?;
        let identities = PhaseInputIdentities::from_parent_outputs(
            input_hash,
            dependency_hashes,
            &parent_outputs,
        );
        Ok(Self {
            snapshot,
            identities,
            parent_outputs,
        })
    }

    /// Creates a dispatch bundle with no parent output handles.
    pub fn without_parent_outputs(
        snapshot: BuildSnapshotId,
        input_hash: Hash,
        dependency_hashes: Vec<Hash>,
    ) -> Self {
        Self {
            snapshot,
            identities: PhaseInputIdentities::without_parent_outputs(input_hash, dependency_hashes),
            parent_outputs: Vec::new(),
        }
    }

    /// Returns the dispatch snapshot that all parent handles were validated against.
    pub const fn snapshot(&self) -> BuildSnapshotId {
        self.snapshot
    }

    /// Validates this bundle against a scheduler-selected dispatch snapshot.
    pub fn validate_snapshot(&self, expected: BuildSnapshotId) -> Result<(), DispatchInputError> {
        if self.snapshot == expected {
            Ok(())
        } else {
            Err(DispatchInputError::DispatchSnapshotMismatch {
                expected,
                actual: self.snapshot,
            })
        }
    }

    /// Returns deterministic phase input identities.
    pub const fn identities(&self) -> &PhaseInputIdentities {
        &self.identities
    }

    /// Returns validated sealed parent output handles.
    pub fn parent_outputs(&self) -> &[SealedParentOutputHandle] {
        &self.parent_outputs
    }

    /// Splits this bundle into identities and parent handles.
    pub fn into_parts(
        self,
    ) -> (
        BuildSnapshotId,
        PhaseInputIdentities,
        Vec<SealedParentOutputHandle>,
    ) {
        (self.snapshot, self.identities, self.parent_outputs)
    }
}

impl fmt::Display for DispatchInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DispatchSnapshotMismatch { expected, actual } => write!(
                formatter,
                "dispatch input bundle for `{actual:?}` cannot be used for scheduler snapshot `{expected:?}`"
            ),
            Self::ParentSnapshotMismatch {
                snapshot,
                parent,
                parent_snapshot,
            } => write!(
                formatter,
                "parent output `{parent:?}` from `{parent_snapshot:?}` cannot be used for dispatch snapshot `{snapshot:?}`"
            ),
            Self::DuplicateParentOutput { parent } => {
                write!(formatter, "duplicate parent output `{parent:?}`")
            }
            Self::Publisher { error } => write!(formatter, "publisher validation failed: {error}"),
            Self::Storage { error } => write!(formatter, "storage validation failed: {error}"),
        }
    }
}

impl Error for DispatchInputError {}

fn reject_parent_snapshot(
    snapshot: BuildSnapshotId,
    handle: &AnyPhaseOutputRef,
) -> Result<(), DispatchInputError> {
    if handle.snapshot() == snapshot {
        Ok(())
    } else {
        Err(DispatchInputError::ParentSnapshotMismatch {
            snapshot,
            parent: handle.output(),
            parent_snapshot: handle.snapshot(),
        })
    }
}

fn reject_bundle_parent_snapshots(
    snapshot: BuildSnapshotId,
    parent_outputs: &[SealedParentOutputHandle],
) -> Result<(), DispatchInputError> {
    for parent in parent_outputs {
        if parent.snapshot() != snapshot {
            return Err(DispatchInputError::ParentSnapshotMismatch {
                snapshot,
                parent: parent.output(),
                parent_snapshot: parent.snapshot(),
            });
        }
    }
    Ok(())
}

fn reject_duplicate_parents(
    parent_outputs: &[SealedParentOutputHandle],
) -> Result<(), DispatchInputError> {
    let mut seen = HashSet::new();
    for parent in parent_outputs {
        if !seen.insert(parent.output()) {
            return Err(DispatchInputError::DuplicateParentOutput {
                parent: parent.output(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        identity::{NamedInputHash, OutputKind, PipelinePhase, SnapshotHandleRegistry, WorkUnit},
        publisher::{AllowedWorkUnit, OutputOrigin, PublicationTarget, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, IrSideTables, IrStorageService, SchemaVersion},
    };

    use super::*;

    #[test]
    fn identities_canonicalize_dependencies_and_parent_handles() {
        let snapshot = snapshot(1);
        let publisher = publisher(snapshot, &["parent-b", "parent-a"]);
        let parent_b = parent(&publisher, snapshot, "parent-b", "bbb");
        let parent_a = parent(&publisher, snapshot, "parent-a", "aaa");

        let bundle = PhaseDispatchInputBundle::new(
            snapshot,
            hash(10),
            vec![hash(3), hash(1), hash(2)],
            vec![parent_b.clone(), parent_a.clone()],
        )
        .expect("bundle is valid");

        assert_eq!(bundle.snapshot(), snapshot);
        assert_eq!(bundle.identities().input_hash(), hash(10));
        assert_eq!(
            bundle.identities().dependency_hashes(),
            &[hash(1), hash(2), hash(3)]
        );
        assert_eq!(
            bundle.identities().parent_output_hashes(),
            &canonical_parent_hashes(&[parent_a.clone(), parent_b.clone()])
        );
        assert_eq!(
            bundle
                .parent_outputs()
                .iter()
                .map(SealedParentOutputHandle::output)
                .collect::<Vec<_>>(),
            canonical_parent_outputs(&[parent_a, parent_b])
        );
    }

    #[test]
    fn duplicate_parent_handles_are_rejected() {
        let snapshot = snapshot(2);
        let publisher = publisher(snapshot, &["parent"]);
        let parent = parent(&publisher, snapshot, "parent", "payload");

        let error = PhaseDispatchInputBundle::new(
            snapshot,
            hash(20),
            Vec::new(),
            vec![parent.clone(), parent.clone()],
        )
        .expect_err("duplicate parent fails");

        assert_eq!(
            error,
            DispatchInputError::DuplicateParentOutput {
                parent: parent.output()
            }
        );
    }

    #[test]
    fn bundle_rejects_parent_from_different_snapshot() {
        let bundle_snapshot = snapshot(7);
        let parent_snapshot = snapshot(8);
        let publisher = publisher(parent_snapshot, &["parent"]);
        let parent = parent(&publisher, parent_snapshot, "parent", "payload");

        let error =
            PhaseDispatchInputBundle::new(bundle_snapshot, hash(30), Vec::new(), vec![parent])
                .expect_err("foreign-snapshot parent fails bundle validation");

        assert!(matches!(
            error,
            DispatchInputError::ParentSnapshotMismatch {
                snapshot,
                parent_snapshot: actual_parent_snapshot,
                ..
            } if snapshot == bundle_snapshot && actual_parent_snapshot == parent_snapshot
        ));
    }

    #[test]
    fn bundle_snapshot_validation_rejects_scheduler_mismatch() {
        let bundle_snapshot = snapshot(9);
        let scheduler_snapshot = snapshot(10);
        let bundle =
            PhaseDispatchInputBundle::without_parent_outputs(bundle_snapshot, hash(40), Vec::new());

        let error = bundle
            .validate_snapshot(scheduler_snapshot)
            .expect_err("wrong scheduler snapshot fails");

        assert_eq!(
            error,
            DispatchInputError::DispatchSnapshotMismatch {
                expected: scheduler_snapshot,
                actual: bundle_snapshot,
            }
        );
    }

    #[test]
    fn wrong_snapshot_parent_is_rejected_before_currentness() {
        let first_snapshot = snapshot(3);
        let second_snapshot = snapshot(4);
        let publisher = publisher(second_snapshot, &["parent"]);
        publisher.register_current_snapshot(first_snapshot);
        let parent = parent(&publisher, second_snapshot, "parent", "payload");

        let error = SealedParentOutputHandle::from_current_output(
            &publisher,
            first_snapshot,
            parent.into_output_ref(),
        )
        .expect_err("wrong snapshot fails");

        assert!(matches!(
            error,
            DispatchInputError::ParentSnapshotMismatch {
                snapshot,
                parent_snapshot,
                ..
            } if snapshot == first_snapshot && parent_snapshot == second_snapshot
        ));
    }

    #[test]
    fn obsolete_current_parent_is_rejected() {
        let snapshot = snapshot(5);
        let publisher = publisher(snapshot, &["parent"]);
        let parent = parent(&publisher, snapshot, "parent", "payload");
        publisher
            .mark_obsolete(snapshot)
            .expect("snapshot can be made obsolete");

        let error = SealedParentOutputHandle::from_current_output(
            &publisher,
            snapshot,
            parent.into_output_ref(),
        )
        .expect_err("obsolete current parent fails");

        assert!(matches!(error, DispatchInputError::Publisher { .. }));
    }

    #[test]
    fn current_parent_from_foreign_storage_is_rejected() {
        let snapshot = snapshot(6);
        let owner_publisher = publisher(snapshot, &["parent"]);
        let foreign_publisher = publisher(snapshot, &[]);
        let parent = parent(&owner_publisher, snapshot, "parent", "payload");

        let error = SealedParentOutputHandle::from_current_output(
            &foreign_publisher,
            snapshot,
            parent.into_output_ref(),
        )
        .expect_err("foreign-storage current parent fails");

        assert!(matches!(error, DispatchInputError::Storage { .. }));
    }

    #[test]
    fn validated_rehydrated_handle_requires_only_sealed_current_snapshot_handle() {
        let snapshot = snapshot(7);
        let publisher = publisher(snapshot, &["parent"]);
        let parent = parent(&publisher, snapshot, "parent", "payload");
        publisher
            .mark_obsolete(snapshot)
            .expect("snapshot can be made obsolete");

        let rehydrated = SealedParentOutputHandle::from_validated_rehydrated_output(
            publisher.storage(),
            snapshot,
            parent.as_output_ref().clone(),
        )
        .expect("already rehydrated handle is sealed in the dispatch snapshot");

        assert_eq!(rehydrated.output(), parent.output());
    }

    #[test]
    fn validated_rehydrated_handle_rejects_wrong_snapshot() {
        let parent_snapshot = snapshot(8);
        let dispatch_snapshot = snapshot(9);
        let publisher = publisher(parent_snapshot, &["parent"]);
        let parent = parent(&publisher, parent_snapshot, "parent", "payload");

        let error = SealedParentOutputHandle::from_validated_rehydrated_output(
            publisher.storage(),
            dispatch_snapshot,
            parent.as_output_ref().clone(),
        )
        .expect_err("wrong-snapshot rehydrated parent fails");

        assert!(matches!(
            error,
            DispatchInputError::ParentSnapshotMismatch {
                snapshot,
                parent_snapshot: actual_parent_snapshot,
                ..
            } if snapshot == dispatch_snapshot && actual_parent_snapshot == parent_snapshot
        ));
    }

    #[test]
    fn validated_rehydrated_handle_from_foreign_storage_is_rejected() {
        let snapshot = snapshot(10);
        let publisher = publisher(snapshot, &["parent"]);
        let foreign_storage = IrStorageService::new();
        let parent = parent(&publisher, snapshot, "parent", "payload");

        let error = SealedParentOutputHandle::from_validated_rehydrated_output(
            &foreign_storage,
            snapshot,
            parent.as_output_ref().clone(),
        )
        .expect_err("foreign-storage rehydrated parent fails");

        assert!(matches!(error, DispatchInputError::Storage { .. }));
    }

    fn publisher(snapshot: BuildSnapshotId, work_units: &[&str]) -> PhaseOutputPublisher {
        let publisher = PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        );
        publisher.register_current_snapshot(snapshot);
        for work_unit in work_units {
            publisher.allow_work_unit(AllowedWorkUnit::new(
                phase(),
                output_kind(),
                WorkUnit::new(*work_unit),
            ));
        }
        publisher
    }

    fn parent(
        publisher: &PhaseOutputPublisher,
        snapshot: BuildSnapshotId,
        work_unit: &str,
        payload: &str,
    ) -> SealedParentOutputHandle {
        let handle = publisher
            .publish(PublishOutputInput {
                slot: publisher.allocate(
                    snapshot,
                    phase(),
                    WorkUnit::new(work_unit),
                    output_kind(),
                    SchemaVersion::new(1),
                ),
                snapshot,
                phase: phase(),
                work_unit: WorkUnit::new(work_unit),
                output_kind: output_kind(),
                schema_version: SchemaVersion::new(1),
                payload: payload.to_owned(),
                canonical_payload: Some(payload.as_bytes().to_vec()),
                decode: string_decoder(),
                parents: Vec::new(),
                named_input_hashes: vec![NamedInputHash {
                    name: "source".to_owned(),
                    domain: "dispatch-input-test".to_owned(),
                    digest: hash(7),
                }],
                side_tables: IrSideTables::default(),
                origin: OutputOrigin::PackageSource,
                target: PublicationTarget::CurrentPackage,
            })
            .expect("parent publishes");
        SealedParentOutputHandle::from_current_output(publisher, snapshot, handle.erase())
            .expect("parent is current")
    }

    fn canonical_parent_hashes(parents: &[SealedParentOutputHandle]) -> Vec<Hash> {
        let mut values = parents
            .iter()
            .map(SealedParentOutputHandle::identity_hash)
            .collect::<Vec<_>>();
        values.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        values
    }

    fn canonical_parent_outputs(parents: &[SealedParentOutputHandle]) -> Vec<PhaseOutputId> {
        let mut values = parents
            .iter()
            .map(SealedParentOutputHandle::output)
            .collect::<Vec<_>>();
        values.sort_by(|left, right| left.hash().as_bytes().cmp(right.hash().as_bytes()));
        values
    }

    fn phase() -> PipelinePhase {
        PipelinePhase::new("dispatch-input-test")
    }

    fn output_kind() -> OutputKind {
        OutputKind::new("dispatch-input-parent")
    }

    fn string_decoder() -> BlobDecoder<String> {
        BlobDecoder::new(|bytes| {
            String::from_utf8(bytes.to_vec())
                .map_err(|error| BlobDecodeError::new(error.to_string()))
        })
    }

    fn snapshot(seed: u8) -> BuildSnapshotId {
        let serialized = format!(
            "mizar-session-build-snapshot-v1:{}",
            format!("{seed:02x}").repeat(Hash::BYTE_LEN)
        );
        BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
    }

    fn hash(seed: u8) -> Hash {
        Hash::from_bytes([seed; Hash::BYTE_LEN])
    }
}
