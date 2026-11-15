# Task LSP-EXPLAIN-DESIGN: explanation-query design
Canonical language: English; [Japanese pointer](../ja/LSP-EXPLAIN-DESIGN.md).
Status: specified (design only). Tier: full; no frozen predecessor contract for this explanation-query surface.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task18. Consumer: task19.
Purpose: specify on-demand resolution of existing explanation handles to bounded payloads and latency/size limits.

## Authority and readiness

Authority: [LSP task18](../../mizar-lsp/en/todo.md), [roadmap Step8 item2](../../todo.md#step-8--user-facing-consumer-wave-); internal03 [Explanation Queries](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#explanation-queries), [Explanation Store](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#explanation-store), [Explanation Reference](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#explanation-reference), [LSP Snapshot Errors](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot-errors) and [Cancellation](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#cancellation); architecture12 [lazy bounded data](../../architecture/en/12.diagnostics_and_lsp.md#explanation-data-is-lazy-and-bounded), [Request Sources](../../architecture/en/12.diagnostics_and_lsp.md#request-sources), [Incremental Update Flow](../../architecture/en/12.diagnostics_and_lsp.md#incremental-update-flow) and [Explanation Reference](../../architecture/en/12.diagnostics_and_lsp.md#explanation-reference).
Dependency: completed task8 [diagnostic publication](../../mizar-lsp/en/diagnostics.md#build-index-publication); [code actions](../../mizar-lsp/en/code_action.md#request-sources-and-conversion) revalidate captured handles. Task19 additionally depends on task17 and mizar-diagnostics task15.
Gap: absent explanation-query owner document is design_drift. Authorities determine this design independently of parked OpenBuffer provenance/storage, Session SourceId encoding and registry activation decisions; none is decided here.

## Scope and ownership

Author paired mizar-lsp/{en,ja}/explain.md, each at most 80 lines. Reuse ExplanationRef as a handle, not an embedded trace; define no new kind, field or payload shape.
Follow the four-step query flow: request DiagnosticId, ExprId or VcId plus snapshot id; resolve against artifact hashes/cache keys; enforce size/depth bounds; return freshness, source ranges and bounded structured explanation data. Query artifacts/services on demand, never bulk compiler-state dumps.
Preserve snapshot/artifact-hash validity and invalidate previous-snapshot handles. Obsolete-snapshot handles receive a stale-handle response, optionally suggesting a re-query on a current diagnostic with matching code/span.
Missing payloads show compact diagnostic preview and report explanation unavailable without fabricated content. Explanation queries are cancellable; published snapshots are not. No numeric latency/size/depth limits are fixed by authority, so invent none; cache-backed data never determines proof status.
LSP owns query conversion; ExplanationStore/query service owns resolution and bounds. Reuse [range conversion](../../mizar-lsp/en/range.md) and [snapshot association](../../mizar-lsp/en/snapshot.md#source-association); change no producer, diagnostics, artifact or snapshot behavior.
Forbidden: protected spec/test/trace changes; task9 publication, task17/task19 implementation or mizar-diagnostics task15 store implementation; deciding parked SourceId/registry matters.
Writes: contract pair and exactly one task18 Task Index row per LSP plan first; then specification pair and task18-only TODO pointers; new-pair audit rows only if required. Outside dispatch paths, only NT2 mechanically required registration lines are permitted.
Coverage: design only; no module completion or §23.6/SCA-003 credit. Task19 stays open.

## Verification and exit

Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and source/document consistency over the whole unit, then full-tier assessment.
Fixed matrix, no test/code additions/changes/deletions: before freeze and after specification commit run both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, plus local link/fragment checks.
Specification heading sets/anchors match after heading-text normalization; contract companion is a pointer. English contract cap: 60 lines/660 words. `cargo test --locked -p mizar-lsp` passes. No fmt/Clippy for either documentation commit.
Exactly two commits, one changed task: freeze then specification/task18 completion. After the second, run `cargo test --locked --no-fail-fast` at default stack (unset RUST_MIN_STACK); tenth outstanding task and requested push trigger this gate. Only exit 0 permits pushing origin main; then HEAD equals origin/main and worktree is clean.
Exit diff contains only dispatch paths/NT2 registrations. Stop for required undetermined behavior (classified gap/exact decision), cap failure, out-of-scope required lint/link repair, or full-gate failure; no further commit or push after full failure.
