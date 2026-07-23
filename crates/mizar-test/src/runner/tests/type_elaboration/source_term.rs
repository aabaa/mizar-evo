use super::{source_term_output, source_term_output_with_mutation, synthetic_source_term_output};

#[test]
fn task252_real_routes_publish_exact_handoff_and_preserve_final_ownership() {
    let cases = [
        (
            "fail_type_elaboration_term_formula_gap_001",
            (2, 0, 2),
            [0, 0, 0, 2, 0],
            0,
        ),
        (
            "pass_type_elaboration_reserved_variable_equality_001",
            (2, 2, 0),
            [2, 0, 0, 0, 0],
            0,
        ),
        (
            "pass_type_elaboration_parenthesized_reserved_variable_equality_001",
            (3, 2, 0),
            [2, 0, 0, 0, 1],
            1,
        ),
    ];
    let mut totals = [0_usize; 3];
    let mut kinds = [0_usize; 5];

    for (id, expected_counts, expected_kinds, expected_parent_edges) in cases {
        let (ast, module, symbols) = task252_real_ast(id);
        let first = source_term_output(&ast, module.clone(), &symbols)
            .unwrap_or_else(|| panic!("{id} should select Task 252"))
            .unwrap_or_else(|error| panic!("{id} Task 252 failed: {error}"));
        let second = source_term_output(&ast, module, &symbols)
            .unwrap_or_else(|| panic!("{id} should remain selected"))
            .unwrap_or_else(|error| panic!("{id} repeated Task 252 failed: {error}"));
        let handoff = first
            .typed_ast
            .source_term()
            .expect("Task 252 handoff should be installed");
        let actual_counts = (
            handoff.terms().len(),
            handoff.references().len(),
            handoff.numeric_type_requests().len(),
        );
        assert_eq!(actual_counts, expected_counts, "{id}");
        for (total, count) in
            totals
                .iter_mut()
                .zip([actual_counts.0, actual_counts.1, actual_counts.2])
        {
            *total += count;
        }

        let mut actual_kinds = [0_usize; 5];
        for (term_id, term) in handoff.terms().iter() {
            assert_eq!(term.source_ordinal(), term_id.index(), "{id}");
            match term.kind() {
                mizar_checker::source_term::SourcePrimaryTermKind::VariableReference => {
                    actual_kinds[0] += 1;
                }
                mizar_checker::source_term::SourcePrimaryTermKind::ConstantReference => {
                    actual_kinds[1] += 1;
                }
                mizar_checker::source_term::SourcePrimaryTermKind::It => {
                    actual_kinds[2] += 1;
                }
                mizar_checker::source_term::SourcePrimaryTermKind::Numeral => {
                    actual_kinds[3] += 1;
                }
                mizar_checker::source_term::SourcePrimaryTermKind::Parenthesized => {
                    actual_kinds[4] += 1;
                }
                kind => panic!("{id} emitted an unexpected Task 252 kind: {kind:?}"),
            }
        }
        assert_eq!(actual_kinds, expected_kinds, "{id}");
        assert_eq!(
            handoff
                .terms()
                .iter()
                .filter(|(_, term)| term.parent().is_some())
                .count(),
            expected_parent_edges,
            "{id}"
        );
        for (_, reference) in handoff.references().iter() {
            assert_eq!(
                reference.binding(),
                mizar_checker::binding_env::BindingId::new(0),
                "{id}"
            );
            assert_eq!(reference.use_ordinal(), 1, "{id}");
            assert_eq!(
                reference.role(),
                mizar_checker::source_term::SourcePrimaryTermReferenceRole::Variable,
                "{id}"
            );
        }
        for (total, count) in kinds.iter_mut().zip(actual_kinds) {
            *total += count;
        }
        assert_eq!(
            first.typed_ast.source_term(),
            first.resolved.source_term(),
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
        assert!(
            first
                .typed_ast
                .debug_text()
                .contains("source-primary-term-debug-v1")
        );
    }

    assert_eq!(totals, [7, 4, 2]);
    assert_eq!(kinds, [4, 0, 0, 2, 1]);

    let (ast, module, symbols) =
        task252_real_ast("pass_type_elaboration_reserved_variable_equality_001");
    let error = source_term_output_with_mutation(&ast, module.clone(), &symbols, |input| {
        input.terms[0].spelling = "wrong".to_owned();
    })
    .expect("bare route should remain selected")
    .expect_err("corrupt source-term input must fail atomically");
    assert!(error.contains("term") || error.contains("reference"));
    assert!(
        source_term_output(&ast, module, &symbols)
            .expect("uncorrupted route should remain selected")
            .is_ok()
    );
}

#[test]
fn task252_exact_selector_excludes_all_frozen_siblings() {
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
    let plan = build_test_plan(&config).expect("Task 252 isolation plan should build");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if source_term_output(&ast, resolver.module, &symbols).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(
        selected,
        [
            "fail_type_elaboration_term_formula_gap_001",
            "pass_type_elaboration_parenthesized_reserved_variable_equality_001",
            "pass_type_elaboration_reserved_variable_equality_001",
        ]
    );
}

#[test]
fn task252_synthetic_constant_and_it_use_the_public_producer() {
    let source_id = source_id(252);
    let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("source.term"));
    let ast = task252_constant_and_it_ast(source_id);
    let scope = mizar_resolve::names::LocalTermScope::new(vec![7]);
    let binding_env = task252_binding_env(
        source_id,
        module.clone(),
        Some((
            "c",
            mizar_checker::binding_env::BindingKind::LocalAbbreviation,
            Some(scope.clone()),
        )),
    );

    let output = synthetic_source_term_output(&ast, module, binding_env)
        .expect("synthetic constant/it transport should build");
    let handoff = output
        .typed_ast
        .source_term()
        .expect("synthetic handoff should be installed");
    assert_eq!(
        handoff
            .terms()
            .iter()
            .map(|(_, term)| (term.kind(), term.role(), term.spelling().to_owned()))
            .collect::<Vec<_>>(),
        [
            (
                mizar_checker::source_term::SourcePrimaryTermKind::ConstantReference,
                mizar_checker::source_term::SourcePrimaryTermRole::Value,
                "c".to_owned(),
            ),
            (
                mizar_checker::source_term::SourcePrimaryTermKind::It,
                mizar_checker::source_term::SourcePrimaryTermRole::CurrentDefinitionResult,
                "it".to_owned(),
            ),
        ]
    );
    let reference = handoff
        .references()
        .get(mizar_checker::source_term::SourcePrimaryTermReferenceId::new(0))
        .expect("constant reference");
    assert_eq!(
        reference.role(),
        mizar_checker::source_term::SourcePrimaryTermReferenceRole::LocalConstant
    );
    assert_eq!(reference.lexical_scope(), Some(&scope));
    assert_eq!(reference.use_ordinal(), 1);
    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
}

