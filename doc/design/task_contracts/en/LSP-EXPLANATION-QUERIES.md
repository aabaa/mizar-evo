# Task LSP-EXPLANATION-QUERIES: explanation requests
Canonical language: English; [Japanese pointer](../ja/LSP-EXPLANATION-QUERIES.md).
Status: specified. Tier: full; first explanation-query consumer.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task19; consumers: editor clients.
Purpose: resolve existing published explanation handles through the diagnostics store.

## Authority and readiness

Follow [Task18](LSP-EXPLAIN-DESIGN.md), [on-demand resolution](../../mizar-lsp/en/explain.md#on-demand-resolution), [freshness](../../mizar-lsp/en/explain.md#freshness-and-stale-handles), [missing payload and cancellation](../../mizar-lsp/en/explain.md#missing-payload-and-cancellation), [range conversion](../../mizar-lsp/en/range.md), [snapshot association](../../mizar-lsp/en/snapshot.md#source-association), and the diagnostics-owned [store](../../mizar-diagnostics/en/explain.md).
Dependencies: LSP tasks17/18 and diagnostics task15 are complete. Missing request routing/conversion is implementation_gap. The existing store resolves registered bounded text, not external files or services; native explanation producers and expression/VC indexes are absent. Missing inputs remain unavailable.

## Scope and ownership

Add `mizar/explain` with params `snapshot` (published schema string) and exactly one of `diagnosticId` (unsigned ordinal), `exprId` or `vcId` (nonempty existing subject key). Select only handles attached to the retained publication's diagnostic records; expression/VC subjects match existing handle subject strings. Missing or ambiguous matches are unavailable, never synthetic handles. No new producer/index or phase-ID encoding.
Add a host entry point receiving Driver, SnapshotRegistry, immutable ExplanationStore and a cancellation predicate. Apply ordinary JSON-RPC/lifecycle/parameter validation. The synchronous stdio route without a host reports unavailable. Check cancellation before work and before returning; cancellation yields RequestCancelled without altering published state.
Validate request snapshot against the selected publication before lookup. An obsolete snapshot returns `stale_handle` and no payload. Retained reads after editor invalidation carry `stale_artifact` freshness and their original source association; never relabel them current. Store snapshot/hash checks remain authoritative. No retained snapshot yields unavailable. Preserve store Available/Missing/Stale/Unavailable outcomes without fabricated content; missing data includes the existing compact handle preview when present.
Return structured status, freshness, source snapshot, original primary source location (URI, document version, UTF-16 range), optional preview and bounded payload format/text/truncation/hash. Convert original range/point/generated anchors with retained native maps; unsupported or invalid source mappings fail closed. No current-buffer relabeling. The store owns byte/line bounds; text has no nested expansion, and LSP adds no numeric budget or truncation policy. No filesystem/cache/service reads or proof-status inference.
Writes: LSP source/tests, this contract pair, paired explain owner/TODO/plan pointers. No diagnostic/store/producer/artifact/snapshot behavior, protected spec/tests/trace, Task15 remainder, later task, protocol or AGENTS changes. §23.6 remains partial; SCA-003 unchanged.

## Test matrix and exit

Cap: six grouped cases: (1) diagnostic/subject queries, deterministic selection and order, absent/ambiguous/no-handle targets; (2) preview-only and registered artifact/cache/query backing, summary mismatch and missing payload with preview fallback; (3) Unicode byte/line bounds, structured text and truncation preserved exactly; (4) original UTF-16 locations with supplementary/combining text, BOM/CRLF/lone CR/EOF, range/point/generated anchors and invalid source refusal; (5) snapshot mismatch, genuine host publication/change/reopen/replacement and no-publication degradation, reusing existing store-level stale tests; (6) malformed params/lifecycle, host cancellation before/after resolution, immutable state and framed stdio unavailable response.
Independent specification/test-intent review before freeze; sufficiency, precision, volume/scope, consistency and final full-tier assessment after implementation.
Freeze: both lint_policy suites including link/fragment/pair checks. Implementation: fmt, workspace all-target/all-feature Clippy with warnings denied, both lint_policy suites, mizar-lsp and mizar-diagnostics tests, framed stdio E2E and full cargo test (requested push and inherited outstanding-task cadence).
Exit: freeze then implementation commits count as one changed task; Task19 and explain module complete for existing handles/store, absent producer data explicitly unavailable. Push only after full success on the pushed tree.
