use mizar_resolve::{
    names::TemplateTypeParameterSourceCollector, resolved_ast::SurfaceResolvedArena,
};

const TASK277R1_FIXTURE: &str = include_str!(
    "../../../../../../tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz"
);

#[test]
fn task277r1_real_fixture_links_exact_template_generator_identity() {
    let (ast, module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(TASK277R1_FIXTURE, 277_100);
    assert_eq!(diagnostics, 0, "Task277R1 fixture parser diagnostics");
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (57, Some(56))
    );

    let resolved = SurfaceResolvedArena::lower(&ast, &module)
        .expect("Task277R1 fixture resolver arena should lower");
    let collection = TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
        .expect("Task277R1 fixture collector should validate the resolver arena")
        .collect()
        .expect("Task277R1 fixture collector should collect the real profile");

    assert_eq!(collection.source_id(), ast.source_id);
    assert_eq!(collection.module(), &module);
    assert_eq!(collection.bindings().len(), 1);
    assert_eq!(collection.generator_links().len(), 1);

    let binding_rows = collection.bindings().iter().collect::<Vec<_>>();
    let [(binding_id, binding)] = binding_rows.as_slice() else {
        panic!("Task277R1 must collect exactly one template type parameter binding");
    };
    assert_eq!(binding_id.index(), 0);
    assert_eq!(binding.definition_block().index(), 53);
    assert_eq!(binding.parameter().index(), 31);
    assert_eq!(binding.binder().index(), 2);
    assert_eq!(binding.spelling(), "T");
    assert_eq!(
        (
            binding.source_range().source_id,
            binding.source_range().start,
            binding.source_range().end
        ),
        (ast.source_id, 606, 620)
    );
    assert_eq!(binding.source_ordinal(), 0);

    let link_rows = collection.generator_links().iter().collect::<Vec<_>>();
    let [link] = link_rows.as_slice() else {
        panic!("Task277R1 must collect exactly one template generator type-head link");
    };
    assert_eq!(link.definition_block().index(), 53);
    assert_eq!(link.type_head().index(), 39);
    assert_eq!(link.identifier().index(), 21);
    assert_eq!(link.binding(), *binding_id);
    assert_eq!(
        (
            link.source_range().source_id,
            link.source_range().start,
            link.source_range().end
        ),
        (ast.source_id, 678, 679)
    );
    assert_eq!(link.source_ordinal(), 0);
}
