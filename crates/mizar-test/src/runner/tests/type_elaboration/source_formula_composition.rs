use super::{
    SourceFormulaCompositionRouteInputs, SourceFormulaCompositionRouteOutput,
    extract_source_formula_connective_grouping, extract_source_formula_quantifier_bound_use,
    extract_source_formula_nested_quantifier_payload, source_formula_composition_output,
    source_formula_composition_output_with_mutation,
    source_formula_composition_output_with_source,
    source_formula_composition_output_with_source_and_mutation,
};

const TASK257B1_CASE: &str = "pass_type_elaboration_formula_quantifier_bound_use_payload_001";
const TASK257B2_CASE: &str = "pass_type_elaboration_formula_connective_grouping_payload_001";
const TASK257B3_CASE: &str = "pass_type_elaboration_formula_nested_quantifier_payload_001";
const TASK257B3_SOURCE: &str = "reserve r for set; theorem FormulaNestedQuantifierPayloadBoundary: for x being set st x = x ex y being set st for r st r = y holds x = r;\n";

#[test]
fn task257b1_real_route_publishes_the_exact_cross_family_transaction() {
    let (ast, module, symbols) = task252_real_ast(TASK257B1_CASE);
    let payload = extract_source_formula_quantifier_bound_use(&ast, &module, &symbols)
        .expect("Task 257B1 selector payload");
    assert_eq!(
        [
            payload.quantified_range.start,
            payload.quantified_range.end,
            payload.binder_segment_range.start,
            payload.binder_segment_range.end,
            payload.binder_identifier_range.start,
            payload.binder_identifier_range.end,
            payload.binder_type_range.start,
            payload.binder_type_range.end,
            payload.binder_type_head_range.start,
            payload.binder_type_head_range.end,
            payload.equality_range.start,
            payload.equality_range.end,
            payload.left_range.start,
            payload.left_range.end,
            payload.right_range.start,
            payload.right_range.end,
        ],
        [
            50, 77, 54, 65, 54, 55, 62, 65, 62, 65, 72, 77, 72, 73, 76, 77
        ]
    );

    let first = source_formula_composition_output(&ast, module.clone(), &symbols)
        .expect("Task 257B1 selects")
        .expect("Task 257B1 transaction");
    let second = source_formula_composition_output(&ast, module, &symbols)
        .expect("Task 257B1 remains selected")
        .expect("Task 257B1 replay");
    let primary = first.typed_ast.source_term().expect("Task 252 handoff");
    let atomic = first
        .typed_ast
        .source_atomic_formula()
        .expect("Task 256 handoff");
    let composite = first
        .typed_ast
        .source_composite_formula()
        .expect("Task 257 handoff");
    let composition = task257b1_handoff(&first);

    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (2, 2, 0)
    );
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.wrappers().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.attributes().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (1, 0, 0, 0, 0, 0, 2, 2)
    );
    assert_eq!(
        (
            composite.formulas().len(),
            composite.wrappers().len(),
            composite.roots().len(),
            composite.binders().len(),
            composite.type_sites().len(),
            composite.edges().len(),
            composite.requests().len(),
        ),
        (1, 0, 1, 1, 1, 0, 2)
    );
    assert_eq!(
        (
            composite.binding_env().contexts().len(),
            composite.binding_env().bindings().len(),
            composite.binding_env().diagnostics().len(),
        ),
        (2, 1, 4)
    );
    assert!(
        composite
            .binding_env()
            .bindings()
            .get(mizar_checker::binding_env::BindingId::new(0))
            .expect("quantifier binding")
            .captured
            .identities()
            .is_empty()
    );
    assert_eq!(
        (
            composition.atomic_edges().len(),
            composition.bound_uses().len()
        ),
        (1, 2)
    );
    assert_eq!(composition.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(
        composition.atomic_formula_fingerprint(),
        atomic.debug_text()
    );
    assert_eq!(
        composition.composite_formula_fingerprint(),
        composite.debug_text()
    );
    assert!(first.typed_ast.source_context().is_none());
    assert_eq!(
        first.typed_ast.source_formula_composition(),
        first.resolved.source_formula_composition()
    );
    assert_eq!(
        first.typed_ast.source_formula_composition(),
        Some(composition)
    );
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
}

