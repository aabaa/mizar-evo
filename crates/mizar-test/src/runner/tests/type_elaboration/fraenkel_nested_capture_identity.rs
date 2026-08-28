const TASK257C4C8R_CANONICAL_SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/miz/pass/types/",
    "pass_types_nested_comprehension_two_outer_generator_captures_001.miz"
));

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
fn task257c4c8r_real_imported_fixture_links_both_outer_generators() {
    let output = task257c4c1_frontend_output(TASK257C4C8R_CANONICAL_SOURCE, 4);
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("C4C8R canonical imported AST");
    assert!(ast.nodes().iter().all(|node| !node.recovered));
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        output.source.package_id,
        output.source.module_path,
    );
    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("C4C8R resolver arena should lower");
    let collector = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("C4C8R collector should validate the resolver arena");
    let collection = collector.collect().expect("C4C8R collector should collect");

    assert_eq!(collection.bindings().len(), 3);
    assert_eq!(collection.uses().len(), 2);
    assert_eq!(collection.source_id(), ast.source_id);
    assert_eq!(collection.module(), &module);
    assert!(collection.debug_text().ends_with("bindings=3|uses=2"));
    let bindings = collection.bindings().iter().collect::<Vec<_>>();
    assert_eq!(bindings[0].0, FraenkelGeneratorVariableBindingId::new(0));
    assert_eq!(bindings[0].1.spelling(), "z");
    assert_eq!(
        (bindings[0].1.segment_range().start, bindings[0].1.segment_range().end),
        (110, 129)
    );
    assert_eq!(
        (bindings[0].1.binder_range().start, bindings[0].1.binder_range().end),
        (110, 111)
    );
    assert_eq!(bindings[0].1.source_ordinal(), 0);
    assert_eq!(bindings[1].0, FraenkelGeneratorVariableBindingId::new(1));
    assert_eq!(bindings[1].1.spelling(), "x");
    assert_eq!(
        (bindings[1].1.segment_range().start, bindings[1].1.segment_range().end),
        (144, 163)
    );
    assert_eq!(
        (bindings[1].1.binder_range().start, bindings[1].1.binder_range().end),
        (144, 145)
    );
    assert_eq!(bindings[1].1.source_ordinal(), 1);
    assert_eq!(bindings[2].0, FraenkelGeneratorVariableBindingId::new(2));
    assert_eq!(bindings[2].1.spelling(), "y");
    assert_eq!(
        (bindings[2].1.segment_range().start, bindings[2].1.segment_range().end),
        (165, 184)
    );
    assert_eq!(
        (bindings[2].1.binder_range().start, bindings[2].1.binder_range().end),
        (165, 166)
    );
    assert_eq!(bindings[2].1.source_ordinal(), 2);
    assert_ne!(
        bindings[0].1.comprehension(),
        bindings[1].1.comprehension()
    );
    assert_eq!(
        bindings[1].1.comprehension(),
        bindings[2].1.comprehension()
    );

    let links = collection.uses().iter().collect::<Vec<_>>();
    assert_eq!(links[0].binding(), FraenkelGeneratorVariableBindingId::new(1));
    assert_eq!(links[0].role(), FraenkelGeneratorVariableUseRole::Mapper);
    assert_eq!(
        (links[0].identifier_range().start, links[0].identifier_range().end),
        (98, 99)
    );
    assert_eq!(
        (links[0].source_ordinal(), links[0].role_source_ordinal()),
        (0, 0)
    );
    assert_eq!(links[1].binding(), FraenkelGeneratorVariableBindingId::new(2));
    assert_eq!(links[1].role(), FraenkelGeneratorVariableUseRole::Mapper);
    assert_eq!(
        (links[1].identifier_range().start, links[1].identifier_range().end),
        (101, 102)
    );
    assert_eq!(
        (links[1].source_ordinal(), links[1].role_source_ordinal()),
        (1, 1)
    );
    assert!(links.iter().all(|link| {
        link.comprehension() == bindings[0].1.comprehension()
            && link.role_owner() != link.term_reference()
            && matches!(
                ast.nodes()[link.role_owner().index()].kind,
                SurfaceNodeKind::TermExpression
            )
            && matches!(
                ast.nodes()[link.term_reference().index()].kind,
                SurfaceNodeKind::TermReference
            )
    }));
    assert_eq!(
        collection,
        collector.collect().expect("C4C8R deterministic replay")
    );

    let (type_ast, type_module, _, symbols) =
        task253_ast_from_source_text(TASK257C4C8R_CANONICAL_SOURCE, 25_743);
    let augmented =
        augment_type_elaboration_import_summaries(&type_ast, &type_module, symbols.clone());
    assert_eq!(augmented, symbols);
}

