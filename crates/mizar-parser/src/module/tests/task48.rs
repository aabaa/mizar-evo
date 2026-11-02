use super::*;

#[test]
fn task48_property_implementation_requires_a_dedicated_top_level_node() {
    let source_id = source_id(249);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("@[", ParserTokenKind::ReservedSymbol),
                ("task48", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("Domain", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("property", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("value", ParserTokenKind::Identifier),
                ("equals", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "canonical property implementation should parse cleanly: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("property implementation should retain an AST");
    let snapshot = ast.snapshot_text();
    assert!(
        snapshot.contains("PropertyImplementation"),
        "Task 48 requires a dedicated top-level node; snapshot={snapshot}"
    );
    assert!(
        snapshot.contains("Annotation"),
        "top-level annotations should stay owned by the property declaration"
    );
    let property = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::PropertyImplementation)
    });
    assert_eq!(
        direct_child_labels(&ast, property),
        vec![
            "Annotation",
            "definition",
            "DefinitionParameter",
            "property",
            "M",
            ".",
            "value",
            "equals",
            "TermDefiniens",
            ";",
            "CorrectnessCondition",
            "end",
            ";",
        ],
        "annotation, header, body, correctness, and outer terminator must remain direct property children"
    );
    let parameter = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::DefinitionParameter)
    });
    assert_eq!(
        direct_child_labels(&ast, parameter),
        vec!["let", "M", "be", "TypeHead", ";"]
    );
    let type_head = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeHead));
    assert_eq!(
        direct_child_labels(&ast, type_head),
        vec!["QualifiedSymbol"],
        "a property mode application must retain its qualified-symbol owner"
    );
}

#[test]
fn task48_parses_means_equals_cases_and_ordered_correctness_nodes() {
    let source_id = source_id(248);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("Domain", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("property", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("value", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("otherwise", ParserTokenKind::ReservedWord),
                ("contradiction", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("uniqueness", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Unique", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("N", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("Pkg", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("Domain", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("]", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("property", ParserTokenKind::ReservedWord),
                ("N", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("selected", ParserTokenKind::Identifier),
                ("equals", ParserTokenKind::ReservedWord),
                ("N", ParserTokenKind::Identifier),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("N", ParserTokenKind::Identifier),
                ("if", ParserTokenKind::ReservedWord),
                ("contradiction", ParserTokenKind::ReservedWord),
                ("otherwise", ParserTokenKind::ReservedWord),
                ("N", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "canonical Task-48 forms should parse cleanly: {:#?}",
        output.diagnostics
    );
    let ast = output.ast.expect("Task-48 forms should retain an AST");
    let compilation_unit = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::CompilationUnit)
    });
    let item_list = ast.node(compilation_unit.children[0]).unwrap();
    let means_item = ast.node(item_list.children[0]).unwrap();
    assert_eq!(
        direct_child_labels(&ast, means_item),
        vec![
            "definition",
            "DefinitionParameter",
            "property",
            "M",
            ".",
            "value",
            "means",
            "FormulaDefiniens",
            ";",
            "CorrectnessCondition",
            "CorrectnessCondition",
            "CorrectnessCondition",
            "end",
            ";",
        ]
    );
    let means_children = structural_children(&ast, means_item);
    assert_eq!(means_children.len(), 5);
    assert!(matches!(
        means_children[0].kind,
        SurfaceNodeKind::DefinitionParameter
    ));
    assert!(matches!(
        means_children[1].kind,
        SurfaceNodeKind::FormulaDefiniens
    ));
    assert!(
        means_children[2..]
            .iter()
            .all(|node| matches!(node.kind, SurfaceNodeKind::CorrectnessCondition))
    );
    let equals_item = ast.node(item_list.children[1]).unwrap();
    assert_eq!(
        direct_child_labels(&ast, equals_item),
        vec![
            "definition",
            "DefinitionParameter",
            "property",
            "N",
            ".",
            "selected",
            "equals",
            "TermDefiniens",
            ";",
            "CorrectnessCondition",
            "end",
            ";",
        ]
    );
    let equals_children = structural_children(&ast, equals_item);
    assert_eq!(equals_children.len(), 3);
    assert!(matches!(
        equals_children[0].kind,
        SurfaceNodeKind::DefinitionParameter
    ));
    assert!(matches!(
        equals_children[1].kind,
        SurfaceNodeKind::TermDefiniens
    ));
    assert!(matches!(
        equals_children[2].kind,
        SurfaceNodeKind::CorrectnessCondition
    ));
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PropertyImplementation
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::DefinitionBlockItem
        )),
        0,
        "property implementations are declarations, not ordinary definition blocks"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::DefinitionParameter
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FormulaDefiniens
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TermDefiniens)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FormulaCase)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TermCase)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CorrectnessCondition
        )),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        1
    );
    let condition_keywords = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::CorrectnessCondition))
        .filter_map(|condition| {
            condition
                .children
                .iter()
                .filter_map(|child| ast.node(*child))
                .find_map(mizar_syntax::SurfaceNode::token_text)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        condition_keywords,
        vec!["existence", "uniqueness", "coherence", "coherence"],
        "correctness conditions must retain source order"
    );
    let second_parameter = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::DefinitionParameter))
        .nth(1)
        .expect("equals form should own its parameter");
    let second_type_head = second_parameter
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .find(|node| matches!(node.kind, SurfaceNodeKind::TypeHead))
        .expect("parameter should own a TypeHead");
    assert_eq!(
        direct_child_labels(&ast, second_type_head),
        vec!["QualifiedSymbol", "TypeArguments"]
    );
}

