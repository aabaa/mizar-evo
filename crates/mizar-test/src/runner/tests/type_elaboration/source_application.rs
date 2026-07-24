use super::{
    SyntheticSourceFunctorApplication, SyntheticSourceFunctorArgument,
    SyntheticSourceFunctorHead, source_application_output,
    source_application_output_with_mutation, synthetic_functional_actual_count,
    synthetic_source_application_output,
};

#[test]
fn task253_real_routes_publish_exact_aggregate_and_preserve_final_ownership() {
    let cases = [
        (
            "fail_type_elaboration_imported_predicate_functor_gap_001",
            (1, 1, 1, 2, 2),
            (2, 0, 2),
        ),
        (
            "fail_type_elaboration_local_functor_application_gap_001",
            (1, 0, 1, 1, 2),
            (1, 1, 0),
        ),
    ];
    let mut application_totals = [0_usize; 5];
    let mut primary_totals = [0_usize; 3];

    for (id, expected_application, expected_primary) in cases {
        let (ast, module, shells, symbols) = task253_real_ast(id);
        let first = source_application_output(&ast, module.clone(), &shells, &symbols)
            .unwrap_or_else(|| panic!("{id} should select Task 253"))
            .unwrap_or_else(|error| panic!("{id} Task 253 failed: {error}"));
        let second = source_application_output(&ast, module, &shells, &symbols)
            .unwrap_or_else(|| panic!("{id} should remain selected"))
            .unwrap_or_else(|error| panic!("{id} repeated Task 253 failed: {error}"));
        let applications = first
            .typed_ast
            .source_application()
            .expect("Task 253 handoff should be installed");
        let primary = first
            .typed_ast
            .source_term()
            .expect("Task 252 dependency should be installed first");
        let actual_application = (
            applications.applications().len(),
            applications.wrappers().len(),
            applications.candidates().len(),
            applications.arguments().len(),
            applications.type_requests().len(),
        );
        let actual_primary = (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        );
        assert_eq!(actual_application, expected_application, "{id}");
        assert_eq!(actual_primary, expected_primary, "{id}");
        for (total, count) in application_totals.iter_mut().zip([
            actual_application.0,
            actual_application.1,
            actual_application.2,
            actual_application.3,
            actual_application.4,
        ]) {
            *total += count;
        }
        for (total, count) in
            primary_totals
                .iter_mut()
                .zip([actual_primary.0, actual_primary.1, actual_primary.2])
        {
            *total += count;
        }
        let application = applications
            .applications()
            .get(mizar_checker::source_application::SourceFunctorApplicationId::new(0))
            .expect("one real application");
        assert_eq!(
            application.kind(),
            mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic
        );
        assert_eq!(application.source_ordinal(), 0);
        assert_eq!(
            applications
                .arguments()
                .iter()
                .map(|(_, argument)| (argument.ordinal(), argument.target()))
                .collect::<Vec<_>>(),
            (0..expected_application.3)
                .map(|ordinal| (
                    ordinal,
                    mizar_checker::source_application::SourceFunctorArgumentTarget::Primary(
                        mizar_checker::source_term::SourcePrimaryTermId::new(ordinal),
                    ),
                ))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            applications
                .type_requests()
                .iter()
                .map(|(_, request)| (request.request_ordinal(), request.kind()))
                .collect::<Vec<_>>(),
            [
                (
                    0,
                    mizar_checker::source_application::SourceFunctorTypeRequestKind::CandidateSignature,
                ),
                (
                    1,
                    mizar_checker::source_application::SourceFunctorTypeRequestKind::ApplicationResultType,
                ),
            ]
        );
        assert_eq!(
            first.typed_ast.source_application(),
            first.resolved.source_application(),
            "{id}"
        );
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
        assert_eq!(
            applications.primary_term_fingerprint(),
            primary.debug_text(),
            "{id}"
        );
    }

    assert_eq!(application_totals, [2, 1, 2, 3, 4]);
    assert_eq!(primary_totals, [3, 1, 2]);
}

#[test]
fn task253_real_wrapper_candidate_and_local_binding_coordinates_are_exact() {
    let (ast, module, shells, symbols) =
        task253_real_ast("fail_type_elaboration_imported_predicate_functor_gap_001");
    let imported = source_application_output(&ast, module, &shells, &symbols)
        .expect("imported selector")
        .unwrap_or_else(|error| panic!("imported Task 253 failed: {error}"));
    let handoff = imported.typed_ast.source_application().expect("handoff");
    let application = handoff
        .applications()
        .get(mizar_checker::source_application::SourceFunctorApplicationId::new(0))
        .expect("application");
    assert_eq!(
        application.form(),
        mizar_checker::source_application::SourceFunctorApplicationForm::Infix
    );
    assert_eq!(application.spelling(), "1 ++ 2");
    let wrapper = handoff
        .wrappers()
        .get(mizar_checker::source_application::SourceFunctorWrapperId::new(0))
        .expect("Task 253 wrapper");
    assert_eq!(wrapper.spelling(), "( 1 ++ 2 )");
    assert!(
        imported
            .typed_ast
            .source_term()
            .expect("primary terms")
            .terms()
            .iter()
        .all(|(_, term)| term.kind()
                != mizar_checker::source_term::SourcePrimaryTermKind::Parenthesized)
    );
    let imported_candidate = handoff
        .candidates()
        .get(mizar_checker::source_application::SourceFunctorCandidateId::new(0))
        .expect("imported candidate");
    assert_eq!(imported_candidate.signature(), None);
    assert_eq!(
        imported_candidate.visibility(),
        mizar_resolve::env::Visibility::Public
    );
    assert!(matches!(
        imported_candidate.export_status(),
        mizar_resolve::env::ExportStatus::Exported
            | mizar_resolve::env::ExportStatus::ReExported
    ));

    let (ast, module, shells, symbols) =
        task253_real_ast("fail_type_elaboration_local_functor_application_gap_001");
    let local = source_application_output(&ast, module, &shells, &symbols)
        .expect("local selector")
        .unwrap_or_else(|error| panic!("local Task 253 failed: {error}"));
    let handoff = local.typed_ast.source_application().expect("handoff");
    let application = handoff
        .applications()
        .get(mizar_checker::source_application::SourceFunctorApplicationId::new(0))
        .expect("application");
    assert_eq!(
        application.form(),
        mizar_checker::source_application::SourceFunctorApplicationForm::Functional
    );
    assert_eq!(
        application.context(),
        mizar_checker::binding_env::BindingContextId::new(1)
    );
    let reference = local
        .typed_ast
        .source_term()
        .expect("primary")
        .references()
        .get(mizar_checker::source_term::SourcePrimaryTermReferenceId::new(0))
        .expect("local actual reference");
    assert_eq!(
        reference.binding(),
        mizar_checker::binding_env::BindingId::new(1)
    );
    assert_eq!(reference.use_ordinal(), 2);
    assert_eq!(
        reference.lexical_scope(),
        Some(&mizar_resolve::names::LocalTermScope::new(vec![1]))
    );
    assert!(matches!(
        handoff
            .candidates()
            .get(mizar_checker::source_application::SourceFunctorCandidateId::new(0))
            .expect("local candidate")
            .signature(),
        Some(mizar_resolve::env::SignatureShell::Opaque { .. })
    ));
}

#[test]
fn task253_real_input_corruption_fails_atomically() {
    for id in [
        "fail_type_elaboration_imported_predicate_functor_gap_001",
        "fail_type_elaboration_local_functor_application_gap_001",
    ] {
        let (ast, module, shells, symbols) = task253_real_ast(id);
        let error = source_application_output_with_mutation(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            |input| {
                input.arguments.pop();
            },
        )
        .unwrap_or_else(|| panic!("{id} selector"))
        .expect_err("partial application arguments must fail atomically");
        assert!(
            error.contains("argument") || error.contains("application"),
            "{id}: {error}"
        );
        assert!(
            source_application_output(&ast, module, &shells, &symbols)
                .expect("uncorrupted selector")
                .is_ok(),
            "{id}"
        );
    }

    task253_imported_private_selector_corruption_matrix();
    task253_local_private_selector_corruption_matrix();
}