#[test]
fn task257c4c8_real_imported_fixture_builds_exact_normalized_capture_graph() {
    let output = task257c4c1_frontend_output(TASK257C4C8R_CANONICAL_SOURCE, 4);
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("C4C8 canonical imported AST");
    assert!(ast.nodes().iter().all(|node| !node.recovered));
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        output.source.package_id,
        output.source.module_path,
    );
    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("C4C8 resolver arena should lower");
    let resolver = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("C4C8 collector should validate the resolver arena")
        .collect()
        .expect("C4C8 collector should collect");
    let resolver_before = resolver.clone();
    let graph =
        mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphProducer::build(
            &resolver,
        )
        .expect("C4C8 graph producer should build");

    assert_eq!(graph.source_id(), ast.source_id);
    assert_eq!(graph.module_id(), &module);
    assert_eq!(graph.resolver_summary(), resolver.debug_text());
    assert_eq!(graph.generators().len(), 3);
    assert_eq!(graph.mappers().len(), 1);
    assert_eq!(graph.predicates().len(), 0);
    assert_eq!(graph.captures().len(), 2);
    assert_eq!(graph.occurrences().len(), 2);
    assert!(graph.predicates().is_empty());

    let generator_rows = graph.generators().iter().collect::<Vec<_>>();
    let resolver_bindings = resolver.bindings().iter().collect::<Vec<_>>();
    assert_eq!(generator_rows.len(), resolver_bindings.len());
    for (index, ((graph_id, graph_row), (resolver_id, resolver_row))) in generator_rows
        .iter()
        .zip(resolver_bindings.iter())
        .enumerate()
    {
        assert_eq!(graph_id.index(), index);
        assert_eq!(graph_row.resolver_binding(), *resolver_id);
        assert_eq!(
            graph_row.definition_block(),
            resolver_row.definition_block()
        );
        assert_eq!(
            graph_row.functor_definition(),
            resolver_row.functor_definition()
        );
        assert_eq!(graph_row.comprehension(), resolver_row.comprehension());
        assert_eq!(graph_row.segment(), resolver_row.segment());
        assert_eq!(graph_row.binder(), resolver_row.binder());
        assert_eq!(graph_row.segment_range(), resolver_row.segment_range());
        assert_eq!(graph_row.binder_range(), resolver_row.binder_range());
    }

    let links = resolver.uses().iter().collect::<Vec<_>>();
    let mapper = graph
        .mappers()
        .get(mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphMapperId::new(0))
        .expect("C4C8 mapper row");
    assert_eq!(mapper.definition_block(), links[0].definition_block());
    assert_eq!(mapper.functor_definition(), links[0].functor_definition());
    assert_eq!(mapper.comprehension(), links[0].comprehension());
    assert_eq!(mapper.owner(), links[0].role_owner());

    let captures = graph.captures().iter().collect::<Vec<_>>();
    assert_eq!(captures[0].1.generator().index(), 1);
    assert_eq!(captures[1].1.generator().index(), 2);
    assert_eq!(captures[0].1.resolver_binding(), links[0].binding());
    assert_eq!(captures[1].1.resolver_binding(), links[1].binding());
    assert_ne!(
        captures[0].1.resolver_binding(),
        mizar_resolve::names::FraenkelGeneratorVariableBindingId::new(0)
    );
    assert_ne!(
        captures[1].1.resolver_binding(),
        mizar_resolve::names::FraenkelGeneratorVariableBindingId::new(0)
    );

    let occurrences = graph.occurrences().iter().collect::<Vec<_>>();
    for (index, ((graph_id, occurrence), link)) in occurrences.iter().zip(links.iter()).enumerate()
    {
        assert_eq!(graph_id.index(), index);
        assert_eq!(occurrence.resolver_use_index(), index);
        assert_eq!(occurrence.resolver_binding(), link.binding());
        assert_eq!(occurrence.comprehension(), link.comprehension());
        assert_eq!(occurrence.role_owner(), link.role_owner());
        assert_eq!(occurrence.term_reference(), link.term_reference());
        assert_eq!(occurrence.identifier(), link.identifier());
        assert_eq!(occurrence.role(), link.role());
        assert_eq!(occurrence.identifier_range(), link.identifier_range());
    }
    assert_eq!(resolver, resolver_before);

    let replay =
        mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphProducer::build(
            &resolver,
        )
        .expect("C4C8 graph deterministic replay");
    assert!(graph == replay);
    assert_eq!(graph.debug_text(), replay.debug_text());

    let (type_ast, type_module, _, symbols) =
        task253_ast_from_source_text(TASK257C4C8R_CANONICAL_SOURCE, 25_743);
    let augmented =
        augment_type_elaboration_import_summaries(&type_ast, &type_module, symbols.clone());
    assert_eq!(augmented, symbols);
}

