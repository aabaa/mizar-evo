use crate::{Hash, SourceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub source_id: SourceId,
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
    source_id: SourceId,
    text_hash: Hash,
    line_starts: Vec<usize>,
    text: String,
    char_boundaries: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceMapError {
    UnknownSourceId {
        source_id: SourceId,
    },
    ReversedRange,
    RangeOutsideSourceText {
        range: SourceRange,
        source_len: usize,
    },
    OffsetNotUtf8Boundary {
        source_id: SourceId,
        offset: usize,
    },
    LineColumnOverflow,
}

const MAX_LINE_COLUMN: usize = u32::MAX as usize;
const SOURCE_TEXT_HASH_DOMAIN: &[u8] = b"mizar-session/source-text/v1";

impl LineMap {
    pub fn new(source_id: SourceId, source: &str) -> Self {
        Self::with_source(source_id, source)
    }

    pub fn with_source(source_id: SourceId, source: &str) -> Self {
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
            source_id,
            line_starts,
            text_hash: hash_source_text(source),
            text: source.to_owned(),
            char_boundaries,
        }
    }

    pub fn source(&self) -> &str {
        &self.text
    }

    pub fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub fn text_hash(&self) -> Hash {
        self.text_hash
    }

    pub fn line_starts(&self) -> &[usize] {
        &self.line_starts
    }

    fn line_column(&self, offset: usize) -> Result<LineColumn, SourceMapError> {
        self.line_column_for_source(self.source_id, offset)
    }

    pub fn line_column_for_source(
        &self,
        source_id: SourceId,
        offset: usize,
    ) -> Result<LineColumn, SourceMapError> {
        self.validate_source_id(source_id)?;
        self.line_column_with_max(offset, MAX_LINE_COLUMN)
    }

    fn line_column_with_max(
        &self,
        offset: usize,
        max_coordinate: usize,
    ) -> Result<LineColumn, SourceMapError> {
        self.validate_offset(self.source_id, offset)?;
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(0) => {
                return Err(SourceMapError::RangeOutsideSourceText {
                    range: SourceRange {
                        source_id: self.source_id,
                        start: offset,
                        end: offset,
                    },
                    source_len: self.text.len(),
                });
            }
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
        self.validate_range(range)?;
        Ok(LineColumnRange {
            start: self.line_column(range.start)?,
            end: self.line_column(range.end)?,
        })
    }

    pub fn validate_range(&self, range: SourceRange) -> Result<(), SourceMapError> {
        self.validate_source_id(range.source_id)?;
        if range.start > range.end {
            return Err(SourceMapError::ReversedRange);
        }
        if range.end > self.text.len() {
            return Err(SourceMapError::RangeOutsideSourceText {
                range,
                source_len: self.text.len(),
            });
        }
        self.validate_offset(range.source_id, range.start)?;
        self.validate_offset(range.source_id, range.end)?;
        Ok(())
    }

    fn validate_source_id(&self, source_id: SourceId) -> Result<(), SourceMapError> {
        if source_id != self.source_id {
            return Err(SourceMapError::UnknownSourceId { source_id });
        }
        Ok(())
    }

    fn validate_offset(&self, source_id: SourceId, offset: usize) -> Result<(), SourceMapError> {
        if offset > self.text.len() {
            return Err(SourceMapError::RangeOutsideSourceText {
                range: SourceRange {
                    source_id,
                    start: offset,
                    end: offset,
                },
                source_len: self.text.len(),
            });
        }
        if self.char_boundaries.binary_search(&offset).is_err() {
            return Err(SourceMapError::OffsetNotUtf8Boundary { source_id, offset });
        }
        Ok(())
    }
}

