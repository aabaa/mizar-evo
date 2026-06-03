use super::{
    CommentKind, GeneratedSpanAnchor, GeneratedSpanOrigin, LexicalSourceMapping,
    LexicalSourceMappingKind, LineColumn, LineColumnRange, LineMap, LoadedToOriginalRange,
    LoadedToOriginalRangeKind, LoadingMap, LoadingMapSegment, LoadingOrigin, MappedSourceRange,
    MappedSourceRangeKind, PreprocessMap, PreprocessSegment, RetainedSourceMapService,
    SourceAnchor, SourceMapError, SourceMapService, SourceRange, TextRange,
};
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
    assert_eq!(
        super::one_based_u32(usize::MAX, usize::MAX),
        Err(SourceMapError::LineColumnOverflow)
    );
}

#[test]
fn line_map_narrowing_uses_checked_conversion_even_when_limit_is_larger() {
    let unrepresentable_u32_coordinate = u32::MAX as usize;

    assert_eq!(
        super::one_based_u32(unrepresentable_u32_coordinate, usize::MAX),
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

#[test]
fn loading_map_identity_maps_loaded_offsets_and_ranges_without_offset_changes() {
    let source_id = source_id(1);
    let map = LoadingMap::identity(source_id, "alpha\nβ", open_buffer_origin());

    assert_eq!(map.source_id(), source_id);
    assert_eq!(map.loaded_len(), "alpha\nβ".len());
    assert_eq!(map.loaded_text_len, "alpha\nβ".len());
    assert_eq!(map.loaded_text_hash(), super::hash_source_text("alpha\nβ"));
    assert_eq!(map.original_offset_for_loaded(source_id, 0), Ok(0));
    assert_eq!(
        map.original_offset_for_loaded(source_id, "alpha\n".len()),
        Ok("alpha\n".len())
    );
    assert_eq!(
        map.original_range_for_loaded(
            source_id,
            TextRange {
                start: "alpha".len(),
                end: "alpha\nβ".len(),
            },
        ),
        Ok(LoadedToOriginalRange {
            original: TextRange {
                start: "alpha".len(),
                end: "alpha\nβ".len(),
            },
            kind: LoadedToOriginalRangeKind::Exact,
        })
    );
}

#[test]
fn loading_map_maps_loaded_zero_after_removed_leading_bom_to_original_byte_three() {
    let source_id = source_id(1);
    let map = LoadingMap::new(
        source_id,
        "alpha",
        open_buffer_origin(),
        vec![
            LoadingMapSegment::RemovedLeadingBom {
                original: TextRange { start: 0, end: 3 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 5 },
                original: TextRange { start: 3, end: 8 },
            },
        ],
    );

    assert_eq!(map.original_offset_for_loaded(source_id, 0), Ok(3));
    assert_eq!(map.original_offset_for_loaded(source_id, 5), Ok(8));
    assert_eq!(
        map.original_range_for_loaded(source_id, TextRange { start: 0, end: 5 }),
        Ok(LoadedToOriginalRange {
            original: TextRange { start: 3, end: 8 },
            kind: LoadedToOriginalRangeKind::Exact,
        })
    );
}

#[test]
fn loading_map_represents_crlf_to_lf_normalized_segments() {
    let source_id = source_id(1);
    let map = crlf_loading_map(source_id);

    assert_eq!(
        map.segments,
        vec![
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 5 },
                original: TextRange { start: 0, end: 5 },
            },
            LoadingMapSegment::NormalizedNewline {
                loaded: TextRange { start: 5, end: 6 },
                original: TextRange { start: 5, end: 7 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 6, end: 10 },
                original: TextRange { start: 7, end: 11 },
            },
        ]
    );
    assert_eq!(map.original_offset_for_loaded(source_id, 5), Ok(5));
    assert_eq!(map.original_offset_for_loaded(source_id, 6), Ok(7));
    assert_eq!(
        map.original_range_for_loaded(source_id, TextRange { start: 5, end: 6 }),
        Ok(LoadedToOriginalRange {
            original: TextRange { start: 5, end: 7 },
            kind: LoadedToOriginalRangeKind::Degraded,
        })
    );
}

#[test]
fn loading_map_degrades_range_mapping_across_normalized_newline_segments() {
    let source_id = source_id(1);
    let map = crlf_loading_map(source_id);

    assert_eq!(
        map.original_range_for_loaded(source_id, TextRange { start: 4, end: 7 }),
        Ok(LoadedToOriginalRange {
            original: TextRange { start: 4, end: 8 },
            kind: LoadedToOriginalRangeKind::Degraded,
        })
    );
}

#[test]
fn loading_map_combines_leading_bom_base_with_crlf_normalized_segments() {
    let source_id = source_id(1);
    let map = LoadingMap::new(
        source_id,
        "alpha\nbeta",
        open_buffer_origin(),
        vec![
            LoadingMapSegment::RemovedLeadingBom {
                original: TextRange { start: 0, end: 3 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 5 },
                original: TextRange { start: 3, end: 8 },
            },
            LoadingMapSegment::NormalizedNewline {
                loaded: TextRange { start: 5, end: 6 },
                original: TextRange { start: 8, end: 10 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 6, end: 10 },
                original: TextRange { start: 10, end: 14 },
            },
        ],
    );

    assert_eq!(map.original_offset_for_loaded(source_id, 0), Ok(3));
    assert_eq!(map.original_offset_for_loaded(source_id, 5), Ok(8));
    assert_eq!(map.original_offset_for_loaded(source_id, 6), Ok(10));
    assert_eq!(
        map.original_range_for_loaded(source_id, TextRange { start: 4, end: 7 }),
        Ok(LoadedToOriginalRange {
            original: TextRange { start: 7, end: 11 },
            kind: LoadedToOriginalRangeKind::Degraded,
        })
    );
}

