use mizar_lexer::SourceSpan as LexerSourceSpan;
use mizar_session::{LineMap, SourceMapError, SourceRange};

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
    span: LexerSourceSpan,
) -> Result<LspRange, RangeMapError> {
    // Keep this bridge explicit while no frontend/diagnostic adapter crate owns
    // lexer-to-session coordinate conversion.
    lsp_range_from_source_range(line_map, source_range_from_lexer_span(span))
}

pub fn source_range_from_lexer_span(span: LexerSourceSpan) -> SourceRange {
    // This is intentionally a field copy: lexer and session source ranges both
    // use byte offsets into the loaded text, but remain owned by their crates.
    SourceRange {
        start: span.start,
        end: span.end,
    }
}

fn lsp_position(
    line_map: &LineMap,
    offset: usize,
    max_utf16_column: usize,
) -> Result<LspPosition, RangeMapError> {
    let line_column = line_map.line_column(offset)?;
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
    use mizar_session::{LineColumn, LineMap, SourceMapError, SourceRange};

    use super::{
        LspPosition, LspRange, RangeMapError, lsp_range_from_lexer_span,
        lsp_range_from_source_range, source_range_from_lexer_span,
    };

    #[test]
    fn lsp_range_uses_zero_based_utf16_columns_at_lsp_boundary() {
        let source = "a😀β\n漢z";
        let line_map = LineMap::new(source);
        let start = "a".len();
        let end = "a😀β".len();

        assert_eq!(
            line_map.line_column(end),
            Ok(LineColumn { line: 1, column: 4 })
        );
        assert_eq!(
            lsp_range_from_source_range(&line_map, SourceRange { start, end }),
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
        let line_map = LineMap::new(source);
        let start = "e".len();
        let end = "e\u{301}".len();

        assert_eq!(
            lsp_range_from_source_range(&line_map, SourceRange { start, end }),
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
        let line_map = LineMap::new(source);
        let start = "a😀β\n".len();
        let end = "a😀β\n漢z".len();

        assert_eq!(
            lsp_range_from_source_range(&line_map, SourceRange { start, end }),
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
        let line_map = LineMap::new(source);
        let start = "ab".len();
        let end = "ab😀\nβ".len();

        assert_eq!(
            lsp_range_from_source_range(&line_map, SourceRange { start, end }),
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
        let line_map = LineMap::new(source);

        assert_eq!(
            lsp_range_from_source_range(
                &line_map,
                SourceRange {
                    start: source.len(),
                    end: source.len(),
                },
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
        let line_map = LineMap::new("");

        assert_eq!(
            lsp_range_from_source_range(&line_map, SourceRange { start: 0, end: 0 }),
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
        let line_map = LineMap::new(source);
        let tokens = lex(source).expect("ASCII source should lex");
        let beta = tokens
            .iter()
            .find(|token| token.lexeme == "beta")
            .expect("token should be present");

        assert_eq!(
            source_range_from_lexer_span(beta.span),
            SourceRange { start: 6, end: 10 }
        );
        assert_eq!(
            lsp_range_from_lexer_span(&line_map, beta.span),
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

        assert_eq!(
            source_range_from_lexer_span(span),
            SourceRange { start: 7, end: 13 }
        );
    }

    #[test]
    fn lsp_range_maps_lexer_token_span_with_utf16_columns() {
        let source = "a😀 beta";
        let line_map = LineMap::new(source);
        let span = LexerSourceSpan {
            start: "a😀 ".len(),
            end: source.len(),
        };

        assert_eq!(
            lsp_range_from_lexer_span(&line_map, span),
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
        let line_map = LineMap::new(source);

        assert_eq!(
            lsp_range_from_lexer_span(&line_map, LexerSourceSpan { start: 2, end: 1 }),
            Err(RangeMapError::SourceMap(SourceMapError::ReversedRange))
        );
    }

    #[test]
    fn lsp_range_from_lexer_span_rejects_out_of_bounds_spans() {
        let source = "abc";
        let line_map = LineMap::new(source);

        assert_eq!(
            lsp_range_from_lexer_span(&line_map, LexerSourceSpan { start: 1, end: 4 }),
            Err(RangeMapError::SourceMap(SourceMapError::OffsetOutOfBounds))
        );
    }

    #[test]
    fn lsp_range_rejects_non_utf8_boundary_offsets() {
        let source = "a😀β";
        let line_map = LineMap::new(source);

        assert_eq!(
            lsp_range_from_source_range(&line_map, SourceRange { start: 2, end: 2 }),
            Err(RangeMapError::SourceMap(
                SourceMapError::OffsetNotCharBoundary
            ))
        );
    }

    #[test]
    fn lsp_range_rejects_reversed_ranges() {
        let source = "abc";
        let line_map = LineMap::new(source);

        assert_eq!(
            lsp_range_from_source_range(&line_map, SourceRange { start: 2, end: 1 }),
            Err(RangeMapError::SourceMap(SourceMapError::ReversedRange))
        );
    }

    #[test]
    fn lsp_range_rejects_out_of_bounds_offsets() {
        let source = "abc";
        let line_map = LineMap::new(source);

        assert_eq!(
            lsp_range_from_source_range(&line_map, SourceRange { start: 1, end: 4 }),
            Err(RangeMapError::SourceMap(SourceMapError::OffsetOutOfBounds))
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
        let line_map = LineMap::new(source);

        assert_eq!(
            super::lsp_range_from_source_range_with_max_utf16_column(
                &line_map,
                SourceRange {
                    start: 0,
                    end: source.len(),
                },
                3,
            ),
            Err(RangeMapError::PositionOverflow)
        );
        assert_eq!(
            super::lsp_range_from_source_range_with_max_utf16_column(
                &line_map,
                SourceRange {
                    start: 0,
                    end: source.len(),
                },
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
}
