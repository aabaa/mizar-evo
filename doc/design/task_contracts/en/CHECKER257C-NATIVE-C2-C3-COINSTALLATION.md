# Task CHECKER257C-NATIVE-C2-C3-COINSTALLATION: native conditioned-chain co-installation
Canonical language: English; [Japanese pointer](../ja/CHECKER257C-NATIVE-C2-C3-COINSTALLATION.md).
Status: frozen for implementation. Purpose: acquire and co-install native C2/C3 for the same conditioned chains without semantic credit.
Owner: [runner plan](../../mizar-test/en/00.crate_plan.md); lower owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); [task index](../../mizar-checker/en/todo.md); [family ledger](../../../../tests/coverage/predicate_chain_composition.tsv).
Pattern tier; predecessors: [co-installation](CHECKER257C-C2-C3-COINSTALLATION.md) at cf9e83d9b, [native scoped conditions](CHECKER257C-NATIVE-SCOPED-PREDICATE-CONDITIONS.md) at 87175d44b and [whole-chain C2](CHECKER257C-SCOPED-PREDICATE-CHAIN-CONDITIONS.md) at 7e93e5dfc. Same owner/consumer and refusal classes; no new public seam or protected surface.
Authority: [EN4.6](../../../spec/en/04.variables_and_constants.md#46-scoping-and-shadowing), [EN9.3.3](../../../spec/en/09.predicates.md#933-multi-argument-predicates), [EN13.4.2](../../../spec/en/13.term_expression.md#1342-set-comprehension-fraenkel-operator), [EN14.2](../../../spec/en/14.formulas.md#142-atomic-formulas), [EN14.3.1](../../../spec/en/14.formulas.md#1431-conjunction-), [EN14.3.5](../../../spec/en/14.formulas.md#1435-negation-not).
Classification: bounded source_drift/test_gap; native scoped C2 already acquires whole chains but does not install conditioned C3. Dependencies implemented; no blocker.

## Frozen boundary
Extend only the existing native scoped condition acquisition in runner/type_elaboration/source_formula_composition.rs and existing runner composition tests. Reuse original-source/Resolver/collector/scoped BindingEnv and complete Primary/Set/Atomic/empty Application/C2 products; no fallback, reconstructed contexts, new carrier or producer.
For each authenticated multi-segment condition atom, derive the complete ordered adjacent-segment conjunction and negative-segment census using existing Atomic segment/edge IDs. Build conditioned C3 with the same complete C2 receipt, install C2 first then C3, and replay all lower products through existing Typed/final authentication. Negative singleton siblings contribute no C3 rows; singleton-only C2 and standalone unconditioned C3 preserve behavior.
Reuse [C2/C3 ownership and census](../../mizar-checker/en/source_formula_composition.md#variable-reference-predicate-chain-associations) and existing native transport detail keys. Missing associations, missing/mismatched C2, duplicate/conflicting installation refuse without partial state. No public API/type/field changes; private local plumbing only; lower ownership unchanged.
Owner indices, paired stale native-deferral wording and family ledger may change. No doc/spec, .miz, expectations, trace status or coverage-credit change; audit unchanged. Defer broader conditions/profiles, grouping, inference, sethood/result typing, facts/truth, proof/acceptance and Core/VC credit.

## Exact test matrix
Cap: 24 cases; exact enumeration: 12 cases, genuine native acquisition with literal source/child-derived row expectations.
1. One positive two-segment condition: native Primary/Set/Atomic/C2/C3/Typed/final authentication and deterministic replay succeed.
2. One three-segment condition with middle negation: complete ordered conjunction/negation census succeeds.
3. Multiple disjoint conditioned chains with mixed polarity: complete C2/C3 censuses succeed.
4. Header and proof chains with shadowing and unused declarations: actual lexical parents and complete lower rows replay.
5. Alpha/layout variant: identical acquisition guarantees and deterministic clone/final replay.
6. Chain plus negative singleton sibling: both C2 associations retained, C3 rows only for chain.
7. Singleton-only native C2: no C3, existing transport/replay retained.
8. Unconditioned native chain: existing standalone C3 route and replay retained.
9. Native co-installation missing its C2 condition association: refused; pristine source and retry unchanged.
10. Native conditioned C3 presented without installed matching C2: refused; lower-only input unchanged.
11. Duplicate native C2 or C3 installation: refused; pristine co-installed input unchanged.
12. Native C3 paired with foreign/conflicting C2: refused; original input and deterministic replay unchanged.

## Exit
Freeze EN/JA pair, owner indices and ledger first; both lint_policy suites validate local links/fragments. Implementation: fmt, all-target/all-feature Clippy, both lint_policy suites, native composition matrix, scoped source oracle, C2/C3 and Typed/final groups, Step5C7; corpus/Driver and type/formula/metadata CLI with independent volume review. Combined implementation/sufficiency/paired-consistency review and removable-only volume review cover the complete diff. Full cargo test --locked runs for the dispatched push cadence before push. Evidence stays outside doc; Task 257C remains open.
