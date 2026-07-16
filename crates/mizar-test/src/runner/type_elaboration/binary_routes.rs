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
    SourceReservedVariableBuiltinType, SourceReservedVariableModeDefinition,
    SourceReservedVariableModeRadix, extract_source_reserved_variable_binary_formula,
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

const TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.multiple_reserve_declaration_equality.invalid_payload";
const TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.multiple_object_reserve_declaration_equality.invalid_payload";
const TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.multiple_object_reserve_declaration_inequality.invalid_payload";
const TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.multiple_reserve_declaration_inequality.invalid_payload";
const TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.multiple_reserve_declaration_membership.invalid_payload";
pub(in crate::runner) const SOURCE_MULTIPLE_RESERVE_DECLARATION_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "MultipleReserveDeclarationEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key: TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "multiple-reserve-declaration-left-result",
    right_result_role: "multiple-reserve-declaration-right-result",
    left_expected_role: Some("multiple-reserve-declaration-left-expected"),
    right_expected_role: Some("multiple-reserve-declaration-right-expected"),
};

pub(in crate::runner) const SOURCE_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "MultipleObjectReserveDeclarationEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Object,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "multiple-object-reserve-declaration-left-result",
    right_result_role: "multiple-object-reserve-declaration-right-result",
    left_expected_role: Some("multiple-object-reserve-declaration-left-expected"),
    right_expected_role: Some("multiple-object-reserve-declaration-right-expected"),
};

pub(in crate::runner) const SOURCE_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "MultipleObjectReserveDeclarationInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Object,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "multiple-object-reserve-declaration-inequality-left-result",
    right_result_role: "multiple-object-reserve-declaration-inequality-right-result",
    left_expected_role: Some("multiple-object-reserve-declaration-inequality-left-expected"),
    right_expected_role: Some("multiple-object-reserve-declaration-inequality-right-expected"),
};

pub(in crate::runner) const SOURCE_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "MultipleReserveDeclarationInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "multiple-reserve-declaration-inequality-left-result",
    right_result_role: "multiple-reserve-declaration-inequality-right-result",
    left_expected_role: Some("multiple-reserve-declaration-inequality-left-expected"),
    right_expected_role: Some("multiple-reserve-declaration-inequality-right-expected"),
};

pub(in crate::runner) const SOURCE_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "MultipleReserveDeclarationMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[None, None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "multiple-reserve-declaration-membership-left-result",
    right_result_role: "multiple-reserve-declaration-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("multiple-reserve-declaration-membership-right-expected"),
};

pub(in crate::runner) fn source_multiple_reserve_declaration_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_multiple_reserve_declaration_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_multiple_object_reserve_declaration_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_multiple_object_reserve_declaration_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_multiple_object_reserve_declaration_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_multiple_object_reserve_declaration_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_multiple_reserve_declaration_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_multiple_reserve_declaration_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_multiple_reserve_declaration_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_multiple_reserve_declaration_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_multiple_reserve_declaration_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_multiple_reserve_declaration_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_multiple_object_reserve_declaration_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_multiple_object_reserve_declaration_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_multiple_object_reserve_declaration_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_multiple_object_reserve_declaration_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_multiple_reserve_declaration_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_multiple_reserve_declaration_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_multiple_reserve_declaration_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_multiple_reserve_declaration_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_multiple_reserve_declaration_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_MULTIPLE_RESERVE_DECLARATION_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_multiple_object_reserve_declaration_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_MULTIPLE_OBJECT_RESERVE_DECLARATION_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_multiple_object_reserve_declaration_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_MULTIPLE_OBJECT_RESERVE_DECLARATION_INEQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_multiple_reserve_declaration_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_MULTIPLE_RESERVE_DECLARATION_INEQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_multiple_reserve_declaration_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_MULTIPLE_RESERVE_DECLARATION_MEMBERSHIP_CONFIG,
    )
}

