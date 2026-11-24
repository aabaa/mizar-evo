# Task VC53-CALLER-ENSURES-E1-TYPING-REQUEST: clause-bearing caller result typing
Canonical language: English; [Japanese pointer](../ja/VC53-CALLER-ENSURES-E1-TYPING-REQUEST.md).
Status: frozen request-only producer. Tier: full; new public semantic-authority API fails pattern eligibility.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [E2 request](VC53-UNARY-CALLER-E2-REQUEST.md), [ensures inhabitation request](VC53-CALLER-ENSURES-E1-INHABITATION-REQUEST.md).
Authority: [§20.1.2, §20.2.6, §20.7.2, §20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [E2](../../mizar-vc/en/generator.md#bounded-unary-caller-e2-request), [four goals](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design), [runner](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; authenticated clause-bearing E2 exists without a result-typing request. No language-intent gap or external prerequisite.

## Frozen design and boundaries
Add only `generate_source_unary_child_call_caller_ensures_e1_typing_request(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<(LocalContext, [VcGeneratedFormula; 4]), String>`.
Call unchanged `generate_source_unary_child_call_caller_e2_request` once first on identical inputs. Preserve authentication errors, exact context, four rows, IDs, kinds, provenance and original binder/links; replace only row one's ProgramEquals shape with `ProgramTypePredicate { subject: original result, ty: object }`, extracting the original result from row zero's AlgorithmReturns. Impossible local shape refusal reuses `algorithms.unary_child.invalid_caller_e2_request`.
Under only the original caller object-input guard, request `forall r. returns_caller(actual,r) -> object(r)`. Returns stays an antecedent; neither requires/ensures, equality nor child returns supplies a premise or proof. Inputs and prior products remain immutable and repeatable; equal cloned Core regenerates the tuple.
Preserve legacy clause-bearing refusals and no-clause/direct-return behavior. No new type, field, helper, fixture, storage or parallel representation.
Exclude checking, Kernel/Proof authority, identity, native adoption, single-valuedness, whole acceptance/status, currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated work.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. No lower-stage, spec/.miz/expectation/trace, diagnostic, AGENTS or protocol edits. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing ensures fixture families; confirm one genuine setup success before expanding. Reuse existing oracles and the [ensures return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); no changed test intent.
1. Exact independent result-typing oracle and E2 projection: sole original guard, fresh original binder, caller relation, object result, implication/universal links, metadata/context retention, input/prior-product purity and repetition. Genuine rename/trivia/equal-spelling/optional-terminating variants and equal cloned Core regenerate the tuple.
2. Existing complete request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error cases retain exact E2 authentication errors and order. Authentic changed inputs succeed together; original recovers.
3. Existing unsupported/malformed clauses retain exact E2 refusals; prior products remain intact.
4. Existing no-clause/direct-return controls preserve upstream refusals and legacy successes; old clause-bearing routes still refuse. No checking or later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed request/authentication matrices run; unaffected heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; only the freshly authenticated unproved typing request becomes available, preserving all excluded authorities.
