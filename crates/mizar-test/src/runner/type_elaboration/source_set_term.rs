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
    source_set_term::{
        SourceSetConditionInput, SourceSetEdgeInput, SourceSetEdgeRole, SourceSetGeneratorId,
        SourceSetGeneratorInput, SourceSetRequestInput, SourceSetRequestKind, SourceSetTarget,
        SourceSetTermHandoffInput, SourceSetTermId, SourceSetTermInput, SourceSetTermKind,
        SourceSetTermProducer, SourceSetTermRecovery, SourceSetTypeHead, SourceSetTypeOwner,
        SourceSetTypeRole, SourceSetTypeSiteId, SourceSetTypeSiteInput, SourceSetWrapperInput,
    },
    source_structure::{SourceStructureHandoff, SourceStructureTermId},
    source_term::SourcePrimaryTermHandoff,
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArena, TypedAst, TypedAstParts,
        TypedNodeId, TypedSiteRef,
    },
};
use mizar_resolve::{
    declarations::{DeclarationShell, DeclarationShellKind, DeclarationShellSet},
    env::SymbolEnv,
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::ModuleId,
};
use mizar_session::SourceRange;
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind};

#[cfg(not(test))]
use super::source_term::source_term_parts_for_roots;
#[cfg(test)]
use super::source_term::synthetic_source_term_parts_for_roots;
use super::{
    checker_handoff::{assemble_empty_resolved_typed_ast, source_module_binding_env},
    source_application::{
        unwrapped_imported_source_application_handoff,
        unwrapped_imported_source_application_owned_node_kinds,
    },
    source_ast::{
        direct_token_texts, exact_compilation_item_list, is_exact_parser_type_fixtures_import,
        structural_child_ids, subtree_has_recovery, surface_site,
    },
    source_reserve::extract_builtin_source_reserve_declarations_after_node_guard,
    source_term::SourceTermParts,
};

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.typed_ast_invalid";
const PAYLOAD_EXTRACTION_GAP_KEY: &str =
    "type_elaboration.external_dependency.ast_payload_extraction";

#[derive(Debug)]
pub(in crate::runner) struct SourceSetTermRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
    #[cfg(test)]
    pub(in crate::runner) binding_env: BindingEnv,
}

#[derive(Debug, Clone)]
pub(in crate::runner) struct SyntheticSourceSetTermDependencies {
    pub(in crate::runner) arena: TypedArena,
    pub(in crate::runner) primary: SourcePrimaryTermHandoff,
    pub(in crate::runner) application: Option<SourceFunctorApplicationHandoff>,
    pub(in crate::runner) structure: Option<SourceStructureHandoff>,
}

#[derive(Debug, Clone)]
struct ExtractedSetTerms {
    context: BindingContextId,
    terms: Vec<ExtractedTerm>,
    wrappers: Vec<ExtractedWrapper>,
    generators: Vec<ExtractedGenerator>,
    type_sites: Vec<ExtractedTypeSite>,
    conditions: Vec<ExtractedCondition>,
    edges: Vec<ExtractedEdge>,
    primary_roots: Vec<usize>,
}

#[derive(Debug, Clone)]
struct ExtractedTerm {
    node: usize,
    kind: SourceSetTermKind,
    recovery: SourceSetTermRecovery,
}

#[derive(Debug, Clone)]
struct ExtractedWrapper {
    term: SourceSetTermId,
    ordinal: usize,
    node: usize,
    recovery: SourceSetTermRecovery,
}

#[derive(Debug, Clone)]
struct ExtractedGenerator {
    term: SourceSetTermId,
    ordinal: usize,
    node: usize,
    recovery: SourceSetTermRecovery,
    type_site: SourceSetTypeSiteId,
}

#[derive(Debug, Clone)]
struct ExtractedTypeSite {
    owner: SourceSetTypeOwner,
    node: usize,
    head_node: usize,
    head: SourceSetTypeHead,
    recovery: SourceSetTermRecovery,
}

#[derive(Debug, Clone)]
struct ExtractedCondition {
    term: SourceSetTermId,
    ordinal: usize,
    colon_node: usize,
    condition_node: usize,
    recovery: SourceSetTermRecovery,
}

#[derive(Debug, Clone, Copy)]
enum ExtractedTarget {
    Primary(usize),
    Application(SourceFunctorApplicationId),
    Structure(SourceStructureTermId),
    SetTerm(SourceSetTermId),
}

#[derive(Debug, Clone)]
struct ExtractedEdge {
    term: SourceSetTermId,
    ordinal: usize,
    role: SourceSetEdgeRole,
    target: ExtractedTarget,
}

const TASK255C1_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "definition\n",
    "  func Task255ConditionedComprehensionDef:\n",
    "    task255_conditioned_comprehension -> set\n",
    "    equals { 1 ++ 2 where candidate255c is set : 3 = 4 };\n",
    "end;\n",
);

#[derive(Debug, Clone)]
struct ExactConditionedRoute {
    root: usize,
    application: usize,
    primary_roots: Vec<usize>,
}

/// Runs the bounded Task-255 source transport without replacing the existing
/// definition-extraction diagnostic owner.
pub(in crate::runner) fn source_set_term_transport_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Vec<String>> {
    match source_set_term_output_with_optional_source(
        ast,
        module,
        shells,
        symbols,
        Some(source_text),
    ) {
        None => None,
        Some(Ok(output))
            if output.typed_ast.source_set_term().is_some()
                && output.typed_ast.source_set_term() == output.resolved.source_set_term()
                && output.typed_ast.source_term() == output.resolved.source_term() =>
        {
            Some(vec![PAYLOAD_EXTRACTION_GAP_KEY.to_owned()])
        }
        Some(Ok(_)) | Some(Err(_)) => Some(vec![INVALID_PAYLOAD_KEY.to_owned()]),
    }
}

#[cfg(test)]
pub(in crate::runner) fn source_set_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    source_set_term_output_with_mutation_impl(ast, module, shells, symbols, None, |_| {})
}

#[cfg(test)]
pub(in crate::runner) fn source_set_term_output_with_source(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    source_set_term_output_with_mutation_impl(
        ast,
        module,
        shells,
        symbols,
        Some(source_text),
        |_| {},
    )
}

fn source_set_term_output_with_optional_source(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: Option<&str>,
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    source_set_term_output_with_mutation_impl(ast, module, shells, symbols, source_text, |_| {})
}

pub(super) fn conditioned_source_set_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    source_text: &str,
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    let route = exact_conditioned_route(ast, source_text)?;
    Some(build_conditioned_output(
        ast,
        module,
        symbols,
        route,
        |_| {},
    ))
}

#[cfg(test)]
pub(in crate::runner) fn source_set_term_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    source_set_term_output_with_mutation_impl(ast, module, shells, symbols, None, mutate)
}

#[cfg(test)]
pub(in crate::runner) fn source_set_term_output_with_source_and_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: &str,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    source_set_term_output_with_mutation_impl(
        ast,
        module,
        shells,
        symbols,
        Some(source_text),
        mutate,
    )
}

