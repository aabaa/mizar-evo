use super::*;

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
                ("by", ParserTokenKind::ReservedWord),
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
        )) >= 2,
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
        ],
        "malformed template args should report exact type diagnostics"
    );
    let ast = output
        .ast
        .expect("malformed template args should recover an AST");
    let first_close = ast
        .nodes()
        .iter()
        .find(|node| node.token_text() == Some("]"))
        .unwrap();
    let last_semicolon = ast
        .nodes()
        .iter()
        .rev()
        .find(|node| node.token_text() == Some(";"))
        .unwrap();
    assert_eq!(output.diagnostics[0].primary, first_close.range);
    assert_eq!(output.diagnostics[1].primary, last_semicolon.range);
    let qua = single_node(&ast, |kind| matches!(kind, SurfaceNodeKind::QuaExpression));
    assert_eq!(
        subtree_token_texts(&ast, qua),
        ["q", "qua", "non", "empty", "T"]
    );
    let target = ast.node(qua.children[2]).unwrap();
    assert_eq!(target.kind, SurfaceNodeKind::TypeExpression);
    assert_eq!(target.children.len(), 2);
    assert_eq!(
        ast.node(target.children[0]).unwrap().kind,
        SurfaceNodeKind::AttributeChain
    );
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
        0
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
fn parser_recovers_imported_postfix_fallback_ownership_matrix() {
    const RECOVERABLE_MUTATIONS: [usize; 9] = [48, 49, 50, 51, 52, 53, 54, 112, 148];

    assert_eq!(B1B1P_RECOVERY_SOURCE.len(), 158);
    assert_eq!(&B1B1P_RECOVERY_SOURCE[48..55], "theorem");
    assert_eq!(&B1B1P_RECOVERY_SOURCE[112..113], "=");
    assert_eq!(&B1B1P_RECOVERY_SOURCE[116..121], "proof");
    assert_eq!(&B1B1P_RECOVERY_SOURCE[148..149], "=");

    for (ordinal, mutation) in RECOVERABLE_MUTATIONS.into_iter().enumerate() {
        let source_id = source_id(220 + ordinal as u8);
        let tokens = b1b1p_recovery_tokens(source_id, mutation);
        let request = || {
            ParseRequest::new(
                source_id,
                Edition::new("2026"),
                tokens.clone(),
                operator_fixture_fixities(),
            )
        };

        let first = parse(request());
        let second = parse(request());
        let first_ast = first
            .ast
            .as_ref()
            .unwrap_or_else(|| panic!("mutation byte {mutation} must retain a recovered AST"));
        let second_ast = second
            .ast
            .as_ref()
            .unwrap_or_else(|| panic!("replay byte {mutation} must retain a recovered AST"));

        assert!(
            !first.diagnostics.is_empty(),
            "mutation byte {mutation} must retain syntax diagnostics"
        );
        assert!(
            first_ast
                .nodes()
                .iter()
                .any(|node| matches!(node.kind, SurfaceNodeKind::ErrorRecovery(_))),
            "mutation byte {mutation} must retain explicit recovery structure"
        );
        if mutation == 112 {
            assert_eq!(
                count_nodes(first_ast, |kind| matches!(
                    kind,
                    SurfaceNodeKind::StatementItem
                )),
                2,
                "the first unclaimed statement boundary after the claimed prefix must be preserved"
            );
            assert_eq!(
                count_nodes(first_ast, |kind| matches!(
                    kind,
                    SurfaceNodeKind::TakeStatement
                )),
                1,
                "the later `take` statement must not be swallowed by theorem recovery"
            );
            assert_eq!(
                count_nodes(first_ast, |kind| matches!(
                    kind,
                    SurfaceNodeKind::ConclusionStatement
                )),
                1,
                "the later `thus` statement must not be swallowed by theorem recovery"
            );
        }
        assert_eq!(
            first.diagnostics, second.diagnostics,
            "mutation byte {mutation} diagnostics must be deterministic"
        );
        assert_eq!(
            first_ast.snapshot_text(),
            second_ast.snapshot_text(),
            "mutation byte {mutation} AST must be deterministic"
        );
    }
}
