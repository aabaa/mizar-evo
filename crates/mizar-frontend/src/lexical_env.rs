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
    MalformedProviderProvenance { message: String },
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

impl LexicalEnvironmentDiagnosticCode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::UnresolvedImport => "unresolved_import",
            Self::MissingSummary => "missing_summary",
            Self::UserSymbolImportConflict => "user_symbol_import_conflict",
            Self::InvalidUserSymbolSpelling => "invalid_user_symbol_spelling",
            Self::InvalidUserSymbolArity => "invalid_user_symbol_arity",
            Self::ReservedWordCollision => "reserved_word_collision",
            Self::ReservedSymbolCollision => "reserved_symbol_collision",
        }
    }
}

pub fn build_active_lexical_environment(
    request: &LexicalEnvironmentRequest<'_>,
    provider: &dyn LexicalSummaryProvider,
) -> Result<ActiveLexicalEnvironmentResult, FrontendLexicalEnvironmentError> {
    let resolved = provider.resolve_imports(request)?;
    let ResolvedImports {
        imports,
        summaries,
        mut diagnostics,
    } = resolved;
    validate_resolved_import_provenance(request, &imports)?;
    validate_provider_diagnostic_provenance(request, &diagnostics)?;
    diagnose_unresolved_imports(request, &imports, &mut diagnostics);
    let active_entries = filter_imports_with_summaries(
        canonical_resolved_imports(imports),
        &summaries,
        &mut diagnostics,
    );
    let environment =
        build_lexical_environment_with_retries(active_entries, &summaries, &mut diagnostics)?;

    Ok(ActiveLexicalEnvironmentResult {
        fingerprint: environment.fingerprint,
        environment,
        diagnostics,
    })
}

fn validate_resolved_import_provenance(
    request: &LexicalEnvironmentRequest<'_>,
    imports: &[ResolvedImportEntry],
) -> Result<(), FrontendLexicalEnvironmentError> {
    for entry in imports {
        let Some(stub) = request.import_stubs.get(entry.stub_ordinal) else {
            return Err(malformed_provider_provenance(format!(
                "resolved import for module `{}` references missing import stub ordinal {}",
                entry.import.module_id.as_str(),
                entry.stub_ordinal
            )));
        };

        if entry.stub_span.source_id != request.source_id {
            return Err(malformed_provider_provenance(format!(
                "resolved import for module `{}` references source `{:?}` but the lexical environment request is for source `{:?}`",
                entry.import.module_id.as_str(),
                entry.stub_span.source_id,
                request.source_id
            )));
        }

        if entry.stub_span != stub.span {
            return Err(malformed_provider_provenance(format!(
                "resolved import for module `{}` references stale span {:?} for import stub ordinal {}; expected {:?}",
                entry.import.module_id.as_str(),
                entry.stub_span,
                entry.stub_ordinal,
                stub.span
            )));
        }
    }

    Ok(())
}

fn validate_provider_diagnostic_provenance(
    request: &LexicalEnvironmentRequest<'_>,
    diagnostics: &[LexicalEnvironmentDiagnostic],
) -> Result<(), FrontendLexicalEnvironmentError> {
    for diagnostic in diagnostics {
        if diagnostic.primary.source_id != request.source_id {
            return Err(malformed_provider_provenance(format!(
                "provider diagnostic `{}` references source `{:?}` but the lexical environment request is for source `{:?}`",
                diagnostic.code.as_str(),
                diagnostic.primary.source_id,
                request.source_id
            )));
        }

        if let Some(stub_ordinal) = diagnostic.import_ordinal {
            let Some(stub) = request.import_stubs.get(stub_ordinal) else {
                return Err(malformed_provider_provenance(format!(
                    "provider diagnostic `{}` references missing import stub ordinal {}",
                    diagnostic.code.as_str(),
                    stub_ordinal
                )));
            };
            if diagnostic.primary != stub.span {
                return Err(malformed_provider_provenance(format!(
                    "provider diagnostic `{}` references stale span {:?} for import stub ordinal {}; expected {:?}",
                    diagnostic.code.as_str(),
                    diagnostic.primary,
                    stub_ordinal,
                    stub.span
                )));
            }
        }

        for anchor in &diagnostic.secondary {
            validate_provider_diagnostic_anchor(request, diagnostic.code, anchor)?;
        }
    }

    Ok(())
}

