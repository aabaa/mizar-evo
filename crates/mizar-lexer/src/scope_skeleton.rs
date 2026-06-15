use crate::raw_lexer::{
    RawToken, RawTokenKind, RawTokenStream, is_identifier, is_identifier_continue,
    is_identifier_start,
};
use crate::source::{SourcePos, SourceRange, SourceSpan};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeSkeleton {
    pub frames: Vec<LexicalScopeFrame>,
    pub blocks: Vec<LexicalBlockRange>,
    pub statements: Vec<LexicalStatementRange>,
    pub diagnostics: Vec<ScopeSkeletonDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalScopeFrame {
    pub range: SourceRange,
    pub bindings: Vec<ScopedBindingShape>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedBindingShape {
    pub spelling: String,
    pub introduced_at: SourceRange,
    pub kind: BindingShapeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BindingShapeKind {
    Let,
    For,
    Ex,
    Reserve,
    Given,
    Consider,
    Set,
    Reconsider,
    Take,
    Deffunc,
    Defpred,
    Var,
    Const,
    Processed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalBlockRange {
    pub kind: LexicalBlockKind,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LexicalBlockKind {
    Algorithm,
    Definition,
    Proof,
    Now,
    Case,
    Suppose,
    Hereby,
    Do,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalStatementRange {
    pub kind: LexicalStatementKind,
    pub range: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LexicalStatementKind {
    Binder,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeSkeletonDiagnostic {
    pub code: ScopeSkeletonDiagnosticCode,
    pub message: String,
    pub span: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScopeSkeletonDiagnosticCode {
    MalformedBinderList,
    UnsupportedBinderShape,
    DuplicateBindingName,
    UnmatchedEnd,
    MissingEnd,
}

pub trait ScopeLexView {
    fn binding_overrides_symbol(&self, spelling: &str, position: SourcePos) -> bool;
}

pub fn build_scope_skeleton(raw: &RawTokenStream) -> ScopeSkeleton {
    let tokens = split_scope_skeleton_tokens(raw);
    ScopeSkeletonBuilder::new(tokens).build()
}
impl ScopeLexView for ScopeSkeleton {
    fn binding_overrides_symbol(&self, spelling: &str, position: SourcePos) -> bool {
        self.frames.iter().any(|frame| {
            frame.range.start <= position
                && position < frame.range.end
                && frame.bindings.iter().any(|binding| {
                    binding.spelling == spelling
                        && binding.introduced_at.end <= position
                        && position < frame.range.end
                })
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct ScopeSkeletonToken {
    kind: ScopeSkeletonTokenKind,
    lexeme: String,
    span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeSkeletonTokenKind {
    Word,
    Comma,
    Semicolon,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpenScopeFrame {
    kind: LexicalBlockKind,
    start: SourcePos,
    bindings: Vec<ScopedBindingShape>,
}

struct ScopeSkeletonBuilder {
    tokens: Vec<ScopeSkeletonToken>,
    cursor: usize,
    root: OpenScopeFrame,
    stack: Vec<OpenScopeFrame>,
    pending_do_bindings: Vec<ScopedBindingShape>,
    frames: Vec<LexicalScopeFrame>,
    blocks: Vec<LexicalBlockRange>,
    statements: Vec<LexicalStatementRange>,
    diagnostics: Vec<ScopeSkeletonDiagnostic>,
    source_end: SourcePos,
}

impl ScopeSkeletonBuilder {
    fn new(tokens: Vec<ScopeSkeletonToken>) -> Self {
        let source_end = tokens.last().map_or(0, |token| token.span.end);
        Self {
            tokens,
            cursor: 0,
            root: OpenScopeFrame {
                kind: LexicalBlockKind::Definition,
                start: 0,
                bindings: Vec::new(),
            },
            stack: Vec::new(),
            pending_do_bindings: Vec::new(),
            frames: Vec::new(),
            blocks: Vec::new(),
            statements: Vec::new(),
            diagnostics: Vec::new(),
            source_end,
        }
    }

    fn build(mut self) -> ScopeSkeleton {
        while let Some(token) = self.peek().cloned() {
            if token.kind == ScopeSkeletonTokenKind::Word {
                match token.lexeme.as_str() {
                    "algorithm" => {
                        self.advance();
                        self.open_frame(LexicalBlockKind::Algorithm, token.span.start);
                        continue;
                    }
                    "definition" => {
                        self.advance();
                        self.open_frame(LexicalBlockKind::Definition, token.span.start);
                        continue;
                    }
                    "proof" => {
                        self.advance();
                        self.open_frame(LexicalBlockKind::Proof, token.span.start);
                        continue;
                    }
                    "now" => {
                        self.advance();
                        self.open_frame(LexicalBlockKind::Now, token.span.start);
                        continue;
                    }
                    "case" => {
                        self.advance();
                        if !self.tokens_until_stop_contain_word("do") {
                            self.open_frame(LexicalBlockKind::Case, token.span.start);
                        }
                        continue;
                    }
                    "suppose" => {
                        self.advance();
                        self.open_frame(LexicalBlockKind::Suppose, token.span.start);
                        continue;
                    }
                    "hereby" => {
                        self.advance();
                        self.open_frame(LexicalBlockKind::Hereby, token.span.start);
                        continue;
                    }
                    "do" => {
                        self.advance();
                        self.open_frame(LexicalBlockKind::Do, token.span.start);
                        continue;
                    }
                    "end" => {
                        self.advance();
                        self.close_frame(token.span);
                        continue;
                    }
                    "let" => {
                        self.advance();
                        self.parse_binders(BindingShapeKind::Let, token.span);
                        continue;
                    }
                    "for" => {
                        self.advance();
                        self.parse_binders(BindingShapeKind::For, token.span);
                        continue;
                    }
                    "ex" => {
                        self.advance();
                        self.parse_binders(BindingShapeKind::Ex, token.span);
                        continue;
                    }
                    "reserve" => {
                        self.advance();
                        self.parse_binders(BindingShapeKind::Reserve, token.span);
                        continue;
                    }
                    "given" => {
                        self.advance();
                        self.parse_binders(BindingShapeKind::Given, token.span);
                        continue;
                    }
                    "consider" => {
                        self.advance();
                        self.parse_binders(BindingShapeKind::Consider, token.span);
                        continue;
                    }
                    "set" if self.should_parse_set_named_equals_binder() => {
                        self.advance();
                        self.parse_named_equals_binder(BindingShapeKind::Set, token.span);
                        continue;
                    }
                    "reconsider" => {
                        self.advance();
                        self.parse_reconsider_binders(token.span);
                        continue;
                    }
                    "take" => {
                        self.advance();
                        self.parse_named_equals_binder(BindingShapeKind::Take, token.span);
                        continue;
                    }
                    "deffunc" => {
                        self.advance();
                        self.parse_binders(BindingShapeKind::Deffunc, token.span);
                        continue;
                    }
                    "defpred" => {
                        self.advance();
                        self.parse_binders(BindingShapeKind::Defpred, token.span);
                        continue;
                    }
                    "var" => {
                        self.advance();
                        self.parse_binders(BindingShapeKind::Var, token.span);
                        continue;
                    }
                    "const" => {
                        self.advance();
                        self.parse_binders(BindingShapeKind::Const, token.span);
                        continue;
                    }
                    "ghost" => {
                        self.advance();
                        self.parse_ghost_binders(token.span);
                        continue;
                    }
                    _ => {}
                }
            }

            self.advance();
        }

        while let Some(open) = self.stack.pop() {
            self.diagnostic(
                ScopeSkeletonDiagnosticCode::MissingEnd,
                "missing `end` for lexical scope block",
                SourceSpan {
                    start: open.start,
                    end: open.start,
                },
            );
            let range = SourceSpan {
                start: open.start,
                end: self.source_end,
            };
            self.blocks.push(LexicalBlockRange {
                kind: open.kind,
                range,
            });
            self.frames.push(LexicalScopeFrame {
                range,
                bindings: open.bindings,
            });
        }

        if !self.root.bindings.is_empty() {
            self.frames.push(LexicalScopeFrame {
                range: SourceSpan {
                    start: self.root.start,
                    end: self.source_end,
                },
                bindings: self.root.bindings,
            });
        }

        self.frames.sort_by(|left, right| {
            left.range
                .start
                .cmp(&right.range.start)
                .then_with(|| left.range.end.cmp(&right.range.end))
        });
        self.blocks.sort_by(|left, right| {
            left.range
                .start
                .cmp(&right.range.start)
                .then_with(|| left.range.end.cmp(&right.range.end))
        });
        self.statements.sort_by(|left, right| {
            left.range
                .start
                .cmp(&right.range.start)
                .then_with(|| left.range.end.cmp(&right.range.end))
        });
        self.diagnostics
            .sort_by_key(|diagnostic| diagnostic.span.start);

        ScopeSkeleton {
            frames: self.frames,
            blocks: self.blocks,
            statements: self.statements,
            diagnostics: self.diagnostics,
        }
    }

    fn open_frame(&mut self, kind: LexicalBlockKind, start: SourcePos) {
        let bindings = if kind == LexicalBlockKind::Do {
            std::mem::take(&mut self.pending_do_bindings)
        } else {
            Vec::new()
        };
        self.stack.push(OpenScopeFrame {
            kind,
            start,
            bindings,
        });
    }

    fn close_frame(&mut self, end_span: SourceSpan) {
        let Some(open) = self.stack.pop() else {
            self.diagnostic(
                ScopeSkeletonDiagnosticCode::UnmatchedEnd,
                "unmatched `end` in lexical scope skeleton",
                end_span,
            );
            return;
        };

        let range = SourceSpan {
            start: open.start,
            end: end_span.end,
        };
        self.blocks.push(LexicalBlockRange {
            kind: open.kind,
            range,
        });
        self.frames.push(LexicalScopeFrame {
            range,
            bindings: open.bindings,
        });
    }

    fn parse_binders(&mut self, kind: BindingShapeKind, keyword_span: SourceSpan) {
        if matches!(kind, BindingShapeKind::Var | BindingShapeKind::Const) {
            self.parse_algorithm_binders(kind, keyword_span);
            return;
        }

        let mut expected_name = true;
        let mut saw_binding = false;
        let mut saw_malformed = false;
        let mut bindings = Vec::new();

        while let Some(token) = self.peek().cloned() {
            if token.kind == ScopeSkeletonTokenKind::Semicolon {
                break;
            }
            if token.kind == ScopeSkeletonTokenKind::Word && binder_list_stop_word(&token.lexeme) {
                break;
            }

            if expected_name {
                if token.kind == ScopeSkeletonTokenKind::Word && is_identifier(&token.lexeme) {
                    let token = self.advance().expect("peeked token exists");
                    bindings.push(ScopedBindingShape {
                        spelling: token.lexeme,
                        introduced_at: token.span,
                        kind,
                    });
                    saw_binding = true;
                    expected_name = false;
                    continue;
                }

                if token.kind == ScopeSkeletonTokenKind::Comma {
                    saw_malformed = true;
                    break;
                }
                break;
            }

            if token.kind == ScopeSkeletonTokenKind::Comma {
                self.advance();
                expected_name = true;
                continue;
            }

            break;
        }

        if !saw_binding && saw_malformed {
            let span = self.peek().map_or(keyword_span, |token| token.span);
            self.diagnostic(
                ScopeSkeletonDiagnosticCode::MalformedBinderList,
                "malformed binder list was under-approximated",
                span,
            );
        } else if !saw_binding {
            let span = self.peek().map_or(keyword_span, |token| token.span);
            self.diagnostic(
                ScopeSkeletonDiagnosticCode::UnsupportedBinderShape,
                "binder keyword is not followed by a supported identifier-shaped binder list",
                span,
            );
        } else if saw_malformed || expected_name {
            let span = self.peek().map_or(keyword_span, |token| token.span);
            self.diagnostic(
                ScopeSkeletonDiagnosticCode::MalformedBinderList,
                "malformed binder list was under-approximated",
                span,
            );
        }

        let has_do_block = self.tokens_until_stop_contain_word("do");
        let processed_binding = if has_do_block {
            self.processed_binder_before_block()
        } else {
            None
        };
        let statement_end = self.recover_to_binder_statement_end();
        if bindings.is_empty() {
            return;
        }
        self.statements.push(LexicalStatementRange {
            kind: LexicalStatementKind::Binder,
            range: SourceSpan {
                start: keyword_span.start,
                end: statement_end,
            },
        });

        match kind {
            BindingShapeKind::For if has_do_block => {
                let mut do_bindings = Vec::new();
                if let Some(binding) = processed_binding {
                    do_bindings.push(binding);
                }
                do_bindings.extend(bindings);
                let existing = self
                    .pending_do_bindings
                    .iter()
                    .map(|binding| binding.spelling.clone())
                    .collect::<Vec<_>>();
                let do_bindings = self.deduplicate_bindings(do_bindings, existing);
                self.pending_do_bindings.extend(do_bindings);
            }
            BindingShapeKind::For | BindingShapeKind::Ex | BindingShapeKind::Given => {
                self.push_statement_frame(keyword_span.start, statement_end, bindings);
            }
            BindingShapeKind::Consider => {
                self.extend_current_or_statement(keyword_span.start, statement_end, bindings);
            }
            BindingShapeKind::Let if self.stack.is_empty() => {
                self.push_statement_frame(keyword_span.start, statement_end, bindings);
            }
            BindingShapeKind::Let
            | BindingShapeKind::Set
            | BindingShapeKind::Reconsider
            | BindingShapeKind::Take
            | BindingShapeKind::Deffunc
            | BindingShapeKind::Defpred
            | BindingShapeKind::Var
            | BindingShapeKind::Const
            | BindingShapeKind::Processed => {
                self.extend_current_or_statement(keyword_span.start, statement_end, bindings);
            }
            BindingShapeKind::Reserve if !self.stack.is_empty() => {
                self.diagnostic(
                    ScopeSkeletonDiagnosticCode::UnsupportedBinderShape,
                    "`reserve` inside a lexical block is not used for scope skeleton bindings",
                    keyword_span,
                );
            }
            BindingShapeKind::Reserve => {
                let existing = self
                    .root
                    .bindings
                    .iter()
                    .map(|binding| binding.spelling.clone())
                    .collect::<Vec<_>>();
                let bindings = self.deduplicate_bindings(bindings, existing);
                self.root.bindings.extend(bindings);
            }
        }
    }

    fn parse_algorithm_binders(&mut self, kind: BindingShapeKind, keyword_span: SourceSpan) {
        let mut bindings = Vec::new();
        let mut expected_name = true;
        let mut depth = 0usize;

        while let Some(token) = self.peek().cloned() {
            if token.kind == ScopeSkeletonTokenKind::Semicolon || token_is_block_boundary(&token) {
                break;
            }
            if depth == 0
                && token.kind == ScopeSkeletonTokenKind::Word
                && matches!(token.lexeme.as_str(), "as" | "by" | "proof")
            {
                break;
            }
            match token.kind {
                ScopeSkeletonTokenKind::LParen => {
                    depth += 1;
                    self.advance();
                }
                ScopeSkeletonTokenKind::RParen => {
                    depth = depth.saturating_sub(1);
                    self.advance();
                }
                ScopeSkeletonTokenKind::Comma if depth == 0 => {
                    expected_name = true;
                    self.advance();
                }
                ScopeSkeletonTokenKind::Word if expected_name && is_identifier(&token.lexeme) => {
                    let token = self.advance().expect("peeked token exists");
                    bindings.push(ScopedBindingShape {
                        spelling: token.lexeme,
                        introduced_at: token.span,
                        kind,
                    });
                    expected_name = false;
                }
                _ => {
                    expected_name = false;
                    self.advance();
                }
            }
        }

        let statement_end = self.recover_to_binder_statement_end();
        if bindings.is_empty() {
            self.diagnostic(
                ScopeSkeletonDiagnosticCode::UnsupportedBinderShape,
                "algorithm binding keyword is not followed by an identifier-shaped binding",
                keyword_span,
            );
            return;
        }
        self.statements.push(LexicalStatementRange {
            kind: LexicalStatementKind::Binder,
            range: SourceSpan {
                start: keyword_span.start,
                end: statement_end,
            },
        });
        self.extend_current_or_statement(keyword_span.start, statement_end, bindings);
    }

    fn parse_named_equals_binder(&mut self, kind: BindingShapeKind, keyword_span: SourceSpan) {
        let Some(name) = self.peek().cloned() else {
            self.diagnostic(
                ScopeSkeletonDiagnosticCode::UnsupportedBinderShape,
                "binder keyword is not followed by a named definition",
                keyword_span,
            );
            return;
        };

        let binding = if name.kind == ScopeSkeletonTokenKind::Word && is_identifier(&name.lexeme) {
            self.advance();
            if self.peek().is_some_and(|token| {
                token.kind == ScopeSkeletonTokenKind::Other && token.lexeme == "="
            }) {
                Some(ScopedBindingShape {
                    spelling: name.lexeme,
                    introduced_at: name.span,
                    kind,
                })
            } else {
                None
            }
        } else {
            None
        };

        if binding.is_none() {
            self.diagnostic(
                ScopeSkeletonDiagnosticCode::UnsupportedBinderShape,
                "binder keyword is not followed by a supported `name =` shape",
                name.span,
            );
        }

        let statement_end = self.recover_to_binder_statement_end();
        if let Some(binding) = binding {
            self.statements.push(LexicalStatementRange {
                kind: LexicalStatementKind::Binder,
                range: SourceSpan {
                    start: keyword_span.start,
                    end: statement_end,
                },
            });
            self.extend_current_or_statement(keyword_span.start, statement_end, vec![binding]);
        }
    }

    fn parse_reconsider_binders(&mut self, keyword_span: SourceSpan) {
        let mut bindings = Vec::new();
        let mut saw_malformed_separator = false;

        while let Some(token) = self.peek().cloned() {
            if token.kind == ScopeSkeletonTokenKind::Semicolon || token_is_block_boundary(&token) {
                break;
            }
            if token.kind == ScopeSkeletonTokenKind::Word && token.lexeme == "as" {
                break;
            }

            if token.kind == ScopeSkeletonTokenKind::Comma {
                saw_malformed_separator = true;
                self.advance();
                continue;
            }

            if token.kind == ScopeSkeletonTokenKind::Word && is_identifier(&token.lexeme) {
                let token = self.advance().expect("peeked token exists");
                bindings.push(ScopedBindingShape {
                    spelling: token.lexeme,
                    introduced_at: token.span,
                    kind: BindingShapeKind::Reconsider,
                });
                self.skip_reconsider_item_tail();
                continue;
            }

            break;
        }

        if bindings.is_empty() && saw_malformed_separator {
            let span = self.peek().map_or(keyword_span, |token| token.span);
            self.diagnostic(
                ScopeSkeletonDiagnosticCode::MalformedBinderList,
                "malformed reconsider binder list was under-approximated",
                span,
            );
        }

        let statement_end = self.recover_to_binder_statement_end();
        if bindings.is_empty() {
            return;
        }
        self.statements.push(LexicalStatementRange {
            kind: LexicalStatementKind::Binder,
            range: SourceSpan {
                start: keyword_span.start,
                end: statement_end,
            },
        });
        self.extend_current_or_statement(keyword_span.start, statement_end, bindings);
    }

    fn skip_reconsider_item_tail(&mut self) {
        if !self
            .peek()
            .is_some_and(|token| token.kind == ScopeSkeletonTokenKind::Other && token.lexeme == "=")
        {
            return;
        }

        self.advance();
        let mut depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        while let Some(token) = self.peek().cloned() {
            if token.kind == ScopeSkeletonTokenKind::Semicolon || token_is_block_boundary(&token) {
                break;
            }
            let top_level = depth == 0 && bracket_depth == 0 && brace_depth == 0;
            if top_level
                && ((token.kind == ScopeSkeletonTokenKind::Word && token.lexeme == "as")
                    || token.kind == ScopeSkeletonTokenKind::Comma)
            {
                break;
            }

            match token.kind {
                ScopeSkeletonTokenKind::LParen => {
                    depth += 1;
                    self.advance();
                }
                ScopeSkeletonTokenKind::RParen => {
                    depth = depth.saturating_sub(1);
                    self.advance();
                }
                ScopeSkeletonTokenKind::LBracket => {
                    bracket_depth += 1;
                    self.advance();
                }
                ScopeSkeletonTokenKind::RBracket => {
                    bracket_depth = bracket_depth.saturating_sub(1);
                    self.advance();
                }
                ScopeSkeletonTokenKind::LBrace => {
                    brace_depth += 1;
                    self.advance();
                }
                ScopeSkeletonTokenKind::RBrace => {
                    brace_depth = brace_depth.saturating_sub(1);
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn should_parse_set_named_equals_binder(&self) -> bool {
        self.named_equals_binder_shape_follows_keyword()
            || self.cursor == 0
            || self
                .tokens
                .get(self.cursor.saturating_sub(1))
                .is_some_and(|token| {
                    token.kind == ScopeSkeletonTokenKind::Semicolon
                        || token_is_block_boundary(token)
                })
    }

    fn named_equals_binder_shape_follows_keyword(&self) -> bool {
        self.tokens
            .get(self.cursor + 1)
            .is_some_and(|token| token.kind == ScopeSkeletonTokenKind::Word)
            && self.tokens.get(self.cursor + 2).is_some_and(|token| {
                token.kind == ScopeSkeletonTokenKind::Other && token.lexeme == "="
            })
    }

    fn tokens_until_stop_contain_word(&self, spelling: &str) -> bool {
        let mut cursor = self.cursor;
        while let Some(token) = self.tokens.get(cursor) {
            if token.kind == ScopeSkeletonTokenKind::Semicolon
                || (token.kind == ScopeSkeletonTokenKind::Word
                    && matches!(
                        token.lexeme.as_str(),
                        "algorithm"
                            | "definition"
                            | "proof"
                            | "now"
                            | "case"
                            | "suppose"
                            | "hereby"
                            | "end"
                    ))
            {
                return false;
            }
            if token.kind == ScopeSkeletonTokenKind::Word && token.lexeme == spelling {
                return true;
            }
            cursor += 1;
        }
        false
    }

    fn processed_binder_before_block(&self) -> Option<ScopedBindingShape> {
        let mut cursor = self.cursor;
        while let Some(token) = self.tokens.get(cursor) {
            if token.kind == ScopeSkeletonTokenKind::Semicolon || token_is_block_boundary(token) {
                return None;
            }
            if token.kind == ScopeSkeletonTokenKind::Word && token.lexeme == "processed" {
                let next = self.tokens.get(cursor + 1)?;
                if next.kind == ScopeSkeletonTokenKind::Word && is_identifier(&next.lexeme) {
                    return Some(ScopedBindingShape {
                        spelling: next.lexeme.clone(),
                        introduced_at: next.span,
                        kind: BindingShapeKind::Processed,
                    });
                }
                return None;
            }
            cursor += 1;
        }
        None
    }

    fn push_statement_frame(
        &mut self,
        start: SourcePos,
        end: SourcePos,
        bindings: Vec<ScopedBindingShape>,
    ) {
        let bindings = self.deduplicate_bindings(bindings, []);
        if bindings.is_empty() {
            return;
        }
        self.frames.push(LexicalScopeFrame {
            range: SourceSpan { start, end },
            bindings,
        });
    }

    fn extend_current_or_statement(
        &mut self,
        start: SourcePos,
        end: SourcePos,
        bindings: Vec<ScopedBindingShape>,
    ) {
        if self.stack.is_empty() {
            self.push_statement_frame(start, end, bindings);
        } else {
            let existing = self
                .stack
                .last()
                .expect("stack is not empty")
                .bindings
                .iter()
                .map(|binding| binding.spelling.clone())
                .collect::<Vec<_>>();
            let bindings = self.deduplicate_bindings(bindings, existing);
            self.current_frame_mut().bindings.extend(bindings);
        }
    }

    fn deduplicate_bindings(
        &mut self,
        bindings: Vec<ScopedBindingShape>,
        existing: impl IntoIterator<Item = String>,
    ) -> Vec<ScopedBindingShape> {
        let mut seen = existing.into_iter().collect::<BTreeSet<_>>();
        let mut deduplicated = Vec::new();
        for binding in bindings {
            if !seen.insert(binding.spelling.clone()) {
                self.diagnostic(
                    ScopeSkeletonDiagnosticCode::DuplicateBindingName,
                    "duplicate binding name in the same lexical scope was ignored",
                    binding.introduced_at,
                );
                continue;
            }
            deduplicated.push(binding);
        }
        deduplicated
    }

    fn parse_ghost_binders(&mut self, ghost_span: SourceSpan) {
        let Some(token) = self.peek().cloned() else {
            self.diagnostic(
                ScopeSkeletonDiagnosticCode::UnsupportedBinderShape,
                "`ghost` is not followed by a supported algorithm binding form",
                ghost_span,
            );
            return;
        };

        match token.lexeme.as_str() {
            "var" => {
                self.advance();
                self.parse_binders(BindingShapeKind::Var, token.span);
            }
            "const" => {
                self.advance();
                self.parse_binders(BindingShapeKind::Const, token.span);
            }
            _ => {
                self.diagnostic(
                    ScopeSkeletonDiagnosticCode::UnsupportedBinderShape,
                    "`ghost` is not followed by `var` or `const`",
                    token.span,
                );
            }
        }
    }

    fn recover_to_binder_statement_end(&mut self) -> SourcePos {
        while let Some(token) = self.peek() {
            if token.kind == ScopeSkeletonTokenKind::Semicolon {
                return token.span.end;
            }
            if token_is_block_boundary(token) {
                return token.span.start;
            }
            self.advance();
        }
        self.source_end
    }

    fn current_frame_mut(&mut self) -> &mut OpenScopeFrame {
        self.stack.last_mut().unwrap_or(&mut self.root)
    }

    fn diagnostic(
        &mut self,
        code: ScopeSkeletonDiagnosticCode,
        message: impl Into<String>,
        span: SourceSpan,
    ) {
        self.diagnostics.push(ScopeSkeletonDiagnostic {
            code,
            message: message.into(),
            span,
        });
    }

    fn peek(&self) -> Option<&ScopeSkeletonToken> {
        self.tokens.get(self.cursor)
    }

    fn advance(&mut self) -> Option<ScopeSkeletonToken> {
        let token = self.tokens.get(self.cursor).cloned();
        if token.is_some() {
            self.cursor += 1;
        }
        token
    }
}

fn binder_list_stop_word(value: &str) -> bool {
    matches!(
        value,
        "be" | "being"
            | "as"
            | "for"
            | "in"
            | "to"
            | "downto"
            | "where"
            | "st"
            | "such"
            | "that"
            | "holds"
            | "do"
            | "proof"
            | "definition"
            | "now"
            | "end"
    )
}

fn token_is_block_boundary(token: &ScopeSkeletonToken) -> bool {
    token.kind == ScopeSkeletonTokenKind::Word
        && matches!(
            token.lexeme.as_str(),
            "algorithm"
                | "definition"
                | "proof"
                | "now"
                | "case"
                | "suppose"
                | "hereby"
                | "do"
                | "end"
        )
}

fn split_scope_skeleton_tokens(raw: &RawTokenStream) -> Vec<ScopeSkeletonToken> {
    let mut tokens = Vec::new();
    for raw_token in &raw.tokens {
        if raw_token.kind == RawTokenKind::Layout {
            continue;
        }
        match raw_token.kind {
            RawTokenKind::LexemeRun => split_lexeme_run_for_scope(raw_token, &mut tokens),
            RawTokenKind::NumeralLike | RawTokenKind::AnnotationMarker | RawTokenKind::Error => {
                tokens.push(ScopeSkeletonToken {
                    kind: ScopeSkeletonTokenKind::Other,
                    lexeme: raw_token.lexeme.clone(),
                    span: raw_token.span,
                });
            }
            RawTokenKind::Layout => {}
        }
    }
    tokens
}

fn split_lexeme_run_for_scope(raw_token: &RawToken, tokens: &mut Vec<ScopeSkeletonToken>) {
    let mut cursor = 0;
    let bytes = raw_token.lexeme.as_bytes();
    while cursor < bytes.len() {
        let start = cursor;
        let ch = raw_token.lexeme[cursor..]
            .chars()
            .next()
            .expect("cursor is inside string");

        if is_identifier_start(ch) {
            cursor += ch.len_utf8();
            while cursor < bytes.len() {
                let next = raw_token.lexeme[cursor..]
                    .chars()
                    .next()
                    .expect("cursor is inside string");
                if !is_identifier_continue(next) {
                    break;
                }
                cursor += next.len_utf8();
            }
            push_scope_piece(
                raw_token,
                tokens,
                ScopeSkeletonTokenKind::Word,
                start,
                cursor,
            );
            continue;
        }

        match ch {
            ',' => {
                cursor += 1;
                push_scope_piece(
                    raw_token,
                    tokens,
                    ScopeSkeletonTokenKind::Comma,
                    start,
                    cursor,
                );
            }
            '(' => {
                cursor += 1;
                push_scope_piece(
                    raw_token,
                    tokens,
                    ScopeSkeletonTokenKind::LParen,
                    start,
                    cursor,
                );
            }
            ')' => {
                cursor += 1;
                push_scope_piece(
                    raw_token,
                    tokens,
                    ScopeSkeletonTokenKind::RParen,
                    start,
                    cursor,
                );
            }
            '[' => {
                cursor += 1;
                push_scope_piece(
                    raw_token,
                    tokens,
                    ScopeSkeletonTokenKind::LBracket,
                    start,
                    cursor,
                );
            }
            ']' => {
                cursor += 1;
                push_scope_piece(
                    raw_token,
                    tokens,
                    ScopeSkeletonTokenKind::RBracket,
                    start,
                    cursor,
                );
            }
            '{' => {
                cursor += 1;
                push_scope_piece(
                    raw_token,
                    tokens,
                    ScopeSkeletonTokenKind::LBrace,
                    start,
                    cursor,
                );
            }
            '}' => {
                cursor += 1;
                push_scope_piece(
                    raw_token,
                    tokens,
                    ScopeSkeletonTokenKind::RBrace,
                    start,
                    cursor,
                );
            }
            ';' => {
                cursor += 1;
                push_scope_piece(
                    raw_token,
                    tokens,
                    ScopeSkeletonTokenKind::Semicolon,
                    start,
                    cursor,
                );
            }
            _ => {
                cursor += ch.len_utf8();
                while cursor < bytes.len() {
                    let next = raw_token.lexeme[cursor..]
                        .chars()
                        .next()
                        .expect("cursor is inside string");
                    if is_identifier_start(next)
                        || matches!(next, ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}')
                    {
                        break;
                    }
                    cursor += next.len_utf8();
                }
                push_scope_piece(
                    raw_token,
                    tokens,
                    ScopeSkeletonTokenKind::Other,
                    start,
                    cursor,
                );
            }
        }
    }
}

fn push_scope_piece(
    raw_token: &RawToken,
    tokens: &mut Vec<ScopeSkeletonToken>,
    kind: ScopeSkeletonTokenKind,
    start: usize,
    end: usize,
) {
    tokens.push(ScopeSkeletonToken {
        kind,
        lexeme: raw_token.lexeme[start..end].to_owned(),
        span: SourceSpan {
            start: raw_token.span.start + start,
            end: raw_token.span.start + end,
        },
    });
}
