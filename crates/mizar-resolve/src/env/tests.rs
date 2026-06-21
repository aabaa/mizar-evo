use super::*;
use crate::resolved_ast::{
    ExportTarget, FullyQualifiedName, LocalSymbolId, ResolvedArenaBuilder, ResolvedExport,
    ResolvedImport, ResolvedImports, ResolvedNode, SymbolId,
};
use mizar_session::{
    BuildSnapshotId, GeneratedSpanAnchor, GeneratedSpanOrigin, Hash, InMemorySessionIdAllocator,
    ModulePath, PackageId, SessionIdAllocator, SourceRange,
};
use mizar_syntax::ast::SurfaceNodeKind;

#[test]
fn index_families_round_trip_insertions_and_lookups() {
    let source_id = source_id(1);
    let module = module_id("pkg", "main");
    let origin = origin(source_id, module.clone());
    let mut contributions = SourceContributionIndex::new();
    let local = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id },
        SourceAnchor::Range(range(source_id, 0, 1)),
    );
    let summary_contribution = contributions.insert(
        module_id("pkg", "dep"),
        ContributionKind::Summary {
            identity: ModuleSummaryIdentity::new("summary:dep:v1"),
        },
        SourceAnchor::Range(range(source_id, 10, 11)),
    );

    let symbol = symbol_id(module.clone(), "pred/0", "pkg::main::pred/0");
    let imported_symbol = symbol_id(module_id("pkg", "dep"), "func/0", "pkg::dep::func/0");
    let namespace = NamespacePath::new("main");
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Predicate,
            namespace.clone(),
            "P",
            origin.clone(),
            local,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
        .with_signature(SignatureShell::Opaque {
            schema: "signature-shell-v1".to_owned(),
            payload: "pred-shell".to_owned(),
        }),
    );
    symbols.insert(SymbolEntry::new(
        imported_symbol.clone(),
        SymbolKind::Functor,
        namespace.clone(),
        "F",
        origin.clone(),
        summary_contribution,
    ));
    assert_eq!(symbols.get(&symbol).unwrap().primary_spelling(), "P");
    assert_eq!(
        symbols
            .by_fqn(&FullyQualifiedName::new("pkg::main::pred/0"))
            .unwrap()
            .symbol(),
        &symbol
    );
    assert_eq!(symbols.visible_candidates(&namespace, "P").len(), 1);
    assert_eq!(symbols.exported_by_module(&module).len(), 1);
    assert_eq!(symbols.by_contribution(summary_contribution).len(), 1);

    let label = LabelOriginPath::new("pkg::main::T1");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            label.clone(),
            LabelKind::Theorem,
            namespace.clone(),
            "T1",
            origin.clone(),
            local,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );
    assert_eq!(labels.get(&label).unwrap().primary_spelling(), "T1");
    assert_eq!(labels.visible_candidates(&namespace, "T1").len(), 1);

    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Predicate,
            origin.clone(),
            local,
        )
        .with_parameters(vec![
            ResolverShellId::new("param:z"),
            ResolverShellId::new("param:a"),
        ])
        .with_binders(vec![
            ResolverShellId::new("binder:z"),
            ResolverShellId::new("binder:a"),
        ])
        .with_arity(1)
        .with_signature(SignatureShell::Pending),
    );
    assert_eq!(definitions.get(definition).unwrap().symbol(), &symbol);
    assert_eq!(definitions.by_symbol(&symbol).unwrap().id(), definition);
    assert_eq!(
        definitions
            .get(definition)
            .unwrap()
            .parameters()
            .iter()
            .map(ResolverShellId::as_str)
            .collect::<Vec<_>>(),
        vec!["param:z", "param:a"]
    );
    assert_eq!(
        definitions
            .get(definition)
            .unwrap()
            .binders()
            .iter()
            .map(ResolverShellId::as_str)
            .collect::<Vec<_>>(),
        vec!["binder:z", "binder:a"]
    );

    let mut overloads = OverloadIndex::new();
    let overload = overloads.insert(
        OverloadKey::new(namespace.clone(), "P", SymbolKind::Predicate, Some(1)),
        vec![imported_symbol.clone(), symbol.clone()],
        local,
    );
    assert!(overloads.add_diagnostic(overload, DiagnosticAnchorId::new(5)));
    assert!(overloads.add_diagnostic(overload, DiagnosticAnchorId::new(3)));
    assert_eq!(
        overloads.get(overload).unwrap().candidates(),
        &[imported_symbol.clone(), symbol.clone()]
    );
    assert_eq!(
        overloads.get(overload).unwrap().diagnostics(),
        &[DiagnosticAnchorId::new(3), DiagnosticAnchorId::new(5)]
    );

    let mut registrations = RegistrationIndex::new();
    let registration = registrations.insert(
        Some(symbol.clone()),
        RegistrationKind::Cluster,
        SignatureShell::Opaque {
            schema: "registration-target-v1".to_owned(),
            payload: "cluster-shell".to_owned(),
        },
        origin.clone(),
        local,
    );
    registrations
        .get_mut(registration)
        .unwrap()
        .set_visibility(Visibility::Public)
        .set_export_status(ExportStatus::Exported)
        .set_dependencies(vec![
            DeclarationDependencyId::new(2),
            DeclarationDependencyId::new(1),
        ]);
    assert_eq!(
        registrations.get(registration).unwrap().kind(),
        RegistrationKind::Cluster
    );
    assert_eq!(
        registrations.get(registration).unwrap().visibility(),
        Visibility::Public
    );
    assert_eq!(
        registrations.get(registration).unwrap().export_status(),
        ExportStatus::Exported
    );
    assert_eq!(
        registrations.get(registration).unwrap().dependencies(),
        &[
            DeclarationDependencyId::new(1),
            DeclarationDependencyId::new(2)
        ]
    );

    let mut lexical_summaries = ModuleLexicalSummaryIndex::new();
    let lexical = lexical_summaries.insert(
        symbol.clone(),
        namespace.clone(),
        "P(_)",
        LexicalSummaryKind::Notation,
        Some(1),
        local,
    );
    assert_eq!(
        lexical_summaries
            .visible_candidates(&namespace, "P(_)")
            .first()
            .map(|entry| entry.id()),
        Some(lexical)
    );
    assert_eq!(lexical_summaries.len(), 1);
    assert_eq!(lexical_summaries.get(lexical).unwrap().spelling(), "P(_)");
    assert_eq!(lexical_summaries.by_contribution(local).len(), 1);

    let mut graph = NamespaceGraph::new();
    let root = graph.insert_node(
        NamespaceNodeKind::Module,
        Some(module.clone()),
        "main",
        local,
    );
    let alias = graph.insert_node(
        NamespaceNodeKind::Alias,
        Some(module_id("pkg", "dep")),
        "D",
        summary_contribution,
    );
    let edge = graph.insert_edge(
        NamespaceEdgeSpec::new(
            (root, alias),
            NamespaceEdgeKind::Import,
            SourceAnchor::Range(range(source_id, 2, 3)),
            local,
        )
        .with_visibility(Visibility::Public)
        .with_target(NamespaceTarget::Module(module_id("pkg", "dep")))
        .with_local_spelling("D"),
    );
    assert_eq!(graph.node(root).unwrap().spelling(), "main");
    assert_eq!(graph.edge(edge).unwrap().local_spelling(), Some("D"));
    assert_eq!(graph.edge(edge).unwrap().visibility(), Visibility::Public);

    let mut dependencies = DeclarationDependencyIndex::new();
    let dependency = dependencies.insert(
        DependencyEndpoint::Symbol(symbol.clone()),
        DependencyEndpoint::NamespaceEdge(edge),
        DeclarationDependencyKind::Import,
        SourceAnchor::Range(range(source_id, 2, 3)),
        local,
    );
    assert_eq!(
        dependencies.get(dependency).unwrap().kind(),
        DeclarationDependencyKind::Import
    );

    let (import_id, export_id) = import_export_ids(source_id, module.clone());
    let mut imports = ResolvedImportIndex::new();
    imports.insert(ImportIndexEntry::new(
        import_id,
        Some(module_id("pkg", "dep")),
        Some("D".to_owned()),
        local,
    ));
    assert_eq!(imports.get(import_id).unwrap().alias(), Some("D"));
    let mut exports = ResolvedExportIndex::new();
    exports.insert(ExportIndexEntry::new(
        export_id,
        Some(DependencyEndpoint::Symbol(symbol.clone())),
        local,
    ));
    assert_eq!(
        exports.get(export_id).unwrap().target(),
        Some(&DependencyEndpoint::Symbol(symbol.clone()))
    );

    let mut summaries = ModuleSummaryIndex::new();
    let summary = summaries.insert(
        module_id("pkg", "dep"),
        ModuleSummaryIdentity::new("summary:dep:v1"),
        summary_contribution,
    );
    assert_eq!(
        summaries.by_module(&module_id("pkg", "dep")).unwrap().id(),
        summary
    );

    contributions.add_symbol(local, symbol.clone());
    contributions.add_label(local, label.clone());
    contributions.add_definition(local, definition);
    contributions.add_overload_group(local, overload);
    contributions.add_registration(local, registration);
    contributions.add_lexical_summary(local, lexical);
    contributions.add_namespace_edge(local, edge);
    contributions.add_declaration_dependency(local, dependency);
    contributions.add_import(local, import_id);
    contributions.add_export(local, export_id);
    contributions.add_diagnostic(local, DiagnosticAnchorId::new(0));

    let env = SymbolEnv::new(
        module.clone(),
        SymbolEnvIndexes {
            imports,
            exports,
            symbols,
            labels,
            definitions,
            overloads,
            registrations,
            lexical_summaries,
            namespace_graph: graph,
            declaration_dependencies: dependencies,
            contributions,
            module_summaries: summaries,
        },
    );
    assert_eq!(env.module_id(), &module);
    assert_eq!(env.symbols().len(), 2);
    assert_eq!(
        env.contributions().affected_by(local).unwrap().symbols(),
        &[symbol]
    );
    assert_eq!(
        env.contributions()
            .affected_by(local)
            .unwrap()
            .lexical_summaries(),
        &[lexical]
    );
    assert_eq!(
        env.contributions()
            .affected_by(local)
            .unwrap()
            .diagnostics(),
        &[DiagnosticAnchorId::new(0)]
    );
}

