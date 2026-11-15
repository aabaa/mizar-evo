# Task LSP-DEFERRED-CODE-ACTIONS: validated deferred consumers
Canonical language: English; [Japanese pointer](../ja/LSP-DEFERRED-CODE-ACTIONS.md).
Status: specified. Tier: full; Task38, Audit G4 follow-up.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), [Task38](../../mizar-lsp/en/todo.md); consumers: editor clients.
Purpose: validate deferred suggestions and revalidate diagnostic commands at execution.

## Authority and readiness

Use [request conversion](../../mizar-lsp/en/code_action.md#request-sources-and-conversion), [freshness](../../mizar-lsp/en/code_action.md#applicability-and-freshness), and the [validator contracts](../../mizar-lsp/en/code_action_validators.md), with Q1–Q5 option A approved by the owner. [Task17](LSP-CODE-ACTIONS.md) supplies local conversion. Missing validators/bindings are implementation gaps; genuine native producers without authenticated payloads remain filtered. Task25 is not a dependency; Task39 owns positive native delivery evidence.

Direct authority: [spec §23.6.1](../../../spec/en/23.package_management_and_build_system.md#2361-hover-code-actions-and-diagnostics), [architecture action classes](../../architecture/en/12.diagnostics_and_lsp.md#code-actions), [internal action flow](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#code-actions). Existing test anchors: [typed conversion](../../../../crates/mizar-lsp/src/diagnostics.rs) `snapshot_and_expected_text_preconditions_fail_closed`, `artifact_command_and_informational_suggestions_are_not_offered`; [native lifecycle](../../../../crates/mizar-lsp/src/hover_tests.rs) `code_actions_require_a_current_live_publication_and_reject_old_locators`; [stdio](../../../../crates/mizar-lsp/tests/stdio.rs).

## Scope and ownership

Reuse Session loaded-text hashing for explicitly producer-registered sole-source preconditions; refuse hashed multi-source fixes. Validate every edited source and every registered dependency against the selected publication's original sources, module/manifest entries and store-framed artifacts. Keep expected-text, range, raw document incarnation and currentness checks.
Add host-owned immutable producer registrations keyed by publication, diagnostic and canonical fix, with exact explanation handles or authenticated obligation/entry selectors. Validate supported producer-owned backing/summary encodings using the approved framed hash domain; unsupported or incomplete provenance refuses. Never infer a relation from titles, paths, adjacency or a unique handle.
Resolve commands to `mizar.executeDiagnosticCommand` with one opaque locator argument, only with an execution host and command-resolve client support. Reconstruct from authority at resolve and execution; revalidate all handles, backing, fixes and original sources. Bounded reads retain original inputs and recheck currentness before returning. Effectful operations require atomic owner admission against replacement/edit/close/reopen; absent owners remain filtered. Apply the approved InvalidParams/MethodNotFound/ContentModified/RequestCancelled/InternalError mapping. No implicit source edit.
Writes: LSP implementation/tests and dependency entries, paired code-action/validator owners and Task38 bookkeeping, this contract pair. Producer registration seams are in scope; diagnostic payload schemas, language/proof behavior, protected specifications/tests/trace, source identity encoding, OpenBuffer artifact storage, Tasks39–42 and Task26 remainder are excluded. No coverage credit change.

## Test matrix and exit

Cap: six grouped cases, using typed consumer fixtures where genuine payload producers are absent:
1. Loaded-text hash agreement across BOM/CRLF/Unicode; equal-id/version-only and raw changes still invalidate actions, with original UTF-16 coordinates preserved.
2. Sole-source explicit hash success/mismatch/missing declaration; hashed multi-source refusal and unchanged unhashed multi-source/expected-text validation.
3. Exact publication source/module/manifest/store artifact success; wrong source/hash/class/schema/unreachable dependency refusal, including multiple dependencies.
4. Exact fix/diagnostic/handle binding and all backing content/summary or authenticated obligation/entry selectors; missing, ambiguous, unsupported or mismatched registration refuses.
5. Fixed command offer/resolve/execute with server reconstruction; capability/host/unknown-ref filtering, forged/stale locators, malformed arguments, unsupported route and host failure mapping.
6. Replacement/edit/close/reopen/cross-Server reuse and cancellation before admission prevent stale work; retained reads recheck currentness, and unavailable effectful owners never run.
Independent specification/test-intent review before freeze; sufficiency, precision, volume/scope, consistency and full-tier assessment after implementation.
Freeze: both lint_policy suites and affected links/fragments. Implementation: fmt, workspace all-target/all-feature Clippy with warnings denied, both lint_policy suites, mizar-lsp and mizar-diagnostics owner tests, affected artifact/hash checks and short stdio E2E. Full cargo test only on a named cadence trigger.
Exit: freeze then implementation commits count once; authenticated consumers work with current owner registrations, unsupported producers remain filtered. Local only; no push.
