use std::collections::{BTreeMap, BTreeSet};

use mizar_checker::{
    binding_env::{BindingContextId, BindingEnv},
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_application::{SourceFunctorApplicationHandoff, SourceFunctorApplicationId},
    source_context::{
        SourceBindingContextBuild, SourceBindingContextInput, SourceBindingContextOwner,
        SourceBindingContextProducer, SourceBindingSiteInput, SourceBindingSiteRole,
        SourceItemInput, SourceItemRecovery, SourceItemRole, SourceItemVisibility,
    },
    source_structure::{
        SourceFieldUpdateInput, SourceStructureEdgeInput, SourceStructureEdgeRole,
        SourceStructureHandoffInput, SourceStructureMemberId, SourceStructureMemberInput,
        SourceStructureMemberRole, SourceStructureProducer, SourceStructureRecovery,
        SourceStructureRequestInput, SourceStructureRequestKind, SourceStructureRootInput,
        SourceStructureTarget, SourceStructureTermId, SourceStructureTermInput,
        SourceStructureTermKind, SourceStructureWrapperInput,
    },
    source_term::SourcePrimaryTermHandoff,
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedAst, TypedAstParts,
        TypedNodeId, TypedSiteRef,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShell, DeclarationShellKind, DeclarationShellSet},
    env::{NamespacePath, SymbolEnv, SymbolKind},
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::{ModuleId, SymbolId},
};
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind};

#[cfg(not(test))]
use super::source_term::source_term_parts_for_roots;
#[cfg(test)]
use super::source_term::synthetic_source_term_parts_for_roots;
use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_ast::{
        direct_token_texts, exact_compilation_item_list, qualified_symbol_spelling,
        structural_child_ids, subtree_has_recovery, surface_site,
    },
    source_reserve::extract_builtin_source_reserve_declarations_after_node_guard,
};

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.typed_ast_invalid";
const PAYLOAD_EXTRACTION_GAP_KEY: &str =
    "type_elaboration.external_dependency.ast_payload_extraction";

#[derive(Debug)]
pub(in crate::runner) struct SourceStructureRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
    #[cfg(test)]
    pub(in crate::runner) binding_env: BindingEnv,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SyntheticSourceStructureDependencies {
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) primary: SourcePrimaryTermHandoff,
    pub(in crate::runner) application: Option<SourceFunctorApplicationHandoff>,
}

#[derive(Debug, Clone)]
struct ExtractedStructure {
    context: BindingContextId,
    terms: Vec<ExtractedTerm>,
    wrappers: Vec<ExtractedWrapper>,
    roots: Vec<ExtractedRoot>,
    members: Vec<ExtractedMember>,
    field_updates: Vec<ExtractedFieldUpdate>,
    edges: Vec<ExtractedEdge>,
    primary_roots: Vec<usize>,
}

#[derive(Debug, Clone)]
struct ExtractedTerm {
    node: usize,
    kind: SourceStructureTermKind,
    recovery: SourceStructureRecovery,
}

#[derive(Debug, Clone)]
struct ExtractedWrapper {
    term: SourceStructureTermId,
    ordinal: usize,
    node: usize,
    recovery: SourceStructureRecovery,
}

#[derive(Debug, Clone)]
struct ExtractedRoot {
    term: SourceStructureTermId,
    symbol: SymbolId,
    contribution: mizar_resolve::env::SourceContributionId,
}

#[derive(Debug, Clone)]
struct ExtractedMember {
    term: SourceStructureTermId,
    ordinal: usize,
    node: usize,
    role: SourceStructureMemberRole,
    parent: Option<SourceStructureMemberId>,
}

#[derive(Debug, Clone)]
struct ExtractedFieldUpdate {
    term: SourceStructureTermId,
    ordinal: usize,
    node: usize,
    first_member: SourceStructureMemberId,
    final_member: SourceStructureMemberId,
}

#[derive(Debug, Clone, Copy)]
enum ExtractedTarget {
    Primary(usize),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
}

#[derive(Debug, Clone)]
struct ExtractedEdge {
    term: SourceStructureTermId,
    ordinal: usize,
    role: SourceStructureEdgeRole,
    member: Option<SourceStructureMemberId>,
    target: ExtractedTarget,
}

