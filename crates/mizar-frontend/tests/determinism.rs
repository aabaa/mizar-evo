use mizar_frontend::cache_key::{
    ActiveLexicalEnvironmentCacheKey, FrontendCacheKeys, ParserLexingPlanCacheKey,
    PreprocessedSourceCacheKey, SourceUnitCacheKey, SurfaceAstCacheKey, TokenStreamCacheKey,
};
use mizar_frontend::lexical_env::{
    ActiveLexicalEnvironmentResult, ExportRank, ExportedOperatorAssociativity,
    ExportedOperatorFixity, ExportedOperatorMetadata, ExportedSymbolShape,
    FrontendLexicalEnvironmentError, LexicalEnvironmentDiagnostic,
    LexicalEnvironmentDiagnosticCode, LexicalEnvironmentFingerprint, LexicalEnvironmentRequest,
    LexicalSummaryFingerprint, LexicalSummaryProvider, ModuleId, ModuleLexicalSummary,
    ResolvedImport, ResolvedImportEntry, ResolvedImports, SymbolId, UserSymbolArity,
    UserSymbolKind, build_active_lexical_environment,
};
use mizar_frontend::lexing::{Token, TokenKind};
use mizar_frontend::orchestration::{DiagnosticClass, Frontend};
use mizar_frontend::parsing::{MizarParserSeam, ParserInputs, ParserSeam};
use mizar_frontend::preprocess::ImportStub;
use mizar_frontend::source::{FrontendSourceLoader, SourceUnitRequest};
use mizar_session::{
    BuildSnapshotId, DiskSourceLoader, Edition, InMemorySessionIdAllocator, ModulePath, PackageId,
    SourceId, SourceInput, SourceOriginInput, SourceRange, normalize_path,
};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

const IDENTICAL_SOURCE: &str = "\
import beta, missing_late, alpha, gamma, missing_early;
definition
?
x+*+y";

const COMMENT_EQUIVALENT_SOURCE_A: &str = "\
:: short provider-local comment
import alpha, beta;
definition
x+*+y
end;
";

const COMMENT_EQUIVALENT_SOURCE_B: &str = "\
:: this comment body changed without changing lexical text
import alpha, beta;
definition
x+*+y
end;
";

const IMPORT_SOURCE_ALPHA_BETA: &str = "\
import alpha, beta;
definition
x+*+y
end;
";

const IMPORT_SOURCE_BETA_ONLY: &str = "\
import beta;
definition
x**y
end;
";

const B1B1P_RECOVERY_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "reserve x for set;\n",
    "theorem FormulaStatementParenthesizedApplicationWitnessSmoke: x = x proof\n",
    "  take (1 ++ 2);\n",
    "  thus x = x;\n",
    "end;\n",
);

#[test]
fn frontend_output_order_and_token_spans_are_independent_of_provider_scheduling() {
    let fixture = PackageFixture::new();
    fixture.write("src/determinism.miz", IDENTICAL_SOURCE);
    let baseline = run_frontend(&fixture, "src/determinism.miz", ProviderSchedule::InOrder);
    let baseline_diagnostics = baseline.diagnostics.clone();
    let baseline_tokens = token_span_signature(baseline.tokens.tokens());

    assert!(
        baseline
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.class == DiagnosticClass::LexicalEnvironment),
        "fixture should exercise provider/import scheduling through frontend diagnostics"
    );
    assert!(
        baseline
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.class == DiagnosticClass::Tokenization),
        "fixture should include tokenization recovery so token spans are checked after diagnostics"
    );
    assert!(
        baseline_tokens
            .iter()
            .any(|(kind, text, _)| { *kind == TokenKind::UserSymbol && text.as_str() == "+*+" }),
        "fixture should prove the scheduled lexical environment affects tokenization"
    );

    for schedule in [ProviderSchedule::Reverse, ProviderSchedule::RotateLeft] {
        let output = run_frontend(&fixture, "src/determinism.miz", schedule);

        assert_eq!(
            output.diagnostics, baseline_diagnostics,
            "{schedule:?} should preserve deterministic diagnostic merge order"
        );
        assert_eq!(
            token_span_signature(output.tokens.tokens()),
            baseline_tokens,
            "{schedule:?} should preserve token kinds/text and source spans"
        );
        assert_eq!(
            output.preprocessed.lexical_hash, baseline.preprocessed.lexical_hash,
            "{schedule:?} should not perturb the lexical cache component"
        );
    }
}

