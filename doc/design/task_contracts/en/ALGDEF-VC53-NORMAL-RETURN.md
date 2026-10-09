# Task ALGDEF-VC53-NORMAL-RETURN: Normal-return and function-graph prerequisites
Canonical language: English; [Japanese pointer](../ja/ALGDEF-VC53-NORMAL-RETURN.md).
Status: blocked on producers. Tier: full gates. Owners: [Core plan](../../mizar-core/en/00.crate_plan.md), [VC plan](../../mizar-vc/en/00.crate_plan.md), [proof plan](../../mizar-proof/en/00.crate_plan.md).
Consumers: mizar-test, module-local promotion; imported acceptance remains ALGDEF-VC59-IMPORTED-PROMOTION-FACTS.
Dependencies: ALGDEF-PARSER-FAIL; authentic Core 42–53 call/range/group/CFG inputs and replayable VC formulas; genuine proof/kernel results. Ready only after each selected source slice has these inputs.
Authority: [§20.2.6, §20.3, §20.7.2, E1–E5 and VC schema](../../../spec/en/20.algorithm_and_verification.md); [algorithm graph](../../mizar-vc/en/source_vc_decomposition.md#accepted-algorithm-vc-task-graph).
Classification: source_drift/test_gap; source return/state/Pick/while profiles provide no general normal-return acceptance.
Owner details: [Core/CFG graph](../../mizar-core/en/source_family_decomposition.md#accepted-phase-10-controlflowir-task-graph), [VC generator](../../mizar-vc/en/generator.md#step-4-algorithm-vcs), [proof status](../../mizar-proof/en/status.md#source-existential-registration-proof).

## Boundary and invariants

Core 43/48/53 carry validated failure placement, runtime report terms and
return-or-fail closure; failure contributes no normal/return/break/continue
context or postcondition. Void fallthrough returns implicitly; value fallthrough
is rejected. Ghost execution cannot fail or influence runtime execution.
VC 44 adds fresh returns and declared ensures on a partial body's normal call
continuation, including recursive induction while checking E2; external E2
adoption requires verified body correctness. Continuation proves no E4.
VC 53 admits independently checked exact-call normal-return evidence for
terminating child calls and iteration ranges; stopping or value equality is
insufficient. Core supplies identities; VC substitutes without capture; proof
owns actual acceptance. No status is inferred from declarations or open goals.
Before E1 symbol introduction justify single-valued execution, typing and
inhabitation on every input where its return type is asserted; E2 needs body
correctness, E4 independently verified normal return. Keep environment/InputOK
guards for syntax-bearing inputs with AST-CONTRACT. Ghost Pick requires static
discharge; non-ghost unsupported set Pick may fail only in partial execution.
Missing evidence stays incomplete/open at its owning boundary, never accepted.

Existing regressions: [return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz), [ghost rejection](../../../../tests/miz/fail/algorithms/fail_type_elaboration_algorithm_ghost_isolation_001.miz); these are bounded, not normal-return proofs.

## Tests and acceptance

Design fresh real sources for partial/recursive continuation, terminating
partial child/range with and without genuine return proof, explicit/implicit
failure, void return and value fallthrough, ghost/report leakage, empty Pick,
and E1 typing/inhabitation/single-valuedness failures. Compare complete Core/CFG
and VC outputs, plus stale/cross-call/group/provenance corruption; distinguish
postcondition correctness from normal return and finite-budget failure.
Acceptance: only complete source-authenticated inputs produce the specified
contexts/goals; only genuine checked evidence enables local promotion.
Forbidden: MVM/E5 execution, imported receipts, public diagnostic allocation,
existing .miz/expectation rebaselining, fabricated proofs or coverage credit.
Audit: retain §20 deferred ownership until executable source slices land.
Reviews: independent specification, test sufficiency, implementation,
volume/scope, consistency and EN/JA; full protocol gates.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test.
