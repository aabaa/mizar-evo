use mizar_resolve::env::SymbolEnv;
use mizar_resolve::resolved_ast::ModuleId as ResolverModuleId;
use mizar_syntax::SurfaceAst;

use super::long_chain_config::{
    SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS, SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
};
#[cfg(test)]
use super::output::SourceReservedVariableTypeAssertionOutput;
use super::output::{
    build_source_reserved_variable_type_assertion_output,
    source_reserved_variable_type_assertion_result_detail_keys,
};
use super::source_formula::{
    SourceReservedVariableAssertedHeadRelation, SourceReservedVariableBuiltinType,
    SourceReservedVariableModeDefinition, SourceReservedVariableModeRadix,
    SourceReservedVariableTypeAssertion, SourceReservedVariableTypeAssertionConfig,
    extract_source_reserved_variable_type_assertion_with_config,
};

const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.local_mode_long_chain_reserved_variable_type_assertion.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role: "long-local-mode-reserved-variable-type-assertion-subject-result",
};

pub(in crate::runner) fn source_local_mode_long_chain_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_long_chain_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_long_chain_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_long_chain_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_mode_long_chain_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_MODE_LONG_CHAIN_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeAssertedHeadPayloadBoundary",
    invalid_payload_key: TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode("ChainMode6"),
    subject_result_role: "long-local-mode-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_mode_long_chain_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_long_chain_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_long_chain_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_long_chain_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_mode_long_chain_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_radix_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "ChainMode5",
    ),
    subject_result_role: "long-local-mode-radix-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_mode_long_chain_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_long_chain_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_long_chain_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_long_chain_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_mode_long_chain_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_two_hop_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeTwoHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
        intermediate_spelling: "ChainMode5",
        asserted_spelling: "ChainMode4",
    },
    subject_result_role: "long-local-mode-two-hop-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_mode_long_chain_two_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_long_chain_two_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_long_chain_two_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_long_chain_two_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_mode_long_chain_two_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_three_hop_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeThreeHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingThreeHopRadix {
        first_intermediate_spelling: "ChainMode5",
        second_intermediate_spelling: "ChainMode4",
        asserted_spelling: "ChainMode3",
    },
    subject_result_role: "long-local-mode-three-hop-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_mode_long_chain_three_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_mode_long_chain_three_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_long_chain_three_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_mode_long_chain_three_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_mode_long_chain_three_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_four_hop_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeFourHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingFourHopRadix {
        first_intermediate_spelling: "ChainMode5",
        second_intermediate_spelling: "ChainMode4",
        third_intermediate_spelling: "ChainMode3",
        asserted_spelling: "ChainMode2",
    },
    subject_result_role: "long-local-mode-four-hop-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_mode_long_chain_four_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_mode_long_chain_four_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_long_chain_four_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_mode_long_chain_four_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_mode_long_chain_four_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_five_hop_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeFiveHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingFiveHopRadix {
        first_intermediate_spelling: "ChainMode5",
        second_intermediate_spelling: "ChainMode4",
        third_intermediate_spelling: "ChainMode3",
        fourth_intermediate_spelling: "ChainMode2",
        asserted_spelling: "ChainMode1",
    },
    subject_result_role: "long-local-mode-five-hop-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_mode_long_chain_five_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_mode_long_chain_five_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_long_chain_five_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_mode_long_chain_five_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_mode_long_chain_five_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_mode_long_chain_six_hop_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalModeSixHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Set,
    binding_source_mode_spelling: Some("ChainMode6"),
    mode_definitions: SOURCE_LOCAL_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Set,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingSixHopRadix {
        first_intermediate_spelling: "ChainMode5",
        second_intermediate_spelling: "ChainMode4",
        third_intermediate_spelling: "ChainMode3",
        fourth_intermediate_spelling: "ChainMode2",
        fifth_intermediate_spelling: "ChainMode1",
        asserted_spelling: "BaseMode",
    },
    subject_result_role: "long-local-mode-six-hop-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_mode_long_chain_six_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_mode_long_chain_six_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_mode_long_chain_six_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_mode_long_chain_six_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_mode_long_chain_six_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_long_chain_six_hop_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeSixHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingSixHopRadix {
        first_intermediate_spelling: "ChainObjectMode5",
        second_intermediate_spelling: "ChainObjectMode4",
        third_intermediate_spelling: "ChainObjectMode3",
        fourth_intermediate_spelling: "ChainObjectMode2",
        fifth_intermediate_spelling: "ChainObjectMode1",
        asserted_spelling: "BaseObjectMode",
    },
    subject_result_role: "long-local-object-mode-six-hop-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_object_mode_long_chain_six_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_six_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_long_chain_six_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_six_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_long_chain_six_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_SIX_HOP_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_long_chain_five_hop_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeFiveHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingFiveHopRadix {
        first_intermediate_spelling: "ChainObjectMode5",
        second_intermediate_spelling: "ChainObjectMode4",
        third_intermediate_spelling: "ChainObjectMode3",
        fourth_intermediate_spelling: "ChainObjectMode2",
        asserted_spelling: "ChainObjectMode1",
    },
    subject_result_role: "long-local-object-mode-five-hop-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_object_mode_long_chain_five_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_five_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_long_chain_five_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_five_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_long_chain_five_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_FIVE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_long_chain_four_hop_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeFourHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingFourHopRadix {
        first_intermediate_spelling: "ChainObjectMode5",
        second_intermediate_spelling: "ChainObjectMode4",
        third_intermediate_spelling: "ChainObjectMode3",
        asserted_spelling: "ChainObjectMode2",
    },
    subject_result_role: "long-local-object-mode-four-hop-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_object_mode_long_chain_four_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_four_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_long_chain_four_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_four_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_long_chain_four_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_FOUR_HOP_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_long_chain_three_hop_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeThreeHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingThreeHopRadix {
        first_intermediate_spelling: "ChainObjectMode5",
        second_intermediate_spelling: "ChainObjectMode4",
        asserted_spelling: "ChainObjectMode3",
    },
    subject_result_role: "long-local-object-mode-three-hop-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_object_mode_long_chain_three_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_three_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_long_chain_three_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_three_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_long_chain_three_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_THREE_HOP_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_long_chain_two_hop_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeTwoHopAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
        intermediate_spelling: "ChainObjectMode5",
        asserted_spelling: "ChainObjectMode4",
    },
    subject_result_role: "long-local-object-mode-two-hop-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_object_mode_long_chain_two_hop_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_two_hop_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_long_chain_two_hop_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_two_hop_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_long_chain_two_hop_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_TWO_HOP_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_long_chain_radix_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "ChainObjectMode5",
    ),
    subject_result_role: "long-local-object-mode-radix-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_object_mode_long_chain_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_long_chain_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_long_chain_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_long_chain_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_long_chain_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RADIX_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_long_chain_asserted_head.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
        "ChainObjectMode6",
    ),
    subject_result_role: "long-local-object-mode-asserted-head-subject-result",
};

