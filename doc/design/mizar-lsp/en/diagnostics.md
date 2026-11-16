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

The separate [non-range presentation proposal](non_range_diagnostics.md) leaves Q1–Q5 open for owner approval before implementation and Task41; it does not relax the current whole-batch refusal.

## Unsaved-text overlays

[Lightweight frontend diagnostics](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lightweight-open-buffer-diagnostics) use current editor text and version, are marked `current`, and claim no verified semantic status. They remain version-keyed overlay diagnostics, separate from the published build index, and never reach CLI output or artifacts, including `VerifiedArtifact`.

Overlay replacement follows internal03; current delivery and retained reads follow [snapshot invalidation](snapshot.md#freshness-and-replacement) and [retention](snapshot.md#currentness-and-retention).

`Server::submit_originating_diagnostic_overlay` explicitly consumes the original native capture and invokes the existing guarded Frontend producer once. Only an eligible complete notification set installs the original immutable index and retained source association as `LspDiagnosticOverlay`, keyed by exact URI and document version. No caller index or diagnostic is accepted; ordinary submission/errors and existing notification/writer APIs remain unchanged. An unavailable result preserves storage; an eligible clean result replaces it with an empty index for that source.

`Server::diagnostic_overlay` selects an immutable `Arc<LspDiagnosticOverlay>` by URI/version only while its original cohort's raw tuples/incarnations and live Driver guard remain current. Current means editor-current Frontend output, without semantic verification. Each read exposes only that document’s original records and native source tuple; the cohort index and association remain privately shared for identity and retention; holding it after invalidation permits original-source inspection only. A new eligible result replaces overlapping keys; accepted edits/close remove affected cohort selections, and shutdown/exit remove all selections. Ignored/invalid/unrelated traffic does not evict them. Successful full snapshot activation removes matching URI/version keys; failed/partial work cannot clear them. Readers keep the original OpenBufferOverlay lease through last-reference release, independently of later selections. Storage stays inside Server, outside semantic publication, CLI rendering and artifact projection; automatic host integration remains task26/DRIVER-G-012.

## Ordering

Preserve the aggregator's [canonical record order](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#build-aggregation) (`BuildDiagnosticIndex::records`) when selecting diagnostics for a source; do not re-sort by message text, locale or asynchronous completion.

The existing explicit native frontend path groups notifications by URI in ordered-map order and preserves index record order within each notification. It does not establish a combined overlay/semantic merge order or a coherent full snapshot.

## Implementation boundary

`src/diagnostics.rs` implements task9. `Server::snapshot_diagnostic_notifications` selects a current retained snapshot; unavailable or stale selection returns no notifications. Conversion errors reject the entire batch, including non-range source-load records whose wire encoding remains undefined.

Notifications use URI order and the captured document version, including empty lists for clean sources. Primary ranges and ordered secondary `relatedInformation` use original raw maps. Diagnostic `data` contains the existing handle as `{snapshot: <published BuildSnapshotId>, id: <ordinal>}`; it refers to the unchanged record in the retained index, preserving anchor shape, generated reasons, provenance, notes and intent without duplicating them in a new representation. It grants no currentness or edit authority.

`Server::publish_snapshot_diagnostics_to_writer` retains that selection through conversion and writing, checking currentness and selection identity before each frame. It propagates conversion/I/O errors; external writes have no atomic-currentness, rollback or retry guarantee. Build scheduling/event consumption remains task11; the existing native frontend path remains syntax-only.

## Test design

Task9 tests cover genuine coherent publication, stale-index suppression, deterministic order, exact-source conversion, retained record identity, explicit batch failures and framed delivery. They grant no task11 or §23.6/SCA-003 completion credit.

`server.rs::diagnostic_overlay_tests` uses genuine native Frontend work and coherent publications for per-document version keys, source filtering, replacement/clean clearing, original-source currentness, last-reader lease retention and selective full-publication eviction. It compares the original semantic index, actual CLI output and serialized VerifiedArtifact before/after nonempty overlays; no host-integration credit follows.

The separate [ranged diagnostic host input](ranged_diagnostic_host.md) admits genuine terminal source-error records without a full semantic snapshot. Diagnostic notification/writer methods select that publication when present, preserving canonical data handles, original maps and whole-batch conversion checks.
