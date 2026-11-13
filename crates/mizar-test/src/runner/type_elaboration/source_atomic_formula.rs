use std::collections::BTreeMap;

use mizar_checker::{
    binding_env::{BindingContextId, BindingContextOwner, BindingEnv, BindingId},
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_atomic_formula::{
        SourceAssertionAttributeInput, SourceAssertionAttributePolarityInput,
        SourceAssertionTypeHead, SourceAssertionTypeSiteInput, SourceAtomicEdgeId,
        SourceAtomicEdgeInput, SourceAtomicEdgeRole, SourceAtomicFormulaHandoffInput,
        SourceAtomicFormulaId, SourceAtomicFormulaInput, SourceAtomicFormulaKind,
        SourceAtomicFormulaProducer, SourceAtomicFormulaRecovery, SourceAtomicRequestInput,
        SourceAtomicRequestKind, SourceAtomicTermTarget, SourceAtomicWrapperInput,
        SourcePredicateCandidateId, SourcePredicateCandidateInput, SourcePredicateHeadId,
        SourcePredicateHeadInput, SourcePredicateSegmentInput, SourcePredicateSegmentPolarityInput,
    },
    source_set_term::SourceSetTermHandoff,
    source_term::{
        SourceNumericTypeRequestInput, SourcePrimaryTermHandoff, SourcePrimaryTermHandoffInput,
        SourcePrimaryTermId, SourcePrimaryTermInput, SourcePrimaryTermKind,
        SourcePrimaryTermProducer, SourcePrimaryTermRecovery, SourcePrimaryTermReferenceInput,
        SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
    },
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedAst, TypedAstParts,
        TypedNode, TypedNodeId, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    env::{SymbolEnv, SymbolKind},
    resolved_ast::{ModuleId, SurfaceResolvedArena, SymbolId},
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNodeId, SurfaceNodeKind};

use super::{
    checker_handoff::{
        assemble_empty_resolved_typed_ast, source_module_binding_env, source_syntax_receiver,
    },
    source_application::{
        imported_source_application_output_with_source_term,
        imported_source_application_owned_node_kinds,
    },
    source_ast::{direct_token_texts, structural_child_ids, surface_site, surface_text},
    source_formula::{
        SourceBuiltinBinaryTermFormula, SourceBuiltinTypeAssertionFormula,
        SourceImportedAttributeAssertionFormula, SourceImportedPredicateChainFormula,
        SourceImportedPredicateFunctorFormula, SourceSetEnumerationFormula,
        extract_source_builtin_binary_term_formula, extract_source_builtin_type_assertion_formula,
        extract_source_imported_attribute_assertion_formula,
        extract_source_imported_non_empty_attribute_assertion_formula,
        extract_source_imported_predicate_chain_formula,
        extract_source_imported_predicate_functor_formula, extract_source_set_enumeration_formula,
    },
    source_set_term::{source_set_term_output_with_source_term, step5c7_set_output},
    source_term::SourceTermParts,
};

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.typed_ast_invalid";

#[derive(Debug)]
pub(in crate::runner) struct SourceAtomicFormulaRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

#[derive(Debug, Clone)]
enum ExactAtomicRoute {
    Binary(SourceBuiltinBinaryTermFormula),
    TypeAssertion(SourceBuiltinTypeAssertionFormula),
    Predicate(SourceImportedPredicateFunctorFormula),
    PredicateChain(SourceImportedPredicateChainFormula),
    Attribute {
        payload: SourceImportedAttributeAssertionFormula,
        negative: bool,
    },
    SetEnumeration(SourceSetEnumerationFormula),
}

/// Runs the bounded Task-256 source transaction and then leaves the existing
/// semantic route as the owner of the externally visible detail keys.
pub(in crate::runner) fn source_atomic_formula_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_atomic_formula_output_with_optional_source(ast, module, symbols, Some(source_text))
    {
        None => None,
        Some(Ok(output))
            if output.typed_ast.source_atomic_formula().is_some()
                && output.typed_ast.source_atomic_formula()
                    == output.resolved.source_atomic_formula()
                && output.typed_ast.source_term() == output.resolved.source_term()
                && output.typed_ast.source_application()
                    == output.resolved.source_application()
                && output.typed_ast.source_set_term() == output.resolved.source_set_term() =>
        {
            if output
                .typed_ast
                .source_atomic_formula()
                .is_some_and(|handoff| !handoff.predicate_segments().is_empty())
            {
                Some(Vec::new())
            } else {
                None
            }
        }
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_atomic_formula_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
) -> Option<Result<SourceAtomicFormulaRouteOutput, String>> {
    source_atomic_formula_output_with_mutation_impl(ast, module, symbols, None, |_| {})
}

pub(in crate::runner) fn source_atomic_formula_output_with_source(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceAtomicFormulaRouteOutput, String>> {
    source_atomic_formula_output_with_mutation_impl(ast, module, symbols, Some(source_text), |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_atomic_formula_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceAtomicFormulaHandoffInput),
) -> Option<Result<SourceAtomicFormulaRouteOutput, String>> {
    source_atomic_formula_output_with_mutation_impl(ast, module, symbols, None, mutate)
}

#[cfg(test)]
pub(in crate::runner) fn source_atomic_formula_output_with_source_and_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceAtomicFormulaHandoffInput),
) -> Option<Result<SourceAtomicFormulaRouteOutput, String>> {
    source_atomic_formula_output_with_mutation_impl(ast, module, symbols, Some(source_text), mutate)
}

fn source_atomic_formula_output_with_optional_source(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: Option<&str>,
) -> Option<Result<SourceAtomicFormulaRouteOutput, String>> {
    source_atomic_formula_output_with_mutation_impl(ast, module, symbols, source_text, |_| {})
}

fn source_atomic_formula_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: Option<&str>,
    mutate: impl FnOnce(&mut SourceAtomicFormulaHandoffInput),
) -> Option<Result<SourceAtomicFormulaRouteOutput, String>> {
    let route = exact_atomic_route(ast, &module, symbols, source_text)?;
    if let ExactAtomicRoute::PredicateChain(payload) = &route {
        let source_matches = || -> Option<()> {
            use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
            let source_text = source_text?;
            ast.canonical_bytes()?;
            if ast.expression_root().is_some() {
                return None;
            }
            let root = ast.root()?;
            let root_node = ast.node(root)?;
            let root_children = super::source_ast::structural_child_ids(ast, root_node);
            let [compilation] = root_children.as_slice() else {
                return None;
            };
            if !root_node.children.iter().copied().eq(ast
                .token_nodes()
                .iter()
                .copied()
                .chain([*compilation]))
            {
                return None;
            }
            let items = super::source_ast::exact_compilation_item_list(ast)?;
            let item_children = structural_child_ids(ast, items);
            let [import_id, _] = item_children.as_slice() else {
                return None;
            };
            let import_node = ast.node(*import_id)?;
            let import_children = structural_child_ids(ast, import_node);
            let [declaration_id] = import_children.as_slice() else {
                return None;
            };
            let declaration_node = ast.node(*declaration_id)?;
            let declaration_children = structural_child_ids(ast, declaration_node);
            let [path_id] = declaration_children.as_slice() else {
                return None;
            };
            let path_node = ast.node(*path_id)?;
            let path_segments = structural_child_ids(ast, path_node);
            let preprocessed = mizar_lexer::preprocess_source_for_lexing(source_text);
            if !preprocessed.diagnostics.is_empty() {
                return None;
            }
            let raw = mizar_lexer::scan_raw(&preprocessed.lexical_text).ok()?;
            let prelude = mizar_lexer::scan_import_prelude(&raw);
            let [import] = prelude.imports.as_slice() else {
                return None;
            };
            if !prelude.diagnostics.is_empty()
                || import.alias.is_some()
                || import.path.relative.is_some()
                || import.path.spelling != "parser.type_fixtures"
                || import.path.components.len() != path_segments.len()
            {
                return None;
            }
            for (span, expected) in [
                (import.span, declaration_node.range),
                (import.path.span, path_node.range),
            ] {
                let ranges = preprocessed
                    .preprocess_map
                    .source_ranges_for_lexical(span)?;
                if ranges.first()?.start != expected.start || ranges.last()?.end != expected.end {
                    return None;
                }
            }
            for (component, id) in import.path.components.iter().zip(&path_segments) {
                let ranges = preprocessed
                    .preprocess_map
                    .source_ranges_for_lexical(component.span)?;
                let [range] = ranges.as_slice() else {
                    return None;
                };
                let expected = ast.node(*id)?;
                if !matches!(expected.kind, K::PathSegment)
                    || range.start != expected.range.start
                    || range.end != expected.range.end
                    || source_text.get(expected.range.start..expected.range.end)
                        != Some(component.spelling.as_str())
                {
                    return None;
                }
            }
            let imports = prelude
                .imports
                .iter()
                .map(|import| mizar_lexer::ResolvedImport {
                    module_id: mizar_lexer::ModuleId::new(import.path.spelling.as_str()),
                })
                .collect::<Vec<_>>();
            let summaries = imports
                .iter()
                .enumerate()
                .map(|(ordinal, import)| mizar_lexer::ModuleLexicalSummary {
                    exported_symbols: crate::runner::import_fixtures::parse_only_fixture_symbols(
                        &import.module_id,
                    ),
                    module_id: import.module_id.clone(),
                    fingerprint: mizar_lexer::LexicalSummaryFingerprint::new((ordinal as u64) + 1),
                })
                .collect::<Vec<_>>();
            let lexical_env = mizar_lexer::build_lexical_environment(&imports, &summaries).ok()?;
            let scopes = mizar_lexer::build_scope_skeleton(&raw);
            let tokens = mizar_lexer::disambiguate(
                &raw,
                &lexical_env,
                &mizar_lexer::ParserLexContext::general(),
                &scopes,
            );
            if !tokens.diagnostics().is_empty() || tokens.tokens().len() != ast.token_nodes().len()
            {
                return None;
            }
            for (actual, supplied) in tokens.tokens().iter().zip(ast.token_views()) {
                let token = supplied.as_token()?;
                let kind = match actual.kind() {
                    mizar_lexer::TokenKind::Identifier => T::Identifier,
                    mizar_lexer::TokenKind::ReservedWord => T::ReservedWord,
                    mizar_lexer::TokenKind::ReservedSymbol => T::ReservedSymbol,
                    mizar_lexer::TokenKind::Numeral => T::Numeral,
                    mizar_lexer::TokenKind::UserSymbol => T::UserSymbol,
                    _ => return None,
                };
                let ranges = preprocessed
                    .preprocess_map
                    .source_ranges_for_lexical(actual.span())?;
                let [range] = ranges.as_slice() else {
                    return None;
                };
                if token.kind != kind
                    || token.text.as_ref() != actual.lexeme()
                    || supplied.range().start != range.start
                    || supplied.range().end != range.end
                    || source_text.get(range.start..range.end) != Some(actual.lexeme())
                {
                    return None;
                }
            }
            let mut reachable = vec![false; ast.nodes().len()];
            let mut pending = vec![root];
            while let Some(id) = pending.pop() {
                if std::mem::replace(reachable.get_mut(id.index())?, true) {
                    continue;
                }
                pending.extend(ast.node(id)?.children.iter().copied());
            }
            if ast.node_views().any(|node| !reachable[node.id().index()]) {
                return None;
            }
            let mut owned_tokens = Vec::new();
            for node in ast.node_views() {
                if node.is_recovered()
                    || source_text
                        .get(node.range().start..node.range().end)
                        .is_none()
                    || ast.range_contains_child_ranges(node.id()) != Some(true)
                {
                    return None;
                }
                if matches!(node.kind(), K::Token(_)) {
                    continue;
                }
                if node.id() == root {
                    if root_node.range.start != ast.node(*ast.token_nodes().first()?)?.range.start
                        || root_node.range.end != ast.node(*ast.token_nodes().last()?)?.range.end
                    {
                        return None;
                    }
                    continue;
                }
                let children = node.child_views().collect::<Vec<_>>();
                if children
                    .windows(2)
                    .any(|pair| pair[0].range().end > pair[1].range().start)
                    || children.first()?.range().start != node.range().start
                    || children.last()?.range().end != node.range().end
                {
                    return None;
                }
                let positions: &[usize] = match node.kind() {
                    K::ImportItem => &[0, 2],
                    K::ModulePath => &[1],
                    K::PathSegment | K::NumeralTerm => &[0],
                    K::TheoremItem => &[0, 1, 2, 4],
                    K::PredicateSegment
                        if payload.segment_sites[1].node().index() == node.id().index() =>
                    {
                        &[0, 1]
                    }
                    K::CompilationUnit
                    | K::ItemList
                    | K::ImportAliasDecl
                    | K::FormulaExpression
                    | K::PredicateApplication
                    | K::PredicateSegment
                    | K::PredicateHead
                    | K::QualifiedSymbol
                    | K::TermExpression => &[],
                    _ => return None,
                };
                if !children
                    .iter()
                    .enumerate()
                    .filter_map(|(index, child)| child.as_token().map(|_| index))
                    .eq(positions.iter().copied())
                {
                    return None;
                }
                owned_tokens.extend(
                    children
                        .iter()
                        .filter_map(|child| child.as_token().map(|_| child.id())),
                );
            }
            owned_tokens.sort();
            if owned_tokens != ast.token_nodes() {
                return None;
            }
            let shells =
                mizar_resolve::declarations::DeclarationShellCollector::new(ast, &module).collect();
            let replay = mizar_resolve::symbols::SignatureProjectionExtractor::new(
                ast,
                &shells,
                mizar_resolve::env::NamespacePath::new(module.path().as_str()),
            )
            .collect(&module);
            if !replay.diagnostics().is_empty() {
                return None;
            }
            let replay = crate::runner::import_fixtures::augment_type_elaboration_import_summaries(
                ast,
                &module,
                replay.into_env(),
            );
            (&replay == symbols).then_some(())
        };
        if source_matches().is_none() {
            return Some(Err("Task257C1 source association is invalid".to_owned()));
        }
    }
    Some(build_output(ast, module, symbols, route, mutate))
}

