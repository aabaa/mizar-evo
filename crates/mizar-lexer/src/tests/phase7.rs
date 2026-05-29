use super::common::*;

#[test]
fn phase7_raw_scanner_spans_are_contiguous_and_deterministic() {
    let seeds = [
        "",
        "alpha \t\n+*+42",
        "@[ import std.core;\n",
        "\"quoted\" 'tick' |.x.|",
        "definition\nlet x be set;\nend;",
    ];

    for source in seeds {
        let first = scan_raw(source).expect("phase7 seed should raw scan");
        let second = scan_raw(source).expect("phase7 seed should scan deterministically");
        assert_eq!(first, second, "{source:?}");

        let mut cursor = 0;
        let mut reconstructed = String::new();
        for token in &first.tokens {
            assert_eq!(token.span.start, cursor, "{source:?}");
            assert!(token.span.start <= token.span.end, "{source:?}");
            assert_eq!(&source[token.span.start..token.span.end], token.lexeme);
            reconstructed.push_str(&token.lexeme);
            cursor = token.span.end;
        }

        assert_eq!(cursor, source.len(), "{source:?}");
        assert_eq!(reconstructed, source);
        assert_eq!(scan_raw(&reconstructed), Ok(first), "{source:?}");
    }
}

#[test]
fn phase7_final_shell_retokenizes_layout_free_token_concatenation() {
    let seeds = ["alpha", "theorem", "42", "@[", ":="];

    for source in seeds {
        let first = lex(source).expect("phase7 final shell seed should lex");
        let concatenated = first
            .iter()
            .map(|token| token.lexeme.as_str())
            .collect::<String>();
        let source_without_layout = source
            .chars()
            .filter(|ch| !is_layout(*ch))
            .collect::<String>();

        assert_eq!(concatenated, source_without_layout, "{source:?}");
        assert_eq!(
            lex(&concatenated).expect("layout-free token stream should re-lex"),
            first,
            "{source:?}"
        );
    }
}

#[test]
fn phase7_final_shell_spans_point_to_original_spellings() {
    let seeds = [
        "",
        "alpha beta\tgamma\n_delta",
        "theorem iff not",
        "42 007 12345",
        "@[ : .{ .* .= ... ;",
        "alpha:=beta 1alpha |.x.|",
    ];

    for source in seeds {
        let tokens = lex(source).expect("phase7 final shell span seed should lex");

        assert_final_token_spans_point_to_lexemes(source, &tokens);
    }
}

