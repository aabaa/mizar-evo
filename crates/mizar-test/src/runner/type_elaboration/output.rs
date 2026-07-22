use std::collections::{BTreeMap, BTreeSet};

use mizar_checker::binding_env::{
    BindingContextId, BindingId, BindingLookupResult, BindingLookupSite,
};
use mizar_checker::type_checker::{
    ExpectedTypeInput, FormulaDeferredReason, FormulaInput, FormulaKind, FormulaStatus,
    NormalizedTypeStatus, SourceReserveBindingInput, TermFormulaChecker,
    TermFormulaInferenceOutput, TermInput, TermKind, TermReference, TermStatus,
    TypeExpressionInput, TypeHeadInput, TypeNormalizer,
};
use mizar_checker::typed_ast::{
    NormalizedTypeId, TypeEntryActual, TypeEntryId, TypeRole, TypeStatus, TypedNodeId, TypedSiteRef,
};
use mizar_core::core_ir::CoreIr;
use mizar_core::elaborator::lower_exact_task180_handoff;
use mizar_resolve::env::SymbolEnv;
use mizar_resolve::resolved_ast::ModuleId as ResolverModuleId;
use mizar_session::SourceRange;
use mizar_syntax::SurfaceAst;

use super::source_formula::{
    SourceImportedAttributeAssertionFormula, SourceParenthesizedOperandSide,
    SourceParenthesizedReservedVariableBinaryFormula, extract_source_builtin_binary_term_formula,
    extract_source_builtin_type_assertion_formula, extract_source_contradiction_formula,
    extract_source_formula_connective_quantifier, extract_source_formula_statement,
    extract_source_imported_attribute_assertion_formula,
    extract_source_imported_non_empty_attribute_assertion_formula,
    extract_source_imported_predicate_functor_formula, extract_source_set_enumeration_formula,
};
use super::{
    SourceReserveHandoff, SourceReservedVariableBinaryFormula,
    SourceReservedVariableBinaryFormulaConfig, SourceReservedVariableBuiltinType,
    SourceReservedVariableTypeAssertion, assemble_source_contradiction_checker_handoff,
    assemble_source_reserve_checker_handoff, assert_source_contradiction_handoff,
    assert_source_reserve_handoff, has_exact_source_contradiction_owner,
    source_binding_matches_reserved_builtin_type, source_binding_use_ordinals,
    source_mode_terminal_builtin_input, source_module_binding_env,
    source_reserved_variable_asserted_head_relation_is_exact,
    source_reserved_variable_mode_expansions_are_exact,
    source_type_expression_matches_reserved_builtin_type,
};

#[derive(Debug)]
pub(in crate::runner) struct SourceReservedVariableBinaryFormulaOutput {
    pub(in crate::runner) payload: SourceReservedVariableBinaryFormula,
    pub(in crate::runner) handoff: SourceReserveHandoff,
    pub(in crate::runner) left_binding: BindingId,
    pub(in crate::runner) right_binding: BindingId,
    pub(in crate::runner) left_result_input: TypeExpressionInput,
    pub(in crate::runner) right_result_input: TypeExpressionInput,
    pub(in crate::runner) left_expected_input: Option<TypeExpressionInput>,
    pub(in crate::runner) right_expected_input: Option<TypeExpressionInput>,
    pub(in crate::runner) term_formula: TermFormulaInferenceOutput,
}

#[derive(Debug)]
pub(in crate::runner) struct SourceParenthesizedReservedVariableBinaryFormulaOutput {
    pub(in crate::runner) source_wrapper_side: SourceParenthesizedOperandSide,
    pub(in crate::runner) source_wrapper_site: TypedSiteRef,
    pub(in crate::runner) source_wrapper_range: SourceRange,
    pub(in crate::runner) wrapper_side: SourceParenthesizedOperandSide,
    pub(in crate::runner) wrapper_site: TypedSiteRef,
    pub(in crate::runner) wrapper_range: SourceRange,
    pub(in crate::runner) formula: SourceReservedVariableBinaryFormulaOutput,
}

#[derive(Debug)]
pub(in crate::runner) struct SourceReservedVariableTypeAssertionOutput {
    pub(in crate::runner) payload: SourceReservedVariableTypeAssertion,
    pub(in crate::runner) handoff: SourceReserveHandoff,
    pub(in crate::runner) subject_binding: BindingId,
    pub(in crate::runner) subject_result_input: TypeExpressionInput,
    pub(in crate::runner) asserted_type_input: TypeExpressionInput,
    pub(in crate::runner) term_formula: TermFormulaInferenceOutput,
}

pub(in crate::runner) fn build_source_reserved_variable_type_assertion_output(
    payload: SourceReservedVariableTypeAssertion,
    symbols: &SymbolEnv,
) -> Result<SourceReservedVariableTypeAssertionOutput, String> {
    let handoff = assemble_source_reserve_checker_handoff(
        symbols,
        &payload.reserve.bridge,
        payload.reserve.mode_expansions.clone(),
    )?;
    let context = payload.reserve.bridge.module_context();
    let subject_binding = match handoff
        .binding_env
        .lookup(&BindingLookupSite::new(
            payload.subject_spelling.clone(),
            context,
            None,
            payload.subject_lookup_ordinal,
        ))
        .map_err(|error| error.to_string())?
    {
        BindingLookupResult::Local(binding) => binding,
        _ => {
            return Err(
                "reserved-variable type assertion lookup did not resolve locally".to_owned(),
            );
        }
    };
    if subject_binding != BindingId::new(0) {
        return Err("reserved-variable type assertion binding identity mismatch".to_owned());
    }
    let source_binding = payload
        .reserve
        .bridge
        .bindings()
        .get(subject_binding.index())
        .ok_or_else(|| "reserved-variable type assertion source binding missing".to_owned())?;
    if source_binding.spelling != payload.config.binding_spelling
        || !source_binding_matches_reserved_builtin_type(
            source_binding,
            payload.config.binding_type,
            payload.config.binding_source_mode_spelling,
            &payload.reserve.mode_expansions,
        )
    {
        return Err("reserved-variable type assertion source binding mismatch".to_owned());
    }

    let subject_result_input = source_reserved_type_projection(
        source_binding,
        payload.subject_site.node(),
        payload.config.subject_result_role,
    );
    let asserted_type_input = TypeExpressionInput::new(
        payload.asserted_type_site.clone(),
        payload.asserted_type.range,
        payload.asserted_type.spelling.clone(),
        payload.asserted_type.head.clone(),
    )
    .with_attributes(payload.asserted_type.attributes.clone());
    let term_formula =
        TermFormulaChecker::new(TypeNormalizer::new(payload.reserve.mode_expansions.clone()))
            .infer(
                symbols,
                &handoff.binding_env,
                [TermInput::new(
                    payload.subject_site.clone(),
                    context,
                    payload.subject_range,
                    TermKind::Variable,
                )
                .with_reference(TermReference::Binding(subject_binding))
                .with_result_type(subject_result_input.clone())],
                [FormulaInput::new(
                    payload.formula_site.clone(),
                    context,
                    payload.formula_range,
                    FormulaKind::TypeAssertion,
                )
                .with_terms(vec![payload.subject_site.clone()])
                .with_asserted_type(asserted_type_input.clone())],
            );

    Ok(SourceReservedVariableTypeAssertionOutput {
        payload,
        handoff,
        subject_binding,
        subject_result_input,
        asserted_type_input,
        term_formula,
    })
}

