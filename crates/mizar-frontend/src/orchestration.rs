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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendOutput<A> {
    pub source: SourceUnit,
    pub preprocessed: PreprocessedSource,
    pub tokens: TokenStream,
    pub ast: Option<A>,
    pub diagnostics: Vec<FrontendDiagnostic>,
}

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
    pub fn new(loader: L, provider: P, parser: PS) -> Self {
        Self {
            loader,
            provider,
            parser,
        }
    }

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
        let tokens = tokenize(
            TokenizeRequest::new(
                &preprocessed,
                &lexical_environment.environment,
                parser_inputs.string_required_positions.parser_lex_context(),
            ),
            &bridge,
        )
        .map_err(|source| FrontendError::SpanBridge { source })?;
        let parser_output = self.parser.parse(ParseRequest::new(&tokens, parser_inputs));

        let diagnostics = merge_phase_diagnostics(
            &preprocessed.diagnostics,
            &lexical_environment.diagnostics,
            &tokens.diagnostics,
            parser_output.diagnostics,
        );

        Ok(FrontendOutput {
            source,
            preprocessed,
            tokens,
            ast: parser_output.ast,
            diagnostics,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendDiagnostic {
    pub code: DiagnosticCode,
    pub message: Arc<str>,
    pub class: DiagnosticClass,
    pub location: DiagnosticLocation,
    pub secondary: Vec<SourceAnchor>,
    pub recovery_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLocation {
    SourceRange(SourceRange),
    SourceLoad(SourceLoadLocation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceLoadLocation {
    Path { path: PathBuf },
    NormalizedPath { path: NormalizedPath },
    OpenBuffer { uri: DocumentUri },
    Generated { anchor: Option<SourceAnchor> },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticCode {
    SourceLoad,
    Preprocess(PreprocessDiagnosticKind),
    LexicalEnvironment(LexicalEnvironmentDiagnosticCode),
    Lexing(LexingDiagnosticKind),
    Syntax(Arc<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticClass {
    SourceLoad,
    LexicalPrecondition,
    CommentStructure,
    ImportPrescan,
    LexicalEnvironment,
    ScopeSkeleton,
    Tokenization,
    Syntax,
    AnnotationSyntax,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendError {
    SourceLoad {
        source: Box<SourceLoadError>,
        diagnostic: Box<FrontendDiagnostic>,
    },
    SpanBridge {
        source: SpanBridgeError,
    },
    LexicalEnvironment {
        source: FrontendLexicalEnvironmentError,
    },
}

pub trait FrontendParserDiagnostic {
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
        mizar_syntax::SyntaxDiagnosticCode::MissingStringLiteral => "missing_string_literal",
        mizar_syntax::SyntaxDiagnosticCode::UnrecoverableInput => "unrecoverable_input",
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
    use crate::parsing::{MizarParserSeam, StubParserSeam};
    use crate::source::{FrontendSourceLoader, SourceUnitRequest};
    use mizar_session::{
        BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath,
        PackageId, SessionIdAllocator, SourceAnchor, SourceId, SourceInput, SourceOriginInput,
        SourceRange, normalize_path,
    };
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
        assert!(
            matches!(
                &output.diagnostics.last().unwrap().code,
                DiagnosticCode::Syntax(code) if code.as_ref() == "missing_end"
            ),
            "real parser diagnostics should be represented by stable syntax keys"
        );
        assert_eq!(
            output.diagnostics.last().unwrap().recovery_note.as_deref(),
            Some("insert `end` before this synchronization point")
        );
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
        assert!(matches!(
            &diagnostic.location,
            DiagnosticLocation::SourceLoad(SourceLoadLocation::Path { .. })
        ));
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
