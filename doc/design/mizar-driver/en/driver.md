# Module: driver

> Canonical language: English. Japanese companion:
> [../ja/driver.md](../ja/driver.md).

Status: specified by task D-007. Source implementation is task D-008.

## Purpose

The `driver` module defines the `CompilerDriver` front door for build
submission, cancellation, and event subscription.

The driver connects request/session state, the phase-service registry,
`mizar-build` planning/task-graph/scheduler authority, diagnostics sinks,
IR output publication, and protocol-agnostic event streams. It owns orchestration. It does
not own phase semantics, scheduler semantics, cache compatibility, proof
acceptance, artifact serialization, or LSP protocol conversion.

## Ownership Boundary

`driver` owns:

- accepting a `BuildRequestDraft` from CLI/watch/LSP-facing callers and
  allocating/capturing a `BuildSession`;
- marking the session current for its driver lane and suppressing obsolete
  watch/LSP sessions through the request publication guard;
- invoking the phase-0 build bootstrap through `mizar-build` planner and module
  index APIs;
- constructing `mizar-build::task_graph::TaskGraphInput` and handing it to
  `mizar-build::task_graph::build_task_graph`;
- constructing `mizar-build::scheduler::SchedulerInput` and consuming
  `mizar-build::scheduler::run_scheduler_with_dispatcher` /
  `run_scheduler` results;
- dispatching registered phase services only through `PhaseRegistry`;
- translating owner-provided phase results, diagnostics batches, and sealed
  output handles into driver/scheduler/session outcomes;
- exposing replayable protocol-agnostic event streams from the `events` module.

`driver` does not own:

- manifest syntax rules, dependency solving, task graph validation, ready-queue
  semantics, resource admission, cache scheduling policy, cancellation
  checkpoint decisions, or scheduler result collation owned by `mizar-build`;
- source loading, parsing, name resolution, type checking, overload resolution,
  VC generation, ATP policy, proof acceptance, kernel trust, or documentation
  extraction owned by phase crates;
- `mizar-cache` cache-key construction, dependency fingerprints, cache-store
  compatibility, proof-reuse validation, or cache promotion;
- `mizar-diagnostics` diagnostic-code allocation, diagnostic identity,
  aggregation, rendering, explanation, or fix semantics;
- `mizar-ir` storage internals, producer payload schemas, or artifact
  projection authority;
- `mizar-artifact` manifest transactions, artifact serialization, or
  publication-token issuance;
- `mizar-lsp` JSON-RPC payloads, document-version handling, range conversion,
  code actions, or editor commands.

## Public Enum Compatibility

All public enums in this module are downstream-facing driver/session boundary
types and are marked `#[non_exhaustive]`. D-017 records no exhaustive exceptions
for:

- `WatchSnapshotReplacementStatus`;
- `WatchModeGapOwner`;
- `WatchOwnerSeam`;
- `WatchSubmitError`;
- `DriverSubmissionStatus`;
- `DriverCancelReason`;
- `DriverSubmitError`.

Downstream crates must use wildcard arms when matching these enums. Future
watch owner seams, cancellation reasons, submission states, and structured
submit errors may be added without giving the driver phase semantics, proof,
cache, artifact, or LSP authority.

## Gap Classification

| Gap | Classification | Driver disposition |
|---|---|---|
| `SourceFrontend` cannot yet publish canonical frontend payloads or diagnostics drafts. | `external_dependency_gap` | Keep the registry missing-service classification from [frontend_adapter.md](frontend_adapter.md); do not synthesize frontend outputs. |
| Semantic/proof/artifact/doc phase adapters are not all available. | `external_dependency_gap` / `deferred` | A submit call may report the missing owner seam as blocked or unavailable; it must not mark the phase complete. |
| Real cache lookup/compatibility is not wired through `mizar-cache` yet. | `external_dependency_gap` | Use disabled/unavailable cache scheduling unless a real cache decision is supplied by the owner seam. |
| Real artifact publication tokens and phase-15 producer emission are unavailable. | `external_dependency_gap` | Do not emit committed-artifact events or manifest publication records from driver-owned code. |
| Owner-provided phase input identities, parent output handles, diagnostics/output publisher handles, and producer publication tokens are not always available for real scheduler-selected dispatch. | `external_dependency_gap` / `deferred` | If scheduler-selected dispatch reaches a task whose owner inputs are absent, block with `BlockedByPhaseDispatchGap`; when supplied, consume the `mizar-build` scheduler-selected dispatcher seam and execute phase services only through `PhaseRegistry`. |
| External file watcher/coalescing owner and LSP build bridge are not available. | `external_dependency_gap` / `deferred` | D-014 may accept owner-provided changed paths and snapshot inputs, but it must not create fake file watcher APIs, source-loading rules, LSP document payloads, or editor snapshot conversion. |
| `mizar-ir` snapshot replacement is available only through `PhaseOutputPublisher`. | real owner seam when supplied; otherwise `external_dependency_gap` | D-014 registers the first watch snapshot as current and calls `replace_current_snapshot(old, new)` only when a real publisher is supplied and the snapshot id changes. Same-snapshot replacement is a no-op. Missing publisher input is a classified gap, not a provisional output handle. |
| Later event consumers such as CLI rendering and LSP protocol conversion are not implemented yet. | `deferred` to entry-point tasks | `events.md` / `src/events.rs` define protocol-agnostic event payloads and replay. Consumers must not pull CLI/LSP authority into the driver. |

