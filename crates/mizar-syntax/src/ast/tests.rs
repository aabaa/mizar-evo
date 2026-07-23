use super::{
    SurfaceAstBuilder, SurfaceFormulaBinaryOperator, SurfaceFormulaConnective,
    SurfaceFormulaConstant, SurfaceFormulaPrefixOperator, SurfaceInfixOperator, SurfaceNodeKind,
    SurfaceNodeView, SurfaceOperatorAssociativity, SurfacePostfixOperator, SurfacePrefixOperator,
    SurfaceQuantifierKind, SurfaceTokenKind, SyntaxKind,
};
use crate::SyntaxRecoveryKind;
use crate::{
    SkippedTokenReason, SurfaceTriviaBuilder, TriviaAttachmentTarget, TriviaNodeTarget,
    TriviaPlacement, WhitespaceHintKind,
};
use mizar_session::{
    BuildSnapshotId, CommentKind, Hash, InMemorySessionIdAllocator, SessionIdAllocator,
    SourceAnchor, SourceId, SourceRange,
};

#[test]
fn builder_round_trips_into_rowan_backed_tree() {
    let source_id = source_id(1);
    let ast = expression_ast(source_id);

    assert_eq!(ast.token_texts(), vec!["a", "++", "b"]);
    assert_eq!(ast.rowan_root().kind(), SyntaxKind::Root);
    let rowan_kinds = ast
        .rowan_root()
        .descendants_with_tokens()
        .map(|element| element.kind())
        .collect::<Vec<_>>();
    assert_eq!(
        rowan_kinds,
        vec![
            SyntaxKind::Root,
            SyntaxKind::InfixExpression,
            SyntaxKind::Token,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::Token,
            SyntaxKind::TokenUserSymbol,
            SyntaxKind::Token,
            SyntaxKind::TokenIdentifier,
        ]
    );
    let rowan_tokens = ast
        .rowan_root()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .map(|token| (token.kind(), token.text().to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(
        rowan_tokens,
        vec![
            (SyntaxKind::TokenIdentifier, "a".to_owned()),
            (SyntaxKind::TokenUserSymbol, "++".to_owned()),
            (SyntaxKind::TokenIdentifier, "b".to_owned()),
        ]
    );
    assert_eq!(
        ast.rowan_root()
            .descendants_with_tokens()
            .filter(|element| element.as_token().is_some())
            .count(),
        3,
        "the rowan storage is source-shaped even while compatibility views keep dense token ids"
    );
    assert_eq!(ast.green_node(), expression_ast(source_id).green_node());
}

#[test]
fn typed_accessors_cover_current_node_and_token_kinds() {
    let source_id = source_id(2);
    let mut builder = SurfaceAstBuilder::new(source_id);
    let token_kinds = [
        SurfaceTokenKind::Identifier,
        SurfaceTokenKind::ReservedWord,
        SurfaceTokenKind::ReservedSymbol,
        SurfaceTokenKind::Numeral,
        SurfaceTokenKind::LexemeRun,
        SurfaceTokenKind::UserSymbol,
        SurfaceTokenKind::AnnotationMarker,
        SurfaceTokenKind::StringLiteral,
        SurfaceTokenKind::ErrorRecovery,
        SurfaceTokenKind::Unknown,
    ];
    let mut token_ids = Vec::new();
    for (index, kind) in token_kinds.into_iter().enumerate() {
        token_ids.push(builder.add_token(
            kind,
            format!("t{index}"),
            range(source_id, index, index + 1),
        ));
    }
    let recovered_token = builder.add_recovered_token(
        SurfaceTokenKind::ErrorRecovery,
        "bad",
        range(source_id, 20, 21),
    );
    let module_prefix_token = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "..",
        range(source_id, 30, 32),
    );
    let module_segment_a = builder.add_token(
        SurfaceTokenKind::Identifier,
        "std",
        range(source_id, 32, 35),
    );
    let module_dot_token = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 35, 36),
    );
    let module_segment_b = builder.add_token(
        SurfaceTokenKind::Identifier,
        "algebra",
        range(source_id, 36, 43),
    );
    let namespace_segment_a = builder.add_token(
        SurfaceTokenKind::Identifier,
        "mml",
        range(source_id, 44, 47),
    );
    let namespace_dot_token = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 47, 48),
    );
    let namespace_segment_b = builder.add_token(
        SurfaceTokenKind::Identifier,
        "nat",
        range(source_id, 48, 51),
    );
    let qualified_segment_a = builder.add_token(
        SurfaceTokenKind::Identifier,
        "algebra",
        range(source_id, 52, 59),
    );
    let qualified_dot_token = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 59, 60),
    );
    let qualified_symbol_token = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "Group",
        range(source_id, 60, 65),
    );
    let item_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 66, 73),
    );
    let item_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 73, 74),
    );
    let import_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "import",
        range(source_id, 75, 81),
    );
    let import_path_prefix = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 82, 83),
    );
    let import_path_tools = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Tools",
        range(source_id, 83, 88),
    );
    let import_branch_opener = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".{",
        range(source_id, 88, 90),
    );
    let import_branch_segment = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Group",
        range(source_id, 90, 95),
    );
    let import_branch_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "}",
        range(source_id, 95, 96),
    );
    let import_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 96, 97),
    );
    let import_alias_path = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Std",
        range(source_id, 98, 101),
    );
    let import_as = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "as",
        range(source_id, 102, 104),
    );
    let import_alias = builder.add_token(
        SurfaceTokenKind::Identifier,
        "G",
        range(source_id, 105, 106),
    );
    let import_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 106, 107),
    );
    let export_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "export",
        range(source_id, 108, 114),
    );
    let export_path_std = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Std",
        range(source_id, 115, 118),
    );
    let export_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 118, 119),
    );
    let visibility_public = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "public",
        range(source_id, 120, 126),
    );
    let visible_theorem = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 127, 134),
    );
    let visible_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 134, 135),
    );
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::ErrorToken,
        range(source_id, 9, 9),
        Vec::new(),
    );
    let infix = builder.add_node(
        SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
            spelling: "++".into(),
            precedence: 10,
            associativity: SurfaceOperatorAssociativity::Left,
        }),
        range(source_id, 0, 3),
        token_ids[..3].to_vec(),
    );
    let module_prefix = builder.add_node(
        SurfaceNodeKind::RelativePrefix,
        range(source_id, 30, 32),
        vec![module_prefix_token],
    );
    let module_path_segment_a = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 32, 35),
        vec![module_segment_a],
    );
    let module_path_segment_b = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 36, 43),
        vec![module_segment_b],
    );
    let module_path = builder.add_node(
        SurfaceNodeKind::ModulePath,
        range(source_id, 30, 43),
        vec![
            module_prefix,
            module_path_segment_a,
            module_dot_token,
            module_path_segment_b,
        ],
    );
    let namespace_path_segment_a = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 44, 47),
        vec![namespace_segment_a],
    );
    let namespace_path_segment_b = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 48, 51),
        vec![namespace_segment_b],
    );
    let namespace_path = builder.add_node(
        SurfaceNodeKind::NamespacePath,
        range(source_id, 44, 51),
        vec![
            namespace_path_segment_a,
            namespace_dot_token,
            namespace_path_segment_b,
        ],
    );
    let qualified_path_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 52, 59),
        vec![qualified_segment_a],
    );
    let qualified_final_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 60, 65),
        vec![qualified_symbol_token],
    );
    let qualified_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 52, 65),
        vec![
            qualified_path_segment,
            qualified_dot_token,
            qualified_final_segment,
        ],
    );
    let placeholder_item = builder.add_node(
        SurfaceNodeKind::PlaceholderItem,
        range(source_id, 66, 74),
        vec![item_keyword, item_semicolon],
    );
    let import_path_prefix_node = builder.add_node(
        SurfaceNodeKind::RelativePrefix,
        range(source_id, 82, 83),
        vec![import_path_prefix],
    );
    let import_path_tools_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 83, 88),
        vec![import_path_tools],
    );
    let import_branch_path = builder.add_node(
        SurfaceNodeKind::ModulePath,
        range(source_id, 82, 88),
        vec![import_path_prefix_node, import_path_tools_node],
    );
    let import_branch_segment_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 90, 95),
        vec![import_branch_segment],
    );
    let module_branch_import = builder.add_node(
        SurfaceNodeKind::ModuleBranchImport,
        range(source_id, 82, 96),
        vec![
            import_branch_path,
            import_branch_opener,
            import_branch_segment_node,
            import_branch_close,
        ],
    );
    let import_alias_path_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 98, 101),
        vec![import_alias_path],
    );
    let import_alias_module_path = builder.add_node(
        SurfaceNodeKind::ModulePath,
        range(source_id, 98, 101),
        vec![import_alias_path_node],
    );
    let import_alias_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 105, 106),
        vec![import_alias],
    );
    let import_alias_decl = builder.add_node(
        SurfaceNodeKind::ImportAliasDecl,
        range(source_id, 98, 106),
        vec![import_alias_module_path, import_as, import_alias_node],
    );
    let import_item = builder.add_node(
        SurfaceNodeKind::ImportItem,
        range(source_id, 75, 107),
        vec![
            import_keyword,
            module_branch_import,
            import_comma,
            import_alias_decl,
            import_semicolon,
        ],
    );
    let export_path_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 115, 118),
        vec![export_path_std],
    );
    let export_module_path = builder.add_node(
        SurfaceNodeKind::ModulePath,
        range(source_id, 115, 118),
        vec![export_path_segment],
    );
    let export_item = builder.add_node(
        SurfaceNodeKind::ExportItem,
        range(source_id, 108, 119),
        vec![export_keyword, export_module_path, export_semicolon],
    );
    let visibility_marker = builder.add_node(
        SurfaceNodeKind::VisibilityMarker,
        range(source_id, 120, 126),
        vec![visibility_public],
    );
    let visible_placeholder = builder.add_node(
        SurfaceNodeKind::PlaceholderItem,
        range(source_id, 127, 135),
        vec![visible_theorem, visible_semicolon],
    );
    let visible_item = builder.add_node(
        SurfaceNodeKind::VisibleItem,
        range(source_id, 120, 135),
        vec![visibility_marker, visible_placeholder],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 66, 135),
        vec![placeholder_item, import_item, export_item, visible_item],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 66, 135),
        vec![item_list],
    );
    let path_tokens = [
        module_prefix_token,
        module_segment_a,
        module_dot_token,
        module_segment_b,
        namespace_segment_a,
        namespace_dot_token,
        namespace_segment_b,
        qualified_segment_a,
        qualified_dot_token,
        qualified_symbol_token,
    ];
    let import_tokens = [
        import_keyword,
        import_path_prefix,
        import_path_tools,
        import_branch_opener,
        import_branch_segment,
        import_branch_close,
        import_comma,
        import_alias_path,
        import_as,
        import_alias,
        import_semicolon,
    ];
    let task7_tokens = [
        export_keyword,
        export_path_std,
        export_semicolon,
        visibility_public,
        visible_theorem,
        visible_semicolon,
    ];
    let root_children = token_ids
        .iter()
        .copied()
        .chain([recovered_token])
        .chain(path_tokens)
        .chain([item_keyword, item_semicolon])
        .chain(import_tokens)
        .chain(task7_tokens)
        .chain([
            infix,
            module_path,
            namespace_path,
            qualified_symbol,
            compilation_unit,
            recovery,
        ])
        .collect::<Vec<_>>();
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 135),
        root_children.clone(),
    );
    let ast = builder.finish(Some(root), Some(infix));

    let root_view = ast.root_view().unwrap();
    assert_eq!(root_view.id(), sid(root));
    assert_eq!(root_view.kind(), &SurfaceNodeKind::Root);
    assert_eq!(root_view.syntax_kind(), SyntaxKind::Root);
    assert_eq!(root_view.range(), range(source_id, 0, 135));
    assert!(!root_view.is_recovered());
    assert!(root_view.as_token().is_none());
    assert!(root_view.as_infix_expression().is_none());
    assert!(root_view.as_recovery().is_none());
    assert_eq!(
        root_view.children(),
        &root_children.iter().copied().map(sid).collect::<Vec<_>>()
    );
    assert_eq!(
        root_view
            .child_views()
            .map(super::SurfaceNodeView::id)
            .collect::<Vec<_>>(),
        root_view.children()
    );

    let expression_view = ast.expression_view().unwrap();
    let infix_operator = expression_view.as_infix_expression().unwrap();
    assert_eq!(expression_view.id(), sid(infix));
    assert_eq!(expression_view.syntax_kind(), SyntaxKind::InfixExpression);
    assert_eq!(expression_view.range(), range(source_id, 0, 3));
    assert_eq!(
        expression_view.children(),
        &token_ids[..3].iter().copied().map(sid).collect::<Vec<_>>()
    );
    assert_eq!(infix_operator.spelling.as_ref(), "++");
    assert_eq!(infix_operator.precedence, 10);
    assert_eq!(
        infix_operator.associativity,
        SurfaceOperatorAssociativity::Left
    );
    assert!(!expression_view.is_recovered());
    assert!(expression_view.as_token().is_none());
    assert!(expression_view.as_recovery().is_none());
    assert!(expression_view.as_module_path().is_none());
    assert!(expression_view.as_namespace_path().is_none());
    assert!(expression_view.as_qualified_symbol().is_none());

    let module_path_view = ast.node_view(sid(module_path)).unwrap();
    assert_eq!(module_path_view.syntax_kind(), SyntaxKind::ModulePath);
    assert_eq!(
        module_path_view.as_module_path().unwrap().id(),
        sid(module_path)
    );
    assert_eq!(module_path_view.range(), range(source_id, 30, 43));
    assert_eq!(
        module_path_view.children(),
        &[
            sid(module_prefix),
            sid(module_path_segment_a),
            sid(module_dot_token),
            sid(module_path_segment_b),
        ]
    );
    assert!(module_path_view.as_token().is_none());
    assert!(module_path_view.as_infix_expression().is_none());
    assert!(module_path_view.as_recovery().is_none());

    let namespace_path_view = ast.node_view(sid(namespace_path)).unwrap();
    assert_eq!(namespace_path_view.syntax_kind(), SyntaxKind::NamespacePath);
    assert_eq!(
        namespace_path_view.as_namespace_path().unwrap().id(),
        sid(namespace_path)
    );
    assert_eq!(
        namespace_path_view.children(),
        &[
            sid(namespace_path_segment_a),
            sid(namespace_dot_token),
            sid(namespace_path_segment_b),
        ]
    );

    let qualified_symbol_view = ast.node_view(sid(qualified_symbol)).unwrap();
    assert_eq!(
        qualified_symbol_view.syntax_kind(),
        SyntaxKind::QualifiedSymbol
    );
    assert_eq!(
        qualified_symbol_view.as_qualified_symbol().unwrap().id(),
        sid(qualified_symbol)
    );
    assert_eq!(
        qualified_symbol_view.children(),
        &[
            sid(qualified_path_segment),
            sid(qualified_dot_token),
            sid(qualified_final_segment),
        ]
    );

    let compilation_unit_view = ast.node_view(sid(compilation_unit)).unwrap();
    assert_eq!(
        compilation_unit_view.syntax_kind(),
        SyntaxKind::CompilationUnit
    );
    assert_eq!(
        compilation_unit_view
            .as_compilation_unit()
            .unwrap()
            .children(),
        &[sid(item_list)]
    );
    assert!(compilation_unit_view.as_token().is_none());
    assert!(compilation_unit_view.as_infix_expression().is_none());
    assert!(compilation_unit_view.as_recovery().is_none());
    let item_list_view = ast.node_view(sid(item_list)).unwrap();
    assert_eq!(item_list_view.syntax_kind(), SyntaxKind::ItemList);
    assert_eq!(
        item_list_view.as_item_list().unwrap().children(),
        &[
            sid(placeholder_item),
            sid(import_item),
            sid(export_item),
            sid(visible_item)
        ]
    );
    let placeholder_item_view = ast.node_view(sid(placeholder_item)).unwrap();
    assert_eq!(
        placeholder_item_view.syntax_kind(),
        SyntaxKind::PlaceholderItem
    );
    assert_eq!(
        placeholder_item_view
            .as_placeholder_item()
            .unwrap()
            .children(),
        &[sid(item_keyword), sid(item_semicolon)]
    );
    let import_item_view = ast.node_view(sid(import_item)).unwrap();
    assert_eq!(import_item_view.syntax_kind(), SyntaxKind::ImportItem);
    assert_eq!(
        import_item_view.as_import_item().unwrap().children(),
        &[
            sid(import_keyword),
            sid(module_branch_import),
            sid(import_comma),
            sid(import_alias_decl),
            sid(import_semicolon),
        ]
    );
    let branch_import_view = ast.node_view(sid(module_branch_import)).unwrap();
    assert_eq!(
        branch_import_view.syntax_kind(),
        SyntaxKind::ModuleBranchImport
    );
    assert_eq!(
        branch_import_view
            .as_module_branch_import()
            .unwrap()
            .children(),
        &[
            sid(import_branch_path),
            sid(import_branch_opener),
            sid(import_branch_segment_node),
            sid(import_branch_close),
        ]
    );
    let import_alias_view = ast.node_view(sid(import_alias_decl)).unwrap();
    assert_eq!(import_alias_view.syntax_kind(), SyntaxKind::ImportAliasDecl);
    assert_eq!(
        import_alias_view.as_import_alias_decl().unwrap().children(),
        &[
            sid(import_alias_module_path),
            sid(import_as),
            sid(import_alias_node)
        ]
    );
    let export_item_view = ast.node_view(sid(export_item)).unwrap();
    assert_eq!(export_item_view.syntax_kind(), SyntaxKind::ExportItem);
    assert_eq!(
        export_item_view.as_export_item().unwrap().children(),
        &[
            sid(export_keyword),
            sid(export_module_path),
            sid(export_semicolon)
        ]
    );
    let visibility_marker_view = ast.node_view(sid(visibility_marker)).unwrap();
    assert_eq!(
        visibility_marker_view.syntax_kind(),
        SyntaxKind::VisibilityMarker
    );
    assert_eq!(
        visibility_marker_view
            .as_visibility_marker()
            .unwrap()
            .children(),
        &[sid(visibility_public)]
    );
    let visible_item_view = ast.node_view(sid(visible_item)).unwrap();
    assert_eq!(visible_item_view.syntax_kind(), SyntaxKind::VisibleItem);
    assert_eq!(
        visible_item_view.as_visible_item().unwrap().children(),
        &[sid(visibility_marker), sid(visible_placeholder)]
    );

    let module_segment_view = ast.node_view(sid(module_path_segment_a)).unwrap();
    assert_eq!(module_segment_view.syntax_kind(), SyntaxKind::PathSegment);
    assert_eq!(
        module_segment_view.as_path_segment().unwrap().children(),
        &[sid(module_segment_a)]
    );
    let prefix_view = ast.node_view(sid(module_prefix)).unwrap();
    assert_eq!(prefix_view.syntax_kind(), SyntaxKind::RelativePrefix);
    assert_eq!(
        prefix_view.as_relative_prefix().unwrap().children(),
        &[sid(module_prefix_token)]
    );

    let recovery_view = ast.node_view(sid(recovery)).unwrap();
    assert_eq!(
        recovery_view.as_recovery(),
        Some(SyntaxRecoveryKind::ErrorToken)
    );
    assert_eq!(recovery_view.syntax_kind(), SyntaxKind::ErrorRecovery);
    assert_eq!(recovery_view.range(), range(source_id, 9, 9));
    assert!(recovery_view.is_recovered());
    assert!(recovery_view.children().is_empty());
    assert!(recovery_view.as_token().is_none());
    assert!(recovery_view.as_infix_expression().is_none());

    let recovered_token_view = ast.node_view(sid(recovered_token)).unwrap();
    assert!(recovered_token_view.is_recovered());
    assert_eq!(recovered_token_view.range(), range(source_id, 20, 21));
    assert_eq!(
        recovered_token_view.as_token().unwrap().text.as_ref(),
        "bad"
    );
    assert!(recovered_token_view.as_infix_expression().is_none());
    assert!(recovered_token_view.as_recovery().is_none());

    let actual_token_kinds = ast
        .token_views()
        .map(|view| view.as_token().unwrap().kind.syntax_kind())
        .collect::<Vec<_>>();
    assert_eq!(
        actual_token_kinds,
        vec![
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenReservedWord,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenNumeral,
            SyntaxKind::TokenLexemeRun,
            SyntaxKind::TokenUserSymbol,
            SyntaxKind::TokenAnnotationMarker,
            SyntaxKind::TokenStringLiteral,
            SyntaxKind::TokenErrorRecovery,
            SyntaxKind::TokenUnknown,
            SyntaxKind::TokenErrorRecovery,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenUserSymbol,
            SyntaxKind::TokenReservedWord,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenReservedWord,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenReservedWord,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenReservedWord,
            SyntaxKind::TokenIdentifier,
            SyntaxKind::TokenReservedSymbol,
            SyntaxKind::TokenReservedWord,
            SyntaxKind::TokenReservedWord,
            SyntaxKind::TokenReservedSymbol,
        ]
    );
    for (index, token_view) in ast.token_views().take(token_kinds.len()).enumerate() {
        assert_eq!(token_view.id(), sid(token_ids[index]));
        assert_eq!(token_view.syntax_kind(), SyntaxKind::Token);
        assert_eq!(token_view.range(), range(source_id, index, index + 1));
        assert_eq!(
            token_view.as_token().unwrap().text.as_ref(),
            format!("t{index}")
        );
        assert!(!token_view.is_recovered());
        assert!(token_view.children().is_empty());
        assert!(token_view.as_infix_expression().is_none());
        assert!(token_view.as_recovery().is_none());
    }
}

#[test]
fn surface_node_raw_kinds_round_trip_through_rowan_boundary() {
    let mut rowan_kinds = current_vocabulary_snapshot_ast(source_id(21))
        .rowan_root()
        .descendants_with_tokens()
        .map(|element| element.kind())
        .collect::<Vec<_>>();
    rowan_kinds.extend(
        atomic_formula_nodes_ast(source_id(25))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        formula_surface_nodes_ast(source_id(26))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        set_comprehension_nodes_ast(source_id(27))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        statement_nodes_ast(source_id(28))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        justification_nodes_ast(source_id(29))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task18_statement_nodes_ast(source_id(30))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task19_statement_nodes_ast(source_id(31))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task20_statement_nodes_ast(source_id(32))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task21_statement_nodes_ast(source_id(33))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task22_theorem_nodes_ast(source_id(34))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task23_definition_nodes_ast(source_id(35))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task25_functor_definition_nodes_ast(source_id(36))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task26_mode_definition_nodes_ast(source_id(38))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task27_redefinition_notation_nodes_ast(source_id(40))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task28_property_clause_nodes_ast(source_id(42))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task29_structure_nodes_ast(source_id(44))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task30_registration_nodes_ast(source_id(46))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task31_template_nodes_ast(source_id(48))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task32_algorithm_nodes_ast(source_id(50))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task33_algorithm_control_flow_nodes_ast(source_id(52))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task34_algorithm_verification_nodes_ast(source_id(54))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task35_annotation_nodes_ast(source_id(56))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );
    rowan_kinds.extend(
        task48_property_implementation_ast(source_id(57))
            .rowan_root()
            .descendants_with_tokens()
            .map(|element| element.kind()),
    );

    for kind in [
        SyntaxKind::CompilationUnit,
        SyntaxKind::ItemList,
        SyntaxKind::PlaceholderItem,
        SyntaxKind::ImportItem,
        SyntaxKind::ImportAliasDecl,
        SyntaxKind::ModuleBranchImport,
        SyntaxKind::ExportItem,
        SyntaxKind::VisibilityMarker,
        SyntaxKind::VisibleItem,
        SyntaxKind::ReserveItem,
        SyntaxKind::ReserveSegment,
        SyntaxKind::TypeExpression,
        SyntaxKind::AttributeChain,
        SyntaxKind::AttributeRef,
        SyntaxKind::ParameterPrefix,
        SyntaxKind::TypeHead,
        SyntaxKind::TypeArguments,
        SyntaxKind::TermPlaceholder,
        SyntaxKind::TermExpression,
        SyntaxKind::TermReference,
        SyntaxKind::NumeralTerm,
        SyntaxKind::ItTerm,
        SyntaxKind::ParenthesizedTerm,
        SyntaxKind::ChoiceTerm,
        SyntaxKind::ApplicationTerm,
        SyntaxKind::StructureConstructor,
        SyntaxKind::FieldArgument,
        SyntaxKind::SetEnumeration,
        SyntaxKind::SetComprehension,
        SyntaxKind::ComprehensionVariableSegment,
        SyntaxKind::StatementItem,
        SyntaxKind::LetStatement,
        SyntaxKind::QualifiedVariableSegment,
        SyntaxKind::AssumptionStatement,
        SyntaxKind::Proposition,
        SyntaxKind::ConditionList,
        SyntaxKind::GivenStatement,
        SyntaxKind::TakeStatement,
        SyntaxKind::Witness,
        SyntaxKind::SetStatement,
        SyntaxKind::Equating,
        SyntaxKind::CompactStatement,
        SyntaxKind::JustificationClause,
        SyntaxKind::ReferenceList,
        SyntaxKind::Reference,
        SyntaxKind::QualifiedReference,
        SyntaxKind::GroupedReference,
        SyntaxKind::GroupedReferenceItem,
        SyntaxKind::BulkReference,
        SyntaxKind::ComputationJustification,
        SyntaxKind::ComputationOption,
        SyntaxKind::ConsiderStatement,
        SyntaxKind::ReconsiderStatement,
        SyntaxKind::ReconsiderItem,
        SyntaxKind::ConclusionStatement,
        SyntaxKind::ThenStatement,
        SyntaxKind::IterativeEqualityStatement,
        SyntaxKind::IterativeEqualityStep,
        SyntaxKind::NowStatement,
        SyntaxKind::HerebyStatement,
        SyntaxKind::CaseReasoningStatement,
        SyntaxKind::CaseItem,
        SyntaxKind::SupposeItem,
        SyntaxKind::InlineFunctorDefinition,
        SyntaxKind::InlinePredicateDefinition,
        SyntaxKind::TypedParameter,
        SyntaxKind::TheoremItem,
        SyntaxKind::LemmaItem,
        SyntaxKind::ProofBlock,
        SyntaxKind::DefinitionBlockItem,
        SyntaxKind::PropertyImplementation,
        SyntaxKind::DefinitionParameter,
        SyntaxKind::AttributeDefinition,
        SyntaxKind::AttributePattern,
        SyntaxKind::FormulaDefiniens,
        SyntaxKind::FormulaCase,
        SyntaxKind::CorrectnessCondition,
        SyntaxKind::PredicateDefinition,
        SyntaxKind::PredicatePattern,
        SyntaxKind::FunctorDefinition,
        SyntaxKind::FunctorPattern,
        SyntaxKind::TermDefiniens,
        SyntaxKind::TermCase,
        SyntaxKind::ModeDefinition,
        SyntaxKind::ModePattern,
        SyntaxKind::ModeProperty,
        SyntaxKind::AttributeRedefinition,
        SyntaxKind::PredicateRedefinition,
        SyntaxKind::FunctorRedefinition,
        SyntaxKind::CoherenceCondition,
        SyntaxKind::NotationAlias,
        SyntaxKind::NotationPattern,
        SyntaxKind::PropertyClause,
        SyntaxKind::StructureDefinition,
        SyntaxKind::StructurePattern,
        SyntaxKind::StructureField,
        SyntaxKind::StructureProperty,
        SyntaxKind::InheritanceDefinition,
        SyntaxKind::InheritanceTarget,
        SyntaxKind::FieldRedefinition,
        SyntaxKind::PropertyRedefinition,
        SyntaxKind::RegistrationBlockItem,
        SyntaxKind::RegistrationParameter,
        SyntaxKind::ExistentialRegistration,
        SyntaxKind::ConditionalRegistration,
        SyntaxKind::FunctorialRegistration,
        SyntaxKind::ReductionRegistration,
        SyntaxKind::TemplateParameter,
        SyntaxKind::TemplateLoci,
        SyntaxKind::TemplateLocus,
        SyntaxKind::TemplateArguments,
        SyntaxKind::TemplateArgument,
        SyntaxKind::AlgorithmDefinition,
        SyntaxKind::AlgorithmParameters,
        SyntaxKind::AlgorithmBody,
        SyntaxKind::AlgorithmStatementList,
        SyntaxKind::VariableDeclaration,
        SyntaxKind::VariableBinding,
        SyntaxKind::AssignmentStatement,
        SyntaxKind::Lvalue,
        SyntaxKind::SnapshotStatement,
        SyntaxKind::ReturnStatement,
        SyntaxKind::ClaimBlockItem,
        SyntaxKind::IfStatement,
        SyntaxKind::WhileStatement,
        SyntaxKind::ForRangeStatement,
        SyntaxKind::ForCollectionStatement,
        SyntaxKind::MatchStatement,
        SyntaxKind::MatchCase,
        SyntaxKind::MatchEnding,
        SyntaxKind::BreakStatement,
        SyntaxKind::ContinueStatement,
        SyntaxKind::AlgorithmTerminationClause,
        SyntaxKind::AlgorithmRequiresClause,
        SyntaxKind::AlgorithmEnsuresClause,
        SyntaxKind::AlgorithmDecreasingClause,
        SyntaxKind::LoopInvariantClause,
        SyntaxKind::LoopDecreasingClause,
        SyntaxKind::AssertStatement,
        SyntaxKind::TermList,
        SyntaxKind::Annotation,
        SyntaxKind::LibraryAnnotation,
        SyntaxKind::AnnotationLabelList,
        SyntaxKind::AnnotationLabel,
        SyntaxKind::AnnotationArgumentList,
        SyntaxKind::AnnotationArgument,
        SyntaxKind::ProofHintOptionList,
        SyntaxKind::ProofHintOption,
        SyntaxKind::StandaloneDiagnosticAnnotation,
        SyntaxKind::AnnotatedStatement,
        SyntaxKind::AnnotatedAlgorithmStatement,
        SyntaxKind::AnnotatedDefinitionContent,
        SyntaxKind::AnnotatedRegistrationContent,
        SyntaxKind::SelectorAccess,
        SyntaxKind::StructureUpdate,
        SyntaxKind::FieldUpdate,
        SyntaxKind::QuaExpression,
        SyntaxKind::FormulaExpression,
        SyntaxKind::BuiltinPredicateApplication,
        SyntaxKind::IsAssertion,
        SyntaxKind::AttributeTestChain,
        SyntaxKind::PredicateApplication,
        SyntaxKind::PredicateSegment,
        SyntaxKind::PredicateHead,
        SyntaxKind::InlinePredicateApplication,
        SyntaxKind::PrefixFormula,
        SyntaxKind::BinaryFormula,
        SyntaxKind::ParenthesizedFormula,
        SyntaxKind::QuantifiedFormula,
        SyntaxKind::QuantifierVariableSegment,
        SyntaxKind::FormulaConstant,
        SyntaxKind::ModulePath,
        SyntaxKind::NamespacePath,
        SyntaxKind::QualifiedSymbol,
        SyntaxKind::PathSegment,
        SyntaxKind::RelativePrefix,
    ] {
        assert_eq!(SyntaxKind::from_raw(kind as u16), kind);
        assert!(kind.is_node_kind());
        assert!(!kind.is_token_kind());
        assert!(
            rowan_kinds.contains(&kind),
            "rowan tree should emit {kind:?} for current structural nodes"
        );
    }

    for kind in [
        SyntaxKind::TokenAnnotationMarker,
        SyntaxKind::TokenIdentifier,
        SyntaxKind::TokenReservedWord,
        SyntaxKind::TokenReservedSymbol,
        SyntaxKind::TokenNumeral,
        SyntaxKind::TokenLexemeRun,
        SyntaxKind::TokenUserSymbol,
        SyntaxKind::TokenStringLiteral,
        SyntaxKind::TokenErrorRecovery,
        SyntaxKind::TokenUnknown,
    ] {
        assert_eq!(SyntaxKind::from_raw(kind as u16), kind);
        assert!(!kind.is_node_kind());
        assert!(kind.is_token_kind());
        assert!(
            rowan_kinds.contains(&kind),
            "rowan tree should emit {kind:?} for current token leaves"
        );
    }
}

