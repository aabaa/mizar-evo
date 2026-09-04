use mizar_checker::{
    binding_env::{BindingContextId, BindingId, BindingRecoveryState},
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_context::{
        SourceBindingContextBuild, SourceBindingContextInput, SourceBindingContextOwner,
        SourceBindingContextProducer, SourceBindingSiteInput, SourceBindingSiteRole,
        SourceItemInput, SourceItemRecovery, SourceItemRole, SourceItemVisibility,
    },
    source_mode_definition::{
        SourceModeApplicationId, SourceModeApplicationInput, SourceModeDefinitionHandoffInput,
        SourceModeDefinitionId, SourceModeDefinitionInput, SourceModeDefinitionProducer,
        SourceModeDefinitionRecovery, SourceModeExpansionId, SourceModeExpansionInput,
        SourceModeInhabitationRequestId, SourceModeInhabitationRequestInput,
        SourceModeInhabitationRequestKind, SourceModeParameterId, SourceModeParameterInput,
        SourceModePropertyId, SourceModePropertyInput, SourceModePropertyKind,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationId, SourceTypeApplicationInput,
        SourceTypeExpressionId, SourceTypeExpressionInput, SourceTypeHandoffInput, SourceTypeHead,
        SourceTypeModeRhsExtensionInput, SourceTypeModeRhsId, SourceTypeModeRhsInput,
        SourceTypeModeRhsProducer, SourceTypeProducer,
    },
    type_checker::{
        CoercionObligationChecker, InitialObligationInput, InitialRequirementKind, ModeExpansion,
        TypeExpressionInput, TypeHeadInput, TypeNormalizer,
    },
    typed_ast::{
        CoercionTable, InitialObligationKind, InitialObligationStatus, InitialObligationTable,
        LocalTypeContextId, NodeRecoveryState, TypeDiagnosticTable, TypeFactTable, TypeTable,
        TypedArena, TypedArenaBuilder, TypedAst, TypedAstParts, TypedNode, TypedNodeId,
        TypedNodeLinks, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShellKind, DeclarationShellSet},
    env::{
        ContributionKind, DefinitionId, DefinitionIndex, DefinitionKind, DefinitionShell,
        ExportStatus, NamespacePath, SourceContributionId, SourceContributionIndex, SymbolEntry,
        SymbolEnv, SymbolEnvIndexes, SymbolIndex, SymbolKind, Visibility,
    },
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{ModuleId, SymbolId},
    symbols::{SignatureProjectionExtractor, SymbolOverloadPolicy},
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use mizar_syntax::{
    SurfaceAst, SurfaceFormulaConstant, SurfaceNode, SurfaceNodeId, SurfaceNodeKind,
    SurfaceTokenKind,
};

use super::checker_handoff::assemble_empty_resolved_typed_ast;
use super::source_ast::{
    leaf_token_texts, structural_child_ids, subtree_has_recovery, surface_nodes_with_kind,
    surface_site, surface_text,
};
use super::source_reserve::extract_builtin_source_type_expression;

pub(in crate::runner) const SOURCE_MODE_DEFINITION_TEXT: &str = concat!(
    "definition\n",
    "  let x be set;\n",
    "  let y be set;\n",
    "  mode Task262ModeDefinition: Task262Mode [x, y] is set;\n",
    "  sethood by computation(steps: 1);\n",
    "end;\n",
);

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.source_mode_definition.invalid_payload";
const STEP5C4_MODE_MISMATCH_KEY: &str = "modes.mode_semantics_mismatch";

#[derive(Debug)]
pub(in crate::runner) struct SourceModeDefinitionRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; variants are private corruption seams.
pub(in crate::runner) enum SourceModeDefinitionRouteMutation {
    None,
    RemoveResolverShell,
    WrongResolverProjection,
    WrongResolverEntry,
    WrongResolverDefinitionEntry,
    WrongResolverContribution,
    WrongContextModuleSite,
    WrongContextItemSite,
    WrongContextBindingSite(usize),
    WrongContextBindingOwner(usize),
    RemoveTypeExpression,
    WrongTypeApplicationBinding(usize),
    WrongTypeApplicationRoot(usize),
    WrongTypeExpressionSite(usize),
    WrongModeRhsOwner,
    WrongModeRhsRange,
    WrongModeRhsExpression,
    RemoveModeDefinition,
    RemoveModeParameter,
    RemoveModeApplication,
    RemoveModeExpansion,
    RemoveModeRequest,
    RemoveModeProperty,
    WrongModeParameterOwner,
    WrongModeParameterPatternRange,
    WrongModeApplicationParameters,
    WrongModeExpansionRhs,
    WrongModeRequestExpansion,
    WrongModeDefinitionProperty,
    WrongModePropertyJustification,
}

enum ExactSurfaceKind {
    Token(SurfaceTokenKind, &'static str),
    Structural(SurfaceNodeKind),
}

struct ExactSurfaceRow {
    kind: ExactSurfaceKind,
    start: usize,
    end: usize,
    children: &'static [usize],
}

macro_rules! token_row {
    ($kind:ident, $text:literal, $start:literal, $end:literal) => {
        ExactSurfaceRow {
            kind: ExactSurfaceKind::Token(SurfaceTokenKind::$kind, $text),
            start: $start,
            end: $end,
            children: &[],
        }
    };
}

macro_rules! structural_row {
    ($kind:ident, $start:literal, $end:literal, [$($child:literal),* $(,)?]) => {
        ExactSurfaceRow {
            kind: ExactSurfaceKind::Structural(SurfaceNodeKind::$kind),
            start: $start,
            end: $end,
            children: &[$($child),*],
        }
    };
}

const EXACT_SURFACE_PROFILE: &[ExactSurfaceRow] = &[
    token_row!(ReservedWord, "definition", 0, 10),
    token_row!(ReservedWord, "let", 13, 16),
    token_row!(Identifier, "x", 17, 18),
    token_row!(ReservedWord, "be", 19, 21),
    token_row!(ReservedWord, "set", 22, 25),
    token_row!(ReservedSymbol, ";", 25, 26),
    token_row!(ReservedWord, "let", 29, 32),
    token_row!(Identifier, "y", 33, 34),
    token_row!(ReservedWord, "be", 35, 37),
    token_row!(ReservedWord, "set", 38, 41),
    token_row!(ReservedSymbol, ";", 41, 42),
    token_row!(ReservedWord, "mode", 45, 49),
    token_row!(Identifier, "Task262ModeDefinition", 50, 71),
    token_row!(ReservedSymbol, ":", 71, 72),
    token_row!(Identifier, "Task262Mode", 73, 84),
    token_row!(ReservedSymbol, "[", 85, 86),
    token_row!(Identifier, "x", 86, 87),
    token_row!(ReservedSymbol, ",", 87, 88),
    token_row!(Identifier, "y", 89, 90),
    token_row!(ReservedSymbol, "]", 90, 91),
    token_row!(ReservedWord, "is", 92, 94),
    token_row!(ReservedWord, "set", 95, 98),
    token_row!(ReservedSymbol, ";", 98, 99),
    token_row!(ReservedWord, "sethood", 102, 109),
    token_row!(ReservedWord, "by", 110, 112),
    token_row!(ReservedWord, "computation", 113, 124),
    token_row!(ReservedSymbol, "(", 124, 125),
    token_row!(Identifier, "steps", 125, 130),
    token_row!(ReservedSymbol, ":", 130, 131),
    token_row!(Numeral, "1", 132, 133),
    token_row!(ReservedSymbol, ")", 133, 134),
    token_row!(ReservedSymbol, ";", 134, 135),
    token_row!(ReservedWord, "end", 136, 139),
    token_row!(ReservedSymbol, ";", 139, 140),
    structural_row!(TypeHead, 22, 25, [4]),
    structural_row!(TypeExpression, 22, 25, [34]),
    structural_row!(QualifiedVariableSegment, 17, 25, [2, 3, 35]),
    structural_row!(DefinitionParameter, 13, 26, [1, 36, 5]),
    structural_row!(TypeHead, 38, 41, [9]),
    structural_row!(TypeExpression, 38, 41, [38]),
    structural_row!(QualifiedVariableSegment, 33, 41, [7, 8, 39]),
    structural_row!(DefinitionParameter, 29, 42, [6, 40, 10]),
    structural_row!(ModePattern, 73, 91, [14, 15, 16, 17, 18, 19]),
    structural_row!(TypeHead, 95, 98, [21]),
    structural_row!(TypeExpression, 95, 98, [43]),
    structural_row!(ComputationOption, 125, 133, [27, 28, 29]),
    structural_row!(ComputationJustification, 113, 134, [25, 26, 45, 30]),
    structural_row!(JustificationClause, 110, 134, [24, 46]),
    structural_row!(ModeProperty, 102, 135, [23, 47, 31]),
    structural_row!(ModeDefinition, 45, 135, [11, 12, 13, 42, 20, 44, 22, 48]),
    structural_row!(DefinitionBlockItem, 0, 140, [0, 37, 41, 49, 32, 33]),
    structural_row!(ItemList, 0, 140, [50]),
    structural_row!(CompilationUnit, 0, 140, [51]),
    structural_row!(
        Root,
        0,
        140,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 52
        ]
    ),
];

#[derive(Debug, Clone, Copy)]
struct ExactModeDefinition {
    definition_block: SurfaceNodeId,
}

pub(in crate::runner) fn source_mode_definition_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_mode_definition_output_impl(
        ast,
        module.clone(),
        shells,
        symbols,
        source_text,
        SourceModeDefinitionRouteMutation::None,
    ) {
        None => None,
        Some(Ok(output)) if route_output_is_exact(&output) => Some(Vec::new()),
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
    .or_else(|| step5c4_mode_semantics_detail_keys(ast, module, shells, symbols))
}

pub(in crate::runner) fn step5c4_mode_semantics_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let modes = surface_nodes_with_kind(ast, SurfaceNodeKind::ModeDefinition);
    if modes.is_empty() {
        return None;
    }
    let dependent = modes.len() == 2
        && modes.iter().all(|(_, mode)| {
            structural_child_ids(ast, mode).into_iter().any(|child| {
                ast.node(child).is_some_and(|node| {
                    matches!(node.kind, SurfaceNodeKind::ModePattern)
                        && leaf_token_texts(ast, node).contains(&"of".to_owned())
                })
            })
        });
    let attributed = modes.len() == 1
        && surface_nodes_with_kind(ast, SurfaceNodeKind::StructureDefinition).len() == 1
        && surface_nodes_with_kind(ast, SurfaceNodeKind::AttributeDefinition).len() == 1
        && mode_rhs(ast, modes[0].1).is_some_and(|(_, rhs)| {
            structural_child_ids(ast, rhs).into_iter().any(|child| {
                ast.node(child)
                    .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::AttributeChain))
            })
        });
    if !dependent && !attributed {
        return None;
    }
    if modes.iter().any(|(id, mode)| {
        subtree_has_recovery(ast, mode)
            || !shells.declarations().iter().any(|shell| {
                shell.kind() == DeclarationShellKind::ModeDefinition
                    && shell.node_id() == *id
                    && shell.module() == &module
                    && shell.range() == mode.range
                    && !shell.recovered()
            })
            || mode_pattern(ast, mode)
                .and_then(|pattern| mode_symbol_for_node(ast, &module, symbols, *id, pattern))
                .is_none()
    }) {
        return Some(vec![STEP5C4_MODE_MISMATCH_KEY.to_owned()]);
    }
    if dependent {
        let wrong = dependent_mode_is_wrong_arity(ast, &module, symbols, &modes);
        return Some(if wrong {
            vec!["modes.dependent.argument_arity_mismatch".to_owned()]
        } else {
            vec![STEP5C4_MODE_MISMATCH_KEY.to_owned()]
        });
    }
    Some(
        if attributed_mode_is_valid(ast, &module, symbols, &modes[0]) {
            Vec::new()
        } else {
            vec![STEP5C4_MODE_MISMATCH_KEY.to_owned()]
        },
    )
}

