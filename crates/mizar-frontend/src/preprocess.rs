//! Source preprocessing, comment retention, and import pre-scan integration.
//!
//! Canonical behavior is specified in the
//! [preprocess design spec](../../../../doc/design/mizar-frontend/en/preprocess.md).

use crate::source::SourceUnit;
use crate::span_bridge::{
    LexerByteSpan, SpanBridge, SpanBridgeError, comment_kind_from_lexer, decode_offsets,
    decode_source_anchor, decode_source_range, encode_offsets, encode_source_anchor,
    encode_source_range,
};
use mizar_lexer::{
    CommentKind as LexerCommentKind, ImportPrescanDiagnostic as LexerImportPrescanDiagnostic,
    ImportStub as LexerImportStub, RawModuleAlias as LexerRawModuleAlias,
    RawModulePath as LexerRawModulePath, RawModuleRelativePrefix as LexerRawModuleRelativePrefix,
    RawScanDiagnostic, RawScanDiagnosticCode, RawToken, RawTokenKind, RawTokenStream,
    RecoverableRawTokenStream, SourcePreprocessMap, SourcePreprocessMapSegment,
    SourceSpan as LexerSourceSpan, preprocess_source_for_lexing, scan_import_prelude,
    scan_raw_recoverable,
};
/// Re-exported import pre-scan diagnostic codes from the lexer crate.
pub use mizar_lexer::{ImportPrescanDiagnosticCode, SourcePreprocessDiagnosticCode};
/// Re-exported comment classification shared with session source maps.
pub use mizar_session::CommentKind;
use mizar_session::{Hash, MappedSourceRange, SourceAnchor, SourceId, SourceRange};
use std::sync::Arc;

const LEXICAL_HASH_DOMAIN: &[u8] = b"mizar-frontend/preprocess/lexical-text/v1";
const PREPROCESSED_SOURCE_SCHEMA: &str = "mizar-frontend/preprocessed-source/v1";
const PREPROCESSED_SOURCE_MAX_BYTES: usize = 16 * 1024 * 1024;

/// Preprocessed source text and metadata passed to later frontend phases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessedSource {
    /// Session source id for the original source unit.
    pub source_id: SourceId,
    /// Lexical text after comment removal and source preprocessing.
    pub lexical_text: LexicalText,
    /// Stable hash of the lexical text.
    pub lexical_hash: Hash,
    /// Non-documentation comments retained from the original source.
    pub comments: Vec<Comment>,
    /// Documentation comments retained from the original source.
    pub doc_comments: Vec<DocComment>,
    /// Shallow import declarations discovered before parsing.
    pub import_stubs: Vec<ImportStub>,
    /// Mapping from lexical text back to source coordinates.
    pub source_map: LexicalSourceMap,
    /// Recoverable diagnostics produced during preprocessing.
    pub diagnostics: Vec<PreprocessDiagnostic>,
}

impl PreprocessedSource {
    /// Returns the canonical compiler-internal storage representation.
    ///
    /// The bytes preserve preprocessing output but do not publish a frontend
    /// phase or grant cache reuse. Session source IDs are omitted and will be
    /// rebound by [`Self::from_canonical_bytes`].
    pub fn canonical_bytes(&self) -> Option<Vec<u8>> {
        if self.source_map.source_id != self.source_id
            || self.source_map.lexical_text_len != self.lexical_text.text.len()
            || self.lexical_hash != lexical_hash(&self.lexical_text)
        {
            return None;
        }

        let comments = self
            .comments
            .iter()
            .map(|comment| {
                let kind = match comment.kind {
                    CommentKind::SingleLine => "single_line",
                    CommentKind::MultiLine => "multi_line",
                    CommentKind::Documentation => "documentation",
                    _ => return None,
                };
                Some(serde_json::json!([
                    kind,
                    encode_source_range(comment.source_range, self.source_id)?
                ]))
            })
            .collect::<Option<Vec<_>>>()?;
        let doc_comments = self
            .doc_comments
            .iter()
            .map(|comment| {
                Some(serde_json::json!([
                    encode_source_range(comment.source_range, self.source_id)?,
                    comment.raw_body.as_ref()
                ]))
            })
            .collect::<Option<Vec<_>>>()?;
        let import_stubs = self
            .import_stubs
            .iter()
            .map(|stub| {
                let relative = match stub.path.relative {
                    Some(ImportStubRelativePrefix::Current) => serde_json::json!("current"),
                    Some(ImportStubRelativePrefix::Parent) => serde_json::json!("parent"),
                    None => serde_json::Value::Null,
                };
                let components = stub
                    .path
                    .components
                    .iter()
                    .map(|component| serde_json::json!(component.as_ref()))
                    .collect::<Vec<_>>();
                let source_segments = stub
                    .path
                    .source_segments
                    .iter()
                    .map(|range| encode_source_range(*range, self.source_id))
                    .collect::<Option<Vec<_>>>()?;
                let path = serde_json::json!([
                    stub.path.spelling.as_ref(),
                    relative,
                    components,
                    source_segments,
                    encode_source_range(stub.path.span, self.source_id)?
                ]);
                let alias = match &stub.alias {
                    Some(alias) => serde_json::json!([
                        alias.spelling.as_ref(),
                        encode_source_range(alias.span, self.source_id)?
                    ]),
                    None => serde_json::Value::Null,
                };
                Some(serde_json::json!([
                    path,
                    alias,
                    encode_source_range(stub.span, self.source_id)?
                ]))
            })
            .collect::<Option<Vec<_>>>()?;
        let map_segments = self
            .source_map
            .preprocess_map
            .segments
            .iter()
            .map(|segment| match segment {
                SourcePreprocessMapSegment::Original { lexical, source }
                    if valid_lexical_range(lexical.start, lexical.end, &self.lexical_text.text) =>
                {
                    Some(serde_json::json!([
                        "original",
                        encode_offsets(lexical.start, lexical.end)?,
                        encode_offsets(source.start, source.end)?
                    ]))
                }
                SourcePreprocessMapSegment::RemovedComment { source, kind } => {
                    let kind = match kind {
                        LexerCommentKind::SingleLine => "single_line",
                        LexerCommentKind::MultiLine => "multi_line",
                        LexerCommentKind::Documentation => "documentation",
                        _ => return None,
                    };
                    Some(serde_json::json!([
                        "removed_comment",
                        encode_offsets(source.start, source.end)?,
                        kind
                    ]))
                }
                SourcePreprocessMapSegment::SyntheticWhitespace { lexical, anchor }
                    if valid_lexical_range(lexical.start, lexical.end, &self.lexical_text.text) =>
                {
                    Some(serde_json::json!([
                        "synthetic_whitespace",
                        encode_offsets(lexical.start, lexical.end)?,
                        encode_offsets(anchor.start, anchor.end)?
                    ]))
                }
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        let diagnostics = self
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let kind = match diagnostic.kind {
                    PreprocessDiagnosticKind::SourcePrecondition(code) => match code {
                        SourcePreprocessDiagnosticCode::CarriageReturn => "source.carriage_return",
                        SourcePreprocessDiagnosticCode::NonAsciiCode => "source.non_ascii_code",
                        SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment => {
                            "source.unterminated_multi_line_comment"
                        }
                        _ => return None,
                    },
                    PreprocessDiagnosticKind::ImportPrescan(code) => match code {
                        ImportPrescanDiagnosticCode::MissingModulePath => {
                            "import.missing_module_path"
                        }
                        ImportPrescanDiagnosticCode::EmptyModulePathComponent => {
                            "import.empty_module_path_component"
                        }
                        ImportPrescanDiagnosticCode::MissingAlias => "import.missing_alias",
                        ImportPrescanDiagnosticCode::MissingSemicolon => "import.missing_semicolon",
                        ImportPrescanDiagnosticCode::UnexpectedToken => "import.unexpected_token",
                        _ => return None,
                    },
                    PreprocessDiagnosticKind::RawImportScan => "raw_import_scan",
                };
                let secondary = diagnostic
                    .secondary
                    .iter()
                    .map(|anchor| encode_source_anchor(anchor, self.source_id))
                    .collect::<Option<Vec<_>>>()?;
                Some(serde_json::json!([
                    kind,
                    diagnostic.message.as_ref(),
                    encode_source_range(diagnostic.primary, self.source_id)?,
                    secondary
                ]))
            })
            .collect::<Option<Vec<_>>>()?;

        let value = serde_json::json!([
            PREPROCESSED_SOURCE_SCHEMA,
            self.lexical_text.text.as_ref(),
            comments,
            doc_comments,
            import_stubs,
            map_segments,
            diagnostics
        ]);
        let bytes = serde_json::to_vec(&value).ok()?;
        (bytes.len() <= PREPROCESSED_SOURCE_MAX_BYTES).then_some(bytes)
    }

