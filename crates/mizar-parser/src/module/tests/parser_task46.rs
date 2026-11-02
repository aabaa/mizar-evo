use super::*;

#[test]
fn parser_task46_parses_operator_declarations_in_all_authorized_positions() {
    let source_id = source_id(246);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("@custom", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("task46", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("infix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"+\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("right", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("95", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("public", ParserTokenKind::ReservedWord),
                ("prefix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"-\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("85", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
                ("private", ParserTokenKind::ReservedWord),
                ("postfix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"!\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("90", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("AfterOperators", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "canonical operator declarations should parse cleanly: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("Task46 declarations should retain an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::OperatorDeclaration
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0
    );
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 4);
    let annotated = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(
        annotated.kind,
        SurfaceNodeKind::OperatorDeclaration
    ));
    assert_eq!(
        direct_child_labels(&ast, annotated),
        vec![
            "Annotation",
            "infix_operator",
            "(",
            "\"+\"",
            ",",
            "right",
            ",",
            "95",
            ")",
            ";",
        ]
    );
    let visible = ast.node(item_list.children[1]).unwrap();
    assert!(matches!(visible.kind, SurfaceNodeKind::VisibleItem));
    assert_eq!(
        direct_child_labels(&ast, visible),
        vec!["VisibilityMarker", "OperatorDeclaration"]
    );
    let definition = ast.node(item_list.children[2]).unwrap();
    assert!(matches!(
        definition.kind,
        SurfaceNodeKind::DefinitionBlockItem
    ));
    assert!(definition.children.iter().any(|child| {
        ast.node(*child)
            .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::VisibleItem))
    }));
    assert!(matches!(
        ast.node(item_list.children[3]).unwrap().kind,
        SurfaceNodeKind::TheoremItem
    ));
}

#[test]
fn parser_task46_recovers_operator_slots_without_losing_following_item() {
    let source_id = source_id(247);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("infix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("missing_string", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("left", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("80", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("infix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"+\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("middle", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("80", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("prefix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"-\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("85", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("AfterRecovery", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect::<Vec<_>>(),
        vec![
            SyntaxDiagnosticCode::MissingStringLiteral,
            SyntaxDiagnosticCode::MalformedTermExpression,
            SyntaxDiagnosticCode::MissingSemicolon,
        ]
    );
    let ast = output
        .ast
        .expect("operator recovery should preserve a surface AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 4);
    assert!(matches!(
        ast.node(item_list.children[3]).unwrap().kind,
        SurfaceNodeKind::TheoremItem
    ));
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStringLiteral)
        )),
        1
    );
}

#[test]
fn parser_task46_pins_each_operator_slot_and_delimiter_diagnostic() {
    struct Case {
        name: &'static str,
        declaration: Vec<(&'static str, ParserTokenKind)>,
        expected_codes: &'static [SyntaxDiagnosticCode],
        first_message: &'static str,
        first_primary_text: &'static str,
    }

    let cases = vec![
        Case {
            name: "missing_open_parenthesis",
            declaration: vec![
                ("prefix_operator", ParserTokenKind::ReservedWord),
                ("\"-\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("85", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
            expected_codes: &[SyntaxDiagnosticCode::MalformedTermExpression],
            first_message: "expected `(` after operator declaration keyword",
            first_primary_text: "\"-\"",
        },
        Case {
            name: "missing_spelling_comma",
            declaration: vec![
                ("prefix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"-\"", ParserTokenKind::StringLiteral),
                ("85", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
            expected_codes: &[SyntaxDiagnosticCode::MalformedTermExpression],
            first_message: "expected `,` after operator spelling",
            first_primary_text: "85",
        },
        Case {
            name: "missing_associativity_comma",
            declaration: vec![
                ("infix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"+\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("left", ParserTokenKind::ReservedWord),
                ("80", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
            expected_codes: &[SyntaxDiagnosticCode::MalformedTermExpression],
            first_message: "expected `,` after infix associativity",
            first_primary_text: "80",
        },
        Case {
            name: "invalid_precedence",
            declaration: vec![
                ("postfix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"!\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("high", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
            expected_codes: &[SyntaxDiagnosticCode::MalformedTermExpression],
            first_message: "expected natural-number precedence in operator declaration",
            first_primary_text: "high",
        },
        Case {
            name: "missing_close_parenthesis",
            declaration: vec![
                ("prefix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"-\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("85", ParserTokenKind::Numeral),
                (";", ParserTokenKind::ReservedSymbol),
            ],
            expected_codes: &[SyntaxDiagnosticCode::MalformedTermExpression],
            first_message: "expected `)` to close operator declaration",
            first_primary_text: ";",
        },
        Case {
            name: "prefix_rejects_infix_associativity_slot",
            declaration: vec![
                ("prefix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"-\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("left", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("85", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
            expected_codes: &[
                SyntaxDiagnosticCode::MalformedTermExpression,
                SyntaxDiagnosticCode::MalformedTermExpression,
                SyntaxDiagnosticCode::MalformedTermExpression,
            ],
            first_message: "expected natural-number precedence in operator declaration",
            first_primary_text: "left",
        },
    ];

    for (index, case) in cases.into_iter().enumerate() {
        let source_id = source_id(230 + u8::try_from(index).unwrap());
        let mut source = case.declaration;
        source.extend([
            ("theorem", ParserTokenKind::ReservedWord),
            ("AfterRecovery", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ]);
        let tokens = token_sequence(source_id, &source);
        let first_primary = nth_token_range(&tokens, case.first_primary_text, 0);
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
            case.expected_codes,
            "{} diagnostic-code contract",
            case.name
        );
        assert_eq!(
            output.diagnostics[0].message.as_ref(),
            case.first_message,
            "{} first diagnostic message",
            case.name
        );
        assert_eq!(
            output.diagnostics[0].primary, first_primary,
            "{} first diagnostic range",
            case.name
        );
        let ast = output.ast.expect("slot recovery should preserve an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(
            item_list.children.len(),
            2,
            "{} must preserve the following theorem as a separate item",
            case.name
        );
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::TheoremItem
        ));
    }
}

#[test]
fn parser_task46_definition_local_recovery_preserves_outer_end_and_following_item() {
    let source_id = source_id(236);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("infix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"*\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("none", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("90", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("prefix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"~\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("85", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("AfterDefinitionRecovery", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.clone())
            .collect::<Vec<_>>(),
        vec![SyntaxDiagnosticCode::MissingSemicolon]
    );
    let ast = output
        .ast
        .expect("definition-local recovery should preserve an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    let definition = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(
        definition.kind,
        SurfaceNodeKind::DefinitionBlockItem
    ));
    assert_eq!(
        direct_child_labels(&ast, definition),
        vec![
            "definition",
            "OperatorDeclaration",
            "OperatorDeclaration",
            "end",
            ";",
        ],
        "the malformed local declaration must not consume the outer terminator"
    );
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::TheoremItem
    ));
}
