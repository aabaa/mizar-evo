use crate::lexical_env::ActiveLexicalEnvironment;
use crate::preprocess::PreprocessedSource;
use crate::span_bridge::{LexerByteSpan, SpanBridge, SpanBridgeError};
use mizar_lexer::{
    RawToken, RawTokenKind, ScopeSkeleton, ScopeSkeletonDiagnostic, SourceSpan as LexerSourceSpan,
    build_scope_skeleton, scan_raw,
};
use mizar_session::{MappedSourceRange, SourceAnchor, SourceId, SourceRange};
use std::sync::Arc;

pub use mizar_lexer::{
    BindingShapeKind, LexDiagnosticCode, LexicalBlockKind, LexicalStatementKind, ParserLexContext,
    ScopeSkeletonDiagnosticCode, TokenKind,
};

pub type InternedText = Arc<str>;

#[derive(Debug, Clone, Copy)]
pub struct TokenizeRequest<'a> {
    pub preprocessed: &'a PreprocessedSource,
    pub environment: &'a ActiveLexicalEnvironment,
    pub parser_context: ParserLexContext,
}

impl<'a> TokenizeRequest<'a> {
    pub fn new(
        preprocessed: &'a PreprocessedSource,
        environment: &'a ActiveLexicalEnvironment,
        parser_context: ParserLexContext,
    ) -> Self {
        Self {
            preprocessed,
            environment,
            parser_context,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStream {
    pub source_id: SourceId,
    pub parser_context: ParserLexContext,
    pub tokens: Vec<Token>,
    pub scope_view: ScopeView,
    pub diagnostics: Vec<LexingDiagnostic>,
}

impl TokenStream {
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn diagnostics(&self) -> &[LexingDiagnostic] {
        &self.diagnostics
    }

    pub fn scope_view(&self) -> &ScopeView {
        &self.scope_view
    }

    pub fn into_parts(self) -> (Vec<Token>, ScopeView, Vec<LexingDiagnostic>) {
        (self.tokens, self.scope_view, self.diagnostics)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: InternedText,
    pub span: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeView {
    pub source_id: SourceId,
    pub frames: Vec<ScopeFrame>,
    pub blocks: Vec<ScopeBlock>,
    pub statements: Vec<ScopeStatement>,
}

impl ScopeView {
    pub fn empty(source_id: SourceId) -> Self {
        Self {
            source_id,
            frames: Vec::new(),
            blocks: Vec::new(),
            statements: Vec::new(),
        }
    }

    pub fn binding_overrides_symbol(&self, spelling: &str, position: usize) -> bool {
        self.frames.iter().any(|frame| {
            frame.range.start <= position
                && position < frame.range.end
                && frame.bindings.iter().any(|binding| {
                    binding.spelling.as_ref() == spelling
                        && binding.introduced_at.end <= position
                        && position < frame.range.end
                })
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeFrame {
    pub range: SourceRange,
    pub bindings: Vec<ScopedBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedBinding {
    pub spelling: InternedText,
    pub introduced_at: SourceRange,
    pub kind: BindingShapeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeBlock {
    pub kind: LexicalBlockKind,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeStatement {
    pub kind: LexicalStatementKind,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexingDiagnostic {
    pub kind: LexingDiagnosticKind,
    pub message: InternedText,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
    pub payload: LexingDiagnosticPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexingDiagnosticKind {
    RawScan,
    ScopeSkeleton(ScopeSkeletonDiagnosticCode),
    Lexer(LexDiagnosticCode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexingDiagnosticPayload {
    None,
}

pub fn tokenize(
    request: TokenizeRequest<'_>,
    bridge: &SpanBridge,
) -> Result<TokenStream, SpanBridgeError> {
    let source_id = request.preprocessed.source_id;
    let lexical_text = request.preprocessed.lexical_text.as_str();
    let raw = match scan_raw(lexical_text) {
        Ok(raw) => raw,
        Err(error) => {
            let (token, diagnostic) =
                raw_scan_recovery(source_id, lexical_text, bridge, error.to_string())?;
            return Ok(TokenStream {
                source_id,
                parser_context: request.parser_context,
                tokens: vec![token],
                scope_view: ScopeView::empty(source_id),
                diagnostics: vec![diagnostic],
            });
        }
    };

    let scope_skeleton = build_scope_skeleton(&raw);
    let tokens = raw
        .tokens()
        .iter()
        .filter_map(|raw_token| raw_token_frontend_token(source_id, bridge, raw_token).transpose())
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    let scope_view = scope_view(source_id, bridge, &scope_skeleton)?;
    let diagnostics = scope_skeleton
        .diagnostics
        .iter()
        .map(|diagnostic| scope_skeleton_diagnostic(source_id, bridge, diagnostic))
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;

    Ok(TokenStream {
        source_id,
        parser_context: request.parser_context,
        tokens,
        scope_view,
        diagnostics,
    })
}

fn raw_token_frontend_token(
    source_id: SourceId,
    bridge: &SpanBridge,
    raw_token: &RawToken,
) -> Result<Option<Token>, SpanBridgeError> {
    let Some(kind) = raw_token_frontend_kind(raw_token.kind) else {
        return Ok(None);
    };
    let mapping = lexical_mapping(source_id, bridge, raw_token.span)?;
    Ok(Some(Token {
        kind,
        text: Arc::<str>::from(raw_token.lexeme.as_str()),
        span: mapping.primary,
    }))
}

fn raw_token_frontend_kind(raw_kind: RawTokenKind) -> Option<TokenKind> {
    match raw_kind {
        RawTokenKind::Layout => None,
        RawTokenKind::Error => Some(TokenKind::ErrorRecovery),
        RawTokenKind::LexemeRun | RawTokenKind::NumeralLike | RawTokenKind::AnnotationMarker => {
            Some(TokenKind::LexemeRun)
        }
        _ => Some(TokenKind::LexemeRun),
    }
}

fn scope_view(
    source_id: SourceId,
    bridge: &SpanBridge,
    skeleton: &ScopeSkeleton,
) -> Result<ScopeView, SpanBridgeError> {
    let frames = skeleton
        .frames
        .iter()
        .map(|frame| {
            Ok(ScopeFrame {
                range: lexical_source_range(source_id, bridge, frame.range)?,
                bindings: frame
                    .bindings
                    .iter()
                    .map(|binding| {
                        Ok(ScopedBinding {
                            spelling: Arc::<str>::from(binding.spelling.as_str()),
                            introduced_at: lexical_source_range(
                                source_id,
                                bridge,
                                binding.introduced_at,
                            )?,
                            kind: binding.kind,
                        })
                    })
                    .collect::<Result<Vec<_>, SpanBridgeError>>()?,
            })
        })
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    let blocks = skeleton
        .blocks
        .iter()
        .map(|block| {
            Ok(ScopeBlock {
                kind: block.kind,
                range: lexical_source_range(source_id, bridge, block.range)?,
            })
        })
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    let statements = skeleton
        .statements
        .iter()
        .map(|statement| {
            Ok(ScopeStatement {
                kind: statement.kind,
                range: lexical_source_range(source_id, bridge, statement.range)?,
            })
        })
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;

    Ok(ScopeView {
        source_id,
        frames,
        blocks,
        statements,
    })
}

fn scope_skeleton_diagnostic(
    source_id: SourceId,
    bridge: &SpanBridge,
    diagnostic: &ScopeSkeletonDiagnostic,
) -> Result<LexingDiagnostic, SpanBridgeError> {
    let mapping = lexical_mapping(source_id, bridge, diagnostic.span)?;
    Ok(LexingDiagnostic {
        kind: LexingDiagnosticKind::ScopeSkeleton(diagnostic.code),
        message: Arc::<str>::from(diagnostic.message.as_str()),
        primary: mapping.primary,
        secondary: mapping.secondary,
        payload: LexingDiagnosticPayload::None,
    })
}

fn raw_scan_recovery(
    source_id: SourceId,
    lexical_text: &str,
    bridge: &SpanBridge,
    error: String,
) -> Result<(Token, LexingDiagnostic), SpanBridgeError> {
    let mapping = whole_lexical_text_mapping(source_id, lexical_text, bridge)?;
    Ok((
        Token {
            kind: TokenKind::ErrorRecovery,
            text: Arc::<str>::from(lexical_text),
            span: mapping.primary,
        },
        LexingDiagnostic {
            kind: LexingDiagnosticKind::RawScan,
            message: Arc::<str>::from(format!("raw scan failed: {error}")),
            primary: mapping.primary,
            secondary: mapping.secondary,
            payload: LexingDiagnosticPayload::None,
        },
    ))
}

fn whole_lexical_text_mapping(
    source_id: SourceId,
    lexical_text: &str,
    bridge: &SpanBridge,
) -> Result<MappedSourceRange, SpanBridgeError> {
    if lexical_text.is_empty() {
        bridge.loaded_mapping(source_id, LexerByteSpan { start: 0, end: 0 })
    } else {
        bridge.lexical_span(
            source_id,
            LexerByteSpan {
                start: 0,
                end: lexical_text.len(),
            },
        )
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
    bridge.lexical_span(source_id, lexer_byte_span(span))
}

fn lexer_byte_span(span: LexerSourceSpan) -> LexerByteSpan {
    LexerByteSpan {
        start: span.start,
        end: span.end,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BindingShapeKind, LexicalBlockKind, LexicalStatementKind, LexingDiagnosticKind,
        LexingDiagnosticPayload, ParserLexContext, ScopeBlock, ScopeFrame,
        ScopeSkeletonDiagnosticCode, ScopeStatement, TokenKind, TokenizeRequest, tokenize,
    };
    use crate::preprocess::preprocess;
    use crate::source::{SourceUnit, register_source_unit};
    use crate::span_bridge::SpanBridge;
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, LineMap, ModulePath, PackageId,
        SessionIdAllocator, SourceOrigin, SourceRange, hash_text, normalize_path,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn raw_scan_preserves_lexeme_run_spans() {
        let text = "alpha \t\n+ beta";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            stream
                .tokens
                .iter()
                .map(|token| (token.kind, token.text.as_ref(), token.span))
                .collect::<Vec<_>>(),
            vec![
                (
                    TokenKind::LexemeRun,
                    "alpha",
                    SourceRange {
                        source_id: source.source_id,
                        start: 0,
                        end: 5,
                    },
                ),
                (
                    TokenKind::LexemeRun,
                    "+",
                    SourceRange {
                        source_id: source.source_id,
                        start: 8,
                        end: 9,
                    },
                ),
                (
                    TokenKind::LexemeRun,
                    "beta",
                    SourceRange {
                        source_id: source.source_id,
                        start: 10,
                        end: 14,
                    },
                ),
            ]
        );
        assert!(stream.diagnostics.is_empty());
    }

    #[test]
    fn raw_scan_token_spans_map_through_preprocess_bridge() {
        let text = "alpha::=hidden=::beta";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(preprocessed.lexical_text.as_str(), "alpha beta");
        assert_eq!(
            stream
                .tokens
                .iter()
                .map(|token| (token.kind, token.text.as_ref(), token.span))
                .collect::<Vec<_>>(),
            vec![
                (
                    TokenKind::LexemeRun,
                    "alpha",
                    SourceRange {
                        source_id: source.source_id,
                        start: 0,
                        end: 5,
                    },
                ),
                (
                    TokenKind::LexemeRun,
                    "beta",
                    SourceRange {
                        source_id: source.source_id,
                        start: 17,
                        end: 21,
                    },
                ),
            ]
        );
        assert!(stream.diagnostics.is_empty());
    }

    #[test]
    fn scope_view_reflects_lexical_shape_without_resolved_bindings() {
        let text = "\
definition
let x be set;
now
let y be set;
y;
end;
y;
end;
x;";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = environment_with_imported_symbol("imported_symbol");

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            stream.scope_view.blocks,
            vec![
                ScopeBlock {
                    kind: LexicalBlockKind::Definition,
                    range: range(
                        source.source_id,
                        nth_index(text, "definition", 0),
                        nth_index(text, "end;\nx;", 0) + "end".len(),
                    ),
                },
                ScopeBlock {
                    kind: LexicalBlockKind::Now,
                    range: range(
                        source.source_id,
                        nth_index(text, "now", 0),
                        nth_index(text, "end;\ny;", 0) + "end".len(),
                    ),
                },
            ]
        );
        assert_eq!(
            stream.scope_view.statements,
            vec![
                ScopeStatement {
                    kind: LexicalStatementKind::Binder,
                    range: range(
                        source.source_id,
                        nth_index(text, "let x", 0),
                        nth_index(text, ";\nnow", 0) + ";".len(),
                    ),
                },
                ScopeStatement {
                    kind: LexicalStatementKind::Binder,
                    range: range(
                        source.source_id,
                        nth_index(text, "let y", 0),
                        nth_index(text, ";\ny;\nend", 0) + ";".len(),
                    ),
                },
            ]
        );
        assert_eq!(
            stream.scope_view.frames,
            vec![
                ScopeFrame {
                    range: range(
                        source.source_id,
                        nth_index(text, "definition", 0),
                        nth_index(text, "end;\nx;", 0) + "end".len(),
                    ),
                    bindings: vec![super::ScopedBinding {
                        spelling: Arc::from("x"),
                        introduced_at: exact_range(source.source_id, text, "x be set"),
                        kind: BindingShapeKind::Let,
                    }],
                },
                ScopeFrame {
                    range: range(
                        source.source_id,
                        nth_index(text, "now", 0),
                        nth_index(text, "end;\ny;", 0) + "end".len(),
                    ),
                    bindings: vec![super::ScopedBinding {
                        spelling: Arc::from("y"),
                        introduced_at: exact_range(source.source_id, text, "y be set"),
                        kind: BindingShapeKind::Let,
                    }],
                },
            ]
        );
        assert!(
            stream
                .scope_view
                .binding_overrides_symbol("x", nth_index(text, "now", 0))
        );
        assert!(
            !stream
                .scope_view
                .binding_overrides_symbol("y", nth_index(text, "y;\nend", 1))
        );
        assert!(
            stream
                .scope_view
                .frames
                .iter()
                .flat_map(|frame| &frame.bindings)
                .all(|binding| binding.spelling.as_ref() != "imported_symbol")
        );
        assert!(stream.diagnostics.is_empty());
    }

    #[test]
    fn scope_skeleton_diagnostics_are_mapped_to_frontend_diagnostics() {
        let text = "end;\ndefinition\nlet x be set;";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(stream.diagnostics.len(), 2);
        assert_eq!(
            stream.diagnostics[0].kind,
            LexingDiagnosticKind::ScopeSkeleton(ScopeSkeletonDiagnosticCode::UnmatchedEnd)
        );
        assert_eq!(
            stream.diagnostics[0].primary,
            range(
                source.source_id,
                nth_index(text, "end", 0),
                nth_index(text, "end", 0) + "end".len()
            )
        );
        assert!(stream.diagnostics[0].message.contains("unmatched `end`"));
        assert!(stream.diagnostics[0].secondary.is_empty());
        assert_eq!(stream.diagnostics[0].payload, LexingDiagnosticPayload::None);
        assert_eq!(
            stream.diagnostics[1].kind,
            LexingDiagnosticKind::ScopeSkeleton(ScopeSkeletonDiagnosticCode::MissingEnd)
        );
        assert_eq!(
            stream.diagnostics[1].primary,
            range(
                source.source_id,
                nth_index(text, "definition", 0),
                nth_index(text, "definition", 0),
            )
        );
        assert!(stream.diagnostics[1].message.contains("missing `end`"));
        assert!(stream.diagnostics[1].secondary.is_empty());
        assert_eq!(stream.diagnostics[1].payload, LexingDiagnosticPayload::None);

        let super::TokenStream {
            source_id: _,
            parser_context: _,
            tokens: _,
            scope_view: _,
            diagnostics,
        } = stream;
        let _: Vec<super::LexingDiagnostic> = diagnostics;
    }

    #[test]
    fn raw_scan_failure_returns_coarse_recovery_token_and_diagnostic() {
        let text = "@-";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(stream.scope_view.frames, Vec::new());
        assert_eq!(stream.tokens.len(), 1);
        assert_eq!(stream.tokens[0].kind, TokenKind::ErrorRecovery);
        assert_eq!(stream.tokens[0].text.as_ref(), "@-");
        assert_eq!(stream.tokens[0].span, range(source.source_id, 0, 2));
        assert_eq!(stream.diagnostics.len(), 1);
        assert_eq!(stream.diagnostics[0].kind, LexingDiagnosticKind::RawScan);
        assert!(stream.diagnostics[0].message.contains("raw scan failed"));
        assert_eq!(stream.diagnostics[0].primary, range(source.source_id, 0, 2));
        assert_eq!(stream.diagnostics[0].payload, LexingDiagnosticPayload::None);
    }

    fn preprocessed_source(
        text: &str,
    ) -> (
        SourceUnit,
        crate::preprocess::PreprocessedSource,
        SpanBridge,
    ) {
        let source = source_unit(text);
        let mut bridge = SpanBridge::new();
        register_source_unit(&mut bridge, &source).unwrap();
        let preprocessed = preprocess(&source, &mut bridge).unwrap();
        (source, preprocessed, bridge)
    }

    fn empty_environment() -> mizar_lexer::ActiveLexicalEnvironment {
        mizar_lexer::build_lexical_environment(&[], &[]).unwrap()
    }

    fn environment_with_imported_symbol(spelling: &str) -> mizar_lexer::ActiveLexicalEnvironment {
        let module = mizar_lexer::ModuleId::new("imported.env");
        mizar_lexer::build_lexical_environment(
            &[mizar_lexer::ResolvedImport {
                module_id: module.clone(),
            }],
            &[mizar_lexer::ModuleLexicalSummary {
                module_id: module.clone(),
                exported_symbols: vec![mizar_lexer::ExportedSymbolShape {
                    spelling: spelling.to_owned(),
                    symbol_id: mizar_lexer::SymbolId::new("imported.env#symbol"),
                    source_module: module,
                    export_rank: mizar_lexer::ExportRank::new(0),
                    kind: mizar_lexer::UserSymbolKind::Functor,
                    arity: mizar_lexer::UserSymbolArity::exact(2),
                }],
                fingerprint: mizar_lexer::LexicalSummaryFingerprint::new(11),
            }],
        )
        .unwrap()
    }

    fn range(source_id: mizar_session::SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn exact_range(
        source_id: mizar_session::SourceId,
        haystack: &str,
        context_needle: &str,
    ) -> SourceRange {
        let context_start = haystack
            .find(context_needle)
            .expect("test fixture contains context needle");
        SourceRange {
            source_id,
            start: context_start,
            end: context_start + context_needle.find(' ').unwrap_or(context_needle.len()),
        }
    }

    fn nth_index(haystack: &str, needle: &str, nth: usize) -> usize {
        haystack
            .match_indices(needle)
            .nth(nth)
            .map(|(index, _)| index)
            .expect("test fixture contains nth needle")
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
                "mizar-frontend-lexing-test-{}-{id}",
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
}
