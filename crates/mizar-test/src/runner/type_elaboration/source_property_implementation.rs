use mizar_checker::{
    binding_env::{BindingContextId, BindingId, BindingRecoveryState},
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_atomic_formula::{
        SourceAtomicEdgeId, SourceAtomicEdgeInput, SourceAtomicEdgeRole,
        SourceAtomicFormulaHandoffInput, SourceAtomicFormulaId, SourceAtomicFormulaInput,
        SourceAtomicFormulaKind, SourceAtomicFormulaProducer, SourceAtomicFormulaRecovery,
        SourceAtomicRequestInput, SourceAtomicRequestKind, SourceAtomicTermTarget,
    },
    source_context::{
        SourceBindingContextBuild, SourceBindingContextInput, SourceBindingContextOwner,
        SourceBindingContextProducer, SourceBindingSiteInput, SourceBindingSiteRole,
        SourceItemInput, SourceItemRecovery, SourceItemRole, SourceItemVisibility,
    },
    source_property_implementation::{
        SourcePropertyCorrectnessInput, SourcePropertyCorrectnessKind, SourcePropertyDefiniensId,
        SourcePropertyDefiniensInput, SourcePropertyDefiniensTarget,
        SourcePropertyImplementationHandoffInput, SourcePropertyImplementationId,
        SourcePropertyImplementationInput, SourcePropertyImplementationProducer,
        SourcePropertyImplementationRecovery, SourcePropertyImplementationStyle,
        SourcePropertyParameterId, SourcePropertyParameterInput, SourcePropertyTargetId,
        SourcePropertyTargetInput,
    },
    source_structure::{
        SourceStructureEdgeInput, SourceStructureEdgeRole, SourceStructureHandoffInput,
        SourceStructureMemberId, SourceStructureMemberInput, SourceStructureMemberRole,
        SourceStructureProducer, SourceStructureRecovery, SourceStructureRequestInput,
        SourceStructureRequestKind, SourceStructureTarget, SourceStructureTermId,
        SourceStructureTermInput, SourceStructureTermKind,
    },
    source_term::{
        SourcePrimaryTermHandoffInput, SourcePrimaryTermId, SourcePrimaryTermInput,
        SourcePrimaryTermKind, SourcePrimaryTermProducer, SourcePrimaryTermRecovery,
        SourcePrimaryTermReferenceInput, SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationId, SourceTypeApplicationInput,
        SourceTypeExpressionId, SourceTypeExpressionInput, SourceTypeHandoffInput, SourceTypeHead,
        SourceTypeProducer, SourceTypeStructureMemberHandoffInput, SourceTypeStructureMemberId,
        SourceTypeStructureMemberInput, SourceTypeStructureMemberProducer,
    },
    type_checker::{TypeExpressionInput, TypeHeadInput, TypeNormalizer},
    typed_ast::{
        CoercionTable, InitialObligationKind, InitialObligationTable, LocalTypeContextId,
        NodeRecoveryState, TypeDiagnosticTable, TypeFactTable, TypeRole, TypeTable, TypedArena,
        TypedArenaBuilder, TypedAst, TypedAstParts, TypedNode, TypedNodeId, TypedNodeLinks,
        TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShellKind, DeclarationShellSet},
    env::{
        ContributionKind, DefinitionId, DefinitionKind, ExportStatus, NamespacePath,
        SourceContributionId, SymbolEnv, SymbolKind, Visibility,
    },
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{ModuleId, SymbolId},
    symbols::{SignatureProjectionExtractor, SymbolOverloadPolicy},
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNodeId, SurfaceNodeKind, SurfaceTokenKind};

use super::checker_handoff::assemble_empty_resolved_typed_ast;
use super::source_ast::{
    leaf_token_texts, subtree_has_recovery, surface_nodes_with_kind, surface_site,
};
use super::source_reserve::extract_builtin_source_type_expression;

pub(in crate::runner) const SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT: &str = concat!(
    "definition\n",
    "  struct Task264Carrier where\n",
    "    field carrier -> set;\n",
    "    property marker -> set;\n",
    "  end;\n",
    "end;\n",
    "\n",
    "definition\n",
    "  let M be Task264Carrier;\n",
    "  property M.marker means it = it;\n",
    "  existence by computation(steps: 1);\n",
    "  uniqueness by computation(steps: 1);\n",
    "end;\n",
);

pub(in crate::runner) const SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT: &str = concat!(
    "definition\n",
    "  struct Task264Carrier where\n",
    "    field carrier -> set;\n",
    "    property marker -> set;\n",
    "  end;\n",
    "end;\n",
    "\n",
    "definition\n",
    "  let M be Task264Carrier;\n",
    "  property M.marker equals M.carrier;\n",
    "end;\n",
);

const INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.source_property_implementation.invalid_payload";

#[derive(Debug)]
pub(in crate::runner) struct SourcePropertyImplementationRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; variants are private corruption seams.
pub(in crate::runner) enum SourcePropertyImplementationRouteMutation {
    None,
    WrongSurfaceRange,
    WrongSurfaceRecovery,
    WrongSurfaceChildren,
    WrongSurfaceRoot,
    WrongShell,
    WrongResolverTarget,
    WrongCarrierProvenance,
    WrongContext,
    WrongType,
    WrongTerm,
    WrongStructure,
    WrongFormula,
    WrongImplementation,
    WrongArena,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Profile {
    Means,
    Equals,
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
        ExactSurfaceRow { kind: ExactSurfaceKind::Structural(SurfaceNodeKind::$kind), start: $start, end: $end, children: &[$($child),*] }
    };
}

