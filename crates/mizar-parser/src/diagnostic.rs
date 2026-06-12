use crate::ParserToken;
use mizar_session::SourceRange;
use mizar_syntax::{SyntaxDiagnostic, SyntaxDiagnosticCode};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExpectedToken {
    description: Arc<str>,
}

impl ExpectedToken {
    pub(super) fn new(description: impl Into<Arc<str>>) -> Self {
        Self {
            description: description.into(),
        }
    }
}

pub(super) fn expected_token_diagnostic(
    code: SyntaxDiagnosticCode,
    expected: ExpectedToken,
    found: Option<&ParserToken>,
    eof_range: SourceRange,
    message: impl Into<Arc<str>>,
) -> SyntaxDiagnostic {
    debug_assert!(
        !expected.description.is_empty(),
        "expected-token diagnostics need a non-empty expectation description"
    );
    let primary = found.map_or(eof_range, |token| token.span);
    SyntaxDiagnostic::new(code, message, primary)
}

#[cfg(test)]
mod tests {
    use super::{ExpectedToken, expected_token_diagnostic};
    use crate::{ParserToken, ParserTokenKind};
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
        SourceRange,
    };
    use mizar_syntax::SyntaxDiagnosticCode;

    #[test]
    fn expected_token_diagnostic_uses_mid_stream_token_range() {
        let source_id = source_id(21);
        let found = ParserToken::new(
            ParserTokenKind::Identifier,
            "alpha",
            SourceRange {
                source_id,
                start: 4,
                end: 9,
            },
        );

        let diagnostic = expected_token_diagnostic(
            SyntaxDiagnosticCode::MissingStringLiteral,
            ExpectedToken::new("string literal"),
            Some(&found),
            SourceRange {
                source_id,
                start: 12,
                end: 12,
            },
            "expected string literal",
        );

        assert_eq!(
            diagnostic.primary,
            SourceRange {
                source_id,
                start: 4,
                end: 9,
            }
        );
    }

    #[test]
    fn expected_token_diagnostic_uses_eof_range_when_no_token_is_present() {
        let source_id = source_id(22);

        let diagnostic = expected_token_diagnostic(
            SyntaxDiagnosticCode::MissingStringLiteral,
            ExpectedToken::new("string literal"),
            None,
            SourceRange {
                source_id,
                start: 17,
                end: 17,
            },
            "expected string literal",
        );

        assert_eq!(
            diagnostic.primary,
            SourceRange {
                source_id,
                start: 17,
                end: 17,
            }
        );
    }

    fn source_id(byte: u8) -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(byte))
            .unwrap()
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
}
