use std::fs;
use std::path::{Path, PathBuf};

use mizar_lexer::{
    BindingShapeKind, ExportRank, ExportedSymbolShape, ImportPrescanDiagnosticCode,
    LexDiagnosticCode, LexicalBlockKind, LexicalStatementKind, LexicalSummaryFingerprint, ModuleId,
    ModuleLexicalSummary, ParserLexContext, RawTokenKind, ResolvedImport, ScopeLexView,
    ScopeSkeletonDiagnosticCode, SymbolId, TokenKind, build_lexical_environment,
    build_scope_skeleton, disambiguate, lex, scan_import_prelude, scan_raw,
};
use mizar_test::{
    DiscoveryConfig, ExpectedOutcome, PipelinePhase, TestProfile, ValidationMode, build_test_plan,
};

#[test]
fn lexical_pass_corpus_matches_token_expectations() {
    let workspace_root = workspace_root();
    let plan = build_test_plan(&DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    })
    .expect("repository corpus plan should build");

    assert_eq!(plan.error_count(), 0, "{:#?}", plan.diagnostics);

    let mut final_checked = 0;
    let mut raw_checked = 0;
    let mut import_prescan_checked = 0;
    let mut scope_skeleton_checked = 0;
    let mut disambiguator_checked = 0;
    for case in plan.cases {
        let expectation = &case.expectation;
        if expectation.expected_outcome != ExpectedOutcome::Pass
            || expectation.expected_phase != Some(PipelinePhase::Lex)
        {
            continue;
        }

        let source = fs::read_to_string(&case.source_path).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", case.source_path.display())
        });
        match expectation.domain.as_str() {
            "lexical" => {
                assert!(
                    !expectation.tokens.is_empty(),
                    "{} should include token expectations",
                    case.expectation_path.display()
                );
                let actual = lex(&source).unwrap_or_else(|error| {
                    panic!("lex failed for {}: {error}", case.source_path.display())
                });
                let actual = actual
                    .iter()
                    .map(|token| (token_kind_name(token.kind), token.lexeme.as_str()))
                    .collect::<Vec<_>>();
                let expected = expectation
                    .tokens
                    .iter()
                    .map(|token| (token.kind.as_str(), token.lexeme.as_str()))
                    .collect::<Vec<_>>();

                assert_eq!(actual, expected, "{}", case.expectation_path.display());
                final_checked += 1;
            }
            "raw_lexer" => {
                let actual = scan_raw(&source).unwrap_or_else(|error| {
                    panic!(
                        "scan_raw failed for {}: {error}",
                        case.source_path.display()
                    )
                });
                let actual = actual
                    .tokens
                    .iter()
                    .map(|token| {
                        (
                            raw_token_kind_name(token.kind),
                            token.lexeme.as_str(),
                            token.span.start as u32,
                            token.span.end as u32,
                        )
                    })
                    .collect::<Vec<_>>();
                let expected = expectation
                    .tokens
                    .iter()
                    .map(|token| {
                        let (span_start, span_end) = expected_span(token, &source, || {
                            panic!(
                                "{} raw token expectations require span_start",
                                case.expectation_path.display()
                            )
                        });
                        (
                            token.kind.as_str(),
                            token.lexeme.as_str(),
                            span_start,
                            span_end,
                        )
                    })
                    .collect::<Vec<_>>();

                assert_eq!(actual, expected, "{}", case.expectation_path.display());
                raw_checked += 1;
            }
            "import_prescan" => {
                let raw = scan_raw(&source).unwrap_or_else(|error| {
                    panic!(
                        "scan_raw failed for {}: {error}",
                        case.source_path.display()
                    )
                });
                let prelude = scan_import_prelude(&raw);
                let mut actual = Vec::new();
                for import in &prelude.imports {
                    actual.push((
                        "import_path",
                        import.path.spelling.as_str(),
                        import.path.span.start as u32,
                        import.path.span.end as u32,
                    ));
                    if import.path.source_segments.len() > 1 {
                        for segment in &import.path.source_segments {
                            actual.push((
                                "import_path_segment",
                                import.path.spelling.as_str(),
                                segment.start as u32,
                                segment.end as u32,
                            ));
                        }
                    }
                    if let Some(alias) = &import.alias {
                        actual.push((
                            "import_alias",
                            alias.spelling.as_str(),
                            alias.span.start as u32,
                            alias.span.end as u32,
                        ));
                    }
                }
                actual.push((
                    "prelude_end",
                    "prelude_end",
                    prelude.end as u32,
                    prelude.end as u32,
                ));
                let expected = expectation
                    .tokens
                    .iter()
                    .map(|token| {
                        let (span_start, span_end) = expected_span(token, &source, || {
                            panic!(
                                "{} import pre-scan expectations require span_start",
                                case.expectation_path.display()
                            )
                        });
                        (
                            token.kind.as_str(),
                            token.lexeme.as_str(),
                            span_start,
                            span_end,
                        )
                    })
                    .collect::<Vec<_>>();
                let actual_diagnostics = prelude
                    .diagnostics
                    .iter()
                    .map(|diagnostic| import_prescan_diagnostic_code_name(diagnostic.code))
                    .collect::<Vec<_>>();
                let expected_diagnostics = expectation
                    .diagnostic_codes
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();

                assert_eq!(actual, expected, "{}", case.expectation_path.display());
                assert_eq!(
                    actual_diagnostics,
                    expected_diagnostics,
                    "{}",
                    case.expectation_path.display()
                );
                import_prescan_checked += 1;
            }
            "scope_skeleton" => {
                let raw = scan_raw(&source).unwrap_or_else(|error| {
                    panic!(
                        "scan_raw failed for {}: {error}",
                        case.source_path.display()
                    )
                });
                let skeleton = build_scope_skeleton(&raw);
                let expects_frames = expectation.tokens.iter().any(|token| {
                    token.kind == "scope_frame" || token.kind.starts_with("scope_binding_")
                });
                let expects_blocks = expectation
                    .tokens
                    .iter()
                    .any(|token| token.kind.starts_with("scope_block_"));
                let expects_statements = expectation
                    .tokens
                    .iter()
                    .any(|token| token.kind.starts_with("scope_statement_"));
                let mut actual = Vec::new();
                if expects_blocks {
                    for block in &skeleton.blocks {
                        actual.push((
                            block_kind_name(block.kind),
                            "scope_block",
                            block.range.start as u32,
                            block.range.end as u32,
                        ));
                    }
                }
                if expects_statements {
                    for statement in &skeleton.statements {
                        actual.push((
                            statement_kind_name(statement.kind),
                            "scope_statement",
                            statement.range.start as u32,
                            statement.range.end as u32,
                        ));
                    }
                }
                if expects_frames {
                    for frame in &skeleton.frames {
                        actual.push((
                            "scope_frame",
                            "scope_frame",
                            frame.range.start as u32,
                            frame.range.end as u32,
                        ));
                        for binding in &frame.bindings {
                            actual.push((
                                binding_shape_kind_name(binding.kind),
                                binding.spelling.as_str(),
                                binding.introduced_at.start as u32,
                                binding.introduced_at.end as u32,
                            ));
                        }
                    }
                }
                let expected_structure = expectation
                    .tokens
                    .iter()
                    .filter(|token| !is_scope_probe_kind(&token.kind))
                    .map(|token| {
                        let (span_start, span_end) = expected_span(token, &source, || {
                            panic!(
                                "{} scope skeleton expectations require span_start",
                                case.expectation_path.display()
                            )
                        });
                        (
                            token.kind.as_str(),
                            token.lexeme.as_str(),
                            span_start,
                            span_end,
                        )
                    })
                    .collect::<Vec<_>>();
                for token in expectation
                    .tokens
                    .iter()
                    .filter(|token| is_scope_probe_kind(&token.kind))
                {
                    let (position, _) = expected_span(token, &source, || {
                        panic!(
                            "{} scope probe expectations require span_start",
                            case.expectation_path.display()
                        )
                    });
                    let position = position as usize;
                    let actual_active = skeleton.binding_overrides_symbol(&token.lexeme, position);
                    let expected_active = match token.kind.as_str() {
                        "scope_active" => true,
                        "scope_inactive" => false,
                        _ => unreachable!("filtered to scope probe kinds"),
                    };
                    assert_eq!(
                        actual_active,
                        expected_active,
                        "{} probe `{}` at byte {}",
                        case.expectation_path.display(),
                        token.lexeme,
                        position
                    );
                }
                let actual_diagnostics = skeleton
                    .diagnostics
                    .iter()
                    .map(|diagnostic| scope_skeleton_diagnostic_code_name(diagnostic.code))
                    .collect::<Vec<_>>();
                let expected_diagnostics = expectation
                    .diagnostic_codes
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();

                assert_eq!(
                    actual,
                    expected_structure,
                    "{}",
                    case.expectation_path.display()
                );
                assert_eq!(
                    actual_diagnostics,
                    expected_diagnostics,
                    "{}",
                    case.expectation_path.display()
                );
                scope_skeleton_checked += 1;
            }
            "disambiguator" => {
                let raw = scan_raw(&source).unwrap_or_else(|error| {
                    panic!(
                        "scan_raw failed for {}: {error}",
                        case.source_path.display()
                    )
                });
                let skeleton = build_scope_skeleton(&raw);
                let env = disambiguator_fixture_environment();
                let context = disambiguator_fixture_context(expectation.id.0.as_str());
                let stream = disambiguate(&raw, &env, &context, &skeleton);
                let actual = stream
                    .tokens
                    .iter()
                    .map(|token| (token_kind_name(token.kind), token.lexeme.as_str()))
                    .collect::<Vec<_>>();
                let expected = expectation
                    .tokens
                    .iter()
                    .map(|token| (token.kind.as_str(), token.lexeme.as_str()))
                    .collect::<Vec<_>>();
                let actual_diagnostics = stream
                    .diagnostics
                    .iter()
                    .map(|diagnostic| lex_diagnostic_code_name(diagnostic.code))
                    .collect::<Vec<_>>();
                let expected_diagnostics = expectation
                    .diagnostic_codes
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();

                assert_eq!(actual, expected, "{}", case.expectation_path.display());
                assert_eq!(
                    actual_diagnostics,
                    expected_diagnostics,
                    "{}",
                    case.expectation_path.display()
                );
                disambiguator_checked += 1;
            }
            other => panic!(
                "unsupported lexical corpus domain `{other}` in {}",
                case.expectation_path.display()
            ),
        }
    }

    assert_eq!(final_checked, 11);
    assert_eq!(raw_checked, 6);
    assert_eq!(import_prescan_checked, 10);
    assert_eq!(scope_skeleton_checked, 7);
    assert_eq!(disambiguator_checked, 19);
}

