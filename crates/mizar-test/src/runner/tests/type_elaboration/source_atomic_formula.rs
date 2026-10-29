use super::{
    SourceAtomicFormulaRouteOutput, TypeElaborationCaseStatus,
    expected_type_elaboration_detail_keys, extract_source_imported_predicate_chain_formula,
    run_type_elaboration_case,
    source_atomic_formula_output, source_atomic_formula_output_with_mutation,
    source_atomic_formula_output_with_source,
    source_atomic_formula_output_with_source_and_mutation,
};

const TASK257C1_CASE: &str =
    "pass_type_elaboration_formula_predicate_chain_segment_payload_001";
const TASK257C1_SOURCE: &str = "import parser.type_fixtures;\ntheorem FormulaPredicateChainPayloadBoundary: 1 divides 2 does not divides 3;\n";

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

#[test]
fn metamorphic_chain_layout_transports_actual_imported_ranges_and_shared_boundary() {
    let config = step5c11_config();
    let root = &config.workspace_root;
    let plan = build_test_plan(&config).unwrap();
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK257C1_CASE)
        .unwrap();
    let original = std::fs::read_to_string(&case.source_path).unwrap();
    let (old_ast, old_module, old_symbols) = task252_real_ast(TASK257C1_CASE);
    let previous =
        extract_source_imported_predicate_chain_formula(&old_ast, &old_module, &old_symbols)
            .unwrap();
    for (layout, source) in [
        ("leading_line", format!("\n{original}")),
        ("intertoken_layout", original.replace(' ', "\t  ")),
        (
            "import_comment",
            original.replacen(
                "parser.type_fixtures",
                "parser :: path trivia\n . type_fixtures",
                1,
            ),
        ),
        (
            "theorem_double_space",
            original.replacen("theorem ", "theorem  ", 1),
        ),
        ("missing_lf", original.trim_end_matches('\n').to_owned()),
        ("trailing_trivia", format!("{original}\n:: trailing\n")),
        (
            "polarity_comment",
            original.replacen("does not", "does :: polarity\n not", 1),
        ),
    ] {
        metamorphic_guard::with_source(case, &source, || {
            let frontend = run_frontend(root, case, ordinal).unwrap();
            assert!(frontend.diagnostics.is_empty(), "{layout}");
            assert_eq!(frontend.source_text.as_ref(), source);
            let ast = frontend.ast.as_ref().unwrap();
            assert!(ast.node_views().all(|node| !node.is_recovered()));
            let resolver = resolver_symbol_collection(root, case, ast);
            assert!(resolver.detail_keys.is_empty());
            let symbols =
                augment_type_elaboration_import_summaries(ast, &resolver.module, resolver.env);
            let lower = source_atomic_formula_output_with_source(
                ast,
                resolver.module.clone(),
                &symbols,
                &source,
            )
            .expect("actual layout selects Atomic chain")
            .expect("actual Atomic chain transaction");
            let output = source_predicate_chain_composition_output_with_source(
                ast,
                resolver.module.clone(),
                &symbols,
                &source,
            )
            .expect("actual layout selects C3")
            .expect("actual C3 transaction");
            let payload =
                extract_source_imported_predicate_chain_formula(ast, &resolver.module, &symbols)
                    .unwrap();
            let old_tokens = old_ast.token_views().collect::<Vec<_>>();
            let tokens = ast.token_views().collect::<Vec<_>>();
            assert_eq!(old_tokens.len(), tokens.len());
            for (old, actual) in old_tokens.iter().zip(&tokens) {
                assert_eq!(old.as_token(), actual.as_token());
                assert_eq!(
                    source.get(actual.range().start..actual.range().end),
                    actual.as_token().map(|token| token.text.as_ref())
                );
            }
            let shifted = |old: mizar_session::SourceRange| mizar_session::SourceRange {
                source_id: ast.source_id,
                start: old_tokens
                    .iter()
                    .zip(&tokens)
                    .find(|(token, _)| token.range().start == old.start)
                    .unwrap()
                    .1
                    .range()
                    .start,
                end: old_tokens
                    .iter()
                    .zip(&tokens)
                    .find(|(token, _)| token.range().end == old.end)
                    .unwrap()
                    .1
                    .range()
                    .end,
            };
            for (actual, old) in [
                (payload.formula_range, previous.formula_range),
                (payload.verb_range, previous.verb_range),
                (payload.not_range, previous.not_range),
            ]
            .into_iter()
            .chain(
                payload
                    .segment_ranges
                    .into_iter()
                    .zip(previous.segment_ranges),
            )
            .chain(payload.head_ranges.into_iter().zip(previous.head_ranges))
            .chain(payload.term_ranges.into_iter().zip(previous.term_ranges))
            {
                assert_eq!(actual, shifted(old), "{layout}");
            }
            for kind in [
                mizar_syntax::SurfaceNodeKind::ImportItem,
                mizar_syntax::SurfaceNodeKind::ImportAliasDecl,
                mizar_syntax::SurfaceNodeKind::ModulePath,
            ] {
                let old = old_ast
                    .nodes()
                    .iter()
                    .find(|node| node.kind == kind)
                    .unwrap();
                let actual = ast.nodes().iter().find(|node| node.kind == kind).unwrap();
                assert_eq!(actual.range, shifted(old.range));
            }
            let primary = output.typed_ast.source_term().unwrap();
            let atomic = output.typed_ast.source_atomic_formula().unwrap();
            let composition = output
                .typed_ast
                .source_predicate_chain_composition()
                .unwrap();
            assert_eq!(lower.typed_ast.source_term(), Some(primary));
            assert_eq!(lower.typed_ast.source_atomic_formula(), Some(atomic));
            assert_eq!(
                (
                    primary.terms().len(),
                    primary.references().len(),
                    primary.numeric_type_requests().len()
                ),
                (3, 0, 3)
            );
            assert_eq!(
                primary
                    .terms()
                    .iter()
                    .map(|(_, term)| term.source_range())
                    .collect::<Vec<_>>(),
                payload.term_ranges
            );
            assert_eq!(
                atomic
                    .predicate_segments()
                    .iter()
                    .map(|(_, segment)| segment.source_range())
                    .collect::<Vec<_>>(),
                payload.segment_ranges
            );
            assert_eq!(
                atomic
                    .predicate_heads()
                    .iter()
                    .map(|(_, head)| head.source_range())
                    .collect::<Vec<_>>(),
                payload.head_ranges
            );
            assert_eq!(
                atomic.formulas().iter().next().unwrap().1.source_range(),
                payload.formula_range
            );
            assert_eq!(
                atomic.formulas().iter().next().unwrap().1.spelling(),
                "1 divides 2 does not divides 3"
            );
            for (index, (_, term)) in primary.terms().iter().enumerate() {
                assert_eq!(
                    source.get(term.source_range().start..term.source_range().end),
                    Some(["1", "2", "3"][index])
                );
            }
            let candidates = atomic.candidates().iter().collect::<Vec<_>>();
            assert_eq!(candidates.len(), 2);
            assert_eq!(candidates[0].1.symbol(), candidates[1].1.symbol());
            assert_eq!(
                candidates[0].1.contribution(),
                candidates[1].1.contribution()
            );
            assert_eq!(candidates[0].1.symbol(), &payload.predicate_symbol);
            let entry = symbols.symbols().get(candidates[0].1.symbol()).unwrap();
            let contribution = symbols
                .contributions()
                .get(candidates[0].1.contribution())
                .unwrap();
            let import = ast
                .nodes()
                .iter()
                .find(|node| matches!(node.kind, mizar_syntax::SurfaceNodeKind::ImportAliasDecl))
                .unwrap();
            assert_eq!(
                entry.symbol().module().path().as_str(),
                "parser.type_fixtures"
            );
            assert_eq!(entry.origin().source_id(), ast.source_id);
            assert_eq!(
                entry.origin().anchor(),
                &mizar_session::SourceAnchor::Range(import.range)
            );
            assert_eq!(
                contribution.anchor(),
                &mizar_session::SourceAnchor::Range(import.range)
            );
            assert!(
                matches!(contribution.kind(), mizar_resolve::env::ContributionKind::ImportedSource { source_id } if *source_id == ast.source_id)
            );
            for (_, head) in atomic.predicate_heads().iter() {
                assert_eq!((head.left_arity(), head.right_arity()), (1, 1));
                assert_eq!(
                    source.get(head.source_range().start..head.source_range().end),
                    Some("divides")
                );
            }
            let negative = atomic.predicate_segments().iter().nth(1).unwrap().1;
            let mizar_checker::source_atomic_formula::SourcePredicateSegmentPolarityInput::Negative { verb_range, not_range, .. } = negative.polarity() else { panic!("actual does-not segment"); };
            assert_eq!(
                (*verb_range, *not_range),
                (payload.verb_range, payload.not_range)
            );
            assert_eq!(source.get(verb_range.start..verb_range.end), Some("does"));
            assert_eq!(source.get(not_range.start..not_range.end), Some("not"));
            assert_eq!(atomic.edges().iter().filter(|(_, edge)| edge.role() == mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::PredicateChainBoundary).map(|(_, edge)| edge.target()).collect::<Vec<_>>(), [mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(mizar_checker::source_term::SourcePrimaryTermId::new(1))]);
            assert_eq!(
                (
                    composition.conjunctions().len(),
                    composition.negations().len()
                ),
                (1, 1)
            );
            assert_eq!(
                composition
                    .conjunctions()
                    .iter()
                    .next()
                    .unwrap()
                    .1
                    .boundary()
                    .index(),
                1
            );
            assert_eq!(
                composition
                    .negations()
                    .iter()
                    .next()
                    .unwrap()
                    .1
                    .segment()
                    .index(),
                1
            );
            assert_eq!(atomic.primary_term_fingerprint(), primary.debug_text());
            assert_eq!(composition.primary_term_fingerprint(), primary.debug_text());
            assert_eq!(
                composition.atomic_formula_fingerprint(),
                atomic.debug_text()
            );
            assert_eq!(
                output.typed_ast.source_predicate_chain_composition(),
                output.resolved.source_predicate_chain_composition()
            );
            assert!(output.typed_ast.facts().is_empty());
            assert!(output.resolved.checked_formulas().is_empty());
            assert!(output.resolved.statement_semantics().is_empty());
            let replay = source_predicate_chain_composition_output_with_source(
                ast,
                resolver.module,
                &symbols,
                &source,
            )
            .unwrap()
            .unwrap();
            assert_eq!(output.typed_ast, replay.typed_ast);
            assert_eq!(output.resolved, replay.resolved);
            let result = run_type_elaboration_case(root, &root.join("tests"), case, ordinal);
            assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
            assert!(result.actual_detail_keys.is_empty());
        });
    }
}

