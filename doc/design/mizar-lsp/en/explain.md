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

`Server::handle_message_with_explanations` serves `mizar/explain` over one retained publication and a host-supplied immutable `ExplanationStore`. Params contain `snapshot` in published schema form and exactly one `diagnosticId` ordinal, `exprId` or `vcId` subject string. Subject queries select existing diagnostic-attached handles; missing or ambiguous matches are unavailable. The store reads registered bounded payloads; no producer files or services are opened by LSP. Ordinary stdio without this host reports unavailable.

Responses carry status (`available`, `unavailable`, `stale_handle`), freshness, source snapshot, original URI/version/ranges, optional existing preview and bounded payload format/text/truncation/summary hash. Obsolete requested snapshots yield `stale_handle`; retained reads after editor invalidation remain `stale_artifact` with original source coordinates. The host cancellation predicate is checked before work and before returning, yielding RequestCancelled without mutating published state.

## Test design

Query fixtures exercise existing subject selection, store outcomes, exact bounded payload preservation and native UTF-16 conversion. Genuine publication fixtures check host selection and freshness; typed fixtures check original source locations. Combined host delivery of explanation content and stale locations awaits native explanation producers. Cancellation, malformed requests and framed stdio degradation are covered. §23.6 remains partial and SCA-003 unchanged.

[Native producer prerequisites](native_payload_producers.md) use the approved Q1 A/Q2 A/Q3 B/Q4 A/Q5 A package and separate [ranged host adapter](ranged_diagnostic_host.md); Task39 redispatch follows producer implementation.