#[test]
fn loading_map_rejects_source_id_mismatch_and_outside_ranges() {
    let primary_source_id = source_id(1);
    let other_source_id = source_id(2);
    let map = crlf_loading_map(primary_source_id);

    assert_eq!(
        map.original_offset_for_loaded(other_source_id, 0),
        Err(SourceMapError::UnknownSourceId {
            source_id: other_source_id,
        })
    );
    assert_eq!(
        map.original_range_for_loaded(primary_source_id, TextRange { start: 9, end: 12 }),
        Err(SourceMapError::RangeOutsideLoadedText {
            source_id: primary_source_id,
            range: TextRange { start: 9, end: 12 },
            loaded_len: 10,
        })
    );
    assert_eq!(
        map.original_offset_for_loaded(primary_source_id, 11),
        Err(SourceMapError::RangeOutsideLoadedText {
            source_id: primary_source_id,
            range: TextRange { start: 11, end: 11 },
            loaded_len: 10,
        })
    );
    assert_eq!(
        map.original_range_for_loaded(primary_source_id, TextRange { start: 3, end: 2 }),
        Err(SourceMapError::ReversedRange)
    );
}

#[test]
fn loading_map_rejects_ranges_outside_loaded_text_even_when_segments_are_longer() {
    let source_id = source_id(1);
    let map = LoadingMap::new(
        source_id,
        "abc",
        open_buffer_origin(),
        vec![LoadingMapSegment::Original {
            loaded: TextRange { start: 0, end: 10 },
            original: TextRange { start: 0, end: 10 },
        }],
    );

    assert_eq!(map.loaded_len(), 3);
    assert_eq!(
        map.original_offset_for_loaded(source_id, 4),
        Err(SourceMapError::RangeOutsideLoadedText {
            source_id,
            range: TextRange { start: 4, end: 4 },
            loaded_len: 3,
        })
    );
    assert_eq!(
        map.original_range_for_loaded(source_id, TextRange { start: 0, end: 4 }),
        Err(SourceMapError::RangeOutsideLoadedText {
            source_id,
            range: TextRange { start: 0, end: 4 },
            loaded_len: 3,
        })
    );
}

#[test]
fn loading_map_reports_missing_segment_for_gaps_inside_loaded_text() {
    let source_id = source_id(1);
    let map = LoadingMap::new(
        source_id,
        "abcd",
        open_buffer_origin(),
        vec![
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 1 },
                original: TextRange { start: 0, end: 1 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 3, end: 4 },
                original: TextRange { start: 3, end: 4 },
            },
        ],
    );

    assert_eq!(
        map.original_range_for_loaded(source_id, TextRange { start: 0, end: 4 }),
        Err(SourceMapError::MissingLoadingMapSegment {
            source_id,
            range: TextRange { start: 1, end: 1 },
        })
    );
    assert_eq!(
        map.original_offset_for_loaded(source_id, 1),
        Err(SourceMapError::MissingLoadingMapSegment {
            source_id,
            range: TextRange { start: 1, end: 1 },
        })
    );
    assert_eq!(
        map.original_range_for_loaded(source_id, TextRange { start: 1, end: 1 }),
        Err(SourceMapError::MissingLoadingMapSegment {
            source_id,
            range: TextRange { start: 1, end: 1 },
        })
    );
}

#[test]
fn loading_map_accepts_empty_identity_point_but_rejects_non_empty_empty_segments() {
    let empty_source_id = source_id(1);
    let empty = LoadingMap::new(empty_source_id, "", open_buffer_origin(), Vec::new());

    assert_eq!(empty.original_offset_for_loaded(empty_source_id, 0), Ok(0));
    assert_eq!(
        empty.original_range_for_loaded(empty_source_id, TextRange { start: 0, end: 0 }),
        Ok(LoadedToOriginalRange {
            original: TextRange { start: 0, end: 0 },
            kind: LoadedToOriginalRangeKind::Exact,
        })
    );

    let non_empty_source_id = source_id(2);
    let non_empty = LoadingMap::new(non_empty_source_id, "abc", open_buffer_origin(), Vec::new());

    assert_eq!(
        non_empty.original_offset_for_loaded(non_empty_source_id, 0),
        Err(SourceMapError::MissingLoadingMapSegment {
            source_id: non_empty_source_id,
            range: TextRange { start: 0, end: 0 },
        })
    );
}

#[test]
fn loading_map_maps_only_real_eof_through_segment_endpoints() {
    let source_id = source_id(1);
    let map = LoadingMap::new(
        source_id,
        "abcd",
        open_buffer_origin(),
        vec![
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 1 },
                original: TextRange { start: 0, end: 1 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 3, end: 4 },
                original: TextRange { start: 30, end: 31 },
            },
        ],
    );

    assert_eq!(map.original_offset_for_loaded(source_id, 4), Ok(31));
    assert_eq!(
        map.original_range_for_loaded(source_id, TextRange { start: 4, end: 4 }),
        Ok(LoadedToOriginalRange {
            original: TextRange { start: 31, end: 31 },
            kind: LoadedToOriginalRangeKind::Exact,
        })
    );
}

