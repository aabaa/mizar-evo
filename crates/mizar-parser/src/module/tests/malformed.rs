use super::*;

#[test]
fn malformed_top_level_annotation_prefix_still_finds_the_host_item() {
    let source_id = source_id(70);
    let tokens = token_sequence(
        source_id,
        &[
            ("@[", ParserTokenKind::ReservedSymbol),
            ("label", ParserTokenKind::Identifier),
            ("theorem", ParserTokenKind::ReservedWord),
            ("Recovered", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("lemma", ParserTokenKind::ReservedWord),
            ("Next", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert!(!output.diagnostics.is_empty());
    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedAnnotation),
        "malformed annotation should not fall through to unexpected top-level recovery: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("malformed annotation prefix should preserve the following theorem");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::TheoremItem
    ));
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::LemmaItem
    ));
}

#[test]
fn malformed_annotation_argument_delimiter_still_finds_nested_hosts() {
    let source_id = source_id(71);
    let tokens = token_sequence(
        source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("StatementHost", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            ("proof", ParserTokenKind::ReservedWord),
            ("@custom", ParserTokenKind::AnnotationMarker),
            ("(", ParserTokenKind::ReservedSymbol),
            ("note", ParserTokenKind::Identifier),
            ("thus", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("@custom", ParserTokenKind::AnnotationMarker),
            ("(", ParserTokenKind::ReservedSymbol),
            ("\"note\"", ParserTokenKind::StringLiteral),
            ("thesis", ParserTokenKind::ReservedWord),
            ("by", ParserTokenKind::ReservedWord),
            ("A", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("definition", ParserTokenKind::ReservedWord),
            ("@custom", ParserTokenKind::AnnotationMarker),
            ("(", ParserTokenKind::ReservedSymbol),
            ("note", ParserTokenKind::Identifier),
            ("theorem", ParserTokenKind::ReservedWord),
            ("DefHost", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("guarded", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("do", ParserTokenKind::ReservedWord),
            ("@trace", ParserTokenKind::AnnotationMarker),
            ("(", ParserTokenKind::ReservedSymbol),
            ("\"entry\"", ParserTokenKind::StringLiteral),
            ("assert", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("registration", ParserTokenKind::ReservedWord),
            ("@custom", ParserTokenKind::AnnotationMarker),
            ("(", ParserTokenKind::ReservedSymbol),
            ("reg", ParserTokenKind::Identifier),
            ("let", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("set", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 5, "{:?}", output.diagnostics);
    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedAnnotation),
        "malformed annotation delimiter recovery diagnostics should stay annotation-local: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("malformed nested annotation prefixes should preserve their hosts");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedStatement
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedDefinitionContent
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedAlgorithmStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedRegistrationContent
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ConclusionStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CompactStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssertStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::RegistrationParameter
        )),
        1
    );
}

#[test]
fn malformed_library_annotation_still_finds_identifier_start_hosts() {
    let source_id = source_id(72);
    let tokens = token_sequence(
        source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("StatementHost", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            ("proof", ParserTokenKind::ReservedWord),
            ("@[", ParserTokenKind::ReservedSymbol),
            ("bad", ParserTokenKind::Identifier),
            ("Step", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            ("by", ParserTokenKind::ReservedWord),
            ("A", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("guarded", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("do", ParserTokenKind::ReservedWord),
            ("@[", ParserTokenKind::ReservedSymbol),
            ("bad", ParserTokenKind::Identifier),
            ("x", ParserTokenKind::Identifier),
            (":=", ParserTokenKind::ReservedSymbol),
            ("y", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 2);
    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedAnnotation),
        "malformed library annotation recovery diagnostics should stay annotation-local: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("malformed library annotation should preserve identifier-start hosts");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedAlgorithmStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CompactStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssignmentStatement
        )),
        1
    );
}

#[test]
fn malformed_proof_hint_options_remain_annotation_local() {
    let source_id = source_id(245);
    let tokens = token_sequence(
        source_id,
        &[
            ("@proof_hint", ParserTokenKind::AnnotationMarker),
            ("(", ParserTokenKind::ReservedSymbol),
            ("foo", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("1", ParserTokenKind::Numeral),
            (",", ParserTokenKind::ReservedSymbol),
            ("max_axioms", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("vampire", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("solver", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("10", ParserTokenKind::Numeral),
            (")", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("BadOptions", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("lemma", ParserTokenKind::ReservedWord),
            ("Next", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 3);
    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedAnnotation),
        "malformed proof-hint options should not fall through to top-level recovery: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("malformed proof-hint options should preserve following items");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::LemmaItem)),
        1
    );
}

#[test]
fn malformed_annotation_stops_at_host_before_later_stray_closer() {
    let source_id = source_id(73);
    let tokens = token_sequence(
        source_id,
        &[
            ("@custom", ParserTokenKind::AnnotationMarker),
            ("(", ParserTokenKind::ReservedSymbol),
            ("note", ParserTokenKind::Identifier),
            ("theorem", ParserTokenKind::ReservedWord),
            ("OrdinaryHost", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("lemma", ParserTokenKind::ReservedWord),
            ("AfterOrdinary", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("@[", ParserTokenKind::ReservedSymbol),
            ("bad", ParserTokenKind::Identifier),
            ("theorem", ParserTokenKind::ReservedWord),
            ("LibraryHost", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("]", ParserTokenKind::ReservedSymbol),
            ("lemma", ParserTokenKind::ReservedWord),
            ("AfterLibrary", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect::<Vec<_>>(),
        vec![
            SyntaxDiagnosticCode::MalformedAnnotation,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken,
            SyntaxDiagnosticCode::MalformedAnnotation,
            SyntaxDiagnosticCode::UnexpectedTopLevelToken,
        ]
    );
    let ast = output
        .ast
        .expect("malformed annotation prefixes should preserve hosts before stray closers");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::LemmaItem)),
        2
    );
}
