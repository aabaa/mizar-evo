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
    MalformedImport,
    MalformedExport,
    MalformedVisibility,
    MalformedTypeExpression,
    MalformedTermExpression,
    MalformedFormulaExpression,
    MalformedJustification,
    MalformedAnnotation,
    UnexpectedTopLevelToken,
    UnrecoverableInput,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
    };

    #[test]
    fn syntax_diagnostic_builder_preserves_secondary_and_recovery_note() {
        let source_id = source_id(1);
        let primary = range(source_id, 10, 20);
        let first_secondary = SourceAnchor::Range(range(source_id, 0, 5));
        let second_secondary = SourceAnchor::Point {
            source_id,
            offset: 24,
        };

        let diagnostic = SyntaxDiagnostic::new(
            SyntaxDiagnosticCode::MalformedTermExpression,
            "missing term",
            primary,
        );

        assert_eq!(
            diagnostic.code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(diagnostic.message.as_ref(), "missing term");
        assert_eq!(diagnostic.primary, primary);
        assert!(diagnostic.secondary.is_empty());
        assert!(diagnostic.recovery_note.is_none());

        let diagnostic = diagnostic
            .with_secondary([first_secondary.clone()])
            .with_secondary([second_secondary.clone()])
            .with_recovery_note("inserted missing term");

        assert_eq!(
            diagnostic.secondary,
            vec![first_secondary, second_secondary]
        );
        assert_eq!(
            diagnostic.recovery_note.as_deref(),
            Some("inserted missing term")
        );
    }

    fn source_id(byte: u8) -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(byte))
            .unwrap()
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
}
