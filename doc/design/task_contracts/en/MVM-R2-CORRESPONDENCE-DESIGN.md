# Task MVM-R2-CORRESPONDENCE-DESIGN: Verified algorithm/body/call binding

Canonical language: English; [Japanese pointer](../ja/MVM-R2-CORRESPONDENCE-DESIGN.md).
Status: design drafted, independent review pending; **documentation only**,
not a freeze of any executable producer. Tier: full documentation gates.
Sole owner: [mizar-core](../../mizar-core/en/00.crate_plan.md).
Design: [Core owner document](../../mizar-core/en/mvm_correspondence.md).
Inputs: [MVM-R1 inventory](../../mizar-vc/en/mvm_readiness.md),
[spec §20.1.1](../../../spec/en/20.algorithm_and_verification.md#2011-algorithm-definition-block),
[§20.9.1](../../../spec/en/20.algorithm_and_verification.md#2091-the-mizar-virtual-machine-mvm),
[§20.13.1](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms),
[AST-LOWER](AST-LOWER.md), [AST-CONTRACT](AST-CONTRACT.md),
[ALGDEF-VC53-NORMAL-RETURN](ALGDEF-VC53-NORMAL-RETURN.md).
Readiness: design may be reviewed while R1 remains review-pending;
R2 implementation is dependency-blocked.

## Scope

- Freeze two separate authenticated relations: (a) algorithm definition,
  checked body, contracts, CFG and accepted evidence, consumed by R3
  **without a concrete call**; (b) typed resolved call/ordered actuals,
  joined with that exported definition at R4. Define negative controls
  and currentness without inventing a runtime payload.
- Distinguish the *ordinary term* argument path from the reflected `expr`
  path by the existing contextual grammar. Propose any narrowing of the
  current AST-TYPE/AST-LOWER dependency for owner/human review, not as an
  already-authorized bypass.
- Define proof-evidence prerequisites: E1 typing, single-valuedness and
  inhabitation; E2 actual partial-correctness acceptance; E4 additionally
  for `terminating` promotion; and E5 as future actual MVM execution only.
- Specify R3's reusable definition-side Artifact export and R4's
  call-specific join, without fixing a serializer, bytecode, crate or API.

## Constraints and exit

Preserve source authority and all current `.miz`, expectations, coverage,
task states, runtime/code-extraction ownership and existing public API.
No semantic change, new runtime representation, `CoreIr` acceptance flag,
synthesized evidence, source reparsing, or Artifact/mock execution. The
existing `idalgo` VC is Open and is not a verified executable.
Review the Q1 ordinary/reflection dependency and Q2 evidence completeness
explicitly; keep R3 representation/loader choice and integration separate.
Deliver the EN/JA definition-versus-call owner design, rejection matrix
and handoff.
Independent specification, boundary, test-sufficiency, volume and EN/JA
reviews plus documentation link/lint checks are required to close this task.
No implementation, MVM readiness, or coverage credit follows from design.