const TYPE_ELABORATION_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.reserved_variable_inequality.invalid_payload";
pub(in crate::runner) const SOURCE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key: TYPE_ELABORATION_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "reserved-variable-membership-left-result",
    right_result_role: "reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("reserved-variable-membership-right-expected"),
};

pub(in crate::runner) const SOURCE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key: TYPE_ELABORATION_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[None],
    mode_definitions: &[],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "reserved-variable-inequality-left-result",
    right_result_role: "reserved-variable-inequality-right-result",
    left_expected_role: Some("reserved-variable-inequality-left-expected"),
    right_expected_role: Some("reserved-variable-inequality-right-expected"),
};

pub(in crate::runner) fn source_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_reserved_variable_inequality.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("LocalModeMembership"), None],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalModeMembershipDef",
        spelling: "LocalModeMembership",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
    }],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "local-mode-reserved-variable-membership-left-result",
    right_result_role: "local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("local-mode-reserved-variable-membership-right-expected"),
};

pub(in crate::runner) const SOURCE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key: TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("LocalModeFormula")],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalModeFormulaDef",
        spelling: "LocalModeFormula",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
    }],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "local-mode-reserved-variable-left-result",
    right_result_role: "local-mode-reserved-variable-right-result",
    left_expected_role: Some("local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("local-mode-reserved-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("LocalModeInequality")],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalModeInequalityDef",
        spelling: "LocalModeInequality",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
    }],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "local-mode-reserved-variable-inequality-left-result",
    right_result_role: "local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("local-mode-reserved-variable-inequality-right-expected"),
};

pub(in crate::runner) fn source_local_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_local_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_local_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_local_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_local_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_reserved_variable_equality.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("LocalObjectModeMembership"), None],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalObjectModeMembershipDef",
        spelling: "LocalObjectModeMembership",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Object),
    }],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("local-object-mode-reserved-variable-membership-right-expected"),
};

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("LocalObjectModeInequality")],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalObjectModeInequalityDef",
        spelling: "LocalObjectModeInequality",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Object),
    }],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("local-object-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("local-object-mode-reserved-variable-inequality-right-expected"),
};

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "LocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("LocalObjectMode")],
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalObjectModeDef",
        spelling: "LocalObjectMode",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Object),
    }],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "local-object-mode-reserved-variable-left-result",
    right_result_role: "local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("local-object-mode-reserved-variable-right-expected"),
};

pub(in crate::runner) fn source_local_object_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_local_object_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_local_object_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_local_object_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_local_object_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_local_object_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_local_object_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

const TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_mode_reserved_variable_inequality.invalid_payload";

pub(in crate::runner) const SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("ChainModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseModeMembershipDef",
            spelling: "BaseModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainModeMembershipDef",
            spelling: "ChainModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "chained-local-mode-reserved-variable-membership-left-result",
    right_result_role: "chained-local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("chained-local-mode-reserved-variable-membership-right-expected"),
};

