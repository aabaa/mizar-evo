# mizar-build Resources

> Canonical language: English. Japanese companion:
> [../ja/resource.md](../ja/resource.md).

## Purpose

This document specifies the resource-budget and queue-admission contract owned
by `mizar-build`.

Resource budgeting bounds parallelism and external process pressure without
changing task correctness, proof acceptance, diagnostic order, artifact order,
or cache behavior. It consumes the `TaskGraph` and scheduler queues defined by
`task_graph.md` and `scheduler.md`; it does not create new semantic authority.

## Context

- [architecture 14](../../architecture/en/14.parallel_verification_and_scheduling.md)
- [architecture 22](../../architecture/en/22.incremental_verification_contract.md)
- [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
- [scheduler.md](./scheduler.md)
- [task_graph.md](./task_graph.md)

## Scope

`resource` owns:

- hierarchical budget scopes for workspace, package, module, VC obligation,
  backend process, and commit work;
- deterministic queue-admission decisions for ready tasks;
- accounting for modeled worker slots, memory units, ATP process slots, timeout
  budgets, capture limits, and I/O commit permits;
- starvation prevention between local semantic work and expensive ATP/backend
  work;
- telemetry records that explain why ready work was admitted or delayed.

`resource` does not own:

- task graph construction or correctness dependency edges;
- phase semantics, proof selection, kernel trust, or ATP policy decisions;
- OS process spawning, child termination, stdout/stderr capture
  implementation, or backend protocol details;
- cache-key construction, dependency fingerprints, proof-reuse validation, or
  cache hit authority;
- artifact publication tokens, manifest writes, or trusted-status promotion;
- driver session ownership, watch/LSP event streams, or `salsa` query storage.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| RES-G001 | `design_drift` | `todo.md` required `resource.md`, but no module spec existed before task 11. | Task 11 adds this spec and its Japanese companion. |
| RES-G002 | `source_drift` / `test_gap` | `src/resource.rs` and resource-budget tests do not exist yet. | Task 12 implements accounting/admission source and focused tests against this spec. |
| RES-G003 | `external_dependency_gap` | Real ATP backend process managers and capture adapters are outside `mizar-build`. | Model backend limits and handoff values only; do not spawn or supervise OS processes here. |
| RES-G004 | `external_dependency_gap` | `mizar-driver` request/session/watch integration is absent in this checkout. | Keep budgets input-driven and session-agnostic; do not add a driver dependency or placeholder driver API. |
| RES-G005 | `deferred` | Cache-aware scheduling remains task 18 and `mizar-cache` owns validation. | Resource decisions may prioritize cache-ready work but must not construct cache keys or promote cache hits. |
| RES-G006 | `deferred` | Cancellation and failure-state policies are tasks 13-16. | Resource release on terminal states is specified here; cancellation tokens and failure categories remain in their modules. |

## Budget Model

Budgets are hierarchical. A task may start only when all applicable ancestor
budgets can reserve their required units.

| Budget | Applies To | Examples |
|---|---|---|
| workspace | whole scheduler run | total local workers, total modeled memory, total ATP processes, total I/O commits |
| package | one package | package build concurrency, package-local semantic workers |
| module | one workspace module | active module-phase workers, active VC obligations from the module |
| obligation | one VC descriptor | ATP timeout budget, axiom/search policy budget, backend fanout limit |
| backend | one backend attempt | process slot, wall-clock timeout, memory ceiling, stdout/stderr capture limit |
| commit | artifact publication boundary | manifest transaction permit, artifact write permit |

Budgets are conservative. If an upstream integration cannot describe a precise
unit, it must request the nearest safe coarse unit rather than overcommit.

Task 12 may start with integral units:

```rust
struct ResourceBudget {
    workspace_workers: usize,
    source_workers: usize,
    proof_workers: usize,
    atp_processes: usize,
    kernel_workers: usize,
    io_commits: usize,
    memory_units: usize,
}

struct TaskResourceRequest {
    queue: SchedulerQueue,
    package: Option<PackageId>,
    module: Option<ModuleId>,
    vc: Option<VcTaskDescriptorId>,
    worker_units: usize,
    memory_units: usize,
    external_process_slots: usize,
}
```

The shape is illustrative. The invariant is that every admitted task has a
bounded request and every terminal task releases its reservation exactly once.

## Queue Admission

Ready tasks first enter the scheduler queue selected by `scheduler.md`.
Resource admission then decides whether a ready task may move to a running
worker slot.

Admission must:

1. preserve correctness dependency edges from the `TaskGraph`;
2. reserve all applicable ancestor budgets atomically;
3. use deterministic tie-breakers for tasks competing for the same budget;
4. leave denied tasks ready or queued, not failed, when denial is due only to
   temporary exhaustion;
5. emit stable admission telemetry without changing canonical diagnostics or
   artifact ordering.

Admission denial for temporary exhaustion is not semantic failure. A task that
is ready but lacks budget remains schedulable when resources are released.

If a task requests more resources than the configured hard limit can ever
satisfy, task 12 must report a stable scheduler/resource diagnostic and block
the task rather than spin indefinitely. The diagnostic is not proof evidence and
does not promote or demote trusted status by itself.

## Deterministic Ordering

Resource availability affects latency only.

When multiple ready tasks compete for a budget, admission order is:

1. scheduler priority key from `scheduler.md`;
2. resource queue rank;
3. package/module/VC/backend canonical work-unit order from the `TaskGraph`;
4. `TaskId`.

Worker completion order must not select proof candidates, diagnostic order,
artifact order, cache publication order, or manifest commit order.

Two runs with the same `TaskGraph`, resource configuration, synthetic task
behavior, cache seam responses, and cancellation inputs must produce the same
canonical `SchedulerRun` after collation even if resource release timing differs
inside the run.

## Worker Pools

Worker pools are modeled by resource classes and queue permits:

| Pool | Scheduler queues | Typical work |
|---|---|---|
| coordinator | coordinator | root bookkeeping and graph-level orchestration |
| source/local CPU | source/local CPU | source loading, frontend, module resolution, checking, VC generation |
| deterministic proof | deterministic proof | deterministic discharge and bounded computation |
| ATP portfolio | ATP portfolio | portfolio coordination and child-subgraph accounting |
| ATP process | ATP process | external backend process attempts |
| kernel | kernel | kernel/SAT evidence validation |
| I/O commit | I/O commit | artifact/manifest commit attempts |
| documentation | documentation | documentation extraction after verified artifacts |

The table order is the default resource queue rank used as the second
admission tie-breaker after scheduler priority. Task 12 may add aging or
fairness within a queue, but it must do so with deterministic state that does
not change canonical result collation.

The ATP portfolio pool is coordination work. It must not consume an external
backend process slot unless it starts a concrete `BackendRun`.

The ATP process pool represents an external-process limit that is handed to the
backend runner. `mizar-build` records and enforces the modeled slot; the backend
manager owns actual process creation, termination, and output capture.

## ATP and Backend Limits

For one VC portfolio:

- `AtpSolve` consumes a proof/portfolio coordination permit.
- Each `BackendRun` consumes one backend process slot and the relevant
  obligation/backend budget.
- Backend fanout is capped by the smaller of the obligation fanout limit and
  the global ATP process pool.
- Backend timeout, memory, and capture limits are handed to the ATP/backend
  runner as constraints, not interpreted as proof outcomes by `mizar-build`.
- Releasing a backend slot does not select accepted evidence; proof selection
  remains policy/kernel work.

If early proof policy later determines that no pending backend can displace the
current winner, cancellation of surplus backend work is coordinated by the
cancellation and ATP modules, not by resource accounting alone.

## Release and Accounting

A resource reservation is released when the admitted task reaches a terminal
scheduling state:

- `Completed`
- `CacheHit`
- `Skipped`
- `Failed`
- `Blocked`
- `Cancelled`

Releasing resources does not publish outputs. Publication is governed by the
scheduler, artifact, proof, and cancellation boundaries.

Resource telemetry may record:

- task id and queue;
- requested units;
- admitted or delayed status;
- blocking budget scope;
- deterministic admission attempt order.

Telemetry is for progress/debugging and later driver events. It is not a
diagnostic ordering key, artifact manifest input, cache validation input, or
proof authority.

## Tests

Task 11 is documentation-only. Task 12 must add focused Rust tests for:

- budget exhaustion queues ready work without overcommit;
- reservations compose workspace/package/module/obligation/backend/commit
  budgets;
- resource release happens exactly once for every terminal state;
- worker-count changes do not change canonical result/event collation;
- ATP portfolio coordination does not consume backend process slots;
- backend fanout is bounded by both obligation and global ATP process budgets;
- I/O commit permits serialize modeled commit work without minting publication
  authority;
- deterministic admission ordering is stable under shuffled ready/completion
  order;
- impossible requests produce stable resource diagnostics rather than infinite
  retries;
- resource telemetry does not affect proof status, diagnostics, artifact order,
  cache identity, or trusted status;
- no `mizar-driver`, `mizar-cache`, ATP OS-process, artifact publication token,
  or proof-authority placeholder is introduced.

## Non-Authority Rules

- Resource admission is not semantic acceptance.
- Having budget does not make a proof candidate accepted.
- Lacking temporary budget is not proof failure.
- Backend process completion order never selects proof evidence.
- Resource telemetry does not affect cache validation or trusted status.
- Artifact commit permits do not mint publication authority.
