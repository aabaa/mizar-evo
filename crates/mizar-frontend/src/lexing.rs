use crate::lexical_env::ActiveLexicalEnvironment;
use crate::preprocess::PreprocessedSource;
use crate::span_bridge::{LexerByteSpan, SpanBridge, SpanBridgeError};
use mizar_lexer::{
    LexDiagnostic as LexerDiagnostic, LexDiagnosticPayload as LexerDiagnosticPayload, RawToken,
    RawTokenStream, RejectedTokenCandidate as LexerRejectedTokenCandidate, ScopeSkeleton,
    ScopeSkeletonDiagnostic, SourceSpan as LexerSourceSpan, Token as LexerToken,
    TokenStream as LexerTokenStream, build_scope_skeleton, disambiguate, scan_raw,
};
use mizar_session::{MappedSourceRange, SourceAnchor, SourceId, SourceRange};
use std::sync::Arc;

pub use mizar_lexer::{
    BindingShapeKind, LexDiagnosticCode, LexRecoveryHint, LexicalBlockKind, LexicalStatementKind,
    MalformedStringLiteralReason, ParserLexContext, ParserLexMode, RawTokenKind,
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
    NoValidTokenCandidate {
        rejected_lexeme: InternedText,
        recovery: LexRecoveryHint,
    },
    ParserContextRejectedCandidate {
        mode: ParserLexMode,
        rejected_lexeme: InternedText,
        candidates: Vec<LexingRejectedTokenCandidate>,
        recovery: LexRecoveryHint,
    },
    MalformedStringLiteral {
        opening_quote: char,
        reason: MalformedStringLiteralReason,
        recovery: LexRecoveryHint,
    },
    UnsupportedRawToken {
        raw_kind: RawTokenKind,
        raw_lexeme: InternedText,
        recovery: LexRecoveryHint,
    },
    UnsupportedLexerPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexingRejectedTokenCandidate {
    pub kind: TokenKind,
    pub text: InternedText,
    pub span: SourceRange,
    pub secondary: Vec<SourceAnchor>,
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

    let (lexer_stream, scope_skeleton) =
        disambiguate_with_contextual_scope(&raw, request.environment, &request.parser_context);
    let tokens = lexer_stream
        .tokens()
        .iter()
        .map(|token| lexer_token(source_id, bridge, token))
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    let scope_view = scope_view(source_id, bridge, &scope_skeleton)?;
    let mut diagnostics = scope_skeleton
        .diagnostics
        .iter()
        .map(|diagnostic| scope_skeleton_diagnostic(source_id, bridge, diagnostic))
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    diagnostics.extend(
        lexer_stream
            .diagnostics()
            .iter()
            .map(|diagnostic| lexer_diagnostic(source_id, bridge, diagnostic))
            .collect::<Result<Vec<_>, SpanBridgeError>>()?,
    );

    Ok(TokenStream {
        source_id,
        parser_context: request.parser_context,
        tokens,
        scope_view,
        diagnostics,
    })
}

fn disambiguate_with_contextual_scope(
    raw: &RawTokenStream,
    environment: &ActiveLexicalEnvironment,
    parser_context: &ParserLexContext,
) -> (LexerTokenStream, ScopeSkeleton) {
    let raw_scope_skeleton = build_scope_skeleton(raw);
    let first_stream = disambiguate(raw, environment, parser_context, &raw_scope_skeleton);
    let contextual_scope_skeleton =
        build_scope_skeleton(&scope_raw_stream_from_tokens(first_stream.tokens()));
    let final_stream = disambiguate(raw, environment, parser_context, &contextual_scope_skeleton);
    let final_scope_skeleton =
        build_scope_skeleton(&scope_raw_stream_from_tokens(final_stream.tokens()));
    (final_stream, final_scope_skeleton)
}

fn scope_raw_stream_from_tokens(tokens: &[LexerToken]) -> RawTokenStream {
    RawTokenStream::new(
        tokens
            .iter()
            .map(|token| RawToken::new(scope_raw_kind(token), token.lexeme.clone(), token.span))
            .collect(),
    )
}

fn scope_raw_kind(token: &LexerToken) -> RawTokenKind {
    match token.kind {
        TokenKind::Identifier | TokenKind::ReservedWord | TokenKind::ReservedSymbol => {
            RawTokenKind::LexemeRun
        }
        TokenKind::UserSymbol if mizar_lexer::is_identifier(&token.lexeme) => {
            RawTokenKind::LexemeRun
        }
        TokenKind::LexemeRun => RawTokenKind::LexemeRun,
        TokenKind::Numeral => RawTokenKind::NumeralLike,
        TokenKind::StringLiteral | TokenKind::UserSymbol | TokenKind::ErrorRecovery => {
            RawTokenKind::Error
        }
        _ => RawTokenKind::Error,
    }
}

fn lexer_token(
    source_id: SourceId,
    bridge: &SpanBridge,
    token: &LexerToken,
) -> Result<Token, SpanBridgeError> {
    let mapping = lexical_mapping(source_id, bridge, token.span)?;
    Ok(Token {
        kind: token.kind,
        text: Arc::<str>::from(token.lexeme.as_str()),
        span: mapping.primary,
    })
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

fn lexer_diagnostic(
    source_id: SourceId,
    bridge: &SpanBridge,
    diagnostic: &LexerDiagnostic,
) -> Result<LexingDiagnostic, SpanBridgeError> {
    let mapping = lexical_mapping(source_id, bridge, diagnostic.span)?;
    Ok(LexingDiagnostic {
        kind: LexingDiagnosticKind::Lexer(diagnostic.code),
        message: Arc::<str>::from(diagnostic.message.as_str()),
        primary: mapping.primary,
        secondary: mapping.secondary,
        payload: lexer_diagnostic_payload(source_id, bridge, &diagnostic.payload)?,
    })
}

fn lexer_diagnostic_payload(
    source_id: SourceId,
    bridge: &SpanBridge,
    payload: &LexerDiagnosticPayload,
) -> Result<LexingDiagnosticPayload, SpanBridgeError> {
    Ok(match payload {
        LexerDiagnosticPayload::None => LexingDiagnosticPayload::None,
        LexerDiagnosticPayload::NoValidTokenCandidate {
            rejected_lexeme,
            recovery,
        } => LexingDiagnosticPayload::NoValidTokenCandidate {
            rejected_lexeme: Arc::<str>::from(rejected_lexeme.as_str()),
            recovery: *recovery,
        },
        LexerDiagnosticPayload::ParserContextRejectedCandidate {
            mode,
            rejected_lexeme,
            candidates,
            recovery,
        } => LexingDiagnosticPayload::ParserContextRejectedCandidate {
            mode: *mode,
            rejected_lexeme: Arc::<str>::from(rejected_lexeme.as_str()),
            candidates: candidates
                .iter()
                .map(|candidate| lexer_rejected_candidate(source_id, bridge, candidate))
                .collect::<Result<Vec<_>, SpanBridgeError>>()?,
            recovery: *recovery,
        },
        LexerDiagnosticPayload::MalformedStringLiteral {
            opening_quote,
            reason,
            recovery,
        } => LexingDiagnosticPayload::MalformedStringLiteral {
            opening_quote: *opening_quote,
            reason: *reason,
            recovery: *recovery,
        },
        LexerDiagnosticPayload::UnsupportedRawToken {
            raw_kind,
            raw_lexeme,
            recovery,
        } => LexingDiagnosticPayload::UnsupportedRawToken {
            raw_kind: *raw_kind,
            raw_lexeme: Arc::<str>::from(raw_lexeme.as_str()),
            recovery: *recovery,
        },
        _ => LexingDiagnosticPayload::UnsupportedLexerPayload,
    })
}

fn lexer_rejected_candidate(
    source_id: SourceId,
    bridge: &SpanBridge,
    candidate: &LexerRejectedTokenCandidate,
) -> Result<LexingRejectedTokenCandidate, SpanBridgeError> {
    let mapping = lexical_mapping(source_id, bridge, candidate.span)?;
    Ok(LexingRejectedTokenCandidate {
        kind: candidate.kind,
        text: Arc::<str>::from(candidate.lexeme.as_str()),
        span: mapping.primary,
        secondary: mapping.secondary,
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
        BindingShapeKind, LexDiagnosticCode, LexRecoveryHint, LexicalBlockKind,
        LexicalStatementKind, LexingDiagnosticKind, LexingDiagnosticPayload,
        LexingRejectedTokenCandidate, MalformedStringLiteralReason, ParserLexContext,
        ParserLexMode, ScopeBlock, ScopeFrame, ScopeSkeletonDiagnosticCode, ScopeStatement,
        TokenKind, TokenizeRequest, tokenize,
    };
    use crate::preprocess::preprocess;
    use crate::source::{SourceUnit, register_source_unit};
    use crate::span_bridge::SpanBridge;
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, LineMap, ModulePath, PackageId,
        SessionIdAllocator, SourceAnchor, SourceOrigin, SourceRange, hash_text, normalize_path,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn disambiguation_preserves_final_token_spans() {
        let text = "alpha \t\n:= beta";
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
                    TokenKind::Identifier,
                    "alpha",
                    SourceRange {
                        source_id: source.source_id,
                        start: 0,
                        end: 5,
                    },
                ),
                (
                    TokenKind::ReservedSymbol,
                    ":=",
                    SourceRange {
                        source_id: source.source_id,
                        start: 8,
                        end: 10,
                    },
                ),
                (
                    TokenKind::Identifier,
                    "beta",
                    SourceRange {
                        source_id: source.source_id,
                        start: 11,
                        end: 15,
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
                    TokenKind::Identifier,
                    "alpha",
                    SourceRange {
                        source_id: source.source_id,
                        start: 0,
                        end: 5,
                    },
                ),
                (
                    TokenKind::Identifier,
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
    fn disambiguation_prefers_longest_user_symbol_inside_raw_runs() {
        let text = "x+*+y";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = environment_with_imported_symbols(&["+", "+*", "+*+"]);

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![
                (TokenKind::Identifier, "x", range(source.source_id, 0, 1)),
                (TokenKind::UserSymbol, "+*+", range(source.source_id, 1, 4)),
                (TokenKind::Identifier, "y", range(source.source_id, 4, 5)),
            ]
        );
        assert!(stream.diagnostics.is_empty());
    }

    #[test]
    fn disambiguation_uses_scope_view_for_identifier_shaped_symbol_overrides() {
        let text = "succ\ndefinition\nlet succ be set;\nsucc;\nend;";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = environment_with_imported_symbol("succ");

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            stream
                .tokens
                .iter()
                .map(|token| (token.kind, token.text.as_ref()))
                .collect::<Vec<_>>(),
            vec![
                (TokenKind::UserSymbol, "succ"),
                (TokenKind::ReservedWord, "definition"),
                (TokenKind::ReservedWord, "let"),
                (TokenKind::UserSymbol, "succ"),
                (TokenKind::ReservedWord, "be"),
                (TokenKind::ReservedWord, "set"),
                (TokenKind::ReservedSymbol, ";"),
                (TokenKind::Identifier, "succ"),
                (TokenKind::ReservedSymbol, ";"),
                (TokenKind::ReservedWord, "end"),
                (TokenKind::ReservedSymbol, ";"),
            ]
        );
        assert_eq!(
            stream.tokens[7].span,
            range(
                source.source_id,
                nth_index(text, "succ;\nend", 0),
                nth_index(text, "succ;\nend", 0) + "succ".len()
            )
        );
        assert!(stream.diagnostics.is_empty());
    }

    #[test]
    fn disambiguation_emits_compound_reserved_symbols_as_single_tokens() {
        let text = ".{.*.=...";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![
                (
                    TokenKind::ReservedSymbol,
                    ".{",
                    range(source.source_id, 0, 2)
                ),
                (
                    TokenKind::ReservedSymbol,
                    ".*",
                    range(source.source_id, 2, 4)
                ),
                (
                    TokenKind::ReservedSymbol,
                    ".=",
                    range(source.source_id, 4, 6)
                ),
                (
                    TokenKind::ReservedSymbol,
                    "...",
                    range(source.source_id, 6, 9)
                ),
            ]
        );
        assert!(stream.diagnostics.is_empty());
    }

    #[test]
    fn parser_context_controls_string_literal_disambiguation_and_payload_mapping() {
        let text = "\"alpha\"";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let string_stream = tokenize(
            TokenizeRequest::new(
                &preprocessed,
                &environment,
                ParserLexContext::string_required(),
            ),
            &bridge,
        )
        .unwrap();
        let general_stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&string_stream),
            vec![(
                TokenKind::StringLiteral,
                "\"alpha\"",
                range(source.source_id, 0, 7)
            )]
        );
        assert!(string_stream.diagnostics.is_empty());
        assert_eq!(
            general_stream
                .tokens
                .iter()
                .map(|token| (token.kind, token.text.as_ref(), token.span))
                .collect::<Vec<_>>(),
            vec![
                (
                    TokenKind::ErrorRecovery,
                    "\"",
                    range(source.source_id, 0, 1),
                ),
                (
                    TokenKind::Identifier,
                    "alpha",
                    range(source.source_id, 1, 6),
                ),
                (
                    TokenKind::ErrorRecovery,
                    "\"",
                    range(source.source_id, 6, 7),
                ),
            ]
        );
        assert_eq!(general_stream.diagnostics.len(), 2);
        assert_eq!(
            general_stream.diagnostics[0].kind,
            LexingDiagnosticKind::Lexer(LexDiagnosticCode::ParserContextRejectedCandidate)
        );
        assert_eq!(
            general_stream.diagnostics[0].payload,
            LexingDiagnosticPayload::ParserContextRejectedCandidate {
                mode: ParserLexMode::General,
                rejected_lexeme: Arc::from("\""),
                candidates: vec![LexingRejectedTokenCandidate {
                    kind: TokenKind::StringLiteral,
                    text: Arc::from("\"alpha\""),
                    span: range(source.source_id, 0, 7),
                    secondary: Vec::new(),
                }],
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
        assert_eq!(
            general_stream.diagnostics[1].kind,
            LexingDiagnosticKind::Lexer(LexDiagnosticCode::NoValidTokenCandidate)
        );
        assert_eq!(
            general_stream.diagnostics[1].payload,
            LexingDiagnosticPayload::NoValidTokenCandidate {
                rejected_lexeme: Arc::from("\""),
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
    }

    #[test]
    fn quote_delimited_active_user_symbol_is_admitted_in_general_context() {
        let text = "\"end\"";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = environment_with_imported_symbol("\"end\"");

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![(
                TokenKind::UserSymbol,
                "\"end\"",
                range(source.source_id, 0, 5)
            )]
        );
        assert_eq!(stream.scope_view.blocks, Vec::new());
        assert!(stream.diagnostics.is_empty());
    }

    #[test]
    fn string_required_scope_words_do_not_emit_scope_diagnostics() {
        let text = "\"end\"";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(
                &preprocessed,
                &environment,
                ParserLexContext::string_required(),
            ),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![(
                TokenKind::StringLiteral,
                "\"end\"",
                range(source.source_id, 0, 5)
            )]
        );
        assert_eq!(stream.scope_view.blocks, Vec::new());
        assert!(stream.diagnostics.is_empty());
    }

    #[test]
    fn malformed_string_literal_payload_is_preserved_with_session_span() {
        let text = "\"bad\\n\"";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(
                &preprocessed,
                &environment,
                ParserLexContext::string_required(),
            ),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![(
                TokenKind::ErrorRecovery,
                "\"bad\\n\"",
                range(source.source_id, 0, 7)
            )]
        );
        assert_eq!(stream.diagnostics.len(), 1);
        assert_eq!(
            stream.diagnostics[0].kind,
            LexingDiagnosticKind::Lexer(LexDiagnosticCode::MalformedStringLiteral)
        );
        assert_eq!(
            stream.diagnostics[0].message.as_ref(),
            "malformed string literal"
        );
        assert_eq!(stream.diagnostics[0].primary, range(source.source_id, 0, 7));
        assert_eq!(
            stream.diagnostics[0].payload,
            LexingDiagnosticPayload::MalformedStringLiteral {
                opening_quote: '"',
                reason: MalformedStringLiteralReason::UnsupportedEscape { escape: 'n' },
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
    }

    #[test]
    fn lexer_diagnostic_mapping_preserves_secondary_anchors_for_payload_candidates() {
        let text = "alpha::=hidden=::beta";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let lexical_span = mizar_lexer::SourceSpan {
            start: 0,
            end: preprocessed.lexical_text.as_str().len(),
        };
        let diagnostic = mizar_lexer::LexDiagnostic::with_payload(
            LexDiagnosticCode::ParserContextRejectedCandidate,
            "parser context rejected synthetic fixture candidate",
            lexical_span,
            mizar_lexer::LexDiagnosticPayload::ParserContextRejectedCandidate {
                mode: ParserLexMode::General,
                rejected_lexeme: preprocessed.lexical_text.as_str().to_owned(),
                candidates: vec![mizar_lexer::RejectedTokenCandidate {
                    kind: TokenKind::Identifier,
                    lexeme: preprocessed.lexical_text.as_str().to_owned(),
                    span: lexical_span,
                }],
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            },
        );

        let mapped = super::lexer_diagnostic(source.source_id, &bridge, &diagnostic).unwrap();

        assert_eq!(
            mapped.message.as_ref(),
            "parser context rejected synthetic fixture candidate"
        );
        assert_eq!(mapped.primary, range(source.source_id, 0, text.len()),);
        assert_eq!(
            mapped.secondary,
            vec![
                SourceAnchor::Range(range(source.source_id, 0, "alpha".len())),
                SourceAnchor::Range(range(
                    source.source_id,
                    nth_index(text, "::=hidden=::", 0),
                    nth_index(text, "beta", 0)
                )),
                SourceAnchor::Range(range(
                    source.source_id,
                    nth_index(text, "beta", 0),
                    text.len()
                )),
            ]
        );
        assert_eq!(
            mapped.payload,
            LexingDiagnosticPayload::ParserContextRejectedCandidate {
                mode: ParserLexMode::General,
                rejected_lexeme: Arc::from(preprocessed.lexical_text.as_str()),
                candidates: vec![LexingRejectedTokenCandidate {
                    kind: TokenKind::Identifier,
                    text: Arc::from(preprocessed.lexical_text.as_str()),
                    span: range(source.source_id, 0, text.len()),
                    secondary: mapped.secondary.clone(),
                }],
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
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

    #[test]
    fn raw_scan_failure_preserves_secondary_anchors_from_preprocess_mapping() {
        let text = "@-::=hidden=::beta";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        assert_eq!(preprocessed.lexical_text.as_str(), "@- beta");

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(stream.tokens.len(), 1);
        assert_eq!(stream.tokens[0].kind, TokenKind::ErrorRecovery);
        assert_eq!(
            stream.tokens[0].span,
            range(source.source_id, 0, text.len())
        );
        assert_eq!(stream.diagnostics.len(), 1);
        assert_eq!(stream.diagnostics[0].kind, LexingDiagnosticKind::RawScan);
        assert_eq!(
            stream.diagnostics[0].primary,
            range(source.source_id, 0, text.len())
        );
        assert_eq!(
            stream.diagnostics[0].secondary,
            vec![
                SourceAnchor::Range(range(source.source_id, 0, 2)),
                SourceAnchor::Range(range(
                    source.source_id,
                    nth_index(text, "::=hidden=::", 0),
                    nth_index(text, "beta", 0)
                )),
                SourceAnchor::Range(range(
                    source.source_id,
                    nth_index(text, "beta", 0),
                    text.len()
                )),
            ]
        );
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
        environment_with_imported_symbols(&[spelling])
    }

    fn environment_with_imported_symbols(
        spellings: &[&str],
    ) -> mizar_lexer::ActiveLexicalEnvironment {
        let module = mizar_lexer::ModuleId::new("imported.env");
        mizar_lexer::build_lexical_environment(
            &[mizar_lexer::ResolvedImport {
                module_id: module.clone(),
            }],
            &[mizar_lexer::ModuleLexicalSummary {
                module_id: module.clone(),
                exported_symbols: spellings
                    .iter()
                    .enumerate()
                    .map(|(rank, spelling)| mizar_lexer::ExportedSymbolShape {
                        spelling: (*spelling).to_owned(),
                        symbol_id: mizar_lexer::SymbolId::new(format!(
                            "imported.env#symbol.{rank}"
                        )),
                        source_module: module.clone(),
                        export_rank: mizar_lexer::ExportRank::new(rank as u32),
                        kind: mizar_lexer::UserSymbolKind::Functor,
                        arity: mizar_lexer::UserSymbolArity::exact(2),
                    })
                    .collect(),
                fingerprint: mizar_lexer::LexicalSummaryFingerprint::new(11),
            }],
        )
        .unwrap()
    }

    fn token_kinds_texts_and_spans(
        stream: &super::TokenStream,
    ) -> Vec<(TokenKind, &str, SourceRange)> {
        stream
            .tokens
            .iter()
            .map(|token| (token.kind, token.text.as_ref(), token.span))
            .collect()
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