#[test]
fn preprocess_map_identity_maps_original_lexical_range_to_source_range() {
    let source_id = source_id(1);
    let map = PreprocessMap::identity(source_id, "alpha beta");

    assert_eq!(map.source_id(), source_id);
    assert_eq!(map.lexical_len(), "alpha beta".len());
    assert_eq!(
        map.lexical_text_hash(),
        super::hash_source_text("alpha beta")
    );
    assert_eq!(
        map.source_range_for_lexical(source_id, TextRange { start: 0, end: 5 }),
        Ok(LexicalSourceMapping {
            primary: Some(SourceRange {
                source_id,
                start: 0,
                end: 5,
            }),
            anchors: vec![SourceAnchor::Range(SourceRange {
                source_id,
                start: 0,
                end: 5,
            })],
            kind: LexicalSourceMappingKind::Exact,
        })
    );
}

#[test]
fn preprocess_map_returns_removed_comment_anchors_at_lexical_boundaries() {
    let source_id = source_id(1);
    let map = comment_synthetic_preprocess_map(source_id);

    assert_eq!(
        map.source_anchors_for_lexical_offset(source_id, 5),
        Ok(vec![
            SourceAnchor::Point {
                source_id,
                offset: 5,
            },
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 5,
                end: 19,
            }),
        ])
    );
    assert_eq!(
        map.source_anchors_for_lexical_offset(source_id, 6),
        Ok(vec![
            SourceAnchor::Range(SourceRange {
                source_id,
                start: 5,
                end: 19,
            }),
            SourceAnchor::Point {
                source_id,
                offset: 19,
            },
        ])
    );
}

#[test]
fn preprocess_map_represents_ranges_spanning_removed_comments_as_composite_mapping() {
    let source_id = source_id(1);
    let map = comment_no_synthetic_preprocess_map(source_id);

    assert_eq!(
        map.source_range_for_lexical(source_id, TextRange { start: 0, end: 11 }),
        Ok(LexicalSourceMapping {
            primary: Some(SourceRange {
                source_id,
                start: 0,
                end: 25,
            }),
            anchors: vec![
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 6,
                }),
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 6,
                    end: 20,
                }),
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 20,
                    end: 25,
                }),
            ],
            kind: LexicalSourceMappingKind::Composite,
        })
    );
}

#[test]
fn preprocess_map_degrades_ranges_that_include_synthetic_whitespace() {
    let source_id = source_id(1);
    let map = comment_synthetic_preprocess_map(source_id);

    assert_eq!(
        map.source_range_for_lexical(source_id, TextRange { start: 0, end: 10 }),
        Ok(LexicalSourceMapping {
            primary: Some(SourceRange {
                source_id,
                start: 0,
                end: 23,
            }),
            anchors: vec![
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 5,
                }),
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 5,
                    end: 19,
                }),
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 19,
                    end: 23,
                }),
            ],
            kind: LexicalSourceMappingKind::Degraded,
        })
    );
}

#[test]
fn preprocess_map_does_not_promote_synthetic_whitespace_to_primary_user_range() {
    let source_id = source_id(1);
    let map = comment_synthetic_preprocess_map(source_id);

    assert_eq!(
        map.source_range_for_lexical(source_id, TextRange { start: 5, end: 6 }),
        Ok(LexicalSourceMapping {
            primary: None,
            anchors: vec![SourceAnchor::Range(SourceRange {
                source_id,
                start: 5,
                end: 19,
            })],
            kind: LexicalSourceMappingKind::Degraded,
        })
    );

    let synthetic_only = PreprocessMap::new(
        source_id,
        " ",
        vec![PreprocessSegment::SyntheticWhitespace {
            lexical: TextRange { start: 0, end: 1 },
            anchor: SourceAnchor::Range(SourceRange {
                source_id,
                start: 5,
                end: 19,
            }),
        }],
    );

    assert_eq!(
        synthetic_only.source_range_for_lexical(source_id, TextRange { start: 0, end: 0 }),
        Ok(LexicalSourceMapping {
            primary: None,
            anchors: vec![SourceAnchor::Range(SourceRange {
                source_id,
                start: 5,
                end: 19,
            })],
            kind: LexicalSourceMappingKind::Degraded,
        })
    );
}

#[test]
fn preprocess_map_can_return_generated_source_anchors() {
    let source_id = source_id(1);
    let map = PreprocessMap::new(
        source_id,
        " ",
        vec![PreprocessSegment::SyntheticWhitespace {
            lexical: TextRange { start: 0, end: 1 },
            anchor: generated_anchor(source_id, 0, 1, "synthetic whitespace"),
        }],
    );

    assert_eq!(
        map.source_range_for_lexical(source_id, TextRange { start: 0, end: 0 }),
        Ok(LexicalSourceMapping {
            primary: None,
            anchors: vec![generated_anchor(source_id, 0, 1, "synthetic whitespace")],
            kind: LexicalSourceMappingKind::Degraded,
        })
    );
}

