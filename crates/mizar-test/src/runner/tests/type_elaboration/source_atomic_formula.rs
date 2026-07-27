use super::{
    SourceAtomicFormulaRouteOutput, TypeElaborationCaseStatus,
    expected_type_elaboration_detail_keys, run_type_elaboration_case,
    source_atomic_formula_output, source_atomic_formula_output_with_mutation,
};

const TASK256_CASES: [(&str, mizar_checker::source_atomic_formula::SourceAtomicFormulaKind); 8] = [
    (
        "fail_type_elaboration_term_formula_gap_001",
        mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
    ),
    (
        "fail_type_elaboration_builtin_inequality_formula_gap_001",
        mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Inequality,
    ),
    (
        "fail_type_elaboration_builtin_membership_formula_gap_001",
        mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Membership,
    ),
    (
        "fail_type_elaboration_builtin_type_assertion_formula_gap_001",
        mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::TypeAssertion,
    ),
    (
        "fail_type_elaboration_imported_predicate_functor_gap_001",
        mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::PredicateApplication,
    ),
    (
        "fail_type_elaboration_imported_attribute_assertion_formula_gap_001",
        mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::AttributeAssertion,
    ),
    (
        "fail_type_elaboration_imported_non_empty_attribute_assertion_formula_gap_001",
        mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::AttributeAssertion,
    ),
    (
        "fail_type_elaboration_set_enumeration_formula_gap_001",
        mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality,
    ),
];

#[test]
fn task256_real_routes_publish_the_frozen_aggregate_and_preserve_final_ownership() {
    let mut aggregate = [0_usize; 8];
    let mut primary_aggregate = [0_usize; 3];
    let mut application_aggregate = [0_usize; 5];
    let mut set_aggregate = [0_usize; 6];

    for (id, expected_kind) in TASK256_CASES {
        let (ast, module, symbols) = task252_real_ast(id);
        let first = source_atomic_formula_output(&ast, module.clone(), &symbols)
            .unwrap_or_else(|| panic!("{id} should select Task 256"))
            .unwrap_or_else(|error| panic!("{id} Task 256 failed: {error}"));
        let second = source_atomic_formula_output(&ast, module, &symbols)
            .unwrap_or_else(|| panic!("{id} should remain selected"))
            .unwrap_or_else(|error| panic!("{id} repeated Task 256 failed: {error}"));
        let handoff = task256_handoff(&first);
        let counts = [
            handoff.formulas().len(),
            handoff.wrappers().len(),
            handoff.predicate_heads().len(),
            handoff.candidates().len(),
            handoff.type_sites().len(),
            handoff.attributes().len(),
            handoff.edges().len(),
            handoff.requests().len(),
        ];
        for (total, count) in aggregate.iter_mut().zip(counts) {
            *total += count;
        }
        assert_eq!(
            handoff
                .formulas()
                .get(mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0))
                .expect("one Task 256 formula")
                .kind(),
            expected_kind,
            "{id}"
        );
        assert_eq!(
            first.typed_ast.source_atomic_formula(),
            first.resolved.source_atomic_formula(),
            "{id}"
        );
        assert_eq!(
            first.typed_ast.source_term(),
            first.resolved.source_term(),
            "{id}"
        );
        assert_eq!(
            first.typed_ast.source_application(),
            first.resolved.source_application(),
            "{id}"
        );
        assert_eq!(
            first.typed_ast.source_set_term(),
            first.resolved.source_set_term(),
            "{id}"
        );
        assert_eq!(
            first.typed_ast.debug_text(),
            second.typed_ast.debug_text(),
            "{id}"
        );
        assert_eq!(first.resolved.debug_text(), second.resolved.debug_text(), "{id}");

        let primary = first
            .typed_ast
            .source_term()
            .expect("Task 256 always installs Task 252 first");
        for (total, count) in primary_aggregate.iter_mut().zip([
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ]) {
            *total += count;
        }
        assert_eq!(handoff.primary_term_fingerprint(), primary.debug_text());

        if let Some(application) = first.typed_ast.source_application() {
            for (total, count) in application_aggregate.iter_mut().zip([
                application.applications().len(),
                application.wrappers().len(),
                application.candidates().len(),
                application.arguments().len(),
                application.type_requests().len(),
            ]) {
                *total += count;
            }
            assert_eq!(
                handoff.application_fingerprint(),
                Some(application.debug_text().as_str()),
                "{id}"
            );
        } else {
            assert_eq!(handoff.application_fingerprint(), None, "{id}");
        }

        if let Some(set_terms) = first.typed_ast.source_set_term() {
            for (total, count) in set_aggregate.iter_mut().zip([
                set_terms.terms().len(),
                set_terms.wrappers().len(),
                set_terms.generators().len(),
                set_terms.type_sites().len(),
                set_terms.edges().len(),
                set_terms.requests().len(),
            ]) {
                *total += count;
            }
            assert_eq!(
                handoff.set_term_fingerprint(),
                Some(set_terms.debug_text().as_str()),
                "{id}"
            );
        } else {
            assert_eq!(handoff.set_term_fingerprint(), None, "{id}");
        }
        assert_eq!(handoff.structure_fingerprint(), None, "{id}");
    }

    assert_eq!(aggregate, [8, 0, 1, 1, 1, 2, 13, 11]);
    assert_eq!(primary_aggregate, [16, 0, 16]);
    assert_eq!(application_aggregate, [1, 1, 1, 2, 2]);
    assert_eq!(set_aggregate, [2, 0, 0, 0, 4, 2]);
}