#[test]
fn task33r_real_fixture_links_capture_graph_to_exact_functor_owner() {
    let output = task257c4c1_frontend_output(TASK257C4C8R_CANONICAL_SOURCE, 4);
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("Task33R canonical C4C7 AST");
    assert!(ast.nodes().iter().all(|node| !node.recovered));
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        output.source.package_id,
        output.source.module_path,
    );
    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("Task33R resolver arena should lower");
    let resolver = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("Task33R collector should validate the resolver arena")
        .collect()
        .expect("Task33R collector should collect the exact relation");
    let resolver_before = resolver.clone();
    let graph =
        mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphProducer::build(
            &resolver,
        )
        .expect("Task33R C4C8 graph should build");
    let graph_before = graph.clone();

    let owner = mizar_resolve::symbols::SourceNestedFraenkelFunctorOwnerProducer::build(
        &ast, &module, &resolved, &resolver,
    )
    .expect("Task33R containing-functor owner should build");
    owner
        .validate_complete()
        .expect("Task33R owner complete validation");
    owner
        .validate_resolver_collection(&resolver)
        .expect("Task33R owner exact resolver validation");

    assert_eq!(owner.source_id(), ast.source_id);
    assert_eq!(owner.module_id(), &module);
    assert_eq!(owner.surface_fingerprint(), ast.snapshot_text());

    let mapper = graph
        .mappers()
        .get(mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphMapperId::new(0))
        .expect("Task33R C4C8 mapper row");
    assert_eq!(owner.definition_block(), mapper.definition_block());
    assert_eq!(owner.functor_definition(), mapper.functor_definition());

    let shells = mizar_resolve::declarations::DeclarationShellCollector::new(&ast, &module)
        .collect();
    let owner_shell = shells
        .declaration(owner.declaration_shell())
        .expect("Task33R owner declaration shell");
    assert_eq!(
        owner_shell.kind(),
        mizar_resolve::declarations::DeclarationShellKind::FunctorDefinition
    );
    assert_eq!(owner_shell.module(), &module);
    assert!(!owner_shell.recovered());
    assert_eq!(
        resolved.resolved_node_for(owner_shell.node_id()),
        Some(owner.functor_definition())
    );

    let projections = mizar_resolve::symbols::SignatureProjectionExtractor::new(
        &ast,
        &shells,
        mizar_resolve::env::NamespacePath::new(module.path().as_str()),
    )
    .extract();
    let symbol_result = mizar_resolve::symbols::SymbolCollector::new(
        ast.source_id,
        &module,
        &shells,
        &projections,
    )
    .collect();
    assert!(symbol_result.diagnostics().is_empty());
    let symbol_entry = symbol_result
        .env()
        .symbols()
        .get(owner.symbol())
        .expect("Task33R final functor symbol");
    assert_eq!(symbol_entry.kind(), mizar_resolve::env::SymbolKind::Functor);
    assert_eq!(symbol_entry.origin(), owner.origin());
    assert_eq!(symbol_entry.contribution(), owner.contribution());
    let definition_entry = symbol_result
        .env()
        .definitions()
        .get(owner.definition())
        .expect("Task33R final functor definition");
    assert_eq!(
        definition_entry.kind(),
        mizar_resolve::env::DefinitionKind::Functor
    );
    assert_eq!(definition_entry.symbol(), owner.symbol());
    assert_eq!(definition_entry.origin(), owner.origin());
    assert_eq!(definition_entry.contribution(), owner.contribution());
    assert_eq!(owner.origin().source_id(), ast.source_id);
    assert_eq!(owner.origin().module_id(), &module);
    assert!(!owner.origin().is_recovered());

    assert!(graph == graph_before);
    assert_eq!(resolver, resolver_before);
    let replay = mizar_resolve::symbols::SourceNestedFraenkelFunctorOwnerProducer::build(
        &ast, &module, &resolved, &resolver,
    )
    .expect("Task33R deterministic owner replay");
    assert!(owner == replay);
    assert_eq!(owner.debug_text(), replay.debug_text());

    let (type_ast, type_module, _, symbols) =
        task253_ast_from_source_text(TASK257C4C8R_CANONICAL_SOURCE, 25_743);
    let augmented =
        augment_type_elaboration_import_summaries(&type_ast, &type_module, symbols.clone());
    assert_eq!(augmented, symbols);
}

