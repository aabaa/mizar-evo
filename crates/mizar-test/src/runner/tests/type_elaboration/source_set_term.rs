use super::type_elaboration::{
    SyntheticSourceSetTermDependencies, synthetic_source_set_term_output,
    synthetic_source_set_term_output_with_mutation,
};
use super::{
    SetEnumerationBindingMutation, SetEnumerationFinalMutation, SetEnumerationHandoffMutation,
    SetEnumerationPrimaryMutation, SetEnumerationProofContextTestOptions,
    SetEnumerationResolverMutation, SetEnumerationSelectionStage, SetEnumerationSurfaceMutation,
    SourceSetTermRouteOutput, source_set_term_output, source_set_term_output_with_mutation,
    source_set_term_output_with_source, source_set_term_output_with_source_and_mutation,
    TASK258B3M2B2B3P_BINDING_FIELD_COUNT, TASK258B3M2B2B3P_RESOLVER_FIELD_COUNT,
    set_enumeration_proof_context_handoff_for_test, set_enumeration_selection_stage_for_test,
};

const TASK255C1_SET_SOURCE: &str = concat!(
    "import parser.type_fixtures;\n",
    "definition\n",
    "  func Task255ConditionedComprehensionDef:\n",
    "    task255_conditioned_comprehension -> set\n",
    "    equals { 1 ++ 2 where candidate255c is set : 3 = 4 };\n",
    "end;\n",
);

const TASK258B3M2B2B3P_SET_SOURCE: &str = concat!(
    "reserve x for set;\n",
    "theorem FormulaStatementSetEnumerationWitnessSmoke: x = x proof\n",
    "  take {1, 2};\n",
    "  thus x = x;\n",
    "end;\n",
);

#[test]
fn task255c1_real_route_publishes_the_exact_seven_table_transaction() {
    let (ast, module, shells, symbols, source) = task255c1_real_ast();
    assert_eq!(&*source, TASK255C1_SET_SOURCE);
    assert_eq!(source.len(), 191);
    let first = source_set_term_output_with_source(
        &ast,
        module.clone(),
        &shells,
        &symbols,
        &source,
    )
    .expect("Task255C1 exact selector")
    .unwrap_or_else(|error| panic!("Task255C1 real route failed: {error}"));
    let second = source_set_term_output_with_source(&ast, module, &shells, &symbols, &source)
        .expect("Task255C1 repeated selector")
        .unwrap_or_else(|error| panic!("Task255C1 repeated route failed: {error}"));

    let primary = first.typed_ast.source_term().expect("Task252 handoff");
    let application = first
        .typed_ast
        .source_application()
        .expect("Task253 handoff");
    let set = first
        .typed_ast
        .source_set_term()
        .expect("Task255 handoff");
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
    let candidate = application
        .candidates()
        .get(mizar_checker::source_application::SourceFunctorCandidateId::new(0))
        .expect("imported mapper candidate");
    let candidate_entry = symbols
        .symbols()
        .get(candidate.symbol())
        .expect("candidate symbol entry");
    let contribution = symbols
        .contributions()
        .get(candidate.contribution())
        .expect("candidate contribution");
    assert_eq!(candidate_entry.primary_spelling(), "++");
    assert_eq!(
        candidate.symbol().module().path().as_str(),
        "parser.type_fixtures"
    );
    assert!(matches!(
        contribution.kind(),
        mizar_resolve::env::ContributionKind::ImportedSource { .. }
    ));
    assert_eq!(
        application
            .arguments()
            .iter()
            .map(|(_, argument)| argument.target())
            .collect::<Vec<_>>(),
        [
            mizar_checker::source_application::SourceFunctorArgumentTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(0)
            ),
            mizar_checker::source_application::SourceFunctorArgumentTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(1)
            ),
        ]
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
        primary
            .terms()
            .iter()
            .map(|(_, term)| (term.source_range().start, term.source_range().end))
            .collect::<Vec<_>>(),
        [(141, 142), (146, 147), (177, 178), (181, 182)]
    );
    let application_row = application
        .applications()
        .get(mizar_checker::source_application::SourceFunctorApplicationId::new(0))
        .expect("mapper application");
    assert_eq!(
        (
            application_row.source_range().start,
            application_row.source_range().end,
        ),
        (141, 147)
    );
    match application_row.head() {
        mizar_checker::source_application::SourceFunctorHeadSite::Single {
            site,
            source_range,
            spelling,
        } => {
            assert_eq!((source_range.start, source_range.end), (143, 145));
            assert_eq!(spelling, "++");
            assert_eq!(
                first
                    .typed_ast
                    .nodes()
                    .node(site.node())
                    .expect("mapper head arena site")
                    .kind
                    .as_str(),
                "source.term.functor-head.single"
            );
        }
        head => panic!("Task255C1 mapper head must be single, got {head:?}"),
    }
    let set_row = set
        .terms()
        .get(mizar_checker::source_set_term::SourceSetTermId::new(0))
        .expect("conditioned comprehension");
    assert_eq!(
        (set_row.source_range().start, set_row.source_range().end),
        (139, 184)
    );
    let generator = set
        .generators()
        .get(mizar_checker::source_set_term::SourceSetGeneratorId::new(0))
        .expect("conditioned generator");
    assert_eq!(
        (generator.source_range().start, generator.source_range().end),
        (154, 167)
    );
    let generator_segment = task255_node_with_kind_and_spelling(
        &ast,
        &SurfaceNodeKind::ComprehensionVariableSegment,
        "candidate255c is set",
    );
    assert_eq!(
        (
            ast.nodes()[generator_segment].range.start,
            ast.nodes()[generator_segment].range.end,
        ),
        (154, 174)
    );
    let type_site = set
        .type_sites()
        .get(mizar_checker::source_set_term::SourceSetTypeSiteId::new(0))
        .expect("generator type site");
    assert_eq!(
        (type_site.source_range().start, type_site.source_range().end),
        (171, 174)
    );
    let condition = set
        .conditions()
        .get(mizar_checker::source_set_term::SourceSetConditionId::new(0))
        .expect("condition row");
    assert_eq!(condition.term().index(), 0);
    assert_eq!(condition.ordinal(), 0);
    assert_eq!((condition.colon_range().start, condition.colon_range().end), (175, 176));
    assert_eq!(condition.colon_spelling(), ":");
    assert_eq!(
        first
            .typed_ast
            .nodes()
            .node(condition.colon_site().node())
            .expect("condition colon site")
            .kind
            .as_str(),
        "source.term.set.comprehension-condition-colon"
    );
    assert_eq!(
        (condition.source_range().start, condition.source_range().end),
        (177, 182)
    );
    assert_eq!(condition.spelling(), "3 = 4");
    let condition_surface =
        task255_node_with_kind_and_spelling(&ast, &SurfaceNodeKind::FormulaExpression, "3 = 4");
    assert_eq!(condition.condition_site().node().index(), condition_surface);
    assert_eq!(first.typed_ast.nodes().len(), ast.nodes().len());
    assert_eq!(
        first
            .typed_ast
            .nodes()
            .node(application_row.site().node())
            .expect("mapper same-arena site")
            .anchor,
        mizar_session::SourceAnchor::Range(application_row.source_range())
    );
    assert_eq!(
        first
            .typed_ast
            .nodes()
            .node(condition.condition_site().node())
            .expect("condition wrapper site")
            .kind
            .as_str(),
        "source.term.set.comprehension-condition"
    );
    let inner = task255_node_with_kind_and_spelling(
        &ast,
        &SurfaceNodeKind::BuiltinPredicateApplication,
        "3 = 4",
    );
    assert_eq!(
        first
            .typed_ast
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(inner))
            .expect("inner equality site")
            .kind
            .as_str(),
        "source.surface.unowned"
    );
    assert_eq!(
        set.edges()
            .get(mizar_checker::source_set_term::SourceSetEdgeId::new(0))
            .expect("mapper edge")
            .target(),
        mizar_checker::source_set_term::SourceSetTarget::Application(
            mizar_checker::source_application::SourceFunctorApplicationId::new(0)
        )
    );
    assert_eq!(first.typed_ast.source_set_term(), first.resolved.source_set_term());
    assert_eq!(
        first.typed_ast.source_application(),
        first.resolved.source_application()
    );
    assert_eq!(first.typed_ast.source_term(), first.resolved.source_term());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn task255c1_condition_association_mutations_fail_atomically() {
    let (ast, module, shells, symbols, source) = task255c1_real_ast();
    let mutations: [fn(&mut mizar_checker::source_set_term::SourceSetTermHandoffInput); 6] = [
        |input| input.conditions.clear(),
        |input| input.conditions[0].colon_site = input.conditions[0].condition_site.clone(),
        |input| input.conditions[0].condition_site = input.conditions[0].colon_site.clone(),
        |input| input.conditions[0].colon_range.start -= 1,
        |input| input.conditions[0].source_range.end += 1,
        |input| input.conditions[0].spelling = "4 = 3".to_owned(),
    ];
    for mutate in mutations {
        assert!(
            source_set_term_output_with_source_and_mutation(
                &ast,
                module.clone(),
                &shells,
                &symbols,
                &source,
                mutate,
            )
            .expect("mutation must retain the exact selector")
            .is_err()
        );
    }
    assert!(
        source_set_term_output_with_source(&ast, module, &shells, &symbols, &source)
            .expect("uncorrupted exact selector")
            .is_ok()
    );
}

