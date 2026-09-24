//! Tokenization, scope-view extraction, and parser-assisted lexing plans.
//!
//! Canonical behavior is specified in the
//! [lexing design spec](../../../../doc/design/mizar-frontend/en/lexing.md).

use crate::lexical_env::{
    ActiveLexicalEnvironment, ExportRank, ExportedOperatorAssociativity, ExportedOperatorFixity,
    ExportedOperatorMetadata, LocalLexicalDeclarations, ModuleId, SymbolId, UserSymbolArity,
    UserSymbolKind, UserSymbolKindSet,
};
use crate::preprocess::PreprocessedSource;
use crate::span_bridge::{
    LexerByteSpan, SpanBridge, SpanBridgeError, decode_offsets, decode_source_anchor,
    decode_source_range, encode_offsets, encode_source_anchor, encode_source_range,
};
use mizar_lexer::{
    LexDiagnostic as LexerDiagnostic, LexDiagnosticPayload as LexerDiagnosticPayload,
    LocalOperatorDeclaration, LocalUserSymbolDeclaration, RawScanDiagnostic, RawScanDiagnosticCode,
    RawToken, RawTokenStream, RecoverableRawTokenStream,
    RejectedTokenCandidate as LexerRejectedTokenCandidate, ScopeSkeleton, ScopeSkeletonDiagnostic,
    SourceSpan as LexerSourceSpan, Token as LexerToken, TokenStream as LexerTokenStream,
    build_scope_skeleton, collect_local_lexical_declarations, disambiguate_with_local_declarations,
    scan_raw_recoverable,
};
use mizar_session::{MappedSourceRange, SourceAnchor, SourceId, SourceRange};
use serde_json::{Value, json};
use std::sync::Arc;

/// Re-exported lexer token, context, scope, and diagnostic vocabulary.
pub use mizar_lexer::{
    BindingShapeKind, LexDiagnosticCode, LexRecoveryHint, LexicalBlockKind, LexicalStatementKind,
    MalformedStringLiteralReason, ParserLexContext, ParserLexMode, RawTokenKind,
    ScopeSkeletonDiagnosticCode, TokenKind,
};

const TOKEN_STREAM_SCHEMA: &str = "mizar-frontend/token-stream/v1";
const TOKEN_STREAM_MAX_BYTES: usize = 16 * 1024 * 1024;
const TOKEN_KINDS: [TokenKind; 9] = [
    TokenKind::Identifier,
    TokenKind::ReservedWord,
    TokenKind::ReservedSymbol,
    TokenKind::Numeral,
    TokenKind::LexemeRun,
    TokenKind::UserSymbol,
    TokenKind::AnnotationMarker,
    TokenKind::StringLiteral,
    TokenKind::ErrorRecovery,
];
const MODES: [ParserLexMode; 6] = [
    ParserLexMode::General,
    ParserLexMode::IdentifierRequired,
    ParserLexMode::Symbolic,
    ParserLexMode::StringRequired,
    ParserLexMode::NamespacePath,
    ParserLexMode::Recovery,
];
const USER_KINDS: [UserSymbolKind; 7] = [
    UserSymbolKind::Functor,
    UserSymbolKind::Predicate,
    UserSymbolKind::Mode,
    UserSymbolKind::Attribute,
    UserSymbolKind::Structure,
    UserSymbolKind::Selector,
    UserSymbolKind::Constructor,
];
const BINDING_KINDS: [BindingShapeKind; 14] = [
    BindingShapeKind::Let,
    BindingShapeKind::For,
    BindingShapeKind::Ex,
    BindingShapeKind::Reserve,
    BindingShapeKind::Given,
    BindingShapeKind::Consider,
    BindingShapeKind::Set,
    BindingShapeKind::Reconsider,
    BindingShapeKind::Take,
    BindingShapeKind::Deffunc,
    BindingShapeKind::Defpred,
    BindingShapeKind::Var,
    BindingShapeKind::Const,
    BindingShapeKind::Processed,
];
const BLOCK_KINDS: [LexicalBlockKind; 9] = [
    LexicalBlockKind::Algorithm,
    LexicalBlockKind::Definition,
    LexicalBlockKind::Registration,
    LexicalBlockKind::Proof,
    LexicalBlockKind::Now,
    LexicalBlockKind::Case,
    LexicalBlockKind::Suppose,
    LexicalBlockKind::Hereby,
    LexicalBlockKind::Do,
];
const STATEMENT_KINDS: [LexicalStatementKind; 2] =
    [LexicalStatementKind::Binder, LexicalStatementKind::Other];
const SCOPE_CODES: [ScopeSkeletonDiagnosticCode; 5] = [
    ScopeSkeletonDiagnosticCode::MalformedBinderList,
    ScopeSkeletonDiagnosticCode::UnsupportedBinderShape,
    ScopeSkeletonDiagnosticCode::DuplicateBindingName,
    ScopeSkeletonDiagnosticCode::UnmatchedEnd,
    ScopeSkeletonDiagnosticCode::MissingEnd,
];
const LEX_CODES: [LexDiagnosticCode; 5] = [
    LexDiagnosticCode::NoValidTokenCandidate,
    LexDiagnosticCode::ParserContextRejectedCandidate,
    LexDiagnosticCode::AmbiguousUserSymbol,
    LexDiagnosticCode::MalformedStringLiteral,
    LexDiagnosticCode::UnsupportedRawToken,
];
const RAW_KINDS: [RawTokenKind; 5] = [
    RawTokenKind::LexemeRun,
    RawTokenKind::NumeralLike,
    RawTokenKind::AnnotationMarker,
    RawTokenKind::Layout,
    RawTokenKind::Error,
];
const RECOVERY_HINTS: [LexRecoveryHint; 1] = [LexRecoveryHint::EmitErrorRecoveryToken];
const FIXITIES: [ExportedOperatorFixity; 5] = [
    ExportedOperatorFixity::Prefix,
    ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
    ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Right),
    ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::NonAssociative),
    ExportedOperatorFixity::Postfix,
];

fn encode_tag<T: PartialEq>(item: &T, vocabulary: &[T]) -> Option<Value> {
    Some(json!(
        vocabulary.iter().position(|candidate| candidate == item)?
    ))
}

fn decode_tag<T: Copy>(value: &Value, vocabulary: &[T]) -> Option<T> {
    vocabulary
        .get(usize::try_from(value.as_u64()?).ok()?)
        .copied()
}

fn fields<const N: usize>(value: &Value) -> Option<&[Value; N]> {
    value.as_array()?.as_slice().try_into().ok()
}

fn encode_list<T>(items: &[T], mut encode: impl FnMut(&T) -> Option<Value>) -> Option<Vec<Value>> {
    items.iter().map(&mut encode).collect()
}

fn decode_list<T>(value: &Value, mut decode: impl FnMut(&Value) -> Option<T>) -> Option<Vec<T>> {
    value.as_array()?.iter().map(&mut decode).collect()
}

pub(crate) fn encode_context(context: ParserLexContext) -> Option<Value> {
    let kinds = USER_KINDS
        .into_iter()
        .filter(|kind| context.user_symbol_kinds().contains(*kind))
        .collect::<Vec<_>>();
    if UserSymbolKindSet::from_slice(&kinds) != context.user_symbol_kinds() {
        return None;
    }
    Some(json!([
        encode_tag(&context.mode(), &MODES)?,
        encode_list(&kinds, |kind| encode_tag(kind, &USER_KINDS))?
    ]))
}

pub(crate) fn decode_context(value: &Value) -> Option<ParserLexContext> {
    let [mode, kinds] = fields::<2>(value)?;
    let mode = decode_tag(mode, &MODES)?;
    let mut previous = None;
    let kinds = decode_list(kinds, |kind| {
        let tag = kind.as_u64()?;
        if previous.is_some_and(|prior| tag <= prior) {
            return None;
        }
        previous = Some(tag);
        decode_tag(kind, &USER_KINDS)
    })?;
    let context = match mode {
        ParserLexMode::General => ParserLexContext::general(),
        ParserLexMode::IdentifierRequired => ParserLexContext::identifier_required(),
        ParserLexMode::Symbolic => ParserLexContext::symbolic(),
        ParserLexMode::StringRequired => ParserLexContext::string_required(),
        ParserLexMode::NamespacePath => ParserLexContext::namespace_path(),
        ParserLexMode::Recovery => ParserLexContext::recovery(),
        _ => return None,
    };
    Some(context.with_user_symbol_kinds(UserSymbolKindSet::from_slice(&kinds)))
}

/// Shared interned text used by frontend token and diagnostic payloads.
pub type InternedText = Arc<str>;

/// Tokenization request for one preprocessed source.
#[derive(Debug, Clone)]
pub struct TokenizeRequest<'a> {
    /// Preprocessed source to tokenize.
    pub preprocessed: &'a PreprocessedSource,
    /// Active lexical environment for disambiguation.
    pub environment: &'a ActiveLexicalEnvironment,
    /// Current module id used for local declaration provenance.
    pub current_module: ModuleId,
    /// Default parser lexing context.
    pub parser_context: ParserLexContext,
    /// Position-sensitive parser lexing plan.
    pub parser_lexing_plan: ParserLexingPlan,
}

impl<'a> TokenizeRequest<'a> {
    /// Creates a tokenization request with a uniform parser lexing context.
    pub fn new(
        preprocessed: &'a PreprocessedSource,
        environment: &'a ActiveLexicalEnvironment,
        parser_context: ParserLexContext,
    ) -> Self {
        Self {
            preprocessed,
            environment,
            current_module: fallback_current_module(preprocessed.source_id),
            parser_context,
            parser_lexing_plan: ParserLexingPlan::uniform(parser_context),
        }
    }

    /// Creates a tokenization request with a position-sensitive lexing plan.
    pub fn with_plan(
        preprocessed: &'a PreprocessedSource,
        environment: &'a ActiveLexicalEnvironment,
        parser_lexing_plan: ParserLexingPlan,
    ) -> Self {
        Self {
            preprocessed,
            environment,
            current_module: fallback_current_module(preprocessed.source_id),
            parser_context: parser_lexing_plan.default_context,
            parser_lexing_plan,
        }
    }

    /// Sets the current module id used by local declaration collection.
    pub fn with_current_module(mut self, current_module: ModuleId) -> Self {
        self.current_module = current_module;
        self
    }
}

/// Position-sensitive lexer context plan produced for parser-sensitive regions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserLexingPlan {
    /// Context used outside explicit context ranges.
    pub default_context: ParserLexContext,
    /// Sorted context overrides in lexical-text byte coordinates.
    pub contexts: Vec<ParserLexingPlanContext>,
}

impl ParserLexingPlan {
    /// Creates a plan with one context for the whole source.
    pub fn uniform(default_context: ParserLexContext) -> Self {
        Self {
            default_context,
            contexts: Vec::new(),
        }
    }

    /// Creates a plan from explicit context ranges, sorting ranges by position.
    pub fn new(
        default_context: ParserLexContext,
        mut contexts: Vec<ParserLexingPlanContext>,
    ) -> Self {
        contexts.sort_by(|left, right| {
            left.range
                .start
                .cmp(&right.range.start)
                .then(left.range.end.cmp(&right.range.end))
        });
        Self {
            default_context,
            contexts,
        }
    }

