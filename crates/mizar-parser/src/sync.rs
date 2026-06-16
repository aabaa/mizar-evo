use crate::{
    ParserToken, ParserTokenKind,
    cursor::{TokenCursor, is_reserved_word_token},
};
use mizar_session::SourceRange;

const TOP_LEVEL_ITEM_KEYWORDS: &[&str] = &[
    "import",
    "export",
    "theorem",
    "lemma",
    "open",
    "assumed",
    "conditional",
    "private",
    "public",
    "definition",
    "registration",
    "claim",
    "reserve",
    "infix_operator",
    "prefix_operator",
    "postfix_operator",
    "synonym",
    "antonym",
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

pub(super) fn opens_recovery_block_at(tokens: &[ParserToken], position: usize) -> bool {
    let Some(token) = tokens.get(position) else {
        return false;
    };
    if token.kind != ParserTokenKind::ReservedWord {
        return false;
    }
    let cursor = TokenCursor::at(token.span.source_id, tokens, position);

    match token.text.as_ref() {
        "algorithm" | "definition" | "registration" | "proof" | "now" | "hereby" | "case"
        | "suppose" | "while" | "match" | "claim" | "struct" => true,
        "inherit" => looks_like_inherit_where_block(&cursor),
        "if" => looks_like_algorithm_if_block(&cursor),
        "for" => looks_like_algorithm_for_loop(&cursor),
        "otherwise" => follows_completed_match_case(&cursor),
        _ => false,
    }
}

fn looks_like_inherit_where_block(cursor: &TokenCursor<'_>) -> bool {
    let mut cursor = cursor.clone();
    while let Some(token) = cursor.current() {
        if is_reserved_word_token(token, "where") {
            return true;
        }
        if is_reserved_symbol_token(token, ";") || is_reserved_word_token(token, "end") {
            return false;
        }
        cursor.advance();
    }
    false
}

fn looks_like_algorithm_if_block(cursor: &TokenCursor<'_>) -> bool {
    let Some(previous) = cursor.previous() else {
        return false;
    };
    if is_reserved_word_token(previous, "else") {
        return false;
    }
    if previous.kind == ParserTokenKind::ReservedWord
        && matches!(
            previous.text.as_ref(),
            "algorithm"
                | "do"
                | "then"
                | "proof"
                | "now"
                | "hereby"
                | "case"
                | "suppose"
                | "while"
                | "for"
                | "match"
                | "claim"
        )
    {
        return true;
    }
    has_statement_body_marker_before_boundary(cursor)
}

fn has_statement_body_marker_before_boundary(cursor: &TokenCursor<'_>) -> bool {
    let mut lookahead = 1;
    while let Some(token) = cursor.token_at(cursor.position() + lookahead) {
        if is_reserved_word_token(token, "do") {
            return true;
        }
        if is_reserved_word_token(token, "end")
            || (token.kind == ParserTokenKind::ReservedSymbol && token.text.as_ref() == ";")
            || is_top_level_item_keyword(token)
        {
            return false;
        }
        lookahead += 1;
    }
    false
}

fn looks_like_algorithm_for_loop(cursor: &TokenCursor<'_>) -> bool {
    if matches!(
        (cursor.peek(1), cursor.peek(2)),
        (
            Some(ParserToken {
                kind: ParserTokenKind::Identifier,
                ..
            }),
            Some(next)
        ) if is_reserved_word_token(next, "in")
            || (next.kind == ParserTokenKind::ReservedSymbol && next.text.as_ref() == "=")
    ) {
        return true;
    }

    has_statement_body_marker_before_boundary(cursor)
}

fn follows_completed_match_case(cursor: &TokenCursor<'_>) -> bool {
    let Some(previous) = cursor.previous() else {
        return false;
    };
    if is_reserved_word_token(previous, "end") {
        return true;
    }
    previous.kind == ParserTokenKind::ReservedSymbol
        && previous.text.as_ref() == ";"
        && cursor
            .position()
            .checked_sub(2)
            .and_then(|position| cursor.token_at(position))
            .is_some_and(|token| is_reserved_word_token(token, "end"))
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
    fn block_openers_are_contextual_for_formula_keywords() {
        let source_id = source_id();
        let formula_if = vec![
            token(ParserTokenKind::Identifier, "P", 0, 1),
            token(ParserTokenKind::ReservedWord, "if", 2, 4),
            token(ParserTokenKind::Identifier, "Q", 5, 6),
        ];
        assert!(!super::opens_recovery_block_at(&formula_if, 1));

        let algorithm_if = vec![
            token(ParserTokenKind::ReservedWord, "algorithm", 0, 9),
            token(ParserTokenKind::ReservedWord, "if", 10, 12),
        ];
        assert!(super::opens_recovery_block_at(&algorithm_if, 1));

        let statement_list_if = vec![
            token(ParserTokenKind::Identifier, "x", 0, 1),
            token(ParserTokenKind::ReservedSymbol, ";", 1, 2),
            token(ParserTokenKind::ReservedWord, "if", 3, 5),
            token(ParserTokenKind::Identifier, "P", 6, 7),
            token(ParserTokenKind::ReservedWord, "do", 8, 10),
        ];
        assert!(super::opens_recovery_block_at(&statement_list_if, 2));

        let range_for = vec![
            token(ParserTokenKind::ReservedWord, "for", 0, 3),
            token(ParserTokenKind::Identifier, "i", 4, 5),
            token(ParserTokenKind::ReservedSymbol, "=", 6, 7),
        ];
        assert!(super::opens_recovery_block_at(&range_for, 0));

        let collection_for = vec![
            token(ParserTokenKind::ReservedWord, "for", 0, 3),
            token(ParserTokenKind::Identifier, "x", 4, 5),
            token(ParserTokenKind::ReservedWord, "in", 6, 8),
        ];
        assert!(super::opens_recovery_block_at(&collection_for, 0));

        let malformed_head_for = vec![
            token(ParserTokenKind::ReservedWord, "for", 0, 3),
            token(ParserTokenKind::Identifier, "item", 4, 8),
            token(ParserTokenKind::Identifier, "over", 9, 13),
            token(ParserTokenKind::Identifier, "Items", 14, 19),
            token(ParserTokenKind::ReservedWord, "do", 20, 22),
        ];
        assert!(super::opens_recovery_block_at(&malformed_head_for, 0));

        let quantifier_for = vec![
            token(ParserTokenKind::ReservedWord, "for", 0, 3),
            token(ParserTokenKind::Identifier, "x", 4, 5),
            token(ParserTokenKind::ReservedWord, "holds", 6, 11),
        ];
        assert!(!super::opens_recovery_block_at(&quantifier_for, 0));

        let malformed_head_without_do = vec![
            token(ParserTokenKind::ReservedWord, "for", 0, 3),
            token(ParserTokenKind::Identifier, "item", 4, 8),
            token(ParserTokenKind::Identifier, "over", 9, 13),
            token(ParserTokenKind::Identifier, "Items", 14, 19),
            token(ParserTokenKind::ReservedSymbol, ";", 19, 20),
        ];
        assert!(!super::opens_recovery_block_at(
            &malformed_head_without_do,
            0
        ));

        let formula_otherwise = vec![
            token(ParserTokenKind::Identifier, "x", 0, 1),
            token(ParserTokenKind::ReservedWord, "otherwise", 2, 11),
        ];
        assert!(!super::opens_recovery_block_at(&formula_otherwise, 1));

        let match_otherwise = vec![
            token(ParserTokenKind::ReservedWord, "end", 0, 3),
            token(ParserTokenKind::ReservedSymbol, ";", 3, 4),
            token(ParserTokenKind::ReservedWord, "otherwise", 5, 14),
        ];
        assert!(super::opens_recovery_block_at(&match_otherwise, 2));

        assert_eq!(formula_if[0].span.source_id, source_id);
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