#[test]
fn task8_typed_accessors_cover_type_expression_nodes() {
    let source_id = source_id(23);
    let mut builder = SurfaceAstBuilder::new(source_id);
    let reserve = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "reserve",
        range(source_id, 0, 7),
    );
    let identifier = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 8, 9));
    let for_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "for",
        range(source_id, 10, 13),
    );
    let non = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "non",
        range(source_id, 14, 17),
    );
    let n = builder.add_token(SurfaceTokenKind::Identifier, "n", range(source_id, 18, 19));
    let hyphen = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "-",
        range(source_id, 19, 20),
    );
    let empty = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "empty",
        range(source_id, 20, 25),
    );
    let type_symbol =
        builder.add_token(SurfaceTokenKind::UserSymbol, "T", range(source_id, 26, 27));
    let of = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "of",
        range(source_id, 28, 30),
    );
    let term = builder.add_token(SurfaceTokenKind::Identifier, "a", range(source_id, 31, 32));
    let semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 32, 33),
    );

    let prefix = builder.add_node(
        SurfaceNodeKind::ParameterPrefix,
        range(source_id, 18, 20),
        vec![n, hyphen],
    );
    let empty_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 20, 25),
        vec![empty],
    );
    let empty_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 20, 25),
        vec![empty_segment],
    );
    let attribute = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(source_id, 14, 25),
        vec![non, prefix, empty_symbol],
    );
    let attribute_chain = builder.add_node(
        SurfaceNodeKind::AttributeChain,
        range(source_id, 14, 25),
        vec![attribute],
    );
    let type_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 26, 27),
        vec![type_symbol],
    );
    let type_symbol_node = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 26, 27),
        vec![type_segment],
    );
    let term_placeholder = builder.add_node(
        SurfaceNodeKind::TermPlaceholder,
        range(source_id, 31, 32),
        vec![term],
    );
    let type_arguments = builder.add_node(
        SurfaceNodeKind::TypeArguments,
        range(source_id, 28, 32),
        vec![of, term_placeholder],
    );
    let type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 26, 32),
        vec![type_symbol_node, type_arguments],
    );
    let type_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 14, 32),
        vec![attribute_chain, type_head],
    );
    let reserve_segment = builder.add_node(
        SurfaceNodeKind::ReserveSegment,
        range(source_id, 8, 32),
        vec![identifier, for_keyword, type_expression],
    );
    let reserve_item = builder.add_node(
        SurfaceNodeKind::ReserveItem,
        range(source_id, 0, 33),
        vec![reserve, reserve_segment, semicolon],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 33),
        vec![
            reserve,
            identifier,
            for_keyword,
            non,
            n,
            hyphen,
            empty,
            type_symbol,
            of,
            term,
            semicolon,
            reserve_item,
        ],
    );
    let ast = builder.finish(Some(root), None);

    assert!(
        ast.node_view(sid(reserve_item))
            .unwrap()
            .as_reserve_item()
            .is_some()
    );
    assert!(
        ast.node_view(sid(reserve_segment))
            .unwrap()
            .as_reserve_segment()
            .is_some()
    );
    assert!(
        ast.node_view(sid(type_expression))
            .unwrap()
            .as_type_expression()
            .is_some()
    );
    assert!(
        ast.node_view(sid(attribute_chain))
            .unwrap()
            .as_attribute_chain()
            .is_some()
    );
    assert!(
        ast.node_view(sid(attribute))
            .unwrap()
            .as_attribute_ref()
            .is_some()
    );
    assert!(
        ast.node_view(sid(prefix))
            .unwrap()
            .as_parameter_prefix()
            .is_some()
    );
    assert!(
        ast.node_view(sid(type_head))
            .unwrap()
            .as_type_head()
            .is_some()
    );
    assert!(
        ast.node_view(sid(type_arguments))
            .unwrap()
            .as_type_arguments()
            .is_some()
    );
    assert!(
        ast.node_view(sid(term_placeholder))
            .unwrap()
            .as_term_placeholder()
            .is_some()
    );
}

#[test]
fn task9_typed_accessors_cover_primary_term_nodes() {
    let source_id = source_id(24);
    let mut builder = SurfaceAstBuilder::new(source_id);
    let identifier = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 0, 1));
    let numeral = builder.add_token(SurfaceTokenKind::Numeral, "42", range(source_id, 2, 4));
    let it = builder.add_token(SurfaceTokenKind::ReservedWord, "it", range(source_id, 5, 7));
    let open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 8, 9),
    );
    let paren_identifier =
        builder.add_token(SurfaceTokenKind::Identifier, "p", range(source_id, 9, 10));
    let close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 10, 11),
    );
    let the = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "the",
        range(source_id, 12, 15),
    );
    let set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 16, 19),
    );
    let function = builder.add_token(SurfaceTokenKind::UserSymbol, "F", range(source_id, 20, 21));
    let app_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 21, 22),
    );
    let app_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 22, 23),
    );
    let structure = builder.add_token(SurfaceTokenKind::UserSymbol, "S", range(source_id, 24, 25));
    let struct_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 25, 26),
    );
    let field = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 26, 27));
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 27, 28),
    );
    let value = builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 28, 29));
    let struct_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 29, 30),
    );
    let set_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "{",
        range(source_id, 31, 32),
    );
    let set_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "}",
        range(source_id, 32, 33),
    );

    let term_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 0, 1),
        vec![identifier],
    );
    let term_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 0, 1),
        vec![term_reference],
    );
    let numeral_term = builder.add_node(
        SurfaceNodeKind::NumeralTerm,
        range(source_id, 2, 4),
        vec![numeral],
    );
    let it_term = builder.add_node(SurfaceNodeKind::ItTerm, range(source_id, 5, 7), vec![it]);
    let paren_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 9, 10),
        vec![paren_identifier],
    );
    let paren_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 9, 10),
        vec![paren_reference],
    );
    let paren_term = builder.add_node(
        SurfaceNodeKind::ParenthesizedTerm,
        range(source_id, 8, 11),
        vec![open, paren_expression, close],
    );
    let type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 16, 19),
        vec![set],
    );
    let type_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 16, 19),
        vec![type_head],
    );
    let choice_term = builder.add_node(
        SurfaceNodeKind::ChoiceTerm,
        range(source_id, 12, 19),
        vec![the, type_expression],
    );
    let function_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 20, 21),
        vec![function],
    );
    let function_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 20, 21),
        vec![function_segment],
    );
    let function_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 20, 21),
        vec![function_symbol],
    );
    let application = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        range(source_id, 20, 23),
        vec![function_reference, app_open, app_close],
    );
    let structure_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 24, 25),
        vec![structure],
    );
    let structure_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 24, 25),
        vec![structure_segment],
    );
    let value_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 28, 29),
        vec![value],
    );
    let value_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 28, 29),
        vec![value_reference],
    );
    let field_argument = builder.add_node(
        SurfaceNodeKind::FieldArgument,
        range(source_id, 26, 29),
        vec![field, colon, value_expression],
    );
    let structure_constructor = builder.add_node(
        SurfaceNodeKind::StructureConstructor,
        range(source_id, 24, 30),
        vec![structure_symbol, struct_open, field_argument, struct_close],
    );
    let set_enumeration = builder.add_node(
        SurfaceNodeKind::SetEnumeration,
        range(source_id, 31, 33),
        vec![set_open, set_close],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 33),
        vec![
            identifier,
            numeral,
            it,
            open,
            paren_identifier,
            close,
            the,
            set,
            function,
            app_open,
            app_close,
            structure,
            struct_open,
            field,
            colon,
            value,
            struct_close,
            set_open,
            set_close,
            term_expression,
            numeral_term,
            it_term,
            paren_term,
            choice_term,
            application,
            structure_constructor,
            set_enumeration,
        ],
    );
    let ast = builder.finish(Some(root), None);

    assert!(
        ast.node_view(sid(term_expression))
            .unwrap()
            .as_term_expression()
            .is_some()
    );
    assert!(
        ast.node_view(sid(term_reference))
            .unwrap()
            .as_term_reference()
            .is_some()
    );
    assert!(
        ast.node_view(sid(numeral_term))
            .unwrap()
            .as_numeral_term()
            .is_some()
    );
    assert!(ast.node_view(sid(it_term)).unwrap().as_it_term().is_some());
    assert!(
        ast.node_view(sid(paren_term))
            .unwrap()
            .as_parenthesized_term()
            .is_some()
    );
    assert!(
        ast.node_view(sid(choice_term))
            .unwrap()
            .as_choice_term()
            .is_some()
    );
    assert!(
        ast.node_view(sid(application))
            .unwrap()
            .as_application_term()
            .is_some()
    );
    assert!(
        ast.node_view(sid(structure_constructor))
            .unwrap()
            .as_structure_constructor()
            .is_some()
    );
    assert!(
        ast.node_view(sid(field_argument))
            .unwrap()
            .as_field_argument()
            .is_some()
    );
    assert!(
        ast.node_view(sid(set_enumeration))
            .unwrap()
            .as_set_enumeration()
            .is_some()
    );
}

#[test]
fn task10_typed_accessors_cover_selector_and_update_nodes() {
    let source_id = source_id(25);
    let mut builder = SurfaceAstBuilder::new(source_id);
    let base = builder.add_token(SurfaceTokenKind::Identifier, "p", range(source_id, 0, 1));
    let dot = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 1, 2),
    );
    let field = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 2, 3));
    let with = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "with",
        range(source_id, 4, 8),
    );
    let open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 9, 10),
    );
    let target = builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 10, 11));
    let assign = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":=",
        range(source_id, 12, 14),
    );
    let value = builder.add_token(SurfaceTokenKind::Identifier, "z", range(source_id, 15, 16));
    let close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 16, 17),
    );

    let base_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 0, 1),
        vec![base],
    );
    let selector_access = builder.add_node(
        SurfaceNodeKind::SelectorAccess,
        range(source_id, 0, 3),
        vec![base_reference, dot, field],
    );
    let value_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 15, 16),
        vec![value],
    );
    let value_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 15, 16),
        vec![value_reference],
    );
    let field_update = builder.add_node(
        SurfaceNodeKind::FieldUpdate,
        range(source_id, 10, 16),
        vec![target, assign, value_expression],
    );
    let structure_update = builder.add_node(
        SurfaceNodeKind::StructureUpdate,
        range(source_id, 0, 17),
        vec![selector_access, with, open, field_update, close],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 17),
        vec![
            base,
            dot,
            field,
            with,
            open,
            target,
            assign,
            value,
            close,
            structure_update,
        ],
    );
    let ast = builder.finish(Some(root), None);

    assert!(
        ast.node_view(sid(selector_access))
            .unwrap()
            .as_selector_access()
            .is_some()
    );
    assert!(
        ast.node_view(sid(structure_update))
            .unwrap()
            .as_structure_update()
            .is_some()
    );
    assert!(
        ast.node_view(sid(field_update))
            .unwrap()
            .as_field_update()
            .is_some()
    );
}

#[test]
fn task11_typed_accessor_covers_qua_expression() {
    let source_id = source_id(26);
    let mut builder = SurfaceAstBuilder::new(source_id);
    let base = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 0, 1));
    let qua = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "qua",
        range(source_id, 2, 5),
    );
    let target = builder.add_token(SurfaceTokenKind::UserSymbol, "T", range(source_id, 6, 7));

    let base_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 0, 1),
        vec![base],
    );
    let target_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 6, 7),
        vec![target],
    );
    let target_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 6, 7),
        vec![target_segment],
    );
    let target_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 6, 7),
        vec![target_symbol],
    );
    let target_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 6, 7),
        vec![target_head],
    );
    let qua_expression = builder.add_node(
        SurfaceNodeKind::QuaExpression,
        range(source_id, 0, 7),
        vec![base_reference, qua, target_expression],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 7),
        vec![base, qua, target, qua_expression],
    );
    let ast = builder.finish(Some(root), None);

    assert!(
        ast.node_view(sid(qua_expression))
            .unwrap()
            .as_qua_expression()
            .is_some()
    );
}

#[test]
fn parent_ranges_contain_child_ranges_except_recovery_attachments() {
    let source_id = source_id(3);
    let ast = expression_ast(source_id);
    let expression = ast.expression_root().unwrap();
    let root = ast.root().unwrap();

    assert_eq!(ast.range_contains_child_ranges(expression), Some(true));
    assert_eq!(ast.range_contains_child_ranges(root), Some(true));

    let mut builder = SurfaceAstBuilder::new(source_id);
    let opener = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "definition",
        range(source_id, 0, 10),
    );
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::MissingEnd,
        range(source_id, 10, 10),
        vec![opener],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 10),
        vec![opener, recovery],
    );
    let recovered_ast = builder.finish(Some(root), None);

    for node_id in [opener, root].map(sid) {
        assert_eq!(
            recovered_ast.range_contains_child_ranges(node_id),
            Some(true),
            "ordinary non-recovery node {node_id:?} should contain all child ranges"
        );
    }
    assert_eq!(
        recovered_ast.range_contains_child_ranges(sid(recovery)),
        Some(false),
        "missing-end recovery attaches the opener as context even though the zero-width insertion range does not contain it"
    );

    let mut missing_string_builder = SurfaceAstBuilder::new(source_id);
    let missing_string = missing_string_builder.add_recovery(
        SyntaxRecoveryKind::MissingStringLiteral,
        range(source_id, 12, 12),
        Vec::new(),
    );
    let root = missing_string_builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 12, 12),
        vec![missing_string],
    );
    let missing_string_ast = missing_string_builder.finish(Some(root), None);
    assert_eq!(
        missing_string_ast.range_contains_child_ranges(sid(missing_string)),
        Some(true),
        "zero-width missing string recovery without context children still satisfies containment"
    );
    assert_eq!(
        missing_string_ast.range_contains_child_ranges(sid(root)),
        Some(true)
    );
}

#[test]
fn recovery_kinds_are_constructible_with_documented_ranges() {
    let source_id = source_id(4);

    for fixture in recovery_fixtures(source_id) {
        let mut builder = SurfaceAstBuilder::new(source_id);
        let context = fixture.has_context_child.then(|| {
            builder.add_token(
                SurfaceTokenKind::ReservedWord,
                "context",
                range(source_id, 0, 7),
            )
        });
        let recovery_children = context.into_iter().collect::<Vec<_>>();
        let recovery = builder.add_recovery(fixture.kind, fixture.range, recovery_children.clone());
        let root_children = recovery_children
            .iter()
            .copied()
            .chain([recovery])
            .collect::<Vec<_>>();
        let root = builder.add_node(
            SurfaceNodeKind::Root,
            range(source_id, 0, 40),
            root_children,
        );
        let ast = builder.finish(Some(root), None);
        let recovery_view = ast.node_view(sid(recovery)).unwrap();

        assert_eq!(recovery_view.as_recovery(), Some(fixture.kind));
        assert_eq!(recovery_view.syntax_kind(), SyntaxKind::ErrorRecovery);
        assert_eq!(recovery_view.range(), fixture.range);
        assert!(recovery_view.is_recovered());
        assert_eq!(
            recovery_view.children(),
            &recovery_children
                .iter()
                .copied()
                .map(sid)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            ast.range_contains_child_ranges(sid(recovery)),
            Some(!fixture.has_context_child),
            "{:?} should follow its documented context-child range rule",
            fixture.kind
        );

        let snapshot_line = format!(
            "ErrorRecovery kind={} range={}..{} recovered=true",
            super::recovery_snapshot_name(fixture.kind),
            fixture.range.start,
            fixture.range.end
        );
        assert!(
            ast.snapshot_text().contains(&snapshot_line),
            "snapshot should render {:?} with its distinct recovery kind name",
            fixture.kind
        );
    }
}

#[test]
fn repeated_construction_produces_deterministic_green_tree_and_views() {
    let source_id = source_id(5);
    let first = expression_ast(source_id);
    let second = expression_ast(source_id);

    assert_eq!(first.green_node(), second.green_node());
    assert_eq!(first.nodes(), second.nodes());
    assert_eq!(first.token_nodes(), second.token_nodes());
    assert_eq!(first.expression_root(), second.expression_root());
}

#[test]
fn repeated_snapshot_rendering_is_byte_identical() {
    let ast = expression_ast(source_id(6));

    assert_eq!(ast.snapshot_text(), ast.snapshot_text());
    assert_eq!(
        ast.snapshot_text(),
        expression_ast(source_id(6)).snapshot_text()
    );
}

#[test]
fn snapshot_rendering_matches_current_vocabulary_baseline() {
    const EXPECTED: &str = include_str!(
        "../../../../tests/snapshots/mizar_syntax_surface_ast_current_vocabulary.snap"
    );
    let ast = current_vocabulary_snapshot_ast(source_id(7));
    let actual = ast.snapshot_text();

    assert_eq!(actual, EXPECTED);
    assert!(actual.contains("ErrorRecovery kind=MissingEnd range=89..89 recovered=true"));
    assert!(actual.contains("text=\"line\\nvalue\""));
    assert!(
        !actual.contains("SourceId"),
        "snapshot text must not expose opaque source-id debug output"
    );
}

#[test]
fn snapshot_payload_names_cover_current_variants() {
    let source_id = source_id(8);

    for (associativity, expected) in [
        (SurfaceOperatorAssociativity::Left, "associativity=Left"),
        (SurfaceOperatorAssociativity::Right, "associativity=Right"),
        (
            SurfaceOperatorAssociativity::NonAssociative,
            "associativity=NonAssociative",
        ),
    ] {
        let ast = expression_ast_with_associativity(source_id, associativity);

        assert!(ast.snapshot_text().contains(expected));
    }

    for recovery_kind in all_recovery_kinds() {
        let expected = format!("kind={}", super::recovery_snapshot_name(recovery_kind));
        let ast = recovery_ast(source_id, recovery_kind);

        assert!(ast.snapshot_text().contains(&expected));
    }

    let mut recovery_snapshot_names = std::collections::BTreeSet::new();
    for recovery_kind in all_recovery_kinds() {
        assert!(
            recovery_snapshot_names.insert(super::recovery_snapshot_name(recovery_kind)),
            "recovery snapshot names must distinguish every SyntaxRecoveryKind"
        );
    }
}

#[test]
fn task12_typed_accessors_cover_prefix_and_postfix_operator_nodes() {
    let source_id = source_id(18);
    let ast = prefix_postfix_expression_ast(source_id);

    let postfix_view = ast.expression_view().unwrap();
    assert_eq!(postfix_view.syntax_kind(), SyntaxKind::PostfixExpression);
    let postfix = postfix_view.as_postfix_expression().unwrap();
    assert_eq!(postfix.spelling.as_ref(), "!");
    assert_eq!(postfix.precedence, 90);
    assert!(postfix_view.as_prefix_expression().is_none());
    assert!(postfix_view.as_infix_expression().is_none());

    let prefix_view = ast.node_view(postfix_view.children()[0]).unwrap();
    assert_eq!(prefix_view.syntax_kind(), SyntaxKind::PrefixExpression);
    let prefix = prefix_view.as_prefix_expression().unwrap();
    assert_eq!(prefix.spelling.as_ref(), "~");
    assert_eq!(prefix.precedence, 70);
    assert!(prefix_view.as_postfix_expression().is_none());
    assert!(prefix_view.as_infix_expression().is_none());

    let snapshot = ast.snapshot_text();
    assert!(snapshot.contains("PrefixExpression spelling=\"~\" precedence=70"));
    assert!(snapshot.contains("PostfixExpression spelling=\"!\" precedence=90"));
}

#[test]
fn task13_typed_accessors_cover_atomic_formula_nodes() {
    let ast = atomic_formula_nodes_ast(source_id(19));
    let root = ast.root_view().unwrap();

    let formula = ast.expression_view().unwrap();
    assert_eq!(formula.syntax_kind(), SyntaxKind::FormulaExpression);
    assert!(formula.as_formula_expression().is_some());
    assert!(formula.as_term_expression().is_none());

    for (kind, assertion) in [
        (
            SyntaxKind::BuiltinPredicateApplication,
            first_view(root, |kind| {
                matches!(kind, SurfaceNodeKind::BuiltinPredicateApplication)
            })
            .unwrap()
            .as_builtin_predicate_application()
            .is_some(),
        ),
        (
            SyntaxKind::IsAssertion,
            first_view(root, |kind| matches!(kind, SurfaceNodeKind::IsAssertion))
                .unwrap()
                .as_is_assertion()
                .is_some(),
        ),
        (
            SyntaxKind::AttributeTestChain,
            first_view(root, |kind| {
                matches!(kind, SurfaceNodeKind::AttributeTestChain)
            })
            .unwrap()
            .as_attribute_test_chain()
            .is_some(),
        ),
        (
            SyntaxKind::PredicateApplication,
            first_view(root, |kind| {
                matches!(kind, SurfaceNodeKind::PredicateApplication)
            })
            .unwrap()
            .as_predicate_application()
            .is_some(),
        ),
        (
            SyntaxKind::PredicateSegment,
            first_view(root, |kind| {
                matches!(kind, SurfaceNodeKind::PredicateSegment)
            })
            .unwrap()
            .as_predicate_segment()
            .is_some(),
        ),
        (
            SyntaxKind::PredicateHead,
            first_view(root, |kind| matches!(kind, SurfaceNodeKind::PredicateHead))
                .unwrap()
                .as_predicate_head()
                .is_some(),
        ),
        (
            SyntaxKind::InlinePredicateApplication,
            first_view(root, |kind| {
                matches!(kind, SurfaceNodeKind::InlinePredicateApplication)
            })
            .unwrap()
            .as_inline_predicate_application()
            .is_some(),
        ),
    ] {
        assert!(assertion, "{kind:?} should have a typed accessor");
    }

    let snapshot = ast.snapshot_text();
    for name in [
        "FormulaExpression",
        "BuiltinPredicateApplication",
        "IsAssertion",
        "AttributeTestChain",
        "PredicateApplication",
        "PredicateSegment",
        "PredicateHead",
        "InlinePredicateApplication",
    ] {
        assert!(
            snapshot.contains(name),
            "snapshot should render task-13 node name {name}"
        );
    }
}

#[test]
fn task14_typed_accessors_cover_formula_connective_and_quantifier_nodes() {
    let ast = formula_surface_nodes_ast(source_id(20));
    let root = ast.root_view().unwrap();

    let binary_view = ast.expression_view().unwrap();
    assert_eq!(binary_view.syntax_kind(), SyntaxKind::BinaryFormula);
    assert!(binary_view.as_binary_formula().is_some());
    assert!(binary_view.as_formula_expression().is_none());
    assert_eq!(
        binary_view.kind(),
        &SurfaceNodeKind::BinaryFormula(SurfaceFormulaBinaryOperator {
            connective: SurfaceFormulaConnective::And,
            repeated: true,
        })
    );

    let prefix_view = first_view(root, |kind| {
        matches!(kind, SurfaceNodeKind::PrefixFormula(_))
    })
    .unwrap();
    assert_eq!(prefix_view.syntax_kind(), SyntaxKind::PrefixFormula);
    assert!(prefix_view.as_prefix_formula().is_some());
    assert_eq!(
        prefix_view.kind(),
        &SurfaceNodeKind::PrefixFormula(SurfaceFormulaPrefixOperator::Not)
    );

    let parenthesized_view = first_view(root, |kind| {
        matches!(kind, SurfaceNodeKind::ParenthesizedFormula)
    })
    .unwrap();
    assert_eq!(
        parenthesized_view.syntax_kind(),
        SyntaxKind::ParenthesizedFormula
    );
    assert!(parenthesized_view.as_parenthesized_formula().is_some());

    let universal_view = first_view(root, |kind| {
        matches!(
            kind,
            SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Universal)
        )
    })
    .unwrap();
    assert_eq!(universal_view.syntax_kind(), SyntaxKind::QuantifiedFormula);
    assert!(universal_view.as_quantified_formula().is_some());

    let existential_view = first_view(root, |kind| {
        matches!(
            kind,
            SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Existential)
        )
    })
    .unwrap();
    assert!(existential_view.as_quantified_formula().is_some());

    let segment_view = first_view(root, |kind| {
        matches!(kind, SurfaceNodeKind::QuantifierVariableSegment)
    })
    .unwrap();
    assert_eq!(
        segment_view.syntax_kind(),
        SyntaxKind::QuantifierVariableSegment
    );
    assert!(segment_view.as_quantifier_variable_segment().is_some());

    let thesis_view = first_view(root, |kind| {
        matches!(
            kind,
            SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis)
        )
    })
    .unwrap();
    assert_eq!(thesis_view.syntax_kind(), SyntaxKind::FormulaConstant);
    assert!(thesis_view.as_formula_constant().is_some());

    let snapshot = ast.snapshot_text();
    for expected in [
        "PrefixFormula operator=Not",
        "BinaryFormula connective=And repeated=true",
        "BinaryFormula connective=Or repeated=false",
        "BinaryFormula connective=Implies repeated=false",
        "BinaryFormula connective=Iff repeated=false",
        "ParenthesizedFormula",
        "QuantifiedFormula quantifier=Universal",
        "QuantifiedFormula quantifier=Existential",
        "QuantifierVariableSegment",
        "FormulaConstant constant=Thesis",
        "FormulaConstant constant=Contradiction",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-14 payload line {expected}"
        );
    }
}

#[test]
fn task15_typed_accessors_cover_set_comprehension_nodes() {
    let ast = set_comprehension_nodes_ast(source_id(27));
    let root = ast.root_view().unwrap();

    let comprehension_view = ast.expression_view().unwrap();
    assert_eq!(
        comprehension_view.syntax_kind(),
        SyntaxKind::SetComprehension
    );
    assert!(comprehension_view.as_set_comprehension().is_some());

    let segment_view = first_view(root, |kind| {
        matches!(kind, SurfaceNodeKind::ComprehensionVariableSegment)
    })
    .unwrap();
    assert_eq!(
        segment_view.syntax_kind(),
        SyntaxKind::ComprehensionVariableSegment
    );
    assert!(segment_view.as_comprehension_variable_segment().is_some());

    let snapshot = ast.snapshot_text();
    for expected in [
        "SetComprehension",
        "ComprehensionVariableSegment",
        "FormulaExpression",
        "FormulaConstant constant=Thesis",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-15 line {expected}"
        );
    }
}

#[test]
fn task16_typed_accessors_cover_simple_statement_nodes() {
    let ast = statement_nodes_ast(source_id(28));
    let root = ast.root_view().unwrap();

    let statement_item_view =
        first_view(root, |kind| matches!(kind, SurfaceNodeKind::StatementItem)).unwrap();
    assert_eq!(statement_item_view.syntax_kind(), SyntaxKind::StatementItem);
    assert!(statement_item_view.as_statement_item().is_some());

    let let_view = first_view(root, |kind| matches!(kind, SurfaceNodeKind::LetStatement)).unwrap();
    assert_eq!(let_view.syntax_kind(), SyntaxKind::LetStatement);
    assert!(let_view.as_let_statement().is_some());

    let segment_view = first_view(root, |kind| {
        matches!(kind, SurfaceNodeKind::QualifiedVariableSegment)
    })
    .unwrap();
    assert_eq!(
        segment_view.syntax_kind(),
        SyntaxKind::QualifiedVariableSegment
    );
    assert!(segment_view.as_qualified_variable_segment().is_some());

    let assumption_view = first_view(root, |kind| {
        matches!(kind, SurfaceNodeKind::AssumptionStatement)
    })
    .unwrap();
    assert_eq!(
        assumption_view.syntax_kind(),
        SyntaxKind::AssumptionStatement
    );
    assert!(assumption_view.as_assumption_statement().is_some());

    let proposition_view =
        first_view(root, |kind| matches!(kind, SurfaceNodeKind::Proposition)).unwrap();
    assert_eq!(proposition_view.syntax_kind(), SyntaxKind::Proposition);
    assert!(proposition_view.as_proposition().is_some());

    let condition_list_view =
        first_view(root, |kind| matches!(kind, SurfaceNodeKind::ConditionList)).unwrap();
    assert_eq!(condition_list_view.syntax_kind(), SyntaxKind::ConditionList);
    assert!(condition_list_view.as_condition_list().is_some());

    let given_view =
        first_view(root, |kind| matches!(kind, SurfaceNodeKind::GivenStatement)).unwrap();
    assert_eq!(given_view.syntax_kind(), SyntaxKind::GivenStatement);
    assert!(given_view.as_given_statement().is_some());

    let take_view =
        first_view(root, |kind| matches!(kind, SurfaceNodeKind::TakeStatement)).unwrap();
    assert_eq!(take_view.syntax_kind(), SyntaxKind::TakeStatement);
    assert!(take_view.as_take_statement().is_some());

    let witness_view = first_view(root, |kind| matches!(kind, SurfaceNodeKind::Witness)).unwrap();
    assert_eq!(witness_view.syntax_kind(), SyntaxKind::Witness);
    assert!(witness_view.as_witness().is_some());

    let set_view = first_view(root, |kind| matches!(kind, SurfaceNodeKind::SetStatement)).unwrap();
    assert_eq!(set_view.syntax_kind(), SyntaxKind::SetStatement);
    assert!(set_view.as_set_statement().is_some());

    let equating_view = first_view(root, |kind| matches!(kind, SurfaceNodeKind::Equating)).unwrap();
    assert_eq!(equating_view.syntax_kind(), SyntaxKind::Equating);
    assert!(equating_view.as_equating().is_some());

    let snapshot = ast.snapshot_text();
    for expected in [
        "StatementItem",
        "LetStatement",
        "QualifiedVariableSegment",
        "AssumptionStatement",
        "Proposition",
        "ConditionList",
        "GivenStatement",
        "TakeStatement",
        "Witness",
        "SetStatement",
        "Equating",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-16 line {expected}"
        );
    }
}

