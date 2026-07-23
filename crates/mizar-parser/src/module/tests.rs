use crate::{
    OperatorAssociativity, OperatorFixityEntry, ParseRequest, ParserToken, ParserTokenKind, parse,
};
use mizar_session::{
    BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceId,
    SourceRange,
};
use mizar_syntax::{
    SkippedTokenReason, SurfaceFormulaConnective, SurfaceFormulaConstant, SurfaceNodeKind,
    SurfaceOperatorAssociativity, SurfaceQuantifierKind, SyntaxDiagnosticCode, SyntaxRecoveryKind,
};

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

#[test]
fn parser_builds_compilation_unit_and_placeholder_items() {
    let source_id = source_id(40);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        vec![
            token(source_id, ParserTokenKind::ReservedWord, "theorem", 0, 7),
            token(source_id, ParserTokenKind::Identifier, "T", 8, 9),
            token(source_id, ParserTokenKind::ReservedSymbol, ":", 9, 10),
            token(source_id, ParserTokenKind::ReservedWord, "thesis", 11, 17),
            token(source_id, ParserTokenKind::ReservedSymbol, ";", 17, 18),
            token(source_id, ParserTokenKind::ReservedWord, "lemma", 19, 24),
            token(source_id, ParserTokenKind::Identifier, "L", 25, 26),
            token(source_id, ParserTokenKind::ReservedSymbol, ";", 26, 27),
        ],
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("module skeleton should parse");
    let compilation_unit = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::CompilationUnit)
    });
    let item_list_id = compilation_unit.children[0];
    assert!(matches!(
        ast.node(item_list_id).unwrap().kind,
        SurfaceNodeKind::ItemList
    ));
    let items = &ast.node(item_list_id).unwrap().children;
    assert_eq!(items.len(), 2);
    assert!(matches!(
        ast.node(items[0]).unwrap().kind,
        SurfaceNodeKind::TheoremItem
    ));
    assert!(matches!(
        ast.node(items[1]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
    assert_eq!(ast.node(items[0]).unwrap().range, range(source_id, 0, 18));
    assert_eq!(ast.node(items[1]).unwrap().range, range(source_id, 19, 27));
    assert!(ast.snapshot_text().contains("CompilationUnit range=0..27"));
    assert!(ast.snapshot_text().contains("ItemList range=0..27"));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
}

#[test]
fn parser_dispatches_every_documented_top_level_start() {
    let cases: &[(&str, &[(&str, ParserTokenKind)])] = &[
        (
            "import",
            &[
                ("import", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "export",
            &[
                ("export", ParserTokenKind::ReservedWord),
                ("Std", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "definition",
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "reserve",
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "registration",
            &[
                ("registration", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "claim",
            &[
                ("claim", ParserTokenKind::ReservedWord),
                ("C", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "theorem",
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "lemma",
            &[
                ("lemma", ParserTokenKind::ReservedWord),
                ("L", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "open",
            &[
                ("open", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("O", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "assumed",
            &[
                ("assumed", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "conditional",
            &[
                ("conditional", ParserTokenKind::ReservedWord),
                ("lemma", ParserTokenKind::ReservedWord),
                ("C", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "private",
            &[
                ("private", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("P", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "public",
            &[
                ("public", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Q", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "infix_operator",
            &[
                ("infix_operator", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "prefix_operator",
            &[
                ("prefix_operator", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "postfix_operator",
            &[
                ("postfix_operator", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "synonym",
            &[
                ("synonym", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("B", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "antonym",
            &[
                ("antonym", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("B", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
    ];

    for (index, (name, entries)) in cases.iter().enumerate() {
        let source_id = source_id(50 + index as u8);
        let tokens = token_sequence(source_id, entries);
        let expected_range = tokens
            .first()
            .zip(tokens.last())
            .map(|(first, last)| range(source_id, first.span.start, last.span.end))
            .unwrap();
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert!(output.diagnostics.is_empty(), "{name} should dispatch");
        let ast = output.ast.expect("top-level start should recover an AST");
        let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
        assert_eq!(item_list.children.len(), 1, "{name} should make one item");
        let item = ast.node(item_list.children[0]).unwrap();
        let expected_kind = match *name {
            "import" => SurfaceNodeKind::ImportItem,
            "export" => SurfaceNodeKind::ExportItem,
            "private" | "public" => SurfaceNodeKind::VisibleItem,
            "reserve" => SurfaceNodeKind::ReserveItem,
            "definition" => SurfaceNodeKind::DefinitionBlockItem,
            "registration" => SurfaceNodeKind::RegistrationBlockItem,
            "claim" => SurfaceNodeKind::ClaimBlockItem,
            "synonym" | "antonym" => SurfaceNodeKind::NotationAlias,
            _ => SurfaceNodeKind::PlaceholderItem,
        };
        assert_eq!(item.kind, expected_kind);
        assert_eq!(item.range, expected_range, "{name} placeholder range");
    }
}

#[test]
fn parser_parses_export_prelude_and_visibility_items() {
    let source_id = source_id(82);
    let tokens = token_sequence(
        source_id,
        &[
            ("import", ParserTokenKind::ReservedWord),
            ("Std", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("export", ParserTokenKind::ReservedWord),
            ("Std", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("Core", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            (".", ParserTokenKind::ReservedSymbol),
            ("Facade", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("public", ParserTokenKind::ReservedWord),
            ("open", ParserTokenKind::ReservedWord),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("private", ParserTokenKind::ReservedWord),
            ("synonym", ParserTokenKind::ReservedWord),
            ("P", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("Q", ParserTokenKind::Identifier),
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
    let ast = output.ast.expect("export and visible items should parse");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 4);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::ImportItem
    ));
    let export = ast.node(item_list.children[1]).unwrap();
    assert!(matches!(export.kind, SurfaceNodeKind::ExportItem));
    assert_eq!(export.children.len(), 5);
    assert_eq!(
        ast.node(export.children[0]).unwrap().token_text(),
        Some("export")
    );
    assert!(matches!(
        ast.node(export.children[1]).unwrap().kind,
        SurfaceNodeKind::ModulePath
    ));
    assert_eq!(
        ast.node(export.children[2]).unwrap().token_text(),
        Some(",")
    );
    assert!(matches!(
        ast.node(export.children[3]).unwrap().kind,
        SurfaceNodeKind::ModulePath
    ));
    assert_eq!(
        ast.node(export.children[4]).unwrap().token_text(),
        Some(";")
    );

    let visible_theorem = ast.node(item_list.children[2]).unwrap();
    assert!(matches!(visible_theorem.kind, SurfaceNodeKind::VisibleItem));
    assert_eq!(visible_theorem.children.len(), 2);
    assert!(matches!(
        ast.node(visible_theorem.children[0]).unwrap().kind,
        SurfaceNodeKind::VisibilityMarker
    ));
    assert!(matches!(
        ast.node(visible_theorem.children[1]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));

    let visible_alias = ast.node(item_list.children[3]).unwrap();
    assert!(matches!(visible_alias.kind, SurfaceNodeKind::VisibleItem));
    assert_eq!(visible_alias.children.len(), 2);
    assert!(matches!(
        ast.node(visible_alias.children[0]).unwrap().kind,
        SurfaceNodeKind::VisibilityMarker
    ));
    assert!(matches!(
        ast.node(visible_alias.children[1]).unwrap().kind,
        SurfaceNodeKind::NotationAlias
    ));
}

#[test]
fn parser_recovers_export_after_ordinary_item() {
    let source_id = source_id(83);
    let tokens = token_sequence(
        source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("export", ParserTokenKind::ReservedWord),
            ("Late", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("lemma", ParserTokenKind::ReservedWord),
            ("U", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let export_range = tokens[3].span;
    let skipped_range = range(source_id, tokens[3].span.start, tokens[5].span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::UnexpectedTopLevelToken
    );
    assert_eq!(output.diagnostics[0].primary, export_range);
    let ast = output.ast.expect("late export should recover");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 3);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
    let skipped = ast.node(item_list.children[1]).unwrap();
    assert!(matches!(
        skipped.kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
    assert_eq!(skipped.range, skipped_range);
    assert!(matches!(
        ast.node(item_list.children[2]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
}

#[test]
fn parser_recovers_annotation_prefixed_import_and_export_as_unexpected_items() {
    let source_id = source_id(94);
    let tokens = token_sequence(
        source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("@[", ParserTokenKind::ReservedSymbol),
            ("note", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("@[", ParserTokenKind::ReservedSymbol),
            ("note2", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("export", ParserTokenKind::ReservedWord),
            ("Late", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("@[", ParserTokenKind::ReservedSymbol),
            ("note", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("import", ParserTokenKind::ReservedWord),
            ("Late", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("lemma", ParserTokenKind::ReservedWord),
            ("U", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let export_annotation_range = tokens[3].span;
    let import_annotation_range = tokens[12].span;
    let export_skipped_range = range(source_id, tokens[3].span.start, tokens[11].span.end);
    let import_skipped_range = range(source_id, tokens[12].span.start, tokens[17].span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 2);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::UnexpectedTopLevelToken
    );
    assert_eq!(output.diagnostics[0].primary, export_annotation_range);
    assert_eq!(
        output.diagnostics[1].code,
        SyntaxDiagnosticCode::UnexpectedTopLevelToken
    );
    assert_eq!(output.diagnostics[1].primary, import_annotation_range);
    let ast = output
        .ast
        .expect("annotated import/export should recover as unexpected top-level input");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 4);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
    assert_eq!(
        ast.node(item_list.children[1]).unwrap().range,
        export_skipped_range
    );
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
    assert_eq!(
        ast.node(item_list.children[2]).unwrap().range,
        import_skipped_range
    );
    assert!(matches!(
        ast.node(item_list.children[2]).unwrap().kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
    assert!(matches!(
        ast.node(item_list.children[3]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
}

#[test]
fn parser_recovers_annotation_prefixed_import_while_import_prelude_open() {
    let source_id = source_id(96);
    let tokens = token_sequence(
        source_id,
        &[
            ("@[", ParserTokenKind::ReservedSymbol),
            ("note", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("import", ParserTokenKind::ReservedWord),
            ("Core", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let annotation_range = tokens[0].span;
    let skipped_range = range(source_id, tokens[0].span.start, tokens[5].span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::UnexpectedTopLevelToken
    );
    assert_eq!(output.diagnostics[0].primary, annotation_range);
    let ast = output
        .ast
        .expect("annotated import should recover even while import prelude is open");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    let recovery = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(
        recovery.kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
    assert_eq!(recovery.range, skipped_range);
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
}

#[test]
fn parser_recovers_annotation_prefixed_export_while_export_prelude_open() {
    let source_id = source_id(97);
    let tokens = token_sequence(
        source_id,
        &[
            ("import", ParserTokenKind::ReservedWord),
            ("Core", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("@[", ParserTokenKind::ReservedSymbol),
            ("note", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("export", ParserTokenKind::ReservedWord),
            ("Facade", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let annotation_range = tokens[3].span;
    let skipped_range = range(source_id, tokens[3].span.start, tokens[8].span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::UnexpectedTopLevelToken
    );
    assert_eq!(output.diagnostics[0].primary, annotation_range);
    let ast = output
        .ast
        .expect("annotated export should recover even while export prelude is open");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 3);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::ImportItem
    ));
    let recovery = ast.node(item_list.children[1]).unwrap();
    assert!(matches!(
        recovery.kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
    assert_eq!(recovery.range, skipped_range);
    assert!(matches!(
        ast.node(item_list.children[2]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
}

#[test]
fn parser_recovers_bad_export_path_tail_inside_export_item() {
    let source_id = source_id(84);
    let tokens = token_sequence(
        source_id,
        &[
            ("export", ParserTokenKind::ReservedWord),
            ("123", ParserTokenKind::Numeral),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let bad_range = tokens[1].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedExport
    );
    assert_eq!(output.diagnostics[0].primary, bad_range);
    let ast = output
        .ast
        .expect("bad export path should recover inside export item");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    let export_item = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(export_item.kind, SurfaceNodeKind::ExportItem));
    let recovery = export_item
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        })
        .expect("bad export path token should be owned by a recovery node");
    assert_eq!(recovery.range, bad_range);
    assert_eq!(
        rowan_token_texts(&ast),
        vec!["export", "123", ";", "theorem", "T", ";"]
    );
}

#[test]
fn parser_recovers_bad_export_tail_after_path_inside_export_item() {
    let source_id = source_id(89);
    let tokens = token_sequence(
        source_id,
        &[
            ("export", ParserTokenKind::ReservedWord),
            ("Std", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let bad_range = tokens[2].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedExport
    );
    assert_eq!(output.diagnostics[0].primary, bad_range);
    let ast = output
        .ast
        .expect("bad export tail should recover inside export item");
    let export_item = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ExportItem));
    let recovery = export_item
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        })
        .expect("bad tail token should be owned by export recovery");
    assert_eq!(recovery.range, bad_range);
    assert_eq!(
        rowan_token_texts(&ast),
        vec!["export", "Std", ".", ";", "theorem", "T", ";"]
    );
}

#[test]
fn parser_recovers_bad_export_path_after_comma_inside_export_item() {
    let source_id = source_id(90);
    let tokens = token_sequence(
        source_id,
        &[
            ("export", ParserTokenKind::ReservedWord),
            ("Std", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("123", ParserTokenKind::Numeral),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let bad_range = tokens[3].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedExport
    );
    assert_eq!(output.diagnostics[0].primary, bad_range);
    let ast = output
        .ast
        .expect("bad export path after comma should recover inside export item");
    let export_item = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ExportItem));
    let recovery = export_item
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        })
        .expect("bad comma tail should be owned by export recovery");
    assert_eq!(recovery.range, bad_range);
    assert_eq!(
        rowan_token_texts(&ast),
        vec!["export", "Std", ",", "123", ";"]
    );
}

#[test]
fn parser_diagnoses_missing_export_semicolon_before_next_item() {
    let source_id = source_id(85);
    let tokens = token_sequence(
        source_id,
        &[
            ("export", ParserTokenKind::ReservedWord),
            ("Std", ParserTokenKind::Identifier),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let theorem_range = tokens[2].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MissingSemicolon
    );
    assert_eq!(output.diagnostics[0].primary, theorem_range);
    let ast = output.ast.expect("missing export semicolon should recover");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::ExportItem
    ));
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
}

#[test]
fn parser_recovers_import_after_export_prelude() {
    let source_id = source_id(91);
    let tokens = token_sequence(
        source_id,
        &[
            ("export", ParserTokenKind::ReservedWord),
            ("Facade", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("import", ParserTokenKind::ReservedWord),
            ("Late", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let import_range = tokens[3].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::UnexpectedTopLevelToken
    );
    assert_eq!(output.diagnostics[0].primary, import_range);
    let ast = output.ast.expect("late import after export should recover");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::ExportItem
    ));
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
}

#[test]
fn parser_wraps_annotated_visible_item_in_source_order() {
    let source_id = source_id(86);
    let tokens = token_sequence(
        source_id,
        &[
            ("@[", ParserTokenKind::ReservedSymbol),
            ("label", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("private", ParserTokenKind::ReservedWord),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let expected_range = range(source_id, 0, tokens.last().unwrap().span.end);
    let target_range = range(source_id, tokens[4].span.start, tokens[6].span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("annotated visible item should parse");
    let visible = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem));
    assert_eq!(visible.range, expected_range);
    assert_eq!(visible.children.len(), 3);
    let annotation = ast.node(visible.children[0]).unwrap();
    assert!(matches!(annotation.kind, SurfaceNodeKind::Annotation));
    assert_eq!(
        subtree_token_texts(&ast, annotation),
        vec!["@[".to_owned(), "label".to_owned(), "]".to_owned()]
    );
    let marker = ast.node(visible.children[1]).unwrap();
    assert!(matches!(marker.kind, SurfaceNodeKind::VisibilityMarker));
    assert_eq!(
        ast.node(marker.children[0]).unwrap().token_text(),
        Some("private")
    );
    let item = ast.node(visible.children[2]).unwrap();
    assert!(matches!(item.kind, SurfaceNodeKind::PlaceholderItem));
    assert_eq!(item.range, target_range);
}

#[test]
fn parser_diagnoses_dangling_visibility_marker_inside_visible_item() {
    let source_id = source_id(92);
    let tokens = token_sequence(
        source_id,
        &[
            ("private", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[1].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedVisibility
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output
        .ast
        .expect("dangling visibility should recover inside visible item");
    let visible = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem));
    assert_eq!(visible.children.len(), 2);
    assert!(matches!(
        ast.node(visible.children[0]).unwrap().kind,
        SurfaceNodeKind::VisibilityMarker
    ));
    assert_eq!(
        ast.node(visible.children[1]).unwrap().token_text(),
        Some(";")
    );
}

#[test]
fn parser_diagnoses_duplicate_visibility_marker_inside_visible_item() {
    let source_id = source_id(87);
    let tokens = token_sequence(
        source_id,
        &[
            ("public", ParserTokenKind::ReservedWord),
            ("private", ParserTokenKind::ReservedWord),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let duplicate_range = tokens[1].span;
    let skipped_range = range(source_id, tokens[1].span.start, tokens[3].span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedVisibility
    );
    assert_eq!(output.diagnostics[0].primary, duplicate_range);
    let ast = output
        .ast
        .expect("duplicate visibility should recover inside one visible item");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 1);
    let visible = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(visible.kind, SurfaceNodeKind::VisibleItem));
    assert_eq!(visible.children.len(), 3);
    assert!(matches!(
        ast.node(visible.children[0]).unwrap().kind,
        SurfaceNodeKind::VisibilityMarker
    ));
    let recovery = ast.node(visible.children[1]).unwrap();
    assert!(matches!(
        recovery.kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
    assert_eq!(recovery.range, skipped_range);
    assert_eq!(
        ast.node(visible.children[2]).unwrap().token_text(),
        Some(";")
    );
    assert_eq!(
        rowan_token_texts(&ast),
        vec!["public", "private", "theorem", "T", ";"]
    );
}

#[test]
fn parser_recovers_block_like_invalid_visibility_target_as_one_visible_item() {
    let source_id = source_id(93);
    let tokens = token_sequence(
        source_id,
        &[
            ("private", ParserTokenKind::ReservedWord),
            ("definition", ParserTokenKind::ReservedWord),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("U", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let definition_range = tokens[1].span;
    let skipped_range = range(source_id, tokens[1].span.start, tokens[5].span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedVisibility
    );
    assert_eq!(output.diagnostics[0].primary, definition_range);
    let ast = output
        .ast
        .expect("block-like invalid visible target should recover inside one visible item");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    let visible = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(visible.kind, SurfaceNodeKind::VisibleItem));
    let recovery = visible
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        })
        .expect("invalid definition target should be owned by visible recovery");
    assert_eq!(recovery.range, skipped_range);
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
}

#[test]
fn parser_recovers_prefixed_block_like_invalid_visibility_target_as_one_visible_item() {
    let source_id = source_id(95);
    let tokens = token_sequence(
        source_id,
        &[
            ("public", ParserTokenKind::ReservedWord),
            ("private", ParserTokenKind::ReservedWord),
            ("open", ParserTokenKind::ReservedWord),
            ("definition", ParserTokenKind::ReservedWord),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("U", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let duplicate_range = tokens[1].span;
    let skipped_range = range(source_id, tokens[1].span.start, tokens[7].span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedVisibility
    );
    assert_eq!(output.diagnostics[0].primary, duplicate_range);
    let ast = output
        .ast
        .expect("prefixed block-like invalid visible target should recover inside one item");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    let visible = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(visible.kind, SurfaceNodeKind::VisibleItem));
    let recovery = visible
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        })
        .expect("prefixed invalid definition target should be owned by visible recovery");
    assert_eq!(recovery.range, skipped_range);
    assert_eq!(
        ast.node(visible.children[2]).unwrap().token_text(),
        Some(";")
    );
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
}

#[test]
fn parser_diagnoses_visibility_on_non_visible_top_level_item() {
    let source_id = source_id(88);
    let tokens = token_sequence(
        source_id,
        &[
            ("private", ParserTokenKind::ReservedWord),
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let reserve_range = tokens[1].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedVisibility
    );
    assert_eq!(output.diagnostics[0].primary, reserve_range);
    let ast = output
        .ast
        .expect("invalid visible target should recover inside visible item");
    let visible = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem));
    assert_eq!(visible.children.len(), 3);
    assert!(matches!(
        ast.node(visible.children[1]).unwrap().kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
}

#[test]
fn parser_parses_import_alias_relative_and_branch_items() {
    let source_id = source_id(74);
    let tokens = token_sequence(
        source_id,
        &[
            ("import", ParserTokenKind::ReservedWord),
            ("..", ParserTokenKind::ReservedSymbol),
            ("Core", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("Algebra", ParserTokenKind::Identifier),
            (".{", ParserTokenKind::ReservedSymbol),
            ("Group", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("Ring", ParserTokenKind::Identifier),
            ("}", ParserTokenKind::ReservedSymbol),
            (",", ParserTokenKind::ReservedSymbol),
            (".", ParserTokenKind::ReservedSymbol),
            ("Tools", ParserTokenKind::Identifier),
            ("as", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("Std", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("A", ParserTokenKind::Identifier),
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
    let ast = output.ast.expect("import item should parse");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    let import_item = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(import_item.kind, SurfaceNodeKind::ImportItem));
    assert_eq!(import_item.children.len(), 7);
    assert_eq!(
        ast.node(import_item.children[0]).unwrap().token_text(),
        Some("import")
    );
    assert!(matches!(
        ast.node(import_item.children[1]).unwrap().kind,
        SurfaceNodeKind::ModuleBranchImport
    ));
    assert_eq!(
        ast.node(import_item.children[2]).unwrap().token_text(),
        Some(",")
    );
    assert!(matches!(
        ast.node(import_item.children[3]).unwrap().kind,
        SurfaceNodeKind::ImportAliasDecl
    ));
    assert_eq!(
        ast.node(import_item.children[4]).unwrap().token_text(),
        Some(",")
    );
    assert!(matches!(
        ast.node(import_item.children[5]).unwrap().kind,
        SurfaceNodeKind::ImportAliasDecl
    ));
    assert_eq!(
        ast.node(import_item.children[6]).unwrap().token_text(),
        Some(";")
    );
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));

    let branch = import_item
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| matches!(node.kind, SurfaceNodeKind::ModuleBranchImport))
        .expect("branch import decl should be a concrete child");
    assert_eq!(branch.children.len(), 6);
    assert!(matches!(
        ast.node(branch.children[0]).unwrap().kind,
        SurfaceNodeKind::ModulePath
    ));
    assert_eq!(
        ast.node(branch.children[1]).unwrap().token_text(),
        Some(".{")
    );
    assert!(matches!(
        ast.node(branch.children[2]).unwrap().kind,
        SurfaceNodeKind::PathSegment
    ));
    assert_eq!(
        ast.node(branch.children[3]).unwrap().token_text(),
        Some(",")
    );
    assert!(matches!(
        ast.node(branch.children[4]).unwrap().kind,
        SurfaceNodeKind::PathSegment
    ));
    assert_eq!(
        ast.node(branch.children[5]).unwrap().token_text(),
        Some("}")
    );

    let alias = import_item
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| matches!(node.kind, SurfaceNodeKind::ImportAliasDecl))
        .expect("alias import decl should be a concrete child");
    assert_eq!(alias.children.len(), 3);
    assert!(matches!(
        ast.node(alias.children[0]).unwrap().kind,
        SurfaceNodeKind::ModulePath
    ));
    assert_eq!(
        ast.node(alias.children[1]).unwrap().token_text(),
        Some("as")
    );
    assert!(matches!(
        ast.node(alias.children[2]).unwrap().kind,
        SurfaceNodeKind::PathSegment
    ));

    let simple = import_item
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ImportAliasDecl))
        .find(|node| node.children.len() == 1)
        .expect("plain module-path import should be an alias decl without alias children");
    assert!(matches!(
        ast.node(simple.children[0]).unwrap().kind,
        SurfaceNodeKind::ModulePath
    ));
}

#[test]
fn parser_recovers_unexpected_top_level_tokens_before_next_item() {
    let source_id = source_id(41);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        vec![
            token(source_id, ParserTokenKind::Identifier, "bad", 0, 3),
            token(source_id, ParserTokenKind::ReservedWord, "theorem", 4, 11),
            token(source_id, ParserTokenKind::Identifier, "T", 12, 13),
            token(source_id, ParserTokenKind::ReservedSymbol, ";", 13, 14),
        ],
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::UnexpectedTopLevelToken
    );
    assert_eq!(output.diagnostics[0].primary, range(source_id, 0, 3));
    let ast = output
        .ast
        .expect("unexpected top-level token should recover");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    let skipped = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(
        skipped.kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
    assert_eq!(skipped.range, range(source_id, 0, 3));
    let item = ast.node(item_list.children[1]).unwrap();
    assert!(matches!(item.kind, SurfaceNodeKind::PlaceholderItem));
    assert_eq!(item.range, range(source_id, 4, 14));
    let recovery = ast
        .nodes()
        .iter()
        .find(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        })
        .expect("skipped token recovery should be present");
    assert_eq!(recovery.range, range(source_id, 0, 3));
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
    assert_eq!(
        ast.trivia().skipped_token_ranges()[0].reason,
        SkippedTokenReason::Recovery
    );
    assert_eq!(
        ast.trivia().skipped_token_ranges()[0].range,
        range(source_id, 0, 3)
    );
}

#[test]
fn parser_recovers_import_after_non_import_item() {
    let source_id = source_id(75);
    let tokens = token_sequence(
        source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("import", ParserTokenKind::ReservedWord),
            ("Late", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("Module", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("U", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let import_token_range = tokens[3].span;
    let skipped_range = range(source_id, tokens[3].span.start, tokens[7].span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::UnexpectedTopLevelToken
    );
    assert_eq!(output.diagnostics[0].primary, import_token_range);
    let ast = output
        .ast
        .expect("late import should recover as skipped tokens");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 3);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
    let skipped = ast.node(item_list.children[1]).unwrap();
    assert!(matches!(
        skipped.kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
    assert_eq!(skipped.range, skipped_range);
    assert!(matches!(
        ast.node(item_list.children[2]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
}

#[test]
fn parser_diagnoses_missing_import_semicolon_before_next_item() {
    let source_id = source_id(76);
    let tokens = token_sequence(
        source_id,
        &[
            ("import", ParserTokenKind::ReservedWord),
            ("Std", ParserTokenKind::Identifier),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let theorem_range = tokens[2].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MissingSemicolon
    );
    assert_eq!(output.diagnostics[0].primary, theorem_range);
    let ast = output.ast.expect("missing import semicolon should recover");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::ImportItem
    ));
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
}

#[test]
fn parser_diagnoses_missing_import_alias_after_as() {
    let source_id = source_id(77);
    let tokens = token_sequence(
        source_id,
        &[
            ("import", ParserTokenKind::ReservedWord),
            ("Std", ParserTokenKind::Identifier),
            ("as", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[3].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedImport
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output.ast.expect("malformed import alias should recover");
    let import_item = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ImportItem));
    let alias = import_item
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| matches!(node.kind, SurfaceNodeKind::ImportAliasDecl))
        .expect("malformed alias decl should still be represented");
    assert_eq!(alias.children.len(), 2);
    assert_eq!(
        ast.node(alias.children[1]).unwrap().token_text(),
        Some("as")
    );
}

#[test]
fn parser_recovers_bad_import_path_tail_inside_import_item() {
    let source_id = source_id(79);
    let tokens = token_sequence(
        source_id,
        &[
            ("import", ParserTokenKind::ReservedWord),
            ("123", ParserTokenKind::Numeral),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let bad_range = tokens[1].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedImport
    );
    assert_eq!(output.diagnostics[0].primary, bad_range);
    let ast = output
        .ast
        .expect("bad import path should recover inside import item");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    let import_item = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(import_item.kind, SurfaceNodeKind::ImportItem));
    let recovery = import_item
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        })
        .expect("bad import path token should be owned by a recovery node");
    assert_eq!(recovery.range, bad_range);
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
    assert_eq!(ast.trivia().skipped_token_ranges()[0].range, bad_range);
    assert_eq!(
        rowan_token_texts(&ast),
        vec!["import", "123", ";", "theorem", "T", ";"]
    );
}

#[test]
fn parser_recovers_bad_alias_tail_without_splitting_a_phantom_item() {
    let source_id = source_id(80);
    let tokens = token_sequence(
        source_id,
        &[
            ("import", ParserTokenKind::ReservedWord),
            ("Std", ParserTokenKind::Identifier),
            ("as", ParserTokenKind::ReservedWord),
            ("theorem", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let bad_range = tokens[3].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedImport
    );
    assert_eq!(output.diagnostics[0].primary, bad_range);
    let ast = output
        .ast
        .expect("bad alias tail should recover inside import item");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(
        item_list.children.len(),
        1,
        "bad alias token must not become a phantom theorem item"
    );
    let import_item = ast.node(item_list.children[0]).unwrap();
    let recovery = import_item
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .flat_map(|node| node.children.iter().filter_map(|child| ast.node(*child)))
        .find(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        })
        .expect("bad alias token should be owned by a nested recovery node");
    assert_eq!(recovery.range, bad_range);
}

#[test]
fn parser_diagnoses_missing_branch_import_close() {
    let source_id = source_id(78);
    let tokens = token_sequence(
        source_id,
        &[
            ("import", ParserTokenKind::ReservedWord),
            ("Std", ParserTokenKind::Identifier),
            (".{", ParserTokenKind::ReservedSymbol),
            ("Branch", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[4].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedImport
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output
        .ast
        .expect("missing branch close should recover at semicolon");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::ImportItem
    ));
    assert!(matches!(
        ast.node(item_list.children[1]).unwrap().kind,
        SurfaceNodeKind::PlaceholderItem
    ));
}

#[test]
fn parser_reports_empty_branch_import_once() {
    let source_id = source_id(81);
    let tokens = token_sequence(
        source_id,
        &[
            ("import", ParserTokenKind::ReservedWord),
            ("Std", ParserTokenKind::Identifier),
            (".{", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[3].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedImport
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output.ast.expect("empty branch import should recover");
    let branch = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::ModuleBranchImport)
    });
    assert_eq!(branch.children.len(), 2);
}

#[test]
fn parser_diagnoses_missing_semicolon_before_next_item_boundary() {
    let source_id = source_id(42);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        vec![
            token(source_id, ParserTokenKind::ReservedWord, "theorem", 0, 7),
            token(source_id, ParserTokenKind::Identifier, "T", 8, 9),
            token(source_id, ParserTokenKind::ReservedWord, "lemma", 10, 15),
            token(source_id, ParserTokenKind::Identifier, "L", 16, 17),
            token(source_id, ParserTokenKind::ReservedSymbol, ";", 17, 18),
        ],
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MissingSemicolon
    );
    assert_eq!(output.diagnostics[0].primary, range(source_id, 10, 15));
    let ast = output.ast.expect("missing semicolon should recover");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    assert_eq!(
        ast.node(item_list.children[0]).unwrap().range,
        range(source_id, 0, 9)
    );
    assert_eq!(
        ast.node(item_list.children[1]).unwrap().range,
        range(source_id, 10, 18)
    );
}

#[test]
fn block_placeholder_requires_semicolon_after_matching_end() {
    let source_id = source_id(43);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        vec![
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "definition",
                0,
                10,
            ),
            token(
                source_id,
                ParserTokenKind::ReservedWord,
                "algorithm",
                11,
                20,
            ),
            token(source_id, ParserTokenKind::ReservedWord, "end", 21, 24),
            token(source_id, ParserTokenKind::ReservedWord, "end", 25, 28),
            token(source_id, ParserTokenKind::ReservedWord, "theorem", 29, 36),
            token(source_id, ParserTokenKind::Identifier, "T", 37, 38),
            token(source_id, ParserTokenKind::ReservedSymbol, ";", 38, 39),
        ],
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MissingSemicolon
    );
    assert_eq!(output.diagnostics[0].primary, range(source_id, 29, 36));
    let ast = output.ast.expect("missing block semicolon should recover");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    assert_eq!(
        ast.node(item_list.children[0]).unwrap().range,
        range(source_id, 0, 28)
    );
    assert_eq!(
        ast.node(item_list.children[1]).unwrap().range,
        range(source_id, 29, 39)
    );
}

#[test]
fn proof_block_semicolons_stay_inside_the_theorem_item() {
    let source_id = source_id(68);
    let tokens = token_sequence(
        source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            ("proof", ParserTokenKind::ReservedWord),
            ("thus", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let expected_range = range(source_id, 0, tokens.last().unwrap().span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("proof-bearing theorem should parse");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 1);
    assert_eq!(
        ast.node(item_list.children[0]).unwrap().range,
        expected_range
    );
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::TheoremItem
    ));
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ConclusionStatement
        )),
        1
    );
}

#[test]
fn parser_emits_task35_annotation_nodes_and_wrappers() {
    let source_id = source_id(229);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("@[", ParserTokenKind::ReservedSymbol),
                ("module_note", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("top", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("AnnotatedTop", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("@custom", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("flag", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("@show_thesis", ParserTokenKind::AnnotationMarker),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("@show_type", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("@definition_block", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("defs", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
                ("@latex", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"P\"", ParserTokenKind::StringLiteral),
                (")", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("LatexDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("@proof_hint", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("max_axioms", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("10", ParserTokenKind::Numeral),
                (",", ParserTokenKind::ReservedSymbol),
                ("solver", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("vampire", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Hinted", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("@algo_note", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("flag", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("annotated", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("do", ParserTokenKind::ReservedWord),
                ("@trace", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"entry\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                ("assert", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("@registration_block", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("regs", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("registration", ParserTokenKind::ReservedWord),
                ("@[", ParserTokenKind::ReservedSymbol),
                ("registration_note", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("param", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("@claim_block", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("claims", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("claim", ParserTokenKind::ReservedWord),
                ("annotated", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("@claim_note", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("init", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("ClaimAnnotated", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-35 annotation surfaces should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-35 annotation surfaces should keep an AST");
    assert!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::Annotation)) >= 8,
        "annotation prefixes should be preserved as concrete nodes"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::LibraryAnnotation
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotationLabelList
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotationLabel
        )),
        2
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotationArgumentList
        )) >= 6
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotationArgument
        )) >= 6
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::StandaloneDiagnosticAnnotation
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ProofHintOptionList
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ProofHintOption
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedStatement
        )),
        1,
        "ordinary proof statements should accept annotation prefixes"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedAlgorithmStatement
        )),
        1
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedDefinitionContent
        )) >= 3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedRegistrationContent
        )),
        1
    );
    assert!(ast.nodes().iter().any(|node| {
        matches!(node.kind, SurfaceNodeKind::AnnotatedStatement)
            && subtree_token_texts(&ast, node)
                .iter()
                .any(|text| text == "@show_thesis")
    }));
}

#[test]
fn parser_wraps_visible_theorem_targets_and_preserves_status_tokens() {
    let source_id = source_id(171);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("public", ParserTokenKind::ReservedWord),
                ("open", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("VisibleT", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("conditional", ParserTokenKind::ReservedWord),
                ("lemma", ParserTokenKind::ReservedWord),
                ("VisibleL", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("assumed", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("AssumedT", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "visible theorem targets should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("visible theorem targets should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 3);
    let visible_items = item_list
        .children
        .iter()
        .take(2)
        .map(|item| ast.node(*item).unwrap())
        .collect::<Vec<_>>();
    assert!(
        visible_items
            .iter()
            .all(|item| matches!(item.kind, SurfaceNodeKind::VisibleItem))
    );

    let assumed = ast.node(item_list.children[2]).unwrap();
    assert!(matches!(assumed.kind, SurfaceNodeKind::TheoremItem));
    assert_eq!(
        direct_token_texts(&ast, assumed),
        vec!["assumed", "theorem", "AssumedT", ":", ";"]
    );
    assert!(direct_child_has_kind(&ast, assumed, |kind| {
        matches!(kind, SurfaceNodeKind::FormulaExpression)
    }));

    let theorem = structural_children(&ast, visible_items[0])
        .into_iter()
        .find(|child| matches!(child.kind, SurfaceNodeKind::TheoremItem))
        .expect("public open theorem should be the visible target");
    assert_eq!(
        direct_token_texts(&ast, theorem),
        vec!["open", "theorem", "VisibleT", ":", ";"]
    );
    assert!(direct_child_has_kind(&ast, theorem, |kind| {
        matches!(kind, SurfaceNodeKind::FormulaExpression)
    }));

    let lemma = structural_children(&ast, visible_items[1])
        .into_iter()
        .find(|child| matches!(child.kind, SurfaceNodeKind::LemmaItem))
        .expect("private conditional lemma should be the visible target");
    assert_eq!(
        direct_token_texts(&ast, lemma),
        vec!["conditional", "lemma", "VisibleL", ":", ";"]
    );
    assert!(direct_child_has_kind(&ast, lemma, |kind| {
        matches!(kind, SurfaceNodeKind::FormulaExpression)
    }));
}

#[test]
fn statement_proof_blocks_are_owned_by_statement_hosts() {
    let source_id = source_id(172);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("StatementProofs", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
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
        output.diagnostics.is_empty(),
        "statement-owned proof blocks should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("statement-owned proof blocks should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        3,
        "the theorem proof and both statement proof tails should be represented"
    );

    let compact = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::CompactStatement)
    });
    let compact_children = structural_children(&ast, compact);
    assert!(
        compact_children
            .iter()
            .any(|child| matches!(child.kind, SurfaceNodeKind::ProofBlock))
    );
    assert_eq!(
        direct_token_texts(&ast, compact),
        vec![";"],
        "the compact statement keeps its enclosing semicolon directly"
    );
    let compact_proposition = compact_children
        .iter()
        .find(|child| matches!(child.kind, SurfaceNodeKind::Proposition))
        .expect("compact statement should own a proposition");
    assert_eq!(
        direct_token_texts(&ast, compact_proposition),
        vec!["A", ":"],
        "the compact statement label belongs to its proposition child"
    );

    let conclusions = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ConclusionStatement))
        .collect::<Vec<_>>();
    let proof_conclusion = conclusions
        .iter()
        .copied()
        .find(|node| {
            structural_children(&ast, node)
                .iter()
                .any(|child| matches!(child.kind, SurfaceNodeKind::ProofBlock))
        })
        .expect("one conclusion statement should own a proof block");
    assert_eq!(
        direct_token_texts(&ast, proof_conclusion),
        vec!["thus", ";"],
        "the conclusion statement keeps its enclosing semicolon directly"
    );
}

#[test]
fn parser_recovers_task22_theorem_and_proof_shapes() {
    let source_id = source_id(173);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingColon", ParserTokenKind::Identifier),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingFormula", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingProofEnd", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Next", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("AfterStatementProof", ParserTokenKind::Identifier),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression })
    );
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
    );
    let ast = output
        .ast
        .expect("task-22 malformed theorem/proof shapes should recover an AST");
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 1,
        "missing theorem labels should insert MissingTerm"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )) >= 1,
        "missing theorem formulas should insert MissingFormula"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
        )) >= 2,
        "the theorem proof and statement proof should both report MissingEnd"
    );
    assert!(ast.nodes().iter().any(|node| {
        matches!(node.kind, SurfaceNodeKind::TheoremItem)
            && direct_token_texts(&ast, node) == vec!["theorem", "MissingColon", ";"]
            && direct_child_has_kind(&ast, node, |kind| {
                matches!(kind, SurfaceNodeKind::FormulaExpression)
            })
    }));
    assert!(ast.nodes().iter().any(|node| {
        matches!(node.kind, SurfaceNodeKind::TheoremItem)
            && direct_token_texts(&ast, node).contains(&"Next".to_owned())
    }));
    assert!(ast.nodes().iter().any(|node| {
        matches!(node.kind, SurfaceNodeKind::TheoremItem)
            && direct_token_texts(&ast, node).contains(&"AfterStatementProof".to_owned())
    }));
}

#[test]
fn parser_parses_task23_definition_blocks_and_attributes() {
    let source_id = source_id(175);
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
                ("attr", ParserTokenKind::ReservedWord),
                ("AttrDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("odd", ParserTokenKind::UserSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("contradiction", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("otherwise", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("attr", ParserTokenKind::ReservedWord),
                ("SingleDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("identifierPattern", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("attr", ParserTokenKind::ReservedWord),
                ("PrefixedDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("-", ParserTokenKind::ReservedSymbol),
                ("dimensional", ParserTokenKind::UserSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("assume", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("uniqueness", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("compatibility", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("reducibility", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("consistency", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Plain", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("lemma", ParserTokenKind::ReservedWord),
                ("InnerLemma", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("public", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Inner", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("type", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("Deferred", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-23 definition blocks should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-23 definition blocks should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::DefinitionBlockItem
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::DefinitionParameter
        )),
        1,
        "ordinary definition parameters remain distinct from task-31 template parameters"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateParameter
        )),
        1,
        "template-shaped definition blocks promote leading `let T be type;`"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AttributeDefinition
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AttributePattern
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ParameterPrefix
        )),
        1,
        "prefixed attribute patterns preserve the local parameter-prefix split"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FormulaDefiniens
        )),
        4,
        "case-form, single-formula, and task-31 template predicate definiens bodies are concrete"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FormulaCase)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CorrectnessCondition
        )),
        6
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        1,
        "correctness conditions accept computation justifications"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::LemmaItem)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssumptionStatement
        )),
        1,
        "definition-block content dispatch accepts assumptions"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0,
        "task-31 template parameter and predicate definition content are concrete"
    );
}

#[test]
fn parser_parses_task31_template_definition_content() {
    let source_id = source_id(176);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("type", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("PDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("FDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("equals", ParserTokenKind::ReservedWord),
                ("f", ParserTokenKind::UserSymbol),
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
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-31 template definition content should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("template definition content should keep an AST");
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
            SurfaceNodeKind::TemplateParameter
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TemplateLoci)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TemplateLocus)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateArguments
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateArgument
        )),
        1
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::PredicatePattern)
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TemplateLoci))
        }),
        "predicate patterns should own TemplateLoci directly"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::FunctorPattern)
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TemplateLoci))
        }),
        "functor patterns should own TemplateLoci directly"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0
    );
}

#[test]
fn parser_parses_task31_template_fixture_with_unregistered_local_symbols() {
    let source_id = source_id(185);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("type", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("TemplatePredDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("matches", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("TemplateFuncDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("pick", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("TemplateUse", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("matches", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("iff", ParserTokenKind::ReservedWord),
                ("pick", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("LocalHeadFirst", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("matches", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "unregistered template-local symbols should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("unregistered local template fixture should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateArguments
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TemplateLoci)),
        2
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::PredicateHead)
                && direct_token_texts(&ast, node)
                    .iter()
                    .any(|token| token == "matches")
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TemplateArguments))
        }),
        "head-first template-local identifier predicates should own TemplateArguments"
    );
}

#[test]
fn parser_recovers_task23_definition_block_gaps() {
    let source_id = source_id(177);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("attr", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("is", ParserTokenKind::ReservedWord),
                ("means", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                ("junk", ParserTokenKind::Identifier),
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
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
    );
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
    );
    let ast = output
        .ast
        .expect("task-23 malformed definition content should recover an AST");
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
            SurfaceNodeKind::AttributeDefinition
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CorrectnessCondition
        )),
        1
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 3
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )) >= 1
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )) >= 1
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
        )) >= 1
    );
}

#[test]
fn parser_recovers_task23_definition_content_at_end_boundary() {
    let source_id = source_id(178);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("attr", ParserTokenKind::ReservedWord),
                ("EndSync", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("odd", ParserTokenKind::UserSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
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
        "definition content ending at `end` without `;` should recover at the block boundary"
    );
    assert!(
        !output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd),
        "the existing block `end` should close the definition after content recovery"
    );
    let ast = output
        .ast
        .expect("definition content synchronized at `end` should keep an AST");
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
            SurfaceNodeKind::AttributeDefinition
        )),
        1
    );
}

#[test]
fn parser_recovers_task23_definition_content_at_next_content_start() {
    let source_id = source_id(179);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("attr", ParserTokenKind::ReservedWord),
                ("NextSync", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("odd", ParserTokenKind::UserSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("existence", ParserTokenKind::ReservedWord),
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
        "definition content should recover when the next content starts without `;`"
    );
    let ast = output
        .ast
        .expect("definition content synchronized at next content should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AttributeDefinition
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CorrectnessCondition
        )),
        1
    );
}

#[test]
fn parser_parses_task24_predicate_definitions() {
    let source_id = source_id(180);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("RelDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::Identifier),
                ("y", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("PrefixDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("R", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("contradiction", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("otherwise", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("MultiDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("partition", ParserTokenKind::Identifier),
                ("z", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("LessOrEqualDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("<=", ParserTokenKind::LexemeRun),
                ("y", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("TemplateLociDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("Rel", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("U", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("public", ParserTokenKind::ReservedWord),
                ("pred", ParserTokenKind::ReservedWord),
                ("PublicDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("public_rel", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("pred", ParserTokenKind::ReservedWord),
                ("HiddenDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("hidden", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-24 predicate definitions should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-24 predicate definitions should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PredicateDefinition
        )),
        7
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PredicatePattern
        )),
        7
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FormulaDefiniens
        )),
        7
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FormulaCase)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
        2,
        "definition-local visibility wraps concrete predicate definitions"
    );
    let pattern_token_texts = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::PredicatePattern))
        .map(|node| direct_token_texts(&ast, node))
        .collect::<Vec<_>>();
    assert!(
        pattern_token_texts
            .iter()
            .any(|texts| texts.iter().map(String::as_str).eq(["x", "<=", "y"])),
        "fresh symbolic predicate definitions should preserve LexemeRun symbols"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::PredicatePattern)
                && subtree_token_texts(&ast, node)
                    .iter()
                    .map(String::as_str)
                    .eq(["x", "Rel", "[", "T", ",", "U", "]", "y"])
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TemplateLoci))
        }),
        "template-loci tokens should be owned by a direct TemplateLoci pattern child"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0
    );
}

#[test]
fn parser_recovers_task24_predicate_definition_gaps() {
    let source_id = source_id(181);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("pred", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("BuiltinIn", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("in", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("BuiltinNe", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("<>", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("EmptyTemplate", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("R", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("MissingMeans", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::Identifier),
                ("y", ParserTokenKind::Identifier),
                ("thesis", ParserTokenKind::ReservedWord),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
    );
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
    );
    let ast = output
        .ast
        .expect("task-24 malformed predicate definitions should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PredicateDefinition
        )),
        5
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PredicatePattern
        )),
        5
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 5,
        "label and malformed predicate patterns should insert missing-term recovery"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )) >= 2,
        "missing definiens bodies and missing means recovery should insert missing formulas"
    );
}

#[test]
fn parser_parses_task25_functor_definitions() {
    let source_id = source_id(182);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("func", ParserTokenKind::ReservedWord),
                ("CaseDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("++", ParserTokenKind::UserSymbol),
                ("y", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("if", ParserTokenKind::ReservedWord),
                ("contradiction", ParserTokenKind::ReservedWord),
                ("otherwise", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("PrefixDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("~", ParserTokenKind::UserSymbol),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("PostfixDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("!", ParserTokenKind::UserSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("CallDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("CircumfixDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("|.", ParserTokenKind::UserSymbol),
                ("x", ParserTokenKind::Identifier),
                (".|", ParserTokenKind::UserSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("LexemeRunDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("<+>", ParserTokenKind::LexemeRun),
                ("y", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("TemplateLociDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("op", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("U", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MeansCaseDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("decide", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("otherwise", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MeansDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("measure", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("public", ParserTokenKind::ReservedWord),
                ("func", ParserTokenKind::ReservedWord),
                ("PublicDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("public_f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("func", ParserTokenKind::ReservedWord),
                ("HiddenDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("hidden_f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-25 functor definitions should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-25 functor definitions should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FunctorDefinition
        )),
        11
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FunctorPattern)),
        11
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TermDefiniens)),
        9
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TermCase)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FormulaDefiniens
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FormulaCase)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
        2,
        "definition-local visibility wraps concrete functor definitions"
    );
    let pattern_token_texts = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::FunctorPattern))
        .map(|node| direct_token_texts(&ast, node))
        .collect::<Vec<_>>();
    for expected in [
        &["x", "++", "y"][..],
        &["~", "x"][..],
        &["x", "!"][..],
        &["f", "(", "x", ",", "y", ")"][..],
        &["|.", "x", ".|"][..],
        &["x", "<+>", "y"][..],
    ] {
        assert!(
            pattern_token_texts.iter().any(|texts| texts
                .iter()
                .map(String::as_str)
                .eq(expected.iter().copied())),
            "raw FunctorPattern children should contain {expected:?}"
        );
    }
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::FunctorPattern)
                && subtree_token_texts(&ast, node)
                    .iter()
                    .map(String::as_str)
                    .eq(["op", "[", "T", ",", "U", "]", "x"])
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TemplateLoci))
        }),
        "template-loci tokens should be owned by a direct TemplateLoci functor-pattern child"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0
    );
}

#[test]
fn parser_parses_task31_template_functor_parameters_and_content() {
    let source_id = source_id(183);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("F", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("func", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("S", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("Deferred", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-31 template functor content should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("template-ambiguous functor content should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FunctorDefinition
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateParameter
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::DefinitionParameter
        )),
        0
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0
    );
}

#[test]
fn parser_recovers_task25_functor_definition_gaps() {
    let source_id = source_id(184);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("func", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("equals", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MalformedPattern", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingColon", ParserTokenKind::Identifier),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingArrow", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingReturn", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingBodyFormula", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingBodyTerm", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingBodyEmpty", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingMeansFormula", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("means", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingMeansOtherwiseFormula", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("otherwise", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingOtherwiseTerm", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("otherwise", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingTermCaseCondition", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("if", ParserTokenKind::ReservedWord),
                ("otherwise", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingTermCase", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("otherwise", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
    );
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
    );
    let ast = output
        .ast
        .expect("task-25 malformed functor definitions should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FunctorDefinition
        )),
        13
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::FunctorPattern)),
        13
    );
    assert!(count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TermDefiniens)) >= 4);
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FormulaDefiniens
        )) >= 1,
        "missing body-keyword recovery should keep formula-start bodies as FormulaDefiniens"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 5,
        "labels, malformed patterns, missing term bodies, and term cases insert MissingTerm"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )) >= 2,
        "missing arrows/return types insert MissingTypeExpression"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )) >= 1,
        "missing formula-start body keywords insert MissingFormula when needed"
    );
}

#[test]
fn parser_parses_task26_mode_definitions() {
    let source_id = source_id(185);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("mode", ParserTokenKind::ReservedWord),
                ("SimpleDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Simple", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("AttrDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("NonEmpty", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("OfDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Subset", ParserTokenKind::Identifier),
                ("of", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("Y", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("OverDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Matrix", ParserTokenKind::Identifier),
                ("over", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("BracketDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Family", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("A", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("B", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("SymbolicDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("SethoodCitationDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Finite", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("sethood", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("SethoodComputationDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Computable", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("sethood", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("SethoodProofDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Ordinal", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("sethood", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("public", ParserTokenKind::ReservedWord),
                ("mode", ParserTokenKind::ReservedWord),
                ("PublicDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("PublicMode", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("mode", ParserTokenKind::ReservedWord),
                ("HiddenDef", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("HiddenMode", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-26 mode definitions should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-26 mode definitions should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeDefinition)),
        11
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModePattern)),
        11
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeProperty)),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::JustificationClause
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
        2,
        "definition-local visibility wraps concrete mode definitions"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::ModeDefinition)
                && subtree_token_texts(&ast, node)
                    .iter()
                    .map(String::as_str)
                    .eq([
                        "mode", "AttrDef", ":", "NonEmpty", "is", "non", "empty", "set", ";",
                    ])
                && structural_children(&ast, node).iter().any(|child| {
                    matches!(child.kind, SurfaceNodeKind::TypeExpression)
                        && structural_children(&ast, child).iter().any(|grandchild| {
                            matches!(grandchild.kind, SurfaceNodeKind::AttributeChain)
                        })
                })
        }),
        "mode bodies with attributes should be represented as TypeExpression nodes owning AttributeChain children"
    );
    let pattern_token_texts = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ModePattern))
        .map(|node| direct_token_texts(&ast, node))
        .collect::<Vec<_>>();
    for expected in [
        &["Subset", "of", "X", ",", "Y"][..],
        &["Matrix", "over", "R"][..],
        &["Family", "[", "A", ",", "B", "]"][..],
        &["T"][..],
    ] {
        assert!(
            pattern_token_texts.iter().any(|texts| texts
                .iter()
                .map(String::as_str)
                .eq(expected.iter().copied())),
            "raw ModePattern children should contain {expected:?}"
        );
    }
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0
    );
}

#[test]
fn parser_recovers_task26_mode_definition_gaps() {
    let source_id = source_id(186);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("mode", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("is", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("MissingColon", ParserTokenKind::Identifier),
                ("Simple", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("EmptyParams", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Bad", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("DanglingParam", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Bad", ParserTokenKind::Identifier),
                ("of", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("MultipleParamGroups", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Bad", ParserTokenKind::Identifier),
                ("of", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                ("over", ParserTokenKind::ReservedWord),
                ("Y", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("LexemeRunName", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("<+>", ParserTokenKind::LexemeRun),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("MissingIs", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Bad", ParserTokenKind::Identifier),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("MissingBody", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Bad", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("MissingSemicolon", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Bad", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("sethood", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("MissingSethoodJustification", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Bad", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("sethood", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("mode", ParserTokenKind::ReservedWord),
                ("LegacyMeans", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Bad", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
    );
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
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
        .expect("task-26 malformed mode definitions should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeDefinition)),
        11
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModePattern)),
        11
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeProperty)),
        2
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 5,
        "labels and malformed mode patterns should insert MissingTerm"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )) >= 3,
        "missing mode bodies and legacy means recovery should insert MissingTypeExpression"
    );
}

#[test]
fn parser_parses_task27_redefinitions_and_notation_aliases() {
    let source_id = source_id(187);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("synonym", ParserTokenKind::ReservedWord),
                ("FinSeq", ParserTokenKind::Identifier),
                ("of", ParserTokenKind::ReservedWord),
                ("G", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("FinSequence", ParserTokenKind::Identifier),
                ("of", ParserTokenKind::ReservedWord),
                ("G", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
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
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
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
                ("with", ParserTokenKind::ReservedWord),
                ("C", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("redefine", ParserTokenKind::ReservedWord),
                ("func", ParserTokenKind::ReservedWord),
                ("MeansR", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("g", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("public", ParserTokenKind::ReservedWord),
                ("redefine", ParserTokenKind::ReservedWord),
                ("func", ParserTokenKind::ReservedWord),
                ("FuncR", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("synonym", ParserTokenKind::ReservedWord),
                ("finite", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("antonym", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (">=", ParserTokenKind::UserSymbol),
                ("y", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("<", ParserTokenKind::UserSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("redefine", ParserTokenKind::ReservedWord),
                ("mode", ParserTokenKind::ReservedWord),
                ("ModeR", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("Bad", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
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
        output.diagnostics.is_empty(),
        "task-27 redefinitions and aliases should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-27 redefinitions and aliases should keep an AST");
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
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FunctorRedefinition
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CoherenceCondition
        )),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::NotationAlias)),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::NotationPattern
        )),
        6
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        1
    );
    let redefinition_nodes = ast
        .nodes()
        .iter()
        .filter(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::AttributeRedefinition
                    | SurfaceNodeKind::PredicateRedefinition
                    | SurfaceNodeKind::FunctorRedefinition
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(redefinition_nodes.len(), 4);
    let mut direct_coherence_children = 0;
    for node in redefinition_nodes {
        let children = structural_children(&ast, node);
        assert!(
            children
                .iter()
                .any(|child| matches!(child.kind, SurfaceNodeKind::CoherenceCondition)),
            "{:?} should own a direct CoherenceCondition child",
            node.kind
        );
        assert!(
            !children
                .iter()
                .any(|child| matches!(child.kind, SurfaceNodeKind::CorrectnessCondition)),
            "{:?} should not own a CorrectnessCondition child",
            node.kind
        );
        direct_coherence_children += children
            .iter()
            .filter(|child| matches!(child.kind, SurfaceNodeKind::CoherenceCondition))
            .count();
    }
    assert_eq!(
        direct_coherence_children,
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CoherenceCondition
        )),
        "all task-27 coherence conditions should be nested under redefinitions"
    );
    let predicate_signatures = surface_views(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::PredicateRedefinition)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &predicate_signatures,
        &[
            "redefine",
            "pred",
            "PredR",
            ":",
            "PredicatePattern",
            "means",
            "FormulaDefiniens",
            ";",
            "CoherenceCondition",
        ],
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
        2,
        "visibility should wrap the functor redefinition and private alias"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::VisibleItem)
                && node.children.get(1).is_some_and(|child_id| {
                    matches!(
                        ast.node(*child_id).unwrap().kind,
                        SurfaceNodeKind::FunctorRedefinition
                    )
                })
        }),
        "definition-local visibility should wrap redefinitions"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::VisibleItem)
                && node.children.get(1).is_some_and(|child_id| {
                    matches!(
                        ast.node(*child_id).unwrap().kind,
                        SurfaceNodeKind::NotationAlias
                    )
                })
        }),
        "definition-local visibility should wrap notation aliases"
    );
    let notation_pattern_texts = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::NotationPattern))
        .map(|node| direct_token_texts(&ast, node))
        .collect::<Vec<_>>();
    for expected in [
        &["FinSeq", "of", "G"][..],
        &["FinSequence", "of", "G"][..],
        &["finite"][..],
        &["empty"][..],
        &["x", ">=", "y"][..],
        &["x", "<", "y"][..],
    ] {
        assert!(
            notation_pattern_texts.iter().any(|texts| texts
                .iter()
                .map(String::as_str)
                .eq(expected.iter().copied())),
            "raw NotationPattern children should contain {expected:?}"
        );
    }
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        1,
        "`redefine mode` is not a spec-defined concrete redefinition"
    );
}

#[test]
fn parser_recovers_missing_predicate_redefinition_label_slot() {
    let source_id = source_id(188);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("redefine", ParserTokenKind::ReservedWord),
                ("pred", ParserTokenKind::ReservedWord),
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
        output
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression }),
        "missing predicate redefinition labels should be diagnosed: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("missing predicate redefinition label should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )),
        1,
        "the omitted predicate redefinition label should insert exactly one MissingTerm"
    );
    let predicate_signatures = surface_views(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::PredicateRedefinition)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &predicate_signatures,
        &[
            "redefine",
            "pred",
            "MissingTerm",
            ":",
            "PredicatePattern",
            "means",
            "FormulaDefiniens",
            ";",
            "CoherenceCondition",
        ],
    );
    assert!(
        ast.snapshot_text()
            .contains("ErrorRecovery kind=MissingTerm"),
        "snapshot should distinguish missing-label recovery"
    );
}

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
fn parser_recovers_task27_redefinition_and_notation_gaps() {
    let source_id = source_id(190);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("redefine", ParserTokenKind::ReservedWord),
                ("func", ParserTokenKind::ReservedWord),
                ("MissingReturn", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("redefine", ParserTokenKind::ReservedWord),
                ("pred", ParserTokenKind::ReservedWord),
                ("means", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("with", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("synonym", ParserTokenKind::ReservedWord),
                ("for", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("antonym", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("y", ParserTokenKind::Identifier),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
    );
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
    );
    let ast = output
        .ast
        .expect("task-27 malformed redefinitions and aliases should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FunctorRedefinition
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PredicateRedefinition
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::NotationAlias)),
        2
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )) >= 1,
        "missing redefinition return types should insert MissingTypeExpression"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 3,
        "malformed patterns and notation sides should insert MissingTerm"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )) >= 1,
        "missing formula definiens bodies should insert MissingFormula"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
        )) >= 2,
        "malformed coherence tails should insert MissingProofStep"
    );
}

