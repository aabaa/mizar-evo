# Task LSP-METADATA-READER-DESIGN: metadata reader design
Canonical language: English; [Japanese pointer](../ja/LSP-METADATA-READER-DESIGN.md).
Status: specified (design only). Tier: full; no frozen predecessor contract for this reader surface.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task12. Consumers: tasks13–18.
Purpose: specify reading existing published expression metadata and module summaries for IDE features, with snapshot freshness and no raw IR.

## Authority and readiness

Authority: [LSP task12](../../mizar-lsp/en/todo.md), roadmap Step8 item2; internal03 [LSP Bridge](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-bridge), [LSP Snapshot](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot) and [Artifact Projection](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#artifact-projection); internal06 [projection boundary](../../internal/en/06.ir_storage_and_snapshot_handles.md#artifact-projection-boundary).
Dependencies: task6 [source association](../../mizar-lsp/en/snapshot.md#source-association), [freshness](../../mizar-lsp/en/snapshot.md#freshness-and-replacement) and [retention](../../mizar-lsp/en/snapshot.md#currentness-and-retention); existing [expression metadata](../../mizar-artifact/en/verified_artifact.md#expression-metadata), [readers](../../mizar-artifact/en/verified_artifact.md#reader-and-writer-requirements) and [module summaries](../../mizar-artifact/en/module_summary.md#scope).
Gap: missing metadata owner document is design_drift. Existing artifact schemas/readers determine the inputs; no required semantic decision is open within this design scope.

## Scope and ownership

Author paired mizar-lsp/{en,ja}/metadata.md, each at most 80 lines: read existing published VerifiedArtifact expression metadata and ModuleSummary projections; preserve immutable snapshot association and explicit stale status; never read raw compiler IR.
Artifact owns formats, validation and projection; Session owns source/maps; snapshot.md owns freshness/retention; metadata owns IDE-facing reads. Reference those owners without copying schemas or inventing a Rust API.
Task7 registry/atomic activation, task9 diagnostic publication, task10 build scheduling and task13 hover implementation remain separate. OpenBuffer provenance carrier/storage decisions and the parked scheduling policy are neither prerequisites nor decisions of this specification.
Forbidden: new fields/formats/summary shapes, public diagnostic codes/names/aliases, changed producer/reader behavior, semantic reconstruction from raw IR, or protected spec/test/trace edits.
Writes: this contract pair and one task12 Task Index row per LSP plan first; then metadata pair and task12-only TODO completion pointers (new-pair audit rows only if established practice requires them). No production or test changes.
Coverage: no implementation, module-status or §23.6/SCA-003 completion credit; task13 stays open.

## Verification and exit

Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and source/document consistency over the whole unit, then full-tier assessment.
Fixed matrix, no test additions/changes/deletions: before freeze and after specification commit, both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, plus local link/fragment checks, pass.
Specification headings and anchors match across languages after heading-text normalization; the contract companion is a pointer stub. English contract cap: 60 lines/660 words.
`cargo test --locked -p mizar-lsp` passes; combined two-commit diff stays within dispatch paths; final `git status --porcelain` is empty.
No fmt/Clippy/full workspace test: documentation only, no cadence trigger, push disabled. Exactly two local commits, one changed task.
Exit: freeze first, paired specification and task12 completion second. Stop for required undetermined behavior (classified gap/exact decision), size-cap failure, or required lint/link fixes outside allowed files.
