# Task VC53-CALLER-ENSURES-E1-TYPING-CHECK: checked clause-bearing caller result typing
Canonical language: English; [Japanese pointer](../ja/VC53-CALLER-ENSURES-E1-TYPING-CHECK.md).
Status: frozen checking-only slice. Tier: full; new public semantic-authority API fails pattern eligibility.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [ensures typing request](VC53-CALLER-ENSURES-E1-TYPING-REQUEST.md), [Kernel typing rule](VC53-UNARY-CALLER-E1-TYPING-KERNEL.md).
Authority: [§20.1.2, §20.2.6, §20.7.2, §20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [typing request](../../mizar-vc/en/generator.md#clause-bearing-unary-caller-e1-typing-request), [Kernel typing](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-typing), [runner](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; authenticated clause-bearing typing requests and Kernel typing exist without their fresh VC checking join. No language-intent gap or external prerequisite.

## Frozen design and boundaries
Add only `check_source_unary_child_call_caller_ensures_e1_typing_request(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<(LocalContext, [VcGeneratedFormula; 4]), String>`.
Call unchanged `generate_source_unary_child_call_caller_ensures_e1_typing_request` once first on identical inputs, preserving upstream errors. Project its original guard, fresh declared binder, exact caller target and atomic object conclusion; rebuild the authenticated Core CFG to obtain the callee flow. Pass those projections and identical request/Core to unchanged Kernel `check_unary_caller_return_typing` once. Return the exact whole tuple only on true; impossible local shape/flow or Kernel refusal reuses `algorithms.unary_child.invalid_caller_e4_request` locally.
This checks only `forall r. returns_caller(actual,r) -> object(r)` under the sole original caller object-input guard. Returns remains an antecedent; requires/ensures, equality and child evidence supply no context premise. Inputs and prior products remain immutable and repeatable; equal cloned Core is checked independently.
Preserve upstream authentication order, exact errors, legacy clause-bearing refusals and no-clause/direct-return behavior. No new type, field, helper, fixture, schema storage or parallel representation; no lower-stage changes.
Exclude Proof and later authority, single-valuedness, E2, whole acceptance/status, currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated work.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. No spec/.miz/expectation/trace, diagnostic, AGENTS or protocol edits. Audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing ensures fixture families; confirm one genuine checked setup before expanding, without new fixtures or helpers. Reuse existing independent typing/projection oracles and the [ensures return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); no changed test intent.
1. Fresh check returns the exact whole producer tuple under the original sole guard, retaining binder/target/object conclusion, metadata and links; input/prior-product purity, repetition, genuine rename/trivia/equal-spelling/optional-terminating variants and equal cloned Core succeed independently.
2. Existing complete request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error cases refuse atomically with exact producer authentication errors and order. Authentic changed inputs succeed together; original recovers. Existing Kernel projection/binder/conclusion/context/flow controls remain effective.
3. Existing unsupported/malformed clauses retain exact producer refusals and prior products remain intact.
4. Existing no-clause/direct-return controls preserve upstream refusals and legacy successes; old clause-bearing routes still refuse. No later authority follows.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed checking/authentication/typing matrices run; unaffected heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; only fresh atomic clause-bearing caller result typing checking becomes available, preserving every excluded authority boundary.
