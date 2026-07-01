# mizar-driver Build Events

> Canonical language: English. Japanese companion: [../ja/events.md](../ja/events.md).

## Purpose

`mizar-driver` owns a protocol-agnostic build event stream for build sessions.
The stream lets batch, watch, and LSP-facing callers observe progress,
readiness, blocking gaps, and terminal state without giving the driver
authority over phase semantics, diagnostic identity, artifact publication, or
LSP protocol conversion.

This specification defines the event contract for task D-010. Task D-009 is
documentation-only and adds no Rust source module.

## Ownership Boundary

The event stream may report:

- session lifecycle changes for a `BuildSessionId`;
- snapshot capture and request freshness decisions;
- task/phase progress using `mizar-build` task graph and scheduler identities;
- phase-service availability gaps classified by the registry;
- scheduler-to-registry dispatch gaps such as DRIVER-G-011;
- diagnostics readiness by referencing `mizar-diagnostics` owner records or
  batches when that owner seam provides them;
- artifact-boundary readiness only when the artifact owner reports a real
  committed result;
- session completion, cancellation, supersession, and stale-publication
  suppression.

The event stream must not:

- allocate diagnostic codes or diagnostic identities;
- deduplicate diagnostic records or render CLI diagnostic text;
- emit LSP JSON-RPC payloads, ranges, severities, code actions, or document
  versions;
- mint artifact publication tokens, serialize artifacts, or treat scheduler
  completion as artifact publication;
- expose `SyntheticOutputRef`, `SchedulerResult.output_refs`, or any fake phase
  output handle;
- decide cache compatibility, proof acceptance, trusted status, or kernel
  acceptance;
- reorder artifacts, diagnostics, or phase results by worker completion order.

## Public Enum Compatibility

All public enums in this module are downstream-facing event boundary types and
are marked `#[non_exhaustive]`. D-017 records no exhaustive exceptions for:

- `BuildEventKind`;
- `PlanningEventStatus`;
- `EventOwner`;
- `OwnerGapClassification`;
- `BuildEventError`.

Downstream crates must use wildcard arms when matching these enums. Future
event kinds, owners, gap classifications, or validation errors may be added
without making the driver the owner of diagnostics identity, artifact
publication, proof/cache decisions, or LSP protocol conversion.

## Event Shape

Concrete Rust names may evolve in task D-010, but every build event must carry
the following common identity:

- `session`: the `BuildSessionId` that owns the event;
- `lane` and `generation`: the driver lane/generation from the accepted
  request;
- `snapshot`: the captured `BuildSnapshotId` when the event refers to
  snapshot-scoped results;
- `publication`: the request-layer `PublicationDecision` for events that may
  become visible as current results.

Event kinds are grouped as follows:

| Kind | Meaning | Authority |
|---|---|---|
| `SessionAccepted` | request id/session id allocated and snapshot capture is about to run | `mizar-driver` + `mizar-session` |
| `SnapshotCaptured` | immutable snapshot lease captured for the session | `mizar-session` |
| `PlanningReady` | phase-0 plan/index/task graph data is available, or a structured planning/index/graph error exists | `mizar-build` |
| `TaskProgress` | scheduler/task state changed or a task became blocked/cancelled | `mizar-build` |
| `PhaseServiceGap` | required phase owner seam is missing, deferred, or unavailable | `PhaseRegistry` |
| `DispatchGap` | task graph contains work that cannot be dispatched because the scheduler-to-registry owner seam is unavailable | `mizar-driver` classification of DRIVER-G-011 |
| `OwnerReadinessGap` | diagnostics, artifact, producer, or bridge owner seam needed for a readiness event is missing, deferred, or unavailable | owning crate closeout/gap classification |
| `PhaseReady` | a real phase owner reports a completed/recoverable/blocking/fatal/cancelled phase result | phase owner via registry |
| `DiagnosticsReady` | diagnostics owner has records or batches ready for consumers | `mizar-diagnostics` |
| `ArtifactBoundary` | artifact owner reports an actual committed artifact/projection handoff | artifact owner |
| `PublicationSuppressed` | an obsolete session produced a result that cannot become current | request publication guard |
| `SessionFinished` | session reached a terminal outcome | `mizar-driver` session lifecycle |

`PhaseReady`, `DiagnosticsReady`, and `ArtifactBoundary` are readiness signals,
not ownership transfers. Their payloads must reference owner-provided records,
indexes, or committed results. If the relevant owner seam is missing, the
driver emits a classified gap event instead of a placeholder payload.
`PhaseServiceGap` is for missing phase adapters, `DispatchGap` is for the
scheduler-to-registry dispatch seam, and `OwnerReadinessGap` is for missing
diagnostics, artifact, producer-output, or protocol bridge seams.

The internal architecture sketch includes a `SnapshotPublished(BuildSnapshotId)`
event. In `mizar-driver`, positive snapshot publication is represented by
events whose `publication` field is `Current` after the combined request guard
passes. The LSP bridge, not the driver event stream, owns editor snapshot
publication and protocol conversion. If an implementation chooses to expose a
named `SnapshotPublished` event in D-010, it must be a protocol-agnostic alias
for a `Current` publication decision, not a separate freshness authority.

