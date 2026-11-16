# Task LSP-NON-RANGE-DIAGNOSTIC-PRESENTATION-DESIGN: non-range diagnostic presentation proposal
Canonical language: English; [Japanese pointer](../ja/LSP-NON-RANGE-DIAGNOSTIC-PRESENTATION-DESIGN.md).
Status: frozen spec-only unit; presentation proposal requires owner approval. Tier: full; a new diagnostic wire boundary fails the no-new-public-seam pattern criterion.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md). Consumers: presentation implementation prerequisite(s) and Task41 Audit G6.
Purpose: propose delivery of existing source-load/non-range records without fabricated locations; preserve explicit whole-batch refusal until owner-approved presentation is implemented.

## Authority and readiness

Authority: spec [§22.1.2](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display); internal03 [aggregation](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#build-aggregation); diagnostic [source-loading locations](../../mizar-diagnostics/en/failure_record.md#source-loading-primary-locations); LSP [range conversion](../../mizar-lsp/en/diagnostics.md#range-conversion), [implementation boundary](../../mizar-lsp/en/diagnostics.md#implementation-boundary), [snapshot freshness](../../mizar-lsp/en/snapshot.md#freshness-and-replacement); [Task41](../../mizar-lsp/en/todo.md#hardening-and-cross-cutting-follow-ups).
Gap: `spec_gap` in non-range wire presentation. LSP conversion rejects non-Span primaries in `crates/mizar-lsp/src/diagnostics.rs`; Task9 supplies no encoding. This unit proposes choices, never resolves the gap by implementation.
Dependencies: Task9 is implemented; owner authorized a separate definition before presentation approval. Source-load records already retain package/path identity without SourceId/text. Standard LSP Diagnostic requires a range; define alternatives without relaxing source-location rules.
Readiness: definition is ready; presentation, compatibility and delivery semantics remain owner questions. Implementation and Task41 follow approval in separate dispatches.

## Scope and boundaries

Author paired `doc/design/mizar-lsp/{en,ja}/non_range_diagnostics.md`: wire alternatives, identity/payload, capability/fallback, replacement/clearing/currentness, mixed-batch conversion and I/O boundaries, open Q1–Q5 options with recommendations, conditional validation and handoff.
LSP owns presentation/conversion/delivery; Diagnostics retains normalization, identity, deduplication and order; Driver/snapshot owners retain currentness and publication. Reuse retained records and handles; no new public diagnostic allocation or producer behavior.
Forbidden: production/test/fixture/doc-spec/expectation/trace/ledger changes, fabricated EOF/insertion point/span or misidentified source URI, span-only conversion of non-range records, implicit owner decisions, unrelated open-question resolution, Task41 completion, AGENTS/protocol edits or coverage credit. Keep current explicit whole-batch refusal unchanged.
Writes: this contract pair and one Task Index row per LSP plan in the freeze; then proposal pair, paired diagnostic prerequisite links, one Task41 sequencing link and `en/bilingual_sync_audit.md` owner registration (JA remains its pointer). No other documentation fan-out.
Coverage: no §23.6/SCA-003 or runtime-delivery credit; no coverage-audit state changes.

## Review matrix and exit

Document-review matrix (cap six; executable checks conditional on approval): (1) source-load records with absent SourceId/text and all existing source-load codes retain package/path/non-range identity; (2) same path in distinct packages and repeated messages retain canonical handles/order without URI collisions or new deduplication; (3) negotiated versus unsupported client paths have explicit delivery/refusal outcomes; (4) clean replacement, stale selection, invalidation and shutdown obey the proposed clearing/currentness boundary; (5) valid mixed ranged/non-range input follows the proposed presentation; invalid anchors/unknown variants fail preflight without partial conversion success; (6) framed delivery, mid-write currentness/I/O failure and retained payload fidelity distinguish attempted delivery from atomic client state.
Existing intent: `diagnostics.rs::conversion_fails_the_whole_batch_for_source_load_unknown_and_invalid_spans`, native wire framing and writer-currentness/I/O tests in `server.rs`. No executable cases added or expectations changed.
Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and consistency/final full-tier assessment over both commits. Additional cases require an authority clause or .miz test.
Each commit: both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, local links/fragments, EN/JA parity and `git diff --check`. Caps: contract 60 lines/660 words; whole-unit documentation 200 added lines.
Documentation only: fmt/Clippy/runtime/E2E unrun; full `cargo test` deferred absent a cadence trigger. Exit: contract freeze then paired proposal in two local commits, one changed task; owner approval remains pending before implementation and Task41; preserve and exclude unrelated owner edits. Push disabled.
