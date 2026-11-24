# Task VC53-CALLER-ENSURES-E4-CHECK: checked clause-bearing caller return
Canonical language: English; [Japanese pointer](../ja/VC53-CALLER-ENSURES-E4-CHECK.md).
Status: frozen local checking. Tier: full; new public API fails pattern eligibility.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md). Consumer: mizar-test.
Dependency: [clause-bearing E4 request](VC53-CALLER-ENSURES-E4-REQUEST.md).
Authority: [§20.1.2, §20.2.6, §20.7.2, §20.13.1 E4](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [E4 request](../../mizar-vc/en/generator.md#clause-bearing-unary-caller-e4-request), [Kernel child instantiation](../../mizar-kernel/en/checker.md#unary-child-existential-instantiation), [caller composition](../../mizar-kernel/en/checker.md#bounded-unary-caller-operational-composition).
Gap: source_drift/test_gap; the authenticated clause-bearing request lacks a local checked-return facade. No language-intent change or external prerequisite.

## Frozen design and boundaries
Add only `check_source_unary_child_call_caller_ensures_e4_result(source, typed, symbols, core, request)` in VC generator.rs, borrowing Core and returning existing `Result<((LocalContext, [VcGeneratedFormula; 2]), AlgorithmReturnsExistentialCheck), String>`.
Call the unchanged ensures E4 request producer once first with identical inputs; propagate authentication errors unchanged before any child/CFG work. Retain its exact descriptor and sole original caller object guard, fresh result binder and caller relation.
From authenticated Core derive the callee CFG flow and sole Return. Invoke unchanged Kernel `check_unary_child_returns_existential_result` using that callee target, the descriptor's definition-free actual/result pair, fresh result binder, request.actual witness and sole guard. Pass its genuine same-Core opaque child into unchanged `check_unary_caller_returns_existential_result` with the exact original caller target and guard.
Only both checks succeeding returns the unchanged descriptor and existing opaque caller result atomically. Local shape/join/Kernel refusal reuses `algorithms.unary_child.invalid_caller_e4_request`; upstream E2 authentication errors retain precedence. No supplied descriptor, witness, child, hash or schema is trusted.
Retain exact caller/flow/Return/Apply/callee/actual joins, actual witness, original guard and existing Kernel schema. Inputs stay immutable and repeatable; equal cloned Core obtains newly checked evidence in that arena.
Legacy descriptor/E1/E4/child/direct-return/Proof routes still refuse caller ensures. This route requires the existing exact clause and grants only local operational checking, with no Proof identity/policy/adoption, E1/E2 adoption, whole acceptance/status, currentness receipt, schema storage, complete footprints/cache, promotion or publication.
No new type, field, helper, fixture or representation; no lower-stage changes. Owner-gated work, broader profiles, spec/.miz/expectation/trace/diagnostic, AGENTS and protocol edits are forbidden. Audit coverage/owner/deferral unchanged.
Files: VC generator.rs, existing runner tests/algorithm.rs, paired VC generator/plan documents and this contract.

## Test matrix and exit
Cap: four existing fixture families; reuse genuine clause-bearing E2/E4 setup and pass one checked composition before expanding.
1. Independently inspected exact unchanged descriptor and opaque result: same Core pointer, caller CFG/Return/input/fresh result target, request.actual witness, sole original guard and existing schema; purity and repeatability.
2. Existing genuine rename/trivia/equal-spelling/optional-terminating variants and equal cloned Core succeed with newly checked evidence; results borrow their own arena and authentic changed sources succeed together.
3. Complete existing request/Core drift and calibrated source/typed/environment/forgery/combined-error matrices refuse atomically with exact upstream E2 errors; original recovers.
4. Existing unsupported/malformed/no-clause/direct-return controls retain exact upstream refusal, legacy clause-bearing E1/E4/descriptor/child/Proof refusals remain, and no-clause downstream controls still pass. No later authority follows.
Existing source oracle: [ensures return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz); existing runner caller-ensures and drift matrices. No changed test intent.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews required.
Run fmt, all-target/all-feature Clippy denied warnings, both lint_policy suites including links/fragments, Kernel/VC owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed checking/authentication matrices run; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; exact freshly authenticated checked caller return available with unchanged legacy boundaries and required gates passing.