pub(in crate::runner) fn build_source_reserved_variable_formula_output(
    payload: SourceReservedVariableBinaryFormula,
    symbols: &SymbolEnv,
) -> Result<SourceReservedVariableBinaryFormulaOutput, String> {
    let handoff = assemble_source_reserve_checker_handoff(
        symbols,
        &payload.reserve.bridge,
        payload.reserve.mode_expansions.clone(),
    )?;

    let context = payload.reserve.bridge.module_context();
    let left_binding = match handoff
        .binding_env
        .lookup(&BindingLookupSite::new(
            payload.left_spelling.clone(),
            context,
            None,
            payload.left_lookup_ordinal,
        ))
        .map_err(|error| error.to_string())?
    {
        BindingLookupResult::Local(binding) => binding,
        _ => {
            return Err("left reserved-variable formula lookup did not resolve locally".to_owned());
        }
    };
    let right_binding = match handoff
        .binding_env
        .lookup(&BindingLookupSite::new(
            payload.right_spelling.clone(),
            context,
            None,
            payload.right_lookup_ordinal,
        ))
        .map_err(|error| error.to_string())?
    {
        BindingLookupResult::Local(binding) => binding,
        _ => {
            return Err(
                "right reserved-variable formula lookup did not resolve locally".to_owned(),
            );
        }
    };
    let expected_left_binding = BindingId::new(payload.config.left_binding_index);
    let expected_right_binding = BindingId::new(payload.config.right_binding_index);
    if left_binding != expected_left_binding || right_binding != expected_right_binding {
        return Err("reserved-variable formula binding identity mismatch".to_owned());
    }
    let left_source_binding = payload
        .reserve
        .bridge
        .bindings()
        .get(left_binding.index())
        .ok_or_else(|| "left reserved-variable formula source binding missing".to_owned())?;
    let right_source_binding = payload
        .reserve
        .bridge
        .bindings()
        .get(right_binding.index())
        .ok_or_else(|| "right reserved-variable formula source binding missing".to_owned())?;
    if left_source_binding.spelling != payload.left_spelling
        || right_source_binding.spelling != payload.right_spelling
    {
        return Err("reserved-variable formula source binding shape mismatch".to_owned());
    }

    let left_result_type = source_reserved_type_projection(
        left_source_binding,
        payload.left_site.node(),
        payload.config.left_result_role,
    );
    let right_result_type = source_reserved_type_projection(
        right_source_binding,
        payload.right_site.node(),
        payload.config.right_result_role,
    );
    let left_result_input = left_result_type.clone();
    let right_result_input = right_result_type.clone();
    let mut expected_types = Vec::new();
    let left_expected_input = payload.config.left_expected_role.map(|role| {
        source_reserved_type_projection(left_source_binding, payload.left_site.node(), role)
    });
    let right_expected_input = payload.config.right_expected_role.map(|role| {
        source_reserved_type_projection(right_source_binding, payload.right_site.node(), role)
    });
    if let Some(expected) = &left_expected_input {
        expected_types.push(ExpectedTypeInput::new(
            payload.left_site.clone(),
            expected.clone(),
            payload.left_range,
        ));
    }
    if let Some(expected) = &right_expected_input {
        expected_types.push(ExpectedTypeInput::new(
            payload.right_site.clone(),
            expected.clone(),
            payload.right_range,
        ));
    }
    let term_formula =
        TermFormulaChecker::new(TypeNormalizer::new(payload.reserve.mode_expansions.clone()))
            .infer(
                symbols,
                &handoff.binding_env,
                [
                    TermInput::new(
                        payload.left_site.clone(),
                        context,
                        payload.left_range,
                        TermKind::Variable,
                    )
                    .with_reference(TermReference::Binding(left_binding))
                    .with_result_type(left_result_type),
                    TermInput::new(
                        payload.right_site.clone(),
                        context,
                        payload.right_range,
                        TermKind::Variable,
                    )
                    .with_reference(TermReference::Binding(right_binding))
                    .with_result_type(right_result_type),
                ],
                [FormulaInput::new(
                    payload.formula_site.clone(),
                    context,
                    payload.formula_range,
                    payload.config.formula_kind,
                )
                .with_terms(vec![payload.left_site.clone(), payload.right_site.clone()])
                .with_expected_types(expected_types)],
            );

    Ok(SourceReservedVariableBinaryFormulaOutput {
        payload,
        handoff,
        left_binding,
        right_binding,
        left_result_input,
        right_result_input,
        left_expected_input,
        right_expected_input,
        term_formula,
    })
}

