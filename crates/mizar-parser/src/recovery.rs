use crate::{
    ParserToken, ParserTokenKind, StringRequiredContext,
    cursor::{TokenCursor, is_reserved_word_token},
    diagnostic::{ExpectedToken, expected_token_diagnostic},
    grammar::Parser,
    sync::{self, SynchronizationBoundary, SynchronizationSet},
};
use mizar_session::{SourceAnchor, SourceRange};
use mizar_syntax::{
    SurfaceBuilderNodeId, SyntaxDiagnostic, SyntaxDiagnosticCode, SyntaxRecoveryKind,
};
use std::sync::Arc;

impl Parser {
    pub(super) fn recover_syntax(&mut self) -> RecoveryOutcome {
        self.recover_missing_string_literal();
        let block_outcome = self.recover_block_ends();
        if block_outcome == RecoveryOutcome::Unrecoverable {
            RecoveryOutcome::Unrecoverable
        } else {
            RecoveryOutcome::Recovered
        }
    }

    fn recover_missing_string_literal(&mut self) {
        match self.request.string_required_context {
            StringRequiredContext::None => return,
            StringRequiredContext::UniformForTest => {}
        }

        let tokens = self.request.tokens.clone();
        let mut cursor = TokenCursor::new(self.request.source_id, &tokens);
        while cursor
            .current()
            .is_some_and(|token| token.kind == ParserTokenKind::StringLiteral)
        {
            cursor.advance();
        }

        if cursor.is_eof() && !tokens.is_empty() {
            return;
        }

        let span = cursor.current().map_or_else(
            || cursor.eof_range(),
            |token| SourceRange {
                source_id: token.span.source_id,
                start: token.span.start,
                end: token.span.start,
            },
        );
        let mut sync_cursor = TokenCursor::at(self.request.source_id, &tokens, cursor.position());
        let sync = sync::synchronize(&mut sync_cursor, SynchronizationSet::item_boundary());
        let _boundary = sync.boundary;
        let _skipped_range = sync.skipped_range;

        self.diagnostics.push(
            expected_token_diagnostic(
                SyntaxDiagnosticCode::MissingStringLiteral,
                ExpectedToken::new("string literal"),
                cursor.current(),
                cursor.eof_range(),
                "expected string literal at this grammar position",
            )
            .with_recovery_note("insert a string literal before continuing"),
        );
        self.add_recovery_node(SyntaxRecoveryKind::MissingStringLiteral, span, Vec::new());
    }

    fn recover_block_ends(&mut self) -> RecoveryOutcome {
        let tokens = self.request.tokens.clone();
        let mut stack = Vec::new();
        let mut cursor = TokenCursor::new(self.request.source_id, &tokens);
        let sync_set = SynchronizationSet::item_boundary();

        while let Some(token) = cursor.current() {
            let boundary = sync::boundary_at(&cursor, sync_set);
            if opens_recovery_block(&cursor) {
                stack.push(BlockStart {
                    keyword: token.text.clone(),
                    span: token.span,
                    token_node_id: self.token_node_ids[cursor.position()],
                });
            } else if boundary == Some(SynchronizationBoundary::EndKeyword) && stack.pop().is_none()
            {
                self.diagnostics.push(
                    SyntaxDiagnostic::new(
                        SyntaxDiagnosticCode::UnrecoverableInput,
                        "`end` has no matching block opener",
                        token.span,
                    )
                    .with_recovery_note(
                        "remove the stray `end` or add a matching block opener before it",
                    ),
                );
                return RecoveryOutcome::Unrecoverable;
            }
            cursor.advance();
        }

        if !stack.is_empty() {
            let offset = tokens.last().map_or(0, |token| token.span.end);
            self.recover_missing_ends(&mut stack, offset);
        }

        RecoveryOutcome::Recovered
    }

    fn recover_missing_ends(&mut self, stack: &mut Vec<BlockStart>, insertion_offset: usize) {
        while let Some(block) = stack.pop() {
            let span = SourceRange {
                source_id: self.request.source_id,
                start: insertion_offset,
                end: insertion_offset,
            };
            self.diagnostics.push(
                SyntaxDiagnostic::new(
                    SyntaxDiagnosticCode::MissingEnd,
                    format!("missing `end` for `{}` block", block.keyword),
                    span,
                )
                .with_secondary([SourceAnchor::Range(block.span)])
                .with_recovery_note("insert `end` before this synchronization point"),
            );
            self.add_recovery_node(
                SyntaxRecoveryKind::MissingEnd,
                span,
                vec![block.token_node_id],
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RecoveryOutcome {
    Recovered,
    Unrecoverable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockStart {
    keyword: Arc<str>,
    span: SourceRange,
    token_node_id: SurfaceBuilderNodeId,
}

fn is_else_keyword(token: &ParserToken) -> bool {
    is_reserved_word_token(token, "else")
}

fn opens_recovery_block(cursor: &TokenCursor<'_>) -> bool {
    let Some(token) = cursor.current() else {
        return false;
    };
    if !is_block_start_keyword(token) {
        return false;
    }

    if is_reserved_word_token(token, "for") {
        return looks_like_algorithm_for_loop(cursor);
    }

    !(is_reserved_word_token(token, "if") && cursor.previous().is_some_and(is_else_keyword))
}

fn looks_like_algorithm_for_loop(cursor: &TokenCursor<'_>) -> bool {
    matches!(
        (cursor.peek(1), cursor.peek(2)),
        (
            Some(ParserToken {
                kind: ParserTokenKind::Identifier,
                ..
            }),
            Some(next)
        ) if is_reserved_word_token(next, "in")
            || (next.kind == ParserTokenKind::ReservedSymbol && next.text.as_ref() == "=")
    )
}

fn is_block_start_keyword(token: &ParserToken) -> bool {
    token.kind == ParserTokenKind::ReservedWord
        && matches!(
            token.text.as_ref(),
            "algorithm"
                | "definition"
                | "registration"
                | "proof"
                | "now"
                | "hereby"
                | "case"
                | "suppose"
                | "if"
                | "while"
                | "for"
                | "match"
                | "claim"
                | "otherwise"
        )
}
