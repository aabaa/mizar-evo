use super::type_elaboration::{
    SOURCE_TEMPLATE_TEXT, source_template_output, source_template_output_with_mutation,
};
use mizar_checker::source_template::{
    SourceTemplateArgumentId, SourceTemplateArgumentsId, SourceTemplateLociId,
    SourceTemplateParameterKind, SourceTemplateParentKind, SourceTemplateRecovery,
};

fn task277a_fixture(
    ordinal: usize,
) -> (
    mizar_syntax::SurfaceAst,
    mizar_resolve::resolved_ast::ModuleId,
) {
    let (ast, module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(SOURCE_TEMPLATE_TEXT, 277_000 + ordinal);
    assert_eq!(diagnostics, 0, "Task277A fixture parser diagnostics");
    (ast, module)
}

#[test]
fn task277a_runner_extracts_exact_real_surface_profile() {
    assert_eq!(SOURCE_TEMPLATE_TEXT.len(), 207);
    assert!(SOURCE_TEMPLATE_TEXT.ends_with('\n'));
    let (ast, module) = task277a_fixture(0);
    let output = source_template_output(&ast, module, SOURCE_TEMPLATE_TEXT)
        .expect("Task277A must select the frozen parser fixture")
        .expect("Task277A exact parser transport must build");
    let handoff = &output.handoff;
    assert_eq!(output.typed_ast.nodes().len(), 116);
    assert_eq!(
        output.typed_ast.nodes().root(),
        Some(mizar_checker::typed_ast::TypedNodeId::new(115))
    );
    for (index, surface) in ast.nodes().iter().enumerate() {
        let typed = output
            .typed_ast
            .nodes()
            .node(mizar_checker::typed_ast::TypedNodeId::new(index))
            .expect("Task277A dense all-surface typed row");
        let expected_kind = match index {
            60 => "AbstractTypeSyntax".to_owned(),
            63 => "TypedValueSyntax".to_owned(),
            _ => format!("{:?}", surface.kind),
        };
        assert_eq!(typed.kind.as_str(), expected_kind);
        assert_eq!(typed.anchor, mizar_session::SourceAnchor::Range(surface.range));
        assert_eq!(
            typed.children,
            surface
                .children
                .iter()
                .map(|child| mizar_checker::typed_ast::TypedNodeId::new(child.index()))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            typed.recovery,
            mizar_checker::typed_ast::NodeRecoveryState::Normal
        );
        assert_eq!(typed.typing, mizar_checker::typed_ast::TypingState::Unknown);
        assert_eq!(typed.resolved_node, None);
        assert_eq!(
            typed.links,
            mizar_checker::typed_ast::TypedNodeLinks::default()
        );
    }
    assert_eq!(handoff.parameters().len(), 2);
    assert_eq!(handoff.loci_groups().len(), 2);
    assert_eq!(handoff.loci().len(), 2);
    assert_eq!(handoff.argument_groups().len(), 2);
    assert_eq!(handoff.arguments().len(), 2);

    let parameters = handoff.parameters().iter().collect::<Vec<_>>();
    assert_eq!(
        parameters[0].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(60)
    );
    assert_eq!(
        parameters[0].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(112)
    );
    assert_eq!(
        parameters[0].1.parent_kind(),
        SourceTemplateParentKind::DefinitionBlockItem
    );
    assert_eq!(
        parameters[0].1.kind(),
        SourceTemplateParameterKind::AbstractTypeSyntax
    );
    assert_eq!(parameters[0].1.source_range(), ast.nodes()[60].range);
    assert_eq!(parameters[0].1.source_ordinal(), 0);
    assert_eq!(parameters[0].1.recovery(), SourceTemplateRecovery::Normal);
    assert_eq!(
        parameters[1].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(63)
    );
    assert_eq!(
        parameters[1].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(112)
    );
    assert_eq!(
        parameters[1].1.parent_kind(),
        SourceTemplateParentKind::DefinitionBlockItem
    );
    assert_eq!(
        parameters[1].1.kind(),
        SourceTemplateParameterKind::TypedValueSyntax
    );
    assert_eq!(parameters[1].1.source_range(), ast.nodes()[63].range);
    assert_eq!(parameters[1].1.source_ordinal(), 1);
    assert_eq!(parameters[1].1.recovery(), SourceTemplateRecovery::Normal);

    let loci = handoff.loci_groups().iter().collect::<Vec<_>>();
    assert_eq!(
        loci[0].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(65)
    );
    assert_eq!(
        loci[0].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(66)
    );
    assert_eq!(
        loci[0].1.parent_kind(),
        SourceTemplateParentKind::PredicatePattern
    );
    assert_eq!(loci[0].1.source_range(), ast.nodes()[65].range);
    assert_eq!(loci[0].1.source_ordinal(), 0);
    assert_eq!(loci[0].1.recovery(), SourceTemplateRecovery::Normal);
    assert_eq!(
        loci[1].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(76)
    );
    assert_eq!(
        loci[1].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(77)
    );
    assert_eq!(
        loci[1].1.parent_kind(),
        SourceTemplateParentKind::FunctorPattern
    );
    assert_eq!(loci[1].1.source_range(), ast.nodes()[76].range);
    assert_eq!(loci[1].1.source_ordinal(), 1);
    assert_eq!(loci[1].1.recovery(), SourceTemplateRecovery::Normal);
    let locus = handoff.loci().iter().collect::<Vec<_>>();
    assert_eq!(
        locus[0].0,
        mizar_checker::source_template::SourceTemplateLocusId::new(0)
    );
    assert_eq!(
        locus[0].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(64)
    );
    assert_eq!(locus[0].1.loci(), SourceTemplateLociId::new(0));
    assert_eq!(locus[0].1.ordinal(), 0);
    assert_eq!(locus[0].1.source_range(), ast.nodes()[64].range);
    assert_eq!(locus[0].1.source_ordinal(), 0);
    assert_eq!(locus[0].1.recovery(), SourceTemplateRecovery::Normal);
    assert_eq!(
        locus[1].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(75)
    );
    assert_eq!(locus[1].1.loci(), SourceTemplateLociId::new(1));
    assert_eq!(locus[1].1.ordinal(), 0);
    assert_eq!(locus[1].1.source_range(), ast.nodes()[75].range);
    assert_eq!(locus[1].1.source_ordinal(), 1);
    assert_eq!(locus[1].1.recovery(), SourceTemplateRecovery::Normal);

    let arguments = handoff.argument_groups().iter().collect::<Vec<_>>();
    assert_eq!(
        arguments[0].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(91)
    );
    assert_eq!(
        arguments[0].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(92)
    );
    assert_eq!(
        arguments[0].1.parent_kind(),
        SourceTemplateParentKind::PredicateHead
    );
    assert_eq!(arguments[0].1.source_range(), ast.nodes()[91].range);
    assert_eq!(arguments[0].1.source_ordinal(), 0);
    assert_eq!(arguments[0].1.recovery(), SourceTemplateRecovery::Normal);
    assert_eq!(
        arguments[1].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(100)
    );
    assert_eq!(
        arguments[1].1.parent(),
        mizar_checker::typed_ast::TypedNodeId::new(101)
    );
    assert_eq!(
        arguments[1].1.parent_kind(),
        SourceTemplateParentKind::TermReference
    );
    assert_eq!(arguments[1].1.source_range(), ast.nodes()[100].range);
    assert_eq!(arguments[1].1.source_ordinal(), 1);
    assert_eq!(arguments[1].1.recovery(), SourceTemplateRecovery::Normal);
    let argument = handoff.arguments().iter().collect::<Vec<_>>();
    assert_eq!(argument[0].0, SourceTemplateArgumentId::new(0));
    assert_eq!(
        argument[0].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(90)
    );
    assert_eq!(argument[0].1.arguments(), SourceTemplateArgumentsId::new(0));
    assert_eq!(argument[0].1.ordinal(), 0);
    assert_eq!(argument[0].1.source_range(), ast.nodes()[90].range);
    assert_eq!(argument[0].1.source_ordinal(), 0);
    assert_eq!(argument[0].1.recovery(), SourceTemplateRecovery::Normal);
    assert_eq!(
        argument[1].1.site(),
        mizar_checker::typed_ast::TypedNodeId::new(99)
    );
    assert_eq!(argument[1].1.arguments(), SourceTemplateArgumentsId::new(1));
    assert_eq!(argument[1].1.ordinal(), 0);
    assert_eq!(argument[1].1.source_range(), ast.nodes()[99].range);
    assert_eq!(argument[1].1.source_ordinal(), 1);
    assert_eq!(argument[1].1.recovery(), SourceTemplateRecovery::Normal);
}

#[test]
fn task277a_runner_replays_deterministically_through_typed_and_resolved() {
    let (ast, module) = task277a_fixture(1);
    let first = source_template_output(&ast, module.clone(), SOURCE_TEMPLATE_TEXT)
        .expect("Task277A must select")
        .expect("first Task277A transport");
    let second = source_template_output(&ast, module, SOURCE_TEMPLATE_TEXT)
        .expect("Task277A must select on replay")
        .expect("second Task277A transport");
    assert_eq!(first.handoff, second.handoff);
    assert_eq!(first.handoff.debug_text(), second.handoff.debug_text());
    assert_eq!(first.typed_ast.debug_text(), second.typed_ast.debug_text());
    assert_eq!(first.resolved.debug_text(), second.resolved.debug_text());
    assert_eq!(first.typed_ast.source_template(), Some(&first.handoff));
    assert_eq!(first.resolved.source_template(), Some(&first.handoff));
    assert!(
        first
            .typed_ast
            .clone()
            .with_source_template(first.handoff.clone())
            .is_err()
    );
}

#[test]
fn task277a_runner_rejects_corrupt_template_edges_and_order() {
    let (ast, module) = task277a_fixture(2);
    let invalid_parent_kind =
        source_template_output_with_mutation(&ast, module.clone(), SOURCE_TEMPLATE_TEXT, |input| {
            input.loci_groups[0].parent_kind = SourceTemplateParentKind::FunctorPattern
        })
        .expect("Task277A source selection")
        .expect_err("wrong loci parent kind must fail");
    assert_eq!(
        invalid_parent_kind,
        "source template loci group 0 is invalid"
    );
    let invalid_parent_edge =
        source_template_output_with_mutation(&ast, module.clone(), SOURCE_TEMPLATE_TEXT, |input| {
            input.loci_groups[0].parent = mizar_checker::typed_ast::TypedNodeId::new(0)
        })
        .expect("Task277A source selection")
        .expect_err("non-direct loci edge must fail");
    assert_eq!(
        invalid_parent_edge,
        "source template loci group 0 is invalid"
    );
    let invalid_locus_group =
        source_template_output_with_mutation(&ast, module.clone(), SOURCE_TEMPLATE_TEXT, |input| {
            input.loci[0].loci = SourceTemplateLociId::new(1)
        })
        .expect("Task277A source selection")
        .expect_err("wrong locus group must fail");
    assert_eq!(invalid_locus_group, "source template locus 0 is invalid");
    let invalid_argument_group =
        source_template_output_with_mutation(&ast, module.clone(), SOURCE_TEMPLATE_TEXT, |input| {
            input.arguments[0].arguments = SourceTemplateArgumentsId::new(1)
        })
        .expect("Task277A source selection")
        .expect_err("wrong argument group must fail");
    assert_eq!(
        invalid_argument_group,
        "source template argument 0 is invalid"
    );
    let reordered =
        source_template_output_with_mutation(&ast, module, SOURCE_TEMPLATE_TEXT, |input| {
            input.argument_groups[1].source_ordinal = 0
        })
        .expect("Task277A source selection")
        .expect_err("reordered argument groups must fail");
    assert_eq!(
        reordered,
        "source template arguments group 1 is out of source order"
    );
}

#[test]
fn task277a_runner_stays_private_targetless_and_semantic_free() {
    let (ast, module) = task277a_fixture(3);
    let output = source_template_output(&ast, module.clone(), SOURCE_TEMPLATE_TEXT)
        .expect("Task277A must select")
        .expect("Task277A transport");
    assert_eq!(output.typed_ast.types().len(), 0);
    assert_eq!(output.typed_ast.facts().len(), 0);
    assert_eq!(output.typed_ast.coercions().len(), 0);
    assert_eq!(output.typed_ast.initial_obligations().len(), 0);
    assert_eq!(output.typed_ast.diagnostics().len(), 0);
    let changed = SOURCE_TEMPLATE_TEXT.replacen("TemplateUse", "TemplateUses", 1);
    let (changed_ast, changed_module, _, _, _) =
        task253_ast_from_source_text_with_diagnostic_count(&changed, 277_999);
    assert!(source_template_output(&changed_ast, changed_module, &changed).is_none());
    assert!(source_template_output(&ast, module, &changed).is_none());
}
