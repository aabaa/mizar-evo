# Task VC53-UNARY-CALLER-E2-KERNEL: bounded caller postcondition checking
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CALLER-E2-KERNEL.md).
Status: frozen Kernel-only slice. Tier: full; new public semantic rule fails pattern eligibility.
Owner: [Kernel plan](../../mizar-kernel/en/00.crate_plan.md). Consumer: mizar-test; fresh VC checking remains separate.
Dependency: [authenticated caller E2 request](VC53-UNARY-CALLER-E2-REQUEST.md).
Authority: [§§20.1.2, 20.2.6, 20.4.1, 20.7.2, 20.13.1 E2](../../../spec/en/20.algorithm_and_verification.md).
Invariants: [caller typing validation](../../mizar-kernel/en/checker.md#bounded-unary-caller-e1-typing), [child joins](../../mizar-kernel/en/checker.md#unary-child-existential-instantiation), [E2 request](../../mizar-vc/en/generator.md#bounded-unary-child-request-descriptor).
Gap: source_drift/test_gap; the genuine E2 request has no local Kernel checker. Exact bodies fix the equality; no language-intent change or external prerequisite.

## Frozen design and boundaries
Add only boolean `check_unary_caller_return_postcondition(core, request, callee_flow, target, binder, conclusion, context)` in Kernel checker.rs, reusing existing relation tuples and an ordered equality pair.
Check `object(actual) |- forall r. (returns_caller(actual,r) -> r=actual)`. Require conclusion `[target.result, target.input]`, definition-free exact input/result, and fresh result binder under the sole original caller object guard.
Reuse unchanged caller typing validation for both exact sole bodies, original guards, request/slot/Apply joins, global freshness/cycle safety and caller CFG/Return matching. This reuses structural validation, not typing or existential evidence as a premise.
Every child return is its immutable formal instantiated by the actual; the caller returns exactly that child's result. Equality follows for every returned result without assuming returns, ensures, a function equation, E4 existence or terminating intent. The local rule checks this equality independently of retained contract clauses; it neither authenticates source nor interprets arbitrary ensures.
No new type, holder, field, helper, fixture, schema API or parallel representation. Preserve old descriptor/VC/Proof clause-bearing refusals, direct-return and E1/E4 behavior; lower-stage APIs remain unchanged.
Exclude VC checking facade, schema reporting/storage, evidence, Proof association/identity/policy/native adoption, whole acceptance/status, caller-owned currentness, complete footprints/cache, promotion/publication, broader profiles and owner-gated work. No spec/.miz/expectation/trace/diagnostic, AGENTS or protocol edits. Audit coverage/owner/deferral unchanged.
Files: Kernel checker.rs, existing runner tests/algorithm.rs, paired Kernel checker/plan documents and this contract.

## Test matrix and exit
Cap: four existing genuine-fixture families; pass one genuine clause-bearing Kernel success before expanding.
1. Existing independent four-row E2 oracle supplies exact caller target, binder, ordered equality and sole guard; repeated and cloned-Core checks succeed with input/product purity. Genuine renames/trivia/equal spellings and all optional terminating flags succeed.
2. Refuse each target algorithm/flow/Return/input/result/definition, callee flow, binder and ordered equality mismatch; missing/extra/swapped guards and occupied binders refuse. Calibrate every family with genuine success, then restore and recover.
3. Reuse existing complete request/Core drift matrices: invalid structural joins, both bodies, Apply, slots, guards, CFG/cycles refuse locally; nonstructural labels/source/contracts remain local successes where exact bodies still prove equality, while fresh VC authentication refuses drift.
4. Existing clause-bearing old-route refusals, no-clause downstream controls and direct-return/raw unknown/incomplete/cache-miss boundaries remain unchanged. Caller E2 refuses a direct-return target; success creates no later authority.
Require independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Kernel/VC/Proof owner groups, algorithm/source_algorithm regressions and short proof-verification E2E. Changed structural/equality matrices run immediately; unchanged heavy evidence/policy/cache/aggregate-acceptance/edit-versus-clean groups defer to Muse sealed integration. Full cargo test only on cadence; lane push:no has no full trigger.
Exit: freeze committed before implementation; only local bounded caller E2 equality becomes checkable, preserving every excluded authority and passing required gates.
