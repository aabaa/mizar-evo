# Task ARTIFACT-COMMENT-PROJECTION-DEFINITION: Q1 projection definitions
Canonical language: English; [Japanese pointer](../ja/ARTIFACT-COMMENT-PROJECTION-DEFINITION.md).
Status: frozen definition scope. Tier: full, documentation only.
Owner: [Artifact schema](../../mizar-artifact/en/00.crate_plan.md); consumers: [Frontend](../../mizar-frontend/en/00.crate_plan.md) producer, [IR](../../mizar-ir/en/00.crate_plan.md) projector, [Driver](../../mizar-driver/en/00.crate_plan.md) orchestrator, [Doc](../../mizar-doc/en/00.crate_plan.md) reader of this contract.
Purpose: define the owner-approved Q1 artifact comment/map projection at emission; freeze plus paired proposal is one changed task.

## Authority And Readiness
Authority: spec [2.9/2.10](../../../spec/en/02.lexical_structure.md#29-comments-and-annotations), [24.1](../../../spec/en/24.documentation_generation.md#241-documentation-comments-syntax); architecture [02 metadata](../../architecture/en/02.source_and_frontend.md#comments-and-doc-comments-are-source-metadata), [13 attachment](../../architecture/en/13.documentation_and_extraction.md#step-2-attach-doc-comments); internal [05 inputs](../../internal/en/05.documentation_extraction.md#documentation-inputs).
Owner approval of Doc6 Q1–Q4 selects all ①: emission-time artifact projection; file-leading module only and ordinary trivia/standalone annotations stop adjacency; remove `:::` plus at most one ASCII space; default presentation warnings retaining text. No ② route is authorized.
Existing [Artifact schema](../../mizar-artifact/en/verified_artifact.md#ownership), [Frontend preprocessing](../../mizar-frontend/en/preprocess.md), [IR projection](../../mizar-ir/en/projection.md#ownership), [Driver emission](../../mizar-driver/en/frontend_adapter.md#source-theorem-unpublished-module-draft), and [Doc inputs](../../mizar-doc/en/comments.md#inputs-and-provenance) bound the definitions.
Missing published comments/maps are design_drift and deferred source_gap; raw lexical inputs are not attachment evidence. Doc7 remains gated on completed Q1 definitions and implemented prerequisites plus Q2 module-leading-trivia/multiple-block and attachment-diagnostic definitions. Doc8 additionally needs Doc7, tag-validation and CommonMark definitions.

## Scope And Ownership
Write this paired contract, paired Artifact `comment_projection.md`, and necessary paired Artifact schema/plan, Doc comments/build/TODO, architecture13/internal05 pointers or approval/transport/gating corrections only.
Artifact owns the projected data contract, availability, compatibility, integrity and hash rules; explicitly assign upstream comment/map production, IR projection, Driver orchestration and Doc consumption. Link existing owner-local invariants rather than copying APIs.
Define unavailable versus available-empty/nonempty data, full publication/module/source/item binding, coordinate frames/maps and adjacency evidence, concrete handoffs, legacy compatibility and canonical/hash participation. These are definitions only; no runtime API or serialized version number is selected.
Consequential adaptations of approved decisions are in scope. A new semantic choice or language-intent change stops the unit for an owner Q-decision; never select an unapproved alternative.
Forbidden: implementation/transport, Rust/Cargo/tests/fixtures/expectations/trace/spec/protocol/AGENTS changes, source-recovery fallback, attachment/Markdown implementation, new proof/cache/reuse authority, Artifact17 or Doc7/8/Steps6–8 closure, and unrelated owner-gated work.
Audit impact: coverage, trace status and follow-up ownership remain unchanged; leave the coverage audit untouched.

## Review And Verification Matrix
No executable tests change. Documentation acceptance has six cases (cap six):
1. Current/legacy publication with unavailable, available-empty or nonempty comments: explicit availability, no empty substitution or source fallback.
2. Disk/OpenBuffer source, package/version/module/source and colliding item names: preserve genuine identity; no currentness, publication or verified attachment inferred from ranges or names.
3. UTF8/BOM/line-ending normalization, lexical masking, blank lines, ordinary trivia and attached/standalone annotations: named coordinate frames, valid maps and sufficient adjacency evidence without defining open Q2 semantics.
4. Raw empty/nonempty `:::` lines and ordinary comments: preserve text/order and Q3 body mapping; no Markdown/tag interpretation at emission.
5. Unsupported/legacy schema, changed comments/maps and forged/stale/hash/path/range/identity inputs: preserve compatibility, canonical hashing and integrity refusals separately from presentation warnings.
6. EN/JA parity, links/fragments, budgets and handoffs: assign every owner and keep downstream prerequisites/Artifact17 open.
Reviews: independent specification before freeze; proposal sufficiency, precision, volume/scope, final source/document consistency and task-scoped full gates. Repeat specification only for scope change.
Both commits: CARGO_BUILD_JOBS=4 cargo test -p mizar-checker --test lint_policy; CARGO_BUILD_JOBS=4 cargo test -p mizar-test --test lint_policy (links/fragments/pairing/volume included); git diff --check. No fmt/Clippy/runtime checks for documentation-only changes. Full cargo test on final tree: requested push and inherited tenth-task cadence.
Exit: reviewed definitions committed and pushed after full verification; any new semantic choice returns to the owner instead of implementation. No downstream completion credit.