const MEANS_SURFACE: &[ExactSurfaceRow] = &[
    token_row!(ReservedWord, "definition", 0, 10),
    token_row!(ReservedWord, "struct", 13, 19),
    token_row!(Identifier, "Task264Carrier", 20, 34),
    token_row!(ReservedWord, "where", 35, 40),
    token_row!(ReservedWord, "field", 45, 50),
    token_row!(Identifier, "carrier", 51, 58),
    token_row!(ReservedSymbol, "->", 59, 61),
    token_row!(ReservedWord, "set", 62, 65),
    token_row!(ReservedSymbol, ";", 65, 66),
    token_row!(ReservedWord, "property", 71, 79),
    token_row!(Identifier, "marker", 80, 86),
    token_row!(ReservedSymbol, "->", 87, 89),
    token_row!(ReservedWord, "set", 90, 93),
    token_row!(ReservedSymbol, ";", 93, 94),
    token_row!(ReservedWord, "end", 97, 100),
    token_row!(ReservedSymbol, ";", 100, 101),
    token_row!(ReservedWord, "end", 102, 105),
    token_row!(ReservedSymbol, ";", 105, 106),
    token_row!(ReservedWord, "definition", 108, 118),
    token_row!(ReservedWord, "let", 121, 124),
    token_row!(Identifier, "M", 125, 126),
    token_row!(ReservedWord, "be", 127, 129),
    token_row!(UserSymbol, "Task264Carrier", 130, 144),
    token_row!(ReservedSymbol, ";", 144, 145),
    token_row!(ReservedWord, "property", 148, 156),
    token_row!(Identifier, "M", 157, 158),
    token_row!(ReservedSymbol, ".", 158, 159),
    token_row!(Identifier, "marker", 159, 165),
    token_row!(ReservedWord, "means", 166, 171),
    token_row!(ReservedWord, "it", 172, 174),
    token_row!(ReservedSymbol, "=", 175, 176),
    token_row!(ReservedWord, "it", 177, 179),
    token_row!(ReservedSymbol, ";", 179, 180),
    token_row!(ReservedWord, "existence", 183, 192),
    token_row!(ReservedWord, "by", 193, 195),
    token_row!(ReservedWord, "computation", 196, 207),
    token_row!(ReservedSymbol, "(", 207, 208),
    token_row!(Identifier, "steps", 208, 213),
    token_row!(ReservedSymbol, ":", 213, 214),
    token_row!(Numeral, "1", 215, 216),
    token_row!(ReservedSymbol, ")", 216, 217),
    token_row!(ReservedSymbol, ";", 217, 218),
    token_row!(ReservedWord, "uniqueness", 221, 231),
    token_row!(ReservedWord, "by", 232, 234),
    token_row!(ReservedWord, "computation", 235, 246),
    token_row!(ReservedSymbol, "(", 246, 247),
    token_row!(Identifier, "steps", 247, 252),
    token_row!(ReservedSymbol, ":", 252, 253),
    token_row!(Numeral, "1", 254, 255),
    token_row!(ReservedSymbol, ")", 255, 256),
    token_row!(ReservedSymbol, ";", 256, 257),
    token_row!(ReservedWord, "end", 258, 261),
    token_row!(ReservedSymbol, ";", 261, 262),
    structural_row!(StructurePattern, 20, 34, [2]),
    structural_row!(TypeHead, 62, 65, [7]),
    structural_row!(TypeExpression, 62, 65, [54]),
    structural_row!(StructureField, 45, 66, [4, 5, 6, 55, 8]),
    structural_row!(TypeHead, 90, 93, [12]),
    structural_row!(TypeExpression, 90, 93, [57]),
    structural_row!(StructureProperty, 71, 94, [9, 10, 11, 58, 13]),
    structural_row!(StructureDefinition, 13, 101, [1, 53, 3, 56, 59, 14, 15]),
    structural_row!(DefinitionBlockItem, 0, 106, [0, 60, 16, 17]),
    structural_row!(PathSegment, 130, 144, [22]),
    structural_row!(QualifiedSymbol, 130, 144, [62]),
    structural_row!(TypeHead, 130, 144, [63]),
    structural_row!(DefinitionParameter, 121, 145, [19, 20, 21, 64, 23]),
    structural_row!(ItTerm, 172, 174, [29]),
    structural_row!(TermExpression, 172, 174, [66]),
    structural_row!(ItTerm, 177, 179, [31]),
    structural_row!(TermExpression, 177, 179, [68]),
    structural_row!(BuiltinPredicateApplication, 172, 179, [67, 30, 69]),
    structural_row!(FormulaExpression, 172, 179, [70]),
    structural_row!(FormulaDefiniens, 172, 179, [71]),
    structural_row!(ComputationOption, 208, 216, [37, 38, 39]),
    structural_row!(ComputationJustification, 196, 217, [35, 36, 73, 40]),
    structural_row!(JustificationClause, 193, 217, [34, 74]),
    structural_row!(CorrectnessCondition, 183, 218, [33, 75, 41]),
    structural_row!(ComputationOption, 247, 255, [46, 47, 48]),
    structural_row!(ComputationJustification, 235, 256, [44, 45, 77, 49]),
    structural_row!(JustificationClause, 232, 256, [43, 78]),
    structural_row!(CorrectnessCondition, 221, 257, [42, 79, 50]),
    structural_row!(
        PropertyImplementation,
        108,
        262,
        [18, 65, 24, 25, 26, 27, 28, 72, 32, 76, 80, 51, 52]
    ),
    structural_row!(ItemList, 0, 262, [61, 81]),
    structural_row!(CompilationUnit, 0, 262, [82]),
    structural_row!(
        Root,
        0,
        262,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 83
        ]
    ),
];

const EQUALS_SURFACE: &[ExactSurfaceRow] = &[
    token_row!(ReservedWord, "definition", 0, 10),
    token_row!(ReservedWord, "struct", 13, 19),
    token_row!(Identifier, "Task264Carrier", 20, 34),
    token_row!(ReservedWord, "where", 35, 40),
    token_row!(ReservedWord, "field", 45, 50),
    token_row!(Identifier, "carrier", 51, 58),
    token_row!(ReservedSymbol, "->", 59, 61),
    token_row!(ReservedWord, "set", 62, 65),
    token_row!(ReservedSymbol, ";", 65, 66),
    token_row!(ReservedWord, "property", 71, 79),
    token_row!(Identifier, "marker", 80, 86),
    token_row!(ReservedSymbol, "->", 87, 89),
    token_row!(ReservedWord, "set", 90, 93),
    token_row!(ReservedSymbol, ";", 93, 94),
    token_row!(ReservedWord, "end", 97, 100),
    token_row!(ReservedSymbol, ";", 100, 101),
    token_row!(ReservedWord, "end", 102, 105),
    token_row!(ReservedSymbol, ";", 105, 106),
    token_row!(ReservedWord, "definition", 108, 118),
    token_row!(ReservedWord, "let", 121, 124),
    token_row!(Identifier, "M", 125, 126),
    token_row!(ReservedWord, "be", 127, 129),
    token_row!(UserSymbol, "Task264Carrier", 130, 144),
    token_row!(ReservedSymbol, ";", 144, 145),
    token_row!(ReservedWord, "property", 148, 156),
    token_row!(Identifier, "M", 157, 158),
    token_row!(ReservedSymbol, ".", 158, 159),
    token_row!(Identifier, "marker", 159, 165),
    token_row!(ReservedWord, "equals", 166, 172),
    token_row!(Identifier, "M", 173, 174),
    token_row!(ReservedSymbol, ".", 174, 175),
    token_row!(Identifier, "carrier", 175, 182),
    token_row!(ReservedSymbol, ";", 182, 183),
    token_row!(ReservedWord, "end", 184, 187),
    token_row!(ReservedSymbol, ";", 187, 188),
    structural_row!(StructurePattern, 20, 34, [2]),
    structural_row!(TypeHead, 62, 65, [7]),
    structural_row!(TypeExpression, 62, 65, [36]),
    structural_row!(StructureField, 45, 66, [4, 5, 6, 37, 8]),
    structural_row!(TypeHead, 90, 93, [12]),
    structural_row!(TypeExpression, 90, 93, [39]),
    structural_row!(StructureProperty, 71, 94, [9, 10, 11, 40, 13]),
    structural_row!(StructureDefinition, 13, 101, [1, 35, 3, 38, 41, 14, 15]),
    structural_row!(DefinitionBlockItem, 0, 106, [0, 42, 16, 17]),
    structural_row!(PathSegment, 130, 144, [22]),
    structural_row!(QualifiedSymbol, 130, 144, [44]),
    structural_row!(TypeHead, 130, 144, [45]),
    structural_row!(DefinitionParameter, 121, 145, [19, 20, 21, 46, 23]),
    structural_row!(TermReference, 173, 174, [29]),
    structural_row!(SelectorAccess, 173, 182, [48, 30, 31]),
    structural_row!(TermExpression, 173, 182, [49]),
    structural_row!(TermDefiniens, 173, 182, [50]),
    structural_row!(
        PropertyImplementation,
        108,
        188,
        [18, 47, 24, 25, 26, 27, 28, 51, 32, 33, 34]
    ),
    structural_row!(ItemList, 0, 188, [43, 52]),
    structural_row!(CompilationUnit, 0, 188, [53]),
    structural_row!(
        Root,
        0,
        188,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 54
        ]
    ),
];

