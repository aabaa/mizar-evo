use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
}

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
pub enum RawTokenKind {
    LexemeRun,
    NumeralLike,
    AnnotationMarker,
    Layout,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    message: String,
}

impl LexError {
    fn new(message: impl Into<String>) -> Self {
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

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let raw = scan_raw(input)?;
    let mut tokens = Vec::new();

    for raw_token in raw.tokens {
        match raw_token.kind {
            RawTokenKind::Layout => {}
            RawTokenKind::LexemeRun if is_identifier(&raw_token.lexeme) => {
                tokens.push(Token {
                    kind: TokenKind::Identifier,
                    lexeme: raw_token.lexeme,
                });
            }
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

fn raw_token(input: &str, kind: RawTokenKind, start: usize, end: usize) -> RawToken {
    RawToken {
        kind,
        lexeme: input[start..end].to_owned(),
        span: SourceSpan { start, end },
    }
}

fn is_layout(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n')
}

fn is_lexeme_run_char(ch: char) -> bool {
    ch.is_ascii_graphic() && ch != '@'
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_identifier_start(first) && chars.all(is_identifier_continue)
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '\''
}

#[cfg(test)]
mod tests {
    use super::{RawToken, RawTokenKind, SourceSpan, Token, TokenKind, lex, scan_raw};

    #[test]
    fn lexes_alpha_as_identifier() {
        let tokens = lex("alpha").expect("alpha should lex as an identifier");

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Identifier,
                lexeme: "alpha".to_owned(),
            }]
        );
    }

    #[test]
    fn lexes_identifier_body_characters() {
        let tokens = lex("_alpha1'").expect("identifier body characters should be supported");

        assert_eq!(
            tokens,
            vec![Token {
                kind: TokenKind::Identifier,
                lexeme: "_alpha1'".to_owned(),
            }]
        );
    }

    #[test]
    fn lexes_whitespace_separated_identifiers() {
        let tokens = lex("alpha beta\tgamma\n_delta").expect("identifiers should lex");

        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: TokenKind::Identifier,
                    lexeme: "alpha".to_owned(),
                },
                Token {
                    kind: TokenKind::Identifier,
                    lexeme: "beta".to_owned(),
                },
                Token {
                    kind: TokenKind::Identifier,
                    lexeme: "gamma".to_owned(),
                },
                Token {
                    kind: TokenKind::Identifier,
                    lexeme: "_delta".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn rejects_unsupported_numeral_until_numeral_tokens_exist() {
        assert!(lex("1alpha").is_err());
    }

    #[test]
    fn rejects_unsupported_punctuation_until_final_tokens_exist() {
        assert!(lex("alpha+beta").is_err());
    }

    #[test]
    fn rejects_non_spec_layout_characters() {
        assert!(lex("alpha\rbeta").is_err());
    }

    #[test]
    fn scans_empty_raw_stream() {
        let raw = scan_raw("").expect("empty input should scan");

        assert!(raw.tokens.is_empty());
    }

    #[test]
    fn scans_raw_spans_for_layout_and_runs() {
        let raw = scan_raw("alpha \t\n+").expect("raw input should scan");

        assert_eq!(
            raw.tokens,
            vec![
                RawToken {
                    kind: RawTokenKind::LexemeRun,
                    lexeme: "alpha".to_owned(),
                    span: SourceSpan { start: 0, end: 5 },
                },
                RawToken {
                    kind: RawTokenKind::Layout,
                    lexeme: " \t\n".to_owned(),
                    span: SourceSpan { start: 5, end: 8 },
                },
                RawToken {
                    kind: RawTokenKind::LexemeRun,
                    lexeme: "+".to_owned(),
                    span: SourceSpan { start: 8, end: 9 },
                },
            ]
        );
    }

    #[test]
    fn keeps_digit_leading_mixed_runs_coarse_for_later_disambiguation() {
        let raw = scan_raw("42abc 0*+x").expect("mixed raw input should scan");

        assert_eq!(
            raw.tokens,
            vec![
                RawToken {
                    kind: RawTokenKind::LexemeRun,
                    lexeme: "42abc".to_owned(),
                    span: SourceSpan { start: 0, end: 5 },
                },
                RawToken {
                    kind: RawTokenKind::Layout,
                    lexeme: " ".to_owned(),
                    span: SourceSpan { start: 5, end: 6 },
                },
                RawToken {
                    kind: RawTokenKind::LexemeRun,
                    lexeme: "0*+x".to_owned(),
                    span: SourceSpan { start: 6, end: 10 },
                },
            ]
        );
    }

    #[test]
    fn scans_annotation_markers_without_import_or_parser_context() {
        let raw = scan_raw("@latex @[").expect("annotation marker shapes should scan");

        assert_eq!(
            raw.tokens,
            vec![
                RawToken {
                    kind: RawTokenKind::AnnotationMarker,
                    lexeme: "@latex".to_owned(),
                    span: SourceSpan { start: 0, end: 6 },
                },
                RawToken {
                    kind: RawTokenKind::Layout,
                    lexeme: " ".to_owned(),
                    span: SourceSpan { start: 6, end: 7 },
                },
                RawToken {
                    kind: RawTokenKind::AnnotationMarker,
                    lexeme: "@[".to_owned(),
                    span: SourceSpan { start: 7, end: 9 },
                },
            ]
        );
    }

    #[test]
    fn reports_stable_raw_diagnostics_for_malformed_characters() {
        let error = scan_raw("alpha\rbeta").expect_err("CR is outside lexer layout");

        assert_eq!(
            "unsupported raw lexer input at byte 5: '\\r'",
            error.to_string()
        );
    }

    #[test]
    fn reports_stable_raw_diagnostics_for_malformed_annotation_markers() {
        let error = scan_raw("@-").expect_err("bare annotation marker should be rejected");

        assert_eq!("unsupported annotation marker at byte 0", error.to_string());
    }
}
