# Published metadata reader
Canonical language: English; [Japanese companion](../ja/metadata.md).

## Responsibility

The metadata reader reads published artifact metadata for IDE features under [internal03](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-bridge). Hover/navigation consume published artifacts and `LspSnapshot` metadata, never raw compiler IR. This reader does not perform type checking, resolution, proof acceptance or artifact mutation.

## Published inputs

Use the selected snapshot's manifest-resolved outputs under [snapshot freshness](snapshot.md#freshness-and-replacement). Read `VerifiedArtifact.expressions` as existing [ExpressionMetadata](../../mizar-artifact/en/verified_artifact.md#expression-metadata), preserving producer-owned expression identities, native source ranges and rendered summaries. Optional metadata remains optional; the reader does not infer missing types or resolve missing symbols.

Read existing [ModuleSummary](../../mizar-artifact/en/module_summary.md#scope) projections for module identity and dependency-facing exported symbol, label, lexical and reexport information. Module summaries exclude expression metadata; they do not replace `VerifiedArtifact.expressions` or supply local semantic detail that was not published.

Artifact owns schemas, compatibility, canonical ordering and validation. Use its existing [validating readers](../../mizar-artifact/en/verified_artifact.md#reader-and-writer-requirements) and [summary reader rules](../../mizar-artifact/en/module_summary.md#reader-and-writer-requirements); this design adds no field, summary shape, format or validation policy.

## Snapshot association and freshness

One metadata read retains one immutable [source association](snapshot.md#source-association) and its matching published metadata throughout the request. Do not mix old expression rows or summaries with a newer snapshot's sources/maps. Range conversion uses the existing [range boundary](range.md) and the associated original source.

Apply the per-snapshot current/stale_artifact/pending rules and [retention](snapshot.md#currentness-and-retention) unchanged. Mark stale semantic data explicitly rather than hiding it solely because it is stale. Retained stale ranges stay associated with their original source; current-buffer delivery and unsafe edits remain subject to snapshot validation.

Missing artifact inputs remain unavailable and degrade IDE features gracefully; they do not imply an empty successful snapshot or scheduled work. Syntax-only open-buffer overlays do not supply semantic metadata. This reader neither establishes currentness nor derives it from an equal build id or retained handle.

## Projection boundary

The upstream [Artifact Projection Boundary](../../internal/en/06.ir_storage_and_snapshot_handles.md#artifact-projection-boundary) owns projection from sealed phase outputs. After publication, [artifact readers](../../internal/en/06.ir_storage_and_snapshot_handles.md#artifact-projection) consume stable projections rather than internal IR storage.

The metadata reader does not load raw AST/IR, follow compiler storage references or reconstruct missing semantic metadata through compiler phases. Rendered metadata and expression identities do not establish proof authority.

## Implementation boundary

Task12 specifies the reader only. Task7's registry/atomic activation, task9's diagnostic publication, task10's build bridge and task13's hover/type implementation remain separate. No Rust API, transport response or feature lookup algorithm is selected here.

Reads use already-published inputs and the snapshot owner's established association. This specification does not choose OpenBuffer provenance carriers, unsaved-artifact storage/publication, or the parked build scheduling policy.

## Test design

No tests change in this design task. Task13 retains its artifact-backed hover fixture and stale-data-marking obligations; this document grants no module implementation or §23.6/SCA-003 completion credit.
