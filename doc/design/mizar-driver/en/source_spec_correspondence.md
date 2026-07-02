# Source/spec correspondence audit

> Canonical language: English. Japanese companion:
> [../ja/source_spec_correspondence.md](../ja/source_spec_correspondence.md).

Status: completed by task D-018.

## Scope

This audit checks the implemented `mizar-driver` source against the English
canonical module specs:

- [request.md](request.md)
- [registry.md](registry.md)
- [driver.md](driver.md)
- [events.md](events.md)
- [cli.md](cli.md)

It traces public APIs and promised behavior to Rust source and tests. It is a
lightweight source/spec/test correspondence map, not a replacement for the
executable tests. If a missing implementation, stale spec, or missing test is
found, this audit records it as a classified follow-up instead of silently
accepting drift. Medium findings must be fixed or explicitly deferred with a
reason; they are not treated as generic follow-up notes.

## Result

- No unresolved blocking, high, or medium source/spec drift was found.
- All current public APIs exposed by `src/request.rs`, `src/registry.rs`,
  `src/driver.rs`, `src/events.rs`, and `src/cli.rs` are covered by the module
  specs or by the task D-017 public enum policy sections.
- Every current public enum is `#[non_exhaustive]`; no exhaustive exceptions are
  recorded.
- The implementation and tests preserve the driver ownership boundary:
  orchestration, request/session lifecycle, registry/query boundary, scheduler
  submission, protocol-agnostic events, CLI batch entry point, and watch
  orchestration are implemented; phase semantics, proof/cache/artifact
  authority, diagnostics identity, and LSP protocol conversion remain outside
  the driver.
- Existing owner gaps remain intentionally classified rather than repaired in
  this audit: semantic/proof/artifact phase adapters are
  `external_dependency_gap`, documentation extraction is `deferred`, real
  clean/incremental/parallel equivalence with producer/cache/artifact/proof
  seams is deferred, and the absent `mizar-artifact` closeout report remains a
  report-only `repo_metadata_conflict`.

## Public API Correspondence

