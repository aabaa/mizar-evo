# Task DRIVER-ACTIVE-RUN-CONTEXT-CACHE-BINDING-DESIGN: active-run cache context proposal
Canonical language: English; [Japanese pointer](../ja/DRIVER-ACTIVE-RUN-CONTEXT-CACHE-BINDING-DESIGN.md).
Status: frozen definition unit; proposed choices require owner approval. Tier: full (new runtime/cache binding boundary).
Owner: [Driver plan](../../mizar-driver/en/00.crate_plan.md#task-index). Consumers: native Disk cache orchestration and [STEP6C](STEP6C-REAL-SERVICE-EQUIVALENCE.md).
Purpose: define the missing genuine active-run context producer/binding without implementing or approving it.

## Authority and readiness

Authority: spec [§23.4](../../../spec/en/23.package_management_and_build_system.md#234-build-lifecycle-and-reproducibility); architecture 22 [equivalence](../../architecture/en/22.incremental_verification_contract.md#clean-build-equivalence), [fallback](../../architecture/en/22.incremental_verification_contract.md#cache-miss-fallback), [freshness](../../architecture/en/22.incremental_verification_contract.md#snapshot-scoped-results), [footprints](../../architecture/en/22.incremental_verification_contract.md#dependency-slice-completeness).
Existing boundaries: Driver [bootstrap](../../mizar-driver/en/driver.md), [native cache preparation](../../mizar-driver/en/frontend_adapter.md#native-disk-cache-preparation), [capture](../../mizar-driver/en/request.md#native-source-capture), [key purity](../../mizar-driver/en/registry.md#cache-key-purity-contract); Build [cache seam](../../mizar-build/en/cache_seam.md#scope); Proof [policy fingerprint](../../mizar-proof/en/policy.md#policy-fingerprint); Cache [readiness](../../mizar-cache/en/integration_readiness.md); IR [expectations](../../mizar-ir/en/cache_adapter.md#producer-payload-expectations).
Gap: `design_gap` in active-run producer/binding definition; implementation remains `external_dependency_gap`/`test_gap`. Supplied metadata/config/lock equality and retained output routes do not establish complete runtime provenance, effective policy association or cold Frontend expectations.
Readiness: definition is authorized; every real design choice stays pending in an explicit Q-decision with alternatives and a recommendation. Owner approval and a subsequent implementation-contract freeze are prerequisites to implementation.

## Scope and boundaries

Author paired `doc/design/mizar-driver/{en,ja}/active_run_context_cache_binding.md`: actual producer inventory and missing seams; proposed producer/consumer route; minimum identity/configuration/policy/dependency associations; lifetime/currentness/refusal rules; Q-decisions; conditional validation and implementation prerequisites.
Distinguish genuine selected runtime/services/toolchain/schema identities from caller labels, descriptor equality, digest equality, generic publication and retained readers. Define evidence required for each association; unavailable provenance stays unavailable, never synthesized.
Driver owns bootstrap/session association and orchestration. Build retains plan/config/lock projection and scheduling; Proof retains effective verifier-policy semantics and reuse/acceptance; Cache retains keys, compatibility, footprints and Store validation; Frontend and IR retain expectations, payloads and publication/retention ownership.
Forbidden: code/tests/fixtures/spec/expectations/trace/coverage-ledger changes; implementation or cache activation; new language, proof, diagnostic or compatibility policy; fabricated producers/receipts/hits; STEP6C completion or coverage credit; AGENTS/protocol/global TODO edits; owner-gated scope.
Writes: contract pair and one Task Index row per Driver plan in freeze; then proposal pair and proposal links in those rows. No other fan-out. Existing ownership and remaining runtime/Frontend, Proof reuse/dependency-binding and service gaps remain open.

## Review matrix and exit

Document-review matrix (cap six; runtime cases conditional on approval):
1. Genuine native Disk capture, actual bootstrap/registry/producers and selected effective policy form one run association; compare cold/warm behavior with independent clean execution without fabricated positive context/hits.
2. Independently viable source/module/package/snapshot/plan/config/lock/policy mismatches and cross-run context swaps refuse optimization without weaker policy or changed ordinary outcomes.
3. Unknown/spoofed service/toolchain/schema/backend provenance and unsupported compatibility/dependencies cannot gain authority from equal labels/hashes; distinguish exhaustive zero-dependency SourceLoad from incomplete later footprints.
4. Cancellation, supersession, publisher replacement, collected handles and retained readers preserve live guards, bounded retention/release and no stale current publication; no lease/token implies atomicity.
5. Disabled cache, unsupported origins/profiles, missing producers, lookup/decode/Store failure preserve existing miss/refusal/error precedence and ordinary execution; input errors are not silently repaired.
6. Frontend cold expectations, imports, proof/witness/reuse and complete STEP6C remain separately gated; authentic context alone grants no semantic acceptance, publication rights or completion.
Existing intent: `native_disk_cache_request_*`, `native_disk_cache_lookup_*`, `native_disk_cache_scheduled_results_match_clean_runs_and_release_owners` in `crates/mizar-driver/tests/source_load/native_disk.rs`; explicit config/lock tests in sibling `explicit.rs`; native toolchain tests in sibling `native_toolchain.rs`; configured/persistent/retained cache tests in sibling `misc.rs`. No executable cases change.
Reviews: independent specification/test-intent before freeze; sufficiency, precision, volume/scope and consistency/final assessment over both commits. Extra cases require an authority clause or .miz test.
Each commit: both `cargo test -p mizar-checker --test lint_policy` and `cargo test -p mizar-test --test lint_policy`, local links/fragments, EN/JA parity and `git diff --check`.
Documentation only: fmt/Clippy/runtime/E2E unrun; full `cargo test` only on a cadence trigger. Caps: contract 60 lines/660 words; whole-unit documentation 200 added lines.
Exit: reviewed contract freeze then paired pending proposal, two local commits counting as one changed task; owner choices stay unapproved, no implementation or push.
