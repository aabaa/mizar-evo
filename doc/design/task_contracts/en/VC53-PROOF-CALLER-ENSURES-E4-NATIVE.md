# Task VC53-PROOF-CALLER-ENSURES-E4-NATIVE: clause-bearing caller native adoption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-ENSURES-E4-NATIVE.md).
Status: frozen bounded adoption. Tier: full; new public APIs fail pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [ensures consumer](VC53-PROOF-CALLER-ENSURES-E4-POLICY.md), [existing native holder](VC53-PROOF-CALLER-E4-NATIVE.md).
Authority: [§20.1.2, §20.2.6, §20.4.1, §20.7.2, §20.13.1 E4](../../../spec/en/20.algorithm_and_verification.md), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Invariants: [association](../../mizar-proof/en/status.md#arena-local-caller-e4-association), [policy](../../mizar-proof/en/status.md#caller-e4-current-policy-consumption), [native adoption](../../mizar-proof/en/status.md#native-caller-e4-adoption), [runner](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; clause-bearing association and policy consumption exist without explicit native adoption. No language-intent gap or external prerequisite.

## Frozen scope and design
Add only `from_source_with_ensures(source, typed, symbols, core, request, policy)` and `adopt_original_request_with_ensures` with the same six inputs to the existing `NativeUnaryCallerE4Existential` in Proof status.rs.
Construction calls the unchanged ensures consumer once, retaining its whole association in the existing field and discarding only the fingerprint.
Adoption FIRST calls the retained association's unchanged `matches_source_with_ensures`, preserving authentication errors. Whole-request/descriptor or exact Core-pointer mismatch returns false before policy checks.
On a match, inline require enabled `BuiltinKernelEvidence` and exact supplied policy checker-schema agreement with the retained actual checked result; otherwise return existing `algorithm.proof.unary_caller_e4_policy_rejected`. Return true only for this original caller existential.
Each adoption checks current policy; earlier success grants no authority. Currentness remains caller-owned. No identity prerequisite, type, field, helper, schema store, detached input or later authority.
Preserve legacy construction/adoption/identity and all lower refusals. Exclude E1/E2 adoption, whole acceptance/status, currentness receipts, complete footprints/cache, publication/promotion, broader profiles and owner-gated scope.
Files: Proof status.rs, existing [algorithm tests](../../../../crates/mizar-test/src/runner/tests/algorithm.rs), paired Proof status/plan documents and this contract. Lower-stage production, spec/.miz/expectation/trace/audit coverage/owner/deferral and AGENTS/protocol remain unchanged.

## Test matrix and exit
Cap: four existing ensures fixture families; establish one genuine native success before expansion, without new shared fixtures. Reuse existing oracles and the [ensures return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); no changed test intent.
1. Genuine rename/trivia/equal-spelling/optional-terminating variants construct/adopt repeatedly under existing eligible current policies; retain whole association/actual Kernel oracles, input/prior-product purity, denied formats/schema refusal and valid recovery. Other formats cannot compensate. Legacy recheck/adoption/identity still refuse.
2. Existing complete request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error matrices preserve exact upstream errors before denied policy. Authentic changed inputs construct/adopt independently; old-holder cross-adoption and equal cloned Core return false before policy, while fresh holders adopt their own arena.
3. Existing unsupported/malformed clauses preserve exact constructor/authentication errors before policy; prior successes persist.
4. Existing no-clause/direct-return controls preserve upstream refusals and legacy successes. Native ensures adoption grants no identity, E1/E2, whole acceptance/status or later authority.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed adoption/authentication/policy matrices run; unaffected heavy evidence/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; native ensures adoption authenticates before mismatch before current policy and preserves every excluded authority boundary.