#[test]
fn task33c_real_fixture_pairs_capture_graph_with_exact_functor_owner() {
    let output = task257c4c1_frontend_output(TASK257C4C8R_CANONICAL_SOURCE, 4);
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("Task33C canonical AST");
    assert!(ast.nodes().iter().all(|node| !node.recovered));
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        output.source.package_id,
        output.source.module_path,
    );
    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("Task33C resolver arena should lower");
    let resolver = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("Task33C collector should validate the resolver arena")
        .collect()
        .expect("Task33C collector should collect the exact relation");
    let graph =
        mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphProducer::build(
            &resolver,
        )
        .expect("Task33C graph should build");
    let owner = mizar_resolve::symbols::SourceNestedFraenkelFunctorOwnerProducer::build(
        &ast, &module, &resolved, &resolver,
    )
    .expect("Task33C owner should build");
    let graph_before = graph.clone();
    let owner_before = owner.clone();
    let receipt = mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphOwnerProducer::build(
        graph.clone(),
        owner.clone(),
    )
    .expect("Task33C graph-owner receipt should build");

    assert_eq!(receipt.source_id(), ast.source_id);
    assert_eq!(receipt.module_id(), &module);
    assert!(receipt.graph() == &graph);
    assert!(receipt.owner() == &owner);
    assert_eq!(receipt.graph().captures().len(), 2);
    assert_eq!(receipt.graph().occurrences().len(), 2);
    assert!(
        receipt
            .graph()
            .captures()
            .iter()
            .all(|(_, capture)| capture.generator().index() != 0)
    );
    assert!(receipt.graph().generators().iter().all(|(_, row)| {
        row.definition_block() == receipt.owner().definition_block()
            && row.functor_definition() == receipt.owner().functor_definition()
    }));
    assert!(receipt.graph().mappers().iter().all(|(_, row)| {
        row.definition_block() == receipt.owner().definition_block()
            && row.functor_definition() == receipt.owner().functor_definition()
    }));
    assert!(receipt.graph().predicates().iter().all(|(_, row)| {
        row.definition_block() == receipt.owner().definition_block()
            && row.functor_definition() == receipt.owner().functor_definition()
    }));
    assert!(graph == graph_before);
    assert!(owner == owner_before);

    let replay = mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphOwnerProducer::build(
        graph,
        owner,
    )
    .expect("Task33C graph-owner replay should build");
    assert!(receipt == replay);
    assert_eq!(receipt.debug_text(), replay.debug_text());

    let (type_ast, type_module, _, symbols) =
        task253_ast_from_source_text(TASK257C4C8R_CANONICAL_SOURCE, 25_743);
    let augmented =
        augment_type_elaboration_import_summaries(&type_ast, &type_module, symbols.clone());
    assert_eq!(augmented, symbols);
}

fn task33c4c8_real_receipt(
) -> mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphOwnerHandoff {
    let output = task257c4c1_frontend_output(TASK257C4C8R_CANONICAL_SOURCE, 4);
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("Task33C4C8 canonical AST");
    assert!(ast.nodes().iter().all(|node| !node.recovered));
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        output.source.package_id,
        output.source.module_path,
    );
    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("Task33C4C8 resolver arena should lower");
    let resolver = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("Task33C4C8 collector should validate the resolver arena")
        .collect()
        .expect("Task33C4C8 collector should collect the exact relation");
    let graph =
        mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphProducer::build(
            &resolver,
        )
        .expect("Task33C4C8 graph should build");
    let owner = mizar_resolve::symbols::SourceNestedFraenkelFunctorOwnerProducer::build(
        &ast, &module, &resolved, &resolver,
    )
    .expect("Task33C4C8 owner should build");
    mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphOwnerProducer::build(
        graph, owner,
    )
    .expect("Task33C4C8 graph-owner receipt should build")
}

fn task33c4c8_core_context(
    receipt: &mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphOwnerHandoff,
    module: mizar_resolve::resolved_ast::ModuleId,
    owner_kind: Option<mizar_core::core_ir::CoreItemKind>,
    existing_vars: &[usize],
) -> mizar_core::elaborator::CoreContext {
    task33c4c8_core_context_with_existing_role(
        receipt,
        module,
        owner_kind,
        existing_vars,
        "existing-term",
    )
}