#[test]
fn task256_cross_family_edges_use_the_same_arena_and_nearest_owner() {
    let (ast, module, symbols) =
        task252_real_ast("fail_type_elaboration_imported_predicate_functor_gap_001");
    let predicate = source_atomic_formula_output(&ast, module, &symbols)
        .expect("predicate selector")
        .expect("predicate transaction");
    let predicate_handoff = task256_handoff(&predicate);
    assert_eq!(
        predicate_handoff
            .edges()
            .iter()
            .map(|(_, edge)| edge.target())
            .collect::<Vec<_>>(),
        [
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(0),
            ),
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Application(
                mizar_checker::source_application::SourceFunctorApplicationId::new(0),
            ),
        ]
    );
    let application = predicate
        .typed_ast
        .source_application()
        .expect("Task 253 dependency");
    assert_eq!(
        application
            .arguments()
            .iter()
            .map(|(_, argument)| argument.target())
            .collect::<Vec<_>>(),
        [
            mizar_checker::source_application::SourceFunctorArgumentTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(1),
            ),
            mizar_checker::source_application::SourceFunctorArgumentTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(2),
            ),
        ]
    );

    let (ast, module, symbols) =
        task252_real_ast("fail_type_elaboration_set_enumeration_formula_gap_001");
    let sets = source_atomic_formula_output(&ast, module, &symbols)
        .expect("set-enumeration selector")
        .expect("set-enumeration transaction");
    let set_handoff = task256_handoff(&sets);
    assert!(
        set_handoff.edges().iter().all(|(_, edge)| matches!(
            edge.target(),
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::SetTerm(_)
        ))
    );
    let set_terms = sets.typed_ast.source_set_term().expect("Task 255 dependency");
    assert_eq!(set_terms.terms().len(), 2);
    assert_eq!(set_terms.edges().len(), 4);
}