fn mode_pattern<'a>(ast: &'a SurfaceAst, mode: &'a SurfaceNode) -> Option<&'a SurfaceNode> {
    structural_child_ids(ast, mode)
        .into_iter()
        .find_map(|child| {
            ast.node(child)
                .filter(|node| matches!(node.kind, SurfaceNodeKind::ModePattern))
        })
}

fn mode_rhs<'a>(
    ast: &'a SurfaceAst,
    mode: &'a SurfaceNode,
) -> Option<(SurfaceNodeId, &'a SurfaceNode)> {
    structural_child_ids(ast, mode)
        .into_iter()
        .find_map(|child| {
            ast.node(child)
                .filter(|node| matches!(node.kind, SurfaceNodeKind::TypeExpression))
                .map(|node| (child, node))
        })
}

fn mode_symbol_for_node(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    mode_id: SurfaceNodeId,
    pattern: &SurfaceNode,
) -> Option<SymbolId> {
    let mode = ast.node(mode_id)?;
    let spelling = leaf_token_texts(ast, pattern).join(" ");
    let mut candidates = symbols
        .symbols()
        .iter()
        .filter(|entry| {
            entry.kind() == SymbolKind::Mode
                && entry.primary_spelling() == spelling
                && entry.origin().source_id() == ast.source_id
                && entry.origin().module_id() == module
                && entry.origin().anchor() == &SourceAnchor::Range(mode.range)
                && !entry.origin().is_recovered()
        })
        .map(|entry| entry.symbol().clone())
        .collect::<Vec<_>>();
    (candidates.len() == 1).then(|| candidates.pop().unwrap_or_else(|| unreachable!()))
}