#[test]
fn task17_typed_accessors_cover_justification_nodes() {
    let ast = justification_nodes_ast(source_id(29));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task17_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task17_view!(
        SurfaceNodeKind::CompactStatement,
        SyntaxKind::CompactStatement,
        as_compact_statement
    );
    assert_task17_view!(
        SurfaceNodeKind::JustificationClause,
        SyntaxKind::JustificationClause,
        as_justification_clause
    );
    assert_task17_view!(
        SurfaceNodeKind::ReferenceList,
        SyntaxKind::ReferenceList,
        as_reference_list
    );
    assert_task17_view!(
        SurfaceNodeKind::Reference,
        SyntaxKind::Reference,
        as_reference
    );
    assert_task17_view!(
        SurfaceNodeKind::QualifiedReference,
        SyntaxKind::QualifiedReference,
        as_qualified_reference
    );
    assert_task17_view!(
        SurfaceNodeKind::GroupedReference,
        SyntaxKind::GroupedReference,
        as_grouped_reference
    );
    assert_task17_view!(
        SurfaceNodeKind::GroupedReferenceItem,
        SyntaxKind::GroupedReferenceItem,
        as_grouped_reference_item
    );
    assert_task17_view!(
        SurfaceNodeKind::BulkReference,
        SyntaxKind::BulkReference,
        as_bulk_reference
    );
    assert_task17_view!(
        SurfaceNodeKind::ComputationJustification,
        SyntaxKind::ComputationJustification,
        as_computation_justification
    );
    assert_task17_view!(
        SurfaceNodeKind::ComputationOption,
        SyntaxKind::ComputationOption,
        as_computation_option
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "CompactStatement",
        "JustificationClause",
        "ReferenceList",
        "Reference",
        "QualifiedReference",
        "GroupedReference",
        "GroupedReferenceItem",
        "BulkReference",
        "ComputationJustification",
        "ComputationOption",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-17 line {expected}"
        );
    }
}

#[test]
fn task18_typed_accessors_cover_consider_reconsider_nodes() {
    let ast = task18_statement_nodes_ast(source_id(30));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task18_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task18_view!(
        SurfaceNodeKind::ConsiderStatement,
        SyntaxKind::ConsiderStatement,
        as_consider_statement
    );
    assert_task18_view!(
        SurfaceNodeKind::ReconsiderStatement,
        SyntaxKind::ReconsiderStatement,
        as_reconsider_statement
    );
    assert_task18_view!(
        SurfaceNodeKind::ReconsiderItem,
        SyntaxKind::ReconsiderItem,
        as_reconsider_item
    );

    let snapshot = ast.snapshot_text();
    for expected in ["ConsiderStatement", "ReconsiderStatement", "ReconsiderItem"] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-18 line {expected}"
        );
    }
}

#[test]
fn task19_typed_accessors_cover_conclusion_then_iterative_nodes() {
    let ast = task19_statement_nodes_ast(source_id(31));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task19_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task19_view!(
        SurfaceNodeKind::ConclusionStatement,
        SyntaxKind::ConclusionStatement,
        as_conclusion_statement
    );
    assert_task19_view!(
        SurfaceNodeKind::ThenStatement,
        SyntaxKind::ThenStatement,
        as_then_statement
    );
    assert_task19_view!(
        SurfaceNodeKind::IterativeEqualityStatement,
        SyntaxKind::IterativeEqualityStatement,
        as_iterative_equality_statement
    );
    assert_task19_view!(
        SurfaceNodeKind::IterativeEqualityStep,
        SyntaxKind::IterativeEqualityStep,
        as_iterative_equality_step
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "ConclusionStatement",
        "ThenStatement",
        "IterativeEqualityStatement",
        "IterativeEqualityStep",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-19 line {expected}"
        );
    }
}

#[test]
fn task20_typed_accessors_cover_block_statement_nodes() {
    let ast = task20_statement_nodes_ast(source_id(32));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task20_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task20_view!(
        SurfaceNodeKind::NowStatement,
        SyntaxKind::NowStatement,
        as_now_statement
    );
    assert_task20_view!(
        SurfaceNodeKind::HerebyStatement,
        SyntaxKind::HerebyStatement,
        as_hereby_statement
    );
    assert_task20_view!(
        SurfaceNodeKind::CaseReasoningStatement,
        SyntaxKind::CaseReasoningStatement,
        as_case_reasoning_statement
    );
    assert_task20_view!(
        SurfaceNodeKind::CaseItem,
        SyntaxKind::CaseItem,
        as_case_item
    );
    assert_task20_view!(
        SurfaceNodeKind::SupposeItem,
        SyntaxKind::SupposeItem,
        as_suppose_item
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "NowStatement",
        "HerebyStatement",
        "CaseReasoningStatement",
        "CaseItem",
        "SupposeItem",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-20 line {expected}"
        );
    }
}

#[test]
fn task21_typed_accessors_cover_inline_definition_nodes() {
    let ast = task21_statement_nodes_ast(source_id(33));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task21_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task21_view!(
        SurfaceNodeKind::InlineFunctorDefinition,
        SyntaxKind::InlineFunctorDefinition,
        as_inline_functor_definition
    );
    assert_task21_view!(
        SurfaceNodeKind::InlinePredicateDefinition,
        SyntaxKind::InlinePredicateDefinition,
        as_inline_predicate_definition
    );
    assert_task21_view!(
        SurfaceNodeKind::TypedParameter,
        SyntaxKind::TypedParameter,
        as_typed_parameter
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "InlineFunctorDefinition",
        "InlinePredicateDefinition",
        "TypedParameter",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-21 line {expected}"
        );
    }
}

#[test]
fn task22_typed_accessors_cover_theorem_and_proof_nodes() {
    let ast = task22_theorem_nodes_ast(source_id(34));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task22_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task22_view!(
        SurfaceNodeKind::TheoremItem,
        SyntaxKind::TheoremItem,
        as_theorem_item
    );
    assert_task22_view!(
        SurfaceNodeKind::LemmaItem,
        SyntaxKind::LemmaItem,
        as_lemma_item
    );
    assert_task22_view!(
        SurfaceNodeKind::ProofBlock,
        SyntaxKind::ProofBlock,
        as_proof_block
    );

    let snapshot = ast.snapshot_text();
    for expected in ["TheoremItem", "LemmaItem", "ProofBlock"] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-22 line {expected}"
        );
    }
}

#[test]
fn task23_typed_accessors_cover_definition_nodes() {
    let ast = task23_definition_nodes_ast(source_id(35));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task23_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task23_view!(
        SurfaceNodeKind::DefinitionBlockItem,
        SyntaxKind::DefinitionBlockItem,
        as_definition_block_item
    );
    assert_task23_view!(
        SurfaceNodeKind::DefinitionParameter,
        SyntaxKind::DefinitionParameter,
        as_definition_parameter
    );
    assert_task23_view!(
        SurfaceNodeKind::AttributeDefinition,
        SyntaxKind::AttributeDefinition,
        as_attribute_definition
    );
    assert_task23_view!(
        SurfaceNodeKind::AttributePattern,
        SyntaxKind::AttributePattern,
        as_attribute_pattern
    );
    assert_task23_view!(
        SurfaceNodeKind::FormulaDefiniens,
        SyntaxKind::FormulaDefiniens,
        as_formula_definiens
    );
    assert_task23_view!(
        SurfaceNodeKind::FormulaCase,
        SyntaxKind::FormulaCase,
        as_formula_case
    );
    assert_task23_view!(
        SurfaceNodeKind::CorrectnessCondition,
        SyntaxKind::CorrectnessCondition,
        as_correctness_condition
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "DefinitionBlockItem",
        "DefinitionParameter",
        "AttributeDefinition",
        "AttributePattern",
        "FormulaDefiniens",
        "FormulaCase",
        "CorrectnessCondition",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-23 line {expected}"
        );
    }
}

#[test]
fn task48_typed_accessor_covers_property_implementation_node() {
    let ast = task48_property_implementation_ast(source_id(58));
    let root = ast.root_view().unwrap();
    let view = first_view(root, |kind| {
        matches!(kind, SurfaceNodeKind::PropertyImplementation)
    })
    .unwrap();
    assert_eq!(view.syntax_kind(), SyntaxKind::PropertyImplementation);
    assert!(view.as_property_implementation().is_some());
    assert_eq!(
        SyntaxKind::from_raw(192),
        SyntaxKind::PropertyImplementation
    );
    assert!(SyntaxKind::PropertyImplementation.is_node_kind());
    assert!(!SyntaxKind::PropertyImplementation.is_token_kind());
    assert!(ast.snapshot_text().contains("PropertyImplementation"));
    assert!(
        ast.rowan_root()
            .descendants()
            .any(|node| node.kind() == SyntaxKind::PropertyImplementation)
    );
}

#[test]
fn task24_typed_accessors_cover_predicate_definition_nodes() {
    let ast = task23_definition_nodes_ast(source_id(36));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task24_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task24_view!(
        SurfaceNodeKind::PredicateDefinition,
        SyntaxKind::PredicateDefinition,
        as_predicate_definition
    );
    assert_task24_view!(
        SurfaceNodeKind::PredicatePattern,
        SyntaxKind::PredicatePattern,
        as_predicate_pattern
    );

    let snapshot = ast.snapshot_text();
    for expected in ["PredicateDefinition", "PredicatePattern"] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-24 line {expected}"
        );
    }
}

#[test]
fn task25_typed_accessors_cover_functor_definition_nodes() {
    let ast = task25_functor_definition_nodes_ast(source_id(37));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task25_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task25_view!(
        SurfaceNodeKind::FunctorDefinition,
        SyntaxKind::FunctorDefinition,
        as_functor_definition
    );
    assert_task25_view!(
        SurfaceNodeKind::FunctorPattern,
        SyntaxKind::FunctorPattern,
        as_functor_pattern
    );
    assert_task25_view!(
        SurfaceNodeKind::TermDefiniens,
        SyntaxKind::TermDefiniens,
        as_term_definiens
    );
    assert_task25_view!(
        SurfaceNodeKind::TermCase,
        SyntaxKind::TermCase,
        as_term_case
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "FunctorDefinition",
        "FunctorPattern",
        "TermDefiniens",
        "TermCase",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-25 line {expected}"
        );
    }
}

#[test]
fn task26_typed_accessors_cover_mode_definition_nodes() {
    let ast = task26_mode_definition_nodes_ast(source_id(39));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task26_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task26_view!(
        SurfaceNodeKind::ModeDefinition,
        SyntaxKind::ModeDefinition,
        as_mode_definition
    );
    assert_task26_view!(
        SurfaceNodeKind::ModePattern,
        SyntaxKind::ModePattern,
        as_mode_pattern
    );
    assert_task26_view!(
        SurfaceNodeKind::ModeProperty,
        SyntaxKind::ModeProperty,
        as_mode_property
    );

    let snapshot = ast.snapshot_text();
    for expected in ["ModeDefinition", "ModePattern", "ModeProperty"] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-26 line {expected}"
        );
    }
}

#[test]
fn task27_typed_accessors_cover_redefinition_and_notation_nodes() {
    let ast = task27_redefinition_notation_nodes_ast(source_id(41));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task27_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task27_view!(
        SurfaceNodeKind::AttributeRedefinition,
        SyntaxKind::AttributeRedefinition,
        as_attribute_redefinition
    );
    assert_task27_view!(
        SurfaceNodeKind::PredicateRedefinition,
        SyntaxKind::PredicateRedefinition,
        as_predicate_redefinition
    );
    assert_task27_view!(
        SurfaceNodeKind::FunctorRedefinition,
        SyntaxKind::FunctorRedefinition,
        as_functor_redefinition
    );
    assert_task27_view!(
        SurfaceNodeKind::CoherenceCondition,
        SyntaxKind::CoherenceCondition,
        as_coherence_condition
    );
    assert_task27_view!(
        SurfaceNodeKind::NotationAlias,
        SyntaxKind::NotationAlias,
        as_notation_alias
    );
    assert_task27_view!(
        SurfaceNodeKind::NotationPattern,
        SyntaxKind::NotationPattern,
        as_notation_pattern
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "AttributeRedefinition",
        "PredicateRedefinition",
        "FunctorRedefinition",
        "CoherenceCondition",
        "NotationAlias",
        "NotationPattern",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-27 line {expected}"
        );
    }

    let predicate_redefinition = first_view(root, |kind| {
        matches!(kind, SurfaceNodeKind::PredicateRedefinition)
    })
    .expect("task-27 fixture should contain PredicateRedefinition");
    assert_eq!(
        child_signature(predicate_redefinition),
        vec![
            "redefine",
            "pred",
            "P",
            ":",
            "PredicatePattern",
            "means",
            "FormulaDefiniens",
            ";",
            "CoherenceCondition",
        ],
        "PredicateRedefinition should expose label before PredicatePattern"
    );
}

#[test]
fn task22_predicate_redefinition_missing_label_snapshot_is_distinct() {
    let source_id = source_id(42);
    let mut builder = SurfaceAstBuilder::new(source_id);
    let redefine = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "redefine",
        range(source_id, 0, 8),
    );
    let pred = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "pred",
        range(source_id, 9, 13),
    );
    let missing_label = builder.add_recovery(
        SyntaxRecoveryKind::MissingTerm,
        range(source_id, 14, 14),
        vec![],
    );
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 14, 15),
    );
    let left = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 16, 17));
    let symbol = builder.add_token(SurfaceTokenKind::UserSymbol, "R", range(source_id, 18, 19));
    let means = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "means",
        range(source_id, 20, 25),
    );
    let thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 26, 32),
    );
    let semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 32, 33),
    );
    let coherence = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "coherence",
        range(source_id, 34, 43),
    );
    let by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 44, 46),
    );
    let reference_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 47, 50),
    );
    let coherence_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 50, 51),
    );

    let pattern = builder.add_node(
        SurfaceNodeKind::PredicatePattern,
        range(source_id, 16, 19),
        vec![left, symbol],
    );
    let formula = thesis_formula_node(&mut builder, source_id, thesis, 26, 32);
    let definiens = builder.add_node(
        SurfaceNodeKind::FormulaDefiniens,
        range(source_id, 26, 32),
        vec![formula],
    );
    let reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 47, 50),
        vec![reference_name],
    );
    let references = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 47, 50),
        vec![reference],
    );
    let justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 44, 50),
        vec![by, references],
    );
    let coherence_node = builder.add_node(
        SurfaceNodeKind::CoherenceCondition,
        range(source_id, 34, 51),
        vec![coherence, justification, coherence_semicolon],
    );
    let redefinition = builder.add_node(
        SurfaceNodeKind::PredicateRedefinition,
        range(source_id, 0, 51),
        vec![
            redefine,
            pred,
            missing_label,
            colon,
            pattern,
            means,
            definiens,
            semicolon,
            coherence_node,
        ],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 51),
        vec![
            redefine,
            pred,
            missing_label,
            colon,
            left,
            symbol,
            means,
            thesis,
            semicolon,
            coherence,
            by,
            reference_name,
            coherence_semicolon,
            redefinition,
        ],
    );
    let ast = builder.finish(Some(root), None);
    let predicate_redefinition = first_view(ast.root_view().unwrap(), |kind| {
        matches!(kind, SurfaceNodeKind::PredicateRedefinition)
    })
    .expect("fixture should contain PredicateRedefinition");

    assert_eq!(
        child_signature(predicate_redefinition),
        vec![
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
        "missing label recovery should occupy the label slot before PredicatePattern"
    );
    assert!(
        ast.snapshot_text()
            .contains("ErrorRecovery kind=MissingTerm"),
        "snapshot should render missing-label recovery distinctly"
    );
}

#[test]
fn task28_typed_accessors_cover_property_clause_nodes() {
    let ast = task28_property_clause_nodes_ast(source_id(43));
    let root = ast.root_view().unwrap();
    let view = first_view(root, |kind| matches!(kind, SurfaceNodeKind::PropertyClause))
        .expect("task-28 property clause fixture should contain PropertyClause");

    assert_eq!(view.syntax_kind(), SyntaxKind::PropertyClause);
    assert!(view.as_property_clause().is_some());
    assert!(ast.snapshot_text().contains("PropertyClause"));
}

#[test]
fn task29_typed_accessors_cover_structure_nodes() {
    let ast = task29_structure_nodes_ast(source_id(45));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task29_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task29_view!(
        SurfaceNodeKind::StructureDefinition,
        SyntaxKind::StructureDefinition,
        as_structure_definition
    );
    assert_task29_view!(
        SurfaceNodeKind::StructurePattern,
        SyntaxKind::StructurePattern,
        as_structure_pattern
    );
    assert_task29_view!(
        SurfaceNodeKind::StructureField,
        SyntaxKind::StructureField,
        as_structure_field
    );
    assert_task29_view!(
        SurfaceNodeKind::StructureProperty,
        SyntaxKind::StructureProperty,
        as_structure_property
    );
    assert_task29_view!(
        SurfaceNodeKind::InheritanceDefinition,
        SyntaxKind::InheritanceDefinition,
        as_inheritance_definition
    );
    assert_task29_view!(
        SurfaceNodeKind::InheritanceTarget,
        SyntaxKind::InheritanceTarget,
        as_inheritance_target
    );
    assert_task29_view!(
        SurfaceNodeKind::FieldRedefinition,
        SyntaxKind::FieldRedefinition,
        as_field_redefinition
    );
    assert_task29_view!(
        SurfaceNodeKind::PropertyRedefinition,
        SyntaxKind::PropertyRedefinition,
        as_property_redefinition
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "StructureDefinition",
        "StructurePattern",
        "StructureField",
        "StructureProperty",
        "InheritanceDefinition",
        "InheritanceTarget",
        "FieldRedefinition",
        "PropertyRedefinition",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-29 line {expected}"
        );
    }
}

#[test]
fn task30_typed_accessors_cover_registration_nodes() {
    let ast = task30_registration_nodes_ast(source_id(46));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task30_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task30_view!(
        SurfaceNodeKind::RegistrationBlockItem,
        SyntaxKind::RegistrationBlockItem,
        as_registration_block_item
    );
    assert_task30_view!(
        SurfaceNodeKind::RegistrationParameter,
        SyntaxKind::RegistrationParameter,
        as_registration_parameter
    );
    assert_task30_view!(
        SurfaceNodeKind::ExistentialRegistration,
        SyntaxKind::ExistentialRegistration,
        as_existential_registration
    );
    assert_task30_view!(
        SurfaceNodeKind::ConditionalRegistration,
        SyntaxKind::ConditionalRegistration,
        as_conditional_registration
    );
    assert_task30_view!(
        SurfaceNodeKind::FunctorialRegistration,
        SyntaxKind::FunctorialRegistration,
        as_functorial_registration
    );
    assert_task30_view!(
        SurfaceNodeKind::ReductionRegistration,
        SyntaxKind::ReductionRegistration,
        as_reduction_registration
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "RegistrationBlockItem",
        "RegistrationParameter",
        "ExistentialRegistration",
        "ConditionalRegistration",
        "FunctorialRegistration",
        "ReductionRegistration",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-30 line {expected}"
        );
    }
}

#[test]
fn task31_typed_accessors_cover_template_nodes() {
    let ast = task31_template_nodes_ast(source_id(48));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task31_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task31_view!(
        SurfaceNodeKind::TemplateParameter,
        SyntaxKind::TemplateParameter,
        as_template_parameter
    );
    assert_task31_view!(
        SurfaceNodeKind::TemplateLoci,
        SyntaxKind::TemplateLoci,
        as_template_loci
    );
    assert_task31_view!(
        SurfaceNodeKind::TemplateLocus,
        SyntaxKind::TemplateLocus,
        as_template_locus
    );
    assert_task31_view!(
        SurfaceNodeKind::TemplateArguments,
        SyntaxKind::TemplateArguments,
        as_template_arguments
    );
    assert_task31_view!(
        SurfaceNodeKind::TemplateArgument,
        SyntaxKind::TemplateArgument,
        as_template_argument
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "TemplateParameter",
        "TemplateLoci",
        "TemplateLocus",
        "TemplateArguments",
        "TemplateArgument",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-31 line {expected}"
        );
    }
}

#[test]
fn task32_typed_accessors_cover_algorithm_nodes() {
    let ast = task32_algorithm_nodes_ast(source_id(50));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task32_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task32_view!(
        SurfaceNodeKind::AlgorithmDefinition,
        SyntaxKind::AlgorithmDefinition,
        as_algorithm_definition
    );
    assert_task32_view!(
        SurfaceNodeKind::AlgorithmParameters,
        SyntaxKind::AlgorithmParameters,
        as_algorithm_parameters
    );
    assert_task32_view!(
        SurfaceNodeKind::AlgorithmBody,
        SyntaxKind::AlgorithmBody,
        as_algorithm_body
    );
    assert_task32_view!(
        SurfaceNodeKind::AlgorithmStatementList,
        SyntaxKind::AlgorithmStatementList,
        as_algorithm_statement_list
    );
    assert_task32_view!(
        SurfaceNodeKind::VariableDeclaration,
        SyntaxKind::VariableDeclaration,
        as_variable_declaration
    );
    assert_task32_view!(
        SurfaceNodeKind::VariableBinding,
        SyntaxKind::VariableBinding,
        as_variable_binding
    );
    assert_task32_view!(
        SurfaceNodeKind::AssignmentStatement,
        SyntaxKind::AssignmentStatement,
        as_assignment_statement
    );
    assert_task32_view!(SurfaceNodeKind::Lvalue, SyntaxKind::Lvalue, as_lvalue);
    assert_task32_view!(
        SurfaceNodeKind::SnapshotStatement,
        SyntaxKind::SnapshotStatement,
        as_snapshot_statement
    );
    assert_task32_view!(
        SurfaceNodeKind::ReturnStatement,
        SyntaxKind::ReturnStatement,
        as_return_statement
    );
    assert_task32_view!(
        SurfaceNodeKind::ClaimBlockItem,
        SyntaxKind::ClaimBlockItem,
        as_claim_block_item
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "AlgorithmDefinition",
        "AlgorithmParameters",
        "AlgorithmBody",
        "AlgorithmStatementList",
        "VariableDeclaration",
        "VariableBinding",
        "AssignmentStatement",
        "Lvalue",
        "SnapshotStatement",
        "ReturnStatement",
        "ClaimBlockItem",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-32 line {expected}"
        );
    }
}

#[test]
fn task33_typed_accessors_cover_algorithm_control_flow_nodes() {
    let ast = task33_algorithm_control_flow_nodes_ast(source_id(52));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task33_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task33_view!(
        SurfaceNodeKind::IfStatement,
        SyntaxKind::IfStatement,
        as_if_statement
    );
    assert_task33_view!(
        SurfaceNodeKind::WhileStatement,
        SyntaxKind::WhileStatement,
        as_while_statement
    );
    assert_task33_view!(
        SurfaceNodeKind::ForRangeStatement,
        SyntaxKind::ForRangeStatement,
        as_for_range_statement
    );
    assert_task33_view!(
        SurfaceNodeKind::ForCollectionStatement,
        SyntaxKind::ForCollectionStatement,
        as_for_collection_statement
    );
    assert_task33_view!(
        SurfaceNodeKind::MatchStatement,
        SyntaxKind::MatchStatement,
        as_match_statement
    );
    assert_task33_view!(
        SurfaceNodeKind::MatchCase,
        SyntaxKind::MatchCase,
        as_match_case
    );
    assert_task33_view!(
        SurfaceNodeKind::MatchEnding,
        SyntaxKind::MatchEnding,
        as_match_ending
    );
    assert_task33_view!(
        SurfaceNodeKind::BreakStatement,
        SyntaxKind::BreakStatement,
        as_break_statement
    );
    assert_task33_view!(
        SurfaceNodeKind::ContinueStatement,
        SyntaxKind::ContinueStatement,
        as_continue_statement
    );

    let snapshot = ast.snapshot_text();
    for expected in [
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
            snapshot.contains(expected),
            "snapshot should render task-33 line {expected}"
        );
    }
}

#[test]
fn task34_typed_accessors_cover_algorithm_verification_nodes() {
    let ast = task34_algorithm_verification_nodes_ast(source_id(54));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task34_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task34_view!(
        SurfaceNodeKind::AlgorithmTerminationClause,
        SyntaxKind::AlgorithmTerminationClause,
        as_algorithm_termination_clause
    );
    assert_task34_view!(
        SurfaceNodeKind::AlgorithmRequiresClause,
        SyntaxKind::AlgorithmRequiresClause,
        as_algorithm_requires_clause
    );
    assert_task34_view!(
        SurfaceNodeKind::AlgorithmEnsuresClause,
        SyntaxKind::AlgorithmEnsuresClause,
        as_algorithm_ensures_clause
    );
    assert_task34_view!(
        SurfaceNodeKind::AlgorithmDecreasingClause,
        SyntaxKind::AlgorithmDecreasingClause,
        as_algorithm_decreasing_clause
    );
    assert_task34_view!(
        SurfaceNodeKind::LoopInvariantClause,
        SyntaxKind::LoopInvariantClause,
        as_loop_invariant_clause
    );
    assert_task34_view!(
        SurfaceNodeKind::LoopDecreasingClause,
        SyntaxKind::LoopDecreasingClause,
        as_loop_decreasing_clause
    );
    assert_task34_view!(
        SurfaceNodeKind::AssertStatement,
        SyntaxKind::AssertStatement,
        as_assert_statement
    );
    assert_task34_view!(
        SurfaceNodeKind::TermList,
        SyntaxKind::TermList,
        as_term_list
    );

    let snapshot = ast.snapshot_text();
    for expected in [
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
            snapshot.contains(expected),
            "snapshot should render task-34 line {expected}"
        );
    }
}

#[test]
fn task35_typed_accessors_cover_annotation_nodes() {
    let ast = task35_annotation_nodes_ast(source_id(56));
    let root = ast.root_view().unwrap();

    macro_rules! assert_task35_view {
        ($pattern:pat, $syntax_kind:expr, $accessor:ident) => {{
            let view = first_view(root, |kind| matches!(kind, $pattern)).unwrap();
            assert_eq!(view.syntax_kind(), $syntax_kind);
            assert!(view.$accessor().is_some());
        }};
    }

    assert_task35_view!(
        SurfaceNodeKind::Annotation,
        SyntaxKind::Annotation,
        as_annotation
    );
    assert_task35_view!(
        SurfaceNodeKind::LibraryAnnotation,
        SyntaxKind::LibraryAnnotation,
        as_library_annotation
    );
    assert_task35_view!(
        SurfaceNodeKind::AnnotationLabelList,
        SyntaxKind::AnnotationLabelList,
        as_annotation_label_list
    );
    assert_task35_view!(
        SurfaceNodeKind::AnnotationLabel,
        SyntaxKind::AnnotationLabel,
        as_annotation_label
    );
    assert_task35_view!(
        SurfaceNodeKind::AnnotationArgumentList,
        SyntaxKind::AnnotationArgumentList,
        as_annotation_argument_list
    );
    assert_task35_view!(
        SurfaceNodeKind::AnnotationArgument,
        SyntaxKind::AnnotationArgument,
        as_annotation_argument
    );
    assert_task35_view!(
        SurfaceNodeKind::ProofHintOptionList,
        SyntaxKind::ProofHintOptionList,
        as_proof_hint_option_list
    );
    assert_task35_view!(
        SurfaceNodeKind::ProofHintOption,
        SyntaxKind::ProofHintOption,
        as_proof_hint_option
    );
    assert_task35_view!(
        SurfaceNodeKind::StandaloneDiagnosticAnnotation,
        SyntaxKind::StandaloneDiagnosticAnnotation,
        as_standalone_diagnostic_annotation
    );
    assert_task35_view!(
        SurfaceNodeKind::AnnotatedStatement,
        SyntaxKind::AnnotatedStatement,
        as_annotated_statement
    );
    assert_task35_view!(
        SurfaceNodeKind::AnnotatedAlgorithmStatement,
        SyntaxKind::AnnotatedAlgorithmStatement,
        as_annotated_algorithm_statement
    );
    assert_task35_view!(
        SurfaceNodeKind::AnnotatedDefinitionContent,
        SyntaxKind::AnnotatedDefinitionContent,
        as_annotated_definition_content
    );
    assert_task35_view!(
        SurfaceNodeKind::AnnotatedRegistrationContent,
        SyntaxKind::AnnotatedRegistrationContent,
        as_annotated_registration_content
    );

    let annotation_marker = ast
        .token_views()
        .find(|view| {
            view.as_token()
                .is_some_and(|token| token.kind == SurfaceTokenKind::AnnotationMarker)
        })
        .unwrap();
    assert_eq!(annotation_marker.syntax_kind(), SyntaxKind::Token);
    assert_eq!(
        annotation_marker.as_token().unwrap().kind.syntax_kind(),
        SyntaxKind::TokenAnnotationMarker
    );
    assert!(
        ast.rowan_root()
            .descendants_with_tokens()
            .any(|element| element.kind() == SyntaxKind::TokenAnnotationMarker),
        "rowan tree should preserve annotation-marker token leaves"
    );

    let snapshot = ast.snapshot_text();
    for expected in [
        "Annotation",
        "LibraryAnnotation",
        "AnnotationLabelList",
        "AnnotationLabel",
        "AnnotationArgumentList",
        "AnnotationArgument",
        "ProofHintOptionList",
        "ProofHintOption",
        "StandaloneDiagnosticAnnotation",
        "AnnotatedStatement",
        "AnnotatedAlgorithmStatement",
        "AnnotatedDefinitionContent",
        "AnnotatedRegistrationContent",
    ] {
        assert!(
            snapshot.contains(expected),
            "snapshot should render task-35 line {expected}"
        );
    }
}

#[test]
fn recovery_snapshot_names_are_unique_and_fully_fixture_backed() {
    let source_id = source_id(9);
    let fixtures = recovery_fixtures(source_id);
    let fixture_kinds = fixtures
        .iter()
        .map(|fixture| fixture.kind)
        .collect::<Vec<_>>();
    let all_kinds = all_recovery_kinds();

    assert_eq!(fixture_kinds, all_kinds);

    let mut names = std::collections::BTreeSet::new();
    for recovery_kind in all_kinds {
        let name = super::recovery_snapshot_name(recovery_kind);

        assert!(!name.is_empty());
        assert!(
            names.insert(name),
            "recovery snapshot name {name:?} should be unique"
        );
    }
}

#[test]
fn snapshot_rendering_includes_trivia_when_requested() {
    let source_id = source_id(10);
    let ast = expression_ast(source_id);
    let expression = ast.expression_root().unwrap();
    let token = ast.token_nodes()[0];
    let mut trivia = SurfaceTriviaBuilder::new(source_id);
    trivia.add_comment(CommentKind::SingleLine, range(source_id, 22, 31));
    trivia.add_doc_comment_attachment(
        range(source_id, 0, 8),
        TriviaAttachmentTarget::Node(TriviaNodeTarget::new(expression, range(source_id, 0, 6))),
        TriviaPlacement::Leading,
    );
    trivia.add_skipped_token_range(
        range(source_id, 32, 35),
        Some(TriviaAttachmentTarget::Token(TriviaNodeTarget::new(
            token,
            range(source_id, 0, 1),
        ))),
        SkippedTokenReason::UnexpectedToken,
    );
    trivia.add_whitespace_hint(
        WhitespaceHintKind::RequiresSeparation,
        range(source_id, 1, 2),
    );
    let ast = ast.with_trivia(trivia.finish());
    let actual = ast.snapshot_text_with_trivia();

    assert!(actual.starts_with(&ast.snapshot_text()));
    assert!(actual.contains("trivia:\n"));
    assert!(actual.contains("Comment kind=SingleLine range=22..31"));
    assert!(actual.contains("DocComment range=0..8 placement=Leading target=node:range:0..6"));
    assert!(
        actual.contains("SkippedTokens reason=UnexpectedToken range=32..35 owner=token:range:0..1")
    );
    assert!(actual.contains("WhitespaceHint kind=RequiresSeparation range=1..2"));
    assert!(
        !ast.snapshot_text().contains("trivia:"),
        "default snapshot rendering stays compatible with task-3 baselines"
    );
}