#[test]
fn metamorphic_chain_layout_native_seed_matches_loaded_frontend() {
    use mizar_frontend::lexical_env::{
        LexicalEnvironmentRequest, LexicalSummaryProvider, ResolvedImportEntry,
        build_active_lexical_environment,
    };
    for source in [
        TASK257C1_SOURCE.to_owned(),
        format!("\n{TASK257C1_SOURCE}"),
        TASK257C1_SOURCE.replacen(
            "parser.type_fixtures",
            "parser :: path trivia\n . type_fixtures",
            1,
        ),
    ] {
        let frontend = task257c4c1_frontend_output(&source, 264);
        assert!(frontend.diagnostics.is_empty());
        let request = LexicalEnvironmentRequest {
            source_id: frontend.source.source_id,
            import_stubs: &frontend.preprocessed.import_stubs,
            edition: frontend.source.edition.clone(),
        };
        let native = ParseOnlyImportProvider.resolve_imports(&request).unwrap();
        assert!(native.diagnostics.is_empty());
        let preprocessed = mizar_lexer::preprocess_source_for_lexing(&source);
        assert!(preprocessed.diagnostics.is_empty());
        assert_eq!(
            preprocessed.lexical_text,
            frontend.preprocessed.lexical_text.as_str()
        );
        let raw = mizar_lexer::scan_raw(&preprocessed.lexical_text).unwrap();
        let prelude = mizar_lexer::scan_import_prelude(&raw);
        assert!(prelude.diagnostics.is_empty());
        assert_eq!(prelude.imports.len(), request.import_stubs.len());
        let mapped = |span| {
            let ranges = preprocessed
                .preprocess_map
                .source_ranges_for_lexical(span)
                .unwrap();
            mizar_session::SourceRange {
                source_id: frontend.source.source_id,
                start: ranges.first().unwrap().start,
                end: ranges.last().unwrap().end,
            }
        };
        for (actual, supplied) in prelude.imports.iter().zip(request.import_stubs) {
            assert!(actual.alias.is_none() && supplied.alias.is_none());
            assert!(actual.path.relative.is_none() && supplied.path.relative.is_none());
            assert_eq!(actual.path.spelling, supplied.path.spelling.as_ref());
            assert_eq!(mapped(actual.span), supplied.span);
            assert_eq!(mapped(actual.path.span), supplied.path.span);
            assert_eq!(
                actual
                    .path
                    .components
                    .iter()
                    .map(|part| part.spelling.as_str())
                    .collect::<Vec<_>>(),
                supplied
                    .path
                    .components
                    .iter()
                    .map(|part| part.as_ref())
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                actual
                    .path
                    .source_segments
                    .iter()
                    .copied()
                    .map(&mapped)
                    .collect::<Vec<_>>(),
                supplied.path.source_segments
            );
            for component in &actual.path.components {
                let range = mapped(component.span);
                assert_eq!(
                    source.get(range.start..range.end),
                    Some(component.spelling.as_str())
                );
            }
        }
        let imports = prelude
            .imports
            .iter()
            .map(|import| mizar_lexer::ResolvedImport {
                module_id: mizar_lexer::ModuleId::new(import.path.spelling.as_str()),
            })
            .collect::<Vec<_>>();
        let summaries = imports
            .iter()
            .enumerate()
            .map(|(ordinal, import)| mizar_lexer::ModuleLexicalSummary {
                module_id: import.module_id.clone(),
                exported_symbols: super::import_fixtures::parse_only_fixture_symbols(
                    &import.module_id,
                ),
                fingerprint: mizar_lexer::LexicalSummaryFingerprint::new((ordinal as u64) + 1),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            native.imports,
            prelude
                .imports
                .iter()
                .zip(&imports)
                .enumerate()
                .map(|(ordinal, (raw, import))| ResolvedImportEntry {
                    stub_ordinal: ordinal,
                    stub_span: mapped(raw.span),
                    import: import.clone()
                })
                .collect::<Vec<_>>()
        );
        assert_eq!(
            native.summaries, summaries,
            "complete exports/ranks/arity/operator metadata and summary fingerprints"
        );
        let direct = mizar_lexer::build_lexical_environment(&imports, &summaries).unwrap();
        let native_environment =
            build_active_lexical_environment(&request, &ParseOnlyImportProvider).unwrap();
        assert!(native_environment.diagnostics.is_empty());
        assert_eq!(direct, native_environment.environment);
        let scopes = mizar_lexer::build_scope_skeleton(&raw);
        let tokens = mizar_lexer::disambiguate(
            &raw,
            &direct,
            &mizar_lexer::ParserLexContext::general(),
            &scopes,
        );
        assert!(tokens.diagnostics().is_empty());
        assert_eq!(tokens.tokens().len(), frontend.tokens.tokens().len());
        for (actual, expected) in tokens.tokens().iter().zip(frontend.tokens.tokens()) {
            assert_eq!(
                (actual.kind(), actual.lexeme(), mapped(actual.span())),
                (expected.kind, expected.text.as_ref(), expected.span)
            );
        }
        let heads = tokens
            .tokens()
            .iter()
            .filter(|token| token.lexeme() == "divides")
            .collect::<Vec<_>>();
        assert_eq!(heads.len(), 2);
        assert!(
            heads
                .iter()
                .all(|token| token.kind() == mizar_lexer::TokenKind::UserSymbol)
        );
    }
}

#[test]
fn metamorphic_chain_layout_distinguishes_selected_errors_and_unsupported_members() {
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    let config = step5c11_config();
    let root = &config.workspace_root;
    let plan = build_test_plan(&config).unwrap();
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK257C1_CASE)
        .unwrap();
    let source = std::fs::read_to_string(&case.source_path).unwrap();
    let frontend = run_frontend(root, case, ordinal).unwrap();
    assert!(frontend.diagnostics.is_empty());
    let ast = frontend.ast.as_ref().unwrap();
    let resolver = resolver_symbol_collection(root, case, ast);
    assert!(resolver.detail_keys.is_empty());
    let module = resolver.module;
    let symbols = augment_type_elaboration_import_summaries(ast, &module, resolver.env);
    let payload = extract_source_imported_predicate_chain_formula(ast, &module, &symbols).unwrap();
    let mut cases = Vec::new();
    for (name, changed) in [
        ("empty_raw", String::new()),
        (
            "stale_intertoken",
            source.replacen("theorem ", "theorem  ", 1),
        ),
        (
            "hidden_not",
            source.replacen("does not", "does :: not\n", 1),
        ),
        (
            "malformed_import",
            source.replacen("parser.type_fixtures", "parser type_fixtures.", 1),
        ),
    ] {
        cases.push((name, ast.clone(), symbols.clone(), changed, true));
    }
    for trailing in [
        source.trim_end_matches('\n').to_owned(),
        format!("{source}:: trailing\n"),
    ] {
        assert!(
            source_atomic_formula_output_with_source(ast, module.clone(), &symbols, &trailing)
                .expect("unchanged actual trailing token anchors")
                .is_ok()
        );
        assert!(
            source_predicate_chain_composition_output_with_source(
                ast,
                module.clone(),
                &symbols,
                &trailing
            )
            .expect("C3 trailing trivia alias")
            .is_ok()
        );
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let foreign_root = std::env::temp_dir().join(format!(
        "mizar-test-chain-foreign-{}-{nanos}",
        std::process::id()
    ));
    let foreign_path = foreign_root
        .join("src")
        .join(case.source_path.file_name().unwrap());
    std::fs::create_dir_all(foreign_path.parent().unwrap()).unwrap();
    std::fs::write(&foreign_path, &source).unwrap();
    let ids = mizar_session::InMemorySessionIdAllocator::new();
    let snapshot = super::shared::snapshot_id(ordinal + 1);
    mizar_session::SessionIdAllocator::next_source_id(&ids, snapshot).unwrap();
    let foreign = mizar_frontend::orchestration::Frontend::new(
        mizar_frontend::source::FrontendSourceLoader::new(mizar_session::DiskSourceLoader::new(
            &foreign_root,
        )),
        ParseOnlyImportProvider,
        mizar_frontend::parsing::MizarParserSeam,
    )
    .run(
        mizar_frontend::source::SourceUnitRequest {
            snapshot,
            input: mizar_session::SourceInput {
                package_id: module.package().clone(),
                module_path: module.path().clone(),
                normalized_path: mizar_session::normalize_path(&foreign_root, &foreign_path)
                    .unwrap(),
                edition: mizar_session::Edition::new("2026"),
                origin: mizar_session::SourceOriginInput::Disk { path: foreign_path },
            },
        },
        &ids,
    )
    .unwrap();
    std::fs::remove_dir_all(&foreign_root).unwrap();
    assert_eq!(foreign.source.source_text.as_ref(), source);
    assert_eq!(&foreign.source.package_id, module.package());
    assert_eq!(&foreign.source.module_path, module.path());
    assert!(foreign.diagnostics.is_empty());
    let foreign_ast = foreign.ast.as_ref().unwrap();
    assert_ne!(foreign_ast.source_id, ast.source_id);
    let fresh = resolver_symbol_collection(root, case, foreign_ast);
    assert!(fresh.detail_keys.is_empty());
    assert_eq!(fresh.module, module);
    let fresh_symbols = augment_type_elaboration_import_summaries(foreign_ast, &module, fresh.env);
    let foreign_lower = source_atomic_formula_output_with_source(
        foreign_ast,
        module.clone(),
        &fresh_symbols,
        &source,
    )
    .expect("genuinely loaded foreign SourceId selects")
    .expect("fresh actual augmented environment");
    assert_eq!(foreign_lower.typed_ast.source_id(), foreign_ast.source_id);
    let foreign_upper = source_predicate_chain_composition_output_with_source(
        foreign_ast,
        module.clone(),
        &fresh_symbols,
        &source,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        foreign_upper.typed_ast.source_term(),
        foreign_lower.typed_ast.source_term()
    );
    cases.push((
        "stale_foreign_env",
        foreign_ast.clone(),
        symbols.clone(),
        source.clone(),
        true,
    ));
    let mut indexes = super::import_fixtures::clone_symbol_env_indexes(&symbols);
    indexes.contributions.insert(
        module.clone(),
        mizar_resolve::env::ContributionKind::LocalSource {
            source_id: ast.source_id,
        },
        mizar_session::SourceAnchor::Range(ast.node(ast.root().unwrap()).unwrap().range),
    );
    cases.push((
        "unowned_full_env_contribution",
        ast.clone(),
        mizar_resolve::env::SymbolEnv::new(module.clone(), indexes),
        source.clone(),
        true,
    ));
    let divides = symbols.symbols().get(&payload.predicate_symbol).unwrap();
    let local = symbols
        .contributions()
        .iter()
        .find(|entry| {
            matches!(
                entry.kind(),
                mizar_resolve::env::ContributionKind::LocalSource { .. }
            )
        })
        .unwrap()
        .id();
    for corruption in 0..4 {
        let mut indexes = super::import_fixtures::clone_symbol_env_indexes(&symbols);
        indexes.symbols = Default::default();
        for entry in symbols.symbols().iter() {
            if entry.symbol() != divides.symbol() {
                indexes.symbols.insert(entry.clone());
                continue;
            }
            match corruption {
                0 => {}
                1 => {
                    indexes.symbols.insert(entry.clone());
                    indexes.symbols.insert(entry.clone());
                }
                _ => indexes.symbols.insert(
                    mizar_resolve::env::SymbolEntry::new(
                        entry.symbol().clone(),
                        if corruption == 2 {
                            mizar_resolve::env::SymbolKind::Functor
                        } else {
                            entry.kind()
                        },
                        entry.namespace().clone(),
                        entry.primary_spelling(),
                        entry.origin().clone(),
                        if corruption == 3 {
                            local
                        } else {
                            entry.contribution()
                        },
                    )
                    .with_visibility(entry.visibility())
                    .with_export_status(entry.export_status()),
                ),
            }
        }
        cases.push((
            [
                "missing_divides",
                "ambiguous_divides",
                "wrong_divides_kind",
                "wrong_divides_provenance",
            ][corruption],
            ast.clone(),
            mizar_resolve::env::SymbolEnv::new(module.clone(), indexes),
            source.clone(),
            false,
        ));
    }
    let formula = payload.formula_site.node().index();
    let formula_wrapper = ast
        .nodes()
        .iter()
        .position(|node| {
            matches!(node.kind, K::FormulaExpression)
                && node.children.iter().any(|child| child.index() == formula)
        })
        .unwrap();
    let term = payload.term_sites[0].node().index();
    let term_wrapper = ast
        .nodes()
        .iter()
        .position(|node| {
            matches!(node.kind, K::TermExpression)
                && node.children.iter().any(|child| child.index() == term)
        })
        .unwrap();
    for corruption in 0..8 {
        let mut changed = source.clone();
        if corruption == 1 {
            changed.push(';');
        }
        let start = source.find("does not divides 3").unwrap();
        if corruption == 7 {
            changed.replace_range(
                start..start + "does not divides 3".len(),
                "divides does not 3",
            );
        }
        let mut tokens = ast
            .token_views()
            .map(|node| {
                let mut range = node.range();
                if corruption == 7 {
                    for (from, end, to) in [(0, 4, 8), (5, 8, 13), (9, 16, 0)] {
                        if range.start == start + from && range.end == start + end {
                            range.start = start + to;
                            range.end = range.start + end - from;
                            break;
                        }
                    }
                }
                (node.id(), node.as_token().unwrap().clone(), range)
            })
            .collect::<Vec<_>>();
        tokens.sort_by_key(|(_, _, range)| range.start);
        let mut builder = mizar_syntax::SurfaceAstBuilder::new(ast.source_id);
        let mut rebuilt = vec![None; ast.nodes().len()];
        let mut ranges = ast
            .nodes()
            .iter()
            .map(|node| node.range)
            .collect::<Vec<_>>();
        let mut global = Vec::new();
        for (id, token, range) in tokens {
            let target = range == payload.head_ranges[0];
            let token_id = if corruption == 3 && target {
                builder.add_recovered_token(token.kind, token.text, range)
            } else {
                builder.add_token(
                    if corruption == 0 && target {
                        T::Identifier
                    } else {
                        token.kind
                    },
                    token.text,
                    range,
                )
            };
            rebuilt[id.index()] = Some(token_id);
            ranges[id.index()] = range;
            global.push(token_id);
        }
        for (index, node) in ast.nodes().iter().enumerate() {
            if matches!(node.kind, K::Token(_)) {
                continue;
            }
            let mut children = node.children.clone();
            children.sort_by_key(|child| ranges[child.index()].start);
            let mut range = node.range;
            let children = if Some(index) == ast.root().map(|id| id.index()) {
                if corruption == 1 {
                    global.push(builder.add_token(
                        T::ReservedSymbol,
                        ";",
                        mizar_session::SourceRange {
                            source_id: ast.source_id,
                            start: source.len(),
                            end: changed.len(),
                        },
                    ));
                    range.end = changed.len();
                }
                let compilation = node
                    .children
                    .iter()
                    .find(|child| matches!(ast.node(**child).unwrap().kind, K::CompilationUnit))
                    .unwrap();
                global
                    .iter()
                    .copied()
                    .chain([rebuilt[compilation.index()].unwrap()])
                    .collect()
            } else {
                range.start = ranges[children.first().unwrap().index()].start;
                range.end = ranges[children.last().unwrap().index()].end;
                children
                    .iter()
                    .map(|child| rebuilt[child.index()].unwrap())
                    .collect()
            };
            let kind = if corruption == 4 && index == formula_wrapper {
                K::TermExpression
            } else if (corruption == 5 && index == formula)
                || (corruption == 6 && index == term_wrapper)
            {
                K::FormulaExpression
            } else {
                node.kind.clone()
            };
            rebuilt[index] = Some(builder.add_node(kind, range, children));
            ranges[index] = range;
        }
        if corruption == 2 {
            builder.add_node(
                K::TermExpression,
                ast.nodes()[term].range,
                vec![rebuilt[term].unwrap()],
            );
        }
        let forged = builder.finish(ast.root().map(|id| rebuilt[id.index()].unwrap()), None);
        assert!(forged.canonical_bytes().is_some());
        assert!(
            forged
                .node_views()
                .all(|node| forged.range_contains_child_ranges(node.id()) == Some(true))
        );
        if corruption != 3 {
            assert!(forged.node_views().all(|node| !node.is_recovered()));
        }
        let lexed = task257c4c1_frontend_output(&changed, 265);
        assert!(lexed.tokens.diagnostics.is_empty());
        assert_eq!(lexed.tokens.tokens().len(), forged.token_nodes().len());
        for (actual, supplied) in lexed.tokens.tokens().iter().zip(forged.token_views()) {
            let kind = match actual.kind {
                mizar_frontend::lexing::TokenKind::Identifier => T::Identifier,
                mizar_frontend::lexing::TokenKind::ReservedWord => T::ReservedWord,
                mizar_frontend::lexing::TokenKind::ReservedSymbol => T::ReservedSymbol,
                mizar_frontend::lexing::TokenKind::Numeral => T::Numeral,
                mizar_frontend::lexing::TokenKind::UserSymbol => T::UserSymbol,
                _ => panic!("unexpected actual chain token"),
            };
            assert_eq!(
                (actual.text.as_ref(), actual.span.start, actual.span.end),
                (
                    supplied.as_token().unwrap().text.as_ref(),
                    supplied.range().start,
                    supplied.range().end
                )
            );
            assert_eq!(
                changed.get(supplied.range().start..supplied.range().end),
                Some(actual.text.as_ref())
            );
            if corruption == 0 && supplied.range() == payload.head_ranges[0] {
                assert_eq!(
                    (kind, supplied.as_token().unwrap().kind),
                    (T::UserSymbol, T::Identifier)
                );
            } else {
                assert_eq!(kind, supplied.as_token().unwrap().kind);
            }
        }
        let mut owners = Vec::new();
        for node in forged
            .node_views()
            .filter(|node| !matches!(node.kind(), K::Token(_)) && Some(node.id()) != forged.root())
        {
            let children = node.child_views().collect::<Vec<_>>();
            assert!(
                children
                    .windows(2)
                    .all(|pair| pair[0].range().end <= pair[1].range().start)
            );
            owners.extend(
                children
                    .iter()
                    .filter(|child| child.as_token().is_some())
                    .map(|child| child.id()),
            );
        }
        owners.sort();
        if corruption == 1 {
            assert_eq!(owners.len() + 1, forged.token_nodes().len());
        } else {
            assert_eq!(owners, forged.token_nodes());
        }
        let mut reachable = vec![false; forged.nodes().len()];
        let mut pending = vec![forged.root().unwrap()];
        while let Some(id) = pending.pop() {
            if !std::mem::replace(&mut reachable[id.index()], true) {
                pending.extend(forged.node(id).unwrap().children.iter().copied());
            }
        }
        assert_eq!(
            reachable.iter().filter(|visited| !**visited).count(),
            usize::from(corruption == 2)
        );
        let fresh = resolver_symbol_collection(root, case, &forged);
        if matches!(corruption, 0 | 1 | 2 | 7) {
            assert!(fresh.detail_keys.is_empty());
        }
        let env = augment_type_elaboration_import_summaries(&forged, &module, fresh.env);
        cases.push((
            [
                "wrong_user_symbol_kind",
                "global_only_token",
                "unreachable_role",
                "recovered_head",
                "wrong_theorem_wrapper",
                "wrong_application_role",
                "wrong_term_wrapper",
                "does_not_after_head",
            ][corruption],
            forged,
            env,
            changed,
            matches!(corruption, 0 | 1 | 2 | 7),
        ));
    }
    for (name, ast, env, raw, selected) in cases {
        assert_eq!(
            extract_source_imported_predicate_chain_formula(&ast, &module, &env).is_some(),
            selected,
            "actual selection prerequisite {name}"
        );
        let lower = source_atomic_formula_output_with_source_and_mutation(
            &ast,
            module.clone(),
            &env,
            &raw,
            |_| panic!("source seal must refuse {name} before Atomic producer"),
        );
        let upper =
            source_predicate_chain_composition_output_with_source(&ast, module.clone(), &env, &raw);
        if selected {
            let error = lower
                .expect("selected source error, never skip")
                .expect_err("actual raw/full-env seal refusal");
            assert_eq!(
                upper
                    .expect("C3 preserves selected error")
                    .expect_err("C3 must not publish"),
                error,
                "same Atomic error propagated for {name}"
            );
        } else {
            assert!(lower.is_none(), "unsupported prerequisite {name}");
            assert!(upper.is_none(), "unsupported C3 prerequisite {name}");
        }
    }
}

#[test]
fn task257c1_exact_route_publishes_predicate_segments_and_shared_boundary() {
    let (ast, module, symbols) = task252_real_ast(TASK257C1_CASE);
    let payload =
        extract_source_imported_predicate_chain_formula(&ast, &module, &symbols)
            .expect("Task 257C1 exact selector payload");
    assert_eq!((payload.formula_range.start, payload.formula_range.end), (75, 105));
    assert_eq!(
        payload
            .segment_ranges
            .map(|range| (range.start, range.end)),
        [(75, 86), (87, 105)]
    );
    assert_eq!(
        payload.head_ranges.map(|range| (range.start, range.end)),
        [(77, 84), (96, 103)]
    );
    assert_eq!(
        payload.term_ranges.map(|range| (range.start, range.end)),
        [(75, 76), (85, 86), (104, 105)]
    );
    assert_eq!((payload.verb_range.start, payload.verb_range.end), (87, 91));
    assert_eq!((payload.not_range.start, payload.not_range.end), (92, 95));

    let first =
        source_atomic_formula_output_with_source(&ast, module.clone(), &symbols, TASK257C1_SOURCE)
            .expect("Task 257C1 exact selector")
            .expect("Task 257C1 source transaction");
    let second =
        source_atomic_formula_output_with_source(&ast, module, &symbols, TASK257C1_SOURCE)
            .expect("Task 257C1 repeated selector")
            .expect("Task 257C1 repeated transaction");
    let handoff = task256_handoff(&first);
    assert_eq!(
        (
            handoff.formulas().len(),
            handoff.wrappers().len(),
            handoff.predicate_segments().len(),
            handoff.predicate_heads().len(),
            handoff.candidates().len(),
            handoff.type_sites().len(),
            handoff.attributes().len(),
            handoff.edges().len(),
            handoff.requests().len(),
        ),
        (1, 0, 2, 2, 2, 0, 0, 3, 2)
    );
    let primary = first.typed_ast.source_term().expect("Task 252 handoff");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (3, 0, 3)
    );
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(_, term)| (term.source_range().start, term.source_range().end))
            .collect::<Vec<_>>(),
        [(75, 76), (85, 86), (104, 105)]
    );

    let segments = handoff
        .predicate_segments()
        .iter()
        .map(|(_, segment)| {
            (
                segment.ordinal(),
                segment.source_range(),
                segment.spelling(),
                segment.head(),
                segment.left_edge(),
                segment.right_edge(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        segments,
        [
            (
                0,
                payload.segment_ranges[0],
                "1 divides 2",
                mizar_checker::source_atomic_formula::SourcePredicateHeadId::new(0),
                mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(0),
                mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(1),
            ),
            (
                1,
                payload.segment_ranges[1],
                "does not divides 3",
                mizar_checker::source_atomic_formula::SourcePredicateHeadId::new(1),
                mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(1),
                mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(2),
            ),
        ]
    );
    let first_segment = handoff
        .predicate_segments()
        .get(mizar_checker::source_atomic_formula::SourcePredicateSegmentId::new(0))
        .expect("positive segment");
    assert!(matches!(
        first_segment.polarity(),
        mizar_checker::source_atomic_formula::SourcePredicateSegmentPolarityInput::Positive
    ));
    let second_segment = handoff
        .predicate_segments()
        .get(mizar_checker::source_atomic_formula::SourcePredicateSegmentId::new(1))
        .expect("negative segment");
    assert!(matches!(
        second_segment.polarity(),
        mizar_checker::source_atomic_formula::SourcePredicateSegmentPolarityInput::Negative {
            verb_range,
            verb_spelling,
            not_range,
            not_spelling,
            ..
        } if *verb_range == payload.verb_range
            && verb_spelling == "does"
            && *not_range == payload.not_range
            && not_spelling == "not"
    ));

    let candidates = handoff.candidates().iter().collect::<Vec<_>>();
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].1.symbol(), candidates[1].1.symbol());
    assert_eq!(
        candidates[0].1.contribution(),
        candidates[1].1.contribution()
    );
    assert_eq!(candidates[0].1.symbol(), &payload.predicate_symbol);
    assert!(matches!(
        symbols
            .contributions()
            .get(candidates[0].1.contribution())
            .expect("imported predicate contribution")
            .kind(),
        mizar_resolve::env::ContributionKind::ImportedSource { .. }
    ));
    assert_eq!(
        handoff
            .edges()
            .iter()
            .map(|(_, edge)| (edge.role(), edge.target()))
            .collect::<Vec<_>>(),
        [
            (
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::PredicateLeftArgument,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(0),
                ),
            ),
            (
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::PredicateChainBoundary,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(1),
                ),
            ),
            (
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::PredicateRightArgument,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2),
                ),
            ),
        ]
    );
    assert_eq!(
        handoff
            .requests()
            .iter()
            .map(|(_, request)| (request.kind(), request.candidate()))
            .collect::<Vec<_>>(),
        [
            (
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::PredicateCandidateSignature,
                Some(mizar_checker::source_atomic_formula::SourcePredicateCandidateId::new(0)),
            ),
            (
                mizar_checker::source_atomic_formula::SourceAtomicRequestKind::PredicateCandidateSignature,
                Some(mizar_checker::source_atomic_formula::SourcePredicateCandidateId::new(1)),
            ),
        ]
    );
    assert!(first.typed_ast.source_application().is_none());
    assert_eq!(
        first.typed_ast.source_atomic_formula(),
        first.resolved.source_atomic_formula()
    );
    assert_eq!(first.typed_ast.source_term(), first.resolved.source_term());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    assert_eq!(
        handoff.primary_term_fingerprint(),
        primary.debug_text(),
        "Task 252 fingerprint must clone-preserve"
    );
    assert!(
        handoff
            .debug_text()
            .contains("role=predicate-chain-boundary target=primary:1")
    );
    assert!(
        source_atomic_formula_output(&ast, first.typed_ast.module_id().clone(), &symbols).is_none(),
        "Task 257C1 must require actual source input"
    );
}