fn exact_atomic_route(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    source_text: Option<&str>,
) -> Option<ExactAtomicRoute> {
    if let Some(payload) = source_text
        .and_then(|_| extract_source_imported_predicate_chain_formula(ast, module, symbols))
    {
        return Some(ExactAtomicRoute::PredicateChain(payload));
    }
    if let Some(payload) = extract_source_builtin_binary_term_formula(ast) {
        return Some(ExactAtomicRoute::Binary(payload));
    }
    if let Some(payload) = extract_source_builtin_type_assertion_formula(ast, module, symbols) {
        return Some(ExactAtomicRoute::TypeAssertion(payload));
    }
    if let Some(payload) = extract_source_imported_predicate_functor_formula(ast, module, symbols) {
        return Some(ExactAtomicRoute::Predicate(payload));
    }
    if let Some(payload) = extract_source_imported_attribute_assertion_formula(ast, module, symbols)
    {
        return Some(ExactAtomicRoute::Attribute {
            payload,
            negative: false,
        });
    }
    if let Some(payload) =
        extract_source_imported_non_empty_attribute_assertion_formula(ast, module, symbols)
    {
        return Some(ExactAtomicRoute::Attribute {
            payload,
            negative: true,
        });
    }
    extract_source_set_enumeration_formula(ast).map(ExactAtomicRoute::SetEnumeration)
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    route: ExactAtomicRoute,
    mutate: impl FnOnce(&mut SourceAtomicFormulaHandoffInput),
) -> Result<SourceAtomicFormulaRouteOutput, String> {
    let binding_env =
        source_module_binding_env(ast, module.clone()).map_err(|error| error.to_string())?;
    let syntax = source_syntax_receiver(ast, &module)?;
    let formula_node = route.formula_node();
    let mut owned_kinds = route.atomic_owned_node_kinds(ast)?;
    if matches!(&route, ExactAtomicRoute::Predicate(_)) {
        let application_kinds = imported_source_application_owned_node_kinds(ast, &module, symbols)
            .ok_or_else(|| "Task256 imported application ownership disappeared".to_owned())?;
        for (node, kind) in application_kinds {
            insert_kind(&mut owned_kinds, node, kind)?;
        }
    }
    let primary_roots = route.primary_roots();
    let source_term = {
        let mut rows = primary_roots
            .iter()
            .map(|root| {
                syntax
                    .source_node_entries()
                    .unwrap()
                    .find(|row| row.site().index() == *root)
                    .ok_or_else(|| "Task256 generic numeral row disappeared".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        rows.sort_by_key(|row| row.preorder_ordinal());
        let mut terms = Vec::new();
        let mut numeric_type_requests = Vec::new();
        for (source_ordinal, row) in rows.iter().enumerate() {
            let site = TypedSiteRef::Node(TypedNodeId::new(row.site().index()));
            terms.push(SourcePrimaryTermInput {
                site: site.clone(),
                source_range: row.range(),
                source_ordinal,
                context: BindingContextId::new(0),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: row.spelling().to_owned(),
                kind: SourcePrimaryTermKind::Numeral,
                role: SourcePrimaryTermRole::Value,
                parent: None,
            });
            numeric_type_requests.push(SourceNumericTypeRequestInput {
                term: SourcePrimaryTermId::new(source_ordinal),
                owner: site,
                source_range: row.range(),
                spelling: row.spelling().to_owned(),
                request_ordinal: source_ordinal,
            });
        }
        let mut nodes = Vec::new();
        for (index, node) in ast.nodes().iter().enumerate() {
            let kind = if rows.iter().any(|row| row.site().index() == index) {
                Some("source.term.numeral")
            } else {
                owned_kinds.get(&index).copied()
            };
            let range = if kind.is_some() {
                syntax
                    .source_node_entries()
                    .unwrap()
                    .find(|row| row.site().index() == index)
                    .ok_or_else(|| "Task256 generic owned row disappeared".to_owned())?
                    .range()
            } else {
                node.range
            };
            nodes.push(
                TypedNode::new(
                    kind.unwrap_or("source.surface.unowned"),
                    SourceAnchor::Range(range),
                )
                .with_children(
                    node.children
                        .iter()
                        .map(|child| TypedNodeId::new(child.index()))
                        .collect(),
                )
                .with_typing(TypingState::Unknown)
                .with_recovery(if node.recovered {
                    NodeRecoveryState::Recovered
                } else {
                    NodeRecoveryState::Normal
                }),
            );
        }
        let arena =
            TypedArena::try_new(ast.root().map(|root| TypedNodeId::new(root.index())), nodes)
                .map_err(|error| error.to_string())?;
        let handoff = SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: ast.source_id,
                module_id: module.clone(),
                terms,
                references: Vec::new(),
                numeric_type_requests,
            },
            &binding_env,
            &arena,
        )
        .map_err(|error| error.to_string())?;
        SourceTermParts { arena, handoff }
    };
    let mut typed_ast = lower_family_typed_ast(
        ast,
        module.clone(),
        symbols,
        &binding_env,
        &route,
        source_term,
        Some(&syntax),
    )?;
    let mut input = route.handoff_input(ast, symbols, &typed_ast, &syntax)?;
    mutate(&mut input);
    let handoff = SourceAtomicFormulaProducer::build(
        input,
        &binding_env,
        symbols,
        typed_ast
            .source_term()
            .ok_or_else(|| "Task256 lost its Task252 dependency".to_owned())?,
        typed_ast.source_application(),
        typed_ast.source_structure(),
        typed_ast.source_set_term(),
        typed_ast.nodes(),
    )
    .map_err(|error| error.to_string())?;
    typed_ast = typed_ast
        .with_source_atomic_formula(handoff)
        .map_err(|error| error.to_string())?;

    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.formula.atomic"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_atomic_formula().is_none()
        || resolved.source_atomic_formula() != typed_ast.source_atomic_formula()
        || resolved.source_term() != typed_ast.source_term()
        || resolved.source_application() != typed_ast.source_application()
        || resolved.source_structure() != typed_ast.source_structure()
        || resolved.source_set_term() != typed_ast.source_set_term()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
    {
        return Err("source atomic-formula immutable final handoff mismatch".to_owned());
    }
    let formula_range = ast
        .nodes()
        .get(formula_node)
        .ok_or_else(|| "Task256 formula node disappeared".to_owned())?
        .range;
    if typed_ast
        .source_atomic_formula()
        .and_then(|handoff| handoff.formulas().get(SourceAtomicFormulaId::new(0)))
        .is_none_or(|formula| formula.source_range() != formula_range)
    {
        return Err("Task256 final formula ownership changed".to_owned());
    }
    Ok(SourceAtomicFormulaRouteOutput {
        typed_ast,
        resolved,
    })
}

fn lower_family_typed_ast(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    binding_env: &BindingEnv,
    route: &ExactAtomicRoute,
    source_term: SourceTermParts,
    syntax: Option<&TypedAst>,
) -> Result<TypedAst, String> {
    match route {
        ExactAtomicRoute::Predicate(_) => {
            let output = imported_source_application_output_with_source_term(
                ast,
                module,
                symbols,
                binding_env.clone(),
                source_term,
                syntax,
            )
            .ok_or_else(|| "Task256 imported application selector disappeared".to_owned())??;
            Ok(output.typed_ast)
        }
        ExactAtomicRoute::SetEnumeration(payload) => {
            let roots = [
                payload.left_site.node().index(),
                payload.right_site.node().index(),
            ];
            let output = source_set_term_output_with_source_term(
                ast,
                module,
                binding_env.clone(),
                &roots,
                source_term,
                syntax,
            )?;
            Ok(output.typed_ast)
        }
        ExactAtomicRoute::PredicateChain(_)
        | ExactAtomicRoute::Binary(_)
        | ExactAtomicRoute::TypeAssertion(_)
        | ExactAtomicRoute::Attribute { .. } => {
            empty_typed_ast_with_primary(ast, module, source_term)
        }
    }
}

fn empty_typed_ast_with_primary(
    ast: &SurfaceAst,
    module: ModuleId,
    source_term: SourceTermParts,
) -> Result<TypedAst, String> {
    TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: source_term.arena,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())?
    .with_source_term(source_term.handoff)
    .map_err(|error| error.to_string())
}