fn token_kind_name(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Identifier => "identifier",
        TokenKind::ReservedWord => "reserved_word",
        TokenKind::ReservedSymbol => "reserved_symbol",
        TokenKind::Numeral => "numeral",
        TokenKind::LexemeRun => "lexeme_run",
        TokenKind::UserSymbol => "user_symbol",
        TokenKind::StringLiteral => "string_literal",
        TokenKind::ErrorRecovery => "error_recovery",
    }
}

fn raw_token_kind_name(kind: RawTokenKind) -> &'static str {
    match kind {
        RawTokenKind::LexemeRun => "raw_lexeme_run",
        RawTokenKind::NumeralLike => "raw_numeral_like",
        RawTokenKind::AnnotationMarker => "raw_annotation_marker",
        RawTokenKind::Layout => "raw_layout",
        RawTokenKind::Error => "raw_error",
    }
}

fn import_prescan_diagnostic_code_name(code: ImportPrescanDiagnosticCode) -> &'static str {
    match code {
        ImportPrescanDiagnosticCode::MissingModulePath => "missing_module_path",
        ImportPrescanDiagnosticCode::EmptyModulePathComponent => "empty_module_path_component",
        ImportPrescanDiagnosticCode::MissingAlias => "missing_alias",
        ImportPrescanDiagnosticCode::MissingSemicolon => "missing_semicolon",
        ImportPrescanDiagnosticCode::UnexpectedToken => "unexpected_token",
    }
}

