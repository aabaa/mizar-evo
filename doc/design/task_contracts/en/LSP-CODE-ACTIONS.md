# Task LSP-CODE-ACTIONS: diagnostic fix conversion
Canonical language: English; [Japanese pointer](../ja/LSP-CODE-ACTIONS.md).
Status: specified. Tier: full; first code-action consumer.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task17; consumer: editor clients.
Purpose: convert existing diagnostic fixes without automatic application.

## Authority and readiness

Follow the frozen [Task16 design](LSP-CODE-ACTION-DESIGN.md), [conversion](../../mizar-lsp/en/code_action.md#request-sources-and-conversion), [applicability](../../mizar-lsp/en/code_action.md#applicability-and-freshness), [explicit application](../../mizar-lsp/en/code_action.md#explicit-application), [range mapping](../../mizar-lsp/en/range.md), and diagnostics-owned [fix rules](../../mizar-diagnostics/en/fix.md).
Task9 publication and diagnostics task13 are available. Missing LSP conversion is implementation_gap. Artifact/fix hash association and opaque command execution have no current validation owner in LSP; fail closed rather than invent their meaning or execution protocol.

## Scope and ownership

Serve standard code-action requests through the existing metadata/build host route, using one current publication and its original native sources. Ordinary stdio without that host returns an empty action list. Derive document version from the live open-document association and match the publication's original source. Validate URI, in-bounds UTF-16 requested range and quickfix kind filtering; preserve diagnostic/fix order and identity. Select diagnostics by half-open primary-range overlap; a zero-width range is its point (inside a nonempty interval, or coincident with another point).
Offer local and snapshot-bound edits only with explicit matching expected text, valid source ranges, matching supplied snapshot and current document versions. All three edit-bearing applicability levels remain explicit editor choices; informational suggestions are not executable actions. Unknown safety/applicability and suggestions whose hash, artifact or command preconditions cannot be validated are filtered, not reinterpreted.
Advertise codeActionProvider.resolveProvider. Offer initial title/kind/data-only quickfix actions only when textDocument.codeAction.dataSupport is true and resolveSupport.properties contains edit; otherwise return no actions. Resolve treats data as a locator, reselecting and revalidating the current original snapshot, diagnostic and fix through the live source association (including close/reopen), then returns edit.documentChanges with versioned TextDocumentEdits. Replacement or source changes invalidate resolution. Never write source text or issue applyEdit/executeCommand. Client diagnostic contents grant no authority; use the published index. No custom command encoding or proof-fact claim.
Writes: LSP source/tests, this contract pair, paired code-action owner/TODO/plan pointers, and necessary audit deferral pointer. No diagnostic payload, producer, artifact, snapshot, protected specification/test/trace, Task15 remainder, Task19+, protocol or AGENTS changes. §23.6 stays partial and SCA-003 unchanged.

## Test matrix and exit

Cap: six grouped cases: (1) local insertion/replacement and all edit applicability levels, successful deferred resolution to versioned edits, title/identity/order and no buffer mutation; (2) snapshot-bound success/mismatch, missing/mismatching expected text and supplied hash refusal; (3) artifact-assisted, command-only and informational fixtures filtered without execution; (4) multiple source edits, supplementary/combining text, BOM/CRLF/lone-CR/EOF and explicit client edit round trips, refusing invalid bounds/source; (5) requested overlap/point boundaries, kind/context/parameter/lifecycle and unsupported-client handling; (6) genuine publication host selection, stale/change/close/reopen/replacement resolution refusal and no-publication framed stdio coverage.
Independent specification/test-intent review before freeze; sufficiency, precision, volume/scope, consistency and full-tier assessment after implementation. Reuse checked-tree command logs.
Freeze: both lint_policy suites, including link/fragment/pair checks. Implementation: fmt, workspace all-target/all-feature Clippy with warnings denied, both lint_policy suites, mizar-lsp and mizar-diagnostics owner tests, short stdio E2E, full cargo test on the tenth-outstanding-task cadence trigger.
Exit: freeze then implementation commits, one changed task; Task17 conversion complete with unvalidated classes explicitly filtered. Local only, no push.
