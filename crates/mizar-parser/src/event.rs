use mizar_session::{SourceId, SourceRange};
use mizar_syntax::{
    SurfaceAst, SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceNodeKind, SurfaceTokenKind,
    SyntaxRecoveryKind,
};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SyntaxEvent {
    Token {
        kind: SurfaceTokenKind,
        text: Arc<str>,
        range: SourceRange,
        recovered: bool,
    },
    Node {
        kind: SurfaceNodeKind,
        range: SourceRange,
        children: Vec<SurfaceBuilderNodeId>,
    },
    Recovery {
        kind: SyntaxRecoveryKind,
        range: SourceRange,
        children: Vec<SurfaceBuilderNodeId>,
    },
}

pub(super) struct SyntaxEventSink {
    builder: SurfaceAstBuilder,
    claimed_non_root_children: BTreeSet<SurfaceBuilderNodeId>,
}

impl SyntaxEventSink {
    pub(super) fn new(source_id: SourceId) -> Self {
        Self {
            builder: SurfaceAstBuilder::new(source_id),
            claimed_non_root_children: BTreeSet::new(),
        }
    }

    pub(super) fn emit(&mut self, event: SyntaxEvent) -> SurfaceBuilderNodeId {
        match event {
            SyntaxEvent::Token {
                kind,
                text,
                range,
                recovered,
            } => {
                if recovered {
                    self.builder.add_recovered_token(kind, text, range)
                } else {
                    self.builder.add_token(kind, text, range)
                }
            }
            SyntaxEvent::Node {
                kind,
                range,
                children,
            } => {
                if !matches!(kind, SurfaceNodeKind::Root) {
                    self.claimed_non_root_children
                        .extend(children.iter().copied());
                }
                self.builder.add_node(kind, range, children)
            }
            SyntaxEvent::Recovery {
                kind,
                range,
                children,
            } => {
                self.claimed_non_root_children
                    .extend(children.iter().copied());
                self.builder.add_recovery(kind, range, children)
            }
        }
    }

    pub(super) fn is_non_root_child_claimed(&self, id: SurfaceBuilderNodeId) -> bool {
        self.claimed_non_root_children.contains(&id)
    }

    pub(super) fn unclaimed_non_root_children(
        &self,
        ids: &[SurfaceBuilderNodeId],
    ) -> Vec<SurfaceBuilderNodeId> {
        ids.iter()
            .copied()
            .filter(|id| !self.is_non_root_child_claimed(*id))
            .collect()
    }

    pub(super) fn node_kind(&self, id: SurfaceBuilderNodeId) -> Option<&SurfaceNodeKind> {
        self.builder.node_kind(id)
    }

    pub(super) fn node_range(&self, id: SurfaceBuilderNodeId) -> Option<SourceRange> {
        self.builder.node_range(id)
    }

    pub(super) fn recovery_node_ids(&self) -> &[SurfaceBuilderNodeId] {
        self.builder.recovery_node_ids()
    }

    pub(super) fn finish(
        self,
        root: Option<SurfaceBuilderNodeId>,
        expression_root: Option<SurfaceBuilderNodeId>,
    ) -> SurfaceAst {
        self.builder.finish(root, expression_root)
    }
}