#[test]
fn phase7_disambiguated_final_spans_point_to_original_spellings() {
    let env = build_lexical_environment(
        &[resolved_import("generated.final_spans")],
        &[summary(
            "generated.final_spans",
            111,
            &[
                exported(
                    "+",
                    "generated.final_spans#plus",
                    "generated.final_spans",
                    0,
                ),
                exported(
                    "+*",
                    "generated.final_spans#plus_star",
                    "generated.final_spans",
                    1,
                ),
                exported(
                    "+*+",
                    "generated.final_spans#plus_star_plus",
                    "generated.final_spans",
                    2,
                ),
                exported(
                    "|.x.|",
                    "generated.final_spans#absolute_x",
                    "generated.final_spans",
                    3,
                ),
                exported(
                    "succ",
                    "generated.final_spans#succ",
                    "generated.final_spans",
                    4,
                ),
                exported(".", "generated.final_spans#dot", "generated.final_spans", 5),
                exported("B", "generated.final_spans#B", "generated.final_spans", 6),
                exported(
                    "q\"",
                    "generated.final_spans#quote",
                    "generated.final_spans",
                    7,
                ),
            ],
        )],
    )
    .expect("final span environment should build");
    let seeds = [
        ("x+*+y", ParserLexContext::general()),
        ("A.B", ParserLexContext::namespace_path()),
        ("|.x.|+* q\"", ParserLexContext::general()),
        (
            "succ\ndefinition\nlet succ be set;\nsucc;\nend;",
            ParserLexContext::general(),
        ),
        (r#""abc\"def""#, ParserLexContext::string_required()),
        (r#""bad\n" tail"#, ParserLexContext::string_required()),
        (r#""abc""#, ParserLexContext::general()),
        (":", ParserLexContext::identifier_required()),
        ("@foo", ParserLexContext::general()),
    ];

    for (source, context) in seeds {
        let raw = scan_raw(source).expect("phase7 disambiguator span seed should raw scan");
        let skeleton = build_scope_skeleton(&raw);
        let stream = disambiguate(&raw, &env, &context, &skeleton);

        assert_final_token_spans_point_to_lexemes(source, &stream.tokens);
        for diagnostic in &stream.diagnostics {
            assert!(
                diagnostic.span.start <= diagnostic.span.end,
                "{source:?}: invalid diagnostic span {:?}",
                diagnostic.span
            );
            assert!(
                diagnostic.span.end <= source.len(),
                "{source:?}: out-of-bounds diagnostic span {:?}",
                diagnostic.span
            );
        }
    }
}

#[test]
fn phase7_generated_user_symbol_overlap_matrix_keeps_longest_match() {
    let env = build_lexical_environment(
        &[resolved_import("generated.overlap")],
        &[summary(
            "generated.overlap",
            91,
            &[
                exported("+", "generated.overlap#plus", "generated.overlap", 0),
                exported("+*", "generated.overlap#plus_star", "generated.overlap", 1),
                exported(
                    "+*+",
                    "generated.overlap#plus_star_plus",
                    "generated.overlap",
                    2,
                ),
                exported("|.", "generated.overlap#abs_open", "generated.overlap", 3),
                exported(
                    "[:",
                    "generated.overlap#product_open",
                    "generated.overlap",
                    4,
                ),
                exported(
                    ":]",
                    "generated.overlap#product_close",
                    "generated.overlap",
                    5,
                ),
                exported("f'", "generated.overlap#prime", "generated.overlap", 6),
                exported("q\"", "generated.overlap#quote", "generated.overlap", 7),
                exported(
                    "|.x.|",
                    "generated.overlap#absolute_x",
                    "generated.overlap",
                    8,
                ),
                exported("succ", "generated.overlap#succ", "generated.overlap", 9),
                exported("succ2", "generated.overlap#succ2", "generated.overlap", 10),
            ],
        )],
    )
    .expect("generated overlap environment should build");

    for (source, expected) in [
        (
            "a+*+b",
            vec![
                (TokenKind::Identifier, "a"),
                (TokenKind::UserSymbol, "+*+"),
                (TokenKind::Identifier, "b"),
            ],
        ),
        (
            "|.x.|+*",
            vec![
                (TokenKind::UserSymbol, "|.x.|"),
                (TokenKind::UserSymbol, "+*"),
            ],
        ),
        (
            "succ2+succ",
            vec![
                (TokenKind::UserSymbol, "succ2"),
                (TokenKind::UserSymbol, "+"),
                (TokenKind::UserSymbol, "succ"),
            ],
        ),
        (
            "[:f':]q\"",
            vec![
                (TokenKind::UserSymbol, "[:"),
                (TokenKind::UserSymbol, "f'"),
                (TokenKind::UserSymbol, ":]"),
                (TokenKind::UserSymbol, "q\""),
            ],
        ),
    ] {
        let raw = scan_raw(source).expect("generated source should raw scan");
        let skeleton = build_scope_skeleton(&raw);
        let stream = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);

        assert_eq!(
            stream
                .tokens
                .iter()
                .map(|token| (token.kind, token.lexeme.as_str()))
                .collect::<Vec<_>>(),
            expected,
            "{source:?}"
        );
        assert!(stream.diagnostics.is_empty(), "{source:?}");
    }
}

#[test]
fn phase7_generated_import_conflict_matrix_stays_lexer_local() {
    let conflict = build_lexical_environment(
        &[
            resolved_import("generated.left"),
            resolved_import("generated.right"),
        ],
        &[
            summary(
                "generated.left",
                101,
                &[exported("+", "generated.left#plus", "generated.left", 0)],
            ),
            summary(
                "generated.right",
                102,
                &[exported("+", "generated.right#plus", "generated.right", 0)],
            ),
        ],
    )
    .expect_err("equal spelling from distinct imports is an environment conflict");
    assert!(matches!(
        conflict,
        LexicalEnvironmentError::UserSymbolImportConflict { .. }
    ));

    let env = build_lexical_environment(
        &[resolved_import("generated.same_import")],
        &[summary(
            "generated.same_import",
            103,
            &[
                exported(
                    "+",
                    "generated.same_import#plus_nat",
                    "generated.same_import",
                    0,
                ),
                exported(
                    "+",
                    "generated.same_import#plus_real",
                    "generated.same_import",
                    1,
                ),
            ],
        )],
    )
    .expect("same-import overloads stay in the active lexicon");
    let raw = scan_raw("x+y").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let stream = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);

    assert_eq!(
        env.longest_user_symbol_at("+", 0)
            .iter()
            .map(|candidate| candidate.symbol_id.clone())
            .collect::<Vec<_>>(),
        vec![
            symbol_id("generated.same_import#plus_nat"),
            symbol_id("generated.same_import#plus_real"),
        ]
    );
    assert_eq!(
        stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.lexeme.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (TokenKind::Identifier, "x"),
            (TokenKind::UserSymbol, "+"),
            (TokenKind::Identifier, "y"),
        ]
    );
    assert!(stream.diagnostics.is_empty());
}