#[test]
fn parser_parses_task28_property_clauses() {
    let source_id = source_id(191);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("symmetry", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("asymmetry", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("connectedness", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("reflexivity", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("irreflexivity", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("commutativity", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("idempotence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("involutiveness", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("projectivity", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("sethood", ParserTokenKind::ReservedWord),
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
        output.diagnostics.is_empty(),
        "task-28 property clauses should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-28 property clauses should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::PropertyClause)),
        10
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ModeProperty)),
        0,
        "standalone sethood property items use the generic task-28 node"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::JustificationClause
        )),
        8
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        2
    );

    let property_texts = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::PropertyClause))
        .map(|node| subtree_token_texts(&ast, node))
        .collect::<Vec<_>>();
    for expected_keyword in [
        "symmetry",
        "asymmetry",
        "connectedness",
        "reflexivity",
        "irreflexivity",
        "commutativity",
        "idempotence",
        "involutiveness",
        "projectivity",
        "sethood",
    ] {
        assert!(
            property_texts.iter().any(|texts| texts
                .first()
                .is_some_and(|text| text.as_str() == expected_keyword)),
            "property clause should preserve keyword {expected_keyword}"
        );
    }
}

#[test]
fn parser_recovers_task28_property_clause_gaps() {
    let source_id = source_id(192);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("symmetry", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("commutativity", ParserTokenKind::ReservedWord),
                ("junk", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("sethood", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("reflexivity", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                ("irreflexivity", ParserTokenKind::ReservedWord),
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
        .expect("task-28 malformed property clauses should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::PropertyClause)),
        5
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
        )) >= 3,
        "missing property justifications should insert MissingProofStep recovery"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::PropertyClause)
                && subtree_token_texts(&ast, node)
                    .iter()
                    .map(String::as_str)
                    .eq(["irreflexivity", "by", "Ref", ";"])
        }),
        "missing semicolon after the preceding property must preserve the next property item"
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
fn parser_parses_task29_structure_definitions_and_inheritance() {
    let source_id = source_id(194);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("struct", ParserTokenKind::ReservedWord),
                ("Simple", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("carrier", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("property", ParserTokenKind::ReservedWord),
                ("size", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("struct", ParserTokenKind::ReservedWord),
                ("Polynomial", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("R", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("coeffs", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (":=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("struct", ParserTokenKind::ReservedWord),
                ("OfStruct", ParserTokenKind::Identifier),
                ("of", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("base", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("struct", ParserTokenKind::ReservedWord),
                ("OverStruct", ParserTokenKind::Identifier),
                ("over", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("scalar", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("inherit", ParserTokenKind::ReservedWord),
                ("Polynomial", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("R", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("extends", ParserTokenKind::ReservedWord),
                ("TypeCaseStruct", ParserTokenKind::Identifier),
                ("over", ParserTokenKind::ReservedWord),
                ("r", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("inherit", ParserTokenKind::ReservedWord),
                ("Derived", ParserTokenKind::Identifier),
                ("extends", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("carrier", ParserTokenKind::Identifier),
                ("from", ParserTokenKind::ReservedWord),
                ("it", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("property", ParserTokenKind::ReservedWord),
                ("size", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("from", ParserTokenKind::ReservedWord),
                ("size", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("inherit", ParserTokenKind::ReservedWord),
                ("ProofChild", ParserTokenKind::Identifier),
                ("extends", ParserTokenKind::ReservedWord),
                ("ProofParent", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("carrier", ParserTokenKind::Identifier),
                ("from", ParserTokenKind::ReservedWord),
                ("carrier", ParserTokenKind::Identifier),
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
                ("public", ParserTokenKind::ReservedWord),
                ("struct", ParserTokenKind::ReservedWord),
                ("PublicStruct", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("inherit", ParserTokenKind::ReservedWord),
                ("Hidden", ParserTokenKind::Identifier),
                ("extends", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("h", ParserTokenKind::Identifier),
                ("from", ParserTokenKind::ReservedWord),
                ("it", ParserTokenKind::ReservedWord),
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
        output.diagnostics.is_empty(),
        "task-29 structure syntax should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("task-29 structures should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::StructureDefinition
        )),
        5
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::StructurePattern
        )),
        5
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StructureField)),
        5
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::StructureProperty
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::InheritanceDefinition
        )),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::InheritanceTarget
        )),
        8
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FieldRedefinition
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PropertyRedefinition
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CoherenceCondition
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
        2,
        "definition-local visibility wraps struct and inherit definitions"
    );
}

#[test]
fn parser_recovers_task29_structure_definition_gaps() {
    let source_id = source_id(195);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("struct", ParserTokenKind::ReservedWord),
                ("Empty", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("struct", ParserTokenKind::ReservedWord),
                ("MissingSemi", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("carrier", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("property", ParserTokenKind::ReservedWord),
                ("size", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("field", ParserTokenKind::ReservedWord),
                ("missing", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("inherit", ParserTokenKind::ReservedWord),
                ("Memberless", ParserTokenKind::Identifier),
                ("extends", ParserTokenKind::ReservedWord),
                ("Base", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("inherit", ParserTokenKind::ReservedWord),
                ("CoherenceOnly", ParserTokenKind::Identifier),
                ("extends", ParserTokenKind::ReservedWord),
                ("Base", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("inherit", ParserTokenKind::ReservedWord),
                ("Broken", ParserTokenKind::Identifier),
                ("extends", ParserTokenKind::ReservedWord),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("from", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("field", ParserTokenKind::ReservedWord),
                ("typed", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("from", ParserTokenKind::ReservedWord),
                ("carrier", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("property", ParserTokenKind::ReservedWord),
                ("p", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("from", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("with", ParserTokenKind::ReservedWord),
                ("C", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("struct", ParserTokenKind::ReservedWord),
                ("Follows", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("field", ParserTokenKind::ReservedWord),
                ("ok", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
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
        .expect("task-29 malformed structures should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::StructureDefinition
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::InheritanceDefinition
        )),
        3
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::StructureProperty)
                && subtree_token_texts(&ast, node)
                    .iter()
                    .map(String::as_str)
                    .eq(["property", "size", "->", "set", ";"])
        }),
        "missing field semicolon must preserve following structure property"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::StructureDefinition)
                && subtree_token_texts(&ast, node)
                    .iter()
                    .any(|text| text == "Follows")
        }),
        "malformed inheritance content must preserve following structure definitions"
    );
}

#[test]
fn parser_parses_task30_registration_blocks_and_items() {
    let source_id = source_id(196);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("registration", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("let", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("@custom", ParserTokenKind::AnnotationMarker),
                ("(", ParserTokenKind::ReservedSymbol),
                ("flag", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("Annotated", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
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
                ("cluster", ParserTokenKind::ReservedWord),
                ("Parametric", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("C", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("empty", ParserTokenKind::UserSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("finite", ParserTokenKind::UserSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("P", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("-", ParserTokenKind::ReservedSymbol),
                ("empty", ParserTokenKind::UserSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("finite", ParserTokenKind::UserSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("F", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("finite", ParserTokenKind::UserSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("Op", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("++", ParserTokenKind::UserSymbol),
                ("x", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("finite", ParserTokenKind::UserSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("Bracket", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("finite", ParserTokenKind::UserSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("reduce", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("to", ParserTokenKind::ReservedWord),
                ("[", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("reducibility", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
                ("public", ParserTokenKind::ReservedWord),
                ("cluster", ParserTokenKind::ReservedWord),
                ("DC", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("empty", ParserTokenKind::UserSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("finite", ParserTokenKind::UserSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("reduce", ParserTokenKind::ReservedWord),
                ("DR", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("to", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("reducibility", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        operator_fixture_fixities(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-30 registration syntax should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-30 registration syntax should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::RegistrationBlockItem
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::RegistrationParameter
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ExistentialRegistration
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AnnotatedRegistrationContent
        )),
        1,
        "registration items should accept task-35 annotation wrappers"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ConditionalRegistration
        )),
        3,
        "top-level and definition-local conditional registrations parse"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FunctorialRegistration
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ReductionRegistration
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
        2,
        "definition-local visibility wraps registration items"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        1
    );

    let guarded_parameter = ast
        .nodes()
        .iter()
        .find(|node| {
            matches!(node.kind, SurfaceNodeKind::RegistrationParameter)
                && subtree_token_texts(&ast, node)
                    .iter()
                    .any(|text| text == "such")
        })
        .expect("registration parameter should preserve `such that` guards");
    assert!(node_subtree_contains_kind(
        &ast,
        guarded_parameter,
        |kind| { matches!(kind, SurfaceNodeKind::ConditionList) }
    ));
    assert!(node_subtree_contains_kind(
        &ast,
        guarded_parameter,
        |kind| { matches!(kind, SurfaceNodeKind::JustificationClause) }
    ));
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::ConditionalRegistration)
                && node_subtree_contains_kind(&ast, node, |kind| {
                    matches!(kind, SurfaceNodeKind::ParameterPrefix)
                })
        }),
        "registration adjective prefixes should be preserved"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::FunctorialRegistration)
                && node_subtree_contains_kind(&ast, node, |kind| {
                    matches!(kind, SurfaceNodeKind::InfixExpression(_))
                })
        }),
        "operator functorial payloads should parse as functorial registrations"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::FunctorialRegistration)
                && node_subtree_contains_kind(&ast, node, |kind| {
                    matches!(kind, SurfaceNodeKind::ApplicationTerm)
                })
                && subtree_token_texts(&ast, node)
                    .iter()
                    .any(|text| text == "[")
        }),
        "bracket functorial payloads should parse as functorial registrations"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::ExistentialRegistration)
                && subtree_token_texts(&ast, node).windows(3).any(|window| {
                    window[0].as_str() == "T"
                        && window[1].as_str() == "of"
                        && window[2].as_str() == "x"
                })
        }),
        "existential registrations should preserve parameterized registered types"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::ConditionalRegistration)
                && subtree_token_texts(&ast, node).windows(3).any(|window| {
                    window[0].as_str() == "T"
                        && window[1].as_str() == "of"
                        && window[2].as_str() == "x"
                })
        }),
        "conditional registrations should preserve parameterized target types"
    );
    let compound_reduction = ast
        .nodes()
        .iter()
        .find(|node| {
            matches!(node.kind, SurfaceNodeKind::ReductionRegistration)
                && subtree_token_texts(&ast, node)
                    .iter()
                    .any(|text| text == "f")
        })
        .expect("compound reduction should parse");
    assert_eq!(
        compound_reduction
            .children
            .iter()
            .filter_map(|child| ast.node(*child))
            .filter(|child| matches!(child.kind, SurfaceNodeKind::TermExpression))
            .count(),
        2,
        "reduction registrations should preserve left and right term expressions"
    );
    assert!(node_subtree_contains_kind(
        &ast,
        compound_reduction,
        |kind| { matches!(kind, SurfaceNodeKind::ApplicationTerm) }
    ));
    assert!(
        subtree_token_texts(&ast, compound_reduction)
            .iter()
            .any(|text| text == "["),
        "compound reduction terms should preserve bracket syntax"
    );
}

#[test]
fn parser_recovers_task30_registration_gaps() {
    let source_id = source_id(197);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("registration", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("type", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("BadExistentialArg", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("bad", ParserTokenKind::UserSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("2", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("MissingCorrectness", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("empty", ParserTokenKind::UserSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("empty", ParserTokenKind::UserSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("AfterMissingCorrectness", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("C", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("BadQua", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("finite", ParserTokenKind::UserSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("BadFunctor", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("0", ParserTokenKind::Numeral),
                ("->", ParserTokenKind::ReservedSymbol),
                ("finite", ParserTokenKind::UserSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("cluster", ParserTokenKind::ReservedWord),
                ("Args", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("empty", ParserTokenKind::UserSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("divisible", ParserTokenKind::UserSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("2", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("coherence", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("reduce", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("to", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("reducibility", ParserTokenKind::ReservedWord),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification)
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
    );
    let ast = output
        .ast
        .expect("task-30 malformed registrations should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::RegistrationBlockItem
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ExistentialRegistration
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ConditionalRegistration
        )),
        3,
        "missing antecedents, missing correctness, and malformed consequents recover as conditional registrations"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FunctorialRegistration
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ReductionRegistration
        )),
        1
    );
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
fn proof_blocks_remain_rejected_for_simple_justification_only_hosts() {
    let source_id = source_id(174);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("proof", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("consider", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (".=", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("proof", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression),
        "unexpected proof tails on simple hosts should be diagnosed: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("simple-host proof tails should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        0,
        "simple-justification-only hosts must not accept proof blocks"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )) >= 4,
        "each non-proof host should recover unexpected proof syntax as skipped tokens"
    );
}

#[test]
fn annotation_prefix_stays_with_the_following_placeholder_item() {
    let source_id = source_id(69);
    let tokens = token_sequence(
        source_id,
        &[
            ("@[", ParserTokenKind::ReservedSymbol),
            ("label", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let expected_range = range(source_id, 0, tokens.last().unwrap().span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("annotated placeholder item should parse");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 1);
    let item = ast.node(item_list.children[0]).unwrap();
    assert_eq!(item.range, expected_range);
    assert_eq!(item.children.len(), 6);
}

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

#[test]
fn reserved_word_garbage_before_first_item_is_recovered() {
    let source_id = source_id(74);
    let tokens = token_sequence(
        source_id,
        &[
            ("and", ParserTokenKind::ReservedWord),
            ("theorem", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::UnexpectedTopLevelToken
    );
    assert_eq!(output.diagnostics[0].primary, range(source_id, 0, 3));
    let ast = output
        .ast
        .expect("reserved garbage before an item should recover");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
    ));
}

#[test]
fn formula_if_and_otherwise_do_not_open_nested_block_depth() {
    let source_id = source_id(71);
    let tokens = token_sequence(
        source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("property", ParserTokenKind::ReservedWord),
            ("F", ParserTokenKind::Identifier),
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
        !output.diagnostics.is_empty(),
        "incomplete direct property syntax should use Task-48 recovery"
    );
    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != SyntaxDiagnosticCode::MissingEnd),
        "formula `if`/`otherwise` must not unbalance the property block: {:#?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("conditional formula keywords should not unbalance the block");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 1);
    assert!(matches!(
        ast.node(item_list.children[0]).unwrap().kind,
        SurfaceNodeKind::PropertyImplementation
    ));
}

#[test]
fn statement_if_after_separator_stays_inside_block_placeholder() {
    let source_id = source_id(73);
    let tokens = token_sequence(
        source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("algorithm", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("if", ParserTokenKind::ReservedWord),
            ("P", ParserTokenKind::Identifier),
            ("do", ParserTokenKind::ReservedWord),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let expected_range = range(source_id, 0, tokens.last().unwrap().span.end);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert!(output.diagnostics.is_empty());
    let ast = output
        .ast
        .expect("statement if should not close its container early");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 1);
    assert_eq!(
        ast.node(item_list.children[0]).unwrap().range,
        expected_range
    );
}

#[test]
fn eof_missing_semicolon_placeholder_keeps_consumed_tokens() {
    let source_id = source_id(72);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        vec![
            token(source_id, ParserTokenKind::ReservedWord, "theorem", 0, 7),
            token(source_id, ParserTokenKind::Identifier, "T", 8, 9),
        ],
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MissingSemicolon
    );
    let ast = output.ast.expect("EOF recovery should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 1);
    let item = ast.node(item_list.children[0]).unwrap();
    assert_eq!(item.range, range(source_id, 0, 9));
    assert_eq!(item.children.len(), 2);
}

#[test]
fn parser_parses_reserve_type_expression_attribute_chain_and_of_arguments() {
    let source_id = source_id(98);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("y", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("non", ParserTokenKind::ReservedWord),
            ("empty", ParserTokenKind::UserSymbol),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("a", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("b", ParserTokenKind::Identifier),
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
    let ast = output.ast.expect("reserve type expression should parse");
    let reserve = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ReserveItem));
    assert_eq!(reserve.children.len(), 3);
    let segment = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ReserveSegment));
    assert_eq!(
        segment
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter_map(mizar_syntax::SurfaceNode::token_text)
            .collect::<Vec<_>>(),
        vec!["x", ",", "y", "for"]
    );
    let type_expression = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeExpression));
    assert_eq!(type_expression.children.len(), 2);
    assert!(matches!(
        ast.node(type_expression.children[0]).unwrap().kind,
        SurfaceNodeKind::AttributeChain
    ));
    assert!(matches!(
        ast.node(type_expression.children[1]).unwrap().kind,
        SurfaceNodeKind::TypeHead
    ));
    let attribute = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::AttributeRef));
    assert_eq!(
        ast.node(attribute.children[0]).unwrap().token_text(),
        Some("non")
    );
    assert!(matches!(
        ast.node(attribute.children[1]).unwrap().kind,
        SurfaceNodeKind::QualifiedSymbol
    ));
    let type_arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    assert_eq!(
        ast.node(type_arguments.children[0]).unwrap().token_text(),
        Some("of")
    );
    assert_eq!(
        type_arguments
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
            .count(),
        2
    );
}

#[test]
fn parser_parses_over_type_arguments_as_term_expressions() {
    let source_id = source_id(116);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("w", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("over", ParserTokenKind::ReservedWord),
            ("c", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("d", ParserTokenKind::Identifier),
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
    let ast = output.ast.expect("over arguments should parse");
    let type_arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    assert_eq!(
        ast.node(type_arguments.children[0]).unwrap().token_text(),
        Some("over")
    );
    assert_eq!(
        type_arguments
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
            .count(),
        2
    );
}

#[test]
fn parser_parses_qualified_symbol_term_references() {
    let source_id = source_id(117);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("Ns", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("Value", ParserTokenKind::UserSymbol),
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
    let ast = output.ast.expect("qualified term should parse");
    let term_reference = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TermReference));
    let qualified = term_reference
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .find(|node| matches!(node.kind, SurfaceNodeKind::QualifiedSymbol))
        .expect("term reference should own a qualified symbol");
    assert_eq!(
        qualified_symbol_token_texts(&ast, qualified),
        vec!["Ns", ".", "Value"]
    );
}

#[test]
fn parser_recovers_missing_commas_between_of_over_terms() {
    let source_id = source_id(118);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("a", ParserTokenKind::Identifier),
            ("b", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let b_range = tokens[6].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, b_range);
    let ast = output.ast.expect("missing comma should recover");
    let type_arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    assert_eq!(
        type_arguments
            .children
            .iter()
            .filter_map(|id| ast.node(*id))
            .filter(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
            .count(),
        1
    );
    assert!(type_arguments.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )
    }));
}

#[test]
fn parser_recovers_missing_commas_inside_delimited_term_lists() {
    let source_id = source_id(119);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("f", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            ("b", ParserTokenKind::Identifier),
            (")", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let b_range = tokens[8].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, b_range);
    let ast = output
        .ast
        .expect("missing comma in application should recover");
    let application = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::ApplicationTerm)
    });
    assert!(application.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )
    }));
    assert!(!application.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
        )
    }));
}

#[test]
fn parser_recovers_missing_attribute_argument_close_before_type_head() {
    let source_id = source_id(120);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("empty", ParserTokenKind::UserSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            ("T", ParserTokenKind::UserSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let type_head_range = tokens[6].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, type_head_range);
    let ast = output.ast.expect("missing attribute close should recover");
    let attribute = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::AttributeRef));
    assert!(attribute.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
        )
    }));
    let type_head = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeHead));
    assert_eq!(
        qualified_symbol_token_texts(
            &ast,
            ast.node(type_head.children[0])
                .expect("type head should own a qualified symbol")
        ),
        vec!["T"]
    );
}

#[test]
fn parser_parses_structure_constructors_with_of_arguments() {
    let source_id = source_id(121);
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
            ("a", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("b", ParserTokenKind::Identifier),
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
        .expect("structure constructor with of args should parse");
    let constructor = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::StructureConstructor)
    });
    assert!(
        constructor
            .children
            .iter()
            .any(|id| { matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TypeArguments) })
    );
    assert!(
        constructor
            .children
            .iter()
            .any(|id| { matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::FieldArgument) })
    );
}

#[test]
fn parser_parses_selector_access_chains_and_calls() {
    let source_id = source_id(124);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("line", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("finish", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("y", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("M", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("binop", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("b", ParserTokenKind::Identifier),
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
    let ast = output.ast.expect("selector terms should parse");
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::SelectorAccess))
            .count(),
        4
    );
    assert!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::SelectorAccess))
            .any(|node| {
                node.children
                    .iter()
                    .filter_map(|id| ast.node(*id))
                    .filter_map(mizar_syntax::SurfaceNode::token_text)
                    .collect::<Vec<_>>()
                    == vec![".", "binop", "(", ",", ")"]
            })
    );
}

#[test]
fn parser_parses_functional_structure_updates() {
    let source_id = source_id(125);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            ("with", ParserTokenKind::ReservedWord),
            ("(", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (":=", ParserTokenKind::ReservedSymbol),
            ("y", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("start", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (":=", ParserTokenKind::ReservedSymbol),
            ("f", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (")", ParserTokenKind::ReservedSymbol),
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
    let ast = output.ast.expect("structure update should parse");
    single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::StructureUpdate)
    });
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::FieldUpdate))
            .count(),
        2
    );
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::ApplicationTerm))
    );
}

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
fn parser_recovers_missing_structure_update_value() {
    let source_id = source_id(127);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            ("with", ParserTokenKind::ReservedWord),
            ("(", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (":=", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let close_range = tokens[10].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, close_range);
    let ast = output.ast.expect("missing update value should recover");
    let field_update = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::FieldUpdate));
    assert!(field_update.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )
    }));
}

#[test]
fn parser_recovers_missing_structure_update_close() {
    let source_id = source_id(128);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            ("with", ParserTokenKind::ReservedWord),
            ("(", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (":=", ParserTokenKind::ReservedSymbol),
            ("y", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[11].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output.ast.expect("missing update close should recover");
    let structure_update = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::StructureUpdate)
    });
    assert!(structure_update.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
        )
    }));
}

#[test]
fn parser_recovers_missing_selector_name() {
    let source_id = source_id(129);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[7].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output.ast.expect("missing selector name should recover");
    single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::SelectorAccess));
}

#[test]
fn parser_recovers_missing_selector_call_close() {
    let source_id = source_id(130);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("f", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[10].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output
        .ast
        .expect("missing selector-call close should recover");
    let selector = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::SelectorAccess));
    assert!(selector.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
        )
    }));
}

