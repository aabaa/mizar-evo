use crate::source::SourceSpan;
use crate::tables::is_reserved_word;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTokenStream {
    pub tokens: Vec<RawToken>,
}

impl RawTokenStream {
    pub fn new(tokens: Vec<RawToken>) -> Self {
        Self { tokens }
    }

    pub fn tokens(&self) -> &[RawToken] {
        &self.tokens
    }

    pub fn into_tokens(self) -> Vec<RawToken> {
        self.tokens
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoverableRawTokenStream {
    pub tokens: Vec<RawToken>,
    pub diagnostics: Vec<RawScanDiagnostic>,
}

impl RecoverableRawTokenStream {
    pub fn new(tokens: Vec<RawToken>, diagnostics: Vec<RawScanDiagnostic>) -> Self {
        Self {
            tokens,
            diagnostics,
        }
    }

    pub fn tokens(&self) -> &[RawToken] {
        &self.tokens
    }

    pub fn diagnostics(&self) -> &[RawScanDiagnostic] {
        &self.diagnostics
    }

    pub fn into_parts(self) -> (Vec<RawToken>, Vec<RawScanDiagnostic>) {
        (self.tokens, self.diagnostics)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawToken {
    pub kind: RawTokenKind,
    pub lexeme: String,
    pub span: SourceSpan,
}

impl RawToken {
    pub fn new(kind: RawTokenKind, lexeme: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            span,
        }
    }

    pub const fn kind(&self) -> RawTokenKind {
        self.kind
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawTokenKind {
    LexemeRun,
    NumeralLike,
    AnnotationMarker,
    Layout,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawScanDiagnostic {
    pub code: RawScanDiagnosticCode,
    pub message: String,
    pub span: SourceSpan,
}

impl RawScanDiagnostic {
    pub fn new(code: RawScanDiagnosticCode, message: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }

    pub const fn code(&self) -> RawScanDiagnosticCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RawScanDiagnosticCode {
    UnsupportedAnnotationMarker,
    UnsupportedInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    message: String,
    span: Option<SourceSpan>,
}

impl LexError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
        }
    }

    pub(crate) fn from_raw_scan_diagnostic(diagnostic: RawScanDiagnostic) -> Self {
        Self {
            message: diagnostic.message,
            span: Some(diagnostic.span),
        }
    }

    pub const fn span(&self) -> Option<SourceSpan> {
        self.span
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for LexError {}
pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError> {
    scan_raw_impl(input, RawScanMode::Strict).map(|stream| RawTokenStream::new(stream.tokens))
}

pub fn scan_raw_recoverable(input: &str) -> RecoverableRawTokenStream {
    scan_raw_impl(input, RawScanMode::Recoverable)
        .expect("recoverable raw scanning should not return hard errors")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawScanMode {
    Strict,
    Recoverable,
}

fn scan_raw_impl(input: &str, mode: RawScanMode) -> Result<RecoverableRawTokenStream, LexError> {
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some((start, ch)) = chars.peek().copied() {
        if is_layout(ch) {
            chars.next();
            let mut end = start + ch.len_utf8();

            while let Some((next_start, next_ch)) = chars.peek().copied() {
                if !is_layout(next_ch) {
                    break;
                }

                chars.next();
                end = next_start + next_ch.len_utf8();
            }

            tokens.push(raw_token(input, RawTokenKind::Layout, start, end));
            continue;
        }

        if ch == '@' {
            chars.next();

            let end = match chars.peek().copied() {
                Some((next_start, '[')) => {
                    chars.next();
                    next_start + '['.len_utf8()
                }
                Some((next_start, next_ch)) if is_identifier_start(next_ch) => {
                    chars.next();
                    let mut end = next_start + next_ch.len_utf8();

                    while let Some((body_start, body_ch)) = chars.peek().copied() {
                        if !is_identifier_continue(body_ch) {
                            break;
                        }

                        chars.next();
                        end = body_start + body_ch.len_utf8();
                    }

                    end
                }
                _ => {
                    push_recoverable_error_token(
                        input,
                        &mut tokens,
                        mode,
                        start,
                        start + ch.len_utf8(),
                    );
                    handle_raw_scan_diagnostic(
                        &mut diagnostics,
                        mode,
                        RawScanDiagnostic::new(
                            RawScanDiagnosticCode::UnsupportedAnnotationMarker,
                            format!("unsupported annotation marker at byte {start}"),
                            SourceSpan::new(start, start + ch.len_utf8()),
                        ),
                    )?;
                    continue;
                }
            };

            tokens.push(raw_token(input, RawTokenKind::AnnotationMarker, start, end));
            continue;
        }

        if is_lexeme_run_char(ch) {
            chars.next();
            let mut end = start + ch.len_utf8();

            while let Some((next_start, next_ch)) = chars.peek().copied() {
                if !is_lexeme_run_char(next_ch) {
                    break;
                }

                chars.next();
                end = next_start + next_ch.len_utf8();
            }

            let kind = if input[start..end].chars().all(|ch| ch.is_ascii_digit()) {
                RawTokenKind::NumeralLike
            } else {
                RawTokenKind::LexemeRun
            };
            tokens.push(raw_token(input, kind, start, end));
            continue;
        }

        chars.next();
        push_recoverable_error_token(input, &mut tokens, mode, start, start + ch.len_utf8());
        handle_raw_scan_diagnostic(
            &mut diagnostics,
            mode,
            RawScanDiagnostic::new(
                RawScanDiagnosticCode::UnsupportedInput,
                format!("unsupported raw lexer input at byte {start}: {ch:?}"),
                SourceSpan::new(start, start + ch.len_utf8()),
            ),
        )?;
    }

    Ok(RecoverableRawTokenStream {
        tokens,
        diagnostics,
    })
}

fn push_recoverable_error_token(
    input: &str,
    tokens: &mut Vec<RawToken>,
    mode: RawScanMode,
    start: usize,
    end: usize,
) {
    if mode == RawScanMode::Recoverable {
        tokens.push(raw_token(input, RawTokenKind::Error, start, end));
    }
}

fn handle_raw_scan_diagnostic(
    diagnostics: &mut Vec<RawScanDiagnostic>,
    mode: RawScanMode,
    diagnostic: RawScanDiagnostic,
) -> Result<(), LexError> {
    match mode {
        RawScanMode::Strict => Err(LexError::from_raw_scan_diagnostic(diagnostic)),
        RawScanMode::Recoverable => {
            diagnostics.push(diagnostic);
            Ok(())
        }
    }
}
fn raw_token(input: &str, kind: RawTokenKind, start: usize, end: usize) -> RawToken {
    RawToken::new(kind, &input[start..end], SourceSpan::new(start, end))
}
pub fn is_layout(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n')
}

pub fn is_lexeme_run_char(ch: char) -> bool {
    ch.is_ascii_graphic() && ch != '@'
}

pub fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_continue) && !is_reserved_word(value)
}

pub fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

pub fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '\''
}

pub fn is_numeral(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
}

pub fn is_user_symbol_spelling(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_graphic() && ch != '@')
}

pub fn is_string_literal_spelling(value: &str) -> bool {
    let Some(quote) = value.chars().next() else {
        return false;
    };
    if quote != '"' && quote != '\'' {
        return false;
    }
    let mut chars = value[quote.len_utf8()..].chars();
    let mut escaped = false;
    while let Some(ch) = chars.next() {
        if escaped {
            if !matches!(ch, '"' | '\'' | '\\') {
                return false;
            }
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return chars.next().is_none();
        }
    }
    false
}
