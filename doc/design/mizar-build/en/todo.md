# mizar-build TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Full module specs are written by their own spec tasks (English and Japanese in
the same change) before the implementation tasks that cite them. The `planner`
and `module_index` sources now cover wave A phase-0 planning and module-index
provider work; wave B module specs and source implementation now include
`cache_seam`. Module names follow
[internal 07](../../internal/en/07.crate_module_layout.md)
(minimum: `task_graph`, `scheduler`, `cancel`, `failure_state`) plus the
phase-0 planning modules from architecture 00/03; the crate refines
architecture 14 and 19 and internal 01.

| Module | Spec | Source | Status |
|---|---|---|---|
| planner | `planner.md` (task 2) | `src/planner.rs` + private `src/planner/tests.rs` | [x] |
| module_index | `module_index.md` (task 5) | `src/module_index.rs` + private `src/module_index/tests.rs` | [x] |
| task_graph | `task_graph.md` (task 7) | `src/task_graph.rs` + private `src/task_graph/tests.rs` | [x] |
| scheduler | `scheduler.md` (task 9) | `src/scheduler.rs` + private `src/scheduler/tests.rs` | [x] |
| resource | `resource.md` (task 11) | `src/resource.rs` | [x] |
| cancel | `cancel.md` (task 13) | `src/cancel.rs` | [x] |
| failure_state | `failure_state.md` (task 15) | `src/failure_state.rs` | [x] |
| artifact_commit | `artifact_commit.md` (task 17) | `src/artifact_commit.rs` | [x] |
| cache_seam | `cache_seam.md` (task 18) | `src/cache_seam.rs` | [x] |
| phase_dispatch | `phase_dispatch.md` (task 27) | `src/scheduler.rs` dispatcher API + driver consumption contract | [x] |

`mizar-build` currently implements pipeline phase 0 (workspace planning:
manifests, lockfile, dependency graph, `BuildPlan`, module index) and owns the
planned parallel verification machinery (task graph, scheduler, resource
budgets, cancellation, blocked-task state). Scheduling is separate from
semantics: parallelism may change latency but never diagnostics order,
artifact order, proof acceptance, or reproducibility. Build requests and the
phase-service registry belong to `mizar-driver`, which depends on this crate —
never the reverse.

It is built in two waves: **wave A** (planner and module index, phase 0)
lands early because the resolver's module-index input replaces its
workspace stub with the real module-index provider built from planner output;
**wave B** (task graph, scheduler, resources, cancellation, failure state)
arrives with the verification phases it schedules and can be developed against
synthetic tasks.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

Wave A depends on `mizar-session` and the manifest formats of spec chapter
23. Wave B is testable with synthetic tasks; its commit boundary integrates
`mizar-artifact` manifest transactions, and cache-aware scheduling consumes
`mizar-cache` through the task-18 scheduler seam without reimplementing cache
internals. Architecture:
[14.parallel_verification_and_scheduling.md](../../architecture/en/14.parallel_verification_and_scheduling.md),
[19.failure_semantics.md](../../architecture/en/19.failure_semantics.md);
internal: [01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md);
spec: [23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md).

## Resolved And Open Decisions

- **Driver split: resolved by internal 00/01.** `mizar-driver` owns build
  requests, CLI/watch/LSP entry points, and the phase registry;
  `mizar-build` owns planning and scheduling and stays
  entry-point-agnostic.
- **Initial task granularity: resolved by task 7.** `task_graph.md` chooses
  module-level phase tasks through VC generation and VC-level proof tasks only
  after explicit VC descriptors are available.
- **Cache-aware scheduling timing: resolved by task 18.** Cache lookup before
  task execution consumes externally validated cache decisions through the
  build-side seam and remains callable later from the required driver-owned
  `salsa` query boundary (`mizar-driver` tasks 4-5). Until that external
  boundary exists, tests use synthetic decisions and `mizar-build` remains
  uncached by default.

## Ordered Task List

