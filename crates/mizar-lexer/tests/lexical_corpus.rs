use std::fs;
use std::path::{Path, PathBuf};

use mizar_lexer::{
    ImportPrescanDiagnosticCode, RawTokenKind, TokenKind, lex, scan_import_prelude, scan_raw,
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
                        let span_start = token.span_start.unwrap_or_else(|| {
                            panic!(
                                "{} raw token expectations require span_start",
                                case.expectation_path.display()
                            )
                        });
                        let span_end = token.span_end.unwrap_or_else(|| {
                            panic!(
                                "{} raw token expectations require span_end",
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
                        let span_start = token.span_start.unwrap_or_else(|| {
                            panic!(
                                "{} import pre-scan expectations require span_start",
                                case.expectation_path.display()
                            )
                        });
                        let span_end = token.span_end.unwrap_or_else(|| {
                            panic!(
                                "{} import pre-scan expectations require span_end",
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
            other => panic!(
                "unsupported lexical corpus domain `{other}` in {}",
                case.expectation_path.display()
            ),
        }
    }

    assert_eq!(final_checked, 11);
    assert_eq!(raw_checked, 5);
    assert_eq!(import_prescan_checked, 10);
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

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-lexer crate should live under crates/")
        .to_path_buf()
}