#[test]
fn task256_real_edges_and_requests_match_every_frozen_ordered_row() {
    use mizar_checker::source_atomic_formula::{
        SourceAssertionAttributeId, SourceAssertionTypeSiteId, SourceAtomicEdgeId,
        SourceAtomicEdgeRole, SourceAtomicRequestKind, SourceAtomicTermTarget,
        SourcePredicateCandidateId,
    };

    for (id, _) in TASK256_CASES {
        let (ast, module, symbols) = task252_real_ast(id);
        let expected_ranges = task256_expected_slot_ranges(id, &ast, &module, &symbols);
        let output = source_atomic_formula_output(&ast, module, &symbols)
            .unwrap_or_else(|| panic!("{id} selector"))
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        let handoff = task256_handoff(&output);
        let expected_edges: Vec<(SourceAtomicEdgeRole, SourceAtomicTermTarget)> = match id {
            "fail_type_elaboration_term_formula_gap_001"
            | "fail_type_elaboration_builtin_inequality_formula_gap_001"
            | "fail_type_elaboration_builtin_membership_formula_gap_001" => vec![
                (
                    SourceAtomicEdgeRole::BuiltinLeftOperand,
                    SourceAtomicTermTarget::Primary(
                        mizar_checker::source_term::SourcePrimaryTermId::new(0),
                    ),
                ),
                (
                    SourceAtomicEdgeRole::BuiltinRightOperand,
                    SourceAtomicTermTarget::Primary(
                        mizar_checker::source_term::SourcePrimaryTermId::new(1),
                    ),
                ),
            ],
            "fail_type_elaboration_builtin_type_assertion_formula_gap_001"
            | "fail_type_elaboration_imported_attribute_assertion_formula_gap_001"
            | "fail_type_elaboration_imported_non_empty_attribute_assertion_formula_gap_001" => {
                vec![(
                    SourceAtomicEdgeRole::AssertionSubject,
                    SourceAtomicTermTarget::Primary(
                        mizar_checker::source_term::SourcePrimaryTermId::new(0),
                    ),
                )]
            }
            "fail_type_elaboration_imported_predicate_functor_gap_001" => vec![
                (
                    SourceAtomicEdgeRole::PredicateLeftArgument,
                    SourceAtomicTermTarget::Primary(
                        mizar_checker::source_term::SourcePrimaryTermId::new(0),
                    ),
                ),
                (
                    SourceAtomicEdgeRole::PredicateRightArgument,
                    SourceAtomicTermTarget::Application(
                        mizar_checker::source_application::SourceFunctorApplicationId::new(0),
                    ),
                ),
            ],
            "fail_type_elaboration_set_enumeration_formula_gap_001" => vec![
                (
                    SourceAtomicEdgeRole::BuiltinLeftOperand,
                    SourceAtomicTermTarget::SetTerm(
                        mizar_checker::source_set_term::SourceSetTermId::new(0),
                    ),
                ),
                (
                    SourceAtomicEdgeRole::BuiltinRightOperand,
                    SourceAtomicTermTarget::SetTerm(
                        mizar_checker::source_set_term::SourceSetTermId::new(1),
                    ),
                ),
            ],
            _ => unreachable!(),
        };
        assert_eq!(
            handoff
                .edges()
                .iter()
                .map(|(edge_id, edge)| (
                    edge_id.index(),
                    edge.formula().index(),
                    edge.ordinal(),
                    edge.role(),
                    edge.target(),
                    task256_target_range(&output, edge.target()),
                ))
                .collect::<Vec<_>>(),
            expected_edges
                .into_iter()
                .zip(expected_ranges)
                .enumerate()
                .map(|(ordinal, ((role, target), range))| {
                    (ordinal, 0, ordinal, role, target, range)
                })
                .collect::<Vec<_>>(),
            "{id}"
        );

        let expected_requests = match id {
            "fail_type_elaboration_term_formula_gap_001"
            | "fail_type_elaboration_builtin_inequality_formula_gap_001"
            | "fail_type_elaboration_set_enumeration_formula_gap_001" => vec![
                (
                    SourceAtomicRequestKind::OperandExpectedType,
                    Some(SourceAtomicEdgeId::new(0)),
                    None,
                    None,
                    None,
                ),
                (
                    SourceAtomicRequestKind::OperandExpectedType,
                    Some(SourceAtomicEdgeId::new(1)),
                    None,
                    None,
                    None,
                ),
            ],
            "fail_type_elaboration_builtin_membership_formula_gap_001" => vec![(
                SourceAtomicRequestKind::OperandExpectedType,
                Some(SourceAtomicEdgeId::new(1)),
                None,
                None,
                None,
            )],
            "fail_type_elaboration_builtin_type_assertion_formula_gap_001" => vec![(
                SourceAtomicRequestKind::TypeAssertionReachability,
                None,
                None,
                Some(SourceAssertionTypeSiteId::new(0)),
                None,
            )],
            "fail_type_elaboration_imported_predicate_functor_gap_001" => vec![(
                SourceAtomicRequestKind::PredicateCandidateSignature,
                None,
                Some(SourcePredicateCandidateId::new(0)),
                None,
                None,
            )],
            "fail_type_elaboration_imported_attribute_assertion_formula_gap_001"
            | "fail_type_elaboration_imported_non_empty_attribute_assertion_formula_gap_001" => {
                vec![(
                    SourceAtomicRequestKind::AttributeAdmissibility,
                    None,
                    None,
                    None,
                    Some(SourceAssertionAttributeId::new(0)),
                )]
            }
            _ => unreachable!(),
        };
        assert_eq!(
            handoff
                .requests()
                .iter()
                .map(|(request_id, request)| (
                    request_id.index(),
                    request.formula().index(),
                    request.ordinal(),
                    request.kind(),
                    request.edge(),
                    request.candidate(),
                    request.type_site(),
                    request.attribute(),
                ))
                .collect::<Vec<_>>(),
            expected_requests
                .into_iter()
                .enumerate()
                .map(
                    |(ordinal, (kind, edge, candidate, type_site, attribute))| (
                        ordinal, 0, ordinal, kind, edge, candidate, type_site, attribute,
                    ),
                )
                .collect::<Vec<_>>(),
            "{id}"
        );
    }
}

