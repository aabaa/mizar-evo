use mizar_checker::typed_ast::{TypedNodeId, TypedSiteRef};
use mizar_syntax::{SurfaceAst, SurfaceNode, SurfaceNodeId, SurfaceNodeKind};

use crate::runner::import_fixtures::module_path_spelling;

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

pub(in crate::runner) fn surface_nodes_with_kind(
    ast: &SurfaceAst,
    kind: SurfaceNodeKind,
) -> Vec<(SurfaceNodeId, &SurfaceNode)> {
    let mut output = Vec::new();
    if let Some(root) = ast.root() {
        collect_surface_nodes_with_kind(ast, root, &kind, &mut output);
    }
    output
}

fn collect_surface_nodes_with_kind<'a>(
    ast: &'a SurfaceAst,
    id: SurfaceNodeId,
    kind: &SurfaceNodeKind,
    output: &mut Vec<(SurfaceNodeId, &'a SurfaceNode)>,
) {
    let Some(node) = ast.node(id) else {
        return;
    };
    if &node.kind == kind {
        output.push((id, node));
    }
    for child in &node.children {
        collect_surface_nodes_with_kind(ast, *child, kind, output);
    }
}

pub(in crate::runner) fn direct_token_texts(ast: &SurfaceAst, node: &SurfaceNode) -> Vec<String> {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter_map(SurfaceNode::token_text)
        .map(str::to_owned)
        .collect()
}

pub(in crate::runner) fn qualified_symbol_spelling(
    ast: &SurfaceAst,
    node: &SurfaceNode,
) -> Result<String, ()> {
    if !matches!(node.kind, SurfaceNodeKind::QualifiedSymbol) || node.children.is_empty() {
        return Err(());
    }
    let mut segments = Vec::new();
    for child_id in &node.children {
        let child = ast.node(*child_id).ok_or(())?;
        if !matches!(child.kind, SurfaceNodeKind::PathSegment) || child.children.len() != 1 {
            return Err(());
        }
        let token = ast
            .node(child.children[0])
            .and_then(SurfaceNode::token_text)
            .ok_or(())?;
        segments.push(token.to_owned());
    }
    Ok(segments.join("."))
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

pub(in crate::runner) fn is_exact_parser_type_fixtures_import(
    ast: &SurfaceAst,
    node: &SurfaceNode,
) -> bool {
    if !matches!(node.kind, SurfaceNodeKind::ImportItem)
        || subtree_has_recovery(ast, node)
        || direct_token_texts(ast, node).as_slice() != ["import", ";"]
    {
        return false;
    }
    let import_children = structural_child_ids(ast, node);
    let [decl_id] = import_children.as_slice() else {
        return false;
    };
    let Some(decl) = ast.node(*decl_id) else {
        return false;
    };
    if !matches!(decl.kind, SurfaceNodeKind::ImportAliasDecl)
        || !direct_token_texts(ast, decl).is_empty()
    {
        return false;
    }
    let decl_children = structural_child_ids(ast, decl);
    let [module_path_id] = decl_children.as_slice() else {
        return false;
    };
    let Some(module_path) = ast.node(*module_path_id) else {
        return false;
    };
    module_path_spelling(ast, module_path).is_ok_and(|spelling| spelling == "parser.type_fixtures")
}