fn binding_shape_kind_name(kind: BindingShapeKind) -> &'static str {
    match kind {
        BindingShapeKind::Let => "scope_binding_let",
        BindingShapeKind::For => "scope_binding_for",
        BindingShapeKind::Ex => "scope_binding_ex",
        BindingShapeKind::Reserve => "scope_binding_reserve",
        BindingShapeKind::Given => "scope_binding_given",
        BindingShapeKind::Consider => "scope_binding_consider",
        BindingShapeKind::Set => "scope_binding_set",
        BindingShapeKind::Reconsider => "scope_binding_reconsider",
        BindingShapeKind::Take => "scope_binding_take",
        BindingShapeKind::Deffunc => "scope_binding_deffunc",
        BindingShapeKind::Defpred => "scope_binding_defpred",
        BindingShapeKind::Var => "scope_binding_var",
        BindingShapeKind::Const => "scope_binding_const",
        BindingShapeKind::Processed => "scope_binding_processed",
    }
}

fn block_kind_name(kind: LexicalBlockKind) -> &'static str {
    match kind {
        LexicalBlockKind::Algorithm => "scope_block_algorithm",
        LexicalBlockKind::Definition => "scope_block_definition",
        LexicalBlockKind::Proof => "scope_block_proof",
        LexicalBlockKind::Now => "scope_block_now",
        LexicalBlockKind::Case => "scope_block_case",
        LexicalBlockKind::Suppose => "scope_block_suppose",
        LexicalBlockKind::Hereby => "scope_block_hereby",
        LexicalBlockKind::Do => "scope_block_do",
    }
}