impl ExactAtomicRoute {
    fn formula_node(&self) -> usize {
        match self {
            Self::Binary(payload) => payload.formula_site.node().index(),
            Self::TypeAssertion(payload) => payload.formula_site.node().index(),
            Self::Predicate(payload) => payload.formula_site.node().index(),
            Self::PredicateChain(payload) => payload.formula_site.node().index(),
            Self::Attribute { payload, .. } => payload.formula_site.node().index(),
            Self::SetEnumeration(payload) => payload.formula_site.node().index(),
        }
    }

    fn formula_site(&self) -> TypedSiteRef {
        match self {
            Self::Binary(payload) => payload.formula_site.clone(),
            Self::TypeAssertion(payload) => payload.formula_site.clone(),
            Self::Predicate(payload) => payload.formula_site.clone(),
            Self::PredicateChain(payload) => payload.formula_site.clone(),
            Self::Attribute { payload, .. } => payload.formula_site.clone(),
            Self::SetEnumeration(payload) => payload.formula_site.clone(),
        }
    }

    fn primary_roots(&self) -> Vec<usize> {
        match self {
            Self::Binary(payload) => vec![
                payload.left_site.node().index(),
                payload.right_site.node().index(),
            ],
            Self::TypeAssertion(payload) => vec![payload.subject_site.node().index()],
            Self::Predicate(payload) => vec![
                payload.left_site.node().index(),
                payload.functor_left_site.node().index(),
                payload.functor_right_site.node().index(),
            ],
            Self::PredicateChain(payload) => payload
                .term_sites
                .iter()
                .map(|site| site.node().index())
                .collect(),
            Self::Attribute { payload, .. } => vec![payload.subject_site.node().index()],
            Self::SetEnumeration(payload) => payload
                .left_items
                .iter()
                .chain(&payload.right_items)
                .map(|site| site.node().index())
                .collect(),
        }
    }

    fn formula_kind(&self) -> SourceAtomicFormulaKind {
        match self {
            Self::Binary(payload) => match payload.formula_kind {
                mizar_checker::type_checker::FormulaKind::Equality => {
                    SourceAtomicFormulaKind::Equality
                }
                mizar_checker::type_checker::FormulaKind::Inequality => {
                    SourceAtomicFormulaKind::Inequality
                }
                mizar_checker::type_checker::FormulaKind::Membership => {
                    SourceAtomicFormulaKind::Membership
                }
                _ => unreachable!("exact Task256 binary selector admitted an unsupported kind"),
            },
            Self::TypeAssertion(_) => SourceAtomicFormulaKind::TypeAssertion,
            Self::Predicate(_) | Self::PredicateChain(_) => {
                SourceAtomicFormulaKind::PredicateApplication
            }
            Self::Attribute { .. } => SourceAtomicFormulaKind::AttributeAssertion,
            Self::SetEnumeration(_) => SourceAtomicFormulaKind::Equality,
        }
    }

    fn atomic_owned_node_kinds(
        &self,
        ast: &SurfaceAst,
    ) -> Result<BTreeMap<usize, &'static str>, String> {
        let mut kinds = BTreeMap::new();
        insert_kind(
            &mut kinds,
            self.formula_node(),
            formula_kind_key(self.formula_kind()),
        )?;
        match self {
            Self::TypeAssertion(payload) => {
                let type_node = payload.asserted_type_site.node().index();
                let head = unique_descendant(ast, type_node, |kind| {
                    matches!(kind, SurfaceNodeKind::TypeHead)
                })?;
                insert_kind(&mut kinds, type_node, "source.formula.atomic.asserted-type")?;
                insert_kind(
                    &mut kinds,
                    head.index(),
                    "source.formula.atomic.asserted-type-head",
                )?;
            }
            Self::Predicate(_) => {
                let head = unique_descendant(ast, self.formula_node(), |kind| {
                    matches!(kind, SurfaceNodeKind::PredicateHead)
                })?;
                insert_kind(
                    &mut kinds,
                    head.index(),
                    "source.formula.atomic.predicate-head",
                )?;
            }
            Self::PredicateChain(payload) => {
                for site in &payload.segment_sites {
                    insert_kind(
                        &mut kinds,
                        site.node().index(),
                        "source.formula.atomic.predicate-segment",
                    )?;
                }
                for site in &payload.head_sites {
                    insert_kind(
                        &mut kinds,
                        site.node().index(),
                        "source.formula.atomic.predicate-head",
                    )?;
                }
                insert_kind(
                    &mut kinds,
                    payload.verb_site.node().index(),
                    "source.formula.atomic.predicate-negation-verb",
                )?;
                insert_kind(
                    &mut kinds,
                    payload.not_site.node().index(),
                    "source.formula.atomic.predicate-negation-not",
                )?;
            }
            Self::Attribute { negative, .. } => {
                let attribute = unique_descendant(ast, self.formula_node(), |kind| {
                    matches!(kind, SurfaceNodeKind::AttributeRef)
                })?;
                let target = unique_descendant(ast, attribute.index(), |kind| {
                    matches!(kind, SurfaceNodeKind::QualifiedSymbol)
                })?;
                insert_kind(
                    &mut kinds,
                    attribute.index(),
                    "source.formula.atomic.attribute",
                )?;
                insert_kind(
                    &mut kinds,
                    target.index(),
                    "source.formula.atomic.attribute-target",
                )?;
                if *negative {
                    let non = unique_direct_token(ast, attribute.index(), "non")?;
                    insert_kind(
                        &mut kinds,
                        non.index(),
                        "source.formula.atomic.attribute-non",
                    )?;
                }
            }
            Self::Binary(_) | Self::SetEnumeration(_) => {}
        }
        Ok(kinds)
    }

    fn handoff_input(
        &self,
        ast: &SurfaceAst,
        symbols: &SymbolEnv,
        typed_ast: &TypedAst,
        syntax: &TypedAst,
    ) -> Result<SourceAtomicFormulaHandoffInput, String> {
        let formula = SourceAtomicFormulaId::new(0);
        let generic_row = |site: &TypedSiteRef| {
            syntax
                .source_node_entries()
                .and_then(|mut rows| rows.find(|row| row.site().index() == site.node().index()))
                .ok_or_else(|| "Task256 generic atomic row disappeared".to_owned())
        };
        let formula_row = generic_row(&self.formula_site())?;
        let formula_range = formula_row.range();
        let mut predicate_segments = Vec::new();
        let mut predicate_heads = Vec::new();
        let mut candidates = Vec::new();
        let mut type_sites = Vec::new();
        let mut attributes = Vec::new();
        let mut edges = Vec::new();
        let mut requests = Vec::new();

        match self {
            Self::Binary(payload) => {
                let left = primary_target(
                    typed_ast
                        .source_term()
                        .ok_or_else(|| "Task256 lost Task252".to_owned())?,
                    generic_row(&payload.left_site)?.range(),
                )?;
                let right = primary_target(
                    typed_ast
                        .source_term()
                        .ok_or_else(|| "Task256 lost Task252".to_owned())?,
                    generic_row(&payload.right_site)?.range(),
                )?;
                edges.extend([
                    edge(formula, 0, SourceAtomicEdgeRole::BuiltinLeftOperand, left),
                    edge(formula, 1, SourceAtomicEdgeRole::BuiltinRightOperand, right),
                ]);
                let request_edges: &[usize] =
                    if matches!(self.formula_kind(), SourceAtomicFormulaKind::Membership) {
                        &[1]
                    } else {
                        &[0, 1]
                    };
                for (ordinal, edge_index) in request_edges.iter().copied().enumerate() {
                    requests.push(SourceAtomicRequestInput {
                        formula,
                        ordinal,
                        kind: SourceAtomicRequestKind::OperandExpectedType,
                        edge: Some(SourceAtomicEdgeId::new(edge_index)),
                        candidate: None,
                        type_site: None,
                        attribute: None,
                    });
                }
            }
            Self::TypeAssertion(payload) => {
                let subject = primary_target(
                    typed_ast
                        .source_term()
                        .ok_or_else(|| "Task256 lost Task252".to_owned())?,
                    generic_row(&payload.subject_site)?.range(),
                )?;
                edges.push(edge(
                    formula,
                    0,
                    SourceAtomicEdgeRole::AssertionSubject,
                    subject,
                ));
                let type_node = payload.asserted_type_site.node().index();
                let head = unique_descendant(ast, type_node, |kind| {
                    matches!(kind, SurfaceNodeKind::TypeHead)
                })?;
                let type_row = generic_row(&payload.asserted_type_site)?;
                let head_row = generic_row(&surface_site(head))?;
                type_sites.push(SourceAssertionTypeSiteInput {
                    formula,
                    site: TypedSiteRef::Node(TypedNodeId::new(type_row.site().index())),
                    source_range: type_row.range(),
                    spelling: type_row.spelling().to_owned(),
                    head_site: TypedSiteRef::Node(TypedNodeId::new(head_row.site().index())),
                    head_range: head_row.range(),
                    head_spelling: head_row.spelling().to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    head: match payload.asserted_type.head {
                        mizar_checker::type_checker::TypeHeadInput::BuiltinSet => {
                            SourceAssertionTypeHead::BuiltinSet
                        }
                        mizar_checker::type_checker::TypeHeadInput::BuiltinObject => {
                            SourceAssertionTypeHead::BuiltinObject
                        }
                        _ => {
                            return Err(
                                "Task256 bare asserted type resolved to a non-builtin head"
                                    .to_owned(),
                            );
                        }
                    },
                });
                requests.push(SourceAtomicRequestInput {
                    formula,
                    ordinal: 0,
                    kind: SourceAtomicRequestKind::TypeAssertionReachability,
                    edge: None,
                    candidate: None,
                    type_site: Some(
                        mizar_checker::source_atomic_formula::SourceAssertionTypeSiteId::new(0),
                    ),
                    attribute: None,
                });
            }
            Self::Predicate(payload) => {
                let head_node = unique_descendant(ast, self.formula_node(), |kind| {
                    matches!(kind, SurfaceNodeKind::PredicateHead)
                })?;
                let head = generic_row(&surface_site(head_node))?;
                predicate_heads.push(SourcePredicateHeadInput {
                    formula,
                    site: TypedSiteRef::Node(TypedNodeId::new(head.site().index())),
                    source_range: head.range(),
                    context: BindingContextId::new(0),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: head.spelling().to_owned(),
                    left_arity: 1,
                    right_arity: 1,
                });
                let entry =
                    exact_symbol(symbols, &payload.predicate_symbol, SymbolKind::Predicate)?;
                candidates.push(SourcePredicateCandidateInput {
                    head: SourcePredicateHeadId::new(0),
                    ordinal: 0,
                    symbol: entry.symbol().clone(),
                    contribution: entry.contribution(),
                });
                let left = primary_target(
                    typed_ast
                        .source_term()
                        .ok_or_else(|| "Task256 lost Task252".to_owned())?,
                    generic_row(&payload.left_site)?.range(),
                )?;
                let application = typed_ast
                    .source_application()
                    .and_then(|handoff| handoff.applications().iter().next())
                    .map(|(id, _)| SourceAtomicTermTarget::Application(id))
                    .ok_or_else(|| "Task256 lost its Task253 root application".to_owned())?;
                edges.extend([
                    edge(
                        formula,
                        0,
                        SourceAtomicEdgeRole::PredicateLeftArgument,
                        left,
                    ),
                    edge(
                        formula,
                        1,
                        SourceAtomicEdgeRole::PredicateRightArgument,
                        application,
                    ),
                ]);
                requests.push(SourceAtomicRequestInput {
                    formula,
                    ordinal: 0,
                    kind: SourceAtomicRequestKind::PredicateCandidateSignature,
                    edge: None,
                    candidate: Some(SourcePredicateCandidateId::new(0)),
                    type_site: None,
                    attribute: None,
                });
            }
            Self::PredicateChain(payload) => {
                let primary = typed_ast
                    .source_term()
                    .ok_or_else(|| "Task257C1 lost Task252".to_owned())?;
                let targets = payload
                    .term_sites
                    .iter()
                    .map(|site| primary_target(primary, generic_row(site)?.range()))
                    .collect::<Result<Vec<_>, _>>()?;
                let entry =
                    exact_symbol(symbols, &payload.predicate_symbol, SymbolKind::Predicate)?;

                for ordinal in 0..2 {
                    let head_row = generic_row(&payload.head_sites[ordinal])?;
                    predicate_heads.push(SourcePredicateHeadInput {
                        formula,
                        site: TypedSiteRef::Node(TypedNodeId::new(head_row.site().index())),
                        source_range: head_row.range(),
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: head_row.spelling().to_owned(),
                        left_arity: 1,
                        right_arity: 1,
                    });
                    candidates.push(SourcePredicateCandidateInput {
                        head: SourcePredicateHeadId::new(ordinal),
                        ordinal: 0,
                        symbol: entry.symbol().clone(),
                        contribution: entry.contribution(),
                    });
                }

                edges.extend([
                    edge(
                        formula,
                        0,
                        SourceAtomicEdgeRole::PredicateLeftArgument,
                        targets[0],
                    ),
                    edge(
                        formula,
                        1,
                        SourceAtomicEdgeRole::PredicateChainBoundary,
                        targets[1],
                    ),
                    edge(
                        formula,
                        2,
                        SourceAtomicEdgeRole::PredicateRightArgument,
                        targets[2],
                    ),
                ]);
                let first_segment = generic_row(&payload.segment_sites[0])?;
                let second_segment = generic_row(&payload.segment_sites[1])?;
                let verb_row = generic_row(&payload.verb_site)?;
                let not_row = generic_row(&payload.not_site)?;
                predicate_segments.extend([
                    SourcePredicateSegmentInput {
                        formula,
                        ordinal: 0,
                        site: TypedSiteRef::Node(TypedNodeId::new(first_segment.site().index())),
                        source_range: first_segment.range(),
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: first_segment.spelling().to_owned(),
                        head: SourcePredicateHeadId::new(0),
                        polarity: SourcePredicateSegmentPolarityInput::Positive,
                        left_edge: SourceAtomicEdgeId::new(0),
                        right_edge: SourceAtomicEdgeId::new(1),
                    },
                    SourcePredicateSegmentInput {
                        formula,
                        ordinal: 1,
                        site: TypedSiteRef::Node(TypedNodeId::new(second_segment.site().index())),
                        source_range: second_segment.range(),
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: second_segment.spelling().to_owned(),
                        head: SourcePredicateHeadId::new(1),
                        polarity: SourcePredicateSegmentPolarityInput::Negative {
                            verb_site: TypedSiteRef::Node(TypedNodeId::new(
                                verb_row.site().index(),
                            )),
                            verb_range: verb_row.range(),
                            verb_spelling: verb_row.spelling().to_owned(),
                            verb_recovery: SourceAtomicFormulaRecovery::Normal,
                            not_site: TypedSiteRef::Node(TypedNodeId::new(not_row.site().index())),
                            not_range: not_row.range(),
                            not_spelling: not_row.spelling().to_owned(),
                            not_recovery: SourceAtomicFormulaRecovery::Normal,
                        },
                        left_edge: SourceAtomicEdgeId::new(1),
                        right_edge: SourceAtomicEdgeId::new(2),
                    },
                ]);
                for ordinal in 0..2 {
                    requests.push(SourceAtomicRequestInput {
                        formula,
                        ordinal,
                        kind: SourceAtomicRequestKind::PredicateCandidateSignature,
                        edge: None,
                        candidate: Some(SourcePredicateCandidateId::new(ordinal)),
                        type_site: None,
                        attribute: None,
                    });
                }
            }
            Self::Attribute { payload, negative } => {
                let subject = primary_target(
                    typed_ast
                        .source_term()
                        .ok_or_else(|| "Task256 lost Task252".to_owned())?,
                    generic_row(&payload.subject_site)?.range(),
                )?;
                edges.push(edge(
                    formula,
                    0,
                    SourceAtomicEdgeRole::AssertionSubject,
                    subject,
                ));
                let attribute = unique_descendant(ast, self.formula_node(), |kind| {
                    matches!(kind, SurfaceNodeKind::AttributeRef)
                })?;
                let target = unique_descendant(ast, attribute.index(), |kind| {
                    matches!(kind, SurfaceNodeKind::QualifiedSymbol)
                })?;
                let attribute_row = generic_row(&surface_site(attribute))?;
                let target_row = generic_row(&surface_site(target))?;
                let entry =
                    exact_symbol(symbols, &payload.attribute_symbol, SymbolKind::Attribute)?;
                let polarity = if *negative {
                    let non = unique_direct_token(ast, attribute.index(), "non")?;
                    let non_row = generic_row(&surface_site(non))?;
                    SourceAssertionAttributePolarityInput::Negative {
                        non_site: TypedSiteRef::Node(TypedNodeId::new(non_row.site().index())),
                        non_range: non_row.range(),
                        non_spelling: non_row.spelling().to_owned(),
                        non_recovery: SourceAtomicFormulaRecovery::Normal,
                    }
                } else {
                    SourceAssertionAttributePolarityInput::Positive
                };
                attributes.push(SourceAssertionAttributeInput {
                    formula,
                    ordinal: 0,
                    site: TypedSiteRef::Node(TypedNodeId::new(attribute_row.site().index())),
                    source_range: attribute_row.range(),
                    spelling: attribute_row.spelling().to_owned(),
                    target_site: TypedSiteRef::Node(TypedNodeId::new(target_row.site().index())),
                    target_range: target_row.range(),
                    target_spelling: target_row.spelling().to_owned(),
                    context: BindingContextId::new(0),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    symbol: entry.symbol().clone(),
                    contribution: entry.contribution(),
                    polarity,
                });
                requests.push(SourceAtomicRequestInput {
                    formula,
                    ordinal: 0,
                    kind: SourceAtomicRequestKind::AttributeAdmissibility,
                    edge: None,
                    candidate: None,
                    type_site: None,
                    attribute: Some(
                        mizar_checker::source_atomic_formula::SourceAssertionAttributeId::new(0),
                    ),
                });
            }
            Self::SetEnumeration(payload) => {
                let set_terms = typed_ast
                    .source_set_term()
                    .ok_or_else(|| "Task256 lost Task255 set roots".to_owned())?;
                let left = set_target(set_terms, generic_row(&payload.left_site)?.range())?;
                let right = set_target(set_terms, generic_row(&payload.right_site)?.range())?;
                edges.extend([
                    edge(formula, 0, SourceAtomicEdgeRole::BuiltinLeftOperand, left),
                    edge(formula, 1, SourceAtomicEdgeRole::BuiltinRightOperand, right),
                ]);
                for ordinal in 0..2 {
                    requests.push(SourceAtomicRequestInput {
                        formula,
                        ordinal,
                        kind: SourceAtomicRequestKind::OperandExpectedType,
                        edge: Some(SourceAtomicEdgeId::new(ordinal)),
                        candidate: None,
                        type_site: None,
                        attribute: None,
                    });
                }
            }
        }

        Ok(SourceAtomicFormulaHandoffInput {
            source_id: ast.source_id,
            module_id: typed_ast.module_id().clone(),
            formulas: vec![SourceAtomicFormulaInput {
                site: TypedSiteRef::Node(TypedNodeId::new(formula_row.site().index())),
                source_range: formula_range,
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: formula_row.spelling().to_owned(),
                kind: self.formula_kind(),
            }],
            wrappers: Vec::new(),
            predicate_segments,
            predicate_heads,
            candidates,
            type_sites,
            attributes,
            edges,
            requests,
        })
    }
}

