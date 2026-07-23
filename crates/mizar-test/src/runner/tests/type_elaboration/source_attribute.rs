#[test]
fn task250_real_routes_publish_exact_raw_handoffs_and_preserve_legacy_outcomes() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 250 repository plan should build");
    let cases = [
        (
            "fail_type_elaboration_argument_bearing_attribute_gap_001",
            [1, 1, 0, 1, 1],
            false,
            "type_elaboration.checker.source_attribute.semantic_dependencies_pending",
        ),
        (
            "fail_type_elaboration_structure_qualified_attribute_gap_001",
            [1, 1, 1, 0, 0],
            false,
            "type_elaboration.checker.source_attribute.semantic_dependencies_pending",
        ),
        (
            "fail_type_elaboration_imported_attribute_gap_001",
            [1, 1, 0, 0, 0],
            true,
            "type_elaboration.checker.checker.declaration.deferred.evidence_query",
        ),
        (
            "fail_type_elaboration_attributed_reserve_gap_001",
            [1, 1, 0, 0, 0],
            true,
            "type_elaboration.checker.checker.declaration.deferred.evidence_query",
        ),
    ];
    let mut aggregate = [0_usize; 5];
    let mut polarities = [0_usize; 2];
    let mut provenance = [0_usize; 2];
    let mut heads = [0_usize; 2];

    for (id, expected, preserves_legacy, expected_key) in cases {
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == id)
            .unwrap_or_else(|| panic!("{id} should remain active"));
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{id} frontend failed: {error}"));
        assert!(frontend.diagnostics.is_empty(), "{id}");
        let ast = frontend.ast.unwrap_or_else(|| panic!("{id} AST"));
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        assert!(resolver.detail_keys.is_empty(), "{id}");
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        assert_eq!(
            source_attribute_detail_keys(&ast, resolver.module.clone(), &symbols),
            Some(vec![expected_key.to_owned()]),
            "{id}"
        );

        let first = source_attribute_output(&ast, resolver.module.clone(), &symbols)
            .unwrap_or_else(|| panic!("{id} should select Task 250"))
            .unwrap_or_else(|error| panic!("{id} Task 250 failed: {error}"));
        let second = source_attribute_output(&ast, resolver.module.clone(), &symbols)
            .unwrap_or_else(|| panic!("{id} should remain selected"))
            .unwrap_or_else(|error| panic!("{id} repeated Task 250 failed: {error}"));
        let source_type = first
            .typed_ast
            .source_type()
            .expect("Task 250 should co-install Task 249");
        let handoff = first
            .typed_ast
            .source_attribute()
            .expect("Task 250 handoff should be installed");
        assert_eq!(
            [
                handoff.chains().len(),
                handoff.attributes().len(),
                handoff.qualifiers().len(),
                handoff.argument_groups().len(),
                handoff.arguments().len(),
            ],
            expected,
            "{id}"
        );
        for (total, count) in aggregate.iter_mut().zip(expected) {
            *total += count;
        }
        assert_eq!(source_type.applications().len(), 1, "{id}");
        assert_eq!(source_type.expressions().len(), 1, "{id}");
        assert!(source_type.arguments().is_empty(), "{id}");
        assert_eq!(
            first.typed_ast.source_type(),
            first.resolved.source_type(),
            "{id}"
        );
        assert_eq!(
            first.typed_ast.source_attribute(),
            first.resolved.source_attribute(),
            "{id}"
        );
        assert_eq!(
            first.typed_ast.debug_text(),
            second.typed_ast.debug_text(),
            "{id}"
        );
        assert_eq!(
            first.resolved.debug_text(),
            second.resolved.debug_text(),
            "{id}"
        );
        assert!(first
            .typed_ast
            .debug_text()
            .contains("source-attribute-debug-v1"));
        assert_eq!(first.binding_env.bindings().len(), 1, "{id}");
        assert_eq!(
            first.legacy_attribute_count,
            usize::from(preserves_legacy),
            "{id}"
        );
        assert_eq!(first.declarations.is_some(), preserves_legacy, "{id}");
        if let Some(declarations) = &first.declarations {
            assert_eq!(
                declarations
                    .diagnostics()
                    .canonical_iter()
                    .map(|(_, diagnostic)| diagnostic.message_key.as_str())
                    .collect::<Vec<_>>(),
                ["checker.declaration.deferred.evidence_query"],
                "{id}"
            );
        }

        let (_, attribute) = handoff
            .attributes()
            .iter()
            .next()
            .expect("one real Task 250 attribute");
        match attribute.polarity() {
            mizar_checker::source_attribute::SourceAttributePolarityInput::Positive => {
                polarities[0] += 1;
            }
            mizar_checker::source_attribute::SourceAttributePolarityInput::Negative { .. } => {
                polarities[1] += 1;
            }
            polarity => panic!("{id} has an unexpected polarity: {polarity:?}"),
        }
        if attribute.symbol().module() == &resolver.module {
            provenance[0] += 1;
        } else {
            provenance[1] += 1;
        }
        let (_, expression) = source_type
            .expressions()
            .iter()
            .next()
            .expect("one real Task 249 expression");
        match expression.head() {
            mizar_checker::source_type::SourceTypeHead::BuiltinSet => heads[0] += 1,
            mizar_checker::source_type::SourceTypeHead::Symbol { symbol, .. }
                if symbol.module() == &resolver.module
                    && symbols
                        .symbols()
                        .get(symbol)
                        .is_some_and(|entry| {
                            entry.kind() == mizar_resolve::env::SymbolKind::Structure
                        }) =>
            {
                heads[1] += 1;
            }
            head => panic!("{id} has an unexpected Task 249 head: {head:?}"),
        }
    }

    assert_eq!(aggregate, [4, 4, 1, 1, 1]);
    assert_eq!(polarities, [3, 1]);
    assert_eq!(provenance, [2, 2]);
    assert_eq!(heads, [3, 1]);
}

