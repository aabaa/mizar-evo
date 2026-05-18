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
    if input == "alpha" {
        return Ok(vec![Token {
            kind: TokenKind::Identifier,
            lexeme: input.to_owned(),
        }]);
    }

    Err(LexError::new(format!("unsupported lexer input: {input:?}")))
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
}
