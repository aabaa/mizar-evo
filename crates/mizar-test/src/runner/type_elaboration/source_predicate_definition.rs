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
    type_checker::{TypeExpressionInput, TypeNormalizer},
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextId, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedArenaBuilder, TypedAst,
        TypedAstParts, TypedNode, TypedNodeId, TypedNodeLinks, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShellKind, DeclarationShellSet},
    env::{
        ContributionKind, DefinitionId, DefinitionKind, ExportStatus, NamespacePath,
        SourceContributionId, SymbolEntry, SymbolEnv, SymbolEnvIndexes, SymbolIndex, SymbolKind,
        Visibility,
    },
    resolved_ast::{ModuleId, SymbolId},
    symbols::SignatureProjectionExtractor,
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNodeId, SurfaceNodeKind, SurfaceTokenKind};

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_ast::{
        authenticated_local_declaration, direct_token_texts, leaf_token_texts,
        structural_child_ids, subtree_has_recovery, surface_nodes_with_kind, surface_site,
    },
    source_context::{
        SourceTwoParameterDefinitionContextSites,
        source_two_parameter_definition_context_projection,
    },
};

#[cfg(test)]
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
    children: &'static [usize],
    recovered: bool,
}

macro_rules! token_row {
    ($kind:ident, $text:literal) => {
        ExactSurfaceRow {
            kind: ExactSurfaceKind::Token(SurfaceTokenKind::$kind, $text),
            children: &[],
            recovered: false,
        }
    };
}

macro_rules! structural_row {
    ($kind:ident, [$($child:literal),* $(,)?]) => {
        ExactSurfaceRow {
            kind: ExactSurfaceKind::Structural(SurfaceNodeKind::$kind),
            children: &[$($child),*],
            recovered: false,
        }
    };
}

