use std::collections::BTreeSet;

use mizar_checker::{
    resolved_typed_ast::{
        ResolvedNodeKindHint, ResolvedNodeKindHintKind, ResolvedTypedAst, SourceNodeRole,
    },
    source_context::{
        SourceBindingContextBuild, SourceBindingContextInput, SourceBindingContextOwner,
        SourceBindingContextProducer, SourceBindingContextProjection, SourceBindingSiteInput,
        SourceBindingSiteRole, SourceItemInput, SourceItemRecovery, SourceItemRole,
        SourceItemVisibility,
    },
    source_type::{
        SourceTypeApplicationForm, SourceTypeApplicationInput, SourceTypeExpressionId,
        SourceTypeExpressionInput, SourceTypeHandoffInput, SourceTypeHead, SourceTypeProducer,
    },
    type_checker::TypeHeadInput,
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextId, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeRole, TypeTable, TypedArena, TypedArenaBuilder,
        TypedAst, TypedAstParts, TypedNode, TypedNodeId, TypedNodeLinks, TypedSiteRef, TypingState,
    },
};
use mizar_resolve::{
    declarations::{
        DeclarationShell, DeclarationShellKind, DeclarationShellSet,
        DeclarationShellVisibilityState,
    },
    env::SymbolEnv,
    names::{LocalTermBinding, LocalTermScope},
    resolved_ast::ModuleId,
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind};

use super::{
    checker_handoff::assemble_empty_resolved_typed_ast,
    source_ast::{
        direct_token_texts, exact_compilation_item_list, structural_child_ids, subtree_has_recovery,
    },
    source_reserve::{
        extract_builtin_source_reserve_declarations_after_node_guard,
        extract_builtin_source_type_expression,
    },
};

const INVALID_PAYLOAD_KEY: &str = "type_elaboration.checker.source_binding_context.invalid_payload";

#[cfg(test)]
macro_rules! test_mutable {
    ($name:ident = $value:expr) => {
        let mut $name = $value;
    };
}

#[cfg(not(test))]
macro_rules! test_mutable {
    ($name:ident = $value:expr) => {
        let $name = $value;
    };
}

#[derive(Debug)]
pub(in crate::runner) struct SourceBindingContextRouteOutput {
    pub(in crate::runner) typed_ast: TypedAst,
    pub(in crate::runner) resolved: ResolvedTypedAst,
}

pub(in crate::runner) fn source_binding_context_detail_keys(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Vec<String>> {
    source_binding_context_output(ast, module, shells, symbols).map(|result| match result {
        Ok(output) => {
            debug_assert_eq!(
                output.typed_ast.source_context(),
                output.resolved.source_context()
            );
            Vec::new()
        }
        Err(_) => vec![INVALID_PAYLOAD_KEY.to_owned()],
    })
}

pub(in crate::runner) fn source_binding_context_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
) -> Option<Result<SourceBindingContextRouteOutput, String>> {
    let candidate = candidate_items(ast)?;
    Some(build_output(ast, module, shells, symbols, candidate))
}

#[derive(Debug, Clone, PartialEq, Eq)]
// Rationale: Task 248 freezes these caller-owned sites before the Task-259 production caller.
#[cfg_attr(not(test), allow(dead_code))]
pub(in crate::runner) struct SourceTwoParameterDefinitionContextSites {
    pub module: TypedSiteRef,
    pub definition: TypedSiteRef,
    pub parameters: [TypedSiteRef; 2],
}