fn attributed_mode_is_valid(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    (mode_id, mode): &(SurfaceNodeId, &SurfaceNode),
) -> bool {
    let Some(pattern) = mode_pattern(ast, mode) else {
        return false;
    };
    let Some((rhs_id, rhs)) = mode_rhs(ast, mode) else {
        return false;
    };
    let Ok(source_type) = extract_builtin_source_type_expression(ast, rhs, module, symbols) else {
        return false;
    };
    let input = TypeExpressionInput::new(
        surface_site(rhs_id),
        source_type.range,
        source_type.spelling,
        source_type.head,
    )
    .with_attributes(source_type.attributes);
    TypeNormalizer::default()
        .normalize(symbols, [input])
        .diagnostics()
        .is_empty()
        && mode_symbol_for_node(ast, module, symbols, *mode_id, pattern).is_some()
}

fn dependent_mode_is_wrong_arity(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    modes: &[(SurfaceNodeId, &SurfaceNode)],
) -> bool {
    let Some((first_id, first_mode)) = modes.first() else {
        return false;
    };
    let Some(first_pattern) = mode_pattern(ast, first_mode) else {
        return false;
    };
    let Some(first_symbol) = mode_symbol_for_node(ast, module, symbols, *first_id, first_pattern)
    else {
        return false;
    };
    let Some((first_rhs_id, first_rhs)) = mode_rhs(ast, first_mode) else {
        return false;
    };
    let Ok(radix) = extract_builtin_source_type_expression(ast, first_rhs, module, symbols) else {
        return false;
    };
    let Some((second_id, second_mode)) = modes.get(1) else {
        return false;
    };
    let Some(second_pattern) = mode_pattern(ast, second_mode) else {
        return false;
    };
    let Some((rhs_id, rhs)) = mode_rhs(ast, second_mode) else {
        return false;
    };
    let children = structural_child_ids(ast, rhs);
    let [head_id] = children.as_slice() else {
        return false;
    };
    let Some(head) = ast.node(*head_id) else {
        return false;
    };
    let head_children = structural_child_ids(ast, head);
    let [head_symbol_id, args_id] = head_children.as_slice() else {
        return false;
    };
    let Some(args_node) = ast.node(*args_id) else {
        return false;
    };
    if !matches!(head.kind, SurfaceNodeKind::TypeHead)
        || !matches!(args_node.kind, SurfaceNodeKind::TypeArguments)
    {
        return false;
    }
    let Some(head_child) = ast.node(*head_symbol_id) else {
        return false;
    };
    let head_spelling = surface_text(ast, head_child);
    let first_name = leaf_token_texts(ast, first_pattern)
        .into_iter()
        .next()
        .unwrap_or_default();
    let Some(applied_symbol) = (head_spelling == first_name).then_some(first_symbol.clone()) else {
        return false;
    };
    let args = structural_child_ids(ast, args_node)
        .into_iter()
        .filter_map(|id| {
            ast.node(id)
                .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
                .map(|node| {
                    TypeExpressionInput::new(
                        surface_site(id),
                        node.range,
                        surface_text(ast, node),
                        TypeHeadInput::BuiltinSet,
                    )
                })
        })
        .collect::<Vec<_>>();
    let declared_arity = leaf_token_texts(ast, first_pattern)
        .iter()
        .position(|token| token == "of")
        .map(|of| {
            leaf_token_texts(ast, first_pattern)[of + 1..]
                .iter()
                .filter(|token| token.as_str() != ",")
                .count()
        });
    if declared_arity != Some(1) || args.len() != 2 || declared_arity == Some(args.len()) {
        return false;
    }
    let radix = TypeExpressionInput::new(
        surface_site(first_rhs_id),
        radix.range,
        radix.spelling,
        radix.head,
    )
    .with_attributes(radix.attributes);
    let input = TypeExpressionInput::new(
        surface_site(rhs_id),
        rhs.range,
        surface_text(ast, rhs),
        TypeHeadInput::Symbol(applied_symbol),
    )
    .with_args(args);
    let output = TypeNormalizer::new([(first_symbol, ModeExpansion::new(radix, Vec::new()))])
        .normalize(symbols, [input]);
    output
        .diagnostics()
        .canonical_iter()
        .filter(|(_, diagnostic)| diagnostic.message_key == "checker.type.wrong_mode_arity")
        .count()
        == 1
        && output
            .diagnostics()
            .canonical_iter()
            .all(|(_, diagnostic)| {
                matches!(
                    diagnostic.message_key.as_str(),
                    "checker.type.wrong_mode_arity" | "checker.type.recovery"
                )
            })
        && mode_symbol_for_node(ast, module, symbols, *second_id, second_pattern).is_some()
}

