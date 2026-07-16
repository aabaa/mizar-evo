use mizar_checker::type_checker::FormulaKind;
use mizar_resolve::env::SymbolEnv;
use mizar_resolve::resolved_ast::ModuleId as ResolverModuleId;
use mizar_syntax::SurfaceAst;

#[cfg(test)]
use super::output::SourceReservedVariableBinaryFormulaOutput;
use super::output::{
    build_source_reserved_variable_formula_output,
    source_reserved_variable_formula_result_detail_keys,
};
use super::source_formula::{
    SourceReservedVariableBinaryFormula, SourceReservedVariableBinaryFormulaConfig,
    SourceReservedVariableBuiltinType, extract_source_reserved_variable_binary_formula,
};

const TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.distinct_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.distinct_reserved_object_variable_equality.invalid_payload";
const TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.distinct_reserved_object_variable_inequality.invalid_payload";
const TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.distinct_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.distinct_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.heterogeneous_reserve_membership.invalid_payload";
const TYPE_ELABORATION_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_object_variable_equality.invalid_payload";
const TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_object_variable_inequality.invalid_payload";
pub(in crate::runner) const SOURCE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key: TYPE_ELABORATION_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "reserved-variable-left-result",
    right_result_role: "reserved-variable-right-result",
    left_expected_role: Some("reserved-variable-left-expected"),
    right_expected_role: Some("reserved-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ReservedObjectVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key: TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "reserved-object-variable-left-result",
    right_result_role: "reserved-object-variable-right-result",
    left_expected_role: Some("reserved-object-variable-left-expected"),
    right_expected_role: Some("reserved-object-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "DistinctReservedObjectVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Object,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: true,
    require_distinct_type_ranges: false,
    left_result_role: "distinct-reserved-object-variable-left-result",
    right_result_role: "distinct-reserved-object-variable-right-result",
    left_expected_role: Some("distinct-reserved-object-variable-left-expected"),
    right_expected_role: Some("distinct-reserved-object-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "DistinctReservedObjectVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Object,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: true,
    require_distinct_type_ranges: false,
    left_result_role: "distinct-reserved-object-variable-inequality-left-result",
    right_result_role: "distinct-reserved-object-variable-inequality-right-result",
    left_expected_role: Some("distinct-reserved-object-variable-inequality-left-expected"),
    right_expected_role: Some("distinct-reserved-object-variable-inequality-right-expected"),
};

pub(in crate::runner) const SOURCE_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ReservedObjectVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key: TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "reserved-object-variable-inequality-left-result",
    right_result_role: "reserved-object-variable-inequality-right-result",
    left_expected_role: Some("reserved-object-variable-inequality-left-expected"),
    right_expected_role: Some("reserved-object-variable-inequality-right-expected"),
};

pub(in crate::runner) const SOURCE_DISTINCT_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "DistinctReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key: TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: true,
    require_distinct_type_ranges: false,
    left_result_role: "distinct-reserved-variable-left-result",
    right_result_role: "distinct-reserved-variable-right-result",
    left_expected_role: Some("distinct-reserved-variable-left-expected"),
    right_expected_role: Some("distinct-reserved-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_DISTINCT_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "DistinctReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key: TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: true,
    require_distinct_type_ranges: false,
    left_result_role: "distinct-reserved-variable-inequality-left-result",
    right_result_role: "distinct-reserved-variable-inequality-right-result",
    left_expected_role: Some("distinct-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("distinct-reserved-variable-inequality-right-expected"),
};

pub(in crate::runner) const SOURCE_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "DistinctReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key: TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: true,
    require_distinct_type_ranges: false,
    left_result_role: "distinct-reserved-variable-membership-left-result",
    right_result_role: "distinct-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("distinct-reserved-variable-membership-right-expected"),
};

pub(in crate::runner) const SOURCE_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "HeterogeneousReserveMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key: TYPE_ELABORATION_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
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
    left_result_role: "heterogeneous-reserve-membership-left-result",
    right_result_role: "heterogeneous-reserve-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("heterogeneous-reserve-membership-right-expected"),
};

pub(in crate::runner) fn source_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_reserved_object_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_object_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_distinct_reserved_object_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_distinct_reserved_object_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_distinct_reserved_object_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_distinct_reserved_object_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_reserved_object_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_object_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_OBJECT_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_distinct_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_distinct_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_distinct_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_distinct_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_distinct_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_distinct_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_DISTINCT_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_heterogeneous_reserve_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_heterogeneous_reserve_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_HETEROGENEOUS_RESERVE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_reserved_object_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_reserved_object_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_distinct_reserved_object_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_distinct_reserved_object_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_distinct_reserved_object_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_distinct_reserved_object_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_reserved_object_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_reserved_object_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_distinct_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_distinct_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_distinct_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_distinct_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_distinct_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_distinct_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_heterogeneous_reserve_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_heterogeneous_reserve_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_reserved_object_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_distinct_reserved_object_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_DISTINCT_RESERVED_OBJECT_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_distinct_reserved_object_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_DISTINCT_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_reserved_object_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_OBJECT_VARIABLE_INEQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_distinct_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_DISTINCT_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_distinct_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_DISTINCT_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_distinct_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_DISTINCT_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_heterogeneous_reserve_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_HETEROGENEOUS_RESERVE_MEMBERSHIP_CONFIG,
    )
}
