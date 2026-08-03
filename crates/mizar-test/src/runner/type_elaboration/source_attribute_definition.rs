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
    source_attribute_definition::{
        SourceAttributeDefiniensId, SourceAttributeDefiniensInput,
        SourceAttributeDefinitionHandoffInput, SourceAttributeDefinitionId,
        SourceAttributeDefinitionInput, SourceAttributeDefinitionProducer,
        SourceAttributeDefinitionRecovery, SourceAttributeParameterInput, SourceAttributeSubjectId,
        SourceAttributeSubjectInput,
    },
    source_context::{
        SourceBindingContextBuild, SourceBindingContextInput, SourceBindingContextOwner,
        SourceBindingContextProducer, SourceBindingSiteInput, SourceBindingSiteRole,
        SourceItemInput, SourceItemRecovery, SourceItemRole, SourceItemVisibility,
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
        ContributionKind, DefinitionId, DefinitionIndex, DefinitionKind, DefinitionShell,
        ExportStatus, NamespacePath, SourceContributionId, SourceContributionIndex, SymbolEntry,
        SymbolEnv, SymbolEnvIndexes, SymbolIndex, SymbolKind, Visibility,
    },
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{ModuleId, SymbolId},
    symbols::{SignatureProjectionExtractor, SymbolOverloadPolicy},
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNodeId, SurfaceNodeKind, SurfaceTokenKind};

use super::checker_handoff::assemble_empty_resolved_typed_ast;

pub(in crate::runner) const SOURCE_ATTRIBUTE_DEFINITION_TEXT: &str = concat!(
    "definition\n",
    "  let x be set;\n",
    "  let y be set;\n",
    "  attr Task261AttributeDefinition: x is task261_marked means x = y;\n",
    "end;\n",
);

const INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.source_attribute_definition.invalid_payload";

#[derive(Debug)]
pub(in crate::runner) struct SourceAttributeDefinitionRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; variants are private corruption seams.
pub(in crate::runner) enum SourceAttributeDefinitionRouteMutation {
    None,
    RemoveResolverShell,
    WrongResolverProjection,
    WrongResolverSymbolEntry,
    WrongResolverDefinitionEntry,
    WrongResolverContribution,
    WrongContextModuleSite,
    WrongContextItemSite,
    StaleContextItemSite,
    WrongContextBindingSite(usize),
    WrongContextBindingOwner(usize),
    RemoveTypeExpression,
    WrongTypeApplicationBinding(usize),
    WrongTypeApplicationRoot(usize),
    WrongTypeExpressionSite(usize),
    WrongTermBinding(usize),
    WrongTermSite(usize),
    RemoveAtomicFormula,
    RemoveAtomicEdge,
    WrongAtomicFormula,
    WrongAtomicEdge(usize),
    WrongAtomicRequest(usize),
    RemoveAttributeParameter,
    RemoveAttributeSubject,
    WrongAttributeParameterOwner,
    WrongAttributeSubjectBinding,
    WrongAttributeDefiniensFormula,
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
    token_row!(ReservedWord, "attr", 45, 49),
    token_row!(Identifier, "Task261AttributeDefinition", 50, 76),
    token_row!(ReservedSymbol, ":", 76, 77),
    token_row!(Identifier, "x", 78, 79),
    token_row!(ReservedWord, "is", 80, 82),
    token_row!(Identifier, "task261_marked", 83, 97),
    token_row!(ReservedWord, "means", 98, 103),
    token_row!(Identifier, "x", 104, 105),
    token_row!(ReservedSymbol, "=", 106, 107),
    token_row!(Identifier, "y", 108, 109),
    token_row!(ReservedSymbol, ";", 109, 110),
    token_row!(ReservedWord, "end", 111, 114),
    token_row!(ReservedSymbol, ";", 114, 115),
    structural_row!(TypeHead, 22, 25, [4]),
    structural_row!(TypeExpression, 22, 25, [24]),
    structural_row!(QualifiedVariableSegment, 17, 25, [2, 3, 25]),
    structural_row!(DefinitionParameter, 13, 26, [1, 26, 5]),
    structural_row!(TypeHead, 38, 41, [9]),
    structural_row!(TypeExpression, 38, 41, [28]),
    structural_row!(QualifiedVariableSegment, 33, 41, [7, 8, 29]),
    structural_row!(DefinitionParameter, 29, 42, [6, 30, 10]),
    structural_row!(AttributePattern, 83, 97, [16]),
    structural_row!(TermReference, 104, 105, [18]),
    structural_row!(TermExpression, 104, 105, [33]),
    structural_row!(TermReference, 108, 109, [20]),
    structural_row!(TermExpression, 108, 109, [35]),
    structural_row!(BuiltinPredicateApplication, 104, 109, [34, 19, 36]),
    structural_row!(FormulaExpression, 104, 109, [37]),
    structural_row!(FormulaDefiniens, 104, 109, [38]),
    structural_row!(
        AttributeDefinition,
        45,
        110,
        [11, 12, 13, 14, 15, 32, 17, 39, 21]
    ),
    structural_row!(DefinitionBlockItem, 0, 115, [0, 27, 31, 40, 22, 23]),
    structural_row!(ItemList, 0, 115, [41]),
    structural_row!(CompilationUnit, 0, 115, [42]),
    structural_row!(
        Root,
        0,
        115,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            43
        ]
    ),
];