// Rationale: Task 248 freezes this private lower-stage seam before its Task-259 production caller.
#[cfg_attr(not(test), allow(dead_code))]
pub(in crate::runner) fn source_two_parameter_definition_context_projection(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    definition_node: SurfaceNodeId,
    nodes: &TypedArena,
    sites: SourceTwoParameterDefinitionContextSites,
) -> Result<SourceBindingContextProjection, String> {
    #[cfg(test)]
    {
        source_two_parameter_definition_context_projection_impl(
            ast,
            module,
            shells,
            symbols,
            definition_node,
            nodes,
            sites,
            None,
        )
    }
    #[cfg(not(test))]
    {
        source_two_parameter_definition_context_projection_impl(
            ast,
            module,
            shells,
            symbols,
            definition_node,
            nodes,
            sites,
        )
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::runner) enum SourceTwoParameterDefinitionContextAuthenticationMutation {
    RootMissing,
    RootRecovery,
    RootRange,
    CompilationItemListMissing,
    CompilationItemListEmpty,
    CompilationItemListDuplicated,
    DefinitionKind,
    DefinitionRecovery,
    DefinitionRange,
    DefinitionTokenText { index: usize },
    DefinitionChildMissing,
    DefinitionChildReordered,
    DefinitionChildDuplicated,
    DefinitionChildThird,
    DefinitionChildNonLeading,
    DefinitionChildNonDirectParameter,
    DefinitionChildNestedParameter,
    ParameterNodeId { index: usize },
    ParameterKind { index: usize },
    ParameterRange { index: usize },
    ParameterRecovery { index: usize },
    ParameterLetText { index: usize },
    ParameterLetRange { index: usize },
    ParameterSemicolonText { index: usize },
    ParameterSemicolonRange { index: usize },
    ParameterSegmentMissing { index: usize },
    ParameterSegmentDuplicated { index: usize },
    SegmentKind { index: usize },
    SegmentRecovery { index: usize },
    SegmentChildCardinality { index: usize },
    SegmentNameText { index: usize },
    SegmentNameRange { index: usize },
    SegmentBeText { index: usize },
    SegmentBeRange { index: usize },
    TypeKind { index: usize },
    TypeRange { index: usize },
    TypeChildCardinality { index: usize },
    TypeHeadKind { index: usize },
    TypeHeadRecovery { index: usize },
    TypeHeadRange { index: usize },
    TypeHeadChildCardinality { index: usize },
    TypeTokenText { index: usize },
    TypeTokenRange { index: usize },
    TypeTokenRecovery { index: usize },
    ExtractedTypeRange { index: usize },
    ExtractedTypeSpelling { index: usize },
    ExtractedTypeHead { index: usize },
    ExtractedTypeAttributes { index: usize },
    ConstructedScope,
    ConstructedSourceOrdinal { index: usize },
    ConstructedLocalSpelling { index: usize },
    ConstructedLocalRange { index: usize },
    ConstructedLocalScope { index: usize },
    ConstructedLocalVisibleOrdinal { index: usize },
}

#[cfg(test)]
// Rationale: the test seam preserves the frozen seven-argument caller shape and adds one mutation.
#[allow(clippy::too_many_arguments)]
pub(in crate::runner) fn source_two_parameter_definition_context_projection_with_authentication_mutation(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    definition_node: SurfaceNodeId,
    nodes: &TypedArena,
    sites: SourceTwoParameterDefinitionContextSites,
    mutation: SourceTwoParameterDefinitionContextAuthenticationMutation,
) -> Result<SourceBindingContextProjection, String> {
    source_two_parameter_definition_context_projection_impl(
        ast,
        module,
        shells,
        symbols,
        definition_node,
        nodes,
        sites,
        Some(mutation),
    )
}

// Rationale: test builds add one mutation argument to the frozen seven-argument production seam.
#[cfg_attr(test, allow(clippy::too_many_arguments))]
fn source_two_parameter_definition_context_projection_impl(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    definition_node: SurfaceNodeId,
    nodes: &TypedArena,
    sites: SourceTwoParameterDefinitionContextSites,
    #[cfg(test)] mutation: Option<SourceTwoParameterDefinitionContextAuthenticationMutation>,
) -> Result<SourceBindingContextProjection, String> {
    if symbols.module_id() != &module {
        return Err("two-parameter source binding context uses another symbol module".to_owned());
    }
    test_mutable!(root = ast.root().and_then(|root| ast.node(root)));
    #[cfg(test)]
    if matches!(
        mutation,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::RootMissing)
    ) {
        root = None;
    }
    let root =
        root.ok_or_else(|| "two-parameter source binding context root disappeared".to_owned())?;
    let definition_range = source_range(ast, 0, 164);
    test_mutable!(root_recovered = root.recovered);
    test_mutable!(root_range = root.range);
    #[cfg(test)]
    match mutation {
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::RootRecovery) => {
            root_recovered = true;
        }
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::RootRange) => {
            root_range.start += 1;
        }
        _ => {}
    }
    if root_recovered || root_range != definition_range {
        return Err("two-parameter source binding context root is not exact".to_owned());
    }
    test_mutable!(item_list = exact_compilation_item_list(ast));
    #[cfg(test)]
    if matches!(
        mutation,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::CompilationItemListMissing)
    ) {
        item_list = None;
    }
    let item_list = item_list
        .ok_or_else(|| "two-parameter source binding context item list is not exact".to_owned())?;
    test_mutable!(item_ids = structural_child_ids(ast, item_list));
    #[cfg(test)]
    match mutation {
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::CompilationItemListEmpty) => {
            item_ids.clear();
        }
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::CompilationItemListDuplicated,
        ) if !item_ids.is_empty() => item_ids.push(item_ids[0]),
        _ => {}
    }
    let [item_id] = item_ids.as_slice() else {
        return Err(
            "two-parameter source binding context requires one top-level source item".to_owned(),
        );
    };
    if *item_id != definition_node || definition_node.index() != 67 {
        return Err(
            "two-parameter source binding context definition identity is not exact".to_owned(),
        );
    }
    let definition = ast
        .node(definition_node)
        .ok_or_else(|| "two-parameter source binding context definition disappeared".to_owned())?;
    test_mutable!(
        definition_kind_is_exact = matches!(definition.kind, SurfaceNodeKind::DefinitionBlockItem)
    );
    test_mutable!(definition_recovered = definition.recovered);
    test_mutable!(observed_definition_range = definition.range);
    test_mutable!(definition_tokens = direct_token_texts(ast, definition));
    #[cfg(test)]
    match mutation {
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionKind) => {
            definition_kind_is_exact = false;
        }
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionRecovery) => {
            definition_recovered = true;
        }
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionRange) => {
            observed_definition_range.start += 1;
        }
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionTokenText {
            index,
        }) if index < definition_tokens.len() => definition_tokens[index] = "mutated".to_owned(),
        _ => {}
    }
    if !definition_kind_is_exact
        || definition_recovered
        || observed_definition_range != definition_range
        || definition_tokens.as_slice() != ["definition", "end", ";"]
        || !shells.exports().is_empty()
    {
        return Err("two-parameter source binding context definition item is not exact".to_owned());
    }

    let top_level_shells = shells
        .declarations()
        .iter()
        .filter(|shell| shell.parent().is_none())
        .collect::<Vec<_>>();
    let [definition_shell] = top_level_shells.as_slice() else {
        return Err(
            "two-parameter source binding context requires one top-level declaration shell"
                .to_owned(),
        );
    };
    validate_shell(
        definition_shell,
        0,
        DeclarationShellKind::DefinitionBlock,
        &module,
        definition_node,
        definition,
    )?;

    test_mutable!(definition_children = structural_child_ids(ast, definition));
    #[cfg(test)]
    mutate_two_parameter_definition_children(ast, &mut definition_children, mutation.as_ref());
    if definition_children.len() < 2 {
        return Err(
            "two-parameter source binding context requires two leading parameters".to_owned(),
        );
    }
    if definition_children[2..].iter().any(|id| {
        ast.node(*id)
            .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::DefinitionParameter))
    }) {
        return Err(
            "two-parameter source binding context contains an additional or non-leading parameter"
                .to_owned(),
        );
    }

    let parameter_specs = [
        TwoParameterDefinitionParameterSpec {
            node_index: 41,
            parameter_start: 13,
            parameter_end: 26,
            let_start: 13,
            let_end: 16,
            spelling: "x",
            name_start: 17,
            name_end: 18,
            be_start: 19,
            be_end: 21,
            type_start: 22,
            type_end: 25,
            semicolon_start: 25,
            semicolon_end: 26,
        },
        TwoParameterDefinitionParameterSpec {
            node_index: 45,
            parameter_start: 29,
            parameter_end: 42,
            let_start: 29,
            let_end: 32,
            spelling: "y",
            name_start: 33,
            name_end: 34,
            be_start: 35,
            be_end: 37,
            type_start: 38,
            type_end: 41,
            semicolon_start: 41,
            semicolon_end: 42,
        },
    ];
    let mut parameters = Vec::with_capacity(2);
    for (ordinal, spec) in parameter_specs.iter().enumerate() {
        #[cfg(test)]
        let parameter = extract_two_parameter_definition_parameter(
            ast,
            &module,
            symbols,
            definition_children[ordinal],
            *spec,
            ordinal,
            mutation.as_ref(),
        )?;
        #[cfg(not(test))]
        let parameter = extract_two_parameter_definition_parameter(
            ast,
            &module,
            symbols,
            definition_children[ordinal],
            *spec,
        )?;
        parameters.push(parameter);
    }

    validate_two_parameter_definition_sites(
        nodes,
        &sites,
        definition_range,
        [
            parameters[0].declaration_range,
            parameters[1].declaration_range,
        ],
    )?;

    let shell = definition_shell.id();
    test_mutable!(local_scope = LocalTermScope::new(vec![shell.index() as u32]));
    #[cfg(test)]
    if matches!(
        mutation,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedScope)
    ) {
        local_scope = LocalTermScope::new(vec![1]);
    }
    if local_scope.path() != [0] {
        return Err("two-parameter source binding context scope is not [0]".to_owned());
    }
    let input = SourceBindingContextInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        module_site: sites.module,
        items: vec![SourceItemInput {
            shell,
            shell_ordinal: definition_shell.ordinal(),
            role: SourceItemRole::DefinitionBlock,
            module_id: module,
            source_range: definition_range,
            parent: None,
            visibility: SourceItemVisibility::Unspecified,
            site: sites.definition,
            local_scope: Some(local_scope.clone()),
            recovery: SourceItemRecovery::Normal,
        }],
        bindings: parameters
            .into_iter()
            .zip(sites.parameters)
            .enumerate()
            .map(|(ordinal, (parameter, site))| {
                test_mutable!(source_ordinal = ordinal);
                let spelling = parameter.spelling.to_owned();
                test_mutable!(local_spelling = parameter.spelling);
                test_mutable!(local_range = parameter.declaration_range);
                test_mutable!(local_scope_for_binding = local_scope.clone());
                test_mutable!(local_visible_ordinal = ordinal);
                #[cfg(test)]
                mutate_two_parameter_constructed_binding(
                    ordinal,
                    mutation.as_ref(),
                    &mut source_ordinal,
                    &mut local_spelling,
                    &mut local_range,
                    &mut local_scope_for_binding,
                    &mut local_visible_ordinal,
                );
                SourceBindingSiteInput {
                    shell,
                    context_owner: SourceBindingContextOwner::Shell(shell),
                    source_ordinal,
                    spelling,
                    declaration_range: parameter.declaration_range,
                    written_type_range: parameter.written_type_range,
                    site,
                    role: SourceBindingSiteRole::DefinitionParameter {
                        local: LocalTermBinding::new(
                            local_spelling,
                            local_scope_for_binding,
                            local_range,
                            local_visible_ordinal,
                        ),
                    },
                    recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
                }
            })
            .collect(),
    };
    match SourceBindingContextProducer::build(input).map_err(|error| error.to_string())? {
        SourceBindingContextBuild::Complete(projection) => Ok(projection),
        SourceBindingContextBuild::Incomplete(_) => {
            Err("two-parameter source binding context unexpectedly remained incomplete".to_owned())
        }
        _ => Err(
            "two-parameter source binding context returned an unsupported build state".to_owned(),
        ),
    }
}

