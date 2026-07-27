use super::{
    SourceConditionFormulaCompositionRouteInputs, SourceFormulaCompositionRouteInputs,
    SourceFormulaCompositionRouteOutput, SourcePredicateChainCompositionRouteInputs,
    extract_source_formula_connective_grouping, extract_source_formula_quantifier_bound_use,
    extract_source_formula_nested_quantifier_payload, source_formula_composition_output,
    source_formula_composition_output_with_mutation,
    source_formula_composition_output_with_source,
    source_formula_composition_output_with_source_and_mutation,
    source_formula_composition_transport_detail_keys,
    source_condition_formula_composition_output_with_source,
    source_condition_formula_composition_output_with_source_and_mutation,
    source_predicate_chain_composition_output_with_source,
    source_predicate_chain_composition_output_with_source_and_mutation,
};

const TASK257B1_CASE: &str = "pass_type_elaboration_formula_quantifier_bound_use_payload_001";
const TASK257B2_CASE: &str = "pass_type_elaboration_formula_connective_grouping_payload_001";
const TASK257B3_CASE: &str = "pass_type_elaboration_formula_nested_quantifier_payload_001";
const TASK257B3_SOURCE: &str = "reserve r for set; theorem FormulaNestedQuantifierPayloadBoundary: for x being set st x = x ex y being set st for r st r = y holds x = r;\n";
const TASK257C2_CASE: &str =
    "fail_type_elaboration_conditioned_comprehension_source_payload_001";

