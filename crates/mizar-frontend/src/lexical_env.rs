use crate::preprocess::ImportStub;
use mizar_session::{Edition, SourceAnchor, SourceId, SourceRange};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

pub use mizar_lexer::{
    ActiveLexicalEnvironment, ExportRank, ExportedSymbolShape, LexicalEnvironmentError,
    LexicalEnvironmentFingerprint, LexicalSummaryFingerprint, ModuleId, ModuleLexicalSummary,
    ResolvedImport, SymbolId, UserSymbolArity, UserSymbolCandidate, UserSymbolIndex,
    UserSymbolKind, UserSymbolKindSet,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalEnvironmentRequest<'a> {
    pub source_id: SourceId,
    pub import_stubs: &'a [ImportStub],
    pub edition: Edition,
}

pub trait LexicalSummaryProvider {
    fn resolve_imports(
        &self,
        request: &LexicalEnvironmentRequest<'_>,
    ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImports {
    pub imports: Vec<ResolvedImportEntry>,
    pub summaries: Vec<ModuleLexicalSummary>,
    pub diagnostics: Vec<LexicalEnvironmentDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImportEntry {
    pub stub_ordinal: usize,
    pub stub_span: SourceRange,
    pub import: ResolvedImport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveLexicalEnvironmentResult {
    pub environment: ActiveLexicalEnvironment,
    pub fingerprint: LexicalEnvironmentFingerprint,
    pub diagnostics: Vec<LexicalEnvironmentDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendLexicalEnvironmentError {
    ProviderUnavailable { message: String },
    MalformedSummary { source: LexicalEnvironmentError },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalEnvironmentDiagnostic {
    pub code: LexicalEnvironmentDiagnosticCode,
    pub message: Arc<str>,
    pub primary: SourceRange,
    pub secondary: Vec<SourceAnchor>,
    pub import_ordinal: Option<usize>,
    pub module_id: Option<ModuleId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexicalEnvironmentDiagnosticCode {
    UnresolvedImport,
    MissingSummary,
    UserSymbolImportConflict,
    InvalidUserSymbolSpelling,
    InvalidUserSymbolArity,
    ReservedWordCollision,
    ReservedSymbolCollision,
}

pub fn build_active_lexical_environment(
    request: &LexicalEnvironmentRequest<'_>,
    provider: &dyn LexicalSummaryProvider,
) -> Result<ActiveLexicalEnvironmentResult, FrontendLexicalEnvironmentError> {
    let resolved = provider.resolve_imports(request)?;
    let active_imports = canonical_resolved_imports(resolved.imports)
        .into_iter()
        .map(|entry| entry.import)
        .collect::<Vec<_>>();
    let environment = mizar_lexer::build_lexical_environment(&active_imports, &resolved.summaries)
        .map_err(|source| FrontendLexicalEnvironmentError::MalformedSummary { source })?;

    Ok(ActiveLexicalEnvironmentResult {
        fingerprint: environment.fingerprint,
        environment,
        diagnostics: resolved.diagnostics,
    })
}

fn canonical_resolved_imports(mut imports: Vec<ResolvedImportEntry>) -> Vec<ResolvedImportEntry> {
    imports.sort_by(|left, right| {
        left.stub_ordinal
            .cmp(&right.stub_ordinal)
            .then_with(|| left.stub_span.start.cmp(&right.stub_span.start))
            .then_with(|| left.stub_span.end.cmp(&right.stub_span.end))
            .then_with(|| left.import.module_id.cmp(&right.import.module_id))
    });

    let mut seen = BTreeSet::new();
    let mut canonical = Vec::new();
    for entry in imports {
        if seen.insert(entry.import.module_id.clone()) {
            canonical.push(entry);
        }
    }
    canonical
}

impl fmt::Display for FrontendLexicalEnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderUnavailable { message } => {
                write!(f, "lexical-summary provider is unavailable: {message}")
            }
            Self::MalformedSummary { source } => {
                write!(
                    f,
                    "lexical-summary provider returned malformed summary data: {source}"
                )
            }
        }
    }
}

impl Error for FrontendLexicalEnvironmentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProviderUnavailable { .. } => None,
            Self::MalformedSummary { source } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ActiveLexicalEnvironmentResult, ExportRank, ExportedSymbolShape,
        FrontendLexicalEnvironmentError, LexicalEnvironmentDiagnostic,
        LexicalEnvironmentDiagnosticCode, LexicalEnvironmentRequest, LexicalSummaryFingerprint,
        LexicalSummaryProvider, ModuleId, ModuleLexicalSummary, ResolvedImport,
        ResolvedImportEntry, ResolvedImports, SymbolId, UserSymbolArity, UserSymbolKind,
        build_active_lexical_environment,
    };
    use crate::preprocess::{ImportStub, ImportStubPath};
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceRange,
    };
    use std::sync::Arc;

    #[test]
    fn provider_imports_and_summaries_build_user_symbol_index_in_stub_order() {
        let source_id = source_id(1);
        let stubs = vec![
            import_stub(source_id, 0, 13, "beta"),
            import_stub(source_id, 15, 29, "alpha"),
        ];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![
                    resolved_entry(&stubs, 1, "alpha"),
                    resolved_entry(&stubs, 0, "beta"),
                ],
                summaries: vec![
                    summary("alpha", 11, vec![symbol("++", "alpha.plus", "alpha")]),
                    summary("beta", 22, vec![symbol("**", "beta.times", "beta")]),
                ],
                diagnostics: Vec::new(),
            },
        };

        let result = build(&stubs, source_id, &provider).unwrap();

