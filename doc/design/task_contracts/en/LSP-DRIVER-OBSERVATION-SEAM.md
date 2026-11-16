# Task LSP-DRIVER-OBSERVATION-SEAM: native checkpoint observations
Canonical language: English; [Japanese pointer](../ja/LSP-DRIVER-OBSERVATION-SEAM.md).
Status: frozen implementation contract. Tier: full; new public observation seam fails the no-new-public-seam pattern criterion.
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md). Consumer: parked Task40; prerequisite: implemented nonterminal execution.

## Authority and readiness

Implement the [observation proposal](../../mizar-lsp/en/observation_seam.md), Q1–Q5 A approved by the owner on 2026-11-16; B alternatives are excluded.
Authority: spec [§23.6](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support), internal01 [cancellation](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#cancellation), Driver [execution](../../mizar-driver/en/driver.md#nonterminal-captured-execution), [scheduler](../../mizar-driver/en/driver.md#scheduler-boundary), [cancellation](../../mizar-driver/en/driver.md#cancellation), [session](../../mizar-driver/en/request.md#buildsession), [events](../../mizar-driver/en/events.md).
Gap: `test_gap` in checkpoint provenance after obsolete phase-result suppression, not an established cancellation defect. No language intent or §23.6/SCA-003 coverage change.

## Frozen boundary

`CompilerDriver::start_captured_observed` opts in at captured start, using the existing start arguments and route; ordinary start/synchronous/Watch submissions remain disabled. `native_checkpoint_observations(session)` is read-only and returns `NativeCheckpointObservations::{UnknownSession, Disabled, Enabled(&[NativeCheckpointObservation])}`; an empty enabled slice means no receipt.
Each immutable summary contains owning `BuildSessionId`, actual graph `TaskId`, `NativeCheckpoint::FrontendServiceEntry`, input `BuildSnapshotId` and exact delivered Build `CancellationToken`. Its presence attests to the native matching-token Cancelled return with empty diagnostics/outputs and no artifact; no compiler payload is retained. Resolve request/lane/request-generation from the original session, never from cancellation generation.
At the existing registry dispatch return, establish native concrete Frontend TypeId, matching input/token snapshot and original Cancelled empty-result shape. Record only after existing snapshot/output/diagnostic validation succeeds, before normalization or stale suppression. The current native matching-token branch is the sole Cancelled origin; any new origin must preserve or strengthen attribution. Non-native lookalikes and unsupported/invalid paths cannot create evidence.
Store summaries with the existing Driver session execution/replay record through drainage and stale replay; release with that record, with no independent resources, consuming drain, retroactive enable or new eviction policy. Dispatch/return order determines summary order. Reads/replay never dispatch, consume cancellation or duplicate terminal events.
Preserve native service TypeId/resources/route, validation, first-reason cancellation, scheduling, canonical results/events, publisher/currentness guards and synchronous compatibility. Receipt is neither drainage nor publication authority. Equal-snapshot replacement and other lanes remain distinct.
Writes: Driver API/session record/registry dispatcher, owner tests, paired Driver boundary documentation, proposal approval status and plan links. Forbidden: AGENTS/protocol, language/spec/expectations/fixtures, scheduler policy, native wrappers/callbacks, Task40 fix/combined bridge acceptance, Task41, presentation or semantic-credit changes.

## Test matrix and exit

Cap six cases, subassertions allowed; existing regressions may supply controls:
1. Genuine captured OpenBuffer native work reaches pending Frontend while Running; enabled empty before invocation, then exact session/task/checkpoint/snapshot/token receipt survives obsolete cleared phase results; disabled and unknown stay distinct.
2. Accepted equal-snapshot replacement preserves distinct original identities; old receipt carries accepted Driver Superseded's SupersededSnapshot reason and no obsolete output/publication, while replacement runs uncancelled with separate evidence.
3. Shutdown receipt retains Shutdown, Cancelling before progress, drainage and one terminal event; evidence survives terminal cleanup and stale replay. Repeat cancel/read/progress preserves first reason and cardinality.
4. Scheduler cancellation without native Frontend dispatch, no-request success, unrelated failure, invalid result/token and completion-before-cancel never create a receipt; non-native descriptor/status lookalikes are rejected.
5. Equal-snapshot other-lane/Watch controls/publishers/evidence stay isolated; reads have no execution/publication side effects; session-record lifetime controls retained evidence, including disabled/unknown access.
6. Enabled/disabled uncancelled native execution preserves canonical status/results/events/publication and synchronous compatibility; deterministic tests detect absent or post-suppression capture. No sleeps, timeout races, private-state mutation, fabricated acceptance or native wrappers as positive evidence.
Reviews: independent specification/test intent before freeze; sufficiency, precision, volume/scope and consistency/final across both commits. Added cases require authority or a .miz test.
Verification: freeze runs both lint_policy suites and local links/fragments; implementation runs fmt, all-target/all-feature Clippy with warnings denied, both lint_policy suites, Build/Driver/LSP/Session owner groups and short native E2E. Snapshot/cache/replay regressions run with owners; unrelated proof heavy matrices defer to the full integration gate. Full cargo test is required for the dispatched push and inherited ten-task cadence.
Exit: freeze then implementation commits count as one changed task; push only after full success. Task40 resumes solely by separate orchestrator dispatch.
