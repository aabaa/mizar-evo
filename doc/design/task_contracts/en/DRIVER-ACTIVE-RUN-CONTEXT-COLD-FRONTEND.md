# Task DRIVER-ACTIVE-RUN-CONTEXT-COLD-FRONTEND: cold native Disk Frontend
Canonical language: English; [Japanese pointer](../ja/DRIVER-ACTIVE-RUN-CONTEXT-COLD-FRONTEND.md).
Status: frozen scope. Tier: full (new Frontend runtime/cache binding).
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index). Consumer: [STEP6C](STEP6C-REAL-SERVICE-EQUIVALENCE.md), without completion credit.
Prerequisites: approved [Q1–Q6 A](../../mizar-driver/en/active_run_context_cache_binding.md#owner-q-decisions), especially Q5-A; [SourceLoad binding](DRIVER-ACTIVE-RUN-CONTEXT-CACHE-BINDING.md).

## Authority and boundary

Authority: spec [§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility); architecture 22 [equivalence](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence), [fallback](../../architecture/en/22.incremental_verification_contract.md#cache-miss-fallback), [freshness](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results), [footprints](../../architecture/en/22.incremental_verification_contract.md#dependency-slice-completeness); IR [expectations](../../mizar-ir/en/cache_adapter.md#producer-payload-expectations).
Existing [leaf encoding](../../mizar-driver/en/frontend_adapter.md#native-leaf-frontend-record-encoding) and [retained lookup](../../mizar-driver/en/frontend_adapter.md#retained-native-leaf-frontend-lookup) define complete keys, storage, side tables, lineage and decoding. `external_dependency_gap`: independent cold expectations and Frontend runtime association are missing.
Generate complete current expectations from protected native Disk capture through genuine `Frontend::run_loaded`, independently of Store records and retained Frontend handles. Run full Frontend analysis; no parsing-avoidance credit. Admit only diagnostic-free AST-bearing zero-import output, no dependency artifacts/summaries/hashes and one exact own current physical SourceLoad parent. Unknown footprints refuse.
Use unchanged owner codecs/key profiles and complete semantic/storage/side/named-input expectations; compare complete incoming raw storage before decoding and preserve Cache key/produced-by and IR validation. Never publish an unchecked payload to test equivalence. Rebind current SourceId/maps through existing decoder.
Extend the private assembled-runtime association to actual selected Frontend, participating implementation/codec residency and descriptors. Preserve installed Proof policy equality, config/lock association, actual plan/index/task correspondence, per-package Store routing, pure registry queries, registered NoKey, currentness, cancellation, scoped retention and fallback precedence. No transferable context or public seam.
Integrate cold lookup into existing native cache submission modes where leaf lookup is eligible; retained-oracle routing remains compatible. Missing SourceLoad hit, unsupported producer/profile, mismatches or lookup/decode errors leave ordinary execution eligible. PersistentPrefix keeps existing best-effort encoding/writes and guards; a miss may repeat Frontend analysis.
Writes: existing Driver registry, source/Frontend adapters, bootstrap/scheduler cache integration and Driver tests; contract pair and Driver plan rows; paired Driver boundary documents only as needed. Lower Build/Frontend/Proof/Cache/IR responsibilities remain unchanged.
Forbidden: spec/.miz/expectation/trace/audit-credit changes, AGENTS/protocol/global TODO edits, new policy or compatibility equivalence, release attestation, imports/Open/proof/witness/reuse activation, full STEP6C credit. New semantic choices stop for owner decision.

## Test matrix and exit

Six cases, reusing native Disk, retained, persistence, runtime-image and Cache/IR matrices:
1. Genuine cold Store writes followed by warm and fresh-capture lookup without retained Frontend handles equal independent-clean complete keys/bytes/maps/sides/lineage/diagnostics/outcomes; resident/blob, workers/order and SourceId rebinding.
2. Otherwise viable source/raw/module/package/snapshot/dispatch/plan/config/lock/policy mismatches and cross-run swaps refuse or miss with ordinary-result equivalence.
3. Actual assembled Frontend/runtime/schema binding admits; manual/spoofed/missing association, incompatible running artifact and unsupported linkage refuse without semantic changes.
4. Current/physical parent, cancellation/supersession/publisher replacement/collection and retained-reader controls retain live-use guards and release all acquired owners on success/refusal/error.
5. Disabled/unsupported/imported/diagnostic/missing-AST/dependency profiles and real Store/lookup/decode failure preserve fallback and invalid-input precedence; coherent wrong aggregate bytes with rewritten self-hashes refuse.
6. Existing retained lookup, direct APIs, registered NoKey, persistent writes and ordinary clean execution remain compatible; context grants no later acceptance/publication authority.
Independent specification review before freeze; sufficiency, precision, volume/scope and final consistency reviews afterward. Extra cases require an authority clause or .miz test.
Freeze: both lint_policy suites, links/fragments, EN/JA parity, diff check. Implementation: fmt, denied-warning all-target/all-feature Clippy, both lint_policy suites, Driver/Frontend/Cache and IR cache-adapter owner groups, short proof-verification E2E. Affected cache/lifecycle matrices run; unchanged resolver/evidence heavy groups defer under cadence.
Exit: freeze and implementation local commits, one changed task, no push; full cargo test only on cadence trigger. Contract cap: 60 lines/660 words; documentation cap: 200 added lines.
