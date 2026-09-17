# Task STEP5C5-PREDICATE-FUNCTOR-SEMANTICS: activate predicate and functor semantics

> Canonical language: English. Japanese pointer:
> [../ja/STEP5C5-PREDICATE-FUNCTOR-SEMANTICS.md](../ja/STEP5C5-PREDICATE-FUNCTOR-SEMANTICS.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index),
[mizar-resolve](../../mizar-resolve/en/00.crate_plan.md#task-index), [mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Partial: nine outcomes complete; eight gaps deferred |
| Tier | Full: semantic-credit and expectation-tag changes |
| Owner / consumer | `mizar-checker` owns definition/type checks; `mizar-resolve` owns identities and ordinary functor/predicate signature conflicts; `mizar-test` admits, extracts, invokes, and compares |
| Dependencies | Step 5A.2, 5A.3, 5A.6, and 5A.7 complete |
| Classification | `source_drift` and executable `test_gap`; no specification or test-intent change |
| Audit impact | Retain Chapters 9/10/19 coverage; update Chapters 3/9/19 for argument typing; do not edit `doc/spec` |

Authority is [§§9.4–9.5.1](../../../spec/en/09.predicates.md#94-dependent-and-typed-parameters), [§§3.4–3.5](../../../spec/en/03.type_system.md#34-built-in-types), [§10.3](../../../spec/en/10.functors.md#103-definition-styles-equals-vs-means),
[§10.6.1](../../../spec/en/10.functors.md#1061-functor-properties), [§19.1](../../../spec/en/19.overload_resolution.md#191-overview),
[§10.10](../../../spec/en/10.functors.md#1010-symbol-resolution-and-imports), the 17 mapped sources and expectations,
[trace records](../../../../tests/coverage/spec_trace.toml), and the ordered
[activation map](../../../../tests/coverage/step5_activation_map.tsv), in repository authority order.
## Exact activation

| Case | Phase / result |
|---|---|
| `fail_type_elaboration_pred_argument_type_mismatch_001` | `type_check` / `predicates.application.argument_type_mismatch` |
| `fail_type_elaboration_pred_duplicate_same_signature_001` | `resolve` / `predicates.definition.duplicate_same_signature` |
| `fail_type_elaboration_pred_property_arity_mismatch_001` | `type_check` / `predicates.property.arity_mismatch` |
| `pass_type_elaboration_pred_properties_declaration_001` | `type_check` / pass |
| `pass_type_elaboration_func_builtin_bracket_pair_001` | `type_check` / pass |
| `fail_type_elaboration_func_equals_result_type_mismatch_001` | `type_check` / `functors.equals.result_type_mismatch` |
| `fail_type_elaboration_func_means_missing_correctness_001` | `type_check` / `functors.means.missing_correctness` |
| `fail_type_elaboration_func_duplicate_same_signature_001` | `resolve` / `functors.definition.duplicate_same_signature` |
| `fail_type_elaboration_func_property_arity_mismatch_001` | `type_check` / `functors.property.arity_mismatch` |

Preserve eight active rows and add only predicate argument mismatch with sole `active_type_elaboration`. The other eight retain
their G1/G2/G6/G9 records, inactive sidecars, phases, outcomes, and keys, including narrower-loci registration deferral.

## Boundary and exit

Admission binds exact id, source/sidecar, stage, phase, outcome and tag; predicate negatives also bind category/key/domain/spec ref and absent payloads.
The [harness bridge](../../mizar-test/en/harness.md#step-5c5-type-elaboration-admission) accepts only
unrecovered definition/theorem shapes, authenticates resolver shells and symbols, and uses existing
`SurfaceAst` and checker type representations. The [predicate owner](../../mizar-checker/en/source_predicate_definition.md#step-5c5-predicate-property-semantics)
and [functor owner](../../mizar-checker/en/source_functor_definition.md#step-5c5-functor-semantics)
require binary-only properties to reject unary definitions; a `means`
functor requires existence and uniqueness clauses; an `equals` definiens must inhabit its declared
return type; the builtin two-argument bracket term is well typed. Predicate duplicates use the
[resolver](../../mizar-resolve/en/symbols.md) bounded builtin-locus key; labels/bodies and locus renaming cannot change that conflict.
The [application checker](../../mizar-checker/en/type_checker.md#ordinary-predicate-application-checking) owns actual ordered argument rejection and builtin-field inhabitation; [formal lookup](../../mizar-resolve/en/names.md) preserves grouped-locus identity.
Add no parallel IR, public type/code, proof acceptance, unfolding or unrelated lower-stage behavior.
Preserve every `.miz`, expectation outcome/phase/key, trace status/order, activation map, oracle
scope, soundness policy, 23-case certificate rejection corpus, Task 277B state, and archive.
Required reviews are specification/documentation, test sufficiency, implementation, volume/scope,
and source/document consistency. Run focused corpus tests, then workspace format,
warnings-denied Clippy, and full tests. Exit requires nine activations and eight unchanged gaps,
renamed-locus and distinct-type/arity/notation/kind controls plus both call/argument slots, and a task-only local commit.