#[test]
fn doc_comment_can_attach_to_following_placeholder_item_node() {
    let source_id = source_id(22);
    let mut builder = SurfaceAstBuilder::new(source_id);
    let theorem = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 4, 11),
    );
    let semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 11, 12),
    );
    let item = builder.add_node(
        SurfaceNodeKind::PlaceholderItem,
        range(source_id, 4, 12),
        vec![theorem, semicolon],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 4, 12),
        vec![item],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 4, 12),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 4, 12),
        vec![theorem, semicolon, compilation_unit],
    );
    let ast = builder.finish(Some(root), None);
    let mut trivia = SurfaceTriviaBuilder::new(source_id);
    trivia.add_doc_comment_attachment(
        range(source_id, 0, 3),
        TriviaAttachmentTarget::Node(TriviaNodeTarget::new(sid(item), range(source_id, 4, 12))),
        TriviaPlacement::Leading,
    );
    let ast = ast.with_trivia(trivia.finish());

    assert!(ast.snapshot_text().contains("PlaceholderItem range=4..12"));
    assert!(
        ast.snapshot_text_with_trivia()
            .contains("DocComment range=0..3 placement=Leading target=node:range:4..12")
    );
}

#[test]
fn trivia_snapshot_rendering_is_sorted_and_byte_identical() {
    let source_id = source_id(9);
    let first = expression_ast(source_id).with_trivia(scrambled_trivia(source_id).finish());
    let second = expression_ast(source_id).with_trivia(scrambled_trivia(source_id).finish());
    let expected = format!(
        "{}{}",
        expression_ast(source_id).snapshot_text(),
        concat!(
            "trivia:\n",
            "  Comment kind=MultiLine range=2..5\n",
            "  Comment kind=SingleLine range=30..39\n",
            "  DocComment range=6..8 placement=Leading target=detached:point:9\n",
            "  DocComment range=10..12 placement=Trailing target=detached:range:12..13\n",
            "  SkippedTokens reason=MalformedAnnotation range=40..44 owner=<none>\n",
            "  SkippedTokens reason=UnexpectedToken range=50..51 owner=detached:range:48..52\n",
            "  WhitespaceHint kind=LineBreakBefore range=14..15\n",
            "  WhitespaceHint kind=SyntheticBoundary range=60..61\n",
        )
    );

    assert_eq!(first.snapshot_text_with_trivia(), expected);
    assert_eq!(
        first.snapshot_text_with_trivia(),
        second.snapshot_text_with_trivia()
    );
}

#[test]
fn trivia_snapshot_target_sorting_breaks_collisions_deterministically() {
    let source_id = source_id(10);
    let first =
        expression_ast(source_id).with_trivia(colliding_target_trivia(source_id, false).finish());
    let second =
        expression_ast(source_id).with_trivia(colliding_target_trivia(source_id, true).finish());
    let expected = format!(
        "{}{}",
        expression_ast(source_id).snapshot_text(),
        concat!(
            "trivia:\n",
            "  DocComment range=20..22 placement=Leading target=detached:point:30\n",
            "  DocComment range=20..22 placement=Leading target=detached:range:30..31\n",
        )
    );

    assert_eq!(first.snapshot_text_with_trivia(), expected);
    assert_eq!(
        first.snapshot_text_with_trivia(),
        second.snapshot_text_with_trivia()
    );
}

#[test]
#[should_panic(expected = "SurfaceAst trivia node target must not refer to a token node")]
fn ast_rejects_token_node_as_trivia_node_target() {
    let source_id = source_id(11);
    let ast = expression_ast(source_id);
    let token = ast.token_nodes()[0];
    let mut trivia = SurfaceTriviaBuilder::new(source_id);
    trivia.add_doc_comment_attachment(
        range(source_id, 0, 8),
        TriviaAttachmentTarget::Node(TriviaNodeTarget::new(token, range(source_id, 0, 1))),
        TriviaPlacement::Leading,
    );

    let _ = ast.with_trivia(trivia.finish());
}

#[test]
#[should_panic(expected = "SurfaceAst trivia token target must refer to a token node")]
fn ast_rejects_non_token_trivia_token_target() {
    let source_id = source_id(12);
    let ast = expression_ast(source_id);
    let expression = ast.expression_root().unwrap();
    let mut trivia = SurfaceTriviaBuilder::new(source_id);
    trivia.add_skipped_token_range(
        range(source_id, 32, 35),
        Some(TriviaAttachmentTarget::Token(TriviaNodeTarget::new(
            expression,
            range(source_id, 0, 6),
        ))),
        SkippedTokenReason::UnexpectedToken,
    );

    let _ = ast.with_trivia(trivia.finish());
}

#[test]
#[should_panic(expected = "SurfaceAst trivia node target range must match the AST node range")]
fn ast_rejects_trivia_target_with_mismatched_range() {
    let source_id = source_id(13);
    let ast = expression_ast(source_id);
    let expression = ast.expression_root().unwrap();
    let mut trivia = SurfaceTriviaBuilder::new(source_id);
    trivia.add_doc_comment_attachment(
        range(source_id, 0, 8),
        TriviaAttachmentTarget::Node(TriviaNodeTarget::new(expression, range(source_id, 0, 5))),
        TriviaPlacement::Leading,
    );

    let _ = ast.with_trivia(trivia.finish());
}

#[test]
#[should_panic(expected = "SurfaceAst trivia must belong to the AST source")]
fn ast_rejects_trivia_from_another_source() {
    let ids = InMemorySessionIdAllocator::new();
    let ast_source_id = ids.next_source_id(snapshot_id(14)).unwrap();
    let trivia_source_id = ids.next_source_id(snapshot_id(15)).unwrap();
    let ast = expression_ast(ast_source_id);
    let mut trivia = SurfaceTriviaBuilder::new(trivia_source_id);
    trivia.add_doc_comment_attachment(
        range(trivia_source_id, 0, 8),
        TriviaAttachmentTarget::Detached(SourceAnchor::Point {
            source_id: trivia_source_id,
            offset: 8,
        }),
        TriviaPlacement::Leading,
    );

    let _ = ast.with_trivia(trivia.finish());
}

#[test]
#[should_panic(expected = "must have been created by this builder")]
fn builder_rejects_child_ids_not_created_by_this_builder() {
    let source_id = source_id(16);
    let other_id = {
        let mut other = SurfaceAstBuilder::new(source_id);
        other.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 0, 1))
    };
    let mut builder = SurfaceAstBuilder::new(source_id);

    builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 1),
        vec![other_id],
    );
}

#[test]
#[should_panic(expected = "cannot be shared by multiple non-root parents")]
fn builder_rejects_token_sharing_between_multiple_structural_parents() {
    let source_id = source_id(17);
    let mut builder = SurfaceAstBuilder::new(source_id);
    let token = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 0, 1));
    let left_expression = builder.add_node(
        SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
            spelling: "left".into(),
            precedence: 1,
            associativity: SurfaceOperatorAssociativity::Left,
        }),
        range(source_id, 0, 1),
        vec![token],
    );
    let right_expression = builder.add_node(
        SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
            spelling: "right".into(),
            precedence: 1,
            associativity: SurfaceOperatorAssociativity::Left,
        }),
        range(source_id, 0, 1),
        vec![token],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 1),
        vec![token, left_expression, right_expression],
    );

    let _ = builder.finish(Some(root), Some(left_expression));
}

fn scrambled_trivia(source_id: SourceId) -> SurfaceTriviaBuilder {
    let mut trivia = SurfaceTriviaBuilder::new(source_id);
    trivia.add_whitespace_hint(
        WhitespaceHintKind::SyntheticBoundary,
        range(source_id, 60, 61),
    );
    trivia.add_comment(CommentKind::SingleLine, range(source_id, 30, 39));
    trivia.add_skipped_token_range(
        range(source_id, 50, 51),
        Some(TriviaAttachmentTarget::Detached(SourceAnchor::Range(
            range(source_id, 48, 52),
        ))),
        SkippedTokenReason::UnexpectedToken,
    );
    trivia.add_doc_comment_attachment(
        range(source_id, 10, 12),
        TriviaAttachmentTarget::Detached(SourceAnchor::Range(range(source_id, 12, 13))),
        TriviaPlacement::Trailing,
    );
    trivia.add_comment(CommentKind::MultiLine, range(source_id, 2, 5));
    trivia.add_whitespace_hint(
        WhitespaceHintKind::LineBreakBefore,
        range(source_id, 14, 15),
    );
    trivia.add_skipped_token_range(
        range(source_id, 40, 44),
        None,
        SkippedTokenReason::MalformedAnnotation,
    );
    trivia.add_doc_comment_attachment(
        range(source_id, 6, 8),
        TriviaAttachmentTarget::Detached(SourceAnchor::Point {
            source_id,
            offset: 9,
        }),
        TriviaPlacement::Leading,
    );
    trivia
}

fn colliding_target_trivia(source_id: SourceId, reverse: bool) -> SurfaceTriviaBuilder {
    let mut trivia = SurfaceTriviaBuilder::new(source_id);
    let point = (
        range(source_id, 20, 22),
        TriviaAttachmentTarget::Detached(SourceAnchor::Point {
            source_id,
            offset: 30,
        }),
    );
    let range_target = (
        range(source_id, 20, 22),
        TriviaAttachmentTarget::Detached(SourceAnchor::Range(range(source_id, 30, 31))),
    );
    let entries = if reverse {
        [range_target, point]
    } else {
        [point, range_target]
    };
    for (range, target) in entries {
        trivia.add_doc_comment_attachment(range, target, TriviaPlacement::Leading);
    }
    trivia
}

fn expression_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let left = builder.add_token(SurfaceTokenKind::Identifier, "a", range(source_id, 0, 1));
    let operator = builder.add_token(SurfaceTokenKind::UserSymbol, "++", range(source_id, 2, 4));
    let right = builder.add_token(SurfaceTokenKind::Identifier, "b", range(source_id, 5, 6));
    let expression = builder.add_node(
        SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
            spelling: "++".into(),
            precedence: 10,
            associativity: SurfaceOperatorAssociativity::Left,
        }),
        range(source_id, 0, 6),
        vec![left, operator, right],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 6),
        vec![left, operator, right, expression],
    );
    builder.finish(Some(root), Some(expression))
}

fn expression_ast_with_associativity(
    source_id: SourceId,
    associativity: SurfaceOperatorAssociativity,
) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let left = builder.add_token(SurfaceTokenKind::Identifier, "a", range(source_id, 0, 1));
    let operator = builder.add_token(SurfaceTokenKind::UserSymbol, "++", range(source_id, 2, 4));
    let right = builder.add_token(SurfaceTokenKind::Identifier, "b", range(source_id, 5, 6));
    let expression = builder.add_node(
        SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
            spelling: "++".into(),
            precedence: 10,
            associativity,
        }),
        range(source_id, 0, 6),
        vec![left, operator, right],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 6),
        vec![left, operator, right, expression],
    );
    builder.finish(Some(root), Some(expression))
}

fn prefix_postfix_expression_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let prefix_token = builder.add_token(SurfaceTokenKind::UserSymbol, "~", range(source_id, 0, 1));
    let operand = builder.add_token(SurfaceTokenKind::Identifier, "a", range(source_id, 2, 3));
    let postfix_token =
        builder.add_token(SurfaceTokenKind::UserSymbol, "!", range(source_id, 4, 5));
    let prefix = builder.add_node(
        SurfaceNodeKind::PrefixExpression(SurfacePrefixOperator {
            spelling: "~".into(),
            precedence: 70,
        }),
        range(source_id, 0, 3),
        vec![prefix_token, operand],
    );
    let postfix = builder.add_node(
        SurfaceNodeKind::PostfixExpression(SurfacePostfixOperator {
            spelling: "!".into(),
            precedence: 90,
        }),
        range(source_id, 0, 5),
        vec![prefix, postfix_token],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 5),
        vec![prefix_token, operand, postfix_token, postfix],
    );
    builder.finish(Some(root), Some(postfix))
}

fn atomic_formula_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let x = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 0, 1));
    let equals = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "=",
        range(source_id, 2, 3),
    );
    let y = builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 4, 5));
    let z = builder.add_token(SurfaceTokenKind::Identifier, "z", range(source_id, 6, 7));
    let is = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "is",
        range(source_id, 8, 10),
    );
    let non = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "non",
        range(source_id, 11, 14),
    );
    let empty = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "empty",
        range(source_id, 15, 20),
    );
    let a = builder.add_token(SurfaceTokenKind::Identifier, "a", range(source_id, 21, 22));
    let divides = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "divides",
        range(source_id, 23, 30),
    );
    let b = builder.add_token(SurfaceTokenKind::Identifier, "b", range(source_id, 31, 32));
    let small = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Small",
        range(source_id, 33, 38),
    );
    let open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 38, 39),
    );
    let c = builder.add_token(SurfaceTokenKind::Identifier, "c", range(source_id, 39, 40));
    let close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 40, 41),
    );

    let x_term = term_expression_node(&mut builder, source_id, x, 0, 1);
    let y_term = term_expression_node(&mut builder, source_id, y, 4, 5);
    let builtin = builder.add_node(
        SurfaceNodeKind::BuiltinPredicateApplication,
        range(source_id, 0, 5),
        vec![x_term, equals, y_term],
    );
    let builtin_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 0, 5),
        vec![builtin],
    );

    let z_term = term_expression_node(&mut builder, source_id, z, 6, 7);
    let empty_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 15, 20),
        vec![empty],
    );
    let empty_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 15, 20),
        vec![empty_segment],
    );
    let attribute = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(source_id, 11, 20),
        vec![non, empty_symbol],
    );
    let attribute_chain = builder.add_node(
        SurfaceNodeKind::AttributeTestChain,
        range(source_id, 11, 20),
        vec![attribute],
    );
    let is_assertion = builder.add_node(
        SurfaceNodeKind::IsAssertion,
        range(source_id, 6, 20),
        vec![z_term, is, attribute_chain],
    );
    let is_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 6, 20),
        vec![is_assertion],
    );

    let a_term = term_expression_node(&mut builder, source_id, a, 21, 22);
    let b_term = term_expression_node(&mut builder, source_id, b, 31, 32);
    let predicate_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 23, 30),
        vec![divides],
    );
    let predicate_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 23, 30),
        vec![predicate_segment],
    );
    let predicate_head = builder.add_node(
        SurfaceNodeKind::PredicateHead,
        range(source_id, 23, 30),
        vec![predicate_symbol],
    );
    let segment = builder.add_node(
        SurfaceNodeKind::PredicateSegment,
        range(source_id, 21, 32),
        vec![a_term, predicate_head, b_term],
    );
    let predicate_application = builder.add_node(
        SurfaceNodeKind::PredicateApplication,
        range(source_id, 21, 32),
        vec![segment],
    );
    let predicate_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 21, 32),
        vec![predicate_application],
    );

    let c_term = term_expression_node(&mut builder, source_id, c, 39, 40);
    let inline = builder.add_node(
        SurfaceNodeKind::InlinePredicateApplication,
        range(source_id, 33, 41),
        vec![small, open, c_term, close],
    );
    let inline_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 33, 41),
        vec![inline],
    );

    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 41),
        vec![
            x,
            equals,
            y,
            z,
            is,
            non,
            empty,
            a,
            divides,
            b,
            small,
            open,
            c,
            close,
            builtin_formula,
            is_formula,
            predicate_formula,
            inline_formula,
        ],
    );
    builder.finish(Some(root), Some(builtin_formula))
}

fn formula_surface_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let not = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "not",
        range(source_id, 0, 3),
    );
    let x = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 4, 5));
    let equals = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "=",
        range(source_id, 6, 7),
    );
    let y = builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 8, 9));
    let and = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "&",
        range(source_id, 10, 11),
    );
    let ellipsis = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "...",
        range(source_id, 12, 15),
    );
    let repeated_and = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "&",
        range(source_id, 16, 17),
    );
    let thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 18, 24),
    );
    let open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 25, 26),
    );
    let contradiction = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "contradiction",
        range(source_id, 26, 39),
    );
    let close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 39, 40),
    );
    let or = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "or",
        range(source_id, 41, 43),
    );
    let contradiction_right = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "contradiction",
        range(source_id, 44, 57),
    );
    let implies_left = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 58, 64),
    );
    let implies = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "implies",
        range(source_id, 65, 72),
    );
    let implies_right = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "contradiction",
        range(source_id, 73, 86),
    );
    let iff_left = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 87, 93),
    );
    let iff = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "iff",
        range(source_id, 94, 97),
    );
    let iff_right = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "contradiction",
        range(source_id, 98, 111),
    );
    let for_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "for",
        range(source_id, 112, 115),
    );
    let qx = builder.add_token(
        SurfaceTokenKind::Identifier,
        "u",
        range(source_id, 116, 117),
    );
    let being = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "being",
        range(source_id, 118, 123),
    );
    let set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 124, 127),
    );
    let st = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "st",
        range(source_id, 128, 130),
    );
    let condition = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 131, 137),
    );
    let holds = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "holds",
        range(source_id, 138, 143),
    );
    let universal_body = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "contradiction",
        range(source_id, 144, 157),
    );
    let ex_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "ex",
        range(source_id, 158, 160),
    );
    let ex_var = builder.add_token(
        SurfaceTokenKind::Identifier,
        "v",
        range(source_id, 161, 162),
    );
    let ex_st = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "st",
        range(source_id, 163, 165),
    );
    let ex_body = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 166, 172),
    );

    let x_term = term_expression_node(&mut builder, source_id, x, 4, 5);
    let y_term = term_expression_node(&mut builder, source_id, y, 8, 9);
    let builtin = builder.add_node(
        SurfaceNodeKind::BuiltinPredicateApplication,
        range(source_id, 4, 9),
        vec![x_term, equals, y_term],
    );
    let prefix = builder.add_node(
        SurfaceNodeKind::PrefixFormula(SurfaceFormulaPrefixOperator::Not),
        range(source_id, 0, 9),
        vec![not, builtin],
    );
    let thesis_formula = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 18, 24),
        vec![thesis],
    );
    let binary_and = builder.add_node(
        SurfaceNodeKind::BinaryFormula(SurfaceFormulaBinaryOperator {
            connective: SurfaceFormulaConnective::And,
            repeated: true,
        }),
        range(source_id, 0, 24),
        vec![prefix, and, ellipsis, repeated_and, thesis_formula],
    );

    let contradiction_formula = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
        range(source_id, 26, 39),
        vec![contradiction],
    );
    let contradiction_expression = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 26, 39),
        vec![contradiction_formula],
    );
    let parenthesized = builder.add_node(
        SurfaceNodeKind::ParenthesizedFormula,
        range(source_id, 25, 40),
        vec![open, contradiction_expression, close],
    );
    let contradiction_right_formula = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
        range(source_id, 44, 57),
        vec![contradiction_right],
    );
    let binary_or = builder.add_node(
        SurfaceNodeKind::BinaryFormula(SurfaceFormulaBinaryOperator {
            connective: SurfaceFormulaConnective::Or,
            repeated: false,
        }),
        range(source_id, 25, 57),
        vec![parenthesized, or, contradiction_right_formula],
    );

    let implies_left_formula = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 58, 64),
        vec![implies_left],
    );
    let implies_right_formula = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
        range(source_id, 73, 86),
        vec![implies_right],
    );
    let binary_implies = builder.add_node(
        SurfaceNodeKind::BinaryFormula(SurfaceFormulaBinaryOperator {
            connective: SurfaceFormulaConnective::Implies,
            repeated: false,
        }),
        range(source_id, 58, 86),
        vec![implies_left_formula, implies, implies_right_formula],
    );

    let iff_left_formula = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 87, 93),
        vec![iff_left],
    );
    let iff_right_formula = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
        range(source_id, 98, 111),
        vec![iff_right],
    );
    let binary_iff = builder.add_node(
        SurfaceNodeKind::BinaryFormula(SurfaceFormulaBinaryOperator {
            connective: SurfaceFormulaConnective::Iff,
            repeated: false,
        }),
        range(source_id, 87, 111),
        vec![iff_left_formula, iff, iff_right_formula],
    );

    let quantifier_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 124, 127),
        vec![set],
    );
    let quantifier_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 124, 127),
        vec![quantifier_type_head],
    );
    let universal_segment = builder.add_node(
        SurfaceNodeKind::QuantifierVariableSegment,
        range(source_id, 116, 127),
        vec![qx, being, quantifier_type],
    );
    let condition_formula = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 131, 137),
        vec![condition],
    );
    let universal_body_formula = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
        range(source_id, 144, 157),
        vec![universal_body],
    );
    let universal = builder.add_node(
        SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Universal),
        range(source_id, 112, 157),
        vec![
            for_keyword,
            universal_segment,
            st,
            condition_formula,
            holds,
            universal_body_formula,
        ],
    );

    let existential_segment = builder.add_node(
        SurfaceNodeKind::QuantifierVariableSegment,
        range(source_id, 161, 162),
        vec![ex_var],
    );
    let existential_body_formula = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 166, 172),
        vec![ex_body],
    );
    let existential = builder.add_node(
        SurfaceNodeKind::QuantifiedFormula(SurfaceQuantifierKind::Existential),
        range(source_id, 158, 172),
        vec![
            ex_keyword,
            existential_segment,
            ex_st,
            existential_body_formula,
        ],
    );

    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 172),
        vec![
            not,
            x,
            equals,
            y,
            and,
            ellipsis,
            repeated_and,
            thesis,
            open,
            contradiction,
            close,
            or,
            contradiction_right,
            implies_left,
            implies,
            implies_right,
            iff_left,
            iff,
            iff_right,
            for_keyword,
            qx,
            being,
            set,
            st,
            condition,
            holds,
            universal_body,
            ex_keyword,
            ex_var,
            ex_st,
            ex_body,
            binary_and,
            binary_or,
            binary_implies,
            binary_iff,
            universal,
            existential,
        ],
    );
    builder.finish(Some(root), Some(binary_and))
}

fn set_comprehension_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "{",
        range(source_id, 0, 1),
    );
    let mapper = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 2, 3));
    let where_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "where",
        range(source_id, 4, 9),
    );
    let generator = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 10, 11));
    let is_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "is",
        range(source_id, 12, 14),
    );
    let set_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 15, 18),
    );
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 19, 20),
    );
    let thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 21, 27),
    );
    let close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "}",
        range(source_id, 28, 29),
    );
    let mapper_expression = term_expression_node(&mut builder, source_id, mapper, 2, 3);
    let type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 15, 18),
        vec![set_token],
    );
    let type_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 15, 18),
        vec![type_head],
    );
    let segment = builder.add_node(
        SurfaceNodeKind::ComprehensionVariableSegment,
        range(source_id, 10, 18),
        vec![generator, is_token, type_expression],
    );
    let thesis_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 21, 27),
        vec![thesis],
    );
    let formula_expression = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 21, 27),
        vec![thesis_constant],
    );
    let comprehension = builder.add_node(
        SurfaceNodeKind::SetComprehension,
        range(source_id, 0, 29),
        vec![
            open,
            mapper_expression,
            where_token,
            segment,
            colon,
            formula_expression,
            close,
        ],
    );
    let term_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 0, 29),
        vec![comprehension],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 29),
        vec![
            open,
            mapper,
            where_token,
            generator,
            is_token,
            set_token,
            colon,
            thesis,
            close,
            term_expression,
        ],
    );
    builder.finish(Some(root), Some(comprehension))
}

fn statement_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let let_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, 0, 3),
    );
    let let_identifier =
        builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 4, 5));
    let be_token = builder.add_token(SurfaceTokenKind::ReservedWord, "be", range(source_id, 6, 8));
    let let_type_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 9, 12),
    );
    let let_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 12, 13),
    );
    let let_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 9, 12),
        vec![let_type_token],
    );
    let let_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 9, 12),
        vec![let_type_head],
    );
    let let_segment = builder.add_node(
        SurfaceNodeKind::QualifiedVariableSegment,
        range(source_id, 4, 12),
        vec![let_identifier, be_token, let_type],
    );
    let let_statement = builder.add_node(
        SurfaceNodeKind::LetStatement,
        range(source_id, 0, 13),
        vec![let_token, let_segment, let_semicolon],
    );
    let let_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 0, 13),
        vec![let_statement],
    );

    let assume_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "assume",
        range(source_id, 14, 20),
    );
    let assumption_that = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "that",
        range(source_id, 21, 25),
    );
    let assumption_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 26, 32),
    );
    let assumption_and = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "and",
        range(source_id, 33, 36),
    );
    let assumption_contradiction = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "contradiction",
        range(source_id, 37, 50),
    );
    let assume_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 50, 51),
    );
    let thesis_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 26, 32),
        vec![assumption_thesis],
    );
    let thesis_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 26, 32),
        vec![thesis_constant],
    );
    let first_proposition = builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, 26, 32),
        vec![thesis_formula],
    );
    let contradiction_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
        range(source_id, 37, 50),
        vec![assumption_contradiction],
    );
    let contradiction_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 37, 50),
        vec![contradiction_constant],
    );
    let second_proposition = builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, 37, 50),
        vec![contradiction_formula],
    );
    let assumption_conditions = builder.add_node(
        SurfaceNodeKind::ConditionList,
        range(source_id, 21, 50),
        vec![
            assumption_that,
            first_proposition,
            assumption_and,
            second_proposition,
        ],
    );
    let assumption_statement = builder.add_node(
        SurfaceNodeKind::AssumptionStatement,
        range(source_id, 14, 51),
        vec![assume_token, assumption_conditions, assume_semicolon],
    );
    let assumption_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 14, 51),
        vec![assumption_statement],
    );

    let given_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "given",
        range(source_id, 52, 57),
    );
    let given_identifier =
        builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 58, 59));
    let being_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "being",
        range(source_id, 60, 65),
    );
    let given_type_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 66, 69),
    );
    let such_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "such",
        range(source_id, 70, 74),
    );
    let given_that = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "that",
        range(source_id, 75, 79),
    );
    let given_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 80, 86),
    );
    let given_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 86, 87),
    );
    let given_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 66, 69),
        vec![given_type_token],
    );
    let given_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 66, 69),
        vec![given_type_head],
    );
    let given_segment = builder.add_node(
        SurfaceNodeKind::QualifiedVariableSegment,
        range(source_id, 58, 69),
        vec![given_identifier, being_token, given_type],
    );
    let given_thesis_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 80, 86),
        vec![given_thesis],
    );
    let given_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 80, 86),
        vec![given_thesis_constant],
    );
    let given_proposition = builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, 80, 86),
        vec![given_formula],
    );
    let given_conditions = builder.add_node(
        SurfaceNodeKind::ConditionList,
        range(source_id, 75, 86),
        vec![given_that, given_proposition],
    );
    let given_statement = builder.add_node(
        SurfaceNodeKind::GivenStatement,
        range(source_id, 52, 87),
        vec![
            given_token,
            given_segment,
            such_token,
            given_conditions,
            given_semicolon,
        ],
    );
    let given_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 52, 87),
        vec![given_statement],
    );

    let take_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "take",
        range(source_id, 88, 92),
    );
    let witness_identifier =
        builder.add_token(SurfaceTokenKind::Identifier, "z", range(source_id, 93, 94));
    let witness_equals = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "=",
        range(source_id, 95, 96),
    );
    let witness_value =
        builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 97, 98));
    let witness_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 98, 99),
    );
    let second_witness_value = builder.add_token(
        SurfaceTokenKind::Identifier,
        "y",
        range(source_id, 100, 101),
    );
    let take_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 101, 102),
    );
    let witness_term = term_expression_node(&mut builder, source_id, witness_value, 97, 98);
    let named_witness = builder.add_node(
        SurfaceNodeKind::Witness,
        range(source_id, 93, 98),
        vec![witness_identifier, witness_equals, witness_term],
    );
    let second_witness_term =
        term_expression_node(&mut builder, source_id, second_witness_value, 100, 101);
    let second_witness = builder.add_node(
        SurfaceNodeKind::Witness,
        range(source_id, 100, 101),
        vec![second_witness_term],
    );
    let take_statement = builder.add_node(
        SurfaceNodeKind::TakeStatement,
        range(source_id, 88, 102),
        vec![
            take_token,
            named_witness,
            witness_comma,
            second_witness,
            take_semicolon,
        ],
    );
    let take_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 88, 102),
        vec![take_statement],
    );

    let set_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 103, 106),
    );
    let equating_identifier = builder.add_token(
        SurfaceTokenKind::Identifier,
        "f",
        range(source_id, 107, 108),
    );
    let equating_equals = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "=",
        range(source_id, 109, 110),
    );
    let equating_value = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 111, 112),
    );
    let set_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 112, 113),
    );
    let equating_term = term_expression_node(&mut builder, source_id, equating_value, 111, 112);
    let equating = builder.add_node(
        SurfaceNodeKind::Equating,
        range(source_id, 107, 112),
        vec![equating_identifier, equating_equals, equating_term],
    );
    let set_statement = builder.add_node(
        SurfaceNodeKind::SetStatement,
        range(source_id, 103, 113),
        vec![set_token, equating, set_semicolon],
    );
    let set_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 103, 113),
        vec![set_statement],
    );

    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 113),
        vec![let_item, assumption_item, given_item, take_item, set_item],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 113),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 113),
        vec![
            let_token,
            let_identifier,
            be_token,
            let_type_token,
            let_semicolon,
            assume_token,
            assumption_that,
            assumption_thesis,
            assumption_and,
            assumption_contradiction,
            assume_semicolon,
            given_token,
            given_identifier,
            being_token,
            given_type_token,
            such_token,
            given_that,
            given_thesis,
            given_semicolon,
            take_token,
            witness_identifier,
            witness_equals,
            witness_value,
            witness_comma,
            second_witness_value,
            take_semicolon,
            set_token,
            equating_identifier,
            equating_equals,
            equating_value,
            set_semicolon,
            compilation_unit,
        ],
    );
    builder.finish(Some(root), None)
}