#[test]
fn index_iteration_is_deterministic_for_all_families() {
    let primary_source_id = source_id(2);
    let module = module_id("pkg", "main");
    let origin = origin(primary_source_id, module.clone());
    let mut contributions = SourceContributionIndex::new();
    let local = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource {
            source_id: primary_source_id,
        },
        SourceAnchor::Range(range(primary_source_id, 0, 1)),
    );
    let other = contributions.insert(
        module_id("pkg", "dep"),
        ContributionKind::Summary {
            identity: ModuleSummaryIdentity::new("summary:dep"),
        },
        SourceAnchor::Range(range(primary_source_id, 1, 2)),
    );
    let z_symbol = symbol_id(module.clone(), "z/0", "pkg::main::z/0");
    let a_symbol = symbol_id(module.clone(), "a/0", "pkg::main::a/0");
    let namespace = NamespacePath::new("main");

    let mut symbols = SymbolIndex::new();
    symbols.insert(SymbolEntry::new(
        z_symbol.clone(),
        SymbolKind::Predicate,
        namespace.clone(),
        "Z",
        origin.clone(),
        local,
    ));
    symbols.insert(SymbolEntry::new(
        a_symbol.clone(),
        SymbolKind::Predicate,
        namespace.clone(),
        "A",
        origin.clone(),
        local,
    ));
    assert_eq!(
        symbols
            .iter()
            .map(|entry| entry.symbol().local().as_str())
            .collect::<Vec<_>>(),
        vec!["a/0", "z/0"]
    );

    let mut labels = LabelIndex::new();
    labels.insert(LabelEntry::new(
        LabelOriginPath::new("pkg::main::Z1"),
        LabelKind::Theorem,
        namespace.clone(),
        "Z1",
        origin.clone(),
        local,
    ));
    labels.insert(LabelEntry::new(
        LabelOriginPath::new("pkg::main::A1"),
        LabelKind::Theorem,
        namespace.clone(),
        "A1",
        origin.clone(),
        local,
    ));
    assert_eq!(
        labels
            .iter()
            .map(|entry| entry.origin_path().as_str())
            .collect::<Vec<_>>(),
        vec!["pkg::main::A1", "pkg::main::Z1"]
    );

    let mut definitions = DefinitionIndex::new();
    let z_definition = definitions.insert(DefinitionShell::new(
        z_symbol.clone(),
        DefinitionKind::Predicate,
        origin.clone(),
        local,
    ));
    let a_definition = definitions.insert(DefinitionShell::new(
        a_symbol.clone(),
        DefinitionKind::Predicate,
        origin.clone(),
        local,
    ));
    assert_eq!(
        definitions
            .iter()
            .map(DefinitionEntry::id)
            .collect::<Vec<_>>(),
        vec![a_definition, z_definition]
    );

    let mut overloads = OverloadIndex::new();
    let z_group = overloads.insert(
        OverloadKey::new(namespace.clone(), "Z", SymbolKind::Predicate, None),
        vec![z_symbol.clone()],
        local,
    );
    let a_group = overloads.insert(
        OverloadKey::new(namespace.clone(), "A", SymbolKind::Predicate, None),
        vec![a_symbol.clone()],
        local,
    );
    assert_eq!(
        overloads.iter().map(OverloadGroup::id).collect::<Vec<_>>(),
        vec![a_group, z_group]
    );

    let mut registrations = RegistrationIndex::new();
    let z_registration = registrations.insert(
        Some(z_symbol.clone()),
        RegistrationKind::Cluster,
        SignatureShell::Pending,
        origin.clone(),
        local,
    );
    let a_registration = registrations.insert(
        Some(a_symbol.clone()),
        RegistrationKind::Cluster,
        SignatureShell::Pending,
        origin.clone(),
        local,
    );
    assert_eq!(
        registrations
            .iter()
            .map(RegistrationEntry::id)
            .collect::<Vec<_>>(),
        vec![a_registration, z_registration]
    );

    let mut lexical_summaries = ModuleLexicalSummaryIndex::new();
    let z_lexical = lexical_summaries.insert(
        z_symbol.clone(),
        namespace.clone(),
        "Z",
        LexicalSummaryKind::Notation,
        None,
        local,
    );
    let a_lexical = lexical_summaries.insert(
        a_symbol.clone(),
        namespace.clone(),
        "A",
        LexicalSummaryKind::Notation,
        None,
        local,
    );
    assert_eq!(
        lexical_summaries
            .iter()
            .map(LexicalSummaryEntry::id)
            .collect::<Vec<_>>(),
        vec![a_lexical, z_lexical]
    );

    let mut graph = NamespaceGraph::new();
    let z_node = graph.insert_node(NamespaceNodeKind::Segment, Some(module.clone()), "z", local);
    let a_node = graph.insert_node(NamespaceNodeKind::Segment, Some(module.clone()), "a", local);
    let a_edge = graph.insert_edge(NamespaceEdgeSpec::new(
        (a_node, z_node),
        NamespaceEdgeKind::Segment,
        SourceAnchor::Range(range(primary_source_id, 5, 6)),
        local,
    ));
    let z_edge = graph.insert_edge(NamespaceEdgeSpec::new(
        (z_node, a_node),
        NamespaceEdgeKind::Segment,
        SourceAnchor::Range(range(primary_source_id, 4, 5)),
        local,
    ));
    assert_eq!(
        graph
            .nodes()
            .map(NamespaceNode::spelling)
            .collect::<Vec<_>>(),
        vec!["a", "z"]
    );
    assert_eq!(
        graph.edges().map(NamespaceEdge::id).collect::<Vec<_>>(),
        vec![z_edge, a_edge]
    );

    let mut dependencies = DeclarationDependencyIndex::new();
    let z_dependency = dependencies.insert(
        DependencyEndpoint::Symbol(z_symbol.clone()),
        DependencyEndpoint::Symbol(a_symbol.clone()),
        DeclarationDependencyKind::SignatureMention,
        SourceAnchor::Range(range(primary_source_id, 6, 7)),
        local,
    );
    let a_dependency = dependencies.insert(
        DependencyEndpoint::Symbol(a_symbol.clone()),
        DependencyEndpoint::Symbol(z_symbol.clone()),
        DeclarationDependencyKind::SignatureMention,
        SourceAnchor::Range(range(primary_source_id, 7, 8)),
        local,
    );
    assert_eq!(
        dependencies
            .iter()
            .map(DeclarationDependency::id)
            .collect::<Vec<_>>(),
        vec![a_dependency, z_dependency]
    );

    let ((first_import, first_export), (second_import, second_export)) =
        import_export_id_pair(primary_source_id, module.clone());
    let mut imports = ResolvedImportIndex::new();
    imports.insert(ImportIndexEntry::new(second_import, None, None, local));
    imports.insert(ImportIndexEntry::new(first_import, None, None, local));
    assert_eq!(
        imports
            .iter()
            .map(ImportIndexEntry::import)
            .collect::<Vec<_>>(),
        vec![first_import, second_import]
    );
    let mut exports = ResolvedExportIndex::new();
    exports.insert(ExportIndexEntry::new(second_export, None, local));
    exports.insert(ExportIndexEntry::new(first_export, None, local));
    assert_eq!(
        exports
            .iter()
            .map(ExportIndexEntry::export)
            .collect::<Vec<_>>(),
        vec![first_export, second_export]
    );

    let mut summaries = ModuleSummaryIndex::new();
    let z_summary = summaries.insert(
        module_id("pkg", "z"),
        ModuleSummaryIdentity::new("summary:z"),
        other,
    );
    let a_summary = summaries.insert(
        module_id("pkg", "a"),
        ModuleSummaryIdentity::new("summary:a"),
        other,
    );
    assert_eq!(
        summaries
            .iter()
            .map(ModuleSummaryEntry::id)
            .collect::<Vec<_>>(),
        vec![a_summary, z_summary]
    );

    assert_eq!(
        contributions
            .iter()
            .map(SourceContribution::id)
            .collect::<Vec<_>>(),
        vec![local, other]
    );
}