#[test]
fn task256_resolver_provenance_type_head_and_attribute_polarity_are_exact() {
    let (ast, module, symbols) =
        task252_real_ast("fail_type_elaboration_imported_predicate_functor_gap_001");
    let expected_predicate =
        extract_source_imported_predicate_functor_formula(&ast, &module, &symbols)
            .expect("predicate extractor")
            .predicate_symbol;
    let predicate = source_atomic_formula_output(&ast, module, &symbols)
        .expect("predicate selector")
        .expect("predicate transaction");
    let handoff = task256_handoff(&predicate);
    let candidate = handoff
        .candidates()
        .get(mizar_checker::source_atomic_formula::SourcePredicateCandidateId::new(0))
        .expect("predicate candidate");
    assert_eq!(candidate.symbol(), &expected_predicate);
    assert_eq!(
        candidate.contribution(),
        symbols
            .symbols()
            .get(&expected_predicate)
            .expect("predicate resolver entry")
            .contribution()
    );

    let (ast, module, symbols) =
        task252_real_ast("fail_type_elaboration_builtin_type_assertion_formula_gap_001");
    let asserted = source_atomic_formula_output(&ast, module, &symbols)
        .expect("type selector")
        .expect("type transaction");
    assert_eq!(
        task256_handoff(&asserted)
            .type_sites()
            .get(mizar_checker::source_atomic_formula::SourceAssertionTypeSiteId::new(0))
            .expect("asserted type")
            .head(),
        mizar_checker::source_atomic_formula::SourceAssertionTypeHead::BuiltinSet
    );

    for (id, negative) in [
        (
            "fail_type_elaboration_imported_attribute_assertion_formula_gap_001",
            false,
        ),
        (
            "fail_type_elaboration_imported_non_empty_attribute_assertion_formula_gap_001",
            true,
        ),
    ] {
        let (ast, module, symbols) = task252_real_ast(id);
        let expected_attribute = if negative {
            extract_source_imported_non_empty_attribute_assertion_formula(
                &ast, &module, &symbols,
            )
            .expect("negative attribute extractor")
            .attribute_symbol
        } else {
            extract_source_imported_attribute_assertion_formula(&ast, &module, &symbols)
                .expect("attribute extractor")
                .attribute_symbol
        };
        let expected_contribution = symbols
            .symbols()
            .get(&expected_attribute)
            .expect("attribute resolver entry")
            .contribution();
        let output = source_atomic_formula_output(&ast, module, &symbols)
            .unwrap_or_else(|| panic!("{id} selector"))
            .unwrap_or_else(|error| panic!("{id}: {error}"));
        let attribute = task256_handoff(&output)
            .attributes()
            .get(mizar_checker::source_atomic_formula::SourceAssertionAttributeId::new(0))
            .expect("one assertion attribute");
        let attribute_nodes = surface_nodes_with_kind(&ast, SurfaceNodeKind::AttributeRef);
        let [(attribute_node_id, attribute_node)] =
            attribute_nodes.as_slice()
        else {
            panic!("{id}: expected one source AttributeRef")
        };
        let target_node_ids = structural_child_ids(&ast, attribute_node);
        let [target_node_id] = target_node_ids.as_slice() else {
            panic!("{id}: expected one source attribute target")
        };
        let target_node = ast
            .node(*target_node_id)
            .unwrap_or_else(|| panic!("{id}: attribute target disappeared"));
        assert_eq!(
            (
                attribute.site().node().index(),
                attribute.source_range(),
                attribute.target_site().node().index(),
                attribute.target_range(),
            ),
            (
                attribute_node_id.index(),
                attribute_node.range,
                target_node_id.index(),
                target_node.range,
            ),
            "{id}"
        );
        assert_eq!(attribute.symbol(), &expected_attribute, "{id}");
        assert_eq!(attribute.contribution(), expected_contribution, "{id}");
        assert_eq!(
            (attribute.spelling(), attribute.target_spelling()),
            (if negative { "non empty" } else { "empty" }, "empty"),
            "{id}"
        );
        assert_eq!(
            matches!(
                attribute.polarity(),
                mizar_checker::source_atomic_formula::SourceAssertionAttributePolarityInput::Negative {
                    non_spelling,
                    ..
                } if non_spelling == "non"
            ),
            negative,
            "{id}"
        );
        if negative {
            let mizar_checker::source_atomic_formula::SourceAssertionAttributePolarityInput::Negative {
                non_site,
                non_range,
                non_spelling,
                non_recovery,
            } = attribute.polarity()
            else {
                unreachable!()
            };
            let node_index = non_site.node().index();
            assert_eq!(ast.nodes()[node_index].range, *non_range);
            assert_eq!(ast.nodes()[node_index].token_text(), Some("non"));
            assert_eq!(non_range.source_id, ast.source_id);
            assert!(non_range.start < non_range.end);
            assert_eq!(non_spelling, "non");
            assert_eq!(
                *non_recovery,
                mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal
            );
            assert!(non_range.end <= attribute.target_range().start);
        }
    }
}