#[test]
fn task257c1_corruption_fails_atomically_and_valid_clone_recovers() {
    let (ast, module, symbols) = task252_real_ast(TASK257C1_CASE);
    type Mutation =
        fn(&mut mizar_checker::source_atomic_formula::SourceAtomicFormulaHandoffInput);
    let corruptions: &[Mutation] = &[
        |input| input.predicate_segments.clear(),
        |input| input.predicate_segments[1].ordinal = 0,
        |input| input.predicate_segments[1].source_range.start = 86,
        |input| input.predicate_segments[1].spelling = "divides 3".to_owned(),
        |input| input.predicate_segments[1].head =
            mizar_checker::source_atomic_formula::SourcePredicateHeadId::new(0),
        |input| input.predicate_segments[1].left_edge =
            mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(0),
        |input| {
            if let mizar_checker::source_atomic_formula::SourcePredicateSegmentPolarityInput::Negative {
                not_spelling,
                ..
            } = &mut input.predicate_segments[1].polarity
            {
                *not_spelling = "non".to_owned();
            }
        },
        |input| input.predicate_heads[1].left_arity = 2,
        |input| input.candidates[1].head =
            mizar_checker::source_atomic_formula::SourcePredicateHeadId::new(0),
        |input| {
            input.edges[1].role =
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::PredicateRightArgument
        },
        |input| {
            input.requests[1].candidate = Some(
                mizar_checker::source_atomic_formula::SourcePredicateCandidateId::new(0),
            )
        },
    ];
    for corrupt in corruptions {
        assert!(
            source_atomic_formula_output_with_source_and_mutation(
                &ast,
                module.clone(),
                &symbols,
                TASK257C1_SOURCE,
                corrupt,
            )
            .expect("corruption preserves the exact selector")
            .is_err()
        );
    }
    let valid =
        source_atomic_formula_output_with_source(&ast, module, &symbols, TASK257C1_SOURCE)
            .expect("valid selector after corruption")
            .expect("valid transaction after corruption");
    assert_eq!(
        valid.typed_ast.source_atomic_formula(),
        valid.resolved.source_atomic_formula()
    );
}

