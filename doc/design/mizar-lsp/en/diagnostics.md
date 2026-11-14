# Diagnostic publication
Canonical language: English; [Japanese companion](../ja/diagnostics.md).

## Responsibility

LSP converts and delivers existing diagnostics under [spec §22.1.2](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display) and [internal03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#diagnostic-aggregator). The aggregator owns normalization, identity, deduplication and ordering; LSP does not rerun producers or allocate public codes, names or aliases. Registry and consumer-adoption gates remain upstream.

## Build-index publication

Read the immutable `BuildDiagnosticIndex` associated with the selected `LspSnapshot`, preserving its records and snapshot-scoped handles. CLI, artifact projection and LSP semantic diagnostics share this index; LSP conversion does not mutate it. Obsolete diagnostics are never delivered as current.

Validate current delivery using the original [snapshot association](snapshot.md#source-association); never combine an old index with new maps or relabel old ranges as current-buffer ranges.

Full-snapshot publication and stale reads follow the [snapshot freshness boundary](snapshot.md#freshness-and-replacement). The genuine coherent manifest-backed producer landed in `27e55711e`; downstream registry/IR-retention/atomic activation is frozen by `LSP-SNAPSHOT-REGISTRY-ACTIVATION`; `ArtifactCommitted`, frontend completion or an index alone cannot establish this boundary.

## Range conversion

Convert native source ranges to LSP UTF-16 ranges only at the LSP boundary, using the exact source snapshot and existing [range mapper](range.md), including its raw OpenBuffer mapping and validation rules.

Preserve primary ranges, retained secondary anchors and their order/multiplicity, Range versus Point, supplied provenance and generated reasons under §22.1.2. Invalid, unknown or unrepresentable diagnostics/anchors fail conversion explicitly; never silently drop them, fabricate ranges or claim successful publication. Existing producer deduplication and representative selection remain unchanged.

Source-load diagnostics retain package/path identity and their non-range location: no fabricated EOF, insertion point or source span. They are not fed through a span-only converter. No new non-range wire encoding is defined here.

## Unsaved-text overlays

[Lightweight frontend diagnostics](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lightweight-open-buffer-diagnostics) use current editor text and version, are marked `current`, and claim no verified semantic status. They remain version-keyed overlay diagnostics, separate from the published build index, and never reach CLI output or artifacts, including `VerifiedArtifact`.

Overlay replacement follows internal03; current delivery and retained reads follow [snapshot invalidation](snapshot.md#freshness-and-replacement) and [retention](snapshot.md#currentness-and-retention).

## Ordering

Preserve the aggregator's [canonical record order](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#build-aggregation) (`BuildDiagnosticIndex::records`) when selecting diagnostics for a source; do not re-sort by message text, locale or asynchronous completion.

The existing explicit native frontend path groups notifications by URI in ordered-map order and preserves index record order within each notification. It does not establish a combined overlay/semantic merge order or a coherent full snapshot.

## Implementation boundary

Task8 defines publication only. Task7's full registry, task9's implementation and task10's scheduling remain open. The coherent full-snapshot producer landed in `27e55711e`; the registry/IR-retention/atomic-activation unit is frozen separately, and the existing native frontend path remains syntax-only. No new transport payload, diagnostic allocation or producer behavior is introduced.

## Test design

No tests change in this design task. Task9 retains its existing publication-fixture, stale-index-suppression and deterministic-order obligations. This document grants no implementation or §23.6/SCA-003 completion credit.
