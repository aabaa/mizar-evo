# Navigation features
Canonical language: English; [Japanese companion](../ja/navigation.md).

## Responsibility

Navigation serves definitions, references, document symbols and semantic tokens using the [architecture12 feature sources](../../architecture/en/12.diagnostics_and_lsp.md#request-sources). Under [internal03's LSP Bridge](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-bridge), LSP owns editor-facing conversion, not resolution, type checking, proof acceptance or artifact mutation.

## Feature sources

Definitions read resolved symbol metadata and export summaries through the [published metadata reader](metadata.md#published-inputs). Preserve producer-owned identities and source ranges; optional or absent metadata does not authorize resolving a missing symbol.

References read the symbol index built from artifacts and source snapshots in the selected [LspSnapshot](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot). This specification assigns the read source only; it defines no symbol-index structure or new artifact projection.

Document symbols read the parsed outline for the current buffer. Enrich that outline with artifact metadata only when fresh for that buffer; stale artifacts do not supply current outline enrichment.

Semantic tokens read lexer/parser output for the current buffer, adding resolved symbol classes only when fresh. Syntax-only tokens remain available without fresh semantic artifacts; do not apply stale semantic classes to current-buffer tokens.

## Snapshot association and freshness

Apply [source association](snapshot.md#source-association) and [freshness](snapshot.md#freshness-and-replacement) unchanged. Each request retains one immutable source/metadata association; replacement does not mix old and new inputs within an in-flight read. Convert native ranges through [range conversion](range.md) using their associated original source.

Retained semantic reads may be `stale_artifact`: mark them explicitly rather than hiding data solely because it is stale. Preserve their original source association, never relabel their ranges as current-buffer ranges, and suppress current delivery whose target cannot be validated. Current-buffer outline/token enrichment still requires fresh metadata.

Missing semantic inputs degrade gracefully to syntax-only features where available, without fabricating semantic facts, an empty successful snapshot or scheduled work. `pending` requires actually scheduled/running verification. A syntax-only overlay supplies no semantic metadata or coherent full snapshot; equal build ids or retained handles do not establish currentness. Keep existing [retention](snapshot.md#currentness-and-retention) rules.

## Projection boundary

Read existing published `VerifiedArtifact` expression metadata and module summaries under the metadata reader's [projection boundary](metadata.md#projection-boundary), together with snapshot data and current-buffer syntax inputs above. Never read raw compiler IR or reconstruct missing semantic metadata through compiler phases.

Artifact retains schema, summary shape, format and validating-reader ownership; navigation adds no metadata field, index representation, public diagnostic code/name/alias, or producer/reader/snapshot behavior.

## Implementation boundary

Task14 specifies reads only. Task7 registry/atomic activation, task9 diagnostic publication, task13 hover and task15 navigation implementation remain separate. Task15 is incremental and paced by metadata producers; this specification chooses no Rust API, transport encoding or lookup algorithm.

Navigation consumes published inputs and existing snapshot associations without presuming parked OpenBuffer artifact provenance/storage, Session SourceId encoding or registry activation decisions. None is decided here.

## Test design

This design unit changes no code or tests. Task15 retains implementation and feature verification ownership; module completion and §23.6/SCA-003 credit do not change.