pub(in crate::runner) fn step5c4_mode_sethood_is_unprovable(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Result<(), String> {
    let modes = surface_nodes_with_kind(ast, SurfaceNodeKind::ModeDefinition);
    if modes.len() != 1 {
        return Err("expected one mode definition".to_owned());
    }
    let (mode_id, mode) = modes[0];
    if subtree_has_recovery(ast, mode)
        || !shells.declarations().iter().any(|shell| {
            shell.kind() == DeclarationShellKind::ModeDefinition
                && shell.node_id() == mode_id
                && shell.module() == module
                && shell.range() == mode.range
                && !shell.recovered()
        })
    {
        return Err("mode declaration identity is not authenticated".to_owned());
    }
    let pattern = mode_pattern(ast, mode).ok_or_else(|| "mode pattern is missing".to_owned())?;
    mode_symbol_for_node(ast, module, symbols, mode_id, pattern)
        .ok_or_else(|| "mode resolver identity is not authenticated".to_owned())?;
    let (rhs_id, rhs) = mode_rhs(ast, mode).ok_or_else(|| "mode radix is missing".to_owned())?;
    let source_type = extract_builtin_source_type_expression(ast, rhs, module, symbols)
        .map_err(|_| "mode radix is not a supported source type".to_owned())?;
    if !matches!(source_type.head, TypeHeadInput::BuiltinSet) || !source_type.attributes.is_empty()
    {
        return Err("mode radix is not a normalized bare set".to_owned());
    }
    let properties = surface_nodes_with_kind(ast, SurfaceNodeKind::ModeProperty);
    let proofs = surface_nodes_with_kind(ast, SurfaceNodeKind::ProofBlock);
    let conclusions = surface_nodes_with_kind(ast, SurfaceNodeKind::ConclusionStatement);
    let constants = surface_nodes_with_kind(
        ast,
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
    );
    if properties.len() != 1 || proofs.len() != 1 || conclusions.len() != 1 || constants.len() != 1
    {
        return Err("sethood proof shape is not exact".to_owned());
    }
    let property = properties[0].1;
    let target = TypeExpressionInput::new(
        surface_site(rhs_id),
        source_type.range,
        source_type.spelling,
        source_type.head,
    );
    let checking = CoercionObligationChecker::default().check(
        symbols,
        &TypeFactTable::new(),
        [],
        [InitialObligationInput::new(
            surface_site(properties[0].0),
            property.range,
            InitialRequirementKind::Sethood,
            target,
        )],
    );
    let obligations = checking.initial_obligations().iter().collect::<Vec<_>>();
    if !checking.diagnostics().is_empty()
        || obligations.len() != 1
        || obligations[0].1.kind != InitialObligationKind::Sethood
        || obligations[0].1.status != InitialObligationStatus::Pending
        || obligations[0].1.owner != surface_site(properties[0].0)
    {
        return Err("checker did not preserve the pending sethood obligation".to_owned());
    }
    if subtree_has_recovery(ast, property)
        || !structural_child_ids(ast, mode).contains(&properties[0].0)
        || !structural_child_ids(ast, property).contains(&proofs[0].0)
        || !leaf_token_texts(ast, property).contains(&"sethood".to_owned())
        || leaf_token_texts(ast, conclusions[0].1)
            .windows(2)
            .all(|tokens| tokens != ["thus", "thesis"])
        || leaf_token_texts(ast, constants[0].1).as_slice() != ["thesis"]
    {
        return Err("sethood proof is not the trivial thesis attempt".to_owned());
    }
    Ok(())
}

#[cfg(test)]
pub(in crate::runner) fn source_mode_definition_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceModeDefinitionRouteOutput, String>> {
    source_mode_definition_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceModeDefinitionRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_mode_definition_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceModeDefinitionRouteMutation,
) -> Option<Result<SourceModeDefinitionRouteOutput, String>> {
    source_mode_definition_output_impl(ast, module, shells, symbols, source_text, mutation)
}

fn source_mode_definition_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceModeDefinitionRouteMutation,
) -> Option<Result<SourceModeDefinitionRouteOutput, String>> {
    let exact = exact_mode_definition(ast, source_text)?;
    Some(build_output(ast, module, shells, symbols, exact, mutation))
}

fn exact_mode_definition(ast: &SurfaceAst, source_text: &str) -> Option<ExactModeDefinition> {
    if source_text != SOURCE_MODE_DEFINITION_TEXT
        || source_text.len() != 141
        || !source_text.ends_with('\n')
        || source_text.ends_with("\n\n")
        || ast.root().map(SurfaceNodeId::index) != Some(53)
        || ast.expression_root().is_some()
        || !surface_profile_is_exact(ast)
    {
        return None;
    }
    let root = exact_node(ast, 53, SurfaceNodeKind::Root, 0, 140)?;
    let definition_block = exact_node(ast, 50, SurfaceNodeKind::DefinitionBlockItem, 0, 140)?;
    let mode = exact_node(ast, 49, SurfaceNodeKind::ModeDefinition, 45, 135)?;
    let pattern = exact_node(ast, 42, SurfaceNodeKind::ModePattern, 73, 91)?;
    let rhs = exact_node(ast, 44, SurfaceNodeKind::TypeExpression, 95, 98)?;
    let property = exact_node(ast, 48, SurfaceNodeKind::ModeProperty, 102, 135)?;
    let justification = exact_node(ast, 46, SurfaceNodeKind::ComputationJustification, 113, 134)?;
    let direct_structural = ast
        .node(definition_block)?
        .children
        .iter()
        .copied()
        .filter(|child| {
            ast.node(*child)
                .is_some_and(|node| !matches!(node.kind, SurfaceNodeKind::Token(_)))
        })
        .map(SurfaceNodeId::index)
        .collect::<Vec<_>>();
    if direct_structural != [37, 41, 49]
        || !is_descendant(ast, root, definition_block)
        || !is_descendant(ast, mode, pattern)
        || !is_descendant(ast, mode, rhs)
        || !is_descendant(ast, mode, property)
        || !is_descendant(ast, property, justification)
        || is_descendant(ast, pattern, rhs)
        || is_descendant(ast, rhs, property)
    {
        return None;
    }
    Some(ExactModeDefinition { definition_block })
}

fn surface_profile_is_exact(ast: &SurfaceAst) -> bool {
    ast.nodes().len() == EXACT_SURFACE_PROFILE.len()
        && ast
            .nodes()
            .iter()
            .zip(EXACT_SURFACE_PROFILE)
            .all(|(actual, expected)| {
                actual.range == source_range(ast.source_id, expected.start, expected.end)
                    && !actual.recovered
                    && actual
                        .children
                        .iter()
                        .map(|child| child.index())
                        .eq(expected.children.iter().copied())
                    && match (&actual.kind, &expected.kind) {
                        (
                            SurfaceNodeKind::Token(actual),
                            ExactSurfaceKind::Token(expected_kind, expected_text),
                        ) => {
                            actual.kind == *expected_kind && actual.text.as_ref() == *expected_text
                        }
                        (actual, ExactSurfaceKind::Structural(expected)) => actual == expected,
                        _ => false,
                    }
            })
}

fn exact_node(
    ast: &SurfaceAst,
    index: usize,
    kind: SurfaceNodeKind,
    start: usize,
    end: usize,
) -> Option<SurfaceNodeId> {
    let view = ast.node_views().find(|view| view.id().index() == index)?;
    (*view.kind() == kind
        && view.range() == source_range(ast.source_id, start, end)
        && !view.is_recovered())
    .then_some(view.id())
}

fn is_descendant(ast: &SurfaceAst, ancestor: SurfaceNodeId, descendant: SurfaceNodeId) -> bool {
    if ancestor == descendant {
        return true;
    }
    ast.node(ancestor).is_some_and(|node| {
        node.children
            .iter()
            .any(|child| is_descendant(ast, *child, descendant))
    })
}

