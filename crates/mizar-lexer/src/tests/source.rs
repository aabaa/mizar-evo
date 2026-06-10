use super::common::*;

#[test]
fn source_span_exposes_stable_boundary_helpers() {
    let span = SourceSpan::new(3, 8);

    assert_eq!(span.start, 3);
    assert_eq!(span.end, 8);
    assert_eq!(SourceSpan::try_new(3, 8), Some(span));
    assert_eq!(span.len(), 5);
    assert!(!span.is_empty());
    assert!(span.is_valid());
    assert!(!span.contains(2));
    assert!(span.contains(3));
    assert!(span.contains(7));
    assert!(!span.contains(8));

    let empty = SourceSpan::new(4, 4);
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert!(empty.is_valid());

    assert_eq!(SourceSpan::try_new(5, 4), None);
    let reversed = SourceSpan { start: 5, end: 4 };
    assert!(!reversed.is_valid());
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
fn source_loading_rejects_invalid_utf8_without_lossy_replacement() {
    let error = load_source_text_from_bytes(b"alpha \xff beta")
        .expect_err("invalid UTF-8 must fail before lexer entry");

    assert_eq!(
        error,
        SourceLoadError::InvalidUtf8 {
            valid_up_to: 6,
            error_len: Some(1),
        }
    );
    let lossy = String::from_utf8_lossy(b"alpha \xff beta");
    assert!(lossy.contains('\u{fffd}'));
}

#[test]
fn source_loading_strips_leading_utf8_bom_and_maps_original_offsets() {
    let loaded = load_source_text_from_bytes(b"\xef\xbb\xbfalpha\nbeta")
        .expect("leading UTF-8 BOM should be accepted");

    assert_eq!(loaded.text, "alpha\nbeta");
    assert_eq!(
        loaded.loading_map.as_ref().map(|map| &map.segments),
        Some(&vec![
            SourceLoadingMapSegment::RemovedLeadingBom {
                original: SourceSpan { start: 0, end: 3 },
            },
            SourceLoadingMapSegment::Original {
                loaded: SourceSpan { start: 0, end: 10 },
                original: SourceSpan { start: 3, end: 13 },
            },
        ])
    );

    let map = loaded
        .loading_map
        .as_ref()
        .expect("BOM stripping should record a loading map");
    assert_eq!(map.original_offset_for_loaded(0), Some(3));
    assert_eq!(map.original_offset_for_loaded(5), Some(8));
    assert_eq!(map.original_offset_for_loaded(loaded.text.len()), Some(13));
}

#[test]
fn source_loading_maps_empty_text_after_bom_stripping() {
    let loaded =
        load_source_text_from_bytes(b"\xef\xbb\xbf").expect("BOM-only input is valid UTF-8");

    assert_eq!(loaded.text, "");
    assert_eq!(
        loaded.loading_map.as_ref().map(|map| &map.segments),
        Some(&vec![
            SourceLoadingMapSegment::RemovedLeadingBom {
                original: SourceSpan { start: 0, end: 3 },
            },
            SourceLoadingMapSegment::Original {
                loaded: SourceSpan { start: 0, end: 0 },
                original: SourceSpan { start: 3, end: 3 },
            },
        ])
    );

    let map = loaded
        .loading_map
        .as_ref()
        .expect("BOM-only stripping should still record an insertion-point map");
    assert_eq!(map.original_offset_for_loaded(0), Some(3));
}

#[test]
fn source_loading_preserves_non_leading_bom_for_lexer_boundary() {
    let loaded = load_source_text_from_bytes("alpha\u{feff}beta".as_bytes())
        .expect("non-leading U+FEFF is valid UTF-8 source text");

    assert_eq!(loaded.text, "alpha\u{feff}beta");
    assert_eq!(loaded.loading_map, None);
    assert_eq!(
        preprocess_source_for_lexing(&loaded.text).diagnostics[0].code,
        SourcePreprocessDiagnosticCode::NonAsciiCode
    );
    assert!(scan_raw(&loaded.text).is_err());
}

#[test]
fn source_loading_normalizes_crlf_to_lf_and_maps_original_offsets() {
    let loaded = load_source_text_from_bytes(b"alpha\r\nbeta\r\ngamma")
        .expect("CRLF input should normalize before lexer entry");

    assert_eq!(loaded.text, "alpha\nbeta\ngamma");
    assert_eq!(
        loaded.loading_map.as_ref().map(|map| &map.segments),
        Some(&vec![
            SourceLoadingMapSegment::Original {
                loaded: SourceSpan { start: 0, end: 5 },
                original: SourceSpan { start: 0, end: 5 },
            },
            SourceLoadingMapSegment::NormalizedNewline {
                loaded: SourceSpan { start: 5, end: 6 },
                original: SourceSpan { start: 5, end: 7 },
            },
            SourceLoadingMapSegment::Original {
                loaded: SourceSpan { start: 6, end: 10 },
                original: SourceSpan { start: 7, end: 11 },
            },
            SourceLoadingMapSegment::NormalizedNewline {
                loaded: SourceSpan { start: 10, end: 11 },
                original: SourceSpan { start: 11, end: 13 },
            },
            SourceLoadingMapSegment::Original {
                loaded: SourceSpan { start: 11, end: 16 },
                original: SourceSpan { start: 13, end: 18 },
            },
        ])
    );

    let map = loaded
        .loading_map
        .as_ref()
        .expect("CRLF normalization should record a loading map");
    assert_eq!(map.original_offset_for_loaded(0), Some(0));
    assert_eq!(map.original_offset_for_loaded(5), Some(5));
    assert_eq!(map.original_offset_for_loaded(6), Some(7));
    assert_eq!(map.original_offset_for_loaded(10), Some(11));
    assert_eq!(map.original_offset_for_loaded(11), Some(13));
    assert_eq!(map.original_offset_for_loaded(loaded.text.len()), Some(18));
    scan_raw(&loaded.text).expect("normalized LF-only text should scan");
}

#[test]
fn source_loading_combines_bom_stripping_with_crlf_mapping() {
    let loaded = load_source_text_from_bytes(b"\xef\xbb\xbfalpha\r\nbeta")
        .expect("BOM and CRLF should normalize before lexer entry");

    assert_eq!(loaded.text, "alpha\nbeta");
    assert_eq!(
        loaded.loading_map.as_ref().map(|map| &map.segments),
        Some(&vec![
            SourceLoadingMapSegment::RemovedLeadingBom {
                original: SourceSpan { start: 0, end: 3 },
            },
            SourceLoadingMapSegment::Original {
                loaded: SourceSpan { start: 0, end: 5 },
                original: SourceSpan { start: 3, end: 8 },
            },
            SourceLoadingMapSegment::NormalizedNewline {
                loaded: SourceSpan { start: 5, end: 6 },
                original: SourceSpan { start: 8, end: 10 },
            },
            SourceLoadingMapSegment::Original {
                loaded: SourceSpan { start: 6, end: 10 },
                original: SourceSpan { start: 10, end: 14 },
            },
        ])
    );

    let map = loaded
        .loading_map
        .as_ref()
        .expect("BOM stripping plus CRLF normalization should record a loading map");
    assert_eq!(map.original_offset_for_loaded(0), Some(3));
    assert_eq!(map.original_offset_for_loaded(5), Some(8));
    assert_eq!(map.original_offset_for_loaded(6), Some(10));
    assert_eq!(map.original_offset_for_loaded(loaded.text.len()), Some(14));
}

#[test]
fn source_loading_preserves_lone_cr_for_lexer_boundary_diagnostics() {
    let loaded = load_source_text_from_bytes(b"alpha\rbeta")
        .expect("lone CR is valid UTF-8 but not a platform newline pair");

    assert_eq!(loaded.text, "alpha\rbeta");
    assert_eq!(loaded.loading_map, None);
    assert_eq!(
        preprocess_source_for_lexing(&loaded.text).diagnostics[0].code,
        SourcePreprocessDiagnosticCode::CarriageReturn
    );
    assert!(scan_raw(&loaded.text).is_err());
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
fn preprocess_map_records_removed_comments_and_synthetic_newlines() {
    let source = "alpha:: ordinary\n::: doc\nbeta";
    let preprocessed = preprocess_source_for_lexing(source);

    assert_eq!(preprocessed.lexical_text, "alpha\n\nbeta");
    assert_eq!(
        preprocessed
            .comments
            .iter()
            .map(|comment| comment.kind)
            .collect::<Vec<_>>(),
        vec![CommentKind::SingleLine, CommentKind::Documentation]
    );
    assert_eq!(
        preprocessed.preprocess_map.segments,
        vec![
            SourcePreprocessMapSegment::Original {
                lexical: SourceSpan { start: 0, end: 5 },
                source: SourceSpan { start: 0, end: 5 },
            },
            SourcePreprocessMapSegment::RemovedComment {
                source: SourceSpan { start: 5, end: 17 },
                kind: CommentKind::SingleLine,
            },
            SourcePreprocessMapSegment::SyntheticWhitespace {
                lexical: SourceSpan { start: 5, end: 6 },
                anchor: SourceSpan { start: 16, end: 17 },
            },
            SourcePreprocessMapSegment::RemovedComment {
                source: SourceSpan { start: 17, end: 25 },
                kind: CommentKind::Documentation,
            },
            SourcePreprocessMapSegment::SyntheticWhitespace {
                lexical: SourceSpan { start: 6, end: 7 },
                anchor: SourceSpan { start: 24, end: 25 },
            },
            SourcePreprocessMapSegment::Original {
                lexical: SourceSpan { start: 7, end: 11 },
                source: SourceSpan { start: 25, end: 29 },
            },
        ]
    );
}

#[test]
fn preprocess_map_maps_ranges_spanning_removed_comments() {
    let source = "alpha::= hidden =::beta";
    let preprocessed = preprocess_source_for_lexing(source);

    assert_eq!(preprocessed.lexical_text, "alpha beta");
    assert_eq!(
        preprocessed.preprocess_map.segments,
        vec![
            SourcePreprocessMapSegment::Original {
                lexical: SourceSpan { start: 0, end: 5 },
                source: SourceSpan { start: 0, end: 5 },
            },
            SourcePreprocessMapSegment::RemovedComment {
                source: SourceSpan { start: 5, end: 19 },
                kind: CommentKind::MultiLine,
            },
            SourcePreprocessMapSegment::SyntheticWhitespace {
                lexical: SourceSpan { start: 5, end: 6 },
                anchor: SourceSpan { start: 5, end: 19 },
            },
            SourcePreprocessMapSegment::Original {
                lexical: SourceSpan { start: 6, end: 10 },
                source: SourceSpan { start: 19, end: 23 },
            },
        ]
    );
    assert_eq!(
        preprocessed
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 0, end: 10 }),
        Some(vec![
            SourceSpan { start: 0, end: 5 },
            SourceSpan { start: 5, end: 19 },
            SourceSpan { start: 19, end: 23 },
        ])
    );
    assert_eq!(
        preprocessed
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 5, end: 6 }),
        Some(vec![SourceSpan { start: 5, end: 19 }])
    );
    assert_eq!(
        preprocessed
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 6, end: 10 }),
        Some(vec![SourceSpan { start: 19, end: 23 }])
    );

    let whitespace_separated = preprocess_source_for_lexing("alpha ::= hidden =:: beta");
    assert_eq!(whitespace_separated.lexical_text, "alpha  beta");
    assert_eq!(
        whitespace_separated
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 0, end: 11 }),
        Some(vec![
            SourceSpan { start: 0, end: 6 },
            SourceSpan { start: 6, end: 20 },
            SourceSpan { start: 20, end: 25 },
        ])
    );
}

