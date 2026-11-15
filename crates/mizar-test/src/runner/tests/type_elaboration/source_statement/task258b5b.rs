#[test]
fn task258b5b_opt_in_imported_public_theorem_label_is_exact() {
    assert_eq!(SOURCE_STATEMENT_B5B_LOWER_TEXT.len(), 146);
    assert!(SOURCE_STATEMENT_B5B_LOWER_TEXT.ends_with('\n'));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B5B_LOWER_TEXT),
        "671e940c9dc749757dc8fddcc30a1a230aecb650058e64d6f1e73c1c66e93e9e"
    );
    let (ast, module, _, raw) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5B_LOWER_TEXT, 258_550);
    assert_eq!(task258b5b_lower_resolver_profile(&raw), [1, 0, 1, 1, 0]);
    let normal = augment_type_elaboration_import_summaries(&ast, &module, raw.clone());
    assert_eq!(task258b5b_lower_resolver_profile(&normal), [8, 0, 1, 3, 1]);
    let opt_in = super::import_fixtures::
        augment_type_elaboration_import_summaries_with_imported_public_theorem_label(
            &ast, &module, raw,
        );
    assert!(task258b5b_imported_label_contract_is_exact(
        &ast, &module, &opt_in
    ));
    assert_eq!(normal.imports(), opt_in.imports());
    assert_eq!(normal.exports(), opt_in.exports());
    assert_eq!(normal.symbols(), opt_in.symbols());
    assert_eq!(normal.definitions(), opt_in.definitions());
    assert_eq!(normal.overloads(), opt_in.overloads());
    assert_eq!(normal.registrations(), opt_in.registrations());
    assert_eq!(normal.lexical_summaries(), opt_in.lexical_summaries());
    assert_eq!(normal.namespace_graph(), opt_in.namespace_graph());
    assert_eq!(
        normal.declaration_dependencies(),
        opt_in.declaration_dependencies()
    );
    assert_eq!(normal.module_summaries(), opt_in.module_summaries());
    for (normal_record, opt_in_record) in normal
        .contributions()
        .iter()
        .zip(opt_in.contributions().iter())
    {
        assert_eq!(normal_record.id(), opt_in_record.id());
        assert_eq!(normal_record.module(), opt_in_record.module());
        assert_eq!(normal_record.kind(), opt_in_record.kind());
        assert_eq!(normal_record.anchor(), opt_in_record.anchor());
        assert_eq!(
            normal_record.effects().symbols(),
            opt_in_record.effects().symbols()
        );
        assert_eq!(
            normal_record.effects().definitions(),
            opt_in_record.effects().definitions()
        );
        assert_eq!(
            normal_record.effects().overload_groups(),
            opt_in_record.effects().overload_groups()
        );
        assert_eq!(
            normal_record.effects().registrations(),
            opt_in_record.effects().registrations()
        );
        assert_eq!(
            normal_record.effects().lexical_summaries(),
            opt_in_record.effects().lexical_summaries()
        );
        assert_eq!(
            normal_record.effects().namespace_edges(),
            opt_in_record.effects().namespace_edges()
        );
        assert_eq!(
            normal_record.effects().declaration_dependencies(),
            opt_in_record.effects().declaration_dependencies()
        );
        assert_eq!(
            normal_record.effects().imports(),
            opt_in_record.effects().imports()
        );
        assert_eq!(
            normal_record.effects().exports(),
            opt_in_record.effects().exports()
        );
        assert_eq!(
            normal_record.effects().diagnostics(),
            opt_in_record.effects().diagnostics()
        );
        if opt_in_record.id().index() != 2 {
            assert_eq!(
                normal_record.effects().labels(),
                opt_in_record.effects().labels()
            );
        } else {
            assert!(normal_record.effects().labels().is_empty());
            assert_eq!(
                opt_in_record.effects().labels()[0].as_str(),
                "summary:parser.type_fixtures::Ref:label:Ref"
            );
        }
    }
}

