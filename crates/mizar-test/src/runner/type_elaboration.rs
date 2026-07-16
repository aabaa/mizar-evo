mod admission;
mod checker_handoff;
mod output;
mod result;
mod source_ast;
mod source_formula;
mod source_reserve;

pub(super) use admission::{is_active_type_elaboration, validate_active_type_elaboration_tags};
#[cfg(test)]
pub(super) use checker_handoff::assemble_source_checker_handoff;
pub(super) use checker_handoff::{
    SourceReserveHandoff, assemble_source_reserve_checker_handoff,
    assert_source_reserve_core_context_readiness, assert_source_reserve_core_summary_readiness,
    assert_source_reserve_handoff, source_module_binding_env,
};
pub(super) use output::{
    SourceParenthesizedReservedVariableBinaryFormulaOutput,
    SourceReservedVariableBinaryFormulaOutput, SourceReservedVariableTypeAssertionOutput,
    build_source_parenthesized_reserved_variable_binary_formula_output,
    build_source_reserved_variable_formula_output,
    build_source_reserved_variable_type_assertion_output,
};
pub(super) use result::{
    expected_type_elaboration_detail_keys, type_elaboration_failure_diagnostic,
};
#[cfg(test)]
pub(super) use source_ast::{
    direct_token_texts, structural_child_ids, surface_nodes_with_kind, surface_site,
};
#[cfg(test)]
pub(super) use source_formula::SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS;
pub(super) use source_formula::{
    SourceImportedAttributeAssertionFormula, SourceParenthesizedOperandSide,
    SourceParenthesizedReservedVariableBinaryFormula, SourceReservedVariableAssertedHeadRelation,
    SourceReservedVariableBinaryFormula, SourceReservedVariableBinaryFormulaConfig,
    SourceReservedVariableBuiltinType, SourceReservedVariableModeDefinition,
    SourceReservedVariableModeRadix, SourceReservedVariableTypeAssertion,
    SourceReservedVariableTypeAssertionConfig, extract_source_builtin_binary_term_formula,
    extract_source_builtin_type_assertion_formula, extract_source_contradiction_formula,
    extract_source_formula_connective_quantifier, extract_source_formula_statement,
    extract_source_imported_attribute_assertion_formula,
    extract_source_imported_non_empty_attribute_assertion_formula,
    extract_source_imported_predicate_functor_formula,
    extract_source_parenthesized_reserved_variable_binary_formula_with_config,
    extract_source_reserved_variable_binary_formula,
    extract_source_reserved_variable_type_assertion_with_config,
    extract_source_set_enumeration_formula, source_binding_matches_reserved_builtin_type,
    source_binding_use_ordinals, source_mode_terminal_builtin_input,
    source_reserved_variable_asserted_head_relation_is_exact,
    source_reserved_variable_mode_expansions_are_exact,
    source_type_expression_matches_reserved_builtin_type,
};
pub(super) use source_reserve::extract_builtin_source_reserve_declarations;
#[cfg(test)]
pub(super) use source_reserve::extract_builtin_source_reserve_declarations_after_node_guard;
#[cfg(test)]
pub(super) use source_reserve::{
    resolve_visible_attribute, resolve_visible_type_head, source_mode_symbol_spelling,
};
