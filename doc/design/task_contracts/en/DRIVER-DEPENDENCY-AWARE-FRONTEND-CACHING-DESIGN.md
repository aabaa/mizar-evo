# Task DRIVER-DEPENDENCY-AWARE-FRONTEND-CACHING-DESIGN: dependency-aware native Disk Frontend proposal
Canonical language: English; [Japanese pointer](../ja/DRIVER-DEPENDENCY-AWARE-FRONTEND-CACHING-DESIGN.md).
Status: frozen definition scope; proposal choices require owner approval. Tier: full (new dependency/cache boundary, not a repeated producer slice).
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index). Consumers: native Disk cache orchestration and [STEP6C](STEP6C-REAL-SERVICE-EQUIVALENCE.md), without completion credit.
Purpose: define a direct workspace-leaf import prerequisite; no implementation or imported cache admission.

## Authority and readiness

Authority: spec [§12.3 imports](../../../spec/en/12.modules_and_namespaces.md#123-import-statements) and [§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility); architecture 22 [equivalence](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence), [fallback](../../architecture/en/22.incremental_verification_contract.md#cache-miss-fallback), [freshness](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results), [lexical/parse footprints](../../architecture/en/22.incremental_verification_contract.md#frontend-lexicalparse-footprints); Cache [fingerprint targets](../../mizar-cache/en/dependency_fingerprint.md#fingerprint-targets) and [lookup](../../mizar-cache/en/cache_store.md#lookup); IR [expectations](../../mizar-ir/en/cache_adapter.md#producer-payload-expectations).
Prerequisites: approved [active-run Q1–Q6 A](../../mizar-driver/en/active_run_context_cache_binding.md#owner-q-decisions), [cold zero-import contract](DRIVER-ACTIVE-RUN-CONTEXT-COLD-FRONTEND.md), genuine [workspace leaf producers](../../mizar-driver/en/frontend_adapter.md#supplied-workspace-leaf-summaries), [scheduler prefix](../../mizar-driver/en/driver.md#scheduled-sourceloadfrontend-prefix).
Gap: `design_gap` for complete imported Frontend key/expectation/runtime/scheduling association; admission remains `external_dependency_gap`/`test_gap`. Genuine leaf producers do not establish cache eligibility. Definition is authorized; implementation requires owner decisions and a subsequent contract freeze.

## Scope and boundaries

Author paired `doc/design/mizar-driver/{en,ja}/dependency_aware_frontend_caching.md`: existing producers and missing seams, complete lexical/parse input closure, authenticated current parent/summary correspondence, independent expectations, actual provider/runtime association, scheduler timing/retention, explicit Q-decisions with alternatives/recommendations, conditional tests and implementation prerequisites.
Bound the proposed first profile to one direct captured workspace target whose genuine Frontend output is an import-free leaf. Define refusal for missing/unknown closure, transitive/artifact/Open/reexport/cycle profiles and diagnostics without inventing semantics. Preserve duplicate import provenance and existing frontend/resolver decisions.
Driver owns capture/runtime/provider association and orchestration; Build owns planning/scheduling; Frontend/Resolver/Artifact own parsing, lexical export/summary identities and import behavior; Cache owns key/compatibility/Store validation; IR owns payload/lineage/retention. Proof retains policy, acceptance and reuse.
Proposed independent expectations use genuine full Frontend analysis with the same authenticated dependencies, independently of Store and retained importer output. No parsing-avoidance credit; unknown is never complete-empty. Keep pure keys separate from live guards and actual current physical parents.
Writes: contract pair and one Task Index row per Driver plan at freeze; proposal pair and proposal links in those rows afterward. No other fan-out or audit/coverage impact.
Forbidden: production/tests/fixtures/spec/expectation/trace/coverage changes, AGENTS/protocol/global TODO edits, cache activation, new language/diagnostic/proof/compatibility policy, fabricated producers/receipts/hits, full STEP6C claims, owner-gated implementation.

## Review matrix and exit

Six document-review cases; runtime obligations conditional on approval:
1. Genuine importer plus direct leaf, real cold writes/warm lookup and fresh captures equal independent clean complete keys/bytes/maps/sides/lineage/diagnostics/outcomes; resident/blob and SourceId rebinding.
2. Independently viable leaf lexical-summary/import-prelude/target/alias/notation/operator/parser/edition/source/config/lock/policy changes invalidate; unchanged worker/order/local IDs do not alter canonical identity.
3. Foreign/stale/collected/swapped/missing/surplus parents or summary/provider/runtime association and coherent corrupted records cannot gain authority from matching hashes or descriptors.
4. Scheduler lookup after genuine parents, cancellation/supersession/publisher replacement and every release path preserve currentness, physical validation and scoped retention; retained readers confer no current use.
5. Disabled/missing/failing Store and unsupported/incomplete/imported-transitive/artifact/diagnostic profiles preserve ordinary execution and invalid-input/producer-error precedence.
6. Zero-import, retained and persistent routes, registered NoKey, owner schemas/compatibility and remaining semantic/proof/publication deferrals stay intact.
Existing intent: Driver `source_load/native_disk.rs` cache tests and `supplied.rs` workspace-leaf and `misc.rs` scheduled-overlay tests; owner Frontend orchestration and Cache/IR adapter tests. No executable cases change.
Reviews: specification/test-intent before freeze; sufficiency, precision, volume/scope and final consistency over both commits. Extra cases require an authority clause or .miz test.
Each commit: both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, local links/fragments, EN/JA parity and diff checks. Documentation only; fmt/Clippy/runtime/E2E unrun; full cargo test only on a cadence trigger.
Exit: freeze then pending paired proposal, two local commits counting as one changed task; no push or implementation. Caps: contract 60 lines/660 words; whole-unit documentation 200 added lines.