#[test]
fn task258b5b_opt_in_imported_label_corruption_and_default_routes_stay_isolated() {
    let (ast, module, _, raw) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5B_LOWER_TEXT, 258_551);
    let normal = augment_type_elaboration_import_summaries(&ast, &module, raw.clone());
    assert!(normal.labels().is_empty());
    assert_eq!(
        normal,
        augment_type_elaboration_import_summaries(&ast, &module, raw.clone())
    );
    let opt_in = super::import_fixtures::
        augment_type_elaboration_import_summaries_with_imported_public_theorem_label(
            &ast, &module, raw,
        );
    assert!(task258b5b_imported_label_contract_is_exact(
        &ast, &module, &opt_in
    ));
    let foreign_source = task258b3m2b2b3a_foreign_source_id(ast.source_id);
    for mutation in [
        Task258B5BImportedLabelMutation::Missing,
        Task258B5BImportedLabelMutation::Duplicate,
        Task258B5BImportedLabelMutation::Private,
        Task258B5BImportedLabelMutation::LocalOnly,
        Task258B5BImportedLabelMutation::ReExported,
        Task258B5BImportedLabelMutation::WrongKind,
        Task258B5BImportedLabelMutation::WrongSpelling,
        Task258B5BImportedLabelMutation::WrongModule,
        Task258B5BImportedLabelMutation::WrongNamespace,
        Task258B5BImportedLabelMutation::WrongContribution,
        Task258B5BImportedLabelMutation::Recovered,
        Task258B5BImportedLabelMutation::WrongOriginPath,
        Task258B5BImportedLabelMutation::WrongSource,
        Task258B5BImportedLabelMutation::WrongRange,
        Task258B5BImportedLabelMutation::WrongStructuralPath,
        Task258B5BImportedLabelMutation::MissingEffect,
        Task258B5BImportedLabelMutation::ExtraEffect,
        Task258B5BImportedLabelMutation::WrongEffect,
    ] {
        let corrupted =
            task258b5b_mutate_imported_label(&opt_in, &module, foreign_source, mutation);
        assert!(
            !task258b5b_imported_label_contract_is_exact(&ast, &module, &corrupted),
            "corruption remained exact: {mutation:?}"
        );
    }
    let (b5a_ast, b5a_module, _, b5a_raw) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_553);
    let b5a_default =
        augment_type_elaboration_import_summaries(&b5a_ast, &b5a_module, b5a_raw.clone());
    assert_eq!(
        super::import_fixtures::
            augment_type_elaboration_import_summaries_with_imported_public_theorem_label(
                &b5a_ast,
                &b5a_module,
                b5a_raw,
            ),
        b5a_default
    );
}