fn exact_resolver_profile(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutation: SourceModeDefinitionRouteMutation,
) -> Result<(SymbolId, DefinitionId, SourceContributionId), String> {
    let declarations = if mutation == SourceModeDefinitionRouteMutation::RemoveResolverShell
        && !shells.declarations().is_empty()
    {
        &shells.declarations()[..shells.declarations().len() - 1]
    } else {
        shells.declarations()
    };
    let [block, mode] = declarations else {
        return Err("Task262 resolver: expected two declaration shells".to_owned());
    };
    if !shells.exports().is_empty()
        || block.id().index() != 0
        || block.ordinal() != 0
        || block.kind() != DeclarationShellKind::DefinitionBlock
        || block.node_id().index() != 50
        || block.parent().is_some()
        || block.range() != source_range(ast.source_id, 0, 140)
        || block.module() != module
        || block.recovered()
        || mode.id().index() != 1
        || mode.ordinal() != 1
        || mode.kind() != DeclarationShellKind::ModeDefinition
        || mode.node_id().index() != 49
        || mode.parent() != Some(block.id())
        || mode.range() != source_range(ast.source_id, 45, 135)
        || mode.module() != module
        || mode.recovered()
        || symbols.module_id() != module
        || symbols.symbols().len() != 1
        || symbols.definitions().len() != 1
        || symbols.contributions().len() != 1
    {
        return Err("Task262 resolver: raw profile is not exact".to_owned());
    }

    let mut projections =
        SignatureProjectionExtractor::new(ast, shells, NamespacePath::new(module.path().as_str()))
            .extract();
    if mutation == SourceModeDefinitionRouteMutation::WrongResolverProjection {
        projections[0] = projections[0]
            .clone()
            .with_definition_kind(DefinitionKind::Predicate);
    }
    let [projection] = projections.as_slice() else {
        return Err("Task262 resolver: expected one parser-backed projection".to_owned());
    };
    let spelling = "Task262Mode [ x , y ]";
    if projection.primary_spelling() != spelling
        || projection.notation_spelling() != Some(spelling)
        || projection.symbol_kind() != SymbolKind::Mode
        || projection.definition_kind() != Some(DefinitionKind::Mode)
        || projection.arity().is_some()
        || projection.overload_policy() != SymbolOverloadPolicy::Overloadable
        || projection.signature().is_none()
    {
        return Err("Task262 resolver: parser-backed projection is not exact".to_owned());
    }

    let definition = symbols
        .definitions()
        .iter()
        .find(|definition| definition.id().index() == 0)
        .ok_or_else(|| "Task262 resolver: definition disappeared".to_owned())?;
    let symbol = symbols
        .symbols()
        .get(definition.symbol())
        .ok_or_else(|| "Task262 resolver: symbol disappeared".to_owned())?;
    if definition.kind() != DefinitionKind::Mode
        || definition.visibility() != Visibility::Public
        || !definition.parameters().is_empty()
        || !definition.binders().is_empty()
        || definition.arity().is_some()
        || definition.notation_shape() != Some(spelling)
        || definition.conflict().is_some()
        || definition.signature() != symbol.signature()
        || symbol.kind() != SymbolKind::Mode
        || symbol.visibility() != Visibility::Public
        || symbol.export_status() != ExportStatus::Exported
        || symbol.primary_spelling() != spelling
        || symbol.notation_spelling() != Some(spelling)
        || symbol.contribution() != definition.contribution()
    {
        return Err("Task262 resolver: mode definition is not exact".to_owned());
    }
    validate_origin(ast.source_id, module, definition.origin())?;
    validate_origin(ast.source_id, module, symbol.origin())?;
    let contribution = symbols
        .contributions()
        .get(definition.contribution())
        .ok_or_else(|| "Task262 resolver: local contribution disappeared".to_owned())?;
    if contribution.module() != module
        || !matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == ast.source_id
        )
        || contribution.effects().symbols() != [symbol.symbol().clone()]
        || contribution.effects().definitions() != [definition.id()]
    {
        return Err("Task262 resolver: local contribution is not exact".to_owned());
    }
    Ok((
        symbol.symbol().clone(),
        definition.id(),
        definition.contribution(),
    ))
}