    /// Decodes canonical compiler-internal storage and rebinds every retained
    /// source range and anchor to `source_id` for the current session.
    ///
    /// This does not load source text, rerun preprocessing, or register the
    /// decoded map with a [`SpanBridge`]. Consumers must register the matching
    /// source and map before using source mappings.
    pub fn from_canonical_bytes(bytes: &[u8], source_id: SourceId) -> Option<Self> {
        if bytes.len() > PREPROCESSED_SOURCE_MAX_BYTES {
            return None;
        }
        let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
        let [
            schema,
            lexical_text,
            comments,
            doc_comments,
            import_stubs,
            map_segments,
            diagnostics,
        ] = value.as_array()?.as_slice()
        else {
            return None;
        };
        if schema.as_str()? != PREPROCESSED_SOURCE_SCHEMA {
            return None;
        }

        let lexical_text = LexicalText {
            text: Arc::<str>::from(lexical_text.as_str()?),
        };
        let comments = comments
            .as_array()?
            .iter()
            .map(|value| {
                let [kind, range] = value.as_array()?.as_slice() else {
                    return None;
                };
                let kind = match kind.as_str()? {
                    "single_line" => CommentKind::SingleLine,
                    "multi_line" => CommentKind::MultiLine,
                    "documentation" => CommentKind::Documentation,
                    _ => return None,
                };
                Some(Comment {
                    kind,
                    source_range: decode_source_range(range, source_id)?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let doc_comments = doc_comments
            .as_array()?
            .iter()
            .map(|value| {
                let [range, raw_body] = value.as_array()?.as_slice() else {
                    return None;
                };
                Some(DocComment {
                    source_range: decode_source_range(range, source_id)?,
                    raw_body: Arc::<str>::from(raw_body.as_str()?),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let import_stubs = import_stubs
            .as_array()?
            .iter()
            .map(|value| {
                let [path, alias, span] = value.as_array()?.as_slice() else {
                    return None;
                };
                let [spelling, relative, components, source_segments, path_span] =
                    path.as_array()?.as_slice()
                else {
                    return None;
                };
                let relative = match relative {
                    serde_json::Value::Null => None,
                    serde_json::Value::String(tag) if tag == "current" => {
                        Some(ImportStubRelativePrefix::Current)
                    }
                    serde_json::Value::String(tag) if tag == "parent" => {
                        Some(ImportStubRelativePrefix::Parent)
                    }
                    _ => return None,
                };
                let components = components
                    .as_array()?
                    .iter()
                    .map(|component| Some(Arc::<str>::from(component.as_str()?)))
                    .collect::<Option<Vec<_>>>()?;
                let source_segments = source_segments
                    .as_array()?
                    .iter()
                    .map(|range| decode_source_range(range, source_id))
                    .collect::<Option<Vec<_>>>()?;
                let alias = if alias.is_null() {
                    None
                } else {
                    let [spelling, span] = alias.as_array()?.as_slice() else {
                        return None;
                    };
                    Some(ImportStubAlias {
                        spelling: Arc::<str>::from(spelling.as_str()?),
                        span: decode_source_range(span, source_id)?,
                    })
                };
                Some(ImportStub {
                    path: ImportStubPath {
                        spelling: Arc::<str>::from(spelling.as_str()?),
                        relative,
                        components,
                        source_segments,
                        span: decode_source_range(path_span, source_id)?,
                    },
                    alias,
                    span: decode_source_range(span, source_id)?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let map_segments = map_segments
            .as_array()?
            .iter()
            .map(|value| {
                let values = value.as_array()?;
                match values.first()?.as_str()? {
                    "original" => {
                        let [_, lexical, source] = values.as_slice() else {
                            return None;
                        };
                        let (start, end) = decode_offsets(lexical)?;
                        if !valid_lexical_range(start, end, &lexical_text.text) {
                            return None;
                        }
                        let (source_start, source_end) = decode_offsets(source)?;
                        Some(SourcePreprocessMapSegment::Original {
                            lexical: LexerSourceSpan { start, end },
                            source: LexerSourceSpan {
                                start: source_start,
                                end: source_end,
                            },
                        })
                    }
                    "removed_comment" => {
                        let [_, source, kind] = values.as_slice() else {
                            return None;
                        };
                        let (start, end) = decode_offsets(source)?;
                        let kind = match kind.as_str()? {
                            "single_line" => LexerCommentKind::SingleLine,
                            "multi_line" => LexerCommentKind::MultiLine,
                            "documentation" => LexerCommentKind::Documentation,
                            _ => return None,
                        };
                        Some(SourcePreprocessMapSegment::RemovedComment {
                            source: LexerSourceSpan { start, end },
                            kind,
                        })
                    }
                    "synthetic_whitespace" => {
                        let [_, lexical, anchor] = values.as_slice() else {
                            return None;
                        };
                        let (start, end) = decode_offsets(lexical)?;
                        if !valid_lexical_range(start, end, &lexical_text.text) {
                            return None;
                        }
                        let (anchor_start, anchor_end) = decode_offsets(anchor)?;
                        Some(SourcePreprocessMapSegment::SyntheticWhitespace {
                            lexical: LexerSourceSpan { start, end },
                            anchor: LexerSourceSpan {
                                start: anchor_start,
                                end: anchor_end,
                            },
                        })
                    }
                    _ => None,
                }
            })
            .collect::<Option<Vec<_>>>()?;
        let diagnostics = diagnostics
            .as_array()?
            .iter()
            .map(|value| {
                let [kind, message, primary, secondary] = value.as_array()?.as_slice() else {
                    return None;
                };
                let kind = match kind.as_str()? {
                    "source.carriage_return" => PreprocessDiagnosticKind::SourcePrecondition(
                        SourcePreprocessDiagnosticCode::CarriageReturn,
                    ),
                    "source.non_ascii_code" => PreprocessDiagnosticKind::SourcePrecondition(
                        SourcePreprocessDiagnosticCode::NonAsciiCode,
                    ),
                    "source.unterminated_multi_line_comment" => {
                        PreprocessDiagnosticKind::SourcePrecondition(
                            SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment,
                        )
                    }
                    "import.missing_module_path" => PreprocessDiagnosticKind::ImportPrescan(
                        ImportPrescanDiagnosticCode::MissingModulePath,
                    ),
                    "import.empty_module_path_component" => {
                        PreprocessDiagnosticKind::ImportPrescan(
                            ImportPrescanDiagnosticCode::EmptyModulePathComponent,
                        )
                    }
                    "import.missing_alias" => PreprocessDiagnosticKind::ImportPrescan(
                        ImportPrescanDiagnosticCode::MissingAlias,
                    ),
                    "import.missing_semicolon" => PreprocessDiagnosticKind::ImportPrescan(
                        ImportPrescanDiagnosticCode::MissingSemicolon,
                    ),
                    "import.unexpected_token" => PreprocessDiagnosticKind::ImportPrescan(
                        ImportPrescanDiagnosticCode::UnexpectedToken,
                    ),
                    "raw_import_scan" => PreprocessDiagnosticKind::RawImportScan,
                    _ => return None,
                };
                Some(PreprocessDiagnostic {
                    kind,
                    message: Arc::<str>::from(message.as_str()?),
                    primary: decode_source_range(primary, source_id)?,
                    secondary: secondary
                        .as_array()?
                        .iter()
                        .map(|anchor| decode_source_anchor(anchor, source_id))
                        .collect::<Option<Vec<_>>>()?,
                })
            })
            .collect::<Option<Vec<_>>>()?;

        let source = Self {
            source_id,
            lexical_hash: lexical_hash(&lexical_text),
            source_map: LexicalSourceMap {
                source_id,
                lexical_text_len: lexical_text.text.len(),
                preprocess_map: SourcePreprocessMap {
                    segments: map_segments,
                },
            },
            lexical_text,
            comments,
            doc_comments,
            import_stubs,
            diagnostics,
        };
        (source.canonical_bytes()?.as_slice() == bytes).then_some(source)
    }
}

fn valid_lexical_range(start: usize, end: usize, lexical_text: &str) -> bool {
    start <= end
        && end <= lexical_text.len()
        && lexical_text.is_char_boundary(start)
        && lexical_text.is_char_boundary(end)
}

/// Interned lexical text produced by preprocessing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalText {
    /// Preprocessed text used by the lexer.
    pub text: Arc<str>,
}

impl LexicalText {
    /// Returns the lexical text as a string slice.
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

/// Retained non-documentation source comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// Source-level comment kind.
    pub kind: CommentKind,
    /// Source range occupied by the comment.
    pub source_range: SourceRange,
}

/// Retained documentation comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocComment {
    /// Source range occupied by the documentation comment.
    pub source_range: SourceRange,
    /// Documentation body without the comment marker.
    pub raw_body: Arc<str>,
}

/// Source map metadata for the lexical text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalSourceMap {
    /// Source id that owns this lexical map.
    pub source_id: SourceId,
    /// Length in bytes of the lexical text.
    pub lexical_text_len: usize,
    /// Lexer-owned preprocess map retained for diagnostics and cache inputs.
    pub preprocess_map: SourcePreprocessMap,
}

impl LexicalSourceMap {
    /// Maps a lexical byte span back to source coordinates.
    pub fn lexical_span(
        &self,
        bridge: &SpanBridge,
        span: LexerByteSpan,
    ) -> Result<MappedSourceRange, SpanBridgeError> {
        bridge.lexical_span(self.source_id, span)
    }

    /// Returns the lexical text length in bytes.
    pub const fn lexical_len(&self) -> usize {
        self.lexical_text_len
    }

    /// Returns whether the lexical text is empty.
    pub const fn is_empty(&self) -> bool {
        self.lexical_text_len == 0
    }
}

/// Shallow import declaration discovered in the lexical prelude.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStub {
    /// Imported module path.
    pub path: ImportStubPath,
    /// Optional import alias.
    pub alias: Option<ImportStubAlias>,
    /// Source range of the import declaration.
    pub span: SourceRange,
}

/// Parsed module path from a shallow import declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStubPath {
    /// Original path spelling.
    pub spelling: Arc<str>,
    /// Optional relative import prefix.
    pub relative: Option<ImportStubRelativePrefix>,
    /// Dot-separated path components.
    pub components: Vec<Arc<str>>,
    /// Source ranges for individual path segments.
    pub source_segments: Vec<SourceRange>,
    /// Source range of the full path.
    pub span: SourceRange,
}

/// Relative prefix attached to an import path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportStubRelativePrefix {
    /// Current module or package-relative import.
    Current,
    /// Parent module or package-relative import.
    Parent,
}

/// Alias attached to a shallow import declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStubAlias {
    /// Alias spelling.
    pub spelling: Arc<str>,
    /// Source range of the alias spelling.
    pub span: SourceRange,
}

/// Recoverable preprocessing diagnostic mapped to source coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessDiagnostic {
    /// Preprocessing diagnostic category.
    pub kind: PreprocessDiagnosticKind,
    /// Human-readable diagnostic message.
    pub message: Arc<str>,
    /// Primary source range for the diagnostic.
    pub primary: SourceRange,
    /// Secondary source anchors for related context.
    pub secondary: Vec<SourceAnchor>,
}

/// Kind of recoverable preprocessing diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PreprocessDiagnosticKind {
    /// Source precondition diagnostic reported by preprocessing.
    SourcePrecondition(SourcePreprocessDiagnosticCode),
    /// Import pre-scan diagnostic reported by the lexer.
    ImportPrescan(ImportPrescanDiagnosticCode),
    /// Recoverable raw scan diagnostic encountered while scanning imports.
    RawImportScan,
}

/// Preprocesses one source unit and registers its lexical source map.
pub fn preprocess(
    source: &SourceUnit,
    bridge: &mut SpanBridge,
) -> Result<PreprocessedSource, SpanBridgeError> {
    let preprocessed = preprocess_source_for_lexing(source.source_text.as_ref());
    let lexical_text = LexicalText {
        text: Arc::<str>::from(preprocessed.lexical_text),
    };
    let source_map = LexicalSourceMap {
        source_id: source.source_id,
        lexical_text_len: lexical_text.text.len(),
        preprocess_map: preprocessed.preprocess_map.clone(),
    };

    bridge.register_preprocess_map(
        source.source_id,
        lexical_text.text.as_ref(),
        preprocessed.preprocess_map,
    )?;

    let mut comments = Vec::new();
    let mut doc_comments = Vec::new();
    for comment in preprocessed.comments {
        let source_range =
            bridge.loaded_span(source.source_id, LexerByteSpan::from(comment.span))?;
        if comment.kind == LexerCommentKind::Documentation {
            doc_comments.push(DocComment {
                source_range,
                raw_body: Arc::<str>::from(doc_comment_body(&comment.lexeme)),
            });
        } else {
            comments.push(Comment {
                kind: comment_kind_from_lexer(source.source_id, comment.kind)?,
                source_range,
            });
        }
    }

    let mut diagnostics = preprocessed
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            let mapping =
                bridge.loaded_mapping(source.source_id, LexerByteSpan::from(diagnostic.span))?;
            Ok(PreprocessDiagnostic {
                kind: PreprocessDiagnosticKind::SourcePrecondition(diagnostic.code),
                message: Arc::<str>::from(diagnostic.message),
                primary: mapping.primary,
                secondary: mapping.secondary,
            })
        })
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    let (import_stubs, mut import_diagnostics) =
        scan_imports(source.source_id, &lexical_text, bridge)?;
    diagnostics.append(&mut import_diagnostics);

    let lexical_hash = lexical_hash(&lexical_text);
    Ok(PreprocessedSource {
        source_id: source.source_id,
        lexical_text,
        lexical_hash,
        comments,
        doc_comments,
        import_stubs,
        source_map,
        diagnostics,
    })
}

/// Computes the stable hash of lexical text used by downstream cache keys.
pub fn lexical_hash(lexical_text: &LexicalText) -> Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(LEXICAL_HASH_DOMAIN);
    hasher.update(&(lexical_text.text.len() as u64).to_le_bytes());
    hasher.update(lexical_text.text.as_bytes());
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

fn doc_comment_body(lexeme: &str) -> &str {
    lexeme.strip_prefix(":::").unwrap_or(lexeme)
}

fn scan_imports(
    source_id: SourceId,
    lexical_text: &LexicalText,
    bridge: &SpanBridge,
) -> Result<(Vec<ImportStub>, Vec<PreprocessDiagnostic>), SpanBridgeError> {
    let raw_scan = match scan_raw_with_string_argument_spans(lexical_text.as_str()) {
        Ok(raw_scan) => raw_scan,
        Err(error) => {
            return Ok((
                Vec::new(),
                vec![raw_import_scan_diagnostic(
                    source_id,
                    bridge,
                    &RawScanDiagnostic::new(
                        RawScanDiagnosticCode::UnsupportedInput,
                        error.to_string(),
                        LexerSourceSpan {
                            start: 0,
                            end: lexical_text.as_str().len(),
                        },
                    ),
                )?],
            ));
        }
    };
    let raw = RawTokenStream::new(raw_scan.tokens);
    let prelude = scan_import_prelude(&raw);
    let import_stubs = prelude
        .imports
        .into_iter()
        .map(|import| import_stub(source_id, bridge, import))
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    let mut diagnostics = raw_scan
        .diagnostics
        .iter()
        .map(|diagnostic| raw_import_scan_diagnostic(source_id, bridge, diagnostic))
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    diagnostics.extend(
        prelude
            .diagnostics
            .into_iter()
            .map(|diagnostic| import_prescan_diagnostic(source_id, bridge, diagnostic))
            .collect::<Result<Vec<_>, SpanBridgeError>>()?,
    );
    Ok((import_stubs, diagnostics))
}

fn scan_raw_with_string_argument_spans(
    lexical_text: &str,
) -> Result<RecoverableRawTokenStream, String> {
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();
    let mut cursor = 0;
    for range in string_argument_ranges(lexical_text) {
        scan_raw_segment(
            lexical_text,
            cursor,
            range.start,
            &mut tokens,
            &mut diagnostics,
        )?;
        push_string_argument_raw_token(lexical_text, range, &mut tokens)?;
        cursor = range.end;
    }
    scan_raw_segment(
        lexical_text,
        cursor,
        lexical_text.len(),
        &mut tokens,
        &mut diagnostics,
    )?;
    Ok(RecoverableRawTokenStream::new(tokens, diagnostics))
}

fn scan_raw_segment(
    lexical_text: &str,
    start: usize,
    end: usize,
    tokens: &mut Vec<RawToken>,
    diagnostics: &mut Vec<RawScanDiagnostic>,
) -> Result<(), String> {
    if start == end {
        return Ok(());
    }
    let segment = lexical_text
        .get(start..end)
        .ok_or_else(|| format!("string-argument scan range {start}..{end} is not a UTF-8 span"))?;
    let (raw_tokens, raw_diagnostics) = scan_raw_recoverable(segment).into_parts();
    tokens.extend(raw_tokens.into_iter().map(|token| {
        RawToken::new(
            token.kind,
            token.lexeme,
            LexerSourceSpan {
                start: token.span.start + start,
                end: token.span.end + start,
            },
        )
    }));
    diagnostics.extend(raw_diagnostics.into_iter().map(|diagnostic| {
        let span = LexerSourceSpan {
            start: diagnostic.span.start + start,
            end: diagnostic.span.end + start,
        };
        RawScanDiagnostic::new(
            diagnostic.code,
            raw_scan_diagnostic_message(diagnostic.code, lexical_text, span.start),
            span,
        )
    }));
    Ok(())
}

fn push_string_argument_raw_token(
    lexical_text: &str,
    range: LexicalByteRange,
    tokens: &mut Vec<RawToken>,
) -> Result<(), String> {
    let lexeme = lexical_text.get(range.start..range.end).ok_or_else(|| {
        format!(
            "string-argument range {}..{} is not a UTF-8 span",
            range.start, range.end
        )
    })?;
    if lexeme.chars().any(|ch| matches!(ch, '\n' | '\r')) {
        return Err(format!(
            "string-argument range {}..{} crosses a line boundary",
            range.start, range.end
        ));
    }
    tokens.push(RawToken::new(
        RawTokenKind::LexemeRun,
        lexeme,
        LexerSourceSpan {
            start: range.start,
            end: range.end,
        },
    ));
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LexicalByteRange {
    start: usize,
    end: usize,
}

fn string_argument_ranges(lexical_text: &str) -> Vec<LexicalByteRange> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while cursor < lexical_text.len() {
        if let Some(end) = string_argument_end(lexical_text, cursor) {
            ranges.push(LexicalByteRange { start: cursor, end });
            cursor = end;
            continue;
        }
        let ch = lexical_text[cursor..]
            .chars()
            .next()
            .expect("cursor is inside lexical text");
        cursor += ch.len_utf8();
    }
    ranges
}

fn string_argument_end(input: &str, start: usize) -> Option<usize> {
    let quote = input[start..].chars().next()?;
    if !matches!(quote, '"' | '\'') || !is_string_argument_start(input, start) {
        return None;
    }

    let mut escaped = false;
    for (relative, ch) in input[start + quote.len_utf8()..].char_indices() {
        if matches!(ch, '\n' | '\r') {
            return None;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some(start + quote.len_utf8() + relative + ch.len_utf8());
        }
    }

    None
}

fn is_string_argument_start(input: &str, start: usize) -> bool {
    previous_significant_char(input, start).is_some_and(|ch| matches!(ch, '(' | ','))
}

fn previous_significant_char(input: &str, end: usize) -> Option<char> {
    input[..end]
        .chars()
        .rev()
        .find(|ch| !matches!(ch, ' ' | '\t' | '\n' | '\r'))
}

fn import_stub(
    source_id: SourceId,
    bridge: &SpanBridge,
    import: LexerImportStub,
) -> Result<ImportStub, SpanBridgeError> {
    Ok(ImportStub {
        path: import_stub_path(source_id, bridge, import.path)?,
        alias: import
            .alias
            .map(|alias| import_stub_alias(source_id, bridge, alias))
            .transpose()?,
        span: lexical_source_range(source_id, bridge, import.span)?,
    })
}

fn import_stub_path(
    source_id: SourceId,
    bridge: &SpanBridge,
    path: LexerRawModulePath,
) -> Result<ImportStubPath, SpanBridgeError> {
    Ok(ImportStubPath {
        spelling: Arc::<str>::from(path.spelling),
        relative: path
            .relative
            .map(|prefix| import_relative_prefix(source_id, prefix))
            .transpose()?,
        components: path
            .components
            .into_iter()
            .map(|component| Arc::<str>::from(component.spelling))
            .collect(),
        source_segments: path
            .source_segments
            .into_iter()
            .map(|segment| lexical_source_range(source_id, bridge, segment))
            .collect::<Result<Vec<_>, SpanBridgeError>>()?,
        span: lexical_source_range(source_id, bridge, path.span)?,
    })
}

fn import_relative_prefix(
    source_id: SourceId,
    prefix: LexerRawModuleRelativePrefix,
) -> Result<ImportStubRelativePrefix, SpanBridgeError> {
    Ok(match prefix {
        LexerRawModuleRelativePrefix::Current => ImportStubRelativePrefix::Current,
        LexerRawModuleRelativePrefix::Parent => ImportStubRelativePrefix::Parent,
        _ => return Err(SpanBridgeError::UnsupportedLexerPreprocessMap { source_id }),
    })
}

fn import_stub_alias(
    source_id: SourceId,
    bridge: &SpanBridge,
    alias: LexerRawModuleAlias,
) -> Result<ImportStubAlias, SpanBridgeError> {
    Ok(ImportStubAlias {
        spelling: Arc::<str>::from(alias.spelling),
        span: lexical_source_range(source_id, bridge, alias.span)?,
    })
}

fn import_prescan_diagnostic(
    source_id: SourceId,
    bridge: &SpanBridge,
    diagnostic: LexerImportPrescanDiagnostic,
) -> Result<PreprocessDiagnostic, SpanBridgeError> {
    let mapping = lexical_mapping(source_id, bridge, diagnostic.span)?;
    Ok(PreprocessDiagnostic {
        kind: PreprocessDiagnosticKind::ImportPrescan(diagnostic.code),
        message: Arc::<str>::from(diagnostic.message),
        primary: mapping.primary,
        secondary: mapping.secondary,
    })
}

fn raw_import_scan_diagnostic(
    source_id: SourceId,
    bridge: &SpanBridge,
    diagnostic: &RawScanDiagnostic,
) -> Result<PreprocessDiagnostic, SpanBridgeError> {
    let mapping = lexical_mapping(source_id, bridge, diagnostic.span)?;
    Ok(PreprocessDiagnostic {
        kind: PreprocessDiagnosticKind::RawImportScan,
        message: Arc::<str>::from(format!(
            "raw import pre-scan recovered after raw scan error: {}",
            diagnostic.message
        )),
        primary: mapping.primary,
        secondary: mapping.secondary,
    })
}

fn raw_scan_diagnostic_message(
    code: RawScanDiagnosticCode,
    lexical_text: &str,
    start: usize,
) -> String {
    match code {
        RawScanDiagnosticCode::UnsupportedAnnotationMarker => {
            format!("unsupported annotation marker at byte {start}")
        }
        RawScanDiagnosticCode::UnsupportedInput => {
            if let Some(ch) = lexical_text
                .get(start..)
                .and_then(|text| text.chars().next())
            {
                format!("unsupported raw lexer input at byte {start}: {ch:?}")
            } else {
                format!("unsupported raw lexer input at byte {start}")
            }
        }
        _ => format!("unsupported raw lexer input at byte {start}"),
    }
}

fn lexical_source_range(
    source_id: SourceId,
    bridge: &SpanBridge,
    span: LexerSourceSpan,
) -> Result<SourceRange, SpanBridgeError> {
    Ok(lexical_mapping(source_id, bridge, span)?.primary)
}

fn lexical_mapping(
    source_id: SourceId,
    bridge: &SpanBridge,
    span: LexerSourceSpan,
) -> Result<MappedSourceRange, SpanBridgeError> {
    bridge.lexical_span(source_id, LexerByteSpan::from(span))
}

#[cfg(test)]
mod tests {
    use super::{
        CommentKind, ImportPrescanDiagnosticCode, ImportStubRelativePrefix,
        PreprocessDiagnosticKind, SourcePreprocessDiagnosticCode, lexical_hash, preprocess,
    };
    use crate::source::{SourceUnit, register_source_unit};
    use crate::span_bridge::{LexerByteSpan, SpanBridge};
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, LineMap, MappedSourceRangeKind,
        ModulePath, PackageId, SessionIdAllocator, SourceOrigin, SourceRange, hash_text,
        normalize_path,
    };
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn ordinary_comments_are_removed_from_lexical_text_and_retained_with_ranges() {
        let (source, mut bridge) = registered_source_unit("alpha :: comment\nbeta");

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(preprocessed.lexical_text.as_str(), "alpha \nbeta");
        assert_eq!(preprocessed.comments.len(), 1);
        assert_eq!(preprocessed.comments[0].kind, CommentKind::SingleLine);
        assert_eq!(
            preprocessed.comments[0].source_range,
            SourceRange {
                source_id: source.source_id,
                start: 6,
                end: 17,
            }
        );
        assert_eq!(
            &source.source_text[preprocessed.comments[0].source_range.start
                ..preprocessed.comments[0].source_range.end],
            ":: comment\n"
        );
        assert!(preprocessed.doc_comments.is_empty());
        assert!(preprocessed.diagnostics.is_empty());
    }

