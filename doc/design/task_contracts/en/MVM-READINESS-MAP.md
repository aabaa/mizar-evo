# Task MVM-READINESS-MAP: Authentic MVM first-slice readiness inventory

Canonical language: English; [Japanese pointer](../ja/MVM-READINESS-MAP.md).
Status: documentation inventory drafted; independent reviews and lint
pending. Sole owner: [mizar-vc](../../mizar-vc/en/00.crate_plan.md).
Inventory: [mvm_readiness.md](../../mizar-vc/en/mvm_readiness.md).
Tier: full documentation gates. No runtime or proof acceptance credit.
Authority: [§20.9](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability),
[§20.13.1](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms),
[§20.13.7](../../../spec/en/20.algorithm_and_verification.md#20137-certified-computation-import-t1t8);
existing AST-LOWER, AST-CONTRACT, AST-ARTIFACT, ALG-MVM-EVALUATION,
AST-COMPUTE and ALGDEF-COMPUTATION-ADMISSION contracts.
Dependencies: source/specification and existing producer contracts only;
read-only inventory requires no executable MVM or accepted algorithm.

## Deliverables

1. Identify a source-backed smallest algorithm body and distinguish
   its actual parser/resolver/checker/Core/VC/Proof evidence from
   genuinely missing typed calls, executable export, loader and VM.
   A declaration, Open VC, theorem reflexivity or symbolic computation
   request cannot be a runtime-positive fixture.
2. Inventory R2–R8 owners, positive/negative test oracles, dependency
   blockers and the *separate* definition-side verified-body/export
   path versus invocation-side typed ground call and execution path.
   Distinguish ordinary ground terms from reflected expr/Binder/C.
   Any narrowing of existing AST-TYPE/AST-LOWER gates needs a reviewed
   owner/specification decision.
3. Record an acyclic graph: validated body -> R3 runtime export;
   validated export + genuine typed call -> R4 execution;
   accepted R6 full computation closure + R4 normal return ->
   R7 candidate conversion/replay; later computation record follows
   R7 and never gates earlier executable export.
   Test preparation is stage-specific and cannot invent producers.
4. Record the matrix in the linked owner document and a Japanese
   companion/pointer. Keep R2–R8 implementations blocked until real
   owner inputs, frozen contracts and acceptance oracles exist.

## Exclusions and review exit

No Rust, specification changes, existing .miz/expectation/trace edits,
coverage/status promotion, new runtime/API/crate, arbitrary executable,
source fallback, invented foundational proofs or kernel acceptance.
Confirm missing inputs, ownership, negative controls and absence of
publication/execution/proof cycles under independent spec, boundary,
test-sufficiency, volume/scope and bilingual review. Run both
lint_policy suites, link/fragment checks and git diff --check.
Review and verification evidence belongs in commit/handoff, not
this contract. No stage or milestone closes through this inventory.
