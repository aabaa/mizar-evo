use super::*;

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
