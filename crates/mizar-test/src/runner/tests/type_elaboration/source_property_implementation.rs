use super::type_elaboration::{
    SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT, SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
    SourcePropertyImplementationRouteMutation, SourcePropertyImplementationRouteOutput,
    source_property_implementation_output, source_property_implementation_output_with_mutation,
    step5c4_property_implementation_detail_keys,
};

const TASK264_MEANS_CASE: &str =
    "pass_type_elaboration_property_implementation_means_payload_001";
const TASK264_EQUALS_CASE: &str =
    "pass_type_elaboration_property_implementation_equals_payload_001";
const TASK264_SPEC_REF: &str =
    "spec.en.checker.type_elaboration.source_property_implementation_payload";

#[test]
fn step5c4_property_bridge_rejects_authenticated_semantic_drift() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/pass/modes/",
        "pass_type_elaboration_mode_property_impl_equals_001.miz"
    ));
    let (ast, module, shells, symbols) = task253_ast_from_source_text(source, 504_010);
    assert_eq!(
        step5c4_property_implementation_detail_keys(&ast, module.clone(), &shells, &symbols),
        Some(Vec::new())
    );
    let mutated_source = source.replacen("B.data", "B.unknown", 1);
    let (mutated_ast, mutated_module, mutated_shells, mutated_symbols) =
        task253_ast_from_source_text(&mutated_source, 504_011);
    assert_eq!(
        step5c4_property_implementation_detail_keys(
            &mutated_ast,
            mutated_module.clone(),
            &mutated_shells,
            &mutated_symbols
        ),
        Some(vec!["modes.property_implementation.unknown_property".to_owned()])
    );
    assert_ne!(
        step5c4_property_implementation_detail_keys(
            &ast,
            module.clone(),
            &mutated_shells,
            &mutated_symbols
        ),
        Some(Vec::new())
    );
    let changed_kind = rebuild_surface_ast_replacing_kind(
        &ast,
        SurfaceNodeKind::StructureProperty,
        SurfaceNodeKind::StructureField,
    );
    assert_eq!(changed_kind.token_texts(), ast.token_texts());
    assert_ne!(
        step5c4_property_implementation_detail_keys(
            &changed_kind,
            module,
            &shells,
            &symbols
        ),
        Some(Vec::new())
    );

    let means_source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/miz/pass/modes/",
        "pass_type_elaboration_mode_property_impl_means_001.miz"
    ));
    let (means_ast, means_module, means_shells, means_symbols) =
        task253_ast_from_source_text(means_source, 504_012);
    assert_eq!(
        step5c4_property_implementation_detail_keys(
            &means_ast,
            means_module,
            &means_shells,
            &means_symbols
        ),
        Some(Vec::new())
    );
    let reordered = means_source
        .replacen("existence;\n  uniqueness;", "uniqueness;\n  existence;", 1);
    let (reordered_ast, reordered_module, reordered_shells, reordered_symbols) =
        task253_ast_from_source_text(&reordered, 504_013);
    assert_ne!(
        step5c4_property_implementation_detail_keys(
            &reordered_ast,
            reordered_module,
            &reordered_shells,
            &reordered_symbols
        ),
        Some(Vec::new())
    );
}

#[test]
fn task264_exact_sources_surface_resolver_lower_and_outputs_are_stable() {
    let profiles = [
        (
            SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
            263,
            "cc90659f10cae4ef68890624df9b8b9d3f0e830dae5e20cc195dc8b263c5fa2b",
            85,
            84,
            2,
        ),
        (
            SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
            189,
            "175135aaf40b9eab1a28e73ca1aae9f250e66278410d50575cdd279f6d7a2784",
            56,
            55,
            0,
        ),
    ];
    for (ordinal, (source, bytes, hash, nodes, root, obligations)) in
        profiles.into_iter().enumerate()
    {
        assert_eq!(source.len(), bytes);
        assert_eq!(sha256_text(source), hash);
        assert!(source.ends_with('\n'));
        assert!(!source.ends_with("\n\n"));
        let (ast, module, shells, symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(source, 264_000 + ordinal);
        assert_eq!(diagnostics, 0);
        assert_eq!((ast.nodes().len(), ast.root().map(|id| id.index())), (nodes, Some(root)));
        assert!(ast.expression_root().is_none());
        assert_eq!((shells.declarations().len(), symbols.symbols().len(), symbols.definitions().len(), symbols.contributions().len()), (5, 3, 3, 1));
        let output = source_property_implementation_output(
            &ast, module, &shells, &symbols, source,
        )
        .expect("Task264 selector")
        .unwrap_or_else(|error| panic!("Task264 profile {ordinal} route: {error}"));
        let handoff = output
            .typed_ast
            .source_property_implementation()
            .expect("Task264 typed handoff");
        let definitions = (0..3)
            .map(|index| {
                symbols
                    .definitions()
                    .iter()
                    .find(|row| row.id().index() == index)
                    .expect("Task264 definition identity")
            })
            .collect::<Vec<_>>();
        let carrier = handoff.carrier_identity();
        assert_eq!(carrier.structure_symbol(), definitions[0].symbol());
        assert_eq!(carrier.structure_definition(), definitions[0].id());
        assert_eq!(carrier.structure_contribution(), definitions[0].contribution());
        assert_eq!(carrier.structure_origin(), definitions[0].origin());
        assert_eq!(carrier.field_symbol(), definitions[1].symbol());
        assert_eq!(carrier.field_definition(), definitions[1].id());
        assert_eq!(carrier.field_contribution(), definitions[1].contribution());
        assert_eq!(carrier.field_origin(), definitions[1].origin());
        assert_eq!(carrier.property_symbol(), definitions[2].symbol());
        assert_eq!(carrier.property_definition(), definitions[2].id());
        assert_eq!(carrier.property_contribution(), definitions[2].contribution());
        assert_eq!(carrier.property_origin(), definitions[2].origin());
        assert_eq!((handoff.implementations().len(), handoff.parameters().len(), handoff.targets().len(), handoff.definientia().len(), handoff.correctness().len()), (1, 1, 1, 1, obligations));
        assert_eq!(output.typed_ast.initial_obligations().len(), obligations);
        assert_eq!(output.typed_ast.source_property_implementation(), output.resolved.source_property_implementation());
        assert_eq!(output.typed_ast.debug_text().matches("source-property-implementation-debug-v2").count(), 1);
        assert_eq!(output.resolved.debug_text().matches("source-property-implementation-debug-v2").count(), 1);
    }
}

#[test]
fn task264_byte_ast_resolver_lower_and_it_mutations_fail_at_the_owner() {
    for (ordinal, source) in [SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT, SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT].into_iter().enumerate() {
        let (ast, module, shells, symbols) = task253_ast_from_source_text(source, 264_100 + ordinal);
        assert!(source_property_implementation_output(&ast, module.clone(), &shells, &symbols, &source[..source.len()-1]).is_some_and(|result| result.is_ok()));
        let mut extended = source.to_owned();
        extended.push(' ');
        assert!(source_property_implementation_output(&ast, module.clone(), &shells, &symbols, &extended).is_some_and(|result| result.is_ok()));
        for mutation in [
            SourcePropertyImplementationRouteMutation::WrongSurfaceRange,
            SourcePropertyImplementationRouteMutation::WrongSurfaceRecovery,
            SourcePropertyImplementationRouteMutation::WrongSurfaceChildren,
            SourcePropertyImplementationRouteMutation::WrongSurfaceRoot,
            SourcePropertyImplementationRouteMutation::WrongShell,
            SourcePropertyImplementationRouteMutation::WrongResolverTarget,
            SourcePropertyImplementationRouteMutation::WrongCarrierProvenance,
            SourcePropertyImplementationRouteMutation::WrongContext,
            SourcePropertyImplementationRouteMutation::WrongType,
            SourcePropertyImplementationRouteMutation::WrongTerm,
            SourcePropertyImplementationRouteMutation::WrongImplementation,
            SourcePropertyImplementationRouteMutation::WrongArena,
        ] {
            assert!(source_property_implementation_output_with_mutation(&ast, module.clone(), &shells, &symbols, source, mutation).expect("Task264 selector").is_err(), "mutation {mutation:?}");
        }
        let profile_mutation = if ordinal == 0 { SourcePropertyImplementationRouteMutation::WrongFormula } else { SourcePropertyImplementationRouteMutation::WrongStructure };
        assert!(source_property_implementation_output_with_mutation(&ast, module, &shells, &symbols, source, profile_mutation).expect("Task264 selector").is_err());
    }
}

#[test]
fn task264_two_case_trace_selection_and_mixed_boundaries_are_exact() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).expect("workspace root").to_path_buf();
    let plan = build_test_plan(&DiscoveryConfig { workspace_root:workspace_root.clone(), tests_root:workspace_root.join("tests"), manifest_path:workspace_root.join("tests/coverage/spec_trace.toml"), profile:TestProfile::Fast, validation_mode:ValidationMode::Metadata }).expect("Task264 plan");
    let selected = active_type_elaboration_cases(&plan).filter(|case| matches!(case.id.0.as_str(), TASK264_MEANS_CASE | TASK264_EQUALS_CASE)).collect::<Vec<_>>();
    assert_eq!(selected.len(), 2);
    assert!(selected.iter().all(|case| case.expectation.spec_refs.iter().map(|row|row.0.as_str()).eq([TASK264_SPEC_REF])));
    let requirement = plan.manifest.requirements.iter().find(|row|row.id.0==TASK264_SPEC_REF).expect("Task264 trace row");
    assert_eq!(requirement.status, crate::traceability::RequirementStatus::Covered);
    assert_eq!(requirement.coverage, crate::traceability::CoverageShape::Pass);
    assert_eq!(
        requirement
            .tests
            .iter()
            .map(|path| path.to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        vec![
            "tests/miz/pass/types/pass_type_elaboration_property_implementation_equals_payload_001.expect.toml",
            "tests/miz/pass/types/pass_type_elaboration_property_implementation_means_payload_001.expect.toml",
        ]
    );
    for case in selected {
        let source=std::fs::read_to_string(&case.source_path).expect("Task264 source");
        assert!(source==SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT||source==SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT);
    }
    let mixed = plan.cases.iter().find(|case|case.id.0=="fail_type_elaboration_predicate_functor_definition_gap_001").expect("mixed boundary");
    let mixed_source=std::fs::read_to_string(&mixed.source_path).expect("mixed source");
    let (ast,module,shells,symbols)=task253_ast_from_source_text(&mixed_source,264_200);
    assert!(source_property_implementation_output(&ast,module,&shells,&symbols,&mixed_source).is_none());
    assert_eq!((plan.cases.len(), plan.manifest.requirements.len()), (635, 545));
    assert_eq!(
        (plan.coverage_report.pass_fail_mix.pass, plan.coverage_report.pass_fail_mix.fail),
        (361, 274)
    );
    assert_eq!(
        (
            crate::active_parse_only_cases(&plan).count(),
            crate::active_declaration_symbol_cases(&plan).count(),
            active_type_elaboration_cases(&plan).count(),
            crate::active_proof_verification_cases(&plan).count(),
        ),
        (172, 13, 273, 23)
    );
    let type_stage = plan
        .coverage_report
        .stages
        .iter()
        .find(|row| row.stage == crate::staged_model::Stage::TypeElaboration)
        .expect("Task264 type coverage stage");
    assert_eq!((type_stage.requirements, type_stage.covered), (307, 295));
    assert_eq!((plan.warning_count(), plan.error_count()), (23, 0));

    let inactive = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_mode_property_overlap_missing_coherence_001")
        .expect("inactive coherence seed");
    assert_eq!(
        inactive.expectation.stage,
        crate::staged_model::Stage::AdvancedSemantics
    );
    assert!(!active_type_elaboration_cases(&plan).any(|case| case.id == inactive.id));
}

#[test]
fn task264_preserves_proof_subtrees_without_semantic_publication() {
    for (ordinal, source) in [SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT, SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT].into_iter().enumerate() {
        let (ast,module,shells,symbols)=task253_ast_from_source_text(source,264_300+ordinal);
        let output=source_property_implementation_output(&ast,module,&shells,&symbols,source).expect("Task264 selector").unwrap_or_else(|error| panic!("Task264 profile {ordinal} output: {error}"));
        let SourcePropertyImplementationRouteOutput { typed_ast, resolved } = output;
        assert!(typed_ast.types().is_empty());
        assert!(typed_ast.facts().is_empty());
        assert!(typed_ast.coercions().is_empty());
        assert!(typed_ast.diagnostics().is_empty());
        assert!(typed_ast.source_statement().is_none());
        assert!(typed_ast.source_statement_references().is_none());
        assert!(typed_ast.source_statement_witnesses().is_none());
        assert!(resolved.source_statement().is_none());
        assert!(resolved.source_statement_references().is_none());
        assert!(resolved.source_statement_witnesses().is_none());
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.checked_proof_nodes().is_empty());
        assert!(resolved.checked_terminal_goals().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.diagnostics().is_empty());
        let obligations = typed_ast.initial_obligations();
        if ordinal == 0 {
            let expected = [
                (
                    mizar_checker::typed_ast::InitialObligationKind::PropertyImplementationExistence,
                    76,
                    183,
                    218,
                    "source.definition.property-implementation.correctness:implementation=0:existence",
                    "source.definition.property-implementation:implementation=0:correctness=0",
                ),
                (
                    mizar_checker::typed_ast::InitialObligationKind::PropertyImplementationUniqueness,
                    80,
                    221,
                    257,
                    "source.definition.property-implementation.correctness:implementation=0:uniqueness",
                    "source.definition.property-implementation:implementation=0:correctness=1",
                ),
            ];
            assert_eq!(obligations.len(), expected.len());
            for (index, (kind, node, start, end, goal, provenance)) in
                expected.into_iter().enumerate()
            {
                let row = obligations
                    .get(mizar_checker::typed_ast::InitialObligationId::new(index))
                    .expect("Task264 exact obligation");
                assert_eq!(row.kind, kind);
                assert_eq!(
                    row.owner,
                    mizar_checker::typed_ast::TypedSiteRef::Node(
                        mizar_checker::typed_ast::TypedNodeId::new(node)
                    )
                );
                assert_eq!((row.source_range.start, row.source_range.end), (start, end));
                assert!(row.assumptions.is_empty());
                assert_eq!(row.goal.as_str(), goal);
                assert_eq!(row.provenance.as_str(), provenance);
                assert_eq!(
                    row.status,
                    mizar_checker::typed_ast::InitialObligationStatus::Pending
                );
            }
        } else {
            assert!(obligations.is_empty());
        }
        let debug=typed_ast.debug_text();
        assert!(!debug.contains("accepted"));
        assert!(!debug.contains("discharged"));
        assert!(!debug.contains("coherence"));
        if ordinal==0 {
            assert!(debug.contains("existence by computation(steps: 1);"));
            assert!(debug.contains("uniqueness by computation(steps: 1);"));
        }
    }
}

#[test]
fn task264_carrier_core_item_context_is_exact_for_means_and_equals() {
    for (ordinal, source) in [
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(source, 264_400 + ordinal);
        let output = source_property_implementation_output(
            &ast, module, &shells, &symbols, source,
        )
        .expect("Task264 selector")
        .unwrap_or_else(|error| panic!("Task264 Core profile {ordinal}: {error}"));
        let checker_owner = output
            .typed_ast
            .source_property_implementation()
            .expect("Task264 checker owner")
            .clone();
        let context = task264_carrier_core_context(&checker_owner);
        let expected_context = context.clone();
        let first = mizar_core::elaborator::SourcePropertyCarrierCoreContextProducer::build(
            context.clone(),
            checker_owner.clone(),
        )
        .expect("Task264 carrier Core context");
        let second = mizar_core::elaborator::SourcePropertyCarrierCoreContextProducer::build(
            context,
            checker_owner.clone(),
        )
        .expect("Task264 carrier deterministic replay");
        assert_eq!(first, second);
        assert_eq!(first.source_id(), checker_owner.source_id());
        assert_eq!(first.module_id(), checker_owner.module_id());
        assert_eq!(first.context(), &expected_context);
        assert_eq!(first.checker_owner(), &checker_owner);

        let identity = checker_owner.carrier_identity();
        let carrier_item = first.carrier_item();
        assert_eq!(
            first
                .context()
                .item_registry()
                .id_for_symbol(identity.structure_symbol()),
            Some(carrier_item)
        );
        assert_eq!(first.context().item_registry().items().len(), 1);
        assert!(first.context().dependency_summaries().is_empty());
        assert!(first.context().generated_origins().table().is_empty());
        assert!(first.context().diagnostics().is_empty());
        assert_eq!(
            first.debug_text(),
            format!(
                "source-property-carrier-core-item-context-v1|module={}.{}|carrier={}:0:0|item=0",
                checker_owner.module_id().package().as_str(),
                checker_owner.module_id().path().as_str(),
                identity.structure_symbol().fqn().as_str(),
            )
        );

        let item = first
            .context()
            .item_registry()
            .items()
            .get(carrier_item)
            .expect("Task264 carrier Core item");
        assert_eq!(item.symbol, *identity.structure_symbol());
        assert_eq!(item.kind, mizar_core::core_ir::CoreItemKind::Structure);
        assert_eq!(item.visibility.as_str(), "public");
        assert_eq!(item.status, mizar_core::core_ir::CoreItemStatus::Valid);
        assert!(item.dependencies.is_empty());
        assert!(item.diagnostics.is_empty());
        assert_eq!(
            item.source.anchor,
            mizar_core::core_ir::CoreSourceAnchor::SourceRange(mizar_session::SourceRange {
                source_id: checker_owner.source_id(),
                start: 13,
                end: 101,
            })
        );
        assert_eq!(
            item.source.provenance,
            vec![mizar_core::core_ir::CoreProvenance::new(
                mizar_core::core_ir::CoreProvenancePhase::Checker,
                "source-property-carrier-core-item-v1.structure",
            )]
        );
        let dependency = first
            .context()
            .item_registry()
            .dependencies(carrier_item)
            .expect("Task264 carrier dependency row");
        assert!(dependency.local.is_empty());
        assert!(dependency.external.is_empty());
        assert!(dependency.missing.is_empty());
        assert_eq!(
            first.context().source_map().item_sources.get(&carrier_item),
            Some(&item.source)
        );
        assert_eq!(first.context().source_map().item_sources.len(), 1);
        assert!(first.context().source_map().term_sources.is_empty());
        assert!(first.context().source_map().formula_sources.is_empty());
        assert!(first.context().source_map().definition_sources.is_empty());
        assert!(first.context().source_map().proof_sources.is_empty());
        assert!(first.context().source_map().algorithm_sources.is_empty());
        assert!(first.context().source_map().generated_sources.is_empty());
        assert!(first.context().source_map().obligation_sources.is_empty());

        let boundary = first
            .context()
            .definition_boundaries()
            .get_by_item(carrier_item)
            .expect("Task264 carrier pending boundary");
        assert_eq!(boundary.item, carrier_item);
        assert_eq!(boundary.symbol, *identity.structure_symbol());
        assert_eq!(
            boundary.kind,
            mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem
        );
        assert_eq!(
            boundary.status,
            mizar_core::elaborator::DefinitionBoundaryStatus::PendingBody
        );
        assert_eq!(boundary.source, item.source);
        assert_eq!(
            boundary.provenance.as_slice(),
            item.source.provenance.as_slice()
        );
        assert_eq!(
            first.context().worklist().entries(),
            &[mizar_core::elaborator::ElaborationWorkItem {
                kind: mizar_core::elaborator::ElaborationWorkItemKind::Item(carrier_item),
                status: mizar_core::elaborator::ElaborationWorkStatus::Pending,
                source: item.source.clone(),
                diagnostics: Vec::new(),
                checker_diagnostics: Vec::new(),
            }]
        );
    }
}

