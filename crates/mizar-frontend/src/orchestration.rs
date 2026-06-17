//! End-to-end frontend orchestration for source-to-syntax processing.
//!
//! Canonical behavior is specified in the
//! [orchestration design spec](../../../../doc/design/mizar-frontend/en/orchestration.md).

use crate::cache_key::{
    ActiveLexicalEnvironmentCacheKey, FrontendCacheKeys, ParserLexingPlanCacheKey,
    PreprocessedSourceCacheKey, SourceUnitCacheKey, SurfaceAstCacheKey, TokenStreamCacheKey,
};
use crate::lexical_env::{
    FrontendLexicalEnvironmentError, LexicalEnvironmentDiagnostic,
    LexicalEnvironmentDiagnosticCode, LexicalEnvironmentRequest, LexicalSummaryProvider,
    build_active_lexical_environment,
};
use crate::lexing::{
    LexRecoveryHint, LexingDiagnostic, LexingDiagnosticKind, LexingDiagnosticPayload, TokenStream,
    TokenizeRequest, tokenize,
};
use crate::parsing::{ParseRequest, ParserInputs, ParserSeam};
use crate::preprocess::{PreprocessDiagnostic, PreprocessDiagnosticKind, PreprocessedSource};
use crate::source::{SourceUnit, SourceUnitLoader, SourceUnitRequest, register_source_unit};
use crate::span_bridge::{SpanBridge, SpanBridgeError};
use mizar_session::{
    DocumentUri, NormalizedPath, SessionIdAllocator, SourceAnchor, SourceInput, SourceLoadError,
    SourceOriginInput, SourceRange,
};
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

/// Complete frontend output for one source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendOutput<A> {
    /// Loaded source unit.
    pub source: SourceUnit,
    /// Preprocessed source and import stubs.
    pub preprocessed: PreprocessedSource,
    /// Token stream produced from the preprocessed source.
    pub tokens: TokenStream,
    /// Parser AST, absent when the configured parser produced no AST.
    pub ast: Option<A>,
    /// Recoverable diagnostics merged across frontend phases.
    pub diagnostics: Vec<FrontendDiagnostic>,
    /// Layered content cache keys for the output.
    pub cache_keys: FrontendCacheKeys,
}

/// Frontend coordinator parameterized by loader, provider, and parser seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontend<L, P, PS>
where
    L: SourceUnitLoader,
    P: LexicalSummaryProvider,
    PS: ParserSeam,
{
    loader: L,
    provider: P,
    parser: PS,
}

impl<L, P, PS> Frontend<L, P, PS>
where
    L: SourceUnitLoader,
    P: LexicalSummaryProvider,
    PS: ParserSeam,
    PS::Diagnostic: FrontendParserDiagnostic,
{
    /// Creates a frontend coordinator.
    pub fn new(loader: L, provider: P, parser: PS) -> Self {
        Self {
            loader,
            provider,
            parser,
        }
    }

    /// Runs the frontend pipeline for one source request.
    pub fn run(
        &self,
        request: SourceUnitRequest,
        ids: &dyn SessionIdAllocator,
    ) -> Result<FrontendOutput<PS::Ast>, FrontendError> {
        let source_load_input = request.input.clone();
        let source = self
            .loader
            .load_source_unit(request, ids)
            .map_err(|source| FrontendError::SourceLoad {
                diagnostic: Box::new(source_load_diagnostic(&source_load_input, &source)),
                source: Box::new(source),
            })?;

        let mut bridge = SpanBridge::new();
        register_source_unit(&mut bridge, &source)
            .map_err(|source| FrontendError::SpanBridge { source })?;
        let preprocessed = crate::preprocess::preprocess(&source, &mut bridge)
            .map_err(|source| FrontendError::SpanBridge { source })?;
        let lexical_environment = build_active_lexical_environment(
            &LexicalEnvironmentRequest {
                source_id: source.source_id,
                import_stubs: &preprocessed.import_stubs,
                edition: source.edition.clone(),
            },
            &self.provider,
        )
        .map_err(|source| FrontendError::LexicalEnvironment { source })?;

        let parser_inputs = ParserInputs::from_active_environment(
            source.edition.clone(),
            &lexical_environment.environment,
        );
        let parser_lexing_plan = parser_inputs
            .string_required_positions
            .parser_lexing_plan(preprocessed.lexical_text.as_str());
        let tokens = tokenize(
            TokenizeRequest::with_plan(
                &preprocessed,
                &lexical_environment.environment,
                parser_lexing_plan,
            ),
            &bridge,
        )
        .map_err(|source| FrontendError::SpanBridge { source })?;
        let token_cache_key = TokenStreamCacheKey::new(
            &preprocessed,
            lexical_environment.fingerprint,
            tokens.parser_context,
            ParserLexingPlanCacheKey::from_plan(&tokens.parser_lexing_plan),
        );
        let token_stream_hash = token_cache_key.stable_hash();
        let parser_cache_key_version = self.parser.cache_key_version();
        let parser_inputs_for_cache = parser_inputs.clone();
        let parser_output = self.parser.parse(ParseRequest::new(&tokens, parser_inputs));
        let ast_cache_key = parser_output.ast.as_ref().map(|_| {
            SurfaceAstCacheKey::new(
                token_stream_hash,
                parser_cache_key_version,
                &parser_inputs_for_cache,
            )
        });

        let diagnostics = merge_phase_diagnostics(
            &preprocessed.diagnostics,
            &lexical_environment.diagnostics,
            &tokens.diagnostics,
            parser_output.diagnostics,
        );
        let cache_keys = FrontendCacheKeys {
            source: SourceUnitCacheKey::from_source(&source),
            preprocessed: PreprocessedSourceCacheKey::from_source(&source),
            active_lexical_environment: ActiveLexicalEnvironmentCacheKey::new(
                lexical_environment.fingerprint,
            ),
            tokens: token_cache_key,
            ast: ast_cache_key,
        };

        Ok(FrontendOutput {
            source,
            preprocessed,
            tokens,
            ast: parser_output.ast,
            diagnostics,
            cache_keys,
        })
    }
}

/// Recoverable frontend diagnostic with source or load-location context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendDiagnostic {
    /// Stable diagnostic code.
    pub code: DiagnosticCode,
    /// Human-readable diagnostic message.
    pub message: Arc<str>,
    /// Diagnostic class used for deterministic merge ordering.
    pub class: DiagnosticClass,
    /// Primary diagnostic location.
    pub location: DiagnosticLocation,
    /// Secondary source anchors for related context.
    pub secondary: Vec<SourceAnchor>,
    /// Optional recovery note from lower phases.
    pub recovery_note: Option<String>,
}

/// Location for a frontend diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLocation {
    /// Diagnostic points at a source range.
    SourceRange(SourceRange),
    /// Diagnostic points at source-loading metadata.
    SourceLoad(SourceLoadLocation),
}

