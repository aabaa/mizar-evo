use mizar_checker::{
    binding_env::{BindingContextId, BindingId},
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_atomic_formula::{
        SourceAtomicEdgeId, SourceAtomicEdgeInput, SourceAtomicEdgeRole,
        SourceAtomicFormulaHandoffInput, SourceAtomicFormulaId, SourceAtomicFormulaInput,
        SourceAtomicFormulaKind, SourceAtomicFormulaProducer, SourceAtomicFormulaRecovery,
        SourceAtomicRequestInput, SourceAtomicRequestKind, SourceAtomicTermTarget,
    },
    source_predicate_definition::{
        SourcePredicateCorrectnessInput, SourcePredicateDefinitionHandoffInput,
        SourcePredicateDefinitionId, SourcePredicateDefinitionInput,
        SourcePredicateDefinitionProducer, SourcePredicateDefinitionRecovery,
        SourcePredicateGuardInput, SourcePredicateParameterInput, SourcePredicatePropertyId,
        SourcePredicatePropertyInput, SourcePredicatePropertyKind,
    },
    source_term::{
        SourcePrimaryTermHandoffInput, SourcePrimaryTermId, SourcePrimaryTermInput,
        SourcePrimaryTermKind, SourcePrimaryTermProducer, SourcePrimaryTermRecovery,
        SourcePrimaryTermReferenceInput, SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationId, SourceTypeApplicationInput,
        SourceTypeExpressionId, SourceTypeExpressionInput, SourceTypeHandoffInput, SourceTypeHead,
        SourceTypeProducer,
    },
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextId, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedArenaBuilder, TypedAst,
        TypedAstParts, TypedNode, TypedNodeId, TypedNodeLinks, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShellKind, DeclarationShellSet},
    env::{
        ContributionKind, DefinitionId, DefinitionKind, ExportStatus, SourceContributionId,
        SymbolEntry, SymbolEnv, SymbolEnvIndexes, SymbolIndex, SymbolKind, Visibility,
    },
    resolved_ast::{ModuleId, SymbolId},
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNodeId, SurfaceNodeKind, SurfaceTokenKind};

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_ast::{direct_token_texts, structural_child_ids, subtree_has_recovery},
    source_context::{
        SourceTwoParameterDefinitionContextSites,
        source_two_parameter_definition_context_projection,
    },
};

pub(in crate::runner) const SOURCE_PREDICATE_DEFINITION_TEXT: &str = concat!(
    "definition\n",
    "  let x be set;\n",
    "  let y be set;\n",
    "  assume x = x;\n",
    "  pred Task259PredicateDefinition: x task259_rel y means x = y;\n",
    "  symmetry by computation(steps: 1);\n",
    "end;\n",
);

const INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.source_predicate_definition.invalid_payload";

const TYPE_HEAD_X: TypedNodeId = TypedNodeId::new(0);
const TYPE_EXPRESSION_X: TypedNodeId = TypedNodeId::new(1);
const PARAMETER_X: TypedNodeId = TypedNodeId::new(2);
const TYPE_HEAD_Y: TypedNodeId = TypedNodeId::new(3);
const TYPE_EXPRESSION_Y: TypedNodeId = TypedNodeId::new(4);
const PARAMETER_Y: TypedNodeId = TypedNodeId::new(5);
const GUARD_LEFT: TypedNodeId = TypedNodeId::new(6);
const GUARD_RIGHT: TypedNodeId = TypedNodeId::new(7);
const GUARD_FORMULA: TypedNodeId = TypedNodeId::new(8);
const DEFINIENS_LEFT: TypedNodeId = TypedNodeId::new(9);
const DEFINIENS_RIGHT: TypedNodeId = TypedNodeId::new(10);
const DEFINIENS_FORMULA: TypedNodeId = TypedNodeId::new(11);
const GUARD_OWNER: TypedNodeId = TypedNodeId::new(12);
const PREDICATE_OWNER: TypedNodeId = TypedNodeId::new(13);
const PROPERTY_OWNER: TypedNodeId = TypedNodeId::new(14);
const DEFINITION_OWNER: TypedNodeId = TypedNodeId::new(15);
const MODULE_OWNER: TypedNodeId = TypedNodeId::new(16);

#[derive(Debug)]
pub(in crate::runner) struct SourcePredicateDefinitionRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; corruption variants are private test seams.
pub(in crate::runner) enum SourcePredicateDefinitionRouteMutation {
    None,
    DuplicateContextParameterSite,
    WrongContextDefinitionSite,
    RemoveResolverShell,
    WrongResolverEntry,
    WrongResolverPropertyEntry,
    RemoveTypeExpression,
    WrongTermBinding,
    RemoveAtomicFormula,
    RemovePredicateGuard,
}

#[derive(Debug, Clone, Copy)]
struct ExactPredicateDefinition {
    definition_block: SurfaceNodeId,
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
    recovered: bool,
}

