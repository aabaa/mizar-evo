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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourcePreprocessDiagnosticCode {
    CarriageReturn,
    NonAsciiCode,
    UnterminatedMultiLineComment,
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
    let mut cursor = 0;

    while cursor < input.len() {
        let rest = &input[cursor..];
        if rest.starts_with(":::") {
            let end = line_comment_end(input, cursor);
            comments.push(CommentTrivia {
                kind: CommentKind::Documentation,
                lexeme: input[cursor..end].to_owned(),
                span: SourceSpan { start: cursor, end },
            });
            preserve_comment_replacement(input, cursor, end, &mut lexical_text);
            cursor = end;
            continue;
        }

        if rest.starts_with("::=") {
            let end = match rest.find("=::") {
                Some(relative) => cursor + relative + "=::".len(),
                None => {
                    diagnostics.push(SourcePreprocessDiagnostic {
                        code: SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment,
                        message: "unterminated multi-line comment".to_owned(),
                        span: SourceSpan {
                            start: cursor,
                            end: input.len(),
                        },
                    });
                    input.len()
                }
            };
            comments.push(CommentTrivia {
                kind: CommentKind::MultiLine,
                lexeme: input[cursor..end].to_owned(),
                span: SourceSpan { start: cursor, end },
            });
            preserve_comment_replacement(input, cursor, end, &mut lexical_text);
            cursor = end;
            continue;
        }

        if rest.starts_with("::") {
            let end = line_comment_end(input, cursor);
            comments.push(CommentTrivia {
                kind: CommentKind::SingleLine,
                lexeme: input[cursor..end].to_owned(),
                span: SourceSpan { start: cursor, end },
            });
            preserve_comment_replacement(input, cursor, end, &mut lexical_text);
            cursor = end;
            continue;
        }

        let ch = rest.chars().next().expect("cursor is inside source");
        let end = cursor + ch.len_utf8();
        if ch == '\r' {
            diagnostics.push(SourcePreprocessDiagnostic {
                code: SourcePreprocessDiagnosticCode::CarriageReturn,
                message: "source text must be LF-only before lexing".to_owned(),
                span: SourceSpan { start: cursor, end },
            });
        } else if !ch.is_ascii() {
            diagnostics.push(SourcePreprocessDiagnostic {
                code: SourcePreprocessDiagnosticCode::NonAsciiCode,
                message: "code regions must be ASCII before lexing".to_owned(),
                span: SourceSpan { start: cursor, end },
            });
        }
        lexical_text.push(ch);
        cursor = end;
    }

    PreprocessedLexicalSource {
        lexical_text,
        comments,
        diagnostics,
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

fn preserve_comment_replacement(input: &str, start: usize, end: usize, output: &mut String) {
    let comment = &input[start..end];
    let had_newline = comment.contains('\n');
    preserve_comment_newlines(comment, output);
    if !had_newline && comment_removal_would_concatenate_tokens(input, end, output) {
        output.push(' ');
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

fn preserve_comment_newlines(comment: &str, output: &mut String) {
    for ch in comment.chars() {
        if ch == '\n' {
            output.push('\n');
        }
    }
}