#[test]
fn task264_carrier_core_context_transports_uniform_prefix_geometry() {
    use mizar_core::{
        core_ir::{CoreProvenance, CoreProvenancePhase, CoreSourceRef},
        elaborator::{
            DefinitionBoundaryStatus, ElaborationWorkItemKind, ElaborationWorkStatus,
            SourcePropertyCarrierCoreContextError as CoreError,
            SourcePropertyCarrierCoreContextProducer as CoreProducer,
        },
    };

    let expected_slices = [
        "struct Task264Carrier where\n    field carrier -> set;\n    property marker -> set;\n  end;",
        "field carrier -> set;",
        "property marker -> set;",
    ];
    let mut prepared = Vec::new();
    for (profile_index, (profile, canonical)) in [
        ("means", SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT),
        ("equals", SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT),
    ]
    .into_iter()
    .enumerate()
    {
        let layouts = [
            (
                "leading_lf",
                format!("\n{canonical}"),
                [(14, 102), (46, 67), (72, 95)],
            ),
            (
                "first_definition_gap_comment",
                canonical.replacen("definition\n", "definition\n:: metamorphics\n", 1),
                [(29, 117), (61, 82), (87, 110)],
            ),
        ];
        for (layout_index, (layout, source, expected_ranges)) in layouts.into_iter().enumerate() {
            let (ast, module, shells, symbols, diagnostics) =
                task253_ast_from_source_text_with_diagnostic_count(
                    &source,
                    264_720 + profile_index * 10 + layout_index,
                );
            assert_eq!(diagnostics, 0, "{profile}/{layout}");
            let output = source_property_implementation_output(
                &ast, module, &shells, &symbols, &source,
            )
            .expect("Task264 prefix selector")
            .unwrap_or_else(|error| panic!("{profile}/{layout}: {error}"));
            assert!(output.typed_ast.source_type().is_some(), "{profile}/{layout}");
            let owner = output
                .typed_ast
                .source_property_implementation()
                .expect("Task264 prefix Property owner")
                .clone();
            let identity = owner.carrier_identity();
            let actual_ranges = [
                match identity.structure_origin().anchor() {
                    mizar_session::SourceAnchor::Range(range) => (range.start, range.end),
                    _ => panic!("Task264 carrier structure origin must have a source range"),
                },
                match identity.field_origin().anchor() {
                    mizar_session::SourceAnchor::Range(range) => (range.start, range.end),
                    _ => panic!("Task264 carrier field origin must have a source range"),
                },
                match identity.property_origin().anchor() {
                    mizar_session::SourceAnchor::Range(range) => (range.start, range.end),
                    _ => panic!("Task264 carrier property origin must have a source range"),
                },
            ];
            assert_eq!(actual_ranges, expected_ranges, "{profile}/{layout} literal ranges");
            for ((start, end), expected) in expected_ranges.into_iter().zip(expected_slices) {
                assert_eq!(source.get(start..end), Some(expected), "{profile}/{layout}");
            }
            let context = task264_carrier_core_context(&owner);
            prepared.push((profile, layout, owner, context));
        }
    }

    let outcomes = prepared
        .iter()
        .map(|(_, _, owner, context)| CoreProducer::build(context.clone(), owner.clone()))
        .collect::<Vec<_>>();

    // This native control must succeed before the four uniform Core outcomes are checked.
    let nonuniform = SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT.replacen(
        "struct Task264Carrier where",
        "struct Task264Carrier :: nonuniform\n where",
        1,
    );
    let (ast, module, shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(&nonuniform, 264_750);
    assert_eq!(diagnostics, 0);
    let output = source_property_implementation_output(&ast, module, &shells, &symbols, &nonuniform)
        .expect("native nonuniform Task264 selector")
        .expect("native nonuniform Task264 Type and Property output");
    let nonuniform_owner = output.typed_ast.source_property_implementation().unwrap();
    let identity = nonuniform_owner.carrier_identity();
    let nonuniform_ranges = [
        match identity.structure_origin().anchor() {
            mizar_session::SourceAnchor::Range(range) => (range.start, range.end),
            _ => panic!("native carrier origin must have a source range"),
        },
        match identity.field_origin().anchor() {
            mizar_session::SourceAnchor::Range(range) => (range.start, range.end),
            _ => panic!("native field origin must have a source range"),
        },
        match identity.property_origin().anchor() {
            mizar_session::SourceAnchor::Range(range) => (range.start, range.end),
            _ => panic!("native property origin must have a source range"),
        },
    ];
    assert_eq!(nonuniform_ranges, [(13, 116), (60, 81), (86, 109)]);
    assert_eq!(
        nonuniform.get(13..116),
        Some("struct Task264Carrier :: nonuniform\n where\n    field carrier -> set;\n    property marker -> set;\n  end;")
    );
    assert_eq!(
        CoreProducer::build(task264_carrier_core_context(nonuniform_owner), nonuniform_owner.clone())
            .expect_err("native nonuniform carrier geometry must fail closed"),
        CoreError::InvalidCheckerOwner
    );

    assert_eq!(outcomes.len(), 4);
    assert!(
        outcomes.iter().all(|outcome| outcome.is_ok()),
        "all four measured prefix layouts must reach Core: {outcomes:?}"
    );
    for ((profile, layout, owner, context), outcome) in prepared.iter().zip(&outcomes) {
        let handoff = outcome.as_ref().expect("Task264 prefix Core handoff");
        assert_eq!(handoff.context(), context, "{profile}/{layout}");
        assert_eq!(handoff.checker_owner(), owner, "{profile}/{layout}");
        assert_eq!(
            CoreProducer::build(task264_carrier_core_context(owner), owner.clone())
                .expect("fresh Task264 Core replay"),
            *handoff,
            "{profile}/{layout}"
        );
        let identity = owner.carrier_identity();
        let item_id = handoff.carrier_item();
        let core = handoff.context();
        assert_eq!(core.item_registry().id_for_symbol(identity.structure_symbol()), Some(item_id));
        assert_eq!(core.item_registry().items().len(), 1);
        let range = match identity.structure_origin().anchor() {
            mizar_session::SourceAnchor::Range(range) => *range,
            _ => panic!("Task264 carrier origin must have a source range"),
        };
        let source = CoreSourceRef::direct(range).with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::Checker,
            "source-property-carrier-core-item-v1.structure",
        )]);
        let item = core.item_registry().items().get(item_id).unwrap();
        assert_eq!(item.source, source, "{profile}/{layout}");
        assert!(item.dependencies.is_empty());
        let dependencies = core.item_registry().dependencies(item_id).unwrap();
        assert!(dependencies.local.is_empty() && dependencies.external.is_empty() && dependencies.missing.is_empty());
        assert_eq!(core.source_map().item_sources.get(&item_id), Some(&item.source));
        let boundary = core.definition_boundaries().get_by_item(item_id).unwrap();
        assert_eq!(boundary.status, DefinitionBoundaryStatus::PendingBody);
        assert_eq!(boundary.source, source);
        assert_eq!(boundary.provenance.as_slice(), source.provenance.as_slice());
        let work = &core.worklist().entries()[0];
        assert_eq!(work.kind, ElaborationWorkItemKind::Item(item_id));
        assert_eq!(work.status, ElaborationWorkStatus::Pending);
        assert_eq!(work.source, source);
        assert!(core.dependency_summaries().is_empty() && core.generated_origins().table().is_empty());
        assert!(core.diagnostics().is_empty() && core.source_map().obligation_sources.is_empty());
        let binder = core.binder_context();
        assert!(binder.frames.is_empty() && binder.free_variables.is_empty());
        assert!(binder.variable_classes.is_empty() && binder.variable_roles.is_empty() && binder.variable_sorts.is_empty());
        assert!(core.binder_sources().iter().next().is_none() && core.binder_type_facts().is_empty());
    }
    for owner in [&prepared[1].2, &prepared[3].2] {
        for mutation in [
            Task264CarrierCoreContextMutation::WrongSource,
            Task264CarrierCoreContextMutation::WrongProvenance,
            Task264CarrierCoreContextMutation::UnexpectedDependency,
        ] {
            assert_eq!(
                CoreProducer::build(task264_carrier_core_context_with_mutation(owner, mutation), owner.clone())
                    .expect_err("shifted checker owner still rejects a bad Core seed"),
                CoreError::InvalidCoreContext,
                "{mutation:?}"
            );
        }
    }
}

#[test]
fn task264_carrier_core_context_mutations_and_foreign_environment_fail_closed() {
    let (ast, module, shells, symbols) = task253_ast_from_source_text(
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
        264_500,
    );
    let output = source_property_implementation_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
    )
    .expect("Task264 selector")
    .expect("Task264 means route");
    let checker_owner = output
        .typed_ast
        .source_property_implementation()
        .expect("Task264 checker owner")
        .clone();

    for mutation in [
        Task264CarrierCoreContextMutation::MissingItem,
        Task264CarrierCoreContextMutation::ExtraItem,
        Task264CarrierCoreContextMutation::WrongKind,
        Task264CarrierCoreContextMutation::WrongVisibility,
        Task264CarrierCoreContextMutation::WrongSymbol,
        Task264CarrierCoreContextMutation::WrongSource,
        Task264CarrierCoreContextMutation::WrongProvenance,
        Task264CarrierCoreContextMutation::WrongBoundaryProvenance,
        Task264CarrierCoreContextMutation::MissingBoundary,
        Task264CarrierCoreContextMutation::WrongBoundary,
        Task264CarrierCoreContextMutation::UnexpectedDependency,
        Task264CarrierCoreContextMutation::UnexpectedGeneratedOrigin,
        Task264CarrierCoreContextMutation::UnexpectedCheckerSite,
        Task264CarrierCoreContextMutation::UnexpectedBinder,
    ] {
        let context = task264_carrier_core_context_with_mutation(&checker_owner, mutation);
        let error = mizar_core::elaborator::SourcePropertyCarrierCoreContextProducer::build(
            context,
            checker_owner.clone(),
        )
        .expect_err("Task264 carrier Core mutation must fail closed");
        assert_eq!(
            error,
            mizar_core::elaborator::SourcePropertyCarrierCoreContextError::InvalidCoreContext,
            "{mutation:?}"
        );
    }

    let (foreign_ast, foreign_module, foreign_shells, foreign_symbols) =
        task253_ast_from_source_text(SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT, 264_501);
    let foreign_output = source_property_implementation_output(
        &foreign_ast,
        foreign_module,
        &foreign_shells,
        &foreign_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
    )
    .expect("foreign Task264 selector")
    .expect("foreign Task264 route");
    let foreign_owner = foreign_output
        .typed_ast
        .source_property_implementation()
        .expect("foreign Task264 checker owner")
        .clone();
    for (context, owner) in [
        (
            task264_carrier_core_context(&checker_owner),
            foreign_owner.clone(),
        ),
        (
            task264_carrier_core_context(&foreign_owner),
            checker_owner.clone(),
        ),
    ] {
        assert_eq!(
            mizar_core::elaborator::SourcePropertyCarrierCoreContextProducer::build(
                context, owner,
            )
            .expect_err("foreign Task264 carrier environment must fail closed"),
            mizar_core::elaborator::SourcePropertyCarrierCoreContextError::EnvironmentMismatch
        );
    }
}