#[test]
fn contribution_tracking_covers_sources_summaries_builtins_and_invalidation() {
    let (primary_source_id, imported_source_id) = source_id_pair(4);
    let module = module_id("pkg", "main");
    let summary_identity = ModuleSummaryIdentity::new("summary:dep:v2");
    let mut contributions = SourceContributionIndex::new();
    let local = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource {
            source_id: primary_source_id,
        },
        SourceAnchor::Range(range(primary_source_id, 0, 1)),
    );
    let imported = contributions.insert(
        module_id("pkg", "dep_source"),
        ContributionKind::ImportedSource {
            source_id: imported_source_id,
        },
        SourceAnchor::Range(range(imported_source_id, 0, 1)),
    );
    let summary = contributions.insert(
        module_id("pkg", "dep_summary"),
        ContributionKind::Summary {
            identity: summary_identity.clone(),
        },
        SourceAnchor::Range(range(primary_source_id, 1, 2)),
    );
    let builtin = contributions.insert(
        module.clone(),
        ContributionKind::Builtin {
            name: "prelude".to_owned(),
        },
        SourceAnchor::Range(range(primary_source_id, 2, 3)),
    );

    let symbol = symbol_id(module.clone(), "pred/0", "pkg::main::pred/0");
    let definition = DefinitionId::new(0);
    let overload = OverloadGroupId::new(0);
    let registration = RegistrationId::new(0);
    let lexical_summary = LexicalSummaryId::new(0);
    let label = LabelOriginPath::new("pkg::main::T1");
    let namespace_edge = NamespaceEdgeId::new(0);
    let dependency = DeclarationDependencyId::new(0);
    let ((import, export), (later_import, later_export)) =
        import_export_id_pair(primary_source_id, module);

    contributions.add_symbol(local, symbol.clone());
    contributions.add_definition(local, definition);
    contributions.add_overload_group(local, overload);
    contributions.add_registration(local, registration);
    contributions.add_lexical_summary(local, lexical_summary);
    contributions.add_label(local, label.clone());
    contributions.add_namespace_edge(local, namespace_edge);
    contributions.add_declaration_dependency(local, dependency);
    contributions.add_import(local, later_import);
    contributions.add_import(local, import);
    contributions.add_import(local, import);
    contributions.add_export(local, later_export);
    contributions.add_export(local, export);
    contributions.add_export(local, export);
    contributions.add_diagnostic(local, DiagnosticAnchorId::new(7));
    contributions.add_diagnostic(local, DiagnosticAnchorId::new(3));
    contributions.add_diagnostic(local, DiagnosticAnchorId::new(7));
    contributions.add_symbol(local, symbol.clone());

    let effects = contributions.affected_by(local).unwrap();
    assert_eq!(effects.symbols(), &[symbol]);
    assert_eq!(effects.definitions(), &[definition]);
    assert_eq!(effects.overload_groups(), &[overload]);
    assert_eq!(effects.registrations(), &[registration]);
    assert_eq!(effects.lexical_summaries(), &[lexical_summary]);
    assert_eq!(effects.labels(), &[label]);
    assert_eq!(effects.namespace_edges(), &[namespace_edge]);
    assert_eq!(effects.declaration_dependencies(), &[dependency]);
    assert_eq!(effects.imports(), &[import, later_import]);
    assert_eq!(effects.exports(), &[export, later_export]);
    assert_eq!(
        effects.diagnostics(),
        &[DiagnosticAnchorId::new(3), DiagnosticAnchorId::new(7)]
    );

    assert_eq!(contributions.by_source(primary_source_id).len(), 1);
    assert_eq!(contributions.by_source(imported_source_id).len(), 1);
    assert_eq!(contributions.by_summary(&summary_identity).len(), 1);
    assert!(matches!(
        contributions.get(builtin).unwrap().kind(),
        ContributionKind::Builtin { name } if name == "prelude"
    ));
    assert_eq!(
        contributions
            .get(imported)
            .unwrap()
            .module()
            .path()
            .as_str(),
        "dep_source"
    );
    assert_eq!(
        contributions.get(summary).unwrap().module().path().as_str(),
        "dep_summary"
    );
}