## Public API

The public API is conceptual; task D-008 may choose concrete ownership and
borrowing details that fit the existing request and registry modules.

```rust
struct CompilerDriver {
    sessions: DriverSessionStore,
    lanes: DriverLanes,
    registry: PhaseRegistry,
}

impl CompilerDriver {
    fn submit(&mut self, request: BuildRequestDraft, input: DriverSubmitInput)
        -> Result<BuildSubmission, DriverSubmitError>;

    fn submit_watch_change(
        &mut self,
        request: BuildRequestDraft,
        input: DriverSubmitInput,
        control: WatchSubmitControl,
    )
        -> Result<WatchSubmission, WatchSubmitError>;

    fn cancel(
        &mut self,
        session: BuildSessionId,
        reason: DriverCancelReason,
        snapshots: &SnapshotRegistry,
    )
        -> DriverCancelOutcome;

    fn events(&self, session: BuildSessionId) -> BuildEventStream;
}
```

`submit` is the canonical primitive that starts a driver-owned build session.
Callers may be CLI, watch mode, or `mizar-lsp`, but they must provide
protocol-agnostic driver inputs. `submit_watch_change` is a watch-facing wrapper
that delegates session creation to `submit` and then handles lane-current
supersession, replay suppression, and optional IR snapshot replacement. LSP
ranges, JSON-RPC ids, and editor command payloads are not accepted by either
API.

`cancel` records a driver/session cancellation request and terminal outcome. It
uses the supplied snapshot registry with the driver lane table to apply the
combined publication guard before appending terminal events. It uses
`mizar-build::cancel::CancellationPolicy` only for supported owner seams such as
snapshot supersession. It does not kill worker threads or reinterpret
cancellation as phase failure.

`events` returns a session-scoped, protocol-agnostic event stream implemented
by [events.md](events.md) / `src/events.rs`. The driver stores replayable
session events, but it must not expose CLI-rendered diagnostics, LSP payloads,
artifact publication tokens, or scheduler synthetic outputs as event payloads.

## Submit Input

`BuildRequestDraft` carries request metadata before allocator-issued request
identity exists: lane/generation, source snapshot inputs, dependency artifacts,
lockfile/toolchain identity, verifier-config identity, targets, profile, and
origin. `BuildRequestId` is allocated only when the draft is accepted into a
`PendingBuildRequest`.

`DriverSubmitInput` carries owner inputs needed to bootstrap the build:

- a `SessionIdAllocator` and `SnapshotRegistry` for request/session ids and
  snapshot capture;
- `mizar-build::planner::PlanRequest`, workspace package manifests, and
  lockfile data for `produce_build_plan`;
- a `mizar-build::module_index::SourceLayoutProvider` and dependency artifact
  indexes for `build_module_index`;
- module dependency overlay and VC descriptors supplied by real owner seams, or
  classified unavailable coverage when those seams are absent;
- task graph profile, scheduler mode, priority hints, resource budget,
  cancellation policy, and cache scheduling plan/policy;
- optional `PhaseDispatchInputProvider` identities for scheduler-selected
  registry dispatch. Missing identities are reported only for tasks that reach
  the scheduler-selected dispatch callback.

The driver may validate that these inputs are present and well classified. It
must not parse manifests with local rules, derive module dependencies by
guessing semantics, construct cache keys, mint output handles, or invent
publication tokens.

## Submit Control Flow

1. Allocate the `BuildRequestDraft` into a `PendingBuildRequest` using the
   supplied session id allocator.
