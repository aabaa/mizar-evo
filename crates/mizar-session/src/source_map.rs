#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineColumn {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineColumnRange {
    pub start: LineColumn,
    pub end: LineColumn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineMap {
    text: String,
    line_starts: Vec<usize>,
    char_boundaries: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceMapError {
    ReversedRange,
    OffsetOutOfBounds,
    OffsetNotCharBoundary,
    LineColumnOverflow,
}

const MAX_LINE_COLUMN: usize = u32::MAX as usize;

impl LineMap {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        let mut char_boundaries = Vec::new();
        for (index, ch) in source.char_indices() {
            char_boundaries.push(index);
            if ch == '\n' {
                line_starts.push(index + ch.len_utf8());
            }
        }
        char_boundaries.push(source.len());
        Self {
            text: source.to_owned(),
            line_starts,
            char_boundaries,
        }
    }

    pub fn source(&self) -> &str {
        &self.text
    }

    pub fn line_column(&self, offset: usize) -> Result<LineColumn, SourceMapError> {
        self.line_column_with_max(offset, MAX_LINE_COLUMN)
    }

    fn line_column_with_max(
        &self,
        offset: usize,
        max_coordinate: usize,
    ) -> Result<LineColumn, SourceMapError> {
        self.validate_offset(offset)?;
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(0) => return Err(SourceMapError::OffsetOutOfBounds),
            Err(next_line) => next_line - 1,
        };
        let line_start = self.line_starts[line_index];
        let column_index = self.text[line_start..offset].chars().count();
        Ok(LineColumn {
            line: one_based_u32(line_index, max_coordinate)?,
            column: one_based_u32(column_index, max_coordinate)?,
        })
    }

    pub fn line_column_range(&self, range: SourceRange) -> Result<LineColumnRange, SourceMapError> {
        if range.start > range.end {
            return Err(SourceMapError::ReversedRange);
        }
        Ok(LineColumnRange {
            start: self.line_column(range.start)?,
            end: self.line_column(range.end)?,
        })
    }

    fn validate_offset(&self, offset: usize) -> Result<(), SourceMapError> {
        if offset > self.text.len() {
            return Err(SourceMapError::OffsetOutOfBounds);
        }
        if self.char_boundaries.binary_search(&offset).is_err() {
            return Err(SourceMapError::OffsetNotCharBoundary);
        }
        Ok(())
    }
}

fn one_based_u32(zero_based: usize, max_coordinate: usize) -> Result<u32, SourceMapError> {
    let one_based = zero_based
        .checked_add(1)
        .ok_or(SourceMapError::LineColumnOverflow)?;
    if one_based > max_coordinate {
        return Err(SourceMapError::LineColumnOverflow);
    }
    u32::try_from(one_based).map_err(|_| SourceMapError::LineColumnOverflow)
}

#[cfg(test)]
mod tests {
    use super::{LineColumn, LineColumnRange, LineMap, SourceMapError, SourceRange};

    #[test]
    fn line_map_reports_one_based_unicode_scalar_columns() {
        let source = "aβ😀z\n漢字";
        let map = LineMap::new(source);

        assert_eq!(map.line_column(0), Ok(LineColumn { line: 1, column: 1 }));
        assert_eq!(
            map.line_column("a".len()),
            Ok(LineColumn { line: 1, column: 2 })
        );
        assert_eq!(
            map.line_column("aβ".len()),
            Ok(LineColumn { line: 1, column: 3 })
        );
        assert_eq!(
            map.line_column("aβ😀".len()),
            Ok(LineColumn { line: 1, column: 4 })
        );
        assert_eq!(
            map.line_column("aβ😀z\n".len()),
            Ok(LineColumn { line: 2, column: 1 })
        );
        assert_eq!(
            map.line_column(source.len()),
            Ok(LineColumn { line: 2, column: 3 })
        );
    }

