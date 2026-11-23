# Task VC53-UNARY-CALLER-E2-CHECK: fresh caller postcondition checking
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E2-CHECK.md).
Status: frozen checking-only slice. Tier: full; new public semantic-authority API fails pattern eligibility.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [fresh E2 request](VC53-UNARY-CALLER-E2-REQUEST.md), [Kernel E2 rule](VC53-UNARY-CALLER-E2-KERNEL.md).
Authority: [§§20.1.2, 20.2.6, 20.4.1, 20.7.2, 20.13.1 E2](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [E2 request](../../mizar-vc/en/generator.md#bounded-unary-caller-e2-request), [Kernel caller validation](../../mizar-kernel/en/checker.md#bounded-unary-caller-e2-postcondition), [dependency ownership](../../mizar-vc/en/dependency_slice.md).
Gap: source_drift/test_gap; fresh E2 generation and local Kernel checking exist without their VC join. Existing specification fixes the meaning; no language-intent change or external prerequisite.

## Frozen design and boundaries
Add only `check_source_unary_child_call_caller_e2_request(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<(LocalContext, [VcGeneratedFormula; 4]), String>`.
First call unchanged `generate_source_unary_child_call_caller_e2_request` once on identical inputs, preserving whole fresh authentication and upstream errors. Project its exact original guard, fresh declared binder, caller relation and ordered equality. Rebuild the authenticated Core CFG for the callee flow; invoke unchanged Kernel `check_unary_caller_return_postcondition` once with those projections and the same Core/request. Return the entire unchanged tuple only on true; impossible local shape/flow or Kernel refusal uses `algorithms.unary_child.invalid_caller_e2_request`.
Only `object(actual) |- forall r. (returns_caller(actual,r) -> r=actual)` is checked. Returns, ensures, child equality, function equation and terminating intent supply no premise or proof.
No new type, field, holder, helper, fixture, schema reporting/storage or parallel representation. Lower-stage APIs, existing clause-bearing refusals and no-clause routes remain unchanged.
Exclude whole acceptance/status, caller-owned currentness, complete footprints/cache, promotion/publication, later Proof association/identity/policy/native authority, broader profiles and owner-gated work. No spec/.miz/expectation/trace/diagnostic, AGENTS or protocol edits. Audit coverage/owner/deferral unchanged.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one genuine successful fresh check before expanding.
1. Existing independent four-row E2 oracle and Kernel projections: fresh check returns the exact whole producer tuple; repetition and cloned Core preserve inputs/products, sole guard, fresh binder and ordered equality without external evidence.
2. Genuine renames/trivia/equal spellings and optional terminating flags succeed with exact joins.
3. Existing complete request/Core drift plus source/typed/environment/forgery/combined errors refuse atomically with exact producer errors; authentic changed source succeeds only with its own inputs; original recovers. Existing Kernel target/binder/equality/guard/flow controls remain effective.
4. Unsupported/no-clause/direct-return inputs refuse; old clause-bearing descriptor/E4/E1/child/direct-return/Proof refusals and no-clause downstream controls remain valid. Success leaves obligation/proof/diagnostic tables unchanged and grants no later authority.
Existing oracle: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), existing runner unary caller ensures/drift families. No test-intent changes.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, VC/Kernel/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed checking/authentication matrices run immediately; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; only fresh bounded E2 checking becomes available, preserving every excluded authority.