pub(in crate::runner) fn source_property_implementation_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_property_implementation_output_impl(
        ast,
        module.clone(),
        shells,
        symbols,
        source_text,
        SourcePropertyImplementationRouteMutation::None,
    ) {
        None => None,
        Some(Ok(output)) if route_output_is_exact(&output) => Some(Vec::new()),
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
    .or_else(|| step5c4_property_implementation_detail_keys(ast, module, shells, symbols))
}

pub(in crate::runner) fn step5c4_property_implementation_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let implementations = surface_nodes_with_kind(ast, SurfaceNodeKind::PropertyImplementation);
    let structures = surface_nodes_with_kind(ast, SurfaceNodeKind::StructureDefinition);
    if implementations.len() != 1
        || structures.len() != 1
        || !surface_nodes_with_kind(ast, SurfaceNodeKind::ModeDefinition).is_empty()
    {
        return None;
    }
    let (implementation_id, implementation) = implementations[0];
    let (structure_id, structure) = structures[0];
    if subtree_has_recovery(ast, implementation)
        || subtree_has_recovery(ast, structure)
        || !shells.declarations().iter().any(|shell| {
            shell.kind() == DeclarationShellKind::PropertyImplementation
                && shell.node_id() == implementation_id
                && shell.module() == &module
                && !shell.recovered()
        })
        || !shells.declarations().iter().any(|shell| {
            shell.kind() == DeclarationShellKind::StructureDefinition
                && shell.node_id() == structure_id
                && shell.module() == &module
                && !shell.recovered()
        })
    {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    }
    let tokens = leaf_token_texts(ast, implementation);
    let property_pos = tokens.iter().position(|token| token == "property")?;
    let dot_pos = tokens.iter().position(|token| token == ".")?;
    let target_name = tokens.get(dot_pos + 1)?;
    let subject_name = tokens.get(dot_pos.wrapping_sub(1))?;
    if dot_pos <= property_pos {
        return None;
    }
    let property_nodes = surface_nodes_with_kind(ast, SurfaceNodeKind::StructureProperty);
    let declared_target = property_nodes.iter().find(|(_, node)| {
        let member_tokens = leaf_token_texts(ast, node);
        member_tokens.get(1).is_some_and(|name| name == target_name)
    });
    let structure_name = surface_nodes_with_kind(ast, SurfaceNodeKind::StructurePattern)
        .into_iter()
        .find_map(|(_, node)| leaf_token_texts(ast, node).into_iter().next());
    let Some(structure_name) = structure_name else {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    };
    let Some(structure_symbol) = symbols
        .symbols()
        .iter()
        .find(|entry| {
            entry.kind() == SymbolKind::Structure
                && entry.primary_spelling() == structure_name
                && entry.origin().source_id() == ast.source_id
                && entry.origin().module_id() == &module
                && entry.origin().anchor() == &SourceAnchor::Range(structure.range)
                && !entry.origin().is_recovered()
        })
        .map(|entry| entry.symbol().clone())
    else {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    };
    let Some((declared_target_id, declared_target)) = declared_target else {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    };
    let target_symbol = symbols.symbols().iter().find(|entry| {
        entry.kind() == SymbolKind::Selector
            && entry.primary_spelling() == *target_name
            && entry.origin().source_id() == ast.source_id
            && entry.origin().module_id() == &module
            && entry.origin().anchor() == &SourceAnchor::Range(declared_target.range)
            && !entry.origin().is_recovered()
    });
    if target_symbol.is_none()
        || !shells.declarations().iter().any(|shell| {
            shell.kind() == DeclarationShellKind::StructureProperty
                && shell.node_id() == *declared_target_id
                && shell.range() == declared_target.range
                && shell.module() == &module
                && !shell.recovered()
        })
    {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    }
    let structure_types = surface_nodes_with_kind(ast, SurfaceNodeKind::TypeExpression)
        .into_iter()
        .filter(|(_, node)| node.range.end <= structure.range.end)
        .filter_map(|(id, node)| {
            extract_builtin_source_type_expression(ast, node, &module, symbols)
                .ok()
                .map(|source| {
                    TypeExpressionInput::new(
                        surface_site(id),
                        source.range,
                        source.spelling,
                        source.head,
                    )
                    .with_attributes(source.attributes)
                })
        })
        .collect::<Vec<_>>();
    let structure_type_nodes = surface_nodes_with_kind(ast, SurfaceNodeKind::TypeExpression)
        .into_iter()
        .filter(|(_, node)| node.range.end <= structure.range.end)
        .count();
    let structure_output = TypeNormalizer::default().normalize(symbols, structure_types);
    if !structure_output.diagnostics().is_empty()
        || structure_output.type_entries().len() != structure_type_nodes
        || structure_type_nodes != 2
    {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    }
    let parameter = surface_nodes_with_kind(ast, SurfaceNodeKind::DefinitionParameter)
        .into_iter()
        .next();
    let Some((parameter_id, parameter)) = parameter else {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    };
    let parameter_tokens = leaf_token_texts(ast, parameter);
    if parameter_tokens.as_slice()
        != [
            "let",
            subject_name.as_str(),
            "be",
            structure_name.as_str(),
            ";",
        ]
        || {
            let parameter_output = TypeNormalizer::default().normalize(
                symbols,
                [TypeExpressionInput::new(
                    surface_site(parameter_id),
                    parameter.range,
                    structure_name.clone(),
                    TypeHeadInput::Symbol(structure_symbol),
                )],
            );
            !parameter_output.diagnostics().is_empty() || parameter_output.type_entries().len() != 1
        }
    {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    }
    let field_name = surface_nodes_with_kind(ast, SurfaceNodeKind::StructureField)
        .into_iter()
        .find_map(|(_, node)| leaf_token_texts(ast, node).get(1).cloned());
    let Some(field_name) = field_name else {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    };
    let field_node = surface_nodes_with_kind(ast, SurfaceNodeKind::StructureField)
        .into_iter()
        .next();
    let Some((field_id, field_node)) = field_node else {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    };
    if !symbols.symbols().iter().any(|entry| {
        entry.kind() == SymbolKind::Selector
            && entry.primary_spelling() == field_name
            && entry.origin().source_id() == ast.source_id
            && entry.origin().module_id() == &module
            && entry.origin().anchor() == &SourceAnchor::Range(field_node.range)
            && !entry.origin().is_recovered()
    }) || !shells.declarations().iter().any(|shell| {
        shell.kind() == DeclarationShellKind::StructureField
            && shell.node_id() == field_id
            && shell.range() == field_node.range
            && shell.module() == &module
            && !shell.recovered()
    }) {
        return Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]);
    }
    let form = tokens
        .iter()
        .find(|token| token.as_str() == "equals" || token.as_str() == "means")
        .map(String::as_str);
    match form {
        Some("equals") => {
            let equals = tokens
                .iter()
                .position(|token| token == "equals")
                .unwrap_or(0);
            let rhs = tokens.get(equals + 1..).unwrap_or_default();
            if rhs
                .windows(3)
                .any(|window| window == [subject_name.as_str(), ".", field_name.as_str()])
                && surface_nodes_with_kind(ast, SurfaceNodeKind::CorrectnessCondition).is_empty()
            {
                Some(Vec::new())
            } else {
                Some(vec![
                    "modes.property_implementation.unknown_property".to_owned(),
                ])
            }
        }
        Some("means") => {
            let means = tokens
                .iter()
                .position(|token| token == "means")
                .unwrap_or(0);
            let formula = tokens.get(means + 1..).unwrap_or_default();
            let conditions = surface_nodes_with_kind(ast, SurfaceNodeKind::CorrectnessCondition);
            if formula.windows(5).any(|window| {
                window == ["it", "=", subject_name.as_str(), ".", field_name.as_str()]
            }) && conditions.len() == 2
                && conditions
                    .iter()
                    .all(|(_, node)| !subtree_has_recovery(ast, node))
                && conditions
                    .iter()
                    .filter_map(|(_, node)| leaf_token_texts(ast, node).into_iter().next())
                    .eq(["existence", "uniqueness"].into_iter().map(str::to_owned))
            {
                Some(Vec::new())
            } else {
                Some(vec![
                    "modes.property_implementation.unknown_property".to_owned(),
                ])
            }
        }
        _ => Some(vec![
            "modes.property_implementation.unknown_property".to_owned(),
        ]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_property_implementation_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourcePropertyImplementationRouteOutput, String>> {
    source_property_implementation_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourcePropertyImplementationRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_property_implementation_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourcePropertyImplementationRouteMutation,
) -> Option<Result<SourcePropertyImplementationRouteOutput, String>> {
    source_property_implementation_output_impl(ast, module, shells, symbols, source_text, mutation)
}

fn source_property_implementation_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourcePropertyImplementationRouteMutation,
) -> Option<Result<SourcePropertyImplementationRouteOutput, String>> {
    let profile = if source_text == SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT
        && source_text.len() == 263
    {
        Profile::Means
    } else if source_text == SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT && source_text.len() == 189
    {
        Profile::Equals
    } else {
        return None;
    };
    if !source_text.ends_with('\n') || source_text.ends_with("\n\n") {
        return None;
    }
    Some(build_output(
        ast, module, shells, symbols, profile, mutation,
    ))
}

fn surface(profile: Profile) -> &'static [ExactSurfaceRow] {
    match profile {
        Profile::Means => MEANS_SURFACE,
        Profile::Equals => EQUALS_SURFACE,
    }
}