    /// Builds the current position-sensitive plan for a lexical text.
    pub fn for_lexical_text(lexical_text: &str) -> Self {
        Self::new(
            ParserLexContext::general(),
            string_argument_ranges(lexical_text)
                .into_iter()
                .map(|range| {
                    ParserLexingPlanContext::new(range, ParserLexContext::string_required())
                })
                .collect(),
        )
    }

    /// Returns the parser lexing context active at the byte offset.
    pub fn context_at(&self, offset: usize) -> ParserLexContext {
        self.contexts
            .iter()
            .find(|context| context.range.contains(offset))
            .map_or(self.default_context, |context| context.context)
    }

    /// Returns whether the plan uses only its default context.
    pub fn is_uniform(&self) -> bool {
        self.contexts.is_empty()
    }
}

/// Parser lexing context override for a lexical byte range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserLexingPlanContext {
    /// Lexical byte range covered by this override.
    pub range: LexicalByteRange,
    /// Parser lexing context active in the range.
    pub context: ParserLexContext,
}

impl ParserLexingPlanContext {
    /// Creates a parser lexing context override.
    pub fn new(range: LexicalByteRange, context: ParserLexContext) -> Self {
        Self { range, context }
    }
}

/// Half-open byte range in lexical-text coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexicalByteRange {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

impl LexicalByteRange {
    /// Creates a lexical byte range, requiring `start <= end`.
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "lexical byte range start must not exceed end");
        Self { start, end }
    }

    /// Returns whether the byte offset is inside the half-open range.
    pub fn contains(self, offset: usize) -> bool {
        self.start <= offset && offset < self.end
    }
}

/// Token stream produced by frontend tokenization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStream {
    /// Source id for all ranges in the token stream.
    pub source_id: SourceId,
    /// Default parser lexing context used by the stream.
    pub parser_context: ParserLexContext,
    /// Position-sensitive parser lexing plan used by the stream.
    pub parser_lexing_plan: ParserLexingPlan,
    /// Tokens sorted in source order.
    pub tokens: Vec<Token>,
    /// Scope skeleton view derived during lexing.
    pub scope_view: ScopeView,
    /// Current-module lexical declarations collected during tokenization.
    pub local_declarations: LocalLexicalDeclarations,
    /// Recoverable lexing diagnostics.
    pub diagnostics: Vec<LexingDiagnostic>,
}

impl TokenStream {
    /// Decodes canonical storage and rebinds source ranges to this session's source id.
    pub fn from_canonical_bytes(bytes: &[u8], source_id: SourceId) -> Option<Self> {
        if bytes.len() > TOKEN_STREAM_MAX_BYTES {
            return None;
        }
        let value: Value = serde_json::from_slice(bytes).ok()?;
        let [
            schema,
            parser_context,
            plan,
            tokens,
            scope,
            local,
            diagnostics,
        ] = fields::<7>(&value)?;
        if schema.as_str()? != TOKEN_STREAM_SCHEMA {
            return None;
        }
        let parser_context = decode_context(parser_context)?;
        let [default_context, contexts] = fields::<2>(plan)?;
        let parser_lexing_plan = ParserLexingPlan {
            default_context: decode_context(default_context)?,
            contexts: decode_list(contexts, |entry| {
                let [range, context] = fields::<2>(entry)?;
                let (start, end) = decode_offsets(range)?;
                Some(ParserLexingPlanContext {
                    range: LexicalByteRange { start, end },
                    context: decode_context(context)?,
                })
            })?,
        };
        let tokens = decode_list(tokens, |token| {
            let [kind, text, span] = fields::<3>(token)?;
            Some(Token {
                kind: decode_tag(kind, &TOKEN_KINDS)?,
                text: Arc::from(text.as_str()?),
                span: decode_source_range(span, source_id)?,
            })
        })?;
        let [frames, blocks, statements] = fields::<3>(scope)?;
        let scope_view = ScopeView {
            source_id,
            frames: decode_list(frames, |frame| {
                let [range, bindings] = fields::<2>(frame)?;
                Some(ScopeFrame {
                    range: decode_source_range(range, source_id)?,
                    bindings: decode_list(bindings, |binding| {
                        let [spelling, introduced_at, kind] = fields::<3>(binding)?;
                        Some(ScopedBinding {
                            spelling: Arc::from(spelling.as_str()?),
                            introduced_at: decode_source_range(introduced_at, source_id)?,
                            kind: decode_tag(kind, &BINDING_KINDS)?,
                        })
                    })?,
                })
            })?,
            blocks: decode_list(blocks, |block| {
                let [kind, range] = fields::<2>(block)?;
                Some(ScopeBlock {
                    kind: decode_tag(kind, &BLOCK_KINDS)?,
                    range: decode_source_range(range, source_id)?,
                })
            })?,
            statements: decode_list(statements, |statement| {
                let [kind, range] = fields::<2>(statement)?;
                Some(ScopeStatement {
                    kind: decode_tag(kind, &STATEMENT_KINDS)?,
                    range: decode_source_range(range, source_id)?,
                })
            })?,
        };
        let [user_symbols, operator_declarations] = fields::<2>(local)?;
        let local_declarations = LocalLexicalDeclarations {
            user_symbols: decode_list(user_symbols, |declaration| {
                let [
                    spelling,
                    symbol_id,
                    source_module,
                    export_rank,
                    kind,
                    arity,
                    operator,
                    range,
                    activation_start,
                ] = fields::<9>(declaration)?;
                let [minimum, maximum] = fields::<2>(arity)?;
                let maximum = if maximum.is_null() {
                    None
                } else {
                    Some(u16::try_from(maximum.as_u64()?).ok()?)
                };
                let operator = if operator.is_null() {
                    None
                } else {
                    let [fixity, precedence] = fields::<2>(operator)?;
                    Some(ExportedOperatorMetadata {
                        fixity: decode_tag(fixity, &FIXITIES)?,
                        precedence: u8::try_from(precedence.as_u64()?).ok()?,
                    })
                };
                let (start, end) = decode_offsets(range)?;
                Some(LocalUserSymbolDeclaration {
                    spelling: spelling.as_str()?.to_owned(),
                    symbol_id: SymbolId::new(symbol_id.as_str()?),
                    source_module: ModuleId::new(source_module.as_str()?),
                    export_rank: ExportRank::new(u32::try_from(export_rank.as_u64()?).ok()?),
                    kind: decode_tag(kind, &USER_KINDS)?,
                    arity: UserSymbolArity {
                        minimum: u16::try_from(minimum.as_u64()?).ok()?,
                        maximum,
                    },
                    operator,
                    declared_at: LexerSourceSpan { start, end },
                    activation_start: usize::try_from(activation_start.as_u64()?).ok()?,
                })
            })?,
            operator_declarations: decode_list(operator_declarations, |declaration| {
                let [spelling, source_module, range, activation_start, operator] =
                    fields::<5>(declaration)?;
                let operator = if operator.is_null() {
                    None
                } else {
                    let [fixity, precedence] = fields::<2>(operator)?;
                    Some(ExportedOperatorMetadata {
                        fixity: decode_tag(fixity, &FIXITIES)?,
                        precedence: u8::try_from(precedence.as_u64()?).ok()?,
                    })
                };
                let (start, end) = decode_offsets(range)?;
                Some(LocalOperatorDeclaration {
                    spelling: spelling.as_str()?.to_owned(),
                    source_module: ModuleId::new(source_module.as_str()?),
                    declared_at: LexerSourceSpan { start, end },
                    activation_start: usize::try_from(activation_start.as_u64()?).ok()?,
                    operator,
                })
            })?,
        };
        let diagnostics = decode_list(diagnostics, |diagnostic| {
            let [kind, message, primary, secondary, payload] = fields::<5>(diagnostic)?;
            let kind = match kind.as_array()?.as_slice() {
                [tag] if tag.as_u64()? == 0 => LexingDiagnosticKind::RawScan,
                [tag, code] if tag.as_u64()? == 1 => {
                    LexingDiagnosticKind::ScopeSkeleton(decode_tag(code, &SCOPE_CODES)?)
                }
                [tag, code] if tag.as_u64()? == 2 => {
                    LexingDiagnosticKind::Lexer(decode_tag(code, &LEX_CODES)?)
                }
                _ => return None,
            };
            let payload = match payload.as_array()?.as_slice() {
                [tag] if tag.as_u64()? == 0 => LexingDiagnosticPayload::None,
                [tag, rejected_lexeme, recovery] if tag.as_u64()? == 1 => {
                    LexingDiagnosticPayload::NoValidTokenCandidate {
                        rejected_lexeme: Arc::from(rejected_lexeme.as_str()?),
                        recovery: decode_tag(recovery, &RECOVERY_HINTS)?,
                    }
                }
                [tag, mode, rejected_lexeme, candidates, recovery] if tag.as_u64()? == 2 => {
                    LexingDiagnosticPayload::ParserContextRejectedCandidate {
                        mode: decode_tag(mode, &MODES)?,
                        rejected_lexeme: Arc::from(rejected_lexeme.as_str()?),
                        candidates: decode_list(candidates, |candidate| {
                            let [kind, text, span, secondary] = fields::<4>(candidate)?;
                            Some(LexingRejectedTokenCandidate {
                                kind: decode_tag(kind, &TOKEN_KINDS)?,
                                text: Arc::from(text.as_str()?),
                                span: decode_source_range(span, source_id)?,
                                secondary: decode_list(secondary, |anchor| {
                                    decode_source_anchor(anchor, source_id)
                                })?,
                            })
                        })?,
                        recovery: decode_tag(recovery, &RECOVERY_HINTS)?,
                    }
                }
                [tag, opening_quote, reason, recovery] if tag.as_u64()? == 3 => {
                    let quote = opening_quote.as_str()?;
                    let mut chars = quote.chars();
                    let opening_quote = chars.next()?;
                    if chars.next().is_some() {
                        return None;
                    }
                    let reason = match reason.as_array()?.as_slice() {
                        [tag] if tag.as_u64()? == 0 => {
                            MalformedStringLiteralReason::MissingClosingQuote
                        }
                        [tag, escape] if tag.as_u64()? == 1 => {
                            let mut chars = escape.as_str()?.chars();
                            let escape = chars.next()?;
                            if chars.next().is_some() {
                                return None;
                            }
                            MalformedStringLiteralReason::UnsupportedEscape { escape }
                        }
                        [tag] if tag.as_u64()? == 2 => MalformedStringLiteralReason::DanglingEscape,
                        _ => return None,
                    };
                    LexingDiagnosticPayload::MalformedStringLiteral {
                        opening_quote,
                        reason,
                        recovery: decode_tag(recovery, &RECOVERY_HINTS)?,
                    }
                }
                [tag, raw_kind, raw_lexeme, recovery] if tag.as_u64()? == 4 => {
                    LexingDiagnosticPayload::UnsupportedRawToken {
                        raw_kind: decode_tag(raw_kind, &RAW_KINDS)?,
                        raw_lexeme: Arc::from(raw_lexeme.as_str()?),
                        recovery: decode_tag(recovery, &RECOVERY_HINTS)?,
                    }
                }
                [tag] if tag.as_u64()? == 5 => LexingDiagnosticPayload::UnsupportedLexerPayload,
                _ => return None,
            };
            Some(LexingDiagnostic {
                kind,
                message: Arc::from(message.as_str()?),
                primary: decode_source_range(primary, source_id)?,
                secondary: decode_list(secondary, |anchor| {
                    decode_source_anchor(anchor, source_id)
                })?,
                payload,
            })
        })?;
        let stream = Self {
            source_id,
            parser_context,
            parser_lexing_plan,
            tokens,
            scope_view,
            local_declarations,
            diagnostics,
        };
        (stream.canonical_bytes()?.as_slice() == bytes).then_some(stream)
    }