#[test]
fn task257c2_real_route_publishes_the_exact_condition_formula_transaction() {
    let (ast, module, _shells, symbols, source) = task255c1_real_ast();
    assert_eq!(&*source, TASK255C1_SET_SOURCE);
    assert_eq!(source.len(), 191);
    assert!(source.ends_with('\n'));
    let output = source_condition_formula_composition_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        &source,
    )
    .expect("Task257C2 exact selector")
    .unwrap_or_else(|error| panic!("Task257C2 real route failed: {error}"));
    let replay = source_condition_formula_composition_output_with_source_and_mutation(
        &ast,
        module,
        &symbols,
        &source,
        |_: &mut SourceConditionFormulaCompositionRouteInputs| {},
    )
    .expect("Task257C2 repeated selector")
    .unwrap_or_else(|error| panic!("Task257C2 repeated route failed: {error}"));

    let primary = output.typed_ast.source_term().expect("Task252 handoff");
    let application = output
        .typed_ast
        .source_application()
        .expect("Task253 handoff");
    let set = output
        .typed_ast
        .source_set_term()
        .expect("Task255 handoff");
    let atomic = output
        .typed_ast
        .source_atomic_formula()
        .expect("Task256 handoff");
    let composition = output
        .typed_ast
        .source_condition_formula_composition()
        .expect("Task257C2 handoff");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (4, 0, 4)
    );
    assert_eq!(
        (
            application.applications().len(),
            application.wrappers().len(),
            application.candidates().len(),
            application.arguments().len(),
            application.type_requests().len(),
        ),
        (1, 0, 1, 2, 2)
    );
    assert_eq!(
        (
            set.terms().len(),
            set.wrappers().len(),
            set.generators().len(),
            set.type_sites().len(),
            set.conditions().len(),
            set.edges().len(),
            set.requests().len(),
        ),
        (1, 0, 1, 1, 1, 1, 2)
    );
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.wrappers().len(),
            atomic.predicate_segments().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.attributes().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (1, 0, 0, 0, 0, 0, 0, 2, 2)
    );

    let candidate = application
        .candidates()
        .get(mizar_checker::source_application::SourceFunctorCandidateId::new(0))
        .expect("Task253 imported mapper candidate");
    let symbol = symbols
        .symbols()
        .get(candidate.symbol())
        .expect("Task253 candidate symbol");
    let contribution = symbols
        .contributions()
        .get(candidate.contribution())
        .expect("Task253 candidate contribution");
    assert_eq!(symbol.primary_spelling(), "++");
    assert_eq!(
        candidate.symbol().module().path().as_str(),
        "parser.type_fixtures"
    );
    assert!(matches!(
        contribution.kind(),
        mizar_resolve::env::ContributionKind::ImportedSource { .. }
    ));

    let condition = set
        .conditions()
        .get(mizar_checker::source_set_term::SourceSetConditionId::new(0))
        .expect("Task255 condition");
    let formula = atomic
        .formulas()
        .get(mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0))
        .expect("Task256 equality");
    assert_eq!(
        (
            condition.source_range().start,
            condition.source_range().end,
            condition.spelling(),
        ),
        (177, 182, "3 = 4")
    );
    assert_eq!(formula.source_range(), condition.source_range());
    assert_eq!(formula.spelling(), condition.spelling());
    assert_eq!(
        condition.recovery(),
        mizar_checker::source_set_term::SourceSetTermRecovery::Normal
    );
    assert_eq!(
        formula.recovery(),
        mizar_checker::source_atomic_formula::SourceAtomicFormulaRecovery::Normal
    );
    assert_eq!(
        formula.kind(),
        mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
    );
    let owner_term = set
        .terms()
        .get(condition.term())
        .expect("Task255 condition owner term");
    assert_eq!(formula.context(), owner_term.context());
    assert_ne!(condition.condition_site(), formula.site());
    assert!(
        output
            .typed_ast
            .nodes()
            .node(condition.condition_site().node())
            .expect("Task255 wrapper arena site")
            .children
            .contains(&formula.site().node())
    );
    assert_eq!(
        output
            .typed_ast
            .nodes()
            .node(condition.condition_site().node())
            .expect("Task255 wrapper arena site")
            .kind
            .as_str(),
        "source.term.set.comprehension-condition"
    );
    assert_eq!(
        output
            .typed_ast
            .nodes()
            .node(formula.site().node())
            .expect("Task256 equality arena site")
            .kind
            .as_str(),
        "source.formula.atomic.equality"
    );
    assert_eq!(
        atomic
            .edges()
            .iter()
            .map(|(_, edge)| (edge.role(), edge.target()))
            .collect::<Vec<_>>(),
        [
            (
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinLeftOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2),
                ),
            ),
            (
                mizar_checker::source_atomic_formula::SourceAtomicEdgeRole::BuiltinRightOperand,
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(3),
                ),
            ),
        ]
    );
    assert!(atomic.requests().iter().all(|(id, request)| {
        request.formula()
            == mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0)
            && request.ordinal() == id.index()
            && request.kind()
                == mizar_checker::source_atomic_formula::SourceAtomicRequestKind::OperandExpectedType
            && request.edge()
                == Some(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(
                    id.index(),
                ))
            && request.candidate().is_none()
            && request.type_site().is_none()
            && request.attribute().is_none()
    }));

    assert_eq!(composition.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(
        composition.application_fingerprint(),
        application.debug_text()
    );
    assert_eq!(composition.set_term_fingerprint(), set.debug_text());
    assert_eq!(
        composition.atomic_formula_fingerprint(),
        atomic.debug_text()
    );
    assert_eq!(composition.edges().len(), 1);
    let edge = composition
        .edges()
        .get(
            mizar_checker::source_formula_composition::SourceConditionFormulaEdgeId::new(0),
        )
        .expect("Task257C2 edge");
    assert_eq!(
        (
            edge.condition(),
            edge.ordinal(),
            edge.formula(),
        ),
        (
            mizar_checker::source_set_term::SourceSetConditionId::new(0),
            0,
            mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0),
        )
    );
    assert_eq!(composition.edges().iter().count(), 1);
    assert!(composition
        .debug_text()
        .starts_with("source-condition-formula-composition-debug-v1\n"));
    assert!(composition
        .debug_text()
        .contains("edges: 1\n  edge#0 condition=0 ordinal=0 formula=0\n"));

    assert_eq!(
        output.typed_ast.source_condition_formula_composition(),
        replay.typed_ast.source_condition_formula_composition()
    );
    assert_eq!(
        output.typed_ast.source_condition_formula_composition(),
        output.resolved.source_condition_formula_composition()
    );
    assert_eq!(output.typed_ast.debug_text(), replay.typed_ast.debug_text());
    assert_eq!(output.resolved.debug_text(), replay.resolved.debug_text());
    let typed_clone = output.typed_ast.clone();
    let resolved_clone = output.resolved.clone();
    assert_eq!(typed_clone, output.typed_ast);
    assert_eq!(resolved_clone, output.resolved);
    assert_eq!(typed_clone.debug_text(), output.typed_ast.debug_text());
    assert_eq!(resolved_clone.debug_text(), output.resolved.debug_text());
    assert!(output.typed_ast.source_composite_formula().is_none());
    assert!(output.typed_ast.source_formula_composition().is_none());
    assert!(output.typed_ast.types().is_empty());
    assert!(output.typed_ast.facts().is_empty());
    assert!(output.typed_ast.coercions().is_empty());
    assert!(output.typed_ast.initial_obligations().is_empty());
    assert!(output.typed_ast.diagnostics().is_empty());
    assert!(output.resolved.expr_metadata().is_empty());
    assert!(output.resolved.cluster_facts().is_empty());
    assert!(output.resolved.diagnostics().is_empty());
}

