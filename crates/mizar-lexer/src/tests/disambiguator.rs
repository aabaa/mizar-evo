use super::common::*;

#[test]
fn parser_facing_token_types_expose_stable_accessors() {
    let token = Token::new(
        TokenKind::Identifier,
        "alpha",
        SourceSpan::new(0, "alpha".len()),
    );
    let diagnostic = crate::LexDiagnostic::new(
        LexDiagnosticCode::NoValidTokenCandidate,
        "no candidate",
        SourceSpan::new(6, 7),
    );
    let stream = crate::TokenStream::new(vec![token.clone()], vec![diagnostic.clone()]);

    assert_eq!(token.kind(), TokenKind::Identifier);
    assert_eq!(token.lexeme(), "alpha");
    assert_eq!(token.span(), SourceSpan::new(0, 5));
    assert_eq!(diagnostic.code(), LexDiagnosticCode::NoValidTokenCandidate);
    assert_eq!(diagnostic.message(), "no candidate");
    assert_eq!(diagnostic.span(), SourceSpan::new(6, 7));
    assert_eq!(diagnostic.payload(), &LexDiagnosticPayload::None);
    assert_eq!(stream.tokens(), &[token.clone()]);
    assert_eq!(stream.diagnostics(), &[diagnostic.clone()]);
    assert_eq!(stream.into_parts(), (vec![token], vec![diagnostic]));
}

#[test]
fn disambiguator_prefers_longest_user_symbol_inside_raw_runs() {
    let env = build_lexical_environment(
        &[resolved_import("std.ops")],
        &[summary(
            "std.ops",
            71,
            &[
                exported("+", "std.ops#plus", "std.ops", 0),
                exported("+*", "std.ops#plus_star", "std.ops", 1),
                exported("+*+", "std.ops#plus_star_plus", "std.ops", 2),
            ],
        )],
    )
    .expect("environment should build");
    let raw = scan_raw("x+*+y").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let stream = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);

    assert_eq!(
        stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.lexeme.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (TokenKind::Identifier, "x"),
            (TokenKind::UserSymbol, "+*+"),
            (TokenKind::Identifier, "y"),
        ]
    );
    assert!(stream.diagnostics.is_empty());
}

#[test]
fn disambiguator_distinguishes_identifier_symbols_from_scoped_bindings() {
    let env = build_lexical_environment(
        &[resolved_import("std.names")],
        &[summary(
            "std.names",
            72,
            &[exported("succ", "std.names#succ", "std.names", 0)],
        )],
    )
    .expect("environment should build");
    let raw = scan_raw("succ\ndefinition\nlet succ be set;\nsucc;\nend;")
        .expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let stream = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);

    assert_eq!(
        stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.lexeme.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (TokenKind::UserSymbol, "succ"),
            (TokenKind::ReservedWord, "definition"),
            (TokenKind::ReservedWord, "let"),
            (TokenKind::UserSymbol, "succ"),
            (TokenKind::ReservedWord, "be"),
            (TokenKind::ReservedWord, "set"),
            (TokenKind::ReservedSymbol, ";"),
            (TokenKind::Identifier, "succ"),
            (TokenKind::ReservedSymbol, ";"),
            (TokenKind::ReservedWord, "end"),
            (TokenKind::ReservedSymbol, ";"),
        ]
    );
    assert!(stream.diagnostics.is_empty());
}

#[test]
fn disambiguator_emits_reserved_words_symbols_and_namespace_dots() {
    let env = build_lexical_environment(
        &[resolved_import("std.application")],
        &[summary(
            "std.application",
            73,
            &[
                exported(".", "std.application#dot", "std.application", 0),
                exported("B", "std.application#B", "std.application", 1),
            ],
        )],
    )
    .expect("environment should build");
    let raw = scan_raw("A.B").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let namespace_stream = disambiguate(&raw, &env, &ParserLexContext::namespace_path(), &skeleton);
    let general_stream = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);

    assert_eq!(
        namespace_stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.lexeme.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (TokenKind::Identifier, "A"),
            (TokenKind::ReservedSymbol, "."),
            (TokenKind::Identifier, "B"),
        ]
    );
    assert_eq!(
        general_stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.lexeme.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (TokenKind::Identifier, "A"),
            (TokenKind::UserSymbol, "."),
            (TokenKind::UserSymbol, "B"),
        ]
    );
    assert!(namespace_stream.diagnostics.is_empty());
    assert!(general_stream.diagnostics.is_empty());
}