    #[test]
    fn line_map_counts_combining_marks_as_unicode_scalars() {
        let source = "e\u{301}x";
        let map = LineMap::new(source);

        assert_eq!(
            map.line_column("e".len()),
            Ok(LineColumn { line: 1, column: 2 })
        );
        assert_eq!(
            map.line_column("e\u{301}".len()),
            Ok(LineColumn { line: 1, column: 3 })
        );
        assert_eq!(
            map.line_column(source.len()),
            Ok(LineColumn { line: 1, column: 4 })
        );
    }

    #[test]
    fn line_map_rejects_invalid_byte_offsets_and_ranges() {
        let source = "aβ😀z\n漢字";
        let map = LineMap::new(source);

        assert_eq!(
            map.line_column(2),
            Err(SourceMapError::OffsetNotCharBoundary)
        );
        assert_eq!(
            map.line_column(source.len() + 1),
            Err(SourceMapError::OffsetOutOfBounds)
        );
        assert_eq!(
            map.line_column_range(SourceRange { start: 5, end: 4 }),
            Err(SourceMapError::ReversedRange)
        );
    }

    #[test]
    fn line_map_converts_ranges_with_unicode_scalar_columns() {
        let source = "alpha\nβ😀z\nomega";
        let map = LineMap::new(source);
        let start = "alpha\nβ".len();
        let end = "alpha\nβ😀".len();

        assert_eq!(
            map.line_column_range(SourceRange { start, end }),
            Ok(LineColumnRange {
                start: LineColumn { line: 2, column: 2 },
                end: LineColumn { line: 2, column: 3 },
            })
        );
    }

    #[test]
    fn line_map_converts_ranges_across_lines() {
        let source = "ab😀\nβc";
        let map = LineMap::new(source);
        let start = "ab".len();
        let end = "ab😀\nβ".len();

        assert_eq!(
            map.line_column_range(SourceRange { start, end }),
            Ok(LineColumnRange {
                start: LineColumn { line: 1, column: 3 },
                end: LineColumn { line: 2, column: 2 },
            })
        );
    }

    #[test]
    fn line_map_reports_next_line_for_offsets_after_trailing_newlines() {
        let source = "alpha\n";
        let map = LineMap::new(source);

        assert_eq!(
            map.line_column(source.len()),
            Ok(LineColumn { line: 2, column: 1 })
        );
    }

    #[test]
    fn line_map_reports_first_position_for_empty_source() {
        let map = LineMap::new("");

        assert_eq!(map.line_column(0), Ok(LineColumn { line: 1, column: 1 }));
        assert_eq!(
            map.line_column_range(SourceRange { start: 0, end: 0 }),
            Ok(LineColumnRange {
                start: LineColumn { line: 1, column: 1 },
                end: LineColumn { line: 1, column: 1 },
            })
        );
    }

    #[test]
    fn line_map_narrowing_reports_overflow_for_unrepresentable_coordinates() {
        assert_eq!(super::one_based_u32(0, u32::MAX as usize), Ok(1));
        assert_eq!(
            super::one_based_u32(u32::MAX as usize - 1, u32::MAX as usize),
            Ok(u32::MAX)
        );
        assert_eq!(
            super::one_based_u32(u32::MAX as usize, u32::MAX as usize),
            Err(SourceMapError::LineColumnOverflow)
        );
    }

    #[test]
    fn line_map_reports_overflow_through_coordinate_conversion_path() {
        let source = "aβ😀z\n漢字";
        let map = LineMap::new(source);

        assert_eq!(
            map.line_column_with_max("aβ😀".len(), 3),
            Err(SourceMapError::LineColumnOverflow)
        );
        assert_eq!(
            map.line_column_with_max("aβ😀z\n".len(), 1),
            Err(SourceMapError::LineColumnOverflow)
        );
        assert_eq!(
            map.line_column_with_max("aβ😀".len(), 4),
            Ok(LineColumn { line: 1, column: 4 })
        );
    }
}