#[test]
fn task250_exact_selector_excludes_frozen_siblings() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 250 isolation plan should build");
    for id in [
        "fail_type_elaboration_imported_empty_positive_gap_001",
        "fail_type_elaboration_imported_empty_object_gap_001",
        "fail_type_elaboration_local_attribute_forward_reference_gap_001",
        "fail_type_elaboration_argument_bearing_mode_gap_001",
        "fail_type_elaboration_bracket_structure_argument_gap_001",
    ] {
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == id)
            .unwrap_or_else(|| panic!("{id} should remain active"));
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{id} frontend failed: {error}"));
        let ast = frontend.ast.unwrap_or_else(|| panic!("{id} AST"));
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        assert!(
            source_attribute_output(&ast, resolver.module, &symbols).is_none(),
            "{id} must not be captured by Task 250"
        );
    }
}

#[test]
fn task250_synthetic_surface_extractor_preserves_prefix_order_and_punctuation() {
    let source_id = source_id(250);
    let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("task250.synthetic"));
    let ast = task250_synthetic_attribute_ast(source_id);
    let symbols = task250_synthetic_attribute_symbols(source_id, module.clone());

    let first = synthetic_source_attribute_output(&ast, module.clone(), &symbols)
        .expect("synthetic Task 250 extractor should build");
    let second = synthetic_source_attribute_output(&ast, module, &symbols)
        .expect("synthetic Task 250 extractor should be deterministic");
    let source_type = first
        .typed_ast
        .source_type()
        .expect("synthetic Task 250 should install Task 249");
    let handoff = first
        .typed_ast
        .source_attribute()
        .expect("synthetic Task 250 should install its handoff");
    assert_eq!(
        (
            source_type.applications().len(),
            source_type.expressions().len(),
            source_type.arguments().len(),
        ),
        (1, 1, 0)
    );
    assert_eq!(
        (
            handoff.chains().len(),
            handoff.attributes().len(),
            handoff.qualifiers().len(),
            handoff.argument_groups().len(),
            handoff.arguments().len(),
        ),
        (1, 2, 0, 2, 3)
    );
    assert_eq!(
        handoff
            .attributes()
            .iter()
            .map(|(_, attribute)| (attribute.ordinal(), attribute.target_spelling()))
            .collect::<Vec<_>>(),
        [(0, "ranked"), (1, "graded")]
    );
    let groups = handoff
        .argument_groups()
        .iter()
        .map(|(_, group)| group)
        .collect::<Vec<_>>();
    assert_eq!(groups[0].kind(), mizar_checker::source_attribute::SourceAttributeArgumentGroupKind::Prefix);
    assert_eq!(groups[0].prefix_form(), Some(mizar_checker::source_attribute::SourceAttributePrefixForm::Single));
    assert_eq!(groups[0].hyphen_spelling(), Some("-"));
    assert_eq!(groups[0].hyphen_range(), Some(range(source_id, 26, 27)));
    assert!(groups[0].open_range().is_none());
    assert!(groups[0].comma_ranges().is_empty());
    assert_eq!(groups[1].kind(), mizar_checker::source_attribute::SourceAttributeArgumentGroupKind::Prefix);
    assert_eq!(groups[1].prefix_form(), Some(mizar_checker::source_attribute::SourceAttributePrefixForm::Parenthesized));
    assert_eq!(groups[1].open_spelling(), Some("("));
    assert_eq!(groups[1].close_spelling(), Some(")"));
    assert_eq!(groups[1].hyphen_spelling(), Some("-"));
    assert_eq!(groups[1].comma_spellings(), [","]);
    assert_eq!(groups[1].open_range(), Some(range(source_id, 35, 36)));
    assert_eq!(groups[1].close_range(), Some(range(source_id, 43, 44)));
    assert_eq!(groups[1].hyphen_range(), Some(range(source_id, 45, 46)));
    assert_eq!(groups[1].comma_ranges(), [range(source_id, 39, 40)]);
    assert_eq!(
        handoff
            .arguments()
            .iter()
            .map(|(_, actual)| (actual.kind(), actual.spelling()))
            .collect::<Vec<_>>(),
        [
            (
                mizar_checker::source_attribute::SourceAttributeActualKind::PrefixIdentifier,
                "p"
            ),
            (
                mizar_checker::source_attribute::SourceAttributeActualKind::PrefixIdentifier,
                "q"
            ),
            (
                mizar_checker::source_attribute::SourceAttributeActualKind::PrefixNumeral,
                "2"
            ),
        ]
    );
    for (id, actual) in handoff.arguments().iter() {
        assert_eq!(
            actual.provenance().structural_path(),
            [actual.group().index() as u32, actual.ordinal() as u32],
            "actual {}",
            id.index()
        );
    }
    assert_eq!(first.typed_ast.source_attribute(), first.resolved.source_attribute());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    assert_eq!(first.legacy_attribute_count, 0);
    assert!(first.declarations.is_none());
}

