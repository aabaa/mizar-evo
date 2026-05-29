use super::common::*;

#[test]
fn raw_token_types_expose_stable_accessors() {
    let token = RawToken::new(RawTokenKind::LexemeRun, "alpha", SourceSpan::new(0, 5));
    let stream = crate::RawTokenStream::new(vec![token.clone()]);

    assert_eq!(token.kind(), RawTokenKind::LexemeRun);
    assert_eq!(token.lexeme(), "alpha");
    assert_eq!(token.span(), SourceSpan::new(0, 5));
    assert_eq!(stream.tokens(), &[token]);
    assert_eq!(stream.into_tokens().len(), 1);
}

#[test]
fn helpers_recognize_layout_symbol_shapes_and_string_shells() {
    assert!(is_layout(' '));
    assert!(is_layout('\t'));
    assert!(is_layout('\n'));
    assert!(!is_layout('\r'));
    assert!(!is_layout('\u{000b}'));
    assert!(!is_layout('\u{000c}'));

    assert!(is_user_symbol_spelling("*+"));
    assert!(is_user_symbol_spelling("succ"));
    assert!(!is_user_symbol_spelling("@latex"));
    for byte in b'!'..=b'~' {
        let ch = char::from(byte);
        assert_eq!(
            is_user_symbol_spelling(&ch.to_string()),
            ch != '@',
            "{ch:?}"
        );
    }
    for spelling in ["", " ", "\t", "\n", "alpha beta", "@", "é"] {
        assert!(
            !is_user_symbol_spelling(spelling),
            "{spelling:?} should not be a user symbol spelling"
        );
    }

    assert!(is_string_literal_spelling("\"say \\\"hi\\\"\""));
    assert!(is_string_literal_spelling("'say \"hi\"'"));
    assert!(is_string_literal_spelling("\"slash\\\\path\""));
    for spelling in [
        r#""quote\"""#,
        r#""quote\'""#,
        r#""slash\\""#,
        r#"'quote\''"#,
        r#"'quote\"'"#,
        r#"'slash\\'"#,
    ] {
        assert!(
            is_string_literal_spelling(spelling),
            "{spelling:?} should be a valid escaped string literal"
        );
    }
    for spelling in [r#""bad\n""#, r#"'bad\t'"#, "\"dangling\\", "'dangling\\"] {
        assert!(
            !is_string_literal_spelling(spelling),
            "{spelling:?} should reject unsupported or dangling escapes"
        );
    }
    assert!(!is_string_literal_spelling("\"bad\\n\""));
    assert!(!is_string_literal_spelling("\"unterminated"));

    assert_eq!(longest_reserved_symbol_prefix("..."), Some("..."));
    assert_eq!(longest_reserved_symbol_prefix(".{"), Some(".{"));
}

#[test]
fn rejects_non_spec_layout_characters() {
    assert!(lex("alpha\rbeta").is_err());
    assert!(lex("alpha\u{000b}beta").is_err());
    assert!(lex("alpha\u{000c}beta").is_err());
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
fn scans_annotation_names_with_identifier_boundaries() {
    let raw = scan_raw("@proof_hint @show_thesis @bad-name")
        .expect("annotation names should use identifier-shaped spelling");

    assert_eq!(
        raw.tokens,
        vec![
            RawToken {
                kind: RawTokenKind::AnnotationMarker,
                lexeme: "@proof_hint".to_owned(),
                span: SourceSpan { start: 0, end: 11 },
            },
            RawToken {
                kind: RawTokenKind::Layout,
                lexeme: " ".to_owned(),
                span: SourceSpan { start: 11, end: 12 },
            },
            RawToken {
                kind: RawTokenKind::AnnotationMarker,
                lexeme: "@show_thesis".to_owned(),
                span: SourceSpan { start: 12, end: 24 },
            },
            RawToken {
                kind: RawTokenKind::Layout,
                lexeme: " ".to_owned(),
                span: SourceSpan { start: 24, end: 25 },
            },
            RawToken {
                kind: RawTokenKind::AnnotationMarker,
                lexeme: "@bad".to_owned(),
                span: SourceSpan { start: 25, end: 29 },
            },
            RawToken {
                kind: RawTokenKind::LexemeRun,
                lexeme: "-name".to_owned(),
                span: SourceSpan { start: 29, end: 34 },
            },
        ]
    );

    for source in ["@", "@-", "@ name", "@1bad"] {
        assert!(scan_raw(source).is_err(), "{source:?}");
    }
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
fn reports_stable_raw_diagnostics_for_unsupported_ascii_controls() {
    for (name, ch) in [("vertical tab", '\u{000b}'), ("form feed", '\u{000c}')] {
        let source = format!("alpha{ch}beta");
        let preprocessed = preprocess_source_for_lexing(&source);
        let error = match scan_raw(&preprocessed.lexical_text) {
            Ok(_) => panic!("{name} should not be treated as raw layout or token text"),
            Err(error) => error,
        };

        assert_eq!(preprocessed.lexical_text, source, "{name}");
        assert!(preprocessed.comments.is_empty(), "{name}");
        assert!(preprocessed.diagnostics.is_empty(), "{name}");
        assert_eq!(
            format!("unsupported raw lexer input at byte 5: {ch:?}"),
            error.to_string(),
            "{name}"
        );
    }
}

#[test]
fn reports_stable_raw_diagnostics_for_unsupported_unicode_code_characters() {
    for (name, ch) in [
        ("NBSP", '\u{00a0}'),
        ("zero-width space", '\u{200b}'),
        ("zero-width non-joiner", '\u{200c}'),
        ("zero-width joiner", '\u{200d}'),
        ("full-width comma", '\u{ff0c}'),
        ("full-width semicolon", '\u{ff1b}'),
        ("BOM", '\u{feff}'),
    ] {
        let source = format!("alpha{ch}beta");
        let error = match scan_raw(&source) {
            Ok(_) => panic!("{name} should not be treated as raw layout or token text"),
            Err(error) => error,
        };

        assert_eq!(
            format!("unsupported raw lexer input at byte 5: {ch:?}"),
            error.to_string(),
            "{name}"
        );
    }
}

#[test]
fn reports_stable_raw_diagnostics_for_malformed_annotation_markers() {
    let error = scan_raw("@-").expect_err("bare annotation marker should be rejected");

    assert_eq!("unsupported annotation marker at byte 0", error.to_string());
}