/// Source-loading location used when no loaded source range exists.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceLoadLocation {
    /// Filesystem path location.
    Path {
        /// Filesystem path.
        path: PathBuf,
    },
    /// Normalized source path location.
    NormalizedPath {
        /// Normalized source path.
        path: NormalizedPath,
    },
    /// Open-buffer document URI location.
    OpenBuffer {
        /// Open-buffer document URI.
        uri: DocumentUri,
    },
    /// Generated source location with an optional anchor.
    Generated {
        /// Optional generated-source anchor.
        anchor: Option<SourceAnchor>,
    },
    /// No stable load location is available.
    Unknown,
}

/// Stable frontend diagnostic code.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticCode {
    /// Source-loading diagnostic.
    SourceLoad,
    /// Preprocessing diagnostic.
    Preprocess(PreprocessDiagnosticKind),
    /// Lexical-environment diagnostic.
    LexicalEnvironment(LexicalEnvironmentDiagnosticCode),
    /// Lexing diagnostic.
    Lexing(LexingDiagnosticKind),
    /// Parser syntax diagnostic code.
    Syntax(Arc<str>),
}

/// Frontend diagnostic class used for grouping and ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticClass {
    /// Source-loading failure.
    SourceLoad,
    /// Lexical source precondition issue.
    LexicalPrecondition,
    /// Comment structure issue.
    CommentStructure,
    /// Import pre-scan issue.
    ImportPrescan,
    /// Lexical-environment issue.
    LexicalEnvironment,
    /// Scope skeleton issue.
    ScopeSkeleton,
    /// Tokenization issue.
    Tokenization,
    /// Parser syntax issue.
    Syntax,
    /// Annotation syntax issue.
    AnnotationSyntax,
}

/// Unrecoverable frontend pipeline error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FrontendError {
    /// Source loading failed before a source unit was available.
    SourceLoad {
        /// Underlying source loading error.
        source: Box<SourceLoadError>,
        /// Diagnostic describing the source loading failure.
        diagnostic: Box<FrontendDiagnostic>,
    },
    /// Span conversion or source-map registration failed.
    SpanBridge {
        /// Underlying span bridge error.
        source: SpanBridgeError,
    },
    /// Active lexical-environment construction failed unrecoverably.
    LexicalEnvironment {
        /// Underlying lexical-environment error.
        source: FrontendLexicalEnvironmentError,
    },
}

/// Converts parser diagnostics into merged frontend diagnostics.
pub trait FrontendParserDiagnostic {
    /// Converts one parser diagnostic, or drops it when the seam has no diagnostics.
    fn into_frontend_diagnostic(self) -> Option<FrontendDiagnostic>;
}

impl FrontendParserDiagnostic for () {
    fn into_frontend_diagnostic(self) -> Option<FrontendDiagnostic> {
        None
    }
}

impl FrontendParserDiagnostic for mizar_syntax::SyntaxDiagnostic {
    fn into_frontend_diagnostic(self) -> Option<FrontendDiagnostic> {
        Some(FrontendDiagnostic {
            code: DiagnosticCode::Syntax(syntax_code_key(&self.code)),
            message: self.message,
            class: DiagnosticClass::Syntax,
            location: DiagnosticLocation::SourceRange(self.primary),
            secondary: self.secondary,
            recovery_note: self.recovery_note.map(|note| note.to_string()),
        })
    }
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceLoad { source, .. } => {
                write!(f, "frontend source load failed: {source}")
            }
            Self::SpanBridge { source } => {
                write!(f, "frontend span bridge failed: {source}")
            }
            Self::LexicalEnvironment { source } => {
                write!(f, "frontend lexical environment failed: {source}")
            }
        }
    }
}

impl Error for FrontendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SourceLoad { source, .. } => Some(source),
            Self::SpanBridge { source } => Some(source),
            Self::LexicalEnvironment { source } => Some(source),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CollectedDiagnostic {
    diagnostic: FrontendDiagnostic,
    emission_ordinal: usize,
}

fn merge_phase_diagnostics<D>(
    preprocess: &[PreprocessDiagnostic],
    lexical_environment: &[LexicalEnvironmentDiagnostic],
    lexing: &[LexingDiagnostic],
    syntax: Vec<D>,
) -> Vec<FrontendDiagnostic>
where
    D: FrontendParserDiagnostic,
{
    let mut diagnostics = Vec::new();
    diagnostics.extend(
        preprocess
            .iter()
            .enumerate()
            .map(|(emission_ordinal, diagnostic)| CollectedDiagnostic {
                diagnostic: frontend_preprocess_diagnostic(diagnostic),
                emission_ordinal,
            }),
    );
    diagnostics.extend(lexical_environment.iter().enumerate().map(
        |(emission_ordinal, diagnostic)| CollectedDiagnostic {
            diagnostic: frontend_lexical_environment_diagnostic(diagnostic),
            emission_ordinal,
        },
    ));
    diagnostics.extend(
        lexing
            .iter()
            .enumerate()
            .map(|(emission_ordinal, diagnostic)| CollectedDiagnostic {
                diagnostic: frontend_lexing_diagnostic(diagnostic),
                emission_ordinal,
            }),
    );
    diagnostics.extend(syntax.into_iter().enumerate().filter_map(
        |(emission_ordinal, diagnostic)| {
            diagnostic
                .into_frontend_diagnostic()
                .map(|diagnostic| CollectedDiagnostic {
                    diagnostic,
                    emission_ordinal,
                })
        },
    ));

    sort_collected_diagnostics(diagnostics)
}

fn sort_collected_diagnostics(
    mut diagnostics: Vec<CollectedDiagnostic>,
) -> Vec<FrontendDiagnostic> {
    diagnostics.sort_by(compare_collected_diagnostics);
    diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.diagnostic)
        .collect()
}

fn compare_collected_diagnostics(
    left: &CollectedDiagnostic,
    right: &CollectedDiagnostic,
) -> Ordering {
    class_rank(left.diagnostic.class)
        .cmp(&class_rank(right.diagnostic.class))
        .then_with(|| {
            location_primary_key(&left.diagnostic.location)
                .cmp(&location_primary_key(&right.diagnostic.location))
        })
        .then_with(|| {
            diagnostic_code_key(&left.diagnostic.code)
                .cmp(&diagnostic_code_key(&right.diagnostic.code))
        })
        .then_with(|| {
            left.diagnostic
                .message
                .as_ref()
                .cmp(right.diagnostic.message.as_ref())
        })
        .then_with(|| {
            secondary_key(&left.diagnostic.secondary)
                .cmp(&secondary_key(&right.diagnostic.secondary))
        })
        .then_with(|| {
            left.diagnostic
                .recovery_note
                .cmp(&right.diagnostic.recovery_note)
        })
        .then_with(|| left.emission_ordinal.cmp(&right.emission_ordinal))
}

fn frontend_preprocess_diagnostic(diagnostic: &PreprocessDiagnostic) -> FrontendDiagnostic {
    FrontendDiagnostic {
        code: DiagnosticCode::Preprocess(diagnostic.kind),
        message: diagnostic.message.clone(),
        class: preprocess_class(diagnostic.kind),
        location: DiagnosticLocation::SourceRange(diagnostic.primary),
        secondary: diagnostic.secondary.clone(),
        recovery_note: None,
    }
}

