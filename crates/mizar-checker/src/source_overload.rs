//! Authenticated source transport into the existing overload pipeline.

use crate::{
    overload_resolution::*,
    type_checker::{TypeExpressionInput, TypeHeadInput, TypeNormalizationOutput, TypeNormalizer},
    typed_ast::{
        CoercionKind, CoercionStatus, FactStatus, Polarity, TypeEntryActual, TypedAst, TypedNodeId,
        TypedSiteRef,
    },
};
use mizar_resolve::{
    env::{SymbolEnv, SymbolKind},
    names::{
        NameReferenceCandidate, NameSymbolProjection, SymbolNameResolver, resolve_template_formal,
    },
    resolved_ast::{ModuleId, NameResolution, ReferenceSite, ResolvedNodeId, SurfaceResolvedArena},
};
use mizar_session::{SourceAnchor, SourceId};
use mizar_syntax::{SurfaceNodeId, SurfaceNodeKind as K};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceOverloadInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub sites: Vec<OverloadSiteInput>,
    pub candidates: Vec<OverloadCandidateInput>,
    pub viability: Vec<CandidateViabilityInput>,
    pub comparisons: Vec<SpecificityComparisonInput>,
    pub resolution: Vec<OverloadSiteResolutionInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceOverloadHandoff {
    original: Arc<TypedAst>,
    normalization: TypeNormalizationOutput,
    collection: OverloadCollectionOutput,
    expansion: TemplateExpansionOutput,
    viability: CandidateViabilityOutput,
    specificity: SpecificityGraphOutput,
    selection: OverloadSelectionOutput,
}

fn invalid() -> String {
    "source_overload.unsupported_or_unmatched_source".to_owned()
}
fn site(id: ResolvedNodeId) -> TypedSiteRef {
    TypedSiteRef::Node(TypedNodeId::new(id.index()))
}