#[test]
fn task258b5b_real_route_publishes_exact_imported_public_theorem_citation() {
    assert_eq!(SOURCE_STATEMENT_B5B_TEXT, SOURCE_STATEMENT_B5B_LOWER_TEXT);
    assert_eq!(SOURCE_STATEMENT_B5B_TEXT.len(), 146);
    assert!(SOURCE_STATEMENT_B5B_TEXT.ends_with('\n'));
    assert!(!SOURCE_STATEMENT_B5B_TEXT.ends_with("\n\n"));
    assert_eq!(
        sha256_text(SOURCE_STATEMENT_B5B_TEXT),
        "671e940c9dc749757dc8fddcc30a1a230aecb650058e64d6f1e73c1c66e93e9e"
    );
    let (ast, module, raw, enriched, inputs) = task258b5b_exact_env(258_560);
    assert!(task258b5b_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B5B_TEXT,
        SourceStatementB5BSurfaceMutation::None,
    ));
    for index in 0..57 {
        for mutation in [
            SourceStatementB5BSurfaceMutation::NodeKind(index),
            SourceStatementB5BSurfaceMutation::NodeRange(index),
            SourceStatementB5BSurfaceMutation::NodeRecovery(index),
            SourceStatementB5BSurfaceMutation::NodeChildren(index),
        ] {
            assert!(
                !task258b5b_surface_profile_with_mutation_for_test(
                    &ast,
                    SOURCE_STATEMENT_B5B_TEXT,
                    mutation,
                ),
                "{mutation:?}"
            );
        }
    }
    assert!(!task258b5b_surface_profile_with_mutation_for_test(
        &ast,
        SOURCE_STATEMENT_B5B_TEXT,
        SourceStatementB5BSurfaceMutation::RootIdentity,
    ));
    let extracted: SourceStatementB5BExtraction =
        extract_imported_public_theorem_citation_source_statement(&ast, SOURCE_STATEMENT_B5B_TEXT)
            .expect("exact B5B surface");
    assert_eq!(
        (ast.nodes().len(), ast.root().expect("root").index()),
        (57, 56)
    );
    assert_eq!(
        (
            extracted.theorem_site.node().index(),
            extracted.theorem_range.start,
            extracted.theorem_range.end,
            extracted.label_range.start,
            extracted.label_range.end,
            extracted.proof_range.start,
            extracted.proof_range.end,
        ),
        (53, 48, 145, 56, 106, 114, 144)
    );
    assert_eq!(
        extracted
            .statement_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [53, 51]
    );
    assert_eq!(
        extracted
            .statement_ranges
            .map(|source_range| (source_range.start, source_range.end)),
        [(48, 145), (122, 140)]
    );
    assert_eq!(
        extracted
            .formula_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [40, 46]
    );
    assert_eq!(
        extracted
            .term_sites
            .iter()
            .map(|site| site.node().index())
            .collect::<Vec<_>>(),
        [35, 37, 41, 43]
    );
    assert_eq!(task258b5b_lower_resolver_profile(&raw), [1, 0, 1, 1, 0]);
    let normal = augment_type_elaboration_import_summaries(&ast, &module, raw.clone());
    assert_eq!(task258b5b_lower_resolver_profile(&normal), [8, 0, 1, 3, 1]);
    assert!(normal.module_summaries().iter().next().is_none());
    assert!(task258b5b_imported_label_contract_is_exact(
        &ast, &module, &enriched
    ));
    assert_eq!(
        source_statement_b5b_resolver_env_for_test(&ast, &module, &raw, SOURCE_STATEMENT_B5B_TEXT,)
            .expect("B5B lower-stage opt-in"),
        enriched
    );

    let binding_env = &inputs.binding_env;
    assert_eq!((binding_env.contexts().len(), binding_env.bindings().len(), binding_env.diagnostics().len()), (2, 1, 0));
    let module_context = binding_env
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(0))
        .expect("module context");
    let proof_context = binding_env
        .contexts()
        .get(mizar_checker::binding_env::BindingContextId::new(1))
        .expect("proof context");
    assert_eq!(
        (
            &module_context.owner,
            module_context.parent,
            module_context.layer,
            module_context.lexical_scope.as_ref(),
            module_context.bindings.as_slice(),
            module_context.visible_bindings.as_slice(),
            module_context.recovery,
        ),
        (
            &mizar_checker::binding_env::BindingContextOwner::Module,
            None,
            mizar_checker::binding_env::BindingContextLayer::Module,
            None,
            &[mizar_checker::binding_env::BindingId::new(0)][..],
            &[mizar_checker::binding_env::BindingId::new(0)][..],
            mizar_checker::binding_env::BindingContextRecovery::Normal,
        )
    );
    assert!(matches!(
        proof_context.owner,
        mizar_checker::binding_env::BindingContextOwner::SourceStatement { source_range }
            if (source_range.start, source_range.end) == (114, 144)
    ));
    assert_eq!(
        (
            proof_context.parent.map(|id| id.index()),
            proof_context.layer,
            proof_context
                .lexical_scope
                .as_ref()
                .map(|scope| scope.path()),
            proof_context.bindings.as_slice(),
            proof_context.visible_bindings.as_slice(),
            proof_context.recovery,
        ),
        (
            Some(0),
            mizar_checker::binding_env::BindingContextLayer::Proof,
            Some(&[0][..]),
            &[][..],
            &[mizar_checker::binding_env::BindingId::new(0)][..],
            mizar_checker::binding_env::BindingContextRecovery::Normal,
        )
    );
    let primary = &inputs.primary;
    assert_eq!(
        primary
            .terms()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.context().index(),
                row.source_ordinal(),
                row.spelling(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 35, 108, 109, 0, 0, "x"),
            (1, 37, 112, 113, 0, 1, "x"),
            (2, 41, 127, 128, 1, 2, "x"),
            (3, 43, 131, 132, 1, 3, "x"),
        ]
    );
    assert_eq!(
        primary
            .references()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.term().index(),
                row.binding().index(),
                row.lexical_scope().map(|scope| scope.path().to_vec()),
                row.use_ordinal(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 0, 0, None, 1),
            (1, 1, 0, None, 1),
            (2, 2, 0, Some(vec![0]), 1),
            (3, 3, 0, Some(vec![0]), 1),
        ]
    );
    let atomic = &inputs.atomic;
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
        (2, 0, 0, 0, 0, 0, 0, 4, 4)
    );
    assert_eq!(
        atomic
            .formulas()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.site().node().index(),
                row.source_range().start,
                row.source_range().end,
                row.context().index(),
                row.source_ordinal(),
                row.spelling(),
            ))
            .collect::<Vec<_>>(),
        [
            (0, 40, 108, 113, 0, 0, "x = x"),
            (1, 46, 127, 132, 1, 1, "x = x"),
        ]
    );
    for index in 0..4 {
        let edge = atomic
            .edges()
            .get(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(index))
            .expect("edge");
        let request = atomic
            .requests()
            .get(mizar_checker::source_atomic_formula::SourceAtomicRequestId::new(index))
            .expect("request");
        assert_eq!(
            (edge.formula().index(), edge.ordinal()),
            (index / 2, index % 2)
        );
        assert_eq!(
            edge.target(),
            mizar_checker::source_atomic_formula::SourceAtomicTermTarget::Primary(
                mizar_checker::source_term::SourcePrimaryTermId::new(index),
            )
        );
        assert_eq!(
            (request.formula().index(), request.ordinal(), request.edge()),
            (
                index / 2,
                index % 2,
                Some(mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(index)),
            )
        );
    }
    let owned = inputs.arena
        .iter()
        .filter(|(_, row)| row.kind.as_str() != "source.surface.unowned")
        .map(|(id, _)| id.index())
        .collect::<Vec<_>>();
    assert_eq!(owned, [35, 37, 40, 41, 43, 46, 51, 53]);
    assert_eq!(
        inputs.arena
            .iter()
            .filter(|(_, row)| row.kind.as_str() == "source.surface.unowned")
            .count(),
        49
    );

}

