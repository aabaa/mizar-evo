# Module: retention

> Canonical language: English. Japanese companion: [../ja/retention.md](../ja/retention.md).

## Purpose

This module defines retention leases and collection policy for `mizar-session`.

It keeps source text, source maps, and snapshot metadata alive while batch, watch, LSP, diagnostics, explanation, cache, artifact, or IR consumers still reference them. It does not retain typed IR outputs directly; `mizar-ir` owns IR output retention and may hold snapshot leases while its handles remain live.

## Public API

```rust
pub struct RetentionManager<A = InMemorySessionIdAllocator> { /* private fields */ }

pub struct RetainSnapshotInput {
    pub snapshot: BuildSnapshotId,
    pub owner: RetainOwner,
    pub reason: RetentionReason,
}

pub struct RetainGuard {
    pub lease_id: SnapshotLeaseId,
    pub snapshot: BuildSnapshotId,
}

pub struct CollectionSummary {
    pub scanned: usize,
    pub collected: usize,
    pub released_sources: usize,
    pub released_maps: usize,
    pub skipped_current: usize,
    pub skipped_live_leases: usize,
    pub lease_diagnostics: Vec<RetentionLeaseDiagnostic>,
}

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

pub struct RetainedSnapshotResources {
    pub sources: usize,
    pub maps: usize,
}

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

// `RetentionReason` is owned by the snapshot/shared lease layer (see snapshot.md)
// and re-exported here, because `SnapshotLease.reason` needs it before this module
// is implemented. Variants:
//   ActiveBuild, CurrentWatchBaseline, PublishedLspSnapshot, OpenBufferOverlay,
//   DiagnosticIndex, ExplanationRequest, PhaseOutputReference, PendingWrite
pub use crate::snapshot::RetentionReason;

impl RetentionManager<InMemorySessionIdAllocator> {
    pub fn new() -> Self;
}

impl<A> RetentionManager<A> {
    pub fn with_allocator(allocator: A) -> Self;
    pub fn register_snapshot(&self, snapshot: BuildSnapshotId) -> bool;
    pub fn record_retained_resources(
        &self,
        snapshot: BuildSnapshotId,
        resources: RetainedSnapshotResources,
    ) -> Result<(), RetentionError>;
    pub fn mark_current(&self, snapshot: BuildSnapshotId) -> Result<bool, RetentionError>;
    pub fn unmark_current(&self, snapshot: BuildSnapshotId) -> Result<bool, RetentionError>;
    pub fn collect(&self) -> CollectionSummary;
}

impl<A: SessionIdAllocator> RetentionManager<A> {
    pub fn retain_snapshot(&self, input: RetainSnapshotInput) -> Result<RetainGuard, RetentionError>;
    pub fn retain_existing_lease(&self, lease: SnapshotLease, owner: RetainOwner) -> Result<RetainGuard, RetentionError>;
    pub fn release(&self, guard: RetainGuard) -> Result<(), RetentionError>;
}
```

Task 17 implements registration of known snapshots plus lease retain/release
accounting. `register_snapshot` records that a snapshot produced elsewhere is
known to the retention manager; it does not mark the snapshot current.
`retain_existing_lease` bridges a `SnapshotLease` that was already allocated by
the snapshot registry, such as the active-build lease returned by
`SnapshotRegistry::create_snapshot`, into the retention ledger without
allocating another lease id.
`RetainGuard` release should be idempotent from the caller's perspective, but
duplicate release attempts are reported with `RetentionError` and do not
underflow reference counts.

Task 18 adds current marks and collection. `mark_current` and `unmark_current`
return whether the mark set changed. Marking a missing snapshot current returns
`RetentionError::AttemptToMarkMissingSnapshotCurrent`. `collect` scans only
retention-manager in-memory state and returns a `CollectionSummary`; it does not
delete published artifacts or cache records.
`record_retained_resources` records the snapshot-owned in-memory source/map
footprint that will be released when the snapshot itself is collected; external
diagnostic, explanation, LSP, and IR references to those resources are still
represented by live leases.

## Dependencies

- Internal: `ids`, `snapshot`, `source`, `source_map`
- External: weak-reference or arena storage utilities, tracing/logging

This module is used by snapshot registry, LSP snapshot publication, diagnostic aggregation, explanation queries, cache/artifact writers, and `mizar-ir`.

## Data Structures

### Retention Record

Each retained snapshot has a record containing:

