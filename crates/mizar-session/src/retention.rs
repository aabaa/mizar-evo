//! Snapshot retention leases.

use crate::{
    BuildRequestId, BuildSnapshotId, IdError, InMemorySessionIdAllocator, SessionIdAllocator,
    SnapshotLease, SnapshotLeaseId, source_map::DocumentUri,
};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::sync::Mutex;

pub use crate::snapshot::RetentionReason;

#[derive(Debug)]
pub struct RetentionManager<A = InMemorySessionIdAllocator> {
    allocator: A,
    state: Mutex<RetentionState>,
}

#[derive(Debug, Default)]
struct RetentionState {
    snapshots: HashSet<BuildSnapshotId>,
    current_marks: HashSet<BuildSnapshotId>,
    leases: HashMap<SnapshotLeaseId, RetainedLease>,
    released_leases: HashSet<SnapshotLeaseId>,
    counts: HashMap<BuildSnapshotId, HashMap<RetentionCountKey, usize>>,
    resources: HashMap<BuildSnapshotId, RetainedSnapshotResources>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetainSnapshotInput {
    pub snapshot: BuildSnapshotId,
    pub owner: RetainOwner,
    pub reason: RetentionReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetainGuard {
    pub lease_id: SnapshotLeaseId,
    pub snapshot: BuildSnapshotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CollectionSummary {
    pub scanned: usize,
    pub collected: usize,
    pub released_sources: usize,
    pub released_maps: usize,
    pub skipped_current: usize,
    pub skipped_live_leases: usize,
    pub lease_diagnostics: Vec<RetentionLeaseDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RetentionLeaseDiagnostic {
    StaleLiveLease {
        lease_id: SnapshotLeaseId,
        snapshot: BuildSnapshotId,
    },
    StaleLeaseCount {
        snapshot: BuildSnapshotId,
        live_count: usize,
    },
    MismatchedLeaseCount {
        snapshot: BuildSnapshotId,
        expected_live_count: usize,
        actual_live_count: usize,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RetainedSnapshotResources {
    pub sources: usize,
    pub maps: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RetainOwner {
    Build(BuildRequestId),
    Watch,
    Lsp(DocumentUri),
    Diagnostics,
    Explanation,
    IrStorage,
    CacheWriter,
    ArtifactWriter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RetentionError {
    UnknownSnapshotId {
        snapshot_id: BuildSnapshotId,
    },
    UnknownOrReleasedLeaseId {
        lease_id: SnapshotLeaseId,
    },
    LeaseSnapshotMismatch {
        lease_id: SnapshotLeaseId,
        expected_snapshot: BuildSnapshotId,
        actual_snapshot: BuildSnapshotId,
    },
    InvalidOwnerReasonCombination {
        owner: RetainOwner,
        reason: RetentionReason,
    },
    AttemptToMarkMissingSnapshotCurrent {
        snapshot_id: BuildSnapshotId,
    },
    CollectionBlockedByInconsistentRetentionState {
        snapshot_id: BuildSnapshotId,
        detail: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RetainedLease {
    snapshot: BuildSnapshotId,
    owner: RetainOwner,
    reason: RetentionReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RetentionCountKey {
    owner: RetainOwner,
    reason: RetentionReason,
}

impl RetentionManager<InMemorySessionIdAllocator> {
    pub fn new() -> Self {
        Self::with_allocator(InMemorySessionIdAllocator::new())
    }
}

impl Default for RetentionManager<InMemorySessionIdAllocator> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A> RetentionManager<A> {
    pub fn with_allocator(allocator: A) -> Self {
        Self {
            allocator,
            state: Mutex::new(RetentionState::default()),
        }
    }

    pub fn register_snapshot(&self, snapshot: BuildSnapshotId) -> bool {
        self.state
            .lock()
            .expect("retention manager mutex poisoned")
            .snapshots
            .insert(snapshot)
    }

    pub fn record_retained_resources(
        &self,
        snapshot: BuildSnapshotId,
        resources: RetainedSnapshotResources,
    ) -> Result<(), RetentionError> {
        let mut state = self.state.lock().expect("retention manager mutex poisoned");
        state.validate_snapshot_exists(snapshot)?;
        state.resources.insert(snapshot, resources);
        Ok(())
    }

    pub fn mark_current(&self, snapshot: BuildSnapshotId) -> Result<bool, RetentionError> {
        let mut state = self.state.lock().expect("retention manager mutex poisoned");
        if !state.snapshots.contains(&snapshot) {
            return Err(RetentionError::AttemptToMarkMissingSnapshotCurrent {
                snapshot_id: snapshot,
            });
        }
        Ok(state.current_marks.insert(snapshot))
    }

    pub fn unmark_current(&self, snapshot: BuildSnapshotId) -> Result<bool, RetentionError> {
        let mut state = self.state.lock().expect("retention manager mutex poisoned");
        state.validate_snapshot_exists(snapshot)?;
        Ok(state.current_marks.remove(&snapshot))
    }

    pub fn collect(&self) -> CollectionSummary {
        let mut state = self.state.lock().expect("retention manager mutex poisoned");
        state.collect()
    }
}

impl<A: SessionIdAllocator> RetentionManager<A> {
    pub fn retain_snapshot(
        &self,
        input: RetainSnapshotInput,
    ) -> Result<RetainGuard, RetentionError> {
        let mut state = self.state.lock().expect("retention manager mutex poisoned");
        state.validate_snapshot_exists(input.snapshot)?;
        validate_owner_reason(&input.owner, input.reason)?;

        let snapshot = input.snapshot;
        for _ in 0..state.unavailable_lease_id_count().saturating_add(1) {
            let lease_id = self
                .allocator
                .next_lease_id(snapshot)
                .map_err(|error| retention_state_error(snapshot, error))?;
            if state.lease_id_is_available(lease_id) {
                state.record_lease(lease_id, input)?;
                return Ok(RetainGuard { lease_id, snapshot });
            }
        }

        Err(
            RetentionError::CollectionBlockedByInconsistentRetentionState {
                snapshot_id: snapshot,
                detail: "duplicate lease id allocation",
            },
        )
    }

    pub fn retain_existing_lease(
        &self,
        lease: SnapshotLease,
        owner: RetainOwner,
    ) -> Result<RetainGuard, RetentionError> {
        let mut state = self.state.lock().expect("retention manager mutex poisoned");
        state.validate_snapshot_exists(lease.snapshot)?;
        validate_owner_reason(&owner, lease.reason)?;

        let guard = RetainGuard {
            lease_id: lease.lease_id,
            snapshot: lease.snapshot,
        };
        state.record_lease(
            lease.lease_id,
            RetainSnapshotInput {
                snapshot: lease.snapshot,
                owner,
                reason: lease.reason,
            },
        )?;
        Ok(guard)
    }

    pub fn release(&self, guard: RetainGuard) -> Result<(), RetentionError> {
        let mut state = self.state.lock().expect("retention manager mutex poisoned");
        state.validate_snapshot_exists(guard.snapshot)?;

        let Some(lease) = state.leases.get(&guard.lease_id).cloned() else {
            return Err(RetentionError::UnknownOrReleasedLeaseId {
                lease_id: guard.lease_id,
            });
        };

        if lease.snapshot != guard.snapshot {
            return Err(RetentionError::LeaseSnapshotMismatch {
                lease_id: guard.lease_id,
                expected_snapshot: guard.snapshot,
                actual_snapshot: lease.snapshot,
            });
        }

        state.decrement_count(&lease)?;
        state.leases.remove(&guard.lease_id);
        state.released_leases.insert(guard.lease_id);
        Ok(())
    }
}

impl RetentionState {
    fn validate_snapshot_exists(&self, snapshot: BuildSnapshotId) -> Result<(), RetentionError> {
        if self.snapshots.contains(&snapshot) {
            Ok(())
        } else {
            Err(RetentionError::UnknownSnapshotId {
                snapshot_id: snapshot,
            })
        }
    }

    fn lease_id_is_available(&self, lease_id: SnapshotLeaseId) -> bool {
        !self.leases.contains_key(&lease_id) && !self.released_leases.contains(&lease_id)
    }

    fn unavailable_lease_id_count(&self) -> usize {
        self.leases.len().saturating_add(self.released_leases.len())
    }

    fn record_lease(
        &mut self,
        lease_id: SnapshotLeaseId,
        input: RetainSnapshotInput,
    ) -> Result<(), RetentionError> {
        if !self.lease_id_is_available(lease_id) {
            return Err(
                RetentionError::CollectionBlockedByInconsistentRetentionState {
                    snapshot_id: input.snapshot,
                    detail: "duplicate lease id allocation",
                },
            );
        }

        let lease = RetainedLease {
            snapshot: input.snapshot,
            owner: input.owner,
            reason: input.reason,
        };
        let count_key = RetentionCountKey::from_lease(&lease);
        *self
            .counts
            .entry(lease.snapshot)
            .or_default()
            .entry(count_key)
            .or_default() += 1;
        self.leases.insert(lease_id, lease);
        Ok(())
    }

    fn collect(&mut self) -> CollectionSummary {
        let actual_live_counts = self.actual_live_counts();
        let count_live_counts = self.count_live_counts();
        let mut summary = self.collection_diagnostics(&actual_live_counts, &count_live_counts);
        let snapshots = self.snapshots.iter().copied().collect::<Vec<_>>();

        for snapshot in snapshots {
            summary.scanned += 1;

            if self.current_marks.contains(&snapshot) {
                summary.skipped_current += 1;
                continue;
            }

            let actual_live_count = actual_live_counts
                .get(&snapshot)
                .copied()
                .unwrap_or_default();
            let count_live_count = count_live_counts
                .get(&snapshot)
                .copied()
                .unwrap_or_default();
            if actual_live_count > 0 || count_live_count > 0 {
                summary.skipped_live_leases += 1;
                continue;
            }

            if let Some(resources) = self.resources.remove(&snapshot) {
                summary.released_sources += resources.sources;
                summary.released_maps += resources.maps;
            }
            self.counts.remove(&snapshot);
            self.snapshots.remove(&snapshot);
            summary.collected += 1;
        }

        summary
    }

    fn actual_live_counts(&self) -> HashMap<BuildSnapshotId, usize> {
        let mut counts = HashMap::new();
        for lease in self.leases.values() {
            *counts.entry(lease.snapshot).or_default() += 1;
        }
        counts
    }

    fn count_live_counts(&self) -> HashMap<BuildSnapshotId, usize> {
        self.counts
            .iter()
            .map(|(snapshot, counts_by_key)| (*snapshot, counts_by_key.values().sum()))
            .collect()
    }

    fn collection_diagnostics(
        &self,
        actual_live_counts: &HashMap<BuildSnapshotId, usize>,
        count_live_counts: &HashMap<BuildSnapshotId, usize>,
    ) -> CollectionSummary {
        let mut summary = CollectionSummary::default();

        for (lease_id, lease) in &self.leases {
            if !self.snapshots.contains(&lease.snapshot) {
                summary
                    .lease_diagnostics
                    .push(RetentionLeaseDiagnostic::StaleLiveLease {
                        lease_id: *lease_id,
                        snapshot: lease.snapshot,
                    });
            }
        }

        for (snapshot, live_count) in count_live_counts {
            if !self.snapshots.contains(snapshot) {
                summary
                    .lease_diagnostics
                    .push(RetentionLeaseDiagnostic::StaleLeaseCount {
                        snapshot: *snapshot,
                        live_count: *live_count,
                    });
            }
        }

        let snapshots_with_lease_state = actual_live_counts
            .keys()
            .chain(count_live_counts.keys())
            .copied()
            .collect::<HashSet<_>>();
        for snapshot in snapshots_with_lease_state {
            let actual_live_count = actual_live_counts
                .get(&snapshot)
                .copied()
                .unwrap_or_default();
            let count_live_count = count_live_counts
                .get(&snapshot)
                .copied()
                .unwrap_or_default();
            if self.snapshots.contains(&snapshot) && actual_live_count != count_live_count {
                summary
                    .lease_diagnostics
                    .push(RetentionLeaseDiagnostic::MismatchedLeaseCount {
                        snapshot,
                        expected_live_count: count_live_count,
                        actual_live_count,
                    });
            }
        }

        summary
    }

    fn decrement_count(&mut self, lease: &RetainedLease) -> Result<(), RetentionError> {
        let count_key = RetentionCountKey::from_lease(lease);
        let Some(counts_by_key) = self.counts.get_mut(&lease.snapshot) else {
            return Err(inconsistent_release_error(lease.snapshot));
        };
        let Some(count) = counts_by_key.get_mut(&count_key) else {
            return Err(inconsistent_release_error(lease.snapshot));
        };
        let Some(decremented) = count.checked_sub(1) else {
            return Err(inconsistent_release_error(lease.snapshot));
        };

        *count = decremented;
        if *count == 0 {
            counts_by_key.remove(&count_key);
        }
        if counts_by_key.is_empty() {
            self.counts.remove(&lease.snapshot);
        }
        Ok(())
    }
}

impl RetentionCountKey {
    fn from_lease(lease: &RetainedLease) -> Self {
        Self {
            owner: lease.owner.clone(),
            reason: lease.reason,
        }
    }
}

fn validate_owner_reason(
    owner: &RetainOwner,
    reason: RetentionReason,
) -> Result<(), RetentionError> {
    let valid = matches!(
        (owner, reason),
        (RetainOwner::Build(_), RetentionReason::ActiveBuild)
            | (RetainOwner::Watch, RetentionReason::CurrentWatchBaseline)
            | (
                RetainOwner::Lsp(_),
                RetentionReason::PublishedLspSnapshot | RetentionReason::OpenBufferOverlay
            )
            | (RetainOwner::Diagnostics, RetentionReason::DiagnosticIndex)
            | (
                RetainOwner::Explanation,
                RetentionReason::ExplanationRequest
            )
            | (
                RetainOwner::IrStorage,
                RetentionReason::PhaseOutputReference
            )
            | (
                RetainOwner::CacheWriter | RetainOwner::ArtifactWriter,
                RetentionReason::PendingWrite
            )
    );

    if valid {
        Ok(())
    } else {
        Err(RetentionError::InvalidOwnerReasonCombination {
            owner: owner.clone(),
            reason,
        })
    }
}

fn retention_state_error(snapshot_id: BuildSnapshotId, error: IdError) -> RetentionError {
    match error {
        IdError::AllocatorOverflow => {
            RetentionError::CollectionBlockedByInconsistentRetentionState {
                snapshot_id,
                detail: "lease id allocation overflow",
            }
        }
        IdError::MalformedSerializedId
        | IdError::WrongIdDomain
        | IdError::UnknownSnapshotRegistry
        | IdError::NonPersistableSerialization => {
            RetentionError::CollectionBlockedByInconsistentRetentionState {
                snapshot_id,
                detail: "lease id allocation failed",
            }
        }
    }
}

fn inconsistent_release_error(snapshot_id: BuildSnapshotId) -> RetentionError {
    RetentionError::CollectionBlockedByInconsistentRetentionState {
        snapshot_id,
        detail: "live lease count missing during release",
    }
}

impl fmt::Display for RetentionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSnapshotId { snapshot_id } => {
                write!(f, "unknown snapshot id `{snapshot_id:?}`")
            }
            Self::UnknownOrReleasedLeaseId { lease_id } => {
                write!(
                    f,
                    "unknown or already-released retention lease `{lease_id:?}`"
                )
            }
            Self::LeaseSnapshotMismatch {
                lease_id,
                expected_snapshot,
                actual_snapshot,
            } => {
                write!(
                    f,
                    "retention lease `{lease_id:?}` belongs to `{actual_snapshot:?}`, not `{expected_snapshot:?}`"
                )
            }
            Self::InvalidOwnerReasonCombination { owner, reason } => {
                write!(
                    f,
                    "invalid retention owner/reason combination: `{owner:?}` with `{reason:?}`"
                )
            }
            Self::AttemptToMarkMissingSnapshotCurrent { snapshot_id } => {
                write!(
                    f,
                    "attempted to mark missing snapshot `{snapshot_id:?}` as current"
                )
            }
            Self::CollectionBlockedByInconsistentRetentionState {
                snapshot_id,
                detail,
            } => {
                write!(
                    f,
                    "collection blocked by inconsistent retention state for `{snapshot_id:?}`: {detail}"
                )
            }
        }
    }
}

impl Error for RetentionError {}

#[cfg(test)]
impl<A> RetentionManager<A> {
    fn live_lease_count_for_test(&self, snapshot: BuildSnapshotId) -> usize {
        self.state
            .lock()
            .expect("retention manager mutex poisoned")
            .counts
            .get(&snapshot)
            .map(|counts_by_key| counts_by_key.values().sum())
            .unwrap_or_default()
    }

    fn lease_count_for_test(
        &self,
        snapshot: BuildSnapshotId,
        owner: RetainOwner,
        reason: RetentionReason,
    ) -> usize {
        self.state
            .lock()
            .expect("retention manager mutex poisoned")
            .counts
            .get(&snapshot)
            .and_then(|counts_by_key| {
                counts_by_key
                    .get(&RetentionCountKey { owner, reason })
                    .copied()
            })
            .unwrap_or_default()
    }

    fn is_collection_eligible_for_test(
        &self,
        snapshot: BuildSnapshotId,
    ) -> Result<bool, RetentionError> {
        let state = self.state.lock().expect("retention manager mutex poisoned");
        state.validate_snapshot_exists(snapshot)?;
        let has_live_lease_count = state.counts.get(&snapshot).is_some_and(|counts_by_key| {
            !counts_by_key.is_empty() && counts_by_key.values().any(|count| *count > 0)
        });
        let has_live_lease = state
            .leases
            .values()
            .any(|lease| lease.snapshot == snapshot);
        Ok(!state.current_marks.contains(&snapshot) && !has_live_lease_count && !has_live_lease)
    }

    fn insert_stale_count_for_test(&self, snapshot: BuildSnapshotId, count: usize) {
        let mut state = self.state.lock().expect("retention manager mutex poisoned");
        state.counts.insert(
            snapshot,
            HashMap::from([(
                RetentionCountKey {
                    owner: RetainOwner::Diagnostics,
                    reason: RetentionReason::DiagnosticIndex,
                },
                count,
            )]),
        );
    }

    fn insert_stale_live_lease_for_test(
        &self,
        lease_id: SnapshotLeaseId,
        snapshot: BuildSnapshotId,
    ) {
        let mut state = self.state.lock().expect("retention manager mutex poisoned");
        state.leases.insert(
            lease_id,
            RetainedLease {
                snapshot,
                owner: RetainOwner::Diagnostics,
                reason: RetentionReason::DiagnosticIndex,
            },
        );
    }

    fn remove_counts_for_test(&self, snapshot: BuildSnapshotId) {
        self.state
            .lock()
            .expect("retention manager mutex poisoned")
            .counts
            .remove(&snapshot);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RetainGuard, RetainOwner, RetainSnapshotInput, RetainedSnapshotResources, RetentionError,
        RetentionLeaseDiagnostic, RetentionManager,
    };
    use crate::{
        BuildRequestId, BuildSessionId, BuildSnapshotId, DependencyArtifactRef, Edition, Hash,
        IdError, InMemorySessionIdAllocator, ModulePath, NormalizedPath, PackageId,
        RetentionReason, SessionIdAllocator, SnapshotInput, SnapshotLease, SnapshotLeaseId,
        SnapshotRegistry, SourceId, SourceMapId, SourceOrigin, SourceVersion, ToolchainInfo,
        WorkspaceRoot, normalize_source_path,
    };
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn retaining_unknown_snapshot_returns_unknown_snapshot_id() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(1);

        let error = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::Build(request_id()),
                RetentionReason::ActiveBuild,
            ))
            .unwrap_err();

        assert!(matches!(
            error,
            RetentionError::UnknownSnapshotId { snapshot_id } if snapshot_id == snapshot
        ));
        assert_eq!(manager.live_lease_count_for_test(snapshot), 0);
    }

    #[test]
    fn active_lease_prevents_collection_eligibility_until_released() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(2);
        manager.register_snapshot(snapshot);
        assert!(manager.is_collection_eligible_for_test(snapshot).unwrap());

        let owner = RetainOwner::Build(request_id());
        let guard = manager
            .retain_snapshot(retain_input(
                snapshot,
                owner.clone(),
                RetentionReason::ActiveBuild,
            ))
            .unwrap();

        assert!(!manager.is_collection_eligible_for_test(snapshot).unwrap());
        assert_eq!(
            manager.lease_count_for_test(snapshot, owner, RetentionReason::ActiveBuild),
            1
        );

        manager.release(guard).unwrap();

        assert!(manager.is_collection_eligible_for_test(snapshot).unwrap());
        assert_eq!(manager.live_lease_count_for_test(snapshot), 0);
    }

    #[test]
    fn current_mark_prevents_collection_without_other_leases() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(32);
        manager.register_snapshot(snapshot);
        manager
            .record_retained_resources(
                snapshot,
                RetainedSnapshotResources {
                    sources: 2,
                    maps: 3,
                },
            )
            .unwrap();

        assert!(!manager.unmark_current(snapshot).unwrap());
        assert!(manager.mark_current(snapshot).unwrap());
        assert!(!manager.mark_current(snapshot).unwrap());
        assert!(!manager.is_collection_eligible_for_test(snapshot).unwrap());

        let summary = manager.collect();

        assert_eq!(summary.scanned, 1);
        assert_eq!(summary.collected, 0);
        assert_eq!(summary.released_sources, 0);
        assert_eq!(summary.released_maps, 0);
        assert_eq!(summary.skipped_current, 1);
        assert_eq!(summary.skipped_live_leases, 0);
        assert!(summary.lease_diagnostics.is_empty());

        assert!(manager.unmark_current(snapshot).unwrap());
        assert!(manager.is_collection_eligible_for_test(snapshot).unwrap());

        let summary = manager.collect();

        assert_eq!(summary.scanned, 1);
        assert_eq!(summary.collected, 1);
        assert_eq!(summary.released_sources, 2);
        assert_eq!(summary.released_maps, 3);
        assert_eq!(summary.skipped_current, 0);
        assert_eq!(summary.skipped_live_leases, 0);
    }

    #[test]
    fn releasing_final_lease_allows_collection() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(33);
        manager.register_snapshot(snapshot);
        manager
            .record_retained_resources(
                snapshot,
                RetainedSnapshotResources {
                    sources: 1,
                    maps: 2,
                },
            )
            .unwrap();
        let guard = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::Diagnostics,
                RetentionReason::DiagnosticIndex,
            ))
            .unwrap();

        let summary = manager.collect();

        assert_eq!(summary.scanned, 1);
        assert_eq!(summary.collected, 0);
        assert_eq!(summary.skipped_live_leases, 1);

        manager.release(guard).unwrap();

        let summary = manager.collect();

        assert_eq!(summary.scanned, 1);
        assert_eq!(summary.collected, 1);
        assert_eq!(summary.released_sources, 1);
        assert_eq!(summary.released_maps, 2);
        assert!(matches!(
            manager
                .retain_snapshot(retain_input(
                    snapshot,
                    RetainOwner::Diagnostics,
                    RetentionReason::DiagnosticIndex,
                ))
                .unwrap_err(),
            RetentionError::UnknownSnapshotId { snapshot_id } if snapshot_id == snapshot
        ));
    }

