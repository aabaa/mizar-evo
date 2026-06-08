use crate::source::SourceUnit;
use crate::span_bridge::{LexerByteSpan, SpanBridge, SpanBridgeError, comment_kind_from_lexer};
use mizar_lexer::{
    CommentKind as LexerCommentKind, ImportPrescanDiagnostic as LexerImportPrescanDiagnostic,
    ImportStub as LexerImportStub, RawModuleAlias as LexerRawModuleAlias,
    RawModulePath as LexerRawModulePath, RawModuleRelativePrefix as LexerRawModuleRelativePrefix,
    SourcePreprocessMap, SourceSpan as LexerSourceSpan, preprocess_source_for_lexing,
    scan_import_prelude, scan_raw,
};
pub use mizar_lexer::{ImportPrescanDiagnosticCode, SourcePreprocessDiagnosticCode};
pub use mizar_session::CommentKind;
use mizar_session::{Hash, MappedSourceRange, SourceAnchor, SourceId, SourceRange};
use std::sync::Arc;

const LEXICAL_HASH_DOMAIN: &[u8] = b"mizar-frontend/preprocess/lexical-text/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessedSource {
    pub source_id: SourceId,
    pub lexical_text: LexicalText,
    pub lexical_hash: Hash,
    pub comments: Vec<Comment>,
    pub doc_comments: Vec<DocComment>,
    pub import_stubs: Vec<ImportStub>,
    pub source_map: LexicalSourceMap,
    pub diagnostics: Vec<PreprocessDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalText {
    pub text: Arc<str>,
}

impl LexicalText {
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub kind: CommentKind,
    pub source_range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocComment {
    pub source_range: SourceRange,
    pub raw_body: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalSourceMap {
    pub source_id: SourceId,
    pub lexical_text_len: usize,
    pub preprocess_map: SourcePreprocessMap,
}

impl LexicalSourceMap {
    pub fn lexical_span(
        &self,
        bridge: &SpanBridge,
        span: LexerByteSpan,
    ) -> Result<MappedSourceRange, SpanBridgeError> {
        bridge.lexical_span(self.source_id, span)
    }

    pub const fn lexical_len(&self) -> usize {
        self.lexical_text_len
    }

    pub const fn is_empty(&self) -> bool {
        self.lexical_text_len == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStub {
    pub path: ImportStubPath,
    pub alias: Option<ImportStubAlias>,
    pub span: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStubPath {
    pub spelling: Arc<str>,
    pub relative: Option<ImportStubRelativePrefix>,
    pub components: Vec<Arc<str>>,
    pub source_segments: Vec<SourceRange>,
    pub span: SourceRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportStubRelativePrefix {
    Current,
    Parent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStubAlias {
    pub spelling: Arc<str>,
    pub span: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessDiagnostic {
    pub kind: PreprocessDiagnosticKind,
    pub message: Arc<str>,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreprocessDiagnosticKind {
    SourcePrecondition(SourcePreprocessDiagnosticCode),
    ImportPrescan(ImportPrescanDiagnosticCode),
    RawImportScan,
}

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
    let raw = match scan_raw(lexical_text.as_str()) {
        Ok(raw) => raw,
        Err(error) => {
            return Ok((
                Vec::new(),
                vec![raw_import_scan_diagnostic(
                    source_id,
                    lexical_text,
                    bridge,
                    error.to_string(),
                )?],
            ));
        }
    };
    let prelude = scan_import_prelude(&raw);
    let import_stubs = prelude
        .imports
        .into_iter()
        .map(|import| import_stub(source_id, bridge, import))
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    let diagnostics = prelude
        .diagnostics
        .into_iter()
        .map(|diagnostic| import_prescan_diagnostic(source_id, bridge, diagnostic))
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    Ok((import_stubs, diagnostics))
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
    lexical_text: &LexicalText,
    bridge: &SpanBridge,
    error: String,
) -> Result<PreprocessDiagnostic, SpanBridgeError> {
    let mapping = if lexical_text.as_str().is_empty() {
        bridge.loaded_mapping(source_id, LexerByteSpan { start: 0, end: 0 })?
    } else {
        bridge.lexical_span(
            source_id,
            LexerByteSpan {
                start: 0,
                end: lexical_text.text.len(),
            },
        )?
    };
    Ok(PreprocessDiagnostic {
        kind: PreprocessDiagnosticKind::RawImportScan,
        message: Arc::<str>::from(format!("raw import pre-scan failed: {error}")),
        primary: mapping.primary,
        secondary: mapping.secondary,
    })
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
    fn raw_import_scan_failure_is_coarse_and_leaves_import_stubs_empty() {
        let (source, mut bridge) = registered_source_unit("@-");

        let preprocessed = preprocess(&source, &mut bridge).unwrap();

        assert!(preprocessed.import_stubs.is_empty());
        assert_eq!(preprocessed.diagnostics.len(), 1);
        assert_eq!(
            preprocessed.diagnostics[0].kind,
            PreprocessDiagnosticKind::RawImportScan
        );
        assert!(
            preprocessed.diagnostics[0]
                .message
                .starts_with("raw import pre-scan failed:")
        );
        assert_eq!(
            preprocessed.diagnostics[0].primary,
            SourceRange {
                source_id: source.source_id,
                start: 0,
                end: 2,
            }
        );
        assert!(preprocessed.diagnostics[0].secondary.is_empty());
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
                start: 0,
                end: source.source_text.len(),
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
                "raw import pre-scan failed: unsupported raw lexer input at byte 5: 'β'",
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
                    start: 0,
                    end: source.source_text.len(),
                },
            ]
        );
        assert!(
            preprocessed
                .diagnostics
                .iter()
                .take(2)
                .all(|diagnostic| diagnostic.secondary.is_empty())
        );
        assert!(
            !preprocessed
                .diagnostics
                .iter()
                .last()
                .expect("raw import scan diagnostic exists")
                .secondary
                .is_empty()
        );
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
