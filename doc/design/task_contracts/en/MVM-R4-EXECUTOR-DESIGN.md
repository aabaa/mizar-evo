# Task MVM-R4-EXECUTOR-DESIGN: Minimal authenticated execution boundary

Canonical language: English; [Japanese pointer](../ja/MVM-R4-EXECUTOR-DESIGN.md).
Status: documentation-only design review closed; implementation remains blocked. Tier: full documentation gates.
**Documentation only**; no runtime implementation or acceptance credit.
Sole owner: [mizar-vc](../../mizar-vc/en/00.crate_plan.md).
Design: [VC owner document](../../mizar-vc/en/mvm_executor.md).
Authority: [spec §20.9.1–4](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability),
[§20.13.1 E5](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms),
[ALG-MVM-EVALUATION](ALG-MVM-EVALUATION.md),
[R2](MVM-R2-CORRESPONDENCE-DESIGN.md),
[R3](MVM-R3-ARTIFACT-LOADER-DESIGN.md),
[computation admission](ALGDEF-COMPUTATION-ADMISSION.md).
Readiness: design review can proceed; R4 execution implementation remains
blocked on authentic R2 verified body/call, R3 validated runtime artifact,
AST-LOWER complete inputs and typed computability facts.

## Scope

- Bound one minimal genuine ground/direct call to its original typed input,
  source/body/contract identity, validated R3 executable handle, VM version
  and complete applicable dependencies; preserve provenance and refusal.
- Specify call-by-value, local-only runtime state, exact normal return and
  non-return/failure/incomplete boundaries for a single supported direct
  return. Define later R5 control-flow, recursion, nondeterminism, ghosts,
  noncomputability and step/time/depth extensions as separate gates.
- Distinguish an internal MVM normal-return report/E5 input from
  separately accepted R6 full-closure admission, R7 `by computation`
  conversion/replay and Proof-owned status. No result is trusted merely
  because an execution API or hash reports success.
- Preserve spec option defaults (`0` is unlimited), with the exact
  step-charge/timeout/nest and budget propagation rule frozen before
  executing even the pilot. Existing pre-ATP discharge policy counters are
  not an implicit VM gas interpretation.

## Acceptance and exclusions

Identify the smallest existing source candidate (the `idalgo` Open VC)
and the genuine call/type, verified body, loader and oracle blockers.
Specify positive/negative tests for correct normal result, wrong body/
input/version, missing or stale Artifact, failed requires, non-return,
unsupported instruction, ghost leak and resource refusal; keep stage and
producer authority explicit. An unavailable path yields no result/fact;
a failed attempt never proves mathematical non-return.
No new public type, VM instruction schema, source reparse, fake runnable
artifact, default budget, proof primitive, status, diagnostic, test-credit
or Rust change. R5 extensions are not implemented by this design task.
Independent specification, test-sufficiency, ownership, volume and EN/JA
reviews plus both documentation lint/link checks precede closure.
No executable tests run in this design task; future cases are bounded by the owner design matrix; additions require a cited specification clause.