const EXACT_SURFACE_PROFILE: &[ExactSurfaceRow] = &[
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
    token_row!(ReservedWord, "assume"),
    token_row!(Identifier, "x"),
    token_row!(ReservedSymbol, "="),
    token_row!(Identifier, "x"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "pred"),
    token_row!(Identifier, "Task259PredicateDefinition"),
    token_row!(ReservedSymbol, ":"),
    token_row!(Identifier, "x"),
    token_row!(Identifier, "task259_rel"),
    token_row!(Identifier, "y"),
    token_row!(ReservedWord, "means"),
    token_row!(Identifier, "x"),
    token_row!(ReservedSymbol, "="),
    token_row!(Identifier, "y"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "symmetry"),
    token_row!(ReservedWord, "by"),
    token_row!(ReservedWord, "computation"),
    token_row!(ReservedSymbol, "("),
    token_row!(Identifier, "steps"),
    token_row!(ReservedSymbol, ":"),
    token_row!(Numeral, "1"),
    token_row!(ReservedSymbol, ")"),
    token_row!(ReservedSymbol, ";"),
    token_row!(ReservedWord, "end"),
    token_row!(ReservedSymbol, ";"),
    structural_row!(TypeHead, [4]),
    structural_row!(TypeExpression, [38]),
    structural_row!(QualifiedVariableSegment, [2, 3, 39]),
    structural_row!(DefinitionParameter, [1, 40, 5]),
    structural_row!(TypeHead, [9]),
    structural_row!(TypeExpression, [42]),
    structural_row!(QualifiedVariableSegment, [7, 8, 43]),
    structural_row!(DefinitionParameter, [6, 44, 10]),
    structural_row!(TermReference, [12]),
    structural_row!(TermExpression, [46]),
    structural_row!(TermReference, [14]),
    structural_row!(TermExpression, [48]),
    structural_row!(BuiltinPredicateApplication, [47, 13, 49]),
    structural_row!(FormulaExpression, [50]),
    structural_row!(Proposition, [51]),
    structural_row!(AssumptionStatement, [11, 52, 15]),
    structural_row!(PredicatePattern, [19, 20, 21]),
    structural_row!(TermReference, [23]),
    structural_row!(TermExpression, [55]),
    structural_row!(TermReference, [25]),
    structural_row!(TermExpression, [57]),
    structural_row!(BuiltinPredicateApplication, [56, 24, 58]),
    structural_row!(FormulaExpression, [59]),
    structural_row!(FormulaDefiniens, [60]),
    structural_row!(PredicateDefinition, [16, 17, 18, 54, 22, 61, 26]),
    structural_row!(ComputationOption, [31, 32, 33]),
    structural_row!(ComputationJustification, [29, 30, 63, 34]),
    structural_row!(JustificationClause, [28, 64]),
    structural_row!(PropertyClause, [27, 65, 35]),
    structural_row!(DefinitionBlockItem, [0, 41, 45, 53, 62, 66, 36, 37]),
    structural_row!(ItemList, [67]),
    structural_row!(CompilationUnit, [68]),
    structural_row!(
        Root,
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

pub(in crate::runner) fn step5c5_predicate_argument_detail_keys(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    symbols: &SymbolEnv,
    typed: &TypedArena,
) -> Result<Vec<String>, String> {
    use mizar_checker::overload_resolution::{
        CandidateDeclarationKind, CandidateRejectionReason, CandidateViabilityStatus,
        OverloadDiagnosticClass, OverloadResultStatus, OverloadSiteKind,
    };
    use mizar_checker::type_checker::{
        NormalizedTypeStatus, TypeHeadRef, check_source_distinct_loci_overloads,
    };
    let (normalization, collection, expansion, viability, _, selection, _) =
        check_source_distinct_loci_overloads(source, symbols, typed, true, None)?;
    let invalid = || "predicates.application.unsupported_argument_evidence".to_owned();
    let mut rejected = 0;
    for (_, decision) in viability.decisions().iter() {
        let candidate = expansion
            .candidates()
            .get(decision.source_candidate)
            .ok_or_else(invalid)?;
        let result = selection
            .results()
            .iter()
            .find(|(_, result)| result.site == decision.site)
            .map(|(_, result)| result)
            .ok_or_else(invalid)?;
        let site = collection.sites().get(decision.site).ok_or_else(invalid)?;
        if candidate.site != decision.site
            || site.kind != OverloadSiteKind::PredicateApplication
            || candidate.declaration_kind != CandidateDeclarationKind::Predicate
            || candidate.parameters.len() != 2
            || site.arguments.len() != 2
            || candidate.symbol != candidate.ordinary_root
            || !symbols.symbols().iter().any(|entry| {
                entry.symbol() == &candidate.symbol && entry.kind() == SymbolKind::Predicate
            })
        {
            return Err(invalid());
        }
        match (&decision.status, &result.status) {
            (CandidateViabilityStatus::Viable { .. }, OverloadResultStatus::Resolved { .. }) => {}
            (
                CandidateViabilityStatus::Rejected { reasons },
                OverloadResultStatus::NoMatch { .. },
            ) => {
                let [diagnostic] = decision.diagnostics.as_slice() else {
                    return Err(invalid());
                };
                let diagnostic = viability
                    .diagnostics()
                    .get(*diagnostic)
                    .ok_or_else(invalid)?;
                if reasons.is_empty()
                    || reasons.len() > 2
                    || reasons
                        .windows(2)
                        .any(|pair| pair[0].argument_index >= pair[1].argument_index)
                {
                    return Err(invalid());
                }
                for reason in reasons {
                    let actual = normalization
                        .normalized_types()
                        .get(reason.actual.ok_or_else(invalid)?)
                        .ok_or_else(invalid)?;
                    let target = normalization
                        .normalized_types()
                        .get(reason.target.ok_or_else(invalid)?)
                        .ok_or_else(invalid)?;
                    if reason.reason != CandidateRejectionReason::MissingEvidence
                        || candidate.parameters.get(reason.argument_index) != Some(&target.id)
                        || actual.status != NormalizedTypeStatus::Known
                        || target.status != NormalizedTypeStatus::Known
                        || actual.head != TypeHeadRef::BuiltinSet
                        || !matches!(&target.head, TypeHeadRef::Structure(symbol) if symbols.symbols().iter().any(|entry| entry.symbol() == symbol && entry.kind() == SymbolKind::Structure))
                        || !actual.args.is_empty()
                        || !target.args.is_empty()
                        || actual.attributes != mizar_checker::type_checker::AttributeSet::empty()
                        || target.attributes != mizar_checker::type_checker::AttributeSet::empty()
                        || diagnostic.site != Some(decision.site)
                        || diagnostic.class != OverloadDiagnosticClass::Viability
                        || diagnostic.message_key.as_str() != "overload.viability.missing_evidence"
                    {
                        return Err(invalid());
                    }
                }
                rejected += 1;
            }
            _ => return Err(invalid()),
        }
    }
    if viability.decisions().len() != collection.sites().len()
        || viability.diagnostics().len() != rejected
    {
        return Err(invalid());
    }
    Ok(if rejected == 0 {
        Vec::new()
    } else {
        vec!["predicates.application.argument_type_mismatch".into()]
    })
}

pub(in crate::runner) fn step5c5_predicate_semantics_detail_keys(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let definitions = surface_nodes_with_kind(ast, SurfaceNodeKind::PredicateDefinition);
    if definitions.is_empty() {
        return None;
    }
    let properties = surface_nodes_with_kind(ast, SurfaceNodeKind::PropertyClause);
    let invalid = || vec!["type_elaboration.checker.step5c5.predicate.invalid_payload".to_owned()];
    let [(definition_id, definition)] = definitions.as_slice() else {
        return Some(invalid());
    };
    if subtree_has_recovery(ast, definition)
        || !authenticated_local_declaration(
            ast,
            module,
            shells,
            symbols,
            *definition_id,
            definition,
            DeclarationShellKind::PredicateDefinition,
            SymbolKind::Predicate,
            DefinitionKind::Predicate,
            true,
        )
        || properties.is_empty()
        || properties.iter().any(|(id, property)| {
            subtree_has_recovery(ast, property)
                || !authenticated_local_declaration(
                    ast,
                    module,
                    shells,
                    symbols,
                    *id,
                    property,
                    DeclarationShellKind::PropertyClause,
                    SymbolKind::Attribute,
                    DefinitionKind::Attribute,
                    true,
                )
        })
    {
        return Some(invalid());
    }
    let Some(pattern) = structural_child_ids(ast, definition)
        .into_iter()
        .find_map(|id| {
            ast.node(id)
                .filter(|node| matches!(node.kind, SurfaceNodeKind::PredicatePattern))
        })
    else {
        return Some(invalid());
    };
    let parameters = surface_nodes_with_kind(ast, SurfaceNodeKind::QualifiedVariableSegment)
        .into_iter()
        .flat_map(|(_, segment)| direct_token_texts(ast, segment))
        .take_while(|token| token != "be" && token != "being")
        .filter(|token| token != ",")
        .collect::<Vec<_>>();
    let arity = leaf_token_texts(ast, pattern)
        .iter()
        .filter(|token| parameters.iter().any(|parameter| parameter == *token))
        .count();
    let property_kinds = properties
        .iter()
        .filter_map(|(_, property)| direct_token_texts(ast, property).into_iter().next())
        .collect::<Vec<_>>();
    if !matches!(property_kinds.as_slice(), [kind] if kind == "symmetry")
        && !matches!(property_kinds.as_slice(), [first, second] if first == "symmetry" && second == "reflexivity")
    {
        return Some(invalid());
    }
    let normalized_parameters = surface_nodes_with_kind(ast, SurfaceNodeKind::DefinitionParameter)
        .into_iter()
        .flat_map(|(_, parameter)| structural_child_ids(ast, parameter))
        .filter_map(|id| ast.node(id).map(|node| (id, node)))
        .filter(|(_, node)| matches!(node.kind, SurfaceNodeKind::QualifiedVariableSegment))
        .filter_map(|(_, segment)| {
            structural_child_ids(ast, segment)
                .into_iter()
                .find_map(|id| ast.node(id).map(|node| (id, node)))
        })
        .filter(|(_, node)| matches!(node.kind, SurfaceNodeKind::TypeExpression))
        .filter_map(|(id, node)| {
            super::source_reserve::extract_builtin_source_type_expression(
                ast, node, module, symbols,
            )
            .ok()
            .map(|source_type| {
                TypeExpressionInput::new(
                    surface_site(id),
                    source_type.range,
                    source_type.spelling,
                    source_type.head,
                )
            })
        })
        .collect::<Vec<_>>();
    if normalized_parameters.is_empty()
        || !TypeNormalizer::default()
            .normalize(symbols, normalized_parameters)
            .diagnostics()
            .is_empty()
    {
        return Some(invalid());
    }
    Some(match arity {
        2 => Vec::new(),
        1 => vec!["predicates.property.arity_mismatch".to_owned()],
        _ => invalid(),
    })
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
    let exact = exact_predicate_definition(ast)?;
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

fn exact_predicate_definition(ast: &SurfaceAst) -> Option<ExactPredicateDefinition> {
    if ast.root().map(SurfaceNodeId::index) != Some(70)
        || ast.expression_root().is_some()
        || !surface_profile_is_exact(ast)
    {
        return None;
    }
    Some(ExactPredicateDefinition {
        definition_block: ast.node_views().nth(67)?.id(),
    })
}

fn surface_profile_is_exact(ast: &SurfaceAst) -> bool {
    ast.canonical_bytes().is_some()
        && ast.nodes().len() == EXACT_SURFACE_PROFILE.len()
        && ast
            .nodes()
            .iter()
            .zip(EXACT_SURFACE_PROFILE)
            .all(|(actual, expected)| {
                actual.range.start < actual.range.end
                    && actual.recovered == expected.recovered
                    && (matches!(actual.kind, SurfaceNodeKind::Token(_))
                        || actual
                            .children
                            .first()
                            .zip(actual.children.last())
                            .is_some_and(|(first, last)| {
                                ast.node(*first).zip(ast.node(*last)).is_some_and(
                                    |(first, last)| {
                                        actual.range.start == first.range.start
                                            && actual.range.end == last.range.end
                                    },
                                )
                            }))
                    && (matches!(actual.kind, SurfaceNodeKind::Root)
                        || actual.children.windows(2).all(|pair| {
                            ast.node(pair[0])
                                .zip(ast.node(pair[1]))
                                .is_some_and(|(left, right)| left.range.end <= right.range.start)
                        }))
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

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
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
                mizar_lexer::TokenKind::Numeral => SurfaceTokenKind::Numeral,
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
        return Err("Task259 source association is invalid".to_owned());
    }
    let arena = task259_arena(ast)?;
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

    let mut type_input = task249_input(ast, module.clone());
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

    let mut term_input = task252_input(ast, module.clone());
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

    let mut atomic_input = task256_input(ast, module.clone());
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

    let mut predicate_input = task259_input(ast, module.clone(), symbol, definition, contribution);
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
        || block.range() != ast.nodes()[67].range
        || block.module() != module
        || block.recovered()
        || predicate.id().index() != 1
        || predicate.ordinal() != 1
        || predicate.kind() != DeclarationShellKind::PredicateDefinition
        || predicate.node_id().index() != 62
        || predicate.parent() != Some(block.id())
        || predicate.range() != ast.nodes()[62].range
        || predicate.module() != module
        || predicate.recovered()
        || property.id().index() != 2
        || property.ordinal() != 2
        || property.kind() != DeclarationShellKind::PropertyClause
        || property.node_id().index() != 66
        || property.parent() != Some(block.id())
        || property.range() != ast.nodes()[66].range
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
        || definition.arity() != Some(2)
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
        ast.nodes()[62].range,
        module,
        definition.origin(),
        &[4, 0, 8, 0],
    )?;
    validate_origin(
        ast.nodes()[62].range,
        module,
        symbol.origin(),
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
        ast.nodes()[66].range,
        module,
        property_definition.origin(),
        &[4, 0, 17, 1],
    )?;
    validate_origin(
        ast.nodes()[66].range,
        module,
        property_symbol.origin(),
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
    expected_range: SourceRange,
    module: &ModuleId,
    origin: &mizar_resolve::resolved_ast::SemanticOrigin,
    structural_path: &[u32],
) -> Result<(), String> {
    if origin.source_id() != expected_range.source_id
        || origin.module_id() != module
        || origin.anchor() != &SourceAnchor::Range(expected_range)
        || origin.structural_path() != structural_path
        || origin.import_edge().is_some()
        || origin.is_recovered()
    {
        return Err("Task259 resolver: predicate origin is not exact".to_owned());
    }
    Ok(())
}

fn task259_arena(ast: &SurfaceAst) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    push_node(
        &mut builder,
        "source.type.head",
        ast.nodes()[38].range,
        LocalTypeContextId::new(1),
        Vec::new(),
    )?;
    push_node(
        &mut builder,
        "source.type.expression",
        ast.nodes()[39].range,
        LocalTypeContextId::new(1),
        vec![TYPE_HEAD_X],
    )?;
    push_node(
        &mut builder,
        "source.definition.predicate.parameter",
        ast.nodes()[2].range,
        LocalTypeContextId::new(1),
        vec![TYPE_EXPRESSION_X],
    )?;
    push_node(
        &mut builder,
        "source.type.head",
        ast.nodes()[42].range,
        LocalTypeContextId::new(1),
        Vec::new(),
    )?;
    push_node(
        &mut builder,
        "source.type.expression",
        ast.nodes()[43].range,
        LocalTypeContextId::new(1),
        vec![TYPE_HEAD_Y],
    )?;
    push_node(
        &mut builder,
        "source.definition.predicate.parameter",
        ast.nodes()[7].range,
        LocalTypeContextId::new(1),
        vec![TYPE_EXPRESSION_Y],
    )?;
    for (id, range) in [
        (GUARD_LEFT, ast.nodes()[46].range),
        (GUARD_RIGHT, ast.nodes()[48].range),
        (DEFINIENS_LEFT, ast.nodes()[55].range),
        (DEFINIENS_RIGHT, ast.nodes()[57].range),
    ] {
        let actual = push_node(
            &mut builder,
            "source.term.variable-reference",
            range,
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
                ast.nodes()[50].range,
                LocalTypeContextId::new(1),
                vec![GUARD_LEFT, GUARD_RIGHT],
            )?;
        }
        if id == DEFINIENS_RIGHT {
            push_node(
                &mut builder,
                "source.formula.atomic.equality",
                ast.nodes()[59].range,
                LocalTypeContextId::new(1),
                vec![DEFINIENS_LEFT, DEFINIENS_RIGHT],
            )?;
        }
    }
    push_node(
        &mut builder,
        "source.definition.predicate.guard",
        ast.nodes()[53].range,
        LocalTypeContextId::new(1),
        vec![GUARD_FORMULA],
    )?;
    push_node(
        &mut builder,
        "source.definition.predicate",
        ast.nodes()[62].range,
        LocalTypeContextId::new(1),
        vec![DEFINIENS_FORMULA],
    )?;
    push_node(
        &mut builder,
        "source.definition.predicate.property",
        ast.nodes()[66].range,
        LocalTypeContextId::new(1),
        Vec::new(),
    )?;
    push_node(
        &mut builder,
        "source.definition",
        ast.nodes()[67].range,
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
        ast.nodes()[70].range,
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

fn task249_input(ast: &SurfaceAst, module_id: ModuleId) -> SourceTypeHandoffInput {
    let source_id = ast.source_id;
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
                ast.nodes()[39].range,
            ),
            bare_set_type(
                source_id,
                module_id,
                TYPE_EXPRESSION_Y,
                TYPE_HEAD_Y,
                ast.nodes()[43].range,
            ),
        ],
        arguments: Vec::new(),
    }
}

fn bare_set_type(
    source_id: SourceId,
    module_id: ModuleId,
    site: TypedNodeId,
    head_site: TypedNodeId,
    written_range: SourceRange,
) -> SourceTypeExpressionInput {
    SourceTypeExpressionInput {
        source_id,
        module_id,
        site: TypedSiteRef::Node(site),
        source_range: written_range,
        spelling: "set".to_owned(),
        head_site: TypedSiteRef::Node(head_site),
        head_range: written_range,
        head_spelling: "set".to_owned(),
        form: SourceTypeApplicationForm::Bare,
        head: SourceTypeHead::BuiltinSet,
        recovery: NodeRecoveryState::Normal,
    }
}

fn task252_input(ast: &SurfaceAst, module_id: ModuleId) -> SourcePrimaryTermHandoffInput {
    let source_id = ast.source_id;
    let specs = [
        (GUARD_LEFT, ast.nodes()[46].range, "x", BindingId::new(0)),
        (GUARD_RIGHT, ast.nodes()[48].range, "x", BindingId::new(0)),
        (
            DEFINIENS_LEFT,
            ast.nodes()[55].range,
            "x",
            BindingId::new(0),
        ),
        (
            DEFINIENS_RIGHT,
            ast.nodes()[57].range,
            "y",
            BindingId::new(1),
        ),
    ];
    SourcePrimaryTermHandoffInput {
        source_id,
        module_id,
        terms: specs
            .iter()
            .enumerate()
            .map(
                |(source_ordinal, (site, range, spelling, _))| SourcePrimaryTermInput {
                    site: TypedSiteRef::Node(*site),
                    source_range: *range,
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
                |(index, (_, _, _, binding))| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(index),
                    binding: *binding,
                    role: SourcePrimaryTermReferenceRole::Variable,
                },
            )
            .collect(),
        numeric_type_requests: Vec::new(),
    }
}

fn task256_input(ast: &SurfaceAst, module_id: ModuleId) -> SourceAtomicFormulaHandoffInput {
    let source_id = ast.source_id;
    let formulas = vec![
        SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(GUARD_FORMULA),
            source_range: ast.nodes()[50].range,
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: "x = x".to_owned(),
            kind: SourceAtomicFormulaKind::Equality,
        },
        SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(DEFINIENS_FORMULA),
            source_range: ast.nodes()[59].range,
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
    ast: &SurfaceAst,
    module_id: ModuleId,
    symbol: SymbolId,
    definition: DefinitionId,
    contribution: SourceContributionId,
) -> SourcePredicateDefinitionHandoffInput {
    let source_id = ast.source_id;
    SourcePredicateDefinitionHandoffInput {
        source_id,
        module_id,
        definitions: vec![SourcePredicateDefinitionInput {
            symbol,
            definition,
            contribution,
            site: TypedSiteRef::Node(PREDICATE_OWNER),
            source_range: ast.nodes()[62].range,
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
                source_range: ast.nodes()[41].range,
                declaration_range: ast.nodes()[2].range,
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
                source_range: ast.nodes()[45].range,
                declaration_range: ast.nodes()[7].range,
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
            source_range: ast.nodes()[53].range,
            context: BindingContextId::new(1),
            recovery: SourcePredicateDefinitionRecovery::Normal,
            spelling: "assume x = x;".to_owned(),
        }],
        properties: vec![SourcePredicatePropertyInput {
            owner: SourcePredicateDefinitionId::new(0),
            ordinal: 0,
            kind: SourcePredicatePropertyKind::Symmetry,
            site: TypedSiteRef::Node(PROPERTY_OWNER),
            source_range: ast.nodes()[66].range,
            justification: SourceAnchor::Range(ast.nodes()[65].range),
            recovery: SourcePredicateDefinitionRecovery::Normal,
            spelling: "symmetry by computation(steps: 1);".to_owned(),
        }],
        correctness: vec![SourcePredicateCorrectnessInput {
            owner: SourcePredicateDefinitionId::new(0),
            property: SourcePredicatePropertyId::new(0),
            ordinal: 0,
            source_anchor: SourceAnchor::Range(ast.nodes()[66].range),
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