2. Capture the immutable snapshot through
   `PendingBuildRequest::capture_snapshot` and keep the returned active snapshot
   lease with the `BuildSession`.
3. Mark the session as lane-current through `DriverLanes::mark_current`. If the
   lane rejects the session as older or conflicting, finish it as
   `Superseded` and publish no current output.
4. Run phase 0 through `mizar-build` owner APIs:
   `planner::produce_build_plan`, `module_index::build_module_index`, and
   `task_graph::build_task_graph`.
5. If planning, module-index, or task-graph diagnostics are returned, preserve
   their structured kind/value data for diagnostics integration. Do not reduce
   diagnostic identity to rendered message text.
6. Check every graph task against `PhaseRegistry`. A missing phase service
   produces a classified missing-service or external-dependency outcome. It
   never produces a synthetic output handle.
7. Build `SchedulerInput` from the `TaskGraph` and the owner-provided
   scheduling controls. Cache decisions may be `Disabled`, `Unavailable`, or a
   real owner-provided plan; the driver must not decide compatibility itself.
8. Do not execute phase services before scheduler submission, and do not replay
   scheduler readiness, cache-hit, cancellation, or resource-admission logic as
   a dispatch preflight. If a scheduler-selected task lacks owner-provided
   dispatch inputs, the registry-backed dispatcher returns a blocked scheduler
   outcome; the driver records `BlockedByPhaseDispatchGap` and the affected
   phases from that scheduler result.
9. Pass the resulting `SchedulerInput` to
   `mizar-build::scheduler::run_scheduler_with_dispatcher` with a
   registry-backed dispatcher. The dispatcher calls `PhaseRegistry` only at
   scheduler-selected execution points, covers every phase listed on the
   selected task by invoking each owning registry service span once in
   task-graph phase order, and maps `PhaseStatus` into scheduler outcomes. The
   driver must not duplicate readiness, dependency ordering, resource
   admission, cache-decision consumption, or cancellation semantics.
10. The driver must not treat scheduler synthetic outputs, including default
    completed outputs, as real `mizar-ir` phase outputs. Real `PhaseResult`
    output references, producer outputs, and artifact tokens remain owner seams.
11. Before any result, diagnostic, or artifact-boundary handoff is exposed as
    current, call the combined request publication guard from `request.md`.
12. Finish the session as succeeded, failed, blocked, cancelled, or superseded
    from the structured scheduler/session outcomes.

The driver may choose fail-fast behavior when a required real owner seam is
missing. Missing phase input identities or publisher/producer seams remain
classified owner gaps. The driver must not use `mizar-build` synthetic output
types to pretend that an absent phase produced output.

## Scheduler Boundary

`mizar-build` owns task graph and scheduler semantics. The driver consumes:

- `TaskGraph` as the canonical dependency and phase-order plan;
- `SchedulerInput` as the only scheduler submission format;
- `SchedulerRun`, `TaskStateRecord`, `SchedulerResult`, `SchedulerEvent`,
  failure records, blocked records, resource telemetry, and scheduler
  diagnostics as authoritative scheduler output.

D-008 consumes the raw `SchedulerRun` internally, but its public
`BuildSubmission` surface exposes only an output-free driver scheduler summary:
task states, scheduler events, and scheduler diagnostics. It does not expose
`SchedulerResult.output_refs` or any `SyntheticOutputRef`.

The driver may map scheduler records into build events and session status. It
must not:

- recompute ready queues;
- reorder scheduler results by worker completion time;
- run phase services ahead of scheduler dispatch to fill modeled outcomes;
- reinterpret `CacheHit` as proof evidence;
- turn `Skipped`, `Cancelled`, or `Blocked` into semantic acceptance;
- treat `SyntheticOutputRef` as a real sealed `AnyPhaseOutputRef`;
- publish artifact commits in completion order.

## Cancellation

`cancel(session, reason, snapshots)` is idempotent.

If the session is unknown or terminal, the driver returns a no-op outcome. If
the session is active, D-011 moves it through `Cancelling` to a terminal session
state and appends the matching terminal build event. Explicit requests and
shutdown finish as `Cancelled`; supersession finishes as `Superseded`. A
suppressed-publication event is emitted when the combined lane/snapshot
publication guard returns `Suppressed`.

The current `mizar-build::CancellationPolicy` can express snapshot
supersession and task-scoped explicit cancellation, but it does not expose a
driver-owned snapshot-wide explicit-request or shutdown mutator. D-011 therefore
propagates `DriverCancelReason::Superseded` through `supersede_snapshot`, leaves
explicit-request/shutdown out of the build policy rather than inventing a false
supersession reason, and records snapshot-wide explicit/shutdown policy
propagation as an `external_dependency_gap`.