#[test]
fn preprocess_map_returns_adjacent_anchors_for_zero_length_boundaries() {
    let source_id = source_id(1);
    let map = PreprocessMap::new(
        source_id,
        "alphabeta",
        vec![
            PreprocessSegment::Original {
                lexical: TextRange { start: 0, end: 5 },
                source: SourceRange {
                    source_id,
                    start: 0,
                    end: 5,
                },
            },
            PreprocessSegment::Original {
                lexical: TextRange { start: 5, end: 9 },
                source: SourceRange {
                    source_id,
                    start: 20,
                    end: 24,
                },
            },
        ],
    );

    assert_eq!(
        map.source_anchors_for_lexical_offset(source_id, 5),
        Ok(vec![
            SourceAnchor::Point {
                source_id,
                offset: 5,
            },
            SourceAnchor::Point {
                source_id,
                offset: 20,
            },
        ])
    );
    assert_eq!(
        map.source_range_for_lexical(source_id, TextRange { start: 5, end: 5 }),
        Ok(LexicalSourceMapping {
            primary: None,
            anchors: vec![
                SourceAnchor::Point {
                    source_id,
                    offset: 5,
                },
                SourceAnchor::Point {
                    source_id,
                    offset: 20,
                },
            ],
            kind: LexicalSourceMappingKind::Composite,
        })
    );
}

#[test]
fn preprocess_map_rejects_source_mismatch_outside_ranges_and_missing_segments() {
    let primary_source_id = source_id(1);
    let other_source_id = source_id(2);
    let map = comment_synthetic_preprocess_map(primary_source_id);

    assert_eq!(
        map.source_range_for_lexical(other_source_id, TextRange { start: 0, end: 1 }),
        Err(SourceMapError::UnknownSourceId {
            source_id: other_source_id,
        })
    );
    assert_eq!(
        map.source_range_for_lexical(primary_source_id, TextRange { start: 0, end: 11 }),
        Err(SourceMapError::RangeOutsideLexicalText {
            source_id: primary_source_id,
            range: TextRange { start: 0, end: 11 },
            lexical_len: 10,
        })
    );
    assert_eq!(
        map.source_anchors_for_lexical_offset(primary_source_id, 11),
        Err(SourceMapError::RangeOutsideLexicalText {
            source_id: primary_source_id,
            range: TextRange { start: 11, end: 11 },
            lexical_len: 10,
        })
    );
    assert_eq!(
        map.source_range_for_lexical(primary_source_id, TextRange { start: 4, end: 3 }),
        Err(SourceMapError::ReversedRange)
    );

    let gap = PreprocessMap::new(
        primary_source_id,
        "abcd",
        vec![
            PreprocessSegment::Original {
                lexical: TextRange { start: 0, end: 1 },
                source: SourceRange {
                    source_id: primary_source_id,
                    start: 0,
                    end: 1,
                },
            },
            PreprocessSegment::Original {
                lexical: TextRange { start: 3, end: 4 },
                source: SourceRange {
                    source_id: primary_source_id,
                    start: 3,
                    end: 4,
                },
            },
        ],
    );

    assert_eq!(
        gap.source_range_for_lexical(primary_source_id, TextRange { start: 0, end: 4 }),
        Err(SourceMapError::MissingPreprocessSegment {
            source_id: primary_source_id,
            range: TextRange { start: 1, end: 1 },
        })
    );
    assert_eq!(
        gap.source_anchors_for_lexical_offset(primary_source_id, 2),
        Err(SourceMapError::MissingPreprocessSegment {
            source_id: primary_source_id,
            range: TextRange { start: 2, end: 2 },
        })
    );
}

#[test]
fn preprocess_map_maps_subranges_inside_non_identity_original_segments() {
    let source_id = source_id(1);
    let map = PreprocessMap::new(
        source_id,
        "abcdef",
        vec![PreprocessSegment::Original {
            lexical: TextRange { start: 2, end: 6 },
            source: SourceRange {
                source_id,
                start: 20,
                end: 24,
            },
        }],
    );

    assert_eq!(
        map.source_range_for_lexical(source_id, TextRange { start: 3, end: 5 }),
        Ok(LexicalSourceMapping {
            primary: Some(SourceRange {
                source_id,
                start: 21,
                end: 23,
            }),
            anchors: vec![SourceAnchor::Range(SourceRange {
                source_id,
                start: 21,
                end: 23,
            })],
            kind: LexicalSourceMappingKind::Exact,
        })
    );
    assert_eq!(
        map.source_anchors_for_lexical_offset(source_id, 4),
        Ok(vec![SourceAnchor::Point {
            source_id,
            offset: 22,
        }])
    );
}