fn surface_is_exact(
    ast: &SurfaceAst,
    profile: Profile,
    mutation: SourcePropertyImplementationRouteMutation,
) -> bool {
    let expected = surface(profile);
    let mut root = ast.root().map(SurfaceNodeId::index);
    let mut ranges = ast
        .nodes()
        .iter()
        .map(|row| (row.range.start, row.range.end))
        .collect::<Vec<_>>();
    let mut recoveries = ast
        .nodes()
        .iter()
        .map(|row| row.recovered)
        .collect::<Vec<_>>();
    let mut children = ast
        .nodes()
        .iter()
        .map(|row| row.children.iter().map(|id| id.index()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    match mutation {
        SourcePropertyImplementationRouteMutation::WrongSurfaceRange if !ranges.is_empty() => {
            ranges[0].1 += 1
        }
        SourcePropertyImplementationRouteMutation::WrongSurfaceRecovery
            if !recoveries.is_empty() =>
        {
            recoveries[0] = true
        }
        SourcePropertyImplementationRouteMutation::WrongSurfaceChildren if !children.is_empty() => {
            children.last_mut().expect("nonempty").clear()
        }
        SourcePropertyImplementationRouteMutation::WrongSurfaceRoot => root = None,
        _ => {}
    }
    root == Some(expected.len() - 1)
        && ast.expression_root().is_none()
        && ast.nodes().len() == expected.len()
        && ast
            .nodes()
            .iter()
            .enumerate()
            .zip(expected)
            .all(|((index, actual), expected)| {
                actual.range.source_id == ast.source_id
                    && ranges[index] == (expected.start, expected.end)
                    && !recoveries[index]
                    && children[index] == expected.children
                    && match (&actual.kind, &expected.kind) {
                        (SurfaceNodeKind::Token(actual), ExactSurfaceKind::Token(kind, text)) => {
                            actual.kind == *kind && actual.text.as_ref() == *text
                        }
                        (actual, ExactSurfaceKind::Structural(expected)) => actual == expected,
                        _ => false,
                    }
            })
}

type ResolverIdentity = (SymbolId, DefinitionId, SourceContributionId);

fn resolver_profile(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    profile: Profile,
    mutation: SourcePropertyImplementationRouteMutation,
) -> Result<[ResolverIdentity; 3], String> {
    let owner = match profile {
        Profile::Means => 81,
        Profile::Equals => 52,
    };
    let root_end = match profile {
        Profile::Means => 262,
        Profile::Equals => 188,
    };
    let declarations = shells.declarations();
    let kinds = [
        DeclarationShellKind::DefinitionBlock,
        DeclarationShellKind::StructureDefinition,
        DeclarationShellKind::StructureField,
        DeclarationShellKind::StructureProperty,
        DeclarationShellKind::PropertyImplementation,
    ];
    let nodes = match profile {
        Profile::Means => [61, 60, 56, 59, 81],
        Profile::Equals => [43, 42, 38, 41, 52],
    };
    let ranges = [(0, 106), (13, 101), (45, 66), (71, 94), (108, root_end)];
    let parents = [None, Some(0), Some(1), Some(1), None];
    if declarations.len() != 5 || !shells.exports().is_empty() {
        return Err("Task264 resolver shell cardinality changed".to_owned());
    }
    for index in 0..5 {
        let row = &declarations[index];
        let ordinal =
            if mutation == SourcePropertyImplementationRouteMutation::WrongShell && index == 4 {
                3
            } else {
                row.ordinal()
            };
        if row.id().index() != index
            || ordinal != index
            || row.kind() != kinds[index]
            || row.node_id().index() != nodes[index]
            || row.range() != range(ast.source_id, ranges[index].0, ranges[index].1)
            || row.parent().map(|id| id.index()) != parents[index]
            || row.module() != module
            || row.recovered()
        {
            return Err(format!("Task264 resolver shell {index} changed"));
        }
    }
    let projections =
        SignatureProjectionExtractor::new(ast, shells, NamespacePath::new(module.path().as_str()))
            .extract();
    let expected = [
        (
            SymbolKind::Structure,
            DefinitionKind::Structure,
            "Task264Carrier",
        ),
        (SymbolKind::Selector, DefinitionKind::Selector, "carrier"),
        (SymbolKind::Selector, DefinitionKind::Selector, "marker"),
    ];
    if projections.len() != 3 {
        return Err("Task264 resolver projection cardinality changed".to_owned());
    }
    for (index, row) in projections.iter().enumerate() {
        if row.symbol_kind() != expected[index].0
            || row.definition_kind() != Some(expected[index].1)
            || row.primary_spelling() != expected[index].2
            || row.overload_policy() != SymbolOverloadPolicy::NonOverloadable
            || row.arity().is_some()
            || row.signature().is_none()
        {
            return Err(format!("Task264 resolver projection {index} changed"));
        }
    }
    if symbols.module_id() != module
        || symbols.symbols().len() != 3
        || symbols.definitions().len() != 3
        || symbols.contributions().len() != 1
    {
        return Err("Task264 resolver env cardinality changed".to_owned());
    }
    let mut identities = Vec::new();
    for (index, (symbol_kind, definition_kind, spelling)) in expected.into_iter().enumerate() {
        let definition = symbols
            .definitions()
            .iter()
            .find(|row| row.id().index() == index)
            .ok_or_else(|| format!("Task264 definition {index} disappeared"))?;
        let symbol = symbols
            .symbols()
            .get(definition.symbol())
            .ok_or_else(|| format!("Task264 symbol {index} disappeared"))?;
        let spelling = if mutation == SourcePropertyImplementationRouteMutation::WrongResolverTarget
            && index == 2
        {
            "carrier"
        } else {
            spelling
        };
        if definition.kind() != definition_kind
            || definition.visibility() != Visibility::Public
            || definition.conflict().is_some()
            || symbol.kind() != symbol_kind
            || symbol.visibility() != Visibility::Public
            || symbol.export_status() != ExportStatus::Exported
            || symbol.primary_spelling() != spelling
            || symbol.contribution() != definition.contribution()
        {
            return Err(format!("Task264 resolver identity {index} changed"));
        }
        identities.push((
            symbol.symbol().clone(),
            definition.id(),
            definition.contribution(),
        ));
    }
    let contribution = symbols
        .contributions()
        .get(identities[0].2)
        .ok_or_else(|| "Task264 contribution disappeared".to_owned())?;
    if contribution.module() != module
        || !matches!(contribution.kind(), ContributionKind::LocalSource { source_id } if *source_id == ast.source_id)
        || contribution.effects().symbols().len() != 3
        || contribution.effects().definitions().len() != 3
    {
        return Err("Task264 contribution changed".to_owned());
    }
    let result: [ResolverIdentity; 3] = identities
        .try_into()
        .map_err(|_| "Task264 resolver result changed".to_owned())?;
    let carrier = symbols
        .definitions()
        .get(result[1].1)
        .ok_or_else(|| "Task264 carrier selector disappeared".to_owned())?;
    let carrier_symbol = symbols
        .symbols()
        .get(&result[1].0)
        .ok_or_else(|| "Task264 carrier symbol disappeared".to_owned())?;
    let carrier_path: &[u32] =
        if mutation == SourcePropertyImplementationRouteMutation::WrongCarrierProvenance {
            &[4, 0, 11, 0, 19, 1]
        } else {
            carrier.origin().structural_path()
        };
    if carrier.origin().anchor() != &SourceAnchor::Range(range(ast.source_id, 45, 66))
        || carrier.origin().source_id() != ast.source_id
        || carrier.origin().module_id() != module
        || carrier_path != [4, 0, 11, 0, 18, 0]
        || carrier.origin().import_edge().is_some()
        || carrier.origin().is_recovered()
        || carrier_symbol.origin() != carrier.origin()
        || !contribution.effects().symbols().contains(&result[1].0)
        || !contribution.effects().definitions().contains(&result[1].1)
    {
        return Err("Task264 carrier-selector provenance changed".to_owned());
    }
    let marker = symbols
        .definitions()
        .get(result[2].1)
        .ok_or_else(|| "Task264 marker disappeared".to_owned())?;
    if marker.origin().anchor() != &SourceAnchor::Range(range(ast.source_id, 71, 94))
        || marker.origin().structural_path() != [4, 0, 11, 0, 19, 1]
        || marker.origin().import_edge().is_some()
        || marker.origin().is_recovered()
        || owner != nodes[4]
    {
        return Err("Task264 marker provenance changed".to_owned());
    }
    Ok(result)
}

fn task264_arena(
    source: SourceId,
    profile: Profile,
    mutation: SourcePropertyImplementationRouteMutation,
) -> Result<TypedArena, String> {
    let rows = surface(profile);
    let mut builder = TypedArenaBuilder::new();
    for (index, row) in rows.iter().enumerate() {
        let kind = match (profile, index) {
            (Profile::Means, 54 | 57 | 64) | (Profile::Equals, 36 | 39 | 46) => "source.type.head",
            (Profile::Means, 55 | 58 | 63) | (Profile::Equals, 37 | 40 | 45) => {
                "source.type.expression"
            }
            (Profile::Means, 56 | 59) | (Profile::Equals, 38 | 41) => {
                "source.definition.structure.member"
            }
            (Profile::Means, 60) | (Profile::Equals, 42) => "source.definition.structure",
            (Profile::Means, 65) | (Profile::Equals, 47) => {
                "source.definition.property-implementation.parameter"
            }
            (Profile::Means, 66 | 68) => "source.term.it",
            (Profile::Means, 70) => "source.formula.atomic.equality",
            (Profile::Means, 72) | (Profile::Equals, 51) => {
                "source.definition.property-implementation.definiens"
            }
            (Profile::Means, 76 | 80) => "source.definition.property-implementation.correctness",
            (Profile::Means, 81) | (Profile::Equals, 52) => {
                "source.definition.property-implementation"
            }
            (Profile::Equals, 48) => "source.term.variable-reference",
            (Profile::Equals, 49) => "source.term.structure.selector",
            (Profile::Equals, 31) => "source.term.structure.member.selector",
            (Profile::Means, 84) | (Profile::Equals, 55) => "source.module",
            _ => "source.surface.unowned",
        };
        let kind = if mutation == SourcePropertyImplementationRouteMutation::WrongArena
            && ((profile == Profile::Means && index == 72)
                || (profile == Profile::Equals && index == 51))
        {
            "source.surface.unowned"
        } else {
            kind
        };
        let (anchor_start, anchor_end) = if (profile == Profile::Means && index == 65)
            || (profile == Profile::Equals && index == 47)
        {
            (125, 126)
        } else {
            (row.start, row.end)
        };
        let context = if kind == "source.module" {
            LocalTypeContextId::new(0)
        } else {
            LocalTypeContextId::new(1)
        };
        let actual = builder
            .push(
                TypedNode::new(
                    kind,
                    SourceAnchor::Range(range(source, anchor_start, anchor_end)),
                )
                .with_children(row.children.iter().copied().map(TypedNodeId::new).collect())
                .with_typing(TypingState::Unknown)
                .with_recovery(NodeRecoveryState::Normal)
                .with_links(TypedNodeLinks {
                    context: Some(context),
                    ..TypedNodeLinks::default()
                }),
            )
            .map_err(|error| error.to_string())?;
        if actual.index() != index {
            return Err("Task264 arena order changed".to_owned());
        }
    }
    builder
        .finish(Some(TypedNodeId::new(rows.len() - 1)))
        .map_err(|error| error.to_string())
}

fn source_context(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    profile: Profile,
    mutation: SourcePropertyImplementationRouteMutation,
) -> Result<mizar_checker::source_context::SourceBindingContextHandoff, String> {
    let owner = match profile {
        Profile::Means => 81,
        Profile::Equals => 52,
    };
    let end = match profile {
        Profile::Means => 262,
        Profile::Equals => 188,
    };
    let parameter = match profile {
        Profile::Means => 65,
        Profile::Equals => 47,
    };
    let shell = shells
        .declarations()
        .get(4)
        .ok_or_else(|| "Task264 property shell disappeared".to_owned())?
        .id();
    let scope = LocalTermScope::new(vec![4]);
    let input = SourceBindingContextInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        module_site: node_site(match profile {
            Profile::Means => 84,
            Profile::Equals => 55,
        }),
        items: vec![SourceItemInput {
            shell,
            shell_ordinal: 4,
            role: SourceItemRole::PropertyImplementation,
            module_id: module,
            source_range: range(ast.source_id, 108, end),
            parent: None,
            visibility: SourceItemVisibility::Unspecified,
            site: node_site(owner),
            local_scope: Some(scope.clone()),
            recovery: SourceItemRecovery::Normal,
        }],
        bindings: vec![SourceBindingSiteInput {
            shell,
            context_owner: if mutation == SourcePropertyImplementationRouteMutation::WrongContext {
                SourceBindingContextOwner::Module
            } else {
                SourceBindingContextOwner::Shell(shell)
            },
            source_ordinal: 0,
            spelling: "M".to_owned(),
            declaration_range: range(ast.source_id, 125, 126),
            written_type_range: range(ast.source_id, 130, 144),
            site: node_site(parameter),
            role: SourceBindingSiteRole::DefinitionParameter {
                local: LocalTermBinding::new("M", scope, range(ast.source_id, 125, 126), 0),
            },
            recovery: BindingRecoveryState::Normal,
        }],
    };
    match SourceBindingContextProducer::build(input)
        .map_err(|error| format!("Task248P source context: {error}"))?
    {
        SourceBindingContextBuild::Complete(row) => Ok(row.into_handoff()),
        SourceBindingContextBuild::Incomplete(_) => {
            Err("Task248P source context incomplete".to_owned())
        }
        _ => Err("Task248P unsupported context build".to_owned()),
    }
}

#[allow(clippy::too_many_arguments)] // Rationale: keep every frozen lower-stage authority and corruption seam explicit at the private Task-264 route.
fn source_type(
    source: SourceId,
    module: ModuleId,
    identities: &[ResolverIdentity; 3],
    context: &mizar_checker::source_context::SourceBindingContextHandoff,
    arena: &TypedArena,
    profile: Profile,
    mutation: SourcePropertyImplementationRouteMutation,
    symbols: &SymbolEnv,
) -> Result<mizar_checker::source_type::SourceTypeApplicationHandoff, String> {
    let (expression_node, head_node, member_specs) = match profile {
        Profile::Means => (
            63,
            64,
            [(56, 45, 66, 55, 54, 62, 65), (59, 71, 94, 58, 57, 90, 93)],
        ),
        Profile::Equals => (
            45,
            46,
            [(38, 45, 66, 37, 36, 62, 65), (41, 71, 94, 40, 39, 90, 93)],
        ),
    };
    let base = SourceTypeProducer::build(
        SourceTypeHandoffInput {
            source_id: source,
            module_id: module.clone(),
            applications: vec![SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            }],
            expressions: vec![SourceTypeExpressionInput {
                source_id: source,
                module_id: module.clone(),
                site: node_site(expression_node),
                source_range: range(source, 130, 144),
                spelling: "Task264Carrier".to_owned(),
                head_site: node_site(head_node),
                head_range: range(source, 130, 144),
                head_spelling: "Task264Carrier".to_owned(),
                form: SourceTypeApplicationForm::Bare,
                head: SourceTypeHead::Symbol {
                    symbol: identities[0].0.clone(),
                    contribution: identities[0].2,
                },
                recovery: NodeRecoveryState::Normal,
            }],
            arguments: Vec::new(),
        },
        context.binding_env(),
        symbols,
        arena,
    )
    .map_err(|error| format!("Task249 source type: {error}"))?;
    let mut members = member_specs
        .into_iter()
        .enumerate()
        .map(
            |(
                source_ordinal,
                (member_node, member_start, member_end, expression_node, head_node, start, end),
            )| SourceTypeStructureMemberInput {
                member_site: node_site(member_node),
                member_range: range(source, member_start, member_end),
                source_ordinal,
                expression: SourceTypeExpressionInput {
                    source_id: source,
                    module_id: module.clone(),
                    site: node_site(expression_node),
                    source_range: range(source, start, end),
                    spelling: "set".to_owned(),
                    head_site: node_site(head_node),
                    head_range: range(source, start, end),
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                },
            },
        )
        .collect::<Vec<_>>();
    if mutation == SourcePropertyImplementationRouteMutation::WrongType {
        members[1].member_range = range(source, 45, 66);
    }
    SourceTypeStructureMemberProducer::extend_property_implementation(
        &base,
        SourceTypeStructureMemberHandoffInput {
            source_id: source,
            module_id: module,
            members,
        },
        arena,
    )
    .map_err(|error| format!("Task249PI source type: {error}"))
}

