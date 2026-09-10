# Task STEP5C8-FORMULA-SEMANTICS: bounded formula checking

Canonical language: English; [Japanese pointer](../ja/STEP5C8-FORMULA-SEMANTICS.md).
Owning plans: [checker](../../mizar-checker/en/00.crate_plan.md#task-index), [test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: complete; full tier; primary owner: checker; consumer: test runner.
- Dependency: 5B.2 is complete; reuse the completed occurrence-resolution bridge.
- Authority: [Chapter 14](../../../spec/en/14.formulas.md) §§14.2.3, 14.3–14.6;
  [Chapter 4](../../../spec/en/04.variables_and_constants.md) §4.5;
  [Chapter 3](../../../spec/en/03.type_system.md) builtin types;
  [Chapter 15](../../../spec/en/15.statements.md) §§15.2.1, 15.4.4, 15.11.5;
  the seven Step 5C.8 [activation-map](../../../../tests/coverage/step5_activation_map.tsv)
  sources, their sidecars, and the unchanged trace manifest.
- Classification: executable `test_gap` / `source_drift`; no specification change.

## Scope and boundaries

Activate only these mapped pairs, preserving their exact ids and all expectations
except adding the sole tag for the declared stage. Public codes remain empty.

| Case suffix | Stage / phase: expected result |
|---|---|
| `connective_precedence_001` | `formula_statement` / `statement_check`: pass |
| `formula_unbound_free_variable_001` | `type_elaboration` / `resolve`: `formulas.free_variable_unbound` |
| `iff_unparenthesized_chain_001` | `parse_only` / `parse`: `formulas.iff.unparenthesized_chain` |
| `iff_parenthesized_001` | `formula_statement` / `statement_check`: pass |
| `is_type_assertion_001` | `formula_statement` / `statement_check`: pass |
| `existential_multi_witness_001` | `formula_statement` / `statement_check`: pass |
| `nested_quantifier_st_holds_001` | `formula_statement` / `statement_check`: pass |

Admission authenticates id, source/sidecar paths, stage, phase, outcome, key,
empty public codes, and sole tag; malformed candidates never fall through.
Reuse `SurfaceAst`, sealed `ResolvedVariableScope`, `TypedArena`/`TypedAst`,
and existing checker term/formula inputs and source identities; add no parallel IR.
Resolver remains responsible for lexical scope and reference identity. The harness
extracts existing representations without deciding formula or statement semantics.
Checker validates builtin-typed atoms, logical child structure, ordered quantifiers,
restrictions, proof-local generalization, and ordered witness types/substitution.
Keep parentheses/precedence, actual bound identities, and restrictions intact;
an empty diagnostic list from a scope-only pass is insufficient semantic evidence.
The parse case requires the parser's actual non-associative-chain diagnostic at
the unparenthesized `iff` chain; unrelated errors or arbitrary source text cannot qualify.
Broader terms/types, named witnesses, general proof discharge, imported facts,
ATP, VC and kernel changes remain outside this bounded statement-checking task.
Preserve completed routes, all existing `.miz`, expectation semantics, trace,
activation map, archive, ratchet, certificate corpus, and Task 277B state.
Owner details: [type checker](../../mizar-checker/en/type_checker.md),
[harness](../../mizar-test/en/harness.md); audit impact: Chapter 14 and its bounded
Chapter 15 witness/generalization dependency, without proof-verification credit.

## Exit

Review specification/docs, test sufficiency, implementation, removable volume,
and source/doc consistency. Exercise the seven corpus outcomes and fixture-free
mutations of admission, binding/provenance, connective/quantifier structure,
asserted types, witness arity/type/order, restrictions, and unrelated parse errors.
Run focused tests/corpora, fmt, warnings-denied Clippy and cargo test.
One local commit within the production/documentation/public-type budgets.