#[test]
fn task258b5b_lower_import_resolver_and_row_mutations_fail_at_owning_stage() {
    let (_, _, _, enriched, baseline) = task258b5b_exact_env(258_570);
    let baseline_debug = format!("{baseline:?}");

    let baseline_primary = &baseline.primary;
    let baseline_binding = &baseline.binding_env;
    for index in 0..4 {
        let mut input = task258b3m2a_primary_input(baseline_primary);
        input.terms[index].source_ordinal = 4;
        assert_ne!(format!("{input:?}"), format!("{:?}", task258b3m2a_primary_input(baseline_primary)));
        let result = mizar_checker::source_term::SourcePrimaryTermProducer::build(
                input.clone(),
                baseline_binding,
                &baseline.arena,
            );
        assert_eq!(result, Err(mizar_checker::source_term::SourcePrimaryTermError::InvalidTerm { term: mizar_checker::source_term::SourcePrimaryTermId::new(index) }), "Task252 term {index}");
        assert_eq!(mizar_checker::source_term::SourcePrimaryTermProducer::build(
                input,
                baseline_binding,
                &baseline.arena,
            ), result, "Task252 term {index} replay");
        let mut input = task258b3m2a_primary_input(baseline_primary);
        input.references[index].term =
            mizar_checker::source_term::SourcePrimaryTermId::new((index + 1) % 4);
        assert_ne!(format!("{input:?}"), format!("{:?}", task258b3m2a_primary_input(baseline_primary)));
        let result = mizar_checker::source_term::SourcePrimaryTermProducer::build(
                input.clone(),
                baseline_binding,
                &baseline.arena,
            );
        assert_eq!(result, Err(mizar_checker::source_term::SourcePrimaryTermError::InvalidReference { reference: mizar_checker::source_term::SourcePrimaryTermReferenceId::new((index + 1).min(3)) }), "Task252 reference {index}");
        assert_eq!(mizar_checker::source_term::SourcePrimaryTermProducer::build(
                input,
                baseline_binding,
                &baseline.arena,
            ), result, "Task252 reference {index} replay");
    }
    let baseline_atomic = &baseline.atomic;
    for index in 0..2 {
        let mut input = task258b3m2b1_atomic_input(baseline_atomic);
        input.formulas[index].source_ordinal = 2;
        assert_ne!(format!("{input:?}"), format!("{:?}", task258b3m2b1_atomic_input(baseline_atomic)));
        let result = mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                input.clone(),
                baseline_binding,
                &enriched,
                baseline_primary,
                None,
                None,
                None,
                &baseline.arena,
            );
        assert_eq!(result, Err(mizar_checker::source_atomic_formula::SourceAtomicFormulaError::InvalidFormula { formula: mizar_checker::source_atomic_formula::SourceAtomicFormulaId::new(index) }), "Task256 formula {index}");
        assert_eq!(mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                input,
                baseline_binding,
                &enriched,
                baseline_primary,
                None,
                None,
                None,
                &baseline.arena,
            ), result, "Task256 formula {index} replay");
    }
    for index in 0..4 {
        let mut input = task258b3m2b1_atomic_input(baseline_atomic);
        input.edges[index].ordinal = 2;
        assert_ne!(format!("{input:?}"), format!("{:?}", task258b3m2b1_atomic_input(baseline_atomic)));
        let result = mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                input.clone(),
                baseline_binding,
                &enriched,
                baseline_primary,
                None,
                None,
                None,
                &baseline.arena,
            );
        assert_eq!(result, Err(mizar_checker::source_atomic_formula::SourceAtomicFormulaError::ReorderedEdge { edge: mizar_checker::source_atomic_formula::SourceAtomicEdgeId::new(index) }), "Task256 edge {index}");
        assert_eq!(mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                input,
                baseline_binding,
                &enriched,
                baseline_primary,
                None,
                None,
                None,
                &baseline.arena,
            ), result, "Task256 edge {index} replay");
        let mut input = task258b3m2b1_atomic_input(baseline_atomic);
        input.requests[index].ordinal = 2;
        assert_ne!(format!("{input:?}"), format!("{:?}", task258b3m2b1_atomic_input(baseline_atomic)));
        let result = mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                input.clone(),
                baseline_binding,
                &enriched,
                baseline_primary,
                None,
                None,
                None,
                &baseline.arena,
            );
        assert_eq!(result, Err(mizar_checker::source_atomic_formula::SourceAtomicFormulaError::ReorderedRequest { request: mizar_checker::source_atomic_formula::SourceAtomicRequestId::new(index) }), "Task256 request {index}");
        assert_eq!(mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(
                input,
                baseline_binding,
                &enriched,
                baseline_primary,
                None,
                None,
                None,
                &baseline.arena,
            ), result, "Task256 request {index} replay");
    }

    assert_eq!(mizar_checker::source_term::SourcePrimaryTermProducer::build(task258b3m2a_primary_input(baseline_primary), baseline_binding, &baseline.arena).expect("pristine primary replay"), *baseline_primary);
    assert_eq!(mizar_checker::source_atomic_formula::SourceAtomicFormulaProducer::build(task258b3m2b1_atomic_input(baseline_atomic), baseline_binding, &enriched, baseline_primary, None, None, None, &baseline.arena).expect("pristine atomic replay"), *baseline_atomic);

    let (_, _, _, _, replay) = task258b5b_exact_env(258_570);
    assert_eq!(format!("{baseline:?}"), baseline_debug);
    assert_eq!(format!("{replay:?}"), baseline_debug);
}