pub(in crate::runner) fn build_source_parenthesized_reserved_variable_binary_formula_output(
    payload: SourceParenthesizedReservedVariableBinaryFormula,
    symbols: &SymbolEnv,
) -> Result<SourceParenthesizedReservedVariableBinaryFormulaOutput, String> {
    let SourceParenthesizedReservedVariableBinaryFormula {
        wrapper_side,
        wrapper_site,
        wrapper_range,
        formula,
    } = payload;
    let source_wrapper_side = wrapper_side;
    let source_wrapper_site = wrapper_site.clone();
    let source_wrapper_range = wrapper_range;
    let formula = build_source_reserved_variable_formula_output(formula, symbols)?;
    Ok(SourceParenthesizedReservedVariableBinaryFormulaOutput {
        source_wrapper_side,
        source_wrapper_site,
        source_wrapper_range,
        wrapper_side,
        wrapper_site,
        wrapper_range,
        formula,
    })
}

fn source_reserved_type_projection(
    binding: &SourceReserveBindingInput,
    node: TypedNodeId,
    role: &str,
) -> TypeExpressionInput {
    TypeExpressionInput::new(
        TypedSiteRef::Role {
            node,
            role: TypeRole::new(role),
        },
        binding.type_range,
        binding.type_spelling.clone(),
        binding.type_head.clone(),
    )
    .with_attributes(binding.type_attributes.clone())
}

pub(in crate::runner) fn assert_source_reserved_variable_type_assertion_output(
    output: &SourceReservedVariableTypeAssertionOutput,
) -> Result<(), String> {
    let payload = &output.payload;
    let [source_binding] = payload.reserve.bridge.bindings() else {
        return Err("reserved-variable type assertion binding count mismatch".to_owned());
    };
    assert_source_reserve_handoff(&output.handoff, &payload.reserve.bridge)?;
    if source_binding.spelling != payload.config.binding_spelling
        || !source_binding_matches_reserved_builtin_type(
            source_binding,
            payload.config.binding_type,
            payload.config.binding_source_mode_spelling,
            &payload.reserve.mode_expansions,
        )
        || !source_reserved_variable_mode_expansions_are_exact(
            &payload.reserve,
            payload.config.mode_definitions,
        )
        || output.subject_binding != BindingId::new(0)
        || output.handoff.binding_env.bindings().len() != 1
        || !output.handoff.binding_env.diagnostics().is_empty()
        || output.handoff.declarations.declarations().len() != 1
        || !output.handoff.declarations.facts().is_empty()
        || !output.handoff.declarations.diagnostics().is_empty()
    {
        return Err("reserved-variable type assertion handoff mismatch".to_owned());
    }
    let [expected_ordinal] =
        source_binding_use_ordinals(payload.reserve.bridge.bindings(), [payload.subject_range])?;
    if payload.subject_lookup_ordinal != expected_ordinal {
        return Err("reserved-variable type assertion lookup ordinal mismatch".to_owned());
    }
    match output
        .handoff
        .binding_env
        .lookup(&BindingLookupSite::new(
            payload.subject_spelling.clone(),
            payload.reserve.bridge.module_context(),
            None,
            payload.subject_lookup_ordinal,
        ))
        .map_err(|error| error.to_string())?
    {
        BindingLookupResult::Local(binding) if binding == output.subject_binding => {}
        _ => return Err("reserved-variable type assertion lookup result mismatch".to_owned()),
    }

    if output.subject_result_input.site
        != (TypedSiteRef::Role {
            node: payload.subject_site.node(),
            role: TypeRole::new(payload.config.subject_result_role),
        })
        || output.subject_result_input.source_range != source_binding.type_range
        || output.subject_result_input.spelling != source_binding.type_spelling
        || output.subject_result_input.head != source_binding.type_head
        || !output.subject_result_input.args.is_empty()
        || !output.subject_result_input.attributes.is_empty()
        || output.asserted_type_input.site != payload.asserted_type_site
        || output.asserted_type_input.source_range != payload.asserted_type.range
        || output.asserted_type_input.spelling != payload.asserted_type.spelling
        || output.asserted_type_input.head != payload.asserted_type.head
        || !output.asserted_type_input.args.is_empty()
        || !output.asserted_type_input.attributes.is_empty()
        || !source_type_expression_matches_reserved_builtin_type(
            &payload.asserted_type,
            payload.config.asserted_type,
            payload.config.asserted_head_relation.source_mode_spelling(),
            &payload.reserve.mode_expansions,
        )
        || !source_reserved_variable_asserted_head_relation_is_exact(
            source_binding,
            &output.asserted_type_input.spelling,
            &output.asserted_type_input.head,
            payload.config,
            &payload.reserve.mode_expansions,
        )
        || output.asserted_type_input.site == output.subject_result_input.site
        || output.asserted_type_input.source_range == output.subject_result_input.source_range
    {
        return Err("reserved-variable type assertion input provenance mismatch".to_owned());
    }

    let term_formula = &output.term_formula;
    if term_formula.terms().len() != 1
        || term_formula.formulas().len() != 1
        || term_formula.type_entries().len() != 3
        || term_formula.normalized_types().len() != 1
        || !term_formula.candidate_sets().is_empty()
        || !term_formula.facts().is_empty()
        || !term_formula.diagnostics().is_empty()
    {
        return Err("reserved-variable type assertion checker count mismatch".to_owned());
    }
    let term = term_formula
        .terms()
        .iter()
        .map(|(_, term)| term)
        .next()
        .ok_or_else(|| "reserved-variable type assertion subject missing".to_owned())?;
    if term.site != payload.subject_site
        || term.context != payload.reserve.bridge.module_context()
        || term.kind != TermKind::Variable
        || term.reference != Some(TermReference::Binding(output.subject_binding))
        || term.expected_type.is_some()
        || term.candidate_set.is_some()
        || term.status != TermStatus::Inferred
        || !term.deferred.is_empty()
    {
        return Err("reserved-variable type assertion subject mismatch".to_owned());
    }
    let subject_entry = term_formula
        .type_entries()
        .get(term.type_entry)
        .ok_or_else(|| "reserved-variable type assertion term type entry missing".to_owned())?;
    if subject_entry.owner != payload.subject_site
        || subject_entry.expected.is_some()
        || subject_entry.status != TypeStatus::Known
    {
        return Err("reserved-variable type assertion subject type entry mismatch".to_owned());
    }
    let TypeEntryActual::Known(subject_actual) = subject_entry.actual else {
        return Err("reserved-variable type assertion subject type unknown".to_owned());
    };
    let result_role_actual = type_entry_known_actual_for_owner(
        term_formula,
        &output.subject_result_input.site,
        "reserved-variable type assertion result role",
    )?;
    let asserted_role_actual = type_entry_known_actual_for_owner(
        term_formula,
        &output.asserted_type_input.site,
        "reserved-variable type assertion asserted role",
    )?;

    let formula = term_formula
        .formulas()
        .iter()
        .map(|(_, formula)| formula)
        .next()
        .ok_or_else(|| "reserved-variable type assertion formula missing".to_owned())?;
    if formula.site != payload.formula_site
        || formula.context != payload.reserve.bridge.module_context()
        || formula.kind != FormulaKind::TypeAssertion
        || formula.terms != [payload.subject_site.clone()]
        || formula.asserted_type != Some(asserted_role_actual)
        || !formula.expected_types.is_empty()
        || formula.candidate_set.is_some()
        || !formula.facts.is_empty()
        || formula.status != FormulaStatus::Checked
        || !formula.deferred.is_empty()
        || subject_actual != result_role_actual
        || subject_actual != asserted_role_actual
    {
        return Err("reserved-variable type assertion formula mismatch".to_owned());
    }
    let normalized = term_formula
        .normalized_types()
        .get(subject_actual)
        .ok_or_else(|| "reserved-variable type assertion normalized type missing".to_owned())?;
    if !normalized_type_is_reserved_builtin_type(
        term_formula,
        subject_actual,
        payload.config.binding_type,
    ) || !normalized.args.is_empty()
        || !normalized.attributes.positive().is_empty()
        || !normalized.attributes.negative().is_empty()
        || normalized.status != NormalizedTypeStatus::Known
    {
        return Err("reserved-variable type assertion normalized type mismatch".to_owned());
    }
    let canonical_source = if payload.config.binding_source_mode_spelling.is_some() {
        let TypeHeadInput::Symbol(symbol) = &source_binding.type_head else {
            return Err("reserved-variable type assertion mode head missing".to_owned());
        };
        let terminal = source_mode_terminal_builtin_input(
            symbol,
            payload.config.binding_type,
            &payload.reserve.mode_expansions,
        )
        .ok_or_else(|| "reserved-variable type assertion terminal source missing".to_owned())?;
        (terminal.source_range, terminal.spelling.as_str())
    } else {
        (
            source_binding.type_range,
            source_binding.type_spelling.as_str(),
        )
    };
    if normalized.source.range != canonical_source.0
        || normalized.source.spelling != canonical_source.1
    {
        return Err("reserved-variable type assertion canonical source mismatch".to_owned());
    }
    Ok(())
}