#[test]
fn preprocess_map_rejects_out_of_bounds_and_reversed_lexical_ranges() {
    let preprocessed = preprocess_source_for_lexing("alpha:: hidden\nbeta");

    assert_eq!(preprocessed.lexical_text, "alpha\nbeta");
    assert_eq!(
        preprocessed
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 0, end: 10 }),
        Some(vec![
            SourceSpan { start: 0, end: 5 },
            SourceSpan { start: 5, end: 15 },
            SourceSpan { start: 15, end: 19 },
        ])
    );
    assert_eq!(
        preprocessed
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 0, end: 11 }),
        None
    );
    assert_eq!(
        preprocessed
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 11, end: 11 }),
        None
    );
    assert_eq!(
        preprocessed
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 4, end: 3 }),
        None
    );
}

#[test]
fn preprocess_map_maps_zero_length_lexical_insertion_points() {
    let plain = preprocess_source_for_lexing("alpha\nbeta");

    assert_eq!(
        plain
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 0, end: 0 }),
        Some(vec![SourceSpan { start: 0, end: 0 }])
    );
    assert_eq!(
        plain
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 5, end: 5 }),
        Some(vec![SourceSpan { start: 5, end: 5 }])
    );
    assert_eq!(
        plain
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 10, end: 10 }),
        Some(vec![SourceSpan { start: 10, end: 10 }])
    );
    assert_eq!(
        plain
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 11, end: 11 }),
        None
    );

    let empty = preprocess_source_for_lexing("");
    assert_eq!(
        empty
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 0, end: 0 }),
        Some(vec![SourceSpan { start: 0, end: 0 }])
    );
    assert_eq!(
        empty
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 1, end: 1 }),
        None
    );

    let with_synthetic_space = preprocess_source_for_lexing("alpha::= hidden =::beta");
    assert_eq!(
        with_synthetic_space
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 5, end: 5 }),
        Some(vec![
            SourceSpan { start: 5, end: 5 },
            SourceSpan { start: 5, end: 19 },
        ])
    );
    assert_eq!(
        with_synthetic_space
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 6, end: 6 }),
        Some(vec![
            SourceSpan { start: 5, end: 19 },
            SourceSpan { start: 19, end: 19 },
        ])
    );
    assert_eq!(
        with_synthetic_space
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 10, end: 10 }),
        Some(vec![SourceSpan { start: 23, end: 23 }])
    );

    let without_synthetic_space = preprocess_source_for_lexing("alpha ::= hidden =:: beta");
    assert_eq!(without_synthetic_space.lexical_text, "alpha  beta");
    assert_eq!(
        without_synthetic_space
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 6, end: 6 }),
        Some(vec![
            SourceSpan { start: 6, end: 6 },
            SourceSpan { start: 6, end: 20 },
            SourceSpan { start: 20, end: 20 },
        ])
    );
    assert_eq!(
        without_synthetic_space
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 7, end: 7 }),
        Some(vec![SourceSpan { start: 21, end: 21 }])
    );

    let with_synthetic_newline = preprocess_source_for_lexing("alpha:: hidden\nbeta");
    assert_eq!(with_synthetic_newline.lexical_text, "alpha\nbeta");
    assert_eq!(
        with_synthetic_newline
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 5, end: 5 }),
        Some(vec![
            SourceSpan { start: 5, end: 5 },
            SourceSpan { start: 5, end: 15 },
            SourceSpan { start: 14, end: 15 },
        ])
    );
    assert_eq!(
        with_synthetic_newline
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 6, end: 6 }),
        Some(vec![
            SourceSpan { start: 14, end: 15 },
            SourceSpan { start: 15, end: 15 },
        ])
    );

    let adjacent_comments = preprocess_source_for_lexing("alpha:: first\n:: second\nomega");
    assert_eq!(adjacent_comments.lexical_text, "alpha\n\nomega");
    assert_eq!(
        adjacent_comments
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 6, end: 6 }),
        Some(vec![
            SourceSpan { start: 13, end: 14 },
            SourceSpan { start: 14, end: 24 },
            SourceSpan { start: 23, end: 24 },
        ])
    );

    let eof_comment = preprocess_source_for_lexing("alpha\n:: eof");
    assert_eq!(eof_comment.lexical_text, "alpha\n");
    assert_eq!(
        eof_comment
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 6, end: 6 }),
        Some(vec![
            SourceSpan { start: 6, end: 6 },
            SourceSpan { start: 6, end: 12 },
        ])
    );
}

