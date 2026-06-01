# Module: retention

> Canonical language: English. Japanese companion: [../ja/retention.md](../ja/retention.md).

## Purpose

This module defines retention leases and collection policy for `mizar-session`.

It keeps source text, source maps, and snapshot metadata alive while batch, watch, LSP, diagnostics, explanation, cache, artifact, or IR consumers still reference them. It does not retain typed IR outputs directly; `mizar-ir` owns IR output retention and may hold snapshot leases while its handles remain live.

## Public API

```rust
pub struct RetentionManager {
    // implementation-owned registry
}

pub struct RetainSnapshotInput {
    pub snapshot: BuildSnapshotId,
    pub owner: RetainOwner,
    pub reason: RetentionReason,
}

pub struct RetainGuard {
    pub lease_id: SnapshotLeaseId,
    pub snapshot: BuildSnapshotId,
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

pub trait SnapshotRetention {
    fn retain_snapshot(&self, input: RetainSnapshotInput) -> Result<RetainGuard, RetentionError>;
    fn release(&self, guard: RetainGuard);
    fn mark_current(&self, request: BuildRequestId, snapshot: BuildSnapshotId) -> Result<(), RetentionError>;
    fn unmark_current(&self, request: BuildRequestId, snapshot: BuildSnapshotId);
    fn collect(&self) -> CollectionSummary;
}
```

`RetainGuard` release should be idempotent from the caller's perspective, but duplicate release attempts are recorded for developer diagnostics.

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

### Current Marks

A current mark means a build request generation may report the snapshot as current. It is separate from a lease: a current mark prevents collection and controls freshness, while a lease only prevents collection.

Old snapshots may retain leases for stale diagnostics or explanation requests after a newer snapshot becomes current.

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

1. Validate that the snapshot exists.
2. Allocate a `SnapshotLeaseId`.
3. Increment the owner/reason count.
4. Return a `RetainGuard`.

Retaining a stale snapshot is allowed when the reason is diagnostic, explanation, LSP stale-artifact display, or IR output retention. It must not make the snapshot current.

### Release

1. Validate the lease id and snapshot id.
2. Decrement the owner/reason count.
3. Mark the lease as released.
4. If no lease and no current mark remains, make the snapshot eligible for collection.

Release must not synchronously delete data that another thread could still read through an active guard.

### Collection

The collector may remove a snapshot when:

- no live lease references it;
- no current mark references it;
- no source map, diagnostic explanation, or LSP publication is registered against it;
- downstream `mizar-ir` has released any phase-output retention lease for it.

Collection drops in-memory source text, source maps, and snapshot metadata. It does not delete published artifacts or cache records.

## Error Handling

`RetentionError` includes:

- unknown snapshot id;
- unknown or already-released lease id;
- lease snapshot mismatch;
- invalid owner/reason combination;
- attempt to mark a missing snapshot as current;
- collection blocked by inconsistent retention state.

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