#[test]
fn parser_recovers_malformed_structure_update_selector_path_once() {
    let source_id = source_id(131);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            ("with", ParserTokenKind::ReservedWord),
            ("(", ParserTokenKind::ReservedSymbol),
            ("start", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let close_range = tokens[10].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, close_range);
    let ast = output
        .ast
        .expect("malformed update selector path should recover");
    single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::FieldUpdate));
}

#[test]
fn parser_parses_qua_qualification_chains_left_associatively() {
    let source_id = source_id(132);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("a", ParserTokenKind::Identifier),
            ("qua", ParserTokenKind::ReservedWord),
            ("R", ParserTokenKind::UserSymbol),
            ("qua", ParserTokenKind::ReservedWord),
            ("S", ParserTokenKind::UserSymbol),
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
    let ast = output.ast.expect("qua chain should parse");
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::QuaExpression))
            .count(),
        2
    );
    assert!(ast.nodes().iter().any(|node| {
        matches!(node.kind, SurfaceNodeKind::QuaExpression)
            && node
                .children
                .iter()
                .any(|id| matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::QuaExpression))
    }));
}

#[test]
fn parser_parses_qua_after_selector_application_and_parentheses() {
    let source_id = source_id(133);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("p", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("qua", ParserTokenKind::ReservedWord),
            ("R", ParserTokenKind::UserSymbol),
            (",", ParserTokenKind::ReservedSymbol),
            ("f", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (")", ParserTokenKind::ReservedSymbol),
            ("qua", ParserTokenKind::ReservedWord),
            ("R", ParserTokenKind::UserSymbol),
            (",", ParserTokenKind::ReservedSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            ("p", ParserTokenKind::Identifier),
            ("qua", ParserTokenKind::ReservedWord),
            ("R", ParserTokenKind::UserSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            (".", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
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
    let ast = output.ast.expect("qua precedence examples should parse");
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::QuaExpression))
            .count(),
        3
    );
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::SelectorAccess))
            .count(),
        2
    );
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::ApplicationTerm))
    );
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm))
    );

    assert!(ast.nodes().iter().any(|node| {
        matches!(node.kind, SurfaceNodeKind::QuaExpression)
            && direct_child_has_kind(&ast, node, |kind| {
                matches!(kind, SurfaceNodeKind::SelectorAccess)
            })
    }));
    assert!(ast.nodes().iter().any(|node| {
        matches!(node.kind, SurfaceNodeKind::QuaExpression)
            && direct_child_has_kind(&ast, node, |kind| {
                matches!(kind, SurfaceNodeKind::ApplicationTerm)
            })
    }));
    assert!(ast.nodes().iter().any(|node| {
        matches!(node.kind, SurfaceNodeKind::SelectorAccess)
            && direct_child_has_kind(&ast, node, |kind| {
                matches!(kind, SurfaceNodeKind::ParenthesizedTerm)
            })
    }));
    assert!(ast.nodes().iter().any(|node| {
        matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm)
            && subtree_contains_kind(&ast, node.children[1], |kind| {
                matches!(kind, SurfaceNodeKind::QuaExpression)
            })
    }));
}

