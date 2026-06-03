use crate::{Hash, NormalizedPath, SourceId};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub source_id: SourceId,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
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

pub type DocumentUri = String;
pub type LspDocumentVersion = i64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadingMap {
    pub source_id: SourceId,
    pub loaded_text_hash: Hash,
    pub loaded_text_len: usize,
    pub origin: LoadingOrigin,
    pub segments: Vec<LoadingMapSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessMap {
    pub source_id: SourceId,
    pub lexical_text_hash: Hash,
    pub lexical_text_len: usize,
    pub segments: Vec<PreprocessSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoadingOrigin {
    DiskBytes {
        normalized_path: NormalizedPath,
    },
    OpenBufferText {
        uri: DocumentUri,
        version: LspDocumentVersion,
    },
    Generated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoadingMapSegment {
    Original {
        loaded: TextRange,
        original: TextRange,
    },
    RemovedLeadingBom {
        original: TextRange,
    },
    NormalizedNewline {
        loaded: TextRange,
        original: TextRange,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PreprocessSegment {
    Original {
        lexical: TextRange,
        source: SourceRange,
    },
    RemovedComment {
        source: SourceRange,
        kind: CommentKind,
    },
    SyntheticWhitespace {
        lexical: TextRange,
        anchor: SourceAnchor,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommentKind {
    SingleLine,
    MultiLine,
    Documentation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceAnchor {
    Range(SourceRange),
    Point { source_id: SourceId, offset: usize },
    Generated(GeneratedSpanOrigin),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedSpanOrigin {
    anchor: GeneratedSpanAnchor,
    reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GeneratedSpanAnchor {
    Range(SourceRange),
    Point { source_id: SourceId, offset: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadedToOriginalRange {
    pub original: TextRange,
    pub kind: LoadedToOriginalRangeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadedToOriginalRangeKind {
    Exact,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalSourceMapping {
    pub primary: Option<SourceRange>,
    pub anchors: Vec<SourceAnchor>,
    pub kind: LexicalSourceMappingKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexicalSourceMappingKind {
    Exact,
    Composite,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedSourceRange {
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
    pub original_input: Option<TextRange>,
    pub kind: MappedSourceRangeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedSourceRangeKind {
    Exact,
    Composite,
    Degraded,
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
    RangeOutsideLoadedText {
        source_id: SourceId,
        range: TextRange,
        loaded_len: usize,
    },
    MissingLoadingMapSegment {
        source_id: SourceId,
        range: TextRange,
    },
    RangeOutsideLexicalText {
        source_id: SourceId,
        range: TextRange,
        lexical_len: usize,
    },
    MissingPreprocessSegment {
        source_id: SourceId,
        range: TextRange,
    },
    GeneratedSpanWithoutOriginReason,
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
                    range: source_point(self.source_id, offset),
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
                range: source_point(source_id, offset),
                source_len: self.text.len(),
            });
        }
        if self.char_boundaries.binary_search(&offset).is_err() {
            return Err(SourceMapError::OffsetNotUtf8Boundary { source_id, offset });
        }
        Ok(())
    }
}

impl TextRange {
    pub const fn new(start: usize, end: usize) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    pub const fn try_new(start: usize, end: usize) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    pub const fn len(self) -> usize {
        assert!(self.start <= self.end);
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

impl GeneratedSpanOrigin {
    pub fn new(
        anchor: GeneratedSpanAnchor,
        reason: impl Into<String>,
    ) -> Result<Self, SourceMapError> {
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(SourceMapError::GeneratedSpanWithoutOriginReason);
        }
        Ok(Self { anchor, reason })
    }

    pub fn anchor(&self) -> GeneratedSpanAnchor {
        self.anchor
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl GeneratedSpanAnchor {
    fn source_id(self) -> SourceId {
        match self {
            Self::Range(range) => range.source_id,
            Self::Point { source_id, .. } => source_id,
        }
    }

    fn to_source_range(self) -> SourceRange {
        match self {
            Self::Range(range) => range,
            Self::Point { source_id, offset } => SourceRange {
                source_id,
                start: offset,
                end: offset,
            },
        }
    }
}

impl LoadingMap {
    pub fn new(
        source_id: SourceId,
        loaded_text: &str,
        origin: LoadingOrigin,
        segments: Vec<LoadingMapSegment>,
    ) -> Self {
        Self {
            source_id,
            loaded_text_hash: hash_source_text(loaded_text),
            loaded_text_len: loaded_text.len(),
            origin,
            segments,
        }
    }

    pub fn identity(source_id: SourceId, loaded_text: &str, origin: LoadingOrigin) -> Self {
        Self::new(
            source_id,
            loaded_text,
            origin,
            vec![LoadingMapSegment::Original {
                loaded: TextRange {
                    start: 0,
                    end: loaded_text.len(),
                },
                original: TextRange {
                    start: 0,
                    end: loaded_text.len(),
                },
            }],
        )
    }

    pub fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub fn loaded_text_hash(&self) -> Hash {
        self.loaded_text_hash
    }

    pub fn loaded_len(&self) -> usize {
        self.loaded_text_len
    }

    pub fn original_offset_for_loaded(
        &self,
        source_id: SourceId,
        offset: usize,
    ) -> Result<usize, SourceMapError> {
        self.validate_source_id(source_id)?;
        let loaded_len = self.loaded_len();
        if offset > loaded_len {
            return Err(SourceMapError::RangeOutsideLoadedText {
                source_id,
                range: text_point(offset),
                loaded_len,
            });
        }
        if self.segments.is_empty() && loaded_len == 0 && offset == 0 {
            return Ok(0);
        }

        self.original_offset_for_loaded_unchecked(offset).ok_or(
            SourceMapError::MissingLoadingMapSegment {
                source_id,
                range: text_point(offset),
            },
        )
    }

    pub fn original_range_for_loaded(
        &self,
        source_id: SourceId,
        loaded: TextRange,
    ) -> Result<LoadedToOriginalRange, SourceMapError> {
        self.validate_source_id(source_id)?;
        if loaded.start > loaded.end {
            return Err(SourceMapError::ReversedRange);
        }
        let loaded_len = self.loaded_len();
        if loaded.end > loaded_len {
            return Err(SourceMapError::RangeOutsideLoadedText {
                source_id,
                range: loaded,
                loaded_len,
            });
        }
        if loaded.is_empty() {
            let offset = self.original_offset_for_loaded(source_id, loaded.start)?;
            return Ok(LoadedToOriginalRange {
                original: TextRange {
                    start: offset,
                    end: offset,
                },
                kind: LoadedToOriginalRangeKind::Exact,
            });
        }

        let mut cursor = loaded.start;
        let mut original_start = None;
        let mut original_end = None;
        let mut kind = LoadedToOriginalRangeKind::Exact;

        while cursor < loaded.end {
            let Some((segment_loaded, segment_original, segment_kind)) =
                self.segment_covering_loaded_offset(cursor)
            else {
                return Err(SourceMapError::MissingLoadingMapSegment {
                    source_id,
                    range: text_point(cursor),
                });
            };

            let covered_end = segment_loaded.end.min(loaded.end);
            let mapped = map_loaded_subrange_to_original(
                segment_loaded,
                segment_original,
                TextRange {
                    start: cursor,
                    end: covered_end,
                },
                segment_kind,
            );
            original_start.get_or_insert(mapped.start);
            original_end = Some(mapped.end);
            if segment_kind == LoadedToOriginalRangeKind::Degraded {
                kind = LoadedToOriginalRangeKind::Degraded;
            }
            cursor = covered_end;
        }

        Ok(LoadedToOriginalRange {
            original: TextRange {
                start: original_start.expect("non-empty range covers at least one segment"),
                end: original_end.expect("non-empty range covers at least one segment"),
            },
            kind,
        })
    }

    fn validate_source_id(&self, source_id: SourceId) -> Result<(), SourceMapError> {
        if source_id != self.source_id {
            return Err(SourceMapError::UnknownSourceId { source_id });
        }
        Ok(())
    }

    fn segment_covering_loaded_offset(
        &self,
        offset: usize,
    ) -> Option<(TextRange, TextRange, LoadedToOriginalRangeKind)> {
        self.segments.iter().find_map(|segment| match segment {
            LoadingMapSegment::Original { loaded, original }
                if loaded.start <= offset && offset < loaded.end =>
            {
                Some((*loaded, *original, LoadedToOriginalRangeKind::Exact))
            }
            LoadingMapSegment::NormalizedNewline { loaded, original }
                if loaded.start <= offset && offset < loaded.end =>
            {
                Some((*loaded, *original, LoadedToOriginalRangeKind::Degraded))
            }
            _ => None,
        })
    }

    fn original_offset_for_loaded_unchecked(&self, offset: usize) -> Option<usize> {
        self.segments
            .iter()
            .find_map(|segment| segment.original_offset_for_loaded_inside(offset))
            .or_else(|| {
                if offset == self.loaded_text_len {
                    self.segments
                        .iter()
                        .find_map(|segment| segment.original_offset_for_loaded_end(offset))
                } else {
                    None
                }
            })
    }
}

impl LoadingMapSegment {
    fn original_offset_for_loaded_inside(&self, offset: usize) -> Option<usize> {
        match self {
            Self::Original { loaded, original }
                if loaded.start <= offset && offset < loaded.end =>
            {
                Some(original.start + (offset - loaded.start))
            }
            Self::NormalizedNewline { loaded, original }
                if loaded.start <= offset && offset < loaded.end =>
            {
                Some(original.start)
            }
            _ => None,
        }
    }

    fn original_offset_for_loaded_end(&self, offset: usize) -> Option<usize> {
        match self {
            Self::Original { loaded, original } if offset == loaded.end => Some(original.end),
            Self::NormalizedNewline { loaded, original } if offset == loaded.end => {
                Some(original.end)
            }
            _ => None,
        }
    }
}

impl PreprocessMap {
    pub fn new(source_id: SourceId, lexical_text: &str, segments: Vec<PreprocessSegment>) -> Self {
        Self {
            source_id,
            lexical_text_hash: hash_source_text(lexical_text),
            lexical_text_len: lexical_text.len(),
            segments,
        }
    }

    pub fn identity(source_id: SourceId, lexical_text: &str) -> Self {
        Self::new(
            source_id,
            lexical_text,
            vec![PreprocessSegment::Original {
                lexical: TextRange {
                    start: 0,
                    end: lexical_text.len(),
                },
                source: SourceRange {
                    source_id,
                    start: 0,
                    end: lexical_text.len(),
                },
            }],
        )
    }

    pub fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub fn lexical_text_hash(&self) -> Hash {
        self.lexical_text_hash
    }

    pub fn lexical_len(&self) -> usize {
        self.lexical_text_len
    }

    pub fn source_anchors_for_lexical_offset(
        &self,
        source_id: SourceId,
        offset: usize,
    ) -> Result<Vec<SourceAnchor>, SourceMapError> {
        self.validate_source_id(source_id)?;
        let lexical_len = self.lexical_len();
        if offset > lexical_len {
            return Err(SourceMapError::RangeOutsideLexicalText {
                source_id,
                range: text_point(offset),
                lexical_len,
            });
        }
        if self.segments.is_empty() && lexical_len == 0 && offset == 0 {
            return Ok(vec![SourceAnchor::Point {
                source_id,
                offset: 0,
            }]);
        }

        let mut anchors = Vec::new();
        for (index, segment) in self.segments.iter().enumerate() {
            match segment {
                PreprocessSegment::Original { lexical, source }
                    if lexical.start <= offset && offset <= lexical.end =>
                {
                    self.validate_source_range(*source)?;
                    let source_offset = source.start + (offset - lexical.start);
                    push_source_anchor(
                        &mut anchors,
                        SourceAnchor::Point {
                            source_id,
                            offset: source_offset,
                        },
                    );
                }
                PreprocessSegment::RemovedComment { source, .. }
                    if self.lexical_anchor_for_removed_comment(index) == Some(offset) =>
                {
                    self.validate_source_range(*source)?;
                    push_source_anchor(&mut anchors, SourceAnchor::Range(*source));
                }
                PreprocessSegment::SyntheticWhitespace { lexical, anchor }
                    if lexical.start <= offset && offset <= lexical.end =>
                {
                    self.validate_anchor(anchor)?;
                    push_source_anchor(&mut anchors, anchor.clone());
                }
                _ => {}
            }
        }

        if anchors.is_empty() {
            return Err(SourceMapError::MissingPreprocessSegment {
                source_id,
                range: text_point(offset),
            });
        }

        Ok(anchors)
    }

    pub fn source_range_for_lexical(
        &self,
        source_id: SourceId,
        lexical: TextRange,
    ) -> Result<LexicalSourceMapping, SourceMapError> {
        self.validate_source_id(source_id)?;
        if lexical.start > lexical.end {
            return Err(SourceMapError::ReversedRange);
        }
        let lexical_len = self.lexical_len();
        if lexical.end > lexical_len {
            return Err(SourceMapError::RangeOutsideLexicalText {
                source_id,
                range: lexical,
                lexical_len,
            });
        }
        if lexical.is_empty() {
            let anchors = self.source_anchors_for_lexical_offset(source_id, lexical.start)?;
            let is_synthetic_only_anchor =
                anchors.len() == 1 && self.synthetic_segment_touching_offset(lexical.start);
            let has_generated_anchor = anchors
                .iter()
                .any(|anchor| matches!(anchor, SourceAnchor::Generated(_)));
            return Ok(LexicalSourceMapping {
                primary: if is_synthetic_only_anchor || has_generated_anchor {
                    None
                } else {
                    primary_range_from_anchors(&anchors)
                },
                kind: if is_synthetic_only_anchor || has_generated_anchor {
                    LexicalSourceMappingKind::Degraded
                } else if anchors.len() == 1 {
                    LexicalSourceMappingKind::Exact
                } else {
                    LexicalSourceMappingKind::Composite
                },
                anchors,
            });
        }

        let mut cursor = lexical.start;
        let mut anchors = Vec::new();
        let mut primary_components = Vec::new();
        let mut saw_synthetic = false;

        while cursor < lexical.end {
            self.push_removed_comments_at_lexical_anchor(
                cursor,
                lexical,
                &mut anchors,
                &mut primary_components,
            )?;

            let Some((segment_lexical, segment_kind)) =
                self.lexical_segment_covering_offset(cursor)?
            else {
                return Err(SourceMapError::MissingPreprocessSegment {
                    source_id,
                    range: text_point(cursor),
                });
            };

            let covered_end = segment_lexical.end.min(lexical.end);
            match segment_kind {
                PreprocessSegmentKind::Original { source } => {
                    let mapped = SourceRange {
                        source_id,
                        start: source.start + (cursor - segment_lexical.start),
                        end: source.start + (covered_end - segment_lexical.start),
                    };
                    push_source_range(&mut primary_components, mapped);
                    push_source_anchor(&mut anchors, SourceAnchor::Range(mapped));
                }
                PreprocessSegmentKind::SyntheticWhitespace { anchor } => {
                    saw_synthetic = true;
                    push_source_anchor(&mut anchors, anchor);
                }
            }
            cursor = covered_end;
        }

        if anchors.is_empty() {
            return Err(SourceMapError::MissingPreprocessSegment {
                source_id,
                range: lexical,
            });
        }

        let primary = enclosing_source_range(&primary_components);
        let kind = if saw_synthetic || primary.is_none() {
            LexicalSourceMappingKind::Degraded
        } else if primary_components.len() == 1 && anchors.len() == 1 {
            LexicalSourceMappingKind::Exact
        } else {
            LexicalSourceMappingKind::Composite
        };

        Ok(LexicalSourceMapping {
            primary,
            anchors,
            kind,
        })
    }

    fn validate_source_id(&self, source_id: SourceId) -> Result<(), SourceMapError> {
        if source_id != self.source_id {
            return Err(SourceMapError::UnknownSourceId { source_id });
        }
        Ok(())
    }

    fn validate_source_range(&self, range: SourceRange) -> Result<(), SourceMapError> {
        self.validate_source_id(range.source_id)
    }

    fn validate_anchor(&self, anchor: &SourceAnchor) -> Result<(), SourceMapError> {
        match anchor {
            SourceAnchor::Range(range) => self.validate_source_range(*range),
            SourceAnchor::Point { source_id, .. } => self.validate_source_id(*source_id),
            SourceAnchor::Generated(origin) => self.validate_source_id(origin.anchor().source_id()),
        }
    }

    fn lexical_segment_covering_offset(
        &self,
        offset: usize,
    ) -> Result<Option<(TextRange, PreprocessSegmentKind)>, SourceMapError> {
        for segment in &self.segments {
            match segment {
                PreprocessSegment::Original { lexical, source }
                    if lexical.start <= offset && offset < lexical.end =>
                {
                    self.validate_source_range(*source)?;
                    return Ok(Some((
                        *lexical,
                        PreprocessSegmentKind::Original { source: *source },
                    )));
                }
                PreprocessSegment::SyntheticWhitespace { lexical, anchor }
                    if lexical.start <= offset && offset < lexical.end =>
                {
                    self.validate_anchor(anchor)?;
                    return Ok(Some((
                        *lexical,
                        PreprocessSegmentKind::SyntheticWhitespace {
                            anchor: anchor.clone(),
                        },
                    )));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn push_removed_comments_at_lexical_anchor(
        &self,
        offset: usize,
        lexical: TextRange,
        anchors: &mut Vec<SourceAnchor>,
        primary_components: &mut Vec<SourceRange>,
    ) -> Result<(), SourceMapError> {
        if !(lexical.start < offset && offset < lexical.end) {
            return Ok(());
        }

        for (index, segment) in self.segments.iter().enumerate() {
            let PreprocessSegment::RemovedComment { source, .. } = segment else {
                continue;
            };
            if self.lexical_anchor_for_removed_comment(index) == Some(offset) {
                self.validate_source_range(*source)?;
                push_source_range(primary_components, *source);
                push_source_anchor(anchors, SourceAnchor::Range(*source));
            }
        }
        Ok(())
    }

    fn lexical_anchor_for_removed_comment(&self, index: usize) -> Option<usize> {
        let previous = self.segments[..index]
            .iter()
            .rev()
            .find_map(PreprocessSegment::lexical_range)
            .map(|range| range.end);
        let next = self.segments[index + 1..]
            .iter()
            .find_map(PreprocessSegment::lexical_range)
            .map(|range| range.start);
        next.or(previous)
    }

    fn synthetic_segment_touching_offset(&self, offset: usize) -> bool {
        self.segments.iter().any(|segment| {
            matches!(
                segment,
                PreprocessSegment::SyntheticWhitespace { lexical, .. }
                    if lexical.start <= offset && offset <= lexical.end
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PreprocessSegmentKind {
    Original { source: SourceRange },
    SyntheticWhitespace { anchor: SourceAnchor },
}

impl PreprocessSegment {
    fn lexical_range(&self) -> Option<TextRange> {
        match self {
            Self::Original { lexical, .. } | Self::SyntheticWhitespace { lexical, .. } => {
                Some(*lexical)
            }
            Self::RemovedComment { .. } => None,
        }
    }
}

pub trait SourceMapService {
    fn line_column(&self, range: SourceRange) -> Result<(LineColumn, LineColumn), SourceMapError>;
    fn original_range_for_loaded(
        &self,
        source_id: SourceId,
        loaded: TextRange,
    ) -> Result<MappedSourceRange, SourceMapError>;
    fn source_range_for_lexical(
        &self,
        source_id: SourceId,
        lexical: TextRange,
    ) -> Result<MappedSourceRange, SourceMapError>;
    fn attach_generated_span(
        &self,
        origin: GeneratedSpanOrigin,
    ) -> Result<SourceAnchor, SourceMapError>;
    fn validate_range(&self, range: SourceRange) -> Result<(), SourceMapError>;
}

#[derive(Debug, Clone, Default)]
pub struct RetainedSourceMapService {
    line_maps: HashMap<SourceId, LineMap>,
    loading_maps: HashMap<SourceId, LoadingMap>,
    preprocess_maps: HashMap<SourceId, PreprocessMap>,
}

impl RetainedSourceMapService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_line_map(&mut self, line_map: LineMap) {
        self.line_maps.insert(line_map.source_id(), line_map);
    }

    pub fn insert_loading_map(&mut self, loading_map: LoadingMap) {
        self.loading_maps
            .insert(loading_map.source_id(), loading_map);
    }

    pub fn insert_preprocess_map(&mut self, preprocess_map: PreprocessMap) {
        self.preprocess_maps
            .insert(preprocess_map.source_id(), preprocess_map);
    }

    pub fn with_line_map(mut self, line_map: LineMap) -> Self {
        self.insert_line_map(line_map);
        self
    }

    pub fn with_loading_map(mut self, loading_map: LoadingMap) -> Self {
        self.insert_loading_map(loading_map);
        self
    }

    pub fn with_preprocess_map(mut self, preprocess_map: PreprocessMap) -> Self {
        self.insert_preprocess_map(preprocess_map);
        self
    }

    fn line_map(&self, source_id: SourceId) -> Result<&LineMap, SourceMapError> {
        self.line_maps
            .get(&source_id)
            .ok_or(SourceMapError::UnknownSourceId { source_id })
    }

    fn loading_map(
        &self,
        source_id: SourceId,
        range: TextRange,
    ) -> Result<&LoadingMap, SourceMapError> {
        self.loading_maps
            .get(&source_id)
            .ok_or(SourceMapError::MissingLoadingMapSegment { source_id, range })
    }

    fn preprocess_map(
        &self,
        source_id: SourceId,
        range: TextRange,
    ) -> Result<&PreprocessMap, SourceMapError> {
        self.preprocess_maps
            .get(&source_id)
            .ok_or(SourceMapError::MissingPreprocessSegment { source_id, range })
    }

    fn validate_anchor(&self, anchor: &SourceAnchor) -> Result<(), SourceMapError> {
        match anchor {
            SourceAnchor::Range(range) => self.validate_range(*range),
            SourceAnchor::Point { source_id, offset } => {
                self.validate_range(source_point(*source_id, *offset))
            }
            SourceAnchor::Generated(origin) => self.validate_generated_origin(origin),
        }
    }

    fn validate_generated_origin(
        &self,
        origin: &GeneratedSpanOrigin,
    ) -> Result<(), SourceMapError> {
        if origin.reason().trim().is_empty() {
            return Err(SourceMapError::GeneratedSpanWithoutOriginReason);
        }
        match origin.anchor() {
            GeneratedSpanAnchor::Range(range) => self.validate_range(range),
            GeneratedSpanAnchor::Point { source_id, offset } => {
                self.validate_range(source_point(source_id, offset))
            }
        }
    }

    fn mapped_source_range_from_anchors(
        &self,
        anchors: &[SourceAnchor],
    ) -> Result<Option<SourceRange>, SourceMapError> {
        for anchor in anchors {
            self.validate_anchor(anchor)?;
            if let Some(range) = source_range_from_anchor(anchor) {
                return Ok(Some(range));
            }
        }
        Ok(None)
    }
}

impl SourceMapService for RetainedSourceMapService {
    fn line_column(&self, range: SourceRange) -> Result<(LineColumn, LineColumn), SourceMapError> {
        let converted = self.line_map(range.source_id)?.line_column_range(range)?;
        Ok((converted.start, converted.end))
    }

    fn original_range_for_loaded(
        &self,
        source_id: SourceId,
        loaded: TextRange,
    ) -> Result<MappedSourceRange, SourceMapError> {
        self.line_map(source_id)?;
        let mapped = self
            .loading_map(source_id, loaded)?
            .original_range_for_loaded(source_id, loaded)?;
        let primary = SourceRange {
            source_id,
            start: loaded.start,
            end: loaded.end,
        };
        self.validate_range(primary)?;
        Ok(MappedSourceRange {
            primary,
            secondary: Vec::new(),
            original_input: Some(mapped.original),
            kind: match mapped.kind {
                LoadedToOriginalRangeKind::Exact => MappedSourceRangeKind::Exact,
                LoadedToOriginalRangeKind::Degraded => MappedSourceRangeKind::Degraded,
            },
        })
    }

    fn source_range_for_lexical(
        &self,
        source_id: SourceId,
        lexical: TextRange,
    ) -> Result<MappedSourceRange, SourceMapError> {
        self.line_map(source_id)?;
        let mapping = self
            .preprocess_map(source_id, lexical)?
            .source_range_for_lexical(source_id, lexical)?;
        let primary = mapping
            .primary
            .or(self.mapped_source_range_from_anchors(&mapping.anchors)?)
            .ok_or(SourceMapError::MissingPreprocessSegment {
                source_id,
                range: lexical,
            })?;
        self.validate_range(primary)?;
        let mut secondary = Vec::new();
        for anchor in mapping.anchors {
            self.validate_anchor(&anchor)?;
            if matches!(anchor, SourceAnchor::Generated(_))
                || source_range_from_anchor(&anchor) != Some(primary)
            {
                push_source_anchor(&mut secondary, anchor);
            }
        }
        Ok(MappedSourceRange {
            primary,
            secondary,
            original_input: None,
            kind: match mapping.kind {
                LexicalSourceMappingKind::Exact => MappedSourceRangeKind::Exact,
                LexicalSourceMappingKind::Composite => MappedSourceRangeKind::Composite,
                LexicalSourceMappingKind::Degraded => MappedSourceRangeKind::Degraded,
            },
        })
    }

    fn attach_generated_span(
        &self,
        origin: GeneratedSpanOrigin,
    ) -> Result<SourceAnchor, SourceMapError> {
        self.validate_generated_origin(&origin)?;
        Ok(SourceAnchor::Generated(origin))
    }

    fn validate_range(&self, range: SourceRange) -> Result<(), SourceMapError> {
        self.line_map(range.source_id)?.validate_range(range)
    }
}

fn map_loaded_subrange_to_original(
    loaded: TextRange,
    original: TextRange,
    subrange: TextRange,
    kind: LoadedToOriginalRangeKind,
) -> TextRange {
    match kind {
        LoadedToOriginalRangeKind::Exact => TextRange {
            start: original.start + (subrange.start - loaded.start),
            end: original.start + (subrange.end - loaded.start),
        },
        LoadedToOriginalRangeKind::Degraded => TextRange {
            start: if subrange.start == loaded.start {
                original.start
            } else {
                original.end
            },
            end: if subrange.end == loaded.start {
                original.start
            } else {
                original.end
            },
        },
    }
}

fn push_source_range(ranges: &mut Vec<SourceRange>, range: SourceRange) {
    if ranges.last() != Some(&range) {
        ranges.push(range);
    }
}

fn push_source_anchor(anchors: &mut Vec<SourceAnchor>, anchor: SourceAnchor) {
    if anchors.last() != Some(&anchor) {
        anchors.push(anchor);
    }
}

fn source_point(source_id: SourceId, offset: usize) -> SourceRange {
    SourceRange {
        source_id,
        start: offset,
        end: offset,
    }
}

fn text_point(offset: usize) -> TextRange {
    TextRange {
        start: offset,
        end: offset,
    }
}

fn primary_range_from_anchors(anchors: &[SourceAnchor]) -> Option<SourceRange> {
    match anchors {
        [SourceAnchor::Range(range)] => Some(*range),
        [SourceAnchor::Point { source_id, offset }] => Some(SourceRange {
            source_id: *source_id,
            start: *offset,
            end: *offset,
        }),
        _ => None,
    }
}

fn source_range_from_anchor(anchor: &SourceAnchor) -> Option<SourceRange> {
    match anchor {
        SourceAnchor::Range(range) => Some(*range),
        SourceAnchor::Point { source_id, offset } => Some(SourceRange {
            source_id: *source_id,
            start: *offset,
            end: *offset,
        }),
        SourceAnchor::Generated(origin) => Some(origin.anchor().to_source_range()),
    }
}

fn enclosing_source_range(ranges: &[SourceRange]) -> Option<SourceRange> {
    let first = ranges.first()?;
    Some(SourceRange {
        source_id: first.source_id,
        start: ranges
            .iter()
            .map(|range| range.start)
            .min()
            .unwrap_or(first.start),
        end: ranges
            .iter()
            .map(|range| range.end)
            .max()
            .unwrap_or(first.end),
    })
}

pub(crate) fn hash_source_text(source: &str) -> Hash {
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
mod tests;
