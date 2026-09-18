# Task STEP5C3-ATTRIBUTE-SEMANTICS: activate attribute semantics

> Canonical language: English. Japanese pointer:
> [../ja/STEP5C3-ATTRIBUTE-SEMANTICS.md](../ja/STEP5C3-ATTRIBUTE-SEMANTICS.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index) and
[mizar-test](../../mizar-test/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Partial: eight mapped rows complete; attributed widening inactive |
| Purpose | Preserve seven original rows and activate only the remaining functor argument mismatch |
| Tier | Full: semantic-credit and expectation-tag changes |
| Owner / consumer | `mizar-checker` owns attribute and argument typing; `mizar-resolve` retains symbol identity and duplicate rejection; `mizar-test` extracts, invokes, and compares |
| Dependencies | Step 5A.2 and Step 5C.2 complete |
| Audit impact | Retain prior Chapter 3/6 coverage; update both chapters for the argument continuation; do not edit `doc/spec` |

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
| `fail_type_elaboration_argument_type_mismatch_functor_001` | `active_type_elaboration` / `type_check`; `types.application.argument_type_mismatch` |

The attributed-widening positive remains inactive after the approved WMarkedExists witness registration; authentic accepted existential evidence remains deferred.

## Boundary and exit

The runner may read `SurfaceAst` only to produce syntax-free inputs. The checker receives only
resolver-authenticated identities and existing representations and invariants in the
[attribute owner](../../mizar-checker/en/source_attribute.md#step-5c3-attribute-semantics). The added negative reuses the
[type-checker owner](../../mizar-checker/en/type_checker.md#distinct-loci-overload-source-checking) with its explicit single-candidate profile; C13 calls retain the strict profile.
Add no term, formula, or statement representation, public type, public diagnostic code, or
lower-stage behavior. Authenticate both actual set-to-structure applications and the builtin-field constructor witness.
Private admission binds all source/sidecar metadata; reserved identities cannot fall through other stages;
its [runner and test design](../../mizar-test/en/harness.md#step-5c3-attribute-semantic-runner)
keeps metadata from selecting semantics. Preserve expectation outcome/phase/key, trace status/order,
all other `.miz`, proof/soundness policy, the 23-case rejection corpus, and Task 277B zero credit.

Required reviews are specification/documentation, test sufficiency, implementation, volume/scope,
and source/document consistency. Run focused corpus tests, then workspace format, warnings-denied
Clippy, and full tests. Exit requires eight exact activations, unchanged attributed-widening deferral and C13 behavior,
one task-only local commit within the same cumulative budgets, and no archive change.