For watch/LSP supersession, the newer session becomes lane-current and the
older session is cancelled or finished as `Superseded`. Obsolete completed
results still pass through the combined publication guard before any event or
diagnostic can become current.

Task D-014 adds a watch-facing submission helper over the same `submit`
boundary. The helper accepts a `BuildRequestOrigin::Watch` draft whose changed
paths and source snapshot inputs were already supplied by an external owner. It
does not watch the filesystem, debounce changes, load source text, infer module
targets, or convert LSP protocol data. After a newer watch generation is
accepted, the helper updates the previous session's stored replay, even if that
session had already reached a terminal state, so replayed events carry a
suppressed publication decision and a terminal `Superseded` outcome rather than
continuing to claim `Current` status.

If the newer watch request fails before snapshot capture, such as request-id
allocation or snapshot validation failure, the helper leaves the previous
session current and records no snapshot replacement. If snapshot capture
succeeds and later planning, module indexing, task graph construction, registry
checking, or scheduler submission fails, the captured newer session has become
the lane generation under review; the previous session is superseded and replay
suppressed, and any supplied publisher seam is asked to register or replace the
captured snapshot according to the same rules as a successful submission.

When a real `mizar-ir::publisher::PhaseOutputPublisher` is supplied, the first
watch session registers its snapshot as current; later watch submissions call
`replace_current_snapshot(old, new)` when the snapshot id changes. If the newer
generation captures the same snapshot id, replacement is a no-op because the
request/session lane guard still handles supersession. If no publisher is
supplied, or if the owner seam rejects replacement, the result records a
classified gap/error. The driver never mints output handles, artifact
publication tokens, or cache/proof reuse decisions while handling this seam.

Cancellation must not interrupt an artifact manifest transaction after the real
artifact owner reports that the atomic commit has started. The driver does not
own that transaction state; it only consumes owner-provided commit events when
they exist.

## Artifact Boundary

The driver may schedule `ArtifactCommit` tasks because the task graph includes
phase 15. It may expose an artifact-boundary handoff only when the artifact
owner seam returns a real committed result or projection output.

The driver must not:

- serialize artifacts itself;
- mint publication tokens;
- treat scheduler `Completed` as artifact publication;
- publish phase-15 events for stale or obsolete sessions;
- turn cached or retained IR handles into verified artifacts without the
  artifact/proof/kernel owner seams.

## Diagnostics Boundary

Phase services emit diagnostics through `mizar-diagnostics` producer sinks or
return `DiagnosticBatch` values. Planning, graph, and scheduler diagnostics
must be converted through structured diagnostic code/category bridges when the
diagnostics owner provides them.

The driver may transport diagnostic batches and readiness events. It must not
allocate diagnostic codes, deduplicate records, render CLI text, publish LSP
diagnostic payloads, or use message strings as identity.

## Testing Requirements

Task D-008 and D-011 source tests must cover:

- submit captures a session snapshot and marks it submitted/running only after
  the `mizar-build` bootstrap succeeds;
- phase-0 bootstrap uses `mizar-build` planner/module-index/task-graph APIs;
- missing real phase services are reported as classified gaps without
  synthetic outputs;
- scheduler submission consumes `SchedulerInput`/`SchedulerRun` rather than
  duplicating ready-queue semantics;
- scheduler-selected registry dispatch is consumed only through
  `run_scheduler_with_dispatcher`, and missing owner dispatch inputs remain
  classified dispatch gaps;
- stale or superseded sessions cannot publish current diagnostics or
  artifact-boundary events;
- source/lint guards prove D-008 does not allocate diagnostic codes, deduplicate
  diagnostic records, render CLI/LSP diagnostics, serialize artifacts, mint
  publication tokens, or treat scheduler completion as artifact publication;
- cancellation is idempotent, reaches terminal `Cancelled` or `Superseded`
  session outcomes, and appends terminal replay events;
- supersession is expressed through
  `mizar-build::CancellationPolicy::supersede_snapshot`;
- explicit-request and shutdown cancellation do not invent unsupported
  snapshot-wide policy tokens and keep that propagation classified as
  `external_dependency_gap`.

If a test needs a phase service, it may use a test-local fixture only for the
specific implemented behavior under test. Fixture services must not be exported
or documented as real adapters.