fn primary_terms(
    source: SourceId,
    module: ModuleId,
    profile: Profile,
    mutation: SourcePropertyImplementationRouteMutation,
    context: &mizar_checker::source_context::SourceBindingContextHandoff,
    arena: &TypedArena,
) -> Result<mizar_checker::source_term::SourcePrimaryTermHandoff, String> {
    let (terms, references) = match profile {
        Profile::Means => (
            vec![
                (
                    66,
                    172,
                    174,
                    "it",
                    SourcePrimaryTermKind::It,
                    SourcePrimaryTermRole::CurrentDefinitionResult,
                ),
                (
                    68,
                    177,
                    179,
                    "it",
                    SourcePrimaryTermKind::It,
                    SourcePrimaryTermRole::CurrentDefinitionResult,
                ),
            ],
            Vec::new(),
        ),
        Profile::Equals => (
            vec![(
                48,
                173,
                174,
                "M",
                SourcePrimaryTermKind::VariableReference,
                SourcePrimaryTermRole::Value,
            )],
            vec![SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(0),
                binding: BindingId::new(0),
                role: SourcePrimaryTermReferenceRole::Variable,
            }],
        ),
    };
    let mut inputs = terms
        .into_iter()
        .enumerate()
        .map(
            |(source_ordinal, (node, start, end, spelling, kind, role))| SourcePrimaryTermInput {
                site: node_site(node),
                source_range: range(source, start, end),
                source_ordinal,
                context: BindingContextId::new(1),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: spelling.to_owned(),
                kind,
                role,
                parent: None,
            },
        )
        .collect::<Vec<_>>();
    if mutation == SourcePropertyImplementationRouteMutation::WrongTerm {
        inputs[0].role = match profile {
            Profile::Means => SourcePrimaryTermRole::Value,
            Profile::Equals => SourcePrimaryTermRole::CurrentDefinitionResult,
        };
    }
    SourcePrimaryTermProducer::build(
        SourcePrimaryTermHandoffInput {
            source_id: source,
            module_id: module,
            terms: inputs,
            references,
            numeric_type_requests: Vec::new(),
        },
        context.binding_env(),
        arena,
    )
    .map_err(|error| format!("Task252 primary term: {error}"))
}

