# mizar-build Source/Spec Correspondence Audit

> Canonical language: English. Japanese companion:
> [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md).

## Scope

Task 22 audits `mizar-build` after task 21. It traces the public API families
and promised behavior in the crate plan, TODO, and module specs to source and
tests. Task 27 extends the audit for the scheduler-selected phase dispatch
callback seam needed by the mizar-driver closeout item DRIVER-G-011.

The audited design inputs are [00.crate_plan.md](./00.crate_plan.md),
[todo.md](./todo.md), [planner.md](./planner.md),
[module_index.md](./module_index.md), [task_graph.md](./task_graph.md),
[scheduler.md](./scheduler.md), [resource.md](./resource.md),
[cancel.md](./cancel.md), [failure_state.md](./failure_state.md),
[artifact_commit.md](./artifact_commit.md), [cache_seam.md](./cache_seam.md),
[phase_dispatch.md](./phase_dispatch.md), [batch_integration.md](./batch_integration.md), and
[determinism_suite.md](./determinism_suite.md). Task 25 re-runs the
architecture-22 portion after task 24 and additionally audits
[incremental_parallel_equivalence.md](./incremental_parallel_equivalence.md).
Task 26 re-runs the source-layout portion for moved unit-test modules and
additionally audits [module_boundary_refactor_gate.md](./module_boundary_refactor_gate.md).
Task 27 re-runs the scheduler, driver-boundary, and bilingual audit portions
for the new dispatch callback design.

Classification result:

- `spec_gap`: none found for implemented `mizar-build` behavior.
- `test_gap`: BUILD-G-016 remains open for direct standalone coverage of
  `sorted_manifest_updates`; commit-order behavior is already covered through
  `commit_manifest_updates`. DISPATCH-G001 / BUILD-G-011 is the task-27
  scheduler-selected callback coverage item and is resolved in the task-27
  worktree by the implementation and driver-consumption tests.
- `design_drift`: none found for implemented `mizar-build` behavior.
- `source_drift`: BUILD-G-018 was opened and resolved as layout-only source
  drift for oversized inline unit-test modules. DISPATCH-G001 / BUILD-G-011
  records the missing scheduler-selected callback before task 27 and is resolved
  by adding the build-owned dispatcher API without adding a `mizar-driver`
  dependency.
- `source_undocumented_behavior`: none found for implemented `mizar-build`
  behavior.
- `test_expectation_drift`: none found for implemented `mizar-build`
  behavior.
- `boundary_violation`: none found.
- `repo_metadata_conflict`: none found.
- `external_dependency_gap`: owner-provided phase input identities, parent
  output handles, diagnostics/output publisher handles, IR producer outputs,
  artifact publication tokens, cache compatibility decisions, proof authority,
  artifact serialization, and LSP conversion remain recorded below. BUILD-G-019
  classifies the owner inputs needed by real phase dispatch; they must not be
  fabricated in `mizar-build`.

## Public API Correspondence

