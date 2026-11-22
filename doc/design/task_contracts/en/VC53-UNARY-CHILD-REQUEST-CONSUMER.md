# Task VC53-UNARY-CHILD-REQUEST-CONSUMER: bounded unary child-request consumer
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CHILD-REQUEST-CONSUMER.md).
Status: frozen bounded descriptor. Tier: full; new VC consumer fails predecessor same-owner/consumer pattern criterion.
Owner: [VC plan](../../mizar-vc/en/00.crate_plan.md); dependencies: [producer](VC53-CORE46-UNARY-CHILD-REQUEST.md), [VC53 boundary](ALGDEF-VC53-NORMAL-RETURN.md). Consumer: mizar-test.
Authority: [§§20.1.1–2, 20.2.6, 20.4.1, 20.7.2, 20.13.1/3](../../../spec/en/20.algorithm_and_verification.md).
Owner APIs/invariants: [Checker selection](../../mizar-checker/en/type_checker.md#selected-direct-call-dependencies), [Core lowering](../../mizar-core/en/elaborator.md#static-source-algorithm-lowering), [VC direct-return boundary](../../mizar-vc/en/generator.md#bounded-direct-return-e1e4-producer-design).
Gap: source_drift/test_gap; Core produces authenticated unary requests, but VC has no authenticated formal-to-actual descriptor. No demonstrated language-intent gap; descriptor-only work is ready.

## Frozen profile and design
Accept only the producer's two source-ordered public object-valued unary algorithms in distinct definition blocks: callee solely returns its immutable formal, caller solely returns the selected callee applied to its own immutable parameter. Optional terminating intent supplies no evidence.
Add `generate_source_unary_child_call_descriptor(source, typed, symbols, core, request)` in generator.rs. Freshly run the existing Checker producer on the complete genuine resolved source/neutral TypedAst/environment; lower from that fresh seal, then compare the whole Core and whole request before projecting anything. Preserve upstream errors; descriptor refusal is `algorithms.unary_child.invalid_request`.
Return existing-type pairs `((formal, actual), (formal_guard, actual_guard), (callee_result, caller_result))` atomically. The guards are original Core formula IDs for atomic object(formal) and object(actual); the latter describes the bounded capture-free instantiation of the former. Require distinct formal/actual variables and result roles. The original immutable Core/request retain exact caller/callee, Apply operands, sole Return, source and declaration associations. The result pair records slots only, never an equality or verified normal return.
No new syntax/term/formula representation, substitution framework, mutable input or schedulable E4 graph. No Kernel/Proof checking, verified caller-input normal-return evidence, adoption/evidence, promotion, acceptance/semantic credit, ranges, syntax/environment generalization, owner-gated work, diagnostics or language-intent changes. Later authenticated instantiation/adoption owns the normal-return evidence join. Existing Open/Deferred, raw/evidence/reuse policy and direct-return APIs remain unchanged.
Files: generator.rs, existing mizar-test runner algorithm tests, paired VC generator/plan documents and this contract. No AGENTS/protocol/spec/.miz/expectation/trace edits; audit/coverage status unchanged.

## Test matrix and exit
Cap: six families; reuse genuine producer setup and confirm one successful case before expanding.
1. Genuine pair: exact formal/actual, both original object guard IDs and callee/caller result slots; independently inspect existing Apply/Return/owner/source joins; no new obligations or credit.
2. Rename/trivia, equal-spelling distinct scopes and optional terminating variants regenerate correctly; repeated calls are deterministic and all inputs remain unchanged.
3. Whole request tampering: every field, swapped identities/roles, valid but wrong IDs and out-of-range IDs refuse atomically; genuine original remains usable.
4. Whole Core tampering: guards/subjects, Apply/Return/binders/owners, source/provenance and unrelated retained tables refuse, including valid-index edits; genuine original remains usable.
5. Foreign/stale source, environment and neutral typed projection, unsupported source/type/call profile refuse via fresh authentication; upstream errors are preserved.
6. Existing producer and direct-return controls remain unchanged; descriptor success supplies no schedulable VCs, Kernel results, evidence or adoption/status changes.
Existing tests: [producer integration](../../../../crates/mizar-test/src/runner/tests/algorithm.rs), Checker source_overload tests and unchanged [direct return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz).
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews are required.
Run fmt, all-target/all-feature Clippy with denied warnings, both lint_policy suites including links/fragments, VC owner tests, new regressions, existing algorithm/source-return groups and short proof-verification E2E. Full cargo test only on cadence; lane dispatch has no full/push trigger. Changed authentication/identity obligations run focused tests; unchanged evidence/policy/cache/acceptance/edit-versus-clean heavy matrices defer to orchestrator integration.
Exit: authenticated atomic descriptor and refusal matrix pass; no broader VC53 admission or audit/trace credit.
