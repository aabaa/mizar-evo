use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
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

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some((start, ch)) = chars.peek().copied() {
        if is_layout(ch) {
            chars.next();
            continue;
        }

        if is_identifier_start(ch) {
            chars.next();
            let mut end = start + ch.len_utf8();

            while let Some((next_start, next_ch)) = chars.peek().copied() {
                if !is_identifier_continue(next_ch) {
                    break;
                }

                chars.next();
                end = next_start + next_ch.len_utf8();
            }

            tokens.push(Token {
                kind: TokenKind::Identifier,
                lexeme: input[start..end].to_owned(),
            });
            continue;
        }

        return Err(LexError::new(format!(
            "unsupported lexer input at byte {start}: {ch:?}"
        )));
    }

    Ok(tokens)
}

fn is_layout(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n')
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '\''
}

#[cfg(test)]
mod tests {
    use super::{Token, TokenKind, lex};

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
    fn rejects_non_spec_layout_characters() {
        assert!(lex("alpha\rbeta").is_err());
    }
}
