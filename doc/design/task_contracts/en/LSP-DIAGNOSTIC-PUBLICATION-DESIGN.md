# Task LSP-DIAGNOSTIC-PUBLICATION-DESIGN: diagnostic publication design
Canonical language: English; [Japanese pointer](../ja/LSP-DIAGNOSTIC-PUBLICATION-DESIGN.md).
Status: specified (design only). Tier: full; no frozen predecessor contract for this publication surface.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task8. Consumers: tasks9–11, 16 and 18.
Purpose: specify publication of existing diagnostics through the LSP snapshot boundary without changing producers or public diagnostic allocation.

## Authority and readiness

Authority: LSP task8 in `doc/design/mizar-lsp/en/todo.md`; [spec §22.1.2](../../../spec/en/22.error_handling_and_diagnostics.md#2212-source-span-and-context-display), internal03 [Aggregator](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#diagnostic-aggregator), [aggregation](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#build-aggregation), [publication](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot-publication), [overlays](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lightweight-open-buffer-diagnostics) and [range conversion](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#range-conversion).
Dependencies: task6 [snapshot](../../mizar-lsp/en/snapshot.md#freshness-and-replacement); existing BuildDiagnosticIndex behavior. Task7's full registry and task9 implementation remain separate.
Gap: absent publication owner document is design_drift. Coherent full-snapshot production remains deferred at the snapshot owner's existing boundary; no new semantic gap is resolved here.

## Scope and ownership

Author paired mizar-lsp/{en,ja}/diagnostics.md, each at most 80 lines: immutable build-index publication, exact-source range conversion, unsaved-text overlays excluded from CLI/artifacts, and aggregator-owned ordering.
Reuse [range conversion](../../mizar-lsp/en/range.md), [source association](../../mizar-lsp/en/snapshot.md#source-association), [retention](../../mizar-lsp/en/snapshot.md#currentness-and-retention) and upstream aggregation; do not duplicate their APIs or representations.
Session owns maps/text; diagnostics owns normalization/identity/deduplication/order; Driver owns scheduling/currentness; Artifact owns manifests; LSP owns protocol conversion and current delivery.
Forbidden: new public codes/names/aliases, producer changes, overlay CLI/artifact projection, task7 registry or task9 implementation, fabricated coherent events/indexes, proof/acceptance or protected spec/test/trace changes. Public allocation stays behind registry and consumer-adoption gates.
Writes: this contract pair and one Task Index backlink per LSP plan in the freeze; then paired diagnostics owners and task8-only TODO completion pointers. No other files or executable/test edits.
Coverage: design-only; §23.6/SCA-003 credit and task9/module implementation state do not change.

## Verification and exit

Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and source/document consistency over the complete unit, then full-tier assessment.
Fixed matrix (no new test cases/files): documentation-only freeze and final trees each pass both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, plus local link/fragment checks.
Existing task9 publication fixtures, stale-index suppression and deterministic-order tests stay deferred to that implementation task; this unit adds no executable cases.
Both new pairs preserve heading sets and anchors after language normalization; the contract companion remains a pointer stub, with matching headings.
Final tree passes `cargo test --locked -p mizar-lsp`; combined two-commit file set is restricted to the dispatch and `git status --porcelain` is empty.
No fmt/Clippy or full workspace test: documentation only, no cadence trigger, push disabled. One changed task across exactly two local commits.
Exit: contract frozen first, paired specification and task8 completion second; task9 stays open. Stop for undetermined required behavior (classified gap), size-cap failure, or required lint/link fixes outside the allowed files.
