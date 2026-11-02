use super::*;

#[test]
fn parser_preserves_following_item_when_redefinition_coherence_is_missing() {
    let source_id = source_id(189);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("redefine", ParserTokenKind::ReservedWord),
                ("attr", ParserTokenKind::ReservedWord),
                ("AttrR", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("redefine", ParserTokenKind::ReservedWord),
                ("pred", ParserTokenKind::ReservedWord),
                ("PredR", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::UserSymbol),
                ("y", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }),
        "missing coherence should be diagnosed without swallowing the next item: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("missing coherence before the next item should still recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AttributeRedefinition
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PredicateRedefinition
        )),
        1,
        "the following redefinition item must remain parseable"
    );
}

#[test]
fn parser_preserves_property_items_after_malformed_property_justifications() {
    let source_id = source_id(193);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("symmetry", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("irreflexivity", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("asymmetry", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("connectedness", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("Broken", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Carrier", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("sethood", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("projectivity", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
    );
    let ast = output
        .ast
        .expect("malformed property justifications should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::PropertyClause)),
        5,
        "malformed property justifications must preserve following property items"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeProperty)),
        1,
        "the malformed immediate sethood property remains attached to the mode definition"
    );
    for expected in [
        &["irreflexivity", "by", "Ref", ";"][..],
        &["connectedness", "by", "Ref", ";"][..],
        &["projectivity", "by", "Ref", ";"][..],
    ] {
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(node.kind, SurfaceNodeKind::PropertyClause)
                    && subtree_token_texts(&ast, node)
                        .iter()
                        .map(String::as_str)
                        .eq(expected.iter().copied())
            }),
            "following property item {expected:?} should remain parseable"
        );
    }
}

#[test]
fn parser_preserves_following_items_when_registration_end_is_missing() {
    let source_id = source_id(198);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("registration", ParserTokenKind::ReservedWord),
                ("cluster", ParserTokenKind::ReservedWord),
                ("E", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd),
        "missing registration end should be diagnosed: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("missing registration end should preserve an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(
        item_list.children.len(),
        3,
        "the following theorem and definition should remain top-level items"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::RegistrationBlockItem
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::DefinitionBlockItem
        )),
        1
    );
}

#[test]
fn parser_preserves_attribute_bearing_bracket_qua_as_term_candidate() {
    let source_id = source_id(138);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("V", ParserTokenKind::Identifier),
            ("qua", ParserTokenKind::ReservedWord),
            ("non", ParserTokenKind::ReservedWord),
            ("empty", ParserTokenKind::UserSymbol),
            ("R", ParserTokenKind::UserSymbol),
            ("]", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.expect("full bracket term candidate");
    let arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    let term = ast.node(arguments.children[1]).unwrap();
    assert_eq!(term.kind, SurfaceNodeKind::TermExpression);
    let qua = ast.node(term.children[0]).unwrap();
    assert_eq!(qua.kind, SurfaceNodeKind::QuaExpression);
    let target = ast.node(qua.children[2]).unwrap();
    assert_eq!(target.kind, SurfaceNodeKind::TypeExpression);
    assert_eq!(target.children.len(), 2);
    assert_eq!(
        ast.node(target.children[0]).unwrap().kind,
        SurfaceNodeKind::AttributeChain
    );
    assert_eq!(
        ast.node(target.children[1]).unwrap().kind,
        SurfaceNodeKind::TypeHead
    );
}

#[test]
fn parser_preserves_parameter_prefix_and_bracket_qua_arguments() {
    let source_id = source_id(99);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("n", ParserTokenKind::Identifier),
            ("-", ParserTokenKind::ReservedSymbol),
            ("empty", ParserTokenKind::UserSymbol),
            ("T", ParserTokenKind::UserSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("set", ParserTokenKind::ReservedWord),
            (",", ParserTokenKind::ReservedSymbol),
            ("Ns", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("U", ParserTokenKind::UserSymbol),
            (",", ParserTokenKind::ReservedSymbol),
            ("V", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("W", ParserTokenKind::Identifier),
            ("qua", ParserTokenKind::ReservedWord),
            ("R", ParserTokenKind::UserSymbol),
            ("]", ParserTokenKind::ReservedSymbol),
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
        .expect("parameter prefix and bracket args should parse");
    let prefix = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::ParameterPrefix)
    });
    assert_eq!(
        prefix
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter_map(mizar_syntax::SurfaceNode::token_text)
            .collect::<Vec<_>>(),
        vec!["n", "-"]
    );
    let type_arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    assert_eq!(
        ast.node(type_arguments.children[0]).unwrap().token_text(),
        Some("[")
    );
    assert_eq!(
        ast.node(*type_arguments.children.last().unwrap())
            .unwrap()
            .token_text(),
        Some("]")
    );
    assert_eq!(
        type_arguments
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter(|node| matches!(node.kind, SurfaceNodeKind::TypeExpression))
            .count(),
        2
    );
    assert!(
        !type_arguments
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .any(|node| matches!(node.kind, SurfaceNodeKind::TermPlaceholder)),
        "bracket qua_arg should no longer use a temporary term placeholder"
    );
    let qua_term = type_arguments
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| {
            matches!(node.kind, SurfaceNodeKind::TermExpression)
                && direct_child_has_kind(&ast, node, |kind| {
                    matches!(kind, SurfaceNodeKind::QuaExpression)
                })
        })
        .expect("bracket qua_arg should be a term expression after task 11");
    assert_eq!(
        type_arguments
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
            .count(),
        2
    );
    assert!(type_arguments.children.iter().any(|id| {
        matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TermExpression)
            && direct_child_has_kind(&ast, ast.node(*id).unwrap(), |kind| {
                matches!(kind, SurfaceNodeKind::TermReference)
            })
    }));
    assert!(
        qua_term
            .children
            .iter()
            .any(|id| { matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::QuaExpression) })
    );
}

