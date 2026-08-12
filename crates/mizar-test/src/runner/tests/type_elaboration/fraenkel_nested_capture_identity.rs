#[test]
fn task257c4c2_real_imported_fixture_links_inner_mapper_to_outer_generator() {
    let output = task257c4c1_frontend_output(TASK257C4C1_CANONICAL_SOURCE, 3);
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("C4C2 canonical imported AST");
    assert!(ast.nodes().iter().all(|node| !node.recovered));
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        output.source.package_id,
        output.source.module_path,
    );
    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("C4C2 resolver arena should lower");
    let collector = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("C4C2 collector should validate the resolver arena");
    let collection = collector.collect().expect("C4C2 collector should collect");

    assert_eq!(collection.bindings().len(), 2);
    assert_eq!(collection.uses().len(), 1);
    assert_eq!(collection.source_id(), ast.source_id);
    assert_eq!(collection.module(), &module);
    assert!(collection.debug_text().ends_with("bindings=2|uses=1"));
    let bindings = collection.bindings().iter().collect::<Vec<_>>();
    assert_eq!(bindings[0].0, FraenkelGeneratorVariableBindingId::new(0));
    assert_eq!(bindings[0].1.spelling(), "y");
    assert_eq!(bindings[0].1.segment_range().start, 102);
    assert_eq!(bindings[0].1.segment_range().end, 121);
    assert_eq!(bindings[0].1.segment_range().source_id, ast.source_id);
    assert_eq!(bindings[0].1.binder_range().start, 102);
    assert_eq!(bindings[0].1.binder_range().end, 103);
    assert_eq!(bindings[0].1.binder_range().source_id, ast.source_id);
    assert_eq!(bindings[0].1.source_ordinal(), 0);
    assert_eq!(bindings[0].1.definition_block().index(), 67);
    assert_eq!(bindings[0].1.functor_definition().index(), 66);
    assert_eq!(bindings[0].1.comprehension().index(), 51);
    assert_eq!(bindings[0].1.segment().index(), 50);
    assert_eq!(bindings[0].1.binder().index(), 15);
    assert_eq!(bindings[1].0, FraenkelGeneratorVariableBindingId::new(1));
    assert_eq!(bindings[1].1.spelling(), "x");
    assert_eq!(bindings[1].1.segment_range().start, 136);
    assert_eq!(bindings[1].1.segment_range().end, 155);
    assert_eq!(bindings[1].1.segment_range().source_id, ast.source_id);
    assert_eq!(bindings[1].1.binder_range().start, 136);
    assert_eq!(bindings[1].1.binder_range().end, 137);
    assert_eq!(bindings[1].1.binder_range().source_id, ast.source_id);
    assert_eq!(bindings[1].1.source_ordinal(), 1);
    assert_eq!(bindings[1].1.definition_block().index(), 67);
    assert_eq!(bindings[1].1.functor_definition().index(), 66);
    assert_eq!(bindings[1].1.comprehension().index(), 63);
    assert_eq!(bindings[1].1.segment().index(), 62);
    assert_eq!(bindings[1].1.binder().index(), 22);

    let use_link = collection.uses().get(0).expect("C4C2 mapper link");
    assert_eq!(use_link.definition_block().index(), 67);
    assert_eq!(use_link.functor_definition().index(), 66);
    assert_eq!(use_link.comprehension().index(), 51);
    assert_eq!(use_link.role_owner().index(), 40);
    assert_eq!(use_link.term_reference().index(), 39);
    assert_eq!(use_link.identifier().index(), 13);
    assert_eq!(
        use_link.binding(),
        FraenkelGeneratorVariableBindingId::new(1)
    );
    assert_eq!(use_link.role(), FraenkelGeneratorVariableUseRole::Mapper);
    assert_eq!(use_link.identifier_range().start, 94);
    assert_eq!(use_link.identifier_range().end, 95);
    assert_eq!(use_link.source_ordinal(), 0);
    assert_eq!(use_link.role_source_ordinal(), 0);
    assert_eq!(use_link.identifier_range().source_id, ast.source_id);
    assert_eq!(
        ast.nodes()[bindings[0].1.definition_block().index()].kind,
        SurfaceNodeKind::DefinitionBlockItem
    );
    assert_eq!(
        ast.nodes()[bindings[0].1.functor_definition().index()].kind,
        SurfaceNodeKind::FunctorDefinition
    );
    assert_eq!(
        ast.nodes()[bindings[0].1.comprehension().index()].kind,
        SurfaceNodeKind::SetComprehension
    );
    assert_eq!(
        ast.nodes()[bindings[0].1.segment().index()].kind,
        SurfaceNodeKind::ComprehensionVariableSegment
    );
    assert!(matches!(
        &ast.nodes()[bindings[0].1.binder().index()].kind,
        SurfaceNodeKind::Token(token) if token.text.as_ref() == "y"
    ));
    assert_eq!(
        ast.nodes()[bindings[1].1.comprehension().index()].kind,
        SurfaceNodeKind::SetComprehension
    );
    assert_eq!(
        ast.nodes()[bindings[1].1.segment().index()].kind,
        SurfaceNodeKind::ComprehensionVariableSegment
    );
    assert!(matches!(
        &ast.nodes()[bindings[1].1.binder().index()].kind,
        SurfaceNodeKind::Token(token) if token.text.as_ref() == "x"
    ));
    assert_eq!(
        ast.nodes()[use_link.comprehension().index()].kind,
        SurfaceNodeKind::SetComprehension
    );
    assert_eq!(
        ast.nodes()[use_link.role_owner().index()].kind,
        SurfaceNodeKind::TermExpression
    );
    assert_eq!(
        ast.nodes()[use_link.term_reference().index()].kind,
        SurfaceNodeKind::TermReference
    );
    assert!(matches!(
        &ast.nodes()[use_link.identifier().index()].kind,
        SurfaceNodeKind::Token(token) if token.text.as_ref() == "x"
    ));
    assert_eq!(
        ast.nodes()[use_link.definition_block().index()].range,
        mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 40,
            end: 163,
        }
    );
    assert_eq!(
        ast.nodes()[use_link.functor_definition().index()].range,
        mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 53,
            end: 158,
        }
    );
    assert_eq!(
        ast.nodes()[use_link.comprehension().index()].range,
        mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 92,
            end: 123,
        }
    );
    assert_eq!(
        ast.nodes()[use_link.role_owner().index()].range,
        mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 94,
            end: 95,
        }
    );
    assert_eq!(
        ast.nodes()[use_link.term_reference().index()].range,
        mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 94,
            end: 95,
        }
    );
    assert_eq!(
        ast.nodes()[use_link.identifier().index()].range,
        mizar_session::SourceRange {
            source_id: ast.source_id,
            start: 94,
            end: 95,
        }
    );
    assert_eq!(
        collection,
        collector.collect().expect("C4C2 deterministic replay")
    );

    let (type_ast, type_module, _, symbols) =
        task253_ast_from_source_text(TASK257C4C1_CANONICAL_SOURCE, 25_742);
    let augmented =
        augment_type_elaboration_import_summaries(&type_ast, &type_module, symbols.clone());
    assert_eq!(augmented, symbols);
}