pub(in crate::runner) fn step5c7_membership_typed_ast(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    scope: &mizar_resolve::names::ResolvedVariableScope,
    binding_env: &BindingEnv,
) -> Result<TypedAst, String> {
    let scoped = ast.node_views().any(|view| {
        view.kind() == &SurfaceNodeKind::SetComprehension
            && ast.node_views().any(|inner| {
                inner.kind() == &SurfaceNodeKind::PredicateApplication
                    && view.range().start <= inner.range().start
                    && inner.range().end <= view.range().end
            })
    });
    let mut arena = membership_typed_arena(ast)?;
    if scoped {
        let source = SurfaceResolvedArena::lower(ast, module).map_err(|error| error.to_string())?;
        let authentic = mizar_checker::type_checker::SourceVariableSemanticsChecker::source_theorem_binding_env(
            &source, symbols, scope,
        )?;
        if binding_env != &authentic {
            return Err("membership scoped binding environment mismatch".into());
        }
        let nodes = arena
            .iter()
            .zip(ast.node_views())
            .map(|((_, node), surface)| {
                source
                    .resolved_node_for(surface.id())
                    .map(|resolved| node.clone().with_resolved_node(resolved))
                    .ok_or_else(|| "membership scoped resolved link missing".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        arena = TypedArena::try_new(arena.root(), nodes).map_err(|error| error.to_string())?;
    }
    let primary = membership_primary_terms(ast, module, symbols, scope, binding_env, &arena)?;
    let roots = ast
        .nodes()
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            matches!(
                node.kind,
                SurfaceNodeKind::SetEnumeration | SurfaceNodeKind::SetComprehension
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    let set_output = step5c7_set_output(
        ast,
        module.clone(),
        binding_env.clone(),
        &roots,
        SourceTermParts {
            arena: arena.clone(),
            handoff: primary.clone(),
        },
    )?;
    let set_handoff = set_output
        .typed_ast
        .source_set_term()
        .ok_or_else(|| "membership set transaction produced no set handoff".to_owned())?
        .clone();
    let mut arena = membership_restore_arena(ast, set_output.typed_ast.nodes())?;
    let condition_atoms = set_handoff
        .conditions()
        .iter()
        .filter_map(|(_, condition)| {
            let condition_node = ast.nodes().get(condition.condition_site().node().index())?;
            let children = structural_child_ids(ast, condition_node);
            let [child] = children.as_slice() else { return None; };
            let outer = ast.node(*child)?;
            if outer.range != condition.source_range()
                || surface_text(ast, outer) != condition.spelling()
            {
                return None;
            }
            let mut cursor = *child;
            let mut wrappers = Vec::new();
            while matches!(ast.node(cursor)?.kind, SurfaceNodeKind::ParenthesizedFormula) {
                wrappers.push(cursor);
                let children = structural_child_ids(ast, ast.node(cursor)?);
                let [expression] = children.as_slice() else { return None; };
                let expression = ast.node(*expression)?;
                if !matches!(expression.kind, SurfaceNodeKind::FormulaExpression) {
                    return None;
                }
                let children = structural_child_ids(ast, expression);
                let [inner] = children.as_slice() else { return None; };
                cursor = *inner;
            }
            let formula = ast.node(cursor)?;
            ((matches!(formula.kind, SurfaceNodeKind::BuiltinPredicateApplication)
                && matches!(direct_token_texts(ast, formula).as_slice(), [operator] if matches!(operator.as_str(), "=" | "<>" | "in")))
                || scoped && wrappers.is_empty() && matches!(formula.kind, SurfaceNodeKind::PredicateApplication))
            .then_some((cursor, wrappers))
        })
        .collect::<Vec<_>>();
    if scoped
        && (condition_atoms.len() != set_handoff.conditions().len()
            || condition_atoms.iter().any(|(site, _)| {
                !matches!(
                    ast.nodes()[site.index()].kind,
                    SurfaceNodeKind::PredicateApplication
                )
            }))
    {
        return Err("unsupported scoped predicate condition".into());
    }
    if !condition_atoms.is_empty() {
        let mut nodes = arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        for (site, wrappers) in &condition_atoms {
            nodes[site.index()].kind = if matches!(
                ast.nodes()[site.index()].kind,
                SurfaceNodeKind::PredicateApplication
            ) {
                "source.formula.atomic.predicate"
            } else {
                match direct_token_texts(ast, &ast.nodes()[site.index()]).as_slice() {
                    [operator] if operator == "=" => "source.formula.atomic.equality",
                    [operator] if operator == "<>" => "source.formula.atomic.inequality",
                    _ => "source.formula.atomic.membership",
                }
            }
            .into();
            for wrapper in wrappers {
                nodes[wrapper.index()].kind = "source.formula.atomic.parenthesized".into();
            }
        }
        arena = TypedArena::try_new(arena.root(), nodes).map_err(|error| error.to_string())?;
    }
    let mut formulas = ast
        .node_views()
        .filter_map(|view| {
            let node = ast.node(view.id())?;
            ((matches!(node.kind, SurfaceNodeKind::BuiltinPredicateApplication)
                && direct_token_texts(ast, node) == ["in"])
                || condition_atoms.iter().any(|(site, _)| *site == view.id()))
            .then_some(view.id())
        })
        .collect::<Vec<_>>();
    formulas.sort_by_key(|id| {
        let node = &ast.nodes()[id.index()];
        (node.range.start, node.range.end, id.index())
    });
    if formulas.is_empty() {
        return Err("membership source AST has no membership formula".to_owned());
    }
    let mut predicate_segments = Vec::new();
    let mut predicate_heads = Vec::new();
    let mut candidates = Vec::new();
    let mut predicate_kinds = BTreeMap::new();
    let mut atomic_formulas = Vec::with_capacity(formulas.len());
    let mut wrappers = Vec::new();
    let mut edges = Vec::with_capacity(formulas.len() * 2);
    let mut requests = Vec::with_capacity(formulas.len() * 2);
    for (formula_ordinal, formula_id) in formulas.iter().copied().enumerate() {
        let formula = ast
            .node(formula_id)
            .ok_or_else(|| "membership formula disappeared".to_owned())?;
        let formula_key = SourceAtomicFormulaId::new(formula_ordinal);
        let predicate = matches!(formula.kind, SurfaceNodeKind::PredicateApplication);
        let children = structural_child_ids(ast, formula);
        let mut predicate_heads_written = Vec::new();
        let operands = if predicate {
            if children.is_empty() {
                return Err("condition predicate has no segments".into());
            }
            let mut operands = Vec::new();
            for (ordinal, segment_id) in children.iter().enumerate() {
                let segment = ast.node(*segment_id).ok_or("missing condition segment")?;
                if !matches!(segment.kind, SurfaceNodeKind::PredicateSegment) {
                    return Err("invalid condition segment".into());
                }
                let (left, written) = if ordinal == 0 {
                    let (left, written) = segment
                        .children
                        .split_first()
                        .ok_or("missing first predicate operand")?;
                    (Some(*left), written)
                } else {
                    (None, segment.children.as_slice())
                };
                let (head_id, right, polarity) = match written {
                    [head, right] => (*head, *right, SourcePredicateSegmentPolarityInput::Positive),
                    [verb, not, head, right]
                        if ast
                            .node(*verb)
                            .and_then(|node| node.token_text())
                            .is_some_and(|text| matches!(text, "does" | "do"))
                            && ast.node(*not).and_then(|node| node.token_text()) == Some("not") =>
                    {
                        let verb_node = ast.node(*verb).ok_or("missing negation verb")?;
                        let not_node = ast.node(*not).ok_or("missing negation not")?;
                        insert_kind(
                            &mut predicate_kinds,
                            verb.index(),
                            "source.formula.atomic.predicate-negation-verb",
                        )?;
                        insert_kind(
                            &mut predicate_kinds,
                            not.index(),
                            "source.formula.atomic.predicate-negation-not",
                        )?;
                        (
                            *head,
                            *right,
                            SourcePredicateSegmentPolarityInput::Negative {
                                verb_site: surface_site(*verb),
                                verb_range: verb_node.range,
                                verb_spelling: surface_text(ast, verb_node),
                                verb_recovery: SourceAtomicFormulaRecovery::Normal,
                                not_site: surface_site(*not),
                                not_range: not_node.range,
                                not_spelling: surface_text(ast, not_node),
                                not_recovery: SourceAtomicFormulaRecovery::Normal,
                            },
                        )
                    }
                    _ => return Err("unsupported condition predicate children".into()),
                };
                let head = ast.node(head_id).ok_or("missing condition head")?;
                if !matches!(head.kind, SurfaceNodeKind::PredicateHead) {
                    return Err("invalid condition head".into());
                }
                insert_kind(
                    &mut predicate_kinds,
                    segment_id.index(),
                    "source.formula.atomic.predicate-segment",
                )?;
                insert_kind(
                    &mut predicate_kinds,
                    head_id.index(),
                    "source.formula.atomic.predicate-head",
                )?;
                predicate_heads_written.push((*segment_id, head_id, polarity));
                operands.extend(left);
                operands.push(right);
            }
            operands
        } else {
            children
        };
        if operands.len() < 2 || !predicate && operands.len() != 2 {
            return Err("membership formula does not have two operands".to_owned());
        }
        let target = |operand| -> Result<SourceAtomicTermTarget, String> {
            let wrapper = ast
                .node(operand)
                .ok_or_else(|| "membership operand disappeared".to_owned())?;
            let children = structural_child_ids(ast, wrapper);
            let [child] = children.as_slice() else {
                return Err("membership operand is not a single wrapper".to_owned());
            };
            let node = ast
                .node(*child)
                .ok_or_else(|| "membership operand disappeared".to_owned())?;
            match node.kind {
                SurfaceNodeKind::TermReference => primary_target(&primary, node.range),
                SurfaceNodeKind::ParenthesizedTerm
                    if primary.terms().iter().any(|(_, row)| {
                        row.parent().is_none() && row.source_range() == node.range
                    }) =>
                {
                    primary_target(&primary, node.range)
                }
                SurfaceNodeKind::SetEnumeration
                | SurfaceNodeKind::SetComprehension
                | SurfaceNodeKind::ParenthesizedTerm => set_target(&set_handoff, node.range),
                _ => Err("membership operand has an unsupported term shape".to_owned()),
            }
        };
        let targets = operands
            .iter()
            .map(|operand| target(*operand))
            .collect::<Result<Vec<_>, _>>()?;
        let left = targets[0];
        let context = if scoped {
            let SourceAtomicTermTarget::Primary(id) = left else {
                return Err("scoped atom needs a Primary left operand".into());
            };
            primary
                .terms()
                .get(id)
                .ok_or("missing scoped operand")?
                .context()
        } else {
            BindingContextId::new(0)
        };
        let comparison = !predicate && direct_token_texts(ast, formula) != ["in"];
        let mut request_ordinal = 0;
        for (ordinal, (segment_id, head_id, polarity)) in
            predicate_heads_written.into_iter().enumerate()
        {
            let head = SourcePredicateHeadId::new(predicate_heads.len());
            let segment = ast.node(segment_id).ok_or("missing predicate segment")?;
            let head_node = ast.node(head_id).ok_or("missing predicate head")?;
            predicate_heads.push(SourcePredicateHeadInput {
                formula: formula_key,
                site: surface_site(head_id),
                source_range: head_node.range,
                context,
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: surface_text(ast, head_node),
                left_arity: 1,
                right_arity: 1,
            });
            predicate_segments.push(SourcePredicateSegmentInput {
                formula: formula_key,
                ordinal,
                site: surface_site(segment_id),
                source_range: segment.range,
                context,
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: surface_text(ast, segment),
                head,
                polarity,
                left_edge: SourceAtomicEdgeId::new(edges.len() + ordinal),
                right_edge: SourceAtomicEdgeId::new(edges.len() + ordinal + 1),
            });
            for (ordinal, entry) in symbols
                .symbols()
                .iter()
                .filter(|entry| entry.kind() == SymbolKind::Predicate)
                .enumerate()
            {
                let candidate = SourcePredicateCandidateId::new(candidates.len());
                candidates.push(SourcePredicateCandidateInput {
                    head,
                    ordinal,
                    symbol: entry.symbol().clone(),
                    contribution: entry.contribution(),
                });
                requests.push(SourceAtomicRequestInput {
                    formula: formula_key,
                    ordinal: request_ordinal,
                    kind: SourceAtomicRequestKind::PredicateCandidateSignature,
                    edge: None,
                    candidate: Some(candidate),
                    type_site: None,
                    attribute: None,
                });
                request_ordinal += 1;
            }
        }
        if let Some((_, groups)) = condition_atoms.iter().find(|(site, _)| *site == formula_id) {
            for (ordinal, site) in groups.iter().copied().enumerate() {
                let group = ast
                    .node(site)
                    .ok_or_else(|| "condition group disappeared".to_owned())?;
                wrappers.push(SourceAtomicWrapperInput {
                    formula: formula_key,
                    ordinal,
                    site: surface_site(site),
                    source_range: group.range,
                    context,
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: surface_text(ast, group),
                });
            }
        }
        atomic_formulas.push(SourceAtomicFormulaInput {
            site: surface_site(formula_id),
            source_range: formula.range,
            source_ordinal: formula_ordinal,
            context,
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: surface_text(ast, formula),
            kind: if predicate {
                SourceAtomicFormulaKind::PredicateApplication
            } else if comparison && direct_token_texts(ast, formula) == ["<>"] {
                SourceAtomicFormulaKind::Inequality
            } else if comparison {
                SourceAtomicFormulaKind::Equality
            } else {
                SourceAtomicFormulaKind::Membership
            },
        });
        let operand_count = targets.len();
        edges.extend(targets.into_iter().enumerate().map(|(ordinal, target)| {
            edge(
                formula_key,
                ordinal,
                if predicate {
                    if ordinal == 0 {
                        SourceAtomicEdgeRole::PredicateLeftArgument
                    } else if ordinal + 1 == operand_count {
                        SourceAtomicEdgeRole::PredicateRightArgument
                    } else {
                        SourceAtomicEdgeRole::PredicateChainBoundary
                    }
                } else if ordinal == 0 {
                    SourceAtomicEdgeRole::BuiltinLeftOperand
                } else {
                    SourceAtomicEdgeRole::BuiltinRightOperand
                },
                target,
            )
        }));
        for (request_ordinal, edge_ordinal) in (if predicate {
            0..0
        } else if comparison {
            0..2
        } else {
            1..2
        })
        .enumerate()
        {
            requests.push(SourceAtomicRequestInput {
                formula: formula_key,
                ordinal: request_ordinal,
                kind: SourceAtomicRequestKind::OperandExpectedType,
                edge: Some(SourceAtomicEdgeId::new(edges.len() - 2 + edge_ordinal)),
                candidate: None,
                type_site: None,
                attribute: None,
            });
        }
    }
    if scoped {
        let mut nodes = arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        for (index, kind) in predicate_kinds {
            nodes[index].kind = kind.into();
        }
        arena = TypedArena::try_new(arena.root(), nodes).map_err(|error| error.to_string())?;
    }
    let handoff = SourceAtomicFormulaProducer::build(
        SourceAtomicFormulaHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            formulas: atomic_formulas,
            wrappers,
            predicate_segments,
            predicate_heads,
            candidates,
            type_sites: Vec::new(),
            attributes: Vec::new(),
            edges,
            requests,
        },
        binding_env,
        symbols,
        &primary,
        None,
        None,
        Some(&set_handoff),
        &arena,
    )
    .map_err(|error| error.to_string())?;
    empty_typed_ast_with_primary(
        ast,
        module.clone(),
        SourceTermParts {
            arena,
            handoff: primary,
        },
    )?
    .with_source_set_term(set_handoff)
    .map_err(|error| error.to_string())?
    .with_source_atomic_formula(handoff)
    .map_err(|error| error.to_string())
}

pub(in crate::runner) fn step5c8_formula_typed_ast(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    scope: &mizar_resolve::names::ResolvedVariableScope,
    binding_env: &BindingEnv,
    proof_organization: bool,
) -> Result<TypedAst, String> {
    let mut source_arena = membership_typed_arena(ast)?;
    let primary = if proof_organization
        && ast
            .node_views()
            .any(|view| matches!(view.kind(), SurfaceNodeKind::ComputationJustification))
    {
        let mut terms = Vec::new();
        let mut nodes = source_arena
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        for atom in ast
            .node_views()
            .filter(|view| matches!(view.kind(), SurfaceNodeKind::BuiltinPredicateApplication))
        {
            for operand in structural_child_ids(ast, ast.node(atom.id()).ok_or("missing equality")?)
            {
                let wrapper = ast.node(operand).ok_or("missing operand")?;
                let children = structural_child_ids(ast, wrapper);
                let [id] = children.as_slice() else {
                    return Err("unsupported computation operand".into());
                };
                let node = ast.node(*id).ok_or("missing numeral")?;
                if !matches!(node.kind, SurfaceNodeKind::NumeralTerm) {
                    return Err("unsupported computation term".into());
                }
                nodes[id.index()].kind = "source.term.numeral".into();
                terms.push(SourcePrimaryTermInput {
                    site: surface_site(*id),
                    source_range: node.range,
                    source_ordinal: terms.len(),
                    context: BindingContextId::new(0),
                    recovery: SourcePrimaryTermRecovery::Normal,
                    spelling: surface_text(ast, node),
                    kind: SourcePrimaryTermKind::Numeral,
                    role: SourcePrimaryTermRole::Value,
                    parent: None,
                });
            }
        }
        source_arena =
            TypedArena::try_new(source_arena.root(), nodes).map_err(|error| error.to_string())?;
        let numeric_type_requests = terms
            .iter()
            .enumerate()
            .map(
                |(index, term)| mizar_checker::source_term::SourceNumericTypeRequestInput {
                    term: SourcePrimaryTermId::new(index),
                    owner: term.site.clone(),
                    source_range: term.source_range,
                    spelling: term.spelling.clone(),
                    request_ordinal: index,
                },
            )
            .collect();
        SourcePrimaryTermProducer::build(
            SourcePrimaryTermHandoffInput {
                source_id: ast.source_id,
                module_id: module.clone(),
                terms,
                references: Vec::new(),
                numeric_type_requests,
            },
            binding_env,
            &source_arena,
        )
        .map_err(|error| error.to_string())?
    } else {
        membership_primary_terms(ast, module, symbols, scope, binding_env, &source_arena)?
    };
    let mut sets = None;
    if proof_organization {
        let roots = ast
            .node_views()
            .filter(|view| matches!(view.kind(), SurfaceNodeKind::SetEnumeration))
            .map(|view| view.id().index())
            .collect::<Vec<_>>();
        if !roots.is_empty() {
            let output = step5c7_set_output(
                ast,
                module.clone(),
                binding_env.clone(),
                &roots,
                SourceTermParts {
                    arena: source_arena,
                    handoff: primary.clone(),
                },
            )?;
            source_arena = output.typed_ast.nodes().clone();
            sets = Some(
                output
                    .typed_ast
                    .source_set_term()
                    .ok_or("missing enumeration handoff")?
                    .clone(),
            );
        }
    }
    let operand = |id| {
        let node = ast.node(id).ok_or("missing formula operand")?;
        let children = structural_child_ids(ast, node);
        if let ([child], Some(sets)) = (children.as_slice(), sets.as_ref())
            && matches!(
                ast.nodes()[child.index()].kind,
                SurfaceNodeKind::SetEnumeration
            )
        {
            return set_target(sets, ast.nodes()[child.index()].range);
        }
        step5c8_primary_operand(ast, id, &primary)
    };
    let mut formulas = ast
        .node_views()
        .filter_map(|view| {
            let node = ast.node(view.id())?;
            let kind = match node.kind {
                SurfaceNodeKind::BuiltinPredicateApplication => {
                    match direct_token_texts(ast, node).as_slice() {
                        [operator] if operator == "=" => Some(SourceAtomicFormulaKind::Equality),
                        [operator] if operator == "in" => Some(SourceAtomicFormulaKind::Membership),
                        _ => None,
                    }
                }
                SurfaceNodeKind::IsAssertion
                    if direct_token_texts(ast, node).as_slice() == ["is"] =>
                {
                    Some(SourceAtomicFormulaKind::TypeAssertion)
                }
                _ => None,
            }?;
            Some((view.id(), kind))
        })
        .collect::<Vec<_>>();
    formulas.sort_by_key(|(id, _)| {
        let range = ast.nodes()[id.index()].range;
        (range.start, range.end, id.index())
    });
    if formulas.is_empty() {
        return Err("formula source AST has no supported atomic formula".into());
    }
    let mut atomic = Vec::with_capacity(formulas.len());
    let mut type_sites = Vec::new();
    let mut edges = Vec::with_capacity(formulas.len() * 2);
    let mut requests = Vec::new();
    let mut owned_kinds = BTreeMap::new();
    for (ordinal, (formula_id, kind)) in formulas.iter().copied().enumerate() {
        let formula = ast
            .node(formula_id)
            .ok_or_else(|| "formula atomic node disappeared".to_owned())?;
        let formula_key = SourceAtomicFormulaId::new(ordinal);
        insert_kind(&mut owned_kinds, formula_id.index(), formula_kind_key(kind))?;
        atomic.push(SourceAtomicFormulaInput {
            site: surface_site(formula_id),
            source_range: formula.range,
            source_ordinal: ordinal,
            context: BindingContextId::new(0),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: surface_text(ast, formula),
            kind,
        });
        match kind {
            SourceAtomicFormulaKind::Equality | SourceAtomicFormulaKind::Membership => {
                let children = structural_child_ids(ast, formula);
                let [left, right] = children.as_slice() else {
                    return Err("formula builtin atom requires two operands".to_owned());
                };
                let left = operand(*left)?;
                let right = operand(*right)?;
                let edge_base = edges.len();
                edges.extend([
                    edge(
                        formula_key,
                        0,
                        SourceAtomicEdgeRole::BuiltinLeftOperand,
                        left,
                    ),
                    edge(
                        formula_key,
                        1,
                        SourceAtomicEdgeRole::BuiltinRightOperand,
                        right,
                    ),
                ]);
                let request_edge = if kind == SourceAtomicFormulaKind::Membership {
                    1
                } else {
                    0
                };
                requests.push(SourceAtomicRequestInput {
                    formula: formula_key,
                    ordinal: 0,
                    kind: SourceAtomicRequestKind::OperandExpectedType,
                    edge: Some(SourceAtomicEdgeId::new(edge_base + request_edge)),
                    candidate: None,
                    type_site: None,
                    attribute: None,
                });
                if kind == SourceAtomicFormulaKind::Equality {
                    requests.push(SourceAtomicRequestInput {
                        formula: formula_key,
                        ordinal: 1,
                        kind: SourceAtomicRequestKind::OperandExpectedType,
                        edge: Some(SourceAtomicEdgeId::new(edge_base + 1)),
                        candidate: None,
                        type_site: None,
                        attribute: None,
                    });
                }
            }
            SourceAtomicFormulaKind::TypeAssertion => {
                let children = structural_child_ids(ast, formula);
                let [subject, asserted_type] = children.as_slice() else {
                    return Err("formula type assertion shape is not binary".to_owned());
                };
                let subject = operand(*subject)?;
                let type_node = ast
                    .node(*asserted_type)
                    .ok_or_else(|| "formula asserted type disappeared".to_owned())?;
                if !matches!(type_node.kind, SurfaceNodeKind::TypeExpression) {
                    return Err("formula assertion target is not a type expression".to_owned());
                }
                let head = unique_descendant(ast, asserted_type.index(), |kind| {
                    matches!(kind, SurfaceNodeKind::TypeHead)
                })?;
                let head_node = ast
                    .node(head)
                    .ok_or_else(|| "formula asserted type head disappeared".to_owned())?;
                let spelling = surface_text(ast, type_node);
                let head_spelling = surface_text(ast, head_node);
                let head_kind = match spelling.as_str() {
                    "set" if head_spelling == "set" => SourceAssertionTypeHead::BuiltinSet,
                    "object" if head_spelling == "object" => SourceAssertionTypeHead::BuiltinObject,
                    _ => return Err("formula assertion target is not a builtin type".to_owned()),
                };
                let type_site_id =
                    mizar_checker::source_atomic_formula::SourceAssertionTypeSiteId::new(
                        type_sites.len(),
                    );
                insert_kind(
                    &mut owned_kinds,
                    asserted_type.index(),
                    "source.formula.atomic.asserted-type",
                )?;
                insert_kind(
                    &mut owned_kinds,
                    head.index(),
                    "source.formula.atomic.asserted-type-head",
                )?;
                type_sites.push(SourceAssertionTypeSiteInput {
                    formula: formula_key,
                    site: surface_site(*asserted_type),
                    source_range: type_node.range,
                    spelling: spelling.clone(),
                    head_site: surface_site(head),
                    head_range: head_node.range,
                    head_spelling,
                    context: BindingContextId::new(0),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    head: head_kind,
                });
                edges.push(edge(
                    formula_key,
                    0,
                    SourceAtomicEdgeRole::AssertionSubject,
                    subject,
                ));
                requests.push(SourceAtomicRequestInput {
                    formula: formula_key,
                    ordinal: 0,
                    kind: SourceAtomicRequestKind::TypeAssertionReachability,
                    edge: None,
                    candidate: None,
                    type_site: Some(type_site_id),
                    attribute: None,
                });
            }
            _ => unreachable!(),
        }
    }
    let mut arena = membership_restore_arena(ast, &source_arena)?;
    let mut nodes = arena
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for (index, kind) in owned_kinds {
        nodes[index].kind = kind.into();
    }
    arena = TypedArena::try_new(arena.root(), nodes).map_err(|error| error.to_string())?;
    let handoff = SourceAtomicFormulaProducer::build(
        SourceAtomicFormulaHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            formulas: atomic,
            wrappers: Vec::new(),
            predicate_segments: Vec::new(),
            predicate_heads: Vec::new(),
            candidates: Vec::new(),
            type_sites,
            attributes: Vec::new(),
            edges,
            requests,
        },
        binding_env,
        symbols,
        &primary,
        None,
        None,
        sets.as_ref(),
        &arena,
    )
    .map_err(|error| error.to_string())?;
    let mut typed = empty_typed_ast_with_primary(
        ast,
        module.clone(),
        SourceTermParts {
            arena,
            handoff: primary,
        },
    )?;
    if let Some(sets) = sets {
        typed = typed
            .with_source_set_term(sets)
            .map_err(|error| error.to_string())?;
    }
    typed
        .with_source_atomic_formula(handoff)
        .map_err(|error| error.to_string())
}
fn step5c8_primary_operand(
    ast: &SurfaceAst,
    operand: SurfaceNodeId,
    primary: &SourcePrimaryTermHandoff,
) -> Result<SourceAtomicTermTarget, String> {
    let wrapper = ast
        .node(operand)
        .ok_or_else(|| "formula operand disappeared".to_owned())?;
    let children = structural_child_ids(ast, wrapper);
    let [child] = children.as_slice() else {
        return Err("formula operand is not a single wrapper".to_owned());
    };
    let node = ast
        .node(*child)
        .ok_or_else(|| "formula operand disappeared".to_owned())?;
    match node.kind {
        SurfaceNodeKind::TermReference | SurfaceNodeKind::NumeralTerm => {
            primary_target(primary, node.range)
        }
        _ => Err("formula atom requires an authenticated variable operand".to_owned()),
    }
}

fn membership_typed_arena(ast: &SurfaceAst) -> Result<TypedArena, String> {
    let root = ast
        .root()
        .ok_or_else(|| "membership surface AST has no root".to_owned())?;
    if ast.nodes().iter().any(|node| node.recovered) {
        return Err("membership surface AST contains recovery".to_owned());
    }
    let containers = ast
        .nodes()
        .iter()
        .filter(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::SetComprehension | SurfaceNodeKind::SetEnumeration
            )
        })
        .flat_map(|node| node.children.iter().filter_map(|child| ast.node(*child)))
        .filter(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::FormulaExpression | SurfaceNodeKind::TermExpression
            )
        })
        .map(|node| node.range)
        .collect::<Vec<_>>();
    let mut nodes: Vec<_> = ast
        .nodes()
        .iter()
        .map(|node| {
            let kind = if matches!(node.kind, SurfaceNodeKind::TermReference) {
                "source.term.variable-reference".to_owned()
            } else {
                "source.surface.unowned".to_owned()
            };
            TypedNode::new(kind, SourceAnchor::Range(node.range))
                .with_children(
                    node.children
                        .iter()
                        .map(|child| TypedNodeId::new(child.index()))
                        .collect(),
                )
                .with_recovery(NodeRecoveryState::Normal)
        })
        .collect();
    for (index, node) in ast.nodes().iter().enumerate() {
        if !matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm)
            || !containers
                .iter()
                .any(|range| range.start <= node.range.start && node.range.end <= range.end)
        {
            continue;
        }
        let mut core = node;
        while matches!(
            core.kind,
            SurfaceNodeKind::ParenthesizedTerm | SurfaceNodeKind::TermExpression
        ) {
            let children = structural_child_ids(ast, core);
            let [child] = children.as_slice() else {
                return Err("unsupported Primary grouping".into());
            };
            core = ast.node(*child).ok_or("missing grouped Primary child")?;
        }
        if matches!(core.kind, SurfaceNodeKind::TermReference) {
            nodes[index].kind = "source.term.parenthesized".into();
            for child in &node.children {
                let row = ast.node(*child).ok_or("missing Primary group part")?;
                nodes[child.index()].kind = format!("{:?}", row.kind).into();
            }
        }
    }
    TypedArena::try_new(Some(TypedNodeId::new(root.index())), nodes)
        .map_err(|error| error.to_string())
}

