# Task LSP-RUNNING-BRIDGE-CANCELLATION: running-session bridge cancellation evidence
Canonical language: English; [Japanese pointer](../ja/LSP-RUNNING-BRIDGE-CANCELLATION.md).
Status: frozen implementation contract. Tier: full; independent Audit G5 evidence unit, not a registered pattern-family slice.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md); consumer: Task40. Dependencies: Task11 and implemented [nonterminal execution](../../mizar-lsp/en/execution_seam.md#implemented-a-package).

## Authority and scope

Authority: [bridge lifecycle](../../mizar-lsp/en/build_bridge.md#supersede-and-terminal-lifecycle), [capture/submission](../../mizar-lsp/en/build_bridge.md#capture-and-submission), [lane isolation](../../mizar-lsp/en/build_bridge.md#watch-mode-interplay), [execution boundary](../../mizar-lsp/en/execution_seam.md#required-observable-boundary), Driver [execution](../../mizar-driver/en/driver.md#nonterminal-captured-execution) and [events](../../mizar-driver/en/events.md).
Existing tests: `build_bridge_event_tests.rs` native progress/replay, capture/structured submission failures and lane controls; `build_bridge_timing_tests.rs` admission/deadline/terminal-discard controls; Driver `tests/source_load/import.rs` genuine Frontend checkpoint cancellation.
Gap: `test_gap` G5 lacks combined bridge cancellation through actual outstanding native scheduler work; synchronous terminal controls and running predicates provide no such evidence. The prerequisite seam is now available.
Writes: LSP bridge tests, this contract pair, plan/Task40 links and checkbox, paired bridge test design and source/spec correspondence G5 row. Reuse native fixtures and existing controls; no new production seam or alternate scheduler.
Forbidden: production behavior/API changes, fabricated acceptance/session/task/results, sleeps as synchronization, scheduler policy, language semantics, spec/.miz/expectation/trace changes, Task41, owner-gated scope, AGENTS/protocol edits, semantic coverage or SCA-003 credit.

## Test matrix and exit

Cap four grouped cases; subassertions and existing controls may be reused. Extra cases require an authority clause or .miz intent:
1. Genuine captured native execution pauses at its admitted Frontend task. Editing/admission leaves it Running; only successful lane-current replacement makes it Cancelling/Superseded. Include equal-snapshot replacement, distinct session/generation/original capture associations, real checkpoint cancellation, drained Superseded outcome, one terminal event and no obsolete accepted output/publication; replacement completes uncancelled.
2. Shutdown and exit with real outstanding Frontend work and an admitted slot discard that slot and request Shutdown; repeated terminal traffic cannot capture/submit or change the winning reason. Observe Cancelling before progress, actual checkpoint cancellation, drained Cancelled outcome, one terminal event and no publication; progress/replay remain usable afterward.
3. Failed capture consumes its generation/slot without cancelling or replacing the previous genuine Running session. No automatic retry; a later valid replacement works. Preserve existing structured accepted-submission-failure and already-terminal controls without treating accepted failure as rejection or changing their intent.
4. A genuine nonterminal other-lane execution with a distinct publisher stays unchanged during LSP supersede/Shutdown, even with equal snapshot identity; its own progress completes normally and LSP replay excludes its events. Preserve existing Watch-origin/lane controls.
Reviews: specification/test intent before freeze; independent sufficiency, precision, volume/scope, consistency/final over the whole unit.
Verification: freeze both lint_policy suites and local links/fragments; implementation fmt, all-target/all-feature Clippy with warnings denied, both lint_policy suites, complete mizar-lsp group including stdio, and explicit native bridge cancellation tests as short source-to-bridge E2E.
Affected cancellation/replay/lane obligations run in those native tests; unchanged Build/Driver scheduler/resource/cache/proof/kernel matrices defer to cadence/integration. Full cargo test runs only on a named cadence trigger.
Exit: freeze then implementation locally committed as one changed task; no push; Task40 checked off with genuine versus predicate evidence explicit and required reviews/checks passing. Missing prerequisites or new semantic choices stop the unit.
