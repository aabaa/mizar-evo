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
        assert_eq!((handoff.implementations().len(), handoff.parameters().len(), handoff.targets().len(), handoff.definientia().len(), handoff.correctness().len()), (1, 1, 1, 1, obligations));
        assert_eq!(output.typed_ast.initial_obligations().len(), obligations);
        assert_eq!(output.typed_ast.source_property_implementation(), output.resolved.source_property_implementation());
        assert_eq!(output.typed_ast.debug_text().matches("source-property-implementation-debug-v1").count(), 1);
        assert_eq!(output.resolved.debug_text().matches("source-property-implementation-debug-v1").count(), 1);
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
    assert_eq!((plan.cases.len(), plan.manifest.requirements.len()), (429, 396));
    assert_eq!(
        (plan.coverage_report.pass_fail_mix.pass, plan.coverage_report.pass_fail_mix.fail),
        (236, 193)
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
    assert_eq!((type_stage.requirements, type_stage.covered), (259, 247));
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