/// Runs the bounded Task-254 transport only for its frozen real consumer.
pub(in crate::runner) fn source_structure_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    match source_structure_output(ast, module, shells, symbols) {
        None => None,
        Some(Ok(output))
            if output.typed_ast.source_structure().is_some()
                && output.typed_ast.source_structure() == output.resolved.source_structure()
                && output.typed_ast.source_term() == output.resolved.source_term() =>
        {
            Some(vec![PAYLOAD_EXTRACTION_GAP_KEY.to_owned()])
        }
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_structure_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Result<SourceStructureRouteOutput, String>> {
    source_structure_output_with_mutation(ast, module, shells, symbols, |_| {})
}

#[cfg(not(test))]
fn source_structure_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Result<SourceStructureRouteOutput, String>> {
    source_structure_output_with_mutation(ast, module, shells, symbols, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_structure_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Option<Result<SourceStructureRouteOutput, String>> {
    source_structure_output_with_mutation_impl(ast, module, shells, symbols, mutate)
}

#[cfg(not(test))]
fn source_structure_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Option<Result<SourceStructureRouteOutput, String>> {
    source_structure_output_with_mutation_impl(ast, module, shells, symbols, mutate)
}

fn source_structure_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Option<Result<SourceStructureRouteOutput, String>> {
    let roots = exact_real_roots(ast)?;
    let binding_env = match source_structure_binding_env(ast, module.clone(), shells, symbols) {
        Ok(binding_env) => binding_env,
        Err(error) => return Some(Err(error)),
    };
    let extracted = match extract_structure(
        ast,
        &module,
        symbols,
        BindingContextId::new(1),
        &roots,
        None,
        &BTreeSet::new(),
    ) {
        Ok(extracted) => extracted,
        Err(error) => return Some(Err(error)),
    };
    Some(build_output(
        ast,
        module,
        symbols,
        binding_env,
        extracted,
        None,
        mutate,
    ))
}

#[cfg(test)]
pub(in crate::runner) fn synthetic_source_structure_output(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    symbols: &SymbolEnv,
    roots: &[usize],
    dependencies: Option<SyntheticSourceStructureDependencies>,
    degraded_terms: &BTreeSet<usize>,
) -> Result<SourceStructureRouteOutput, String> {
    synthetic_source_structure_output_with_mutation(
        ast,
        module,
        binding_env,
        symbols,
        roots,
        dependencies,
        degraded_terms,
        |_| {},
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)] // Rationale: keep every synthetic dependency and corruption seam explicit.
pub(in crate::runner) fn synthetic_source_structure_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    symbols: &SymbolEnv,
    roots: &[usize],
    dependencies: Option<SyntheticSourceStructureDependencies>,
    degraded_terms: &BTreeSet<usize>,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Result<SourceStructureRouteOutput, String> {
    let application = dependencies
        .as_ref()
        .and_then(|dependencies| dependencies.application.as_ref());
    let extracted = extract_structure(
        ast,
        &module,
        symbols,
        BindingContextId::new(0),
        roots,
        application,
        degraded_terms,
    )?;
    build_output(
        ast,
        module,
        symbols,
        binding_env,
        extracted,
        dependencies,
        mutate,
    )
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)] // Rationale: keep extraction and producer-validation provenance authorities explicit.
pub(in crate::runner) fn synthetic_source_structure_output_with_validation_symbols(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    extraction_symbols: &SymbolEnv,
    validation_symbols: &SymbolEnv,
    roots: &[usize],
    dependencies: Option<SyntheticSourceStructureDependencies>,
    degraded_terms: &BTreeSet<usize>,
) -> Result<SourceStructureRouteOutput, String> {
    let application = dependencies
        .as_ref()
        .and_then(|dependencies| dependencies.application.as_ref());
    let extracted = extract_structure(
        ast,
        &module,
        extraction_symbols,
        BindingContextId::new(0),
        roots,
        application,
        degraded_terms,
    )?;
    build_output(
        ast,
        module,
        validation_symbols,
        binding_env,
        extracted,
        dependencies,
        |_| {},
    )
}

fn exact_real_roots(ast: &SurfaceAst) -> Option<Vec<usize>> {
    let items = exact_compilation_item_list(ast)?;
    let item_ids = structural_child_ids(ast, items);
    let [reserve_id, definition_id] = item_ids.as_slice() else {
        return None;
    };
    let reserve = ast.node(*reserve_id)?;
    let definition = ast.node(*definition_id)?;
    if !matches!(reserve.kind, SurfaceNodeKind::ReserveItem)
        || !matches!(definition.kind, SurfaceNodeKind::DefinitionBlockItem)
        || subtree_has_recovery(ast, reserve)
        || subtree_has_recovery(ast, definition)
        || subtree_tokens(ast, reserve) != ["reserve", "seed", "for", "set", ";"]
    {
        return None;
    }
    let expected_definition = [
        "definition",
        "let",
        "seed",
        "be",
        "set",
        ";",
        "struct",
        "Task254Pair",
        "where",
        "field",
        "carrier",
        "->",
        "set",
        ";",
        "field",
        "marker",
        "->",
        "set",
        ";",
        "end",
        ";",
        "func",
        "Task254ConstructorDef",
        ":",
        "task254_constructor",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "Task254Pair",
        "(",
        "carrier",
        ":",
        "1",
        ",",
        "marker",
        ":",
        "2",
        ")",
        ";",
        "func",
        "Task254SelectorDef",
        ":",
        "task254_selector",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "Task254Pair",
        "(",
        "carrier",
        ":",
        "3",
        ",",
        "marker",
        ":",
        "4",
        ")",
        ".",
        "carrier",
        ";",
        "func",
        "Task254UpdateDef",
        ":",
        "task254_update",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "Task254Pair",
        "(",
        "carrier",
        ":",
        "5",
        ",",
        "marker",
        ":",
        "6",
        ")",
        "with",
        "(",
        "carrier",
        ":=",
        "7",
        ",",
        "marker",
        ":=",
        "8",
        ")",
        ";",
        "end",
        ";",
    ];
    if subtree_tokens(ast, definition) != expected_definition {
        return None;
    }

    let parents = parent_indexes(ast);
    let roots = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| is_structure_kind(&node.kind))
        .filter(|(index, _)| !has_structure_ancestor(ast, &parents, *index))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if roots.len() == 3
        && ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::StructureConstructor))
            .count()
            == 3
        && ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::SelectorAccess))
            .count()
            == 1
        && ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::StructureUpdate))
            .count()
            == 1
    {
        Some(roots)
    } else {
        None
    }
}

fn extract_structure(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    context: BindingContextId,
    roots: &[usize],
    application: Option<&SourceFunctorApplicationHandoff>,
    degraded_terms: &BTreeSet<usize>,
) -> Result<ExtractedStructure, String> {
    if symbols.module_id() != module {
        return Err("source-structure resolver module identity mismatch".to_owned());
    }
    let mut extracted = ExtractedStructure {
        context,
        terms: Vec::new(),
        wrappers: Vec::new(),
        roots: Vec::new(),
        members: Vec::new(),
        field_updates: Vec::new(),
        edges: Vec::new(),
        primary_roots: Vec::new(),
    };
    let mut ordered_roots = roots.to_vec();
    ordered_roots.sort_by_key(|root| {
        ast.nodes()
            .get(*root)
            .map_or((usize::MAX, usize::MAX), |node| {
                (node.range.start, usize::MAX - node.range.end)
            })
    });
    ordered_roots.dedup();
    for root in ordered_roots {
        if structure_root_is_excluded(ast, root, application) {
            continue;
        }
        collect_target(
            ast,
            module,
            symbols,
            root,
            application,
            degraded_terms,
            &mut extracted,
        )?
        .structure()
        .ok_or_else(|| "synthetic source-structure root is not a structure term".to_owned())?;
    }
    normalize_extracted_tables(&mut extracted);
    Ok(extracted)
}

