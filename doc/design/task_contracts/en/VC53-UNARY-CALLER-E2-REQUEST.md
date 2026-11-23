# Task VC53-UNARY-CALLER-E2-REQUEST: bounded caller postcondition request
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E2-REQUEST.md).
Status: frozen request-only producer. Tier: full; new public API fails pattern eligibility.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependencies: [authenticated clause handoff](VC53-CORE46-UNARY-CALLER-ENSURES-HANDOFF.md), [VC53 boundary](ALGDEF-VC53-NORMAL-RETURN.md).
Authority: [§20.1.2, §20.4, §20.7.2, §20.13.1 E2](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [unary authentication](../../mizar-vc/en/generator.md#bounded-unary-child-request-descriptor), [Core lowering](../../mizar-core/en/elaborator.md#static-source-algorithm-lowering), [separate goals](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design).
Gap: source_drift/test_gap; the genuine caller equality is retained without an E2 request. Existing specification fixes intent; no external prerequisite or language-intent change.

## Frozen design and boundaries
Add only `generate_source_unary_child_call_caller_e2_request(source, typed, symbols, core, request)` in VC generator.rs, returning existing `Result<(LocalContext, [VcGeneratedFormula; 4]), String>`.
Freshly call unchanged Checker unary authentication and Core lowering on identical inputs before comparing the entire regenerated Core/request. Preserve upstream errors; mismatch or missing exact caller clause refuses atomically with `algorithms.unary_child.invalid_caller_e2_request`.
Require the authenticated single caller `ensures result = actual` equality. Retain only its original caller object-input guard as a CheckerFact and a fresh result BinderDeclaration above all authenticated parameter/result and Var IDs (checked addition); policy inputs stay empty.
Rows zero through three are AlgorithmReturns for the exact caller/flow/Return/actual and fresh result, ProgramEquals of fresh result and actual, Implies from returns to equality, and Forall over the result declaration. IDs are row indices, all rows are SplitGoal with empty provenance. Values have no definition annotation.
This requests only `object(actual) |- forall r. (returns_caller(actual,r) -> r=actual)`. Neither returns, ensures, child equality, function equation nor terminating intent is installed as a premise or proof. No VcSet, seed, evidence, checked result or acceptance credit is created. Inputs remain immutable and repeatable; equal cloned Core regenerates the tuple.
Old descriptor/E4/E1/child/direct-return/Proof routes remain unchanged and refuse clause-bearing pairs. No new type, field, holder, helper, fixture or parallel representation; no lower-stage changes.
Exclude checking, Kernel/Proof authority, identity, policy/native adoption, whole acceptance/status, caller-owned currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated work. No spec/.miz/expectation/trace, diagnostic, AGENTS or protocol edits. Audit coverage/owner/deferral unchanged.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract.

## Test matrix and exit
Cap: four existing genuine-fixture families; establish one clause-bearing fixture and pass one successful E2 setup before expanding.
1. Independent exact formula oracle: sole original object guard, fresh declared binder, exact caller relation, ordered equality/implication/universal links, row metadata, empty policy inputs; repetition and input/product purity.
2. Genuine renames/trivia/equal spellings and optional terminating flags preserve joins; equal cloned Core produces the same tuple.
3. Complete request/Core drift plus calibrated source/typed/environment and combined errors refuse atomically in authentication order; authentic changed source succeeds only with its own inputs; original recovers. Unsupported/no-clause inputs refuse. Reuse existing malformed-clause Checker controls.
4. Clause-bearing old descriptor/E4/E1/child/direct-return/Proof refusals and no-clause downstream controls stay valid; empty obligation/proof/diagnostic tables and no later credit.
Existing oracles: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), existing runner unary caller ensures and drift matrices. No test-intent or expectation changes.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews required.
Run fmt, all-target/all-feature Clippy denied warnings, both lint_policy suites including links/fragments, VC owner group, algorithm/source_algorithm regressions and short proof-verification E2E. Changed request/authentication matrices run; unaffected heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; exact freshly authenticated unproved E2 request available, old credit boundaries preserved, required gates pass.