    /// Encodes bounded canonical compiler-internal token stream storage.
    pub fn canonical_bytes(&self) -> Option<Vec<u8>> {
        if self.scope_view.source_id != self.source_id {
            return None;
        }
        let source_id = self.source_id;
        let plan = json!([
            encode_context(self.parser_lexing_plan.default_context)?,
            encode_list(&self.parser_lexing_plan.contexts, |entry| Some(json!([
                encode_offsets(entry.range.start, entry.range.end)?,
                encode_context(entry.context)?
            ])))?
        ]);
        let tokens = encode_list(&self.tokens, |token| {
            Some(json!([
                encode_tag(&token.kind, &TOKEN_KINDS)?,
                token.text.as_ref(),
                encode_source_range(token.span, source_id)?
            ]))
        })?;
        let scope = json!([
            encode_list(&self.scope_view.frames, |frame| Some(json!([
                encode_source_range(frame.range, source_id)?,
                encode_list(&frame.bindings, |binding| Some(json!([
                    binding.spelling.as_ref(),
                    encode_source_range(binding.introduced_at, source_id)?,
                    encode_tag(&binding.kind, &BINDING_KINDS)?
                ])))?
            ])))?,
            encode_list(&self.scope_view.blocks, |block| Some(json!([
                encode_tag(&block.kind, &BLOCK_KINDS)?,
                encode_source_range(block.range, source_id)?
            ])))?,
            encode_list(&self.scope_view.statements, |statement| Some(json!([
                encode_tag(&statement.kind, &STATEMENT_KINDS)?,
                encode_source_range(statement.range, source_id)?
            ])))?
        ]);
        let local = json!([
            encode_list(&self.local_declarations.user_symbols, |declaration| {
                let operator = match declaration.operator {
                    Some(operator) => json!([
                        encode_tag(&operator.fixity, &FIXITIES)?,
                        operator.precedence
                    ]),
                    None => Value::Null,
                };
                Some(json!([
                    declaration.spelling,
                    declaration.symbol_id.as_str(),
                    declaration.source_module.as_str(),
                    declaration.export_rank.get(),
                    encode_tag(&declaration.kind, &USER_KINDS)?,
                    [
                        json!(declaration.arity.minimum),
                        json!(declaration.arity.maximum)
                    ],
                    operator,
                    encode_offsets(declaration.declared_at.start, declaration.declared_at.end)?,
                    declaration.activation_start
                ]))
            })?,
            encode_list(
                &self.local_declarations.operator_declarations,
                |declaration| {
                    let operator = match declaration.operator {
                        Some(operator) => json!([
                            encode_tag(&operator.fixity, &FIXITIES)?,
                            operator.precedence
                        ]),
                        None => Value::Null,
                    };
                    Some(json!([
                        declaration.spelling,
                        declaration.source_module.as_str(),
                        encode_offsets(declaration.declared_at.start, declaration.declared_at.end)?,
                        declaration.activation_start,
                        operator
                    ]))
                }
            )?
        ]);
        let diagnostics = encode_list(&self.diagnostics, |diagnostic| {
            let kind = match diagnostic.kind {
                LexingDiagnosticKind::RawScan => json!([0]),
                LexingDiagnosticKind::ScopeSkeleton(code) => {
                    json!([1, encode_tag(&code, &SCOPE_CODES)?])
                }
                LexingDiagnosticKind::Lexer(code) => json!([2, encode_tag(&code, &LEX_CODES)?]),
            };
            let payload = match &diagnostic.payload {
                LexingDiagnosticPayload::None => json!([0]),
                LexingDiagnosticPayload::NoValidTokenCandidate {
                    rejected_lexeme,
                    recovery,
                } => json!([
                    1,
                    rejected_lexeme.as_ref(),
                    encode_tag(recovery, &RECOVERY_HINTS)?
                ]),
                LexingDiagnosticPayload::ParserContextRejectedCandidate {
                    mode,
                    rejected_lexeme,
                    candidates,
                    recovery,
                } => json!([
                    2,
                    encode_tag(mode, &MODES)?,
                    rejected_lexeme.as_ref(),
                    encode_list(candidates, |candidate| Some(json!([
                        encode_tag(&candidate.kind, &TOKEN_KINDS)?,
                        candidate.text.as_ref(),
                        encode_source_range(candidate.span, source_id)?,
                        encode_list(&candidate.secondary, |anchor| encode_source_anchor(
                            anchor, source_id
                        ))?
                    ])))?,
                    encode_tag(recovery, &RECOVERY_HINTS)?
                ]),
                LexingDiagnosticPayload::MalformedStringLiteral {
                    opening_quote,
                    reason,
                    recovery,
                } => {
                    let reason = match reason {
                        MalformedStringLiteralReason::MissingClosingQuote => json!([0]),
                        MalformedStringLiteralReason::UnsupportedEscape { escape } => {
                            json!([1, escape.to_string()])
                        }
                        MalformedStringLiteralReason::DanglingEscape => json!([2]),
                        _ => return None,
                    };
                    json!([
                        3,
                        opening_quote.to_string(),
                        reason,
                        encode_tag(recovery, &RECOVERY_HINTS)?
                    ])
                }
                LexingDiagnosticPayload::UnsupportedRawToken {
                    raw_kind,
                    raw_lexeme,
                    recovery,
                } => json!([
                    4,
                    encode_tag(raw_kind, &RAW_KINDS)?,
                    raw_lexeme.as_ref(),
                    encode_tag(recovery, &RECOVERY_HINTS)?
                ]),
                LexingDiagnosticPayload::UnsupportedLexerPayload => json!([5]),
            };
            Some(json!([
                kind,
                diagnostic.message.as_ref(),
                encode_source_range(diagnostic.primary, source_id)?,
                encode_list(&diagnostic.secondary, |anchor| encode_source_anchor(
                    anchor, source_id
                ))?,
                payload
            ]))
        })?;
        let bytes = serde_json::to_vec(&json!([
            TOKEN_STREAM_SCHEMA,
            encode_context(self.parser_context)?,
            plan,
            tokens,
            scope,
            local,
            diagnostics
        ]))
        .ok()?;
        (bytes.len() <= TOKEN_STREAM_MAX_BYTES).then_some(bytes)
    }

    /// Returns the tokens in source order.
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Returns recoverable lexing diagnostics.
    pub fn diagnostics(&self) -> &[LexingDiagnostic] {
        &self.diagnostics
    }

    /// Returns the scope skeleton view.
    pub fn scope_view(&self) -> &ScopeView {
        &self.scope_view
    }

    /// Returns local lexical declarations collected from the raw token stream.
    pub fn local_declarations(&self) -> &LocalLexicalDeclarations {
        &self.local_declarations
    }

    /// Splits the stream into tokens, scope view, and diagnostics.
    pub fn into_parts(self) -> (Vec<Token>, ScopeView, Vec<LexingDiagnostic>) {
        (self.tokens, self.scope_view, self.diagnostics)
    }
}

/// Frontend token with session source coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Final token kind after disambiguation.
    pub kind: TokenKind,
    /// Token text as it appears in lexical text.
    pub text: InternedText,
    /// Source range mapped through preprocessing.
    pub span: SourceRange,
}

/// Scope skeleton view exposed for later phases and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeView {
    /// Source id for all ranges in the scope view.
    pub source_id: SourceId,
    /// Lexical scope frames and their bindings.
    pub frames: Vec<ScopeFrame>,
    /// Lexical block ranges.
    pub blocks: Vec<ScopeBlock>,
    /// Lexical statement ranges.
    pub statements: Vec<ScopeStatement>,
}

impl ScopeView {
    /// Creates an empty scope view for a source.
    pub fn empty(source_id: SourceId) -> Self {
        Self {
            source_id,
            frames: Vec::new(),
            blocks: Vec::new(),
            statements: Vec::new(),
        }
    }

    /// Returns whether a local binding overrides a user symbol at the position.
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

/// Lexical scope frame with locally introduced bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeFrame {
    /// Source range covered by the frame.
    pub range: SourceRange,
    /// Bindings introduced inside the frame.
    pub bindings: Vec<ScopedBinding>,
}

/// Binding introduced by the scope skeleton.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedBinding {
    /// Binding spelling.
    pub spelling: InternedText,
    /// Source range where the binding was introduced.
    pub introduced_at: SourceRange,
    /// Shape of the binding.
    pub kind: BindingShapeKind,
}

/// Lexical block recognized by the scope skeleton.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeBlock {
    /// Block kind.
    pub kind: LexicalBlockKind,
    /// Source range covered by the block.
    pub range: SourceRange,
}

/// Lexical statement recognized by the scope skeleton.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeStatement {
    /// Statement kind.
    pub kind: LexicalStatementKind,
    /// Source range covered by the statement.
    pub range: SourceRange,
}

/// Recoverable lexing diagnostic mapped to source coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexingDiagnostic {
    /// Diagnostic category.
    pub kind: LexingDiagnosticKind,
    /// Human-readable diagnostic message.
    pub message: InternedText,
    /// Primary source range.
    pub primary: SourceRange,
    /// Secondary source anchors for related context.
    pub secondary: Vec<SourceAnchor>,
    /// Structured diagnostic payload.
    pub payload: LexingDiagnosticPayload,
}

/// Category of recoverable lexing diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LexingDiagnosticKind {
    /// Diagnostic reported by recoverable raw scanning.
    RawScan,
    /// Diagnostic reported by scope skeleton construction.
    ScopeSkeleton(ScopeSkeletonDiagnosticCode),
    /// Diagnostic reported by token disambiguation.
    Lexer(LexDiagnosticCode),
}

/// Structured payload attached to recoverable lexing diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LexingDiagnosticPayload {
    /// No structured payload is available.
    None,
    /// No valid token candidate was accepted for a lexeme.
    NoValidTokenCandidate {
        /// Rejected lexeme text.
        rejected_lexeme: InternedText,
        /// Lexer recovery hint.
        recovery: LexRecoveryHint,
    },
    /// Parser context rejected otherwise valid token candidates.
    ParserContextRejectedCandidate {
        /// Parser lexing mode active for the rejected lexeme.
        mode: ParserLexMode,
        /// Rejected lexeme text.
        rejected_lexeme: InternedText,
        /// Candidate tokens rejected by the active context.
        candidates: Vec<LexingRejectedTokenCandidate>,
        /// Lexer recovery hint.
        recovery: LexRecoveryHint,
    },
    /// Malformed string literal diagnostic payload.
    MalformedStringLiteral {
        /// Opening quote character for the literal.
        opening_quote: char,
        /// Reason the string literal is malformed.
        reason: MalformedStringLiteralReason,
        /// Lexer recovery hint.
        recovery: LexRecoveryHint,
    },
    /// Raw token variant not supported by frontend token mapping.
    UnsupportedRawToken {
        /// Raw token kind that could not be mapped.
        raw_kind: RawTokenKind,
        /// Raw token lexeme.
        raw_lexeme: InternedText,
        /// Lexer recovery hint.
        recovery: LexRecoveryHint,
    },
    /// Fallback for future lexer payload variants with no frontend mapping yet.
    UnsupportedLexerPayload,
}

