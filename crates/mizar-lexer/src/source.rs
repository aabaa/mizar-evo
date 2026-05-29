use crate::raw_lexer::{is_identifier, is_layout};
use std::error::Error;
use std::fmt;
use std::str::Utf8Error;

const UTF8_BOM: &str = "\u{feff}";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

pub type SourcePos = usize;
pub type SourceRange = SourceSpan;

impl SourceSpan {
    pub const fn new(start: SourcePos, end: SourcePos) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    pub const fn try_new(start: SourcePos, end: SourcePos) -> Option<Self> {
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

    pub const fn is_valid(self) -> bool {
        self.start <= self.end
    }

    pub const fn contains(self, offset: SourcePos) -> bool {
        self.start <= offset && offset < self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLineIndex {
    line_starts: Vec<usize>,
    char_boundaries: Vec<usize>,
    source_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSourceText {
    pub text: String,
    pub loading_map: Option<SourceLoadingMap>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLoadingMap {
    pub segments: Vec<SourceLoadingMapSegment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceLoadingMapSegment {
    Original {
        loaded: SourceRange,
        original: SourceRange,
    },
    NormalizedNewline {
        loaded: SourceRange,
        original: SourceRange,
    },
    RemovedLeadingBom {
        original: SourceRange,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocationRange {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessedLexicalSource {
    pub lexical_text: String,
    pub comments: Vec<CommentTrivia>,
    pub diagnostics: Vec<SourcePreprocessDiagnostic>,
    pub preprocess_map: SourcePreprocessMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePreprocessMap {
    pub segments: Vec<SourcePreprocessMapSegment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePreprocessMapSegment {
    Original {
        lexical: SourceRange,
        source: SourceRange,
    },
    RemovedComment {
        source: SourceRange,
        kind: CommentKind,
    },
    SyntheticWhitespace {
        lexical: SourceRange,
        anchor: SourceRange,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentTrivia {
    pub kind: CommentKind,
    pub lexeme: String,
    pub span: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommentKind {
    SingleLine,
    MultiLine,
    Documentation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePreprocessDiagnostic {
    pub code: SourcePreprocessDiagnosticCode,
    pub message: String,
    pub span: SourceRange,
    pub payload: SourcePreprocessDiagnosticPayload,
}

impl SourcePreprocessDiagnostic {
    pub fn new(
        code: SourcePreprocessDiagnosticCode,
        message: impl Into<String>,
        span: SourceRange,
    ) -> Self {
        Self::with_payload(code, message, span, SourcePreprocessDiagnosticPayload::None)
    }

    pub fn with_payload(
        code: SourcePreprocessDiagnosticCode,
        message: impl Into<String>,
        span: SourceRange,
        payload: SourcePreprocessDiagnosticPayload,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            span,
            payload,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePreprocessDiagnosticCode {
    CarriageReturn,
    NonAsciiCode,
    UnterminatedMultiLineComment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePreprocessDiagnosticPayload {
    None,
    CarriageReturn {
        recovery: SourcePreprocessRecoveryHint,
    },
    NonAsciiCode {
        character: char,
        utf8_len: usize,
    },
    UnterminatedMultiLineComment {
        opener: SourceRange,
        recovery: SourcePreprocessRecoveryHint,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePreprocessRecoveryHint {
    NormalizeCrLfBeforeLexerEntry,
    PreserveNewlinesAndDropCommentText,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSourceName {
    pub file_name: String,
    pub module_name: String,
    pub namespace_components: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModuleNamingError {
    MissingMizExtension { path: String },
    MissingFileStem { path: String },
    InvalidPackageName { package_name: String },
    MissingSourceRoot { path: String },
    PackageRootMismatch { package_name: String, root: String },
    InvalidNamespaceComponent { component: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceLoadError {
    InvalidUtf8 {
        valid_up_to: usize,
        error_len: Option<usize>,
    },
}

impl fmt::Display for ModuleNamingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMizExtension { path } => {
                write!(f, "module source path `{path}` must end with `.miz`")
            }
            Self::MissingFileStem { path } => {
                write!(f, "module source path `{path}` must include a file name")
            }
            Self::InvalidPackageName { package_name } => {
                write!(f, "invalid package namespace root `{package_name}`")
            }
            Self::MissingSourceRoot { path } => {
                write!(f, "module source path `{path}` must be under a `src` root")
            }
            Self::PackageRootMismatch { package_name, root } => {
                write!(
                    f,
                    "module source package root `{root}` must match package `{package_name}`"
                )
            }
            Self::InvalidNamespaceComponent { component } => {
                write!(f, "invalid namespace component `{component}`")
            }
        }
    }
}

impl Error for ModuleNamingError {}

impl fmt::Display for SourceLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8 {
                valid_up_to,
                error_len,
            } => match error_len {
                Some(error_len) => write!(
                    f,
                    "source bytes are not valid UTF-8 at byte {valid_up_to} over {error_len} byte(s)"
                ),
                None => write!(
                    f,
                    "source bytes end with an incomplete UTF-8 sequence starting at byte {valid_up_to}"
                ),
            },
        }
    }
}

impl Error for SourceLoadError {}

impl From<Utf8Error> for SourceLoadError {
    fn from(error: Utf8Error) -> Self {
        Self::InvalidUtf8 {
            valid_up_to: error.valid_up_to(),
            error_len: error.error_len(),
        }
    }
}

impl SourceLoadingMap {
    pub fn original_offset_for_loaded(&self, offset: SourcePos) -> Option<SourcePos> {
        self.segments.iter().find_map(|segment| match segment {
            SourceLoadingMapSegment::Original { loaded, original }
                if loaded.start <= offset && offset <= loaded.end =>
            {
                Some(original.start + (offset - loaded.start))
            }
            SourceLoadingMapSegment::NormalizedNewline { loaded, original }
                if loaded.start <= offset && offset <= loaded.end =>
            {
                Some(if offset == loaded.start {
                    original.start
                } else {
                    original.end
                })
            }
            _ => None,
        })
    }
}

impl SourcePreprocessMap {
    pub fn source_ranges_for_lexical(&self, lexical: SourceRange) -> Option<Vec<SourceRange>> {
        if lexical.start > lexical.end {
            return None;
        }
        let source_len = self.lexical_len();
        if lexical.end > source_len {
            return None;
        }
        if lexical.start == lexical.end {
            return self.source_insertion_point_for_lexical(lexical.start);
        }

        let mut ranges = Vec::new();
        for (index, segment) in self.segments.iter().enumerate() {
            match segment {
                SourcePreprocessMapSegment::Original {
                    lexical: segment_lexical,
                    source,
                } => {
                    let Some(intersection) = intersect_ranges(lexical, *segment_lexical) else {
                        continue;
                    };
                    push_mapped_source_range(
                        &mut ranges,
                        SourceSpan {
                            start: source.start + (intersection.start - segment_lexical.start),
                            end: source.start + (intersection.end - segment_lexical.start),
                        },
                    );
                }
                SourcePreprocessMapSegment::RemovedComment { source, .. } => {
                    let Some(anchor) = self.lexical_anchor_for_removed_comment(index) else {
                        continue;
                    };
                    if lexical.start < anchor && anchor < lexical.end {
                        push_mapped_source_range(&mut ranges, *source);
                    }
                }
                SourcePreprocessMapSegment::SyntheticWhitespace {
                    lexical: segment_lexical,
                    anchor,
                } => {
                    if intersect_ranges(lexical, *segment_lexical).is_some() {
                        push_mapped_source_range(&mut ranges, *anchor);
                    }
                }
            }
        }

        if ranges.is_empty() && lexical.start != lexical.end {
            None
        } else {
            Some(ranges)
        }
    }

    fn lexical_len(&self) -> SourcePos {
        self.segments
            .iter()
            .filter_map(preprocess_segment_lexical_range)
            .map(|range| range.end)
            .max()
            .unwrap_or(0)
    }

    fn source_insertion_point_for_lexical(&self, offset: SourcePos) -> Option<Vec<SourceRange>> {
        let mut ranges = Vec::new();
        for (index, segment) in self.segments.iter().enumerate() {
            match segment {
                SourcePreprocessMapSegment::Original { lexical, source }
                    if lexical.start <= offset && offset <= lexical.end =>
                {
                    let source_offset = source.start + (offset - lexical.start);
                    push_insertion_source_anchor(
                        &mut ranges,
                        SourceSpan {
                            start: source_offset,
                            end: source_offset,
                        },
                    );
                }
                SourcePreprocessMapSegment::SyntheticWhitespace { lexical, anchor }
                    if lexical.start <= offset && offset <= lexical.end =>
                {
                    push_insertion_source_anchor(&mut ranges, *anchor);
                }
                SourcePreprocessMapSegment::RemovedComment { source, .. }
                    if self.lexical_anchor_for_removed_comment(index) == Some(offset) =>
                {
                    push_insertion_source_anchor(&mut ranges, *source);
                }
                _ => {}
            }
        }
        if ranges.is_empty() && offset == 0 {
            push_insertion_source_anchor(&mut ranges, SourceSpan { start: 0, end: 0 });
        }
        (!ranges.is_empty()).then_some(ranges)
    }

    fn lexical_anchor_for_removed_comment(&self, index: usize) -> Option<SourcePos> {
        let previous = self.segments[..index]
            .iter()
            .rev()
            .find_map(preprocess_segment_lexical_range)
            .map(|range| range.end);
        let next = self.segments[index + 1..]
            .iter()
            .find_map(preprocess_segment_lexical_range)
            .map(|range| range.start);
        next.or(previous)
    }
}

pub fn load_source_text_from_bytes(bytes: &[u8]) -> Result<LoadedSourceText, SourceLoadError> {
    let text = std::str::from_utf8(bytes)?;
    let (source_text, original_base, mut segments) =
        if let Some(stripped) = text.strip_prefix(UTF8_BOM) {
            (
                stripped,
                UTF8_BOM.len(),
                vec![SourceLoadingMapSegment::RemovedLeadingBom {
                    original: SourceSpan { start: 0, end: 3 },
                }],
            )
        } else {
            (text, 0, Vec::new())
        };

    if !source_text.contains("\r\n") {
        if !segments.is_empty() {
            segments.push(SourceLoadingMapSegment::Original {
                loaded: SourceSpan {
                    start: 0,
                    end: source_text.len(),
                },
                original: SourceSpan {
                    start: original_base,
                    end: original_base + source_text.len(),
                },
            });
            return Ok(LoadedSourceText {
                text: source_text.to_owned(),
                loading_map: Some(SourceLoadingMap { segments }),
            });
        }

        return Ok(LoadedSourceText {
            text: text.to_owned(),
            loading_map: None,
        });
    }

    let normalized = normalize_crlf_to_lf(source_text, original_base, &mut segments);
    Ok(LoadedSourceText {
        text: normalized,
        loading_map: Some(SourceLoadingMap { segments }),
    })
}

fn normalize_crlf_to_lf(
    source_text: &str,
    original_base: usize,
    segments: &mut Vec<SourceLoadingMapSegment>,
) -> String {
    let mut normalized = String::with_capacity(source_text.len());
    let mut cursor = 0;
    let mut next_crlf = source_text.find("\r\n");

    while let Some(crlf_start) = next_crlf {
        normalized.push_str(&source_text[cursor..crlf_start]);
        if cursor < crlf_start {
            segments.push(SourceLoadingMapSegment::Original {
                loaded: SourceSpan {
                    start: normalized.len() - (crlf_start - cursor),
                    end: normalized.len(),
                },
                original: SourceSpan {
                    start: original_base + cursor,
                    end: original_base + crlf_start,
                },
            });
        }

        let loaded_start = normalized.len();
        normalized.push('\n');
        segments.push(SourceLoadingMapSegment::NormalizedNewline {
            loaded: SourceSpan {
                start: loaded_start,
                end: loaded_start + 1,
            },
            original: SourceSpan {
                start: original_base + crlf_start,
                end: original_base + crlf_start + 2,
            },
        });

        cursor = crlf_start + 2;
        next_crlf = source_text[cursor..]
            .find("\r\n")
            .map(|relative| cursor + relative);
    }

    normalized.push_str(&source_text[cursor..]);
    if cursor < source_text.len() {
        segments.push(SourceLoadingMapSegment::Original {
            loaded: SourceSpan {
                start: normalized.len() - (source_text.len() - cursor),
                end: normalized.len(),
            },
            original: SourceSpan {
                start: original_base + cursor,
                end: original_base + source_text.len(),
            },
        });
    }

    normalized
}

impl SourceLineIndex {
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
            line_starts,
            char_boundaries,
            source_len: source.len(),
        }
    }

    pub fn location(&self, offset: usize) -> Option<SourceLocation> {
        if offset > self.source_len || self.char_boundaries.binary_search(&offset).is_err() {
            return None;
        }
        let line = match self.line_starts.binary_search(&offset) {
            Ok(line) => line,
            Err(0) => return None,
            Err(next_line) => next_line - 1,
        };
        Some(SourceLocation {
            line,
            column: offset - self.line_starts[line],
        })
    }

    pub fn range(&self, span: SourceSpan) -> Option<SourceLocationRange> {
        if span.start > span.end {
            return None;
        }
        Some(SourceLocationRange {
            start: self.location(span.start)?,
            end: self.location(span.end)?,
        })
    }
}

pub fn preprocess_source_for_lexing(input: &str) -> PreprocessedLexicalSource {
    let mut lexical_text = String::with_capacity(input.len());
    let mut comments = Vec::new();
    let mut diagnostics = Vec::new();
    let mut map_segments = Vec::new();
    let mut cursor = 0;

    while cursor < input.len() {
        let rest = &input[cursor..];
        if rest.starts_with(":::") {
            let end = line_comment_end(input, cursor);
            let span = SourceSpan { start: cursor, end };
            comments.push(CommentTrivia {
                kind: CommentKind::Documentation,
                lexeme: input[cursor..end].to_owned(),
                span,
            });
            map_segments.push(SourcePreprocessMapSegment::RemovedComment {
                source: span,
                kind: CommentKind::Documentation,
            });
            preserve_comment_replacement(input, cursor, end, &mut lexical_text, &mut map_segments);
            cursor = end;
            continue;
        }

        if rest.starts_with("::=") {
            let end = match rest.find("=::") {
                Some(relative) => cursor + relative + "=::".len(),
                None => {
                    let span = SourceSpan {
                        start: cursor,
                        end: input.len(),
                    };
                    diagnostics.push(SourcePreprocessDiagnostic::with_payload(
                        SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment,
                        "unterminated multi-line comment",
                        span,
                        SourcePreprocessDiagnosticPayload::UnterminatedMultiLineComment {
                            opener: SourceSpan {
                                start: cursor,
                                end: cursor + "::=".len(),
                            },
                            recovery:
                                SourcePreprocessRecoveryHint::PreserveNewlinesAndDropCommentText,
                        },
                    ));
                    input.len()
                }
            };
            let span = SourceSpan { start: cursor, end };
            comments.push(CommentTrivia {
                kind: CommentKind::MultiLine,
                lexeme: input[cursor..end].to_owned(),
                span,
            });
            map_segments.push(SourcePreprocessMapSegment::RemovedComment {
                source: span,
                kind: CommentKind::MultiLine,
            });
            preserve_comment_replacement(input, cursor, end, &mut lexical_text, &mut map_segments);
            cursor = end;
            continue;
        }

        if rest.starts_with("::") {
            let end = line_comment_end(input, cursor);
            let span = SourceSpan { start: cursor, end };
            comments.push(CommentTrivia {
                kind: CommentKind::SingleLine,
                lexeme: input[cursor..end].to_owned(),
                span,
            });
            map_segments.push(SourcePreprocessMapSegment::RemovedComment {
                source: span,
                kind: CommentKind::SingleLine,
            });
            preserve_comment_replacement(input, cursor, end, &mut lexical_text, &mut map_segments);
            cursor = end;
            continue;
        }

        let ch = rest.chars().next().expect("cursor is inside source");
        let end = cursor + ch.len_utf8();
        if ch == '\r' {
            diagnostics.push(SourcePreprocessDiagnostic::with_payload(
                SourcePreprocessDiagnosticCode::CarriageReturn,
                "source text must be LF-only before lexing",
                SourceSpan { start: cursor, end },
                SourcePreprocessDiagnosticPayload::CarriageReturn {
                    recovery: SourcePreprocessRecoveryHint::NormalizeCrLfBeforeLexerEntry,
                },
            ));
        } else if !ch.is_ascii() {
            diagnostics.push(SourcePreprocessDiagnostic::with_payload(
                SourcePreprocessDiagnosticCode::NonAsciiCode,
                "code regions must be ASCII before lexing",
                SourceSpan { start: cursor, end },
                SourcePreprocessDiagnosticPayload::NonAsciiCode {
                    character: ch,
                    utf8_len: ch.len_utf8(),
                },
            ));
        }
        lexical_text.push(ch);
        push_original_preprocess_segment(
            &mut map_segments,
            SourceSpan {
                start: lexical_text.len() - ch.len_utf8(),
                end: lexical_text.len(),
            },
            SourceSpan { start: cursor, end },
        );
        cursor = end;
    }

    PreprocessedLexicalSource {
        lexical_text,
        comments,
        diagnostics,
        preprocess_map: SourcePreprocessMap {
            segments: map_segments,
        },
    }
}

pub fn module_source_name_from_path(
    package_name: &str,
    path: &str,
) -> Result<ModuleSourceName, ModuleNamingError> {
    if !is_identifier(package_name) {
        return Err(ModuleNamingError::InvalidPackageName {
            package_name: package_name.to_owned(),
        });
    }

    let normalized = path.replace('\\', "/");
    let Some(file_name) = normalized
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
    else {
        return Err(ModuleNamingError::MissingFileStem {
            path: path.to_owned(),
        });
    };
    let Some(module_name) = file_name
        .strip_suffix(".miz")
        .filter(|stem| !stem.is_empty())
    else {
        return Err(ModuleNamingError::MissingMizExtension {
            path: path.to_owned(),
        });
    };

    let path_without_extension = normalized
        .strip_suffix(".miz")
        .expect("extension was just validated");
    let Some((package_root, source_relative)) = path_without_extension.split_once("/src/") else {
        return Err(ModuleNamingError::MissingSourceRoot {
            path: path.to_owned(),
        });
    };
    let root_name = package_root.rsplit('/').next().unwrap_or(package_root);
    if root_name != package_name {
        return Err(ModuleNamingError::PackageRootMismatch {
            package_name: package_name.to_owned(),
            root: root_name.to_owned(),
        });
    }

    let mut namespace_components = Vec::new();
    namespace_components.push(package_name.to_owned());
    namespace_components.extend(
        source_relative
            .split('/')
            .map(str::to_owned)
            .collect::<Vec<_>>(),
    );

    for component in &namespace_components {
        if !is_identifier(component) {
            return Err(ModuleNamingError::InvalidNamespaceComponent {
                component: component.clone(),
            });
        }
    }

    Ok(ModuleSourceName {
        file_name: file_name.to_owned(),
        module_name: module_name.to_owned(),
        namespace_components,
    })
}

fn line_comment_end(input: &str, start: usize) -> usize {
    input[start..]
        .find('\n')
        .map_or(input.len(), |relative| start + relative + '\n'.len_utf8())
}

fn preserve_comment_replacement(
    input: &str,
    start: usize,
    end: usize,
    output: &mut String,
    map_segments: &mut Vec<SourcePreprocessMapSegment>,
) {
    let comment = &input[start..end];
    let had_newline = comment.contains('\n');
    preserve_comment_newlines(comment, start, output, map_segments);
    if !had_newline && comment_removal_would_concatenate_tokens(input, end, output) {
        let lexical_start = output.len();
        output.push(' ');
        map_segments.push(SourcePreprocessMapSegment::SyntheticWhitespace {
            lexical: SourceSpan {
                start: lexical_start,
                end: lexical_start + 1,
            },
            anchor: SourceSpan { start, end },
        });
    }
}

fn comment_removal_would_concatenate_tokens(input: &str, end: usize, output: &str) -> bool {
    let Some(previous) = output.chars().next_back() else {
        return false;
    };
    let Some(next) = input[end..].chars().next() else {
        return false;
    };
    !is_layout(previous) && !is_layout(next)
}

fn preserve_comment_newlines(
    comment: &str,
    comment_start: usize,
    output: &mut String,
    map_segments: &mut Vec<SourcePreprocessMapSegment>,
) {
    for (relative, ch) in comment.char_indices() {
        if ch == '\n' {
            let lexical_start = output.len();
            output.push('\n');
            map_segments.push(SourcePreprocessMapSegment::SyntheticWhitespace {
                lexical: SourceSpan {
                    start: lexical_start,
                    end: lexical_start + 1,
                },
                anchor: SourceSpan {
                    start: comment_start + relative,
                    end: comment_start + relative + '\n'.len_utf8(),
                },
            });
        }
    }
}

fn push_original_preprocess_segment(
    segments: &mut Vec<SourcePreprocessMapSegment>,
    lexical: SourceRange,
    source: SourceRange,
) {
    if let Some(SourcePreprocessMapSegment::Original {
        lexical: previous_lexical,
        source: previous_source,
    }) = segments.last_mut()
        && previous_lexical.end == lexical.start
        && previous_source.end == source.start
    {
        previous_lexical.end = lexical.end;
        previous_source.end = source.end;
        return;
    }

    segments.push(SourcePreprocessMapSegment::Original { lexical, source });
}

fn preprocess_segment_lexical_range(segment: &SourcePreprocessMapSegment) -> Option<SourceRange> {
    match segment {
        SourcePreprocessMapSegment::Original { lexical, .. }
        | SourcePreprocessMapSegment::SyntheticWhitespace { lexical, .. } => Some(*lexical),
        SourcePreprocessMapSegment::RemovedComment { .. } => None,
    }
}

fn push_mapped_source_range(ranges: &mut Vec<SourceRange>, range: SourceRange) {
    if ranges
        .iter()
        .any(|existing| existing.start <= range.start && range.end <= existing.end)
    {
        return;
    }
    ranges.retain(|existing| !(range.start <= existing.start && existing.end <= range.end));
    if ranges.last() != Some(&range) {
        ranges.push(range);
    }
}

fn push_insertion_source_anchor(ranges: &mut Vec<SourceRange>, range: SourceRange) {
    if ranges.last() != Some(&range) {
        ranges.push(range);
    }
}

fn intersect_ranges(left: SourceRange, right: SourceRange) -> Option<SourceRange> {
    let start = left.start.max(right.start);
    let end = left.end.min(right.end);
    (start < end).then_some(SourceSpan { start, end })
}