fn membership_restore_arena(
    ast: &SurfaceAst,
    set_arena: &TypedArena,
) -> Result<TypedArena, String> {
    if ast.nodes().len() != set_arena.len() {
        return Err("membership set arena identity changed".to_owned());
    }
    let mut nodes = set_arena
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for (index, node) in nodes.iter_mut().enumerate() {
        if node.kind.as_str() == "source.surface.unowned" {
            let surface = &ast.nodes()[index];
            node.kind = if matches!(surface.kind, SurfaceNodeKind::BuiltinPredicateApplication)
                && direct_token_texts(ast, surface) == ["in"]
            {
                "source.formula.atomic.membership".into()
            } else {
                format!("{:?}", surface.kind).into()
            };
        }
    }
    TypedArena::try_new(set_arena.root(), nodes).map_err(|error| error.to_string())
}

fn membership_primary_terms(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    scope: &mizar_resolve::names::ResolvedVariableScope,
    binding_env: &BindingEnv,
    arena: &TypedArena,
) -> Result<SourcePrimaryTermHandoff, String> {
    let mut terms = Vec::new();
    let mut references = Vec::new();
    let parents = ast
        .node_views()
        .flat_map(|view| view.children().iter().map(move |child| (*child, view.id())))
        .collect::<BTreeMap<_, _>>();
    let scoped = binding_env.contexts().iter().any(|(_, context)| {
        matches!(
            context.owner,
            BindingContextOwner::SourceComprehension { .. }
        )
    });
    for reference in scope.references() {
        let context = if scoped {
            binding_env
                .contexts()
                .iter()
                .find(|(_, context)| context.lexical_scope.as_ref() == Some(reference.scope()))
                .map(|(id, _)| id)
                .ok_or("membership Primary lexical context missing")?
        } else {
            BindingContextId::new(0)
        };
        let mut cursor = reference.node();
        let mut groups = Vec::new();
        while let Some(parent) = parents.get(&cursor) {
            let node = ast.node(*parent).ok_or("missing Primary ancestor")?;
            match node.kind {
                SurfaceNodeKind::TermExpression => {}
                SurfaceNodeKind::ParenthesizedTerm
                    if arena
                        .node(TypedNodeId::new(parent.index()))
                        .is_some_and(|row| row.kind.as_str() == "source.term.parenthesized") =>
                {
                    groups.push(*parent)
                }
                _ => break,
            }
            cursor = *parent;
        }
        let mut parent = None;
        for site in groups.into_iter().rev() {
            let node = ast.node(site).ok_or("missing Primary shell")?;
            let id = SourcePrimaryTermId::new(terms.len());
            terms.push(SourcePrimaryTermInput {
                site: surface_site(site),
                source_range: node.range,
                source_ordinal: id.index(),
                context,
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: surface_text(ast, node),
                kind: SourcePrimaryTermKind::Parenthesized,
                role: SourcePrimaryTermRole::Value,
                parent,
            });
            parent = Some(id);
        }
        let leaf = SourcePrimaryTermId::new(terms.len());
        terms.push(SourcePrimaryTermInput {
            site: surface_site(reference.node()),
            source_range: reference.range(),
            source_ordinal: leaf.index(),
            context,
            recovery: SourcePrimaryTermRecovery::Normal,
            spelling: reference.spelling().to_owned(),
            kind: SourcePrimaryTermKind::VariableReference,
            role: SourcePrimaryTermRole::Value,
            parent,
        });
        references.push(SourcePrimaryTermReferenceInput {
            term: leaf,
            binding: BindingId::new(reference.binding().index()),
            role: SourcePrimaryTermReferenceRole::Variable,
        });
    }
    let input = SourcePrimaryTermHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms,
        references,
        numeric_type_requests: Vec::new(),
    };
    if scoped {
        let source = SurfaceResolvedArena::lower(ast, module).map_err(|error| error.to_string())?;
        SourcePrimaryTermProducer::build_from_scoped_theorem_occurrences(
            input,
            binding_env,
            arena,
            &source,
            symbols,
            scope,
        )
    } else {
        SourcePrimaryTermProducer::build_from_occurrences(input, binding_env, arena, scope)
    }
    .map_err(|error| error.to_string())
}

