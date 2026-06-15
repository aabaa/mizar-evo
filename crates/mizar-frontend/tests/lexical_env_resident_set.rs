use mizar_frontend::lexical_env::{
    ExportRank, ExportedSymbolShape, FrontendLexicalEnvironmentError, LexicalEnvironmentRequest,
    LexicalSummaryFingerprint, LexicalSummaryProvider, ModuleId, ModuleLexicalSummary,
    ResolvedImport, ResolvedImportEntry, ResolvedImports, SymbolId, UserSymbolArity,
    UserSymbolKind, build_active_lexical_environment,
};
use mizar_frontend::preprocess::{ImportStub, ImportStubPath};
use mizar_session::{
    BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
    SourceRange,
};
use std::sync::{Arc, Mutex};

#[test]
fn provider_is_queried_once_for_current_file_imports_without_closure_expansion() {
    let source_id = source_id(23);
    let stubs = vec![
        import_stub(source_id, 0, 13, "alpha"),
        import_stub(source_id, 15, 28, "beta"),
    ];
    let provider = RecordingProvider::default();

    let result = build_active_lexical_environment(
        &LexicalEnvironmentRequest {
            source_id,
            import_stubs: &stubs,
            edition: Edition::new("2026"),
        },
        &provider,
    )
    .expect("direct lexical summaries should build an active environment");

    let calls = provider.calls();
    assert_eq!(
        calls,
        vec![ProviderCall {
            source_id,
            edition: Edition::new("2026"),
            import_spellings: vec!["alpha".to_owned(), "beta".to_owned()],
            import_spans: stubs.iter().map(|stub| stub.span).collect(),
        }],
        "the frontend should ask the provider exactly once, scoped to the current file's ImportStub list"
    );

    assert_symbol_candidate(
        result.environment.user_symbol("++").unwrap(),
        ExpectedCandidate {
            spelling: "++",
            symbol_id: "alpha.plus",
            source_module: "alpha",
            imported_module: "alpha",
            import_ordinal: 0,
            export_rank: 1,
            kind: UserSymbolKind::Functor,
            arity: UserSymbolArity::exact(2),
        },
    );
    assert_symbol_candidate(
        result.environment.user_symbol("**").unwrap(),
        ExpectedCandidate {
            spelling: "**",
            symbol_id: "beta.times",
            source_module: "beta",
            imported_module: "beta",
            import_ordinal: 1,
            export_rank: 2,
            kind: UserSymbolKind::Predicate,
            arity: UserSymbolArity::range(1, 2),
        },
    );
    assert!(
        result.environment.user_symbol("##").is_none(),
        "transitive dependency symbols are not resident unless they appear in a direct ModuleLexicalSummary"
    );
    assert!(result.diagnostics.is_empty());
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderCall {
    source_id: SourceId,
    edition: Edition,
    import_spellings: Vec<String>,
    import_spans: Vec<SourceRange>,
}

#[derive(Debug, Default)]
struct RecordingProvider {
    calls: Mutex<Vec<ProviderCall>>,
}

impl RecordingProvider {
    fn calls(&self) -> Vec<ProviderCall> {
        self.calls.lock().unwrap().clone()
    }
}

impl LexicalSummaryProvider for RecordingProvider {
    fn resolve_imports(
        &self,
        request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
        self.calls.lock().unwrap().push(ProviderCall {
            source_id: request.source_id,
            edition: request.edition.clone(),
            import_spellings: request
                .import_stubs
                .iter()
                .map(|stub| stub.path.spelling.to_string())
                .collect(),
            import_spans: request.import_stubs.iter().map(|stub| stub.span).collect(),
        });

        let imports = request
            .import_stubs
            .iter()
            .enumerate()
            .filter_map(|(stub_ordinal, stub)| match stub.path.spelling.as_ref() {
                "alpha" | "beta" => Some(ResolvedImportEntry {
                    stub_ordinal,
                    stub_span: stub.span,
                    import: ResolvedImport {
                        module_id: ModuleId::new(stub.path.spelling.as_ref()),
                    },
                }),
                _ => None,
            })
            .collect();

        Ok(ResolvedImports {
            imports,
            summaries: vec![
                summary(
                    "alpha",
                    101,
                    vec![symbol(
                        "++",
                        "alpha.plus",
                        "alpha",
                        ExportRank::new(1),
                        UserSymbolKind::Functor,
                        UserSymbolArity::exact(2),
                    )],
                ),
                summary(
                    "beta",
                    202,
                    vec![symbol(
                        "**",
                        "beta.times",
                        "beta",
                        ExportRank::new(2),
                        UserSymbolKind::Predicate,
                        UserSymbolArity::range(1, 2),
                    )],
                ),
                summary(
                    "transitive",
                    303,
                    vec![symbol(
                        "##",
                        "transitive.hash_hash",
                        "transitive",
                        ExportRank::new(3),
                        UserSymbolKind::Functor,
                        UserSymbolArity::exact(0),
                    )],
                ),
            ],
            diagnostics: Vec::new(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct ExpectedCandidate<'a> {
    spelling: &'a str,
    symbol_id: &'a str,
    source_module: &'a str,
    imported_module: &'a str,
    import_ordinal: usize,
    export_rank: u32,
    kind: UserSymbolKind,
    arity: UserSymbolArity,
}

fn assert_symbol_candidate(
    actual: &mizar_frontend::lexical_env::UserSymbolCandidate,
    expected: ExpectedCandidate<'_>,
) {
    assert_eq!(actual.spelling, expected.spelling);
    assert_eq!(actual.symbol_id, SymbolId::new(expected.symbol_id));
    assert_eq!(actual.source_module, ModuleId::new(expected.source_module));
    assert_eq!(
        actual.imported_module,
        ModuleId::new(expected.imported_module)
    );
    assert_eq!(actual.import_ordinal, expected.import_ordinal);
    assert_eq!(actual.export_rank, ExportRank::new(expected.export_rank));
    assert_eq!(actual.kind, expected.kind);
    assert_eq!(actual.arity, expected.arity);
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

fn symbol(
    spelling: &str,
    symbol_id: &str,
    source_module: &str,
    export_rank: ExportRank,
    kind: UserSymbolKind,
    arity: UserSymbolArity,
) -> ExportedSymbolShape {
    ExportedSymbolShape {
        spelling: spelling.to_owned(),
        symbol_id: SymbolId::new(symbol_id),
        source_module: ModuleId::new(source_module),
        export_rank,
        kind,
        arity,
        operator: None,
    }
}

fn import_stub(source_id: SourceId, start: usize, end: usize, spelling: &str) -> ImportStub {
    let span = SourceRange {
        source_id,
        start,
        end,
    };
    ImportStub {
        path: ImportStubPath {
            spelling: Arc::from(spelling),
            relative: None,
            components: spelling.split('.').map(Arc::<str>::from).collect(),
            source_segments: vec![span],
            span,
        },
        alias: None,
        span,
    }
}

fn source_id(byte: u8) -> SourceId {
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot_id(byte))
        .unwrap()
}

fn snapshot_id(byte: u8) -> BuildSnapshotId {
    let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .unwrap()
}
