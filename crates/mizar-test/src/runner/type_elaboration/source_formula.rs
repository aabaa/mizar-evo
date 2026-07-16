use std::collections::{BTreeMap, BTreeSet};

use mizar_checker::type_checker::{
    FormulaKind, ModeExpansion, SourceReserveBindingInput, TypeExpressionInput, TypeHeadInput,
    TypeHeadRef,
};
use mizar_checker::typed_ast::TypedSiteRef;
use mizar_resolve::env::{ContributionKind, NamespacePath, SymbolEnv, SymbolKind};
use mizar_resolve::resolved_ast::{ModuleId as ResolverModuleId, SymbolId as ResolverSymbolId};
use mizar_session::SourceRange;
use mizar_syntax::{
    SurfaceAst, SurfaceFormulaConnective, SurfaceFormulaConstant, SurfaceFormulaPrefixOperator,
    SurfaceNode, SurfaceNodeId, SurfaceNodeKind, SurfaceQuantifierKind,
};

use super::source_ast::{
    direct_token_texts, exact_compilation_item_list, is_exact_parser_type_fixtures_import,
    qualified_symbol_spelling, structural_child_ids, subtree_has_recovery, surface_nodes_with_kind,
    surface_site,
};
use super::source_reserve::{
    SourceReserveExtraction, SourceTypeExpression, extract_builtin_source_type_expression,
    mode_definition_pattern_spelling, source_mode_symbol_spelling,
};
#[derive(Debug)]
pub(in crate::runner) struct SourceReservedVariableBinaryFormulaConfig {
    pub(in crate::runner) label: &'static str,
    pub(in crate::runner) operator: &'static str,
    pub(in crate::runner) formula_kind: FormulaKind,
    pub(in crate::runner) invalid_payload_key: &'static str,
    pub(in crate::runner) reserve_item_count: usize,
    pub(in crate::runner) binding_spellings: &'static [&'static str],
    pub(in crate::runner) binding_types: &'static [SourceReservedVariableBuiltinType],
    pub(in crate::runner) binding_source_mode_spellings: &'static [Option<&'static str>],
    pub(in crate::runner) mode_definitions: &'static [SourceReservedVariableModeDefinition],
    pub(in crate::runner) left_binding_index: usize,
    pub(in crate::runner) right_binding_index: usize,
    pub(in crate::runner) require_shared_type_range: bool,
    pub(in crate::runner) require_distinct_type_ranges: bool,
    pub(in crate::runner) left_result_role: &'static str,
    pub(in crate::runner) right_result_role: &'static str,
    pub(in crate::runner) left_expected_role: Option<&'static str>,
    pub(in crate::runner) right_expected_role: Option<&'static str>,
}