| Spec | Public API checked | Source | Test evidence | Finding |
|---|---|---|---|---|
| [00.crate_plan.md](./00.crate_plan.md), [todo.md](./todo.md) | Public modules `planner`, `module_index`, `task_graph`, `scheduler`, `resource`, `cancel`, `failure_state`, `artifact_commit`, and `cache_seam`; scheduler-owned dispatch callback API | `crates/mizar-build/src/lib.rs`, `crates/mizar-build/src/scheduler.rs` | `tests/lint_policy.rs` guards workspace lint opt-in and the public enum policy; crate tests exercise each public module and the task-27 dispatcher seam. | No finding. |
| [planner.md](./planner.md) | Manifest, lockfile, dependency graph, package plan, config, version constraint, diagnostic, validation, and `BuildPlan` APIs; `parse_package_manifest`, `parse_workspace_manifest`, `parse_lockfile`, `validate_package_manifest`, `validate_package_id_spelling`, `validate_lockfile_for_workspace`, `produce_build_plan`, `is_lowercase_snake_case_package_id` | `crates/mizar-build/src/planner.rs`; private unit tests in `crates/mizar-build/src/planner/tests.rs` | Planner unit tests cover valid and invalid package/workspace/lockfile inputs, deterministic diagnostic ordering, package-id spelling, lockfile consistency, dependency graph cycles, version conflicts, dev-dependency selection, unsupported editions, and shuffled-input `BuildPlan` equality. | No finding. |
| [module_index.md](./module_index.md) | `ModuleIndex`, package/namespace/module/dependency-summary entries, source layout provider, build-side provider traits, diagnostics, provider errors, and `build_module_index` | `crates/mizar-build/src/module_index.rs`; private unit tests in `crates/mizar-build/src/module_index/tests.rs`; downstream wildcard consumption in `crates/mizar-resolve/src/module_index.rs` | Module-index unit tests cover multi-package workspaces, dependency summaries, alias-independent module identity, deterministic source discovery, duplicate/conflict diagnostics, provider lookup, provider errors, and dependency artifact validation. `cargo test -p mizar-resolve` verifies the downstream provider seam. | No finding. |
| [task_graph.md](./task_graph.md) | `TaskGraphVersion`, `TaskGraphInput`, `TaskGraph`, `BuildTask`, `TaskId`, task kinds, phases, work units, dependency coverage, resource/priority classes, module dependency overlays, VC/backend/evidence IDs, diagnostics, and `build_task_graph` | `crates/mizar-build/src/task_graph.rs`; private unit tests in `crates/mizar-build/src/task_graph/tests.rs` | Task-graph unit tests cover deterministic IDs, package/module ordering, phase expansion, dependency-summary inputs, package and module dependency edges, coverage diagnostics, explicit VC descriptors, duplicate/cycle rejection, placeholder absence, and non-authority boundaries. | No finding. |
| [scheduler.md](./scheduler.md), [phase_dispatch.md](./phase_dispatch.md) | `SchedulerInput`, `SchedulerRun`, task state/result/event records, modes, cache policy, synthetic outcomes, output/diagnostic refs, queues, order keys, diagnostics, `CancellationPolicy` re-export, `run_scheduler`, `run_scheduler_with_dispatcher`, `SchedulerTaskDispatcher`, `SchedulerDispatchTask`, `SchedulerDispatchOutcome`, and `SchedulerDispatchStatus` | `crates/mizar-build/src/scheduler.rs`; private unit tests in `crates/mizar-build/src/scheduler/tests.rs` | Scheduler unit tests cover readiness transitions, queues, priority hints, completion-order independence, cache hit/miss scheduling, resource admission, cancellation, failure/block propagation, dispatch callback ordering, cache-hit callback skip, dispatch-block propagation, running-checkpoint cancellation before callback, event/result collation, immutable synthetic outputs, and placeholder absence. | No finding. |
| [resource.md](./resource.md) | `ResourceBudget`, `TaskResourceRequest`, request units, admission status, admission records, telemetry, scopes, `ResourceManager`, and `resource_queue_rank` | `crates/mizar-build/src/resource.rs` | Resource tests cover hierarchical scopes, delayed admission without overcommit, impossible requests, idempotent duplicate admission, release accounting, worker/memory pools, ATP portfolio/process separation, backend fanout, and deterministic telemetry. | No finding. |
| [cancel.md](./cancel.md) | `CancellationGeneration`, policy/state/token/decision records, reasons, decisions, checkpoints, freshness and publication guards, and graph-ordered decision helpers | `crates/mizar-build/src/cancel.rs`; scheduler integration in `src/scheduler.rs` | Cancellation tests cover monotonic generations, snapshot supersession, pending/ready/running/completed decisions, checkpoint decisions, obsolete result discard, idempotent requests, canonical ordering, scheduler cancellation, and resource release. | No finding. |
| [failure_state.md](./failure_state.md) | `FailureCategory`, `BlockReason`, `FailureSourceOrder`, `BuildFailureRecord`, `BlockedTaskRecord`, synthetic failure categories, and stable sort keys | `crates/mizar-build/src/failure_state.rs`; scheduler integration in `src/scheduler.rs` | Failure-state and scheduler tests cover direct failures, bounded blockers, nearest blockers, independent failures, deterministic ordering, cancelled versus failed states, and absence of inherited producer outputs. | No finding. |
| [artifact_commit.md](./artifact_commit.md) | `ManifestCommitRequest`, `ScheduledManifestUpdate`, `ManifestCommitSummary`, `CommittedModuleUpdate`, `ArtifactCommitError`, `commit_manifest_updates`, and `sorted_manifest_updates` | `crates/mizar-build/src/artifact_commit.rs` | Artifact-commit tests cover shuffled update determinism through `commit_manifest_updates`, freshness rejection preserving previous manifests, `mizar-artifact` manifest error propagation, no publication-authority placeholders, and commit-order integration from batch/determinism suites. | BUILD-G-016: standalone public-helper coverage for `sorted_manifest_updates` is missing. |
| [cache_seam.md](./cache_seam.md) | `CacheSchedulingPlan`, task decisions, hit/miss/unavailable/no-key outcomes, validated hit payloads, cache output/diagnostic refs, plan diagnostics, and `validated_decision_map` | `crates/mizar-build/src/cache_seam.rs`; scheduler integration in `src/scheduler.rs` | Cache-seam and scheduler tests cover externally supplied validated hits, clean-equivalent scheduler payloads, fallback execution, disabled policy behavior, duplicate/unknown decisions, deterministic hit payload collation, and absence of local cache-key/fingerprint/proof-reuse logic. | No finding. |
| [phase_dispatch.md](./phase_dispatch.md) driver-consumption contract | Build-owned scheduler callback can be consumed by `mizar-driver` through a registry-backed dispatcher without driver-side readiness, dependency ordering, resource admission, cache-decision consumption, or cancellation semantics | `crates/mizar-driver/src/driver.rs`, `crates/mizar-driver/src/driver/scheduler.rs`; driver contract tests | Driver tests show missing owner dispatch inputs remain a dispatch gap and owner-provided inputs allow registry-backed phase execution from the scheduler-selected callback. Source guards keep scheduler semantics out of `mizar-driver` front-door code and fake outputs/tokens out of the seam. | No finding. |
| [batch_integration.md](./batch_integration.md) | Available batch path over planner, module index, task graph, scheduler, cache seam, and artifact commit | `crates/mizar-build/tests/batch_integration.rs` | Integration tests cover plan -> graph -> schedule -> commit, cache hit non-authority, and explicit external-gap placeholder guards. | No finding. |
| [determinism_suite.md](./determinism_suite.md) | Cross-boundary determinism for implemented seams | `crates/mizar-build/tests/determinism_suite.rs` | Determinism tests cover shuffled logical inputs, scheduler worker/priority/completion variants, cache hit/miss placement, shuffled manifest updates, and boundary placeholder absence. | No finding. |
| [incremental_parallel_equivalence.md](./incremental_parallel_equivalence.md) | Implemented-seam architecture-22 equivalence gate | `crates/mizar-build/tests/determinism_suite.rs` | `clean_and_incremental_parallel_runs_publish_identical_visible_projection` compares clean sequential, clean parallel, incremental sequential, and incremental parallel visible projections over the same snapshot, including scheduler output refs, manifest hashes, summary refs, proof-witness entries, diagnostics, failure records, and blocked records. `superseded_or_stale_incremental_results_do_not_publish_current_artifacts` checks stale validated hits and superseded snapshots before current publication. | No finding. |
| All module specs | Public enum forward-compatibility policy for every current public enum | `#[non_exhaustive]` attributes in `crates/mizar-build/src/**/*.rs`; downstream wildcard arm in `crates/mizar-resolve/src/module_index.rs` | `tests/lint_policy.rs` scans source, checks exact EN/JA policy rows, and requires downstream-compatible public enum declarations. `cargo test -p mizar-resolve` verifies the current downstream build-side consumer. | No finding. |