#[test]
fn parser_preserves_struct_qualified_attribute_spelling() {
    let source_id = source_id(102);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("TypeCaseStruct", ParserTokenKind::UserSymbol),
            (".", ParserTokenKind::ReservedSymbol),
            ("TypeCaseAttr", ParserTokenKind::UserSymbol),
            ("TypeCaseMode", ParserTokenKind::UserSymbol),
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
    let ast = output.ast.expect("struct-qualified attribute should parse");
    let attribute = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::AttributeRef));
    let qualified = attribute
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| matches!(node.kind, SurfaceNodeKind::QualifiedSymbol))
        .expect("attribute should own a qualified symbol");
    assert_eq!(
        qualified_symbol_token_texts(&ast, qualified),
        vec!["TypeCaseStruct", ".", "TypeCaseAttr"]
    );
    let type_head = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeHead));
    assert_eq!(
        qualified_symbol_token_texts(
            &ast,
            ast.node(type_head.children[0])
                .expect("type head should own its qualified symbol")
        ),
        vec!["TypeCaseMode"]
    );
}

#[test]
fn parser_preserves_parameterized_attribute_declaration_and_use_tree_shape() {
    let source_id = source_id(148);
    let tokens = token_sequence(
        source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("set", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("attr", ParserTokenKind::ReservedWord),
            ("RankedDef", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("is", ParserTokenKind::ReservedWord),
            ("2", ParserTokenKind::Numeral),
            ("-", ParserTokenKind::ReservedSymbol),
            ("ranked", ParserTokenKind::UserSymbol),
            ("means", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("reserve", ParserTokenKind::ReservedWord),
            ("y", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("ranked", ParserTokenKind::UserSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            ("2", ParserTokenKind::Numeral),
            (")", ParserTokenKind::ReservedSymbol),
            ("set", ParserTokenKind::ReservedWord),
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
        .expect("parameterized attribute declaration and use should parse");
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::ParameterPrefix))
            .count(),
        1
    );
    let attribute_refs = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::AttributeRef))
        .collect::<Vec<_>>();
    assert_eq!(attribute_refs.len(), 1);
    assert!(
        attribute_refs[0]
            .children
            .iter()
            .any(|id| { matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TermExpression) })
    );
}

#[test]
fn parser_preserves_template_local_identifier_type_in_quantified_binder() {
    let template_source_id = source_id(190);
    let tokens = token_sequence(
        template_source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("type", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("let", ParserTokenKind::ReservedWord),
            ("P", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("pred", ParserTokenKind::ReservedWord),
            ("(", ParserTokenKind::ReservedSymbol),
            ("T", ParserTokenKind::Identifier),
            (")", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("pred", ParserTokenKind::ReservedWord),
            ("THoldsDef", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("tholds", ParserTokenKind::Identifier),
            ("[", ParserTokenKind::ReservedSymbol),
            ("T", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("P", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("means", ParserTokenKind::ReservedWord),
            ("for", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("being", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            ("holds", ParserTokenKind::ReservedWord),
            ("P", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (")", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let binder_type_range = nth_token_range(&tokens, "T", 3);
    let output = parse(ParseRequest::new(
        template_source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "template-local binder type should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("template-local binder should keep an AST");
    assert!(ast.nodes().iter().any(|node| {
        node.range == binder_type_range && matches!(node.kind, SurfaceNodeKind::TypeHead)
    }));

    let outside_source_id = source_id(191);
    let outside_tokens = token_sequence(
        outside_source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("for", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("being", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            ("holds", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let outside_type_range = nth_token_range(&outside_tokens, "T", 0);
    let outside_output = parse(ParseRequest::new(
        outside_source_id,
        Edition::new("2026"),
        outside_tokens,
        Vec::new(),
    ));
    assert!(outside_output.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression
            && diagnostic.primary == outside_type_range
    }));
}

#[test]
fn parser_preserves_task31_nest_computation_option_trace() {
    let source_id = source_id(184);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("nest", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("3", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "nest computation option should stay parser-visible: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("nest option should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationOption
        )),
        1
    );
}