fn preprocess_class(kind: PreprocessDiagnosticKind) -> DiagnosticClass {
    match kind {
        PreprocessDiagnosticKind::SourcePrecondition(code) => match code {
            crate::preprocess::SourcePreprocessDiagnosticCode::UnterminatedMultiLineComment => {
                DiagnosticClass::CommentStructure
            }
            _ => DiagnosticClass::LexicalPrecondition,
        },
        PreprocessDiagnosticKind::ImportPrescan(_) | PreprocessDiagnosticKind::RawImportScan => {
            DiagnosticClass::ImportPrescan
        }
    }
}

fn frontend_lexical_environment_diagnostic(
    diagnostic: &LexicalEnvironmentDiagnostic,
) -> FrontendDiagnostic {
    FrontendDiagnostic {
        code: DiagnosticCode::LexicalEnvironment(diagnostic.code),
        message: diagnostic.message.clone(),
        class: DiagnosticClass::LexicalEnvironment,
        location: DiagnosticLocation::SourceRange(diagnostic.primary),
        secondary: diagnostic.secondary.clone(),
        recovery_note: None,
    }
}

fn frontend_lexing_diagnostic(diagnostic: &LexingDiagnostic) -> FrontendDiagnostic {
    FrontendDiagnostic {
        code: DiagnosticCode::Lexing(diagnostic.kind),
        message: diagnostic.message.clone(),
        class: lexing_class(diagnostic.kind),
        location: DiagnosticLocation::SourceRange(diagnostic.primary),
        secondary: diagnostic.secondary.clone(),
        recovery_note: lexing_recovery_note(&diagnostic.payload),
    }
}

fn lexing_class(kind: LexingDiagnosticKind) -> DiagnosticClass {
    match kind {
        LexingDiagnosticKind::RawScan | LexingDiagnosticKind::Lexer(_) => {
            DiagnosticClass::Tokenization
        }
        LexingDiagnosticKind::ScopeSkeleton(_) => DiagnosticClass::ScopeSkeleton,
    }
}

fn lexing_recovery_note(payload: &LexingDiagnosticPayload) -> Option<String> {
    match payload {
        LexingDiagnosticPayload::NoValidTokenCandidate { recovery, .. }
        | LexingDiagnosticPayload::ParserContextRejectedCandidate { recovery, .. }
        | LexingDiagnosticPayload::MalformedStringLiteral { recovery, .. }
        | LexingDiagnosticPayload::UnsupportedRawToken { recovery, .. } => {
            Some(lex_recovery_note(*recovery).to_owned())
        }
        LexingDiagnosticPayload::None | LexingDiagnosticPayload::UnsupportedLexerPayload => None,
    }
}

fn lex_recovery_note(recovery: LexRecoveryHint) -> &'static str {
    match recovery {
        LexRecoveryHint::EmitErrorRecoveryToken => "emitted an error-recovery token",
        _ => "applied lexer recovery",
    }
}

fn source_load_diagnostic(input: &SourceInput, source: &SourceLoadError) -> FrontendDiagnostic {
    FrontendDiagnostic {
        code: DiagnosticCode::SourceLoad,
        message: Arc::<str>::from(source.to_string()),
        class: DiagnosticClass::SourceLoad,
        location: DiagnosticLocation::SourceLoad(source_load_location(input)),
        secondary: Vec::new(),
        recovery_note: None,
    }
}

fn source_load_location(input: &SourceInput) -> SourceLoadLocation {
    match &input.origin {
        SourceOriginInput::Disk { path } => SourceLoadLocation::Path { path: path.clone() },
        SourceOriginInput::OpenBuffer { uri, .. } => {
            SourceLoadLocation::OpenBuffer { uri: uri.clone() }
        }
        SourceOriginInput::Generated { anchor, .. } => SourceLoadLocation::Generated {
            anchor: anchor.clone(),
        },
        _ => SourceLoadLocation::NormalizedPath {
            path: input.normalized_path.clone(),
        },
    }
}

fn class_rank(class: DiagnosticClass) -> u8 {
    match class {
        DiagnosticClass::SourceLoad => 0,
        DiagnosticClass::LexicalPrecondition => 1,
        DiagnosticClass::CommentStructure => 2,
        DiagnosticClass::ImportPrescan => 3,
        DiagnosticClass::LexicalEnvironment => 4,
        DiagnosticClass::ScopeSkeleton => 5,
        DiagnosticClass::Tokenization => 6,
        DiagnosticClass::Syntax | DiagnosticClass::AnnotationSyntax => 7,
    }
}

fn location_primary_key(location: &DiagnosticLocation) -> String {
    match location {
        DiagnosticLocation::SourceRange(range) => {
            format!(
                "{:?}:{:020}:{:020}",
                range.source_id, range.start, range.end
            )
        }
        DiagnosticLocation::SourceLoad(location) => source_load_location_key(location),
    }
}

fn source_load_location_key(location: &SourceLoadLocation) -> String {
    match location {
        SourceLoadLocation::Path { path } => format!("path:{}", path.display()),
        SourceLoadLocation::NormalizedPath { path } => {
            format!("normalized:{}", path.as_str())
        }
        SourceLoadLocation::OpenBuffer { uri } => format!("open-buffer:{uri}"),
        SourceLoadLocation::Generated { anchor } => {
            format!(
                "generated:{}",
                anchor.as_ref().map(source_anchor_key).unwrap_or_default()
            )
        }
        SourceLoadLocation::Unknown => "unknown".to_owned(),
    }
}

fn diagnostic_code_key(code: &DiagnosticCode) -> String {
    match code {
        DiagnosticCode::SourceLoad => "source_load".to_owned(),
        DiagnosticCode::Preprocess(kind) => format!("preprocess:{kind:?}"),
        DiagnosticCode::LexicalEnvironment(code) => {
            format!("lexical_environment:{code:?}")
        }
        DiagnosticCode::Lexing(kind) => format!("lexing:{kind:?}"),
        DiagnosticCode::Syntax(code) => format!("syntax:{code}"),
    }
}

fn secondary_key(secondary: &[SourceAnchor]) -> Vec<String> {
    secondary.iter().map(source_anchor_key).collect()
}

fn source_anchor_key(anchor: &SourceAnchor) -> String {
    match anchor {
        SourceAnchor::Range(range) => {
            format!(
                "range:{:?}:{:020}:{:020}",
                range.source_id, range.start, range.end
            )
        }
        SourceAnchor::Point { source_id, offset } => {
            format!("point:{source_id:?}:{offset:020}")
        }
        SourceAnchor::Generated(origin) => format!("generated:{origin:?}"),
        _ => format!("unknown:{anchor:?}"),
    }
}