#[derive(Debug)]
pub(in crate::runner) struct SourceReservedVariableTypeAssertionConfig {
    pub(in crate::runner) label: &'static str,
    pub(in crate::runner) invalid_payload_key: &'static str,
    pub(in crate::runner) binding_spelling: &'static str,
    pub(in crate::runner) binding_type: SourceReservedVariableBuiltinType,
    pub(in crate::runner) binding_source_mode_spelling: Option<&'static str>,
    pub(in crate::runner) mode_definitions: &'static [SourceReservedVariableModeDefinition],
    pub(in crate::runner) asserted_type: SourceReservedVariableBuiltinType,
    pub(in crate::runner) asserted_head_relation: SourceReservedVariableAssertedHeadRelation,
    pub(in crate::runner) subject_result_role: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SourceReservedVariableAssertedHeadRelation {
    Builtin,
    SameMode(&'static str),
    BindingImmediateRadix(&'static str),
    BindingTwoHopRadix {
        intermediate_spelling: &'static str,
        asserted_spelling: &'static str,
    },
    BindingThreeHopRadix {
        first_intermediate_spelling: &'static str,
        second_intermediate_spelling: &'static str,
        asserted_spelling: &'static str,
    },
    BindingFourHopRadix {
        first_intermediate_spelling: &'static str,
        second_intermediate_spelling: &'static str,
        third_intermediate_spelling: &'static str,
        asserted_spelling: &'static str,
    },
    BindingFiveHopRadix {
        first_intermediate_spelling: &'static str,
        second_intermediate_spelling: &'static str,
        third_intermediate_spelling: &'static str,
        fourth_intermediate_spelling: &'static str,
        asserted_spelling: &'static str,
    },
    BindingSixHopRadix {
        first_intermediate_spelling: &'static str,
        second_intermediate_spelling: &'static str,
        third_intermediate_spelling: &'static str,
        fourth_intermediate_spelling: &'static str,
        fifth_intermediate_spelling: &'static str,
        asserted_spelling: &'static str,
    },
}

impl SourceReservedVariableAssertedHeadRelation {
    pub(in crate::runner) fn source_mode_spelling(self) -> Option<&'static str> {
        match self {
            Self::Builtin => None,
            Self::SameMode(spelling) | Self::BindingImmediateRadix(spelling) => Some(spelling),
            Self::BindingTwoHopRadix {
                asserted_spelling, ..
            }
            | Self::BindingThreeHopRadix {
                asserted_spelling, ..
            }
            | Self::BindingFourHopRadix {
                asserted_spelling, ..
            }
            | Self::BindingFiveHopRadix {
                asserted_spelling, ..
            }
            | Self::BindingSixHopRadix {
                asserted_spelling, ..
            } => Some(asserted_spelling),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(in crate::runner) struct SourceReservedVariableModeDefinition {
    pub(in crate::runner) label: &'static str,
    pub(in crate::runner) spelling: &'static str,
    pub(in crate::runner) radix: SourceReservedVariableModeRadix,
}

#[derive(Debug, Clone, Copy)]
pub(in crate::runner) enum SourceReservedVariableModeRadix {
    Builtin(SourceReservedVariableBuiltinType),
    Mode(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::runner) enum SourceReservedVariableBuiltinType {
    Object,
    Set,
}

impl SourceReservedVariableBuiltinType {
    fn spelling(self) -> &'static str {
        match self {
            Self::Object => "object",
            Self::Set => "set",
        }
    }

    fn input_head(self) -> TypeHeadInput {
        match self {
            Self::Object => TypeHeadInput::BuiltinObject,
            Self::Set => TypeHeadInput::BuiltinSet,
        }
    }

    pub(in crate::runner) fn normalized_head(self) -> TypeHeadRef {
        match self {
            Self::Object => TypeHeadRef::BuiltinObject,
            Self::Set => TypeHeadRef::BuiltinSet,
        }
    }
}

pub(in crate::runner) fn source_binding_matches_reserved_builtin_type(
    binding: &SourceReserveBindingInput,
    expected_type: SourceReservedVariableBuiltinType,
    source_mode_spelling: Option<&str>,
    mode_expansions: &BTreeMap<ResolverSymbolId, ModeExpansion>,
) -> bool {
    if !binding.type_attributes.is_empty() {
        return false;
    }
    let Some(source_mode_spelling) = source_mode_spelling else {
        return binding.type_spelling == expected_type.spelling()
            && binding.type_head == expected_type.input_head();
    };
    let TypeHeadInput::Symbol(symbol) = &binding.type_head else {
        return false;
    };
    binding.type_spelling == source_mode_spelling
        && source_mode_terminal_builtin_input(symbol, expected_type, mode_expansions).is_some()
}

pub(in crate::runner) fn source_type_expression_matches_reserved_builtin_type(
    expression: &SourceTypeExpression,
    expected_type: SourceReservedVariableBuiltinType,
    source_mode_spelling: Option<&str>,
    mode_expansions: &BTreeMap<ResolverSymbolId, ModeExpansion>,
) -> bool {
    if !expression.attributes.is_empty() {
        return false;
    }
    let Some(source_mode_spelling) = source_mode_spelling else {
        return expression.spelling == expected_type.spelling()
            && expression.head == expected_type.input_head();
    };
    let TypeHeadInput::Symbol(symbol) = &expression.head else {
        return false;
    };
    expression.spelling == source_mode_spelling
        && source_mode_terminal_builtin_input(symbol, expected_type, mode_expansions).is_some()
}

pub(in crate::runner) fn source_reserved_variable_asserted_head_relation_is_exact(
    source_binding: &SourceReserveBindingInput,
    asserted_spelling: &str,
    asserted_head: &TypeHeadInput,
    config: &SourceReservedVariableTypeAssertionConfig,
    mode_expansions: &BTreeMap<ResolverSymbolId, ModeExpansion>,
) -> bool {
    match config.asserted_head_relation {
        SourceReservedVariableAssertedHeadRelation::Builtin => {
            asserted_spelling == config.asserted_type.spelling()
                && asserted_head == &config.asserted_type.input_head()
        }
        SourceReservedVariableAssertedHeadRelation::SameMode(expected_spelling) => {
            config.binding_source_mode_spelling == Some(expected_spelling)
                && source_binding.type_spelling == expected_spelling
                && asserted_spelling == expected_spelling
                && asserted_head == &source_binding.type_head
        }
        SourceReservedVariableAssertedHeadRelation::BindingImmediateRadix(expected_spelling) => {
            let Some(binding_spelling) = config.binding_source_mode_spelling else {
                return false;
            };
            let TypeHeadInput::Symbol(binding_symbol) = &source_binding.type_head else {
                return false;
            };
            let Some(binding_expansion) = mode_expansions.get(binding_symbol) else {
                return false;
            };
            let TypeHeadInput::Symbol(asserted_symbol) = asserted_head else {
                return false;
            };
            binding_spelling != expected_spelling
                && source_binding.type_spelling == binding_spelling
                && asserted_spelling == expected_spelling
                && asserted_symbol != binding_symbol
                && asserted_head == &binding_expansion.radix.head
                && binding_expansion.radix.spelling == expected_spelling
                && binding_expansion.radix.args.is_empty()
                && binding_expansion.radix.attributes.is_empty()
                && mode_expansions.contains_key(asserted_symbol)
                && source_mode_terminal_builtin_input(
                    asserted_symbol,
                    config.asserted_type,
                    mode_expansions,
                )
                .is_some()
        }
        SourceReservedVariableAssertedHeadRelation::BindingTwoHopRadix {
            intermediate_spelling,
            asserted_spelling: expected_asserted_spelling,
        } => {
            let Some(binding_spelling) = config.binding_source_mode_spelling else {
                return false;
            };
            let TypeHeadInput::Symbol(binding_symbol) = &source_binding.type_head else {
                return false;
            };
            let Some(binding_expansion) = mode_expansions.get(binding_symbol) else {
                return false;
            };
            let TypeHeadInput::Symbol(intermediate_symbol) = &binding_expansion.radix.head else {
                return false;
            };
            let Some(intermediate_expansion) = mode_expansions.get(intermediate_symbol) else {
                return false;
            };
            let TypeHeadInput::Symbol(asserted_symbol) = asserted_head else {
                return false;
            };
            binding_spelling != intermediate_spelling
                && binding_spelling != expected_asserted_spelling
                && intermediate_spelling != expected_asserted_spelling
                && source_binding.type_spelling == binding_spelling
                && asserted_spelling == expected_asserted_spelling
                && binding_symbol != intermediate_symbol
                && binding_symbol != asserted_symbol
                && intermediate_symbol != asserted_symbol
                && binding_expansion.attributes.is_empty()
                && binding_expansion.radix.spelling == intermediate_spelling
                && binding_expansion.radix.args.is_empty()
                && binding_expansion.radix.attributes.is_empty()
                && intermediate_expansion.attributes.is_empty()
                && intermediate_expansion.radix.head == *asserted_head
                && intermediate_expansion.radix.spelling == expected_asserted_spelling
                && intermediate_expansion.radix.args.is_empty()
                && intermediate_expansion.radix.attributes.is_empty()
                && source_mode_terminal_builtin_input(
                    asserted_symbol,
                    config.asserted_type,
                    mode_expansions,
                )
                .is_some()
        }
        SourceReservedVariableAssertedHeadRelation::BindingThreeHopRadix {
            first_intermediate_spelling,
            second_intermediate_spelling,
            asserted_spelling: expected_asserted_spelling,
        } => {
            let Some(binding_spelling) = config.binding_source_mode_spelling else {
                return false;
            };
            let TypeHeadInput::Symbol(binding_symbol) = &source_binding.type_head else {
                return false;
            };
            let Some(binding_expansion) = mode_expansions.get(binding_symbol) else {
                return false;
            };
            let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head
            else {
                return false;
            };
            let Some(first_intermediate_expansion) = mode_expansions.get(first_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(second_intermediate_symbol) =
                &first_intermediate_expansion.radix.head
            else {
                return false;
            };
            let Some(second_intermediate_expansion) =
                mode_expansions.get(second_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(asserted_symbol) = asserted_head else {
                return false;
            };
            let spellings = [
                binding_spelling,
                first_intermediate_spelling,
                second_intermediate_spelling,
                expected_asserted_spelling,
            ];
            let symbols = [
                binding_symbol,
                first_intermediate_symbol,
                second_intermediate_symbol,
                asserted_symbol,
            ];
            spellings.iter().enumerate().all(|(index, spelling)| {
                spellings[index + 1..].iter().all(|other| spelling != other)
            }) && symbols
                .iter()
                .enumerate()
                .all(|(index, symbol)| symbols[index + 1..].iter().all(|other| symbol != other))
                && source_binding.type_spelling == binding_spelling
                && asserted_spelling == expected_asserted_spelling
                && binding_expansion.attributes.is_empty()
                && binding_expansion.radix.spelling == first_intermediate_spelling
                && binding_expansion.radix.args.is_empty()
                && binding_expansion.radix.attributes.is_empty()
                && first_intermediate_expansion.attributes.is_empty()
                && first_intermediate_expansion.radix.spelling == second_intermediate_spelling
                && first_intermediate_expansion.radix.args.is_empty()
                && first_intermediate_expansion.radix.attributes.is_empty()
                && second_intermediate_expansion.attributes.is_empty()
                && second_intermediate_expansion.radix.head == *asserted_head
                && second_intermediate_expansion.radix.spelling == expected_asserted_spelling
                && second_intermediate_expansion.radix.args.is_empty()
                && second_intermediate_expansion.radix.attributes.is_empty()
                && source_mode_terminal_builtin_input(
                    asserted_symbol,
                    config.asserted_type,
                    mode_expansions,
                )
                .is_some()
        }
        SourceReservedVariableAssertedHeadRelation::BindingFourHopRadix {
            first_intermediate_spelling,
            second_intermediate_spelling,
            third_intermediate_spelling,
            asserted_spelling: expected_asserted_spelling,
        } => {
            let Some(binding_spelling) = config.binding_source_mode_spelling else {
                return false;
            };
            let TypeHeadInput::Symbol(binding_symbol) = &source_binding.type_head else {
                return false;
            };
            let Some(binding_expansion) = mode_expansions.get(binding_symbol) else {
                return false;
            };
            let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head
            else {
                return false;
            };
            let Some(first_intermediate_expansion) = mode_expansions.get(first_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(second_intermediate_symbol) =
                &first_intermediate_expansion.radix.head
            else {
                return false;
            };
            let Some(second_intermediate_expansion) =
                mode_expansions.get(second_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(third_intermediate_symbol) =
                &second_intermediate_expansion.radix.head
            else {
                return false;
            };
            let Some(third_intermediate_expansion) = mode_expansions.get(third_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(asserted_symbol) = asserted_head else {
                return false;
            };
            let spellings = [
                binding_spelling,
                first_intermediate_spelling,
                second_intermediate_spelling,
                third_intermediate_spelling,
                expected_asserted_spelling,
            ];
            let symbols = [
                binding_symbol,
                first_intermediate_symbol,
                second_intermediate_symbol,
                third_intermediate_symbol,
                asserted_symbol,
            ];
            spellings.iter().enumerate().all(|(index, spelling)| {
                spellings[index + 1..].iter().all(|other| spelling != other)
            }) && symbols
                .iter()
                .enumerate()
                .all(|(index, symbol)| symbols[index + 1..].iter().all(|other| symbol != other))
                && source_binding.type_spelling == binding_spelling
                && asserted_spelling == expected_asserted_spelling
                && binding_expansion.attributes.is_empty()
                && binding_expansion.radix.spelling == first_intermediate_spelling
                && binding_expansion.radix.args.is_empty()
                && binding_expansion.radix.attributes.is_empty()
                && first_intermediate_expansion.attributes.is_empty()
                && first_intermediate_expansion.radix.spelling == second_intermediate_spelling
                && first_intermediate_expansion.radix.args.is_empty()
                && first_intermediate_expansion.radix.attributes.is_empty()
                && second_intermediate_expansion.attributes.is_empty()
                && second_intermediate_expansion.radix.spelling == third_intermediate_spelling
                && second_intermediate_expansion.radix.args.is_empty()
                && second_intermediate_expansion.radix.attributes.is_empty()
                && third_intermediate_expansion.attributes.is_empty()
                && third_intermediate_expansion.radix.head == *asserted_head
                && third_intermediate_expansion.radix.spelling == expected_asserted_spelling
                && third_intermediate_expansion.radix.args.is_empty()
                && third_intermediate_expansion.radix.attributes.is_empty()
                && source_mode_terminal_builtin_input(
                    asserted_symbol,
                    config.asserted_type,
                    mode_expansions,
                )
                .is_some()
        }
        SourceReservedVariableAssertedHeadRelation::BindingFiveHopRadix {
            first_intermediate_spelling,
            second_intermediate_spelling,
            third_intermediate_spelling,
            fourth_intermediate_spelling,
            asserted_spelling: expected_asserted_spelling,
        } => {
            let Some(binding_spelling) = config.binding_source_mode_spelling else {
                return false;
            };
            let TypeHeadInput::Symbol(binding_symbol) = &source_binding.type_head else {
                return false;
            };
            let Some(binding_expansion) = mode_expansions.get(binding_symbol) else {
                return false;
            };
            let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head
            else {
                return false;
            };
            let Some(first_intermediate_expansion) = mode_expansions.get(first_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(second_intermediate_symbol) =
                &first_intermediate_expansion.radix.head
            else {
                return false;
            };
            let Some(second_intermediate_expansion) =
                mode_expansions.get(second_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(third_intermediate_symbol) =
                &second_intermediate_expansion.radix.head
            else {
                return false;
            };
            let Some(third_intermediate_expansion) = mode_expansions.get(third_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(fourth_intermediate_symbol) =
                &third_intermediate_expansion.radix.head
            else {
                return false;
            };
            let Some(fourth_intermediate_expansion) =
                mode_expansions.get(fourth_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(asserted_symbol) = asserted_head else {
                return false;
            };
            let spellings = [
                binding_spelling,
                first_intermediate_spelling,
                second_intermediate_spelling,
                third_intermediate_spelling,
                fourth_intermediate_spelling,
                expected_asserted_spelling,
            ];
            let symbols = [
                binding_symbol,
                first_intermediate_symbol,
                second_intermediate_symbol,
                third_intermediate_symbol,
                fourth_intermediate_symbol,
                asserted_symbol,
            ];
            spellings.iter().enumerate().all(|(index, spelling)| {
                spellings[index + 1..].iter().all(|other| spelling != other)
            }) && symbols
                .iter()
                .enumerate()
                .all(|(index, symbol)| symbols[index + 1..].iter().all(|other| symbol != other))
                && source_binding.type_spelling == binding_spelling
                && asserted_spelling == expected_asserted_spelling
                && binding_expansion.attributes.is_empty()
                && binding_expansion.radix.spelling == first_intermediate_spelling
                && binding_expansion.radix.args.is_empty()
                && binding_expansion.radix.attributes.is_empty()
                && first_intermediate_expansion.attributes.is_empty()
                && first_intermediate_expansion.radix.spelling == second_intermediate_spelling
                && first_intermediate_expansion.radix.args.is_empty()
                && first_intermediate_expansion.radix.attributes.is_empty()
                && second_intermediate_expansion.attributes.is_empty()
                && second_intermediate_expansion.radix.spelling == third_intermediate_spelling
                && second_intermediate_expansion.radix.args.is_empty()
                && second_intermediate_expansion.radix.attributes.is_empty()
                && third_intermediate_expansion.attributes.is_empty()
                && third_intermediate_expansion.radix.spelling == fourth_intermediate_spelling
                && third_intermediate_expansion.radix.args.is_empty()
                && third_intermediate_expansion.radix.attributes.is_empty()
                && fourth_intermediate_expansion.attributes.is_empty()
                && fourth_intermediate_expansion.radix.head == *asserted_head
                && fourth_intermediate_expansion.radix.spelling == expected_asserted_spelling
                && fourth_intermediate_expansion.radix.args.is_empty()
                && fourth_intermediate_expansion.radix.attributes.is_empty()
                && source_mode_terminal_builtin_input(
                    asserted_symbol,
                    config.asserted_type,
                    mode_expansions,
                )
                .is_some()
        }
        SourceReservedVariableAssertedHeadRelation::BindingSixHopRadix {
            first_intermediate_spelling,
            second_intermediate_spelling,
            third_intermediate_spelling,
            fourth_intermediate_spelling,
            fifth_intermediate_spelling,
            asserted_spelling: expected_asserted_spelling,
        } => {
            let Some(binding_spelling) = config.binding_source_mode_spelling else {
                return false;
            };
            let TypeHeadInput::Symbol(binding_symbol) = &source_binding.type_head else {
                return false;
            };
            let Some(binding_expansion) = mode_expansions.get(binding_symbol) else {
                return false;
            };
            let TypeHeadInput::Symbol(first_intermediate_symbol) = &binding_expansion.radix.head
            else {
                return false;
            };
            let Some(first_intermediate_expansion) = mode_expansions.get(first_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(second_intermediate_symbol) =
                &first_intermediate_expansion.radix.head
            else {
                return false;
            };
            let Some(second_intermediate_expansion) =
                mode_expansions.get(second_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(third_intermediate_symbol) =
                &second_intermediate_expansion.radix.head
            else {
                return false;
            };
            let Some(third_intermediate_expansion) = mode_expansions.get(third_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(fourth_intermediate_symbol) =
                &third_intermediate_expansion.radix.head
            else {
                return false;
            };
            let Some(fourth_intermediate_expansion) =
                mode_expansions.get(fourth_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(fifth_intermediate_symbol) =
                &fourth_intermediate_expansion.radix.head
            else {
                return false;
            };
            let Some(fifth_intermediate_expansion) = mode_expansions.get(fifth_intermediate_symbol)
            else {
                return false;
            };
            let TypeHeadInput::Symbol(asserted_symbol) = asserted_head else {
                return false;
            };
            let spellings = [
                binding_spelling,
                first_intermediate_spelling,
                second_intermediate_spelling,
                third_intermediate_spelling,
                fourth_intermediate_spelling,
                fifth_intermediate_spelling,
                expected_asserted_spelling,
            ];
            let symbols = [
                binding_symbol,
                first_intermediate_symbol,
                second_intermediate_symbol,
                third_intermediate_symbol,
                fourth_intermediate_symbol,
                fifth_intermediate_symbol,
                asserted_symbol,
            ];
            spellings.iter().enumerate().all(|(index, spelling)| {
                spellings[index + 1..].iter().all(|other| spelling != other)
            }) && symbols
                .iter()
                .enumerate()
                .all(|(index, symbol)| symbols[index + 1..].iter().all(|other| symbol != other))
                && source_binding.type_spelling == binding_spelling
                && asserted_spelling == expected_asserted_spelling
                && binding_expansion.attributes.is_empty()
                && binding_expansion.radix.spelling == first_intermediate_spelling
                && binding_expansion.radix.args.is_empty()
                && binding_expansion.radix.attributes.is_empty()
                && first_intermediate_expansion.attributes.is_empty()
                && first_intermediate_expansion.radix.spelling == second_intermediate_spelling
                && first_intermediate_expansion.radix.args.is_empty()
                && first_intermediate_expansion.radix.attributes.is_empty()
                && second_intermediate_expansion.attributes.is_empty()
                && second_intermediate_expansion.radix.spelling == third_intermediate_spelling
                && second_intermediate_expansion.radix.args.is_empty()
                && second_intermediate_expansion.radix.attributes.is_empty()
                && third_intermediate_expansion.attributes.is_empty()
                && third_intermediate_expansion.radix.spelling == fourth_intermediate_spelling
                && third_intermediate_expansion.radix.args.is_empty()
                && third_intermediate_expansion.radix.attributes.is_empty()
                && fourth_intermediate_expansion.attributes.is_empty()
                && fourth_intermediate_expansion.radix.spelling == fifth_intermediate_spelling
                && fourth_intermediate_expansion.radix.args.is_empty()
                && fourth_intermediate_expansion.radix.attributes.is_empty()
                && fifth_intermediate_expansion.attributes.is_empty()
                && fifth_intermediate_expansion.radix.head == *asserted_head
                && fifth_intermediate_expansion.radix.spelling == expected_asserted_spelling
                && fifth_intermediate_expansion.radix.args.is_empty()
                && fifth_intermediate_expansion.radix.attributes.is_empty()
                && source_mode_terminal_builtin_input(
                    asserted_symbol,
                    config.asserted_type,
                    mode_expansions,
                )
                .is_some()
        }
    }
}

pub(in crate::runner) fn source_mode_terminal_builtin_input<'a>(
    symbol: &ResolverSymbolId,
    expected_type: SourceReservedVariableBuiltinType,
    mode_expansions: &'a BTreeMap<ResolverSymbolId, ModeExpansion>,
) -> Option<&'a TypeExpressionInput> {
    let mut visiting = BTreeSet::new();
    let mut current = symbol;
    while visiting.insert(current.clone()) {
        let expansion = mode_expansions.get(current)?;
        if !expansion.attributes.is_empty()
            || !expansion.radix.args.is_empty()
            || !expansion.radix.attributes.is_empty()
        {
            return None;
        }
        match &expansion.radix.head {
            head if head == &expected_type.input_head()
                && expansion.radix.spelling == expected_type.spelling() =>
            {
                return Some(&expansion.radix);
            }
            TypeHeadInput::Symbol(next) => current = next,
            _ => return None,
        }
    }
    None
}

pub(in crate::runner) fn source_reserved_variable_mode_definition_is_exact(
    ast: &SurfaceAst,
    mode_definition_specs: &[SourceReservedVariableModeDefinition],
) -> bool {
    let definition_blocks = surface_nodes_with_kind(ast, SurfaceNodeKind::DefinitionBlockItem);
    let mode_definitions = surface_nodes_with_kind(ast, SurfaceNodeKind::ModeDefinition);
    if mode_definition_specs.is_empty() {
        return definition_blocks.is_empty() && mode_definitions.is_empty();
    }
    if definition_blocks.len() != mode_definition_specs.len()
        || mode_definitions.len() != mode_definition_specs.len()
    {
        return false;
    }
    definition_blocks
        .iter()
        .zip(mode_definitions.iter())
        .zip(mode_definition_specs.iter())
        .all(|(((_, block), (definition_id, definition)), expected)| {
            if subtree_has_recovery(ast, block)
                || direct_token_texts(ast, block).as_slice() != ["definition", "end", ";"]
                || structural_child_ids(ast, block).as_slice() != [*definition_id]
                || direct_token_texts(ast, definition).as_slice()
                    != ["mode", expected.label, ":", "is", ";"]
            {
                return false;
            }
            let definition_children = structural_child_ids(ast, definition);
            let [pattern_id, rhs_id] = definition_children.as_slice() else {
                return false;
            };
            ast.node(*pattern_id)
                .is_some_and(|pattern| matches!(pattern.kind, SurfaceNodeKind::ModePattern))
                && ast
                    .node(*rhs_id)
                    .is_some_and(|rhs| matches!(rhs.kind, SurfaceNodeKind::TypeExpression))
                && mode_definition_pattern_spelling(ast, definition).as_deref()
                    == Some(expected.spelling)
        })
}

pub(in crate::runner) fn source_reserved_variable_mode_expansions_are_exact(
    reserve: &SourceReserveExtraction,
    mode_definition_specs: &[SourceReservedVariableModeDefinition],
) -> bool {
    if reserve.mode_expansions.len() != mode_definition_specs.len() {
        return false;
    }
    mode_definition_specs.iter().all(|expected| {
        let Some((_, expansion)) =
            source_mode_expansion_for_spelling(&reserve.mode_expansions, expected.spelling)
        else {
            return false;
        };
        if !expansion.attributes.is_empty()
            || !expansion.radix.args.is_empty()
            || !expansion.radix.attributes.is_empty()
        {
            return false;
        }
        match expected.radix {
            SourceReservedVariableModeRadix::Builtin(expected_type) => {
                expansion.radix.head == expected_type.input_head()
                    && expansion.radix.spelling == expected_type.spelling()
            }
            SourceReservedVariableModeRadix::Mode(expected_spelling) => {
                expansion.radix.spelling == expected_spelling
                    && matches!(
                        &expansion.radix.head,
                        TypeHeadInput::Symbol(symbol)
                            if source_mode_symbol_spelling(symbol) == Some(expected_spelling)
                    )
            }
        }
    })
}

fn source_mode_expansion_for_spelling<'a>(
    mode_expansions: &'a BTreeMap<ResolverSymbolId, ModeExpansion>,
    spelling: &str,
) -> Option<(&'a ResolverSymbolId, &'a ModeExpansion)> {
    let mut matches = mode_expansions
        .iter()
        .filter(|(symbol, _)| source_mode_symbol_spelling(symbol) == Some(spelling));
    let matched = matches.next()?;
    matches.next().is_none().then_some(matched)
}

pub(in crate::runner) fn exact_identifier_term_operand(
    ast: &SurfaceAst,
    term_expression_id: SurfaceNodeId,
) -> Option<(SurfaceNodeId, SourceRange, String)> {
    let expression = ast.node(term_expression_id)?;
    if !matches!(expression.kind, SurfaceNodeKind::TermExpression)
        || subtree_has_recovery(ast, expression)
    {
        return None;
    }
    let expression_children = structural_child_ids(ast, expression);
    let [reference_id] = expression_children.as_slice() else {
        return None;
    };
    let reference = ast.node(*reference_id)?;
    let direct_tokens = direct_token_texts(ast, reference);
    let [spelling] = direct_tokens.as_slice() else {
        return None;
    };
    if !matches!(reference.kind, SurfaceNodeKind::TermReference)
        || !structural_child_ids(ast, reference).is_empty()
    {
        return None;
    }
    Some((*reference_id, reference.range, (*spelling).to_owned()))
}

pub(in crate::runner) fn source_binding_use_ordinals<const N: usize>(
    bindings: &[SourceReserveBindingInput],
    use_ranges: [SourceRange; N],
) -> Result<[usize; N], String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Event {
        Binding(usize),
        Use(usize),
    }

    let mut events = bindings
        .iter()
        .enumerate()
        .map(|(index, binding)| (binding.binding_range, Event::Binding(index)))
        .chain(
            use_ranges
                .iter()
                .copied()
                .enumerate()
                .map(|(index, range)| (range, Event::Use(index))),
        )
        .collect::<Vec<_>>();
    if events.iter().any(|(range, _)| range.start >= range.end) {
        return Err("source binding/use event has an empty range".to_owned());
    }
    events.sort_by_key(|(range, _)| (range.start, range.end));
    if events
        .windows(2)
        .any(|pair| pair[0].0.source_id != pair[1].0.source_id || pair[0].0.end > pair[1].0.start)
    {
        return Err("source binding/use events overlap or cross sources".to_owned());
    }

    let mut ordinals = [usize::MAX; N];
    for (ordinal, (_, event)) in events.into_iter().enumerate() {
        match event {
            Event::Binding(index) if index == ordinal && index < bindings.len() => {}
            Event::Binding(_) => {
                return Err("source binding order does not match binding ordinals".to_owned());
            }
            Event::Use(index) if index < N => ordinals[index] = ordinal,
            Event::Use(_) => return Err("source use ordinal index overflow".to_owned()),
        }
    }
    if ordinals.contains(&usize::MAX) {
        return Err("source use ordinal missing".to_owned());
    }
    Ok(ordinals)
}

pub(in crate::runner) fn resolve_imported_fixture_term_formula_symbol(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    spelling: &str,
    kind: SymbolKind,
) -> Result<ResolverSymbolId, ()> {
    let namespace = NamespacePath::new(module.path().as_str());
    let candidates = symbols
        .symbols()
        .visible_candidates(&namespace, spelling)
        .into_iter()
        .filter(|entry| entry.kind() == kind)
        .collect::<Vec<_>>();
    let [entry] = candidates.as_slice() else {
        return Err(());
    };
    if is_imported_fixture_term_formula_symbol(symbols, module, entry.symbol(), spelling, kind) {
        Ok(entry.symbol().clone())
    } else {
        Err(())
    }
}

fn is_imported_fixture_term_formula_symbol(
    symbols: &SymbolEnv,
    module: &ResolverModuleId,
    symbol: &ResolverSymbolId,
    spelling: &str,
    kind: SymbolKind,
) -> bool {
    let Some(entry) = symbols.symbols().get(symbol) else {
        return false;
    };
    let Some(contribution) = symbols.contributions().get(entry.contribution()) else {
        return false;
    };
    symbol.module() != module
        && contribution.module() == symbol.module()
        && matches!(contribution.kind(), ContributionKind::ImportedSource { .. })
        && symbol.module().path().as_str() == "parser.type_fixtures"
        && entry.kind() == kind
        && entry.primary_spelling() == spelling
}

#[derive(Debug, Clone, Copy)]
pub(in crate::runner) struct SourceBuiltinBinaryTermFormulaConfig {
    pub(in crate::runner) label: &'static str,
    pub(in crate::runner) operator: &'static str,
    pub(in crate::runner) left: &'static str,
    pub(in crate::runner) right: &'static str,
    formula_kind: FormulaKind,
}

pub(in crate::runner) const SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS:
    &[SourceBuiltinBinaryTermFormulaConfig] = &[
    SourceBuiltinBinaryTermFormulaConfig {
        label: "TermFormulaPayloadBoundary",
        operator: "=",
        left: "1",
        right: "1",
        formula_kind: FormulaKind::Equality,
    },
    SourceBuiltinBinaryTermFormulaConfig {
        label: "BuiltinInequalityPayloadBoundary",
        operator: "<>",
        left: "1",
        right: "2",
        formula_kind: FormulaKind::Inequality,
    },
    SourceBuiltinBinaryTermFormulaConfig {
        label: "BuiltinMembershipPayloadBoundary",
        operator: "in",
        left: "1",
        right: "1",
        formula_kind: FormulaKind::Membership,
    },
];

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceBuiltinBinaryTermFormula {
    pub(in crate::runner) formula_site: TypedSiteRef,
    pub(in crate::runner) formula_range: SourceRange,
    pub(in crate::runner) formula_kind: FormulaKind,
    pub(in crate::runner) left_site: TypedSiteRef,
    pub(in crate::runner) left_range: SourceRange,
    pub(in crate::runner) right_site: TypedSiteRef,
    pub(in crate::runner) right_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceBuiltinTypeAssertionFormula {
    pub(in crate::runner) formula_site: TypedSiteRef,
    pub(in crate::runner) formula_range: SourceRange,
    pub(in crate::runner) subject_site: TypedSiteRef,
    pub(in crate::runner) subject_range: SourceRange,
    pub(in crate::runner) asserted_type_site: TypedSiteRef,
    pub(in crate::runner) asserted_type: SourceTypeExpression,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceImportedPredicateFunctorFormula {
    pub(in crate::runner) formula_site: TypedSiteRef,
    pub(in crate::runner) formula_range: SourceRange,
    pub(in crate::runner) predicate_symbol: ResolverSymbolId,
    pub(in crate::runner) left_site: TypedSiteRef,
    pub(in crate::runner) left_range: SourceRange,
    pub(in crate::runner) functor_site: TypedSiteRef,
    pub(in crate::runner) functor_range: SourceRange,
    pub(in crate::runner) functor_symbol: ResolverSymbolId,
    pub(in crate::runner) functor_left_site: TypedSiteRef,
    pub(in crate::runner) functor_left_range: SourceRange,
    pub(in crate::runner) functor_right_site: TypedSiteRef,
    pub(in crate::runner) functor_right_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceImportedAttributeAssertionFormula {
    pub(in crate::runner) formula_site: TypedSiteRef,
    pub(in crate::runner) formula_range: SourceRange,
    pub(in crate::runner) subject_site: TypedSiteRef,
    pub(in crate::runner) subject_range: SourceRange,
    pub(in crate::runner) attribute_symbol: ResolverSymbolId,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceSetEnumerationFormula {
    pub(in crate::runner) formula_site: TypedSiteRef,
    pub(in crate::runner) formula_range: SourceRange,
    pub(in crate::runner) left_site: TypedSiteRef,
    pub(in crate::runner) left_range: SourceRange,
    pub(in crate::runner) left_items: Vec<(TypedSiteRef, SourceRange)>,
    pub(in crate::runner) right_site: TypedSiteRef,
    pub(in crate::runner) right_range: SourceRange,
    pub(in crate::runner) right_items: Vec<(TypedSiteRef, SourceRange)>,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceFormulaConnectiveQuantifier {
    pub(in crate::runner) premise_constant_site: TypedSiteRef,
    pub(in crate::runner) premise_constant_range: SourceRange,
    pub(in crate::runner) implication_site: TypedSiteRef,
    pub(in crate::runner) implication_range: SourceRange,
    pub(in crate::runner) quantified_site: TypedSiteRef,
    pub(in crate::runner) quantified_range: SourceRange,
    pub(in crate::runner) negation_site: TypedSiteRef,
    pub(in crate::runner) negation_range: SourceRange,
    pub(in crate::runner) body_constant_site: TypedSiteRef,
    pub(in crate::runner) body_constant_range: SourceRange,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SourceFormulaStatement {
    pub(in crate::runner) formula_site: TypedSiteRef,
    pub(in crate::runner) formula_range: SourceRange,
}

pub(in crate::runner) fn extract_source_formula_statement(
    ast: &SurfaceAst,
) -> Option<SourceFormulaStatement> {
    extract_exact_source_formula_constant(
        ast,
        "FormulaPayloadBoundary",
        SurfaceFormulaConstant::Thesis,
    )
}

pub(in crate::runner) fn extract_source_contradiction_formula(
    ast: &SurfaceAst,
) -> Option<SourceFormulaStatement> {
    extract_exact_source_formula_constant(
        ast,
        "SourceDerivedContradictionConstantBoundary",
        SurfaceFormulaConstant::Contradiction,
    )
}

pub(in crate::runner) fn extract_source_builtin_binary_term_formula(
    ast: &SurfaceAst,
) -> Option<SourceBuiltinBinaryTermFormula> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_builtin_binary_theorem_bridge_node(node))
    {
        return None;
    }
    let theorem_items = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
    let [(_, theorem)] = theorem_items.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, theorem) {
        return None;
    }
    let theorem_tokens = direct_token_texts(ast, theorem);
    let config = SOURCE_BUILTIN_BINARY_TERM_FORMULA_CONFIGS
        .iter()
        .copied()
        .find(|config| theorem_tokens.as_slice() == ["theorem", config.label, ":", ";"])?;

    let theorem_structural_children = structural_child_ids(ast, theorem);
    let formula_expressions = theorem_structural_children
        .iter()
        .copied()
        .filter(|id| {
            ast.node(*id)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::FormulaExpression))
        })
        .collect::<Vec<_>>();
    if formula_expressions.len() != 1
        || theorem_structural_children
            .iter()
            .any(|child| !formula_expressions.contains(child))
    {
        return None;
    }
    let formula_expression = ast.node(formula_expressions[0])?;
    let formula_children = structural_child_ids(ast, formula_expression);
    let [formula_id] = formula_children.as_slice() else {
        return None;
    };
    let formula = ast.node(*formula_id)?;
    let operator_tokens = direct_token_texts(ast, formula);
    if !matches!(formula.kind, SurfaceNodeKind::BuiltinPredicateApplication)
        || subtree_has_recovery(ast, formula)
        || operator_tokens.len() != 1
        || operator_tokens[0] != config.operator
    {
        return None;
    }

    let predicate_structural_children = structural_child_ids(ast, formula);
    let term_expressions = predicate_structural_children
        .iter()
        .copied()
        .filter(|id| {
            ast.node(*id)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
        })
        .collect::<Vec<_>>();
    if term_expressions.len() != 2
        || predicate_structural_children
            .iter()
            .any(|child| !term_expressions.contains(child))
    {
        return None;
    }

    let left = exact_numeral_term_operand(ast, term_expressions[0], config.left)?;
    let right = exact_numeral_term_operand(ast, term_expressions[1], config.right)?;
    Some(SourceBuiltinBinaryTermFormula {
        formula_site: surface_site(*formula_id),
        formula_range: formula.range,
        formula_kind: config.formula_kind,
        left_site: surface_site(left.0),
        left_range: left.1,
        right_site: surface_site(right.0),
        right_range: right.1,
    })
}

pub(in crate::runner) fn extract_source_builtin_type_assertion_formula(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceBuiltinTypeAssertionFormula> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_builtin_type_assertion_theorem_bridge_node(node))
    {
        return None;
    }
    let theorem_items = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
    let [(_, theorem)] = theorem_items.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, theorem) {
        return None;
    }
    let theorem_tokens = direct_token_texts(ast, theorem);
    if theorem_tokens.as_slice() != ["theorem", "BuiltinTypeAssertionPayloadBoundary", ":", ";"] {
        return None;
    }

    let theorem_structural_children = structural_child_ids(ast, theorem);
    let formula_expressions = theorem_structural_children
        .iter()
        .copied()
        .filter(|id| {
            ast.node(*id)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::FormulaExpression))
        })
        .collect::<Vec<_>>();
    if formula_expressions.len() != 1
        || theorem_structural_children
            .iter()
            .any(|child| !formula_expressions.contains(child))
    {
        return None;
    }
    let formula_expression = ast.node(formula_expressions[0])?;
    let formula_children = structural_child_ids(ast, formula_expression);
    let [formula_id] = formula_children.as_slice() else {
        return None;
    };
    let formula = ast.node(*formula_id)?;
    if !matches!(formula.kind, SurfaceNodeKind::IsAssertion)
        || subtree_has_recovery(ast, formula)
        || direct_token_texts(ast, formula).as_slice() != ["is"]
    {
        return None;
    }

    let assertion_structural_children = structural_child_ids(ast, formula);
    let [term_expression_id, type_expression_id] = assertion_structural_children.as_slice() else {
        return None;
    };
    let term_expression = ast.node(*term_expression_id)?;
    let type_expression = ast.node(*type_expression_id)?;
    if !matches!(term_expression.kind, SurfaceNodeKind::TermExpression)
        || !matches!(type_expression.kind, SurfaceNodeKind::TypeExpression)
    {
        return None;
    }
    let subject = exact_numeral_term_operand(ast, *term_expression_id, "1")?;
    let asserted_type =
        extract_builtin_source_type_expression(ast, type_expression, module, symbols).ok()?;
    if asserted_type.spelling != "set"
        || asserted_type.head != TypeHeadInput::BuiltinSet
        || !asserted_type.attributes.is_empty()
    {
        return None;
    }
    Some(SourceBuiltinTypeAssertionFormula {
        formula_site: surface_site(*formula_id),
        formula_range: formula.range,
        subject_site: surface_site(subject.0),
        subject_range: subject.1,
        asserted_type_site: surface_site(*type_expression_id),
        asserted_type,
    })
}

pub(in crate::runner) fn extract_source_imported_predicate_functor_formula(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceImportedPredicateFunctorFormula> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_imported_predicate_functor_theorem_bridge_node(node))
    {
        return None;
    }

    let item_list = exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let [import_item_id, theorem_id] = item_children.as_slice() else {
        return None;
    };
    let import_item = ast.node(*import_item_id)?;
    if !is_exact_parser_type_fixtures_import(ast, import_item) {
        return None;
    }

    let theorem = ast.node(*theorem_id)?;
    if !matches!(theorem.kind, SurfaceNodeKind::TheoremItem) || subtree_has_recovery(ast, theorem) {
        return None;
    }
    let theorem_tokens = direct_token_texts(ast, theorem);
    if theorem_tokens.as_slice()
        != [
            "theorem",
            "ImportedPredicateFunctorPayloadBoundary",
            ":",
            ";",
        ]
    {
        return None;
    }

    let theorem_structural_children = structural_child_ids(ast, theorem);
    let [formula_expression_id] = theorem_structural_children.as_slice() else {
        return None;
    };
    let formula_expression = ast.node(*formula_expression_id)?;
    if !matches!(formula_expression.kind, SurfaceNodeKind::FormulaExpression) {
        return None;
    }
    let formula_children = structural_child_ids(ast, formula_expression);
    let [formula_id] = formula_children.as_slice() else {
        return None;
    };
    let formula = ast.node(*formula_id)?;
    if !matches!(formula.kind, SurfaceNodeKind::PredicateApplication)
        || subtree_has_recovery(ast, formula)
        || !direct_token_texts(ast, formula).is_empty()
    {
        return None;
    }

    let predicate_children = structural_child_ids(ast, formula);
    let [segment_id] = predicate_children.as_slice() else {
        return None;
    };
    let segment = ast.node(*segment_id)?;
    if !matches!(segment.kind, SurfaceNodeKind::PredicateSegment)
        || !direct_token_texts(ast, segment).is_empty()
    {
        return None;
    }
    let segment_children = structural_child_ids(ast, segment);
    let [
        left_term_expression_id,
        predicate_head_id,
        right_term_expression_id,
    ] = segment_children.as_slice()
    else {
        return None;
    };

    let predicate_head = ast.node(*predicate_head_id)?;
    if !matches!(predicate_head.kind, SurfaceNodeKind::PredicateHead)
        || !direct_token_texts(ast, predicate_head).is_empty()
    {
        return None;
    }
    let predicate_head_children = structural_child_ids(ast, predicate_head);
    let [predicate_symbol_id] = predicate_head_children.as_slice() else {
        return None;
    };
    let predicate_symbol_node = ast.node(*predicate_symbol_id)?;
    if !matches!(predicate_symbol_node.kind, SurfaceNodeKind::QualifiedSymbol)
        || qualified_symbol_spelling(ast, predicate_symbol_node)
            .ok()?
            .as_str()
            != "divides"
    {
        return None;
    }
    let predicate_symbol = resolve_imported_fixture_term_formula_symbol(
        symbols,
        module,
        "divides",
        SymbolKind::Predicate,
    )
    .ok()?;

    let left = exact_numeral_term_operand(ast, *left_term_expression_id, "1")?;
    let functor = exact_imported_infix_functor_term(ast, *right_term_expression_id)?;
    let functor_symbol =
        resolve_imported_fixture_term_formula_symbol(symbols, module, "++", SymbolKind::Functor)
            .ok()?;

    Some(SourceImportedPredicateFunctorFormula {
        formula_site: surface_site(*formula_id),
        formula_range: formula.range,
        predicate_symbol,
        left_site: surface_site(left.0),
        left_range: left.1,
        functor_site: surface_site(functor.term_id),
        functor_range: functor.term_range,
        functor_symbol,
        functor_left_site: surface_site(functor.left.0),
        functor_left_range: functor.left.1,
        functor_right_site: surface_site(functor.right.0),
        functor_right_range: functor.right.1,
    })
}

pub(in crate::runner) fn extract_source_imported_attribute_assertion_formula(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceImportedAttributeAssertionFormula> {
    extract_source_imported_attribute_assertion_formula_with_shape(
        ast,
        module,
        symbols,
        "ImportedAttributeAssertionPayloadBoundary",
        false,
    )
}

pub(in crate::runner) fn extract_source_imported_non_empty_attribute_assertion_formula(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceImportedAttributeAssertionFormula> {
    extract_source_imported_attribute_assertion_formula_with_shape(
        ast,
        module,
        symbols,
        "ImportedNonEmptyAttributeAssertionPayloadBoundary",
        true,
    )
}

fn extract_source_imported_attribute_assertion_formula_with_shape(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
    expected_label: &str,
    negative_attribute: bool,
) -> Option<SourceImportedAttributeAssertionFormula> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_imported_attribute_assertion_theorem_bridge_node(node))
    {
        return None;
    }

    let item_list = exact_compilation_item_list(ast)?;
    let item_children = structural_child_ids(ast, item_list);
    let [import_item_id, theorem_id] = item_children.as_slice() else {
        return None;
    };
    let import_item = ast.node(*import_item_id)?;
    if !is_exact_parser_type_fixtures_import(ast, import_item) {
        return None;
    }

    let theorem = ast.node(*theorem_id)?;
    if !matches!(theorem.kind, SurfaceNodeKind::TheoremItem) || subtree_has_recovery(ast, theorem) {
        return None;
    }
    let theorem_tokens = direct_token_texts(ast, theorem);
    if theorem_tokens.as_slice() != ["theorem", expected_label, ":", ";"] {
        return None;
    }

    let theorem_structural_children = structural_child_ids(ast, theorem);
    let [formula_expression_id] = theorem_structural_children.as_slice() else {
        return None;
    };
    let formula_expression = ast.node(*formula_expression_id)?;
    if !matches!(formula_expression.kind, SurfaceNodeKind::FormulaExpression) {
        return None;
    }
    let formula_children = structural_child_ids(ast, formula_expression);
    let [formula_id] = formula_children.as_slice() else {
        return None;
    };
    let formula = ast.node(*formula_id)?;
    if !matches!(formula.kind, SurfaceNodeKind::IsAssertion)
        || subtree_has_recovery(ast, formula)
        || direct_token_texts(ast, formula).as_slice() != ["is"]
    {
        return None;
    }

    let assertion_structural_children = structural_child_ids(ast, formula);
    let [term_expression_id, attribute_chain_id] = assertion_structural_children.as_slice() else {
        return None;
    };
    let term_expression = ast.node(*term_expression_id)?;
    let attribute_chain = ast.node(*attribute_chain_id)?;
    if !matches!(term_expression.kind, SurfaceNodeKind::TermExpression)
        || !matches!(attribute_chain.kind, SurfaceNodeKind::AttributeTestChain)
        || !direct_token_texts(ast, attribute_chain).is_empty()
    {
        return None;
    }

    let attribute_children = structural_child_ids(ast, attribute_chain);
    let [attribute_ref_id] = attribute_children.as_slice() else {
        return None;
    };
    let attribute_ref = ast.node(*attribute_ref_id)?;
    if !matches!(attribute_ref.kind, SurfaceNodeKind::AttributeRef) {
        return None;
    }
    let attribute_ref_tokens = direct_token_texts(ast, attribute_ref);
    if negative_attribute {
        if attribute_ref_tokens.as_slice() != ["non"] {
            return None;
        }
    } else if !attribute_ref_tokens.is_empty() {
        return None;
    }
    let attribute_ref_children = structural_child_ids(ast, attribute_ref);
    let [attribute_symbol_id] = attribute_ref_children.as_slice() else {
        return None;
    };
    let attribute_symbol_node = ast.node(*attribute_symbol_id)?;
    if !matches!(attribute_symbol_node.kind, SurfaceNodeKind::QualifiedSymbol)
        || qualified_symbol_spelling(ast, attribute_symbol_node)
            .ok()?
            .as_str()
            != "empty"
    {
        return None;
    }
    let attribute_symbol = resolve_imported_fixture_term_formula_symbol(
        symbols,
        module,
        "empty",
        SymbolKind::Attribute,
    )
    .ok()?;
    let subject = exact_numeral_term_operand(ast, *term_expression_id, "1")?;

    Some(SourceImportedAttributeAssertionFormula {
        formula_site: surface_site(*formula_id),
        formula_range: formula.range,
        subject_site: surface_site(subject.0),
        subject_range: subject.1,
        attribute_symbol,
    })
}

pub(in crate::runner) fn extract_source_set_enumeration_formula(
    ast: &SurfaceAst,
) -> Option<SourceSetEnumerationFormula> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_set_enumeration_theorem_bridge_node(node))
    {
        return None;
    }
    let theorem_items = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
    let [(_, theorem)] = theorem_items.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, theorem) {
        return None;
    }
    let theorem_tokens = direct_token_texts(ast, theorem);
    if theorem_tokens.as_slice() != ["theorem", "SetEnumerationPayloadBoundary", ":", ";"] {
        return None;
    }

    let theorem_structural_children = structural_child_ids(ast, theorem);
    let [formula_expression_id] = theorem_structural_children.as_slice() else {
        return None;
    };
    let formula_expression = ast.node(*formula_expression_id)?;
    if !matches!(formula_expression.kind, SurfaceNodeKind::FormulaExpression) {
        return None;
    }
    let formula_children = structural_child_ids(ast, formula_expression);
    let [formula_id] = formula_children.as_slice() else {
        return None;
    };
    let formula = ast.node(*formula_id)?;
    if !matches!(formula.kind, SurfaceNodeKind::BuiltinPredicateApplication)
        || subtree_has_recovery(ast, formula)
        || direct_token_texts(ast, formula).as_slice() != ["="]
    {
        return None;
    }

    let formula_structural_children = structural_child_ids(ast, formula);
    let [left_expression_id, right_expression_id] = formula_structural_children.as_slice() else {
        return None;
    };
    let left = exact_set_enumeration_term_operand(ast, *left_expression_id)?;
    let right = exact_set_enumeration_term_operand(ast, *right_expression_id)?;
    Some(SourceSetEnumerationFormula {
        formula_site: surface_site(*formula_id),
        formula_range: formula.range,
        left_site: surface_site(left.term_id),
        left_range: left.term_range,
        left_items: left.items,
        right_site: surface_site(right.term_id),
        right_range: right.term_range,
        right_items: right.items,
    })
}

#[derive(Debug, Clone)]
struct ExactSetEnumerationTerm {
    term_id: SurfaceNodeId,
    term_range: SourceRange,
    items: Vec<(TypedSiteRef, SourceRange)>,
}

fn exact_set_enumeration_term_operand(
    ast: &SurfaceAst,
    term_expression_id: SurfaceNodeId,
) -> Option<ExactSetEnumerationTerm> {
    let term_expression = ast.node(term_expression_id)?;
    if !matches!(term_expression.kind, SurfaceNodeKind::TermExpression)
        || subtree_has_recovery(ast, term_expression)
    {
        return None;
    }
    let term_children = structural_child_ids(ast, term_expression);
    let [set_id] = term_children.as_slice() else {
        return None;
    };
    let set = ast.node(*set_id)?;
    if !matches!(set.kind, SurfaceNodeKind::SetEnumeration)
        || subtree_has_recovery(ast, set)
        || direct_token_texts(ast, set).as_slice() != ["{", ",", "}"]
    {
        return None;
    }
    let item_children = structural_child_ids(ast, set);
    let [first_expression_id, second_expression_id] = item_children.as_slice() else {
        return None;
    };
    let first = exact_numeral_term_operand(ast, *first_expression_id, "1")?;
    let second = exact_numeral_term_operand(ast, *second_expression_id, "2")?;
    Some(ExactSetEnumerationTerm {
        term_id: *set_id,
        term_range: set.range,
        items: vec![
            (surface_site(first.0), first.1),
            (surface_site(second.0), second.1),
        ],
    })
}

pub(in crate::runner) fn extract_source_formula_connective_quantifier(
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Option<SourceFormulaConnectiveQuantifier> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_formula_connective_quantifier_theorem_bridge_node(node))
    {
        return None;
    }
    let theorem_items = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
    let [(_, theorem)] = theorem_items.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, theorem) {
        return None;
    }
    let theorem_tokens = direct_token_texts(ast, theorem);
    if theorem_tokens.as_slice()
        != [
            "theorem",
            "FormulaConnectiveQuantifierPayloadBoundary",
            ":",
            ";",
        ]
    {
        return None;
    }

    let theorem_structural_children = structural_child_ids(ast, theorem);
    let [formula_expression_id] = theorem_structural_children.as_slice() else {
        return None;
    };
    let formula_expression = ast.node(*formula_expression_id)?;
    if !matches!(formula_expression.kind, SurfaceNodeKind::FormulaExpression) {
        return None;
    }
    let formula_children = structural_child_ids(ast, formula_expression);
    let [implication_id] = formula_children.as_slice() else {
        return None;
    };
    let implication = ast.node(*implication_id)?;
    if !matches!(
        implication.kind,
        SurfaceNodeKind::BinaryFormula(operator)
            if operator.connective == SurfaceFormulaConnective::Implies && !operator.repeated
    ) || subtree_has_recovery(ast, implication)
        || direct_token_texts(ast, implication).as_slice() != ["implies"]
    {
        return None;
    }
    let implication_children = structural_child_ids(ast, implication);
    let [left_id, quantified_id] = implication_children.as_slice() else {
        return None;
    };
    let left = ast.node(*left_id)?;
    if !matches!(
        left.kind,
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction)
    ) || direct_token_texts(ast, left).as_slice() != ["contradiction"]
    {
        return None;
    }

    let quantified = ast.node(*quantified_id)?;
    if !matches!(
        quantified.kind,
        SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Universal)
    ) || subtree_has_recovery(ast, quantified)
        || direct_token_texts(ast, quantified).as_slice() != ["for", "holds"]
    {
        return None;
    }
    let quantified_children = structural_child_ids(ast, quantified);
    let [segment_id, negation_id] = quantified_children.as_slice() else {
        return None;
    };
    let segment = ast.node(*segment_id)?;
    if !matches!(segment.kind, SurfaceNodeKind::QuantifierVariableSegment)
        || subtree_has_recovery(ast, segment)
        || direct_token_texts(ast, segment).as_slice() != ["x", "being"]
    {
        return None;
    }
    let segment_children = structural_child_ids(ast, segment);
    let [type_expression_id] = segment_children.as_slice() else {
        return None;
    };
    let type_expression = ast.node(*type_expression_id)?;
    let binder_type =
        extract_builtin_source_type_expression(ast, type_expression, module, symbols).ok()?;
    if binder_type.spelling != "set"
        || binder_type.head != TypeHeadInput::BuiltinSet
        || !binder_type.attributes.is_empty()
    {
        return None;
    }

    let negation = ast.node(*negation_id)?;
    if !matches!(
        negation.kind,
        SurfaceNodeKind::PrefixFormula(SurfaceFormulaPrefixOperator::Not)
    ) || subtree_has_recovery(ast, negation)
        || direct_token_texts(ast, negation).as_slice() != ["not"]
    {
        return None;
    }
    let negation_children = structural_child_ids(ast, negation);
    let [negated_id] = negation_children.as_slice() else {
        return None;
    };
    let negated = ast.node(*negated_id)?;
    if !matches!(
        negated.kind,
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction)
    ) || direct_token_texts(ast, negated).as_slice() != ["contradiction"]
    {
        return None;
    }

    Some(SourceFormulaConnectiveQuantifier {
        premise_constant_site: surface_site(*left_id),
        premise_constant_range: left.range,
        implication_site: surface_site(*implication_id),
        implication_range: implication.range,
        quantified_site: surface_site(*quantified_id),
        quantified_range: quantified.range,
        negation_site: surface_site(*negation_id),
        negation_range: negation.range,
        body_constant_site: surface_site(*negated_id),
        body_constant_range: negated.range,
    })
}

fn extract_exact_source_formula_constant(
    ast: &SurfaceAst,
    expected_label: &str,
    expected_constant: SurfaceFormulaConstant,
) -> Option<SourceFormulaStatement> {
    if ast
        .nodes()
        .iter()
        .any(|node| !is_supported_formula_statement_theorem_bridge_node(node))
    {
        return None;
    }
    let theorem_items = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
    let [(_, theorem)] = theorem_items.as_slice() else {
        return None;
    };
    if subtree_has_recovery(ast, theorem) {
        return None;
    }
    let theorem_tokens = direct_token_texts(ast, theorem);
    if theorem_tokens.len() != 4
        || theorem_tokens[0] != "theorem"
        || theorem_tokens[1] != expected_label
        || theorem_tokens[2] != ":"
        || theorem_tokens[3] != ";"
    {
        return None;
    }

    let theorem_structural_children = structural_child_ids(ast, theorem);
    let [formula_expression_id] = theorem_structural_children.as_slice() else {
        return None;
    };
    let formula_expression = ast.node(*formula_expression_id)?;
    if !matches!(formula_expression.kind, SurfaceNodeKind::FormulaExpression) {
        return None;
    }
    let formula_children = structural_child_ids(ast, formula_expression);
    let [formula_id] = formula_children.as_slice() else {
        return None;
    };
    let formula = ast.node(*formula_id)?;
    let expected_spelling = match expected_constant {
        SurfaceFormulaConstant::Thesis => "thesis",
        SurfaceFormulaConstant::Contradiction => "contradiction",
    };
    let constant_matches = match expected_constant {
        SurfaceFormulaConstant::Thesis => matches!(
            formula.kind,
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis)
        ),
        SurfaceFormulaConstant::Contradiction => matches!(
            formula.kind,
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction)
        ),
    };
    if !constant_matches
        || direct_token_texts(ast, formula).as_slice() != [expected_spelling]
        || !structural_child_ids(ast, formula).is_empty()
    {
        return None;
    }