#[test]
fn task257c2_dependency_edge_and_arena_mutations_fail_atomically() {
    let (ast, module, _shells, symbols, source) = task255c1_real_ast();
    let baseline = source_condition_formula_composition_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        &source,
    )
    .expect("Task257C2 exact selector")
    .expect("Task257C2 baseline");
    let mutations: [fn(&mut SourceConditionFormulaCompositionRouteInputs); 9] = [
        |input| input.composition.edges.clear(),
        |input| input.composition.edges.push(input.composition.edges[0].clone()),
        |input| {
            input.composition.edges[0].condition =
                mizar_checker::source_set_term::SourceSetConditionId::new(1)
        },
        |input| input.composition.edges[0].ordinal = 1,
        |input| {
            input.composition.edges[0].formula =
                mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(1)
        },
        |input| input.atomic.formulas[0].spelling = "4 = 3".to_owned(),
        |input| {
            input.atomic.formulas[0].kind =
                mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Inequality
        },
        |input| {
            input.atomic.edges[0].target =
                mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(3),
                )
        },
        |input| {
            input.atomic.requests.swap(0, 1);
        },
    ];
    for mutate in mutations {
        let result = source_condition_formula_composition_output_with_source_and_mutation(
            &ast,
            module.clone(),
            &symbols,
            &source,
            mutate,
        )
        .expect("Task257C2 mutation retains the exact selector");
        assert!(result.is_err(), "Task257C2 corruption must fail closed");
        let replay = source_condition_formula_composition_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            &source,
        )
        .expect("Task257C2 selector after corruption")
        .expect("Task257C2 replay after corruption");
        assert_eq!(replay.typed_ast.debug_text(), baseline.typed_ast.debug_text());
        assert_eq!(replay.resolved.debug_text(), baseline.resolved.debug_text());
    }

    let wrapper = task255_node_with_kind_and_spelling(
        &ast,
        &SurfaceNodeKind::FormulaExpression,
        "3 = 4",
    );
    assert!(
        source_condition_formula_composition_output_with_source_and_mutation(
            &ast,
            module.clone(),
            &symbols,
            &source,
            |input| {
                input.atomic.formulas[0].site =
                    mizar_checker::typed_ast::TypedSiteRef::Node(
                        mizar_checker::typed_ast::TypedNodeId::new(wrapper),
                    );
            },
        )
        .expect("Task257C2 copied-site mutation retains the exact selector")
        .is_err()
    );

    let condition_site = baseline
        .typed_ast
        .source_set_term()
        .expect("Task255 baseline")
        .conditions()
        .get(mizar_checker::source_set_term::SourceSetConditionId::new(0))
        .expect("Task255 condition")
        .condition_site()
        .node();
    let formula_site = baseline
        .typed_ast
        .source_atomic_formula()
        .expect("Task256 baseline")
        .formulas()
        .get(mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(0))
        .expect("Task256 formula")
        .site()
        .node();
    let arena_error = source_condition_formula_composition_output_with_source_and_mutation(
        &ast,
        module.clone(),
        &symbols,
        &source,
        |input| {
            let mut nodes = input
                .arena
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            nodes[condition_site.index()]
                .children
                .retain(|child| *child != formula_site);
            input.arena =
                mizar_checker::typed_ast::TypedArena::try_new(input.arena.root(), nodes)
                    .expect("structurally valid stale Task257C2 arena");
        },
    )
    .expect("Task257C2 arena mutation retains the exact selector")
    .expect_err("Task257C2 arena mutation must fail closed");
    assert_eq!(
        arena_error,
        "source atomic-formula set-term dependency mismatch"
    );
    let replay = source_condition_formula_composition_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        &source,
    )
    .expect("Task257C2 replay selector after arena mutation")
    .expect("Task257C2 replay after arena mutation");
    assert_eq!(replay.typed_ast.debug_text(), baseline.typed_ast.debug_text());
    assert_eq!(replay.resolved.debug_text(), baseline.resolved.debug_text());
    assert!(
        source_condition_formula_composition_output_with_source(
            &ast,
            module,
            &symbols,
            &source,
        )
        .expect("Task257C2 final replay selector")
        .is_ok()
    );
}