fn statement_kind_name(kind: LexicalStatementKind) -> &'static str {
    match kind {
        LexicalStatementKind::Binder => "scope_statement_binder",
        LexicalStatementKind::Other => "scope_statement_other",
    }
}

fn scope_skeleton_diagnostic_code_name(code: ScopeSkeletonDiagnosticCode) -> &'static str {
    match code {
        ScopeSkeletonDiagnosticCode::MalformedBinderList => "malformed_binder_list",
        ScopeSkeletonDiagnosticCode::UnsupportedBinderShape => "unsupported_binder_shape",
        ScopeSkeletonDiagnosticCode::DuplicateBindingName => "duplicate_binding_name",
        ScopeSkeletonDiagnosticCode::UnmatchedEnd => "unmatched_end",
        ScopeSkeletonDiagnosticCode::MissingEnd => "missing_end",
    }
}

fn lex_diagnostic_code_name(code: LexDiagnosticCode) -> &'static str {
    match code {
        LexDiagnosticCode::NoValidTokenCandidate => "no_valid_token_candidate",
        LexDiagnosticCode::ParserContextRejectedCandidate => "parser_context_rejected_candidate",
        LexDiagnosticCode::AmbiguousUserSymbol => "ambiguous_user_symbol",
        LexDiagnosticCode::MalformedStringLiteral => "malformed_string_literal",
        LexDiagnosticCode::UnsupportedRawToken => "unsupported_raw_token",
    }
}

fn disambiguator_fixture_context(id: &str) -> ParserLexContext {
    if id.contains("string_literal_context") {
        ParserLexContext::string_required()
    } else if id.contains("namespace_path") {
        ParserLexContext::namespace_path()
    } else {
        ParserLexContext::general()
    }
}

fn disambiguator_fixture_environment() -> mizar_lexer::ActiveLexicalEnvironment {
    build_lexical_environment(
        &[ResolvedImport {
            module_id: ModuleId("fixture.symbols".to_owned()),
        }],
        &[ModuleLexicalSummary {
            module_id: ModuleId("fixture.symbols".to_owned()),
            fingerprint: LexicalSummaryFingerprint(9001),
            exported_symbols: vec![
                exported("+", "fixture#plus", 0),
                exported("+*", "fixture#plus_star", 1),
                exported("+*+", "fixture#plus_star_plus", 2),
                exported("succ", "fixture#succ", 3),
                exported(".", "fixture#dot", 4),
                exported("Seen", "fixture#Seen", 5),
                exported("!#~", "fixture#bang_hash_tilde", 6),
                exported("A_1`", "fixture#identifier_graphic", 7),
            ],
        }],
    )
    .expect("disambiguator fixture environment should build")
}

fn exported(spelling: &str, symbol: &str, rank: u32) -> ExportedSymbolShape {
    ExportedSymbolShape {
        spelling: spelling.to_owned(),
        symbol_id: SymbolId(symbol.to_owned()),
        source_module: ModuleId("fixture.symbols".to_owned()),
        export_rank: ExportRank(rank),
    }
}

fn is_scope_probe_kind(kind: &str) -> bool {
    matches!(kind, "scope_active" | "scope_inactive")
}

fn expected_span(
    token: &mizar_test::expectation::TokenExpectation,
    source: &str,
    missing: impl FnOnce(),
) -> (u32, u32) {
    if let (Some(start), Some(end)) = (token.span_start, token.span_end) {
        return (start, end);
    }
    if let (Some(start_line), Some(start_col), Some(end_line), Some(end_col)) = (
        token.span_start_line,
        token.span_start_col,
        token.span_end_line,
        token.span_end_col,
    ) {
        return (
            source_pos_from_line_col(source, start_line, start_col),
            source_pos_from_line_col(source, end_line, end_col),
        );
    }
    missing();
    unreachable!("missing span callback should not return")
}

fn source_pos_from_line_col(source: &str, line: u32, col: u32) -> u32 {
    let mut current_line = 1;
    let mut current_col = 1;
    for (index, ch) in source.char_indices() {
        if current_line == line && current_col == col {
            return index as u32;
        }
        if ch == '\n' {
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
    }
    if current_line == line && current_col == col {
        return source.len() as u32;
    }
    panic!("line/column span {line}:{col} is outside source");
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-lexer crate should live under crates/")
        .to_path_buf()
}
