#[test]
fn task257c4b_real_fixture_builds_exact_fraenkel_generator_bound_uses() {
    let (ast, module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(TASK257C4A_FIXTURE, 257_104);
    assert_eq!(diagnostics, 0, "Task257C4B fixture parser diagnostics");
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (57, Some(56))
    );

    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("Task257C4B fixture resolver arena should lower");
    let templates =
        mizar_resolve::names::TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
            .expect("Task257C4B template collector should validate the resolver arena")
            .collect()
            .expect("Task257C4B template collector should collect the real profile");
    let generators = mizar_resolve::names::FraenkelGeneratorVariableSourceCollector::new(
        &ast, &module, &resolved,
    )
    .expect("Task257C4B generator collector should validate the resolver arena")
    .collect()
    .expect("Task257C4B generator collector should collect the real profile");
    let profile = typed_ast_from_surface_resolved_profile(&ast, module.clone(), &resolved);
    let template = mizar_checker::source_template_type_parameter_association::SourceTemplateTypeParameterAssociationProducer::build(
        &templates,
        &profile.typed_ast,
    )
    .expect("Task257C4B template handoff should build");
    let structural = mizar_checker::source_template_type_parameter_association::SourceTemplateFraenkelStructuralCompositionProducer::build(
        &template,
        &generators,
        &profile.typed_ast,
    )
    .expect("Task257C4B structural handoff should build");
    let binding_context = mizar_checker::source_formula_composition::SourceFraenkelGeneratorBindingContextProducer::build(
        &structural,
        &generators,
        &profile.typed_ast,
    )
    .expect("Task257C4B binding context should build");
    let binding_context_before = binding_context.clone();
    let handoff = mizar_checker::source_formula_composition::SourceFraenkelGeneratorBoundUseProducer::build(
        &binding_context,
    )
    .expect("Task257C4B bound uses should build the real profile");

    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(handoff.dependency_summary(), binding_context.debug_text());
    assert_eq!(handoff.bound_uses().len(), 3);
    assert!(!handoff.bound_uses().is_empty());
    assert_eq!(
        handoff
            .bound_uses()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.use_position().index(),
                row.binding_context().index(),
                row.resolver_use_index(),
                row.source_ordinal(),
                row.lookup_ordinal(),
                row.context().index(),
                row.binding().index(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, 0, 0, 0, 0, 1, 1, 0),
            (1, 1, 0, 1, 1, 2, 1, 0),
            (2, 2, 0, 2, 2, 3, 1, 0),
        ]
    );
    assert!(
        handoff
            .bound_uses()
            .get(
                mizar_checker::source_formula_composition::SourceFraenkelGeneratorBoundUseId::new(
                    3,
                )
            )
            .is_none()
    );
    for (_, row) in handoff.bound_uses().iter() {
        assert!(matches!(
            binding_context.binding_env().lookup(
                &mizar_checker::binding_env::BindingLookupSite::new(
                    "x",
                    row.context(),
                    None,
                    row.lookup_ordinal(),
                )
            ),
            Ok(mizar_checker::binding_env::BindingLookupResult::Local(binding))
                if binding == row.binding()
        ));
    }
    assert!(matches!(
        binding_context.binding_env().lookup(
            &mizar_checker::binding_env::BindingLookupSite::new(
                "x",
                mizar_checker::binding_env::BindingContextId::new(1),
                None,
                0,
            )
        ),
        Ok(mizar_checker::binding_env::BindingLookupResult::ForwardReference {
            candidates,
            ..
        }) if candidates == vec![mizar_checker::binding_env::BindingId::new(0)]
    ));
    assert_eq!(
        handoff.debug_text(),
        format!(
            "source-fraenkel-generator-bound-use-v1|module={}.{}|bound-uses=3",
            module.package().as_str(),
            module.path().as_str(),
        )
    );
    assert!(binding_context == binding_context_before);
}