/// Candidate token rejected during lexing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexingRejectedTokenCandidate {
    /// Candidate token kind.
    pub kind: TokenKind,
    /// Candidate token text.
    pub text: InternedText,
    /// Candidate source range.
    pub span: SourceRange,
    /// Candidate secondary anchors from source mapping.
    pub secondary: Vec<SourceAnchor>,
}

/// Tokenizes a preprocessed source using the active lexical environment.
pub fn tokenize(
    request: TokenizeRequest<'_>,
    bridge: &SpanBridge,
) -> Result<TokenStream, SpanBridgeError> {
    let source_id = request.preprocessed.source_id;
    let lexical_text = request.preprocessed.lexical_text.as_str();
    let raw = match scan_raw_with_plan(lexical_text, &request.parser_lexing_plan) {
        Ok(raw) => raw,
        Err(error) => {
            let (token, diagnostic) =
                whole_text_raw_scan_recovery(source_id, lexical_text, bridge, error)?;
            return Ok(TokenStream {
                source_id,
                parser_context: request.parser_context,
                parser_lexing_plan: request.parser_lexing_plan,
                tokens: vec![token],
                scope_view: ScopeView::empty(source_id),
                local_declarations: LocalLexicalDeclarations::empty(),
                diagnostics: vec![diagnostic],
            });
        }
    };
    let (raw_tokens, raw_scan_diagnostics) = raw.into_parts();
    let raw = RawTokenStream::new(
        raw_tokens
            .into_iter()
            .filter(|token| token.kind != RawTokenKind::Error)
            .collect(),
    );

    let mut token_stream = token_stream_from_raw(
        source_id,
        request.parser_lexing_plan,
        &raw,
        request.environment,
        request.current_module,
        bridge,
    )?;
    let (mut recovery_tokens, mut raw_diagnostics) =
        raw_scan_recovery(source_id, lexical_text, bridge, &raw_scan_diagnostics)?;
    token_stream.tokens.append(&mut recovery_tokens);
    token_stream.tokens.sort_by(|left, right| {
        left.span
            .start
            .cmp(&right.span.start)
            .then(left.span.end.cmp(&right.span.end))
    });
    raw_diagnostics.append(&mut token_stream.diagnostics);
    token_stream.diagnostics = raw_diagnostics;
    Ok(token_stream)
}

fn token_stream_from_raw(
    source_id: SourceId,
    parser_lexing_plan: ParserLexingPlan,
    raw: &RawTokenStream,
    environment: &ActiveLexicalEnvironment,
    current_module: ModuleId,
    bridge: &SpanBridge,
) -> Result<TokenStream, SpanBridgeError> {
    let local_declarations = collect_local_lexical_declarations(raw, current_module);
    let (lexer_stream, scope_skeleton) = disambiguate_with_contextual_scope(
        raw,
        environment,
        &local_declarations,
        &parser_lexing_plan,
    );
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
        parser_context: parser_lexing_plan.default_context,
        parser_lexing_plan,
        tokens,
        scope_view,
        local_declarations,
        diagnostics,
    })
}

fn disambiguate_with_contextual_scope(
    raw: &RawTokenStream,
    environment: &ActiveLexicalEnvironment,
    local_declarations: &LocalLexicalDeclarations,
    parser_lexing_plan: &ParserLexingPlan,
) -> (LexerTokenStream, ScopeSkeleton) {
    let raw_scope_skeleton = build_scope_skeleton(raw);
    let first_stream = disambiguate_with_plan(
        raw,
        environment,
        local_declarations,
        parser_lexing_plan,
        &raw_scope_skeleton,
    );
    let contextual_scope_skeleton =
        build_scope_skeleton(&scope_raw_stream_from_tokens(first_stream.tokens()));
    let final_stream = disambiguate_with_plan(
        raw,
        environment,
        local_declarations,
        parser_lexing_plan,
        &contextual_scope_skeleton,
    );
    let final_scope_skeleton =
        build_scope_skeleton(&scope_raw_stream_from_tokens(final_stream.tokens()));
    (final_stream, final_scope_skeleton)
}

fn disambiguate_with_plan(
    raw: &RawTokenStream,
    environment: &ActiveLexicalEnvironment,
    local_declarations: &LocalLexicalDeclarations,
    parser_lexing_plan: &ParserLexingPlan,
    scope_view: &dyn mizar_lexer::ScopeLexView,
) -> LexerTokenStream {
    if parser_lexing_plan.is_uniform() {
        return disambiguate_with_local_declarations(
            raw,
            environment,
            local_declarations,
            &parser_lexing_plan.default_context,
            scope_view,
        );
    }

    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();
    for raw_token in raw.tokens() {
        let context = parser_lexing_plan.context_at(raw_token.span.start);
        let stream = disambiguate_with_local_declarations(
            &RawTokenStream::new(vec![raw_token.clone()]),
            environment,
            local_declarations,
            &context,
            scope_view,
        );
        let (mut stream_tokens, mut stream_diagnostics) = stream.into_parts();
        tokens.append(&mut stream_tokens);
        diagnostics.append(&mut stream_diagnostics);
    }

    LexerTokenStream::new(tokens, diagnostics)
}

fn fallback_current_module(source_id: SourceId) -> ModuleId {
    ModuleId::new(format!("source:{source_id:?}"))
}