#[test]
fn parser_parses_qua_inside_target_type_arguments_before_outer_chain() {
    let source_id = source_id(134);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("qua", ParserTokenKind::ReservedWord),
            ("Element", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("S", ParserTokenKind::UserSymbol),
            ("qua", ParserTokenKind::ReservedWord),
            ("Magma", ParserTokenKind::UserSymbol),
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
        .expect("qua inside target type argument should parse");
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::QuaExpression))
            .count(),
        2
    );
    assert!(ast.nodes().iter().any(|node| {
        matches!(node.kind, SurfaceNodeKind::QuaExpression)
            && node.children.iter().any(|id| {
                matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TypeExpression)
                    && subtree_contains_kind(&ast, *id, |kind| {
                        matches!(kind, SurfaceNodeKind::QuaExpression)
                    })
            })
    }));
}

#[test]
fn parser_recovers_missing_qua_target_type() {
    let source_id = source_id(135);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("qua", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[7].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output.ast.expect("missing qua target should recover");
    let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
    assert!(qua.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )
    }));
}

#[test]
fn parser_recovers_malformed_qua_target_tail() {
    let source_id = source_id(136);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("qua", ParserTokenKind::ReservedWord),
            ("bad", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let bad_range = tokens[7].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(output.diagnostics[0].primary, bad_range);
    let ast = output.ast.expect("malformed qua target should recover");
    let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
    assert!(qua.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )
    }));
    assert!(qua.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )
    }));
}

#[test]
fn parser_recovers_missing_bracket_qua_target_type() {
    let source_id = source_id(137);
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
            ("]", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let close_range = tokens[7].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(output.diagnostics[0].primary, close_range);
    let ast = output
        .ast
        .expect("missing bracket qua target should recover");
    let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
    assert!(qua.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )
    }));
}

#[test]
fn parser_rejects_attribute_bearing_bracket_qua_target_as_radix_only() {
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
    let non_range = tokens[7].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(output.diagnostics[0].primary, non_range);
    let ast = output
        .ast
        .expect("attribute-bearing bracket qua target should recover");
    let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
    assert!(qua.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )
    }));
    assert!(qua.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )
    }));
}

#[test]
fn parser_recovers_choice_terms_missing_type_expression() {
    let source_id = source_id(122);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("the", ParserTokenKind::ReservedWord),
            (",", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let comma_range = tokens[6].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(output.diagnostics[0].primary, comma_range);
    let ast = output
        .ast
        .expect("choice missing type expression should recover");
    let choice = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ChoiceTerm));
    assert!(choice.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )
    }));
}

#[test]
fn parser_parses_choice_terms_with_set_enumeration_type_args_in_attribute_args() {
    let source_id = source_id(123);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("empty", ParserTokenKind::UserSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            ("the", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("{", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("b", ParserTokenKind::Identifier),
            ("}", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("U", ParserTokenKind::UserSymbol),
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
        .expect("choice term with set enumeration type arg should parse");
    let attribute = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::AttributeRef));
    assert!(
        attribute
            .children
            .iter()
            .any(|id| { matches!(ast.node(*id).unwrap().kind, SurfaceNodeKind::TermExpression) })
    );
    single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ChoiceTerm));
    single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::SetEnumeration));
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
fn parser_parses_primary_terms_in_type_and_attribute_arguments() {
    let source_id = source_id(111);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("empty", ParserTokenKind::UserSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            ("it", ParserTokenKind::ReservedWord),
            (",", ParserTokenKind::ReservedSymbol),
            ("7", ParserTokenKind::Numeral),
            (",", ParserTokenKind::ReservedSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (")", ParserTokenKind::ReservedSymbol),
            (",", ParserTokenKind::ReservedSymbol),
            ("the", ParserTokenKind::ReservedWord),
            ("set", ParserTokenKind::ReservedWord),
            (",", ParserTokenKind::ReservedSymbol),
            ("f", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("1", ParserTokenKind::Numeral),
            (")", ParserTokenKind::ReservedSymbol),
            (",", ParserTokenKind::ReservedSymbol),
            ("S", ParserTokenKind::UserSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("y", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("{", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("b", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            ("}", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            (",", ParserTokenKind::ReservedSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("b", ParserTokenKind::Identifier),
            ("]", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("{", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            ("b", ParserTokenKind::Identifier),
            ("}", ParserTokenKind::ReservedSymbol),
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
    let ast = output.ast.expect("primary terms should parse");
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::ItTerm))
    );
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::NumeralTerm))
    );
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm))
    );
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::ChoiceTerm))
    );
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::ApplicationTerm))
    );
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::StructureConstructor))
    );
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::FieldArgument))
            .count(),
        2
    );
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::SetEnumeration))
    );
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
fn parser_recovers_missing_type_argument_bracket_close() {
    let source_id = source_id(100);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("set", ParserTokenKind::ReservedWord),
            (",", ParserTokenKind::ReservedSymbol),
            ("object", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[8].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output.ast.expect("missing bracket close should recover");
    let type_arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    assert!(type_arguments.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
        )
    }));
}

#[test]
fn parser_recovers_empty_and_trailing_bracket_type_arguments() {
    let empty_source_id = source_id(103);
    let tokens = token_sequence(
        empty_source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("]", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let close_range = tokens[5].span;
    let output = parse(ParseRequest::new(
        empty_source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(output.diagnostics[0].primary, close_range);
    let ast = output.ast.expect("empty bracket args should recover");
    let type_arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    assert!(type_arguments.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )
    }));

    let trailing_source_id = source_id(104);
    let tokens = token_sequence(
        trailing_source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("set", ParserTokenKind::ReservedWord),
            (",", ParserTokenKind::ReservedSymbol),
            ("]", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let close_range = tokens[7].span;
    let output = parse(ParseRequest::new(
        trailing_source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(output.diagnostics[0].primary, close_range);
    let ast = output
        .ast
        .expect("trailing comma bracket args should recover");
    let type_arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    assert!(type_arguments.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )
    }));
}

#[test]
fn parser_diagnoses_missing_of_over_type_arguments_around_commas() {
    let leading_source_id = source_id(105);
    let tokens = token_sequence(
        leading_source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("over", ParserTokenKind::ReservedWord),
            (",", ParserTokenKind::ReservedSymbol),
            ("c", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let comma_range = tokens[5].span;
    let output = parse(ParseRequest::new(
        leading_source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, comma_range);
    let ast = output.ast.expect("leading missing term should recover");
    let type_arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    assert!(type_arguments.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )
    }));

    let trailing_source_id = source_id(106);
    let tokens = token_sequence(
        trailing_source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("a", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[7].span;
    let output = parse(ParseRequest::new(
        trailing_source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output.ast.expect("trailing missing term should recover");
    let type_arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    assert!(type_arguments.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )
    }));
}

#[test]
fn parser_diagnoses_missing_attribute_arguments_around_commas() {
    let empty_source_id = source_id(107);
    let tokens = token_sequence(
        empty_source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("empty", ParserTokenKind::UserSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("T", ParserTokenKind::UserSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let close_range = tokens[5].span;
    let output = parse(ParseRequest::new(
        empty_source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, close_range);
    let ast = output.ast.expect("empty attribute args should recover");
    let attribute = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::AttributeRef));
    assert!(attribute.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )
    }));

    let leading_source_id = source_id(108);
    let tokens = token_sequence(
        leading_source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("empty", ParserTokenKind::UserSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            (",", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (")", ParserTokenKind::ReservedSymbol),
            ("T", ParserTokenKind::UserSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let comma_range = tokens[5].span;
    let output = parse(ParseRequest::new(
        leading_source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, comma_range);

    let trailing_source_id = source_id(109);
    let tokens = token_sequence(
        trailing_source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("empty", ParserTokenKind::UserSymbol),
            ("(", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (",", ParserTokenKind::ReservedSymbol),
            (")", ParserTokenKind::ReservedSymbol),
            ("T", ParserTokenKind::UserSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let close_range = tokens[7].span;
    let output = parse(ParseRequest::new(
        trailing_source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTermExpression
    );
    assert_eq!(output.diagnostics[0].primary, close_range);
}

#[test]
fn parser_recovers_missing_primary_term_delimiters() {
    for (case, opener, expected_recovery_parent) in [
        (
            113,
            ("(", ParserTokenKind::ReservedSymbol),
            SurfaceNodeKind::ParenthesizedTerm,
        ),
        (
            114,
            ("[", ParserTokenKind::ReservedSymbol),
            SurfaceNodeKind::ApplicationTerm,
        ),
        (
            115,
            ("{", ParserTokenKind::ReservedSymbol),
            SurfaceNodeKind::SetEnumeration,
        ),
    ] {
        let source_id = source_id(case);
        let tokens = token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                opener,
                ("a", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        );
        let semicolon_range = tokens[7].span;
        let output = parse(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            tokens,
            Vec::new(),
        ));

        assert_eq!(output.diagnostics.len(), 1);
        assert_eq!(
            output.diagnostics[0].code,
            SyntaxDiagnosticCode::MalformedTermExpression
        );
        assert_eq!(output.diagnostics[0].primary, semicolon_range);
        let ast = output.ast.expect("missing term delimiter should recover");
        let parent = single_node(&ast, |kind| *kind == expected_recovery_parent);
        assert!(parent.children.iter().any(|id| {
            matches!(
                ast.node(*id).unwrap().kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
            )
        }));
    }
}

#[test]
fn parser_recovers_trailing_bracket_type_argument_before_boundary() {
    let source_id = source_id(110);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("[", ParserTokenKind::ReservedSymbol),
            ("set", ParserTokenKind::ReservedWord),
            (",", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let semicolon_range = tokens[7].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(output.diagnostics[0].primary, semicolon_range);
    let ast = output
        .ast
        .expect("trailing bracket arg before boundary should recover");
    let type_arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
    assert!(type_arguments.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )
    }));
    assert!(type_arguments.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
        )
    }));
}

#[test]
fn parser_recovers_malformed_reserve_type_expression_tail() {
    let source_id = source_id(101);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("non", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let non_range = tokens[3].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::MalformedTypeExpression
    );
    assert_eq!(output.diagnostics[0].primary, non_range);
    let ast = output.ast.expect("malformed reserve type should recover");
    let segment = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ReserveSegment));
    assert!(segment.children.iter().any(|id| {
        matches!(
            ast.node(*id).unwrap().kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )
    }));
}

#[test]
fn parser_groups_active_operator_terms_before_qua() {
    let source_id = source_id(160);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("~", ParserTokenKind::UserSymbol),
            ("f", ParserTokenKind::Identifier),
            ("(", ParserTokenKind::ReservedSymbol),
            ("a", ParserTokenKind::Identifier),
            (")", ParserTokenKind::ReservedSymbol),
            ("!", ParserTokenKind::UserSymbol),
            ("++", ParserTokenKind::UserSymbol),
            ("p", ParserTokenKind::Identifier),
            (".", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("**", ParserTokenKind::UserSymbol),
            ("b", ParserTokenKind::Identifier),
            ("qua", ParserTokenKind::ReservedWord),
            ("R", ParserTokenKind::UserSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        operator_fixture_fixities(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("operator term should parse");
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::PrefixExpression(_)))
            .count(),
        1
    );
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::PostfixExpression(_)))
            .count(),
        1
    );
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::InfixExpression(_)))
            .count(),
        2
    );

    let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
    assert!(direct_child_has_kind(&ast, qua, |kind| {
        matches!(kind, SurfaceNodeKind::InfixExpression(operator) if operator.spelling.as_ref() == "++")
    }));

    let prefix = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::PrefixExpression(_))
    });
    let SurfaceNodeKind::PrefixExpression(prefix_operator) = &prefix.kind else {
        panic!("expected prefix expression");
    };
    assert_eq!(prefix_operator.spelling.as_ref(), "~");
    assert_eq!(prefix_operator.precedence, 70);
    assert!(direct_child_has_kind(&ast, prefix, |kind| {
        matches!(kind, SurfaceNodeKind::PostfixExpression(_))
    }));

    let postfix = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::PostfixExpression(_))
    });
    let SurfaceNodeKind::PostfixExpression(postfix_operator) = &postfix.kind else {
        panic!("expected postfix expression");
    };
    assert_eq!(postfix_operator.spelling.as_ref(), "!");
    assert_eq!(postfix_operator.precedence, 90);
    assert!(direct_child_has_kind(&ast, postfix, |kind| {
        matches!(kind, SurfaceNodeKind::ApplicationTerm)
    }));

    assert!(ast.nodes().iter().any(|node| {
        matches!(
            &node.kind,
            SurfaceNodeKind::InfixExpression(operator)
                if operator.spelling.as_ref() == "**"
                    && operator.associativity == SurfaceOperatorAssociativity::Right
                    && direct_child_has_kind(&ast, node, |kind| {
                        matches!(kind, SurfaceNodeKind::SelectorAccess)
                    })
        )
    }));
}