fn structure_root_is_excluded(
    ast: &SurfaceAst,
    root: usize,
    applications: Option<&SourceFunctorApplicationHandoff>,
) -> bool {
    fn excluded_descendant_kind(kind: &SurfaceNodeKind) -> bool {
        matches!(
            kind,
            SurfaceNodeKind::TypeArguments
                | SurfaceNodeKind::SetEnumeration
                | SurfaceNodeKind::SetComprehension
                | SurfaceNodeKind::ChoiceTerm
                | SurfaceNodeKind::QuaExpression
                | SurfaceNodeKind::TemplateParameter
                | SurfaceNodeKind::TemplateLoci
                | SurfaceNodeKind::TemplateLocus
                | SurfaceNodeKind::TemplateArguments
                | SurfaceNodeKind::TemplateArgument
        )
    }
    fn subtree_is_excluded(ast: &SurfaceAst, node: usize) -> bool {
        ast.nodes()[node].children.iter().any(|child| {
            excluded_descendant_kind(&ast.nodes()[child.index()].kind)
                || subtree_is_excluded(ast, child.index())
        })
    }

    if subtree_is_excluded(ast, root) {
        return true;
    }
    let parents = parent_indexes(ast);
    let mut cursor = parents[root];
    while let Some(parent) = cursor {
        let node = &ast.nodes()[parent];
        if matches!(
            node.kind,
            SurfaceNodeKind::TemplateParameter
                | SurfaceNodeKind::TemplateLoci
                | SurfaceNodeKind::TemplateLocus
                | SurfaceNodeKind::TemplateArguments
                | SurfaceNodeKind::TemplateArgument
        ) {
            return true;
        }
        if matches!(
            node.kind,
            SurfaceNodeKind::ApplicationTerm
                | SurfaceNodeKind::PrefixExpression(_)
                | SurfaceNodeKind::InfixExpression(_)
                | SurfaceNodeKind::PostfixExpression(_)
        ) && applications.is_some_and(|applications| {
            applications
                .applications()
                .iter()
                .any(|(_, application)| application.source_range() == node.range)
        }) {
            return true;
        }
        cursor = parents[parent];
    }
    false
}

fn normalize_extracted_tables(extracted: &mut ExtractedStructure) {
    extracted
        .wrappers
        .sort_by_key(|wrapper| (wrapper.term.index(), wrapper.ordinal));
    extracted.roots.sort_by_key(|root| root.term.index());

    let mut member_order = (0..extracted.members.len()).collect::<Vec<_>>();
    member_order.sort_by_key(|index| {
        let member = &extracted.members[*index];
        (member.term.index(), member.ordinal)
    });
    let member_remap = member_order
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, SourceStructureMemberId::new(new)))
        .collect::<BTreeMap<_, _>>();
    let old_members = std::mem::take(&mut extracted.members);
    extracted.members = member_order
        .into_iter()
        .map(|old| {
            let mut member = old_members[old].clone();
            member.parent = member.parent.map(|parent| member_remap[&parent.index()]);
            member
        })
        .collect();
    for update in &mut extracted.field_updates {
        update.first_member = member_remap[&update.first_member.index()];
        update.final_member = member_remap[&update.final_member.index()];
    }
    for edge in &mut extracted.edges {
        edge.member = edge.member.map(|member| member_remap[&member.index()]);
    }
    extracted
        .field_updates
        .sort_by_key(|update| (update.term.index(), update.ordinal));
    extracted
        .edges
        .sort_by_key(|edge| (edge.term.index(), edge.ordinal));
}

impl ExtractedTarget {
    const fn structure(self) -> Option<SourceStructureTermId> {
        match self {
            Self::Structure(term) => Some(term),
            Self::Primary(_) | Self::Application(_) => None,
        }
    }
}

#[allow(clippy::too_many_arguments)] // Rationale: preserve the recursive extractor's explicit authority inputs.
fn collect_target(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    node_index: usize,
    application: Option<&SourceFunctorApplicationHandoff>,
    degraded_terms: &BTreeSet<usize>,
    extracted: &mut ExtractedStructure,
) -> Result<ExtractedTarget, String> {
    let original = ast
        .nodes()
        .get(node_index)
        .ok_or_else(|| "source-structure child site disappeared".to_owned())?;
    if let Some(application_id) = application_target(application, original.range) {
        return Ok(ExtractedTarget::Application(application_id));
    }

    let (core, wrappers) = peel_structure_shells(ast, node_index)?;
    let node = &ast.nodes()[core];
    if !is_structure_kind(&node.kind) {
        extracted.primary_roots.push(node_index);
        return Ok(ExtractedTarget::Primary(node_index));
    }
    if subtree_has_recovery(ast, node) && !degraded_terms.contains(&core) {
        return Err("source-structure normal term contains recovery".to_owned());
    }

    let term = SourceStructureTermId::new(extracted.terms.len());
    let kind = match node.kind {
        SurfaceNodeKind::StructureConstructor => SourceStructureTermKind::Constructor,
        SurfaceNodeKind::SelectorAccess => SourceStructureTermKind::SelectorAccess,
        SurfaceNodeKind::StructureUpdate => SourceStructureTermKind::FunctionalUpdate,
        _ => unreachable!("guarded above"),
    };
    let recovery = if degraded_terms.contains(&core) {
        SourceStructureRecovery::Degraded
    } else {
        SourceStructureRecovery::Normal
    };
    extracted.terms.push(ExtractedTerm {
        node: core,
        kind,
        recovery,
    });
    for (ordinal, wrapper) in wrappers.into_iter().enumerate() {
        extracted.wrappers.push(ExtractedWrapper {
            term,
            ordinal,
            node: wrapper,
            recovery,
        });
    }

    match kind {
        SourceStructureTermKind::Constructor => collect_constructor(
            ast,
            module,
            symbols,
            term,
            core,
            application,
            degraded_terms,
            extracted,
        )?,
        SourceStructureTermKind::SelectorAccess => collect_selector(
            ast,
            module,
            symbols,
            term,
            core,
            application,
            degraded_terms,
            extracted,
        )?,
        SourceStructureTermKind::FunctionalUpdate => collect_update(
            ast,
            module,
            symbols,
            term,
            core,
            application,
            degraded_terms,
            extracted,
        )?,
        _ => return Err("unsupported source-structure term kind".to_owned()),
    }
    Ok(ExtractedTarget::Structure(term))
}