fn resolver_env_with_mutation(
    symbols: &SymbolEnv,
    mutation: SourceModeDefinitionRouteMutation,
) -> SymbolEnv {
    let mut rebuilt_symbols = SymbolIndex::new();
    for entry in symbols.symbols().iter() {
        let primary_spelling = if mutation == SourceModeDefinitionRouteMutation::WrongResolverEntry
        {
            "Task262CorruptedMode [ x , y ]"
        } else {
            entry.primary_spelling()
        };
        let mut rebuilt = SymbolEntry::new(
            entry.symbol().clone(),
            entry.kind(),
            entry.namespace().clone(),
            primary_spelling,
            entry.origin().clone(),
            entry.contribution(),
        )
        .with_visibility(entry.visibility())
        .with_export_status(entry.export_status())
        .with_relations(entry.relations().to_vec());
        if let Some(notation) = entry.notation_spelling() {
            rebuilt = rebuilt.with_notation_spelling(notation);
        }
        if let Some(signature) = entry.signature() {
            rebuilt = rebuilt.with_signature(signature.clone());
        }
        rebuilt_symbols.insert(rebuilt);
    }

    let rebuilt_definitions =
        if mutation == SourceModeDefinitionRouteMutation::WrongResolverDefinitionEntry {
            let mut definitions = DefinitionIndex::new();
            for original in symbols.definitions().iter() {
                let mut rebuilt = DefinitionShell::new(
                    original.symbol().clone(),
                    DefinitionKind::Predicate,
                    original.origin().clone(),
                    original.contribution(),
                )
                .with_visibility(original.visibility())
                .with_parameters(original.parameters().to_vec())
                .with_binders(original.binders().to_vec())
                .with_dependencies(original.dependencies().to_vec());
                if let Some(notation_shape) = original.notation_shape() {
                    rebuilt = rebuilt.with_notation_shape(notation_shape);
                }
                if let Some(signature) = original.signature() {
                    rebuilt = rebuilt.with_signature(signature.clone());
                }
                let rebuilt_id = definitions.insert(rebuilt);
                debug_assert_eq!(rebuilt_id, original.id());
            }
            definitions
        } else {
            symbols.definitions().clone()
        };
    let rebuilt_contributions =
        if mutation == SourceModeDefinitionRouteMutation::WrongResolverContribution {
            let mut contributions = SourceContributionIndex::new();
            for original in symbols.contributions().iter() {
                let rebuilt_id = contributions.insert(
                    original.module().clone(),
                    original.kind().clone(),
                    original.anchor().clone(),
                );
                debug_assert_eq!(rebuilt_id, original.id());
            }
            contributions
        } else {
            symbols.contributions().clone()
        };

    SymbolEnv::new(
        symbols.module_id().clone(),
        SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: rebuilt_symbols,
            labels: symbols.labels().clone(),
            definitions: rebuilt_definitions,
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: rebuilt_contributions,
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

fn validate_origin(
    source_id: SourceId,
    module: &ModuleId,
    origin: &mizar_resolve::resolved_ast::SemanticOrigin,
) -> Result<(), String> {
    if origin.source_id() != source_id
        || origin.module_id() != module
        || origin.anchor() != &SourceAnchor::Range(source_range(source_id, 45, 135))
        || origin.structural_path() != [4, 0, 10, 0]
        || origin.import_edge().is_some()
        || origin.is_recovered()
    {
        return Err("Task262 resolver: mode origin is not exact".to_owned());
    }
    Ok(())
}

fn task262_arena(source_id: SourceId) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    for (index, row) in EXACT_SURFACE_PROFILE.iter().enumerate() {
        let generic_kind = match row.kind {
            ExactSurfaceKind::Token(_, _) => "source.surface.token",
            ExactSurfaceKind::Structural(_) => "source.surface.structural",
        };
        let (kind, start, end) = match index {
            34 => ("source.type.head", 22, 25),
            35 => ("source.type.expression", 22, 25),
            37 => ("source.definition.mode.parameter", 17, 18),
            38 => ("source.type.head", 38, 41),
            39 => ("source.type.expression", 38, 41),
            41 => ("source.definition.mode.parameter", 33, 34),
            42 => ("source.definition.mode.application", 73, 91),
            43 => ("source.type.head", 95, 98),
            44 => ("source.type.expression", 95, 98),
            46 => ("source.definition.mode.property.justification", 113, 134),
            48 => ("source.definition.mode.property", 102, 135),
            49 => ("source.definition.mode", 45, 135),
            50 => ("source.definition", 0, 140),
            53 => ("source.module", 0, 140),
            _ => (generic_kind, row.start, row.end),
        };
        let children = row
            .children
            .iter()
            .copied()
            .map(TypedNodeId::new)
            .collect::<Vec<_>>();
        let context = if index == 53 {
            LocalTypeContextId::new(0)
        } else {
            LocalTypeContextId::new(1)
        };
        let actual = builder
            .push(
                TypedNode::new(
                    kind,
                    SourceAnchor::Range(source_range(source_id, start, end)),
                )
                .with_children(children)
                .with_typing(TypingState::Unknown)
                .with_recovery(NodeRecoveryState::Normal)
                .with_links(TypedNodeLinks {
                    context: Some(context),
                    ..TypedNodeLinks::default()
                }),
            )
            .map_err(|error| error.to_string())?;
        if actual.index() != index {
            return Err("Task262 arena order changed".to_owned());
        }
    }
    builder
        .finish(Some(TypedNodeId::new(53)))
        .map_err(|error| error.to_string())
}

fn task248_context(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    exact: ExactModeDefinition,
    mutation: SourceModeDefinitionRouteMutation,
) -> Result<mizar_checker::source_context::SourceBindingContextHandoff, String> {
    let block = shells
        .declarations()
        .first()
        .ok_or_else(|| "Task248 source context: definition shell disappeared".to_owned())?;
    if block.kind() != DeclarationShellKind::DefinitionBlock
        || block.node_id() != exact.definition_block
        || block.id().index() != 0
    {
        return Err("Task248 source context: definition shell is not exact".to_owned());
    }
    let shell = block.id();
    let scope = LocalTermScope::new(vec![0]);
    let module_site = if mutation == SourceModeDefinitionRouteMutation::WrongContextModuleSite {
        TypedSiteRef::Node(TypedNodeId::new(50))
    } else {
        TypedSiteRef::Node(TypedNodeId::new(53))
    };
    let item_site = if mutation == SourceModeDefinitionRouteMutation::WrongContextItemSite {
        TypedSiteRef::Node(TypedNodeId::new(53))
    } else {
        TypedSiteRef::Node(TypedNodeId::new(50))
    };
    let input = SourceBindingContextInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        module_site,
        items: vec![SourceItemInput {
            shell,
            shell_ordinal: 0,
            role: SourceItemRole::DefinitionBlock,
            module_id: module,
            source_range: source_range(ast.source_id, 0, 140),
            parent: None,
            visibility: SourceItemVisibility::Unspecified,
            site: item_site,
            local_scope: Some(scope.clone()),
            recovery: SourceItemRecovery::Normal,
        }],
        bindings: vec![
            SourceBindingSiteInput {
                shell,
                source_ordinal: 0,
                spelling: "x".to_owned(),
                declaration_range: source_range(ast.source_id, 17, 18),
                written_type_range: source_range(ast.source_id, 22, 25),
                site: if mutation == SourceModeDefinitionRouteMutation::WrongContextBindingSite(0) {
                    TypedSiteRef::Node(TypedNodeId::new(41))
                } else {
                    TypedSiteRef::Node(TypedNodeId::new(37))
                },
                context_owner: if mutation
                    == SourceModeDefinitionRouteMutation::WrongContextBindingOwner(0)
                {
                    SourceBindingContextOwner::Module
                } else {
                    SourceBindingContextOwner::Shell(shell)
                },
                role: SourceBindingSiteRole::DefinitionParameter {
                    local: LocalTermBinding::new(
                        "x",
                        scope.clone(),
                        source_range(ast.source_id, 17, 18),
                        0,
                    ),
                },
                recovery: BindingRecoveryState::Normal,
            },
            SourceBindingSiteInput {
                shell,
                source_ordinal: 1,
                spelling: "y".to_owned(),
                declaration_range: source_range(ast.source_id, 33, 34),
                written_type_range: source_range(ast.source_id, 38, 41),
                site: if mutation == SourceModeDefinitionRouteMutation::WrongContextBindingSite(1) {
                    TypedSiteRef::Node(TypedNodeId::new(37))
                } else {
                    TypedSiteRef::Node(TypedNodeId::new(41))
                },
                context_owner: if mutation
                    == SourceModeDefinitionRouteMutation::WrongContextBindingOwner(1)
                {
                    SourceBindingContextOwner::Module
                } else {
                    SourceBindingContextOwner::Shell(shell)
                },
                role: SourceBindingSiteRole::DefinitionParameter {
                    local: LocalTermBinding::new(
                        "y",
                        scope,
                        source_range(ast.source_id, 33, 34),
                        1,
                    ),
                },
                recovery: BindingRecoveryState::Normal,
            },
        ],
    };
    match SourceBindingContextProducer::build(input)
        .map_err(|error| format!("Task248 source context: {error}"))?
    {
        SourceBindingContextBuild::Complete(projection) => Ok(projection.into_handoff()),
        SourceBindingContextBuild::Incomplete(_) => {
            Err("Task248 source context: unexpectedly incomplete".to_owned())
        }
        _ => Err("Task248 source context: unsupported build state".to_owned()),
    }
}

fn task249_input(source_id: SourceId, module_id: ModuleId) -> SourceTypeHandoffInput {
    SourceTypeHandoffInput {
        source_id,
        module_id: module_id.clone(),
        applications: vec![
            SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            },
            SourceTypeApplicationInput {
                binding: BindingId::new(1),
                source_ordinal: 1,
                root: SourceTypeExpressionId::new(1),
            },
        ],
        expressions: vec![
            bare_set_type(source_id, module_id.clone(), 35, 34, 22, 25),
            bare_set_type(source_id, module_id, 39, 38, 38, 41),
        ],
        arguments: Vec::new(),
    }
}

