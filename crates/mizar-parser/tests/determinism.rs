use mizar_parser::{
    OperatorAssociativity, OperatorFixityEntry, ParseRequest, ParserToken, ParserTokenKind, parse,
};
use mizar_session::{
    BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
    SourceRange,
};
use mizar_syntax::{SurfaceAst, SyntaxDiagnostic, SyntaxDiagnosticCode};

#[test]
fn identical_module_recovery_streams_preserve_ast_and_diagnostic_order() {
    let source_id = source_id(1);
    let tokens = token_sequence(
        source_id,
        &[
            ("and", ParserTokenKind::ReservedWord),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("definition", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );

    let baseline = assert_repeated_parse_is_deterministic(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));
    let ast = baseline
        .ast
        .as_ref()
        .expect("recovery fixture should produce an AST");
    assert!(
        ast.snapshot.contains("ErrorRecovery kind=SkippedToken"),
        "recovery fixture should preserve skipped-token recovery shape: {ast:#?}"
    );
    assert!(
        baseline
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::UnexpectedTopLevelToken),
        "fixture should exercise top-level recovery diagnostics: {:#?}",
        baseline.diagnostics
    );
}

#[test]
fn identical_operator_streams_preserve_expression_tree_order() {
    let source_id = source_id(2);
    let tokens = token_sequence(
        source_id,
        &[
            ("a", ParserTokenKind::Identifier),
            ("++", ParserTokenKind::UserSymbol),
            ("b", ParserTokenKind::Identifier),
            ("**", ParserTokenKind::UserSymbol),
            ("c", ParserTokenKind::Identifier),
            ("++", ParserTokenKind::UserSymbol),
            ("d", ParserTokenKind::Identifier),
        ],
    );

    let baseline = assert_repeated_parse_is_deterministic(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        vec![
            OperatorFixityEntry::new("++", 10, OperatorAssociativity::Left),
            OperatorFixityEntry::new("**", 20, OperatorAssociativity::Right),
        ],
    ));
    let ast = baseline
        .ast
        .as_ref()
        .expect("operator fixture should produce an AST");
    assert!(
        ast.expression_root.is_some(),
        "operator fixture should exercise a Pratt expression root: {ast:#?}"
    );
    assert!(
        ast.snapshot.contains("InfixExpression spelling=\"++\"")
            && ast.snapshot.contains("InfixExpression spelling=\"**\""),
        "operator fixture should preserve infix expression nodes: {ast:#?}"
    );
}

#[test]
fn identical_malformed_streams_preserve_multiple_diagnostics_order() {
    let source_id = source_id(3);
    let tokens = token_sequence(
        source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("diagnostic_flow", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("do", ParserTokenKind::ReservedWord),
            ("if", ParserTokenKind::ReservedWord),
            ("then", ParserTokenKind::ReservedWord),
            ("while", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );

    let baseline = assert_repeated_parse_is_deterministic(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));
    assert!(
        baseline.diagnostics.len() >= 2,
        "fixture should exercise deterministic ordering for multiple diagnostics: {:#?}",
        baseline.diagnostics
    );
    assert!(
        baseline.ast.is_some(),
        "malformed fixture should recover with an AST"
    );
}

fn assert_repeated_parse_is_deterministic(request: ParseRequest) -> ParseSignature {
    let baseline = parse_signature(parse(request.clone()));
    for _ in 0..8 {
        assert_eq!(parse_signature(parse(request.clone())), baseline);
    }
    baseline
}

fn parse_signature(output: mizar_parser::ParseOutput) -> ParseSignature {
    ParseSignature {
        ast: output.ast.as_ref().map(ast_signature),
        diagnostics: output.diagnostics,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParseSignature {
    ast: Option<AstSignature>,
    diagnostics: Vec<SyntaxDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AstSignature {
    snapshot: String,
    token_nodes: Vec<(usize, String, SourceRange)>,
    expression_root: Option<usize>,
}

fn ast_signature(ast: &SurfaceAst) -> AstSignature {
    AstSignature {
        snapshot: ast.snapshot_text(),
        token_nodes: ast
            .token_nodes()
            .iter()
            .map(|id| {
                let node = ast.node(*id).expect("token node id should be valid");
                (
                    id.index(),
                    node.token_text()
                        .expect("token node should expose token text")
                        .to_owned(),
                    node.range,
                )
            })
            .collect(),
        expression_root: ast.expression_root().map(|id| id.index()),
    }
}

fn token_sequence(source_id: SourceId, entries: &[(&str, ParserTokenKind)]) -> Vec<ParserToken> {
    let mut cursor = 0;
    entries
        .iter()
        .map(|(text, kind)| {
            let start = cursor;
            let end = start + text.len();
            cursor = end + 1;
            token(source_id, *kind, text, start, end)
        })
        .collect()
}

fn token(
    source_id: SourceId,
    kind: ParserTokenKind,
    text: &str,
    start: usize,
    end: usize,
) -> ParserToken {
    ParserToken::new(
        kind,
        text,
        SourceRange {
            source_id,
            start,
            end,
        },
    )
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
