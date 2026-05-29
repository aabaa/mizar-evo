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
}

impl From<SourceMapError> for RangeMapError {
    fn from(error: SourceMapError) -> Self {
        Self::SourceMap(error)
    }
}

pub fn lsp_range_from_source_range(
    line_map: &LineMap,
    range: SourceRange,
) -> Result<LspRange, RangeMapError> {
    line_map.line_column_range(range)?;
    Ok(LspRange {
        start: lsp_position(line_map, range.start)?,
        end: lsp_position(line_map, range.end)?,
    })
}

fn lsp_position(line_map: &LineMap, offset: usize) -> Result<LspPosition, RangeMapError> {
    let line_column = line_map.line_column(offset)?;
    let line_start = line_start_offset(line_map.source(), offset);
    let utf16_column = line_map.source()[line_start..offset].encode_utf16().count() as u32;
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

#[cfg(test)]
mod tests {
    use mizar_session::{LineColumn, LineMap, SourceMapError, SourceRange};

    use super::{LspPosition, LspRange, RangeMapError, lsp_range_from_source_range};

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
}
