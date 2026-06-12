use crate::{ParserToken, ParserTokenKind, StringRequiredContext, grammar::Parser};
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
        if self.request.string_required_context != StringRequiredContext::UniformForTest {
            return;
        }

        let missing_position = self
            .request
            .tokens
            .iter()
            .position(|token| token.kind != ParserTokenKind::StringLiteral);
        let (position, span) = missing_position.map_or_else(
            || {
                let offset = self.request.tokens.last().map_or(0, |token| token.span.end);
                (
                    None,
                    SourceRange {
                        source_id: self.request.source_id,
                        start: offset,
                        end: offset,
                    },
                )
            },
            |position| {
                let token = &self.request.tokens[position];
                (
                    Some(position),
                    SourceRange {
                        source_id: token.span.source_id,
                        start: token.span.start,
                        end: token.span.start,
                    },
                )
            },
        );

        if position.is_none() && !self.request.tokens.is_empty() {
            return;
        }

        let diagnostic_primary =
            position.map_or(span, |position| self.request.tokens[position].span);
        self.diagnostics.push(
            SyntaxDiagnostic::new(
                SyntaxDiagnosticCode::MissingStringLiteral,
                "expected string literal at this grammar position",
                diagnostic_primary,
            )
            .with_recovery_note("insert a string literal before continuing"),
        );
        self.add_recovery_node(SyntaxRecoveryKind::MissingStringLiteral, span, Vec::new());
    }

    fn recover_block_ends(&mut self) -> RecoveryOutcome {
        let tokens = self.request.tokens.clone();
        let mut stack = Vec::new();

        for (position, token) in tokens.iter().enumerate() {
            if opens_recovery_block(&tokens, position) {
                stack.push(BlockStart {
                    keyword: token.text.clone(),
                    span: token.span,
                    token_node_id: self.token_node_ids[position],
                });
            } else if is_end_keyword(token) && stack.pop().is_none() {
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

fn is_reserved_word_token(token: &ParserToken, spelling: &str) -> bool {
    token.kind == ParserTokenKind::ReservedWord && token.text.as_ref() == spelling
}

fn is_end_keyword(token: &ParserToken) -> bool {
    is_reserved_word_token(token, "end")
}

fn is_else_keyword(token: &ParserToken) -> bool {
    is_reserved_word_token(token, "else")
}

fn opens_recovery_block(tokens: &[ParserToken], position: usize) -> bool {
    let token = &tokens[position];
    if !is_block_start_keyword(token) {
        return false;
    }

    if is_reserved_word_token(token, "for") {
        return looks_like_algorithm_for_loop(tokens, position);
    }

    !(is_reserved_word_token(token, "if") && position > 0 && is_else_keyword(&tokens[position - 1]))
}

fn looks_like_algorithm_for_loop(tokens: &[ParserToken], position: usize) -> bool {
    matches!(
        (tokens.get(position + 1), tokens.get(position + 2)),
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