fn justification_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);

    let first_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 0, 6),
    );
    let first_by = builder.add_token(SurfaceTokenKind::ReservedWord, "by", range(source_id, 7, 9));
    let local_label =
        builder.add_token(SurfaceTokenKind::Identifier, "A", range(source_id, 10, 11));
    let first_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 11, 12),
    );
    let qualified_namespace_a = builder.add_token(
        SurfaceTokenKind::Identifier,
        "mml",
        range(source_id, 13, 16),
    );
    let qualified_namespace_dot = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 16, 17),
    );
    let qualified_namespace_b = builder.add_token(
        SurfaceTokenKind::Identifier,
        "foo",
        range(source_id, 17, 20),
    );
    let qualified_final_dot = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 20, 21),
    );
    let qualified_label = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Th1",
        range(source_id, 21, 24),
    );
    let second_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 24, 25),
    );
    let grouped_namespace_a = builder.add_token(
        SurfaceTokenKind::Identifier,
        "mml",
        range(source_id, 26, 29),
    );
    let grouped_namespace_dot = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 29, 30),
    );
    let grouped_namespace_b = builder.add_token(
        SurfaceTokenKind::Identifier,
        "foo",
        range(source_id, 30, 33),
    );
    let grouped_opener = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".{",
        range(source_id, 33, 35),
    );
    let grouped_label_a =
        builder.add_token(SurfaceTokenKind::Identifier, "G1", range(source_id, 35, 37));
    let grouped_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 37, 38),
    );
    let grouped_label_b =
        builder.add_token(SurfaceTokenKind::Identifier, "G2", range(source_id, 39, 41));
    let grouped_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "}",
        range(source_id, 41, 42),
    );
    let third_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 42, 43),
    );
    let bulk_namespace_a = builder.add_token(
        SurfaceTokenKind::Identifier,
        "mml",
        range(source_id, 44, 47),
    );
    let bulk_namespace_dot = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 47, 48),
    );
    let bulk_namespace_b = builder.add_token(
        SurfaceTokenKind::Identifier,
        "foo",
        range(source_id, 48, 51),
    );
    let bulk_operator = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".*",
        range(source_id, 51, 53),
    );
    let first_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 53, 54),
    );
    let second_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 55, 61),
    );
    let second_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 62, 64),
    );
    let computation = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "computation",
        range(source_id, 65, 76),
    );
    let computation_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 76, 77),
    );
    let steps = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "steps",
        range(source_id, 77, 82),
    );
    let option_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 82, 83),
    );
    let option_value = builder.add_token(SurfaceTokenKind::Numeral, "10", range(source_id, 84, 86));
    let computation_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 86, 87),
    );
    let second_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 87, 88),
    );

    let first_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 0, 6),
        vec![first_thesis],
    );
    let first_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 0, 6),
        vec![first_constant],
    );
    let first_proposition = builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, 0, 6),
        vec![first_formula],
    );
    let local_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 10, 11),
        vec![local_label],
    );
    let qualified_segment_a = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 13, 16),
        vec![qualified_namespace_a],
    );
    let qualified_segment_b = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 17, 20),
        vec![qualified_namespace_b],
    );
    let qualified_namespace = builder.add_node(
        SurfaceNodeKind::NamespacePath,
        range(source_id, 13, 20),
        vec![
            qualified_segment_a,
            qualified_namespace_dot,
            qualified_segment_b,
        ],
    );
    let qualified_reference = builder.add_node(
        SurfaceNodeKind::QualifiedReference,
        range(source_id, 13, 24),
        vec![qualified_namespace, qualified_final_dot, qualified_label],
    );
    let grouped_segment_a = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 26, 29),
        vec![grouped_namespace_a],
    );
    let grouped_segment_b = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 30, 33),
        vec![grouped_namespace_b],
    );
    let grouped_namespace = builder.add_node(
        SurfaceNodeKind::NamespacePath,
        range(source_id, 26, 33),
        vec![grouped_segment_a, grouped_namespace_dot, grouped_segment_b],
    );
    let grouped_item_a = builder.add_node(
        SurfaceNodeKind::GroupedReferenceItem,
        range(source_id, 35, 37),
        vec![grouped_label_a],
    );
    let grouped_item_b = builder.add_node(
        SurfaceNodeKind::GroupedReferenceItem,
        range(source_id, 39, 41),
        vec![grouped_label_b],
    );
    let grouped_reference = builder.add_node(
        SurfaceNodeKind::GroupedReference,
        range(source_id, 26, 42),
        vec![
            grouped_namespace,
            grouped_opener,
            grouped_item_a,
            grouped_comma,
            grouped_item_b,
            grouped_close,
        ],
    );
    let bulk_segment_a = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 44, 47),
        vec![bulk_namespace_a],
    );
    let bulk_segment_b = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 48, 51),
        vec![bulk_namespace_b],
    );
    let bulk_namespace = builder.add_node(
        SurfaceNodeKind::NamespacePath,
        range(source_id, 44, 51),
        vec![bulk_segment_a, bulk_namespace_dot, bulk_segment_b],
    );
    let bulk_reference = builder.add_node(
        SurfaceNodeKind::BulkReference,
        range(source_id, 44, 53),
        vec![bulk_namespace, bulk_operator],
    );
    let reference_list = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 10, 53),
        vec![
            local_reference,
            first_comma,
            qualified_reference,
            second_comma,
            grouped_reference,
            third_comma,
            bulk_reference,
        ],
    );
    let reference_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 7, 53),
        vec![first_by, reference_list],
    );
    let reference_statement = builder.add_node(
        SurfaceNodeKind::CompactStatement,
        range(source_id, 0, 54),
        vec![first_proposition, reference_justification, first_semicolon],
    );
    let reference_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 0, 54),
        vec![reference_statement],
    );

    let second_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 55, 61),
        vec![second_thesis],
    );
    let second_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 55, 61),
        vec![second_constant],
    );
    let second_proposition = builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, 55, 61),
        vec![second_formula],
    );
    let computation_option = builder.add_node(
        SurfaceNodeKind::ComputationOption,
        range(source_id, 77, 86),
        vec![steps, option_colon, option_value],
    );
    let computation_justification = builder.add_node(
        SurfaceNodeKind::ComputationJustification,
        range(source_id, 65, 87),
        vec![
            computation,
            computation_open,
            computation_option,
            computation_close,
        ],
    );
    let computation_clause = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 62, 87),
        vec![second_by, computation_justification],
    );
    let computation_statement = builder.add_node(
        SurfaceNodeKind::CompactStatement,
        range(source_id, 55, 88),
        vec![second_proposition, computation_clause, second_semicolon],
    );
    let computation_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 55, 88),
        vec![computation_statement],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 88),
        vec![reference_item, computation_item],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 88),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 88),
        vec![
            first_thesis,
            first_by,
            local_label,
            first_comma,
            qualified_namespace_a,
            qualified_namespace_dot,
            qualified_namespace_b,
            qualified_final_dot,
            qualified_label,
            second_comma,
            grouped_namespace_a,
            grouped_namespace_dot,
            grouped_namespace_b,
            grouped_opener,
            grouped_label_a,
            grouped_comma,
            grouped_label_b,
            grouped_close,
            third_comma,
            bulk_namespace_a,
            bulk_namespace_dot,
            bulk_namespace_b,
            bulk_operator,
            first_semicolon,
            second_thesis,
            second_by,
            computation,
            computation_open,
            steps,
            option_colon,
            option_value,
            computation_close,
            second_semicolon,
            compilation_unit,
        ],
    );
    builder.finish(Some(root), None)
}

fn task18_statement_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let consider = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "consider",
        range(source_id, 0, 8),
    );
    let x = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 9, 10));
    let shared_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 10, 11),
    );
    let y = builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 12, 13));
    let being = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "being",
        range(source_id, 14, 19),
    );
    let consider_type_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 20, 23),
    );
    let such = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "such",
        range(source_id, 24, 28),
    );
    let that = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "that",
        range(source_id, 29, 33),
    );
    let label = builder.add_token(SurfaceTokenKind::Identifier, "A", range(source_id, 34, 35));
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 35, 36),
    );
    let thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 37, 43),
    );
    let and = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "and",
        range(source_id, 44, 47),
    );
    let contradiction = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "contradiction",
        range(source_id, 48, 61),
    );
    let consider_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 62, 64),
    );
    let reference_label =
        builder.add_token(SurfaceTokenKind::Identifier, "A", range(source_id, 65, 66));
    let consider_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 67, 68),
    );

    let reconsider = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "reconsider",
        range(source_id, 70, 80),
    );
    let reconsider_x =
        builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 81, 82));
    let item_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 82, 83),
    );
    let reconsider_y =
        builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 84, 85));
    let equals = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "=",
        range(source_id, 86, 87),
    );
    let z = builder.add_token(SurfaceTokenKind::Identifier, "z", range(source_id, 88, 89));
    let as_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "as",
        range(source_id, 90, 92),
    );
    let reconsider_type_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 93, 96),
    );
    let reconsider_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 97, 99),
    );
    let reconsider_reference_label = builder.add_token(
        SurfaceTokenKind::Identifier,
        "A",
        range(source_id, 100, 101),
    );
    let reconsider_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 102, 103),
    );

    let consider_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 20, 23),
        vec![consider_type_token],
    );
    let consider_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 20, 23),
        vec![consider_type_head],
    );
    let segment = builder.add_node(
        SurfaceNodeKind::QualifiedVariableSegment,
        range(source_id, 9, 23),
        vec![x, shared_comma, y, being, consider_type],
    );
    let thesis_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 37, 43),
        vec![thesis],
    );
    let thesis_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 37, 43),
        vec![thesis_constant],
    );
    let labelled_proposition = builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, 34, 43),
        vec![label, colon, thesis_formula],
    );
    let contradiction_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
        range(source_id, 48, 61),
        vec![contradiction],
    );
    let contradiction_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 48, 61),
        vec![contradiction_constant],
    );
    let unlabelled_proposition = builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, 48, 61),
        vec![contradiction_formula],
    );
    let conditions = builder.add_node(
        SurfaceNodeKind::ConditionList,
        range(source_id, 29, 61),
        vec![that, labelled_proposition, and, unlabelled_proposition],
    );
    let reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 65, 66),
        vec![reference_label],
    );
    let reference_list = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 65, 66),
        vec![reference],
    );
    let justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 62, 66),
        vec![consider_by, reference_list],
    );
    let consider_statement = builder.add_node(
        SurfaceNodeKind::ConsiderStatement,
        range(source_id, 0, 68),
        vec![
            consider,
            segment,
            such,
            conditions,
            justification,
            consider_semicolon,
        ],
    );
    let consider_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 0, 68),
        vec![consider_statement],
    );

    let bare_reconsider_item = builder.add_node(
        SurfaceNodeKind::ReconsiderItem,
        range(source_id, 81, 82),
        vec![reconsider_x],
    );
    let z_term = term_expression_node(&mut builder, source_id, z, 88, 89);
    let equated_reconsider_item = builder.add_node(
        SurfaceNodeKind::ReconsiderItem,
        range(source_id, 84, 89),
        vec![reconsider_y, equals, z_term],
    );
    let reconsider_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 93, 96),
        vec![reconsider_type_token],
    );
    let reconsider_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 93, 96),
        vec![reconsider_type_head],
    );
    let reconsider_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 100, 101),
        vec![reconsider_reference_label],
    );
    let reconsider_reference_list = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 100, 101),
        vec![reconsider_reference],
    );
    let reconsider_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 97, 101),
        vec![reconsider_by, reconsider_reference_list],
    );
    let reconsider_statement = builder.add_node(
        SurfaceNodeKind::ReconsiderStatement,
        range(source_id, 70, 103),
        vec![
            reconsider,
            bare_reconsider_item,
            item_comma,
            equated_reconsider_item,
            as_token,
            reconsider_type,
            reconsider_justification,
            reconsider_semicolon,
        ],
    );
    let reconsider_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 70, 103),
        vec![reconsider_statement],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 103),
        vec![consider_item, reconsider_item],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 103),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 103),
        vec![
            consider,
            x,
            shared_comma,
            y,
            being,
            consider_type_token,
            such,
            that,
            label,
            colon,
            thesis,
            and,
            contradiction,
            consider_by,
            reference_label,
            consider_semicolon,
            reconsider,
            reconsider_x,
            item_comma,
            reconsider_y,
            equals,
            z,
            as_token,
            reconsider_type_token,
            reconsider_by,
            reconsider_reference_label,
            reconsider_semicolon,
            compilation_unit,
        ],
    );
    builder.finish(Some(root), None)
}

fn task19_statement_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let thus = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thus",
        range(source_id, 0, 4),
    );
    let thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 5, 11),
    );
    let first_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 11, 12),
    );
    let then = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "then",
        range(source_id, 13, 17),
    );
    let x = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 18, 19));
    let equals = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "=",
        range(source_id, 20, 21),
    );
    let y = builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 22, 23));
    let first_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 24, 26),
    );
    let a = builder.add_token(SurfaceTokenKind::Identifier, "A", range(source_id, 27, 28));
    let dot_equals = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".=",
        range(source_id, 29, 31),
    );
    let z = builder.add_token(SurfaceTokenKind::Identifier, "z", range(source_id, 32, 33));
    let second_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 34, 36),
    );
    let b = builder.add_token(SurfaceTokenKind::Identifier, "B", range(source_id, 37, 38));
    let second_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 38, 39),
    );

    let thesis_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, 5, 11),
        vec![thesis],
    );
    let thesis_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 5, 11),
        vec![thesis_constant],
    );
    let proposition = builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, 5, 11),
        vec![thesis_formula],
    );
    let conclusion = builder.add_node(
        SurfaceNodeKind::ConclusionStatement,
        range(source_id, 0, 12),
        vec![thus, proposition, first_semicolon],
    );
    let conclusion_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 0, 12),
        vec![conclusion],
    );

    let x_term = term_expression_node(&mut builder, source_id, x, 18, 19);
    let y_term = term_expression_node(&mut builder, source_id, y, 22, 23);
    let z_term = term_expression_node(&mut builder, source_id, z, 32, 33);
    let first_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 27, 28),
        vec![a],
    );
    let first_reference_list = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 27, 28),
        vec![first_reference],
    );
    let first_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 24, 28),
        vec![first_by, first_reference_list],
    );
    let second_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 37, 38),
        vec![b],
    );
    let second_reference_list = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 37, 38),
        vec![second_reference],
    );
    let second_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 34, 38),
        vec![second_by, second_reference_list],
    );
    let iterative_step = builder.add_node(
        SurfaceNodeKind::IterativeEqualityStep,
        range(source_id, 29, 38),
        vec![dot_equals, z_term, second_justification],
    );
    let iterative_statement = builder.add_node(
        SurfaceNodeKind::IterativeEqualityStatement,
        range(source_id, 18, 39),
        vec![
            x_term,
            equals,
            y_term,
            first_justification,
            iterative_step,
            second_semicolon,
        ],
    );
    let then_statement = builder.add_node(
        SurfaceNodeKind::ThenStatement,
        range(source_id, 13, 39),
        vec![then, iterative_statement],
    );
    let then_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 13, 39),
        vec![then_statement],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 39),
        vec![conclusion_item, then_item],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 39),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 39),
        vec![
            thus,
            thesis,
            first_semicolon,
            then,
            x,
            equals,
            y,
            first_by,
            a,
            dot_equals,
            z,
            second_by,
            b,
            second_semicolon,
            compilation_unit,
        ],
    );
    builder.finish(Some(root), None)
}

fn task20_statement_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let a1 = builder.add_token(SurfaceTokenKind::Identifier, "A1", range(source_id, 0, 2));
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 2, 3),
    );
    let now = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "now",
        range(source_id, 4, 7),
    );
    let first_end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 8, 11),
    );
    let first_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 11, 12),
    );
    let hereby = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "hereby",
        range(source_id, 13, 19),
    );
    let second_end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 20, 23),
    );
    let second_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 23, 24),
    );
    let first_per = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "per",
        range(source_id, 25, 28),
    );
    let first_cases = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "cases",
        range(source_id, 29, 34),
    );
    let third_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 34, 35),
    );
    let case = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "case",
        range(source_id, 36, 40),
    );
    let case_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 41, 47),
    );
    let fourth_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 47, 48),
    );
    let third_end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 49, 52),
    );
    let fifth_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 52, 53),
    );
    let second_per = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "per",
        range(source_id, 54, 57),
    );
    let second_cases = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "cases",
        range(source_id, 58, 63),
    );
    let by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 64, 66),
    );
    let a = builder.add_token(SurfaceTokenKind::Identifier, "A", range(source_id, 67, 68));
    let sixth_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 68, 69),
    );
    let suppose = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "suppose",
        range(source_id, 70, 77),
    );
    let that = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "that",
        range(source_id, 78, 82),
    );
    let suppose_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 83, 89),
    );
    let seventh_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 89, 90),
    );
    let fourth_end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 91, 94),
    );
    let eighth_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 94, 95),
    );

    let now_statement = builder.add_node(
        SurfaceNodeKind::NowStatement,
        range(source_id, 0, 12),
        vec![a1, colon, now, first_end, first_semicolon],
    );
    let now_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 0, 12),
        vec![now_statement],
    );
    let hereby_statement = builder.add_node(
        SurfaceNodeKind::HerebyStatement,
        range(source_id, 13, 24),
        vec![hereby, second_end, second_semicolon],
    );
    let hereby_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 13, 24),
        vec![hereby_statement],
    );

    let case_proposition = thesis_proposition_node(&mut builder, source_id, case_thesis, 41, 47);
    let case_item = builder.add_node(
        SurfaceNodeKind::CaseItem,
        range(source_id, 36, 53),
        vec![
            case,
            case_proposition,
            fourth_semicolon,
            third_end,
            fifth_semicolon,
        ],
    );
    let case_reasoning = builder.add_node(
        SurfaceNodeKind::CaseReasoningStatement,
        range(source_id, 25, 53),
        vec![first_per, first_cases, third_semicolon, case_item],
    );
    let case_reasoning_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 25, 53),
        vec![case_reasoning],
    );

    let reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 67, 68),
        vec![a],
    );
    let reference_list = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 67, 68),
        vec![reference],
    );
    let justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 64, 68),
        vec![by, reference_list],
    );
    let suppose_proposition =
        thesis_proposition_node(&mut builder, source_id, suppose_thesis, 83, 89);
    let condition_list = builder.add_node(
        SurfaceNodeKind::ConditionList,
        range(source_id, 78, 89),
        vec![that, suppose_proposition],
    );
    let suppose_item = builder.add_node(
        SurfaceNodeKind::SupposeItem,
        range(source_id, 70, 95),
        vec![
            suppose,
            condition_list,
            seventh_semicolon,
            fourth_end,
            eighth_semicolon,
        ],
    );
    let suppose_reasoning = builder.add_node(
        SurfaceNodeKind::CaseReasoningStatement,
        range(source_id, 54, 95),
        vec![
            second_per,
            second_cases,
            justification,
            sixth_semicolon,
            suppose_item,
        ],
    );
    let suppose_reasoning_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 54, 95),
        vec![suppose_reasoning],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 95),
        vec![
            now_item,
            hereby_item,
            case_reasoning_item,
            suppose_reasoning_item,
        ],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 95),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 95),
        vec![
            a1,
            colon,
            now,
            first_end,
            first_semicolon,
            hereby,
            second_end,
            second_semicolon,
            first_per,
            first_cases,
            third_semicolon,
            case,
            case_thesis,
            fourth_semicolon,
            third_end,
            fifth_semicolon,
            second_per,
            second_cases,
            by,
            a,
            sixth_semicolon,
            suppose,
            that,
            suppose_thesis,
            seventh_semicolon,
            fourth_end,
            eighth_semicolon,
            compilation_unit,
        ],
    );
    builder.finish(Some(root), None)
}

fn task21_statement_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let deffunc = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "deffunc",
        range(source_id, 0, 7),
    );
    let f = builder.add_token(SurfaceTokenKind::Identifier, "F", range(source_id, 8, 9));
    let first_lparen = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 9, 10),
    );
    let x_param = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 10, 11));
    let be = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "be",
        range(source_id, 12, 14),
    );
    let first_nat = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Nat",
        range(source_id, 15, 18),
    );
    let first_rparen = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 18, 19),
    );
    let arrow = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "->",
        range(source_id, 20, 22),
    );
    let second_nat = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Nat",
        range(source_id, 23, 26),
    );
    let equals = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "equals",
        range(source_id, 27, 33),
    );
    let x_body = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 34, 35));
    let first_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 35, 36),
    );
    let defpred = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "defpred",
        range(source_id, 37, 44),
    );
    let p = builder.add_token(SurfaceTokenKind::Identifier, "P", range(source_id, 45, 46));
    let second_lparen = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 46, 47),
    );
    let second_rparen = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 47, 48),
    );
    let means = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "means",
        range(source_id, 49, 54),
    );
    let thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 55, 61),
    );
    let second_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 61, 62),
    );

    let parameter_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 15, 18),
        vec![first_nat],
    );
    let typed_parameter = builder.add_node(
        SurfaceNodeKind::TypedParameter,
        range(source_id, 10, 18),
        vec![x_param, be, parameter_type],
    );
    let return_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 23, 26),
        vec![second_nat],
    );
    let body_term = term_expression_node(&mut builder, source_id, x_body, 34, 35);
    let functor_definition = builder.add_node(
        SurfaceNodeKind::InlineFunctorDefinition,
        range(source_id, 0, 36),
        vec![
            deffunc,
            f,
            first_lparen,
            typed_parameter,
            first_rparen,
            arrow,
            return_type,
            equals,
            body_term,
            first_semicolon,
        ],
    );
    let functor_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 0, 36),
        vec![functor_definition],
    );

    let formula = thesis_formula_node(&mut builder, source_id, thesis, 55, 61);
    let predicate_definition = builder.add_node(
        SurfaceNodeKind::InlinePredicateDefinition,
        range(source_id, 37, 62),
        vec![
            defpred,
            p,
            second_lparen,
            second_rparen,
            means,
            formula,
            second_semicolon,
        ],
    );
    let predicate_item = builder.add_node(
        SurfaceNodeKind::StatementItem,
        range(source_id, 37, 62),
        vec![predicate_definition],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 62),
        vec![functor_item, predicate_item],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 62),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 62),
        vec![
            deffunc,
            f,
            first_lparen,
            x_param,
            be,
            first_nat,
            first_rparen,
            arrow,
            second_nat,
            equals,
            x_body,
            first_semicolon,
            defpred,
            p,
            second_lparen,
            second_rparen,
            means,
            thesis,
            second_semicolon,
            compilation_unit,
        ],
    );
    builder.finish(Some(root), None)
}

fn task22_theorem_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let theorem = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 0, 7),
    );
    let t = builder.add_token(SurfaceTokenKind::Identifier, "T", range(source_id, 8, 9));
    let first_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 9, 10),
    );
    let first_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 11, 17),
    );
    let proof = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "proof",
        range(source_id, 18, 23),
    );
    let end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 24, 27),
    );
    let first_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 27, 28),
    );
    let lemma = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "lemma",
        range(source_id, 29, 34),
    );
    let l = builder.add_token(SurfaceTokenKind::Identifier, "L", range(source_id, 35, 36));
    let second_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 36, 37),
    );
    let second_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 38, 44),
    );
    let second_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 44, 45),
    );

    let first_formula = thesis_formula_node(&mut builder, source_id, first_thesis, 11, 17);
    let proof_block = builder.add_node(
        SurfaceNodeKind::ProofBlock,
        range(source_id, 18, 27),
        vec![proof, end],
    );
    let theorem_item = builder.add_node(
        SurfaceNodeKind::TheoremItem,
        range(source_id, 0, 28),
        vec![
            theorem,
            t,
            first_colon,
            first_formula,
            proof_block,
            first_semicolon,
        ],
    );
    let second_formula = thesis_formula_node(&mut builder, source_id, second_thesis, 38, 44);
    let lemma_item = builder.add_node(
        SurfaceNodeKind::LemmaItem,
        range(source_id, 29, 45),
        vec![lemma, l, second_colon, second_formula, second_semicolon],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 45),
        vec![theorem_item, lemma_item],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 45),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 45),
        vec![
            theorem,
            t,
            first_colon,
            first_thesis,
            proof,
            end,
            first_semicolon,
            lemma,
            l,
            second_colon,
            second_thesis,
            second_semicolon,
            compilation_unit,
        ],
    );
    builder.finish(Some(root), None)
}

fn task23_definition_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let definition = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "definition",
        range(source_id, 0, 10),
    );
    let let_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, 11, 14),
    );
    let x_decl = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 15, 16));
    let be_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "be",
        range(source_id, 17, 19),
    );
    let set_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 20, 23),
    );
    let first_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 23, 24),
    );
    let attr = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "attr",
        range(source_id, 25, 29),
    );
    let label = builder.add_token(SurfaceTokenKind::Identifier, "A", range(source_id, 30, 31));
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 31, 32),
    );
    let subject = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 33, 34));
    let is_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "is",
        range(source_id, 35, 37),
    );
    let odd = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "odd",
        range(source_id, 38, 41),
    );
    let means = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "means",
        range(source_id, 42, 47),
    );
    let first_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 48, 54),
    );
    let if_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "if",
        range(source_id, 55, 57),
    );
    let contradiction = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "contradiction",
        range(source_id, 58, 71),
    );
    let otherwise = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "otherwise",
        range(source_id, 72, 81),
    );
    let second_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 82, 88),
    );
    let attr_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 88, 89),
    );
    let existence = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "existence",
        range(source_id, 90, 99),
    );
    let existence_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 99, 100),
    );
    let pred = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "pred",
        range(source_id, 101, 105),
    );
    let pred_label = builder.add_token(
        SurfaceTokenKind::Identifier,
        "P",
        range(source_id, 106, 107),
    );
    let pred_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 107, 108),
    );
    let left_locus = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 109, 110),
    );
    let pred_symbol = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "R",
        range(source_id, 111, 112),
    );
    let right_locus = builder.add_token(
        SurfaceTokenKind::Identifier,
        "y",
        range(source_id, 113, 114),
    );
    let pred_means = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "means",
        range(source_id, 115, 120),
    );
    let pred_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 121, 127),
    );
    let pred_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 127, 128),
    );
    let end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 129, 132),
    );
    let block_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 132, 133),
    );

    let type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 20, 23),
        vec![set_keyword],
    );
    let type_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 20, 23),
        vec![type_head],
    );
    let segment = builder.add_node(
        SurfaceNodeKind::QualifiedVariableSegment,
        range(source_id, 15, 23),
        vec![x_decl, be_keyword, type_expression],
    );
    let parameter = builder.add_node(
        SurfaceNodeKind::DefinitionParameter,
        range(source_id, 11, 24),
        vec![let_keyword, segment, first_semicolon],
    );
    let pattern = builder.add_node(
        SurfaceNodeKind::AttributePattern,
        range(source_id, 38, 41),
        vec![odd],
    );
    let first_formula = thesis_formula_node(&mut builder, source_id, first_thesis, 48, 54);
    let contradiction_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Contradiction),
        range(source_id, 58, 71),
        vec![contradiction],
    );
    let condition_formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 58, 71),
        vec![contradiction_constant],
    );
    let formula_case = builder.add_node(
        SurfaceNodeKind::FormulaCase,
        range(source_id, 48, 71),
        vec![first_formula, if_keyword, condition_formula],
    );
    let otherwise_formula = thesis_formula_node(&mut builder, source_id, second_thesis, 82, 88);
    let definiens = builder.add_node(
        SurfaceNodeKind::FormulaDefiniens,
        range(source_id, 48, 88),
        vec![formula_case, otherwise, otherwise_formula],
    );
    let attribute = builder.add_node(
        SurfaceNodeKind::AttributeDefinition,
        range(source_id, 25, 89),
        vec![
            attr,
            label,
            colon,
            subject,
            is_keyword,
            pattern,
            means,
            definiens,
            attr_semicolon,
        ],
    );
    let correctness = builder.add_node(
        SurfaceNodeKind::CorrectnessCondition,
        range(source_id, 90, 100),
        vec![existence, existence_semicolon],
    );
    let predicate_pattern = builder.add_node(
        SurfaceNodeKind::PredicatePattern,
        range(source_id, 109, 114),
        vec![left_locus, pred_symbol, right_locus],
    );
    let predicate_formula = thesis_formula_node(&mut builder, source_id, pred_thesis, 121, 127);
    let predicate_definiens = builder.add_node(
        SurfaceNodeKind::FormulaDefiniens,
        range(source_id, 121, 127),
        vec![predicate_formula],
    );
    let predicate = builder.add_node(
        SurfaceNodeKind::PredicateDefinition,
        range(source_id, 101, 128),
        vec![
            pred,
            pred_label,
            pred_colon,
            predicate_pattern,
            pred_means,
            predicate_definiens,
            pred_semicolon,
        ],
    );
    let definition_item = builder.add_node(
        SurfaceNodeKind::DefinitionBlockItem,
        range(source_id, 0, 133),
        vec![
            definition,
            parameter,
            attribute,
            correctness,
            predicate,
            end,
            block_semicolon,
        ],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 133),
        vec![definition_item],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 133),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 133),
        vec![
            definition,
            let_keyword,
            x_decl,
            be_keyword,
            set_keyword,
            first_semicolon,
            attr,
            label,
            colon,
            subject,
            is_keyword,
            odd,
            means,
            first_thesis,
            if_keyword,
            contradiction,
            otherwise,
            second_thesis,
            attr_semicolon,
            existence,
            existence_semicolon,
            pred,
            pred_label,
            pred_colon,
            left_locus,
            pred_symbol,
            right_locus,
            pred_means,
            pred_thesis,
            pred_semicolon,
            end,
            block_semicolon,
            compilation_unit,
        ],
    );
    builder.finish(Some(root), None)
}

fn task48_property_implementation_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let definition = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "definition",
        range(source_id, 0, 10),
    );
    let end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 11, 14),
    );
    let semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 14, 15),
    );
    let property = builder.add_node(
        SurfaceNodeKind::PropertyImplementation,
        range(source_id, 0, 15),
        vec![definition, end, semicolon],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 0, 15),
        vec![property],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 0, 15),
        vec![item_list],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 15),
        vec![definition, end, semicolon, compilation_unit],
    );
    builder.finish(Some(root), None)
}

fn task25_functor_definition_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let func = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "func",
        range(source_id, 0, 4),
    );
    let label = builder.add_token(SurfaceTokenKind::Identifier, "F", range(source_id, 5, 6));
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 6, 7),
    );
    let left_locus = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 8, 9));
    let functor_symbol =
        builder.add_token(SurfaceTokenKind::UserSymbol, "++", range(source_id, 10, 12));
    let right_locus =
        builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 13, 14));
    let arrow = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "->",
        range(source_id, 15, 17),
    );
    let set_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 18, 21),
    );
    let equals = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "equals",
        range(source_id, 22, 28),
    );
    let first_value =
        builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 29, 30));
    let if_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "if",
        range(source_id, 31, 33),
    );
    let thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 34, 40),
    );
    let otherwise = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "otherwise",
        range(source_id, 41, 50),
    );
    let fallback_value =
        builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 51, 52));
    let semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 52, 53),
    );

    let pattern = builder.add_node(
        SurfaceNodeKind::FunctorPattern,
        range(source_id, 8, 14),
        vec![left_locus, functor_symbol, right_locus],
    );
    let type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 18, 21),
        vec![set_keyword],
    );
    let return_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 18, 21),
        vec![type_head],
    );
    let first_term = term_expression_node(&mut builder, source_id, first_value, 29, 30);
    let condition = thesis_formula_node(&mut builder, source_id, thesis, 34, 40);
    let term_case = builder.add_node(
        SurfaceNodeKind::TermCase,
        range(source_id, 29, 40),
        vec![first_term, if_keyword, condition],
    );
    let fallback_term = term_expression_node(&mut builder, source_id, fallback_value, 51, 52);
    let term_definiens = builder.add_node(
        SurfaceNodeKind::TermDefiniens,
        range(source_id, 29, 52),
        vec![term_case, otherwise, fallback_term],
    );
    let functor = builder.add_node(
        SurfaceNodeKind::FunctorDefinition,
        range(source_id, 0, 53),
        vec![
            func,
            label,
            colon,
            pattern,
            arrow,
            return_type,
            equals,
            term_definiens,
            semicolon,
        ],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 53),
        vec![
            func,
            label,
            colon,
            left_locus,
            functor_symbol,
            right_locus,
            arrow,
            set_keyword,
            equals,
            first_value,
            if_keyword,
            thesis,
            otherwise,
            fallback_value,
            semicolon,
            functor,
        ],
    );
    builder.finish(Some(root), None)
}