#[test]
fn task255c1_exact_selector_rejects_loaded_source_and_ast_near_misses() {
    let (ast, module, shells, symbols, source) = task255c1_real_ast();
    assert!(
        source_set_term_output_with_source(
            &ast,
            module.clone(),
            &shells,
            &symbols,
            source.trim_end_matches('\n'),
        )
        .is_none(),
        "missing final LF must reject"
    );
    let whitespace_drift = source.replacen("equals {", "equals  {", 1);
    assert!(
        source_set_term_output_with_source(
            &ast,
            module,
            &shells,
            &symbols,
            &whitespace_drift,
        )
        .is_none(),
        "loaded-source whitespace drift must reject"
    );

    let extra_item = format!(
        "{TASK255C1_SET_SOURCE}theorem Task255ConditionedExtraItem: thesis;\n"
    );
    let local_mapper = concat!(
        "import parser.type_fixtures;\n",
        "definition\n",
        "  let x, y be set;\n",
        "  func Task255ConditionedLocalMapperDef:\n",
        "    task255_conditioned_local_mapper(x, y) -> set equals x;\n",
        "  func Task255ConditionedComprehensionDef:\n",
        "    task255_conditioned_comprehension -> set\n",
        "    equals { task255_conditioned_local_mapper(1, 2) where candidate255c is set : 3 = 4 };\n",
        "end;\n",
    )
    .to_owned();
    let template_mapper = concat!(
        "import parser.type_fixtures;\n",
        "definition\n",
        "  let T be type;\n",
        "  let x be T;\n",
        "  func Task255ConditionedTemplateMapperDef:\n",
        "    task255_conditioned_template_mapper[T](x) -> T equals x;\n",
        "  func Task255ConditionedComprehensionDef:\n",
        "    task255_conditioned_comprehension -> set\n",
        "    equals { task255_conditioned_template_mapper[set](1) where candidate255c is set : 3 = 4 };\n",
        "end;\n",
    )
    .to_owned();
    let inline_mapper = concat!(
        "import parser.type_fixtures;\n",
        "deffunc Task255ConditionedInlineMapper(x be set, y being set) -> set equals x;\n",
        "definition\n",
        "  func Task255ConditionedComprehensionDef:\n",
        "    task255_conditioned_comprehension -> set\n",
        "    equals { Task255ConditionedInlineMapper(1, 2) where candidate255c is set : 3 = 4 };\n",
        "end;\n",
    )
    .to_owned();
    let multiple_comprehensions = concat!(
        "import parser.type_fixtures;\n",
        "definition\n",
        "  func Task255ConditionedComprehensionDef:\n",
        "    task255_conditioned_comprehension -> set\n",
        "    equals { 1 ++ 2 where candidate255c is set : 3 = 4 };\n",
        "  func Task255ConditionedComprehensionSecondDef:\n",
        "    task255_conditioned_comprehension_second -> set\n",
        "    equals { 1 ++ 2 where candidate255c is set : 3 = 4 };\n",
        "end;\n",
    )
    .to_owned();
    let near_misses = vec![
        TASK255C1_SET_SOURCE.replacen(
            "import parser.type_fixtures;",
            "import parser.other_type_fixtures;",
            1,
        ),
        extra_item,
        TASK255C1_SET_SOURCE.replacen(
            "Task255ConditionedComprehensionDef",
            "Task255ConditionedComprehensionNearMiss",
            1,
        ),
        TASK255C1_SET_SOURCE.replacen(
            "task255_conditioned_comprehension ->",
            "task255_conditioned_comprehension() ->",
            1,
        ),
        TASK255C1_SET_SOURCE.replacen("1 ++ 2", "2 ++ 1", 1),
        TASK255C1_SET_SOURCE.replacen("1 ++ 2", "(1 ++ 2)", 1),
        TASK255C1_SET_SOURCE.replacen("1 ++ 2", "1 + 2", 1),
        local_mapper,
        template_mapper,
        inline_mapper,
        TASK255C1_SET_SOURCE.replacen(
            " where candidate255c is set",
            "",
            1,
        ),
        TASK255C1_SET_SOURCE.replacen(
            "candidate255c is set",
            "candidate255c is set, extra255c is object",
            1,
        ),
        TASK255C1_SET_SOURCE.replacen(
            "1 ++ 2 where candidate255c",
            "candidate255c where candidate255c",
            1,
        ),
        TASK255C1_SET_SOURCE.replacen(" : 3 = 4", "", 1),
        TASK255C1_SET_SOURCE.replacen(": 3 = 4", ": 3 = 4 & 4 = 3", 1),
        TASK255C1_SET_SOURCE.replacen(": 3 = 4", ": 3 <> 4", 1),
        TASK255C1_SET_SOURCE.replacen(
            "{ 1 ++ 2 where candidate255c is set : 3 = 4 }",
            "{ { 1 ++ 2 where candidate255c is set : 3 = 4 } }",
            1,
        ),
        multiple_comprehensions,
    ];
    for (ordinal, near_source) in near_misses.iter().enumerate() {
        let (near_ast, near_module, near_symbols) =
            task255_ast_from_source_text(near_source, 255_200 + ordinal);
        let near_symbols =
            augment_type_elaboration_import_summaries(&near_ast, &near_module, near_symbols);
        let near_shells =
            mizar_resolve::declarations::DeclarationShellCollector::new(&near_ast, &near_module)
                .collect();
        assert!(
            source_set_term_output_with_source(
                &near_ast,
                near_module.clone(),
                &near_shells,
                &near_symbols,
                TASK255C1_SET_SOURCE,
            )
            .is_none(),
            "AST near miss {ordinal} must reject even under exact loaded bytes"
        );
        assert!(
            source_set_term_output_with_source(
                &near_ast,
                near_module,
                &near_shells,
                &near_symbols,
                near_source,
            )
            .is_none(),
            "raw near miss {ordinal} must reject"
        );
    }
}

#[test]
fn task255c1_conditioned_selector_isolated_to_one_active_case() {
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
    let plan = build_test_plan(&config).expect("Task255C1 isolation plan should build");
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
        let Some(Ok(output)) = source_set_term_output_with_source(
            &ast,
            resolver.module,
            &resolver.shells,
            &symbols,
            &source,
        ) else {
            continue;
        };
        if output
            .typed_ast
            .source_set_term()
            .is_some_and(|handoff| !handoff.conditions().is_empty())
        {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(
        selected,
        ["fail_type_elaboration_conditioned_comprehension_source_payload_001"]
    );
}

#[test]
fn task255_real_route_publishes_exact_aggregate_and_preserves_final_ownership() {
    let (ast, module, shells, symbols) = task255_real_ast();
    let first = source_set_term_output(&ast, module.clone(), &shells, &symbols)
        .expect("Task 255 exact selector")
        .unwrap_or_else(|error| panic!("Task 255 real route failed: {error}"));
    let second = source_set_term_output(&ast, module, &shells, &symbols)
        .expect("Task 255 exact selector should be deterministic")
        .unwrap_or_else(|error| panic!("Task 255 repeated route failed: {error}"));

    assert_task255_real_oracle(
        &ast,
        &first,
        mizar_checker::binding_env::BindingContextId::new(1),
    );
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn task255_real_table_association_corruption_fails_atomically() {
    let (ast, module, shells, symbols) = task255_real_ast();
    let corruptions: [fn(&mut mizar_checker::source_set_term::SourceSetTermHandoffInput); 6] = [
        |input| input.terms.swap(0, 1),
        |input| {
            input.generators[0].type_site =
                mizar_checker::source_set_term::SourceSetTypeSiteId::new(2)
        },
        |input| {
            input.type_sites[0].owner = mizar_checker::source_set_term::SourceSetTypeOwner::Term {
                term: mizar_checker::source_set_term::SourceSetTermId::new(0),
                role: mizar_checker::source_set_term::SourceSetTypeRole::ChoiceTarget,
            }
        },
        |input| input.edges.swap(0, 1),
        |input| input.requests.swap(1, 2),
        |input| input.terms[0].spelling = "{ 2 , 1 }".to_owned(),
    ];
    for corrupt in corruptions {
        assert!(
            source_set_term_output_with_mutation(&ast, module.clone(), &shells, &symbols, corrupt,)
                .expect("corruption must not change the exact selector")
                .is_err(),
            "corrupt Task 255 transaction must fail atomically"
        );
    }
    assert!(
        source_set_term_output(&ast, module, &shells, &symbols)
            .expect("uncorrupted exact selector")
            .is_ok()
    );
}

#[test]
fn task255_synthetic_entry_points_preserve_the_explicit_mutation_boundary() {
    let (ast, module, shells, symbols) = task255_real_ast();
    let real = source_set_term_output(&ast, module.clone(), &shells, &symbols)
        .expect("Task 255 exact selector")
        .expect("Task 255 real route");
    let roots = task255_outer_set_roots(&ast);
    let output = synthetic_source_set_term_output(
        &ast,
        module.clone(),
        real.binding_env.clone(),
        &roots,
        None::<SyntheticSourceSetTermDependencies>,
        &BTreeSet::new(),
    )
    .expect("Task 255 synthetic entry point");
    assert_task255_real_oracle(
        &ast,
        &output,
        mizar_checker::binding_env::BindingContextId::new(0),
    );

    let error = synthetic_source_set_term_output_with_mutation(
        &ast,
        module,
        real.binding_env,
        &roots,
        None::<SyntheticSourceSetTermDependencies>,
        &BTreeSet::new(),
        |input| {
            input.requests.pop();
        },
    )
    .expect_err("Task 255 synthetic partial request table must fail");
    assert!(error.contains("set"), "{error}");
}

#[test]
fn task255_synthetic_shape_matrix_covers_cardinality_nesting_wrappers_and_degradation() {
    let (ast, module, _) = task255_ast_from_source_text(
        r#"
definition
  func Empty255: empty255() -> set equals {};
  func Many255: many255() -> set equals {1, 2, 3};
  func Multi255: multi255() -> set
    equals {4 where alpha255 is set, beta255 is object};
  func Wrapped255: wrapped255() -> set equals {((the object))};
  func Degraded255: degraded255() -> set
    equals {6 where degraded_generator255 is set};
  func Qua255: qua255() -> set equals (7 qua set) qua object;
end;
"#,
        255_001,
    );
    let roots = task255_outer_set_roots(&ast);
    assert_eq!(roots.len(), 6, "six synthetic outer Task-255 roots");
    let degraded_comprehension = task255_node_with_kind_and_spelling(
        &ast,
        &SurfaceNodeKind::SetComprehension,
        "{ 6 where degraded_generator255 is set }",
    );
    let degraded_choice =
        task255_node_with_kind_and_spelling(&ast, &SurfaceNodeKind::ChoiceTerm, "the object");
    let output = synthetic_source_set_term_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module),
        &roots,
        None::<SyntheticSourceSetTermDependencies>,
        &BTreeSet::from([degraded_comprehension, degraded_choice]),
    )
    .expect("Task 255 synthetic shape matrix");
    let handoff = output
        .typed_ast
        .source_set_term()
        .expect("Task 255 handoff");

    assert_eq!(
        (
            handoff.terms().len(),
            handoff.wrappers().len(),
            handoff.generators().len(),
            handoff.type_sites().len(),
            handoff.edges().len(),
            handoff.requests().len(),
        ),
        (8, 3, 3, 6, 8, 14)
    );
    let empty = task255_term_id_with_spelling(handoff, "{ }");
    let many = task255_term_id_with_spelling(handoff, "{ 1 , 2 , 3 }");
    let multi =
        task255_term_id_with_spelling(handoff, "{ 4 where alpha255 is set , beta255 is object }");
    let degraded =
        task255_term_id_with_spelling(handoff, "{ 6 where degraded_generator255 is set }");
    let wrapped_choice = task255_term_id_with_spelling(handoff, "the object");
    let qua_outer = task255_term_id_with_spelling(handoff, "( 7 qua set ) qua object");
    let qua_inner = task255_term_id_with_spelling(handoff, "7 qua set");

    assert_eq!(
        handoff
            .edges()
            .iter()
            .filter(|(_, edge)| edge.term() == empty)
            .count(),
        0
    );
    assert_eq!(
        handoff
            .edges()
            .iter()
            .filter(|(_, edge)| edge.term() == many)
            .count(),
        3
    );
    assert_eq!(
        handoff
            .generators()
            .iter()
            .filter(|(_, generator)| generator.term() == multi)
            .map(|(_, generator)| (generator.ordinal(), generator.spelling()))
            .collect::<Vec<_>>(),
        [(0, "alpha255"), (1, "beta255")]
    );
    assert!(handoff.edges().iter().any(|(_, edge)| {
        edge.term() == qua_outer
            && matches!(
                edge.target(),
                mizar_checker::source_set_term::SourceSetTarget::SetTerm(target)
                    if target == qua_inner
            )
    }));
    assert!(
        handoff.edges().iter().any(|(_, edge)| matches!(
            edge.target(),
            mizar_checker::source_set_term::SourceSetTarget::Primary(_)
        )),
        "synthetic matrix should retain primary children"
    );

    let degraded_wrappers = handoff
        .wrappers()
        .iter()
        .filter(|(_, wrapper)| wrapper.term() == wrapped_choice)
        .map(|(_, wrapper)| (wrapper.ordinal(), wrapper.recovery()))
        .collect::<Vec<_>>();
    assert_eq!(
        degraded_wrappers,
        [
            (
                0,
                mizar_checker::source_set_term::SourceSetTermRecovery::Degraded
            ),
            (
                1,
                mizar_checker::source_set_term::SourceSetTermRecovery::Degraded
            ),
        ]
    );
    assert_eq!(
        handoff
            .terms()
            .get(wrapped_choice)
            .expect("degraded wrapped choice")
            .recovery(),
        mizar_checker::source_set_term::SourceSetTermRecovery::Degraded
    );
    assert_eq!(
        handoff
            .terms()
            .get(degraded)
            .expect("degraded comprehension")
            .recovery(),
        mizar_checker::source_set_term::SourceSetTermRecovery::Degraded
    );
    let degraded_generators = handoff
        .generators()
        .iter()
        .filter(|(_, generator)| generator.term() == degraded)
        .map(|(_, generator)| generator)
        .collect::<Vec<_>>();
    let [degraded_generator] = degraded_generators.as_slice() else {
        panic!("degraded comprehension should have one generator");
    };
    assert_eq!(
        degraded_generator.recovery(),
        mizar_checker::source_set_term::SourceSetTermRecovery::Degraded
    );
    let degraded_type = handoff
        .type_sites()
        .get(degraded_generator.type_site())
        .expect("degraded generator type");
    assert_eq!(
        degraded_type.recovery(),
        mizar_checker::source_set_term::SourceSetTermRecovery::Degraded
    );
    assert_eq!(
        output.typed_ast.source_set_term(),
        output.resolved.source_set_term()
    );
}