#[test]
fn task257b1_bound_uses_preserve_reference_ownership_and_source_order() {
    let (ast, module, symbols) = task252_real_ast(TASK257B1_CASE);
    let output = source_formula_composition_output(&ast, module, &symbols)
        .expect("Task 257B1 selects")
        .expect("Task 257B1 transaction");
    let primary = output.typed_ast.source_term().expect("Task 252 handoff");
    let atomic = output
        .typed_ast
        .source_atomic_formula()
        .expect("Task 256 handoff");
    let composition = task257b1_handoff(&output);

    let references = primary
        .references()
        .iter()
        .map(|(id, row)| {
            (
                id.index(),
                row.term().index(),
                row.binding().index(),
                row.use_ordinal(),
                row.lexical_scope().map(|scope| scope.path().to_vec()),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        references,
        [(0, 0, 0, 1, Some(vec![0])), (1, 1, 0, 1, Some(vec![0])),]
    );
    let atomic_edges = atomic
        .edges()
        .iter()
        .map(|(_, row)| (row.ordinal(), row.role(), row.target()))
        .collect::<Vec<_>>();
    assert_eq!(
        atomic_edges,
        [
            (
                0,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(0),
                ),
            ),
            (
                1,
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(1),
                ),
            ),
        ]
    );
    let bound_uses = composition
        .bound_uses()
        .iter()
        .map(|(_, row)| {
            (
                row.binder().index(),
                row.ordinal(),
                row.body_edge().index(),
                row.term().index(),
                row.reference().index(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(bound_uses, [(0, 0, 0, 0, 0), (0, 1, 0, 1, 1)]);
}

#[test]
fn task257b1_corruption_matrix_fails_closed_and_valid_replay_recovers() {
    let (ast, module, symbols) = task252_real_ast(TASK257B1_CASE);
    let corruptions: [fn(&mut SourceFormulaCompositionRouteInputs); 8] = [
        |input| {
            input.primary.terms[0].context = mizar_checker::binding_env::BindingContextId::new(0)
        },
        |input| input.primary.references.swap(0, 1),
        |input| input.atomic.formulas[0].spelling = "x <> x".to_owned(),
        |input| input.atomic.edges.swap(0, 1),
        |input| input.composite.formulas[0].spelling = "ex holds".to_owned(),
        |input| input.composite.requests.pop().map_or((), drop),
        |input| {
            input.composition.atomic_edges[0].child =
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1)
        },
        |input| input.composition.bound_uses.swap(0, 1),
    ];
    for corrupt in corruptions {
        assert!(
            source_formula_composition_output_with_mutation(
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
        source_formula_composition_output(&ast, module, &symbols)
            .expect("valid route remains selected")
            .is_ok()
    );
}

#[test]
fn task257b_selectors_are_exclusive_and_do_not_capture_task257a() {
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
    let plan = build_test_plan(&config).expect("Task 257B1 plan");
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
        if source_formula_composition_output_with_source(
            &ast,
            resolver.module,
            &symbols,
            &frontend.source_text,
        )
        .is_some()
        {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(
        selected,
        [
            TASK257B2_CASE.to_owned(),
            TASK257B3_CASE.to_owned(),
            TASK257B1_CASE.to_owned(),
        ]
    );

    let (task257a_ast, task257a_module, task257a_symbols) =
        task252_real_ast("fail_type_elaboration_formula_connective_quantifier_gap_001");
    assert!(
        extract_source_formula_nested_quantifier_payload(
            &task257a_ast,
            &task257a_module,
            &task257a_symbols,
            "",
        )
        .is_none()
    );
    assert!(
        source_formula_composition_output(&task257a_ast, task257a_module, &task257a_symbols)
            .is_none()
    );
    let (ast, module, symbols) = task252_real_ast(TASK257B1_CASE);
    assert!(
        extract_source_formula_nested_quantifier_payload(&ast, &module, &symbols, "").is_none()
    );
    assert!(source_composite_formula_output(&ast, module, &symbols).is_none());
    let (ast, module, symbols) = task252_real_ast(TASK257B2_CASE);
    assert!(
        extract_source_formula_nested_quantifier_payload(&ast, &module, &symbols, "").is_none()
    );
    assert!(
        source_formula_composition_output(&ast, module, &symbols)
            .expect("Task 257B2 remains selected")
            .is_ok()
    );
}

#[test]
fn task257b2_real_route_publishes_exact_profiles_ranges_and_final_ownership() {
    let (ast, module, symbols) = task252_real_ast(TASK257B2_CASE);
    let payload = extract_source_formula_connective_grouping(&ast, &module, &symbols)
        .expect("Task 257B2 selector payload");
    assert_eq!(
        payload
            .formula_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (50, 164),
            (72, 164),
            (73, 121),
            (74, 93),
            (99, 120),
            (128, 163),
            (129, 142),
            (148, 162),
        ]
    );
    assert_eq!(
        payload
            .wrapper_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (72, 122),
            (73, 94),
            (98, 121),
            (127, 164),
            (128, 143),
            (147, 163),
        ]
    );
    assert_eq!(
        payload
            .equality_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (74, 79),
            (88, 93),
            (99, 104),
            (115, 120),
            (129, 134),
            (137, 142),
            (148, 153),
            (157, 162),
        ]
    );
    assert_eq!(
        payload
            .numeral_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (74, 75),
            (78, 79),
            (88, 89),
            (92, 93),
            (99, 100),
            (103, 104),
            (115, 116),
            (119, 120),
            (129, 130),
            (133, 134),
            (137, 138),
            (141, 142),
            (148, 149),
            (152, 153),
            (157, 158),
            (161, 162),
        ]
    );
    assert_eq!(
        payload.numeral_spellings,
        [
            "0", "0", "0", "3", "0", "0", "0", "3", "0", "0", "0", "0", "0", "0", "0",
            "0",
        ]
    );
    assert_eq!(
        (
            payload.binder_segment_range.start,
            payload.binder_segment_range.end,
            payload.binder_identifier_range.start,
            payload.binder_identifier_range.end,
            payload.binder_type_range.start,
            payload.binder_type_range.end,
            payload.binder_type_head_range.start,
            payload.binder_type_head_range.end,
        ),
        (54, 65, 54, 55, 62, 65, 62, 65)
    );

    let first = source_formula_composition_output(&ast, module.clone(), &symbols)
        .expect("Task 257B2 selects")
        .expect("Task 257B2 transaction");
    let second = source_formula_composition_output(&ast, module, &symbols)
        .expect("Task 257B2 remains selected")
        .expect("Task 257B2 replay");
    let primary = first.typed_ast.source_term().expect("Task 252 handoff");
    let atomic = first
        .typed_ast
        .source_atomic_formula()
        .expect("Task 256 handoff");
    let composite = first
        .typed_ast
        .source_composite_formula()
        .expect("Task 257 handoff");
    let composition = task257b1_handoff(&first);
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (16, 0, 16)
    );
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.wrappers().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.attributes().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (8, 0, 0, 0, 0, 0, 16, 16)
    );
    assert_eq!(
        (
            composite.formulas().len(),
            composite.wrappers().len(),
            composite.roots().len(),
            composite.binders().len(),
            composite.type_sites().len(),
            composite.edges().len(),
            composite.requests().len(),
        ),
        (8, 6, 1, 1, 1, 7, 9)
    );
    assert_eq!(
        (
            composition.atomic_edges().len(),
            composition.bound_uses().len()
        ),
        (8, 0)
    );
    assert_eq!(
        (
            composite.binding_env().contexts().len(),
            composite.binding_env().bindings().len(),
            composite.binding_env().diagnostics().len(),
        ),
        (2, 1, 4)
    );
    assert!(
        composite
            .binding_env()
            .bindings()
            .get(mizar_checker::binding_env::BindingId::new(0))
            .expect("unused explicit binder")
            .captured
            .identities()
            .is_empty()
    );
    assert_eq!(composition.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(
        composition.atomic_formula_fingerprint(),
        atomic.debug_text()
    );
    assert_eq!(
        composition.composite_formula_fingerprint(),
        composite.debug_text()
    );
    assert_eq!(
        first.typed_ast.source_formula_composition(),
        first.resolved.source_formula_composition()
    );
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
}

#[test]
fn task257b2_formula_wrapper_and_atomic_edges_are_exact() {
    let (ast, module, symbols) = task252_real_ast(TASK257B2_CASE);
    let output = source_formula_composition_output(&ast, module, &symbols)
        .expect("Task 257B2 selects")
        .expect("Task 257B2 transaction");
    let composite = output
        .typed_ast
        .source_composite_formula()
        .expect("composite");
    assert_eq!(
        composite
            .formulas()
            .iter()
            .map(|(_, row)| (row.kind(), row.context().index(), row.spelling()))
            .collect::<Vec<_>>(),
        [
            (
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Universal,
                0,
                "for holds"
            ),
            (
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Biconditional,
                1,
                "iff"
            ),
            (
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Disjunction,
                1,
                "or"
            ),
            (
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::RepeatedConjunction,
                1,
                "& ... &"
            ),
            (
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::RepeatedDisjunction,
                1,
                "or ... or"
            ),
            (
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Disjunction,
                1,
                "or"
            ),
            (
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Conjunction,
                1,
                "&"
            ),
            (
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Disjunction,
                1,
                "or"
            ),
        ]
    );
    assert_eq!(
        composite
            .wrappers()
            .iter()
            .map(|(_, row)| {
                (
                    row.formula().index(),
                    row.ordinal(),
                    row.source_range().start,
                    row.source_range().end,
                    row.spelling(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (2, 0, 72, 122, "( or )"),
            (3, 0, 73, 94, "( & ... & )"),
            (4, 0, 98, 121, "( or ... or )"),
            (5, 0, 127, 164, "( or )"),
            (6, 0, 128, 143, "( & )"),
            (7, 0, 147, 163, "( or )"),
        ]
    );
    assert_eq!(
        composite
            .edges()
            .iter()
            .map(|(_, row)| {
                (
                    row.parent().index(),
                    row.ordinal(),
                    row.role(),
                    row.child().index(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::UniversalBody,
                1
            ),
            (
                1,
                0,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::BiconditionalLeft,
                2
            ),
            (
                1,
                1,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::BiconditionalRight,
                5
            ),
            (
                2,
                0,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::DisjunctionLeft,
                3
            ),
            (
                2,
                1,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::DisjunctionRight,
                4
            ),
            (
                5,
                0,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::DisjunctionLeft,
                6
            ),
            (
                5,
                1,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::DisjunctionRight,
                7
            ),
        ]
    );
    assert_eq!(
        task257b1_handoff(&output)
            .atomic_edges()
            .iter()
            .map(|(_, row)| {
                (
                    row.formula().index(),
                    row.ordinal(),
                    row.role(),
                    row.child().index(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                3,
                0,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::ConjunctionLeft,
                0
            ),
            (
                3,
                1,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::ConjunctionRight,
                1
            ),
            (
                4,
                0,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::DisjunctionLeft,
                2
            ),
            (
                4,
                1,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::DisjunctionRight,
                3
            ),
            (
                6,
                0,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::ConjunctionLeft,
                4
            ),
            (
                6,
                1,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::ConjunctionRight,
                5
            ),
            (
                7,
                0,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::DisjunctionLeft,
                6
            ),
            (
                7,
                1,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::DisjunctionRight,
                7
            ),
        ]
    );
}

#[test]
fn task257b2_corruption_matrix_fails_closed_and_valid_replay_recovers() {
    let (ast, module, symbols) = task252_real_ast(TASK257B2_CASE);
    let corruptions: [fn(&mut SourceFormulaCompositionRouteInputs); 12] = [
        |input| input.primary.terms[0].context =
            mizar_checker::binding_env::BindingContextId::new(0),
        |input| input.primary.numeric_type_requests.pop().map_or((), drop),
        |input| input.atomic.formulas.swap(0, 1),
        |input| input.atomic.edges.swap(0, 1),
        |input| input.atomic.requests[0].edge =
            Some(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(1)),
        |input| {
            input.composite.formulas[3].kind =
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Conjunction
        },
        |input| input.composite.wrappers[0].source_range.end -= 1,
        |input| input.composite.wrappers.swap(0, 1),
        |input| input.composite.edges.swap(1, 2),
        |input| input.composite.requests.pop().map_or((), drop),
        |input| {
            input.composition.atomic_edges[0].role =
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::DisjunctionLeft
        },
        |input| {
            input
                .composition
                .bound_uses
                .push(mizar_checker::source_formula_composition::SourceQuantifierBoundUseInput {
                    binder: mizar_checker::source_composite_formula::SourceQuantifierBinderId::new(
                        0,
                    ),
                    ordinal: 0,
                    body_edge:
                        mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeId::new(0),
                    term: mizar_checker::source_term::SourcePrimaryTermId::new(0),
                    reference:
                        mizar_checker::source_term::SourcePrimaryTermReferenceId::new(0),
                })
        },
    ];
    for corrupt in corruptions {
        assert!(
            source_formula_composition_output_with_mutation(
                &ast,
                module.clone(),
                &symbols,
                corrupt,
            )
            .expect("corruption preserves Task 257B2 selector")
            .is_err()
        );
    }
    assert!(
        source_formula_composition_output(&ast, module, &symbols)
            .expect("valid Task 257B2 route remains selected")
            .is_ok()
    );
}

#[test]
fn task257b2_source_selector_near_miss_matrix_remains_unselected() {
    const EXACT: &str = "theorem FormulaConnectiveGroupingPayloadBoundary: for x being set holds ((0 = 0 & ... & 0 = 3) or (0 = 0 or ... or 0 = 3)) iff ((0 = 0 & 0 = 0) or (0 = 0 or 0 = 0));\n";
    let near_misses = [
        EXACT.replacen("theorem ", "canceled theorem ", 1),
        EXACT.replacen(";", " by;", 1),
        format!("{EXACT}theorem ExtraItem: 0 = 0;\n"),
        EXACT.replacen(
            "FormulaConnectiveGroupingPayloadBoundary",
            "AnotherFormulaConnectiveGroupingBoundary",
            1,
        ),
        EXACT.replacen("for x being set", "for y being set", 1),
        EXACT.replacen("being set", "being object", 1),
        EXACT.replacen("0 = 0 & ... & 0 = 3", "0 = 3 & ... & 0 = 0", 1),
        EXACT.replacen("0 = 0 & 0 = 0", "0 = 0 & 0 = 0 & 0 = 0", 1),
        EXACT
            .replacen("holds ((", "holds (((", 1)
            .replacen(")) iff", "))) iff", 1),
        EXACT.replacen(
            "((0 = 0 & ... & 0 = 3) or (0 = 0 or ... or 0 = 3))",
            "(0 = 0 & ... & 0 = 3 or ((0 = 0 or ... or 0 = 3)))",
            1,
        ),
        EXACT.replacen("& ... &", "&", 1),
        EXACT.replacen("or ... or", "or", 1),
        EXACT.replacen(" iff ", " implies ", 1),
        "theorem FormulaConnectiveGroupingPayloadBoundary: for x being set holds ex y being set st y = y;\n".to_owned(),
        "theorem FormulaConnectiveGroupingPayloadBoundary: x divides y does not divides z;\n"
            .to_owned(),
        "theorem FormulaConnectiveGroupingPayloadBoundary: { x where x is set : thesis } = y;\n"
            .to_owned(),
    ];
    for (ordinal, source) in near_misses.into_iter().enumerate() {
        let (ast, module, _, symbols) = task253_ast_from_source_text(&source, 20_000 + ordinal);
        assert!(
            extract_source_formula_connective_grouping(&ast, &module, &symbols).is_none(),
            "near miss {ordinal} unexpectedly selected"
        );
        assert!(
            source_formula_composition_output(&ast, module, &symbols).is_none(),
            "near miss {ordinal} unexpectedly reached the producer"
        );
    }
}

#[test]
fn task257b3_real_route_publishes_exact_nested_ranges_profiles_and_final_ownership() {
    let (ast, module, symbols) = task252_real_ast(TASK257B3_CASE);
    let payload =
        extract_source_formula_nested_quantifier_payload(&ast, &module, &symbols, TASK257B3_SOURCE)
            .expect("Task 257B3 selector payload");
    assert_eq!(
        payload
            .formula_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(67, 136), (92, 136), (110, 136)]
    );
    assert_eq!(
        payload
            .binder_segment_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(71, 82), (95, 106), (114, 115)]
    );
    assert_eq!(
        payload
            .binder_identifier_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(71, 72), (95, 96), (114, 115)]
    );
    assert_eq!(
        payload
            .binder_type_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(79, 82), (103, 106), (14, 17)]
    );
    assert_eq!(
        payload
            .binder_type_head_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(79, 82), (103, 106), (14, 17)]
    );
    assert_eq!(
        payload
            .equality_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [(86, 91), (119, 124), (131, 136)]
    );
    assert_eq!(
        payload
            .term_ranges
            .iter()
            .map(|range| (range.start, range.end))
            .collect::<Vec<_>>(),
        [
            (86, 87),
            (90, 91),
            (119, 120),
            (123, 124),
            (131, 132),
            (135, 136),
        ]
    );
    assert_eq!(payload.term_spellings, ["x", "x", "r", "y", "x", "r"]);
    assert_eq!(payload.reserve.bindings().len(), 1);
    assert_eq!(payload.reserve.bindings()[0].spelling, "r");
    assert_eq!(payload.reserve.module_context().index(), 0);

    let first =
        source_formula_composition_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            TASK257B3_SOURCE,
        )
        .expect("Task 257B3 selects")
        .expect("Task 257B3 transaction");
    let second =
        source_formula_composition_output_with_source(&ast, module, &symbols, TASK257B3_SOURCE)
            .expect("Task 257B3 remains selected")
            .expect("Task 257B3 replay");
    let primary = first.typed_ast.source_term().expect("Task 252 handoff");
    let atomic = first
        .typed_ast
        .source_atomic_formula()
        .expect("Task 256 handoff");
    let composite = first
        .typed_ast
        .source_composite_formula()
        .expect("Task 257 handoff");
    let composition = task257b1_handoff(&first);
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (6, 6, 0)
    );
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.wrappers().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.attributes().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (3, 0, 0, 0, 0, 0, 6, 6)
    );
    assert_eq!(
        (
            composite.formulas().len(),
            composite.wrappers().len(),
            composite.roots().len(),
            composite.binders().len(),
            composite.type_sites().len(),
            composite.edges().len(),
            composite.requests().len(),
        ),
        (3, 0, 1, 3, 3, 2, 6)
    );
    assert_eq!(
        (
            composite.binding_env().contexts().len(),
            composite.binding_env().bindings().len(),
            composite.binding_env().diagnostics().len(),
            composition.atomic_edges().len(),
            composition.bound_uses().len(),
        ),
        (4, 4, 0, 3, 6)
    );
    assert_eq!(composition.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(
        composition.atomic_formula_fingerprint(),
        atomic.debug_text()
    );
    assert_eq!(
        composition.composite_formula_fingerprint(),
        composite.debug_text()
    );
    assert!(first.typed_ast.source_context().is_none());
    assert_eq!(
        first.typed_ast.source_formula_composition(),
        first.resolved.source_formula_composition()
    );
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
}

#[test]
fn task257b3_nested_visibility_shadowing_atomic_edges_and_bound_uses_are_exact() {
    let (ast, module, symbols) = task252_real_ast(TASK257B3_CASE);
    let output =
        source_formula_composition_output_with_source(&ast, module, &symbols, TASK257B3_SOURCE)
            .expect("Task 257B3 selects")
            .expect("Task 257B3 transaction");
    let primary = output.typed_ast.source_term().expect("Task 252 handoff");
    let atomic = output
        .typed_ast
        .source_atomic_formula()
        .expect("Task 256 handoff");
    let composite = output
        .typed_ast
        .source_composite_formula()
        .expect("Task 257 composite");
    let composition = task257b1_handoff(&output);

    assert_eq!(
        composite
            .binding_env()
            .contexts()
            .iter()
            .map(|(_, row)| {
                (
                    row.lexical_scope
                        .as_ref()
                        .map(|scope| scope.path().to_vec()),
                    row.visible_bindings
                        .iter()
                        .map(|binding| binding.index())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (None, vec![0]),
            (Some(vec![0]), vec![0, 1]),
            (Some(vec![0, 0]), vec![0, 1, 2]),
            (Some(vec![0, 0, 0]), vec![0, 1, 2, 3]),
        ]
    );
    assert_eq!(
        composite
            .binding_env()
            .bindings()
            .iter()
            .map(|(id, row)| {
                (
                    id.index(),
                    row.spelling.as_str(),
                    row.owner_context.index(),
                    row.visible_after_ordinal,
                )
            })
            .collect::<Vec<_>>(),
        [
            (0, "r", 0, 0),
            (1, "x", 1, 1),
            (2, "y", 2, 2),
            (3, "r", 3, 3),
        ]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(_, row)| {
                (
                    row.term().index(),
                    row.binding().index(),
                    row.use_ordinal(),
                    row.lexical_scope().map(|scope| scope.path().to_vec()),
                )
            })
            .collect::<Vec<_>>(),
        [
            (0, 1, 2, Some(vec![0])),
            (1, 1, 2, Some(vec![0])),
            (2, 3, 4, Some(vec![0, 0, 0])),
            (3, 2, 4, Some(vec![0, 0, 0])),
            (4, 1, 4, Some(vec![0, 0, 0])),
            (5, 3, 4, Some(vec![0, 0, 0])),
        ]
    );
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(_, row)| (row.context().index(), row.spelling()))
            .collect::<Vec<_>>(),
        [(1, "x = x"), (3, "r = y"), (3, "x = r")]
    );
    assert_eq!(
        composite
            .edges()
            .iter()
            .map(|(_, row)| {
                (
                    row.parent().index(),
                    row.ordinal(),
                    row.role(),
                    row.child().index(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::UniversalBody,
                1,
            ),
            (
                1,
                0,
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::ExistentialBody,
                2,
            ),
        ]
    );
    assert_eq!(
        composition
            .atomic_edges()
            .iter()
            .map(|(_, row)| {
                (
                    row.formula().index(),
                    row.ordinal(),
                    row.role(),
                    row.child().index(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::UniversalRestriction,
                0,
            ),
            (
                2,
                0,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::UniversalRestriction,
                1,
            ),
            (
                2,
                1,
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::UniversalBody,
                2,
            ),
        ]
    );
    assert_eq!(
        composition
            .bound_uses()
            .iter()
            .map(|(_, row)| {
                (
                    row.binder().index(),
                    row.ordinal(),
                    row.body_edge().index(),
                    row.term().index(),
                    row.reference().index(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (0, 0, 0, 0, 0),
            (0, 1, 0, 1, 1),
            (2, 0, 1, 2, 2),
            (1, 0, 1, 3, 3),
            (0, 2, 2, 4, 4),
            (2, 1, 2, 5, 5),
        ]
    );
}

#[test]
fn task257b3_corruption_matrix_fails_closed_and_valid_replay_recovers() {
    let (ast, module, symbols) = task252_real_ast(TASK257B3_CASE);
    let corruptions: [fn(&mut SourceFormulaCompositionRouteInputs); 18] = [
        |input| {
            input.primary.terms[0].context =
                mizar_checker::binding_env::BindingContextId::new(3)
        },
        |input| {
            input.primary.references[2].binding =
                mizar_checker::binding_env::BindingId::new(0)
        },
        |input| input.primary.references.swap(0, 1),
        |input| input.atomic.formulas.swap(0, 1),
        |input| input.atomic.edges.swap(0, 1),
        |input| {
            input.atomic.requests[0].edge =
                Some(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(1))
        },
        |input| {
            input.composite.formulas[1].kind =
                mizar_checker::source_composite_formula::SourceCompositeFormulaKind::Universal
        },
        |input| {
            input.composite.binders[2].binding =
                mizar_checker::binding_env::BindingId::new(0)
        },
        |input| {
            input.composite.type_sites[2].context =
                mizar_checker::binding_env::BindingContextId::new(2)
        },
        |input| {
            input.composite.edges[1].role =
                mizar_checker::source_composite_formula::SourceFormulaEdgeRole::UniversalBody
        },
        |input| input.composite.requests.swap(2, 3),
        |input| input.composition.atomic_edges.swap(0, 1),
        |input| {
            input.composition.atomic_edges[0].role =
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeRole::UniversalBody
        },
        |input| {
            input.composition.bound_uses[2].binder =
                mizar_checker::source_composite_formula::SourceQuantifierBinderId::new(0)
        },
        |input| {
            input.composition.bound_uses[2].body_edge =
                mizar_checker::source_formula_composition::SourceFormulaAtomicEdgeId::new(0)
        },
        |input| {
            input.composition.bound_uses[2].term =
                mizar_checker::source_term::SourcePrimaryTermId::new(3)
        },
        |input| input.composition.bound_uses.swap(0, 1),
        |input| input.composition.bound_uses.pop().map_or((), drop),
    ];
    for corrupt in corruptions {
        assert!(
            source_formula_composition_output_with_source_and_mutation(
                &ast,
                module.clone(),
                &symbols,
                TASK257B3_SOURCE,
                corrupt,
            )
            .expect("corruption preserves Task 257B3 selector")
            .is_err()
        );
    }
    assert!(
        source_formula_composition_output_with_source(&ast, module, &symbols, TASK257B3_SOURCE)
            .expect("valid Task 257B3 route remains selected")
            .is_ok()
    );
}

#[test]
fn task257b3_source_selector_near_miss_matrix_remains_unselected() {
    let near_misses = [
        TASK257B3_SOURCE.trim_end_matches('\n').to_owned(),
        TASK257B3_SOURCE.replacen("reserve r for set; ", "reserve r for set;\n", 1),
        TASK257B3_SOURCE.replacen("reserve r for set; ", "reserve r for set;  ", 1),
        TASK257B3_SOURCE.replacen("reserve r for set; ", "", 1),
        TASK257B3_SOURCE.replacen("reserve r", "reserve s", 1),
        TASK257B3_SOURCE.replacen("reserve r for set", "reserve r for object", 1),
        TASK257B3_SOURCE.replacen(
            "FormulaNestedQuantifierPayloadBoundary",
            "AnotherNestedQuantifierPayloadBoundary",
            1,
        ),
        TASK257B3_SOURCE.replacen("for x being set", "for z being set", 1),
        TASK257B3_SOURCE.replacen("x being set", "x being object", 1),
        TASK257B3_SOURCE.replacen("for x being set st", "ex x being set st", 1),
        TASK257B3_SOURCE.replacen(" st x = x ex ", " holds x = x ex ", 1),
        TASK257B3_SOURCE.replacen("ex y being set", "ex z being set", 1),
        TASK257B3_SOURCE.replacen("y being set", "y being object", 1),
        TASK257B3_SOURCE.replacen("ex y being set st", "for y being set st", 1),
        TASK257B3_SOURCE.replacen("for r st", "ex r st", 1),
        TASK257B3_SOURCE.replacen("for r st r = y holds", "for r holds r = y", 1),
        TASK257B3_SOURCE.replacen("r = y", "y = r", 1),
        TASK257B3_SOURCE.replacen("x = r;", "r = x;", 1),
        TASK257B3_SOURCE.replacen("x = x ex", "(x = x) ex", 1),
        TASK257B3_SOURCE.replacen(";", " by;", 2),
        format!("{TASK257B3_SOURCE}theorem ExtraItem: 0 = 0;\n"),
        format!(
            "{}{}",
            "theorem FormulaNestedQuantifierPayloadBoundary: for x being set st x = x ex y being set st for r st r = y holds x = r; ",
            "reserve r for set;\n"
        ),
        TASK257B3_SOURCE.replacen("reserve r for set;", "reserve r, s for set;", 1),
        TASK257B3_SOURCE.replacen("theorem ", "canceled theorem ", 1),
        TASK257B3_SOURCE.replacen("x = r;", "x = r by;", 1),
        TASK257B3_SOURCE.replacen("for x being set", "for x", 1),
        TASK257B3_SOURCE.replacen("for r st", "for r being set st", 1),
        TASK257B3_SOURCE.replacen("reserve r for set", "reserve r for non empty set", 1),
        TASK257B3_SOURCE.replacen("x being set", "x being non empty set", 1),
        TASK257B3_SOURCE.replacen("reserve r for set", "reserve r for Element of {{}}", 1),
        TASK257B3_SOURCE.replacen("x being set", "x being Element of {{}}", 1),
        format!(
            "{}{}",
            TASK257B3_SOURCE,
            "definition let z be set; func task257b3_extra(z) -> set equals z; end;\n"
        ),
        format!(
            "{}{}",
            TASK257B3_SOURCE,
            "registration cluster set -> empty; end;\n"
        ),
        "reserve r for set; definition let r be set; end;\n".to_owned(),
        "reserve r for set; theorem FormulaNestedQuantifierPayloadBoundary: x divides y does not divides z;\n".to_owned(),
        "reserve r for set; theorem FormulaNestedQuantifierPayloadBoundary: { x where x is set : thesis } = y;\n".to_owned(),
    ];
    for (ordinal, source) in near_misses.into_iter().enumerate() {
        let (ast, module, _, symbols) = task253_ast_from_source_text(&source, 21_000 + ordinal);
        assert!(
            extract_source_formula_nested_quantifier_payload(&ast, &module, &symbols, &source)
                .is_none(),
            "near miss {ordinal} unexpectedly selected"
        );
        assert!(
            source_formula_composition_output_with_source(&ast, module, &symbols, &source)
                .is_none(),
            "near miss {ordinal} unexpectedly reached the producer"
        );
    }
}

#[test]
fn task257b3_recovered_source_tree_is_rejected_before_production() {
    let recovered_source = "reserve r for set; theorem FormulaNestedQuantifierPayloadBoundary: for x being set st x = x ex y being set st for r st r = y x = r;\n";
    let (ast, module, _, symbols) =
        task253_ast_from_source_text(recovered_source, 21_999);
    assert!(
        ast.nodes().iter().any(|node| matches!(
            node.kind,
            mizar_syntax::SurfaceNodeKind::ErrorRecovery(_)
        )),
        "the malformed recovery probe must contain a recovery node"
    );
    assert!(
        extract_source_formula_nested_quantifier_payload(
            &ast,
            &module,
            &symbols,
            recovered_source,
        )
        .is_none()
    );
    assert!(
        source_formula_composition_output_with_source(
            &ast,
            module,
            &symbols,
            recovered_source,
        )
        .is_none()
    );
}

#[test]
fn task257b3_pass_sidecar_observes_transport_only() {
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
    let plan = build_test_plan(&config).expect("Task 257B3 plan");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK257B3_CASE)
        .expect("Task 257B3 case active");
    let result = run_type_elaboration_case(&workspace_root, &tests_root, case, ordinal);
    assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
    assert!(result.actual_detail_keys.is_empty());
    assert_eq!(
        result.actual_detail_keys,
        expected_type_elaboration_detail_keys(case)
    );
}

#[test]
fn task257b2_pass_sidecar_observes_transport_only() {
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
    let plan = build_test_plan(&config).expect("Task 257B2 plan");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK257B2_CASE)
        .expect("Task 257B2 case active");
    let result = run_type_elaboration_case(&workspace_root, &tests_root, case, ordinal);
    assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
    assert!(result.actual_detail_keys.is_empty());
    assert_eq!(
        result.actual_detail_keys,
        expected_type_elaboration_detail_keys(case)
    );
}

#[test]
fn task257b1_pass_sidecar_observes_transport_only() {
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
    let plan = build_test_plan(&config).expect("Task 257B1 plan");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK257B1_CASE)
        .expect("Task 257B1 case active");
    let result = run_type_elaboration_case(&workspace_root, &tests_root, case, ordinal);
    assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
    assert!(result.actual_detail_keys.is_empty());
    assert_eq!(
        result.actual_detail_keys,
        expected_type_elaboration_detail_keys(case)
    );
}

fn task257b1_handoff(
    output: &SourceFormulaCompositionRouteOutput,
) -> &mizar_checker::source_formula_composition::SourceFormulaCompositionHandoff {
    output
        .typed_ast
        .source_formula_composition()
        .expect("Task 257B1 handoff")
}
