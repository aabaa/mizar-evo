use mizar_checker::source_template_type_parameter_association::{
    SourceTemplateFraenkelStructuralCompositionId,
    SourceTemplateFraenkelStructuralCompositionProducer,
};

const TASK277C_FIXTURE: &str = include_str!(
    "../../../../../../tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz"
);

#[test]
fn task277c_real_fixture_builds_exact_template_fraenkel_structural_composition() {
    let (ast, module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(TASK277C_FIXTURE, 277_103);
    assert_eq!(diagnostics, 0, "Task277C fixture parser diagnostics");
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (57, Some(56))
    );

    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("Task277C fixture resolver arena should lower");
    let templates =
        mizar_resolve::names::TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
            .expect("Task277C template collector should validate the resolver arena")
            .collect()
            .expect("Task277C template collector should collect the real profile");
    let generators = mizar_resolve::names::FraenkelGeneratorVariableSourceCollector::new(
        &ast, &module, &resolved,
    )
    .expect("Task277C generator collector should validate the resolver arena")
    .collect()
    .expect("Task277C generator collector should collect the real profile");
    let profile = typed_ast_from_surface_resolved_profile(&ast, module.clone(), &resolved);
    let template = mizar_checker::source_template_type_parameter_association::SourceTemplateTypeParameterAssociationProducer::build(&templates, &profile.typed_ast)
        .expect("Task277C template handoff should build");
    let before = profile.typed_ast.clone();
    let handoff = SourceTemplateFraenkelStructuralCompositionProducer::build(
        &template,
        &generators,
        &profile.typed_ast,
    )
    .expect("Task277C structural composition should build the real profile");

    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(handoff.compositions().len(), 1);
    assert_eq!(handoff.compositions().iter().count(), 1);
    let rows = handoff.compositions().iter().collect::<Vec<_>>();
    let [(composition_id, row)] = rows.as_slice() else {
        panic!("Task277C fixture must produce exactly one composition");
    };
    assert_eq!(composition_id.index(), 0);
    assert_eq!(row.template_association().index(), 0);
    assert_eq!(
        row.template_binding(),
        mizar_resolve::names::TemplateTypeParameterBindingId::new(0)
    );
    assert_eq!(
        row.generator_binding(),
        mizar_resolve::names::FraenkelGeneratorVariableBindingId::new(0)
    );
    assert_eq!(
        (
            row.definition_block().index(),
            row.parameter().index(),
            row.template_binder().index(),
            row.type_head().index(),
            row.template_identifier().index(),
        ),
        (53, 31, 2, 39, 21)
    );
    assert_eq!(
        (
            row.functor_definition().index(),
            row.comprehension().index(),
            row.segment().index(),
            row.generator_binder().index(),
            row.type_expression().index(),
        ),
        (52, 49, 41, 19, 40)
    );
    assert_eq!(
        (
            row.mapper_role_owner().index(),
            row.mapper_term_reference().index(),
            row.mapper_identifier().index(),
        ),
        (38, 37, 17)
    );
    assert_eq!(
        (
            row.first_condition_role_owner().index(),
            row.first_condition_term_reference().index(),
            row.first_condition_identifier().index(),
        ),
        (48, 42, 24)
    );
    assert_eq!(
        (
            row.second_condition_role_owner().index(),
            row.second_condition_term_reference().index(),
            row.second_condition_identifier().index(),
        ),
        (48, 44, 26)
    );
    assert_eq!(
        (
            row.mapper_source_ordinal(),
            row.mapper_role_source_ordinal(),
            row.first_condition_source_ordinal(),
            row.first_condition_role_source_ordinal(),
            row.second_condition_source_ordinal(),
            row.second_condition_role_source_ordinal(),
        ),
        (0, 0, 1, 0, 2, 1)
    );
    assert_eq!(
        handoff.debug_text(),
        "source-template-fraenkel-structural-composition-v1|module=mizar-test-task253-corruption.tests.task253_local_corruption_277103|compositions=1|uses=3"
    );
    assert!(
        handoff
            .compositions()
            .get(*composition_id)
            .is_some_and(|stored| stored == *row)
    );
    assert!(
        handoff
            .compositions()
            .get(SourceTemplateFraenkelStructuralCompositionId::new(1))
            .is_none()
    );
    assert_eq!(profile.typed_ast, before);
}