fn syntax_code_key(code: &mizar_syntax::SyntaxDiagnosticCode) -> Arc<str> {
    Arc::<str>::from(match code {
        mizar_syntax::SyntaxDiagnosticCode::UnexpectedErrorToken => "unexpected_error_token",
        mizar_syntax::SyntaxDiagnosticCode::DanglingOperator => "dangling_operator",
        mizar_syntax::SyntaxDiagnosticCode::NonAssociativeOperatorChain => {
            "non_associative_operator_chain"
        }
        mizar_syntax::SyntaxDiagnosticCode::MissingEnd => "missing_end",
        mizar_syntax::SyntaxDiagnosticCode::MissingSemicolon => "missing_semicolon",
        mizar_syntax::SyntaxDiagnosticCode::MissingStringLiteral => "missing_string_literal",
        mizar_syntax::SyntaxDiagnosticCode::MalformedImport => "malformed_import",
        mizar_syntax::SyntaxDiagnosticCode::MalformedExport => "malformed_export",
        mizar_syntax::SyntaxDiagnosticCode::MalformedVisibility => "malformed_visibility",
        mizar_syntax::SyntaxDiagnosticCode::MalformedTypeExpression => "malformed_type_expression",
        mizar_syntax::SyntaxDiagnosticCode::MalformedTermExpression => "malformed_term_expression",
        mizar_syntax::SyntaxDiagnosticCode::MalformedFormulaExpression => {
            "malformed_formula_expression"
        }
        mizar_syntax::SyntaxDiagnosticCode::MalformedJustification => "malformed_justification",
        mizar_syntax::SyntaxDiagnosticCode::MalformedAnnotation => "malformed_annotation",
        mizar_syntax::SyntaxDiagnosticCode::UnexpectedTopLevelToken => "unexpected_top_level_token",
        mizar_syntax::SyntaxDiagnosticCode::UnrecoverableInput => "unrecoverable_input",
        _ => "syntax_diagnostic",
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CollectedDiagnostic, DiagnosticClass, DiagnosticCode, DiagnosticLocation, Frontend,
        FrontendDiagnostic, FrontendError, SourceLoadLocation, compare_collected_diagnostics,
        sort_collected_diagnostics,
    };
    use crate::lexical_env::{
        LexicalEnvironmentDiagnostic, LexicalEnvironmentDiagnosticCode, LexicalEnvironmentRequest,
        LexicalSummaryProvider, ResolvedImports,
    };
    use crate::lexing::{
        LexDiagnosticCode, LexingDiagnostic, LexingDiagnosticKind, LexingDiagnosticPayload,
        ParserLexMode, TokenKind,
    };
    use crate::parsing::{
        MIZAR_PARSER_CACHE_KEY_VERSION, MizarParserSeam, ParseOutput, ParseRequest, ParserSeam,
        StubParserSeam,
    };
    use crate::source::{FrontendSourceLoader, SourceUnit, SourceUnitLoader, SourceUnitRequest};
    use crate::span_bridge::SpanBridgeError;
    use mizar_session::{
        BuildSnapshotId, DiskSourceLoader, Edition, GeneratedSourceKind,
        InMemorySessionIdAllocator, LineMap, ModulePath, PackageId, SessionIdAllocator,
        SourceAnchor, SourceId, SourceInput, SourceLoadError, SourceMapError, SourceOrigin,
        SourceOriginInput, SourceRange, hash_text, normalize_path,
    };
    use mizar_syntax::{SyntaxDiagnostic, SyntaxDiagnosticCode};
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn stub_parser_frontend_returns_artifacts_without_parser_diagnostics() {
        let fixture = PackageFixture::new();
        fixture.write("src/basic.miz", "definition\nend;\n");
        let frontend = frontend_for_fixture(&fixture, StubParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let output = frontend
            .run(fixture.request("src/basic.miz"), &ids)
            .expect("well-formed source should run through the stub frontend");

        assert_eq!(output.source.source_text.as_ref(), "definition\nend;\n");
        assert_eq!(output.preprocessed.source_id, output.source.source_id);
        assert_eq!(output.tokens.source_id, output.source.source_id);
        assert!(output.ast.is_none());
        assert!(
            output
                .diagnostics
                .iter()
                .all(|diagnostic| !matches!(diagnostic.code, DiagnosticCode::Syntax(_))),
            "stub parser seam should not emit parser diagnostics"
        );
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn real_parser_frontend_returns_ast_and_merges_diagnostics_by_phase() {
        let fixture = PackageFixture::new();
        fixture.write("src/recovered.miz", full_merge_fixture_text());
        let frontend =
            frontend_for_fixture_with_provider(&fixture, DiagnosticProvider, MizarParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let output = frontend
            .run(fixture.request("src/recovered.miz"), &ids)
            .expect("recoverable diagnostics should stay in FrontendOutput");

        assert!(output.ast.is_some());
        assert_eq!(
            consecutive_classes(&output.diagnostics),
            vec![
                DiagnosticClass::ImportPrescan,
                DiagnosticClass::LexicalEnvironment,
                DiagnosticClass::ScopeSkeleton,
                DiagnosticClass::Tokenization,
                DiagnosticClass::Syntax,
            ]
        );
        let syntax = output
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.class == DiagnosticClass::Syntax)
            .collect::<Vec<_>>();
        assert!(
            syntax.iter().any(|diagnostic| matches!(
                &diagnostic.code,
                DiagnosticCode::Syntax(code) if code.as_ref() == "missing_end"
            )),
            "missing end diagnostics should use stable syntax keys"
        );
        assert!(
            syntax.iter().any(|diagnostic| matches!(
                &diagnostic.code,
                DiagnosticCode::Syntax(code) if code.as_ref() == "missing_semicolon"
            )),
            "missing semicolon diagnostics should use stable syntax keys"
        );
        let missing_end = syntax
            .iter()
            .find(|diagnostic| {
                matches!(
                    &diagnostic.code,
                    DiagnosticCode::Syntax(code) if code.as_ref() == "missing_end"
                )
            })
            .expect("missing end diagnostic should be present");
        assert_eq!(
            missing_end.recovery_note.as_deref(),
            Some("insert `end` before this synchronization point")
        );
    }

    #[test]
    fn real_parser_frontend_maps_malformed_formula_diagnostic_key() {
        let fixture = PackageFixture::new();
        fixture.write("src/formula_recovered.miz", "theorem T: x = y &;\n");
        let frontend = frontend_for_fixture(&fixture, MizarParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let output = frontend
            .run(fixture.request("src/formula_recovered.miz"), &ids)
            .expect("formula recovery diagnostics should stay in FrontendOutput");

        assert!(output.ast.is_some());
        assert!(output.diagnostics.iter().any(|diagnostic| matches!(
            &diagnostic.code,
            DiagnosticCode::Syntax(code) if code.as_ref() == "malformed_formula_expression"
        )));
    }

    #[test]
    fn real_parser_frontend_merges_preprocess_classes_before_tokenization_and_syntax() {
        let fixture = PackageFixture::new();
        fixture.write("src/preprocess_recovered.miz", "α\n?\n::=\nopen block");
        let frontend = frontend_for_fixture(&fixture, MizarParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let output = frontend
            .run(fixture.request("src/preprocess_recovered.miz"), &ids)
            .expect("preprocess diagnostics should be recoverable in FrontendOutput");

        assert!(output.ast.is_some());
        assert_eq!(
            consecutive_classes(&output.diagnostics),
            vec![
                DiagnosticClass::LexicalPrecondition,
                DiagnosticClass::CommentStructure,
                DiagnosticClass::ImportPrescan,
                DiagnosticClass::Tokenization,
                DiagnosticClass::Syntax,
            ]
        );
    }

    #[test]
    fn real_parser_frontend_returns_ast_without_diagnostics_for_well_formed_source() {
        let fixture = PackageFixture::new();
        fixture.write("src/real_ok.miz", "definition\nend;\n");
        let frontend = frontend_for_fixture(&fixture, MizarParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let output = frontend
            .run(fixture.request("src/real_ok.miz"), &ids)
            .expect("well-formed source should run through the real parser frontend");

        assert!(output.ast.is_some());
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn real_parser_frontend_accepts_annotation_string_arguments_from_source() {
        let fixture = PackageFixture::new();
        fixture.write("src/annotation_string.miz", "@[label(\"α::β\")]\n");
        let frontend = frontend_for_fixture(&fixture, MizarParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let output = frontend
            .run(fixture.request("src/annotation_string.miz"), &ids)
            .expect("annotation string source should run through the real parser frontend");

        assert!(output.ast.is_some());
        assert!(output.diagnostics.is_empty());
        assert!(output.tokens.tokens().iter().any(
            |token| token.kind == TokenKind::StringLiteral && token.text.as_ref() == "\"α::β\""
        ));
        assert_eq!(
            output.cache_keys.tokens.parser_lexing_plan.contexts.len(),
            1
        );
        assert_eq!(
            output.cache_keys.tokens.parser_lexing_plan.contexts[0]
                .context
                .mode(),
            ParserLexMode::StringRequired
        );
    }

    #[test]
    fn real_parser_frontend_merges_nested_missing_end_and_uses_parser_v2_cache_key() {
        let fixture = PackageFixture::new();
        fixture.write(
            "src/nested_missing_end.miz",
            "definition\nalgorithm\nend;\n",
        );
        let frontend = frontend_for_fixture(&fixture, MizarParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let output = frontend
            .run(fixture.request("src/nested_missing_end.miz"), &ids)
            .expect("nested missing end should recover through the real parser frontend");

        assert!(output.ast.is_some());
        let syntax = output
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.class == DiagnosticClass::Syntax)
            .expect("nested missing end should be merged as a syntax diagnostic");
        assert!(matches!(
            &syntax.code,
            DiagnosticCode::Syntax(code) if code.as_ref() == "missing_end"
        ));
        assert_eq!(
            syntax.recovery_note.as_deref(),
            Some("insert `end` before this synchronization point")
        );
        assert_eq!(
            output
                .cache_keys
                .ast
                .as_ref()
                .expect("recovered AST should have an AST cache key")
                .parser_version
                .version
                .as_ref(),
            MIZAR_PARSER_CACHE_KEY_VERSION
        );
        assert_eq!(
            MIZAR_PARSER_CACHE_KEY_VERSION, "mizar-parser/surface-ast-v2",
            "task-28 parser output semantics must not reuse the v1 AST cache namespace"
        );
    }

    #[test]
    fn repeated_frontend_runs_preserve_diagnostic_order() {
        let fixture = PackageFixture::new();
        fixture.write("src/repeated.miz", full_merge_fixture_text());
        let frontend = frontend_for_fixture(&fixture, MizarParserSeam);

        let first = frontend
            .run(
                fixture.request("src/repeated.miz"),
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap()
            .diagnostics;
        let second = frontend
            .run(
                fixture.request("src/repeated.miz"),
                &InMemorySessionIdAllocator::new(),
            )
            .unwrap()
            .diagnostics;

        assert_eq!(first, second);
    }

    #[test]
    fn diagnostic_sort_is_total_for_same_class_start_and_code() {
        let source_id = source_id();
        let primary = SourceRange {
            source_id,
            start: 4,
            end: 5,
        };
        let secondary = SourceAnchor::Range(SourceRange {
            source_id,
            start: 0,
            end: 1,
        });
        let first = diagnostic(primary, "same", Vec::new(), None);
        let second = diagnostic(primary, "same", vec![secondary], None);
        let third = diagnostic(primary, "same", Vec::new(), Some("recover"));
        let fourth = diagnostic(primary, "zzz", Vec::new(), None);

        let sorted_once = sort_collected_diagnostics(vec![
            collected(fourth.clone(), 0),
            collected(third.clone(), 0),
            collected(second.clone(), 0),
            collected(first.clone(), 1),
        ]);
        let sorted_twice = sort_collected_diagnostics(vec![
            collected(second, 0),
            collected(first, 1),
            collected(fourth, 0),
            collected(third, 0),
        ]);

        let key = |diagnostic: &FrontendDiagnostic| {
            (
                diagnostic.message.to_string(),
                diagnostic.secondary.len(),
                diagnostic.recovery_note.clone(),
            )
        };
        assert_eq!(
            sorted_once.iter().map(key).collect::<Vec<_>>(),
            vec![
                ("same".to_owned(), 0, None),
                ("same".to_owned(), 0, Some("recover".to_owned())),
                ("same".to_owned(), 1, None),
                ("zzz".to_owned(), 0, None),
            ]
        );
        assert_eq!(
            sorted_once.iter().map(key).collect::<Vec<_>>(),
            sorted_twice.iter().map(key).collect::<Vec<_>>()
        );
        assert_eq!(
            compare_collected_diagnostics(
                &collected(diagnostic(primary, "same", Vec::new(), None), 0),
                &collected(diagnostic(primary, "same", Vec::new(), None), 1),
            ),
            std::cmp::Ordering::Less,
            "identical diagnostics should fall back to phase-local emission ordinal"
        );
    }

    #[test]
    fn reserved_frontend_diagnostic_surfaces_have_stable_local_policy() {
        let fixture = PackageFixture::new();
        fixture.write("src/reserved_surface.miz", "definition\nend;\n");
        let normalized_path =
            normalize_path(fixture.root(), &fixture.path("src/reserved_surface.miz")).unwrap();
        let source_id = source_id();
        let primary = SourceRange {
            source_id,
            start: 0,
            end: 0,
        };
        let lexing = super::frontend_lexing_diagnostic(&LexingDiagnostic {
            kind: LexingDiagnosticKind::Lexer(LexDiagnosticCode::NoValidTokenCandidate),
            message: Arc::from("future lexer payload"),
            primary,
            secondary: Vec::new(),
            payload: LexingDiagnosticPayload::UnsupportedLexerPayload,
        });
        assert_eq!(lexing.class, DiagnosticClass::Tokenization);
        assert_eq!(
            lexing.recovery_note, None,
            "fallback payloads must not invent recovery advice"
        );

        let normalized = source_load_reserved_diagnostic(
            "normalized",
            SourceLoadLocation::NormalizedPath {
                path: normalized_path,
            },
        );
        let unknown = source_load_reserved_diagnostic("unknown", SourceLoadLocation::Unknown);
        let annotation = FrontendDiagnostic {
            code: DiagnosticCode::Syntax(Arc::from("annotation_reserved")),
            message: Arc::from("annotation"),
            class: DiagnosticClass::AnnotationSyntax,
            location: DiagnosticLocation::SourceRange(primary),
            secondary: Vec::new(),
            recovery_note: None,
        };

        let sorted = sort_collected_diagnostics(vec![
            collected(annotation, 0),
            collected(unknown, 0),
            collected(normalized, 0),
        ]);

        assert_eq!(
            sorted
                .iter()
                .map(|diagnostic| diagnostic.message.as_ref())
                .collect::<Vec<_>>(),
            vec!["normalized", "unknown", "annotation"]
        );
    }

    #[test]
    fn source_load_error_uses_file_level_location() {
        let fixture = PackageFixture::new();
        let frontend = frontend_for_fixture(&fixture, StubParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let error = frontend
            .run(fixture.request("src/missing.miz"), &ids)
            .expect_err("missing disk source should be a FrontendError");

        let FrontendError::SourceLoad { diagnostic, .. } = error else {
            panic!("expected source-load error");
        };
        assert_eq!(diagnostic.class, DiagnosticClass::SourceLoad);
        assert_eq!(diagnostic.code, DiagnosticCode::SourceLoad);
        assert!(matches!(
            &diagnostic.location,
            DiagnosticLocation::SourceLoad(SourceLoadLocation::Path { .. })
        ));
    }

    #[test]
    fn source_load_error_for_open_buffer_keeps_non_range_location() {
        let fixture = PackageFixture::new();
        fixture.write("src/open_buffer.miz", "definition\nend;\n");
        let frontend = frontend_for_fixture(&fixture, StubParserSeam);
        let ids = InMemorySessionIdAllocator::new();
        let request = fixture.open_buffer_request(
            "src/open_buffer.miz",
            "file:///fixture/src/open_buffer.miz",
            2,
            1,
            "definition\nend;\n",
        );

        let error = frontend
            .run(request, &ids)
            .expect_err("stale open buffer should be a FrontendError");

        let FrontendError::SourceLoad { source, diagnostic } = error else {
            panic!("expected source-load error");
        };
        assert!(matches!(
            source.as_ref(),
            SourceLoadError::StaleLspDocumentVersion {
                expected: 2,
                actual: 1,
            }
        ));
        assert_eq!(diagnostic.class, DiagnosticClass::SourceLoad);
        assert!(matches!(
            &diagnostic.location,
            DiagnosticLocation::SourceLoad(SourceLoadLocation::OpenBuffer { uri })
                if uri == "file:///fixture/src/open_buffer.miz"
        ));
        assert!(
            !matches!(diagnostic.location, DiagnosticLocation::SourceRange(_)),
            "source-load diagnostics must not fabricate a zero-length SourceRange"
        );
    }

    #[test]
    fn source_load_error_for_generated_source_keeps_anchor_location() {
        let fixture = PackageFixture::new();
        let anchor = SourceAnchor::Range(SourceRange {
            source_id: source_id(),
            start: 1,
            end: 4,
        });
        let module_path = ModulePath::new("generated.failure");
        let frontend = Frontend::new(
            ErrorSourceLoader {
                error: SourceLoadError::GeneratedSourceWithoutMetadata {
                    module_path: module_path.clone(),
                },
            },
            EmptyProvider,
            StubParserSeam,
        );
        let ids = InMemorySessionIdAllocator::new();

        let error = frontend
            .run(
                fixture.generated_request(
                    "src/generated_failure.miz",
                    module_path,
                    "generated text",
                    Some(anchor.clone()),
                ),
                &ids,
            )
            .expect_err("generated load failure should be a FrontendError");

        let FrontendError::SourceLoad { diagnostic, .. } = error else {
            panic!("expected source-load error");
        };
        assert_eq!(diagnostic.code, DiagnosticCode::SourceLoad);
        assert_eq!(diagnostic.class, DiagnosticClass::SourceLoad);
        assert!(matches!(
            &diagnostic.location,
            DiagnosticLocation::SourceLoad(SourceLoadLocation::Generated {
                anchor: Some(returned_anchor),
            }) if returned_anchor == &anchor
        ));
        assert!(
            !matches!(diagnostic.location, DiagnosticLocation::SourceRange(_)),
            "generated source-load diagnostics must not fabricate a SourceRange"
        );
    }

    #[test]
    fn span_bridge_registration_hard_failure_returns_frontend_error() {
        let fixture = PackageFixture::new();
        fixture.write("src/span_bridge_registration.miz", "definition\nend;\n");
        let frontend = Frontend::new(
            BrokenSourceLoader::new(BrokenLineMapMode::WrongSourceId),
            EmptyProvider,
            StubParserSeam,
        );
        let ids = InMemorySessionIdAllocator::new();

        let error = frontend
            .run(fixture.request("src/span_bridge_registration.miz"), &ids)
            .expect_err("broken source registration should be a FrontendError");

        let FrontendError::SpanBridge { source } = error else {
            panic!("expected span-bridge error");
        };
        assert!(matches!(
            source,
            SpanBridgeError::SourceMap {
                source: SourceMapError::UnknownSourceId { .. },
            }
        ));
    }

    #[test]
    fn span_bridge_preprocess_hard_failure_returns_frontend_error() {
        let fixture = PackageFixture::new();
        fixture.write("src/span_bridge_preprocess.miz", "definition\nend;\n");
        let frontend = Frontend::new(
            BrokenSourceLoader::new(BrokenLineMapMode::WrongText),
            EmptyProvider,
            StubParserSeam,
        );
        let ids = InMemorySessionIdAllocator::new();

        let error = frontend
            .run(fixture.request("src/span_bridge_preprocess.miz"), &ids)
            .expect_err("broken preprocess mapping should be a FrontendError");

        let FrontendError::SpanBridge { source } = error else {
            panic!("expected span-bridge error");
        };
        assert!(matches!(
            source,
            SpanBridgeError::SourceMap {
                source: SourceMapError::RangeOutsideSourceText { .. },
            }
        ));
    }

    #[test]
    fn span_bridge_lexing_hard_failure_returns_frontend_error() {
        let fixture = PackageFixture::new();
        fixture.write("src/span_bridge_lexing.miz", "a b");
        let frontend = Frontend::new(
            BrokenSourceLoader::new(BrokenLineMapMode::LexingTokenBoundary),
            EmptyProvider,
            StubParserSeam,
        );
        let ids = InMemorySessionIdAllocator::new();

        let error = frontend
            .run(fixture.request("src/span_bridge_lexing.miz"), &ids)
            .expect_err("broken lexing span mapping should be a FrontendError");

        let FrontendError::SpanBridge { source } = error else {
            panic!("expected span-bridge error");
        };
        assert!(matches!(
            source,
            SpanBridgeError::SourceMap {
                source: SourceMapError::OffsetNotUtf8Boundary { offset: 2, .. },
            }
        ));
    }

    #[test]
    fn lexical_environment_hard_failure_returns_frontend_error() {
        let fixture = PackageFixture::new();
        fixture.write(
            "src/lexical_environment_hard_failure.miz",
            "definition\nend;\n",
        );
        let frontend =
            frontend_for_fixture_with_provider(&fixture, FailingProvider, StubParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let error = frontend
            .run(
                fixture.request("src/lexical_environment_hard_failure.miz"),
                &ids,
            )
            .expect_err("provider hard failure should be a FrontendError");

        let FrontendError::LexicalEnvironment { source } = error else {
            panic!("expected lexical-environment error");
        };
        assert_eq!(
            source,
            crate::lexical_env::FrontendLexicalEnvironmentError::ProviderUnavailable {
                message: "fixture provider unavailable".to_owned(),
            }
        );
    }

    #[test]
    fn none_ast_parser_seam_preserves_earlier_diagnostics() {
        let fixture = PackageFixture::new();
        fixture.write("src/none_ast_recovered.miz", full_merge_fixture_text());
        let frontend =
            frontend_for_fixture_with_provider(&fixture, DiagnosticProvider, NoneAstParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let output = frontend
            .run(fixture.request("src/none_ast_recovered.miz"), &ids)
            .expect("recoverable diagnostics should remain in FrontendOutput");

        assert!(output.ast.is_none());
        assert_eq!(
            consecutive_classes(&output.diagnostics),
            vec![
                DiagnosticClass::ImportPrescan,
                DiagnosticClass::LexicalEnvironment,
                DiagnosticClass::ScopeSkeleton,
                DiagnosticClass::Tokenization,
                DiagnosticClass::Syntax,
            ]
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.class == DiagnosticClass::ImportPrescan),
            "pre-parser diagnostics should be preserved when the parser returns ast = None"
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.class == DiagnosticClass::Syntax),
            "test seam should still contribute syntax diagnostics"
        );
    }

    #[test]
    fn merged_range_backed_diagnostics_have_valid_session_ranges() {
        let fixture = PackageFixture::new();
        fixture.write("src/range_backed.miz", full_merge_fixture_text());
        let frontend =
            frontend_for_fixture_with_provider(&fixture, DiagnosticProvider, MizarParserSeam);
        let ids = InMemorySessionIdAllocator::new();

        let output = frontend
            .run(fixture.request("src/range_backed.miz"), &ids)
            .expect("recoverable diagnostics should stay in FrontendOutput");

        assert!(!output.diagnostics.is_empty());
        assert!(
            output.diagnostics.iter().all(|diagnostic| matches!(
                diagnostic.location,
                DiagnosticLocation::SourceRange(_)
            )),
            "returned phase diagnostics should be range-backed"
        );
        assert!(
            output
                .diagnostics
                .iter()
                .any(|diagnostic| !diagnostic.secondary.is_empty()),
            "fixture should exercise secondary range validation"
        );
        for diagnostic in &output.diagnostics {
            let DiagnosticLocation::SourceRange(primary) = diagnostic.location else {
                unreachable!("asserted above");
            };
            assert_valid_source_range(&output.source, primary);
            for anchor in &diagnostic.secondary {
                assert_valid_source_anchor(&output.source, anchor);
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct EmptyProvider;

    impl LexicalSummaryProvider for EmptyProvider {
        fn resolve_imports(
            &self,
            _request: &LexicalEnvironmentRequest<'_>,
        ) -> Result<ResolvedImports, crate::lexical_env::FrontendLexicalEnvironmentError> {
            Ok(ResolvedImports {
                imports: Vec::new(),
                summaries: Vec::new(),
                diagnostics: Vec::new(),
            })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DiagnosticProvider;

    impl LexicalSummaryProvider for DiagnosticProvider {
        fn resolve_imports(
            &self,
            request: &LexicalEnvironmentRequest<'_>,
        ) -> Result<ResolvedImports, crate::lexical_env::FrontendLexicalEnvironmentError> {
            Ok(ResolvedImports {
                imports: Vec::new(),
                summaries: Vec::new(),
                diagnostics: vec![LexicalEnvironmentDiagnostic {
                    code: LexicalEnvironmentDiagnosticCode::UnresolvedImport,
                    message: Arc::<str>::from("fixture lexical environment diagnostic"),
                    primary: SourceRange {
                        source_id: request.source_id,
                        start: 0,
                        end: 0,
                    },
                    secondary: Vec::new(),
                    import_ordinal: None,
                    module_id: None,
                }],
            })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct FailingProvider;

    impl LexicalSummaryProvider for FailingProvider {
        fn resolve_imports(
            &self,
            _request: &LexicalEnvironmentRequest<'_>,
        ) -> Result<ResolvedImports, crate::lexical_env::FrontendLexicalEnvironmentError> {
            Err(
                crate::lexical_env::FrontendLexicalEnvironmentError::ProviderUnavailable {
                    message: "fixture provider unavailable".to_owned(),
                },
            )
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct NoneAstParserSeam;

    impl ParserSeam for NoneAstParserSeam {
        type Ast = ();
        type Diagnostic = SyntaxDiagnostic;

        fn parse(&self, request: ParseRequest<'_>) -> ParseOutput<Self::Ast, Self::Diagnostic> {
            let primary = request
                .tokens
                .tokens
                .first()
                .map(|token| token.span)
                .unwrap_or(SourceRange {
                    source_id: request.tokens.source_id,
                    start: 0,
                    end: 0,
                });
            ParseOutput::new(
                None,
                vec![
                    SyntaxDiagnostic::new(
                        SyntaxDiagnosticCode::UnrecoverableInput,
                        "fixture parser could not recover an AST",
                        primary,
                    )
                    .with_recovery_note("fixture parser stopped after preserving diagnostics"),
                ],
            )
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum BrokenLineMapMode {
        WrongSourceId,
        WrongText,
        LexingTokenBoundary,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct BrokenSourceLoader {
        mode: BrokenLineMapMode,
    }

    impl BrokenSourceLoader {
        fn new(mode: BrokenLineMapMode) -> Self {
            Self { mode }
        }
    }

    impl SourceUnitLoader for BrokenSourceLoader {
        fn load_source_unit(
            &self,
            request: SourceUnitRequest,
            ids: &dyn SessionIdAllocator,
        ) -> Result<SourceUnit, SourceLoadError> {
            let source_id = ids
                .next_source_id(request.snapshot)
                .map_err(|error| SourceLoadError::SourceIdAllocation { error })?;
            let text: Arc<str> = Arc::from(match self.mode {
                BrokenLineMapMode::LexingTokenBoundary => "a b",
                BrokenLineMapMode::WrongSourceId | BrokenLineMapMode::WrongText => {
                    "definition\nend;\n"
                }
            });
            let line_map = match self.mode {
                BrokenLineMapMode::WrongSourceId => {
                    let other_source_id = ids
                        .next_source_id(request.snapshot)
                        .map_err(|error| SourceLoadError::SourceIdAllocation { error })?;
                    LineMap::new(other_source_id, &text)
                }
                BrokenLineMapMode::WrongText => LineMap::new(source_id, ""),
                BrokenLineMapMode::LexingTokenBoundary => LineMap::new(source_id, "aβ"),
            };
            let file_path = match &request.input.origin {
                SourceOriginInput::Disk { path } => path.clone(),
                _ => PathBuf::from(request.input.normalized_path.as_str()),
            };

            Ok(SourceUnit {
                source_id,
                package_id: request.input.package_id,
                module_path: request.input.module_path,
                normalized_path: request.input.normalized_path,
                edition: request.input.edition,
                file_path,
                source_text: text.clone(),
                source_hash: hash_text(&text),
                line_map,
                loading_map: None,
                origin: SourceOrigin::Disk,
                generated_anchor: None,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ErrorSourceLoader {
        error: SourceLoadError,
    }

    impl SourceUnitLoader for ErrorSourceLoader {
        fn load_source_unit(
            &self,
            _request: SourceUnitRequest,
            _ids: &dyn SessionIdAllocator,
        ) -> Result<SourceUnit, SourceLoadError> {
            Err(self.error.clone())
        }
    }

    fn frontend_for_fixture<PS>(
        fixture: &PackageFixture,
        parser: PS,
    ) -> Frontend<FrontendSourceLoader<DiskSourceLoader>, EmptyProvider, PS>
    where
        PS: crate::parsing::ParserSeam,
        PS::Diagnostic: super::FrontendParserDiagnostic,
    {
        Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root())),
            EmptyProvider,
            parser,
        )
    }

    fn frontend_for_fixture_with_provider<P, PS>(
        fixture: &PackageFixture,
        provider: P,
        parser: PS,
    ) -> Frontend<FrontendSourceLoader<DiskSourceLoader>, P, PS>
    where
        P: LexicalSummaryProvider,
        PS: crate::parsing::ParserSeam,
        PS::Diagnostic: super::FrontendParserDiagnostic,
    {
        Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root())),
            provider,
            parser,
        )
    }

    fn diagnostic(
        primary: SourceRange,
        message: &str,
        secondary: Vec<SourceAnchor>,
        recovery_note: Option<&str>,
    ) -> FrontendDiagnostic {
        FrontendDiagnostic {
            code: DiagnosticCode::Syntax(Arc::<str>::from("same_code")),
            message: Arc::<str>::from(message),
            class: DiagnosticClass::Syntax,
            location: DiagnosticLocation::SourceRange(primary),
            secondary,
            recovery_note: recovery_note.map(str::to_owned),
        }
    }

    fn collected(diagnostic: FrontendDiagnostic, emission_ordinal: usize) -> CollectedDiagnostic {
        CollectedDiagnostic {
            diagnostic,
            emission_ordinal,
        }
    }

    fn source_load_reserved_diagnostic(
        message: &str,
        location: SourceLoadLocation,
    ) -> FrontendDiagnostic {
        FrontendDiagnostic {
            code: DiagnosticCode::SourceLoad,
            message: Arc::from(message),
            class: DiagnosticClass::SourceLoad,
            location: DiagnosticLocation::SourceLoad(location),
            secondary: Vec::new(),
            recovery_note: None,
        }
    }

    fn full_merge_fixture_text() -> &'static str {
        "import std.;\ndefinition\n?\nalpha"
    }

    fn consecutive_classes(diagnostics: &[FrontendDiagnostic]) -> Vec<DiagnosticClass> {
        let mut classes = Vec::new();
        for diagnostic in diagnostics {
            if classes.last() != Some(&diagnostic.class) {
                classes.push(diagnostic.class);
            }
        }
        classes
    }

    fn assert_valid_source_range(source: &SourceUnit, range: SourceRange) {
        assert_eq!(range.source_id, source.source_id);
        source
            .line_map
            .validate_range(range)
            .expect("range-backed diagnostic should be valid in the session line map");
        assert!(range.start <= range.end);
        assert!(range.end <= source.source_text.len());
        assert!(source.source_text.is_char_boundary(range.start));
        assert!(source.source_text.is_char_boundary(range.end));
    }

    fn assert_valid_source_anchor(source: &SourceUnit, anchor: &SourceAnchor) {
        match anchor {
            SourceAnchor::Range(range) => assert_valid_source_range(source, *range),
            SourceAnchor::Point { source_id, offset } => {
                assert_eq!(*source_id, source.source_id);
                assert!(*offset <= source.source_text.len());
                assert!(source.source_text.is_char_boundary(*offset));
            }
            SourceAnchor::Generated(_) => {}
            _ => {}
        }
    }

    fn source_id() -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(90))
            .unwrap()
    }

    fn snapshot_id(id: u8) -> BuildSnapshotId {
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{id:064x}"
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
                "mizar-frontend-orchestration-test-{}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(root.join("src")).unwrap();
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
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, text).unwrap();
        }

        fn request(&self, relative: &str) -> SourceUnitRequest {
            let path = self.path(relative);
            let normalized_path = normalize_path(&self.root, &path).unwrap_or_else(|_| {
                let fallback = self.path("src/__fallback__.miz");
                if !fallback.exists() {
                    self.write("src/__fallback__.miz", "");
                }
                normalize_path(&self.root, &fallback).unwrap()
            });
            SourceUnitRequest {
                snapshot: snapshot_id(1),
                input: SourceInput {
                    package_id: PackageId::new("fixture"),
                    module_path: ModulePath::new(module_path_from_relative(relative)),
                    normalized_path,
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Disk { path },
                },
            }
        }

        fn open_buffer_request(
            &self,
            relative: &str,
            uri: &str,
            expected_version: i64,
            actual_version: i64,
            text: &str,
        ) -> SourceUnitRequest {
            let path = self.path(relative);
            let normalized_path = normalize_path(&self.root, &path).unwrap();
            SourceUnitRequest {
                snapshot: snapshot_id(1),
                input: SourceInput {
                    package_id: PackageId::new("fixture"),
                    module_path: ModulePath::new(module_path_from_relative(relative)),
                    normalized_path,
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::OpenBuffer {
                        uri: uri.to_owned(),
                        expected_version,
                        actual_version,
                        text: Arc::<str>::from(text),
                    },
                },
            }
        }

        fn generated_request(
            &self,
            relative: &str,
            module_path: ModulePath,
            text: &str,
            anchor: Option<SourceAnchor>,
        ) -> SourceUnitRequest {
            if !self.path(relative).exists() {
                self.write(relative, "");
            }
            let path = self.path(relative);
            let normalized_path = normalize_path(&self.root, &path).unwrap();
            SourceUnitRequest {
                snapshot: snapshot_id(1),
                input: SourceInput {
                    package_id: PackageId::new("fixture"),
                    module_path,
                    normalized_path,
                    edition: Edition::new("2026"),
                    origin: SourceOriginInput::Generated {
                        generator: GeneratedSourceKind::new("fixture-generator"),
                        text: Arc::<str>::from(text),
                        anchor,
                    },
                },
            }
        }
    }

    impl Drop for PackageFixture {
        fn drop(&mut self) {
            match fs::remove_dir_all(&self.root) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(_) => {}
            }
        }
    }

    fn module_path_from_relative(relative: &str) -> String {
        relative
            .strip_prefix("src/")
            .unwrap_or(relative)
            .strip_suffix(".miz")
            .unwrap_or(relative)
            .replace('/', ".")
    }
}
