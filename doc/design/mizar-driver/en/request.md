# Module: request

> Canonical language: English. Japanese companion:
> [../ja/request.md](../ja/request.md).

## Purpose

This module defines the driver-owned build request and build session boundary.

`mizar-driver` accepts batch, watch, and LSP-originated build requests, captures
one validated `mizar-session` snapshot for each submitted session, and exposes a
current-session publication guard that prevents obsolete watch/LSP sessions
from publishing as current. It does not own source loading semantics, phase
semantics, diagnostic identity, artifact serialization, or LSP protocol
conversion.

## Ownership Boundary

`request` owns:

- request shapes for batch, watch, and LSP-originated builds;
- the driver-level currentness lane used to replace sessions for superseding
  watch/LSP generations without redefining `mizar-session::BuildRequestId`;
- `BuildSession` records that bind one `BuildSessionId`, one request lane, one
  `BuildSnapshot`, and the active snapshot lease returned by
  `SnapshotRegistry::create_snapshot`;
- lifecycle state transitions used by scheduler submission, cancellation, and
  event publication gates;
- the publication guard that checks both the driver-owned lane-current session
  and the `mizar-session` request-generation current snapshot before exposing
  diagnostics, build events, or artifact-boundary handoff as current.

`request` does not own:

- parsing source text, resolving imports, type checking, VC generation, proof
  acceptance, trusted status, or kernel acceptance;
- cache compatibility, cache-key validation, or proof-reuse validation;
- artifact manifest transactions, artifact serialization, or publication-token
  issuance;
- LSP document-version protocol conversion, range conversion, code actions, or
  editor commands.

## Public Enum Compatibility

All public enums in this module are downstream-facing driver envelopes and are
marked `#[non_exhaustive]`. D-017 records no exhaustive exceptions for:

- `BuildRequestOrigin`;
- `LspPriority`;
- `BuildSessionState`;
- `BuildSessionOutcome`;
- `PublicationDecision`.

Downstream crates must use wildcard arms when matching these enums. Crate-local
code may still match the known variants internally when a new variant should be
reviewed with the request/session lifecycle.

## Data Model

### Request Identity And Currentness Lane

`mizar-session::BuildRequestId` identifies one batch/watch/LSP request
generation. The driver must allocate a fresh `BuildRequestId` for each
submitted generation and pass it to `SnapshotRegistry::create_snapshot` and
`SnapshotRegistry::is_current_for_request`.

Supersession is driver-owned and uses a distinct `BuildLaneId`. For batch
builds, a lane normally has exactly one generation. For watch and LSP builds,
multiple edit generations may share one lane so a newer session can supersede
older sessions atomically even when two generations produce the same
`BuildSnapshotId`. The lane and generation are driver-owned metadata on
`BuildRequest`; neither is part of `BuildSnapshotId`, and neither may be used as
cache or artifact compatibility authority.

### BuildRequest

```rust
struct BuildRequest {
    id: BuildRequestId,
    lane: BuildLaneId,
    origin: BuildRequestOrigin,
    generation: BuildRequestGeneration,
    workspace_root: WorkspaceRoot,
    profile: BuildProfile,
    targets: BuildTargets,
    source_inputs: SourceInputSet,
    dependency_inputs: DependencyInputSet,
    verifier_config: VerifierConfigInput,
}

enum BuildRequestOrigin {
    Batch(BatchRequest),
    Watch(WatchRequest),
    Lsp(LspRequest),
}
```

`BuildRequest` is a driver input envelope. It names what should be built and
which source/dependency state should be captured into a snapshot. It must not
contain mutable phase outputs, cache decisions, artifact handles, LSP protocol
payloads, or pre-rendered diagnostic text.

Batch requests are produced from CLI arguments plus package defaults. Watch
requests are produced by a watch-facing orchestrator after an owner-provided
file-change/coalescing layer has normalized changed paths and source snapshot
inputs. The driver does not own OS file watching, debouncing, source loading,
or file-to-module discovery rules. LSP requests are produced by the
`mizar-lsp` bridge after protocol conversion; the driver receives
protocol-agnostic open-buffer source inputs and priority hints only.

