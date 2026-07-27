use super::{
    SourceBindingContextRouteOutput, SourceCompositeFormulaRouteOutput,
    source_composite_formula_output, source_composite_formula_output_with_mutation,
};

const TASK257A_CASE: &str = "fail_type_elaboration_formula_connective_quantifier_gap_001";

#[test]
fn task257a_real_route_publishes_the_exact_tree_binder_and_final_ownership() {
    let (ast, module, symbols) = task252_real_ast(TASK257A_CASE);
    let payload = extract_source_formula_connective_quantifier(&ast, &module, &symbols)
        .expect("Task 257A selector payload");
    let first = source_composite_formula_output(&ast, module.clone(), &symbols)
        .expect("Task 257A selects")
        .expect("Task 257A transaction");
    let second = source_composite_formula_output(&ast, module, &symbols)
        .expect("Task 257A remains selected")
        .expect("Task 257A replay");
    let handoff = task257a_handoff(&first);

    assert_eq!(
        (
            handoff.formulas().len(),
            handoff.wrappers().len(),
            handoff.roots().len(),
            handoff.binders().len(),
            handoff.type_sites().len(),
            handoff.edges().len(),
            handoff.requests().len(),
        ),
        (5, 0, 1, 1, 1, 4, 6)
    );
    assert_eq!(
        (
            handoff.binding_env().contexts().len(),
            handoff.binding_env().bindings().len(),
            handoff.binding_env().diagnostics().len(),
        ),
        (2, 1, 4)
    );
    assert!(first.typed_ast.source_context().is_none());
    assert_eq!(
        first.typed_ast.source_composite_formula(),
        first.resolved.source_composite_formula()
    );
    assert_eq!(
        first.typed_ast.debug_text(),
        second.typed_ast.debug_text()
    );
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());

    let formulas = handoff
        .formulas()
        .iter()
        .map(|(_, formula)| {
            (
                formula.site().node().index(),
                formula.source_range().start,
                formula.source_range().end,
                formula.context().index(),
                formula.spelling().to_owned(),
                formula.kind(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        formulas,
        [
            (
                payload.implication_site.node().index(),
                52,
                113,
                0,
                "implies".to_owned(),
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Implication,
            ),
            (
                payload.premise_constant_site.node().index(),
                52,
                65,
                0,
                "contradiction".to_owned(),
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Contradiction,
            ),
            (
                payload.quantified_site.node().index(),
                74,
                113,
                0,
                "for holds".to_owned(),
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Universal,
            ),
            (
                payload.negation_site.node().index(),
                96,
                113,
                1,
                "not".to_owned(),
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Negation,
            ),
            (
                payload.body_constant_site.node().index(),
                100,
                113,
                1,
                "contradiction".to_owned(),
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Contradiction,
            ),
        ]
    );
    assert_eq!(
        handoff
            .formulas()
            .iter()
            .map(|(_, formula)| (formula.source_ordinal(), formula.recovery()))
            .collect::<Vec<_>>(),
        (0..5)
            .map(|ordinal| {
                (
                    ordinal,
                    mizar_checker::source_composite_formula::SourceCompositeFormulaRecovery::Normal,
                )
            })
            .collect::<Vec<_>>()
    );
    let root = handoff
        .roots()
        .get(mizar_checker::source_composite_formula::SourceFormulaRootId::new(0))
        .expect("root");
    assert_eq!(
        (root.formula().index(), root.ordinal(), root.ownership()),
        (
            0,
            0,
            mizar_checker::source_composite_formula::SourceFormulaRootOwnership::UnassignedStatement,
        )
    );
    let binder = handoff
        .binders()
        .get(mizar_checker::source_composite_formula::SourceQuantifierBinderId::new(0))
        .expect("binder");
    assert_eq!(
        (
            binder.segment_site().node().index(),
            binder.segment_range().start,
            binder.segment_range().end,
            binder.identifier_site().node().index(),
            binder.identifier_range().start,
            binder.identifier_range().end,
            binder.identifier_spelling(),
            binder.local().scope().path(),
            binder.binding().index(),
            binder.body_context().index(),
        ),
        (
            payload.binder_segment_site.node().index(),
            78,
            89,
            payload.binder_identifier_site.node().index(),
            78,
            79,
            "x",
            [0].as_slice(),
            0,
            1,
        )
    );
    assert_eq!(
        (
            binder.formula().index(),
            binder.ordinal(),
            binder.segment_spelling(),
            binder.local().spelling(),
            binder.local().declaration_range(),
            binder.local().visible_after_ordinal(),
            binder.type_site().index(),
            binder.recovery(),
        ),
        (
            2,
            0,
            "x being",
            "x",
            binder.identifier_range(),
            0,
            0,
            mizar_checker::source_composite_formula::SourceCompositeFormulaRecovery::Normal,
        )
    );
    let type_site = handoff
        .type_sites()
        .get(mizar_checker::source_composite_formula::SourceBinderTypeSiteId::new(0))
        .expect("binder type");
    assert_eq!(
        (
            type_site.site().node().index(),
            type_site.source_range().start,
            type_site.source_range().end,
            type_site.head_site().node().index(),
            type_site.head_range().start,
            type_site.head_range().end,
            type_site.context().index(),
        ),
        (
            payload.binder_type_site.node().index(),
            86,
            89,
            payload.binder_type_head_site.node().index(),
            86,
            89,
            0,
        )
    );
    assert_eq!(
        (
            type_site.binder().index(),
            type_site.spelling(),
            type_site.head_spelling(),
            type_site.recovery(),
            type_site.head(),
        ),
        (
            0,
            "set",
            "set",
            mizar_checker::source_composite_formula::SourceCompositeFormulaRecovery::Normal,
            mizar_checker::source_composite_formula::SourceBinderTypeHead::BuiltinSet,
        )
    );
    assert_eq!(
        handoff
            .edges()
            .iter()
            .map(|(_, edge)| {
                (
                    edge.parent().index(),
                    edge.ordinal(),
                    edge.role(),
                    edge.child().index(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::ImplicationLeft,
                1,
            ),
            (
                0,
                1,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::ImplicationRight,
                2,
            ),
            (
                2,
                0,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::UniversalBody,
                3,
            ),
            (
                3,
                0,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::NegatedFormula,
                4,
            ),
        ]
    );
    assert_eq!(
        handoff
            .requests()
            .iter()
            .map(|(_, request)| {
                (
                    request.formula().index(),
                    request.ordinal(),
                    request.kind(),
                    request.binder().map(|binder| binder.index()),
                    request.type_site().map(|type_site| type_site.index()),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_composite_formula::SourceFormulaRequestKind::ConnectiveSemantics,
                None,
                None,
            ),
            (
                1,
                0,
                mizar_checker::source_composite_formula::SourceFormulaRequestKind::ConstantSemantics,
                None,
                None,
            ),
            (
                2,
                0,
                mizar_checker::source_composite_formula::SourceFormulaRequestKind::QuantifierSemantics,
                None,
                None,
            ),
            (
                2,
                1,
                mizar_checker::source_composite_formula::SourceFormulaRequestKind::BinderType,
                Some(0),
                Some(0),
            ),
            (
                3,
                0,
                mizar_checker::source_composite_formula::SourceFormulaRequestKind::NegationSemantics,
                None,
                None,
            ),
            (
                4,
                0,
                mizar_checker::source_composite_formula::SourceFormulaRequestKind::ConstantSemantics,
                None,
                None,
            ),
        ]
    );
}

#[test]
fn task257a_corruptions_fail_atomically_and_the_valid_route_recovers() {
    let (ast, module, symbols) = task252_real_ast(TASK257A_CASE);
    let corruptions: [fn(
        &mut mizar_checker::source_composite_formula::SourceCompositeFormulaHandoffInput,
    ); 8] = [
        |input| input.formulas[0].spelling = "iff".to_owned(),
        |input| {
            input
                .wrappers
                .push(mizar_checker::source_composite_formula::SourceFormulaWrapperInput {
                    formula:
                        mizar_checker::source_composite_formula::SourceCompositeFormulaId::new(0),
                    ordinal: 0,
                    site: input.formulas[0].site.clone(),
                    source_range: input.formulas[0].source_range,
                    context: input.formulas[0].context,
                    recovery: mizar_checker::source_composite_formula::SourceCompositeFormulaRecovery::Normal,
                    spelling: "( )".to_owned(),
                })
        },
        |input| input.roots.clear(),
        |input| input.binders[0].body_context = mizar_checker::binding_env::BindingContextId::new(0),
        |input| input.type_sites[0].head_spelling = "object".to_owned(),
        |input| input.edges.swap(0, 1),
        |input| input.requests[3].binder = None,
        |input| input.requests[3].type_site = None,
    ];
    for corrupt in corruptions {
        assert!(
            source_composite_formula_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                corrupt,
            )
            .expect("corruption preserves selector")
            .is_err()
        );
    }
    assert!(
        source_composite_formula_output(&ast, module, &symbols)
            .expect("valid route remains selected")
            .is_ok()
    );
}

#[test]
fn task257a_selector_is_exact_and_does_not_expand_lower_families() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate below workspace")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 257A plan");
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
        if source_composite_formula_output(&ast, resolver.module, &symbols).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(selected, [TASK257A_CASE.to_owned()]);

    for lower in [
        "fail_type_elaboration_imported_predicate_functor_gap_001",
        "fail_type_elaboration_imported_structure_gap_001",
        "fail_type_elaboration_set_enumeration_formula_gap_001",
    ] {
        let (ast, module, symbols) = task252_real_ast(lower);
        assert!(
            source_composite_formula_output(&ast, module, &symbols).is_none(),
            "{lower}"
        );
    }
}

#[test]
fn task257a_transport_preserves_the_existing_semantic_detail_vector() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate below workspace")
        .to_path_buf();
    let tests_root = workspace_root.join("tests");
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: tests_root.clone(),
        manifest_path: tests_root.join("coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 257A plan");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK257A_CASE)
        .expect("Task 257A case active");
    let result = run_type_elaboration_case(&workspace_root, &tests_root, case, ordinal);
    assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
    assert_eq!(
        result.actual_detail_keys,
        [
            "type_elaboration.checker.checker.formula.external.formula_payload".to_owned(),
            "type_elaboration.checker.checker.formula.external.quantifier_payload".to_owned(),
        ]
    );
    assert_eq!(
        result.actual_detail_keys,
        expected_type_elaboration_detail_keys(case)
    );

    let (ast, module, symbols) = task252_real_ast(TASK257A_CASE);
    assert!(
        source_formula_connective_quantifier_output(&ast, module, &symbols).is_some(),
        "older semantic detail owner remains admitted"
    );
}