    Some(SourceFormulaStatement {
        formula_site: surface_site(*formula_id),
        formula_range: formula.range,
    })
}

pub(in crate::runner) fn exact_numeral_term_node_or_expression(
    ast: &SurfaceAst,
    id: SurfaceNodeId,
    expected_spelling: &str,
) -> Option<(SurfaceNodeId, SourceRange)> {
    let node = ast.node(id)?;
    match node.kind {
        SurfaceNodeKind::TermExpression => exact_numeral_term_operand(ast, id, expected_spelling),
        SurfaceNodeKind::NumeralTerm => exact_numeral_term_node(ast, id, expected_spelling),
        _ => None,
    }
}

pub(in crate::runner) fn exact_numeral_term_operand(
    ast: &SurfaceAst,
    term_expression_id: SurfaceNodeId,
    expected_spelling: &str,
) -> Option<(SurfaceNodeId, SourceRange)> {
    let term_expression = ast.node(term_expression_id)?;
    if !matches!(term_expression.kind, SurfaceNodeKind::TermExpression)
        || subtree_has_recovery(ast, term_expression)
    {
        return None;
    }
    let term_children = structural_child_ids(ast, term_expression);
    let [term_id] = term_children.as_slice() else {
        return None;
    };
    exact_numeral_term_node(ast, *term_id, expected_spelling)
}

