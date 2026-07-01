# Diagnostic Aggregator

> Canonical language: English. Japanese companion:
> [../ja/aggregator.md](../ja/aggregator.md).

## Purpose

This document specifies the build-snapshot diagnostic aggregator owned by
`mizar-diagnostics`. The aggregator consumes immutable producer batches, removes
duplicates, assigns deterministic snapshot-local handles, and publishes an
immutable `BuildDiagnosticIndex` for later CLI, artifact, or LSP projection.

The aggregator is the first owner of `DiagnosticRecord` publication. It does
not own phase success, proof acceptance, trusted status, kernel acceptance,
driver session orchestration, artifact mutation, or LSP protocol conversion.

## Scope

The aggregator owns:

- accepting sealed `DiagnosticBatch` values from producer sinks;
- filtering obsolete producer snapshots out of current publication;
- projecting current `DiagnosticDraft` values to `DiagnosticRecord` values;
- deduplicating diagnostics by stable structured identity, not message text;
- assigning deterministic `DiagnosticId` and `DiagnosticHandle` values;
- sorting records into a deterministic source-indexed publication order;
- producing immutable `BuildDiagnosticIndex` values and byte-stable debug
  snapshots for tests.

The aggregator does not own:

- creating diagnostic codes or deciding registry compatibility;
- constructing phase-local drafts or validating producer-local facts;
- CLI formatting, terminal styles, source excerpts, or line/column conversion;
- LSP UTF-16 ranges, JSON-RPC publication, overlays, or code actions;
- phase status, scheduler recovery, proof/kernel/trusted acceptance, or build
  success;
- cache writes, artifact commits, source-map path allocation, or driver event
  streams.

## Inputs

Task 9 aggregation consumes this conceptual input:

```rust
struct DiagnosticAggregationInput {
    publication_snapshot: BuildSnapshotId,
    batches: Vec<DiagnosticBatch>,
}
```

Each batch carries a `DiagnosticProducerScope` and a local-order list of
validated `DiagnosticDraft` values. The sink has already checked that every
draft in a batch has the same phase and source snapshot as the batch scope.
Aggregation may re-check those invariants defensively, but it must not treat a
sink error as user-facing diagnostic output.

`publication_snapshot` is the coherent build snapshot for current publication.
The aggregator receives it from the caller; it does not create snapshots or
decide when a driver session is coherent.

## BuildDiagnosticIndex

Task 9 should expose an immutable index equivalent to:

```rust
struct BuildDiagnosticIndex {
    snapshot: BuildSnapshotId,
    records: Vec<DiagnosticRecord>,
    by_source: BTreeMap<SourceKey, Vec<DiagnosticHandle>>,
    by_id: BTreeMap<DiagnosticId, usize>,
}
```

`records` is the canonical publication order. `by_source` indexes handles by
the primary span source. `by_id` maps snapshot-local ids to canonical record
positions. The concrete implementation may hide `SourceKey` behind methods, but
lookup behavior must be deterministic.

`SourceId` values from `mizar-session` are compiler-native identities. They are
not currently persistable and do not implement semantic ordering. For task 9,
the canonical source key is the published-schema string if the session id can be
serialized, otherwise its `Debug` rendering. This is a deterministic test key,
not a durable artifact path. Workspace path normalization remains a caller or
source-map responsibility.

Phase status is intentionally absent from task 9's index. Architecture sketches
show phase status adjacent to diagnostics, but `mizar-diagnostics` does not own
phase semantics or driver orchestration. Joining phase status with an index is a
future driver/LSP adoption point and is an `external_dependency_gap`, not a
placeholder field here.

## Freshness And Obsolete Snapshots

Only drafts whose `source_snapshot` equals `publication_snapshot` may become
current records in the `BuildDiagnosticIndex`.

Drafts from any other source snapshot are obsolete for the current publication.
Task 9 must not publish them as `DiagnosticFreshness::Current`. The initial
implementation should withhold them from `records` and expose deterministic
stale/obsolete accounting for tests. It may later produce explicit stale or
historical views for LSP overlays or artifact reads, but those views are not
current `BuildDiagnosticIndex` output.

