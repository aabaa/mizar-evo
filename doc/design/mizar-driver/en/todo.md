# mizar-driver TODO

> Canonical language: English. Japanese companion: [../ja/todo.md](../ja/todo.md).

## Status Legend

- [ ] not started
- [~] in progress
- [x] done

## Module Implementation

Module specs land incrementally; each is written by its own spec task (English
and Japanese in the same change) before the implementation tasks that cite it.
The crate refines [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
per the ownership map of
[internal 07](../../internal/en/07.crate_module_layout.md).

| Module | Spec | Source | Status |
|---|---|---|---|
| request | `request.md` (task 2) | `src/request.rs` | [x] |
| registry | `registry.md` (task 4) | `src/registry.rs`; private helpers in `src/registry/catalog.rs` | [x] |
| driver | `driver.md` (task 7) | `src/driver.rs`; private helpers in `src/driver/{event_log,scheduler,watch}.rs`; unit tests in `src/driver/tests.rs` | [x] |
| events | `events.md` (task 9) | `src/events.rs` | [x] |
| cli | `cli.md` (task 12) | `src/cli.rs`; private rendering helpers in `src/cli/output.rs` | [x] |

Task D-006 records the `SourceFrontend` adapter readiness decision in
[frontend_adapter.md](frontend_adapter.md). It is not a module source surface;
the registry continues to classify that real adapter as an external dependency
gap until owner seams exist.

`mizar-driver` is the front door for all build modes: it maps CLI/watch/LSP
requests into `BuildRequestDraft`s, bootstraps phase 0 through the `mizar-build`
planner, creates `BuildSession`s with source and dependency snapshots,
registers phase service implementations behind the `PhaseService` trait,
submits the initial task graph to the scheduler, and publishes build events
to progress reporters and the LSP bridge. It owns no phase semantics, no
cache compatibility decisions, no artifact serialization, and no editor
protocol conversion — it wires the pieces together and stays thin.

Dependency order: `request` → `registry` → `driver` → `events` → `cli` /
watch mode.

Each task below is deliberately small — one module spec, or one behavior slice
of one module — so that a single task can be implemented, tested, and
committed autonomously without holding the rest of the crate in flight.

## Crate Prerequisites

The crate depends on `mizar-session`, `mizar-build` (planner, task graph,
scheduler), `mizar-ir` (output storage and snapshot handles),
`mizar-diagnostics` (sink and aggregation), and — through phase-service
adapters registered at the binary level — on the pipeline crates as they
land (`mizar-frontend` first). It is the last subsystem to assemble; start
it with `mizar-build` wave B. Internal:
[01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md);
spec: [23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md).

`salsa` is the required target query engine for the final driver/phase-service
orchestration layer. The syntax and parser crates stay `salsa`-free; this crate
owns the database/query boundary that wraps phase services and exposes pure
inputs/outputs to the build scheduler and cache seams.

## Resolved And Open Decisions

- **Driver/build split: resolved by internal 00/01.** Planning and
  scheduling live in `mizar-build`; this crate owns request lifecycle,
  service registry, and entry points.
- **CLI surface: open, resolved by task 12.** Decide the binary name and
  subcommand set against the build lifecycle of spec chapter 23 (default
  candidate: a single `mizar` binary with `build`/`check`/`doc`
  subcommands grown incrementally) and record the decision in `cli.md`.
- **`cache_key` purity: resolved by internal 01.** `PhaseService::cache_key`
  is a pure projection from input identities, configuration, schema
  versions, and dependency hashes; the registry enforces and tests this
  contract.
- **Salsa query boundary: specified by task 4; initial registry seam implemented
  by task 5.** `salsa` is required for the final query/cache layer.
  `registry.md` specifies how `PhaseService` inputs, outputs,
  cancellation/versioning, and cache-key intents map onto a driver-owned salsa
  database while keeping phase crates free of direct driver/query-engine
  dependencies. Real cache lookup and real phase adapters remain later owner
  seam work.

## Ordered Task List

Keep `cargo test -p mizar-driver` green after each task (see
[Recommended Verification](#recommended-verification)).

### Requests and services

1. **Crate scaffold and lint-policy guard.** [x]
   - Add the `mizar-driver` workspace member depending on `mizar-session`,
     `mizar-build`, `mizar-ir`, and `mizar-diagnostics`; add
     `tests/lint_policy.rs` mirroring the `mizar-frontend` guard.
   - Tests: lint-policy guard passes; workspace builds.
   - Deps: `mizar-build` task 10, `mizar-ir` task 8,
     `mizar-diagnostics` task 9. Spec: internal 01.
   - Completed by task D-001: added the workspace member, minimal crate
     manifest and library scaffold, driver lint-policy guard, and the
     diagnostics reverse-dependency guard exception for the driver scaffold.
     No request/session/registry/event/CLI/watch behavior or placeholder seam
     was introduced.

2. **Spec: `request.md`.** [x]
   - Write the request spec (English and Japanese, no code):
     `BuildRequest` shapes for batch/watch/LSP, `BuildSession` and its
     source/dependency snapshots, session lifecycle states, and the combined
     driver lane-current-session plus request-generation snapshot guard that
     rejects obsolete publications from superseded watch/LSP sessions.
   - Deps: 1. Spec: [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
     "Build Session".
   - Completed by task D-002: [request.md](request.md) defines request
     origins, driver-owned currentness lanes, session lifecycle states, snapshot
     capture through `mizar-session`, and obsolete-publication suppression for
     superseded watch/LSP sessions through the combined publication guard. No
     source implementation was added.

3. **`BuildRequest` and `BuildSession`.** [x]
   - Implement requests and sessions with snapshot capture through
     `mizar-session`/`mizar-ir` identities, including the combined
     lane-current-session and request-generation snapshot guard used to reject
     obsolete publications from superseded watch/LSP sessions.
   - Tests: session round-trips; identical workspace states produce
     identical snapshot ids; fresh request ids share a driver lane across
     watch/LSP generations; a superseded session is rejected even when it has
     the same snapshot id as the lane-current session.
   - Deps: 2. Spec: `request.md`.
   - Completed by task D-003: `src/request.rs` implements request drafts,
     allocated pending requests, captured sessions, lifecycle transitions,
     driver lane-current-session tracking, and publication decisions. The
     request tests cover snapshot capture through `mizar-session`, same
     canonical snapshot ids with distinct allocator-issued ids, same-snapshot
     supersession rejection, stale request-generation snapshots, stale lane
     update rejection, snapshot-creation failure context, suppressed
     publication decisions, and idempotent cancellation. No LSP protocol
     conversion, scheduler semantics, artifact publication, cache/proof
     authority, or phase semantics was added.

4. **Spec: `registry.md`.** [x]
   - Write the registry spec (English and Japanese, no code): the
     `PhaseService` trait (`phase`, `cache_key`, `execute`),
     `PhaseContext`/`PhaseResult`, the service table for phases 0-16, and
     the cache-key purity contract.
   - Define the required `salsa` integration boundary: database lifetime,
     input queries, derived phase-output queries, cancellation/snapshot-version
     interaction, and the rule that phase crates expose pure services rather
     than depending on `salsa` directly.
   - Deps: 2. Spec: [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
     "Phase Services"/"Phase Service API".
   - Completed by task D-004: [registry.md](registry.md) defines the
     deterministic phase-service registry, phase 0-16 service table,
     driver-owned salsa query boundary, cache-key purity contract,
     scheduler/cache seam handoff, and diagnostics/artifact/LSP non-ownership
     rules. Missing real phase adapters, artifact publication tokens,
     producer outputs, and LSP bridge work remain classified gaps rather than
     fake adapters or provisional APIs.

5. **Phase service registry.** [x]
   - Implement registration and lookup of phase services with
     duplicate-phase rejection and a purity test harness for `cache_key`.
   - Add the initial salsa-backed registry seam from `registry.md`: services
     execute through deterministic query adapters, the registry owns the
     database handles, and all inputs/outputs pass through the same
     query-compatible boundary used by later real phase services.
   - Tests: registration fixtures; duplicate rejection; `cache_key`
     determinism harness with test-only fixture services; missing real owner
     seams reported as classified gaps without synthetic outputs; a positive
     guard for the driver-owned salsa/query boundary; dependency scans proving
     syntax/parser/phase owner crates do not gain driver or salsa dependencies;
     boundary guards against cache compatibility, proof acceptance, artifact
     publication-token, LSP payload, or scheduler-readiness ownership.
   - Deps: 4. Spec: `registry.md`.
   - Completed by task D-005: `src/registry.rs` implements deterministic
     phase-service registration, duplicate coverage rejection, the phase 0-16
     requirement table, missing-service gap reporting, `PhaseService`
     cache/execute adapters, a driver-local salsa database/query observation
     boundary, and focused registry/lint tests. Only test-local fixture
     services are used. No real semantic adapter, cache compatibility decision,
     proof acceptance, artifact publication token, LSP payload, or scheduler
     readiness logic was introduced.

6. **`SourceFrontend` service adapter.** [x]
   - Wrap `mizar-frontend` phases 1-3 as the first real `PhaseService`
     (input: plan slice; output: frontend outputs sealed through
     `mizar-ir`).
   - Tests: adapter round-trip over a fixture module; diagnostics flow
     into the sink.
   - Deps: 5, `mizar-ir` task 8. Spec: `registry.md`,
     [mizar-frontend todo](../../mizar-frontend/en/todo.md).
   - Completed by task D-006 as a classified `external_dependency_gap`, not as
     source implementation: [frontend_adapter.md](frontend_adapter.md) records
     that `mizar-frontend` has a real in-memory `FrontendOutput`, but the
     required canonical `mizar-ir` producer payload, diagnostics-draft bridge,
     and driver build-plan-to-source-request mapping are not yet real seams. No
     fake adapter, synthetic IR payload, or message-text keyed diagnostic bridge
     was added. The registry's `SourceFrontend` requirement remains
     `external_dependency_gap`.

### Orchestration

7. **Spec: `driver.md`.** [x]
   - Write the driver spec (English and Japanese, no code): the
     `CompilerDriver` API (`submit`, `cancel`, `events`), phase-0
     bootstrap, task-graph submission, and the artifact commit boundary
     hand-off.
   - Deps: 4. Spec: [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
     "Driver API"/"Control Flow".
   - Completed by task D-007: [driver.md](driver.md) defines the
     `CompilerDriver` submit/cancel/events boundary, phase-0 bootstrap through
     `mizar-build`, task-graph and scheduler submission ownership, terminal
     cancellation behavior with supported supersession through
     `mizar-build::CancellationPolicy`, diagnostics and artifact handoff
     boundaries, and the rule that missing phase services remain classified
     gaps rather than synthetic scheduler outputs.

8. **Driver core.** [x]
   - Implement `submit`: bootstrap phase 0 via the `mizar-build` planner,
     create the session, expand and submit the task graph, and consume the
     current modeled scheduler seam without duplicating scheduler semantics.
     Real scheduler-driven service execution waits for the D-007
     `external_dependency_gap` dispatch seam.
   - Tests: batch-oriented fixture workspace that captures a session, builds
     the plan/index/graph through `mizar-build`, reports missing real phase
     services as classified gaps without synthetic outputs, validates phase-0
     scheduler submission against `mizar-build` authority, and verifies that
     descriptor-present non-phase-0 work blocks on DRIVER-G-011 instead of
     exposing synthetic phase outputs. A real frontend-service fixture is
     required only after the D-006 owner seams exist.
   - Deps: 3, 5, 6, 7, `mizar-build` task 8. Spec: `driver.md`.
   - Completed by task D-008: `src/driver.rs` implements `CompilerDriver`
     session submission, phase-0 bootstrap through `mizar-build`
     planner/module-index/task-graph APIs, modeled scheduler submission/result
     consumption, missing-phase-service blocking without synthetic outputs,
     same-lane stale request suppression before scheduler submission, minimal
     protocol-agnostic event stream handle, and idempotent terminal
     cancellation state. Public scheduler submission results expose only
     output-free task-state/event/diagnostic summaries. Test-local
     descriptor-only phase fixtures prove that non-phase-0 work blocks on the
     DRIVER-G-011 dispatch gap even when descriptors exist, and panic if cache
     keys or execution are requested. Real scheduler-driven phase-service
     dispatch remains DRIVER-G-011 `external_dependency_gap`. Task D-011
     records snapshot-wide explicit/shutdown cancellation policy propagation as
     an `external_dependency_gap` because `mizar-build` exposes no driver-owned
     mutator for that reason.

9. **Spec: `events.md`.** [x]
   - Write the events spec (English and Japanese, no code): the
     `BuildEventStream` (progress, phase completion, diagnostics
     readiness, commit), deterministic event ordering, and consumer rules
     for CLI and LSP.
   - Deps: 7. Spec: [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
     "Build Events".
   - Completed by task D-009: [events.md](events.md) defines the
     protocol-agnostic event boundary, freshness suppression, deterministic
     ordering key, diagnostics and artifact readiness limits, CLI/LSP consumer
     rules, and D-010 test requirements. It deliberately adds no `src/events.rs`
     implementation and leaves artifact/LSP/diagnostics authority with their
     owner crates.

10. **Build event stream.** [x]
    - Implement event publication with deterministic ordering independent
      of worker completion order.
    - Tests: shuffled completion produces identical event sequences;
      events reference valid sessions/snapshots; stale sessions suppress
      current publication; dispatch, phase-service, diagnostics, and artifact
      gap/non-authority guards are enforced.
    - Deps: 8, 9. Spec: `events.md`.
   - Completed by task D-010: `src/events.rs` defines protocol-agnostic
     `BuildEventStream`, event identity/order keys, deterministic sorting and
     replay, `DispatchGap`, `OwnerReadinessGap`, phase-service gap, owner
     readiness reference, and stale-publication event carriers. Event tests
     cover shuffled ordering, session/snapshot validity, stale suppression,
     gap/non-authority guards, cancelled phase readiness, owner refs without
     diagnostics/LSP authority, replay, and source authority guards. The module
     does not implement CLI rendering, LSP conversion, diagnostic aggregation,
     artifact tokens, or real phase dispatch.

11. **Cancellation flow.** [x]
    - Implement `cancel`: propagate supported supersession through
      `mizar-build` cancellation policy and report a terminal session state;
      superseded watch sessions cancel cleanly.
    - Tests: cancel mid-build reaches a terminal state without partial
      publications; double-cancel is idempotent.
    - Deps: 8, `mizar-build` task 14. Spec: `driver.md`,
      [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
      "Cancellation".
   - Completed by task D-011: `CompilerDriver::cancel` now moves active sessions
     to terminal `Cancelled` or `Superseded` outcomes, appends terminal replay
     events, keeps double-cancel idempotent, and propagates only real
     supersession policy through `mizar-build`. Snapshot-wide explicit/shutdown
     policy propagation is recorded as `external_dependency_gap`.

### Entry points

12. **Spec: `cli.md` and the CLI surface decision.** [x]
    - Resolve the CLI-surface decision against the spec-23 build
      lifecycle; write the CLI spec (binary, subcommands, exit codes,
      progress rendering from `BuildEventStream`, diagnostics rendering via
      `mizar-diagnostics`).
    - Deps: 7. Spec:
      [23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md).
    - Completed by task D-012: [cli.md](cli.md) defines `mizar build`, batch
      request mapping, progress rendering from `BuildEventStream`, diagnostics
      rendering through `mizar-diagnostics`, stable exit codes, and gap handling
      for unavailable owner seams. No source implementation was added.

13. **CLI batch entry point.** [x]
    - Implement the batch subcommand: parse arguments into a
      `BuildRequestDraft`, run the driver, render diagnostics and progress,
      and map results to exit codes.
    - Tests: end-to-end CLI run over a fixture workspace; stable exit
      codes; golden-file output or inline golden output assertions.
    - Deps: 10, 12. Spec: `cli.md`.
    - Completed by task D-013: `src/cli.rs` implements the library-level
      batch entry point for `mizar build`, argument parsing, request-draft
      creation over owner-provided snapshot inputs, `CompilerDriver::submit`
      execution, protocol-agnostic human/JSON progress rendering from
      `BuildEventStream`, and stable exit-code mapping. Planning/lockfile
      diagnostics are reported as an explicit diagnostics owner bridge gap
      until real `mizar-diagnostics` records are available. Missing phase
      services and dispatch gaps exit as `UnavailableOwner`; no artifact
      publication token, committed output path, LSP payload, cache
      compatibility decision, proof acceptance, or fake producer output was
      introduced.
      The D-013 library entry point rejects unresolved manifest-path selection,
      non-matching package/module targets, and source-layout/snapshot mismatches
      as `external_dependency_gap` before driver submission so it cannot claim a
      current build for work outside the captured request snapshot.

14. **Watch mode.** [x]
    - Implement the watch-facing orchestration helper over owner-provided
      changed paths and snapshot inputs: incremental resubmission through
      `CompilerDriver::submit`, superseded-session cancellation, stale replay
      suppression, and real `mizar-ir::PhaseOutputPublisher` snapshot
      replacement when that owner seam is supplied.
    - Do not implement OS file watching, debounce/coalescing, source loading,
      file-to-module discovery, LSP protocol conversion, fake watcher APIs, or
      provisional producer/artifact tokens. Missing file-watcher/LSP/publisher
      seams are classified gaps.
    - Tests: change → rebuild fixtures over owner-provided snapshot inputs;
      stale sessions, including already-terminal sessions, never replay current
      publication after supersession; real publisher replacement is invoked;
      same-snapshot replacement is a no-op; missing publisher is a classified
      gap; non-watch requests are rejected; source guards prevent watcher/LSP/
      artifact/proof/cache authority from entering the driver.
    - Deps: 11, 13, `mizar-ir` task 13. Spec:
      [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
      "Watch and LSP Build", `driver.md`, and
      [mizar-ir crate exit report](../../mizar-ir/en/crate_exit_report.md)
      snapshot replacement.
    - Completed by task D-014: `CompilerDriver::submit_watch_change` accepts
      watch-origin drafts over owner-provided changed paths and snapshot inputs,
      derives/validates the lane-current previous watch session, resubmits
      through `CompilerDriver::submit`, mutates stale previous replay to
      suppressed `Superseded`, and consumes the real
      `mizar-ir::PhaseOutputPublisher` replacement seam when supplied. Missing
      file-watcher/LSP/publisher owner seams remain classified gaps. No fake
      watcher, source loader, LSP payload, artifact token, cache/proof
      decision, or producer output was introduced.

15. **Phase service adapters for semantic phases.** [x] — paced by the
    pipeline crates.
    - Register adapters for `ModuleResolver`, `SemanticChecker`,
      `Elaborator`, `VcService`, `AtpService`, `KernelService`,
      `ArtifactService`, and `DocExtractionService` as each crate's
      service-facing surface lands; one adapter per change. Checked off
      for this task stream when every currently available owner-provided
      adapter has either been registered and tested or classified as
      unavailable; future real adapter landings require one task per adapter.
    - Tests per adapter: fixture run through the driver; diagnostics and
      outputs flow end-to-end.
    - Deps: 8; pairs with the respective crates' integration tasks. Spec:
      `registry.md`.
    - Completed by task D-015 as a readiness/classification task, not a source
      adapter implementation: [registry.md](registry.md) records that no
      semantic/proof/artifact/doc owner currently exposes the full
      driver-callable service input, canonical `mizar-ir` producer output,
      `mizar-diagnostics` bridge, and proof/cache/artifact/LSP authority
      boundaries required for a real adapter. `ModuleResolver`,
      `SemanticChecker`, `Elaborator`, `VcService`, `AtpService`,
      `KernelService`, and `ArtifactService` remain classified
      `external_dependency_gap`; `DocExtractionService` remains `deferred`; the
      absent `mizar-artifact` closeout report remains a report-only
      `repo_metadata_conflict`. Existing registry tests cover missing-service
      gap reporting and boundary guards. No fake adapter, placeholder producer
      output, provisional publication token, proof/cache authority movement, or
      LSP bridge was introduced.

### Hardening and cross-cutting follow-ups

16. **End-to-end determinism suite.** [x]
    - Property coverage that identical workspaces produce identical event
      streams, diagnostics, and exit codes across worker counts and runs.
      After the cache/scheduler seam is wired, include architecture-22
      equivalence cases for clean versus incremental and cache hit/miss timing
      through the driver-owned query boundary, with stale or obsolete snapshot
      outputs rejected before they can publish as current diagnostics or
      artifacts.
    - Deps: 13, 14, `mizar-build` task 24. Spec:
      [20.test_strategy.md](../../architecture/en/20.test_strategy.md),
      [22.incremental_verification_contract.md](../../architecture/en/22.incremental_verification_contract.md).
    - Completed by task D-016: `tests/determinism.rs` compares the
      driver-owned public projection for identical phase-zero workspaces across
      repeated runs, worker counts, and scheduler completion orders; checks
      byte-stable CLI human/JSON output and exit codes for successful builds,
      manifest diagnostics owner gaps, and unavailable phase-service owner
      gaps; verifies that multi-task source/module work is deterministically
      blocked before scheduler submission while real phase dispatch remains an
      owner gap; and verifies that superseded watch replay deterministically
      emits suppressed publications without `diagnostics_ready` or
      `artifact_boundary` events. The suite does not invent semantic/proof
      adapters, cache compatibility decisions, artifact publication tokens, or
      LSP protocol bridges. Full clean/incremental/parallel equivalence with
      real cache hits, producer outputs, artifact commits, proof reuse, and
      multi-task driver phase dispatch remains deferred until those owner seams
      exist.

17. **Public-enum forward-compatibility policy.** [x]
    - Apply the `mizar-frontend` task-25 procedure to each public enum.
    - Deps: 13. Spec: all module specs.
    - Completed by task D-017: every current public enum in `request`,
      `registry`, `driver`, `events`, and `cli` is classified in its owning
      module spec as a downstream-facing `#[non_exhaustive]` boundary type with
      no exhaustive exceptions. The existing enum declarations already
      satisfied that policy and were audited rather than changed; only the
      lint-policy guard wording was updated to describe the completed policy.
      The source/spec correspondence table portion of the frontend task-25
      procedure is reserved for task 18, which is the driver-wide
      source/spec correspondence audit. No runtime behavior changed.

18. **Source/spec correspondence audit.** [x]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 17. Spec: all module specs and this TODO.
    - Completed by task D-018: [source_spec_correspondence.md](source_spec_correspondence.md)
      traces the implemented `request`, `registry`, `driver`, `events`, and
      `cli` public APIs and promised behaviors to source and tests. No new
      unresolved blocking, high, or medium source/spec drift was found. Existing
      `external_dependency_gap`, `deferred`, and report-only
      `repo_metadata_conflict` items remain classified follow-ups rather than
      source changes in this audit.

19. **Bilingual documentation sync audit.** [x]
    - Compare each English canonical document under
      `doc/design/mizar-driver/en/` with its Japanese companion and
      synchronize content.
    - Deps: 18. Spec: repository documentation policy.
    - Completed by task D-019: [bilingual_documentation_sync.md](bilingual_documentation_sync.md)
      records that the driver design corpus is paired one-to-one across
      `en/` and `ja/`, including crate plan, TODO, module specs, adapter
      readiness, source/spec correspondence, and the new sync audit itself.
      No unresolved blocking or high EN/JA documentation drift was found.
      Existing `external_dependency_gap`, `deferred`, and report-only
      `repo_metadata_conflict` records remain unchanged and synchronized.

20. **Architecture-22 follow-up audit.** [x]
    - Re-run the source/spec correspondence and bilingual documentation sync
      audits for the task-16 driver query-boundary, stale-output rejection,
      diagnostics, and artifact-publication contract; record any remaining
      architecture-22 gaps as follow-up tasks.
    - Deps: 16, 19. Spec: all module specs, this TODO, and repository
      documentation policy.
    - Completed by task D-020: [architecture_22_follow_up_audit.md](architecture_22_follow_up_audit.md)
      records no unresolved blocking/high architecture-22 drift for implemented
      driver seams. Query boundary, stale-output suppression, diagnostics
      owner-gap handling, artifact-boundary non-ownership, and worker-count
      determinism remain covered by existing tests and docs. Full real
      clean/incremental/parallel equivalence, scheduler-selected phase
      dispatch, LSP bridge, and semantic/proof/artifact adapters remain
      classified as `deferred` or `external_dependency_gap`.

21. **Module-boundary refactor gate.** [x]
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
    - Deps: 20. Spec: this TODO,
      [internal 07](../../internal/en/07.crate_module_layout.md), all module
      specs.
    - Completed by task D-021: [module_boundary_refactor_gate.md](module_boundary_refactor_gate.md)
      records the source-layout audit and private helper split. `cli` output
      rendering moved to `src/cli/output.rs`; driver event construction,
      scheduler helpers, watch helpers, and unit tests moved to private
      `src/driver/` children; registry phase catalog/hash helpers moved to
      `src/registry/catalog.rs`. Public modules, public APIs, deterministic
      renderings, diagnostics/artifact-facing schemas, and owner boundaries did
      not change.

## Recommended Verification

Run after each task:

```text
cargo test -p mizar-driver
cargo clippy -p mizar-driver --all-targets -- -D warnings
```

For orchestration tasks, also run:

```text
cargo test -p mizar-build
cargo test -p mizar-ir
cargo test -p mizar-frontend
```

Check the task off here once tests pass.

## Notes

- The driver owns wiring, not semantics: no type checking, no overload
  resolution, no VC generation, no proof acceptance, no cache
  compatibility decisions, no artifact serialization, no LSP range
  conversion.
- `PhaseService::cache_key` must stay a pure projection; the registry's
  purity harness is the enforcement point.
- `salsa` is introduced at the driver/registry layer, not in syntax, parser, or
  semantic phase crates. Phase adapters provide pure query-compatible
  boundaries.
- Diagnostics from obsolete snapshots are never published as current;
  artifact commits never happen in completion order.
- LSP entry points reuse the same driver API through `mizar-lsp`; this
  crate stays protocol-agnostic.