#[allow(clippy::too_many_arguments)] // Rationale: preserve constructor extraction and dependency ownership inputs.
fn collect_constructor(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    term: SourceStructureTermId,
    node_index: usize,
    application: Option<&SourceFunctorApplicationHandoff>,
    degraded_terms: &BTreeSet<usize>,
    extracted: &mut ExtractedStructure,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    let children = structural_child_ids(ast, node);
    let Some(root_id) = children.first().copied() else {
        return Err("source-structure constructor lost its root".to_owned());
    };
    let root_node = ast
        .node(root_id)
        .ok_or_else(|| "source-structure constructor root disappeared".to_owned())?;
    if !matches!(root_node.kind, SurfaceNodeKind::QualifiedSymbol)
        || children.iter().any(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::TypeArguments))
        })
    {
        return Err("source-structure constructor root/type-argument shape drift".to_owned());
    }
    let spelling = qualified_symbol_spelling(ast, root_node)
        .map_err(|()| "source-structure constructor root spelling drift".to_owned())?;
    let namespace = NamespacePath::new(module.path().as_str());
    let candidates = symbols
        .symbols()
        .visible_candidates(&namespace, &spelling)
        .into_iter()
        .filter(|entry| entry.kind() == SymbolKind::Structure)
        .collect::<Vec<_>>();
    let [root] = candidates.as_slice() else {
        return Err("source-structure constructor needs one structure root".to_owned());
    };
    extracted.roots.push(ExtractedRoot {
        term,
        symbol: root.symbol().clone(),
        contribution: root.contribution(),
    });

    let fields = children
        .into_iter()
        .filter(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::FieldArgument))
        })
        .collect::<Vec<_>>();
    for (ordinal, field_id) in fields.into_iter().enumerate() {
        let field = ast.node(field_id).expect("field id came from the AST");
        let label = one_identifier_token(ast, field, &[":"])?;
        let member = SourceStructureMemberId::new(extracted.members.len());
        extracted.members.push(ExtractedMember {
            term,
            ordinal,
            node: label.index(),
            role: SourceStructureMemberRole::ConstructorAssignment,
            parent: None,
        });
        let value = one_structural_child(ast, field)?;
        let target = collect_target(
            ast,
            module,
            symbols,
            value.index(),
            application,
            degraded_terms,
            extracted,
        )?;
        push_edge(
            extracted,
            term,
            SourceStructureEdgeRole::ConstructorValue,
            Some(member),
            target,
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Rationale: preserve selector extraction and dependency ownership inputs.
fn collect_selector(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    term: SourceStructureTermId,
    node_index: usize,
    application: Option<&SourceFunctorApplicationHandoff>,
    degraded_terms: &BTreeSet<usize>,
    extracted: &mut ExtractedStructure,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    let label = one_identifier_token(ast, node, &[".", "(", ",", ")"])?;
    extracted.members.push(ExtractedMember {
        term,
        ordinal: 0,
        node: label.index(),
        role: SourceStructureMemberRole::Selector,
        parent: None,
    });
    let children = structural_child_ids(ast, node);
    let Some((base, arguments)) = children.split_first() else {
        return Err("source-structure selector lost its base".to_owned());
    };
    let base = collect_target(
        ast,
        module,
        symbols,
        base.index(),
        application,
        degraded_terms,
        extracted,
    )?;
    push_edge(
        extracted,
        term,
        SourceStructureEdgeRole::SelectorBase,
        None,
        base,
    );
    for argument in arguments {
        let target = collect_target(
            ast,
            module,
            symbols,
            argument.index(),
            application,
            degraded_terms,
            extracted,
        )?;
        push_edge(
            extracted,
            term,
            SourceStructureEdgeRole::SelectorArgument,
            None,
            target,
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Rationale: preserve update extraction and dependency ownership inputs.
fn collect_update(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    term: SourceStructureTermId,
    node_index: usize,
    application: Option<&SourceFunctorApplicationHandoff>,
    degraded_terms: &BTreeSet<usize>,
    extracted: &mut ExtractedStructure,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    let children = structural_child_ids(ast, node);
    let Some((base, updates)) = children.split_first() else {
        return Err("source-structure update lost its base".to_owned());
    };
    let base = collect_target(
        ast,
        module,
        symbols,
        base.index(),
        application,
        degraded_terms,
        extracted,
    )?;
    push_edge(
        extracted,
        term,
        SourceStructureEdgeRole::UpdateBase,
        None,
        base,
    );
    for (ordinal, update_id) in updates.iter().copied().enumerate() {
        let update = ast
            .node(update_id)
            .ok_or_else(|| "source-structure field update disappeared".to_owned())?;
        if !matches!(update.kind, SurfaceNodeKind::FieldUpdate) {
            return Err("source-structure update has a non-FieldUpdate child".to_owned());
        }
        let path = identifier_tokens(ast, update, &[".", ":="])?;
        if path.is_empty() {
            return Err("source-structure field update has an empty path".to_owned());
        }
        let mut parent = None;
        let first = SourceStructureMemberId::new(extracted.members.len());
        for (member_ordinal, segment) in path.into_iter().enumerate() {
            let member = SourceStructureMemberId::new(extracted.members.len());
            extracted.members.push(ExtractedMember {
                term,
                ordinal: extracted
                    .members
                    .iter()
                    .filter(|member| member.term == term)
                    .count(),
                node: segment.index(),
                role: SourceStructureMemberRole::UpdatePathSegment,
                parent,
            });
            parent = Some(member);
            debug_assert_eq!(member.index(), first.index() + member_ordinal);
        }
        let final_member = parent.expect("nonempty path");
        extracted.field_updates.push(ExtractedFieldUpdate {
            term,
            ordinal,
            node: update_id.index(),
            first_member: first,
            final_member,
        });
        let value = one_structural_child(ast, update)?;
        let target = collect_target(
            ast,
            module,
            symbols,
            value.index(),
            application,
            degraded_terms,
            extracted,
        )?;
        push_edge(
            extracted,
            term,
            SourceStructureEdgeRole::UpdateValue,
            Some(final_member),
            target,
        );
    }
    Ok(())
}

fn push_edge(
    extracted: &mut ExtractedStructure,
    term: SourceStructureTermId,
    role: SourceStructureEdgeRole,
    member: Option<SourceStructureMemberId>,
    target: ExtractedTarget,
) {
    let ordinal = extracted
        .edges
        .iter()
        .filter(|edge| edge.term == term)
        .count();
    extracted.edges.push(ExtractedEdge {
        term,
        ordinal,
        role,
        member,
        target,
    });
}

fn peel_structure_shells(ast: &SurfaceAst, start: usize) -> Result<(usize, Vec<usize>), String> {
    let mut current = start;
    let mut wrappers = Vec::new();
    loop {
        let node = ast
            .nodes()
            .get(current)
            .ok_or_else(|| "source-structure shell disappeared".to_owned())?;
        match node.kind {
            SurfaceNodeKind::TermExpression => {
                if !direct_token_texts(ast, node).is_empty() {
                    break;
                }
                let children = structural_child_ids(ast, node);
                let [child] = children.as_slice() else {
                    break;
                };
                current = child.index();
            }
            SurfaceNodeKind::ParenthesizedTerm
                if direct_token_texts(ast, node).as_slice() == ["(", ")"]
                    && contains_structure(ast, node) =>
            {
                wrappers.push(current);
                let children = structural_child_ids(ast, node);
                let [child] = children.as_slice() else {
                    return Err("source-structure wrapper lost its child".to_owned());
                };
                current = child.index();
            }
            _ => break,
        }
    }
    Ok((current, wrappers))
}

fn contains_structure(ast: &SurfaceAst, node: &SurfaceNode) -> bool {
    node.children.iter().any(|child| {
        ast.node(*child)
            .is_some_and(|child| is_structure_kind(&child.kind) || contains_structure(ast, child))
    })
}

fn application_target(
    application: Option<&SourceFunctorApplicationHandoff>,
    range: SourceRange,
) -> Option<SourceFunctorApplicationId> {
    let application = application?;
    application.applications().iter().find_map(|(id, row)| {
        let outer = application
            .wrappers()
            .iter()
            .filter(|(_, wrapper)| wrapper.application() == id)
            .min_by_key(|(_, wrapper)| wrapper.ordinal())
            .map_or(row.source_range(), |(_, wrapper)| wrapper.source_range());
        (outer == range || row.source_range() == range).then_some(id)
    })
}

fn one_structural_child(ast: &SurfaceAst, node: &SurfaceNode) -> Result<SurfaceNodeId, String> {
    let children = structural_child_ids(ast, node);
    let [child] = children.as_slice() else {
        return Err("source-structure association requires one value child".to_owned());
    };
    Ok(*child)
}

fn one_identifier_token(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    punctuation: &[&str],
) -> Result<SurfaceNodeId, String> {
    let identifiers = identifier_tokens(ast, node, punctuation)?;
    let [identifier] = identifiers.as_slice() else {
        return Err("source-structure member requires one identifier".to_owned());
    };
    Ok(*identifier)
}

fn identifier_tokens(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    punctuation: &[&str],
) -> Result<Vec<SurfaceNodeId>, String> {
    let mut identifiers = Vec::new();
    for child in &node.children {
        let Some(child_node) = ast.node(*child) else {
            return Err("source-structure token site disappeared".to_owned());
        };
        let Some(text) = child_node.token_text() else {
            continue;
        };
        if punctuation.contains(&text) {
            continue;
        }
        if text.is_empty() {
            return Err("source-structure member spelling is empty".to_owned());
        }
        identifiers.push(*child);
    }
    Ok(identifiers)
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    binding_env: BindingEnv,
    extracted: ExtractedStructure,
    dependencies: Option<SyntheticSourceStructureDependencies>,
    mutate: impl FnOnce(&mut SourceStructureHandoffInput),
) -> Result<SourceStructureRouteOutput, String> {
    if binding_env.source_id() != ast.source_id || binding_env.module_id() != &module {
        return Err("source-structure binding environment identity mismatch".to_owned());
    }
    let mut kinds = BTreeMap::new();
    let mut recoveries = BTreeMap::new();
    for term in &extracted.terms {
        insert_kind(&mut kinds, term.node, term_kind_key(term.kind))?;
        if term.recovery == SourceStructureRecovery::Degraded {
            recoveries.insert(term.node, NodeRecoveryState::Degraded);
        }
    }
    for wrapper in &extracted.wrappers {
        insert_kind(
            &mut kinds,
            wrapper.node,
            "source.term.structure.parenthesized",
        )?;
        if wrapper.recovery == SourceStructureRecovery::Degraded {
            recoveries.insert(wrapper.node, NodeRecoveryState::Degraded);
        }
    }
    for member in &extracted.members {
        insert_kind(&mut kinds, member.node, member_kind_key(member.role))?;
        if extracted.terms[member.term.index()].recovery == SourceStructureRecovery::Degraded {
            recoveries.insert(member.node, NodeRecoveryState::Degraded);
        }
    }
    for update in &extracted.field_updates {
        insert_kind(
            &mut kinds,
            update.node,
            "source.term.structure.field-update",
        )?;
        if extracted.terms[update.term.index()].recovery == SourceStructureRecovery::Degraded {
            recoveries.insert(update.node, NodeRecoveryState::Degraded);
        }
    }

    let (arena, primary, application) = if let Some(dependencies) = dependencies {
        let arena = arena_with_overrides(ast, &dependencies.arena, &kinds, &recoveries)?;
        (arena, dependencies.primary, dependencies.application)
    } else {
        #[cfg(test)]
        let parts = synthetic_source_term_parts_for_roots(
            ast,
            module.clone(),
            &binding_env,
            extracted.primary_roots.iter().copied(),
            extracted.context,
            &kinds,
            &recoveries,
        )?;
        #[cfg(not(test))]
        let parts = source_term_parts_for_roots(
            ast,
            module.clone(),
            &binding_env,
            extracted.primary_roots.iter().copied(),
            extracted.context,
            &kinds,
        )?;
        (parts.arena, parts.handoff, None)
    };
    let primary_id = |root: usize| {
        let range = ast.nodes().get(root)?.range;
        primary
            .terms()
            .iter()
            .find(|(_, term)| term.parent().is_none() && term.source_range() == range)
            .map(|(id, _)| id)
    };
    let mut requests = Vec::new();
    for (term_ordinal, _) in extracted.terms.iter().enumerate() {
        let term = SourceStructureTermId::new(term_ordinal);
        let mut ordinal = 0;
        if extracted
            .terms
            .get(term_ordinal)
            .is_some_and(|row| row.kind == SourceStructureTermKind::Constructor)
        {
            requests.push(SourceStructureRequestInput {
                term,
                member: None,
                request_ordinal: ordinal,
                kind: SourceStructureRequestKind::ConstructorSignature,
            });
            ordinal += 1;
        }
        for (member_id, _) in extracted
            .members
            .iter()
            .enumerate()
            .filter(|(_, member)| member.term == term)
        {
            let member = SourceStructureMemberId::new(member_id);
            requests.push(SourceStructureRequestInput {
                term,
                member: Some(member),
                request_ordinal: ordinal,
                kind: SourceStructureRequestKind::MemberIdentity,
            });
            ordinal += 1;
            requests.push(SourceStructureRequestInput {
                term,
                member: Some(member),
                request_ordinal: ordinal,
                kind: SourceStructureRequestKind::InheritancePath,
            });
            ordinal += 1;
        }
        requests.push(SourceStructureRequestInput {
            term,
            member: None,
            request_ordinal: ordinal,
            kind: SourceStructureRequestKind::ResultType,
        });
    }

    let mut input = SourceStructureHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms: extracted
            .terms
            .iter()
            .enumerate()
            .map(|(source_ordinal, term)| {
                let node = &ast.nodes()[term.node];
                SourceStructureTermInput {
                    site: TypedSiteRef::Node(TypedNodeId::new(term.node)),
                    source_range: node.range,
                    source_ordinal,
                    context: extracted.context,
                    recovery: term.recovery,
                    spelling: subtree_tokens(ast, node).join(" "),
                    kind: term.kind,
                }
            })
            .collect(),
        wrappers: extracted
            .wrappers
            .iter()
            .map(|wrapper| {
                let node = &ast.nodes()[wrapper.node];
                SourceStructureWrapperInput {
                    term: wrapper.term,
                    ordinal: wrapper.ordinal,
                    site: TypedSiteRef::Node(TypedNodeId::new(wrapper.node)),
                    source_range: node.range,
                    context: extracted.context,
                    spelling: subtree_tokens(ast, node).join(" "),
                    recovery: wrapper.recovery,
                }
            })
            .collect(),
        roots: extracted
            .roots
            .iter()
            .map(|root| SourceStructureRootInput {
                term: root.term,
                symbol: root.symbol.clone(),
                contribution: root.contribution,
            })
            .collect(),
        members: extracted
            .members
            .iter()
            .map(|member| {
                let node = &ast.nodes()[member.node];
                SourceStructureMemberInput {
                    term: member.term,
                    ordinal: member.ordinal,
                    site: TypedSiteRef::Node(TypedNodeId::new(member.node)),
                    source_range: node.range,
                    spelling: node.token_text().unwrap_or_default().to_owned(),
                    role: member.role,
                    parent: member.parent,
                }
            })
            .collect(),
        field_updates: extracted
            .field_updates
            .iter()
            .map(|update| {
                let node = &ast.nodes()[update.node];
                SourceFieldUpdateInput {
                    term: update.term,
                    ordinal: update.ordinal,
                    site: TypedSiteRef::Node(TypedNodeId::new(update.node)),
                    source_range: node.range,
                    spelling: subtree_tokens(ast, node).join(" "),
                    first_member: update.first_member,
                    final_member: update.final_member,
                }
            })
            .collect(),
        edges: extracted
            .edges
            .iter()
            .map(|edge| {
                let target = match edge.target {
                    ExtractedTarget::Primary(root) => {
                        SourceStructureTarget::Primary(primary_id(root).ok_or_else(|| {
                            "source-structure primary child is not a Task-252 root".to_owned()
                        })?)
                    }
                    ExtractedTarget::Application(application) => {
                        SourceStructureTarget::Application(application)
                    }
                    ExtractedTarget::Structure(structure) => {
                        SourceStructureTarget::Structure(structure)
                    }
                };
                Ok(SourceStructureEdgeInput {
                    term: edge.term,
                    ordinal: edge.ordinal,
                    role: edge.role,
                    member: edge.member,
                    target,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        requests,
    };
    mutate(&mut input);
    let handoff = SourceStructureProducer::build(
        input,
        symbols,
        &binding_env,
        &primary,
        application.as_ref(),
        &arena,
    )
    .map_err(|error| error.to_string())?;
    let mut typed_ast = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes: arena,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())?
    .with_source_term(primary)
    .map_err(|error| error.to_string())?;
    if let Some(application) = application {
        typed_ast = typed_ast
            .with_source_application(application)
            .map_err(|error| error.to_string())?;
    }
    let typed_ast = typed_ast
        .with_source_structure(handoff)
        .map_err(|error| error.to_string())?;
    let node_hints = typed_ast
        .nodes()
        .iter()
        .map(|(typed_node, _)| ResolvedNodeKindHint {
            typed_node,
            kind: ResolvedNodeKindHintKind::SourcePreserved {
                role: SourceNodeRole::new("source.term.surface"),
            },
        })
        .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_structure().is_none()
        || resolved.source_structure() != typed_ast.source_structure()
        || resolved.source_application() != typed_ast.source_application()
        || resolved.source_term() != typed_ast.source_term()
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !resolved.expr_metadata().is_empty()
        || !resolved.cluster_facts().is_empty()
        || !resolved.diagnostics().is_empty()
    {
        return Err("source-structure immutable final handoff mismatch".to_owned());
    }
    Ok(SourceStructureRouteOutput {
        typed_ast,
        resolved,
        #[cfg(test)]
        binding_env,
    })
}

fn arena_with_overrides(
    ast: &SurfaceAst,
    source: &TypedArena,
    kinds: &BTreeMap<usize, &'static str>,
    recoveries: &BTreeMap<usize, NodeRecoveryState>,
) -> Result<TypedArena, String> {
    if source.len() != ast.nodes().len() {
        return Err("source-structure dependency arena is not surface-indexed".to_owned());
    }
    let mut nodes = source
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for (index, kind) in kinds {
        let node = nodes
            .get_mut(*index)
            .ok_or_else(|| "source-structure arena override site disappeared".to_owned())?;
        if node.kind.as_str() != "source.surface.unowned" {
            return Err("source-structure arena override site is already owned".to_owned());
        }
        node.kind = (*kind).into();
    }
    for (index, recovery) in recoveries {
        nodes
            .get_mut(*index)
            .ok_or_else(|| "source-structure recovery override site disappeared".to_owned())?
            .recovery = *recovery;
    }
    TypedArena::try_new(source.root(), nodes).map_err(|error| error.to_string())
}

fn insert_kind(
    kinds: &mut BTreeMap<usize, &'static str>,
    node: usize,
    kind: &'static str,
) -> Result<(), String> {
    if kinds.insert(node, kind).is_some() {
        return Err("source-structure arena site is multiply owned".to_owned());
    }
    Ok(())
}

fn term_kind_key(kind: SourceStructureTermKind) -> &'static str {
    match kind {
        SourceStructureTermKind::Constructor => "source.term.structure.constructor",
        SourceStructureTermKind::SelectorAccess => "source.term.structure.selector",
        SourceStructureTermKind::FunctionalUpdate => "source.term.structure.update",
        _ => "source.term.structure.unsupported",
    }
}

fn member_kind_key(role: SourceStructureMemberRole) -> &'static str {
    match role {
        SourceStructureMemberRole::ConstructorAssignment => {
            "source.term.structure.member.constructor-assignment"
        }
        SourceStructureMemberRole::Selector => "source.term.structure.member.selector",
        SourceStructureMemberRole::UpdatePathSegment => {
            "source.term.structure.member.update-path-segment"
        }
        _ => "source.term.structure.member.unsupported",
    }
}

fn source_structure_binding_env(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Result<BindingEnv, String> {
    let items = exact_compilation_item_list(ast)
        .ok_or_else(|| "Task254 compilation item list disappeared".to_owned())?;
    let item_ids = structural_child_ids(ast, items);
    let [reserve_id, definition_id] = item_ids.as_slice() else {
        return Err("Task254 requires one reserve and one definition block".to_owned());
    };
    let reserve = ast
        .node(*reserve_id)
        .ok_or_else(|| "Task254 reserve disappeared".to_owned())?;
    let definition = ast
        .node(*definition_id)
        .ok_or_else(|| "Task254 definition block disappeared".to_owned())?;
    let reserve_shell = unique_shell(shells, *reserve_id, DeclarationShellKind::Reserve)?;
    let definition_shell = unique_shell(
        shells,
        *definition_id,
        DeclarationShellKind::DefinitionBlock,
    )?;
    let reserve_payload =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)
            .map_err(|()| "Task254 reserve extraction failed".to_owned())?;
    let [reserve_binding] = reserve_payload.bridge.bindings() else {
        return Err("Task254 requires one reserve binding".to_owned());
    };
    let reserve_children = structural_child_ids(ast, reserve);
    let [reserve_segment_id] = reserve_children.as_slice() else {
        return Err("Task254 reserve requires one segment".to_owned());
    };
    let parameter_id = structural_child_ids(ast, definition)
        .into_iter()
        .find(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::DefinitionParameter))
        })
        .ok_or_else(|| "Task254 definition parameter disappeared".to_owned())?;
    let parameter = ast
        .node(parameter_id)
        .ok_or_else(|| "Task254 definition parameter disappeared".to_owned())?;
    let parameter_children = structural_child_ids(ast, parameter);
    let [segment_id] = parameter_children.as_slice() else {
        return Err("Task254 definition parameter requires one segment".to_owned());
    };
    let segment = ast
        .node(*segment_id)
        .ok_or_else(|| "Task254 definition parameter segment disappeared".to_owned())?;
    let segment_children = structural_child_ids(ast, segment);
    let [type_id] = segment_children.as_slice() else {
        return Err("Task254 definition parameter requires one written type".to_owned());
    };
    let written_type = ast
        .node(*type_id)
        .ok_or_else(|| "Task254 definition parameter type disappeared".to_owned())?;
    if !matches!(segment.kind, SurfaceNodeKind::QualifiedVariableSegment)
        || direct_token_texts(ast, segment).as_slice() != ["seed", "be"]
        || !matches!(written_type.kind, SurfaceNodeKind::TypeExpression)
        || subtree_tokens(ast, written_type) != ["set"]
    {
        return Err("Task254 definition parameter shape drift".to_owned());
    }
    let declaration_range = unique_direct_token_range(ast, segment, "seed")?;
    let root_id = ast
        .root()
        .ok_or_else(|| "Task254 root disappeared".to_owned())?;
    let local_scope = LocalTermScope::new(vec![definition_shell.id().index() as u32]);
    let build = SourceBindingContextProducer::build(SourceBindingContextInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        module_site: surface_site(root_id),
        items: vec![
            SourceItemInput {
                shell: reserve_shell.id(),
                shell_ordinal: reserve_shell.ordinal(),
                role: SourceItemRole::Reserve,
                module_id: module.clone(),
                source_range: reserve.range,
                parent: None,
                visibility: SourceItemVisibility::Unspecified,
                site: surface_site(*reserve_id),
                local_scope: None,
                recovery: SourceItemRecovery::Normal,
            },
            SourceItemInput {
                shell: definition_shell.id(),
                shell_ordinal: definition_shell.ordinal(),
                role: SourceItemRole::DefinitionBlock,
                module_id: module.clone(),
                source_range: definition.range,
                parent: None,
                visibility: SourceItemVisibility::Unspecified,
                site: surface_site(*definition_id),
                local_scope: Some(local_scope.clone()),
                recovery: SourceItemRecovery::Normal,
            },
        ],
        bindings: vec![
            SourceBindingSiteInput {
                shell: reserve_shell.id(),
                context_owner: SourceBindingContextOwner::Module,
                source_ordinal: 0,
                spelling: "seed".to_owned(),
                declaration_range: reserve_binding.binding_range,
                written_type_range: reserve_binding.type_range,
                site: surface_site(*reserve_segment_id),
                role: SourceBindingSiteRole::ReserveDefault,
                recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
            },
            SourceBindingSiteInput {
                shell: definition_shell.id(),
                context_owner: SourceBindingContextOwner::Shell(definition_shell.id()),
                source_ordinal: 1,
                spelling: "seed".to_owned(),
                declaration_range,
                written_type_range: written_type.range,
                site: surface_site(parameter_id),
                role: SourceBindingSiteRole::DefinitionParameter {
                    local: LocalTermBinding::new("seed", local_scope, declaration_range, 1),
                },
                recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
            },
        ],
    })
    .map_err(|error| error.to_string())?;
    match build {
        SourceBindingContextBuild::Complete(projection) => {
            let handoff = projection.into_handoff();
            if handoff.binding_env().contexts().len() != 2
                || handoff.binding_env().bindings().len() != 2
            {
                return Err("Task254 binding projection cardinality mismatch".to_owned());
            }
            Ok(handoff.binding_env().clone())
        }
        SourceBindingContextBuild::Incomplete(_) => {
            Err("Task254 binding projection remained incomplete".to_owned())
        }
        _ => Err("Task254 binding projection returned an unsupported state".to_owned()),
    }
}

fn unique_shell(
    shells: &DeclarationShellSet,
    node: SurfaceNodeId,
    kind: DeclarationShellKind,
) -> Result<&DeclarationShell, String> {
    let matches = shells
        .declarations()
        .iter()
        .filter(|shell| shell.node_id() == node && shell.kind() == kind && !shell.recovered())
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [shell] => Ok(*shell),
        _ => Err(format!("Task254 requires one normal {kind:?} shell")),
    }
}

