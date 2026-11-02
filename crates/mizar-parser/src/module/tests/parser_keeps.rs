use super::*;

#[test]
fn parser_keeps_selector_arguments_before_structure_fields() {
    let source_id = source_id(126);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("S", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            ("f", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("y", ParserTokenKind::Identifier),
            (")", ParserTokenKind::ReservedSymbol),
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
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("selector type arg before fields should parse");
    let constructor = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::StructureConstructor)
    });
    assert!(
        constructor
            .children
            .iter()
            .any(|id| { matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::FieldArgument) })
    );
    single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::SelectorAccess));
}

#[test]
fn parser_keeps_zero_field_constructor_syntax_as_application() {
    let source_id = source_id(112);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("S", ParserTokenKind::UserSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert!(output.diagnostics.is_empty());
    let ast = output
        .ast
        .expect("zero-field constructor syntax should parse");
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::ApplicationTerm))
            .count(),
        1
    );
    assert!(
        !ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::StructureConstructor))
    );
}

#[test]
fn parser_keeps_template_arguments_between_prefix_operator_and_operand() {
    let source_id = source_id(189);
    let tokens = token_sequence(
        source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("PrefixTemplate", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("local_tid", ParserTokenKind::UserSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("set", ParserTokenKind::ReservedWord),
            ("]", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("by", ParserTokenKind::ReservedWord),
            ("Ref", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        vec![OperatorFixityEntry::prefix("local_tid", 64)],
    ));

    assert!(
        output.diagnostics.is_empty(),
        "prefix template application should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("prefix template application should keep an AST");
    let prefix = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::PrefixExpression(_))
    });
    assert_eq!(prefix.children.len(), 3);
    assert!(matches!(
        &ast.node(prefix.children[0]).unwrap().kind,
        SurfaceNodeKind::Token(_)
    ));
    assert!(matches!(
        &ast.node(prefix.children[1]).unwrap().kind,
        SurfaceNodeKind::TemplateArguments
    ));
    assert!(matches!(
        &ast.node(prefix.children[2]).unwrap().kind,
        SurfaceNodeKind::TermReference
    ));
}

#[test]
fn parser_keeps_bracket_functor_as_prefix_operand_without_trailing_term() {
    let source_id = source_id(192);
    let tokens = token_sequence(
        source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("PrefixBracket", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("local_prefix", ParserTokenKind::UserSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("=", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("by", ParserTokenKind::ReservedWord),
            ("Ref", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        vec![OperatorFixityEntry::prefix("local_prefix", 64)],
    ));

    assert!(
        output.diagnostics.is_empty(),
        "bracket functor operand should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("bracket functor operand should keep an AST");
    let prefix = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::PrefixExpression(_))
    });
    assert!(direct_child_has_kind(&ast, prefix, |kind| {
        matches!(kind, SurfaceNodeKind::ApplicationTerm)
    }));
}

#[test]
fn parser_keeps_non_generator_comma_in_comprehension_type_arguments() {
    let source_id = source_id(171);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("NonGeneratorComma", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("Element", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("NAT", ParserTokenKind::UserSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("p", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("NonGeneratorOverComma", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("Element", ParserTokenKind::UserSymbol),
                ("over", ParserTokenKind::ReservedWord),
                ("NAT", ParserTokenKind::UserSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("p", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "non-generator comma should remain a type-argument comma: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("non-generator comma should keep a full AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::SetComprehension
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComprehensionVariableSegment
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(_)
        )),
        0
    );
    let type_arguments = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::TypeArguments))
        .collect::<Vec<_>>();
    assert_eq!(type_arguments.len(), 2);
    assert!(type_arguments.iter().all(|type_arguments| {
        type_arguments
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
            .count()
            == 2
    }));

    let malformed = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("MalformedContextual", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("Element", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("NAT", ParserTokenKind::UserSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("p", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));
    assert_eq!(malformed.diagnostics.len(), 1);
    assert_eq!(
        malformed.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(
        malformed.diagnostics[0].message.as_ref(),
        "expected type after set-comprehension generator `is`"
    );
    assert_eq!(malformed.diagnostics[0].primary, range(source_id, 67, 68));
    assert_eq!(
        malformed.diagnostics[0].recovery_note.as_deref(),
        Some("repair the type expression before continuing")
    );
    let malformed_ast = malformed
        .ast
        .expect("malformed contextual generator should keep an AST");
    assert_eq!(
        count_nodes(&malformed_ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )),
        1
    );
    assert_eq!(
        count_nodes(&malformed_ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(_)
        )),
        1
    );
    assert_eq!(
        count_nodes(&malformed_ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComprehensionVariableSegment
        )),
        2
    );
    let malformed_comprehension =
        set_comprehension_term_for_label(&malformed_ast, "MalformedContextual");
    assert_eq!(
        direct_token_texts(&malformed_ast, malformed_comprehension),
        vec!["{", "where", ",", "}"]
    );
}

#[test]
fn parser_keeps_template_required_type_fallback_single_token_and_ambiguity_bounded() {
    let ambiguity_source_id = source_id(250);
    let tokens = token_sequence(
        ambiguity_source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("type", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("func", ParserTokenKind::ReservedWord),
            ("Ambiguous", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("f", ParserTokenKind::UserSymbol),
            ("->", ParserTokenKind::ReservedSymbol),
            ("set", ParserTokenKind::ReservedWord),
            ("equals", ParserTokenKind::ReservedWord),
            ("{", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("where", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("is", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            ("U", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            ("}", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let first_range = nth_token_range(&tokens, "T", 1);
    let second_range = nth_token_range(&tokens, "U", 0);
    let output = parse(ParseRequest::new(
        ambiguity_source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));
    assert!(!output.diagnostics.is_empty());
    let ast = output
        .ast
        .expect("multi-token near miss should retain a recovered AST");
    assert!(ast.nodes().iter().any(|node| {
        node.range == first_range && matches!(node.kind, SurfaceNodeKind::TypeHead)
    }));
    assert!(!ast.nodes().iter().any(|node| {
        node.range == range(ambiguity_source_id, first_range.start, second_range.end)
            && matches!(
                node.kind,
                SurfaceNodeKind::TypeHead | SurfaceNodeKind::TypeExpression
            )
    }));
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::ErrorRecovery(_)))
    );

    let assertion_source_id = source_id(249);
    let assertion_tokens = token_sequence(
        assertion_source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("type", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("Assertion", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("is", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let assertion_type_range = nth_token_range(&assertion_tokens, "T", 1);
    let assertion_output = parse(ParseRequest::new(
        assertion_source_id,
        Edition::new("2026"),
        assertion_tokens,
        Vec::new(),
    ));
    assert!(assertion_output.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression
            && diagnostic.primary == assertion_type_range
    }));
    let assertion_ast = assertion_output
        .ast
        .expect("is-assertion near miss should retain a recovered AST");
    assert!(!assertion_ast.nodes().iter().any(|node| {
        node.range == assertion_type_range && matches!(node.kind, SurfaceNodeKind::TypeHead)
    }));

    let bracket_source_id = source_id(248);
    let bracket_tokens = token_sequence(
        bracket_source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("type", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("func", ParserTokenKind::ReservedWord),
            ("Bracket", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("f", ParserTokenKind::UserSymbol),
            ("->", ParserTokenKind::ReservedSymbol),
            ("Mode", ParserTokenKind::UserSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("T", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("equals", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let bracket_type_range = nth_token_range(&bracket_tokens, "T", 1);
    let bracket_output = parse(ParseRequest::new(
        bracket_source_id,
        Edition::new("2026"),
        bracket_tokens,
        Vec::new(),
    ));
    assert!(bracket_output.diagnostics.is_empty());
    let bracket_ast = bracket_output
        .ast
        .expect("bracket argument control should retain an AST");
    assert!(!bracket_ast.nodes().iter().any(|node| {
        node.range == bracket_type_range && matches!(node.kind, SurfaceNodeKind::TypeHead)
    }));

    let argument_source_id = source_id(247);
    let argument_tokens = token_sequence(
        argument_source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("type", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("func", ParserTokenKind::ReservedWord),
            ("TemplateArg", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("f", ParserTokenKind::UserSymbol),
            ("->", ParserTokenKind::ReservedSymbol),
            ("set", ParserTokenKind::ReservedWord),
            ("equals", ParserTokenKind::ReservedWord),
            ("g", ParserTokenKind::Identifier),
            ("[", ParserTokenKind::ReservedSymbol),
            ("T", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (")", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let argument_type_range = nth_token_range(&argument_tokens, "T", 1);
    let argument_output = parse(ParseRequest::new(
        argument_source_id,
        Edition::new("2026"),
        argument_tokens,
        Vec::new(),
    ));
    assert!(argument_output.diagnostics.is_empty());
    let argument_ast = argument_output
        .ast
        .expect("template argument control should retain an AST");
    assert!(!argument_ast.nodes().iter().any(|node| {
        node.range == argument_type_range && matches!(node.kind, SurfaceNodeKind::TypeHead)
    }));
}

#[test]
fn parser_keeps_task33_branch_boundaries_during_recovery() {
    let source_id = source_id(202);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("boundary_flow", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("do", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                ("else", ParserTokenKind::ReservedWord),
                ("continue", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("match", ParserTokenKind::ReservedWord),
                ("t", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("case", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                ("case", ParserTokenKind::ReservedWord),
                ("q", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("continue", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("otherwise", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon),
        "missing branch-local semicolons should diagnose: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("branch-boundary recovery should preserve an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::IfStatement)),
        2,
        "recovery inside nested if then-bodies must not swallow match branches"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::MatchCase)),
        2,
        "recovery before the second `case` must not swallow the next branch"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::MatchEnding)),
        1,
        "recovery before `otherwise` must return to the match ending"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::BreakStatement)),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ContinueStatement
        )),
        2
    );
}

#[test]
fn parser_keeps_set_type_and_set_statement_contexts_separate() {
    let source_id = source_id(78);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("assume", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("f", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "`set` type and `set` statement should not conflict: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("set contexts should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssumptionStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::IsAssertion)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SetStatement)),
        1
    );
}

#[test]
fn parser_keeps_task17_from_tails_deferred() {
    let source_id = source_id(91);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("thesis", ParserTokenKind::ReservedWord),
                ("from", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "deferred `from` tails should stay on the legacy placeholder path: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("deferred from tails should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::JustificationClause
                | SurfaceNodeKind::ReferenceList
                | SurfaceNodeKind::ComputationJustification
        )),
        0
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CompactStatement
        )),
        0
    );
}
