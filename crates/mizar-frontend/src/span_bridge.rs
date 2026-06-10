use mizar_lexer::{
    CommentKind as LexerCommentKind, SourcePreprocessMap, SourcePreprocessMapSegment,
    SourceSpan as LexerSourceSpan,
};
use mizar_session::{
    CommentKind, LineMap, LoadingMap, MappedSourceRange, MappedSourceRangeKind, PreprocessMap,
    PreprocessSegment, RetainedSourceMapService, SourceAnchor, SourceId, SourceMapError,
    SourceMapService, SourceRange, TextRange,
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub struct SpanBridge {
    source_maps: HashMap<SourceId, SourceRegistration>,
    preprocess_maps: HashMap<SourceId, PreprocessMap>,
    service: RetainedSourceMapService,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceRegistration {
    line_map: LineMap,
    loading_map: Option<LoadingMap>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexerByteSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpanBridgeError {
    SourceNotRegistered { source_id: SourceId },
    PreprocessMapNotRegistered { source_id: SourceId },
    ConflictingSourceRegistration { source_id: SourceId },
    ConflictingPreprocessMapRegistration { source_id: SourceId },
    UnsupportedLexerPreprocessMap { source_id: SourceId },
    SourceMap { source: SourceMapError },
}

impl SpanBridge {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn source_map_service(&self) -> &dyn SourceMapService {
        &self.service
    }

    pub fn register_source(
        &mut self,
        source_id: SourceId,
        line_map: LineMap,
        loading_map: Option<LoadingMap>,
    ) -> Result<(), SpanBridgeError> {
        validate_source_registration(source_id, &line_map, loading_map.as_ref())?;

        let registration = SourceRegistration {
            line_map,
            loading_map,
        };
        if let Some(existing) = self.source_maps.get(&source_id) {
            if existing == &registration {
                return Ok(());
            }
            return Err(SpanBridgeError::ConflictingSourceRegistration { source_id });
        }

        self.service.insert_line_map(registration.line_map.clone());
        if let Some(loading_map) = &registration.loading_map {
            self.service.insert_loading_map(loading_map.clone());
        }
        self.source_maps.insert(source_id, registration);
        Ok(())
    }

    pub fn register_preprocess_map(
        &mut self,
        source_id: SourceId,
        lexical_text: &str,
        preprocess_map: SourcePreprocessMap,
    ) -> Result<(), SpanBridgeError> {
        self.require_source(source_id)?;

        let preprocess_map =
            session_preprocess_map(source_id, lexical_text, preprocess_map, &self.service)?;
        if let Some(existing) = self.preprocess_maps.get(&source_id) {
            if existing == &preprocess_map {
                return Ok(());
            }
            return Err(SpanBridgeError::ConflictingPreprocessMapRegistration { source_id });
        }

        self.service.insert_preprocess_map(preprocess_map.clone());
        self.preprocess_maps.insert(source_id, preprocess_map);
        Ok(())
    }

    pub fn loaded_span(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<SourceRange, SpanBridgeError> {
        self.require_source(source_id)?;

        let range = SourceRange {
            source_id,
            start: span.start,
            end: span.end,
        };
        self.service.validate_range(range)?;
        Ok(range)
    }

    pub fn loaded_mapping(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<MappedSourceRange, SpanBridgeError> {
        let primary = self.loaded_span(source_id, span)?;
        if self.source_registration(source_id)?.loading_map.is_some() {
            return self
                .service
                .original_range_for_loaded(source_id, text_range_from_source(primary))
                .map_err(SpanBridgeError::from);
        }

        Ok(MappedSourceRange {
            primary,
            secondary: Vec::new(),
            original_input: None,
            kind: MappedSourceRangeKind::Exact,
        })
    }

    pub fn lexical_span(
        &self,
        source_id: SourceId,
        span: LexerByteSpan,
    ) -> Result<MappedSourceRange, SpanBridgeError> {
        self.require_source(source_id)?;
        if !self.preprocess_maps.contains_key(&source_id) {
            return Err(SpanBridgeError::PreprocessMapNotRegistered { source_id });
        }

        self.service
            .source_range_for_lexical(source_id, text_range_from_lexer(span)?)
            .map_err(SpanBridgeError::from)
    }

    pub(crate) fn whole_lexical_text_mapping(
        &self,
        source_id: SourceId,
        lexical_text: &str,
    ) -> Result<MappedSourceRange, SpanBridgeError> {
        if lexical_text.is_empty() {
            return self.loaded_mapping(source_id, LexerByteSpan { start: 0, end: 0 });
        }

        self.lexical_span(
            source_id,
            LexerByteSpan {
                start: 0,
                end: lexical_text.len(),
            },
        )
    }

    fn require_source(&self, source_id: SourceId) -> Result<(), SpanBridgeError> {
        self.source_registration(source_id).map(drop)
    }

    fn source_registration(
        &self,
        source_id: SourceId,
    ) -> Result<&SourceRegistration, SpanBridgeError> {
        self.source_maps
            .get(&source_id)
            .ok_or(SpanBridgeError::SourceNotRegistered { source_id })
    }
}

impl From<LexerSourceSpan> for LexerByteSpan {
    fn from(span: LexerSourceSpan) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl From<SourceMapError> for SpanBridgeError {
    fn from(source: SourceMapError) -> Self {
        Self::SourceMap { source }
    }
}

impl fmt::Display for SpanBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceNotRegistered { source_id } => {
                write!(
                    f,
                    "source `{source_id:?}` is not registered in the span bridge"
                )
            }
            Self::PreprocessMapNotRegistered { source_id } => {
                write!(
                    f,
                    "preprocess map for source `{source_id:?}` is not registered in the span bridge"
                )
            }
            Self::ConflictingSourceRegistration { source_id } => {
                write!(
                    f,
                    "source `{source_id:?}` was registered with conflicting source maps"
                )
            }
            Self::ConflictingPreprocessMapRegistration { source_id } => {
                write!(
                    f,
                    "source `{source_id:?}` was registered with conflicting preprocess maps"
                )
            }
            Self::UnsupportedLexerPreprocessMap { source_id } => {
                write!(
                    f,
                    "source `{source_id:?}` uses a lexer preprocess map variant this span bridge does not support"
                )
            }
            Self::SourceMap { source } => {
                write!(f, "session source-map conversion failed: {source:?}")
            }
        }
    }
}

impl Error for SpanBridgeError {}

fn validate_source_registration(
    source_id: SourceId,
    line_map: &LineMap,
    loading_map: Option<&LoadingMap>,
) -> Result<(), SpanBridgeError> {
    if line_map.source_id() != source_id {
        return Err(SourceMapError::UnknownSourceId { source_id }.into());
    }
    if loading_map.is_some_and(|loading_map| loading_map.source_id() != source_id) {
        return Err(SourceMapError::UnknownSourceId { source_id }.into());
    }
    if let Some(loading_map) = loading_map
        && (loading_map.loaded_len() != line_map.source().len()
            || loading_map.loaded_text_hash() != line_map.text_hash())
    {
        return Err(SpanBridgeError::ConflictingSourceRegistration { source_id });
    }
    Ok(())
}

fn session_preprocess_map(
    source_id: SourceId,
    lexical_text: &str,
    preprocess_map: SourcePreprocessMap,
    service: &dyn SourceMapService,
) -> Result<PreprocessMap, SpanBridgeError> {
    let segments = preprocess_map
        .segments
        .into_iter()
        .map(|segment| session_preprocess_segment(source_id, segment, service))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PreprocessMap::new(source_id, lexical_text, segments))
}

fn session_preprocess_segment(
    source_id: SourceId,
    segment: SourcePreprocessMapSegment,
    service: &dyn SourceMapService,
) -> Result<PreprocessSegment, SpanBridgeError> {
    match segment {
        SourcePreprocessMapSegment::Original { lexical, source } => {
            let source = source_range_from_lexer(source_id, source)?;
            service.validate_range(source)?;
            Ok(PreprocessSegment::Original {
                lexical: text_range_from_lexer_span(lexical)?,
                source,
            })
        }
        SourcePreprocessMapSegment::RemovedComment { source, kind } => {
            let source = source_range_from_lexer(source_id, source)?;
            service.validate_range(source)?;
            Ok(PreprocessSegment::RemovedComment {
                source,
                kind: comment_kind_from_lexer(source_id, kind)?,
            })
        }
        SourcePreprocessMapSegment::SyntheticWhitespace { lexical, anchor } => {
            let anchor = source_range_from_lexer(source_id, anchor)?;
            service.validate_range(anchor)?;
            Ok(PreprocessSegment::SyntheticWhitespace {
                lexical: text_range_from_lexer_span(lexical)?,
                anchor: SourceAnchor::Range(anchor),
            })
        }
        _ => Err(SpanBridgeError::UnsupportedLexerPreprocessMap { source_id }),
    }
}

pub(crate) fn comment_kind_from_lexer(
    source_id: SourceId,
    kind: LexerCommentKind,
) -> Result<CommentKind, SpanBridgeError> {
    Ok(match kind {
        LexerCommentKind::SingleLine => CommentKind::SingleLine,
        LexerCommentKind::MultiLine => CommentKind::MultiLine,
        LexerCommentKind::Documentation => CommentKind::Documentation,
        _ => return Err(SpanBridgeError::UnsupportedLexerPreprocessMap { source_id }),
    })
}

fn source_range_from_lexer(
    source_id: SourceId,
    span: LexerSourceSpan,
) -> Result<SourceRange, SpanBridgeError> {
    if span.start > span.end {
        return Err(SourceMapError::ReversedRange.into());
    }
    Ok(SourceRange {
        source_id,
        start: span.start,
        end: span.end,
    })
}

fn text_range_from_lexer(span: LexerByteSpan) -> Result<TextRange, SpanBridgeError> {
    text_range(span.start, span.end)
}

fn text_range_from_lexer_span(span: LexerSourceSpan) -> Result<TextRange, SpanBridgeError> {
    text_range(span.start, span.end)
}

fn text_range(start: usize, end: usize) -> Result<TextRange, SpanBridgeError> {
    TextRange::try_new(start, end).ok_or_else(|| SourceMapError::ReversedRange.into())
}

fn text_range_from_source(range: SourceRange) -> TextRange {
    TextRange {
        start: range.start,
        end: range.end,
    }
}

#[cfg(test)]
mod tests {
    use super::{LexerByteSpan, SpanBridge, SpanBridgeError};
    use mizar_lexer::preprocess_source_for_lexing;
    use mizar_session::{
        BuildSnapshotId, CommentKind, Hash, InMemorySessionIdAllocator, LineMap, LoadingMap,
        LoadingMapSegment, LoadingOrigin, MappedSourceRange, MappedSourceRangeKind, PreprocessMap,
        PreprocessSegment, SessionIdAllocator, SourceAnchor, SourceId, SourceMapError, SourceRange,
        TextRange,
    };

    #[test]
    fn loaded_span_stays_loaded_coordinate_while_mapping_reports_bom_original_offsets() {
        let source_id = source_id(1);
        let mut bridge = SpanBridge::new();
        bridge
            .register_source(
                source_id,
                LineMap::with_source(source_id, "alpha"),
                Some(bom_loading_map(source_id)),
            )
            .unwrap();

        let span = LexerByteSpan { start: 0, end: 5 };

        assert_eq!(
            bridge.loaded_span(source_id, span),
            Ok(SourceRange {
                source_id,
                start: 0,
                end: 5,
            })
        );
        assert_eq!(
            bridge.loaded_mapping(source_id, span),
            Ok(MappedSourceRange {
                primary: SourceRange {
                    source_id,
                    start: 0,
                    end: 5,
                },
                secondary: Vec::new(),
                original_input: Some(TextRange { start: 3, end: 8 }),
                kind: MappedSourceRangeKind::Exact,
            })
        );
    }

    #[test]
    fn identity_loaded_source_without_loading_map_returns_exact_loaded_mapping() {
        let source_id = source_id(1);
        let mut bridge = SpanBridge::new();
        bridge
            .register_source(source_id, LineMap::with_source(source_id, "alpha"), None)
            .unwrap();

        assert_eq!(
            bridge.loaded_mapping(source_id, LexerByteSpan { start: 1, end: 4 }),
            Ok(MappedSourceRange {
                primary: SourceRange {
                    source_id,
                    start: 1,
                    end: 4,
                },
                secondary: Vec::new(),
                original_input: None,
                kind: MappedSourceRangeKind::Exact,
            })
        );
        assert_eq!(
            bridge
                .source_map_service()
                .original_range_for_loaded(source_id, TextRange { start: 1, end: 4 }),
            Err(SourceMapError::MissingLoadingMapSegment {
                source_id,
                range: TextRange { start: 1, end: 4 },
            })
        );
    }

    #[test]
    fn lexical_span_maps_lexer_preprocess_map_to_loaded_source_coordinates() {
        let source_id = source_id(1);
        let source = "alpha beta";
        let preprocessed = preprocess_source_for_lexing(source);
        let mut bridge = registered_source(source_id, source);
        bridge
            .register_preprocess_map(
                source_id,
                &preprocessed.lexical_text,
                preprocessed.preprocess_map,
            )
            .unwrap();

        assert_eq!(
            bridge.lexical_span(source_id, LexerByteSpan { start: 6, end: 10 }),
            Ok(MappedSourceRange {
                primary: SourceRange {
                    source_id,
                    start: 6,
                    end: 10,
                },
                secondary: Vec::new(),
                original_input: None,
                kind: MappedSourceRangeKind::Exact,
            })
        );
    }

    #[test]
    fn lexical_span_crossing_removed_comment_yields_primary_and_secondary_anchors() {
        let source_id = source_id(1);
        let source = "alpha ::=comment=:: beta";
        let preprocessed = preprocess_source_for_lexing(source);
        assert_eq!(preprocessed.lexical_text, "alpha  beta");
        let mut bridge = registered_source(source_id, source);
        bridge
            .register_preprocess_map(
                source_id,
                &preprocessed.lexical_text,
                preprocessed.preprocess_map,
            )
            .unwrap();

        assert_eq!(
            bridge.lexical_span(source_id, LexerByteSpan { start: 0, end: 11 }),
            Ok(MappedSourceRange {
                primary: SourceRange {
                    source_id,
                    start: 0,
                    end: 24,
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
                        end: 19,
                    }),
                    SourceAnchor::Range(SourceRange {
                        source_id,
                        start: 19,
                        end: 24,
                    }),
                ],
                original_input: None,
                kind: MappedSourceRangeKind::Composite,
            })
        );
    }

    #[test]
    fn synthetic_only_lexical_span_yields_degraded_anchor_backed_mapping() {
        let source_id = source_id(1);
        let source = "alpha::=comment=::beta";
        let preprocessed = preprocess_source_for_lexing(source);
        assert_eq!(preprocessed.lexical_text, "alpha beta");
        let mut bridge = registered_source(source_id, source);
        bridge
            .register_preprocess_map(
                source_id,
                &preprocessed.lexical_text,
                preprocessed.preprocess_map,
            )
            .unwrap();

        assert_eq!(
            bridge.lexical_span(source_id, LexerByteSpan { start: 5, end: 6 }),
            Ok(MappedSourceRange {
                primary: SourceRange {
                    source_id,
                    start: 5,
                    end: 18,
                },
                secondary: Vec::new(),
                original_input: None,
                kind: MappedSourceRangeKind::Degraded,
            })
        );
    }

    #[test]
    fn whole_lexical_text_mapping_uses_loaded_start_for_empty_text() {
        let source_id = source_id(1);
        let preprocessed = preprocess_source_for_lexing("");
        let mut bridge = registered_source(source_id, "");
        bridge
            .register_preprocess_map(
                source_id,
                &preprocessed.lexical_text,
                preprocessed.preprocess_map,
            )
            .unwrap();

        assert_eq!(
            bridge.whole_lexical_text_mapping(source_id, &preprocessed.lexical_text),
            Ok(MappedSourceRange {
                primary: SourceRange {
                    source_id,
                    start: 0,
                    end: 0,
                },
                secondary: Vec::new(),
                original_input: None,
                kind: MappedSourceRangeKind::Exact,
            })
        );
    }

    #[test]
    fn whole_lexical_text_mapping_uses_preprocess_map_for_non_empty_text() {
        let source_id = source_id(1);
        let source = "alpha ::=comment=:: beta";
        let preprocessed = preprocess_source_for_lexing(source);
        assert_eq!(preprocessed.lexical_text, "alpha  beta");
        let mut bridge = registered_source(source_id, source);
        bridge
            .register_preprocess_map(
                source_id,
                &preprocessed.lexical_text,
                preprocessed.preprocess_map,
            )
            .unwrap();

        assert_eq!(
            bridge.whole_lexical_text_mapping(source_id, &preprocessed.lexical_text),
            Ok(MappedSourceRange {
                primary: SourceRange {
                    source_id,
                    start: 0,
                    end: source.len(),
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
                        end: 19,
                    }),
                    SourceAnchor::Range(SourceRange {
                        source_id,
                        start: 19,
                        end: 24,
                    }),
                ],
                original_input: None,
                kind: MappedSourceRangeKind::Composite,
            })
        );
    }

    #[test]
    fn loaded_spans_reject_non_utf8_boundaries_and_out_of_range_offsets() {
        let source_id = source_id(1);
        let bridge = registered_source(source_id, "aβ");

        assert_eq!(
            bridge.loaded_span(source_id, LexerByteSpan { start: 2, end: 2 }),
            Err(SpanBridgeError::SourceMap {
                source: SourceMapError::OffsetNotUtf8Boundary {
                    source_id,
                    offset: 2,
                },
            })
        );
        assert_eq!(
            bridge.loaded_span(source_id, LexerByteSpan { start: 0, end: 4 }),
            Err(SpanBridgeError::SourceMap {
                source: SourceMapError::RangeOutsideSourceText {
                    range: SourceRange {
                        source_id,
                        start: 0,
                        end: 4,
                    },
                    source_len: "aβ".len(),
                },
            })
        );
    }

    #[test]
    fn lexical_spans_reject_non_utf8_boundaries_and_out_of_range_offsets() {
        let source_id = source_id(1);
        let source = "aβ";
        let preprocessed = preprocess_source_for_lexing(source);
        let mut bridge = registered_source(source_id, source);
        bridge
            .register_preprocess_map(
                source_id,
                &preprocessed.lexical_text,
                preprocessed.preprocess_map,
            )
            .unwrap();

        assert_eq!(
            bridge.lexical_span(source_id, LexerByteSpan { start: 2, end: 2 }),
            Err(SpanBridgeError::SourceMap {
                source: SourceMapError::OffsetNotUtf8Boundary {
                    source_id,
                    offset: 2,
                },
            })
        );
        assert_eq!(
            bridge.lexical_span(source_id, LexerByteSpan { start: 0, end: 4 }),
            Err(SpanBridgeError::SourceMap {
                source: SourceMapError::RangeOutsideLexicalText {
                    source_id,
                    range: TextRange { start: 0, end: 4 },
                    lexical_len: "aβ".len(),
                },
            })
        );
    }

    #[test]
    fn missing_registrations_are_reported_before_session_lookup() {
        let registered_source_id = source_id(1);
        let unknown_source_id = source_id(2);
        let bridge = registered_source(registered_source_id, "alpha");

        assert_eq!(
            bridge.loaded_span(unknown_source_id, LexerByteSpan { start: 0, end: 1 }),
            Err(SpanBridgeError::SourceNotRegistered {
                source_id: unknown_source_id,
            })
        );
        assert_eq!(
            bridge.lexical_span(registered_source_id, LexerByteSpan { start: 0, end: 1 }),
            Err(SpanBridgeError::PreprocessMapNotRegistered {
                source_id: registered_source_id,
            })
        );
    }

    #[test]
    fn registration_rejects_unregistered_or_mismatched_source_ids() {
        let registered_source_id = source_id(1);
        let other_source_id = source_id(2);
        let preprocessed = preprocess_source_for_lexing("alpha");
        let mut bridge = SpanBridge::new();

        assert_eq!(
            bridge.register_preprocess_map(
                registered_source_id,
                &preprocessed.lexical_text,
                preprocessed.preprocess_map,
            ),
            Err(SpanBridgeError::SourceNotRegistered {
                source_id: registered_source_id,
            })
        );
        assert_eq!(
            bridge.register_source(
                registered_source_id,
                LineMap::with_source(other_source_id, "alpha"),
                None,
            ),
            Err(SpanBridgeError::SourceMap {
                source: SourceMapError::UnknownSourceId {
                    source_id: registered_source_id,
                },
            })
        );
        assert_eq!(
            bridge.register_source(
                registered_source_id,
                LineMap::with_source(registered_source_id, "alpha"),
                Some(bom_loading_map(other_source_id)),
            ),
            Err(SpanBridgeError::SourceMap {
                source: SourceMapError::UnknownSourceId {
                    source_id: registered_source_id,
                },
            })
        );
        assert_eq!(
            bridge.register_source(
                registered_source_id,
                LineMap::with_source(registered_source_id, "bravo"),
                Some(bom_loading_map(registered_source_id)),
            ),
            Err(SpanBridgeError::ConflictingSourceRegistration {
                source_id: registered_source_id,
            })
        );
    }

    #[test]
    fn conflicting_source_and_preprocess_registrations_are_reported() {
        let source_id = source_id(1);
        let mut bridge = registered_source(source_id, "alpha beta");
        assert_eq!(
            bridge.register_source(source_id, LineMap::with_source(source_id, "beta"), None),
            Err(SpanBridgeError::ConflictingSourceRegistration { source_id })
        );

        let first = preprocess_source_for_lexing("alpha beta");
        let second = mizar_lexer::SourcePreprocessMap {
            segments: vec![mizar_lexer::SourcePreprocessMapSegment::Original {
                lexical: mizar_lexer::SourceSpan { start: 0, end: 5 },
                source: mizar_lexer::SourceSpan { start: 0, end: 5 },
            }],
        };
        bridge
            .register_preprocess_map(source_id, &first.lexical_text, first.preprocess_map.clone())
            .unwrap();
        bridge
            .register_preprocess_map(source_id, &first.lexical_text, first.preprocess_map)
            .unwrap();
        assert_eq!(
            bridge.register_preprocess_map(source_id, "alpha", second),
            Err(SpanBridgeError::ConflictingPreprocessMapRegistration { source_id })
        );
    }

    #[test]
    fn converting_lexer_preprocess_map_rejects_invalid_loaded_source_anchors() {
        let source_id = source_id(1);
        let mut bridge = registered_source(source_id, "aβ");
        let preprocess_map = mizar_lexer::SourcePreprocessMap {
            segments: vec![mizar_lexer::SourcePreprocessMapSegment::Original {
                lexical: mizar_lexer::SourceSpan { start: 0, end: 1 },
                source: mizar_lexer::SourceSpan { start: 2, end: 2 },
            }],
        };

        assert_eq!(
            bridge.register_preprocess_map(source_id, "a", preprocess_map),
            Err(SpanBridgeError::SourceMap {
                source: SourceMapError::OffsetNotUtf8Boundary {
                    source_id,
                    offset: 2,
                },
            })
        );
    }

    #[test]
    fn unsupported_lexer_preprocess_map_variant_remains_public_defensive_surface() {
        let source_id = source_id(1);
        let error = SpanBridgeError::UnsupportedLexerPreprocessMap { source_id };

        assert_eq!(
            error.to_string(),
            format!(
                "source `{source_id:?}` uses a lexer preprocess map variant this span bridge does not support"
            )
        );
    }

    fn registered_source(source_id: SourceId, source: &str) -> SpanBridge {
        let mut bridge = SpanBridge::new();
        bridge
            .register_source(source_id, LineMap::with_source(source_id, source), None)
            .unwrap();
        bridge
    }

    fn bom_loading_map(source_id: SourceId) -> LoadingMap {
        LoadingMap::new(
            source_id,
            "alpha",
            LoadingOrigin::OpenBufferText {
                uri: "file:///pkg/src/test.miz".to_owned(),
                version: 1,
            },
            vec![
                LoadingMapSegment::RemovedLeadingBom {
                    original: TextRange { start: 0, end: 3 },
                },
                LoadingMapSegment::Original {
                    loaded: TextRange { start: 0, end: 5 },
                    original: TextRange { start: 3, end: 8 },
                },
            ],
        )
    }

    fn source_id(seed: u8) -> SourceId {
        let allocator = InMemorySessionIdAllocator::new();
        let snapshot = snapshot_id(seed);
        let mut source_id = allocator.next_source_id(snapshot).unwrap();
        for _ in 1..seed {
            source_id = allocator.next_source_id(snapshot).unwrap();
        }
        source_id
    }

    fn snapshot_id(seed: u8) -> BuildSnapshotId {
        let bytes = [seed; Hash::BYTE_LEN];
        let mut serialized = String::from("mizar-session-build-snapshot-v1:");
        for byte in bytes {
            serialized.push_str(&format!("{byte:02x}"));
        }
        BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
    }

    #[test]
    fn converted_lexer_comment_kinds_match_session_comment_kinds() {
        let source_id = source_id(1);
        let source = "alpha :: comment\n::: doc\n::=multi=:: beta";
        let preprocessed = preprocess_source_for_lexing(source);
        let mut bridge = registered_source(source_id, source);
        bridge
            .register_preprocess_map(
                source_id,
                &preprocessed.lexical_text,
                preprocessed.preprocess_map,
            )
            .unwrap();

        let session_map = bridge.preprocess_maps.get(&source_id).unwrap();
        let kinds = session_map
            .segments
            .iter()
            .filter_map(|segment| match segment {
                PreprocessSegment::RemovedComment { kind, .. } => Some(*kind),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                CommentKind::SingleLine,
                CommentKind::Documentation,
                CommentKind::MultiLine,
            ]
        );
    }

    #[test]
    fn session_preprocess_map_hashes_the_registered_lexical_text() {
        let source_id = source_id(1);
        let source = "alpha beta";
        let preprocessed = preprocess_source_for_lexing(source);
        let session_identity = PreprocessMap::identity(source_id, &preprocessed.lexical_text);
        let mut bridge = registered_source(source_id, source);
        bridge
            .register_preprocess_map(
                source_id,
                &preprocessed.lexical_text,
                preprocessed.preprocess_map,
            )
            .unwrap();

        assert_eq!(
            bridge
                .preprocess_maps
                .get(&source_id)
                .unwrap()
                .lexical_text_hash(),
            session_identity.lexical_text_hash()
        );
    }
}