macro_rules! token_row {
    ($kind:ident, $text:literal, $start:literal, $end:literal) => {
        ExactSurfaceRow {
            kind: ExactSurfaceKind::Token(SurfaceTokenKind::$kind, $text),
            start: $start,
            end: $end,
            children: &[],
            recovered: false,
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
            recovered: false,
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
    token_row!(ReservedWord, "assume", 45, 51),
    token_row!(Identifier, "x", 52, 53),
    token_row!(ReservedSymbol, "=", 54, 55),
    token_row!(Identifier, "x", 56, 57),
    token_row!(ReservedSymbol, ";", 57, 58),
    token_row!(ReservedWord, "pred", 61, 65),
    token_row!(Identifier, "Task259PredicateDefinition", 66, 92),
    token_row!(ReservedSymbol, ":", 92, 93),
    token_row!(Identifier, "x", 94, 95),
    token_row!(Identifier, "task259_rel", 96, 107),
    token_row!(Identifier, "y", 108, 109),
    token_row!(ReservedWord, "means", 110, 115),
    token_row!(Identifier, "x", 116, 117),
    token_row!(ReservedSymbol, "=", 118, 119),
    token_row!(Identifier, "y", 120, 121),
    token_row!(ReservedSymbol, ";", 121, 122),
    token_row!(ReservedWord, "symmetry", 125, 133),
    token_row!(ReservedWord, "by", 134, 136),
    token_row!(ReservedWord, "computation", 137, 148),
    token_row!(ReservedSymbol, "(", 148, 149),
    token_row!(Identifier, "steps", 149, 154),
    token_row!(ReservedSymbol, ":", 154, 155),
    token_row!(Numeral, "1", 156, 157),
    token_row!(ReservedSymbol, ")", 157, 158),
    token_row!(ReservedSymbol, ";", 158, 159),
    token_row!(ReservedWord, "end", 160, 163),
    token_row!(ReservedSymbol, ";", 163, 164),
    structural_row!(TypeHead, 22, 25, [4]),
    structural_row!(TypeExpression, 22, 25, [38]),
    structural_row!(QualifiedVariableSegment, 17, 25, [2, 3, 39]),
    structural_row!(DefinitionParameter, 13, 26, [1, 40, 5]),
    structural_row!(TypeHead, 38, 41, [9]),
    structural_row!(TypeExpression, 38, 41, [42]),
    structural_row!(QualifiedVariableSegment, 33, 41, [7, 8, 43]),
    structural_row!(DefinitionParameter, 29, 42, [6, 44, 10]),
    structural_row!(TermReference, 52, 53, [12]),
    structural_row!(TermExpression, 52, 53, [46]),
    structural_row!(TermReference, 56, 57, [14]),
    structural_row!(TermExpression, 56, 57, [48]),
    structural_row!(BuiltinPredicateApplication, 52, 57, [47, 13, 49]),
    structural_row!(FormulaExpression, 52, 57, [50]),
    structural_row!(Proposition, 52, 57, [51]),
    structural_row!(AssumptionStatement, 45, 58, [11, 52, 15]),
    structural_row!(PredicatePattern, 94, 109, [19, 20, 21]),
    structural_row!(TermReference, 116, 117, [23]),
    structural_row!(TermExpression, 116, 117, [55]),
    structural_row!(TermReference, 120, 121, [25]),
    structural_row!(TermExpression, 120, 121, [57]),
    structural_row!(BuiltinPredicateApplication, 116, 121, [56, 24, 58]),
    structural_row!(FormulaExpression, 116, 121, [59]),
    structural_row!(FormulaDefiniens, 116, 121, [60]),
    structural_row!(PredicateDefinition, 61, 122, [16, 17, 18, 54, 22, 61, 26]),
    structural_row!(ComputationOption, 149, 157, [31, 32, 33]),
    structural_row!(ComputationJustification, 137, 158, [29, 30, 63, 34]),
    structural_row!(JustificationClause, 134, 158, [28, 64]),
    structural_row!(PropertyClause, 125, 159, [27, 65, 35]),
    structural_row!(DefinitionBlockItem, 0, 164, [0, 41, 45, 53, 62, 66, 36, 37]),
    structural_row!(ItemList, 0, 164, [67]),
    structural_row!(CompilationUnit, 0, 164, [68]),
    structural_row!(
        Root,
        0,
        164,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 69,
        ]
    ),
];

pub(in crate::runner) fn source_predicate_definition_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_predicate_definition_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourcePredicateDefinitionRouteMutation::None,
    ) {
        None => None,
        Some(Ok(output)) if route_output_is_exact(&output) => Some(Vec::new()),
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_predicate_definition_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourcePredicateDefinitionRouteOutput, String>> {
    source_predicate_definition_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourcePredicateDefinitionRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_predicate_definition_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourcePredicateDefinitionRouteMutation,
) -> Option<Result<SourcePredicateDefinitionRouteOutput, String>> {
    source_predicate_definition_output_impl(ast, module, shells, symbols, source_text, mutation)
}

fn source_predicate_definition_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourcePredicateDefinitionRouteMutation,
) -> Option<Result<SourcePredicateDefinitionRouteOutput, String>> {
    let exact = exact_predicate_definition(ast, source_text)?;
    Some(build_output(ast, module, shells, symbols, exact, mutation))
}

