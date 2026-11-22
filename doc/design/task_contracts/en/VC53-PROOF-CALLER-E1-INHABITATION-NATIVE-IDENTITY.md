# Task VC53-PROOF-CALLER-E1-INHABITATION-NATIVE-IDENTITY: bounded native caller identity
Canonical language: English; [Japanese pointer](../ja/VC53-PROOF-CALLER-E1-INHABITATION-NATIVE-IDENTITY.md).
Status: frozen bounded identity. Tier: full; new public method fails pattern eligibility.
Owner: [Proof](../../mizar-proof/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [native adoption](VC53-PROOF-CALLER-E1-INHABITATION-NATIVE.md), [passive identity](VC53-PROOF-CALLER-E1-INHABITATION-IDENTITY.md), [E4 composition](VC53-PROOF-CALLER-E4-NATIVE-IDENTITY.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md), [identity model](../../architecture/en/22.incremental_verification_contract.md#identity-model-vcid-vs-obligationanchor).
Invariants: [native adoption](../../mizar-proof/en/status.md#native-caller-e1-inhabitation-adoption), [whole association](../../mizar-proof/en/status.md#arena-local-caller-e1-inhabitation-association).
Gap: source_drift/test_gap; authentic native adoption and passive identity exist without their composition. No language-intent gap or external prerequisite.

## Frozen scope
Add only `NativeUnaryCallerE1Inhabitation::obligation_identity(source, typed, symbols, core, request, policy)` returning `Result<Option<(ProofObligationIdentity, DependencySlice)>, String>`. FIRST call unchanged `adopt_original_request` once on all six inputs; propagate errors and map false to None. On true return the retained association's unchanged `obligation_identity` on identical five source inputs.
Preserve authentication before whole-request/tuple/exact-arena mismatch before current-policy refusal, `algorithm.proof.native_e1_policy_rejected`, E1 domains/anchor/hashes and whole raw slice. Reevaluate supplied policy each call; earlier success grants no authority. Currentness remains caller-owned.
No new producer, holder, field, helper, fixture or Kernel check. Exclude typing/single-valuedness checking, E2, whole acceptance/status/currentness, complete footprints/cache, promotion/publication, mixed-group/imported acceptance, broader profiles and owner-gated scope. Preserve original guard/binder/actual witness and E1 domain separation.
Files: Proof status.rs, existing runner tests/algorithm.rs, paired Proof status/plan documents and this contract. Lower-stage production and child/E4/direct-return APIs stay unchanged; Open/unknown/IncompleteUncacheable/cache-miss persist. No language intent, diagnostics, spec/.miz/expectation/trace/audit-status or AGENTS/protocol changes; audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: five existing genuine-fixture families; establish one native identity success before expansion, without a new shared fixture.
1. Independent canonical/full-slice oracle: native identity equals independently verified passive E1 identity and whole slice; repeated calls preserve inputs and retained products.
2. Normalization/owners/flags/clones: trivia/alpha/equal-spelling variants preserve identity; owner/terminating changes participate. Old holder returns None for equal cloned Core before denied policy; fresh clone holder returns equal identity only in its own arena.
3. Policy refusal/recovery: eligible policies return unchanged identity/slice; denied formats/Builtin/schema retain E1 policy error; previous success grants no authority and eligible recovery succeeds.
4. Complete drift/authentication/error-order matrices: reuse every existing request/Core mutation and calibrated source/typed/environment/forgery/combined error under denied policy. Authentication errors precede mismatch; authenticated cross-source mismatch returns None; fresh own-source holder succeeds.
5. Unchanged raw-state and child/E4/direct-return boundaries: passive identity stays policy-independent; Open/unknown/incomplete/cache-miss persist without later authority.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Kernel/Proof/VC owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed identity/policy/authentication matrices run; unchanged heavy evidence/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane has no full/push trigger.
Exit: native E1 object-inhabitation identity only after original caller adoption under current policy, preserving all excluded authority.