pub(crate) fn select_algorithm_calls(
    source: &SurfaceResolvedArena,
    symbols: &SymbolEnv,
    typed: &TypedAst,
    definiens: &mut BTreeMap<SurfaceNodeId, mizar_resolve::resolved_ast::SymbolRef>,
    body: &mut BTreeMap<SurfaceNodeId, mizar_resolve::resolved_ast::SymbolRef>,
    originals: &mut BTreeMap<SurfaceNodeId, mizar_resolve::resolved_ast::SymbolRef>,
) -> Result<(), String> {
    use crate::typed_ast::{TypedAstParts, TypedNode};
    use mizar_resolve::{env::SymbolEntry, resolved_ast::SymbolRef};
    let ast = mizar_resolve::symbols::validated_source_symbol_ast(source, symbols)?;
    let surface_ids: Vec<_> = ast.node_views().map(|node| node.id()).collect();
    let surface = |id: ResolvedNodeId| surface_ids[id.index()];
    let rows = crate::type_checker::source_nodes(source, typed)?;
    let mut neutral = TypedAst::try_new(TypedAstParts {
        source_id: source.source_id(),
        module_id: source.module().clone(),
        resolved_root: Some(source.arena().root()),
        nodes: typed.nodes().clone(),
        source_context: None,
        source_type: None,
        source_attribute: None,
        contexts: Default::default(),
        types: Default::default(),
        facts: Default::default(),
        coercions: Default::default(),
        initial_obligations: Default::default(),
        diagnostics: Default::default(),
    })
    .map_err(|_| invalid())?;
    if typed.source_node_entries().is_some() {
        neutral
            .install_source_nodes(source)
            .map_err(|_| invalid())?;
    }
    if &neutral != typed {
        return Err(invalid());
    }
    for (id, current) in source.arena().iter() {
        let expected = TypedNode::new(
            format!("{:?}", current.kind()),
            current.origin().anchor().clone(),
        )
        .with_resolved_node(id)
        .with_children(
            current
                .children()
                .iter()
                .map(|id| TypedNodeId::new(id.index()))
                .collect(),
        );
        if current.origin().is_recovered()
            || matches!(current.kind(), K::ErrorRecovery(_))
            || typed.nodes().node(TypedNodeId::new(id.index())) != Some(&expected)
        {
            return Err(invalid());
        }
    }
    let node = |id| source.arena().node(id).ok_or_else(invalid);
    let range = |id| match node(id)?.origin().anchor() {
        SourceAnchor::Range(r) => Ok(*r),
        _ => Err(invalid()),
    };
    let text = |id| match node(id).ok()?.kind() {
        K::Token(t) => Some(t.text.as_ref()),
        _ => None,
    };
    let leaves = |id| -> Result<Vec<ResolvedNodeId>, String> {
        let span = range(id)?;
        let mut tokens: Vec<_> = rows
            .iter()
            .filter(|row| {
                matches!(row.kind, K::Token(_))
                    && span.start <= row.range.start
                    && row.range.end <= span.end
            })
            .collect();
        tokens.sort_by_key(|row| row.range.start);
        Ok(tokens.into_iter().map(|row| row.site).collect())
    };
    let direct = |mut id| -> Option<ResolvedNodeId> {
        loop {
            match node(id).ok()?.kind() {
                K::Token(_) => return Some(id),
                K::TermExpression
                | K::TermReference
                | K::QualifiedSymbol
                | K::PathSegment
                | K::TermList => {
                    let [child] = node(id).ok()?.children() else {
                        return None;
                    };
                    id = *child;
                }
                _ => return None,
            }
        }
    };
    let parents: BTreeMap<_, _> = rows
        .iter()
        .filter_map(|r| r.parent.map(|p| (r.site, p)))
        .collect();
    let ancestor = |mut id, kind: &K| -> Option<ResolvedNodeId> {
        loop {
            if node(id).ok()?.kind() == kind {
                return Some(id);
            }
            id = *parents.get(&id)?;
        }
    };
    let mut expressions = Vec::new();
    for (id, _) in source
        .arena()
        .iter()
        .filter(|(_, n)| n.kind() == &K::TypeExpression)
    {
        let tokens = leaves(id)?;
        if let [token] = tokens.as_slice() {
            let head = match text(*token) {
                Some("set") => TypeHeadInput::BuiltinSet,
                Some("object") => TypeHeadInput::BuiltinObject,
                _ => continue,
            };
            expressions.push(TypeExpressionInput::new(
                site(id),
                range(id)?,
                text(*token).unwrap_or_default(),
                head,
            ));
        }
    }
    let normalization = TypeNormalizer::default().normalize(symbols, expressions);
    if !normalization.diagnostics().is_empty() {
        return Err(invalid());
    }
    let normalized = |id| {
        normalization.type_entries().iter().find_map(|(_, e)| {
            if e.owner == site(id)
                && let TypeEntryActual::Known(ty) = e.actual
            {
                Some(ty)
            } else {
                None
            }
        })
    };
    let bindings = |owner, formal| -> Vec<ResolvedNodeId> {
        let Some(block) = ancestor(owner, &K::DefinitionBlockItem) else {
            return Vec::new();
        };
        node(block)
            .into_iter()
            .flat_map(|block| block.children())
            .filter(|child| {
                node(**child).is_ok_and(|n| n.kind() == &K::DefinitionParameter)
                    && range(**child)
                        .is_ok_and(|r| range(owner).is_ok_and(|owner| r.end <= owner.start))
            })
            .flat_map(|child| node(*child).into_iter().flat_map(|n| n.children()))
            .filter(|segment| {
                node(**segment).is_ok_and(|n| n.kind() == &K::QualifiedVariableSegment)
            })
            .filter_map(|segment| {
                node(*segment)
                    .ok()?
                    .children()
                    .iter()
                    .take_while(|id| !matches!(text(**id), Some("be" | "being")))
                    .any(|id| text(*id) == text(formal))
                    .then_some(*segment)
            })
            .collect()
    };
    let parameter = |owner, formal| -> Option<_> {
        let block = ancestor(owner, &K::DefinitionBlockItem)?;
        if node(block)
            .ok()?
            .children()
            .iter()
            .any(|child| node(*child).is_ok_and(|n| n.kind() == &K::TemplateParameter))
        {
            return None;
        }
        let matching = bindings(owner, formal);
        let [segment] = matching.as_slice() else {
            return None;
        };
        let [_, _, ty] = node(*segment).ok()?.children() else {
            return None;
        };
        Some(*ty)
    };
    let mut declarations = Vec::new();
    for entry in symbols.symbols().iter() {
        let Some((id, current)) = source
            .arena()
            .iter()
            .find(|(_, n)| n.origin().anchor() == entry.origin().anchor())
        else {
            return Err(invalid());
        };
        let pattern = current.children().iter().find(|id| {
            node(**id).is_ok_and(|n| matches!(n.kind(), K::FunctorPattern | K::NotationPattern))
        });
        let name = if let Some(pattern) = pattern {
            leaves(*pattern)?.first().copied()
        } else {
            leaves(id)?
                .into_iter()
                .find(|id| text(*id) == Some(entry.primary_spelling()))
        };
        if let Some(name) = name {
            declarations.push((id, entry, name));
        }
    }
    declarations.sort_by_key(|(id, ..)| range(*id).map(|r| r.start).unwrap_or_default());
    let projections: Vec<_> = declarations
        .iter()
        .map(|(id, e, name)| {
            Ok(NameSymbolProjection::current_module(
                e.symbol().clone(),
                e.namespace().clone(),
                text(*name).unwrap_or_default(),
                e.kind(),
                e.visibility(),
                range(*id)?,
                range(*id)?.end,
            ))
        })
        .collect::<Result<_, String>>()?;
    let resolve = |token, namespace| -> Result<Vec<&SymbolEntry>, String> {
        if rows.iter().any(|row| {
            matches!(
                row.kind,
                K::ReserveSegment | K::ImportAliasDecl | K::ModuleBranchImport
            )
        }) {
            return Ok(Vec::new());
        }
        let candidate = NameReferenceCandidate::unqualified(
            ReferenceSite::new(token, range(token)?, text(token).ok_or_else(invalid)?),
            node(token)?.origin().clone(),
            range(token)?.start,
        );
        let resolved = SymbolNameResolver::new(&projections, &[]).resolve(
            source.module(),
            namespace,
            &[candidate],
        );
        let targets = match resolved.table().iter().next().map(|(_, r)| r.resolution()) {
            Some(NameResolution::Resolved(r)) => vec![r.symbol()],
            Some(NameResolution::Ambiguous(r)) => {
                r.candidates().iter().map(|r| r.symbol()).collect()
            }
            _ => Vec::new(),
        };
        Ok(targets
            .into_iter()
            .filter_map(|s| symbols.symbols().get(s))
            .collect())
    };
    let select = |id: ResolvedNodeId,
                  argument: ResolvedNodeId,
                  entry: &SymbolEntry,
                  parameter,
                  result,
                  actual|
     -> Result<Option<SymbolRef>, String> {
        let key = OverloadSiteKey::new(format!("source-call:{}", id.index()));
        let collection = OverloadCollectionOutput::collect(
            vec![OverloadSiteInput {
                key: key.clone(),
                owner: site(id),
                source_range: range(id)?,
                kind: OverloadSiteKind::FunctorApplication,
                name: OverloadNameKey::new(entry.primary_spelling()),
                arguments: vec![site(argument)],
                expected: None,
                source_qua: Vec::new(),
                recovery: OverloadSiteRecovery::Normal,
            }],
            vec![OverloadCandidateInput {
                site: key,
                symbol: entry.symbol().clone(),
                ordinary_root: entry.symbol().clone(),
                // Callable engine role only; this does not lower or promote an algorithm.
                declaration_kind: CandidateDeclarationKind::Functor,
                parameters: vec![parameter],
                result: Some(result),
                origin: CandidateOrigin::Ordinary,
                template: None,
                coherence: None,
                provenance: CandidateProvenance {
                    stable_key: CandidateProvenanceKey::new(entry.symbol().fqn().as_str()),
                    source_range: match entry.origin().anchor() {
                        SourceAnchor::Range(range) => Some(*range),
                        _ => None,
                    },
                    scope: CandidateScope::Local,
                    declaration_order: 0,
                },
            }],
        );
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability = CandidateViabilityOutput::filter(
            &expansion,
            vec![CandidateViabilityInput {
                candidate: OverloadCandidateId::new(0),
                arguments: vec![ArgumentViabilityEvidence::Exact { actual }],
            }],
        );
        let graph = SpecificityGraphOutput::build(&viability, Vec::new());
        let selected = OverloadSelectionOutput::resolve(
            &graph,
            vec![OverloadSiteResolutionInput {
                site: OverloadSiteId::new(0),
                refinements: Vec::new(),
                refinement_join: RefinementJoinPayload {
                    status: RefinementJoinStatus::Compatible,
                    exposed_result: None,
                },
                inserted_views: Vec::new(),
            }],
        );
        Ok(selected.results().iter().find_map(|(_, r)| match r.status {
            OverloadResultStatus::Resolved { root, .. } => graph
                .candidates()
                .get(root)
                .map(|c| SymbolRef::new(c.symbol.clone(), range(id).expect("authenticated site"))),
            _ => None,
        }))
    };
    let mut signatures = BTreeMap::new();
    for (id, entry, _) in &declarations {
        if !matches!(entry.kind(), SymbolKind::Algorithm | SymbolKind::Functor) {
            continue;
        }
        let kind = if entry.kind() == SymbolKind::Algorithm {
            K::AlgorithmParameters
        } else {
            K::FunctorPattern
        };
        let Some(pattern) = node(*id)?
            .children()
            .iter()
            .find(|p| node(**p).is_ok_and(|n| n.kind() == &kind))
        else {
            continue;
        };
        let tokens = leaves(*pattern)?;
        let formal = match (entry.kind(), tokens.as_slice()) {
            (SymbolKind::Algorithm, [open, formal, close])
            | (SymbolKind::Functor, [_, open, formal, close])
                if text(*open) == Some("(") && text(*close) == Some(")") =>
            {
                *formal
            }
            _ => continue,
        };
        let Some(result) = node(*id)?
            .children()
            .iter()
            .find(|p| node(**p).is_ok_and(|n| n.kind() == &K::TypeExpression))
            .and_then(|id| normalized(*id))
        else {
            continue;
        };
        if let Some(ty) = parameter(*id, formal).and_then(normalized) {
            signatures.insert(entry.symbol().clone(), (ty, result));
        }
    }
    for (id, entry, _) in &declarations {
        if entry.kind() != SymbolKind::Synonym {
            continue;
        }
        let [synonym, alternate, _, original, _] = node(*id)?.children() else {
            continue;
        };
        let left = leaves(*alternate)?;
        let right = leaves(*original)?;
        let ([_, lo, formal, lc], [target, ro, actual, rc]) = (left.as_slice(), right.as_slice())
        else {
            continue;
        };
        if text(*synonym) != Some("synonym")
            || text(*lo) != Some("(")
            || text(*ro) != Some("(")
            || text(*lc) != Some(")")
            || text(*rc) != Some(")")
            || text(*formal) != text(*actual)
            || !bindings(*id, *target).is_empty()
        {
            continue;
        }
        let targets = resolve(*target, entry.namespace())?;
        let [target] = targets.as_slice() else {
            continue;
        };
        let Some((ty, result)) = signatures.get(target.symbol()).copied() else {
            continue;
        };
        if parameter(*id, *formal).and_then(normalized) != Some(ty) {
            continue;
        }
        if let Some(original) = select(*id, *actual, target, ty, result, ty)? {
            originals.insert(surface(*id), original);
            signatures.insert(entry.symbol().clone(), (ty, result));
        }
    }
    let (inventory, candidates) =
        mizar_resolve::symbols::source_algorithm_call_candidates(source, symbols)?;
    for (id, current) in source
        .arena()
        .iter()
        .filter(|(_, n)| n.kind() == &K::ApplicationTerm)
    {
        let [reference, _, actual, _] = current.children() else {
            continue;
        };
        let argument = *actual;
        let (Some(callee), Some(actual)) = (direct(*reference), direct(argument)) else {
            continue;
        };
        let (owner, anchor, is_body, targets) = if let Some(owner) = inventory.iter().find(|o| {
            o.calls()
                .iter()
                .any(|c| c.application() == surface(id) && c.region() == o.body())
        }) {
            (
                source
                    .resolved_node_for(owner.declaration())
                    .ok_or_else(invalid)?,
                id,
                true,
                candidates.get(&surface(id)).cloned().unwrap_or_default(),
            )
        } else if ancestor(id, &K::TermDefiniens).is_some()
            && let Some(owner) = ancestor(id, &K::FunctorDefinition)
        {
            let Some((_, entry, _)) = declarations.iter().find(|(decl, ..)| *decl == owner) else {
                continue;
            };
            (
                owner,
                *reference,
                false,
                resolve(callee, entry.namespace())?,
            )
        } else {
            continue;
        };
        if !bindings(owner, callee).is_empty() {
            continue;
        }
        let owner_range = range(owner)?;
        if rows.iter().any(|r| {
            owner_range.start <= r.range.start
                && r.range.end <= owner_range.end
                && matches!(
                    r.kind,
                    K::VariableDeclaration
                        | K::BinderExpression
                        | K::LetStatement
                        | K::GivenStatement
                        | K::ConsiderStatement
                        | K::SetStatement
                        | K::ReconsiderStatement
                        | K::ReconsiderItem
                        | K::Equating
                        | K::Witness
                        | K::InlineFunctorDefinition
                        | K::InlinePredicateDefinition
                        | K::TypedParameter
                        | K::ComprehensionVariableSegment
                        | K::QuantifierVariableSegment
                        | K::ProofBlock
                        | K::ForRangeStatement
                        | K::ForCollectionStatement
                        | K::MatchStatement
                        | K::QuantifiedFormula(_)
                )
        }) {
            continue;
        }
        let [target] = targets.as_slice() else {
            continue;
        };
        let Some((ty, result)) = signatures.get(target.symbol()).copied() else {
            continue;
        };
        let Some(actual_type) = parameter(owner, actual).and_then(normalized) else {
            continue;
        };
        if let Some(selected) = select(anchor, argument, target, ty, result, actual_type)? {
            if is_body {
                body.insert(surface(anchor), selected);
            } else {
                definiens.insert(surface(anchor), selected);
            }
        }
    }
    Ok(())
}