fn edge(
    formula: SourceAtomicFormulaId,
    ordinal: usize,
    role: SourceAtomicEdgeRole,
    target: SourceAtomicTermTarget,
) -> SourceAtomicEdgeInput {
    SourceAtomicEdgeInput {
        formula,
        ordinal,
        role,
        target,
    }
}

fn primary_target(
    handoff: &SourcePrimaryTermHandoff,
    range: SourceRange,
) -> Result<SourceAtomicTermTarget, String> {
    handoff
        .terms()
        .iter()
        .find(|(_, term)| term.parent().is_none() && term.source_range() == range)
        .map(|(id, _)| SourceAtomicTermTarget::Primary(id))
        .ok_or_else(|| "Task256 direct primary slot is not a Task252 root".to_owned())
}

fn set_target(
    handoff: &SourceSetTermHandoff,
    range: SourceRange,
) -> Result<SourceAtomicTermTarget, String> {
    handoff
        .terms()
        .iter()
        .find(|(id, term)| {
            (term.source_range() == range
                || handoff.wrappers().iter().any(|(_, wrapper)| {
                    wrapper.term() == *id
                        && wrapper.ordinal() == 0
                        && wrapper.source_range() == range
                }))
                && !handoff.edges().iter().any(
                    |(_, edge)| matches!(edge.target(), mizar_checker::source_set_term::SourceSetTarget::SetTerm(target) if target == *id),
                )
        })
        .map(|(id, _)| SourceAtomicTermTarget::SetTerm(id))
        .ok_or_else(|| "Task256 direct set slot is not a Task255 root".to_owned())
}