#[cfg(test)]
fn mutate_two_parameter_definition_children(
    ast: &SurfaceAst,
    children: &mut Vec<SurfaceNodeId>,
    mutation: Option<&SourceTwoParameterDefinitionContextAuthenticationMutation>,
) {
    match mutation {
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildMissing) => {
            children.truncate(1);
        }
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildReordered,
        ) if children.len() >= 2 => {
            children.swap(0, 1);
        }
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildDuplicated,
        ) if children.len() >= 2 => {
            children[1] = children[0];
        }
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildThird)
            if !children.is_empty() =>
        {
            children.push(children[0]);
        }
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildNonLeading,
        ) if children.len() >= 3 => {
            children.swap(0, 2);
        }
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildNonDirectParameter,
        ) if !children.is_empty() => {
            if let Some(descendant) = ast
                .node(children[0])
                .and_then(|parameter| structural_child_ids(ast, parameter).into_iter().next())
            {
                children[0] = descendant;
            }
        }
        _ => {}
    }
}

#[cfg(test)]
fn mutate_two_parameter_constructed_binding(
    ordinal: usize,
    mutation: Option<&SourceTwoParameterDefinitionContextAuthenticationMutation>,
    source_ordinal: &mut usize,
    local_spelling: &mut &'static str,
    local_range: &mut SourceRange,
    local_scope: &mut LocalTermScope,
    local_visible_ordinal: &mut usize,
) {
    match mutation {
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedSourceOrdinal {
                index,
            },
        ) if *index == ordinal => *source_ordinal += 10,
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedLocalSpelling {
                index,
            },
        ) if *index == ordinal => *local_spelling = "mutated",
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedLocalRange {
                index,
            },
        ) if *index == ordinal => local_range.start += 1,
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedLocalScope {
                index,
            },
        ) if *index == ordinal => *local_scope = LocalTermScope::new(vec![9]),
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ConstructedLocalVisibleOrdinal {
                index,
            },
        ) if *index == ordinal => *local_visible_ordinal += 10,
        _ => {}
    }
}

#[derive(Debug, Clone, Copy)]
// Rationale: Task 248 freezes this exact source-shape descriptor before Task 259 calls it.
#[cfg_attr(not(test), allow(dead_code))]
struct TwoParameterDefinitionParameterSpec {
    node_index: usize,
    parameter_start: usize,
    parameter_end: usize,
    let_start: usize,
    let_end: usize,
    spelling: &'static str,
    name_start: usize,
    name_end: usize,
    be_start: usize,
    be_end: usize,
    type_start: usize,
    type_end: usize,
    semicolon_start: usize,
    semicolon_end: usize,
}

