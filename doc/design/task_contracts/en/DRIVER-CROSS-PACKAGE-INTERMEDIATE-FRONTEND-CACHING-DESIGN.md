# Task DRIVER-CROSS-PACKAGE-INTERMEDIATE-FRONTEND-CACHING-DESIGN: cross-package cache proposal
Canonical language: English; [Japanese pointer](../ja/DRIVER-CROSS-PACKAGE-INTERMEDIATE-FRONTEND-CACHING-DESIGN.md).
Status: frozen definition scope; proposal decisions pending owner approval. Tier: full (cache-admission boundary expansion).
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index). Consumer: [STEP6C](STEP6C-REAL-SERVICE-EQUIVALENCE.md), without completion credit.
Purpose: define native Disk Frontend caching for the existing [STEP6A47](STEP6A47-CROSS-PACKAGE-WORKSPACE-LEXICAL-HANDOFF.md) app/main→app/mid→core/base producer; no implementation.

## Authority and readiness

Authority: spec [lexical preprocessing](../../../spec/en/02.lexical_structure.md#210-lexical-preprocessing), [conflicts](../../../spec/en/11.symbol_management.md#1124-precedence-rules), [imports/exports](../../../spec/en/12.modules_and_namespaces.md#123-import-statements), [workspace](../../../spec/en/23.package_management_and_build_system.md#233-workspace-layout), [reproducibility](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility); architecture 22 [footprints](../../architecture/en/22.incremental_verification_contract.md#frontend-lexicalparse-footprints), [completeness](../../architecture/en/22.incremental_verification_contract.md#dependency-slice-completeness), [equivalence](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence), [currentness](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results), [fallback](../../architecture/en/22.incremental_verification_contract.md#cache-miss-fallback).
Prerequisites: [same-package cache contract](DRIVER-INTERMEDIATE-WORKSPACE-FRONTEND-CACHING.md), [approved proposal](../../mizar-driver/en/intermediate_workspace_frontend_caching.md), [workspace summaries](../../mizar-driver/en/frontend_adapter.md#supplied-workspace-leaf-summaries), Cache [lookup](../../mizar-cache/en/cache_store.md#lookup), IR [expectations](../../mizar-ir/en/cache_adapter.md#producer-payload-expectations).
Gap: `design_gap` for cross-package closure admission/authentication; implementation/evidence remain `external_dependency_gap`/`test_gap`. Definition is ready; implementation requires owner Q-decisions and subsequent Muse dispatch/freeze.

## Scope and boundaries

Author paired `doc/design/mizar-driver/{en,ja}/cross_package_intermediate_frontend_caching.md`: exact producer profile, per-package footprints, independent expectations, authentication, currentness/retention, guarded persistence, conditional verification and Q-decisions with alternatives.
Bound the proposal to three distinct clean AST-bearing captured native Disk modules I=app/main→M=app/mid→A=core/base, with two unique package plans and import-free A. Preserve local-only complete A13 exports, complete-empty versus unavailable, ordered duplicate import provenance and ordinary conflicts/aliases/qualification. No ancestor symbols implicitly become I-visible.
Adapt approved bottom-up full-source expectations and complete ancestor payload comparisons to each module's own package/index/version/edition/source and package-associated config, shared workspace lock and actual bound provider/runtime. Preserve summary lockfile_identity None. Define whether private v2 covers this topology; preserve v1 and same-package routes. Unmapped inputs/public compatibility changes return to owner.
Adapt dependency-ready lookup, all-level scoped retention and guarded genuine PersistentPrefix writes; preserve direct physical output lineage and ordinary failure precedence. Never omit an inadmissible ancestor.
Driver owns capture/provider binding; Build graph/readiness/package configuration; Frontend keys/output; Resolver/Artifact resolution/A13 identity; Cache Store/keys; IR reconstruction/retention; Proof acceptance/reuse. Link existing owners without new public seams.
Writes: contract pair and one Task Index row per Driver plan at freeze; proposal pair and proposal links in those rows afterward. No other fan-out or audit/coverage impact.
Forbidden: code/tests/fixtures/spec/expectations/trace, AGENTS/protocol/global TODO, activation, language/diagnostic/proof/acceptance changes, artifact/reexport/cycle/branch/deeper/general cross-package caching, Open/Generated, parsing-avoidance or full STEP6C credit. N1/N2 remain repaired and outside scope; Doc4 is not this unit.

## Review matrix and exit

Six document-review families; runtime checks conditional on approval:
1. Genuine two-package chain: cold/warm/fresh-capture resident/blob and mixed Completed/CacheHit outputs equal independent clean full results across worker/order/local-ID permutations.
2. Independent A lexical, M export and I source edits; target-owned version/edition/config changes with importer source fixed and separate shared-lock controls invalidate affected footprints; no implicit transitive exports.
3. Otherwise viable swapped/missing/surplus/duplicate/foreign/stale/collected package/index/root/source/summary/provider/edge evidence or coherently corrupted payloads refuse even with equal exports.
4. Exact graph readiness, complete retention, partial-acquisition cleanup, ancestor failure, cancellation/supersession/publisher replacement around work/I/O/handoff and every terminal release.
5. Unsupported/incomplete profiles and disabled/failing Store preserve ordinary outcomes/error precedence; one inadmissible member suppresses whole-importer hit/write while eligible writes stand.
6. Legacy v1, same-package v2, direct/retained/persistent/NoKey and later-service/proof/STEP6C deferrals remain intact.
Reuse STEP6A47 real producer/metadata refusals and existing intermediate independent edit-clean, refusal and lifecycle controls; proposal names exact tests. Extra families require specification or .miz authority.
Reviews: specification/test-intent before freeze; sufficiency, precision, volume/scope and final consistency over the whole unit.
Each commit: both lint_policy suites, local links/fragments, EN/JA parity and diff checks. Documentation only; fmt/Clippy/runtime/E2E unrun; full cargo test only on a named cadence trigger.
Exit: freeze then paired pending proposal, two local commits counting as one changed task; push:no. Contract ≤60 lines/660 words; whole-unit documentation ≤200 added lines. No implementation before owner approval.