#[test]
fn task257a_rejects_a_preinstalled_task248_source_context() {
    let task248 = task248_real_output();
    assert!(task248.typed_ast.source_context().is_some());
    assert!(task248.typed_ast.source_composite_formula().is_none());

    let (ast, module, symbols) = task252_real_ast(TASK257A_CASE);
    let task257a = source_composite_formula_output(&ast, module, &symbols)
        .expect("Task 257A selector")
        .expect("Task 257A transaction");
    let handoff = task257a
        .typed_ast
        .source_composite_formula()
        .expect("Task 257A handoff")
        .clone();
    assert_eq!(
        task248
            .typed_ast
            .clone()
            .with_source_composite_formula(handoff)
            .expect_err("Task 248/257A coexistence must fail"),
        mizar_checker::typed_ast::TypedAstError::InvalidSourceCompositeFormula
    );
    assert!(task248.typed_ast.source_composite_formula().is_none());
}

fn task257a_handoff(
    output: &SourceCompositeFormulaRouteOutput,
) -> &mizar_checker::source_composite_formula::SourceCompositeFormulaHandoff {
    output
        .typed_ast
        .source_composite_formula()
        .expect("Task 257A handoff")
}

fn task248_real_output() -> SourceBindingContextRouteOutput {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate below workspace")
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 248 plan");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "pass_type_elaboration_source_binding_context_shadowing_001"
        })
        .expect("Task 248 case active");
    let frontend = run_frontend(&workspace_root, case, ordinal).expect("Task 248 frontend");
    let ast = frontend.ast.expect("Task 248 AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    let shells = resolver.shells.clone();
    let symbols =
        augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
    source_binding_context_output(&ast, resolver.module, &shells, &symbols)
        .expect("Task 248 selector")
        .expect("Task 248 transaction")
}
