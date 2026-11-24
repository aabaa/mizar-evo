# Task VC53-CALLER-ENSURES-E1-INHABITATION-REQUEST: clause-bearing caller object inhabitation
Canonical language: English; [Japanese pointer](../ja/VC53-CALLER-ENSURES-E1-INHABITATION-REQUEST.md).
Status: frozen request-only producer. Tier: full; new public API fails pattern eligibility.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [ensures E4 request](VC53-CALLER-ENSURES-E4-REQUEST.md), [legacy E1 request](VC53-UNARY-CALLER-E1-INHABITATION-REQUEST.md).
Authority: [§20.1.2, §20.2.6, §20.7.2, §20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [ensures E4](../../mizar-vc/en/generator.md#clause-bearing-unary-caller-e4-request), [legacy E1](../../mizar-vc/en/generator.md#bounded-unary-caller-e1-inhabitation-request), [runner](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; authenticated clause-bearing caller E4 exists without an explicit E1 inhabitation request. No language-intent gap or external prerequisite.

## Frozen design and boundaries
Add only `generate_source_unary_child_call_caller_ensures_e1_inhabitation_request(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<(LocalContext, [VcGeneratedFormula; 2], CoreTermId), String>`.
Call unchanged ensures-E4 request production once first on identical inputs, preserving authentication and upstream errors. Extract the fresh result from row zero's AlgorithmReturns shape; replace only that shape with `ProgramTypePredicate { subject: result, ty: object }`. Preserve the exact context, Exists row, IDs, kinds and provenance; return `request.actual` as the actual-input witness. Impossible local shape refusal reuses `algorithms.unary_child.invalid_caller_e2_request`.
The request is `object(actual) |- exists r. object(r)` on all typed inputs. It assumes neither returns nor requires/ensures; the witness is neither Apply, callee formal nor caller result slot. Inputs remain immutable and repeatable; equal cloned Core regenerates the tuple.
Preserve legacy clause-bearing refusals and no-clause/direct-return behavior. No type, field, helper, fixture, storage or parallel representation.
Exclude checking, Kernel/Proof evidence, identity, native adoption, E1 typing/single-valuedness, E2, whole acceptance/status, caller-owned currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated work.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. No lower-stage, spec/.miz/expectation/trace, diagnostic, AGENTS or protocol edits. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing ensures fixture families; establish one genuine success before expanding, without new fixtures or helpers. Reuse existing oracles and the [ensures return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); no changed test intent.
1. Independent exact object-inhabitation/witness oracle plus exact E4 projection, context/Exists/metadata preservation, input/prior-product purity and repetition; genuine rename/trivia/equal-spelling/optional-terminating variants and equal cloned Core regenerate the tuple.
2. Existing complete request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error cases preserve exact E4 errors in authentication order. Authentic changed inputs succeed together, original recovers.
3. Existing unsupported/malformed clauses preserve exact upstream refusals; prior products remain intact.
4. Existing no-clause/direct-return controls preserve upstream refusals and legacy successes; old clause-bearing routes still refuse, and no checking or later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed request/authentication matrices run; unaffected heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; only the freshly authenticated unproved inhabitation request becomes available, preserving every excluded authority boundary.
