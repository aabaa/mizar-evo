# Task LSP-CODE-ACTION-VALIDATOR-CONTRACTS-DESIGN: deferred validator proposal
Canonical language: English; [Japanese pointer](../ja/LSP-CODE-ACTION-VALIDATOR-CONTRACTS-DESIGN.md).
Status: frozen spec-only unit; resulting proposal awaits owner approval. Tier: full; new validator/command trust boundary, no pattern predecessor.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), Task38 prerequisite. Consumers: Task38; Task39 remains separately blocked by Task38 and native payload producers.
Purpose: define an owner-reviewable proposal for validator hash domains, fix-to-artifact/handle provenance, and command resolution/execution; implement nothing.

## Authority and readiness

Authority: spec [§23.6.1](../../../spec/en/23.package_management_and_build_system.md#2361-hover-code-actions-and-diagnostics); architecture12 [action classes](../../architecture/en/12.diagnostics_and_lsp.md#code-actions); internal03 [fix preconditions](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#fix-suggestion) and [action flow](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#code-actions).
Existing owners: [fix model](../../mizar-diagnostics/en/fix.md#data-model), [explanation model](../../mizar-diagnostics/en/explain.md#data-model), [manifest hashes](../../mizar-artifact/en/manifest.md#hash-string-domains), [code actions](../../mizar-lsp/en/code_action.md), [snapshot association](../../mizar-lsp/en/snapshot.md#source-association), [explanation queries](../../mizar-lsp/en/explain.md).
Gap: Task38's deferred validators have undefined hash domains, no authoritative association from a particular fix to backing artifact/handle, and no command interpretation/execution contract (design/spec gaps). Existing generic hash fields and opaque command refs confer no validation authority.
Readiness: owner authorized this separate proposal unit; every genuinely new semantic choice stays an explicit question with options. Proposal completion does not authorize Task38 redispatch.

## Scope and boundaries

Author paired `doc/design/mizar-lsp/{en,ja}/code_action_validators.md`: distinguish existing requirements, proposed domain/input/validation contracts and owner decisions. Give each open question alternatives, recommendation/tradeoff and approval dependency; never imply an option is approved.
Cover raw editor versus normalized build source identity, existing store-framed artifact hashes, explanation content/summary and missing-facts identities; avoid interpreting one generic Hash as several domains. Bind publication, diagnostic, canonical fix, target sources, backing artifact and exact handle; retain fail-closed missing/ambiguous/stale behavior.
Cover offering, resolve and execution, opaque command references, dispatch ownership/capabilities, currentness rechecks, outcome mapping and explicit edit application. Keep existing action/command filtering until approved contracts and owner implementations exist.
Preserve upstream ownership, immutable original-source reads, candidate versus kernel-verified facts, existing payload/schema/API shapes and applicability. Do not decide parked OpenBuffer provenance/storage, Session SourceId encoding, producer completeness or other units' questions.
Forbidden: Rust/test/fixture/spec/expectation/trace/ledger edits, validator or consumer implementation, new production APIs or payload fields, task completion/coverage credit, Task39 or any other task work, AGENTS.md/protocol edits.
Writes: this contract pair and one Task Index link per LSP plan first; then the paired proposal, paired code-action owner links, Task38-only TODO prerequisite link and bilingual-owner registration. No other document fan-out.
Coverage: §23.6 remains partial; SCA-003 and source/test correspondence retain existing credit and deferrals.

## Review matrix and exit

Document-review matrix (cap six; no executable cases added): (1) raw/BOM/CRLF/Unicode and equal-build-id edits: separate raw validity from normalized source identity; (2) multiple source edits/hash inputs: explicit complete scope or refusal; (3) matching versus wrong manifest/module/hash/schema: exact owner validation or refusal; (4) two fixes/diagnostics with shared or missing/ambiguous explanation/missing-facts handles: exact provenance or refusal; (5) known/unknown command and forged locator/capability absence: bounded dispatch or refusal; (6) edit/reopen/replacement/cancellation between offer, resolve and execution: revalidate before effects, no stale edits. Expected positive behavior is conditional on owner approval, not current implementation credit.
Existing intent stays unchanged: `diagnostics.rs` tests `artifact_command_and_informational_suggestions_are_not_offered`, `snapshot_and_expected_text_preconditions_fail_closed`; `hover_tests.rs::code_actions_require_a_current_live_publication_and_reject_old_locators`; `stdio_code_action_without_publication_returns_an_empty_list`. No tests are rewritten to accept proposed behavior.
Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope, consistency and final full-tier assessment over both commits. A sufficiency case beyond the cap must cite an authority clause or .miz test.
Verification before each commit: both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, local links/fragments, EN/JA section parity and `git diff --check`. Caps: contract 60 lines/660 words; documentation 200 added lines, summed from git numstat across both commits. Documentation only: no fmt/Clippy/runtime/E2E; full cargo test only on a named cadence trigger.
Exit: two local documentation commits (freeze then paired proposal), one changed task; clean tree; owner approval explicitly pending and Task38 filtering preserved. Owner reviews options before any later implementation dispatch; this unit ends without selecting another task.