#[test]
fn task257c2_loaded_source_named_near_misses_and_active_isolation_reject() {
    let (ast, module, _shells, symbols, source) = task255c1_real_ast();
    assert!(
        source_condition_formula_composition_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            source.trim_end_matches('\n'),
        )
        .is_none()
    );
    assert!(
        source_condition_formula_composition_output_with_source(
            &ast,
            module,
            &symbols,
            &source.replacen("equals {", "equals  {", 1),
        )
        .is_none()
    );

    let near_misses = [
        TASK255C1_SET_SOURCE.replacen(
            "Task255ConditionedComprehensionDef",
            "Task257C2NamedNearMiss",
            1,
        ),
        TASK255C1_SET_SOURCE.replacen("1 ++ 2", "2 ++ 1", 1),
        TASK255C1_SET_SOURCE.replacen(": 3 = 4", ": 3 <> 4", 1),
        format!("{TASK255C1_SET_SOURCE}theorem Task257C2Extra: thesis;\n"),
    ];
    for (ordinal, near_source) in near_misses.iter().enumerate() {
        let (near_ast, near_module, near_symbols) =
            task255_ast_from_source_text(near_source, 257_200 + ordinal);
        let near_symbols =
            augment_type_elaboration_import_summaries(&near_ast, &near_module, near_symbols);
        assert!(
            source_condition_formula_composition_output_with_source(
                &near_ast,
                near_module.clone(),
                &near_symbols,
                TASK255C1_SET_SOURCE,
            )
            .is_none(),
            "Task257C2 AST near miss {ordinal} must reject under exact loaded bytes"
        );
        assert!(
            source_condition_formula_composition_output_with_source(
                &near_ast,
                near_module,
                &near_symbols,
                near_source,
            )
            .is_none(),
            "Task257C2 raw near miss {ordinal} must reject"
        );
    }

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
    let plan = build_test_plan(&config).expect("Task257C2 isolation plan");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if matches!(
            source_condition_formula_composition_output_with_source(
                &ast,
                resolver.module,
                &symbols,
                &source,
            ),
            Some(Ok(_))
        ) {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(selected, [TASK257C2_CASE]);
}

#[test]
fn task257c2_fail_sidecar_preserves_the_definition_intake_boundary() {
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
    let plan = build_test_plan(&config).expect("Task257C2 plan");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK257C2_CASE)
        .expect("Task257C2 case active");
    let result = run_type_elaboration_case(&workspace_root, &tests_root, case, ordinal);
    assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
    assert_eq!(
        result.actual_detail_keys,
        ["type_elaboration.external_dependency.ast_payload_extraction"]
    );
    assert_eq!(
        result.actual_detail_keys,
        expected_type_elaboration_detail_keys(case)
    );
}

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