#[derive(Debug, Clone, Copy)]
struct ExactAttributeDefinition {
    definition_block: SurfaceNodeId,
}

pub(in crate::runner) fn source_attribute_definition_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_attribute_definition_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceAttributeDefinitionRouteMutation::None,
    ) {
        None => None,
        Some(Ok(output)) if route_output_is_exact(&output) => Some(Vec::new()),
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_attribute_definition_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceAttributeDefinitionRouteOutput, String>> {
    source_attribute_definition_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceAttributeDefinitionRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_attribute_definition_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceAttributeDefinitionRouteMutation,
) -> Option<Result<SourceAttributeDefinitionRouteOutput, String>> {
    source_attribute_definition_output_impl(ast, module, shells, symbols, source_text, mutation)
}

fn source_attribute_definition_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceAttributeDefinitionRouteMutation,
) -> Option<Result<SourceAttributeDefinitionRouteOutput, String>> {
    let exact = exact_attribute_definition(ast, source_text)?;
    Some(build_output(ast, module, shells, symbols, exact, mutation))
}

fn exact_attribute_definition(
    ast: &SurfaceAst,
    source_text: &str,
) -> Option<ExactAttributeDefinition> {
    if source_text != SOURCE_ATTRIBUTE_DEFINITION_TEXT
        || source_text.len() != 116
        || !source_text.ends_with('\n')
        || source_text.ends_with("\n\n")
        || ast.root().map(SurfaceNodeId::index) != Some(44)
        || ast.expression_root().is_some()
        || !surface_profile_is_exact(ast)
    {
        return None;
    }
    let root = exact_node(ast, 44, SurfaceNodeKind::Root, 0, 115)?;
    let definition_block = exact_node(ast, 41, SurfaceNodeKind::DefinitionBlockItem, 0, 115)?;
    let attribute = exact_node(ast, 40, SurfaceNodeKind::AttributeDefinition, 45, 110)?;
    let pattern = exact_node(ast, 32, SurfaceNodeKind::AttributePattern, 83, 97)?;
    let formula = exact_node(
        ast,
        37,
        SurfaceNodeKind::BuiltinPredicateApplication,
        104,
        109,
    )?;
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
    if direct_structural != [27, 31, 40]
        || !is_descendant(ast, root, definition_block)
        || !is_descendant(ast, attribute, pattern)
        || !is_descendant(ast, attribute, formula)
        || is_descendant(ast, pattern, formula)
    {
        return None;
    }
    Some(ExactAttributeDefinition { definition_block })
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
    mutation: SourceAttributeDefinitionRouteMutation,
) -> Result<(SymbolId, DefinitionId, SourceContributionId), String> {
    let declarations = if mutation == SourceAttributeDefinitionRouteMutation::RemoveResolverShell
        && !shells.declarations().is_empty()
    {
        &shells.declarations()[..shells.declarations().len() - 1]
    } else {
        shells.declarations()
    };
    let [block, attribute] = declarations else {
        return Err("Task261 resolver: expected two declaration shells".to_owned());
    };
    if !shells.exports().is_empty()
        || block.id().index() != 0
        || block.ordinal() != 0
        || block.kind() != DeclarationShellKind::DefinitionBlock
        || block.node_id().index() != 41
        || block.parent().is_some()
        || block.range() != source_range(ast.source_id, 0, 115)
        || block.module() != module
        || block.recovered()
        || attribute.id().index() != 1
        || attribute.ordinal() != 1
        || attribute.kind() != DeclarationShellKind::AttributeDefinition
        || attribute.node_id().index() != 40
        || attribute.parent() != Some(block.id())
        || attribute.range() != source_range(ast.source_id, 45, 110)
        || attribute.module() != module
        || attribute.recovered()
        || symbols.module_id() != module
        || symbols.symbols().len() != 1
        || symbols.definitions().len() != 1
        || symbols.contributions().len() != 1
    {
        return Err("Task261 resolver: raw profile is not exact".to_owned());
    }

    let mut projections =
        SignatureProjectionExtractor::new(ast, shells, NamespacePath::new(module.path().as_str()))
            .extract();
    if mutation == SourceAttributeDefinitionRouteMutation::WrongResolverProjection {
        projections[0] = projections[0]
            .clone()
            .with_definition_kind(DefinitionKind::Predicate);
    }
    let [projection] = projections.as_slice() else {
        return Err("Task261 resolver: expected one parser-backed projection".to_owned());
    };
    if projection.primary_spelling() != "task261_marked"
        || projection.notation_spelling() != Some("task261_marked")
        || projection.symbol_kind() != SymbolKind::Attribute
        || projection.definition_kind() != Some(DefinitionKind::Attribute)
        || projection.arity().is_some()
        || projection.overload_policy() != SymbolOverloadPolicy::Overloadable
        || projection.signature().is_none()
    {
        return Err("Task261 resolver: parser-backed projection is not exact".to_owned());
    }

    let definition = symbols
        .definitions()
        .iter()
        .find(|definition| definition.id().index() == 0)
        .ok_or_else(|| "Task261 resolver: definition disappeared".to_owned())?;
    let symbol = symbols
        .symbols()
        .get(definition.symbol())
        .ok_or_else(|| "Task261 resolver: symbol disappeared".to_owned())?;
    if definition.kind() != DefinitionKind::Attribute
        || definition.visibility() != Visibility::Public
        || !definition.parameters().is_empty()
        || !definition.binders().is_empty()
        || definition.arity().is_some()
        || definition.notation_shape() != Some("task261_marked")
        || definition.conflict().is_some()
        || definition.signature() != symbol.signature()
        || symbol.kind() != SymbolKind::Attribute
        || symbol.visibility() != Visibility::Public
        || symbol.export_status() != ExportStatus::Exported
        || symbol.primary_spelling() != "task261_marked"
        || symbol.notation_spelling() != Some("task261_marked")
        || symbol.contribution() != definition.contribution()
    {
        return Err("Task261 resolver: attribute definition is not exact".to_owned());
    }
    validate_origin(ast.source_id, module, definition.origin())?;
    validate_origin(ast.source_id, module, symbol.origin())?;
    let contribution = symbols
        .contributions()
        .get(definition.contribution())
        .ok_or_else(|| "Task261 resolver: local contribution disappeared".to_owned())?;
    if contribution.module() != module
        || !matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == ast.source_id
        )
        || contribution.effects().symbols() != [symbol.symbol().clone()]
        || contribution.effects().definitions() != [definition.id()]
    {
        return Err("Task261 resolver: local contribution is not exact".to_owned());
    }
    Ok((
        symbol.symbol().clone(),
        definition.id(),
        definition.contribution(),
    ))
}