// Rationale: Task 248 keeps this authenticated parameter transport dormant until Task 259.
#[cfg_attr(not(test), allow(dead_code))]
struct TwoParameterDefinitionParameter {
    spelling: &'static str,
    declaration_range: SourceRange,
    written_type_range: SourceRange,
}

// Rationale: Task 248 freezes this private parameter extractor before its Task-259 caller.
#[cfg_attr(not(test), allow(dead_code))]
fn extract_two_parameter_definition_parameter(
    ast: &SurfaceAst,
    module: &ModuleId,
    symbols: &SymbolEnv,
    parameter_id: SurfaceNodeId,
    spec: TwoParameterDefinitionParameterSpec,
    #[cfg(test)] ordinal: usize,
    #[cfg(test)] mutation: Option<&SourceTwoParameterDefinitionContextAuthenticationMutation>,
) -> Result<TwoParameterDefinitionParameter, String> {
    let parameter = ast.node(parameter_id).ok_or_else(|| {
        format!(
            "two-parameter source binding context parameter {} disappeared",
            spec.spelling
        )
    })?;
    test_mutable!(parameter_node_index = parameter_id.index());
    test_mutable!(
        parameter_kind_is_exact = matches!(parameter.kind, SurfaceNodeKind::DefinitionParameter)
    );
    test_mutable!(parameter_range = parameter.range);
    test_mutable!(parameter_has_recovery = subtree_has_recovery(ast, parameter));
    test_mutable!(parameter_tokens = direct_token_texts(ast, parameter));
    test_mutable!(let_range = token_range(ast, parameter, "let")?);
    test_mutable!(semicolon_range = token_range(ast, parameter, ";")?);
    #[cfg(test)]
    match mutation {
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterNodeId {
            index,
        }) if *index == ordinal => parameter_node_index += 1,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterKind {
            index,
        }) if *index == ordinal => parameter_kind_is_exact = false,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterRange {
            index,
        }) if *index == ordinal => parameter_range.start += 1,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterRecovery {
            index,
        }) if *index == ordinal => parameter_has_recovery = true,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterLetText {
            index,
        }) if *index == ordinal => parameter_tokens[0] = "LET".to_owned(),
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterLetRange {
            index,
        }) if *index == ordinal => let_range.start += 1,
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterSemicolonText {
                index,
            },
        ) if *index == ordinal => parameter_tokens[1] = ".".to_owned(),
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterSemicolonRange {
                index,
            },
        ) if *index == ordinal => semicolon_range.start += 1,
        _ => {}
    }
    if parameter_node_index != spec.node_index
        || !parameter_kind_is_exact
        || parameter_range != source_range(ast, spec.parameter_start, spec.parameter_end)
        || parameter_has_recovery
        || parameter_tokens.as_slice() != ["let", ";"]
        || let_range != source_range(ast, spec.let_start, spec.let_end)
        || semicolon_range != source_range(ast, spec.semicolon_start, spec.semicolon_end)
    {
        return Err(format!(
            "two-parameter source binding context parameter {} is not exact",
            spec.spelling
        ));
    }
    test_mutable!(parameter_children = structural_child_ids(ast, parameter));
    #[cfg(test)]
    match mutation {
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::DefinitionChildNestedParameter,
        ) if ordinal == 0 => parameter_children.push(parameter_id),
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterSegmentMissing {
                index,
            },
        ) if *index == ordinal => parameter_children.clear(),
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ParameterSegmentDuplicated {
                index,
            },
        ) if *index == ordinal && !parameter_children.is_empty() => {
            parameter_children.push(parameter_children[0]);
        }
        _ => {}
    }
    let [segment_id] = parameter_children.as_slice() else {
        return Err(format!(
            "two-parameter source binding context parameter {} requires one segment",
            spec.spelling
        ));
    };
    let segment = ast.node(*segment_id).ok_or_else(|| {
        format!(
            "two-parameter source binding context segment {} disappeared",
            spec.spelling
        )
    })?;
    test_mutable!(
        segment_kind_is_exact = matches!(segment.kind, SurfaceNodeKind::QualifiedVariableSegment)
    );
    test_mutable!(segment_has_recovery = subtree_has_recovery(ast, segment));
    test_mutable!(segment_tokens = direct_token_texts(ast, segment));
    test_mutable!(name_range = token_range(ast, segment, spec.spelling)?);
    test_mutable!(be_range = token_range(ast, segment, "be")?);
    #[cfg(test)]
    match mutation {
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentKind { index })
            if *index == ordinal =>
        {
            segment_kind_is_exact = false;
        }
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentRecovery {
            index,
        }) if *index == ordinal => segment_has_recovery = true,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentNameText {
            index,
        }) if *index == ordinal => segment_tokens[0] = "mutated".to_owned(),
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentNameRange {
            index,
        }) if *index == ordinal => name_range.start += 1,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentBeText {
            index,
        }) if *index == ordinal => segment_tokens[1] = "BE".to_owned(),
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentBeRange {
            index,
        }) if *index == ordinal => be_range.start += 1,
        _ => {}
    }
    if !segment_kind_is_exact
        || segment_has_recovery
        || segment_tokens.as_slice() != [spec.spelling, "be"]
        || name_range != source_range(ast, spec.name_start, spec.name_end)
        || be_range != source_range(ast, spec.be_start, spec.be_end)
    {
        return Err(format!(
            "two-parameter source binding context segment {} is not exact",
            spec.spelling
        ));
    }
    test_mutable!(segment_children = structural_child_ids(ast, segment));
    #[cfg(test)]
    if matches!(
        mutation,
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::SegmentChildCardinality {
                index
            }
        ) if *index == ordinal
    ) {
        segment_children.push(segment_children[0]);
    }
    let [type_id] = segment_children.as_slice() else {
        return Err(format!(
            "two-parameter source binding context parameter {} requires one written type",
            spec.spelling
        ));
    };
    let type_node = ast.node(*type_id).ok_or_else(|| {
        format!(
            "two-parameter source binding context type {} disappeared",
            spec.spelling
        )
    })?;
    let expected_type_range = source_range(ast, spec.type_start, spec.type_end);
    test_mutable!(type_kind_is_exact = matches!(type_node.kind, SurfaceNodeKind::TypeExpression));
    test_mutable!(type_range = type_node.range);
    test_mutable!(type_children = type_node.children.clone());
    #[cfg(test)]
    match mutation {
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::TypeKind { index })
            if *index == ordinal =>
        {
            type_kind_is_exact = false;
        }
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::TypeRange { index })
            if *index == ordinal =>
        {
            type_range.start += 1;
        }
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::TypeChildCardinality {
            index,
        }) if *index == ordinal => type_children.push(type_children[0]),
        _ => {}
    }
    if !type_kind_is_exact || type_range != expected_type_range || type_children.len() != 1 {
        return Err(format!(
            "two-parameter source binding context type {} has the wrong shape",
            spec.spelling
        ));
    }
    let type_head = ast
        .node(type_children[0])
        .ok_or_else(|| "two-parameter source binding context type head disappeared".to_owned())?;
    test_mutable!(type_head_kind_is_exact = matches!(type_head.kind, SurfaceNodeKind::TypeHead));
    test_mutable!(type_head_recovered = type_head.recovered);
    test_mutable!(type_head_range = type_head.range);
    test_mutable!(type_head_children = type_head.children.clone());
    #[cfg(test)]
    match mutation {
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::TypeHeadKind { index })
            if *index == ordinal =>
        {
            type_head_kind_is_exact = false;
        }
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::TypeHeadRecovery {
            index,
        }) if *index == ordinal => type_head_recovered = true,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::TypeHeadRange {
            index,
        }) if *index == ordinal => {
            type_head_range.start += 1;
        }
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::TypeHeadChildCardinality {
                index,
            },
        ) if *index == ordinal => type_head_children.push(type_head_children[0]),
        _ => {}
    }
    let [type_token_id] = type_head_children.as_slice() else {
        return Err("two-parameter source binding context type head is not bare".to_owned());
    };
    let type_token = ast
        .node(*type_token_id)
        .ok_or_else(|| "two-parameter source binding context type token disappeared".to_owned())?;
    test_mutable!(type_token_recovered = type_token.recovered);
    test_mutable!(type_token_range = type_token.range);
    test_mutable!(type_token_text = type_token.token_text());
    #[cfg(test)]
    match mutation {
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::TypeTokenText {
            index,
        }) if *index == ordinal => type_token_text = Some("object"),
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::TypeTokenRange {
            index,
        }) if *index == ordinal => type_token_range.start += 1,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::TypeTokenRecovery {
            index,
        }) if *index == ordinal => type_token_recovered = true,
        _ => {}
    }
    if !type_head_kind_is_exact
        || type_head_recovered
        || type_head_range != expected_type_range
        || type_token_recovered
        || type_token_range != expected_type_range
        || type_token_text != Some("set")
    {
        return Err("two-parameter source binding context type is not bare set".to_owned());
    }
    let written_type = extract_builtin_source_type_expression(ast, type_node, module, symbols)
        .map_err(|()| {
            format!(
                "two-parameter source binding context type {} extraction failed",
                spec.spelling
            )
        })?;
    test_mutable!(extracted_range_is_exact = written_type.range == expected_type_range);
    test_mutable!(extracted_spelling_is_exact = written_type.spelling == "set");
    test_mutable!(extracted_head_is_exact = written_type.head == TypeHeadInput::BuiltinSet);
    test_mutable!(extracted_attributes_are_empty = written_type.attributes.is_empty());
    #[cfg(test)]
    match mutation {
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::ExtractedTypeRange {
            index,
        }) if *index == ordinal => extracted_range_is_exact = false,
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ExtractedTypeSpelling {
                index,
            },
        ) if *index == ordinal => extracted_spelling_is_exact = false,
        Some(SourceTwoParameterDefinitionContextAuthenticationMutation::ExtractedTypeHead {
            index,
        }) if *index == ordinal => extracted_head_is_exact = false,
        Some(
            SourceTwoParameterDefinitionContextAuthenticationMutation::ExtractedTypeAttributes {
                index,
            },
        ) if *index == ordinal => extracted_attributes_are_empty = false,
        _ => {}
    }
    if !extracted_range_is_exact
        || !extracted_spelling_is_exact
        || !extracted_head_is_exact
        || !extracted_attributes_are_empty
    {
        return Err(format!(
            "two-parameter source binding context type {} is not builtin set",
            spec.spelling
        ));
    }
    Ok(TwoParameterDefinitionParameter {
        spelling: spec.spelling,
        declaration_range: source_range(ast, spec.name_start, spec.name_end),
        written_type_range: expected_type_range,
    })
}

