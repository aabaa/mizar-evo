# Build bridge
Canonical language: English; [Japanese companion](../ja/build_bridge.md).

## Responsibility

Task10 specifies scheduling and event consumption for one Server connection's single `BuildLaneId`, origin `BuildRequestOrigin::Lsp`. The owner-decided policy below refines [internal01's Watch and LSP Build](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#watch-and-lsp-build). Task11 implements it; [snapshot](snapshot.md) and [diagnostics](diagnostics.md) retain source/freshness and publication ownership.

## Admission and deadlines

An accepted event is a state-changing didOpen, didChange or didClose under [Server synchronization](server.md), including a didChange with an empty change list that advances version. Ignored, invalid, duplicate and stale notifications are not accepted events. didSave is not a trigger.
The LSP lane has at most one admitted, not-yet-flushed slot, with no text, version, generation or freshness label: it is not `pending`. Admission calls no Driver entry point and captures nothing. First admission sets t_first and t_last to now; further admission updates only t_last, across all documents and packages in that lane.
Use an injected monotonic clock; evaluate triggers immediately after each accepted event and at the next deadline. Specification constants are quiet_period = 300 ms and max_wait = 2 s, with quiet_period <= max_wait; no editor-configuration surface is defined.
Flush at the first instant now - t_last >= quiet_period, or now - t_first >= max_wait AND no session is running in the lane. A submitted request has a generation and Driver session; running means that session is nonterminal. Quiet expiry may flush while a session runs; cap expiry alone may not.
Only admitted requests coalesce. Submitted requests never coalesce with each other or with the slot; there is no coalescing across lanes or origins. Intermediate editor versions are not built.

## Capture and submission

Without interleaving another accepted event, flush clears the slot, allocates one generation strictly greater than every generation previously allocated in the lane, captures all then-open documents at their then-current versions as one cohort, then submits exactly once on capture success. If no document is open, discard the slot without capture or submission; no build runs with zero open documents.
Use [Server's native capture](snapshot.md#retained-native-sources) and existing captured-session Driver submission (`submit_captured` or its existing import-discovery/native-index entry points), not `submit_watch_change`. The bridge supplies the complete current open-document coordinate set; Driver does not enumerate Server buffers. Preserve the original capture/ticket handoff and lease ownership, with no reconstructed source maps or caller-fabricated capture.
Capture is at flush time, never admission time. A failed capture consumes the slot without automatic retry, does not reuse its allocated generation, leaves the previous lane-current session unchanged and uncancelled, clears no diagnostics and fabricates no pending state. User notification of this failure is unspecified.
If submission fails after capture succeeds, the new generation remains the lane's generation; do not revert to the old one. Driver's existing structured submission errors and cleanup ownership remain unchanged.

## Public enum policy

`BuildBridgeError` deliberately remains exhaustive: Capture/Submit distinguish the bridge's two failure stages, with Driver errors preserved inside Submit. A new stage must force callers to revisit failure handling at compile time.

## Supersede and terminal lifecycle

Continued editing only updates t_last; it makes no Driver request and does not cancel running generation g. After replacement capture succeeds and Driver accepts the replacement as lane-current, request `cancel(session_g, DriverCancelReason::Superseded, snapshots)` exactly once for the session belonging to g if nonterminal; do nothing if already terminal. Failed capture never cancels g.
Shutdown request or exit discards the slot without flushing and requests `cancel(session, DriverCancelReason::Shutdown, snapshots)` once for a running session. No admission, capture or submission follows terminal lifecycle traffic. Cancellation remains cooperative and Driver-owned under its existing [request guards](../../mizar-driver/en/request.md).

## Build event consumption

Read the existing `CompilerDriver::events(session)` history for owned submitted sessions. Use its known-session state and actual event identity (session, lane, generation, optional snapshot, publication) to associate events with that submission and original captured cohort; never infer editor version/incarnation from BuildSnapshotId equality. This is replay retrieval, not a new subscription API.
Consume only this LSP lane's events. Preserve Driver ordering and publication guards; replayed readiness grants no renewed current authority. Progress, gaps, suppression and SessionFinished retain the existing [BuildEventKind meanings](../../mizar-driver/en/events.md); downstream non-exhaustive matches require a wildcard. Introduce no event kind, payload field, diagnostic code/name/alias or producer behavior.
Only the genuine coherent `SnapshotPublished { snapshot }` payload is handed toward task7's registry/atomic activation and task9's diagnostic publication boundary. ArtifactBoundary/ArtifactCommitted, Frontend completion, DiagnosticsReady or an index alone are not a coherent full snapshot. Event consumption does not activate a registry or emit diagnostics here; current delivery and stale reads remain governed by the original source association and their owners.

## Watch-mode interplay

The bridge submits/cancels/consumes only its own LSP lane and creates no Watch-origin request; it neither submits, cancels nor subscribes to the watch lane. Watch and LSP have distinct BuildLaneId values, each carrying one origin. There is no arbitration or cross-lane generation comparison, supersede, cancel or coalescing.
Each lane's admission, flush, capture failure and cancellation leave the other lane's admitted, submitted and running requests unchanged. Disk file changes never trigger the LSP lane; editor buffer edits never trigger the watch lane.
Independence requires Driver's lane table to remain separated by lane. IR `replace_current_snapshot` receives only that lane's immediately preceding snapshot as `old`; LSP and watch do not share a `PhaseOutputPublisher` instance. Publisher currentness remains separate from [editor currentness](snapshot.md#currentness-and-retention).
Shared Disk artifacts and cache obey only their owners' manifest transactions and cache-validation rules; the bridge adds none. Forward only owned LSP events toward registry/publication. Reading Disk-committed artifacts follows snapshot/metadata `stale_artifact` rules outside this bridge. These boundaries cover every bridge operation and target, without exception.
Consequently Disk-side changes are not reflected in LSP results until the next accepted event; this limitation is deferred under [DRIVER-G-012](../../mizar-driver/en/00.crate_plan.md#known-gaps-and-drift).

## Implementation boundary

The [nonterminal execution seam proposal](execution_seam.md) defines Task40's missing execution prerequisite; its Q-decisions require owner approval before separate implementation.
`src/build_bridge.rs` implements task11 through `Server::with_build_bridge`, `drive_build_bridge`, `next_build_deadline` and `build_bridge_events`. The host supplies a lane-exclusive Driver and its matching publisher, injects a monotonic clock, drives incoming traffic and returned deadlines, and resolves the complete open-URI coordinate set only at flush. Configure the bridge before accepting document traffic. The existing stdio lifecycle does not install a compiler host; native host/watch integration remains under DRIVER-G-012.
Event replay retains the original capture and does not activate a registry or deliver diagnostics; task7 and task9 keep those boundaries. OpenBuffer provenance, artifact storage and Session SourceId encoding belong to the parked Driver unit: this scheduling policy neither depends on nor decides them, and grants no OpenBuffer artifact-production capability. SCA-003 credit stays unchanged.

## Test design

Task11 fixtures use injected time for accepted-event admission, edit bursts, quiet/cap deadlines and terminal discard; genuine native captures and Driver submissions check failure consumption, original event association and lane separation. Driver's current synchronous submission returns terminal sessions; the running-session deadline predicate is tested separately, while cooperative cancellation behavior remains Driver-owned. Existing lifecycle, capture and publication test intent stays with its owners.
