use super::{SurfaceNode, SurfaceNodeId, SurfaceNodeKind, SyntaxKind, contains_range};

pub(super) fn build_green_tree(
    nodes: &[SurfaceNode],
    root: Option<SurfaceNodeId>,
) -> rowan::GreenNode {
    let mut builder = rowan::GreenNodeBuilder::new();
    builder.start_node(rowan::SyntaxKind(SyntaxKind::Root as u16));
    if let Some(root) = root.and_then(|root| nodes.get(root.index()).map(|node| (root, node))) {
        append_root_contents(&mut builder, nodes, root.1);
    }
    builder.finish_node();
    builder.finish()
}

fn append_root_contents(
    builder: &mut rowan::GreenNodeBuilder<'_>,
    nodes: &[SurfaceNode],
    root: &SurfaceNode,
) {
    let structural_children = root
        .children
        .iter()
        .copied()
        .filter(|child| {
            node_kind(nodes, *child).is_some_and(|kind| {
                !matches!(
                    kind,
                    SurfaceNodeKind::Token(_) | SurfaceNodeKind::ErrorRecovery(_)
                )
            })
        })
        .collect::<Vec<_>>();
    let structural_tokens = structural_children
        .iter()
        .flat_map(|child| collect_token_descendants(nodes, *child))
        .collect::<Vec<_>>();
    let mut appended_structural = Vec::new();

    for child in root.children.iter().copied() {
        if structural_tokens.contains(&child) {
            let containing_structural = structural_children.iter().copied().find(|structure| {
                collect_token_descendants(nodes, *structure)
                    .first()
                    .copied()
                    == Some(child)
            });
            if let Some(structure) = containing_structural {
                append_green_node(builder, nodes, structure);
                appended_structural.push(structure);
            }
            continue;
        }
        if structural_children.contains(&child) {
            if !appended_structural.contains(&child) {
                append_green_node(builder, nodes, child);
                appended_structural.push(child);
            }
            continue;
        }
        append_green_node(builder, nodes, child);
    }
}

fn append_green_node(
    builder: &mut rowan::GreenNodeBuilder<'_>,
    nodes: &[SurfaceNode],
    id: SurfaceNodeId,
) {
    let Some(node) = nodes.get(id.index()) else {
        return;
    };
    builder.start_node(rowan::SyntaxKind(node.kind.syntax_kind() as u16));
    if let SurfaceNodeKind::Token(token) = &node.kind {
        builder.token(
            rowan::SyntaxKind(token.kind.syntax_kind() as u16),
            token.text.as_ref(),
        );
    }
    for child in children_to_append(nodes, node) {
        append_green_node(builder, nodes, child);
    }
    builder.finish_node();
}

fn children_to_append(nodes: &[SurfaceNode], node: &SurfaceNode) -> Vec<SurfaceNodeId> {
    if matches!(node.kind, SurfaceNodeKind::ErrorRecovery(_)) {
        node.children
            .iter()
            .copied()
            .filter(|child| {
                nodes
                    .get(child.index())
                    .is_some_and(|child| contains_range(node.range, child.range))
            })
            .collect()
    } else {
        node.children.clone()
    }
}

fn collect_token_descendants(nodes: &[SurfaceNode], id: SurfaceNodeId) -> Vec<SurfaceNodeId> {
    let Some(node) = nodes.get(id.index()) else {
        return Vec::new();
    };
    if matches!(node.kind, SurfaceNodeKind::Token(_)) {
        return vec![id];
    }
    node.children
        .iter()
        .flat_map(|child| collect_token_descendants(nodes, *child))
        .collect()
}

fn node_kind(nodes: &[SurfaceNode], id: SurfaceNodeId) -> Option<&SurfaceNodeKind> {
    nodes.get(id.index()).map(|node| &node.kind)
}