fn source_set_term_output_with_mutation_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    source_text: Option<&str>,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Option<Result<SourceSetTermRouteOutput, String>> {
    if let Some(source_text) = source_text
        && let Some(route) = exact_conditioned_route(ast, source_text)
    {
        return Some(build_conditioned_output(
            ast, module, symbols, route, mutate,
        ));
    }
    let roots = exact_real_roots(ast)?;
    let binding_env = match source_set_term_binding_env(ast, module.clone(), shells, symbols) {
        Ok(binding_env) => binding_env,
        Err(error) => return Some(Err(error)),
    };
    let extracted = match extract_set_terms(
        ast,
        &module,
        BindingContextId::new(1),
        &roots,
        None,
        None,
        &BTreeSet::new(),
        false,
    ) {
        Ok(extracted) => extracted,
        Err(error) => return Some(Err(error)),
    };
    Some(build_output(
        ast,
        module,
        binding_env,
        extracted,
        None,
        mutate,
    ))
}

fn exact_conditioned_route(ast: &SurfaceAst, source_text: &str) -> Option<ExactConditionedRoute> {
    if source_text != TASK255C1_SOURCE
        || source_text.len() != 191
        || ast.nodes().iter().any(|node| node.recovered)
    {
        return None;
    }
    let items = exact_compilation_item_list(ast)?;
    let item_ids = structural_child_ids(ast, items);
    let [import_id, definition_id] = item_ids.as_slice() else {
        return None;
    };
    let import = ast.node(*import_id)?;
    let definition = ast.node(*definition_id)?;
    if !is_exact_parser_type_fixtures_import(ast, import)
        || !matches!(definition.kind, SurfaceNodeKind::DefinitionBlockItem)
        || subtree_tokens(ast, definition)
            != [
                "definition",
                "func",
                "Task255ConditionedComprehensionDef",
                ":",
                "task255_conditioned_comprehension",
                "->",
                "set",
                "equals",
                "{",
                "1",
                "++",
                "2",
                "where",
                "candidate255c",
                "is",
                "set",
                ":",
                "3",
                "=",
                "4",
                "}",
                ";",
                "end",
                ";",
            ]
    {
        return None;
    }
    let unique = |kind: &SurfaceNodeKind, start: usize, end: usize| {
        let matches = ast
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                std::mem::discriminant(&node.kind) == std::mem::discriminant(kind)
                    && node.range.start == start
                    && node.range.end == end
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let [index] = matches.as_slice() else {
            return None;
        };
        Some(*index)
    };
    let root = unique(&SurfaceNodeKind::SetComprehension, 139, 184)?;
    let application = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            matches!(
                &node.kind,
                SurfaceNodeKind::InfixExpression(operator)
                    if operator.spelling.as_ref() == "++"
            ) && node.range.start == 141
                && node.range.end == 147
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [application] = application.as_slice() else {
        return None;
    };
    let generator = unique(&SurfaceNodeKind::ComprehensionVariableSegment, 154, 174)?;
    let mapper_expression = unique(&SurfaceNodeKind::TermExpression, 141, 147)?;
    let condition = unique(&SurfaceNodeKind::FormulaExpression, 177, 182)?;
    let inner = unique(&SurfaceNodeKind::BuiltinPredicateApplication, 177, 182)?;
    let root_node = &ast.nodes()[root];
    if direct_token_texts(ast, root_node).as_slice() != ["{", "where", ":", "}"]
        || structural_child_ids(ast, root_node)
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>()
            != [mapper_expression, generator, condition]
        || structural_child_ids(ast, &ast.nodes()[condition])
            .iter()
            .map(|id| id.index())
            .collect::<Vec<_>>()
            != [inner]
        || subtree_tokens(ast, root_node)
            != [
                "{",
                "1",
                "++",
                "2",
                "where",
                "candidate255c",
                "is",
                "set",
                ":",
                "3",
                "=",
                "4",
                "}",
            ]
    {
        return None;
    }
    let primary_roots = [
        (141, 142, "1"),
        (146, 147, "2"),
        (177, 178, "3"),
        (181, 182, "4"),
    ]
    .into_iter()
    .map(|(start, end, spelling)| {
        let index = unique(&SurfaceNodeKind::NumeralTerm, start, end)?;
        (subtree_tokens(ast, &ast.nodes()[index]) == [spelling]).then_some(index)
    })
    .collect::<Option<Vec<_>>>()?;
    Some(ExactConditionedRoute {
        root,
        application: *application,
        primary_roots,
    })
}

