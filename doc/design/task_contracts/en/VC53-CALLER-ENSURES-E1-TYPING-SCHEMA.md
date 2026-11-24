# Task VC53-CALLER-ENSURES-E1-TYPING-SCHEMA: clause-bearing caller typing schema
Canonical language: English; [Japanese pointer](../ja/VC53-CALLER-ENSURES-E1-TYPING-SCHEMA.md).
Status: frozen schema-reporting slice. Tier: full; new public semantic-authority API fails pattern eligibility.
Owner: [VC](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [ensures typing check](VC53-CALLER-ENSURES-E1-TYPING-CHECK.md), [existing Kernel schema](VC53-UNARY-CALLER-E1-TYPING-SCHEMA.md), [ensures association](VC53-PROOF-CALLER-ENSURES-E1-TYPING-ASSOCIATION.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1 E1](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [VC typing](../../mizar-vc/en/generator.md#clause-bearing-unary-caller-e1-typing-request), [Kernel typing](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-typing), [runner](../../mizar-test/en/harness.md#algorithm--logic).
Gap: source_drift/test_gap; authenticated ensures typing checking exists without same-call schema reporting. No language-intent gap or external prerequisite.

## Frozen design and boundaries
Add only VC `check_source_unary_child_call_caller_ensures_e1_typing_request_with_schema(source, typed, symbols, core, request) -> Result<((LocalContext, [VcGeneratedFormula; 4]), u16), String>`.
Call unchanged ensures typing producer once first on identical inputs; preserve every upstream error. Reuse existing projections of sole original guard, fresh declared binder, exact caller target, atomic object conclusion and authenticated rebuilt callee flow. Call existing Kernel `check_unary_caller_return_typing_schema` once; return the unchanged whole tuple with its actual same-call schema only on success. Local refusal retains `algorithms.unary_child.invalid_caller_e4_request`.
Existing checker delegates and discards only the scalar, preserving signature, errors and refusals. No duplicate authentication/check, externally requested or stamped schema, new type, field, helper, fixture, storage or parallel representation.
Only `forall r. returns_caller(actual,r) -> object(r)` under the original object-input guard is checked. Returns remains an antecedent; requires/ensures/equality and child evidence add no context premise. Preserve repeatability, purity, independent cloned-Core checks and legacy refusals.
Exclude Kernel/Proof production changes, schema association, policy/native adoption, single-valuedness, E2/E4 authority, whole acceptance/status/currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated work. Lower stages and raw unknown/incomplete/cache-miss states remain unchanged.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract. No spec/.miz/expectation/trace, diagnostic, audit-status, AGENTS or protocol edits; audit coverage/owner/deferral unchanged.

## Test matrix and exit
Cap: four existing ensures fixture families; confirm one genuine successful schema setup before expanding, without new fixtures/helpers. Reuse independent typing/projection oracles and the [ensures return fixture](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); preserve test intent.
1. Exact whole producer tuple plus actual Kernel schema; old-checker parity, repetition, input/product purity, original guard/binder/target/object/metadata/links and independent cloned-Core success.
2. Genuine rename/trivia/equal-spelling/optional-terminating variants and authentic changed inputs retain tuple/schema success; no later authority follows.
3. Complete existing request/Core drift and calibrated foreign/stale/forged source/typed/environment/combined-error cases preserve exact producer errors before local refusal; originals recover. Existing Kernel projection/binder/conclusion/context/flow controls remain effective.
4. Unsupported/malformed clauses and no-clause/direct-return controls preserve producer refusals and legacy successes; old clause-bearing identity/policy/native and lower-stage refusals remain.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites with links/fragments, Kernel/VC/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed schema/typing/authentication matrices run; unaffected heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; only fresh clause-bearing tuple/schema handoff becomes available, preserving all excluded authority.