// Rationale: Task 248 validates the future caller-owned shared arena before Task 259 uses it.
#[cfg_attr(not(test), allow(dead_code))]
fn validate_two_parameter_definition_sites(
    nodes: &TypedArena,
    sites: &SourceTwoParameterDefinitionContextSites,
    definition_range: SourceRange,
    parameter_ranges: [SourceRange; 2],
) -> Result<(), String> {
    let all_sites = [
        &sites.module,
        &sites.definition,
        &sites.parameters[0],
        &sites.parameters[1],
    ];
    if all_sites
        .iter()
        .map(|site| (*site).clone())
        .collect::<BTreeSet<_>>()
        .len()
        != all_sites.len()
    {
        return Err("two-parameter source binding context sites are not distinct".to_owned());
    }
    let root = nodes
        .root()
        .ok_or_else(|| "two-parameter source binding context arena has no root".to_owned())?;
    if sites.module != TypedSiteRef::Node(root) {
        return Err(
            "two-parameter source binding context module site is not the arena root".to_owned(),
        );
    }
    validate_two_parameter_definition_site(
        nodes,
        &sites.module,
        definition_range,
        LocalTypeContextId::new(0),
        "module",
    )?;
    validate_two_parameter_definition_site(
        nodes,
        &sites.definition,
        definition_range,
        LocalTypeContextId::new(1),
        "definition",
    )?;
    for (index, (site, range)) in sites.parameters.iter().zip(parameter_ranges).enumerate() {
        validate_two_parameter_definition_site(
            nodes,
            site,
            range,
            LocalTypeContextId::new(1),
            &format!("parameter {index}"),
        )?;
    }
    Ok(())
}

