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
    type_checker::TypeHeadInput,
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextId, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArenaBuilder, TypedAst, TypedAstParts,
        TypedNode, TypedNodeId, TypedNodeLinks, TypedSiteRef, TypingState,
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
            definition_item: definition_item.range,
            definition_binding: declaration_range,
        },
        projection,
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
    definition_item: SourceRange,
    definition_binding: SourceRange,
}

fn assemble_output(
    source_id: mizar_session::SourceId,
    module: ModuleId,
    ranges: SourceNodeRanges,
    projection: SourceBindingContextProjection,
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
    let typed_ast = TypedAst::try_new(TypedAstParts {
        source_id,
        module_id: module,
        resolved_root: None,
        source_context: Some(source_context),
        nodes: builder
            .finish(Some(TypedNodeId::new(4)))
            .map_err(|error| error.to_string())?,
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
