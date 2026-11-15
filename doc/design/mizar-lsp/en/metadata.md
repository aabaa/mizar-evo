# Published metadata reader
Canonical language: English; [Japanese companion](../ja/metadata.md).

## Responsibility

The metadata reader reads published artifact metadata for IDE features under [internal03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-bridge). Hover/navigation consume published artifacts and `LspSnapshot` metadata, never raw compiler IR. This reader does not perform type checking, resolution, proof acceptance or artifact mutation.

## Published inputs

Use the selected snapshot's manifest-resolved outputs under [snapshot freshness](snapshot.md#freshness-and-replacement). Read `VerifiedArtifact.expressions` as existing [ExpressionMetadata](../../mizar-artifact/en/verified_artifact.md#expression-metadata), preserving producer-owned expression identities, native source ranges and rendered summaries. Optional metadata remains optional; the reader does not infer missing types or resolve missing symbols.

Read existing [ModuleSummary](../../mizar-artifact/en/module_summary.md#scope) projections for module identity and dependency-facing exported symbol, label, lexical and reexport information. Module summaries exclude expression metadata; they do not replace `VerifiedArtifact.expressions` or supply local semantic detail that was not published.

Completion producer inputs follow the approved [publication contract](producer_metadata_publication.md). `SnapshotPublication::source_metadata` retains each manifest-backed summary and optional immutable notation/import views with the original source association; unavailable products differ from complete-empty views. Native profile eligibility is unchanged.

Artifact owns schemas, compatibility, canonical ordering and validation. Use its existing [validating readers](../../mizar-artifact/en/verified_artifact.md#reader-and-writer-requirements) and [summary reader rules](../../mizar-artifact/en/module_summary.md#reader-and-writer-requirements); this design adds no field, summary shape, format or validation policy.

## Snapshot association and freshness

One metadata read retains one immutable [source association](snapshot.md#source-association) and its matching published metadata throughout the request. Do not mix old expression rows or summaries with a newer snapshot's sources/maps. Range conversion uses the existing [range boundary](range.md) and the associated original source.

Apply the per-snapshot current/stale_artifact/pending rules and [retention](snapshot.md#currentness-and-retention) unchanged. Mark stale semantic data explicitly rather than hiding it solely because it is stale. Retained stale ranges stay associated with their original source; current-buffer delivery and unsafe edits remain subject to snapshot validation.

Missing artifact inputs remain unavailable and degrade IDE features gracefully; they do not imply an empty successful snapshot or scheduled work. Syntax-only open-buffer overlays do not supply semantic metadata. This reader neither establishes currentness nor derives it from an equal build id or retained handle.

## Projection boundary

The upstream [Artifact Projection Boundary](../../internal/en/06.ir_storage_and_snapshot_handles.md#artifact-projection-boundary) owns projection from sealed phase outputs. After publication, [artifact readers](../../internal/en/06.ir_storage_and_snapshot_handles.md#artifact-projection) consume stable projections rather than internal IR storage.

The metadata reader does not load raw AST/IR, follow compiler storage references or reconstruct missing semantic metadata through compiler phases. Rendered metadata and expression identities do not establish proof authority.

## Implementation boundary

`Server::snapshot_hover` reads expression metadata from one retained publication using the original OpenBuffer URI and UTF-16 position. The smallest enclosing half-open expression range wins, with expression identity breaking equal-range ties. Plaintext hover includes published surface, identity and available type, symbol, coercion, thesis and overload summaries; missing fields are not reconstructed.

Current hover includes the original mapped LSP range. Stale hover preserves metadata, identifies `stale_artifact` and the original source URI/version, states that lookup coordinates belong to that retained source, and omits the current-buffer LSP range. Invalid source associations/ranges fail explicitly; absent metadata yields no hover.

`Server::handle_message_with_metadata` supplies Driver/registry freshness to hover requests; the build bridge uses this route. The ordinary handler validates requests and returns no hover without a metadata host. Executable host/build integration and navigation remain separate.

Reads use already-published inputs and the snapshot owner's established association. This specification does not choose OpenBuffer provenance carriers or unsaved-artifact storage/publication; build scheduling is owned by [build bridge](build_bridge.md).

## Test design

Artifact fixtures cover rendered optional metadata, expression selection and original raw-source mapping. Genuine coherent publications cover request serving, stale marking and replacement. This reader does not complete §23.6 or the SCA-003 annotation projection audit.
