use std::collections::BTreeMap;

use mizar_checker::{
    binding_env::{BindingContextId, BindingEnv},
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_atomic_formula::{
        SourceAssertionAttributeInput, SourceAssertionAttributePolarityInput,
        SourceAssertionTypeHead, SourceAssertionTypeSiteInput, SourceAtomicEdgeId,
        SourceAtomicEdgeInput, SourceAtomicEdgeRole, SourceAtomicFormulaHandoffInput,
        SourceAtomicFormulaId, SourceAtomicFormulaInput, SourceAtomicFormulaKind,
        SourceAtomicFormulaProducer, SourceAtomicFormulaRecovery, SourceAtomicRequestInput,
        SourceAtomicRequestKind, SourceAtomicTermTarget, SourcePredicateCandidateId,
        SourcePredicateCandidateInput, SourcePredicateHeadId, SourcePredicateHeadInput,
        SourcePredicateSegmentInput, SourcePredicateSegmentPolarityInput,
    },
    source_set_term::SourceSetTermHandoff,
    source_term::SourcePrimaryTermHandoff,
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, TypeDiagnosticTable,
        TypeFactTable, TypeTable, TypedAst, TypedAstParts, TypedSiteRef,
    },
};
use mizar_resolve::{
    env::{SymbolEnv, SymbolKind},
    resolved_ast::{ModuleId, SymbolId},
};
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind};

use super::{
    checker_handoff::{assemble_empty_resolved_typed_ast, source_module_binding_env},
    source_application::{
        imported_source_application_output_with_source_term,
        imported_source_application_owned_node_kinds,
    },
    source_ast::{structural_child_ids, surface_site},
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
    source_set_term::source_set_term_output_with_source_term,
    source_term::{SourceTermParts, source_term_parts_for_roots},
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

#[cfg(test)]
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
    Some(build_output(ast, module, symbols, route, mutate))
}

