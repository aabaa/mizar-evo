# Task VC53-CALLER-ENSURES-E1-SINGLE-VALUEDNESS-SCHEMA: clause-bearing caller uniqueness schema
Canonical language: English; [Japanese pointer](../ja/VC53-CALLER-ENSURES-E1-SINGLE-VALUEDNESS-SCHEMA.md).
Status: frozen schema-reporting slice. Tier: full; new public semantic-authority API fails pattern eligibility.
Owner: [VC](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [ensures uniqueness check](VC53-CALLER-ENSURES-E1-SINGLE-VALUEDNESS-CHECK.md), [existing Kernel schema](VC53-UNARY-CALLER-E1-SINGLE-VALUEDNESS-SCHEMA.md), [ensures association](VC53-PROOF-CALLER-ENSURES-E1-SINGLE-VALUEDNESS-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.4.1, 20.7.2, 20.13.1 E1/E3](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [VC uniqueness](../../mizar-vc/en/generator.md#clause-bearing-unary-caller-e1-single-valuedness-request), [Kernel uniqueness](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-single-valuedness), [runner/test design](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; authenticated ensures uniqueness checking exists without same-call schema reporting. No language-intent gap or external prerequisite.

## Frozen design and boundaries
Add only VC `check_source_unary_child_call_caller_ensures_e1_single_valuedness_request_with_schema(source, typed, symbols, core, request) -> Result<((LocalContext, [VcGeneratedFormula; 6]), u16), String>`.
Call unchanged ensures uniqueness producer once first on identical inputs; preserve every upstream error. Reuse existing projections of both exact caller targets, both original declared binders, ordered definition-free equality, sole original Core guard and authenticated rebuilt callee flow. Call existing Kernel `check_unary_caller_return_single_valuedness_schema` once; return the unchanged whole tuple with its actual same-call schema only on success. Local refusal retains `algorithms.unary_child.invalid_caller_e4_request`.
Existing checker delegates and discards only the scalar, preserving signature, errors and refusals. No duplicate authentication/check, externally requested or stamped schema, new type, field, helper, fixture, storage or parallel representation.
Only `forall r,s. (returns_caller(actual,r) & returns_caller(actual,s)) -> r=s` under the original object-input guard is checked. Returns remain antecedents; typing, E1/E3, requires/ensures, child/E4 evidence and function equality add no context premise or proof. Preserve repeatability, purity, independent cloned-Core checks and legacy refusals.
Exclude Kernel/Proof production changes, schema association, identity/policy/native authority, E2/E4 authority, whole acceptance/status/currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated work. Lower stages and raw unknown/incomplete/cache-miss states remain unchanged.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. No spec/.miz/expectation/trace, diagnostic, audit-status, AGENTS or protocol edits; audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing ensures fixture families; confirm one genuine successful schema setup before expanding, without new fixtures/helpers. Reuse independent uniqueness/projection oracles and the [ensures return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); preserve test intent.
1. Exact whole producer tuple plus actual Kernel schema; old-checker parity, repetition, input/product purity, original guard, ordered binders/equality, targets, metadata/links and independent cloned-Core success without external child/E4 or typing evidence.
2. Genuine rename/trivia/equal-spelling/optional-terminating variants and authentic changed inputs retain tuple/schema success; no later authority follows.
3. Complete existing request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error cases preserve exact producer errors before local refusal; originals recover. Existing Kernel target/binder/equality/context/flow controls remain effective.
4. Unsupported/malformed clauses and no-clause/direct-return controls preserve producer refusals and legacy successes; old clause-bearing identity/policy/native and lower-stage refusals remain.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites with links/fragments, Kernel/VC/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed schema/uniqueness/authentication matrices run; unaffected heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; only fresh clause-bearing tuple/schema handoff becomes available, preserving all excluded authority.