### Request Origins

```rust
struct BatchRequest {
    invocation: BatchInvocation,
}

struct WatchRequest {
    watch_root: WorkspaceRoot,
    changed_paths: Vec<NormalizedPath>,
}

struct LspRequest {
    focus: Option<LspFocus>,
    priority: LspPriority,
}
```

Batch requests may run to a terminal status without superseding another
session. Watch requests supersede previous sessions in the same watch lane when
the next coalesced generation is accepted, including when that generation
captures the same `BuildSnapshotId`; same-snapshot replacement is an IR
publisher no-op, not a reason to keep the older session current. LSP requests
supersede previous sessions in the same LSP lane when newer open-buffer inputs
should replace older diagnostics for that lane.

`LspRequest` must stay protocol-agnostic. It may refer to source inputs already
normalized by `mizar-session` and priority information selected by `mizar-lsp`,
but it must not contain LSP ranges, document URIs as protocol payloads, JSON-RPC
ids, code-action edits, or rendered diagnostic objects.

### BuildSession

```rust
struct BuildSession {
    id: BuildSessionId,
    request: BuildRequest,
    captured: CapturedSnapshot,
    state: BuildSessionState,
}

struct PendingBuildRequest {
    session_id: BuildSessionId,
    request: BuildRequest,
}

struct CapturedSnapshot {
    snapshot: BuildSnapshot,
    active_snapshot_lease: SnapshotLease,
}

enum BuildSessionState {
    SnapshotCaptured,
    Submitted,
    Running,
    Cancelling,
    Finished(BuildSessionOutcome),
}

enum BuildSessionOutcome {
    Succeeded,
    Failed,
    Blocked,
    Cancelled,
    Superseded,
}
```

A session represents one scheduler run over one immutable snapshot. The session
id is allocator-issued and local to driver/scheduler bookkeeping. The snapshot
id is content-derived and identifies the source/dependency/toolchain/verifier
input state. Task results belong to the session snapshot, not to the session id,
and may be reused in another session only through owner-provided cache
validation against the target snapshot.

`PendingBuildRequest` is the pre-snapshot record for an accepted request. A full
`BuildSession` exists only after snapshot capture. `SnapshotCaptured` means
`SnapshotRegistry::create_snapshot` returned a `BuildSnapshot` and an
`ActiveBuild` lease. `Submitted` means the driver has handed the planned graph
to the scheduler. `Running` means at least one scheduler-owned task may still
produce events for the session. `Cancelling` means cancellation has been
requested and in-flight tasks are expected to stop at safe checkpoints.
`Finished` is terminal.

`Superseded` is terminal for a watch/LSP session whose lane now points at a
newer current session. A superseded session may still have stale diagnostic
data or retained explanation handles, but it must not publish those outputs as
current.

## Snapshot Capture

The driver builds a `mizar-session::SnapshotInput` from already-loaded source
versions, dependency artifact references, lockfile identity, toolchain identity,
and verifier configuration identity. Snapshot identity and validation remain
owned by `mizar-session`.

The only public creation path for a registry snapshot is:

```rust
SnapshotRegistry::create_snapshot(request.id, snapshot_input)
```

The returned `BuildSnapshot` and `SnapshotLease` become the `CapturedSnapshot`
part of `BuildSession`. The lease reason must be
`RetentionReason::ActiveBuild`. The driver may bridge the lease into
`RetentionManager` when the later retention handoff is needed, but retaining a
snapshot must not make it current. Request-generation currentness is still
checked through `SnapshotRegistry::is_current_for_request`, while watch/LSP
supersession currentness is checked through the driver-owned lane table.

Identical canonical snapshot inputs must produce identical `BuildSnapshotId`
values even when they appear in different sessions. Reusing a
`BuildSnapshotId` does not imply that allocator-issued session, request, source,
source-map, or lease ids are reusable.