fn validate_provider_diagnostic_anchor(
    request: &LexicalEnvironmentRequest<'_>,
    code: LexicalEnvironmentDiagnosticCode,
    anchor: &SourceAnchor,
) -> Result<(), FrontendLexicalEnvironmentError> {
    match anchor {
        SourceAnchor::Range(range) if range.source_id != request.source_id => {
            Err(malformed_provider_provenance(format!(
                "provider diagnostic `{}` secondary range references source `{:?}` but the lexical environment request is for source `{:?}`",
                code.as_str(),
                range.source_id,
                request.source_id
            )))
        }
        SourceAnchor::Point { source_id, .. } if *source_id != request.source_id => {
            Err(malformed_provider_provenance(format!(
                "provider diagnostic `{}` secondary point references source `{:?}` but the lexical environment request is for source `{:?}`",
                code.as_str(),
                source_id,
                request.source_id
            )))
        }
        _ => Ok(()),
    }
}

fn diagnose_unresolved_imports(
    request: &LexicalEnvironmentRequest<'_>,
    imports: &[ResolvedImportEntry],
    diagnostics: &mut Vec<LexicalEnvironmentDiagnostic>,
) {
    let resolved_stub_ordinals = imports
        .iter()
        .map(|entry| entry.stub_ordinal)
        .collect::<BTreeSet<_>>();

    for (stub_ordinal, stub) in request.import_stubs.iter().enumerate() {
        if resolved_stub_ordinals.contains(&stub_ordinal) {
            continue;
        }
        push_diagnostic_if_absent(
            diagnostics,
            LexicalEnvironmentDiagnostic {
                code: LexicalEnvironmentDiagnosticCode::UnresolvedImport,
                message: Arc::<str>::from(format!(
                    "import `{}` could not be resolved; excluding it from the active lexical environment",
                    stub.path.spelling
                )),
                primary: stub.span,
                secondary: Vec::new(),
                import_ordinal: Some(stub_ordinal),
                module_id: None,
            },
        );
    }
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

fn filter_imports_with_summaries(
    imports: Vec<ResolvedImportEntry>,
    summaries: &[ModuleLexicalSummary],
    diagnostics: &mut Vec<LexicalEnvironmentDiagnostic>,
) -> Vec<ResolvedImportEntry> {
    let modules_with_summaries = summaries
        .iter()
        .map(|summary| summary.module_id.clone())
        .collect::<BTreeSet<_>>();
    let mut active_entries = Vec::new();

    for entry in imports {
        if modules_with_summaries.contains(&entry.import.module_id) {
            active_entries.push(entry);
            continue;
        }

        push_diagnostic_if_absent(
            diagnostics,
            LexicalEnvironmentDiagnostic {
                code: LexicalEnvironmentDiagnosticCode::MissingSummary,
                message: Arc::<str>::from(format!(
                    "lexical summary for module `{}` is unavailable; excluding the import from the active lexical environment",
                    entry.import.module_id.as_str()
                )),
                primary: entry.stub_span,
                secondary: Vec::new(),
                import_ordinal: Some(entry.stub_ordinal),
                module_id: Some(entry.import.module_id),
            },
        );
    }

    active_entries
}

fn build_lexical_environment_with_retries(
    mut active_entries: Vec<ResolvedImportEntry>,
    summaries: &[ModuleLexicalSummary],
    diagnostics: &mut Vec<LexicalEnvironmentDiagnostic>,
) -> Result<ActiveLexicalEnvironment, FrontendLexicalEnvironmentError> {
    let max_retries = active_entries.len();
    let mut retries = 0;

    loop {
        let active_imports = active_entries
            .iter()
            .map(|entry| entry.import.clone())
            .collect::<Vec<_>>();
        match mizar_lexer::build_lexical_environment(&active_imports, summaries) {
            Ok(environment) => return Ok(environment),
            Err(LexicalEnvironmentError::UserSymbolImportConflict {
                spelling,
                earlier_import,
                later_import,
            }) => {
                if retries >= max_retries {
                    return Err(malformed_user_symbol_import_conflict(
                        spelling,
                        earlier_import,
                        later_import,
                    ));
                }
                retries += 1;

                let Some(later_index) = active_entries
                    .iter()
                    .position(|entry| entry.import.module_id == later_import)
                else {
                    return Err(malformed_user_symbol_import_conflict(
                        spelling,
                        earlier_import,
                        later_import,
                    ));
                };
                let Some(earlier_entry) = active_entries
                    .iter()
                    .find(|entry| entry.import.module_id == earlier_import)
                    .cloned()
                else {
                    return Err(malformed_user_symbol_import_conflict(
                        spelling,
                        earlier_import,
                        later_import,
                    ));
                };
                let later_entry = active_entries[later_index].clone();

                push_diagnostic_if_absent(
                    diagnostics,
                    user_symbol_import_conflict_diagnostic(
                        &spelling,
                        &earlier_import,
                        &later_import,
                        &earlier_entry,
                        &later_entry,
                    ),
                );
                active_entries.remove(later_index);
            }
            Err(source) => {
                return Err(FrontendLexicalEnvironmentError::MalformedSummary { source });
            }
        }
    }
}

fn user_symbol_import_conflict_diagnostic(
    spelling: &str,
    earlier_import: &ModuleId,
    later_import: &ModuleId,
    earlier_entry: &ResolvedImportEntry,
    later_entry: &ResolvedImportEntry,
) -> LexicalEnvironmentDiagnostic {
    LexicalEnvironmentDiagnostic {
        code: LexicalEnvironmentDiagnosticCode::UserSymbolImportConflict,
        message: Arc::<str>::from(format!(
            "user symbol `{spelling}` imported from module `{}` conflicts with an earlier import from module `{}`; excluding the later import from the active lexical environment",
            later_import.as_str(),
            earlier_import.as_str()
        )),
        primary: later_entry.stub_span,
        secondary: vec![SourceAnchor::Range(earlier_entry.stub_span)],
        import_ordinal: Some(later_entry.stub_ordinal),
        module_id: Some(later_import.clone()),
    }
}

fn malformed_user_symbol_import_conflict(
    spelling: String,
    earlier_import: ModuleId,
    later_import: ModuleId,
) -> FrontendLexicalEnvironmentError {
    FrontendLexicalEnvironmentError::MalformedSummary {
        source: LexicalEnvironmentError::UserSymbolImportConflict {
            spelling,
            earlier_import,
            later_import,
        },
    }
}

fn malformed_provider_provenance(message: String) -> FrontendLexicalEnvironmentError {
    FrontendLexicalEnvironmentError::MalformedProviderProvenance { message }
}

fn push_diagnostic_if_absent(
    diagnostics: &mut Vec<LexicalEnvironmentDiagnostic>,
    diagnostic: LexicalEnvironmentDiagnostic,
) {
    if diagnostics
        .iter()
        .any(|existing| existing.code == diagnostic.code && existing.primary == diagnostic.primary)
    {
        return;
    }

    diagnostics.push(diagnostic);
}

impl fmt::Display for FrontendLexicalEnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderUnavailable { message } => {
                write!(f, "lexical-summary provider is unavailable: {message}")
            }
            Self::MalformedProviderProvenance { message } => {
                write!(
                    f,
                    "lexical-summary provider returned malformed import provenance: {message}"
                )
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
            Self::MalformedProviderProvenance { .. } => None,
            Self::MalformedSummary { source } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ActiveLexicalEnvironmentResult, ExportRank, ExportedSymbolShape,
        FrontendLexicalEnvironmentError, LexicalEnvironmentDiagnostic,
        LexicalEnvironmentDiagnosticCode, LexicalEnvironmentError, LexicalEnvironmentRequest,
        LexicalSummaryFingerprint, LexicalSummaryProvider, ModuleId, ModuleLexicalSummary,
        ResolvedImport, ResolvedImportEntry, ResolvedImports, SymbolId, UserSymbolArity,
        UserSymbolKind, build_active_lexical_environment,
    };
    use crate::preprocess::{ImportStub, ImportStubPath};
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator,
        SourceAnchor, SourceRange,
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

    #[test]
    fn provider_import_provenance_with_missing_stub_ordinal_is_a_hard_failure() {
        let source_id = source_id(4);
        let stubs = vec![import_stub(source_id, 0, 13, "alpha")];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![ResolvedImportEntry {
                    stub_ordinal: 1,
                    stub_span: stubs[0].span,
                    import: ResolvedImport {
                        module_id: ModuleId::new("alpha"),
                    },
                }],
                summaries: vec![summary(
                    "alpha",
                    11,
                    vec![symbol("++", "alpha.plus", "alpha")],
                )],
                diagnostics: Vec::new(),
            },
        };

        let error = build(&stubs, source_id, &provider).unwrap_err();

        assert!(matches!(
            error,
            FrontendLexicalEnvironmentError::MalformedProviderProvenance { ref message }
                if message.contains("missing import stub ordinal 1")
        ));
    }

    #[test]
    fn provider_import_provenance_with_stale_stub_span_is_a_hard_failure() {
        let current_source_id = source_id(4);
        let stubs = vec![import_stub(current_source_id, 0, 13, "alpha")];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![ResolvedImportEntry {
                    stub_ordinal: 0,
                    stub_span: SourceRange {
                        source_id: current_source_id,
                        start: 1,
                        end: 14,
                    },
                    import: ResolvedImport {
                        module_id: ModuleId::new("alpha"),
                    },
                }],
                summaries: vec![summary(
                    "alpha",
                    11,
                    vec![symbol("++", "alpha.plus", "alpha")],
                )],
                diagnostics: Vec::new(),
            },
        };

        let error = build(&stubs, current_source_id, &provider).unwrap_err();

        assert!(matches!(
            error,
            FrontendLexicalEnvironmentError::MalformedProviderProvenance { ref message }
                if message.contains("references stale span")
        ));
    }

    #[test]
    fn provider_import_provenance_with_foreign_source_is_a_hard_failure() {
        let (current_source_id, other_source_id) = distinct_source_ids(4);
        let stubs = vec![import_stub(current_source_id, 0, 13, "alpha")];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![ResolvedImportEntry {
                    stub_ordinal: 0,
                    stub_span: SourceRange {
                        source_id: other_source_id,
                        start: 0,
                        end: 13,
                    },
                    import: ResolvedImport {
                        module_id: ModuleId::new("alpha"),
                    },
                }],
                summaries: vec![summary(
                    "alpha",
                    11,
                    vec![symbol("++", "alpha.plus", "alpha")],
                )],
                diagnostics: Vec::new(),
            },
        };

        let error = build(&stubs, current_source_id, &provider).unwrap_err();

        assert!(matches!(
            error,
            FrontendLexicalEnvironmentError::MalformedProviderProvenance { ref message }
                if message.contains("references source")
        ));
    }

    #[test]
    fn provider_diagnostic_with_foreign_primary_source_is_a_hard_failure() {
        let (current_source_id, other_source_id) = distinct_source_ids(4);
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: Vec::new(),
                summaries: Vec::new(),
                diagnostics: vec![LexicalEnvironmentDiagnostic {
                    code: LexicalEnvironmentDiagnosticCode::UnresolvedImport,
                    message: Arc::from("foreign primary"),
                    primary: SourceRange {
                        source_id: other_source_id,
                        start: 0,
                        end: 0,
                    },
                    secondary: Vec::new(),
                    import_ordinal: None,
                    module_id: None,
                }],
            },
        };

        let error = build(&[], current_source_id, &provider).unwrap_err();

        assert!(matches!(
            error,
            FrontendLexicalEnvironmentError::MalformedProviderProvenance { ref message }
                if message.contains("provider diagnostic `unresolved_import` references source")
        ));
    }

    #[test]
    fn provider_diagnostic_with_stale_import_span_is_a_hard_failure() {
        let current_source_id = source_id(4);
        let stubs = vec![import_stub(current_source_id, 0, 13, "alpha")];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: Vec::new(),
                summaries: Vec::new(),
                diagnostics: vec![LexicalEnvironmentDiagnostic {
                    code: LexicalEnvironmentDiagnosticCode::UnresolvedImport,
                    message: Arc::from("stale primary"),
                    primary: SourceRange {
                        source_id: current_source_id,
                        start: 1,
                        end: 14,
                    },
                    secondary: Vec::new(),
                    import_ordinal: Some(0),
                    module_id: None,
                }],
            },
        };

        let error = build(&stubs, current_source_id, &provider).unwrap_err();

        assert!(matches!(
            error,
            FrontendLexicalEnvironmentError::MalformedProviderProvenance { ref message }
                if message.contains("provider diagnostic `unresolved_import` references stale span")
        ));
    }

    #[test]
    fn provider_diagnostic_with_missing_stub_ordinal_is_a_hard_failure() {
        let current_source_id = source_id(4);
        let stubs = vec![import_stub(current_source_id, 0, 13, "alpha")];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: Vec::new(),
                summaries: Vec::new(),
                diagnostics: vec![LexicalEnvironmentDiagnostic {
                    code: LexicalEnvironmentDiagnosticCode::UnresolvedImport,
                    message: Arc::from("missing diagnostic ordinal"),
                    primary: stubs[0].span,
                    secondary: Vec::new(),
                    import_ordinal: Some(1),
                    module_id: None,
                }],
            },
        };

        let error = build(&stubs, current_source_id, &provider).unwrap_err();

        assert!(matches!(
            error,
            FrontendLexicalEnvironmentError::MalformedProviderProvenance { ref message }
                if message.contains("provider diagnostic `unresolved_import` references missing import stub ordinal 1")
        ));
    }

    #[test]
    fn provider_diagnostic_with_foreign_secondary_source_is_a_hard_failure() {
        let (current_source_id, other_source_id) = distinct_source_ids(4);
        let stubs = vec![import_stub(current_source_id, 0, 13, "alpha")];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: Vec::new(),
                summaries: Vec::new(),
                diagnostics: vec![LexicalEnvironmentDiagnostic {
                    code: LexicalEnvironmentDiagnosticCode::UserSymbolImportConflict,
                    message: Arc::from("foreign secondary"),
                    primary: stubs[0].span,
                    secondary: vec![SourceAnchor::Range(SourceRange {
                        source_id: other_source_id,
                        start: 0,
                        end: 13,
                    })],
                    import_ordinal: Some(0),
                    module_id: Some(ModuleId::new("alpha")),
                }],
            },
        };

        let error = build(&stubs, current_source_id, &provider).unwrap_err();

        assert!(matches!(
            error,
            FrontendLexicalEnvironmentError::MalformedProviderProvenance { ref message }
                if message.contains("secondary range references source")
        ));
    }

    #[test]
    fn unresolved_imports_are_diagnosed_and_excluded_while_remaining_symbols_load() {
        let source_id = source_id(5);
        let stubs = vec![
            import_stub(source_id, 0, 13, "alpha"),
            import_stub(source_id, 15, 29, "missing"),
            import_stub(source_id, 31, 43, "beta"),
        ];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![
                    resolved_entry(&stubs, 0, "alpha"),
                    resolved_entry(&stubs, 2, "beta"),
                ],
                summaries: vec![
                    summary("alpha", 11, vec![symbol("++", "alpha.plus", "alpha")]),
                    summary("beta", 22, vec![symbol("**", "beta.times", "beta")]),
                ],
                diagnostics: Vec::new(),
            },
        };

        let result = build(&stubs, source_id, &provider).unwrap();

        assert_eq!(
            result
                .environment
                .user_symbol("++")
                .unwrap()
                .imported_module,
            ModuleId::new("alpha")
        );
        let times = result.environment.user_symbol("**").unwrap();
        assert_eq!(times.imported_module, ModuleId::new("beta"));
        assert_eq!(
            times.import_ordinal, 1,
            "unresolved imports are not counted in compact active import ordinals"
        );
        assert_eq!(result.diagnostics.len(), 1);
        let diagnostic = &result.diagnostics[0];
        assert_eq!(
            diagnostic.code,
            LexicalEnvironmentDiagnosticCode::UnresolvedImport
        );
        assert_eq!(diagnostic.primary, stubs[1].span);
        assert_eq!(diagnostic.import_ordinal, Some(1));
        assert_eq!(diagnostic.module_id, None);
    }

    #[test]
    fn imports_without_matching_summaries_are_diagnosed_and_excluded_before_lexer_call() {
        let source_id = source_id(6);
        let stubs = vec![
            import_stub(source_id, 0, 13, "alpha"),
            import_stub(source_id, 15, 31, "no_summary"),
            import_stub(source_id, 33, 45, "beta"),
        ];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![
                    resolved_entry(&stubs, 0, "alpha"),
                    resolved_entry(&stubs, 1, "no_summary"),
                    resolved_entry(&stubs, 2, "beta"),
                ],
                summaries: vec![
                    summary("alpha", 11, vec![symbol("++", "alpha.plus", "alpha")]),
                    summary("beta", 22, vec![symbol("**", "beta.times", "beta")]),
                ],
                diagnostics: Vec::new(),
            },
        };

        let result = build(&stubs, source_id, &provider).unwrap();

        let times = result.environment.user_symbol("**").unwrap();
        assert_eq!(times.imported_module, ModuleId::new("beta"));
        assert_eq!(
            times.import_ordinal, 1,
            "missing-summary imports are omitted before the lexer sees active imports"
        );
        assert_eq!(result.diagnostics.len(), 1);
        let diagnostic = &result.diagnostics[0];
        assert_eq!(
            diagnostic.code,
            LexicalEnvironmentDiagnosticCode::MissingSummary
        );
        assert_eq!(diagnostic.primary, stubs[1].span);
        assert_eq!(diagnostic.import_ordinal, Some(1));
        assert_eq!(diagnostic.module_id, Some(ModuleId::new("no_summary")));
    }

    #[test]
    fn user_symbol_import_conflicts_are_diagnosed_and_retried_until_stable() {
        let source_id = source_id(7);
        let stubs = vec![
            import_stub(source_id, 0, 13, "alpha"),
            import_stub(source_id, 15, 28, "beta"),
            import_stub(source_id, 30, 51, "beta.duplicate"),
            import_stub(source_id, 53, 66, "gamma"),
            import_stub(source_id, 68, 86, "unresolved"),
        ];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![
                    resolved_entry(&stubs, 3, "gamma"),
                    resolved_entry(&stubs, 2, "beta"),
                    resolved_entry(&stubs, 0, "alpha"),
                    resolved_entry(&stubs, 1, "beta"),
                ],
                summaries: vec![
                    summary("alpha", 11, vec![symbol("++", "alpha.plus", "alpha")]),
                    summary(
                        "beta",
                        22,
                        vec![
                            symbol("++", "beta.plus", "beta"),
                            symbol("--", "beta.minus", "beta"),
                        ],
                    ),
                    summary(
                        "gamma",
                        33,
                        vec![
                            symbol("++", "gamma.plus", "gamma"),
                            symbol("**", "gamma.times", "gamma"),
                        ],
                    ),
                ],
                diagnostics: vec![LexicalEnvironmentDiagnostic {
                    code: LexicalEnvironmentDiagnosticCode::UnresolvedImport,
                    message: Arc::from("provider could not resolve import"),
                    primary: stubs[4].span,
                    secondary: Vec::new(),
                    import_ordinal: Some(4),
                    module_id: None,
                }],
            },
        };

        let result = build(&stubs, source_id, &provider).unwrap();

        let plus = result.environment.user_symbol("++").unwrap();
        assert_eq!(plus.imported_module, ModuleId::new("alpha"));
        assert_eq!(plus.import_ordinal, 0);
        assert!(result.environment.user_symbol("--").is_none());
        assert!(result.environment.user_symbol("**").is_none());
        assert_eq!(result.diagnostics.len(), 3);
        assert_eq!(
            result.diagnostics[0].code,
            LexicalEnvironmentDiagnosticCode::UnresolvedImport
        );
        assert_eq!(result.diagnostics[0].primary, stubs[4].span);
        assert_eq!(
            result.diagnostics[1].code,
            LexicalEnvironmentDiagnosticCode::UserSymbolImportConflict
        );
        assert_eq!(
            result.diagnostics[1].primary, stubs[1].span,
            "conflict diagnostics use the first canonical stub even when provider order is shuffled"
        );
        assert_eq!(
            result.diagnostics[1].secondary,
            vec![SourceAnchor::Range(stubs[0].span)]
        );
        assert_eq!(result.diagnostics[1].import_ordinal, Some(1));
        assert_eq!(result.diagnostics[1].module_id, Some(ModuleId::new("beta")));
        assert_eq!(
            result.diagnostics[2].code,
            LexicalEnvironmentDiagnosticCode::UserSymbolImportConflict
        );
        assert_eq!(result.diagnostics[2].primary, stubs[3].span);
        assert_eq!(
            result.diagnostics[2].secondary,
            vec![SourceAnchor::Range(stubs[0].span)]
        );
        assert_eq!(result.diagnostics[2].import_ordinal, Some(3));
        assert_eq!(
            result.diagnostics[2].module_id,
            Some(ModuleId::new("gamma"))
        );
    }

    #[test]
    fn non_conflict_lexer_errors_become_malformed_summary() {
        let source_id = source_id(8);
        let stubs = vec![import_stub(source_id, 0, 13, "alpha")];
        let provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![resolved_entry(&stubs, 0, "alpha")],
                summaries: vec![summary(
                    "alpha",
                    11,
                    vec![symbol("definition", "alpha.definition", "alpha")],
                )],
                diagnostics: Vec::new(),
            },
        };

        let error = build(&stubs, source_id, &provider).unwrap_err();

        assert!(matches!(
            error,
            FrontendLexicalEnvironmentError::MalformedSummary {
                source: LexicalEnvironmentError::ReservedWordCollision { .. }
            }
        ));
    }

    #[test]
    fn fingerprint_changes_for_dependency_summary_edits_and_ignores_local_comment_only_spans() {
        let base_source_id = source_id(9);
        let edited_source_id = source_id(10);
        let stubs = vec![import_stub(base_source_id, 0, 13, "alpha")];
        let edited_stubs = vec![import_stub(edited_source_id, 50, 63, "alpha")];
        let base_provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![resolved_entry(&stubs, 0, "alpha")],
                summaries: vec![summary(
                    "alpha",
                    11,
                    vec![symbol("++", "alpha.plus", "alpha")],
                )],
                diagnostics: Vec::new(),
            },
        };
        let edited_local_provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![resolved_entry(&edited_stubs, 0, "alpha")],
                summaries: vec![summary(
                    "alpha",
                    11,
                    vec![symbol("++", "alpha.plus", "alpha")],
                )],
                diagnostics: Vec::new(),
            },
        };
        let changed_dependency_provider = FakeProvider {
            resolved: ResolvedImports {
                imports: vec![resolved_entry(&stubs, 0, "alpha")],
                summaries: vec![summary(
                    "alpha",
                    12,
                    vec![symbol("++", "alpha.plus", "alpha")],
                )],
                diagnostics: Vec::new(),
            },
        };

        let base = build(&stubs, base_source_id, &base_provider).unwrap();
        let edited_local = build(&edited_stubs, edited_source_id, &edited_local_provider).unwrap();
        let changed_dependency =
            build(&stubs, base_source_id, &changed_dependency_provider).unwrap();

        assert_eq!(base.fingerprint, edited_local.fingerprint);
        assert_ne!(base.fingerprint, changed_dependency.fingerprint);
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

    fn distinct_source_ids(byte: u8) -> (mizar_session::SourceId, mizar_session::SourceId) {
        let ids = InMemorySessionIdAllocator::new();
        let snapshot = snapshot_id(byte);
        (
            ids.next_source_id(snapshot).unwrap(),
            ids.next_source_id(snapshot).unwrap(),
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
