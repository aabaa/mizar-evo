use crate::raw_lexer::is_identifier;
use std::error::Error;
use std::fmt;

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
    source_len: usize,
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
pub enum ModuleNamingError {
    MissingMizExtension { path: String },
    MissingFileStem { path: String },
    InvalidPackageName { package_name: String },
    MissingSourceRoot { path: String },
    PackageRootMismatch { package_name: String, root: String },
    InvalidNamespaceComponent { component: String },
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
impl SourceLineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (index, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(index + ch.len_utf8());
            }
        }
        Self {
            line_starts,
            source_len: source.len(),
        }
    }

    pub fn location(&self, offset: usize) -> Option<SourceLocation> {
        if offset > self.source_len {
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
            preserve_comment_newlines(&input[cursor..end], &mut lexical_text);
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
            preserve_comment_newlines(&input[cursor..end], &mut lexical_text);
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
            preserve_comment_newlines(&input[cursor..end], &mut lexical_text);
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

fn preserve_comment_newlines(comment: &str, output: &mut String) {
    for ch in comment.chars() {
        if ch == '\n' {
            output.push('\n');
        }
    }
}