fn task33c4c8_core_context_with_existing_role(
    receipt: &mizar_checker::source_formula_composition::SourceNestedFraenkelCaptureGraphOwnerHandoff,
    module: mizar_resolve::resolved_ast::ModuleId,
    owner_kind: Option<mizar_core::core_ir::CoreItemKind>,
    existing_vars: &[usize],
    existing_role: &str,
) -> mizar_core::elaborator::CoreContext {
    use mizar_core::{
        binder_normalization::{NormalizedVarClass, NormalizedVarSort},
        core_ir::{CoreSourceRef, CoreVarId, GeneratedOriginKind},
        elaborator::{
            CheckerOwnedProvenance, CoreBinderSeed, CoreContextInput, CoreItemSeed,
            CoreVariableSeed, GeneratedOriginSeed, ResolvedTypedAstSummary, prepare_core_context,
        },
    };

    let mut input = CoreContextInput::new(ResolvedTypedAstSummary::new(
        receipt.source_id(),
        module,
    ));
    let owner_range = match receipt.owner().origin().anchor() {
        mizar_session::SourceAnchor::Range(range) => *range,
        other => panic!("Task33C4C8 owner needs a source range, found {other:?}"),
    };
    if let Some(kind) = owner_kind {
        input.item_seeds.push(CoreItemSeed::new(
            receipt.owner().symbol().clone(),
            kind,
            "public",
            CoreSourceRef::direct(owner_range),
            CheckerOwnedProvenance::resolver("task33c4c8.owner"),
        ));
    }
    for &index in existing_vars {
        let var = CoreVarId::new(index);
        let provenance =
            CheckerOwnedProvenance::checker(format!("task33c4c8.existing.{index}"));
        input.variable_seeds.push(CoreVariableSeed::new(
            var,
            NormalizedVarClass::Free,
            existing_role,
            NormalizedVarSort::Term,
            provenance.clone(),
        ));
        input.binder_seeds.push(CoreBinderSeed::new(
            var,
            CoreSourceRef::direct(owner_range)
                .with_provenance(provenance.as_slice().to_vec()),
            provenance,
        ));
    }
    if let Some(&index) = existing_vars.iter().max() {
        input.generated_origin_seeds.push(
            GeneratedOriginSeed::new(
                receipt.owner().symbol().clone(),
                GeneratedOriginKind::StableChoice,
                "task33c4c8-existing-generated-origin",
                CoreSourceRef::direct(owner_range),
                CheckerOwnedProvenance::checker("task33c4c8.existing.generated-origin"),
            )
            .with_params(vec![CoreVarId::new(index)]),
        );
    }
    prepare_core_context(input).expect("Task33C4C8 Core context should prepare")
}

#[test]
fn task33c4c8_real_fixture_installs_exact_capture_variables_deterministically() {
    use mizar_core::{
        binder_normalization::{NormalizedVarClass, NormalizedVarSort},
        core_ir::{
            CoreProvenance, CoreProvenanceKey, CoreProvenancePhase, CoreSourceAnchor, CoreVarId,
        },
        elaborator::SourceNestedFraenkelCaptureCoreContextProducer,
    };

    let receipt = task33c4c8_real_receipt();
    let context = task33c4c8_core_context(
        &receipt,
        receipt.module_id().clone(),
        Some(mizar_core::core_ir::CoreItemKind::Functor),
        &[],
    );
    let original_item_registry = context.item_registry().clone();
    let original_dependencies = context.dependency_summaries().clone();
    let original_boundaries = context.definition_boundaries().clone();
    let original_generated_origins = context.generated_origins().clone();
    let original_source_map = context.source_map().clone();
    let original_diagnostics = context.diagnostics().clone();
    let original_worklist = context.worklist().clone();
    let replay = SourceNestedFraenkelCaptureCoreContextProducer::build(
        context.clone(),
        receipt.clone(),
    )
    .expect("Task33C4C8 deterministic replay should build");
    let handoff = SourceNestedFraenkelCaptureCoreContextProducer::build(context, receipt)
        .expect("Task33C4C8 capture context should build");
    assert!(handoff == replay);
    assert_eq!(handoff.captured_variables().len(), 2);
    assert!(!handoff.captured_variables().is_empty());
    assert_eq!(handoff.context().generated_origins().table().len(), 0);
    assert_eq!(handoff.context().item_registry(), &original_item_registry);
    assert_eq!(handoff.context().dependency_summaries(), &original_dependencies);
    assert_eq!(handoff.context().definition_boundaries(), &original_boundaries);
    assert_eq!(handoff.context().generated_origins(), &original_generated_origins);
    assert_eq!(handoff.context().source_map(), &original_source_map);
    assert_eq!(handoff.context().diagnostics(), &original_diagnostics);
    assert_eq!(handoff.context().worklist(), &original_worklist);

    let rows = handoff.captured_variables().iter().collect::<Vec<_>>();
    assert_eq!(rows[0].0.index(), 0);
    assert_eq!(rows[1].0.index(), 1);
    assert_eq!(rows[0].1.core_var(), CoreVarId::new(0));
    assert_eq!(rows[1].1.core_var(), CoreVarId::new(1));
    assert_eq!(rows[0].1.generator().index(), 1);
    assert_eq!(rows[1].1.generator().index(), 2);
    assert_eq!(rows[0].1.resolver_binding().index(), 1);
    assert_eq!(rows[1].1.resolver_binding().index(), 2);
    assert_ne!(rows[0].1.generator().index(), 0);
    assert_ne!(rows[1].1.generator().index(), 0);

    for (capture_id, row) in rows {
        assert_eq!(capture_id, row.capture());
        assert_eq!(
            handoff
                .context()
                .binder_context()
                .variable_classes
                .get(&row.core_var()),
            Some(&NormalizedVarClass::Free)
        );
        assert_eq!(
            handoff
                .context()
                .binder_context()
                .variable_sorts
                .get(&row.core_var()),
            Some(&NormalizedVarSort::Term)
        );
        assert_eq!(
            handoff
                .context()
                .binder_context()
                .variable_roles
                .get(&row.core_var())
                .map(|role| role.as_str()),
            Some("fraenkel-captured-parameter")
        );
        assert!(handoff.context().binder_type_facts()[&row.core_var()].is_empty());
        let source = handoff
            .context()
            .binder_sources()
            .get(row.core_var())
            .expect("Task33C4C8 installed binder source");
        let generator = handoff
            .checker_receipt()
            .graph()
            .generators()
            .get(row.generator())
            .expect("Task33C4C8 retained generator");
        assert_eq!(
            source.source.anchor,
            CoreSourceAnchor::SourceRange(generator.binder_range())
        );
        let expected_key = CoreProvenanceKey::new(format!(
            "source-nested-fraenkel-capture-core-variable-v1.capture.{}",
            capture_id.index()
        ));
        let expected_provenance = CoreProvenance::new(
            CoreProvenancePhase::Checker,
            expected_key,
        );
        assert_eq!(source.source.provenance, vec![expected_provenance.clone()]);
        assert_eq!(source.provenance.as_slice(), [expected_provenance]);
    }
    assert_eq!(
        handoff
            .context()
            .item_registry()
            .id_for_symbol(handoff.checker_receipt().owner().symbol()),
        Some(handoff.owner_item())
    );
    assert!(handoff.debug_text().contains("captures=2|vars=0,1"));
}

