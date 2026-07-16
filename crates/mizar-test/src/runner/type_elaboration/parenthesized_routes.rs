use mizar_checker::type_checker::FormulaKind;
use mizar_resolve::env::SymbolEnv;
use mizar_resolve::resolved_ast::ModuleId as ResolverModuleId;
use mizar_syntax::SurfaceAst;

use super::output::source_parenthesized_reserved_variable_binary_formula_payload_detail_keys;
#[cfg(test)]
use super::output::{
    SourceParenthesizedReservedVariableBinaryFormulaOutput,
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config,
    build_source_parenthesized_reserved_variable_binary_formula_output,
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config,
};
use super::source_formula::{
    SourceParenthesizedOperandSide, SourceParenthesizedReservedVariableBinaryFormula,
    SourceReservedVariableBinaryFormulaConfig, SourceReservedVariableBuiltinType,
    SourceReservedVariableModeDefinition, SourceReservedVariableModeRadix,
    extract_source_parenthesized_reserved_variable_binary_formula_with_config,
};

const TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_heterogeneous_reserve_membership.invalid_payload";
const TYPE_ELABORATION_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.right_parenthesized_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_two_edge_local_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_reserved_object_variable_equality.invalid_payload";
const TYPE_ELABORATION_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.parenthesized_reserved_object_variable_inequality.invalid_payload";

pub(in crate::runner) static SOURCE_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-reserved-variable-left-result",
    right_result_role: "parenthesized-reserved-variable-right-result",
    left_expected_role: Some("parenthesized-reserved-variable-left-expected"),
    right_expected_role: Some("parenthesized-reserved-variable-right-expected"),
};

pub(in crate::runner) static SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-reserved-variable-inequality-left-result",
    right_result_role: "parenthesized-reserved-variable-inequality-right-result",
    left_expected_role: Some("parenthesized-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("parenthesized-reserved-variable-inequality-right-expected"),
};

pub(in crate::runner) static SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-reserved-variable-membership-left-result",
    right_result_role: "parenthesized-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("parenthesized-reserved-variable-membership-right-expected"),
};

pub(in crate::runner) static SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedHeterogeneousReserveMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "parenthesized-heterogeneous-reserve-membership-left-result",
    right_result_role: "parenthesized-heterogeneous-reserve-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("parenthesized-heterogeneous-reserve-membership-right-expected"),
};

pub(in crate::runner) static SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "RightParenthesizedReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "right-parenthesized-reserved-variable-membership-left-result",
    right_result_role: "right-parenthesized-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("right-parenthesized-reserved-variable-membership-right-expected"),
};

pub(in crate::runner) static SOURCE_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedTwoEdgeLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeModeEqualityDef",
            spelling: "BaseTwoEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Set,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeModeEqualityDef",
            spelling: "MiddleTwoEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeModeEqualityDef",
            spelling: "OuterTwoEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-two-edge-local-mode-reserved-variable-equality-left-result",
    right_result_role: "parenthesized-two-edge-local-mode-reserved-variable-equality-right-result",
    left_expected_role: Some(
        "parenthesized-two-edge-local-mode-reserved-variable-equality-left-expected",
    ),
    right_expected_role: Some(
        "parenthesized-two-edge-local-mode-reserved-variable-equality-right-expected",
    ),
};

pub(in crate::runner) static SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedReservedObjectVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-reserved-object-variable-left-result",
    right_result_role: "parenthesized-reserved-object-variable-right-result",
    left_expected_role: Some("parenthesized-reserved-object-variable-left-expected"),
    right_expected_role: Some("parenthesized-reserved-object-variable-right-expected"),
};

pub(in crate::runner) static SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ParenthesizedReservedObjectVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "parenthesized-reserved-object-variable-inequality-left-result",
    right_result_role: "parenthesized-reserved-object-variable-inequality-right-result",
    left_expected_role: Some("parenthesized-reserved-object-variable-inequality-left-expected"),
    right_expected_role: Some("parenthesized-reserved-object-variable-inequality-right-expected"),
};