pub(in crate::runner) fn source_reserved_variable_type_assertion_result_detail_keys(
    output: Result<SourceReservedVariableTypeAssertionOutput, String>,
    invalid_payload_key: &str,
) -> Vec<String> {
    match output.and_then(|output| {
        assert_source_reserved_variable_type_assertion_output(&output)?;
        Ok(output)
    }) {
        Ok(output) => source_reserved_variable_type_assertion_output_detail_keys(&output),
        Err(_) => vec![invalid_payload_key.to_owned()],
    }
}

fn source_reserved_variable_type_assertion_output_detail_keys(
    output: &SourceReservedVariableTypeAssertionOutput,
) -> Vec<String> {
    let mut keys = output
        .handoff
        .binding_env
        .diagnostics()
        .canonical_iter()
        .map(|(_, diagnostic)| format!("type_elaboration.checker.{}", diagnostic.message_key))
        .chain(
            output
                .handoff
                .declarations
                .diagnostics()
                .canonical_iter()
                .map(|(_, diagnostic)| {
                    format!("type_elaboration.checker.{}", diagnostic.message_key)
                }),
        )
        .chain(
            output
                .term_formula
                .diagnostics()
                .canonical_iter()
                .map(|(_, diagnostic)| {
                    format!("type_elaboration.checker.{}", diagnostic.message_key)
                }),
        )
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys
}

#[rustfmt::skip]
pub(in crate::runner) fn term_formula_output_detail_keys(output: &TermFormulaInferenceOutput) -> Vec<String> {
    let mut keys = output
        .diagnostics()
        .canonical_iter()
        .map(|(_, diagnostic)| format!("type_elaboration.checker.{}", diagnostic.message_key))
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys
}

pub(in crate::runner) fn source_contradiction_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_contradiction_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

pub(in crate::runner) fn source_contradiction_core_ir_snapshot(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<String, String>> {
    extract_source_contradiction_formula(ast)?;
    if !has_exact_source_contradiction_owner(ast.source_id, &module, symbols) {
        return None;
    }
    Some((|| {
        let first = source_contradiction_core_ir(ast, module.clone(), symbols)?;
        let second = source_contradiction_core_ir(ast, module, symbols)?;
        if first != second || first.debug_text() != second.debug_text() {
            return Err("exact Task-180 CoreIr rerun was nondeterministic".to_owned());
        }
        Ok(first.debug_text())
    })())
}

pub(in crate::runner) fn source_contradiction_core_ir(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Result<CoreIr, String> {
    let handoff = assemble_source_contradiction_checker_handoff(ast, module, symbols)?;
    assert_source_contradiction_handoff(&handoff)?;
    lower_exact_task180_handoff(&handoff.resolved).map_err(|error| error.to_string())
}

pub(in crate::runner) fn source_contradiction_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let handoff = assemble_source_contradiction_checker_handoff(ast, module, symbols).ok()?;
    assert_source_contradiction_handoff(&handoff).ok()?;
    Some(handoff.term_formula)
}

pub(in crate::runner) fn source_formula_statement_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_formula_statement_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

pub(in crate::runner) fn source_formula_statement_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_formula_statement(ast)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::Thesis,
        )
        .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload])],
    );
    Some(output)
}