fn task249m_input(source_id: SourceId, module_id: ModuleId) -> SourceTypeModeRhsExtensionInput {
    SourceTypeModeRhsExtensionInput {
        source_id,
        module_id: module_id.clone(),
        rhs: vec![SourceTypeModeRhsInput {
            definition_site: TypedSiteRef::Node(TypedNodeId::new(49)),
            definition_range: source_range(source_id, 45, 135),
            source_ordinal: 0,
            expression: bare_set_type(source_id, module_id, 44, 43, 95, 98),
        }],
    }
}

fn bare_set_type(
    source_id: SourceId,
    module_id: ModuleId,
    site: usize,
    head_site: usize,
    start: usize,
    end: usize,
) -> SourceTypeExpressionInput {
    SourceTypeExpressionInput {
        source_id,
        module_id,
        site: TypedSiteRef::Node(TypedNodeId::new(site)),
        source_range: source_range(source_id, start, end),
        spelling: "set".to_owned(),
        head_site: TypedSiteRef::Node(TypedNodeId::new(head_site)),
        head_range: source_range(source_id, start, end),
        head_spelling: "set".to_owned(),
        form: SourceTypeApplicationForm::Bare,
        head: SourceTypeHead::BuiltinSet,
        recovery: NodeRecoveryState::Normal,
    }
}

fn task262_input(
    source_id: SourceId,
    module_id: ModuleId,
    resolver: (SymbolId, DefinitionId, SourceContributionId),
) -> SourceModeDefinitionHandoffInput {
    SourceModeDefinitionHandoffInput {
        source_id,
        module_id,
        definitions: vec![SourceModeDefinitionInput {
            symbol: resolver.0,
            definition: resolver.1,
            contribution: resolver.2,
            site: TypedSiteRef::Node(TypedNodeId::new(49)),
            source_range: source_range(source_id, 45, 135),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceModeDefinitionRecovery::Normal,
            spelling: concat!(
                "mode Task262ModeDefinition: Task262Mode [x, y] is set;\n",
                "  sethood by computation(steps: 1);"
            )
            .to_owned(),
            application: SourceModeApplicationId::new(0),
            expansion: SourceModeExpansionId::new(0),
            inhabitation_request: SourceModeInhabitationRequestId::new(0),
            property: Some(SourceModePropertyId::new(0)),
        }],
        parameters: vec![
            SourceModeParameterInput {
                owner: SourceModeDefinitionId::new(0),
                ordinal: 0,
                binding: BindingId::new(0),
                written_type: SourceTypeApplicationId::new(0),
                site: TypedSiteRef::Node(TypedNodeId::new(37)),
                source_range: source_range(source_id, 13, 26),
                declaration_range: source_range(source_id, 17, 18),
                pattern_range: source_range(source_id, 86, 87),
                context: BindingContextId::new(1),
                recovery: SourceModeDefinitionRecovery::Normal,
                spelling: "let x be set;".to_owned(),
            },
            SourceModeParameterInput {
                owner: SourceModeDefinitionId::new(0),
                ordinal: 1,
                binding: BindingId::new(1),
                written_type: SourceTypeApplicationId::new(1),
                site: TypedSiteRef::Node(TypedNodeId::new(41)),
                source_range: source_range(source_id, 29, 42),
                declaration_range: source_range(source_id, 33, 34),
                pattern_range: source_range(source_id, 89, 90),
                context: BindingContextId::new(1),
                recovery: SourceModeDefinitionRecovery::Normal,
                spelling: "let y be set;".to_owned(),
            },
        ],
        applications: vec![SourceModeApplicationInput {
            owner: SourceModeDefinitionId::new(0),
            ordinal: 0,
            parameters: vec![SourceModeParameterId::new(0), SourceModeParameterId::new(1)],
            site: TypedSiteRef::Node(TypedNodeId::new(42)),
            source_range: source_range(source_id, 73, 91),
            context: BindingContextId::new(1),
            recovery: SourceModeDefinitionRecovery::Normal,
            spelling: "Task262Mode [ x , y ]".to_owned(),
        }],
        expansions: vec![SourceModeExpansionInput {
            owner: SourceModeDefinitionId::new(0),
            ordinal: 0,
            rhs: SourceTypeModeRhsId::new(0),
            site: TypedSiteRef::Node(TypedNodeId::new(44)),
            source_range: source_range(source_id, 95, 98),
            context: BindingContextId::new(1),
            recovery: SourceModeDefinitionRecovery::Normal,
            spelling: "set".to_owned(),
        }],
        inhabitation_requests: vec![SourceModeInhabitationRequestInput {
            owner: SourceModeDefinitionId::new(0),
            ordinal: 0,
            expansion: SourceModeExpansionId::new(0),
            kind: SourceModeInhabitationRequestKind::Rhs,
            site: TypedSiteRef::Node(TypedNodeId::new(44)),
            source_range: source_range(source_id, 95, 98),
            context: BindingContextId::new(1),
            recovery: SourceModeDefinitionRecovery::Normal,
            spelling: "set".to_owned(),
        }],
        properties: vec![SourceModePropertyInput {
            owner: SourceModeDefinitionId::new(0),
            ordinal: 0,
            kind: SourceModePropertyKind::Sethood,
            site: TypedSiteRef::Node(TypedNodeId::new(48)),
            source_range: source_range(source_id, 102, 135),
            justification: SourceAnchor::Range(source_range(source_id, 113, 134)),
            recovery: SourceModeDefinitionRecovery::Normal,
            spelling: "sethood by computation(steps: 1);".to_owned(),
        }],
    }
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    exact: ExactModeDefinition,
    mutation: SourceModeDefinitionRouteMutation,
) -> Result<SourceModeDefinitionRouteOutput, String> {
    let corrupted_symbols = matches!(
        mutation,
        SourceModeDefinitionRouteMutation::WrongResolverEntry
            | SourceModeDefinitionRouteMutation::WrongResolverDefinitionEntry
            | SourceModeDefinitionRouteMutation::WrongResolverContribution
    )
    .then(|| resolver_env_with_mutation(symbols, mutation));
    let symbols = corrupted_symbols.as_ref().unwrap_or(symbols);
    let resolver = exact_resolver_profile(ast, &module, shells, symbols, mutation)?;
    let arena = task262_arena(ast.source_id)?;
    let source_context = task248_context(ast, module.clone(), shells, exact, mutation)?;

    let mut type_input = task249_input(ast.source_id, module.clone());
    if mutation == SourceModeDefinitionRouteMutation::RemoveTypeExpression {
        type_input.expressions.pop();
    }
    if let SourceModeDefinitionRouteMutation::WrongTypeApplicationBinding(index) = mutation {
        type_input.applications[index].binding = BindingId::new(1 - index);
    }
    if let SourceModeDefinitionRouteMutation::WrongTypeApplicationRoot(index) = mutation {
        type_input.applications[index].root = SourceTypeExpressionId::new(1 - index);
    }
    if let SourceModeDefinitionRouteMutation::WrongTypeExpressionSite(index) = mutation {
        type_input.expressions[index].site =
            TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 39 } else { 35 }));
    }
    let base_type =
        SourceTypeProducer::build(type_input, source_context.binding_env(), symbols, &arena)
            .map_err(|error| format!("Task249 source type: {error}"))?;
    let mut rhs_input = task249m_input(ast.source_id, module.clone());
    if mutation == SourceModeDefinitionRouteMutation::WrongModeRhsOwner {
        rhs_input.rhs[0].definition_site = TypedSiteRef::Node(TypedNodeId::new(48));
    }
    if mutation == SourceModeDefinitionRouteMutation::WrongModeRhsRange {
        rhs_input.rhs[0].definition_range = source_range(ast.source_id, 45, 134);
    }
    if mutation == SourceModeDefinitionRouteMutation::WrongModeRhsExpression {
        rhs_input.rhs[0].expression.site = TypedSiteRef::Node(TypedNodeId::new(43));
    }
    let source_type = SourceTypeModeRhsProducer::extend(&base_type, rhs_input, &arena)
        .map_err(|error| format!("Task249M mode RHS: {error}"))?;

    let contexts = source_context.local_contexts().clone();
    let mut typed_ast = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module.clone(),
        resolved_root: None,
        source_context: Some(source_context),
        source_type: Some(source_type),
        source_attribute: None,
        nodes: arena,
        contexts,
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| format!("Task249M typed installation: {error}"))?;

    let mut mode_input = task262_input(ast.source_id, module, resolver);
    match mutation {
        SourceModeDefinitionRouteMutation::RemoveModeDefinition => mode_input.definitions.clear(),
        SourceModeDefinitionRouteMutation::RemoveModeParameter => {
            mode_input.parameters.pop();
        }
        SourceModeDefinitionRouteMutation::RemoveModeApplication => {
            mode_input.applications.clear();
        }
        SourceModeDefinitionRouteMutation::RemoveModeExpansion => mode_input.expansions.clear(),
        SourceModeDefinitionRouteMutation::RemoveModeRequest => {
            mode_input.inhabitation_requests.clear();
        }
        SourceModeDefinitionRouteMutation::RemoveModeProperty => mode_input.properties.clear(),
        SourceModeDefinitionRouteMutation::WrongModeParameterOwner => {
            mode_input.parameters[0].owner = SourceModeDefinitionId::new(1);
        }
        SourceModeDefinitionRouteMutation::WrongModeParameterPatternRange => {
            mode_input.parameters[0].pattern_range = source_range(ast.source_id, 89, 90);
        }
        SourceModeDefinitionRouteMutation::WrongModeApplicationParameters => {
            mode_input.applications[0].parameters.swap(0, 1);
        }
        SourceModeDefinitionRouteMutation::WrongModeExpansionRhs => {
            mode_input.expansions[0].rhs = SourceTypeModeRhsId::new(1);
        }
        SourceModeDefinitionRouteMutation::WrongModeRequestExpansion => {
            mode_input.inhabitation_requests[0].expansion = SourceModeExpansionId::new(1);
        }
        SourceModeDefinitionRouteMutation::WrongModeDefinitionProperty => {
            mode_input.definitions[0].property = None;
        }
        SourceModeDefinitionRouteMutation::WrongModePropertyJustification => {
            mode_input.properties[0].justification =
                SourceAnchor::Range(source_range(ast.source_id, 113, 133));
        }
        _ => {}
    }
    let projection = SourceModeDefinitionProducer::build(
        mode_input,
        symbols,
        typed_ast
            .source_context()
            .ok_or_else(|| "Task248 source context disappeared".to_owned())?,
        typed_ast
            .source_type()
            .ok_or_else(|| "Task249M source type disappeared".to_owned())?,
        typed_ast.initial_obligations(),
        typed_ast.nodes(),
    )
    .map_err(|error| format!("Task262 mode definition: {error}"))?;
    typed_ast = typed_ast
        .with_source_mode_definition(projection)
        .map_err(|error| format!("Task262 typed installation: {error}"))?;

    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.definition.mode"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)
        .map_err(|error| format!("Task262 final assembly: {error}"))?;
    Ok(SourceModeDefinitionRouteOutput {
        typed_ast,
        resolved,
    })
}