#[test]
fn task252_nested_parentheses_exclude_mixed_subtrees_and_keep_siblings() {
    let source_id = source_id(253);
    let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("source.nested"));
    let ast = task252_nested_and_mixed_ast(source_id);
    let binding_env = task252_binding_env(source_id, module.clone(), None);

    let first = synthetic_source_term_output(&ast, module.clone(), binding_env.clone())
        .expect("nested/sibling source-term transport should build");
    let second = synthetic_source_term_output(&ast, module, binding_env)
        .expect("nested/sibling source-term replay should build");
    let handoff = first
        .typed_ast
        .source_term()
        .expect("nested handoff should be installed");
    assert_eq!(
        handoff
            .terms()
            .iter()
            .map(|(_, term)| (
                term.kind(),
                term.spelling().to_owned(),
                term.parent().map(|parent| parent.index()),
            ))
            .collect::<Vec<_>>(),
        [
            (
                mizar_checker::source_term::SourcePrimaryTermKind::Parenthesized,
                "( ( 7 ) )".to_owned(),
                None,
            ),
            (
                mizar_checker::source_term::SourcePrimaryTermKind::Parenthesized,
                "( 7 )".to_owned(),
                Some(0),
            ),
            (
                mizar_checker::source_term::SourcePrimaryTermKind::Numeral,
                "7".to_owned(),
                Some(1),
            ),
            (
                mizar_checker::source_term::SourcePrimaryTermKind::It,
                "it".to_owned(),
                None,
            ),
        ]
    );
    assert_eq!(handoff.numeric_type_requests().len(), 1);
    assert!(
        handoff
            .terms()
            .iter()
            .all(|(_, term)| !term.spelling().contains('8'))
    );
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

fn task252_real_ast(id: &str) -> (SurfaceAst, ResolverModuleId, SymbolEnv) {
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
    let plan = build_test_plan(&config).expect("Task 252 repository plan should build");
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
    let module = resolver.module;
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
    (ast, module, symbols)
}

