# Task LSP-DRIVER-EXECUTION-SEAM: nonterminal execution
Canonical language: English; [Japanese pointer](../ja/LSP-DRIVER-EXECUTION-SEAM.md).
Status: frozen implementation contract. Tier: full; new execution API, no pattern predecessor.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md); lower owners: [Driver plan](../../mizar-driver/en/00.crate_plan.md), [Build plan](../../mizar-build/en/00.crate_plan.md). Consumer: parked Task40.

## Authority and readiness

Implement [execution seam](../../mizar-lsp/en/execution_seam.md), approved Q1–Q5 A by owner on 2026-11-16. The prior definition contract remains historical.
Authority: spec [§23.6](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support); internal01 [cancellation](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#cancellation); Build [scheduler](../../mizar-build/en/scheduler.md), [cancellation](../../mizar-build/en/cancel.md); Driver [scheduler](../../mizar-driver/en/driver.md#scheduler-boundary), [cancellation](../../mizar-driver/en/driver.md#cancellation), [session](../../mizar-driver/en/request.md#buildsession), [events](../../mizar-driver/en/events.md); LSP [bridge](../../mizar-lsp/en/build_bridge.md).
Gap: synchronous submission stores only completed sessions; real nonterminal control and Shutdown propagation are absent (`design_drift`, `test_gap`).

## Frozen implementation boundary

Build `SchedulerExecution::start(input)` validates once; `advance(dispatcher)` retains queues/resources/order and dispatches at most its previous admitted task; `pending_task()` exposes that actual BuildTask. `is_finished()` identifies drainage; consuming `finish()` returns a run only then. Synchronous runners drain this core; no reruns or invented results.
Driver `start_captured`, `progress`, `pending_task`, `terminal_submission` expose stored identity/state/events and repeat-safe terminal retrieval. Acceptance linearizes at lane marking; control becomes available at stored execution. Shared preparation/cache finalization preserves synchronous/cache/watch callers. Planning/import discovery retain structured synchronous failures; native discovery/index behavior survives. Nonterminal start rejects borrowed cache options.
A first-request-wins per-execution signal composes Build policy without snapshot-global mutation. Snapshot its exact reason into existing CancellationToken at service-entry checkpoints; prevent further eligible dispatch, preserve atomic commits. No long in-call latency guarantee. Cancellation linearizes at signal request and enters Cancelling; terminal completion linearizes only after drainage, emits one SessionFinished and preserves any earlier terminal winner. Unknown/terminal/repeated cancellation is idempotent.
Each execution retains original capture, phase inputs/results and leases until safe cleanup. Recheck lane/session and publisher/snapshot guards at delayed result/index/artifact/coherent publication; equal snapshot ids never confer replacement currentness or cross-execution cancellation rights. Terminal retrieval/replay cannot revive work or authorize stale outputs.
Configured LSP hosts opt into nonterminal captured submission and explicitly drive progress between protocol traffic, including cleanup after Shutdown/exit; existing synchronous bridge callers remain compatible. Preserve acceptance-before-supersede, generation/failure rules and original replay associations. Do not install an executable compiler host or change Watch integration.
Writes: Build continuation/cancellation, Driver execution/control, bounded LSP bridge adaptation and prerequisite tests; paired execution/affected owner documents and plan links. No AGENTS/protocol, language semantics, spec, fixtures/expectations, scheduler policy, Task40 evidence/tests, Task41 decision, presentation files or coverage-credit changes.

## Test matrix and exit

Cap six prerequisite cases (subassertions allowed); existing tests supply unchanged failure/lane controls:
1. Genuine captured native SourceLoad/Frontend execution yields at its actual admitted task; authoritative nonterminal identity and original capture remain accessible; uncancelled completion matches synchronous canonical status/results/events.
2. Cancel that genuine admitted task with each Driver reason; real phase checkpoint observes cancellation, Cancelling precedes drainage, one terminal event follows; no accepted output or coherent publication escapes cancellation.
3. Equal-snapshot old/replacement and distinct-lane sessions isolate signals/resources and preserve guards; old cancellation cannot affect replacement and retained readers survive cleanup.
4. Unknown/repeated/terminal cancellation and completion-before-cancel preserve winning terminal result/events; early errors stay structured and never fabricate live work; resource cleanup is once-only.
5. Explicit configured-host bridge route returns real nonterminal acceptance, progresses to canonical native terminal replay with original cohort, and permits protocol traffic/control between steps; existing synchronous controls stay passing. Combined bridge supersede/Shutdown cancellation evidence belongs exclusively to Task40.
6. Existing readiness/resource, cache/replay/publication, native diagnostic-index and synchronous/watch regression groups remain passing; no timeout races or synthetic acceptance count as native evidence.
Reviews: specification/test intent before freeze; sufficiency, precision, volume/scope, consistency/final over the whole unit. Extra cases require authority or a .miz test.
Verification: both lint_policy suites and links/fragments for freeze; implementation fmt, all-target/all-feature Clippy with warnings denied, both lint_policy suites, Build/Driver/LSP owner groups and short native E2E. Affected cache/replay/resource groups run immediately; unrelated proof/kernel heavy groups defer to cadence. Full cargo test only on a named cadence trigger.
Exit: freeze then implementation locally; one changed task, push disabled. Task40 stays parked for separate redispatch; §23.6/SCA-003 unchanged.