#[derive(Debug, Clone, Copy)]
struct ExactImportedInfixFunctorTerm {
    term_id: SurfaceNodeId,
    term_range: SourceRange,
    left: (SurfaceNodeId, SourceRange),
    right: (SurfaceNodeId, SourceRange),
}

fn exact_imported_infix_functor_term(
    ast: &SurfaceAst,
    term_expression_id: SurfaceNodeId,
) -> Option<ExactImportedInfixFunctorTerm> {
    let term_expression = ast.node(term_expression_id)?;
    if !matches!(term_expression.kind, SurfaceNodeKind::TermExpression)
        || subtree_has_recovery(ast, term_expression)
    {
        return None;
    }
    let term_children = structural_child_ids(ast, term_expression);
    let [parenthesized_id] = term_children.as_slice() else {
        return None;
    };
    let parenthesized = ast.node(*parenthesized_id)?;
    if !matches!(parenthesized.kind, SurfaceNodeKind::ParenthesizedTerm)
        || direct_token_texts(ast, parenthesized).as_slice() != ["(", ")"]
    {
        return None;
    }
    let parenthesized_children = structural_child_ids(ast, parenthesized);
    let [inner_expression_id] = parenthesized_children.as_slice() else {
        return None;
    };
    let inner_expression = ast.node(*inner_expression_id)?;
    if !matches!(inner_expression.kind, SurfaceNodeKind::TermExpression) {
        return None;
    }
    let inner_children = structural_child_ids(ast, inner_expression);
    let [infix_id] = inner_children.as_slice() else {
        return None;
    };
    let infix = ast.node(*infix_id)?;
    if !matches!(
        &infix.kind,
        SurfaceNodeKind::InfixExpression(operator) if operator.spelling.as_ref() == "++"
    ) || direct_token_texts(ast, infix).as_slice() != ["++"]
    {
        return None;
    }
    let infix_children = structural_child_ids(ast, infix);
    let [left_expression_id, right_expression_id] = infix_children.as_slice() else {
        return None;
    };
    let left = exact_numeral_term_node_or_expression(ast, *left_expression_id, "1")?;
    let right = exact_numeral_term_node_or_expression(ast, *right_expression_id, "2")?;
    Some(ExactImportedInfixFunctorTerm {
        term_id: *infix_id,
        term_range: infix.range,
        left,
        right,
    })
}

