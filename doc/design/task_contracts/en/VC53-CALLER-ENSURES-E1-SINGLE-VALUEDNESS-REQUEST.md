# Task VC53-CALLER-ENSURES-E1-SINGLE-VALUEDNESS-REQUEST: clause-bearing caller uniqueness
Canonical language: English; [Japanese pointer](../ja/VC53-CALLER-ENSURES-E1-SINGLE-VALUEDNESS-REQUEST.md).
Status: frozen request-only producer. Tier: full; new public semantic API fails pattern eligibility.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [ensures E4 request](VC53-CALLER-ENSURES-E4-REQUEST.md), [caller uniqueness request](VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-REQUEST.md).
Authority: [§20.1.2, §20.2.6, §20.7.2, §20.13.1 E1/E3](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [ensures E4](../../mizar-vc/en/generator.md#clause-bearing-unary-caller-e4-request), [four goals](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design), [runner](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; authenticated clause-bearing E4 exists without a caller single-valuedness request. Existing specification fixes its meaning; no language-intent gap or external prerequisite.

## Frozen design and boundaries
Add only `generate_source_unary_child_call_caller_ensures_e1_single_valuedness_request(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<(LocalContext, [VcGeneratedFormula; 6]), String>`.
Call unchanged `generate_source_unary_child_call_caller_ensures_e4_request` once first on identical inputs, preserving whole fresh authentication and upstream errors. Retain its exact context prefix/policy inputs and row zero caller relation. Append a result BinderDeclaration at the next context ID with a distinct sort key and variable one greater than the original fresh result (checked addition); retain the original binder first. Reuse row one's metadata for a second returns atom, changing only the result to the new binder. Rows two through five are And of both returns, ProgramEquals of ordered results, Implies from And to equality, and Forall over both ordered declarations. IDs are row indices; appended rows are SplitGoal with empty provenance. Local shape/overflow/context refusal reuses `algorithms.unary_child.invalid_caller_e4_request`.
Under only the original caller object-input guard, request `forall r,s. (returns_caller(actual,r) & returns_caller(actual,s)) -> r=s`. Both relations share the exact caller/flow/Return/actual association. Returns remain antecedents; no E1/E3, requires/ensures, child return or function equation supplies a premise or proof. Inputs/prior products remain immutable and repeatable; equal cloned Core regenerates the tuple.
Preserve legacy clause-bearing refusals and no-clause/direct-return behavior. No new type, field, helper, fixture, storage or parallel representation.
Exclude checking, Kernel/Proof authority, identity, policy/native adoption, whole acceptance/status, currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated work.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. No lower-stage, spec/.miz/expectation/trace, diagnostic, AGENTS or protocol edits. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing ensures fixture families; confirm one genuine setup success before expanding. Reuse existing oracles and the [ensures return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); no changed test intent.
1. Independent exact formula oracle and E4 projection: sole original guard, two distinct fresh declared binders, exact shared caller relation/actual input, ordered And/equality/implication/universal links, metadata/context retention, input/prior-product purity and repetition. Genuine rename/trivia/equal-spelling/optional-terminating variants and equal cloned Core regenerate the tuple.
2. Existing complete request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error cases retain exact E4 authentication errors and order. Authentic changed inputs succeed together; original recovers.
3. Existing unsupported/malformed clauses retain exact E4 refusals; prior products remain intact.
4. Existing no-clause/direct-return controls preserve upstream refusals and legacy successes; old clause-bearing routes still refuse. No checking or later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed request/authentication matrices run; unaffected heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; only the freshly authenticated unproved uniqueness request becomes available, preserving all excluded authorities.