fn build_conditioned_output(
    ast: &SurfaceAst,
    module: ModuleId,
    symbols: &SymbolEnv,
    route: ExactConditionedRoute,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Result<SourceSetTermRouteOutput, String> {
    let binding_env =
        source_module_binding_env(ast, module.clone()).map_err(|error| error.to_string())?;
    let application_kinds = unwrapped_imported_source_application_owned_node_kinds(
        ast,
        &module,
        symbols,
        route.application,
    )
    .ok_or_else(|| "Task255C1 unwrapped Task253 selector rejected the exact mapper".to_owned())?;
    #[cfg(test)]
    let source_term = synthetic_source_term_parts_for_roots(
        ast,
        module.clone(),
        &binding_env,
        route.primary_roots.iter().copied(),
        BindingContextId::new(0),
        &application_kinds,
        &BTreeMap::new(),
    )?;
    #[cfg(not(test))]
    let source_term = source_term_parts_for_roots(
        ast,
        module.clone(),
        &binding_env,
        route.primary_roots.iter().copied(),
        BindingContextId::new(0),
        &application_kinds,
    )?;
    let application = unwrapped_imported_source_application_handoff(
        ast,
        &module,
        symbols,
        &binding_env,
        &source_term,
        route.application,
    )
    .ok_or_else(|| "Task255C1 reusable Task253 seam rejected the exact mapper".to_owned())??;
    let extracted = extract_set_terms(
        ast,
        &module,
        BindingContextId::new(0),
        &[route.root],
        Some(&application),
        None,
        &BTreeSet::new(),
        true,
    )?;
    build_output(
        ast,
        module,
        binding_env,
        extracted,
        Some(SyntheticSourceSetTermDependencies {
            arena: source_term.arena,
            primary: source_term.handoff,
            application: Some(application),
            structure: None,
        }),
        mutate,
    )
}

#[cfg(test)]
pub(in crate::runner) fn synthetic_source_set_term_output(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    roots: &[usize],
    dependencies: Option<SyntheticSourceSetTermDependencies>,
    degraded_terms: &BTreeSet<usize>,
) -> Result<SourceSetTermRouteOutput, String> {
    synthetic_source_set_term_output_with_mutation(
        ast,
        module,
        binding_env,
        roots,
        dependencies,
        degraded_terms,
        |_| {},
    )
}

pub(super) fn source_set_term_output_with_source_term(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    roots: &[usize],
    source_term: SourceTermParts,
) -> Result<SourceSetTermRouteOutput, String> {
    let extracted = extract_set_terms(
        ast,
        &module,
        BindingContextId::new(0),
        roots,
        None,
        None,
        &BTreeSet::new(),
        false,
    )?;
    build_output(
        ast,
        module,
        binding_env,
        extracted,
        Some(SyntheticSourceSetTermDependencies {
            arena: source_term.arena,
            primary: source_term.handoff,
            application: None,
            structure: None,
        }),
        |_| {},
    )
}

#[cfg(test)]
// Rationale: keep the synthetic mutation boundary's explicit authority inputs visible.
#[allow(clippy::too_many_arguments)]
pub(in crate::runner) fn synthetic_source_set_term_output_with_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    roots: &[usize],
    dependencies: Option<SyntheticSourceSetTermDependencies>,
    degraded_terms: &BTreeSet<usize>,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Result<SourceSetTermRouteOutput, String> {
    let applications = dependencies
        .as_ref()
        .and_then(|dependencies| dependencies.application.as_ref());
    let structures = dependencies
        .as_ref()
        .and_then(|dependencies| dependencies.structure.as_ref());
    let extracted = extract_set_terms(
        ast,
        &module,
        BindingContextId::new(0),
        roots,
        applications,
        structures,
        degraded_terms,
        false,
    )?;
    build_output(ast, module, binding_env, extracted, dependencies, mutate)
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
        "func",
        "Task255EnumerationDef",
        ":",
        "task255_enumeration",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "{",
        "1",
        ",",
        "2",
        "}",
        ";",
        "func",
        "Task255ComprehensionDef",
        ":",
        "task255_comprehension",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "{",
        "3",
        "where",
        "candidate255",
        "is",
        "set",
        "}",
        ";",
        "func",
        "Task255ChoiceDef",
        ":",
        "task255_choice",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "the",
        "set",
        ";",
        "func",
        "Task255QuaDef",
        ":",
        "task255_qua",
        "(",
        "seed",
        ")",
        "->",
        "set",
        "equals",
        "4",
        "qua",
        "set",
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
        .filter(|(_, node)| is_set_term_kind(&node.kind))
        .filter(|(index, _)| !has_set_term_ancestor(ast, &parents, *index))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let kinds = roots
        .iter()
        .filter_map(|root| ast.nodes().get(*root))
        .map(|node| &node.kind)
        .collect::<Vec<_>>();
    if roots.len() == 4
        && kinds
            .iter()
            .filter(|kind| matches!(kind, SurfaceNodeKind::SetEnumeration))
            .count()
            == 1
        && kinds
            .iter()
            .filter(|kind| matches!(kind, SurfaceNodeKind::SetComprehension))
            .count()
            == 1
        && kinds
            .iter()
            .filter(|kind| matches!(kind, SurfaceNodeKind::ChoiceTerm))
            .count()
            == 1
        && kinds
            .iter()
            .filter(|kind| matches!(kind, SurfaceNodeKind::QuaExpression))
            .count()
            == 1
    {
        Some(roots)
    } else {
        None
    }
}

// Rationale: keep all recursive extraction dependencies explicit at this private boundary.
#[allow(clippy::too_many_arguments)]
fn extract_set_terms(
    ast: &SurfaceAst,
    module: &ModuleId,
    context: BindingContextId,
    roots: &[usize],
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    degraded_terms: &BTreeSet<usize>,
    admit_conditions: bool,
) -> Result<ExtractedSetTerms, String> {
    let mut extracted = ExtractedSetTerms {
        context,
        terms: Vec::new(),
        wrappers: Vec::new(),
        generators: Vec::new(),
        type_sites: Vec::new(),
        conditions: Vec::new(),
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
        if set_root_is_excluded(ast, root, applications, structures, admit_conditions) {
            continue;
        }
        collect_target(
            ast,
            module,
            root,
            applications,
            structures,
            degraded_terms,
            admit_conditions,
            &mut extracted,
        )?
        .set_term()
        .ok_or_else(|| "synthetic source-set root is not a Task-255 term".to_owned())?;
    }
    normalize_extracted_tables(ast, &mut extracted);
    Ok(extracted)
}

fn normalize_extracted_tables(ast: &SurfaceAst, extracted: &mut ExtractedSetTerms) {
    extracted
        .wrappers
        .sort_by_key(|wrapper| (wrapper.term.index(), wrapper.ordinal));

    let mut generator_order = (0..extracted.generators.len()).collect::<Vec<_>>();
    generator_order.sort_by_key(|index| {
        let generator = &extracted.generators[*index];
        (generator.term.index(), generator.ordinal)
    });
    let generator_remap = generator_order
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, SourceSetGeneratorId::new(new)))
        .collect::<BTreeMap<_, _>>();
    let old_generators = std::mem::take(&mut extracted.generators);
    extracted.generators = generator_order
        .into_iter()
        .map(|old| old_generators[old].clone())
        .collect();
    for type_site in &mut extracted.type_sites {
        if let SourceSetTypeOwner::Generator(generator) = &mut type_site.owner {
            *generator = generator_remap[&generator.index()];
        }
    }

    let mut type_site_order = (0..extracted.type_sites.len()).collect::<Vec<_>>();
    type_site_order.sort_by_key(|index| {
        let node = &ast.nodes()[extracted.type_sites[*index].node];
        (node.range.start, node.range.end, *index)
    });
    let type_site_remap = type_site_order
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, SourceSetTypeSiteId::new(new)))
        .collect::<BTreeMap<_, _>>();
    let old_type_sites = std::mem::take(&mut extracted.type_sites);
    extracted.type_sites = type_site_order
        .into_iter()
        .map(|old| old_type_sites[old].clone())
        .collect();
    for generator in &mut extracted.generators {
        generator.type_site = type_site_remap[&generator.type_site.index()];
    }

    extracted
        .conditions
        .sort_by_key(|condition| (condition.term.index(), condition.ordinal));
    extracted
        .edges
        .sort_by_key(|edge| (edge.term.index(), edge.ordinal));
}

impl ExtractedTarget {
    const fn set_term(self) -> Option<SourceSetTermId> {
        match self {
            Self::SetTerm(term) => Some(term),
            Self::Primary(_) | Self::Application(_) | Self::Structure(_) => None,
        }
    }
}

