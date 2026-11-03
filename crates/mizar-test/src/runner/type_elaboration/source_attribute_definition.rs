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
        CoercionTable, InitialObligationTable, LocalTypeContextId, NodeRecoveryState, SourceNode,
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
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNodeId, SurfaceNodeKind, SurfaceToken, SurfaceTokenKind};
use std::collections::BTreeSet;

use super::checker_handoff::{assemble_empty_resolved_typed_ast, source_syntax_receiver};

#[cfg(test)]
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

macro_rules! token_row {
    ($kind:ident, $text:literal) => {
        (SurfaceTokenKind::$kind, $text)
    };
}

const EXACT_ATTRIBUTE_TERMINALS: &[(SurfaceTokenKind, &str)] = &[
    token_row!(ReservedWord, "definition"),
    token_row!(ReservedWord, "let"),
    token_row!(Identifier, "x"),
    token_row!(ReservedWord, "be"),
    token_row!(ReservedWord, "set"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "let"),
    token_row!(Identifier, "y"),
    token_row!(ReservedWord, "be"),
    token_row!(ReservedWord, "set"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "attr"),
    token_row!(Identifier, "Task261AttributeDefinition"),
    token_row!(ReservedSymbol, ":"),
    token_row!(Identifier, "x"),
    token_row!(ReservedWord, "is"),
    token_row!(Identifier, "task261_marked"),
    token_row!(ReservedWord, "means"),
    token_row!(Identifier, "x"),
    token_row!(ReservedSymbol, "="),
    token_row!(Identifier, "y"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "end"),
    token_row!(ReservedSymbol, ";"),
];

#[derive(Debug, Clone, Copy)]
struct ExactAttributeDefinition {
    definition_block: SurfaceNodeId,
}

fn source_row(syntax: &TypedAst, index: usize) -> Result<&SourceNode, String> {
    syntax
        .source_node_entries()
        .and_then(|mut rows| rows.find(|row| row.site().index() == index))
        .ok_or_else(|| mizar_checker::typed_ast::TypedAstError::InvalidSourceStatement.to_string())
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
    let exact = exact_attribute_definition(ast, &module)?;
    Some(build_output(
        ast,
        module,
        shells,
        symbols,
        source_text,
        exact,
        mutation,
    ))
}

