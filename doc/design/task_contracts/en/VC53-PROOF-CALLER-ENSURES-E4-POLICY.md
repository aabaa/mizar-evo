# Task VC53-PROOF-CALLER-ENSURES-E4-POLICY: clause-bearing caller current-policy consumption
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-ENSURES-E4-POLICY.md).
Status: frozen bounded consumer. Tier: full; new public API fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [ensures association](VC53-PROOF-CALLER-ENSURES-E4-ASSOCIATION.md), [legacy policy](VC53-PROOF-CALLER-E4-POLICY.md).
Authority: [§20.1.2, §20.2.6, §20.4.1, §20.7.2, §20.13.1 E4](../../../spec/en/20.algorithm_and_verification.md), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Invariants: [association](../../mizar-proof/en/status.md#arena-local-caller-e4-association), [policy rule](../../mizar-proof/en/status.md#caller-e4-current-policy-consumption), [runner](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; checked clause-bearing association lacks an explicit current-policy consumer. No language-intent gap or external prerequisite.

## Frozen scope and design
Add only `consume_source_unary_caller_ensures_e4_existential(source, typed, symbols, core, request, policy)` in Proof status.rs returning existing `(SourceUnaryCallerE4Association, PolicyFingerprint)`.
FIRST call unchanged `SourceUnaryCallerE4Association::from_source_with_ensures` exactly once on identical inputs, preserving upstream errors. Inline require enabled `BuiltinKernelEvidence` and exact policy checker-schema equality with that association's actual checked result; otherwise return existing `algorithm.proof.unary_caller_e4_policy_rejected`. Return the whole association and current `policy.policy_fingerprint()` atomically.
Each invocation checks its supplied current policy; earlier products grant no authority. No identity prerequisite, type, field, helper, schema store, detached input or parallel representation. Currentness remains caller-owned.
Preserve legacy clause refusals, identity, child/direct-return consumers and native adoption unchanged. Exclude E1/E2 adoption, whole acceptance/status, currentness receipts, complete footprints/cache, publication/promotion, broader profiles and owner-gated scope.
Files: Proof status.rs, existing [algorithm tests](../../../../crates/mizar-test/src/runner/tests/algorithm.rs), paired Proof status/plan documents and this contract. Lower-stage production and spec/.miz/expectation/trace/audit coverage/owner/deferral remain unchanged; no AGENTS/protocol edits.

## Test matrix and exit
Cap: four existing ensures fixture families; establish one genuine success before expansion, without new shared fixtures. Reuse existing policy matrices and the [ensures return oracle](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); no changed test intent.
1. Genuine rename/trivia/equal-spelling/optional-terminating variants: compare whole request/descriptor, exact Core, actual witness, guard/binder and schema with existing association/VC/Kernel oracles. Repeatability and input/prior-product purity hold; legacy recheck/identity refuse.
2. Current default, builtin-only, development, interactive, changed profile/policy-schema return exact current fingerprints. Empty/formula-only formats and wrong checker schema refuse; other formats cannot compensate. Prior products persist and later valid policy recovers.
3. Existing complete request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error matrices preserve exact constructor errors even under denied policy. Equal cloned Core consumes freshly but cannot match old association; authentic changed inputs consume independently.
4. Existing unsupported/malformed/no-clause/direct-return controls retain upstream refusals before policy. Legacy clause-bearing descriptor/E1/E4/child identity/policy/native refusals and no-clause controls remain; no later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Proof/VC/Kernel owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed policy/authentication/association matrices run; unaffected heavy evidence/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; explicit current-policy consumption returns the whole checked ensures association while preserving every legacy authority boundary.