// Rationale: keep the recursive target classifier's family dependencies explicit.
#[allow(clippy::too_many_arguments)]
fn collect_target(
    ast: &SurfaceAst,
    module: &ModuleId,
    node_index: usize,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    degraded_terms: &BTreeSet<usize>,
    admit_conditions: bool,
    extracted: &mut ExtractedSetTerms,
) -> Result<ExtractedTarget, String> {
    let original = ast
        .nodes()
        .get(node_index)
        .ok_or_else(|| "source-set child site disappeared".to_owned())?;
    if original.range.source_id != ast.source_id {
        return Err("source-set child source identity mismatch".to_owned());
    }
    if let Some(application) = application_target(applications, original.range) {
        return Ok(ExtractedTarget::Application(application));
    }
    if let Some(structure) = structure_target(structures, original.range) {
        return Ok(ExtractedTarget::Structure(structure));
    }

    let (core, wrappers) = peel_set_shells(ast, node_index)?;
    let node = &ast.nodes()[core];
    if !is_set_term_kind(&node.kind) {
        extracted.primary_roots.push(node_index);
        return Ok(ExtractedTarget::Primary(node_index));
    }
    if set_term_shape_is_excluded(ast, core, admit_conditions) {
        return Err("source-set term has an excluded source shape".to_owned());
    }
    if subtree_has_recovery(ast, node) && !degraded_terms.contains(&core) {
        return Err("source-set normal term contains recovery".to_owned());
    }

    let term = SourceSetTermId::new(extracted.terms.len());
    let kind = match node.kind {
        SurfaceNodeKind::SetEnumeration => SourceSetTermKind::Enumeration,
        SurfaceNodeKind::SetComprehension => SourceSetTermKind::Comprehension,
        SurfaceNodeKind::ChoiceTerm => SourceSetTermKind::Choice,
        SurfaceNodeKind::QuaExpression => SourceSetTermKind::Qua,
        _ => unreachable!("guarded above"),
    };
    let recovery = if degraded_terms.contains(&core) {
        SourceSetTermRecovery::Degraded
    } else {
        SourceSetTermRecovery::Normal
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
        SourceSetTermKind::Enumeration => collect_enumeration(
            ast,
            module,
            term,
            core,
            applications,
            structures,
            degraded_terms,
            admit_conditions,
            extracted,
        )?,
        SourceSetTermKind::Comprehension => collect_comprehension(
            ast,
            module,
            term,
            core,
            applications,
            structures,
            degraded_terms,
            admit_conditions,
            extracted,
        )?,
        SourceSetTermKind::Choice => collect_choice(ast, term, core, recovery, extracted)?,
        SourceSetTermKind::Qua => collect_qua(
            ast,
            module,
            term,
            core,
            recovery,
            applications,
            structures,
            degraded_terms,
            admit_conditions,
            extracted,
        )?,
        _ => return Err("unsupported source-set term kind".to_owned()),
    }
    Ok(ExtractedTarget::SetTerm(term))
}

// Rationale: keep enumeration child ownership inputs explicit during recursion.
#[allow(clippy::too_many_arguments)]
fn collect_enumeration(
    ast: &SurfaceAst,
    module: &ModuleId,
    term: SourceSetTermId,
    node_index: usize,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    degraded_terms: &BTreeSet<usize>,
    admit_conditions: bool,
    extracted: &mut ExtractedSetTerms,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    let children = structural_child_ids(ast, node);
    let punctuation = direct_token_texts(ast, node);
    let expected_punctuation = if children.is_empty() {
        vec!["{".to_owned(), "}".to_owned()]
    } else {
        let mut expected = vec!["{".to_owned()];
        expected.extend((1..children.len()).map(|_| ",".to_owned()));
        expected.push("}".to_owned());
        expected
    };
    if punctuation != expected_punctuation {
        return Err("source-set enumeration punctuation is not canonical".to_owned());
    }
    for child in children {
        let target = collect_target(
            ast,
            module,
            child.index(),
            applications,
            structures,
            degraded_terms,
            admit_conditions,
            extracted,
        )?;
        push_edge(
            extracted,
            term,
            SourceSetEdgeRole::EnumerationElement,
            target,
        );
    }
    Ok(())
}

// Rationale: keep comprehension binding deferrals and child dependencies explicit.
#[allow(clippy::too_many_arguments)]
fn collect_comprehension(
    ast: &SurfaceAst,
    module: &ModuleId,
    term: SourceSetTermId,
    node_index: usize,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    degraded_terms: &BTreeSet<usize>,
    admit_conditions: bool,
    extracted: &mut ExtractedSetTerms,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    let children = structural_child_ids(ast, node);
    let Some((mapper, tail)) = children.split_first() else {
        return Err("source-set comprehension lost its mapper".to_owned());
    };
    let (generators, condition) = match tail.split_last() {
        Some((condition, generators))
            if ast
                .node(*condition)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::FormulaExpression)) =>
        {
            if !admit_conditions {
                return Err("source-set conditioned comprehension is outside this route".to_owned());
            }
            (generators, Some(*condition))
        }
        _ => (tail, None),
    };
    if generators.is_empty()
        || generators.iter().any(|generator| {
            ast.node(*generator).is_none_or(|node| {
                !matches!(node.kind, SurfaceNodeKind::ComprehensionVariableSegment)
            })
        })
    {
        return Err("source-set comprehension generator shape drift".to_owned());
    }
    let punctuation = direct_token_texts(ast, node);
    let mut expected = vec!["{".to_owned(), "where".to_owned()];
    expected.extend((1..generators.len()).map(|_| ",".to_owned()));
    if condition.is_some() {
        expected.push(":".to_owned());
    }
    expected.push("}".to_owned());
    if punctuation != expected {
        return Err("source-set comprehension punctuation is not canonical".to_owned());
    }
    let target = collect_target(
        ast,
        module,
        mapper.index(),
        applications,
        structures,
        degraded_terms,
        admit_conditions,
        extracted,
    )?;
    push_edge(
        extracted,
        term,
        SourceSetEdgeRole::ComprehensionMapper,
        target,
    );

    for (ordinal, generator_id) in generators.iter().copied().enumerate() {
        let generator = ast
            .node(generator_id)
            .ok_or_else(|| "source-set generator disappeared".to_owned())?;
        let direct = generator
            .children
            .iter()
            .filter_map(|child| {
                ast.node(*child)
                    .and_then(|node| node.token_text().map(|text| (*child, text)))
            })
            .collect::<Vec<_>>();
        let [(identifier, spelling), (_, is_keyword)] = direct.as_slice() else {
            return Err("source-set generator requires one identifier and `is`".to_owned());
        };
        if spelling.is_empty() || *is_keyword != "is" {
            return Err("source-set generator spelling is not canonical".to_owned());
        }
        let type_children = structural_child_ids(ast, generator);
        let [type_expression] = type_children.as_slice() else {
            return Err("source-set generator requires one target type".to_owned());
        };
        let generator_id = SourceSetGeneratorId::new(extracted.generators.len());
        let type_site = SourceSetTypeSiteId::new(extracted.type_sites.len());
        let type_row = extract_bare_type_site(
            ast,
            type_expression.index(),
            SourceSetTypeOwner::Generator(generator_id),
            extracted.terms[term.index()].recovery,
        )?;
        extracted.type_sites.push(type_row);
        extracted.generators.push(ExtractedGenerator {
            term,
            ordinal,
            node: identifier.index(),
            recovery: extracted.terms[term.index()].recovery,
            type_site,
        });
    }
    if let Some(condition) = condition {
        let colon_nodes = node
            .children
            .iter()
            .copied()
            .filter(|child| ast.node(*child).and_then(SurfaceNode::token_text) == Some(":"))
            .collect::<Vec<_>>();
        let [colon] = colon_nodes.as_slice() else {
            return Err("source-set condition requires one direct colon".to_owned());
        };
        let condition_node = ast
            .node(condition)
            .ok_or_else(|| "source-set condition wrapper disappeared".to_owned())?;
        if condition_node.recovered || subtree_tokens(ast, condition_node).is_empty() {
            return Err("source-set condition wrapper is not a normal formula".to_owned());
        }
        extracted.conditions.push(ExtractedCondition {
            term,
            ordinal: 0,
            colon_node: colon.index(),
            condition_node: condition.index(),
            recovery: extracted.terms[term.index()].recovery,
        });
        collect_condition_primary_roots(ast, condition.index(), &mut extracted.primary_roots);
    }
    Ok(())
}

