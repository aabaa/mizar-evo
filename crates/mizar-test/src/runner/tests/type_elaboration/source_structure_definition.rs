use super::type_elaboration::{
    SOURCE_STRUCTURE_DEFINITION_TEXT, SourceStructureDefinitionRouteMutation,
    source_structure_definition_output, source_structure_definition_output_with_mutation,
};

const TASK263_CASE: &str = "pass_type_elaboration_structure_definition_payload_001";
const TASK263_SPEC_REF: &str =
    "spec.en.checker.type_elaboration.source_structure_definition_payload";

#[test]
fn task263_structure_definition_source_surface_and_resolver_are_exact() {
    assert_eq!(SOURCE_STRUCTURE_DEFINITION_TEXT.len(), 320);
    assert_eq!(SOURCE_STRUCTURE_DEFINITION_TEXT.lines().count(), 16);
    assert!(SOURCE_STRUCTURE_DEFINITION_TEXT.ends_with('\n'));
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_STRUCTURE_DEFINITION_TEXT, 263_000);
    assert_eq!(ast.nodes().len(), 75);
    assert_eq!(ast.root().map(|id| id.index()), Some(74));
    assert!(ast.expression_root().is_none());
    assert_eq!(shells.declarations().len(), 10);
    assert_eq!(symbols.symbols().len(), 8);
    assert_eq!(symbols.definitions().len(), 8);
    assert_eq!(symbols.contributions().len(), 1);
    let output = source_structure_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_STRUCTURE_DEFINITION_TEXT,
    )
    .expect("Task263 exact route")
    .expect("Task263 exact payload");
    let structure = output
        .typed_ast
        .source_structure_definition()
        .expect("Task263 typed handoff");
    assert_eq!(structure.definitions().len(), 2);
    assert_eq!(structure.members().len(), 4);
    assert_eq!(structure.inheritances().len(), 1);
    assert_eq!(structure.mappings().len(), 2);
    assert!(structure.coherence_requests().is_empty());
    assert_eq!(
        output.typed_ast.source_structure_definition(),
        output.resolved.source_structure_definition()
    );
}

#[test]
fn task263_structure_definition_lower_payload_and_subtree_corruption_fail_closed() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_STRUCTURE_DEFINITION_TEXT, 263_001);
    let mut mutations = Vec::new();
    mutations.extend(
        (0..75).map(SourceStructureDefinitionRouteMutation::WrongSurfaceRow),
    );
    mutations.extend([
        SourceStructureDefinitionRouteMutation::WrongSurfaceRange(57),
        SourceStructureDefinitionRouteMutation::WrongSurfaceRecovery(65),
        SourceStructureDefinitionRouteMutation::WrongSurfaceChildren(70),
        SourceStructureDefinitionRouteMutation::WrongSurfaceRoot,
    ]);
    mutations.extend((0..10).map(SourceStructureDefinitionRouteMutation::WrongShellRow));
    mutations.extend((0..8).map(SourceStructureDefinitionRouteMutation::WrongProjectionRow));
    mutations.extend([
        SourceStructureDefinitionRouteMutation::WrongResolverDefinition,
        SourceStructureDefinitionRouteMutation::WrongLowerMember,
        SourceStructureDefinitionRouteMutation::WrongDefinition,
        SourceStructureDefinitionRouteMutation::WrongMapping,
        SourceStructureDefinitionRouteMutation::WrongArena,
    ]);
    for mutation in mutations {
        assert!(
            source_structure_definition_output_with_mutation(
                &ast,
                module.clone(),
                &shells,
                &symbols,
                SOURCE_STRUCTURE_DEFINITION_TEXT,
                mutation,
            )
            .expect("Task263 exact source selection")
            .is_err(),
            "mutation {mutation:?} must fail closed"
        );
    }
    let without_final_lf = SOURCE_STRUCTURE_DEFINITION_TEXT.trim_end_matches('\n');
    let (wrong_ast, wrong_module, wrong_shells, wrong_symbols) =
        task253_ast_from_source_text(without_final_lf, 263_002);
    assert!(
        source_structure_definition_output(
            &wrong_ast,
            wrong_module,
            &wrong_shells,
            &wrong_symbols,
            without_final_lf,
        )
        .is_none()
    );
}

