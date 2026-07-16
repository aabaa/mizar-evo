use mizar_checker::binding_env::BindingId;
use mizar_checker::type_checker::{TermFormulaInferenceOutput, TypeExpressionInput};
use mizar_checker::typed_ast::TypedSiteRef;
use mizar_session::SourceRange;

use super::{
    SourceParenthesizedOperandSide, SourceReserveHandoff, SourceReservedVariableBinaryFormula,
    SourceReservedVariableTypeAssertion,
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