fn unique_direct_token_range(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    spelling: &str,
) -> Result<SourceRange, String> {
    let ranges = node
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter(|child| child.token_text() == Some(spelling))
        .map(|child| child.range)
        .collect::<Vec<_>>();
    let [range] = ranges.as_slice() else {
        return Err(format!("Task254 requires one direct `{spelling}` token"));
    };
    Ok(*range)
}

fn is_structure_kind(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::StructureConstructor
            | SurfaceNodeKind::SelectorAccess
            | SurfaceNodeKind::StructureUpdate
    )
}

fn parent_indexes(ast: &SurfaceAst) -> Vec<Option<usize>> {
    let mut parents = vec![None; ast.nodes().len()];
    for (parent, node) in ast.nodes().iter().enumerate() {
        for child in &node.children {
            parents[child.index()] = Some(parent);
        }
    }
    parents
}

fn has_structure_ancestor(ast: &SurfaceAst, parents: &[Option<usize>], node: usize) -> bool {
    let mut cursor = parents[node];
    while let Some(parent) = cursor {
        if is_structure_kind(&ast.nodes()[parent].kind) {
            return true;
        }
        cursor = parents[parent];
    }
    false
}

fn subtree_tokens<'a>(ast: &'a SurfaceAst, node: &'a SurfaceNode) -> Vec<&'a str> {
    let mut tokens = Vec::new();
    collect_subtree_tokens(ast, node, &mut tokens);
    tokens
}

fn collect_subtree_tokens<'a>(
    ast: &'a SurfaceAst,
    node: &'a SurfaceNode,
    tokens: &mut Vec<&'a str>,
) {
    if let Some(token) = node.token_text() {
        tokens.push(token);
        return;
    }
    for child in &node.children {
        if let Some(child) = ast.node(*child) {
            collect_subtree_tokens(ast, child, tokens);
        }
    }
}