#[test]
fn task258b5b_source_fixture_and_import_visibility_near_misses_stay_isolated() {
    let (exact_ast, exact_module, raw, _, _) = task258b5b_exact_env(258_580);
    let normal = augment_type_elaboration_import_summaries(&exact_ast, &exact_module, raw.clone());

    for symbols in [&raw, &normal] {
        assert_eq!(source_statement_b5b_lower_with_mutation(&exact_ast, exact_module.clone(), symbols, SOURCE_STATEMENT_B5B_TEXT, |_| {}).expect("strict B5B opt-in selector").expect_err("raw/default resolver must fail preflight"), "Task258B5B opt-in resolver profile mismatch");
    }

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("mizar-test crate below workspace")
        .to_path_buf();
    for (ordinal, (relative_path, expected_hash)) in [
        (
            "tests/miz/pass/parser/pass_parser_justifications_001.miz",
            "1f6d8a1a461aa682fffa987adb726bec6b66941bac407728d625f1b9e387d80d",
        ),
        (
            "tests/miz/pass/parser/pass_parser_redefinition_notation_001.miz",
            "ce77250e61a9c483d8d055a7ea195399dd4f602c8dfeb39089acc1fed7d3012e",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let fixture_path = workspace_root.join(relative_path);
        let fixture = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", fixture_path.display()));
        assert_eq!(sha256_text(&fixture), expected_hash, "{relative_path}");
        let (fixture_ast, _, _, _, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&fixture, 258_581 + ordinal);
        assert_eq!(diagnostics, 0, "{relative_path}");
        assert!(
            extract_imported_public_theorem_citation_source_statement(&fixture_ast, &fixture)
                .is_none(),
            "{relative_path}"
        );

    }
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task258B5B isolation plan");
    assert!(plan.cases.iter().any(|case| {
        case.id.0 == "pass_parser_justifications_001"
            && case
                .source_path
                .ends_with("pass_parser_justifications_001.miz")
    }));
    assert!(plan.cases.iter().any(|case| {
        case.id.0 == "pass_parser_redefinition_notation_001"
            && case
                .source_path
                .ends_with("pass_parser_redefinition_notation_001.miz")
    }));

}

#[test]
fn task258b5b_b5a_profile_installation_orders_and_ownership_are_atomic() {
    let (b1_ast, b1_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B1_TEXT, 258_590);
    let (b1, _) = task258_native_output(&b1_ast, &b1_module);
    let (b5a_ast, b5a_module, _, _) =
        task253_ast_from_source_text(SOURCE_STATEMENT_B5A_TEXT, 258_590);
    let (b5a, _) = task258_native_output(&b5a_ast, &b5a_module);
    let (b5b_ast, b5b_module, _, _, _) = task258b5b_exact_env(258_590);
    assert_eq!(b1.source_id(), b5b_ast.source_id);
    assert_eq!(b1.module_id(), &b5b_module);
    assert_eq!(b5a.source_id(), b5b_ast.source_id);
    assert_eq!(b5a.module_id(), &b5b_module);

    let b1_statement = b1
        .source_statement()
        .expect("B1 statement")
        .clone();
    let b1_references = b1
        .source_statement_references()
        .expect("B1 references")
        .clone();
    let b5a_statement = b5a
        .source_statement()
        .expect("B5A statement")
        .clone();
    let b5a_references = b5a
        .source_statement_references()
        .expect("B5A references")
        .clone();

    // Accepted B5B pair, cross-profile and duplicate directions live in Checker.
    let task248 = task248_real_output();
    let before = task248.typed_ast.debug_text();

    assert_eq!(task248.typed_ast.debug_text(), before);
    assert_eq!(b1.source_statement(), Some(&b1_statement));
    assert_eq!(
        b1.source_statement_references(),
        Some(&b1_references)
    );
    assert_eq!(b5a.source_statement(), Some(&b5a_statement));
    assert_eq!(
        b5a.source_statement_references(),
        Some(&b5a_references)
    );
}

#[test]
fn task258b5b_final_clone_debug_replay_and_empty_semantics_are_stable() {
    let (_, _, _, _, first) = task258b5b_exact_env(258_600);
    let pristine = format!("{first:?}");
    assert_eq!(format!("{:?}", first.clone()), pristine);
    let (_, _, _, _, second) = task258b5b_exact_env(258_600);
    assert_eq!(format!("{second:?}"), pristine);

    let (_, _, _, _, replay) = task258b5b_exact_env(258_600);
    assert_eq!(format!("{first:?}"), pristine);
    assert_eq!(format!("{replay:?}"), pristine);
}
