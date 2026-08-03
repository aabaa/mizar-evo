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
    source_functor_definition::{
        SourceFunctorCorrectnessInput, SourceFunctorCorrectnessKind, SourceFunctorDefiniensId,
        SourceFunctorDefiniensInput, SourceFunctorDefiniensTarget,
        SourceFunctorDefinitionHandoffInput, SourceFunctorDefinitionId,
        SourceFunctorDefinitionInput, SourceFunctorDefinitionProducer,
        SourceFunctorDefinitionRecovery, SourceFunctorDefinitionStyle, SourceFunctorGuardInput,
        SourceFunctorParameterInput,
    },
    source_term::{
        SourcePrimaryTermHandoffInput, SourcePrimaryTermId, SourcePrimaryTermInput,
        SourcePrimaryTermKind, SourcePrimaryTermProducer, SourcePrimaryTermRecovery,
        SourcePrimaryTermReferenceInput, SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationId, SourceTypeApplicationInput,
        SourceTypeDefinitionReturnExtensionInput, SourceTypeDefinitionReturnId,
        SourceTypeDefinitionReturnInput, SourceTypeDefinitionReturnProducer,
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

pub(in crate::runner) const SOURCE_FUNCTOR_DEFINITION_TEXT: &str = concat!(
    "definition\n",
    "  let x be set;\n",
    "  let y be set;\n",
    "  assume x = x;\n",
    "  func Task260EqualsDef: task260_equals(x) -> set equals x;\n",
    "  func Task260MeansDef: task260_means(y) -> set means x = y;\n",
    "  existence by computation(steps: 1);\n",
    "  uniqueness by computation(steps: 1);\n",
    "end;\n",
);

const INVALID_PAYLOAD_KEY: &str =
    "type_elaboration.checker.source_functor_definition.invalid_payload";

#[derive(Debug)]
pub(in crate::runner) struct SourceFunctorDefinitionRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Rationale: production selects `None`; variants are private corruption seams.
pub(in crate::runner) enum SourceFunctorDefinitionRouteMutation {
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
    WrongReturnType,
    WrongReturnRange,
    WrongReturnExpression(usize),
    WrongTermBinding(usize),
    WrongTermSite(usize),
    RemoveAtomicFormula,
    RemoveAtomicEdge,
    WrongAtomicFormula(usize),
    WrongAtomicEdge(usize),
    WrongAtomicRequest(usize),
    RemoveFunctorGuard,
    WrongFunctorDefiniens,
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
    token_row!(ReservedWord, "assume", 45, 51),
    token_row!(Identifier, "x", 52, 53),
    token_row!(ReservedSymbol, "=", 54, 55),
    token_row!(Identifier, "x", 56, 57),
    token_row!(ReservedSymbol, ";", 57, 58),
    token_row!(ReservedWord, "func", 61, 65),
    token_row!(Identifier, "Task260EqualsDef", 66, 82),
    token_row!(ReservedSymbol, ":", 82, 83),
    token_row!(Identifier, "task260_equals", 84, 98),
    token_row!(ReservedSymbol, "(", 98, 99),
    token_row!(Identifier, "x", 99, 100),
    token_row!(ReservedSymbol, ")", 100, 101),
    token_row!(ReservedSymbol, "->", 102, 104),
    token_row!(ReservedWord, "set", 105, 108),
    token_row!(ReservedWord, "equals", 109, 115),
    token_row!(Identifier, "x", 116, 117),
    token_row!(ReservedSymbol, ";", 117, 118),
    token_row!(ReservedWord, "func", 121, 125),
    token_row!(Identifier, "Task260MeansDef", 126, 141),
    token_row!(ReservedSymbol, ":", 141, 142),
    token_row!(Identifier, "task260_means", 143, 156),
    token_row!(ReservedSymbol, "(", 156, 157),
    token_row!(Identifier, "y", 157, 158),
    token_row!(ReservedSymbol, ")", 158, 159),
    token_row!(ReservedSymbol, "->", 160, 162),
    token_row!(ReservedWord, "set", 163, 166),
    token_row!(ReservedWord, "means", 167, 172),
    token_row!(Identifier, "x", 173, 174),
    token_row!(ReservedSymbol, "=", 175, 176),
    token_row!(Identifier, "y", 177, 178),
    token_row!(ReservedSymbol, ";", 178, 179),
    token_row!(ReservedWord, "existence", 182, 191),
    token_row!(ReservedWord, "by", 192, 194),
    token_row!(ReservedWord, "computation", 195, 206),
    token_row!(ReservedSymbol, "(", 206, 207),
    token_row!(Identifier, "steps", 207, 212),
    token_row!(ReservedSymbol, ":", 212, 213),
    token_row!(Numeral, "1", 214, 215),
    token_row!(ReservedSymbol, ")", 215, 216),
    token_row!(ReservedSymbol, ";", 216, 217),
    token_row!(ReservedWord, "uniqueness", 220, 230),
    token_row!(ReservedWord, "by", 231, 233),
    token_row!(ReservedWord, "computation", 234, 245),
    token_row!(ReservedSymbol, "(", 245, 246),
    token_row!(Identifier, "steps", 246, 251),
    token_row!(ReservedSymbol, ":", 251, 252),
    token_row!(Numeral, "1", 253, 254),
    token_row!(ReservedSymbol, ")", 254, 255),
    token_row!(ReservedSymbol, ";", 255, 256),
    token_row!(ReservedWord, "end", 257, 260),
    token_row!(ReservedSymbol, ";", 260, 261),
    structural_row!(TypeHead, 22, 25, [4]),
    structural_row!(TypeExpression, 22, 25, [62]),
    structural_row!(QualifiedVariableSegment, 17, 25, [2, 3, 63]),
    structural_row!(DefinitionParameter, 13, 26, [1, 64, 5]),
    structural_row!(TypeHead, 38, 41, [9]),
    structural_row!(TypeExpression, 38, 41, [66]),
    structural_row!(QualifiedVariableSegment, 33, 41, [7, 8, 67]),
    structural_row!(DefinitionParameter, 29, 42, [6, 68, 10]),
    structural_row!(TermReference, 52, 53, [12]),
    structural_row!(TermExpression, 52, 53, [70]),
    structural_row!(TermReference, 56, 57, [14]),
    structural_row!(TermExpression, 56, 57, [72]),
    structural_row!(BuiltinPredicateApplication, 52, 57, [71, 13, 73]),
    structural_row!(FormulaExpression, 52, 57, [74]),
    structural_row!(Proposition, 52, 57, [75]),
    structural_row!(AssumptionStatement, 45, 58, [11, 76, 15]),
    structural_row!(FunctorPattern, 84, 101, [19, 20, 21, 22]),
    structural_row!(TypeHead, 105, 108, [24]),
    structural_row!(TypeExpression, 105, 108, [79]),
    structural_row!(TermReference, 116, 117, [26]),
    structural_row!(TermExpression, 116, 117, [81]),
    structural_row!(TermDefiniens, 116, 117, [82]),
    structural_row!(
        FunctorDefinition,
        61,
        118,
        [16, 17, 18, 78, 23, 80, 25, 83, 27]
    ),
    structural_row!(FunctorPattern, 143, 159, [31, 32, 33, 34]),
    structural_row!(TypeHead, 163, 166, [36]),
    structural_row!(TypeExpression, 163, 166, [86]),
    structural_row!(TermReference, 173, 174, [38]),
    structural_row!(TermExpression, 173, 174, [88]),
    structural_row!(TermReference, 177, 178, [40]),
    structural_row!(TermExpression, 177, 178, [90]),
    structural_row!(BuiltinPredicateApplication, 173, 178, [89, 39, 91]),
    structural_row!(FormulaExpression, 173, 178, [92]),
    structural_row!(FormulaDefiniens, 173, 178, [93]),
    structural_row!(
        FunctorDefinition,
        121,
        179,
        [28, 29, 30, 85, 35, 87, 37, 94, 41]
    ),
    structural_row!(ComputationOption, 207, 215, [46, 47, 48]),
    structural_row!(ComputationJustification, 195, 216, [44, 45, 96, 49]),
    structural_row!(JustificationClause, 192, 216, [43, 97]),
    structural_row!(CorrectnessCondition, 182, 217, [42, 98, 50]),
    structural_row!(ComputationOption, 246, 254, [55, 56, 57]),
    structural_row!(ComputationJustification, 234, 255, [53, 54, 100, 58]),
    structural_row!(JustificationClause, 231, 255, [52, 101]),
    structural_row!(CorrectnessCondition, 220, 256, [51, 102, 59]),
    structural_row!(
        DefinitionBlockItem,
        0,
        261,
        [0, 65, 69, 77, 84, 95, 99, 103, 60, 61]
    ),
    structural_row!(ItemList, 0, 261, [104]),
    structural_row!(CompilationUnit, 0, 261, [105]),
    structural_row!(
        Root,
        0,
        261,
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 106
        ]
    ),
];