#[test]
fn task257c4c3_real_imported_fixture_builds_checker_identity_handoff() {
    let output = task257c4c1_frontend_output(TASK257C4C1_CANONICAL_SOURCE, 3);
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("C4C3 canonical imported AST");
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        output.source.package_id,
        output.source.module_path,
    );
    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("C4C3 resolver arena should lower");
    let resolver = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("C4C3 collector should validate the resolver arena")
        .collect()
        .expect("C4C3 collector should collect the exact relation");
    let profile = typed_ast_from_surface_resolved_profile(&ast, module.clone(), &resolved);
    let typed_before = profile.typed_ast.clone();
    let resolver_before = resolver.clone();
    let handoff =
        mizar_checker::source_formula_composition::SourceNestedFraenkelBinderUseProducer::build(
            &resolver,
            &profile.typed_ast,
        )
        .expect("C4C3 checker identity handoff should build");

    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(
        handoff.resolver_summary(),
        format!(
            "fraenkel-generator-variable-source-v1|module={}.{}|bindings=2|uses=1",
            module.package().as_str(),
            module.path().as_str(),
        )
    );
    assert_eq!(handoff.binder_uses().len(), 1);
    assert!(!handoff.binder_uses().is_empty());
    let row = handoff
        .binder_uses()
        .get(mizar_checker::source_formula_composition::SourceNestedFraenkelBinderUseId::new(0))
        .expect("C4C3 one-row identity handoff");
    assert_eq!(row.resolver_use_index(), 0);
    assert_eq!(
        row.resolver_binding(),
        FraenkelGeneratorVariableBindingId::new(1)
    );
    assert_eq!(
        row.outer_binder(),
        typed_for_surface_index(&ast, &profile.typed_by_surface, 22)
    );
    assert_eq!(
        row.inner_mapper_use(),
        typed_for_surface_index(&ast, &profile.typed_by_surface, 13)
    );
    assert_eq!(row.source_ordinal(), 0);
    assert!(
        handoff
            .binder_uses()
            .get(mizar_checker::source_formula_composition::SourceNestedFraenkelBinderUseId::new(1))
            .is_none()
    );
    assert_eq!(
        handoff.debug_text(),
        format!(
            "source-nested-fraenkel-binder-use-v1|module={}.{}|binder-uses=1",
            module.package().as_str(),
            module.path().as_str(),
        )
    );
    assert_eq!(resolver, resolver_before);
    assert_eq!(profile.typed_ast, typed_before);
}

