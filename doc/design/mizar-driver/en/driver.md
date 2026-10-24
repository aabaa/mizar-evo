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
- optional `mizar-ir` `PhaseDispatchInputProvider<BuildTask>` bundles for
  scheduler-selected registry dispatch. The bundle contains IR-owned phase
  input identities plus validated sealed parent output handles and must match
  the scheduler-selected snapshot. Missing bundles are reported only for tasks
  that reach the scheduler-selected dispatch callback; invalid IR dispatch
  inputs are reported separately and fail closed.

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
   a dispatch preflight. If a scheduler-selected task lacks an owner-provided
   `mizar-ir` dispatch input bundle, the registry-backed dispatcher returns a
   blocked scheduler outcome; the driver records `BlockedByPhaseDispatchGap`
   and the affected phases from that scheduler result. The driver must not
   replace missing parent handles with raw hashes. If a supplied IR bundle is
   for a different snapshot or reports validation failure, dispatch fails with
   an invalid-input diagnostic rather than treating it as a missing bundle.
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
    output references are validated through the supplied IR publisher; producer
    adapters and artifact tokens remain owner seams.
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

`BuildSubmission` exposes scheduler task states, events, and diagnostics plus
`DriverSchedulerRun.phase_results`, ordered by task id and service invocation.
These are actual `PhaseResult` values, never `SchedulerResult.output_refs` or
`SyntheticOutputRef`. `CompilerDriver::with_output_publisher` shares an existing
`Arc<PhaseOutputPublisher>` with scheduled services. The owner registers snapshots
and allowed work units; the driver does not mint or register publication rights.
Returned handles must validate as current package outputs in that publisher and
snapshot. Returned handles without a supplied publisher, foreign/stale handles,
or wrong-snapshot diagnostic batches fail dispatch without retaining the invalid result. Non-complete results
retain diagnostics but no output handles. The session publication guard suppresses
all phase results when the submission is obsolete, including same-snapshot
supersession. IR snapshot/output validity is rechecked before result exposure.
Retained handles are internal
results, not artifact publication authority; consumers revalidate at publication.

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

## Current native Disk diagnostic index