#[test]
fn preprocess_map_maps_lexer_and_preprocessor_diagnostic_ranges() {
    let source = "alpha::= hidden =::\u{00a0}";
    let preprocessed = preprocess_source_for_lexing(source);

    assert_eq!(preprocessed.lexical_text, "alpha \u{00a0}");
    assert_eq!(preprocessed.diagnostics.len(), 1);
    assert_eq!(
        preprocessed.diagnostics[0].code,
        SourcePreprocessDiagnosticCode::NonAsciiCode
    );
    assert_eq!(
        preprocessed.diagnostics[0].span,
        SourceSpan { start: 19, end: 21 }
    );
    assert_eq!(
        preprocessed.diagnostics[0].payload,
        SourcePreprocessDiagnosticPayload::NonAsciiCode {
            character: '\u{00a0}',
            utf8_len: 2,
        }
    );
    assert_eq!(
        preprocessed
            .preprocess_map
            .source_ranges_for_lexical(SourceSpan { start: 6, end: 8 }),
        Some(vec![SourceSpan { start: 19, end: 21 }])
    );
    assert!(scan_raw(&preprocessed.lexical_text).is_err());
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
fn preprocess_source_treats_nested_multiline_comment_markers_as_text() {
    for (name, source, expected_lexical_text, expected_comment) in [
        (
            "inline nested marker",
            "alpha::= outer ::= inner =::omega",
            "alpha omega",
            "::= outer ::= inner =::",
        ),
        (
            "nested marker with preserved newlines",
            "alpha::=\nouter ::= inner\n=::omega",
            "alpha\n\nomega",
            "::=\nouter ::= inner\n=::",
        ),
    ] {
        let preprocessed = preprocess_source_for_lexing(source);

        assert_eq!(preprocessed.lexical_text, expected_lexical_text, "{name}");
        assert_eq!(preprocessed.comments.len(), 1, "{name}");
        assert_eq!(
            preprocessed.comments[0].kind,
            CommentKind::MultiLine,
            "{name}"
        );
        assert!(
            preprocessed.comments[0].lexeme.contains("::= inner"),
            "{name}"
        );
        assert_eq!(preprocessed.comments[0].lexeme, expected_comment, "{name}");
        assert_eq!(
            &source[preprocessed.comments[0].span.start..preprocessed.comments[0].span.end],
            preprocessed.comments[0].lexeme,
            "{name}"
        );
        assert!(preprocessed.diagnostics.is_empty(), "{name}");
        assert!(scan_raw(&preprocessed.lexical_text).is_ok(), "{name}");
    }
}

#[test]
fn preprocess_source_reports_code_region_precondition_violations() {
    let preprocessed = preprocess_source_for_lexing("alpha\r\n\u{03b2}\n::: doc \u{03b2}\nomega");

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
    assert_eq!(
        preprocessed.diagnostics[0].payload,
        SourcePreprocessDiagnosticPayload::CarriageReturn {
            recovery: SourcePreprocessRecoveryHint::NormalizeCrLfBeforeLexerEntry,
        }
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
        assert_eq!(
            preprocessed.diagnostics[0].payload,
            SourcePreprocessDiagnosticPayload::NonAsciiCode {
                character: ch,
                utf8_len: ch.len_utf8(),
            },
            "{name}"
        );
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
    assert_eq!(
        preprocessed.diagnostics[0].payload,
        SourcePreprocessDiagnosticPayload::UnterminatedMultiLineComment {
            opener: SourceSpan { start: 6, end: 9 },
            recovery: SourcePreprocessRecoveryHint::PreserveNewlinesAndDropCommentText,
        }
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
fn preprocess_source_preserves_annotation_string_argument_contents() {
    let source = "@latex(\"α::β\")\nalphaβ";
    let preprocessed = preprocess_source_for_lexing(source);

    assert_eq!(preprocessed.lexical_text, source);
    assert!(
        preprocessed.comments.is_empty(),
        "comment markers inside string arguments must remain source text"
    );
    assert_eq!(preprocessed.diagnostics.len(), 1);
    assert_eq!(
        preprocessed.diagnostics[0].code,
        SourcePreprocessDiagnosticCode::NonAsciiCode,
        "non-ASCII outside string arguments remains a code-region precondition"
    );
    assert_eq!(
        preprocessed.diagnostics[0].span,
        SourceSpan {
            start: "@latex(\"α::β\")\nalpha".len(),
            end: source.len(),
        }
    );
}

#[test]
fn preprocess_source_does_not_preserve_multiline_string_argument_contents() {
    let source = "@latex(\"α\rβ\")";
    let preprocessed = preprocess_source_for_lexing(source);

    assert_eq!(preprocessed.lexical_text, source);
    assert!(preprocessed.comments.is_empty());
    assert_eq!(
        preprocessed
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            SourcePreprocessDiagnosticCode::NonAsciiCode,
            SourcePreprocessDiagnosticCode::CarriageReturn,
            SourcePreprocessDiagnosticCode::NonAsciiCode,
        ],
        "line-boundary-crossing quoted text must not bypass code-region preconditions"
    );
}

#[test]
fn preprocess_source_ignores_comment_text_when_detecting_string_arguments() {
    let source = "alpha ::= (, =:: \"β\"";
    let preprocessed = preprocess_source_for_lexing(source);
    let beta_start = source.find('β').expect("fixture contains beta");

    assert_eq!(preprocessed.comments.len(), 1);
    assert_eq!(preprocessed.lexical_text, "alpha  \"β\"");
    assert_eq!(preprocessed.diagnostics.len(), 1);
    assert_eq!(
        preprocessed.diagnostics[0].code,
        SourcePreprocessDiagnosticCode::NonAsciiCode,
        "removed comment contents must not make the following quote a string argument"
    );
    assert_eq!(
        preprocessed.diagnostics[0].span,
        SourceSpan {
            start: beta_start,
            end: beta_start + 'β'.len_utf8(),
        }
    );
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