#[test]
fn task257c3_real_route_publishes_exact_composition_and_resolver_provenance() {
    let (ast, module, symbols) = task252_real_ast(TASK257C1_CASE);
    assert_eq!(TASK257C1_SOURCE.len(), 107);
    assert!(TASK257C1_SOURCE.ends_with('\n'));
    let payload =
        extract_source_imported_predicate_chain_formula(&ast, &module, &symbols, TASK257C1_SOURCE)
            .expect("Task257C3 exact lower selector");
    assert_eq!(
        (
            payload.formula_range.start,
            payload.formula_range.end,
            payload.segment_ranges.map(|range| (range.start, range.end)),
            payload.head_ranges.map(|range| (range.start, range.end)),
            payload.term_ranges.map(|range| (range.start, range.end)),
            (payload.verb_range.start, payload.verb_range.end),
            (payload.not_range.start, payload.not_range.end),
        ),
        (
            75,
            105,
            [(75, 86), (87, 105)],
            [(77, 84), (96, 103)],
            [(75, 76), (85, 86), (104, 105)],
            (87, 91),
            (92, 95),
        )
    );

    let first = source_predicate_chain_composition_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        TASK257C1_SOURCE,
    )
    .expect("Task257C3 exact selector")
    .unwrap_or_else(|error| panic!("Task257C3 real route failed: {error}"));
    let second = source_predicate_chain_composition_output_with_source(
        &ast,
        module,
        &symbols,
        TASK257C1_SOURCE,
    )
    .expect("Task257C3 repeated selector")
    .unwrap_or_else(|error| panic!("Task257C3 repeated route failed: {error}"));
    let primary = first.typed_ast.source_term().expect("Task252 handoff");
    let atomic = first
        .typed_ast
        .source_atomic_formula()
        .expect("Task256 handoff");
    let composition = first
        .typed_ast
        .source_predicate_chain_composition()
        .expect("Task257C3 handoff");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (3, 0, 3)
    );
    assert_eq!(
        (
            atomic.formulas().len(),
            atomic.wrappers().len(),
            atomic.predicate_segments().len(),
            atomic.predicate_heads().len(),
            atomic.candidates().len(),
            atomic.type_sites().len(),
            atomic.attributes().len(),
            atomic.edges().len(),
            atomic.requests().len(),
        ),
        (1, 0, 2, 2, 2, 0, 0, 3, 2)
    );
    let candidates = atomic.candidates().iter().collect::<Vec<_>>();
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
            .expect("Task257C3 imported predicate contribution")
            .kind(),
        mizar_resolve::env::ContributionKind::ImportedSource { .. }
    ));
    assert_eq!(
        composition
            .conjunctions()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.formula().index(),
                row.ordinal(),
                row.left_segment().index(),
                row.right_segment().index(),
                row.boundary().index(),
            ))
            .collect::<Vec<_>>(),
        [(0, 0, 0, 0, 1, 1)]
    );
    assert_eq!(
        composition
            .negations()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.formula().index(),
                row.ordinal(),
                row.segment().index(),
            ))
            .collect::<Vec<_>>(),
        [(0, 0, 0, 1)]
    );
    assert_eq!(
        composition.primary_term_fingerprint(),
        primary.debug_text()
    );
    assert_eq!(
        composition.atomic_formula_fingerprint(),
        atomic.debug_text()
    );
    assert_eq!(
        first.typed_ast.source_predicate_chain_composition(),
        first.resolved.source_predicate_chain_composition()
    );
    assert!(first.typed_ast.source_composite_formula().is_none());
    assert!(first.typed_ast.source_formula_composition().is_none());
    assert!(
        first
            .typed_ast
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    let typed_clone = first.typed_ast.clone();
    assert_eq!(typed_clone, first.typed_ast);
    assert_eq!(
        typed_clone.source_predicate_chain_composition(),
        Some(composition)
    );
    assert_eq!(typed_clone.debug_text(), first.typed_ast.debug_text());
    let resolved_clone = first.resolved.clone();
    assert_eq!(resolved_clone, first.resolved);
    assert_eq!(
        resolved_clone.source_predicate_chain_composition(),
        Some(composition)
    );
    assert_eq!(resolved_clone.debug_text(), first.resolved.debug_text());
    let typed_debug = first.typed_ast.debug_text();
    assert!(
        typed_debug
            .find("source-primary-term-debug-v1")
            .is_some_and(|term| {
                let atomic = typed_debug
                    .find("source-atomic-formula-debug-v1")
                    .expect("typed Task256 debug");
                let composition = typed_debug
                    .find("source-predicate-chain-composition-debug-v1")
                    .expect("typed Task257C3 debug");
                let nodes = typed_debug.find("nodes:").expect("typed nodes");
                term < atomic && atomic < composition && composition < nodes
            })
    );
    let debug = first.resolved.debug_text();
    assert!(
        debug
            .find("source-primary-term-debug-v1")
            .is_some_and(|term| {
                let atomic = debug
                    .find("source-atomic-formula-debug-v1")
                    .expect("Task256 debug");
                let composition = debug
                    .find("source-predicate-chain-composition-debug-v1")
                    .expect("Task257C3 debug");
                let nodes = debug.find("nodes:").expect("resolved nodes");
                term < atomic && atomic < composition && composition < nodes
            })
    );
}

