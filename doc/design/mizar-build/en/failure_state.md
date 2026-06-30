# mizar-build Failure State

> Canonical language: English. Japanese companion:
> [../ja/failure_state.md](../ja/failure_state.md).

## Purpose

This document specifies the failure-state contract owned by `mizar-build`.

Failure state is a scheduling, propagation, and reporting boundary. It lets the
build scheduler record direct task failures, keep blocked work explicit, and
continue independent work without changing source semantics, proof authority,
cache validation, or artifact publication authority.

## Context

- [architecture 14](../../architecture/en/14.parallel_verification_and_scheduling.md)
- [architecture 19](../../architecture/en/19.failure_semantics.md)
- [architecture 22](../../architecture/en/22.incremental_verification_contract.md)
- [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
- [scheduler.md](./scheduler.md)
- [task_graph.md](./task_graph.md)
- [resource.md](./resource.md)
- [cancel.md](./cancel.md)

## Scope

`failure_state` owns:

- build-side records for direct task failure, blocked task state, and the
  task/snapshot context needed to collate those records deterministically;
- bounded propagation of blocking terminal states over task-graph correctness
  edges;
- stable build-side block reasons that distinguish "failed" from "not run
  because a dependency failed, was cancelled, or lacked required coverage";
- deterministic ordering of failure and blocked-work reports under arbitrary
  worker count and completion order;
- the no-output-publication rule for failed or blocked work.

`failure_state` does not own:

- parser, resolver, checker, VC, ATP, kernel, proof-policy, or artifact
  semantic failure detection;
- the diagnostic registry, diagnostic rendering, LSP publication, or failure
  snapshot storage owned by future `mizar-diagnostics` / LSP surfaces;
- `mizar-driver` requests, sessions, watch/LSP event streams, phase registries,
  or `salsa` query boundaries;
- `mizar-ir` output handles, sealed blobs, or snapshot rehydration;
- `mizar-cache` `CacheKey` construction, dependency fingerprint construction,
  cache-store compatibility checks, or proof-reuse validation;
- artifact manifest transaction internals, producer publication tokens, proof
  witness publication, or artifact writes;
- proof search, proof acceptance, kernel trust, backend winner selection, or
  trusted-status promotion.

## Gap Classification

| ID | Class | Evidence | Action |
|---|---|---|---|
| FAIL-G001 | `design_drift` | `todo.md` required `failure_state.md`, but no module spec existed before task 15. | Task 15 adds this spec and its Japanese companion. |
| FAIL-G002 | `source_drift` / `test_gap` | `src/failure_state.rs` and focused failure-state tests were absent before task 16. | Task 16 adds build-side records, bounded propagation helpers, deterministic ordering, scheduler integration, and focused tests. |
| FAIL-G003 | `external_dependency_gap` | This checkout has no `mizar-diagnostics` crate for the stable diagnostic registry or rendered failure snapshots. | Keep failure records synthetic/build-side; do not invent diagnostic-registry APIs in `mizar-build`. |
| FAIL-G004 | `external_dependency_gap` | `mizar-driver` and real phase-service failure emission are absent in this checkout. | Consume future phase failure records by value; do not add driver dependencies or placeholder phase-service APIs. |
| FAIL-G005 | `external_dependency_gap` | `mizar-ir` output storage and phase output handles are absent. | Do not publish failed or blocked output handles; keep tests synthetic until real output handles exist. |
| FAIL-G006 | `deferred` | Fine-grained degraded-mode permissions for non-semantic follow-up work are not implemented before task 16. | Specify the permission boundary here; task 16 may implement only the explicit cases covered by tests. |
| FAIL-G007 | `deferred` | Cache-aware scheduling is task 18 and `mizar-cache` owns cache validation. | Cache miss or default cache error handling is not a semantic failure; explicit cache-required failure remains future work. |

## Data Model

The following shapes define the contract, not necessarily final Rust names:

```rust
enum FailureCategory {
    ParseError,
    ResolveError,
    TypeError,
    OverloadAmbiguity,
    ClusterLoop,
    AtpTimeout,
    CertificateRejection,
    KernelRejection,
    BuildInfrastructure,
}

enum BlockReason {
    DependencyFailed,
    DependencyBlocked,
    DependencyCancelled,
    MissingDependencyCoverage,
    ImpossibleResourceRequest,
    NoSchedulablePath,
}

struct FailureSourceOrder {
    package_id: Option<String>,
    module_path: Option<String>,
    source_range: Option<String>,
}

struct BuildFailureRecord {
    task_id: TaskId,
    snapshot: BuildSnapshotId,
    category: FailureCategory,
    phase: PipelinePhase,
    source_order: Option<FailureSourceOrder>,
    severity_rank: usize,
    canonical_order: SchedulerOrderKey,
    diagnostic_code: String,
    stable_detail_key: String,
    rejection_reason: Option<String>,
}

struct BlockedTaskRecord {
    task_id: TaskId,
    snapshot: BuildSnapshotId,
    // Empty for direct scheduler blocks such as impossible resources or no
    // remaining schedulable path.
    blocked_by: Vec<TaskId>,
    reason: BlockReason,
    canonical_order: SchedulerOrderKey,
}
```

`FailureCategory` consumes architecture 19's stable producer phase-level
categories and adds the build-side `BuildInfrastructure` extension for
scheduler/resource or artifact-boundary failures that have no producer semantic
category. `mizar-build` may store and sort those categories, but producer
crates own their semantic detection. `BlockReason` is scheduler-local: it
explains why a task did not run and must not be reported as semantic failure of
that task. Direct failure records carry producer diagnostic ordering metadata
when it is available, plus the scheduler order fallback for synthetic or
build-side-only failures.

## Direct Failure Versus Blocked Work

A direct failure means the task ran, or an owning phase reported a failed
outcome for that task, and the failure belongs to that task's phase boundary.

Blocked work means the task did not run because either at least one required
correctness dependency could not provide an output for the current run, or the
scheduler determined directly that the task cannot produce a valid result in
this run. Direct scheduler blocks include missing dependency coverage,
impossible resource requests, and a remaining pending task with no schedulable
path. Dependency-caused blocked work is not a second copy of the predecessor
failure.

Rules:

1. `Failed` records may carry failure diagnostics, stable category metadata,
   and structured rejection detail.
2. `Blocked` records carry zero or more `blocked_by` task ids plus a
   build-side `BlockReason`, but they do not copy producer outputs or producer
   failure diagnostics into the blocked task result. Empty `blocked_by` is
   valid only for direct scheduler blocks.
3. `Cancelled` is not `Failed`. It may block correctness dependents, but it
   does not prove semantic failure or produce proof diagnostics.
4. `Skipped` is not failure. It is unblocking only for a statically declared
   conditional subgraph whose parent made the child unnecessary.
5. `CacheHit` is not proof evidence. If cache validation is unavailable,
   incomplete, or unsupported, scheduling must treat the lookup as a miss
   unless a future explicit cache-required mode says otherwise.

## Bounded Propagation

Failure propagation follows task-graph correctness edges only.

- A direct failure blocks dependents that require the failed task's outputs.
- Cancelled or superseded work is recorded as `Cancelled`; if it blocks a
  correctness dependent, the dependent's block reason is
  `DependencyCancelled`.
- A blocked task may block its own dependents, but each blocked record keeps
  the nearest blocking predecessor set rather than duplicating the whole
  upstream chain.
- Direct scheduler blocks, such as missing dependency coverage, impossible
  resource requests, or no remaining schedulable path, record their
  `BlockReason` without inventing a predecessor failure.
- Independent tasks continue whenever their required dependencies remain
  successful, validated cache hits, or explicitly unblocking skips.
- A failed VC does not hide independent VC failures in the same module.
- ATP timeout leaves an obligation open unless another accepted kernel-evidence
  result is available; it never becomes proof acceptance.
- Artifact I/O failure blocks affected commit/documentation work, but it does
  not retroactively invalidate proof results already computed in memory.

The scheduler may choose to stop after all currently useful work is terminal,
but it must not silently omit blocked tasks. A user or test must be able to
distinguish "this task failed" from "this task was not run because these
dependencies failed, were cancelled, or lacked required coverage."

## Stable Failure Categories

`failure_state` uses architecture 19's phase categories as the stable
machine-readable classification for producer-owned direct failures, plus the
build-side `build_infrastructure` extension for non-semantic scheduler/resource
or artifact-boundary failures:

| Category | Build-side meaning |
|---|---|
| `parse_error` | Source text could not be parsed into usable syntax for dependent semantic work. |
| `resolve_error` | Module, namespace, label, import, or symbol resolution failed. |
| `type_error` | Type checking or registration/type prerequisite checking failed. |
| `overload_ambiguity` | Overload/refinement selection could not choose a sound unique candidate. |
| `cluster_loop` | Registration or cluster expansion hit a cycle or bounded saturation failure. |
| `atp_timeout` | ATP/backend work ended without accepted evidence due to timeout or equivalent non-acceptance. |
| `certificate_rejection` | Evidence/certificate material was malformed, unsupported, or failed evidence-level validation. |
| `kernel_rejection` | Kernel replay/checking rejected the proof evidence. |
| `build_infrastructure` | Build-side setup, scheduling, resource, artifact I/O, or documentation boundary failure without a producer-owned semantic category. |

More specific rejection reasons, such as `malformed_certificate`,
`unsupported_certificate_format`, `invalid_substitution`, or
`invalid_sat_refutation`, remain structured detail on the record. They refine
diagnostics but do not replace the phase-level category used for propagation
and stable ordering.

`build_infrastructure` is a build-side extension for scheduler/resource or
artifact-boundary failures that lack producer semantic metadata. It is not a
proof rejection, semantic acceptance result, or trusted-status input.

Until real producer failure records are available, synthetic scheduler tests
project task kinds to categories as follows:

| Task kind | Synthetic category |
|---|---|
| `PackageResolve`, `ArtifactCommit`, `DocumentationExtract` | `build_infrastructure` |
| `SourceLoad`, `Frontend` | `parse_error` |
| `ModuleResolve` | `resolve_error` |
| `CheckAndElaborate`, `VcGenerate`, `VcDischarge` | `type_error` |
| `AtpSolve`, `BackendRun` | `atp_timeout` |
| `KernelCheck` | `kernel_rejection` |

Synthetic build-side records derive `stable_detail_key` from stable task
identity plus diagnostic code. Diagnostic message text is not an ordering key.

## Deterministic Ordering

Failure and blocked-work reports are sorted by stable inputs only. Completion
order, worker id, resource-admission timing, cache lookup timing, backend
runtime, and temporary path spelling are never ordering inputs.

The canonical build-side ordering key is:

1. `BuildSnapshotId`;
2. source package id and module path when known;
3. source range when known;
4. pipeline phase rank;
5. severity;
6. diagnostic code;
7. stable detail key;
8. task graph index;
9. `TaskId`;
10. structured rejection reason or block reason.

If source coordinates are unavailable, task graph order and `TaskId` provide
the deterministic fallback. Blocked records use the blocked task's own order,
sort `blocked_by` task ids canonically, and preserve direct scheduler block
reasons even when `blocked_by` is empty. Direct failure records use producer
`source_order` and `severity_rank` when supplied; otherwise they fall back to
their scheduler `canonical_order` and `TaskId`.

## Publication And Authority

Failure state is not proof authority.

- A `Failed` task may publish failure diagnostics, but it must not publish
  phase outputs, cache records, artifact drafts, proof witnesses, or artifact
  commits to dependents.
- A `Blocked` task publishes only blocked-state metadata and optional
  scheduler diagnostics; it must not publish producer outputs or inherited
  failure diagnostics as if it had run.
- A failed or blocked task never upgrades proof trust, semantic acceptance,
  backend evidence, cache reuse, or artifact records.
- Theorem proving success must never depend on ignoring an earlier
  deterministic predecessor failure.
- Resource exhaustion and timeouts are non-acceptance outcomes. They may be
  diagnostics, but they are not proof rejection unless the owning proof/kernel
  phase emits such a category.

Artifact and cache records remain separate authorities owned by
`mizar-artifact` and `mizar-cache`. `mizar-build` consumes their scheduling
outcomes only at documented seams.

## Task 16 Coverage

Task 16 adds focused Rust coverage for:

- one failed task blocks exactly the correctness dependents that require its
  outputs;
- blocked records keep nearest `blocked_by` dependencies and do not duplicate
  the full upstream chain;
- direct scheduler blocks for missing coverage, impossible resources, and no
  schedulable path are represented without inventing dependency failures;
- independent failures and independent successful tasks remain visible after a
  failure elsewhere in the graph;
- failure and blocked-record ordering is deterministic under shuffled ready
  order, completion order, and worker count;
- failed tasks carry diagnostics but no output references;
- blocked and cancelled tasks carry no synthetic producer outputs or inherited
  producer diagnostics;
- cancellation remains distinct from failure while still blocking correctness
  dependents when required;
- cache miss/unavailable/default error-as-miss behavior does not create
  failure categories or proof authority;
- no `mizar-driver`, `mizar-ir`, diagnostic-registry, cache-key,
  dependency-fingerprint, proof-reuse, publication-token, or proof-authority
  placeholder is introduced.

## Public Enum Policy

No exhaustive public enum exceptions are owned by this module.

| Enum | Decision |
|---|---|
| `FailureCategory` | `#[non_exhaustive]`; downstream callers must include wildcard match arms. |
| `BlockReason` | `#[non_exhaustive]`; downstream callers must include wildcard match arms. |
