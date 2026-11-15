# Task LSP-NAVIGATION-DESIGN: navigation design
Canonical language: English; [Japanese pointer](../ja/LSP-NAVIGATION-DESIGN.md).
Status: specified (design only). Tier: full; no frozen predecessor contract for this navigation surface.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task14. Consumer: task15.
Purpose: specify definitions, references, document symbols and semantic tokens over published artifact metadata and snapshot data.

## Authority and readiness

Authority: [LSP task14](../../mizar-lsp/en/todo.md), roadmap Step8 item2; architecture12 [Request Sources](../../architecture/en/12.diagnostics_and_lsp.md#request-sources); internal03 [LSP Bridge](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-bridge) and [LSP Snapshot](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot).
Dependency: completed task12 [metadata inputs](../../mizar-lsp/en/metadata.md#published-inputs) and [projection boundary](../../mizar-lsp/en/metadata.md#projection-boundary); task6 [source association](../../mizar-lsp/en/snapshot.md#source-association), [freshness](../../mizar-lsp/en/snapshot.md#freshness-and-replacement) and [retention](../../mizar-lsp/en/snapshot.md#currentness-and-retention).
Gap: missing navigation owner document is design_drift. Authorities determine the feature sources and boundaries; producer availability paces task15, not this design.

## Scope and ownership

Author paired mizar-lsp/{en,ja}/navigation.md, each at most 80 lines. Definitions read resolved symbol metadata/export summaries; references read a symbol index built from artifacts/source snapshots. Document symbols read the current-buffer parsed outline, enriched by fresh artifacts; semantic tokens read current-buffer lexer/parser output, plus resolved symbol classes when fresh.
Apply snapshot freshness unchanged: mark retained stale data, never serve it as current; preserve one immutable source/metadata association and existing range conversion. Missing semantic inputs degrade gracefully to syntax-only features. No raw compiler IR reads.
Artifact owns published VerifiedArtifact expression metadata, module summaries and validating readers; snapshot owns freshness/retention; navigation owns feature reads, not new schemas, index structures or Rust APIs.
Forbidden: new metadata fields, summary shapes, formats, public diagnostic codes/names/aliases, changed producer/artifact/reader/snapshot behavior, or protected spec/test/trace edits.
Task7 registry/atomic activation, task9 publication, task13 hover and task15 implementation remain separate. Parked OpenBuffer provenance/storage and Session SourceId encoding are neither prerequisites nor decisions here.
Writes: contract pair and one task14 Task Index row per LSP plan first; then navigation pair and task14-only TODO pointers; new-pair audit rows only if required. Outside dispatch paths, only mechanically required registration lines under NT2 are permitted.
Coverage: no implementation, module-status or §23.6/SCA-003 completion credit; task15 stays open.

## Verification and exit

Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and source/document consistency over the whole unit, then full-tier assessment.
Fixed matrix, no test additions/changes/deletions: before freeze and after specification commit, both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, plus local link/fragment checks, pass.
Specification headings/anchors match after heading-text normalization; contract companion is a pointer. English contract cap: 60 lines/660 words. `cargo test --locked -p mizar-lsp` passes.
Exactly two local documentation commits, one changed task; combined diff limited to dispatch paths/NT2 registration lines; final `git status --porcelain` empty. No fmt/Clippy/full workspace test: no cadence trigger, push disabled.
Exit: freeze first, specification/task14 completion second. Stop for required undetermined behavior (classified gap/exact decision), size-cap failure, or required lint/link fixes beyond dispatch/NT2 scope.
