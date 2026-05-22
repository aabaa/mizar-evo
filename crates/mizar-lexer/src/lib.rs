mod disambiguator;
mod import_prescan;
mod lexical_environment;
mod raw_lexer;
mod scope_skeleton;
mod source;
mod tables;

pub use disambiguator::{
    LexDiagnostic, LexDiagnosticCode, ParserLexContext, ParserLexMode, Token, TokenKind,
    TokenStream, disambiguate, disambiguate_reserved_shell, lex,
};
pub use import_prescan::{
    ImportPrelude, ImportPrescanDiagnostic, ImportPrescanDiagnosticCode, ImportStub,
    RawModuleAlias, RawModulePath, RawModulePathComponent, RawModuleRelativePrefix,
    scan_import_prelude,
};
pub use lexical_environment::{
    ActiveLexicalEnvironment, ExportRank, ExportedSymbolShape, LexicalEnvironmentError,
    LexicalEnvironmentFingerprint, LexicalSummaryFingerprint, ModuleId, ModuleLexicalSummary,
    ResolvedImport, SymbolId, UserSymbolCandidate, UserSymbolIndex, build_lexical_environment,
};
pub use raw_lexer::{
    LexError, RawToken, RawTokenKind, RawTokenStream, is_identifier, is_identifier_continue,
    is_identifier_start, is_layout, is_lexeme_run_char, is_numeral, is_string_literal_spelling,
    is_user_symbol_spelling, scan_raw,
};
pub use scope_skeleton::{
    BindingShapeKind, LexicalBlockKind, LexicalBlockRange, LexicalScopeFrame, LexicalStatementKind,
    LexicalStatementRange, ScopeLexView, ScopeSkeleton, ScopeSkeletonDiagnostic,
    ScopeSkeletonDiagnosticCode, ScopedBindingShape, build_scope_skeleton,
};
pub use source::{
    CommentKind, CommentTrivia, ModuleNamingError, ModuleSourceName, PreprocessedLexicalSource,
    SourceLineIndex, SourceLocation, SourceLocationRange, SourcePos, SourcePreprocessDiagnostic,
    SourcePreprocessDiagnosticCode, SourceRange, SourceSpan, module_source_name_from_path,
    preprocess_source_for_lexing,
};
pub use tables::{
    RESERVED_SYMBOLS, RESERVED_WORDS, ReservedSymbolTable, ReservedWordTable, is_reserved_symbol,
    is_reserved_word, longest_reserved_symbol_prefix,
};

#[cfg(test)]
mod tests {
    use super::{
        CommentKind, ExportRank, ExportedSymbolShape, ImportPrescanDiagnosticCode,
        LexDiagnosticCode, LexicalBlockKind, LexicalEnvironmentError, LexicalSummaryFingerprint,
        ModuleId, ModuleLexicalSummary, ModuleNamingError, ParserLexContext, RESERVED_SYMBOLS,
        RESERVED_WORDS, RawModuleRelativePrefix, RawToken, RawTokenKind, ResolvedImport,
        ScopeLexView, ScopeSkeletonDiagnosticCode, SourceLineIndex, SourceLocation,
        SourceLocationRange, SourcePreprocessDiagnosticCode, SourceSpan, SymbolId, Token,
        TokenKind, UserSymbolCandidate, build_lexical_environment, build_scope_skeleton,
        disambiguate, is_identifier, is_layout, is_numeral, is_reserved_symbol, is_reserved_word,
        is_string_literal_spelling, is_user_symbol_spelling, lex, longest_reserved_symbol_prefix,
        module_source_name_from_path, preprocess_source_for_lexing, scan_import_prelude, scan_raw,
    };