#[test]
fn disambiguator_leaves_same_import_overloads_for_later_resolution() {
    let env = build_lexical_environment(
        &[resolved_import("std.overloaded")],
        &[summary(
            "std.overloaded",
            74,
            &[
                exported("+", "std.overloaded#plus_nat", "std.overloaded", 0),
                exported("+", "std.overloaded#plus_real", "std.overloaded", 1),
            ],
        )],
    )
    .expect("environment should build");
    let raw = scan_raw("x+y").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let stream = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);

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

#[test]
fn disambiguator_filters_user_symbols_by_parser_kind_context() {
    let env = build_lexical_environment(
        &[resolved_import("std.kinds")],
        &[summary(
            "std.kinds",
            75,
            &[
                exported_with_metadata(
                    "Pred",
                    "std.kinds#Pred",
                    "std.kinds",
                    0,
                    UserSymbolKind::Predicate,
                    UserSymbolArity::exact(2),
                ),
                exported_with_metadata(
                    "Func",
                    "std.kinds#Func",
                    "std.kinds",
                    1,
                    UserSymbolKind::Functor,
                    UserSymbolArity::exact(1),
                ),
            ],
        )],
    )
    .expect("environment should build");
    let raw = scan_raw("Pred Func").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let context = ParserLexContext::general()
        .with_user_symbol_kinds(UserSymbolKindSet::only(UserSymbolKind::Predicate));
    let stream = disambiguate(&raw, &env, &context, &skeleton);

    assert_eq!(
        stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.lexeme.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (TokenKind::UserSymbol, "Pred"),
            (TokenKind::Identifier, "Func"),
        ]
    );
    assert!(stream.diagnostics.is_empty());
}

#[test]
fn disambiguator_filters_same_spelling_overloads_by_parser_kind_context() {
    let env = build_lexical_environment(
        &[resolved_import("std.kind_overloads")],
        &[summary(
            "std.kind_overloads",
            76,
            &[
                exported_with_metadata(
                    "op",
                    "std.kind_overloads#op_functor",
                    "std.kind_overloads",
                    0,
                    UserSymbolKind::Functor,
                    UserSymbolArity::exact(1),
                ),
                exported_with_metadata(
                    "op",
                    "std.kind_overloads#op_predicate",
                    "std.kind_overloads",
                    1,
                    UserSymbolKind::Predicate,
                    UserSymbolArity::exact(2),
                ),
            ],
        )],
    )
    .expect("same-spelling overloads with distinct kinds should build");
    let raw = scan_raw("op").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let predicate_context = ParserLexContext::general()
        .with_user_symbol_kinds(UserSymbolKindSet::only(UserSymbolKind::Predicate));
    let mode_context = ParserLexContext::general()
        .with_user_symbol_kinds(UserSymbolKindSet::only(UserSymbolKind::Mode));
    let predicate_stream = disambiguate(&raw, &env, &predicate_context, &skeleton);
    let mode_stream = disambiguate(&raw, &env, &mode_context, &skeleton);

    assert_eq!(
        predicate_stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.lexeme.as_str()))
            .collect::<Vec<_>>(),
        vec![(TokenKind::UserSymbol, "op")]
    );
    assert_eq!(
        mode_stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.lexeme.as_str()))
            .collect::<Vec<_>>(),
        vec![(TokenKind::Identifier, "op")]
    );
    assert!(predicate_stream.diagnostics.is_empty());
    assert!(mode_stream.diagnostics.is_empty());
}

#[test]
fn disambiguator_recognizes_strings_only_when_required() {
    let env = build_lexical_environment(&[], &[]).expect("environment should build");
    let raw = scan_raw("\"abc\"").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let string_stream = disambiguate(&raw, &env, &ParserLexContext::string_required(), &skeleton);
    let general_stream = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);

    assert_eq!(
        string_stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.lexeme.as_str()))
            .collect::<Vec<_>>(),
        vec![(TokenKind::StringLiteral, "\"abc\"")]
    );
    assert_eq!(
        general_stream
            .tokens
            .iter()
            .map(|token| token.kind)
            .collect::<Vec<_>>(),
        vec![
            TokenKind::ErrorRecovery,
            TokenKind::Identifier,
            TokenKind::ErrorRecovery,
        ]
    );
    assert_eq!(
        general_stream
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            LexDiagnosticCode::ParserContextRejectedCandidate,
            LexDiagnosticCode::NoValidTokenCandidate,
        ]
    );
    assert!(string_stream.diagnostics.is_empty());
}

#[test]
fn disambiguator_preserves_final_token_spans() {
    let env = build_lexical_environment(
        &[resolved_import("std.symbols")],
        &[summary(
            "std.symbols",
            88,
            &[exported("*+", "std.symbols#star_plus", "std.symbols", 0)],
        )],
    )
    .expect("environment should build");
    let raw = scan_raw("x*+y").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let stream = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);

    assert_eq!(
        stream.tokens,
        vec![
            token(TokenKind::Identifier, "x", 0, 1),
            token(TokenKind::UserSymbol, "*+", 1, 3),
            token(TokenKind::Identifier, "y", 3, 4),
        ]
    );
    assert!(stream.diagnostics.is_empty());
}