fn exact_attribute_definition(
    ast: &SurfaceAst,
    module: &ModuleId,
) -> Option<(ExactAttributeDefinition, TypedAst)> {
    use SurfaceNodeKind as K;
    if ast.root().map(SurfaceNodeId::index) != Some(44) || ast.expression_root().is_some() {
        return None;
    }
    ast.canonical_bytes()?;
    let syntax = source_syntax_receiver(ast, module).ok()?;
    let row_count = syntax.source_node_entries()?.len();
    let mut seen = BTreeSet::new();
    let observe = |id: TypedNodeId, expected: SurfaceNodeKind, seen: &mut BTreeSet<usize>| {
        let row = source_row(&syntax, id.index()).ok()?;
        let node = syntax.nodes().node(id)?;
        if row.kind() != &expected
            || row.range().source_id != syntax.source_id()
            || row.range().start >= row.range().end
            || row.recovery() != NodeRecoveryState::Normal
            || !seen.insert(id.index())
        {
            return None;
        }
        let children = &node.children;
        if children.is_empty() != matches!(&expected, K::Token(_)) {
            return None;
        }
        if !children.is_empty() {
            let first = source_row(&syntax, children.first()?.index()).ok()?;
            let last = source_row(&syntax, children.last()?.index()).ok()?;
            if row.range().start != first.range().start || row.range().end != last.range().end {
                return None;
            }
        }
        let mut previous_end = row.range().start;
        for (ordinal, child_id) in children.iter().enumerate() {
            let child = source_row(&syntax, child_id.index()).ok()?;
            let root_alias = expected == K::Root && matches!(child.kind(), K::Token(_));
            if child.range().source_id != row.range().source_id
                || child.range().start < row.range().start
                || child.range().end > row.range().end
                || (!root_alias
                    && (child.parent() != Some(row.site()) || child.child_ordinal() != ordinal))
                || (expected != K::Root && previous_end > child.range().start)
            {
                return None;
            }
            previous_end = child.range().end;
        }
        Some(children)
    };
    let one = |id, kind, seen: &mut BTreeSet<usize>| {
        let [child] = observe(id, kind, seen)?.as_slice() else {
            return None;
        };
        Some(*child)
    };
    // The fixed lower schema uses the Frontend's token-first, structural postorder IDs.
    let mut structural = EXACT_ATTRIBUTE_TERMINALS.len();
    let finish = |ids: &[TypedNodeId], next: &mut usize| {
        for id in ids {
            if id.index() != *next {
                return None;
            }
            *next += 1;
        }
        Some(())
    };
    let mut next = 0;
    let terms = |ids: &[TypedNodeId], next: &mut usize, seen: &mut BTreeSet<usize>| {
        for id in ids {
            if id.index() != *next {
                return None;
            }
            let (kind, text) = EXACT_ATTRIBUTE_TERMINALS.get(*next)?;
            observe(*id, K::Token(SurfaceToken::new(*kind, *text)), seen)?;
            *next += 1;
        }
        Some(())
    };
    let root_children = observe(syntax.nodes().root()?, K::Root, &mut seen)?;
    let (unit, aliases) = root_children.split_last()?;
    if !aliases
        .iter()
        .map(|id| id.index())
        .eq(ast.token_nodes().iter().map(|id| id.index()))
    {
        return None;
    }
    let list = one(*unit, K::CompilationUnit, &mut seen)?;
    let block = one(list, K::ItemList, &mut seen)?;
    let [open, p0, p1, attribute, end, block_semi] =
        observe(block, K::DefinitionBlockItem, &mut seen)?.as_slice()
    else {
        return None;
    };
    terms(&[*open], &mut next, &mut seen)?;
    for parameter in [*p0, *p1] {
        let [let_kw, segment, semi] =
            observe(parameter, K::DefinitionParameter, &mut seen)?.as_slice()
        else {
            return None;
        };
        terms(&[*let_kw], &mut next, &mut seen)?;
        let [name, be, expression] =
            observe(*segment, K::QualifiedVariableSegment, &mut seen)?.as_slice()
        else {
            return None;
        };
        terms(&[*name, *be], &mut next, &mut seen)?;
        let head = one(*expression, K::TypeExpression, &mut seen)?;
        let set = one(head, K::TypeHead, &mut seen)?;
        terms(&[set, *semi], &mut next, &mut seen)?;
        finish(&[head, *expression, *segment, parameter], &mut structural)?;
    }
    let [
        kw,
        label,
        colon,
        subject,
        is_kw,
        pattern,
        means,
        definiens,
        semi,
    ] = observe(*attribute, K::AttributeDefinition, &mut seen)?.as_slice()
    else {
        return None;
    };
    terms(
        &[*kw, *label, *colon, *subject, *is_kw],
        &mut next,
        &mut seen,
    )?;
    let name = one(*pattern, K::AttributePattern, &mut seen)?;
    terms(&[name, *means], &mut next, &mut seen)?;
    finish(&[*pattern], &mut structural)?;
    let formula = one(*definiens, K::FormulaDefiniens, &mut seen)?;
    let equality = one(formula, K::FormulaExpression, &mut seen)?;
    let [left, equals, right] =
        observe(equality, K::BuiltinPredicateApplication, &mut seen)?.as_slice()
    else {
        return None;
    };
    for (ordinal, expression) in [*left, *right].into_iter().enumerate() {
        if ordinal == 1 {
            terms(&[*equals], &mut next, &mut seen)?;
        }
        let reference = one(expression, K::TermExpression, &mut seen)?;
        let token = one(reference, K::TermReference, &mut seen)?;
        terms(&[token], &mut next, &mut seen)?;
        finish(&[reference, expression], &mut structural)?;
    }
    finish(
        &[
            equality,
            formula,
            *definiens,
            *attribute,
            block,
            list,
            *unit,
            syntax.nodes().root()?,
        ],
        &mut structural,
    )?;
    terms(&[*semi, *end, *block_semi], &mut next, &mut seen)?;
    if next != EXACT_ATTRIBUTE_TERMINALS.len()
        || seen.len() != row_count
        || seen.len() != syntax.nodes().len()
    {
        return None;
    }
    let definition_block = ast
        .node_views()
        .find(|view| view.id().index() == block.index())?
        .id();
    Some((ExactAttributeDefinition { definition_block }, syntax))
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
        || block.range() != ast.nodes()[41].range
        || block.module() != module
        || block.recovered()
        || attribute.id().index() != 1
        || attribute.ordinal() != 1
        || attribute.kind() != DeclarationShellKind::AttributeDefinition
        || attribute.node_id().index() != 40
        || attribute.parent() != Some(block.id())
        || attribute.range() != ast.nodes()[40].range
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
        || projection.arity() != Some(0)
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
        || definition.arity() != Some(0)
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
    validate_origin(ast.nodes()[40].range, module, definition.origin())?;
    validate_origin(ast.nodes()[40].range, module, symbol.origin())?;
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
    expected_range: SourceRange,
    module: &ModuleId,
    origin: &mizar_resolve::resolved_ast::SemanticOrigin,
) -> Result<(), String> {
    if origin.source_id() != expected_range.source_id
        || origin.module_id() != module
        || origin.anchor() != &SourceAnchor::Range(expected_range)
        || origin.structural_path() != [4, 0, 7, 0]
        || origin.import_edge().is_some()
        || origin.is_recovered()
    {
        return Err("Task261 resolver: attribute origin is not exact".to_owned());
    }
    Ok(())
}