fn task26_mode_definition_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let mode = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "mode",
        range(source_id, 0, 4),
    );
    let label = builder.add_token(SurfaceTokenKind::Identifier, "M", range(source_id, 5, 6));
    let colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 6, 7),
    );
    let mode_name = builder.add_token(SurfaceTokenKind::UserSymbol, "T", range(source_id, 8, 9));
    let of = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "of",
        range(source_id, 10, 12),
    );
    let parameter = builder.add_token(SurfaceTokenKind::Identifier, "X", range(source_id, 13, 14));
    let is = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "is",
        range(source_id, 15, 17),
    );
    let non = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "non",
        range(source_id, 18, 21),
    );
    let empty = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "empty",
        range(source_id, 22, 27),
    );
    let set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 28, 31),
    );
    let body_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 31, 32),
    );
    let sethood = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "sethood",
        range(source_id, 33, 40),
    );
    let by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 41, 43),
    );
    let reference_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 44, 47),
    );
    let property_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 47, 48),
    );

    let pattern = builder.add_node(
        SurfaceNodeKind::ModePattern,
        range(source_id, 8, 14),
        vec![mode_name, of, parameter],
    );
    let attribute_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 22, 27),
        vec![empty],
    );
    let attribute = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(source_id, 18, 27),
        vec![non, attribute_symbol],
    );
    let attribute_chain = builder.add_node(
        SurfaceNodeKind::AttributeChain,
        range(source_id, 18, 27),
        vec![attribute],
    );
    let type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 28, 31),
        vec![set],
    );
    let body_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 18, 31),
        vec![attribute_chain, type_head],
    );
    let reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 44, 47),
        vec![reference_name],
    );
    let reference_list = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 44, 47),
        vec![reference],
    );
    let justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 41, 47),
        vec![by, reference_list],
    );
    let property = builder.add_node(
        SurfaceNodeKind::ModeProperty,
        range(source_id, 33, 48),
        vec![sethood, justification, property_semicolon],
    );
    let mode_definition = builder.add_node(
        SurfaceNodeKind::ModeDefinition,
        range(source_id, 0, 48),
        vec![
            mode,
            label,
            colon,
            pattern,
            is,
            body_type,
            body_semicolon,
            property,
        ],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 48),
        vec![
            mode,
            label,
            colon,
            mode_name,
            of,
            parameter,
            is,
            non,
            empty,
            set,
            body_semicolon,
            sethood,
            by,
            reference_name,
            property_semicolon,
            mode_definition,
        ],
    );
    builder.finish(Some(root), None)
}

fn task27_redefinition_notation_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let attr_redefine = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "redefine",
        range(source_id, 0, 8),
    );
    let attr_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "attr",
        range(source_id, 9, 13),
    );
    let attr_label = builder.add_token(SurfaceTokenKind::Identifier, "A", range(source_id, 14, 15));
    let attr_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 15, 16),
    );
    let attr_subject =
        builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 17, 18));
    let attr_is = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "is",
        range(source_id, 19, 21),
    );
    let attr_name = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "empty",
        range(source_id, 22, 27),
    );
    let attr_means = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "means",
        range(source_id, 28, 33),
    );
    let attr_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 34, 40),
    );
    let attr_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 40, 41),
    );
    let attr_coherence = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "coherence",
        range(source_id, 42, 51),
    );
    let attr_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 52, 54),
    );
    let attr_reference_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 55, 58),
    );
    let attr_coherence_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 58, 59),
    );

    let pred_redefine = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "redefine",
        range(source_id, 60, 68),
    );
    let pred_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "pred",
        range(source_id, 69, 73),
    );
    let pred_redefinition_label =
        builder.add_token(SurfaceTokenKind::Identifier, "P", range(source_id, 74, 75));
    let pred_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 75, 76),
    );
    let pred_left = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 77, 78));
    let pred_symbol =
        builder.add_token(SurfaceTokenKind::UserSymbol, "R", range(source_id, 79, 80));
    let pred_means = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "means",
        range(source_id, 81, 86),
    );
    let pred_thesis = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 87, 93),
    );
    let pred_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 93, 94),
    );
    let pred_coherence = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "coherence",
        range(source_id, 95, 104),
    );
    let pred_with = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "with",
        range(source_id, 105, 109),
    );
    let pred_coherence_label = builder.add_token(
        SurfaceTokenKind::Identifier,
        "C",
        range(source_id, 110, 111),
    );
    let pred_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 112, 114),
    );
    let pred_reference_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 115, 118),
    );
    let pred_coherence_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 118, 119),
    );

    let func_redefine = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "redefine",
        range(source_id, 120, 128),
    );
    let func_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "func",
        range(source_id, 129, 133),
    );
    let func_label = builder.add_token(
        SurfaceTokenKind::Identifier,
        "F",
        range(source_id, 134, 135),
    );
    let func_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 135, 136),
    );
    let func_pattern_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 137, 138),
    );
    let func_arrow = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "->",
        range(source_id, 139, 141),
    );
    let set_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 142, 145),
    );
    let func_equals = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "equals",
        range(source_id, 146, 152),
    );
    let func_body_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 153, 154),
    );
    let func_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 154, 155),
    );
    let func_coherence = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "coherence",
        range(source_id, 156, 165),
    );
    let proof = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "proof",
        range(source_id, 166, 171),
    );
    let end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 172, 175),
    );
    let func_coherence_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 175, 176),
    );

    let synonym = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "synonym",
        range(source_id, 180, 187),
    );
    let alternate = builder.add_token(
        SurfaceTokenKind::Identifier,
        "infinite",
        range(source_id, 188, 196),
    );
    let for_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "for",
        range(source_id, 197, 200),
    );
    let original = builder.add_token(
        SurfaceTokenKind::Identifier,
        "finite",
        range(source_id, 201, 207),
    );
    let alias_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 207, 208),
    );

    let attr_pattern = builder.add_node(
        SurfaceNodeKind::AttributePattern,
        range(source_id, 22, 27),
        vec![attr_name],
    );
    let attr_formula = thesis_formula_node(&mut builder, source_id, attr_thesis, 34, 40);
    let attr_definiens = builder.add_node(
        SurfaceNodeKind::FormulaDefiniens,
        range(source_id, 34, 40),
        vec![attr_formula],
    );
    let attr_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 55, 58),
        vec![attr_reference_name],
    );
    let attr_references = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 55, 58),
        vec![attr_reference],
    );
    let attr_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 52, 58),
        vec![attr_by, attr_references],
    );
    let attr_coherence_node = builder.add_node(
        SurfaceNodeKind::CoherenceCondition,
        range(source_id, 42, 59),
        vec![attr_coherence, attr_justification, attr_coherence_semicolon],
    );
    let attribute_redefinition = builder.add_node(
        SurfaceNodeKind::AttributeRedefinition,
        range(source_id, 0, 59),
        vec![
            attr_redefine,
            attr_keyword,
            attr_label,
            attr_colon,
            attr_subject,
            attr_is,
            attr_pattern,
            attr_means,
            attr_definiens,
            attr_semicolon,
            attr_coherence_node,
        ],
    );

    let predicate_pattern = builder.add_node(
        SurfaceNodeKind::PredicatePattern,
        range(source_id, 77, 80),
        vec![pred_left, pred_symbol],
    );
    let pred_formula = thesis_formula_node(&mut builder, source_id, pred_thesis, 87, 93);
    let pred_definiens = builder.add_node(
        SurfaceNodeKind::FormulaDefiniens,
        range(source_id, 87, 93),
        vec![pred_formula],
    );
    let pred_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 115, 118),
        vec![pred_reference_name],
    );
    let pred_references = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 115, 118),
        vec![pred_reference],
    );
    let pred_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 112, 118),
        vec![pred_by, pred_references],
    );
    let pred_coherence_node = builder.add_node(
        SurfaceNodeKind::CoherenceCondition,
        range(source_id, 95, 119),
        vec![
            pred_coherence,
            pred_with,
            pred_coherence_label,
            pred_justification,
            pred_coherence_semicolon,
        ],
    );
    let predicate_redefinition = builder.add_node(
        SurfaceNodeKind::PredicateRedefinition,
        range(source_id, 60, 119),
        vec![
            pred_redefine,
            pred_keyword,
            pred_redefinition_label,
            pred_colon,
            predicate_pattern,
            pred_means,
            pred_definiens,
            pred_semicolon,
            pred_coherence_node,
        ],
    );

    let functor_pattern = builder.add_node(
        SurfaceNodeKind::FunctorPattern,
        range(source_id, 137, 138),
        vec![func_pattern_name],
    );
    let type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 142, 145),
        vec![set_keyword],
    );
    let return_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 142, 145),
        vec![type_head],
    );
    let term_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 153, 154),
        vec![func_body_name],
    );
    let term = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 153, 154),
        vec![term_reference],
    );
    let term_definiens = builder.add_node(
        SurfaceNodeKind::TermDefiniens,
        range(source_id, 153, 154),
        vec![term],
    );
    let proof_block = builder.add_node(
        SurfaceNodeKind::ProofBlock,
        range(source_id, 166, 175),
        vec![proof, end],
    );
    let func_coherence_node = builder.add_node(
        SurfaceNodeKind::CoherenceCondition,
        range(source_id, 156, 176),
        vec![func_coherence, proof_block, func_coherence_semicolon],
    );
    let functor_redefinition = builder.add_node(
        SurfaceNodeKind::FunctorRedefinition,
        range(source_id, 120, 176),
        vec![
            func_redefine,
            func_keyword,
            func_label,
            func_colon,
            functor_pattern,
            func_arrow,
            return_type,
            func_equals,
            term_definiens,
            func_semicolon,
            func_coherence_node,
        ],
    );

    let alternate_pattern = builder.add_node(
        SurfaceNodeKind::NotationPattern,
        range(source_id, 188, 196),
        vec![alternate],
    );
    let original_pattern = builder.add_node(
        SurfaceNodeKind::NotationPattern,
        range(source_id, 201, 207),
        vec![original],
    );
    let alias = builder.add_node(
        SurfaceNodeKind::NotationAlias,
        range(source_id, 180, 208),
        vec![
            synonym,
            alternate_pattern,
            for_keyword,
            original_pattern,
            alias_semicolon,
        ],
    );

    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 208),
        vec![
            attr_redefine,
            attr_keyword,
            attr_label,
            attr_colon,
            attr_subject,
            attr_is,
            attr_name,
            attr_means,
            attr_thesis,
            attr_semicolon,
            attr_coherence,
            attr_by,
            attr_reference_name,
            attr_coherence_semicolon,
            pred_redefine,
            pred_keyword,
            pred_redefinition_label,
            pred_colon,
            pred_left,
            pred_symbol,
            pred_means,
            pred_thesis,
            pred_semicolon,
            pred_coherence,
            pred_with,
            pred_coherence_label,
            pred_by,
            pred_reference_name,
            pred_coherence_semicolon,
            func_redefine,
            func_keyword,
            func_label,
            func_colon,
            func_pattern_name,
            func_arrow,
            set_keyword,
            func_equals,
            func_body_name,
            func_semicolon,
            func_coherence,
            proof,
            end,
            func_coherence_semicolon,
            synonym,
            alternate,
            for_keyword,
            original,
            alias_semicolon,
            attribute_redefinition,
            predicate_redefinition,
            functor_redefinition,
            alias,
        ],
    );
    builder.finish(Some(root), None)
}

fn task28_property_clause_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let property_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "symmetry",
        range(source_id, 0, 8),
    );
    let by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 9, 11),
    );
    let reference_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 12, 15),
    );
    let semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 15, 16),
    );

    let reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 12, 15),
        vec![reference_name],
    );
    let references = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 12, 15),
        vec![reference],
    );
    let justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 9, 15),
        vec![by, references],
    );
    let property_clause = builder.add_node(
        SurfaceNodeKind::PropertyClause,
        range(source_id, 0, 16),
        vec![property_keyword, justification, semicolon],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 16),
        vec![
            property_keyword,
            by,
            reference_name,
            semicolon,
            property_clause,
        ],
    );
    builder.finish(Some(root), None)
}

fn task29_structure_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let struct_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "struct",
        range(source_id, 0, 6),
    );
    let struct_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Carrier",
        range(source_id, 7, 14),
    );
    let struct_where = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "where",
        range(source_id, 15, 20),
    );
    let field_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "field",
        range(source_id, 21, 26),
    );
    let field_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "carrier",
        range(source_id, 27, 34),
    );
    let field_arrow = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "->",
        range(source_id, 35, 37),
    );
    let field_set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 38, 41),
    );
    let field_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 41, 42),
    );
    let property_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "property",
        range(source_id, 43, 51),
    );
    let property_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "size",
        range(source_id, 52, 56),
    );
    let property_arrow = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "->",
        range(source_id, 57, 59),
    );
    let property_set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 60, 63),
    );
    let property_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 63, 64),
    );
    let struct_end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 65, 68),
    );
    let struct_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 68, 69),
    );

    let inherit_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "inherit",
        range(source_id, 70, 77),
    );
    let child = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Child",
        range(source_id, 78, 83),
    );
    let extends = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "extends",
        range(source_id, 84, 91),
    );
    let parent = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Parent",
        range(source_id, 92, 98),
    );
    let inherit_where = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "where",
        range(source_id, 99, 104),
    );
    let redef_field_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "field",
        range(source_id, 105, 110),
    );
    let redef_field_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "carrier",
        range(source_id, 111, 118),
    );
    let field_from = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "from",
        range(source_id, 119, 123),
    );
    let it = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "it",
        range(source_id, 124, 126),
    );
    let redef_field_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 126, 127),
    );
    let redef_property_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "property",
        range(source_id, 128, 136),
    );
    let redef_property_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "size",
        range(source_id, 137, 141),
    );
    let redef_property_arrow = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "->",
        range(source_id, 142, 144),
    );
    let redef_property_set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 145, 148),
    );
    let property_from = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "from",
        range(source_id, 149, 153),
    );
    let source_property = builder.add_token(
        SurfaceTokenKind::Identifier,
        "size",
        range(source_id, 154, 158),
    );
    let redef_property_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 158, 159),
    );
    let coherence_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "coherence",
        range(source_id, 160, 169),
    );
    let by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 170, 172),
    );
    let ref_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 173, 176),
    );
    let coherence_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 176, 177),
    );
    let inherit_end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 178, 181),
    );
    let inherit_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 181, 182),
    );

    let structure_pattern = builder.add_node(
        SurfaceNodeKind::StructurePattern,
        range(source_id, 7, 14),
        vec![struct_name],
    );
    let field_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 38, 41),
        vec![field_set],
    );
    let field_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 38, 41),
        vec![field_type_head],
    );
    let structure_field = builder.add_node(
        SurfaceNodeKind::StructureField,
        range(source_id, 21, 42),
        vec![
            field_kw,
            field_name,
            field_arrow,
            field_type,
            field_semicolon,
        ],
    );
    let property_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 60, 63),
        vec![property_set],
    );
    let property_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 60, 63),
        vec![property_type_head],
    );
    let structure_property = builder.add_node(
        SurfaceNodeKind::StructureProperty,
        range(source_id, 43, 64),
        vec![
            property_kw,
            property_name,
            property_arrow,
            property_type,
            property_semicolon,
        ],
    );
    let structure_definition = builder.add_node(
        SurfaceNodeKind::StructureDefinition,
        range(source_id, 0, 69),
        vec![
            struct_kw,
            structure_pattern,
            struct_where,
            structure_field,
            structure_property,
            struct_end,
            struct_semicolon,
        ],
    );

    let child_target = builder.add_node(
        SurfaceNodeKind::InheritanceTarget,
        range(source_id, 78, 83),
        vec![child],
    );
    let parent_target = builder.add_node(
        SurfaceNodeKind::InheritanceTarget,
        range(source_id, 92, 98),
        vec![parent],
    );
    let field_redefinition = builder.add_node(
        SurfaceNodeKind::FieldRedefinition,
        range(source_id, 105, 127),
        vec![
            redef_field_kw,
            redef_field_name,
            field_from,
            it,
            redef_field_semicolon,
        ],
    );
    let redef_property_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 145, 148),
        vec![redef_property_set],
    );
    let redef_property_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 145, 148),
        vec![redef_property_type_head],
    );
    let property_redefinition = builder.add_node(
        SurfaceNodeKind::PropertyRedefinition,
        range(source_id, 128, 159),
        vec![
            redef_property_kw,
            redef_property_name,
            redef_property_arrow,
            redef_property_type,
            property_from,
            source_property,
            redef_property_semicolon,
        ],
    );
    let reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 173, 176),
        vec![ref_name],
    );
    let references = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 173, 176),
        vec![reference],
    );
    let justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 170, 176),
        vec![by, references],
    );
    let coherence = builder.add_node(
        SurfaceNodeKind::CoherenceCondition,
        range(source_id, 160, 177),
        vec![coherence_kw, justification, coherence_semicolon],
    );
    let inheritance_definition = builder.add_node(
        SurfaceNodeKind::InheritanceDefinition,
        range(source_id, 70, 182),
        vec![
            inherit_kw,
            child_target,
            extends,
            parent_target,
            inherit_where,
            field_redefinition,
            property_redefinition,
            coherence,
            inherit_end,
            inherit_semicolon,
        ],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 182),
        vec![
            struct_kw,
            struct_name,
            struct_where,
            field_kw,
            field_name,
            field_arrow,
            field_set,
            field_semicolon,
            property_kw,
            property_name,
            property_arrow,
            property_set,
            property_semicolon,
            struct_end,
            struct_semicolon,
            inherit_kw,
            child,
            extends,
            parent,
            inherit_where,
            redef_field_kw,
            redef_field_name,
            field_from,
            it,
            redef_field_semicolon,
            redef_property_kw,
            redef_property_name,
            redef_property_arrow,
            redef_property_set,
            property_from,
            source_property,
            redef_property_semicolon,
            coherence_kw,
            by,
            ref_name,
            coherence_semicolon,
            inherit_end,
            inherit_semicolon,
            structure_definition,
            inheritance_definition,
        ],
    );
    builder.finish(Some(root), None)
}

fn task30_registration_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let registration_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "registration",
        range(source_id, 0, 12),
    );
    let let_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, 13, 16),
    );
    let variable = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 17, 18));
    let be_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "be",
        range(source_id, 19, 21),
    );
    let parameter_set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 22, 25),
    );
    let parameter_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 25, 26),
    );
    let cluster_exists = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "cluster",
        range(source_id, 27, 34),
    );
    let exists_label =
        builder.add_token(SurfaceTokenKind::Identifier, "E", range(source_id, 35, 36));
    let exists_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 36, 37),
    );
    let non_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "non",
        range(source_id, 38, 41),
    );
    let empty_attr = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "empty",
        range(source_id, 42, 47),
    );
    let exists_set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 48, 51),
    );
    let exists_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 51, 52),
    );
    let existence_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "existence",
        range(source_id, 53, 62),
    );
    let exists_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 63, 65),
    );
    let exists_ref = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 66, 69),
    );
    let existence_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 69, 70),
    );
    let cluster_conditional = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "cluster",
        range(source_id, 71, 78),
    );
    let conditional_label =
        builder.add_token(SurfaceTokenKind::Identifier, "C", range(source_id, 79, 80));
    let conditional_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 80, 81),
    );
    let antecedent = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "empty",
        range(source_id, 82, 87),
    );
    let conditional_arrow = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "->",
        range(source_id, 88, 90),
    );
    let consequent = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "finite",
        range(source_id, 91, 97),
    );
    let conditional_for = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "for",
        range(source_id, 98, 101),
    );
    let conditional_set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 102, 105),
    );
    let conditional_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 105, 106),
    );
    let coherence_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "coherence",
        range(source_id, 107, 116),
    );
    let coherence_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 117, 119),
    );
    let coherence_ref = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 120, 123),
    );
    let coherence_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 123, 124),
    );
    let cluster_functorial = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "cluster",
        range(source_id, 125, 132),
    );
    let functorial_label = builder.add_token(
        SurfaceTokenKind::Identifier,
        "F",
        range(source_id, 133, 134),
    );
    let functorial_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 134, 135),
    );
    let functor = builder.add_token(
        SurfaceTokenKind::Identifier,
        "f",
        range(source_id, 136, 137),
    );
    let open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 137, 138),
    );
    let arg = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 138, 139),
    );
    let close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 139, 140),
    );
    let functorial_arrow = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "->",
        range(source_id, 141, 143),
    );
    let functorial_attr = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "finite",
        range(source_id, 144, 150),
    );
    let functorial_for = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "for",
        range(source_id, 151, 154),
    );
    let functorial_set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 155, 158),
    );
    let functorial_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 158, 159),
    );
    let functorial_coherence = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "coherence",
        range(source_id, 160, 169),
    );
    let functorial_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 170, 172),
    );
    let functorial_ref = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 173, 176),
    );
    let functorial_coherence_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 176, 177),
    );
    let reduce_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "reduce",
        range(source_id, 178, 184),
    );
    let reduce_label = builder.add_token(
        SurfaceTokenKind::Identifier,
        "R",
        range(source_id, 185, 186),
    );
    let reduce_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 186, 187),
    );
    let reduce_left = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 188, 189),
    );
    let to_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "to",
        range(source_id, 190, 192),
    );
    let reduce_right = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 193, 194),
    );
    let reduce_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 194, 195),
    );
    let reducibility_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "reducibility",
        range(source_id, 196, 208),
    );
    let reducibility_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 209, 211),
    );
    let reducibility_ref = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 212, 215),
    );
    let reducibility_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 215, 216),
    );
    let registration_end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 217, 220),
    );
    let registration_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 220, 221),
    );

    let parameter_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 22, 25),
        vec![parameter_set],
    );
    let parameter_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 22, 25),
        vec![parameter_type_head],
    );
    let parameter_segment = builder.add_node(
        SurfaceNodeKind::QualifiedVariableSegment,
        range(source_id, 17, 25),
        vec![variable, be_kw, parameter_type],
    );
    let registration_parameter = builder.add_node(
        SurfaceNodeKind::RegistrationParameter,
        range(source_id, 13, 26),
        vec![let_kw, parameter_segment, parameter_semicolon],
    );
    let empty_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 42, 47),
        vec![empty_attr],
    );
    let empty_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 42, 47),
        vec![empty_segment],
    );
    let exists_attribute = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(source_id, 38, 47),
        vec![non_kw, empty_symbol],
    );
    let exists_attribute_chain = builder.add_node(
        SurfaceNodeKind::AttributeChain,
        range(source_id, 38, 47),
        vec![exists_attribute],
    );
    let exists_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 48, 51),
        vec![exists_set],
    );
    let exists_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 38, 51),
        vec![exists_attribute_chain, exists_type_head],
    );
    let exists_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 66, 69),
        vec![exists_ref],
    );
    let exists_references = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 66, 69),
        vec![exists_reference],
    );
    let exists_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 63, 69),
        vec![exists_by, exists_references],
    );
    let existential = builder.add_node(
        SurfaceNodeKind::ExistentialRegistration,
        range(source_id, 27, 70),
        vec![
            cluster_exists,
            exists_label,
            exists_colon,
            exists_type,
            exists_semicolon,
            existence_kw,
            exists_justification,
            existence_semicolon,
        ],
    );

    let antecedent_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 82, 87),
        vec![antecedent],
    );
    let antecedent_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 82, 87),
        vec![antecedent_segment],
    );
    let antecedent_node = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(source_id, 82, 87),
        vec![antecedent_symbol],
    );
    let consequent_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 91, 97),
        vec![consequent],
    );
    let consequent_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 91, 97),
        vec![consequent_segment],
    );
    let consequent_node = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(source_id, 91, 97),
        vec![consequent_symbol],
    );
    let conditional_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 102, 105),
        vec![conditional_set],
    );
    let conditional_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 102, 105),
        vec![conditional_type_head],
    );
    let coherence_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 120, 123),
        vec![coherence_ref],
    );
    let coherence_references = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 120, 123),
        vec![coherence_reference],
    );
    let coherence_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 117, 123),
        vec![coherence_by, coherence_references],
    );
    let conditional = builder.add_node(
        SurfaceNodeKind::ConditionalRegistration,
        range(source_id, 71, 124),
        vec![
            cluster_conditional,
            conditional_label,
            conditional_colon,
            antecedent_node,
            conditional_arrow,
            consequent_node,
            conditional_for,
            conditional_type,
            conditional_semicolon,
            coherence_kw,
            coherence_justification,
            coherence_semicolon,
        ],
    );

    let callee = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 136, 137),
        vec![functor],
    );
    let argument_ref = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 138, 139),
        vec![arg],
    );
    let argument = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 138, 139),
        vec![argument_ref],
    );
    let application = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        range(source_id, 136, 140),
        vec![callee, open, argument, close],
    );
    let functorial_payload = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 136, 140),
        vec![application],
    );
    let functorial_attr_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 144, 150),
        vec![functorial_attr],
    );
    let functorial_attr_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 144, 150),
        vec![functorial_attr_segment],
    );
    let functorial_attr_node = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(source_id, 144, 150),
        vec![functorial_attr_symbol],
    );
    let functorial_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 155, 158),
        vec![functorial_set],
    );
    let functorial_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 155, 158),
        vec![functorial_type_head],
    );
    let functorial_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 173, 176),
        vec![functorial_ref],
    );
    let functorial_references = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 173, 176),
        vec![functorial_reference],
    );
    let functorial_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 170, 176),
        vec![functorial_by, functorial_references],
    );
    let functorial = builder.add_node(
        SurfaceNodeKind::FunctorialRegistration,
        range(source_id, 125, 177),
        vec![
            cluster_functorial,
            functorial_label,
            functorial_colon,
            functorial_payload,
            functorial_arrow,
            functorial_attr_node,
            functorial_for,
            functorial_type,
            functorial_semicolon,
            functorial_coherence,
            functorial_justification,
            functorial_coherence_semicolon,
        ],
    );

    let reduce_left_ref = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 188, 189),
        vec![reduce_left],
    );
    let reduce_left_term = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 188, 189),
        vec![reduce_left_ref],
    );
    let reduce_right_ref = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 193, 194),
        vec![reduce_right],
    );
    let reduce_right_term = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 193, 194),
        vec![reduce_right_ref],
    );
    let reducibility_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 212, 215),
        vec![reducibility_ref],
    );
    let reducibility_references = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 212, 215),
        vec![reducibility_reference],
    );
    let reducibility_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 209, 215),
        vec![reducibility_by, reducibility_references],
    );
    let reduction = builder.add_node(
        SurfaceNodeKind::ReductionRegistration,
        range(source_id, 178, 216),
        vec![
            reduce_kw,
            reduce_label,
            reduce_colon,
            reduce_left_term,
            to_kw,
            reduce_right_term,
            reduce_semicolon,
            reducibility_kw,
            reducibility_justification,
            reducibility_semicolon,
        ],
    );
    let registration_block = builder.add_node(
        SurfaceNodeKind::RegistrationBlockItem,
        range(source_id, 0, 221),
        vec![
            registration_kw,
            registration_parameter,
            existential,
            conditional,
            functorial,
            reduction,
            registration_end,
            registration_semicolon,
        ],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 221),
        vec![
            registration_kw,
            let_kw,
            variable,
            be_kw,
            parameter_set,
            parameter_semicolon,
            cluster_exists,
            exists_label,
            exists_colon,
            non_kw,
            empty_attr,
            exists_set,
            exists_semicolon,
            existence_kw,
            exists_by,
            exists_ref,
            existence_semicolon,
            cluster_conditional,
            conditional_label,
            conditional_colon,
            antecedent,
            conditional_arrow,
            consequent,
            conditional_for,
            conditional_set,
            conditional_semicolon,
            coherence_kw,
            coherence_by,
            coherence_ref,
            coherence_semicolon,
            cluster_functorial,
            functorial_label,
            functorial_colon,
            functor,
            open,
            arg,
            close,
            functorial_arrow,
            functorial_attr,
            functorial_for,
            functorial_set,
            functorial_semicolon,
            functorial_coherence,
            functorial_by,
            functorial_ref,
            functorial_coherence_semicolon,
            reduce_kw,
            reduce_label,
            reduce_colon,
            reduce_left,
            to_kw,
            reduce_right,
            reduce_semicolon,
            reducibility_kw,
            reducibility_by,
            reducibility_ref,
            reducibility_semicolon,
            registration_end,
            registration_semicolon,
            registration_block,
        ],
    );
    builder.finish(Some(root), None)
}

fn task31_template_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let let_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, 0, 3),
    );
    let parameter_name =
        builder.add_token(SurfaceTokenKind::Identifier, "T", range(source_id, 4, 5));
    let be_kw = builder.add_token(SurfaceTokenKind::ReservedWord, "be", range(source_id, 6, 8));
    let type_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "type",
        range(source_id, 9, 13),
    );
    let parameter_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 13, 14),
    );
    let loci_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "[",
        range(source_id, 15, 16),
    );
    let locus_name = builder.add_token(SurfaceTokenKind::Identifier, "L", range(source_id, 16, 17));
    let loci_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "]",
        range(source_id, 17, 18),
    );
    let reference_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Ref",
        range(source_id, 19, 22),
    );
    let arguments_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "[",
        range(source_id, 22, 23),
    );
    let argument_name =
        builder.add_token(SurfaceTokenKind::UserSymbol, "T", range(source_id, 23, 24));
    let arguments_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "]",
        range(source_id, 24, 25),
    );

    let template_parameter = builder.add_node(
        SurfaceNodeKind::TemplateParameter,
        range(source_id, 0, 14),
        vec![let_kw, parameter_name, be_kw, type_kw, parameter_semicolon],
    );
    let template_locus = builder.add_node(
        SurfaceNodeKind::TemplateLocus,
        range(source_id, 16, 17),
        vec![locus_name],
    );
    let template_loci = builder.add_node(
        SurfaceNodeKind::TemplateLoci,
        range(source_id, 15, 18),
        vec![loci_open, template_locus, loci_close],
    );
    let argument_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 23, 24),
        vec![argument_name],
    );
    let argument_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 23, 24),
        vec![argument_type_head],
    );
    let template_argument = builder.add_node(
        SurfaceNodeKind::TemplateArgument,
        range(source_id, 23, 24),
        vec![argument_type],
    );
    let template_arguments = builder.add_node(
        SurfaceNodeKind::TemplateArguments,
        range(source_id, 22, 25),
        vec![arguments_open, template_argument, arguments_close],
    );
    let reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 19, 25),
        vec![reference_name, template_arguments],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 25),
        vec![
            let_kw,
            parameter_name,
            be_kw,
            type_kw,
            parameter_semicolon,
            loci_open,
            locus_name,
            loci_close,
            reference_name,
            arguments_open,
            argument_name,
            arguments_close,
            template_parameter,
            template_loci,
            reference,
        ],
    );
    builder.finish(Some(root), None)
}

