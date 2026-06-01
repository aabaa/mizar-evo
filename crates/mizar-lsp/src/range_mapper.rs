use mizar_lexer::SourceSpan as LexerSourceSpan;
use mizar_session::{LineMap, SourceId, SourceMapError, SourceRange};

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
}

const MAX_LSP_POSITION: usize = u32::MAX as usize;

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
        line: line_column.line - 1,
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
