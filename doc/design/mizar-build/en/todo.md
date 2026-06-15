# mizar-build TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Full module specs do not exist yet; each is written by its own spec task
(English and Japanese in the same change) before the implementation tasks that
cite it. The initial `planner` source exists only for the bounded package-name
validation slice required by the synchronized package manifest specification.
Module names follow [internal 07](../../internal/en/07.crate_module_layout.md)
(minimum: `task_graph`, `scheduler`, `failure_state`) plus the phase-0
planning modules from architecture 00/03; the crate refines architecture 14
and 19 and internal 01.

| Module | Spec | Source | Status |
|---|---|---|---|
| planner | `planner.md` (task 2) | `src/planner.rs` | [~] |
| module_index | `module_index.md` (task 5) | `src/module_index.rs` | [ ] |
| task_graph | `task_graph.md` (task 7) | `src/task_graph.rs` | [ ] |
| scheduler | `scheduler.md` (task 9) | `src/scheduler.rs` | [ ] |
| resource | `resource.md` (task 11) | `src/resource.rs` | [ ] |
| cancel | `cancel.md` (task 13) | `src/cancel.rs` | [ ] |
| failure_state | `failure_state.md` (task 15) | `src/failure_state.rs` | [ ] |

`mizar-build` implements pipeline phase 0 (workspace planning: manifests,
lockfile, dependency graph, `BuildPlan`, module index) and the parallel
verification machinery (task graph, scheduler, resource budgets,
cancellation, blocked-task state). Scheduling is separate from semantics:
parallelism may change latency but never diagnostics order, artifact order,
proof acceptance, or reproducibility. Build requests and the phase-service
registry belong to `mizar-driver`, which depends on this crate — never the
reverse.

It is built in two waves: **wave A** (planner and module index, phase 0)
lands early because the resolver's module-index input replaces its
workspace stub with the real planner output; **wave B** (task graph,
scheduler, resources, cancellation, failure state) arrives with the
verification phases it schedules and can be developed against synthetic
tasks.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

Wave A depends on `mizar-session` and the manifest formats of spec chapter
23. Wave B is testable with synthetic tasks; its commit boundary integrates
`mizar-artifact` manifest transactions, and cache-aware scheduling
integrates `mizar-cache` when that crate exists. Architecture:
[14.parallel_verification_and_scheduling.md](../../architecture/en/14.parallel_verification_and_scheduling.md),
[19.failure_semantics.md](../../architecture/en/19.failure_semantics.md);
internal: [01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md);
spec: [23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md).

## Resolved And Open Decisions

- **Driver split: resolved by internal 00/01.** `mizar-driver` owns build
  requests, CLI/watch/LSP entry points, and the phase registry;
  `mizar-build` owns planning and scheduling and stays
  entry-point-agnostic.
- **Initial task granularity: open, resolved by task 8.** The scheduler may
  choose coarser task granularity initially (architecture 14 constraints)
  as long as architectural dependency boundaries hold; decide the first
  granularity (default candidate: per-module semantic tasks, per-VC proof
  tasks later) and record it in `task_graph.md`.
- **Cache-aware scheduling timing: open, resolved by task 18.** Cache
  lookup before task execution needs `mizar-cache` and must be callable from
  the required driver-owned `salsa` query boundary (`mizar-driver` tasks 4-5);
  until then the scheduler runs uncached with the seam in place.

## Ordered Task List