fn task32_algorithm_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let algorithm_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "algorithm",
        range(source_id, 0, 9),
    );
    let algorithm_name =
        builder.add_token(SurfaceTokenKind::Identifier, "f", range(source_id, 10, 11));
    let loci_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "[",
        range(source_id, 11, 12),
    );
    let locus_name = builder.add_token(SurfaceTokenKind::Identifier, "T", range(source_id, 12, 13));
    let loci_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "]",
        range(source_id, 13, 14),
    );
    let params_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 14, 15),
    );
    let param_name = builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 15, 16));
    let params_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 16, 17),
    );
    let do_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "do",
        range(source_id, 18, 20),
    );
    let var_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "var",
        range(source_id, 21, 24),
    );
    let binding_name =
        builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 25, 26));
    let binding_assign = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":=",
        range(source_id, 27, 29),
    );
    let binding_term_token =
        builder.add_token(SurfaceTokenKind::Identifier, "x", range(source_id, 30, 31));
    let decl_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 31, 32),
    );
    let assign_target =
        builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 33, 34));
    let assign_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":=",
        range(source_id, 35, 37),
    );
    let assign_term_token =
        builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 38, 39));
    let assign_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 39, 40),
    );
    let snapshot_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "snapshot",
        range(source_id, 41, 49),
    );
    let snapshot_name =
        builder.add_token(SurfaceTokenKind::Identifier, "S", range(source_id, 50, 51));
    let snapshot_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 51, 52),
    );
    let return_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "return",
        range(source_id, 53, 59),
    );
    let return_term_token =
        builder.add_token(SurfaceTokenKind::Identifier, "y", range(source_id, 60, 61));
    let return_by = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "by",
        range(source_id, 62, 64),
    );
    let return_ref_token =
        builder.add_token(SurfaceTokenKind::Identifier, "A", range(source_id, 65, 66));
    let return_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 66, 67),
    );
    let end_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 68, 71),
    );
    let algorithm_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 71, 72),
    );
    let claim_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "claim",
        range(source_id, 73, 78),
    );
    let claim_name = builder.add_token(SurfaceTokenKind::Identifier, "f", range(source_id, 79, 80));
    let claim_do = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "do",
        range(source_id, 81, 83),
    );
    let theorem_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 84, 91),
    );
    let theorem_label =
        builder.add_token(SurfaceTokenKind::Identifier, "C", range(source_id, 92, 93));
    let theorem_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 93, 94),
    );
    let thesis_kw = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "thesis",
        range(source_id, 95, 101),
    );
    let theorem_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 101, 102),
    );
    let claim_end = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "end",
        range(source_id, 103, 106),
    );
    let claim_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 106, 107),
    );

    let template_locus = builder.add_node(
        SurfaceNodeKind::TemplateLocus,
        range(source_id, 12, 13),
        vec![locus_name],
    );
    let template_loci = builder.add_node(
        SurfaceNodeKind::TemplateLoci,
        range(source_id, 11, 14),
        vec![loci_open, template_locus, loci_close],
    );
    let parameters = builder.add_node(
        SurfaceNodeKind::AlgorithmParameters,
        range(source_id, 14, 17),
        vec![params_open, param_name, params_close],
    );
    let binding_term = term_expression_node(&mut builder, source_id, binding_term_token, 30, 31);
    let binding = builder.add_node(
        SurfaceNodeKind::VariableBinding,
        range(source_id, 25, 31),
        vec![binding_name, binding_assign, binding_term],
    );
    let declaration = builder.add_node(
        SurfaceNodeKind::VariableDeclaration,
        range(source_id, 21, 32),
        vec![var_kw, binding, decl_semicolon],
    );
    let lvalue = builder.add_node(
        SurfaceNodeKind::Lvalue,
        range(source_id, 33, 34),
        vec![assign_target],
    );
    let assign_term = term_expression_node(&mut builder, source_id, assign_term_token, 38, 39);
    let assignment = builder.add_node(
        SurfaceNodeKind::AssignmentStatement,
        range(source_id, 33, 40),
        vec![lvalue, assign_colon, assign_term, assign_semicolon],
    );
    let snapshot = builder.add_node(
        SurfaceNodeKind::SnapshotStatement,
        range(source_id, 41, 52),
        vec![snapshot_kw, snapshot_name, snapshot_semicolon],
    );
    let return_term = term_expression_node(&mut builder, source_id, return_term_token, 60, 61);
    let return_reference = builder.add_node(
        SurfaceNodeKind::Reference,
        range(source_id, 65, 66),
        vec![return_ref_token],
    );
    let return_references = builder.add_node(
        SurfaceNodeKind::ReferenceList,
        range(source_id, 65, 66),
        vec![return_reference],
    );
    let return_justification = builder.add_node(
        SurfaceNodeKind::JustificationClause,
        range(source_id, 62, 66),
        vec![return_by, return_references],
    );
    let return_statement = builder.add_node(
        SurfaceNodeKind::ReturnStatement,
        range(source_id, 53, 67),
        vec![
            return_kw,
            return_term,
            return_justification,
            return_semicolon,
        ],
    );
    let statement_list = builder.add_node(
        SurfaceNodeKind::AlgorithmStatementList,
        range(source_id, 21, 67),
        vec![declaration, assignment, snapshot, return_statement],
    );
    let body = builder.add_node(
        SurfaceNodeKind::AlgorithmBody,
        range(source_id, 18, 71),
        vec![do_kw, statement_list, end_kw],
    );
    let algorithm = builder.add_node(
        SurfaceNodeKind::AlgorithmDefinition,
        range(source_id, 0, 72),
        vec![
            algorithm_kw,
            algorithm_name,
            template_loci,
            parameters,
            body,
            algorithm_semicolon,
        ],
    );
    let theorem_proposition = thesis_proposition_node(&mut builder, source_id, thesis_kw, 95, 101);
    let theorem = builder.add_node(
        SurfaceNodeKind::TheoremItem,
        range(source_id, 84, 102),
        vec![
            theorem_kw,
            theorem_label,
            theorem_colon,
            theorem_proposition,
            theorem_semicolon,
        ],
    );
    let claim = builder.add_node(
        SurfaceNodeKind::ClaimBlockItem,
        range(source_id, 73, 107),
        vec![
            claim_kw,
            claim_name,
            claim_do,
            theorem,
            claim_end,
            claim_semicolon,
        ],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 107),
        vec![algorithm, claim],
    );
    builder.finish(Some(root), None)
}

fn task33_algorithm_control_flow_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);

    macro_rules! token {
        ($kind:expr, $text:expr, $start:expr, $end:expr) => {
            builder.add_token($kind, $text, range(source_id, $start, $end))
        };
    }

    let if_kw = token!(SurfaceTokenKind::ReservedWord, "if", 0, 2);
    let if_cond_token = token!(SurfaceTokenKind::ReservedWord, "thesis", 3, 9);
    let if_do = token!(SurfaceTokenKind::ReservedWord, "do", 10, 12);
    let break_kw = token!(SurfaceTokenKind::ReservedWord, "break", 13, 18);
    let break_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 18, 19);
    let else_kw = token!(SurfaceTokenKind::ReservedWord, "else", 20, 24);
    let continue_kw = token!(SurfaceTokenKind::ReservedWord, "continue", 25, 33);
    let continue_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 33, 34);
    let if_end = token!(SurfaceTokenKind::ReservedWord, "end", 35, 38);
    let if_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 38, 39);

    let if_condition = thesis_formula_node(&mut builder, source_id, if_cond_token, 3, 9);
    let break_statement = builder.add_node(
        SurfaceNodeKind::BreakStatement,
        range(source_id, 13, 19),
        vec![break_kw, break_semicolon],
    );
    let then_list = builder.add_node(
        SurfaceNodeKind::AlgorithmStatementList,
        range(source_id, 13, 19),
        vec![break_statement],
    );
    let continue_statement = builder.add_node(
        SurfaceNodeKind::ContinueStatement,
        range(source_id, 25, 34),
        vec![continue_kw, continue_semicolon],
    );
    let else_list = builder.add_node(
        SurfaceNodeKind::AlgorithmStatementList,
        range(source_id, 25, 34),
        vec![continue_statement],
    );
    let if_statement = builder.add_node(
        SurfaceNodeKind::IfStatement,
        range(source_id, 0, 39),
        vec![
            if_kw,
            if_condition,
            if_do,
            then_list,
            else_kw,
            else_list,
            if_end,
            if_semicolon,
        ],
    );

    let while_kw = token!(SurfaceTokenKind::ReservedWord, "while", 40, 45);
    let while_cond_token = token!(SurfaceTokenKind::ReservedWord, "thesis", 46, 52);
    let while_do = token!(SurfaceTokenKind::ReservedWord, "do", 53, 55);
    let while_end = token!(SurfaceTokenKind::ReservedWord, "end", 56, 59);
    let while_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 59, 60);
    let while_condition = thesis_formula_node(&mut builder, source_id, while_cond_token, 46, 52);
    let while_list = builder.add_node(
        SurfaceNodeKind::AlgorithmStatementList,
        range(source_id, 56, 56),
        Vec::new(),
    );
    let while_statement = builder.add_node(
        SurfaceNodeKind::WhileStatement,
        range(source_id, 40, 60),
        vec![
            while_kw,
            while_condition,
            while_do,
            while_list,
            while_end,
            while_semicolon,
        ],
    );

    let range_for_kw = token!(SurfaceTokenKind::ReservedWord, "for", 61, 64);
    let range_var = token!(SurfaceTokenKind::Identifier, "i", 65, 66);
    let range_eq = token!(SurfaceTokenKind::ReservedSymbol, "=", 67, 68);
    let range_start_token = token!(SurfaceTokenKind::Identifier, "a", 69, 70);
    let range_to = token!(SurfaceTokenKind::ReservedWord, "to", 71, 73);
    let range_end_token = token!(SurfaceTokenKind::Identifier, "b", 74, 75);
    let range_step = token!(SurfaceTokenKind::ReservedWord, "step", 76, 80);
    let range_step_token = token!(SurfaceTokenKind::Identifier, "s", 81, 82);
    let range_do = token!(SurfaceTokenKind::ReservedWord, "do", 83, 85);
    let range_block_end = token!(SurfaceTokenKind::ReservedWord, "end", 86, 89);
    let range_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 89, 90);
    let range_start = term_expression_node(&mut builder, source_id, range_start_token, 69, 70);
    let range_end_term = term_expression_node(&mut builder, source_id, range_end_token, 74, 75);
    let range_step_term = term_expression_node(&mut builder, source_id, range_step_token, 81, 82);
    let range_list = builder.add_node(
        SurfaceNodeKind::AlgorithmStatementList,
        range(source_id, 86, 86),
        Vec::new(),
    );
    let for_range_statement = builder.add_node(
        SurfaceNodeKind::ForRangeStatement,
        range(source_id, 61, 90),
        vec![
            range_for_kw,
            range_var,
            range_eq,
            range_start,
            range_to,
            range_end_term,
            range_step,
            range_step_term,
            range_do,
            range_list,
            range_block_end,
            range_semicolon,
        ],
    );

    let collection_for_kw = token!(SurfaceTokenKind::ReservedWord, "for", 91, 94);
    let collection_var = token!(SurfaceTokenKind::Identifier, "x", 95, 96);
    let collection_in = token!(SurfaceTokenKind::ReservedWord, "in", 97, 99);
    let collection_term_token = token!(SurfaceTokenKind::Identifier, "S", 100, 101);
    let processed_kw = token!(SurfaceTokenKind::ReservedWord, "processed", 102, 111);
    let processed_name = token!(SurfaceTokenKind::Identifier, "Seen", 112, 116);
    let collection_do = token!(SurfaceTokenKind::ReservedWord, "do", 117, 119);
    let collection_end = token!(SurfaceTokenKind::ReservedWord, "end", 120, 123);
    let collection_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 123, 124);
    let collection_term =
        term_expression_node(&mut builder, source_id, collection_term_token, 100, 101);
    let collection_list = builder.add_node(
        SurfaceNodeKind::AlgorithmStatementList,
        range(source_id, 120, 120),
        Vec::new(),
    );
    let for_collection_statement = builder.add_node(
        SurfaceNodeKind::ForCollectionStatement,
        range(source_id, 91, 124),
        vec![
            collection_for_kw,
            collection_var,
            collection_in,
            collection_term,
            processed_kw,
            processed_name,
            collection_do,
            collection_list,
            collection_end,
            collection_semicolon,
        ],
    );

    let match_kw = token!(SurfaceTokenKind::ReservedWord, "match", 125, 130);
    let scrutinee_token = token!(SurfaceTokenKind::Identifier, "t", 131, 132);
    let match_do = token!(SurfaceTokenKind::ReservedWord, "do", 133, 135);
    let case_kw = token!(SurfaceTokenKind::ReservedWord, "case", 136, 140);
    let pattern_token = token!(SurfaceTokenKind::Identifier, "p", 141, 142);
    let case_do = token!(SurfaceTokenKind::ReservedWord, "do", 143, 145);
    let case_end = token!(SurfaceTokenKind::ReservedWord, "end", 146, 149);
    let case_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 149, 150);
    let otherwise_kw = token!(SurfaceTokenKind::ReservedWord, "otherwise", 151, 160);
    let otherwise_end = token!(SurfaceTokenKind::ReservedWord, "end", 161, 164);
    let otherwise_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 164, 165);
    let match_end = token!(SurfaceTokenKind::ReservedWord, "end", 166, 169);
    let match_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 169, 170);
    let scrutinee = term_expression_node(&mut builder, source_id, scrutinee_token, 131, 132);
    let pattern = term_expression_node(&mut builder, source_id, pattern_token, 141, 142);
    let case_list = builder.add_node(
        SurfaceNodeKind::AlgorithmStatementList,
        range(source_id, 146, 146),
        Vec::new(),
    );
    let match_case = builder.add_node(
        SurfaceNodeKind::MatchCase,
        range(source_id, 136, 150),
        vec![
            case_kw,
            pattern,
            case_do,
            case_list,
            case_end,
            case_semicolon,
        ],
    );
    let otherwise_list = builder.add_node(
        SurfaceNodeKind::AlgorithmStatementList,
        range(source_id, 161, 161),
        Vec::new(),
    );
    let match_ending = builder.add_node(
        SurfaceNodeKind::MatchEnding,
        range(source_id, 151, 165),
        vec![
            otherwise_kw,
            otherwise_list,
            otherwise_end,
            otherwise_semicolon,
        ],
    );
    let match_statement = builder.add_node(
        SurfaceNodeKind::MatchStatement,
        range(source_id, 125, 170),
        vec![
            match_kw,
            scrutinee,
            match_do,
            match_case,
            match_ending,
            match_end,
            match_semicolon,
        ],
    );

    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 170),
        vec![
            if_statement,
            while_statement,
            for_range_statement,
            for_collection_statement,
            match_statement,
        ],
    );
    builder.finish(Some(root), None)
}

fn task34_algorithm_verification_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);

    macro_rules! token {
        ($kind:expr, $text:expr, $start:expr, $end:expr) => {
            builder.add_token($kind, $text, range(source_id, $start, $end))
        };
    }

    let terminating_kw = token!(SurfaceTokenKind::ReservedWord, "terminating", 0, 11);
    let algorithm_kw = token!(SurfaceTokenKind::ReservedWord, "algorithm", 12, 21);
    let algorithm_name = token!(SurfaceTokenKind::Identifier, "verified", 22, 30);
    let params_open = token!(SurfaceTokenKind::ReservedSymbol, "(", 30, 31);
    let params_close = token!(SurfaceTokenKind::ReservedSymbol, ")", 31, 32);
    let requires_kw = token!(SurfaceTokenKind::ReservedWord, "requires", 33, 41);
    let requires_token = token!(SurfaceTokenKind::ReservedWord, "thesis", 42, 48);
    let ensures_kw = token!(SurfaceTokenKind::ReservedWord, "ensures", 49, 56);
    let ensures_token = token!(SurfaceTokenKind::ReservedWord, "thesis", 57, 63);
    let header_decreasing_kw = token!(SurfaceTokenKind::ReservedWord, "decreasing", 64, 74);
    let header_measure = token!(SurfaceTokenKind::Identifier, "m", 75, 76);
    let header_comma = token!(SurfaceTokenKind::ReservedSymbol, ",", 76, 77);
    let header_next_measure = token!(SurfaceTokenKind::Identifier, "n", 78, 79);
    let algorithm_do = token!(SurfaceTokenKind::ReservedWord, "do", 80, 82);

    let while_kw = token!(SurfaceTokenKind::ReservedWord, "while", 83, 88);
    let while_condition_token = token!(SurfaceTokenKind::ReservedWord, "thesis", 89, 95);
    let while_do = token!(SurfaceTokenKind::ReservedWord, "do", 96, 98);
    let invariant_kw = token!(SurfaceTokenKind::ReservedWord, "invariant", 99, 108);
    let invariant_token = token!(SurfaceTokenKind::ReservedWord, "thesis", 109, 115);
    let invariant_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 115, 116);
    let loop_decreasing_kw = token!(SurfaceTokenKind::ReservedWord, "decreasing", 117, 127);
    let loop_measure = token!(SurfaceTokenKind::Identifier, "m", 128, 129);
    let loop_comma = token!(SurfaceTokenKind::ReservedSymbol, ",", 129, 130);
    let loop_next_measure = token!(SurfaceTokenKind::Identifier, "n", 131, 132);
    let loop_decreasing_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 132, 133);
    let assert_kw = token!(SurfaceTokenKind::ReservedWord, "assert", 134, 140);
    let assert_token = token!(SurfaceTokenKind::ReservedWord, "thesis", 141, 147);
    let assert_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 147, 148);
    let while_end = token!(SurfaceTokenKind::ReservedWord, "end", 149, 152);
    let while_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 152, 153);
    let algorithm_end = token!(SurfaceTokenKind::ReservedWord, "end", 154, 157);
    let algorithm_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 157, 158);

    let termination = builder.add_node(
        SurfaceNodeKind::AlgorithmTerminationClause,
        range(source_id, 0, 11),
        vec![terminating_kw],
    );
    let parameters = builder.add_node(
        SurfaceNodeKind::AlgorithmParameters,
        range(source_id, 30, 32),
        vec![params_open, params_close],
    );

    let requires_formula = thesis_formula_node(&mut builder, source_id, requires_token, 42, 48);
    let requires_clause = builder.add_node(
        SurfaceNodeKind::AlgorithmRequiresClause,
        range(source_id, 33, 48),
        vec![requires_kw, requires_formula],
    );
    let ensures_formula = thesis_formula_node(&mut builder, source_id, ensures_token, 57, 63);
    let ensures_clause = builder.add_node(
        SurfaceNodeKind::AlgorithmEnsuresClause,
        range(source_id, 49, 63),
        vec![ensures_kw, ensures_formula],
    );

    let header_measure_term = term_expression_node(&mut builder, source_id, header_measure, 75, 76);
    let header_next_measure_term =
        term_expression_node(&mut builder, source_id, header_next_measure, 78, 79);
    let header_term_list = builder.add_node(
        SurfaceNodeKind::TermList,
        range(source_id, 75, 79),
        vec![header_measure_term, header_comma, header_next_measure_term],
    );
    let header_decreasing_clause = builder.add_node(
        SurfaceNodeKind::AlgorithmDecreasingClause,
        range(source_id, 64, 79),
        vec![header_decreasing_kw, header_term_list],
    );

    let while_condition =
        thesis_formula_node(&mut builder, source_id, while_condition_token, 89, 95);
    let invariant_formula = thesis_formula_node(&mut builder, source_id, invariant_token, 109, 115);
    let loop_invariant_clause = builder.add_node(
        SurfaceNodeKind::LoopInvariantClause,
        range(source_id, 99, 116),
        vec![invariant_kw, invariant_formula, invariant_semicolon],
    );

    let loop_measure_term = term_expression_node(&mut builder, source_id, loop_measure, 128, 129);
    let loop_next_measure_term =
        term_expression_node(&mut builder, source_id, loop_next_measure, 131, 132);
    let loop_term_list = builder.add_node(
        SurfaceNodeKind::TermList,
        range(source_id, 128, 132),
        vec![loop_measure_term, loop_comma, loop_next_measure_term],
    );
    let loop_decreasing_clause = builder.add_node(
        SurfaceNodeKind::LoopDecreasingClause,
        range(source_id, 117, 133),
        vec![
            loop_decreasing_kw,
            loop_term_list,
            loop_decreasing_semicolon,
        ],
    );

    let assert_formula = thesis_formula_node(&mut builder, source_id, assert_token, 141, 147);
    let assert_statement = builder.add_node(
        SurfaceNodeKind::AssertStatement,
        range(source_id, 134, 148),
        vec![assert_kw, assert_formula, assert_semicolon],
    );
    let while_body_list = builder.add_node(
        SurfaceNodeKind::AlgorithmStatementList,
        range(source_id, 134, 148),
        vec![assert_statement],
    );
    let while_statement = builder.add_node(
        SurfaceNodeKind::WhileStatement,
        range(source_id, 83, 153),
        vec![
            while_kw,
            while_condition,
            while_do,
            loop_invariant_clause,
            loop_decreasing_clause,
            while_body_list,
            while_end,
            while_semicolon,
        ],
    );

    let algorithm_statement_list = builder.add_node(
        SurfaceNodeKind::AlgorithmStatementList,
        range(source_id, 83, 153),
        vec![while_statement],
    );
    let body = builder.add_node(
        SurfaceNodeKind::AlgorithmBody,
        range(source_id, 80, 157),
        vec![algorithm_do, algorithm_statement_list, algorithm_end],
    );
    let algorithm = builder.add_node(
        SurfaceNodeKind::AlgorithmDefinition,
        range(source_id, 0, 158),
        vec![
            termination,
            algorithm_kw,
            algorithm_name,
            parameters,
            requires_clause,
            ensures_clause,
            header_decreasing_clause,
            body,
            algorithm_semicolon,
        ],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 158),
        vec![algorithm],
    );
    builder.finish(Some(root), None)
}

fn task35_annotation_nodes_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);

    macro_rules! token {
        ($kind:expr, $text:expr, $start:expr, $end:expr) => {
            builder.add_token($kind, $text, range(source_id, $start, $end))
        };
    }

    let latex_marker = token!(SurfaceTokenKind::AnnotationMarker, "@latex", 0, 6);
    let latex_open = token!(SurfaceTokenKind::ReservedSymbol, "(", 6, 7);
    let latex_string = token!(SurfaceTokenKind::StringLiteral, "\"x\"", 7, 10);
    let latex_close = token!(SurfaceTokenKind::ReservedSymbol, ")", 10, 11);
    let annotation_argument = builder.add_node(
        SurfaceNodeKind::AnnotationArgument,
        range(source_id, 7, 10),
        vec![latex_string],
    );
    let annotation_argument_list = builder.add_node(
        SurfaceNodeKind::AnnotationArgumentList,
        range(source_id, 6, 11),
        vec![latex_open, annotation_argument, latex_close],
    );
    let annotation = builder.add_node(
        SurfaceNodeKind::Annotation,
        range(source_id, 0, 11),
        vec![latex_marker, annotation_argument_list],
    );

    let library_marker = token!(SurfaceTokenKind::ReservedSymbol, "@[", 12, 14);
    let library_label_name = token!(SurfaceTokenKind::Identifier, "foo", 14, 17);
    let library_close = token!(SurfaceTokenKind::ReservedSymbol, "]", 17, 18);
    let annotation_label = builder.add_node(
        SurfaceNodeKind::AnnotationLabel,
        range(source_id, 14, 17),
        vec![library_label_name],
    );
    let annotation_label_list = builder.add_node(
        SurfaceNodeKind::AnnotationLabelList,
        range(source_id, 14, 17),
        vec![annotation_label],
    );
    let library_annotation = builder.add_node(
        SurfaceNodeKind::LibraryAnnotation,
        range(source_id, 12, 18),
        vec![library_marker, annotation_label_list, library_close],
    );
    let library_wrapper = builder.add_node(
        SurfaceNodeKind::Annotation,
        range(source_id, 12, 18),
        vec![library_annotation],
    );

    let proof_marker = token!(SurfaceTokenKind::AnnotationMarker, "@proof_hint", 20, 31);
    let proof_open = token!(SurfaceTokenKind::ReservedSymbol, "(", 31, 32);
    let proof_option_name = token!(SurfaceTokenKind::Identifier, "steps", 32, 37);
    let proof_colon = token!(SurfaceTokenKind::ReservedSymbol, ":", 37, 38);
    let proof_value = token!(SurfaceTokenKind::Numeral, "3", 38, 39);
    let proof_close = token!(SurfaceTokenKind::ReservedSymbol, ")", 39, 40);
    let proof_hint_option = builder.add_node(
        SurfaceNodeKind::ProofHintOption,
        range(source_id, 32, 39),
        vec![proof_option_name, proof_colon, proof_value],
    );
    let proof_hint_options = builder.add_node(
        SurfaceNodeKind::ProofHintOptionList,
        range(source_id, 31, 40),
        vec![proof_open, proof_hint_option, proof_close],
    );
    let proof_annotation = builder.add_node(
        SurfaceNodeKind::Annotation,
        range(source_id, 20, 40),
        vec![proof_marker, proof_hint_options],
    );

    let diagnostic_marker = token!(SurfaceTokenKind::AnnotationMarker, "@show_type", 41, 51);
    let diagnostic_open = token!(SurfaceTokenKind::ReservedSymbol, "(", 51, 52);
    let diagnostic_argument = token!(SurfaceTokenKind::Identifier, "x", 52, 53);
    let diagnostic_close = token!(SurfaceTokenKind::ReservedSymbol, ")", 53, 54);
    let standalone_diagnostic = builder.add_node(
        SurfaceNodeKind::StandaloneDiagnosticAnnotation,
        range(source_id, 41, 54),
        vec![
            diagnostic_marker,
            diagnostic_open,
            diagnostic_argument,
            diagnostic_close,
        ],
    );

    let statement_annotation_marker =
        token!(SurfaceTokenKind::AnnotationMarker, "@suppress", 55, 64);
    let statement_annotation = builder.add_node(
        SurfaceNodeKind::Annotation,
        range(source_id, 55, 64),
        vec![statement_annotation_marker],
    );
    let let_keyword = token!(SurfaceTokenKind::ReservedWord, "let", 65, 68);
    let let_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 68, 69);
    let let_statement = builder.add_node(
        SurfaceNodeKind::LetStatement,
        range(source_id, 65, 69),
        vec![let_keyword, let_semicolon],
    );
    let annotated_statement = builder.add_node(
        SurfaceNodeKind::AnnotatedStatement,
        range(source_id, 55, 69),
        vec![statement_annotation, let_statement],
    );

    let algorithm_annotation_marker = token!(SurfaceTokenKind::AnnotationMarker, "@trace", 70, 76);
    let algorithm_annotation = builder.add_node(
        SurfaceNodeKind::Annotation,
        range(source_id, 70, 76),
        vec![algorithm_annotation_marker],
    );
    let return_keyword = token!(SurfaceTokenKind::ReservedWord, "return", 77, 83);
    let return_semicolon = token!(SurfaceTokenKind::ReservedSymbol, ";", 83, 84);
    let return_statement = builder.add_node(
        SurfaceNodeKind::ReturnStatement,
        range(source_id, 77, 84),
        vec![return_keyword, return_semicolon],
    );
    let annotated_algorithm_statement = builder.add_node(
        SurfaceNodeKind::AnnotatedAlgorithmStatement,
        range(source_id, 70, 84),
        vec![algorithm_annotation, return_statement],
    );

    let definition_annotation_marker = token!(SurfaceTokenKind::AnnotationMarker, "@def", 85, 89);
    let definition_annotation = builder.add_node(
        SurfaceNodeKind::Annotation,
        range(source_id, 85, 89),
        vec![definition_annotation_marker],
    );
    let attr_keyword = token!(SurfaceTokenKind::ReservedWord, "attr", 90, 94);
    let attribute_definition = builder.add_node(
        SurfaceNodeKind::AttributeDefinition,
        range(source_id, 90, 94),
        vec![attr_keyword],
    );
    let annotated_definition_content = builder.add_node(
        SurfaceNodeKind::AnnotatedDefinitionContent,
        range(source_id, 85, 94),
        vec![definition_annotation, attribute_definition],
    );

    let registration_annotation_marker = token!(SurfaceTokenKind::AnnotationMarker, "@reg", 95, 99);
    let registration_annotation = builder.add_node(
        SurfaceNodeKind::Annotation,
        range(source_id, 95, 99),
        vec![registration_annotation_marker],
    );
    let reduce_keyword = token!(SurfaceTokenKind::ReservedWord, "reduce", 100, 106);
    let reduction_registration = builder.add_node(
        SurfaceNodeKind::ReductionRegistration,
        range(source_id, 100, 106),
        vec![reduce_keyword],
    );
    let annotated_registration_content = builder.add_node(
        SurfaceNodeKind::AnnotatedRegistrationContent,
        range(source_id, 95, 106),
        vec![registration_annotation, reduction_registration],
    );

    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 106),
        vec![
            annotation,
            library_wrapper,
            proof_annotation,
            standalone_diagnostic,
            annotated_statement,
            annotated_algorithm_statement,
            annotated_definition_content,
            annotated_registration_content,
        ],
    );
    builder.finish(Some(root), None)
}

fn thesis_formula_node(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    token: super::SurfaceBuilderNodeId,
    start: usize,
    end: usize,
) -> super::SurfaceBuilderNodeId {
    let thesis_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, start, end),
        vec![token],
    );
    builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, start, end),
        vec![thesis_constant],
    )
}

fn thesis_proposition_node(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    token: super::SurfaceBuilderNodeId,
    start: usize,
    end: usize,
) -> super::SurfaceBuilderNodeId {
    let thesis_constant = builder.add_node(
        SurfaceNodeKind::FormulaConstant(SurfaceFormulaConstant::Thesis),
        range(source_id, start, end),
        vec![token],
    );
    let formula = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, start, end),
        vec![thesis_constant],
    );
    builder.add_node(
        SurfaceNodeKind::Proposition,
        range(source_id, start, end),
        vec![formula],
    )
}

fn term_expression_node(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    token: super::SurfaceBuilderNodeId,
    start: usize,
    end: usize,
) -> super::SurfaceBuilderNodeId {
    let reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, start, end),
        vec![token],
    );
    builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, start, end),
        vec![reference],
    )
}

fn recovery_ast(source_id: SourceId, recovery_kind: SyntaxRecoveryKind) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let recovery = builder.add_recovery(recovery_kind, range(source_id, 0, 0), Vec::new());
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 0),
        vec![recovery],
    );
    builder.finish(Some(root), None)
}

#[derive(Clone, Copy)]
struct RecoveryFixture {
    kind: SyntaxRecoveryKind,
    range: SourceRange,
    has_context_child: bool,
}