#[test]
fn task250_synthetic_extractor_fails_closed_on_recovery_and_symbol_corruption() {
    let source_id = source_id(251);
    let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("task250.recovery"));
    let recovered = task250_synthetic_attribute_ast_with_recovery(source_id, true);
    let symbols = task250_synthetic_attribute_symbols(source_id, module.clone());
    assert!(
        synthetic_source_attribute_output(&recovered, module.clone(), &symbols)
            .expect_err("recovered prefix must not publish Task 250")
            .contains("recovered")
    );

    let exact = task250_synthetic_attribute_ast(source_id);
    let corrupt_symbols = task250_synthetic_attribute_symbols_with_kinds(
        source_id,
        module.clone(),
        [SymbolKind::Attribute, SymbolKind::Mode],
    );
    assert!(
        synthetic_source_attribute_output(&exact, module, &corrupt_symbols)
            .expect_err("wrong-role attribute must not publish Task 250")
            .contains("not uniquely visible")
    );
}

fn task250_synthetic_attribute_ast(source_id: SourceId) -> SurfaceAst {
    task250_synthetic_attribute_ast_with_recovery(source_id, false)
}

fn task250_synthetic_attribute_ast_with_recovery(
    source_id: SourceId,
    recover_first_hyphen: bool,
) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let mut offset = 10;
    let reserve_keyword = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::ReservedWord,
        "reserve",
    );
    let binding = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::Identifier,
        "x",
    );
    let for_keyword = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::ReservedWord,
        "for",
    );
    let type_start = offset;

    let prefix_one_start = offset;
    let p = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::Identifier,
        "p",
    );
    let first_hyphen = if recover_first_hyphen {
        add_recovered_token(
            &mut builder,
            source_id,
            &mut offset,
            SurfaceTokenKind::ReservedSymbol,
            "-",
        )
    } else {
        add_token(
            &mut builder,
            source_id,
            &mut offset,
            SurfaceTokenKind::ReservedSymbol,
            "-",
        )
    };
    let prefix_one = builder.add_node(
        SurfaceNodeKind::ParameterPrefix,
        range(
            source_id,
            prefix_one_start,
            builder.node_range(first_hyphen).unwrap().end,
        ),
        vec![p, first_hyphen],
    );
    let ranked = task250_synthetic_attribute_symbol(
        &mut builder,
        source_id,
        &mut offset,
        "ranked",
    );
    let first_attribute = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(
            source_id,
            prefix_one_start,
            builder.node_range(ranked).unwrap().end,
        ),
        vec![prefix_one, ranked],
    );

    let prefix_two_start = offset;
    let open = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let q = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::Identifier,
        "q",
    );
    let comma = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::ReservedSymbol,
        ",",
    );
    let two = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::Numeral,
        "2",
    );
    let close = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let second_hyphen = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::ReservedSymbol,
        "-",
    );
    let prefix_two = builder.add_node(
        SurfaceNodeKind::ParameterPrefix,
        range(
            source_id,
            prefix_two_start,
            builder.node_range(second_hyphen).unwrap().end,
        ),
        vec![open, q, comma, two, close, second_hyphen],
    );
    let graded = task250_synthetic_attribute_symbol(
        &mut builder,
        source_id,
        &mut offset,
        "graded",
    );
    let second_attribute = builder.add_node(
        SurfaceNodeKind::AttributeRef,
        range(
            source_id,
            prefix_two_start,
            builder.node_range(graded).unwrap().end,
        ),
        vec![prefix_two, graded],
    );
    let chain = builder.add_node(
        SurfaceNodeKind::AttributeChain,
        range(
            source_id,
            type_start,
            builder.node_range(second_attribute).unwrap().end,
        ),
        vec![first_attribute, second_attribute],
    );
    let set = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::ReservedWord,
        "set",
    );
    let type_end = builder.node_range(set).unwrap().end;
    let head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        builder.node_range(set).unwrap(),
        vec![set],
    );
    let type_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, type_start, type_end),
        vec![chain, head],
    );
    let segment = builder.add_node(
        SurfaceNodeKind::ReserveSegment,
        range(
            source_id,
            builder.node_range(binding).unwrap().start,
            type_end,
        ),
        vec![binding, for_keyword, type_expression],
    );
    let semicolon = add_token(
        &mut builder,
        source_id,
        &mut offset,
        SurfaceTokenKind::ReservedSymbol,
        ";",
    );
    let reserve = builder.add_node(
        SurfaceNodeKind::ReserveItem,
        range(
            source_id,
            builder.node_range(reserve_keyword).unwrap().start,
            builder.node_range(semicolon).unwrap().end,
        ),
        vec![reserve_keyword, segment, semicolon],
    );
    finish_compilation_ast(builder, source_id, vec![reserve])
}