#[test]
fn disambiguator_reports_context_rejection_stably() {
    let env = build_lexical_environment(&[], &[]).expect("environment should build");
    let raw = scan_raw(":").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let stream = disambiguate(
        &raw,
        &env,
        &ParserLexContext::identifier_required(),
        &skeleton,
    );

    assert_eq!(
        stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.lexeme.as_str()))
            .collect::<Vec<_>>(),
        vec![(TokenKind::ErrorRecovery, ":")]
    );
    assert_eq!(
        stream
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![LexDiagnosticCode::ParserContextRejectedCandidate]
    );
    assert_eq!(stream.tokens[0].span, stream.diagnostics[0].span);
    assert_eq!(
        stream.diagnostics[0].payload,
        LexDiagnosticPayload::ParserContextRejectedCandidate {
            mode: ParserLexMode::IdentifierRequired,
            rejected_lexeme: ":".to_owned(),
            candidates: vec![RejectedTokenCandidate {
                kind: TokenKind::ReservedSymbol,
                lexeme: ":".to_owned(),
                span: SourceSpan { start: 0, end: 1 },
            }],
            recovery: LexRecoveryHint::EmitErrorRecoveryToken,
        }
    );
}

#[test]
fn disambiguator_diagnostics_carry_structured_recovery_payloads() {
    let env = build_lexical_environment(&[], &[]).expect("environment should build");

    let raw = scan_raw("\"").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let no_candidate = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);
    assert_eq!(
        no_candidate.diagnostics[0].payload,
        LexDiagnosticPayload::NoValidTokenCandidate {
            rejected_lexeme: "\"".to_owned(),
            recovery: LexRecoveryHint::EmitErrorRecoveryToken,
        }
    );

    let raw = scan_raw("\"bad\\n\"").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let malformed = disambiguate(&raw, &env, &ParserLexContext::string_required(), &skeleton);
    assert_eq!(
        malformed.diagnostics[0].payload,
        LexDiagnosticPayload::MalformedStringLiteral {
            opening_quote: '"',
            reason: MalformedStringLiteralReason::UnsupportedEscape { escape: 'n' },
            recovery: LexRecoveryHint::EmitErrorRecoveryToken,
        }
    );

    for (source, reason) in [
        (
            "\"unterminated",
            MalformedStringLiteralReason::MissingClosingQuote,
        ),
        ("\"dangling\\", MalformedStringLiteralReason::DanglingEscape),
    ] {
        let raw = scan_raw(source).expect("source should raw scan");
        let skeleton = build_scope_skeleton(&raw);
        let malformed = disambiguate(&raw, &env, &ParserLexContext::string_required(), &skeleton);

        assert_eq!(
            malformed.diagnostics[0].payload,
            LexDiagnosticPayload::MalformedStringLiteral {
                opening_quote: '"',
                reason,
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            },
            "{source:?}"
        );
    }

    let raw = scan_raw("\"abc\"").expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);
    let rejected = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);
    assert_eq!(
        rejected.diagnostics[0].payload,
        LexDiagnosticPayload::ParserContextRejectedCandidate {
            mode: ParserLexMode::General,
            rejected_lexeme: "\"".to_owned(),
            candidates: vec![RejectedTokenCandidate {
                kind: TokenKind::StringLiteral,
                lexeme: "\"abc\"".to_owned(),
                span: SourceSpan { start: 0, end: 5 },
            }],
            recovery: LexRecoveryHint::EmitErrorRecoveryToken,
        }
    );
}

#[test]
fn disambiguator_unsupported_raw_token_payload_identifies_raw_token() {
    let env = build_lexical_environment(&[], &[]).expect("environment should build");
    let raw = crate::RawTokenStream::new(vec![RawToken::new(
        RawTokenKind::Error,
        "\u{000b}",
        SourceSpan::new(0, 1),
    )]);
    let skeleton = build_scope_skeleton(&raw);
    let stream = disambiguate(&raw, &env, &ParserLexContext::general(), &skeleton);

    assert_eq!(
        stream.diagnostics[0].payload,
        LexDiagnosticPayload::UnsupportedRawToken {
            raw_kind: RawTokenKind::Error,
            raw_lexeme: "\u{000b}".to_owned(),
            recovery: LexRecoveryHint::EmitErrorRecoveryToken,
        }
    );
}