#[test]
fn task256_corruption_fails_atomically_without_poisoning_the_next_transaction() {
    let (ast, module, symbols) =
        task252_real_ast("fail_type_elaboration_imported_predicate_functor_gap_001");
    let corruptions: [
        fn(&mut mizar_checker::source_atomic_formula::SourceAtomicFormulaHandoffInput);
        5
    ] = [
        |input| input.formulas[0].source_ordinal = 1,
        |input| input.predicate_heads.clear(),
        |input| input.candidates.clear(),
        |input| input.edges.swap(0, 1),
        |input| {
            input.requests[0].candidate =
                Some(mizar_checker::source_atomic_formula::SourcePredicateCandidateId::new(1))
        },
    ];
    for corrupt in corruptions {
        assert!(
            source_atomic_formula_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                corrupt,
            )
            .expect("corruption must not change the exact selector")
            .is_err(),
            "corrupt Task 256 transaction must fail atomically"
        );
    }
    assert!(
        source_atomic_formula_output(&ast, module, &symbols)
            .expect("uncorrupted selector")
            .is_ok()
    );
}

#[test]
fn task256_selector_is_exact_and_lower_family_selectors_remain_unchanged() {
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
    let plan = build_test_plan(&config).expect("Task 256 repository plan should build");
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
        if source_atomic_formula_output(&ast, resolver.module, &symbols).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    let mut expected = TASK256_CASES
        .iter()
        .map(|(id, _)| (*id).to_owned())
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(selected, expected);

    let (ast, module, shells, symbols) =
        task253_real_ast("fail_type_elaboration_imported_predicate_functor_gap_001");
    let standalone_application = source_application_output(&ast, module, &shells, &symbols)
        .expect("Task 253 selector")
        .expect("Task 253 standalone transaction");
    assert_eq!(
        (
            standalone_application
                .typed_ast
                .source_term()
                .expect("Task 252 dependency")
                .terms()
                .len(),
            standalone_application
                .typed_ast
                .source_application()
                .expect("Task 253 handoff")
                .arguments()
                .len(),
        ),
        (2, 2)
    );

    let (ast, module, shells, symbols) = task255_real_ast();
    let standalone_set = source_set_term_output(&ast, module, &shells, &symbols)
        .expect("Task 255 selector")
        .expect("Task 255 standalone transaction");
    assert_eq!(
        (
            standalone_set
                .typed_ast
                .source_set_term()
                .expect("Task 255 handoff")
                .terms()
                .len(),
            standalone_set
                .typed_ast
                .source_term()
                .expect("Task 252 dependency")
                .terms()
                .len(),
        ),
        (4, 4)
    );
}

