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

use super::{
    checker_handoff::{assemble_empty_resolved_typed_ast, source_syntax_receiver},
    source_ast::{
        authenticated_local_declaration, direct_token_texts, leaf_token_texts,
        structural_child_ids, subtree_has_recovery, surface_nodes_with_kind, surface_site,
    },
};

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

pub(in crate::runner) fn step5c3_functor_argument_detail_keys(
    source: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
    symbols: &SymbolEnv,
    typed: &TypedArena,
) -> Result<Vec<String>, String> {
    use mizar_checker::overload_resolution::{
        CandidateRejectionReason, CandidateViabilityStatus, OverloadDiagnosticClass,
        OverloadResultStatus,
    };
    use mizar_checker::type_checker::{
        NormalizedTypeStatus, TypeHeadRef, check_source_distinct_loci_overloads,
    };
    let (normalization, collection, expansion, viability, _, selection, _) =
        check_source_distinct_loci_overloads(source, symbols, typed, true, None)?;
    let invalid = || "types.application.unsupported_argument_evidence".to_owned();
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
        if candidate.site != decision.site
            || collection.sites().get(decision.site).is_none()
            || candidate.symbol != candidate.ordinary_root
            || !symbols.symbols().iter().any(|entry| {
                entry.symbol() == &candidate.symbol && entry.kind() == SymbolKind::Functor
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
                let [reason] = reasons.as_slice() else {
                    return Err(invalid());
                };
                let actual = normalization
                    .normalized_types()
                    .get(reason.actual.ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
                let target = normalization
                    .normalized_types()
                    .get(reason.target.ok_or_else(invalid)?)
                    .ok_or_else(invalid)?;
                let [diagnostic] = decision.diagnostics.as_slice() else {
                    return Err(invalid());
                };
                let diagnostic = viability
                    .diagnostics()
                    .get(*diagnostic)
                    .ok_or_else(invalid)?;
                if reason.argument_index != 0
                    || reason.reason != CandidateRejectionReason::MissingEvidence
                    || candidate.parameters.as_slice() != [target.id]
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
        vec!["types.application.argument_type_mismatch".into()]
    })
}

pub(in crate::runner) fn step5c5_functor_duplicate_detail_keys(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    resolver_keys: &[String],
) -> Option<Vec<String>> {
    let definitions = surface_nodes_with_kind(ast, SurfaceNodeKind::FunctorDefinition);
    let predicates = surface_nodes_with_kind(ast, SurfaceNodeKind::PredicateDefinition);
    if definitions.is_empty() && !predicates.is_empty() {
        use mizar_resolve::{
            declarations::DeclarationShellCollector,
            env::DeclarationConflictClass,
            symbols::{SymbolCollector, SymbolDiagnosticClass},
        };
        let [left, right] = predicates.as_slice() else {
            return None;
        };
        let items = super::source_ast::exact_compilation_item_list(ast)?;
        if subtree_has_recovery(ast, items)
            || ast
                .nodes()
                .iter()
                .any(|node| node.range.source_id != ast.source_id)
            || structural_child_ids(ast, items).iter().any(|id| {
                ast.node(*id)
                    .is_none_or(|node| node.kind != SurfaceNodeKind::DefinitionBlockItem)
            })
            || shells.declarations().iter().any(|shell| {
                !matches!(
                    shell.kind(),
                    DeclarationShellKind::DefinitionBlock
                        | DeclarationShellKind::PredicateDefinition
                )
            })
            || resolver_keys != ["declaration_symbol.signature.same_signature_definition_conflict"]
        {
            return None;
        }
        let actual_shells = DeclarationShellCollector::new(ast, module).collect();
        let projections = SignatureProjectionExtractor::new(
            ast,
            &actual_shells,
            NamespacePath::new(module.path().as_str()),
        )
        .extract();
        let actual =
            SymbolCollector::new(ast.source_id, module, &actual_shells, &projections).collect();
        let [diagnostic] = actual.diagnostics() else {
            return None;
        };
        if &actual_shells != shells
            || actual.env() != symbols
            || symbols.symbols().len() != 2
            || symbols.definitions().len() != 2
            || diagnostic.class() != SymbolDiagnosticClass::SameSignatureDefinitionConflict
            || diagnostic.range() != left.1.range
            || diagnostic.candidates().len() != 2
            || !diagnostic
                .candidates()
                .iter()
                .zip([left, right])
                .all(|(symbol, (id, node))| {
                    authenticated_local_declaration(
                        ast,
                        module,
                        shells,
                        symbols,
                        *id,
                        node,
                        DeclarationShellKind::PredicateDefinition,
                        SymbolKind::Predicate,
                        DefinitionKind::Predicate,
                        false,
                    ) && symbols
                        .definitions()
                        .by_symbol(symbol)
                        .is_some_and(|definition| {
                            definition.origin().anchor() == &SourceAnchor::Range(node.range)
                                && definition.conflict()
                                    == Some(
                                        &DeclarationConflictClass::SameSignatureDefinitionConflict,
                                    )
                        })
                })
        {
            return None;
        }
        return Some(vec![
            "predicates.definition.duplicate_same_signature".to_owned(),
        ]);
    }
    let [left, right] = definitions.as_slice() else {
        return None;
    };
    let patterns = definitions
        .iter()
        .filter_map(|(_, definition)| {
            structural_child_ids(ast, definition)
                .into_iter()
                .find_map(|id| {
                    ast.node(id)
                        .filter(|node| matches!(node.kind, SurfaceNodeKind::FunctorPattern))
                })
        })
        .map(|pattern| leaf_token_texts(ast, pattern))
        .collect::<Vec<_>>();
    let returns = definitions
        .iter()
        .filter_map(|(_, definition)| {
            structural_child_ids(ast, definition)
                .into_iter()
                .find_map(|id| {
                    ast.node(id)
                        .filter(|node| matches!(node.kind, SurfaceNodeKind::TypeExpression))
                })
        })
        .map(|node| leaf_token_texts(ast, node))
        .collect::<Vec<_>>();
    let parameter_types = surface_nodes_with_kind(ast, SurfaceNodeKind::DefinitionParameter)
        .into_iter()
        .filter_map(|(_, parameter)| {
            structural_child_ids(ast, parameter)
                .into_iter()
                .filter_map(|id| ast.node(id))
                .find(|node| matches!(node.kind, SurfaceNodeKind::QualifiedVariableSegment))
        })
        .filter_map(|segment| {
            structural_child_ids(ast, segment)
                .into_iter()
                .filter_map(|id| ast.node(id))
                .find(|node| matches!(node.kind, SurfaceNodeKind::TypeExpression))
        })
        .map(|node| leaf_token_texts(ast, node))
        .collect::<Vec<_>>();
    if resolver_keys != ["declaration_symbol.signature.same_signature_definition_conflict"]
        || ![left, right].into_iter().all(|(id, node)| {
            authenticated_local_declaration(
                ast,
                module,
                shells,
                symbols,
                *id,
                node,
                DeclarationShellKind::FunctorDefinition,
                SymbolKind::Functor,
                DefinitionKind::Functor,
                false,
            )
        })
        || !matches!(patterns.as_slice(), [first, second] if first == second)
        || !matches!(returns.as_slice(), [first, second] if first == second)
        || parameter_types.as_slice() != [vec!["set"], vec!["set"]]
    {
        return None;
    }
    Some(vec![
        "functors.definition.duplicate_same_signature".to_owned(),
    ])
}

pub(in crate::runner) fn step5c5_functor_semantics_detail_keys(
    ast: &SurfaceAst,
    module: &ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    let definitions = surface_nodes_with_kind(ast, SurfaceNodeKind::FunctorDefinition);
    if definitions.is_empty() {
        let applications = surface_nodes_with_kind(ast, SurfaceNodeKind::ApplicationTerm)
            .into_iter()
            .filter(|(_, node)| direct_token_texts(ast, node).as_slice() == ["[", ",", "]"])
            .collect::<Vec<_>>();
        if applications.is_empty() {
            return None;
        }
        let invalid =
            || vec!["type_elaboration.checker.step5c5.functor.invalid_payload".to_owned()];
        let theorems = surface_nodes_with_kind(ast, SurfaceNodeKind::TheoremItem);
        let [(theorem_id, theorem)] = theorems.as_slice() else {
            return Some(invalid());
        };
        if applications.len() != 4
            || subtree_has_recovery(ast, theorem)
            || applications.iter().any(|(_, application)| {
                subtree_has_recovery(ast, application)
                    || leaf_token_texts(ast, application).as_slice() != ["[", "x", ",", "y", "]"]
                    || structural_child_ids(ast, application)
                        .into_iter()
                        .filter_map(|id| ast.node(id))
                        .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
                        .count()
                        != 2
            })
            || leaf_token_texts(ast, theorem)
                .iter()
                .filter(|token| token.as_str() == "=")
                .count()
                != 2
            || !authenticated_local_declaration(
                ast,
                module,
                shells,
                symbols,
                *theorem_id,
                theorem,
                DeclarationShellKind::Theorem,
                SymbolKind::Theorem,
                DefinitionKind::Theorem,
                true,
            )
        {
            return Some(invalid());
        }
        let binding_shapes = [
            SurfaceNodeKind::QuantifierVariableSegment,
            SurfaceNodeKind::QualifiedVariableSegment,
        ]
        .into_iter()
        .flat_map(|kind| surface_nodes_with_kind(ast, kind))
        .map(|(_, node)| leaf_token_texts(ast, node))
        .collect::<Vec<_>>();
        if binding_shapes.as_slice()
            != [
                vec!["x", ",", "y", "being", "object"],
                vec!["x", ",", "y", "be", "object"],
            ]
        {
            return Some(invalid());
        }
        let type_inputs = surface_nodes_with_kind(ast, SurfaceNodeKind::TypeExpression)
            .into_iter()
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
        return Some(
            if type_inputs.len() == 2
                && TypeNormalizer::default()
                    .normalize(symbols, type_inputs)
                    .diagnostics()
                    .is_empty()
            {
                Vec::new()
            } else {
                invalid()
            },
        );
    }
    let invalid = || vec!["type_elaboration.checker.step5c5.functor.invalid_payload".to_owned()];
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
            DeclarationShellKind::FunctorDefinition,
            SymbolKind::Functor,
            DefinitionKind::Functor,
            true,
        )
    {
        return Some(invalid());
    }
    let direct = direct_token_texts(ast, definition);
    let properties = surface_nodes_with_kind(ast, SurfaceNodeKind::PropertyClause);
    let parameters = surface_nodes_with_kind(ast, SurfaceNodeKind::QualifiedVariableSegment)
        .into_iter()
        .flat_map(|(_, segment)| direct_token_texts(ast, segment))
        .take_while(|token| token != "be" && token != "being")
        .filter(|token| token != ",")
        .collect::<Vec<_>>();
    let arity = structural_child_ids(ast, definition)
        .into_iter()
        .find_map(|id| {
            ast.node(id)
                .filter(|node| matches!(node.kind, SurfaceNodeKind::FunctorPattern))
        })
        .map_or(0, |pattern| {
            leaf_token_texts(ast, pattern)
                .iter()
                .filter(|token| parameters.iter().any(|parameter| parameter == *token))
                .count()
        });
    if direct.iter().any(|token| token == "means") {
        let formula_definiens = surface_nodes_with_kind(ast, SurfaceNodeKind::FormulaDefiniens);
        if !properties.is_empty()
            || !matches!(formula_definiens.as_slice(), [(_, node)] if !subtree_has_recovery(ast, node))
            || arity != 1
        {
            return Some(invalid());
        }
        let conditions = surface_nodes_with_kind(ast, SurfaceNodeKind::CorrectnessCondition)
            .into_iter()
            .filter_map(|(_, condition)| direct_token_texts(ast, condition).into_iter().next())
            .collect::<Vec<_>>();
        return Some(match conditions.as_slice() {
            [existence, uniqueness] if existence == "existence" && uniqueness == "uniqueness" => {
                Vec::new()
            }
            [] => vec!["functors.means.missing_correctness".to_owned()],
            _ => invalid(),
        });
    }
    if !direct.iter().any(|token| token == "equals") {
        return Some(invalid());
    }
    if !properties.is_empty() {
        let property_kinds = properties
            .iter()
            .filter_map(|(_, property)| direct_token_texts(ast, property).into_iter().next())
            .collect::<Vec<_>>();
        if property_kinds.as_slice() != ["commutativity"]
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
        return Some(match arity {
            2 => Vec::new(),
            1 => vec!["functors.property.arity_mismatch".to_owned()],
            _ => invalid(),
        });
    }
    let result_type_matches = (|| {
        let children = structural_child_ids(ast, definition);
        let (return_id, return_node) = children.iter().find_map(|id| {
            ast.node(*id)
                .filter(|node| matches!(node.kind, SurfaceNodeKind::TypeExpression))
                .map(|node| (*id, node))
        })?;
        let definiens = children.iter().find_map(|id| {
            ast.node(*id)
                .filter(|node| matches!(node.kind, SurfaceNodeKind::TermDefiniens))
        })?;
        let parameter_name = leaf_token_texts(ast, definiens)
            .into_iter()
            .find(|token| token != ";")?;
        let (parameter_id, parameter_node) =
            surface_nodes_with_kind(ast, SurfaceNodeKind::QualifiedVariableSegment)
                .into_iter()
                .find(|(_, segment)| {
                    direct_token_texts(ast, segment)
                        .iter()
                        .take_while(|token| token.as_str() != "be" && token.as_str() != "being")
                        .any(|token| token == &parameter_name)
                })
                .and_then(|(_, segment)| {
                    structural_child_ids(ast, segment)
                        .into_iter()
                        .find_map(|id| {
                            ast.node(id)
                                .filter(|node| matches!(node.kind, SurfaceNodeKind::TypeExpression))
                                .map(|node| (id, node))
                        })
                })?;
        let parameter = super::source_reserve::extract_builtin_source_type_expression(
            ast,
            parameter_node,
            module,
            symbols,
        )
        .ok()?;
        let result = super::source_reserve::extract_builtin_source_type_expression(
            ast,
            return_node,
            module,
            symbols,
        )
        .ok()?;
        let matches = parameter.head == result.head;
        TypeNormalizer::default()
            .normalize(
                symbols,
                [
                    TypeExpressionInput::new(
                        surface_site(parameter_id),
                        parameter.range,
                        parameter.spelling,
                        parameter.head,
                    ),
                    TypeExpressionInput::new(
                        surface_site(return_id),
                        result.range,
                        result.spelling,
                        result.head,
                    ),
                ],
            )
            .diagnostics()
            .is_empty()
            .then_some(matches)
    })();
    Some(match result_type_matches {
        Some(true) => Vec::new(),
        Some(false) => vec!["functors.equals.result_type_mismatch".to_owned()],
        None => invalid(),
    })
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
    let exact = exact_functor_definition(ast, &module, source_text)?;
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

fn exact_functor_definition(
    ast: &SurfaceAst,
    module: &ModuleId,
    source_text: &str,
) -> Option<ExactFunctorDefinition> {
    if !surface_nodes_with_kind(ast, SurfaceNodeKind::TermCase).is_empty() {
        let blocks = surface_nodes_with_kind(ast, SurfaceNodeKind::DefinitionBlockItem);
        let [(definition_block, block)] = blocks.as_slice() else {
            return None;
        };
        let definitions = structural_child_ids(ast, block)
            .into_iter()
            .filter(|id| {
                ast.node(*id)
                    .is_some_and(|node| node.kind == SurfaceNodeKind::FunctorDefinition)
            })
            .collect::<Vec<_>>();
        let [definition] = definitions.as_slice() else {
            return None;
        };
        let guarded = structural_child_ids(ast, ast.node(*definition)?)
            .into_iter()
            .any(|id| {
                ast.node(id).is_some_and(|body| {
                    body.kind == SurfaceNodeKind::TermDefiniens
                        && structural_child_ids(ast, body).into_iter().any(|case| {
                            ast.node(case)
                                .is_some_and(|node| node.kind == SurfaceNodeKind::TermCase)
                        })
                })
            });
        return guarded.then_some(ExactFunctorDefinition {
            definition_block: *definition_block,
        });
    }
    if ast.root().map(SurfaceNodeId::index) != Some(107) || ast.expression_root().is_some() {
        return None;
    }
    let preprocessed = mizar_lexer::preprocess_source_for_lexing(source_text);
    if !preprocessed.diagnostics.is_empty() {
        return None;
    }
    let raw = mizar_lexer::scan_raw(&preprocessed.lexical_text).ok()?;
    let environment = mizar_lexer::build_lexical_environment(&[], &[]).ok()?;
    let locals = mizar_lexer::collect_local_lexical_declarations(
        &raw,
        mizar_lexer::ModuleId::new(module.path().as_str()),
    );
    let scopes = mizar_lexer::build_scope_skeleton(&raw);
    if !scopes.diagnostics.is_empty() {
        return None;
    }
    let native = mizar_lexer::disambiguate_with_local_declarations(
        &raw,
        &environment,
        &locals,
        &mizar_lexer::ParserLexContext::general(),
        &scopes,
    );
    if !native.diagnostics().is_empty() || native.tokens().len() != 62 {
        return None;
    }
    let mut mapped = Vec::with_capacity(62);
    for (token, expected) in native.tokens().iter().zip(EXACT_SURFACE_PROFILE) {
        let kind = match token.kind() {
            mizar_lexer::TokenKind::ReservedWord => SurfaceTokenKind::ReservedWord,
            mizar_lexer::TokenKind::ReservedSymbol => SurfaceTokenKind::ReservedSymbol,
            mizar_lexer::TokenKind::Identifier => SurfaceTokenKind::Identifier,
            mizar_lexer::TokenKind::Numeral => SurfaceTokenKind::Numeral,
            _ => return None,
        };
        if !matches!(&expected.kind, ExactSurfaceKind::Token(expected_kind, text)
            if kind == *expected_kind && token.lexeme() == *text)
        {
            return None;
        }
        let fragments = preprocessed
            .preprocess_map
            .source_ranges_for_lexical(token.span())?;
        let [span] = fragments.as_slice() else {
            return None;
        };
        if source_text.get(span.start..span.end) != Some(token.lexeme()) {
            return None;
        }
        mapped.push(*span);
    }
    let m = mapped[0].start;
    let d = mapped[1].start.checked_sub(13)?;
    let is_layout = |text: &str| {
        text.bytes()
            .all(|byte| matches!(byte, b' ' | b'\t' | b'\n'))
    };
    // Native prefix support remains bounded independently of the lower Type producer.
    if d > 20
        || m > d
        || d.checked_add(262)? != source_text.len()
        || !is_layout(source_text.get(..m)?)
        || source_text.get(d + 13..) != SOURCE_FUNCTOR_DEFINITION_TEXT.get(13..)
        || !surface_profile_is_exact(ast, &mapped, m, d)
    {
        return None;
    }
    if preprocessed.comments.is_empty() {
        if m != d || source_text.get(m..d + 13) != SOURCE_FUNCTOR_DEFINITION_TEXT.get(..13) {
            return None;
        }
    } else {
        let mut previous_end = mapped[0].end;
        for comment in &preprocessed.comments {
            if !matches!(
                comment.kind,
                mizar_lexer::CommentKind::SingleLine | mizar_lexer::CommentKind::MultiLine
            ) || comment.span.start < previous_end
                || comment.span.end < comment.span.start
                || comment.span.end > mapped[1].start
                || !is_layout(source_text.get(previous_end..comment.span.start)?)
            {
                return None;
            }
            previous_end = comment.span.end;
        }
        if !is_layout(source_text.get(previous_end..mapped[1].start)?) {
            return None;
        }
    }

    let root = exact_node(ast, 107, SurfaceNodeKind::Root)?;
    let definition_block = exact_node(ast, 104, SurfaceNodeKind::DefinitionBlockItem)?;
    let equals = exact_node(ast, 84, SurfaceNodeKind::FunctorDefinition)?;
    let means = exact_node(ast, 95, SurfaceNodeKind::FunctorDefinition)?;
    let existence = exact_node(ast, 99, SurfaceNodeKind::CorrectnessCondition)?;
    let uniqueness = exact_node(ast, 103, SurfaceNodeKind::CorrectnessCondition)?;
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

fn surface_profile_is_exact(
    ast: &SurfaceAst,
    mapped: &[mizar_lexer::SourceSpan],
    m: usize,
    d: usize,
) -> bool {
    ast.nodes().len() == EXACT_SURFACE_PROFILE.len()
        && ast.token_nodes().iter().map(|id| id.index()).eq(0..62)
        && ast
            .nodes()
            .iter()
            .enumerate()
            .zip(EXACT_SURFACE_PROFILE)
            .all(|((index, actual), expected)| {
                actual.range
                    == match index {
                        0 => source_range(ast.source_id, m, m + 10),
                        104..=107 => source_range(ast.source_id, m, d + 261),
                        _ => source_range(ast.source_id, d + expected.start, d + expected.end),
                    }
                    && !actual.recovered
                    && actual.children.len() == expected.children.len()
                    && actual
                        .children
                        .iter()
                        .zip(expected.children)
                        .all(|(actual, expected)| actual.index() == *expected)
                    && match (&actual.kind, &expected.kind) {
                        (
                            SurfaceNodeKind::Token(token),
                            ExactSurfaceKind::Token(expected_kind, expected_text),
                        ) => {
                            token.kind == *expected_kind
                                && token.text.as_ref() == *expected_text
                                && mapped.get(index).is_some_and(|span| {
                                    (actual.range.start, actual.range.end) == (span.start, span.end)
                                })
                        }
                        (kind, ExactSurfaceKind::Structural(expected_kind)) => {
                            kind == expected_kind
                        }
                        _ => false,
                    }
            })
}

fn exact_node(ast: &SurfaceAst, index: usize, kind: SurfaceNodeKind) -> Option<SurfaceNodeId> {
    let view = ast.node_views().find(|view| view.id().index() == index)?;
    (*view.kind() == kind && !view.is_recovered()).then_some(view.id())
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
        || block.range() != ast.nodes()[104].range
        || block.module() != module
        || block.recovered()
        || equals.id().index() != 1
        || equals.ordinal() != 1
        || equals.kind() != DeclarationShellKind::FunctorDefinition
        || equals.node_id().index() != 84
        || equals.parent() != Some(block.id())
        || equals.range() != ast.nodes()[84].range
        || equals.module() != module
        || equals.recovered()
        || means.id().index() != 2
        || means.ordinal() != 2
        || means.kind() != DeclarationShellKind::FunctorDefinition
        || means.node_id().index() != 95
        || means.parent() != Some(block.id())
        || means.range() != ast.nodes()[95].range
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
                    || actual.arity() != Some(1)
                    || actual.overload_policy() != SymbolOverloadPolicy::Overloadable
            })
    {
        return Err("Task260 resolver: parser-backed projections are not exact".to_owned());
    }
    let mut result = Vec::with_capacity(2);
    for (index, (expected, owner, path)) in [
        (expected_spellings[0], 84, &[4, 0, 9, 0][..]),
        (expected_spellings[1], 95, &[4, 0, 9, 1][..]),
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
            || definition.arity() != Some(1)
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
        validate_origin(ast.nodes()[owner].range, module, definition.origin(), path)?;
        validate_origin(ast.nodes()[owner].range, module, symbol.origin(), path)?;
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
        || contribution.anchor() != &SourceAnchor::Range(ast.nodes()[104].range)
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
    range: SourceRange,
    module: &ModuleId,
    origin: &mizar_resolve::resolved_ast::SemanticOrigin,
    structural_path: &[u32],
) -> Result<(), String> {
    if origin.source_id() != range.source_id
        || origin.module_id() != module
        || origin.anchor() != &SourceAnchor::Range(range)
        || origin.structural_path() != structural_path
        || origin.import_edge().is_some()
        || origin.is_recovered()
    {
        return Err("Task260 resolver: functor origin is not exact".to_owned());
    }
    Ok(())
}

fn task260_arena(ast: &SurfaceAst) -> Result<TypedArena, String> {
    let mut builder = TypedArenaBuilder::new();
    for (index, row) in ast.nodes().iter().enumerate() {
        let generic_kind = match row.kind {
            SurfaceNodeKind::Token(_) => "source.surface.token",
            _ => "source.surface.structural",
        };
        let kind = match index {
            62 => "source.type.head",
            63 => "source.type.expression",
            65 => "source.definition.functor.parameter",
            66 => "source.type.head",
            67 => "source.type.expression",
            69 => "source.definition.functor.parameter",
            70 => "source.term.variable-reference",
            72 => "source.term.variable-reference",
            75 => "source.formula.atomic.equality",
            77 => "source.definition.functor.guard",
            79 => "source.type.head",
            80 => "source.type.expression",
            81 => "source.term.variable-reference",
            83 => "source.definition.functor.definiens",
            84 => "source.definition.functor",
            86 => "source.type.head",
            87 => "source.type.expression",
            88 => "source.term.variable-reference",
            90 => "source.term.variable-reference",
            93 => "source.formula.atomic.equality",
            94 => "source.definition.functor.definiens",
            95 => "source.definition.functor",
            99 => "source.definition.functor.correctness",
            103 => "source.definition.functor.correctness",
            104 => "source.definition",
            107 => "source.module",
            _ => generic_kind,
        };
        let children = row
            .children
            .iter()
            .map(|id| TypedNodeId::new(id.index()))
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
                    SourceAnchor::Range(match index {
                        65 => ast.nodes()[2].range,
                        69 => ast.nodes()[7].range,
                        _ => row.range,
                    }),
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

fn source_spelling(ast: &SurfaceAst, source_text: &str, index: usize) -> String {
    let range = ast.nodes()[index].range;
    source_text
        .get(range.start..range.end)
        .expect("authenticated Functor source range")
        .to_owned()
}

fn task248_context(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    source_text: &str,
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
            source_range: ast.nodes()[104].range,
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
                spelling: source_spelling(ast, source_text, 2),
                declaration_range: ast.nodes()[2].range,
                written_type_range: ast.nodes()[63].range,
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
                        source_spelling(ast, source_text, 2),
                        scope.clone(),
                        ast.nodes()[2].range,
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
                spelling: source_spelling(ast, source_text, 7),
                declaration_range: ast.nodes()[7].range,
                written_type_range: ast.nodes()[67].range,
                site: if mutation
                    == SourceFunctorDefinitionRouteMutation::WrongContextBindingSite(1)
                {
                    TypedSiteRef::Node(TypedNodeId::new(65))
                } else {
                    TypedSiteRef::Node(TypedNodeId::new(69))
                },
                role: SourceBindingSiteRole::DefinitionParameter {
                    local: LocalTermBinding::new(
                        source_spelling(ast, source_text, 7),
                        scope,
                        ast.nodes()[7].range,
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

fn task249_input(
    ast: &SurfaceAst,
    source_text: &str,
    module_id: ModuleId,
) -> SourceTypeHandoffInput {
    SourceTypeHandoffInput {
        source_id: ast.source_id,
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
            bare_set_type(ast, source_text, module_id.clone(), 63, 62),
            bare_set_type(ast, source_text, module_id, 67, 66),
        ],
        arguments: Vec::new(),
    }
}

fn task249r_input(
    ast: &SurfaceAst,
    source_text: &str,
    module_id: ModuleId,
) -> SourceTypeDefinitionReturnExtensionInput {
    SourceTypeDefinitionReturnExtensionInput {
        source_id: ast.source_id,
        module_id: module_id.clone(),
        returns: vec![
            SourceTypeDefinitionReturnInput {
                definition_site: TypedSiteRef::Node(TypedNodeId::new(84)),
                definition_range: ast.nodes()[84].range,
                source_ordinal: 0,
                expression: bare_set_type(ast, source_text, module_id.clone(), 80, 79),
            },
            SourceTypeDefinitionReturnInput {
                definition_site: TypedSiteRef::Node(TypedNodeId::new(95)),
                definition_range: ast.nodes()[95].range,
                source_ordinal: 1,
                expression: bare_set_type(ast, source_text, module_id, 87, 86),
            },
        ],
    }
}

fn bare_set_type(
    ast: &SurfaceAst,
    source_text: &str,
    module_id: ModuleId,
    site: usize,
    head_site: usize,
) -> SourceTypeExpressionInput {
    SourceTypeExpressionInput {
        source_id: ast.source_id,
        module_id,
        site: TypedSiteRef::Node(TypedNodeId::new(site)),
        source_range: ast.nodes()[site].range,
        spelling: source_spelling(ast, source_text, site),
        head_site: TypedSiteRef::Node(TypedNodeId::new(head_site)),
        head_range: ast.nodes()[head_site].range,
        head_spelling: source_spelling(ast, source_text, head_site),
        form: SourceTypeApplicationForm::Bare,
        head: SourceTypeHead::BuiltinSet,
        recovery: NodeRecoveryState::Normal,
    }
}

fn task252_input(
    ast: &SurfaceAst,
    source_text: &str,
    module_id: ModuleId,
) -> SourcePrimaryTermHandoffInput {
    let specs = [
        (70, BindingId::new(0)),
        (72, BindingId::new(0)),
        (81, BindingId::new(0)),
        (88, BindingId::new(0)),
        (90, BindingId::new(1)),
    ];
    SourcePrimaryTermHandoffInput {
        source_id: ast.source_id,
        module_id,
        terms: specs
            .iter()
            .enumerate()
            .map(|(source_ordinal, (site, _))| SourcePrimaryTermInput {
                site: TypedSiteRef::Node(TypedNodeId::new(*site)),
                source_range: ast.nodes()[*site].range,
                source_ordinal,
                context: BindingContextId::new(1),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: source_spelling(ast, source_text, *site),
                kind: SourcePrimaryTermKind::VariableReference,
                role: SourcePrimaryTermRole::Value,
                parent: None,
            })
            .collect(),
        references: specs
            .iter()
            .enumerate()
            .map(|(index, (_, binding))| SourcePrimaryTermReferenceInput {
                term: SourcePrimaryTermId::new(index),
                binding: *binding,
                role: SourcePrimaryTermReferenceRole::Variable,
            })
            .collect(),
        numeric_type_requests: Vec::new(),
    }
}

fn task256_input(
    ast: &SurfaceAst,
    source_text: &str,
    module_id: ModuleId,
) -> SourceAtomicFormulaHandoffInput {
    let formulas = vec![
        SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(TypedNodeId::new(75)),
            source_range: ast.nodes()[75].range,
            source_ordinal: 0,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: source_spelling(ast, source_text, 75),
            kind: SourceAtomicFormulaKind::Equality,
        },
        SourceAtomicFormulaInput {
            site: TypedSiteRef::Node(TypedNodeId::new(93)),
            source_range: ast.nodes()[93].range,
            source_ordinal: 1,
            context: BindingContextId::new(1),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: source_spelling(ast, source_text, 93),
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
        source_id: ast.source_id,
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
    ast: &SurfaceAst,
    source_text: &str,
    module_id: ModuleId,
    resolver: [(SymbolId, DefinitionId, SourceContributionId); 2],
) -> SourceFunctorDefinitionHandoffInput {
    SourceFunctorDefinitionHandoffInput {
        source_id: ast.source_id,
        module_id,
        definitions: vec![
            SourceFunctorDefinitionInput {
                symbol: resolver[0].0.clone(),
                definition: resolver[0].1,
                contribution: resolver[0].2,
                site: TypedSiteRef::Node(TypedNodeId::new(84)),
                source_range: ast.nodes()[84].range,
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: source_spelling(ast, source_text, 84),
                style: SourceFunctorDefinitionStyle::Equals,
                return_type: SourceTypeDefinitionReturnId::new(0),
                definiens: SourceFunctorDefiniensId::new(0),
            },
            SourceFunctorDefinitionInput {
                symbol: resolver[1].0.clone(),
                definition: resolver[1].1,
                contribution: resolver[1].2,
                site: TypedSiteRef::Node(TypedNodeId::new(95)),
                source_range: ast.nodes()[95].range,
                source_ordinal: 1,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: source_spelling(ast, source_text, 95),
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
                source_range: ast.nodes()[65].range,
                declaration_range: ast.nodes()[2].range,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: source_spelling(ast, source_text, 65),
            },
            SourceFunctorParameterInput {
                ordinal: 1,
                binding: BindingId::new(1),
                written_type: SourceTypeApplicationId::new(1),
                site: TypedSiteRef::Node(TypedNodeId::new(69)),
                source_range: ast.nodes()[69].range,
                declaration_range: ast.nodes()[7].range,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: source_spelling(ast, source_text, 69),
            },
        ],
        guards: vec![SourceFunctorGuardInput {
            ordinal: 0,
            formula: SourceAtomicFormulaId::new(0),
            site: TypedSiteRef::Node(TypedNodeId::new(77)),
            source_range: ast.nodes()[77].range,
            context: BindingContextId::new(1),
            recovery: SourceFunctorDefinitionRecovery::Normal,
            spelling: source_spelling(ast, source_text, 77),
        }],
        definientia: vec![
            SourceFunctorDefiniensInput {
                owner: SourceFunctorDefinitionId::new(0),
                ordinal: 0,
                target: SourceFunctorDefiniensTarget::Primary(SourcePrimaryTermId::new(2)),
                site: TypedSiteRef::Node(TypedNodeId::new(83)),
                source_range: ast.nodes()[83].range,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: source_spelling(ast, source_text, 83),
            },
            SourceFunctorDefiniensInput {
                owner: SourceFunctorDefinitionId::new(1),
                ordinal: 1,
                target: SourceFunctorDefiniensTarget::AtomicFormula(SourceAtomicFormulaId::new(1)),
                site: TypedSiteRef::Node(TypedNodeId::new(94)),
                source_range: ast.nodes()[94].range,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: source_spelling(ast, source_text, 94),
            },
        ],
        correctness: vec![
            SourceFunctorCorrectnessInput {
                owner: SourceFunctorDefinitionId::new(1),
                ordinal: 0,
                kind: SourceFunctorCorrectnessKind::Existence,
                site: TypedSiteRef::Node(TypedNodeId::new(99)),
                source_range: ast.nodes()[99].range,
                justification: SourceAnchor::Range(ast.nodes()[98].range),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: source_spelling(ast, source_text, 99),
            },
            SourceFunctorCorrectnessInput {
                owner: SourceFunctorDefinitionId::new(1),
                ordinal: 1,
                kind: SourceFunctorCorrectnessKind::Uniqueness,
                site: TypedSiteRef::Node(TypedNodeId::new(103)),
                source_range: ast.nodes()[103].range,
                justification: SourceAnchor::Range(ast.nodes()[102].range),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: source_spelling(ast, source_text, 103),
            },
        ],
    }
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    exact: ExactFunctorDefinition,
    mutation: SourceFunctorDefinitionRouteMutation,
) -> Result<SourceFunctorDefinitionRouteOutput, String> {
    let guarded = !surface_nodes_with_kind(ast, SurfaceNodeKind::TermCase).is_empty();
    if guarded && mutation != SourceFunctorDefinitionRouteMutation::None {
        return Err("Task257B4 does not use the exact260 corruption profile".to_owned());
    }
    let corrupted_symbols = matches!(
        mutation,
        SourceFunctorDefinitionRouteMutation::WrongResolverEntry
            | SourceFunctorDefinitionRouteMutation::WrongResolverDefinitionEntry
            | SourceFunctorDefinitionRouteMutation::WrongResolverContribution
    )
    .then(|| resolver_env_with_mutation(symbols, mutation));
    let symbols = corrupted_symbols.as_ref().unwrap_or(symbols);
    let syntax = source_syntax_receiver(ast, &module)?;
    let (
        arena,
        source_context,
        mut type_input,
        mut return_input,
        mut term_input,
        mut atomic_input,
        mut functor_input,
    ) = if guarded {
        use mizar_resolve::{names::resolve_template_formal, resolved_ast::SurfaceResolvedArena};
        use mizar_syntax::SurfaceNodeKind as K;
        let invalid = || "Task257B4 actual guarded source inputs are incomplete".to_owned();
        let rows = syntax
            .source_node_entries()
            .ok_or_else(invalid)?
            .collect::<Vec<_>>();
        let preprocessed = mizar_lexer::preprocess_source_for_lexing(source_text);
        if !preprocessed.diagnostics.is_empty() {
            return Err(invalid());
        }
        let raw_tokens =
            mizar_lexer::scan_raw(&preprocessed.lexical_text).map_err(|_| invalid())?;
        let environment =
            mizar_lexer::build_lexical_environment(&[], &[]).map_err(|_| invalid())?;
        let locals = mizar_lexer::collect_local_lexical_declarations(
            &raw_tokens,
            mizar_lexer::ModuleId::new(module.path().as_str()),
        );
        let scopes = mizar_lexer::build_scope_skeleton(&raw_tokens);
        if !scopes.diagnostics.is_empty() {
            return Err(invalid());
        }
        let lexical = mizar_lexer::disambiguate_with_local_declarations(
            &raw_tokens,
            &environment,
            &locals,
            &mizar_lexer::ParserLexContext::general(),
            &scopes,
        );
        let mut original_tokens = rows
            .iter()
            .filter_map(|row| match row.kind() {
                K::Token(token) => Some((row.range(), token)),
                _ => None,
            })
            .collect::<Vec<_>>();
        original_tokens.sort_by_key(|(range, _)| (range.start, range.end));
        if !lexical.diagnostics().is_empty() || lexical.tokens().len() != original_tokens.len() {
            return Err(invalid());
        }
        for (actual, (range, expected)) in lexical.tokens().iter().zip(original_tokens) {
            let kind = match actual.kind() {
                mizar_lexer::TokenKind::ReservedWord => SurfaceTokenKind::ReservedWord,
                mizar_lexer::TokenKind::ReservedSymbol => SurfaceTokenKind::ReservedSymbol,
                mizar_lexer::TokenKind::Identifier => SurfaceTokenKind::Identifier,
                mizar_lexer::TokenKind::Numeral => SurfaceTokenKind::Numeral,
                _ => return Err(invalid()),
            };
            let fragments = preprocessed
                .preprocess_map
                .source_ranges_for_lexical(actual.span())
                .ok_or_else(invalid)?;
            let [span] = fragments.as_slice() else {
                return Err(invalid());
            };
            if kind != expected.kind
                || actual.lexeme() != expected.text.as_ref()
                || (span.start, span.end) != (range.start, range.end)
                || source_text.get(span.start..span.end) != Some(actual.lexeme())
            {
                return Err(invalid());
            }
        }
        let one = |kind: K| {
            let found = rows
                .iter()
                .copied()
                .filter(|row| row.kind() == &kind)
                .collect::<Vec<_>>();
            if let [row] = found.as_slice() {
                Ok(*row)
            } else {
                Err(invalid())
            }
        };
        let children = |parent| {
            rows.iter()
                .copied()
                .filter(|row| row.parent() == Some(parent))
                .collect::<Vec<_>>()
        };
        let site = |row: &mizar_checker::typed_ast::SourceNode| {
            TypedSiteRef::Node(TypedNodeId::new(row.site().index()))
        };
        let raw = |row: &mizar_checker::typed_ast::SourceNode| {
            source_text
                .get(row.range().start..row.range().end)
                .map(str::to_owned)
                .ok_or_else(invalid)
        };
        let definition = one(K::FunctorDefinition)?;
        let block_row = one(K::DefinitionBlockItem)?;
        let block = shells
            .declarations()
            .iter()
            .find(|shell| {
                shell.kind() == DeclarationShellKind::DefinitionBlock
                    && shell.range() == block_row.range()
            })
            .ok_or_else(invalid)?;
        let declaration = symbols
            .definitions()
            .iter()
            .find(|row| row.origin().anchor() == &SourceAnchor::Range(definition.range()))
            .ok_or_else(invalid)?;
        let parameters = rows
            .iter()
            .copied()
            .filter(|row| row.kind() == &K::DefinitionParameter)
            .collect::<Vec<_>>();
        let mut formals = Vec::new();
        let mut types = Vec::new();
        for parameter in &parameters {
            let segment = children(parameter.site())
                .into_iter()
                .find(|row| row.kind() == &K::QualifiedVariableSegment)
                .ok_or_else(invalid)?;
            formals.push(
                children(segment.site())
                    .into_iter()
                    .find(|row| {
                        matches!(row.kind(), K::Token(token)
                if token.kind == SurfaceTokenKind::Identifier)
                    })
                    .ok_or_else(invalid)?,
            );
            types.push(
                children(segment.site())
                    .into_iter()
                    .find(|row| row.kind() == &K::TypeExpression)
                    .ok_or_else(invalid)?,
            );
        }
        let result = children(definition.site())
            .into_iter()
            .find(|row| row.kind() == &K::TypeExpression)
            .ok_or_else(invalid)?;
        let cases = rows
            .iter()
            .copied()
            .filter(|row| row.kind() == &K::TermCase)
            .collect::<Vec<_>>();
        let body = one(K::TermDefiniens)?;
        let fallback = children(body.site())
            .into_iter()
            .find(|row| row.kind() == &K::TermExpression)
            .ok_or_else(invalid)?;
        let formulas = cases
            .iter()
            .map(|row| {
                children(row.site())
                    .into_iter()
                    .find(|row| row.kind() == &K::FormulaExpression)
                    .ok_or_else(invalid)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let correctness = one(K::CorrectnessCondition)?;
        let proof = children(correctness.site())
            .into_iter()
            .find(|row| row.kind() == &K::ProofBlock)
            .ok_or_else(invalid)?;
        let mut term_rows = rows
            .iter()
            .copied()
            .filter(|row| row.kind() == &K::TermReference)
            .collect::<Vec<_>>();
        term_rows.sort_by_key(|row| row.range().start);
        let native = SurfaceResolvedArena::lower(ast, &module).map_err(|_| invalid())?;
        native
            .validate_against(ast, &module)
            .map_err(|_| invalid())?;
        let pattern = children(definition.site())
            .into_iter()
            .find(|row| row.kind() == &K::FunctorPattern)
            .ok_or_else(invalid)?;
        let loci = children(pattern.site());
        let [_, _, left, _, right, _] = loci.as_slice() else {
            return Err(invalid());
        };
        for (index, locus) in [left, right].into_iter().enumerate() {
            if resolve_template_formal(&native, locus.site()).map_err(|_| invalid())?
                != formals.get(index).ok_or_else(invalid)?.site()
            {
                return Err(invalid());
            }
        }
        let bindings = term_rows
            .iter()
            .map(|row| {
                let token = children(row.site())
                    .into_iter()
                    .next()
                    .ok_or_else(invalid)?;
                let formal =
                    resolve_template_formal(&native, token.site()).map_err(|_| invalid())?;
                formals
                    .iter()
                    .position(|row| row.site() == formal)
                    .map(BindingId::new)
                    .ok_or_else(invalid)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut builder = TypedArenaBuilder::new();
        for (index, row) in ast.nodes().iter().enumerate() {
            let kind = match &row.kind {
                K::TypeHead => "source.type.head",
                K::TypeExpression => "source.type.expression",
                K::DefinitionParameter => "source.definition.functor.parameter",
                K::TermReference => "source.term.variable-reference",
                K::FormulaExpression if formulas.iter().any(|row| row.site().index() == index) => {
                    "source.formula.atomic.equality"
                }
                K::TermCase => "source.definition.functor.definiens",
                K::TermExpression if fallback.site().index() == index => {
                    "source.definition.functor.definiens"
                }
                K::FunctorDefinition => "source.definition.functor",
                K::CorrectnessCondition => "source.definition.functor.correctness",
                K::DefinitionBlockItem => "source.definition",
                K::Root => "source.module",
                K::Token(_) => "source.surface.token",
                _ => "source.surface.structural",
            };
            let anchor = parameters
                .iter()
                .position(|parameter| parameter.site().index() == index)
                .map_or(row.range, |index| formals[index].range());
            builder
                .push(
                    TypedNode::new(kind, SourceAnchor::Range(anchor))
                        .with_children(
                            row.children
                                .iter()
                                .map(|child| TypedNodeId::new(child.index()))
                                .collect(),
                        )
                        .with_typing(TypingState::Unknown)
                        .with_recovery(if row.recovered {
                            NodeRecoveryState::Recovered
                        } else {
                            NodeRecoveryState::Normal
                        })
                        .with_links(TypedNodeLinks {
                            context: Some(LocalTypeContextId::new(usize::from(
                                row.kind != K::Root,
                            ))),
                            ..Default::default()
                        }),
                )
                .map_err(|_| invalid())?;
        }
        let arena = builder
            .finish(ast.root().map(|root| TypedNodeId::new(root.index())))
            .map_err(|_| invalid())?;
        let scope = LocalTermScope::new(vec![0]);
        let source_context = SourceBindingContextProducer::build(SourceBindingContextInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            module_site: TypedSiteRef::Node(arena.root().ok_or_else(invalid)?),
            items: vec![SourceItemInput {
                shell: block.id(),
                shell_ordinal: block.ordinal(),
                role: SourceItemRole::DefinitionBlock,
                module_id: module.clone(),
                source_range: block.range(),
                parent: None,
                visibility: SourceItemVisibility::Unspecified,
                site: site(block_row),
                local_scope: Some(scope.clone()),
                recovery: SourceItemRecovery::Normal,
            }],
            bindings: formals
                .iter()
                .enumerate()
                .map(|(index, formal)| SourceBindingSiteInput {
                    shell: block.id(),
                    context_owner: SourceBindingContextOwner::Shell(block.id()),
                    source_ordinal: index,
                    spelling: formal.spelling().to_owned(),
                    declaration_range: formal.range(),
                    written_type_range: types[index].range(),
                    site: site(parameters[index]),
                    role: SourceBindingSiteRole::DefinitionParameter {
                        local: LocalTermBinding::new(
                            formal.spelling(),
                            scope.clone(),
                            formal.range(),
                            index,
                        ),
                    },
                    recovery: BindingRecoveryState::Normal,
                })
                .collect(),
        })
        .map_err(|_| invalid())?
        .into_complete()
        .map_err(|_| invalid())?
        .into_handoff();
        let type_expression = |row: &mizar_checker::typed_ast::SourceNode| {
            let head = children(row.site())
                .into_iter()
                .find(|row| row.kind() == &K::TypeHead)
                .ok_or_else(invalid)?;
            if row.spelling() != "set" || head.spelling() != "set" {
                return Err(invalid());
            }
            raw(row)?;
            raw(head)?;
            Ok::<_, String>(bare_set_type(
                ast,
                source_text,
                module.clone(),
                row.site().index(),
                head.site().index(),
            ))
        };
        let type_input = SourceTypeHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            applications: formals
                .iter()
                .enumerate()
                .map(|(index, _)| SourceTypeApplicationInput {
                    binding: BindingId::new(index),
                    source_ordinal: index,
                    root: SourceTypeExpressionId::new(index),
                })
                .collect(),
            expressions: types
                .iter()
                .map(|row| type_expression(row))
                .collect::<Result<Vec<_>, _>>()?,
            arguments: Vec::new(),
        };
        let return_input = SourceTypeDefinitionReturnExtensionInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            returns: vec![SourceTypeDefinitionReturnInput {
                definition_site: site(definition),
                definition_range: definition.range(),
                source_ordinal: 0,
                expression: type_expression(result)?,
            }],
        };
        let term_input = SourcePrimaryTermHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            terms: term_rows
                .iter()
                .enumerate()
                .map(|(index, row)| SourcePrimaryTermInput {
                    site: site(row),
                    source_range: row.range(),
                    source_ordinal: index,
                    context: BindingContextId::new(1),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: row.spelling().to_owned(),
                    kind: SourcePrimaryTermKind::VariableReference,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                })
                .collect(),
            references: bindings
                .iter()
                .enumerate()
                .map(|(index, binding)| SourcePrimaryTermReferenceInput {
                    term: SourcePrimaryTermId::new(index),
                    binding: *binding,
                    role: SourcePrimaryTermReferenceRole::Variable,
                })
                .collect(),
            numeric_type_requests: Vec::new(),
        };
        let term_id = |range| {
            term_rows
                .iter()
                .position(|row| row.range() == range)
                .map(SourcePrimaryTermId::new)
                .ok_or_else(invalid)
        };
        let mut edges = Vec::new();
        for (index, formula) in formulas.iter().enumerate() {
            let equality = children(formula.site())
                .into_iter()
                .find(|row| row.kind() == &K::BuiltinPredicateApplication)
                .ok_or_else(invalid)?;
            for (ordinal, operand) in children(equality.site())
                .into_iter()
                .filter(|row| row.kind() == &K::TermExpression)
                .enumerate()
            {
                edges.push(SourceAtomicEdgeInput {
                    formula: SourceAtomicFormulaId::new(index),
                    ordinal,
                    role: if ordinal == 0 {
                        SourceAtomicEdgeRole::BuiltinLeftOperand
                    } else {
                        SourceAtomicEdgeRole::BuiltinRightOperand
                    },
                    target: SourceAtomicTermTarget::Primary(term_id(operand.range())?),
                });
            }
        }
        let atomic_input = SourceAtomicFormulaHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            formulas: formulas
                .iter()
                .enumerate()
                .map(|(index, row)| SourceAtomicFormulaInput {
                    site: site(row),
                    source_range: row.range(),
                    source_ordinal: index,
                    context: BindingContextId::new(1),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: row.spelling().to_owned(),
                    kind: SourceAtomicFormulaKind::Equality,
                })
                .collect(),
            wrappers: Vec::new(),
            predicate_segments: Vec::new(),
            predicate_heads: Vec::new(),
            candidates: Vec::new(),
            type_sites: Vec::new(),
            attributes: Vec::new(),
            requests: edges
                .iter()
                .enumerate()
                .map(|(index, edge)| SourceAtomicRequestInput {
                    formula: edge.formula,
                    ordinal: edge.ordinal,
                    kind: SourceAtomicRequestKind::OperandExpectedType,
                    edge: Some(SourceAtomicEdgeId::new(index)),
                    candidate: None,
                    type_site: None,
                    attribute: None,
                })
                .collect(),
            edges,
        };
        let mut definientia = cases
            .iter()
            .enumerate()
            .map(|(index, row)| {
                let value = children(row.site())
                    .into_iter()
                    .find(|row| row.kind() == &K::TermExpression)
                    .ok_or_else(invalid)?;
                Ok(SourceFunctorDefiniensInput {
                    owner: SourceFunctorDefinitionId::new(0),
                    ordinal: index,
                    target: SourceFunctorDefiniensTarget::TermCase(
                        term_id(value.range())?,
                        SourceAtomicFormulaId::new(index),
                    ),
                    site: site(row),
                    source_range: row.range(),
                    context: BindingContextId::new(1),
                    recovery: SourceFunctorDefinitionRecovery::Normal,
                    spelling: row.spelling().to_owned(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        definientia.push(SourceFunctorDefiniensInput {
            owner: SourceFunctorDefinitionId::new(0),
            ordinal: definientia.len(),
            target: SourceFunctorDefiniensTarget::Otherwise(term_id(fallback.range())?),
            site: site(fallback),
            source_range: fallback.range(),
            context: BindingContextId::new(1),
            recovery: SourceFunctorDefinitionRecovery::Normal,
            spelling: fallback.spelling().to_owned(),
        });
        let functor_input = SourceFunctorDefinitionHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            definitions: vec![SourceFunctorDefinitionInput {
                symbol: declaration.symbol().clone(),
                definition: declaration.id(),
                contribution: declaration.contribution(),
                site: site(definition),
                source_range: definition.range(),
                source_ordinal: 0,
                context: BindingContextId::new(1),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: raw(definition)?,
                style: SourceFunctorDefinitionStyle::Equals,
                return_type: SourceTypeDefinitionReturnId::new(0),
                definiens: SourceFunctorDefiniensId::new(0),
            }],
            parameters: parameters
                .iter()
                .enumerate()
                .map(|(index, row)| {
                    Ok(SourceFunctorParameterInput {
                        ordinal: index,
                        binding: BindingId::new(index),
                        written_type: SourceTypeApplicationId::new(index),
                        site: site(row),
                        source_range: row.range(),
                        declaration_range: formals[index].range(),
                        context: BindingContextId::new(1),
                        recovery: SourceFunctorDefinitionRecovery::Normal,
                        spelling: raw(row)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?,
            guards: Vec::new(),
            definientia,
            correctness: vec![SourceFunctorCorrectnessInput {
                owner: SourceFunctorDefinitionId::new(0),
                ordinal: 0,
                kind: SourceFunctorCorrectnessKind::Consistency,
                site: site(correctness),
                source_range: correctness.range(),
                justification: SourceAnchor::Range(proof.range()),
                recovery: SourceFunctorDefinitionRecovery::Normal,
                spelling: raw(correctness)?,
            }],
        };
        (
            arena,
            source_context,
            type_input,
            return_input,
            term_input,
            atomic_input,
            functor_input,
        )
    } else {
        let resolver = exact_resolver_profile(ast, &module, shells, symbols, mutation)?;
        (
            task260_arena(ast)?,
            task248_context(ast, module.clone(), shells, source_text, exact, mutation)?,
            task249_input(ast, source_text, module.clone()),
            task249r_input(ast, source_text, module.clone()),
            task252_input(ast, source_text, module.clone()),
            task256_input(ast, source_text, module.clone()),
            task260_input(ast, source_text, module.clone(), resolver),
        )
    };
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
    if mutation == SourceFunctorDefinitionRouteMutation::WrongReturnType {
        return_input.returns[1].definition_site = TypedSiteRef::Node(TypedNodeId::new(84));
    }
    if mutation == SourceFunctorDefinitionRouteMutation::WrongReturnRange {
        return_input.returns[1].definition_range = ast.nodes()[84].range;
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
        &syntax,
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
    let guarded = functor.definitions().len() == 1;
    let (
        expression_count,
        return_count,
        term_count,
        definition_count,
        guard_count,
        definiens_count,
        correctness_count,
    ) = if guarded {
        (3, 1, 7, 1, 0, 3, 1)
    } else {
        (4, 2, 5, 2, 1, 2, 2)
    };
    context.items().len() == 1
        && context.declarations().len() == 2
        && context.binding_env().bindings().len() == 2
        && context.binding_env().contexts().len() == 2
        && context.local_contexts().len() == 2
        && context.context_links().len() == 2
        && context.binding_env().diagnostics().is_empty()
        && source_type.applications().len() == 2
        && source_type.expressions().len() == expression_count
        && source_type.arguments().is_empty()
        && source_type.definition_returns().len() == return_count
        && source_term.terms().len() == term_count
        && source_term.references().len() == term_count
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
        && functor.definitions().len() == definition_count
        && functor.parameters().len() == 2
        && functor.guards().len() == guard_count
        && functor.definientia().len() == definiens_count
        && functor.correctness().len() == correctness_count
        && typed.source_functor_definition() == resolved.source_functor_definition()
        && typed.source_context() == resolved.source_context()
        && typed.source_type() == resolved.source_type()
        && typed.source_term() == resolved.source_term()
        && typed.source_atomic_formula() == resolved.source_atomic_formula()
        && typed.initial_obligations().len() == correctness_count
        && (!guarded
            || (functor
                .correctness()
                .iter()
                .all(|(_, row)| row.kind() == SourceFunctorCorrectnessKind::Consistency)
                && typed.initial_obligations().iter().all(|(_, row)| {
                    row.kind == mizar_checker::typed_ast::InitialObligationKind::FunctorConsistency
                        && row.status == mizar_checker::typed_ast::InitialObligationStatus::Pending
                })))
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

#[cfg(test)]
mod predicate_duplicate_tests {
    use super::*;
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::DeclarationConflictClass,
        symbols::{SymbolCollector, SymbolDiagnosticClass},
    };
    use mizar_session::{ModulePath, PackageId};

    #[test]
    fn step5c5_predicate_duplicate_uses_actual_loci_and_conflict_provenance() {
        let exact = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/miz/fail/predicates/fail_type_elaboration_pred_duplicate_same_signature_001.miz"
        ));
        let (first, tail) = exact.split_once("\n\ndefinition\n").unwrap();
        let second = format!("definition\n{tail}");
        let heterogeneous = first.replace("let X, Y be set;", "let X be set; let Y be object;");
        let module = ModuleId::new(
            PackageId::new("predicate-source"),
            ModulePath::new("duplicate"),
        );
        let collect = |text: &str| {
            let output = crate::runner::formula_statement::step5c8_test_frontend(text);
            assert!(
                output.diagnostics.is_empty(),
                "{text}: {:?}",
                output.diagnostics
            );
            let ast = output.ast.unwrap();
            let shells = DeclarationShellCollector::new(&ast, &module).collect();
            let projections = SignatureProjectionExtractor::new(
                &ast,
                &shells,
                NamespacePath::new(module.path().as_str()),
            )
            .extract();
            let collected =
                SymbolCollector::new(ast.source_id, &module, &shells, &projections).collect();
            (ast, shells, collected)
        };
        let key =
            vec!["declaration_symbol.signature.same_signature_definition_conflict".to_owned()];
        let expected = Some(vec![
            "predicates.definition.duplicate_same_signature".to_owned(),
        ]);
        for (source, conflict) in [
            (exact.to_owned(), true),
            (
                format!("{first}\n{}", second.replace('X', "A").replace('Y', "B")),
                true,
            ),
            (
                exact
                    .replace("Dup1Def", "First")
                    .replace("Dup2Def", "Second")
                    .replace("means X = Y", "means X in Y")
                    .replace("means Y = X", "means Y = Y"),
                true,
            ),
            (format!("{second}\n{first}"), true),
            (
                format!("{first}\n{}", second.replace("X, Y be set", "Y, X be set")),
                true,
            ),
            (
                format!(
                    "{first}\n{}",
                    second.replace("let X, Y be set;", "let X be set; let Y be set;")
                ),
                true,
            ),
            (
                format!(
                    "{first}\n{}",
                    second.replace("let X, Y be set;", "let X be object; let Y be set;")
                ),
                false,
            ),
            (
                format!("{first}\n{}", second.replace("duppred", "otherpred")),
                false,
            ),
            (
                format!("{first}\ndefinition let X be set; pred duppred X means X=X; end;"),
                false,
            ),
            (
                format!(
                    "{heterogeneous}\n{}",
                    second.replace("let X, Y be set;", "let X be object; let Y be set;")
                ),
                false,
            ),
            (
                format!(
                    "{heterogeneous}\n{}",
                    second.replace("let X, Y be set;", "let Y be object; let X be set;")
                ),
                true,
            ),
            (
                format!(
                    "{heterogeneous}\n{}",
                    second
                        .replace("let X, Y be set;", "let X be object; let Y be set;")
                        .replace("X duppred Y", "Y duppred X")
                ),
                true,
            ),
            (
                format!(
                    "{first}\ndefinition let X,Y be set; func X duppred Y -> set equals X; end;"
                ),
                false,
            ),
        ] {
            let (ast, shells, collected) = collect(&source);
            let conflicts = collected
                .diagnostics()
                .iter()
                .filter(|d| d.class() == SymbolDiagnosticClass::SameSignatureDefinitionConflict)
                .collect::<Vec<_>>();
            assert_eq!(conflicts.len(), usize::from(conflict), "{source}");
            let mapped = step5c5_functor_duplicate_detail_keys(
                &ast,
                &module,
                &shells,
                collected.env(),
                &key,
            );
            if !conflict {
                assert_eq!(mapped, None, "{source}");
                continue;
            }
            assert_eq!(mapped, expected, "{source}");
            assert_eq!(collected.diagnostics().len(), 1);
            let definitions = surface_nodes_with_kind(&ast, SurfaceNodeKind::PredicateDefinition);
            let diagnostic = conflicts[0];
            assert_eq!(diagnostic.range(), definitions[0].1.range);
            assert_eq!(diagnostic.candidates().len(), 2);
            for (symbol, (_, node)) in diagnostic.candidates().iter().zip(definitions) {
                let definition = collected.env().definitions().by_symbol(symbol).unwrap();
                assert_eq!(definition.kind(), DefinitionKind::Predicate);
                assert_eq!(
                    definition.conflict(),
                    Some(&DeclarationConflictClass::SameSignatureDefinitionConflict)
                );
                assert_eq!(definition.origin().source_id(), ast.source_id);
                assert_eq!(definition.origin().module_id(), &module);
                assert_eq!(
                    definition.origin().anchor(),
                    &SourceAnchor::Range(node.range)
                );
            }
        }
        let functors = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/miz/fail/functors/fail_type_elaboration_func_duplicate_same_signature_001.miz"
        ));
        let mixed = functors.replacen(
            "  coherence;",
            "  coherence;\n  pred Extra: extra X means X=X;",
            1,
        );
        let (mixed_ast, mixed_shells, mixed_symbols) = collect(&mixed);
        assert_eq!(mixed_symbols.diagnostics().len(), 1);
        assert_eq!(
            step5c5_functor_duplicate_detail_keys(
                &mixed_ast,
                &module,
                &mixed_shells,
                mixed_symbols.env(),
                &key
            ),
            Some(vec![
                "functors.definition.duplicate_same_signature".to_owned()
            ])
        );
        let forward = format!(
            "{first}\ndefinition let X be set; pred X duppred Y means X=X; let Y be set; end;"
        );
        let (forward_ast, forward_shells, forward_symbols) = collect(&forward);
        assert!(
            forward_symbols
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.class()
                    != SymbolDiagnosticClass::SameSignatureDefinitionConflict)
        );
        assert_eq!(
            step5c5_functor_duplicate_detail_keys(
                &forward_ast,
                &module,
                &forward_shells,
                forward_symbols.env(),
                &key
            ),
            None
        );
        for unsupported in [
            "definition let X,Y be set; pred X duppred(Y) means X=Y; end;",
            "definition let T be type; let X,Y be T; pred X duppred[T] Y means X=Y; end;",
            "definition let X,Y be set; pred X duppred X means X=X; end;",
            "definition let X be set; pred X duppred Y means X=X; end;",
            "definition let X,Y be set; assume X=X; pred X duppred Y means X=Y; end;",
        ] {
            let source = format!("{first}\n{unsupported}");
            let (ast, shells, collected) = collect(&source);
            assert_eq!(
                step5c5_functor_duplicate_detail_keys(
                    &ast,
                    &module,
                    &shells,
                    collected.env(),
                    &key
                ),
                None,
                "{source}"
            );
        }
        let (ast, shells, collected) = collect(exact);
        let symbols = collected.env();
        for malformed in [
            exact.replace("means Y = X", "means Y ="),
            format!("{first}\ndefinition let X,Y be Missing; pred X duppred Y means X=Y; end;"),
            "definition let X,duppred be set; pred L1: X duppred Y means X=X; end; definition let A,B be set; pred L2: A B Y means A=A; end;".to_owned(),
            "definition let duppred,Y be set; pred L1: X duppred Y means Y=Y; end; definition let A,B be set; pred L2: X A B means A=A; end;".to_owned(),
        ] {
            let recovered = crate::runner::formula_statement::step5c8_test_frontend(&malformed);
            assert!(!recovered.diagnostics.is_empty(), "{malformed}");
            let recovered_ast = recovered.ast.unwrap();
            let recovered_shells = DeclarationShellCollector::new(&recovered_ast, &module).collect();
            let projections = SignatureProjectionExtractor::new(&recovered_ast, &recovered_shells, NamespacePath::new(module.path().as_str())).extract();
            let recovered_symbols = SymbolCollector::new(recovered_ast.source_id, &module, &recovered_shells, &projections).collect();
            assert_eq!(step5c5_functor_duplicate_detail_keys(&recovered_ast, &module, &recovered_shells, recovered_symbols.env(), &key), None, "{malformed}");
        }
        let mut foreign_source = ast.clone();
        use mizar_session::SessionIdAllocator;
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshot = crate::runner::shared::snapshot_id(0);
        ids.next_source_id(snapshot).unwrap();
        foreign_source.source_id = ids.next_source_id(snapshot).unwrap();
        assert_eq!(
            step5c5_functor_duplicate_detail_keys(&foreign_source, &module, &shells, symbols, &key),
            None
        );
        let foreign_module = ModuleId::new(PackageId::new("foreign"), ModulePath::new("duplicate"));
        assert_eq!(
            step5c5_functor_duplicate_detail_keys(&ast, &foreign_module, &shells, symbols, &key),
            None
        );
        assert_eq!(
            step5c5_functor_duplicate_detail_keys(
                &ast,
                &module,
                &DeclarationShellSet::default(),
                symbols,
                &key
            ),
            None
        );
        for keys in [vec![], vec![key[0].clone(), "unrelated".into()]] {
            assert_eq!(
                step5c5_functor_duplicate_detail_keys(&ast, &module, &shells, symbols, &keys),
                None
            );
        }
        let (_, _, stale) = collect(&exact.replace("duppred", "otherpred"));
        assert_eq!(
            step5c5_functor_duplicate_detail_keys(&ast, &module, &shells, stale.env(), &key),
            None
        );
        let altered = resolver_env_with_mutation(
            symbols,
            SourceFunctorDefinitionRouteMutation::WrongResolverContribution,
        );
        assert_ne!(&altered, symbols);
        assert_eq!(
            step5c5_functor_duplicate_detail_keys(&ast, &module, &shells, &altered, &key),
            None
        );
        let mut forged_definitions = DefinitionIndex::new();
        for definition in symbols.definitions().iter() {
            let mut shell = DefinitionShell::new(
                definition.symbol().clone(),
                definition.kind(),
                definition.origin().clone(),
                definition.contribution(),
            )
            .with_visibility(definition.visibility())
            .with_parameters(definition.parameters().to_vec())
            .with_binders(definition.binders().to_vec())
            .with_dependencies(definition.dependencies().to_vec())
            .with_conflict(DeclarationConflictClass::DuplicateSpelling);
            if let Some(arity) = definition.arity() {
                shell = shell.with_arity(arity);
            }
            if let Some(notation) = definition.notation_shape() {
                shell = shell.with_notation_shape(notation);
            }
            if let Some(doc) = definition.doc_attachment() {
                shell = shell.with_doc_attachment(doc.clone());
            }
            if let Some(signature) = definition.signature() {
                shell = shell.with_signature(signature.clone());
            }
            assert_eq!(forged_definitions.insert(shell), definition.id());
        }
        let forged = SymbolEnv::new(
            module.clone(),
            SymbolEnvIndexes {
                imports: symbols.imports().clone(),
                exports: symbols.exports().clone(),
                symbols: symbols.symbols().clone(),
                labels: symbols.labels().clone(),
                definitions: forged_definitions,
                overloads: symbols.overloads().clone(),
                registrations: symbols.registrations().clone(),
                lexical_summaries: symbols.lexical_summaries().clone(),
                namespace_graph: symbols.namespace_graph().clone(),
                declaration_dependencies: symbols.declaration_dependencies().clone(),
                contributions: symbols.contributions().clone(),
                module_summaries: symbols.module_summaries().clone(),
            },
        );
        assert_eq!(
            step5c5_functor_duplicate_detail_keys(&ast, &module, &shells, &forged, &key),
            None
        );
        let (unrelated_ast, unrelated_shells, unrelated) = collect(&format!(
            "{exact}\ntheorem Extra: for X being set holds X=X; theorem Extra: for X being set holds X=X;"
        ));
        assert!(unrelated.diagnostics().len() > 1);
        assert_eq!(
            step5c5_functor_duplicate_detail_keys(
                &unrelated_ast,
                &module,
                &unrelated_shells,
                unrelated.env(),
                &key
            ),
            None
        );
    }
}