#[test]
fn task257c3_mutations_and_stale_arena_fail_then_valid_route_recovers() {
    let (ast, module, symbols) = task252_real_ast(TASK257C1_CASE);
    let baseline = source_predicate_chain_composition_output_with_source(
        &ast,
        module.clone(),
        &symbols,
        TASK257C1_SOURCE,
    )
    .expect("Task257C3 baseline selector")
    .expect("Task257C3 baseline output");
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    type Mutation = fn(&mut SourcePredicateChainCompositionRouteInputs);
    let corruptions: &[Mutation] = &[
        |input| input.primary = None,
        |input| input.atomic = None,
        |input| input.composition.conjunctions.clear(),
        |input| {
            input
                .composition
                .conjunctions
                .push(input.composition.conjunctions[0].clone())
        },
        |input| {
            input.composition.conjunctions[0].left_segment =
                mizar_checker::source_atomic_formula::SourcePredicateSegmentId::new(1)
        },
        |input| {
            input.composition.conjunctions[0].boundary =
                mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(0)
        },
        |input| input.composition.negations.clear(),
        |input| {
            input.composition.negations[0].segment =
                mizar_checker::source_atomic_formula::SourcePredicateSegmentId::new(0)
        },
        |input| {
            let root = input.arena.root();
            let mut nodes = input
                .arena
                .iter()
                .map(|(_, node)| node.clone())
                .collect::<Vec<_>>();
            let range = nodes
                .iter_mut()
                .find(|node| node.kind.as_str() == "source.term.numeral")
                .and_then(|node| match &mut node.anchor {
                    mizar_session::SourceAnchor::Range(range) => Some(range),
                    _ => None,
                })
                .expect("Task257C3 primary anchor must be present");
            range.start = 74;
            range.end = 75;
            input.arena = mizar_checker::typed_ast::TypedArena::try_new(root, nodes)
                .expect("stale Task257C3 arena remains structurally valid");
        },
    ];
    for (ordinal, corrupt) in corruptions.iter().copied().enumerate() {
        assert!(
            source_predicate_chain_composition_output_with_source_and_mutation(
                &ast,
                module.clone(),
                &symbols,
                TASK257C1_SOURCE,
                corrupt,
            )
            .expect("Task257C3 corruption preserves selector")
            .is_err(),
            "Task257C3 corruption {ordinal} must fail"
        );
        let recovered = source_predicate_chain_composition_output_with_source(
            &ast,
            module.clone(),
            &symbols,
            TASK257C1_SOURCE,
        )
        .expect("Task257C3 selector recovers")
        .expect("Task257C3 valid route recovers");
        assert_eq!(recovered.typed_ast.debug_text(), baseline_typed);
        assert_eq!(recovered.resolved.debug_text(), baseline_resolved);
    }
}