pub(in crate::runner) fn source_builtin_binary_term_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_builtin_binary_term_formula(ast)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [
            TermInput::new(
                payload.left_site.clone(),
                context,
                payload.left_range,
                TermKind::Numeral,
            ),
            TermInput::new(
                payload.right_site.clone(),
                context,
                payload.right_range,
                TermKind::Numeral,
            ),
        ],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            payload.formula_kind,
        )
        .with_terms(vec![payload.left_site, payload.right_site])],
    );
    Some(term_formula_output_detail_keys(&output))
}

pub(in crate::runner) fn source_builtin_type_assertion_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_builtin_type_assertion_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

pub(in crate::runner) fn source_builtin_type_assertion_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_builtin_type_assertion_formula(ast, &module, symbols)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let asserted_type = TypeExpressionInput::new(
        payload.asserted_type_site,
        payload.asserted_type.range,
        payload.asserted_type.spelling,
        payload.asserted_type.head,
    )
    .with_attributes(payload.asserted_type.attributes);
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [TermInput::new(
            payload.subject_site.clone(),
            context,
            payload.subject_range,
            TermKind::Numeral,
        )],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::TypeAssertion,
        )
        .with_terms(vec![payload.subject_site])
        .with_asserted_type(asserted_type)],
    );
    Some(output)
}

pub(in crate::runner) fn source_imported_attribute_assertion_formula_output_from_payload(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
    payload: SourceImportedAttributeAssertionFormula,
) -> Option<TermFormulaInferenceOutput> {
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let _attribute_symbol = payload.attribute_symbol.clone();
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [TermInput::new(
            payload.subject_site.clone(),
            context,
            payload.subject_range,
            TermKind::Numeral,
        )],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::AttributeAssertion,
        )
        .with_terms(vec![payload.subject_site])
        .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload])],
    );
    Some(output)
}

pub(in crate::runner) fn source_imported_attribute_assertion_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_imported_attribute_assertion_formula(ast, &module, symbols)?;
    source_imported_attribute_assertion_formula_output_from_payload(ast, module, symbols, payload)
}

pub(in crate::runner) fn source_imported_attribute_assertion_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_imported_attribute_assertion_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

pub(in crate::runner) fn source_imported_non_empty_attribute_assertion_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload =
        extract_source_imported_non_empty_attribute_assertion_formula(ast, &module, symbols)?;
    source_imported_attribute_assertion_formula_output_from_payload(ast, module, symbols, payload)
}

pub(in crate::runner) fn source_imported_non_empty_attribute_assertion_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output =
        source_imported_non_empty_attribute_assertion_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

pub(in crate::runner) fn source_imported_predicate_functor_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_imported_predicate_functor_formula(ast, &module, symbols)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let _predicate_symbol = payload.predicate_symbol.clone();
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [
            TermInput::new(
                payload.left_site.clone(),
                context,
                payload.left_range,
                TermKind::Numeral,
            ),
            TermInput::new(
                payload.functor_left_site.clone(),
                context,
                payload.functor_left_range,
                TermKind::Numeral,
            ),
            TermInput::new(
                payload.functor_right_site.clone(),
                context,
                payload.functor_right_range,
                TermKind::Numeral,
            ),
            TermInput::new(
                payload.functor_site.clone(),
                context,
                payload.functor_range,
                TermKind::FunctorApplication,
            )
            .with_reference(TermReference::Symbol(payload.functor_symbol)),
        ],
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::PredicateApplication,
        )
        .with_terms(vec![payload.left_site, payload.functor_site])],
    );
    Some(output)
}

pub(in crate::runner) fn source_imported_predicate_functor_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_imported_predicate_functor_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

pub(in crate::runner) fn source_formula_connective_quantifier_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_formula_connective_quantifier(ast, &module, symbols)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        [],
        [
            FormulaInput::new(
                payload.premise_constant_site,
                context,
                payload.premise_constant_range,
                FormulaKind::Contradiction,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]),
            FormulaInput::new(
                payload.implication_site,
                context,
                payload.implication_range,
                FormulaKind::Implication,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]),
            FormulaInput::new(
                payload.quantified_site,
                context,
                payload.quantified_range,
                FormulaKind::Quantified,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingQuantifierPayload]),
            FormulaInput::new(
                payload.negation_site,
                context,
                payload.negation_range,
                FormulaKind::Negation,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]),
            FormulaInput::new(
                payload.body_constant_site,
                context,
                payload.body_constant_range,
                FormulaKind::Contradiction,
            )
            .with_deferred(vec![FormulaDeferredReason::MissingFormulaPayload]),
        ],
    );
    Some(output)
}

pub(in crate::runner) fn source_formula_connective_quantifier_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_formula_connective_quantifier_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

pub(in crate::runner) fn source_set_enumeration_formula_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<TermFormulaInferenceOutput> {
    let payload = extract_source_set_enumeration_formula(ast)?;
    let binding_env = source_module_binding_env(ast, module).ok()?;
    let context = BindingContextId::new(0);
    let mut term_inputs = Vec::new();
    for (site, range) in payload.left_items.iter().chain(payload.right_items.iter()) {
        term_inputs.push(TermInput::new(
            site.clone(),
            context,
            *range,
            TermKind::Numeral,
        ));
    }
    term_inputs.push(TermInput::new(
        payload.left_site.clone(),
        context,
        payload.left_range,
        TermKind::SetEnumeration,
    ));
    term_inputs.push(TermInput::new(
        payload.right_site.clone(),
        context,
        payload.right_range,
        TermKind::SetEnumeration,
    ));
    let output = TermFormulaChecker::default().infer(
        symbols,
        &binding_env,
        term_inputs,
        [FormulaInput::new(
            payload.formula_site,
            context,
            payload.formula_range,
            FormulaKind::Equality,
        )
        .with_terms(vec![payload.left_site, payload.right_site])],
    );
    Some(output)
}

pub(in crate::runner) fn source_set_enumeration_formula_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let output = source_set_enumeration_formula_output(ast, module, symbols)?;
    Some(term_formula_output_detail_keys(&output))
}

