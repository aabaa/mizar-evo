# Task LSP-SNAPSHOT-REGISTRY-ACTIVATION: full snapshot registry and activation
Canonical language: English; [Japanese pointer](../ja/LSP-SNAPSHOT-REGISTRY-ACTIVATION.md).
Status: frozen scope; implementation follows freeze. Tier: full; first registry/IR-retention owner and public activation seam.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md), task7. Consumers: task9 diagnostics and later metadata readers.
Purpose: complete task7 using the genuine coherent producer landed in `27e55711e`; task9 remains a separate implementation.

## Authority and readiness

Authority: [producer contract](LSP-SNAPSHOT-REGISTRY-PRODUCER-DESIGN.md#scope-and-ownership) and its [exit](LSP-SNAPSHOT-REGISTRY-PRODUCER-DESIGN.md#verification-and-exit); [retained sources](LSP-RETAINED-NATIVE-SOURCES.md), [freshness](LSP-SNAPSHOT-FRESHNESS-DESIGN.md), [diagnostic design](LSP-DIAGNOSTIC-PUBLICATION-DESIGN.md); [snapshot association](../../mizar-lsp/en/snapshot.md#source-association), [replacement](../../mizar-lsp/en/snapshot.md#freshness-and-replacement), [retention](../../mizar-lsp/en/snapshot.md#currentness-and-retention), [test design](../../mizar-lsp/en/snapshot.md#test-design); [diagnostics](../../mizar-lsp/en/diagnostics.md); internal03 [snapshot](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot)/[publication](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lsp-snapshot-publication); [tasks7–9](../../mizar-lsp/en/todo.md#ordered-task-list).
Gap: source_drift/test_gap, not new semantics: retained native sources exist, but the full current/stale artifact registry, independent IR retention and atomic full activation are absent. The coherent producer is available; older absence wording is stale.
Existing tests: server.rs retained_native_sources_share_original_objects_across_equal_id_replacement, retained_native_selection_preserves_ignored_and_failed_capture_but_evicts_changes, retained_native_leases_follow_real_ticket_reader_and_driver_ownership and full_sync_keeps_exact_text_and_rejects_invalid_updates_atomically; Driver genuine snapshot producer tests remain unchanged.
OPEN DECISION: none identified; stop for any unresolved semantic-intent conflict with these authorities.

## Scope and ownership

Implement in order, as one implementation commit after the freeze:
- R1 registry and retention: manifest-first loading of genuine SnapshotPublished payloads; retain one complete immutable set of original sources, diagnostic index, manifest-backed module metadata and typed artifact/IR outputs. Track current/stale selection against exact accepted editor versions/incarnations, including equal-id invalidation; stale reads retain original source association explicitly. Independently acquire/release actual PublishedLspSnapshot Session leases and actual sealed IR retain/release through the last association/reader.
- R2 atomic activation: validate and retain the entire set before one Server-owned pointer replacement. Existing readers finish on their original set; subsequent readers select the complete replacement. Reject stale/superseded payloads and incomplete/noncoherent inputs without replacing the selection, fabricating pending work or clearing diagnostics.
LSP owns registry/indexes and activation; Driver keeps production coherence/currentness; Session and IR keep retention authority; Artifact keeps manifests. Reuse original payload objects and existing public APIs; no parallel metadata representation or new module file.
Writes: existing crates/mizar-lsp/src files and crates/mizar-lsp/tests; paired LSP snapshot/diagnostics owners only for boundary changes; paired task7 TODO completion. Freeze writes only this pair, one task7 Task Index row per LSP plan, and the four snapshot/diagnostics availability updates. No supporting plan is added.
Forbidden: task9 conversion/delivery/publication, task10 scheduling/cancellation, placeholder snapshots/registries/indexes/events, fabricated producer/cache/allocator results or pending work, language/proof changes, Driver/Session/IR/diagnostics crate edits, spec/.miz/expectation/trace/coverage-credit changes. No diagnostic clearing on absent producer/current maps.
Audit: existing §23.6/SCA-003 credit and owner/deferral mapping remain unchanged.

## Test matrix

Cap: eight new cases total, using genuine producer outputs and accepted editor messages only.
1. Complete genuine SnapshotPublished payload: manifest-first load atomically activates one coherent full current LspSnapshot.
2. Monotonic accepted edits, including version-only changes: versions remain monotonic and invalidate prior currentness.
3. Equal-id version-only/normalization-equivalent raw edits and identical-tuple reopen: invalidate currentness; no equal-id IR replacement or obsolete-id revival.
4. Read retained across edit: explicit stale_artifact with original sources, never current; retention does not override IR validity/currentness.
5. Reader spans replacement: one complete old/new set, never mixed; later reads select the new complete set.
6. Independent Session leases and IR retention: genuine lease ids and actual retain/release results hold through last reader/association, then release exactly once.
7. Partial/frontend-only/index-only/ArtifactCommitted-only input: no full activation, pending fabrication or diagnostic clearing.
8. Stale/superseded payload: cannot activate current.

## Verification and exit

Freeze: independent specification/test-intent review, both lint_policy suites and local links/fragments/pair/cap checks; no fmt/Clippy.
Implementation: full-tier sufficiency, precision, volume/scope, consistency and final assessment; cargo fmt then cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test -p mizar-checker --test lint_policy; cargo test -p mizar-test --test lint_policy; affected mizar-lsp groups and a genuine real-build SnapshotPublished end-to-end check.
Full cargo test --locked only on a named cadence trigger; none applies to this local fourth outstanding task. Freeze plus implementation counts as one changed task; push disabled.
Exit: all eight cases pass, task7 is checked in both TODOs and task9 stays unchecked but startable. Exactly two local commits, clean tree; stop if required edits exceed scope or an OPEN DECISION arises.
