use crate::{Hash, NormalizedPath, SourceId};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceAnchor {
    Range(SourceRange),
    Point { source_id: SourceId, offset: usize },
    Generated,
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
                range: TextRange {
                    start: offset,
                    end: offset,
                },
                loaded_len,
            });
        }
        if self.segments.is_empty() && loaded_len == 0 && offset == 0 {
            return Ok(0);
        }

        self.original_offset_for_loaded_unchecked(offset).ok_or(
            SourceMapError::MissingLoadingMapSegment {
                source_id,
                range: TextRange {
                    start: offset,
                    end: offset,
                },
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
                    range: TextRange {
                        start: cursor,
                        end: cursor,
                    },
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
                range: TextRange {
                    start: offset,
                    end: offset,
                },
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
                    self.validate_anchor(*anchor)?;
                    push_source_anchor(&mut anchors, *anchor);
                }
                _ => {}
            }
        }

        if anchors.is_empty() {
            return Err(SourceMapError::MissingPreprocessSegment {
                source_id,
                range: TextRange {
                    start: offset,
                    end: offset,
                },
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
                .any(|anchor| matches!(anchor, SourceAnchor::Generated));
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
                    range: TextRange {
                        start: cursor,
                        end: cursor,
                    },
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

    fn validate_anchor(&self, anchor: SourceAnchor) -> Result<(), SourceMapError> {
        match anchor {
            SourceAnchor::Range(range) => self.validate_source_range(range),
            SourceAnchor::Point { source_id, .. } => self.validate_source_id(source_id),
            SourceAnchor::Generated => Ok(()),
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
                    self.validate_anchor(*anchor)?;
                    return Ok(Some((
                        *lexical,
                        PreprocessSegmentKind::SyntheticWhitespace { anchor: *anchor },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    use super::{
        CommentKind, LexicalSourceMapping, LexicalSourceMappingKind, LineColumn, LineColumnRange,
        LineMap, LoadedToOriginalRange, LoadedToOriginalRangeKind, LoadingMap, LoadingMapSegment,
        LoadingOrigin, PreprocessMap, PreprocessSegment, SourceAnchor, SourceMapError, SourceRange,
        TextRange,
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
        let non_empty =
            LoadingMap::new(non_empty_source_id, "abc", open_buffer_origin(), Vec::new());

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
                anchor: SourceAnchor::Generated,
            }],
        );

        assert_eq!(
            map.source_range_for_lexical(source_id, TextRange { start: 0, end: 0 }),
            Ok(LexicalSourceMapping {
                primary: None,
                anchors: vec![SourceAnchor::Generated],
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
                anchor: SourceAnchor::Generated,
            }],
        );

        assert_eq!(
            map.source_range_for_lexical(source_id, TextRange { start: 0, end: 1 }),
            Ok(LexicalSourceMapping {
                primary: None,
                anchors: vec![SourceAnchor::Generated],
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
            non_empty
                .source_range_for_lexical(non_empty_source_id, TextRange { start: 0, end: 1 },),
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

    fn line_map(source: &str) -> LineMap {
        LineMap::with_source(source_id(1), source)
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
}
