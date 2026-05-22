use crate::source::SourceSpan;
use crate::tables::is_reserved_word;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTokenStream {
    pub tokens: Vec<RawToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawToken {
    pub kind: RawTokenKind,
    pub lexeme: String,
    pub span: SourceSpan,
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
pub struct LexError {
    message: String,
}

impl LexError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for LexError {}
pub fn scan_raw(input: &str) -> Result<RawTokenStream, LexError> {
    let mut tokens = Vec::new();
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
                    return Err(LexError::new(format!(
                        "unsupported annotation marker at byte {start}"
                    )));
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

        return Err(LexError::new(format!(
            "unsupported raw lexer input at byte {start}: {ch:?}"
        )));
    }

    Ok(RawTokenStream { tokens })
}
fn raw_token(input: &str, kind: RawTokenKind, start: usize, end: usize) -> RawToken {
    RawToken {
        kind,
        lexeme: input[start..end].to_owned(),
        span: SourceSpan { start, end },
    }
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