#[test]
fn task256_whole_subtree_exclusions_are_named_and_fail_closed() {
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
    let plan = build_test_plan(&config).expect("Task 256 exclusion plan should build");
    let excluded = [
        (
            "fail_type_elaboration_argument_bearing_attribute_gap_001",
            "argument-bearing attribute",
        ),
        (
            "fail_type_elaboration_structure_qualified_attribute_gap_001",
            "qualified attribute",
        ),
        (
            "fail_type_elaboration_non_builtin_type_gap_001",
            "non-bare asserted type family",
        ),
        (
            "pass_parser_atomic_formulas_001",
            "predicate chain, segment negation, and formula-level is-not",
        ),
        (
            "pass_parser_formula_connectives_001",
            "formula operators and binders",
        ),
        (
            "pass_parser_predicate_definitions_001",
            "predicate-definition and inline predicate family",
        ),
        (
            "fail_template_type_actual_missing_existential_001",
            "template predicate/formula family",
        ),
        (
            "pass_parser_set_comprehensions_001",
            "conditioned comprehension",
        ),
    ];

    for (id, boundary) in excluded {
        let (ordinal, case) = plan
            .cases
            .iter()
            .enumerate()
            .find(|(_, case)| case.id.0 == id)
            .unwrap_or_else(|| panic!("{id} should remain in the repository plan"));
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{id} frontend failed: {error}"));
        let ast = frontend.ast.unwrap_or_else(|| panic!("{id} AST"));
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        let module = resolver.module;
        let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
        assert!(
            source_atomic_formula_output(&ast, module, &symbols).is_none(),
            "{id}: {boundary} must remain a whole-subtree exclusion"
        );
    }
}

#[test]
fn task256_transport_keeps_all_eight_external_detail_vectors_owned_by_existing_routes() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate should live below the workspace root")
        .to_path_buf();
    let tests_root = workspace_root.join("tests");
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: tests_root.clone(),
        manifest_path: tests_root.join("coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task 256 repository plan should build");
    for id in TASK256_CASES.map(|(id, _)| id) {
        let (ordinal, case) = active_type_elaboration_cases(&plan)
            .enumerate()
            .find(|(_, case)| case.id.0 == id)
            .unwrap_or_else(|| panic!("{id} remains active"));
        let result = run_type_elaboration_case(&workspace_root, &tests_root, case, ordinal);
        assert_eq!(result.status, TypeElaborationCaseStatus::Passed, "{id}");
        assert_eq!(
            result.actual_detail_keys,
            expected_type_elaboration_detail_keys(case),
            "{id}"
        );
    }
}

fn task256_handoff(
    output: &SourceAtomicFormulaRouteOutput,
) -> &mizar_checker::source_atomic_formula::SourceAtomicFormulaHandoff {
    output
        .typed_ast
        .source_atomic_formula()
        .expect("Task 256 handoff")
}

