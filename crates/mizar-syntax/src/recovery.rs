use mizar_session::{SourceAnchor, SourceRange};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SyntaxRecoveryKind {
    ErrorToken,
    MissingEnd,
    MissingStringLiteral,
    MissingItem,
    MissingTypeExpression,
    MissingTerm,
    MissingFormula,
    MissingStatement,
    MissingProofStep,
    MissingAnnotationArgument,
    SkippedToken,
    UnmatchedOpeningDelimiter,
    UnmatchedClosingDelimiter,
    MalformedAnnotation,
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

    pub fn with_secondary(mut self, secondary: impl IntoIterator<Item = SourceAnchor>) -> Self {
        self.secondary.extend(secondary);
        self
    }

    pub fn with_recovery_note(mut self, note: impl Into<Arc<str>>) -> Self {
        self.recovery_note = Some(note.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SyntaxDiagnosticCode {
    UnexpectedErrorToken,
    DanglingOperator,
    NonAssociativeOperatorChain,
    MissingEnd,
    MissingSemicolon,
    MissingStringLiteral,
    UnexpectedTopLevelToken,
    UnrecoverableInput,
}