#[test]
fn namespace_alias_node_order_uses_canonical_module_before_alias_spelling() {
    let source_id = source_id(6);
    let module = module_id("pkg", "main");
    let mut contributions = SourceContributionIndex::new();
    let local = contributions.insert(
        module,
        ContributionKind::LocalSource { source_id },
        SourceAnchor::Range(range(source_id, 0, 1)),
    );
    let mut graph = NamespaceGraph::new();
    let dep = module_id("pkg", "dep");
    graph.insert_node(
        NamespaceNodeKind::Alias,
        Some(dep.clone()),
        "z_alias",
        local,
    );
    graph.insert_node(NamespaceNodeKind::Alias, Some(dep), "a_alias", local);

    assert_eq!(
        graph
            .nodes()
            .map(NamespaceNode::spelling)
            .collect::<Vec<_>>(),
        vec!["z_alias", "a_alias"]
    );

    let mut cross_module_graph = NamespaceGraph::new();
    cross_module_graph.insert_node(
        NamespaceNodeKind::Alias,
        Some(module_id("pkg", "z_dep")),
        "a_alias",
        local,
    );
    cross_module_graph.insert_node(
        NamespaceNodeKind::Alias,
        Some(module_id("pkg", "a_dep")),
        "z_alias",
        local,
    );
    assert_eq!(
        cross_module_graph
            .nodes()
            .map(|node| node.module().unwrap().path().as_str())
            .collect::<Vec<_>>(),
        vec!["a_dep", "z_dep"]
    );
}

