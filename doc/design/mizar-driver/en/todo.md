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
| registry | `registry.md` (task 4) | `src/registry.rs` | [x] |
| driver | `driver.md` (task 7) | `src/driver.rs` | [ ] |
| events | `events.md` (task 9) | `src/events.rs` | [ ] |
| cli | `cli.md` (task 12) | `src/cli.rs` | [ ] |

`mizar-driver` is the front door for all build modes: it parses CLI/watch/LSP
requests into `BuildRequest`s, bootstraps phase 0 through the `mizar-build`
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

6. **`SourceFrontend` service adapter.** [ ]
   - Wrap `mizar-frontend` phases 1-3 as the first real `PhaseService`
     (input: plan slice; output: frontend outputs sealed through
     `mizar-ir`).
   - Tests: adapter round-trip over a fixture module; diagnostics flow
     into the sink.
   - Deps: 5, `mizar-ir` task 8. Spec: `registry.md`,
     [mizar-frontend todo](../../mizar-frontend/en/todo.md).

### Orchestration

7. **Spec: `driver.md`.** [ ]
   - Write the driver spec (English and Japanese, no code): the
     `CompilerDriver` API (`submit`, `cancel`, `events`), phase-0
     bootstrap, task-graph submission, and the artifact commit boundary
     hand-off.
   - Deps: 4. Spec: [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
     "Driver API"/"Control Flow".

8. **Driver core.** [ ]
   - Implement `submit`: bootstrap phase 0 via the `mizar-build` planner,
     create the session, expand and submit the task graph, and drive
     registered services through the scheduler.
   - Tests: batch run over a fixture workspace with the frontend service;
     deterministic phase ordering.
   - Deps: 3, 5, 6, 7, `mizar-build` task 8. Spec: `driver.md`.

9. **Spec: `events.md`.** [ ]
   - Write the events spec (English and Japanese, no code): the
     `BuildEventStream` (progress, phase completion, diagnostics
     readiness, commit), deterministic event ordering, and consumer rules
     for CLI and LSP.
   - Deps: 7. Spec: [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
     "Build Events".

10. **Build event stream.** [ ]
    - Implement event publication with deterministic ordering independent
      of worker completion order.
    - Tests: shuffled completion produces identical event sequences;
      events reference valid sessions.
    - Deps: 8, 9. Spec: `events.md`.

11. **Cancellation flow.** [ ]
    - Implement `cancel`: propagate through `mizar-build` cancellation
      tokens and report a terminal session state; superseded watch
      sessions cancel cleanly.
    - Tests: cancel mid-build reaches a terminal state without partial
      publications; double-cancel is idempotent.
    - Deps: 8, `mizar-build` task 14. Spec: `driver.md`,
      [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
      "Cancellation".

### Entry points

12. **Spec: `cli.md` and the CLI surface decision.** [ ]
    - Resolve the CLI-surface decision against the spec-23 build
      lifecycle; write the CLI spec (binary, subcommands, exit codes,
      progress rendering via `mizar-diagnostics`).
    - Deps: 7. Spec:
      [23.package_management_and_build_system.md](../../../spec/en/23.package_management_and_build_system.md).

13. **CLI batch entry point.** [ ]
    - Implement the batch subcommand: parse arguments into a
      `BuildRequest`, run the driver, render diagnostics and progress,
      and map results to exit codes.
    - Tests: end-to-end CLI run over a fixture workspace; stable exit
      codes; golden-file output.
    - Deps: 10, 12. Spec: `cli.md`.

14. **Watch mode.** [ ]
    - Implement the watch loop: file-change detection, snapshot
      replacement through `mizar-ir`, superseding-session cancellation,
      and incremental resubmission.
    - Tests: change → rebuild fixtures; stale sessions never publish;
      replacement keeps retained outputs alive.
    - Deps: 11, 13, `mizar-ir` task 13. Spec:
      [internal 01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md)
      "Watch and LSP Build".

15. **Phase service adapters for semantic phases.** [ ] — paced by the
    pipeline crates.
    - Register adapters for `ModuleResolver`, `SemanticChecker`,
      `Elaborator`, `VcService`, `AtpService`, `KernelService`,
      `ArtifactService`, and `DocExtractionService` as each crate's
      service-facing surface lands; one adapter per change. Checked off
      when the last adapter lands.
    - Tests per adapter: fixture run through the driver; diagnostics and
      outputs flow end-to-end.
    - Deps: 8; pairs with the respective crates' integration tasks. Spec:
      `registry.md`.

### Hardening and cross-cutting follow-ups

16. **End-to-end determinism suite.** [ ]
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

17. **Public-enum forward-compatibility policy.** [ ]
    - Apply the `mizar-frontend` task-25 procedure to each public enum.
    - Deps: 13. Spec: all module specs.

18. **Source/spec correspondence audit.** [ ]
    - Trace every public API and promised behavior in the module specs to
      implementation and tests; record gaps as follow-up tasks.
    - Deps: 17. Spec: all module specs and this TODO.

19. **Bilingual documentation sync audit.** [ ]
    - Compare each English canonical document under
      `doc/design/mizar-driver/en/` with its Japanese companion and
      synchronize content.
    - Deps: 18. Spec: repository documentation policy.

20. **Architecture-22 follow-up audit.** [ ]
    - Re-run the source/spec correspondence and bilingual documentation sync
      audits for the task-16 driver query-boundary, stale-output rejection,
      diagnostics, and artifact-publication contract; record any remaining
      architecture-22 gaps as follow-up tasks.
    - Deps: 16, 19. Spec: all module specs, this TODO, and repository
      documentation policy.

21. **Module-boundary refactor gate.** [ ]
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
