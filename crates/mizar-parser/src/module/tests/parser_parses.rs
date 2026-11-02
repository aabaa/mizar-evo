use super::*;

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
fn parser_parses_parameterized_multiple_comprehension_generators() {
    let source_id = source_id(170);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        token_sequence(
            source_id,
            &[
                ("theorem", ParserTokenKind::ReservedWord),
                ("NestedCaptureTwo", ParserTokenKind::Identifier),
                (":", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("{", ParserTokenKind::ReservedSymbol),
                ("[", ParserTokenKind::ReservedSymbol),
                ("x", ParserTokenKind::Identifier),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("]", ParserTokenKind::ReservedSymbol),
                ("where", ParserTokenKind::ReservedWord),
                ("z", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("Element", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("NAT", ParserTokenKind::UserSymbol),
                ("}", ParserTokenKind::ReservedSymbol),
                ("where", ParserTokenKind::ReservedWord),
                ("x", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("Element", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("NAT", ParserTokenKind::UserSymbol),
                (",", ParserTokenKind::ReservedSymbol),
                ("y", ParserTokenKind::Identifier),
                ("is", ParserTokenKind::ReservedWord),
                ("Element", ParserTokenKind::UserSymbol),
                ("of", ParserTokenKind::ReservedWord),
                ("NAT", ParserTokenKind::UserSymbol),
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
        "parameterized generators should parse without diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("parameterized generators should keep a full AST");
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
        3
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ApplicationTerm
        )),
        1
    );
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(_)
        )),
        0
    );

    let outer = set_comprehension_term_for_label(&ast, "NestedCaptureTwo");
    let outer_segments = structural_children(&ast, outer)
        .into_iter()
        .filter(|child| matches!(child.kind, SurfaceNodeKind::ComprehensionVariableSegment))
        .collect::<Vec<_>>();
    assert_eq!(outer_segments.len(), 2);
    assert_eq!(
        outer_segments
            .iter()
            .map(|segment| direct_token_texts(&ast, segment))
            .collect::<Vec<_>>(),
        vec![vec!["x", "is"], vec!["y", "is"],]
    );
    assert_eq!(
        outer_segments
            .iter()
            .map(|segment| {
                let types = structural_children(&ast, segment)
                    .into_iter()
                    .filter(|child| matches!(child.kind, SurfaceNodeKind::TypeExpression))
                    .collect::<Vec<_>>();
                assert_eq!(types.len(), 1);
                subtree_token_texts(&ast, types[0])
            })
            .collect::<Vec<_>>(),
        vec![vec!["Element", "of", "NAT"], vec!["Element", "of", "NAT"],]
    );

    let inner = structural_children(&ast, outer)
        .into_iter()
        .find(|child| matches!(child.kind, SurfaceNodeKind::TermExpression))
        .and_then(|mapper| {
            structural_children(&ast, mapper)
                .into_iter()
                .find(|child| matches!(child.kind, SurfaceNodeKind::SetComprehension))
        })
        .expect("outer mapper should be the inner comprehension");
    let inner_segments = structural_children(&ast, inner)
        .into_iter()
        .filter(|child| matches!(child.kind, SurfaceNodeKind::ComprehensionVariableSegment))
        .collect::<Vec<_>>();
    assert_eq!(inner_segments.len(), 1);
    assert_eq!(direct_token_texts(&ast, inner_segments[0]), vec!["z", "is"]);

    let bracket_mapper = structural_children(&ast, inner)
        .into_iter()
        .find(|child| matches!(child.kind, SurfaceNodeKind::TermExpression))
        .expect("inner mapper term expression");
    let bracket = structural_children(&ast, bracket_mapper);
    assert_eq!(bracket.len(), 1);
    assert!(matches!(bracket[0].kind, SurfaceNodeKind::ApplicationTerm));
    assert_eq!(direct_token_texts(&ast, bracket[0]), vec!["[", ",", "]"]);
    let bracket_terms = structural_children(&ast, bracket[0])
        .into_iter()
        .filter(|child| matches!(child.kind, SurfaceNodeKind::TermExpression))
        .collect::<Vec<_>>();
    assert_eq!(bracket_terms.len(), 2);
    assert_eq!(
        bracket_terms
            .iter()
            .map(|term| subtree_token_texts(&ast, term))
            .collect::<Vec<_>>(),
        vec![vec!["x"], vec!["y"]]
    );
}

#[test]
fn parser_parses_template_type_parameter_as_set_comprehension_generator_type() {
    let source_id = source_id(252);
    let output = parse(ParseRequest::new(
        source_id,
        Edition::new("2026"),
        fraenkel_template_tokens(source_id),
        Vec::new(),
    ));

    assert!(
        output.diagnostics.is_empty(),
        "the immutable Fraenkel seed should parse without syntax diagnostics: {:?}",
        output.diagnostics
    );
    let ast = output
        .ast
        .expect("the Fraenkel seed should keep a full AST");
    assert_eq!(ast.nodes().len(), 57);
    assert_eq!(ast.root().expect("seed should have a root").index(), 56);
    assert_eq!(
        count_nodes(&ast, |kind| matches!(
            kind,
            SurfaceNodeKind::ErrorRecovery(_)
        )),
        0
    );
    assert!(ast.trivia().skipped_token_ranges().is_empty());

    for (range, kind) in [
        (range(source_id, 678, 679), "TypeExpression"),
        (range(source_id, 678, 679), "TypeHead"),
        (range(source_id, 673, 679), "ComprehensionVariableSegment"),
        (range(source_id, 682, 692), "FormulaExpression"),
        (range(source_id, 663, 694), "SetComprehension"),
        (range(source_id, 623, 695), "FunctorDefinition"),
        (range(source_id, 593, 700), "DefinitionBlockItem"),
    ] {
        assert!(
            ast.nodes().iter().any(|node| {
                node.range == range
                    && match kind {
                        "TypeExpression" => {
                            matches!(node.kind, SurfaceNodeKind::TypeExpression)
                        }
                        "TypeHead" => matches!(node.kind, SurfaceNodeKind::TypeHead),
                        "ComprehensionVariableSegment" => {
                            matches!(node.kind, SurfaceNodeKind::ComprehensionVariableSegment)
                        }
                        "FormulaExpression" => {
                            matches!(node.kind, SurfaceNodeKind::FormulaExpression)
                        }
                        "SetComprehension" => {
                            matches!(node.kind, SurfaceNodeKind::SetComprehension)
                        }
                        "FunctorDefinition" => {
                            matches!(node.kind, SurfaceNodeKind::FunctorDefinition)
                        }
                        "DefinitionBlockItem" => {
                            matches!(node.kind, SurfaceNodeKind::DefinitionBlockItem)
                        }
                        _ => false,
                    }
            }),
            "expected {kind} at {}..{}; snapshot={}",
            range.start,
            range.end,
            ast.snapshot_text()
        );
    }
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
        1
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