#[test]
fn parser_diagnoses_non_associative_operator_terms() {
    let source_id = source_id(161);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("a", ParserTokenKind::Identifier),
            ("%%", ParserTokenKind::UserSymbol),
            ("b", ParserTokenKind::Identifier),
            ("%%", ParserTokenKind::UserSymbol),
            ("c", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let second_operator = tokens[8].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        operator_fixture_fixities(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::NonAssociativeOperatorChain
    );
    assert_eq!(output.diagnostics[0].primary, second_operator);
    let ast = output
        .ast
        .expect("non-associative chain should keep an AST");
    assert_eq!(
        ast.nodes()
            .iter()
            .filter(|node| {
                matches!(
                    &node.kind,
                    SurfaceNodeKind::InfixExpression(operator)
                        if operator.spelling.as_ref() == "%%"
                )
            })
            .count(),
        2
    );
}

#[test]
fn parser_diagnoses_dangling_infix_operator_terms_once() {
    let source_id = source_id(162);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("a", ParserTokenKind::Identifier),
            ("++", ParserTokenKind::UserSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let operator_range = tokens[6].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        operator_fixture_fixities(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::DanglingOperator
    );
    assert_eq!(output.diagnostics[0].primary, operator_range);
    let ast = output.ast.expect("dangling operator should keep an AST");
    single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TermExpression));
    assert!(
        !ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::InfixExpression(_)))
    );
}

#[test]
fn parser_diagnoses_dangling_prefix_operator_terms() {
    let source_id = source_id(163);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("~", ParserTokenKind::UserSymbol),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let operator_range = tokens[5].span;
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        operator_fixture_fixities(),
    ));

    assert_eq!(output.diagnostics.len(), 1);
    assert_eq!(
        output.diagnostics[0].code,
        SyntaxDiagnosticCode::DanglingOperator
    );
    assert_eq!(
        output.diagnostics[0].message.as_ref(),
        "operator has no operand"
    );
    assert_eq!(output.diagnostics[0].primary, operator_range);
    let ast = output.ast.expect("dangling prefix should keep an AST");
    let prefix = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::PrefixExpression(_))
    });
    assert!(direct_child_has_kind(&ast, prefix, |kind| {
        matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )
    }));
}

#[test]
fn parser_considers_same_spelling_infix_when_postfix_binding_is_too_low() {
    let source_id = source_id(164);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("a", ParserTokenKind::Identifier),
            ("++", ParserTokenKind::UserSymbol),
            ("b", ParserTokenKind::Identifier),
            ("@", ParserTokenKind::UserSymbol),
            ("c", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        vec![
            OperatorFixityEntry::infix("++", 10, OperatorAssociativity::Left),
            OperatorFixityEntry::postfix("@", 5),
            OperatorFixityEntry::infix("@", 20, OperatorAssociativity::Left),
        ],
    ));

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("same-spelling postfix/infix case should parse");
    assert!(ast.nodes().iter().any(|node| {
        matches!(
            &node.kind,
            SurfaceNodeKind::InfixExpression(operator)
                if operator.spelling.as_ref() == "@"
                    && operator.precedence == 20
                    && operator.associativity == SurfaceOperatorAssociativity::Left
        )
    }));
    assert!(!ast.nodes().iter().any(|node| {
        matches!(
            &node.kind,
            SurfaceNodeKind::PostfixExpression(operator)
                if operator.spelling.as_ref() == "@"
        )
    }));
}

#[test]
fn parser_prefers_same_spelling_infix_when_right_operand_is_present() {
    let source_id = source_id(165);
    let tokens = token_sequence(
        source_id,
        &[
            ("reserve", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("for", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("a", ParserTokenKind::Identifier),
            ("@", ParserTokenKind::UserSymbol),
            ("b", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        vec![
            OperatorFixityEntry::postfix("@", 90),
            OperatorFixityEntry::infix("@", 20, OperatorAssociativity::Left),
        ],
    ));

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("same-spelling infix with right operand should parse");
    assert!(ast.nodes().iter().any(|node| {
        matches!(
            &node.kind,
            SurfaceNodeKind::InfixExpression(operator)
                if operator.spelling.as_ref() == "@"
                    && operator.precedence == 20
                    && operator.associativity == SurfaceOperatorAssociativity::Left
        )
    }));
    assert!(!ast.nodes().iter().any(|node| {
        matches!(
            &node.kind,
            SurfaceNodeKind::PostfixExpression(operator)
                if operator.spelling.as_ref() == "@"
        )
    }));
}

#[test]
fn legacy_token_preservation_streams_without_item_starts_stay_diagnostic_free() {
    let source_id = source_id(44);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        vec![
            token(source_id, ParserTokenKind::Identifier, "alpha", 0, 5),
            token(source_id, ParserTokenKind::ReservedSymbol, ";", 5, 6),
            token(source_id, ParserTokenKind::Identifier, "beta", 7, 11),
        ],
        Vec::new(),
    ));

    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("legacy stream should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert!(item_list.children.is_empty());
}

#[test]
fn parser_parses_atomic_formula_placeholder_payloads() {
    let source_id = source_id(45);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("I", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("in", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("E", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("N", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("<>", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("BA", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("IN", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("not", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("P", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("divides", ParserTokenKind::UserSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("PL", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("divides", ParserTokenKind::UserSymbol),
                ("z", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("DN", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("not", ParserTokenKind::ReservedWord),
                ("divides", ParserTokenKind::UserSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("CH", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("divides", ParserTokenKind::UserSymbol),
                ("y", ParserTokenKind::Identifier),
                ("does", ParserTokenKind::ReservedWord),
                ("not", ParserTokenKind::ReservedWord),
                ("divides", ParserTokenKind::UserSymbol),
                ("z", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("lemma", ParserTokenKind::ReservedWord),
                ("L", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("C", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("IsSmall", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("atomic formulas should parse");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FormulaExpression
        )),
        13
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::BuiltinPredicateApplication)
        }),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::IsAssertion)),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AttributeTestChain
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::PredicateApplication)
        }),
        4
    );
    let predicate_segments = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::PredicateSegment))
        .collect::<Vec<_>>();
    assert_eq!(predicate_segments.len(), 5);
    assert!(
        predicate_segments.iter().any(|segment| {
            segment.children.iter().any(|child| {
                ast.node(*child)
                    .and_then(mizar_syntax::SurfaceNode::token_text)
                    == Some("does")
            })
        }),
        "predicate chain segment should preserve `does not` negation tokens"
    );
    assert!(
        predicate_segments.iter().any(|segment| {
            segment.children.iter().any(|child| {
                ast.node(*child)
                    .and_then(mizar_syntax::SurfaceNode::token_text)
                    == Some("do")
            })
        }),
        "predicate segment should preserve `do not` negation tokens"
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::InlinePredicateApplication)
        }),
        1
    );

    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    let first_item = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(first_item.kind, SurfaceNodeKind::TheoremItem));
    let formula_id = first_item
        .children
        .iter()
        .copied()
        .find(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::FormulaExpression))
        })
        .expect("theorem item should own a concrete FormulaExpression child");
    let formula = ast.node(formula_id).unwrap();
    assert_eq!(formula.children.len(), 1);
    assert!(matches!(
        ast.node(formula.children[0]).unwrap().kind,
        SurfaceNodeKind::BuiltinPredicateApplication
    ));
    assert!(
        item_list.children.iter().any(|item| {
            let Some(item) = ast.node(*item) else {
                return false;
            };
            item.children.iter().any(|child| {
                ast.node(*child)
                    .and_then(mizar_syntax::SurfaceNode::token_text)
                    == Some("lemma")
            }) && direct_child_has_kind(&ast, item, |kind| {
                matches!(kind, SurfaceNodeKind::FormulaExpression)
            })
        }),
        "lemma item should host a concrete formula payload"
    );
}

#[test]
fn parser_parses_formula_connectives_constants_and_grouping() {
    let source_id = source_id(49);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("Thesis", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Conj", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("&", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Repeat", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("&", ParserTokenKind::ReservedSymbol),
                ("...", ParserTokenKind::ReservedSymbol),
                ("&", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("OrImplies", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("or", ParserTokenKind::ReservedWord),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                ("implies", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("IffGroup", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("iff", ParserTokenKind::ReservedWord),
                ("contradiction", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("NotAnd", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("not", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("&", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("RepeatOr", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("or", ParserTokenKind::ReservedWord),
                ("...", ParserTokenKind::ReservedSymbol),
                ("or", ParserTokenKind::ReservedWord),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("AndLeft", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("&", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                ("&", ParserTokenKind::ReservedSymbol),
                ("u", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("v", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("OrLeft", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("or", ParserTokenKind::ReservedWord),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                ("or", ParserTokenKind::ReservedWord),
                ("u", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("v", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("ImpliesRight", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("implies", ParserTokenKind::ReservedWord),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                ("implies", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("AndBeforeOr", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("&", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                ("or", ParserTokenKind::ReservedWord),
                ("u", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("v", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("ImpliesBeforeIff", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("implies", ParserTokenKind::ReservedWord),
                ("contradiction", ParserTokenKind::ReservedWord),
                ("iff", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("ParenOverride", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("or", ParserTokenKind::ReservedWord),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("&", ParserTokenKind::ReservedSymbol),
                ("u", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("v", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("ImpliesQuantifier", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("implies", ParserTokenKind::ReservedWord),
                ("for", ParserTokenKind::ReservedWord),
                ("q", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("holds", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-14 formula forms should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("task-14 formulas should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FormulaExpression
        )),
        16
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::BinaryFormula(_)
        )),
        20
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PrefixFormula(_)
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::ParenthesizedFormula)
        }),
        2
    );
    for connective in [
        SurfaceFormulaConnective::And,
        SurfaceFormulaConnective::Or,
        SurfaceFormulaConnective::Implies,
        SurfaceFormulaConnective::Iff,
    ] {
        assert!(
            ast.nodes().iter().any(|node| {
                matches!(
                    node.kind,
                    SurfaceNodeKind::BinaryFormula(operator)
                        if operator.connective == connective
                )
            }),
            "formula AST should contain {connective:?}"
        );
    }
    assert!(ast.nodes().iter().any(|node| {
        matches!(
            node.kind,
            SurfaceNodeKind::BinaryFormula(operator)
                if operator.connective == SurfaceFormulaConnective::And && operator.repeated
        )
    }));
    assert!(ast.nodes().iter().any(|node| {
        matches!(
            node.kind,
            SurfaceNodeKind::BinaryFormula(operator)
                if operator.connective == SurfaceFormulaConnective::Or && operator.repeated
        )
    }));
    assert!(ast.nodes().iter().any(|node| {
        matches!(
            node.kind,
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis)
        )
    }));
    assert!(ast.nodes().iter().any(|node| {
        matches!(
            node.kind,
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction)
        )
    }));

    let or_implies = formula_root_for_label(&ast, "OrImplies");
    assert_binary_formula(or_implies, SurfaceFormulaConnective::Implies, false);
    let or_implies_children = structural_children(&ast, or_implies);
    assert_binary_formula(or_implies_children[0], SurfaceFormulaConnective::Or, false);
    assert_formula_constant(or_implies_children[1], SurfaceFormulaConstant::Thesis);

    let not_and = formula_root_for_label(&ast, "NotAnd");
    assert_binary_formula(not_and, SurfaceFormulaConnective::And, false);
    let not_and_children = structural_children(&ast, not_and);
    assert!(matches!(
        not_and_children[0].kind,
        SurfaceNodeKind::PrefixFormula(_)
    ));
    assert!(matches!(
        structural_children(&ast, not_and_children[0])[0].kind,
        SurfaceNodeKind::BuiltinPredicateApplication
    ));

    let iff_group = formula_root_for_label(&ast, "IffGroup");
    assert_binary_formula(iff_group, SurfaceFormulaConnective::Iff, false);
    let iff_children = structural_children(&ast, iff_group);
    assert!(matches!(
        iff_children[0].kind,
        SurfaceNodeKind::ParenthesizedFormula
    ));
    assert_formula_constant(iff_children[1], SurfaceFormulaConstant::Contradiction);

    let and_left = formula_root_for_label(&ast, "AndLeft");
    assert_binary_formula(and_left, SurfaceFormulaConnective::And, false);
    let and_children = structural_children(&ast, and_left);
    assert_binary_formula(and_children[0], SurfaceFormulaConnective::And, false);
    assert!(matches!(
        and_children[1].kind,
        SurfaceNodeKind::BuiltinPredicateApplication
    ));

    let or_left = formula_root_for_label(&ast, "OrLeft");
    assert_binary_formula(or_left, SurfaceFormulaConnective::Or, false);
    let or_children = structural_children(&ast, or_left);
    assert_binary_formula(or_children[0], SurfaceFormulaConnective::Or, false);
    assert!(matches!(
        or_children[1].kind,
        SurfaceNodeKind::BuiltinPredicateApplication
    ));

    let implies_right = formula_root_for_label(&ast, "ImpliesRight");
    assert_binary_formula(implies_right, SurfaceFormulaConnective::Implies, false);
    let implies_children = structural_children(&ast, implies_right);
    assert!(matches!(
        implies_children[0].kind,
        SurfaceNodeKind::BuiltinPredicateApplication
    ));
    assert_binary_formula(
        implies_children[1],
        SurfaceFormulaConnective::Implies,
        false,
    );

    let repeat = formula_root_for_label(&ast, "Repeat");
    assert_binary_formula(repeat, SurfaceFormulaConnective::And, true);
    assert_eq!(direct_token_texts(&ast, repeat), vec!["&", "...", "&"]);

    let repeat_or = formula_root_for_label(&ast, "RepeatOr");
    assert_binary_formula(repeat_or, SurfaceFormulaConnective::Or, true);
    assert_eq!(direct_token_texts(&ast, repeat_or), vec!["or", "...", "or"]);

    let and_before_or = formula_root_for_label(&ast, "AndBeforeOr");
    assert_binary_formula(and_before_or, SurfaceFormulaConnective::Or, false);
    let and_before_or_children = structural_children(&ast, and_before_or);
    assert_binary_formula(
        and_before_or_children[0],
        SurfaceFormulaConnective::And,
        false,
    );
    assert!(matches!(
        and_before_or_children[1].kind,
        SurfaceNodeKind::BuiltinPredicateApplication
    ));

    let implies_before_iff = formula_root_for_label(&ast, "ImpliesBeforeIff");
    assert_binary_formula(implies_before_iff, SurfaceFormulaConnective::Iff, false);
    let implies_before_iff_children = structural_children(&ast, implies_before_iff);
    assert_binary_formula(
        implies_before_iff_children[0],
        SurfaceFormulaConnective::Implies,
        false,
    );
    assert_formula_constant(
        implies_before_iff_children[1],
        SurfaceFormulaConstant::Thesis,
    );

    let paren_override = formula_root_for_label(&ast, "ParenOverride");
    assert_binary_formula(paren_override, SurfaceFormulaConnective::And, false);
    let paren_override_children = structural_children(&ast, paren_override);
    assert!(matches!(
        paren_override_children[0].kind,
        SurfaceNodeKind::ParenthesizedFormula
    ));
    let parenthesized_children = structural_children(&ast, paren_override_children[0]);
    assert_eq!(parenthesized_children.len(), 1);
    assert!(matches!(
        parenthesized_children[0].kind,
        SurfaceNodeKind::FormulaExpression
    ));
    assert_binary_formula(
        structural_children(&ast, parenthesized_children[0])[0],
        SurfaceFormulaConnective::Or,
        false,
    );
    assert!(matches!(
        paren_override_children[1].kind,
        SurfaceNodeKind::BuiltinPredicateApplication
    ));

    let implies_quantifier = formula_root_for_label(&ast, "ImpliesQuantifier");
    assert_binary_formula(implies_quantifier, SurfaceFormulaConnective::Implies, false);
    let implies_quantifier_children = structural_children(&ast, implies_quantifier);
    assert!(matches!(
        implies_quantifier_children[0].kind,
        SurfaceNodeKind::BuiltinPredicateApplication
    ));
    assert_quantified_formula(
        implies_quantifier_children[1],
        SurfaceQuantifierKind::Universal,
    );
}

#[test]
fn parser_parses_set_comprehension_terms() {
    let source_id = source_id(168);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("OmittedCondition", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Conditioned", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MultipleGenerators", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("p", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("OperatorMapper", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                ("++", ParserTokenKind::UserSymbol),
                ("b", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("q", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Nested", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("}", ParserTokenKind::ReservedSymbol),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Enumeration", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        operator_fixture_fixities(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "set-comprehension terms should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("set-comprehension terms should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::SetComprehension
        )),
        6
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComprehensionVariableSegment
        )),
        7
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SetEnumeration)),
        1
    );

    let omitted = set_comprehension_term_for_label(&ast, "OmittedCondition");
    assert_eq!(direct_token_texts(&ast, omitted), vec!["{", "where", "}"]);
    assert!(
        subtree_contains_kind(&ast, set_comprehension_mapper(&ast, omitted), |kind| {
            matches!(kind, SurfaceNodeKind::SelectorAccess)
        }),
        "selector mapper should preserve the existing term surface"
    );
    assert_eq!(
        structural_children(&ast, omitted)
            .iter()
            .filter(|child| matches!(child.kind, SurfaceNodeKind::ComprehensionVariableSegment))
            .count(),
        1
    );

    let conditioned = set_comprehension_term_for_label(&ast, "Conditioned");
    assert_eq!(
        direct_token_texts(&ast, conditioned),
        vec!["{", "where", ":", "}"]
    );
    assert!(
        structural_children(&ast, conditioned)
            .iter()
            .any(|child| matches!(child.kind, SurfaceNodeKind::FormulaExpression))
    );

    let multiple = set_comprehension_term_for_label(&ast, "MultipleGenerators");
    assert_eq!(
        structural_children(&ast, multiple)
            .iter()
            .filter(|child| matches!(child.kind, SurfaceNodeKind::ComprehensionVariableSegment))
            .count(),
        2
    );
    assert_eq!(
        direct_token_texts(&ast, multiple),
        vec!["{", "where", ",", ":", "}"]
    );

    let operator_mapper = set_comprehension_term_for_label(&ast, "OperatorMapper");
    assert!(
        subtree_contains_kind(
            &ast,
            set_comprehension_mapper(&ast, operator_mapper),
            |kind| {
                matches!(
                    kind,
                    SurfaceNodeKind::InfixExpression(operator)
                        if operator.spelling.as_ref() == "++"
                )
            }
        ),
        "operator mapper should preserve Pratt grouping inside SetComprehension"
    );

    let nested = set_comprehension_term_for_label(&ast, "Nested");
    assert!(
        subtree_contains_kind(&ast, set_comprehension_mapper(&ast, nested), |kind| {
            matches!(kind, SurfaceNodeKind::SetComprehension)
        }),
        "nested mapper should preserve the inner SetComprehension term"
    );

    let enumeration_formula = formula_root_for_label(&ast, "Enumeration");
    let enumeration_left = builtin_predicate_left_term_payload(&ast, enumeration_formula);
    assert!(matches!(
        enumeration_left.kind,
        SurfaceNodeKind::SetEnumeration
    ));
}

#[test]
fn parser_recovers_malformed_set_comprehensions() {
    let source_id = source_id(169);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingGeneratorType", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingCondition", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingClose", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("IsSmall", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingGenerator", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingIs", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("set", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| &diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            &SyntaxDiagnosticCode::MalformedTypeExpression,
            &SyntaxDiagnosticCode::MalformedFormulaExpression,
            &SyntaxDiagnosticCode::MalformedTermExpression,
            &SyntaxDiagnosticCode::MalformedTermExpression,
            &SyntaxDiagnosticCode::MalformedTermExpression,
        ]
    );
    let ast = output
        .ast
        .expect("malformed set comprehensions should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::SetComprehension
        )),
        5
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )),
        1
    );
}

#[test]
fn parser_parses_task31_template_predicate_args_inside_set_comprehension_conditions() {
    let source_id = source_id(170);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("TemplateCondition", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("NestedTemplateCondition", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("IsSmall", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "template predicate args inside comprehension conditions should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("template comprehension condition should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FormulaExpression
        )),
        4
    );
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
            SurfaceNodeKind::TemplateArguments
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0
    );
}

#[test]
fn parser_parses_task31_template_predicate_args() {
    let source_id = source_id(72);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("TemplateArgs", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("divides", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "template predicate args should parse as task-31 syntax: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("template args should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FormulaExpression
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateArguments
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0
    );
}

#[test]
fn parser_parses_task31_template_predicate_args_after_formula_boundaries() {
    let source_id = source_id(166);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("IsAssertionThenTemplate", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("&", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("QuantifierHoldsTemplate", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("st", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("holds", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "template predicate args past formula boundaries should parse: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("template args should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::FormulaExpression
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateArguments
        )),
        2
    );
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0
    );
}

#[test]
fn parser_parses_task31_template_parameters_and_template_only_blocks() {
    let source_id = source_id(181);
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
                ("OrdinaryLead", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("type", ParserTokenKind::ReservedWord),
                ("extends", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("let", ParserTokenKind::ReservedWord),
                ("P", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("pred", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("let", ParserTokenKind::ReservedWord),
                ("F", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("func", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("B", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("pred", ParserTokenKind::ReservedWord),
                ("UsesParams", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "template parameters should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("template parameter fixture should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateParameter
        )),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::DefinitionParameter
        )),
        0
    );
}

#[test]
fn parser_parses_task31_template_arguments_on_references_and_functors() {
    let source_id = source_id(182);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("ReferenceArgs", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Ref", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("mml", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("foo", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("Th", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("Q", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("mml", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("foo", ParserTokenKind::Identifier),
                (".{", ParserTokenKind::ReservedSymbol),
                ("G", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("}", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("FunctorArgs", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("g", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("NestedActual", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("h", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("f", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "template arguments on references and functors should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("reference/functor template args should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateArguments
        )),
        7
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateArgument
        )),
        7
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression)),
        2
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::Reference)
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TemplateArguments))
        }),
        "local references should own TemplateArguments directly"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::QualifiedReference)
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TemplateArguments))
        }),
        "qualified references should own TemplateArguments directly"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::GroupedReferenceItem)
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TemplateArguments))
        }),
        "grouped reference items should own TemplateArguments directly"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::TermReference)
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TemplateArguments))
        }),
        "template functor term references should own TemplateArguments before application args"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::TemplateArgument)
                && node_subtree_contains_kind(&ast, node, |kind| {
                    matches!(kind, SurfaceNodeKind::ApplicationTerm)
                })
        }),
        "template arguments should accept nested template functor applications"
    );
}