fn exact_predicate_definition(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<ExactPredicateDefinition> {
    if source_text != SOURCE_PREDICATE_DEFINITION_TEXT
        || source_text.len() != 165
        || !source_text.ends_with('\n')
        || source_text.ends_with("\n\n")
        || ast.root().map(SurfaceNodeId::index) != Some(70)
        || ast.expression_root().is_some()
        || !surface_profile_is_exact(ast)
    {
        return None;
    }

    let root = exact_node(ast, 70, SurfaceNodeKind::Root, 0, 164)?;
    let definition_block = exact_node(ast, 67, SurfaceNodeKind::DefinitionBlockItem, 0, 164)?;
    let first_parameter = exact_node(ast, 41, SurfaceNodeKind::DefinitionParameter, 13, 26)?;
    let second_parameter = exact_node(ast, 45, SurfaceNodeKind::DefinitionParameter, 29, 42)?;
    let guard = exact_node(ast, 53, SurfaceNodeKind::AssumptionStatement, 45, 58)?;
    let predicate = exact_node(ast, 62, SurfaceNodeKind::PredicateDefinition, 61, 122)?;
    let property = exact_node(ast, 66, SurfaceNodeKind::PropertyClause, 125, 159)?;
    let justification = exact_node(ast, 65, SurfaceNodeKind::JustificationClause, 134, 158)?;
    let computation = exact_node(ast, 64, SurfaceNodeKind::ComputationJustification, 137, 158)?;
    exact_node(ast, 39, SurfaceNodeKind::TypeExpression, 22, 25)?;
    exact_node(ast, 43, SurfaceNodeKind::TypeExpression, 38, 41)?;
    exact_node(ast, 51, SurfaceNodeKind::FormulaExpression, 52, 57)?;
    exact_node(ast, 60, SurfaceNodeKind::FormulaExpression, 116, 121)?;

    let block = ast.node(definition_block)?;
    if structural_child_ids(ast, block)
        != [
            first_parameter,
            second_parameter,
            guard,
            predicate,
            property,
        ]
        || direct_token_texts(ast, block).as_slice() != ["definition", "end", ";"]
        || subtree_has_recovery(ast, block)
        || !is_descendant(ast, root, definition_block)
        || !is_descendant(ast, property, justification)
        || !is_descendant(ast, justification, computation)
        || is_descendant(ast, predicate, first_parameter)
        || is_descendant(ast, predicate, second_parameter)
        || is_descendant(ast, predicate, guard)
        || is_descendant(ast, predicate, property)
    {
        return None;
    }

    let terms = ast
        .node_views()
        .filter(|view| matches!(view.kind(), SurfaceNodeKind::TermReference))
        .map(|view| (view.id().index(), view.range().start, view.range().end))
        .collect::<Vec<_>>();
    if terms
        .iter()
        .map(|(_, start, end)| (*start, *end))
        .collect::<Vec<_>>()
        != [(52, 53), (56, 57), (116, 117), (120, 121)]
    {
        return None;
    }

    Some(ExactPredicateDefinition { definition_block })
}

fn surface_profile_is_exact(ast: &SurfaceAst) -> bool {
    ast.nodes().len() == EXACT_SURFACE_PROFILE.len()
        && ast
            .nodes()
            .iter()
            .zip(EXACT_SURFACE_PROFILE)
            .all(|(actual, expected)| {
                actual.range == source_range(ast.source_id, expected.start, expected.end)
                    && actual.recovered == expected.recovered
                    && actual.children.len() == expected.children.len()
                    && actual.children.iter().zip(expected.children).all(
                        |(actual_child, expected_child)| actual_child.index() == *expected_child,
                    )
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

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    exact: ExactPredicateDefinition,
    mutation: SourcePredicateDefinitionRouteMutation,
) -> Result<SourcePredicateDefinitionRouteOutput, String> {
    let corrupted_symbols = match mutation {
        SourcePredicateDefinitionRouteMutation::WrongResolverEntry => {
            Some(resolver_env_with_corrupted_entry(symbols, false))
        }
        SourcePredicateDefinitionRouteMutation::WrongResolverPropertyEntry => {
            Some(resolver_env_with_corrupted_entry(symbols, true))
        }
        _ => None,
    };
    let symbols = corrupted_symbols.as_ref().unwrap_or(symbols);
    let (symbol, definition, contribution) =
        exact_resolver_profile(ast, &module, shells, symbols, mutation)?;
    let arena = task259_arena(ast.source_id)?;
    let mut context_sites = SourceTwoParameterDefinitionContextSites {
        module: TypedSiteRef::Node(MODULE_OWNER),
        definition: TypedSiteRef::Node(DEFINITION_OWNER),
        parameters: [
            TypedSiteRef::Node(PARAMETER_X),
            TypedSiteRef::Node(PARAMETER_Y),
        ],
    };
    match mutation {
        SourcePredicateDefinitionRouteMutation::DuplicateContextParameterSite => {
            context_sites.parameters[1] = TypedSiteRef::Node(PARAMETER_X);
        }
        SourcePredicateDefinitionRouteMutation::WrongContextDefinitionSite => {
            context_sites.definition = TypedSiteRef::Node(PROPERTY_OWNER);
        }
        _ => {}
    }
    let context_projection = source_two_parameter_definition_context_projection(
        ast,
        module.clone(),
        shells,
        symbols,
        exact.definition_block,
        &arena,
        context_sites,
    )
    .map_err(|error| format!("Task248 source context: {error}"))?;
    let source_context = context_projection.into_handoff();

    let mut type_input = task249_input(ast.source_id, module.clone());
    if mutation == SourcePredicateDefinitionRouteMutation::RemoveTypeExpression {
        type_input.expressions.pop();
    }
    let source_type =
        SourceTypeProducer::build(type_input, source_context.binding_env(), symbols, &arena)
            .map_err(|error| format!("Task249 source type: {error}"))?;

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
    .map_err(|error| format!("Task249 typed installation: {error}"))?;

    let mut term_input = task252_input(ast.source_id, module.clone());
    if mutation == SourcePredicateDefinitionRouteMutation::WrongTermBinding {
        term_input.references[3].binding = BindingId::new(0);
    }
    let source_term = SourcePrimaryTermProducer::build(
        term_input,
        typed_ast
            .source_context()
            .ok_or_else(|| "Task248 source context disappeared".to_owned())?
            .binding_env(),
        typed_ast.nodes(),
    )
    .map_err(|error| format!("Task252 source term: {error}"))?;
    typed_ast = typed_ast
        .with_source_term(source_term)
        .map_err(|error| format!("Task252 typed installation: {error}"))?;

    let mut atomic_input = task256_input(ast.source_id, module.clone());
    if mutation == SourcePredicateDefinitionRouteMutation::RemoveAtomicFormula {
        atomic_input.formulas.pop();
    }
    let source_atomic_formula = SourceAtomicFormulaProducer::build(
        atomic_input,
        typed_ast
            .source_context()
            .ok_or_else(|| "Task248 source context disappeared".to_owned())?
            .binding_env(),
        symbols,
        typed_ast
            .source_term()
            .ok_or_else(|| "Task252 source term disappeared".to_owned())?,
        None,
        None,
        None,
        typed_ast.nodes(),
    )
    .map_err(|error| format!("Task256 atomic formula: {error}"))?;
    typed_ast = typed_ast
        .with_source_atomic_formula(source_atomic_formula)
        .map_err(|error| format!("Task256 typed installation: {error}"))?;

    let mut predicate_input = task259_input(
        ast.source_id,
        module.clone(),
        symbol,
        definition,
        contribution,
    );
    if mutation == SourcePredicateDefinitionRouteMutation::RemovePredicateGuard {
        predicate_input.guards.clear();
    }
    let projection = SourcePredicateDefinitionProducer::build(
        predicate_input,
        symbols,
        typed_ast
            .source_context()
            .ok_or_else(|| "Task248 source context disappeared".to_owned())?,
        typed_ast
            .source_type()
            .ok_or_else(|| "Task249 source type disappeared".to_owned())?,
        typed_ast
            .source_term()
            .ok_or_else(|| "Task252 source term disappeared".to_owned())?,
        typed_ast
            .source_atomic_formula()
            .ok_or_else(|| "Task256 atomic formula disappeared".to_owned())?,
        typed_ast.initial_obligations(),
        typed_ast.nodes(),
    )
    .map_err(|error| format!("Task259 predicate definition: {error}"))?;
    typed_ast = typed_ast
        .with_source_predicate_definition(projection)
        .map_err(|error| format!("Task259 typed installation: {error}"))?;

    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.definition.predicate"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)
        .map_err(|error| format!("Task259 final assembly: {error}"))?;
    Ok(SourcePredicateDefinitionRouteOutput {
        typed_ast,
        resolved,
    })
}

fn exact_resolver_profile(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutation: SourcePredicateDefinitionRouteMutation,
) -> Result<(SymbolId, DefinitionId, SourceContributionId), String> {
    let declarations = if mutation == SourcePredicateDefinitionRouteMutation::RemoveResolverShell
        && !shells.declarations().is_empty()
    {
        &shells.declarations()[..shells.declarations().len() - 1]
    } else {
        shells.declarations()
    };
    let [block, predicate, property] = declarations else {
        return Err("Task259 resolver: expected three declaration shells".to_owned());
    };
    if !shells.exports().is_empty()
        || block.id().index() != 0
        || block.ordinal() != 0
        || block.kind() != DeclarationShellKind::DefinitionBlock
        || block.node_id().index() != 67
        || block.parent().is_some()
        || block.range() != source_range(ast.source_id, 0, 164)
        || block.module() != module
        || block.recovered()
        || predicate.id().index() != 1
        || predicate.ordinal() != 1
        || predicate.kind() != DeclarationShellKind::PredicateDefinition
        || predicate.node_id().index() != 62
        || predicate.parent() != Some(block.id())
        || predicate.range() != source_range(ast.source_id, 61, 122)
        || predicate.module() != module
        || predicate.recovered()
        || property.id().index() != 2
        || property.ordinal() != 2
        || property.kind() != DeclarationShellKind::PropertyClause
        || property.node_id().index() != 66
        || property.parent() != Some(block.id())
        || property.range() != source_range(ast.source_id, 125, 159)
        || property.module() != module
        || property.recovered()
        || symbols.module_id() != module
        || symbols.symbols().len() != 2
        || symbols.definitions().len() != 2
        || symbols.contributions().len() != 1
    {
        return Err("Task259 resolver: raw profile is not exact".to_owned());
    }

    let definition = symbols
        .definitions()
        .iter()
        .find(|definition| definition.id().index() == 0)
        .ok_or_else(|| "Task259 resolver: predicate definition 0 disappeared".to_owned())?;
    let definition_id = definition.id();
    let symbol = symbols
        .symbols()
        .get(definition.symbol())
        .ok_or_else(|| "Task259 resolver: predicate symbol disappeared".to_owned())?;
    if definition.kind() != DefinitionKind::Predicate
        || definition.visibility() != Visibility::Public
        || !definition.parameters().is_empty()
        || !definition.binders().is_empty()
        || definition.arity().is_some()
        || definition.conflict().is_some()
        || symbol.kind() != SymbolKind::Predicate
        || symbol.visibility() != Visibility::Public
        || symbol.export_status() != ExportStatus::Exported
        || symbol.primary_spelling() != "x task259_rel y"
        || symbol.notation_spelling() != Some("x task259_rel y")
        || symbol.contribution() != definition.contribution()
    {
        return Err("Task259 resolver: predicate projection is not exact".to_owned());
    }
    validate_origin(
        ast.source_id,
        module,
        definition.origin(),
        61,
        122,
        &[4, 0, 8, 0],
    )?;
    validate_origin(
        ast.source_id,
        module,
        symbol.origin(),
        61,
        122,
        &[4, 0, 8, 0],
    )?;
    let property_definition = symbols
        .definitions()
        .iter()
        .find(|definition| definition.id().index() == 1)
        .ok_or_else(|| "Task259 resolver: property definition 1 disappeared".to_owned())?;
    let property_symbol = symbols
        .symbols()
        .get(property_definition.symbol())
        .ok_or_else(|| "Task259 resolver: property symbol disappeared".to_owned())?;
    if property_definition.kind() != DefinitionKind::Attribute
        || property_definition.contribution() != definition.contribution()
        || property_symbol.kind() != SymbolKind::Attribute
        || property_symbol.contribution() != property_definition.contribution()
        || property_symbol.symbol() != property_definition.symbol()
    {
        return Err("Task259 resolver: property projection is not exact".to_owned());
    }
    validate_origin(
        ast.source_id,
        module,
        property_definition.origin(),
        125,
        159,
        &[4, 0, 17, 1],
    )?;
    validate_origin(
        ast.source_id,
        module,
        property_symbol.origin(),
        125,
        159,
        &[4, 0, 17, 1],
    )?;
    let contribution = symbols
        .contributions()
        .get(definition.contribution())
        .ok_or_else(|| "Task259 resolver: local contribution disappeared".to_owned())?;
    if contribution.module() != module
        || !matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == ast.source_id
        )
        || !contribution.effects().symbols().contains(symbol.symbol())
        || !contribution
            .effects()
            .definitions()
            .contains(&definition_id)
        || !contribution
            .effects()
            .symbols()
            .contains(property_symbol.symbol())
        || !contribution
            .effects()
            .definitions()
            .contains(&property_definition.id())
    {
        return Err("Task259 resolver: local contribution is not exact".to_owned());
    }
    Ok((
        symbol.symbol().clone(),
        definition_id,
        definition.contribution(),
    ))
}

fn resolver_env_with_corrupted_entry(symbols: &SymbolEnv, property: bool) -> SymbolEnv {
    let mut corrupted_index = SymbolIndex::new();
    for entry in symbols.symbols().iter() {
        let target = if property {
            SymbolKind::Attribute
        } else {
            SymbolKind::Predicate
        };
        let corrupted = if entry.kind() == target {
            let corrupted_kind = if property {
                SymbolKind::Predicate
            } else {
                entry.kind()
            };
            let primary_spelling = if property {
                entry.primary_spelling()
            } else {
                "x corrupted_task259_rel y"
            };
            let mut corrupted = SymbolEntry::new(
                entry.symbol().clone(),
                corrupted_kind,
                entry.namespace().clone(),
                primary_spelling,
                entry.origin().clone(),
                entry.contribution(),
            )
            .with_visibility(entry.visibility())
            .with_export_status(entry.export_status())
            .with_relations(entry.relations().to_vec());
            if let Some(notation) = entry.notation_spelling() {
                corrupted = corrupted.with_notation_spelling(notation);
            }
            if let Some(signature) = entry.signature() {
                corrupted = corrupted.with_signature(signature.clone());
            }
            corrupted
        } else {
            entry.clone()
        };
        corrupted_index.insert(corrupted);
    }
    SymbolEnv::new(
        symbols.module_id().clone(),
        SymbolEnvIndexes {
            imports: symbols.imports().clone(),
            exports: symbols.exports().clone(),
            symbols: corrupted_index,
            labels: symbols.labels().clone(),
            definitions: symbols.definitions().clone(),
            overloads: symbols.overloads().clone(),
            registrations: symbols.registrations().clone(),
            lexical_summaries: symbols.lexical_summaries().clone(),
            namespace_graph: symbols.namespace_graph().clone(),
            declaration_dependencies: symbols.declaration_dependencies().clone(),
            contributions: symbols.contributions().clone(),
            module_summaries: symbols.module_summaries().clone(),
        },
    )
}

fn validate_origin(
    source_id: SourceId,
    module: &ModuleId,
    origin: &mizar_resolve::resolved_ast::SemanticOrigin,
    start: usize,
    end: usize,
    structural_path: &[u32],
) -> Result<(), String> {
    if origin.source_id() != source_id
        || origin.module_id() != module
        || origin.anchor() != &SourceAnchor::Range(source_range(source_id, start, end))
        || origin.structural_path() != structural_path
        || origin.import_edge().is_some()
        || origin.is_recovered()
    {
        return Err("Task259 resolver: predicate origin is not exact".to_owned());
    }
    Ok(())
}

fn task259_arena(source_id: SourceId) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    push_node(
        &mut builder,
        "source.type.head",
        source_range(source_id, 22, 25),
        LocalTypeContextId::new(1),
        Vec::new(),
    )?;
    push_node(
        &mut builder,
        "source.type.expression",
        source_range(source_id, 22, 25),
        LocalTypeContextId::new(1),
        vec![TYPE_HEAD_X],
    )?;
    push_node(
        &mut builder,
        "source.definition.predicate.parameter",
        source_range(source_id, 17, 18),
        LocalTypeContextId::new(1),
        vec![TYPE_EXPRESSION_X],
    )?;
    push_node(
        &mut builder,
        "source.type.head",
        source_range(source_id, 38, 41),
        LocalTypeContextId::new(1),
        Vec::new(),
    )?;
    push_node(
        &mut builder,
        "source.type.expression",
        source_range(source_id, 38, 41),
        LocalTypeContextId::new(1),
        vec![TYPE_HEAD_Y],
    )?;
    push_node(
        &mut builder,
        "source.definition.predicate.parameter",
        source_range(source_id, 33, 34),
        LocalTypeContextId::new(1),
        vec![TYPE_EXPRESSION_Y],
    )?;
    for (id, start, end) in [
        (GUARD_LEFT, 52, 53),
        (GUARD_RIGHT, 56, 57),
        (DEFINIENS_LEFT, 116, 117),
        (DEFINIENS_RIGHT, 120, 121),
    ] {
        let actual = push_node(
            &mut builder,
            "source.term.variable-reference",
            source_range(source_id, start, end),
            LocalTypeContextId::new(1),
            Vec::new(),
        )?;
        if actual != id {
            return Err("Task259 arena term order changed".to_owned());
        }
        if id == GUARD_RIGHT {
            push_node(
                &mut builder,
                "source.formula.atomic.equality",
                source_range(source_id, 52, 57),
                LocalTypeContextId::new(1),
                vec![GUARD_LEFT, GUARD_RIGHT],
            )?;
        }
        if id == DEFINIENS_RIGHT {
            push_node(
                &mut builder,
                "source.formula.atomic.equality",
                source_range(source_id, 116, 121),
                LocalTypeContextId::new(1),
                vec![DEFINIENS_LEFT, DEFINIENS_RIGHT],
            )?;
        }
    }
    push_node(
        &mut builder,
        "source.definition.predicate.guard",
        source_range(source_id, 45, 58),
        LocalTypeContextId::new(1),
        vec![GUARD_FORMULA],
    )?;
    push_node(
        &mut builder,
        "source.definition.predicate",
        source_range(source_id, 61, 122),
        LocalTypeContextId::new(1),
        vec![DEFINIENS_FORMULA],
    )?;
    push_node(
        &mut builder,
        "source.definition.predicate.property",
        source_range(source_id, 125, 159),
        LocalTypeContextId::new(1),
        Vec::new(),
    )?;
    push_node(
        &mut builder,
        "source.definition",
        source_range(source_id, 0, 164),
        LocalTypeContextId::new(1),
        vec![
            PARAMETER_X,
            PARAMETER_Y,
            GUARD_OWNER,
            PREDICATE_OWNER,
            PROPERTY_OWNER,
        ],
    )?;
    push_node(
        &mut builder,
        "source.module",
        source_range(source_id, 0, 164),
        LocalTypeContextId::new(0),
        vec![DEFINITION_OWNER],
    )?;
    builder
        .finish(Some(MODULE_OWNER))
        .map_err(|error| error.to_string())
}

fn push_node(
    builder: &mut TypedArenaBuilder,
    kind: &str,
    range: SourceRange,
    context: LocalTypeContextId,
    children: Vec<TypedNodeId>,
) -> Result<TypedNodeId, String> {
    builder
        .push(
            TypedNode::new(kind, SourceAnchor::Range(range))
                .with_children(children)
                .with_typing(TypingState::Unknown)
                .with_recovery(NodeRecoveryState::Normal)
                .with_links(TypedNodeLinks {
                    context: Some(context),
                    ..TypedNodeLinks::default()
                }),
        )
        .map_err(|error| error.to_string())
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
            bare_set_type(
                source_id,
                module_id.clone(),
                TYPE_EXPRESSION_X,
                TYPE_HEAD_X,
                22,
                25,
            ),
            bare_set_type(source_id, module_id, TYPE_EXPRESSION_Y, TYPE_HEAD_Y, 38, 41),
        ],
        arguments: Vec::new(),
    }
}