pub(in crate::runner) const SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("ChainModeFormula")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseModeFormulaDef",
            spelling: "BaseModeFormula",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainModeFormulaDef",
            spelling: "ChainModeFormula",
            radix: SourceReservedVariableModeRadix::Mode("BaseModeFormula"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "chained-local-mode-reserved-variable-left-result",
    right_result_role: "chained-local-mode-reserved-variable-right-result",
    left_expected_role: Some("chained-local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("chained-local-mode-reserved-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["x"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("ChainModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseModeInequalityDef",
            spelling: "BaseModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainModeInequalityDef",
            spelling: "ChainModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "chained-local-mode-reserved-variable-inequality-left-result",
    right_result_role: "chained-local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("chained-local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("chained-local-mode-reserved-variable-inequality-right-expected"),
};

pub(in crate::runner) fn source_chained_local_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_chained_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_chained_local_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_chained_local_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_chained_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_chained_local_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_chained_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_chained_local_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_chained_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_chained_local_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.chained_local_object_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.chained_local_object_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.chained_local_object_mode_reserved_variable_inequality.invalid_payload";

pub(in crate::runner) const SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("ChainObjectModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeMembershipDef",
            spelling: "BaseObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainObjectModeMembershipDef",
            spelling: "ChainObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "chained-local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "chained-local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some(
        "chained-local-object-mode-reserved-variable-membership-right-expected",
    ),
};

pub(in crate::runner) const SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("ChainObjectMode")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeDef",
            spelling: "BaseObjectMode",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainObjectModeDef",
            spelling: "ChainObjectMode",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectMode"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "chained-local-object-mode-reserved-variable-left-result",
    right_result_role: "chained-local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("chained-local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("chained-local-object-mode-reserved-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ChainedLocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("ChainObjectModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeInequalityDef",
            spelling: "BaseObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainObjectModeInequalityDef",
            spelling: "ChainObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "chained-local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "chained-local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some(
        "chained-local-object-mode-reserved-variable-inequality-left-expected",
    ),
    right_expected_role: Some(
        "chained-local-object-mode-reserved-variable-inequality-right-expected",
    ),
};

pub(in crate::runner) fn source_chained_local_object_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_chained_local_object_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_chained_local_object_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_chained_local_object_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_chained_local_object_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_chained_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_chained_local_object_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_chained_local_object_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_chained_local_object_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_chained_local_object_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

const TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.two_edge_local_mode_reserved_variable_inequality.invalid_payload";

pub(in crate::runner) const SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeModeMembershipDef",
            spelling: "BaseTwoEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeModeMembershipDef",
            spelling: "MiddleTwoEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeModeMembershipDef",
            spelling: "OuterTwoEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "two-edge-local-mode-reserved-variable-membership-left-result",
    right_result_role: "two-edge-local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("two-edge-local-mode-reserved-variable-membership-right-expected"),
};

pub(in crate::runner) const SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeModeEqualityDef",
            spelling: "BaseTwoEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
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
    left_result_role: "two-edge-local-mode-reserved-variable-left-result",
    right_result_role: "two-edge-local-mode-reserved-variable-right-result",
    left_expected_role: Some("two-edge-local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("two-edge-local-mode-reserved-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeModeInequalityDef",
            spelling: "BaseTwoEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeModeInequalityDef",
            spelling: "MiddleTwoEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeModeInequalityDef",
            spelling: "OuterTwoEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "two-edge-local-mode-reserved-variable-inequality-left-result",
    right_result_role: "two-edge-local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("two-edge-local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("two-edge-local-mode-reserved-variable-inequality-right-expected"),
};

pub(in crate::runner) fn source_two_edge_local_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_two_edge_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_two_edge_local_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_two_edge_local_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_two_edge_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_two_edge_local_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_two_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_two_edge_local_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_two_edge_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_two_edge_local_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.two_edge_local_object_mode_reserved_variable_membership.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.two_edge_local_object_mode_reserved_variable_inequality.invalid_payload";
const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.two_edge_local_object_mode_reserved_variable_equality.invalid_payload";

pub(in crate::runner) const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeObjectModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeObjectModeMembershipDef",
            spelling: "BaseTwoEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeObjectModeMembershipDef",
            spelling: "MiddleTwoEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeObjectModeMembershipDef",
            spelling: "OuterTwoEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeObjectModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "two-edge-local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "two-edge-local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some(
        "two-edge-local-object-mode-reserved-variable-membership-right-expected",
    ),
};

pub(in crate::runner) const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeObjectModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeObjectModeInequalityDef",
            spelling: "BaseTwoEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeObjectModeInequalityDef",
            spelling: "MiddleTwoEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeObjectModeInequalityDef",
            spelling: "OuterTwoEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeObjectModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "two-edge-local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "two-edge-local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some(
        "two-edge-local-object-mode-reserved-variable-inequality-left-expected",
    ),
    right_expected_role: Some(
        "two-edge-local-object-mode-reserved-variable-inequality-right-expected",
    ),
};