| Spec | Public API checked | Source | Test evidence |
|---|---|---|---|
| [request.md](request.md) | Request/session envelopes: `BuildRequestDraft`, `BuildRequest`, `PendingBuildRequest`, `CaptureSnapshotError`, `CapturedSnapshot`, `BuildSession`, `DriverLanes`, `LaneCurrentSession`, `BuildLaneId`, `BuildRequestGeneration`, `BuildProfile`, `BuildTargets`, `SourceInputSet`, `DependencyInputSet`, `VerifierConfigInput`, `BatchRequest`, `BatchInvocation`, `WatchRequest`, `LspRequest`, `LspFocus`, `ObsoletePublication`; enums `BuildRequestOrigin`, `LspPriority`, `BuildSessionState`, `BuildSessionOutcome`, `PublicationDecision`; methods for id allocation, snapshot input projection, snapshot capture, lifecycle transitions, lane currentness, and publication decisions. | `crates/mizar-driver/src/request.rs` | `tests/request.rs` covers batch/watch/LSP request shapes, snapshot capture through `mizar-session`, identical snapshot inputs, lane/generation currentness, same-snapshot supersession rejection, publication suppression, capture errors, and lifecycle/cancellation idempotence. `tests/watch.rs`, `tests/driver.rs`, `tests/events.rs`, and `tests/determinism.rs` exercise the same boundary through driver submit, watch supersession, event publication, and stale-output rejection. |
| [registry.md](registry.md) | Phase registry and query boundary: `PhaseDescriptor`, `PhaseRequirement`, `PhaseInput`, `PhaseInputIdentities`, `PhaseContext`, `PhaseCacheContext`, `PhaseExecutionContext`, `PhaseExecutionResources`, `PhaseResult`, `PhaseCacheObservation`, `PhaseCacheQueryResult`, `PhaseExecutionQueryResult`, `PhaseRegistryBuilder`, `PhaseRegistry`, `DriverQueryBoundary`, `DriverQueryDatabase`, `PhaseService`, `required_phase_services`, and enums `PhaseOwner`, `PhaseServiceAvailability`, `PhaseStatus`, `PhaseCacheIntent`, `PhaseRegistryError`. | `crates/mizar-driver/src/registry.rs` | `tests/registry.rs` covers the phase service table, deterministic registration, duplicate rejection, descriptor normalization, missing owner seams, cache-key purity, dependency/parent query identity, salsa/query-boundary execution, and source guards against semantic/proof/cache/artifact/LSP authority. `tests/lint_policy.rs` checks phase owner crates do not gain driver or salsa dependencies. |
| [driver.md](driver.md) | Driver front door and watch orchestration: `BuildSubmission`, `DriverSubmitInput`, `PhaseDispatchInputProvider`, `CompilerDriver`, `WatchSubmission`, `WatchSubmitControl`, `WatchSupersededSession`, `WatchSnapshotReplacement`, `WatchModeGap`, `WatchSubmitFailure`, `DriverSchedulerRun`, `DriverMissingPhaseService`, `DriverCancelOutcome`; enums `WatchSnapshotReplacementStatus`, `WatchModeGapOwner`, `WatchOwnerSeam`, `WatchSubmitError`, `DriverSubmissionStatus`, `DriverCancelReason`, `DriverSubmitError`; methods `CompilerDriver::{new, registry, session, cancellation_policy, submit_watch_change, submit, cancel, events}`. | `crates/mizar-driver/src/driver.rs`, `crates/mizar-driver/src/driver/scheduler.rs` | `tests/driver.rs` covers phase-0 bootstrap, `mizar-build` planner/index/task graph use, scheduler submission/result consumption, missing phase-service gaps, dispatch-gap blocking without synthetic outputs when owner inputs are absent, registry-backed scheduler-selected dispatch when owner inputs are supplied, stale same-lane suppression, failed module-index session storage, cancellation, and source guards against non-owner authority. `tests/watch.rs` covers watch replacement, superseded replay, missing watcher/LSP/publisher seams, publisher replacement failures, and previous-session validation. |
| [events.md](events.md) | Event stream boundary: `BuildEventStream`, `BuildEvent`, `BuildEventIdentity`, `BuildEventOrderKey`, `TaskEventRef`, `OwnerRecordRef`, `BuildEventLog`, `BuildEventIdentityKey`, `diagnostics_gap_event`, and enums `BuildEventKind`, `PlanningEventStatus`, `EventOwner`, `OwnerGapClassification`, `BuildEventError`. | `crates/mizar-driver/src/events.rs` | `tests/events.rs` covers deterministic event sorting, stream/session/snapshot validation, publication suppression, stale diagnostics/artifact rejection, failure isolation, gap events, task-progress non-artifact behavior, owner refs, replay order, and source guards against diagnostics/artifact/scheduler/LSP authority. `tests/watch.rs` and `tests/determinism.rs` cover suppressed replay from real driver sessions. |
| [cli.md](cli.md) | Batch CLI surface: `CliInvocation`, `CliSnapshotInputs`, `CliBatchInput`, `CliOutput`, `CliUsageError`; enums `CliCommand`, `CliBuildProfile`, `CliMessageFormat`, `CliExitCode`; methods `CliInvocation::{parse, request_draft}`, `CliSnapshotInputs::new`, `CliBatchInput::new`, `CliOutput::process_code`, `CliBuildProfile::as_str`, `CliExitCode::process_code`, `run_batch`, `run_batch_with_driver`, and `run_invocation_with_driver`. | `crates/mizar-driver/src/cli.rs` | `tests/cli.rs` covers argument parsing, request/profile/target/scheduler controls, success/progress rendering, JSON event output, manifest and module-index diagnostic owner gaps, usage/internal errors, owner-unavailable paths, source snapshot/layout guards, cancellation mapping, quiet output, and source guards against LSP/artifact/proof/cache/phase-semantics authority. `tests/determinism.rs` covers byte-stable human/JSON CLI output and exit codes across repeated runs and worker counts. |
| [todo.md](todo.md) task D-017 | Public enum compatibility policy for every public driver enum. | `crates/mizar-driver/src/*.rs`, `crates/mizar-driver/tests/lint_policy.rs` | `tests/lint_policy.rs::public_driver_enums_are_forward_compatible` scans the crate target files and fails any public enum that lacks adjacent `#[non_exhaustive]`. D-017 module-spec sections record no exhaustive exceptions. |