fn scan_raw_with_plan(
    lexical_text: &str,
    parser_lexing_plan: &ParserLexingPlan,
) -> Result<RecoverableRawTokenStream, String> {
    if parser_lexing_plan.is_uniform() {
        return Ok(scan_raw_recoverable(lexical_text));
    }

    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();
    let mut cursor = 0;
    for context in &parser_lexing_plan.contexts {
        if context.range.start < cursor {
            continue;
        }
        scan_raw_segment(
            lexical_text,
            cursor,
            context.range.start,
            &mut tokens,
            &mut diagnostics,
        )?;
        if context.context.mode() == ParserLexMode::StringRequired {
            push_planned_string_raw_token(lexical_text, context.range, &mut tokens)?;
        } else {
            scan_raw_segment(
                lexical_text,
                context.range.start,
                context.range.end,
                &mut tokens,
                &mut diagnostics,
            )?;
        }
        cursor = context.range.end;
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
        .ok_or_else(|| format!("parser lexing plan range {start}..{end} is not a UTF-8 span"))?;
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

fn push_planned_string_raw_token(
    lexical_text: &str,
    range: LexicalByteRange,
    tokens: &mut Vec<RawToken>,
) -> Result<(), String> {
    let lexeme = lexical_text.get(range.start..range.end).ok_or_else(|| {
        format!(
            "parser lexing plan string range {}..{} is not a UTF-8 span",
            range.start, range.end
        )
    })?;
    if lexeme.chars().any(|ch| matches!(ch, '\n' | '\r')) {
        return Err(format!(
            "parser lexing plan string range {}..{} crosses a line boundary",
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

fn string_argument_ranges(lexical_text: &str) -> Vec<LexicalByteRange> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while cursor < lexical_text.len() {
        if let Some(end) = string_argument_end(lexical_text, cursor) {
            ranges.push(LexicalByteRange::new(cursor, end));
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
        TokenKind::AnnotationMarker => RawTokenKind::AnnotationMarker,
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
    diagnostics: &[RawScanDiagnostic],
) -> Result<(Vec<Token>, Vec<LexingDiagnostic>), SpanBridgeError> {
    let pairs = diagnostics
        .iter()
        .map(|diagnostic| {
            let mapping = lexical_mapping(source_id, bridge, diagnostic.span)?;
            let text = lexical_text
                .get(diagnostic.span.start..diagnostic.span.end)
                .unwrap_or("");
            Ok((
                Token {
                    kind: TokenKind::ErrorRecovery,
                    text: Arc::<str>::from(text),
                    span: mapping.primary,
                },
                LexingDiagnostic {
                    kind: LexingDiagnosticKind::RawScan,
                    message: Arc::<str>::from(format!(
                        "raw scan recovered after raw scan error: {}",
                        diagnostic.message
                    )),
                    primary: mapping.primary,
                    secondary: mapping.secondary,
                    payload: LexingDiagnosticPayload::None,
                },
            ))
        })
        .collect::<Result<Vec<_>, SpanBridgeError>>()?;
    Ok(pairs.into_iter().unzip())
}

fn whole_text_raw_scan_recovery(
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
    bridge.whole_lexical_text_mapping(source_id, lexical_text)
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
    #[test]
    fn token_stream_storage_round_trips_producer_outputs_with_fresh_id() {
        use super::TokenStream;
        use crate::preprocess::preprocess;
        use mizar_lexer::ModuleId;

        let ids = InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(2)).unwrap();
        let fresh_id = ids.next_source_id(snapshot_id(2)).unwrap();
        for (text, plan_kind) in [
            (
                "func Plus: x + y -> set; :: declaration\ninfix_operator(\"+\", left, 80); :: use\na + b;\n",
                0,
            ),
            ("@[label(\"α::β\", \"γ::δ\")]\n", 1),
            ("end;\ndefinition\nlet x be set;\n@ ?\n", 0),
            ("\"alpha\"", 2),
            ("op op", 3),
        ] {
            let (mut source, preprocessed, bridge) = preprocessed_source(text);
            assert_ne!(source.source_id, fresh_id);
            let environment = if plan_kind == 3 {
                environment_with_same_spelling_kind_overloads()
            } else {
                empty_environment()
            };
            let plan = match plan_kind {
                1 => ParserLexingPlan::for_lexical_text(preprocessed.lexical_text.as_str()),
                2 => ParserLexingPlan::uniform(ParserLexContext::identifier_required()),
                3 => ParserLexingPlan::new(
                    ParserLexContext::general().with_user_symbol_kinds(
                        mizar_lexer::UserSymbolKindSet::only(
                            mizar_lexer::UserSymbolKind::Predicate,
                        ),
                    ),
                    vec![ParserLexingPlanContext::new(
                        LexicalByteRange::new(3, 5),
                        ParserLexContext::general().with_user_symbol_kinds(
                            mizar_lexer::UserSymbolKindSet::only(mizar_lexer::UserSymbolKind::Mode),
                        ),
                    )],
                ),
                _ => ParserLexingPlan::uniform(ParserLexContext::general()),
            };
            let original = tokenize(
                TokenizeRequest::with_plan(&preprocessed, &environment, plan.clone())
                    .with_current_module(ModuleId::new("current")),
                &bridge,
            )
            .unwrap();
            let bytes = original.canonical_bytes().unwrap();
            assert_eq!(
                TokenStream::from_canonical_bytes(&bytes, source.source_id),
                Some(original.clone()),
                "{text:?}"
            );

            source.source_id = fresh_id;
            source.line_map = LineMap::with_source(fresh_id, text);
            let mut fresh_bridge = SpanBridge::new();
            register_source_unit(&mut fresh_bridge, &source).unwrap();
            let fresh_preprocessed = preprocess(&source, &mut fresh_bridge).unwrap();
            let expected = tokenize(
                TokenizeRequest::with_plan(&fresh_preprocessed, &environment, plan)
                    .with_current_module(ModuleId::new("current")),
                &fresh_bridge,
            )
            .unwrap();
            assert_eq!(
                TokenStream::from_canonical_bytes(&bytes, fresh_id),
                Some(expected)
            );
            assert_eq!(
                TokenStream::from_canonical_bytes(&bytes, fresh_id)
                    .unwrap()
                    .canonical_bytes()
                    .unwrap(),
                bytes
            );
            if plan_kind == 0 && text.starts_with("func") {
                assert!(!original.local_declarations.user_symbols.is_empty());
                assert!(!original.local_declarations.operator_declarations.is_empty());
                assert!(
                    original
                        .tokens
                        .iter()
                        .any(|token| token.kind == TokenKind::UserSymbol)
                );
            }
            if plan_kind == 1 {
                assert_eq!(original.parser_lexing_plan.contexts.len(), 2);
                assert!(
                    original
                        .tokens
                        .iter()
                        .any(|token| token.kind == TokenKind::StringLiteral)
                );
            }
            if text.starts_with("end;") {
                assert!(
                    original
                        .diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.kind == LexingDiagnosticKind::RawScan)
                );
                assert!(original.diagnostics.iter().any(|diagnostic| matches!(
                    diagnostic.kind,
                    LexingDiagnosticKind::ScopeSkeleton(_)
                )));
            }
            if plan_kind == 2 {
                assert!(original.diagnostics.iter().any(|diagnostic| matches!(
                    diagnostic.payload,
                    LexingDiagnosticPayload::ParserContextRejectedCandidate { .. }
                )));
            }
            if plan_kind == 3 {
                assert_eq!(
                    original
                        .tokens
                        .iter()
                        .map(|token| token.kind)
                        .collect::<Vec<_>>(),
                    vec![TokenKind::UserSymbol, TokenKind::Identifier]
                );
            }
        }
    }

    #[test]
    fn token_stream_storage_fixed_empty_wire_and_canonical_controls() {
        use super::TokenStream;

        const EMPTY: &[u8] =
            br#"["mizar-frontend/token-stream/v1",[0,[]],[[0,[]],[]],[],[[],[],[]],[[],[]],[]]"#;
        let source_id = source_unit("").source_id;
        let stream = TokenStream::from_canonical_bytes(EMPTY, source_id).unwrap();
        assert_eq!(stream.source_id, source_id);
        assert_eq!(stream.scope_view.source_id, source_id);
        assert_eq!(stream.parser_context.mode(), ParserLexMode::General);
        assert_eq!(
            stream.parser_context.user_symbol_kinds(),
            mizar_lexer::UserSymbolKindSet::empty()
        );
        assert_eq!(stream.canonical_bytes().unwrap(), EMPTY);
        for invalid in [
            b"".as_slice(),
            b"null",
            b"[]",
            b"{}",
            b"\xff",
            b" [\"mizar-frontend/token-stream/v1\",[0,[]],[[0,[]],[]],[],[[],[],[]],[[],[]],[]]",
        ] {
            assert!(TokenStream::from_canonical_bytes(invalid, source_id).is_none());
        }
        let mut with_newline = EMPTY.to_vec();
        with_newline.push(b'\n');
        assert!(TokenStream::from_canonical_bytes(&with_newline, source_id).is_none());
    }

    #[test]
    fn token_stream_storage_fixed_vocabulary_and_diagnostic_wire() {
        use super::TokenStream;
        use mizar_lexer::{
            ExportedOperatorAssociativity as Assoc, ExportedOperatorFixity as Fixity,
            UserSymbolKind, UserSymbolKindSet,
        };
        use serde_json::json;

        let source_id = source_unit("").source_id;
        let value = json!([
            "mizar-frontend/token-stream/v1",
            [0, [0, 2, 6]],
            [
                [0, [0, 2, 6]],
                [
                    [[0, 1], [1, []]],
                    [[1, 2], [2, [1]]],
                    [[2, 3], [3, [2]]],
                    [[3, 4], [4, [3]]],
                    [[4, 5], [5, [4]]],
                    [[5, 6], [0, [5]]]
                ]
            ],
            [
                [0, "id", [0, 1]],
                [1, "word", [1, 2]],
                [2, ";", [2, 3]],
                [3, "2", [3, 4]],
                [4, "run", [4, 5]],
                [5, "+", [5, 6]],
                [6, "@latex", [6, 7]],
                [7, "\"x\"", [7, 8]],
                [8, "?", [8, 9]]
            ],
            [
                [[
                    [0, 20],
                    [
                        ["a", [0, 1], 0],
                        ["b", [1, 2], 1],
                        ["c", [2, 3], 2],
                        ["d", [3, 4], 3],
                        ["e", [4, 5], 4],
                        ["f", [5, 6], 5],
                        ["g", [6, 7], 6],
                        ["h", [7, 8], 7],
                        ["i", [8, 9], 8],
                        ["j", [9, 10], 9],
                        ["k", [10, 11], 10],
                        ["l", [11, 12], 11],
                        ["m", [12, 13], 12],
                        ["n", [13, 14], 13]
                    ]
                ]],
                [
                    [0, [0, 1]],
                    [1, [1, 2]],
                    [2, [2, 3]],
                    [3, [3, 4]],
                    [4, [4, 5]],
                    [5, [5, 6]],
                    [6, [6, 7]],
                    [7, [7, 8]],
                    [8, [8, 9]]
                ],
                [[0, [0, 1]], [1, [1, 2]]]
            ],
            [
                [
                    [
                        "+",
                        "current#plus",
                        "current",
                        4294967295u32,
                        0,
                        [0, null],
                        [1, 255],
                        [1, 2],
                        3
                    ],
                    ["P", "current#p", "current", 1, 1, [2, 4], null, [2, 3], 4]
                ],
                [
                    ["pre", "current", [0, 1], 1, [0, 0]],
                    ["left", "current", [1, 2], 2, [1, 1]],
                    ["right", "current", [2, 3], 3, [2, 2]],
                    ["non", "current", [3, 4], 4, [3, 3]],
                    ["post", "current", [4, 5], 5, [4, 4]]
                ]
            ],
            [
                [
                    [0],
                    "raw",
                    [0, 1],
                    [
                        ["range", [0, 1]],
                        ["point", 2],
                        ["generated", ["range", [1, 2]], " range β "],
                        ["generated", ["point", 3], " point "]
                    ],
                    [0]
                ],
                [[1, 0], "scope0", [0, 1], [], [0]],
                [[1, 1], "scope1", [0, 1], [], [0]],
                [[1, 2], "scope2", [0, 1], [], [0]],
                [[1, 3], "scope3", [0, 1], [], [0]],
                [[1, 4], "scope4", [0, 1], [], [0]],
                [[2, 0], "lex0", [0, 1], [], [1, "?", 0]],
                [
                    [2, 1],
                    "lex1",
                    [0, 1],
                    [["point", 7]],
                    [
                        2,
                        1,
                        "x",
                        [[
                            5,
                            "candidate",
                            [1, 2],
                            [["generated", ["point", 8], "candidate"]]
                        ]],
                        0
                    ]
                ],
                [[2, 2], "lex2", [0, 1], [], [3, "\"", [0], 0]],
                [[2, 3], "lex3", [0, 1], [], [3, "'", [1, "β"], 0]],
                [[2, 3], "lex3b", [0, 1], [], [3, "\"", [2], 0]],
                [[2, 4], "lex4", [0, 1], [], [4, 0, "run", 0]],
                [[2, 4], "lex4b", [0, 1], [], [4, 1, "number", 0]],
                [[2, 4], "lex4c", [0, 1], [], [4, 2, "@", 0]],
                [[2, 4], "lex4d", [0, 1], [], [4, 3, " ", 0]],
                [[2, 4], "lex4e", [0, 1], [], [4, 4, "?", 0]],
                [[2, 4], "unsupported", [0, 1], [], [5]]
            ]
        ]);
        let bytes = serde_json::to_vec(&value).unwrap();
        let stream = TokenStream::from_canonical_bytes(&bytes, source_id).unwrap();
        assert_eq!(stream.canonical_bytes().unwrap(), bytes);
        assert_eq!(stream.parser_context.mode(), ParserLexMode::General);
        assert_eq!(
            stream.parser_context.user_symbol_kinds(),
            UserSymbolKindSet::from_slice(&[
                UserSymbolKind::Functor,
                UserSymbolKind::Mode,
                UserSymbolKind::Constructor,
            ])
        );
        assert_eq!(
            stream
                .parser_lexing_plan
                .contexts
                .iter()
                .map(|entry| entry.context.mode())
                .collect::<Vec<_>>(),
            vec![
                ParserLexMode::IdentifierRequired,
                ParserLexMode::Symbolic,
                ParserLexMode::StringRequired,
                ParserLexMode::NamespacePath,
                ParserLexMode::Recovery,
                ParserLexMode::General
            ]
        );
        assert_eq!(
            stream
                .parser_lexing_plan
                .contexts
                .iter()
                .map(|entry| entry.context.user_symbol_kinds())
                .collect::<Vec<_>>(),
            vec![
                UserSymbolKindSet::empty(),
                UserSymbolKindSet::only(UserSymbolKind::Predicate),
                UserSymbolKindSet::only(UserSymbolKind::Mode),
                UserSymbolKindSet::only(UserSymbolKind::Attribute),
                UserSymbolKindSet::only(UserSymbolKind::Structure),
                UserSymbolKindSet::only(UserSymbolKind::Selector),
            ]
        );
        assert_eq!(
            stream
                .tokens
                .iter()
                .map(|token| token.kind)
                .collect::<Vec<_>>(),
            vec![
                TokenKind::Identifier,
                TokenKind::ReservedWord,
                TokenKind::ReservedSymbol,
                TokenKind::Numeral,
                TokenKind::LexemeRun,
                TokenKind::UserSymbol,
                TokenKind::AnnotationMarker,
                TokenKind::StringLiteral,
                TokenKind::ErrorRecovery,
            ]
        );
        assert_eq!(stream.scope_view.frames[0].bindings.len(), 14);
        assert_eq!(
            stream.scope_view.frames[0]
                .bindings
                .iter()
                .map(|binding| binding.kind)
                .collect::<Vec<_>>(),
            vec![
                BindingShapeKind::Let,
                BindingShapeKind::For,
                BindingShapeKind::Ex,
                BindingShapeKind::Reserve,
                BindingShapeKind::Given,
                BindingShapeKind::Consider,
                BindingShapeKind::Set,
                BindingShapeKind::Reconsider,
                BindingShapeKind::Take,
                BindingShapeKind::Deffunc,
                BindingShapeKind::Defpred,
                BindingShapeKind::Var,
                BindingShapeKind::Const,
                BindingShapeKind::Processed,
            ]
        );
        assert_eq!(
            stream
                .scope_view
                .blocks
                .iter()
                .map(|block| block.kind)
                .collect::<Vec<_>>(),
            vec![
                LexicalBlockKind::Algorithm,
                LexicalBlockKind::Definition,
                LexicalBlockKind::Registration,
                LexicalBlockKind::Proof,
                LexicalBlockKind::Now,
                LexicalBlockKind::Case,
                LexicalBlockKind::Suppose,
                LexicalBlockKind::Hereby,
                LexicalBlockKind::Do,
            ]
        );
        assert_eq!(
            stream
                .scope_view
                .statements
                .iter()
                .map(|statement| statement.kind)
                .collect::<Vec<_>>(),
            vec![LexicalStatementKind::Binder, LexicalStatementKind::Other]
        );
        assert_eq!(
            stream.local_declarations.user_symbols[0].symbol_id.as_str(),
            "current#plus"
        );
        assert_eq!(
            stream.local_declarations.user_symbols[0].export_rank.get(),
            u32::MAX
        );
        assert_eq!(
            stream
                .local_declarations
                .user_symbols
                .iter()
                .map(|symbol| symbol.kind)
                .collect::<Vec<_>>(),
            vec![UserSymbolKind::Functor, UserSymbolKind::Predicate]
        );
        assert_eq!(
            stream
                .local_declarations
                .operator_declarations
                .iter()
                .map(|entry| entry.operator.unwrap().fixity)
                .collect::<Vec<_>>(),
            vec![
                Fixity::Prefix,
                Fixity::Infix(Assoc::Left),
                Fixity::Infix(Assoc::Right),
                Fixity::Infix(Assoc::NonAssociative),
                Fixity::Postfix
            ]
        );
        assert_eq!(stream.diagnostics.len(), 17);
        assert_eq!(stream.diagnostics[0].kind, LexingDiagnosticKind::RawScan);
        assert_eq!(
            stream.diagnostics[1..6]
                .iter()
                .map(|diag| diag.kind)
                .collect::<Vec<_>>(),
            vec![
                LexingDiagnosticKind::ScopeSkeleton(
                    ScopeSkeletonDiagnosticCode::MalformedBinderList
                ),
                LexingDiagnosticKind::ScopeSkeleton(
                    ScopeSkeletonDiagnosticCode::UnsupportedBinderShape
                ),
                LexingDiagnosticKind::ScopeSkeleton(
                    ScopeSkeletonDiagnosticCode::DuplicateBindingName
                ),
                LexingDiagnosticKind::ScopeSkeleton(ScopeSkeletonDiagnosticCode::UnmatchedEnd),
                LexingDiagnosticKind::ScopeSkeleton(ScopeSkeletonDiagnosticCode::MissingEnd),
            ]
        );
        assert_eq!(
            [6, 7, 8, 9, 11].map(|index| stream.diagnostics[index].kind),
            [
                LexingDiagnosticKind::Lexer(LexDiagnosticCode::NoValidTokenCandidate),
                LexingDiagnosticKind::Lexer(LexDiagnosticCode::ParserContextRejectedCandidate),
                LexingDiagnosticKind::Lexer(LexDiagnosticCode::AmbiguousUserSymbol),
                LexingDiagnosticKind::Lexer(LexDiagnosticCode::MalformedStringLiteral),
                LexingDiagnosticKind::Lexer(LexDiagnosticCode::UnsupportedRawToken),
            ]
        );
        assert_eq!(
            stream.diagnostics[6].payload,
            LexingDiagnosticPayload::NoValidTokenCandidate {
                rejected_lexeme: Arc::from("?"),
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
        assert!(matches!(stream.diagnostics[7].payload,
            LexingDiagnosticPayload::ParserContextRejectedCandidate { ref candidates, .. }
            if candidates[0].secondary.len() == 1
        ));
        assert_eq!(stream.diagnostics[0].secondary.len(), 4);
        assert_eq!(
            stream.diagnostics[9].payload,
            LexingDiagnosticPayload::MalformedStringLiteral {
                opening_quote: '\'',
                reason: MalformedStringLiteralReason::UnsupportedEscape { escape: 'β' },
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
        assert_eq!(
            stream.diagnostics[8].payload,
            LexingDiagnosticPayload::MalformedStringLiteral {
                opening_quote: '"',
                reason: MalformedStringLiteralReason::MissingClosingQuote,
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
        assert_eq!(
            stream.diagnostics[10].payload,
            LexingDiagnosticPayload::MalformedStringLiteral {
                opening_quote: '"',
                reason: MalformedStringLiteralReason::DanglingEscape,
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
        assert_eq!(
            stream.diagnostics[11..16]
                .iter()
                .map(|diag| match &diag.payload {
                    LexingDiagnosticPayload::UnsupportedRawToken { raw_kind, .. } => *raw_kind,
                    _ => panic!("expected raw payload"),
                })
                .collect::<Vec<_>>(),
            vec![
                super::RawTokenKind::LexemeRun,
                super::RawTokenKind::NumeralLike,
                super::RawTokenKind::AnnotationMarker,
                super::RawTokenKind::Layout,
                super::RawTokenKind::Error,
            ]
        );
        assert_eq!(
            stream.diagnostics[16].payload,
            LexingDiagnosticPayload::UnsupportedLexerPayload
        );
        for (diagnostic, path, replacement) in [
            (9, "/4/1", json!("ab")),
            (9, "/4/2/1", json!("ab")),
            (9, "/4/2/0", json!(3)),
            (11, "/4/1", json!(5)),
        ] {
            let mut invalid = value.clone();
            *invalid[6][diagnostic].pointer_mut(path).unwrap() = replacement;
            assert!(
                TokenStream::from_canonical_bytes(
                    &serde_json::to_vec(&invalid).unwrap(),
                    source_id
                )
                .is_none()
            );
        }

        let ids = InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(3)).unwrap();
        let fresh_id = ids.next_source_id(snapshot_id(3)).unwrap();
        assert_ne!(source_id, fresh_id);
        let rebound = TokenStream::from_canonical_bytes(&bytes, fresh_id).unwrap();
        assert_eq!(rebound.source_id, fresh_id);
        assert_eq!(
            rebound.scope_view.frames[0].bindings[0]
                .introduced_at
                .source_id,
            fresh_id
        );
        assert_eq!(rebound.diagnostics[7].primary.source_id, fresh_id);
        assert_eq!(rebound.canonical_bytes().unwrap(), bytes);
    }

    #[test]
    fn token_stream_storage_rejects_corruption_and_foreign_ids() {
        use super::TokenStream;
        use serde_json::{Value, json};

        let source_id = source_unit("").source_id;
        let base = json!([
            "mizar-frontend/token-stream/v1",
            [0, [0, 2]],
            [[0, [0, 2]], [[[0, 1], [1, [1]]]]],
            [[0, "", [0, 1]]],
            [[[[0, 2], [["x", [0, 1], 0]]]], [[0, [0, 2]]], [[0, [0, 1]]]],
            [
                [[
                    "+",
                    "current#plus",
                    "current",
                    0,
                    0,
                    [1, 2],
                    [1, 80],
                    [0, 1],
                    1
                ]],
                [["+", "current", [0, 1], 1, [1, 80]]]
            ],
            [[
                [2, 1],
                "context",
                [0, 1],
                [
                    ["range", [0, 1]],
                    ["point", 2],
                    ["generated", ["range", [0, 1]], "r"],
                    ["generated", ["point", 2], "p"]
                ],
                [
                    2,
                    1,
                    "x",
                    [[
                        0,
                        "x",
                        [0, 1],
                        [
                            ["range", [0, 1]],
                            ["point", 2],
                            ["generated", ["range", [0, 1]], "r"],
                            ["generated", ["point", 2], "p"]
                        ]
                    ]],
                    0
                ]
            ]]
        ]);
        let bytes = serde_json::to_vec(&base).unwrap();
        let original = TokenStream::from_canonical_bytes(&bytes, source_id).unwrap();
        let reject = |value: &Value| {
            assert!(
                TokenStream::from_canonical_bytes(&serde_json::to_vec(value).unwrap(), source_id)
                    .is_none()
            );
        };
        for (pointer, replacement) in [
            ("/0", json!("unknown")),
            ("/1/0", json!(6)),
            ("/1/1", json!([2, 0])),
            ("/1/1", json!([0, 0])),
            ("/2/1/0/0", json!([2, 1])),
            ("/3/0/0", json!(9)),
            ("/3/0/2", json!([2, 1])),
            ("/4/0/0/0", json!([2, 1])),
            ("/4/0/0/1/0/2", json!(14)),
            ("/4/1/0/0", json!(9)),
            ("/4/2/0/0", json!(2)),
            ("/5/0/0/3", json!(4294967296u64)),
            ("/5/0/0/4", json!(7)),
            ("/5/0/0/5/0", json!(65536)),
            ("/5/0/0/6/0", json!(5)),
            ("/5/0/0/6/1", json!(256)),
            ("/5/0/0/7", json!([2, 1])),
            ("/6/0/0", json!([2, 5])),
            ("/6/0/3/0/0", json!("bad")),
            ("/6/0/3/1/1", json!(-1)),
            ("/6/0/3/2/1", json!(["generated", ["point", 0], "nested"])),
            ("/6/0/3/2/2", json!(" \t\n")),
            ("/6/0/4/3/0/3/0/0", json!("bad")),
            ("/6/0/4/3/0/3/2/2", json!("")),
            ("/6/0/4/4", json!(1)),
        ] {
            let mut invalid = base.clone();
            *invalid.pointer_mut(pointer).unwrap() = replacement;
            reject(&invalid);
        }
        for invalid in [
            json!(["mizar-frontend/token-stream/v1", [0, [0, 2]]]),
            json!([
                "mizar-frontend/token-stream/v1",
                [0, [0, 2]],
                [[0, [0, 2]], []],
                [],
                [[], [], []],
                [[], []],
                [],
                null
            ]),
        ] {
            reject(&invalid);
        }
        let mut invalid = base.clone();
        invalid[3][0].as_array_mut().unwrap().push(json!("extra"));
        reject(&invalid);
        let mut invalid = bytes.clone();
        invalid.insert(1, b' ');
        assert!(TokenStream::from_canonical_bytes(&invalid, source_id).is_none());

        let ids = InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(4)).unwrap();
        let foreign_id = ids.next_source_id(snapshot_id(4)).unwrap();
        assert_ne!(source_id, foreign_id);
        for field in 0..16 {
            let mut invalid = original.clone();
            match field {
                0 => invalid.scope_view.source_id = foreign_id,
                1 => invalid.tokens[0].span.source_id = foreign_id,
                2 => invalid.scope_view.frames[0].range.source_id = foreign_id,
                3 => {
                    invalid.scope_view.frames[0].bindings[0]
                        .introduced_at
                        .source_id = foreign_id
                }
                4 => invalid.scope_view.blocks[0].range.source_id = foreign_id,
                5 => invalid.scope_view.statements[0].range.source_id = foreign_id,
                6 => invalid.diagnostics[0].primary.source_id = foreign_id,
                7 => {
                    invalid.diagnostics[0].secondary[0] =
                        SourceAnchor::Range(range(foreign_id, 0, 1))
                }
                8 => {
                    invalid.diagnostics[0].secondary[1] = SourceAnchor::Point {
                        source_id: foreign_id,
                        offset: 2,
                    }
                }
                9 => {
                    invalid.diagnostics[0].payload =
                        LexingDiagnosticPayload::ParserContextRejectedCandidate {
                            mode: ParserLexMode::IdentifierRequired,
                            rejected_lexeme: Arc::from("x"),
                            candidates: vec![LexingRejectedTokenCandidate {
                                kind: TokenKind::Identifier,
                                text: Arc::from("x"),
                                span: range(foreign_id, 0, 1),
                                secondary: Vec::new(),
                            }],
                            recovery: LexRecoveryHint::EmitErrorRecoveryToken,
                        }
                }
                10 => {
                    invalid.diagnostics[0].secondary[2] = SourceAnchor::Generated(
                        mizar_session::GeneratedSpanOrigin::new(
                            mizar_session::GeneratedSpanAnchor::Range(range(foreign_id, 0, 1)),
                            "r",
                        )
                        .unwrap(),
                    )
                }
                11 => {
                    invalid.diagnostics[0].secondary[3] = SourceAnchor::Generated(
                        mizar_session::GeneratedSpanOrigin::new(
                            mizar_session::GeneratedSpanAnchor::Point {
                                source_id: foreign_id,
                                offset: 2,
                            },
                            "p",
                        )
                        .unwrap(),
                    )
                }
                12..=15 => {
                    let LexingDiagnosticPayload::ParserContextRejectedCandidate {
                        candidates, ..
                    } = &mut invalid.diagnostics[0].payload
                    else {
                        unreachable!()
                    };
                    candidates[0].secondary[field - 12] = match field {
                        12 => SourceAnchor::Range(range(foreign_id, 0, 1)),
                        13 => SourceAnchor::Point {
                            source_id: foreign_id,
                            offset: 2,
                        },
                        14 => SourceAnchor::Generated(
                            mizar_session::GeneratedSpanOrigin::new(
                                mizar_session::GeneratedSpanAnchor::Range(range(foreign_id, 0, 1)),
                                "r",
                            )
                            .unwrap(),
                        ),
                        15 => SourceAnchor::Generated(
                            mizar_session::GeneratedSpanOrigin::new(
                                mizar_session::GeneratedSpanAnchor::Point {
                                    source_id: foreign_id,
                                    offset: 2,
                                },
                                "p",
                            )
                            .unwrap(),
                        ),
                        _ => unreachable!(),
                    };
                }
                _ => unreachable!(),
            }
            assert!(invalid.canonical_bytes().is_none(), "foreign field {field}");
        }
        assert!(
            TokenStream::from_canonical_bytes(&vec![b' '; 16 * 1024 * 1024 + 1], source_id)
                .is_none()
        );
    }

    #[test]
    fn token_stream_storage_enforces_exact_payload_limit() {
        use super::TokenStream;
        let source_id = source_unit("").source_id;
        let value = serde_json::json!([
            "mizar-frontend/token-stream/v1",
            [0, []],
            [[0, []], []],
            [[0, "", [0, 0]]],
            [[], [], []],
            [[], []],
            []
        ]);
        let mut stream =
            TokenStream::from_canonical_bytes(&serde_json::to_vec(&value).unwrap(), source_id)
                .unwrap();
        let overhead = stream.canonical_bytes().unwrap().len();
        stream.tokens[0].text = Arc::from("x".repeat(16 * 1024 * 1024 - overhead));
        let exact = stream.canonical_bytes().unwrap();
        assert_eq!(exact.len(), 16 * 1024 * 1024);
        assert_eq!(
            TokenStream::from_canonical_bytes(&exact, source_id),
            Some(stream.clone())
        );
        stream.tokens[0].text = Arc::from(format!("{}x", stream.tokens[0].text));
        assert!(stream.canonical_bytes().is_none());
        let mut oversized = exact;
        let text_start = br#"["mizar-frontend/token-stream/v1",[0,[]],[[0,[]],[]],[[0,""#.len();
        oversized.insert(text_start, b'x');
        assert!(TokenStream::from_canonical_bytes(&oversized, source_id).is_none());
    }

    use super::{
        BindingShapeKind, LexDiagnosticCode, LexRecoveryHint, LexicalBlockKind, LexicalByteRange,
        LexicalStatementKind, LexingDiagnosticKind, LexingDiagnosticPayload,
        LexingRejectedTokenCandidate, MalformedStringLiteralReason, ParserLexContext,
        ParserLexMode, ParserLexingPlan, ParserLexingPlanContext, ScopeBlock, ScopeFrame,
        ScopeSkeletonDiagnosticCode, ScopeStatement, TokenKind, TokenizeRequest, tokenize,
    };
    use crate::preprocess::preprocess;
    use crate::source::{SourceUnit, register_source_unit};
    use crate::span_bridge::{LexerByteSpan, SpanBridge};
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
        let text = ".{.*.=.....";
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
                (
                    TokenKind::ReservedSymbol,
                    "..",
                    range(source.source_id, 9, 11)
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
    fn position_sensitive_plan_accepts_annotation_string_argument_unicode() {
        let text = "@[label(\"α::β\", \"γ::δ\")]\n";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();
        let plan = ParserLexingPlan::for_lexical_text(preprocessed.lexical_text.as_str());

        assert_eq!(preprocessed.lexical_text.as_str(), text);
        assert!(preprocessed.comments.is_empty());
        assert!(preprocessed.diagnostics.is_empty());
        assert_eq!(plan.contexts.len(), 2);
        assert_eq!(
            plan.contexts[0].range,
            LexicalByteRange::new(nth_index(text, "\"α", 0), nth_index(text, "\",", 0) + 1)
        );
        assert_eq!(
            plan.contexts[1].range,
            LexicalByteRange::new(nth_index(text, "\"γ", 0), nth_index(text, "\")]", 0) + 1)
        );

        let stream = tokenize(
            TokenizeRequest::with_plan(&preprocessed, &environment, plan.clone()),
            &bridge,
        )
        .unwrap();

        assert_eq!(stream.parser_lexing_plan, plan);
        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![
                (
                    TokenKind::ReservedSymbol,
                    "@[",
                    range(source.source_id, 0, 2)
                ),
                (
                    TokenKind::Identifier,
                    "label",
                    range(source.source_id, 2, 7)
                ),
                (
                    TokenKind::ReservedSymbol,
                    "(",
                    range(source.source_id, 7, 8)
                ),
                (
                    TokenKind::StringLiteral,
                    "\"α::β\"",
                    range(source.source_id, 8, 16)
                ),
                (
                    TokenKind::ReservedSymbol,
                    ",",
                    range(source.source_id, 16, 17)
                ),
                (
                    TokenKind::StringLiteral,
                    "\"γ::δ\"",
                    range(source.source_id, 18, 26)
                ),
                (
                    TokenKind::ReservedSymbol,
                    ")",
                    range(source.source_id, 26, 27)
                ),
                (
                    TokenKind::ReservedSymbol,
                    "]",
                    range(source.source_id, 27, 28)
                ),
            ]
        );
        assert!(stream.diagnostics.is_empty());
    }

    #[test]
    fn position_sensitive_plan_filters_user_symbol_kinds_by_range() {
        let text = "op op";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = environment_with_same_spelling_kind_overloads();
        let predicate_context = ParserLexContext::general().with_user_symbol_kinds(
            mizar_lexer::UserSymbolKindSet::only(mizar_lexer::UserSymbolKind::Predicate),
        );
        let mode_context = ParserLexContext::general().with_user_symbol_kinds(
            mizar_lexer::UserSymbolKindSet::only(mizar_lexer::UserSymbolKind::Mode),
        );
        let plan = ParserLexingPlan::new(
            predicate_context,
            vec![ParserLexingPlanContext::new(
                LexicalByteRange::new(3, 5),
                mode_context,
            )],
        );

        let stream = tokenize(
            TokenizeRequest::with_plan(&preprocessed, &environment, plan),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![
                (TokenKind::UserSymbol, "op", range(source.source_id, 0, 2)),
                (TokenKind::Identifier, "op", range(source.source_id, 3, 5)),
            ]
        );
        assert!(stream.diagnostics.is_empty());
    }

    #[test]
    fn position_sensitive_plan_raw_scan_recovery_uses_absolute_spans() {
        let text = "foo(\"α::β\")\n@ name";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();
        let plan = ParserLexingPlan::for_lexical_text(preprocessed.lexical_text.as_str());

        assert_eq!(plan.contexts.len(), 1);
        assert_eq!(
            plan.contexts[0].range,
            LexicalByteRange::new(nth_index(text, "\"α", 0), nth_index(text, "\")", 0) + 1)
        );

        let stream = tokenize(
            TokenizeRequest::with_plan(&preprocessed, &environment, plan),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![
                (TokenKind::Identifier, "foo", range(source.source_id, 0, 3)),
                (
                    TokenKind::ReservedSymbol,
                    "(",
                    range(source.source_id, 3, 4)
                ),
                (
                    TokenKind::StringLiteral,
                    "\"α::β\"",
                    range(source.source_id, 4, 12),
                ),
                (
                    TokenKind::ReservedSymbol,
                    ")",
                    range(source.source_id, 12, 13),
                ),
                (
                    TokenKind::ErrorRecovery,
                    "@",
                    range(source.source_id, 14, 15),
                ),
                (
                    TokenKind::Identifier,
                    "name",
                    range(source.source_id, 16, 20),
                ),
            ]
        );
        assert_eq!(stream.diagnostics.len(), 1);
        assert_eq!(stream.diagnostics[0].kind, LexingDiagnosticKind::RawScan);
        assert_eq!(
            stream.diagnostics[0].primary,
            range(source.source_id, 14, 15)
        );
        assert!(stream.diagnostics[0].message.contains("byte 14"));
    }

    #[test]
    fn planned_string_range_cannot_cross_line_boundary() {
        let text = "@[label(\"α\nβ\")]\n";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();
        let plan = ParserLexingPlan::new(
            ParserLexContext::general(),
            vec![ParserLexingPlanContext::new(
                LexicalByteRange::new(nth_index(text, "\"α", 0), nth_index(text, "\")]", 0) + 1),
                ParserLexContext::string_required(),
            )],
        );

        assert!(preprocessed.diagnostics.iter().any(|diagnostic| {
            diagnostic.kind
                == crate::preprocess::PreprocessDiagnosticKind::SourcePrecondition(
                    mizar_lexer::SourcePreprocessDiagnosticCode::NonAsciiCode,
                )
        }));

        let stream = tokenize(
            TokenizeRequest::with_plan(&preprocessed, &environment, plan),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![(
                TokenKind::ErrorRecovery,
                text,
                range(source.source_id, 0, text.len())
            )]
        );
        assert_eq!(stream.diagnostics.len(), 1);
        assert_eq!(stream.diagnostics[0].kind, LexingDiagnosticKind::RawScan);
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
    fn recoverable_malformed_lexeme_emits_error_recovery_and_resumes() {
        let text = "alpha?beta";
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
                    TokenKind::Identifier,
                    "alpha",
                    range(source.source_id, 0, 5)
                ),
                (TokenKind::ErrorRecovery, "?", range(source.source_id, 5, 6)),
                (
                    TokenKind::Identifier,
                    "beta",
                    range(source.source_id, 6, 10)
                ),
            ]
        );
        assert_eq!(stream.diagnostics.len(), 1);
        assert_eq!(
            stream.diagnostics[0].kind,
            LexingDiagnosticKind::Lexer(LexDiagnosticCode::NoValidTokenCandidate)
        );
        assert_eq!(stream.diagnostics[0].primary, range(source.source_id, 5, 6));
        assert_eq!(
            stream.diagnostics[0].payload,
            LexingDiagnosticPayload::NoValidTokenCandidate {
                rejected_lexeme: Arc::from("?"),
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
    }

    #[test]
    fn annotation_marker_tokenizes_in_general_context() {
        let text = "@latex alpha";
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
                    TokenKind::AnnotationMarker,
                    "@latex",
                    range(source.source_id, 0, 6)
                ),
                (
                    TokenKind::Identifier,
                    "alpha",
                    range(source.source_id, 7, 12)
                ),
            ]
        );
        assert_eq!(stream.diagnostics, Vec::new());
    }

    #[test]
    fn annotation_prefixed_registration_cluster_keeps_symbolic_attribute_tokens() {
        let text = "registration\n  @custom(flag)\n  cluster E: non empty set;\n  existence by Ref;\nend;\n";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = environment_with_imported_symbol("empty");

        let stream = tokenize(
            TokenizeRequest::with_plan(
                &preprocessed,
                &environment,
                ParserLexingPlan::for_lexical_text(preprocessed.lexical_text.as_str()),
            ),
            &bridge,
        )
        .unwrap();

        assert_eq!(stream.diagnostics, Vec::new());
        assert!(
            token_kinds_texts_and_spans(&stream)
                .windows(4)
                .any(|window| {
                    matches!(
                        window,
                        [
                            (TokenKind::ReservedWord, "non", _),
                            (TokenKind::UserSymbol, "empty", _),
                            (TokenKind::ReservedWord, "set", _),
                            (TokenKind::ReservedSymbol, ";", _),
                        ]
                    )
                }),
            "annotation-prefixed registration clusters must keep attribute tokens symbolic: {:?}",
            token_kinds_texts_and_spans(&stream)
        );
        assert!(
            token_kinds_texts_and_spans(&stream)
                .iter()
                .any(|(kind, text, span)| {
                    *kind == TokenKind::AnnotationMarker
                        && *text == "@custom"
                        && *span == range(source.source_id, 15, 22)
                })
        );
    }

    #[test]
    fn parser_context_rejected_annotation_marker_recovery_preserves_candidate_and_resumes() {
        let text = "@latex alpha";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(
                &preprocessed,
                &environment,
                ParserLexContext::identifier_required(),
            ),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![
                (
                    TokenKind::ErrorRecovery,
                    "@latex",
                    range(source.source_id, 0, 6)
                ),
                (
                    TokenKind::Identifier,
                    "alpha",
                    range(source.source_id, 7, 12)
                ),
            ]
        );
        assert_eq!(stream.diagnostics.len(), 1);
        assert_eq!(
            stream.diagnostics[0].kind,
            LexingDiagnosticKind::Lexer(LexDiagnosticCode::ParserContextRejectedCandidate)
        );
        assert_eq!(stream.diagnostics[0].primary, range(source.source_id, 0, 6));
        assert_eq!(
            stream.diagnostics[0].payload,
            LexingDiagnosticPayload::ParserContextRejectedCandidate {
                mode: ParserLexMode::IdentifierRequired,
                rejected_lexeme: Arc::from("@latex"),
                candidates: vec![LexingRejectedTokenCandidate {
                    kind: TokenKind::AnnotationMarker,
                    text: Arc::from("@latex"),
                    span: range(source.source_id, 0, 6),
                    secondary: Vec::new(),
                }],
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
    }

    #[test]
    fn parser_context_rejected_numeral_recovery_preserves_candidate_and_resumes() {
        let text = "123 alpha";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(
                &preprocessed,
                &environment,
                ParserLexContext::identifier_required(),
            ),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![
                (
                    TokenKind::ErrorRecovery,
                    "123",
                    range(source.source_id, 0, 3)
                ),
                (
                    TokenKind::Identifier,
                    "alpha",
                    range(source.source_id, 4, 9)
                ),
            ]
        );
        assert_eq!(stream.diagnostics.len(), 1);
        assert_eq!(
            stream.diagnostics[0].kind,
            LexingDiagnosticKind::Lexer(LexDiagnosticCode::ParserContextRejectedCandidate)
        );
        assert_eq!(
            stream.diagnostics[0].payload,
            LexingDiagnosticPayload::ParserContextRejectedCandidate {
                mode: ParserLexMode::IdentifierRequired,
                rejected_lexeme: Arc::from("123"),
                candidates: vec![LexingRejectedTokenCandidate {
                    kind: TokenKind::Numeral,
                    text: Arc::from("123"),
                    span: range(source.source_id, 0, 3),
                    secondary: Vec::new(),
                }],
                recovery: LexRecoveryHint::EmitErrorRecoveryToken,
            }
        );
    }

    #[test]
    fn scope_diagnostics_survive_recoverable_lexer_errors_after_disambiguation() {
        let text = "definition\n?\nalpha";
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
                    TokenKind::ReservedWord,
                    "definition",
                    range(source.source_id, 0, 10)
                ),
                (
                    TokenKind::ErrorRecovery,
                    "?",
                    range(source.source_id, 11, 12)
                ),
                (
                    TokenKind::Identifier,
                    "alpha",
                    range(source.source_id, 13, 18)
                ),
            ]
        );
        assert_eq!(stream.diagnostics.len(), 2);
        assert_eq!(
            stream.diagnostics[0].kind,
            LexingDiagnosticKind::ScopeSkeleton(ScopeSkeletonDiagnosticCode::MissingEnd)
        );
        assert_eq!(stream.diagnostics[0].primary, range(source.source_id, 0, 0));
        assert_eq!(
            stream.diagnostics[1].kind,
            LexingDiagnosticKind::Lexer(LexDiagnosticCode::NoValidTokenCandidate)
        );
        assert_eq!(
            stream.diagnostics[1].primary,
            range(source.source_id, 11, 12)
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
    fn scope_skeleton_unnamed_take_term_is_not_a_frontend_diagnostic() {
        const TEXT: &str = "proof\ntake 101;\nend;\n";
        let (source, preprocessed, bridge) = preprocessed_source(TEXT);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(TEXT.len(), 21);
        assert_eq!(
            stream.scope_view.blocks,
            vec![ScopeBlock {
                kind: LexicalBlockKind::Proof,
                range: range(source.source_id, 0, 19),
            }]
        );
        assert_eq!(
            stream.scope_view.frames,
            vec![ScopeFrame {
                range: range(source.source_id, 0, 19),
                bindings: Vec::new(),
            }]
        );
        assert!(stream.scope_view.statements.is_empty());
        assert!(
            stream.diagnostics.iter().all(|diagnostic| !matches!(
                diagnostic.kind,
                LexingDiagnosticKind::ScopeSkeleton(_)
            ))
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
            parser_lexing_plan: _,
            tokens: _,
            scope_view: _,
            local_declarations: _,
            diagnostics,
        } = stream;
        let _: Vec<super::LexingDiagnostic> = diagnostics;
    }

    #[test]
    fn tokenization_collects_local_declarations_across_preprocessed_comments() {
        let text = concat!(
            "func Plus: x + y -> set; :: comment before operator\n",
            "infix_operator(\"+\", left, 80); :: comment before use\n",
            "a + b;\n"
        );
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general())
                .with_current_module(mizar_lexer::ModuleId::new("current")),
            &bridge,
        )
        .unwrap();

        let operator = stream
            .local_declarations
            .operator_declarations
            .first()
            .expect("operator declaration should be collected");
        let mapped_activation = bridge
            .lexical_span(
                source.source_id,
                LexerByteSpan {
                    start: operator.activation_start,
                    end: operator.activation_start,
                },
            )
            .expect("activation point should map to source coordinates");
        assert!(
            mapped_activation.primary.start > operator.activation_start,
            "removed comments before the use should make source and lexical offsets differ"
        );
        assert!(
            stream.tokens.iter().any(|token| {
                token.kind == TokenKind::UserSymbol
                    && token.text.as_ref() == "+"
                    && token.span.start == nth_index(text, "+", 2)
            }),
            "local functor spelling should be active at the later use"
        );
    }

    #[test]
    fn raw_scan_recovery_returns_precise_token_and_continues() {
        let text = "@ name";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(stream.scope_view.frames, Vec::new());
        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![
                (TokenKind::ErrorRecovery, "@", range(source.source_id, 0, 1)),
                (TokenKind::Identifier, "name", range(source.source_id, 2, 6)),
            ]
        );
        assert_eq!(stream.diagnostics.len(), 1);
        assert_eq!(stream.diagnostics[0].kind, LexingDiagnosticKind::RawScan);
        assert!(
            stream.diagnostics[0]
                .message
                .starts_with("raw scan recovered after raw scan error:")
        );
        assert_eq!(stream.diagnostics[0].primary, range(source.source_id, 0, 1));
        assert_eq!(stream.diagnostics[0].payload, LexingDiagnosticPayload::None);
    }

    #[test]
    fn raw_scan_recovery_preserves_partial_tokens_after_removed_comment() {
        let text = "@::=hidden=::beta";
        let (source, preprocessed, bridge) = preprocessed_source(text);
        let environment = empty_environment();

        assert_eq!(preprocessed.lexical_text.as_str(), "@ beta");

        let stream = tokenize(
            TokenizeRequest::new(&preprocessed, &environment, ParserLexContext::general()),
            &bridge,
        )
        .unwrap();

        assert_eq!(
            token_kinds_texts_and_spans(&stream),
            vec![
                (TokenKind::ErrorRecovery, "@", range(source.source_id, 0, 1)),
                (
                    TokenKind::Identifier,
                    "beta",
                    range(
                        source.source_id,
                        nth_index(text, "beta", 0),
                        nth_index(text, "beta", 0) + "beta".len(),
                    ),
                ),
            ]
        );
        assert_eq!(stream.diagnostics.len(), 1);
        assert_eq!(stream.diagnostics[0].kind, LexingDiagnosticKind::RawScan);
        assert_eq!(stream.diagnostics[0].primary, range(source.source_id, 0, 1));
        assert!(stream.diagnostics[0].secondary.is_empty());
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
                        operator: None,
                    })
                    .collect(),
                fingerprint: mizar_lexer::LexicalSummaryFingerprint::new(11),
            }],
        )
        .unwrap()
    }

    fn environment_with_same_spelling_kind_overloads() -> mizar_lexer::ActiveLexicalEnvironment {
        let module = mizar_lexer::ModuleId::new("imported.kind_overloads");
        mizar_lexer::build_lexical_environment(
            &[mizar_lexer::ResolvedImport {
                module_id: module.clone(),
            }],
            &[mizar_lexer::ModuleLexicalSummary {
                module_id: module.clone(),
                exported_symbols: vec![
                    mizar_lexer::ExportedSymbolShape {
                        spelling: "op".to_owned(),
                        symbol_id: mizar_lexer::SymbolId::new("imported.kind_overloads#predicate"),
                        source_module: module.clone(),
                        export_rank: mizar_lexer::ExportRank::new(0),
                        kind: mizar_lexer::UserSymbolKind::Predicate,
                        arity: mizar_lexer::UserSymbolArity::exact(2),
                        operator: None,
                    },
                    mizar_lexer::ExportedSymbolShape {
                        spelling: "op".to_owned(),
                        symbol_id: mizar_lexer::SymbolId::new("imported.kind_overloads#functor"),
                        source_module: module.clone(),
                        export_rank: mizar_lexer::ExportRank::new(1),
                        kind: mizar_lexer::UserSymbolKind::Functor,
                        arity: mizar_lexer::UserSymbolArity::exact(1),
                        operator: None,
                    },
                ],
                fingerprint: mizar_lexer::LexicalSummaryFingerprint::new(17),
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