## Behavior Correspondence

| Promised behavior | Source/test correspondence | Finding |
|---|---|---|
| Phase-0 planning is deterministic and rejects invalid manifests, lockfiles, dependency cycles, version conflicts, unsupported editions, and non-canonical paths. | `planner.rs` parser/validator/resolver source plus focused planner unit tests. | No finding. |
| Module identity is package-scoped, alias-independent, and provider-accessible without source/snapshot identity allocation. | `module_index.rs` source plus provider and fixture tests. | No finding. |
| Task graph identity, correctness edges, dependency coverage, and VC descriptor handling are deterministic and separate from proof/cache authority. | `task_graph.rs` source plus graph expansion, edge, coverage, and boundary tests. | No finding. |
| Scheduler readiness, queue routing, priority hints, cache hits, cancellation, failures, and resource admission affect execution latency/state only, not canonical semantic or artifact ordering. | `scheduler.rs`, `resource.rs`, `cancel.rs`, `failure_state.rs`, `cache_seam.rs`, and integration/determinism tests. | No finding. |
| Resource budgets queue rather than overcommit, release exactly once, separate ATP portfolio coordination from backend process slots, and never mint publication or proof authority. | `resource.rs` and scheduler resource-admission tests. | No finding. |
| Cancellation is cooperative, versioned, deterministic, and prevents stale or partial current publication without becoming proof failure or cache validation. | `cancel.rs` and scheduler cancellation tests. | No finding. |
| Failure propagation records direct failures and bounded blocked states without copying producer outputs, fabricating diagnostics, or collapsing cancellation into proof failure. | `failure_state.rs` and scheduler failure tests. | No finding. |
| Artifact commit consumes `mizar-artifact` manifest transactions and caller-supplied entries only; it does not own artifact schema, producer payloads, tokens, or proof authority. | `artifact_commit.rs`, `mizar-artifact` tests, and batch/determinism suites. | No finding. |
| Cache-aware scheduling consumes externally validated cache decisions only; cache hit may skip execution but never upgrades semantic acceptance, proof authority, or trusted status. | `cache_seam.rs`, scheduler cache tests, batch cache test, determinism suite, and `mizar-cache` adjacent tests. | No finding. |
| Scheduler-selected phase dispatch runs only after build-owned readiness, dependency ordering, cache miss/fallback, cancellation-before-start, and resource admission. It returns scheduler states only and does not move phase semantics, proof, cache compatibility, artifact, diagnostics identity, or LSP authority into `mizar-build`. | `scheduler.rs` dispatch API plus task-27 scheduler tests and driver registry-consumption tests. | No finding. |
| Architecture-22 implemented-seam equivalence preserves externally visible projection across clean sequential, clean parallel, incremental sequential, and incremental parallel scheduler runs, and stale/superseded work does not publish current artifacts. | Task-24 additions to `tests/determinism_suite.rs` compare visible projections and stale-publication guards; task-25 follow-up audit is recorded in [architecture_22_follow_up_audit.md](./architecture_22_follow_up_audit.md). | No finding. |
| Module-boundary source layout keeps public modules stable while moving large inline unit-test bodies into private child modules. | Task-26 private test modules are recorded in [module_boundary_refactor_gate.md](./module_boundary_refactor_gate.md); `cargo test -p mizar-build` confirms the moved unit tests still exercise the same parent modules. | No finding. |
| The crate has no dependency on `mizar-driver` and does not implement driver-owned requests, sessions, event streams, registry semantics, phase input identity production, or `salsa` query storage. The scheduler exposes a callback seam that the driver may consume from the driver-to-build direction. | `Cargo.toml` dependency tree, `scheduler.rs`, `tests/batch_integration.rs`, task-27 scheduler tests, and driver boundary guard tests. | No finding. |

