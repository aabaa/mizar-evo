# Task MVM-R5-EXTENDED-EXECUTION-DESIGN: Control, failure and resource cases

Canonical language: English; [Japanese pointer](../ja/MVM-R5-EXTENDED-EXECUTION-DESIGN.md).
Status: documentation-only design review closed; implementation remains blocked.
Documentation-only; no R5 implementation readiness or semantic coverage.
Sole owner: [mizar-vc plan](../../mizar-vc/en/00.crate_plan.md).
Design: [VC owner document](../../mizar-vc/en/mvm_extended_execution.md).
Authority: [spec §20.1.3–4](../../../spec/en/20.algorithm_and_verification.md#2013-variable-declarations-var-const),
[§20.2](../../../spec/en/20.algorithm_and_verification.md#202-control-flow),
[§20.3](../../../spec/en/20.algorithm_and_verification.md#203-non-determinism-pick),
[§20.9](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability),
[R4](MVM-R4-EXECUTOR-DESIGN.md),
[ALG-MVM-EVALUATION](ALG-MVM-EVALUATION.md) and
[R6 admission](ALGDEF-COMPUTATION-ADMISSION.md).
Readiness: design can proceed; executable slices require genuine R2/R3
inputs, a completed supported R4 evaluator and feature-specific Core/CFG.

## Scope and decomposition

- Stage R5-A local `var`/`const`, typed assignments, runtime conditionals
  and early `return`; stage R5-B `while`, range and sequence loops with
  break/continue and exact exit; stage R5-C checked calls/recursion and
  transitive validated dependencies; stage R5-D collection-set ordering,
  executable Pick and reflected `match` only where certified support exists.
- Stage R5-E explicit/implicit `fail`, noncomputable operations and
  ghost-isolation refusals; stage R5-F `steps`, `timeout`, `nest`,
  cancellation and exact failure/incomplete reporting. R4 must freeze a
  minimal accounting rule before first execution; R5 expands its tests.
- For every stage distinguish (1) source/typed/CFG admissibility, (2) real
  validated execution and E5 internal report, (3) R6 nondeterministic-closure
  proof admission and (4) R7 certified public import/replay. Never conflate
  any pair or invent a new VM format, schema, Kernel rule or proof status.
- Document feature-specific positive, negative, stale/forged and boundary
  tests, precise prerequisite ownership, safe refusal and release triggers.

This design task executes no semantic tests. The future test matrix is limited to
the owner design table's rows; adding a row requires the specification
clause that requires it.

## Acceptance and restrictions

Keep R4's call-by-value, local state, real artifact identity, runtime-version
and all-transitive dependency checks. Failed execution, missing artifacts,
unproven preconditions, `fail` and resource exhaustion yield no successful
return, no accepted contract fact and no proof of non-return. Non-ghost Pick
and unordered set iteration require accepted unique-result eligibility for
computation-based *discharge*, not a fabricated deterministic choice.
Follow source §20 rules for loop order, capture, and ghost erasure.
No Rust, .miz/sidecar, trace, spec, public API, diagnostic or task-status
promotion in this design task. Never rebaseline existing VC-only cases.
Exit is an EN/JA owner design with one dependency/rejection matrix and
implementation freeze triggers; independent specification, test-sufficiency,
boundary, volume, EN/JA reviews and both lint/link checks must pass.