#[test]
fn task264_selector_type_context_is_exact_for_means_and_equals() {
    for (ordinal, source) in [
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(source, 264_600 + ordinal);
        let output = source_property_implementation_output(
            &ast, module, &shells, &symbols, source,
        )
        .expect("Task264 selector")
        .unwrap_or_else(|error| panic!("Task264 selector/type profile {ordinal}: {error}"));
        let checker_owner = output
            .typed_ast
            .source_property_implementation()
            .expect("Task264 checker owner")
            .clone();
        let source_type = output
            .typed_ast
            .source_type()
            .expect("Task264 complete source type")
            .clone();
        let carrier_context = task264_carrier_core_handoff(&checker_owner);
        let first =
            mizar_core::elaborator::SourcePropertySelectorTypeContextProducer::build(
                carrier_context.clone(),
                source_type.clone(),
            )
            .expect("Task264 selector/type context");
        let second =
            mizar_core::elaborator::SourcePropertySelectorTypeContextProducer::build(
                carrier_context.clone(),
                source_type.clone(),
            )
            .expect("Task264 selector/type replay");
        assert_eq!(first, second);
        assert_eq!(first.source_id(), checker_owner.source_id());
        assert_eq!(first.module_id(), checker_owner.module_id());
        assert_eq!(first.carrier_context(), &carrier_context);
        assert_eq!(first.source_type(), &source_type);
        assert_eq!(first.carrier_item(), carrier_context.carrier_item());

        let identity = checker_owner.carrier_identity();
        assert_eq!(source_type.applications().len(), 1);
        assert_eq!(source_type.expressions().len(), 3);
        assert!(source_type.arguments().is_empty());
        assert!(source_type.definition_returns().is_empty());
        assert!(source_type.mode_rhs().is_empty());
        assert_eq!(source_type.structure_members().len(), 2);
        let application = source_type
            .applications()
            .get(mizar_checker::source_type::SourceTypeApplicationId::new(0))
            .expect("Task264 parameter type application");
        assert_eq!(application.id().index(), 0);
        assert_eq!(application.binding().index(), 0);
        assert_eq!(application.source_ordinal(), 0);
        assert_eq!(application.root().index(), 0);

        let (application_site, application_head_site, member_sites) = if ordinal == 0 {
            (63, 64, [56, 59])
        } else {
            (45, 46, [38, 41])
        };
        for (index, (site, head_site, start, end)) in [
            (application_site, application_head_site, 130, 144),
            (member_sites[0] - 1, member_sites[0] - 2, 62, 65),
            (member_sites[1] - 1, member_sites[1] - 2, 90, 93),
        ]
        .into_iter()
        .enumerate()
        {
            let expression = source_type
                .expressions()
                .get(mizar_checker::source_type::SourceTypeExpressionId::new(index))
                .expect("Task264 exact type expression");
            let expected_range = mizar_session::SourceRange {
                source_id: checker_owner.source_id(),
                start,
                end,
            };
            assert_eq!(expression.id().index(), index);
            assert_eq!(expression.source_id(), checker_owner.source_id());
            assert_eq!(expression.module_id(), checker_owner.module_id());
            assert_eq!(
                expression.site(),
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(site)
                )
            );
            assert_eq!(expression.source_range(), expected_range);
            assert_eq!(
                expression.head_site(),
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(head_site)
                )
            );
            assert_eq!(expression.head_range(), expected_range);
            assert_eq!(
                expression.form(),
                mizar_checker::source_type::SourceTypeApplicationForm::Bare
            );
            assert_eq!(
                expression.recovery(),
                mizar_checker::typed_ast::NodeRecoveryState::Normal
            );
            if index == 0 {
                assert_eq!(expression.spelling(), "Task264Carrier");
                assert_eq!(expression.head_spelling(), "Task264Carrier");
                match expression.head() {
                    mizar_checker::source_type::SourceTypeHead::Symbol {
                        symbol,
                        contribution,
                    } => {
                        assert_eq!(symbol, identity.structure_symbol());
                        assert_eq!(*contribution, identity.structure_contribution());
                    }
                    other => panic!("unexpected Task264 carrier type head: {other:?}"),
                }
            } else {
                assert_eq!(expression.spelling(), "set");
                assert_eq!(expression.head_spelling(), "set");
                assert_eq!(
                    expression.head(),
                    &mizar_checker::source_type::SourceTypeHead::BuiltinSet
                );
            }
        }
        for (index, (site, start, end, root)) in [
            (member_sites[0], 45, 66, 1),
            (member_sites[1], 71, 94, 2),
        ]
        .into_iter()
        .enumerate()
        {
            let member = source_type
                .structure_members()
                .get(mizar_checker::source_type::SourceTypeStructureMemberId::new(index))
                .expect("Task264 exact member type row");
            assert_eq!(member.id().index(), index);
            assert_eq!(member.source_ordinal(), index);
            assert_eq!(member.root().index(), root);
            assert_eq!(
                member.member_site(),
                &mizar_checker::typed_ast::TypedSiteRef::Node(
                    mizar_checker::typed_ast::TypedNodeId::new(site)
                )
            );
            assert_eq!(
                member.member_range(),
                mizar_session::SourceRange {
                    source_id: checker_owner.source_id(),
                    start,
                    end,
                }
            );
        }

        let (parameter_id, parameter) = checker_owner
            .parameters()
            .iter()
            .next()
            .expect("Task264 property parameter row");
        let (target_id, target) = checker_owner
            .targets()
            .iter()
            .next()
            .expect("Task264 property target row");
        assert_eq!(parameter_id.index(), 0);
        assert_eq!(parameter.id().index(), 0);
        assert_eq!(parameter.binding().index(), 0);
        assert_eq!(target.subject(), parameter.binding());

        let domain = first.domain();
        assert_eq!(domain.binding(), parameter.binding());
        assert_eq!(domain.application(), parameter.written_type());
        assert_eq!(domain.application(), application.id());
        assert_eq!(application.binding(), domain.binding());
        assert_eq!(domain.root(), application.root());
        let domain_expression = source_type
            .expressions()
            .get(domain.root())
            .expect("Task264 property domain root");
        assert_eq!(domain_expression.id(), domain.root());
        match domain_expression.head() {
            mizar_checker::source_type::SourceTypeHead::Symbol {
                symbol,
                contribution,
            } => {
                assert_eq!(symbol, identity.structure_symbol());
                assert_eq!(*contribution, identity.structure_contribution());
            }
            other => panic!("unexpected Task264 domain head: {other:?}"),
        }
        assert_eq!(domain.carrier_item(), first.carrier_item());
        assert_eq!(domain.carrier_item().index(), 0);
        assert_eq!(
            first
                .carrier_context()
                .context()
                .item_registry()
                .id_for_symbol(identity.structure_symbol()),
            Some(domain.carrier_item())
        );

        let association = first.association();
        assert_eq!(target_id.index(), 0);
        assert_eq!(target.id().index(), 0);
        assert_eq!(target.symbol(), identity.property_symbol());
        assert_eq!(target.return_type(), association.member_type());
        assert_eq!(association.symbol(), identity.property_symbol());
        assert_eq!(association.member_type().index(), 1);
        assert_eq!(association.root().index(), 2);
        let definition_owner = first.definition_owner();
        assert_eq!(definition_owner.anchor_item(), first.carrier_item());
        assert_eq!(definition_owner.anchor_item().index(), 0);
        assert_eq!(definition_owner.item(), None);
        assert_eq!(definition_owner.property_symbol(), Some(association.symbol()));
        assert_eq!(
            source_type
                .structure_members()
                .get(association.member_type())
                .expect("Task264 property return member row")
                .root(),
            association.root()
        );
        assert_eq!(
            first.debug_text(),
            format!(
                "source-property-selector-type-context-v1|module={}.{}|carrier-item=0|property={}:2:0:1:2",
                checker_owner.module_id().package().as_str(),
                checker_owner.module_id().path().as_str(),
                identity.property_symbol().fqn().as_str(),
            )
        );
        assert_eq!(source_type.debug_text(), checker_owner.source_type_fingerprint());
        if ordinal == 1 {
            let terms = output
                .typed_ast
                .source_term()
                .expect("Task264 equals primary terms")
                .clone();
            let structures = output
                .typed_ast
                .source_structure()
                .expect("Task264 equals structure")
                .clone();
            let first = mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(
                &symbols,
                checker_owner.clone(),
                terms.clone(),
                structures.clone(),
            )
            .expect("Task264D equals selector identity");
            let second = mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(
                &symbols,
                checker_owner.clone(),
                terms.clone(),
                structures.clone(),
            )
            .expect("Task264D equals selector replay");
            assert_eq!(first, second);
            assert_eq!(first.property(), &checker_owner);
            assert_eq!(first.terms(), &terms);
            assert_eq!(first.structures(), &structures);
            let selector = first.association();
            assert_eq!(selector.implementation().index(), 0);
            assert_eq!(selector.definiens().index(), 0);
            assert_eq!(selector.structure_term().index(), 0);
            assert_eq!(selector.member().index(), 0);
            assert_eq!(selector.member_request().index(), 0);
            assert_eq!(selector.base_edge().index(), 0);
            assert_eq!(selector.base_term().index(), 0);
            assert_eq!(selector.base_reference().index(), 0);
            assert_eq!(selector.base_binding().index(), 0);
            assert_eq!(selector.selector_symbol(), identity.field_symbol());
            assert!(first
                .debug_text()
                .starts_with("source-property-equals-selector-identity-debug-v1\n"));
        }
    }
}