fn structure(
    source: SourceId,
    module: ModuleId,
    mutation: SourcePropertyImplementationRouteMutation,
    symbols: &SymbolEnv,
    context: &mizar_checker::source_context::SourceBindingContextHandoff,
    terms: &mizar_checker::source_term::SourcePrimaryTermHandoff,
    arena: &TypedArena,
) -> Result<mizar_checker::source_structure::SourceStructureHandoff, String> {
    let mut input = SourceStructureHandoffInput {
        source_id: source,
        module_id: module,
        terms: vec![SourceStructureTermInput {
            site: node_site(49),
            source_range: range(source, 173, 182),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceStructureRecovery::Normal,
            spelling: "M.carrier".to_owned(),
            kind: SourceStructureTermKind::SelectorAccess,
        }],
        wrappers: Vec::new(),
        roots: Vec::new(),
        members: vec![SourceStructureMemberInput {
            term: SourceStructureTermId::new(0),
            ordinal: 0,
            site: node_site(31),
            source_range: range(source, 175, 182),
            spelling: "carrier".to_owned(),
            role: SourceStructureMemberRole::Selector,
            parent: None,
        }],
        field_updates: Vec::new(),
        edges: vec![SourceStructureEdgeInput {
            term: SourceStructureTermId::new(0),
            ordinal: 0,
            role: SourceStructureEdgeRole::SelectorBase,
            member: None,
            target: SourceStructureTarget::Primary(SourcePrimaryTermId::new(0)),
        }],
        requests: vec![
            SourceStructureRequestInput {
                term: SourceStructureTermId::new(0),
                member: Some(SourceStructureMemberId::new(0)),
                request_ordinal: 0,
                kind: SourceStructureRequestKind::MemberIdentity,
            },
            SourceStructureRequestInput {
                term: SourceStructureTermId::new(0),
                member: Some(SourceStructureMemberId::new(0)),
                request_ordinal: 1,
                kind: SourceStructureRequestKind::InheritancePath,
            },
            SourceStructureRequestInput {
                term: SourceStructureTermId::new(0),
                member: None,
                request_ordinal: 2,
                kind: SourceStructureRequestKind::ResultType,
            },
        ],
    };
    if mutation == SourcePropertyImplementationRouteMutation::WrongStructure {
        input.members[0].spelling = "marker".to_owned();
    }
    SourceStructureProducer::build(input, symbols, context.binding_env(), terms, None, arena)
        .map_err(|error| format!("Task254 structure: {error}"))
}