#[test]
fn lexical_environment_fingerprint_and_cache_key_are_stable_for_equivalent_inputs() {
    let fixture = PackageFixture::new();
    fixture.write("src/comment_a.miz", COMMENT_EQUIVALENT_SOURCE_A);
    fixture.write("src/comment_b.miz", COMMENT_EQUIVALENT_SOURCE_B);

    let first = run_frontend(&fixture, "src/comment_a.miz", ProviderSchedule::Reverse);
    let second = run_frontend(&fixture, "src/comment_b.miz", ProviderSchedule::RotateLeft);

    assert_eq!(
        first.preprocessed.lexical_text.as_str(),
        second.preprocessed.lexical_text.as_str(),
        "the fixture texts should differ only in removed comment bodies"
    );
    assert_eq!(
        first.preprocessed.lexical_hash, second.preprocessed.lexical_hash,
        "comment-only edits that preserve lexical text should keep the lexical hash stable"
    );

    let first_fingerprint = lexical_environment_fingerprint(
        first.source.source_id,
        &first.preprocessed.import_stubs,
        ProviderSchedule::Reverse,
    );
    let second_fingerprint = lexical_environment_fingerprint(
        second.source.source_id,
        &second.preprocessed.import_stubs,
        ProviderSchedule::RotateLeft,
    );

    assert_eq!(
        first_fingerprint, second_fingerprint,
        "equivalent import and dependency-summary inputs should keep the lexical environment fingerprint stable"
    );
    assert_eq!(
        first.cache_keys.tokens, second.cache_keys.tokens,
        "the token cache key should be stable for equivalent lexical text, environment, and parser context"
    );
    assert_eq!(
        first.cache_keys.ast, second.cache_keys.ast,
        "the AST cache key should be stable when tokenization, parser version, and edition are unchanged"
    );
    assert_ne!(
        first.cache_keys.preprocessed, second.cache_keys.preprocessed,
        "comment-only edits still invalidate preprocessing because source_hash changes"
    );
    assert_eq!(
        first.cache_keys.tokens.active_lexical_environment, first_fingerprint,
        "the token key should expose the active lexical environment fingerprint"
    );
    assert_eq!(
        second.cache_keys.tokens.active_lexical_environment, second_fingerprint,
        "the token key should expose the active lexical environment fingerprint"
    );
    assert_eq!(
        first.cache_keys,
        expected_cache_keys(&first, ProviderSchedule::Reverse, DEFAULT_BETA_FINGERPRINT),
        "FrontendOutput should expose the cache keys computed from its phase artifacts"
    );
    assert_eq!(
        second.cache_keys,
        expected_cache_keys(
            &second,
            ProviderSchedule::RotateLeft,
            DEFAULT_BETA_FINGERPRINT
        ),
        "FrontendOutput should expose the cache keys computed from its phase artifacts"
    );
}

#[test]
fn dependency_export_edits_invalidate_frontend_cache_keys_end_to_end() {
    let fixture = PackageFixture::new();
    fixture.write("src/dependency_change.miz", IMPORT_SOURCE_ALPHA_BETA);

    let baseline = run_frontend(
        &fixture,
        "src/dependency_change.miz",
        ProviderSchedule::InOrder,
    );
    let changed_dependency = run_frontend_with_beta_fingerprint(
        &fixture,
        "src/dependency_change.miz",
        ProviderSchedule::InOrder,
        DEFAULT_BETA_FINGERPRINT + 1,
    );

    assert_eq!(
        baseline.cache_keys.source,
        changed_dependency.cache_keys.source
    );
    assert_eq!(
        baseline.cache_keys.preprocessed,
        changed_dependency.cache_keys.preprocessed
    );
    assert_ne!(
        baseline.cache_keys.active_lexical_environment,
        changed_dependency.cache_keys.active_lexical_environment
    );
    assert_ne!(
        baseline.cache_keys.tokens,
        changed_dependency.cache_keys.tokens
    );
    assert_ne!(baseline.cache_keys.ast, changed_dependency.cache_keys.ast);
}

#[test]
fn import_edits_invalidate_frontend_cache_keys_end_to_end() {
    let fixture = PackageFixture::new();
    fixture.write("src/import_change.miz", IMPORT_SOURCE_ALPHA_BETA);
    let baseline = run_frontend(&fixture, "src/import_change.miz", ProviderSchedule::InOrder);

    fixture.write("src/import_change.miz", IMPORT_SOURCE_BETA_ONLY);
    let changed_import = run_frontend(&fixture, "src/import_change.miz", ProviderSchedule::InOrder);

    assert_ne!(baseline.cache_keys.source, changed_import.cache_keys.source);
    assert_ne!(
        baseline.cache_keys.preprocessed,
        changed_import.cache_keys.preprocessed
    );
    assert_ne!(
        baseline.cache_keys.active_lexical_environment,
        changed_import.cache_keys.active_lexical_environment
    );
    assert_ne!(baseline.cache_keys.tokens, changed_import.cache_keys.tokens);
    assert_ne!(baseline.cache_keys.ast, changed_import.cache_keys.ast);
}