#[test]
fn task255_synthetic_nested_independent_comprehensions_preserve_preorder() {
    let (ast, module, _) = task255_ast_from_source_text(
        r#"
definition
  func Nested255: nested255() -> set
    equals {{5 where inner255 is set} where outer255 is object};
end;
"#,
        255_002,
    );
    let roots = task255_outer_set_roots(&ast);
    assert_eq!(roots.len(), 1);
    let output = synthetic_source_set_term_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module),
        &roots,
        None::<SyntheticSourceSetTermDependencies>,
        &BTreeSet::new(),
    )
    .expect("nested independent Task-255 comprehensions");
    let handoff = output
        .typed_ast
        .source_set_term()
        .expect("Task 255 handoff");
    assert_eq!(
        (
            handoff.terms().len(),
            handoff.generators().len(),
            handoff.type_sites().len(),
            handoff.edges().len(),
            handoff.requests().len(),
        ),
        (2, 2, 2, 2, 4)
    );
    let outer = task255_term_id_with_spelling(
        handoff,
        "{ { 5 where inner255 is set } where outer255 is object }",
    );
    let inner = task255_term_id_with_spelling(handoff, "{ 5 where inner255 is set }");
    assert_eq!((outer.index(), inner.index()), (0, 1));
    assert!(handoff.edges().iter().any(|(_, edge)| {
        edge.term() == outer
            && matches!(
                edge.target(),
                mizar_checker::source_set_term::SourceSetTarget::SetTerm(target)
                    if target == inner
            )
    }));
    assert_eq!(
        handoff
            .generators()
            .iter()
            .map(|(_, generator)| (generator.term().index(), generator.ordinal()))
            .collect::<Vec<_>>(),
        [(0, 0), (1, 0)]
    );
}

#[test]
fn task255_synthetic_exclusion_matrix_rejects_deferred_shapes_whole_subtree() {
    let (ast, module, _) = task255_ast_from_source_text(
        r#"
import parser.type_fixtures;
definition
  struct Reverse255 where
    field carrier -> set;
  end;
  func Conditioned255: conditioned255() -> set
    equals {1 where conditioned_generator255 is set : thesis};
  func GeneratorReference255: generator_reference255() -> set
    equals {referenced_generator255 where referenced_generator255 is set};
  func NonBareQua255: nonbare_qua255() -> set
    equals 3 qua T of 4;
  func ReverseApplication255: reverse_application255() -> set
    equals deferred255({5});
  func ReverseStructure255: reverse_structure255() -> set
    equals Reverse255(carrier: {6});
end;
"#,
        255_003,
    );
    let roots = task255_outer_set_roots(&ast);
    assert_eq!(
        roots.len(),
        5,
        "each deferred shape should parse as a set root"
    );
    assert_eq!(
        roots
            .iter()
            .map(|root| (
                std::mem::discriminant(&ast.nodes()[*root].kind),
                task255_subtree_tokens(&ast, &ast.nodes()[*root]).join(" "),
            ))
            .collect::<Vec<_>>(),
        [
            (
                std::mem::discriminant(&SurfaceNodeKind::SetComprehension),
                "{ 1 where conditioned_generator255 is set : thesis }".to_owned(),
            ),
            (
                std::mem::discriminant(&SurfaceNodeKind::SetComprehension),
                "{ referenced_generator255 where referenced_generator255 is set }".to_owned(),
            ),
            (
                std::mem::discriminant(&SurfaceNodeKind::QuaExpression),
                "3 qua T of 4".to_owned(),
            ),
            (
                std::mem::discriminant(&SurfaceNodeKind::SetEnumeration),
                "{ 5 }".to_owned(),
            ),
            (
                std::mem::discriminant(&SurfaceNodeKind::SetEnumeration),
                "{ 6 }".to_owned(),
            ),
        ]
    );
    let nonbare_qua = roots[2];
    let qua_children = task255_structural_children(&ast, nonbare_qua);
    let [_, nonbare_type] = qua_children.as_slice() else {
        panic!(
            "non-bare qua should preserve one base and one target type, got {:?}",
            qua_children
                .iter()
                .map(|child| &ast.nodes()[*child].kind)
                .collect::<Vec<_>>()
        );
    };
    let target_type_children = task255_structural_children(&ast, *nonbare_type);
    let [nonbare_head] = target_type_children.as_slice() else {
        panic!("argument-bearing target type should preserve one head");
    };
    assert!(
        !task255_structural_children(&ast, *nonbare_head).is_empty(),
        "argument-bearing target head should preserve its qualified symbol or arguments"
    );
    assert!(task255_has_ancestor_kind(
        &ast,
        roots[3],
        &SurfaceNodeKind::ApplicationTerm,
    ));
    assert!(task255_has_ancestor_kind(
        &ast,
        roots[4],
        &SurfaceNodeKind::StructureConstructor,
    ));
    let output = synthetic_source_set_term_output(
        &ast,
        module.clone(),
        task254_module_binding_env(&ast, module),
        &roots,
        None::<SyntheticSourceSetTermDependencies>,
        &BTreeSet::new(),
    )
    .expect("Task 255 whole-subtree exclusion matrix");
    let handoff = output
        .typed_ast
        .source_set_term()
        .expect("Task 255 handoff");
    assert_eq!(
        (
            handoff.terms().len(),
            handoff.wrappers().len(),
            handoff.generators().len(),
            handoff.type_sites().len(),
            handoff.edges().len(),
            handoff.requests().len(),
        ),
        (0, 0, 0, 0, 0, 0)
    );
    assert_eq!(handoff.application_fingerprint(), None);
    assert_eq!(handoff.structure_fingerprint(), None);
}