fn task250_synthetic_attribute_symbol(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    offset: &mut usize,
    spelling: &str,
) -> SurfaceBuilderNodeId {
    let start = *offset;
    let token = add_token(
        builder,
        source_id,
        offset,
        SurfaceTokenKind::UserSymbol,
        spelling,
    );
    let range = range(source_id, start, start + spelling.len());
    let segment = builder.add_node(SurfaceNodeKind::PathSegment, range, vec![token]);
    builder.add_node(SurfaceNodeKind::QualifiedSymbol, range, vec![segment])
}

fn task250_synthetic_attribute_symbols(
    source_id: SourceId,
    module: ResolverModuleId,
) -> SymbolEnv {
    task250_synthetic_attribute_symbols_with_kinds(
        source_id,
        module,
        [SymbolKind::Attribute, SymbolKind::Attribute],
    )
}

fn task250_synthetic_attribute_symbols_with_kinds(
    source_id: SourceId,
    module: ResolverModuleId,
    kinds: [SymbolKind; 2],
) -> SymbolEnv {
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id },
        SourceAnchor::Range(range(source_id, 0, 1)),
    );
    for (ordinal, (spelling, kind)) in ["ranked", "graded"].into_iter().zip(kinds).enumerate() {
        let symbol = ResolverSymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("Attribute/{spelling}/{ordinal}")),
            FullyQualifiedName::new(format!(
                "{}::{spelling}/{ordinal}",
                module.path().as_str()
            )),
        );
        indexes.symbols.insert(
            SymbolEntry::new(
                symbol.clone(),
                kind,
                NamespacePath::new(module.path().as_str()),
                spelling,
                SemanticOrigin::new(
                    source_id,
                    module.clone(),
                    SourceAnchor::Range(range(source_id, ordinal, ordinal + 1)),
                    vec![ordinal as u32],
                ),
                contribution,
            )
            .with_visibility(Visibility::Public)
            .with_export_status(ExportStatus::Exported),
        );
        indexes.contributions.add_symbol(contribution, symbol);
    }
    SymbolEnv::new(module, indexes)
}
