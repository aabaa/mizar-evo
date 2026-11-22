# Task VC53-UNARY-CHILD-EXISTENTIAL: authenticated unary child existential
Canonical language: English; [Japanese pointer](../ja/VC53-UNARY-CHILD-EXISTENTIAL.md).
Status: frozen bounded producer. Tier: full; checked instantiation introduces a new trust-boundary rule.
Owners: [VC](../../mizar-vc/en/00.crate_plan.md), [Kernel](../../mizar-kernel/en/00.crate_plan.md). Consumer: mizar-test; Proof consumption is subsequent work.
Dependencies: [authenticated descriptor](VC53-UNARY-CHILD-REQUEST-CONSUMER.md), [Core producer](VC53-CORE46-UNARY-CHILD-REQUEST.md), [VC53 boundary](ALGDEF-VC53-NORMAL-RETURN.md).
Authority: [§§20.1.2, 20.2.6, 20.7.2, 20.13.1](../../../spec/en/20.algorithm_and_verification.md), [§16.7.3](../../../spec/en/16.theorems_and_proofs.md#1673-skeleton-encoding).
Owner invariants/APIs: [VC descriptor](../../mizar-vc/en/generator.md#bounded-unary-child-request-descriptor), [Kernel existential](../../mizar-kernel/en/checker.md#original-algorithmreturns-existential).
Gap: source_drift/test_gap; authentication exists, but original direct-return APIs exclude paired Core and caller-input instantiation. No demonstrated language-intent gap.

## Frozen design and scope
Accept exactly the descriptor's genuine two-algorithm unary object profile, with optional terminating intent supplying no evidence. Fresh descriptor authentication and whole regenerated Core/request equality precede all projections.
Add `generate_source_unary_child_call_existential_request` and `check_source_unary_child_call_existential_result` in generator.rs with the descriptor's five inputs. The request is existing types `(LocalContext, [VcGeneratedFormula; 2], CoreTermId)`: sole CheckerFact is the original caller object guard, followed by a fresh result binder; row zero is `AlgorithmReturns(callee, original flow, original callee Return, actual, r)` and row one is `Exists r` over row zero. No fabricated seed, VcSet, scheduling or snapshot identity. Retain exact original source/Apply/Return/CFG and formal/actual/result-slot associations through immutable Core/request authentication.
Kernel adds `check_unary_child_returns_existential_result(core, request, target, binder, witness, context)`, returning existing `AlgorithmReturnsExistentialCheck` only on same-call success. Reuse the unchanged formal existential rule, then check exact caller Apply/Return/request joins, atomic object guard transport formal→actual, definition-free input/result, actual witness and capture-free binder. Retain Core, callee/flow/Return/actual/binder target, witness and sole caller guard. VC returns the unchanged request together with this result atomically. Preserve upstream errors; new local refusal is `algorithms.unary_child.invalid_existential_request`.
No Proof identity/current-policy/adoption, caller E4 acceptance, E1 discharge, generic substitution, broader profiles, evidence/status/promotion/publication/coverage credit, specification/diagnostic changes or owner-gated scope. Ranges, syntax/environment generalization and general malformed-Core safety stay deferred. G10 still gates P13; R3-Q2/R5-Q2 remain reserved.
Files: VC generator.rs, Kernel checker.rs, existing runner algorithm tests, paired owner module/plan/API-inventory documents and this contract. No AGENTS/protocol/spec/.miz/expectation/trace edits; spec_coverage_audit status unchanged.

## Test matrix and exit
Cap: six families; reuse genuine producer setup and pass one case before expansion.
1. Genuine pair: independently inspect caller guard, fresh binder, exact two-row existential, callee flow/Return, actual witness and same-call Kernel result; retain source/Apply/Return/result-slot joins.
2. Rename/trivia, equal spelling in distinct scopes and optional terminating variants preserve semantics, determinism and immutable inputs.
3. Whole Core/request drift and foreign/stale source/typed/environment refuse through fresh authentication; upstream errors precede local refusal; original remains usable.
4. Direct Kernel target/request tampering: callee/caller/Apply/Return/flow/formal/actual/result roles, definitions, witness, context and captured binder refuse, including valid wrong IDs; no detached equality or guard suffices.
5. Atomic object substitution/guard mutations refuse; formal-rule and unrelated-algorithm CFG/cycle refusal controls remain valid.
6. Existing direct-return/descriptor controls retain their boundaries; success grants no VcSet, adoption, status or credit.
Independent specification/test-intent/trust, sufficiency, precision/trust, removable-only volume, source/docs/EN-JA and final quality reviews required.
Run fmt, all-target/all-feature Clippy -D warnings, both lint_policy suites including links/fragments, Kernel/VC/Proof owner groups, focused regressions, algorithm/source-return groups and short proof-verification E2E. Changed instantiation/guard obligations run focused matrices; unchanged evidence/policy/cache/acceptance/edit-versus-clean heavy groups defer to orchestrator integration. Full cargo test only on cadence; lane dispatch has no full/push trigger.
Exit: authenticated caller-input child existential and same-call Kernel result with refusal matrix; no broader VC53 admission.
