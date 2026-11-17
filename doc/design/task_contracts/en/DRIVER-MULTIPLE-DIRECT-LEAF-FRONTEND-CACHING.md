# Task DRIVER-MULTIPLE-DIRECT-LEAF-FRONTEND-CACHING: native Disk caching
Canonical language: English; [Japanese pointer](../ja/DRIVER-MULTIPLE-DIRECT-LEAF-FRONTEND-CACHING.md).
Status: frozen implementation scope. Tier: full (expanded cache-admission boundary).
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index). Consumer: [STEP6C](STEP6C-REAL-SERVICE-EQUIVALENCE.md), without completion credit.
Prerequisites: [definition](DRIVER-MULTIPLE-DIRECT-LEAF-FRONTEND-CACHING-DESIGN.md), [proposal](../../mizar-driver/en/multiple_direct_leaf_frontend_caching.md), Q1–Q5 all option A approved by owner on 2026-11-17; [one-leaf predecessor](DRIVER-DEPENDENCY-AWARE-FRONTEND-CACHING.md).

## Authority and scope

Authority: proposal [authority links](../../mizar-driver/en/multiple_direct_leaf_frontend_caching.md#status-and-authority), including spec lexical preprocessing/imports/conflicts/reproducibility, architecture complete footprints/clean equivalence/freshness/fallback, Cache keys/Store and IR expectations.
Resolve multiple-target design_gap/test_gap: admit any finite distinct direct captured import-free workspace Disk leaves, clean AST-bearing importer/leaves without reexports. Preserve complete-empty exports and ordered duplicate/alias/branch provenance. Intermediate/transitive/artifact/cyclic/Open/Generated/incomplete/diagnostic profiles withhold optimization; ordinary semantics remain authoritative.
Require exact resolved/prescanned/parsed target, authenticated immutable summary, accepted Frontend and captured SourceLoad bijection. Reject missing/surplus/duplicate/swapped/foreign/stale/collected associations even with equal fingerprints; authenticate each leaf through existing zero-import validation and target-owned metadata/A13 exports.
Q3 retains genuine full importer analysis with the bound provider and complete authenticated summaries before lookup, independent of Store/retained importer; preserve raw storage comparison, owner keys/decoding, semantic/side/named-input expectations and current SourceId/lineage reconstruction. No parsing-avoidance credit.

## Integration and encoding

Q2 retains unchanged complete FrontendCacheKeys and private JSON-array domain/schema mizar-driver/native-disk-frontend-dependencies/v1, Cache named input frontend-dependencies. Exact fields/codecs/order remain the [predecessor mapping](DRIVER-DEPENDENCY-AWARE-FRONTEND-CACHING.md#closure-mapping-and-integration): preprocessing/token canonical bytes; ordinal-ordered resolved provenance; target-sorted complete summary identity/fingerprint/owner-ordered shapes; phase/work-unit-sorted own SourceLoad/all leaf Frontends/all leaf SourceLoads with content/side commitments. Preserve native source/storage/config/lock/policy/runtime/schema inputs, owner ordering and zero-/one-leaf bytes. Unmapped coverage or encoding ambiguity stops for owner decision.
Q4 reuses the [readiness callback](../../mizar-driver/en/driver.md#dependency-ready-native-disk-frontend-binding) and exact accepted PackageResolve/own SourceLoad/every leaf Frontend results, Completed or CacheHit. Retain every physical dependency and reconstructed importer through scheduler consumption; partial acquisition releases prior owners. Recheck cancellation/supersession/publisher authority at existing handoffs; no speculative producers or synthetic completion.
Q5 extends guarded PersistentPrefix reads and writes together under identical whole-closure validation; one inadmissible leaf suppresses importer optimization without dropping members. Preserve valid independent leaf writes, best-effort Store fallback and real producer-error precedence.
Write paths: registry.rs::native_frontend_cache_parents; frontend_adapter/source.rs::prepare_native_disk_frontend_cache_payload and private tests; existing Driver scheduler/driver integration only if required; tests/source_load/{native_disk,supplied,misc,frontend}.rs; paired Driver frontend_adapter/driver/proposal/plan boundary documents and this contract pair.
Keep existing Driver/Build/Frontend/Resolver/Artifact/Cache/IR/Proof ownership; no public seam, lower-owner representation, spec/.miz/expectation/trace/audit-credit, AGENTS/protocol/global TODO, compatibility, later-service/proof/publication/reuse or full STEP6C change. New semantic choices stop.

## Test matrix and exit

Six families, inputs/outcomes as [approved verification](../../mizar-driver/en/multiple_direct_leaf_frontend_caching.md#conditional-verification-and-handoff):
1. Genuine two/three-target cold/warm/fresh-capture resident/blob, duplicate imports/complete-empty exports, independent clean full keys/bytes/maps/sides/lineage/diagnostics/outcomes; scheduling/local IDs preserve identity.
2. Independently edit each leaf with unchanged importer, including observable lexical tokens; membership/duplicate/alias/branch changes miss/refuse. Reuse unchanged owner invalidation controls.
3. Otherwise viable records: attack every parent/summary/provider association and coherent corruption; refuse unchecked publication, including equal fingerprints.
4. Stagger readiness, mix Completed/CacheHit, fail/cancel parents and cancel/supersede/replace publisher; assert all-parent retention and partial/terminal cleanup.
5. Exactly one inadmissible member: no importer hit/write, independent clean equivalence; conflicts/qualification/aliases, unsupported profiles and disabled/failing Store retain ordinary outcomes/error precedence.
6. Preserve zero-/one-leaf/direct/retained/persistent/NoKey bytes, compatibility and deferred credit.
Case cap: six families; extras require an authority clause or .miz test. Reuse existing Driver and affected Build/Frontend/Resolver/Cache/IR matrices.
Reviews: specification/test-intent before freeze; sufficiency, precision, volume/scope, final consistency afterward.
Freeze checks: both lint_policy suites, links/fragments, EN/JA parity, diff. Implementation: fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites, Driver/owner groups, short proof-verification E2E and full cargo test (tenth outstanding task).
Exit: freeze then implementation local commits, one changed task; no push. Contract 60 lines/660 words; whole-unit documentation 200 added lines. Full success required before landing; pre-push T1 audit follows under standing rules.
