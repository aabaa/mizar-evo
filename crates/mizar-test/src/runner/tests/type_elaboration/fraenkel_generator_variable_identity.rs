use mizar_resolve::names::{
    FraenkelGeneratorVariableBindingId, FraenkelGeneratorVariableSourceCollector,
    FraenkelGeneratorVariableUseRole,
};

const TASK277R2_FIXTURE: &str = include_str!(
    "../../../../../../tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz"
);

#[test]
fn task277r2_real_fixture_links_exact_fraenkel_generator_binding_and_uses() {
    let (ast, module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(TASK277R2_FIXTURE, 277_102);
    assert_eq!(diagnostics, 0, "Task277R2 fixture parser diagnostics");
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (57, Some(56))
    );

    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("Task277R2 fixture resolver arena should lower");
    let collection = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("Task277R2 fixture collector should validate the resolver arena")
        .collect()
        .expect("Task277R2 fixture collector should collect the real profile");

    assert_eq!(collection.source_id(), ast.source_id);
    assert_eq!(collection.module(), &module);
    assert_eq!(
        collection.debug_text(),
        "fraenkel-generator-variable-source-v1|module=mizar-test-task253-corruption.tests.task253_local_corruption_277102|bindings=1|uses=3"
    );
    assert_eq!(collection.bindings().len(), 1);
    assert_eq!(collection.uses().len(), 3);

    let binding_rows = collection.bindings().iter().collect::<Vec<_>>();
    let [(binding_id, binding)] = binding_rows.as_slice() else {
        panic!("Task277R2 must collect exactly one generator-variable binding");
    };
    assert_eq!(*binding_id, FraenkelGeneratorVariableBindingId::new(0));
    assert_eq!(binding.definition_block().index(), 53);
    assert_eq!(binding.functor_definition().index(), 52);
    assert_eq!(binding.comprehension().index(), 49);
    assert_eq!(binding.segment().index(), 41);
    assert_eq!(binding.binder().index(), 19);
    assert_eq!(binding.spelling(), "x");
    assert_eq!(
        (
            binding.segment_range().source_id,
            binding.segment_range().start,
            binding.segment_range().end,
        ),
        (ast.source_id, 673, 679)
    );
    assert_eq!(
        (
            binding.binder_range().source_id,
            binding.binder_range().start,
            binding.binder_range().end,
        ),
        (ast.source_id, 673, 674)
    );
    assert_eq!(binding.source_ordinal(), 0);

    let uses = collection.uses().iter().collect::<Vec<_>>();
    let [mapper, first_condition, second_condition] = uses.as_slice() else {
        panic!("Task277R2 must collect the exact mapper and two condition uses");
    };
    assert_eq!(mapper.definition_block().index(), 53);
    assert_eq!(mapper.functor_definition().index(), 52);
    assert_eq!(mapper.comprehension().index(), 49);
    assert_eq!(mapper.role_owner().index(), 38);
    assert_eq!(mapper.term_reference().index(), 37);
    assert_eq!(mapper.identifier().index(), 17);
    assert_eq!(mapper.binding(), *binding_id);
    assert_eq!(mapper.role(), FraenkelGeneratorVariableUseRole::Mapper);
    assert_eq!(mapper.source_ordinal(), 0);
    assert_eq!(mapper.role_source_ordinal(), 0);
    assert_eq!(
        (
            mapper.identifier_range().source_id,
            mapper.identifier_range().start,
            mapper.identifier_range().end,
        ),
        (ast.source_id, 665, 666)
    );

    for (expected_reference, expected_identifier, expected_ordinal, expected_role_ordinal, link) in [
        (42, 24, 1, 0, *first_condition),
        (44, 26, 2, 1, *second_condition),
    ] {
        assert_eq!(link.definition_block().index(), 53);
        assert_eq!(link.functor_definition().index(), 52);
        assert_eq!(link.comprehension().index(), 49);
        assert_eq!(link.role_owner().index(), 48);
        assert_eq!(link.term_reference().index(), expected_reference);
        assert_eq!(link.identifier().index(), expected_identifier);
        assert_eq!(link.binding(), *binding_id);
        assert_eq!(link.role(), FraenkelGeneratorVariableUseRole::Condition);
        assert_eq!(link.source_ordinal(), expected_ordinal);
        assert_eq!(link.role_source_ordinal(), expected_role_ordinal);
    }
    assert_eq!(first_condition.identifier_range().start, 686);
    assert_eq!(second_condition.identifier_range().start, 691);
    assert!(collection.uses().get(3).is_none());
}