#[test]
fn task255_synthetic_cross_family_children_target_task253_and_task254_roots() {
    let (application_inventory, application_module, _, application_symbols) = task254_real_ast();
    let mut application_syntax = Task254SyntheticSyntax::new(application_inventory.source_id);
    let application_base = application_syntax.primary("4");
    let application_base_range = application_syntax.range(application_base);
    let qua_keyword = application_syntax.token(mizar_syntax::SurfaceTokenKind::ReservedWord, "qua");
    let set_token = application_syntax.token(mizar_syntax::SurfaceTokenKind::Identifier, "set");
    let set_head = application_syntax.node(SurfaceNodeKind::TypeHead, vec![set_token]);
    let set_type = application_syntax.node(SurfaceNodeKind::TypeExpression, vec![set_head]);
    let application_qua = application_syntax.node(
        SurfaceNodeKind::QuaExpression,
        vec![application_base, qua_keyword, set_type],
    );
    let application_root_range = application_syntax.range(application_qua);
    let application_ast = application_syntax.finish(vec![application_qua]);
    let application_root = task254_node_with_range_and_kind(
        &application_ast,
        application_root_range,
        &SurfaceNodeKind::QuaExpression,
    );
    let application_site = task254_node_with_range_and_kind(
        &application_ast,
        application_base_range,
        &SurfaceNodeKind::TermExpression,
    );
    let application_dependencies = task254_bare_application_dependencies(
        &application_ast,
        &application_module,
        &application_symbols,
        application_site,
        Some("4"),
    );
    let application_output = synthetic_source_set_term_output(
        &application_ast,
        application_module.clone(),
        task254_module_binding_env(&application_ast, application_module),
        &[application_root],
        Some(SyntheticSourceSetTermDependencies {
            arena: application_dependencies.arena,
            primary: application_dependencies.primary,
            application: application_dependencies.application,
            structure: None,
        }),
        &BTreeSet::new(),
    )
    .expect("Task 255 Task-253 child target");
    let application_handoff = application_output
        .typed_ast
        .source_set_term()
        .expect("Task 255 application target handoff");
    assert_eq!(
        application_handoff
            .edges()
            .iter()
            .filter(|(_, edge)| matches!(
                edge.target(),
                mizar_checker::source_set_term::SourceSetTarget::Application(_)
            ))
            .count(),
        1
    );
    assert_eq!(
        application_handoff.application_fingerprint(),
        Some(
            application_output
                .typed_ast
                .source_application()
                .expect("installed Task 253")
                .debug_text()
                .as_str()
        )
    );
    assert_eq!(application_handoff.structure_fingerprint(), None);

    let (structure_inventory, structure_module, _, structure_symbols) = task254_real_ast();
    let mut structure_syntax = Task254SyntheticSyntax::new(structure_inventory.source_id);
    let structure_value = structure_syntax.primary("9");
    let structure_constructor =
        structure_syntax.constructor(vec![("carrier", structure_value)], false);
    let structure_range = structure_syntax.range(structure_constructor);
    let mut open_cursor = structure_range.start.saturating_sub(2);
    let open = structure_syntax.token_at(
        mizar_syntax::SurfaceTokenKind::ReservedSymbol,
        "{",
        &mut open_cursor,
    );
    let mut close_cursor = structure_range.end + 1;
    let close = structure_syntax.token_at(
        mizar_syntax::SurfaceTokenKind::ReservedSymbol,
        "}",
        &mut close_cursor,
    );
    let set_enumeration = structure_syntax.node(
        SurfaceNodeKind::SetEnumeration,
        vec![open, structure_constructor, close],
    );
    let set_range = structure_syntax.range(set_enumeration);
    let structure_ast = structure_syntax.finish(vec![set_enumeration]);
    let structure_root = task254_node_with_range_and_kind(
        &structure_ast,
        structure_range,
        &SurfaceNodeKind::StructureConstructor,
    );
    let structure_output = synthetic_source_structure_output(
        &structure_ast,
        structure_module.clone(),
        task254_module_binding_env(&structure_ast, structure_module.clone()),
        &structure_symbols,
        &[structure_root],
        None::<SyntheticSourceStructureDependencies>,
        &BTreeSet::new(),
    )
    .expect("Task 254 dependency for Task 255");
    let set_root = task254_node_with_range_and_kind(
        &structure_ast,
        set_range,
        &SurfaceNodeKind::SetEnumeration,
    );
    let structure_output = synthetic_source_set_term_output(
        &structure_ast,
        structure_module.clone(),
        task254_module_binding_env(&structure_ast, structure_module),
        &[set_root],
        Some(SyntheticSourceSetTermDependencies {
            arena: structure_output.typed_ast.nodes().clone(),
            primary: structure_output
                .typed_ast
                .source_term()
                .expect("Task 252 dependency")
                .clone(),
            application: structure_output.typed_ast.source_application().cloned(),
            structure: structure_output.typed_ast.source_structure().cloned(),
        }),
        &BTreeSet::new(),
    )
    .expect("Task 255 Task-254 child target");
    let structure_handoff = structure_output
        .typed_ast
        .source_set_term()
        .expect("Task 255 structure target handoff");
    assert_eq!(
        structure_handoff
            .edges()
            .iter()
            .filter(|(_, edge)| matches!(
                edge.target(),
                mizar_checker::source_set_term::SourceSetTarget::Structure(_)
            ))
            .count(),
        1
    );
    assert_eq!(
        structure_handoff.structure_fingerprint(),
        Some(
            structure_output
                .typed_ast
                .source_structure()
                .expect("installed Task 254")
                .debug_text()
                .as_str()
        )
    );
    assert_eq!(
        structure_output.typed_ast.source_set_term(),
        structure_output.resolved.source_set_term()
    );
}

#[test]
fn task255_exact_selector_excludes_every_other_active_type_case() {
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
    let plan = build_test_plan(&config).expect("Task 255 isolation plan should build");
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
        if source_set_term_output(&ast, resolver.module, &resolver.shells, &symbols).is_some() {
            selected.push(case.id.0.clone());
        }
    }
    assert_eq!(
        selected,
        ["fail_type_elaboration_local_set_choice_qua_term_gap_001"]
    );
}

fn task255_outer_set_roots(ast: &SurfaceAst) -> Vec<usize> {
    let mut parents = vec![None; ast.nodes().len()];
    for (parent, node) in ast.nodes().iter().enumerate() {
        for child in &node.children {
            parents[child.index()] = Some(parent);
        }
    }
    ast.nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            matches!(
                node.kind,
                SurfaceNodeKind::SetEnumeration
                    | SurfaceNodeKind::SetComprehension
                    | SurfaceNodeKind::ChoiceTerm
                    | SurfaceNodeKind::QuaExpression
            )
        })
        .filter(|(index, _)| {
            let mut cursor = parents[*index];
            while let Some(parent) = cursor {
                if matches!(
                    ast.nodes()[parent].kind,
                    SurfaceNodeKind::SetEnumeration
                        | SurfaceNodeKind::SetComprehension
                        | SurfaceNodeKind::ChoiceTerm
                        | SurfaceNodeKind::QuaExpression
                ) {
                    return false;
                }
                cursor = parents[parent];
            }
            true
        })
        .map(|(index, _)| index)
        .collect()
}