pub(in crate::runner) fn source_local_object_mode_long_chain_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_object_mode_long_chain_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_long_chain_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_object_mode_long_chain_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_long_chain_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.local_object_mode_long_chain_reserved_variable_type_assertion.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LongLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectMode6"),
    mode_definitions: SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_DEFINITIONS,
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role:
        "long-local-object-mode-reserved-variable-type-assertion-subject-result",
};

pub(in crate::runner) fn source_local_object_mode_long_chain_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_long_chain_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_long_chain_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_LONG_CHAIN_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.local_object_mode_reserved_variable_type_assertion.invalid_payload";

pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "LocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("LocalObjectModeTypeAssertion"),
    mode_definitions: &[SourceReservedVariableModeDefinition {
        label: "LocalObjectModeTypeAssertionDef",
        spelling: "LocalObjectModeTypeAssertion",
        radix: SourceReservedVariableModeRadix::Builtin(SourceReservedVariableBuiltinType::Object),
    }],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role: "local-object-mode-reserved-variable-type-assertion-subject-result",
};

pub(in crate::runner) fn source_local_object_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_local_object_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        TYPE_ELABORATION_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_local_object_mode_reserved_variable_type_assertion(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str =
    "type_elaboration.checker.chained_local_object_mode_reserved_variable_type_assertion.invalid_payload";

pub(in crate::runner) const SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ChainedLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeTypeAssertionDef",
            spelling: "BaseObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainObjectModeTypeAssertionDef",
            spelling: "ChainObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectModeTypeAssertion"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role:
        "chained-local-object-mode-reserved-variable-type-assertion-subject-result",
};

