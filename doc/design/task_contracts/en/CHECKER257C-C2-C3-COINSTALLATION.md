# Task CHECKER257C-C2-C3-COINSTALLATION: conditioned-chain co-installation
Canonical language: English; [Japanese pointer](../ja/CHECKER257C-C2-C3-COINSTALLATION.md).
Status: frozen for implementation. Purpose: co-install whole-chain C2 associations and conditioned C3 rows without semantic credit.
Owner: [checker plan](../../mizar-checker/en/00.crate_plan.md); consumer: [Typed ownership](../../mizar-checker/en/typed_ast.md#task-257c3-frozen-ownership)/[final replay](../../mizar-checker/en/resolved_typed_ast.md#task-257c3-frozen-final-projection); [task index](../../mizar-checker/en/todo.md); [family ledger](../../../../tests/coverage/predicate_chain_composition.tsv).
Pattern tier; predecessors: [run 6 C2](CHECKER257C-SCOPED-PREDICATE-CHAIN-CONDITIONS.md) at 7e93e5dfc and [run 7 C3](CHECKER257C-CONDITIONED-CHAIN-C3-ROWS.md) at 1f0157db4. Same owner, consumer, rows and refusal classes; no public seam or protected-surface change.
Authority: [EN9.3.3](../../../spec/en/09.predicates.md#933-multi-argument-predicates), [EN13.4.2](../../../spec/en/13.term_expression.md#1342-set-comprehension-fraenkel-operator), [EN14.2](../../../spec/en/14.formulas.md#142-atomic-formulas), [EN14.3.1](../../../spec/en/14.formulas.md#1431-conjunction-), [EN14.3.5](../../../spec/en/14.formulas.md#1435-negation-not).
Classification: bounded source_drift/test_gap; reciprocal exclusion currently prevents the already authenticated products sharing Typed/final slots. Dependencies are implemented; no blocker.

## Frozen boundary
Install C2 first, then conditioned C3 only when its retained complete C2 receipt equals the installed C2. Reauthenticate equality and all lower dependencies on immutable final replay; missing or conflicting C2 refuses without partial state. Conditioned C3 construction remains valid, but installation without C2 is now refused. Keep existing duplicate and C3-first C2 refusals.
Reuse [owner validation and census](../../mizar-checker/en/source_formula_composition.md#variable-reference-predicate-chain-associations), physical ownership and empty semantic tables. Preserve standalone unconditioned C3, singleton C2, native C2 route selection, row identities and debug chunks.
Change only source_formula_composition.rs, typed_ast.rs, resolved_typed_ast.rs, existing source_term.rs and runner composition tests, paired owner-boundary documents/indices and ledger. No new public API/type/field; private replay plumbing only. Lower-stage ownership is unchanged.
Defer broader conditions/profiles, native C3 selection, typing, sethood, truth/facts, proof/acceptance and Core/VC credit. No doc/spec, .miz, expectation, trace status or coverage-credit change; audit unchanged.

## Exact test matrix
Cap: 24 cases; enumeration: 14 cases, using genuine source lower products and the existing scoped oracle.
1. Single condition, positive two-segment chain: C2/C3 authenticate, co-install and clone/replay through Typed/final.
2. Single condition, three segments with middle negation: complete ordered rows co-install and replay.
3. Multiple disjoint conditions with mixed polarity: complete whole-formula C2 and C3 censuses co-install and replay.
4. Chain with singleton sibling: both C2 associations persist, C3 contains chain rows only, Typed/final replay succeeds.
5. Alpha/layout variant: deterministic genuine-source co-installation and Typed/final replay.
6. Singleton-only C2 and unconditioned-chain C3: existing installation/replay behavior preserved.
7. Missing C2 condition association at construction: refused; genuine input and pristine replay unchanged.
8. Conditioned C3 installed without C2: refused; lower-only state preserved.
9. Foreign or conflicting installed C2 receipt: C3 installation refused; original C2 state preserved.
10. Duplicate C3 installation onto co-installed state: refused; pristine co-installed replay preserved.
11. Duplicate C2 or C2 after C3: refused; pristine input state preserved.
12. Test-injected C3 without its matching installed C2: final replay refused.
13. Test-injected conflicting C2/C3 pair: final replay refused.
14. Missing/stale Application or Set dependency: co-installation/replay refused with original state preserved.

## Exit
Freeze EN/JA pair, owner indices and ledger first; both lint_policy suites and local links/fragments. Implementation: fmt, Clippy, both lint_policy suites, scoped C2/C3 source oracle, Typed/final, family oracle/metamorphic and Step5C7 regressions. Combined implementation/sufficiency review and volume review cover the complete diff; paired owner consistency is included. Corpus/Driver and type/formula/metadata CLI run with volume review; full cargo test only on named cadence triggers. Evidence stays outside doc; Task 257C remains open.