#[test]
fn imported_postfix_parser_recovery_is_deterministic_end_to_end() {
    const RECOVERABLE_MUTATIONS: [usize; 9] = [48, 49, 50, 51, 52, 53, 54, 112, 148];

    assert_eq!(B1B1P_RECOVERY_SOURCE.len(), 158);
    assert_eq!(&B1B1P_RECOVERY_SOURCE[48..55], "theorem");
    assert_eq!(&B1B1P_RECOVERY_SOURCE[112..113], "=");
    assert_eq!(&B1B1P_RECOVERY_SOURCE[116..121], "proof");
    assert_eq!(&B1B1P_RECOVERY_SOURCE[148..149], "=");

    let fixture = PackageFixture::new();
    fixture.write("src/recovery.miz", B1B1P_RECOVERY_SOURCE);
    let control = run_frontend(&fixture, "src/recovery.miz", ProviderSchedule::InOrder);
    let control_replay = run_frontend(&fixture, "src/recovery.miz", ProviderSchedule::InOrder);
    let control_ast = control
        .ast
        .as_ref()
        .expect("the unchanged source must retain its previously returning AST");
    assert!(
        !control
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.class == DiagnosticClass::Syntax),
        "the unchanged source must remain syntax-diagnostic free"
    );
    assert_eq!(
        control_ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, mizar_syntax::SurfaceNodeKind::TheoremItem))
            .count(),
        1,
        "the unchanged source must retain its theorem structure"
    );
    assert_eq!(
        control_ast
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind, mizar_syntax::SurfaceNodeKind::ProofBlock))
            .count(),
        1,
        "the unchanged source must retain its proof boundary"
    );
    assert!(
        !control_ast
            .nodes()
            .iter()
            .any(|node| matches!(node.kind, mizar_syntax::SurfaceNodeKind::ErrorRecovery(_))),
        "the unchanged source must not gain recovery structure"
    );
    assert_eq!(
        control.ast, control_replay.ast,
        "the unchanged source AST must remain deterministic"
    );
    assert_eq!(
        control.cache_keys, control_replay.cache_keys,
        "the unchanged source cache keys must remain deterministic"
    );

    for mutation in RECOVERABLE_MUTATIONS {
        let mut source = B1B1P_RECOVERY_SOURCE.as_bytes().to_vec();
        source[mutation] = b'!';
        let source = String::from_utf8(source).expect("single-byte mutations remain valid UTF-8");
        fixture.write("src/recovery.miz", &source);

        let first = run_frontend(&fixture, "src/recovery.miz", ProviderSchedule::InOrder);
        let second = run_frontend(&fixture, "src/recovery.miz", ProviderSchedule::InOrder);
        let first_ast = first
            .ast
            .as_ref()
            .unwrap_or_else(|| panic!("mutation byte {mutation} must retain a recovered AST"));

        assert!(
            first
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.class == DiagnosticClass::Syntax),
            "mutation byte {mutation} must retain a syntax diagnostic"
        );
        assert!(
            first_ast
                .nodes()
                .iter()
                .any(|node| matches!(node.kind, mizar_syntax::SurfaceNodeKind::ErrorRecovery(_))),
            "mutation byte {mutation} must retain explicit recovery structure"
        );
        assert!(
            first
                .tokens
                .tokens()
                .iter()
                .any(|token| { token.kind == TokenKind::UserSymbol && token.text.as_ref() == "!" }),
            "mutation byte {mutation} must tokenize imported postfix `!` as a user symbol"
        );
        assert!(
            first.tokens.tokens().iter().any(|token| {
                token.kind == TokenKind::UserSymbol && token.text.as_ref() == "++"
            }),
            "mutation byte {mutation} must retain imported infix `++` provenance"
        );
        assert_eq!(
            first.diagnostics, second.diagnostics,
            "mutation byte {mutation} diagnostics must be deterministic"
        );
        assert_eq!(
            first.ast, second.ast,
            "mutation byte {mutation} AST must be deterministic"
        );
        assert_eq!(
            first.cache_keys, second.cache_keys,
            "mutation byte {mutation} cache keys must be deterministic"
        );
    }

    for mutation in 116..=120 {
        let mut source = B1B1P_RECOVERY_SOURCE.as_bytes().to_vec();
        source[mutation] = b'!';
        let source = String::from_utf8(source).expect("single-byte mutations remain valid UTF-8");
        fixture.write("src/recovery.miz", &source);

        let first = run_frontend(&fixture, "src/recovery.miz", ProviderSchedule::InOrder);
        let second = run_frontend(&fixture, "src/recovery.miz", ProviderSchedule::InOrder);

        assert!(
            first.ast.is_none(),
            "excluded proof mutation byte {mutation} must retain the documented absent AST"
        );
        assert!(
            first
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.class == DiagnosticClass::Syntax),
            "excluded proof mutation byte {mutation} must retain syntax diagnostics"
        );
        assert_eq!(
            first.diagnostics, second.diagnostics,
            "excluded proof mutation byte {mutation} diagnostics must remain deterministic"
        );
        assert_eq!(
            first.cache_keys, second.cache_keys,
            "excluded proof mutation byte {mutation} cache keys must remain deterministic"
        );
        assert!(
            first.cache_keys.ast.is_none(),
            "excluded proof mutation byte {mutation} must not create a reusable AST artifact"
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderSchedule {
    InOrder,
    Reverse,
    RotateLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeterminismProvider {
    schedule: ProviderSchedule,
    beta_fingerprint: u64,
}

const DEFAULT_BETA_FINGERPRINT: u64 = 202;

impl LexicalSummaryProvider for DeterminismProvider {
    fn resolve_imports(
        &self,
        request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
        let mut imports = Vec::new();
        let mut diagnostics = Vec::new();
        let mut uses_type_fixtures = false;

        for (stub_ordinal, stub) in request.import_stubs.iter().enumerate() {
            match stub.path.spelling.as_ref() {
                "alpha" | "beta" | "gamma" | "parser.type_fixtures" => {
                    uses_type_fixtures |= stub.path.spelling.as_ref() == "parser.type_fixtures";
                    imports.push(ResolvedImportEntry {
                        stub_ordinal,
                        stub_span: stub.span,
                        import: ResolvedImport {
                            module_id: ModuleId::new(stub.path.spelling.as_ref()),
                        },
                    });
                }
                unresolved => {
                    diagnostics.push(LexicalEnvironmentDiagnostic {
                        code: LexicalEnvironmentDiagnosticCode::UnresolvedImport,
                        message: Arc::<str>::from(format!(
                            "fixture provider could not resolve `{unresolved}`"
                        )),
                        primary: stub.span,
                        secondary: Vec::new(),
                        import_ordinal: Some(stub_ordinal),
                        module_id: None,
                    });
                }
            }
        }

        let mut summaries = vec![
            summary(
                "alpha",
                101,
                vec![symbol("+*+", "alpha.plus_star_plus", "alpha")],
            ),
            summary(
                "beta",
                self.beta_fingerprint,
                vec![symbol("**", "beta.times", "beta")],
            ),
        ];
        if uses_type_fixtures {
            summaries.push(summary(
                "parser.type_fixtures",
                303,
                vec![
                    operator_symbol(
                        "!",
                        "parser.type_fixtures.factorial",
                        "parser.type_fixtures",
                        UserSymbolArity::exact(1),
                        ExportedOperatorFixity::Postfix,
                        90,
                    ),
                    operator_symbol(
                        "++",
                        "parser.type_fixtures.append",
                        "parser.type_fixtures",
                        UserSymbolArity::exact(2),
                        ExportedOperatorFixity::Infix(ExportedOperatorAssociativity::Left),
                        10,
                    ),
                ],
            ));
        }

        Ok(ResolvedImports {
            imports: schedule_items(imports, self.schedule),
            summaries: schedule_items(summaries, self.schedule),
            diagnostics: schedule_items(diagnostics, self.schedule),
        })
    }
}

fn run_frontend(
    fixture: &PackageFixture,
    relative: &str,
    schedule: ProviderSchedule,
) -> mizar_frontend::orchestration::FrontendOutput<mizar_syntax::SurfaceAst> {
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root())),
        DeterminismProvider {
            schedule,
            beta_fingerprint: DEFAULT_BETA_FINGERPRINT,
        },
        MizarParserSeam,
    );

    frontend
        .run(
            fixture.request(relative),
            &InMemorySessionIdAllocator::new(),
        )
        .expect("determinism fixtures should produce recovered frontend output")
}