pub(in crate::runner) fn source_chained_local_object_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_chained_local_object_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_chained_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_chained_local_object_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

const TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.two_edge_local_object_mode_reserved_variable_type_assertion.invalid_payload";

pub(in crate::runner) const SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "TwoEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterTwoEdgeObjectModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseTwoEdgeObjectModeTypeAssertionDef",
            spelling: "BaseTwoEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleTwoEdgeObjectModeTypeAssertionDef",
            spelling: "MiddleTwoEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("BaseTwoEdgeObjectModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterTwoEdgeObjectModeTypeAssertionDef",
            spelling: "OuterTwoEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("MiddleTwoEdgeObjectModeTypeAssertion"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role:
        "two-edge-local-object-mode-reserved-variable-type-assertion-subject-result",
};

pub(in crate::runner) fn source_two_edge_local_object_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_two_edge_local_object_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_two_edge_local_object_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_TWO_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

const TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.three_edge_local_object_mode_reserved_variable_type_assertion.invalid_payload";

pub(in crate::runner) const SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ThreeEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterThreeEdgeObjectModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseThreeEdgeObjectModeTypeAssertionDef",
            spelling: "BaseThreeEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerThreeEdgeObjectModeTypeAssertionDef",
            spelling: "InnerThreeEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode(
                "BaseThreeEdgeObjectModeTypeAssertion",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleThreeEdgeObjectModeTypeAssertionDef",
            spelling: "MiddleThreeEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode(
                "InnerThreeEdgeObjectModeTypeAssertion",
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterThreeEdgeObjectModeTypeAssertionDef",
            spelling: "OuterThreeEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode(
                "MiddleThreeEdgeObjectModeTypeAssertion",
            ),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role:
        "three-edge-local-object-mode-reserved-variable-type-assertion-subject-result",
};

pub(in crate::runner) fn source_three_edge_local_object_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_three_edge_local_object_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_three_edge_local_object_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_THREE_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

const TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY:
    &str = "type_elaboration.checker.four_edge_local_object_mode_reserved_variable_type_assertion.invalid_payload";

pub(in crate::runner) const SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "FourEdgeLocalObjectModeReservedVariableTypeAssertionPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("TooDeepFourEdgeObjectModeTypeAssertion"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseFourEdgeObjectModeTypeAssertionDef",
            spelling: "BaseFourEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "InnerFourEdgeObjectModeTypeAssertionDef",
            spelling: "InnerFourEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("BaseFourEdgeObjectModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "MiddleFourEdgeObjectModeTypeAssertionDef",
            spelling: "MiddleFourEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("InnerFourEdgeObjectModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterFourEdgeObjectModeTypeAssertionDef",
            spelling: "OuterFourEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("MiddleFourEdgeObjectModeTypeAssertion"),
        },
        SourceReservedVariableModeDefinition {
            label: "TooDeepFourEdgeObjectModeTypeAssertionDef",
            spelling: "TooDeepFourEdgeObjectModeTypeAssertion",
            radix: SourceReservedVariableModeRadix::Mode("OuterFourEdgeObjectModeTypeAssertion"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::Builtin,
    subject_result_role:
        "four-edge-local-object-mode-reserved-variable-type-assertion-subject-result",
};

pub(in crate::runner) fn source_four_edge_local_object_mode_reserved_variable_type_assertion_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_four_edge_local_object_mode_reserved_variable_type_assertion_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
        ast, module, symbols,
    )?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_four_edge_local_object_mode_reserved_variable_type_assertion(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_FOUR_EDGE_LOCAL_OBJECT_MODE_RESERVED_VARIABLE_TYPE_ASSERTION_CONFIG,
    )
}

const TYPE_ELABORATION_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.local_object_mode_asserted_head.invalid_payload";

