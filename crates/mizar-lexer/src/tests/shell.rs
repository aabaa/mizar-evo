use super::common::*;

#[test]
fn lexes_alpha_as_identifier() {
    let tokens = lex("alpha").expect("alpha should lex as an identifier");

    assert_eq!(tokens, vec![token(TokenKind::Identifier, "alpha", 0, 5)]);
}

#[test]
fn lexes_identifier_body_characters() {
    let tokens = lex("_alpha1'").expect("identifier body characters should be supported");

    assert_eq!(tokens, vec![token(TokenKind::Identifier, "_alpha1'", 0, 8)]);
}

#[test]
fn lexes_whitespace_separated_identifiers() {
    let tokens = lex("alpha beta\tgamma\n_delta").expect("identifiers should lex");

    assert_eq!(
        tokens,
        vec![
            token(TokenKind::Identifier, "alpha", 0, 5),
            token(TokenKind::Identifier, "beta", 6, 10),
            token(TokenKind::Identifier, "gamma", 11, 16),
            token(TokenKind::Identifier, "_delta", 17, 23),
        ]
    );
}

#[test]
fn keeps_digit_leading_symbol_shapes_unsplit() {
    let tokens = lex("1alpha").expect("digit-leading symbol shape should lex");

    assert_eq!(tokens, vec![token(TokenKind::LexemeRun, "1alpha", 0, 6)]);
}

#[test]
fn keeps_symbol_shaped_raw_runs_unsplit() {
    let tokens = lex("alpha:=beta").expect("symbol-shaped raw run should lex");

    assert_eq!(
        tokens,
        vec![token(TokenKind::LexemeRun, "alpha:=beta", 0, 11)]
    );
}

#[test]
fn recognizes_reserved_word_table_entries() {
    for word in RESERVED_WORDS {
        assert!(is_reserved_word(word), "{word} should be reserved");
        assert!(!is_identifier(word), "{word} should not be an identifier");
        assert_eq!(
            lex(word).expect("reserved word should lex"),
            vec![token(TokenKind::ReservedWord, word, 0, word.len())]
        );
    }
}

#[test]
fn recognizes_reserved_symbol_table_entries() {
    for symbol in RESERVED_SYMBOLS {
        assert!(is_reserved_symbol(symbol), "{symbol} should be reserved");
        assert_eq!(
            lex(symbol).expect("reserved symbol should lex"),
            vec![token(TokenKind::ReservedSymbol, symbol, 0, symbol.len())]
        );
    }
}

#[test]
fn reserved_words_are_case_sensitive() {
    assert_eq!(
        lex("Theorem").expect("case-distinct spelling should lex"),
        vec![token(TokenKind::Identifier, "Theorem", 0, 7)]
    );
}

#[test]
fn helper_recognizes_numerals() {
    assert!(is_numeral("42"));
    assert!(!is_numeral(""));
    assert!(!is_numeral("42abc"));
}