fn atomic_formula(
    source: SourceId,
    module: ModuleId,
    mutation: SourcePropertyImplementationRouteMutation,
    symbols: &SymbolEnv,
    context: &mizar_checker::source_context::SourceBindingContextHandoff,
    terms: &mizar_checker::source_term::SourcePrimaryTermHandoff,
    arena: &TypedArena,
) -> Result<mizar_checker::source_atomic_formula::SourceAtomicFormulaHandoff, String> {
    let mut input = SourceAtomicFormulaHandoffInput {
        source_id: source,
        module_id: module,
        formulas: vec![SourceAtomicFormulaInput {
            site: node_site(70),
            source_range: range(source, 172, 179),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "it = it".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        }],
        wrappers: Vec::new(),
        predicate_segments: Vec::new(),
        predicate_heads: Vec::new(),
        candidates: Vec::new(),
        type_sites: Vec::new(),
        attributes: Vec::new(),
        edges: vec![
            SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                role: SourceAtomicEdgeRole::BuiltinLeftOperand,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0)),
            },
            SourceAtomicEdgeInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 1,
                role: SourceAtomicEdgeRole::BuiltinRightOperand,
                target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(1)),
            },
        ],
        requests: vec![
            SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 0,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(0)),
                candidate: None,
                type_site: None,
                attribute: None,
            },
            SourceAtomicRequestInput {
                formula: SourceAtomicFormulaId::new(0),
                ordinal: 1,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(1)),
                candidate: None,
                type_site: None,
                attribute: None,
            },
        ],
    };
    if mutation == SourcePropertyImplementationRouteMutation::WrongFormula {
        input.edges[1].target = SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(0));
    }
    SourceAtomicFormulaProducer::build(
        input,
        context.binding_env(),
        symbols,
        terms,
        None,
        None,
        None,
        arena,
    )
    .map_err(|error| format!("Task256 atomic formula: {error}"))
}

