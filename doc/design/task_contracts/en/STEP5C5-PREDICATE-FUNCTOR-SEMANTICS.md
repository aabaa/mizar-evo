# Task STEP5C5-PREDICATE-FUNCTOR-SEMANTICS: activate predicate and functor semantics

> Canonical language: English. Japanese pointer:
> [../ja/STEP5C5-PREDICATE-FUNCTOR-SEMANTICS.md](../ja/STEP5C5-PREDICATE-FUNCTOR-SEMANTICS.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index) and
[mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Complete; ten gap-blocked pairs remain deferred |
| Purpose | Activate only the seven non-G1/G2/G6/G9 Step 5C.5 pairs |
| Tier | Full: semantic-credit and expectation-tag changes |
| Owner / consumer | `mizar-checker` owns definition/type checks; `mizar-resolve` retains identities and rejects the duplicate functor signature; `mizar-test` admits, extracts, invokes, and compares |
| Dependencies | Step 5A.2, 5A.3, 5A.6, and 5A.7 complete |
| Classification | `source_drift` and executable `test_gap`; no specification or test-intent change |
| Audit impact | Update only the Chapter 9 and 10 rows in `doc/design/spec_coverage_audit.md`; do not edit `doc/spec` |

Authority is [§9.5.1](../../../spec/en/09.predicates.md#951-predicate-properties), [§10.3](../../../spec/en/10.functors.md#103-definition-styles-equals-vs-means),
[§10.6.1](../../../spec/en/10.functors.md#1061-functor-properties),
[§10.10](../../../spec/en/10.functors.md#1010-symbol-resolution-and-imports), the 17 mapped sources and expectations,
[trace records](../../../../tests/coverage/spec_trace.toml), and the ordered
[activation map](../../../../tests/coverage/step5_activation_map.tsv), in repository authority order.

## Exact activation

| Case | Phase / result |
|---|---|
| `fail_type_elaboration_pred_property_arity_mismatch_001` | `type_check` / `predicates.property.arity_mismatch` |
| `pass_type_elaboration_pred_properties_declaration_001` | `type_check` / pass |
| `pass_type_elaboration_func_builtin_bracket_pair_001` | `type_check` / pass |
| `fail_type_elaboration_func_equals_result_type_mismatch_001` | `type_check` / `functors.equals.result_type_mismatch` |
| `fail_type_elaboration_func_means_missing_correctness_001` | `type_check` / `functors.means.missing_correctness` |
| `fail_type_elaboration_func_duplicate_same_signature_001` | `resolve` / `functors.definition.duplicate_same_signature` |
| `fail_type_elaboration_func_property_arity_mismatch_001` | `type_check` / `functors.property.arity_mismatch` |

All seven receive the sole `active_type_elaboration` tag. The other ten Step 5C.5 rows retain
their G1/G2/G6/G9 records, inactive sidecars, phases, outcomes, and keys.

## Boundary and exit

Admission authenticates exact id, workspace-relative path, stage, phase, outcome, and sole tag.
The [harness bridge](../../mizar-test/en/harness.md#step-5c5-type-elaboration-admission) accepts only
unrecovered definition/theorem shapes, authenticates resolver shells and symbols, and uses existing
`SurfaceAst` and checker type representations. The [predicate owner](../../mizar-checker/en/source_predicate_definition.md#step-5c5-predicate-property-semantics)
and [functor owner](../../mizar-checker/en/source_functor_definition.md#step-5c5-functor-semantics)
require binary-only properties to reject unary definitions; a `means`
functor requires existence and uniqueness clauses; an `equals` definiens must inhabit its declared
return type; the builtin two-argument bracket term is well typed; duplicate same-signature functors
retain resolver ownership. Add no term, formula, or statement representation, public type, public
diagnostic code, proof acceptance, definitional unfolding, or lower-stage behavior.

Preserve every `.miz`, expectation outcome/phase/key, trace status/order, activation map, oracle
scope, soundness policy, 23-case certificate rejection corpus, Task 277B state, and archive.
Required reviews are specification/documentation, test sufficiency, implementation, volume/scope,
and source/document consistency. Run focused corpus tests, then workspace format,
warnings-denied Clippy, and full tests. Exit requires seven exact activations, ten unchanged gaps,
all full gates, one task-only local commit, and no archive change.
