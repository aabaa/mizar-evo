# Task DRIVER-MULTIPLE-DIRECT-LEAF-FRONTEND-CACHING-DESIGN: multiple direct-leaf caching proposal
Canonical language: English; [Japanese pointer](../ja/DRIVER-MULTIPLE-DIRECT-LEAF-FRONTEND-CACHING-DESIGN.md).
Status: frozen definition scope; proposed decisions pending owner approval. Tier: full (expanded cache-admission boundary).
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index). Consumer: [STEP6C](STEP6C-REAL-SERVICE-EQUIVALENCE.md), without completion credit.
Purpose: define the multiple-target prerequisite of [Q1-B in the approved one-leaf proposal](../../mizar-driver/en/dependency_aware_frontend_caching.md#owner-q-decisions); no implementation.

## Authority and readiness

Authority: spec [lexical preprocessing](../../../spec/en/02.lexical_structure.md#210-lexical-preprocessing), [import conflicts](../../../spec/en/11.symbol_management.md#1124-precedence-rules), [imports](../../../spec/en/12.modules_and_namespaces.md#123-import-statements), [reproducibility](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility); architecture 22 [footprints](../../architecture/en/22.incremental_verification_contract.md#frontend-lexicalparse-footprints), [completeness](../../architecture/en/22.incremental_verification_contract.md#dependency-slice-completeness), [equivalence](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence), [freshness](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results), [fallback](../../architecture/en/22.incremental_verification_contract.md#cache-miss-fallback); Cache [lookup](../../mizar-cache/en/cache_store.md#lookup); IR [expectations](../../mizar-ir/en/cache_adapter.md#producer-payload-expectations).
Prerequisites: approved [one-leaf proposal](../../mizar-driver/en/dependency_aware_frontend_caching.md), landed [bounded contract](DRIVER-DEPENDENCY-AWARE-FRONTEND-CACHING.md), [cache-backed independent comparison](STEP6C-REAL-SERVICE-EQUIVALENCE.md).
Gap: `design_gap` for multiple-target closure/admission; implementation remains `external_dependency_gap`/`test_gap`. Definition authorized; owner Q-decisions and a subsequent Muse implementation dispatch/freeze required for admission.

## Scope and boundaries

Author paired `doc/design/mizar-driver/{en,ja}/multiple_direct_leaf_frontend_caching.md`: authenticated distinct leaf/parent/summary correspondence; existing projection coverage and missing mappings; independent expectations; actual provider/runtime association; dependency readiness, scoped retention, conditional verification and recommended Q-decisions with alternatives.
Propose multiple distinct direct captured import-free workspace leaves only. Preserve ordered duplicate import provenance, target-owned metadata, complete-empty versus unavailable contributions, and existing import/conflict semantics. Exclude intermediate/transitive/artifact/reexport/cyclic/Open/Generated/diagnostic profiles from the optimization; ordinary execution remains authoritative.
Driver owns orchestration/capture/provider binding; Build planning/readiness; Frontend preprocessing/keys/lexical/parse output; Resolver/Artifact resolution/A13 identities; Cache canonical keys/compatibility/Store; IR reconstruction/physical lineage/retention; Proof policy/acceptance/reuse. No new owner seam is approved.
Reuse complete owner keys and existing dependency projection where sufficient; expose unmapped coverage rather than inventing hashes. Genuine full analysis with exact authenticated summaries supplies expectations independently of Store/retained importer. No parsing-avoidance credit.
Writes: contract pair and one Task Index row per Driver plan at freeze; proposal pair and links in those rows afterward. No other fan-out or audit/coverage impact.
Forbidden: production/test/fixture/spec/expectation/trace changes; AGENTS/protocol/global TODO edits; activation, new language/diagnostic/compatibility/proof policy, fabricated producers/hits, broader caching/Proof reuse or full STEP6C completion.

## Review matrix and exit

Six document-review families; runtime checks conditional on owner approval:
1. Multiple genuine leaves, duplicate imports and complete-empty exports: real cold/warm/fresh-capture resident/blob results equal independent clean keys/bytes/maps/sides/lineage/diagnostics/outcomes.
2. Per-leaf source/lexical/metadata and import membership/provenance/alias changes invalidate; worker/completion/capture order and local IDs preserve canonical identity.
3. Individually viable records with stale/foreign/collected/swapped/missing/surplus/duplicate parents, summary/provider mismatch or coherent corruption refuse unchecked publication.
4. Staggered accepted parent readiness, mixed Completed/CacheHit parents, cancellation/supersession/publisher replacement and partial acquisition release all scoped owners correctly.
5. Conflicting same-spelling imports error; qualification/alias resolution follows ordinary semantics. Unsupported/incomplete/transitive/artifact profiles and disabled/failing Store preserve ordinary results, diagnostics and producer-error precedence.
6. Existing zero-/one-leaf, retained/persistent/NoKey, schemas/compatibility, later-service and proof/publication deferrals remain intact.
Existing intent: Driver `source_load/{native_disk,supplied,misc}.rs` and `frontend.rs::frontend_imported_symbol_conflict_uses_real_captured_summaries`, Build readiness and Frontend/Cache/IR owner matrices. No executable cases change; extra families require an authority clause or .miz test.
Reviews: specification/test-intent before freeze; sufficiency, precision, volume/scope, final consistency over the whole unit.
Each commit: both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, local links/fragments, EN/JA parity and diff checks. Documentation only; fmt/Clippy/runtime/E2E unrun; full cargo test only on cadence trigger.
Exit: freeze then paired pending proposal, two local commits counting as one changed task; no push/implementation. Caps: contract 60 lines/660 words; whole-unit documentation 200 added lines.
