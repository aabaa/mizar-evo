use super::*;

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
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"+\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("left", ParserTokenKind::ReservedWord),
                (",", ParserTokenKind::ReservedSymbol),
                ("80", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "prefix_operator",
            &[
                ("prefix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"-\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("85", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        (
            "postfix_operator",
            &[
                ("postfix_operator", ParserTokenKind::ReservedWord),
                ("(", ParserTokenKind::ReservedSymbol),
                ("\"!\"", ParserTokenKind::StringLiteral),
                (",", ParserTokenKind::ReservedSymbol),
                ("95", ParserTokenKind::Numeral),
                (")", ParserTokenKind::ReservedSymbol),
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
            "infix_operator" | "prefix_operator" | "postfix_operator" => {
                SurfaceNodeKind::OperatorDeclaration
            }
            "synonym" | "antonym" => SurfaceNodeKind::NotationAlias,
            _ => SurfaceNodeKind::PlaceholderItem,
        };
        assert_eq!(item.kind, expected_kind);
        assert_eq!(item.range, expected_range, "{name} placeholder range");
    }
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
fn parser_retains_head_first_predicate_when_same_spelling_has_prefix_fixity() {
    let source_id = source_id(193);
    let tokens = token_sequence(
        source_id,
        &[
            ("theorem", ParserTokenKind::ReservedWord),
            ("OverloadedHead", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("shared_head", ParserTokenKind::UserSymbol),
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
        vec![OperatorFixityEntry::prefix("shared_head", 64)],
    ));

    assert!(
        output.diagnostics.is_empty(),
        "head-first predicate fallback should parse: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("head-first predicate fallback should keep an AST");
    assert!(
        ast.nodes()
            .iter()
            .any(|node| matches!(node.kind, SurfaceNodeKind::PredicateApplication))
    );
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
fn parser_scopes_required_identifier_type_fallback_to_template_definitions() {
    let scoped_source_id = source_id(253);
    let tokens = token_sequence(
        scoped_source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("type", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("func", ParserTokenKind::ReservedWord),
            ("Inner", ParserTokenKind::Identifier),
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
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            ("}", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("theorem", ParserTokenKind::ReservedWord),
            ("Outside", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("{", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("where", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("is", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            ("}", ParserTokenKind::ReservedSymbol),
            ("=", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let inner_type_range = nth_token_range(&tokens, "T", 1);
    let outside_type_range = nth_token_range(&tokens, "T", 2);
    let output = parse(ParseRequest::new(
        scoped_source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));
    let ast = output
        .ast
        .expect("scope fixture should retain a recovered AST");

    assert!(ast.nodes().iter().any(|node| {
        node.range == inner_type_range && matches!(node.kind, SurfaceNodeKind::TypeHead)
    }));
    assert!(!ast.nodes().iter().any(|node| {
        node.range == outside_type_range && matches!(node.kind, SurfaceNodeKind::TypeHead)
    }));
    assert!(output.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == SyntaxDiagnosticCode::MalformedTypeExpression
            && diagnostic.primary == outside_type_range
    }));

    let symbol_source_id = source_id(251);
    let symbol_tokens = token_sequence(
        symbol_source_id,
        &[
            ("definition", ParserTokenKind::ReservedWord),
            ("let", ParserTokenKind::ReservedWord),
            ("T", ParserTokenKind::Identifier),
            ("be", ParserTokenKind::ReservedWord),
            ("type", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
            ("func", ParserTokenKind::ReservedWord),
            ("Symbolic", ParserTokenKind::Identifier),
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
            ("Mode", ParserTokenKind::UserSymbol),
            (":", ParserTokenKind::ReservedSymbol),
            ("thesis", ParserTokenKind::ReservedWord),
            ("}", ParserTokenKind::ReservedSymbol),
            (";", ParserTokenKind::ReservedSymbol),
            ("end", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let symbol_range = nth_token_range(&symbol_tokens, "Mode", 0);
    let symbol_output = parse(ParseRequest::new(
        symbol_source_id,
        Edition::new("2026"),
        symbol_tokens,
        Vec::new(),
    ));
    assert!(symbol_output.diagnostics.is_empty());
    let symbol_ast = symbol_output
        .ast
        .expect("strict user-symbol generator type should retain an AST");
    let symbol_head = symbol_ast
        .nodes()
        .iter()
        .find(|node| node.range == symbol_range && matches!(node.kind, SurfaceNodeKind::TypeHead))
        .expect("user-symbol generator type should have a TypeHead");
    assert!(
        structural_children(&symbol_ast, symbol_head)
            .iter()
            .any(|child| matches!(child.kind, SurfaceNodeKind::QualifiedSymbol))
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
fn step5a4_parses_omitted_compact_justifications_with_and_without_then() {
    let source_id = source_id(254);
    let tokens = token_sequence(
        source_id,
        &[
            ("A1", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("then", ParserTokenKind::ReservedWord),
            ("A2", ParserTokenKind::Identifier),
            (":", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("then", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
            ("then", ParserTokenKind::ReservedWord),
            ("x", ParserTokenKind::Identifier),
            ("=", ParserTokenKind::ReservedSymbol),
            ("x", ParserTokenKind::Identifier),
            ("by", ParserTokenKind::ReservedWord),
            ("A1", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
    );
    let compact_ranges = [(0, 5), (6, 9), (11, 16), (18, 21), (23, 28)]
        .map(|(first, last)| range(source_id, tokens[first].span.start, tokens[last].span.end));
    let then_ranges = [(10, 16), (17, 21), (22, 28)]
        .map(|(first, last)| range(source_id, tokens[first].span.start, tokens[last].span.end));
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        tokens,
        Vec::new(),
    ));

    assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
    let ast = output
        .ast
        .expect("omitted compact statements should keep an AST");
    let mut compacts = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::CompactStatement))
        .collect::<Vec<_>>();
    compacts.sort_by_key(|node| node.range.start);
    assert_eq!(
        compacts.iter().map(|node| node.range).collect::<Vec<_>>(),
        compact_ranges
    );
    for compact in &compacts[..4] {
        assert_eq!(direct_token_texts(&ast, compact), vec![";"]);
        assert!(
            structural_children(&ast, compact)
                .iter()
                .all(|child| !matches!(child.kind, SurfaceNodeKind::JustificationClause))
        );
    }
    assert!(
        structural_children(&ast, compacts[4])
            .iter()
            .any(|child| matches!(child.kind, SurfaceNodeKind::JustificationClause))
    );

    let mut then_statements = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::ThenStatement))
        .collect::<Vec<_>>();
    then_statements.sort_by_key(|node| node.range.start);
    assert_eq!(
        then_statements
            .iter()
            .map(|node| node.range)
            .collect::<Vec<_>>(),
        then_ranges
    );
    assert!(then_statements.iter().all(|node| {
        let children = structural_children(&ast, node);
        children.len() == 1 && matches!(children[0].kind, SurfaceNodeKind::CompactStatement)
    }));
}

#[test]
fn step5a4_preserves_then_rejection_and_missing_semicolon_recovery() {
    let source_id = source_id(255);
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
                ("then", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("then", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                ("by", ParserTokenKind::ReservedWord),
                ("A", ParserTokenKind::Identifier),
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
            .any(|diagnostic| diagnostic.code == SyntaxDiagnosticCode::MissingSemicolon)
    );
    let ast = output.ast.expect("then recovery should retain an AST");
    assert_eq!(
        count_nodes(&ast, |kind| matches!(kind, SurfaceNodeKind::ThenStatement)),
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingStatement)
        )),
        2
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::CompactStatement
        )),
        1
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

#[test]
fn step5a6_unbracketed_type_argument_follow_set_preserves_ownership() {
    let source_id = source_id(251);
    let boundary_cases: &[&[(&str, ParserTokenKind)]] = &[
        &[("by", ParserTokenKind::ReservedWord)],
        &[("proof", ParserTokenKind::ReservedWord)],
        &[("->", ParserTokenKind::ReservedSymbol)],
        &[("equals", ParserTokenKind::ReservedWord)],
        &[("means", ParserTokenKind::ReservedWord)],
        &[("st", ParserTokenKind::ReservedWord)],
        &[("holds", ParserTokenKind::ReservedWord)],
        &[("&", ParserTokenKind::ReservedSymbol)],
        &[
            ("&", ParserTokenKind::ReservedSymbol),
            ("...", ParserTokenKind::ReservedSymbol),
            ("&", ParserTokenKind::ReservedSymbol),
        ],
        &[("or", ParserTokenKind::ReservedWord)],
        &[
            ("or", ParserTokenKind::ReservedWord),
            ("...", ParserTokenKind::ReservedSymbol),
            ("or", ParserTokenKind::ReservedWord),
        ],
        &[("implies", ParserTokenKind::ReservedWord)],
        &[("iff", ParserTokenKind::ReservedWord)],
    ];

    for boundary in boundary_cases {
        let mut entries = vec![
            ("T", ParserTokenKind::UserSymbol),
            ("of", ParserTokenKind::ReservedWord),
            ("a", ParserTokenKind::Identifier),
        ];
        entries.extend_from_slice(boundary);
        let mut parser = crate::grammar::Parser::new(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(source_id, &entries),
            Vec::new(),
        ));
        parser.add_token_nodes();
        let parsed = parser
            .parse_type_expression_at(0)
            .expect("the dependent-mode type expression should parse");
        assert_eq!(
            parsed.next_position, 3,
            "the type expression must leave its outer follow unconsumed: {boundary:?}"
        );
        assert!(parser.diagnostics.is_empty(), "{:#?}", parser.diagnostics);
        let ast = parser.events.finish(None, Some(parsed.id));
        let arguments = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::TypeArguments));
        assert_eq!(subtree_token_texts(&ast, arguments), vec!["of", "a"]);
        assert_eq!(
            &ast.token_texts()[3..],
            &boundary.iter().map(|(text, _)| *text).collect::<Vec<_>>(),
            "the outer follow tokens must remain in the source-preserving AST"
        );
    }

    for continuation in [
        (",", ParserTokenKind::ReservedSymbol),
        ("b", ParserTokenKind::Identifier),
    ] {
        let parser = crate::grammar::Parser::new(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("T", ParserTokenKind::UserSymbol),
                    ("of", ParserTokenKind::ReservedWord),
                    ("a", ParserTokenKind::Identifier),
                    continuation,
                ],
            ),
            Vec::new(),
        ));
        assert!(
            !parser.is_unbracketed_type_argument_follow_at(3),
            "comma and a possible next term must remain inside the argument-list grammar"
        );
    }

    for boundary in [
        ("holds", ParserTokenKind::ReservedWord),
        ("means", ParserTokenKind::ReservedWord),
    ] {
        let parser = crate::grammar::Parser::new(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(
                source_id,
                &[
                    ("T", ParserTokenKind::UserSymbol),
                    ("of", ParserTokenKind::ReservedWord),
                    ("a", ParserTokenKind::Identifier),
                    boundary,
                ],
            ),
            Vec::new(),
        ));
        assert!(
            !parser.is_term_argument_list_boundary_at(3),
            "the shared term-list boundary must not be widened"
        );
    }
}

#[test]
fn step5a6_parses_local_dependent_mode_uses_before_holds_and_means() {
    let source_id = source_id(252);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("Dependent", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("for", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                ("for", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("being", ParserTokenKind::ReservedWord),
                ("ParamMode", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                ("holds", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("proof", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("ParamMode", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("thus", ParserTokenKind::ReservedWord),
                ("y", ParserTokenKind::Identifier),
                ("=", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("definition", ParserTokenKind::ReservedWord),
                ("let", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                ("be", ParserTokenKind::ReservedWord),
                ("set", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("func", ParserTokenKind::ReservedWord),
                ("ParamModeFunctor", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("idembox", ParserTokenKind::UserSymbol),
                ("X", ParserTokenKind::Identifier),
                ("->", ParserTokenKind::ReservedSymbol),
                ("ParamMode", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("X", ParserTokenKind::Identifier),
                ("means", ParserTokenKind::ReservedWord),
                ("it", ParserTokenKind::ReservedWord),
                ("=", ParserTokenKind::ReservedSymbol),
                ("X", ParserTokenKind::Identifier),
                (";", ParserTokenKind::ReservedSymbol),
                ("existence", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("uniqueness", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
                ("end", ParserTokenKind::ReservedWord),
                (";", ParserTokenKind::ReservedSymbol),
            ],
        ),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "dependent-mode type arguments should preserve outer grammar tokens: {:#?}",
        output.diagnostics
    );
    let ast = output.ast.expect("dependent-mode uses should keep an AST");
    let arguments = ast
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind, SurfaceNodeKind::TypeArguments))
        .collect::<Vec<_>>();
    assert_eq!(arguments.len(), 3);
    for arguments in arguments {
        assert_eq!(subtree_token_texts(&ast, arguments), vec!["of", "X"]);
        assert_eq!(
            structural_children(&ast, arguments)
                .iter()
                .filter(|child| matches!(child.kind, SurfaceNodeKind::TermExpression))
                .count(),
            1
        );
    }
}

#[test]
fn step5a8_private_paths_accept_immediate_semicolons() {
    assert_step5a8_omitted_path(
        &[
            ("existence", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        |parser| {
            parser
                .parse_registration_correctness_condition_at(0, "existence")
                .id
        },
        |kind| matches!(kind, SurfaceNodeKind::CorrectnessCondition),
    );
    assert_step5a8_omitted_path(
        &[
            ("sethood", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        |parser| parser.parse_mode_property_at(0).id,
        |kind| matches!(kind, SurfaceNodeKind::ModeProperty),
    );
    assert_step5a8_omitted_path(
        &[
            ("symmetry", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        |parser| parser.parse_property_clause_at(0).id,
        |kind| matches!(kind, SurfaceNodeKind::PropertyClause),
    );
    assert_step5a8_omitted_path(
        &[
            ("coherence", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        |parser| parser.parse_coherence_condition_at(0).id,
        |kind| matches!(kind, SurfaceNodeKind::CoherenceCondition),
    );
    assert_step5a8_omitted_path(
        &[
            ("coherence", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        |parser| parser.parse_inheritance_coherence_condition_at(0).id,
        |kind| matches!(kind, SurfaceNodeKind::CoherenceCondition),
    );
    assert_step5a8_omitted_path(
        &[
            ("coherence", ParserTokenKind::ReservedWord),
            ("with", ParserTokenKind::ReservedWord),
            ("Label", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        |parser| parser.parse_coherence_condition_at(0).id,
        |kind| matches!(kind, SurfaceNodeKind::CoherenceCondition),
    );
}

#[test]
fn step5a8_private_paths_retain_malformed_justification_recovery() {
    assert_step5a8_malformed_path(
        &[
            ("sethood", ParserTokenKind::ReservedWord),
            ("by", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        |parser| parser.parse_mode_property_at(0).id,
        |kind| matches!(kind, SurfaceNodeKind::ModeProperty),
        |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
            )
        },
    );
    assert_step5a8_malformed_path(
        &[
            ("symmetry", ParserTokenKind::ReservedWord),
            ("junk", ParserTokenKind::Identifier),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        |parser| parser.parse_property_clause_at(0).id,
        |kind| matches!(kind, SurfaceNodeKind::PropertyClause),
        |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
            )
        },
    );
    assert_step5a8_malformed_path(
        &[
            ("coherence", ParserTokenKind::ReservedWord),
            ("with", ParserTokenKind::ReservedWord),
            (";", ParserTokenKind::ReservedSymbol),
        ],
        |parser| parser.parse_coherence_condition_at(0).id,
        |kind| matches!(kind, SurfaceNodeKind::CoherenceCondition),
        |kind| {
            matches!(
                kind,
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingProofStep)
            )
        },
    );
}

#[test]
fn complete_attribute_ref_probe_tracks_delimiters_and_stops_at_barriers() {
    let cases: &[(usize, &[&str], Option<usize>)] = &[
        (
            1,
            &[
                "source", "non", "M", ".", "small", "(", "the", "set", ",", "{", "x", "where", "x",
                "is", "set", ":", "P", "[", "x", "]", "}", ",", "[", "a", ",", "(", "b", ")", "]",
                ")", "next",
            ],
            Some(30),
        ),
        (0, &["small", "(", ")", "next"], Some(3)),
        (0, &["small", "(", "x", ",", ")", "next"], Some(5)),
        (0, &["small", "(", "x", "]", ")"], None),
        (0, &["small", "(", "[", "x", ")", "]", ")"], None),
        (0, &["small", "(", "[", "x", ")"], None),
        (0, &["small", "(", "x", ";", "y", ")"], None),
        (0, &["small", "(", "x", "end", "y", ")"], None),
        (0, &["small", "(", "x", "definition", "y", ")"], None),
        (0, &["small", "(", "x"], None),
    ];
    let source_id = source_id(253);
    for (start, spellings, expected_end) in cases {
        let entries = spellings
            .iter()
            .map(|text| {
                (
                    *text,
                    match *text {
                        "non" | "the" | "set" | "where" | "is" | "end" | "definition" => {
                            ParserTokenKind::ReservedWord
                        }
                        "(" | ")" | "[" | "]" | "{" | "}" | "," | ":" | "." | ";" => {
                            ParserTokenKind::ReservedSymbol
                        }
                        "small" => ParserTokenKind::UserSymbol,
                        _ => ParserTokenKind::Identifier,
                    },
                )
            })
            .collect::<Vec<_>>();
        let parser = crate::grammar::Parser::new(ParseRequest::new(
            source_id,
            Edition::new("2026"),
            token_sequence(source_id, &entries),
            Vec::new(),
        ));
        let plan = parser.plan_complete_attribute_ref_at(*start);
        assert_eq!(
            plan.map(|plan| (plan.start_position, plan.next_position)),
            expected_end.map(|end| (*start, end)),
            "{spellings:?}"
        );
        assert!(parser.token_node_ids.is_empty());
        assert!(parser.diagnostics.is_empty());
        assert!(parser.events.finish(None, None).nodes().is_empty());
    }
}
