use mizar_checker::binding_env::{BindingId, BindingLookupResult, BindingLookupSite};
use mizar_checker::type_checker::{
    ExpectedTypeInput, FormulaInput, FormulaKind, SourceReserveBindingInput, TermFormulaChecker,
    TermFormulaInferenceOutput, TermInput, TermKind, TermReference, TypeExpressionInput,
    TypeNormalizer,
};
use mizar_checker::typed_ast::{TypeRole, TypedNodeId, TypedSiteRef};
use mizar_resolve::env::SymbolEnv;
use mizar_session::SourceRange;

use super::{
    SourceParenthesizedOperandSide, SourceParenthesizedReservedVariableBinaryFormula,
    SourceReserveHandoff, SourceReservedVariableBinaryFormula, SourceReservedVariableTypeAssertion,
    assemble_source_reserve_checker_handoff, source_binding_matches_reserved_builtin_type,
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
