# Audit: Semantic-Bridge Oracle Corpus Map (September 2026, Audit 1)

> Canonical language: English. Japanese companion:
> [../ja/semantic_bridge_corpus_map.md](../ja/semantic_bridge_corpus_map.md)
> (pointer only, per the September 2026 status-document language policy).

## Purpose

This document is the coverage map for the September 2026 audit-1 corpus
increment: a grammar-complete, spec-derived `.miz` oracle corpus for the
Step-5 semantic bridge. The corpus makes the semantic behavior of every
grammar chapter (`doc/spec/en/` chapters 3-20) executable-test-shaped before
the Step-5 re-decomposition (audit 2) rewrites the task plan, so that coarse
implementation tasks can be bound to concrete accept/reject expectations
instead of per-slice enumeration.

Tests are binding on later agents per the AGENTS.md authority order. Fail
expectations must not be weakened to match current behavior.

## Increment Summary

- 120 new case pairs (`.miz` + `.expect.toml`): 60 pass, 60 fail.
- 103 new traceability requirements in
  [`tests/coverage/spec_trace.toml`](../../../../tests/coverage/spec_trace.toml),
  spanning 17 grammar chapters.
- All pairs are **inactive oracle seeds**: no `active_*` runner tag, no
  current parser/semantic/route credit (Task 257C4C0/C4C7 precedent). They
  are discovered, metadata-validated, and traced; activation belongs to the
  owner tasks that close each family.
- Corpus totals move from 430 to 550 discovered cases; metadata validation
  passes with zero errors and only the pre-existing warnings.
- 91 of the 120 sources parse cleanly under the current frontend; the other
  29 are spec-correct sources blocked by the frontend gaps inventoried in
  [semantic_bridge_frontend_gaps.md](./semantic_bridge_frontend_gaps.md) and
  [`tests/coverage/audit1_frontend_gaps.tsv`](../../../../tests/coverage/audit1_frontend_gaps.tsv).

## Conventions

- Sources are self-contained: they declare their own attributes, modes,
  structures, functors, and predicates over the builtin `set`/`object`
  radixes, except three module cases that import the existing
  `parser.type_fixtures` / `parser.nested_capture_fixtures` fixture modules.
- Pass theorems are chosen so their eventual discharge is trivial for the
  deterministic checker or any ATP (reflexive equalities, definitional
  unfoldings, excluded-middle case splits); declaration-only cases carry no
  proof obligations beyond their correctness blocks.
- Sidecar `stage`/`expected_phase` state the earliest sound rejection point
  (fail) or the acceptance boundary (pass) per the staged model; failure
  identities use `failure_category` plus a stable `stable_detail_key`.
- New corpus subdirectories preserve the pass/fail split:
  `variables`, `structures`, `modes` (pass), `attributes` (fail),
  `predicates`, `functors`, `terms`, `formulas`, `theorems` (fail),
  `overload` (pass), `templates` (pass), `algorithms`.

## Deliberate Scope Notes

- Algorithm *execution* (MVM, spec chapter 20 sections 20.9-20.10) stays
  excluded per the roadmap; algorithm-verification constructs (contracts,
  invariants, ghost state, claim blocks, computation justifications) are
  covered.
- A self-contained positive `sethood` case is impossible over builtin
  radixes (instances of any bare `set`-radix mode form a proper class), so
  sethood is covered by the unprovable-fail case
  `fail_proof_verification_mode_sethood_unprovable_001`; a positive case
  needs bounded library types and is deferred to library-availability.
- Deeper import matrices (aliased qualification, export chains, relative
  paths) remain owned by the resolver crate corpus; this increment adds only
  the branch-import form, alias-conflict, and unknown-module boundaries.
- `@show_*`/`@eval`/ATP-hint annotations stay with SCA-003 (plan step 8).

## Spec Findings (for audit 2)