fn run_frontend_with_beta_fingerprint(
    fixture: &PackageFixture,
    relative: &str,
    schedule: ProviderSchedule,
    beta_fingerprint: u64,
) -> mizar_frontend::orchestration::FrontendOutput<mizar_syntax::SurfaceAst> {
    let frontend = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root())),
        DeterminismProvider {
            schedule,
            beta_fingerprint,
        },
        MizarParserSeam,
    );

    frontend
        .run(
            fixture.request(relative),
            &InMemorySessionIdAllocator::new(),
        )
        .expect("determinism fixtures should produce recovered frontend output")
}

fn lexical_environment_fingerprint(
    source_id: SourceId,
    import_stubs: &[ImportStub],
    schedule: ProviderSchedule,
) -> LexicalEnvironmentFingerprint {
    lexical_environment(source_id, import_stubs, schedule, DEFAULT_BETA_FINGERPRINT).fingerprint
}

fn lexical_environment(
    source_id: SourceId,
    import_stubs: &[ImportStub],
    schedule: ProviderSchedule,
    beta_fingerprint: u64,
) -> ActiveLexicalEnvironmentResult {
    build_active_lexical_environment(
        &LexicalEnvironmentRequest {
            source_id,
            import_stubs,
            edition: Edition::new("2026"),
        },
        &DeterminismProvider {
            schedule,
            beta_fingerprint,
        },
    )
    .expect("fixture provider should build a lexical environment")
}

fn expected_cache_keys(
    output: &mizar_frontend::orchestration::FrontendOutput<mizar_syntax::SurfaceAst>,
    schedule: ProviderSchedule,
    beta_fingerprint: u64,
) -> FrontendCacheKeys {
    let lexical_environment = lexical_environment(
        output.source.source_id,
        &output.preprocessed.import_stubs,
        schedule,
        beta_fingerprint,
    );
    let token_key = TokenStreamCacheKey::new(
        &output.preprocessed,
        lexical_environment.fingerprint,
        output.tokens.parser_context,
        ParserLexingPlanCacheKey::current(),
    );
    let parser_inputs = ParserInputs::from_active_environment(
        output.source.edition.clone(),
        &lexical_environment.environment,
    );
    let ast = output.ast.as_ref().map(|_| {
        SurfaceAstCacheKey::new(
            token_key.stable_hash(),
            MizarParserSeam.cache_key_version(),
            &parser_inputs,
        )
    });

    FrontendCacheKeys {
        source: SourceUnitCacheKey::from_source(&output.source),
        preprocessed: PreprocessedSourceCacheKey::from_source(&output.source),
        active_lexical_environment: ActiveLexicalEnvironmentCacheKey::new(
            lexical_environment.fingerprint,
        ),
        tokens: token_key,
        ast,
    }
}

fn token_span_signature(tokens: &[Token]) -> Vec<(TokenKind, String, SourceRange)> {
    tokens
        .iter()
        .map(|token| (token.kind, token.text.to_string(), token.span))
        .collect()
}

fn schedule_items<T>(mut items: Vec<T>, schedule: ProviderSchedule) -> Vec<T> {
    match schedule {
        ProviderSchedule::InOrder => items,
        ProviderSchedule::Reverse => {
            items.reverse();
            items
        }
        ProviderSchedule::RotateLeft => {
            if !items.is_empty() {
                items.rotate_left(1);
            }
            items
        }
    }
}

fn summary(
    module_id: &str,
    fingerprint: u64,
    exported_symbols: Vec<ExportedSymbolShape>,
) -> ModuleLexicalSummary {
    ModuleLexicalSummary {
        module_id: ModuleId::new(module_id),
        exported_symbols,
        fingerprint: LexicalSummaryFingerprint::new(fingerprint),
    }
}

fn symbol(spelling: &str, symbol_id: &str, source_module: &str) -> ExportedSymbolShape {
    ExportedSymbolShape {
        spelling: spelling.to_owned(),
        symbol_id: SymbolId::new(symbol_id),
        source_module: ModuleId::new(source_module),
        export_rank: ExportRank::new(0),
        kind: UserSymbolKind::Functor,
        arity: UserSymbolArity::exact(2),
        operator: None,
    }
}