fn exact_atomic_route(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    source_text: Option<&str>,
) -> Option<ExactAtomicRoute> {
    if let Some(payload) = source_text.and_then(|source_text| {
        extract_source_imported_predicate_chain_formula(ast, module, symbols, source_text)
    }) {
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
    let source_term = source_term_parts_for_roots(
        ast,
        module.clone(),
        &binding_env,
        primary_roots,
        BindingContextId::new(0),
        &owned_kinds,
    )?;
    let mut typed_ast = lower_family_typed_ast(
        ast,
        module.clone(),
        symbols,
        &binding_env,
        &route,
        source_term,
    )?;
    let mut input = route.handoff_input(ast, symbols, &typed_ast)?;
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
) -> Result<TypedAst, String> {
    match route {
        ExactAtomicRoute::Predicate(_) => {
            let output = imported_source_application_output_with_source_term(
                ast,
                module,
                symbols,
                binding_env.clone(),
                source_term,
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

    fn formula_range(&self) -> SourceRange {
        match self {
            Self::Binary(payload) => payload.formula_range,
            Self::TypeAssertion(payload) => payload.formula_range,
            Self::Predicate(payload) => payload.formula_range,
            Self::PredicateChain(payload) => payload.formula_range,
            Self::Attribute { payload, .. } => payload.formula_range,
            Self::SetEnumeration(payload) => payload.formula_range,
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
                .map(|(site, _)| site.node().index())
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
    ) -> Result<SourceAtomicFormulaHandoffInput, String> {
        let formula = SourceAtomicFormulaId::new(0);
        let formula_node = ast
            .nodes()
            .get(self.formula_node())
            .ok_or_else(|| "Task256 formula node disappeared".to_owned())?;
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
                    payload.left_range,
                )?;
                let right = primary_target(
                    typed_ast
                        .source_term()
                        .ok_or_else(|| "Task256 lost Task252".to_owned())?,
                    payload.right_range,
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
                    payload.subject_range,
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
                let head_node = &ast.nodes()[head.index()];
                type_sites.push(SourceAssertionTypeSiteInput {
                    formula,
                    site: payload.asserted_type_site.clone(),
                    source_range: payload.asserted_type.range,
                    spelling: payload.asserted_type.spelling.clone(),
                    head_site: surface_site(head),
                    head_range: head_node.range,
                    head_spelling: subtree_tokens(ast, head_node).join(" "),
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
                let head = &ast.nodes()[head_node.index()];
                predicate_heads.push(SourcePredicateHeadInput {
                    formula,
                    site: surface_site(head_node),
                    source_range: head.range,
                    context: BindingContextId::new(0),
                    recovery: SourceAtomicFormulaRecovery::Normal,
                    spelling: subtree_tokens(ast, head).join(" "),
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
                    payload.left_range,
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
                    .term_ranges
                    .iter()
                    .copied()
                    .map(|range| primary_target(primary, range))
                    .collect::<Result<Vec<_>, _>>()?;
                let entry =
                    exact_symbol(symbols, &payload.predicate_symbol, SymbolKind::Predicate)?;

                for ordinal in 0..2 {
                    predicate_heads.push(SourcePredicateHeadInput {
                        formula,
                        site: payload.head_sites[ordinal].clone(),
                        source_range: payload.head_ranges[ordinal],
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "divides".to_owned(),
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
                predicate_segments.extend([
                    SourcePredicateSegmentInput {
                        formula,
                        ordinal: 0,
                        site: payload.segment_sites[0].clone(),
                        source_range: payload.segment_ranges[0],
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "1 divides 2".to_owned(),
                        head: SourcePredicateHeadId::new(0),
                        polarity: SourcePredicateSegmentPolarityInput::Positive,
                        left_edge: SourceAtomicEdgeId::new(0),
                        right_edge: SourceAtomicEdgeId::new(1),
                    },
                    SourcePredicateSegmentInput {
                        formula,
                        ordinal: 1,
                        site: payload.segment_sites[1].clone(),
                        source_range: payload.segment_ranges[1],
                        context: BindingContextId::new(0),
                        recovery: SourceAtomicFormulaRecovery::Normal,
                        spelling: "does not divides 3".to_owned(),
                        head: SourcePredicateHeadId::new(1),
                        polarity: SourcePredicateSegmentPolarityInput::Negative {
                            verb_site: payload.verb_site.clone(),
                            verb_range: payload.verb_range,
                            verb_spelling: "does".to_owned(),
                            verb_recovery: SourceAtomicFormulaRecovery::Normal,
                            not_site: payload.not_site.clone(),
                            not_range: payload.not_range,
                            not_spelling: "not".to_owned(),
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
                    payload.subject_range,
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
                let attribute_node = &ast.nodes()[attribute.index()];
                let target_node = &ast.nodes()[target.index()];
                let entry =
                    exact_symbol(symbols, &payload.attribute_symbol, SymbolKind::Attribute)?;
                let polarity = if *negative {
                    let non = unique_direct_token(ast, attribute.index(), "non")?;
                    let non_node = &ast.nodes()[non.index()];
                    SourceAssertionAttributePolarityInput::Negative {
                        non_site: surface_site(non),
                        non_range: non_node.range,
                        non_spelling: "non".to_owned(),
                        non_recovery: SourceAtomicFormulaRecovery::Normal,
                    }
                } else {
                    SourceAssertionAttributePolarityInput::Positive
                };
                attributes.push(SourceAssertionAttributeInput {
                    formula,
                    ordinal: 0,
                    site: surface_site(attribute),
                    source_range: attribute_node.range,
                    spelling: subtree_tokens(ast, attribute_node).join(" "),
                    target_site: surface_site(target),
                    target_range: target_node.range,
                    target_spelling: subtree_tokens(ast, target_node).join(" "),
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
                let left = set_target(set_terms, payload.left_range)?;
                let right = set_target(set_terms, payload.right_range)?;
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
                site: self.formula_site(),
                source_range: self.formula_range(),
                source_ordinal: 0,
                context: BindingContextId::new(0),
                recovery: SourceAtomicFormulaRecovery::Normal,
                spelling: subtree_tokens(ast, formula_node).join(" "),
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
            term.source_range() == range
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

fn subtree_tokens<'a>(ast: &'a SurfaceAst, node: &'a SurfaceNode) -> Vec<&'a str> {
    fn visit<'a>(ast: &'a SurfaceAst, node: &'a SurfaceNode, output: &mut Vec<&'a str>) {
        if let Some(token) = node.token_text() {
            output.push(token);
            return;
        }
        for child in &node.children {
            if let Some(child) = ast.node(*child) {
                visit(ast, child, output);
            }
        }
    }
    let mut output = Vec::new();
    visit(ast, node, &mut output);
    output
}