pub(in crate::runner) const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "TwoEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("OuterTwoEdgeObjectModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeObjectModeEqualityDef",
            spelling: "BaseTwoEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeObjectModeEqualityDef",
            spelling: "MiddleTwoEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeObjectModeEqualityDef",
            spelling: "OuterTwoEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeObjectModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "two-edge-local-object-mode-reserved-variable-left-result",
    right_result_role: "two-edge-local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("two-edge-local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("two-edge-local-object-mode-reserved-variable-right-expected"),
};

pub(in crate::runner) fn source_two_edge_local_object_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_two_edge_local_object_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_two_edge_local_object_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_two_edge_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_two_edge_local_object_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_two_edge_local_object_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_two_edge_local_object_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_two_edge_local_object_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_two_edge_local_object_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_two_edge_local_object_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_two_edge_local_object_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.three_edge_local_mode_reserved_variable_membership.invalid_payload";

const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.three_edge_local_mode_reserved_variable_equality.invalid_payload";

const TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.three_edge_local_mode_reserved_variable_inequality.invalid_payload";

pub(in crate::runner) const SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeModeMembershipDef",
            spelling: "BaseThreeEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeModeMembershipDef",
            spelling: "InnerThreeEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeModeMembershipDef",
            spelling: "MiddleThreeEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeModeMembershipDef",
            spelling: "OuterThreeEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "three-edge-local-mode-reserved-variable-membership-left-result",
    right_result_role: "three-edge-local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("three-edge-local-mode-reserved-variable-membership-right-expected"),
};

pub(in crate::runner) const SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeModeEqualityDef",
            spelling: "BaseThreeEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeModeEqualityDef",
            spelling: "InnerThreeEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeModeEqualityDef",
            spelling: "MiddleThreeEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeModeEqualityDef",
            spelling: "OuterThreeEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "three-edge-local-mode-reserved-variable-left-result",
    right_result_role: "three-edge-local-mode-reserved-variable-right-result",
    left_expected_role: Some("three-edge-local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("three-edge-local-mode-reserved-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeModeInequalityDef",
            spelling: "BaseThreeEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeModeInequalityDef",
            spelling: "InnerThreeEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeModeInequalityDef",
            spelling: "MiddleThreeEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeModeInequalityDef",
            spelling: "OuterThreeEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "three-edge-local-mode-reserved-variable-inequality-left-result",
    right_result_role: "three-edge-local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("three-edge-local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("three-edge-local-mode-reserved-variable-inequality-right-expected"),
};

pub(in crate::runner) fn source_three_edge_local_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_three_edge_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_three_edge_local_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_three_edge_local_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_three_edge_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_three_edge_local_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_three_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_three_edge_local_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_three_edge_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_three_edge_local_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.three_edge_local_object_mode_reserved_variable_membership.invalid_payload";

const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.three_edge_local_object_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.three_edge_local_object_mode_reserved_variable_inequality.invalid_payload";

pub(in crate::runner) const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeObjectModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeMembershipDef",
            spelling: "BaseThreeEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeMembershipDef",
            spelling: "InnerThreeEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeMembershipDef",
            spelling: "MiddleThreeEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeMembershipDef",
            spelling: "OuterThreeEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeObjectModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "three-edge-local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "three-edge-local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some(
        "three-edge-local-object-mode-reserved-variable-membership-right-expected",
    ),
};

pub(in crate::runner) const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeObjectModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeEqualityDef",
            spelling: "BaseThreeEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeEqualityDef",
            spelling: "InnerThreeEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeEqualityDef",
            spelling: "MiddleThreeEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeEqualityDef",
            spelling: "OuterThreeEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeObjectModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "three-edge-local-object-mode-reserved-variable-left-result",
    right_result_role: "three-edge-local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("three-edge-local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("three-edge-local-object-mode-reserved-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "ThreeEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("OuterThreeEdgeObjectModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeInequalityDef",
            spelling: "BaseThreeEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeInequalityDef",
            spelling: "InnerThreeEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseThreeEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeInequalityDef",
            spelling: "MiddleThreeEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("InnerThreeEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeInequalityDef",
            spelling: "OuterThreeEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleThreeEdgeObjectModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "three-edge-local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "three-edge-local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some(
        "three-edge-local-object-mode-reserved-variable-inequality-left-expected",
    ),
    right_expected_role: Some(
        "three-edge-local-object-mode-reserved-variable-inequality-right-expected",
    ),
};

