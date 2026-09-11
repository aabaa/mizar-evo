# Task STEP5C10-THEOREM-SKELETONS: bounded theorem bridge

Canonical language: English; [Japanese pointer](../ja/STEP5C10-THEOREM-SKELETONS.md).
Owning plans: [checker](../../mizar-checker/en/00.crate_plan.md#task-index),
[resolve](../../mizar-resolve/en/00.crate_plan.md#task-index), [core](../../mizar-core/en/00.crate_plan.md#task-index),
[VC](../../mizar-vc/en/00.crate_plan.md#task-index), [proof](../../mizar-proof/en/00.crate_plan.md#task-index),
[test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: complete; full tier; checker owns semantics; dependencies 5C.8/5C.9 complete.
- Authority: [Chapter 16](../../../spec/en/16.theorems_and_proofs.md) §§16.1–5;
  [Chapter 14](../../../spec/en/14.formulas.md) quantification/equality;
  [Chapter 4](../../../spec/en/04.variables_and_constants.md) §4.6 scope.
- Exact sources/expectations: the six Step 5C.10 rows of the
  [activation map](../../../../tests/coverage/step5_activation_map.tsv).
- Classification: executable test gap and source drift: current source bridges
  omit lemma owners, status projection and these proof-skeleton outcomes.

## Scope and boundaries

Activate only those six rows; add only their sole active-stage tags.
Preserve each frozen phase: lemma citation reaches VC generation; open/assumed
registration reaches statement check; unknown label fails at resolve; the three
skeleton failures reach statement check with their existing distinct detail keys.
Authenticate source/sidecar identity, stage, phase, outcome, keys and tags;
malformed admission or unrelated errors never earn a frozen failure result.

Use SurfaceAst, resolver occurrence/label receipts and existing checker source
term/formula rows. Check raw role/status tokens, owner provenance, builtin types,
ordered universal generalization, actual pending thesis and conclusion, and
earlier module-local citation site/origin/target. Unsupported forms fail closed.
Reuse SourceTheoremOwnerInput and its role/status enums for owner metadata.
One opaque borrowed SourceTheoremCheck seals the checked transaction; no new
term/formula/statement representation. Core consumes this seal, never SymbolEnv,
through existing seeds/lowerers; VC consumes their real obligations and citations.
No hand-built VC, source-text equality, name-only theorem fact, or acceptance
based on case id. Harness dispatches outcomes; it is not a semantic authority.

Open and assumed declarations retain distinct existing policy projections,
never kernel-verified status. Reject forbidden assumed justifications and
unmodified dependencies on non-clean items; conditional proof support is deferred.
Justified open declarations are valid language, but deferred beyond this proofless profile.
General proof search, imported/grouped/bulk citations, broader skeletons, richer
types/terms and theorem/certificate acceptance remain deferred.
Preserve existing profiles, .miz/expectation intent, trace/map/ratchet, archive,
completed tasks, fail-closed boundaries and Task 277B not-ready/zero-credit.
Owner details: [checker](../../mizar-checker/en/type_checker.md),
[labels](../../mizar-resolve/en/labels.md), [Core](../../mizar-core/en/elaborator.md),
[VC](../../mizar-vc/en/generator.md), [policy](../../mizar-proof/en/policy.md),
[harness](../../mizar-test/en/harness.md). Audit impact: bounded Chapter 16 coverage.

## Exit

Review specification/docs, tests, implementation, removable volume and consistency.
Cover all six outcomes plus alpha-renaming, role/status/citation/provenance,
thesis/type drift, unsupported shapes and cross-stage admission mutations.
Run focused tests/corpora, fmt, warnings-denied Clippy and cargo test.
One local commit; approved production ≤1,800 added lines; documentation ≤200.