fn resolver_env_with_mutation(
    symbols: &SymbolEnv,
    mutation: SourceAttributeDefinitionRouteMutation,
) -> SymbolEnv {
    let mut rebuilt_symbols = SymbolIndex::new();
    for entry in symbols.symbols().iter() {
        let primary_spelling =
            if mutation == SourceAttributeDefinitionRouteMutation::WrongResolverSymbolEntry {
                "task261_corrupted"
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
        if mutation == SourceAttributeDefinitionRouteMutation::WrongResolverDefinitionEntry {
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
                if let Some(arity) = original.arity() {
                    rebuilt = rebuilt.with_arity(arity);
                }
                if let Some(notation) = original.notation_shape() {
                    rebuilt = rebuilt.with_notation_shape(notation);
                }
                if let Some(doc) = original.doc_attachment() {
                    rebuilt = rebuilt.with_doc_attachment(doc.clone());
                }
                if let Some(conflict) = original.conflict() {
                    rebuilt = rebuilt.with_conflict(conflict.clone());
                }
                if let Some(signature) = original.signature() {
                    rebuilt = rebuilt.with_signature(signature.clone());
                }
                let id = definitions.insert(rebuilt);
                debug_assert_eq!(id, original.id());
            }
            definitions
        } else {
            symbols.definitions().clone()
        };

    let rebuilt_contributions =
        if mutation == SourceAttributeDefinitionRouteMutation::WrongResolverContribution {
            let mut contributions = SourceContributionIndex::new();
            for original in symbols.contributions().iter() {
                let id = contributions.insert(
                    original.module().clone(),
                    original.kind().clone(),
                    original.anchor().clone(),
                );
                debug_assert_eq!(id, original.id());
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
        || origin.anchor() != &SourceAnchor::Range(source_range(source_id, 45, 110))
        || origin.structural_path() != [4, 0, 7, 0]
        || origin.import_edge().is_some()
        || origin.is_recovered()
    {
        return Err("Task261 resolver: attribute origin is not exact".to_owned());
    }
    Ok(())
}

fn task261_arena(source_id: SourceId) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    for (index, row) in EXACT_SURFACE_PROFILE.iter().enumerate() {
        let generic_kind = match row.kind {
            ExactSurfaceKind::Token(_, _) => "source.surface.token",
            ExactSurfaceKind::Structural(_) => "source.surface.structural",
        };
        let (kind, start, end) = match index {
            24 => ("source.type.head", 22, 25),
            25 => ("source.type.expression", 22, 25),
            27 => ("source.definition.attribute.parameter", 17, 18),
            28 => ("source.type.head", 38, 41),
            29 => ("source.type.expression", 38, 41),
            31 => ("source.definition.attribute.parameter", 33, 34),
            33 => ("source.term.variable-reference", 104, 105),
            35 => ("source.term.variable-reference", 108, 109),
            37 => ("source.formula.atomic.equality", 104, 109),
            39 => ("source.definition.attribute.definiens", 104, 109),
            40 => ("source.definition.attribute", 45, 110),
            41 => ("source.definition", 0, 115),
            44 => ("source.module", 0, 115),
            _ => (generic_kind, row.start, row.end),
        };
        let context = if index == 44 {
            LocalTypeContextId::new(0)
        } else {
            LocalTypeContextId::new(1)
        };
        let id = builder
            .push(
                TypedNode::new(
                    kind,
                    SourceAnchor::Range(source_range(source_id, start, end)),
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
        if id.index() != index {
            return Err("Task261 arena order changed".to_owned());
        }
    }
    builder
        .finish(Some(TypedNodeId::new(44)))
        .map_err(|error| error.to_string())
}

fn task248_context(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    exact: ExactAttributeDefinition,
    mutation: SourceAttributeDefinitionRouteMutation,
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
    let module_site = if mutation == SourceAttributeDefinitionRouteMutation::WrongContextModuleSite
    {
        TypedSiteRef::Node(TypedNodeId::new(41))
    } else {
        TypedSiteRef::Node(TypedNodeId::new(44))
    };
    let item_site = match mutation {
        SourceAttributeDefinitionRouteMutation::WrongContextItemSite => {
            TypedSiteRef::Node(TypedNodeId::new(44))
        }
        SourceAttributeDefinitionRouteMutation::StaleContextItemSite => {
            TypedSiteRef::Node(TypedNodeId::new(42))
        }
        _ => TypedSiteRef::Node(TypedNodeId::new(41)),
    };
    let binding_site = |index: usize| {
        if mutation == SourceAttributeDefinitionRouteMutation::WrongContextBindingSite(index) {
            TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 31 } else { 27 }))
        } else {
            TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 27 } else { 31 }))
        }
    };
    let context_owner = |index: usize| {
        if mutation == SourceAttributeDefinitionRouteMutation::WrongContextBindingOwner(index) {
            SourceBindingContextOwner::Module
        } else {
            SourceBindingContextOwner::Shell(shell)
        }
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
            source_range: source_range(ast.source_id, 0, 115),
            parent: None,
            visibility: SourceItemVisibility::Unspecified,
            site: item_site,
            local_scope: Some(scope.clone()),
            recovery: SourceItemRecovery::Normal,
        }],
        bindings: vec![
            SourceBindingSiteInput {
                shell,
                context_owner: context_owner(0),
                source_ordinal: 0,
                spelling: "x".to_owned(),
                declaration_range: source_range(ast.source_id, 17, 18),
                written_type_range: source_range(ast.source_id, 22, 25),
                site: binding_site(0),
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
                context_owner: context_owner(1),
                source_ordinal: 1,
                spelling: "y".to_owned(),
                declaration_range: source_range(ast.source_id, 33, 34),
                written_type_range: source_range(ast.source_id, 38, 41),
                site: binding_site(1),
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
            bare_set_type(source_id, module_id.clone(), 25, 24, 22, 25),
            bare_set_type(source_id, module_id, 29, 28, 38, 41),
        ],
        arguments: Vec::new(),
    }
}