    fn token(kind: TokenKind, lexeme: &str, start: usize, end: usize) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_owned(),
            span: SourceSpan { start, end },
        }
    }

    fn assert_final_token_spans_point_to_lexemes(source: &str, tokens: &[Token]) {
        for token in tokens {
            assert!(
                token.span.start <= token.span.end,
                "{source:?}: invalid span {:?} for {:?}",
                token.span,
                token
            );
            assert!(
                token.span.end <= source.len(),
                "{source:?}: out-of-bounds span {:?} for {:?}",
                token.span,
                token
            );
            assert_eq!(
                &source[token.span.start..token.span.end],
                token.lexeme,
                "{source:?}: final token span should point back to its spelling"
            );
        }
    }

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

    #[test]
    fn source_line_index_maps_byte_spans_to_zero_based_locations() {
        let source = "alpha\nβeta\n";
        let index = SourceLineIndex::new(source);

        assert_eq!(
            index.location(0),
            Some(SourceLocation { line: 0, column: 0 })
        );
        assert_eq!(
            index.location(6),
            Some(SourceLocation { line: 1, column: 0 })
        );
        assert_eq!(
            index.location(source.len()),
            Some(SourceLocation { line: 2, column: 0 })
        );
        assert_eq!(
            index.range(SourceSpan { start: 0, end: 11 }),
            Some(SourceLocationRange {
                start: SourceLocation { line: 0, column: 0 },
                end: SourceLocation { line: 1, column: 5 },
            })
        );
        assert_eq!(index.location(source.len() + 1), None);
        assert_eq!(index.range(SourceSpan { start: 2, end: 1 }), None);
    }

    #[test]
    fn source_line_index_rejects_non_utf8_boundary_offsets() {
        let source = "a\nβeta\n漢字";
        let index = SourceLineIndex::new(source);

        assert_eq!(
            index.location(2),
            Some(SourceLocation { line: 1, column: 0 })
        );
        assert_eq!(index.location(3), None);
        assert_eq!(
            index.location(4),
            Some(SourceLocation { line: 1, column: 2 })
        );
        assert_eq!(
            index.location(8),
            Some(SourceLocation { line: 2, column: 0 })
        );
        assert_eq!(index.location(9), None);
        assert_eq!(index.location(10), None);
        assert_eq!(
            index.location(11),
            Some(SourceLocation { line: 2, column: 3 })
        );
        assert_eq!(
            index.location(source.len()),
            Some(SourceLocation { line: 2, column: 6 })
        );
        assert_eq!(
            index.range(SourceSpan { start: 2, end: 5 }),
            Some(SourceLocationRange {
                start: SourceLocation { line: 1, column: 0 },
                end: SourceLocation { line: 1, column: 3 },
            })
        );
        assert_eq!(index.range(SourceSpan { start: 2, end: 3 }), None);
        assert_eq!(index.range(SourceSpan { start: 8, end: 10 }), None);
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

    #[test]
    fn preprocess_source_removes_comments_and_preserves_trivia() {
        let source = "alpha :: comment \u{03b1}\n::: doc \u{03b2}\n::=\nblock \u{03b3}\n=::\nomega";
        let preprocessed = preprocess_source_for_lexing(source);

        assert_eq!(preprocessed.lexical_text, "alpha \n\n\n\n\nomega");
        assert_eq!(
            preprocessed
                .comments
                .iter()
                .map(|comment| comment.kind)
                .collect::<Vec<_>>(),
            vec![
                CommentKind::SingleLine,
                CommentKind::Documentation,
                CommentKind::MultiLine,
            ]
        );
        assert!(preprocessed.diagnostics.is_empty());
        assert!(scan_raw(&preprocessed.lexical_text).is_ok());
    }

    #[test]
    fn preprocess_source_covers_comment_removal_edges() {
        for (name, source, expected_lexical_text, expected_kinds) in [
            (
                "adjacent line comments",
                "alpha:: first\n:: second\nomega",
                "alpha\n\nomega",
                vec![CommentKind::SingleLine, CommentKind::SingleLine],
            ),
            (
                "comment at eof",
                "alpha\n:: eof",
                "alpha\n",
                vec![CommentKind::SingleLine],
            ),
            (
                "inline block comment between token-shaped text",
                "alpha::= hidden =::beta",
                "alpha beta",
                vec![CommentKind::MultiLine],
            ),
            (
                "adjacent inline block comments between token-shaped text",
                "alpha::= first =::::= second =::beta",
                "alpha beta",
                vec![CommentKind::MultiLine, CommentKind::MultiLine],
            ),
            (
                "multi-line comment preserves multiple newlines",
                "alpha::=\nline 1\nline 2\n=::omega",
                "alpha\n\n\nomega",
                vec![CommentKind::MultiLine],
            ),
        ] {
            let preprocessed = preprocess_source_for_lexing(source);

            assert_eq!(preprocessed.lexical_text, expected_lexical_text, "{name}");
            assert_eq!(
                preprocessed
                    .comments
                    .iter()
                    .map(|comment| comment.kind)
                    .collect::<Vec<_>>(),
                expected_kinds,
                "{name}"
            );
            assert!(preprocessed.diagnostics.is_empty(), "{name}");
            assert!(scan_raw(&preprocessed.lexical_text).is_ok(), "{name}");
            for comment in &preprocessed.comments {
                assert_eq!(
                    &source[comment.span.start..comment.span.end],
                    comment.lexeme,
                    "{name}"
                );
            }
        }
    }

    #[test]
    fn preprocess_source_reports_code_region_precondition_violations() {
        let preprocessed =
            preprocess_source_for_lexing("alpha\r\n\u{03b2}\n::: doc \u{03b2}\nomega");

        assert_eq!(
            preprocessed
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                SourcePreprocessDiagnosticCode::CarriageReturn,
                SourcePreprocessDiagnosticCode::NonAsciiCode,
            ]
        );
        assert_eq!(preprocessed.comments[0].kind, CommentKind::Documentation);
    }

    #[test]
    fn preprocess_source_pins_unsupported_unicode_code_region_diagnostics() {
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
            let preprocessed = preprocess_source_for_lexing(&source);
            let expected_span = SourceSpan {
                start: "alpha".len(),
                end: "alpha".len() + ch.len_utf8(),
            };

            assert_eq!(preprocessed.lexical_text, source, "{name}");
            assert!(preprocessed.comments.is_empty(), "{name}");
            assert_eq!(preprocessed.diagnostics.len(), 1, "{name}");
            assert_eq!(
                preprocessed.diagnostics[0].code,
                SourcePreprocessDiagnosticCode::NonAsciiCode,
                "{name}"
            );
            assert_eq!(preprocessed.diagnostics[0].span, expected_span, "{name}");
            assert!(
                scan_raw(&preprocessed.lexical_text).is_err(),
                "{name} must remain unsupported if it reaches raw scanning"
            );
        }
    }

    #[test]
    fn preprocess_source_reports_unterminated_multiline_comment() {
        let preprocessed = preprocess_source_for_lexing("alpha\n::=\nopen block");

        assert_eq!(
            preprocessed
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment]
        );
        assert_eq!(preprocessed.comments[0].kind, CommentKind::MultiLine);
        assert_eq!(preprocessed.lexical_text, "alpha\n\n");
    }

    #[test]
    fn preprocess_source_keeps_annotations_parser_visible() {
        let preprocessed = preprocess_source_for_lexing("@latex(\"alpha\")\n@[Lemma]\n");

        assert_eq!(preprocessed.lexical_text, "@latex(\"alpha\")\n@[Lemma]\n");
        assert!(preprocessed.comments.is_empty());
        assert!(preprocessed.diagnostics.is_empty());
    }

    #[test]
    fn module_source_name_api_derives_miz_module_and_namespace() {
        let naming = module_source_name_from_path("algebra", "algebra/src/groups/basic.miz")
            .expect(".miz module path should derive names");

        assert_eq!(naming.file_name, "basic.miz");
        assert_eq!(naming.module_name, "basic");
        assert_eq!(
            naming.namespace_components,
            vec![
                "algebra".to_owned(),
                "groups".to_owned(),
                "basic".to_owned()
            ]
        );

        assert!(matches!(
            module_source_name_from_path("algebra", "algebra/src/groups/basic.txt"),
            Err(ModuleNamingError::MissingMizExtension { .. })
        ));
        assert!(matches!(
            module_source_name_from_path("algebra", "algebra/src/bad-name.miz"),
            Err(ModuleNamingError::InvalidNamespaceComponent { .. })
        ));
        assert!(matches!(
            module_source_name_from_path("algebra", "groups/basic.miz"),
            Err(ModuleNamingError::MissingSourceRoot { .. })
        ));
        assert!(matches!(
            module_source_name_from_path("bad-name", "algebra/src/groups/basic.miz"),
            Err(ModuleNamingError::InvalidPackageName { .. })
        ));
        assert!(matches!(
            module_source_name_from_path("algebra", "analysis/src/groups/basic.miz"),
            Err(ModuleNamingError::PackageRootMismatch { .. })
        ));
    }

    #[test]
    fn scans_empty_import_prelude() {
        let raw = scan_raw("definition\nend;").expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        assert!(prelude.imports.is_empty());
        assert_eq!(prelude.end, 0);
        assert!(prelude.diagnostics.is_empty());
    }

    #[test]
    fn scans_imports_aliases_and_relative_paths_from_raw_runs() {
        let raw = scan_raw("import std.algebra.group, ..common as C, .utils;")
            .expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        let paths = prelude
            .imports
            .iter()
            .map(|import| import.path.spelling.as_str())
            .collect::<Vec<_>>();
        assert_eq!(paths, vec!["std.algebra.group", "..common", ".utils"]);
        assert_eq!(prelude.imports[1].alias.as_ref().unwrap().spelling, "C");
        assert_eq!(
            prelude.imports[1].path.relative,
            Some(RawModuleRelativePrefix::Parent)
        );
        assert_eq!(
            prelude.imports[2].path.relative,
            Some(RawModuleRelativePrefix::Current)
        );
        assert_eq!(
            prelude.end,
            "import std.algebra.group, ..common as C, .utils;".len()
        );
        assert!(prelude.diagnostics.is_empty());
    }

    #[test]
    fn scans_contiguous_import_statements() {
        let source = "\
import std.algebra.group;
import std.topology.metric_space as Metric;
import pkg.mathcomp_mizar.algebra.ring;";
        let raw = scan_raw(source).expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        let paths = prelude
            .imports
            .iter()
            .map(|import| import.path.spelling.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            vec![
                "std.algebra.group",
                "std.topology.metric_space",
                "pkg.mathcomp_mizar.algebra.ring"
            ]
        );
        assert_eq!(
            prelude.imports[1].alias.as_ref().unwrap().spelling,
            "Metric"
        );
        assert_eq!(prelude.end, source.len());
        assert!(prelude.diagnostics.is_empty());
    }

    #[test]
    fn scans_branch_import_paths() {
        let source = "import algebra.linear.{eigen_value, jordan};";
        let raw = scan_raw(source).expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        let paths = prelude
            .imports
            .iter()
            .map(|import| import.path.spelling.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            paths,
            vec!["algebra.linear.eigen_value", "algebra.linear.jordan"]
        );
        assert_eq!(
            prelude.imports[1].path.source_segments,
            vec![
                SourceSpan { start: 7, end: 21 },
                SourceSpan { start: 36, end: 42 },
            ]
        );
        assert_eq!(prelude.end, source.len());
        assert!(prelude.diagnostics.is_empty());
    }

    #[test]
    fn stops_at_first_non_import_top_level_text() {
        let raw = scan_raw("import std.core;\ndefinition\nimport dev.late;")
            .expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        assert_eq!(prelude.imports.len(), 1);
        assert_eq!(prelude.imports[0].path.spelling, "std.core");
        assert_eq!(prelude.end, "import std.core;".len());
        assert!(prelude.diagnostics.is_empty());
    }

    #[test]
    fn recovers_malformed_imports_with_diagnostics() {
        let raw = scan_raw("import std., pkg.math as ;").expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        let paths = prelude
            .imports
            .iter()
            .map(|import| import.path.spelling.as_str())
            .collect::<Vec<_>>();
        assert_eq!(paths, vec!["std.", "pkg.math"]);
        assert_eq!(
            prelude
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                ImportPrescanDiagnosticCode::EmptyModulePathComponent,
                ImportPrescanDiagnosticCode::MissingAlias,
            ]
        );
    }

    #[test]
    fn comma_separated_import_stub_spans_cover_each_declaration() {
        let raw = scan_raw("import std.core, pkg.math;").expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        assert_eq!(prelude.imports[0].span, SourceSpan { start: 7, end: 15 });
        assert_eq!(prelude.imports[1].span, SourceSpan { start: 17, end: 25 });
    }

    #[test]
    fn missing_semicolon_does_not_consume_top_level_terminator() {
        let raw = scan_raw("import std.core\ndefinition\nend;").expect("source should raw scan");
        let prelude = scan_import_prelude(&raw);

        assert_eq!(prelude.imports.len(), 1);
        assert_eq!(prelude.end, "import std.core".len());
        assert_eq!(
            prelude.diagnostics[0].code,
            ImportPrescanDiagnosticCode::MissingSemicolon
        );
    }

    #[test]
    fn lexical_environment_always_contains_reserved_tables() {
        let env = build_lexical_environment(&[], &[]).expect("empty imports should build");

        assert_eq!(env.reserved_word("theorem"), Some("theorem"));
        assert_eq!(env.reserved_symbol(":="), Some(":="));
        assert!(env.user_symbol("+").is_none());
    }

    #[test]
    fn lexical_environment_imports_identifier_punctuation_and_dot_symbols() {
        let env = build_lexical_environment(
            &[resolved_import("std.algebra.ops")],
            &[summary(
                "std.algebra.ops",
                11,
                &[
                    exported("succ", "std.algebra.ops#succ", "std.algebra.ops", 0),
                    exported("*+", "std.algebra.ops#star_plus", "std.algebra.ops", 1),
                    exported("|.", "std.algebra.ops#abs_open", "std.algebra.ops", 2),
                    exported("grp.mul", "std.algebra.ops#qualified", "std.algebra.ops", 3),
                ],
            )],
        )
        .expect("environment should build");

        assert_eq!(
            env.user_symbol("succ")
                .expect("identifier-shaped symbol")
                .symbol_id,
            symbol_id("std.algebra.ops#succ")
        );
        assert_eq!(
            env.longest_user_symbol_at("*+x", 0)[0].symbol_id,
            symbol_id("std.algebra.ops#star_plus")
        );
        assert_eq!(
            env.longest_user_symbol_at("|.x.|", 0)[0].symbol_id,
            symbol_id("std.algebra.ops#abs_open")
        );
        assert_eq!(
            env.longest_user_symbol_at("let grp.mul be", 4)[0].symbol_id,
            symbol_id("std.algebra.ops#qualified")
        );
    }

    #[test]
    fn lexical_environment_longest_match_prefers_longest_user_symbol() {
        let env = build_lexical_environment(
            &[resolved_import("std.algebra.ops")],
            &[summary(
                "std.algebra.ops",
                12,
                &[
                    exported("+", "std.algebra.ops#plus", "std.algebra.ops", 0),
                    exported("+*", "std.algebra.ops#plus_star", "std.algebra.ops", 1),
                    exported(
                        "+*+",
                        "std.algebra.ops#plus_star_plus",
                        "std.algebra.ops",
                        2,
                    ),
                ],
            )],
        )
        .expect("environment should build");

        assert_eq!(
            env.longest_user_symbol_at("+*+x", 0),
            vec![UserSymbolCandidate {
                spelling: "+*+".to_owned(),
                symbol_id: symbol_id("std.algebra.ops#plus_star_plus"),
                source_module: module_id("std.algebra.ops"),
                imported_module: module_id("std.algebra.ops"),
                import_ordinal: 0,
                export_rank: ExportRank(2),
            }]
        );
    }

    #[test]
    fn lexical_environment_distinguishes_equal_length_symbols_by_spelling() {
        let env = build_lexical_environment(
            &[resolved_import("std.first"), resolved_import("std.second")],
            &[
                summary(
                    "std.first",
                    13,
                    &[exported("++", "std.first#plusplus", "std.first", 0)],
                ),
                summary(
                    "std.second",
                    14,
                    &[exported("+*", "std.second#plus_star", "std.second", 0)],
                ),
            ],
        )
        .expect("environment should build");

        let candidates = env.longest_user_symbol_at("+*++", 0);
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.symbol_id.clone())
                .collect::<Vec<_>>(),
            vec![symbol_id("std.second#plus_star")]
        );

        let same_start = env.longest_user_symbol_at("++", 0);
        assert_eq!(
            same_start
                .iter()
                .map(|candidate| candidate.symbol_id.clone())
                .collect::<Vec<_>>(),
            vec![symbol_id("std.first#plusplus")]
        );
    }

    #[test]
    fn lexical_environment_returns_empty_lookup_for_invalid_offsets() {
        let env = build_lexical_environment(
            &[resolved_import("std.unicode_fixture")],
            &[summary(
                "std.unicode_fixture",
                15,
                &[exported(
                    "+",
                    "std.unicode_fixture#plus",
                    "std.unicode_fixture",
                    0,
                )],
            )],
        )
        .expect("environment should build");

        assert!(env.longest_user_symbol_at("+", 4).is_empty());
        assert!(env.longest_user_symbol_at("aé+", 2).is_empty());
    }

    #[test]
    fn lexical_environment_rejects_equal_spelling_across_imports() {
        let error = build_lexical_environment(
            &[resolved_import("std.first"), resolved_import("std.second")],
            &[
                summary(
                    "std.first",
                    21,
                    &[exported("+", "std.first#plus", "std.first", 0)],
                ),
                summary(
                    "std.second",
                    22,
                    &[exported("+", "std.second#plus", "std.second", 0)],
                ),
            ],
        )
        .expect_err("equal imported user-symbol spelling should be a conflict");

        assert!(matches!(
            error,
            LexicalEnvironmentError::UserSymbolImportConflict { .. }
        ));
    }

    #[test]
    fn lexical_environment_import_conflict_reports_imported_modules() {
        let error = build_lexical_environment(
            &[resolved_import("facade.a"), resolved_import("facade.b")],
            &[
                summary(
                    "facade.a",
                    24,
                    &[exported("+", "std.origin#plus", "std.origin", 0)],
                ),
                summary(
                    "facade.b",
                    25,
                    &[exported("+", "std.origin#plus", "std.origin", 0)],
                ),
            ],
        )
        .expect_err("conflict diagnostics should mention imported modules");

        assert_eq!(
            error,
            LexicalEnvironmentError::UserSymbolImportConflict {
                spelling: "+".to_owned(),
                earlier_import: module_id("facade.a"),
                later_import: module_id("facade.b"),
            }
        );
    }

    #[test]
    fn lexical_environment_keeps_same_import_candidates_for_same_spelling() {
        let env = build_lexical_environment(
            &[resolved_import("std.overloaded")],
            &[summary(
                "std.overloaded",
                23,
                &[
                    exported("+", "std.overloaded#plus_nat", "std.overloaded", 0),
                    exported("+", "std.overloaded#plus_real", "std.overloaded", 1),
                ],
            )],
        )
        .expect("same imported module may export overloaded notation candidates");

        assert_eq!(
            env.longest_user_symbol_at("+ x", 0)
                .iter()
                .map(|candidate| candidate.symbol_id.clone())
                .collect::<Vec<_>>(),
            vec![
                symbol_id("std.overloaded#plus_nat"),
                symbol_id("std.overloaded#plus_real")
            ]
        );
    }

    #[test]
    fn lexical_environment_rejects_illegal_reserved_collisions() {
        let word_error = build_lexical_environment(
            &[resolved_import("bad.words")],
            &[summary(
                "bad.words",
                31,
                &[exported("theorem", "bad.words#theorem", "bad.words", 0)],
            )],
        )
        .expect_err("reserved word collision should fail");
        assert!(matches!(
            word_error,
            LexicalEnvironmentError::ReservedWordCollision { .. }
        ));

        let symbol_error = build_lexical_environment(
            &[resolved_import("bad.symbols")],
            &[summary(
                "bad.symbols",
                32,
                &[exported(":=", "bad.symbols#assign", "bad.symbols", 0)],
            )],
        )
        .expect_err("reserved symbol collision should fail");
        assert!(matches!(
            symbol_error,
            LexicalEnvironmentError::ReservedSymbolCollision { .. }
        ));
    }

    #[test]
    fn lexical_environment_rejects_invalid_user_symbol_spelling() {
        let error = build_lexical_environment(
            &[resolved_import("bad.annotations")],
            &[summary(
                "bad.annotations",
                34,
                &[exported(
                    "@bad",
                    "bad.annotations#bad",
                    "bad.annotations",
                    0,
                )],
            )],
        )
        .expect_err("annotation marker characters are not valid user symbols");

        assert!(matches!(
            error,
            LexicalEnvironmentError::InvalidUserSymbolSpelling { .. }
        ));
    }

    #[test]
    fn lexical_environment_allows_dot_user_symbol_exception() {
        let env = build_lexical_environment(
            &[resolved_import("std.application")],
            &[summary(
                "std.application",
                33,
                &[exported(".", "std.application#dot", "std.application", 0)],
            )],
        )
        .expect("dot is the reserved-symbol collision exception");

        assert_eq!(
            env.user_symbol(".").expect("dot user symbol").symbol_id,
            symbol_id("std.application#dot")
        );
    }

    #[test]
    fn lexical_environment_fingerprint_is_stable_for_same_ordered_inputs() {
        let imports = vec![resolved_import("std.first"), resolved_import("std.second")];
        let summaries = vec![
            summary(
                "std.second",
                42,
                &[exported("*+", "s#star", "std.second", 0)],
            ),
            summary(
                "std.first",
                41,
                &[exported("succ", "f#succ", "std.first", 0)],
            ),
        ];

        let first = build_lexical_environment(&imports, &summaries)
            .expect("first environment should build");
        let second = build_lexical_environment(&imports, &summaries)
            .expect("second environment should build");
        let reversed_imports = vec![resolved_import("std.second"), resolved_import("std.first")];
        let reversed = build_lexical_environment(&reversed_imports, &summaries)
            .expect("reversed environment should build");

        assert_eq!(first.fingerprint, second.fingerprint);
        assert_ne!(first.fingerprint, reversed.fingerprint);
    }

    #[test]
    fn lexical_environment_reports_missing_and_inconsistent_summaries() {
        let missing = build_lexical_environment(&[resolved_import("missing")], &[])
            .expect_err("missing summary should fail");
        assert!(matches!(
            missing,
            LexicalEnvironmentError::MissingModuleSummary { .. }
        ));

        let inconsistent = build_lexical_environment(
            &[resolved_import("dup")],
            &[
                summary("dup", 1, &[exported("+", "dup#plus", "dup", 0)]),
                summary("dup", 2, &[exported("+", "dup#plus", "dup", 0)]),
            ],
        )
        .expect_err("inconsistent duplicate summary should fail");
        assert!(matches!(
            inconsistent,
            LexicalEnvironmentError::InconsistentDuplicateSummary { .. }
        ));

        let same_fingerprint_different_exports = build_lexical_environment(
            &[resolved_import("same_hash")],
            &[
                summary(
                    "same_hash",
                    5,
                    &[exported("+", "same_hash#plus", "same_hash", 0)],
                ),
                summary(
                    "same_hash",
                    5,
                    &[exported("*", "same_hash#star", "same_hash", 0)],
                ),
            ],
        )
        .expect_err("duplicate summary content must match exactly");
        assert!(matches!(
            same_fingerprint_different_exports,
            LexicalEnvironmentError::InconsistentDuplicateSummary { .. }
        ));
    }

    #[test]
    fn lexical_environment_treats_summary_order_as_canonical_input() {
        let imports = vec![resolved_import("canonical")];
        let canonical = vec![summary(
            "canonical",
            61,
            &[
                exported("+", "canonical#plus", "canonical", 0),
                exported("*", "canonical#star", "canonical", 1),
            ],
        )];
        let reordered = vec![summary(
            "canonical",
            61,
            &[
                exported("*", "canonical#star", "canonical", 1),
                exported("+", "canonical#plus", "canonical", 0),
            ],
        )];

        let canonical_env = build_lexical_environment(&imports, &canonical)
            .expect("canonical summary should build");
        let reordered_env = build_lexical_environment(&imports, &reordered)
            .expect("environment does not recanonicalize summaries");

        assert_ne!(canonical_env.fingerprint, reordered_env.fingerprint);
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
        let namespace_stream =
            disambiguate(&raw, &env, &ParserLexContext::namespace_path(), &skeleton);
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
    fn disambiguator_recognizes_strings_only_when_required() {
        let env = build_lexical_environment(&[], &[]).expect("environment should build");
        let raw = scan_raw("\"abc\"").expect("source should raw scan");
        let skeleton = build_scope_skeleton(&raw);
        let string_stream =
            disambiguate(&raw, &env, &ParserLexContext::string_required(), &skeleton);
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
                LexDiagnosticCode::NoValidTokenCandidate,
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
    }

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

    #[test]
    fn scope_skeleton_handles_empty_stream() {
        let raw = scan_raw("").expect("empty input should raw scan");
        let skeleton = build_scope_skeleton(&raw);

        assert!(skeleton.frames.is_empty());
        assert!(skeleton.diagnostics.is_empty());
    }

    #[test]
    fn scope_skeleton_records_simple_and_comma_separated_let_binders() {
        let source = "let x, y be set;";
        let raw = scan_raw(source).expect("source should raw scan");
        let skeleton = build_scope_skeleton(&raw);

        assert_eq!(skeleton.frames.len(), 1);
        assert_eq!(skeleton.frames[0].range, SourceSpan { start: 0, end: 16 });
        assert_eq!(
            skeleton.frames[0]
                .bindings
                .iter()
                .map(|binding| binding.spelling.as_str())
                .collect::<Vec<_>>(),
            vec!["x", "y"]
        );
        assert!(skeleton.binding_overrides_symbol("x", 6));
        assert!(skeleton.binding_overrides_symbol("y", 9));
        assert!(!skeleton.binding_overrides_symbol("x", 4));
        assert!(!skeleton.binding_overrides_symbol("z", 6));
        assert!(skeleton.diagnostics.is_empty());
    }

    #[test]
    fn scope_skeleton_records_supported_for_reserve_and_given_binders() {
        let source = "reserve A, B for set;\ngiven c being object;";
        let raw = scan_raw(source).expect("source should raw scan");
        let skeleton = build_scope_skeleton(&raw);

        assert_eq!(skeleton.frames.len(), 2);
        assert_eq!(
            skeleton.frames[0]
                .bindings
                .iter()
                .map(|binding| binding.spelling.as_str())
                .collect::<Vec<_>>(),
            vec!["A", "B"]
        );
        assert_eq!(
            skeleton.frames[1]
                .bindings
                .iter()
                .map(|binding| binding.spelling.as_str())
                .collect::<Vec<_>>(),
            vec!["c"]
        );
        assert!(skeleton.binding_overrides_symbol("A", source.len() - 1));
        assert!(skeleton.binding_overrides_symbol("c", source.len() - 1));
        assert!(skeleton.diagnostics.is_empty());
    }

    #[test]
    fn scope_skeleton_limits_for_and_given_binders_to_statement_ranges() {
        let source = "for x holds thesis;\nx;\ngiven y being object;\ny;";
        let raw = scan_raw(source).expect("source should raw scan");
        let skeleton = build_scope_skeleton(&raw);

        assert_eq!(skeleton.frames.len(), 2);
        assert!(skeleton.binding_overrides_symbol("x", 6));
        assert!(!skeleton.binding_overrides_symbol("x", 21));
        assert!(skeleton.binding_overrides_symbol("y", 41));
        assert!(!skeleton.binding_overrides_symbol("y", source.len() - 1));
    }

    #[test]
    fn scope_skeleton_separates_let_reserve_and_statement_lifetimes() {
        let source = "\
reserve R for set;
definition
let x be set;
now
let y be set;
for z holds y = z;
y;
end;
y;
end;
x;
R;";
        let raw = scan_raw(source).expect("source should raw scan");
        let skeleton = build_scope_skeleton(&raw);

        let r_declaration = nth_index(source, "R", 0);
        let r_after_definition = nth_index(source, "R", 1);
        let x_inside_definition = nth_index(source, "x", 0) + 1;
        let x_after_definition = nth_index(source, "x", 1);
        let y_inside_now = nth_index(source, "y", 1);
        let y_after_now = nth_index(source, "y", 3);
        let z_inside_for = nth_index(source, "z", 1);
        let y_before_for = nth_index(source, "y", 0);

        assert!(!skeleton.binding_overrides_symbol("R", r_declaration));
        assert!(skeleton.binding_overrides_symbol("R", r_after_definition));
        assert!(skeleton.binding_overrides_symbol("x", x_inside_definition));
        assert!(!skeleton.binding_overrides_symbol("x", x_after_definition));
        assert!(skeleton.binding_overrides_symbol("y", y_inside_now));
        assert!(!skeleton.binding_overrides_symbol("y", y_after_now));
        assert!(skeleton.binding_overrides_symbol("z", z_inside_for));
        assert!(!skeleton.binding_overrides_symbol("z", y_before_for));
        assert!(skeleton.diagnostics.is_empty());
    }

    #[test]
    fn scope_skeleton_under_approximates_block_local_reserve() {
        let source = "definition\nreserve R for set;\nR;\nend;\nR;";
        let raw = scan_raw(source).expect("source should raw scan");
        let skeleton = build_scope_skeleton(&raw);

        assert!(!skeleton.binding_overrides_symbol("R", nth_index(source, "R", 1)));
        assert!(!skeleton.binding_overrides_symbol("R", nth_index(source, "R", 2)));
        assert_eq!(
            skeleton
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![ScopeSkeletonDiagnosticCode::UnsupportedBinderShape]
        );
    }

    #[test]
    fn scope_skeleton_pairs_nested_block_ranges() {
        let source = "definition\nlet x be set;\nproof\nnow\nlet y be set;\nend;\nend;\nend;";
        let raw = scan_raw(source).expect("source should raw scan");
        let skeleton = build_scope_skeleton(&raw);

        assert_eq!(skeleton.frames.len(), 3);
        assert_eq!(
            skeleton
                .frames
                .iter()
                .map(|frame| frame.range)
                .collect::<Vec<_>>(),
            vec![
                SourceSpan { start: 0, end: 62 },
                SourceSpan { start: 25, end: 57 },
                SourceSpan { start: 31, end: 52 },
            ]
        );
        assert!(skeleton.binding_overrides_symbol("x", 25));
        assert!(skeleton.binding_overrides_symbol("x", 61));
        assert!(skeleton.binding_overrides_symbol("y", 51));
        assert!(!skeleton.binding_overrides_symbol("y", 52));
        assert!(skeleton.diagnostics.is_empty());
    }

    #[test]
    fn scope_skeleton_records_proof_case_suppose_and_algorithm_shapes() {
        let source = "\
definition
proof
given g being object;
consider c being object such that c = c;
set s = c;
reconsider rc = c as object;
take tk = c;
deffunc F(object) = c;
defpred P[object] means c = c;
case
let k be set;
end;
suppose c = c;
let sp be set;
end;
end;
end;
algorithm
do
var a, b = (c, d);
const n = 1;
ghost var gv;
ghost const gc = 2;
for i = 0 to 2 do
var inner;
end;
for item in Items processed Seen do
var consumed;
end;
end;
end;";
        let raw = scan_raw(source).expect("source should raw scan");
        let skeleton = build_scope_skeleton(&raw);

        assert_eq!(
            skeleton
                .blocks
                .iter()
                .map(|block| block.kind)
                .collect::<Vec<_>>(),
            vec![
                LexicalBlockKind::Definition,
                LexicalBlockKind::Proof,
                LexicalBlockKind::Case,
                LexicalBlockKind::Suppose,
                LexicalBlockKind::Algorithm,
                LexicalBlockKind::Do,
                LexicalBlockKind::Do,
                LexicalBlockKind::Do,
            ]
        );
        assert!(skeleton.binding_overrides_symbol("g", nth_index(source, "object", 0)));
        assert!(!skeleton.binding_overrides_symbol("g", nth_index(source, "consider", 0)));
        assert!(skeleton.binding_overrides_symbol("c", nth_index(source, "deffunc", 0)));
        assert!(skeleton.binding_overrides_symbol("F", nth_index(source, "defpred", 0)));
        assert!(skeleton.binding_overrides_symbol("a", nth_index(source, "const", 0)));
        assert!(skeleton.binding_overrides_symbol("gv", nth_index(source, "for i", 0)));
        assert!(skeleton.binding_overrides_symbol("i", nth_index(source, "inner", 0)));
        assert!(!skeleton.binding_overrides_symbol("i", nth_index(source, "for item", 0)));
        assert!(skeleton.binding_overrides_symbol("Seen", nth_index(source, "consumed", 0)));
        assert!(!skeleton.binding_overrides_symbol("Seen", source.len()));
        assert!(skeleton.diagnostics.is_empty());
    }

    #[test]
    fn scope_skeleton_under_approximates_malformed_binders() {
        let raw = scan_raw("let , x be set;\nfor + y holds thesis;").expect("source should scan");
        let skeleton = build_scope_skeleton(&raw);

        assert!(skeleton.frames.is_empty());
        assert_eq!(
            skeleton
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                ScopeSkeletonDiagnosticCode::MalformedBinderList,
                ScopeSkeletonDiagnosticCode::UnsupportedBinderShape,
            ]
        );
    }

    #[test]
    fn scope_skeleton_reports_recoverable_block_diagnostics_deterministically() {
        let raw = scan_raw("end;\ndefinition\nlet x be set;").expect("source should scan");
        let first = build_scope_skeleton(&raw);
        let second = build_scope_skeleton(&raw);

        assert_eq!(first, second);
        assert_eq!(
            first
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                ScopeSkeletonDiagnosticCode::UnmatchedEnd,
                ScopeSkeletonDiagnosticCode::MissingEnd,
            ]
        );
        assert!(first.binding_overrides_symbol("x", 27));
    }

    fn resolved_import(module: &str) -> ResolvedImport {
        ResolvedImport {
            module_id: module_id(module),
        }
    }

    fn summary(
        module: &str,
        fingerprint: u64,
        exported_symbols: &[ExportedSymbolShape],
    ) -> ModuleLexicalSummary {
        ModuleLexicalSummary {
            module_id: module_id(module),
            exported_symbols: exported_symbols.to_vec(),
            fingerprint: LexicalSummaryFingerprint(fingerprint),
        }
    }

    fn exported(
        spelling: &str,
        symbol: &str,
        source_module: &str,
        rank: u32,
    ) -> ExportedSymbolShape {
        ExportedSymbolShape {
            spelling: spelling.to_owned(),
            symbol_id: symbol_id(symbol),
            source_module: module_id(source_module),
            export_rank: ExportRank(rank),
        }
    }

    fn module_id(value: &str) -> ModuleId {
        ModuleId(value.to_owned())
    }

    fn symbol_id(value: &str) -> SymbolId {
        SymbolId(value.to_owned())
    }

    fn nth_index(haystack: &str, needle: &str, ordinal: usize) -> usize {
        haystack
            .match_indices(needle)
            .nth(ordinal)
            .map(|(index, _)| index)
            .expect("test source should contain requested occurrence")
    }
}
