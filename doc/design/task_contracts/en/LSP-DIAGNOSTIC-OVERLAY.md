# Task LSP-DIAGNOSTIC-OVERLAY: lightweight diagnostic storage
Canonical language: English; [Japanese pointer](../ja/LSP-DIAGNOSTIC-OVERLAY.md).
Status: implemented. Tier: full.
Owner: [LSP plan](../../mizar-lsp/en/00.crate_plan.md); consumer: [task42](../../mizar-lsp/en/todo.md).

## Authority and readiness

Authority: [unsaved-text overlays](../../mizar-lsp/en/diagnostics.md#unsaved-text-overlays), internal03 [aggregation](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#build-aggregation)/[lightweight diagnostics](../../internal/en/03.diagnostics_model_and_lsp_bridge.md#lightweight-open-buffer-diagnostics), and existing [source association](../../mizar-lsp/en/snapshot.md#source-association)/[retention](../../mizar-lsp/en/snapshot.md#currentness-and-retention).
Dependency task9 is complete. Genuine native Frontend production, originating capture, immutable source retention and coherent full publication exist. G8 is source_drift/test_gap: transient native notifications do not implement diagnostic storage or prove its isolation. Import-discovery ModuleDependencyOverlay is unrelated.
API, replacement, retained-read and currentness decisions belong to [diagnostics](../../mizar-lsp/en/diagnostics.md#unsaved-text-overlays); existing producer eligibility belongs to [Server](../../mizar-lsp/en/server.md#native-frontend-diagnostic-notifications).
Existing evidence: server.rs native notification/originating/retained-source tests, RegistryFixture and task9 publication tests; diagnostics.rs conversion tests. Preserve their intent and assertions.

## Scope

Add explicit originating submission/storage and version-keyed retained diagnostic reads within mizar-lsp. Reuse genuine native Frontend eligibility; retain the original diagnostic index and source association, without inserting overlays into the semantic build index. Successful eligible output, including a clean empty result, replaces that source's selection; absent/ineligible/error output preserves prior storage and grants no clean clearing. Full coherent publication removes overlays for its source versions. Old readers keep their original records/maps/leases through replacement and invalidation.
Affected artifacts: crates/mizar-lsp/src/{server,diagnostics}.rs and localized tests; paired diagnostics/plan owners, task42 TODO and G8 correspondence, this contract/pointer. No new language coverage or root coverage-audit ownership change.
Forbidden: protected spec/.miz/expectation/trace changes, diagnostic identity/code/order changes, fabricated producers or acceptance, CLI/artifact/schema changes, semantic/overlay merging, import-overlay changes, AGENTS/protocol edits, host activation, tasks26/38–41 or other tasks.
Native host/producer integration remains task26/DRIVER-G-012; this unit adds explicit storage using the existing producer only. Session/Driver/IR/Artifact authority stays upstream; retained syntax reads confer no proof, current-delivery or artifact authority.

## Test matrix and exit

Cap: six grouped cases using real accepted messages and native producers.
1. Error and clean open buffers: exact URI/version keys, source-filtered original current Frontend records/order and source/map identity in a two-source cohort; unavailable/wrong-version reads refuse.
2. Same-version replacement and newer-version replacement: new selection, clean clearing, old immutable reader unchanged; ineligible/error work preserves selection without manufacturing success.
3. Version-only/normalization-equivalent changes, close/identical reopen, unrelated/invalid traffic, other Server and terminal messages: current reads follow original tuples/incarnations; retained reads remain original.
4. Actual OpenBufferOverlay lease survives replacement/invalidation while a reader exists, releases exactly at last owner; do not fabricate retention or rely on timing.
5. Genuine full publication supersedes only matching source versions, including selective eviction in a two-source cohort and equal normalized build ids; retained overlay reader survives, partial/unavailable full work cannot clear it.
6. Nonempty syntax overlay versus genuine semantic publication: original semantic index, CLI rendering and serialized VerifiedArtifact remain unchanged and contain no overlay diagnostics; syntax-only work creates no full snapshot.
Independent specification/test-intent review before freeze, then sufficiency, precision, volume/scope, consistency and final assessment over the whole unit. Resolve findings within this scope.
Freeze verification: both lint_policy suites and affected links/fragments. Implementation: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; both lint_policy suites; cargo test -p mizar-lsp, affected diagnostic/CLI projection checks and short executable stdio check. Full cargo test only on a cadence trigger; push:no.
Exit: matrix passes, owner docs synchronized, task42 checked and G8 resolved for storage/isolation only; host integration remains deferred. Freeze plus implementation count as one changed task.