#[test]
fn preprocess_map_rejects_mismatched_source_ids_inside_segments() {
    let primary_source_id = source_id(1);
    let other_source_id = source_id(2);
    let map = PreprocessMap::new(
        primary_source_id,
        "abc",
        vec![PreprocessSegment::Original {
            lexical: TextRange { start: 0, end: 3 },
            source: SourceRange {
                source_id: other_source_id,
                start: 0,
                end: 3,
            },
        }],
    );

    assert_eq!(
        map.source_range_for_lexical(primary_source_id, TextRange { start: 0, end: 1 }),
        Err(SourceMapError::UnknownSourceId {
            source_id: other_source_id,
        })
    );

    let removed_comment = PreprocessMap::new(
        primary_source_id,
        "ab",
        vec![
            PreprocessSegment::Original {
                lexical: TextRange { start: 0, end: 1 },
                source: SourceRange {
                    source_id: primary_source_id,
                    start: 0,
                    end: 1,
                },
            },
            PreprocessSegment::RemovedComment {
                source: SourceRange {
                    source_id: other_source_id,
                    start: 1,
                    end: 5,
                },
                kind: CommentKind::SingleLine,
            },
            PreprocessSegment::Original {
                lexical: TextRange { start: 1, end: 2 },
                source: SourceRange {
                    source_id: primary_source_id,
                    start: 5,
                    end: 6,
                },
            },
        ],
    );

    assert_eq!(
        removed_comment.source_anchors_for_lexical_offset(primary_source_id, 1),
        Err(SourceMapError::UnknownSourceId {
            source_id: other_source_id,
        })
    );
    assert_eq!(
        removed_comment
            .source_range_for_lexical(primary_source_id, TextRange { start: 0, end: 2 },),
        Err(SourceMapError::UnknownSourceId {
            source_id: other_source_id,
        })
    );

    let synthetic_range_anchor = PreprocessMap::new(
        primary_source_id,
        " ",
        vec![PreprocessSegment::SyntheticWhitespace {
            lexical: TextRange { start: 0, end: 1 },
            anchor: SourceAnchor::Range(SourceRange {
                source_id: other_source_id,
                start: 1,
                end: 5,
            }),
        }],
    );

    assert_eq!(
        synthetic_range_anchor
            .source_range_for_lexical(primary_source_id, TextRange { start: 0, end: 1 },),
        Err(SourceMapError::UnknownSourceId {
            source_id: other_source_id,
        })
    );

    let synthetic_point_anchor = PreprocessMap::new(
        primary_source_id,
        " ",
        vec![PreprocessSegment::SyntheticWhitespace {
            lexical: TextRange { start: 0, end: 1 },
            anchor: SourceAnchor::Point {
                source_id: other_source_id,
                offset: 1,
            },
        }],
    );

    assert_eq!(
        synthetic_point_anchor.source_anchors_for_lexical_offset(primary_source_id, 0),
        Err(SourceMapError::UnknownSourceId {
            source_id: other_source_id,
        })
    );
}

#[test]
fn preprocess_map_degrades_non_empty_generated_anchors_without_primary_range() {
    let source_id = source_id(1);
    let map = PreprocessMap::new(
        source_id,
        " ",
        vec![PreprocessSegment::SyntheticWhitespace {
            lexical: TextRange { start: 0, end: 1 },
            anchor: generated_anchor(source_id, 0, 1, "synthetic whitespace"),
        }],
    );

    assert_eq!(
        map.source_range_for_lexical(source_id, TextRange { start: 0, end: 1 }),
        Ok(LexicalSourceMapping {
            primary: None,
            anchors: vec![generated_anchor(source_id, 0, 1, "synthetic whitespace")],
            kind: LexicalSourceMappingKind::Degraded,
        })
    );
}

#[test]
fn preprocess_map_handles_empty_maps_like_loading_map_empty_identity() {
    let empty_source_id = source_id(1);
    let empty = PreprocessMap::new(empty_source_id, "", Vec::new());

    assert_eq!(
        empty.source_anchors_for_lexical_offset(empty_source_id, 0),
        Ok(vec![SourceAnchor::Point {
            source_id: empty_source_id,
            offset: 0,
        }])
    );
    assert_eq!(
        empty.source_range_for_lexical(empty_source_id, TextRange { start: 0, end: 0 }),
        Ok(LexicalSourceMapping {
            primary: Some(SourceRange {
                source_id: empty_source_id,
                start: 0,
                end: 0,
            }),
            anchors: vec![SourceAnchor::Point {
                source_id: empty_source_id,
                offset: 0,
            }],
            kind: LexicalSourceMappingKind::Exact,
        })
    );

    let non_empty_source_id = source_id(2);
    let non_empty = PreprocessMap::new(non_empty_source_id, "abc", Vec::new());

    assert_eq!(
        non_empty.source_range_for_lexical(non_empty_source_id, TextRange { start: 0, end: 1 },),
        Err(SourceMapError::MissingPreprocessSegment {
            source_id: non_empty_source_id,
            range: TextRange { start: 0, end: 0 },
        })
    );
    assert_eq!(
        non_empty.source_anchors_for_lexical_offset(non_empty_source_id, 0),
        Err(SourceMapError::MissingPreprocessSegment {
            source_id: non_empty_source_id,
            range: TextRange { start: 0, end: 0 },
        })
    );
}

#[test]
fn source_map_service_line_column_converts_ranges_through_line_map() {
    let source_id = source_id(1);
    let service =
        RetainedSourceMapService::new().with_line_map(LineMap::with_source(source_id, "aβ\nz"));

    assert_eq!(
        service.line_column(SourceRange {
            source_id,
            start: "a".len(),
            end: "aβ\n".len(),
        }),
        Ok((
            LineColumn { line: 1, column: 2 },
            LineColumn { line: 2, column: 1 },
        ))
    );
}