impl SourceOverloadHandoff {
    pub fn from_source(
        source: &SurfaceResolvedArena,
        symbols: &SymbolEnv,
        typed: &TypedAst,
    ) -> Result<(TypeNormalizationOutput, Self), String> {
        let (input, normalization) = SourceOverloadInput::from_source(source, symbols, typed)?;
        let handoff = Self::build(input, source, symbols, typed, &normalization)?;
        Ok((normalization, handoff))
    }

    pub fn build(
        input: SourceOverloadInput,
        source: &SurfaceResolvedArena,
        symbols: &SymbolEnv,
        typed: &TypedAst,
        normalization: &TypeNormalizationOutput,
    ) -> Result<Self, String> {
        let (expected, expected_normalization) =
            SourceOverloadInput::from_source(source, symbols, typed)?;
        if input.source_id != expected.source_id
            || input.module_id != expected.module_id
            || input.sites != expected.sites
            || input.candidates != expected.candidates
            || normalization != &expected_normalization
            || input.viability.len() != expected.viability.len()
            || input.resolution.len() != expected.resolution.len()
        {
            return Err(invalid());
        }
        for (actual, expected) in input.viability.iter().zip(&expected.viability) {
            if actual.candidate != expected.candidate
                || actual.arguments.len() != expected.arguments.len()
            {
                return Err(invalid());
            }
            let candidate = &input.candidates[actual.candidate.index()];
            let source_site = input
                .sites
                .iter()
                .find(|site| site.key == candidate.site)
                .ok_or_else(invalid)?;
            for (argument_index, (actual, expected)) in
                actual.arguments.iter().zip(&expected.arguments).enumerate()
            {
                let ArgumentViabilityEvidence::Exact {
                    actual: source_type,
                } = expected
                else {
                    return Err(invalid());
                };
                let recorded = match actual {
                    ArgumentViabilityEvidence::Exact { actual } => Some(*actual),
                    ArgumentViabilityEvidence::FactWidening {
                        actual,
                        target,
                        facts,
                        status,
                    } => {
                        if *target != candidate.parameters[argument_index]
                            || facts.is_empty()
                            || *status != ViabilityFactStatus::Consumable
                            || facts.iter().any(|id| {
                                typed.facts().get(*id).is_none_or(|fact| {
                                    fact.subject != source_site.arguments[argument_index]
                                        || fact.status != FactStatus::Known
                                        || fact.polarity != Polarity::Positive
                                        || normalization.normalized_types().get(*target).is_none_or(
                                            |target| {
                                                target.source.spelling != fact.predicate.as_str()
                                            },
                                        )
                                })
                            })
                        {
                            return Err(invalid());
                        }
                        Some(*actual)
                    }
                    ArgumentViabilityEvidence::Coercion {
                        actual,
                        target,
                        coercion,
                        kind,
                        status,
                        facts,
                        path,
                    } => {
                        let recorded = typed.coercions().get(*coercion).ok_or_else(invalid)?;
                        let expected_kind = match kind {
                            ViabilityCoercionKind::Widening => CoercionKind::Widening,
                            ViabilityCoercionKind::SourceQua => CoercionKind::SourceQua,
                            ViabilityCoercionKind::Narrowing => return Err(invalid()),
                        };
                        if *target != candidate.parameters[argument_index]
                            || recorded.site != source_site.arguments[argument_index]
                            || recorded.from != Some(*actual)
                            || recorded.to != *target
                            || recorded.kind != expected_kind
                            || recorded.status != CoercionStatus::Candidate
                            || recorded.obligation.is_some()
                            || *status != ViabilityCoercionStatus::Accepted
                            || recorded.supporting_facts != *facts
                            || facts.iter().any(|id| {
                                typed.facts().get(*id).is_none_or(|fact| {
                                    fact.subject != recorded.site
                                        || fact.status != FactStatus::Known
                                        || fact.polarity != Polarity::Positive
                                })
                            })
                            || (*kind == ViabilityCoercionKind::SourceQua
                                && !source_site.source_qua.iter().any(|written| {
                                    written.argument == recorded.site
                                        && written.target == *target
                                        && written.path == *path
                                        && written.evidence_facts == *facts
                                }))
                            || (*kind == ViabilityCoercionKind::Widening && path.is_some())
                        {
                            return Err(invalid());
                        }
                        Some(*actual)
                    }
                    ArgumentViabilityEvidence::Missing { actual, target }
                    | ArgumentViabilityEvidence::DeferredExternalDependency {
                        actual,
                        target,
                        ..
                    } if *target == candidate.parameters[argument_index] => *actual,
                    _ => return Err(invalid()),
                };
                if recorded != Some(*source_type) {
                    return Err(invalid());
                }
            }
        }
        for (actual, expected) in input.resolution.iter().zip(&expected.resolution) {
            if actual.site != expected.site
                || !actual.refinements.is_empty()
                || actual.refinement_join != expected.refinement_join
            {
                return Err(invalid());
            }
            let source_site = input.sites.get(actual.site.index()).ok_or_else(invalid)?;
            let mut arguments = BTreeSet::new();
            for view in &actual.inserted_views {
                if !arguments.insert(view.argument.clone())
                    || !source_site.source_qua.iter().any(|written| {
                        written.argument == view.argument
                            && written.target == view.target
                            && written.path == view.path
                            && written.evidence_facts == view.evidence_facts
                    })
                    || view.kind != InsertedViewKind::SourceQua
                {
                    return Err(invalid());
                }
            }
        }
        let collection = OverloadCollectionOutput::collect(input.sites, input.candidates);
        let expansion = TemplateExpansionOutput::expand(&collection);
        let viability = CandidateViabilityOutput::filter(&expansion, input.viability);
        let mut pairs = BTreeSet::new();
        for comparison in &input.comparisons {
            let left = viability
                .candidates()
                .get(comparison.left)
                .ok_or_else(invalid)?;
            let right = viability
                .candidates()
                .get(comparison.right)
                .ok_or_else(invalid)?;
            if left.site != right.site
                || comparison.left == comparison.right
                || comparison.reasons.is_empty()
                || !pairs.insert((
                    comparison.left.min(comparison.right),
                    comparison.left.max(comparison.right),
                ))
            {
                return Err(invalid());
            }
        }
        for resolution in &input.resolution {
            for view in &resolution.inserted_views {
                if !viability
                    .candidates()
                    .get(view.selected_candidate)
                    .is_some_and(|candidate| candidate.site == resolution.site)
                {
                    return Err(invalid());
                }
            }
        }
        for (_, graph_site) in collection.sites().iter() {
            let viable: Vec<_> = viability
                .decisions()
                .iter()
                .filter(|(_, row)| {
                    row.site == graph_site.id
                        && matches!(row.status, CandidateViabilityStatus::Viable { .. })
                })
                .map(|(_, row)| row.output_candidate.expect("viable output"))
                .collect();
            for (index, left) in viable.iter().enumerate() {
                for right in &viable[index + 1..] {
                    if !pairs.contains(&((*left).min(*right), (*left).max(*right))) {
                        return Err(invalid());
                    }
                }
            }
        }
        let specificity = SpecificityGraphOutput::build(&viability, input.comparisons);
        let selection = OverloadSelectionOutput::resolve(&specificity, input.resolution);
        Ok(Self {
            original: Arc::new(typed.clone()),
            normalization: normalization.clone(),
            collection,
            expansion,
            viability,
            specificity,
            selection,
        })
    }

