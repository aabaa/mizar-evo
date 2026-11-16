# Task LSP-DRIVER-EXECUTION-SEAM-DESIGN: nonterminal execution proposal
Canonical language: English; [Japanese pointer](../ja/LSP-DRIVER-EXECUTION-SEAM-DESIGN.md).
Status: frozen spec-only unit; proposal requires owner approval. Tier: full; new execution boundary, no pattern predecessor.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md). Consumers: Driver execution owner and parked Task40 Audit G5.
Purpose: define an owner-supported real nonterminal Driver execution seam for bridge supersede/Shutdown evidence without changing language semantics.

## Authority and readiness

Authority: spec [§23.6](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support); internal01 [Watch/LSP](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#watch-and-lsp-build), [cancellation](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#cancellation); Driver [session](../../mizar-driver/en/request.md#buildsession), [supersession](../../mizar-driver/en/request.md#supersession), [scheduler boundary](../../mizar-driver/en/driver.md#scheduler-boundary), [cancellation](../../mizar-driver/en/driver.md#cancellation), [events](../../mizar-driver/en/events.md); LSP [bridge lifecycle](../../mizar-lsp/en/build_bridge.md#supersede-and-terminal-lifecycle).
Gap: `design_drift`/execution incompleteness and `test_gap`: synchronous Driver finishes before storing the submission; bridge cannot obtain a genuine live session to cancel. Existing typed cancellation and running-predicate fixtures do not fill that gap.
Readiness: owner authorized a separate definition unit; execution/API/lifetime choices remain explicit Q-decisions pending approval. Task40 remains parked until approved prerequisite implementation lands.

## Scope and boundaries

Author paired `doc/design/mizar-lsp/{en,ja}/execution_seam.md`: established invariants, concrete conditional seam requirements, open questions/options/recommendations/tradeoffs, validation and separate implementation handoff.
Define accepted nonterminal identity/state, real scheduled work/progress, cancellation access and observation, replacement ordering, completion races, event/publication guards, native capture/lease lifetime and cleanup. Separate session currentness from execution drainage; preserve owner scheduler/phase semantics.
Propose execution ownership, control/progress API, cooperative boundaries and deterministic genuine-work test synchronization. Explicit Q-decisions cover exclusive-borrow-compatible control, the known Shutdown policy-propagation gap, cancellation versus execution drainage, and session-scoped cancellation/resource isolation despite shared snapshot ids. Preserve synchronous callers, canonical results and structured failures; exact API names remain implementation-owned after approval.
Forbidden: production/test/fixture/doc-spec/expectation/trace/ledger changes, seam implementation, synthetic acceptance, scheduler policy or language/proof/diagnostic changes, Task40 edits/work, Task41 decisions, AGENTS.md/protocol edits, OpenBuffer provenance/storage/SourceId decisions or coverage credit.
Writes: contract pair and one Task Index row per LSP plan first; paired proposal, paired bridge prerequisite link and bilingual-owner registration second. No other documentation fan-out.
Coverage: §23.6/SCA-003 status unchanged; this definition establishes no execution, cancellation or semantic acceptance evidence.

## Review matrix and exit

Document-review matrix (cap six; runtime outcomes conditional on approval/implementation): (1) genuine captured submission exposes authoritative nonterminal identity with outstanding real work and independent cancellation access; (2) accepted replacement, including equal snapshot id, makes only its lane current before one Superseded request, suppressing old publication; (3) Shutdown/exit discards admitted work, cancels once and permits cleanup/progress without new admission; (4) failed capture/submission, mere edits, already-terminal and completion races preserve existing no-cancel/failure rules; (5) old/replacement and watch/LSP sessions retain isolated resources, guards and canonical events through cooperative drainage; (6) deterministic real-work cancellation and uncancelled equivalence distinguish native evidence from fabricated states/results and identify implementation/Task40 redispatch gates.
Existing intent: Driver cancellation/event/current-session tests; LSP `build_bridge_timing_tests.rs`/`build_bridge_event_tests.rs` genuine capture/lane/failure controls and running-deadline predicate; no executable tests or runtime claims in this unit.
Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and consistency/final full-tier assessment over both commits. Extra cases require authority or a .miz test.
Each commit: both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy` (links/fragments included), EN/JA question/section parity and `git diff --check`. Caps: contract 60 lines/660 words; combined documentation 200 added lines.
Documentation only: fmt/Clippy/runtime/E2E unrun; full tests only on a named cadence trigger. Exit: freeze then paired proposal in two local commits, one changed task; owner approval pending before separate seam implementation and Task40 redispatch; exclude unrelated owner edits.