fn operator_symbol(
    spelling: &str,
    symbol_id: &str,
    source_module: &str,
    arity: UserSymbolArity,
    fixity: ExportedOperatorFixity,
    precedence: u8,
) -> ExportedSymbolShape {
    ExportedSymbolShape {
        spelling: spelling.to_owned(),
        symbol_id: SymbolId::new(symbol_id),
        source_module: ModuleId::new(source_module),
        export_rank: ExportRank::new(0),
        kind: UserSymbolKind::Functor,
        arity,
        operator: Some(ExportedOperatorMetadata { fixity, precedence }),
    }
}

struct PackageFixture {
    root: PathBuf,
}

impl PackageFixture {
    fn new() -> Self {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "mizar-frontend-determinism-test-{}-{id}",
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
        SourceUnitRequest {
            snapshot: snapshot_id(1),
            input: SourceInput {
                package_id: PackageId::new("fixture"),
                module_path: ModulePath::new(module_path_from_relative(relative)),
                normalized_path: normalize_path(&self.root, &path).unwrap(),
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

fn snapshot_id(id: u8) -> BuildSnapshotId {
    BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{id:064x}"
    ))
    .unwrap()
}

fn module_path_from_relative(relative: &str) -> String {
    relative
        .strip_prefix("src/")
        .unwrap_or(relative)
        .strip_suffix(".miz")
        .unwrap_or(relative)
        .replace('/', ".")
}

mod comment_metadata {
    use super::*;
    use mizar_frontend::orchestration::{CommentMetadata, FrontendOutput};
    use mizar_frontend::source::{SourceUnit, SourceUnitLoader};
    use mizar_lexer::{CommentKind, SourcePreprocessMapSegment};
    use mizar_session::{
        GeneratedSourceKind, LineMap, LoadingMap, LoadingMapSegment, LoadingOrigin,
        SessionIdAllocator, SourceOrigin, TextRange, hash_text,
    };
    use mizar_syntax::{SurfaceAst, SyntaxKind};

    fn frontend(
        fixture: &PackageFixture,
    ) -> Frontend<FrontendSourceLoader<DiskSourceLoader>, DeterminismProvider, MizarParserSeam>
    {
        Frontend::new(
            FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root())),
            DeterminismProvider {
                schedule: ProviderSchedule::InOrder,
                beta_fingerprint: DEFAULT_BETA_FINGERPRINT,
            },
            MizarParserSeam,
        )
    }

