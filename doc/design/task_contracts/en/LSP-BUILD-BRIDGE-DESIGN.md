# Task LSP-BUILD-BRIDGE-DESIGN: build bridge design
Canonical language: English; [Japanese pointer](../ja/LSP-BUILD-BRIDGE-DESIGN.md).
Status: specified (design only). Tier: full; unmet pattern criterion: no frozen predecessor contract for this publication surface.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task10. Consumers: task11 and the [Driver plan](../../mizar-driver/en/00.crate_plan.md).
Purpose: freeze scheduling, existing Driver event consumption, edit debounce/supersede and independent watch/LSP lanes.

## Authority and readiness

Authority: [task10](../../mizar-lsp/en/todo.md#wave-b-server-snapshots-diagnostics-build-bridge), [internal01](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#watch-and-lsp-build), existing CompilerDriver/BuildEventKind behavior, and owner decisions D-Q1/D-Q2/D-S5 in the resumed dispatch.
Dependencies: task4 [synchronization](../../mizar-lsp/en/server.md), task6 [source association](../../mizar-lsp/en/snapshot.md#source-association), Driver [requests](../../mizar-driver/en/request.md) and [events](../../mizar-driver/en/events.md).
Gap: missing bridge design is design_drift. Scheduling/lane policy is decided below; runtime bridge/watcher integration remains DRIVER-G-012 external_dependency_gap/deferred.

## Scope and ownership

Author paired mizar-lsp/{en,ja}/build_bridge.md, each at most 80 lines, using existing capture, submission, cancellation and event APIs only.
An accepted state-changing didOpen/didChange/didClose (including empty-list version changes) admits one text/version-free slot per Server LSP lane. Invalid/ignored/duplicate/stale notifications do nothing; admission calls no Driver, captures nothing and allocates no generation.
First admission sets t_first/t_last; later admissions update only t_last. Injected monotonic time evaluates after accepted events and at deadlines. Flush at quiet >= 300 ms, or wait >= 2 s with no running session; constants satisfy quiet_period <= max_wait, with no editor configuration. Save is no trigger.
Flush atomically against accepted events: clear slot, allocate a strictly increasing lane generation, capture all then-open documents/versions as one cohort, submit once on success. With zero open documents discard without capture/submission. Generations from failed captures are not reused.
Coalesce only admitted events across documents/packages within that lane; never submitted requests or other lanes/origins. Capture uses flush-time contents; intermediate versions are not built.
Editing never cancels. After successful replacement capture and Driver acceptance as lane-current, request cancel(previous, Superseded) exactly once if nonterminal. Capture failure consumes the slot without retry, preserves lane-current/diagnostics and creates no pending state; failure notification is unspecified. Submission failure never reverts the new lane generation.
Shutdown/exit discard the slot and request cancel(running, Shutdown) once. Admitted state has no freshness label; it is not pending before submission.
Own only this LSP lane's submissions/cancellation/events. Watch has a distinct lane/origin: no cross-lane comparison, supersede, cancel, coalescing or trigger; Disk changes do not trigger LSP and buffer edits do not trigger watch.
Independence requires lane-separated Driver tables, distinct PhaseOutputPublisher instances and same-lane immediately preceding replacement snapshots. Shared Disk artifacts/cache obey their owners alone; bridge forwards only its lane's events. Disk stale_artifact reads stay snapshot/metadata-owned. Disk changes await the next accepted event for LSP results, deferred under DRIVER-G-012; no target exception.
Registry/atomic activation (task7), diagnostics publication (task9) and implementation (task11) remain separate. Session owns capture/maps; Driver owns session/event guards; IR/Artifact/cache retain their boundaries.
Forbidden: new Driver APIs/events/payloads or diagnostic codes/names/aliases, producer changes, parked OpenBuffer provenance/storage/SourceId decisions, code/tests, protected spec/trace changes or implementation credit. Coverage/SCA-003 is unchanged.
Writes: this contract pair and one task10 row per owner plan first; paired specifications and task10-only TODO completion pointers second; new-pair audits only if established practice requires them.

## Verification and exit

Reviews: specification/test-intent before freeze; sufficiency, precision, volume/scope, source/document consistency and full-tier assessment over both commits.
Fixed matrix, zero new executable cases: each documentation commit passes `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy` and local link/fragment checks; final pair has matching headings/anchors and passes cargo test --locked -p mizar-lsp.
No fmt/Clippy/full workspace test: documentation-only, no cadence trigger, no push. Exactly two local commits count as one changed task; final status is clean and combined paths match the dispatch.
Exit: freeze first, paired specification/task10 completion second; task11/module state unchanged. Stop for undetermined required behavior, caps, or required fixes outside allowed files.