#[test]
fn task33c4c8_allocation_starts_above_existing_core_variables() {
    use mizar_core::{
        core_ir::CoreVarId,
        elaborator::SourceNestedFraenkelCaptureCoreContextProducer,
    };

    let receipt = task33c4c8_real_receipt();
    let context = task33c4c8_core_context(
        &receipt,
        receipt.module_id().clone(),
        Some(mizar_core::core_ir::CoreItemKind::Functor),
        &[2, 9],
    );
    let original_generated_origins = context.generated_origins().clone();
    let original_source_map = context.source_map().clone();
    let handoff = SourceNestedFraenkelCaptureCoreContextProducer::build(context, receipt)
        .expect("Task33C4C8 populated context should build");
    let vars = handoff
        .captured_variables()
        .iter()
        .map(|(_, row)| row.core_var())
        .collect::<Vec<_>>();
    assert_eq!(vars, [CoreVarId::new(10), CoreVarId::new(11)]);
    assert_eq!(handoff.context().generated_origins(), &original_generated_origins);
    assert_eq!(handoff.context().source_map(), &original_source_map);
    let (_, existing_origin) = handoff
        .context()
        .generated_origins()
        .table()
        .iter()
        .next()
        .expect("pre-existing generated origin is retained");
    assert_eq!(existing_origin.params, vec![CoreVarId::new(9)]);
}

#[test]
fn task33c4c8_rejects_environment_and_owner_mismatches() {
    use mizar_core::elaborator::{
        SourceNestedFraenkelCaptureCoreContextError,
        SourceNestedFraenkelCaptureCoreContextProducer,
    };

    let receipt = task33c4c8_real_receipt();
    let foreign_module = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task33c4c8-foreign"),
        mizar_session::ModulePath::new("foreign.module"),
    );
    let foreign = task33c4c8_core_context(&receipt, foreign_module, None, &[]);
    assert!(matches!(
        SourceNestedFraenkelCaptureCoreContextProducer::build(foreign, receipt.clone()),
        Err(SourceNestedFraenkelCaptureCoreContextError::EnvironmentMismatch)
    ));

    let missing = task33c4c8_core_context(
        &receipt,
        receipt.module_id().clone(),
        None,
        &[],
    );
    assert!(matches!(
        SourceNestedFraenkelCaptureCoreContextProducer::build(missing, receipt.clone()),
        Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation)
    ));

    let wrong_kind = task33c4c8_core_context(
        &receipt,
        receipt.module_id().clone(),
        Some(mizar_core::core_ir::CoreItemKind::Predicate),
        &[],
    );
    assert!(matches!(
        SourceNestedFraenkelCaptureCoreContextProducer::build(wrong_kind, receipt),
        Err(SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation)
    ));
}