#[test]
fn source_map_service_maps_loaded_exact_and_degraded_ranges() {
    let source_id = source_id(1);
    let service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(source_id, "alpha\nbeta"))
        .with_loading_map(crlf_loading_map(source_id));

    assert_eq!(
        service.original_range_for_loaded(source_id, TextRange { start: 0, end: 5 }),
        Ok(MappedSourceRange {
            primary: SourceRange {
                source_id,
                start: 0,
                end: 5,
            },
            secondary: Vec::new(),
            original_input: Some(TextRange { start: 0, end: 5 }),
            kind: MappedSourceRangeKind::Exact,
        })
    );
    assert_eq!(
        service.original_range_for_loaded(source_id, TextRange { start: 4, end: 7 }),
        Ok(MappedSourceRange {
            primary: SourceRange {
                source_id,
                start: 4,
                end: 7,
            },
            secondary: Vec::new(),
            original_input: Some(TextRange { start: 4, end: 8 }),
            kind: MappedSourceRangeKind::Degraded,
        })
    );
}

#[test]
fn source_map_service_keeps_loaded_primary_range_separate_from_original_input_bytes() {
    let source_id = source_id(1);
    let service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(source_id, "alpha"))
        .with_loading_map(LoadingMap::new(
            source_id,
            "alpha",
            open_buffer_origin(),
            vec![
                LoadingMapSegment::RemovedLeadingBom {
                    original: TextRange { start: 0, end: 3 },
                },
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 0, end: 5 },
                    original: TextRange { start: 3, end: 8 },
                },
            ],
        ));

    let mapped = service
        .original_range_for_loaded(source_id, TextRange { start: 0, end: 5 })
        .unwrap();

    assert_eq!(
        mapped,
        MappedSourceRange {
            primary: SourceRange {
                source_id,
                start: 0,
                end: 5,
            },
            secondary: Vec::new(),
            original_input: Some(TextRange { start: 3, end: 8 }),
            kind: MappedSourceRangeKind::Exact,
        }
    );
    assert_eq!(service.validate_range(mapped.primary), Ok(()));
}

#[test]
fn source_map_service_maps_lexical_original_composite_and_synthetic_ranges() {
    let source_id = source_id(1);
    let source_text = "abcdefghijklmnopqrstuvwxyz";
    let exact_service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(source_id, source_text))
        .with_preprocess_map(PreprocessMap::identity(source_id, "alpha beta"));
    let composite_service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(source_id, source_text))
        .with_preprocess_map(comment_no_synthetic_preprocess_map(source_id));
    let synthetic_service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(source_id, source_text))
        .with_preprocess_map(comment_synthetic_preprocess_map(source_id));

    assert_eq!(
        exact_service.source_range_for_lexical(source_id, TextRange { start: 0, end: 5 }),
        Ok(MappedSourceRange {
            primary: SourceRange {
                source_id,
                start: 0,
                end: 5,
            },
            secondary: Vec::new(),
            original_input: None,
            kind: MappedSourceRangeKind::Exact,
        })
    );
    assert_eq!(
        composite_service.source_range_for_lexical(source_id, TextRange { start: 0, end: 11 }),
        Ok(MappedSourceRange {
            primary: SourceRange {
                source_id,
                start: 0,
                end: 25,
            },
            secondary: vec![
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 0,
                    end: 6,
                }),
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 6,
                    end: 20,
                }),
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 20,
                    end: 25,
                }),
            ],
            original_input: None,
            kind: MappedSourceRangeKind::Composite,
        })
    );
    assert_eq!(
        synthetic_service.source_range_for_lexical(source_id, TextRange { start: 5, end: 6 }),
        Ok(MappedSourceRange {
            primary: SourceRange {
                source_id,
                start: 5,
                end: 19,
            },
            secondary: Vec::new(),
            original_input: None,
            kind: MappedSourceRangeKind::Degraded,
        })
    );
}

#[test]
fn source_map_service_preserves_generated_origin_reason_as_secondary_anchor() {
    let source_id = source_id(1);
    let origin = GeneratedSpanOrigin::new(
        GeneratedSpanAnchor::Range(SourceRange {
            source_id,
            start: 1,
            end: 2,
        }),
        "synthetic recovery whitespace",
    )
    .unwrap();
    let service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(source_id, "abc"))
        .with_preprocess_map(PreprocessMap::new(
            source_id,
            " ",
            vec![PreprocessSegment::SyntheticWhitespace {
                lexical: TextRange { start: 0, end: 1 },
                anchor: SourceAnchor::Generated(origin.clone()),
            }],
        ));

    assert_eq!(
        service.source_range_for_lexical(source_id, TextRange { start: 0, end: 1 }),
        Ok(MappedSourceRange {
            primary: SourceRange {
                source_id,
                start: 1,
                end: 2,
            },
            secondary: vec![SourceAnchor::Generated(origin)],
            original_input: None,
            kind: MappedSourceRangeKind::Degraded,
        })
    );
}

#[test]
fn source_map_service_maps_generated_point_origin_to_zero_length_primary_range() {
    let source_id = source_id(1);
    let origin = GeneratedSpanOrigin::new(
        GeneratedSpanAnchor::Point {
            source_id,
            offset: 2,
        },
        "implicit insertion point",
    )
    .unwrap();
    let service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(source_id, "abc"))
        .with_preprocess_map(PreprocessMap::new(
            source_id,
            " ",
            vec![PreprocessSegment::SyntheticWhitespace {
                lexical: TextRange { start: 0, end: 1 },
                anchor: SourceAnchor::Generated(origin.clone()),
            }],
        ));

    assert_eq!(
        service.source_range_for_lexical(source_id, TextRange { start: 0, end: 1 }),
        Ok(MappedSourceRange {
            primary: SourceRange {
                source_id,
                start: 2,
                end: 2,
            },
            secondary: vec![SourceAnchor::Generated(origin)],
            original_input: None,
            kind: MappedSourceRangeKind::Degraded,
        })
    );
}