#[test]
fn parser_recovers_task31_malformed_template_arguments() {
    let source_id = source_id(183);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("TrailingArg", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("AttributeQua", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("q", ParserTokenKind::Identifier),
                ("qua", ParserTokenKind::ReservedWord),
                ("non", ParserTokenKind::ReservedWord),
                ("empty", ParserTokenKind::UserSymbol),
                ("T", ParserTokenKind::UserSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Unclosed", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("R", ParserTokenKind::UserSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::UserSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
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
            SyntaxDiagnosticCode::MalformedTypeExpression,
            SyntaxDiagnosticCode::MalformedTypeExpression,
            SyntaxDiagnosticCode::MalformedTypeExpression,
        ],
        "malformed template args should report exact type diagnostics"
    );
    let ast = output
        .ast
        .expect("malformed template args should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateArguments
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateArgument
        )),
        5
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::TemplateArguments)
                && !subtree_token_texts(&ast, node)
                    .iter()
                    .any(|token| token == "]")
        }),
        "unclosed template argument lists should recover as TemplateArguments"
    );
}

#[test]
fn parser_parses_task32_algorithm_and_claim_surfaces() {
    let source_id = source_id(185);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("public", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("f", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("var", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("const", ParserTokenKind::ReservedWord),
                ("c", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("ghost", ParserTokenKind::ReservedWord),
                ("var", ParserTokenKind::ReservedWord),
                ("seen", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                ("c", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("ghost", ParserTokenKind::ReservedWord),
                ("const", ParserTokenKind::ReservedWord),
                ("witness", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                ("seen", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                ("c", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("field", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                ("c", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("ghost", ParserTokenKind::ReservedWord),
                ("seen", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("snapshot", ParserTokenKind::ReservedWord),
                ("Init", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("return", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("Ret", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("log", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("do", ParserTokenKind::ReservedWord),
                ("return", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("claim", ParserTokenKind::ReservedWord),
                ("f", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("InitState", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("lemma", ParserTokenKind::ReservedWord),
                ("Progress", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-32 algorithm and claim surfaces should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-32 algorithm and claim surfaces should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmDefinition
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmParameters
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TemplateLoci)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::TemplateArguments
        )),
        0,
        "algorithm definition schema params use TemplateLoci, not call-site TemplateArguments"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::VariableDeclaration
        )),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::VariableBinding
        )),
        5
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssignmentStatement
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::Lvalue)),
        3
    );
    assert!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::Lvalue))
            .any(|node| subtree_token_texts(&ast, node) == ["y", ".", "field"]),
        "dotted assignment targets should be preserved inside Lvalue"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::SnapshotStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ReturnStatement
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ClaimBlockItem)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::VisibleItem)),
        2,
        "definition-local visibility wraps both algorithm definitions"
    );
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
fn parser_recovers_task32_algorithm_gaps() {
    let source_id = source_id(186);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("Broken", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("]", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("do", ParserTokenKind::ReservedWord),
                ("claim", ParserTokenKind::ReservedWord),
                ("Inside", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("theorem", ParserTokenKind::ReservedWord),
                ("BadPlace", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("+", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("const", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("ghost", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("snapshot", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("return", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (":=", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("claim", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("Bad", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("@[", ParserTokenKind::ReservedSymbol),
                ("label", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Deferred", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
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
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }),
        "malformed claim contents should report formula diagnostics"
    );
    let ast = output
        .ast
        .expect("malformed task-32 algorithm surfaces should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmDefinition
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::VariableDeclaration
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssignmentStatement
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::SnapshotStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ReturnStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ClaimBlockItem)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem)),
        1,
        "task-35 claim-local theorem annotations should preserve theorem nodes"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )),
        1
    );
}

#[test]
fn parser_recovers_task32_algorithm_header_at_eof() {
    let source_id = source_id(199);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("Broken", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
    );
    let ast = output
        .ast
        .expect("algorithm header ending at EOF should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmDefinition
        )),
        1
    );
}

#[test]
fn parser_parses_task33_algorithm_control_flow_surfaces() {
    let source_id = source_id(200);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("flow", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("do", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("else", ParserTokenKind::ReservedWord),
                ("if", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("continue", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("while", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("i", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("a", ParserTokenKind::Identifier),
                ("to", ParserTokenKind::ReservedWord),
                ("b", ParserTokenKind::Identifier),
                ("step", ParserTokenKind::ReservedWord),
                ("s", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("continue", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("j", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("b", ParserTokenKind::Identifier),
                ("downto", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("in", ParserTokenKind::ReservedWord),
                ("S", ParserTokenKind::Identifier),
                ("processed", ParserTokenKind::ReservedWord),
                ("Seen", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("plain", ParserTokenKind::Identifier),
                ("in", ParserTokenKind::ReservedWord),
                ("Items", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
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
                ("continue", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("case", ParserTokenKind::ReservedWord),
                ("p2", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
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
                ("match", ParserTokenKind::ReservedWord),
                ("u", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("case", ParserTokenKind::ReservedWord),
                ("q", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("exhaustive", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Exhaustive", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("match", ParserTokenKind::ReservedWord),
                ("v", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("case", ParserTokenKind::ReservedWord),
                ("r", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("continue", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("exhaustive", ParserTokenKind::ReservedWord),
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
        output.diagnostics.is_empty(),
        "task-33 algorithm control flow should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-33 algorithm control flow should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::IfStatement)),
        2,
        "else-if should preserve a nested IfStatement"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::WhileStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ForRangeStatement
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ForCollectionStatement
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::MatchStatement)),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::MatchCase)),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::MatchEnding)),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::BreakStatement)),
        7
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ContinueStatement
        )),
        5
    );
    assert_task33_control_flow_accessors_and_child_order(&ast);
}

#[test]
fn parser_recovers_task33_algorithm_control_flow_gaps() {
    let source_id = source_id(201);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("broken_flow", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("do", ParserTokenKind::ReservedWord),
                ("while", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("if", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("continue", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("else", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("i", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("do", ParserTokenKind::ReservedWord),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("match", ParserTokenKind::ReservedWord),
                ("t", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("exhaustive", ParserTokenKind::ReservedWord),
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
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        }),
        "malformed if condition should diagnose: {:?}",
        output.diagnostics
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression),
        "malformed for loop and missing match case should diagnose terms/statements"
    );
    let ast = output
        .ast
        .expect("malformed task-33 control flow should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::WhileStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::IfStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ForRangeStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ForCollectionStatement
        )),
        0
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::MatchStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::MatchEnding)),
        1
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )) >= 1,
        "malformed if condition should insert MissingFormula"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStatement)
        )) >= 1,
        "match without a case should insert MissingStatement"
    );
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
            ("exhaustive", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
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
            nth_token_range(&tokens, "exhaustive", 0),
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
            ("exhaustive", ParserTokenKind::ReservedWord),
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
            ("exhaustive", ParserTokenKind::ReservedWord),
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
    assert_task33_algorithm_body_diagnostics_exact(
        source_id(220),
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
        &[ExpectedParserDiagnostic::formula(
            "expected `otherwise` or `exhaustive` match ending",
            "end",
            1,
        )],
    );
}

#[test]
fn parser_parses_task34_algorithm_verification_surfaces() {
    let source_id = source_id(230);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("public", ParserTokenKind::ReservedWord),
                ("terminating", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("verified", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("n", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("requires", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("ensures", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("decreasing", ParserTokenKind::ReservedWord),
                ("n", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("measure", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("while", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("invariant", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("Inv", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("decreasing", ParserTokenKind::ReservedWord),
                ("n", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("measure", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("Dec", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("assert", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("AssertRef", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("i", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("lower", ParserTokenKind::Identifier),
                ("to", ParserTokenKind::ReservedWord),
                ("upper", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("invariant", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("RangeInv", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("continue", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("item", ParserTokenKind::Identifier),
                ("in", ParserTokenKind::ReservedWord),
                ("Items", ParserTokenKind::Identifier),
                ("do", ParserTokenKind::ReservedWord),
                ("invariant", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("terminating", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("bare", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("do", ParserTokenKind::ReservedWord),
                ("assert", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("private", ParserTokenKind::ReservedWord),
                ("terminating", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("hidden", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("do", ParserTokenKind::ReservedWord),
                ("assert", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
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
        output.diagnostics.is_empty(),
        "task-34 algorithm verification clauses should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-34 algorithm verification clauses should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmDefinition
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmTerminationClause
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmRequiresClause
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmEnsuresClause
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmDecreasingClause
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::LoopInvariantClause
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::LoopDecreasingClause
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssertStatement
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TermList)),
        2
    );
    assert_task34_verification_accessors_and_child_order(&ast);
}

#[test]
fn parser_recovers_task34_algorithm_verification_gaps() {
    let source_id = source_id(231);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("definition", ParserTokenKind::ReservedWord),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("dup_contract", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("requires", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("requires", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("assert", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("invariant", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("out_of_order", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("ensures", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("requires", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
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
                ("algorithm", ParserTokenKind::ReservedWord),
                ("missing_formulas", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("requires", ParserTokenKind::ReservedWord),
                ("ensures", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("while", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("do", ParserTokenKind::ReservedWord),
                ("decreasing", ParserTokenKind::ReservedWord),
                ("first", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("break", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("algorithm", ParserTokenKind::ReservedWord),
                ("loop_gaps", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("decreasing", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("measure", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("do", ParserTokenKind::ReservedWord),
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
                ("invariant", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("continue", ParserTokenKind::ReservedWord),
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
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.message.as_ref() == "duplicate or out-of-order algorithm verification clause"
        }),
        "duplicate/out-of-order header clauses should diagnose: {:?}",
        output.diagnostics
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.as_ref() == "expected formula after `requires`"),
        "requires without a formula should diagnose"
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.as_ref() == "expected formula after `ensures`"),
        "ensures without a formula should diagnose"
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.as_ref() == "expected assertion formula"),
        "assert without a formula should diagnose"
    );
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.message.as_ref()
                == "range and collection loops accept only invariant verification clauses"
        }),
        "for-decreasing should diagnose"
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.as_ref() == "expected invariant formula"),
        "empty invariant should diagnose"
    );
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.message.as_ref() == "expected decreasing measure before `,`"
        }),
        "empty decreasing term-list slot should diagnose"
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.as_ref() == "expected term after `,`"),
        "dangling decreasing term-list comma should diagnose"
    );
    let ast = output
        .ast
        .expect("task-34 verification recovery should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmDefinition
        )),
        4
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )) >= 2
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )) >= 2
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 2
    );

    let recovered_term_list_signatures =
        surface_views(&ast, |kind| matches!(kind, SurfaceNodeKind::TermList))
            .into_iter()
            .map(direct_child_signature)
            .collect::<Vec<_>>();
    assert_contains_signature(
        &recovered_term_list_signatures,
        &["MissingTerm", ",", "TermExpression", ",", "MissingTerm"],
    );
    assert_contains_signature(
        &recovered_term_list_signatures,
        &["TermExpression", ",", "MissingTerm"],
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

fn assert_task33_control_flow_accessors_and_child_order(ast: &mizar_syntax::SurfaceAst) {
    let snapshot = ast.snapshot_text();
    for node_name in [
        "IfStatement",
        "WhileStatement",
        "ForRangeStatement",
        "ForCollectionStatement",
        "MatchStatement",
        "MatchCase",
        "MatchEnding",
        "BreakStatement",
        "ContinueStatement",
    ] {
        assert!(
            snapshot.contains(node_name),
            "parser-produced snapshot should include {node_name}"
        );
    }

    let if_views = surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::IfStatement));
    assert!(
        if_views
            .iter()
            .all(|view| (*view).as_if_statement().is_some())
    );
    let outer_if = if_views
        .iter()
        .copied()
        .find(|view| direct_structural_child_kind_names(*view).contains(&"IfStatement"))
        .expect("else-if should be owned as a nested IfStatement child");
    assert_eq!(
        direct_structural_child_kind_names(outer_if),
        vec!["FormulaExpression", "AlgorithmStatementList", "IfStatement"]
    );

    let while_views = surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::WhileStatement));
    assert!(
        while_views
            .iter()
            .all(|view| (*view).as_while_statement().is_some())
    );
    let while_signatures = while_views
        .into_iter()
        .map(direct_child_signature)
        .collect::<Vec<_>>();
    assert_contains_signature(
        &while_signatures,
        &[
            "while",
            "FormulaExpression",
            "do",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );

    let range_views = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::ForRangeStatement)
    });
    assert!(
        range_views
            .iter()
            .all(|view| (*view).as_for_range_statement().is_some())
    );
    let range_signatures = range_views
        .into_iter()
        .map(direct_child_signature)
        .collect::<Vec<_>>();
    assert_contains_signature(
        &range_signatures,
        &[
            "for",
            "i",
            "=",
            "TermExpression",
            "to",
            "TermExpression",
            "step",
            "TermExpression",
            "do",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );
    assert_contains_signature(
        &range_signatures,
        &[
            "for",
            "j",
            "=",
            "TermExpression",
            "downto",
            "TermExpression",
            "do",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );

    let collection_views = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::ForCollectionStatement)
    });
    assert!(
        collection_views
            .iter()
            .all(|view| (*view).as_for_collection_statement().is_some())
    );
    let collection_signatures = collection_views
        .into_iter()
        .map(direct_child_signature)
        .collect::<Vec<_>>();
    assert_contains_signature(
        &collection_signatures,
        &[
            "for",
            "x",
            "in",
            "TermExpression",
            "processed",
            "Seen",
            "do",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );
    assert_contains_signature(
        &collection_signatures,
        &[
            "for",
            "plain",
            "in",
            "TermExpression",
            "do",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );
    assert!(
        surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::BreakStatement))
            .iter()
            .all(|view| (*view).as_break_statement().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ContinueStatement
        ))
        .iter()
        .all(|view| (*view).as_continue_statement().is_some())
    );

    let match_views = surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::MatchStatement));
    assert!(
        match_views
            .iter()
            .all(|view| (*view).as_match_statement().is_some())
    );
    let multi_case_match = match_views
        .iter()
        .copied()
        .find(|view| {
            view.child_views()
                .filter(|child| child.as_match_case().is_some())
                .count()
                == 2
        })
        .expect("first match should own both case branches before its ending");
    assert_eq!(
        direct_structural_child_kind_names(multi_case_match),
        vec!["TermExpression", "MatchCase", "MatchCase", "MatchEnding"]
    );

    let ending_views = surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::MatchEnding));
    assert!(
        ending_views
            .iter()
            .all(|view| (*view).as_match_ending().is_some())
    );
    let otherwise_ending = ending_views
        .iter()
        .copied()
        .find(|view| view_starts_with_token(*view, "otherwise"))
        .expect("otherwise ending should be present");
    assert_eq!(
        direct_structural_child_kind_names(otherwise_ending),
        vec!["AlgorithmStatementList"]
    );

    let exhaustive_endings = ending_views
        .iter()
        .copied()
        .filter(|view| view_starts_with_token(*view, "exhaustive"))
        .collect::<Vec<_>>();
    assert_eq!(exhaustive_endings.len(), 2);
    assert!(
        exhaustive_endings
            .iter()
            .any(|view| direct_structural_child_kind_names(*view) == vec!["JustificationClause"])
    );
    assert!(
        exhaustive_endings
            .iter()
            .any(|view| direct_structural_child_kind_names(*view).is_empty())
    );
}

fn assert_task34_verification_accessors_and_child_order(ast: &mizar_syntax::SurfaceAst) {
    let snapshot = ast.snapshot_text();
    for node_name in [
        "AlgorithmTerminationClause",
        "AlgorithmRequiresClause",
        "AlgorithmEnsuresClause",
        "AlgorithmDecreasingClause",
        "LoopInvariantClause",
        "LoopDecreasingClause",
        "AssertStatement",
        "TermList",
    ] {
        assert!(
            snapshot.contains(node_name),
            "parser-produced snapshot should include {node_name}"
        );
    }

    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmTerminationClause
        ))
        .iter()
        .all(|view| (*view).as_algorithm_termination_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmRequiresClause
        ))
        .iter()
        .all(|view| (*view).as_algorithm_requires_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmEnsuresClause
        ))
        .iter()
        .all(|view| (*view).as_algorithm_ensures_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AlgorithmDecreasingClause
        ))
        .iter()
        .all(|view| (*view).as_algorithm_decreasing_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::LoopInvariantClause
        ))
        .iter()
        .all(|view| (*view).as_loop_invariant_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(
            kind,
            SurfaceNodeKind::LoopDecreasingClause
        ))
        .iter()
        .all(|view| (*view).as_loop_decreasing_clause().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::AssertStatement))
            .iter()
            .all(|view| (*view).as_assert_statement().is_some())
    );
    assert!(
        surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::TermList))
            .iter()
            .all(|view| (*view).as_term_list().is_some())
    );

    let algorithm_signatures = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::AlgorithmDefinition)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &algorithm_signatures,
        &[
            "AlgorithmTerminationClause",
            "algorithm",
            "verified",
            "AlgorithmParameters",
            "->",
            "TypeExpression",
            "AlgorithmRequiresClause",
            "AlgorithmEnsuresClause",
            "AlgorithmDecreasingClause",
            "AlgorithmBody",
            ";",
        ],
    );
    assert_contains_signature(
        &algorithm_signatures,
        &[
            "AlgorithmTerminationClause",
            "algorithm",
            "bare",
            "AlgorithmParameters",
            "AlgorithmBody",
            ";",
        ],
    );

    let term_list_signatures = surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::TermList))
        .into_iter()
        .map(direct_child_signature)
        .collect::<Vec<_>>();
    assert_contains_signature(
        &term_list_signatures,
        &["TermExpression", ",", "TermExpression"],
    );

    let invariant_signatures = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::LoopInvariantClause)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &invariant_signatures,
        &["invariant", "FormulaExpression", "JustificationClause", ";"],
    );
    assert_contains_signature(
        &invariant_signatures,
        &["invariant", "FormulaExpression", ";"],
    );

    let decreasing_signatures = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::LoopDecreasingClause)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &decreasing_signatures,
        &["decreasing", "TermList", "JustificationClause", ";"],
    );

    let while_signatures =
        surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::WhileStatement))
            .into_iter()
            .map(direct_child_signature)
            .collect::<Vec<_>>();
    assert_contains_signature(
        &while_signatures,
        &[
            "while",
            "FormulaExpression",
            "do",
            "LoopInvariantClause",
            "LoopDecreasingClause",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );

    let range_signatures = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::ForRangeStatement)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &range_signatures,
        &[
            "for",
            "i",
            "=",
            "TermExpression",
            "to",
            "TermExpression",
            "do",
            "LoopInvariantClause",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );

    let collection_signatures = surface_views(ast, |kind| {
        matches!(kind, SurfaceNodeKind::ForCollectionStatement)
    })
    .into_iter()
    .map(direct_child_signature)
    .collect::<Vec<_>>();
    assert_contains_signature(
        &collection_signatures,
        &[
            "for",
            "item",
            "in",
            "TermExpression",
            "do",
            "LoopInvariantClause",
            "AlgorithmStatementList",
            "end",
            ";",
        ],
    );

    let assert_signatures =
        surface_views(ast, |kind| matches!(kind, SurfaceNodeKind::AssertStatement))
            .into_iter()
            .map(direct_child_signature)
            .collect::<Vec<_>>();
    assert_contains_signature(
        &assert_signatures,
        &["assert", "FormulaExpression", "JustificationClause", ";"],
    );
    assert_contains_signature(&assert_signatures, &["assert", "FormulaExpression", ";"]);
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

#[test]
fn parser_parses_formula_quantifiers_and_variable_segments() {
    let source_id = source_id(73);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("U", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("st", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("holds", ParserTokenKind::ReservedWord),
                ("contradiction", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Nested", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("holds", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("E", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("ex", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("st", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Implicit", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("holds", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("ExplicitImplicit", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("holds", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("Recursive", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("st", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("or", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("holds", ParserTokenKind::ReservedWord),
                ("not", ParserTokenKind::ReservedWord),
                ("contradiction", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("ExistentialRecursive", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("ex", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("st", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("&", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "quantified formulas should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("quantified formulas should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::QuantifiedFormula(_)
        )),
        8
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::QuantifierVariableSegment
        )),
        9
    );
    assert!(ast.nodes().iter().any(|node| {
        matches!(
            node.kind,
            SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Universal)
        )
    }));
    assert!(ast.nodes().iter().any(|node| {
        matches!(
            node.kind,
            SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Existential)
        )
    }));
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeExpression)),
        2
    );
    assert!(
        ast.nodes()
            .iter()
            .filter(|node| matches!(node.kind, SurfaceNodeKind::QuantifierVariableSegment))
            .any(|segment| {
                segment
                    .children
                    .iter()
                    .filter_map(|id| ast.node(*id))
                    .any(|child| child.token_text() == Some(","))
            }),
        "implicit variable segment should preserve comma-separated variables"
    );

    let universal = formula_root_for_label(&ast, "U");
    assert_quantified_formula(universal, SurfaceQuantifierKind::Universal);
    assert_eq!(
        direct_token_texts(&ast, universal),
        vec!["for", "st", "holds"]
    );
    let universal_children = structural_children(&ast, universal);
    assert_eq!(universal_children.len(), 3);
    assert!(matches!(
        universal_children[0].kind,
        SurfaceNodeKind::QuantifierVariableSegment
    ));
    assert_eq!(
        direct_token_texts(&ast, universal_children[0]),
        vec!["x", "being"]
    );
    assert!(
        structural_children(&ast, universal_children[0])
            .iter()
            .any(|child| matches!(child.kind, SurfaceNodeKind::TypeExpression))
    );
    assert_formula_constant(universal_children[1], SurfaceFormulaConstant::Thesis);
    assert_formula_constant(universal_children[2], SurfaceFormulaConstant::Contradiction);

    let nested_outer = formula_root_for_label(&ast, "Nested");
    assert_quantified_formula(nested_outer, SurfaceQuantifierKind::Universal);
    assert_eq!(direct_token_texts(&ast, nested_outer), vec!["for"]);
    let nested_outer_children = structural_children(&ast, nested_outer);
    assert_eq!(nested_outer_children.len(), 2);
    assert_quantified_formula(nested_outer_children[1], SurfaceQuantifierKind::Universal);
    assert_eq!(
        direct_token_texts(&ast, nested_outer_children[1]),
        vec!["for", "holds"]
    );

    let existential = formula_root_for_label(&ast, "E");
    assert_quantified_formula(existential, SurfaceQuantifierKind::Existential);
    assert_eq!(direct_token_texts(&ast, existential), vec!["ex", "st"]);
    let existential_children = structural_children(&ast, existential);
    assert_eq!(existential_children.len(), 2);
    assert!(matches!(
        existential_children[0].kind,
        SurfaceNodeKind::QuantifierVariableSegment
    ));
    assert_formula_constant(existential_children[1], SurfaceFormulaConstant::Thesis);

    let implicit = formula_root_for_label(&ast, "Implicit");
    assert_quantified_formula(implicit, SurfaceQuantifierKind::Universal);
    assert_eq!(direct_token_texts(&ast, implicit), vec!["for", "holds"]);
    let implicit_children = structural_children(&ast, implicit);
    assert_eq!(
        direct_token_texts(&ast, implicit_children[0]),
        vec!["x", ",", "y"]
    );

    let explicit_implicit = formula_root_for_label(&ast, "ExplicitImplicit");
    assert_quantified_formula(explicit_implicit, SurfaceQuantifierKind::Universal);
    assert_eq!(
        direct_token_texts(&ast, explicit_implicit),
        vec!["for", ",", "holds"]
    );
    let explicit_implicit_children = structural_children(&ast, explicit_implicit);
    assert_eq!(explicit_implicit_children.len(), 3);
    assert_eq!(
        direct_token_texts(&ast, explicit_implicit_children[0]),
        vec!["x", "being"]
    );
    assert!(
        structural_children(&ast, explicit_implicit_children[0])
            .iter()
            .any(|child| matches!(child.kind, SurfaceNodeKind::TypeExpression))
    );
    assert_eq!(
        direct_token_texts(&ast, explicit_implicit_children[1]),
        vec!["y"]
    );
    assert!(structural_children(&ast, explicit_implicit_children[1]).is_empty());
    assert_formula_constant(
        explicit_implicit_children[2],
        SurfaceFormulaConstant::Thesis,
    );

    let recursive = formula_root_for_label(&ast, "Recursive");
    assert_quantified_formula(recursive, SurfaceQuantifierKind::Universal);
    assert_eq!(
        direct_token_texts(&ast, recursive),
        vec!["for", "st", "holds"]
    );
    let recursive_children = structural_children(&ast, recursive);
    assert_eq!(recursive_children.len(), 3);
    assert_binary_formula(recursive_children[1], SurfaceFormulaConnective::Or, false);
    assert!(matches!(
        recursive_children[2].kind,
        SurfaceNodeKind::PrefixFormula(_)
    ));

    let existential_recursive = formula_root_for_label(&ast, "ExistentialRecursive");
    assert_quantified_formula(existential_recursive, SurfaceQuantifierKind::Existential);
    assert_eq!(
        direct_token_texts(&ast, existential_recursive),
        vec!["ex", "st"]
    );
    let existential_recursive_children = structural_children(&ast, existential_recursive);
    assert_eq!(existential_recursive_children.len(), 2);
    assert_binary_formula(
        existential_recursive_children[1],
        SurfaceFormulaConnective::And,
        false,
    );
}

#[test]
fn parser_parses_task16_simple_statement_nodes() {
    let source_id = source_id(76);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("reserve", ParserTokenKind::ReservedWord),
                ("r", ParserTokenKind::Identifier),
                ("for", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("assume", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("and", ParserTokenKind::ReservedWord),
                ("contradiction", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("assume", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("given", ParserTokenKind::ReservedWord),
                ("z", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("take", ParserTokenKind::ReservedWord),
                ("a", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("f", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("g", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "simple statements should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("simple statements should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 7);
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ReserveItem)),
        1,
        "top-level reserve should keep the existing ReserveItem path"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StatementItem)),
        6
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::LetStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::QualifiedVariableSegment
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssumptionStatement
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ConditionList)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::Proposition)),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::GivenStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TakeStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::Witness)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SetStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::Equating)),
        2
    );

    let let_statement = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::LetStatement));
    assert_eq!(direct_token_texts(&ast, let_statement), vec!["let", ";"]);
    let let_segment = structural_children(&ast, let_statement)
        .into_iter()
        .find(|child| matches!(child.kind, SurfaceNodeKind::QualifiedVariableSegment))
        .expect("let should own a qualified variable segment");
    assert_eq!(
        direct_token_texts(&ast, let_segment),
        vec!["x", ",", "y", "be"]
    );

    let assumption_conditions =
        single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ConditionList));
    assert_eq!(
        direct_token_texts(&ast, assumption_conditions),
        vec!["that", "and"]
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::Proposition)
                && direct_token_texts(&ast, node) == vec!["A", ":"]
        }),
        "statement propositions should preserve optional label tokens"
    );

    let set_statement = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::SetStatement));
    assert_eq!(
        direct_token_texts(&ast, set_statement),
        vec!["set", ",", ";"]
    );
    let equatings = structural_children(&ast, set_statement)
        .into_iter()
        .filter(|child| matches!(child.kind, SurfaceNodeKind::Equating))
        .collect::<Vec<_>>();
    assert_eq!(equatings.len(), 2);
    assert_eq!(direct_token_texts(&ast, equatings[0]), vec!["f", "="]);
    assert_eq!(direct_token_texts(&ast, equatings[1]), vec!["g", "="]);
}