    #[test]
    fn doc_comments_are_preserved_as_raw_bodies_and_not_fed_to_lexical_text() {
        let (source, mut bridge) = registered_source_unit("::: doc \u{03b2}\nalpha");

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(preprocessed.lexical_text.as_str(), "\nalpha");
        assert!(preprocessed.comments.is_empty());
        assert_eq!(preprocessed.doc_comments.len(), 1);
        assert_eq!(
            preprocessed.doc_comments[0].raw_body.as_ref(),
            " doc \u{03b2}\n"
        );
        assert_eq!(
            preprocessed.doc_comments[0].source_range,
            SourceRange {
                source_id: source.source_id,
                start: 0,
                end: "::: doc \u{03b2}\n".len(),
            }
        );
        assert!(preprocessed.diagnostics.is_empty());
    }

    #[test]
    fn annotation_syntax_stays_in_lexical_text() {
        let (source, mut bridge) = registered_source_unit("@latex(\"alpha\")\n@[Lemma]\n");

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(
            preprocessed.lexical_text.as_str(),
            source.source_text.as_ref()
        );
        assert!(preprocessed.comments.is_empty());
        assert!(preprocessed.doc_comments.is_empty());
        assert!(preprocessed.diagnostics.is_empty());
    }

    #[test]
    fn import_prelude_is_prescanned_into_ordered_mapped_stubs() {
        let text = "\
import std.algebra.group, .utils as U, ..common;
import algebra.linear.{eigen_value, jordan};
definition
end;";
        let (source, mut bridge) = registered_source_unit(text);

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(
            preprocessed
                .import_stubs
                .iter()
                .map(|stub| stub.path.spelling.as_ref())
                .collect::<Vec<_>>(),
            vec![
                "std.algebra.group",
                ".utils",
                "..common",
                "algebra.linear.eigen_value",
                "algebra.linear.jordan",
            ]
        );
        assert_eq!(
            preprocessed.import_stubs[1]
                .alias
                .as_ref()
                .unwrap()
                .spelling
                .as_ref(),
            "U"
        );
        assert_eq!(
            preprocessed.import_stubs[1].path.relative,
            Some(ImportStubRelativePrefix::Current)
        );
        assert_eq!(
            preprocessed.import_stubs[2].path.relative,
            Some(ImportStubRelativePrefix::Parent)
        );
        assert_eq!(
            preprocessed.import_stubs[3].path.components,
            ["algebra", "linear", "eigen_value"]
                .into_iter()
                .map(Arc::<str>::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            preprocessed.import_stubs[1].path.span,
            source_range_of(source.source_id, text, ".utils")
        );
        assert_eq!(
            preprocessed.import_stubs[1].alias.as_ref().unwrap().span,
            source_range_of(source.source_id, text, "U")
        );
        assert_eq!(
            preprocessed.import_stubs[1].span,
            source_range_of(source.source_id, text, ".utils as U")
        );
        assert_eq!(
            preprocessed.import_stubs[3].path.source_segments,
            vec![
                source_range_of(source.source_id, text, "algebra.linear"),
                source_range_of(source.source_id, text, "eigen_value"),
            ]
        );
        assert_eq!(
            preprocessed.import_stubs[4].path.source_segments,
            vec![
                source_range_of(source.source_id, text, "algebra.linear"),
                source_range_of(source.source_id, text, "jordan"),
            ]
        );
        assert!(preprocessed.diagnostics.is_empty());
    }

    #[test]
    fn import_stub_spans_map_through_removed_doc_comment_prefix() {
        let text = "\
::: module imports
import algebra.linear.{eigen_value, jordan}, .utils as U;
definition
end;";
        let (source, mut bridge) = registered_source_unit(text);

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(
            preprocessed.lexical_text.as_str(),
            "\nimport algebra.linear.{eigen_value, jordan}, .utils as U;\ndefinition\nend;"
        );
        assert_eq!(preprocessed.import_stubs.len(), 3);
        assert_eq!(
            preprocessed
                .import_stubs
                .iter()
                .map(|stub| stub.path.spelling.as_ref())
                .collect::<Vec<_>>(),
            vec![
                "algebra.linear.eigen_value",
                "algebra.linear.jordan",
                ".utils",
            ]
        );
        assert_eq!(
            preprocessed.import_stubs[0].path.span,
            SourceRange {
                source_id: source.source_id,
                start: text.find("algebra.linear").unwrap(),
                end: text.find("eigen_value").unwrap() + "eigen_value".len(),
            }
        );
        assert_eq!(
            preprocessed.import_stubs[0].path.source_segments,
            vec![
                source_range_of(source.source_id, text, "algebra.linear"),
                source_range_of(source.source_id, text, "eigen_value"),
            ]
        );
        assert_eq!(
            preprocessed.import_stubs[1].path.source_segments,
            vec![
                source_range_of(source.source_id, text, "algebra.linear"),
                source_range_of(source.source_id, text, "jordan"),
            ]
        );
        assert_eq!(
            preprocessed.import_stubs[2].path.span,
            source_range_of(source.source_id, text, ".utils")
        );
        assert_eq!(
            preprocessed.import_stubs[2].alias.as_ref().unwrap().span,
            source_range_of(source.source_id, text, "U")
        );
        assert_eq!(
            preprocessed.import_stubs[2].span,
            source_range_of(source.source_id, text, ".utils as U")
        );
        assert!(preprocessed.diagnostics.is_empty());
    }

    #[test]
    fn malformed_imports_emit_prescan_diagnostics_without_aborting() {
        let (source, mut bridge) =
            registered_source_unit("import std., pkg.math as ;\ndefinition\nend;");

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(
            preprocessed
                .import_stubs
                .iter()
                .map(|stub| stub.path.spelling.as_ref())
                .collect::<Vec<_>>(),
            vec!["std.", "pkg.math"]
        );
        assert_eq!(
            preprocessed
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.kind)
                .collect::<Vec<_>>(),
            vec![
                PreprocessDiagnosticKind::ImportPrescan(
                    ImportPrescanDiagnosticCode::EmptyModulePathComponent
                ),
                PreprocessDiagnosticKind::ImportPrescan(ImportPrescanDiagnosticCode::MissingAlias),
            ]
        );
        assert_eq!(
            preprocessed.diagnostics[0].primary,
            SourceRange {
                source_id: source.source_id,
                start: "import std.".len(),
                end: "import std.".len(),
            }
        );
        assert_eq!(
            preprocessed.diagnostics[1].primary,
            source_range_of(source.source_id, source.source_text.as_ref(), ";")
        );
    }

