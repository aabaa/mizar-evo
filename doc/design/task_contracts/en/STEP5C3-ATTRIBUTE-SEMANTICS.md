# Task STEP5C3-ATTRIBUTE-SEMANTICS: activate attribute semantics

> Canonical language: English. Japanese pointer:
> [../ja/STEP5C3-ATTRIBUTE-SEMANTICS.md](../ja/STEP5C3-ATTRIBUTE-SEMANTICS.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index) and
[mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Complete; two G1 pairs remain deferred |
| Purpose | Activate only the seven non-G1 Step 5C.3 attribute pairs |
| Tier | Full: semantic-credit and expectation-tag changes |
| Owner / consumer | `mizar-checker` owns attribute typing; `mizar-resolve` retains symbol identity and duplicate rejection; `mizar-test` extracts, invokes, and compares |
| Dependencies | Step 5A.2 and Step 5C.2 complete |
| Classification | `source_drift` and executable `test_gap`; no specification or test-intent change |
| Audit impact | Update only the Chapter 3/6 rows in `doc/design/spec_coverage_audit.md`; do not edit `doc/spec` |

Authority is [Chapter 3 §3.5](../../../spec/en/03.type_system.md#35-subtyping-and-widening),
[Chapter 6](../../../spec/en/06.attributes.md), the nine mapped `.miz` sources and
expectations, [trace records](../../../../tests/coverage/spec_trace.toml), and the ordered
[activation map](../../../../tests/coverage/step5_activation_map.tsv), in repository authority order.

## Exact activation

| Case | Tag / phase |
|---|---|
| `fail_type_elaboration_attr_duplicate_same_subject_001` | `active_type_elaboration` / `resolve` |
| `pass_type_elaboration_attr_struct_qualified_reference_001` | `active_type_elaboration` / `type_check` |
| `pass_type_elaboration_attr_param_prefix_declaration_001` | `active_type_elaboration` / `type_check` |
| `fail_type_elaboration_attr_param_prefix_unbound_001` | `active_parse_only` / `parse` |
| `pass_type_elaboration_attr_redefine_narrower_subject_001` | `active_type_elaboration` / `type_check` |
| `fail_type_elaboration_attr_non_attribute_symbol_001` | `active_type_elaboration` / `type_check` |
| `pass_formula_statement_attr_negated_chain_assertion_001` | `active_formula_statement` / `statement_check` |

The two G1 argument-widening pairs remain inactive gap records with unchanged sidecars.

## Boundary and exit

The runner may read `SurfaceAst` only to produce syntax-free inputs. The checker receives only
resolver-authenticated identities and existing representations and invariants in the
[attribute owner](../../mizar-checker/en/source_attribute.md#step-5c3-attribute-semantics).
Add no term, formula, or statement representation, public type, public diagnostic code, or
lower-stage behavior. Private runner routing is exact by id, path, stage, phase, outcome, and tag;
its [runner and test design](../../mizar-test/en/harness.md#step-5c3-attribute-semantic-runner)
keeps metadata from selecting semantics. Preserve expectation outcome/phase/key, trace status/order,
all `.miz`, proof/soundness policy, the 23-case rejection corpus, and Task 277B zero credit.

Required reviews are specification/documentation, test sufficiency, implementation, volume/scope,
and source/document consistency. Run focused corpus tests, then workspace format, warnings-denied
Clippy, and full tests. Exit requires seven exact activations, two unchanged G1 gaps, all full gates,
one task-only local commit, and no archive change.