pub(in crate::runner) fn source_parenthesized_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_parenthesized_reserved_variable_equality(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

pub(in crate::runner) fn source_parenthesized_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_parenthesized_reserved_variable_inequality(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

pub(in crate::runner) fn source_parenthesized_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_parenthesized_reserved_variable_membership(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

pub(in crate::runner) fn source_parenthesized_heterogeneous_reserve_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_parenthesized_heterogeneous_reserve_membership(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

pub(in crate::runner) fn source_right_parenthesized_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_right_parenthesized_reserved_variable_membership(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
            SourceParenthesizedOperandSide::Right,
        ),
    )
}

pub(in crate::runner) fn source_parenthesized_two_edge_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

pub(in crate::runner) fn source_parenthesized_reserved_object_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_parenthesized_reserved_object_variable_equality(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

pub(in crate::runner) fn source_parenthesized_reserved_object_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_parenthesized_reserved_object_variable_inequality(ast, module, symbols)?;
    Some(
        source_parenthesized_reserved_variable_binary_formula_payload_detail_keys(
            payload,
            symbols,
            &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
            SourceParenthesizedOperandSide::Left,
        ),
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_reserved_variable_equality_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_reserved_variable_inequality_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_reserved_variable_membership_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_heterogeneous_reserve_membership_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_right_parenthesized_reserved_variable_membership_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Right,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_two_edge_local_mode_reserved_variable_equality_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_reserved_object_variable_equality_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_reserved_object_variable_inequality_output_detail_keys(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Vec<String> {
    source_parenthesized_reserved_variable_binary_formula_output_detail_keys_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_parenthesized_reserved_variable_equality(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_parenthesized_reserved_variable_inequality(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_parenthesized_reserved_variable_membership(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_heterogeneous_reserve_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_parenthesized_heterogeneous_reserve_membership(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_right_parenthesized_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_right_parenthesized_reserved_variable_membership(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_two_edge_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_reserved_object_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_parenthesized_reserved_object_variable_equality(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_parenthesized_reserved_object_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_parenthesized_reserved_object_variable_inequality(ast, module, symbols)?;
    build_source_parenthesized_reserved_variable_binary_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn assert_source_parenthesized_reserved_variable_equality_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn assert_source_parenthesized_reserved_variable_inequality_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn assert_source_parenthesized_reserved_variable_membership_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn assert_source_parenthesized_heterogeneous_reserve_membership_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn assert_source_right_parenthesized_reserved_variable_membership_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Right,
    )
}

#[cfg(test)]
pub(in crate::runner) fn assert_source_parenthesized_two_edge_local_mode_reserved_variable_equality_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn assert_source_parenthesized_reserved_object_variable_equality_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

#[cfg(test)]
pub(in crate::runner) fn assert_source_parenthesized_reserved_object_variable_inequality_output(
    output: &SourceParenthesizedReservedVariableBinaryFormulaOutput,
) -> Result<(), String> {
    assert_source_parenthesized_reserved_variable_binary_formula_output_with_config(
        output,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

pub(in crate::runner) fn extract_source_parenthesized_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

pub(in crate::runner) fn extract_source_parenthesized_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

pub(in crate::runner) fn extract_source_parenthesized_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

pub(in crate::runner) fn extract_source_parenthesized_heterogeneous_reserve_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

pub(in crate::runner) fn extract_source_right_parenthesized_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_RIGHT_PARENTHESIZED_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
        SourceParenthesizedOperandSide::Right,
    )
}

pub(in crate::runner) fn extract_source_parenthesized_two_edge_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

pub(in crate::runner) fn extract_source_parenthesized_reserved_object_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}

pub(in crate::runner) fn extract_source_parenthesized_reserved_object_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceParenthesizedReservedVariableBinaryFormula> {
    extract_source_parenthesized_reserved_variable_binary_formula_with_config(
        ast,
        module,
        symbols,
        &SOURCE_PARENTHESIZED_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
        SourceParenthesizedOperandSide::Left,
    )
}
