# Task MVM-R7-CERTIFIED-IMPORT-DESIGN: Certified import and replay

Canonical English; [Japanese pointer](../ja/MVM-R7-CERTIFIED-IMPORT-DESIGN.md).
Status: design draft, independent review and documentation lint pending.
**Documentation only**; no computed or kernel-verified proof acceptance.
Sole design owner: [mizar-proof](../../mizar-proof/en/00.crate_plan.md).
Design: [Proof owner document](../../mizar-proof/en/mvm_computation_import.md).
Authority: [spec §20.9.2](../../../spec/en/20.algorithm_and_verification.md#2092-by-computation-proof-step),
[§20.13.7](../../../spec/en/20.algorithm_and_verification.md#20137-certified-computation-import-t1t8),
[AST-COMPUTE](AST-COMPUTE.md),
[R6 admission](MVM-R6-ADMISSION-DESIGN.md),
[AST-ARTIFACT](AST-ARTIFACT.md),
[Build computation policy](BUILD-COMPUTATION-POLICY.md).
Readiness: design review may start; implementation remains blocked on real
R2–R4/5 body, artifact, execution, accepted R6 admission, formation,
contract, Proof and policy producers.

## Scope and owners

- **R7-A** authenticate the full MVM E5 normal-return input, exact
  source/call/context/C/artifact/versions, accepted contracts and R6
  admission; prove requires and Adm/InputOK independently.
- **R7-B** instantiate whole ensures and value-equality candidates,
  then apply finite directed T1–T8 with terminal typed UID substitution,
  ordered guards and certified Binder/structure operations.
- **R7-C** inspect each complete converted candidate including quantified
  types, annotations and containers; retain internal return on failure,
  publish no residual or successful ensures fragment.
- **R7-D** keep default `computed` separate from all-call equation/SAT
  replay `kernel_verified`; manifest default, explicit override, and
  strict release / transitive computed-taint remain Proof-owned.
- **R7-E** stage exact result, effective replay mode, transitive executable
  and dependency identities for Artifact/Cache/Build consumption *after*
  Proof acceptance; reject stale or weaker cache reuse.

## Exclusions and exit

No new grammar, representation, kernel arithmetic/λ rule, universal
reflection evaluator, unbounded unfolding, source fallback, status
upgrade or synthetic certificate. Original R3 runtime export does not
depend on R7 result publication. Include T1–T8 and residual negative
controls, replay/strict/corruption matrix and stage-specific blockers.
R7-A–E are design slices only, not implementation contracts.
Do not edit Rust, language spec, .miz/expectation/trace, coverage, CI or
existing acceptance policy. Independent specification, test-sufficiency,
trust-boundary, volume, EN/JA review plus documentation lint/link checks
are required before declaring the design task complete.
