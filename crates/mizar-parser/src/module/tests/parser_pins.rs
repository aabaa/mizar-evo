use super::*;

#[test]
fn parser_pins_task33_recovery_diagnostic_payloads() {
    let source_id = source_id(203);
    let tokens = token_sequence(
        source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("diagnostics_flow", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("do", ParserTokenKind::ReservedWord),
            ("while", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("invariant", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("by", ParserTokenKind::ReservedWord),
            ("Inv", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("if", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("match", ParserTokenKind::ReservedWord),
            ("t", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
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
        tokens.clone(),
        Vec::new(),
    ));

    let expected = [
        (
            SyntaxDiagnosticCode::MalformedFormulaExpression,
            "expected `if` condition",
            nth_token_range(&tokens, "do", 2),
            "repair the formula expression before continuing",
        ),
        (
            SyntaxDiagnosticCode::MalformedTermExpression,
            "expected match case",
            nth_token_range(&tokens, "end", 2),
            "repair the term expression before continuing",
        ),
    ];

    assert_eq!(
        output.diagnostics.len(),
        expected.len(),
        "{:#?}",
        output.diagnostics
    );
    for (diagnostic, (code, message, primary, recovery_note)) in
        output.diagnostics.iter().zip(expected)
    {
        assert_eq!(diagnostic.code, code);
        assert_eq!(diagnostic.message.as_ref(), message);
        assert_eq!(diagnostic.primary, primary);
        assert_eq!(diagnostic.recovery_note.as_deref(), Some(recovery_note));
    }
}

#[test]
fn parser_pins_task33_control_flow_diagnostic_matrix() {
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(212),
        &[
            ("for", ParserTokenKind::ReservedWord),
            ("i", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("to", ParserTokenKind::ReservedWord),
            ("upper", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected range lower bound",
            "to",
            0,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(213),
        &[
            ("for", ParserTokenKind::ReservedWord),
            ("i", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("lower", ParserTokenKind::Identifier),
            ("upper", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected `to` or `downto` in range loop",
            "upper",
            0,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(214),
        &[
            ("for", ParserTokenKind::ReservedWord),
            ("i", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("lower", ParserTokenKind::Identifier),
            ("to", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected range upper bound",
            "do",
            1,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(215),
        &[
            ("for", ParserTokenKind::ReservedWord),
            ("item", ParserTokenKind::Identifier),
            ("in", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected collection term",
            "do",
            1,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(216),
        &[
            ("for", ParserTokenKind::ReservedWord),
            ("item", ParserTokenKind::Identifier),
            ("in", ParserTokenKind::ReservedWord),
            ("Items", ParserTokenKind::Identifier),
            ("processed", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected processed binding name",
            "do",
            1,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(217),
        &[
            ("for", ParserTokenKind::ReservedWord),
            ("i", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("lower", ParserTokenKind::Identifier),
            ("to", ParserTokenKind::ReservedWord),
            ("upper", ParserTokenKind::Identifier),
            ("step", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected step term",
            "do",
            1,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(221),
        &[
            ("for", ParserTokenKind::ReservedWord),
            ("=", ParserTokenKind::ReservedSymbol),
            ("lower", ParserTokenKind::Identifier),
            ("to", ParserTokenKind::ReservedWord),
            ("upper", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected loop variable",
            "=",
            0,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(222),
        &[
            ("for", ParserTokenKind::ReservedWord),
            ("item", ParserTokenKind::Identifier),
            ("over", ParserTokenKind::Identifier),
            ("Items", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[
            ExpectedParserDiagnostic::term("expected `=` or `in` after loop variable", "over", 0),
            ExpectedParserDiagnostic::formula("expected `do` in loop statement", "break", 0),
        ],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(223),
        &[
            ("while", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "expected `while` condition",
            "do",
            1,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(224),
        &[
            ("if", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "expected `do` after `if` condition",
            "end",
            0,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(225),
        &[
            ("while", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "expected `do` after `while` condition",
            "end",
            0,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(218),
        &[
            ("match", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("case", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected match scrutinee",
            "do",
            1,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(219),
        &[
            ("match", ParserTokenKind::ReservedWord),
            ("t", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("case", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected match case pattern",
            "do",
            2,
        )],
    );
    let source_id = source_id(220);
    let tokens = task33_algorithm_body_tokens(
        source_id,
        &[
            ("match", ParserTokenKind::ReservedWord),
            ("t", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("case", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
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
    assert!(
        output.diagnostics.is_empty(),
        "a match may omit its otherwise ending: {:?}",
        output.diagnostics
    );
    assert!(
        output.ast.is_some(),
        "a match without otherwise should be accepted"
    );
}

#[test]
fn parser_pins_task34_verification_diagnostic_matrix() {
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(232),
        &[
            ("assert", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "expected assertion formula",
            ";",
            0,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(233),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("late_while_invariant", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("do", ParserTokenKind::ReservedWord),
            ("while", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("invariant", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "expected algorithm statement",
            "invariant",
            0,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(234),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("late_for_invariant", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("do", ParserTokenKind::ReservedWord),
            ("for", ParserTokenKind::ReservedWord),
            ("item", ParserTokenKind::Identifier),
            ("in", ParserTokenKind::ReservedWord),
            ("Items", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("continue", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("invariant", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "expected algorithm statement",
            "invariant",
            0,
        )],
    );
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(235),
        &[
            ("for", ParserTokenKind::ReservedWord),
            ("item", ParserTokenKind::Identifier),
            ("in", ParserTokenKind::ReservedWord),
            ("Items", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("decreasing", ParserTokenKind::ReservedWord),
            ("measure", ParserTokenKind::Identifier),
            ("by", ParserTokenKind::ReservedWord),
            ("Dec", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("continue", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "range and collection loops accept only invariant verification clauses",
            "decreasing",
            0,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(236),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("duplicate_header", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("requires", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("requires", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "duplicate or out-of-order algorithm verification clause",
            "requires",
            1,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(237),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("out_of_order_header", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("ensures", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("requires", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "duplicate or out-of-order algorithm verification clause",
            "requires",
            0,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(238),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("missing_requires_formula", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("requires", ParserTokenKind::ReservedWord),
            ("ensures", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "expected formula after `requires`",
            "ensures",
            0,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(239),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("missing_ensures_formula", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("requires", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("ensures", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "expected formula after `ensures`",
            "do",
            0,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(240),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("missing_measure", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("decreasing", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected decreasing measure",
            "do",
            0,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(241),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            (
                "missing_loop_invariant_formula",
                ParserTokenKind::Identifier,
            ),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("do", ParserTokenKind::ReservedWord),
            ("while", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("invariant", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::formula(
            "expected invariant formula",
            ";",
            0,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(242),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("dangling_loop_measure", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("do", ParserTokenKind::ReservedWord),
            ("while", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            ("do", ParserTokenKind::ReservedWord),
            ("decreasing", ParserTokenKind::ReservedWord),
            ("measure", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("break", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected term after `,`",
            ";",
            0,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(243),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("dangling_measure", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("decreasing", ParserTokenKind::ReservedWord),
            ("measure", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("do", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected term after `,`",
            "do",
            0,
        )],
    );
    assert_parser_diagnostics_exact(
        source_id(244),
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("missing_comma", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("decreasing", ParserTokenKind::ReservedWord),
            ("measure", ParserTokenKind::Identifier),
            ("next", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        &[ExpectedParserDiagnostic::term(
            "expected `,` between decreasing measures",
            "next",
            0,
        )],
    );
}