fn checker_input(
    source: SourceId,
    module: ModuleId,
    profile: Profile,
    shell: mizar_resolve::declarations::DeclarationShellId,
    identities: &[ResolverIdentity; 3],
) -> SourcePropertyImplementationHandoffInput {
    let (
        owner,
        end,
        parameter,
        style,
        implementation_spelling,
        definiens_node,
        definiens_range,
        definiens_spelling,
        target,
        correctness,
    ) = match profile {
        Profile::Means => (
            81,
            262,
            65,
            SourcePropertyImplementationStyle::Means,
            "definition\n  let M be Task264Carrier;\n  property M.marker means it = it;\n  existence by computation(steps: 1);\n  uniqueness by computation(steps: 1);\nend;",
            72,
            (172, 179),
            "it = it",
            SourcePropertyDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(0)),
            vec![
                (
                    SourcePropertyCorrectnessKind::Existence,
                    76,
                    183,
                    218,
                    193,
                    217,
                    "existence by computation(steps: 1);",
                ),
                (
                    SourcePropertyCorrectnessKind::Uniqueness,
                    80,
                    221,
                    257,
                    232,
                    256,
                    "uniqueness by computation(steps: 1);",
                ),
            ],
        ),
        Profile::Equals => (
            52,
            188,
            47,
            SourcePropertyImplementationStyle::Equals,
            "definition\n  let M be Task264Carrier;\n  property M.marker equals M.carrier;\nend;",
            51,
            (173, 182),
            "M.carrier",
            SourcePropertyDefiniensTarget::Structure(SourceStructureTermId::new(0)),
            Vec::new(),
        ),
    };
    SourcePropertyImplementationHandoffInput {
        source_id: source,
        module_id: module,
        implementations: vec![SourcePropertyImplementationInput {
            shell,
            site: node_site(owner),
            source_range: range(source, 108, end),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourcePropertyImplementationRecovery::Normal,
            spelling: implementation_spelling.to_owned(),
            style,
            parameter: SourcePropertyParameterId::new(0),
            target: SourcePropertyTargetId::new(0),
            definiens: SourcePropertyDefiniensId::new(0),
        }],
        parameters: vec![SourcePropertyParameterInput {
            owner: SourcePropertyImplementationId::new(0),
            ordinal: 0,
            binding: BindingId::new(0),
            written_type: SourceTypeApplicationId::new(0),
            site: node_site(parameter),
            source_range: range(source, 121, 145),
            declaration_range: range(source, 125, 126),
            context: BindingContextId::new(1),
            recovery: SourcePropertyImplementationRecovery::Normal,
            spelling: "let M be Task264Carrier;".to_owned(),
        }],
        targets: vec![SourcePropertyTargetInput {
            owner: SourcePropertyImplementationId::new(0),
            ordinal: 0,
            subject: BindingId::new(0),
            symbol: identities[2].0.clone(),
            definition: identities[2].1,
            contribution: identities[2].2,
            site: TypedSiteRef::Role {
                node: TypedNodeId::new(owner),
                role: TypeRole::new("source.property-implementation.target"),
            },
            source_range: range(source, 157, 165),
            subject_range: range(source, 157, 158),
            name_range: range(source, 159, 165),
            spelling: "M.marker".to_owned(),
            return_type: SourceTypeStructureMemberId::new(1),
        }],
        definientia: vec![SourcePropertyDefiniensInput {
            owner: SourcePropertyImplementationId::new(0),
            ordinal: 0,
            target,
            site: node_site(definiens_node),
            source_range: range(source, definiens_range.0, definiens_range.1),
            context: BindingContextId::new(1),
            recovery: SourcePropertyImplementationRecovery::Normal,
            spelling: definiens_spelling.to_owned(),
        }],
        correctness: correctness
            .into_iter()
            .enumerate()
            .map(
                |(ordinal, (kind, node, start, end, proof_start, proof_end, spelling))| {
                    SourcePropertyCorrectnessInput {
                        owner: SourcePropertyImplementationId::new(0),
                        ordinal,
                        kind,
                        site: node_site(node),
                        source_range: range(source, start, end),
                        justification: SourceAnchor::Range(range(source, proof_start, proof_end)),
                        recovery: SourcePropertyImplementationRecovery::Normal,
                        spelling: spelling.to_owned(),
                    }
                },
            )
            .collect(),
    }
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    profile: Profile,
    mutation: SourcePropertyImplementationRouteMutation,
) -> Result<SourcePropertyImplementationRouteOutput, String> {
    if !surface_is_exact(ast, profile, mutation) {
        return Err("Task264 Surface profile is not exact".to_owned());
    }
    let identities = resolver_profile(ast, &module, shells, symbols, profile, mutation)?;
    let arena = task264_arena(ast.source_id, profile, mutation)?;
    let context = source_context(ast, module.clone(), shells, profile, mutation)?;
    let source_type = source_type(
        ast.source_id,
        module.clone(),
        &identities,
        &context,
        &arena,
        profile,
        mutation,
        symbols,
    )?;
    let contexts = context.local_contexts().clone();
    let mut typed = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module.clone(),
        resolved_root: None,
        source_context: Some(context),
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
    .map_err(|error| format!("Task249PI typed base: {error}"))?;
    let terms = primary_terms(
        ast.source_id,
        module.clone(),
        profile,
        mutation,
        typed
            .source_context()
            .ok_or_else(|| "Task248P context disappeared".to_owned())?,
        typed.nodes(),
    )?;
    typed = typed
        .with_source_term(terms)
        .map_err(|error| format!("Task252 typed install: {error}"))?;
    match profile {
        Profile::Means => {
            let formula = atomic_formula(
                ast.source_id,
                module.clone(),
                mutation,
                symbols,
                typed
                    .source_context()
                    .ok_or_else(|| "Task248P context disappeared".to_owned())?,
                typed
                    .source_term()
                    .ok_or_else(|| "Task252 terms disappeared".to_owned())?,
                typed.nodes(),
            )?;
            typed = typed
                .with_source_atomic_formula(formula)
                .map_err(|error| format!("Task256 typed install: {error}"))?;
        }
        Profile::Equals => {
            let row = structure(
                ast.source_id,
                module.clone(),
                mutation,
                symbols,
                typed
                    .source_context()
                    .ok_or_else(|| "Task248P context disappeared".to_owned())?,
                typed
                    .source_term()
                    .ok_or_else(|| "Task252 terms disappeared".to_owned())?,
                typed.nodes(),
            )?;
            typed = typed
                .with_source_structure(row)
                .map_err(|error| format!("Task254 typed install: {error}"))?;
        }
    }
    let shell = shells.declarations()[4].id();
    let mut input = checker_input(ast.source_id, module, profile, shell, &identities);
    if mutation == SourcePropertyImplementationRouteMutation::WrongImplementation {
        input.targets[0].return_type = SourceTypeStructureMemberId::new(0);
    }
    let projection = SourcePropertyImplementationProducer::build(
        input,
        symbols,
        typed
            .source_context()
            .ok_or_else(|| "Task248P context disappeared".to_owned())?,
        typed
            .source_type()
            .ok_or_else(|| "Task249PI type disappeared".to_owned())?,
        typed
            .source_term()
            .ok_or_else(|| "Task252 terms disappeared".to_owned())?,
        None,
        typed.source_structure(),
        None,
        typed.source_atomic_formula(),
        typed.initial_obligations(),
        typed.nodes(),
    )
    .map_err(|error| format!("Task264 property implementation: {error}"))?;
    typed = typed
        .with_source_property_implementation(projection)
        .map_err(|error| format!("Task264 typed install: {error}"))?;
    let node_hints = typed
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.definition.property-implementation"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed, node_hints)
        .map_err(|error| format!("Task264 final assembly: {error}"))?;
    Ok(SourcePropertyImplementationRouteOutput {
        typed_ast: typed,
        resolved,
    })
}

fn route_output_is_exact(output: &SourcePropertyImplementationRouteOutput) -> bool {
    let typed = &output.typed_ast;
    let resolved = &output.resolved;
    let Some(handoff) = typed.source_property_implementation() else {
        return false;
    };
    let means = handoff
        .implementations()
        .get(SourcePropertyImplementationId::new(0))
        .is_some_and(|row| row.style() == SourcePropertyImplementationStyle::Means);
    handoff.implementations().len() == 1
        && handoff.parameters().len() == 1
        && handoff.targets().len() == 1
        && handoff.definientia().len() == 1
        && handoff.correctness().len() == if means { 2 } else { 0 }
        && typed.source_property_implementation() == resolved.source_property_implementation()
        && typed.source_context() == resolved.source_context()
        && typed.source_type() == resolved.source_type()
        && typed.source_term() == resolved.source_term()
        && typed.source_structure() == resolved.source_structure()
        && typed.source_atomic_formula() == resolved.source_atomic_formula()
        && typed.initial_obligations().len() == if means { 2 } else { 0 }
        && typed.initial_obligations().iter().all(|(_, row)| {
            matches!(
                row.kind,
                InitialObligationKind::PropertyImplementationExistence
                    | InitialObligationKind::PropertyImplementationUniqueness
            )
        })
        && typed.source_predicate_definition().is_none()
        && typed.source_functor_definition().is_none()
        && typed.types().is_empty()
        && typed.facts().is_empty()
        && typed.coercions().is_empty()
        && typed.diagnostics().is_empty()
        && resolved.expr_metadata().is_empty()
        && resolved.cluster_facts().is_empty()
        && resolved.diagnostics().is_empty()
        && resolved.checked_formulas().is_empty()
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
const fn node_site(index: usize) -> TypedSiteRef {
    TypedSiteRef::Node(TypedNodeId::new(index))
}