#[test]
fn parser_recovers_task16_simple_statement_gaps() {
    let source_id = source_id(77);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("assume", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("take", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("=", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("f", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("take", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("assume", ParserTokenKind::ReservedWord),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
    );
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
    );
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
    );
    let ast = output.ast.expect("statement recovery should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StatementItem)),
        7
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )
        }),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )
        }),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )
        }),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        }),
        1
    );
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
    assert_eq!(
        ast.trivia().skipped_token_ranges()[0].reason,
        SkippedTokenReason::Recovery
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
fn parser_parses_task17_justification_nodes() {
    let source_id = source_id(79);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("mml", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("foo", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("Th1", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("mml", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("foo", ParserTokenKind::Identifier),
                (".{", ParserTokenKind::ReservedSymbol),
                ("G1", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("G2", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("mml", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("foo", ParserTokenKind::Identifier),
                (".*", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("10", ParserTokenKind::Numeral),
                (",", ParserTokenKind::ReservedSymbol),
                ("timeout", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("20", ParserTokenKind::Numeral),
                (",", ParserTokenKind::ReservedSymbol),
                ("nest", ParserTokenKind::ReservedWord),
                (":", ParserTokenKind::ReservedSymbol),
                ("3", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-17 justifications should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("justification statements should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 4);
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StatementItem)),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::LetStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CompactStatement
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::JustificationClause
        )),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ReferenceList)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::Reference)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::QualifiedReference
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::GroupedReference
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::GroupedReferenceItem
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::BulkReference)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationOption
        )),
        3
    );

    let let_statement = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::LetStatement));
    assert!(direct_child_has_kind(&ast, let_statement, |kind| {
        matches!(kind, SurfaceNodeKind::JustificationClause)
    }));
    let grouped = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::GroupedReference)
    });
    assert_eq!(direct_token_texts(&ast, grouped), vec![".{", ",", "}"]);
    let computation = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::ComputationJustification)
    });
    assert_eq!(
        direct_token_texts(&ast, computation),
        vec!["computation", "(", ",", ",", ")"]
    );
}

#[test]
fn parser_recovers_task17_justification_gaps() {
    let source_id = source_id(80);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                ("[", ParserTokenKind::ReservedSymbol),
                ("T", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("B", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("mml", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("foo", ParserTokenKind::Identifier),
                (".{", ParserTokenKind::ReservedSymbol),
                ("G1", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("}", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("mml", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("foo", ParserTokenKind::Identifier),
                (".{", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("G1", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("mml", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("foo", ParserTokenKind::Identifier),
                (".{", ParserTokenKind::ReservedSymbol),
                ("G1", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("G2", ParserTokenKind::Identifier),
                ("}", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("mml", ParserTokenKind::Identifier),
                (".", ParserTokenKind::ReservedSymbol),
                ("foo", ParserTokenKind::Identifier),
                (".{", ParserTokenKind::ReservedSymbol),
                ("G1", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (",", ParserTokenKind::ReservedSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("timeout", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("2", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification),
        "only justification diagnostics should be reported: {:?}",
        output.diagnostics
    );
    assert!(
        output.diagnostics.len() >= 5,
        "expected each malformed justification shape to diagnose: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("malformed justifications should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
        )),
        10
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )) >= 1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::JustificationClause
        )),
        12
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        3,
        "`let ... by computation` should recover as malformed references, not computation"
    );
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
    assert!(
        ast.trivia()
            .skipped_token_ranges()
            .iter()
            .all(|range| range.reason == SkippedTokenReason::Recovery)
    );
}

#[test]
fn parser_recovers_let_by_before_following_statement() {
    let source_id = source_id(90);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                ("assume", ParserTokenKind::ReservedWord),
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
            .map(|diagnostic| &diagnostic.code)
            .collect::<Vec<_>>(),
        vec![&SyntaxDiagnosticCode::MissingSemicolon]
    );
    let ast = output.ast.expect("let-by recovery should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 2);
    let let_item = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(let_item.kind, SurfaceNodeKind::StatementItem));
    let assumption_item = ast.node(item_list.children[1]).unwrap();
    assert!(matches!(
        assumption_item.kind,
        SurfaceNodeKind::StatementItem
    ));
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::LetStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::JustificationClause
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssumptionStatement
        )),
        1
    );
}

#[test]
fn parser_does_not_treat_later_set_statement_as_let_type_word() {
    let source_id = source_id(81);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("f", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                ("assume", ParserTokenKind::ReservedWord),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon),
        "missing semicolon before the second `set` should be diagnosed: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("set-statement boundary recovery should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 3);
    let let_item = ast.node(item_list.children[0]).unwrap();
    assert!(matches!(let_item.kind, SurfaceNodeKind::StatementItem));
    let set_item = ast.node(item_list.children[1]).unwrap();
    assert!(matches!(set_item.kind, SurfaceNodeKind::StatementItem));
    let assumption_item = ast.node(item_list.children[2]).unwrap();
    assert!(matches!(
        assumption_item.kind,
        SurfaceNodeKind::StatementItem
    ));
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SetStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssumptionStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0
    );
}

#[test]
fn parser_diagnoses_non_associative_formula_iff_chain() {
    let source_id = source_id(74);
    let tokens = token_sequence(
        source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("IffChain", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            ("iff", ParserTokenKind::ReservedWord),
            ("contradiction", ParserTokenKind::ReservedWord),
            ("iff", ParserTokenKind::ReservedWord),
            ("thesis", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let second_iff = tokens[6].span;
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
            .map(|diagnostic| &diagnostic.code)
            .collect::<Vec<_>>(),
        vec![&SyntaxDiagnosticCode::NonAssociativeOperatorChain]
    );
    assert_eq!(output.diagnostics[0].primary, second_iff);
    let ast = output.ast.expect("iff chain should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::BinaryFormula(operator)
                if operator.connective == SurfaceFormulaConnective::Iff
        )),
        2
    );
}

#[test]
fn parser_recovers_missing_formula_operands_and_unmatched_formula_grouping() {
    let source_id = source_id(75);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingNot", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("not", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingConnective", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("&", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingSt", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("ex", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("st", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingHolds", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("holds", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingHoldsKeyword", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingStKeyword", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("ex", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("MissingParen", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| &diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            &SyntaxDiagnosticCode::MalformedFormulaExpression,
            &SyntaxDiagnosticCode::MalformedFormulaExpression,
            &SyntaxDiagnosticCode::MalformedFormulaExpression,
            &SyntaxDiagnosticCode::MalformedFormulaExpression,
            &SyntaxDiagnosticCode::MalformedFormulaExpression,
            &SyntaxDiagnosticCode::MalformedFormulaExpression,
            &SyntaxDiagnosticCode::MalformedFormulaExpression,
        ]
    );
    let ast = output.ast.expect("formula recovery should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
            )
        }),
        6
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
            )
        }),
        1
    );
}

#[test]
fn parser_parses_theorem_tails_after_formulas_as_concrete_items() {
    let source_id = source_id(50);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("ByTail", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("ProofTail", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("proof", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("ConnectiveByTail", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("&", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("PrefixByTail", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("not", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("QuantifierProofTail", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("holds", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("theorem", ParserTokenKind::ReservedWord),
                ("ConstantByTail", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "theorem proof/justification tails should parse as concrete items: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("theorem tails should keep an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TheoremItem)),
        6
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::JustificationClause
        )),
        4
    );
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 6);
    assert!(item_list.children.iter().all(|item| {
        let item = ast.node(*item).unwrap();
        matches!(item.kind, SurfaceNodeKind::TheoremItem)
            && direct_child_has_kind(&ast, item, |kind| {
                matches!(kind, SurfaceNodeKind::FormulaExpression)
            })
    }));
}

#[test]
fn parser_recovers_missing_user_predicate_chain_term() {
    let source_id = source_id(51);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("divides", ParserTokenKind::UserSymbol),
                ("y", ParserTokenKind::Identifier),
                ("divides", ParserTokenKind::UserSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| &diagnostic.code)
            .collect::<Vec<_>>(),
        vec![&SyntaxDiagnosticCode::MalformedTermExpression]
    );
    let ast = output.ast.expect("missing chain term should recover");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PredicateSegment
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )
        }),
        1
    );
}

#[test]
fn parser_recovers_missing_builtin_predicate_right_term() {
    let source_id = source_id(46);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| &diagnostic.code)
            .collect::<Vec<_>>(),
        vec![&SyntaxDiagnosticCode::MalformedTermExpression]
    );
    let ast = output
        .ast
        .expect("missing predicate operand should recover");
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::BuiltinPredicateApplication)
        }),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
            )
        }),
        1
    );
}

#[test]
fn parser_rejects_mixed_user_and_builtin_predicate_chain() {
    let source_id = source_id(47);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("divides", ParserTokenKind::UserSymbol),
                ("y", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| &diagnostic.code)
            .collect::<Vec<_>>(),
        vec![&SyntaxDiagnosticCode::MalformedTermExpression]
    );
    let ast = output.ast.expect("mixed predicate chain should recover");
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(kind, SurfaceNodeKind::PredicateApplication)
        }),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
            )
        }),
        1
    );
}

#[test]
fn parser_recovers_missing_is_assertion_body() {
    let source_id = source_id(48);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| &diagnostic.code)
            .collect::<Vec<_>>(),
        vec![&SyntaxDiagnosticCode::MalformedTypeExpression]
    );
    let ast = output.ast.expect("missing is body should recover");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::IsAssertion)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
            )
        }),
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

#[test]
fn parser_parses_task18_consider_and_reconsider_nodes() {
    let source_id = source_id(93);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("consider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("and", ParserTokenKind::ReservedWord),
                ("contradiction", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("consider", ParserTokenKind::ReservedWord),
                ("z", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("B", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("reconsider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-18 statements should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("consider/reconsider statements should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 3);
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StatementItem)),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ConsiderStatement
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ReconsiderStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::QualifiedVariableSegment
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ConditionList)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::JustificationClause
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ReconsiderItem)),
        2
    );

    let shared_segment = ast
        .nodes()
        .iter()
        .find(|node| {
            matches!(node.kind, SurfaceNodeKind::QualifiedVariableSegment)
                && direct_token_texts(&ast, node) == vec!["x", ",", "y", "being"]
        })
        .expect("consider should preserve shared-type variable list");
    assert!(
        structural_children(&ast, shared_segment)
            .iter()
            .any(|child| matches!(child.kind, SurfaceNodeKind::TypeExpression))
    );

    let reconsider_statement = single_node(&ast, |kind| {
        matches!(kind, SurfaceNodeKind::ReconsiderStatement)
    });
    assert_eq!(
        direct_token_texts(&ast, reconsider_statement),
        vec!["reconsider", ",", "as", ";"]
    );
    let reconsider_items = structural_children(&ast, reconsider_statement)
        .into_iter()
        .filter(|child| matches!(child.kind, SurfaceNodeKind::ReconsiderItem))
        .collect::<Vec<_>>();
    assert_eq!(direct_token_texts(&ast, reconsider_items[0]), vec!["x"]);
    assert_eq!(
        direct_token_texts(&ast, reconsider_items[1]),
        vec!["y", "="]
    );
}

#[test]
fn parser_task47_parses_all_reconsider_tail_forms_with_exact_ownership() {
    let source_id = source_id(247);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("reconsider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("reconsider", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("reconsider", ParserTokenKind::ReservedWord),
                ("z", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "all canonical reconsider tails should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("canonical reconsider tails should retain an AST");
    let statements = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ReconsiderStatement))
        .collect::<Vec<_>>();
    assert_eq!(statements.len(), 3);

    let omitted_children = structural_children(&ast, statements[0]);
    assert!(!omitted_children.iter().any(|child| matches!(
        child.kind,
        SurfaceNodeKind::JustificationClause | SurfaceNodeKind::ProofBlock
    )));
    assert_eq!(
        direct_token_texts(&ast, statements[0]),
        vec!["reconsider", "as", ";"]
    );

    let explicit_children = structural_children(&ast, statements[1]);
    assert_eq!(
        explicit_children
            .iter()
            .filter(|child| matches!(child.kind, SurfaceNodeKind::JustificationClause))
            .count(),
        1,
        "explicit-simple reconsider should own exactly one JustificationClause"
    );
    assert_eq!(
        explicit_children
            .iter()
            .filter(|child| matches!(child.kind, SurfaceNodeKind::ProofBlock))
            .count(),
        0
    );

    let proof_children = structural_children(&ast, statements[2]);
    assert_eq!(
        proof_children
            .iter()
            .filter(|child| matches!(child.kind, SurfaceNodeKind::ProofBlock))
            .count(),
        1,
        "proof-tail reconsider should own exactly one ProofBlock child"
    );
    assert_eq!(
        proof_children
            .iter()
            .filter(|child| matches!(child.kind, SurfaceNodeKind::JustificationClause))
            .count(),
        0
    );
    let proof = proof_children
        .iter()
        .find(|child| matches!(child.kind, SurfaceNodeKind::ProofBlock))
        .expect("proof-tail reconsider should own one ProofBlock child");
    assert_eq!(direct_token_texts(&ast, proof), vec!["proof", "end"]);
    assert_eq!(
        direct_token_texts(&ast, statements[2]),
        vec!["reconsider", ",", "as", ";"],
        "the reconsider statement, not ProofBlock, should own the final semicolon"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ReconsiderItem)),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(_)
        )),
        0,
        "canonical tails should not insert recovery nodes"
    );
}

#[test]
fn parser_task47_keeps_reconsider_computation_justifications_forbidden() {
    let source_id = source_id(249);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("reconsider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedJustification),
        "reconsider computation tails should retain malformed-simple diagnostics"
    );
    let ast = output
        .ast
        .expect("forbidden reconsider computation tail should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        0,
        "reconsider must not admit computation justification nodes"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ReconsiderStatement
        )),
        1
    );
}

#[test]
fn parser_task47_unterminated_reconsider_proof_reuses_missing_end_recovery() {
    let source_id = source_id(248);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("reconsider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("proof", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    let ast = output
        .ast
        .expect("unterminated reconsider proof should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ProofBlock)),
        1
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
        )) >= 1,
        "reconsider proof tails should reuse proof-block missing-end recovery"
    );
}

#[test]
fn parser_recovers_task18_consider_and_reconsider_gaps() {
    let source_id = source_id(94);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("consider", ParserTokenKind::ReservedWord),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("consider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("consider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("consider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("consider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("reconsider", ParserTokenKind::ReservedWord),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("reconsider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("reconsider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("reconsider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("reconsider", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
    );
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression)
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
    );
    let ast = output
        .ast
        .expect("task-18 malformed statements should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ConsiderStatement
        )),
        5
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ReconsiderStatement
        )),
        5
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        0,
        "`by computation` should recover as malformed simple citation syntax"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )) >= 2
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 2
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )) >= 2
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
        )) >= 2,
        "the omitted reconsider tail no longer contributes MissingProofStep recovery"
    );
}

#[test]
fn parser_parses_task19_conclusion_then_iterative_nodes() {
    let source_id = source_id(95);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("thus", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("hence", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("A1", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("A2", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (".=", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("B", ParserTokenKind::Identifier),
                (".=", ParserTokenKind::ReservedSymbol),
                ("w", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (".=", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("B", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("consider", ParserTokenKind::ReservedWord),
                ("c", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("such", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("reconsider", ParserTokenKind::ReservedWord),
                ("r", ParserTokenKind::Identifier),
                ("as", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-19 statements should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output.ast.expect("task-19 statements should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 12);
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::StatementItem)),
        12
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::PlaceholderItem
        )),
        0,
        "`then per cases` is upgraded by task 20"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ConclusionStatement
        )),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ThenStatement)),
        7
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CompactStatement
        )),
        3,
        "`x = y by A;`, `then thesis by A;`, and `then x = y by A;` should stay compact"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::IterativeEqualityStatement
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::IterativeEqualityStep
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ConsiderStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ReconsiderStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CaseReasoningStatement
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        2,
        "conclusion statements should allow computation justifications"
    );

    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::CompactStatement)
                && structural_children(&ast, node).iter().any(|child| {
                    matches!(child.kind, SurfaceNodeKind::Proposition)
                        && direct_token_texts(&ast, child) == vec!["A1", ":"]
                })
        }),
        "labelled `x = y by A;` should dispatch to CompactStatement"
    );

    let then_child_kinds = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ThenStatement))
        .flat_map(|node| structural_children(&ast, node))
        .map(|child| &child.kind)
        .collect::<Vec<_>>();
    assert!(
        then_child_kinds
            .iter()
            .any(|kind| matches!(kind, SurfaceNodeKind::ConclusionStatement)),
        "`then` should wrap conclusion statements"
    );
    assert!(
        then_child_kinds
            .iter()
            .any(|kind| matches!(kind, SurfaceNodeKind::ReconsiderStatement)),
        "`then` should wrap reconsider statements"
    );
    assert!(
        then_child_kinds
            .iter()
            .any(|kind| matches!(kind, SurfaceNodeKind::CaseReasoningStatement)),
        "`then` should wrap case reasoning statements"
    );

    let then_compact_token_texts = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ThenStatement))
        .flat_map(|node| structural_children(&ast, node))
        .filter(|child| matches!(child.kind, SurfaceNodeKind::CompactStatement))
        .map(|child| subtree_token_texts(&ast, child))
        .collect::<Vec<_>>();
    assert!(
        then_compact_token_texts.iter().any(|tokens| tokens
            .iter()
            .map(String::as_str)
            .eq(["x", "=", "y", "by", "A", ";"])),
        "`then x = y by A;` should dispatch to a compact child"
    );
}

#[test]
fn parser_recovers_task19_conclusion_then_iterative_gaps() {
    let source_id = source_id(96);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("then", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (".=", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("B", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (".=", ParserTokenKind::ReservedSymbol),
                ("by", ParserTokenKind::ReservedWord),
                ("B", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (".=", ParserTokenKind::ReservedSymbol),
                ("z", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
    );
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
        .expect("task-19 malformed statements should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ThenStatement)),
        1
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
            SurfaceNodeKind::IterativeEqualityStatement
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::IterativeEqualityStep
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStatement)
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )),
        1
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        0,
        "`by computation` should recover as malformed simple citation syntax in iterative equality"
    );
}

#[test]
fn parser_parses_task20_block_statement_nodes() {
    let source_id = source_id(97);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("A1", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("now", ParserTokenKind::ReservedWord),
                ("assume", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("hereby", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("hereby", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("case", ParserTokenKind::ReservedWord),
                ("C1", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("case", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("hence", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("suppose", ParserTokenKind::ReservedWord),
                ("S1", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("suppose", ParserTokenKind::ReservedWord),
                ("that", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("case", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-20 block statements should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-20 block statements should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 5);
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::NowStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::HerebyStatement
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CaseReasoningStatement
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::CaseItem)),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SupposeItem)),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ThenStatement)),
        2,
        "`then thesis by A;` and `then per cases; ...` should both be represented"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::CaseItem)
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::ConditionList))
        }),
        "`case that thesis;` should preserve a condition list branch header"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::SupposeItem)
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::ConditionList))
        }),
        "`suppose that thesis;` should preserve a condition list branch header"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::ThenStatement)
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::CaseReasoningStatement))
        }),
        "`then per cases` should wrap case reasoning"
    );
}