fn exact_symbol<'a>(
    symbols: &'a SymbolEnv,
    symbol: &SymbolId,
    kind: SymbolKind,
) -> Result<&'a mizar_resolve::env::SymbolEntry, String> {
    let entry = symbols
        .symbols()
        .get(symbol)
        .ok_or_else(|| "Task256 resolver symbol disappeared".to_owned())?;
    if entry.kind() != kind {
        return Err("Task256 resolver symbol kind changed".to_owned());
    }
    Ok(entry)
}

fn insert_kind(
    kinds: &mut BTreeMap<usize, &'static str>,
    node: usize,
    kind: &'static str,
) -> Result<(), String> {
    if kinds.insert(node, kind).is_some() {
        return Err("Task256 typed-arena site is multiply owned".to_owned());
    }
    Ok(())
}

fn formula_kind_key(kind: SourceAtomicFormulaKind) -> &'static str {
    match kind {
        SourceAtomicFormulaKind::PredicateApplication => "source.formula.atomic.predicate",
        SourceAtomicFormulaKind::Equality => "source.formula.atomic.equality",
        SourceAtomicFormulaKind::Inequality => "source.formula.atomic.inequality",
        SourceAtomicFormulaKind::Membership => "source.formula.atomic.membership",
        SourceAtomicFormulaKind::TypeAssertion => "source.formula.atomic.type-assertion",
        SourceAtomicFormulaKind::AttributeAssertion => "source.formula.atomic.attribute-assertion",
        _ => "source.formula.atomic.unsupported",
    }
}

