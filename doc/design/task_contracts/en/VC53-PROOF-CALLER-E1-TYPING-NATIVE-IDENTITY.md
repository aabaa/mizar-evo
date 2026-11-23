# Task VC53-PROOF-CALLER-E1-TYPING-NATIVE-IDENTITY: bounded native typing identity
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E1-TYPING-NATIVE-IDENTITY.md).
Status: frozen bounded identity. Tier: full; new public method fails the pattern-tier API criterion.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [native adoption](VC53-PROOF-CALLER-E1-TYPING-NATIVE.md), [passive identity](VC53-PROOF-CALLER-E1-TYPING-IDENTITY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md), [identity model](../../architecture/en/22.incremental_verification_contract.md#identity-model-vcid-vs-obligationanchor).
Invariants: [native adoption](../../mizar-proof/en/status.md#native-caller-e1-typing-adoption), [whole association](../../mizar-proof/en/status.md#arena-local-caller-e1-typing-association), [runner/test design](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; authenticated native adoption and passive identity exist without their composition. No language-intent gap or external prerequisite.

## Frozen scope
Add only `NativeUnaryCallerE1Typing::obligation_identity(source, typed, symbols, core, request, policy)` returning `Result<Option<(ProofObligationIdentity, DependencySlice)>, String>`. FIRST call unchanged `adopt_original_request` once on all six inputs; propagate errors and map false to None. On true return the retained association's unchanged `obligation_identity` on identical five source inputs.
Preserve authentication before whole-request/tuple/exact-arena mismatch before current-policy refusal, `algorithm.proof.unary_caller_e1_typing_policy_rejected`, typing domains/anchor/hashes and whole raw slice. Reevaluate supplied policy each call; earlier success grants no authority. Currentness remains caller-owned.
No new producer, holder, field, helper, fixture or extra Kernel check beyond adoption. Exclude single-valuedness, E2, whole acceptance/status/currentness receipts, complete footprints/cache, promotion/publication, broader profiles and owner-gated scope. Preserve child/E4/inhabitation/direct-return and raw Open/unknown/IncompleteUncacheable/cache-miss boundaries.
Files: Proof status.rs, existing runner tests/algorithm.rs, paired Proof status/plan documents and this contract. Lower-stage production and audit coverage/owner/deferral unchanged. No language intent, diagnostics, spec/.miz/expectation/trace/audit-status or AGENTS/protocol changes.

## Test matrix and exit
Cap: four existing genuine-fixture families; establish one native identity success before expansion, without a new shared fixture.
1. Canonical/full-slice oracle: native identity equals independently verified passive typing identity and whole slice; repeat calls, normalization/owner/terminating variants and recovery preserve inputs and retained products.
2. Policy: default, builtin-only, development, interactive and changed profile/policy-schema return unchanged identity/slice. Empty/FormulaSubstitution-only formats and wrong checker schemas preserve the typing policy error; prior success cannot bypass refusal and eligible recovery succeeds. Passive identity remains policy-independent.
3. Complete existing request/Core drift and clone matrix: upstream errors persist; authenticated whole-request/tuple or equal cloned Core mismatch returns None before denied policy. Fresh own-arena holder returns the corresponding passive identity; cross-arena equality grants no authority.
4. Calibrated foreign/stale/forged source/typed/environment, combined errors and unsupported direct-return preserve upstream errors before denied policy. Authenticated cross-source mismatch returns None; fresh own-source holder succeeds. Existing raw-state and excluded-authority controls remain unchanged.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Kernel/Proof/VC owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed identity/policy/authentication matrices run; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: native caller E1 typing identity only after original caller adoption under fresh current policy, preserving all excluded authority.