#[derive(Debug, Clone, Copy)]
struct ExactFunctorDefinition {
    definition_block: SurfaceNodeId,
}

pub(in crate::runner) fn source_functor_definition_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_functor_definition_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceFunctorDefinitionRouteMutation::None,
    ) {
        None => None,
        Some(Ok(output)) if route_output_is_exact(&output) => Some(Vec::new()),
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_functor_definition_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceFunctorDefinitionRouteOutput, String>> {
    source_functor_definition_output_impl(
        ast,
        module,
        shells,
        symbols,
        source_text,
        SourceFunctorDefinitionRouteMutation::None,
    )
}

#[cfg(test)]
pub(in crate::runner) fn source_functor_definition_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceFunctorDefinitionRouteMutation,
) -> Option<Result<SourceFunctorDefinitionRouteOutput, String>> {
    source_functor_definition_output_impl(ast, module, shells, symbols, source_text, mutation)
}

fn source_functor_definition_output_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutation: SourceFunctorDefinitionRouteMutation,
) -> Option<Result<SourceFunctorDefinitionRouteOutput, String>> {
    let exact = exact_functor_definition(ast, source_text)?;
    Some(build_output(ast, module, shells, symbols, exact, mutation))
}

fn exact_functor_definition(ast: &SurfaceAst, source_text: &str) -> Option<ExactFunctorDefinition> {
    if source_text != SOURCE_FUNCTOR_DEFINITION_TEXT
        || source_text.len() != 262
        || !source_text.ends_with('\n')
        || source_text.ends_with("\n\n")
        || ast.root().map(SurfaceNodeId::index) != Some(107)
        || ast.expression_root().is_some()
        || !surface_profile_is_exact(ast)
    {
        return None;
    }

    let root = exact_node(ast, 107, SurfaceNodeKind::Root, 0, 261)?;
    let definition_block = exact_node(ast, 104, SurfaceNodeKind::DefinitionBlockItem, 0, 261)?;
    let equals = exact_node(ast, 84, SurfaceNodeKind::FunctorDefinition, 61, 118)?;
    let means = exact_node(ast, 95, SurfaceNodeKind::FunctorDefinition, 121, 179)?;
    let existence = exact_node(ast, 99, SurfaceNodeKind::CorrectnessCondition, 182, 217)?;
    let uniqueness = exact_node(ast, 103, SurfaceNodeKind::CorrectnessCondition, 220, 256)?;
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
    if direct_structural != [65, 69, 77, 84, 95, 99, 103]
        || !is_descendant(ast, root, definition_block)
        || is_descendant(ast, equals, means)
        || is_descendant(ast, equals, existence)
        || is_descendant(ast, means, uniqueness)
    {
        return None;
    }
    Some(ExactFunctorDefinition { definition_block })
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
                    && actual.children.len() == expected.children.len()
                    && actual
                        .children
                        .iter()
                        .zip(expected.children)
                        .all(|(actual, expected)| actual.index() == *expected)
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
    mutation: SourceFunctorDefinitionRouteMutation,
) -> Result<[(SymbolId, DefinitionId, SourceContributionId); 2], String> {
    let declarations = if mutation == SourceFunctorDefinitionRouteMutation::RemoveResolverShell
        && !shells.declarations().is_empty()
    {
        &shells.declarations()[..shells.declarations().len() - 1]
    } else {
        shells.declarations()
    };
    let [block, equals, means] = declarations else {
        return Err("Task260 resolver: expected three declaration shells".to_owned());
    };
    if !shells.exports().is_empty()
        || block.id().index() != 0
        || block.ordinal() != 0
        || block.kind() != DeclarationShellKind::DefinitionBlock
        || block.node_id().index() != 104
        || block.parent().is_some()
        || block.range() != source_range(ast.source_id, 0, 261)
        || block.module() != module
        || block.recovered()
        || equals.id().index() != 1
        || equals.ordinal() != 1
        || equals.kind() != DeclarationShellKind::FunctorDefinition
        || equals.node_id().index() != 84
        || equals.parent() != Some(block.id())
        || equals.range() != source_range(ast.source_id, 61, 118)
        || equals.module() != module
        || equals.recovered()
        || means.id().index() != 2
        || means.ordinal() != 2
        || means.kind() != DeclarationShellKind::FunctorDefinition
        || means.node_id().index() != 95
        || means.parent() != Some(block.id())
        || means.range() != source_range(ast.source_id, 121, 179)
        || means.module() != module
        || means.recovered()
        || symbols.module_id() != module
        || symbols.symbols().len() != 2
        || symbols.definitions().len() != 2
        || symbols.contributions().len() != 1
    {
        return Err("Task260 resolver: raw profile is not exact".to_owned());
    }

    let mut projections =
        SignatureProjectionExtractor::new(ast, shells, NamespacePath::new(module.path().as_str()))
            .extract();
    if mutation == SourceFunctorDefinitionRouteMutation::WrongResolverProjection {
        projections[0] = projections[0]
            .clone()
            .with_definition_kind(DefinitionKind::Predicate);
    }
    let expected_spellings = ["task260_equals ( x )", "task260_means ( y )"];
    if projections.len() != 2
        || projections
            .iter()
            .zip(expected_spellings)
            .any(|(actual, expected)| {
                actual.primary_spelling() != expected
                    || actual.notation_spelling() != Some(expected)
                    || actual.symbol_kind() != SymbolKind::Functor
                    || actual.definition_kind() != Some(DefinitionKind::Functor)
                    || actual.arity().is_some()
                    || actual.overload_policy() != SymbolOverloadPolicy::Overloadable
            })
    {
        return Err("Task260 resolver: parser-backed projections are not exact".to_owned());
    }
    let mut result = Vec::with_capacity(2);
    for (index, (expected, start, end, path)) in [
        (expected_spellings[0], 61, 118, &[4, 0, 9, 0][..]),
        (expected_spellings[1], 121, 179, &[4, 0, 9, 1][..]),
    ]
    .into_iter()
    .enumerate()
    {
        let definition = symbols
            .definitions()
            .iter()
            .find(|definition| definition.id().index() == index)
            .ok_or_else(|| format!("Task260 resolver: definition {index} disappeared"))?;
        let symbol = symbols
            .symbols()
            .get(definition.symbol())
            .ok_or_else(|| format!("Task260 resolver: symbol {index} disappeared"))?;
        if definition.kind() != DefinitionKind::Functor
            || definition.visibility() != Visibility::Public
            || !definition.parameters().is_empty()
            || !definition.binders().is_empty()
            || definition.arity().is_some()
            || definition.conflict().is_some()
            || symbol.kind() != SymbolKind::Functor
            || symbol.visibility() != Visibility::Public
            || symbol.export_status() != ExportStatus::Exported
            || symbol.primary_spelling() != expected
            || symbol.notation_spelling() != Some(expected)
            || symbol.contribution() != definition.contribution()
        {
            return Err(format!("Task260 resolver: functor {index} is not exact"));
        }
        validate_origin(ast.source_id, module, definition.origin(), start, end, path)?;
        validate_origin(ast.source_id, module, symbol.origin(), start, end, path)?;
        result.push((
            symbol.symbol().clone(),
            definition.id(),
            definition.contribution(),
        ));
    }
    let contribution_id = result[0].2;
    let contribution = symbols
        .contributions()
        .get(contribution_id)
        .ok_or_else(|| "Task260 resolver: local contribution disappeared".to_owned())?;
    if result[1].2 != contribution_id
        || contribution.module() != module
        || !matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == ast.source_id
        )
        || result.iter().any(|(symbol, definition, _)| {
            !contribution.effects().symbols().contains(symbol)
                || !contribution.effects().definitions().contains(definition)
        })
    {
        return Err("Task260 resolver: local contribution is not exact".to_owned());
    }
    result
        .try_into()
        .map_err(|_| "Task260 resolver: result cardinality changed".to_owned())
}