#[test]
fn task48_recovers_exact_parameter_and_correctness_failures_without_losing_next_item() {
    let source_id = source_id(247);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("N", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("Domain", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("property", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                ("value", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("uniqueness", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("property", ParserTokenKind::ReservedWord),
                (".", ParserTokenKind::ReservedSymbol),
                ("value", ParserTokenKind::Identifier),
                ("equals", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("Y", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("Domain", ParserTokenKind::UserSymbol),
                ("property", ParserTokenKind::ReservedWord),
                ("Y", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("value", ParserTokenKind::Identifier),
                ("equals", ParserTokenKind::ReservedWord),
                ("Y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("After", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
    );
    let ast = output.ast.expect("Task-48 recovery should retain an AST");
    let compilation_unit = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::CompilationUnit)
    });
    let item_list = ast.node(compilation_unit.children[0]).unwrap();
    assert_eq!(item_list.children.len(), 4);
    for item in &item_list.children[..3] {
        assert!(matches!(
            ast.node(*item).unwrap().kind,
            SurfaceNodeKind::PropertyImplementation
        ));
    }
    assert!(matches!(
        ast.node(item_list.children[3]).unwrap().kind,
        SurfaceNodeKind::TheoremItem
    ));
}

#[test]
fn task48_discriminator_ignores_property_tokens_inside_nested_proofs() {
    let source_id = source_id(246);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("property", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    let ast = output.ast.expect("bounded negative should retain an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::DefinitionBlockItem
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PropertyImplementation
        )),
        0,
        "a nested proof token must not select the top-level property producer"
    );
}

#[test]
fn task48_discriminator_stops_at_an_immediate_item_boundary() {
    let source_id = source_id(244);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Inside", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("property", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    let ast = output.ast.expect("bounded negative should retain an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::DefinitionBlockItem
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PropertyImplementation
        )),
        0,
        "an item head immediately after malformed `let` must stop the discriminator"
    );
}

#[test]
fn task48_parameter_recovery_ignores_nested_property_before_real_header() {
    let source_id = source_id(245);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("N", ParserTokenKind::Identifier),
                ("proof", ParserTokenKind::ReservedWord),
                ("now", ParserTokenKind::ReservedWord),
                ("property", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                ("be", ParserTokenKind::ReservedWord),
                ("Domain", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("property", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("value", ParserTokenKind::Identifier),
                ("equals", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("After", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        !output.diagnostics.is_empty(),
        "malformed parameter material should be diagnosed"
    );
    let ast = output
        .ast
        .expect("nested malformed parameter recovery should retain an AST");
    let compilation_unit = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::CompilationUnit)
    });
    let item_list = ast.node(compilation_unit.children[0]).unwrap();
    assert_eq!(item_list.children.len(), 2);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::PropertyImplementation
    ));
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::TheoremItem
    ));
    let property = ast.node(item_list.children[0]).unwrap();
    let labels = direct_child_labels(&ast, property);
    assert!(
        labels
            .windows(5)
            .any(|window| window == ["property", "M", ".", "value", "equals"]),
        "the producer must own the real top-level property header: {labels:?}"
    );
    assert!(
        labels.ends_with(&["end", ";"]),
        "the producer must own the real outer terminator: {labels:?}"
    );
}