fn task252_input(source_id: SourceId, module_id: ModuleId) -> SourcePrimaryTermHandoffInput {
    let specs = [
        (33, 104, 105, "x", BindingId::new(0)),
        (35, 108, 109, "y", BindingId::new(1)),
    ];
    SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: specs
            .iter()
            .enumerate()
            .map(
                |(source_ordinal, (site, start, end, spelling, _))| SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(*site)),
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
    SourceAtomicFormulaHandoffInput {
        source_id,
        module_id,
        formulas: vec![SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(TypedNodeId::new(37)),
            source_range: source_range(source_id, 104, 109),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = y".to_owned(),
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
    }
}

fn task261_input(
    source_id: SourceId,
    module_id: ModuleId,
    resolver: (SymbolId, DefinitionId, SourceContributionId),
) -> SourceAttributeDefinitionHandoffInput {
    SourceAttributeDefinitionHandoffInput {
        source_id,
        module_id,
        definitions: vec![SourceAttributeDefinitionInput {
            symbol: resolver.0,
            definition: resolver.1,
            contribution: resolver.2,
            site: TypedSiteRef::Node(TypedNodeId::new(40)),
            source_range: source_range(source_id, 45, 110),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceAttributeDefinitionRecovery::Normal,
            spelling: "attr Task261AttributeDefinition: x is task261_marked means x = y;"
                .to_owned(),
            subject: SourceAttributeSubjectId::new(0),
            definiens: SourceAttributeDefiniensId::new(0),
        }],
        parameters: vec![
            SourceAttributeParameterInput {
                owner: SourceAttributeDefinitionId::new(0),
                ordinal: 0,
                binding: BindingId::new(0),
                written_type: SourceTypeApplicationId::new(0),
                site: TypedSiteRef::Node(TypedNodeId::new(27)),
                source_range: source_range(source_id, 13, 26),
                declaration_range: source_range(source_id, 17, 18),
                context: BindingContextId::new(1),
                recovery: SourceAttributeDefinitionRecovery::Normal,
                spelling: "let x be set;".to_owned(),
            },
            SourceAttributeParameterInput {
                owner: SourceAttributeDefinitionId::new(0),
                ordinal: 1,
                binding: BindingId::new(1),
                written_type: SourceTypeApplicationId::new(1),
                site: TypedSiteRef::Node(TypedNodeId::new(31)),
                source_range: source_range(source_id, 29, 42),
                declaration_range: source_range(source_id, 33, 34),
                context: BindingContextId::new(1),
                recovery: SourceAttributeDefinitionRecovery::Normal,
                spelling: "let y be set;".to_owned(),
            },
        ],
        subjects: vec![SourceAttributeSubjectInput {
            owner: SourceAttributeDefinitionId::new(0),
            binding: BindingId::new(0),
            site: TypedSiteRef::Node(TypedNodeId::new(40)),
            source_range: source_range(source_id, 78, 79),
            context: BindingContextId::new(1),
            recovery: SourceAttributeDefinitionRecovery::Normal,
            spelling: "x".to_owned(),
        }],
        definientia: vec![SourceAttributeDefiniensInput {
            owner: SourceAttributeDefinitionId::new(0),
            ordinal: 0,
            formula: SourceAtomicFormulaId::new(0),
            site: TypedSiteRef::Node(TypedNodeId::new(39)),
            source_range: source_range(source_id, 104, 109),
            context: BindingContextId::new(1),
            recovery: SourceAttributeDefinitionRecovery::Normal,
            spelling: "x = y".to_owned(),
        }],
    }
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    exact: ExactAttributeDefinition,
    mutation: SourceAttributeDefinitionRouteMutation,
) -> Result<SourceAttributeDefinitionRouteOutput, String> {
    let corrupted_symbols = matches!(
        mutation,
        SourceAttributeDefinitionRouteMutation::WrongResolverSymbolEntry
            | SourceAttributeDefinitionRouteMutation::WrongResolverDefinitionEntry
            | SourceAttributeDefinitionRouteMutation::WrongResolverContribution
    )
    .then(|| resolver_env_with_mutation(symbols, mutation));
    let symbols = corrupted_symbols.as_ref().unwrap_or(symbols);
    let resolver = exact_resolver_profile(ast, &module, shells, symbols, mutation)?;
    let arena = task261_arena(ast.source_id)?;
    let source_context = task248_context(ast, module.clone(), shells, exact, mutation)?;

    let mut type_input = task249_input(ast.source_id, module.clone());
    if mutation == SourceAttributeDefinitionRouteMutation::RemoveTypeExpression {
        type_input.expressions.pop();
    }
    if let SourceAttributeDefinitionRouteMutation::WrongTypeApplicationBinding(index) = mutation {
        type_input.applications[index].binding = BindingId::new(1 - index);
    }
    if let SourceAttributeDefinitionRouteMutation::WrongTypeApplicationRoot(index) = mutation {
        type_input.applications[index].root = SourceTypeExpressionId::new(1 - index);
    }
    if let SourceAttributeDefinitionRouteMutation::WrongTypeExpressionSite(index) = mutation {
        type_input.expressions[index].site =
            TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 29 } else { 25 }));
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
    if let SourceAttributeDefinitionRouteMutation::WrongTermBinding(index) = mutation {
        term_input.references[index].binding = BindingId::new(1 - index);
    }
    if let SourceAttributeDefinitionRouteMutation::WrongTermSite(index) = mutation {
        term_input.terms[index].site =
            TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 35 } else { 33 }));
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
    if mutation == SourceAttributeDefinitionRouteMutation::RemoveAtomicFormula {
        atomic_input.formulas.clear();
    }
    if mutation == SourceAttributeDefinitionRouteMutation::RemoveAtomicEdge {
        atomic_input.edges.pop();
    }
    if mutation == SourceAttributeDefinitionRouteMutation::WrongAtomicFormula {
        atomic_input.formulas[0].site = TypedSiteRef::Node(TypedNodeId::new(40));
    }
    if let SourceAttributeDefinitionRouteMutation::WrongAtomicEdge(index) = mutation {
        atomic_input.edges[index].target =
            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(99));
    }
    if let SourceAttributeDefinitionRouteMutation::WrongAtomicRequest(index) = mutation {
        atomic_input.requests[index].edge = Some(SourceAtomicEdgeId::new(99));
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

    let obligations_before = typed_ast.initial_obligations().clone();
    let mut attribute_input = task261_input(ast.source_id, module, resolver);
    if mutation == SourceAttributeDefinitionRouteMutation::RemoveAttributeParameter {
        attribute_input.parameters.pop();
    }
    if mutation == SourceAttributeDefinitionRouteMutation::RemoveAttributeSubject {
        attribute_input.subjects.clear();
    }
    if mutation == SourceAttributeDefinitionRouteMutation::WrongAttributeParameterOwner {
        attribute_input.parameters[0].owner = SourceAttributeDefinitionId::new(1);
    }
    if mutation == SourceAttributeDefinitionRouteMutation::WrongAttributeSubjectBinding {
        attribute_input.subjects[0].binding = BindingId::new(1);
    }
    if mutation == SourceAttributeDefinitionRouteMutation::WrongAttributeDefiniensFormula {
        attribute_input.definientia[0].formula = SourceAtomicFormulaId::new(1);
    }
    let attribute = SourceAttributeDefinitionProducer::build(
        attribute_input,
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
        typed_ast.nodes(),
    )
    .map_err(|error| format!("Task261 attribute definition: {error}"))?;
    typed_ast = typed_ast
        .with_source_attribute_definition(attribute)
        .map_err(|error| format!("Task261 typed installation: {error}"))?;
    if typed_ast.initial_obligations() != &obligations_before {
        return Err("Task261 typed installation changed initial obligations".to_owned());
    }

    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.definition.attribute"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)
        .map_err(|error| format!("Task261 final assembly: {error}"))?;
    Ok(SourceAttributeDefinitionRouteOutput {
        typed_ast,
        resolved,
    })
}