fn exact_numeral_term_node(
    ast: &SurfaceAst,
    term_id: SurfaceNodeId,
    expected_spelling: &str,
) -> Option<(SurfaceNodeId, SourceRange)> {
    let term = ast.node(term_id)?;
    if matches!(term.kind, SurfaceNodeKind::NumeralTerm)
        && direct_token_texts(ast, term).as_slice() == [expected_spelling]
        && structural_child_ids(ast, term).is_empty()
    {
        Some((term_id, term.range))
    } else {
        None
    }
}

fn is_supported_formula_statement_theorem_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::FormulaConstant(_)
            | SurfaceNodeKind::Token(_)
    )
}

fn is_supported_builtin_binary_theorem_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::BuiltinPredicateApplication
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::Token(_)
    )
}

fn is_supported_builtin_type_assertion_theorem_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::IsAssertion
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::TypeExpression
            | SurfaceNodeKind::TypeHead
            | SurfaceNodeKind::Token(_)
    )
}

fn is_supported_imported_predicate_functor_theorem_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::ImportItem
            | SurfaceNodeKind::ImportAliasDecl
            | SurfaceNodeKind::ModulePath
            | SurfaceNodeKind::PathSegment
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::PredicateApplication
            | SurfaceNodeKind::PredicateSegment
            | SurfaceNodeKind::PredicateHead
            | SurfaceNodeKind::QualifiedSymbol
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::ParenthesizedTerm
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::InfixExpression(_)
            | SurfaceNodeKind::Token(_)
    )
}