fn task256_expected_slot_ranges(
    id: &str,
    ast: &SurfaceAst,
    module: &ResolverModuleId,
    symbols: &SymbolEnv,
) -> Vec<SourceRange> {
    match id {
        "fail_type_elaboration_term_formula_gap_001"
        | "fail_type_elaboration_builtin_inequality_formula_gap_001"
        | "fail_type_elaboration_builtin_membership_formula_gap_001" => {
            let payload = extract_source_builtin_binary_term_formula(ast).expect("binary payload");
            vec![payload.left_range, payload.right_range]
        }
        "fail_type_elaboration_builtin_type_assertion_formula_gap_001" => vec![
            extract_source_builtin_type_assertion_formula(ast, module, symbols)
                .expect("type payload")
                .subject_range,
        ],
        "fail_type_elaboration_imported_predicate_functor_gap_001" => {
            let payload = extract_source_imported_predicate_functor_formula(ast, module, symbols)
                .expect("predicate payload");
            let wrappers = ast
                .nodes()
                .iter()
                .filter(|node| matches!(node.kind, SurfaceNodeKind::ParenthesizedTerm))
                .filter(|node| {
                    node.range.start < payload.functor_range.start
                        && payload.functor_range.end < node.range.end
                })
                .map(|node| node.range)
                .collect::<Vec<_>>();
            let [wrapper_range] = wrappers.as_slice() else {
                panic!("Task 256 imported application must have one outer effective wrapper");
            };
            vec![payload.left_range, *wrapper_range]
        }
        "fail_type_elaboration_imported_attribute_assertion_formula_gap_001" => vec![
            extract_source_imported_attribute_assertion_formula(ast, module, symbols)
                .expect("attribute payload")
                .subject_range,
        ],
        "fail_type_elaboration_imported_non_empty_attribute_assertion_formula_gap_001" => vec![
            extract_source_imported_non_empty_attribute_assertion_formula(ast, module, symbols)
                .expect("negative attribute payload")
                .subject_range,
        ],
        "fail_type_elaboration_set_enumeration_formula_gap_001" => {
            let payload = extract_source_set_enumeration_formula(ast).expect("set payload");
            vec![payload.left_range, payload.right_range]
        }
        _ => unreachable!(),
    }
}

fn task256_target_range(
    output: &SourceAtomicFormulaRouteOutput,
    target: mizar_checker::source_atomic_formula::SourceAtomicTermTarget,
) -> SourceRange {
    match target {
        mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(id) => output
            .typed_ast
            .source_term()
            .and_then(|handoff| handoff.terms().get(id))
            .expect("primary target")
            .source_range(),
        mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Application(id) => {
            let handoff = output
                .typed_ast
                .source_application()
                .expect("application target handoff");
            handoff
                .wrappers()
                .iter()
                .find(|(_, wrapper)| wrapper.application() == id && wrapper.ordinal() == 0)
                .map(|(_, wrapper)| wrapper.source_range())
                .unwrap_or_else(|| {
                    handoff
                        .applications()
                        .get(id)
                        .expect("application target")
                        .source_range()
                })
        }
        mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Structure(id) => {
            let handoff = output
                .typed_ast
                .source_structure()
                .expect("structure target handoff");
            handoff
                .wrappers()
                .iter()
                .find(|(_, wrapper)| wrapper.term() == id && wrapper.ordinal() == 0)
                .map(|(_, wrapper)| wrapper.source_range())
                .unwrap_or_else(|| {
                    handoff
                        .terms()
                        .get(id)
                        .expect("structure target")
                        .source_range()
                })
        }
        mizar_checker::source_atomic_formula::SourceAtomicTermTarget::SetTerm(id) => {
            let handoff = output
                .typed_ast
                .source_set_term()
                .expect("set target handoff");
            handoff
                .wrappers()
                .iter()
                .find(|(_, wrapper)| wrapper.term() == id && wrapper.ordinal() == 0)
                .map(|(_, wrapper)| wrapper.source_range())
                .unwrap_or_else(|| {
                    handoff
                        .terms()
                        .get(id)
                        .expect("set target")
                        .source_range()
                })
        }
        _ => unreachable!("future Task 256 target kind"),
    }
}
