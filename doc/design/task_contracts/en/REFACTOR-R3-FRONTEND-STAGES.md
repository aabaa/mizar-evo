# Task REFACTOR-R3-FRONTEND-STAGES: staged frontend adapters
Canonical language: English; [Japanese pointer](../ja/REFACTOR-R3-FRONTEND-STAGES.md).

## Contract

- Status: frozen; full tier. Primary owner: mizar-driver; consumers: registry, scheduler, mizar-lsp and public codec callers.
- Purpose: split the integration layer by stage, share publication/framing mechanics, and retire the audit H1 production-unreachable routes.
- Owners: [Driver plan](../../mizar-driver/en/00.crate_plan.md), [source services](../../mizar-driver/en/frontend_adapter.md), [registry boundary](../../mizar-driver/en/registry.md#ownership-boundary).
- Authority: [build lifecycle](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility), [Driver responsibility](../../internal/en/01.compiler_driver_and_pipeline_scheduler.md#compiler-driver), [snapshot rules](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results).
- Readiness: R1/R2 are landed; settled provenance/staging and facade/composition-root decisions remain binding.
- Evidence: owner-adopted design audit 2026-11-15, section 7 R3 and H1: mixed stage ownership, repeated publication/framing mechanics, and superseded test-only routes.
- Classification: bounded design/source drift; no language-specification, trace, coverage-credit or deferral change.

## Scope and decisions

- Keep frontend_adapter as a private composition module with existing live exports. Move caller, publication, codec and service implementations into private stage modules; source capture/cache/dependency, Frontend, theorem resolver/summary, signature/type/registration, Core/preparation, VC/discharge, candidate/check/witness and artifact commit retain existing ownership.
- Share checked length-prefixed framing and publish allocation/identity boilerplate at three or more real sites. Preserve domain strings, byte order, size/overflow limits, schema, parent order, named hashes, side tables, decoders, source routing, validation/error precedence, fresh Frontend execution and replay-based trust checks. Do not cache or pass previously computed semantic values to replace replay.
- Remove exactly these H1 methods and tests exclusively exercising them; retain assertions for every live route in mixed tests:
  `prepare_native_disk_frontend_cache_request`, `native_public_theorem_formula_text_candidates`, `native_algorithm_module_summary_candidate`, `native_theorem_metadata_with_frontend_parent`;
  `native_theorem_reflexivity_with_core_vc_parents`, `native_theorem_candidate_witness_file_with_core_vc_discharge_check_parents`, `native_algorithm_module_summary_with_frontend_parent`, `native_algorithm_core_with_frontend_parent`;
  `native_theorem_core_with_frontend_parent`, `native_algorithm_vcs_with_frontend_parent`, `native_theorem_vcs_with_frontend_parent`, `publish_native_algorithm_vcs`;
  `publish_native_theorem_vcs`, `publish_native_algorithm_core`, `publish_native_theorem_core`, `publish_native_algorithm_module_summary`.
- Confirm their lack of production callers before deletion; preserve independently exported codecs and live helpers. No replacement API or compatibility wrapper for retired methods.
- Allowed artifacts: frontend_adapter.rs and private children, lib export plumbing if necessary, affected driver tests and exact source-surface lint inventory, paired live Driver module/plan/correspondence owners and this contract. Historical contracts remain unchanged; retain link anchors when retiring live documentation.
- Forbidden: AGENTS/protocol/specification edits, owner-gated scope, Task 9/later R units, reopening R1/R2, new public seams/dependencies or semantic representations. No restructuring of Session identity/retention, lexer/syntax/parser/frontend/resolve, Artifact, Build scheduling, cache keys, IR projection, Kernel evidence, ATP execution/encoders or dependency prohibitions.
- General service completion, accepted inputs, semantic/proof policies, codec trust-model changes and broad documentation compaction remain deferred to their owners.

## Tests and exit

- Reuse live `crates/mizar-driver/tests/source_load/`, registry/cache/currentness/scheduling and LSP assertions. Removing unreachable-route tests grants no semantic coverage credit.
- Matrix (cap six groups):
  1. Captured singleton theorem, unsupported source and edited/stale parents retain success/refusal results in live stage suites ([Core tests](../../mizar-driver/en/frontend_adapter.md#native-core-caller-tests), [VC tests](../../mizar-driver/en/frontend_adapter.md#native-vc-caller-tests)).
  2. Empty/nonempty length-prefixed fields emit independently specified literal bytes with the original domain and little-endian lengths.
  3. Wrong domain, truncation, overflowing length, trailing data and oversize frames reject; valid boundary frames retain each caller limit.
  4. Genuine current theorem publication retains exact payload/side-table/parent identity; mismatched or stale parents reject ([publication tests](../../mizar-driver/en/frontend_adapter.md#native-core-publication-tests)).
  5. Exported algorithm codecs replay supported syntax exactly and refuse malformed or mismatched data ([storage tests](../../mizar-driver/en/frontend_adapter.md#native-algorithm-core-storage-tests), [VC storage tests](../../mizar-driver/en/frontend_adapter.md#native-algorithm-vc-storage-tests)).
  6. Genuine coherent OpenBuffer build through the production registry retains publication/currentness/retention (`mizar-lsp::server` coherent-snapshot tests).
- New tests are limited to framing equivalence and malformed-frame boundaries not already exercised. No new .miz fixture or expectation change.
- Reviews: independent specification/test intent, sufficiency, implementation precision, volume/scope, source/document consistency and final hard-gate/quality assessment.
- Freeze: both lint_policy suites including links/fragments/pairing/volume. Implementation: fmt, workspace all-target/all-feature Clippy with warnings denied, both lint_policy suites, full Driver/LSP owner suites and exact genuine coherent-snapshot E2E.
- Full cargo test follows cadence. Unchanged heavy lower-owner proof/evidence/resolver/cache-key obligations defer to periodic/integration gates; changed adapter replay/publication/cache obligations run in owner suites. Exit: focused gates, no unresolved findings and local implementation commit; no push.
