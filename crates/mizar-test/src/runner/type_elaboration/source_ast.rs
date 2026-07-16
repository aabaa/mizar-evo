use mizar_checker::typed_ast::{TypedNodeId, TypedSiteRef};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind};

pub(in crate::runner) fn exact_compilation_item_list(ast: &SurfaceAst) -> Option<&SurfaceNode> {
    let root = ast.node(ast.root()?)?;
    if !matches!(root.kind, SurfaceNodeKind::Root) {
        return None;
    }
    let root_children = structural_child_ids(ast, root);
    let [compilation_unit_id] = root_children.as_slice() else {
        return None;
    };
    let compilation_unit = ast.node(*compilation_unit_id)?;
    if !matches!(compilation_unit.kind, SurfaceNodeKind::CompilationUnit) {
        return None;
    }
    let compilation_children = structural_child_ids(ast, compilation_unit);
    let [item_list_id] = compilation_children.as_slice() else {
        return None;
    };
    let item_list = ast.node(*item_list_id)?;
    if matches!(item_list.kind, SurfaceNodeKind::ItemList) {
        Some(item_list)
    } else {
        None
    }
}

pub(in crate::runner) fn structural_child_ids(
    ast: &SurfaceAst,
    node: &SurfaceNode,
) -> Vec<SurfaceNodeId> {
    node.children
        .iter()
        .copied()
        .filter(|child| {
            ast.node(*child)
                .is_some_and(|child_node| !matches!(child_node.kind, SurfaceNodeKind::Token(_)))
        })
        .collect()
}

pub(in crate::runner) fn direct_token_texts(ast: &SurfaceAst, node: &SurfaceNode) -> Vec<String> {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter_map(SurfaceNode::token_text)
        .map(str::to_owned)
        .collect()
}

pub(in crate::runner) fn surface_site(id: SurfaceNodeId) -> TypedSiteRef {
    TypedSiteRef::Node(TypedNodeId::new(id.index()))
}

pub(in crate::runner) fn subtree_has_recovery(ast: &SurfaceAst, node: &SurfaceNode) -> bool {
    node.recovered
        || node
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .any(|child| subtree_has_recovery(ast, child))
}