#[test]
fn task257c1_source_selector_is_exact_and_active_corpus_adds_only_one_route() {
    let near_misses = [
        (
            "changed left numeral",
            TASK257C1_SOURCE.replacen("1 divides 2", "10 divides 2", 1),
        ),
        (
            "changed middle numeral",
            TASK257C1_SOURCE.replacen("1 divides 2", "1 divides 20", 1),
        ),
        (
            "changed label",
            TASK257C1_SOURCE.replacen(
                "FormulaPredicateChainPayloadBoundary",
                "AnotherPredicateChainPayloadBoundary",
                1,
            ),
        ),
        (
            "changed import",
            TASK257C1_SOURCE.replacen("parser.type_fixtures", "parser.other_fixtures", 1),
        ),
        (
            "another compilation item",
            format!("{TASK257C1_SOURCE}theorem ExtraItem: 0 = 0;\n"),
        ),
        (
            "theorem status",
            TASK257C1_SOURCE.replacen("theorem ", "open theorem ", 1),
        ),
        (
            "theorem justification",
            TASK257C1_SOURCE.replacen(
                "3;\n",
                "3 by FormulaPredicateChainPayloadBoundary;\n",
                1,
            ),
        ),
        (
            "predicate substitution",
            TASK257C1_SOURCE.replace("divides", "precedes"),
        ),
        (
            "changed term order",
            TASK257C1_SOURCE.replacen("1 divides 2", "2 divides 1", 1),
        ),
        (
            "changed term value",
            TASK257C1_SOURCE.replacen("divides 3", "divides 4", 1),
        ),
        (
            "term-count expansion",
            TASK257C1_SOURCE.replacen("1 divides 2", "1, 2 divides 3", 1),
        ),
        (
            "missing segment",
            TASK257C1_SOURCE.replacen(" divides 2 does not divides 3", " divides 2", 1),
        ),
        (
            "extra segment",
            TASK257C1_SOURCE.replacen(
                "does not divides 3",
                "does not divides 3 divides 4",
                1,
            ),
        ),
        (
            "positive-only chain",
            TASK257C1_SOURCE.replacen("does not divides", "divides", 1),
        ),
        (
            "changed negation",
            TASK257C1_SOURCE.replacen("does not", "does never", 1),
        ),
        (
            "do-not negation",
            TASK257C1_SOURCE.replacen("does not", "do not", 1),
        ),
        (
            "prefix notation",
            TASK257C1_SOURCE.replacen("1 divides 2", "divides 1, 2", 1),
        ),
        (
            "postfix notation",
            TASK257C1_SOURCE.replacen("1 divides 2", "1, 2 divides", 1),
        ),
        (
            "multi-argument notation",
            TASK257C1_SOURCE.replacen("1 divides 2", "1, 2 divides 3", 1),
        ),
        (
            "template predicate",
            TASK257C1_SOURCE.replacen("divides 2", "divides[parser.type_fixtures] 2", 1),
        ),
        (
            "inline predicate",
            TASK257C1_SOURCE.replacen(
                "1 divides 2 does not divides 3",
                "InlinePredicate(1)",
                1,
            ),
        ),
        (
            "local predicate",
            TASK257C1_SOURCE.replacen(
                "1 divides 2 does not divides 3",
                "LocalPredicate(1)",
                1,
            ),
        ),
        (
            "built-in predicate",
            TASK257C1_SOURCE.replacen("1 divides 2 does not divides 3", "1 = 2", 1),
        ),
        (
            "mixed built-in chain",
            TASK257C1_SOURCE.replacen("does not divides 3", "= 3", 1),
        ),
        (
            "parenthesized wrapper",
            TASK257C1_SOURCE.replacen(
                "1 divides 2 does not divides 3",
                "(1 divides 2 does not divides 3)",
                1,
            ),
        ),
        (
            "formula connective",
            TASK257C1_SOURCE.replacen(
                "1 divides 2 does not divides 3",
                "1 divides 2 does not divides 3 & 0 = 0",
                1,
            ),
        ),
        (
            "formula quantifier",
            TASK257C1_SOURCE.replacen(
                "1 divides 2 does not divides 3",
                "for x being set holds 1 divides 2 does not divides 3",
                1,
            ),
        ),
        (
            "conditioned comprehension",
            TASK257C1_SOURCE.replacen(
                "1 divides 2 does not divides 3",
                "{x where x is set : x = x} = {x where x is set : x = x}",
                1,
            ),
        ),
        (
            "recovered missing chain term",
            TASK257C1_SOURCE.replacen("divides 3;", "divides ;", 1),
        ),
    ];
    for (ordinal, (label, source)) in near_misses.into_iter().enumerate() {
        let (ast, module, _, symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 22_000 + ordinal);
        if ordinal < 2 {
            assert_eq!(diagnostics, 0, "normal unsupported numeral {label}");
            assert!(ast.node_views().all(|node| !node.is_recovered()));
        }
        let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
        assert!(
            extract_source_imported_predicate_chain_formula(&ast, &module, &symbols)
                .is_none(),
            "{label} unexpectedly selected"
        );
        assert!(
            source_atomic_formula_output_with_source(&ast, module, &symbols, &source).is_none(),
            "{label} unexpectedly reached production"
        );
    }

    let (exact_ast, exact_module, exact_symbols) = task252_real_ast(TASK257C1_CASE);
    assert!(extract_source_imported_predicate_chain_formula(&exact_ast, &exact_module, &exact_symbols).is_some());
    for (label, loaded_text) in [
        (
            "exact AST with stale leading layout",
            format!("\n{TASK257C1_SOURCE}"),
        ),
        (
            "exact AST with mismatched loaded text",
            TASK257C1_SOURCE.replacen("divides 3", "divides 4", 1),
        ),
    ] {
        assert!(
            source_atomic_formula_output_with_source(
                &exact_ast,
                exact_module.clone(),
                &exact_symbols,
                &loaded_text,
            )
            .expect("selected chain raw mismatch").is_err(),
            "{label}"
        );
        assert!(source_predicate_chain_composition_output_with_source(&exact_ast, exact_module.clone(), &exact_symbols, &loaded_text).expect("C3 propagates selected Atomic error").is_err(), "{label}");
    }

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
    let plan = build_test_plan(&config).expect("Task 257C1 repository plan should build");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source_text = frontend.source_text;
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if source_atomic_formula_output_with_source(
            &ast,
            resolver.module,
            &symbols,
            &source_text,
        )
        .is_some()
        {
            selected.push(case.id.0.clone());
        }
    }
    let mut expected = TASK256_CASES
        .iter()
        .map(|(id, _)| (*id).to_owned())
        .chain([TASK257C1_CASE.to_owned()])
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(selected, expected);
}

#[test]
fn task257c1_active_pass_keeps_external_detail_vector_empty() {
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
    let plan = build_test_plan(&config).expect("Task 257C1 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK257C1_CASE)
        .expect("Task 257C1 remains active");
    let result = run_type_elaboration_case(&workspace_root, &tests_root, case, ordinal);
    assert_eq!(
        result.actual_detail_keys,
        expected_type_elaboration_detail_keys(case),
        "Task 257C1 external detail keys"
    );
    assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
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