    #[test]
    fn import_prescan_diagnostics_follow_preprocess_diagnostics_and_map_through_comments() {
        let text = "\
::: module imports
import std., pkg.math as ;
::=
open block";
        let (source, mut bridge) = registered_source_unit(text);

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(
            preprocessed
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.kind)
                .collect::<Vec<_>>(),
            vec![
                PreprocessDiagnosticKind::SourcePrecondition(
                    SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment
                ),
                PreprocessDiagnosticKind::ImportPrescan(
                    ImportPrescanDiagnosticCode::EmptyModulePathComponent
                ),
                PreprocessDiagnosticKind::ImportPrescan(ImportPrescanDiagnosticCode::MissingAlias),
            ]
        );
        assert_eq!(
            preprocessed.diagnostics[0].primary,
            SourceRange {
                source_id: source.source_id,
                start: text.find("::=").unwrap(),
                end: text.len(),
            }
        );
        assert_eq!(
            preprocessed.diagnostics[1].primary,
            SourceRange {
                source_id: source.source_id,
                start: text.find("import std.").unwrap() + "import std.".len(),
                end: text.find("import std.").unwrap() + "import std.".len(),
            }
        );
        assert_eq!(
            preprocessed.diagnostics[2].primary,
            source_range_of(source.source_id, text, ";")
        );
        assert_eq!(
            preprocessed
                .import_stubs
                .iter()
                .map(|stub| stub.path.spelling.as_ref())
                .collect::<Vec<_>>(),
            vec!["std.", "pkg.math"]
        );
    }

    #[test]
    fn raw_import_scan_recovery_is_precise_and_preserves_partial_imports() {
        let text = "import std.core;\n@ name\n";
        let (source, mut bridge) = registered_source_unit(text);

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(
            preprocessed
                .import_stubs
                .iter()
                .map(|stub| stub.path.spelling.as_ref())
                .collect::<Vec<_>>(),
            vec!["std.core"]
        );
        assert_eq!(preprocessed.diagnostics.len(), 1);
        assert_eq!(
            preprocessed.diagnostics[0].kind,
            PreprocessDiagnosticKind::RawImportScan
        );
        assert!(
            preprocessed.diagnostics[0]
                .message
                .starts_with("raw import pre-scan recovered after raw scan error:")
        );
        assert_eq!(
            preprocessed.diagnostics[0].primary,
            source_range_of(source.source_id, text, "@")
        );
        assert!(preprocessed.diagnostics[0].secondary.is_empty());
    }

    #[test]
    fn raw_import_scan_recovery_does_not_join_path_across_error_sentinel() {
        let text = "import std.\u{03b2}core;\n";
        let (source, mut bridge) = registered_source_unit(text);

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert!(
            preprocessed
                .import_stubs
                .iter()
                .all(|stub| stub.path.spelling.as_ref() != "std.core")
        );
        assert!(
            preprocessed
                .diagnostics
                .iter()
                .any(
                    |diagnostic| diagnostic.kind == PreprocessDiagnosticKind::RawImportScan
                        && diagnostic.primary
                            == source_range_of(source.source_id, text, "\u{03b2}")
                )
        );
    }

    #[test]
    fn raw_import_scan_recovery_rebases_diagnostic_message_after_string_argument() {
        let text = "@[label(\"β\")]\n@ name";
        let (source, mut bridge) = registered_source_unit(text);

        let preprocessed = preprocess(&source, &mut bridge).unwrap();
        let at = text
            .rfind('@')
            .expect("fixture has trailing annotation marker");
        let diagnostic = preprocessed
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.kind == PreprocessDiagnosticKind::RawImportScan)
            .expect("raw import scan recovery diagnostic exists");

        assert_eq!(
            diagnostic.primary,
            SourceRange {
                source_id: source.source_id,
                start: at,
                end: at + 1,
            }
        );
        assert!(diagnostic.message.contains(&format!("byte {at}")));
    }

    #[test]
    fn lexical_range_crossing_removed_comment_yields_composite_mapping() {
        let (source, mut bridge) = registered_source_unit("alpha ::= hidden =:: beta");

        let preprocessed = preprocess(&source, &mut bridge).unwrap();
        let mapping = preprocessed
            .source_map
            .lexical_span(&bridge, LexerByteSpan { start: 0, end: 11 })
            .unwrap();

        assert_eq!(preprocessed.lexical_text.as_str(), "alpha  beta");
        assert_eq!(mapping.kind, MappedSourceRangeKind::Composite);
        assert_eq!(
            mapping.primary,
            SourceRange {
                source_id: source.source_id,
                start: 0,
                end: source.source_text.len(),
            }
        );
        assert_eq!(mapping.secondary.len(), 3);
    }

    #[test]
    fn synthetic_whitespace_is_exposed_as_degraded_anchor_backed_mapping() {
        let (source, mut bridge) = registered_source_unit("alpha::=hidden=::beta");

        let preprocessed = preprocess(&source, &mut bridge).unwrap();
        let mapping = preprocessed
            .source_map
            .lexical_span(&bridge, LexerByteSpan { start: 5, end: 6 })
            .unwrap();

        assert_eq!(preprocessed.lexical_text.as_str(), "alpha beta");
        assert_eq!(mapping.kind, MappedSourceRangeKind::Degraded);
    }

    #[test]
    fn lexical_hash_is_stable_for_comment_only_edits_that_preserve_lexical_text() {
        let (first_source, mut first_bridge) = registered_source_unit("alpha:: one\nbeta");
        let (second_source, mut second_bridge) = registered_source_unit("alpha:: two\nbeta");

        let first = preprocess(&first_source, &mut first_bridge).unwrap();
        let second = preprocess(&second_source, &mut second_bridge).unwrap();

        assert_eq!(first.lexical_text, second.lexical_text);
        assert_eq!(first.lexical_hash, second.lexical_hash);
        assert_eq!(first.lexical_hash, lexical_hash(&first.lexical_text));
    }

    #[test]
    fn code_region_non_ascii_is_reported_and_recovered() {
        let (source, mut bridge) = registered_source_unit("alpha\u{03b2}omega");

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(preprocessed.lexical_text.as_str(), "alpha\u{03b2}omega");
        assert_eq!(preprocessed.diagnostics.len(), 2);
        assert_eq!(
            preprocessed.diagnostics[0].kind,
            PreprocessDiagnosticKind::SourcePrecondition(
                SourcePreprocessDiagnosticCode::NonAsciiCode
            )
        );
        assert_eq!(
            preprocessed.diagnostics[0].primary,
            SourceRange {
                source_id: source.source_id,
                start: "alpha".len(),
                end: "alpha\u{03b2}".len(),
            }
        );
        assert_eq!(
            preprocessed.diagnostics[1].kind,
            PreprocessDiagnosticKind::RawImportScan
        );
        assert_eq!(
            preprocessed.diagnostics[1].primary,
            SourceRange {
                source_id: source.source_id,
                start: "alpha".len(),
                end: "alpha\u{03b2}".len(),
            }
        );
    }

    #[test]
    fn unterminated_block_comment_is_reported_and_recovered() {
        let (source, mut bridge) = registered_source_unit("alpha\n::=\nopen block");

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(preprocessed.lexical_text.as_str(), "alpha\n\n");
        assert_eq!(preprocessed.comments.len(), 1);
        assert_eq!(preprocessed.comments[0].kind, CommentKind::MultiLine);
        assert_eq!(preprocessed.diagnostics.len(), 1);
        assert_eq!(
            preprocessed.diagnostics[0].kind,
            PreprocessDiagnosticKind::SourcePrecondition(
                SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment
            )
        );
        assert_eq!(
            preprocessed.diagnostics[0].primary,
            SourceRange {
                source_id: source.source_id,
                start: 6,
                end: source.source_text.len(),
            }
        );
    }

    #[test]
    fn multiple_preprocess_diagnostics_preserve_source_order_and_mapped_details() {
        let (source, mut bridge) = registered_source_unit("alpha\u{03b2}\n::=\nopen block");

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert_eq!(preprocessed.diagnostics.len(), 3);
        assert_eq!(
            preprocessed
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.kind)
                .collect::<Vec<_>>(),
            vec![
                PreprocessDiagnosticKind::SourcePrecondition(
                    SourcePreprocessDiagnosticCode::NonAsciiCode
                ),
                PreprocessDiagnosticKind::SourcePrecondition(
                    SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment
                ),
                PreprocessDiagnosticKind::RawImportScan,
            ]
        );
        assert_eq!(
            preprocessed
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.as_ref())
                .collect::<Vec<_>>(),
            vec![
                "code regions must be ASCII before lexing",
                "unterminated multi-line comment",
                "raw import pre-scan recovered after raw scan error: unsupported raw lexer input at byte 5: 'β'",
            ]
        );
        assert_eq!(
            preprocessed
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.primary)
                .collect::<Vec<_>>(),
            vec![
                SourceRange {
                    source_id: source.source_id,
                    start: "alpha".len(),
                    end: "alpha\u{03b2}".len(),
                },
                SourceRange {
                    source_id: source.source_id,
                    start: "alpha\u{03b2}\n".len(),
                    end: source.source_text.len(),
                },
                SourceRange {
                    source_id: source.source_id,
                    start: "alpha".len(),
                    end: "alpha\u{03b2}".len(),
                },
            ]
        );
        assert!(
            preprocessed
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.secondary.is_empty())
        );
    }

    #[test]
    fn preprocessing_storage_round_trips_real_outputs_with_fresh_source_ids() {
        use super::PreprocessedSource;
        let mut comment_tags = std::collections::BTreeSet::new();
        let mut map_tags = std::collections::BTreeSet::new();
        let mut relative_tags = std::collections::BTreeSet::new();
        let ids = InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(2)).unwrap();
        let fresh_id = ids.next_source_id(snapshot_id(2)).unwrap();
        for text in [
            "",
            "::: doc β\nalpha::=hidden=::beta :: tail\n",
            "import std.algebra.group, .utils as U, ..common;\nimport algebra.linear.{eigen_value, jordan};\ndefinition\nend;",
            "import ; import foo..bar; import foo as ; import foo bar;",
            "import foo",
            "import foo, §, bar;\nα\r\n::= unfinished",
            "@latex(\"β :: text\")\nalpha",
        ] {
            let (mut source, mut bridge) = registered_source_unit(text);
            assert_ne!(fresh_id, source.source_id);
            let original = preprocess(&source, &mut bridge).unwrap();
            let bytes = original.canonical_bytes().unwrap();
            let wire: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            for comment in wire[2].as_array().unwrap() {
                comment_tags.insert(comment[0].as_str().unwrap().to_owned());
            }
            for segment in wire[5].as_array().unwrap() {
                map_tags.insert(segment[0].as_str().unwrap().to_owned());
                if segment[0] == "removed_comment" {
                    comment_tags.insert(segment[2].as_str().unwrap().to_owned());
                }
            }
            for import in wire[4].as_array().unwrap() {
                if let Some(tag) = import[0][1].as_str() {
                    relative_tags.insert(tag.to_owned());
                }
            }
            assert_eq!(
                PreprocessedSource::from_canonical_bytes(&bytes, source.source_id),
                Some(original.clone()),
                "{text:?}"
            );
            source.source_id = fresh_id;
            source.line_map = LineMap::with_source(fresh_id, text);
            let mut fresh_bridge = SpanBridge::new();
            register_source_unit(&mut fresh_bridge, &source).unwrap();
            let expected = preprocess(&source, &mut fresh_bridge).unwrap();
            let restored = PreprocessedSource::from_canonical_bytes(&bytes, fresh_id).unwrap();
            assert_eq!(restored, expected, "{text:?}");
            assert_eq!(restored.canonical_bytes().unwrap(), bytes);
            if text.is_empty() {
                assert_eq!(
                    bytes,
                    br#"["mizar-frontend/preprocessed-source/v1","",[],[],[],[],[]]"#
                );
            }
        }
        assert_eq!(
            comment_tags.into_iter().collect::<Vec<_>>(),
            ["documentation", "multi_line", "single_line"]
        );
        assert_eq!(
            map_tags.into_iter().collect::<Vec<_>>(),
            ["original", "removed_comment", "synthetic_whitespace"]
        );
        assert_eq!(
            relative_tags.into_iter().collect::<Vec<_>>(),
            ["current", "parent"]
        );
    }

    #[test]
    fn preprocessing_storage_preserves_diagnostic_kinds_and_all_anchor_forms() {
        use super::{PreprocessDiagnostic, PreprocessedSource};
        use mizar_session::{GeneratedSpanAnchor, GeneratedSpanOrigin, SourceAnchor};
        let (source, mut bridge) = registered_source_unit("alpha");
        let mut original = preprocess(&source, &mut bridge).unwrap();
        let range = SourceRange {
            source_id: source.source_id,
            start: 1,
            end: 3,
        };
        let anchors = vec![
            SourceAnchor::Range(range),
            SourceAnchor::Point {
                source_id: source.source_id,
                offset: 4,
            },
            SourceAnchor::Generated(
                GeneratedSpanOrigin::new(GeneratedSpanAnchor::Range(range), " range β ").unwrap(),
            ),
            SourceAnchor::Generated(
                GeneratedSpanOrigin::new(
                    GeneratedSpanAnchor::Point {
                        source_id: source.source_id,
                        offset: 2,
                    },
                    " point ",
                )
                .unwrap(),
            ),
        ];
        let kinds = [
            PreprocessDiagnosticKind::SourcePrecondition(
                SourcePreprocessDiagnosticCode::CarriageReturn,
            ),
            PreprocessDiagnosticKind::SourcePrecondition(
                SourcePreprocessDiagnosticCode::NonAsciiCode,
            ),
            PreprocessDiagnosticKind::SourcePrecondition(
                SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment,
            ),
            PreprocessDiagnosticKind::ImportPrescan(ImportPrescanDiagnosticCode::MissingModulePath),
            PreprocessDiagnosticKind::ImportPrescan(
                ImportPrescanDiagnosticCode::EmptyModulePathComponent,
            ),
            PreprocessDiagnosticKind::ImportPrescan(ImportPrescanDiagnosticCode::MissingAlias),
            PreprocessDiagnosticKind::ImportPrescan(ImportPrescanDiagnosticCode::MissingSemicolon),
            PreprocessDiagnosticKind::ImportPrescan(ImportPrescanDiagnosticCode::UnexpectedToken),
            PreprocessDiagnosticKind::RawImportScan,
        ];
        original.diagnostics = kinds
            .into_iter()
            .map(|kind| PreprocessDiagnostic {
                kind,
                message: Arc::from("message β\n\"quoted\""),
                primary: range,
                secondary: anchors.clone(),
            })
            .collect();
        let bytes = original.canonical_bytes().unwrap();
        let ids = InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(2)).unwrap();
        let fresh_id = ids.next_source_id(snapshot_id(2)).unwrap();
        assert_ne!(fresh_id, source.source_id);
        let restored = PreprocessedSource::from_canonical_bytes(&bytes, fresh_id).unwrap();
        let mut expected = original.clone();
        expected.source_id = fresh_id;
        expected.source_map.source_id = fresh_id;
        for diagnostic in &mut expected.diagnostics {
            diagnostic.primary.source_id = fresh_id;
            for anchor in &mut diagnostic.secondary {
                match anchor {
                    SourceAnchor::Range(range) => range.source_id = fresh_id,
                    SourceAnchor::Point { source_id, .. } => *source_id = fresh_id,
                    SourceAnchor::Generated(origin) => {
                        let rebound = match origin.anchor() {
                            GeneratedSpanAnchor::Range(mut range) => {
                                range.source_id = fresh_id;
                                GeneratedSpanAnchor::Range(range)
                            }
                            GeneratedSpanAnchor::Point { offset, .. } => {
                                GeneratedSpanAnchor::Point {
                                    source_id: fresh_id,
                                    offset,
                                }
                            }
                            _ => unreachable!(),
                        };
                        *origin = GeneratedSpanOrigin::new(rebound, origin.reason()).unwrap();
                    }
                    _ => unreachable!(),
                }
            }
        }
        assert_eq!(restored, expected);
        assert_eq!(restored.canonical_bytes().unwrap(), bytes);
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            value[6]
                .as_array()
                .unwrap()
                .iter()
                .map(|diagnostic| diagnostic[0].as_str().unwrap())
                .collect::<Vec<_>>(),
            vec![
                "source.carriage_return",
                "source.non_ascii_code",
                "source.unterminated_multi_line_comment",
                "import.missing_module_path",
                "import.empty_module_path_component",
                "import.missing_alias",
                "import.missing_semicolon",
                "import.unexpected_token",
                "raw_import_scan",
            ]
        );
        assert_eq!(
            value[6][0][3],
            serde_json::json!([
                ["range", [1, 3]],
                ["point", 4],
                ["generated", ["range", [1, 3]], " range β "],
                ["generated", ["point", 2], " point "]
            ])
        );
        let mut foreign_primary = expected.clone();
        foreign_primary.diagnostics[0].primary.source_id = source.source_id;
        assert!(foreign_primary.canonical_bytes().is_none());
        for (pointer, replacement) in [
            ("/6/0/0", serde_json::json!("unknown")),
            ("/6/0/3/0/0", serde_json::json!("unknown")),
            ("/6/0/3/1/1", serde_json::json!(-1)),
            (
                "/6/0/3/2/1",
                serde_json::json!(["generated", ["point", 0], "nested"]),
            ),
            ("/6/0/3/2/2", serde_json::json!(" \t\n")),
            ("/6/0/3/3/1/1", serde_json::json!(1.5)),
        ] {
            let mut invalid = value.clone();
            *invalid.pointer_mut(pointer).unwrap() = replacement;
            assert!(
                PreprocessedSource::from_canonical_bytes(
                    &serde_json::to_vec(&invalid).unwrap(),
                    fresh_id
                )
                .is_none(),
                "{pointer}"
            );
        }
        // Each anchor variant must reject an old identity during encoding.
        for (index, anchor) in anchors.iter().enumerate() {
            let mut invalid = expected.clone();
            invalid.diagnostics[0].secondary[index] = anchor.clone();
            assert!(invalid.canonical_bytes().is_none());
        }
    }

    #[test]
    fn preprocessing_storage_rejects_structural_and_coordinate_corruption() {
        use super::PreprocessedSource;
        use serde_json::{Value, json};
        let (source, mut bridge) =
            registered_source_unit("::: β\nimport .foo as F;\nalpha::=hidden=::beta :: tail\n");
        let original = preprocess(&source, &mut bridge).unwrap();
        let bytes = original.canonical_bytes().unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        let reject = |bytes: &[u8]| {
            assert!(PreprocessedSource::from_canonical_bytes(bytes, source.source_id).is_none())
        };
        for invalid in [
            b"".as_slice(),
            b"null",
            b"{}",
            b"\xff",
            &bytes[..bytes.len() - 1],
        ] {
            reject(invalid);
        }
        let mut spaced = bytes.clone();
        spaced.push(b' ');
        reject(&spaced);
        reject(&serde_json::to_vec_pretty(&value).unwrap());
        for (pointer, replacement) in [
            ("/0", json!("unknown")),
            ("/1", Value::Null),
            ("/2/0/0", json!("unknown")),
            ("/2/0/1", json!([2, 1])),
            ("/3/0/0/0", json!(-1)),
            ("/3/0/0/1", json!(1.5)),
            ("/4/0/0/1", json!("unknown")),
            ("/4/0/0/2/0", json!(7)),
            ("/4/0/0/3/0", json!([4, 3])),
            ("/4/0/1", json!(false)),
            ("/5/0/0", json!("unknown")),
        ] {
            let mut invalid = value.clone();
            *invalid.pointer_mut(pointer).unwrap() = replacement;
            reject(&serde_json::to_vec(&invalid).unwrap());
        }
        // Every non-list record rejects extra fields, including nested ranges.
        for pointer in [
            "", "/2/0", "/2/0/1", "/3/0", "/4/0", "/4/0/0", "/4/0/1", "/5/0",
        ] {
            let mut invalid = value.clone();
            invalid
                .pointer_mut(pointer)
                .unwrap()
                .as_array_mut()
                .unwrap()
                .push(Value::Null);
            reject(&serde_json::to_vec(&invalid).unwrap());
        }
        // All map variants have distinct ranges and tags; exercise each directly.
        for map in [
            json!(["original", [0, 9999], [0, 1]]),
            json!(["original", [1, 0], [0, 1]]),
            json!(["original", [0, 1], [2, 1]]),
            json!(["removed_comment", [2, 1], "single_line"]),
            json!(["removed_comment", [0, 1], "unknown"]),
            json!(["synthetic_whitespace", [0, 9999], [0, 1]]),
            json!(["synthetic_whitespace", [0, 1], [2, 1]]),
        ] {
            let mut invalid = value.clone();
            invalid[5] = json!([map]);
            reject(&serde_json::to_vec(&invalid).unwrap());
        }
        let mut unicode = value.clone();
        unicode[1] = json!("β");
        unicode[5] = json!([["original", [0, 1], [0, 1]]]);
        reject(&serde_json::to_vec(&unicode).unwrap());
        let ids = InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(2)).unwrap();
        let foreign_id = ids.next_source_id(snapshot_id(2)).unwrap();
        assert_ne!(foreign_id, source.source_id);
        for field in 0..9 {
            let mut invalid = original.clone();
            match field {
                0 => invalid.source_map.source_id = foreign_id,
                1 => invalid.comments[0].source_range.source_id = foreign_id,
                2 => invalid.doc_comments[0].source_range.source_id = foreign_id,
                3 => invalid.import_stubs[0].span.source_id = foreign_id,
                4 => invalid.import_stubs[0].path.span.source_id = foreign_id,
                5 => invalid.import_stubs[0].path.source_segments[0].source_id = foreign_id,
                6 => {
                    invalid.import_stubs[0]
                        .alias
                        .as_mut()
                        .unwrap()
                        .span
                        .source_id = foreign_id
                }
                7 => invalid.lexical_hash = Hash::from_bytes([0; Hash::BYTE_LEN]),
                8 => invalid.source_map.lexical_text_len += 1,
                _ => unreachable!(),
            }
            assert!(invalid.canonical_bytes().is_none(), "field {field}");
        }
    }

    #[test]
    fn preprocessing_storage_enforces_payload_limit() {
        use super::PreprocessedSource;
        let (source, mut bridge) = registered_source_unit("");
        let mut original = preprocess(&source, &mut bridge).unwrap();
        let overhead = original.canonical_bytes().unwrap().len();
        original.lexical_text.text = Arc::from("a".repeat(16 * 1024 * 1024 - overhead));
        original.lexical_hash = lexical_hash(&original.lexical_text);
        original.source_map.lexical_text_len = original.lexical_text.text.len();
        let bytes = original.canonical_bytes().unwrap();
        assert_eq!(bytes.len(), 16 * 1024 * 1024);
        assert_eq!(
            PreprocessedSource::from_canonical_bytes(&bytes, source.source_id),
            Some(original.clone())
        );
        let mut oversized = bytes;
        // Add a byte inside lexical text, preserving canonical JSON.
        let text_start = br#"["mizar-frontend/preprocessed-source/v1",""#.len();
        oversized.insert(text_start, b'a');
        assert!(PreprocessedSource::from_canonical_bytes(&oversized, source.source_id).is_none());
        original.lexical_text.text = Arc::from(format!("{}a", original.lexical_text.as_str()));
        original.lexical_hash = lexical_hash(&original.lexical_text);
        original.source_map.lexical_text_len += 1;
        assert!(original.canonical_bytes().is_none());
    }

    fn source_range_of(
        source_id: mizar_session::SourceId,
        haystack: &str,
        needle: &str,
    ) -> SourceRange {
        let start = haystack.find(needle).expect("test fixture contains needle");
        SourceRange {
            source_id,
            start,
            end: start + needle.len(),
        }
    }

    fn registered_source_unit(text: &str) -> (SourceUnit, SpanBridge) {
        let source = source_unit(text);
        let mut bridge = SpanBridge::new();
        register_source_unit(&mut bridge, &source).unwrap();
        (source, bridge)
    }

    fn source_unit(text: &str) -> SourceUnit {
        let package = PackageFixture::new();
        package.write("src/test/basic.miz", text);
        let source_id = InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(1))
            .unwrap();
        SourceUnit {
            source_id,
            package_id: PackageId::new("mml"),
            module_path: ModulePath::new("test.basic"),
            normalized_path: normalize_path(package.root(), &package.path("src/test/basic.miz"))
                .unwrap(),
            edition: Edition::new("2026"),
            file_path: package.path("src/test/basic.miz"),
            source_text: Arc::from(text),
            source_hash: hash_text(text),
            line_map: LineMap::with_source(source_id, text),
            loading_map: None,
            origin: SourceOrigin::Disk,
            generated_anchor: None,
        }
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }

    struct PackageFixture {
        root: PathBuf,
    }

    impl PackageFixture {
        fn new() -> Self {
            let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "mizar-frontend-preprocess-test-{}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(&root).unwrap();
            Self { root }
        }

        fn root(&self) -> &Path {
            &self.root
        }

        fn path(&self, relative: &str) -> PathBuf {
            self.root.join(relative)
        }

        fn write(&self, relative: &str, text: &str) {
            let path = self.path(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
    }

    impl Drop for PackageFixture {
        fn drop(&mut self) {
            match fs::remove_dir_all(&self.root) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => panic!(
                    "failed to remove temporary package `{}`: {error}",
                    self.root.display()
                ),
            }
        }
    }
}