fn recovery_fixtures(source_id: SourceId) -> Vec<RecoveryFixture> {
    vec![
        RecoveryFixture {
            kind: SyntaxRecoveryKind::ErrorToken,
            range: range(source_id, 8, 11),
            has_context_child: false,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::MissingEnd,
            range: range(source_id, 12, 12),
            has_context_child: true,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::MissingStringLiteral,
            range: range(source_id, 13, 13),
            has_context_child: false,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::MissingItem,
            range: range(source_id, 14, 14),
            has_context_child: true,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::MissingTypeExpression,
            range: range(source_id, 15, 15),
            has_context_child: true,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::MissingTerm,
            range: range(source_id, 16, 16),
            has_context_child: true,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::MissingFormula,
            range: range(source_id, 17, 17),
            has_context_child: true,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::MissingStatement,
            range: range(source_id, 18, 18),
            has_context_child: true,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::MissingProofStep,
            range: range(source_id, 19, 19),
            has_context_child: true,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::MissingAnnotationArgument,
            range: range(source_id, 20, 20),
            has_context_child: true,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::SkippedToken,
            range: range(source_id, 21, 24),
            has_context_child: false,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
            range: range(source_id, 25, 25),
            has_context_child: true,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::UnmatchedClosingDelimiter,
            range: range(source_id, 26, 27),
            has_context_child: false,
        },
        RecoveryFixture {
            kind: SyntaxRecoveryKind::MalformedAnnotation,
            range: range(source_id, 28, 35),
            has_context_child: false,
        },
    ]
}

fn all_recovery_kinds() -> [SyntaxRecoveryKind; 14] {
    [
        SyntaxRecoveryKind::ErrorToken,
        SyntaxRecoveryKind::MissingEnd,
        SyntaxRecoveryKind::MissingStringLiteral,
        SyntaxRecoveryKind::MissingItem,
        SyntaxRecoveryKind::MissingTypeExpression,
        SyntaxRecoveryKind::MissingTerm,
        SyntaxRecoveryKind::MissingFormula,
        SyntaxRecoveryKind::MissingStatement,
        SyntaxRecoveryKind::MissingProofStep,
        SyntaxRecoveryKind::MissingAnnotationArgument,
        SyntaxRecoveryKind::SkippedToken,
        SyntaxRecoveryKind::UnmatchedOpeningDelimiter,
        SyntaxRecoveryKind::UnmatchedClosingDelimiter,
        SyntaxRecoveryKind::MalformedAnnotation,
    ]
}

fn current_vocabulary_snapshot_ast(source_id: SourceId) -> crate::SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let identifier = builder.add_token(SurfaceTokenKind::Identifier, "id", range(source_id, 0, 2));
    let reserved_word = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 3, 10),
    );
    let reserved_symbol = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 10, 11),
    );
    let numeral = builder.add_token(SurfaceTokenKind::Numeral, "42", range(source_id, 12, 14));
    let lexeme_run =
        builder.add_token(SurfaceTokenKind::LexemeRun, "abc", range(source_id, 15, 18));
    let user_symbol =
        builder.add_token(SurfaceTokenKind::UserSymbol, "++", range(source_id, 19, 21));
    let string_literal = builder.add_token(
        SurfaceTokenKind::StringLiteral,
        "line\nvalue",
        range(source_id, 22, 32),
    );
    let error_token = builder.add_token(
        SurfaceTokenKind::ErrorRecovery,
        "<error>",
        range(source_id, 32, 39),
    );
    let unknown = builder.add_token(SurfaceTokenKind::Unknown, "?", range(source_id, 39, 40));
    let recovered_token = builder.add_recovered_token(
        SurfaceTokenKind::ErrorRecovery,
        "bad\ttext",
        range(source_id, 40, 48),
    );
    let module_prefix = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "..",
        range(source_id, 49, 51),
    );
    let module_std = builder.add_token(
        SurfaceTokenKind::Identifier,
        "std",
        range(source_id, 51, 54),
    );
    let module_dot = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 54, 55),
    );
    let module_algebra = builder.add_token(
        SurfaceTokenKind::Identifier,
        "algebra",
        range(source_id, 55, 62),
    );
    let namespace_mml = builder.add_token(
        SurfaceTokenKind::Identifier,
        "mml",
        range(source_id, 63, 66),
    );
    let namespace_dot = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 66, 67),
    );
    let namespace_nat = builder.add_token(
        SurfaceTokenKind::Identifier,
        "nat",
        range(source_id, 67, 70),
    );
    let qualified_top = builder.add_token(
        SurfaceTokenKind::Identifier,
        "top",
        range(source_id, 71, 74),
    );
    let qualified_dot = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 74, 75),
    );
    let qualified_space = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "Space",
        range(source_id, 75, 80),
    );
    let item_theorem = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 81, 88),
    );
    let item_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 88, 89),
    );
    let import_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "import",
        range(source_id, 90, 96),
    );
    let import_path_prefix = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 97, 98),
    );
    let import_path_core = builder.add_token(
        SurfaceTokenKind::Identifier,
        "core",
        range(source_id, 98, 102),
    );
    let import_branch_opener = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".{",
        range(source_id, 102, 104),
    );
    let import_branch_segment = builder.add_token(
        SurfaceTokenKind::Identifier,
        "linear",
        range(source_id, 104, 110),
    );
    let import_branch_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "}",
        range(source_id, 110, 111),
    );
    let import_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 111, 112),
    );
    let import_alias_path = builder.add_token(
        SurfaceTokenKind::Identifier,
        "algebra",
        range(source_id, 113, 120),
    );
    let import_alias_as = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "as",
        range(source_id, 121, 123),
    );
    let import_alias = builder.add_token(
        SurfaceTokenKind::Identifier,
        "A",
        range(source_id, 124, 125),
    );
    let import_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 125, 126),
    );
    let export_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "export",
        range(source_id, 127, 133),
    );
    let export_path_std = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Std",
        range(source_id, 134, 137),
    );
    let export_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 137, 138),
    );
    let visibility_public = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "public",
        range(source_id, 139, 145),
    );
    let visible_theorem = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "theorem",
        range(source_id, 146, 153),
    );
    let visible_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 153, 154),
    );
    let reserve_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "reserve",
        range(source_id, 155, 162),
    );
    let reserve_x = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 163, 164),
    );
    let reserve_for = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "for",
        range(source_id, 165, 168),
    );
    let reserve_non = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "non",
        range(source_id, 169, 172),
    );
    let reserve_empty = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "empty",
        range(source_id, 173, 178),
    );
    let reserve_type_symbol = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "T",
        range(source_id, 179, 180),
    );
    let reserve_arg_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "[",
        range(source_id, 181, 182),
    );
    let reserve_set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 182, 185),
    );
    let reserve_arg_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 185, 186),
    );
    let reserve_arg_identifier = builder.add_token(
        SurfaceTokenKind::Identifier,
        "V",
        range(source_id, 187, 188),
    );
    let reserve_qua = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "qua",
        range(source_id, 189, 192),
    );
    let reserve_radix = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "R",
        range(source_id, 193, 194),
    );
    let reserve_arg_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "]",
        range(source_id, 194, 195),
    );
    let reserve_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 195, 196),
    );
    let standalone_prefix_n = builder.add_token(
        SurfaceTokenKind::Identifier,
        "n",
        range(source_id, 197, 198),
    );
    let standalone_prefix_hyphen = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "-",
        range(source_id, 198, 199),
    );
    let term_reference_token = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 200, 201),
    );
    let term_numeral_token =
        builder.add_token(SurfaceTokenKind::Numeral, "99", range(source_id, 202, 204));
    let term_it_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "it",
        range(source_id, 205, 207),
    );
    let term_paren_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 208, 209),
    );
    let term_paren_identifier = builder.add_token(
        SurfaceTokenKind::Identifier,
        "p",
        range(source_id, 209, 210),
    );
    let term_paren_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 210, 211),
    );
    let term_the = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "the",
        range(source_id, 212, 215),
    );
    let term_choice_set = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "set",
        range(source_id, 216, 219),
    );
    let term_apply_symbol = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "F",
        range(source_id, 220, 221),
    );
    let term_apply_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 221, 222),
    );
    let term_apply_arg = builder.add_token(
        SurfaceTokenKind::Identifier,
        "a",
        range(source_id, 222, 223),
    );
    let term_apply_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 223, 224),
    );
    let term_struct_symbol = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "S",
        range(source_id, 225, 226),
    );
    let term_struct_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 226, 227),
    );
    let term_field_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 227, 228),
    );
    let term_field_colon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":",
        range(source_id, 228, 229),
    );
    let term_field_value = builder.add_token(
        SurfaceTokenKind::Identifier,
        "y",
        range(source_id, 229, 230),
    );
    let term_struct_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 230, 231),
    );
    let term_set_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "{",
        range(source_id, 232, 233),
    );
    let term_set_first = builder.add_token(
        SurfaceTokenKind::Identifier,
        "a",
        range(source_id, 233, 234),
    );
    let term_set_comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 234, 235),
    );
    let term_set_second = builder.add_token(
        SurfaceTokenKind::Identifier,
        "b",
        range(source_id, 235, 236),
    );
    let term_set_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "}",
        range(source_id, 236, 237),
    );
    let term_selector_base = builder.add_token(
        SurfaceTokenKind::Identifier,
        "p",
        range(source_id, 238, 239),
    );
    let term_selector_dot = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 239, 240),
    );
    let term_selector_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 240, 241),
    );
    let term_selector_call_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 241, 242),
    );
    let term_selector_call_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 242, 243),
    );
    let term_update_base = builder.add_token(
        SurfaceTokenKind::Identifier,
        "p",
        range(source_id, 244, 245),
    );
    let term_update_with = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "with",
        range(source_id, 246, 250),
    );
    let term_update_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 251, 252),
    );
    let term_update_field_start = builder.add_token(
        SurfaceTokenKind::Identifier,
        "start",
        range(source_id, 252, 257),
    );
    let term_update_field_dot = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ".",
        range(source_id, 257, 258),
    );
    let term_update_field_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 258, 259),
    );
    let term_update_assign = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ":=",
        range(source_id, 260, 262),
    );
    let term_update_value = builder.add_token(
        SurfaceTokenKind::Identifier,
        "y",
        range(source_id, 263, 264),
    );
    let term_update_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 264, 265),
    );
    let term_qua_base = builder.add_token(
        SurfaceTokenKind::Identifier,
        "q",
        range(source_id, 266, 267),
    );
    let term_qua_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "qua",
        range(source_id, 268, 271),
    );
    let term_qua_target = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "T",
        range(source_id, 272, 273),
    );
    let formula_left = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, 274, 275),
    );
    let formula_equals = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "=",
        range(source_id, 276, 277),
    );
    let formula_right = builder.add_token(
        SurfaceTokenKind::Identifier,
        "y",
        range(source_id, 278, 279),
    );
    let formula_subject = builder.add_token(
        SurfaceTokenKind::Identifier,
        "z",
        range(source_id, 280, 281),
    );
    let formula_is = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "is",
        range(source_id, 282, 284),
    );
    let formula_non = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "non",
        range(source_id, 285, 288),
    );
    let formula_empty = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "empty",
        range(source_id, 289, 294),
    );
    let predicate_left = builder.add_token(
        SurfaceTokenKind::Identifier,
        "a",
        range(source_id, 295, 296),
    );
    let predicate_divides = builder.add_token(
        SurfaceTokenKind::UserSymbol,
        "divides",
        range(source_id, 297, 304),
    );
    let predicate_right = builder.add_token(
        SurfaceTokenKind::Identifier,
        "b",
        range(source_id, 305, 306),
    );
    let inline_predicate = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Small",
        range(source_id, 307, 312),
    );
    let inline_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 312, 313),
    );
    let inline_arg = builder.add_token(
        SurfaceTokenKind::Identifier,
        "c",
        range(source_id, 313, 314),
    );
    let inline_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 314, 315),
    );
    let expression = builder.add_node(
        SurfaceNodeKind::InfixExpression(SurfaceInfixOperator {
            spelling: "++".into(),
            precedence: 10,
            associativity: SurfaceOperatorAssociativity::Right,
        }),
        range(source_id, 0, 21),
        vec![identifier, user_symbol, numeral],
    );
    let module_prefix_node = builder.add_node(
        SurfaceNodeKind::RelativePrefix,
        range(source_id, 49, 51),
        vec![module_prefix],
    );
    let module_std_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 51, 54),
        vec![module_std],
    );
    let module_algebra_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 55, 62),
        vec![module_algebra],
    );
    let module_path = builder.add_node(
        SurfaceNodeKind::ModulePath,
        range(source_id, 49, 62),
        vec![
            module_prefix_node,
            module_std_node,
            module_dot,
            module_algebra_node,
        ],
    );
    let namespace_mml_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 63, 66),
        vec![namespace_mml],
    );
    let namespace_nat_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 67, 70),
        vec![namespace_nat],
    );
    let namespace_path = builder.add_node(
        SurfaceNodeKind::NamespacePath,
        range(source_id, 63, 70),
        vec![namespace_mml_node, namespace_dot, namespace_nat_node],
    );
    let qualified_top_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 71, 74),
        vec![qualified_top],
    );
    let qualified_space_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 75, 80),
        vec![qualified_space],
    );
    let qualified_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 71, 80),
        vec![qualified_top_node, qualified_dot, qualified_space_node],
    );
    let placeholder_item = builder.add_node(
        SurfaceNodeKind::PlaceholderItem,
        range(source_id, 81, 89),
        vec![item_theorem, item_semicolon],
    );
    let import_path_prefix_node = builder.add_node(
        SurfaceNodeKind::RelativePrefix,
        range(source_id, 97, 98),
        vec![import_path_prefix],
    );
    let import_path_core_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 98, 102),
        vec![import_path_core],
    );
    let import_branch_path = builder.add_node(
        SurfaceNodeKind::ModulePath,
        range(source_id, 97, 102),
        vec![import_path_prefix_node, import_path_core_node],
    );
    let import_branch_segment_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 104, 110),
        vec![import_branch_segment],
    );
    let module_branch_import = builder.add_node(
        SurfaceNodeKind::ModuleBranchImport,
        range(source_id, 97, 111),
        vec![
            import_branch_path,
            import_branch_opener,
            import_branch_segment_node,
            import_branch_close,
        ],
    );
    let import_alias_path_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 113, 120),
        vec![import_alias_path],
    );
    let import_alias_module_path = builder.add_node(
        SurfaceNodeKind::ModulePath,
        range(source_id, 113, 120),
        vec![import_alias_path_node],
    );
    let import_alias_node = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 124, 125),
        vec![import_alias],
    );
    let import_alias_decl = builder.add_node(
        SurfaceNodeKind::ImportAliasDecl,
        range(source_id, 113, 125),
        vec![import_alias_module_path, import_alias_as, import_alias_node],
    );
    let import_item = builder.add_node(
        SurfaceNodeKind::ImportItem,
        range(source_id, 90, 126),
        vec![
            import_keyword,
            module_branch_import,
            import_comma,
            import_alias_decl,
            import_semicolon,
        ],
    );
    let export_path_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 134, 137),
        vec![export_path_std],
    );
    let export_module_path = builder.add_node(
        SurfaceNodeKind::ModulePath,
        range(source_id, 134, 137),
        vec![export_path_segment],
    );
    let export_item = builder.add_node(
        SurfaceNodeKind::ExportItem,
        range(source_id, 127, 138),
        vec![export_keyword, export_module_path, export_semicolon],
    );
    let visibility_marker = builder.add_node(
        SurfaceNodeKind::VisibilityMarker,
        range(source_id, 139, 145),
        vec![visibility_public],
    );
    let visible_placeholder = builder.add_node(
        SurfaceNodeKind::PlaceholderItem,
        range(source_id, 146, 154),
        vec![visible_theorem, visible_semicolon],
    );
    let visible_item = builder.add_node(
        SurfaceNodeKind::VisibleItem,
        range(source_id, 139, 154),
        vec![visibility_marker, visible_placeholder],
    );
    let reserve_empty_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 173, 178),
        vec![reserve_empty],
    );
    let reserve_empty_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 173, 178),
        vec![reserve_empty_segment],
    );
    let reserve_attribute = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(source_id, 169, 178),
        vec![reserve_non, reserve_empty_symbol],
    );
    let reserve_attribute_chain = builder.add_node(
        SurfaceNodeKind::AttributeChain,
        range(source_id, 169, 178),
        vec![reserve_attribute],
    );
    let reserve_type_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 179, 180),
        vec![reserve_type_symbol],
    );
    let reserve_type_symbol_node = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 179, 180),
        vec![reserve_type_segment],
    );
    let reserve_set_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 182, 185),
        vec![reserve_set],
    );
    let reserve_set_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 182, 185),
        vec![reserve_set_head],
    );
    let reserve_qua_placeholder = builder.add_node(
        SurfaceNodeKind::TermPlaceholder,
        range(source_id, 187, 194),
        vec![reserve_arg_identifier, reserve_qua, reserve_radix],
    );
    let reserve_type_arguments = builder.add_node(
        SurfaceNodeKind::TypeArguments,
        range(source_id, 181, 195),
        vec![
            reserve_arg_open,
            reserve_set_expression,
            reserve_arg_comma,
            reserve_qua_placeholder,
            reserve_arg_close,
        ],
    );
    let reserve_type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 179, 195),
        vec![reserve_type_symbol_node, reserve_type_arguments],
    );
    let reserve_type_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 169, 195),
        vec![reserve_attribute_chain, reserve_type_head],
    );
    let reserve_segment = builder.add_node(
        SurfaceNodeKind::ReserveSegment,
        range(source_id, 163, 195),
        vec![reserve_x, reserve_for, reserve_type_expression],
    );
    let reserve_item = builder.add_node(
        SurfaceNodeKind::ReserveItem,
        range(source_id, 155, 196),
        vec![reserve_keyword, reserve_segment, reserve_semicolon],
    );
    let standalone_parameter_prefix = builder.add_node(
        SurfaceNodeKind::ParameterPrefix,
        range(source_id, 197, 199),
        vec![standalone_prefix_n, standalone_prefix_hyphen],
    );
    let term_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 200, 201),
        vec![term_reference_token],
    );
    let term_reference_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 200, 201),
        vec![term_reference],
    );
    let numeral_term = builder.add_node(
        SurfaceNodeKind::NumeralTerm,
        range(source_id, 202, 204),
        vec![term_numeral_token],
    );
    let numeral_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 202, 204),
        vec![numeral_term],
    );
    let it_term = builder.add_node(
        SurfaceNodeKind::ItTerm,
        range(source_id, 205, 207),
        vec![term_it_token],
    );
    let it_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 205, 207),
        vec![it_term],
    );
    let paren_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 209, 210),
        vec![term_paren_identifier],
    );
    let paren_inner_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 209, 210),
        vec![paren_reference],
    );
    let parenthesized_term = builder.add_node(
        SurfaceNodeKind::ParenthesizedTerm,
        range(source_id, 208, 211),
        vec![term_paren_open, paren_inner_expression, term_paren_close],
    );
    let parenthesized_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 208, 211),
        vec![parenthesized_term],
    );
    let choice_set_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 216, 219),
        vec![term_choice_set],
    );
    let choice_type_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 216, 219),
        vec![choice_set_head],
    );
    let choice_term = builder.add_node(
        SurfaceNodeKind::ChoiceTerm,
        range(source_id, 212, 219),
        vec![term_the, choice_type_expression],
    );
    let choice_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 212, 219),
        vec![choice_term],
    );
    let apply_symbol_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 220, 221),
        vec![term_apply_symbol],
    );
    let apply_symbol_node = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 220, 221),
        vec![apply_symbol_segment],
    );
    let apply_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 220, 221),
        vec![apply_symbol_node],
    );
    let apply_argument_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 222, 223),
        vec![term_apply_arg],
    );
    let apply_argument = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 222, 223),
        vec![apply_argument_reference],
    );
    let application_term = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        range(source_id, 220, 224),
        vec![
            apply_reference,
            term_apply_open,
            apply_argument,
            term_apply_close,
        ],
    );
    let application_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 220, 224),
        vec![application_term],
    );
    let struct_symbol_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 225, 226),
        vec![term_struct_symbol],
    );
    let struct_symbol_node = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 225, 226),
        vec![struct_symbol_segment],
    );
    let field_value_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 229, 230),
        vec![term_field_value],
    );
    let field_value_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 229, 230),
        vec![field_value_reference],
    );
    let field_argument = builder.add_node(
        SurfaceNodeKind::FieldArgument,
        range(source_id, 227, 230),
        vec![term_field_name, term_field_colon, field_value_expression],
    );
    let structure_constructor = builder.add_node(
        SurfaceNodeKind::StructureConstructor,
        range(source_id, 225, 231),
        vec![
            struct_symbol_node,
            term_struct_open,
            field_argument,
            term_struct_close,
        ],
    );
    let structure_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 225, 231),
        vec![structure_constructor],
    );
    let set_first_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 233, 234),
        vec![term_set_first],
    );
    let set_first_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 233, 234),
        vec![set_first_reference],
    );
    let set_second_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 235, 236),
        vec![term_set_second],
    );
    let set_second_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 235, 236),
        vec![set_second_reference],
    );
    let set_enumeration = builder.add_node(
        SurfaceNodeKind::SetEnumeration,
        range(source_id, 232, 237),
        vec![
            term_set_open,
            set_first_expression,
            term_set_comma,
            set_second_expression,
            term_set_close,
        ],
    );
    let set_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 232, 237),
        vec![set_enumeration],
    );
    let selector_base_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 238, 239),
        vec![term_selector_base],
    );
    let selector_access = builder.add_node(
        SurfaceNodeKind::SelectorAccess,
        range(source_id, 238, 243),
        vec![
            selector_base_reference,
            term_selector_dot,
            term_selector_name,
            term_selector_call_open,
            term_selector_call_close,
        ],
    );
    let selector_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 238, 243),
        vec![selector_access],
    );
    let update_base_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 244, 245),
        vec![term_update_base],
    );
    let update_value_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 263, 264),
        vec![term_update_value],
    );
    let update_value_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 263, 264),
        vec![update_value_reference],
    );
    let field_update = builder.add_node(
        SurfaceNodeKind::FieldUpdate,
        range(source_id, 252, 264),
        vec![
            term_update_field_start,
            term_update_field_dot,
            term_update_field_name,
            term_update_assign,
            update_value_expression,
        ],
    );
    let structure_update = builder.add_node(
        SurfaceNodeKind::StructureUpdate,
        range(source_id, 244, 265),
        vec![
            update_base_reference,
            term_update_with,
            term_update_open,
            field_update,
            term_update_close,
        ],
    );
    let structure_update_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 244, 265),
        vec![structure_update],
    );
    let qua_base_reference = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 266, 267),
        vec![term_qua_base],
    );
    let qua_target_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 272, 273),
        vec![term_qua_target],
    );
    let qua_target_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 272, 273),
        vec![qua_target_segment],
    );
    let qua_target_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 272, 273),
        vec![qua_target_symbol],
    );
    let qua_target_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 272, 273),
        vec![qua_target_head],
    );
    let qua_expression = builder.add_node(
        SurfaceNodeKind::QuaExpression,
        range(source_id, 266, 273),
        vec![qua_base_reference, term_qua_keyword, qua_target_expression],
    );
    let qua_term_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 266, 273),
        vec![qua_expression],
    );
    let formula_left_term = term_expression_node(&mut builder, source_id, formula_left, 274, 275);
    let formula_right_term = term_expression_node(&mut builder, source_id, formula_right, 278, 279);
    let builtin_predicate_application = builder.add_node(
        SurfaceNodeKind::BuiltinPredicateApplication,
        range(source_id, 274, 279),
        vec![formula_left_term, formula_equals, formula_right_term],
    );
    let builtin_formula_expression = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 274, 279),
        vec![builtin_predicate_application],
    );
    let formula_subject_term =
        term_expression_node(&mut builder, source_id, formula_subject, 280, 281);
    let formula_empty_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 289, 294),
        vec![formula_empty],
    );
    let formula_empty_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 289, 294),
        vec![formula_empty_segment],
    );
    let formula_attribute = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(source_id, 285, 294),
        vec![formula_non, formula_empty_symbol],
    );
    let formula_attribute_test_chain = builder.add_node(
        SurfaceNodeKind::AttributeTestChain,
        range(source_id, 285, 294),
        vec![formula_attribute],
    );
    let is_assertion = builder.add_node(
        SurfaceNodeKind::IsAssertion,
        range(source_id, 280, 294),
        vec![
            formula_subject_term,
            formula_is,
            formula_attribute_test_chain,
        ],
    );
    let is_formula_expression = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 280, 294),
        vec![is_assertion],
    );
    let predicate_left_term =
        term_expression_node(&mut builder, source_id, predicate_left, 295, 296);
    let predicate_right_term =
        term_expression_node(&mut builder, source_id, predicate_right, 305, 306);
    let predicate_segment = builder.add_node(
        SurfaceNodeKind::PathSegment,
        range(source_id, 297, 304),
        vec![predicate_divides],
    );
    let predicate_symbol = builder.add_node(
        SurfaceNodeKind::QualifiedSymbol,
        range(source_id, 297, 304),
        vec![predicate_segment],
    );
    let predicate_head = builder.add_node(
        SurfaceNodeKind::PredicateHead,
        range(source_id, 297, 304),
        vec![predicate_symbol],
    );
    let predicate_segment = builder.add_node(
        SurfaceNodeKind::PredicateSegment,
        range(source_id, 295, 306),
        vec![predicate_left_term, predicate_head, predicate_right_term],
    );
    let predicate_application = builder.add_node(
        SurfaceNodeKind::PredicateApplication,
        range(source_id, 295, 306),
        vec![predicate_segment],
    );
    let predicate_formula_expression = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 295, 306),
        vec![predicate_application],
    );
    let inline_arg_term = term_expression_node(&mut builder, source_id, inline_arg, 313, 314);
    let inline_predicate_application = builder.add_node(
        SurfaceNodeKind::InlinePredicateApplication,
        range(source_id, 307, 315),
        vec![inline_predicate, inline_open, inline_arg_term, inline_close],
    );
    let inline_formula_expression = builder.add_node(
        SurfaceNodeKind::FormulaExpression,
        range(source_id, 307, 315),
        vec![inline_predicate_application],
    );
    let item_list = builder.add_node(
        SurfaceNodeKind::ItemList,
        range(source_id, 81, 154),
        vec![placeholder_item, import_item, export_item, visible_item],
    );
    let compilation_unit = builder.add_node(
        SurfaceNodeKind::CompilationUnit,
        range(source_id, 81, 154),
        vec![item_list],
    );
    let recovery = builder.add_recovery(
        SyntaxRecoveryKind::MissingEnd,
        range(source_id, 89, 89),
        vec![reserved_word],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 315),
        vec![
            identifier,
            reserved_word,
            reserved_symbol,
            numeral,
            lexeme_run,
            user_symbol,
            string_literal,
            error_token,
            unknown,
            recovered_token,
            module_prefix,
            module_std,
            module_dot,
            module_algebra,
            namespace_mml,
            namespace_dot,
            namespace_nat,
            qualified_top,
            qualified_dot,
            qualified_space,
            item_theorem,
            item_semicolon,
            import_keyword,
            import_path_prefix,
            import_path_core,
            import_branch_opener,
            import_branch_segment,
            import_branch_close,
            import_comma,
            import_alias_path,
            import_alias_as,
            import_alias,
            import_semicolon,
            export_keyword,
            export_path_std,
            export_semicolon,
            visibility_public,
            visible_theorem,
            visible_semicolon,
            reserve_keyword,
            reserve_x,
            reserve_for,
            reserve_non,
            reserve_empty,
            reserve_type_symbol,
            reserve_arg_open,
            reserve_set,
            reserve_arg_comma,
            reserve_arg_identifier,
            reserve_qua,
            reserve_radix,
            reserve_arg_close,
            reserve_semicolon,
            standalone_prefix_n,
            standalone_prefix_hyphen,
            term_reference_token,
            term_numeral_token,
            term_it_token,
            term_paren_open,
            term_paren_identifier,
            term_paren_close,
            term_the,
            term_choice_set,
            term_apply_symbol,
            term_apply_open,
            term_apply_arg,
            term_apply_close,
            term_struct_symbol,
            term_struct_open,
            term_field_name,
            term_field_colon,
            term_field_value,
            term_struct_close,
            term_set_open,
            term_set_first,
            term_set_comma,
            term_set_second,
            term_set_close,
            term_selector_base,
            term_selector_dot,
            term_selector_name,
            term_selector_call_open,
            term_selector_call_close,
            term_update_base,
            term_update_with,
            term_update_open,
            term_update_field_start,
            term_update_field_dot,
            term_update_field_name,
            term_update_assign,
            term_update_value,
            term_update_close,
            term_qua_base,
            term_qua_keyword,
            term_qua_target,
            formula_left,
            formula_equals,
            formula_right,
            formula_subject,
            formula_is,
            formula_non,
            formula_empty,
            predicate_left,
            predicate_divides,
            predicate_right,
            inline_predicate,
            inline_open,
            inline_arg,
            inline_close,
            expression,
            module_path,
            namespace_path,
            qualified_symbol,
            compilation_unit,
            reserve_item,
            standalone_parameter_prefix,
            term_reference_expression,
            numeral_expression,
            it_expression,
            parenthesized_expression,
            choice_expression,
            application_expression,
            structure_expression,
            set_expression,
            selector_expression,
            structure_update_expression,
            qua_term_expression,
            builtin_formula_expression,
            is_formula_expression,
            predicate_formula_expression,
            inline_formula_expression,
            recovery,
        ],
    );
    builder.finish(Some(root), Some(expression))
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

const fn sid(id: super::SurfaceBuilderNodeId) -> super::SurfaceNodeId {
    id.into_surface_node_id()
}

fn first_view<'a>(
    view: SurfaceNodeView<'a>,
    predicate: impl Fn(&SurfaceNodeKind) -> bool + Copy,
) -> Option<SurfaceNodeView<'a>> {
    if predicate(view.kind()) {
        return Some(view);
    }
    view.child_views()
        .find_map(|child| first_view(child, predicate))
}

fn child_signature(view: SurfaceNodeView<'_>) -> Vec<&'static str> {
    view.child_views()
        .map(|child| {
            if let Some(token) = child.as_token() {
                return match token.text.as_ref() {
                    "redefine" => "redefine",
                    "pred" => "pred",
                    "P" => "P",
                    ":" => ":",
                    "means" => "means",
                    ";" => ";",
                    _ => "Token",
                };
            }
            match child.kind() {
                SurfaceNodeKind::ErrorRecovery(SyntaxRecoveryKind::MissingTerm) => "MissingTerm",
                SurfaceNodeKind::PredicatePattern => "PredicatePattern",
                SurfaceNodeKind::FormulaDefiniens => "FormulaDefiniens",
                SurfaceNodeKind::CoherenceCondition => "CoherenceCondition",
                _ => "Node",
            }
        })
        .collect()
}

fn snapshot_id(byte: u8) -> BuildSnapshotId {
    let hex = format!("{byte:02x}").repeat(Hash::BYTE_LEN);
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .unwrap()
}