1. **Empty justification inconsistency.** A.15 admits an empty
   justification, and the parser accepts bare `existence;`/`uniqueness;`,
   but rejects bare `coherence;`, `symmetry;`, `reducibility;`, `sethood;`
   (`malformed_justification`). Either the grammar or the parser must be
   corrected; the corpus uses explicit `proof thus thesis; end;` blocks so
   each case isolates its own target construct.
2. **`sample_codes.md` sketch forms.** The samples write bare obligation
   names as ellipsis ("omitted correctness obligations are shown only by
   name"), which reads as legal syntax but is not decided by the spec text.
3. **Mandatory property-implementation correctness.** A.7 makes
   `existence`/`uniqueness` grammatically mandatory for means-form property
   implementations, so their omission is a *syntax* rejection
   (`fail_parse_only_mode_property_impl_missing_correctness_001`), not a
   type error.
4. **param_prefix rejection layer.** Per A.2, an unbound prefix spelling
   falls back to whole-spelling lexicon matching and is a lexical/parse
   rejection (`fail_type_elaboration_attr_param_prefix_unbound_001` is
   classified `parse_only` accordingly despite its historical file name).

## Chapter Coverage

The tables below are generated from the audit-1 requirement rows in
`tests/coverage/spec_trace.toml` (ids listed in this document's generator).
Existing pre-audit coverage (notably chapters 2, 11, 12 resolver rows and
the 205 active type-elaboration cases) is not restated here.

### Chapter 03 (03.type_system.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.03.types.widening.argument_position` | 3.5 Subtyping and Widening | `type_elaboration` | pass_and_fail | `fail_type_elaboration_argument_type_mismatch_functor_001`<br>`pass_type_elaboration_argument_attribute_widening_001` |

### Chapter 04 (04.variables_and_constants.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.04.variables.inline_deffunc_defpred.semantic` | 4.4 Local Constants (set, reconsider, take, deffunc, defpred) | `formula_statement` | pass | `pass_formula_statement_deffunc_defpred_local_001` |
| `spec.en.04.variables.let_such_that.semantic` | 4.2 Variable Declarations (let) | `formula_statement` | pass | `pass_formula_statement_let_such_that_assumption_001` |
| `spec.en.04.variables.local_constant_set.semantic` | 4.4 Local Constants (set, reconsider, take, deffunc, defpred) | `type_elaboration` | fail | `fail_type_elaboration_set_duplicate_local_constant_001`<br>`fail_type_elaboration_set_forward_reference_001` |
| `spec.en.04.variables.local_constant_set.witness_use` | 4.4 Local Constants (set, reconsider, take, deffunc, defpred) | `formula_statement` | pass | `pass_formula_statement_set_local_constant_take_001` |
| `spec.en.04.variables.reconsider.narrowing_justification` | 4.4 Local Constants (set, reconsider, take, deffunc, defpred) | `type_elaboration` | fail | `fail_type_elaboration_reconsider_unjustified_narrowing_001` |
| `spec.en.04.variables.reconsider.widening` | 4.4 Local Constants (set, reconsider, take, deffunc, defpred) | `formula_statement` | pass | `pass_formula_statement_reconsider_builtin_widening_001` |
| `spec.en.04.variables.reserve.explicit_override` | 4.3 Reserved Variables (reserve) | `type_elaboration` | pass | `pass_type_elaboration_reserve_shadow_explicit_type_001` |
| `spec.en.04.variables.reserve.implicit_typing` | 4.3 Reserved Variables (reserve) | `type_elaboration` | pass_and_fail | `fail_type_elaboration_unreserved_implicit_variable_001`<br>`pass_type_elaboration_reserve_implicit_typing_001` |
| `spec.en.04.variables.scoping.duplicate_generalization` | 4.6 Scoping and Shadowing | `formula_statement` | fail | `fail_formula_statement_duplicate_generalization_001` |
| `spec.en.04.variables.take_exemplification.semantic` | 4.4 Local Constants (set, reconsider, take, deffunc, defpred) | `formula_statement` | fail | `fail_formula_statement_take_non_existential_thesis_001` |

### Chapter 05 (05.structures.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.05.structures.constructor.beta_projection` | 5.5 Constructors | `proof_verification` | pass | `pass_proof_verification_struct_constructor_access_001` |
| `spec.en.05.structures.constructor.field_access` | 5.5 Constructors | `type_elaboration` | fail | `fail_type_elaboration_struct_constructor_missing_field_001` |
| `spec.en.05.structures.definition.fields` | 5.2 Syntax | `type_elaboration` | pass_and_fail | `fail_type_elaboration_struct_duplicate_member_001`<br>`pass_type_elaboration_struct_definition_basic_001` |
| `spec.en.05.structures.definition.properties` | 5.2 Syntax | `type_elaboration` | pass | `pass_type_elaboration_struct_property_member_001` |
| `spec.en.05.structures.dependent.bracket_parameters` | 5.6 Dependent Structures | `type_elaboration` | pass | `pass_type_elaboration_struct_dependent_bracket_params_001` |
| `spec.en.05.structures.inherit.diamond_consistency` | 5.4 Multiple and Diamond Inheritance | `type_elaboration` | pass_and_fail | `fail_type_elaboration_struct_diamond_inconsistent_001`<br>`pass_type_elaboration_struct_diamond_consistent_001` |
| `spec.en.05.structures.inherit.from_set` | 5.3 Inheritance | `type_elaboration` | pass | `pass_type_elaboration_struct_inherit_from_set_001` |
| `spec.en.05.structures.inherit.member_coverage` | 5.3 Inheritance | `type_elaboration` | fail | `fail_type_elaboration_struct_inherit_uncovered_member_001` |
| `spec.en.05.structures.inherit.rename` | 5.3 Inheritance | `type_elaboration` | pass_and_fail | `fail_type_elaboration_struct_inherit_unknown_source_001`<br>`pass_type_elaboration_struct_inherit_rename_001` |
| `spec.en.05.structures.selector.resolution` | 5.7 Field Access (Selectors) | `type_elaboration` | fail | `fail_type_elaboration_struct_unknown_selector_001` |
| `spec.en.05.structures.with_update.semantic` | 5.7 Field Access (Selectors) | `proof_verification` | pass | `pass_proof_verification_struct_with_update_001` |

### Chapter 06 (06.attributes.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.06.attributes.definition.uniqueness` | 6.1 Definition and Purpose | `type_elaboration` | fail | `fail_type_elaboration_attr_duplicate_same_subject_001` |
| `spec.en.06.attributes.disambiguation.struct_qualified` | 6.6 Disambiguation | `type_elaboration` | pass | `pass_type_elaboration_attr_struct_qualified_reference_001` |
| `spec.en.06.attributes.param_prefix.declaration` | 6.2 Syntax | `type_elaboration` | pass | `pass_type_elaboration_attr_param_prefix_declaration_001` |
| `spec.en.06.attributes.param_prefix.lexicon_rejection` | 6.2 Syntax | `parse_only` | fail | `fail_type_elaboration_attr_param_prefix_unbound_001` |
| `spec.en.06.attributes.redefine.coherence` | 6.7 Redefinition | `type_elaboration` | pass | `pass_type_elaboration_attr_redefine_narrower_subject_001` |
| `spec.en.06.attributes.reference.symbol_class` | 6.3 Usage | `type_elaboration` | fail | `fail_type_elaboration_attr_non_attribute_symbol_001` |
| `spec.en.06.attributes.test_chain.negation` | 6.3 Usage | `formula_statement` | pass | `pass_formula_statement_attr_negated_chain_assertion_001` |

### Chapter 07 (07.modes.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.07.modes.attributed_radix.struct` | 7.2 Syntax for Declaring and Using Modes | `type_elaboration` | pass | `pass_type_elaboration_mode_attributed_struct_radix_001` |
| `spec.en.07.modes.dependent.of_parameters` | 7.7 Dependent Modes | `type_elaboration` | pass_and_fail | `fail_type_elaboration_mode_dependent_arity_mismatch_001`<br>`pass_type_elaboration_mode_dependent_of_params_001` |
| `spec.en.07.modes.property_implementation.equals` | 7.4.1 Property Implementation | `type_elaboration` | pass | `pass_type_elaboration_mode_property_impl_equals_001` |
| `spec.en.07.modes.property_implementation.means` | 7.4.1 Property Implementation | `type_elaboration` | pass | `pass_type_elaboration_mode_property_impl_means_001` |
| `spec.en.07.modes.property_implementation.means_missing_correctness` | 7.4.1 Property Implementation | `parse_only` | fail | `fail_parse_only_mode_property_impl_missing_correctness_001` |
| `spec.en.07.modes.property_implementation.subject` | 7.4.1 Property Implementation | `type_elaboration` | fail | `fail_type_elaboration_mode_property_impl_unknown_property_001` |
| `spec.en.07.modes.sethood.property` | 7.8 Correctness Conditions | `proof_verification` | fail | `fail_proof_verification_mode_sethood_unprovable_001` |

### Chapter 09 (09.predicates.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.09.predicates.application.negation` | 9.1 Overview and Syntax | `formula_statement` | pass | `pass_formula_statement_pred_negated_application_001` |
| `spec.en.09.predicates.application.typing` | 9.4 Dependent and Typed Parameters | `type_elaboration` | fail | `fail_type_elaboration_pred_argument_type_mismatch_001` |
| `spec.en.09.predicates.definition.phrase` | 9.3 Definition Styles: Symbolic vs Phrase | `proof_verification` | pass | `pass_proof_verification_pred_phrase_identifier_001` |
| `spec.en.09.predicates.definition.symbolic` | 9.3 Definition Styles: Symbolic vs Phrase | `proof_verification` | pass | `pass_proof_verification_pred_symbolic_infix_001` |
| `spec.en.09.predicates.definition.uniqueness` | 9.8 Symbol Resolution and Imports | `type_elaboration` | fail | `fail_type_elaboration_pred_duplicate_same_signature_001` |
| `spec.en.09.predicates.properties.declaration` | 9.5 Correctness Conditions | `type_elaboration` | pass_and_fail | `fail_type_elaboration_pred_property_arity_mismatch_001`<br>`pass_type_elaboration_pred_properties_declaration_001` |
| `spec.en.09.predicates.redefine.coherence` | 9.6 Predicate Redefinition | `type_elaboration` | pass | `pass_type_elaboration_pred_redefine_narrower_loci_001` |

### Chapter 10 (10.functors.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.10.functors.bracket_application.builtin` | 10.1 Overview and Syntax | `type_elaboration` | pass | `pass_type_elaboration_func_builtin_bracket_pair_001` |
| `spec.en.10.functors.definition.equals` | 10.3 Definition Styles: equals vs means | `type_elaboration` | fail | `fail_type_elaboration_func_equals_result_type_mismatch_001` |
| `spec.en.10.functors.definition.means` | 10.3 Definition Styles: equals vs means | `type_elaboration` | fail | `fail_type_elaboration_func_means_missing_correctness_001` |
| `spec.en.10.functors.definition.uniqueness` | 10.10 Symbol Resolution and Imports | `type_elaboration` | fail | `fail_type_elaboration_func_duplicate_same_signature_001` |
| `spec.en.10.functors.dependent_return.semantic` | 10.5 Dependent Return Types | `type_elaboration` | pass | `pass_type_elaboration_func_dependent_return_type_001` |
| `spec.en.10.functors.equals.definitional_unfolding` | 10.3 Definition Styles: equals vs means | `proof_verification` | pass | `pass_proof_verification_func_equals_infix_operator_001` |
| `spec.en.10.functors.means.definitional_unfolding` | 10.3 Definition Styles: equals vs means | `proof_verification` | pass | `pass_proof_verification_func_means_prefix_001` |
| `spec.en.10.functors.properties.declaration` | 10.6 Correctness Conditions | `type_elaboration` | pass_and_fail | `fail_type_elaboration_func_property_arity_mismatch_001`<br>`pass_type_elaboration_func_commutativity_property_001` |

### Chapter 11 (11.symbol_management.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.11.symbols.antonym.predicate` | 11.1 Synonyms and Antonyms | `type_elaboration` | pass | `pass_type_elaboration_antonym_predicate_001` |
| `spec.en.11.symbols.synonym.functor` | 11.1 Synonyms and Antonyms | `type_elaboration` | pass_and_fail | `fail_type_elaboration_synonym_loci_mismatch_001`<br>`pass_type_elaboration_synonym_functor_001` |

### Chapter 12 (12.modules_and_namespaces.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.12.modules.import.alias` | 12.3 Import Statements | `declaration_symbol` | fail | `fail_declaration_symbol_import_duplicate_alias_001` |
| `spec.en.12.modules.import.branch_form` | 12.3 Import Statements | `declaration_symbol` | pass | `pass_declaration_symbol_branch_import_form_001` |
| `spec.en.12.modules.import.resolution` | 12.3 Import Statements | `declaration_symbol` | fail | `fail_declaration_symbol_import_unknown_module_001` |
| `spec.en.12.modules.visibility.private_local` | 12.5 Visibility Control (Private/Public) | `declaration_symbol` | pass | `pass_declaration_symbol_private_theorem_visibility_001` |

### Chapter 13 (13.term_expression.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.13.terms.choice.inhabited_type` | 13.5 The Choice Operator (the) | `type_elaboration` | pass_and_fail | `fail_type_elaboration_term_choice_uninhabited_001`<br>`pass_type_elaboration_term_choice_builtin_001` |
| `spec.en.13.terms.numeral.typing` | 13.1 Primary Expressions | `type_elaboration` | pass | `pass_type_elaboration_term_numeral_equality_001` |
| `spec.en.13.terms.qua.widening` | 13.6 Type Qualification (qua) | `type_elaboration` | pass_and_fail | `fail_type_elaboration_term_qua_invalid_narrowing_001`<br>`pass_type_elaboration_term_qua_widening_001` |
| `spec.en.13.terms.set_comprehension.guarded` | 13.4 Set Expressions | `type_elaboration` | fail | `fail_type_elaboration_term_comprehension_unbound_mapper_001` |
| `spec.en.13.terms.set_comprehension.membership` | 13.4 Set Expressions | `proof_verification` | pass | `pass_proof_verification_term_comprehension_guarded_001` |
| `spec.en.13.terms.set_enumeration.semantic` | 13.4 Set Expressions | `proof_verification` | pass | `pass_proof_verification_term_set_enumeration_membership_001` |

### Chapter 14 (14.formulas.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.14.formulas.connectives.precedence` | 14.6 Precedence and Associativity | `formula_statement` | pass | `pass_formula_statement_connective_precedence_001` |
| `spec.en.14.formulas.free_variables.binding` | 14.1 Overview | `type_elaboration` | fail | `fail_type_elaboration_formula_unbound_free_variable_001` |
| `spec.en.14.formulas.iff.non_associative` | 14.6 Precedence and Associativity | `parse_only` | fail | `fail_parse_only_iff_unparenthesized_chain_001` |
| `spec.en.14.formulas.iff.parenthesized_form` | 14.6 Precedence and Associativity | `formula_statement` | pass | `pass_formula_statement_iff_parenthesized_001` |
| `spec.en.14.formulas.is_assertion.type` | 14.5 Special Formula Forms | `formula_statement` | pass | `pass_formula_statement_is_type_assertion_001` |
| `spec.en.14.formulas.quantifiers.existential_multi` | 14.4 Quantified Formulas | `formula_statement` | pass | `pass_formula_statement_existential_multi_witness_001` |
| `spec.en.14.formulas.quantifiers.nesting` | 14.4 Quantified Formulas | `formula_statement` | pass | `pass_formula_statement_nested_quantifier_st_holds_001` |

### Chapter 15 (15.statements.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.15.statements.consider.choice` | 15.2 Variable and Constant Introduction | `formula_statement` | pass | `pass_formula_statement_consider_choice_001` |
| `spec.en.15.statements.diffuse.now` | 15.6 Proof Organization | `formula_statement` | pass | `pass_formula_statement_now_diffuse_statement_001` |
| `spec.en.15.statements.given.existential_assumption` | 15.3 Assumptions and Assertions | `formula_statement` | pass | `pass_formula_statement_given_existential_assumption_001` |
| `spec.en.15.statements.hereby.diffuse_conclusion` | 15.4 Conclusions and Derivations | `formula_statement` | pass | `pass_formula_statement_hereby_diffuse_conclusion_001` |
| `spec.en.15.statements.iterative_equality.chain` | 15.7 Iterative Equality | `formula_statement` | pass | `pass_formula_statement_iterative_equality_001` |
| `spec.en.15.statements.linking.then_hence` | 15.4 Conclusions and Derivations | `formula_statement` | pass | `pass_formula_statement_then_hence_linking_001` |
| `spec.en.15.statements.per_cases.completeness_obligation` | 15.6 Proof Organization | `proof_verification` | fail | `fail_proof_verification_per_cases_incomplete_001` |
| `spec.en.15.statements.per_cases.suppose` | 15.6 Proof Organization | `formula_statement` | pass | `pass_formula_statement_per_cases_suppose_001` |

### Chapter 16 (16.theorems_and_proofs.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.16.theorems.reference.lemma_citation` | 16.5 Proof Justification (by) | `proof_verification` | pass | `pass_proof_verification_lemma_reference_001` |
| `spec.en.16.theorems.reference.module_local_label` | 16.5 Proof Justification (by) | `type_elaboration` | fail | `fail_type_elaboration_unknown_reference_label_001` |
| `spec.en.16.theorems.skeleton.thesis_tracking` | 16.3 Proof Skeleton and Natural Deduction | `formula_statement` | fail | `fail_formula_statement_assume_without_antecedent_001`<br>`fail_formula_statement_conclusion_mismatch_001`<br>`fail_formula_statement_incomplete_proof_001` |
| `spec.en.16.theorems.status.open_assumed` | 16.1 Theorem Roles and Status | `proof_verification` | pass | `pass_proof_verification_theorem_status_open_assumed_001` |

### Chapter 17 (17.clusters_and_registrations.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.17.clusters.adjective.restricted_form` | 17.7 Cluster Resolution Rules | `parse_only` | fail | `fail_parse_only_cluster_adjective_argument_list_001` |
| `spec.en.17.clusters.conditional.registration` | 17.4 Conditional Clusters | `advanced_semantics` | pass | `pass_advanced_semantics_conditional_registration_001` |
| `spec.en.17.clusters.existential.registration` | 17.3 Existential Clusters | `advanced_semantics` | pass | `pass_advanced_semantics_existential_registration_001` |
| `spec.en.17.clusters.functorial.false_coherence` | 17.5 Functorial Clusters | `proof_verification` | fail | `fail_proof_verification_functorial_false_coherence_001` |
| `spec.en.17.clusters.functorial.registration` | 17.5 Functorial Clusters | `advanced_semantics` | pass | `pass_advanced_semantics_functorial_registration_001` |
| `spec.en.17.clusters.reduce.false_reducibility` | 17.6 Reduction Registrations | `proof_verification` | fail | `fail_proof_verification_reduce_false_reducibility_001` |
| `spec.en.17.clusters.reduce.registration` | 17.6 Reduction Registrations | `advanced_semantics` | pass | `pass_advanced_semantics_reduce_registration_001` |

### Chapter 18 (18.templates.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.18.templates.instantiation.arity` | 18.2 Template Declarations | `type_elaboration` | fail | `fail_type_elaboration_template_arity_mismatch_001` |
| `spec.en.18.templates.predicate_parameter.declaration` | 18.6 Template for Predicates (pred) | `type_elaboration` | pass | `pass_type_elaboration_template_pred_param_001` |
| `spec.en.18.templates.type_parameter.extends_bound` | 18.2 Template Declarations | `type_elaboration` | pass_and_fail | `fail_type_elaboration_template_bound_violation_001`<br>`pass_type_elaboration_template_extends_bound_001` |
| `spec.en.18.templates.type_parameter.functor` | 18.7 Template for Functors (func) | `type_elaboration` | pass | `pass_type_elaboration_template_type_param_functor_001` |

### Chapter 19 (19.overload_resolution.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.19.overload.resolution.ambiguity` | 19.4 Overload Resolution Algorithm | `advanced_semantics` | fail | `fail_advanced_semantics_overload_ambiguous_candidates_001` |
| `spec.en.19.overload.resolution.distinct_loci` | 19.4 Overload Resolution Algorithm | `advanced_semantics` | pass | `pass_advanced_semantics_overload_distinct_loci_001` |

### Chapter 20 (20.algorithm_and_verification.md)

| Requirement | Section | Stage | Coverage | Cases |
|---|---|---|---|---|
| `spec.en.20.algorithms.claim.block` | 20.6 Snapshot and Claim | `proof_verification` | pass | `pass_proof_verification_claim_block_theorem_001` |
| `spec.en.20.algorithms.computation.justification` | 20.9 Execution Model and Computability | `proof_verification` | pass | `pass_proof_verification_computation_justification_001` |
| `spec.en.20.algorithms.contracts.ensures` | 20.4 Contracts (requires, ensures, assert) | `proof_verification` | pass_and_fail | `fail_proof_verification_algorithm_ensures_unprovable_001`<br>`pass_proof_verification_algorithm_ensures_return_001` |
| `spec.en.20.algorithms.control_flow.loop_scope` | 20.2 Control Flow | `type_elaboration` | fail | `fail_type_elaboration_algorithm_break_outside_loop_001` |
| `spec.en.20.algorithms.ghost.isolation_static` | 20.6 Snapshot and Claim | `type_elaboration` | fail | `fail_type_elaboration_algorithm_ghost_isolation_001` |
| `spec.en.20.algorithms.ghost.snapshot` | 20.6 Snapshot and Claim | `proof_verification` | pass | `pass_proof_verification_algorithm_ghost_snapshot_001` |
| `spec.en.20.algorithms.loops.while_invariant` | 20.5 Loop Verification | `proof_verification` | pass | `pass_proof_verification_algorithm_while_invariant_001` |
| `spec.en.20.algorithms.state.var_const_assert` | 20.1 Algorithm Syntax | `proof_verification` | pass_and_fail | `fail_proof_verification_algorithm_assert_unprovable_001`<br>`pass_proof_verification_algorithm_var_const_assert_001` |

## Follow-Ups

- Audit 2 (2026-09-02) bound each requirement family above to the bounded
  coarse owner tasks in the revised Step 5 of
  [../../todo.md](../../todo.md); the per-case binding, including blocking
  gaps, is the ledger
  [`tests/coverage/step5_activation_map.tsv`](../../../../tests/coverage/step5_activation_map.tsv).
  No case here may be activated by matching expectations to current
  behavior.
- The 29 parse-blocked sources become immediately usable syntax regression
  material for the gap-closure tasks in
  [semantic_bridge_frontend_gaps.md](./semantic_bridge_frontend_gaps.md).
- A committed corpus-wide syntax smoke guard (parse every non-parse_only
  corpus source, assert no syntax diagnostics or record the ledger) is
  recommended once gaps G1-G9 close; the throwaway audit harness used to
  produce these results is reproduced in the gaps document.