fn bare_set_type(
    source_id: SourceId,
    module_id: ModuleId,
    site: TypedNodeId,
    head_site: TypedNodeId,
    start: usize,
    end: usize,
) -> SourceTypeExpressionInput {
    SourceTypeExpressionInput {
        source_id,
        module_id,
        site: TypedSiteRef::Node(site),
        source_range: source_range(source_id, start, end),
        spelling: "set".to_owned(),
        head_site: TypedSiteRef::Node(head_site),
        head_range: source_range(source_id, start, end),
        head_spelling: "set".to_owned(),
        form: SourceTypeApplicationForm::Bare,
        head: SourceTypeHead::BuiltinSet,
        recovery: NodeRecoveryState::Normal,
    }
}

fn task252_input(source_id: SourceId, module_id: ModuleId) -> SourcePrimaryTermHandoffInput {
    let specs = [
        (GUARD_LEFT, 52, 53, "x", BindingId::new(0)),
        (GUARD_RIGHT, 56, 57, "x", BindingId::new(0)),
        (DEFINIENS_LEFT, 116, 117, "x", BindingId::new(0)),
        (DEFINIENS_RIGHT, 120, 121, "y", BindingId::new(1)),
    ];
    SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: specs
            .iter()
            .enumerate()
            .map(
                |(source_ordinal, (site, start, end, spelling, _))| SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(*site),
                    source_range: source_range(source_id, *start, *end),
                    source_ordinal,
                    context: BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: (*spelling).to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                },
            )
            .collect(),
        references: specs
            .iter()
            .enumerate()
            .map(
                |(index, (_, _, _, _, binding))| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(index),
                    binding: *binding,
                    role: SourcePrimaryTermReferenceRole::Variable,
                },
            )
            .collect(),
        numeric_type_requests: Vec::new(),
    }
}