This rule is strict even when code, span, and message text are otherwise
identical. A stale diagnostic can remain visible only when an owner outside this
crate, such as the LSP bridge, marks it as stale and suppresses unsafe actions.

## Deduplication Identity

Deduplication is keyed on stable machine-readable fields:

1. `DiagnosticCode`.
2. `PipelinePhase`.
3. `FailureCategory`.
4. Primary span source key, start, end, role, freshness, and zero-width intent.
5. `stable_detail_key`.
6. Canonically ordered structured details.
7. Ordered canonical fix payloads: suggestion id, producer key,
   applicability, safety, ordered edits including expected text, optional
   command reference, and snapshot/hash preconditions.
8. Optional canonical explanation handle identity.

Message text, localized text, rendered labels, terminal styling, LSP ranges,
source excerpts, and producer order are not identity. Changing a message must
not change whether two diagnostics deduplicate.

The aggregator must not merge diagnostics with different primary spans,
different structured details, different canonical fix payloads, or different
canonical explanation handle identities. Fix titles, diagnostic messages,
rendered help, localized text, and rendered explanation previews are
presentation, not identity. Secondary spans and notes are preserved from the
canonical representative; producers should put identity-bearing distinctions in
structured details, fixes, or explanation handle metadata instead of relying on
human text.

When several drafts have the same dedup identity but different presentation
payloads, the aggregator chooses a representative deterministically by sorting
the full presentation payload as a tie-breaker. This tie-breaker affects only
which human text is displayed; it is not diagnostic identity.

## Ordering And Handles

Aggregation must be independent of producer order and batch order. Shuffling
input batches or drafts that carry the same content must produce byte-identical
indexes and the same handles.

Canonical record order is:

1. Primary source key.
2. Primary range start.
3. Primary range end.
4. Phase order from `PipelinePhase`.
5. Registry severity order.
6. `DiagnosticCode`.
7. `FailureCategory`.
8. `stable_detail_key`.
9. Dedup identity key.
10. Full presentation tie-breaker.

After deduplication and sorting, `DiagnosticId` values are assigned as dense
zero-based ordinals in canonical record order. A `DiagnosticHandle` is
meaningful only as `(publication_snapshot, DiagnosticId)`. Consumers must not
persist bare ids across unrelated snapshots and must key durable workflows on
`DiagnosticCode` plus structured fields.

## Debug Snapshot

`BuildDiagnosticIndex::debug_snapshot()` is deterministic test output, not CLI
rendering. It uses LF line endings, no color, no localized field names, and this
field order:

1. `kind=index`.
2. `snapshot`.
3. `record_count`.
4. `obsolete_count`.
5. `record[0]`, `record[1]`, ... in canonical order, embedding each
   `DiagnosticRecord::debug_snapshot()` with trailing newline removed and
   internal newlines escaped as `\n`.

If task 9 exposes obsolete accounting entries, their debug form must be sorted
by source snapshot, producer name, local draft ordinal, and draft debug
snapshot, and rendered after the `record[n]` lines as `obsolete[n]` lines. The
snapshot must not include memory addresses, thread ids, hash-map iteration
order, localized text as keys, or process-local ordering.

## Public Enum Compatibility

Task 18 marks `DiagnosticAggregationError` as `#[non_exhaustive]` for downstream
forward compatibility. Aggregation errors are producer/index boundary failures,
not user diagnostic identities. Adding an error variant requires updating tests
and this spec; downstream consumers must keep wildcard handling.

## Boundary Rules

- Aggregation publishes records; it does not render them.
- Diagnostic severity can affect ordering, but it cannot decide phase success
  or proof/kernel acceptance.
- `DiagnosticCode` remains the stable tool identity. Message text is never the
  authority for deduplication, lookup, or consumer behavior.
- Existing lexer/frontend/parser/resolver diagnostic migration remains
  deferred until real consumers adopt the shared index. Task 9 must not add
  placeholder adapters or stub APIs for those crates.
- Driver, LSP, artifact, and resolver adoption are external dependencies. If
  they are not ready, record the gap and keep `mizar-diagnostics` independent.