// Rationale: Task 248 keeps per-site validation dormant until the Task-259 production caller.
#[cfg_attr(not(test), allow(dead_code))]
fn validate_two_parameter_definition_site(
    nodes: &TypedArena,
    site: &TypedSiteRef,
    expected_range: SourceRange,
    expected_context: LocalTypeContextId,
    role: &str,
) -> Result<(), String> {
    let node = nodes.node(site.node()).ok_or_else(|| {
        format!("two-parameter source binding context {role} site does not resolve")
    })?;
    if node.anchor != SourceAnchor::Range(expected_range)
        || node.links.context != Some(expected_context)
        || node.recovery != NodeRecoveryState::Normal
    {
        return Err(format!(
            "two-parameter source binding context {role} site is not exact"
        ));
    }
    Ok(())
}

// Rationale: Task 248 uses this exact-source range helper only in its dormant lower extractor.
#[cfg_attr(not(test), allow(dead_code))]
fn source_range(ast: &SurfaceAst, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id: ast.source_id,
        start,
        end,
    }
}

struct CandidateItems<'a> {
    reserve_id: SurfaceNodeId,
    reserve_item: &'a SurfaceNode,
    definition_id: SurfaceNodeId,
    definition_item: &'a SurfaceNode,
}

fn candidate_items(ast: &SurfaceAst) -> Option<CandidateItems<'_>> {
    let item_list = exact_compilation_item_list(ast)?;
    let item_ids = structural_child_ids(ast, item_list);
    let [reserve_id, definition_id] = item_ids.as_slice() else {
        return None;
    };
    let reserve = ast.node(*reserve_id)?;
    let definition = ast.node(*definition_id)?;
    if !matches!(reserve.kind, SurfaceNodeKind::ReserveItem)
        || !matches!(definition.kind, SurfaceNodeKind::DefinitionBlockItem)
        || subtree_has_recovery(ast, reserve)
        || subtree_has_recovery(ast, definition)
        || !source_binding_context_token_shape_is_exact(
            &subtree_token_texts(ast, reserve),
            &subtree_token_texts(ast, definition),
        )
    {
        return None;
    }
    let definition_children = structural_child_ids(ast, definition);
    let [parameter_id] = definition_children.as_slice() else {
        return None;
    };
    if ast
        .node(*parameter_id)
        .is_none_or(|node| !matches!(node.kind, SurfaceNodeKind::DefinitionParameter))
    {
        return None;
    }
    Some(CandidateItems {
        reserve_id: *reserve_id,
        reserve_item: reserve,
        definition_id: *definition_id,
        definition_item: definition,
    })
}

pub(in crate::runner) fn source_binding_context_token_shape_is_exact(
    reserve_tokens: &[&str],
    definition_tokens: &[&str],
) -> bool {
    reserve_tokens == ["reserve", "x", "for", "set", ";"]
        && definition_tokens == ["definition", "let", "x", "be", "set", ";", "end", ";"]
}