fn hash_source_text(source: &str) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SOURCE_TEXT_HASH_DOMAIN);
    hasher.update(&(source.len() as u64).to_le_bytes());
    hasher.update(source.as_bytes());
    Hash::from_bytes(*hasher.finalize().as_bytes())
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
    use crate::{BuildSnapshotId, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId};

    #[test]
    fn line_map_reports_one_based_unicode_scalar_columns() {
        let source = "aβ😀z\n漢字";
        let map = line_map(source);

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
        let map = line_map(source);

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
        let map = line_map(source);

        assert_eq!(
            map.line_column(2),
            Err(SourceMapError::OffsetNotUtf8Boundary {
                source_id: map.source_id(),
                offset: 2,
            })
        );
        assert_eq!(
            map.line_column(source.len() + 1),
            Err(SourceMapError::RangeOutsideSourceText {
                range: SourceRange {
                    source_id: map.source_id(),
                    start: source.len() + 1,
                    end: source.len() + 1,
                },
                source_len: source.len(),
            })
        );
        assert_eq!(
            map.line_column_range(SourceRange {
                source_id: map.source_id(),
                start: 5,
                end: 4,
            }),
            Err(SourceMapError::ReversedRange)
        );
    }

    #[test]
    fn line_map_converts_ranges_with_unicode_scalar_columns() {
        let source = "alpha\nβ😀z\nomega";
        let map = line_map(source);
        let start = "alpha\nβ".len();
        let end = "alpha\nβ😀".len();

        assert_eq!(
            map.line_column_range(SourceRange {
                source_id: map.source_id(),
                start,
                end,
            }),
            Ok(LineColumnRange {
                start: LineColumn { line: 2, column: 2 },
                end: LineColumn { line: 2, column: 3 },
            })
        );
    }

    #[test]
    fn line_map_converts_ranges_across_lines() {
        let source = "ab😀\nβc";
        let map = line_map(source);
        let start = "ab".len();
        let end = "ab😀\nβ".len();

        assert_eq!(
            map.line_column_range(SourceRange {
                source_id: map.source_id(),
                start,
                end,
            }),
            Ok(LineColumnRange {
                start: LineColumn { line: 1, column: 3 },
                end: LineColumn { line: 2, column: 2 },
            })
        );
    }

    #[test]
    fn line_map_reports_next_line_for_offsets_after_trailing_newlines() {
        let source = "alpha\n";
        let map = line_map(source);

        assert_eq!(
            map.line_column(source.len()),
            Ok(LineColumn { line: 2, column: 1 })
        );
    }

    #[test]
    fn line_map_reports_first_position_for_empty_source() {
        let map = line_map("");

        assert_eq!(map.line_column(0), Ok(LineColumn { line: 1, column: 1 }));
        assert_eq!(
            map.line_column_range(SourceRange {
                source_id: map.source_id(),
                start: 0,
                end: 0,
            }),
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
        let map = line_map(source);

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

    #[test]
    fn line_map_records_source_identity_and_text_hash() {
        let first_source_id = source_id(1);
        let map = LineMap::with_source(first_source_id, "abc");
        let same = LineMap::new(first_source_id, "abc");
        let same_text_different_source = LineMap::with_source(source_id(2), "abc");
        let different = LineMap::with_source(first_source_id, "abcd");

        assert_eq!(map.source_id(), first_source_id);
        assert_eq!(map.source(), "abc");
        assert_eq!(map.text_hash(), same.text_hash());
        assert_eq!(map.text_hash(), same_text_different_source.text_hash());
        assert_ne!(map.text_hash(), different.text_hash());
        assert_eq!(map.line_starts(), &[0]);
    }

    #[test]
    fn line_map_accepts_matching_source_id_for_offset_and_range_conversion() {
        let map = line_map("alpha\nbeta");
        let range = SourceRange {
            source_id: map.source_id(),
            start: "alpha\n".len(),
            end: "alpha\nbeta".len(),
        };

        assert_eq!(
            map.line_column_for_source(map.source_id(), range.start),
            Ok(LineColumn { line: 2, column: 1 })
        );
        assert_eq!(
            map.line_column_range(range),
            Ok(LineColumnRange {
                start: LineColumn { line: 2, column: 1 },
                end: LineColumn { line: 2, column: 5 },
            })
        );
    }

    #[test]
    fn line_map_rejects_cross_source_ranges() {
        let map = line_map("abc");
        let other_source_id = source_id(2);

        assert_eq!(
            map.line_column_range(SourceRange {
                source_id: other_source_id,
                start: 0,
                end: 1,
            }),
            Err(SourceMapError::UnknownSourceId {
                source_id: other_source_id,
            })
        );
    }

    #[test]
    fn line_map_rejects_unknown_source_id_for_offset_conversion() {
        let map = line_map("abc");
        let unknown_source_id = source_id(3);

        assert_eq!(
            map.line_column_for_source(unknown_source_id, 0),
            Err(SourceMapError::UnknownSourceId {
                source_id: unknown_source_id,
            })
        );
    }

    #[test]
    fn line_map_rejects_ranges_outside_source_text() {
        let source = "abc";
        let map = line_map(source);
        let range = SourceRange {
            source_id: map.source_id(),
            start: 1,
            end: 4,
        };

        assert_eq!(
            map.line_column_range(range),
            Err(SourceMapError::RangeOutsideSourceText {
                range,
                source_len: source.len(),
            })
        );
    }

    #[test]
    fn line_map_validate_range_checks_public_range_contract() {
        let source = "aβ";
        let map = line_map(source);
        let valid = SourceRange {
            source_id: map.source_id(),
            start: "a".len(),
            end: source.len(),
        };
        let invalid_boundary = SourceRange {
            source_id: map.source_id(),
            start: 2,
            end: 2,
        };

        assert_eq!(map.validate_range(valid), Ok(()));
        assert_eq!(
            map.validate_range(invalid_boundary),
            Err(SourceMapError::OffsetNotUtf8Boundary {
                source_id: map.source_id(),
                offset: 2,
            })
        );
    }

    fn line_map(source: &str) -> LineMap {
        LineMap::with_source(source_id(1), source)
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