#[test]
fn task257c4c4_real_imported_fixture_builds_mapper_primary_handoff() {
    use mizar_checker::{
        binding_env::{
            BinderIdentity, BindingContextId, BindingKind, BindingRecoveryState, BindingStatus,
            BindingTypeSite,
        },
        source_term::{
            SourcePrimaryTermId, SourcePrimaryTermKind, SourcePrimaryTermRecovery,
            SourcePrimaryTermReferenceId, SourcePrimaryTermReferenceRole, SourcePrimaryTermRole,
        },
        typed_ast::{NodeRecoveryState, TypedNodeId, TypedSiteRef, TypingState},
    };
    use mizar_session::{SourceAnchor, SourceRange};

    let output = task257c4c1_frontend_output(TASK257C4C1_CANONICAL_SOURCE, 3);
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("C4C4 canonical imported AST");
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        output.source.package_id,
        output.source.module_path,
    );
    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("C4C4 resolver arena should lower");
    let resolver = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("C4C4 collector should validate the resolver arena")
        .collect()
        .expect("C4C4 collector should collect the exact relation");
    let profile = typed_ast_from_surface_resolved_profile(&ast, module.clone(), &resolved);
    let dependency =
        mizar_checker::source_formula_composition::SourceNestedFraenkelBinderUseProducer::build(
            &resolver,
            &profile.typed_ast,
        )
        .expect("C4C3 checker identity handoff should build");
    let handoff =
        mizar_checker::source_term::SourceNestedFraenkelMapperPrimaryProducer::build(dependency)
            .expect("C4C4 mapper primary handoff should build");

    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(
        handoff.dependency_fingerprint(),
        handoff.dependency().debug_text()
    );
    assert_eq!(
        handoff.dependency().resolver_summary(),
        format!(
            "fraenkel-generator-variable-source-v1|module={}.{}|bindings=2|uses=1",
            module.package().as_str(),
            module.path().as_str(),
        )
    );
    assert_eq!(handoff.dependency().binder_uses().len(), 1);
    let dependency_row = handoff
        .dependency()
        .binder_uses()
        .get(mizar_checker::source_formula_composition::SourceNestedFraenkelBinderUseId::new(0))
        .expect("retained C4C3 row");
    assert_eq!(dependency_row.resolver_use_index(), 0);
    assert_eq!(dependency_row.resolver_binding(), FraenkelGeneratorVariableBindingId::new(1));
    assert_eq!(
        dependency_row.outer_binder(),
        typed_for_surface_index(&ast, &profile.typed_by_surface, 22)
    );
    assert_eq!(
        dependency_row.inner_mapper_use(),
        typed_for_surface_index(&ast, &profile.typed_by_surface, 13)
    );
    assert_eq!(dependency_row.source_ordinal(), 0);
    assert_eq!(handoff.binding_env().contexts().len(), 3);
    assert_eq!(handoff.binding_env().bindings().len(), 1);
    assert!(handoff.binding_env().diagnostics().is_empty());
    let binding = handoff
        .binding_env()
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(0))
        .expect("outer x-only binding");
    assert_eq!(binding.spelling, "x");
    assert_eq!(binding.kind, BindingKind::QuantifierBinder);
    assert_eq!(
        binding.identity,
        BinderIdentity::SourceBound {
            context: BindingContextId::new(1),
            ordinal: 0,
        }
    );
    assert_eq!(binding.owner_context, BindingContextId::new(1));
    assert_eq!(
        binding.declaration_range,
        SourceRange {
            source_id: ast.source_id,
            start: 136,
            end: 137,
        }
    );
    assert_eq!(binding.visible_after_ordinal, 0);
    assert_eq!(
        binding.type_site,
        BindingTypeSite::Source(SourceRange {
            source_id: ast.source_id,
            start: 141,
            end: 155,
        })
    );
    assert_eq!(binding.status, BindingStatus::Active);
    assert!(binding.captured.identities().is_empty());
    assert!(binding.diagnostics.is_empty());
    assert_eq!(binding.recovery, BindingRecoveryState::Normal);
    assert_eq!(handoff.projection_arena().len(), 1);
    assert_eq!(
        handoff.projection_arena().root(),
        Some(mizar_checker::typed_ast::TypedNodeId::new(0))
    );
    let node = handoff
        .projection_arena()
        .node(TypedNodeId::new(0))
        .expect("one projection node");
    assert_eq!(node.kind.as_str(), "source.term.variable-reference");
    assert_eq!(
        node.anchor,
        SourceAnchor::Range(SourceRange {
            source_id: ast.source_id,
            start: 94,
            end: 95,
        })
    );
    assert!(node.resolved_node.is_none());
    assert!(node.children.is_empty());
    assert_eq!(node.typing, TypingState::Unknown);
    assert_eq!(node.recovery, NodeRecoveryState::Normal);
    assert_eq!(node.links, Default::default());
    assert_eq!(handoff.source_term().terms().len(), 1);
    assert_eq!(handoff.source_term().references().len(), 1);
    assert!(handoff.source_term().numeric_type_requests().is_empty());
    let term = handoff
        .source_term()
        .terms()
        .get(SourcePrimaryTermId::new(0))
        .expect("one mapper term");
    assert_eq!(term.site(), &TypedSiteRef::Node(TypedNodeId::new(0)));
    assert_eq!(term.spelling(), "x");
    assert_eq!(
        term.source_range(),
        SourceRange {
            source_id: ast.source_id,
            start: 94,
            end: 95,
        }
    );
    assert_eq!(term.source_ordinal(), 0);
    assert_eq!(term.context(), BindingContextId::new(2));
    assert_eq!(term.recovery(), SourcePrimaryTermRecovery::Normal);
    assert_eq!(term.kind(), SourcePrimaryTermKind::VariableReference);
    assert_eq!(term.role(), SourcePrimaryTermRole::Value);
    assert_eq!(term.parent(), None);
    let reference = handoff
        .source_term()
        .references()
        .get(SourcePrimaryTermReferenceId::new(0))
        .expect("one outer x reference");
    assert_eq!(reference.term(), SourcePrimaryTermId::new(0));
    assert_eq!(reference.binding(), mizar_checker::binding_env::BindingId::new(0));
    assert_eq!(reference.role(), SourcePrimaryTermReferenceRole::Variable);
    assert_eq!(reference.use_ordinal(), 1);
    assert!(reference.lexical_scope().is_none());
}
