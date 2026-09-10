# Task STEP5C7-TERM-SEMANTICS: bounded term semantics

Canonical language: English; [Japanese pointer](../ja/STEP5C7-TERM-SEMANTICS.md).
Owning plans: [checker](../../mizar-checker/en/00.crate_plan.md#task-index), [resolve](../../mizar-resolve/en/00.crate_plan.md#task-index), [core](../../mizar-core/en/00.crate_plan.md#task-index),
[test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

- Status: complete; full tier; dependency 5A.1 is complete.
- Purpose: activate only the seven non-gap Step 5C.7 pairs in the [activation map](../../../../tests/coverage/step5_activation_map.tsv).
- Authority: [Chapter 13](../../../spec/en/13.term_expression.md) §§13.1.4,
  13.4–13.6, 13.8.6; [Chapter 3](../../../spec/en/03.type_system.md) builtin widening;
  [§7.8.1](../../../spec/en/07.modes.md#781-mode-property-sethood) and
  [§17.3.4](../../../spec/en/17.clusters_and_registrations.md#1734-inhabitation-evidence-and-existential-registrations), [Chapter 15](../../../spec/en/15.statements.md) premise labels;
  mapped `.miz` sources, expectations, and unchanged trace manifest.
- The approved direct-membership set bound is specified in §13.4.2 (EN/JA).
  It does not grant `sethood(object)` or infer bounds from arbitrary formulas.
- Classification: executable `test_gap` / `source_drift`; the guarded-comprehension `spec_gap` is resolved by the approved specification change.

## Scope and boundaries

Type/proof rows use sole tags `active_type_elaboration`/`active_proof_verification`,
respectively. Public codes stay empty; phase names below are exact.

| Case suffix (existing mapped id) | Stage / phase: expected result |
|---|---|
| `term_choice_uninhabited_001` | type / `type_check`: `terms.choice.missing_inhabitation` |
| `term_choice_builtin_001` | type / `type_check`: pass |
| `term_numeral_equality_001` | type / `type_check`: pass |
| `term_qua_widening_001` | type / `type_check`: pass |
| `term_comprehension_unbound_mapper_001` | type / `resolve`: `terms.comprehension.unbound_mapper_variable` |
| `term_comprehension_guarded_001` | proof / `vc_generation`: pass |
| `term_set_enumeration_membership_001` | proof / `vc_generation`: pass |

The G5 narrowing pair stays inactive. Admission authenticates id, source/sidecar
paths, stage, phase, outcome, key, empty public codes, and sole stage tag.
Malformed mapped rows must fail closed rather than fall through to older routes.
Reuse `SurfaceAst`, existing resolver binding identities, checker `Source*`
handoffs/type inference, and Core source-handoff normalization; add no parallel IR.
Resolve real occurrences and scope before checking types. Missing inhabitation
is not inferred from spelling. Proof passes require source-derived propositions,
proof-local bindings, guards/citations, and discharged membership obligations;
matching token text or an empty diagnostic list alone is not proof acceptance.
Resolver owns scope, checker owns typing/evidence, Core owns normalization,
and the harness owns extraction/admission. Broader terms, arbitrary guard solving,
imported semantics, and ATP/kernel changes are outside this bounded task.
Preserve all existing `.miz`, expectation outcomes/phases/keys, trace, activation
map, archive, ratchet, certificate rejection corpus, and Task 277B state.
Owner APIs: [occurrences](../../mizar-resolve/en/names.md), [labels](../../mizar-resolve/en/labels.md), [primary](../../mizar-checker/en/source_term.md), [typing](../../mizar-checker/en/type_checker.md).
[Membership](../../mizar-checker/en/source_set_term.md), [atomic ownership](../../mizar-checker/en/source_atomic_formula.md), [normalization](../../mizar-core/en/elaborator.md), [runner](../../mizar-test/en/harness.md); audit: Chapter 13 and its bounded Chapter 15 proof dependency.

## Exit

Review specification/docs, test sufficiency, implementation, removable volume,
and source/doc consistency. Runner unit tests reject missing/wrong-root/extra-tag
admission, missing/non-set/dependent guards, wrong mapper binding, missing/wrong
citation, and altered conclusions; malformed provenance must not earn oracle credit.
Keep these mutations fixture-free; test the seven exact corpus outcomes separately.
Run focused tests/corpus commands, fmt, warnings-denied Clippy, and cargo test.
One local commit; respect the approved production/documentation/public-type budgets.