Keep `cargo test -p mizar-build` green after each task (see
[Recommended Verification](#recommended-verification)).

### Wave A: workspace planning (phase 0)

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-build` workspace member depending on `mizar-session`;
     add `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: none. Spec: architecture 14, internal 01.

2. **Spec: `planner.md`.** [x]
   - Write the planning spec (English and Japanese, no code): `mizar.pkg`,
     `mizar.workspace`, and `mizar.lock` models per spec chapter 23,
     `BuildPlan` (packages, dependency graph, toolchain, verifier and build
     config), and deterministic planning rules.
   - Deps: 1. Spec:
     [23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md),
     architecture 00 "Interface Definitions".

3. **Manifest and lockfile parsing.** [x]
   - Parse and validate package/workspace manifests and the lockfile with
     manifest-error diagnostics.
   - Package manifest `name` spelling validation landed as the first bounded
     slice and the full parsing/validation slice is now complete: package ids
     must be lowercase `snake_case` (`[a-z][a-z0-9]*(?:_[a-z0-9]+)*`),
     hyphenated spellings are rejected, no hyphen-to-underscore normalization
     is performed, TOML package/workspace manifests are validated, and existing
     lockfiles are parsed and checked for package/version/source consistency.
   - Tests: valid/invalid manifest fixtures; lockfile mismatch diagnostics;
     deterministic error order.
   - Deps: 2. Spec: `planner.md`.

4. **Dependency graph resolution and `BuildPlan` production.** [x]
   - Resolve the package dependency graph (cycles rejected, versions and
     editions checked) and produce a deterministic `BuildPlan`.
   - Tests: graph fixtures including cycles and version conflicts;
     identical inputs produce identical plans.
   - Deps: 3. Spec: `planner.md`.

5. **Spec: `module_index.md`.** [x]
   - Write the module-index spec (English and Japanese, no code): package →
     module identity mapping per architecture 03 Step 1, the provider
     contract the resolver consumes.
   - Deps: 2. Spec: architecture 03 "Step 1".

6. **Module index construction.** [x]
   - Build the module index from the `BuildPlan` and source layout; expose the
     build-side provider/accessor contract defined in `module_index.md`.
     Confirm `mizar-resolve` task 7 before attempting resolver-stub replacement:
     if that task is still open, classify resolver parity as an external
     dependency gap and do not invent resolver-owned fixtures or compatibility
     shims in `mizar-build`.
   - Completed build-side slice: `ModuleIndex`, package/namespace/module
     entries, dependency-summary-backed module entries, static source layout
     provider, deterministic diagnostics, and provider accessors. Historical
     check on 2026-06-18 found `mizar-resolve` task 7 open; R-007 has since
     landed the resolver-owned seam and workspace stub provider, so the
     build-side external dependency gap is closed without adding resolver
     fixtures or compatibility shims to `mizar-build`.
   - Tests: multi-package fixtures; alias-independent module identity;
     deterministic source-discovery order; provider parity with resolver stub
     fixtures only after `mizar-resolve` task 7 supplies that seam.
   - Deps: 4, 5. Resolver-stub replacement additionally depends on
     `mizar-resolve` task 7. Spec: `module_index.md`.

### Wave B: task graph and scheduling

7. **Spec: `task_graph.md`.** [x]
   - Wrote the task-graph spec (English and Japanese, no code): task kinds,
     versioned task identity, dependency edges (module dependencies gate
     semantic phases; VCs as fine-grained tasks), dependency-coverage
     handling, and the initial granularity decision.
   - Deps: 2. Spec: architecture 14 "Task Graph"/"VCs Are Fine-Grained
     Tasks", [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md).

8. **Task graph construction.** [x]
   - Expand a `BuildPlan`, `ModuleIndex`, dependency overlay, and explicit VC
     descriptors into the versioned task graph.
   - Tests: graph expansion fixtures; dependency edges match the
     architecture boundaries; deterministic expansion.
   - Result: implemented `src/task_graph.rs` with deterministic task IDs,
     dependency coverage diagnostics, explicit VC/backend/kernel subgraphs,
     artifact/documentation scheduling edges, and focused unit tests.
   - Deps: 7. Spec: `task_graph.md`.

9. **Spec: `scheduler.md`.** [x]
   - Write the scheduler spec (English and Japanese, no code): work queues,
     priority policy, batch versus watch/LSP modes, build events, and the
     deterministic-result-ordering rule (completion order is never semantic
     or artifact order).
   - Result: added synchronized `scheduler.md` specs covering task states,
     ready queues, priority/collation, scheduler events, cache-aware seam
     boundaries, and deferred resource/cancel/failure/commit seams.
   - Deps: 7. Spec: architecture 14 "Deterministic Result Ordering",
     [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
     "Pipeline Scheduler".

10. **Scheduler core.** [x]
    - Implement deterministic dispatch batches and queue execution over the
      task graph with deterministic result ordering under arbitrary completion
      order; synthetic tasks for tests. Resource-budgeted worker pools remain
      tasks 11-12.
    - Tests: shuffled completion and worker-count variation produce identical
      result and event orders; immutable published outputs.
    - Result: implemented `src/scheduler.rs` with deterministic synthetic
      scheduling, queue routing, terminal/blocked states, canonical event and
      result collation, disabled cache seam behavior, synthetic cancellation,
      and focused unit tests.
    - Deps: 8, 9. Spec: `scheduler.md`.

11. **Spec: `resource.md`.** [x]
    - Write the resource-budget spec (English and Japanese, no code):
      hierarchical budgets (build → package → task), worker pool sizing,
      and external-process limits handed to ATP runners.
    - Result: added synchronized `resource.md` specs covering hierarchical
      budgets, deterministic queue admission, worker pools, ATP/backend limits,
      release accounting, telemetry, and non-authority rules.
    - Deps: 9. Spec: architecture 14 "Resource Budgets Are Hierarchical".

12. **Resource budgets.** [x]
    - Implement budget accounting and enforcement in the scheduler.
    - Tests: budget exhaustion queues rather than overcommits; budgets compose
      hierarchically through workspace/package/module/obligation/backend/commit
      scopes; terminal states release exactly once; ATP portfolio work does not
      consume backend process slots; backend fanout respects obligation and
      global process limits; worker-count changes preserve canonical
      result/event collation; admission is deterministic under shuffled
      ready/completion order; impossible requests produce stable diagnostics;
      telemetry and I/O commit permits do not create proof, cache, artifact
      publication, or trusted-status authority; no `mizar-driver`,
      `mizar-cache`, ATP OS-process, artifact publication token, or
      proof-authority placeholder is introduced.
    - Result: implemented `src/resource.rs` with modeled hierarchical budget
      accounting, deterministic admission/release telemetry, per-pool and
      per-scope limits, ATP portfolio/process separation, backend fanout, and
      commit permits; integrated scheduler admission without adding driver,
      cache, OS-process, publication-token, or proof-authority boundaries.
    - Deps: 10, 11. Spec: `resource.md`.

13. **Spec: `cancel.md`.** [x]
    - Write the cancellation spec (English and Japanese, no code):
      cooperative versioned cancellation tokens, snapshot invalidation for
      watch/LSP, and the no-partial-artifacts rule.
    - Result: added synchronized `cancel.md` specs covering versioned
      cancellation, snapshot freshness checks, cooperative checkpoints,
      no-current-publication for cancelled/obsolete work, resource-release
      handoff, and explicit non-authority/cache/artifact/driver boundaries.
    - Deps: 9. Spec: architecture 14 "Cancellation Is Cooperative and
      Versioned".

14. **Cancellation.** [x]
    - Implement cancellation tokens and snapshot-version invalidation;
      cancelled work never publishes outputs.
    - Tests: pending/ready pre-start cancellation; running cancellation at
      safe checkpoints; monotonic generation/token propagation; canonical
      cancellation-decision ordering; stale completed-result discard before
      publication; no current diagnostics, cache records, or artifact commit
      attempts from cancelled work; exactly-once resource release;
      commit-boundary behavior before/after the modeled atomic transaction
      begins; deterministic/idempotent cancellation; no driver/cache/IR/
      process/artifact-token/proof-authority placeholders.
    - Result: implemented `src/cancel.rs` with versioned cancellation policy,
      monotonic generations, tokens, canonical decisions, snapshot freshness
      checks, commit-started decisions, and scheduler integration for
      pre-start, checkpoint, and obsolete-completed cancellation without adding
      driver/cache/IR/process/artifact-token/proof-authority placeholders.
    - Deps: 10, 13. Spec: `cancel.md`.

15. **Spec: `failure_state.md`.** [x]
    - Write the failure-state spec (English and Japanese, no code):
      blocked-task states, bounded failure propagation, and stable failure
      categories per architecture 19.
    - Result: added synchronized `failure_state.md` specs covering direct
      failure records, blocked-work records, bounded propagation, stable
      categories, deterministic ordering, publication/authority boundaries,
      and task-16 coverage.
    - Deps: 9. Spec: architecture 14 "Failure Propagation Is Bounded",
      [19.failure_semantics.md](../../architecture/en/19.failure_semantics.md).

16. **Failure propagation.** [x]
    - Implement blocked/failed task states with bounded propagation and
      deterministic failure reporting.
    - Tests: one failing task blocks exactly its dependents; failure order
      deterministic.
    - Result: added `src/failure_state.rs` with direct failure records,
      blocked-task records, stable block reasons, deterministic ordering, and
      synthetic task-category projection; scheduler runs now emit
      `failure_records` and `blocked_records`, keep nearest blockers, preserve
      direct scheduler block reasons, and keep failed/blocked/cancelled work
      from publishing outputs.
    - Deps: 10, 15. Spec: `failure_state.md`.

17. **Deterministic commit boundary.** [x]
    - Integrate artifact commit through `mizar-artifact` manifest
      transactions: commits serialize in canonical order regardless of
      completion order.
    - Tests: shuffled completion commits identical manifests; interrupted
      commit leaves old state visible.
    - Result: added `src/artifact_commit.rs` with deterministic
      `ModuleArtifactEntry` staging, `mizar-artifact` `ManifestTransaction`
      consumption, freshness-check forwarding, deterministic commit records,
      and focused tests for shuffled ordering, obsolete freshness rejection,
      conflict propagation, and boundary placeholder absence.
    - Deps: 10, `mizar-artifact` task 14. Spec:
      [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
      "Artifact Commit Boundary"; `artifact_commit.md`.

18. **Cache-aware scheduling seam.** [x]
    - Add the cache-lookup-before-execution seam (internal 02 control flow)
      behind an interface so `mizar-cache` can plug in; uncached execution
      remains the default until then.
    - Provide the scheduler/cache seam consumed by the driver-owned `salsa`
      query boundary (`mizar-driver` tasks 4-5): the driver may skip, reuse, or
      enqueue work through this seam, while result ordering and artifact commits
      remain deterministic. `mizar-build` still does not depend on
      `mizar-driver`.
    - Tests: seam fixtures with synthetic caller-supplied cache scheduling
      decisions; hits skip execution with identical externally visible
      results.
    - Deps: 10. Spec: [internal 02](../../internal/en/02.artifact_store_cache_key_and_manifest.md)
      "Cache Lookup Before Task Execution"; `cache_seam.md`.
    - Completed by task 18: `cache_seam.md` and `src/cache_seam.rs` add the
      validated-hit decision boundary. `mizar-driver`, `mizar-ir`, and real
      producer publication-token integration remain `external_dependency_gap`s.

### Hardening and cross-cutting follow-ups

19. **Batch-build integration suite.** [x]
    - End-to-end batch build over a small workspace through plan → graph →
      schedule → commit with the build-side boundaries available at the time.
      Frontend-shaped tasks are scheduled with synthetic outcomes until a real
      driver-owned phase-service boundary is available to consume.
    - Task 19 scope: the fixture covers the public `mizar-build` boundaries
      available at that time and records driver/IR/producer-token owner seams
      that were not consumable by `mizar-build` as `external_dependency_gap`;
      it must not add fake driver APIs, IR handles, producer publication
      tokens, or proof authority.
    - Deps: 6, 17. Spec: [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
      "Batch Build"; `batch_integration.md`.
    - Completed by task 19: `tests/batch_integration.rs` exercises plan,
      module index, task graph, batch scheduling, and deterministic manifest
      commit through `mizar-artifact`. Real driver, IR, and producer-token
      integration remain `external_dependency_gap`s; explicit placeholder
      guards and a validated-cache-hit non-authority check cover the Task 19
      boundary.

20. **Determinism suite.** [x]
    - Property coverage that plans, graphs, schedules, events, and commits
      are identical for identical inputs across worker counts.
    - Task 20 scope: cover implemented `mizar-build` seams with table-driven
      fixtures; record driver/IR/producer-token clean/incremental owner seams
      that were not consumable by `mizar-build` as `external_dependency_gap`
      and do not add placeholders.
    - Deps: 17, 18, 19. Spec: [20.test_strategy.md](../../architecture/en/20.test_strategy.md);
      `determinism_suite.md`.
    - Completed by task 20: `tests/determinism_suite.rs` compares deterministic
      plan/index/graph projections, scheduler results/events across worker and
      priority variants, cache hit/miss commit projections, shuffled manifest
      commits, and explicit external-gap placeholder guards.

21. **Public-enum forward-compatibility policy.** [x]
    - Apply the `mizar-frontend` task-25 procedure to each public enum.
    - Completed by task 21: every current `mizar-build` public enum is
      `#[non_exhaustive]`; each owning English and Japanese module spec records
      the decision in a `Public Enum Policy` table and states that there are no
      exhaustive public enum exceptions. `tests/lint_policy.rs` guards future
      source/spec drift.
    - Deps: 16. Spec: all module specs.

22. **Source/spec correspondence audit.** [x]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Completed by task 22: `source_spec_correspondence.md` records the
      source/spec/test map for implemented public API families and behavior
      promises. It found no new blocking/high drift; BUILD-G-016 records one
      non-blocking public-helper `test_gap`, and existing driver, IR,
      producer-token, and full real clean/incremental integration gaps remain
      `external_dependency_gap`.
    - Deps: 21. Spec: all module specs and this TODO.

23. **Bilingual documentation sync audit.** [x]
    - Compare each English canonical document under
      `doc/design/mizar-build/en/` with its Japanese companion and
      synchronize content.
    - Completed by task 23: `bilingual_documentation_synchronization.md`
      records the paired-file audit. All English canonical design docs have
      same-named Japanese companions; no deferred companion updates remain,
      BUILD-G-016 stays a non-blocking `test_gap`, and existing driver, IR,
      producer-token, and full real clean/incremental integration gaps remain
      `external_dependency_gap`.
    - Deps: 22. Spec: repository documentation policy.

24. **Incremental/parallel equivalence gate.** [x]
    - Add the scheduler-level regression gate for architecture 22: clean
      sequential, clean parallel, incremental sequential, and incremental
      parallel execution over the same `BuildSnapshot` and verifier policy must
      commit identical published artifacts, interface hashes,
      dependency-facing summaries, proof acceptance, and canonical diagnostics.
      Progress or build-event timing may differ when cache hits skip work, but
      event consumers must not observe stale publications as current results.
    - Tests: randomized ready-task scheduling and worker counts; synthetic
      cache decision hit/miss timing; cancellation/supersession leaves no
      partial publication; cache misses enqueue work without changing the
      deterministic commit boundary.
    - Deps: 14, 18, 20. Spec:
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md),
      [14.parallel_verification_and_scheduling.md](../../architecture/en/14.parallel_verification_and_scheduling.md),
      [20.test_strategy.md](../../architecture/en/20.test_strategy.md).
    - Completed by task 24: `incremental_parallel_equivalence.md` records the
      implemented-seam scope and BUILD-G-017 `external_dependency_gap`.
      `tests/determinism_suite.rs` compares the externally visible projection
      for clean sequential, clean parallel, incremental sequential, and
      incremental parallel scheduler runs over the same snapshot, and checks
      that stale or superseded incremental results do not publish current
      manifest updates.

25. **Architecture-22 follow-up audit.** [x]
    - Re-run the source/spec correspondence and bilingual documentation sync
      audits for the task-24 scheduler equivalence, cancellation, and cache
      seam contract; record any remaining stale-publication or deterministic
      commit-boundary gaps as follow-up tasks.
    - Completed by task 25: `architecture_22_follow_up_audit.md` records the
      source/spec and bilingual audit re-run. `source_spec_correspondence.md`
      and `bilingual_documentation_synchronization.md` now include the task-24
      gate, BUILD-G-017, and synchronized EN/JA status. No stale-publication or
      deterministic commit-boundary gap remains unresolved above low severity.
    - Deps: 24. Spec: all module specs, this TODO, and repository
      documentation policy.

26. **Module-boundary refactor gate.** [x]
    - Before treating the crate as ready for downstream consumers, audit the
      source layout for oversized files, mixed responsibilities, and private
      helpers that should be split along the module table and spec boundaries.
      Split any review-bottleneck implementation files into private modules
      without changing public APIs, diagnostics, deterministic renderings,
      artifact-facing schemas, or consumer-visible behavior.
    - After any split, update this module table/source paths as needed and
      re-run the source/spec and bilingual documentation audit scopes for the
      moved APIs. Do not mix behavior cleanup or API exposure into the move;
      those require their own spec tasks.
    - Deps: 25. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.
    - Completed by task 26: `module_boundary_refactor_gate.md` records
      BUILD-G-018 as a resolved layout-only `source_drift`. Inline unit-test
      bodies for `planner`, `module_index`, `task_graph`, and `scheduler`
      moved into private child modules without changing public exports,
      diagnostics, deterministic renderings, schemas, or behavior.

27. **Scheduler-selected real phase dispatch callback seam.** [x]
    - Expose a `mizar-build` scheduler-selected dispatch callback API that is
      separate from `SchedulerInput`, so scheduler input remains deterministic
      build data while an external dispatcher can execute the selected task.
    - `mizar-build` keeps readiness, dependency ordering, resource admission,
      cancellation checkpoints, cache-aware decision consumption, and
      deterministic collation. `mizar-driver` consumes the seam by invoking the
      phase registry only from the scheduler-selected callback.
    - Tests: focused scheduler callback tests for deterministic dispatch order,
      cache-hit skip, blocked callback propagation, and cancellation; driver
      contract tests showing registry-backed consumption without duplicating
      scheduler semantics.
    - Deps: completed `mizar-driver` closeout through D-022, existing
      registry/query boundary, and task 18 cache seam. Spec:
      `phase_dispatch.md`, `scheduler.md`, internal 01, architecture 14/22,
      mizar-driver `driver.md` and `registry.md`.
    - Forbidden: no `mizar-driver` dependency in `mizar-build`; no fake phase
      adapter, stub producer output, provisional artifact token, cache/proof
      authority, artifact serialization, diagnostics identity allocation, or
      LSP protocol conversion. Missing owner-provided phase inputs, producer
      outputs, artifact tokens, diagnostics bridge, or LSP bridge remain
      `external_dependency_gap` / `deferred`.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-build
cargo clippy -p mizar-build --all-targets -- -D warnings
```

For tasks that touch the resolver provider or the commit boundary, also run the
available adjacent crate checks:

```text
cargo test -p mizar-resolve
cargo test -p mizar-artifact
```

For the architecture-22 equivalence gate, also run the available adjacent crate
checks, and explicitly justify any missing crate such as an uncreated
`mizar-driver`:

```text
cargo test -p mizar-cache
cargo test -p mizar-artifact
cargo test -p mizar-vc
cargo test -p mizar-proof
cargo test -p mizar-driver
cargo test -p mizar-test
```

Check the task off here once tests pass.

Closeout is recorded in [crate_exit_report.md](./crate_exit_report.md). It
summarizes hard gates, deferred items, verification, human review surface, and
the next-phase handoff.

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