fn assert_task255_real_oracle(
    ast: &SurfaceAst,
    output: &SourceSetTermRouteOutput,
    context: mizar_checker::binding_env::BindingContextId,
) {
    let handoff = output
        .typed_ast
        .source_set_term()
        .expect("Task 255 handoff");
    let primary = output.typed_ast.source_term().expect("Task 252 handoff");
    let term_nodes = [
        task255_node_with_kind_and_spelling(ast, &SurfaceNodeKind::SetEnumeration, "{ 1 , 2 }"),
        task255_node_with_kind_and_spelling(
            ast,
            &SurfaceNodeKind::SetComprehension,
            "{ 3 where candidate255 is set }",
        ),
        task255_node_with_kind_and_spelling(ast, &SurfaceNodeKind::ChoiceTerm, "the set"),
        task255_node_with_kind_and_spelling(ast, &SurfaceNodeKind::QuaExpression, "4 qua set"),
    ];
    let comprehension_children = task255_structural_children(ast, term_nodes[1]);
    let [_, generator_node] = comprehension_children.as_slice() else {
        panic!("Task 255 comprehension should have one mapper and one generator");
    };
    let generator_children = task255_structural_children(ast, *generator_node);
    let [generator_type] = generator_children.as_slice() else {
        panic!("Task 255 generator should have one written type");
    };
    let choice_children = task255_structural_children(ast, term_nodes[2]);
    let [choice_type] = choice_children.as_slice() else {
        panic!("Task 255 choice should have one target type");
    };
    let qua_children = task255_structural_children(ast, term_nodes[3]);
    let [_, qua_type] = qua_children.as_slice() else {
        panic!("Task 255 qua should have one base and one target type");
    };
    let type_nodes = [*generator_type, *choice_type, *qua_type];
    let type_head_nodes = type_nodes.map(|node| {
        let children = task255_structural_children(ast, node);
        let [head] = children.as_slice() else {
            panic!("Task 255 target type should have one bare head");
        };
        *head
    });
    let generator_site = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| node.token_text() == Some("candidate255"))
        .map(|(index, _)| index)
        .next()
        .expect("Task 255 generator identifier");
    assert_eq!(
        (
            handoff.terms().len(),
            handoff.wrappers().len(),
            handoff.generators().len(),
            handoff.type_sites().len(),
            handoff.edges().len(),
            handoff.requests().len(),
        ),
        (4, 0, 1, 3, 4, 7)
    );
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (4, 0, 4)
    );
    assert_eq!(
        handoff
            .terms()
            .iter()
            .map(|(id, term)| {
                (
                    id.index(),
                    term.site(),
                    term.source_range(),
                    term.source_ordinal(),
                    term.context(),
                    term.recovery(),
                    term.kind(),
                    term.spelling(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(term_nodes[0]),
                ),
                ast.nodes()[term_nodes[0]].range,
                0,
                context,
                mizar_checker::source_set_term::SourceSetTermRecovery::Normal,
                mizar_checker::source_set_term::SourceSetTermKind::Enumeration,
                "{ 1 , 2 }",
            ),
            (
                1,
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(term_nodes[1]),
                ),
                ast.nodes()[term_nodes[1]].range,
                1,
                context,
                mizar_checker::source_set_term::SourceSetTermRecovery::Normal,
                mizar_checker::source_set_term::SourceSetTermKind::Comprehension,
                "{ 3 where candidate255 is set }",
            ),
            (
                2,
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(term_nodes[2]),
                ),
                ast.nodes()[term_nodes[2]].range,
                2,
                context,
                mizar_checker::source_set_term::SourceSetTermRecovery::Normal,
                mizar_checker::source_set_term::SourceSetTermKind::Choice,
                "the set",
            ),
            (
                3,
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(term_nodes[3]),
                ),
                ast.nodes()[term_nodes[3]].range,
                3,
                context,
                mizar_checker::source_set_term::SourceSetTermRecovery::Normal,
                mizar_checker::source_set_term::SourceSetTermKind::Qua,
                "4 qua set",
            ),
        ]
    );
    let (generator_id, generator) = handoff
        .generators()
        .iter()
        .next()
        .expect("Task 255 comprehension generator");
    assert_eq!(generator_id.index(), 0);
    assert_eq!(
        generator.term(),
        mizar_checker::source_set_term::SourceSetTermId::new(1)
    );
    assert_eq!(generator.spelling(), "candidate255");
    assert_eq!(generator.ordinal(), 0);
    assert_eq!(
        generator.site(),
        &mizar_checker::typed_ast::TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(
            generator_site
        ),)
    );
    assert_eq!(generator.source_range(), ast.nodes()[generator_site].range);
    assert_eq!(generator.context(), context);
    assert_eq!(
        generator.recovery(),
        mizar_checker::source_set_term::SourceSetTermRecovery::Normal
    );
    assert_eq!(
        generator.type_site(),
        mizar_checker::source_set_term::SourceSetTypeSiteId::new(0)
    );
    assert_eq!(
        handoff
            .type_sites()
            .iter()
            .map(|(id, site)| {
                (
                    id.index(),
                    site.owner(),
                    site.site(),
                    site.source_range(),
                    site.spelling(),
                    site.head_site(),
                    site.head_range(),
                    site.head_spelling(),
                    site.context(),
                    site.recovery(),
                    site.head(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                mizar_checker::source_set_term::SourceSetTypeOwner::Generator(
                    mizar_checker::source_set_term::SourceSetGeneratorId::new(0),
                ),
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(type_nodes[0]),
                ),
                ast.nodes()[type_nodes[0]].range,
                "set",
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(type_head_nodes[0]),
                ),
                ast.nodes()[type_head_nodes[0]].range,
                "set",
                context,
                mizar_checker::source_set_term::SourceSetTermRecovery::Normal,
                mizar_checker::source_set_term::SourceSetTypeHead::BuiltinSet,
            ),
            (
                1,
                mizar_checker::source_set_term::SourceSetTypeOwner::Term {
                    term: mizar_checker::source_set_term::SourceSetTermId::new(2),
                    role: mizar_checker::source_set_term::SourceSetTypeRole::ChoiceTarget,
                },
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(type_nodes[1]),
                ),
                ast.nodes()[type_nodes[1]].range,
                "set",
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(type_head_nodes[1]),
                ),
                ast.nodes()[type_head_nodes[1]].range,
                "set",
                context,
                mizar_checker::source_set_term::SourceSetTermRecovery::Normal,
                mizar_checker::source_set_term::SourceSetTypeHead::BuiltinSet,
            ),
            (
                2,
                mizar_checker::source_set_term::SourceSetTypeOwner::Term {
                    term: mizar_checker::source_set_term::SourceSetTermId::new(3),
                    role: mizar_checker::source_set_term::SourceSetTypeRole::QuaTarget,
                },
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(type_nodes[2]),
                ),
                ast.nodes()[type_nodes[2]].range,
                "set",
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(type_head_nodes[2]),
                ),
                ast.nodes()[type_head_nodes[2]].range,
                "set",
                context,
                mizar_checker::source_set_term::SourceSetTermRecovery::Normal,
                mizar_checker::source_set_term::SourceSetTypeHead::BuiltinSet,
            ),
        ]
    );
    assert_eq!(
        handoff
            .edges()
            .iter()
            .map(|(id, edge)| {
                (
                    id.index(),
                    edge.term(),
                    edge.ordinal(),
                    edge.role(),
                    edge.target(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                mizar_checker::source_set_term::SourceSetTermId::new(0),
                0,
                mizar_checker::source_set_term::SourceSetEdgeRole::EnumerationElement,
                mizar_checker::source_set_term::SourceSetTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(0),
                ),
            ),
            (
                1,
                mizar_checker::source_set_term::SourceSetTermId::new(0),
                1,
                mizar_checker::source_set_term::SourceSetEdgeRole::EnumerationElement,
                mizar_checker::source_set_term::SourceSetTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(1),
                ),
            ),
            (
                2,
                mizar_checker::source_set_term::SourceSetTermId::new(1),
                0,
                mizar_checker::source_set_term::SourceSetEdgeRole::ComprehensionMapper,
                mizar_checker::source_set_term::SourceSetTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2),
                ),
            ),
            (
                3,
                mizar_checker::source_set_term::SourceSetTermId::new(3),
                0,
                mizar_checker::source_set_term::SourceSetEdgeRole::QuaBase,
                mizar_checker::source_set_term::SourceSetTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(3),
                ),
            ),
        ]
    );
    assert_eq!(
        handoff
            .requests()
            .iter()
            .map(|(id, request)| {
                (
                    id.index(),
                    request.term(),
                    request.ordinal(),
                    request.kind(),
                    request.generator(),
                    request.type_site(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                mizar_checker::source_set_term::SourceSetTermId::new(0),
                0,
                mizar_checker::source_set_term::SourceSetRequestKind::ResultType,
                None,
                None,
            ),
            (
                1,
                mizar_checker::source_set_term::SourceSetTermId::new(1),
                0,
                mizar_checker::source_set_term::SourceSetRequestKind::GeneratorSethood,
                Some(mizar_checker::source_set_term::SourceSetGeneratorId::new(0)),
                Some(mizar_checker::source_set_term::SourceSetTypeSiteId::new(0)),
            ),
            (
                2,
                mizar_checker::source_set_term::SourceSetTermId::new(1),
                1,
                mizar_checker::source_set_term::SourceSetRequestKind::ResultType,
                None,
                None,
            ),
            (
                3,
                mizar_checker::source_set_term::SourceSetTermId::new(2),
                0,
                mizar_checker::source_set_term::SourceSetRequestKind::ChoiceNonempty,
                None,
                Some(mizar_checker::source_set_term::SourceSetTypeSiteId::new(1)),
            ),
            (
                4,
                mizar_checker::source_set_term::SourceSetTermId::new(2),
                1,
                mizar_checker::source_set_term::SourceSetRequestKind::ResultType,
                None,
                None,
            ),
            (
                5,
                mizar_checker::source_set_term::SourceSetTermId::new(3),
                0,
                mizar_checker::source_set_term::SourceSetRequestKind::QuaWidening,
                None,
                Some(mizar_checker::source_set_term::SourceSetTypeSiteId::new(2)),
            ),
            (
                6,
                mizar_checker::source_set_term::SourceSetTermId::new(3),
                1,
                mizar_checker::source_set_term::SourceSetRequestKind::ResultType,
                None,
                None,
            ),
        ]
    );
    assert_eq!(handoff.application_fingerprint(), None);
    assert_eq!(handoff.structure_fingerprint(), None);
    assert_eq!(
        handoff.primary_term_fingerprint(),
        primary.debug_text().as_str()
    );
    assert_eq!(output.binding_env.contexts().len(), 2);
    assert_eq!(output.binding_env.bindings().len(), 2);
    assert!(output.typed_ast.source_application().is_none());
    assert!(output.typed_ast.source_structure().is_none());
    assert_eq!(
        output.typed_ast.source_set_term(),
        output.resolved.source_set_term()
    );
    assert_eq!(
        output.typed_ast.source_term(),
        output.resolved.source_term()
    );
    assert!(output.typed_ast.types().is_empty());
    assert!(output.typed_ast.facts().is_empty());
    assert!(output.typed_ast.coercions().is_empty());
    assert!(output.typed_ast.initial_obligations().is_empty());
    assert!(output.typed_ast.diagnostics().is_empty());
}

fn task255_real_ast() -> (
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
    let plan = build_test_plan(&config).expect("Task 255 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == "fail_type_elaboration_local_set_choice_qua_term_gap_001")
        .expect("Task 255 case should remain active");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .unwrap_or_else(|error| panic!("Task 255 frontend failed: {error}"));
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let ast = frontend.ast.expect("Task 255 AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    let module = resolver.module;
    let shells = resolver.shells;
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
    (ast, module, shells, symbols)
}

fn task255c1_real_ast() -> (
    SurfaceAst,
    ResolverModuleId,
    mizar_resolve::declarations::DeclarationShellSet,
    SymbolEnv,
    std::sync::Arc<str>,
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
    let plan = build_test_plan(&config).expect("Task255C1 repository plan should build");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| {
            case.id.0 == "fail_type_elaboration_conditioned_comprehension_source_payload_001"
        })
        .expect("Task255C1 case should remain active");
    let frontend = run_frontend(&workspace_root, case, ordinal)
        .unwrap_or_else(|error| panic!("Task255C1 frontend failed: {error}"));
    assert!(
        frontend.diagnostics.is_empty(),
        "{:?}",
        frontend.diagnostics
    );
    let source = frontend.source_text.clone();
    let ast = frontend.ast.expect("Task255C1 AST");
    let resolver = resolver_symbol_collection(&workspace_root, case, &ast);
    assert!(
        resolver.detail_keys.is_empty(),
        "{:?}",
        resolver.detail_keys
    );
    let module = resolver.module;
    let shells = resolver.shells;
    let symbols = augment_type_elaboration_import_summaries(&ast, &module, resolver.env);
    (ast, module, shells, symbols, source)
}

fn task255_node_with_kind_and_spelling(
    ast: &SurfaceAst,
    kind: &SurfaceNodeKind,
    spelling: &str,
) -> usize {
    let matches = ast
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, node)| std::mem::discriminant(&node.kind) == std::mem::discriminant(kind))
        .filter(|(_, node)| task255_subtree_tokens(ast, node).join(" ") == spelling)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let [index] = matches.as_slice() else {
        panic!("Task 255 `{spelling}` node should be unique, got {matches:?}");
    };
    *index
}

