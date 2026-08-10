use mizar_checker::{
    source_template_type_parameter_association::SourceTemplateTypeParameterAssociationProducer,
    typed_ast::{
        CoercionTable, InitialObligationTable, LocalTypeContextTable, NodeRecoveryState,
        TypeDiagnosticTable, TypeFactTable, TypeTable, TypedArenaBuilder, TypedAst, TypedAstParts,
        TypedNode,
    },
};
use mizar_resolve::resolved_ast::ModuleId;

const TASK277BL_FIXTURE: &str = include_str!(
    "../../../../../../tests/miz/fail/templates/fail_template_fraenkel_over_type_param_001.miz"
);

#[test]
fn task277bl_real_fixture_builds_exact_template_type_parameter_association() {
    let (ast, module, _, _, diagnostics) =
        task253_ast_from_source_text_with_diagnostic_count(TASK277BL_FIXTURE, 277_101);
    assert_eq!(diagnostics, 0, "Task277B-L fixture parser diagnostics");
    assert_eq!(
        (ast.nodes().len(), ast.root().map(|root| root.index())),
        (57, Some(56))
    );

    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("Task277B-L fixture resolver arena should lower");
    let collection =
        mizar_resolve::names::TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
            .expect("Task277B-L collector should validate the resolver arena")
            .collect()
            .expect("Task277B-L collector should collect the real profile");
    let profile = typed_ast_from_surface_resolved_profile(&ast, module.clone(), &resolved);
    let handoff =
        SourceTemplateTypeParameterAssociationProducer::build(&collection, &profile.typed_ast)
            .expect("Task277B-L producer should associate the real profile");

    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(handoff.associations().len(), 1);
    assert_eq!(handoff.associations().iter().count(), 1);
    let association_rows = handoff.associations().iter().collect::<Vec<_>>();
    let [(association_id, association)] = association_rows.as_slice() else {
        panic!("Task277B-L fixture must produce exactly one association");
    };
    assert_eq!(association_id.index(), 0);
    assert_eq!(association.binding().index(), 0);
    let definition_block = typed_for_surface_index(&ast, &profile.typed_by_surface, 53);
    let parameter = typed_for_surface_index(&ast, &profile.typed_by_surface, 31);
    let binder = typed_for_surface_index(&ast, &profile.typed_by_surface, 2);
    let type_head = typed_for_surface_index(&ast, &profile.typed_by_surface, 39);
    let identifier = typed_for_surface_index(&ast, &profile.typed_by_surface, 21);
    assert_eq!(association.definition_block(), definition_block);
    assert_eq!(association.parameter(), parameter);
    assert_eq!(association.binder(), binder);
    assert_eq!(association.type_head(), type_head);
    assert_eq!(association.identifier(), identifier);
    assert_eq!(
        association.parameter_range(),
        mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 606,
            end: 620,
        }
    );
    assert_eq!(
        association.type_head_range(),
        mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 678,
            end: 679,
        }
    );
    assert_eq!(association.parameter_source_ordinal(), 0);
    assert_eq!(association.type_head_source_ordinal(), 0);
    assert_eq!(
        profile
            .typed_ast
            .nodes()
            .node(binder)
            .expect("Task277B-L binder typed node")
            .anchor,
        mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 610,
            end: 611,
        })
    );
    assert_eq!(
        profile
            .typed_ast
            .nodes()
            .node(definition_block)
            .expect("Task277B-L definition typed node")
            .anchor,
        mizar_session::SourceAnchor::Range(mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 593,
            end: 700,
        })
    );
}

