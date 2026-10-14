# Task ATP-STRUCTURED-LOCAL-RANKING: structured candidate ordering
Canonical language: English; [Japanese pointer](../ja/ATP-STRUCTURED-LOCAL-RANKING.md).
Status: implemented within supplied-candidate ordering. Tier: full. Owner: [ATP plan](../../mizar-atp/en/00.crate_plan.md), task 37, bounded task 31 prerequisite.
Consumers: future VC selected-input assembly; no source acceptance or pruning authority.
Authority: spec §22.4.4; [problem owner](../../mizar-atp/en/problem.md#structured-local-ranking), ATPK-PREMISE-RANKING.
Dependencies: existing normalized immutable AtpProblem and its structured source projections; full visible library, citation/budget assembly and suggestions remain absent.
Gap: source_drift/test_gap; available structured candidate inputs have no deterministic symbol ordering.
Scope: add AtpProblem::rank_axioms() -> Result<(&'static str, Vec<&AtpFormula>), String> in existing problem.rs, returning every original axiom reference and configuration symbol-overlap-v1.
Collect distinct named predicate/function symbols from complete goal and axiom trees; include nested terms and zero-argument functions, ignore variables, binders, sorts, equality and logical operators. Missing trees fail closed; never recover contents from text/hash.
Use the owner's exact integer weighting and unique formula-ID tie-break; repeated symbols never inflate occurrence or overlap.
Rarity is relative only to this supplied candidate set, not a global library estimate. Preserve every row, formula, provenance, target, profile and input identity; mutate no problem and add no conjecture as an axiom.
Tests: fixed data-only normalized trees with literal expected rare/common/shared-symbol orders, repetition, nested terms/connectives/quantifiers, variables, ties, empty candidates and missing payloads.
Append actual single/three-registration NeedsAtp source projection tests, original/renamed/false definitions and both existing encodings; exact original rows/anchors, recorded configuration and deterministic order remain bound to their actual producer.
Data-only ordering fixtures are not source/authentication/accepted-evidence producers. Existing source, ATP, Kernel and Proof tests retain their intent.
Forbidden: new type/field/file/helper or duplicated AST, floating/learned/nondeterministic scoring, premise pruning/addition, uncited acceptance, backend/Kernel calls, policy/diagnostic changes, fake source/evidence/status, protected spec/.miz/expectation/trace edits.
Source scope: existing problem.rs with additive local tests and appended runner/tests.rs source tests; no broader refactor or abstraction.
Audit: local untrusted ordering prerequisite only; full task31/library rarity/max_axioms/retry/labels/artifacts and Steps 6–8 remain open with no semantic coverage promotion.
Reviews: specification, tests, code, volume/scope, consistency, scoped gates/quality; parent fmt, full Clippy -D warnings and cargo test.
Exit: actual structured inputs produce deterministic complete ordering and configuration, unsupported payloads supply no ranking, and no proof/citation/budget authority changes.
