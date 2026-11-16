# Task LSP-NON-RANGE-DIAGNOSTIC-DELIVERY: Task41 delivery evidence
Canonical language: English; [Japanese pointer](../ja/LSP-NON-RANGE-DIAGNOSTIC-DELIVERY.md).
Status: frozen implementation contract. Tier: full; independent Audit G6 evidence unit, not a registered pattern-family slice.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md); consumer: Task41 in the [LSP TODO](../../mizar-lsp/en/todo.md).
Dependencies: implemented [genuine capture/host](LSP-SOURCE-LOAD-DIAGNOSTIC-HOST.md) and [approved presentation](LSP-NON-RANGE-DIAGNOSTIC-PRESENTATION.md).

## Authority and scope

Authority: spec [source locations](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display), [conversion](../../mizar-lsp/en/diagnostics.md#range-conversion), [host admission](../../mizar-lsp/en/ranged_diagnostic_host.md#admission-and-ownership), [currentness](../../mizar-lsp/en/snapshot.md#currentness-and-retention), and owner-approved Q1–Q5 A [presentation](../../mizar-lsp/en/non_range_diagnostics.md#open-owner-decisions).
Existing tests: `server.rs::genuine_source_load_diagnostic_host_tests`, non-range writer/currentness controls, and `diagnostics.rs` typed conversion fixtures. Typed/caller-built indexes alone are not genuine host evidence.
Gap: `test_gap`: the genuine same-path two-package test stops at Driver submission, without originating host admission or framed delivery; the shared delivery oracle does not bind wire snapshots to original handles or check URI sequence exactly.
Scope: strengthen existing LSP tests to connect real native failures, selected host, original retained records and production framed writer; reuse existing approved validation controls. Close Task41 only when the evidence establishes that boundary.
Forbidden: B alternatives, production/public seam changes, fabricated primary text/coordinates, arbitrary-index positive host evidence, new scheduling/build integration, language/spec/expectation changes, CLI/artifact/overlay changes, and §23.6/SCA-003 coverage credit. Any genuinely new semantic choice returns to the owner.
Affected: existing `crates/mizar-lsp/src/server.rs` tests and paired plan/TODO indexing. Driver/Diagnostics retain producer ownership; no lower-stage implementation changes.

## Test matrix and exit

Cap six grouped cases, reusing existing tests unless the stated gap requires strengthening; extra cases require an authority clause or `.miz` test:
1. Genuine E0600–E0603 failures without primary text, including missing/unreadable/invalid UTF-8/outside-root: selected host and framed v1 records preserve exact package/path, code, severity, message, original handle snapshot/id and canonical order; no invented range/version.
2. Same normalized path in two genuinely captured packages: originating host admission and framed ordered distinct records; original index retained after failed files are recreated. Secondary-anchor fidelity/refusal stays in existing typed conversion tests, not attributed to the anchor-free native producer.
3. Supported/missing/malformed/unknown capabilities: negotiated delivery or zero-frame refusal; unchanged ranged-only compatibility.
4. Clean/package replacement, editor invalidation, equal-snapshot replacement and terminal closure: whole-set replacement/guarded clearing, distinct selection tokens and retained stale readers; no obsolete records revived.
5. Mixed/non-range-only, invalid/unknown/stale records and anchors: existing whole-batch preflight/refusal and source-map controls; no typed fixtures counted as genuine producer evidence.
6. Production framing, mid-batch currentness/replacement and failing writer: URI-ordered ranged frames before extension, explicit errors and stopped writes; unchanged retained/CLI/artifact/overlay controls. Strengthen equal-snapshot lifecycle with framed old/new/clean delivery rather than notification values alone.
Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and consistency/final over the complete unit, with scoped full-tier gates and score.
Verification: fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites, complete mizar-lsp owners including native stdio, explicit genuine source-load host tests as the short source-to-wire check, and documentation links/fragments.
No runtime, producer, identity/schema or test-selection policy changes: unaffected Driver/cache/evidence/proof/kernel heavy matrices and full cargo test defer to cadence/integration triggers; no push.
Exit: freeze then implementation locally committed as one changed task; required reviews/checks pass and Task41 is checked off with genuine/typed evidence boundaries preserved.