## Current-Session Publication Boundary

Before the driver publishes any session output as current, it must check:

```rust
driver_lanes.is_current_session(
    session.request.lane,
    session.request.generation,
    session.id,
    session.captured.snapshot.id,
) &&
snapshot_registry.is_current_for_request(session.captured.snapshot.id, session.request.id)
```

Both checks are required. The driver-owned lane check rejects obsolete
watch/LSP sessions, including the case where an older and newer generation have
the same `BuildSnapshotId`. The `mizar-session` registry check confirms that
the captured snapshot is still the current snapshot for the request generation
that created it.

The combined check is required for:

- current diagnostics readiness;
- current build-event stream publication;
- final session status reported to a watch/LSP consumer;
- artifact-boundary handoff events when a real artifact owner seam reports a
  committed result;
- any later driver API that exposes "latest" build state.

If either check fails, the request layer must return a suppressed publication
decision. The later event layer may turn that decision into a protocol-agnostic
stale/suppressed event for observers that are still subscribed to the old
session. The driver must not relabel stale diagnostics, phase outputs, cache
decisions, or artifact records as current.

Batch sessions normally have no superseding generation, but they still use the
same guard so all driver publication paths share one freshness rule.

## Supersession

Watch and LSP orchestration supersede a session by creating a newer request
generation with a fresh `BuildRequestId` in the same driver `BuildLaneId`, then
marking the newer session current in the driver-owned lane table.
`SnapshotRegistry::create_snapshot` updates only the current snapshot for the
new request generation; it is not the watch/LSP lane supersession authority.
The lane table must be monotonic by `BuildRequestGeneration`: an older
generation must not become current again after a newer generation is marked
current, and repeating the same generation is valid only for the same
session/snapshot tuple. After the driver lane table is updated:

- older sessions in the lane are obsolete for current publication, even if they
  have the same `BuildSnapshotId` as the newer generation;
- older in-flight tasks are cancelled through scheduler/cancellation seams;
- completed older results are discarded unless an owner-provided cache
  validation path proves they apply to the newer snapshot;
- diagnostic, explanation, or IR retention leases may keep old snapshot
  resources alive without changing currentness.

The driver must treat supersession as a freshness boundary, not as proof that
the newer build succeeded. A failed newer session may still be the current
session for the lane and must publish its own diagnostics/status rather than
falling back to older successful outputs.

## Error Handling

Snapshot creation errors are reported as driver request/session failures without
inventing phase diagnostics. The request layer must preserve the pending
request/session context when returning a snapshot creation error so callers can
report the failure against the submitted request. The driver may wrap owner
errors for event delivery, but structured identity must remain the
owner-provided error or diagnostic record. Message text is presentation and
must not be used as diagnostic identity.

Cancellation is idempotent at the driver boundary. Cancelling a session that is
already terminal does not revive it, release unrelated snapshots, or publish
stale outputs.

## Tests

Task D-003 implementation must add Rust tests that cover:

- batch, watch, and LSP request construction without protocol payload leakage;
- snapshot capture through `SnapshotRegistry::create_snapshot`;
- identical canonical snapshot inputs yielding identical `BuildSnapshotId`
  values across sessions;
- fresh `BuildRequestId` allocation per submitted generation while multiple
  watch/LSP generations share one driver `BuildLaneId`;
- watch/LSP supersession replacing the lane-current session, not only the
  lane-current snapshot;
- rejection of stale lane-current updates that try to restore an older
  generation, and rejection of same-generation replacement by a different
  session;
- obsolete publication rejection for superseded watch/LSP sessions, including
  an older session whose snapshot id equals the newer lane-current session's
  snapshot id;
- the combined lane-current-session and `SnapshotRegistry::is_current_for_request`
  publication guard;
- stale/suppressed publication decisions without current diagnostics/artifact
  publication;
- snapshot creation errors returning the pending request/session context;
- idempotent cancellation of current and already-superseded sessions.

No `.miz` tests are required because this module defines orchestration state,
not language behavior.