fn task255_structural_children(ast: &SurfaceAst, node: usize) -> Vec<usize> {
    ast.nodes()[node]
        .children
        .iter()
        .filter_map(|child| {
            ast.node(*child)
                .filter(|child| !matches!(child.kind, SurfaceNodeKind::Token(_)))
                .map(|_| child.index())
        })
        .collect()
}

fn task255_has_ancestor_kind(ast: &SurfaceAst, node: usize, kind: &SurfaceNodeKind) -> bool {
    let mut parents = vec![None; ast.nodes().len()];
    for (parent, node) in ast.nodes().iter().enumerate() {
        for child in &node.children {
            parents[child.index()] = Some(parent);
        }
    }
    let mut cursor = parents[node];
    while let Some(parent) = cursor {
        if std::mem::discriminant(&ast.nodes()[parent].kind) == std::mem::discriminant(kind) {
            return true;
        }
        cursor = parents[parent];
    }
    false
}

fn task255_subtree_tokens<'a>(
    ast: &'a SurfaceAst,
    node: &'a mizar_syntax::SurfaceNode,
) -> Vec<&'a str> {
    let mut tokens = Vec::new();
    fn collect<'a>(
        ast: &'a SurfaceAst,
        node: &'a mizar_syntax::SurfaceNode,
        tokens: &mut Vec<&'a str>,
    ) {
        if let Some(token) = node.token_text() {
            tokens.push(token);
            return;
        }
        for child in &node.children {
            if let Some(child) = ast.node(*child) {
                collect(ast, child, tokens);
            }
        }
    }
    collect(ast, node, &mut tokens);
    tokens
}

fn task255_ast_from_source_text(
    source: &str,
    ordinal: usize,
) -> (SurfaceAst, ResolverModuleId, SymbolEnv) {
    let (ast, module, _, symbols) = task253_ast_from_source_text(source, ordinal);
    (ast, module, symbols)
}

fn task255_term_id_with_spelling(
    handoff: &mizar_checker::source_set_term::SourceSetTermHandoff,
    spelling: &str,
) -> mizar_checker::source_set_term::SourceSetTermId {
    let matches = handoff
        .terms()
        .iter()
        .filter(|(_, term)| term.spelling() == spelling)
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    let [id] = matches.as_slice() else {
        panic!("Task 255 handoff term `{spelling}` should be unique, got {matches:?}");
    };
    *id
}

#[test]
fn task258b3m2b2b3p_set_enumeration_proof_context_reuse_is_exact() {
    use mizar_checker::source_set_term::{
        SourceSetEdgeRole, SourceSetRequestKind, SourceSetTarget, SourceSetTermId,
        SourceSetTermKind, SourceSetTermRecovery,
    };
    let (ast, module, shells, symbols, diagnostic_count) =
        task253_ast_from_source_text_with_diagnostic_count(
            TASK258B3M2B2B3P_SET_SOURCE,
            25_831,
        );
    assert_eq!(diagnostic_count, 0);
    assert_eq!(TASK258B3M2B2B3P_SET_SOURCE.len(), 117);
    assert_eq!(
        sha256_text(TASK258B3M2B2B3P_SET_SOURCE),
        "4f8ea5b9cadf763ea108b6f7deb6b481cb6f997dec2048b4351f07fd5dc38539"
    );
    assert_eq!((ast.nodes().len(), ast.root().map(|root| root.index())), (57, Some(56)));
    let bindings = task258b3m2b2b1p_binding_env(&ast, &module, &symbols);
    let first = set_enumeration_proof_context_handoff_for_test(
        &ast,
        &module,
        &shells,
        &symbols,
        &bindings,
        TASK258B3M2B2B3P_SET_SOURCE,
        SetEnumerationProofContextTestOptions::default(),
    )
    .expect("B3P exact selector")
    .expect("B3P exact transaction");
    let second = set_enumeration_proof_context_handoff_for_test(
        &ast,
        &module,
        &shells,
        &symbols,
        &bindings,
        TASK258B3M2B2B3P_SET_SOURCE,
        SetEnumerationProofContextTestOptions::default(),
    )
    .expect("B3P replay selector")
    .expect("B3P replay transaction");

    assert_eq!(
        (
            first.binding_env.contexts().len(),
            first.binding_env.bindings().len(),
            first.binding_env.diagnostics().len(),
        ),
        (2, 1, 0)
    );
    let primary = first.typed_ast.source_term().expect("Task252 handoff");
    assert_eq!(
        (
            primary.terms().len(),
            primary.references().len(),
            primary.numeric_type_requests().len(),
        ),
        (6, 4, 2)
    );
    let set = first.typed_ast.source_set_term().expect("Task255 handoff");
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
        (1, 0, 0, 0, 0, 2, 1)
    );
    let term = set
        .terms()
        .get(SourceSetTermId::new(0))
        .expect("enumeration term");
    assert_eq!(term.site().node().index(), 40);
    assert_eq!(
        (term.source_range().start, term.source_range().end),
        (90, 96)
    );
    assert_eq!(term.source_ordinal(), 0);
    assert_eq!(term.context().index(), 1);
    assert_eq!(term.recovery(), SourceSetTermRecovery::Normal);
    assert_eq!(term.spelling(), "{ 1 , 2 }");
    assert_eq!(term.kind(), SourceSetTermKind::Enumeration);
    assert_eq!(
        set.edges()
            .iter()
            .map(|(_, edge)| {
                (
                    edge.term().index(),
                    edge.ordinal(),
                    edge.role(),
                    edge.target(),
                )
            })
            .collect::<Vec<_>>(),
        [
            (
                0,
                0,
                SourceSetEdgeRole::EnumerationElement,
                SourceSetTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(2)
                ),
            ),
            (
                0,
                1,
                SourceSetEdgeRole::EnumerationElement,
                SourceSetTarget::Primary(
                    mizar_checker::source_term::SourcePrimaryTermId::new(3)
                ),
            ),
        ]
    );
    let request = set
        .requests()
        .iter()
        .next()
        .map(|(_, request)| request)
        .expect("result-type request");
    assert_eq!(request.term().index(), 0);
    assert_eq!(request.ordinal(), 0);
    assert_eq!(request.kind(), SourceSetRequestKind::ResultType);
    assert_eq!(request.generator(), None);
    assert_eq!(request.type_site(), None);
    assert_eq!(set.primary_term_fingerprint(), primary.debug_text());
    assert_eq!(set.application_fingerprint(), None);
    assert_eq!(set.structure_fingerprint(), None);

    let owned = first
        .typed_ast
        .nodes()
        .iter()
        .filter(|(_, node)| node.kind.as_str() != "source.surface.unowned")
        .map(|(id, node)| (id.index(), node.kind.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        owned,
        [
            (30, "source.term.variable-reference"),
            (32, "source.term.variable-reference"),
            (36, "source.term.numeral"),
            (38, "source.term.numeral"),
            (40, "source.term.set.enumeration"),
            (44, "source.term.variable-reference"),
            (46, "source.term.variable-reference"),
        ]
    );
    assert_eq!(
        first
            .typed_ast
            .nodes()
            .iter()
            .filter(|(_, node)| node.kind.as_str() == "source.surface.unowned")
            .map(|(id, _)| id.index())
            .collect::<Vec<_>>(),
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            21, 22, 23, 24, 25, 26, 27, 28, 29, 31, 33, 34, 35, 37, 39, 41, 42, 43, 45,
            47, 48, 49, 50, 51, 52, 53, 54, 55, 56,
        ]
    );
    assert_eq!(first.typed_ast.source_term(), first.resolved.source_term());
    assert_eq!(
        first.typed_ast.source_set_term(),
        first.resolved.source_set_term()
    );
    assert!(first.typed_ast.source_context().is_none());
    assert!(first.typed_ast.source_type().is_none());
    assert!(first.typed_ast.source_attribute().is_none());
    assert!(first.typed_ast.source_evidence().is_none());
    assert!(first.typed_ast.source_application().is_none());
    assert!(first.typed_ast.source_structure().is_none());
    assert!(first.typed_ast.source_atomic_formula().is_none());
    assert!(first.typed_ast.source_composite_formula().is_none());
    assert!(first.typed_ast.source_formula_composition().is_none());
    assert!(
        first
            .typed_ast
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        first
            .typed_ast
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(first.typed_ast.source_statement().is_none());
    assert!(first.typed_ast.source_statement_references().is_none());
    assert!(first.typed_ast.source_statement_witnesses().is_none());
    assert!(first.typed_ast.contexts().is_empty());
    assert!(first.typed_ast.types().is_empty());
    assert!(first.typed_ast.facts().is_empty());
    assert!(first.typed_ast.coercions().is_empty());
    assert!(first.typed_ast.initial_obligations().is_empty());
    assert!(first.typed_ast.diagnostics().is_empty());
    assert!(first.resolved.source_context().is_none());
    assert!(first.resolved.source_type().is_none());
    assert!(first.resolved.source_attribute().is_none());
    assert!(first.resolved.source_evidence().is_none());
    assert!(first.resolved.source_application().is_none());
    assert!(first.resolved.source_structure().is_none());
    assert!(first.resolved.source_atomic_formula().is_none());
    assert!(first.resolved.source_composite_formula().is_none());
    assert!(first.resolved.source_formula_composition().is_none());
    assert!(
        first
            .resolved
            .source_condition_formula_composition()
            .is_none()
    );
    assert!(
        first
            .resolved
            .source_predicate_chain_composition()
            .is_none()
    );
    assert!(first.resolved.source_statement().is_none());
    assert!(first.resolved.source_statement_references().is_none());
    assert!(first.resolved.source_statement_witnesses().is_none());
    assert!(first.resolved.checked_formulas().is_empty());
    assert!(first.resolved.checked_proofs().is_empty());
    assert!(first.resolved.checked_proof_nodes().is_empty());
    assert!(first.resolved.checked_terminal_goals().is_empty());
    assert!(first.resolved.statement_semantics().is_empty());
    assert!(first.resolved.expr_metadata().is_empty());
    assert!(first.resolved.collection_candidates().is_empty());
    assert!(first.resolved.expanded_candidates().is_empty());
    assert!(first.resolved.template_expansions().is_empty());
    assert!(first.resolved.viable_candidates().is_empty());
    assert!(first.resolved.viability_decisions().is_empty());
    assert!(first.resolved.specificity_graphs().is_empty());
    assert!(first.resolved.resolved_overloads().is_empty());
    assert!(first.resolved.inserted_coercions().is_empty());
    assert!(first.resolved.cluster_facts().is_empty());
    assert!(first.resolved.diagnostics().is_empty());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
}

