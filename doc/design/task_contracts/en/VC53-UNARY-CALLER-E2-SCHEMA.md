# Task VC53-UNARY-CALLER-E2-SCHEMA: caller postcondition schema handoff
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E2-SCHEMA.md).
Status: frozen bounded schema reporting. Tier: full; new public semantic-authority APIs fail pattern eligibility.
Owners: [Kernel](../../mizar-kernel/en/00.crate_plan.md), [VC](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [Kernel E2 rule](VC53-UNARY-CALLER-E2-KERNEL.md), [fresh E2 check](VC53-UNARY-CALLER-E2-CHECK.md).
Authority: [§§20.1.2, 20.2.6, 20.4.1, 20.7.2, 20.13.1 E2](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [Kernel caller validation](../../mizar-kernel/en/checker.md#bounded-unary-caller-e2-postcondition), [authenticated E2 request](../../mizar-vc/en/generator.md#bounded-unary-caller-e2-request), [dependency ownership](../../mizar-vc/en/dependency_slice.md).
Gap: source_drift/test_gap; local E2 and authenticated checking exist without success-only schema reporting. Existing specification fixes the meaning; no language-intent change or external prerequisite.

## Frozen design and boundaries
Add Kernel `check_unary_caller_return_postcondition_schema` with the identical seven inputs as the unchanged boolean rule. Invoke that rule once; return Kernel-owned `Some(1_u16)` only on true, otherwise `None`. The scalar reports the checked rule schema without a new type, field, holder, helper or parallel representation.
Add VC `check_source_unary_child_call_caller_e2_request_with_schema(source, typed, symbols, core, request)` returning `Result<((LocalContext, [VcGeneratedFormula; 4]), u16), String>`. Invoke the unchanged authenticated E2 producer once on identical inputs, preserving every upstream error. Project its exact original guard, declared fresh binder, caller relation and ordered equality; rebuild the callee flow from the authenticated Core. Invoke the Kernel schema API once; return the whole unchanged tuple and its actual same-call schema only on success. Preserve `algorithms.unary_child.invalid_caller_e2_request` for local refusal. Never stamp or accept an externally requested schema.
Keep the old VC checking signature and behavior through delegation to this API, discarding only the scalar. No duplicate authentication or Kernel check. Lower-stage APIs, existing clause-bearing refusals and no-clause routes remain unchanged.
The proposition remains only `object(actual) |- forall r.(returns_caller(actual,r) -> r=actual)`. Contract clauses, child/E4 evidence and terminating intent supply no premise. No schema storage, Proof association/identity/policy/native authority, whole acceptance/status/currentness, complete footprints/cache, promotion/publication, broader profiles or owner-gated scope.
No spec/.miz/expectation/trace/diagnostic, AGENTS or protocol edits. Audit coverage/owner/deferral unchanged.
Files: existing Kernel checker.rs, VC generator.rs, runner/tests/algorithm.rs and paired owner checker/generator/plan/source-inventory documents, plus this contract. Lower-stage ownership remains unchanged.

## Test matrix and exit
Cap: four existing genuine-fixture families; confirm one successful independent four-row setup before expanding, with no new shared fixture.
1. Existing independent E2 oracle: Kernel boolean/schema success parity (`Some(1)`), VC handoff equals the whole unchanged tuple and actual same-call Kernel schema; old checking parity, repeat determinism, cloned Core and input/product purity.
2. Genuine renames/trivia/equal spellings and optional terminating flags retain exact joins and successful tuple/schema handoff without external evidence.
3. Extend complete request/Core and Kernel target/binder/equality/guard/flow drift controls with boolean/schema refusal parity. Calibrated source/typed/environment/forgery/combined errors return exact upstream producer errors; authentic changed source succeeds with its own inputs; original recovers.
4. Unsupported/no-clause/direct-return inputs refuse; old descriptor/E4/E1/child/direct-return/Proof boundaries and no-clause downstream controls remain unchanged. Local success leaves obligation/proof/diagnostic tables unchanged and supplies no later authority.
Existing oracle: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), existing runner unary caller ensures/drift families. No test-intent changes.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Kernel/VC/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed schema/equality/authentication matrices run; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; success-only caller E2 schema reporting and authenticated same-call tuple/schema handoff preserve every excluded authority.
