use super::{
    SourceFormulaCompositionRouteInputs, SourceFormulaCompositionRouteOutput,
    extract_source_formula_quantifier_bound_use, source_formula_composition_output,
    source_formula_composition_output_with_mutation,
};

const TASK257B1_CASE: &str = "pass_type_elaboration_formula_quantifier_bound_use_payload_001";

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
fn task257b1_selector_is_exclusive_and_does_not_capture_task257a() {
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
        if source_formula_composition_output(&ast, resolver.module, &symbols).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(selected, [TASK257B1_CASE.to_owned()]);

    let (task257a_ast, task257a_module, task257a_symbols) =
        task252_real_ast("fail_type_elaboration_formula_connective_quantifier_gap_001");
    assert!(
        source_formula_composition_output(&task257a_ast, task257a_module, &task257a_symbols)
            .is_none()
    );
    let (ast, module, symbols) = task252_real_ast(TASK257B1_CASE);
    assert!(source_composite_formula_output(&ast, module, &symbols).is_none());
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