#[test]
fn source_map_service_attach_generated_span_preserves_origin_reason() {
    let source_id = source_id(1);
    let service =
        RetainedSourceMapService::new().with_line_map(LineMap::with_source(source_id, "abc"));
    let origin = GeneratedSpanOrigin::new(
        GeneratedSpanAnchor::Range(SourceRange {
            source_id,
            start: 1,
            end: 2,
        }),
        "implicit obligation",
    )
    .unwrap();

    assert_eq!(
        service.attach_generated_span(origin.clone()),
        Ok(SourceAnchor::Generated(origin))
    );

    let point_origin = GeneratedSpanOrigin::new(
        GeneratedSpanAnchor::Point {
            source_id,
            offset: 0,
        },
        "implicit insertion point",
    )
    .unwrap();
    assert_eq!(
        service.attach_generated_span(point_origin.clone()),
        Ok(SourceAnchor::Generated(point_origin))
    );
}

#[test]
fn generated_span_origin_requires_reason() {
    let source_id = source_id(1);

    assert_eq!(
        GeneratedSpanOrigin::new(
            GeneratedSpanAnchor::Point {
                source_id,
                offset: 0,
            },
            "",
        ),
        Err(SourceMapError::GeneratedSpanWithoutOriginReason)
    );
    assert_eq!(
        GeneratedSpanOrigin::new(
            GeneratedSpanAnchor::Range(SourceRange {
                source_id,
                start: 0,
                end: 0,
            }),
            " \t\n",
        ),
        Err(SourceMapError::GeneratedSpanWithoutOriginReason)
    );
}

#[test]
fn source_map_service_attach_generated_span_rejects_invalid_origin() {
    let primary_source_id = source_id(1);
    let unknown_source_id = source_id(2);
    let service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(primary_source_id, "abc"));
    let unknown_origin = GeneratedSpanOrigin::new(
        GeneratedSpanAnchor::Range(SourceRange {
            source_id: unknown_source_id,
            start: 0,
            end: 0,
        }),
        "implicit obligation",
    )
    .unwrap();
    let malformed_reason = GeneratedSpanOrigin {
        anchor: GeneratedSpanAnchor::Point {
            source_id: primary_source_id,
            offset: 0,
        },
        reason: " ".to_owned(),
    };

    assert_eq!(
        service.attach_generated_span(unknown_origin),
        Err(SourceMapError::UnknownSourceId {
            source_id: unknown_source_id,
        })
    );
    assert_eq!(
        service.attach_generated_span(malformed_reason),
        Err(SourceMapError::GeneratedSpanWithoutOriginReason)
    );
}

#[test]
fn source_map_service_reports_unknown_missing_maps_and_invalid_ranges() {
    let primary_source_id = source_id(1);
    let unknown_source_id = source_id(2);
    let service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(primary_source_id, "aβ"));

    assert_eq!(
        service.validate_range(SourceRange {
            source_id: unknown_source_id,
            start: 0,
            end: 0,
        }),
        Err(SourceMapError::UnknownSourceId {
            source_id: unknown_source_id,
        })
    );
    assert_eq!(
        service.original_range_for_loaded(primary_source_id, TextRange { start: 0, end: 1 }),
        Err(SourceMapError::MissingLoadingMapSegment {
            source_id: primary_source_id,
            range: TextRange { start: 0, end: 1 },
        })
    );
    assert_eq!(
        service.source_range_for_lexical(primary_source_id, TextRange { start: 0, end: 1 }),
        Err(SourceMapError::MissingPreprocessSegment {
            source_id: primary_source_id,
            range: TextRange { start: 0, end: 1 },
        })
    );
    assert_eq!(
        service.validate_range(SourceRange {
            source_id: primary_source_id,
            start: 2,
            end: 2,
        }),
        Err(SourceMapError::OffsetNotUtf8Boundary {
            source_id: primary_source_id,
            offset: 2,
        })
    );
}

#[test]
fn source_map_service_forwards_line_column_errors() {
    let primary_source_id = source_id(1);
    let unknown_source_id = source_id(2);
    let service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(primary_source_id, "abc"));

    assert_eq!(
        service.line_column(SourceRange {
            source_id: unknown_source_id,
            start: 0,
            end: 0,
        }),
        Err(SourceMapError::UnknownSourceId {
            source_id: unknown_source_id,
        })
    );
    assert_eq!(
        service.line_column(SourceRange {
            source_id: primary_source_id,
            start: 2,
            end: 4,
        }),
        Err(SourceMapError::RangeOutsideSourceText {
            range: SourceRange {
                source_id: primary_source_id,
                start: 2,
                end: 4,
            },
            source_len: 3,
        })
    );
}

