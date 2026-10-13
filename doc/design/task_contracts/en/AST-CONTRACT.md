# Task AST-CONTRACT: Verified reflected algorithm obligations
Canonical language: English; [Japanese pointer](../ja/AST-CONTRACT.md).
Status: planned. Tier: full gates. Primary owner: [mizar-vc](../../mizar-vc/en/00.crate_plan.md).
Consumers: mizar-kernel, mizar-proof and the MVM producer.
Dependencies: AST-LOWER complete input/context/C and genuine checked
algorithm/CFG facts; AST-FORMATION-PROOFS accepted independent formation and proof inputs, ALGDEF-VC53-NORMAL-RETURN; Core54/VC56/VC57/VC58/VC59 retain their separate scopes.
Readiness: blocked on these handoffs; request transport is not normal return.
Authority: [§20.4](../../../spec/en/20.algorithm_and_verification.md#204-contracts-requires-ensures-assert),
[§20.7](../../../spec/en/20.algorithm_and_verification.md#207-termination-proofs),
[§20.13.1](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms),
and [architecture 15](../../architecture/en/15.kernel_certificate_format.md#reflection-rule-and-execution-boundary).
Gap: source_drift/test_gap for E1 formation, M8-guarded partial correctness
and verified normal-return/promotion connections.
Scope: generate result typing/inhabitation/single-valuedness prerequisites,
partial correctness over identical complete input; E4 normal-return obligations
apply where terminating/promotion requires them, not to every partial definition;
preserve Adm/InputOK and each compatible environment.
Include explicit/implicit failure, child requires/normal return, iteration
ranges, invariants and the selected decreasing measure; expose labels/equations
only after the specified verification prerequisites.
Existing promotion/uniqueness follow-ups retain their own unresolved inputs;
this task must not treat their planning records as verified evidence.
Forbidden: E1/E2/E4 from declarations/contracts alone, return from value equality,
bare algorithm citations, fabricated proof status, MVM E5 or kernel trust here.
Owner-local invariants/tests: [generator.md](../../mizar-vc/en/generator.md#reflected-algorithm-obligations).
Owner details: [algorithm generation](../../mizar-vc/en/generator.md#step-4-algorithm-vcs),
[local context](../../mizar-vc/en/vc_ir.md#localcontext),
[request-only boundary](../../mizar-vc/en/generator.md#bounded-computation-request-generation).
Tests: use the linked owner-local matrix.
Existing [Open request](../../../../tests/miz/pass/algorithms/pass_proof_verification_computation_justification_001.miz) and [bounded return](../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_ensures_return_001.miz) remain regressions, not normal-return proofs.
Audit: generation credit never establishes E5, MVM execution or accepted proof.

Reviews: specification/documentation, test sufficiency, implementation,
volume/scope, and source/document consistency under the canonical protocol.
Verification: cargo fmt --check; cargo clippy --all-targets --all-features --
-D warnings; cargo test; affected owner tests and local-link/parity checks.
Exit: frozen scope satisfied by authentic producer inputs and reviewed tests;
no completion or coverage claim for deferred consumers.