#[test]
fn task33c4c8_rejects_invalid_core_context_before_owner_association() {
    use mizar_core::elaborator::{
        SourceNestedFraenkelCaptureCoreContextError,
        SourceNestedFraenkelCaptureCoreContextProducer,
    };

    let receipt = task33c4c8_real_receipt();
    let invalid = task33c4c8_core_context_with_existing_role(
        &receipt,
        receipt.module_id().clone(),
        Some(mizar_core::core_ir::CoreItemKind::Functor),
        &[7],
        "fraenkel-captured-parameter",
    );
    assert!(matches!(
        SourceNestedFraenkelCaptureCoreContextProducer::build(invalid, receipt.clone()),
        Err(SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext)
    ));

    let foreign_module = mizar_resolve::resolved_ast::ModuleId::new(
        mizar_session::PackageId::new("task33c4c8-invalid-foreign"),
        mizar_session::ModulePath::new("foreign.invalid"),
    );
    let invalid_foreign = task33c4c8_core_context_with_existing_role(
        &receipt,
        foreign_module,
        None,
        &[7],
        "fraenkel-captured-parameter",
    );
    assert!(matches!(
        SourceNestedFraenkelCaptureCoreContextProducer::build(invalid_foreign, receipt),
        Err(SourceNestedFraenkelCaptureCoreContextError::EnvironmentMismatch)
    ));
}

