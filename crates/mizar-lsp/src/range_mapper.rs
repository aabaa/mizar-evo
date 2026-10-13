use mizar_lexer::SourceSpan as LexerSourceSpan;
use mizar_session::{
    LineMap, LoadingMap, LoadingOrigin, SourceId, SourceMapError, SourceOriginInput, SourceRange,
    TextRange, normalize_source_text,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspRange {
    pub start: LspPosition,
    pub end: LspPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RangeMapError {
    SourceMap(SourceMapError),
    PositionOverflow,
    InvalidOpenBufferContext,
}

const MAX_LSP_POSITION: usize = i32::MAX as usize;

impl From<SourceMapError> for RangeMapError {
    fn from(error: SourceMapError) -> Self {
        Self::SourceMap(error)
    }
}

pub fn lsp_range_from_source_range(
    line_map: &LineMap,
    range: SourceRange,
) -> Result<LspRange, RangeMapError> {
    lsp_range_from_source_range_with_max_utf16_column(line_map, range, MAX_LSP_POSITION)
}

/// Converts loaded bytes to exact raw OpenBuffer UTF-16 coordinates.
/// The caller owns snapshot, document identity and freshness validation.
pub fn lsp_range_from_open_buffer(
    line_map: &LineMap,
    loading_map: Option<&LoadingMap>,
    origin: &SourceOriginInput,
    range: SourceRange,
) -> Result<LspRange, RangeMapError> {
    line_map.validate_range(range)?;
    let SourceOriginInput::OpenBuffer {
        uri,
        expected_version,
        actual_version,
        text,
    } = origin
    else {
        return Err(RangeMapError::InvalidOpenBufferContext);
    };
    if *expected_version < 0 || expected_version != actual_version {
        return Err(RangeMapError::InvalidOpenBufferContext);
    }
    let (normalized, segments) = normalize_source_text(text);
    let expected_map = segments.map(|segments| {
        LoadingMap::new(
            line_map.source_id(),
            &normalized,
            LoadingOrigin::OpenBufferText {
                uri: uri.clone(),
                version: *actual_version,
            },
            segments,
        )
    });
    if line_map.source() != normalized || loading_map != expected_map.as_ref() {
        return Err(RangeMapError::InvalidOpenBufferContext);
    }
    let loaded = TextRange {
        start: range.start,
        end: range.end,
    };
    let original = match loading_map {
        Some(map) => {
            map.original_range_for_loaded(range.source_id, loaded)?
                .original
        }
        None => loaded,
    };
    let [start, end] = [original.start, original.end].map(|offset| {
        let (mut line, mut line_start, mut content_end) = (0, 0, text.len());
        let mut scalars = text.char_indices().peekable();
        while let Some((start, scalar)) = scalars.next() {
            if !matches!(scalar, '\r' | '\n') {
                continue;
            }
            let mut end = start + 1;
            if scalar == '\r' && scalars.peek().is_some_and(|(_, next)| *next == '\n') {
                scalars.next();
                end += 1;
            }
            if offset < end {
                content_end = start;
                break;
            }
            line += 1;
            line_start = end;
        }
        Ok::<_, RangeMapError>(LspPosition {
            line: u32_lsp_coordinate(line, MAX_LSP_POSITION)?,
            character: u32_lsp_coordinate(
                text[line_start..offset.min(content_end)]
                    .encode_utf16()
                    .count(),
                MAX_LSP_POSITION,
            )?,
        })
    });
    Ok(LspRange {
        start: start?,
        end: end?,
    })
}

fn lsp_range_from_source_range_with_max_utf16_column(
    line_map: &LineMap,
    range: SourceRange,
    max_utf16_column: usize,
) -> Result<LspRange, RangeMapError> {
    line_map.line_column_range(range)?;
    Ok(LspRange {
        start: lsp_position(line_map, range.start, max_utf16_column)?,
        end: lsp_position(line_map, range.end, max_utf16_column)?,
    })
}

pub fn lsp_range_from_lexer_span(
    line_map: &LineMap,
    source_id: SourceId,
    span: LexerSourceSpan,
) -> Result<LspRange, RangeMapError> {
    // Keep this bridge explicit while no frontend/diagnostic adapter crate owns
    // lexer-to-session coordinate conversion.
    lsp_range_from_source_range(line_map, source_range_from_lexer_span(source_id, span))
}

pub fn source_range_from_lexer_span(source_id: SourceId, span: LexerSourceSpan) -> SourceRange {
    // This is intentionally a field copy: lexer and session source ranges both
    // use byte offsets into the loaded text, but remain owned by their crates.
    SourceRange {
        source_id,
        start: span.start,
        end: span.end,
    }
}

fn lsp_position(
    line_map: &LineMap,
    offset: usize,
    max_utf16_column: usize,
) -> Result<LspPosition, RangeMapError> {
    let line_column = line_map.line_column_for_source(line_map.source_id(), offset)?;
    let line_start = line_start_offset(line_map.source(), offset);
    let utf16_column = u32_lsp_coordinate(
        line_map.source()[line_start..offset].encode_utf16().count(),
        max_utf16_column,
    )?;
    Ok(LspPosition {
        line: u32_lsp_coordinate((line_column.line - 1) as usize, MAX_LSP_POSITION)?,
        character: utf16_column,
    })
}

fn line_start_offset(source: &str, offset: usize) -> usize {
    source[..offset]
        .rfind('\n')
        .map_or(0, |newline| newline + '\n'.len_utf8())
}

fn u32_lsp_coordinate(value: usize, max_coordinate: usize) -> Result<u32, RangeMapError> {
    if value > max_coordinate {
        return Err(RangeMapError::PositionOverflow);
    }
    u32::try_from(value).map_err(|_| RangeMapError::PositionOverflow)
}

#[cfg(test)]
mod tests {
    use mizar_lexer::{SourceSpan as LexerSourceSpan, lex};
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, LineColumn, LineMap, SessionIdAllocator,
        SourceId, SourceMapError, SourceRange,
    };

    use super::{
        LspPosition, LspRange, RangeMapError, lsp_range_from_lexer_span,
        lsp_range_from_source_range, source_range_from_lexer_span,
    };

    #[test]
    fn lsp_range_uses_zero_based_utf16_columns_at_lsp_boundary() {
        let source = "a😀β\n漢z";
        let line_map = line_map(source);
        let start = "a".len();
        let end = "a😀β".len();

        assert_eq!(
            line_map.line_column_for_source(line_map.source_id(), end),
            Ok(LineColumn { line: 1, column: 4 })
        );
        assert_eq!(
            lsp_range_from_source_range(&line_map, source_range(&line_map, start, end)),
            Ok(LspRange {
                start: LspPosition {
                    line: 0,
                    character: 1,
                },
                end: LspPosition {
                    line: 0,
                    character: 4,
                },
            })
        );
    }

    #[test]
    fn lsp_range_counts_combining_marks_as_utf16_code_units() {
        let source = "e\u{301}x";
        let line_map = line_map(source);
        let start = "e".len();
        let end = "e\u{301}".len();

        assert_eq!(
            lsp_range_from_source_range(&line_map, source_range(&line_map, start, end)),
            Ok(LspRange {
                start: LspPosition {
                    line: 0,
                    character: 1,
                },
                end: LspPosition {
                    line: 0,
                    character: 2,
                },
            })
        );
    }

    #[test]
    fn lsp_range_restarts_utf16_columns_after_newline() {
        let source = "a😀β\n漢z";
        let line_map = line_map(source);
        let start = "a😀β\n".len();
        let end = "a😀β\n漢z".len();

        assert_eq!(
            lsp_range_from_source_range(&line_map, source_range(&line_map, start, end)),
            Ok(LspRange {
                start: LspPosition {
                    line: 1,
                    character: 0,
                },
                end: LspPosition {
                    line: 1,
                    character: 2,
                },
            })
        );
    }

    #[test]
    fn lsp_range_converts_ranges_across_lines() {
        let source = "ab😀\nβc";
        let line_map = line_map(source);
        let start = "ab".len();
        let end = "ab😀\nβ".len();

        assert_eq!(
            lsp_range_from_source_range(&line_map, source_range(&line_map, start, end)),
            Ok(LspRange {
                start: LspPosition {
                    line: 0,
                    character: 2,
                },
                end: LspPosition {
                    line: 1,
                    character: 1,
                },
            })
        );
    }

    #[test]
    fn lsp_range_reports_zero_character_on_line_after_trailing_newline() {
        let source = "alpha\n";
        let line_map = line_map(source);

        assert_eq!(
            lsp_range_from_source_range(
                &line_map,
                source_range(&line_map, source.len(), source.len()),
            ),
            Ok(LspRange {
                start: LspPosition {
                    line: 1,
                    character: 0,
                },
                end: LspPosition {
                    line: 1,
                    character: 0,
                },
            })
        );
    }

    #[test]
    fn lsp_range_reports_start_position_for_empty_source() {
        let line_map = line_map("");

        assert_eq!(
            lsp_range_from_source_range(&line_map, source_range(&line_map, 0, 0)),
            Ok(LspRange {
                start: LspPosition {
                    line: 0,
                    character: 0,
                },
                end: LspPosition {
                    line: 0,
                    character: 0,
                },
            })
        );
    }

    #[test]
    fn lsp_range_maps_lexer_token_span_through_session_range() {
        let source = "alpha\nbeta";
        let line_map = line_map(source);
        let tokens = lex(source).expect("ASCII source should lex");
        let beta = tokens
            .iter()
            .find(|token| token.lexeme == "beta")
            .expect("token should be present");

        assert_eq!(
            source_range_from_lexer_span(line_map.source_id(), beta.span),
            source_range(&line_map, 6, 10)
        );
        assert_eq!(
            lsp_range_from_lexer_span(&line_map, line_map.source_id(), beta.span),
            Ok(LspRange {
                start: LspPosition {
                    line: 1,
                    character: 0,
                },
                end: LspPosition {
                    line: 1,
                    character: 4,
                },
            })
        );
    }

    #[test]
    fn source_range_from_lexer_span_is_a_field_copy() {
        let span = LexerSourceSpan { start: 7, end: 13 };
        let source_id = source_id(1);

        assert_eq!(
            source_range_from_lexer_span(source_id, span),
            SourceRange {
                source_id,
                start: 7,
                end: 13,
            }
        );
    }

    #[test]
    fn lsp_range_maps_lexer_token_span_with_utf16_columns() {
        let source = "a😀 beta";
        let line_map = line_map(source);
        let span = LexerSourceSpan {
            start: "a😀 ".len(),
            end: source.len(),
        };

        assert_eq!(
            lsp_range_from_lexer_span(&line_map, line_map.source_id(), span),
            Ok(LspRange {
                start: LspPosition {
                    line: 0,
                    character: 4,
                },
                end: LspPosition {
                    line: 0,
                    character: 8,
                },
            })
        );
    }

    #[test]
    fn lsp_range_from_lexer_span_rejects_reversed_spans() {
        let source = "abc";
        let line_map = line_map(source);

        assert_eq!(
            lsp_range_from_lexer_span(
                &line_map,
                line_map.source_id(),
                LexerSourceSpan { start: 2, end: 1 }
            ),
            Err(RangeMapError::SourceMap(SourceMapError::ReversedRange))
        );
    }

    #[test]
    fn lsp_range_from_lexer_span_rejects_out_of_bounds_spans() {
        let source = "abc";
        let line_map = line_map(source);
        let range = source_range(&line_map, 1, 4);

        assert_eq!(
            lsp_range_from_lexer_span(
                &line_map,
                line_map.source_id(),
                LexerSourceSpan { start: 1, end: 4 }
            ),
            Err(RangeMapError::SourceMap(
                SourceMapError::RangeOutsideSourceText {
                    range,
                    source_len: source.len(),
                }
            ))
        );
    }

    #[test]
    fn lsp_range_from_lexer_span_rejects_mismatched_source_id() {
        let source = "abc";
        let line_map = line_map(source);
        let other_source_id = source_id(2);

        assert_eq!(
            lsp_range_from_lexer_span(
                &line_map,
                other_source_id,
                LexerSourceSpan { start: 0, end: 1 }
            ),
            Err(RangeMapError::SourceMap(SourceMapError::UnknownSourceId {
                source_id: other_source_id,
            }))
        );
    }

    #[test]
    fn lsp_range_rejects_non_utf8_boundary_offsets() {
        let source = "a😀β";
        let line_map = line_map(source);

        assert_eq!(
            lsp_range_from_source_range(&line_map, source_range(&line_map, 2, 2)),
            Err(RangeMapError::SourceMap(
                SourceMapError::OffsetNotUtf8Boundary {
                    source_id: line_map.source_id(),
                    offset: 2,
                }
            ))
        );
    }

    #[test]
    fn lsp_range_rejects_reversed_ranges() {
        let source = "abc";
        let line_map = line_map(source);

        assert_eq!(
            lsp_range_from_source_range(&line_map, source_range(&line_map, 2, 1)),
            Err(RangeMapError::SourceMap(SourceMapError::ReversedRange))
        );
    }

    #[test]
    fn lsp_range_rejects_out_of_bounds_offsets() {
        let source = "abc";
        let line_map = line_map(source);
        let range = source_range(&line_map, 1, 4);

        assert_eq!(
            lsp_range_from_source_range(&line_map, range),
            Err(RangeMapError::SourceMap(
                SourceMapError::RangeOutsideSourceText {
                    range,
                    source_len: source.len(),
                }
            ))
        );
    }

    #[test]
    fn lsp_coordinate_narrowing_reports_overflow() {
        assert_eq!(super::u32_lsp_coordinate(0, u32::MAX as usize), Ok(0));
        assert_eq!(
            super::u32_lsp_coordinate(u32::MAX as usize, u32::MAX as usize),
            Ok(u32::MAX)
        );
        assert_eq!(
            super::u32_lsp_coordinate(u32::MAX as usize + 1, u32::MAX as usize),
            Err(RangeMapError::PositionOverflow)
        );
    }

    #[test]
    fn lsp_coordinates_obey_the_protocol_uinteger_limit() {
        let max = super::MAX_LSP_POSITION;
        assert_eq!(max, 2_147_483_647);
        assert_eq!(super::u32_lsp_coordinate(max, max), Ok(max as u32));
        assert_eq!(
            super::u32_lsp_coordinate(max + 1, max),
            Err(RangeMapError::PositionOverflow)
        );
    }

    #[test]
    fn fixture_ranges_match_independent_utf16_boundary_positions() {
        for source in [
            include_str!("../../../tests/miz/pass/parser/pass_parser_module_skeleton_001.miz"),
            include_str!("../../../tests/miz/pass/parser/pass_parser_atomic_formulas_001.miz"),
            "",
            "\n",
            "a😀β\ne\u{301}漢\n",
        ] {
            let line_map = line_map(source);
            let mut position = LspPosition {
                line: 0,
                character: 0,
            };
            let mut boundaries = Vec::new();
            for (offset, scalar) in source.char_indices() {
                boundaries.push((offset, position));
                if scalar == '\n' {
                    position.line += 1;
                    position.character = 0;
                } else {
                    position.character += if scalar as u32 > 0xffff { 2 } else { 1 };
                }
            }
            boundaries.push((source.len(), position));
            for (index, &(start, expected_start)) in boundaries.iter().enumerate() {
                for &(end, expected_end) in &boundaries[index..] {
                    assert_eq!(
                        lsp_range_from_source_range(&line_map, source_range(&line_map, start, end)),
                        Ok(LspRange {
                            start: expected_start,
                            end: expected_end
                        }),
                        "{source:?}: {start}..{end}"
                    );
                }
            }
        }
    }

    #[test]
    fn every_multibyte_interior_is_rejected_at_both_range_endpoints() {
        let source = "a😀βe\u{301}漢\n";
        let line_map = line_map(source);
        for offset in (0..source.len()).filter(|&offset| !source.is_char_boundary(offset)) {
            let expected = Err(RangeMapError::SourceMap(
                SourceMapError::OffsetNotUtf8Boundary {
                    source_id: line_map.source_id(),
                    offset,
                },
            ));
            for (start, end) in [(0, offset), (offset, source.len())] {
                assert_eq!(
                    lsp_range_from_source_range(&line_map, source_range(&line_map, start, end)),
                    expected
                );
            }
        }
    }

    #[test]
    fn actual_loaded_crlf_and_lf_sources_have_equivalent_ranges() {
        use mizar_session::{
            DiskSourceLoader, Edition, ModulePath, PackageId, SourceInput, SourceLoader,
            SourceOriginInput,
        };
        use std::{fs, sync::Arc};

        let root = std::env::temp_dir().join(format!("mizar-lsp-range-{}", std::process::id()));
        fs::create_dir_all(root.join("src")).unwrap();
        let path = root.join("src/input.miz");
        fs::write(&path, "disk text").unwrap();
        let uri = format!(
            "file:///{}",
            path.to_str()
                .unwrap()
                .replace('\\', "/")
                .trim_start_matches('/')
        );
        let loader = DiskSourceLoader::new(&root);
        let ids = InMemorySessionIdAllocator::new();
        let snapshot = BuildSnapshotId::from_published_schema_str(&snapshot_string(1)).unwrap();
        let sources = ["a😀\r\nβe\u{301}\r\n", "a😀\nβe\u{301}\n"].map(|text| {
            loader
                .load(
                    snapshot,
                    SourceInput {
                        package_id: PackageId::new("range"),
                        module_path: ModulePath::new("input"),
                        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                        edition: Edition::new("2026"),
                        origin: SourceOriginInput::OpenBuffer {
                            uri: uri.clone(),
                            expected_version: 1,
                            actual_version: 1,
                            text: Arc::from(text),
                        },
                    },
                    &ids,
                )
                .unwrap()
        });
        assert_eq!(sources[0].text, sources[1].text);
        assert_ne!(sources[0].source_id, sources[1].source_id);
        assert!(sources[0].loading_map.is_some());
        let text = sources[0].text.as_ref();
        let boundaries = text
            .char_indices()
            .map(|(offset, _)| offset)
            .chain(std::iter::once(text.len()))
            .collect::<Vec<_>>();
        for (index, &start) in boundaries.iter().enumerate() {
            for &end in &boundaries[index..] {
                let mapped = sources.each_ref().map(|source| {
                    lsp_range_from_source_range(
                        &source.line_map,
                        SourceRange {
                            source_id: source.source_id,
                            start,
                            end,
                        },
                    )
                    .unwrap()
                });
                assert_eq!(mapped[0], mapped[1]);
            }
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_open_buffers_map_exact_raw_editor_ranges_and_reject_mismatched_context() {
        use mizar_session::{
            DiskSourceLoader, Edition, LoadingMap, LoadingMapSegment, LoadingOrigin, ModulePath,
            PackageId, SourceInput, SourceLoader, SourceOriginInput, TextRange,
        };
        use std::{fs, sync::Arc};
        let root = std::env::temp_dir().join(format!("mizar-lsp-raw-range-{}", std::process::id()));
        fs::create_dir_all(root.join("src")).unwrap();
        let path = root.join("src/input.miz");
        fs::write(&path, "unrelated disk text").unwrap();
        let uri = format!(
            "file:///{}",
            path.to_str()
                .unwrap()
                .replace('\\', "/")
                .trim_start_matches('/')
        );
        let loader = DiskSourceLoader::new(&root);
        let ids = InMemorySessionIdAllocator::new();
        let snapshot = BuildSnapshotId::from_published_schema_str(&snapshot_string(1)).unwrap();
        for (index, (raw, normalized, points)) in [
            (
                "a😀e\u{301}",
                "a😀e\u{301}",
                vec![(0, 0, 0), (1, 0, 1), (5, 0, 3), (6, 0, 4), (8, 0, 5)],
            ),
            ("\u{feff}a😀", "a😀", vec![(0, 0, 1), (1, 0, 2), (5, 0, 4)]),
            (
                "a😀\r\nβ\r\n",
                "a😀\nβ\n",
                vec![
                    (0, 0, 0),
                    (1, 0, 1),
                    (5, 0, 3),
                    (6, 1, 0),
                    (8, 1, 1),
                    (9, 2, 0),
                ],
            ),
            (
                "a\rb",
                "a\rb",
                vec![(0, 0, 0), (1, 0, 1), (2, 1, 0), (3, 1, 1)],
            ),
            (
                "a\r\nb\rc\n",
                "a\nb\rc\n",
                vec![
                    (0, 0, 0),
                    (1, 0, 1),
                    (2, 1, 0),
                    (3, 1, 1),
                    (4, 2, 0),
                    (5, 2, 1),
                    (6, 3, 0),
                ],
            ),
            (
                "a\r\n\r\n",
                "a\n\n",
                vec![(0, 0, 0), (1, 0, 1), (2, 1, 0), (3, 2, 0)],
            ),
            ("a\n", "a\n", vec![(0, 0, 0), (1, 0, 1), (2, 1, 0)]),
            ("a\r", "a\r", vec![(0, 0, 0), (1, 0, 1), (2, 1, 0)]),
            (
                "😀\r\n\u{301}",
                "😀\n\u{301}",
                vec![(0, 0, 0), (4, 0, 2), (5, 1, 0), (7, 1, 1)],
            ),
            ("", "", vec![(0, 0, 0)]),
            ("\u{feff}", "", vec![(0, 0, 1)]),
            ("\r\n", "\n", vec![(0, 0, 0), (1, 1, 0)]),
            ("\r", "\r", vec![(0, 0, 0), (1, 1, 0)]),
            (
                "\u{feff}a\r\nβ",
                "a\nβ",
                vec![(0, 0, 1), (1, 0, 2), (2, 1, 0), (4, 1, 1)],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let version = match index {
                0 => 0,
                3 => i64::from(i32::MAX) + 1,
                4 => i64::MAX,
                _ => 1,
            };
            let origin = SourceOriginInput::OpenBuffer {
                uri: uri.clone(),
                expected_version: version,
                actual_version: version,
                text: Arc::from(raw),
            };
            let loaded = loader
                .load(
                    snapshot,
                    SourceInput {
                        package_id: PackageId::new("range"),
                        module_path: ModulePath::new("input"),
                        normalized_path: mizar_session::normalize_path(&root, &path).unwrap(),
                        edition: Edition::new("2026"),
                        origin: origin.clone(),
                    },
                    &ids,
                )
                .unwrap();
            assert_eq!(loaded.text.as_ref(), normalized);
            let mapped = |map: Option<&LoadingMap>, origin: &SourceOriginInput, start, end| {
                super::lsp_range_from_open_buffer(
                    &loaded.line_map,
                    map,
                    origin,
                    source_range(&loaded.line_map, start, end),
                )
            };
            for (position, &(start, start_line, start_character)) in points.iter().enumerate() {
                for &(end, end_line, end_character) in &points[position..] {
                    assert_eq!(
                        mapped(loaded.loading_map.as_ref(), &origin, start, end),
                        Ok(LspRange {
                            start: LspPosition {
                                line: start_line,
                                character: start_character
                            },
                            end: LspPosition {
                                line: end_line,
                                character: end_character
                            },
                        }),
                        "{raw:?}: {start}..{end}"
                    );
                }
            }
            let other = ids.next_source_id(snapshot).unwrap();
            let invalid_origin = SourceOriginInput::Disk { path: path.clone() };
            for (range, error) in [
                (
                    SourceRange {
                        source_id: other,
                        start: 0,
                        end: 0,
                    },
                    SourceMapError::UnknownSourceId { source_id: other },
                ),
                (
                    source_range(&loaded.line_map, 1, 0),
                    SourceMapError::ReversedRange,
                ),
                (
                    source_range(&loaded.line_map, 0, normalized.len() + 1),
                    SourceMapError::RangeOutsideSourceText {
                        range: source_range(&loaded.line_map, 0, normalized.len() + 1),
                        source_len: normalized.len(),
                    },
                ),
            ] {
                assert_eq!(
                    super::lsp_range_from_open_buffer(
                        &loaded.line_map,
                        loaded.loading_map.as_ref(),
                        &invalid_origin,
                        range
                    ),
                    Err(RangeMapError::SourceMap(error))
                );
            }
            for offset in
                (0..normalized.len()).filter(|&offset| !normalized.is_char_boundary(offset))
            {
                for (start, end) in [(0, offset), (offset, normalized.len())] {
                    assert_eq!(
                        mapped(loaded.loading_map.as_ref(), &invalid_origin, start, end),
                        Err(RangeMapError::SourceMap(
                            SourceMapError::OffsetNotUtf8Boundary {
                                source_id: loaded.source_id,
                                offset
                            }
                        ))
                    );
                }
            }
            assert_eq!(
                mapped(loaded.loading_map.as_ref(), &invalid_origin, 0, 0),
                Err(RangeMapError::InvalidOpenBufferContext)
            );
            let generated = SourceOriginInput::Generated {
                generator: mizar_session::GeneratedSourceKind::new("test"),
                text: Arc::from(raw),
                anchor: None,
            };
            assert_eq!(
                mapped(loaded.loading_map.as_ref(), &generated, 0, 0),
                Err(RangeMapError::InvalidOpenBufferContext)
            );
            for change in 0..4 {
                let mut bad = origin.clone();
                let SourceOriginInput::OpenBuffer {
                    expected_version,
                    actual_version,
                    text,
                    ..
                } = &mut bad
                else {
                    unreachable!()
                };
                match change {
                    0 => *expected_version = version - 1,
                    1 => {
                        *expected_version = -1;
                        *actual_version = -1;
                    }
                    2 => *text = Arc::from(format!("{raw}x")),
                    _ => *text = Arc::from(format!("x{raw}")),
                }
                assert_eq!(
                    mapped(loaded.loading_map.as_ref(), &bad, 0, 0),
                    Err(RangeMapError::InvalidOpenBufferContext)
                );
            }
            let wrong_text = LineMap::new(loaded.source_id, "different loaded text");
            assert_eq!(
                super::lsp_range_from_open_buffer(
                    &wrong_text,
                    loaded.loading_map.as_ref(),
                    &origin,
                    source_range(&wrong_text, 0, 0)
                ),
                Err(RangeMapError::InvalidOpenBufferContext)
            );
            if let Some(map) = &loaded.loading_map {
                assert_eq!(
                    mapped(None, &origin, 0, 0),
                    Err(RangeMapError::InvalidOpenBufferContext)
                );
                for change in 0..8 {
                    let mut bad = map.clone();
                    match change {
                        0 => bad.source_id = other,
                        1 => bad.loaded_text_hash = Hash::from_bytes([0; Hash::BYTE_LEN]),
                        2 => bad.loaded_text_len += 1,
                        3 => bad.origin = LoadingOrigin::Generated,
                        4 => {
                            bad.origin = LoadingOrigin::OpenBufferText {
                                uri: format!("{uri}other"),
                                version,
                            }
                        }
                        5 => {
                            bad.origin = LoadingOrigin::OpenBufferText {
                                uri: uri.clone(),
                                version: 0,
                            }
                        }
                        6 => {
                            bad.segments.pop();
                        }
                        _ => bad.segments.push(LoadingMapSegment::Original {
                            loaded: TextRange {
                                start: 0,
                                end: normalized.len(),
                            },
                            original: TextRange {
                                start: 0,
                                end: raw.len(),
                            },
                        }),
                    }
                    assert_eq!(
                        mapped(Some(&bad), &origin, 0, 0),
                        Err(RangeMapError::InvalidOpenBufferContext),
                        "map mutation {change}: {raw:?}"
                    );
                }
                if index == 2 {
                    let LoadingMapSegment::NormalizedNewline { loaded, original } =
                        &map.segments[1]
                    else {
                        panic!("native CRLF segment");
                    };
                    for change in 0..3 {
                        let mut bad = map.clone();
                        match change {
                            0 => {
                                bad.segments[1] = LoadingMapSegment::NormalizedNewline {
                                    loaded: *loaded,
                                    original: TextRange {
                                        end: original.end - 1,
                                        ..*original
                                    },
                                }
                            }
                            1 => {
                                bad.segments[1] = LoadingMapSegment::Original {
                                    loaded: *loaded,
                                    original: *original,
                                }
                            }
                            _ => bad.segments.swap(2, 3),
                        }
                        assert_eq!(
                            mapped(Some(&bad), &origin, 0, 0),
                            Err(RangeMapError::InvalidOpenBufferContext),
                            "in-place native segment mutation {change}"
                        );
                    }
                }
                let mut wrong_uri = origin.clone();
                let SourceOriginInput::OpenBuffer { uri, .. } = &mut wrong_uri else {
                    unreachable!()
                };
                uri.push_str("other");
                assert_eq!(
                    mapped(Some(map), &wrong_uri, 0, 0),
                    Err(RangeMapError::InvalidOpenBufferContext)
                );
                let mut wrong_raw = origin.clone();
                let SourceOriginInput::OpenBuffer { text, .. } = &mut wrong_raw else {
                    unreachable!()
                };
                *text = Arc::from(normalized);
                assert_eq!(
                    mapped(Some(map), &wrong_raw, 0, 0),
                    Err(RangeMapError::InvalidOpenBufferContext)
                );
            } else {
                let invented = LoadingMap::identity(
                    loaded.source_id,
                    normalized,
                    LoadingOrigin::OpenBufferText {
                        uri: uri.clone(),
                        version,
                    },
                );
                assert_eq!(
                    mapped(Some(&invented), &origin, 0, 0),
                    Err(RangeMapError::InvalidOpenBufferContext)
                );
            }
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lsp_range_reports_utf16_overflow_through_range_conversion_path() {
        let source = "a😀β";
        let line_map = line_map(source);

        assert_eq!(
            super::lsp_range_from_source_range_with_max_utf16_column(
                &line_map,
                source_range(&line_map, 0, source.len()),
                3,
            ),
            Err(RangeMapError::PositionOverflow)
        );
        assert_eq!(
            super::lsp_range_from_source_range_with_max_utf16_column(
                &line_map,
                source_range(&line_map, 0, source.len()),
                4,
            ),
            Ok(LspRange {
                start: LspPosition {
                    line: 0,
                    character: 0,
                },
                end: LspPosition {
                    line: 0,
                    character: 4,
                },
            })
        );
    }

    fn line_map(source: &str) -> LineMap {
        LineMap::with_source(source_id(1), source)
    }

    fn source_range(line_map: &LineMap, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id: line_map.source_id(),
            start,
            end,
        }
    }

    fn source_id(seed: u8) -> SourceId {
        let allocator = InMemorySessionIdAllocator::new();
        let snapshot = BuildSnapshotId::from_published_schema_str(&snapshot_string(seed)).unwrap();
        let mut source_id = allocator.next_source_id(snapshot).unwrap();
        for _ in 1..seed {
            source_id = allocator.next_source_id(snapshot).unwrap();
        }
        source_id
    }

    fn snapshot_string(seed: u8) -> String {
        let bytes = [seed; Hash::BYTE_LEN];
        let mut serialized = String::from("mizar-session-build-snapshot-v1:");
        for byte in bytes {
            serialized.push_str(&format!("{byte:02x}"));
        }
        serialized
    }
}
