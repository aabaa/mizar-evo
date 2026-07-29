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