#[test]
fn equivalent_construction_is_stable_and_checker_facts_are_absent() {
    let first = build_equivalent_env_snapshot(source_id(7));
    let second = build_equivalent_env_snapshot(source_id(7));

    assert_eq!(first, second);
    assert!(matches!(
        first.0.as_ref(),
        Some(SignatureShell::Opaque { schema, payload })
            if schema == "signature-shell-v1" && payload == "opaque"
    ));
}

#[test]
fn symbol_env_snapshot_text_is_stable_and_covers_index_families() {
    let (local_source, imported_source) = source_id_pair(8);
    let first = debug_snapshot_env_fixture(local_source, imported_source).snapshot_text();
    let second = debug_snapshot_env_fixture(local_source, imported_source).snapshot_text();

    assert_eq!(first, second);
    assert!(first.starts_with("symbol-env-debug-v1\nmodule: pkg::main\n"));
    assert!(
        SymbolEnv::new(module_id("pkg", "empty"), SymbolEnvIndexes::default())
            .snapshot_text()
            .contains("lexical_summaries:\n  <none>\n")
    );
    assert!(!first.contains("SourceId"));
    assert!(!first.contains("/tmp/private"));
    assert!(!first.contains('\r'));
    assert_ordered_fragments(
        &first,
        &[
            "module: pkg::main\n",
            "imports:\n",
            "exports:\n",
            "symbols:\n",
            "labels:\n",
            "definitions:\n",
            "overloads:\n",
            "registrations:\n",
            "lexical_summaries:\n",
            "namespace_graph:\n",
            "declaration_dependencies:\n",
            "contributions:\n",
            "module_summaries:\n",
        ],
    );
    for expected in [
        "imports:\n  import#0 module=pkg::dep alias=\"D\" contribution=contribution#0",
        "exports:\n  export#0 target=symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} contribution=contribution#0",
        "symbols:\n  symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} kind=predicate visibility=public export=exported namespace=\"main\" spelling=\"P\" notation=\"P(_)\" signature=opaque(schema=\"signature-v1\", payload=\"pred-shell\\n\\\"\\\\\") relations=[synonym->",
        "labels:\n  label=\"pkg::main::T1\" kind=theorem visibility=public export=exported namespace=\"main\" spelling=\"T1\"",
        "definitions:\n  definition#0 symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} kind=predicate visibility=public parameters=[\"param:x\"] binders=[\"binder:x\"] arity=1 notation=\"P(_)\" doc=\"doc:T1\" conflict=duplicate_spelling dependencies=[dependency#0] signature=pending",
        "overloads:\n  overload#0 key={namespace=\"main\" spelling=\"P\" kind=predicate arity=1} candidates=[",
        "diagnostics=[diagnostic#4]",
        "registrations:\n  registration#0 symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} kind=cluster target=malformed(class=\"recovered-target\") visibility=public export=re_exported dependencies=[dependency#0]",
        "lexical_summaries:\n  lexical#0 symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} namespace=\"main\" spelling=\"P(_)\" kind=notation arity=1 contribution=contribution#0",
        "namespace_graph:\n  nodes:\n    node#0 kind=module module=pkg::main spelling=\"main\" contribution=contribution#0",
        "  edges:\n    edge#0 from=node#0 to=node#1 kind=import anchor=range(2..3) visibility=public target=module=pkg::dep local_spelling=\"D\" contribution=contribution#0",
        "declaration_dependencies:\n  dependency#0 source=symbol={fqn=\"pkg::main::pred/0\" module=pkg::main local=\"pred/0\"} target=namespace_edge#0 kind=import anchor=range(2..3) contribution=contribution#0",
        "contributions:\n  contribution#0 module=pkg::main kind=local_source anchor=range(0..1) effects={symbols=[",
        " definitions=[definition#0] overloads=[overload#0] registrations=[registration#0] lexical_summaries=[lexical#0] labels=[\"pkg::main::T1\"] namespace_edges=[edge#0] declaration_dependencies=[dependency#0] imports=[import#0] exports=[export#0] diagnostics=[diagnostic#9]}",
        "contribution#1 module=pkg::dep_source kind=imported_source anchor=point(5)",
        "contribution#2 module=pkg::dep_summary kind=summary identity=\"summary:dep:v1\" anchor=generated(range(1..2), reason=present)",
        "contribution#3 module=pkg::builtin kind=builtin name=\"prelude\"",
        "module_summaries:\n  summary#0 module=pkg::dep identity=\"summary:dep:v1\" contribution=contribution#2",
    ] {
        assert!(
            first.contains(expected),
            "snapshot should contain fixture fragment: {expected}\n{first}"
        );
    }
}