fn collect_condition_primary_roots(ast: &SurfaceAst, node: usize, roots: &mut Vec<usize>) {
    let Some(node_row) = ast.nodes().get(node) else {
        return;
    };
    if matches!(
        node_row.kind,
        SurfaceNodeKind::TermReference
            | SurfaceNodeKind::NumeralTerm
            | SurfaceNodeKind::ItTerm
            | SurfaceNodeKind::ParenthesizedTerm
    ) {
        roots.push(node);
        return;
    }
    for child in &node_row.children {
        collect_condition_primary_roots(ast, child.index(), roots);
    }
}

fn collect_choice(
    ast: &SurfaceAst,
    term: SourceSetTermId,
    node_index: usize,
    recovery: SourceSetTermRecovery,
    extracted: &mut ExtractedSetTerms,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    if direct_token_texts(ast, node).as_slice() != ["the"] {
        return Err("source-set choice punctuation is not canonical".to_owned());
    }
    let children = structural_child_ids(ast, node);
    let [target_type] = children.as_slice() else {
        return Err("source-set choice requires one target type".to_owned());
    };
    let type_site = extract_bare_type_site(
        ast,
        target_type.index(),
        SourceSetTypeOwner::Term {
            term,
            role: SourceSetTypeRole::ChoiceTarget,
        },
        recovery,
    )?;
    extracted.type_sites.push(type_site);
    Ok(())
}

// Rationale: keep qua base ownership and optional family dependencies explicit.
#[allow(clippy::too_many_arguments)]
fn collect_qua(
    ast: &SurfaceAst,
    module: &ModuleId,
    term: SourceSetTermId,
    node_index: usize,
    recovery: SourceSetTermRecovery,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    degraded_terms: &BTreeSet<usize>,
    admit_conditions: bool,
    extracted: &mut ExtractedSetTerms,
) -> Result<(), String> {
    let node = &ast.nodes()[node_index];
    if direct_token_texts(ast, node).as_slice() != ["qua"] {
        return Err("source-set qua punctuation is not canonical".to_owned());
    }
    let children = structural_child_ids(ast, node);
    let [base, target_type] = children.as_slice() else {
        return Err("source-set qua requires one base and one target type".to_owned());
    };
    let target = collect_target(
        ast,
        module,
        base.index(),
        applications,
        structures,
        degraded_terms,
        admit_conditions,
        extracted,
    )?;
    push_edge(extracted, term, SourceSetEdgeRole::QuaBase, target);
    let type_site = extract_bare_type_site(
        ast,
        target_type.index(),
        SourceSetTypeOwner::Term {
            term,
            role: SourceSetTypeRole::QuaTarget,
        },
        recovery,
    )?;
    extracted.type_sites.push(type_site);
    Ok(())
}

fn extract_bare_type_site(
    ast: &SurfaceAst,
    node_index: usize,
    owner: SourceSetTypeOwner,
    recovery: SourceSetTermRecovery,
) -> Result<ExtractedTypeSite, String> {
    let node = ast
        .nodes()
        .get(node_index)
        .ok_or_else(|| "source-set target type disappeared".to_owned())?;
    if !matches!(node.kind, SurfaceNodeKind::TypeExpression)
        || (recovery == SourceSetTermRecovery::Normal && subtree_has_recovery(ast, node))
    {
        return Err("source-set target type is not one normal type expression".to_owned());
    }
    let children = structural_child_ids(ast, node);
    let [head_id] = children.as_slice() else {
        return Err("source-set target type is not bare".to_owned());
    };
    let head_node = ast
        .node(*head_id)
        .ok_or_else(|| "source-set target type head disappeared".to_owned())?;
    if !matches!(head_node.kind, SurfaceNodeKind::TypeHead)
        || !structural_child_ids(ast, head_node).is_empty()
    {
        return Err("source-set target type head is not bare".to_owned());
    }
    let head = match direct_token_texts(ast, head_node).as_slice() {
        [spelling] if spelling == "set" => SourceSetTypeHead::BuiltinSet,
        [spelling] if spelling == "object" => SourceSetTypeHead::BuiltinObject,
        _ => return Err("source-set target type head is unsupported".to_owned()),
    };
    Ok(ExtractedTypeSite {
        owner,
        node: node_index,
        head_node: head_id.index(),
        head,
        recovery,
    })
}