    pub fn source_id(&self) -> SourceId {
        self.original.source_id()
    }
    pub fn module_id(&self) -> &ModuleId {
        self.original.module_id()
    }
    pub const fn collection(&self) -> &OverloadCollectionOutput {
        &self.collection
    }
    pub const fn expansion(&self) -> &TemplateExpansionOutput {
        &self.expansion
    }
    pub const fn viability(&self) -> &CandidateViabilityOutput {
        &self.viability
    }
    pub const fn specificity(&self) -> &SpecificityGraphOutput {
        &self.specificity
    }
    pub const fn selection(&self) -> &OverloadSelectionOutput {
        &self.selection
    }
    pub(crate) fn original(&self) -> &TypedAst {
        &self.original
    }
    pub(crate) fn normalization(&self) -> &TypeNormalizationOutput {
        &self.normalization
    }
    pub(crate) fn validate_installation(&self, typed: &TypedAst) -> Result<(), String> {
        if self
            .original
            .as_ref()
            .clone()
            .with_source_overload(self.clone())
            .as_ref()
            != Ok(typed)
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn debug_text(&self) -> String {
        format!(
            "source-overload-debug-v1\n{}{}{}{}{}",
            self.collection.debug_text(),
            self.expansion.debug_text(),
            self.viability.debug_text(),
            self.specificity.debug_text(),
            self.selection.debug_text()
        )
    }
}

impl SourceOverloadInput {
    /// Derives the supported unary builtin profile from authenticated original syntax.
    /// Comparisons remain explicit upstream inputs for contested sites.
    pub fn from_source(
        source: &SurfaceResolvedArena,
        symbols: &SymbolEnv,
        typed: &TypedAst,
    ) -> Result<(Self, TypeNormalizationOutput), String> {
        mizar_resolve::symbols::validate_source_symbol_env(source, symbols)?;
        crate::type_checker::source_nodes(source, typed)?;
        if typed.source_overload().is_some()
            || !typed.types().is_empty()
            || symbols
                .symbols()
                .iter()
                .any(|symbol| symbol.kind() == SymbolKind::Redefinition)
        {
            return Err(invalid());
        }
        let node = |id| source.arena().node(id).ok_or_else(invalid);
        let range = |id| match node(id)?.origin().anchor() {
            SourceAnchor::Range(range) => Ok(*range),
            _ => Err(invalid()),
        };
        let text = |id| match node(id)?.kind() {
            K::Token(token) => Ok(token.text.as_ref()),
            _ => Err(invalid()),
        };
        let only = |id| match node(id)?.children() {
            [child] => Ok(*child),
            _ => Err(invalid()),
        };
        let type_text = |id| {
            if node(id)?.kind() != &K::TypeExpression {
                return Err(invalid());
            }
            text(only(only(id)?)?)
        };
        let head = |spelling| match spelling {
            "set" => Ok(TypeHeadInput::BuiltinSet),
            "object" => Ok(TypeHeadInput::BuiltinObject),
            _ => Err(invalid()),
        };
        let mut parents = BTreeMap::new();
        for (id, current) in source.arena().iter() {
            if current.origin().is_recovered()
                || typed
                    .nodes()
                    .node(TypedNodeId::new(id.index()))
                    .is_none_or(|typed| typed.kind.as_str() != format!("{:?}", current.kind()))
            {
                return Err(invalid());
            }
            for child in current.children() {
                if id != source.arena().root() || !matches!(node(*child)?.kind(), K::Token(_)) {
                    parents.insert(*child, id);
                }
            }
        }
        let mut expressions = Vec::new();
        for (id, current) in source.arena().iter() {
            if current.kind() == &K::TypeExpression
                && let Ok(spelling) = type_text(id)
                && let Ok(head) = head(spelling)
            {
                expressions.push(TypeExpressionInput::new(
                    site(id),
                    range(id)?,
                    spelling,
                    head,
                ));
            }
        }
        let mut calls = Vec::new();
        for (id, current) in source.arena().iter() {
            let (callee, actuals, argument) = match current.kind() {
                K::ApplicationTerm => {
                    let [reference, open, argument, close] = current.children() else {
                        return Err(invalid());
                    };
                    if node(*reference)?.kind() != &K::TermReference
                        || text(*open)? != "("
                        || text(*close)? != ")"
                    {
                        return Err(invalid());
                    }
                    match node(*reference)?.children() {
                        [callee] => (*callee, None, *argument),
                        [callee, actuals] if node(*actuals)?.kind() == &K::TemplateArguments => {
                            (*callee, Some(*actuals), *argument)
                        }
                        _ => return Err(invalid()),
                    }
                }
                _ => continue,
            };
            let mut callee = callee;
            while matches!(node(callee)?.kind(), K::QualifiedSymbol | K::PathSegment) {
                callee = only(callee)?;
            }
            let mut base = argument;
            let mut qua = None;
            while matches!(node(base)?.kind(), K::TermExpression | K::ParenthesizedTerm) {
                base = match node(base)?.children() {
                    [child] => *child,
                    [_, child, _] if node(base)?.kind() == &K::ParenthesizedTerm => *child,
                    _ => return Err(invalid()),
                };
            }
            if node(base)?.kind() == &K::QuaExpression {
                let [operand, keyword, target] = node(base)?.children() else {
                    return Err(invalid());
                };
                if text(*keyword)? != "qua" {
                    return Err(invalid());
                }
                qua = Some((base, *target));
                base = *operand;
                while node(base)?.kind() == &K::TermExpression {
                    base = only(base)?;
                }
            }
            if node(base)?.kind() != &K::TermReference {
                return Err(invalid());
            }
            let mut reference = only(base)?;
            while matches!(node(reference)?.kind(), K::QualifiedSymbol | K::PathSegment) {
                reference = only(reference)?;
            }
            let binder = resolve_template_formal(source, reference).or_else(|_| {
                let mut owner = reference;
                while let Some(parent) = parents.get(&owner) {
                    owner = *parent;
                    if matches!(
                        node(owner)?.kind(),
                        K::DefinitionBlockItem | K::ProofBlock | K::QuantifiedFormula(_)
                    ) {
                        return Err(invalid());
                    }
                }
                let matching: Vec<_> = source
                    .arena()
                    .iter()
                    .filter_map(|(_, candidate)| {
                        if candidate.kind() != &K::ReserveSegment {
                            return None;
                        }
                        let [binder, _, _] = candidate.children() else {
                            return None;
                        };
                        (text(*binder).ok() == text(reference).ok()
                            && range(*binder).ok()?.end < range(reference).ok()?.start)
                            .then_some(*binder)
                    })
                    .collect();
                match matching.as_slice() {
                    [binder] => Ok(*binder),
                    _ => Err(invalid()),
                }
            })?;
            let binder_parent = *parents.get(&binder).ok_or_else(invalid)?;
            let binder_type = match node(binder_parent)?.children() {
                [_, _, ty] => *ty,
                [_, _, _, ty, _] if node(binder_parent)?.kind() == &K::TemplateParameter => *ty,
                _ => return Err(invalid()),
            };
            let source_spelling = type_text(binder_type)?;
            let spelling = if let Some((_, target)) = qua {
                let target = type_text(target)?;
                if source_spelling != target && !(source_spelling == "set" && target == "object") {
                    return Err(invalid());
                }
                target
            } else {
                source_spelling
            };
            expressions.push(TypeExpressionInput::new(
                site(argument),
                range(argument)?,
                spelling,
                head(spelling)?,
            ));
            calls.push((id, callee, actuals, argument, qua));
        }
        if calls.is_empty() {
            return Err(invalid());
        }
        let normalization = TypeNormalizer::default().normalize(symbols, expressions);
        if !normalization.diagnostics().is_empty() {
            return Err(invalid());
        }
        let normalized = |id| match normalization
            .type_entries()
            .iter()
            .find(|(_, entry)| entry.owner == site(id))
            .map(|(_, entry)| &entry.actual)
        {
            Some(TypeEntryActual::Known(id)) => Ok(*id),
            _ => Err(invalid()),
        };
        let mut input = Self {
            source_id: source.source_id(),
            module_id: source.module().clone(),
            sites: Vec::new(),
            candidates: Vec::new(),
            viability: Vec::new(),
            comparisons: Vec::new(),
            resolution: Vec::new(),
        };
        calls.sort_by_key(|(id, ..)| {
            range(*id)
                .map(|range| (range.start, range.end))
                .unwrap_or_default()
        });
        for (id, callee, actuals, argument, qua) in calls {
            let key = OverloadSiteKey::new(format!("source:{}", id.index()));
            let source_qua = qua
                .map(|(owner, target)| {
                    Ok::<_, String>(SourceQuaView {
                        argument: site(argument),
                        target: normalized(target)?,
                        source_range: range(owner)?,
                        path: None,
                        evidence_facts: Vec::new(),
                    })
                })
                .transpose()?
                .into_iter()
                .collect();
            input.sites.push(OverloadSiteInput {
                key: key.clone(),
                owner: site(id),
                source_range: range(id)?,
                kind: OverloadSiteKind::FunctorApplication,
                name: OverloadNameKey::new(text(callee)?),
                arguments: vec![site(argument)],
                expected: None,
                source_qua,
                recovery: OverloadSiteRecovery::Normal,
            });
            let mut declarations: Vec<_> = symbols
                .symbols()
                .iter()
                .filter(|entry| {
                    entry.kind() == SymbolKind::Functor
                        && source
                            .arena()
                            .iter()
                            .find(|(_, node)| {
                                node.kind() == &K::FunctorDefinition
                                    && node.origin().anchor() == entry.origin().anchor()
                            })
                            .and_then(|(_, definition)| definition.children().get(3))
                            .and_then(|pattern| node(*pattern).ok()?.children().first())
                            .is_some_and(|name| text(*name).ok() == text(callee).ok())
                        && entry.origin().source_id() == source.source_id()
                })
                .collect();
            declarations.sort_by_key(|entry| match entry.origin().anchor() {
                SourceAnchor::Range(range) => range.start,
                _ => usize::MAX,
            });
            if declarations.is_empty() {
                return Err(invalid());
            }
            let projections: Vec<_> = declarations
                .iter()
                .map(|entry| {
                    NameSymbolProjection::current_module(
                        entry.symbol().clone(),
                        entry.namespace().clone(),
                        text(callee).unwrap_or_default(),
                        entry.kind(),
                        entry.visibility(),
                        match entry.origin().anchor() {
                            SourceAnchor::Range(range) => *range,
                            _ => unreachable!(),
                        },
                        match entry.origin().anchor() {
                            SourceAnchor::Range(range) => range.end,
                            _ => usize::MAX,
                        },
                    )
                })
                .collect();
            let reference = NameReferenceCandidate::unqualified(
                ReferenceSite::new(callee, range(callee)?, text(callee)?),
                node(callee)?.origin().clone(),
                range(callee)?.start,
            );
            let resolved = SymbolNameResolver::new(&projections, &[]).resolve(
                source.module(),
                declarations[0].namespace(),
                &[reference],
            );
            let resolved_symbols: BTreeSet<_> = match resolved
                .table()
                .iter()
                .next()
                .map(|(_, row)| row.resolution())
            {
                Some(NameResolution::Resolved(reference)) => {
                    [reference.symbol().clone()].into_iter().collect()
                }
                Some(NameResolution::Ambiguous(reference)) => reference
                    .candidates()
                    .iter()
                    .map(|candidate| candidate.symbol().clone())
                    .collect(),
                _ => return Err(invalid()),
            };
            if resolved_symbols
                != declarations
                    .iter()
                    .map(|entry| entry.symbol().clone())
                    .collect()
            {
                return Err(invalid());
            }
            for (ordinal, entry) in declarations.into_iter().enumerate() {
                if !entry.relations().is_empty() {
                    return Err(invalid());
                }
                let definition = symbols
                    .definitions()
                    .by_symbol(entry.symbol())
                    .ok_or_else(invalid)?;
                if definition.conflict().is_some() {
                    return Err(invalid());
                }
                let (definition_id, _) = source
                    .arena()
                    .iter()
                    .find(|(_, current)| {
                        current.kind() == &K::FunctorDefinition
                            && current.origin().anchor() == entry.origin().anchor()
                    })
                    .ok_or_else(invalid)?;
                let [_, _, _, pattern, _, result, _, _, _] = node(definition_id)?.children() else {
                    return Err(invalid());
                };
                let (formal, loci) = match node(*pattern)?.children() {
                    [_, formal] => (*formal, None),
                    [_, loci, formal] if node(*loci)?.kind() == &K::TemplateLoci => {
                        (*formal, Some(*loci))
                    }
                    [_, open, formal, close] if text(*open)? == "(" && text(*close)? == ")" => {
                        (*formal, None)
                    }
                    [_, loci, open, formal, close]
                        if node(*loci)?.kind() == &K::TemplateLoci
                            && text(*open)? == "("
                            && text(*close)? == ")" =>
                    {
                        (*formal, Some(*loci))
                    }
                    _ => return Err(invalid()),
                };
                let mut block = definition_id;
                while node(block)?.kind() != &K::DefinitionBlockItem {
                    block = *parents.get(&block).ok_or_else(invalid)?;
                }
                let mut formal_types = Vec::new();
                for child in node(block)?.children() {
                    let current = node(*child)?;
                    let segment = match current.kind() {
                        K::DefinitionParameter => match current.children() {
                            [_, segment, _] => node(*segment)?,
                            _ => return Err(invalid()),
                        },
                        K::TemplateParameter => current,
                        _ => continue,
                    };
                    let (binder, ty) = match segment.children() {
                        [binder, _, ty] => (*binder, *ty),
                        [_, binder, _, ty, _] if node(*ty)?.kind() == &K::TypeExpression => {
                            (*binder, *ty)
                        }
                        _ => continue,
                    };
                    if text(binder)? == text(formal)?
                        && range(*child)?.end < range(definition_id)?.start
                    {
                        formal_types.push(ty);
                    }
                }
                let [parameter_type] = formal_types.as_slice() else {
                    return Err(invalid());
                };
                let parameter_type = *parameter_type;
                if range(block)?.end > range(id)?.start {
                    return Err(invalid());
                }
                let template = match (loci, actuals) {
                    (None, None) => None,
                    (Some(loci), Some(actuals)) => {
                        let [_, actual, _] = node(actuals)?.children() else {
                            return Err(invalid());
                        };
                        let actual = only(*actual)?;
                        let [_, locus, _] = node(loci)?.children() else {
                            return Err(invalid());
                        };
                        let type_binder = resolve_template_formal(source, only(*locus)?)?;
                        let type_parameter = *parents.get(&type_binder).ok_or_else(invalid)?;
                        let value_parameter = *parents.get(&parameter_type).ok_or_else(invalid)?;
                        let [let_kw, binder, be, type_kw, semi] = node(type_parameter)?.children()
                        else {
                            return Err(invalid());
                        };
                        if node(type_parameter)?.kind() != &K::TemplateParameter
                            || *binder != type_binder
                            || text(*let_kw)? != "let"
                            || text(*be)? != "be"
                            || text(*type_kw)? != "type"
                            || text(*semi)? != ";"
                            || parents.get(&type_parameter) != Some(&block)
                            || node(block)?.children().iter().any(|child| {
                                match node(*child).map(|node| node.kind()) {
                                    Ok(K::TemplateParameter) => {
                                        *child != type_parameter && *child != value_parameter
                                    }
                                    Ok(
                                        K::Token(_)
                                        | K::FunctorDefinition
                                        | K::CorrectnessCondition,
                                    ) => false,
                                    _ => true,
                                }
                            })
                        {
                            return Err(invalid());
                        }
                        if resolve_template_formal(source, only(only(parameter_type)?)?)?
                            != type_binder
                            || resolve_template_formal(source, only(only(*result)?)?)?
                                != type_binder
                        {
                            return Err(invalid());
                        }
                        Some(TemplateCandidatePayload {
                            template: entry.symbol().clone(),
                            instantiation_key: TemplateInstantiationKey::new(format!(
                                "source:{}:{}",
                                definition_id.index(),
                                id.index()
                            )),
                            parameters: vec![TemplateParameterKey::new(format!(
                                "source:{}",
                                type_binder.index()
                            ))],
                            arguments: vec![TemplateArgument::Explicit(normalized(actual)?)],
                            inferred_arguments: Vec::new(),
                            constraints: Vec::new(),
                        })
                    }
                    _ => return Err(invalid()),
                };
                let (parameter, result_type) = if let Some(template) = &template {
                    let TemplateArgument::Explicit(actual) = template.arguments[0] else {
                        return Err(invalid());
                    };
                    (actual, actual)
                } else {
                    (normalized(parameter_type)?, normalized(*result)?)
                };
                let candidate_id = OverloadCandidateId::new(input.candidates.len());
                input.candidates.push(OverloadCandidateInput {
                    site: key.clone(),
                    symbol: entry.symbol().clone(),
                    ordinary_root: entry.symbol().clone(),
                    declaration_kind: if template.is_some() {
                        CandidateDeclarationKind::Template
                    } else {
                        CandidateDeclarationKind::Functor
                    },
                    parameters: vec![parameter],
                    result: Some(result_type),
                    origin: CandidateOrigin::Ordinary,
                    template,
                    coherence: None,
                    provenance: CandidateProvenance {
                        stable_key: CandidateProvenanceKey::new(format!(
                            "source:{}",
                            definition_id.index()
                        )),
                        source_range: Some(range(definition_id)?),
                        scope: CandidateScope::Local,
                        declaration_order: ordinal,
                    },
                });
                input.viability.push(CandidateViabilityInput {
                    candidate: candidate_id,
                    arguments: vec![ArgumentViabilityEvidence::Exact {
                        actual: normalized(argument)?,
                    }],
                });
            }
            input.resolution.push(OverloadSiteResolutionInput {
                site: OverloadSiteId::new(input.sites.len() - 1),
                refinements: Vec::new(),
                refinement_join: RefinementJoinPayload {
                    status: RefinementJoinStatus::Compatible,
                    exposed_result: None,
                },
                inserted_views: Vec::new(),
            });
        }
        Ok((input, normalization))
    }
}