`validate_current_disk_diagnostics(session, snapshots) -> Result<(), String>` retains the all-Disk restriction and delegates the [shared native-session predicates](#current-native-diagnostic-index). OpenBuffer/Generated captures remain unsupported by this Disk-only API; unknown/unsupported/obsolete state returns Err under the shared rules.
`submit_captured_with_diagnostic_index(session, allocator, snapshots, input) -> Result<(BuildSubmission, Option<BuildDiagnosticIndex>), DriverSubmitError>` calls unchanged submit_captured and aggregates only its original actual phase-result batches through Diagnostics from_batches. Require the guard before and after aggregation and an actual scheduler run; otherwise return None without changing the submission. Aggregation failure also returns None; normal submission errors stay unchanged. Failed/blocked real phases may supply diagnostics; clean real runs may supply an empty index. Never infer drafts from scheduler messages or require semantic/proof success. These Disk-only methods exclude Open/Generated captures; overlays, current LSP publication and later consumers remain deferred. Returned indexes are immutable snapshot data; consumers recheck the live guard before later handoff, and still own file/version/map/routing obligations.

## Current native diagnostic index

`validate_current_native_diagnostics(session, snapshots) -> Result<(), String>` owns shared live session eligibility. Read only the Driver's own stored record; require protected native capture, nonempty versions all Disk or OpenBuffer (including mixtures), Finished(Succeeded/Failed/Blocked), the existing live Driver lane/request-snapshot Current decision and a supplied publisher authorizing that snapshot. Unknown, unsupported, cancelled/nonterminal, obsolete or missing-publisher state is Err; Generated/future origins are unsupported. Protected capture retains responsibility for loaded version/path validity.
`submit_captured_with_native_diagnostic_index(session, allocator, snapshots, input)` has the Disk-only wrapper's existing parameter/return types and original-run aggregation contract, calling unchanged submit_captured first and this native guard before and after Diagnostics from_batches. Actual same-call phase-result batches alone are input; actual scheduler run required. Guard/aggregation failure yields None with the original submission; ordinary submit errors propagate unchanged. Supported failed/blocked diagnostics and clean empty indexes do not require later semantic/proof success.
Existing Diagnostics retains all record identity/filter/dedup/sorting authority.
Tests add genuine OpenBuffer and mixed native batches, actual frontend failure/clean emptiness, normal-submission equality and independent record/source/raw-map facts through the existing aggregation oracle and actual NoKey. Retain original Disk/common-live-guard and error tests; add distinct native-origin exclusions and live OpenBuffer supersession controls without duplicating URI, capture or map matrices. Captured raw text/maps remain original after file edits/deletion; no native SourceLoad body-reopen/error producer is invented.
This validates current Driver-session eligibility only. It neither reads Server buffers nor proves the latest editor URI/version/text, source-map or IR lifetime, file/config freshness, arbitrary batch/index authenticity or coherent PublishedLspSnapshot. Later LSP consumers separately bind original raw descriptors/maps to current documents and revalidate before handoff. Returned indexes remain immutable point-in-time data; no scheduling/cache/proof/artifact/publication or full native-host credit follows.

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

## Scheduled SourceLoad/Frontend prefix

Missing later services do not suppress the scheduler-selected prefix when the registry contains only the existing built-in SourceLoad and Frontend descriptor identities, or that exact pair plus the explicitly registered [bounded resolver](frontend_adapter.md#scheduled-source-theorem-module-resolver) with its exact descriptor and concrete service type, or that exact triple plus the explicit [signature service](frontend_adapter.md#scheduled-source-theorem-signature-collection) with exact descriptor and both concrete resolver/signature types, or that exact quartet plus the explicit [type service](frontend_adapter.md#scheduled-source-theorem-type-checking) with exact descriptor and concrete resolver/signature/type services, or that exact quintet plus the explicit [registration service](frontend_adapter.md#scheduled-source-theorem-registration-resolution) with exact descriptor and concrete resolver/signature/type/registration services, or that exact sextet plus the explicit [overload service](frontend_adapter.md#scheduled-source-theorem-overload-resolution) with exact descriptor and concrete resolver/signature/type/registration/overload services, or that exact septet plus the explicit [elaboration service](frontend_adapter.md#scheduled-source-theorem-elaboration) with exact descriptor and concrete resolver/signature/type/registration/overload/elaboration services, or that exact octet plus the explicit [algorithm preparation service](frontend_adapter.md#scheduled-source-theorem-algorithm-preparation) with exact descriptor and all seven concrete semantic service types, or that exact nine-service set plus the explicitly configured [VC generation service](frontend_adapter.md#scheduled-source-theorem-vc-generation) with exact registered/actual descriptor and concrete VC type, and the caller supplies a publisher current for the captured snapshot. Without these prerequisites, retain missing-service preflight blocking. The driver never registers a snapshot or grants publication rights in this route. The exact eleven-service [discharge route](frontend_adapter.md#scheduled-source-theorem-vc-discharge) is also admitted with its real configured type/descriptor and at most one actual VcDischarge task per module; duplicate module labels retain missing-service preflight blocking without a scheduler run. The exact twelve-service theorem ATP route is additionally admitted only with the actual configured ATP TypeId/descriptor and current publisher, retaining the existing at-most-one Discharge task per module preflight.
A supplied dispatch provider remains authoritative, including None/error/invalid bundles. Only with no provider, prepare SourceLoad input from the unique matching captured SourceVersion using the frontend adapter’s shared SourceUnitCacheKey derivation, no dependencies or parents. Frontend uses that same key, the sorted multiset of captured index summary hashes and exactly one completed SourceLoad output from this task's dependencies for the same work unit, plus retained Complete Frontend/FrontendOutput results from those dependency task ids; seal all through the current IR publisher. PackageResolve contributes no output. Complete or ImportsOnly ImportSummary graph edges order workspace Frontends; the frontend adapter authenticates supported workspace parents and actual imports, rejecting missing or surplus workspace parents. A cached task without a real retained output cannot supply a parent. The [bounded resolver](frontend_adapter.md#scheduled-source-theorem-module-resolver) additionally gains default input from its actual unique same-module Frontend dependency; only the exact leading CheckAndElaborate [signature prefix](frontend_adapter.md#scheduled-source-theorem-signature-collection) additionally receives its genuine ModuleResolve dependency while immediate TypeChecking is missing. The exact five-service [type prefix](frontend_adapter.md#scheduled-source-theorem-type-checking) instead constructs TypeChecking input from its own task's actual Complete signature output while immediate RegistrationResolution is missing. The exact six-service [registration prefix](frontend_adapter.md#scheduled-source-theorem-registration-resolution) additionally hands the same task's actual Type output to RegistrationResolution while immediate OverloadResolution is missing. The exact seven-service [overload prefix](frontend_adapter.md#scheduled-source-theorem-overload-resolution) then constructs OverloadResolution input from this same task's actual Type and Registration outputs while immediate Elaboration is missing. The exact eight-service [elaboration prefix](frontend_adapter.md#scheduled-source-theorem-elaboration) additionally constructs Elaboration input from this same task's actual Metadata output while immediate AlgorithmPreparation is missing. The exact nine-service [preparation prefix](frontend_adapter.md#scheduled-source-theorem-algorithm-preparation) additionally constructs AlgorithmPreparation input from this same task's actual Core output while the separate VcGenerate service is missing. The exact ten-service [VC route](frontend_adapter.md#scheduled-source-theorem-vc-generation) enables all those grouped bridges and constructs the separate VcGenerate bundle from its unique qualifying dependency’s exact six retained rows, using both actual Core and Preparation; other phases gain none. The eleven-service route inherits the old grouped/VC preparation and supplies actual retained Core/VC parents to VcDischarge; its service owns the bounded Vc-task to module-data association. The twelve-service route additionally supplies AtpSolve with actual Core, original VC and Discharge handles: select its unique direct complete D row, recover the unique retained original VC and six-row group by exact D/VC/Core/Preparation links and matching snapshot/module/source/schema metadata, and seal only those three real physical refs.
For the prefix exception, mark supplied ValidatedHit decisions for tasks with missing services Unavailable while retaining task ids for normal scheduler validation; available-service cache behavior is unchanged. Retain missing-service classifications and events after scheduling. Failed frontend/source tasks keep the session Failed; a clean prefix stays Blocked when downstream services are absent, using BlockedByMissingPhaseServices unless a selected owner-input gap requires BlockedByPhaseDispatchGap. Selected missing services are classified before default input construction. Cancellation, parent validation and final lane/publisher currentness still govern execution/reporting; the bounded resolved output, separately registered signature summary, supported TypeChecking output, registration closure, whole metadata, theorem Core and theorem preparation gain phase credit only within their documented profiles. Earlier incomplete CheckAndElaborate prefixes remain Blocked. In the exact nine-service preparation route, six Complete results finish that grouped task; the separate VcGenerate task blocks at its missing-service precheck without phase rows, leaving downstream work and the session Blocked. The exact ten-service route also completes that separate VC task; with empty explicit descriptors the next actual missing-service block is ArtifactCommit with no artifact rows, not dynamically created discharge tasks. Proof/cache/artifact and complete semantic-service success do not follow from prefix completion. The eleven-service route additionally completes actual descriptor-declared singleton discharge processing with module NeedsAtp data; declared ATP/evidence tasks and Artifact retain their actual missing-owner blocks. No dynamic task or proof acceptance follows. In the exact twelve-service profile only the precise AtpSolve/[AtpSolve]/Vc task gains this bridge; old eleven-service missing-ATP precedence remains exact. Actual AtpSolve completion next reaches missing BackendRun, with any explicitly declared KernelCheck dependency-blocked; Artifact/session remain Blocked and no checking/cache/acceptance follows.

The exact thirteen-service [backend-free check route](frontend_adapter.md#scheduled-source-theorem-candidate-check) additionally requires the actual configured Kernel service TypeId/descriptor and at most one actual Kernel task per module, retaining the existing D limit. Only precise KernelCheck/[KernelCheck]/EvidenceCandidate uses the ATP bridge's direct Complete D, retained original VC and six-row group joins to supply three real sealed Core/VC/D parents. Query stays EvidenceCandidate/three parents while the service publishes MODULE/four-parent data. Provider authority, old profiles and missing-service precedence remain unchanged. Backend-bearing graphs still block Kernel behind missing BackendRun; backend-free direct-D checking can complete, leaving Artifact/session blocked and source acceptance/cache/status unchanged.

## Import-discovered native diagnostic index

`submit_captured_with_import_discovery_and_native_diagnostic_index<A: SessionIdAllocator, L: SourceLayoutProvider>(&mut self, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>) -> Result<(BuildSubmission, Option<BuildDiagnosticIndex>), DriverSubmitError>` first calls existing submit_captured_with_import_discovery once, then applies the existing native guard and original-run Diagnostics aggregation. Errors/submission, actual ImportsOnly coverage, empty-Unavailable eligibility, supplied-overlay precedence and whole-discovery failure fallback remain unchanged; never guess edges or promote semantic completeness.
Private `current_diagnostic_index` borrows the actual submission and registry and receives the existing Disk/native validation function. Its three callers are the unchanged ordinary Disk entry, unchanged ordinary native entry, and this opt-in entry. Require an actual scheduler run, validate before collecting original phase_results batches, call unchanged Diagnostics from_batches with the actual snapshot, then validate again. None/errors retain the original submission. No stored/copied alternate run/index or new representation; old entries still call ordinary submit_captured first, with their respective guards and identical aggregation order.
Append three genuine families reusing existing Fixture/capture/publisher/services: native Disk/Open/mixed import-aware executions and full equality with the existing discovery submission plus original aggregate/independent record facts and actual NoKey; discovery failure and supplied-overlay precedence without fabricated positive Complete coverage; eligibility/currentness and structured submit errors with viable controls. Retain source IDs/raw maps after disk edits, and reuse existing lower import, URI/map, Disk-only, common-guard and error matrices instead of copying them. Any new setup function serves all three families; old tests remain exact.
This explicit composition adds no import-resolution semantics, diagnostic identity/code/filter policy, source reload or cache/acceptance authority. Opaque context and metadata hashes gain no runtime authentication. Current editor tuples/lifetimes/maps and host/wire binding stay with LSP; complete semantic imports, runtime/current-build publication and broader gates remain deferred.

## Pregraph import discovery

`CompilerDriver::submit_with_import_discovery` opts into authenticated disk-source preflight after plan/index construction and before graph construction. It replaces only an empty Unavailable overlay; other supplied coverage or edges remain authoritative. Ordinary `submit` retains all supplied overlays, including intentional Unavailable. The existing frontend adapter owns source acquisition and provisional path resolution; the driver core only consumes its overlay. Preflight is not phase execution and publishes no output or diagnostics; its allocator and source identities are private.
Require every workspace module to have one matching captured disk version and workspace plan/index identity, safe package/source paths contained in the workspace, and loaded package/module/path/edition/hash/origin equal to capture. Use the session loader, frontend preprocessing and resolver provisional import mapping. Reject unreadable, drifted, unsupported, duplicate/missing, preprocessing-diagnostic or unresolved input as a whole, retaining Unavailable rather than partial inferred edges; existing graph rejection then prevents scheduling. Resolved artifact targets need no workspace edge. Otherwise emit deduplicated workspace ImportSummary edges as ImportsOnly, never Complete semantic coverage. Graph self/cycle/reference validation still applies.
The scheduled SourceLoad rechecks disk content against capture; Frontend authenticates actual parsed imports, retained parents and supported summaries before publication. Preflight cannot authorize reexports, semantic imports, public diagnostics, proof/cache/artifact success or open-buffer acquisition. Changes between preflight and scheduled loading cannot publish altered input under the captured key.

## Published output authorization

[Publisher authorization](../../mizar-ir/en/publisher.md#published-output-authorization)
can establish genuine OpenBuffer/InternalOnly publication provenance. Driver
consumers retain current/package checks until native capture and input binding
are supplied; provenance alone never authenticates a summary-only/Disk result.
Physical storage, current snapshot and lane checks remain required. Canonical
snapshot equality alone never proves raw-buffer/version freshness.

## Captured native submission

`submit_captured(session, allocator, snapshots, input)` and `submit_captured_with_import_discovery` consume a genuinely native-captured session. Non-native/non-SnapshotCaptured, a session id already stored in this Driver, or a non-current registry request fails with `InvalidCapturedSession { session }` before lane mutation. Prepared clones may be submitted once per Driver; legacy submits keep normal allocation/capture.
The caller's publisher owner authorizes actual snapshot/work units first; Driver never reacquires, guesses IDs, registers/revives snapshots or grants rights. Existing planning, prefix status, lane/request guards, cancellation and resource/error handling apply.
Native resources use the existing borrowed [service inputs](frontend_adapter.md#native-captured-sources). Only genuine native dispatch adopts published parent/result/Frontend-parent/final-retention authorization; legacy retains all four current/package checks. Complete captured source/version/root binding remains mandatory. Later services and LSP current diagnostics remain deferred.

Rejected native preconditions produce no accepted watch replacement. CLI maps both errors to its existing InternalError envelope with `invalid_native_submission`, without phase diagnostics or events.

Raw/version changes can retain canonical snapshot identity while changing storage lineage. The existing [IR identity](../../mizar-ir/en/identity.md) conflicting-slot rule still rejects same-owner conflicting publication; fresh owner publishers are required for these positive replays. Old handles remain subject to physical/source/lane guards.

## Explicit native Disk cached SourceLoad scheduling

`CompilerDriver::submit_captured_with_native_disk_cache<A: SessionIdAllocator, L: SourceLayoutProvider>(&mut self, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>, policy: &mizar_proof::policy::VerifierPolicy, store: &mizar_cache::cache_store::CacheStoreRoot) -> Result<BuildSubmission, DriverSubmitError>` is an explicit source-only optimization. It uses the ordinary captured-session validation, planning, graph, scheduler and final lane/publisher guards. Ordinary entry points and both registered NoKey intents stay unchanged. This route replaces caller cache decisions with its own genuine SourceLoad decisions; it never accepts string references as typed outputs.
Only CacheSchedulingPolicy::Enabled, no supplied dispatch provider, the exact built-in SourceLoad/Frontend descriptors AND concrete service types, nonempty all-Disk native capture, a current supplied publisher and explicit successful native lock/config content comparisons enable lookup. Other profiles execute normally with no cache decisions. Require the registered objects’ Any::type_id to match SourceLoadService and FrontendService, obtained through the existing crate-local service lookup; freely supplied descriptors alone cannot authenticate implementations. Existing entry points do not acquire this new restriction. The caller chooses the Store and immutable policy; content equality and this explicit association do not prove full active configuration, released toolchain, file/dependency freshness or global runtime provenance.
After the actual graph exists, construct SourceLoad inputs from its exact module and unique captured version using the existing source_input_hash, no dependencies/parents. Use the existing guarded native Disk lookup and same supplied publisher through IrCacheAdapter. Only a genuine Rehydrated SourceUnit with successful current physical validation and IR retention may create a source-only Complete PhaseResult with that actual handle, empty diagnostics and no invented cache query observation, and a matching scheduler decision projected from its actual identity/hash. All misses/errors, unsupported binding and retention failures leave ordinary SourceLoad execution eligible; never execute a phase service ahead of its scheduler task or insert Store records.
Keep prepared cached results private under exact task IDs. Default Frontend may consume them only after normal graph dependency readiness accepts the SourceLoad skip; no new scheduler callback or readiness policy is added. At final reporting retain a prepared result only if that exact task actually finished CacheHit; disabled, cancelled, superseded, obsolete or unaccepted skips produce no cached PhaseResult. Existing final physical/publisher/lane checks still suppress all stale results. Do not fabricate results for missing/later services or treat prefix completion as full build success.
Retain each genuine cached source under a distinct per-active-Driver/session/task opaque IR owner through scheduler consumers and final validation; ownership labels carry no semantic identity. Release every acquired owner on every normal success/error/cancellation/supersession return after lookup. Returned handles have the same caller-managed post-return lifetime as ordinary results; retention never revives currentness and a Session lease is not an IR slot retain. Reuse the existing dispatcher result map and local ownership tuples, with no new type, field, adapter, stored state or source file.
Tests use real scheduled old SourceLoad output, existing encoder and actual Store insertion, new captured inputs and this explicit route with default Frontend inputs. Compare genuine resident/blob source and Frontend payloads/maps/diagnostics with clean execution and check actual CacheHit task/result/parent association. Add changed raw storage/miss and unsupported-profile fallback, cancellation/error release and stale-result controls with viable positive cases; preserve existing actual NoKey and unretained-string-hit rejection tests. Reuse the existing Registry fixture with the real built-in descriptors from public descriptor lookup; require its borrowed PhaseService Any TypeId to remain TypeId::of::<FixtureService>() while those descriptors change. Actual cache-route positives use the real built-ins and exercise their concrete guards. The external fixture does not name private built-in types or observe their TypeIds; this fixture identity invariance and inspected route conjunction do not claim a direct cross-type comparison or an executed full native cache rejection with a fake producer. Check owners are released using actual storage collection; reuse existing lifetime/producer matrices instead of duplicating them. No fictional cache record, result, hit or proof acceptance is a positive oracle.
This completes neither full automatic runtime/policy/root association nor Frontend cache producers, later services, complete cached equivalence, artifacts, proof or Steps6–8. The current native source lookup and IR acceptance already exist; this owner adds only Driver association and scoped lifetime around genuine scheduled skips.

## Package-configured native Disk cache scheduling

`CompilerDriver::submit_captured_with_configured_native_disk_cache<A: SessionIdAllocator, L: SourceLayoutProvider>(&mut self, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>, policy: &mizar_proof::policy::VerifierPolicy, store: &mizar_cache::cache_store::CacheStoreRoot) -> Result<BuildSubmission, DriverSubmitError>` opts into package settings while reusing the preceding genuine SourceLoad scheduling route, its full eligibility checks, lookup, scheduler acceptance, default Frontend consumption and scoped retention/release. Keep the existing caller-selected route and ordinary entries unchanged; carry this private choice in the existing optional cache tuple, without another loop or runtime object.
Before lookup for each SourceLoad module, require its unique PackagePlan and Workspace source, that package's build_config.incremental, and a non-dot cache_dir passing the existing crate-local safe_relative guard unchanged. This preserves Build's current lexical path profile for arbitrary directly typed configs without reimplementing its parser. Share only that existing guard by crate visibility; do not broaden it or introduce public path APIs.
Require Store.root() to equal Path::new(build_plan.workspace_root.as_str()).join(package_workspace_root).join(cache_dir) using native Path equality, with no canonicalization, new filesystem access or directory creation. A missing/ambiguous package, registry source, disabled/invalid setting or nonmatching root simply withholds that task's cache preparation; ordinary work and global caller cache policy still govern. Do not take importer/first-aggregate/default registry settings. Filesystem aliases may fall back; this comparison neither attests storage freshness nor changes path syntax or symlink policy. A missing matching store reaches ordinary lookup/miss fallback.
Tests acquire actual parsed package settings and capture real sources; warm through scheduled production, encode and insert genuine records, then observe actual source skips and default Frontend parents/results. Cover resident/blob, default/nested configured directories, global disable/miss/foreign stores with otherwise-viable old-route controls, package opt-out and invalid directly typed paths, and distinct multi-package settings/directories under reordered inputs/workers. Compare actual payloads/maps/diagnostics with clean execution and preserve all existing tests. Inline first; at most one new private setup helper shared by three actual consumer families, no other new test types/adapters.
This is an explicit retained-metadata optimization profile. Policy/Store remain caller-supplied; complete active configuration, selected toolchain/dependencies, current files, automatic host/runtime, Frontend cache expectations, later services, artifacts and Steps6–8 remain separate.

## Persistent native Disk SourceLoad cache

`CompilerDriver::submit_captured_with_persistent_native_disk_cache<A: SessionIdAllocator, L: SourceLayoutProvider>(&mut self, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>, policy: &mizar_proof::policy::VerifierPolicy, store: &mizar_cache::cache_store::CacheStoreRoot) -> Result<BuildSubmission, DriverSubmitError>` explicitly opts into configured lookup and best-effort persistence. Extend only the existing private cache tuple with a write flag: both old methods select false, the new entry selects configured=true/write=true. Ordinary methods stay unchanged. Collect local (actual TaskId, cloned PhaseInput) candidates only after the shared configured eligibility and unique version/input construction, before lookup. No independent guard, scheduler, source execution or persistent runtime state is added.
After scheduling and the existing full physical/publisher/lane result filtering, withhold all writes if any final task state is Cancelled. For each collected candidate require that exact source task to be Completed, exactly one Complete result and exactly one output; CacheHit/blocked/skipped/cancelled tasks supply no cold producer. Downstream Frontend failure may coexist with a valid completed SourceLoad and does not prohibit that source record. Missing later services still mean an incomplete build.
Use the same actual capture/plan/index/allocator and publisher through SourceLoadInputs and IrCacheAdapter. Recover the actual typed SourceUnit and call encode_native_disk_cache_record; its complete native binding/current physical output/storage/side-table validation remains authoritative. Only Encoded(record) may reach Store.insert. Before each insertion, after encoding, freshly require current lane/request/snapshot, publisher current-output authorization and physical handle validity. Guard loss stops subsequent writes; failed/skipped encoding only skips that candidate. No snapshot registration, grant, new key/codec/hash/schema or cached frontend is produced.
Store insertion is best-effort internal optimization: errors/rejected outcomes neither fail the build nor alter diagnostics, submission/status, policy or proof results; never claim insertion succeeded. Retain Store's complete-record validation, idempotence and divergence/atomic-write behavior. Currentness checks are handoff-time only; Store I/O is not atomic with lane/publisher state and earlier complete records are not rolled back. SourceLoad's existing complete zero-artifact-dependency footprint permits these internal records before artifact emission, without publishing an artifact or witness.
Tests start with an absent Store and use no test-side insert: observe actual cold execution and inserted record via the genuine native key, then fresh capture/local IDs and a real warm skip/default Frontend. Compare independent clean outputs/maps/side tables/diagnostics and release, resident/blob, source/raw edits and failed Frontend prefixes. Cold controls distinguish disabled/ineligible roots/provider/context, cancellation and completed-obsolete real producers, currentness suppression and a real file blocking Store creation, with viable positives. Reuse existing broader shared-route matrices; preserve all old tests. At most one new private setup helper shared at three actual family sites; no other new test types/helpers/adapters.
This separate opt-in preserves both older read-only contracts, including absent-store behavior. Complete runtime identity, policy/toolchain/dependency provenance, Frontend independent expectations/lookup, later services, full cached equivalence, artifact/host/ENTRY and Steps6–8 remain open.

## Persistent native leaf Frontend cache

`CompilerDriver::submit_captured_with_persistent_native_prefix_cache` takes the same arguments and returns the same result as `submit_captured_with_persistent_native_disk_cache`. It explicitly enables configured SourceLoad persistence plus best-effort post-production Frontend persistence. Extend only the existing private cache tuple with a Frontend write flag, false for all existing entries; reuse shared configured eligibility, pending source task/input associations, scheduler and finalized physical/publisher/lane filtering. No pre-execution Frontend lookup or extra phase execution occurs.
Withhold all writes on any final Cancelled task. For each eligible pending source association require its actual source task to have finished Completed or CacheHit with one Complete result and one genuine output. Find exactly one same-work-unit single-Frontend task with exactly two graph dependencies: that source task and the actual Workspace/PackageResolve root with phase PackageResolve. Require that root and Frontend task Completed, one Complete diagnostic-free Frontend result and one output; reject every other dependency. Construct its PhaseInput from the pending source input identity and actual sealed published source parent. Do not substitute another task's result, cached string, retained oracle or obsolete output.
Recover the actual typed FrontendOutput through the same publisher and call the unchanged encode_native_disk_frontend_cache_record with genuine capture/plan/index/allocator and that PhaseInput. Its current physical parent/output, exact lineage, raw source/maps, leaf/no-import/dependency/diagnostic and complete payload checks remain authoritative. Only Encoded(record) reaches Store.insert, after a fresh lane/request/snapshot, parent/output current authorization and physical validity check; guard loss stops subsequent writes. Failed/skipped encoding only skips the candidate.
Insertion errors/rejected outcomes preserve ordinary status, diagnostics and proof results. These zero-artifact-dependency internal records assert no artifact acceptance or publication. Checks occur at handoff and are not atomic with Store I/O; earlier writes are not rolled back. The source-only persistence entry retains its old behavior; older read-only entries write neither record kind.
Tests start with an absent Store and observe Driver-produced records via independently prepared current keys and retained same-capture lookup; no positive test inserts a record. Compare actual resident/blob aggregates, maps, side tables, parents, diagnostics and registered NoKey observations with independent clean runs, including a genuine SourceLoad CacheHit parent, source/raw edits and package/order/worker controls. Cold disabled/ineligible/provider/context, cancelled/completed-obsolete, failed/imported Frontend and real Store-error controls retain viable ordinary runs and preserve the old test prefix. At most one new private setup helper serves three actual families.
This explicit persistence does not establish cold Frontend expectations, retained scheduler skips, full active runtime/toolchain/dependency identity, imports/Open caching, later services, artifact/host/ENTRY or Steps6–8 completion.

## Retained native Frontend cache scheduling

`CompilerDriver::submit_captured_with_retained_native_frontend_cache<A: SessionIdAllocator, L: SourceLayoutProvider>(&mut self, session: BuildSession, allocator: &A, snapshots: &SnapshotRegistry<A>, input: DriverSubmitInput<L>, cache: (&mizar_proof::policy::VerifierPolicy, &mizar_cache::cache_store::CacheStoreRoot), oracle: (&crate::registry::PhaseInput, &mizar_ir::cache_adapter::IrCacheAdapter, &crate::registry::PhaseResult)) -> Result<BuildSubmission, DriverSubmitError>` explicitly associates one caller-supplied genuine retained Frontend oracle with the same protected native Disk capture/current SourceIds. Group policy/Store as one existing tuple; carry the optional borrowed result oracle in the existing private submission-options tuple beside discover_imports/prepared, absent in every older entry; preserve the existing five-element cache tuple. This selects the existing caller-chosen read-only SourceLoad cache route, without configured-root/persistence changes. Ordinary entries, registered NoKey intents and query observations remain unchanged.
Reuse all existing native cache eligibility, concrete built-in service checks, native lock/config content comparisons, SourceLoad lookup and scheduler setup. Attempt Frontend lookup only after a genuine SourceLoad Rehydrated output has passed physical/current validation and retention. Require exactly one actual graph SourceLoad task for the oracle work unit, one Workspace/PackageResolve/[PackageResolve] root, and one same-unit [Frontend] task whose dependencies are exactly that source task and root. Reject ambiguous or other dependency shapes. Source misses and unsupported eligibility supply no Frontend preparation; they remain ordinary scheduled work. Do not execute the root or either phase ahead of the scheduler.
Construct the target Frontend PhaseInput from that source task's actual unique captured version/source hash and its own sealed published source handle. Require the supplied actual PhaseResult to be Complete, diagnostic-free and contain exactly one output. Recover its typed Frontend handle through the oracle adapter storage and use unchanged rehydrate_native_disk_frontend_cache with that handle and target adapter. Keep phase types in the owning Frontend adapter; Driver consumes the existing PhaseResult protocol and infers the handle type at that call, as in existing persistence. Do not add aliases/reexports, lint suppressions or weaken the core-authority guard. Its exact capture, source IDs, dispatch identities, equal deterministic source-parent ID, independent whole oracle bytes/keys/side tables, current physical publisher, producer, decoder and leaf-profile guards remain authoritative. A generic published oracle is no concrete execution attestation; the caller supplies genuine production and concurrent oracle lifetime. A different storage without the identical Frontend output permits reconstruction; existing duplicate-output failures simply withhold preparation, with no deduplication, permission or publication workaround.
Only a genuine Frontend Rehydrated handle that passes physical/current validation and explicit IR retention may create one Complete diagnostic-free private PhaseResult with its actual erased handle and cache_observation=None under the exact Frontend task ID, plus a matching scheduler decision projected from that handle's real identity/hash. All misses/errors/retention failures leave normal execution eligible; caller string decisions cannot supply accepted handles. Reuse local ownership tuples with erased actual handles for both payload kinds, not new types/fields/adapters. The usual graph readiness, pending cancellation and publication freshness must accept each skip independently; root completion and source acceptance are never inferred from preparation.
Keep a prepared result in reporting only if that exact task finally CacheHit. Existing whole-map physical/publisher/lane filtering still suppresses obsolete results. Retain each target source/Frontend under a distinct active Driver/session/task owner through scheduler consumers and final validation; release all acquired owners on scheduler error and normal return, including cancellation/supersession. Retention neither grants currentness nor changes caller-managed post-return lifetime. Existing read-only routes and this entry write no Store records.
Append three retained_native_frontend_cache_* families using unchanged genuine Fixture/capture_sources/built-ins/encoders/Store and independent clean runs; at most one new private setup helper must serve all three. Cover resident/blob whole aggregates, maps, complete sides/parents/diagnostics, source and Frontend TaskStarted absence only for actual CacheHit, immutable captured inputs and replay/input-order/worker controls. Exercise otherwise viable source/Frontend misses, foreign or stale/collected oracle, binding/context/provider/disabled and dependency-shape fallback, real resource failure/cancellation/obsolete release with actual collection, reusing existing lower matrices and NoKey tests. Review the preserved scheduler-Err release loop statically: this valid-graph public route supplies no invalid outcomes/decisions or missing root to reach that branch. Borrow the genuine oracle PhaseResult from actual scheduled results, never reconstruct a positive Complete result; add malformed status/diagnostic/output-cardinality controls as adversarial derivatives. Adversarial derivatives are negative controls only.
This is explicit same-capture retained scheduling, not cold record-independent Frontend expectations, fresh-capture/process reuse, atomic runtime lifetime, full active configuration/toolchain/dependency authenticity, import/Open caching, semantic/proof/artifact services, full STEP6C or Steps6–8 completion. The caller-owned oracle association and unchanged leaf footprint retain those deferrals.