pub(in crate::runner) fn source_three_edge_local_object_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_three_edge_local_object_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_three_edge_local_object_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_three_edge_local_object_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_three_edge_local_object_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_three_edge_local_object_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_three_edge_local_object_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_three_edge_local_object_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_three_edge_local_object_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_reserved_variable_membership.invalid_payload";

const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_reserved_variable_equality.invalid_payload";

const TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.four_edge_local_mode_reserved_variable_inequality.invalid_payload";

pub(in crate::runner) const SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Set,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeMembershipDef",
            spelling: "BaseFourEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeMembershipDef",
            spelling: "InnerFourEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeMembershipDef",
            spelling: "MiddleFourEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeMembershipDef",
            spelling: "OuterFourEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeMembershipDef",
            spelling: "TooDeepFourEdgeModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "four-edge-local-mode-reserved-variable-membership-left-result",
    right_result_role: "four-edge-local-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some("four-edge-local-mode-reserved-variable-membership-right-expected"),
};

pub(in crate::runner) const SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeEqualityDef",
            spelling: "BaseFourEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeEqualityDef",
            spelling: "InnerFourEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeEqualityDef",
            spelling: "MiddleFourEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeEqualityDef",
            spelling: "OuterFourEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeEqualityDef",
            spelling: "TooDeepFourEdgeModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "four-edge-local-mode-reserved-variable-left-result",
    right_result_role: "four-edge-local-mode-reserved-variable-right-result",
    left_expected_role: Some("four-edge-local-mode-reserved-variable-left-expected"),
    right_expected_role: Some("four-edge-local-mode-reserved-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Set],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeModeInequalityDef",
            spelling: "BaseFourEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Set),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeModeInequalityDef",
            spelling: "InnerFourEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeModeInequalityDef",
            spelling: "MiddleFourEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeModeInequalityDef",
            spelling: "OuterFourEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeModeInequalityDef",
            spelling: "TooDeepFourEdgeModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "four-edge-local-mode-reserved-variable-inequality-left-result",
    right_result_role: "four-edge-local-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some("four-edge-local-mode-reserved-variable-inequality-left-expected"),
    right_expected_role: Some("four-edge-local-mode-reserved-variable-inequality-right-expected"),
};

pub(in crate::runner) fn source_four_edge_local_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_four_edge_local_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_four_edge_local_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_four_edge_local_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_membership(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_four_edge_local_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_equality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_four_edge_local_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload =
        extract_source_four_edge_local_mode_reserved_variable_inequality(ast, module, symbols)?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_four_edge_local_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_four_edge_local_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_four_edge_local_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}

const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.four_edge_local_object_mode_reserved_variable_membership.invalid_payload";

const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.four_edge_local_object_mode_reserved_variable_equality.invalid_payload";
const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.four_edge_local_object_mode_reserved_variable_inequality.invalid_payload";

pub(in crate::runner) const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalObjectModeReservedVariableMembershipPayloadBoundary",
    operator: "in",
    formula_kind: FormulaKind::Membership,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    reserve_item_count: 2,
    binding_spellings: &["x", "y"],
    binding_types: &[
        SourceReservedVariableBuiltinType::Object,
        SourceReservedVariableBuiltinType::Set,
    ],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeObjectModeMembership"), None],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeMembershipDef",
            spelling: "BaseFourEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeMembershipDef",
            spelling: "InnerFourEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeMembershipDef",
            spelling: "MiddleFourEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeMembershipDef",
            spelling: "OuterFourEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeObjectModeMembership"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeMembershipDef",
            spelling: "TooDeepFourEdgeObjectModeMembership",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeObjectModeMembership"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 1,
    require_shared_type_range: false,
    require_distinct_type_ranges: true,
    left_result_role: "four-edge-local-object-mode-reserved-variable-membership-left-result",
    right_result_role: "four-edge-local-object-mode-reserved-variable-membership-right-result",
    left_expected_role: None,
    right_expected_role: Some(
        "four-edge-local-object-mode-reserved-variable-membership-right-expected",
    ),
};