#[test]
fn task264_selector_type_context_transports_uniform_prefix_geometry() {
    use mizar_checker::source_type::{
        SourceTypeApplicationId as ApplicationId, SourceTypeExpressionId as ExpressionId,
    };
    use mizar_core::elaborator::{
        SourcePropertySelectorTypeContextError as SelectorError,
        SourcePropertySelectorTypeContextProducer as SelectorProducer,
    };

    let expected_slices = [
        "Task264Carrier",
        "set",
        "set",
        "field carrier -> set;",
        "property marker -> set;",
    ];
    let mut prepared = Vec::new();
    for (profile_index, (profile, canonical)) in [
        ("means", SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT),
        ("equals", SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT),
    ]
    .into_iter()
    .enumerate()
    {
        for (layout_index, (layout, source, expected_ranges)) in [
            (
                "leading_lf",
                format!("\n{canonical}"),
                [(131, 145), (63, 66), (91, 94), (46, 67), (72, 95)],
            ),
            (
                "first_definition_gap_comment",
                canonical.replacen("definition\n", "definition\n:: metamorphics\n", 1),
                [(146, 160), (78, 81), (106, 109), (61, 82), (87, 110)],
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let (ast, module, shells, symbols, diagnostics) =
                task253_ast_from_source_text_with_diagnostic_count(
                    &source,
                    264_760 + profile_index * 10 + layout_index,
                );
            assert_eq!(diagnostics, 0, "{profile}/{layout}");
            let output = source_property_implementation_output(
                &ast, module, &shells, &symbols, &source,
            )
            .expect("Task264 selector/type prefix route")
            .unwrap_or_else(|error| panic!("{profile}/{layout}: {error}"));
            let owner = output
                .typed_ast
                .source_property_implementation()
                .expect("Task264 selector/type Property owner")
                .clone();
            let source_type = output
                .typed_ast
                .source_type()
                .expect("Task264 selector/type handoff")
                .clone();
            let actual_ranges = [
                source_type.expressions().get(ExpressionId::new(0)).unwrap().source_range(),
                source_type.expressions().get(ExpressionId::new(1)).unwrap().source_range(),
                source_type.expressions().get(ExpressionId::new(2)).unwrap().source_range(),
                source_type.structure_members().get(mizar_checker::source_type::SourceTypeStructureMemberId::new(0)).unwrap().member_range(),
                source_type.structure_members().get(mizar_checker::source_type::SourceTypeStructureMemberId::new(1)).unwrap().member_range(),
            ];
            assert_eq!(
                actual_ranges.map(|range| (range.start, range.end)),
                expected_ranges,
                "{profile}/{layout} literal ranges"
            );
            for (index, expected) in expected_ranges.into_iter().zip(expected_slices) {
                assert_eq!(source.get(index.0..index.1), Some(expected), "{profile}/{layout}");
            }
            for (index, expected) in expected_ranges.into_iter().take(3).enumerate() {
                let expression = source_type.expressions().get(ExpressionId::new(index)).unwrap();
                assert_eq!(
                    (expression.head_range().start, expression.head_range().end),
                    expected,
                    "{profile}/{layout} expression head {index}"
                );
            }
            let carrier = task264_carrier_core_handoff(&owner);
            prepared.push((profile, layout, owner, carrier, source_type));
        }
    }

    let outcomes = prepared
        .iter()
        .map(|(_, _, _, carrier, source_type)| {
            SelectorProducer::build(carrier.clone(), source_type.clone())
        })
        .collect::<Vec<_>>();

    let between_blocks = SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT
        .replacen("\n\ndefinition", "\n\n\ndefinition", 1);
    let (ast, module, shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(&between_blocks, 264_790);
    assert_eq!(diagnostics, 0);
    let output = source_property_implementation_output(&ast, module, &shells, &symbols, &between_blocks)
        .expect("native between-block Task264 route")
        .expect("native between-block Task264 Some(Ok)");
    let control_owner = output.typed_ast.source_property_implementation().unwrap();
    let control_type = output.typed_ast.source_type().expect("native Task264 source type");
    let control_carrier = task264_carrier_core_handoff(control_owner);
    let carrier_range = match control_owner.carrier_identity().structure_origin().anchor() {
        mizar_session::SourceAnchor::Range(range) => (range.start, range.end),
        _ => panic!("Task264 carrier origin must have a source range"),
    };
    assert_eq!(carrier_range, (13, 101));
    let domain = control_type.expressions().get(ExpressionId::new(0)).unwrap();
    assert_eq!((domain.source_range().start, domain.source_range().end), (131, 145));
    assert_eq!((domain.head_range().start, domain.head_range().end), (131, 145));
    assert_eq!(between_blocks.get(131..145), Some("Task264Carrier"));
    assert_eq!(
        [
            control_type.expressions().get(ExpressionId::new(1)).unwrap().source_range(),
            control_type.expressions().get(ExpressionId::new(2)).unwrap().source_range(),
            control_type.structure_members().get(mizar_checker::source_type::SourceTypeStructureMemberId::new(0)).unwrap().member_range(),
            control_type.structure_members().get(mizar_checker::source_type::SourceTypeStructureMemberId::new(1)).unwrap().member_range(),
        ]
        .map(|range| (range.start, range.end)),
        [(62, 65), (90, 93), (45, 66), (71, 94)]
    );
    for (index, expected) in [(1, (62, 65)), (2, (90, 93))] {
        let expression = control_type.expressions().get(ExpressionId::new(index)).unwrap();
        assert_eq!((expression.head_range().start, expression.head_range().end), expected);
    }
    assert_eq!(
        SelectorProducer::build(control_carrier, control_type.clone())
            .expect_err("between-block domain drift must fail closed"),
        SelectorError::InvalidSourceType
    );

    assert_eq!(outcomes.len(), 4);
    assert!(
        outcomes.iter().all(|outcome| outcome.is_ok()),
        "all four measured selector/type prefix outcomes must succeed: {outcomes:?}"
    );
    for ((profile, layout, owner, carrier, source_type), outcome) in prepared.iter().zip(&outcomes) {
        let handoff = outcome.as_ref().expect("Task264 selector/type prefix handoff");
        assert_eq!(handoff.carrier_context(), carrier, "{profile}/{layout}");
        assert_eq!(handoff.source_type(), source_type, "{profile}/{layout}");
        let replay = SelectorProducer::build(carrier.clone(), source_type.clone())
            .expect("fresh Task264 selector/type replay");
        assert_eq!(&replay, handoff, "{profile}/{layout}");
        assert_eq!(handoff.definition_owner(), replay.definition_owner());
        let identity = owner.carrier_identity();
        let application = source_type.applications().get(ApplicationId::new(0)).unwrap();
        let (_, parameter) = owner.parameters().iter().next().unwrap();
        let domain = handoff.domain();
        assert_eq!(domain.binding(), parameter.binding());
        assert_eq!(domain.application(), parameter.written_type());
        assert_eq!(domain.application(), application.id());
        assert_eq!(domain.root(), application.root());
        assert_eq!(domain.carrier_item(), handoff.carrier_item());
        assert_eq!(carrier.context().item_registry().id_for_symbol(identity.structure_symbol()), Some(handoff.carrier_item()));
        let association = handoff.association();
        let (_, target) = owner.targets().iter().next().unwrap();
        assert_eq!(association.symbol(), identity.property_symbol());
        assert_eq!(association.symbol(), target.symbol());
        assert_eq!(association.member_type(), target.return_type());
        assert_eq!(
            source_type.structure_members().get(association.member_type()).unwrap().root(),
            association.root()
        );
        let definition_owner = handoff.definition_owner();
        assert_eq!(definition_owner.anchor_item(), handoff.carrier_item());
        assert_eq!(definition_owner.item(), None);
        assert_eq!(definition_owner.property_symbol(), Some(association.symbol()));
        let core = carrier.context();
        assert!(core.dependency_summaries().is_empty() && core.generated_origins().table().is_empty());
        assert!(core.diagnostics().is_empty() && core.source_map().obligation_sources.is_empty());
        let item = core.item_registry().items().get(handoff.carrier_item()).unwrap();
        assert!(item.dependencies.is_empty());
        let dependencies = core.item_registry().dependencies(handoff.carrier_item()).unwrap();
        assert!(dependencies.local.is_empty() && dependencies.external.is_empty() && dependencies.missing.is_empty());
        let sources = core.source_map();
        assert!(sources.term_sources.is_empty() && sources.formula_sources.is_empty());
        assert!(sources.definition_sources.is_empty() && sources.proof_sources.is_empty());
        assert!(sources.algorithm_sources.is_empty() && sources.generated_sources.is_empty());
        let binder = core.binder_context();
        assert!(binder.frames.is_empty() && binder.free_variables.is_empty());
        assert!(binder.variable_classes.is_empty() && binder.variable_roles.is_empty() && binder.variable_sorts.is_empty());
        assert!(core.binder_sources().iter().next().is_none() && core.binder_type_facts().is_empty());
    }
}

#[test]
fn task264_selector_type_cross_profile_and_foreign_transactions_fail_closed() {
    let shared_ordinal = 264_700;
    let (means_ast, means_module, means_shells, means_symbols) = task253_ast_from_source_text(
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
        shared_ordinal,
    );
    let means_output = source_property_implementation_output(
        &means_ast,
        means_module,
        &means_shells,
        &means_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
    )
    .expect("Task264 means selector")
    .expect("Task264 means route");
    let (equals_ast, equals_module, equals_shells, equals_symbols) = task253_ast_from_source_text(
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
        shared_ordinal,
    );
    let equals_output = source_property_implementation_output(
        &equals_ast,
        equals_module,
        &equals_shells,
        &equals_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    )
    .expect("Task264 equals selector")
    .expect("Task264 equals route");
    let means_owner = means_output
        .typed_ast
        .source_property_implementation()
        .expect("Task264 means owner")
        .clone();
    let equals_owner = equals_output
        .typed_ast
        .source_property_implementation()
        .expect("Task264 equals owner")
        .clone();
    let means_type = means_output
        .typed_ast
        .source_type()
        .expect("Task264 means type")
        .clone();
    let equals_type = equals_output
        .typed_ast
        .source_type()
        .expect("Task264 equals type")
        .clone();
    let means_terms = means_output
        .typed_ast
        .source_term()
        .expect("Task264 means terms")
        .clone();
    let equals_terms = equals_output
        .typed_ast
        .source_term()
        .expect("Task264 equals terms")
        .clone();
    let equals_structure = equals_output
        .typed_ast
        .source_structure()
        .expect("Task264 equals structure")
        .clone();
    assert_eq!(means_owner.source_id(), equals_owner.source_id());
    assert_eq!(means_owner.module_id(), equals_owner.module_id());
    assert_eq!(
        mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(
            &equals_symbols,
            means_owner.clone(),
            equals_terms.clone(),
            equals_structure.clone(),
        )
        .expect_err("Task264D means profile must remain unsupported"),
        mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityError::UnsupportedProfile
    );
    assert_eq!(
        mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(
            &equals_symbols,
            equals_owner.clone(),
            means_terms,
            equals_structure,
        )
        .expect_err("Task264D mixed lower profile must fail closed"),
        mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityError::DependencyMismatch
    );
    for (carrier, source_type) in [
        (task264_carrier_core_handoff(&means_owner), equals_type.clone()),
        (task264_carrier_core_handoff(&equals_owner), means_type.clone()),
    ] {
        assert_eq!(
            mizar_core::elaborator::SourcePropertySelectorTypeContextProducer::build(
                carrier,
                source_type,
            )
            .expect_err("same-environment mixed Task264 profile must fail at source type"),
            mizar_core::elaborator::SourcePropertySelectorTypeContextError::InvalidSourceType
        );
    }

    let (foreign_ast, foreign_module, foreign_shells, foreign_symbols) =
        task253_ast_from_source_text(SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT, 264_701);
    let foreign_output = source_property_implementation_output(
        &foreign_ast,
        foreign_module,
        &foreign_shells,
        &foreign_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
    )
    .expect("foreign Task264 selector")
    .expect("foreign Task264 route");
    let foreign_owner = foreign_output
        .typed_ast
        .source_property_implementation()
        .expect("foreign Task264 owner")
        .clone();
    let foreign_type = foreign_output
        .typed_ast
        .source_type()
        .expect("foreign Task264 type")
        .clone();
    for (carrier, source_type) in [
        (task264_carrier_core_handoff(&means_owner), foreign_type),
        (task264_carrier_core_handoff(&foreign_owner), means_type.clone()),
    ] {
        assert_eq!(
            mizar_core::elaborator::SourcePropertySelectorTypeContextProducer::build(
                carrier,
                source_type,
            )
            .expect_err("foreign Task264 selector/type environment must fail closed"),
            mizar_core::elaborator::SourcePropertySelectorTypeContextError::EnvironmentMismatch
        );
    }

    let valid_means = mizar_core::elaborator::SourcePropertySelectorTypeContextProducer::build(
        task264_carrier_core_handoff(&means_owner),
        means_type,
    )
    .expect("valid means transaction remains isolated");
    let valid_equals = mizar_core::elaborator::SourcePropertySelectorTypeContextProducer::build(
        task264_carrier_core_handoff(&equals_owner),
        equals_type,
    )
    .expect("valid equals transaction remains isolated");
    assert_ne!(valid_means.source_type(), valid_equals.source_type());
}

#[test]
fn task264_parameter_core_context_is_exact_for_means_and_equals() {
    for (ordinal, source) in [
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    ]
    .into_iter()
    .enumerate()
    {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(source, 264_800 + ordinal);
        let output = source_property_implementation_output(
            &ast, module, &shells, &symbols, source,
        )
        .expect("Task264 parameter selector")
        .unwrap_or_else(|error| panic!("Task264 parameter profile {ordinal}: {error}"));
        let selector_context = task264_selector_type_handoff(&output);
        let source_context = output
            .typed_ast
            .source_context()
            .expect("Task264 parameter source context")
            .clone();
        let first = mizar_core::elaborator::SourcePropertyParameterCoreContextProducer::build(
            selector_context.clone(),
            source_context.clone(),
        )
        .expect("Task264 parameter Core context");
        let second = mizar_core::elaborator::SourcePropertyParameterCoreContextProducer::build(
            selector_context.clone(),
            source_context.clone(),
        )
        .expect("Task264 parameter Core replay");
        assert_eq!(first, second);
        assert_eq!(first.source_id(), selector_context.source_id());
        assert_eq!(first.module_id(), selector_context.module_id());
        assert_eq!(first.selector_context(), &selector_context);
        assert_eq!(first.source_context(), &source_context);
        assert_eq!(first.context(), first.source_bindings().context());
        assert_eq!(first.source_bindings().binding_env(), source_context.binding_env());

        let association = first.association();
        assert_eq!(association.parameter().index(), 0);
        assert_eq!(association.binding().index(), 0);
        assert_eq!(association.core_var().index(), 0);
        let variable = first
            .source_bindings()
            .variables()
            .get(association.binding())
            .expect("Task264 parameter Core variable");
        assert_eq!(variable.binding(), association.binding());
        assert_eq!(variable.core_var(), association.core_var());
        assert_eq!(first.source_bindings().variables().len(), 1);
        assert_eq!(first.context().binder_context().free_variables.len(), 1);
        assert!(first
            .context()
            .binder_context()
            .free_variables
            .contains(&association.core_var()));
        assert_eq!(
            first
                .context()
                .binder_context()
                .variable_classes
                .get(&association.core_var()),
            Some(&mizar_core::binder_normalization::NormalizedVarClass::Free)
        );
        assert_eq!(
            first
                .context()
                .binder_context()
                .variable_roles
                .get(&association.core_var())
                .expect("Task264 parameter role")
                .as_str(),
            "definition-parameter"
        );
        assert_eq!(
            first
                .context()
                .binder_context()
                .variable_sorts
                .get(&association.core_var()),
            Some(&mizar_core::binder_normalization::NormalizedVarSort::Term)
        );
        assert_eq!(
            first
                .context()
                .binder_type_facts()
                .get(&association.core_var()),
            Some(&Vec::new())
        );
        assert!(first.context().binder_context().frames.is_empty());
        let source_record = first
            .context()
            .binder_sources()
            .get(association.core_var())
            .expect("Task264 parameter binder source");
        assert_eq!(
            source_record.source.anchor,
            mizar_core::core_ir::CoreSourceAnchor::SourceRange(mizar_session::SourceRange {
                source_id: first.source_id(),
                start: 125,
                end: 126,
            })
        );
        assert_eq!(source_record.source.provenance.len(), 1);
        assert_eq!(source_record.provenance.as_slice().len(), 1);
        assert_eq!(
            source_record.source.provenance[0].key.as_str(),
            "source-binding-core-variable-v1.binding.0"
        );
        assert_eq!(
            source_record.provenance.as_slice()[0].key.as_str(),
            "source-binding-core-variable-v1.binding.0"
        );
        let carrier_symbol = selector_context
            .carrier_context()
            .checker_owner()
            .carrier_identity()
            .structure_symbol();
        assert_eq!(
            first.context().item_registry().id_for_symbol(carrier_symbol),
            Some(selector_context.carrier_item())
        );
        assert_eq!(first.context().item_registry().items().len(), 1);
        assert_eq!(
            first.debug_text(),
            format!(
                "source-property-parameter-core-context-v1|module={}.{}|carrier-item=0|bindings=1|parameter=0:0:0",
                first.module_id().package().as_str(),
                first.module_id().path().as_str(),
            )
        );
    }
}

#[test]
fn task264_parameter_core_context_joins_uniform_domain_prefix_ranges() {
    use mizar_core::{
        core_ir::{CoreProvenance, CoreProvenancePhase, CoreSourceRef},
        elaborator::{
            SourcePropertyParameterCoreContextError as ParameterError,
            SourcePropertyParameterCoreContextProducer as ParameterProducer,
        },
    };

    let mut prepared = Vec::new();
    for (profile_index, (profile, canonical)) in [
        ("means", SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT),
        ("equals", SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT),
    ]
    .into_iter()
    .enumerate()
    {
        for (layout, source, declaration, written_type) in [
            (
                "leading_lf",
                format!("\n{canonical}"),
                (126, 127),
                (131, 145),
            ),
            (
                "first_definition_gap_comment",
                canonical.replacen("definition\n", "definition\n:: metamorphics\n", 1),
                (141, 142),
                (146, 160),
            ),
        ] {
            let (ast, module, shells, symbols, diagnostics) =
                task253_ast_from_source_text_with_diagnostic_count(
                    &source,
                    264_940 + profile_index,
                );
            assert_eq!(diagnostics, 0, "{profile}/{layout}");
            let output = source_property_implementation_output(
                &ast, module, &shells, &symbols, &source,
            )
            .expect("Task264 parameter prefix route")
            .unwrap_or_else(|error| panic!("{profile}/{layout}: {error}"));
            let selector = task264_selector_type_handoff(&output);
            let source_context = output
                .typed_ast
                .source_context()
                .expect("Task264 parameter SourceContext")
                .clone();
            let (declaration_id, row) = source_context
                .declarations()
                .iter()
                .next()
                .expect("Task264 parameter declaration");
            let expected_declaration = mizar_session::SourceRange {
                source_id: source_context.source_id(),
                start: declaration.0,
                end: declaration.1,
            };
            let expected_written_type = mizar_session::SourceRange {
                source_id: source_context.source_id(),
                start: written_type.0,
                end: written_type.1,
            };
            assert_eq!(declaration_id.index(), 0);
            assert_eq!(row.declaration_range, expected_declaration, "{profile}/{layout}");
            assert_eq!(row.written_type_range, expected_written_type, "{profile}/{layout}");
            assert_eq!(source.get(declaration.0..declaration.1), Some("M"), "{profile}/{layout}");
            assert_eq!(source.get(written_type.0..written_type.1), Some("Task264Carrier"), "{profile}/{layout}");
            assert_eq!(source_context.source_id(), ast.source_id, "{profile}/{layout}");
            let domain = selector.source_type().expressions().get(selector.domain().root()).unwrap();
            assert_eq!(domain.source_range(), expected_written_type, "{profile}/{layout}");
            assert_eq!(domain.source_id(), source_context.source_id(), "{profile}/{layout}");
            assert_eq!(selector.source_id(), source_context.source_id(), "{profile}/{layout}");
            assert_eq!(selector.module_id(), source_context.module_id(), "{profile}/{layout}");
            prepared.push((profile, layout, selector, source_context));
        }
    }

    let outcomes = prepared
        .iter()
        .map(|(_, _, selector, source_context)| {
            ParameterProducer::build(selector.clone(), source_context.clone())
        })
        .collect::<Vec<_>>();

    for (left, right) in [(0, 1), (2, 3)] {
        let (_, left_layout, left_selector, left_context) = &prepared[left];
        let (_, right_layout, right_selector, right_context) = &prepared[right];
        for (selector, context) in [
            (left_selector.clone(), right_context.clone()),
            (right_selector.clone(), left_context.clone()),
        ] {
            assert_eq!(selector.source_id(), context.source_id());
            assert_eq!(selector.module_id(), context.module_id());
            assert_eq!(
                ParameterProducer::build(selector, context)
                    .expect_err("same-environment mixed domain layout must fail closed"),
                ParameterError::InvalidSourceContext,
                "{left_layout}/{right_layout}"
            );
        }
    }

    assert_eq!(outcomes.len(), 4);
    assert!(
        outcomes.iter().all(|outcome| outcome.is_ok()),
        "all four measured parameter domain layouts must succeed: {outcomes:?}"
    );
    for ((profile, layout, selector, source_context), outcome) in prepared.iter().zip(&outcomes) {
        let handoff = outcome.as_ref().expect("Task264 parameter prefix handoff");
        assert_eq!(handoff.selector_context(), selector, "{profile}/{layout}");
        assert_eq!(handoff.source_context(), source_context, "{profile}/{layout}");
        assert_eq!(handoff.context(), handoff.source_bindings().context());
        assert_eq!(handoff.source_bindings().binding_env(), source_context.binding_env());
        let replay = ParameterProducer::build(selector.clone(), source_context.clone())
            .expect("fresh Task264 parameter replay");
        assert_eq!(&replay, handoff, "{profile}/{layout}");

        let association = handoff.association();
        assert_eq!(association.parameter().index(), 0);
        assert_eq!(association.binding().index(), 0);
        assert_eq!(association.core_var().index(), 0);
        let variable = handoff.source_bindings().variables().get(association.binding()).unwrap();
        assert_eq!(variable.binding(), association.binding());
        assert_eq!(variable.core_var(), association.core_var());
        let declaration = source_context.declarations().iter().next().unwrap().1;
        let source_record = handoff.context().binder_sources().get(association.core_var()).unwrap();
        let expected_source = CoreSourceRef::direct(declaration.declaration_range).with_provenance(vec![
            CoreProvenance::new(
                CoreProvenancePhase::Checker,
                "source-binding-core-variable-v1.binding.0",
            ),
        ]);
        assert_eq!(source_record.source, expected_source, "{profile}/{layout}");
        assert_eq!(source_record.provenance.as_slice(), expected_source.provenance.as_slice());
        let binder = handoff.context().binder_context();
        assert!(binder.frames.is_empty());
        assert!(binder.free_variables.contains(&association.core_var()));
        assert_eq!(binder.variable_classes.get(&association.core_var()), Some(&mizar_core::binder_normalization::NormalizedVarClass::Free));
        assert_eq!(binder.variable_roles.get(&association.core_var()).unwrap().as_str(), "definition-parameter");
        assert_eq!(binder.variable_sorts.get(&association.core_var()), Some(&mizar_core::binder_normalization::NormalizedVarSort::Term));
        assert_eq!(handoff.context().binder_type_facts().get(&association.core_var()), Some(&Vec::new()));
        assert!(handoff.context().source_map().term_sources.is_empty());
        assert!(handoff.context().source_map().formula_sources.is_empty());
        assert!(handoff.context().source_map().definition_sources.is_empty());
        assert!(handoff.context().source_map().proof_sources.is_empty());
        assert!(handoff.context().source_map().algorithm_sources.is_empty());
        assert!(handoff.context().source_map().obligation_sources.is_empty());
        assert!(handoff.context().dependency_summaries().is_empty());
        assert!(handoff.context().generated_origins().table().is_empty());
        assert!(handoff.context().diagnostics().is_empty());
    }
}

#[test]
fn task264_parameter_core_context_rejects_mixed_and_foreign_transactions() {
    let shared_ordinal = 264_900;
    let (means_ast, means_module, means_shells, means_symbols) = task253_ast_from_source_text(
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
        shared_ordinal,
    );
    let means_output = source_property_implementation_output(
        &means_ast,
        means_module,
        &means_shells,
        &means_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
    )
    .expect("Task264 parameter means selector")
    .expect("Task264 parameter means route");
    let (equals_ast, equals_module, equals_shells, equals_symbols) = task253_ast_from_source_text(
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
        shared_ordinal,
    );
    let equals_output = source_property_implementation_output(
        &equals_ast,
        equals_module,
        &equals_shells,
        &equals_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    )
    .expect("Task264 parameter equals selector")
    .expect("Task264 parameter equals route");
    let means_selector = task264_selector_type_handoff(&means_output);
    let equals_selector = task264_selector_type_handoff(&equals_output);
    let means_context = means_output
        .typed_ast
        .source_context()
        .expect("Task264 means source context")
        .clone();
    let equals_context = equals_output
        .typed_ast
        .source_context()
        .expect("Task264 equals source context")
        .clone();
    assert_eq!(means_selector.source_id(), equals_selector.source_id());
    assert_eq!(means_selector.module_id(), equals_selector.module_id());
    for (selector, context) in [
        (means_selector.clone(), equals_context),
        (equals_selector.clone(), means_context.clone()),
    ] {
        assert_eq!(
            mizar_core::elaborator::SourcePropertyParameterCoreContextProducer::build(
                selector, context,
            )
            .expect_err("mixed Task264 parameter context must fail closed"),
            mizar_core::elaborator::SourcePropertyParameterCoreContextError::InvalidSourceContext
        );
    }

    let (foreign_ast, foreign_module, foreign_shells, foreign_symbols) =
        task253_ast_from_source_text(SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT, 264_901);
    let foreign_output = source_property_implementation_output(
        &foreign_ast,
        foreign_module,
        &foreign_shells,
        &foreign_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
    )
    .expect("foreign Task264 parameter selector")
    .expect("foreign Task264 parameter route");
    let foreign_selector = task264_selector_type_handoff(&foreign_output);
    assert_eq!(
        mizar_core::elaborator::SourcePropertyParameterCoreContextProducer::build(
            foreign_selector,
            means_context,
        )
        .expect_err("foreign Task264 parameter environment must fail closed"),
        mizar_core::elaborator::SourcePropertyParameterCoreContextError::EnvironmentMismatch
    );
}

#[test]
fn task264_equals_selector_term_seeds_are_exact_and_deterministic() {
    let (ast, module, shells, symbols) = task253_ast_from_source_text(
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
        265_000,
    );
    let output = source_property_implementation_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    )
    .expect("Task35E264 selector")
    .expect("Task35E264 equals route");
    let checker_owner = output
        .typed_ast
        .source_property_implementation()
        .expect("Task35E264 checker owner")
        .clone();
    let selector_identity = mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(
        &symbols,
        checker_owner.clone(),
        output
            .typed_ast
            .source_term()
            .expect("Task35E264 primary term")
            .clone(),
        output
            .typed_ast
            .source_structure()
            .expect("Task35E264 structure term")
            .clone(),
    )
    .expect("Task35E264 selector identity");
    let selector_context = task264_selector_type_handoff(&output);
    let parameter_context =
        mizar_core::elaborator::SourcePropertyParameterCoreContextProducer::build(
            selector_context.clone(),
            output
                .typed_ast
                .source_context()
                .expect("Task35E264 source context")
                .clone(),
        )
        .expect("Task35E264 parameter context");
    let first = mizar_core::elaborator::SourcePropertyEqualsSelectorTermSeedProducer::build(
        parameter_context.clone(),
        selector_identity.clone(),
    )
    .expect("Task35E264 term seeds");
    let second = mizar_core::elaborator::SourcePropertyEqualsSelectorTermSeedProducer::build(
        parameter_context.clone(),
        selector_identity.clone(),
    )
    .expect("Task35E264 deterministic replay");

    assert_eq!(first, second);
    assert_eq!(first.source_id(), parameter_context.source_id());
    assert_eq!(first.module_id(), parameter_context.module_id());
    assert_eq!(first.parameter_context(), &parameter_context);
    assert_eq!(first.selector_identity(), &selector_identity);
    assert_eq!(first.definition_owner(), &selector_context.definition_owner());
    assert_eq!(first.definition_owner().anchor_item().index(), 0);
    assert_eq!(first.definition_owner().item(), None);
    assert_eq!(
        first.definition_owner().property_symbol(),
        Some(checker_owner.carrier_identity().property_symbol())
    );

    let association = first.association();
    assert_eq!(association.parameter().index(), 0);
    assert_eq!(association.binding().index(), 0);
    assert_eq!(association.core_var().index(), 0);
    assert_eq!(association.source_base().index(), 0);
    assert_eq!(association.base_seed().index(), 0);
    assert_eq!(association.source_selector().index(), 0);
    assert_eq!(association.selector_seed().index(), 1);

    let terms = first.terms();
    assert_eq!(terms.len(), 2);
    assert_eq!(
        terms[0].kind,
        mizar_core::elaborator::CoreTermSeedKind::Var(
            mizar_core::core_ir::CoreVarId::new(0)
        )
    );
    assert_eq!(
        terms[0].source.anchor,
        mizar_core::core_ir::CoreSourceAnchor::SourceRange(mizar_session::SourceRange {
            source_id: first.source_id(),
            start: 173,
            end: 174,
        })
    );
    assert!(terms[0].source.provenance.is_empty());
    assert_eq!(terms[0].provenance.as_slice().len(), 1);
    assert_eq!(
        terms[0].provenance.as_slice()[0].phase,
        mizar_core::core_ir::CoreProvenancePhase::Checker
    );
    assert_eq!(
        terms[0].provenance.as_slice()[0].key.as_str(),
        "source-property-equals-selector-term-seed-v1.base"
    );
    assert_eq!(
        terms[1].kind,
        mizar_core::elaborator::CoreTermSeedKind::Select {
            selector: checker_owner.carrier_identity().field_symbol().clone(),
            base: mizar_core::elaborator::CoreTermSeedId::new(0),
        }
    );
    assert_eq!(
        terms[1].source.anchor,
        mizar_core::core_ir::CoreSourceAnchor::SourceRange(mizar_session::SourceRange {
            source_id: first.source_id(),
            start: 173,
            end: 182,
        })
    );
    assert!(terms[1].source.provenance.is_empty());
    assert_eq!(terms[1].provenance.as_slice().len(), 1);
    assert_eq!(
        terms[1].provenance.as_slice()[0].phase,
        mizar_core::core_ir::CoreProvenancePhase::Checker
    );
    assert_eq!(
        terms[1].provenance.as_slice()[0].key.as_str(),
        "source-property-equals-selector-term-seed-v1.selector"
    );
    assert_eq!(
        first.debug_text(),
        format!(
            concat!(
                "source-property-equals-selector-term-seeds-v1|module={}.{}|",
                "owner-anchor=0|property={}|selector={}|source=0:0|seed=0:1|",
                "parameter=0:0:0"
            ),
            first.module_id().package().as_str(),
            first.module_id().path().as_str(),
            checker_owner
                .carrier_identity()
                .property_symbol()
                .fqn()
                .as_str(),
            checker_owner
                .carrier_identity()
                .field_symbol()
                .fqn()
                .as_str(),
        )
    );
    assert!(!first.debug_text().ends_with('\n'));
}

#[test]
fn task264_equals_selector_term_seeds_transport_uniform_prefix_geometry() {
    use mizar_core::{
        core_ir::{CoreProvenance, CoreProvenancePhase, CoreSourceRef, CoreTermKind, CoreVarId},
        elaborator::{
            CoreTermSeedId, CoreTermSeedKind,
            SourcePropertyEqualsSelectorTermLoweringProducer as LoweringProducer,
            SourcePropertyEqualsSelectorTermSeedError as SeedError,
            SourcePropertyEqualsSelectorTermSeedProducer as SeedProducer,
            SourcePropertyParameterCoreContextProducer as ParameterProducer,
        },
    };

    let layouts = [
        ("leading_lf", format!("\n{SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT}"), [(174, 175), (174, 183), (176, 183)]),
        ("first_definition_gap_comment", SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT.replacen("definition\n", "definition\n:: metamorphics\n", 1), [(189, 190), (189, 198), (191, 198)]),
    ];
    let mut prepared = Vec::new();
    for (index, (layout, source, literal_ranges)) in layouts.into_iter().enumerate() {
        let (ast, module, shells, symbols, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(&source, 265_500 + index);
        assert_eq!(diagnostics, 0, "{layout}");
        let output = source_property_implementation_output(&ast, module, &shells, &symbols, &source)
            .expect("Task264 Equals prefix route")
            .unwrap_or_else(|error| panic!("{layout}: {error}"));
        let checker_owner = output.typed_ast.source_property_implementation()
            .expect("Task264 Equals checker owner").clone();
        let terms = output.typed_ast.source_term().expect("Task264 Equals primary terms");
        let structures = output.typed_ast.source_structure().expect("Task264 Equals structure terms");
        let identity = mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(
            &symbols, checker_owner.clone(), terms.clone(), structures.clone(),
        ).expect("Task264 Equals checker identity");
        assert_eq!(identity.property(), &checker_owner, "{layout}");
        let selector_context = task264_selector_type_handoff(&output);
        let source_context = output.typed_ast.source_context().expect("Task264 Equals SourceContext").clone();
        let parameter = ParameterProducer::build(selector_context.clone(), source_context.clone())
            .expect("Task264 Equals parameter context");
        let association = identity.association();
        let base = terms.terms().get(association.base_term()).expect("Task264 Equals base M");
        let selector = structures.terms().get(association.structure_term()).expect("Task264 Equals selector M.carrier");
        let member = structures.members().get(association.member()).expect("Task264 Equals member carrier");
        let source_id = ast.source_id;
        assert_eq!((checker_owner.source_id(), terms.source_id(), structures.source_id(), identity.source_id(), parameter.source_id()),
            (source_id, source_id, source_id, source_id, source_id), "{layout} source IDs");
        let ranges = literal_ranges.map(|(start, end)| mizar_session::SourceRange { source_id, start, end });
        assert_eq!([base.source_range(), selector.source_range(), member.source_range()], ranges, "{layout} literal ranges");
        for (range, expected) in ranges.into_iter().zip(["M", "M.carrier", "carrier"]) {
            assert_eq!(source.get(range.start..range.end), Some(expected), "{layout} physical source");
        }
        prepared.push((layout, parameter, identity, ranges));
    }

    let outcomes = prepared.iter().map(|(_, parameter, identity, _)| {
        SeedProducer::build(parameter.clone(), identity.clone())
    }).collect::<Vec<_>>();

    let body_only_lf = SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT
        .replacen("  property M.marker equals", "\n  property M.marker equals", 1);
    let (ast, module, shells, symbols, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(&body_only_lf, 265_502);
    assert_eq!(diagnostics, 0, "body-only LF control");
    let output = source_property_implementation_output(&ast, module, &shells, &symbols, &body_only_lf)
        .expect("native body-only LF route")
        .expect("native body-only LF Some(Ok)");
    let checker_owner = output.typed_ast.source_property_implementation().unwrap().clone();
    let terms = output.typed_ast.source_term().unwrap().clone();
    let structures = output.typed_ast.source_structure().unwrap().clone();
    let identity = mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(
        &symbols, checker_owner.clone(), terms.clone(), structures.clone(),
    ).expect("body-only LF checker identity");
    let selector_context = task264_selector_type_handoff(&output);
    let parameter = ParameterProducer::build(
        selector_context.clone(), output.typed_ast.source_context().unwrap().clone(),
    ).expect("body-only LF parameter context");
    let carrier_range = match checker_owner.carrier_identity().structure_origin().anchor() {
        mizar_session::SourceAnchor::Range(range) => (range.start, range.end),
        _ => panic!("body-only LF carrier origin must have a source range"),
    };
    assert_eq!(carrier_range, (13, 101));
    let declaration = output.typed_ast.source_context().unwrap().declarations().iter().next().unwrap().1;
    assert_eq!((declaration.declaration_range.start, declaration.declaration_range.end), (125, 126));
    assert_eq!(declaration.declaration_range.source_id, ast.source_id);
    let domain = selector_context.source_type().expressions().get(selector_context.domain().root()).unwrap();
    assert_eq!((domain.source_range().start, domain.source_range().end), (130, 144));
    assert_eq!(domain.source_id(), ast.source_id);
    assert_eq!(body_only_lf.get(130..144), Some("Task264Carrier"));
    let control_ranges = [
        terms.terms().get(identity.association().base_term()).unwrap().source_range(),
        structures.terms().get(identity.association().structure_term()).unwrap().source_range(),
        structures.members().get(identity.association().member()).unwrap().source_range(),
    ];
    assert_eq!(control_ranges, [(174, 175), (174, 183), (176, 183)]
        .map(|(start, end)| mizar_session::SourceRange { source_id: ast.source_id, start, end }));
    for (range, expected) in control_ranges.into_iter().zip(["M", "M.carrier", "carrier"]) {
        assert_eq!(body_only_lf.get(range.start..range.end), Some(expected));
    }
    assert_eq!(
        SeedProducer::build(parameter, identity)
            .expect_err("body-only LF selector displacement must fail closed"),
        SeedError::InvalidSelectorIdentity,
    );

    assert_eq!(outcomes.len(), 2);
    assert!(outcomes.iter().all(Result::is_ok), "both measured Equals prefix seeds must succeed: {outcomes:?}");
    for ((layout, parameter, identity, ranges), outcome) in prepared.iter().zip(&outcomes) {
        let seed = outcome.as_ref().expect("Task264 Equals prefix seed");
        let replay = SeedProducer::build(parameter.clone(), identity.clone()).expect("fresh seed replay");
        assert_eq!(&replay, seed, "{layout} seed replay");
        assert_eq!(seed.parameter_context(), parameter);
        assert_eq!(seed.selector_identity(), identity);
        assert_eq!(seed.source_id(), parameter.source_id());
        assert_eq!(seed.module_id(), parameter.module_id());
        let owner = seed.definition_owner();
        assert_eq!(owner.anchor_item().index(), 0);
        assert!(owner.item().is_none());
        assert_eq!(owner.property_symbol(), Some(identity.property().carrier_identity().property_symbol()));

        let association = seed.association();
        assert_eq!((association.parameter().index(), association.binding().index(), association.core_var().index()), (0, 0, 0));
        assert_eq!((association.source_base().index(), association.base_seed().index()), (0, 0));
        assert_eq!((association.source_selector().index(), association.selector_seed().index()), (0, 1));
        let seeds = seed.terms();
        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0].kind, CoreTermSeedKind::Var(CoreVarId::new(0)));
        assert_eq!(seeds[1].kind, CoreTermSeedKind::Select {
            selector: identity.association().selector_symbol().clone(), base: CoreTermSeedId::new(0),
        });
        for (index, (range, key)) in ranges.iter().zip([
            "source-property-equals-selector-term-seed-v1.base",
            "source-property-equals-selector-term-seed-v1.selector",
        ]).enumerate() {
            assert_eq!(seeds[index].source, CoreSourceRef::direct(*range), "{layout} direct source");
            assert_eq!(seeds[index].provenance.as_slice(), [CoreProvenance::new(CoreProvenancePhase::Checker, key)], "{layout} checker provenance");
        }

        let lowered = LoweringProducer::build(seed.clone()).expect("Task35L264 local lowering");
        assert_eq!(lowered.seed_handoff(), seed);
        assert_eq!(lowered.definition_owner(), owner);
        let local = lowered.association();
        assert_eq!((local.base_seed().index(), local.base_term().index()), (0, 0));
        assert_eq!((local.selector_seed().index(), local.selector_term().index(), local.root_term().index()), (1, 1, 1));
        assert_eq!(lowered.terms().len(), 2);
        let expected_sources = ranges.iter().zip(seeds).map(|(range, seed)| {
            CoreSourceRef::direct(*range).with_provenance(seed.provenance.as_slice().to_vec())
        }).collect::<Vec<_>>();
        let local_base = lowered.terms().get(local.base_term()).expect("lowered base term");
        let local_selector = lowered.terms().get(local.selector_term()).expect("lowered selector term");
        assert_eq!(local_base.kind, CoreTermKind::Var(CoreVarId::new(0)));
        assert_eq!(local_selector.kind, CoreTermKind::Select {
            selector: identity.association().selector_symbol().clone(), base: local.base_term(),
        });
        assert_eq!([&local_base.source, &local_selector.source], [&expected_sources[0], &expected_sources[1]]);
        let map = lowered.source_map();
        assert_eq!(map.term_sources.len(), 2);
        assert_eq!(map.term_sources.get(&local.base_term()), Some(&expected_sources[0]));
        assert_eq!(map.term_sources.get(&local.selector_term()), Some(&expected_sources[1]));
        assert!(map.item_sources.is_empty() && map.formula_sources.is_empty() && map.definition_sources.is_empty());
        assert!(map.proof_sources.is_empty() && map.algorithm_sources.is_empty() && map.generated_sources.is_empty() && map.obligation_sources.is_empty());
    }
}

#[test]
fn task264_equals_selector_term_seeds_reject_mixed_and_foreign_transactions() {
    let shared_ordinal = 265_100;
    let (means_ast, means_module, means_shells, means_symbols) = task253_ast_from_source_text(
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
        shared_ordinal,
    );
    let means_output = source_property_implementation_output(
        &means_ast,
        means_module,
        &means_shells,
        &means_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
    )
    .expect("Task35E264 means selector")
    .expect("Task35E264 means route");
    let (equals_ast, equals_module, equals_shells, equals_symbols) = task253_ast_from_source_text(
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
        shared_ordinal,
    );
    let equals_output = source_property_implementation_output(
        &equals_ast,
        equals_module,
        &equals_shells,
        &equals_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    )
    .expect("Task35E264 equals selector")
    .expect("Task35E264 equals route");
    let means_parameter =
        mizar_core::elaborator::SourcePropertyParameterCoreContextProducer::build(
            task264_selector_type_handoff(&means_output),
            means_output
                .typed_ast
                .source_context()
                .expect("Task35E264 means source context")
                .clone(),
        )
        .expect("Task35E264 means parameter context");
    let equals_owner = equals_output
        .typed_ast
        .source_property_implementation()
        .expect("Task35E264 equals owner")
        .clone();
    let equals_identity = mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(
        &equals_symbols,
        equals_owner,
        equals_output
            .typed_ast
            .source_term()
            .expect("Task35E264 equals primary term")
            .clone(),
        equals_output
            .typed_ast
            .source_structure()
            .expect("Task35E264 equals structure term")
            .clone(),
    )
    .expect("Task35E264 equals selector identity");
    assert_eq!(means_parameter.source_id(), equals_identity.source_id());
    assert_eq!(means_parameter.module_id(), equals_identity.module_id());
    assert_eq!(
        mizar_core::elaborator::SourcePropertyEqualsSelectorTermSeedProducer::build(
            means_parameter,
            equals_identity.clone(),
        )
        .expect_err("same-environment mixed Task35E264 transaction must fail closed"),
        mizar_core::elaborator::SourcePropertyEqualsSelectorTermSeedError::InvalidSelectorIdentity
    );

    let (foreign_ast, foreign_module, foreign_shells, foreign_symbols) =
        task253_ast_from_source_text(SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT, 265_101);
    let foreign_output = source_property_implementation_output(
        &foreign_ast,
        foreign_module,
        &foreign_shells,
        &foreign_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    )
    .expect("foreign Task35E264 selector")
    .expect("foreign Task35E264 equals route");
    let foreign_parameter =
        mizar_core::elaborator::SourcePropertyParameterCoreContextProducer::build(
            task264_selector_type_handoff(&foreign_output),
            foreign_output
                .typed_ast
                .source_context()
                .expect("foreign Task35E264 source context")
                .clone(),
        )
        .expect("foreign Task35E264 parameter context");
    assert_eq!(
        mizar_core::elaborator::SourcePropertyEqualsSelectorTermSeedProducer::build(
            foreign_parameter,
            equals_identity,
        )
        .expect_err("foreign Task35E264 transaction must fail closed"),
        mizar_core::elaborator::SourcePropertyEqualsSelectorTermSeedError::EnvironmentMismatch
    );
}

fn task264_equals_selector_term_seed_handoff(
    output: &SourcePropertyImplementationRouteOutput,
    symbols: &mizar_resolve::env::SymbolEnv,
) -> mizar_core::elaborator::SourcePropertyEqualsSelectorTermSeedHandoff {
    let checker_owner = output
        .typed_ast
        .source_property_implementation()
        .expect("Task35L264 checker owner")
        .clone();
    let selector_identity = mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(
        symbols,
        checker_owner,
        output
            .typed_ast
            .source_term()
            .expect("Task35L264 primary term")
            .clone(),
        output
            .typed_ast
            .source_structure()
            .expect("Task35L264 structure term")
            .clone(),
    )
    .expect("Task35L264 selector identity");
    let parameter_context =
        mizar_core::elaborator::SourcePropertyParameterCoreContextProducer::build(
            task264_selector_type_handoff(output),
            output
                .typed_ast
                .source_context()
                .expect("Task35L264 source context")
                .clone(),
        )
        .expect("Task35L264 parameter context");
    mizar_core::elaborator::SourcePropertyEqualsSelectorTermSeedProducer::build(
        parameter_context,
        selector_identity,
    )
    .expect("Task35L264 term seeds")
}

#[test]
fn task264_equals_selector_term_lowering_is_exact_and_deterministic() {
    let (ast, module, shells, symbols) = task253_ast_from_source_text(
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
        265_200,
    );
    let output = source_property_implementation_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    )
    .expect("Task35L264 selector")
    .expect("Task35L264 equals route");
    let seed_handoff = task264_equals_selector_term_seed_handoff(&output, &symbols);
    let first =
        mizar_core::elaborator::SourcePropertyEqualsSelectorTermLoweringProducer::build(
            seed_handoff.clone(),
        )
        .expect("Task35L264 lowering");
    let second =
        mizar_core::elaborator::SourcePropertyEqualsSelectorTermLoweringProducer::build(
            seed_handoff.clone(),
        )
        .expect("Task35L264 deterministic replay");

    assert_eq!(first, second);
    assert_eq!(first.seed_handoff(), &seed_handoff);
    assert_eq!(first.source_id(), seed_handoff.source_id());
    assert_eq!(first.module_id(), seed_handoff.module_id());
    assert_eq!(first.definition_owner(), seed_handoff.definition_owner());
    assert_eq!(first.definition_owner().anchor_item().index(), 0);
    assert_eq!(first.definition_owner().item(), None);

    let association = first.association();
    assert_eq!(association.base_seed().index(), 0);
    assert_eq!(association.base_term().index(), 0);
    assert_eq!(association.selector_seed().index(), 1);
    assert_eq!(association.selector_term().index(), 1);
    assert_eq!(association.root_term().index(), 1);

    let base = first
        .terms()
        .get(association.base_term())
        .expect("Task35L264 base term");
    let selector = first
        .terms()
        .get(association.selector_term())
        .expect("Task35L264 selector term");
    assert_eq!(first.terms().len(), 2);
    assert_eq!(
        base.kind,
        mizar_core::core_ir::CoreTermKind::Var(mizar_core::core_ir::CoreVarId::new(0))
    );
    assert_eq!(
        base.source.anchor,
        mizar_core::core_ir::CoreSourceAnchor::SourceRange(mizar_session::SourceRange {
            source_id: first.source_id(),
            start: 173,
            end: 174,
        })
    );
    assert_eq!(base.source.provenance.len(), 1);
    assert_eq!(
        base.source.provenance[0].phase,
        mizar_core::core_ir::CoreProvenancePhase::Checker
    );
    assert_eq!(
        base.source.provenance[0].key.as_str(),
        "source-property-equals-selector-term-seed-v1.base"
    );
    assert_eq!(
        selector.kind,
        mizar_core::core_ir::CoreTermKind::Select {
            selector: seed_handoff
                .selector_identity()
                .association()
                .selector_symbol()
                .clone(),
            base: association.base_term(),
        }
    );
    assert_eq!(
        selector.source.anchor,
        mizar_core::core_ir::CoreSourceAnchor::SourceRange(mizar_session::SourceRange {
            source_id: first.source_id(),
            start: 173,
            end: 182,
        })
    );
    assert_eq!(selector.source.provenance.len(), 1);
    assert_eq!(
        selector.source.provenance[0].phase,
        mizar_core::core_ir::CoreProvenancePhase::Checker
    );
    assert_eq!(
        selector.source.provenance[0].key.as_str(),
        "source-property-equals-selector-term-seed-v1.selector"
    );

    let source_map = first.source_map();
    assert_eq!(source_map.term_sources.len(), 2);
    assert_eq!(
        source_map.term_sources.get(&association.base_term()),
        Some(&base.source)
    );
    assert_eq!(
        source_map.term_sources.get(&association.selector_term()),
        Some(&selector.source)
    );
    assert!(source_map.item_sources.is_empty());
    assert!(source_map.formula_sources.is_empty());
    assert!(source_map.definition_sources.is_empty());
    assert!(source_map.proof_sources.is_empty());
    assert!(source_map.algorithm_sources.is_empty());
    assert!(source_map.generated_sources.is_empty());
    assert!(source_map.obligation_sources.is_empty());
    assert_eq!(
        first.debug_text(),
        format!(
            concat!(
                "source-property-equals-selector-term-lowering-v1|module={}.{}|",
                "owner-anchor=0|property={}|seed=0:1|term=0:1|root=1"
            ),
            first.module_id().package().as_str(),
            first.module_id().path().as_str(),
            first
                .definition_owner()
                .property_symbol()
                .expect("Task35L264 property owner")
                .fqn()
                .as_str(),
        )
    );
    assert!(!first.debug_text().ends_with('\n'));
}