fn build_output(
    ast: &SurfaceAst,
    module: ModuleId,
    shells: &DeclarationShellSet,
    symbols: &SymbolEnv,
    candidate: CandidateItems<'_>,
) -> Result<SourceBindingContextRouteOutput, String> {
    let CandidateItems {
        reserve_id,
        reserve_item,
        definition_id,
        definition_item,
    } = candidate;
    if subtree_has_recovery(ast, reserve_item)
        || subtree_has_recovery(ast, definition_item)
        || direct_token_texts(ast, reserve_item).as_slice() != ["reserve", ";"]
        || direct_token_texts(ast, definition_item).as_slice() != ["definition", "end", ";"]
        || !shells.exports().is_empty()
    {
        return Err("source binding context has a recovered or non-exact item shell".to_owned());
    }
    let [reserve_shell, definition_shell] = shells.declarations() else {
        return Err("source binding context requires exactly two declaration shells".to_owned());
    };
    validate_shell(
        reserve_shell,
        0,
        DeclarationShellKind::Reserve,
        &module,
        reserve_id,
        reserve_item,
    )?;
    validate_shell(
        definition_shell,
        1,
        DeclarationShellKind::DefinitionBlock,
        &module,
        definition_id,
        definition_item,
    )?;

    let reserve =
        extract_builtin_source_reserve_declarations_after_node_guard(ast, module.clone(), symbols)
            .map_err(|()| "source binding context reserve extraction failed".to_owned())?;
    let [reserve_binding] = reserve.bridge.bindings() else {
        return Err("source binding context requires one reserve binding".to_owned());
    };
    if reserve.bridge.source_range() != reserve_item.range
        || reserve_binding.spelling != "x"
        || reserve_binding.type_spelling != "set"
        || reserve_binding.type_head != TypeHeadInput::BuiltinSet
        || !reserve_binding.type_attributes.is_empty()
    {
        return Err("source binding context reserve payload is not exact".to_owned());
    }

    let definition_children = structural_child_ids(ast, definition_item);
    let [parameter_id] = definition_children.as_slice() else {
        return Err("source binding context definition requires one parameter".to_owned());
    };
    let parameter = ast
        .node(*parameter_id)
        .ok_or_else(|| "source binding context parameter disappeared".to_owned())?;
    if subtree_has_recovery(ast, parameter)
        || direct_token_texts(ast, parameter).as_slice() != ["let", ";"]
    {
        return Err("source binding context parameter is not exact".to_owned());
    }
    let parameter_children = structural_child_ids(ast, parameter);
    let [segment_id] = parameter_children.as_slice() else {
        return Err("source binding context requires one qualified-variable segment".to_owned());
    };
    let segment = ast
        .node(*segment_id)
        .ok_or_else(|| "source binding context variable segment disappeared".to_owned())?;
    if !matches!(segment.kind, SurfaceNodeKind::QualifiedVariableSegment)
        || subtree_has_recovery(ast, segment)
        || direct_token_texts(ast, segment).as_slice() != ["x", "be"]
    {
        return Err("source binding context variable segment is not exact".to_owned());
    }
    let segment_children = structural_child_ids(ast, segment);
    let [type_id] = segment_children.as_slice() else {
        return Err("source binding context parameter requires one written type".to_owned());
    };
    let type_node = ast
        .node(*type_id)
        .ok_or_else(|| "source binding context written type disappeared".to_owned())?;
    if !matches!(type_node.kind, SurfaceNodeKind::TypeExpression) {
        return Err("source binding context parameter type has the wrong shape".to_owned());
    }
    let written_type = extract_builtin_source_type_expression(ast, type_node, &module, symbols)
        .map_err(|()| "source binding context parameter type extraction failed".to_owned())?;
    if written_type.spelling != "set"
        || written_type.head != TypeHeadInput::BuiltinSet
        || !written_type.attributes.is_empty()
    {
        return Err("source binding context parameter type is not builtin set".to_owned());
    }
    let declaration_range = token_range(ast, segment, "x")?;
    let root_range = ast
        .root()
        .and_then(|root| ast.node(root))
        .map(|root| root.range)
        .ok_or_else(|| "source binding context root disappeared".to_owned())?;
    let reserve_shell_id = reserve_shell.id();
    let definition_shell_id = definition_shell.id();
    let local_scope = LocalTermScope::new(vec![definition_shell.id().index() as u32]);
    let input = SourceBindingContextInput {
        source_id: ast.source_id,
        module_id: module.clone(),
        module_site: TypedSiteRef::Node(TypedNodeId::new(4)),
        items: vec![
            SourceItemInput {
                shell: reserve_shell_id,
                shell_ordinal: reserve_shell.ordinal(),
                role: SourceItemRole::Reserve,
                module_id: module.clone(),
                source_range: reserve_item.range,
                parent: None,
                visibility: SourceItemVisibility::Unspecified,
                site: TypedSiteRef::Node(TypedNodeId::new(1)),
                local_scope: None,
                recovery: SourceItemRecovery::Normal,
            },
            SourceItemInput {
                shell: definition_shell_id,
                shell_ordinal: definition_shell.ordinal(),
                role: SourceItemRole::DefinitionBlock,
                module_id: module.clone(),
                source_range: definition_item.range,
                parent: None,
                visibility: SourceItemVisibility::Unspecified,
                site: TypedSiteRef::Node(TypedNodeId::new(3)),
                local_scope: Some(local_scope.clone()),
                recovery: SourceItemRecovery::Normal,
            },
        ],
        bindings: vec![
            SourceBindingSiteInput {
                shell: reserve_shell_id,
                context_owner: SourceBindingContextOwner::Module,
                source_ordinal: 0,
                spelling: reserve_binding.spelling.clone(),
                declaration_range: reserve_binding.binding_range,
                written_type_range: reserve_binding.type_range,
                site: TypedSiteRef::Node(TypedNodeId::new(0)),
                role: SourceBindingSiteRole::ReserveDefault,
                recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
            },
            SourceBindingSiteInput {
                shell: definition_shell_id,
                context_owner: SourceBindingContextOwner::Shell(definition_shell_id),
                source_ordinal: 1,
                spelling: "x".to_owned(),
                declaration_range,
                written_type_range: written_type.range,
                site: TypedSiteRef::Node(TypedNodeId::new(2)),
                role: SourceBindingSiteRole::DefinitionParameter {
                    local: LocalTermBinding::new("x", local_scope, declaration_range, 1),
                },
                recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
            },
        ],
    };
    let projection = match SourceBindingContextProducer::build(input)
        .map_err(|error| error.to_string())?
    {
        SourceBindingContextBuild::Complete(projection) => projection,
        SourceBindingContextBuild::Incomplete(_) => {
            return Err("source binding context unexpectedly remained incomplete".to_owned());
        }
        _ => return Err("source binding context returned an unsupported build state".to_owned()),
    };
    assemble_output(
        ast.source_id,
        module,
        SourceNodeRanges {
            root: root_range,
            reserve_item: reserve_item.range,
            reserve_binding: reserve_binding.binding_range,
            reserve_type: reserve_binding.type_range,
            definition_item: definition_item.range,
            definition_binding: declaration_range,
            definition_type: written_type.range,
        },
        projection,
        symbols,
    )
}