fn type_entry_known_actual_for_owner(
    output: &TermFormulaInferenceOutput,
    owner: &TypedSiteRef,
    description: &str,
) -> Result<NormalizedTypeId, String> {
    let (_, entry) = output
        .type_entries()
        .iter()
        .find(|(_, entry)| &entry.owner == owner)
        .ok_or_else(|| format!("{description} type entry missing"))?;
    if entry.expected.is_some() || entry.status != TypeStatus::Known {
        return Err(format!("{description} type entry mismatch"));
    }
    match entry.actual {
        TypeEntryActual::Known(actual) => Ok(actual),
        _ => Err(format!("{description} type entry unknown")),
    }
}

fn normalized_type_is_reserved_builtin_type(
    output: &TermFormulaInferenceOutput,
    id: NormalizedTypeId,
    expected_type: SourceReservedVariableBuiltinType,
) -> bool {
    matches!(
        output.normalized_types().get(id),
        Some(normalized)
            if normalized.head == expected_type.normalized_head()
                && normalized.args.is_empty()
                && normalized.attributes.positive().is_empty()
                && normalized.attributes.negative().is_empty()
                && normalized.status == NormalizedTypeStatus::Known
    )
}

pub(in crate::runner) fn assert_source_reserved_variable_formula_output(
    output: &SourceReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    let payload = &output.payload;
    let source_bindings = payload.reserve.bridge.bindings();
    if source_bindings.len() != payload.config.binding_spellings.len()
        || source_bindings.len() != payload.config.binding_types.len()
        || source_bindings.len() != payload.config.binding_source_mode_spellings.len()
    {
        return Err("reserved-variable formula binding count mismatch".to_owned());
    }
    assert_source_reserve_handoff(&output.handoff, &payload.reserve.bridge)?;
    if source_bindings.iter().enumerate().any(|(index, binding)| {
        let spelling = payload.config.binding_spellings[index];
        binding.spelling != spelling
            || !source_binding_matches_reserved_builtin_type(
                binding,
                payload.config.binding_types[index],
                payload.config.binding_source_mode_spellings[index],
                &payload.reserve.mode_expansions,
            )
    }) || !source_reserved_variable_mode_expansions_are_exact(
        &payload.reserve,
        payload.config.mode_definitions,
    ) || (payload.config.require_shared_type_range
        && source_bindings
            .windows(2)
            .any(|pair| pair[0].type_range != pair[1].type_range))
        || (payload.config.require_distinct_type_ranges
            && source_bindings.windows(2).any(|pair| {
                pair[0].type_range == pair[1].type_range
                    || (pair[0].type_range.start, pair[0].type_range.end)
                        >= (pair[1].type_range.start, pair[1].type_range.end)
            }))
        || output.handoff.binding_env.bindings().len() != source_bindings.len()
        || !output.handoff.binding_env.diagnostics().is_empty()
        || output
            .handoff
            .binding_env
            .bindings()
            .iter()
            .any(|(_, binding)| !binding.diagnostics.is_empty())
        || output.handoff.declarations.declarations().len() != source_bindings.len()
        || !output.handoff.declarations.facts().is_empty()
        || !output.handoff.declarations.diagnostics().is_empty()
    {
        return Err("reserved-variable formula declaration payload mismatch".to_owned());
    }

    let expected_ordinals = source_binding_use_ordinals(
        payload.reserve.bridge.bindings(),
        [payload.left_range, payload.right_range],
    )?;
    let expected_left_binding = BindingId::new(payload.config.left_binding_index);
    let expected_right_binding = BindingId::new(payload.config.right_binding_index);
    if [payload.left_lookup_ordinal, payload.right_lookup_ordinal] != expected_ordinals
        || output.left_binding != expected_left_binding
        || output.right_binding != expected_right_binding
    {
        return Err("reserved-variable formula lookup metadata mismatch".to_owned());
    }
    for (spelling, ordinal, expected_binding) in [
        (
            payload.left_spelling.as_str(),
            payload.left_lookup_ordinal,
            output.left_binding,
        ),
        (
            payload.right_spelling.as_str(),
            payload.right_lookup_ordinal,
            output.right_binding,
        ),
    ] {
        match output
            .handoff
            .binding_env
            .lookup(&BindingLookupSite::new(
                spelling,
                payload.reserve.bridge.module_context(),
                None,
                ordinal,
            ))
            .map_err(|error| error.to_string())?
        {
            BindingLookupResult::Local(binding) if binding == expected_binding => {}
            _ => return Err("reserved-variable formula lookup result mismatch".to_owned()),
        }
    }

    for (input, source_binding, node, role) in [
        (
            &output.left_result_input,
            &source_bindings[payload.config.left_binding_index],
            payload.left_site.node(),
            payload.config.left_result_role,
        ),
        (
            &output.right_result_input,
            &source_bindings[payload.config.right_binding_index],
            payload.right_site.node(),
            payload.config.right_result_role,
        ),
    ] {
        if !source_type_projection_matches(input, source_binding, node, role) {
            return Err("reserved-variable formula result input provenance mismatch".to_owned());
        }
    }
    for (input, source_binding, node, role) in [
        (
            output.left_expected_input.as_ref(),
            &source_bindings[payload.config.left_binding_index],
            payload.left_site.node(),
            payload.config.left_expected_role,
        ),
        (
            output.right_expected_input.as_ref(),
            &source_bindings[payload.config.right_binding_index],
            payload.right_site.node(),
            payload.config.right_expected_role,
        ),
    ] {
        match (input, role) {
            (Some(input), Some(role))
                if source_type_projection_matches(input, source_binding, node, role) => {}
            (None, None) => {}
            _ => {
                return Err(
                    "reserved-variable formula expected input provenance mismatch".to_owned(),
                );
            }
        }
    }

    let term_formula = &output.term_formula;
    let expected_type_count = usize::from(payload.config.left_expected_role.is_some())
        + usize::from(payload.config.right_expected_role.is_some());
    if term_formula.terms().len() != 2
        || term_formula.formulas().len() != 1
        || !term_formula.candidate_sets().is_empty()
        || !term_formula.facts().is_empty()
        || !term_formula.diagnostics().is_empty()
        || term_formula.type_entries().len() != 4 + expected_type_count
    {
        return Err(format!(
            "reserved-variable formula checker count mismatch: terms={} formulas={} candidates={} facts={} diagnostics={} type_entries={} expected_type_entries={}",
            term_formula.terms().len(),
            term_formula.formulas().len(),
            term_formula.candidate_sets().len(),
            term_formula.facts().len(),
            term_formula.diagnostics().len(),
            term_formula.type_entries().len(),
            4 + expected_type_count,
        ));
    }
    let mut term_actuals = BTreeMap::new();
    let mut semantic_ids_by_type = BTreeMap::new();
    for (site, binding, binding_index) in [
        (
            &payload.left_site,
            output.left_binding,
            payload.config.left_binding_index,
        ),
        (
            &payload.right_site,
            output.right_binding,
            payload.config.right_binding_index,
        ),
    ] {
        let term = term_formula
            .terms()
            .iter()
            .map(|(_, term)| term)
            .find(|term| &term.site == site)
            .ok_or_else(|| "reserved-variable formula term missing".to_owned())?;
        if term.context != payload.reserve.bridge.module_context()
            || term.kind != TermKind::Variable
            || term.reference != Some(TermReference::Binding(binding))
            || term.expected_type.is_some()
            || term.candidate_set.is_some()
            || term.status != TermStatus::Inferred
            || !term.deferred.is_empty()
        {
            return Err("reserved-variable formula term payload mismatch".to_owned());
        }
        let expected_type = payload.config.binding_types[binding_index];
        let actual = assert_reserved_variable_builtin_type_entry(
            term_formula,
            &term.site,
            Some(term.type_entry),
            expected_type,
        )?;
        if semantic_ids_by_type
            .insert(expected_type, actual)
            .is_some_and(|existing| existing != actual)
        {
            return Err("reserved-variable formula semantic type identity mismatch".to_owned());
        }
        term_actuals.insert(term.site.clone(), actual);
    }

    let formula = term_formula
        .formulas()
        .iter()
        .map(|(_, formula)| formula)
        .next()
        .ok_or_else(|| "reserved-variable formula missing".to_owned())?;
    if formula.site != payload.formula_site
        || formula.context != payload.reserve.bridge.module_context()
        || formula.kind != payload.config.formula_kind
        || formula.terms != [payload.left_site.clone(), payload.right_site.clone()]
        || formula.asserted_type.is_some()
        || formula.candidate_set.is_some()
        || formula.status != FormulaStatus::Checked
        || !formula.facts.is_empty()
        || !formula.deferred.is_empty()
        || formula.expected_types.len() != expected_type_count
    {
        return Err("reserved-variable formula payload mismatch".to_owned());
    }
    let expected_constraints = [
        payload.config.left_expected_role.map(|role| {
            (
                &payload.left_site,
                payload.left_range,
                role,
                payload.config.left_binding_index,
            )
        }),
        payload.config.right_expected_role.map(|role| {
            (
                &payload.right_site,
                payload.right_range,
                role,
                payload.config.right_binding_index,
            )
        }),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    for (constraint, (site, range, role, binding_index)) in formula
        .expected_types
        .iter()
        .zip(expected_constraints.iter().copied())
    {
        let expected_type = payload.config.binding_types[binding_index];
        if &constraint.term != site
            || constraint.source_range != range
            || constraint.status != TypeStatus::Known
            || !normalized_type_is_reserved_builtin_type(
                term_formula,
                constraint.expected,
                expected_type,
            )
        {
            return Err("reserved-variable formula expected type mismatch".to_owned());
        }
        let owner = TypedSiteRef::Role {
            node: site.node(),
            role: TypeRole::new(role),
        };
        let role_actual =
            assert_reserved_variable_builtin_type_entry(term_formula, &owner, None, expected_type)?;
        if role_actual != constraint.expected || term_actuals.get(site) != Some(&role_actual) {
            return Err("reserved-variable expected role is not referenced".to_owned());
        }
    }
    for (site, role, binding_index) in [
        (
            &payload.left_site,
            payload.config.left_result_role,
            payload.config.left_binding_index,
        ),
        (
            &payload.right_site,
            payload.config.right_result_role,
            payload.config.right_binding_index,
        ),
    ] {
        let owner = TypedSiteRef::Role {
            node: site.node(),
            role: TypeRole::new(role),
        };
        let role_actual = assert_reserved_variable_builtin_type_entry(
            term_formula,
            &owner,
            None,
            payload.config.binding_types[binding_index],
        )?;
        if term_actuals.get(site) != Some(&role_actual) {
            return Err("reserved-variable result role is not referenced".to_owned());
        }
    }
    let expected_semantic_type_count = payload
        .config
        .binding_types
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .len();
    if semantic_ids_by_type.len() != expected_semantic_type_count
        || term_formula.normalized_types().len() != expected_semantic_type_count
    {
        return Err("reserved-variable formula semantic type identity mismatch".to_owned());
    }
    for (expected_type, semantic_id) in semantic_ids_by_type {
        let canonical_source = source_bindings
            .iter()
            .enumerate()
            .filter(|(index, _)| payload.config.binding_types[*index] == expected_type)
            .filter_map(|(index, binding)| {
                let Some(_) = payload.config.binding_source_mode_spellings[index] else {
                    return Some((binding.type_range, binding.type_spelling.as_str()));
                };
                let TypeHeadInput::Symbol(symbol) = &binding.type_head else {
                    return None;
                };
                source_mode_terminal_builtin_input(
                    symbol,
                    expected_type,
                    &payload.reserve.mode_expansions,
                )
                .map(|terminal| (terminal.source_range, terminal.spelling.as_str()))
            })
            .min_by_key(|(range, _)| (range.start, range.end))
            .ok_or_else(|| "reserved-variable formula canonical source missing".to_owned())?;
        let normalized = term_formula
            .normalized_types()
            .get(semantic_id)
            .ok_or_else(|| "reserved-variable formula normalized type missing".to_owned())?;
        if normalized.source.range != canonical_source.0
            || normalized.source.spelling != canonical_source.1
        {
            return Err("reserved-variable formula canonical source mismatch".to_owned());
        }
    }
    Ok(())
}

pub(in crate::runner) fn source_reserved_variable_formula_result_detail_keys(
    output: Result<SourceReservedVariableBinaryFormulaOutput, String>,
    invalid_payload_key: &str,
) -> Vec<String> {
    match output {
        Ok(output) => source_reserved_variable_formula_output_detail_keys(&output),
        Err(_) => vec![invalid_payload_key.to_owned()],
    }
}

pub(in crate::runner) fn source_reserved_variable_formula_output_detail_keys(
    output: &SourceReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    if assert_source_reserved_variable_formula_output(output).is_err() {
        return vec![output.payload.config.invalid_payload_key.to_owned()];
    }
    let mut keys = output
        .handoff
        .declarations
        .diagnostics()
        .canonical_iter()
        .map(|(_, diagnostic)| format!("type_elaboration.checker.{}", diagnostic.message_key))
        .chain(
            output
                .term_formula
                .diagnostics()
                .canonical_iter()
                .map(|(_, diagnostic)| {
                    format!("type_elaboration.checker.{}", diagnostic.message_key)
                }),
        )
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys
}

fn source_type_projection_matches(
    input: &TypeExpressionInput,
    source_binding: &SourceReserveBindingInput,
    node: TypedNodeId,
    role: &str,
) -> bool {
    input.site
        == (TypedSiteRef::Role {
            node,
            role: TypeRole::new(role),
        })
        && input.source_range == source_binding.type_range
        && input.spelling == source_binding.type_spelling
        && input.head == source_binding.type_head
        && input.args.is_empty()
        && input.attributes == source_binding.type_attributes
}

fn assert_reserved_variable_builtin_type_entry(
    output: &TermFormulaInferenceOutput,
    owner: &TypedSiteRef,
    expected_id: Option<TypeEntryId>,
    expected_type: SourceReservedVariableBuiltinType,
) -> Result<NormalizedTypeId, String> {
    let (id, entry) = output
        .type_entries()
        .iter()
        .find(|(_, entry)| &entry.owner == owner)
        .ok_or_else(|| "reserved-variable equality type entry missing".to_owned())?;
    if expected_id.is_some_and(|expected| expected != id)
        || entry.expected.is_some()
        || entry.status != TypeStatus::Known
    {
        return Err("reserved-variable equality type entry mismatch".to_owned());
    }
    let TypeEntryActual::Known(actual) = entry.actual else {
        return Err("reserved-variable equality type entry is not known".to_owned());
    };
    if !normalized_type_is_reserved_builtin_type(output, actual, expected_type) {
        return Err("reserved-variable equality normalized type mismatch".to_owned());
    }
    Ok(actual)
}

pub(in crate::runner) fn assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
    config: &'static SourceReservedVariableBinaryFormulaConfig,
    expected_side: SourceParenthesizedOperandSide,
) -> Result<(), String> {
    assert_source_reserved_variable_formula_output(&output.formula)?;
    let payload = &output.formula.payload;
    let wrapper_range_is_valid = match expected_side {
        SourceParenthesizedOperandSide::Left => {
            output.wrapper_range.start < payload.left_range.start
                && output.wrapper_range.end > payload.left_range.end
                && output.wrapper_range.end <= payload.right_range.start
                && payload.formula_range.start <= output.wrapper_range.start
                && payload.formula_range.end >= payload.right_range.end
        }
        SourceParenthesizedOperandSide::Right => {
            payload.left_range.end <= output.wrapper_range.start
                && output.wrapper_range.start < payload.right_range.start
                && output.wrapper_range.end > payload.right_range.end
                && payload.formula_range.start <= payload.left_range.start
                && payload.formula_range.end >= output.wrapper_range.end
        }
    };
    if !std::ptr::eq(payload.config, config)
        || output.source_wrapper_side != expected_side
        || output.wrapper_side != output.source_wrapper_side
        || output.wrapper_site != output.source_wrapper_site
        || output.wrapper_range != output.source_wrapper_range
        || output.wrapper_site == payload.formula_site
        || output.wrapper_site == payload.left_site
        || output.wrapper_site == payload.right_site
        || payload.formula_site == payload.left_site
        || payload.formula_site == payload.right_site
        || payload.left_site == payload.right_site
        || output.wrapper_range.source_id != payload.left_range.source_id
        || output.wrapper_range.source_id != payload.right_range.source_id
        || output.wrapper_range.source_id != payload.formula_range.source_id
        || !wrapper_range_is_valid
        || output
            .formula
            .term_formula
            .terms()
            .iter()
            .any(|(_, term)| term.site.node() == output.wrapper_site.node())
        || output
            .formula
            .term_formula
            .type_entries()
            .iter()
            .any(|(_, entry)| entry.owner.node() == output.wrapper_site.node())
        || output
            .formula
            .term_formula
            .formulas()
            .iter()
            .any(|(_, formula)| {
                formula.site.node() == output.wrapper_site.node()
                    || formula
                        .terms
                        .iter()
                        .any(|term| term.node() == output.wrapper_site.node())
            })
    {
        return Err("parenthesized reserved-variable binary formula wrapper mismatch".to_owned());
    }
    Ok(())
}

pub(in crate::runner) fn source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
    config: &'static SourceReservedVariableBinaryFormulaConfig,
    expected_side: SourceParenthesizedOperandSide,
) -> Vec<String> {
    if assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        config,
        expected_side,
    )
    .is_err()
    {
        return vec![config.invalid_payload_key.to_owned()];
    }
    source_reserved_variable_formula_output_detail_keys(&output.formula)
}

pub(in crate::runner) fn source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
    payload: SourceParenthesizedReservedVariableBinaryFormula,
    symbols: &SymbolEnv,
    config: &'static SourceReservedVariableBinaryFormulaConfig,
    expected_side: SourceParenthesizedOperandSide,
) -> Vec<String> {
    match build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols) {
        Ok(output) => {
            source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
                &output,
                config,
                expected_side,
            )
        }
        Err(_) => vec![config.invalid_payload_key.to_owned()],
    }
}