#[test]
fn task33c4c8_rejects_allocator_overflow_and_has_stable_error_text() {
    use mizar_core::{
        core_ir::CoreVarId,
        elaborator::{
            SourceNestedFraenkelCaptureCoreContextError,
            SourceNestedFraenkelCaptureCoreContextProducer,
        },
    };

    let receipt = task33c4c8_real_receipt();
    let context = task33c4c8_core_context(
        &receipt,
        receipt.module_id().clone(),
        Some(mizar_core::core_ir::CoreItemKind::Functor),
        &[usize::MAX],
    );
    assert!(matches!(
        SourceNestedFraenkelCaptureCoreContextProducer::build(context, receipt),
        Err(SourceNestedFraenkelCaptureCoreContextError::CoreVariableAllocationOverflow)
    ));

    let cases = [
        (
            SourceNestedFraenkelCaptureCoreContextError::EnvironmentMismatch,
            "nested Fraenkel capture Core context environment is invalid".to_owned(),
        ),
        (
            SourceNestedFraenkelCaptureCoreContextError::InvalidCoreContext,
            "nested Fraenkel capture Core context is invalid".to_owned(),
        ),
        (
            SourceNestedFraenkelCaptureCoreContextError::InvalidOwnerAssociation,
            "nested Fraenkel capture Core owner association is invalid".to_owned(),
        ),
        (
            SourceNestedFraenkelCaptureCoreContextError::CoreVariableAllocationOverflow,
            "nested Fraenkel capture Core variable allocation overflowed".to_owned(),
        ),
        (
            SourceNestedFraenkelCaptureCoreContextError::CoreVariableCollision {
                var: CoreVarId::new(7),
            },
            "nested Fraenkel capture Core variable 7 collides".to_owned(),
        ),
        (
            SourceNestedFraenkelCaptureCoreContextError::InvalidCaptureAssociation,
            "nested Fraenkel capture Core association is invalid".to_owned(),
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
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

#[test]
fn task257c4c5_real_imported_fixture_builds_capture_identity_handoff() {
    use mizar_checker::{
        binding_env::{BindingContextId, BindingId},
        source_formula_composition::{
            SourceNestedFraenkelBinderUseProducer, SourceNestedFraenkelCaptureIdentityId,
            SourceNestedFraenkelCaptureIdentityProducer,
        },
        source_term::SourceNestedFraenkelMapperPrimaryProducer,
    };
    use mizar_session::SourceRange;

    let output = task257c4c1_frontend_output(TASK257C4C1_CANONICAL_SOURCE, 3);
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("C4C5 canonical imported AST");
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        output.source.package_id,
        output.source.module_path,
    );
    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("C4C5 resolver arena should lower");
    let resolver = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("C4C5 collector should validate the resolver arena")
        .collect()
        .expect("C4C5 collector should collect the exact relation");
    let profile = typed_ast_from_surface_resolved_profile(&ast, module.clone(), &resolved);
    let c4c3 = SourceNestedFraenkelBinderUseProducer::build(&resolver, &profile.typed_ast)
        .expect("C4C3 checker identity handoff should build");
    let c4c4 = SourceNestedFraenkelMapperPrimaryProducer::build(c4c3)
        .expect("C4C4 mapper primary handoff should build");
    let expected_dependency = c4c4.debug_text();
    let handoff = SourceNestedFraenkelCaptureIdentityProducer::build(c4c4)
        .expect("C4C5 capture identity handoff should build");

    assert_eq!(handoff.source_id(), ast.source_id);
    assert_eq!(handoff.module_id(), &module);
    assert_eq!(handoff.dependency_fingerprint(), expected_dependency);
    assert_eq!(handoff.identities().len(), 1);
    assert!(handoff
        .identities()
        .get(SourceNestedFraenkelCaptureIdentityId::new(1))
        .is_none());
    let identity = handoff
        .identities()
        .get(SourceNestedFraenkelCaptureIdentityId::new(0))
        .expect("sole C4C5 identity");
    assert_eq!(identity.owner_context(), BindingContextId::new(2));
    assert_eq!(
        identity.owner_range(),
        SourceRange {
            source_id: ast.source_id,
            start: 92,
            end: 123,
        }
    );
    assert_eq!(
        identity.mapper_term(),
        mizar_checker::source_term::SourcePrimaryTermId::new(0)
    );
    assert_eq!(
        identity.mapper_reference(),
        mizar_checker::source_term::SourcePrimaryTermReferenceId::new(0)
    );
    assert_eq!(identity.projected_binding(), BindingId::new(0));
    assert_eq!(identity.resolver_use_index(), 0);
    assert_eq!(
        identity.resolver_binding(),
        FraenkelGeneratorVariableBindingId::new(1)
    );
    assert_eq!(identity.source_ordinal(), 0);
    assert_eq!(
        handoff
            .dependency()
            .source_term()
            .references()
            .get(identity.mapper_reference())
            .expect("retained mapper reference")
            .binding(),
        identity.projected_binding()
    );
    assert_eq!(
        handoff
            .dependency()
            .dependency()
            .binder_uses()
            .get(mizar_checker::source_formula_composition::SourceNestedFraenkelBinderUseId::new(
                0,
            ))
            .expect("retained resolver association")
            .resolver_binding(),
        identity.resolver_binding()
    );
    assert!(handoff
        .dependency()
        .binding_env()
        .bindings()
        .get(identity.projected_binding())
        .expect("retained outer-x binding")
        .captured
        .identities()
        .is_empty());
}

#[test]
fn task257c4c6_real_imported_fixture_installs_typed_capture_identity_receipt() {
    use mizar_checker::source_formula_composition::{
        SourceNestedFraenkelBinderUseProducer, SourceNestedFraenkelCaptureIdentityProducer,
    };
    use mizar_checker::source_term::SourceNestedFraenkelMapperPrimaryProducer;

    let output = task257c4c1_frontend_output(TASK257C4C1_CANONICAL_SOURCE, 3);
    assert!(output.diagnostics.is_empty());
    let ast = output.ast.expect("C4C6 canonical imported AST");
    let module = mizar_resolve::resolved_ast::ModuleId::new(
        output.source.package_id,
        output.source.module_path,
    );
    let resolved = mizar_resolve::resolved_ast::SurfaceResolvedArena::lower(&ast, &module)
        .expect("C4C6 resolver arena should lower");
    let resolver = FraenkelGeneratorVariableSourceCollector::new(&ast, &module, &resolved)
        .expect("C4C6 collector should validate the resolver arena")
        .collect()
        .expect("C4C6 collector should collect the exact relation");
    let profile = typed_ast_from_surface_resolved_profile(&ast, module.clone(), &resolved);
    let c4c3 = SourceNestedFraenkelBinderUseProducer::build(&resolver, &profile.typed_ast)
        .expect("C4C3 checker identity handoff should build");
    let c4c4 = SourceNestedFraenkelMapperPrimaryProducer::build(c4c3)
        .expect("C4C4 mapper primary handoff should build");
    let handoff = SourceNestedFraenkelCaptureIdentityProducer::build(c4c4)
        .expect("C4C5 capture identity handoff should build");
    let typed_debug_before = profile.typed_ast.debug_text();
    let typed = profile
        .typed_ast
        .with_source_nested_fraenkel_capture_identity(handoff.clone())
        .expect("C4C6 typed owner should install");

    let typed_debug = typed.debug_text();
    assert_eq!(typed, typed.clone());
    assert_eq!(typed_debug, typed.clone().debug_text());
    let debug_insertion_before = typed_debug_before
        .find("nodes:\n")
        .expect("C4C6 pre-install typed debug node section");
    assert_eq!(
        typed_debug,
        format!(
            "{}{}\n{}",
            &typed_debug_before[..debug_insertion_before],
            handoff.debug_text(),
            &typed_debug_before[debug_insertion_before..]
        )
    );
    assert_eq!(typed_debug.matches(&handoff.debug_text()).count(), 1);
    assert!(typed
        .source_nested_fraenkel_capture_identity()
        .is_some_and(|installed| installed == &handoff));
    assert!(typed
        .source_nested_fraenkel_capture_identity()
        .unwrap()
        .dependency()
        .binding_env()
        .bindings()
        .get(mizar_checker::binding_env::BindingId::new(0))
        .expect("C4C6 retained binding")
        .captured
        .identities()
        .is_empty());
    let resolved = assemble_empty_resolved_typed_ast(&typed, Vec::new())
        .expect("C4C6 syntax-only final clone should assemble");
    assert!(resolved
        .source_nested_fraenkel_capture_identity()
        .is_some_and(|installed| installed == &handoff));
}
