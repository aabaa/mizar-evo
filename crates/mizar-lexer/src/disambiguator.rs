use crate::lexical_environment::ActiveLexicalEnvironment;
use crate::raw_lexer::{
    LexError, RawToken, RawTokenKind, RawTokenStream, is_identifier, is_identifier_continue,
    is_identifier_start, scan_raw,
};
use crate::scope_skeleton::ScopeLexView;
use crate::source::{SourceRange, SourceSpan};
use crate::tables::{is_reserved_symbol, is_reserved_word, longest_reserved_symbol_prefix};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<LexDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexDiagnostic {
    pub code: LexDiagnosticCode,
    pub message: String,
    pub span: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexDiagnosticCode {
    NoValidTokenCandidate,
    ParserContextRejectedCandidate,
    AmbiguousUserSymbol,
    MalformedStringLiteral,
    UnsupportedRawToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserLexContext {
    mode: ParserLexMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserLexMode {
    General,
    IdentifierRequired,
    Symbolic,
    StringRequired,
    NamespacePath,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    ReservedWord,
    ReservedSymbol,
    Numeral,
    LexemeRun,
    UserSymbol,
    StringLiteral,
    ErrorRecovery,
}
pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let raw = scan_raw(input)?;
    disambiguate_reserved_shell(&raw)
}

pub fn disambiguate_reserved_shell(raw: &RawTokenStream) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();

    for raw_token in &raw.tokens {
        match raw_token.kind {
            RawTokenKind::Layout => {}
            RawTokenKind::NumeralLike => {
                tokens.push(Token {
                    kind: TokenKind::Numeral,
                    lexeme: raw_token.lexeme.clone(),
                    span: raw_token.span,
                });
            }
            RawTokenKind::AnnotationMarker if raw_token.lexeme == "@[" => {
                tokens.push(Token {
                    kind: TokenKind::ReservedSymbol,
                    lexeme: raw_token.lexeme.clone(),
                    span: raw_token.span,
                });
            }
            RawTokenKind::LexemeRun => tokens.push(classify_lexeme_run_shell(raw_token)),
            _ => {
                return Err(LexError::new(format!(
                    "unsupported lexer token at byte {}: {:?}",
                    raw_token.span.start, raw_token.lexeme
                )));
            }
        }
    }

    Ok(tokens)
}
pub fn disambiguate(
    raw: &RawTokenStream,
    lexical_env: &ActiveLexicalEnvironment,
    parser_context: &ParserLexContext,
    scope_view: &dyn ScopeLexView,
) -> TokenStream {
    Disambiguator::new(raw, lexical_env, *parser_context, scope_view).run()
}
impl ParserLexContext {
    pub fn general() -> Self {
        Self {
            mode: ParserLexMode::General,
        }
    }

    pub fn identifier_required() -> Self {
        Self {
            mode: ParserLexMode::IdentifierRequired,
        }
    }

    pub fn symbolic() -> Self {
        Self {
            mode: ParserLexMode::Symbolic,
        }
    }

    pub fn string_required() -> Self {
        Self {
            mode: ParserLexMode::StringRequired,
        }
    }

    pub fn namespace_path() -> Self {
        Self {
            mode: ParserLexMode::NamespacePath,
        }
    }

    pub fn recovery() -> Self {
        Self {
            mode: ParserLexMode::Recovery,
        }
    }

    pub fn mode(&self) -> ParserLexMode {
        self.mode
    }

    fn admits_identifier(self) -> bool {
        matches!(
            self.mode,
            ParserLexMode::General
                | ParserLexMode::IdentifierRequired
                | ParserLexMode::Symbolic
                | ParserLexMode::NamespacePath
                | ParserLexMode::Recovery
        )
    }

    fn admits_reserved_word(self) -> bool {
        matches!(
            self.mode,
            ParserLexMode::General | ParserLexMode::Symbolic | ParserLexMode::Recovery
        )
    }

    fn admits_symbol(self, spelling: &str) -> bool {
        match self.mode {
            ParserLexMode::General | ParserLexMode::Symbolic | ParserLexMode::Recovery => true,
            ParserLexMode::NamespacePath => spelling == ".",
            ParserLexMode::IdentifierRequired | ParserLexMode::StringRequired => false,
        }
    }

    fn admits_user_symbol(self, _spelling: &str) -> bool {
        match self.mode {
            ParserLexMode::General | ParserLexMode::Symbolic | ParserLexMode::Recovery => true,
            ParserLexMode::NamespacePath => false,
            ParserLexMode::IdentifierRequired | ParserLexMode::StringRequired => false,
        }
    }

    fn admits_numeral(self) -> bool {
        matches!(
            self.mode,
            ParserLexMode::General | ParserLexMode::Symbolic | ParserLexMode::Recovery
        )
    }

    fn requires_string(self) -> bool {
        self.mode == ParserLexMode::StringRequired
    }
}
fn classify_lexeme_run_shell(raw_token: &RawToken) -> Token {
    let kind = if is_reserved_symbol(&raw_token.lexeme) {
        TokenKind::ReservedSymbol
    } else if is_reserved_word(&raw_token.lexeme) {
        TokenKind::ReservedWord
    } else if is_identifier(&raw_token.lexeme) {
        TokenKind::Identifier
    } else {
        TokenKind::LexemeRun
    };

    Token {
        kind,
        lexeme: raw_token.lexeme.clone(),
        span: raw_token.span,
    }
}

struct Disambiguator<'a> {
    raw: &'a RawTokenStream,
    lexical_env: &'a ActiveLexicalEnvironment,
    parser_context: ParserLexContext,
    scope_view: &'a dyn ScopeLexView,
    tokens: Vec<Token>,
    diagnostics: Vec<LexDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DisambiguationCandidate {
    kind: TokenKind,
    len: usize,
    priority: u8,
}

impl<'a> Disambiguator<'a> {
    fn new(
        raw: &'a RawTokenStream,
        lexical_env: &'a ActiveLexicalEnvironment,
        parser_context: ParserLexContext,
        scope_view: &'a dyn ScopeLexView,
    ) -> Self {
        Self {
            raw,
            lexical_env,
            parser_context,
            scope_view,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn run(mut self) -> TokenStream {
        for raw_token in &self.raw.tokens {
            match raw_token.kind {
                RawTokenKind::Layout => {}
                RawTokenKind::NumeralLike => self.disambiguate_numeral_like(raw_token),
                RawTokenKind::LexemeRun => self.disambiguate_lexeme_run(raw_token),
                RawTokenKind::AnnotationMarker if raw_token.lexeme == "@[" => {
                    if self.parser_context.admits_symbol("@[") {
                        self.push_token(
                            TokenKind::ReservedSymbol,
                            &raw_token.lexeme,
                            raw_token.span,
                        );
                    } else {
                        self.push_error(
                            LexDiagnosticCode::ParserContextRejectedCandidate,
                            "parser context rejected annotation symbol",
                            raw_token.span,
                            &raw_token.lexeme,
                        );
                    }
                }
                RawTokenKind::AnnotationMarker | RawTokenKind::Error => {
                    self.push_error(
                        LexDiagnosticCode::UnsupportedRawToken,
                        "raw token cannot be disambiguated",
                        raw_token.span,
                        &raw_token.lexeme,
                    );
                }
            }
        }
        TokenStream {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn disambiguate_numeral_like(&mut self, raw_token: &RawToken) {
        if self.parser_context.admits_numeral() {
            self.push_token(TokenKind::Numeral, &raw_token.lexeme, raw_token.span);
        } else {
            self.push_error(
                LexDiagnosticCode::ParserContextRejectedCandidate,
                "parser context rejected numeral",
                raw_token.span,
                &raw_token.lexeme,
            );
        }
    }

    fn disambiguate_lexeme_run(&mut self, raw_token: &RawToken) {
        let mut cursor = 0;
        while cursor < raw_token.lexeme.len() {
            let starts_string = raw_token.lexeme[cursor..]
                .chars()
                .next()
                .is_some_and(|ch| ch == '"' || ch == '\'');
            if self.parser_context.requires_string() && starts_string {
                match string_literal_prefix_len(&raw_token.lexeme[cursor..]) {
                    Some(len) => {
                        let span = SourceSpan {
                            start: raw_token.span.start + cursor,
                            end: raw_token.span.start + cursor + len,
                        };
                        self.push_token(
                            TokenKind::StringLiteral,
                            &raw_token.lexeme[cursor..cursor + len],
                            span,
                        );
                        cursor += len;
                    }
                    None => {
                        self.push_error(
                            LexDiagnosticCode::MalformedStringLiteral,
                            "malformed string literal",
                            SourceSpan {
                                start: raw_token.span.start + cursor,
                                end: raw_token.span.end,
                            },
                            &raw_token.lexeme[cursor..],
                        );
                        break;
                    }
                }
                continue;
            }

            match self.best_candidate(raw_token, cursor) {
                Some(candidate) => {
                    let span = SourceSpan {
                        start: raw_token.span.start + cursor,
                        end: raw_token.span.start + cursor + candidate.len,
                    };
                    self.push_token(
                        candidate.kind,
                        &raw_token.lexeme[cursor..cursor + candidate.len],
                        span,
                    );
                    cursor += candidate.len;
                }
                None => {
                    let ch = raw_token.lexeme[cursor..]
                        .chars()
                        .next()
                        .expect("cursor is inside lexeme run");
                    let end = cursor + ch.len_utf8();
                    let code = if self.has_context_rejected_candidate(raw_token, cursor) {
                        LexDiagnosticCode::ParserContextRejectedCandidate
                    } else {
                        LexDiagnosticCode::NoValidTokenCandidate
                    };
                    self.push_error(
                        code,
                        "no valid token candidate",
                        SourceSpan {
                            start: raw_token.span.start + cursor,
                            end: raw_token.span.start + end,
                        },
                        &raw_token.lexeme[cursor..end],
                    );
                    cursor = end;
                }
            }
        }
    }

    fn best_candidate(
        &mut self,
        raw_token: &RawToken,
        cursor: usize,
    ) -> Option<DisambiguationCandidate> {
        let mut candidates = Vec::new();
        self.push_user_symbol_candidates(raw_token, cursor, &mut candidates);
        self.push_reserved_symbol_candidate(raw_token, cursor, &mut candidates);
        self.push_word_candidate(raw_token, cursor, &mut candidates);
        self.push_numeral_candidate(raw_token, cursor, &mut candidates);

        candidates.into_iter().max_by(|left, right| {
            left.len
                .cmp(&right.len)
                .then(left.priority.cmp(&right.priority))
        })
    }

    fn push_user_symbol_candidates(
        &mut self,
        raw_token: &RawToken,
        cursor: usize,
        candidates: &mut Vec<DisambiguationCandidate>,
    ) {
        let user_symbols = self
            .lexical_env
            .longest_user_symbol_at(&raw_token.lexeme, cursor);
        if user_symbols.is_empty() {
            return;
        }
        let spelling = &user_symbols[0].spelling;
        let position = raw_token.span.start + cursor;
        if is_identifier(spelling)
            && self.scope_view.binding_overrides_symbol(spelling, position)
            && self.parser_context.admits_identifier()
        {
            candidates.push(DisambiguationCandidate {
                kind: TokenKind::Identifier,
                len: spelling.len(),
                priority: 8,
            });
            return;
        }
        if self.parser_context.admits_user_symbol(spelling) {
            candidates.push(DisambiguationCandidate {
                kind: TokenKind::UserSymbol,
                len: spelling.len(),
                priority: 7,
            });
        }
    }

    fn push_reserved_symbol_candidate(
        &self,
        raw_token: &RawToken,
        cursor: usize,
        candidates: &mut Vec<DisambiguationCandidate>,
    ) {
        if let Some(symbol) = longest_reserved_symbol_prefix(&raw_token.lexeme[cursor..])
            && self.parser_context.admits_symbol(symbol)
        {
            let priority =
                if self.parser_context.mode() == ParserLexMode::NamespacePath && symbol == "." {
                    9
                } else {
                    6
                };
            candidates.push(DisambiguationCandidate {
                kind: TokenKind::ReservedSymbol,
                len: symbol.len(),
                priority,
            });
        }
    }

    fn push_word_candidate(
        &self,
        raw_token: &RawToken,
        cursor: usize,
        candidates: &mut Vec<DisambiguationCandidate>,
    ) {
        let Some(len) = identifier_prefix_len(&raw_token.lexeme[cursor..]) else {
            return;
        };
        let spelling = &raw_token.lexeme[cursor..cursor + len];
        if is_reserved_word(spelling) {
            if self.parser_context.admits_reserved_word() {
                candidates.push(DisambiguationCandidate {
                    kind: TokenKind::ReservedWord,
                    len,
                    priority: 5,
                });
            }
        } else if self.parser_context.admits_identifier() {
            candidates.push(DisambiguationCandidate {
                kind: TokenKind::Identifier,
                len,
                priority: 4,
            });
        }
    }

    fn push_numeral_candidate(
        &self,
        raw_token: &RawToken,
        cursor: usize,
        candidates: &mut Vec<DisambiguationCandidate>,
    ) {
        let Some(len) = numeral_prefix_len(&raw_token.lexeme[cursor..]) else {
            return;
        };
        if self.parser_context.admits_numeral() {
            candidates.push(DisambiguationCandidate {
                kind: TokenKind::Numeral,
                len,
                priority: 3,
            });
        }
    }

    fn has_context_rejected_candidate(&self, raw_token: &RawToken, cursor: usize) -> bool {
        longest_reserved_symbol_prefix(&raw_token.lexeme[cursor..]).is_some()
            || identifier_prefix_len(&raw_token.lexeme[cursor..]).is_some()
            || numeral_prefix_len(&raw_token.lexeme[cursor..]).is_some()
            || !self
                .lexical_env
                .longest_user_symbol_at(&raw_token.lexeme, cursor)
                .is_empty()
    }

    fn push_token(&mut self, kind: TokenKind, lexeme: &str, span: SourceRange) {
        self.tokens.push(Token {
            kind,
            lexeme: lexeme.to_owned(),
            span,
        });
    }

    fn push_error(
        &mut self,
        code: LexDiagnosticCode,
        message: impl Into<String>,
        span: SourceRange,
        lexeme: &str,
    ) {
        self.diagnostics.push(LexDiagnostic {
            code,
            message: message.into(),
            span,
        });
        self.push_token(TokenKind::ErrorRecovery, lexeme, span);
    }
}

fn identifier_prefix_len(value: &str) -> Option<usize> {
    let mut chars = value.char_indices();
    let (_, first) = chars.next()?;
    if !is_identifier_start(first) {
        return None;
    }
    let mut end = first.len_utf8();
    for (index, ch) in chars {
        if !is_identifier_continue(ch) {
            break;
        }
        end = index + ch.len_utf8();
    }
    Some(end)
}

fn numeral_prefix_len(value: &str) -> Option<usize> {
    let mut end = 0;
    for (index, ch) in value.char_indices() {
        if !ch.is_ascii_digit() {
            break;
        }
        end = index + ch.len_utf8();
    }
    (end > 0).then_some(end)
}

fn string_literal_prefix_len(value: &str) -> Option<usize> {
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let mut escaped = false;
    for (index, ch) in value[quote.len_utf8()..].char_indices() {
        if escaped {
            if !matches!(ch, '"' | '\'' | '\\') {
                return None;
            }
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some(quote.len_utf8() + index + ch.len_utf8());
        }
    }
    None
}