fn is_supported_imported_attribute_assertion_theorem_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::ImportItem
            | SurfaceNodeKind::ImportAliasDecl
            | SurfaceNodeKind::ModulePath
            | SurfaceNodeKind::PathSegment
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::IsAssertion
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::AttributeTestChain
            | SurfaceNodeKind::AttributeRef
            | SurfaceNodeKind::QualifiedSymbol
            | SurfaceNodeKind::Token(_)
    )
}

fn is_supported_set_enumeration_theorem_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::BuiltinPredicateApplication
            | SurfaceNodeKind::TermExpression
            | SurfaceNodeKind::SetEnumeration
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::Token(_)
    )
}

fn is_supported_formula_connective_quantifier_theorem_bridge_node(node: &SurfaceNode) -> bool {
    matches!(
        node.kind,
        SurfaceNodeKind::Root
            | SurfaceNodeKind::CompilationUnit
            | SurfaceNodeKind::ItemList
            | SurfaceNodeKind::TheoremItem
            | SurfaceNodeKind::FormulaExpression
            | SurfaceNodeKind::BinaryFormula(_)
            | SurfaceNodeKind::QuantifiedFormula(_)
            | SurfaceNodeKind::QuantifierVariableSegment
            | SurfaceNodeKind::PrefixFormula(_)
            | SurfaceNodeKind::FormulaConstant(_)
            | SurfaceNodeKind::TypeExpression
            | SurfaceNodeKind::TypeHead
            | SurfaceNodeKind::Token(_)
    )
}
