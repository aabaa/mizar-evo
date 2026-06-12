use crate::{
    ParserToken, ParserTokenKind,
    cursor::{TokenCursor, is_reserved_word_token},
};
use mizar_session::SourceRange;

const TOP_LEVEL_ITEM_KEYWORDS: &[&str] = &[
    "theorem",
    "definition",
    "registration",
    "notation",
    "scheme",
    "reserve",
    "begin",
    "environ",
    "vocabularies",
    "constructors",
    "requirements",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SynchronizationBoundary {
    Semicolon,
    EndKeyword,
    TopLevelItemKeyword,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SynchronizationSet {
    semicolon: bool,
    end_keyword: bool,
    top_level_item_keyword: bool,
}

impl SynchronizationSet {
    pub(super) const fn item_boundary() -> Self {
        Self {
            semicolon: true,
            end_keyword: true,
            top_level_item_keyword: true,
        }
    }

    fn matches(self, token: &ParserToken) -> Option<SynchronizationBoundary> {
        if self.semicolon && is_reserved_symbol_token(token, ";") {
            return Some(SynchronizationBoundary::Semicolon);
        }
        if self.end_keyword && is_reserved_word_token(token, "end") {
            return Some(SynchronizationBoundary::EndKeyword);
        }
        if self.top_level_item_keyword && is_top_level_item_keyword(token) {
            return Some(SynchronizationBoundary::TopLevelItemKeyword);
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Synchronization {
    pub(super) boundary: SynchronizationBoundary,
    pub(super) skipped_range: Option<SourceRange>,
}

pub(super) fn synchronize(
    cursor: &mut TokenCursor<'_>,
    set: SynchronizationSet,
) -> Synchronization {
    let skipped_start = cursor.current().map(|token| token.span.start);
    let mut skipped_end = None;

    while let Some(token) = cursor.current() {
        if let Some(boundary) = set.matches(token) {
            return Synchronization {
                boundary,
                skipped_range: skipped_range(token.span.source_id, skipped_start, skipped_end),
            };
        }
        skipped_end = Some(token.span.end);
        cursor.advance();
    }

    Synchronization {
        boundary: SynchronizationBoundary::Eof,
        skipped_range: skipped_range(cursor.eof_range().source_id, skipped_start, skipped_end),
    }
}

pub(super) fn boundary_at(
    cursor: &TokenCursor<'_>,
    set: SynchronizationSet,
) -> Option<SynchronizationBoundary> {
    cursor.current().and_then(|token| set.matches(token))
}

pub(super) fn is_top_level_item_keyword(token: &ParserToken) -> bool {
    token.kind == ParserTokenKind::ReservedWord
        && TOP_LEVEL_ITEM_KEYWORDS.contains(&token.text.as_ref())
}

fn skipped_range(
    source_id: mizar_session::SourceId,
    start: Option<usize>,
    end: Option<usize>,
) -> Option<SourceRange> {
    start.zip(end).map(|(start, end)| SourceRange {
        source_id,
        start,
        end,
    })
}

fn is_reserved_symbol_token(token: &ParserToken, spelling: &str) -> bool {
    token.kind == ParserTokenKind::ReservedSymbol && token.text.as_ref() == spelling
}

#[cfg(test)]
mod tests {
    use super::{
        SynchronizationBoundary, SynchronizationSet, TOP_LEVEL_ITEM_KEYWORDS, synchronize,
    };
    use crate::{ParserToken, ParserTokenKind, cursor::TokenCursor};
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
        SourceRange,
    };

    #[test]
    fn synchronization_skips_to_semicolon_and_records_skipped_range() {
        assert_sync_boundary(
            vec![
                token(ParserTokenKind::Identifier, "bad", 0, 3),
                token(ParserTokenKind::ReservedSymbol, ";", 4, 5),
            ],
            SynchronizationBoundary::Semicolon,
            Some((0, 3)),
        );
    }

    #[test]
    fn synchronization_skips_to_end_keyword_and_records_skipped_range() {
        assert_sync_boundary(
            vec![
                token(ParserTokenKind::Identifier, "bad", 0, 3),
                token(ParserTokenKind::ReservedWord, "end", 4, 7),
            ],
            SynchronizationBoundary::EndKeyword,
            Some((0, 3)),
        );
    }

    #[test]
    fn synchronization_skips_to_top_level_item_keyword_and_records_skipped_range() {
        for keyword in TOP_LEVEL_ITEM_KEYWORDS {
            assert_sync_boundary(
                vec![
                    token(ParserTokenKind::Identifier, "bad", 0, 3),
                    token(ParserTokenKind::ReservedWord, keyword, 4, 4 + keyword.len()),
                ],
                SynchronizationBoundary::TopLevelItemKeyword,
                Some((0, 3)),
            );
        }
    }

    #[test]
    fn synchronization_does_not_treat_other_reserved_words_as_top_level_items() {
        assert_sync_boundary(
            vec![token(ParserTokenKind::ReservedWord, "func", 0, 4)],
            SynchronizationBoundary::Eof,
            Some((0, 4)),
        );
    }

    #[test]
    fn synchronization_at_boundary_records_no_skipped_range() {
        assert_sync_boundary(
            vec![token(ParserTokenKind::ReservedSymbol, ";", 0, 1)],
            SynchronizationBoundary::Semicolon,
            None,
        );
    }

    #[test]
    fn synchronization_on_empty_input_reports_eof_with_no_skipped_range() {
        assert_sync_boundary(Vec::new(), SynchronizationBoundary::Eof, None);
    }

    #[test]
    fn synchronization_reports_eof_after_skipping_to_stream_end() {
        assert_sync_boundary(
            vec![
                token(ParserTokenKind::Identifier, "bad", 0, 3),
                token(ParserTokenKind::Identifier, "tail", 4, 8),
            ],
            SynchronizationBoundary::Eof,
            Some((0, 8)),
        );
    }

    fn assert_sync_boundary(
        tokens: Vec<ParserToken>,
        expected_boundary: SynchronizationBoundary,
        expected_skipped: Option<(usize, usize)>,
    ) {
        let source_id = source_id();
        let tokens = tokens
            .into_iter()
            .map(|mut token| {
                token.span.source_id = source_id;
                token
            })
            .collect::<Vec<_>>();
        let mut cursor = TokenCursor::new(source_id, &tokens);

        let synchronization = synchronize(&mut cursor, SynchronizationSet::item_boundary());

        assert_eq!(synchronization.boundary, expected_boundary);
        assert_eq!(
            synchronization.skipped_range,
            expected_skipped.map(|(start, end)| SourceRange {
                source_id,
                start,
                end,
            })
        );
    }

    fn token(kind: ParserTokenKind, text: &str, start: usize, end: usize) -> ParserToken {
        ParserToken::new(
            kind,
            text,
            SourceRange {
                source_id: source_id(),
                start,
                end,
            },
        )
    }

    fn source_id() -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id())
            .unwrap()
    }

    fn snapshot_id() -> BuildSnapshotId {
        let hex = "23".repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
}
