# mizar-build Cancellation

> Canonical language: English. Japanese companion:
> [../ja/cancel.md](../ja/cancel.md).

## Purpose

This document specifies the cancellation contract owned by `mizar-build`.

Cancellation is a scheduling and freshness control. It lets `mizar-build`
stop or discard obsolete work while preserving deterministic results,
clean-build equivalence, proof authority boundaries, cache-validation
boundaries, and the no-partial-publication rule.

## Context

- [architecture 14](../../architecture/en/14.parallel_verification_and_scheduling.md)
- [architecture 22](../../architecture/en/22.incremental_verification_contract.md)
- [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
- [scheduler.md](./scheduler.md)
- [resource.md](./resource.md)

## Scope

`cancel` owns:

- build-side cancellation state for scheduler runs and task-graph snapshots;
- cooperative cancellation tokens and monotonically increasing cancellation
  generations;
- deterministic cancellation decisions for pending, ready, running, and
  completed-obsolete work;
- freshness checks before scheduler-visible result, diagnostic, cache-record,
  or artifact-commit attempts;
- release handoff for resource reservations held by cancelled tasks.

`cancel` does not own:

- `mizar-driver` build requests, watch/LSP event streams, live sessions,
  `salsa` queries, or phase-service registries;
- hard worker-thread killing or OS-process supervision;
- ATP/backend process spawning, termination, stdout/stderr capture, or backend
  protocol details;
- `mizar-ir` output storage, sealed handles, or snapshot-handle rehydration;
- `mizar-cache` `CacheKey` construction, dependency fingerprint construction,
  cache-store compatibility checks, or proof-reuse validation;
- artifact schema ownership, manifest transaction internals, producer-owned
  publication tokens, or artifact writes;
- proof search, proof acceptance, kernel trust, backend winner selection, or
  trusted-status promotion.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| CAN-G001 | `design_drift` | `todo.md` required `cancel.md`, but no module spec existed before task 13. | Task 13 adds this spec and its Japanese companion. |
| CAN-G002 | `source_drift` / `test_gap` | `src/cancel.rs` and cancellation tests were absent before task 14. | Task 14 adds versioned tokens, snapshot invalidation, publication guards, scheduler integration, and focused tests. |
| CAN-G003 | `external_dependency_gap` | Driver request/session/watch/LSP integration is outside `mizar-build`. | Keep cancellation input-driven and snapshot-oriented; do not add a driver dependency or placeholder driver API. |
| CAN-G004 | `external_dependency_gap` | Real IR output storage and snapshot-handle rehydration are not available through a build-owned seam. | Keep cancelled/obsolete output handling synthetic until a real IR seam exists; do not invent IR handles or storage APIs. |
| CAN-G005 | `external_dependency_gap` | ATP/backend process managers are outside `mizar-build`. | Model cooperative cancellation and backend cancellation outcomes only; do not spawn, kill, or supervise OS processes. |
| CAN-G006 | `external_dependency_gap` | Real producer artifact publication tokens are not available to `mizar-build`. | Specify freshness and no-partial-publication guards; do not mint fake publication tokens. |
| CAN-G007 | `deferred` | Cache-aware scheduling is task 18 and `mizar-cache` owns cache keys, fingerprints, and proof-reuse validation. | Obsolete work may be reused only through future external cache validation; do not reimplement cache internals here. |
| CAN-G008 | `deferred` | Detailed failure taxonomy and bounded propagation are tasks 15-16. | Define cancellation as a terminal scheduling state; detailed failure categories remain in `failure_state.md`. |

## Data Model

The following shapes define the cancellation contract, not necessarily final
Rust names:

```rust
struct CancellationInput {
    current_snapshot: BuildSnapshotId,
    superseded_snapshots: Vec<BuildSnapshotId>,
    explicit_task_cancellations: Vec<TaskId>,
    generation: CancellationGeneration,
}

struct CancellationToken {
    snapshot: BuildSnapshotId,
    generation: CancellationGeneration,
    reason: Option<CancellationReason>,
}

enum CancellationReason {
    SupersededSnapshot,
    ExplicitRequest,
    Shutdown,
    BudgetPolicy,
}

enum CancellationDecision {
    Continue,
    CancelBeforeStart,
    CancelAtCheckpoint,
    DiscardObsoleteResult,
    CommitAlreadyStarted,
}
```

Every cancellation decision is scoped to a task graph version and
`BuildSnapshotId`. A future driver may provide the snapshot stream, but
`mizar-build` consumes the identifiers and decisions without depending on
`mizar-driver`.

If `current_snapshot` is known and differs from the graph snapshot,
`mizar-build` treats the graph as already superseded for scheduling purposes:
pending and ready work is cancelled before start, running work observes the
next safe checkpoint, and any already completed result still passes through the
publication freshness guard before it can become scheduler-visible output. An
artifact commit whose atomic transaction has already started is the exception:
`mizar-build` does not interrupt that transaction, and the deterministic commit
boundary decides the visible outcome.

## Versioned Cancellation

Every scheduled task belongs to exactly one task graph version and one
`BuildSnapshotId`. Cancellation state is versioned by a monotonically
increasing generation within a scheduler run.

Rules:

1. a newer source/dependency snapshot may supersede older snapshots;
2. once a snapshot is cancelled, every task in that snapshot observes a
   cancellation token with the same or newer generation;
3. repeated cancellation of the same snapshot or task is idempotent;
4. cancellation decisions are sorted deterministically by snapshot, task graph
   order, and `TaskId`;
5. `Cancelled` is a terminal scheduling state, not a proof status, cache
   validation result, semantic acceptance, or artifact publication state.

Cancellation may change which work is executed, but it must not change the
semantic meaning of a non-cancelled result.

## Checkpoints And State Transitions

Cancellation is cooperative for in-process `mizar-build` work.

| Existing state | Cancellation behavior |
|---|---|
| `Pending` | Move to `Cancelled` without becoming ready, running, or publishing outputs. |
| `Ready` | Remove from dispatch/admission and move to `Cancelled`. |
| `Running` | Continue until the next phase boundary or safe checkpoint, then move to `Cancelled` and release resources exactly once. |
| completed but obsolete | Discard scheduler-visible outputs before publication unless a later cache seam externally validates reuse for the current snapshot. |
| artifact commit not started | Cancel before opening the atomic publication transaction. |
| artifact commit already started | Do not interrupt the transaction inside `mizar-build`; the deterministic commit boundary decides the visible outcome. |

Safe checkpoints include scheduler dispatch boundaries, resource-admission
boundaries, task phase boundaries, cache-probe boundaries, and publication
freshness checks. A long-running phase that wants finer cancellation must expose
its own safe checkpoints through its phase service; `mizar-build` must not kill
threads to force that behavior.

For external ATP/backend processes, `mizar-build` may record that the task was
cancelled after the backend manager reports termination or a cancellation
outcome. Process creation and termination remain outside this crate.

## Publication And Freshness

Cancellation protects current-snapshot publication:

- diagnostics from cancelled or superseded work must not be published as
  current diagnostics;
- proof statuses from cancelled or superseded work must not become accepted,
  trusted, or more authoritative;
- phase outputs, cache records, artifact drafts, and commit requests from
  obsolete snapshots must not be published as current results;
- LSP/watch consumers may keep previously visible stale diagnostics until a
  newer snapshot replaces them, but `mizar-build` must label and collate
  results by snapshot so stale work cannot masquerade as current work;
- open editor buffers are source snapshots, not artifacts;
- no partial artifact may become visible because a cancellation interrupted a
  scheduler run.

If a cancelled task had reserved resources, the scheduler/resource boundary
releases them when the task reaches `Cancelled`. Releasing resources never
publishes outputs.

## Cache, Artifact, And Proof Boundaries

Cancellation is not cache validation. A completed result from an obsolete
snapshot may become useful for a later snapshot only if the cache-aware seam
later asks `mizar-cache` and receives an externally validated decision. This
crate must not construct local cache keys, dependency fingerprints, or proof
reuse validators to justify that reuse.

Cancellation is not artifact authority. It may prevent a stale commit attempt
from starting, but it does not own manifest transactions, artifact schema, or
publication tokens.

Cancellation is not proof authority. A cancelled task does not prove failure,
success, incompleteness, or acceptance. It only says the scheduler must not
publish that task's work as current for the cancelled snapshot.

## Determinism

Cancellation timing affects latency and amount of wasted work only.

For the same task graph, snapshot sequence, cancellation generations,
synthetic task outcomes, resource configuration, and cache decisions, the
canonical scheduler run after collation must be identical across worker counts
and worker completion orders.

Canonical diagnostics, artifact ordering, cache publication ordering, and proof
status ordering must not depend on the instant at which a worker observed a
cancellation token. Cancellation telemetry may explain discarded work, but it
is not a semantic ordering key.

## Task 14 Coverage

Task 14 adds focused Rust coverage for:

- pending and ready work cancelled before start;
- running work cancelled only at safe checkpoints;
- monotonic cancellation-generation advancement, same-or-newer token
  propagation to cancelled snapshot tasks, and canonical cancellation-decision
  ordering by snapshot, task graph order, and `TaskId`;
- idempotent repeated cancellation of the same snapshot or task;
- snapshot supersession discarding obsolete completed results before
  publication;
- cancelled work producing no current diagnostics, outputs, cache records, or
  artifact commit attempts;
- resources released exactly once for admitted cancelled tasks;
- deterministic cancellation results under shuffled ready/completion order and
  different worker counts;
- commit-boundary behavior before and after the modeled atomic transaction
  begins;
- absence of `mizar-driver`, `mizar-cache`, `mizar-ir`, OS-process, fake
  artifact-token, or proof-authority placeholders.

## Non-Authority Rules

- A cache hit remains an execution-skip candidate only after external cache
  validation; cancellation never validates a hit.
- An artifact record or cancellation event never upgrades trusted status.
- A cancelled proof task never counts as kernel acceptance, backend victory, or
  semantic rejection.
- `mizar-build` remains independent of `mizar-driver`; driver-owned sessions
  may call into this contract, but this crate must not depend on them.

## Public Enum Policy

No exhaustive public enum exceptions are owned by this module.

| Enum | Decision |
|---|---|
| `CancellationReason` | `#[non_exhaustive]`; downstream callers must include wildcard match arms. |
| `CancellationDecision` | `#[non_exhaustive]`; downstream callers must include wildcard match arms. |
| `CancellationCheckpoint` | `#[non_exhaustive]`; downstream callers must include wildcard match arms. |