pub(in crate::runner) const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalObjectModeReservedVariableEqualityPayloadBoundary",
    operator: "=",
    formula_kind: FormulaKind::Equality,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeObjectModeEquality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeEqualityDef",
            spelling: "BaseFourEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeEqualityDef",
            spelling: "InnerFourEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeEqualityDef",
            spelling: "MiddleFourEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeEqualityDef",
            spelling: "OuterFourEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeObjectModeEquality"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeEqualityDef",
            spelling: "TooDeepFourEdgeObjectModeEquality",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeObjectModeEquality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "four-edge-local-object-mode-reserved-variable-left-result",
    right_result_role: "four-edge-local-object-mode-reserved-variable-right-result",
    left_expected_role: Some("four-edge-local-object-mode-reserved-variable-left-expected"),
    right_expected_role: Some("four-edge-local-object-mode-reserved-variable-right-expected"),
};

pub(in crate::runner) const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG:
    SourceReservedVariableBinaryFormulaConfig = SourceReservedVariableBinaryFormulaConfig {
    label: "FourEdgeLocalObjectModeReservedVariableInequalityPayloadBoundary",
    operator: "<>",
    formula_kind: FormulaKind::Inequality,
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    reserve_item_count: 1,
    binding_spellings: &["z"],
    binding_types: &[SourceReservedVariableBuiltinType::Object],
    binding_source_mode_spellings: &[Some("TooDeepFourEdgeObjectModeInequality")],
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeInequalityDef",
            spelling: "BaseFourEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeInequalityDef",
            spelling: "InnerFourEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeInequalityDef",
            spelling: "MiddleFourEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeInequalityDef",
            spelling: "OuterFourEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeObjectModeInequality"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeInequalityDef",
            spelling: "TooDeepFourEdgeObjectModeInequality",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeObjectModeInequality"),
        },
    ],
    left_binding_index: 0,
    right_binding_index: 0,
    require_shared_type_range: false,
    require_distinct_type_ranges: false,
    left_result_role: "four-edge-local-object-mode-reserved-variable-inequality-left-result",
    right_result_role: "four-edge-local-object-mode-reserved-variable-inequality-right-result",
    left_expected_role: Some(
        "four-edge-local-object-mode-reserved-variable-inequality-left-expected",
    ),
    right_expected_role: Some(
        "four-edge-local-object-mode-reserved-variable-inequality-right-expected",
    ),
};

pub(in crate::runner) fn source_four_edge_local_object_mode_reserved_variable_membership_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_four_edge_local_object_mode_reserved_variable_equality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

pub(in crate::runner) fn source_four_edge_local_object_mode_reserved_variable_inequality_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    Some(source_reserved_variable_formula_result_detail_keys(
        build_source_reserved_variable_formula_output(payload, symbols),
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_four_edge_local_object_mode_reserved_variable_membership_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_membership(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_four_edge_local_object_mode_reserved_variable_equality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_equality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

#[cfg(test)]
pub(in crate::runner) fn source_four_edge_local_object_mode_reserved_variable_inequality_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormulaOutput> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_inequality(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_formula_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_four_edge_local_object_mode_reserved_variable_membership(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_MEMBERSHIP_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_four_edge_local_object_mode_reserved_variable_equality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_EQUALITY_CONFIG,
    )
}

pub(in crate::runner) fn extract_source_four_edge_local_object_mode_reserved_variable_inequality(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableBinaryFormula> {
    extract_source_reserved_variable_binary_formula(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_INEQUALITY_CONFIG,
    )
}
