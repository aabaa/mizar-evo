use crate::{ParserToken, ParserTokenKind};
use mizar_session::{SourceId, SourceRange};

pub(super) const MAX_LOOKAHEAD: usize = 3;

#[derive(Debug, Clone)]
pub(super) struct TokenCursor<'a> {
    source_id: SourceId,
    tokens: &'a [ParserToken],
    position: usize,
}

impl<'a> TokenCursor<'a> {
    pub(super) fn new(source_id: SourceId, tokens: &'a [ParserToken]) -> Self {
        Self {
            source_id,
            tokens,
            position: 0,
        }
    }

    pub(super) fn at(source_id: SourceId, tokens: &'a [ParserToken], position: usize) -> Self {
        Self {
            source_id,
            tokens,
            position: position.min(tokens.len()),
        }
    }

    pub(super) const fn position(&self) -> usize {
        self.position
    }

    pub(super) fn current(&self) -> Option<&'a ParserToken> {
        self.tokens.get(self.position)
    }

    pub(super) fn token_at(&self, position: usize) -> Option<&'a ParserToken> {
        self.tokens.get(position)
    }

    pub(super) fn peek(&self, lookahead: usize) -> Option<&'a ParserToken> {
        assert!(
            lookahead <= MAX_LOOKAHEAD,
            "parser token lookahead must stay within the bounded cursor window"
        );
        self.tokens.get(self.position + lookahead)
    }

    pub(super) fn previous(&self) -> Option<&'a ParserToken> {
        self.position
            .checked_sub(1)
            .and_then(|position| self.tokens.get(position))
    }

    pub(super) fn advance(&mut self) -> Option<&'a ParserToken> {
        let token = self.current()?;
        self.position += 1;
        Some(token)
    }

    pub(super) fn is_eof(&self) -> bool {
        self.position >= self.tokens.len()
    }

    pub(super) fn eof_range(&self) -> SourceRange {
        let offset = self.tokens.last().map_or(0, |token| token.span.end);
        SourceRange {
            source_id: self.source_id,
            start: offset,
            end: offset,
        }
    }
}

pub(super) fn is_reserved_word_token(token: &ParserToken, spelling: &str) -> bool {
    token.kind == ParserTokenKind::ReservedWord && token.text.as_ref() == spelling
}

#[cfg(test)]
mod tests {
    use super::{MAX_LOOKAHEAD, TokenCursor};
    use crate::{ParserToken, ParserTokenKind};
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
        SourceRange,
    };

    #[test]
    fn cursor_peeks_within_the_bounded_lookahead_window() {
        let source_id = source_id(24);
        let tokens = vec![
            token(source_id, "t0", 0, 2),
            token(source_id, "t1", 3, 5),
            token(source_id, "t2", 6, 8),
            token(source_id, "t3", 9, 11),
        ];
        let cursor = TokenCursor::new(source_id, &tokens);

        assert_eq!(
            cursor.peek(MAX_LOOKAHEAD).map(|token| token.text.as_ref()),
            Some("t3")
        );
    }

    #[test]
    #[should_panic(expected = "bounded cursor window")]
    fn cursor_rejects_lookahead_beyond_the_bounded_window() {
        let source_id = source_id(25);
        let tokens = vec![token(source_id, "t0", 0, 2)];
        let cursor = TokenCursor::new(source_id, &tokens);

        let _ = cursor.peek(MAX_LOOKAHEAD + 1);
    }

    #[test]
    fn cursor_at_clamps_to_eof_and_reports_last_token_end_as_eof_range() {
        let source_id = source_id(26);
        let tokens = vec![token(source_id, "t0", 4, 6), token(source_id, "t1", 9, 11)];
        let cursor = TokenCursor::at(source_id, &tokens, 99);

        assert!(cursor.is_eof());
        assert_eq!(
            cursor.eof_range(),
            SourceRange {
                source_id,
                start: 11,
                end: 11,
            }
        );
    }

    #[test]
    fn empty_cursor_reports_zero_width_eof_range_at_start() {
        let source_id = source_id(27);
        let tokens = Vec::new();
        let cursor = TokenCursor::new(source_id, &tokens);

        assert!(cursor.is_eof());
        assert_eq!(
            cursor.eof_range(),
            SourceRange {
                source_id,
                start: 0,
                end: 0,
            }
        );
    }

    fn token(source_id: SourceId, text: &str, start: usize, end: usize) -> ParserToken {
        ParserToken::new(
            ParserTokenKind::Identifier,
            text,
            SourceRange {
                source_id,
                start,
                end,
            },
        )
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
