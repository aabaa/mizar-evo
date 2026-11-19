# Task MVM-R8-CORPUS-DESIGN: Stage-gated MVM integration corpus

Canonical language: English; [Japanese pointer](../ja/MVM-R8-CORPUS-DESIGN.md).
Status: design draft; independent review and documentation lint pending.
**Documentation only**; no corpus activation, passing runtime test or
coverage credit. Sole owner: [mizar-test](../../mizar-test/en/00.crate_plan.md).
Design: [test owner matrix](../../mizar-test/en/mvm_corpus_matrix.md).
Authority: [spec §20.9](../../../spec/en/20.algorithm_and_verification.md#209-execution-model-and-computability),
[§20.13.1](../../../spec/en/20.algorithm_and_verification.md#20131-contract-axioms),
[§20.13.7](../../../spec/en/20.algorithm_and_verification.md#20137-certified-computation-import-t1t8),
[AST-CORPUS](AST-CORPUS.md), [staged model](../../mizar-test/en/staged_model.md),
[expectations](../../mizar-test/en/expectation_schema.md),
[traceability](../../mizar-test/en/traceability.md).
Dependencies: R1–R7 design contracts for the matrix; genuine producer and
stage-specific runner gates for implementation and activation.

## Scope

- Freeze an evidence matrix for R8-A source/parser/typing,
  R8-B Core/CFG/VC, R8-C genuine Proof acceptance, R8-D validated
  Artifact/import, R8-E MVM execution/failure/budgets, R8-F full-closure
  computation admission, R8-G T1–T8/replay/status/cache and R8-H
  full source-to-artifact verification-time integration.
- For each family name a spec-derived positive oracle and independent
  negative/corruption and metamorphic controls; distinguish parser-only,
  Open VC, accepted body, actual E5 return, Proof-computed and fully
  replayed kernel_verified evidence. Test omissions and stale dependencies
  fail closed; no fabricated producer or self-confirming golden.
- Preserve the seven existing canonical stage ids and the current
  external expected_phase vocabulary. A distinct runtime-only stage or
  expected_phase requires separate harness/schema/trace review, **not**
  an invented identifier in this design.
- Identify the owner, stage and accepted input that unlock each new test;
  include exact replay/strict-release, transitive taint, cache invalidation,
  public/private imports, ghost, noncomputability and resource limits.

## Exit and prohibited changes

This task creates only an EN/JA owner design, contract and roadmap indexes.
No changes to .miz/sidecars/snapshots, test registry, spec_trace, coverage,
Rust, public schema, MVM behavior, status, expected phase or proof policy.
Existing VC-only sidecars retain their expectations and cannot earn runtime
coverage. R8-A–H are planned test families, not dispatchable contracts
or accepted fixtures. R1–R7 implementation blockers stay in force.
Independent specification, test-sufficiency, trust/ownership, volume and
EN/JA reviews and repository documentation/link checks precede closeout.
