# mizar-build Scheduler-Selected Phase Dispatch

> Canonical language: English. Japanese companion:
> [../ja/phase_dispatch.md](../ja/phase_dispatch.md).

## Purpose

This document specifies the `mizar-build` seam that lets the scheduler ask an
external owner to execute the task that the scheduler has selected.

The seam closes the build-side part of DRIVER-G-011. `mizar-build` continues to
own task readiness, dependency ordering, resource admission, cancellation
checkpoints, cache-aware decision consumption, and deterministic collation.
`mizar-driver` may consume the seam by passing a registry-backed dispatcher,
but the driver must not reproduce scheduler semantics.

## Context

- [scheduler.md](./scheduler.md)
- [cache_seam.md](./cache_seam.md)
- [task_graph.md](./task_graph.md)
- [architecture 14](../../architecture/en/14.parallel_verification_and_scheduling.md)
- [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
- [architecture 22](../../architecture/en/22.incremental_verification_contract.md)
- [mizar-driver driver.md](../../mizar-driver/en/driver.md)
- [mizar-driver registry.md](../../mizar-driver/en/registry.md)

## Ownership Boundary

`mizar-build` owns:

- selecting ready tasks from the canonical task graph;
- consuming validated cache decisions before execution;
- admitting selected tasks under worker and resource limits;
- applying cancellation before start, at the running checkpoint, and before
  publication;
- calling the dispatch callback only after the scheduler selects a task for
  execution;
- mapping the callback's scheduler outcome to task state, failure/block
  records, resource release, and deterministic event/result collation.

`mizar-build` does not own:

- phase semantics, type checking, name resolution, overload resolution, VC
  generation, ATP policy, kernel acceptance, proof acceptance, trusted status,
  or proof reuse;
- driver build requests, sessions, event streams, phase-service registry, or
  `salsa` query storage;
- `mizar-cache` cache-key construction, dependency fingerprints, cache-store
  compatibility, or proof-reuse validation;
- `mizar-ir` sealed output storage, producer payload schemas, or snapshot
  rehydration;
- artifact serialization, manifest publication tokens, producer-owned
  artifact projection, or LSP protocol conversion.

The callback returns a scheduler outcome only. It does not transfer phase
semantics into `mizar-build`, and it does not let `mizar-build` mint output
handles, artifact publication tokens, cache compatibility decisions, or proof
authority.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| DISPATCH-G001 | `source_drift` / `test_gap` | DRIVER-G-011 records that `mizar-build` did not expose a scheduler-selected real phase dispatch callback even though scheduling owns the execution-selection point. | This task adds the callback seam and focused tests without adding a `mizar-driver` dependency. |
| DISPATCH-G002 | `external_dependency_gap` | Real phase services require owner-supplied phase input identities, parent output handles, diagnostics sinks, and output publishers. These are not build-owned data. | The driver consumes the seam at scheduler-selected dispatch points. If owner inputs are missing for a selected task, the registry-backed dispatcher records an owner gap and does not fabricate inputs. |
| DISPATCH-G003 | `external_dependency_gap` | Real producer outputs and artifact publication tokens remain outside `mizar-build`. | Dispatch completion is scheduler completion only. Artifact publication waits for the owning producer/artifact seam. |
| DISPATCH-G004 | `deferred` | Full clean/incremental/parallel equivalence over real semantic, proof, artifact, cache, and LSP integrations requires more owner seams than this task exposes. | Keep existing implemented-seam equivalence tests and add focused dispatch-seam tests. |

## API Contract

The Rust surface is intentionally separated from `SchedulerInput` so the input
remains cloneable/comparable deterministic build data:

```rust
pub trait SchedulerTaskDispatcher {
    fn dispatch(&mut self, task: SchedulerDispatchTask<'_>) -> SchedulerDispatchOutcome;
}

pub struct SchedulerDispatchTask<'a> {
    pub task: &'a BuildTask,
    pub snapshot: BuildSnapshotId,
    pub cancellation: Option<CancellationToken>,
}

pub struct SchedulerDispatchOutcome {
    pub status: SchedulerDispatchStatus,
    pub diagnostics: Vec<SchedulerDiagnosticRef>,
}

#[non_exhaustive]
pub enum SchedulerDispatchStatus {
    Complete,
    Failed,
    Blocked,
    Skipped,
    Cancelled,
}

pub fn run_scheduler_with_dispatcher<D: SchedulerTaskDispatcher>(
    input: SchedulerInput,
    dispatcher: &mut D,
) -> Result<SchedulerRun, SchedulerDiagnostics>;
```

`run_scheduler` remains available for modeled/synthetic scheduler fixtures. It
does not execute real phase services.

External crates consume the selected task through public fields on
`SchedulerDispatchTask`. `SchedulerDispatchOutcome` exposes public fields and
constructor helpers (`complete`, `failed`, `blocked`, `skipped`, and
`cancelled`) so a driver/registry adapter can return scheduler states without
constructing fake producer outputs or artifact publication tokens.

The callback is invoked only for a task that has passed scheduler readiness,
cache fallback, cancellation-before-start, and resource admission. A validated
cache hit still skips callback execution. The callback receives the
build-owned task identity, snapshot, and an optional cooperative cancellation
token. It does not receive mutable scheduler internals.

The callback may return:

- `Complete`: the scheduler records `Completed`;
- `Failed`: the scheduler records `Failed` and deterministic failure records;
- `Blocked`: the scheduler records a direct dispatch block and blocks
  correctness dependents;
- `Skipped`: the scheduler records `Skipped`; existing conditional subgraph
  rules still determine which dependents can be unblocked or skipped before any
  dependent callback runs;
- `Cancelled`: the scheduler records `Cancelled`.

Before publishing the callback outcome, the scheduler still applies the
completed-before-publication freshness checkpoint. Resource releases and event
ordering are scheduler-owned and deterministic.

## Driver Consumption Contract

`mizar-driver` consumes this seam by passing a dispatcher whose body calls the
phase registry query boundary for the scheduler-selected task. The driver may
prepare phase inputs, diagnostics sinks, cancellation resources, and output
publisher handles because those are driver/owner boundaries. It must not:

- compute ready queues or dependency ordering;
- run phase services before scheduler dispatch;
- duplicate cache compatibility or proof-reuse decisions;
- treat `SchedulerResult.output_refs` or synthetic outputs as real
  `mizar-ir` handles;
- publish artifact-boundary events without the artifact owner;
- convert diagnostics to CLI/LSP presentation at the registry boundary.

If the real phase input identity, producer output, artifact token, diagnostics
bridge, or LSP bridge is unavailable, the driver records an
`external_dependency_gap` or `deferred` owner gap. It must not create a fake
phase adapter, stub producer output, provisional artifact token, fake cache
decision, proof authority, or LSP payload.

## Tests

This task adds focused Rust tests for:

- scheduler callback invocation only after scheduler-selected dispatch and in
  deterministic task order;
- callback order independence from simulated worker completion order;
- cache hits skipping the callback while still unblocking dependents;
- resource admission failures preventing callback execution;
- dispatcher mode preserving existing skipped-dependency scheduler semantics;
- callback `Blocked` outcomes blocking dependents without synthetic outputs;
- running-checkpoint cancellation preventing callback execution;
- driver consumption through `PhaseRegistry`, including multi-phase task
  service-span dispatch, registry status mapping, and cache-hit tasks not
  requiring owner dispatch inputs;
- `mizar-driver` consuming the seam through a test-local registry fixture
  without reproducing scheduler readiness, dependency ordering, cache
  decisions, or result collation;
- source guards proving the seam does not add driver, IR, cache-key,
  proof-authority, artifact-token, producer-output, or LSP authority to
  `mizar-build`.

Test-local fixture phase services are allowed only to prove the scheduler and
driver integration contract. They must not be exported or documented as real
adapters.

## Public Enum Policy

No exhaustive public enum exceptions are owned by this module.

| Enum | Decision |
|---|---|
| `SchedulerDispatchStatus` | `#[non_exhaustive]`; downstream callers must include wildcard match arms. |