#[test]
fn task263_structure_definition_selection_trace_and_family_isolation_are_exact() {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap()
        .to_path_buf();
    let config = DiscoveryConfig {
        workspace_root: workspace_root.clone(),
        tests_root: workspace_root.join("tests"),
        manifest_path: workspace_root.join("tests/coverage/spec_trace.toml"),
        profile: TestProfile::Fast,
        validation_mode: ValidationMode::Metadata,
    };
    let plan = build_test_plan(&config).expect("Task263 repository plan");
    let (ordinal, case) = active_type_elaboration_cases(&plan)
        .enumerate()
        .find(|(_, case)| case.id.0 == TASK263_CASE)
        .expect("Task263 active case");
    assert_eq!(case.expectation.spec_refs.len(), 1);
    assert_eq!(case.expectation.spec_refs[0].0, TASK263_SPEC_REF);
    assert!(case.expectation.diagnostic_codes.is_empty());
    assert!(case.expectation.diagnostic_payloads.is_empty());
    let requirement = plan
        .manifest
        .requirements
        .iter()
        .find(|row| row.id.0 == TASK263_SPEC_REF)
        .expect("Task263 trace row");
    assert!(requirement.required);
    assert_eq!(
        requirement.status,
        crate::traceability::RequirementStatus::Covered
    );
    assert_eq!(
        requirement.coverage,
        crate::traceability::CoverageShape::Pass
    );
    assert_eq!(requirement.tests.len(), 1);
    let result = run_type_elaboration_case(
        &workspace_root,
        &workspace_root.join("tests"),
        case,
        ordinal,
    );
    assert_eq!(result.status, TypeElaborationCaseStatus::Passed);
    assert!(result.actual_detail_keys.is_empty());

    for (source, source_ordinal) in [
        (super::type_elaboration::SOURCE_PREDICATE_DEFINITION_TEXT, 263_010),
        (super::type_elaboration::SOURCE_FUNCTOR_DEFINITION_TEXT, 263_011),
        (super::type_elaboration::SOURCE_ATTRIBUTE_DEFINITION_TEXT, 263_012),
        (super::type_elaboration::SOURCE_MODE_DEFINITION_TEXT, 263_013),
    ] {
        let (ast, module, shells, symbols) =
            task253_ast_from_source_text(source, source_ordinal);
        assert!(
            source_structure_definition_output(&ast, module, &shells, &symbols, source).is_none()
        );
    }

    let gap = plan
        .cases
        .iter()
        .find(|case| case.id.0 == "fail_type_elaboration_mode_structure_definition_gap_001")
        .expect("mixed mode/structure gap remains");
    let gap_source = std::fs::read_to_string(&gap.source_path).unwrap();
    let (gap_ast, gap_module, gap_shells, gap_symbols) =
        task253_ast_from_source_text(&gap_source, 263_014);
    assert!(
        source_structure_definition_output(
            &gap_ast,
            gap_module,
            &gap_shells,
            &gap_symbols,
            &gap_source,
        )
        .is_none()
    );
}

#[test]
fn task263_structure_definition_semantic_deferrals_are_not_published() {
    let (ast, module, shells, symbols) =
        task253_ast_from_source_text(SOURCE_STRUCTURE_DEFINITION_TEXT, 263_020);
    let output = source_structure_definition_output(
        &ast,
        module,
        &shells,
        &symbols,
        SOURCE_STRUCTURE_DEFINITION_TEXT,
    )
    .unwrap()
    .unwrap();
    let typed = &output.typed_ast;
    let resolved = &output.resolved;
    assert!(typed.source_context().is_none());
    assert!(typed.initial_obligations().is_empty());
    assert!(typed.facts().is_empty());
    assert!(typed.coercions().is_empty());
    assert!(typed.diagnostics().is_empty());
    assert!(typed.source_predicate_definition().is_none());
    assert!(typed.source_functor_definition().is_none());
    assert!(typed.source_attribute_definition().is_none());
    assert!(typed.source_mode_definition().is_none());
    assert!(resolved.checked_formulas().is_empty());
    assert!(resolved.statement_semantics().is_empty());
    assert!(resolved.checked_proofs().is_empty());
    assert!(resolved.cluster_facts().is_empty());
    assert!(resolved.diagnostics().is_empty());
    let debug = typed
        .source_structure_definition()
        .unwrap()
        .debug_text();
    for prohibited in ["goal=", "guard=", "proof=", "accepted=", "fact=", "axiom=", "core", "control-flow", "vc="] {
        assert!(!debug.contains(prohibited), "published prohibited semantic field {prohibited}");
    }
}
