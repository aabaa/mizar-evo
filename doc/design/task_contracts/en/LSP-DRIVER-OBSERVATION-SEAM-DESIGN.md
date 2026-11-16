# Task LSP-DRIVER-OBSERVATION-SEAM-DESIGN: native checkpoint observation proposal
Canonical language: English; [Japanese pointer](../ja/LSP-DRIVER-OBSERVATION-SEAM-DESIGN.md).
Status: frozen spec-only unit; proposal requires owner approval. Tier: full; adds a public observation boundary, failing the no-new-public-seam pattern criterion.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md). Consumers: Driver observation implementation and parked Task40 Audit G5.
Purpose: define Driver-owned evidence of genuine native Frontend checkpoint cancellation before stale-result suppression, without changing native TypeId, execution path or language semantics.

## Authority and readiness

Authority: spec [§23.6](../../../spec/en/23.package_management_and_build_system.md#236-lsp-integration-and-ide-support); internal01 [cancellation](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#cancellation); Driver [execution](../../mizar-driver/en/driver.md#nonterminal-captured-execution), [scheduler boundary](../../mizar-driver/en/driver.md#scheduler-boundary), [cancellation](../../mizar-driver/en/driver.md#cancellation), [session](../../mizar-driver/en/request.md#buildsession), [events](../../mizar-driver/en/events.md); LSP [approved execution seam](../../mizar-lsp/en/execution_seam.md#implemented-a-package), [bridge lifecycle](../../mizar-lsp/en/build_bridge.md#supersede-and-terminal-lifecycle); frozen [Task40 cases 1–2](LSP-RUNNING-BRIDGE-CANCELLATION.md#test-matrix-and-exit).
Gap: `test_gap`/observation incompleteness. Task40's scheduler Cancelled and empty-output assertions cannot prove native checkpoint receipt: Driver suppresses obsolete phase results at `crates/mizar-driver/src/driver.rs:1777`. This is an evidence gap, not a demonstrated production defect.
Readiness: separate definition authorized; approved execution seam is implemented. Observation API, evidence provenance, retention and validation choices remain open for owner approval; implementation and Task40 fix redispatch follow separately.

## Scope and boundaries

Author paired `doc/design/mizar-lsp/{en,ja}/observation_seam.md`: required observation ordering/provenance, session/task identity, read-only access/lifetime, invariants, open Q1–Q5 options with recommendations, conditional validation and implementation handoff.
Distinguish request, scheduler cancellation, genuine native checkpoint receipt/result, stale-result suppression and drainage. Define sufficient positive and negative evidence without wrapping/replacing the native service, changing its TypeId/path or weakening suppression/publication guards. Preserve the approved execution/control/cancellation lifecycle and reason semantics.
Forbidden: production/test/fixture/doc-spec/expectation/trace/ledger changes, seam implementation, synthetic acceptance/results/checkpoint evidence, scheduler or language/proof/diagnostic semantics, Task40 work/edits, Task41 decisions, owner-gated scope, AGENTS/protocol edits or coverage credit. New semantic choices stay explicit owner questions, never implicit decisions.
Writes: contract pair and one Task Index row per LSP plan first; paired proposal, paired bridge prerequisite link and bilingual-owner registration second. No other documentation fan-out.
Coverage: §23.6/SCA-003 status unchanged; no runtime cancellation or semantic acceptance evidence is established by this definition.

## Review matrix and exit

Document-review matrix (cap six; executable validation conditional on approval/implementation): (1) genuine native invocation/checkpoint result observed before suppression and keyed to authoritative session/task identity; (2) accepted equal-snapshot replacement yields distinct old/replacement evidence and exact Superseded receipt without obsolete publication; (3) Shutdown/exit yields exact Shutdown receipt, drainage and one terminal event with post-terminal observation access; (4) pre-dispatch scheduler cancellation, no request, non-checkpoint failure and completion races cannot masquerade as native receipt; (5) replacement/Watch isolation, read-only retrieval, bounded retention/cleanup and unchanged canonical synchronous/nonterminal behavior; (6) deterministic native checkpoint evidence plus uncancelled controls, implementation prerequisites and separate Task40 fix handoff.
Existing intent: Driver `tests/source_load/import.rs` native checkpoint tests, Driver lifecycle/replay controls, LSP `build_bridge_event_tests.rs` native supersede/Shutdown/equal-snapshot/lane tests and `build_bridge_timing_tests.rs`; no executable changes or runtime claims here.
Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and consistency/final full-tier assessment over both commits. Extra cases require authority or a .miz test.
Each commit: both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, local links/fragments, EN/JA parity and `git diff --check`. Caps: contract 60 lines/660 words; whole-unit documentation 200 added lines.
Documentation only: fmt/Clippy/runtime/E2E unrun; full `cargo test` required on the final tree by the tenth-outstanding-task cadence trigger. Exit: freeze then proposal in two local commits, one changed task; owner approval pending before separately dispatched implementation and Task40 fix; exclude unrelated owner edits.