fn resolver_env_with_mutation(
    symbols: &SymbolEnv,
    mutation: SourceFunctorDefinitionRouteMutation,
) -> SymbolEnv {
    let mut rebuilt_symbols = SymbolIndex::new();
    let mut corrupted_entry = false;
    for entry in symbols.symbols().iter() {
        let primary_spelling = if mutation
            == SourceFunctorDefinitionRouteMutation::WrongResolverEntry
            && !corrupted_entry
            && entry.kind() == SymbolKind::Functor
        {
            corrupted_entry = true;
            "corrupted_task260_equals ( x )"
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
        if let Some(notation_spelling) = entry.notation_spelling() {
            rebuilt = rebuilt.with_notation_spelling(notation_spelling);
        }
        if let Some(signature) = entry.signature() {
            rebuilt = rebuilt.with_signature(signature.clone());
        }
        rebuilt_symbols.insert(rebuilt);
    }

    let rebuilt_contributions =
        if mutation == SourceFunctorDefinitionRouteMutation::WrongResolverContribution {
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

    let rebuilt_definitions =
        if mutation == SourceFunctorDefinitionRouteMutation::WrongResolverDefinitionEntry {
            let mut definitions = DefinitionIndex::new();
            for (index, original) in symbols.definitions().iter().enumerate() {
                let kind = if index == 0 {
                    DefinitionKind::Predicate
                } else {
                    original.kind()
                };
                let mut rebuilt = DefinitionShell::new(
                    original.symbol().clone(),
                    kind,
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
                if let Some(notation_shape) = original.notation_shape() {
                    rebuilt = rebuilt.with_notation_shape(notation_shape);
                }
                if let Some(doc_attachment) = original.doc_attachment() {
                    rebuilt = rebuilt.with_doc_attachment(doc_attachment.clone());
                }
                if let Some(conflict) = original.conflict() {
                    rebuilt = rebuilt.with_conflict(conflict.clone());
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
        return Err("Task260 resolver: functor origin is not exact".to_owned());
    }
    Ok(())
}

fn task260_arena(source_id: SourceId) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    for (index, row) in EXACT_SURFACE_PROFILE.iter().enumerate() {
        let generic_kind = match row.kind {
            ExactSurfaceKind::Token(_, _) => "source.surface.token",
            ExactSurfaceKind::Structural(_) => "source.surface.structural",
        };
        let (kind, start, end) = match index {
            62 => ("source.type.head", 22, 25),
            63 => ("source.type.expression", 22, 25),
            65 => ("source.definition.functor.parameter", 17, 18),
            66 => ("source.type.head", 38, 41),
            67 => ("source.type.expression", 38, 41),
            69 => ("source.definition.functor.parameter", 33, 34),
            70 => ("source.term.variable-reference", 52, 53),
            72 => ("source.term.variable-reference", 56, 57),
            75 => ("source.formula.atomic.equality", 52, 57),
            77 => ("source.definition.functor.guard", 45, 58),
            79 => ("source.type.head", 105, 108),
            80 => ("source.type.expression", 105, 108),
            81 => ("source.term.variable-reference", 116, 117),
            83 => ("source.definition.functor.definiens", 116, 117),
            84 => ("source.definition.functor", 61, 118),
            86 => ("source.type.head", 163, 166),
            87 => ("source.type.expression", 163, 166),
            88 => ("source.term.variable-reference", 173, 174),
            90 => ("source.term.variable-reference", 177, 178),
            93 => ("source.formula.atomic.equality", 173, 178),
            94 => ("source.definition.functor.definiens", 173, 178),
            95 => ("source.definition.functor", 121, 179),
            99 => ("source.definition.functor.correctness", 182, 217),
            103 => ("source.definition.functor.correctness", 220, 256),
            104 => ("source.definition", 0, 261),
            107 => ("source.module", 0, 261),
            _ => (generic_kind, row.start, row.end),
        };
        let children = row
            .children
            .iter()
            .copied()
            .map(TypedNodeId::new)
            .collect::<Vec<_>>();
        let context = if index == 107 {
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
            return Err("Task260 arena order changed".to_owned());
        }
    }
    builder
        .finish(Some(TypedNodeId::new(107)))
        .map_err(|error| error.to_string())
}

fn task248_context(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    exact: ExactFunctorDefinition,
    mutation: SourceFunctorDefinitionRouteMutation,
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
    let module_site = if mutation == SourceFunctorDefinitionRouteMutation::WrongContextModuleSite {
        TypedSiteRef::Node(TypedNodeId::new(104))
    } else {
        TypedSiteRef::Node(TypedNodeId::new(107))
    };
    let item_site = if mutation == SourceFunctorDefinitionRouteMutation::WrongContextItemSite {
        TypedSiteRef::Node(TypedNodeId::new(107))
    } else {
        TypedSiteRef::Node(TypedNodeId::new(104))
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
            source_range: source_range(ast.source_id, 0, 261),
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
                site: if mutation
                    == SourceFunctorDefinitionRouteMutation::WrongContextBindingSite(0)
                {
                    TypedSiteRef::Node(TypedNodeId::new(69))
                } else {
                    TypedSiteRef::Node(TypedNodeId::new(65))
                },
                context_owner: if mutation
                    == SourceFunctorDefinitionRouteMutation::WrongContextBindingOwner(0)
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
                context_owner: if mutation
                    == SourceFunctorDefinitionRouteMutation::WrongContextBindingOwner(1)
                {
                    SourceBindingContextOwner::Module
                } else {
                    SourceBindingContextOwner::Shell(shell)
                },
                source_ordinal: 1,
                spelling: "y".to_owned(),
                declaration_range: source_range(ast.source_id, 33, 34),
                written_type_range: source_range(ast.source_id, 38, 41),
                site: if mutation
                    == SourceFunctorDefinitionRouteMutation::WrongContextBindingSite(1)
                {
                    TypedSiteRef::Node(TypedNodeId::new(65))
                } else {
                    TypedSiteRef::Node(TypedNodeId::new(69))
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
            bare_set_type(source_id, module_id.clone(), 63, 62, 22, 25),
            bare_set_type(source_id, module_id, 67, 66, 38, 41),
        ],
        arguments: Vec::new(),
    }
}

fn task249r_input(
    source_id: SourceId,
    module_id: ModuleId,
) -> SourceTypeDefinitionReturnExtensionInput {
    SourceTypeDefinitionReturnExtensionInput {
        source_id,
        module_id: module_id.clone(),
        returns: vec![
            SourceTypeDefinitionReturnInput {
                definition_site: TypedSiteRef::Node(TypedNodeId::new(84)),
                definition_range: source_range(source_id, 61, 118),
                source_ordinal: 0,
                expression: bare_set_type(source_id, module_id.clone(), 80, 79, 105, 108),
            },
            SourceTypeDefinitionReturnInput {
                definition_site: TypedSiteRef::Node(TypedNodeId::new(95)),
                definition_range: source_range(source_id, 121, 179),
                source_ordinal: 1,
                expression: bare_set_type(source_id, module_id, 87, 86, 163, 166),
            },
        ],
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

fn task252_input(source_id: SourceId, module_id: ModuleId) -> SourcePrimaryTermHandoffInput {
    let specs = [
        (70, 52, 53, "x", BindingId::new(0)),
        (72, 56, 57, "x", BindingId::new(0)),
        (81, 116, 117, "x", BindingId::new(0)),
        (88, 173, 174, "x", BindingId::new(0)),
        (90, 177, 178, "y", BindingId::new(1)),
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
    let formulas = vec![
        SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(TypedNodeId::new(75)),
            source_range: source_range(source_id, 52, 57),
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = x".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        },
        SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(TypedNodeId::new(93)),
            source_range: source_range(source_id, 173, 178),
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
        (1, 0, SourceAtomicEdgeRole::BuiltinLeftOperand, 3),
        (1, 1, SourceAtomicEdgeRole::BuiltinRightOperand, 4),
    ]
    .into_iter()
    .map(|(formula, ordinal, role, target)| SourceAtomicEdgeInput {
        formula: SourceAtomicFormulaId::new(formula),
        ordinal,
        role,
        target: SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(target)),
    })
    .collect::<Vec<_>>();
    let requests = (0..4)
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

fn task260_input(
    source_id: SourceId,
    module_id: ModuleId,
    resolver: [(SymbolId, DefinitionId, SourceContributionId); 2],
) -> SourceFunctorDefinitionHandoffInput {
    SourceFunctorDefinitionHandoffInput {
        source_id,
        module_id,
        definitions: vec![
            SourceFunctorDefinitionInput {
                symbol: resolver[0].0.clone(),
                definition: resolver[0].1,
                contribution: resolver[0].2,
                site: TypedSiteRef::Node(TypedNodeId::new(84)),
                source_range: source_range(source_id, 61, 118),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "func Task260EqualsDef: task260_equals(x) -> set equals x;".to_owned(),
                style: SourceFunctorDefinitionStyle::Equals,
                return_type: SourceTypeDefinitionReturnId::new(0),
                definiens: SourceFunctorDefiniensId::new(0),
            },
            SourceFunctorDefinitionInput {
                symbol: resolver[1].0.clone(),
                definition: resolver[1].1,
                contribution: resolver[1].2,
                site: TypedSiteRef::Node(TypedNodeId::new(95)),
                source_range: source_range(source_id, 121, 179),
                source_ordinal: 1,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "func Task260MeansDef: task260_means(y) -> set means x = y;".to_owned(),
                style: SourceFunctorDefinitionStyle::Means,
                return_type: SourceTypeDefinitionReturnId::new(1),
                definiens: SourceFunctorDefiniensId::new(1),
            },
        ],
        parameters: vec![
            SourceFunctorParameterInput {
                ordinal: 0,
                binding: BindingId::new(0),
                written_type: SourceTypeApplicationId::new(0),
                site: TypedSiteRef::Node(TypedNodeId::new(65)),
                source_range: source_range(source_id, 13, 26),
                declaration_range: source_range(source_id, 17, 18),
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "let x be set;".to_owned(),
            },
            SourceFunctorParameterInput {
                ordinal: 1,
                binding: BindingId::new(1),
                written_type: SourceTypeApplicationId::new(1),
                site: TypedSiteRef::Node(TypedNodeId::new(69)),
                source_range: source_range(source_id, 29, 42),
                declaration_range: source_range(source_id, 33, 34),
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "let y be set;".to_owned(),
            },
        ],
        guards: vec![SourceFunctorGuardInput {
            ordinal: 0,
            formula: SourceAtomicFormulaId::new(0),
            site: TypedSiteRef::Node(TypedNodeId::new(77)),
            source_range: source_range(source_id, 45, 58),
            context: BindingContextId::new(1),
            recovery: SourceFunctorDefinitionRecovery::Normal,
            spelling: "assume x = x;".to_owned(),
        }],
        definientia: vec![
            SourceFunctorDefiniensInput {
                owner: SourceFunctorDefinitionId::new(0),
                ordinal: 0,
                target: SourceFunctorDefiniensTarget::Primary(SourcePrimaryTermId::new(2)),
                site: TypedSiteRef::Node(TypedNodeId::new(83)),
                source_range: source_range(source_id, 116, 117),
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "x".to_owned(),
            },
            SourceFunctorDefiniensInput {
                owner: SourceFunctorDefinitionId::new(1),
                ordinal: 1,
                target: SourceFunctorDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(1)),
                site: TypedSiteRef::Node(TypedNodeId::new(94)),
                source_range: source_range(source_id, 173, 178),
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "x = y".to_owned(),
            },
        ],
        correctness: vec![
            SourceFunctorCorrectnessInput {
                owner: SourceFunctorDefinitionId::new(1),
                ordinal: 0,
                kind: SourceFunctorCorrectnessKind::Existence,
                site: TypedSiteRef::Node(TypedNodeId::new(99)),
                source_range: source_range(source_id, 182, 217),
                justification: SourceAnchor::Range(source_range(source_id, 192, 216)),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "existence by computation(steps: 1);".to_owned(),
            },
            SourceFunctorCorrectnessInput {
                owner: SourceFunctorDefinitionId::new(1),
                ordinal: 1,
                kind: SourceFunctorCorrectnessKind::Uniqueness,
                site: TypedSiteRef::Node(TypedNodeId::new(103)),
                source_range: source_range(source_id, 220, 256),
                justification: SourceAnchor::Range(source_range(source_id, 231, 255)),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: "uniqueness by computation(steps: 1);".to_owned(),
            },
        ],
    }
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    exact: ExactFunctorDefinition,
    mutation: SourceFunctorDefinitionRouteMutation,
) -> Result<SourceFunctorDefinitionRouteOutput, String> {
    let corrupted_symbols = matches!(
        mutation,
        SourceFunctorDefinitionRouteMutation::WrongResolverEntry
            | SourceFunctorDefinitionRouteMutation::WrongResolverDefinitionEntry
            | SourceFunctorDefinitionRouteMutation::WrongResolverContribution
    )
    .then(|| resolver_env_with_mutation(symbols, mutation));
    let symbols = corrupted_symbols.as_ref().unwrap_or(symbols);
    let resolver = exact_resolver_profile(ast, &module, shells, symbols, mutation)?;
    let arena = task260_arena(ast.source_id)?;
    let source_context = task248_context(ast, module.clone(), shells, exact, mutation)?;

    let mut type_input = task249_input(ast.source_id, module.clone());
    if mutation == SourceFunctorDefinitionRouteMutation::RemoveTypeExpression {
        type_input.expressions.pop();
    }
    if let SourceFunctorDefinitionRouteMutation::WrongTypeApplicationBinding(index) = mutation {
        type_input.applications[index].binding = BindingId::new(1 - index);
    }
    if let SourceFunctorDefinitionRouteMutation::WrongTypeApplicationRoot(index) = mutation {
        type_input.applications[index].root = SourceTypeExpressionId::new(1 - index);
    }
    if let SourceFunctorDefinitionRouteMutation::WrongTypeExpressionSite(index) = mutation {
        type_input.expressions[index].site =
            TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 67 } else { 63 }));
    }
    let base_type =
        SourceTypeProducer::build(type_input, source_context.binding_env(), symbols, &arena)
            .map_err(|error| format!("Task249 source type: {error}"))?;
    let mut return_input = task249r_input(ast.source_id, module.clone());
    if mutation == SourceFunctorDefinitionRouteMutation::WrongReturnType {
        return_input.returns[1].definition_site = TypedSiteRef::Node(TypedNodeId::new(84));
    }
    if mutation == SourceFunctorDefinitionRouteMutation::WrongReturnRange {
        return_input.returns[1].definition_range = source_range(ast.source_id, 61, 118);
    }
    if let SourceFunctorDefinitionRouteMutation::WrongReturnExpression(index) = mutation {
        return_input.returns[index].expression.site =
            TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 87 } else { 80 }));
    }
    let source_type = SourceTypeDefinitionReturnProducer::extend(&base_type, return_input, &arena)
        .map_err(|error| format!("Task249R return type: {error}"))?;

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
    .map_err(|error| format!("Task249R typed installation: {error}"))?;

    let mut term_input = task252_input(ast.source_id, module.clone());
    if let SourceFunctorDefinitionRouteMutation::WrongTermBinding(index) = mutation {
        term_input.references[index].binding = if index == 4 {
            BindingId::new(0)
        } else {
            BindingId::new(1)
        };
    }
    if let SourceFunctorDefinitionRouteMutation::WrongTermSite(index) = mutation {
        term_input.terms[index].site =
            TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 72 } else { 70 }));
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
    if mutation == SourceFunctorDefinitionRouteMutation::RemoveAtomicFormula {
        atomic_input.formulas.pop();
    }
    if mutation == SourceFunctorDefinitionRouteMutation::RemoveAtomicEdge {
        atomic_input.edges.pop();
    }
    if let SourceFunctorDefinitionRouteMutation::WrongAtomicFormula(index) = mutation {
        atomic_input.formulas[index].site =
            TypedSiteRef::Node(TypedNodeId::new(if index == 0 { 93 } else { 75 }));
    }
    if let SourceFunctorDefinitionRouteMutation::WrongAtomicEdge(index) = mutation {
        atomic_input.edges[index].target =
            SourceAtomicTermTarget::Primary(SourcePrimaryTermId::new(99));
    }
    if let SourceFunctorDefinitionRouteMutation::WrongAtomicRequest(index) = mutation {
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

    let mut functor_input = task260_input(ast.source_id, module, resolver);
    if mutation == SourceFunctorDefinitionRouteMutation::RemoveFunctorGuard {
        functor_input.guards.clear();
    }
    if mutation == SourceFunctorDefinitionRouteMutation::WrongFunctorDefiniens {
        functor_input.definientia[1].target =
            SourceFunctorDefiniensTarget::Primary(SourcePrimaryTermId::new(2));
    }
    let projection = SourceFunctorDefinitionProducer::build(
        functor_input,
        symbols,
        typed_ast
            .source_context()
            .ok_or_else(|| "Task248 source context disappeared".to_owned())?,
        typed_ast
            .source_type()
            .ok_or_else(|| "Task249R source type disappeared".to_owned())?,
        typed_ast
            .source_term()
            .ok_or_else(|| "Task252 source term disappeared".to_owned())?,
        None,
        None,
        None,
        typed_ast.source_atomic_formula(),
        typed_ast.initial_obligations(),
        typed_ast.nodes(),
    )
    .map_err(|error| format!("Task260 functor definition: {error}"))?;
    typed_ast = typed_ast
        .with_source_functor_definition(projection)
        .map_err(|error| format!("Task260 typed installation: {error}"))?;

    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.definition.functor"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)
        .map_err(|error| format!("Task260 final assembly: {error}"))?;
    Ok(SourceFunctorDefinitionRouteOutput {
        typed_ast,
        resolved,
    })
}