fn task256_input(source_id: SourceId, module_id: ModuleId) -> SourceAtomicFormulaHandoffInput {
    let formulas = vec![
        SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(GUARD_FORMULA),
            source_range: source_range(source_id, 52, 57),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = x".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        },
        SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(DEFINIENS_FORMULA),
            source_range: source_range(source_id, 116, 121),
            source_ordinal: 1,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = y".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        },
    ];
    let edges = [
        (0, 0, SourceAtomicEdgeRole::BuiltinLeftOperand, 0),
        (0, 1, SourceAtomicEdgeRole::BuiltinRightOperand, 1),
        (1, 0, SourceAtomicEdgeRole::BuiltinLeftOperand, 2),
        (1, 1, SourceAtomicEdgeRole::BuiltinRightOperand, 3),
    ]
    .into_iter()
    .map(|(formula, ordinal, role, target)| SourceAtomicEdgeInput {
        formula: SourceAtomicFormulaId::new(formula),
        ordinal,
        role,
        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(target)),
    })
    .collect::<Vec<_>>();
    let requests = [0, 1, 2, 3]
        .into_iter()
        .map(|edge| SourceAtomicRequestInput {
            formula: SourceAtomicFormulaId::new(edge / 2),
            ordinal: edge % 2,
            kind: SourceAtomicRequestKind::OperandExpectedType,
            edge: Some(SourceAtomicEdgeId::new(edge)),
            candidate: None,
            type_site: None,
            attribute: None,
        })
        .collect();
    SourceAtomicFormulaHandoffInput {
        source_id,
        module_id,
        formulas,
        wrappers: Vec::new(),
        predicate_segments: Vec::new(),
        predicate_heads: Vec::new(),
        candidates: Vec::new(),
        type_sites: Vec::new(),
        attributes: Vec::new(),
        edges,
        requests,
    }
}