fn assert_ordered_fragments(snapshot: &str, fragments: &[&str]) {
    let mut cursor = 0;
    for fragment in fragments {
        let Some(offset) = snapshot[cursor..].find(fragment) else {
            panic!("missing ordered fragment: {fragment}\n{snapshot}");
        };
        cursor += offset + fragment.len();
    }
}

fn debug_snapshot_env_fixture(local_source: SourceId, imported_source: SourceId) -> SymbolEnv {
    let module = module_id("pkg", "main");
    let origin = origin(local_source, module.clone());
    let namespace = NamespacePath::new("main");
    let symbol = symbol_id(module.clone(), "pred/0", "pkg::main::pred/0");
    let related = symbol_id(module.clone(), "other/0", "pkg::main::other/0");

    let mut contributions = SourceContributionIndex::new();
    let local = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource {
            source_id: local_source,
        },
        SourceAnchor::Range(range(local_source, 0, 1)),
    );
    contributions.insert(
        module_id("pkg", "dep_source"),
        ContributionKind::ImportedSource {
            source_id: imported_source,
        },
        SourceAnchor::Point {
            source_id: imported_source,
            offset: 5,
        },
    );
    let generated_summary_anchor = GeneratedSpanOrigin::new(
        GeneratedSpanAnchor::Range(range(local_source, 1, 2)),
        "/tmp/private/generated-summary",
    )
    .unwrap();
    let summary = contributions.insert(
        module_id("pkg", "dep_summary"),
        ContributionKind::Summary {
            identity: ModuleSummaryIdentity::new("summary:dep:v1"),
        },
        SourceAnchor::Generated(generated_summary_anchor),
    );
    contributions.insert(
        module_id("pkg", "builtin"),
        ContributionKind::Builtin {
            name: "prelude".to_owned(),
        },
        SourceAnchor::Range(range(local_source, 2, 3)),
    );

    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Predicate,
            namespace.clone(),
            "P",
            origin.clone(),
            local,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
        .with_notation_spelling("P(_)")
        .with_signature(SignatureShell::Opaque {
            schema: "signature-v1".to_owned(),
            payload: "pred-shell\n\"\\".to_owned(),
        })
        .with_relations(vec![RelationMetadata::new(
            RelationKind::Synonym,
            related.clone(),
        )]),
    );

    let label = LabelOriginPath::new("pkg::main::T1");
    let mut labels = LabelIndex::new();
    labels.insert(
        LabelEntry::new(
            label.clone(),
            LabelKind::Theorem,
            namespace.clone(),
            "T1",
            origin.clone(),
            local,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported),
    );

    let mut definitions = DefinitionIndex::new();
    let definition = definitions.insert(
        DefinitionShell::new(
            symbol.clone(),
            DefinitionKind::Predicate,
            origin.clone(),
            local,
        )
        .with_visibility(Visibility::Public)
        .with_parameters(vec![ResolverShellId::new("param:x")])
        .with_binders(vec![ResolverShellId::new("binder:x")])
        .with_arity(1)
        .with_notation_shape("P(_)")
        .with_doc_attachment(ResolverShellId::new("doc:T1"))
        .with_conflict(DeclarationConflictClass::DuplicateSpelling)
        .with_dependencies(vec![DeclarationDependencyId::new(0)])
        .with_signature(SignatureShell::Pending),
    );

    let mut overloads = OverloadIndex::new();
    let overload = overloads.insert(
        OverloadKey::new(namespace.clone(), "P", SymbolKind::Predicate, Some(1)),
        vec![related.clone(), symbol.clone()],
        local,
    );
    assert!(overloads.add_diagnostic(overload, DiagnosticAnchorId::new(4)));

    let mut registrations = RegistrationIndex::new();
    let registration = registrations.insert(
        Some(symbol.clone()),
        RegistrationKind::Cluster,
        SignatureShell::Malformed {
            class: "recovered-target".to_owned(),
        },
        origin.clone(),
        local,
    );
    registrations
        .get_mut(registration)
        .unwrap()
        .set_visibility(Visibility::Public)
        .set_export_status(ExportStatus::ReExported)
        .set_dependencies(vec![DeclarationDependencyId::new(0)]);

    let mut lexical_summaries = ModuleLexicalSummaryIndex::new();
    let lexical_summary = lexical_summaries.insert(
        symbol.clone(),
        namespace.clone(),
        "P(_)",
        LexicalSummaryKind::Notation,
        Some(1),
        local,
    );

    let mut namespace_graph = NamespaceGraph::new();
    let root = namespace_graph.insert_node(
        NamespaceNodeKind::Module,
        Some(module.clone()),
        "main",
        local,
    );
    let alias = namespace_graph.insert_node(
        NamespaceNodeKind::Alias,
        Some(module_id("pkg", "dep")),
        "D",
        summary,
    );
    let namespace_edge = namespace_graph.insert_edge(
        NamespaceEdgeSpec::new(
            (root, alias),
            NamespaceEdgeKind::Import,
            SourceAnchor::Range(range(local_source, 2, 3)),
            local,
        )
        .with_visibility(Visibility::Public)
        .with_target(NamespaceTarget::Module(module_id("pkg", "dep")))
        .with_local_spelling("D"),
    );

    let mut declaration_dependencies = DeclarationDependencyIndex::new();
    let dependency = declaration_dependencies.insert(
        DependencyEndpoint::Symbol(symbol.clone()),
        DependencyEndpoint::NamespaceEdge(namespace_edge),
        DeclarationDependencyKind::Import,
        SourceAnchor::Range(range(local_source, 2, 3)),
        local,
    );

    let (import, export) = import_export_ids(local_source, module.clone());
    let mut imports = ResolvedImportIndex::new();
    imports.insert(ImportIndexEntry::new(
        import,
        Some(module_id("pkg", "dep")),
        Some("D".to_owned()),
        local,
    ));
    let mut exports = ResolvedExportIndex::new();
    exports.insert(ExportIndexEntry::new(
        export,
        Some(DependencyEndpoint::Symbol(symbol.clone())),
        local,
    ));

    let mut module_summaries = ModuleSummaryIndex::new();
    module_summaries.insert(
        module_id("pkg", "dep"),
        ModuleSummaryIdentity::new("summary:dep:v1"),
        summary,
    );

    contributions.add_symbol(local, symbol);
    contributions.add_definition(local, definition);
    contributions.add_overload_group(local, overload);
    contributions.add_registration(local, registration);
    contributions.add_lexical_summary(local, lexical_summary);
    contributions.add_label(local, label);
    contributions.add_namespace_edge(local, namespace_edge);
    contributions.add_declaration_dependency(local, dependency);
    contributions.add_import(local, import);
    contributions.add_export(local, export);
    contributions.add_diagnostic(local, DiagnosticAnchorId::new(9));

    SymbolEnv::new(
        module,
        SymbolEnvIndexes {
            imports,
            exports,
            symbols,
            labels,
            definitions,
            overloads,
            registrations,
            lexical_summaries,
            namespace_graph,
            declaration_dependencies,
            contributions,
            module_summaries,
        },
    )
}