#[test]
fn task253_exact_selector_admits_only_two_frozen_real_consumers() {
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
    let plan = build_test_plan(&config).expect("Task 253 isolation plan should build");
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
        if source_application_output(&ast, resolver.module, &resolver.shells, &symbols).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(
        selected,
        [
            "fail_type_elaboration_imported_predicate_functor_gap_001",
            "fail_type_elaboration_local_functor_application_gap_001",
        ]
    );
}

fn task253_real_ast(
    id: &str,
) -> (
    SurfaceAst,
    ResolverModuleId,
    mizar_resolve::declarations::DeclarationShellSet,
    SymbolEnv,
) {
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
    let plan = build_test_plan(&config).expect("Task 253 repository plan should build");
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
    let shells = resolver.shells;
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
    (ast, module, shells, symbols)
}

fn task253_imported_private_selector_corruption_matrix() {
    let (real_ast, module, _, symbols) =
        task253_real_ast("fail_type_elaboration_imported_predicate_functor_gap_001");
    let source = real_ast.source_id;
    let spec = exact_imported_predicate_functor_theorem_spec();
    let exact = imported_predicate_functor_theorem_ast(
        source,
        &["parser.type_fixtures"],
        spec,
    );
    let shells =
        mizar_resolve::declarations::DeclarationShellCollector::new(&exact, &module).collect();
    let exact_output = source_application_output(&exact, module.clone(), &shells, &symbols)
        .expect("exact imported selector");
    assert!(exact_output.is_ok(), "{exact_output:?}");

    for (label, gap_symbols) in [
        (
            "missing imported functor provenance",
            SymbolEnv::new(module.clone(), SymbolEnvIndexes::default()),
        ),
        (
            "wrong imported functor kind",
            imported_predicate_wrong_functor_kind_env(module.clone()),
        ),
        (
            "duplicate imported functor provenance",
            ambiguous_imported_predicate_functor_env(module.clone(), "++"),
        ),
    ] {
        assert!(
            source_application_output(&exact, module.clone(), &shells, &gap_symbols).is_none(),
            "{label}"
        );
    }

    let wrong_head = ImportedPredicateFunctorTheoremSpec {
        functor: "--",
        ..spec
    };
    let wrong_numeral_order = ImportedPredicateFunctorTheoremSpec {
        functor_left: "2",
        functor_right: "1",
        ..spec
    };
    let raw_corruptions = [
        (
            "wrong ++ head",
            imported_predicate_functor_theorem_ast(
                source,
                &["parser.type_fixtures"],
                wrong_head,
            ),
        ),
        (
            "wrong infix form shell",
            imported_predicate_functor_theorem_ast_with_corruption(
                source,
                &["parser.type_fixtures"],
                spec,
                ImportedPredicateFunctorTheoremCorruption {
                    extra_inner_expression_child: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
        ),
        (
            "wrong infix arity",
            imported_predicate_functor_theorem_ast_with_corruption(
                source,
                &["parser.type_fixtures"],
                spec,
                ImportedPredicateFunctorTheoremCorruption {
                    extra_infix_operand: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
        ),
        (
            "wrong numeral order",
            imported_predicate_functor_theorem_ast(
                source,
                &["parser.type_fixtures"],
                wrong_numeral_order,
            ),
        ),
        (
            "recovered functor head",
            imported_predicate_functor_theorem_ast_with_corruption(
                source,
                &["parser.type_fixtures"],
                spec,
                ImportedPredicateFunctorTheoremCorruption {
                    recovered_functor: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
        ),
        (
            "extra wrapper child",
            imported_predicate_functor_theorem_ast_with_corruption(
                source,
                &["parser.type_fixtures"],
                spec,
                ImportedPredicateFunctorTheoremCorruption {
                    extra_parenthesized_child: true,
                    ..ImportedPredicateFunctorTheoremCorruption::default()
                },
            ),
        ),
    ];
    for (label, corrupt) in raw_corruptions {
        let corrupt_shells =
            mizar_resolve::declarations::DeclarationShellCollector::new(&corrupt, &module)
                .collect();
        assert!(
            source_application_output(
                &corrupt,
                module.clone(),
                &corrupt_shells,
                &symbols
            )
            .is_none(),
            "{label}"
        );
    }
}

fn task253_local_private_selector_corruption_matrix() {
    const EXACT: &str = "reserve x for set;\n\
\n\
definition\n\
  let x be set;\n\
  func Task253LocalSourceDef:\n\
    task253_local_source(x) -> set equals x;\n\
  func Task253LocalConsumerDef:\n\
    task253_local_consumer(x) -> set\n\
    equals task253_local_source(x);\n\
end;\n";
    let (ast, module, shells, symbols) = task253_ast_from_source_text(EXACT, 0);
    assert!(
        source_application_output(&ast, module, &shells, &symbols)
            .expect("exact local selector")
            .is_ok()
    );

    let reversed_definitions = "reserve x for set;\n\
\n\
definition\n\
  let x be set;\n\
  func Task253LocalConsumerDef:\n\
    task253_local_consumer(x) -> set\n\
    equals task253_local_source(x);\n\
  func Task253LocalSourceDef:\n\
    task253_local_source(x) -> set equals x;\n\
end;\n"
        .to_owned();
    let corruptions = [
        ("reversed definition order and forward use", reversed_definitions),
        (
            "wrong application head",
            EXACT.replace(
                "equals task253_local_source(x);",
                "equals task253_local_consumer(x);",
            ),
        ),
        (
            "wrong actual",
            EXACT.replace(
                "equals task253_local_source(x);",
                "equals task253_local_source(y);",
            ),
        ),
        (
            "extra actual",
            EXACT.replace(
                "equals task253_local_source(x);",
                "equals task253_local_source(x,x);",
            ),
        ),
        (
            "extra application",
            EXACT.replace("-> set equals x;", "-> set equals task253_local_source(x);"),
        ),
        (
            "extra compilation item",
            format!("reserve y for set;\n{EXACT}"),
        ),
        (
            "recovered local application",
            EXACT.replace(
                "equals task253_local_source(x);",
                "equals task253_local_source(x;",
            ),
        ),
        (
            "outer BindingId(0) instead of definition parameter",
            EXACT.replace("let x be set;", "let y be set;"),
        ),
    ];
    for (ordinal, (label, source)) in corruptions.into_iter().enumerate() {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(&source, ordinal + 1);
        assert!(
            source_application_output(&ast, module, &shells, &symbols).is_none(),
            "{label}"
        );
    }
}

fn task253_ast_from_source_text(
    source: &str,
    ordinal: usize,
) -> (
    SurfaceAst,
    ResolverModuleId,
    mizar_resolve::declarations::DeclarationShellSet,
    SymbolEnv,
) {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let package_root = std::env::temp_dir().join(format!(
        "mizar-test-task253-selector-{}-{ordinal}-{unique}",
        std::process::id()
    ));
    let source_path = package_root.join("src").join("task253_local.miz");
    std::fs::create_dir_all(source_path.parent().expect("source parent"))
        .expect("create Task 253 selector package");
    std::fs::write(&source_path, source).expect("write Task 253 selector source");
    let package = PackageId::new("mizar-test-task253-corruption");
    let module_path = ModulePath::new(format!("tests.task253_local_corruption_{ordinal}"));
    let normalized_path =
        mizar_session::normalize_path(&package_root, &source_path).expect("normalize source path");
    let frontend = mizar_frontend::orchestration::Frontend::new(
        mizar_frontend::source::FrontendSourceLoader::new(
            mizar_session::DiskSourceLoader::new(&package_root),
        ),
        ParseOnlyImportProvider,
        mizar_frontend::parsing::MizarParserSeam,
    );
    let output = frontend
        .run(
            mizar_frontend::source::SourceUnitRequest {
                snapshot: super::shared::snapshot_id(10_000 + ordinal),
                input: mizar_session::SourceInput {
                    package_id: package.clone(),
                    module_path: module_path.clone(),
                    normalized_path,
                    edition: Edition::new("2026"),
                    origin: mizar_session::SourceOriginInput::Disk {
                        path: source_path,
                    },
                },
            },
            &InMemorySessionIdAllocator::new(),
        )
        .expect("Task 253 selector frontend should run");
    std::fs::remove_dir_all(&package_root).expect("clean Task 253 selector package");
    let ast = output.ast.expect("Task 253 selector AST");
    let module = ResolverModuleId::new(package, module_path);
    let shells =
        mizar_resolve::declarations::DeclarationShellCollector::new(&ast, &module).collect();
    let projections = mizar_resolve::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        NamespacePath::new(module.path().as_str()),
    )
    .extract();
    let symbols =
        mizar_resolve::symbols::SymbolCollector::new(ast.source_id, &module, &shells, &projections)
            .collect()
            .into_env();
    (ast, module, shells, symbols)
}

#[derive(Debug, Clone, Copy)]
enum Task253RawSiteKind {
    Token,
    TermExpression,
    TermReference,
    Parenthesized,
    Application,
    Prefix,
    Postfix,
}

#[derive(Debug, Clone, Copy)]
struct Task253RawSite {
    range: SourceRange,
    kind: Task253RawSiteKind,
}

#[derive(Debug, Clone)]
enum Task253RawArgument {
    Primary(Task253RawSite),
    Application(usize),
}

#[derive(Debug, Clone)]
struct Task253RawApplication {
    application: Task253RawSite,
    form: mizar_checker::source_application::SourceFunctorApplicationForm,
    kind: mizar_checker::source_application::SourceFunctorApplicationKind,
    head: Vec<Task253RawSite>,
    wrappers: Vec<Task253RawSite>,
    arguments: Vec<Task253RawArgument>,
    candidate_indexes: Vec<usize>,
    degraded: bool,
}

#[test]
fn task253_synthetic_ordinary_form_matrix_uses_private_extractor_and_public_producer() {
    let source = source_id(254);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("task253.synthetic.forms"),
    );
    let mut builder = SurfaceAstBuilder::new(source);
    let mut roots = Vec::new();
    let mut raw = Vec::new();

    let (bare_head, bare_head_site) = task253_head(&mut builder, source, 100, "c");
    let bare_range = range(source, 100, 101);
    let bare = builder.add_node(
        SurfaceNodeKind::TermExpression,
        bare_range,
        vec![bare_head],
    );
    roots.push(bare);
    raw.push(Task253RawApplication {
        application: Task253RawSite {
            range: bare_range,
            kind: Task253RawSiteKind::TermExpression,
        },
        form: mizar_checker::source_application::SourceFunctorApplicationForm::Bare,
        kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
        head: vec![bare_head_site],
        wrappers: Vec::new(),
        arguments: Vec::new(),
        candidate_indexes: vec![0],
        degraded: false,
    });

    let (prefix_head, prefix_head_site) = task253_head(&mut builder, source, 110, "pre");
    let (prefix_actual, prefix_actual_site) =
        task253_reference(&mut builder, source, 114, "x");
    let prefix_range = range(source, 110, 115);
    let prefix = builder.add_node(
        SurfaceNodeKind::PrefixExpression(mizar_syntax::SurfacePrefixOperator {
            spelling: "pre".into(),
            precedence: 10,
        }),
        prefix_range,
        vec![prefix_head, prefix_actual],
    );
    roots.push(prefix);
    raw.push(Task253RawApplication {
        application: Task253RawSite {
            range: prefix_range,
            kind: Task253RawSiteKind::Prefix,
        },
        form: mizar_checker::source_application::SourceFunctorApplicationForm::Prefix,
        kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
        head: vec![prefix_head_site],
        wrappers: Vec::new(),
        arguments: vec![Task253RawArgument::Primary(prefix_actual_site)],
        candidate_indexes: vec![1],
        degraded: false,
    });

    let (postfix_actual, postfix_actual_site) =
        task253_reference(&mut builder, source, 120, "x");
    let (postfix_head, postfix_head_site) = task253_token_site(
        &mut builder,
        source,
        122,
        SurfaceTokenKind::UserSymbol,
        "!",
    );
    let postfix_range = range(source, 120, 123);
    let postfix = builder.add_node(
        SurfaceNodeKind::PostfixExpression(mizar_syntax::SurfacePostfixOperator {
            spelling: "!".into(),
            precedence: 10,
        }),
        postfix_range,
        vec![postfix_actual, postfix_head],
    );
    roots.push(postfix);
    raw.push(Task253RawApplication {
        application: Task253RawSite {
            range: postfix_range,
            kind: Task253RawSiteKind::Postfix,
        },
        form: mizar_checker::source_application::SourceFunctorApplicationForm::Postfix,
        kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
        head: vec![postfix_head_site],
        wrappers: Vec::new(),
        arguments: vec![Task253RawArgument::Primary(postfix_actual_site)],
        candidate_indexes: vec![2],
        degraded: false,
    });

    let (left, left_site) = task253_token_site(
        &mut builder,
        source,
        130,
        SurfaceTokenKind::UserSymbol,
        "[:",
    );
    let (bracket_x, bracket_x_site) = task253_reference(&mut builder, source, 133, "x");
    let (comma, _) = task253_token_site(
        &mut builder,
        source,
        135,
        SurfaceTokenKind::ReservedSymbol,
        ",",
    );
    let (bracket_y, bracket_y_site) = task253_reference(&mut builder, source, 137, "y");
    let (right, right_site) = task253_token_site(
        &mut builder,
        source,
        139,
        SurfaceTokenKind::UserSymbol,
        ":]",
    );
    let bracket_range = range(source, 130, 141);
    let bracket = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        bracket_range,
        vec![left, bracket_x, comma, bracket_y, right],
    );
    roots.push(bracket);
    raw.push(Task253RawApplication {
        application: Task253RawSite {
            range: bracket_range,
            kind: Task253RawSiteKind::Application,
        },
        form: mizar_checker::source_application::SourceFunctorApplicationForm::Bracket,
        kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
        head: vec![left_site, right_site],
        wrappers: Vec::new(),
        arguments: vec![
            Task253RawArgument::Primary(bracket_x_site),
            Task253RawArgument::Primary(bracket_y_site),
        ],
        candidate_indexes: vec![3],
        degraded: false,
    });

    let (functional_head, functional_head_site) =
        task253_head(&mut builder, source, 150, "fun");
    let (open, _) = task253_token_site(
        &mut builder,
        source,
        154,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (functional_x, functional_x_site) =
        task253_reference(&mut builder, source, 156, "x");
    let (comma_one, _) = task253_token_site(
        &mut builder,
        source,
        158,
        SurfaceTokenKind::ReservedSymbol,
        ",",
    );
    let (functional_y, functional_y_site) =
        task253_reference(&mut builder, source, 160, "y");
    let (comma_two, _) = task253_token_site(
        &mut builder,
        source,
        162,
        SurfaceTokenKind::ReservedSymbol,
        ",",
    );
    let (functional_z, functional_z_site) =
        task253_reference(&mut builder, source, 164, "z");
    let (close, _) = task253_token_site(
        &mut builder,
        source,
        166,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let functional_range = range(source, 150, 167);
    let functional = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        functional_range,
        vec![
            functional_head,
            open,
            functional_x,
            comma_one,
            functional_y,
            comma_two,
            functional_z,
            close,
        ],
    );
    roots.push(functional);
    raw.push(Task253RawApplication {
        application: Task253RawSite {
            range: functional_range,
            kind: Task253RawSiteKind::Application,
        },
        form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
        kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
        head: vec![functional_head_site],
        wrappers: Vec::new(),
        arguments: vec![
            Task253RawArgument::Primary(functional_x_site),
            Task253RawArgument::Primary(functional_y_site),
            Task253RawArgument::Primary(functional_z_site),
        ],
        candidate_indexes: vec![4],
        degraded: false,
    });

    let ast = task253_finish_ast(builder, source, roots);
    let bindings = task253_synthetic_binding_env(source, module.clone());
    let (symbols, candidates) =
        task253_synthetic_symbols(source, module.clone(), ["c", "pre", "!", "[:", "fun"]);
    let probes = task253_probes(&ast, &raw, &candidates);
    let first = synthetic_source_application_output(
        &ast,
        module.clone(),
        bindings.clone(),
        &symbols,
        &probes,
    )
    .expect("ordinary synthetic matrix");
    let second =
        synthetic_source_application_output(&ast, module, bindings, &symbols, &probes)
            .expect("ordinary synthetic deterministic replay");
    assert_eq!(task253_counts(&first.typed_ast), (5, 0, 5, 7, 10));
    assert_eq!(
        first
            .typed_ast
            .source_term()
            .expect("Task 252")
            .terms()
            .len(),
        7
    );
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn task253_synthetic_inline_zero_one_two_are_raw_shapes_with_injected_kind_only() {
    let source = source_id(255);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("task253.synthetic.inline"),
    );
    let mut builder = SurfaceAstBuilder::new(source);
    let mut roots = Vec::new();
    let mut raw = Vec::new();
    for (ordinal, actual_count) in [0_usize, 1, 2].into_iter().enumerate() {
        let start = 100 + ordinal * 30;
        let (head, head_site) =
            task253_head(&mut builder, source, start, &format!("inline{ordinal}"));
        let head_end = start + format!("inline{ordinal}").len();
        let (open, _) = task253_token_site(
            &mut builder,
            source,
            head_end + 1,
            SurfaceTokenKind::ReservedSymbol,
            "(",
        );
        let mut children = vec![head, open];
        let mut arguments = Vec::new();
        let mut cursor = head_end + 3;
        for actual in 0..actual_count {
            let spelling = if actual == 0 { "x" } else { "y" };
            let (node, site) = task253_reference(&mut builder, source, cursor, spelling);
            cursor += spelling.len();
            children.push(node);
            arguments.push(Task253RawArgument::Primary(site));
            if actual + 1 < actual_count {
                let (comma, _) = task253_token_site(
                    &mut builder,
                    source,
                    cursor + 1,
                    SurfaceTokenKind::ReservedSymbol,
                    ",",
                );
                children.push(comma);
                cursor += 3;
            }
        }
        let (close, close_site) = task253_token_site(
            &mut builder,
            source,
            cursor + 1,
            SurfaceTokenKind::ReservedSymbol,
            ")",
        );
        children.push(close);
        let application_range = range(source, start, close_site.range.end);
        let application =
            builder.add_node(SurfaceNodeKind::ApplicationTerm, application_range, children);
        roots.push(application);
        raw.push(Task253RawApplication {
            application: Task253RawSite {
                range: application_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Inline,
            head: vec![head_site],
            wrappers: Vec::new(),
            arguments,
            candidate_indexes: Vec::new(),
            degraded: false,
        });
    }
    let ast = task253_finish_ast(builder, source, roots);
    let probes = task253_probes(&ast, &raw, &[]);
    assert_eq!(
        probes
            .iter()
            .map(|probe| synthetic_functional_actual_count(&ast, probe.application).unwrap())
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let bindings = task253_synthetic_binding_env(source, module.clone());
    let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
    let output = synthetic_source_application_output(&ast, module, bindings, &symbols, &probes)
        .expect("inline shape DTO seam");
    assert_eq!(task253_counts(&output.typed_ast), (3, 0, 0, 3, 0));
    assert_eq!(
        output
            .typed_ast
            .source_term()
            .expect("Task 252")
            .terms()
            .len(),
        3
    );

    let malformed_source = source_id(6);
    let mut malformed_builder = SurfaceAstBuilder::new(malformed_source);
    let (head, _) = task253_head(&mut malformed_builder, malformed_source, 10, "bad");
    let (open, _) = task253_token_site(
        &mut malformed_builder,
        malformed_source,
        14,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let malformed_range = range(malformed_source, 10, 15);
    let malformed = malformed_builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        malformed_range,
        vec![head, open],
    );
    let malformed_ast =
        task253_finish_ast(malformed_builder, malformed_source, vec![malformed]);
    let malformed_index = task253_node_index(
        &malformed_ast,
        Task253RawSite {
            range: malformed_range,
            kind: Task253RawSiteKind::Application,
        },
    );
    assert!(
        synthetic_functional_actual_count(&malformed_ast, malformed_index)
            .expect_err("missing close delimiter")
            .contains("parentheses")
    );
}

#[test]
fn task253_synthetic_nested_application_uses_cross_family_edges_and_exact_task252_slice() {
    let source = source_id(0);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("task253.synthetic.nested"),
    );
    let mut builder = SurfaceAstBuilder::new(source);
    let (outer_head, outer_head_site) = task253_head(&mut builder, source, 100, "f");
    let (outer_open, _) = task253_token_site(
        &mut builder,
        source,
        102,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (inner_head, inner_head_site) = task253_head(&mut builder, source, 104, "g");
    let (inner_open, _) = task253_token_site(
        &mut builder,
        source,
        106,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (one, one_site) = task253_numeral(&mut builder, source, 108, "1");
    let (inner_close, _) = task253_token_site(
        &mut builder,
        source,
        110,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let inner_range = range(source, 104, 111);
    let inner = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        inner_range,
        vec![inner_head, inner_open, one, inner_close],
    );
    let (comma, _) = task253_token_site(
        &mut builder,
        source,
        112,
        SurfaceTokenKind::ReservedSymbol,
        ",",
    );
    let (x, x_site) = task253_reference(&mut builder, source, 114, "x");
    let (outer_close, _) = task253_token_site(
        &mut builder,
        source,
        116,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let outer_range = range(source, 100, 117);
    let outer = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        outer_range,
        vec![
            outer_head,
            outer_open,
            inner,
            comma,
            x,
            outer_close,
        ],
    );
    let ast = task253_finish_ast(builder, source, vec![outer]);
    let raw = vec![
        Task253RawApplication {
            application: Task253RawSite {
                range: outer_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
            head: vec![outer_head_site],
            wrappers: Vec::new(),
            arguments: vec![
                Task253RawArgument::Application(1),
                Task253RawArgument::Primary(x_site),
            ],
            candidate_indexes: vec![0],
            degraded: false,
        },
        Task253RawApplication {
            application: Task253RawSite {
                range: inner_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
            head: vec![inner_head_site],
            wrappers: Vec::new(),
            arguments: vec![Task253RawArgument::Primary(one_site)],
            candidate_indexes: vec![1],
            degraded: false,
        },
    ];
    let bindings = task253_synthetic_definition_parameter_binding_env(source, module.clone());
    let parameter = mizar_checker::binding_env::BindingId::new(0);
    assert_eq!(
        bindings
            .bindings()
            .get(parameter)
            .expect("synthetic definition parameter")
            .kind,
        mizar_checker::binding_env::BindingKind::DefinitionParameter
    );
    let (symbols, candidates) =
        task253_synthetic_symbols(source, module.clone(), ["f", "g"]);
    let probes = task253_probes(&ast, &raw, &candidates);
    let output = synthetic_source_application_output(&ast, module, bindings, &symbols, &probes)
        .expect("nested f(g(1),x)");
    assert_eq!(task253_counts(&output.typed_ast), (2, 0, 2, 3, 4));
    let primary = output.typed_ast.source_term().expect("Task252");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (2, 1, 1)
    );
    let parameter_reference = primary
        .references()
        .get(mizar_checker::source_term::SourcePrimaryTermReferenceId::new(0))
        .expect("nested x definition-parameter reference");
    assert_eq!(parameter_reference.binding(), parameter);
    assert_eq!(parameter_reference.use_ordinal(), 1);
    assert_eq!(
        parameter_reference.lexical_scope(),
        Some(&mizar_resolve::names::LocalTermScope::new(vec![1]))
    );
    assert_eq!(
        output
            .typed_ast
            .source_application()
            .expect("Task253")
            .arguments()
            .iter()
            .map(|(_, argument)| argument.target())
            .collect::<Vec<_>>(),
        [
            mizar_checker::source_application::SourceFunctorArgumentTarget::Application(
                mizar_checker::source_application::SourceFunctorApplicationId::new(1),
            ),
            mizar_checker::source_application::SourceFunctorArgumentTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(1),
            ),
            mizar_checker::source_application::SourceFunctorArgumentTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(0),
            ),
        ]
    );
}

#[test]
fn task253_synthetic_primary_parentheses_target_only_the_task252_root() {
    let source = source_id(1);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("task253.synthetic.primary-parentheses"),
    );
    let mut builder = SurfaceAstBuilder::new(source);
    let (head, head_site) = task253_head(&mut builder, source, 100, "f");
    let (open, _) = task253_token_site(
        &mut builder,
        source,
        102,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (primary_open, _) = task253_token_site(
        &mut builder,
        source,
        104,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (x, _) = task253_reference(&mut builder, source, 106, "x");
    let (primary_close, _) = task253_token_site(
        &mut builder,
        source,
        108,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let primary_range = range(source, 104, 109);
    let primary = builder.add_node(
        SurfaceNodeKind::ParenthesizedTerm,
        primary_range,
        vec![primary_open, x, primary_close],
    );
    let (close, _) = task253_token_site(
        &mut builder,
        source,
        110,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let application_range = range(source, 100, 111);
    let application = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        application_range,
        vec![head, open, primary, close],
    );
    let ast = task253_finish_ast(builder, source, vec![application]);
    let raw = [Task253RawApplication {
        application: Task253RawSite {
            range: application_range,
            kind: Task253RawSiteKind::Application,
        },
        form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
        kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
        head: vec![head_site],
        wrappers: Vec::new(),
        arguments: vec![Task253RawArgument::Primary(Task253RawSite {
            range: primary_range,
            kind: Task253RawSiteKind::Parenthesized,
        })],
        candidate_indexes: vec![0],
        degraded: false,
    }];
    let bindings = task253_synthetic_binding_env(source, module.clone());
    let (symbols, candidates) = task253_synthetic_symbols(source, module.clone(), ["f"]);
    let probes = task253_probes(&ast, &raw, &candidates);
    let output = synthetic_source_application_output(&ast, module, bindings, &symbols, &probes)
        .expect("primary-only parentheses");
    assert_eq!(task253_counts(&output.typed_ast), (1, 0, 1, 1, 2));
    let primary = output.typed_ast.source_term().expect("Task252");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (2, 1, 0)
    );
    let argument = output
        .typed_ast
        .source_application()
        .expect("Task253")
        .arguments()
        .get(mizar_checker::source_application::SourceFunctorArgumentId::new(0))
        .expect("argument");
    assert_eq!(
        argument.target(),
        mizar_checker::source_application::SourceFunctorArgumentTarget::Primary(
            mizar_checker::source_term::SourcePrimaryTermId::new(0),
        )
    );
}

#[test]
fn task253_synthetic_application_wrappers_are_outer_to_inner_and_degraded_is_positive() {
    let wrapped = task253_single_wrapped_output(source_id(2), 2, false);
    assert_eq!(task253_counts(&wrapped.typed_ast), (1, 2, 1, 1, 2));
    assert_eq!(
        wrapped
            .typed_ast
            .source_application()
            .expect("Task253")
            .wrappers()
            .iter()
            .map(|(_, wrapper)| (wrapper.ordinal(), wrapper.spelling()))
            .collect::<Vec<_>>(),
        [(0, "( ( f ( x ) ) )"), (1, "( f ( x ) )")]
    );
    let degraded = task253_single_wrapped_output(source_id(3), 1, true);
    assert_eq!(task253_counts(&degraded.typed_ast), (1, 1, 1, 1, 2));
    let handoff = degraded.typed_ast.source_application().expect("Task253");
    assert_eq!(
        handoff
            .applications()
            .get(mizar_checker::source_application::SourceFunctorApplicationId::new(0))
            .expect("degraded application")
            .recovery(),
        mizar_checker::source_application::SourceFunctorApplicationRecovery::Degraded
    );
    assert_eq!(
        handoff
            .wrappers()
            .get(mizar_checker::source_application::SourceFunctorWrapperId::new(0))
            .expect("degraded wrapper")
            .recovery(),
        mizar_checker::source_application::SourceFunctorApplicationRecovery::Degraded
    );
}

#[test]
fn task253_synthetic_nested_wrapper_is_owned_by_the_inner_application() {
    let source = source_id(4);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("task253.synthetic.nested-wrapper"),
    );
    let mut builder = SurfaceAstBuilder::new(source);
    let (outer_head, outer_head_site) = task253_head(&mut builder, source, 100, "f");
    let (outer_open, _) = task253_token_site(
        &mut builder,
        source,
        102,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (wrapper_open, _) = task253_token_site(
        &mut builder,
        source,
        104,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (inner_head, inner_head_site) = task253_head(&mut builder, source, 106, "g");
    let (inner_open, _) = task253_token_site(
        &mut builder,
        source,
        108,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (x, x_site) = task253_reference(&mut builder, source, 110, "x");
    let (inner_close, _) = task253_token_site(
        &mut builder,
        source,
        112,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let inner_range = range(source, 106, 113);
    let inner = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        inner_range,
        vec![inner_head, inner_open, x, inner_close],
    );
    let (wrapper_close, _) = task253_token_site(
        &mut builder,
        source,
        114,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let wrapper_range = range(source, 104, 115);
    let wrapper = builder.add_node(
        SurfaceNodeKind::ParenthesizedTerm,
        wrapper_range,
        vec![wrapper_open, inner, wrapper_close],
    );
    let (outer_close, _) = task253_token_site(
        &mut builder,
        source,
        116,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let outer_range = range(source, 100, 117);
    let outer = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        outer_range,
        vec![outer_head, outer_open, wrapper, outer_close],
    );
    let ast = task253_finish_ast(builder, source, vec![outer]);
    let raw = [
        Task253RawApplication {
            application: Task253RawSite {
                range: outer_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
            head: vec![outer_head_site],
            wrappers: Vec::new(),
            arguments: vec![Task253RawArgument::Application(1)],
            candidate_indexes: vec![0],
            degraded: false,
        },
        Task253RawApplication {
            application: Task253RawSite {
                range: inner_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
            head: vec![inner_head_site],
            wrappers: vec![Task253RawSite {
                range: wrapper_range,
                kind: Task253RawSiteKind::Parenthesized,
            }],
            arguments: vec![Task253RawArgument::Primary(x_site)],
            candidate_indexes: vec![1],
            degraded: false,
        },
    ];
    let bindings = task253_synthetic_binding_env(source, module.clone());
    let (symbols, candidates) =
        task253_synthetic_symbols(source, module.clone(), ["f", "g"]);
    let probes = task253_probes(&ast, &raw, &candidates);
    let output = synthetic_source_application_output(&ast, module, bindings, &symbols, &probes)
        .expect("nested wrapped application");
    assert_eq!(task253_counts(&output.typed_ast), (2, 1, 2, 2, 4));
    assert_eq!(
        (
            output
                .typed_ast
                .source_term()
                .expect("Task252")
                .terms()
                .len(),
            output
                .typed_ast
                .source_term()
                .expect("Task252")
                .references()
                .len(),
        ),
        (1, 1)
    );
    assert_eq!(
        output
            .typed_ast
            .source_application()
            .expect("Task253")
            .wrappers()
            .get(mizar_checker::source_application::SourceFunctorWrapperId::new(0))
            .expect("inner wrapper")
            .application(),
        mizar_checker::source_application::SourceFunctorApplicationId::new(1)
    );
}

#[test]
fn task253_synthetic_candidate_subset_and_template_sibling_exclusion_are_exact() {
    let source = source_id(5);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("task253.synthetic.isolation"),
    );
    let mut builder = SurfaceAstBuilder::new(source);
    let (head, head_site) = task253_head(&mut builder, source, 100, "f");
    let (open, _) = task253_token_site(
        &mut builder,
        source,
        102,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (x, x_site) = task253_reference(&mut builder, source, 104, "x");
    let (close, _) = task253_token_site(
        &mut builder,
        source,
        106,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let eligible_range = range(source, 100, 107);
    let eligible = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        eligible_range,
        vec![head, open, x, close],
    );

    let (excluded_head, excluded_head_site) =
        task253_head(&mut builder, source, 120, "template_f");
    let (excluded_open, _) = task253_token_site(
        &mut builder,
        source,
        130,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (excluded_x, excluded_x_site) =
        task253_reference(&mut builder, source, 132, "x");
    let (excluded_close, _) = task253_token_site(
        &mut builder,
        source,
        134,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let excluded_range = range(source, 120, 135);
    let excluded = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        excluded_range,
        vec![excluded_head, excluded_open, excluded_x, excluded_close],
    );
    let template_range = range(source, 118, 137);
    let template = builder.add_node(
        SurfaceNodeKind::TemplateArgument,
        template_range,
        vec![excluded],
    );

    let (mixed_head, mixed_head_site) =
        task253_head(&mut builder, source, 140, "mixed_f");
    let (mixed_open, _) = task253_token_site(
        &mut builder,
        source,
        148,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (structure_token, _) = task253_token_site(
        &mut builder,
        source,
        150,
        SurfaceTokenKind::UserSymbol,
        "Struct",
    );
    let structure_range = range(source, 150, 156);
    let structure = builder.add_node(
        SurfaceNodeKind::StructureConstructor,
        structure_range,
        vec![structure_token],
    );
    let mixed_actual = builder.add_node(
        SurfaceNodeKind::TermExpression,
        structure_range,
        vec![structure],
    );
    let (mixed_close, _) = task253_token_site(
        &mut builder,
        source,
        158,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let mixed_range = range(source, 140, 159);
    let mixed = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        mixed_range,
        vec![mixed_head, mixed_open, mixed_actual, mixed_close],
    );
    let ast = task253_finish_ast(builder, source, vec![eligible, template, mixed]);
    let raw = [
        Task253RawApplication {
            application: Task253RawSite {
                range: excluded_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
            head: vec![excluded_head_site],
            wrappers: Vec::new(),
            arguments: vec![Task253RawArgument::Primary(excluded_x_site)],
            candidate_indexes: vec![1],
            degraded: false,
        },
        Task253RawApplication {
            application: Task253RawSite {
                range: mixed_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
            head: vec![mixed_head_site],
            wrappers: Vec::new(),
            arguments: vec![Task253RawArgument::Primary(Task253RawSite {
                range: structure_range,
                kind: Task253RawSiteKind::TermExpression,
            })],
            candidate_indexes: vec![3],
            degraded: false,
        },
        Task253RawApplication {
            application: Task253RawSite {
                range: eligible_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
            head: vec![head_site],
            wrappers: Vec::new(),
            arguments: vec![Task253RawArgument::Primary(x_site)],
            candidate_indexes: vec![0, 2],
            degraded: false,
        },
    ];
    let bindings = task253_synthetic_binding_env(source, module.clone());
    let (symbols, candidates) = task253_synthetic_symbols(
        source,
        module.clone(),
        ["f", "f", "f", "mixed_f"],
    );
    let probes = task253_probes(&ast, &raw, &candidates);
    let output = synthetic_source_application_output(&ast, module, bindings, &symbols, &probes)
        .expect("eligible sibling only");
    assert_eq!(task253_counts(&output.typed_ast), (1, 0, 2, 1, 3));
    assert_eq!(
        output
            .typed_ast
            .source_application()
            .expect("Task253")
            .candidates()
            .iter()
            .map(|(_, candidate)| candidate.symbol().clone())
            .collect::<Vec<_>>(),
        [candidates[0].0.clone(), candidates[2].0.clone()]
    );
}

#[test]
fn task253_synthetic_unowned_argument_excludes_the_complete_nested_subtree() {
    let source = source_id(7);
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new("task253.synthetic.unowned-subtree"),
    );
    let mut builder = SurfaceAstBuilder::new(source);

    let (outer_head, outer_head_site) = task253_head(&mut builder, source, 100, "f");
    let (outer_open, _) = task253_token_site(
        &mut builder,
        source,
        102,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (inner_head, inner_head_site) = task253_head(&mut builder, source, 104, "g");
    let (inner_open, _) = task253_token_site(
        &mut builder,
        source,
        106,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (inner_x, inner_x_site) = task253_reference(&mut builder, source, 108, "x");
    let (inner_close, _) = task253_token_site(
        &mut builder,
        source,
        110,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let inner_range = range(source, 104, 111);
    let inner = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        inner_range,
        vec![inner_head, inner_open, inner_x, inner_close],
    );
    let (comma, _) = task253_token_site(
        &mut builder,
        source,
        112,
        SurfaceTokenKind::ReservedSymbol,
        ",",
    );
    let (set_open, _) = task253_token_site(
        &mut builder,
        source,
        114,
        SurfaceTokenKind::ReservedSymbol,
        "{",
    );
    let (set_y, _) = task253_reference(&mut builder, source, 116, "y");
    let (set_close, _) = task253_token_site(
        &mut builder,
        source,
        118,
        SurfaceTokenKind::ReservedSymbol,
        "}",
    );
    let set_range = range(source, 114, 119);
    let set = builder.add_node(
        SurfaceNodeKind::SetEnumeration,
        set_range,
        vec![set_open, set_y, set_close],
    );
    let set_expression =
        builder.add_node(SurfaceNodeKind::TermExpression, set_range, vec![set]);
    let (outer_close, _) = task253_token_site(
        &mut builder,
        source,
        120,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let outer_range = range(source, 100, 121);
    let outer = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        outer_range,
        vec![
            outer_head,
            outer_open,
            inner,
            comma,
            set_expression,
            outer_close,
        ],
    );

    let (sibling_head, sibling_head_site) =
        task253_head(&mut builder, source, 130, "s");
    let (sibling_open, _) = task253_token_site(
        &mut builder,
        source,
        132,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (sibling_x, sibling_x_site) =
        task253_reference(&mut builder, source, 134, "x");
    let (sibling_close, _) = task253_token_site(
        &mut builder,
        source,
        136,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let sibling_range = range(source, 130, 137);
    let sibling = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        sibling_range,
        vec![sibling_head, sibling_open, sibling_x, sibling_close],
    );
    let ast = task253_finish_ast(builder, source, vec![outer, sibling]);
    let raw = [
        Task253RawApplication {
            application: Task253RawSite {
                range: outer_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
            head: vec![outer_head_site],
            wrappers: Vec::new(),
            arguments: vec![
                Task253RawArgument::Application(1),
                Task253RawArgument::Primary(Task253RawSite {
                    range: set_range,
                    kind: Task253RawSiteKind::TermExpression,
                }),
            ],
            candidate_indexes: vec![0],
            degraded: false,
        },
        Task253RawApplication {
            application: Task253RawSite {
                range: inner_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
            head: vec![inner_head_site],
            wrappers: Vec::new(),
            arguments: vec![Task253RawArgument::Primary(inner_x_site)],
            candidate_indexes: vec![1],
            degraded: false,
        },
        Task253RawApplication {
            application: Task253RawSite {
                range: sibling_range,
                kind: Task253RawSiteKind::Application,
            },
            form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
            kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
            head: vec![sibling_head_site],
            wrappers: Vec::new(),
            arguments: vec![Task253RawArgument::Primary(sibling_x_site)],
            candidate_indexes: vec![2],
            degraded: false,
        },
    ];
    let bindings = task253_synthetic_binding_env(source, module.clone());
    let (symbols, candidates) =
        task253_synthetic_symbols(source, module.clone(), ["f", "g", "s"]);
    let probes = task253_probes(&ast, &raw, &candidates);
    let output = synthetic_source_application_output(&ast, module, bindings, &symbols, &probes)
        .expect("unowned containing subtree should be omitted atomically");
    assert_eq!(task253_counts(&output.typed_ast), (1, 0, 1, 1, 2));
    assert_eq!(
        output
            .typed_ast
            .source_application()
            .expect("Task253")
            .applications()
            .get(mizar_checker::source_application::SourceFunctorApplicationId::new(0))
            .expect("eligible sibling")
            .spelling(),
        "s ( x )"
    );
}

fn task253_token_site(
    builder: &mut SurfaceAstBuilder,
    source: SourceId,
    start: usize,
    kind: SurfaceTokenKind,
    spelling: &str,
) -> (SurfaceBuilderNodeId, Task253RawSite) {
    let source_range = range(source, start, start + spelling.len());
    (
        builder.add_token(kind, spelling, source_range),
        Task253RawSite {
            range: source_range,
            kind: Task253RawSiteKind::Token,
        },
    )
}

fn task253_head(
    builder: &mut SurfaceAstBuilder,
    source: SourceId,
    start: usize,
    spelling: &str,
) -> (SurfaceBuilderNodeId, Task253RawSite) {
    let (token, token_site) = task253_token_site(
        builder,
        source,
        start,
        SurfaceTokenKind::UserSymbol,
        spelling,
    );
    let head = builder.add_node(
        SurfaceNodeKind::TermReference,
        token_site.range,
        vec![token],
    );
    (
        head,
        Task253RawSite {
            range: token_site.range,
            kind: Task253RawSiteKind::TermReference,
        },
    )
}

fn task253_reference(
    builder: &mut SurfaceAstBuilder,
    source: SourceId,
    start: usize,
    spelling: &str,
) -> (SurfaceBuilderNodeId, Task253RawSite) {
    let source_range = range(source, start, start + spelling.len());
    let token = builder.add_token(SurfaceTokenKind::Identifier, spelling, source_range);
    let reference =
        builder.add_node(SurfaceNodeKind::TermReference, source_range, vec![token]);
    (
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            source_range,
            vec![reference],
        ),
        Task253RawSite {
            range: source_range,
            kind: Task253RawSiteKind::TermExpression,
        },
    )
}

fn task253_numeral(
    builder: &mut SurfaceAstBuilder,
    source: SourceId,
    start: usize,
    spelling: &str,
) -> (SurfaceBuilderNodeId, Task253RawSite) {
    let source_range = range(source, start, start + spelling.len());
    let token = builder.add_token(SurfaceTokenKind::Numeral, spelling, source_range);
    let numeral = builder.add_node(SurfaceNodeKind::NumeralTerm, source_range, vec![token]);
    (
        builder.add_node(
            SurfaceNodeKind::TermExpression,
            source_range,
            vec![numeral],
        ),
        Task253RawSite {
            range: source_range,
            kind: Task253RawSiteKind::TermExpression,
        },
    )
}

fn task253_single_wrapped_output(
    source: SourceId,
    wrapper_count: usize,
    degraded: bool,
) -> super::SyntheticSourceApplicationOutput {
    assert!((1..=2).contains(&wrapper_count));
    let module = ResolverModuleId::new(
        PackageId::new("test"),
        ModulePath::new(format!(
            "task253.synthetic.wrapper.{wrapper_count}.{degraded}"
        )),
    );
    let mut builder = SurfaceAstBuilder::new(source);
    let (outer_open, _) = (wrapper_count == 2)
        .then(|| {
            task253_token_site(
                &mut builder,
                source,
                90,
                SurfaceTokenKind::ReservedSymbol,
                "(",
            )
        })
        .unzip();
    let (inner_open, _) = task253_token_site(
        &mut builder,
        source,
        92,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (head, head_site) = task253_head(&mut builder, source, 100, "f");
    let (open, _) = task253_token_site(
        &mut builder,
        source,
        102,
        SurfaceTokenKind::ReservedSymbol,
        "(",
    );
    let (x, x_site) = task253_reference(&mut builder, source, 104, "x");
    let (close, _) = task253_token_site(
        &mut builder,
        source,
        106,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let application_range = range(source, 100, 107);
    let application = builder.add_node(
        SurfaceNodeKind::ApplicationTerm,
        application_range,
        vec![head, open, x, close],
    );
    let (inner_close, _) = task253_token_site(
        &mut builder,
        source,
        108,
        SurfaceTokenKind::ReservedSymbol,
        ")",
    );
    let inner_range = range(source, 92, 109);
    let inner = builder.add_node(
        SurfaceNodeKind::ParenthesizedTerm,
        inner_range,
        vec![inner_open, application, inner_close],
    );
    let (root, wrapper_sites) = if let Some(outer_open) = outer_open {
        let (outer_close, _) = task253_token_site(
            &mut builder,
            source,
            110,
            SurfaceTokenKind::ReservedSymbol,
            ")",
        );
        let outer_range = range(source, 90, 111);
        (
            builder.add_node(
                SurfaceNodeKind::ParenthesizedTerm,
                outer_range,
                vec![outer_open, inner, outer_close],
            ),
            vec![
                Task253RawSite {
                    range: outer_range,
                    kind: Task253RawSiteKind::Parenthesized,
                },
                Task253RawSite {
                    range: inner_range,
                    kind: Task253RawSiteKind::Parenthesized,
                },
            ],
        )
    } else {
        (
            inner,
            vec![Task253RawSite {
                range: inner_range,
                kind: Task253RawSiteKind::Parenthesized,
            }],
        )
    };
    let ast = task253_finish_ast(builder, source, vec![root]);
    let raw = [Task253RawApplication {
        application: Task253RawSite {
            range: application_range,
            kind: Task253RawSiteKind::Application,
        },
        form: mizar_checker::source_application::SourceFunctorApplicationForm::Functional,
        kind: mizar_checker::source_application::SourceFunctorApplicationKind::Symbolic,
        head: vec![head_site],
        wrappers: wrapper_sites,
        arguments: vec![Task253RawArgument::Primary(x_site)],
        candidate_indexes: vec![0],
        degraded,
    }];
    let bindings = task253_synthetic_binding_env(source, module.clone());
    let (symbols, candidates) = task253_synthetic_symbols(source, module.clone(), ["f"]);
    let probes = task253_probes(&ast, &raw, &candidates);
    synthetic_source_application_output(&ast, module, bindings, &symbols, &probes)
        .expect("wrapped synthetic application")
}

fn task253_finish_ast(
    mut builder: SurfaceAstBuilder,
    source: SourceId,
    roots: Vec<SurfaceBuilderNodeId>,
) -> SurfaceAst {
    let end = roots
        .iter()
        .filter_map(|root| builder.node_range(*root))
        .map(|source_range| source_range.end)
        .max()
        .unwrap_or(1);
    let root = builder.add_node(SurfaceNodeKind::Root, range(source, 0, end + 1), roots);
    builder.finish(Some(root), None)
}

fn task253_node_index(ast: &SurfaceAst, site: Task253RawSite) -> usize {
    let matches_kind = |kind: &SurfaceNodeKind| match site.kind {
        Task253RawSiteKind::Token => matches!(kind, SurfaceNodeKind::Token(_)),
        Task253RawSiteKind::TermExpression => matches!(kind, SurfaceNodeKind::TermExpression),
        Task253RawSiteKind::TermReference => matches!(kind, SurfaceNodeKind::TermReference),
        Task253RawSiteKind::Parenthesized => matches!(kind, SurfaceNodeKind::ParenthesizedTerm),
        Task253RawSiteKind::Application => matches!(kind, SurfaceNodeKind::ApplicationTerm),
        Task253RawSiteKind::Prefix => matches!(kind, SurfaceNodeKind::PrefixExpression(_)),
        Task253RawSiteKind::Postfix => matches!(kind, SurfaceNodeKind::PostfixExpression(_)),
    };
    let matches = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| node.range == site.range && matches_kind(&node.kind))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [index] = matches.as_slice() else {
        panic!(
            "Task253 synthetic site {:?} {:?} should be unique, got {matches:?}",
            site.range, site.kind
        );
    };
    *index
}

fn task253_probes(
    ast: &SurfaceAst,
    raw: &[Task253RawApplication],
    candidates: &[(
        ResolverSymbolId,
        mizar_resolve::env::SourceContributionId,
    )],
) -> Vec<SyntheticSourceFunctorApplication> {
    raw.iter()
        .map(|application| SyntheticSourceFunctorApplication {
            application: task253_node_index(ast, application.application),
            form: application.form,
            kind: application.kind,
            head: match application.head.as_slice() {
                [head] => SyntheticSourceFunctorHead::Single(task253_node_index(ast, *head)),
                [left, right] => SyntheticSourceFunctorHead::Paired {
                    left: task253_node_index(ast, *left),
                    right: task253_node_index(ast, *right),
                },
                heads => panic!("Task253 synthetic head cardinality drift: {heads:?}"),
            },
            wrappers: application
                .wrappers
                .iter()
                .map(|wrapper| task253_node_index(ast, *wrapper))
                .collect(),
            arguments: application
                .arguments
                .iter()
                .map(|argument| match argument {
                    Task253RawArgument::Primary(root) => {
                        SyntheticSourceFunctorArgument::Primary(task253_node_index(ast, *root))
                    }
                    Task253RawArgument::Application(application) => {
                        SyntheticSourceFunctorArgument::Application(*application)
                    }
                })
                .collect(),
            candidates: application
                .candidate_indexes
                .iter()
                .map(|index| candidates[*index].clone())
                .collect(),
            degraded: application.degraded,
        })
        .collect()
}

fn task253_synthetic_binding_env(
    source: SourceId,
    module: ResolverModuleId,
) -> mizar_checker::binding_env::BindingEnv {
    let mut bindings = mizar_checker::binding_env::BindingTable::new();
    let ids = ["x", "y", "z"]
        .into_iter()
        .enumerate()
        .map(|(ordinal, spelling)| {
            let declaration_range = range(source, 1 + ordinal * 2, 2 + ordinal * 2);
            bindings.insert(mizar_checker::binding_env::BindingDraft {
                spelling: spelling.to_owned(),
                kind: mizar_checker::binding_env::BindingKind::ReservedVariable,
                identity:
                    mizar_checker::binding_env::BinderIdentity::ReservedVariable {
                        spelling: spelling.to_owned(),
                        declaration_range,
                    },
                owner_context: mizar_checker::binding_env::BindingContextId::new(0),
                declaration_range,
                visible_after_ordinal: ordinal,
                type_site: mizar_checker::binding_env::BindingTypeSite::Missing,
                status: mizar_checker::binding_env::BindingStatus::Active,
                captured: mizar_checker::binding_env::CapturedFreeVariables::default(),
                diagnostics: Vec::new(),
                recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
            })
        })
        .collect::<Vec<_>>();
    let mut contexts = mizar_checker::binding_env::BindingContextTable::new();
    contexts.insert(mizar_checker::binding_env::BindingContextDraft {
        owner: mizar_checker::binding_env::BindingContextOwner::Module,
        parent: None,
        layer: mizar_checker::binding_env::BindingContextLayer::Module,
        lexical_scope: None,
        bindings: ids.clone(),
        visible_bindings: ids,
        recovery: mizar_checker::binding_env::BindingContextRecovery::Normal,
    });
    mizar_checker::binding_env::BindingEnv::try_new(
        mizar_checker::binding_env::BindingEnvParts {
            source_id: source,
            module_id: module,
            contexts,
            bindings,
            diagnostics: mizar_checker::binding_env::BindingDiagnosticTable::new(),
        },
    )
    .expect("Task253 synthetic binding environment")
}

fn task253_synthetic_definition_parameter_binding_env(
    source: SourceId,
    module: ResolverModuleId,
) -> mizar_checker::binding_env::BindingEnv {
    let declaration_range = range(source, 1, 2);
    let scope = mizar_resolve::names::LocalTermScope::new(vec![1]);
    let mut bindings = mizar_checker::binding_env::BindingTable::new();
    let parameter = bindings.insert(mizar_checker::binding_env::BindingDraft {
        spelling: "x".to_owned(),
        kind: mizar_checker::binding_env::BindingKind::DefinitionParameter,
        identity: mizar_checker::binding_env::BinderIdentity::ResolverLocal {
            scope: scope.clone(),
            ordinal: 0,
            declaration_range,
        },
        owner_context: mizar_checker::binding_env::BindingContextId::new(0),
        declaration_range,
        visible_after_ordinal: 0,
        type_site: mizar_checker::binding_env::BindingTypeSite::Missing,
        status: mizar_checker::binding_env::BindingStatus::Active,
        captured: mizar_checker::binding_env::CapturedFreeVariables::default(),
        diagnostics: Vec::new(),
        recovery: mizar_checker::binding_env::BindingRecoveryState::Normal,
    });
    let mut contexts = mizar_checker::binding_env::BindingContextTable::new();
    contexts.insert(mizar_checker::binding_env::BindingContextDraft {
        owner: mizar_checker::binding_env::BindingContextOwner::Module,
        parent: None,
        layer: mizar_checker::binding_env::BindingContextLayer::Module,
        lexical_scope: Some(scope),
        bindings: vec![parameter],
        visible_bindings: vec![parameter],
        recovery: mizar_checker::binding_env::BindingContextRecovery::Normal,
    });
    mizar_checker::binding_env::BindingEnv::try_new(
        mizar_checker::binding_env::BindingEnvParts {
            source_id: source,
            module_id: module,
            contexts,
            bindings,
            diagnostics: mizar_checker::binding_env::BindingDiagnosticTable::new(),
        },
    )
    .expect("Task253 synthetic definition-parameter binding environment")
}

fn task253_synthetic_symbols<const N: usize>(
    source: SourceId,
    module: ResolverModuleId,
    spellings: [&str; N],
) -> (
    SymbolEnv,
    Vec<(
        ResolverSymbolId,
        mizar_resolve::env::SourceContributionId,
    )>,
) {
    let mut indexes = SymbolEnvIndexes::default();
    let contribution = indexes.contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id: source },
        SourceAnchor::Range(range(source, 7, 8)),
    );
    let mut candidates = Vec::new();
    for (ordinal, spelling) in spellings.into_iter().enumerate() {
        let symbol = ResolverSymbolId::new(
            module.clone(),
            LocalSymbolId::new(format!("Functor/{spelling}/{ordinal}")),
            FullyQualifiedName::new(format!(
                "{}::{spelling}/{ordinal}",
                module.path().as_str()
            )),
        );
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 10 + ordinal, 11 + ordinal)),
            vec![ordinal as u32],
        );
        indexes.symbols.insert(SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Functor,
            NamespacePath::new(module.path().as_str()),
            spelling,
            origin.clone(),
            contribution,
        ));
        indexes
            .contributions
            .add_symbol(contribution, symbol.clone());
        let definition = indexes.definitions.insert(
            mizar_resolve::env::DefinitionShell::new(
                symbol.clone(),
                mizar_resolve::env::DefinitionKind::Functor,
                origin,
                contribution,
            ),
        );
        indexes
            .contributions
            .add_definition(contribution, definition);
        candidates.push((symbol, contribution));
    }
    (SymbolEnv::new(module, indexes), candidates)
}

fn task253_counts(
    typed_ast: &mizar_checker::typed_ast::TypedAst,
) -> (usize, usize, usize, usize, usize) {
    let handoff = typed_ast
        .source_application()
        .expect("Task253 synthetic handoff");
    (
        handoff.applications().len(),
        handoff.wrappers().len(),
        handoff.candidates().len(),
        handoff.arguments().len(),
        handoff.type_requests().len(),
    )
}