#[test]
fn task264_equals_selector_term_lowerings_are_unattached_and_transaction_local() {
    let (first_ast, first_module, first_shells, first_symbols) =
        task253_ast_from_source_text(SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT, 265_300);
    let first_output = source_property_implementation_output(
        &first_ast,
        first_module,
        &first_shells,
        &first_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    )
    .expect("first Task35L264 selector")
    .expect("first Task35L264 equals route");
    let (second_ast, second_module, second_shells, second_symbols) =
        task253_ast_from_source_text(SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT, 265_301);
    let second_output = source_property_implementation_output(
        &second_ast,
        second_module,
        &second_shells,
        &second_symbols,
        SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
    )
    .expect("second Task35L264 selector")
    .expect("second Task35L264 equals route");

    let first_seed =
        task264_equals_selector_term_seed_handoff(&first_output, &first_symbols);
    let second_seed =
        task264_equals_selector_term_seed_handoff(&second_output, &second_symbols);
    let first =
        mizar_core::elaborator::SourcePropertyEqualsSelectorTermLoweringProducer::build(
            first_seed,
        )
        .expect("first Task35L264 lowering");
    let second =
        mizar_core::elaborator::SourcePropertyEqualsSelectorTermLoweringProducer::build(
            second_seed,
        )
        .expect("second Task35L264 lowering");

    assert_ne!(first.module_id(), second.module_id());
    assert_ne!(first.seed_handoff(), second.seed_handoff());
    assert_ne!(first.terms(), second.terms());
    assert!(!std::ptr::eq(first.terms(), second.terms()));
    assert!(!std::ptr::eq(first.source_map(), second.source_map()));
    assert_ne!(
        first
            .seed_handoff()
            .selector_identity()
            .association()
            .selector_symbol(),
        second
            .seed_handoff()
            .selector_identity()
            .association()
            .selector_symbol()
    );
    for lowering in [&first, &second] {
        assert_eq!(lowering.terms().len(), 2);
        assert_eq!(lowering.association().base_term().index(), 0);
        assert_eq!(lowering.association().selector_term().index(), 1);
        assert_eq!(lowering.association().root_term().index(), 1);
        assert_eq!(lowering.source_map().term_sources.len(), 2);
        assert!(lowering.definition_owner().item().is_none());
        assert!(lowering.source_map().item_sources.is_empty());
        assert!(lowering.source_map().definition_sources.is_empty());
        assert!(lowering.source_map().formula_sources.is_empty());
        assert!(lowering.source_map().generated_sources.is_empty());
        assert!(lowering.source_map().obligation_sources.is_empty());
        assert!(lowering.source_map().term_sources.values().all(|source| {
            matches!(
                source.anchor,
                mizar_core::core_ir::CoreSourceAnchor::SourceRange(range)
                    if range.source_id == lowering.source_id()
            )
        }));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task264CarrierCoreContextMutation {
    Baseline,
    MissingItem,
    ExtraItem,
    WrongKind,
    WrongVisibility,
    WrongSymbol,
    WrongSource,
    WrongProvenance,
    WrongBoundaryProvenance,
    MissingBoundary,
    WrongBoundary,
    UnexpectedDependency,
    UnexpectedGeneratedOrigin,
    UnexpectedCheckerSite,
    UnexpectedBinder,
}

fn task264_carrier_core_context(
    checker_owner: &mizar_checker::source_property_implementation::SourcePropertyImplementationHandoff,
) -> mizar_core::elaborator::CoreContext {
    task264_carrier_core_context_with_mutation(
        checker_owner,
        Task264CarrierCoreContextMutation::Baseline,
    )
}

fn task264_carrier_core_handoff(
    checker_owner: &mizar_checker::source_property_implementation::SourcePropertyImplementationHandoff,
) -> mizar_core::elaborator::SourcePropertyCarrierCoreContextHandoff {
    mizar_core::elaborator::SourcePropertyCarrierCoreContextProducer::build(
        task264_carrier_core_context(checker_owner),
        checker_owner.clone(),
    )
    .expect("Task264 carrier Core handoff")
}

fn task264_selector_type_handoff(
    output: &SourcePropertyImplementationRouteOutput,
) -> mizar_core::elaborator::SourcePropertySelectorTypeContextHandoff {
    let checker_owner = output
        .typed_ast
        .source_property_implementation()
        .expect("Task264 checker owner")
        .clone();
    let source_type = output
        .typed_ast
        .source_type()
        .expect("Task264 source type")
        .clone();
    mizar_core::elaborator::SourcePropertySelectorTypeContextProducer::build(
        task264_carrier_core_handoff(&checker_owner),
        source_type,
    )
    .expect("Task264 selector/type context")
}

fn task264_carrier_core_context_with_mutation(
    checker_owner: &mizar_checker::source_property_implementation::SourcePropertyImplementationHandoff,
    mutation: Task264CarrierCoreContextMutation,
) -> mizar_core::elaborator::CoreContext {
    let identity = checker_owner.carrier_identity();
    let structure_range = match identity.structure_origin().anchor() {
        mizar_session::SourceAnchor::Range(range) => *range,
        _ => panic!("Task264 carrier structure origin must have a source range"),
    };
    let summary = mizar_core::elaborator::ResolvedTypedAstSummary::new(
        checker_owner.source_id(),
        checker_owner.module_id().clone(),
    );
    let summary = if mutation == Task264CarrierCoreContextMutation::UnexpectedCheckerSite {
        summary.with_checker_sites(vec![
            mizar_core::elaborator::CheckerSiteSummary::failed_overload(
                mizar_checker::resolved_typed_ast::OverloadResolutionId::new(0),
                mizar_core::core_ir::CoreSourceRef::direct(structure_range),
            ),
        ])
    } else {
        summary
    };
    let mut input = mizar_core::elaborator::CoreContextInput::new(summary);
    if mutation != Task264CarrierCoreContextMutation::MissingItem {
        let provenance_key =
            if mutation == Task264CarrierCoreContextMutation::WrongProvenance {
                "wrong-task264-carrier-provenance"
            } else {
                "source-property-carrier-core-item-v1.structure"
            };
        let boundary_provenance_key =
            if mutation == Task264CarrierCoreContextMutation::WrongBoundaryProvenance {
                "wrong-task264-carrier-boundary-provenance"
            } else {
                "source-property-carrier-core-item-v1.structure"
            };
        let source_range = if mutation == Task264CarrierCoreContextMutation::WrongSource {
            mizar_session::SourceRange {
                source_id: structure_range.source_id,
                start: structure_range.start + 1,
                end: structure_range.end,
            }
        } else {
            structure_range
        };
        let source = mizar_core::core_ir::CoreSourceRef::direct(source_range).with_provenance(vec![
            mizar_core::core_ir::CoreProvenance::new(
                mizar_core::core_ir::CoreProvenancePhase::Checker,
                provenance_key,
            ),
        ]);
        let symbol = if mutation == Task264CarrierCoreContextMutation::WrongSymbol {
            task264_carrier_extra_symbol(checker_owner, "task264-carrier-wrong")
        } else {
            identity.structure_symbol().clone()
        };
        let kind = if mutation == Task264CarrierCoreContextMutation::WrongKind {
            mizar_core::core_ir::CoreItemKind::Mode
        } else {
            mizar_core::core_ir::CoreItemKind::Structure
        };
        let visibility = if mutation == Task264CarrierCoreContextMutation::WrongVisibility {
            "private"
        } else {
            "public"
        };
        let mut seed = mizar_core::elaborator::CoreItemSeed::new(
            symbol,
            kind,
            visibility,
            source,
            mizar_core::elaborator::CheckerOwnedProvenance::checker(
                boundary_provenance_key,
            ),
        );
        if mutation != Task264CarrierCoreContextMutation::MissingBoundary {
            seed = seed.with_definition_boundary(
                if mutation == Task264CarrierCoreContextMutation::WrongBoundary {
                    mizar_core::elaborator::DefinitionBoundaryKind::Theorem
                } else {
                    mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem
                },
            );
        }
        if mutation == Task264CarrierCoreContextMutation::UnexpectedDependency {
            seed = seed.with_dependencies(vec![task264_carrier_extra_symbol(
                checker_owner,
                "task264-carrier-missing",
            )]);
        }
        input.item_seeds.push(seed);
    }
    if mutation == Task264CarrierCoreContextMutation::ExtraItem {
        input.item_seeds.push(
            mizar_core::elaborator::CoreItemSeed::new(
                task264_carrier_extra_symbol(checker_owner, "task264-carrier-extra"),
                mizar_core::core_ir::CoreItemKind::Structure,
                "public",
                mizar_core::core_ir::CoreSourceRef::direct(structure_range)
                .with_provenance(vec![mizar_core::core_ir::CoreProvenance::new(
                    mizar_core::core_ir::CoreProvenancePhase::Checker,
                    "source-property-carrier-core-item-v1.extra",
                )]),
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "source-property-carrier-core-item-v1.extra",
                ),
            )
            .with_definition_boundary(
                mizar_core::elaborator::DefinitionBoundaryKind::DefinitionalItem,
            ),
        );
    }
    if mutation == Task264CarrierCoreContextMutation::UnexpectedGeneratedOrigin {
        input.generated_origin_seeds.push(
            mizar_core::elaborator::GeneratedOriginSeed::new(
                identity.structure_symbol().clone(),
                mizar_core::core_ir::GeneratedOriginKind::StableChoice,
                "task264-carrier-unexpected-generated-origin",
                mizar_core::core_ir::CoreSourceRef::direct(structure_range),
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "task264-carrier-unexpected-generated-origin",
                ),
            ),
        );
    }
    if mutation == Task264CarrierCoreContextMutation::UnexpectedBinder {
        let var = mizar_core::core_ir::CoreVarId::new(0);
        input
            .variable_seeds
            .push(mizar_core::elaborator::CoreVariableSeed::new(
                var,
                mizar_core::binder_normalization::NormalizedVarClass::Free,
                "task264-carrier-unexpected-binder",
                mizar_core::binder_normalization::NormalizedVarSort::Term,
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "task264-carrier-unexpected-binder",
                ),
            ));
        input
            .binder_seeds
            .push(mizar_core::elaborator::CoreBinderSeed::new(
                var,
                mizar_core::core_ir::CoreSourceRef::direct(structure_range),
                mizar_core::elaborator::CheckerOwnedProvenance::checker(
                    "task264-carrier-unexpected-binder",
                ),
            ));
    }
    mizar_core::elaborator::prepare_core_context(input)
        .expect("Task264 carrier Core context seed should prepare")
}