fn build_equivalent_env_snapshot(
    source_id: SourceId,
) -> (Option<SignatureShell>, Vec<String>, Vec<usize>) {
    let module = module_id("pkg", "main");
    let origin = origin(source_id, module.clone());
    let mut contributions = SourceContributionIndex::new();
    let local = contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id },
        SourceAnchor::Range(range(source_id, 0, 1)),
    );
    let symbol = symbol_id(module, "pred/0", "pkg::main::pred/0");
    let namespace = NamespacePath::new("main");
    let mut symbols = SymbolIndex::new();
    symbols.insert(
        SymbolEntry::new(
            symbol.clone(),
            SymbolKind::Predicate,
            namespace.clone(),
            "P",
            origin.clone(),
            local,
        )
        .with_signature(SignatureShell::Opaque {
            schema: "signature-shell-v1".to_owned(),
            payload: "opaque".to_owned(),
        }),
    );
    let mut overloads = OverloadIndex::new();
    let overload = overloads.insert(
        OverloadKey::new(namespace, "P", SymbolKind::Predicate, None),
        vec![symbol.clone()],
        local,
    );
    contributions.add_symbol(local, symbol.clone());
    contributions.add_overload_group(local, overload);

    (
        symbols.get(&symbol).unwrap().signature().cloned(),
        symbols
            .iter()
            .map(|entry| entry.symbol().fqn().as_str().to_owned())
            .collect(),
        contributions
            .affected_by(local)
            .unwrap()
            .overload_groups()
            .iter()
            .map(|id| id.index())
            .collect(),
    )
}

