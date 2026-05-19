use std::fs;
use std::path::{Path, PathBuf};

use mizar_lexer::{TokenKind, lex};
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

    let mut checked = 0;
    for case in plan.cases {
        let expectation = &case.expectation;
        if expectation.expected_outcome != ExpectedOutcome::Pass
            || expectation.expected_phase != Some(PipelinePhase::Lex)
        {
            continue;
        }

        assert!(
            !expectation.tokens.is_empty(),
            "{} should include token expectations",
            case.expectation_path.display()
        );

        let source = fs::read_to_string(&case.source_path).unwrap_or_else(|error| {
            panic!("failed to read {}: {error}", case.source_path.display())
        });
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
        checked += 1;
    }

    assert_eq!(checked, 6);
}

fn token_kind_name(kind: TokenKind) -> &'static str {
    match kind {
        TokenKind::Identifier => "identifier",
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-lexer crate should live under crates/")
        .to_path_buf()
}