#[rustfmt::skip]
pub(in crate::runner) const SOURCE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG: SourceReservedVariableTypeAssertionConfig =
    SourceReservedVariableTypeAssertionConfig {
        label: "LocalObjectModeAssertedHeadPayloadBoundary",
        invalid_payload_key: TYPE_ELABORATION_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
        binding_spelling: "x",
        binding_type: SourceReservedVariableBuiltinType::Object,
        binding_source_mode_spelling: Some("LocalObjectModeAssertedHead"),
        mode_definitions: &[SourceReservedVariableModeDefinition {
            label: "LocalObjectModeAssertedHeadDef",
            spelling: "LocalObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        }],
        asserted_type: SourceReservedVariableBuiltinType::Object,
        asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
            "LocalObjectModeAssertedHead",
        ),
        subject_result_role: "local-object-mode-asserted-head-subject-result",
    };

pub(in crate::runner) fn source_local_object_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_local_object_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_local_object_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_local_object_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_local_object_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_object_mode_asserted_head.invalid_payload";

#[rustfmt::skip]
pub(in crate::runner) const SOURCE_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ChainedLocalObjectModeAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("ChainObjectModeAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeAssertedHeadDef",
            spelling: "BaseObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "ChainObjectModeAssertedHeadDef",
            spelling: "ChainObjectModeAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectModeAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::SameMode(
        "ChainObjectModeAssertedHead",
    ),
    subject_result_role: "chained-local-object-mode-asserted-head-subject-result",
};

pub(in crate::runner) fn source_chained_local_object_mode_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload = extract_source_chained_local_object_mode_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_chained_local_object_mode_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload = extract_source_chained_local_object_mode_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_chained_local_object_mode_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_ASSERTED_HEAD_CONFIG,
    )
}

const TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.chained_local_object_mode_radix_asserted_head.invalid_payload";

#[rustfmt::skip]
pub(in crate::runner) const SOURCE_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_CONFIG:
    SourceReservedVariableTypeAssertionConfig = SourceReservedVariableTypeAssertionConfig {
    label: "ChainedLocalObjectModeRadixAssertedHeadPayloadBoundary",
    invalid_payload_key:
        TYPE_ELABORATION_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_INVALID_PAYLOAD_KEY,
    binding_spelling: "x",
    binding_type: SourceReservedVariableBuiltinType::Object,
    binding_source_mode_spelling: Some("OuterObjectModeRadixAssertedHead"),
    mode_definitions: &[
        SourceReservedVariableModeDefinition {
            label: "BaseObjectModeRadixAssertedHeadDef",
            spelling: "BaseObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Builtin(
                SourceReservedVariableBuiltinType::Object,
            ),
        },
        SourceReservedVariableModeDefinition {
            label: "OuterObjectModeRadixAssertedHeadDef",
            spelling: "OuterObjectModeRadixAssertedHead",
            radix: SourceReservedVariableModeRadix::Mode("BaseObjectModeRadixAssertedHead"),
        },
    ],
    asserted_type: SourceReservedVariableBuiltinType::Object,
    asserted_head_relation: SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(
        "BaseObjectModeRadixAssertedHead",
    ),
    subject_result_role: "chained-local-object-mode-radix-asserted-head-subject-result",
};

pub(in crate::runner) fn source_chained_local_object_mode_radix_asserted_head_detail_keys(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let payload =
        extract_source_chained_local_object_mode_radix_asserted_head(ast, module, symbols)?;
    let invalid_payload_key = payload.config.invalid_payload_key;
    Some(source_reserved_variable_type_assertion_result_detail_keys(
        build_source_reserved_variable_type_assertion_output(payload, symbols),
        invalid_payload_key,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_chained_local_object_mode_radix_asserted_head_output(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertionOutput> {
    let payload =
        extract_source_chained_local_object_mode_radix_asserted_head(ast, module, symbols)?;
    build_source_reserved_variable_type_assertion_output(payload, symbols).ok()
}

pub(in crate::runner) fn extract_source_chained_local_object_mode_radix_asserted_head(
    ast: &SurfaceAst,
    module: ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceReservedVariableTypeAssertion> {
    extract_source_reserved_variable_type_assertion_with_config(
        ast,
        module,
        symbols,
        &SOURCE_CHAINED_LOCAL_OBJECT_MODE_RADIX_ASSERTED_HEAD_CONFIG,
    )
}
