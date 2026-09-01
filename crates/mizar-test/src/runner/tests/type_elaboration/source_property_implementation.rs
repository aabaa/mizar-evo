use super::type_elaboration::{
    SOURCE_PROPERTY_IMPLEMENTATION_EQUALS_TEXT, SOURCE_PROPERTY_IMPLEMENTATION_MEANS_TEXT,
    SourcePropertyImplementationRouteMutation, SourcePropertyImplementationRouteOutput,
    source_property_implementation_output, source_property_implementation_output_with_mutation,
};

const TASK264_MEANS_CASE: &str =
    "pass_type_elaboration_property_implementation_means_payload_001";
const TASK264_EQUALS_CASE: &str =
    "pass_type_elaboration_property_implementation_equals_payload_001";
const TASK264_SPEC_REF: &str =
    "spec.en.checker.type_elaboration.source_property_implementation_payload";

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
        assert!(source_property_implementation_output(&ast, module.clone(), &shells, &symbols, &source[..source.len()-1]).is_none());
        let mut extended = source.to_owned();
        extended.push(' ');
        assert!(source_property_implementation_output(&ast, module.clone(), &shells, &symbols, &extended).is_none());
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
    assert_eq!((plan.cases.len(), plan.manifest.requirements.len()), (550, 499));
    assert_eq!(
        (plan.coverage_report.pass_fail_mix.pass, plan.coverage_report.pass_fail_mix.fail),
        (307, 243)
    );
    assert_eq!(
        (
            crate::active_parse_only_cases(&plan).count(),
            crate::active_declaration_symbol_cases(&plan).count(),
            active_type_elaboration_cases(&plan).count(),
            crate::active_proof_verification_cases(&plan).count(),
        ),
        (101, 7, 205, 1)
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
    let summary = mizar_core::elaborator::ResolvedTypedAstSummary::new(
        checker_owner.source_id(),
        checker_owner.module_id().clone(),
    );
    let summary = if mutation == Task264CarrierCoreContextMutation::UnexpectedCheckerSite {
        summary.with_checker_sites(vec![
            mizar_core::elaborator::CheckerSiteSummary::failed_overload(
                mizar_checker::resolved_typed_ast::OverloadResolutionId::new(0),
                mizar_core::core_ir::CoreSourceRef::direct(mizar_session::SourceRange {
                    source_id: checker_owner.source_id(),
                    start: 13,
                    end: 101,
                }),
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
                source_id: checker_owner.source_id(),
                start: 14,
                end: 101,
            }
        } else {
            mizar_session::SourceRange {
                source_id: checker_owner.source_id(),
                start: 13,
                end: 101,
            }
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
                mizar_core::core_ir::CoreSourceRef::direct(mizar_session::SourceRange {
                    source_id: checker_owner.source_id(),
                    start: 13,
                    end: 101,
                })
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
                mizar_core::core_ir::CoreSourceRef::direct(mizar_session::SourceRange {
                    source_id: checker_owner.source_id(),
                    start: 13,
                    end: 101,
                }),
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
                mizar_core::core_ir::CoreSourceRef::direct(mizar_session::SourceRange {
                    source_id: checker_owner.source_id(),
                    start: 13,
                    end: 101,
                }),
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