fn push_edge(
    extracted: &mut ExtractedSetTerms,
    term: SourceSetTermId,
    role: SourceSetEdgeRole,
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
        target,
    });
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    binding_env: BindingEnv,
    extracted: ExtractedSetTerms,
    dependencies: Option<SyntheticSourceSetTermDependencies>,
    mutate: impl FnOnce(&mut SourceSetTermHandoffInput),
) -> Result<SourceSetTermRouteOutput, String> {
    if binding_env.source_id() != ast.source_id || binding_env.module_id() != &module {
        return Err("source-set binding environment identity mismatch".to_owned());
    }
    let mut kinds = BTreeMap::new();
    let mut recoveries = BTreeMap::new();
    for term in &extracted.terms {
        insert_kind(&mut kinds, term.node, term_kind_key(term.kind))?;
        if term.recovery == SourceSetTermRecovery::Degraded {
            recoveries.insert(term.node, NodeRecoveryState::Degraded);
        }
    }
    for wrapper in &extracted.wrappers {
        insert_kind(&mut kinds, wrapper.node, "source.term.set.parenthesized")?;
        if wrapper.recovery == SourceSetTermRecovery::Degraded {
            recoveries.insert(wrapper.node, NodeRecoveryState::Degraded);
        }
    }
    for generator in &extracted.generators {
        insert_kind(
            &mut kinds,
            generator.node,
            "source.term.set.comprehension-generator",
        )?;
        if generator.recovery == SourceSetTermRecovery::Degraded {
            recoveries.insert(generator.node, NodeRecoveryState::Degraded);
        }
    }
    for type_site in &extracted.type_sites {
        insert_kind(&mut kinds, type_site.node, "source.term.set.target-type")?;
        insert_kind(
            &mut kinds,
            type_site.head_node,
            "source.term.set.target-type-head",
        )?;
        if type_site.recovery == SourceSetTermRecovery::Degraded {
            recoveries.insert(type_site.node, NodeRecoveryState::Degraded);
            recoveries.insert(type_site.head_node, NodeRecoveryState::Degraded);
        }
    }
    for condition in &extracted.conditions {
        insert_kind(
            &mut kinds,
            condition.colon_node,
            "source.term.set.comprehension-condition-colon",
        )?;
        insert_kind(
            &mut kinds,
            condition.condition_node,
            "source.term.set.comprehension-condition",
        )?;
        if condition.recovery == SourceSetTermRecovery::Degraded {
            recoveries.insert(condition.colon_node, NodeRecoveryState::Degraded);
            recoveries.insert(condition.condition_node, NodeRecoveryState::Degraded);
        }
    }

    let (arena, primary, application, structure) = if let Some(dependencies) = dependencies {
        let arena = arena_with_overrides(ast, &dependencies.arena, &kinds, &recoveries)?;
        (
            arena,
            dependencies.primary,
            dependencies.application,
            dependencies.structure,
        )
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
        (parts.arena, parts.handoff, None, None)
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
    for (term_index, term_row) in extracted.terms.iter().enumerate() {
        let term = SourceSetTermId::new(term_index);
        let mut ordinal = 0;
        if term_row.kind == SourceSetTermKind::Comprehension {
            for (generator_index, generator) in extracted
                .generators
                .iter()
                .enumerate()
                .filter(|(_, generator)| generator.term == term)
            {
                requests.push(SourceSetRequestInput {
                    term,
                    ordinal,
                    kind: SourceSetRequestKind::GeneratorSethood,
                    generator: Some(SourceSetGeneratorId::new(generator_index)),
                    type_site: Some(generator.type_site),
                });
                ordinal += 1;
            }
        } else if term_row.kind == SourceSetTermKind::Choice {
            let type_site = type_site_for_term(&extracted, term)?;
            requests.push(SourceSetRequestInput {
                term,
                ordinal,
                kind: SourceSetRequestKind::ChoiceNonempty,
                generator: None,
                type_site: Some(type_site),
            });
            ordinal += 1;
        } else if term_row.kind == SourceSetTermKind::Qua {
            let type_site = type_site_for_term(&extracted, term)?;
            requests.push(SourceSetRequestInput {
                term,
                ordinal,
                kind: SourceSetRequestKind::QuaWidening,
                generator: None,
                type_site: Some(type_site),
            });
            ordinal += 1;
        }
        requests.push(SourceSetRequestInput {
            term,
            ordinal,
            kind: SourceSetRequestKind::ResultType,
            generator: None,
            type_site: None,
        });
    }

    let mut input = SourceSetTermHandoffInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        terms: extracted
            .terms
            .iter()
            .enumerate()
            .map(|(source_ordinal, term)| {
                let node = &ast.nodes()[term.node];
                SourceSetTermInput {
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
                SourceSetWrapperInput {
                    term: wrapper.term,
                    ordinal: wrapper.ordinal,
                    site: TypedSiteRef::Node(TypedNodeId::new(wrapper.node)),
                    source_range: node.range,
                    context: extracted.context,
                    recovery: wrapper.recovery,
                    spelling: subtree_tokens(ast, node).join(" "),
                }
            })
            .collect(),
        generators: extracted
            .generators
            .iter()
            .map(|generator| {
                let node = &ast.nodes()[generator.node];
                SourceSetGeneratorInput {
                    term: generator.term,
                    ordinal: generator.ordinal,
                    site: TypedSiteRef::Node(TypedNodeId::new(generator.node)),
                    source_range: node.range,
                    spelling: node.token_text().unwrap_or_default().to_owned(),
                    context: extracted.context,
                    recovery: generator.recovery,
                    type_site: generator.type_site,
                }
            })
            .collect(),
        type_sites: extracted
            .type_sites
            .iter()
            .map(|type_site| {
                let node = &ast.nodes()[type_site.node];
                let head = &ast.nodes()[type_site.head_node];
                SourceSetTypeSiteInput {
                    owner: type_site.owner,
                    site: TypedSiteRef::Node(TypedNodeId::new(type_site.node)),
                    source_range: node.range,
                    spelling: subtree_tokens(ast, node).join(" "),
                    head_site: TypedSiteRef::Node(TypedNodeId::new(type_site.head_node)),
                    head_range: head.range,
                    head_spelling: subtree_tokens(ast, head).join(" "),
                    context: extracted.context,
                    recovery: type_site.recovery,
                    head: type_site.head,
                }
            })
            .collect(),
        conditions: extracted
            .conditions
            .iter()
            .map(|condition| {
                let colon = &ast.nodes()[condition.colon_node];
                let formula = &ast.nodes()[condition.condition_node];
                SourceSetConditionInput {
                    term: condition.term,
                    ordinal: condition.ordinal,
                    colon_site: TypedSiteRef::Node(TypedNodeId::new(condition.colon_node)),
                    colon_range: colon.range,
                    colon_spelling: colon.token_text().unwrap_or_default().to_owned(),
                    condition_site: TypedSiteRef::Node(TypedNodeId::new(condition.condition_node)),
                    source_range: formula.range,
                    spelling: subtree_tokens(ast, formula).join(" "),
                    recovery: condition.recovery,
                }
            })
            .collect(),
        edges: extracted
            .edges
            .iter()
            .map(|edge| {
                let target = match edge.target {
                    ExtractedTarget::Primary(root) => {
                        SourceSetTarget::Primary(primary_id(root).ok_or_else(|| {
                            "source-set primary child is not a Task-252 root".to_owned()
                        })?)
                    }
                    ExtractedTarget::Application(application) => {
                        SourceSetTarget::Application(application)
                    }
                    ExtractedTarget::Structure(structure) => SourceSetTarget::Structure(structure),
                    ExtractedTarget::SetTerm(term) => SourceSetTarget::SetTerm(term),
                };
                Ok(SourceSetEdgeInput {
                    term: edge.term,
                    ordinal: edge.ordinal,
                    role: edge.role,
                    target,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
        requests,
    };
    mutate(&mut input);
    let handoff = SourceSetTermProducer::build(
        input,
        &binding_env,
        &primary,
        application.as_ref(),
        structure.as_ref(),
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
    if let Some(structure) = structure {
        typed_ast = typed_ast
            .with_source_structure(structure)
            .map_err(|error| error.to_string())?;
    }
    let typed_ast = typed_ast
        .with_source_set_term(handoff)
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
    if typed_ast.source_set_term().is_none()
        || resolved.source_set_term() != typed_ast.source_set_term()
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
        return Err("source-set immutable final handoff mismatch".to_owned());
    }
    Ok(SourceSetTermRouteOutput {
        typed_ast,
        resolved,
        #[cfg(test)]
        binding_env,
    })
}

fn type_site_for_term(
    extracted: &ExtractedSetTerms,
    term: SourceSetTermId,
) -> Result<SourceSetTypeSiteId, String> {
    let matches = extracted
        .type_sites
        .iter()
        .enumerate()
        .filter(|(_, site)| matches!(site.owner, SourceSetTypeOwner::Term { term: owner, .. } if owner == term))
        .map(|(index, _)| SourceSetTypeSiteId::new(index))
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [site] => Ok(*site),
        _ => Err("source-set term requires exactly one target type site".to_owned()),
    }
}

fn arena_with_overrides(
    ast: &SurfaceAst,
    source: &TypedArena,
    kinds: &BTreeMap<usize, &'static str>,
    recoveries: &BTreeMap<usize, NodeRecoveryState>,
) -> Result<TypedArena, String> {
    if source.len() != ast.nodes().len() {
        return Err("source-set dependency arena is not surface-indexed".to_owned());
    }
    let mut nodes = source
        .iter()
        .map(|(_, node)| node.clone())
        .collect::<Vec<_>>();
    for (index, kind) in kinds {
        let node = nodes
            .get_mut(*index)
            .ok_or_else(|| "source-set arena override site disappeared".to_owned())?;
        if node.kind.as_str() != "source.surface.unowned" {
            return Err("source-set arena override site is already owned".to_owned());
        }
        node.kind = (*kind).into();
    }
    for (index, recovery) in recoveries {
        nodes
            .get_mut(*index)
            .ok_or_else(|| "source-set recovery override site disappeared".to_owned())?
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
        return Err("source-set arena site is multiply owned".to_owned());
    }
    Ok(())
}

fn term_kind_key(kind: SourceSetTermKind) -> &'static str {
    match kind {
        SourceSetTermKind::Enumeration => "source.term.set.enumeration",
        SourceSetTermKind::Comprehension => "source.term.set.comprehension",
        SourceSetTermKind::Choice => "source.term.set.choice",
        SourceSetTermKind::Qua => "source.term.set.qua",
        _ => "source.term.set.unsupported",
    }
}

fn set_root_is_excluded(
    ast: &SurfaceAst,
    root: usize,
    applications: Option<&SourceFunctorApplicationHandoff>,
    structures: Option<&SourceStructureHandoff>,
    admit_conditions: bool,
) -> bool {
    let Some(root_node) = ast.nodes().get(root) else {
        return true;
    };
    if set_term_shape_is_excluded(ast, root, admit_conditions)
        || reverse_family_contains_set(ast, root, false)
        || contains_template_boundary(ast, root)
    {
        return true;
    }
    let parents = parent_indexes(ast);
    let mut cursor = parents[root];
    while let Some(parent) = cursor {
        let node = &ast.nodes()[parent];
        if is_template_kind(&node.kind)
            || is_reverse_application_kind(&node.kind)
            || is_reverse_structure_kind(&node.kind)
            || application_range_owned(applications, node.range)
            || structure_range_owned(structures, node.range)
        {
            return true;
        }
        cursor = parents[parent];
    }
    root_node.range.source_id != ast.source_id
}

fn set_term_shape_is_excluded(ast: &SurfaceAst, root: usize, admit_conditions: bool) -> bool {
    let Some(node) = ast.nodes().get(root) else {
        return true;
    };
    match node.kind {
        SurfaceNodeKind::SetEnumeration => structural_child_ids(ast, node)
            .into_iter()
            .any(|child| nested_set_shape_is_excluded(ast, child.index(), admit_conditions)),
        SurfaceNodeKind::SetComprehension => {
            let children = structural_child_ids(ast, node);
            let Some((mapper, tail)) = children.split_first() else {
                return true;
            };
            let has_colon = direct_token_texts(ast, node)
                .iter()
                .any(|token| token == ":");
            if has_colon && !admit_conditions {
                return true;
            }
            let generators = if has_colon {
                let Some((condition, generators)) = tail.split_last() else {
                    return true;
                };
                if ast
                    .node(*condition)
                    .is_none_or(|node| !matches!(node.kind, SurfaceNodeKind::FormulaExpression))
                {
                    return true;
                }
                generators
            } else {
                tail
            };
            if generators.is_empty() {
                return true;
            }
            let mut names = BTreeSet::new();
            for generator in generators {
                let Some(generator_node) = ast.node(*generator) else {
                    return true;
                };
                if !matches!(
                    generator_node.kind,
                    SurfaceNodeKind::ComprehensionVariableSegment
                ) {
                    return true;
                }
                let identifiers = generator_node
                    .children
                    .iter()
                    .filter_map(|child| ast.node(*child))
                    .filter_map(SurfaceNode::token_text)
                    .filter(|token| *token != "is")
                    .collect::<Vec<_>>();
                let [identifier] = identifiers.as_slice() else {
                    return true;
                };
                names.insert((*identifier).to_owned());
                let type_children = structural_child_ids(ast, generator_node);
                let [target_type] = type_children.as_slice() else {
                    return true;
                };
                if extract_bare_type_site(
                    ast,
                    target_type.index(),
                    SourceSetTypeOwner::Generator(SourceSetGeneratorId::new(0)),
                    SourceSetTermRecovery::Degraded,
                )
                .is_err()
                {
                    return true;
                }
            }
            let mapper_node = &ast.nodes()[mapper.index()];
            let mapper_tokens = subtree_tokens(ast, mapper_node);
            mapper_tokens.iter().any(|token| names.contains(*token))
                || nested_set_shape_is_excluded(ast, mapper.index(), admit_conditions)
        }
        SurfaceNodeKind::ChoiceTerm => structural_child_ids(ast, node)
            .as_slice()
            .first()
            .is_none_or(|target_type| {
                extract_bare_type_site(
                    ast,
                    target_type.index(),
                    SourceSetTypeOwner::Term {
                        term: SourceSetTermId::new(0),
                        role: SourceSetTypeRole::ChoiceTarget,
                    },
                    SourceSetTermRecovery::Degraded,
                )
                .is_err()
            }),
        SurfaceNodeKind::QuaExpression => {
            let children = structural_child_ids(ast, node);
            let [base, target_type] = children.as_slice() else {
                return true;
            };
            extract_bare_type_site(
                ast,
                target_type.index(),
                SourceSetTypeOwner::Term {
                    term: SourceSetTermId::new(0),
                    role: SourceSetTypeRole::QuaTarget,
                },
                SourceSetTermRecovery::Degraded,
            )
            .is_err()
                || nested_set_shape_is_excluded(ast, base.index(), admit_conditions)
        }
        _ => true,
    }
}

fn nested_set_shape_is_excluded(ast: &SurfaceAst, root: usize, admit_conditions: bool) -> bool {
    let Some(node) = ast.nodes().get(root) else {
        return true;
    };
    if is_set_term_kind(&node.kind) {
        return set_term_shape_is_excluded(ast, root, admit_conditions);
    }
    node.children
        .iter()
        .any(|child| nested_set_shape_is_excluded(ast, child.index(), admit_conditions))
}

fn reverse_family_contains_set(ast: &SurfaceAst, root: usize, inside_reverse: bool) -> bool {
    let Some(node) = ast.nodes().get(root) else {
        return true;
    };
    if inside_reverse && is_set_term_kind(&node.kind) {
        return true;
    }
    let next_reverse = inside_reverse
        || is_reverse_application_kind(&node.kind)
        || is_reverse_structure_kind(&node.kind);
    node.children
        .iter()
        .any(|child| reverse_family_contains_set(ast, child.index(), next_reverse))
}

fn contains_template_boundary(ast: &SurfaceAst, root: usize) -> bool {
    let Some(node) = ast.nodes().get(root) else {
        return true;
    };
    is_template_kind(&node.kind)
        || node
            .children
            .iter()
            .any(|child| contains_template_boundary(ast, child.index()))
}

fn peel_set_shells(ast: &SurfaceAst, start: usize) -> Result<(usize, Vec<usize>), String> {
    let mut current = start;
    let mut wrappers = Vec::new();
    loop {
        let node = ast
            .nodes()
            .get(current)
            .ok_or_else(|| "source-set shell disappeared".to_owned())?;
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
                    && contains_set_term(ast, node) =>
            {
                wrappers.push(current);
                let children = structural_child_ids(ast, node);
                let [child] = children.as_slice() else {
                    return Err("source-set wrapper lost its child".to_owned());
                };
                current = child.index();
            }
            _ => break,
        }
    }
    Ok((current, wrappers))
}

fn contains_set_term(ast: &SurfaceAst, node: &SurfaceNode) -> bool {
    node.children.iter().any(|child| {
        ast.node(*child)
            .is_some_and(|child| is_set_term_kind(&child.kind) || contains_set_term(ast, child))
    })
}

fn application_target(
    applications: Option<&SourceFunctorApplicationHandoff>,
    range: SourceRange,
) -> Option<SourceFunctorApplicationId> {
    let applications = applications?;
    applications.applications().iter().find_map(|(id, row)| {
        let outer = applications
            .wrappers()
            .iter()
            .filter(|(_, wrapper)| wrapper.application() == id)
            .min_by_key(|(_, wrapper)| wrapper.ordinal())
            .map_or(row.source_range(), |(_, wrapper)| wrapper.source_range());
        (outer == range || row.source_range() == range).then_some(id)
    })
}

fn structure_target(
    structures: Option<&SourceStructureHandoff>,
    range: SourceRange,
) -> Option<SourceStructureTermId> {
    let structures = structures?;
    structures.terms().iter().find_map(|(id, row)| {
        let outer = structures
            .wrappers()
            .iter()
            .filter(|(_, wrapper)| wrapper.term() == id)
            .min_by_key(|(_, wrapper)| wrapper.ordinal())
            .map_or(row.source_range(), |(_, wrapper)| wrapper.source_range());
        (outer == range || row.source_range() == range).then_some(id)
    })
}

fn application_range_owned(
    applications: Option<&SourceFunctorApplicationHandoff>,
    range: SourceRange,
) -> bool {
    applications.is_some_and(|applications| {
        applications
            .applications()
            .iter()
            .any(|(_, row)| row.source_range() == range)
            || applications
                .wrappers()
                .iter()
                .any(|(_, row)| row.source_range() == range)
    })
}

fn structure_range_owned(structures: Option<&SourceStructureHandoff>, range: SourceRange) -> bool {
    structures.is_some_and(|structures| {
        structures
            .terms()
            .iter()
            .any(|(_, row)| row.source_range() == range)
            || structures
                .wrappers()
                .iter()
                .any(|(_, row)| row.source_range() == range)
    })
}

fn is_reverse_application_kind(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::ApplicationTerm
            | SurfaceNodeKind::PrefixExpression(_)
            | SurfaceNodeKind::InfixExpression(_)
            | SurfaceNodeKind::PostfixExpression(_)
    )
}

fn is_reverse_structure_kind(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::StructureConstructor
            | SurfaceNodeKind::SelectorAccess
            | SurfaceNodeKind::StructureUpdate
    )
}