fn route_output_is_exact(output: &SourceFunctorDefinitionRouteOutput) -> bool {
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
    let Some(functor) = typed.source_functor_definition() else {
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
        && source_type.expressions().len() == 4
        && source_type.arguments().is_empty()
        && source_type.definition_returns().len() == 2
        && source_term.terms().len() == 5
        && source_term.references().len() == 5
        && source_term.numeric_type_requests().is_empty()
        && atomic.formulas().len() == 2
        && atomic.wrappers().is_empty()
        && atomic.predicate_segments().is_empty()
        && atomic.predicate_heads().is_empty()
        && atomic.candidates().is_empty()
        && atomic.type_sites().is_empty()
        && atomic.attributes().is_empty()
        && atomic.edges().len() == 4
        && atomic.requests().len() == 4
        && functor.definitions().len() == 2
        && functor.parameters().len() == 2
        && functor.guards().len() == 1
        && functor.definientia().len() == 2
        && functor.correctness().len() == 2
        && typed.source_functor_definition() == resolved.source_functor_definition()
        && typed.source_context() == resolved.source_context()
        && typed.source_type() == resolved.source_type()
        && typed.source_term() == resolved.source_term()
        && typed.source_atomic_formula() == resolved.source_atomic_formula()
        && typed.initial_obligations().len() == 2
        && typed.types().is_empty()
        && typed.facts().is_empty()
        && typed.coercions().is_empty()
        && typed.diagnostics().is_empty()
        && typed.source_predicate_definition().is_none()
        && resolved.source_predicate_definition().is_none()
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
