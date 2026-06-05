use mizar_session::{SourceAnchor, SourceId, SourceRange};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceAst {
    pub source_id: SourceId,
    pub nodes: Vec<SurfaceNode>,
    pub root: Option<SurfaceNodeId>,
    pub token_nodes: Vec<SurfaceNodeId>,
    pub expression_root: Option<SurfaceNodeId>,
}

impl SurfaceAst {
    pub fn new(
        source_id: SourceId,
        nodes: Vec<SurfaceNode>,
        root: Option<SurfaceNodeId>,
        token_nodes: Vec<SurfaceNodeId>,
        expression_root: Option<SurfaceNodeId>,
    ) -> Self {
        Self {
            source_id,
            nodes,
            root,
            token_nodes,
            expression_root,
        }
    }

    pub fn node(&self, id: SurfaceNodeId) -> Option<&SurfaceNode> {
        self.nodes.get(id.index())
    }

    pub fn token_texts(&self) -> Vec<&str> {
        self.token_nodes
            .iter()
            .filter_map(|id| self.node(*id))
            .filter_map(SurfaceNode::token_text)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceNodeId(usize);

impl SurfaceNodeId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceNode {
    pub kind: SurfaceNodeKind,
    pub range: SourceRange,
    pub children: Vec<SurfaceNodeId>,
    pub recovered: bool,
}

impl SurfaceNode {
    pub fn new(kind: SurfaceNodeKind, range: SourceRange, children: Vec<SurfaceNodeId>) -> Self {
        Self {
            kind,
            range,
            children,
            recovered: false,
        }
    }

    pub fn recovered(
        kind: SurfaceNodeKind,
        range: SourceRange,
        children: Vec<SurfaceNodeId>,
    ) -> Self {
        Self {
            kind,
            range,
            children,
            recovered: true,
        }
    }

    pub fn token_text(&self) -> Option<&str> {
        match &self.kind {
            SurfaceNodeKind::Token(token) => Some(token.text.as_ref()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceNodeKind {
    Root,
    Token(SurfaceToken),
    InfixExpression(SurfaceInfixOperator),
    ErrorRecovery(SyntaxRecoveryKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceToken {
    pub kind: SurfaceTokenKind,
    pub text: Arc<str>,
}

impl SurfaceToken {
    pub fn new(kind: SurfaceTokenKind, text: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceTokenKind {
    Identifier,
    ReservedWord,
    ReservedSymbol,
    Numeral,
    LexemeRun,
    UserSymbol,
    StringLiteral,
    ErrorRecovery,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceInfixOperator {
    pub spelling: Arc<str>,
    pub precedence: u8,
    pub associativity: SurfaceOperatorAssociativity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceOperatorAssociativity {
    Left,
    Right,
    NonAssociative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxRecoveryKind {
    ErrorToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxDiagnostic {
    pub code: SyntaxDiagnosticCode,
    pub message: Arc<str>,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
    pub recovery_note: Option<Arc<str>>,
}

impl SyntaxDiagnostic {
    pub fn new(
        code: SyntaxDiagnosticCode,
        message: impl Into<Arc<str>>,
        primary: SourceRange,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            primary,
            secondary: Vec::new(),
            recovery_note: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxDiagnosticCode {
    UnexpectedErrorToken,
    DanglingOperator,
    NonAssociativeOperatorChain,
}
