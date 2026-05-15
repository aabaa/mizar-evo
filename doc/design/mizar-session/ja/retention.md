# Module: retention

> Canonical language: English. English canonical version: [../en/retention.md](../en/retention.md).

## Purpose

この module は `mizar-session` の retention leases and collection policy を定義する。

Batch、watch、LSP、diagnostics、explanation、cache、artifact、IR consumers が source text、source maps、snapshot metadata を参照している間、それらを alive に保つ。Typed IR outputs は直接 retain しない。`mizar-ir` が IR output retention を所有し、その handles が live な間 snapshot leases を保持してよい。

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

pub enum RetentionReason {
    ActiveBuild,
    CurrentWatchBaseline,
    PublishedLspSnapshot,
    OpenBufferOverlay,
    DiagnosticIndex,
    ExplanationRequest,
    PhaseOutputReference,
    PendingWrite,
}

pub trait SnapshotRetention {
    fn retain_snapshot(&self, input: RetainSnapshotInput) -> Result<RetainGuard, RetentionError>;
    fn release(&self, guard: RetainGuard);
    fn mark_current(&self, request: BuildRequestId, snapshot: BuildSnapshotId) -> Result<(), RetentionError>;
    fn unmark_current(&self, request: BuildRequestId, snapshot: BuildSnapshotId);
    fn collect(&self) -> CollectionSummary;
}
```

`RetainGuard` release は caller から見ると idempotent であるべきだが、duplicate release attempts は developer diagnostics 用に記録される。

## Dependencies

- Internal: `ids`, `snapshot`, `source`, `source_map`
- External: weak-reference or arena storage utilities、tracing/logging

この module は snapshot registry、LSP snapshot publication、diagnostic aggregation、explanation queries、cache/artifact writers、`mizar-ir` から使われる。

## Data Structures

### Retention Record

各 retained snapshot は次を含む record を持つ。

- snapshot id
- reference counts by owner and reason
- current request generations naming it
- retained loaded sources
- retained line maps and preprocessing maps
- collection eligibility metadata
- optional debug creation/release traces

Record は session-local retention state を追跡する。Published artifacts には serialize しない。

### Current Marks

Current mark は build request generation が snapshot を current として report できることを意味する。Lease とは別である。Current mark は collection を防ぎ freshness を制御する一方、lease は collection のみを防ぐ。

Newer snapshot が current になった後でも、old snapshots は stale diagnostics or explanation requests のため lease を retain してよい。

### Collection Summary

`CollectionSummary` reports:

- snapshots scanned
- snapshots collected
- sources and maps released
- snapshots skipped because of current marks
- snapshots skipped because of live leases
- stale or mismatched lease diagnostics

Build semantics ではなく logging and tests 用である。

## Algorithm / Logic

### Retain

1. Snapshot が存在することを validate する。
2. `SnapshotLeaseId` を allocate する。
3. Owner/reason count を increment する。
4. `RetainGuard` を返す。

Diagnostic、explanation、LSP stale-artifact display、IR output retention の reason では stale snapshot を retain してよい。それによって snapshot を current にしてはならない。

### Release

1. Lease id and snapshot id を validate する。
2. Owner/reason count を decrement する。
3. Lease を released として mark する。
4. Lease and current mark が残っていなければ snapshot を collection eligible にする。

Release は、active guard を通して別 thread がまだ読める data を同期的に delete してはならない。

### Collection

Collector は次の条件を満たす snapshot を remove してよい。

- no live lease references it
- no current mark references it
- no source map, diagnostic explanation, or LSP publication is registered against it
- downstream `mizar-ir` has released any phase-output retention lease for it

Collection は in-memory source text、source maps、snapshot metadata を drop する。Published artifacts or cache records は削除しない。

## Error Handling

`RetentionError` includes:

- unknown snapshot id
- unknown or already-released lease id
- lease snapshot mismatch
- invalid owner/reason combination
- attempt to mark a missing snapshot as current
- collection blocked by inconsistent retention state

Invalid retention state は compiler internal error である。User-facing builds は可能なら previous coherent snapshot を使い続けるべきである。

## Tests

Key scenarios:

- active build lease prevents collection
- current mark prevents collection even without other leases
- stale LSP or diagnostic lease retains old source maps without marking the snapshot current
- releasing the final lease makes the snapshot collectible
- `mizar-ir` phase-output lease blocks snapshot collection until released
- duplicate release is reported but does not underflow counts
- collection does not delete artifacts or cache records

## Constraints and Assumptions

- Retention controls memory lifetime, not semantic validity.
- Old snapshots may be readable while referenced, but cannot be reported as current after replacement.
- Collection order must not affect deterministic build output.
- The retention manager must avoid retaining all historical snapshots indefinitely in watch/LSP mode.
