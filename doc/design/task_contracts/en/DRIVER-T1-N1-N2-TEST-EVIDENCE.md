# Task DRIVER-T1-N1-N2-TEST-EVIDENCE: cache refusal evidence
Canonical language: English; [Japanese pointer](../ja/DRIVER-T1-N1-N2-TEST-EVIDENCE.md).
Status: frozen test-only scope. Tier: pattern.
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index). Consumer: existing native Disk Frontend cache tests.
Predecessor: full-tier [multiple direct-leaf caching](DRIVER-MULTIPLE-DIRECT-LEAF-FRONTEND-CACHING.md); dependency: landed [bounded intermediate caching](DRIVER-INTERMEDIATE-WORKSPACE-FRONTEND-CACHING.md). No owner decision pending.

## Authority and boundaries

Authority: predecessor [approved authority](../../mizar-driver/en/multiple_direct_leaf_frontend_caching.md#status-and-authority), [imported cache authentication](../../mizar-driver/en/frontend_adapter.md#imported-native-disk-frontend-cache), [intermediate persistence](../../mizar-driver/en/intermediate_workspace_frontend_caching.md#currentness-retention-and-persistence), Cache [Store lookup](../../mizar-cache/en/cache_store.md#lookup), IR [producer expectations](../../mizar-ir/en/cache_adapter.md#producer-payload-expectations).
Repair two test_gap findings from the T1 audit: swapped-lineage refusal currently exits at lower publication identity rejection; repeated inadmissible/conflicting execution does not itself prove absence of Store writes. Preserve identity guards and ordinary semantics.
Writes: existing private tests in crates/mizar-driver/src/frontend_adapter/source.rs and crates/mizar-driver/tests/source_load/native_disk.rs, plus this contract pair and paired Driver-plan registration rows. No production behavior, new API/type/abstraction, lower-owner change, spec/.miz/expectation/trace/audit-credit, AGENTS/protocol/global TODO, or owner-document boundary change.
The stale transitive-leaf no-write assertion intent is narrowed to the inadmissible multi-leaf importer: the approved intermediate route permits independently eligible dependency writes. Conflicting importers still receive no cache write. Preserve all clean-equivalence and eligible-write controls.

## Test matrix and exit

Case cap: two existing control families; extra cases require an authority clause or .miz test.
1. N1: genuine viable multi-leaf record and equal-export leaves; publish a valid lower-owner Frontend with the other leaf's SourceLoad lineage. Assert publication/storage authorization succeeds, then invoke cache authentication and assert refusal with zero successful rehydrations. No early success on publication rejection; preserve unrelated attack controls and IR guards.
2. N2: existing multi-leaf importer with one intermediate member, and lexical-conflict importer, after viable cached baselines and on repetition. Inspect actual Store records to prove no new importer record; substantiate eligible dependency records, including the intermediate dependency now admitted. Retain ordinary scheduled-result and independent-clean comparisons.
Reviews: independent contract/test-intent before freeze; combined implementation/test-sufficiency afterward, then volume/scope. Parent checks source/documentation consistency; no coverage-audit boundary change.
Freeze: both lint_policy suites, local links/fragments, EN/JA pointer parity and diff checks. Implementation: fmt, all-target/all-feature Clippy with denied warnings, both lint_policy suites, Driver tests including both changed families and cache oracle/metamorphic controls, affected Cache/IR tests, short proof-verification E2E, links and diff checks.
Unchanged evidence/proof/Resolver heavy obligations defer to periodic/integration cadence; full cargo test only on a named cadence trigger.
Exit: freeze then implementation local commits, one changed task, push:no; no production change or new semantic choice. Contract at most 60 lines/660 words; test-only derived documentation at most 60 added lines.