## Remaining Gaps

No new blocking/high `spec_gap`, `test_gap`, `design_drift`,
`source_undocumented_behavior`, `test_expectation_drift`, `boundary_violation`,
or `repo_metadata_conflict` remains after task 27 source/test implementation.
BUILD-G-018 was opened and
resolved in task 26 as a layout-only `source_drift` by moving large inline
unit-test bodies into private child modules. DISPATCH-G001 / BUILD-G-011 was
opened for the missing scheduler-selected dispatch seam and is resolved by task
27 source and tests.

Existing non-blocking follow-up gaps remain:

| Gap | Class | Disposition |
|---|---|---|
| BUILD-G-016 | `test_gap` | `sorted_manifest_updates` is a public helper and is exercised indirectly through `commit_manifest_updates`, but lacks a direct focused test for standalone canonical ordering. Add that focused test in a later artifact-commit hardening slice before claiming method-level helper coverage. |
| BUILD-G-002 | `external_dependency_gap` | Real driver-owned requests, sessions, event streams, cache-query adapter, and `salsa` query storage are not implemented in `mizar-build`. `mizar-driver` exists and consumes build from the driver-to-build direction; `mizar-build` remains entry-point agnostic and has no `mizar-driver` dependency. |
| BUILD-G-003 / BUILD-G-012 | `external_dependency_gap` | Real sealed output handles, output storage, and snapshot-handle rehydration are owned outside `mizar-build`. Implemented scheduler tests use synthetic immutable output refs only, and the dispatch callback does not fabricate IR handles. |
| BUILD-G-004 / BUILD-G-013 | `external_dependency_gap` | Real producer artifact publication tokens and full phase-15 emission inputs remain unavailable. `mizar-build` consumes caller-supplied `mizar-artifact` entries and does not invent tokens. |
| BUILD-G-006 / BUILD-G-015 | `external_dependency_gap` | Full real resolver/checker/VC/proof/kernel/driver integration and clean/incremental/parallel equivalence remain unavailable until external seams exist. Task 24 owns the implemented-seam equivalence gate. |
| BUILD-G-009 | `external_dependency_gap` | Driver-owned cache query integration, `mizar-ir` output rehydration, and producer publication tokens remain absent. The cache seam continues to consume caller-supplied decisions only. |
| BUILD-G-017 | `external_dependency_gap` | Task 24/25 covers architecture-22 equivalence for implemented `mizar-build` seams. Full real driver sessions, real IR output rehydration, producer-owned artifact projection, and producer publication tokens remain unavailable and must not be stubbed. |
| BUILD-G-019 / DISPATCH-G002 / DISPATCH-G003 | `external_dependency_gap` | Owner-provided phase input identities, parent output handles, diagnostics/output publisher handles, producer outputs, and artifact publication tokens are required for full real phase output publication. Task 27 exposes only the scheduler-selected dispatch callback and records missing owner inputs as gaps; it does not create fake adapters, stub outputs, or provisional tokens. |

## Verification

Task 22 is documentation-only. Verification is recorded with the task commit and
includes documentation diff checks. Task 25 is also documentation-only and
records the post-task-24 audit re-run; its verification is the documentation
diff checks for the task commit. The task-24 commit already verified the
implemented equivalence gate with `cargo test -p mizar-build`, clippy, fmt, and
the available adjacent cache/artifact/VC/proof crate tests. Task 26 changes
Rust source layout only; its verification passed `cargo test -p mizar-build`,
`cargo clippy -p mizar-build --all-targets -- -D warnings`, `cargo fmt
--check`, and the additional available adjacent checks for `mizar-cache`,
`mizar-artifact`, `mizar-vc`, and `mizar-proof`. Task 27 verification is
recorded with the task commit and includes focused `mizar-build` scheduler
tests, `mizar-driver` contract tests, adjacent seam checks where available, fmt,
clippy, full workspace tests, and diff checks.
