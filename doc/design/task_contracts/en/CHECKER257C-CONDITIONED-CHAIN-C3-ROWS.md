# Task CHECKER257C-CONDITIONED-CHAIN-C3-ROWS: conditioned-chain C3 rows
Canonical language: English; [Japanese pointer](../ja/CHECKER257C-CONDITIONED-CHAIN-C3-ROWS.md).
Status: frozen for implementation. Purpose: authenticate conjunction and segment-negation rows for whole-chain predicate conditions without installing C2 and C3 together.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: existing Typed/final C3 replay; [family ledger](../../../../tests/coverage/predicate_chain_composition.tsv); [task index](../../mizar-checker/en/todo.md).
Predecessors: full-tier Task 257C3 [composition boundary](../../mizar-checker/en/source_formula_composition.md#variable-reference-predicate-chain-associations) and [whole-chain C2 associations](CHECKER257C-SCOPED-PREDICATE-CHAIN-CONDITIONS.md), implemented at 7e93e5dfc after freeze d936038d6.
Authority: [EN9.3.3](../../../spec/en/09.predicates.md#933-multi-argument-predicates), [EN13.4.2](../../../spec/en/13.term_expression.md#1342-set-comprehension-fraenkel-operator), [EN14.2](../../../spec/en/14.formulas.md#142-atomic-formulas), [EN14.3.1](../../../spec/en/14.formulas.md#1431-conjunction-), [EN14.3.5](../../../spec/en/14.formulas.md#1435-negation-not).
Classification: bounded source_drift/test_gap. Full tier: the named pattern criterion that fails is a new public seam; existing C3 build cannot receive a C2 receipt or its Application/Set dependencies.

## Frozen boundary
Add only `SourcePredicateChainCompositionProducer::build_conditioned(input, primary, applications, sets, atomic, conditions, arena)`, using existing argument types. Retain a private optional C2 handoff inside C3; no new public type, field or row meaning. Existing `build`, input rows, IDs, errors and standalone debug output remain unchanged.
Authenticate the supplied complete C2 receipt against all lower dependencies on construction and immutable Typed/final replay. Retain the receipt as a dependency, never in the installed C2 slot. Require installed matching Application/Set dependencies for conditioned C3 replay. Preserve reciprocal C2/C3 installation exclusion.
Reuse the existing C3 complete ordered adjacent-pair and negative-segment census. C2 associates each condition with its whole Atomic formula, never a segment. Admit only run-6 lower-authenticated chains, including multiple disjoint single-condition Comprehensions, mixed singleton siblings and original source contexts. No new condition shape or lower-stage authentication rule.
Change only source_formula_composition.rs, necessary existing typed_ast.rs/resolved_typed_ast.rs replay plumbing, existing source_term.rs and runner composition tests, paired owner-boundary documents and indices, and the ledger. Native route selection remains C2; this slice exercises the existing C3 consumer directly.
Defer C2/C3 co-installation, broader conditions and predicate profiles, typing/applicability, sethood, truth/facts, proof/acceptance and Core/VC credit. No doc/spec, .miz, expectation, trace status, coverage credit or diagnostic changes.

## Exact test matrix
Cap: 24 cases; enumeration: 16 cases. Cases 1–8 authenticate genuine source lower products, exact C3 rows, deterministic build/clone and Typed/final replay with empty semantic tables.
1. One condition, two positive segments: one conjunction, no negation.
2. One condition, three positive segments: two ordered conjunctions.
3. Two disjoint conditions: complete per-formula conjunction rows.
4. Two segments, first `does not`: first-segment negation.
5. Two segments, last `do not`: last-segment negation.
6. Three segments, middle negative: middle-segment negation.
7. Multiple conditions with mixed polarity: independent ordered negation ordinals.
8. Alpha/layout variation and negative singleton sibling: chain rows only, singleton unchanged.
9. Singleton-only conditioned input: C2 preserved, C3 refused.
10. Existing unconditioned numeric and variable chains: existing C3 behavior preserved.
11. Condition associated to a segment or another condition's formula: C2/C3 refused.
12. Negation row targets a positive segment: C3 refused.
13. Conditioned C3 supplied through existing build without C2: refused.
14. Missing or mismatched Application/Set replay dependencies: refused.
15. Foreign/stale C2 receipt or changed physical condition owner: refused.
16. Both C2/C3 installation orders: refused with pristine state/replay preserved.

## Exit
Reuse existing scoped source oracle and Task257C3/Step5C7 regressions; no forged lower success. Independent PRE authority/test-intent/trust, POST sufficiency/precision/consistency, volume and final reviews follow the [protocol](../../autonomous_crate_development.md#task-workflow).
Freeze: EN/JA pair, owner indices and ledger; both lint_policy suites and local links/fragments. Implementation: fmt, Clippy, both lint_policy suites, scoped Primary/Set/Atomic/Application/C2/C3/Typed-final and Step5C7 owner tests, verifier/resolver standard, Driver source_load and short type/formula/metadata CLI. Full cargo test follows named cadence triggers only. Evidence remains outside doc; Task 257C stays open.