fn task261_arena(syntax: &TypedAst) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    for (typed_id, node) in syntax.nodes().iter() {
        let index = typed_id.index();
        let row = source_row(syntax, index)?;
        let generic_kind = match row.kind() {
            SurfaceNodeKind::Token(_) => "source.surface.token",
            _ => "source.surface.structural",
        };
        let kind = match index {
            24 | 28 => "source.type.head",
            25 | 29 => "source.type.expression",
            27 | 31 => "source.definition.attribute.parameter",
            33 | 35 => "source.term.variable-reference",
            37 => "source.formula.atomic.equality",
            39 => "source.definition.attribute.definiens",
            40 => "source.definition.attribute",
            41 => "source.definition",
            44 => "source.module",
            _ => generic_kind,
        };
        let source_range = match index {
            27 => source_row(syntax, 2)?.range(),
            31 => source_row(syntax, 7)?.range(),
            _ => row.range(),
        };
        let context = if index == 44 {
            LocalTypeContextId::new(0)
        } else {
            LocalTypeContextId::new(1)
        };
        let id = builder
            .push(
                TypedNode::new(kind, SourceAnchor::Range(source_range))
                    .with_children(node.children.clone())
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
        .finish(syntax.nodes().root())
        .map_err(|error| error.to_string())
}

fn task248_context(
    syntax: &TypedAst,
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
    let module_row = source_row(syntax, 44)?;
    let item_row = source_row(syntax, 41)?;
    let parameters = [source_row(syntax, 27)?, source_row(syntax, 31)?];
    let declarations = [source_row(syntax, 2)?, source_row(syntax, 7)?];
    let types = [source_row(syntax, 25)?, source_row(syntax, 29)?];
    let shell = block.id();
    let scope = LocalTermScope::new(vec![0]);
    let module_site = if mutation == SourceAttributeDefinitionRouteMutation::WrongContextModuleSite
    {
        TypedSiteRef::Node(TypedNodeId::new(41))
    } else {
        TypedSiteRef::Node(TypedNodeId::new(module_row.site().index()))
    };
    let item_site = match mutation {
        SourceAttributeDefinitionRouteMutation::WrongContextItemSite => {
            TypedSiteRef::Node(TypedNodeId::new(44))
        }
        SourceAttributeDefinitionRouteMutation::StaleContextItemSite => {
            TypedSiteRef::Node(TypedNodeId::new(42))
        }
        _ => TypedSiteRef::Node(TypedNodeId::new(item_row.site().index())),
    };
    let binding_site = |index: usize| {
        if mutation == SourceAttributeDefinitionRouteMutation::WrongContextBindingSite(index) {
            TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 31 } else { 27 }))
        } else {
            TypedSiteRef::Node(TypedNodeId::new(parameters[index].site().index()))
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
        source_id: syntax.source_id(),
        module_id: module.clone(),
        module_site,
        items: vec![SourceItemInput {
            shell,
            shell_ordinal: 0,
            role: SourceItemRole::DefinitionBlock,
            module_id: module,
            source_range: item_row.range(),
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
                declaration_range: declarations[0].range(),
                written_type_range: types[0].range(),
                site: binding_site(0),
                role: SourceBindingSiteRole::DefinitionParameter {
                    local: LocalTermBinding::new("x", scope.clone(), declarations[0].range(), 0),
                },
                recovery: BindingRecoveryState::Normal,
            },
            SourceBindingSiteInput {
                shell,
                context_owner: context_owner(1),
                source_ordinal: 1,
                spelling: "y".to_owned(),
                declaration_range: declarations[1].range(),
                written_type_range: types[1].range(),
                site: binding_site(1),
                role: SourceBindingSiteRole::DefinitionParameter {
                    local: LocalTermBinding::new("y", scope, declarations[1].range(), 1),
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
    expression: &SourceNode,
    head: &SourceNode,
    module_id: ModuleId,
) -> SourceTypeExpressionInput {
    SourceTypeExpressionInput {
        source_id: expression.range().source_id,
        module_id,
        site: TypedSiteRef::Node(TypedNodeId::new(expression.site().index())),
        source_range: expression.range(),
        spelling: "set".to_owned(),
        head_site: TypedSiteRef::Node(TypedNodeId::new(head.site().index())),
        head_range: head.range(),
        head_spelling: "set".to_owned(),
        form: SourceTypeApplicationForm::Bare,
        head: SourceTypeHead::BuiltinSet,
        recovery: NodeRecoveryState::Normal,
    }
}

fn task249_input(syntax: &TypedAst, module_id: ModuleId) -> Result<SourceTypeHandoffInput, String> {
    let source_id = syntax.source_id();
    Ok(SourceTypeHandoffInput {
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
                source_row(syntax, 25)?,
                source_row(syntax, 24)?,
                module_id.clone(),
            ),
            bare_set_type(source_row(syntax, 29)?, source_row(syntax, 28)?, module_id),
        ],
        arguments: Vec::new(),
    })
}

fn task252_input(
    syntax: &TypedAst,
    module_id: ModuleId,
) -> Result<SourcePrimaryTermHandoffInput, String> {
    let source_id = syntax.source_id();
    let specs = [
        (source_row(syntax, 33)?, "x", BindingId::new(0)),
        (source_row(syntax, 35)?, "y", BindingId::new(1)),
    ];
    Ok(SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: specs
            .iter()
            .enumerate()
            .map(
                |(source_ordinal, (row, spelling, _))| SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(row.site().index())),
                    source_range: row.range(),
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
            .map(|(index, (_, _, binding))| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(index),
                binding: *binding,
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect(),
        numeric_type_requests: Vec::new(),
    })
}

fn task256_input(
    syntax: &TypedAst,
    module_id: ModuleId,
) -> Result<SourceAtomicFormulaHandoffInput, String> {
    let source_id = syntax.source_id();
    let formula = source_row(syntax, 37)?;
    Ok(SourceAtomicFormulaHandoffInput {
        source_id,
        module_id,
        formulas: vec![SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(TypedNodeId::new(formula.site().index())),
            source_range: formula.range(),
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
    })
}

fn task261_input(
    syntax: &TypedAst,
    module_id: ModuleId,
    resolver: (SymbolId, DefinitionId, SourceContributionId),
) -> Result<SourceAttributeDefinitionHandoffInput, String> {
    let source_id = syntax.source_id();
    let definition = source_row(syntax, 40)?;
    let parameters = [source_row(syntax, 27)?, source_row(syntax, 31)?];
    let declarations = [source_row(syntax, 2)?, source_row(syntax, 7)?];
    let subject = source_row(syntax, 14)?;
    let definiens = source_row(syntax, 39)?;
    Ok(SourceAttributeDefinitionHandoffInput {
        source_id,
        module_id,
        definitions: vec![SourceAttributeDefinitionInput {
            symbol: resolver.0,
            definition: resolver.1,
            contribution: resolver.2,
            site: TypedSiteRef::Node(TypedNodeId::new(definition.site().index())),
            source_range: definition.range(),
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
                site: TypedSiteRef::Node(TypedNodeId::new(parameters[0].site().index())),
                source_range: parameters[0].range(),
                declaration_range: declarations[0].range(),
                context: BindingContextId::new(1),
                recovery: SourceAttributeDefinitionRecovery::Normal,
                spelling: "let x be set;".to_owned(),
            },
            SourceAttributeParameterInput {
                owner: SourceAttributeDefinitionId::new(0),
                ordinal: 1,
                binding: BindingId::new(1),
                written_type: SourceTypeApplicationId::new(1),
                site: TypedSiteRef::Node(TypedNodeId::new(parameters[1].site().index())),
                source_range: parameters[1].range(),
                declaration_range: declarations[1].range(),
                context: BindingContextId::new(1),
                recovery: SourceAttributeDefinitionRecovery::Normal,
                spelling: "let y be set;".to_owned(),
            },
        ],
        subjects: vec![SourceAttributeSubjectInput {
            owner: SourceAttributeDefinitionId::new(0),
            binding: BindingId::new(0),
            site: TypedSiteRef::Node(TypedNodeId::new(definition.site().index())),
            source_range: subject.range(),
            context: BindingContextId::new(1),
            recovery: SourceAttributeDefinitionRecovery::Normal,
            spelling: "x".to_owned(),
        }],
        definientia: vec![SourceAttributeDefiniensInput {
            owner: SourceAttributeDefinitionId::new(0),
            ordinal: 0,
            formula: SourceAtomicFormulaId::new(0),
            site: TypedSiteRef::Node(TypedNodeId::new(definiens.site().index())),
            source_range: definiens.range(),
            context: BindingContextId::new(1),
            recovery: SourceAttributeDefinitionRecovery::Normal,
            spelling: "x = y".to_owned(),
        }],
    })
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    exact: (ExactAttributeDefinition, TypedAst),
    mutation: SourceAttributeDefinitionRouteMutation,
) -> Result<SourceAttributeDefinitionRouteOutput, String> {
    let (exact, syntax) = exact;
    let corrupted_symbols = matches!(
        mutation,
        SourceAttributeDefinitionRouteMutation::WrongResolverSymbolEntry
            | SourceAttributeDefinitionRouteMutation::WrongResolverDefinitionEntry
            | SourceAttributeDefinitionRouteMutation::WrongResolverContribution
    )
    .then(|| resolver_env_with_mutation(symbols, mutation));
    let symbols = corrupted_symbols.as_ref().unwrap_or(symbols);
    let resolver = exact_resolver_profile(ast, &module, shells, symbols, mutation)?;
    let source_matches = || -> Option<()> {
        let preprocessed = mizar_lexer::preprocess_source_for_lexing(source_text);
        if !preprocessed.diagnostics.is_empty() {
            return None;
        }
        let raw = mizar_lexer::scan_raw(&preprocessed.lexical_text).ok()?;
        let lexical_env = mizar_lexer::build_lexical_environment(&[], &[]).ok()?;
        let scopes = mizar_lexer::build_scope_skeleton(&raw);
        let tokens = mizar_lexer::disambiguate(
            &raw,
            &lexical_env,
            &mizar_lexer::ParserLexContext::general(),
            &scopes,
        );
        if !tokens.diagnostics().is_empty() || tokens.tokens().len() != ast.token_nodes().len() {
            return None;
        }
        for (actual, supplied) in tokens.tokens().iter().zip(ast.token_views()) {
            let kind = match actual.kind() {
                mizar_lexer::TokenKind::Identifier => SurfaceTokenKind::Identifier,
                mizar_lexer::TokenKind::ReservedWord => SurfaceTokenKind::ReservedWord,
                mizar_lexer::TokenKind::ReservedSymbol => SurfaceTokenKind::ReservedSymbol,
                _ => return None,
            };
            let ranges = preprocessed
                .preprocess_map
                .source_ranges_for_lexical(actual.span())?;
            let [range] = ranges.as_slice() else {
                return None;
            };
            let token = supplied.as_token()?;
            if token.kind != kind
                || token.text.as_ref() != actual.lexeme()
                || supplied.range().start != range.start
                || supplied.range().end != range.end
                || source_text.get(range.start..range.end) != Some(actual.lexeme())
            {
                return None;
            }
        }
        let replay_shells =
            mizar_resolve::declarations::DeclarationShellCollector::new(ast, &module).collect();
        if &replay_shells != shells {
            return None;
        }
        let replay = SignatureProjectionExtractor::new(
            ast,
            &replay_shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        (replay.diagnostics().is_empty() && &replay.into_env() == symbols).then_some(())
    };
    if source_matches().is_none() {
        return Err("Task261 source association is invalid".to_owned());
    }
    let arena = task261_arena(&syntax)?;
    let source_context = task248_context(&syntax, module.clone(), shells, exact, mutation)?;

    let mut type_input = task249_input(&syntax, module.clone())?;
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

    let mut term_input = task252_input(&syntax, module.clone())?;
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

    let mut atomic_input = task256_input(&syntax, module.clone())?;
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
    let mut attribute_input = task261_input(&syntax, module, resolver)?;
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