## Public Method Surface Correspondence

The type-level table above is expanded here for public constructors,
accessors, query entry points, and free functions. Private helpers and private
trait adapters are not part of this public API audit.

- `request.rs`: `BuildLaneId::{new, get}`,
  `BuildRequestGeneration::{new, get}`, `BuildProfile::new`,
  `DependencyInputSet::new`, `VerifierConfigInput::new`,
  `BuildRequestDraft::allocate`, `BuildRequest::snapshot_input`,
  `PendingBuildRequest::capture_snapshot`,
  `BuildSession::{lane_current_session, mark_submitted, mark_running, cancel,
  finish, is_terminal}`, and
  `DriverLanes::{mark_current, current, is_current_session,
  is_session_current, publication_decision}` map the request spec's snapshot
  boundary, lane/generation currentness, lifecycle, and publication-decision
  rules to `tests/request.rs`, with downstream exercise through
  `tests/driver.rs`, `tests/watch.rs`, `tests/events.rs`, and
  `tests/determinism.rs`.
- `registry.rs`: `PhaseDescriptor::new`, `PhaseInput::new`,
  `PhaseInputIdentities::new`, `PhaseResult::complete`,
  `PhaseRegistryBuilder::{new, register, register_arc, build}`,
  `PhaseRegistry::{empty, descriptors, query_boundary, descriptor_for_phase,
  cache_key_for_phase, execute_phase, execute_phase_with_resources}`,
  `DriverQueryBoundary::{cache_key, execute}`,
  `PhaseService::{phase, cache_key, execute}`, and `required_phase_services`
  map the registry spec's deterministic registration, duplicate rejection,
  missing-service classification, pure cache-key projection, and driver-owned
  salsa query boundary to `tests/registry.rs` and the owner-boundary guards in
  `tests/lint_policy.rs`.
- `driver.rs`: `CompilerDriver::{new, registry, session,
  cancellation_policy, submit_watch_change, submit, cancel, events}` and
  `DriverSubmitInput::new` map the driver-front-door, scheduler-submission,
  cancellation, session storage, event replay, and watch-orchestration rules to
  `tests/driver.rs`, `tests/watch.rs`, `tests/events.rs`, and
  `tests/determinism.rs`.
- `events.rs`: `BuildEventStream::{empty, from_events, events, replay}`,
  `BuildEvent::new`, `BuildEventOrderKey::{new, with_scheduler_order,
  with_phase, with_work_unit, with_owner_order}`, `TaskEventRef::new`,
  `OwnerRecordRef::new`, `BuildEventLog::{new, push, extend, into_stream}`,
  and `diagnostics_gap_event` map the event spec's identity validation,
  deterministic ordering, replay, stale-publication rejection, owner-record
  references, and diagnostics-owner-gap event construction to
  `tests/events.rs`, with watch and determinism replay coverage in
  `tests/watch.rs` and `tests/determinism.rs`.
- `cli.rs`: `CliInvocation::{parse, request_draft}`,
  `CliSnapshotInputs::new`, `CliBatchInput::new`, `CliOutput::process_code`,
  `CliBuildProfile::as_str`, `CliExitCode::process_code`, `run_batch`,
  `run_batch_with_driver`, and `run_invocation_with_driver` map the CLI spec's
  batch parsing, request construction, driver invocation, deterministic output,
  and exit-code contract to `tests/cli.rs` and `tests/determinism.rs`.

## Promised Behavior Correspondence

