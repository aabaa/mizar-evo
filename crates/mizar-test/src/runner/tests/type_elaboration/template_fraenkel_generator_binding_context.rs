use mizar_checker::{
    binding_env::{BindingLookupResult, BindingLookupSite},
    source_formula_composition::{
        SourceFraenkelGeneratorBindingContextId,
        SourceFraenkelGeneratorBindingContextProducer, SourceFraenkelGeneratorUsePositionId,
    },
};

const TASK257C4A_FIXTURE: &str = include_str!(
    "../../../../../../tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz"
);

#[test]
fn task257c4a_real_fixture_builds_exact_fraenkel_generator_binding_context() {
    let (ast, module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(TASK257C4A_FIXTURE, 257_104);
    assert_eq!(diagnostics, 0, "Task257C4A fixture parser diagnostics");
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (57, Some(56))
    );

    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("Task257C4A fixture resolver arena should lower");
    let templates =
        mizar_resolve::names::TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
            .expect("Task257C4A template collector should validate the resolver arena")
            .collect()
            .expect("Task257C4A template collector should collect the real profile");
    let generators = mizar_resolve::names::FraenkelGeneratorVariableSourceCollector::new(
        &ast, &module, &resolved,
    )
    .expect("Task257C4A generator collector should validate the resolver arena")
    .collect()
    .expect("Task257C4A generator collector should collect the real profile");
    let profile = typed_ast_from_surface_resolved_profile(&ast, module.clone(), &resolved);
    let template = SourceTemplateTypeParameterAssociationProducer::build(&templates, &profile.typed_ast)
        .expect("Task257C4A template handoff should build");
    let structural = SourceTemplateFraenkelStructuralCompositionProducer::build(
        &template,
        &generators,
        &profile.typed_ast,
    )
    .expect("Task257C4A structural handoff should build");
    let before = profile.typed_ast.clone();
    let handoff = SourceFraenkelGeneratorBindingContextProducer::build(
        &structural,
        &generators,
        &profile.typed_ast,
    )
    .expect("Task257C4A binding context should build the real profile");

    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(
        handoff.structural_summary(),
        format!(
            "source-template-fraenkel-structural-composition-v1|module={}.{}|compositions=1|uses=3",
            module.package().as_str(),
            module.path().as_str(),
        )
    );
    assert_eq!(
        handoff.resolver_summary(),
        format!(
            "fraenkel-generator-variable-source-v1|module={}.{}|bindings=1|uses=3",
            module.package().as_str(),
            module.path().as_str(),
        )
    );
    assert_eq!(handoff.bindings().len(), 1);
    assert_eq!(handoff.bindings().iter().count(), 1);
    let rows = handoff.bindings().iter().collect::<Vec<_>>();
    let [(binding_context, binding)] = rows.as_slice() else {
        panic!("Task257C4A fixture must produce exactly one binding context");
    };
    assert_eq!(*binding_context, SourceFraenkelGeneratorBindingContextId::new(0));
    assert_eq!(binding.composition().index(), 0);
    assert_eq!(binding.resolver_binding().index(), 0);
    assert_eq!(binding.context(), BindingContextId::new(1));
    assert_eq!(binding.binding(), BindingId::new(0));
    assert_eq!(binding.source_ordinal(), 0);
    assert!(
        handoff
            .bindings()
            .get(*binding_context)
            .is_some_and(|stored| stored == *binding)
    );
    assert!(
        handoff
            .bindings()
            .get(SourceFraenkelGeneratorBindingContextId::new(1))
            .is_none()
    );

    assert_eq!(handoff.use_positions().len(), 3);
    assert_eq!(
        handoff
            .use_positions()
            .iter()
            .map(|(id, row)| (
                id.index(),
                row.binding_context().index(),
                row.resolver_use_index(),
                row.source_ordinal(),
                row.lookup_ordinal(),
            ))
            .collect::<Vec<_>>(),
        vec![(0, 0, 0, 0, 1), (1, 0, 1, 1, 2), (2, 0, 2, 2, 3)]
    );
    assert!(
        handoff
            .use_positions()
            .get(SourceFraenkelGeneratorUsePositionId::new(3))
            .is_none()
    );

    assert_eq!(
        handoff.binding_env().debug_text(),
        format!(
            "binding-env-debug-v1\nmodule: {}::{}\ncontexts:\n  context#0 owner=module parent=none layer=module scope=none bindings=[] visible=[] recovery=normal\n  context#1 owner=source-comprehension(663..694) parent=context#0 layer=expression scope=none bindings=[binding#0] visible=[binding#0] recovery=normal\nbindings:\n  binding#0 spelling=\"x\" kind=quantifier_binder owner=context#1 identity=source_bound(context#1, ordinal=0) range=673..674 visible_after=0 type=source(678..679) status=active captured=[] diagnostics=[] recovery=normal\ndiagnostics:\n",
            module.package().as_str(),
            module.path().as_str(),
        )
    );
    assert!(matches!(
        handoff.binding_env().lookup(&BindingLookupSite::new(
            "x",
            BindingContextId::new(1),
            None,
            0,
        )),
        Ok(BindingLookupResult::ForwardReference { candidates, .. })
            if candidates == vec![BindingId::new(0)]
    ));
    for ordinal in 1..=3 {
        assert!(matches!(
            handoff.binding_env().lookup(&BindingLookupSite::new(
                "x",
                BindingContextId::new(1),
                None,
                ordinal,
            )),
            Ok(BindingLookupResult::Local(binding)) if binding == BindingId::new(0)
        ));
    }
    assert_eq!(
        handoff.debug_text(),
        format!(
            "source-fraenkel-generator-binding-context-v1|module={}.{}|bindings=1|use-positions=3",
            module.package().as_str(),
            module.path().as_str(),
        )
    );
    assert_eq!(profile.typed_ast, before);
}