fn route_output_is_exact(output: &SourceModeDefinitionRouteOutput) -> bool {
    let typed = &output.typed_ast;
    let resolved = &output.resolved;
    let Some(context) = typed.source_context() else {
        return false;
    };
    let Some(source_type) = typed.source_type() else {
        return false;
    };
    let Some(mode) = typed.source_mode_definition() else {
        return false;
    };
    context.items().len() == 1
        && context.declarations().len() == 2
        && context.binding_env().bindings().len() == 2
        && context.binding_env().contexts().len() == 2
        && context.local_contexts().len() == 2
        && context.context_links().len() == 2
        && context.binding_env().diagnostics().is_empty()
        && source_type.applications().len() == 2
        && source_type.expressions().len() == 3
        && source_type.arguments().is_empty()
        && source_type.definition_returns().is_empty()
        && source_type.mode_rhs().len() == 1
        && mode.definitions().len() == 1
        && mode.parameters().len() == 2
        && mode.applications().len() == 1
        && mode.expansions().len() == 1
        && mode.inhabitation_requests().len() == 1
        && mode.properties().len() == 1
        && typed.source_mode_definition() == resolved.source_mode_definition()
        && typed.source_context() == resolved.source_context()
        && typed.source_type() == resolved.source_type()
        && typed.initial_obligations().len() == 1
        && typed.types().is_empty()
        && typed.facts().is_empty()
        && typed.coercions().is_empty()
        && typed.diagnostics().is_empty()
        && typed.source_term().is_none()
        && typed.source_attribute().is_none()
        && typed.source_evidence().is_none()
        && typed.source_predicate_definition().is_none()
        && typed.source_functor_definition().is_none()
        && typed.source_attribute_definition().is_none()
        && resolved.source_predicate_definition().is_none()
        && resolved.source_functor_definition().is_none()
        && resolved.source_attribute_definition().is_none()
        && resolved.expr_metadata().is_empty()
        && resolved.cluster_facts().is_empty()
        && resolved.diagnostics().is_empty()
}

const fn source_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