fn unique_descendant(
    ast: &SurfaceAst,
    root: usize,
    predicate: impl Fn(&SurfaceNodeKind) -> bool,
) -> Result<SurfaceNodeId, String> {
    fn collect(
        ast: &SurfaceAst,
        root: usize,
        predicate: &impl Fn(&SurfaceNodeKind) -> bool,
        output: &mut Vec<SurfaceNodeId>,
    ) {
        for child in structural_child_ids(ast, &ast.nodes()[root]) {
            let index = child.index();
            if predicate(&ast.nodes()[index].kind) {
                output.push(child);
            }
            collect(ast, index, predicate, output);
        }
    }
    let mut matches = Vec::new();
    collect(ast, root, &predicate, &mut matches);
    match matches.as_slice() {
        [node] => Ok(*node),
        _ => Err("Task256 expected exactly one source descendant".to_owned()),
    }
}

fn unique_direct_token(
    ast: &SurfaceAst,
    owner: usize,
    spelling: &str,
) -> Result<SurfaceNodeId, String> {
    let matches = ast.nodes()[owner]
        .children
        .iter()
        .copied()
        .filter(|child| ast.nodes()[child.index()].token_text() == Some(spelling))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [node] => Ok(*node),
        _ => Err(format!(
            "Task256 expected one direct `{spelling}` source token"
        )),
    }
}

#[cfg(test)]
#[test]
fn condition_composition_rejects_genuine_outside_primary_group() {
    use mizar_checker::{
        source_application::{
            SourceFunctorApplicationHandoffInput, SourceFunctorApplicationProducer,
        },
        source_formula_composition::{
            SourceConditionFormulaCompositionHandoffInput,
            SourceConditionFormulaCompositionProducer, SourceConditionFormulaEdgeInput,
        },
        type_checker::SourceVariableSemanticsChecker,
    };
    use mizar_resolve::{
        declarations::DeclarationShellCollector,
        env::NamespacePath,
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::SignatureProjectionExtractor,
    };
    let original = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tests/miz/pass/terms/pass_proof_verification_term_comprehension_guarded_001.miz",
    ))
    .unwrap();
    let text = original.replacen("st x in A holds", "st (x) in A holds", 1);
    let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.unwrap();
    let module = ModuleId::new(
        mizar_session::PackageId::new("formula-mutation"),
        mizar_session::ModulePath::new("formula_mutation"),
    );
    let shells = DeclarationShellCollector::new(&ast, &module).collect();
    let collected = SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .collect(&module);
    assert!(collected.diagnostics().is_empty());
    let symbols = collected.env();
    let scope = SourceVariableScopeResolver::resolve_occurrences(SourceVariableScopeInput::new(
        &ast, &module, symbols,
    ))
    .unwrap();
    let bindings = SourceVariableSemanticsChecker::occurrence_binding_env(&scope);
    let outside = ast
        .node_views()
        .find(|view| matches!(view.kind(), SurfaceNodeKind::ParenthesizedTerm))
        .unwrap();
    let group = ast.node(outside.id()).unwrap();
    assert_eq!(&text[group.range.start..group.range.end], "(x)");
    let bridges = structural_child_ids(&ast, group);
    let [bridge_id] = bridges.as_slice() else {
        panic!("outside Primary bridge")
    };
    let bridge = ast.node(*bridge_id).unwrap();
    assert!(matches!(bridge.kind, SurfaceNodeKind::TermExpression));
    let leaves = structural_child_ids(&ast, bridge);
    let [leaf_id] = leaves.as_slice() else {
        panic!("outside Primary leaf")
    };
    let outside_leaf = *leaf_id;
    let mut nodes = membership_typed_arena(&ast)
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, (_, node))| {
            let mut node = node.clone();
            if group.children.iter().any(|child| child.index() == index) {
                node.kind = format!("{:?}", ast.nodes()[index].kind).into();
            }
            node
        })
        .collect::<Vec<_>>();
    nodes[outside.id().index()].kind = "source.term.parenthesized".into();
    let arena =
        TypedArena::try_new(ast.root().map(|id| TypedNodeId::new(id.index())), nodes).unwrap();
    let mut terms = Vec::new();
    let mut references = Vec::new();
    let mut outside_root = None;
    for actual in scope.references() {
        let parent = if actual.node() == outside_leaf {
            let parent = SourcePrimaryTermId::new(terms.len());
            terms.push(SourcePrimaryTermInput {
                site: surface_site(outside.id()),
                source_range: group.range,
                source_ordinal: terms.len(),
                context: BindingContextId::new(0),
                recovery: SourcePrimaryTermRecovery::Normal,
                spelling: "( x )".into(),
                kind: SourcePrimaryTermKind::Parenthesized,
                role: SourcePrimaryTermRole::Value,
                parent: None,
            });
            outside_root = Some(parent);
            Some(parent)
        } else {
            None
        };
        let term = SourcePrimaryTermId::new(terms.len());
        terms.push(SourcePrimaryTermInput {
            site: surface_site(actual.node()),
            source_range: actual.range(),
            source_ordinal: terms.len(),
            context: BindingContextId::new(0),
            recovery: SourcePrimaryTermRecovery::Normal,
            spelling: actual.spelling().into(),
            kind: SourcePrimaryTermKind::VariableReference,
            role: SourcePrimaryTermRole::Value,
            parent,
        });
        references.push(SourcePrimaryTermReferenceInput {
            term,
            binding: BindingId::new(actual.binding().index()),
            role: SourcePrimaryTermReferenceRole::Variable,
        });
    }
    let primary = SourcePrimaryTermProducer::build_from_occurrences(
        SourcePrimaryTermHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            terms,
            references,
            numeric_type_requests: vec![],
        },
        &bindings,
        &arena,
        &scope,
    )
    .expect("genuine sealed outside Primary group");
    let outside_root = outside_root.unwrap();
    let roots = ast
        .node_views()
        .filter(|view| matches!(view.kind(), SurfaceNodeKind::SetComprehension))
        .map(|view| view.id().index())
        .collect::<Vec<_>>();
    let set_output = step5c7_set_output(
        &ast,
        module.clone(),
        bindings.clone(),
        &roots,
        SourceTermParts {
            arena,
            handoff: primary.clone(),
        },
    )
    .expect("genuine Set lower with outside Primary metadata");
    let sets = set_output.typed_ast.source_set_term().unwrap();
    let restored = membership_restore_arena(&ast, set_output.typed_ast.nodes()).unwrap();
    let mut nodes = restored
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    let mut atoms = ast
        .node_views()
        .filter(|view| {
            matches!(view.kind(), SurfaceNodeKind::BuiltinPredicateApplication)
                && direct_token_texts(&ast, ast.node(view.id()).unwrap()) == ["in"]
        })
        .map(|view| view.id())
        .collect::<Vec<_>>();
    atoms.sort_by_key(|id| ast.node(*id).unwrap().range.start);
    let mut formulas = Vec::new();
    let mut edges = Vec::new();
    let mut requests = Vec::new();
    for (ordinal, id) in atoms.into_iter().enumerate() {
        let node = ast.node(id).unwrap();
        nodes[id.index()].kind = "source.formula.atomic.membership".into();
        let formula = SourceAtomicFormulaId::new(ordinal);
        formulas.push(SourceAtomicFormulaInput {
            site: surface_site(id),
            source_range: node.range,
            source_ordinal: ordinal,
            context: BindingContextId::new(0),
            recovery: SourceAtomicFormulaRecovery::Normal,
            spelling: surface_text(&ast, node),
            kind: SourceAtomicFormulaKind::Membership,
        });
        for (operand_ordinal, operand) in structural_child_ids(&ast, node).into_iter().enumerate() {
            let children = structural_child_ids(&ast, ast.node(operand).unwrap());
            let [child] = children.as_slice() else {
                panic!("actual outside atom operand")
            };
            let child = ast.node(*child).unwrap();
            let target = if matches!(
                child.kind,
                SurfaceNodeKind::TermReference | SurfaceNodeKind::ParenthesizedTerm
            ) {
                primary_target(&primary, child.range).unwrap()
            } else {
                set_target(sets, child.range).unwrap()
            };
            edges.push(edge(
                formula,
                operand_ordinal,
                if operand_ordinal == 0 {
                    SourceAtomicEdgeRole::BuiltinLeftOperand
                } else {
                    SourceAtomicEdgeRole::BuiltinRightOperand
                },
                target,
            ));
        }
        requests.push(SourceAtomicRequestInput {
            formula,
            ordinal: 0,
            kind: SourceAtomicRequestKind::OperandExpectedType,
            edge: Some(SourceAtomicEdgeId::new(edges.len() - 1)),
            candidate: None,
            type_site: None,
            attribute: None,
        });
    }
    let arena = TypedArena::try_new(restored.root(), nodes).unwrap();
    let atomic = SourceAtomicFormulaProducer::build(
        SourceAtomicFormulaHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            formulas,
            wrappers: vec![],
            predicate_segments: vec![],
            predicate_heads: vec![],
            candidates: vec![],
            type_sites: vec![],
            attributes: vec![],
            edges,
            requests,
        },
        &bindings,
        symbols,
        &primary,
        None,
        None,
        Some(sets),
        &arena,
    )
    .expect("genuine outside grouped operand Atomic lower");
    assert!(
        atomic
            .edges()
            .iter()
            .any(|(_, edge)| edge.target() == SourceAtomicTermTarget::Primary(outside_root))
    );
    let application = SourceFunctorApplicationProducer::build(
        SourceFunctorApplicationHandoffInput {
            source_id: ast.source_id,
            module_id: module.clone(),
            applications: vec![],
            wrappers: vec![],
            candidates: vec![],
            arguments: vec![],
            type_requests: vec![],
        },
        symbols,
        &bindings,
        &primary,
        &arena,
    )
    .unwrap();
    let edges = sets
        .conditions()
        .iter()
        .map(|(id, condition)| {
            let children = structural_child_ids(
                &ast,
                &ast.nodes()[condition.condition_site().node().index()],
            );
            let [atom] = children.as_slice() else {
                panic!("actual bare condition atom")
            };
            let (formula, _) = atomic
                .formulas()
                .iter()
                .find(|(_, formula)| formula.site().node().index() == atom.index())
                .unwrap();
            SourceConditionFormulaEdgeInput {
                condition: id,
                ordinal: condition.ordinal(),
                formula,
            }
        })
        .collect();
    let input = SourceConditionFormulaCompositionHandoffInput {
        source_id: ast.source_id,
        module_id: module,
        edges,
    };
    assert!(input.edges.iter().all(|edge| {
        atomic
            .edges()
            .iter()
            .filter(|(_, operand)| operand.formula() == edge.formula)
            .all(|(_, operand)| operand.target() != SourceAtomicTermTarget::Primary(outside_root))
    }));
    assert!(
        SourceConditionFormulaCompositionProducer::build(
            input,
            &primary,
            &application,
            sets,
            &atomic,
            &arena
        )
        .is_err(),
        "outside grouped Primary root admitted by native C2"
    );
}