Keep `cargo test -p mizar-build` green after each task (see
[Recommended Verification](#recommended-verification)).

### Wave A: workspace planning (phase 0)

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-build` workspace member depending on `mizar-session`;
     add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: none. Spec: architecture 14, internal 01.

2. **Spec: `planner.md`.** [ ]
   - Write the planning spec (English and Japanese, no code): `mizar.pkg`,
     `mizar.workspace`, and `mizar.lock` models per spec chapter 23,
     `BuildPlan` (packages, dependency graph, toolchain, verifier and build
     config), and deterministic planning rules.
   - Deps: 1. Spec:
     [23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md),
     architecture 00 "Interface Definitions".

3. **Manifest and lockfile parsing.** [~]
   - Parse and validate package/workspace manifests and the lockfile with
     manifest-error diagnostics.
   - Package manifest `name` spelling validation has landed as the first
     bounded slice: package ids must be lowercase `snake_case`
     (`[a-z][a-z0-9]*(?:_[a-z0-9]+)*`), hyphenated spellings are rejected, and
     no hyphen-to-underscore normalization is performed. Full TOML parsing,
     workspace manifest validation, and lockfile validation remain pending.
   - Tests: valid/invalid manifest fixtures; lockfile mismatch diagnostics;
     deterministic error order.
   - Deps: 2. Spec: `planner.md`.

4. **Dependency graph resolution and `BuildPlan` production.** [ ]
   - Resolve the package dependency graph (cycles rejected, versions and
     editions checked) and produce a deterministic `BuildPlan`.
   - Tests: graph fixtures including cycles and version conflicts;
     identical inputs produce identical plans.
   - Deps: 3. Spec: `planner.md`.

5. **Spec: `module_index.md`.** [ ]
   - Write the module-index spec (English and Japanese, no code): package →
     module identity mapping per architecture 03 Step 1, the provider
     contract the resolver consumes.
   - Deps: 2. Spec: architecture 03 "Step 1".

6. **Module index construction.** [ ]
   - Build the module index from the `BuildPlan` and source layout;
     implement the resolver's provider contract so the `mizar-resolve`
     workspace stub can be replaced (closes its task-7 interim seam).
   - Tests: multi-package fixtures; alias-independent module identity;
     provider parity with the resolver stub fixtures.
   - Deps: 4, 5, `mizar-resolve` task 7. Spec: `module_index.md`.

### Wave B: task graph and scheduling

7. **Spec: `task_graph.md`.** [ ]
   - Write the task-graph spec (English and Japanese, no code): task kinds,
     versioned task identity, dependency edges (module dependencies gate
     semantic phases; VCs as fine-grained tasks), and the initial
     granularity decision.
   - Deps: 2. Spec: architecture 14 "Task Graph"/"VCs Are Fine-Grained
     Tasks", [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md).

8. **Task graph construction.** [ ]
   - Expand a `BuildPlan` into the versioned task graph; resolve and record
     the granularity decision.
   - Tests: graph expansion fixtures; dependency edges match the
     architecture boundaries; deterministic expansion.
   - Deps: 7. Spec: `task_graph.md`.

9. **Spec: `scheduler.md`.** [ ]
   - Write the scheduler spec (English and Japanese, no code): work queues,
     priority policy, batch versus watch/LSP modes, build events, and the
     deterministic-result-ordering rule (completion order is never semantic
     or artifact order).
   - Deps: 7. Spec: architecture 14 "Deterministic Result Ordering",
     [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
     "Pipeline Scheduler".

10. **Scheduler core.** [ ]
    - Implement worker pools and queue execution over the task graph with
      deterministic result ordering under arbitrary completion order;
      synthetic tasks for tests.
    - Tests: shuffled completion produces identical result and event
      orders; immutable published outputs.
    - Deps: 8, 9. Spec: `scheduler.md`.

11. **Spec: `resource.md`.** [ ]
    - Write the resource-budget spec (English and Japanese, no code):
      hierarchical budgets (build → package → task), worker pool sizing,
      and external-process limits handed to ATP runners.
    - Deps: 9. Spec: architecture 14 "Resource Budgets Are Hierarchical".

12. **Resource budgets.** [ ]
    - Implement budget accounting and enforcement in the scheduler.
    - Tests: budget exhaustion queues rather than overcommits; budgets
      compose hierarchically.
    - Deps: 10, 11. Spec: `resource.md`.

13. **Spec: `cancel.md`.** [ ]
    - Write the cancellation spec (English and Japanese, no code):
      cooperative versioned cancellation tokens, snapshot invalidation for
      watch/LSP, and the no-partial-artifacts rule.
    - Deps: 9. Spec: architecture 14 "Cancellation Is Cooperative and
      Versioned".

14. **Cancellation.** [ ]
    - Implement cancellation tokens and snapshot-version invalidation;
      cancelled work never publishes outputs.
    - Tests: cancel mid-graph leaves no partial published state; stale
      snapshot versions never publish.
    - Deps: 10, 13. Spec: `cancel.md`.

15. **Spec: `failure_state.md`.** [ ]
    - Write the failure-state spec (English and Japanese, no code):
      blocked-task states, bounded failure propagation, and stable failure
      categories per architecture 19.
    - Deps: 9. Spec: architecture 14 "Failure Propagation Is Bounded",
      [19.failure_semantics.md](../../architecture/en/19.failure_semantics.md).

16. **Failure propagation.** [ ]
    - Implement blocked/failed task states with bounded propagation and
      deterministic failure reporting.
    - Tests: one failing task blocks exactly its dependents; failure order
      deterministic.
    - Deps: 10, 15. Spec: `failure_state.md`.

17. **Deterministic commit boundary.** [ ]
    - Integrate artifact commit through `mizar-artifact` manifest
      transactions: commits serialize in canonical order regardless of
      completion order.
    - Tests: shuffled completion commits identical manifests; interrupted
      commit leaves old state visible.
    - Deps: 10, `mizar-artifact` task 14. Spec:
      [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
      "Artifact Commit Boundary".

18. **Cache-aware scheduling seam.** [ ]
    - Add the cache-lookup-before-execution seam (internal 02 control flow)
      behind an interface so `mizar-cache` can plug in; uncached execution
      remains the default until then.
    - Provide the scheduler/cache seam consumed by the driver-owned `salsa`
      query boundary (`mizar-driver` tasks 4-5): the driver may skip, reuse, or
      enqueue work through this seam, while result ordering and artifact commits
      remain deterministic. `mizar-build` still does not depend on
      `mizar-driver`.
    - Tests: seam fixtures with a mock cache; hits skip execution with
      identical externally visible results.
    - Deps: 10. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
      "Cache Lookup Before Task Execution".

### Hardening and cross-cutting follow-ups

19. **Batch-build integration suite.** [ ]
    - End-to-end batch build over a small workspace through plan → graph →
      schedule → commit with the phase services available at the time
      (frontend now, semantic phases as they land).
    - Deps: 6, 17. Spec: [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
      "Batch Build".

20. **Determinism suite.** [ ]
    - Property coverage that plans, graphs, schedules, events, and commits
      are identical for identical inputs across worker counts.
    - Deps: 17. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md).

21. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum.
    - Deps: 16. Spec: all module specs.

22. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 21. Spec: all module specs and this TODO.

23. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-build/en/` with its Japanese companion and
      synchronize content.
    - Deps: 22. Spec: repository documentation policy.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-build
cargo clippy -p mizar-build --all-targets -- -D warnings
```

For tasks that touch the resolver provider or the commit boundary, also run:

```text
cargo test -p mizar-resolve
cargo test -p mizar-artifact
```

Check the task off here once tests pass.

## Notes

- Scheduling must not change verified semantics, diagnostic order, artifact
  ordering, or proof acceptance; completion order is never used as semantic
  or artifact order.
- All task outputs are immutable once published to dependents; cancelled or
  failed work never publishes.
- `mizar-driver` consumes this crate for batch/watch/LSP entry points;
  keep this crate free of CLI and protocol concerns.
- Cache hits must satisfy the same validation rules as a clean build
  (cache is optimization, not authority — `mizar-cache`'s contract).