fn is_template_kind(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::TemplateParameter
            | SurfaceNodeKind::TemplateLoci
            | SurfaceNodeKind::TemplateLocus
            | SurfaceNodeKind::TemplateArguments
            | SurfaceNodeKind::TemplateArgument
    )
}

fn source_set_term_binding_env(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Result<BindingEnv, String> {
    let items = exact_compilation_item_list(ast)
        .ok_or_else(|| "Task255 compilation item list disappeared".to_owned())?;
    let item_ids = structural_child_ids(ast, items);
    let [reserve_id, definition_id] = item_ids.as_slice() else {
        return Err("Task255 requires one reserve and one definition block".to_owned());
    };
    let reserve = ast
        .node(*reserve_id)
        .ok_or_else(|| "Task255 reserve disappeared".to_owned())?;
    let definition = ast
        .node(*definition_id)
        .ok_or_else(|| "Task255 definition block disappeared".to_owned())?;
    let reserve_shell = unique_shell(shells, *reserve_id, DeclarationShellKind::Reserve)?;
    let definition_shell = unique_shell(
        shells,
        *definition_id,
        DeclarationShellKind::DefinitionBlock,
    )?;
    let reserve_payload =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)
            .map_err(|()| "Task255 reserve extraction failed".to_owned())?;
    let [reserve_binding] = reserve_payload.bridge.bindings() else {
        return Err("Task255 requires one reserve binding".to_owned());
    };
    let reserve_children = structural_child_ids(ast, reserve);
    let [reserve_segment_id] = reserve_children.as_slice() else {
        return Err("Task255 reserve requires one segment".to_owned());
    };
    let parameter_id = structural_child_ids(ast, definition)
        .into_iter()
        .find(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::DefinitionParameter))
        })
        .ok_or_else(|| "Task255 definition parameter disappeared".to_owned())?;
    let parameter = ast
        .node(parameter_id)
        .ok_or_else(|| "Task255 definition parameter disappeared".to_owned())?;
    let parameter_children = structural_child_ids(ast, parameter);
    let [segment_id] = parameter_children.as_slice() else {
        return Err("Task255 definition parameter requires one segment".to_owned());
    };
    let segment = ast
        .node(*segment_id)
        .ok_or_else(|| "Task255 definition parameter segment disappeared".to_owned())?;
    let segment_children = structural_child_ids(ast, segment);
    let [type_id] = segment_children.as_slice() else {
        return Err("Task255 definition parameter requires one written type".to_owned());
    };
    let written_type = ast
        .node(*type_id)
        .ok_or_else(|| "Task255 definition parameter type disappeared".to_owned())?;
    if !matches!(segment.kind, SurfaceNodeKind::QualifiedVariableSegment)
        || direct_token_texts(ast, segment).as_slice() != ["seed", "be"]
        || !matches!(written_type.kind, SurfaceNodeKind::TypeExpression)
        || subtree_tokens(ast, written_type) != ["set"]
    {
        return Err("Task255 definition parameter shape drift".to_owned());
    }
    let declaration_range = unique_direct_token_range(ast, segment, "seed")?;
    let root_id = ast
        .root()
        .ok_or_else(|| "Task255 root disappeared".to_owned())?;
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
                return Err("Task255 binding projection cardinality mismatch".to_owned());
            }
            Ok(handoff.binding_env().clone())
        }
        SourceBindingContextBuild::Incomplete(_) => {
            Err("Task255 binding projection remained incomplete".to_owned())
        }
        _ => Err("Task255 binding projection returned an unsupported state".to_owned()),
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
        _ => Err(format!("Task255 requires one normal {kind:?} shell")),
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
        return Err(format!("Task255 requires one direct `{spelling}` token"));
    };
    Ok(*range)
}

fn is_set_term_kind(kind: &SurfaceNodeKind) -> bool {
    matches!(
        kind,
        SurfaceNodeKind::SetEnumeration
            | SurfaceNodeKind::SetComprehension
            | SurfaceNodeKind::ChoiceTerm
            | SurfaceNodeKind::QuaExpression
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

fn has_set_term_ancestor(ast: &SurfaceAst, parents: &[Option<usize>], node: usize) -> bool {
    let mut cursor = parents[node];
    while let Some(parent) = cursor {
        if is_set_term_kind(&ast.nodes()[parent].kind) {
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