fn import_export_ids(
    source_id: SourceId,
    module: ModuleId,
) -> (ResolvedImportId, ResolvedExportId) {
    import_export_id_pair(source_id, module).0
}

fn import_export_id_pair(
    source_id: SourceId,
    module: ModuleId,
) -> (
    (ResolvedImportId, ResolvedExportId),
    (ResolvedImportId, ResolvedExportId),
) {
    let origin = origin(source_id, module.clone());
    let mut builder = ResolvedArenaBuilder::new();
    let first_import_node = builder
        .push(ResolvedNode::new(
            SurfaceNodeKind::ImportItem,
            Vec::new(),
            origin.clone(),
        ))
        .unwrap();
    let first_export_node = builder
        .push(ResolvedNode::new(
            SurfaceNodeKind::ExportItem,
            Vec::new(),
            origin.clone(),
        ))
        .unwrap();
    let second_import_node = builder
        .push(ResolvedNode::new(
            SurfaceNodeKind::ImportItem,
            Vec::new(),
            origin.clone(),
        ))
        .unwrap();
    let second_export_node = builder
        .push(ResolvedNode::new(
            SurfaceNodeKind::ExportItem,
            Vec::new(),
            origin.clone(),
        ))
        .unwrap();
    let mut imports = ResolvedImports::new();
    let first_import_id = imports.push_import(ResolvedImport::new(
        first_import_node,
        range(source_id, 0, 1),
        "import dep;",
        None,
        crate::resolved_ast::ImportResolution::Resolved(module_id("pkg", "dep")),
        origin.clone(),
    ));
    let second_import_id = imports.push_import(ResolvedImport::new(
        second_import_node,
        range(source_id, 2, 3),
        "import other;",
        None,
        crate::resolved_ast::ImportResolution::Resolved(module_id("pkg", "other")),
        origin.clone(),
    ));
    let first_export_id = imports.push_export(ResolvedExport::new(
        first_export_node,
        range(source_id, 1, 2),
        "export dep;",
        ExportTarget::Module(module.clone()),
        origin.clone(),
    ));
    let second_export_id = imports.push_export(ResolvedExport::new(
        second_export_node,
        range(source_id, 3, 4),
        "export other;",
        ExportTarget::Module(module),
        origin,
    ));
    (
        (first_import_id, first_export_id),
        (second_import_id, second_export_id),
    )
}

fn source_id(seed: u8) -> SourceId {
    let snapshot_id = snapshot_id(seed);
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot_id)
        .unwrap()
}

fn source_id_pair(seed: u8) -> (SourceId, SourceId) {
    let snapshot_id = snapshot_id(seed);
    let allocator = InMemorySessionIdAllocator::new();
    (
        allocator.next_source_id(snapshot_id).unwrap(),
        allocator.next_source_id(snapshot_id).unwrap(),
    )
}

fn snapshot_id(seed: u8) -> BuildSnapshotId {
    let hex = format!("{seed:02x}").repeat(Hash::BYTE_LEN);
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .unwrap()
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn origin(source_id: SourceId, module_id: ModuleId) -> SemanticOrigin {
    SemanticOrigin::new(
        source_id,
        module_id,
        SourceAnchor::Range(range(source_id, 0, 1)),
        vec![0],
    )
}

fn module_id(package: &str, path: &str) -> ModuleId {
    ModuleId::new(PackageId::new(package), ModulePath::new(path))
}

fn symbol_id(module: ModuleId, local: &str, fqn: &str) -> SymbolId {
    SymbolId::new(
        module,
        LocalSymbolId::new(local),
        FullyQualifiedName::new(fqn),
    )
}