## Freshness And Suppression

Before any event is exposed as current diagnostics, current build status,
current artifact-boundary handoff, or latest watch/LSP state, the driver must
call the combined request publication guard from [request.md](request.md).

If the guard returns `Current`, the event may be visible to current consumers.
If the guard returns `Suppressed`, the event stream may still emit a
protocol-agnostic `PublicationSuppressed` event to subscribers of the obsolete
session, but it must not relabel the obsolete diagnostics, phase outputs, cache
decisions, or artifact records as current.

Batch sessions use the same guard even though they are normally not superseded.
Watch and LSP sessions must use lane/generation currentness in addition to
snapshot currentness so an older generation cannot publish current results
after a newer generation is accepted.

## Deterministic Ordering

Event ordering is deterministic and independent of worker completion order.
Task D-010 must define a stable ordering key with these components, in order:

1. session-local lifecycle rank;
2. canonical scheduler/task order from `mizar-build` when the event refers to a
   task;
3. pipeline phase order from `mizar-build::task_graph::PipelinePhase`;
4. canonical work-unit identity, such as package, module, VC descriptor, backend
   attempt, or evidence candidate identity;
5. owner-provided diagnostic/artifact order when the event references owner
   records;
6. stable event-kind tie-breaker.

The order may include progress events that mirror scheduler state, but
diagnostic readiness and artifact-boundary readiness must be collated by the
canonical owner order. Events must never imply that the first worker to finish
is the semantic winner, artifact order, or diagnostic order.

Terminal `SessionFinished` appears after all in-session events that are
accepted for publication or suppression. Cancellation and supersession events
use the same ordering key and must remain deterministic under repeated
delivery.

## Diagnostics Events

`DiagnosticsReady` events reference `mizar-diagnostics` records, indexes,
batches, or future owner-provided handles. They may include counts, package or
module identity, severity/category summaries supplied by the diagnostics owner,
and freshness information.

They must not use rendered message text as identity, allocate diagnostic ids,
deduplicate records, or convert to CLI/LSP presentation. CLI rendering belongs
to the CLI entry point through `mizar-diagnostics`. LSP diagnostic conversion
belongs to the LSP bridge.

Planning, module-index, task-graph, scheduler, and driver lifecycle errors may
produce diagnostics-readiness events only after they are represented as
structured owner/driver records. Until the diagnostics bridge exists, D-010 may
emit a classified readiness/gap event instead of inventing diagnostic records.

## Artifact Events

`ArtifactBoundary` events are allowed only after the artifact owner reports a
real committed result or projection output. The event may reference
owner-provided artifact identity, content hash, package/module identity, and
manifest transaction status.

The driver must not produce an `ArtifactBoundary` event from:

- `TaskState::Completed` alone;
- a scheduler synthetic output;
- a retained IR handle without artifact/proof/kernel owner acceptance;
- a provisional publication token invented by the driver.

While the artifact owner seam is unavailable, task D-010 must report an
`external_dependency_gap` or `deferred` event for artifact-boundary readiness
instead of a fake commit event.

## Consumer Rules

CLI consumers may subscribe to the event stream to render progress and choose a
batch exit path. The CLI must use `mizar-diagnostics` to render diagnostics and
must not treat event text as diagnostic identity.

LSP/watch consumers may subscribe to the same protocol-agnostic stream, but the
LSP bridge remains responsible for JSON-RPC ids, document URIs, ranges,
diagnostic severities, code actions, progress tokens, and editor snapshot
publication. The bridge must ignore or translate `PublicationSuppressed` events
as stale-session notifications, never as current diagnostics.

A consumer that reconnects or subscribes late may receive a deterministic replay
of retained session events. Replay must preserve the same ordering and
freshness decisions as live delivery.

## Testing Requirements

Task D-010 tests must cover:

- shuffled scheduler/worker completion produces identical event ordering;
- every event references a known `BuildSessionId` and, when applicable, the
  captured `BuildSnapshotId`;
- stale watch/LSP generations emit suppression rather than current
  diagnostics/artifact events;
- missing phase services and missing artifact publication seams produce
  classified gap events, not placeholder phase outputs or publication tokens;
- DRIVER-G-011 dispatch-gap blocking produces a classified `DispatchGap` event
  and never submits non-phase-0 synthetic scheduler output as current progress;
- missing diagnostics, artifact, producer-output, or bridge readiness seams
  produce `OwnerReadinessGap` events rather than fake readiness payloads;
- scheduler `TaskState::Completed`, scheduler synthetic outputs, and retained
  IR handles without artifact/proof/kernel owner acceptance never produce
  `ArtifactBoundary` events;
- diagnostics readiness carries structured owner identity and never rendered
  message text as identity;
- driver-owned events do not allocate diagnostic ids/codes and do not
  deduplicate diagnostic records;
- CLI/LSP protocol terms do not appear in driver-owned event payloads;
- event replay preserves deterministic ordering and publication decisions.