#[test]
fn source_map_service_forwards_loading_map_errors() {
    let source_id = source_id(1);
    let service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(source_id, "alpha\nbeta"))
        .with_loading_map(crlf_loading_map(source_id));
    let gap_service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(source_id, "abcd"))
        .with_loading_map(LoadingMap::new(
            source_id,
            "abcd",
            open_buffer_origin(),
            vec![
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 0, end: 1 },
                    original: TextRange { start: 0, end: 1 },
                },
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 3, end: 4 },
                    original: TextRange { start: 3, end: 4 },
                },
            ],
        ));

    assert_eq!(
        service.original_range_for_loaded(source_id, TextRange { start: 3, end: 2 }),
        Err(SourceMapError::ReversedRange)
    );
    assert_eq!(
        service.original_range_for_loaded(source_id, TextRange { start: 9, end: 12 }),
        Err(SourceMapError::RangeOutsideLoadedText {
            source_id,
            range: TextRange { start: 9, end: 12 },
            loaded_len: 10,
        })
    );
    assert_eq!(
        gap_service.original_range_for_loaded(source_id, TextRange { start: 0, end: 4 }),
        Err(SourceMapError::MissingLoadingMapSegment {
            source_id,
            range: TextRange { start: 1, end: 1 },
        })
    );
}

#[test]
fn source_map_service_forwards_preprocess_map_errors() {
    let source_id = source_id(1);
    let service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(
            source_id,
            "abcdefghijklmnopqrstuvwxyz",
        ))
        .with_preprocess_map(comment_synthetic_preprocess_map(source_id));
    let gap_service = RetainedSourceMapService::new()
        .with_line_map(LineMap::with_source(source_id, "abcd"))
        .with_preprocess_map(PreprocessMap::new(
            source_id,
            "abcd",
            vec![
                PreprocessSegment::Original {
                    lexical: TextRange { start: 0, end: 1 },
                    source: SourceRange {
                        source_id,
                        start: 0,
                        end: 1,
                    },
                },
                PreprocessSegment::Original {
                    lexical: TextRange { start: 3, end: 4 },
                    source: SourceRange {
                        source_id,
                        start: 3,
                        end: 4,
                    },
                },
            ],
        ));

    assert_eq!(
        service.source_range_for_lexical(source_id, TextRange { start: 4, end: 3 }),
        Err(SourceMapError::ReversedRange)
    );
    assert_eq!(
        service.source_range_for_lexical(source_id, TextRange { start: 0, end: 11 }),
        Err(SourceMapError::RangeOutsideLexicalText {
            source_id,
            range: TextRange { start: 0, end: 11 },
            lexical_len: 10,
        })
    );
    assert_eq!(
        gap_service.source_range_for_lexical(source_id, TextRange { start: 0, end: 4 }),
        Err(SourceMapError::MissingPreprocessSegment {
            source_id,
            range: TextRange { start: 1, end: 1 },
        })
    );
}

fn line_map(source: &str) -> LineMap {
    LineMap::with_source(source_id(1), source)
}

fn generated_anchor(source_id: SourceId, start: usize, end: usize, reason: &str) -> SourceAnchor {
    SourceAnchor::Generated(
        GeneratedSpanOrigin::new(
            GeneratedSpanAnchor::Range(SourceRange {
                source_id,
                start,
                end,
            }),
            reason,
        )
        .expect("test generated spans have reasons"),
    )
}

fn crlf_loading_map(source_id: SourceId) -> LoadingMap {
    LoadingMap::new(
        source_id,
        "alpha\nbeta",
        open_buffer_origin(),
        vec![
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 5 },
                original: TextRange { start: 0, end: 5 },
            },
            LoadingMapSegment::NormalizedNewline {
                loaded: TextRange { start: 5, end: 6 },
                original: TextRange { start: 5, end: 7 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange { start: 6, end: 10 },
                original: TextRange { start: 7, end: 11 },
            },
        ],
    )
}

fn open_buffer_origin() -> LoadingOrigin {
    LoadingOrigin::OpenBufferText {
        uri: "file:///pkg/src/test.miz".to_owned(),
        version: 1,
    }
}

fn comment_synthetic_preprocess_map(source_id: SourceId) -> PreprocessMap {
    PreprocessMap::new(
        source_id,
        "alpha beta",
        vec![
            PreprocessSegment::Original {
                lexical: TextRange { start: 0, end: 5 },
                source: SourceRange {
                    source_id,
                    start: 0,
                    end: 5,
                },
            },
            PreprocessSegment::RemovedComment {
                source: SourceRange {
                    source_id,
                    start: 5,
                    end: 19,
                },
                kind: CommentKind::MultiLine,
            },
            PreprocessSegment::SyntheticWhitespace {
                lexical: TextRange { start: 5, end: 6 },
                anchor: SourceAnchor::Range(SourceRange {
                    source_id,
                    start: 5,
                    end: 19,
                }),
            },
            PreprocessSegment::Original {
                lexical: TextRange { start: 6, end: 10 },
                source: SourceRange {
                    source_id,
                    start: 19,
                    end: 23,
                },
            },
        ],
    )
}

fn comment_no_synthetic_preprocess_map(source_id: SourceId) -> PreprocessMap {
    PreprocessMap::new(
        source_id,
        "alpha  beta",
        vec![
            PreprocessSegment::Original {
                lexical: TextRange { start: 0, end: 6 },
                source: SourceRange {
                    source_id,
                    start: 0,
                    end: 6,
                },
            },
            PreprocessSegment::RemovedComment {
                source: SourceRange {
                    source_id,
                    start: 6,
                    end: 20,
                },
                kind: CommentKind::MultiLine,
            },
            PreprocessSegment::Original {
                lexical: TextRange { start: 6, end: 11 },
                source: SourceRange {
                    source_id,
                    start: 20,
                    end: 25,
                },
            },
        ],
    )
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
