# Task VC53-CALLER-ENSURES-E1-INHABITATION-CHECK: checked clause-bearing caller object inhabitation
Canonical language: English; [Japanese pointer](../ja/VC53-CALLER-ENSURES-E1-INHABITATION-CHECK.md).
Status: frozen checking-only slice. Tier: full; new public semantic-authority API fails pattern eligibility.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [ensures E1 request](VC53-CALLER-ENSURES-E1-INHABITATION-REQUEST.md), [legacy E1 checking](VC53-UNARY-CALLER-E1-INHABITATION-CHECK.md).
Authority: [§20.1.2, §20.2.6, §20.7.2, §20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [ensures E1 request](../../mizar-vc/en/generator.md#clause-bearing-unary-caller-e1-inhabitation-request), [Kernel introduction](../../mizar-kernel/en/checker.md#bounded-e1-inhabitation-witness-introduction), [runner](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; authenticated clause-bearing caller E1 request and Kernel introduction exist without atomic fresh checking. No language-intent gap or external prerequisite.

## Frozen design and boundaries
Add only `check_source_unary_child_call_caller_ensures_e1_inhabitation_result(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<((LocalContext, [VcGeneratedFormula; 2], CoreTermId), ObjectInhabitationCheck<'core>), String>`.
Call unchanged ensures-E1 request production once first on identical inputs; preserve upstream errors. Project its fresh declared binder, atomic object body and sole original caller guard. Call unchanged Kernel `check_object_inhabitation_result` once with that target, `request.actual` retained as the tuple witness, and the original guard as both child obligation and sole context. Return the entire unchanged tuple and same-call opaque result only on success; local refusal reuses `algorithms.unary_child.invalid_existential_request`.
Checking proves only `exists r. object(r)` on all typed inputs under the caller object-input guard, assuming neither returns nor requires/ensures. Inputs remain immutable and repeatable; equal cloned Core receives newly checked evidence bound to that clone.
Preserve upstream authentication order, exact errors, all legacy clause-bearing refusals and no-clause/direct-return behavior. No new type, field, helper, fixture, schema storage or parallel representation; no lower-stage changes.
Exclude Proof association/identity/policy/native adoption, E1 typing/single-valuedness, E2, whole acceptance/status, caller currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated work.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. No spec/.miz/expectation/trace, diagnostic, AGENTS or protocol edits. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing ensures fixture families; establish one genuine checked success before expanding, without new fixtures or helpers. Reuse existing oracles and the [ensures return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); no changed test intent.
1. Independent exact formula/witness oracle and E4 projection: unchanged whole request tuple, same-Core result, fresh declared binder, sole original caller guard and existing Kernel schema; input/prior-product purity and repetition. Genuine rename/trivia/equal-spelling/optional-terminating variants and equal cloned Core succeed with evidence in their own arena.
2. Existing complete request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error cases refuse atomically with exact upstream errors in authentication order. Authentic changed inputs succeed together; original recovers.
3. Existing unsupported/malformed clauses preserve exact upstream refusals; prior products remain intact.
4. Existing no-clause/direct-return controls preserve upstream refusals and legacy successes; old clause-bearing routes still refuse, with no later authority.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed checking/authentication matrices run; unaffected heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; only freshly authenticated caller object-inhabitation checking becomes available, preserving every excluded authority boundary.