    #[test]
    fn phase_output_reference_lease_blocks_collection_until_released() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(34);
        manager.register_snapshot(snapshot);
        let guard = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::IrStorage,
                RetentionReason::PhaseOutputReference,
            ))
            .unwrap();

        let summary = manager.collect();

        assert_eq!(summary.scanned, 1);
        assert_eq!(summary.collected, 0);
        assert_eq!(summary.skipped_live_leases, 1);

        manager.release(guard).unwrap();

        let summary = manager.collect();

        assert_eq!(summary.scanned, 1);
        assert_eq!(summary.collected, 1);
    }

    #[test]
    fn marking_missing_snapshot_current_returns_retention_error() {
        let manager = RetentionManager::new();
        let missing = snapshot_id(35);

        let error = manager.mark_current(missing).unwrap_err();

        assert!(matches!(
            error,
            RetentionError::AttemptToMarkMissingSnapshotCurrent { snapshot_id }
                if snapshot_id == missing
        ));
    }

    #[test]
    fn collection_summary_reports_skips_and_lease_diagnostics() {
        let manager = RetentionManager::new();
        let current = snapshot_id(36);
        let live = snapshot_id(37);
        let mismatched = snapshot_id(38);
        let stale = snapshot_id(39);
        let stale_live = snapshot_id(45);
        manager.register_snapshot(current);
        manager.register_snapshot(live);
        manager.register_snapshot(mismatched);
        manager.mark_current(current).unwrap();
        let _live_guard = manager
            .retain_snapshot(retain_input(
                live,
                RetainOwner::Diagnostics,
                RetentionReason::DiagnosticIndex,
            ))
            .unwrap();
        let _mismatched_guard = manager
            .retain_snapshot(retain_input(
                mismatched,
                RetainOwner::Explanation,
                RetentionReason::ExplanationRequest,
            ))
            .unwrap();
        manager.remove_counts_for_test(mismatched);
        manager.insert_stale_count_for_test(stale, 1);
        let lease_allocator = InMemorySessionIdAllocator::new();
        let stale_live_lease = (0..10)
            .map(|_| lease_allocator.next_lease_id(stale_live).unwrap())
            .last()
            .unwrap();
        manager.insert_stale_live_lease_for_test(stale_live_lease, stale_live);

        let summary = manager.collect();

        assert_eq!(summary.scanned, 3);
        assert_eq!(summary.collected, 0);
        assert_eq!(summary.skipped_current, 1);
        assert_eq!(summary.skipped_live_leases, 2);
        assert!(summary.lease_diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            RetentionLeaseDiagnostic::StaleLeaseCount {
                snapshot,
                live_count: 1,
            } if *snapshot == stale
        )));
        assert!(summary.lease_diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            RetentionLeaseDiagnostic::StaleLiveLease { lease_id, snapshot }
                if *lease_id == stale_live_lease && *snapshot == stale_live
        )));
        assert!(summary.lease_diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            RetentionLeaseDiagnostic::MismatchedLeaseCount {
                snapshot,
                expected_live_count: 0,
                actual_live_count: 1,
            } if *snapshot == mismatched
        )));
    }

    #[test]
    fn cache_and_artifact_leases_do_not_become_collection_outputs() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(44);
        manager.register_snapshot(snapshot);
        manager
            .record_retained_resources(
                snapshot,
                RetainedSnapshotResources {
                    sources: 4,
                    maps: 5,
                },
            )
            .unwrap();
        let cache = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::CacheWriter,
                RetentionReason::PendingWrite,
            ))
            .unwrap();
        let artifact = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::ArtifactWriter,
                RetentionReason::PendingWrite,
            ))
            .unwrap();

        let summary = manager.collect();

        assert_eq!(summary.collected, 0);
        assert_eq!(summary.skipped_live_leases, 1);
        assert_eq!(summary.released_sources, 0);
        assert_eq!(summary.released_maps, 0);

        manager.release(cache).unwrap();
        manager.release(artifact).unwrap();

        let summary = manager.collect();

        assert_eq!(summary.collected, 1);
        assert_eq!(summary.released_sources, 4);
        assert_eq!(summary.released_maps, 5);
    }

    #[test]
    fn snapshot_registry_active_build_lease_can_block_retention_collection_eligibility() {
        let registry = SnapshotRegistry::new();
        let request = request_id();
        let (snapshot, active_build) = registry
            .create_snapshot(request, snapshot_input(hash(20)))
            .unwrap();
        let manager = RetentionManager::new();
        manager.register_snapshot(snapshot.id);

        let guard = manager
            .retain_existing_lease(active_build.clone(), RetainOwner::Build(request))
            .unwrap();

        assert_eq!(guard.lease_id, active_build.lease_id);
        assert_eq!(guard.snapshot, snapshot.id);
        assert!(
            !manager
                .is_collection_eligible_for_test(snapshot.id)
                .unwrap()
        );
        assert_eq!(manager.live_lease_count_for_test(snapshot.id), 1);

        manager.release(guard).unwrap();
        registry
            .release_lease(snapshot.id, active_build.lease_id)
            .unwrap();

        assert!(
            manager
                .is_collection_eligible_for_test(snapshot.id)
                .unwrap()
        );
    }

    #[test]
    fn duplicate_release_is_reported_without_underflow() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(3);
        manager.register_snapshot(snapshot);
        let guard = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::Diagnostics,
                RetentionReason::DiagnosticIndex,
            ))
            .unwrap();

        manager.release(guard).unwrap();
        let error = manager.release(guard).unwrap_err();

        assert!(matches!(
            error,
            RetentionError::UnknownOrReleasedLeaseId { lease_id } if lease_id == guard.lease_id
        ));
        assert_eq!(manager.live_lease_count_for_test(snapshot), 0);
        assert!(manager.is_collection_eligible_for_test(snapshot).unwrap());
    }

    #[test]
    fn same_owner_reason_retain_reference_counts_until_final_release() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(30);
        manager.register_snapshot(snapshot);
        let owner = RetainOwner::Diagnostics;

        let first = manager
            .retain_snapshot(retain_input(
                snapshot,
                owner.clone(),
                RetentionReason::DiagnosticIndex,
            ))
            .unwrap();
        let second = manager
            .retain_snapshot(retain_input(
                snapshot,
                owner.clone(),
                RetentionReason::DiagnosticIndex,
            ))
            .unwrap();

        assert_ne!(first.lease_id, second.lease_id);
        assert_eq!(
            manager.lease_count_for_test(snapshot, owner.clone(), RetentionReason::DiagnosticIndex),
            2
        );
        assert_eq!(manager.live_lease_count_for_test(snapshot), 2);

        manager.release(first).unwrap();

        assert_eq!(
            manager.lease_count_for_test(snapshot, owner, RetentionReason::DiagnosticIndex),
            1
        );
        assert_eq!(manager.live_lease_count_for_test(snapshot), 1);
        assert!(!manager.is_collection_eligible_for_test(snapshot).unwrap());

        manager.release(second).unwrap();

        assert_eq!(manager.live_lease_count_for_test(snapshot), 0);
        assert!(manager.is_collection_eligible_for_test(snapshot).unwrap());
    }

    #[test]
    fn release_for_never_issued_lease_id_is_reported_without_changing_counts() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(31);
        manager.register_snapshot(snapshot);
        let never_issued = InMemorySessionIdAllocator::new()
            .next_lease_id(snapshot)
            .unwrap();

        let error = manager
            .release(RetainGuard {
                lease_id: never_issued,
                snapshot,
            })
            .unwrap_err();

        assert!(matches!(
            error,
            RetentionError::UnknownOrReleasedLeaseId { lease_id } if lease_id == never_issued
        ));
        assert_eq!(manager.live_lease_count_for_test(snapshot), 0);
        assert!(manager.is_collection_eligible_for_test(snapshot).unwrap());
    }

    #[test]
    fn invalid_owner_reason_combination_is_rejected() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(4);
        manager.register_snapshot(snapshot);

        let error = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::Diagnostics,
                RetentionReason::ActiveBuild,
            ))
            .unwrap_err();

        assert!(matches!(
            error,
            RetentionError::InvalidOwnerReasonCombination {
                owner: RetainOwner::Diagnostics,
                reason: RetentionReason::ActiveBuild
            }
        ));
        assert_eq!(manager.live_lease_count_for_test(snapshot), 0);
    }

    #[test]
    fn duplicate_allocated_lease_id_reports_inconsistent_retention_state() {
        let snapshot = snapshot_id(40);
        let duplicate_lease_id = InMemorySessionIdAllocator::new()
            .next_lease_id(snapshot)
            .unwrap();
        let manager =
            RetentionManager::with_allocator(DuplicateLeaseAllocator::new(duplicate_lease_id));
        manager.register_snapshot(snapshot);

        let first = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::Explanation,
                RetentionReason::ExplanationRequest,
            ))
            .unwrap();
        let error = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::Explanation,
                RetentionReason::ExplanationRequest,
            ))
            .unwrap_err();

        assert!(matches!(
            error,
            RetentionError::CollectionBlockedByInconsistentRetentionState {
                snapshot_id,
                detail: "duplicate lease id allocation",
            } if snapshot_id == snapshot
        ));
        assert_eq!(manager.live_lease_count_for_test(snapshot), 1);

        manager.release(first).unwrap();

        assert_eq!(manager.live_lease_count_for_test(snapshot), 0);
    }

    #[test]
    fn retain_snapshot_skips_all_bridged_released_lease_ids() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(43);
        let external_ids = InMemorySessionIdAllocator::new();
        manager.register_snapshot(snapshot);

        for _ in 0..20 {
            let lease = SnapshotLease {
                lease_id: external_ids.next_lease_id(snapshot).unwrap(),
                snapshot,
                reason: RetentionReason::DiagnosticIndex,
            };
            let guard = manager
                .retain_existing_lease(lease, RetainOwner::Diagnostics)
                .unwrap();
            manager.release(guard).unwrap();
        }

        let guard = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::Diagnostics,
                RetentionReason::DiagnosticIndex,
            ))
            .unwrap();

        assert_eq!(manager.live_lease_count_for_test(snapshot), 1);

        manager.release(guard).unwrap();

        assert_eq!(manager.live_lease_count_for_test(snapshot), 0);
    }

    #[test]
    fn lease_id_allocation_failure_reports_inconsistent_retention_state() {
        let manager = RetentionManager::with_allocator(LeaseFailingAllocator::new());
        let snapshot = snapshot_id(41);
        manager.register_snapshot(snapshot);

        let error = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::Build(request_id()),
                RetentionReason::ActiveBuild,
            ))
            .unwrap_err();

        assert!(matches!(
            error,
            RetentionError::CollectionBlockedByInconsistentRetentionState {
                snapshot_id,
                detail: "lease id allocation overflow",
            } if snapshot_id == snapshot
        ));
        assert_eq!(manager.live_lease_count_for_test(snapshot), 0);
    }

    #[test]
    fn release_with_missing_live_lease_count_reports_inconsistent_retention_state() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(46);
        manager.register_snapshot(snapshot);
        let guard = manager
            .retain_snapshot(retain_input(
                snapshot,
                RetainOwner::Diagnostics,
                RetentionReason::DiagnosticIndex,
            ))
            .unwrap();
        manager.remove_counts_for_test(snapshot);

        let error = manager.release(guard).unwrap_err();

        assert!(matches!(
            error,
            RetentionError::CollectionBlockedByInconsistentRetentionState {
                snapshot_id,
                detail: "live lease count missing during release",
            } if snapshot_id == snapshot
        ));
    }

    #[test]
    fn retention_error_reserved_variants_are_available_for_task_18() {
        let snapshot = snapshot_id(42);

        let missing_current = RetentionError::AttemptToMarkMissingSnapshotCurrent {
            snapshot_id: snapshot,
        };
        let inconsistent = RetentionError::CollectionBlockedByInconsistentRetentionState {
            snapshot_id: snapshot,
            detail: "test inconsistent state",
        };

        assert!(matches!(
            missing_current,
            RetentionError::AttemptToMarkMissingSnapshotCurrent { snapshot_id }
                if snapshot_id == snapshot
        ));
        assert!(matches!(
            inconsistent,
            RetentionError::CollectionBlockedByInconsistentRetentionState {
                snapshot_id,
                detail: "test inconsistent state",
            } if snapshot_id == snapshot
        ));
    }

    #[test]
    fn release_for_wrong_snapshot_reports_mismatch_without_changing_counts() {
        let manager = RetentionManager::new();
        let first = snapshot_id(5);
        let second = snapshot_id(6);
        manager.register_snapshot(first);
        manager.register_snapshot(second);
        let guard = manager
            .retain_snapshot(retain_input(
                first,
                RetainOwner::Explanation,
                RetentionReason::ExplanationRequest,
            ))
            .unwrap();

        let error = manager
            .release(RetainGuard {
                lease_id: guard.lease_id,
                snapshot: second,
            })
            .unwrap_err();

        assert!(matches!(
            error,
            RetentionError::LeaseSnapshotMismatch {
                lease_id,
                expected_snapshot,
                actual_snapshot,
            } if lease_id == guard.lease_id
                && expected_snapshot == second
                && actual_snapshot == first
        ));
        assert_eq!(manager.live_lease_count_for_test(first), 1);
        assert_eq!(manager.live_lease_count_for_test(second), 0);

        manager.release(guard).unwrap();

        assert_eq!(manager.live_lease_count_for_test(first), 0);
    }

    #[test]
    fn stale_snapshot_retains_for_stale_display_reasons_without_becoming_current() {
        let registry = SnapshotRegistry::new();
        let request = request_id();
        let (older, _) = registry
            .create_snapshot(request, snapshot_input(hash(10)))
            .unwrap();
        let (newer, _) = registry
            .create_snapshot(request, snapshot_input(hash(11)))
            .unwrap();
        let manager = RetentionManager::new();
        manager.register_snapshot(older.id);
        manager.register_snapshot(newer.id);

        for (owner, reason) in [
            (RetainOwner::Diagnostics, RetentionReason::DiagnosticIndex),
            (
                RetainOwner::Explanation,
                RetentionReason::ExplanationRequest,
            ),
            (
                RetainOwner::Lsp("file:///workspace/src/alpha.miz".to_owned()),
                RetentionReason::PublishedLspSnapshot,
            ),
            (
                RetainOwner::IrStorage,
                RetentionReason::PhaseOutputReference,
            ),
        ] {
            let guard = manager
                .retain_snapshot(retain_input(older.id, owner, reason))
                .unwrap();

            assert!(!registry.is_current_for_request(older.id, request));
            assert!(registry.is_current_for_request(newer.id, request));

            manager.release(guard).unwrap();
        }
    }

    #[test]
    fn valid_owner_reason_pairs_are_counted_independently() {
        let manager = RetentionManager::new();
        let snapshot = snapshot_id(7);
        manager.register_snapshot(snapshot);

        let inputs = [
            retain_input(
                snapshot,
                RetainOwner::Build(request_id()),
                RetentionReason::ActiveBuild,
            ),
            retain_input(
                snapshot,
                RetainOwner::Watch,
                RetentionReason::CurrentWatchBaseline,
            ),
            retain_input(
                snapshot,
                RetainOwner::Lsp("file:///workspace/src/alpha.miz".to_owned()),
                RetentionReason::OpenBufferOverlay,
            ),
            retain_input(
                snapshot,
                RetainOwner::CacheWriter,
                RetentionReason::PendingWrite,
            ),
            retain_input(
                snapshot,
                RetainOwner::ArtifactWriter,
                RetentionReason::PendingWrite,
            ),
        ];

        let guards = inputs
            .into_iter()
            .map(|input| manager.retain_snapshot(input).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(manager.live_lease_count_for_test(snapshot), 5);

        for guard in guards {
            manager.release(guard).unwrap();
        }

        assert_eq!(manager.live_lease_count_for_test(snapshot), 0);
    }

    fn retain_input(
        snapshot: BuildSnapshotId,
        owner: RetainOwner,
        reason: RetentionReason,
    ) -> RetainSnapshotInput {
        RetainSnapshotInput {
            snapshot,
            owner,
            reason,
        }
    }

    fn snapshot_input(source_hash: Hash) -> SnapshotInput {
        let snapshot_scope = snapshot_id(200);
        let allocator = InMemorySessionIdAllocator::new();
        SnapshotInput {
            workspace_root: WorkspaceRoot::new("workspace"),
            source_versions: vec![SourceVersion {
                source_id: allocator.next_source_id(snapshot_scope).unwrap(),
                package_id: PackageId::new("mml"),
                module_path: ModulePath::new("alpha"),
                normalized_path: normalized_path("src/alpha.miz"),
                source_hash,
                edition: Edition::new("2026"),
                origin: SourceOrigin::Disk,
            }],
            dependency_artifacts: vec![DependencyArtifactRef::new("kernel/base.vo", hash(91))],
            lockfile_hash: hash(92),
            toolchain: ToolchainInfo::new("mizar-2026.1"),
            verifier_config_hash: hash(93),
        }
    }

    fn request_id() -> crate::BuildRequestId {
        InMemorySessionIdAllocator::new().next_request_id().unwrap()
    }

    fn snapshot_id(first_byte: u8) -> BuildSnapshotId {
        let mut serialized = String::from("mizar-session-build-snapshot-v1:");
        serialized.push_str(&format!("{first_byte:02x}"));
        for _ in 1..Hash::BYTE_LEN {
            serialized.push_str("00");
        }
        BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
    }

    fn hash(first_byte: u8) -> Hash {
        let mut bytes = [0; Hash::BYTE_LEN];
        bytes[0] = first_byte;
        Hash::from_bytes(bytes)
    }

    #[derive(Debug)]
    struct DuplicateLeaseAllocator {
        inner: InMemorySessionIdAllocator,
        lease_id: SnapshotLeaseId,
    }

    impl DuplicateLeaseAllocator {
        fn new(lease_id: SnapshotLeaseId) -> Self {
            Self {
                inner: InMemorySessionIdAllocator::new(),
                lease_id,
            }
        }
    }

    impl SessionIdAllocator for DuplicateLeaseAllocator {
        fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
            self.inner.next_session_id()
        }

        fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
            self.inner.next_request_id()
        }

        fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
            self.inner.next_source_id(snapshot)
        }

        fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
            self.inner.next_source_map_id(snapshot)
        }

        fn next_lease_id(&self, _snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
            Ok(self.lease_id)
        }
    }

    #[derive(Debug)]
    struct LeaseFailingAllocator {
        inner: InMemorySessionIdAllocator,
    }

    impl LeaseFailingAllocator {
        fn new() -> Self {
            Self {
                inner: InMemorySessionIdAllocator::new(),
            }
        }
    }

    impl SessionIdAllocator for LeaseFailingAllocator {
        fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
            self.inner.next_session_id()
        }

        fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
            self.inner.next_request_id()
        }

        fn next_source_id(&self, snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
            self.inner.next_source_id(snapshot)
        }

        fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
            self.inner.next_source_map_id(snapshot)
        }

        fn next_lease_id(&self, _snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
            Err(IdError::AllocatorOverflow)
        }
    }

    fn normalized_path(path: &str) -> NormalizedPath {
        let fixture = SourcePathFixture::new();
        fixture.write(path, "");
        normalize_source_path(fixture.root(), Path::new(path)).unwrap()
    }

    struct SourcePathFixture {
        base: std::path::PathBuf,
        root: std::path::PathBuf,
    }

    impl SourcePathFixture {
        fn new() -> Self {
            let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            let base = std::env::temp_dir().join(format!(
                "mizar-session-retention-test-{}-{id}",
                std::process::id()
            ));
            let root = base.join("package");
            fs::create_dir_all(root.join("src")).unwrap();
            Self { base, root }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn write(&self, path: &str, contents: &str) {
            let path = self.root.join(path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, contents).unwrap();
        }
    }

    impl Drop for SourcePathFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.base);
        }
    }
}