fn task252_binding_env(
    source_id: SourceId,
    module: ResolverModuleId,
    binding: Option<(
        &str,
        mizar_checker::binding_env::BindingKind,
        Option<mizar_resolve::names::LocalTermScope>,
    )>,
) -> mizar_checker::binding_env::BindingEnv {
    let mut bindings = mizar_checker::binding_env::BindingTable::new();
    let binding_id = binding.as_ref().map(|(spelling, kind, scope)| {
        let declaration_range = range(source_id, 1, 2);
        bindings.insert(mizar_checker::binding_env::BindingDraft {
            spelling: (*spelling).to_owned(),
            kind: *kind,
            identity: scope.as_ref().map_or_else(
                || mizar_checker::binding_env::BinderIdentity::ReservedVariable {
                    spelling: (*spelling).to_owned(),
                    declaration_range,
                },
                |scope| mizar_checker::binding_env::BinderIdentity::ResolverLocal {
                    scope: scope.clone(),
                    ordinal: 0,
                    declaration_range,
                },
            ),
            owner_context: mizar_checker::binding_env::BindingContextId::new(0),
            declaration_range,
            visible_after_ordinal: 0,
            type_site: mizar_checker::binding_env::BindingTypeSite::Missing,
            status: mizar_checker::binding_env::BindingStatus::Active,
            captured: mizar_checker::binding_env::CapturedFreeVariables::default(),
            diagnostics: Vec::new(),
            recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
        })
    });
    let lexical_scope = binding.and_then(|(_, _, scope)| scope);
    let ids = binding_id.into_iter().collect::<Vec<_>>();
    let mut contexts = mizar_checker::binding_env::BindingContextTable::new();
    let context = contexts.insert(mizar_checker::binding_env::BindingContextDraft {
        owner: mizar_checker::binding_env::BindingContextOwner::Module,
        parent: None,
        layer: mizar_checker::binding_env::BindingContextLayer::Module,
        lexical_scope,
        bindings: ids.clone(),
        visible_bindings: ids,
        recovery: mizar_checker::binding_env::BindingContextRecovery::Normal,
    });
    assert_eq!(
        context,
        mizar_checker::binding_env::BindingContextId::new(0)
    );
    mizar_checker::binding_env::BindingEnv::try_new(mizar_checker::binding_env::BindingEnvParts {
        source_id,
        module_id: module,
        contexts,
        bindings,
        diagnostics: mizar_checker::binding_env::BindingDiagnosticTable::new(),
    })
    .expect("synthetic Task 252 binding environment")
}

fn task252_constant_and_it_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let constant_token =
        builder.add_token(SurfaceTokenKind::Identifier, "c", range(source_id, 10, 11));
    let constant = builder.add_node(
        SurfaceNodeKind::TermReference,
        range(source_id, 10, 11),
        vec![constant_token],
    );
    let constant_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 10, 11),
        vec![constant],
    );
    let it_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "it",
        range(source_id, 20, 22),
    );
    let it = builder.add_node(
        SurfaceNodeKind::ItTerm,
        range(source_id, 20, 22),
        vec![it_token],
    );
    let it_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 20, 22),
        vec![it],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 30),
        vec![constant_expression, it_expression],
    );
    builder.finish(Some(root), None)
}

fn task252_nested_and_mixed_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);

    let seven_token = builder.add_token(SurfaceTokenKind::Numeral, "7", range(source_id, 14, 15));
    let seven = builder.add_node(
        SurfaceNodeKind::NumeralTerm,
        range(source_id, 14, 15),
        vec![seven_token],
    );
    let seven_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 14, 15),
        vec![seven],
    );
    let inner_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 12, 13),
    );
    let inner_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 16, 17),
    );
    let inner_parenthesized = builder.add_node(
        SurfaceNodeKind::ParenthesizedTerm,
        range(source_id, 12, 17),
        vec![inner_open, seven_expression, inner_close],
    );
    let inner_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 12, 17),
        vec![inner_parenthesized],
    );
    let outer_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 10, 11),
    );
    let outer_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 18, 19),
    );
    let outer_parenthesized = builder.add_node(
        SurfaceNodeKind::ParenthesizedTerm,
        range(source_id, 10, 19),
        vec![outer_open, inner_expression, outer_close],
    );
    let nested_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 10, 19),
        vec![outer_parenthesized],
    );

    let eight_token = builder.add_token(SurfaceTokenKind::Numeral, "8", range(source_id, 34, 35));
    let eight = builder.add_node(
        SurfaceNodeKind::NumeralTerm,
        range(source_id, 34, 35),
        vec![eight_token],
    );
    let argument = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 34, 35),
        vec![eight],
    );
    let functor = builder.add_token(SurfaceTokenKind::Identifier, "f", range(source_id, 32, 33));
    let application = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        range(source_id, 32, 35),
        vec![functor, argument],
    );
    let application_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 32, 35),
        vec![application],
    );
    let mixed_open = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "(",
        range(source_id, 30, 31),
    );
    let mixed_close = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ")",
        range(source_id, 36, 37),
    );
    let mixed_parenthesized = builder.add_node(
        SurfaceNodeKind::ParenthesizedTerm,
        range(source_id, 30, 37),
        vec![mixed_open, application_expression, mixed_close],
    );
    let mixed_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 30, 37),
        vec![mixed_parenthesized],
    );

    let it_token = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "it",
        range(source_id, 40, 42),
    );
    let it = builder.add_node(
        SurfaceNodeKind::ItTerm,
        range(source_id, 40, 42),
        vec![it_token],
    );
    let it_expression = builder.add_node(
        SurfaceNodeKind::TermExpression,
        range(source_id, 40, 42),
        vec![it],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 50),
        vec![nested_expression, mixed_expression, it_expression],
    );
    builder.finish(Some(root), None)
}
