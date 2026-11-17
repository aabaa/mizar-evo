# Task DRIVER-INTERMEDIATE-WORKSPACE-FRONTEND-CACHING-DESIGN: bounded intermediate caching proposal
Canonical language: English; [Japanese pointer](../ja/DRIVER-INTERMEDIATE-WORKSPACE-FRONTEND-CACHING-DESIGN.md).
Status: frozen definition scope; proposed decisions pending owner approval. Tier: full (expanded cache-admission boundary).
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index). Consumer: [STEP6C](STEP6C-REAL-SERVICE-EQUIVALENCE.md), without completion credit.
Purpose: define native Disk Frontend caching for the same-package three-module chain of [STEP6A45](STEP6A45-WORKSPACE-INTERMEDIATE-LEXICAL-HANDOFF.md); no implementation.

## Authority and readiness

Authority: spec [lexical preprocessing](../../../spec/en/02.lexical_structure.md#210-lexical-preprocessing), [import conflicts](../../../spec/en/11.symbol_management.md#1124-precedence-rules), [imports](../../../spec/en/12.modules_and_namespaces.md#123-import-statements), [exports](../../../spec/en/12.modules_and_namespaces.md#124-export-statements), [reproducibility](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility); architecture 22 [footprints](../../architecture/en/22.incremental_verification_contract.md#frontend-lexicalparse-footprints), [completeness](../../architecture/en/22.incremental_verification_contract.md#dependency-slice-completeness), [equivalence](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence), [freshness](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results), [fallback](../../architecture/en/22.incremental_verification_contract.md#cache-miss-fallback).
Prerequisites: landed [multiple-leaf caching](DRIVER-MULTIPLE-DIRECT-LEAF-FRONTEND-CACHING.md), [intermediate summary owner](../../mizar-driver/en/frontend_adapter.md#workspace-intermediate-summaries), Cache [lookup](../../mizar-cache/en/cache_store.md#lookup), IR [expectations](../../mizar-ir/en/cache_adapter.md#producer-payload-expectations).
Gap: `design_gap` for ancestor-footprint authentication and lifecycle beyond direct leaves; implementation/evidence remain `external_dependency_gap`/`test_gap`. Definition is authorized; admission requires owner Q-decisions and a subsequent Muse implementation dispatch/freeze.

## Scope and boundaries

Author paired `doc/design/mizar-driver/{en,ja}/intermediate_workspace_frontend_caching.md`: exact three-module closure, ancestor authentication and projection coverage, independent cold expectations, actual provider/runtime association, currentness/retention, guarded persistence, conditional matrix and recommended Q-decisions with alternatives.
Bound the proposed new route to same-package importer → intermediate → import-free ancestor, all captured clean AST-bearing Disk modules without reexports; preserve existing zero-/direct-leaf routes. Exclude artifact dependencies, cycles, arbitrary depth/branching, cross-package intermediates, Open/Generated and diagnostic profiles from this optimization. Unsupported cases preserve ordinary execution.
Preserve direct local A13 exports only, complete-empty versus unavailable contributions, ordered duplicate import provenance and ordinary conflict/qualification/alias semantics. Ancestor input changes may alter intermediate lexing without implicitly exporting ancestor symbols to the importer.
Driver owns capture/runtime/provider binding; Build graph/readiness; Frontend preprocessing/keys/output; Resolver/Artifact resolution/A13 identities; Cache keys/Store; IR reconstruction/physical lineage/retention; Proof acceptance/reuse. Link existing owners; no new public seam or schema approval.
Define where current sealed-publication trust is sufficient and where independent ancestor evidence is required. Reuse complete owner keys/projections only with demonstrated coverage; unmapped inputs or compatibility changes become Q-decisions. Genuine full current-source analysis supplies expectations independently of Store and retained importer/intermediate results; no parsing-avoidance credit.
Writes: contract pair and one Task Index row per Driver plan at freeze; proposal pair and proposal links in those rows afterward. No other fan-out or audit/coverage impact.
Forbidden: code/test/fixture/spec/expectation/trace changes; AGENTS/protocol/global TODO edits; cache activation, new language/diagnostic/proof/acceptance policy, fabricated producers/hits, T1 N1/N2 follow-up, broader caching or full STEP6C completion.

## Review matrix and exit

Six document-review families; runtime checks conditional on owner approval:
1. Genuine three-module cold/warm/fresh-capture resident/blob outputs equal independent clean keys/bytes/maps/sides/lineage/diagnostics/outcomes; mixed Completed/CacheHit dependencies.
2. Fixed importer/intermediate bytes with ancestor lexical edits change intermediate token content and invalidate affected footprints; intermediate own-export and importer edits, duplicate/alias/branch provenance, worker/capture/local-ID permutations.
3. Otherwise viable stale/foreign/collected/missing/surplus/duplicate/swapped ancestor, source, summary or provider associations and coherent payload corruption refuse unchecked publication, even with equal export fingerprints.
4. Accepted graph readiness, all-level scoped retention through scheduler consumption, partial-acquisition cleanup, cancellation/supersession/publisher replacement and every terminal release path.
5. Unsupported artifact/reexport/cycle/deeper/branching/cross-package or incomplete profiles, conflicts and failing/disabled Store preserve ordinary results and real error precedence; one inadmissible ancestor cannot be omitted.
6. Existing zero-/one-/multiple-direct-leaf, retained/persistent/NoKey, canonical compatibility, later-service/proof/publication and STEP6C deferrals stay intact.
Existing intent: STEP6A45 real scheduled chain and ancestor-edit controls; Driver native_disk independent edit/clean and supplied-summary tests; affected Build/Frontend/Resolver/Cache/IR owner groups. No executable cases change; extra families require a specification clause or .miz test.
Reviews: specification/test-intent before freeze; sufficiency, precision, volume/scope and final consistency over the whole unit.
Each commit: both lint_policy suites, local links/fragments, EN/JA parity and diff checks. Documentation only; fmt/Clippy/runtime/E2E unrun. Full `cargo test` required on final pushed tree (push trigger).
Exit: freeze then paired pending proposal, two commits counting as one changed task; push after full success, no implementation. Caps: contract 60 lines/660 words; whole-unit documentation 200 added lines.
