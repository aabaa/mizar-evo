# Task STEP5C4-MODE-SEMANTICS: activate mode semantics

> Canonical language: English. Japanese pointer:
> [../ja/STEP5C4-MODE-SEMANTICS.md](../ja/STEP5C4-MODE-SEMANTICS.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index) and
[mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Complete; all eight mapped rows active |
| Purpose | Preserve the original seven and activate the remaining plain-RHS dependent-mode pair |
| Tier | Full: semantic-credit and expectation-tag changes |
| Owner / consumer | `mizar-checker` owns mode, property, and sethood checks; `mizar-resolve` retains identities; `mizar-test` extracts, invokes, and compares |
| Dependencies | Step 5A.6 and Step 5C.2 complete; Step 5A.8 informs the explicit proof case |
| Classification | `source_drift` and executable `test_gap`; no specification or test-intent change |
| Audit impact | Update only the Chapter 7 rows in `doc/design/spec_coverage_audit.md`; do not edit `doc/spec` |

Authority is [Chapter 7](../../../spec/en/07.modes.md), the eight mapped `.miz` sources and
expectations, [trace records](../../../../tests/coverage/spec_trace.toml), and the ordered
[activation map](../../../../tests/coverage/step5_activation_map.tsv), in repository authority order.

## Exact activation

| Case | Tag / phase |
|---|---|
| `pass_type_elaboration_mode_attributed_struct_radix_001` | `active_type_elaboration` / `type_check` |
| `fail_type_elaboration_mode_dependent_arity_mismatch_001` | `active_type_elaboration` / `type_check` |
| `pass_type_elaboration_mode_dependent_of_params_001` | `active_type_elaboration` / `type_check` |
| `pass_type_elaboration_mode_property_impl_equals_001` | `active_type_elaboration` / `type_check` |
| `pass_type_elaboration_mode_property_impl_means_001` | `active_type_elaboration` / `type_check` |
| `fail_parse_only_mode_property_impl_missing_correctness_001` | `active_parse_only` / `parse` |
| `fail_type_elaboration_mode_property_impl_unknown_property_001` | `active_type_elaboration` / `type_check` |
| `fail_proof_verification_mode_sethood_unprovable_001` | `active_proof_verification` / `verification` |

The added pair checks actual formal/argument bindings at both mode uses and builtin-set RHS inhabitation.
Dependent functors, attributed/imported modes, sethood and proof acceptance remain outside this continuation.

## Boundary and exit

The runner authenticates exact source/sidecar identities, stage, phase, outcome, domain, spec refs and sole tag;
reserved identities cannot fall through other stages. It authenticates resolver identities and invokes existing checker representations described
by the [mode owner](../../mizar-checker/en/source_mode_definition.md#step-5c4-mode-semantics) and
[property owner](../../mizar-checker/en/source_property_implementation.md#step-5c4-property-implementation-semantics).
Add no term, formula, or statement representation, public type, public diagnostic code, or
lower-stage behavior. Means requires both correctness clauses, equals requires a declared property,
mode applications preserve actual parameter identity and arity, and a bare-`set` mode cannot discharge sethood merely by
`thus thesis`. Preserve outcome/phase/key, trace status/order, every `.miz`, oracle scope, proof and
soundness policy, and the 23-case rejection corpus.

Required reviews are specification/documentation, test sufficiency, implementation, volume/scope,
and source/document consistency. Run focused corpus tests, then workspace format, warnings-denied
Clippy, and full tests. Exit requires all eight exact activations within the same cumulative task budgets,
one task-only local commit, and no archive or Task262 transport change.
