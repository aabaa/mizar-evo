# Task STEP5C3-ATTRIBUTE-SEMANTICS: activate attribute semantics

> Canonical language: English. Japanese pointer:
> [../ja/STEP5C3-ATTRIBUTE-SEMANTICS.md](../ja/STEP5C3-ATTRIBUTE-SEMANTICS.md).

Owning plans: [mizar-checker](../../mizar-checker/en/00.crate_plan.md#task-index) and
[mizar-test](../../mizar-test/en/00.crate_plan.md#task-index), [kernel](../../mizar-kernel/en/00.crate_plan.md#task-index), [Core](../../mizar-core/en/00.crate_plan.md#task-index), [VC](../../mizar-vc/en/00.crate_plan.md#task-index), and [proof](../../mizar-proof/en/00.crate_plan.md#task-index).

## Frozen assignment

| Field | Value |
|---|---|
| Status | Partial: eight mapped rows complete; attributed widening inactive |
| Purpose | Preserve eight active rows and kernel equality; implement the bounded genuine WMarkedExists producer, leaving widening inactive |
| Tier | Full: source proof and accepted-registration association; no new mapped activation |
| Owner / consumer | Checker authenticates inputs; Core owns proof/choice; VC owns leaves/evidence; kernel checks; proof owns policy and source-bound activation |
| Dependencies | Step 5A.2 and Step 5C.2 complete |
| Audit impact | Chapters 13/15/17 gain the implemented bounded producer; preserve Chapter 14 kernel reflexivity and all mapped deferrals |

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

The attributed-widening positive remains inactive; the approved full source supplies only the bounded WMarkedExists prerequisite.
Preserve [kernel equality reflexivity](../../mizar-kernel/en/sat_encoding.md) and its existing evidence/policy boundaries.
[Proof status](../../mizar-proof/en/status.md#source-existential-registration-proof) freshly authenticates source→Core→VC→both kernel leaves and activates only the same-source registration.
Retain the parent existential, actual take/citation, independent builtin nonempty accounting and full Core/VC/handoff baselines.
Foreign/missing gate, source, proof, citation, leaf or policy evidence rejects; no caller Accepted/result receipt is trusted.

## Boundary and exit

The runner may read `SurfaceAst` only to produce syntax-free inputs. The checker receives only
resolver-authenticated identities and existing representations and invariants in the
[attribute owner](../../mizar-checker/en/source_attribute.md#step-5c3-attribute-semantics). The added negative reuses the
[type-checker owner](../../mizar-checker/en/type_checker.md#distinct-loci-overload-source-checking) with its explicit single-candidate profile; C13 calls retain the strict profile.
Authenticate both actual set-to-structure applications and the builtin-field constructor witness.
Private admission binds all source/sidecar metadata; reserved identities cannot fall through other stages;
its [runner and test design](../../mizar-test/en/harness.md#step-5c3-attribute-semantic-runner)
keeps metadata from selecting semantics. Preserve expectation outcome/phase/key, trace status/order,
all `.miz`, proof/soundness policy, the 23-case rejection corpus, and Task 277B zero credit.
C5/C13, general Task274/artifact import, cluster/reduction effects and Steps6/7 remain outside this prerequisite.

Required reviews are specification/documentation, test sufficiency, implementation, volume/scope,
and source/document consistency. Run focused corpus tests, then workspace format, warnings-denied
Clippy, and full tests. Exit requires eight exact activations, unchanged attributed-widening deferral and C13 behavior,
one task-only local commit within the same cumulative budgets, and no archive change.
