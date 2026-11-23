# Task VC53-PROOF-CALLER-E2-NATIVE-IDENTITY: bounded native postcondition identity
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E2-NATIVE-IDENTITY.md).
Status: frozen bounded identity. Tier: full; new public method fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [native adoption](VC53-PROOF-CALLER-E2-NATIVE.md), [passive E2 identity](VC53-PROOF-CALLER-E2-IDENTITY.md).
Authority: [§§20.1.2, 20.2.6, 20.4.1, 20.7.2, 20.13.1 E2](../../../spec/en/20.algorithm_and_verification.md), [identity boundary](../../architecture/en/22.incremental_verification_contract.md#identity-model-vcid-vs-obligationanchor).
Invariants: [native adoption](../../mizar-proof/en/status.md#native-caller-e2-adoption), [whole association](../../mizar-proof/en/status.md#arena-local-caller-e2-association), [runner/test design](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; existing E2 adoption and passive identity lack native composition. No language-intent gap or external prerequisite.

## Frozen scope
Add only `NativeUnaryCallerE2::obligation_identity(source, typed, symbols, core, request, policy)` in Proof status.rs, returning `Result<Option<(ProofObligationIdentity, DependencySlice)>, String>`. FIRST call unchanged `adopt_original_request` once on all six inputs; propagate errors and return None on false. On true return the retained association's unchanged `obligation_identity` on identical five inputs, preserving E2 hashes, domain, anchor and whole conservative slice.
Fresh policy checks remain mandatory each call. Authentication precedes whole-request/tuple/exact-arena mismatch, which precedes policy refusal. Currentness stays caller-owned; canonical equality cannot authorize another arena.
No new producer, field, helper, fixture or lower-stage production change. Exclude schema storage, complete footprints/cache, whole caller acceptance/status/currentness receipts, promotion/publication, broader profiles and owner-gated scope. Preserve child/E4/E1/direct-return and raw unknown/incomplete/cache-miss boundaries. Muse sealed-tree integration remains separate.
No language intent, diagnostics, spec/.miz/expectation/trace/audit-status or AGENTS/protocol changes.
Files: Proof status.rs, existing runner tests/algorithm.rs, paired Proof status/plan/API-inventory documents and this contract. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm independent four-row/Kernel-backed setup and one native identity success before expansion. No new shared fixture.
1. Independent canonical/passive oracle: native identity equals every retained passive identity and whole-slice field; repeat/purity preserve inputs and products. Existing normalization/owner/flag variants retain their equality or distinction.
2. Policy refusal/recovery: eligible policies return the unchanged identity/slice; denied formats/Builtin/checker-schema preserve exact policy errors. Earlier success cannot bypass current policy. Passive identity remains policy-independent.
3. Complete existing request/Core and calibrated source/typed/environment drift matrices preserve authentication errors before mismatch and denied policy; authenticated cross-source/whole-tuple mismatch and equal cloned Core return None. Fresh own-source/clone holder succeeds with unchanged identity; combined-error precedence and recovery hold.
4. Freshly lower no-clause inputs, replacing the stale-Core-first control, and exercise canonical, passive and native identity rejection. Preserve direct-return/unsupported controls and conservative unknown/incomplete/cache-miss without broader authority.
Existing oracle: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) and existing unary caller ensures/drift families. No test-intent change.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Kernel/Proof/VC owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed identity/policy/authentication matrices run; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: freeze committed before implementation; native E2 identity only after original adoption under current policy, without later authority.