fn task264_carrier_extra_symbol(
    checker_owner: &mizar_checker::source_property_implementation::SourcePropertyImplementationHandoff,
    local: &str,
) -> mizar_resolve::resolved_ast::SymbolId {
    mizar_resolve::resolved_ast::SymbolId::new(
        checker_owner.module_id().clone(),
        mizar_resolve::resolved_ast::LocalSymbolId::new(local),
        mizar_resolve::resolved_ast::FullyQualifiedName::new(format!(
            "{}.{}",
            checker_owner.module_id().path().as_str(),
            local,
        )),
    )
}

#[test]
fn task264_native_layout_lexical_stream_and_surface_topology_are_equivalent() {
    let mut executed = 0;
    for (profile, original, node_count, token_count, parameter) in [
        (
            "means",
            SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
            85,
            53,
            65,
        ),
        (
            "equals",
            SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
            56,
            35,
            47,
        ),
    ] {
        let baseline = task257c4c1_frontend_output(original, 264_950 + executed);
        assert!(baseline.diagnostics.is_empty(), "{profile} baseline");
        let baseline_ast = baseline.ast.as_ref().expect("native baseline AST");
        assert_eq!(baseline_ast.nodes().len(), node_count);
        assert_eq!(
            baseline_ast.root().map(|id| id.index()),
            Some(node_count - 1)
        );
        let mut sources = vec![("original", original.to_owned())];
        sources.extend(
            metamorphic_edits::variants(baseline_ast, original)
                .into_iter()
                .filter(|(kind, _)| matches!(*kind, "leading_line" | "intertoken_layout"))
                .map(|(kind, edits)| (kind, metamorphic_edits::apply(original, &edits))),
        );
        assert_eq!(sources.len(), 3);
        assert_eq!(
            (sources[1].0, sources[2].0),
            ("leading_line", "intertoken_layout")
        );
        sources.push((
            "missing_final_lf",
            original.strip_suffix('\n').unwrap().to_owned(),
        ));
        sources.push(("appended_space", format!("{original} ")));
        let compact = original
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace(" -> ", "->")
            .replace("; ", ";")
            .replace("steps: 1", "steps:1");
        let compact = if profile == "means" {
            compact.replace("it = it", "it::=λ=::=it")
        } else {
            compact.replace("equals M.carrier", "equals M . ::=λ=::carrier")
        };
        assert!(compact.contains("::=λ=::"));
        sources.push(("compact_unicode_comment", compact));
        assert_eq!(sources.len(), 6);
        for (layout, source) in sources {
            let frontend = task257c4c1_frontend_output(&source, 264_960 + executed);
            assert!(frontend.diagnostics.is_empty(), "{profile}/{layout}");
            assert!(
                frontend.preprocessed.diagnostics.is_empty(),
                "{profile}/{layout}"
            );
            assert!(frontend.tokens.diagnostics.is_empty(), "{profile}/{layout}");
            assert!(frontend.preprocessed.import_stubs.is_empty());
            assert!(frontend.preprocessed.doc_comments.is_empty());
            assert_eq!(frontend.source.source_text.as_ref(), source);
            assert_eq!(frontend.source.package_id, baseline.source.package_id);
            assert_eq!(frontend.source.edition, baseline.source.edition);
            let ast = frontend.ast.as_ref().expect("native layout AST");
            assert_eq!(ast.source_id, frontend.source.source_id);
            assert_eq!(frontend.tokens.source_id, ast.source_id);
            assert_eq!(frontend.preprocessed.source_id, ast.source_id);
            assert_eq!(frontend.preprocessed.source_map.source_id, ast.source_id);
            assert!(ast.canonical_bytes().is_some(), "{profile}/{layout}");
            assert!(ast.node_views().all(|node| !node.is_recovered()));
            assert!(ast.expression_root().is_none());
            assert_eq!(ast.nodes().len(), node_count);
            assert_eq!(ast.root().map(|id| id.index()), Some(node_count - 1));
            assert_eq!(ast.token_nodes().len(), token_count);
            for (id, (node, expected)) in ast.nodes().iter().zip(baseline_ast.nodes()).enumerate() {
                assert_eq!(
                    (&node.kind, &node.children),
                    (&expected.kind, &expected.children),
                    "{profile}/{layout}/node{id}"
                );
                assert!(
                    node.range.start < node.range.end
                        && source.get(node.range.start..node.range.end).is_some()
                );
                for child in &node.children {
                    let child = ast.node(*child).unwrap();
                    assert!(
                        node.range.start <= child.range.start && child.range.end <= node.range.end
                    );
                }
                if let (Some(first), Some(last)) = (node.children.first(), node.children.last()) {
                    assert_eq!(node.range.start, ast.node(*first).unwrap().range.start);
                    assert_eq!(node.range.end, ast.node(*last).unwrap().range.end);
                    if id != node_count - 1 {
                        assert!(
                            node.children.windows(2).all(|pair| ast
                                .node(pair[0])
                                .unwrap()
                                .range
                                .end
                                <= ast.node(pair[1]).unwrap().range.start)
                        );
                    }
                }
            }
            let root = ast.root_view().unwrap();
            let (compilation, aliases) = ast
                .node(ast.root().unwrap())
                .unwrap()
                .children
                .split_last()
                .unwrap();
            assert_eq!(aliases, ast.token_nodes());
            assert_eq!(root.range(), ast.node(*compilation).unwrap().range);
            assert_eq!(
                ast.node(*compilation).unwrap().kind,
                SurfaceNodeKind::CompilationUnit
            );
            let parameter = &ast.nodes()[parameter];
            let declaration = ast.node(ast.token_nodes()[20]).unwrap();
            assert_eq!(parameter.kind, SurfaceNodeKind::DefinitionParameter);
            assert_eq!(
                parameter.range.start,
                ast.node(ast.token_nodes()[19]).unwrap().range.start
            );
            assert_eq!(
                parameter.range.end,
                ast.node(ast.token_nodes()[23]).unwrap().range.end
            );
            assert_eq!(parameter.children[1], ast.token_nodes()[20]);
            assert_ne!(parameter.range, declaration.range);
            assert_eq!(
                source.get(declaration.range.start..declaration.range.end),
                Some("M")
            );
            let native = frontend.tokens.tokens();
            assert_eq!(native.len(), token_count);
            for (index, kind, text) in [
                (2, mizar_lexer::TokenKind::Identifier, "Task264Carrier"),
                (22, mizar_lexer::TokenKind::UserSymbol, "Task264Carrier"),
            ] {
                assert_eq!(
                    (native[index].kind, native[index].text.as_ref()),
                    (kind, text)
                );
            }
            if profile == "means" {
                for index in [39, 48] {
                    assert_eq!(
                        (native[index].kind, native[index].text.as_ref()),
                        (mizar_lexer::TokenKind::Numeral, "1")
                    );
                }
            }
            let preprocessed = mizar_lexer::preprocess_source_for_lexing(&source);
            assert!(preprocessed.diagnostics.is_empty());
            assert_eq!(
                preprocessed.lexical_text,
                frontend.preprocessed.lexical_text.as_str()
            );
            assert_eq!(
                preprocessed.preprocess_map,
                frontend.preprocessed.source_map.preprocess_map
            );
            if layout == "compact_unicode_comment" {
                assert!(preprocessed.comments.iter().any(|comment| comment.kind
                    == mizar_lexer::CommentKind::MultiLine
                    && comment.lexeme.contains('λ')));
            }
            let raw = mizar_lexer::scan_raw(&preprocessed.lexical_text).expect("strict raw scan");
            let environment = mizar_lexer::build_lexical_environment(&[], &[]).unwrap();
            let locals = mizar_lexer::collect_local_lexical_declarations(
                &raw,
                mizar_lexer::ModuleId::new(frontend.source.module_path.as_str()),
            );
            assert_eq!(&locals, frontend.tokens.local_declarations());
            let scopes = mizar_lexer::build_scope_skeleton(&raw);
            assert!(scopes.diagnostics.is_empty());
            let manual = mizar_lexer::disambiguate_with_local_declarations(
                &raw,
                &environment,
                &locals,
                &mizar_lexer::ParserLexContext::general(),
                &scopes,
            );
            assert!(manual.diagnostics().is_empty());
            assert_eq!(manual.tokens().len(), native.len());
            for ((token, native), surface) in
                manual.tokens().iter().zip(native).zip(ast.token_views())
            {
                let mapped = preprocessed
                    .preprocess_map
                    .source_ranges_for_lexical(token.span)
                    .unwrap();
                assert_eq!(mapped.len(), 1, "{profile}/{layout}: fragmented token");
                let span = SourceRange {
                    source_id: frontend.source.source_id,
                    start: mapped[0].start,
                    end: mapped[0].end,
                };
                assert_eq!(
                    (token.kind, token.lexeme.as_str(), span),
                    (native.kind, native.text.as_ref(), native.span),
                    "{profile}/{layout}"
                );
                let kind = match token.kind {
                    mizar_lexer::TokenKind::ReservedWord => SurfaceTokenKind::ReservedWord,
                    mizar_lexer::TokenKind::ReservedSymbol => SurfaceTokenKind::ReservedSymbol,
                    mizar_lexer::TokenKind::Identifier => SurfaceTokenKind::Identifier,
                    mizar_lexer::TokenKind::UserSymbol => SurfaceTokenKind::UserSymbol,
                    mizar_lexer::TokenKind::Numeral => SurfaceTokenKind::Numeral,
                    _ => panic!("unexpected closed-profile token kind"),
                };
                let surface_token = surface.as_token().unwrap();
                assert_eq!(
                    (
                        surface_token.kind,
                        surface_token.text.as_ref(),
                        surface.range()
                    ),
                    (kind, token.lexeme.as_str(), span)
                );
                assert_eq!(
                    source.get(span.start..span.end),
                    Some(token.lexeme.as_str())
                );
            }
            executed += 1;
        }
    }
    assert_eq!(executed, 12);
}