- snapshot id;
- reference counts by owner and reason;
- current request generations naming it;
- retained loaded sources;
- retained line maps and preprocessing maps;
- collection eligibility metadata;
- optional debug creation/release traces.

The record tracks session-local retention state. It is not serialized into published artifacts.
The retained source/map resource counts are collection accounting metadata, not
a separate external keep-alive; consumers that still need those resources hold a
lease with the appropriate `RetentionReason`.

### Current Marks

A current mark means a build request generation may report the snapshot as current. It is separate from a lease: a current mark prevents collection and controls freshness, while a lease only prevents collection.

Old snapshots may retain leases for stale diagnostics or explanation requests after a newer snapshot becomes current.

Retain/release never creates or updates current marks. Retaining an old snapshot
for `DiagnosticIndex`, `ExplanationRequest`, `PublishedLspSnapshot`, or
`PhaseOutputReference` keeps the snapshot alive for that consumer without making
it current. A current mark can be removed independently of any live lease.

### Collection Summary

`CollectionSummary` reports:

- snapshots scanned;
- snapshots collected;
- sources and maps released;
- snapshots skipped because of current marks;
- snapshots skipped because of live leases;
- stale or mismatched lease diagnostics.

It is intended for logging and tests, not for build semantics.

## Algorithm / Logic

### Retain

For `retain_snapshot`:

1. Validate that the snapshot exists.
2. Allocate a `SnapshotLeaseId`, skipping ids already known to the retention
   manager.
3. Increment the owner/reason count.
4. Return a `RetainGuard`.

For `retain_existing_lease`:

1. Validate that the snapshot exists.
2. Validate the provided `SnapshotLease` owner/reason pairing.
3. Record the existing lease id without allocating a duplicate id.
4. Increment the owner/reason count and return a `RetainGuard`.

Retaining a stale snapshot is allowed when the reason is diagnostic, explanation, LSP stale-artifact display, or IR output retention. It must not make the snapshot current.

### Release

1. Validate the lease id and snapshot id.
2. Decrement the owner/reason count.
3. Mark the lease as released.
4. If no lease remains, the retention record is no longer blocked by live
   leases and can be collected once no current mark remains.

Release must not synchronously delete data that another thread could still read through an active guard.

### Collection

The collector may remove a snapshot when:

- no live lease references it;
- no current mark references it;
- no source-map, diagnostic-explanation, LSP-publication, or IR phase-output
  reference lease remains live for it.

Collection drops in-memory source text, source maps, and snapshot metadata. It does not delete published artifacts or cache records.

`PhaseOutputReference` is represented as a normal live lease owned by
`IrStorage`, so it blocks collection until released. `PendingWrite` leases for
cache and artifact writers also block while live, but collection itself has no
artifact/cache deletion output. Stale live leases or mismatches between the live
lease map and reference-count ledger are recorded in `CollectionSummary` and
make collection conservative for affected registered snapshots.

## Error Handling

`RetentionError` includes:

- unknown snapshot id;
- unknown or already-released lease id;
- lease snapshot mismatch;
- invalid owner/reason combination;
- attempt to mark a missing snapshot as current;
- collection blocked by inconsistent retention state.

Valid owner/reason pairs are:

- `Build(_)` with `ActiveBuild`;
- `Watch` with `CurrentWatchBaseline`;
- `Lsp(_)` with `PublishedLspSnapshot` or `OpenBufferOverlay`;
- `Diagnostics` with `DiagnosticIndex`;
- `Explanation` with `ExplanationRequest`;
- `IrStorage` with `PhaseOutputReference`;
- `CacheWriter` or `ArtifactWriter` with `PendingWrite`.

Invalid retention state is a compiler internal error. User-facing builds should continue using the previous coherent snapshot when possible.

## Tests

Key scenarios:

- active build lease prevents collection;
- current mark prevents collection even without other leases;
- stale LSP or diagnostic lease retains old source maps without marking the snapshot current;
- releasing the final lease makes the snapshot collectible;
- `mizar-ir` phase-output lease blocks snapshot collection until released;
- duplicate release is reported but does not underflow counts;
- collection does not delete artifacts or cache records.

## Constraints and Assumptions

- Retention controls memory lifetime, not semantic validity.
- Old snapshots may be readable while referenced, but cannot be reported as current after replacement.
- Collection order must not affect deterministic build output.
- The retention manager must avoid retaining all historical snapshots indefinitely in watch/LSP mode.