fn validate_shell(
    shell: &DeclarationShell,
    ordinal: usize,
    kind: DeclarationShellKind,
    module: &ModuleId,
    node_id: SurfaceNodeId,
    node: &SurfaceNode,
) -> Result<(), String> {
    if shell.ordinal() != ordinal
        || shell.id().index() != ordinal
        || shell.kind() != kind
        || shell.module() != module
        || shell.node_id() != node_id
        || shell.syntax_kind() != node.kind.syntax_kind()
        || shell.range() != node.range
        || shell.parent().is_some()
        || shell.visibility().state() != DeclarationShellVisibilityState::Unspecified
        || shell.visibility().marker_range().is_some()
        || shell.visibility().spelling().is_some()
        || shell.recovered()
    {
        return Err(format!(
            "source binding context shell {ordinal} is inconsistent"
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(in crate::runner) fn validate_source_context_shell_for_test(
    shell: &DeclarationShell,
    ordinal: usize,
    kind: DeclarationShellKind,
    module: &ModuleId,
    node_id: SurfaceNodeId,
    node: &SurfaceNode,
) -> Result<(), String> {
    validate_shell(shell, ordinal, kind, module, node_id, node)
}

fn token_range(
    ast: &SurfaceAst,
    node: &SurfaceNode,
    spelling: &str,
) -> Result<SourceRange, String> {
    let matches = node
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter(|child| child.token_text() == Some(spelling))
        .map(|child| child.range)
        .collect::<Vec<_>>();
    let [range] = matches.as_slice() else {
        return Err(format!(
            "source binding context requires one `{spelling}` token"
        ));
    };
    Ok(*range)
}

fn subtree_token_texts<'a>(ast: &'a SurfaceAst, node: &'a SurfaceNode) -> Vec<&'a str> {
    let mut tokens = Vec::new();
    collect_subtree_token_texts(ast, node, &mut tokens);
    tokens
}

fn collect_subtree_token_texts<'a>(
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
            collect_subtree_token_texts(ast, child, tokens);
        }
    }
}

struct SourceNodeRanges {
    root: SourceRange,
    reserve_item: SourceRange,
    reserve_binding: SourceRange,
    reserve_type: SourceRange,
    definition_item: SourceRange,
    definition_binding: SourceRange,
    definition_type: SourceRange,
}

fn assemble_output(
    source_id: mizar_session::SourceId,
    module: ModuleId,
    ranges: SourceNodeRanges,
    projection: SourceBindingContextProjection,
    symbols: &SymbolEnv,
) -> Result<SourceBindingContextRouteOutput, String> {
    let expected_handoff = projection.handoff().clone();
    let source_context = projection.into_handoff();
    let contexts = source_context.local_contexts().clone();
    let mut builder = TypedArenaBuilder::new();
    push_node(
        &mut builder,
        "source.reserve.binding",
        ranges.reserve_binding,
        0,
        Vec::new(),
    )?;
    push_node(
        &mut builder,
        "source.reserve",
        ranges.reserve_item,
        0,
        vec![TypedNodeId::new(0)],
    )?;
    push_node(
        &mut builder,
        "source.definition.parameter",
        ranges.definition_binding,
        1,
        Vec::new(),
    )?;
    push_node(
        &mut builder,
        "source.definition",
        ranges.definition_item,
        1,
        vec![TypedNodeId::new(2)],
    )?;
    push_node(
        &mut builder,
        "source.module",
        ranges.root,
        0,
        vec![TypedNodeId::new(1), TypedNodeId::new(3)],
    )?;
    let nodes = builder
        .finish(Some(TypedNodeId::new(4)))
        .map_err(|error| error.to_string())?;
    let source_type = SourceTypeProducer::build(
        SourceTypeHandoffInput {
            source_id,
            module_id: module.clone(),
            applications: vec![
                SourceTypeApplicationInput {
                    binding: mizar_checker::binding_env::BindingId::new(0),
                    source_ordinal: 0,
                    root: SourceTypeExpressionId::new(0),
                },
                SourceTypeApplicationInput {
                    binding: mizar_checker::binding_env::BindingId::new(1),
                    source_ordinal: 1,
                    root: SourceTypeExpressionId::new(1),
                },
            ],
            expressions: vec![
                SourceTypeExpressionInput {
                    source_id,
                    module_id: module.clone(),
                    site: TypedSiteRef::Role {
                        node: TypedNodeId::new(1),
                        role: TypeRole::new("source.type.expression"),
                    },
                    source_range: ranges.reserve_type,
                    spelling: "set".to_owned(),
                    head_site: TypedSiteRef::Role {
                        node: TypedNodeId::new(1),
                        role: TypeRole::new("source.type.head"),
                    },
                    head_range: ranges.reserve_type,
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                },
                SourceTypeExpressionInput {
                    source_id,
                    module_id: module.clone(),
                    site: TypedSiteRef::Role {
                        node: TypedNodeId::new(3),
                        role: TypeRole::new("source.type.expression"),
                    },
                    source_range: ranges.definition_type,
                    spelling: "set".to_owned(),
                    head_site: TypedSiteRef::Role {
                        node: TypedNodeId::new(3),
                        role: TypeRole::new("source.type.head"),
                    },
                    head_range: ranges.definition_type,
                    head_spelling: "set".to_owned(),
                    form: SourceTypeApplicationForm::Bare,
                    head: SourceTypeHead::BuiltinSet,
                    recovery: NodeRecoveryState::Normal,
                },
            ],
            arguments: Vec::new(),
        },
        source_context.binding_env(),
        symbols,
        &nodes,
    )
    .map_err(|error| error.to_string())?;
    let typed_ast = TypedAst::try_new(TypedAstParts {
        source_id,
        module_id: module,
        resolved_root: None,
        source_context: Some(source_context),
        source_type: Some(source_type),
        source_attribute: None,
        nodes,
        contexts,
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .map_err(|error| error.to_string())?;
    let node_hints = [
        (0, "source.reserve.binding"),
        (1, "source.reserve"),
        (2, "source.definition.parameter"),
        (3, "source.definition"),
        (4, "source.module"),
    ]
    .into_iter()
    .map(|(node, role)| ResolvedNodeKindHint {
        typed_node: TypedNodeId::new(node),
        kind: ResolvedNodeKindHintKind::SourcePreserved {
            role: SourceNodeRole::new(role),
        },
    })
    .collect();
    let resolved = assemble_empty_resolved_typed_ast(&typed_ast, node_hints)?;
    if typed_ast.source_context() != Some(&expected_handoff)
        || resolved.source_context() != typed_ast.source_context()
        || typed_ast.source_type().is_none()
        || resolved.source_type() != typed_ast.source_type()
        || typed_ast.source_type().is_none_or(|handoff| {
            handoff.applications().len() != 2
                || handoff.expressions().len() != 2
                || !handoff.arguments().is_empty()
        })
        || !typed_ast.types().is_empty()
        || !typed_ast.facts().is_empty()
        || !typed_ast.coercions().is_empty()
        || !typed_ast.initial_obligations().is_empty()
        || !typed_ast.diagnostics().is_empty()
        || !typed_ast.debug_text().contains("shadowed=0")
        || typed_ast.debug_text().contains("normalized-types")
    {
        return Err("source binding context final handoff invariant failed".to_owned());
    }
    Ok(SourceBindingContextRouteOutput {
        typed_ast,
        resolved,
    })
}

fn push_node(
    builder: &mut TypedArenaBuilder,
    kind: &str,
    range: SourceRange,
    context: usize,
    children: Vec<TypedNodeId>,
) -> Result<TypedNodeId, String> {
    builder
        .push(
            TypedNode::new(kind, SourceAnchor::Range(range))
                .with_children(children)
                .with_typing(TypingState::Unknown)
                .with_recovery(NodeRecoveryState::Normal)
                .with_links(TypedNodeLinks {
                    context: Some(LocalTypeContextId::new(context)),
                    ..TypedNodeLinks::default()
                }),
        )
        .map_err(|error| error.to_string())
}
