# Task STEP5C3-ATTRIBUTE-SEMANTICS: activate attribute semantics

> Canonical language: English. Japanese pointer:
> [../ja/STEP5C3-ATTRIBUTE-SEMANTICS.md](../ja/STEP5C3-ATTRIBUTE-SEMANTICS.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index) and
[mizar-test](../../mizar-test/en/00.crate_plan.md#task-index); the prerequisite also belongs to [mizar-kernel](../../mizar-kernel/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Partial: eight mapped rows complete; attributed widening inactive |
| Purpose | Preserve eight active rows; add only the kernel equality prerequisite for the deferred attributed-widening proof |
| Tier | Full: trusted-kernel semantics; no new mapped activation in this prerequisite |
| Owner / consumer | `mizar-kernel` owns the equality prerequisite; existing checker, resolver and harness semantic ownership remains unchanged |
| Dependencies | Step 5A.2 and Step 5C.2 complete |
| Audit impact | Retain Chapter 3/6 source coverage; record only Chapter 14 normal-kernel equality reflexivity, with source acceptance and Task274 deferred |

Authority is [Chapter 3 §3.5](../../../spec/en/03.type_system.md#35-subtyping-and-widening),
[Chapter 6](../../../spec/en/06.attributes.md), [14 §14.5.2](../../../spec/en/14.formulas.md#1452-equality-and-inequality), [21](../../../spec/en/21.source_code_annotation_and_atp.md), the nine mapped `.miz` sources and
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
The next prerequisite gives existing Equality atoms their reflexive logical meaning through [kernel-owned SAT encoding](../../mizar-kernel/en/sat_encoding.md), with genuine same-term validation.
It changes no evidence family or proof policy and provides no source-proof acceptance, attributed evidence or Task274 closure.
Required controls cover unequal operands, ordinary predicate atoms, substitutions, deterministic derived clauses and proof-obligation versus consistency polarity.

## Boundary and exit

The runner may read `SurfaceAst` only to produce syntax-free inputs. The checker receives only
resolver-authenticated identities and existing representations and invariants in the
[attribute owner](../../mizar-checker/en/source_attribute.md#step-5c3-attribute-semantics). The added negative reuses the
[type-checker owner](../../mizar-checker/en/type_checker.md#distinct-loci-overload-source-checking) with its explicit single-candidate profile; C13 calls retain the strict profile.
The completed negative adds no term, formula, or statement representation, public type, public diagnostic code, or
lower-stage behavior. Authenticate both actual set-to-structure applications and the builtin-field constructor witness.
Private admission binds all source/sidecar metadata; reserved identities cannot fall through other stages;
its [runner and test design](../../mizar-test/en/harness.md#step-5c3-attribute-semantic-runner)
keeps metadata from selecting semantics. Preserve expectation outcome/phase/key, trace status/order,
all other `.miz`, proof/soundness policy, the 23-case rejection corpus, and Task 277B zero credit.

Required reviews are specification/documentation, test sufficiency, implementation, volume/scope,
and source/document consistency. Run focused corpus tests, then workspace format, warnings-denied
Clippy, and full tests. Exit requires eight exact activations, unchanged attributed-widening deferral and C13 behavior,
one task-only local commit within the same cumulative budgets, and no archive change.