    fn load(
        fixture: &PackageFixture,
        request: SourceUnitRequest,
        ids: &InMemorySessionIdAllocator,
    ) -> SourceUnit {
        FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root()))
            .load_source_unit(request, ids)
            .unwrap()
    }

    fn capture(
        fixture: &PackageFixture,
        text: &str,
    ) -> (FrontendOutput<SurfaceAst>, CommentMetadata) {
        fixture.write("src/metadata.miz", text);
        let source = load(
            fixture,
            fixture.request("src/metadata.miz"),
            &InMemorySessionIdAllocator::new(),
        );
        let (output, metadata) = frontend(fixture)
            .run_loaded_with_comment_metadata(source, text)
            .unwrap();
        (
            output,
            metadata.expect("genuine capture should provide complete metadata"),
        )
    }

    #[test]
    fn disk_preserves_interleaved_raw_comments_and_whitespace() {
        let fixture = PackageFixture::new();
        let text = " \t:: ordinary β\n\n::= multi\nline =:: \t::: doc Ω\n:::";
        let (output, metadata) = capture(&fixture, text);
        assert_eq!(metadata.original_text(), text);
        assert_eq!(metadata.loaded_text(), text);
        assert_eq!(
            metadata
                .comments()
                .iter()
                .map(|comment| (comment.kind, comment.lexeme.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (CommentKind::SingleLine, ":: ordinary β\n"),
                (CommentKind::MultiLine, "::= multi\nline =::"),
                (CommentKind::Documentation, "::: doc Ω\n"),
                (CommentKind::Documentation, ":::"),
            ]
        );
        for comment in metadata.comments() {
            assert_eq!(&text[comment.span.start..comment.span.end], comment.lexeme);
        }
        assert!(
            metadata
                .comments()
                .windows(2)
                .all(|pair| pair[0].span.end <= pair[1].span.start)
        );
        assert_eq!(
            metadata
                .whitespace()
                .iter()
                .map(|range| &text[range.start..range.end])
                .collect::<Vec<_>>(),
            vec![" \t", "\n", " \t"]
        );
        assert_eq!(
            output
                .preprocessed
                .doc_comments
                .iter()
                .map(|comment| comment.raw_body.as_ref())
                .collect::<Vec<_>>(),
            vec![" doc Ω\n", ""]
        );
        assert_eq!(
            metadata.lexical_text(),
            output.preprocessed.lexical_text.as_str()
        );
    }

    #[test]
    fn normalized_disk_and_open_buffer_keep_explicit_frames_and_map_segments() {
        let fixture = PackageFixture::new();
        let original = "\u{feff}theorem::= β =::T: thesis;\r\n";
        let (output, metadata) = capture(&fixture, original);
        let loaded = "theorem::= β =::T: thesis;\n";
        assert_eq!(metadata.original_text(), original);
        assert_eq!(metadata.loaded_text(), loaded);
        assert_eq!(metadata.lexical_text(), "theorem T: thesis;\n");
        assert!(matches!(
            metadata.loading_segments()[0],
            LoadingMapSegment::RemovedLeadingBom {
                original: TextRange { start: 0, end: 3 }
            }
        ));
        assert!(
            matches!(metadata.loading_segments().last().unwrap(), LoadingMapSegment::NormalizedNewline { loaded: newline, original: raw } if *newline == (TextRange { start: loaded.len()-1, end: loaded.len() }) && *raw == (TextRange { start: original.len()-2, end: original.len() }))
        );
        assert_eq!(
            metadata
                .preprocess_map()
                .source_ranges_for_lexical(mizar_lexer::SourceRange { start: 0, end: 7 }),
            Some(vec![mizar_lexer::SourceRange { start: 0, end: 7 }])
        );
        assert!(metadata.preprocess_map().segments.iter().any(|segment| matches!(segment, SourcePreprocessMapSegment::SyntheticWhitespace { lexical, anchor } if *lexical == (mizar_lexer::SourceRange { start: 7, end: 8 }) && &loaded[anchor.start..anchor.end] == "::= β =::")));
        assert!(metadata.preprocess_map().segments.iter().any(|segment| matches!(segment, SourcePreprocessMapSegment::RemovedComment { source, kind: CommentKind::MultiLine } if &loaded[source.start..source.end] == "::= β =::")));
        assert_eq!(
            metadata
                .preprocess_map()
                .source_ranges_for_lexical(mizar_lexer::SourceRange { start: 8, end: 9 }),
            Some(vec![mizar_lexer::SourceRange { start: 17, end: 18 }])
        );
        assert_eq!(
            metadata.comments()[0].span,
            mizar_lexer::SourceRange { start: 7, end: 17 }
        );
        assert_eq!(
            &metadata.original_text()[10..20],
            metadata.comments()[0].lexeme
        );
        assert_eq!(metadata.source_hash(), &output.source.source_hash);
        let mut previous = None;
        for version in [3, 4] {
            let mut request = fixture.request("src/metadata.miz");
            request.input.origin = SourceOriginInput::OpenBuffer {
                uri: format!("file://{}", fixture.path("src/metadata.miz").display()),
                expected_version: version,
                actual_version: version,
                text: Arc::from(original),
            };
            let input = request.input.clone();
            let source = load(&fixture, request, &InMemorySessionIdAllocator::new());
            let (output, metadata) = frontend(&fixture)
                .run_loaded_with_comment_metadata(source, original)
                .unwrap();
            let metadata = metadata.unwrap();
            assert_eq!(metadata.origin(), &SourceOrigin::OpenBuffer { version });
            assert_eq!(metadata.original_text(), original);
            assert_eq!(metadata.loaded_text(), loaded);
            let bytes = output.canonical_open_buffer_bytes(&input).unwrap();
            let restored = FrontendOutput::from_canonical_open_buffer_bytes(
                &bytes,
                output.source.source_id,
                &input,
            )
            .unwrap();
            assert_eq!(restored, output);
            if let Some(previous) = previous {
                assert_ne!(metadata, previous);
            }
            previous = Some(metadata);
        }
    }

    #[test]
    fn real_ast_preserves_ordered_wrappers_annotations_nested_candidates_and_recovery() {
        let fixture = PackageFixture::new();
        let text = concat!(
            "export Demo;\n",
            "@[label(\"attached\")] public theorem Named: thesis;\n",
            "@show_type(x);\n",
            "definition\n  algorithm choose() do return; end;\nend;\n",
            "registration\n  cluster Exists: non empty set; existence by Ref;\nend;\n",
            "definition algorithm broken() do return; end;\n",
        );
        let (output, metadata) = capture(&fixture, text);
        let boundaries = metadata.boundaries();
        for kind in [
            SyntaxKind::ExportItem,
            SyntaxKind::VisibleItem,
            SyntaxKind::Annotation,
            SyntaxKind::StandaloneDiagnosticAnnotation,
            SyntaxKind::TheoremItem,
            SyntaxKind::DefinitionBlockItem,
            SyntaxKind::AlgorithmDefinition,
            SyntaxKind::RegistrationBlockItem,
            SyntaxKind::ExistentialRegistration,
        ] {
            assert!(
                boundaries.iter().any(|boundary| boundary.kind() == kind),
                "missing {kind:?}"
            );
        }
        assert!(
            boundaries
                .windows(2)
                .all(|pair| pair[0].range().start <= pair[1].range().start)
        );
        let (visible_index, visible) = boundaries
            .iter()
            .enumerate()
            .find(|(_, boundary)| boundary.kind() == SyntaxKind::VisibleItem)
            .unwrap();
        assert_eq!(
            &text[visible.range().start..visible.range().end],
            "@[label(\"attached\")] public theorem Named: thesis;"
        );
        assert_eq!(visible.parent(), None);
        let (annotation_index, attached) = boundaries
            .iter()
            .enumerate()
            .find(|(_, boundary)| boundary.kind() == SyntaxKind::Annotation)
            .unwrap();
        assert_eq!(
            &text[attached.range().start..attached.range().end],
            "@[label(\"attached\")]"
        );
        assert_eq!(attached.parent(), Some(visible_index));
        let library = boundaries
            .iter()
            .find(|boundary| boundary.kind() == SyntaxKind::LibraryAnnotation)
            .unwrap();
        assert_eq!(library.range(), attached.range());
        assert_eq!(library.parent(), Some(annotation_index));
        let standalone = boundaries
            .iter()
            .find(|boundary| boundary.kind() == SyntaxKind::StandaloneDiagnosticAnnotation)
            .unwrap();
        assert_eq!(
            &text[standalone.range().start..standalone.range().end],
            "@show_type(x)"
        );
        let standalone_item = &boundaries[standalone.parent().unwrap()];
        assert_eq!(standalone_item.kind(), SyntaxKind::StatementItem);
        assert_eq!(standalone_item.range(), standalone.range());
        assert_eq!(standalone_item.parent(), None);
        let named = boundaries
            .iter()
            .find(|boundary| boundary.kind() == SyntaxKind::TheoremItem)
            .unwrap();
        assert_eq!(
            &text[named.range().start..named.range().end],
            "theorem Named: thesis;"
        );
        assert_eq!(named.parent(), Some(visible_index));
        let algorithm = boundaries
            .iter()
            .find(|boundary| {
                boundary.kind() == SyntaxKind::AlgorithmDefinition
                    && text[boundary.range().start..boundary.range().end].contains("choose")
            })
            .unwrap();
        assert_eq!(
            &text[algorithm.range().start..algorithm.range().end],
            "algorithm choose() do return; end;"
        );
        assert_eq!(
            boundaries[algorithm.parent().unwrap()].kind(),
            SyntaxKind::DefinitionBlockItem
        );
        let cluster = boundaries
            .iter()
            .find(|boundary| boundary.kind() == SyntaxKind::ExistentialRegistration)
            .unwrap();
        assert_eq!(
            boundaries[cluster.parent().unwrap()].kind(),
            SyntaxKind::RegistrationBlockItem
        );
        assert!(boundaries.iter().any(|boundary| boundary.recovered()));
        assert!(
            !output.diagnostics.is_empty(),
            "missing end remains a real parser recovery"
        );
        for (index, boundary) in boundaries.iter().enumerate() {
            let range = boundary.range();
            assert!(text.get(range.start..range.end).is_some());
            if let Some(parent) = boundary.parent() {
                assert!(parent < index);
                let outer = boundaries[parent].range();
                assert!(outer.start <= range.start && range.end <= outer.end);
            }
        }
    }

    #[test]
    fn available_empty_and_empty_doc_differ_and_comment_edits_keep_lexical_reuse() {
        let fixture = PackageFixture::new();
        let (_, empty) = capture(&fixture, "theorem T: thesis;\n");
        assert!(empty.comments().is_empty());
        let (_, doc) = capture(&fixture, ":::");
        assert_eq!(doc.comments().len(), 1);
        assert_eq!(doc.comments()[0].lexeme, ":::");
        let (first, first_metadata) = capture(&fixture, COMMENT_EQUIVALENT_SOURCE_A);
        let (second, second_metadata) = capture(&fixture, COMMENT_EQUIVALENT_SOURCE_B);
        assert_ne!(first_metadata, second_metadata);
        assert_ne!(first_metadata.source_hash(), second_metadata.source_hash());
        assert_eq!(
            first_metadata.lexical_text(),
            second_metadata.lexical_text()
        );
        assert_eq!(first.cache_keys.tokens, second.cache_keys.tokens);
        assert_eq!(first.cache_keys.ast, second.cache_keys.ast);
    }

    #[test]
    fn invalid_capture_identity_maps_and_origins_refuse_metadata_without_pipeline_changes() {
        let fixture = PackageFixture::new();
        let text = "::: β\ntheorem T: thesis;\n";
        fixture.write("src/metadata.miz", text);
        let ids = InMemorySessionIdAllocator::new();
        let source = load(&fixture, fixture.request("src/metadata.miz"), &ids);
        let foreign_id = ids.next_source_id(snapshot_id(1)).unwrap();
        let mut variants = vec![(source.clone(), "::: changed\ntheorem T: thesis;\n")];
        let mut bad = source.clone();
        bad.source_hash = hash_text("foreign hash");
        variants.push((bad, text));
        let mut bad = source.clone();
        bad.line_map = LineMap::new(foreign_id, text);
        variants.push((bad, text));
        let mut bad = source.clone();
        bad.line_map = LineMap::new(source.source_id, "foreign text");
        variants.push((bad, text));
        let identity = LoadingMap::identity(
            source.source_id,
            text,
            LoadingOrigin::DiskBytes {
                normalized_path: source.normalized_path.clone(),
            },
        );
        let mut bad = source.clone();
        let mut map = identity.clone();
        map.source_id = foreign_id;
        bad.loading_map = Some(map);
        variants.push((bad, text));
        let mut bad = source.clone();
        let mut map = identity.clone();
        map.origin = LoadingOrigin::Generated;
        bad.loading_map = Some(map);
        variants.push((bad, text));
        let mut bad = source.clone();
        let mut map = identity;
        map.segments = vec![
            LoadingMapSegment::Original {
                loaded: TextRange { start: 0, end: 5 },
                original: TextRange { start: 0, end: 5 },
            },
            LoadingMapSegment::Original {
                loaded: TextRange {
                    start: 5,
                    end: text.len(),
                },
                original: TextRange {
                    start: 5,
                    end: text.len(),
                },
            },
        ];
        bad.loading_map = Some(map);
        variants.push((bad, text));
        let mut bad = source.clone();
        bad.origin = SourceOrigin::OpenBuffer { version: -1 };
        variants.push((bad, text));
        let mut bad = source;
        bad.origin = SourceOrigin::Generated {
            generator: GeneratedSourceKind::new("test"),
        };
        variants.push((bad, text));
        for (source, captured) in variants {
            let ordinary = frontend(&fixture).run_loaded(source.clone());
            let captured = frontend(&fixture).run_loaded_with_comment_metadata(source, captured);
            match (ordinary, captured) {
                (Ok(ordinary), Ok((output, metadata))) => {
                    assert!(metadata.is_none());
                    assert_eq!(ordinary, output);
                }
                (Err(ordinary), Err(captured)) => assert_eq!(ordinary, captured),
                other => panic!("metadata must preserve pipeline acceptance: {other:?}"),
            }
        }
    }

    #[test]
    fn equivalent_captures_ignore_runtime_ids_and_local_paths_and_retain_storage() {
        let first_fixture = PackageFixture::new();
        let second_fixture = PackageFixture::new();
        let ids = InMemorySessionIdAllocator::new();
        for text in [
            "::: docs β\ntheorem::= Ω =::T: thesis;\n",
            "\u{feff}::: docs β\ntheorem::= Ω =::T: thesis;\n",
            "::: docs β\r\ntheorem::= Ω =::T: thesis;\r\n",
            "\u{feff}::: docs β\r\ntheorem::= Ω =::T: thesis;\r\n",
        ] {
            for open_buffer in [false, true] {
                let mut baseline = None;
                for (index, fixture) in [&first_fixture, &second_fixture, &first_fixture]
                    .into_iter()
                    .enumerate()
                {
                    fixture.write("src/metadata.miz", text);
                    let mut request = fixture.request("src/metadata.miz");
                    if open_buffer {
                        request.input.origin = SourceOriginInput::OpenBuffer {
                            uri: format!("file://{}", fixture.path("src/metadata.miz").display()),
                            expected_version: 3,
                            actual_version: 3,
                            text: Arc::from(text),
                        };
                    }
                    let source = load(fixture, request.clone(), &ids);
                    let (output, metadata) = frontend(fixture)
                        .run_loaded_with_comment_metadata(source.clone(), text)
                        .unwrap();
                    let metadata = metadata.unwrap();
                    assert_eq!(metadata.package_id(), &source.package_id);
                    assert_eq!(metadata.module_path(), &source.module_path);
                    assert_eq!(metadata.normalized_path(), &source.normalized_path);
                    assert_eq!(metadata.edition(), &source.edition);
                    assert!(metadata.matches_output(&output));
                    let debug = format!("{metadata:?}");
                    for root in [first_fixture.root(), second_fixture.root()] {
                        assert!(!debug.contains(root.to_str().unwrap()));
                    }
                    let bytes = if open_buffer {
                        output.canonical_open_buffer_bytes(&request.input)
                    } else {
                        output.canonical_disk_bytes()
                    }
                    .unwrap();
                    if let Some((previous_output, previous_metadata, previous_debug)) = &baseline {
                        let previous_output: &FrontendOutput<SurfaceAst> = previous_output;
                        assert_ne!(previous_output.source.source_id, source.source_id);
                        if index == 1 {
                            assert_ne!(previous_output.source.file_path, source.file_path);
                        }
                        assert_eq!(&metadata, previous_metadata);
                        assert_eq!(&debug, previous_debug);
                        assert!(metadata.matches_output(previous_output));
                        assert!(previous_metadata.matches_output(&output));
                    } else {
                        baseline = Some((output.clone(), metadata.clone(), debug));
                    }
                    let restored = FrontendOutput::from_canonical_storage_bytes(
                        &bytes,
                        ids.next_source_id(snapshot_id(1)).unwrap(),
                        &request.input,
                    )
                    .unwrap();
                    assert!(metadata.matches_output(&restored));
                    let restored_bytes = if open_buffer {
                        restored.canonical_open_buffer_bytes(&request.input)
                    } else {
                        restored.canonical_disk_bytes()
                    }
                    .unwrap();
                    assert_eq!(restored_bytes, bytes);
                    let (replayed, replay_metadata) = frontend(fixture)
                        .run_loaded_with_comment_metadata(restored.source.clone(), text)
                        .unwrap();
                    assert_eq!(replayed, restored);
                    assert_eq!(replay_metadata.as_ref(), Some(&metadata));

                    if open_buffer
                        && index == 0
                        && text.starts_with('\u{feff}')
                        && text.contains("\r\n")
                    {
                        let mut changed_source = output.clone();
                        changed_source.source.source_text = Arc::from("theorem Other: thesis;\n");
                        let mut changed_loading = output.clone();
                        changed_loading
                            .source
                            .loading_map
                            .as_mut()
                            .unwrap()
                            .segments
                            .clear();
                        let mut changed_preprocess = output.clone();
                        changed_preprocess
                            .preprocessed
                            .source_map
                            .preprocess_map
                            .segments
                            .clear();
                        let mut changed_tokens = output.clone();
                        changed_tokens.tokens.tokens.remove(0);
                        let mut changed_ast = output.clone();
                        assert!(changed_ast.ast.take().is_some());
                        let mut changed_version = output.clone();
                        changed_version.source.origin = SourceOrigin::OpenBuffer { version: 4 };
                        for (label, changed) in [
                            ("source", changed_source),
                            ("loading map", changed_loading),
                            ("preprocess map", changed_preprocess),
                            ("tokens", changed_tokens),
                            ("AST", changed_ast),
                            ("version", changed_version),
                        ] {
                            assert!(
                                !metadata.matches_output(&changed),
                                "accepted changed {label}"
                            );
                        }
                    }
                }
            }
        }
    }
}