| Behavior promise | Source status | Test evidence |
|---|---|---|
| Driver owns orchestration, not phase semantics, proof acceptance, cache compatibility, artifact serialization, diagnostics identity, or LSP protocol conversion. | Ownership boundaries are documented in every module and guarded in source by narrow APIs and absence of forbidden authority terms. | Source guards in `tests/driver.rs`, `tests/registry.rs`, `tests/events.rs`, `tests/cli.rs`, `tests/watch.rs`, and `tests/lint_policy.rs`. |
| Build requests and sessions capture exactly one current source/dependency snapshot and reject obsolete watch/LSP publication. | `request.rs` stores lane/generation metadata, captures snapshots through `mizar-session`, tracks current lanes, and computes `PublicationDecision`. | `tests/request.rs`, `tests/watch.rs`, `tests/events.rs`, and `tests/determinism.rs`. |
| Phase registry registration is deterministic, duplicate coverage is rejected, and cache-key/query-boundary inputs stay pure. | `registry.rs` normalizes descriptors, sorts registrations, records requirements, owns the driver salsa query boundary, and reports missing owner seams. | `tests/registry.rs` and `tests/lint_policy.rs`. |
| Driver consumes `mizar-build` planning, task graph, scheduler, and cancellation authority without duplicating scheduler semantics. | `driver.rs` calls `mizar-build` planner, module index, task graph builder, and scheduler. Missing owner phase-input identities are reported only when a scheduler-selected task reaches the registry-backed dispatcher; supplied `PhaseDispatchInputProvider` identities flow scheduler-selected tasks through `run_scheduler_with_dispatcher`, `RegistrySchedulerDispatcher`, and `PhaseRegistry` without replaying readiness, dependency ordering, cache, resource, or cancellation decisions. | `tests/driver.rs`, including `registered_services_execute_from_scheduler_selected_dispatch`, `cache_hit_tasks_do_not_require_phase_dispatch_inputs`, and `registry_dispatch_statuses_map_to_scheduler_outcomes_without_publication`; `tests/determinism.rs`; and `cargo test -p mizar-build` coverage from D-016/task 27. |
| Event streams are protocol-agnostic, deterministically ordered, and reject current diagnostics/artifact payloads for suppressed sessions. | `events.rs` validates event identity/publication and sorts events by stable order keys. | `tests/events.rs`, `tests/watch.rs`, and `tests/determinism.rs`. |
| CLI maps command inputs to driver requests, renders progress from event streams, reports diagnostics owner gaps without fake records, and maps exit codes deterministically. | `cli.rs` parses the current batch command, constructs `BuildRequestDraft`/`DriverSubmitInput`, calls `CompilerDriver::submit`, and renders human/JSON progress. | `tests/cli.rs` and `tests/determinism.rs`. |
| Watch mode consumes owner-provided changed paths/snapshot inputs and optional real `mizar-ir` snapshot replacement; missing watcher/LSP/publisher seams are classified. | `driver.rs::submit_watch_change` validates prior sessions, delegates submit, supersedes replay, optionally calls real `PhaseOutputPublisher`, and records watch gaps. | `tests/watch.rs` and `tests/determinism.rs`. |
| End-to-end determinism is covered for implemented seams while full real cache/producer/artifact/proof equivalence remains deferred. | D-016 added crate-local deterministic projections and documented full-system deferral. | `tests/determinism.rs`; D-016 records in `todo.md` and `00.crate_plan.md`. |

## Gap And Follow-up Records

No new blocking, high, or medium source/spec drift was found by this audit.

Existing classified follow-ups remain:

- `DRIVER-G-001`: missing `mizar-artifact` closeout report remains a
  report-only `repo_metadata_conflict`.
- `DRIVER-G-013`: semantic/proof/artifact phase adapters remain
  `external_dependency_gap` until owner crates expose driver-callable inputs,
  canonical `mizar-ir` producer outputs, diagnostics bridges, and non-driver
  proof/cache/artifact/LSP authority contracts.
- `DRIVER-G-014`: `DocExtractionService` remains `deferred` until a
  documentation/extraction owner crate and service surface exist.
- Full clean/incremental/parallel equivalence with real cache hits, producer
  outputs, artifact commits, proof reuse, and multi-task driver phase dispatch
  remains deferred until those owner seams exist.
- D-019 completed the bilingual documentation sync audit; this D-018 audit
  checked source/spec correspondence, not full translation quality.

## Verification

Relevant verification for this audit is documentation review plus the existing
crate tests that enforce the traced behavior. D-018 itself changes only design
documents unless a later review finds blocking/high drift or a medium finding
that must be fixed in source.

Because the D-018 change is docs-only, the required local checks are
`git diff --check` and, after staging, `git diff --cached --check`. If a later
review requires Rust source changes, the Rust verification path applies:
`cargo fmt --check`, `cargo clippy -p mizar-driver --all-targets -- -D warnings`,
and `cargo test -p mizar-driver`.
