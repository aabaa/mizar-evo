# Task MVM-R6-ADMISSION-DESIGN: Full-closure computation admission

Canonical English; [Japanese pointer](../ja/MVM-R6-ADMISSION-DESIGN.md).
Status: design draft; independent reviews and documentation lint pending.
Documentation-only; **no computation acceptance or runtime implementation**.
Sole design owner: [mizar-vc](../../mizar-vc/en/00.crate_plan.md).
Design: [VC owner document](../../mizar-vc/en/mvm_computation_admission.md).
Proof remains the separate acceptance/diagnostic owner under
[ALGDEF-COMPUTATION-ADMISSION](ALGDEF-COMPUTATION-ADMISSION.md).
Authority: [spec §20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step),
[§20.3](../../../spec/en/20.algorithm_and_verification.md#203-non-determinism-pick),
[§22.4.5](../../../spec/en/22.error_handling_and_diagnostics.md#2245-atp-timeout-and-computation-limit-handling),
[VC58](ALGDEF-VC58-COMPUTATION-UNIQUENESS.md),
[Resolver call edges](ALGDEF-RESOLVE-CALL-EDGES.md),
[R5 design](MVM-R5-EXTENDED-EXECUTION-DESIGN.md).
Readiness: design is independently reviewable; real full-closure
generation/admission is blocked on source call, type, accepted proof and
R3 validated exported-body/dependency producers.

## Design scope

- Partition R6 into: A authenticated direct/transitive local/imported
  call closure, including private helpers; B exact trigger versus
  non-trigger classification and permitted `ensures` exemptions;
  C full per-callee §20.9.2 uniqueness VC with source/goal/guards and
  evidence identity; D Proof-owned acceptance and E0330 projection.
- Every **value-returning** callee containing non-ghost Pick or unordered
  set `for ... in` needs accepted uniqueness evidence unless its exact
  resolved `ensures` has one of the two specified result-independent
  exemption forms. A non-trigger, a ghost-only Pick and an ordered
  sequence without Pick do not create this obligation.
- Preserve independent E1 single-valuedness, requires, computability,
  AST formation, replay and R7 certified conversion. Neither a raw VM
  return nor an Open goal, syntactic exemption flag or cache entry proves
  those other conditions. This gate is required **before computation-based
  discharge**, not necessarily before an internal failed evaluation.
- Document stage-specific positive, refusal and corrupted-source/export/
  proof cases; never infer imported bodies or original statements from
  their names, a signature or a local source fallback.

## Exit and prohibitions

Design review must identify the first genuinely sourced local path,
the imported-closure blockers, and a single owner for each production
and acceptance handoff. R6-A–D remain design slices until paired
implementation contracts are independently frozen by their owners.
No Rust, .miz/expectation/trace edits, new VC/status/kernel rule,
artificial proof, policy relaxation or public diagnostic allocation.
Keep full computation admission blocked while the actual producer
chain is absent. Independent specification, proof-sufficiency,
boundary, volume and EN/JA reviews and repository documentation checks
are necessary before marking this design task complete.