#[test]
fn task264_raw_fixed_prefix_body_bridge_preserves_transport_and_selected_errors() {
    use mizar_resolve::env::{NamespaceNodeKind, SymbolEnvIndexes};
    let mut positives = Vec::new();
    let mut refusals = Vec::new();
    for (profile, original, owner, parameter, definiens) in [
        (
            "means",
            SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
            81,
            65,
            72,
        ),
        (
            "equals",
            SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT,
            52,
            47,
            51,
        ),
    ] {
        let ordinal = if profile == "means" { 265_000 } else { 265_100 };
        let (baseline_ast, baseline_module, baseline_shells, baseline_env, diagnostics) =
            task253_ast_from_source_text_with_diagnostic_count(original, ordinal);
        assert_eq!(diagnostics, 0);
        source_property_implementation_output(
            &baseline_ast,
            baseline_module.clone(),
            &baseline_shells,
            &baseline_env,
            original,
        )
        .expect("original selected profile")
        .expect("original public route");
        let layout = original
            .replacen(
                "definition\n  let M be Task264Carrier;",
                "definition\n   let  M be   Task264Carrier;",
                1,
            )
            .replacen("\n  property M.", "\n\t\t\tproperty M.", 1)
            .replacen("\n  existence", "\n     existence", 1)
            .replacen("\n  uniqueness", "\n\tuniqueness", 1);
        let layout = if profile == "means" {
            layout
                .replace("it = it", "it=it")
                .replace("steps: 1", "steps:1")
        } else {
            layout
                .replace("M.marker", "M . marker")
                .replace("M.carrier", "M . carrier")
        };
        let comments = original
            .replacen("definition\n  let", "definition\n::=λ body=::  let", 1)
            .replacen("\n  property M.", " :: ordinary body\n  property M.", 1);
        let comments = if profile == "means" {
            comments.replace("it = it", "it::=λ=::= it")
        } else {
            comments.replace("M.carrier", "M::=λ=::.carrier")
        };
        let comments = format!("{comments}:: ordinary suffix\n");
        for (index, (case, source)) in [("layout", layout), ("comments", comments)]
            .into_iter()
            .enumerate()
        {
            let frontend = task257c4c1_frontend_output(&source, ordinal + index + 1);
            assert!(frontend.diagnostics.is_empty(), "{profile}/{case}");
            assert!(frontend.preprocessed.diagnostics.is_empty());
            assert!(frontend.tokens.diagnostics.is_empty());
            assert_eq!(frontend.source.source_text.as_ref(), source);
            let ast = frontend.ast.as_ref().expect("actual disk Frontend AST");
            assert_eq!(ast.source_id, frontend.source.source_id);
            assert_eq!(frontend.preprocessed.source_id, ast.source_id);
            assert_eq!(frontend.tokens.source_id, ast.source_id);
            assert!(frontend.preprocessed.import_stubs.is_empty());
            assert!(frontend.preprocessed.doc_comments.is_empty());
            let module = mizar_resolve::resolved_ast::ModuleId::new(
                frontend.source.package_id.clone(),
                frontend.source.module_path.clone(),
            );
            let shells =
                mizar_resolve::declarations::DeclarationShellCollector::new(ast, &module).collect();
            let collected = mizar_resolve::symbols::SignatureProjectionExtractor::new(
                ast,
                &shells,
                mizar_resolve::env::NamespacePath::new(module.path().as_str()),
            )
            .collect(&module);
            assert!(
                collected.diagnostics().is_empty(),
                "actual resolver prerequisite"
            );
            let env = collected.into_env();
            assert_eq!(
                (
                    shells.declarations().len(),
                    env.symbols().len(),
                    env.definitions().len(),
                    env.contributions().len()
                ),
                (5, 3, 3, 1)
            );
            assert_eq!(
                ast.node(ast.token_nodes()[18]).unwrap().range,
                SourceRange {
                    source_id: ast.source_id,
                    start: 108,
                    end: 118
                }
            );
            let result =
                source_property_implementation_output(ast, module.clone(), &shells, &env, &source);
            if case == "layout" {
                refusals.push((
                    profile,
                    "selected stale body",
                    source_property_implementation_output(
                        &baseline_ast,
                        baseline_module.clone(),
                        &baseline_shells,
                        &baseline_env,
                        &source,
                    )
                    .is_some_and(|result| result.is_err()),
                ));
                for fault in ["token_kind", "wrapper_child", "child_order"] {
                    let mut builder = mizar_syntax::SurfaceAstBuilder::new(ast.source_id);
                    let mut nodes = Vec::new();
                    for (node_index, node) in ast.nodes().iter().enumerate() {
                        let mut children = node
                            .children
                            .iter()
                            .map(|child| nodes[child.index()])
                            .collect::<Vec<_>>();
                        if fault == "wrapper_child" && node_index == definiens {
                            children = vec![nodes[29]];
                        }
                        if fault == "child_order" && node_index == owner {
                            children.swap(0, 1);
                        }
                        let id = match &node.kind {
                            SurfaceNodeKind::Token(token) => builder.add_token(
                                if fault == "token_kind" && node_index == 22 {
                                    SurfaceTokenKind::Identifier
                                } else {
                                    token.kind
                                },
                                token.text.as_ref(),
                                node.range,
                            ),
                            kind => builder.add_node(kind.clone(), node.range, children),
                        };
                        nodes.push(id);
                    }
                    let changed = builder.finish(
                        ast.root().map(|root| nodes[root.index()]),
                        ast.expression_root().map(|root| nodes[root.index()]),
                    );
                    refusals.push((
                        profile,
                        fault,
                        source_property_implementation_output(
                            &changed,
                            module.clone(),
                            &shells,
                            &env,
                            &source,
                        )
                        .is_some_and(|result| result.is_err()),
                    ));
                }
                let mut indexes = SymbolEnvIndexes {
                    imports: env.imports().clone(),
                    exports: env.exports().clone(),
                    symbols: env.symbols().clone(),
                    labels: env.labels().clone(),
                    definitions: env.definitions().clone(),
                    overloads: env.overloads().clone(),
                    registrations: env.registrations().clone(),
                    lexical_summaries: env.lexical_summaries().clone(),
                    namespace_graph: env.namespace_graph().clone(),
                    declaration_dependencies: env.declaration_dependencies().clone(),
                    contributions: env.contributions().clone(),
                    module_summaries: env.module_summaries().clone(),
                };
                indexes.namespace_graph.insert_node(
                    NamespaceNodeKind::Segment,
                    Some(module.clone()),
                    "unused_property_body",
                    env.contributions().iter().next().unwrap().id(),
                );
                let altered = mizar_resolve::env::SymbolEnv::new(module.clone(), indexes);
                assert_eq!(altered.symbols(), env.symbols());
                assert_eq!(altered.definitions(), env.definitions());
                assert_eq!(altered.contributions(), env.contributions());
                assert_ne!(altered.namespace_graph(), env.namespace_graph());
                refusals.push((
                    profile,
                    "full-env-only metadata",
                    source_property_implementation_output(
                        ast,
                        module.clone(),
                        &shells,
                        &altered,
                        &source,
                    )
                    .is_some_and(|result| result.is_err()),
                ));
                let foreign_root = std::env::temp_dir().join(format!(
                    "mizar-property-body-foreign-{}-{ordinal}",
                    std::process::id()
                ));
                let foreign_path = foreign_root.join("src").join("foreign.miz");
                std::fs::create_dir_all(foreign_path.parent().expect("foreign source parent"))
                    .expect("foreign source root");
                std::fs::write(&foreign_path, &source).expect("real foreign source");
                let ids = mizar_session::InMemorySessionIdAllocator::new();
                let snapshot = super::shared::snapshot_id(26_500 + ordinal);
                mizar_session::SessionIdAllocator::next_source_id(&ids, snapshot)
                    .expect("advance real source allocator");
                let foreign = mizar_frontend::orchestration::Frontend::new(
                    mizar_frontend::source::FrontendSourceLoader::new(
                        mizar_session::DiskSourceLoader::new(&foreign_root),
                    ),
                    ParseOnlyImportProvider,
                    mizar_frontend::parsing::MizarParserSeam,
                )
                .run(
                    mizar_frontend::source::SourceUnitRequest {
                        snapshot,
                        input: mizar_session::SourceInput {
                            package_id: frontend.source.package_id.clone(),
                            module_path: frontend.source.module_path.clone(),
                            normalized_path: mizar_session::normalize_path(
                                &foreign_root,
                                &foreign_path,
                            )
                            .expect("foreign normalized path"),
                            edition: frontend.source.edition.clone(),
                            origin: mizar_session::SourceOriginInput::Disk { path: foreign_path },
                        },
                    },
                    &ids,
                )
                .expect("actual Frontend with a distinct allocated source ID");
                std::fs::remove_dir_all(&foreign_root).expect("clean foreign source");
                assert!(foreign.diagnostics.is_empty());
                let foreign_ast = foreign.ast.as_ref().unwrap();
                assert_ne!(foreign_ast.source_id, ast.source_id);
                refusals.push((
                    profile,
                    "mixed source IDs",
                    source_property_implementation_output(
                        foreign_ast,
                        module.clone(),
                        &shells,
                        &env,
                        &source,
                    )
                    .is_some_and(|result| result.is_err()),
                ));
                assert_ne!(module, baseline_module, "mixed-module premise");
                refusals.push((
                    profile,
                    "mixed module",
                    source_property_implementation_output(
                        ast,
                        baseline_module.clone(),
                        &shells,
                        &env,
                        &source,
                    )
                    .is_some_and(|result| result.is_err()),
                ));
            }
            positives.push((
                profile,
                case,
                source,
                ast.clone(),
                module,
                shells,
                env,
                result,
                owner,
                parameter,
                definiens,
            ));
        }
        let documentation = original.replacen(
            "definition\n  let",
            "definition\n::: unsupported body metadata\n  let",
            1,
        );
        refusals.push((
            profile,
            "unsupported documentation before AST association",
            source_property_implementation_output(
                &baseline_ast,
                baseline_module.clone(),
                &baseline_shells,
                &baseline_env,
                &documentation,
            )
            .is_none(),
        ));
        for (guard_index, (guard, edits)) in metamorphic_edits::variants(&baseline_ast, original)
            .into_iter()
            .filter(|(kind, _)| matches!(*kind, "leading_line" | "intertoken_layout"))
            .enumerate()
        {
            let source = metamorphic_edits::apply(original, &edits);
            let (ast, module, shells, env, diagnostics) =
                task253_ast_from_source_text_with_diagnostic_count(
                    &source,
                    ordinal + 20 + guard_index,
                );
            assert_eq!(diagnostics, 0);
            refusals.push((
                profile,
                guard,
                source_property_implementation_output(&ast, module, &shells, &env, &source)
                    .is_some_and(|result| result.is_ok()),
            ));
        }
    }
    assert_eq!(positives.len(), 4);
    assert_eq!(refusals.len(), 20);
    let failures = positives
        .iter()
        .filter_map(
            |(profile, case, _, _, _, _, _, result, _, _, _)| match result {
                None => Some(format!("{profile}/{case}: None")),
                Some(Err(error)) => Some(format!("{profile}/{case}: {error}")),
                Some(Ok(_)) => None,
            },
        )
        .collect::<Vec<_>>();
    assert!(
        failures.is_empty(),
        "raw body route positives: {failures:?}"
    );
    for (profile, case, refused) in refusals {
        assert!(refused, "{profile}/{case}");
    }
    for (profile, case, source, ast, module, shells, env, result, owner, parameter, definiens) in
        positives
    {
        let output = result.expect("all selected").expect("all successful");
        let typed = &output.typed_ast;
        let resolved = &output.resolved;
        let context = typed.source_context().unwrap();
        let types = typed.source_type().unwrap();
        let terms = typed.source_term().unwrap();
        let property = typed.source_property_implementation().unwrap();
        let tokens = ast.token_nodes();
        let declaration = ast.node(tokens[20]).unwrap().range;
        let written_type = ast.node(tokens[22]).unwrap().range;
        let item = property.implementations().iter().next().unwrap().1;
        let param = property.parameters().iter().next().unwrap().1;
        let target = property.targets().iter().next().unwrap().1;
        let body = property.definientia().iter().next().unwrap().1;
        let context_declaration = context.declarations().iter().next().unwrap().1;
        assert_eq!(context_declaration.declaration_range, declaration);
        assert_eq!(context_declaration.written_type_range, written_type);
        assert_eq!(
            types.expressions().iter().next().unwrap().1.source_range(),
            written_type
        );
        assert_eq!(item.source_range(), ast.nodes()[owner].range);
        assert_eq!(param.source_range(), ast.nodes()[parameter].range);
        assert_eq!(param.declaration_range(), declaration);
        assert_eq!(
            typed
                .nodes()
                .node(mizar_checker::typed_ast::TypedNodeId::new(parameter))
                .unwrap()
                .anchor,
            SourceAnchor::Range(declaration)
        );
        assert_eq!(target.subject_range(), ast.node(tokens[25]).unwrap().range);
        assert_eq!(target.name_range(), ast.node(tokens[27]).unwrap().range);
        assert_eq!(target.source_range().start, target.subject_range().start);
        assert_eq!(target.source_range().end, target.name_range().end);
        assert_eq!(body.source_range(), ast.nodes()[definiens].range);
        for (spelling, span) in [
            (item.spelling(), item.source_range()),
            (param.spelling(), param.source_range()),
            (target.spelling(), target.source_range()),
            (body.spelling(), body.source_range()),
        ] {
            assert_eq!(Some(spelling), source.get(span.start..span.end));
        }
        for (index, (_, term)) in terms.terms().iter().enumerate() {
            assert_eq!(
                term.source_range(),
                ast.node(tokens[if index == 0 { 29 } else { 31 }])
                    .unwrap()
                    .range
            );
        }
        if profile == "means" {
            let atomic = typed.source_atomic_formula().unwrap();
            let formula = atomic.formulas().iter().next().unwrap().1;
            assert_eq!(formula.source_range(), body.source_range());
            assert_eq!(formula.spelling(), "it = it");
            assert_eq!(atomic.primary_term_fingerprint(), terms.debug_text());
            if case == "layout" {
                assert_eq!(body.spelling(), "it=it");
            }
            for (index, (_, row)) in property.correctness().iter().enumerate() {
                assert_eq!(
                    row.source_range(),
                    ast.nodes()[if index == 0 { 76 } else { 80 }].range
                );
                assert_eq!(
                    row.justification(),
                    &SourceAnchor::Range(ast.nodes()[if index == 0 { 75 } else { 79 }].range)
                );
                assert_eq!(
                    typed
                        .initial_obligations()
                        .get(row.obligation())
                        .unwrap()
                        .status,
                    mizar_checker::typed_ast::InitialObligationStatus::Pending
                );
            }
        } else {
            let structures = typed.source_structure().unwrap();
            let selector = structures.terms().iter().next().unwrap().1;
            assert_eq!(selector.source_range(), body.source_range());
            assert_eq!(selector.spelling(), "M.carrier");
            assert_eq!(structures.primary_term_fingerprint(), terms.debug_text());
            if case == "layout" {
                assert_eq!(body.spelling(), "M . carrier");
                assert_eq!(target.spelling(), "M . marker");
            }
            let consumer = mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(&env, property.clone(), terms.clone(), structures.clone()).expect("actual raw Equals consumer");
            assert_eq!(consumer.property(), property);
        }
        assert_eq!(
            typed.initial_obligations().len(),
            if profile == "means" { 2 } else { 0 }
        );
        assert_eq!(resolved.source_context(), typed.source_context());
        assert_eq!(resolved.source_type(), typed.source_type());
        assert_eq!(resolved.source_term(), typed.source_term());
        assert_eq!(resolved.source_structure(), typed.source_structure());
        assert_eq!(
            resolved.source_atomic_formula(),
            typed.source_atomic_formula()
        );
        assert_eq!(resolved.source_property_implementation(), Some(property));
        let replay = source_property_implementation_output(&ast, module, &shells, &env, &source)
            .unwrap()
            .expect("actual full route replay");
        assert_eq!(replay.typed_ast, output.typed_ast);
        assert_eq!(replay.resolved, output.resolved);
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.diagnostics().is_empty());
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.diagnostics().is_empty());
    }
}

