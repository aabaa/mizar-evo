# mizar-build Scheduler

> Canonical language: English. Japanese companion:
> [../ja/scheduler.md](../ja/scheduler.md).

## Purpose

This document specifies the scheduler contract owned by `mizar-build`.

The scheduler consumes the deterministic `TaskGraph` produced by
`task_graph`, executes or skips tasks under dependency constraints, records
task states, and collates task results and scheduler events in canonical order.
It is an execution-order component, not a semantic authority.

## Context

- [architecture 14](../../architecture/en/14.parallel_verification_and_scheduling.md)
- [architecture 22](../../architecture/en/22.incremental_verification_contract.md)
- [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
- [task_graph.md](./task_graph.md)

## Scope

`scheduler` owns:

- ready-state computation over a validated `TaskGraph`;
- task state transitions for pending, ready, running, terminal, skipped, and
  blocked work;
- work-queue selection and priority keys for batch, watch, and LSP-oriented
  runs;
- deterministic result and event collation under arbitrary worker completion
  order;
- the build-side seam that later resource, cancellation, failure-state, cache,
  and commit modules consume.

`scheduler` does not own:

- `mizar-driver` build requests, sessions, phase registries, live progress
  transports, or LSP bridges;
- phase semantics, proof acceptance, ATP policy, kernel trust, artifact schema
  projection, or diagnostic rendering;
- `mizar-cache` `CacheKey`, dependency fingerprint, proof-reuse validation, or
  cache-store lookup;
- resource accounting internals, cancellation-token storage, failure taxonomy,
  artifact publication tokens, or manifest writes before their dedicated tasks;
- promotion of cache hits, artifact records, or skipped tasks to trusted proof
  status.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| SCH-G001 | `design_drift` | `todo.md` required `scheduler.md`, but no module spec existed before task 9. | Task 9 adds this spec and its Japanese companion. |
| SCH-G002 | `source_drift` / `test_gap` | `src/scheduler.rs` and scheduler tests do not exist yet. | Task 10 implements source/tests against this spec and `task_graph.md`. |
| SCH-G003 | `external_dependency_gap` | `mizar-driver` request/session/registry/event integration is absent in this checkout. | Accept caller-supplied graph/session-like inputs in later source; do not add a driver dependency or placeholder driver API. |
| SCH-G004 | `external_dependency_gap` | `mizar-ir` sealed output handles and storage adapters are absent. | Use synthetic in-memory task outputs in scheduler tests; do not invent IR storage APIs. |
| SCH-G005 | `deferred` | Resource-budget accounting is tasks 11-12. | Define resource classes and queue admission seams only; enforcement belongs to `resource.md` and `src/resource.rs`. |
| SCH-G006 | `deferred` | Cancellation is tasks 13-14. | Define versioned cancellation state transitions only; token storage and snapshot invalidation belong to `cancel.md`. |
| SCH-G007 | `deferred` | Failure-state propagation is tasks 15-16. | Define bounded blocked-state behavior only; detailed failure taxonomy belongs to `failure_state.md`. |
| SCH-G008 | `deferred` | Cache-aware scheduling is task 18 and `mizar-cache` already owns cache validation. | Model cache hits as validated execution-skip outcomes only; do not construct cache keys or proof-reuse decisions here. |
| SCH-G009 | `external_dependency_gap` | Real producer artifact publication tokens are not available to `mizar-build`. | Order commit tasks deterministically but do not publish artifacts or mint publication authority. |

## Data Model

The following shapes define the scheduler contract, not necessarily final Rust
names:

```rust
struct SchedulerInput {
    graph: TaskGraph,
    mode: SchedulerMode,
    priority_hints: PriorityHints,
    cache: CacheSchedulingSeam,
    resource_admission: ResourceAdmissionSeam,
    cancellation: CancellationSeam,
}

struct SchedulerRun {
    graph_version: TaskGraphVersion,
    snapshot: BuildSnapshotId,
    task_states: Vec<TaskStateRecord>,
    results: Vec<SchedulerResult>,
    events: Vec<SchedulerEvent>,
}

struct TaskStateRecord {
    task_id: TaskId,
    state: TaskState,
    dependencies: Vec<TaskId>,
    blocked_by: Vec<TaskId>,
}
```

`SchedulerInput` is build-side data. A future driver may wrap it in sessions
and live event streams, but `mizar-build` must remain usable without depending
on `mizar-driver`.

### TaskState

```rust
enum TaskState {
    Pending,
    Ready,
    Running,
    Completed,
    CacheHit,
    Skipped,
    Failed,
    Blocked,
    Cancelled,
}
```

State meanings:

- `Pending`: dependencies are not terminal yet.
- `Ready`: all correctness dependencies are terminal and the task may wait in
  a queue for resource admission before execution or cache probing.
- `Running`: a worker owns the task execution attempt.
- `Completed`: execution finished and produced scheduler-visible outputs.
- `CacheHit`: execution was skipped after external cache validation succeeded.
- `Skipped`: a statically declared conditional subgraph is no longer needed,
  for example ATP/backend work after deterministic discharge closes a VC.
- `Failed`: the task ran and produced a failing task outcome.
- `Blocked`: a required dependency failed, was cancelled, or lacks required
  coverage, so this task cannot produce a valid result in the current run.
- `Cancelled`: the task belongs to a superseded or explicitly cancelled
  snapshot.

`Completed`, `CacheHit`, `Skipped`, `Failed`, `Blocked`, and `Cancelled` are
scheduling terminal states. `Completed` and validated `CacheHit` are
successful/unblocking terminal states. `Skipped` is unblocking only for
conditional subgraphs whose parent task makes the child unnecessary. `Failed`,
`Blocked`, and `Cancelled` are blocking terminal states for correctness
dependents unless a later failure-state rule explicitly permits degraded
non-semantic work. None of these states by itself means semantic acceptance,
trusted proof status, or artifact publication.

### SchedulerResult

```rust
struct SchedulerResult {
    task_id: TaskId,
    state: TaskState,
    canonical_order: SchedulerOrderKey,
    output_refs: Vec<SyntheticOutputRef>,
    diagnostics: Vec<SchedulerDiagnosticRef>,
}
```

Task 10 may use synthetic output references. Real phase output handles remain
an external dependency gap until `mizar-ir` or the driver phase registry
provides them.

### SchedulerEvent

```rust
enum SchedulerEventKind {
    TaskBecameReady,
    TaskStarted,
    TaskFinished,
    TaskSkipped,
    TaskBlocked,
    RunFinished,
}

struct SchedulerEvent {
    kind: SchedulerEventKind,
    task_id: Option<TaskId>,
    order: SchedulerOrderKey,
}
```

Events are progress records for later driver integration. The canonical event
stream produced by `mizar-build` is sorted by `SchedulerOrderKey`; live worker
telemetry, when added by `mizar-driver`, is not an artifact, diagnostic, or
proof-ordering rule.

When events share the same scheduler order and task, the lifecycle rank is:
`TaskBecameReady`, `TaskStarted`, `TaskSkipped`, `TaskBlocked`,
`TaskFinished`, then `RunFinished`. A run-finished event sorts after all task
events for the same run.

## Readiness and State Transitions

Initial state:

1. Every non-root task starts as `Pending`.
2. In the current task-graph contract, the single `PackageResolve` root starts
   as `Completed` because phase 0 has already produced the `BuildPlan` before
   scheduling. A future scheduler-owned phase-0 mode may start it differently,
   but that is outside task 10.
3. A task becomes `Ready` when all graph dependencies are terminal states that
   satisfy the task's dependency coverage.

Terminal dependency handling:

- `Completed` and validated `CacheHit` unblock dependents.
- `Skipped` unblocks only dependents whose task kind allows a conditional
  proof subgraph to be skipped.
- `Failed`, `Blocked`, and `Cancelled` block correctness dependents unless a
  later failure-state rule explicitly permits degraded non-semantic work.

Execution transition:

```text
Pending -> Ready -> Running -> Completed
Pending -> Ready -> CacheHit  (after task 18 cache integration)
Pending -> Blocked
Ready   -> Blocked
Running -> Failed
Pending/Ready/Running -> Cancelled
Ready/Running -> Skipped
```

Task 10 implements the core deterministic scheduler for synthetic tasks.
Resource denial, cancellation, cache validation, and rich failure propagation
are represented through seams until their dedicated tasks implement them.

## Work Queues

The scheduler partitions ready tasks by scheduler queue. Queue selection is
derived from the task graph resource class and may refine it by `TaskKind` when
one resource class contains both coordination and worker-process tasks:

| Queue | Task classes |
|---|---|
| coordinator | root and graph bookkeeping |
| source/local CPU | `SourceLoad`, `Frontend`, `ModuleResolve`, `CheckAndElaborate`, `VcGenerate` |
| deterministic proof | `VcDischarge` |
| ATP portfolio | `AtpSolve` coordination and child-subgraph collation |
| ATP process | `BackendRun` child-process work |
| kernel | `KernelCheck` |
| I/O commit | `ArtifactCommit` |
| documentation | `DocumentationExtract` |

Queue choice does not change correctness. A resource module may delay queue
admission, but it must not remove graph dependencies or make completion order
semantic.

In the task-8 graph, `AtpSolve` and `BackendRun` both use the `AtpProcess`
resource class. The scheduler routes `AtpSolve` to portfolio coordination and
`BackendRun` to external backend process slots by task kind; this keeps the
Task 8 resource-class API stable until the later resource-budget tasks decide
whether a distinct portfolio resource class is needed.

## Priority Policy

The default priority key is a stable tuple:

1. build mode priority class;
2. open/requested-file hint rank;
3. graph dependency depth, with unblocking tasks before leaf work;
4. task kind rank from `task_graph.md`;
5. package/module/VC/backend/evidence canonical work-unit order;
6. `TaskId`.

Batch mode favors dependency summaries and lower phase work that unlocks the
most downstream tasks. Watch and LSP modes may raise source/frontend work for
open or requested files, but only by priority rank. Priority affects latency,
not diagnostics order, artifact order, proof selection, or cache publication.

Backend profile priority comes from the `TaskGraph` descriptor ordering. A
later ATP portfolio service may choose which backend work to declare, but the
scheduler must not select accepted evidence by worker completion time.

## Deterministic Collation

The scheduler may execute ready tasks in any worker order, but it collates
externally visible records by canonical keys:

- task states by task graph order and `TaskId`;
- scheduler results by package, module, task kind, descriptor order, and
  `TaskId`;
- diagnostics by the architecture diagnostic ordering, never by worker finish
  time;
- artifact commit attempts by canonical module and manifest order;
- canonical scheduler events by `SchedulerOrderKey`.

When two executions use the same `TaskGraph`, synthetic task behavior, resource
admission decisions, and cache seam responses, their `SchedulerRun` records
must be byte-for-byte equivalent after canonical serialization.

## Batch, Watch, and LSP Modes

### Batch

Batch mode attempts to drive every non-obsolete task to a terminal state. It
continues after recoverable task failures when independent work can still
produce useful diagnostics. The final build status is computed after all ready
work is completed, skipped, failed, blocked, or cancelled.

### Watch

Watch mode uses the same graph and state transitions, but a newer snapshot may
supersede pending or running work. Until cancellation is implemented, scheduler
tests use explicit synthetic cancellation seam responses instead of driver
watch APIs.

### LSP

LSP mode may prioritize open files and syntax/local feedback. It must not mix
diagnostics from incompatible snapshots or publish stale results as current.
Snapshot publication and editor protocol conversion remain `mizar-driver`/LSP
bridge responsibilities.

## Cache-Aware Scheduling Seam

Cache-aware scheduling is an optimization seam.

Before executing an admitted ready task, the scheduler may ask an external
cache seam for a validated outcome:

```rust
enum CacheSchedulingOutcome {
    ValidatedHit,
    Miss,
    Unavailable,
    Error,
}
```

Rules:

- `scheduler` does not construct `CacheKey`s.
- `scheduler` does not compute dependency fingerprints.
- `scheduler` does not validate proof reuse.
- `ValidatedHit` is enabled only when task 18 connects the real `mizar-cache`
  public contract; it may move a task to `CacheHit` and skip execution.
- `Miss` and `Unavailable` run the task normally.
- `Error` is treated as `Miss` unless a later explicit cache-required mode says
  to emit a cache diagnostic.
- `CacheHit` is a completed scheduling dependency, not proof evidence.
- cache miss timing and lookup order do not affect proof selection,
  diagnostics, artifact order, or trusted status.

Task 10 defaults this seam to `Unavailable` or `Miss` and must not produce
`CacheHit`. Task 18 connects the real `mizar-cache` public contract and adds
validated-hit execution-skip behavior.

## Failure, Cancellation, and Commit Seams

Failure propagation is bounded. A failed task blocks only correctness
dependents that require its outputs. Independent tasks may continue. Detailed
failure classes, blocked-state diagnostics, and degraded-mode permissions are
specified by `failure_state.md` in tasks 15-16.

Cancellation is versioned by snapshot. A cancelled task never publishes current
diagnostics or artifacts. Actual cancellation-token storage, child-process
termination, and snapshot supersession are specified by `cancel.md` in tasks
13-14.

Artifact commit remains a scheduling boundary in this spec. The scheduler
orders `ArtifactCommit` tasks canonically and records their outcomes, but it
does not write manifests, mint publication tokens, or treat artifact records
as proof authority.

## Tests

Task 9 is documentation-only. Task 10 must add focused Rust tests for:

- deterministic execution under shuffled worker completion order;
- worker-count variation producing identical result and event collation;
- immutable synthetic output publication and no shared mutable semantic output;
- readiness transitions over the Task 8 graph, including `PackageConservative`,
  `MissingModuleDependencyOverlay`, and `MissingVcDescriptors` coverage;
- coordinator, source/local CPU, deterministic proof, ATP portfolio, ATP
  process, kernel, I/O commit, and documentation queues;
- priority hints affecting execution latency but not canonical result order;
- reversed `BackendRun` and `KernelCheck` completion orders collating by
  descriptor/backend order without exposing proof-acceptance authority;
- `BuildTask.dependencies` and scheduler state records staying consistent;
- disabled cache seam behavior: `Miss`, `Unavailable`, and error-as-miss run
  tasks normally, produce no `CacheHit`, and do not construct cache keys;
- missing or failed dependencies producing bounded `Blocked` states;
- synthetic cancellation preventing current publication;
- absence of `mizar-driver`, `mizar-ir`, cache-key construction,
  dependency-fingerprint construction, proof-reuse validation, publication
  token, and proof-authority placeholders.

## Non-Authority Rules

- Scheduling readiness is not semantic acceptance.
- `CacheHit` is not proof evidence.
- `Skipped` ATP/backend work is not accepted proof status.
- Worker completion order never selects a proof candidate, diagnostic order,
  artifact order, or cache publication order.
- Artifact commit records do not promote trusted status.
