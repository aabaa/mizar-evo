use mizar_session::{SourceId, SourceRange};
use mizar_syntax::{
    SurfaceAst, SurfaceAstBuilder, SurfaceBuilderNodeId, SurfaceNodeKind, SurfaceTokenKind,
    SyntaxRecoveryKind,
};
use std::sync::Arc;

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
}

impl SyntaxEventSink {
    pub(super) fn new(source_id: SourceId) -> Self {
        Self {
            builder: SurfaceAstBuilder::new(source_id),
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
            } => self.builder.add_node(kind, range, children),
            SyntaxEvent::Recovery {
                kind,
                range,
                children,
            } => self.builder.add_recovery(kind, range, children),
        }
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