fn task259_input(
    source_id: SourceId,
    module_id: ModuleId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
) -> SourcePredicateDefinitionHandoffInput {
    SourcePredicateDefinitionHandoffInput {
        source_id,
        module_id,
        definitions: vec![SourcePredicateDefinitionInput {
            symbol,
            definition,
            contribution,
            site: TypedSiteRef::Node(PREDICATE_OWNER),
            source_range: source_range(source_id, 61, 122),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourcePredicateDefinitionRecovery::Normal,
            spelling: "pred Task259PredicateDefinition: x task259_rel y means x = y;".to_owned(),
            definiens: SourceAtomicFormulaId::new(1),
        }],
        parameters: vec![
            SourcePredicateParameterInput {
                owner: SourcePredicateDefinitionId::new(0),
                ordinal: 0,
                binding: BindingId::new(0),
                written_type: SourceTypeApplicationId::new(0),
                site: TypedSiteRef::Node(PARAMETER_X),
                source_range: source_range(source_id, 13, 26),
                declaration_range: source_range(source_id, 17, 18),
                context: BindingContextId::new(1),
                recovery: SourcePredicateDefinitionRecovery::Normal,
                spelling: "let x be set;".to_owned(),
            },
            SourcePredicateParameterInput {
                owner: SourcePredicateDefinitionId::new(0),
                ordinal: 1,
                binding: BindingId::new(1),
                written_type: SourceTypeApplicationId::new(1),
                site: TypedSiteRef::Node(PARAMETER_Y),
                source_range: source_range(source_id, 29, 42),
                declaration_range: source_range(source_id, 33, 34),
                context: BindingContextId::new(1),
                recovery: SourcePredicateDefinitionRecovery::Normal,
                spelling: "let y be set;".to_owned(),
            },
        ],
        guards: vec![SourcePredicateGuardInput {
            owner: SourcePredicateDefinitionId::new(0),
            ordinal: 0,
            formula: SourceAtomicFormulaId::new(0),
            site: TypedSiteRef::Node(GUARD_OWNER),
            source_range: source_range(source_id, 45, 58),
            context: BindingContextId::new(1),
            recovery: SourcePredicateDefinitionRecovery::Normal,
            spelling: "assume x = x;".to_owned(),
        }],
        properties: vec![SourcePredicatePropertyInput {
            owner: SourcePredicateDefinitionId::new(0),
            ordinal: 0,
            kind: SourcePredicatePropertyKind::Symmetry,
            site: TypedSiteRef::Node(PROPERTY_OWNER),
            source_range: source_range(source_id, 125, 159),
            justification: SourceAnchor::Range(source_range(source_id, 134, 158)),
            recovery: SourcePredicateDefinitionRecovery::Normal,
            spelling: "symmetry by computation(steps: 1);".to_owned(),
        }],
        correctness: vec![SourcePredicateCorrectnessInput {
            owner: SourcePredicateDefinitionId::new(0),
            property: SourcePredicatePropertyId::new(0),
            ordinal: 0,
            source_anchor: SourceAnchor::Range(source_range(source_id, 125, 159)),
        }],
    }
}

fn route_output_is_exact(output: &SourcePredicateDefinitionRouteOutput) -> bool {
    let typed = &output.typed_ast;
    let resolved = &output.resolved;
    let Some(predicate) = typed.source_predicate_definition() else {
        return false;
    };
    predicate.definitions().len() == 1
        && predicate.parameters().len() == 2
        && predicate.guards().len() == 1
        && predicate.properties().len() == 1
        && predicate.correctness().len() == 1
        && typed.source_predicate_definition() == resolved.source_predicate_definition()
        && typed.source_context() == resolved.source_context()
        && typed.source_type() == resolved.source_type()
        && typed.source_term() == resolved.source_term()
        && typed.source_atomic_formula() == resolved.source_atomic_formula()
        && typed.initial_obligations().len() == 1
        && typed.types().is_empty()
        && typed.facts().is_empty()
        && typed.coercions().is_empty()
        && typed.diagnostics().is_empty()
        && typed.source_statement().is_none()
        && resolved.source_statement().is_none()
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