#[test]
fn task257c3_selector_is_exclusive_and_precedes_legacy_formula_routes() {
    let (exact_ast, exact_module, exact_symbols) = task252_real_ast(TASK257C1_CASE);
    assert_eq!(
        source_formula_composition_transport_detail_keys(
            &exact_ast,
            exact_module.clone(),
            &exact_symbols,
            TASK257C1_SOURCE,
        ),
        Some(Vec::new())
    );
    assert!(
        source_atomic_formula_output_with_source(
            &exact_ast,
            exact_module.clone(),
            &exact_symbols,
            TASK257C1_SOURCE,
        )
        .is_some(),
        "the lower Task257C1 route remains independently available"
    );
    for (label, loaded) in [
        (
            "missing final LF",
            TASK257C1_SOURCE.trim_end_matches('\n').to_owned(),
        ),
        (
            "named near miss",
            TASK257C1_SOURCE.replacen(
                "FormulaPredicateChainPayloadBoundary",
                "FormulaPredicateChainCompositionNearMiss",
                1,
            ),
        ),
        (
            "positive-only chain",
            TASK257C1_SOURCE.replacen("does not divides", "divides", 1),
        ),
        (
            "formula subtree",
            TASK257C1_SOURCE.replacen(
                "1 divides 2 does not divides 3",
                "(1 divides 2 does not divides 3) & 0 = 0",
                1,
            ),
        ),
    ] {
        let (ast, module, _, symbols) = task253_ast_from_source_text(&loaded, 257_300);
        let symbols = augment_type_elaboration_import_summaries(&ast, &module, symbols);
        assert!(
            source_predicate_chain_composition_output_with_source(
                &ast, module, &symbols, &loaded,
            )
            .is_none(),
            "{label}"
        );
    }
    assert!(
        source_predicate_chain_composition_output_with_source(
            &exact_ast,
            exact_module.clone(),
            &exact_symbols,
            TASK257C1_SOURCE.trim_end_matches('\n'),
        )
        .is_none()
    );

    let (b1_ast, b1_module, b1_symbols) = task252_real_ast(TASK257B1_CASE);
    assert_eq!(
        source_formula_composition_transport_detail_keys(
            &b1_ast,
            b1_module,
            &b1_symbols,
            "",
        ),
        Some(Vec::new()),
        "Task257B1 route remains reachable after C3 preflight"
    );
    let (b2_ast, b2_module, b2_symbols) = task252_real_ast(TASK257B2_CASE);
    assert_eq!(
        source_formula_composition_transport_detail_keys(
            &b2_ast,
            b2_module,
            &b2_symbols,
            "",
        ),
        Some(Vec::new()),
        "Task257B2 route remains reachable after C3 preflight"
    );
    let (c2_ast, c2_module, _shells, c2_symbols, c2_source) = task255c1_real_ast();
    assert_eq!(
        source_formula_composition_transport_detail_keys(
            &c2_ast,
            c2_module,
            &c2_symbols,
            &c2_source,
        ),
        Some(vec![
            "type_elaboration.external_dependency.ast_payload_extraction".to_owned()
        ]),
        "Task257C2 route remains reachable after C3 preflight"
    );

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
    let plan = build_test_plan(&config).expect("Task257C3 isolation plan");
    let mut selected = Vec::new();
    for (ordinal, case) in active_type_elaboration_cases(&plan).enumerate() {
        let frontend = run_frontend(&workspace_root, case, ordinal)
            .unwrap_or_else(|error| panic!("{} frontend failed: {error}", case.id.0));
        let source = frontend.source_text;
        let Some(ast) = frontend.ast else {
            continue;
        };
        let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
        if !resolver.detail_keys.is_empty() {
            continue;
        }
        let symbols =
            augment_type_elaboration_import_summaries(&ast, &resolver.module, resolver.env);
        if matches!(
            source_predicate_chain_composition_output_with_source(
                &ast,
                resolver.module,
                &symbols,
                &source,
            ),
            Some(Ok(_))
        ) {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(selected, [TASK257C1_CASE]);
}

#[test]
fn task257c3_trace_and_shared_sidecar_identity_are_exact() {
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
    let plan = build_test_plan(&config).expect("Task257C3 plan");
    let requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|requirement| {
            requirement.id.0
                == "spec.en.checker.type_elaboration.source_predicate_chain_composition"
        })
        .expect("Task257C3 trace row");
    assert_eq!(
        requirement.source,
        Path::new("doc/design/mizar-checker/en/source_formula_composition.md")
    );
    assert_eq!(
        requirement.section,
        "Task 257C3 Frozen Predicate-Chain Composition"
    );
    assert_eq!(
        requirement.stage,
        crate::staged_model::Stage::TypeElaboration
    );
    assert_eq!(
        requirement.status,
        crate::traceability::RequirementStatus::Covered
    );
    assert!(requirement.required);
    assert_eq!(
        requirement.coverage,
        crate::traceability::CoverageShape::Pass
    );
    assert_eq!(
        requirement.tests,
        [PathBuf::from(
            "tests/miz/pass/types/pass_type_elaboration_formula_predicate_chain_segment_payload_001.expect.toml"
        )]
    );

    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK257C1_CASE)
        .expect("Task257C1/C3 shared case active");
    assert_eq!(
        case.expectation
            .spec_refs
            .iter()
            .map(|id| id.0.as_str())
            .collect::<Vec<_>>(),
        [
            "spec.en.checker.type_elaboration.source_predicate_chain_segment_payload",
            "spec.en.checker.type_elaboration.source_predicate_chain_composition",
        ]
    );
    assert_eq!(
        case.expectation.kind,
        crate::expectation::TestKind::Pass
    );
    assert_eq!(
        case.expectation.expected_outcome,
        crate::expectation::ExpectedOutcome::Pass
    );
    assert_eq!(
        case.expectation.expected_phase,
        Some(crate::expectation::PipelinePhase::TypeCheck)
    );
    assert_eq!(case.expectation.tags, ["active_type_elaboration"]);
    assert!(case.expectation.diagnostic_codes.is_empty());
    assert!(case.expectation.diagnostic_payloads.is_empty());
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