fn typed_ast_from_surface_resolved_profile(
    ast: &mizar_syntax::SurfaceAst,
    module: ModuleId,
    resolved: &mizar_resolve::resolved_ast::SurfaceResolvedArena,
) -> TypedProfile {
    let mut builder = TypedArenaBuilder::new();
    let mut typed_by_surface = std::collections::BTreeMap::new();
    for view in ast.node_views() {
        let surface = ast.node(view.id()).expect("surface node for view");
        let resolved_node = resolved
            .resolved_node_for(view.id())
            .expect("validated resolver arena mapping");
        let typed = builder
            .push(
                TypedNode::new(
                    task277bl_typed_kind(surface),
                    mizar_session::SourceAnchor::Range(surface.range),
                )
                .with_children(
                    surface
                        .children
                        .iter()
                        .map(|child| {
                            *typed_by_surface
                                .get(child)
                                .expect("child must be mapped before its parent")
                        })
                        .collect(),
                )
                .with_recovery(if surface.recovered {
                    NodeRecoveryState::Recovered
                } else {
                    NodeRecoveryState::Normal
                })
                .with_resolved_node(resolved_node),
            )
            .expect("Task277B-L test-only typed arena node");
        assert!(typed_by_surface.insert(view.id(), typed).is_none());
    }
    let nodes = builder
        .finish(
            ast.root()
                .and_then(|root| typed_by_surface.get(&root).copied()),
        )
        .expect("Task277B-L test-only typed arena");

    assert_eq!(nodes.len(), ast.nodes().len());
    assert_eq!(typed_by_surface.len(), ast.nodes().len());
    assert_eq!(
        nodes.root(),
        ast.root()
            .and_then(|root| typed_by_surface.get(&root).copied())
    );
    for view in ast.node_views() {
        let surface = ast.node(view.id()).expect("surface node for view");
        let typed = nodes
            .node(
                *typed_by_surface
                    .get(&view.id())
                    .expect("surface node must map to a typed node"),
            )
            .expect("test-only typed node");
        assert_eq!(typed.kind.as_str(), task277bl_typed_kind(surface));
        assert_eq!(
            typed.anchor,
            mizar_session::SourceAnchor::Range(surface.range)
        );
        assert_eq!(
            typed.children,
            surface
                .children
                .iter()
                .map(|child| {
                    *typed_by_surface
                        .get(child)
                        .expect("child must map to a typed node")
                })
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            typed.recovery,
            if surface.recovered {
                NodeRecoveryState::Recovered
            } else {
                NodeRecoveryState::Normal
            }
        );
        assert_eq!(typed.resolved_node, resolved.resolved_node_for(view.id()));
    }

    let typed_ast = TypedAst::try_new(TypedAstParts {
        source_id: ast.source_id,
        module_id: module,
        resolved_root: None,
        source_context: None,
        source_type: None,
        source_attribute: None,
        nodes,
        contexts: LocalTypeContextTable::new(),
        types: TypeTable::new(),
        facts: TypeFactTable::new(),
        coercions: CoercionTable::new(),
        initial_obligations: InitialObligationTable::new(),
        diagnostics: TypeDiagnosticTable::new(),
    })
    .expect("Task277B-L test-only typed AST");
    TypedProfile {
        typed_ast,
        typed_by_surface,
    }
}

fn task277bl_typed_kind(surface: &mizar_syntax::SurfaceNode) -> String {
    match &surface.kind {
        mizar_syntax::SurfaceNodeKind::Token(token)
            if token.kind == mizar_syntax::SurfaceTokenKind::Identifier =>
        {
            "Identifier".to_owned()
        }
        _ => format!("{:?}", surface.kind),
    }
}

struct TypedProfile {
    typed_ast: TypedAst,
    typed_by_surface: std::collections::BTreeMap<
        mizar_syntax::SurfaceNodeId,
        mizar_checker::typed_ast::TypedNodeId,
    >,
}

fn typed_for_surface_index(
    ast: &mizar_syntax::SurfaceAst,
    typed_by_surface: &std::collections::BTreeMap<
        mizar_syntax::SurfaceNodeId,
        mizar_checker::typed_ast::TypedNodeId,
    >,
    index: usize,
) -> mizar_checker::typed_ast::TypedNodeId {
    let surface = ast
        .node_views()
        .find(|view| view.id().index() == index)
        .expect("frozen Task277B-L surface node");
    *typed_by_surface
        .get(&surface.id())
        .expect("frozen Task277B-L typed mapping")
}