#[test]
fn task264_raw_prefix_bridge_preserves_transport_and_selected_errors() {
    let mut positives = Vec::new();
    let mut refusals = Vec::new();
    for (ordinal, (profile, original, prefix, parameter)) in [
        ("means", SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT, [61, 60, 56, 59, 81], 65),
        ("equals", SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT, [43, 42, 38, 41, 52], 47),
    ].into_iter().enumerate() {
        let baseline = task257c4c1_frontend_output(original, 265_200 + ordinal * 10);
        let variants = metamorphic_edits::variants(baseline.ast.as_ref().unwrap(), original)
            .into_iter()
            .filter(|(kind, _)| matches!(*kind, "leading_line" | "intertoken_layout"))
            .enumerate()
            .map(|(index, (case, edits))| {
                let source = metamorphic_edits::apply(original, &edits);
                let frontend = task257c4c1_frontend_output(&source, 265_201 + ordinal * 10 + index);
                (case, source, frontend)
            }).collect::<Vec<_>>();
        assert_eq!(variants.len(), 2);
        let mut baseline_artifacts = None;
        for (case, source, frontend) in std::iter::once(("original", original.to_owned(), baseline)).chain(variants) {
            assert!(frontend.diagnostics.is_empty(), "{profile}/{case}");
            assert!(frontend.preprocessed.diagnostics.is_empty());
            assert!(frontend.tokens.diagnostics.is_empty());
            assert!(frontend.preprocessed.import_stubs.is_empty());
            assert!(frontend.preprocessed.doc_comments.is_empty());
            assert_eq!(frontend.source.source_text.as_ref(), source);
            let ast = frontend.ast.as_ref().expect("actual disk Frontend AST");
            assert_eq!(ast.source_id, frontend.source.source_id);
            assert_eq!(frontend.preprocessed.source_id, ast.source_id);
            assert_eq!(frontend.preprocessed.source_map.source_id, ast.source_id);
            assert_eq!(frontend.tokens.source_id, ast.source_id);
            assert!(ast.canonical_bytes().is_some());
            assert!(ast.node_views().all(|node| !node.is_recovered()));
            let preprocessed = mizar_lexer::preprocess_source_for_lexing(&source);
            assert_eq!(preprocessed.lexical_text, frontend.preprocessed.lexical_text.as_str());
            assert_eq!(preprocessed.preprocess_map, frontend.preprocessed.source_map.preprocess_map);
            assert_eq!(ast.token_nodes().len(), frontend.tokens.tokens().len());
            for (surface, token) in ast.token_views().zip(frontend.tokens.tokens()) {
                assert_eq!(surface.range(), token.span);
                assert_eq!(source.get(token.span.start..token.span.end), Some(surface.as_token().unwrap().text.as_ref()));
                assert_eq!(surface.as_token().unwrap().text.as_ref(), token.text.as_ref());
            }
            let module = mizar_resolve::resolved_ast::ModuleId::new(
                frontend.source.package_id.clone(), frontend.source.module_path.clone());
            let shells = mizar_resolve::declarations::DeclarationShellCollector::new(ast, &module).collect();
            let collected = mizar_resolve::symbols::SignatureProjectionExtractor::new(
                ast, &shells, mizar_resolve::env::NamespacePath::new(module.path().as_str())).collect(&module);
            assert!(collected.diagnostics().is_empty());
            let env = collected.into_env();
            assert_eq!((shells.declarations().len(), env.symbols().len(), env.definitions().len(), env.contributions().len()), (5, 3, 3, 1));
            let result = source_property_implementation_output(ast, module.clone(), &shells, &env, &source);
            if case == "original" {
                result.expect("baseline selected").expect("baseline succeeds");
                baseline_artifacts = Some((ast.clone(), module, shells, env));
                continue;
            }
            let (baseline_ast, baseline_module, baseline_shells, baseline_env) = baseline_artifacts.as_ref().unwrap();
            let moved = ast.nodes()[prefix[2]].range;
            let original_range = baseline_ast.nodes()[prefix[2]].range;
            assert_ne!((moved.start, moved.end), (original_range.start, original_range.end));
            refusals.push((profile, case, source_property_implementation_output(
                baseline_ast, baseline_module.clone(), baseline_shells, baseline_env, &source)
                .is_some_and(|result| result.is_err())));
            positives.push((profile, case, source, ast.clone(), module, shells, env, result, prefix, parameter));
        }
    }
    assert_eq!((positives.len(), refusals.len()), (4, 4));
    let failures = positives.iter().filter_map(|(profile, case, _, _, _, _, _, result, _, _)| match result {
        None => Some(format!("{profile}/{case}: None")),
        Some(Err(error)) => Some(format!("{profile}/{case}: {error}")),
        Some(Ok(_)) => None,
    }).collect::<Vec<_>>();
    assert!(failures.is_empty(), "raw prefix route positives: {failures:?}");
    for (profile, case, refused) in refusals {
        assert!(refused, "{profile}/{case}: selected stale prefix");
    }
    for (profile, _, source, ast, module, shells, env, result, prefix, parameter) in positives {
        let output = result.unwrap().unwrap();
        let typed = &output.typed_ast;
        let resolved = &output.resolved;
        let property = typed.source_property_implementation().unwrap();
        let types = typed.source_type().unwrap();
        let context = typed.source_context().unwrap();
        for (shell, node) in shells.declarations().iter().zip(prefix) {
            assert_eq!(shell.node_id().index(), node);
            assert_eq!(shell.range(), ast.nodes()[node].range);
        }
        let carrier = property.carrier_identity();
        for (index, origin) in [carrier.structure_origin(), carrier.field_origin(), carrier.property_origin()].into_iter().enumerate() {
            let definition = env.definitions().iter().find(|row| row.id().index() == index).unwrap();
            assert_eq!(origin, definition.origin());
            assert_eq!(origin, env.symbols().get(definition.symbol()).unwrap().origin());
            assert_eq!(origin.anchor(), &SourceAnchor::Range(ast.nodes()[prefix[index + 1]].range));
            assert_eq!(origin.source_id(), ast.source_id);
            assert_eq!(origin.module_id(), &module);
            assert!(!origin.is_recovered());
            assert!(origin.import_edge().is_none());
        }
        assert_eq!(types.structure_members().len(), 2);
        assert_eq!(types.expressions().len(), 3);
        for (index, (_, member)) in types.structure_members().iter().enumerate() {
            let site = prefix[index + 2];
            assert_eq!(member.member_range(), ast.nodes()[site].range);
            assert_eq!(member.member_site(), &mizar_checker::typed_ast::TypedSiteRef::Node(mizar_checker::typed_ast::TypedNodeId::new(site)));
            let expression = types.expressions().get(member.root()).unwrap();
            let set = ast.nodes()[ast.token_nodes()[if index == 0 { 7 } else { 12 }].index()].range;
            assert_eq!((expression.source_range(), expression.head_range()), (set, set));
            assert_eq!(expression.head(), &mizar_checker::source_type::SourceTypeHead::BuiltinSet);
        }
        let declaration = ast.nodes()[ast.token_nodes()[20].index()].range;
        let written_type = ast.nodes()[ast.token_nodes()[22].index()].range;
        let row = context.declarations().iter().next().unwrap().1;
        assert_eq!(row.declaration_range, declaration);
        assert_eq!(row.written_type_range, written_type);
        assert_ne!(row.declaration_range, ast.nodes()[parameter].range);
        assert_eq!(typed.nodes().node(mizar_checker::typed_ast::TypedNodeId::new(parameter)).unwrap().anchor, SourceAnchor::Range(declaration));
        assert_eq!(types.expressions().iter().next().unwrap().1.source_range(), written_type);
        assert_eq!(property.parameters().iter().next().unwrap().1.declaration_range(), declaration);
        if profile == "equals" {
            let consumer = mizar_checker::source_property_implementation::SourcePropertyEqualsSelectorIdentityProducer::build(
                &env, property.clone(), typed.source_term().unwrap().clone(), typed.source_structure().unwrap().clone()).unwrap();
            assert_eq!(consumer.property(), property);
        }
        assert_eq!(typed.initial_obligations().len(), if profile == "means" { 2 } else { 0 });
        for (_, row) in property.correctness().iter() {
            assert_eq!(typed.initial_obligations().get(row.obligation()).unwrap().status, mizar_checker::typed_ast::InitialObligationStatus::Pending);
        }
        assert_eq!(resolved.source_context(), typed.source_context());
        assert_eq!(resolved.source_type(), typed.source_type());
        assert_eq!(resolved.source_term(), typed.source_term());
        assert_eq!(resolved.source_structure(), typed.source_structure());
        assert_eq!(resolved.source_atomic_formula(), typed.source_atomic_formula());
        assert_eq!(resolved.source_property_implementation(), Some(property));
        let replay = source_property_implementation_output(&ast, module, &shells, &env, &source).unwrap().unwrap();
        assert_eq!((replay.typed_ast, replay.resolved), (output.typed_ast.clone(), output.resolved.clone()));
        assert!(typed.types().is_empty());
        assert!(typed.facts().is_empty());
        assert!(typed.coercions().is_empty());
        assert!(typed.diagnostics().is_empty());
        assert!(resolved.expr_metadata().is_empty());
        assert!(resolved.inserted_coercions().is_empty());
        assert!(resolved.cluster_facts().is_empty());
        assert!(resolved.checked_formulas().is_empty());
        assert!(resolved.statement_semantics().is_empty());
        assert!(resolved.checked_proofs().is_empty());
        assert!(resolved.diagnostics().is_empty());
    }
}

#[test]
fn task264_native_original_prefix_alias_has_late_arena_owner() {
    use mizar_syntax::SurfaceNodeKind;

    for (ordinal, (profile, original)) in [
        ("means", SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT),
        ("equals", SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT),
    ]
    .into_iter()
    .enumerate()
    {
        let baseline = task257c4c1_frontend_output(original, 265_400 + ordinal * 2);
        assert!(baseline.diagnostics.is_empty(), "{profile} baseline");
        let baseline_ast = baseline.ast.as_ref().expect("native baseline AST");
        let source = metamorphic_edits::variants(baseline_ast, original)
            .into_iter()
            .find(|(kind, _)| *kind == "intertoken_layout")
            .map(|(_, edits)| metamorphic_edits::apply(original, &edits))
            .expect("native intertoken-layout variant");
        assert_ne!(source, original);
        let frontend = task257c4c1_frontend_output(&source, 265_401 + ordinal * 2);
        assert!(
            frontend.diagnostics.is_empty()
                && frontend.preprocessed.diagnostics.is_empty()
                && frontend.tokens.diagnostics.is_empty(),
            "{profile} frontend, preprocessing, or lexer diagnostics"
        );
        assert_eq!(frontend.source.source_text.as_ref(), source);
        let ast = frontend.ast.as_ref().expect("native variant AST");
        assert_eq!(ast.source_id, frontend.source.source_id);
        assert!(ast.node_views().all(|node| !node.is_recovered()));
        let first_block = |ast: &mizar_syntax::SurfaceAst| {
            ast.nodes()
                .iter()
                .position(|node| matches!(&node.kind, SurfaceNodeKind::DefinitionBlockItem))
                .expect("native DefinitionBlockItem")
        };
        let (baseline_index, index) = (first_block(baseline_ast), first_block(ast));
        assert_eq!(index, baseline_index, "{profile} first block identity");
        let (block, baseline_block) = (&ast.nodes()[index], &baseline_ast.nodes()[baseline_index]);
        assert_eq!(block.kind, baseline_block.kind);
        assert_ne!(
            (block.range.start, block.range.end),
            (baseline_block.range.start, baseline_block.range.end)
        );

        let module = mizar_resolve::resolved_ast::ModuleId::new(
            frontend.source.package_id.clone(),
            frontend.source.module_path.clone(),
        );
        let shells = mizar_resolve::declarations::DeclarationShellCollector::new(ast, &module).collect();
        let collected = mizar_resolve::symbols::SignatureProjectionExtractor::new(
            ast,
            &shells,
            mizar_resolve::env::NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        assert!(collected.diagnostics().is_empty(), "{profile} resolver");
        let env = collected.into_env();
        let source_arc = frontend.source.source_text.clone();
        let ast_before = ast.clone();
        let pristine = source_property_implementation_output(
            ast, module.clone(), &shells, &env, source_arc.as_ref(),
        );
        assert!(
            pristine.as_ref().is_some_and(|result| result.is_ok()),
            "{profile} pristine route: {pristine:?}"
        );
        assert_eq!(ast, &ast_before);
        let error = source_property_implementation_output_with_mutation(
            ast,
            module,
            &shells,
            &env,
            source_arc.as_ref(),
            SourcePropertyImplementationRouteMutation::WrongNativePrefixAlias,
        )
        .expect("native selector")
        .expect_err("corrupt native prefix alias must fail at its arena owner");
        assert_eq!(
            error,
            "Task264 property implementation: invalid property-implementation typed-arena ownership"
        );
        assert_eq!(ast, &ast_before);
        assert_eq!(frontend.source.source_text, source_arc);
    }
}