#[test]
fn parser_recovers_task20_block_statement_gaps() {
    let source_id = source_id(98);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("then", ParserTokenKind::ReservedWord),
                ("now", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("hereby", ParserTokenKind::ReservedWord),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("computation", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("steps", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                ("case", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("case", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("junk", ParserTokenKind::Identifier),
                ("case", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("case", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("case", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("suppose", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
    );
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
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
    );
    let ast = output
        .ast
        .expect("task-20 malformed blocks should recover an AST");
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStatement)
        )) >= 2,
        "`then now` and `then hereby` should stay non-linkable"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
        )) >= 1,
        "`per cases by computation(...)` should recover as malformed simple citation syntax"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
        )) >= 1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ComputationJustification
        )),
        0,
        "case reasoning uses simple justifications only"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::CaseItem)) >= 5,
        "`per cases by A` with a missing header semicolon should still preserve the following case branch"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::CaseReasoningStatement)
                && structural_children(&ast, node)
                    .iter()
                    .filter(|child| matches!(child.kind, SurfaceNodeKind::CaseItem))
                    .count()
                    >= 2
        }),
        "malformed branch-body tokens should not swallow the following `case` header"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::SupposeItem)),
        0,
        "mixed `suppose` branches should stay outside the existing case-list node"
    );
}

#[test]
fn parser_recovers_task20_missing_now_hereby_ends() {
    let source_id = source_id(167);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("now", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("hereby", ParserTokenKind::ReservedWord),
                ("thus", ParserTokenKind::ReservedWord),
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
            .filter(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingEnd)
            .count()
            >= 2,
        "missing `end` diagnostics should be emitted for both `now` and nested `hereby`"
    );
    let ast = output
        .ast
        .expect("missing task-20 block ends should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::NowStatement)),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::HerebyStatement
        )),
        1
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingEnd)
        )) >= 2
    );
}

#[test]
fn parser_parses_task21_inline_definition_nodes() {
    let source_id = source_id(168);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("deffunc", ParserTokenKind::ReservedWord),
                ("F", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("C", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("42", ParserTokenKind::Numeral),
                (";", ParserTokenKind::ReservedSymbol),
                ("defpred", ParserTokenKind::ReservedWord),
                ("P", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (")", ParserTokenKind::ReservedSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("defpred", ParserTokenKind::ReservedWord),
                ("Trivial", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("now", ParserTokenKind::ReservedWord),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("H", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("defpred", ParserTokenKind::ReservedWord),
                ("Q", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "task-21 inline definitions should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("task-21 inline definitions should keep an AST");
    let item_list = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::ItemList));
    assert_eq!(item_list.children.len(), 5);
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::InlineFunctorDefinition
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::InlinePredicateDefinition
        )),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::TypedParameter)),
        4
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::NowStatement)),
        1,
        "reasoning bodies should accept local inline definitions"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ThenStatement)),
        0,
        "inline definitions are standalone statements, not linkable children"
    );
}

#[test]
fn parser_recovers_task21_inline_definition_gaps() {
    let source_id = source_id(169);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("then", ParserTokenKind::ReservedWord),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("F", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("defpred", ParserTokenKind::ReservedWord),
                ("P", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("N", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("G", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("Nat", ParserTokenKind::Identifier),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("H", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("I", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("J", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("K", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("equals", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("L", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("M", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("equals", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("defpred", ParserTokenKind::ReservedWord),
                ("R", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("defpred", ParserTokenKind::ReservedWord),
                ("U", ParserTokenKind::Identifier),
                ("x", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("defpred", ParserTokenKind::ReservedWord),
                ("S", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("means", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("defpred", ParserTokenKind::ReservedWord),
                ("T", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("means", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == SyntaxDiagnosticCode::MalformedTermExpression })
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
    );
    assert!(
        output.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == SyntaxDiagnosticCode::MalformedFormulaExpression
        })
    );
    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
    );
    let ast = output
        .ast
        .expect("task-21 malformed inline definitions should recover an AST");
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStatement)
        )) >= 2,
        "`then deffunc` and `then defpred` should remain non-linkable"
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm)
        )) >= 2
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTypeExpression)
        )) >= 2
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingFormula)
        )) >= 1
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)
        )) >= 1
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::InlineFunctorDefinition
        )) >= 8
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::InlinePredicateDefinition
        )) >= 4
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::InlineFunctorDefinition)
                && subtree_token_texts(&ast, node)
                    .iter()
                    .map(String::as_str)
                    .eq([
                        "deffunc", "N", "x", "be", "set", "->", "set", "equals", "x", ";",
                    ])
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TypedParameter))
        }),
        "missing `(` in `deffunc N x be set -> ...` should preserve the typed parameter"
    );
    assert!(
        ast.nodes().iter().any(|node| {
            matches!(node.kind, SurfaceNodeKind::InlinePredicateDefinition)
                && subtree_token_texts(&ast, node)
                    .iter()
                    .map(String::as_str)
                    .eq(["defpred", "U", "x", "be", "set", "means", "thesis", ";"])
                && structural_children(&ast, node)
                    .iter()
                    .any(|child| matches!(child.kind, SurfaceNodeKind::TypedParameter))
        }),
        "missing `(` in `defpred U x be set means ...` should preserve the typed parameter"
    );
}

#[test]
fn parser_recovers_task21_inline_definition_before_case_branch() {
    let source_id = source_id(170);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("per", ParserTokenKind::ReservedWord),
                ("cases", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("case", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("deffunc", ParserTokenKind::ReservedWord),
                ("Bad", ParserTokenKind::Identifier),
                ("(", ParserTokenKind::ReservedSymbol),
                (")", ParserTokenKind::ReservedSymbol),
                ("->", ParserTokenKind::ReservedSymbol),
                ("1", ParserTokenKind::Numeral),
                ("case", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression)
    );
    let ast = output
        .ast
        .expect("malformed inline definition before case branch should recover an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::CaseItem)),
        2,
        "inline-definition recovery must not swallow the following `case` header"
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::InlineFunctorDefinition
        )),
        1
    );
    assert!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )) >= 1,
        "the malformed return type should be recovered without consuming the next branch"
    );
}

#[test]
fn parser_rejects_task17_noncanonical_statement_justification_tails() {
    let source_id = source_id(92);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("assume", ParserTokenKind::ReservedWord),
                ("thesis", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("given", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("take", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("set", ParserTokenKind::ReservedWord),
                ("f", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert_eq!(
        output
            .diagnostics
            .iter()
            .map(|diagnostic| &diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            &SyntaxDiagnosticCode::MalformedTermExpression,
            &SyntaxDiagnosticCode::MalformedTermExpression,
            &SyntaxDiagnosticCode::MalformedTermExpression,
            &SyntaxDiagnosticCode::MalformedTermExpression,
        ]
    );
    let ast = output
        .ast
        .expect("noncanonical justification tails should recover");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::JustificationClause
        )),
        0
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::SkippedToken)
        )),
        4
    );
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 4);
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::AssumptionStatement
                | SurfaceNodeKind::GivenStatement
                | SurfaceNodeKind::TakeStatement
                | SurfaceNodeKind::SetStatement
        )),
        4
    );
}

fn single_node(
    ast: &mizar_syntax::SurfaceAst,
    predicate: impl Fn(&SurfaceNodeKind) -> bool,
) -> &mizar_syntax::SurfaceNode {
    ast.nodes()
        .iter()
        .find(|node| predicate(&node.kind))
        .expect("expected exactly one matching node")
}

fn formula_root_for_label<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    label: &str,
) -> &'a mizar_syntax::SurfaceNode {
    let item = ast
        .nodes()
        .iter()
        .find(|node| {
            matches!(
                node.kind,
                SurfaceNodeKind::TheoremItem
                    | SurfaceNodeKind::LemmaItem
                    | SurfaceNodeKind::PlaceholderItem
            ) && node.children.iter().any(|child| {
                ast.node(*child)
                    .and_then(mizar_syntax::SurfaceNode::token_text)
                    == Some(label)
            })
        })
        .unwrap_or_else(|| panic!("expected theorem-like item labelled `{label}`"));
    let formula_expression = item
        .children
        .iter()
        .filter_map(|child| ast.node(*child))
        .find(|child| matches!(child.kind, SurfaceNodeKind::FormulaExpression))
        .unwrap_or_else(|| panic!("expected formula expression for `{label}`"));
    assert_eq!(
        formula_expression.children.len(),
        1,
        "FormulaExpression for `{label}` should wrap exactly one child"
    );
    ast.node(formula_expression.children[0])
        .unwrap_or_else(|| panic!("expected formula child for `{label}`"))
}

fn set_comprehension_term_for_label<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    label: &str,
) -> &'a mizar_syntax::SurfaceNode {
    let formula = formula_root_for_label(ast, label);
    let term = builtin_predicate_left_term_payload(ast, formula);
    assert!(
        matches!(term.kind, SurfaceNodeKind::SetComprehension),
        "expected left term for `{label}` to be SetComprehension, got {:?}",
        term.kind
    );
    term
}

fn builtin_predicate_left_term_payload<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    formula: &mizar_syntax::SurfaceNode,
) -> &'a mizar_syntax::SurfaceNode {
    assert!(
        matches!(formula.kind, SurfaceNodeKind::BuiltinPredicateApplication),
        "expected builtin predicate application, got {:?}",
        formula.kind
    );
    let terms = structural_children(ast, formula)
        .into_iter()
        .filter(|child| matches!(child.kind, SurfaceNodeKind::TermExpression))
        .collect::<Vec<_>>();
    assert!(
        terms.len() >= 2,
        "builtin predicate application should have left and right terms"
    );
    let left_children = structural_children(ast, terms[0]);
    assert_eq!(
        left_children.len(),
        1,
        "left TermExpression should wrap one concrete term"
    );
    left_children[0]
}

fn set_comprehension_mapper(
    ast: &mizar_syntax::SurfaceAst,
    comprehension: &mizar_syntax::SurfaceNode,
) -> mizar_syntax::SurfaceNodeId {
    assert!(
        matches!(comprehension.kind, SurfaceNodeKind::SetComprehension),
        "expected SetComprehension, got {:?}",
        comprehension.kind
    );
    comprehension
        .children
        .iter()
        .copied()
        .find(|child| {
            ast.node(*child)
                .is_some_and(|node| matches!(node.kind, SurfaceNodeKind::TermExpression))
        })
        .expect("SetComprehension should own a mapper TermExpression")
}

fn structural_children<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
) -> Vec<&'a mizar_syntax::SurfaceNode> {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter(|child| !matches!(child.kind, SurfaceNodeKind::Token(_)))
        .collect()
}

fn direct_child_labels<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    node: &'a mizar_syntax::SurfaceNode,
) -> Vec<&'a str> {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .map(|child| match child.token_text() {
            Some(text) => text,
            None => match child.kind {
                SurfaceNodeKind::Annotation => "Annotation",
                SurfaceNodeKind::DefinitionParameter => "DefinitionParameter",
                SurfaceNodeKind::QualifiedSymbol => "QualifiedSymbol",
                SurfaceNodeKind::TypeArguments => "TypeArguments",
                SurfaceNodeKind::TypeHead => "TypeHead",
                SurfaceNodeKind::FormulaDefiniens => "FormulaDefiniens",
                SurfaceNodeKind::TermDefiniens => "TermDefiniens",
                SurfaceNodeKind::CorrectnessCondition => "CorrectnessCondition",
                SurfaceNodeKind::ErrorRecovery(_) => "ErrorRecovery",
                ref other => panic!("unexpected direct child kind in Task-48 assertion: {other:?}"),
            },
        })
        .collect()
}

fn surface_views<'a>(
    ast: &'a mizar_syntax::SurfaceAst,
    predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
) -> Vec<mizar_syntax::SurfaceNodeView<'a>> {
    let mut views = Vec::new();
    if let Some(root) = ast.root_view() {
        collect_surface_views(root, predicate, &mut views);
    }
    views
}

fn collect_surface_views<'a>(
    view: mizar_syntax::SurfaceNodeView<'a>,
    predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
    views: &mut Vec<mizar_syntax::SurfaceNodeView<'a>>,
) {
    if predicate(view.kind()) {
        views.push(view);
    }
    for child in view.child_views() {
        collect_surface_views(child, predicate, views);
    }
}

fn direct_structural_child_kind_names(
    view: mizar_syntax::SurfaceNodeView<'_>,
) -> Vec<&'static str> {
    view.child_views()
        .filter(|child| !matches!(child.kind(), SurfaceNodeKind::Token(_)))
        .filter_map(|child| task33_kind_name(child.kind()))
        .collect()
}

fn direct_child_signature(view: mizar_syntax::SurfaceNodeView<'_>) -> Vec<String> {
    view.child_views()
        .filter_map(|child| {
            view_token_text(child)
                .map(str::to_owned)
                .or_else(|| task33_kind_name(child.kind()).map(str::to_owned))
        })
        .collect()
}

fn assert_contains_signature(signatures: &[Vec<String>], expected: &[&str]) {
    let expected = expected
        .iter()
        .copied()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert!(
        signatures.iter().any(|signature| signature == &expected),
        "expected signature {:?}, got {signatures:#?}",
        expected
    );
}

fn task33_kind_name(kind: &SurfaceNodeKind) -> Option<&'static str> {
    match kind {
        SurfaceNodeKind::FormulaExpression => Some("FormulaExpression"),
        SurfaceNodeKind::TermExpression => Some("TermExpression"),
        SurfaceNodeKind::TypeExpression => Some("TypeExpression"),
        SurfaceNodeKind::AlgorithmParameters => Some("AlgorithmParameters"),
        SurfaceNodeKind::AlgorithmBody => Some("AlgorithmBody"),
        SurfaceNodeKind::AlgorithmStatementList => Some("AlgorithmStatementList"),
        SurfaceNodeKind::AlgorithmTerminationClause => Some("AlgorithmTerminationClause"),
        SurfaceNodeKind::AlgorithmRequiresClause => Some("AlgorithmRequiresClause"),
        SurfaceNodeKind::AlgorithmEnsuresClause => Some("AlgorithmEnsuresClause"),
        SurfaceNodeKind::AlgorithmDecreasingClause => Some("AlgorithmDecreasingClause"),
        SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm) => Some("MissingTerm"),
        SurfaceNodeKind::IfStatement => Some("IfStatement"),
        SurfaceNodeKind::LoopInvariantClause => Some("LoopInvariantClause"),
        SurfaceNodeKind::LoopDecreasingClause => Some("LoopDecreasingClause"),
        SurfaceNodeKind::AssertStatement => Some("AssertStatement"),
        SurfaceNodeKind::TermList => Some("TermList"),
        SurfaceNodeKind::MatchCase => Some("MatchCase"),
        SurfaceNodeKind::MatchEnding => Some("MatchEnding"),
        SurfaceNodeKind::JustificationClause => Some("JustificationClause"),
        SurfaceNodeKind::PredicatePattern => Some("PredicatePattern"),
        SurfaceNodeKind::FormulaDefiniens => Some("FormulaDefiniens"),
        SurfaceNodeKind::CoherenceCondition => Some("CoherenceCondition"),
        _ => None,
    }
}

fn view_starts_with_token(view: mizar_syntax::SurfaceNodeView<'_>, expected: &str) -> bool {
    view.child_views().find_map(view_token_text) == Some(expected)
}

fn view_token_text(view: mizar_syntax::SurfaceNodeView<'_>) -> Option<&str> {
    match view.kind() {
        SurfaceNodeKind::Token(token) => Some(token.text.as_ref()),
        _ => None,
    }
}

fn direct_token_texts(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
) -> Vec<String> {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .filter_map(mizar_syntax::SurfaceNode::token_text)
        .map(str::to_owned)
        .collect()
}

fn subtree_token_texts(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
) -> Vec<String> {
    let mut texts = Vec::new();
    collect_subtree_token_texts(ast, node, &mut texts);
    texts
}

fn collect_subtree_token_texts(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
    texts: &mut Vec<String>,
) {
    for child in &node.children {
        let Some(child) = ast.node(*child) else {
            continue;
        };
        if let Some(text) = child.token_text() {
            texts.push(text.to_owned());
        } else {
            collect_subtree_token_texts(ast, child, texts);
        }
    }
}

fn assert_binary_formula(
    node: &mizar_syntax::SurfaceNode,
    connective: SurfaceFormulaConnective,
    repeated: bool,
) {
    let SurfaceNodeKind::BinaryFormula(operator) = &node.kind else {
        panic!("expected BinaryFormula, got {:?}", node.kind);
    };
    assert_eq!(operator.connective, connective);
    assert_eq!(operator.repeated, repeated);
}

fn assert_quantified_formula(node: &mizar_syntax::SurfaceNode, quantifier: SurfaceQuantifierKind) {
    let SurfaceNodeKind::QuantifiedFormula(actual) = &node.kind else {
        panic!("expected QuantifiedFormula, got {:?}", node.kind);
    };
    assert_eq!(*actual, quantifier);
}

fn assert_formula_constant(node: &mizar_syntax::SurfaceNode, constant: SurfaceFormulaConstant) {
    let SurfaceNodeKind::FormulaConstant(actual) = &node.kind else {
        panic!("expected FormulaConstant, got {:?}", node.kind);
    };
    assert_eq!(*actual, constant);
}

fn count_nodes(
    ast: &mizar_syntax::SurfaceAst,
    predicate: impl Fn(&SurfaceNodeKind) -> bool,
) -> usize {
    ast.nodes()
        .iter()
        .filter(|node| predicate(&node.kind))
        .count()
}

fn subtree_contains_kind(
    ast: &mizar_syntax::SurfaceAst,
    id: mizar_syntax::SurfaceNodeId,
    predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
) -> bool {
    let Some(node) = ast.node(id) else {
        return false;
    };
    predicate(&node.kind)
        || node
            .children
            .iter()
            .any(|child| subtree_contains_kind(ast, *child, predicate))
}

fn node_subtree_contains_kind(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
    predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
) -> bool {
    predicate(&node.kind)
        || node
            .children
            .iter()
            .any(|child| subtree_contains_kind(ast, *child, predicate))
}

fn direct_child_has_kind(
    ast: &mizar_syntax::SurfaceAst,
    node: &mizar_syntax::SurfaceNode,
    predicate: impl Fn(&SurfaceNodeKind) -> bool,
) -> bool {
    node.children
        .iter()
        .filter_map(|child| ast.node(*child))
        .any(|child| predicate(&child.kind))
}

fn rowan_token_texts(ast: &mizar_syntax::SurfaceAst) -> Vec<String> {
    ast.rowan_root()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .map(|token| token.text().to_owned())
        .collect()
}

fn qualified_symbol_token_texts(
    ast: &mizar_syntax::SurfaceAst,
    qualified: &mizar_syntax::SurfaceNode,
) -> Vec<String> {
    qualified
        .children
        .iter()
        .filter_map(|id| ast.node(*id))
        .filter_map(|node| {
            node.token_text().map(str::to_owned).or_else(|| {
                node.children
                    .first()
                    .and_then(|id| ast.node(*id))
                    .and_then(mizar_syntax::SurfaceNode::token_text)
                    .map(str::to_owned)
            })
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
    ParserToken::new(kind, text, range(source_id, start, end))
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

#[derive(Debug)]
struct ExpectedParserDiagnostic {
    code: SyntaxDiagnosticCode,
    message: &'static str,
    primary_text: &'static str,
    primary_ordinal: usize,
    recovery_note: &'static str,
}

impl ExpectedParserDiagnostic {
    fn term(message: &'static str, primary_text: &'static str, primary_ordinal: usize) -> Self {
        Self {
            code: SyntaxDiagnosticCode::MalformedTermExpression,
            message,
            primary_text,
            primary_ordinal,
            recovery_note: "repair the term expression before continuing",
        }
    }

    fn formula(message: &'static str, primary_text: &'static str, primary_ordinal: usize) -> Self {
        Self {
            code: SyntaxDiagnosticCode::MalformedFormulaExpression,
            message,
            primary_text,
            primary_ordinal,
            recovery_note: "repair the formula expression before continuing",
        }
    }
}

fn assert_task33_algorithm_body_diagnostics_exact(
    source_id: SourceId,
    body: &[(&str, ParserTokenKind)],
    expected: &[ExpectedParserDiagnostic],
) {
    let tokens = task33_algorithm_body_tokens(source_id, body);
    assert_parsed_tokens_diagnostics_exact(source_id, tokens, expected);
}

fn assert_parser_diagnostics_exact(
    source_id: SourceId,
    entries: &[(&str, ParserTokenKind)],
    expected: &[ExpectedParserDiagnostic],
) {
    let tokens = token_sequence(source_id, entries);
    assert_parsed_tokens_diagnostics_exact(source_id, tokens, expected);
}

fn assert_parsed_tokens_diagnostics_exact(
    source_id: SourceId,
    tokens: Vec<ParserToken>,
    expected: &[ExpectedParserDiagnostic],
) {
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens.clone(),
        Vec::new(),
    ));

    assert_eq!(
        output.diagnostics.len(),
        expected.len(),
        "{:#?}",
        output.diagnostics
    );
    for (diagnostic, expected) in output.diagnostics.iter().zip(expected) {
        assert_matches_expected_diagnostic(diagnostic, &tokens, expected);
    }
}

fn task33_algorithm_body_tokens(
    source_id: SourceId,
    body: &[(&str, ParserTokenKind)],
) -> Vec<ParserToken> {
    let mut entries = vec![
        ("definition", ParserTokenKind::ReservedWord),
        ("algorithm", ParserTokenKind::ReservedWord),
        ("diagnostic_flow", ParserTokenKind::Identifier),
        ("(", ParserTokenKind::ReservedSymbol),
        (")", ParserTokenKind::ReservedSymbol),
        ("do", ParserTokenKind::ReservedWord),
    ];
    entries.extend_from_slice(body);
    entries.extend_from_slice(&[
        ("end", ParserTokenKind::ReservedWord),
        (";", ParserTokenKind::ReservedSymbol),
        ("end", ParserTokenKind::ReservedWord),
        (";", ParserTokenKind::ReservedSymbol),
    ]);
    token_sequence(source_id, &entries)
}

fn assert_matches_expected_diagnostic(
    diagnostic: &mizar_syntax::SyntaxDiagnostic,
    tokens: &[ParserToken],
    expected: &ExpectedParserDiagnostic,
) {
    assert!(
        diagnostic_matches_expected(diagnostic, tokens, expected),
        "expected {expected:?}, got {diagnostic:#?}"
    );
}

fn diagnostic_matches_expected(
    diagnostic: &mizar_syntax::SyntaxDiagnostic,
    tokens: &[ParserToken],
    expected: &ExpectedParserDiagnostic,
) -> bool {
    diagnostic.code == expected.code
        && diagnostic.message.as_ref() == expected.message
        && diagnostic.primary
            == nth_token_range(tokens, expected.primary_text, expected.primary_ordinal)
        && diagnostic.recovery_note.as_deref() == Some(expected.recovery_note)
}

fn nth_token_range(tokens: &[ParserToken], text: &str, nth: usize) -> SourceRange {
    tokens
        .iter()
        .filter(|token| token.text.as_ref() == text)
        .nth(nth)
        .unwrap_or_else(|| panic!("expected token `{text}` at ordinal {nth}"))
        .span
}

fn operator_fixture_fixities() -> Vec<OperatorFixityEntry> {
    vec![
        OperatorFixityEntry::prefix("~", 70),
        OperatorFixityEntry::postfix("!", 90),
        OperatorFixityEntry::infix("++", 10, OperatorAssociativity::Left),
        OperatorFixityEntry::infix("**", 20, OperatorAssociativity::Right),
        OperatorFixityEntry::infix("%%", 10, OperatorAssociativity::NonAssociative),
    ]
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
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