fn route_output_is_exact(output: &SourceAttributeDefinitionRouteOutput) -> bool {
    let typed = &output.typed_ast;
    let resolved = &output.resolved;
    let Some(context) = typed.source_context() else {
        return false;
    };
    let Some(source_type) = typed.source_type() else {
        return false;
    };
    let Some(source_term) = typed.source_term() else {
        return false;
    };
    let Some(atomic) = typed.source_atomic_formula() else {
        return false;
    };
    let Some(attribute) = typed.source_attribute_definition() else {
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
        && source_type.expressions().len() == 2
        && source_type.arguments().is_empty()
        && source_type.definition_returns().is_empty()
        && source_term.terms().len() == 2
        && source_term.references().len() == 2
        && source_term.numeric_type_requests().is_empty()
        && atomic.formulas().len() == 1
        && atomic.wrappers().is_empty()
        && atomic.predicate_segments().is_empty()
        && atomic.predicate_heads().is_empty()
        && atomic.candidates().is_empty()
        && atomic.type_sites().is_empty()
        && atomic.attributes().is_empty()
        && atomic.edges().len() == 2
        && atomic.requests().len() == 2
        && attribute.definitions().len() == 1
        && attribute.parameters().len() == 2
        && attribute.subjects().len() == 1
        && attribute.definientia().len() == 1
        && typed.source_attribute_definition() == resolved.source_attribute_definition()
        && typed.source_context() == resolved.source_context()
        && typed.source_type() == resolved.source_type()
        && typed.source_term() == resolved.source_term()
        && typed.source_atomic_formula() == resolved.source_atomic_formula()
        && typed.initial_obligations().is_empty()
        && typed.types().is_empty()
        && typed.facts().is_empty()
        && typed.coercions().is_empty()
        && typed.diagnostics().is_empty()
        && typed.source_predicate_definition().is_none()
        && typed.source_functor_definition().is_none()
        && resolved.source_predicate_definition().is_none()
        && resolved.source_functor_definition().is_none()
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
