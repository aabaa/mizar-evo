# Explanation queries
Canonical language: English; [Japanese companion](../ja/explain.md).

## Responsibility

LSP resolves existing explanation handles on demand under internal03 [Explanation Queries](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#explanation-queries). Explain commands read [explanation artifacts and bounded query services](../../architecture/en/12.diagnostics_and_lsp.md#request-sources); they do not extract bulk compiler state.

Use the existing [ExplanationRef](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#explanation-reference), a handle rather than an embedded trace. [Diagnostic publication](diagnostics.md#build-index-publication) preserves those handles; [code-action commands](code_action.md#request-sources-and-conversion) capture snapshot and diagnostic handles and revalidate when executing. No explanation kind, handle field or payload schema is added here.

## On-demand resolution

Follow internal03's four-step flow:

1. The request supplies a `DiagnosticId`, `ExprId` or `VcId` plus a snapshot id.
2. `ExplanationStore` resolves the handle against artifact hashes or cache keys.
3. The query service enforces size and depth bounds.
4. The response includes freshness, source ranges and bounded structured explanation data.

The [default diagnostic record](../../architecture/en/12.diagnostics_and_lsp.md#explanation-data-is-lazy-and-bounded) retains explanation kind, relevant ids, an artifact path when emitted and bounded preview text. Detailed data stays lazy in the [Explanation Store](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#explanation-store) backing artifacts, cache-backed query data or dedicated explanation files. Cache records are optional backing/acceleration data; diagnostics remain interpretable without them, and cache-backed explanations never determine proof status.

## Latency and size limits

The authorities fix no numeric latency, size or depth budget, so this specification adds none and defines no new timeout or truncation policy.

## Freshness and stale handles

A handle is valid only within the snapshot or artifact hash for which it was created. The [incremental update flow](../../architecture/en/12.diagnostics_and_lsp.md#incremental-update-flow) invalidates stale explanation handles from the previous snapshot. An obsolete-snapshot handle receives a stale-handle response; the bridge may suggest re-running on the current diagnostic when a matching code/span exists.

Keep response freshness and source ranges associated with their source snapshot under the existing [snapshot association](snapshot.md#source-association) and [range conversion](range.md) rules. Never relabel old ranges as current-buffer ranges.

## Missing payload and cancellation

Under [LSP Snapshot Errors](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot-errors), a missing explanation payload shows the compact diagnostic preview and reports the explanation unavailable. Do not fabricate explanation content.

[Explanation queries are cancellable](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#cancellation). Published snapshots remain immutable and are not cancelled after publication.

## Implementation boundary

Task18 specifies query behavior only. Task9 publication, task17 code actions, task19 explanation-query implementation and mizar-diagnostics task15's ExplanationStore implementation remain separate. Change no producer, diagnostics, artifact or snapshot behavior.

## Test design

This design unit changes no code or tests. Task19 retains its query-fixture, bounded-payload and missing-backing-data degradation obligations, with dependencies on tasks17/18 and mizar-diagnostics task15. Task19 and the explain module remain open; §23.6/SCA-003 credit does not change.