#[test]
fn task258b3m2b2b3p_set_enumeration_corruption_replay_and_legacy_output_fail_closed() {
    let (ast, module, shells, symbols, diagnostic_count) =
        task253_ast_from_source_text_with_diagnostic_count(
            TASK258B3M2B2B3P_SET_SOURCE,
            25_832,
        );
    assert_eq!(diagnostic_count, 0);
    let bindings = task258b3m2b2b1p_binding_env(&ast, &module, &symbols);
    let run = |options| {
        set_enumeration_proof_context_handoff_for_test(
            &ast,
            &module,
            &shells,
            &symbols,
            &bindings,
            TASK258B3M2B2B3P_SET_SOURCE,
            options,
        )
    };
    let baseline = run(SetEnumerationProofContextTestOptions::default())
        .expect("B3P baseline selector")
        .expect("B3P baseline");
    let baseline_set = baseline
        .typed_ast
        .source_set_term()
        .expect("B3P set handoff")
        .debug_text();
    let baseline_typed = baseline.typed_ast.debug_text();
    let baseline_resolved = baseline.resolved.debug_text();
    assert!(
        source_set_term_output(&ast, module.clone(), &shells, &symbols).is_none(),
        "B3P must not activate the legacy Task255 production route"
    );
    let reject = |options, label: &str| match run(options) {
        None | Some(Err(_)) => {}
        Some(Ok(_)) => panic!("{label} was accepted"),
    };
    let clean_replay = || {
        let replay = run(SetEnumerationProofContextTestOptions::default())
            .expect("B3P replay selector")
            .expect("B3P replay");
        assert_eq!(
            replay
                .typed_ast
                .source_set_term()
                .expect("replay set")
                .debug_text(),
            baseline_set
        );
        assert_eq!(replay.typed_ast.debug_text(), baseline_typed);
        assert_eq!(replay.resolved.debug_text(), baseline_resolved);
    };

    for byte in 0..TASK258B3M2B2B3P_SET_SOURCE.len() {
        let mut source = TASK258B3M2B2B3P_SET_SOURCE.as_bytes().to_vec();
        source[byte] = if source[byte] == b'!' { b'?' } else { b'!' };
        let source = String::from_utf8(source).expect("ASCII B3P source");
        assert!(
            set_enumeration_proof_context_handoff_for_test(
                &ast,
                &module,
                &shells,
                &symbols,
                &bindings,
                &source,
                SetEnumerationProofContextTestOptions::default(),
            )
            .is_none(),
            "loaded-source byte {byte}"
        );
        clean_replay();
    }
    clean_replay();
    for source in [
        TASK258B3M2B2B3P_SET_SOURCE
            .trim_end_matches('\n')
            .to_owned(),
        format!("{TASK258B3M2B2B3P_SET_SOURCE}\n"),
    ] {
        assert!(
            set_enumeration_proof_context_handoff_for_test(
                &ast,
                &module,
                &shells,
                &symbols,
                &bindings,
                &source,
                SetEnumerationProofContextTestOptions::default(),
            )
            .is_none()
        );
        clean_replay();
    }
    clean_replay();
    for (ordinal, source) in [
        TASK258B3M2B2B3P_SET_SOURCE.replacen("{1, 2}", "{}", 1),
        TASK258B3M2B2B3P_SET_SOURCE.replacen("{1, 2}", "{1}", 1),
        TASK258B3M2B2B3P_SET_SOURCE.replacen("{1, 2}", "{1, 2, 3}", 1),
        TASK258B3M2B2B3P_SET_SOURCE.replacen("{1, 2}", "({1, 2})", 1),
        TASK258B3M2B2B3P_SET_SOURCE.replacen("{1, 2}", "{{1}, 2}", 1),
        TASK258B3M2B2B3P_SET_SOURCE.replacen("{1, 2}", "{1 where y is set}", 1),
        TASK258B3M2B2B3P_SET_SOURCE.replacen("{1, 2}", "the set", 1),
        TASK258B3M2B2B3P_SET_SOURCE.replacen("{1, 2}", "1 qua set", 1),
        TASK258B3M2B2B3P_SET_SOURCE.replacen(
            "FormulaStatementSetEnumerationWitnessSmoke",
            "FormulaStatementSetEnumerationWitnessNearMiss",
            1,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (near_ast, near_module, near_shells, near_symbols, _) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 25_900 + ordinal);
        assert!(
            set_enumeration_proof_context_handoff_for_test(
                &near_ast,
                &near_module,
                &near_shells,
                &near_symbols,
                &bindings,
                &source,
                SetEnumerationProofContextTestOptions::default(),
            )
            .is_none(),
            "parsed near miss {ordinal}"
        );
        assert!(
            source_set_term_output(
                &near_ast,
                near_module,
                &near_shells,
                &near_symbols
            )
            .is_none(),
            "parsed near miss {ordinal} activated legacy Task255"
        );
        clean_replay();
    }
    clean_replay();
    for node in 0..57 {
        for surface in [
            SetEnumerationSurfaceMutation::NodeKind(node),
            SetEnumerationSurfaceMutation::NodeRange(node),
            SetEnumerationSurfaceMutation::NodeRecovery(node),
            SetEnumerationSurfaceMutation::NodeChildren(node),
        ] {
            reject(
                SetEnumerationProofContextTestOptions {
                    surface,
                    ..Default::default()
                },
                "surface substitution",
            );
            clean_replay();
        }
    }
    clean_replay();
    reject(
        SetEnumerationProofContextTestOptions {
            surface: SetEnumerationSurfaceMutation::RootIdentity,
            ..Default::default()
        },
        "root substitution",
    );
    clean_replay();
    for field in 0..TASK258B3M2B2B3P_RESOLVER_FIELD_COUNT {
        reject(
            SetEnumerationProofContextTestOptions {
                resolver: SetEnumerationResolverMutation::Field(field),
                ..Default::default()
            },
            "resolver substitution",
        );
        clean_replay();
    }
    clean_replay();
    for field in 0..TASK258B3M2B2B3P_BINDING_FIELD_COUNT {
        let result = run(SetEnumerationProofContextTestOptions {
            binding: SetEnumerationBindingMutation::Field(field),
            ..Default::default()
        })
        .expect("Task48 mutation retains source/resolver selection")
        .expect_err("Task48 mutation");
        assert!(result.starts_with("Task48:"), "{field}: {result}");
        clean_replay();
    }
    clean_replay();

    let mut primary_mutations = vec![
        SetEnumerationPrimaryMutation::DuplicateRoot,
        SetEnumerationPrimaryMutation::MissingRoot,
        SetEnumerationPrimaryMutation::SourceId,
        SetEnumerationPrimaryMutation::ModuleId,
        SetEnumerationPrimaryMutation::ReferenceScopeModule,
        SetEnumerationPrimaryMutation::ReferenceScopeProof,
        SetEnumerationPrimaryMutation::ReferenceUseOrdinal,
    ];
    for term in 0..6 {
        primary_mutations.extend([
            SetEnumerationPrimaryMutation::TermSite(term),
            SetEnumerationPrimaryMutation::TermRange(term),
            SetEnumerationPrimaryMutation::TermOrdinal(term),
            SetEnumerationPrimaryMutation::TermContext(term),
            SetEnumerationPrimaryMutation::TermRecovery(term),
            SetEnumerationPrimaryMutation::TermSpelling(term),
            SetEnumerationPrimaryMutation::TermKind(term),
            SetEnumerationPrimaryMutation::TermRole(term),
            SetEnumerationPrimaryMutation::TermParent(term),
        ]);
    }
    for reference in 0..4 {
        primary_mutations.extend([
            SetEnumerationPrimaryMutation::ReferenceTerm(reference),
            SetEnumerationPrimaryMutation::ReferenceBinding(reference),
            SetEnumerationPrimaryMutation::ReferenceRole(reference),
        ]);
    }
    for request in 0..2 {
        primary_mutations.extend([
            SetEnumerationPrimaryMutation::NumericTerm(request),
            SetEnumerationPrimaryMutation::NumericOwner(request),
            SetEnumerationPrimaryMutation::NumericRange(request),
            SetEnumerationPrimaryMutation::NumericSpelling(request),
            SetEnumerationPrimaryMutation::NumericOrdinal(request),
        ]);
    }
    for primary in primary_mutations {
        let error = run(SetEnumerationProofContextTestOptions {
            primary,
            ..Default::default()
        })
        .expect("Task252 mutation retains earlier selection")
        .expect_err("Task252 corruption");
        assert!(error.starts_with("Task252:"), "{primary:?}: {error}");
        if primary == SetEnumerationPrimaryMutation::ReferenceUseOrdinal {
            assert!(
                error.contains("exact lower profile mismatch"),
                "real four-row use-ordinal substitution must reach the shared exact profile: {error}"
            );
        }
        clean_replay();
    }

    let mut handoff_mutations = vec![
        SetEnumerationHandoffMutation::SourceId,
        SetEnumerationHandoffMutation::ModuleId,
        SetEnumerationHandoffMutation::TermSite,
        SetEnumerationHandoffMutation::TermRange,
        SetEnumerationHandoffMutation::TermOrdinal,
        SetEnumerationHandoffMutation::TermContext,
        SetEnumerationHandoffMutation::TermRecovery,
        SetEnumerationHandoffMutation::TermSpelling,
        SetEnumerationHandoffMutation::TermKind,
        SetEnumerationHandoffMutation::ExtraWrapper,
        SetEnumerationHandoffMutation::ExtraGenerator,
        SetEnumerationHandoffMutation::ExtraTypeSite,
        SetEnumerationHandoffMutation::ExtraCondition,
        SetEnumerationHandoffMutation::RequestTerm,
        SetEnumerationHandoffMutation::RequestOrdinal,
        SetEnumerationHandoffMutation::RequestKind,
        SetEnumerationHandoffMutation::RequestGenerator,
        SetEnumerationHandoffMutation::RequestTypeSite,
        SetEnumerationHandoffMutation::CoherentApplicationFingerprint,
        SetEnumerationHandoffMutation::CoherentStructureFingerprint,
    ];
    for edge in 0..2 {
        handoff_mutations.extend([
            SetEnumerationHandoffMutation::EdgeTerm(edge),
            SetEnumerationHandoffMutation::EdgeOrdinal(edge),
            SetEnumerationHandoffMutation::EdgeRole(edge),
            SetEnumerationHandoffMutation::EdgeTarget(edge),
        ]);
    }
    for handoff in handoff_mutations {
        let error = run(SetEnumerationProofContextTestOptions {
            handoff,
            ..Default::default()
        })
        .expect("Task255 mutation retains lower selection")
        .expect_err("Task255 corruption");
        assert!(error.starts_with("Task255:"), "{handoff:?}: {error}");
        let fingerprint_kind = match handoff {
            SetEnumerationHandoffMutation::CoherentApplicationFingerprint => Some("application"),
            SetEnumerationHandoffMutation::CoherentStructureFingerprint => Some("structure"),
            _ => None,
        };
        if let Some(kind) = fingerprint_kind {
            assert_eq!(
                error,
                format!(
                    "Task255: exact B3P fingerprint-only profile rejected coherent non-None {kind} dependency fingerprint"
                ),
                "{handoff:?}"
            );
        }
        clean_replay();
    }
    let stale = run(SetEnumerationProofContextTestOptions {
        primary: SetEnumerationPrimaryMutation::StaleFingerprintReplay,
        ..Default::default()
    })
    .expect("stale selector")
    .expect_err("stale Task252 fingerprint");
    assert!(
        stale.starts_with("TypedAst: rejected stale Task252 fingerprint:"),
        "{stale}"
    );
    clean_replay();

    for (final_clone, prefix) in [
        (SetEnumerationFinalMutation::TypedClone, "TypedAst:"),
        (
            SetEnumerationFinalMutation::ResolvedClone,
            "ResolvedTypedAst:",
        ),
    ] {
        let error = run(SetEnumerationProofContextTestOptions {
            final_clone,
            ..Default::default()
        })
        .expect("final clone selector")
        .expect_err("final clone corruption");
        assert!(error.starts_with(prefix), "{error}");
        clean_replay();
    }

    assert_eq!(
        set_enumeration_selection_stage_for_test(
            &ast,
            &module,
            &shells,
            &symbols,
            "!",
            SetEnumerationSurfaceMutation::NodeKind(0),
            SetEnumerationResolverMutation::Field(0),
        ),
        SetEnumerationSelectionStage::Source
    );
    clean_replay();
    assert_eq!(
        set_enumeration_selection_stage_for_test(
            &ast,
            &module,
            &shells,
            &symbols,
            TASK258B3M2B2B3P_SET_SOURCE,
            SetEnumerationSurfaceMutation::NodeKind(0),
            SetEnumerationResolverMutation::Field(0),
        ),
        SetEnumerationSelectionStage::Surface
    );
    clean_replay();
    assert_eq!(
        set_enumeration_selection_stage_for_test(
            &ast,
            &module,
            &shells,
            &symbols,
            TASK258B3M2B2B3P_SET_SOURCE,
            SetEnumerationSurfaceMutation::None,
            SetEnumerationResolverMutation::Field(0),
        ),
        SetEnumerationSelectionStage::Resolver
    );
    clean_replay();
    assert!(
        set_enumeration_proof_context_handoff_for_test(
            &ast,
            &module,
            &shells,
            &symbols,
            &bindings,
            "!",
            SetEnumerationProofContextTestOptions {
                surface: SetEnumerationSurfaceMutation::NodeKind(0),
                resolver: SetEnumerationResolverMutation::Field(0),
                binding: SetEnumerationBindingMutation::Field(0),
                primary: SetEnumerationPrimaryMutation::TermRange(0),
                handoff: SetEnumerationHandoffMutation::TermRange,
                final_clone: SetEnumerationFinalMutation::TypedClone,
            },
        )
        .is_none(),
        "loaded source must win simultaneous corruption"
    );
    clean_replay();
    assert!(
        run(SetEnumerationProofContextTestOptions {
            surface: SetEnumerationSurfaceMutation::NodeKind(0),
            resolver: SetEnumerationResolverMutation::Field(0),
            binding: SetEnumerationBindingMutation::Field(0),
            primary: SetEnumerationPrimaryMutation::TermRange(0),
            handoff: SetEnumerationHandoffMutation::TermRange,
            final_clone: SetEnumerationFinalMutation::TypedClone,
        })
        .is_none(),
        "arena/root must win after source selection"
    );
    clean_replay();
    assert!(
        run(SetEnumerationProofContextTestOptions {
            resolver: SetEnumerationResolverMutation::Field(0),
            binding: SetEnumerationBindingMutation::Field(0),
            primary: SetEnumerationPrimaryMutation::TermRange(0),
            handoff: SetEnumerationHandoffMutation::TermRange,
            final_clone: SetEnumerationFinalMutation::TypedClone,
            ..Default::default()
        })
        .is_none(),
        "resolver must win after arena/root"
    );
    clean_replay();
    let task48 = run(SetEnumerationProofContextTestOptions {
        binding: SetEnumerationBindingMutation::Field(0),
        primary: SetEnumerationPrimaryMutation::TermRange(0),
        handoff: SetEnumerationHandoffMutation::TermRange,
        final_clone: SetEnumerationFinalMutation::TypedClone,
        ..Default::default()
    })
    .expect("Task48 simultaneous selector")
    .expect_err("Task48 simultaneous corruption");
    assert!(task48.starts_with("Task48:"), "{task48}");
    clean_replay();
    let task252 = run(SetEnumerationProofContextTestOptions {
        primary: SetEnumerationPrimaryMutation::TermRange(0),
        handoff: SetEnumerationHandoffMutation::TermRange,
        final_clone: SetEnumerationFinalMutation::TypedClone,
        ..Default::default()
    })
    .expect("Task252 simultaneous selector")
    .expect_err("Task252 simultaneous corruption");
    assert!(task252.starts_with("Task252:"), "{task252}");
    clean_replay();
    let task255 = run(SetEnumerationProofContextTestOptions {
        primary: SetEnumerationPrimaryMutation::StaleFingerprintReplay,
        handoff: SetEnumerationHandoffMutation::TermRange,
        final_clone: SetEnumerationFinalMutation::TypedClone,
        ..Default::default()
    })
    .expect("Task255 simultaneous selector")
    .expect_err("Task255 simultaneous corruption");
    assert!(task255.starts_with("Task255:"), "{task255}");
    clean_replay();
    let stale_before_final = run(SetEnumerationProofContextTestOptions {
        primary: SetEnumerationPrimaryMutation::StaleFingerprintReplay,
        final_clone: SetEnumerationFinalMutation::ResolvedClone,
        ..Default::default()
    })
    .expect("stale/final simultaneous selector")
    .expect_err("stale/final simultaneous corruption");
    assert!(
        stale_before_final.starts_with("TypedAst: rejected stale Task252 fingerprint:"),
        "{stale_before_final}"
    );
    clean_replay();

    let (legacy_ast, legacy_module, legacy_symbols) =
        task252_real_ast("fail_type_elaboration_set_enumeration_formula_gap_001");
    let legacy_first = source_atomic_formula_output(
        &legacy_ast,
        legacy_module.clone(),
        &legacy_symbols,
    )
    .expect("legacy Task111 selector")
    .expect("legacy Task111 output");
    let legacy_second =
        source_atomic_formula_output(&legacy_ast, legacy_module, &legacy_symbols)
            .expect("legacy Task111 replay selector")
            .expect("legacy Task111 replay");
    assert_eq!(
        sha256_text(
            &legacy_first
                .typed_ast
                .source_set_term()
                .expect("legacy Task255 handoff")
                .debug_text()
        ),
        "30b72230bb7ff39464962133b58df212e23afccccc8f4e4788ab9a9d0481c43a"
    );
    assert_eq!(
        sha256_text(&legacy_first.typed_ast.debug_text()),
        "1bb296c06ab62691684260aa94987adee23081baa4a35aac9e485d95370d2cb9"
    );
    assert_eq!(
        sha256_text(&legacy_first.resolved.debug_text()),
        "cdb4eaae9605f62269d6a74d64267a8fcb1e8d8008564d8b9e014037665df1e4"
    );
    assert_eq!(
        legacy_first.typed_ast.debug_text(),
        legacy_second.typed_ast.debug_text()
    );
    assert_eq!(
        legacy_first.resolved.debug_text(),
        legacy_second.resolved.debug_text()
    );
}