        let plus = result.environment.user_symbol("++").unwrap();
        assert_eq!(plus.imported_module, ModuleId::new("alpha"));
        assert_eq!(plus.source_module, ModuleId::new("alpha"));
        assert_eq!(
            plus.import_ordinal, 1,
            "alpha sorts before beta but was imported second"
        );
        let times = result.environment.user_symbol("**").unwrap();
        assert_eq!(times.imported_module, ModuleId::new("beta"));
        assert_eq!(times.source_module, ModuleId::new("beta"));
        assert_eq!(
            times.import_ordinal, 0,
            "beta sorts after alpha but was imported first"
        );
        assert_eq!(result.fingerprint, result.environment.fingerprint);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn duplicate_modules_are_deduplicated_before_the_lexer_call_and_provider_diagnostics_remain() {
        let source_id = source_id(2);
        let stubs = vec![
            import_stub(source_id, 0, 14, "alpha"),
            import_stub(source_id, 16, 34, "alpha.duplicate"),
            import_stub(source_id, 36, 49, "beta"),
        ];
        let duplicate_span = stubs[1].span;
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![
                    resolved_entry(&stubs, 0, "alpha"),
                    resolved_entry(&stubs, 1, "alpha"),
                    resolved_entry(&stubs, 2, "beta"),
                ],
                summaries: vec![
                    summary("alpha", 11, vec![symbol("++", "alpha.plus", "alpha")]),
                    summary("beta", 22, vec![symbol("**", "beta.times", "beta")]),
                ],
                diagnostics: vec![LexicalEnvironmentDiagnostic {
                    code: LexicalEnvironmentDiagnosticCode::UnresolvedImport,
                    message: Arc::from("duplicate provenance diagnostic"),
                    primary: duplicate_span,
                    secondary: Vec::new(),
                    import_ordinal: Some(1),
                    module_id: Some(ModuleId::new("alpha")),
                }],
            },
        };

        let result = build(&stubs, source_id, &provider).unwrap();

        let plus = result.environment.user_symbol("++").unwrap();
        assert_eq!(plus.imported_module, ModuleId::new("alpha"));
        assert_eq!(plus.import_ordinal, 0);
        let times = result.environment.user_symbol("**").unwrap();
        assert_eq!(times.imported_module, ModuleId::new("beta"));
        assert_eq!(
            times.import_ordinal, 1,
            "the second active module should use canonical import order, not the original stub ordinal"
        );
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(result.diagnostics[0].primary, duplicate_span);
        assert_eq!(result.diagnostics[0].import_ordinal, Some(1));
        assert_eq!(
            result.diagnostics[0].module_id,
            Some(ModuleId::new("alpha"))
        );
    }

    #[test]
    fn reserved_tables_are_present_without_imports() {
        let source_id = source_id(3);
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: Vec::new(),
                summaries: Vec::new(),
                diagnostics: Vec::new(),
            },
        };

        let result = build(&[], source_id, &provider).unwrap();

        assert_eq!(
            result.environment.reserved_word("definition"),
            Some("definition")
        );
        assert_eq!(result.environment.reserved_symbol(":="), Some(":="));
        assert_eq!(result.fingerprint, result.environment.fingerprint);
    }

    #[test]
    fn provider_infrastructure_errors_use_frontend_owned_error() {
        let source_id = source_id(4);
        let provider = FailingProvider;

        let error = build(&[], source_id, &provider).unwrap_err();

        assert_eq!(
            error,
            FrontendLexicalEnvironmentError::ProviderUnavailable {
                message: "fixture provider unavailable".to_owned(),
            }
        );
    }

    struct FakeProvider {
        resolved: ResolvedImports,
    }

    impl LexicalSummaryProvider for FakeProvider {
        fn resolve_imports(
            &self,
            request: &LexicalEnvironmentRequest<'_>,
        ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
            assert_eq!(request.edition, Edition::new("2026"));
            assert_eq!(
                request.import_stubs.len(),
                self.resolved.imports.len().min(3)
            );
            Ok(self.resolved.clone())
        }
    }

    struct FailingProvider;

    impl LexicalSummaryProvider for FailingProvider {
        fn resolve_imports(
            &self,
            _request: &LexicalEnvironmentRequest<'_>,
        ) -> Result<ResolvedImports, FrontendLexicalEnvironmentError> {
            Err(FrontendLexicalEnvironmentError::ProviderUnavailable {
                message: "fixture provider unavailable".to_owned(),
            })
        }
    }

    fn build(
        import_stubs: &[ImportStub],
        source_id: mizar_session::SourceId,
        provider: &dyn LexicalSummaryProvider,
    ) -> Result<ActiveLexicalEnvironmentResult, FrontendLexicalEnvironmentError> {
        build_active_lexical_environment(
            &LexicalEnvironmentRequest {
                source_id,
                import_stubs,
                edition: Edition::new("2026"),
            },
            provider,
        )
    }

    fn resolved_entry(
        stubs: &[ImportStub],
        stub_ordinal: usize,
        module_id: &str,
    ) -> ResolvedImportEntry {
        ResolvedImportEntry {
            stub_ordinal,
            stub_span: stubs[stub_ordinal].span,
            import: ResolvedImport {
                module_id: ModuleId::new(module_id),
            },
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
        }
    }

    fn import_stub(
        source_id: mizar_session::SourceId,
        start: usize,
        end: usize,
        spelling: &str,
    ) -> ImportStub {
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

    fn source_id(byte: u8) -> mizar_session::SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(byte))
            .unwrap()
    }

    fn snapshot_id(byte: u8) -> BuildSnapshotId {
        let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }
}