#[test]
fn task48_missing_outer_terminators_preserve_following_declarations() {
    let cases: &[(&str, &[(&str, ParserTokenKind)])] = &[
        (
            "missing outer end",
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("AfterMissingEnd", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "missing final semicolon",
            &[
                ("end", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("AfterMissingSemicolon", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
    ];

    for (ordinal, (name, tail)) in cases.iter().enumerate() {
        let source_id = source_id(238 + ordinal as u8);
        let mut entries = vec![
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("Domain", ParserTokenKind::UserSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("property", ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("value", ParserTokenKind::Identifier),
            ("equals", ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ];
        entries.extend_from_slice(tail);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(source_id, &entries),
            Vec::new(),
        ));
        let ast = output
            .ast
            .unwrap_or_else(|| panic!("`{name}` recovery should retain an AST"));
        assert!(
            output.diagnostics.iter().any(|diagnostic| matches!(
                diagnostic.code,
                SyntaxDiagnosticCode::MissingEnd | SyntaxDiagnosticCode::MissingSemicolon
            )),
            "`{name}` should report its missing terminator"
        );
        let compilation_unit = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::CompilationUnit)
        });
        let item_list = ast.node(compilation_unit.children[0]).unwrap();
        assert_eq!(
            item_list.children.len(),
            2,
            "`{name}` must preserve the theorem"
        );
        assert!(matches!(
            ast.node(item_list.children[0]).unwrap().kind,
            SurfaceNodeKind::PropertyImplementation
        ));
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::TheoremItem
        ));
    }
}

#[test]
fn task48_malformed_body_and_trailing_material_recover_to_the_outer_end() {
    let cases: &[(&str, &[(&str, ParserTokenKind)])] = &[
        (
            "missing body keyword before correctness",
            &[
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "unexpected material after valid body",
            &[
                ("equals", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("unexpected", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "two-level nested malformed tail",
            &[
                ("equals", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("proof", ParserTokenKind::ReservedWord),
                ("now", ParserTokenKind::ReservedWord),
                ("property", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("unexpected", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
    ];

    for (ordinal, (name, body)) in cases.iter().enumerate() {
        let source_id = source_id(232 + ordinal as u8);
        let mut entries = vec![
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("Domain", ParserTokenKind::UserSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("property", ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("value", ParserTokenKind::Identifier),
        ];
        entries.extend_from_slice(body);
        entries.extend_from_slice(&[
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("After", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ]);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(source_id, &entries),
            Vec::new(),
        ));
        assert!(
            output.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
            }),
            "`{name}` should be diagnosed: {:#?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .unwrap_or_else(|| panic!("`{name}` recovery should retain an AST"));
        let compilation_unit = single_node(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::CompilationUnit)
        });
        let item_list = ast.node(compilation_unit.children[0]).unwrap();
        assert_eq!(item_list.children.len(), 2);
        let property = ast.node(item_list.children[0]).unwrap();
        assert!(matches!(
            property.kind,
            SurfaceNodeKind::PropertyImplementation
        ));
        assert!(matches!(
            ast.node(item_list.children[1]).unwrap().kind,
            SurfaceNodeKind::TheoremItem
        ));
        assert!(
            property.children.iter().rev().take(2).any(|child| {
                ast.node(*child)
                    .and_then(mizar_syntax::SurfaceNode::token_text)
                    == Some("end")
            }),
            "`{name}` property must retain its actual outer `end`"
        );
    }
}

#[test]
fn task48_rejects_each_disallowed_or_missing_correctness_shape() {
    let cases: &[(&str, &str, &[&str])] = &[
        ("means missing uniqueness", "means", &["existence"]),
        ("equals existence", "equals", &["existence"]),
        ("equals uniqueness", "equals", &["uniqueness"]),
        (
            "equals duplicate coherence",
            "equals",
            &["coherence", "coherence"],
        ),
    ];

    for (ordinal, (name, body_keyword, conditions)) in cases.iter().enumerate() {
        let source_id = source_id(234 + ordinal as u8);
        let mut entries = vec![
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("Domain", ParserTokenKind::UserSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("property", ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("value", ParserTokenKind::Identifier),
            (*body_keyword, ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ];
        for condition in *conditions {
            entries.push((*condition, ParserTokenKind::ReservedWord));
            entries.push((";", ParserTokenKind::ReservedSymbol));
        }
        entries.extend_from_slice(&[
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ]);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(source_id, &entries),
            Vec::new(),
        ));
        assert!(
            output.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
            }),
            "`{name}` must be rejected: {:#?}",
            output.diagnostics
        );
        let ast = output
            .ast
            .unwrap_or_else(|| panic!("`{name}` recovery should retain an AST"));
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PropertyImplementation
            )),
            1
        );
    }
}

#[test]
fn task48_rejects_generic_definition_parameter_tails_and_attribute_chains() {
    let cases: &[(&str, &[(&str, ParserTokenKind)])] = &[
        (
            "such constraint",
            &[
                ("be", ParserTokenKind::ReservedWord),
                ("Domain", ParserTokenKind::UserSymbol),
                ("such", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
            ],
        ),
        (
            "by constraint",
            &[
                ("be", ParserTokenKind::ReservedWord),
                ("Domain", ParserTokenKind::UserSymbol),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
            ],
        ),
        (
            "attribute chain",
            &[
                ("be", ParserTokenKind::ReservedWord),
                ("odd", ParserTokenKind::UserSymbol),
                ("Domain", ParserTokenKind::UserSymbol),
            ],
        ),
        ("missing be", &[("Domain", ParserTokenKind::UserSymbol)]),
    ];

    for (ordinal, (name, parameter_tail)) in cases.iter().enumerate() {
        let source_id = source_id(240 + ordinal as u8);
        let mut entries = vec![
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
        ];
        entries.extend_from_slice(parameter_tail);
        entries.extend_from_slice(&[
            (";", ParserTokenKind::ReservedSymbol),
            ("property", ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("value", ParserTokenKind::Identifier),
            ("equals", ParserTokenKind::ReservedWord),
            ("M", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ]);
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(source_id, &entries),
            Vec::new(),
        ));
        assert!(
            !output.diagnostics.is_empty(),
            "noncanonical property parameter `{name}` must be rejected"
        );
        let ast = output
            .ast
            .unwrap_or_else(|| panic!("`{name}` recovery should retain an AST"));
        assert_eq!(
            count_nodes(&ast, |kind| matches!(
                kind,
                SurfaceNodeKind::PropertyImplementation
            )),
            1,
            "`{name}` should stay on the dedicated recovery path"
        );
    }
}
