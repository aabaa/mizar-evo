use super::*;
use crate::imports::{ImportPathCandidate, ImportPathPrefix, ImportPathResolver};
use crate::module_index::WorkspaceStubModuleIndexProvider;
use crate::resolved_ast::{
    AmbiguousNameRef, FullyQualifiedName, LocalSymbolId, NameRefEntry, NameResolution,
    NameResolutionCandidate, ResolvedArenaBuilder, ResolvedNode, SymbolId,
};
use mizar_build::module_index::{
    DependencyModuleSummaryRef, ModuleIndexEntry, ModuleIndexLocation, PackageIndexEntry,
    PackageIndexSource,
};
use mizar_session::{
    BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, SessionIdAllocator, SourceAnchor,
    SourceId,
};
use mizar_syntax::ast::SurfaceNodeKind;
use mizar_syntax::{SurfaceAst, SurfaceAstBuilder, SurfaceTokenKind};
use semver::Version;

#[test]
fn task277r1_collects_exact_template_generator_identity() {
    let source_id = source_id();
    let module = module_id("pkg", "templates");
    let ast = task277r1_identity_ast(source_id);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection = TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
        .unwrap()
        .collect()
        .unwrap();

    assert_eq!(collection.source_id(), source_id);
    assert_eq!(collection.module(), &module);
    assert_eq!(collection.bindings().len(), 1);
    assert!(!collection.bindings().is_empty());
    assert_eq!(collection.generator_links().len(), 1);
    assert!(!collection.generator_links().is_empty());
    assert_eq!(
        collection.debug_text(),
        "template-type-parameter-source-v1|module=pkg.templates|bindings=1|generator-links=1"
    );

    let binding_id = TemplateTypeParameterBindingId::new(0);
    assert_eq!(binding_id.index(), 0);
    let binding = collection.bindings().get(binding_id).unwrap();
    assert_eq!(
        binding.definition_block(),
        resolved_node(&ast, &resolved, 0, 62, |kind| {
            matches!(kind, SurfaceNodeKind::DefinitionBlockItem)
        })
    );
    assert_eq!(
        binding.parameter(),
        resolved_node(&ast, &resolved, 11, 25, |kind| {
            matches!(kind, SurfaceNodeKind::TemplateParameter)
        })
    );
    assert_eq!(
        binding.binder(),
        resolved_node(&ast, &resolved, 15, 16, |kind| {
            matches!(kind, SurfaceNodeKind::Token(token) if token.kind == SurfaceTokenKind::Identifier && token.text.as_ref() == "T")
        })
    );
    assert_eq!(binding.spelling(), "T");
    assert_eq!(binding.source_range(), range(source_id, 11, 25));
    assert_eq!(binding.source_ordinal(), 0);
    assert_eq!(
        collection
            .bindings()
            .iter()
            .map(|(id, row)| (id, row.spelling()))
            .collect::<Vec<_>>(),
        vec![(binding_id, "T")]
    );
    assert!(
        collection
            .bindings()
            .get(TemplateTypeParameterBindingId::new(1))
            .is_none()
    );

    let link = collection.generator_links().get(0).unwrap();
    assert_eq!(link.definition_block(), binding.definition_block());
    assert_eq!(
        link.type_head(),
        resolved_node(&ast, &resolved, 48, 49, |kind| {
            matches!(kind, SurfaceNodeKind::TypeHead)
        })
    );
    assert_eq!(
        link.identifier(),
        resolved_node(&ast, &resolved, 48, 49, |kind| {
            matches!(kind, SurfaceNodeKind::Token(token) if token.kind == SurfaceTokenKind::Identifier && token.text.as_ref() == "T")
        })
    );
    assert_eq!(link.binding(), binding_id);
    assert_eq!(link.source_range(), range(source_id, 48, 49));
    assert_eq!(link.source_ordinal(), 0);
    assert_eq!(
        collection.generator_links().iter().collect::<Vec<_>>(),
        vec![link]
    );
    assert!(collection.generator_links().get(1).is_none());
}

#[test]
fn task277r1_isolates_scope_and_ignores_non_generator_roles() {
    let source_id = source_id();
    let module = module_id("pkg", "templates");
    let ast = task277r1_scoped_ast(source_id);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection = TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
        .unwrap()
        .collect()
        .unwrap();

    assert_eq!(
        collection
            .bindings()
            .iter()
            .map(|(id, binding)| (id.index(), binding.spelling(), binding.source_ordinal()))
            .collect::<Vec<_>>(),
        vec![
            (0, "T", 0),
            (1, "T", 1),
            (2, "V", 2),
            (3, "N", 3),
            (4, "X", 4),
            (5, "U", 5),
        ]
    );
    assert_eq!(collection.generator_links().len(), 3);
    assert_eq!(
        collection
            .generator_links()
            .iter()
            .map(|link| (
                link.binding().index(),
                link.source_range(),
                link.source_ordinal()
            ))
            .collect::<Vec<_>>(),
        vec![
            (2, range(source_id, 138, 139), 0),
            (5, range(source_id, 198, 199), 1),
            (2, range(source_id, 268, 269), 2),
        ]
    );
}

#[test]
fn task277r1_rejects_unsupported_parameter_and_recovery_shapes() {
    let source_id = source_id();
    let module = module_id("pkg", "templates");
    let ast = task277r1_rejected_shapes_ast(source_id);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collection = TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved)
        .unwrap()
        .collect()
        .unwrap();

    assert_eq!(collection.bindings().len(), 1);
    let binding = collection
        .bindings()
        .get(TemplateTypeParameterBindingId::new(0))
        .unwrap();
    assert_eq!(binding.spelling(), "Z");
    assert_eq!(binding.source_range(), range(source_id, 190, 204));
    assert!(collection.generator_links().is_empty());
}

#[test]
fn task277r1_revalidates_surface_resolved_arena_and_replays_deterministically() {
    let source_id = source_id();
    let module = module_id("pkg", "templates");
    let ast = task277r1_identity_ast(source_id);
    let resolved = SurfaceResolvedArena::lower(&ast, &module).unwrap();
    let collector = TemplateTypeParameterSourceCollector::new(&ast, &module, &resolved).unwrap();

    let first = collector.collect().unwrap();
    let second = collector.collect().unwrap();
    assert_eq!(first, second);
    assert_eq!(first.debug_text(), second.debug_text());

    let wrong_module = module_id("pkg", "other");
    assert!(matches!(
        TemplateTypeParameterSourceCollector::new(&ast, &wrong_module, &resolved),
        Err(SurfaceResolvedArenaError::ModuleMismatch)
    ));

    let stale_ast = task277r1_scoped_ast(source_id);
    assert!(matches!(
        TemplateTypeParameterSourceCollector::new(&stale_ast, &module, &resolved),
        Err(SurfaceResolvedArenaError::NodeCountMismatch)
    ));
    let stale_collector = TemplateTypeParameterSourceCollector {
        ast: &stale_ast,
        module: &module,
        resolved: &resolved,
    };
    assert!(matches!(
        stale_collector.collect(),
        Err(SurfaceResolvedArenaError::NodeCountMismatch)
    ));
}

#[test]
fn unqualified_lookup_uses_declaration_point_shadowing_and_builtins() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let dep = module_id("dep", "logic");
    let namespace = NamespacePath::new("main");
    let projections = vec![
        imported_projection(
            source_id,
            dep.clone(),
            namespace.clone(),
            "P",
            "dep::logic::P",
            Visibility::Public,
            0,
        ),
        current_private_projection(
            source_id,
            current.clone(),
            namespace.clone(),
            "P",
            "app::main::P",
            10,
            5,
        ),
        current_public_projection(
            source_id,
            current.clone(),
            namespace.clone(),
            "Fwd",
            "app::main::Fwd",
            20,
            20,
        ),
    ];
    let builtins = vec![
        BuiltinNameProjection::new(BuiltinId::new("builtin:P"), "P"),
        BuiltinNameProjection::new(BuiltinId::new("builtin:TRUE"), "TRUE"),
    ];
    let candidates = vec![
        name_candidate(source_id, current.clone(), 12, 50, "TRUE"),
        name_candidate(source_id, current.clone(), 10, 40, "Fwd"),
        name_candidate(source_id, current.clone(), 9, 30, "P"),
    ];

    let resolved =
        SymbolNameResolver::new(&projections, &builtins).resolve(&current, &namespace, &candidates);

    assert!(resolved.has_unresolved());
    assert_resolved_symbol(&resolved, 0, "app::main::P");
    assert_unresolved(&resolved, 1, NameLookupClass::Symbol, "Fwd");
    assert_resolved_builtin(&resolved, 2, "builtin:TRUE");
}

#[test]
fn qualified_lookup_restricts_namespace_and_visibility() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let dep = module_id("dep", "logic");
    let current_namespace = NamespacePath::new("main");
    let dep_namespace = NamespacePath::new("logic");
    let projections = vec![
        imported_projection(
            source_id,
            dep.clone(),
            dep_namespace.clone(),
            "Q",
            "dep::logic::Q",
            Visibility::Public,
            0,
        ),
        imported_projection(
            source_id,
            dep.clone(),
            dep_namespace.clone(),
            "Secret",
            "dep::logic::Secret",
            Visibility::Private,
            1,
        ),
        current_private_projection(
            source_id,
            current.clone(),
            current_namespace.clone(),
            "Secret",
            "app::main::Secret",
            2,
            0,
        ),
    ];
    let builtins = vec![BuiltinNameProjection::new(BuiltinId::new("builtin:Q"), "Q")];
    let qualified_public = qualified_name_candidate(
        source_id,
        current.clone(),
        0,
        20,
        "Q",
        dep.clone(),
        dep_namespace.clone(),
    );
    let qualified_private_dep = qualified_name_candidate(
        source_id,
        current.clone(),
        1,
        30,
        "Secret",
        dep,
        dep_namespace,
    );
    let qualified_current_private = qualified_name_candidate(
        source_id,
        current.clone(),
        2,
        40,
        "Secret",
        current.clone(),
        current_namespace.clone(),
    );
    let qualified_missing = qualified_name_candidate(
        source_id,
        current.clone(),
        3,
        50,
        "Missing",
        current.clone(),
        current_namespace.clone(),
    );

    let resolved = SymbolNameResolver::new(&projections, &builtins).resolve(
        &current,
        &current_namespace,
        &[
            qualified_missing,
            qualified_current_private,
            qualified_private_dep,
            qualified_public,
        ],
    );

    assert_resolved_symbol(&resolved, 0, "dep::logic::Q");
    assert_unresolved(&resolved, 1, NameLookupClass::Symbol, "Secret");
    assert_resolved_symbol(&resolved, 2, "app::main::Secret");
    assert_unresolved(&resolved, 3, NameLookupClass::Symbol, "Missing");
}

#[test]
fn overload_groups_collapse_without_names_phase_ambiguity() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let dep = module_id("dep", "logic");
    let namespace = NamespacePath::new("main");
    let overload_group = symbol_id(current.clone(), "P#group", "app::main::P#group");
    let projections = vec![
        current_public_projection(
            source_id,
            current.clone(),
            namespace.clone(),
            "P/1",
            "app::main::P/1",
            0,
            0,
        )
        .with_overload_group(overload_group.clone()),
        imported_projection(
            source_id,
            dep.clone(),
            namespace.clone(),
            "P/3",
            "dep::logic::P/3",
            Visibility::Public,
            2,
        )
        .with_overload_group(overload_group),
        imported_projection(
            source_id,
            dep.clone(),
            namespace.clone(),
            "Q/dep",
            "dep::logic::Q",
            Visibility::Public,
            3,
        ),
        imported_projection(
            source_id,
            module_id("dep", "alt"),
            namespace.clone(),
            "Q/alt",
            "dep::alt::Q",
            Visibility::Public,
            4,
        ),
    ];
    let candidates = vec![
        name_candidate(source_id, current.clone(), 11, 40, "Q"),
        name_candidate(source_id, current.clone(), 10, 30, "P"),
    ];

    let resolved =
        SymbolNameResolver::new(&projections, &[]).resolve(&current, &namespace, &candidates);

    assert_resolved_symbol(&resolved, 0, "app::main::P#group");
    let NameResolution::Ambiguous(ambiguous) = resolved
        .table()
        .get(resolved.ids()[1])
        .unwrap()
        .resolution()
    else {
        panic!("expected ambiguous Q");
    };
    let candidates = ambiguous
        .candidates()
        .iter()
        .map(|candidate| candidate.symbol().fqn().as_str())
        .collect::<Vec<_>>();
    assert_eq!(candidates, vec!["dep::alt::Q", "dep::logic::Q"]);
}

#[test]
fn failed_recovered_and_malformed_name_candidates_are_unresolved_in_order() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let projections = vec![current_public_projection(
        source_id,
        current.clone(),
        namespace.clone(),
        "Recovered",
        "app::main::Recovered",
        0,
        0,
    )];
    let candidates = vec![
        recovered_name_candidate(source_id, current.clone(), 2, 40, "Recovered"),
        empty_name_candidate(source_id, current.clone(), 1, 30),
        failed_namespace_candidate(source_id, current.clone(), 0, 20, "Ns.Missing"),
    ];

    let resolved =
        SymbolNameResolver::new(&projections, &[]).resolve(&current, &namespace, &candidates);

    assert!(resolved.has_unresolved());
    assert_unresolved(&resolved, 0, NameLookupClass::Namespace, "Ns.Missing");
    assert_unresolved(&resolved, 1, NameLookupClass::Symbol, "");
    assert_unresolved(&resolved, 2, NameLookupClass::Symbol, "Recovered");
}

#[test]
fn name_diagnostics_preserve_ambiguous_candidate_order() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let dep = module_id("dep", "logic");
    let namespace = NamespacePath::new("main");
    let projections = vec![
        imported_projection(
            source_id,
            dep,
            namespace.clone(),
            "Q/dep",
            "dep::logic::Q",
            Visibility::Public,
            20,
        ),
        imported_projection(
            source_id,
            module_id("dep", "alt"),
            namespace.clone(),
            "Q/alt",
            "dep::alt::Q",
            Visibility::Public,
            10,
        ),
    ];
    let resolved = SymbolNameResolver::new(&projections, &[]).resolve(
        &current,
        &namespace,
        &[name_candidate(source_id, current.clone(), 0, 50, "Q")],
    );

    let report = NameDiagnosticCollector::new().collect_resolution(&resolved);

    let primary = report.primary().collect::<Vec<_>>();
    assert_eq!(primary.len(), 1);
    assert_eq!(primary[0].role(), NameDiagnosticRole::Primary);
    assert_eq!(primary[0].kind(), NameDiagnosticKind::AmbiguousName);
    let candidate_fqns = primary[0]
        .symbol_candidates()
        .iter()
        .map(|candidate| candidate.symbol().fqn().as_str())
        .collect::<Vec<_>>();
    assert_eq!(candidate_fqns, vec!["dep::alt::Q", "dep::logic::Q"]);
}

#[test]
fn unresolved_import_dependency_produces_one_primary_name_diagnostic() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let import_resolution = ImportPathResolver::new(input).resolve(
        &current,
        &[ImportPathCandidate::new(
            vec!["missing".to_owned(), "thing".to_owned()],
            ImportPathPrefix::Unprefixed,
            Some("util".to_owned()),
            range(source_id, 0, 18),
            0,
        )
        .with_alias_range(range(source_id, 14, 18))],
    );
    let namespace_candidates = vec![
        candidate(source_id, 0, 25, &["util"]),
        candidate(source_id, 1, 40, &["util"]),
    ];
    let namespace_resolution = NamespaceResolver::new(input).resolve(
        &current,
        import_resolution.resolved(),
        import_resolution.unresolved(),
        &namespace_candidates,
    );
    let name_resolution = SymbolNameResolver::new(&[], &[]).resolve(
        &current,
        &namespace,
        &[
            failed_namespace_candidate(source_id, current.clone(), 0, 25, "util"),
            failed_namespace_candidate(source_id, current.clone(), 1, 40, "util"),
            failed_namespace_candidate(source_id, current.clone(), 2, 25, "util"),
        ],
    );

    let report = NameDiagnosticCollector::new()
        .with_namespace_roots(namespace_resolution.unresolved())
        .collect_resolution(&name_resolution);

    let primary = report.primary().collect::<Vec<_>>();
    assert_eq!(primary.len(), 1);
    assert_eq!(
        primary[0].kind(),
        NameDiagnosticKind::UnresolvedImportAliasDependency {
            class: ImportPathFailureClass::UnknownModule
        }
    );
    assert_eq!(primary[0].attempted_spelling(), "util");
    assert_eq!(
        primary[0].dependent_ranges(),
        &[range(source_id, 25, 29), range(source_id, 40, 44)]
    );
    let cascades = report.cascades().collect::<Vec<_>>();
    assert_eq!(cascades.len(), 5);
    assert!(
        cascades
            .iter()
            .all(|diagnostic| diagnostic.root() == primary[0].root())
    );
    let cascade_order = cascades
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.kind(),
                diagnostic.range(),
                diagnostic.name_ref().map(NameRefId::index),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cascade_order,
        vec![
            (
                NameDiagnosticKind::UnresolvedName {
                    lookup: NameLookupClass::Namespace
                },
                range(source_id, 25, 29),
                Some(0),
            ),
            (
                NameDiagnosticKind::UnresolvedName {
                    lookup: NameLookupClass::Namespace
                },
                range(source_id, 25, 29),
                Some(2),
            ),
            (
                NameDiagnosticKind::UnresolvedName {
                    lookup: NameLookupClass::Namespace
                },
                range(source_id, 40, 44),
                Some(1),
            ),
            (
                NameDiagnosticKind::UnresolvedNamespace {
                    class: NamespaceFailureClass::UnresolvedImportAlias
                },
                range(source_id, 25, 29),
                None,
            ),
            (
                NameDiagnosticKind::UnresolvedNamespace {
                    class: NamespaceFailureClass::UnresolvedImportAlias
                },
                range(source_id, 40, 44),
                None,
            ),
        ]
    );
    assert_eq!(
        cascades
            .iter()
            .filter(|diagnostic| matches!(
                diagnostic.kind(),
                NameDiagnosticKind::UnresolvedNamespace { .. }
            ))
            .count(),
        2
    );
    assert_eq!(
        cascades
            .iter()
            .filter(|diagnostic| matches!(
                diagnostic.kind(),
                NameDiagnosticKind::UnresolvedName {
                    lookup: NameLookupClass::Namespace
                }
            ))
            .count(),
        3
    );
}

#[test]
fn name_diagnostics_use_mixed_root_ordering() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let import_resolution = ImportPathResolver::new(input).resolve(
        &current,
        &[ImportPathCandidate::new(
            vec!["missing".to_owned(), "thing".to_owned()],
            ImportPathPrefix::Unprefixed,
            Some("util".to_owned()),
            range(source_id, 0, 18),
            0,
        )
        .with_alias_range(range(source_id, 14, 18))],
    );
    let namespace_resolution = NamespaceResolver::new(input).resolve(
        &current,
        import_resolution.resolved(),
        import_resolution.unresolved(),
        &[candidate(source_id, 0, 25, &["util"])],
    );
    let projections = vec![
        imported_projection(
            source_id,
            module_id("dep", "logic"),
            namespace.clone(),
            "Q/dep",
            "dep::logic::Q",
            Visibility::Public,
            20,
        ),
        imported_projection(
            source_id,
            module_id("dep", "alt"),
            namespace.clone(),
            "Q/alt",
            "dep::alt::Q",
            Visibility::Public,
            10,
        ),
    ];
    let name_resolution = SymbolNameResolver::new(&projections, &[]).resolve(
        &current,
        &namespace,
        &[
            name_candidate(source_id, current.clone(), 1, 70, "Q"),
            failed_namespace_candidate(source_id, current.clone(), 0, 25, "util"),
        ],
    );

    let report = NameDiagnosticCollector::new()
        .with_namespace_roots(namespace_resolution.unresolved())
        .collect_resolution(&name_resolution);

    let order = report
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.role(),
                diagnostic.kind(),
                diagnostic.attempted_spelling().to_owned(),
                diagnostic.range(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        order,
        vec![
            (
                NameDiagnosticRole::Primary,
                NameDiagnosticKind::UnresolvedImportAliasDependency {
                    class: ImportPathFailureClass::UnknownModule
                },
                "util".to_owned(),
                range(source_id, 0, 18),
            ),
            (
                NameDiagnosticRole::Cascade,
                NameDiagnosticKind::UnresolvedName {
                    lookup: NameLookupClass::Namespace
                },
                "util".to_owned(),
                range(source_id, 25, 29),
            ),
            (
                NameDiagnosticRole::Cascade,
                NameDiagnosticKind::UnresolvedNamespace {
                    class: NamespaceFailureClass::UnresolvedImportAlias
                },
                "util".to_owned(),
                range(source_id, 25, 29),
            ),
            (
                NameDiagnosticRole::Primary,
                NameDiagnosticKind::AmbiguousName,
                "Q".to_owned(),
                range(source_id, 70, 71),
            ),
        ]
    );
}

#[test]
fn name_diagnostics_keep_namespace_payloads_on_import_dependency_primaries() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let import_resolution = ImportPathResolver::new(input).resolve(
        &current,
        &[
            ImportPathCandidate::new(
                vec!["app".to_owned(), "util".to_owned()],
                ImportPathPrefix::Unprefixed,
                Some("Shared".to_owned()),
                range(source_id, 30, 46),
                9,
            ),
            ImportPathCandidate::new(
                vec!["dep".to_owned(), "logic".to_owned()],
                ImportPathPrefix::Unprefixed,
                Some("Shared".to_owned()),
                range(source_id, 0, 16),
                3,
            ),
        ],
    );
    let namespace_resolution = NamespaceResolver::new(input).resolve(
        &current,
        import_resolution.resolved(),
        import_resolution.unresolved(),
        &[candidate(source_id, 0, 60, &["Shared"])],
    );

    let report = NameDiagnosticCollector::new()
        .with_namespace_roots(namespace_resolution.unresolved())
        .collect(&NameRefTable::new());

    let primaries = report.primary().collect::<Vec<_>>();
    assert_eq!(primaries.len(), 2);
    for primary in primaries {
        assert_eq!(
            primary.kind(),
            NameDiagnosticKind::UnresolvedImportAliasDependency {
                class: ImportPathFailureClass::DuplicateAlias
            }
        );
        assert_eq!(
            primary.normalized_namespace_prefix(),
            &["Shared".to_owned()]
        );
        let candidates = primary
            .namespace_candidates()
            .iter()
            .map(|candidate| {
                (
                    candidate.stable_variant(),
                    candidate.target().package().as_str(),
                    candidate.target().path().as_str(),
                    candidate.range(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            candidates,
            vec![
                (
                    "import-alias-target",
                    "app",
                    "util",
                    range(source_id, 30, 46),
                ),
                (
                    "import-alias-target",
                    "dep",
                    "logic",
                    range(source_id, 0, 16),
                ),
            ]
        );
        assert_eq!(primary.dependent_ranges(), &[range(source_id, 60, 66)]);
    }
}

#[test]
fn name_diagnostics_include_reserved_roots_in_normalized_prefixes() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let namespace_resolution = NamespaceResolver::new(input).resolve(
        &current,
        &[],
        &[],
        &[
            candidate(source_id, 0, 10, &["pub", "math", "missing"]),
            candidate(source_id, 1, 40, &["std", "missing"]),
        ],
    );

    let report = NameDiagnosticCollector::new()
        .with_namespace_roots(namespace_resolution.unresolved())
        .collect(&NameRefTable::new());

    let prefixes = report
        .primary()
        .map(|diagnostic| diagnostic.normalized_namespace_prefix().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(
        prefixes,
        vec![
            vec!["pub".to_owned(), "math".to_owned()],
            vec!["std".to_owned()],
        ]
    );
}

#[test]
fn name_diagnostics_match_namespace_roots_by_spelling_and_range() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let first_namespace = unresolved_namespace_fixture(
        source_id,
        0,
        10,
        "Ns",
        NamespaceFailureClass::UnknownNamespaceSegment,
        &["Ns"],
    );
    let second_namespace = unresolved_namespace_fixture(
        source_id,
        1,
        40,
        "Ns",
        NamespaceFailureClass::RecoveredSyntax,
        &["RecoveredNs"],
    );
    let first_order = vec![first_namespace.clone(), second_namespace.clone()];
    let second_order = vec![second_namespace, first_namespace];
    let mut name_refs = NameRefTable::new();
    let first_name = failed_namespace_candidate(source_id, current.clone(), 1, 40, "Ns");
    name_refs.insert(NameRefEntry::new(
        first_name.site().clone(),
        unresolved_name(&first_name, NameLookupClass::Namespace),
        first_name.origin().clone(),
    ));
    let second_name = failed_namespace_candidate(source_id, current.clone(), 0, 10, "Ns");
    name_refs.insert(NameRefEntry::new(
        second_name.site().clone(),
        unresolved_name(&second_name, NameLookupClass::Namespace),
        second_name.origin().clone(),
    ));

    let first_report = NameDiagnosticCollector::new()
        .with_namespace_roots(&first_order)
        .collect(&name_refs);
    let second_report = NameDiagnosticCollector::new()
        .with_namespace_roots(&second_order)
        .collect(&name_refs);

    let first_roots = primary_root_ranges(&first_report);
    let second_roots = primary_root_ranges(&second_report);
    assert_eq!(first_roots, second_roots);
    assert_eq!(
        first_roots,
        vec![
            (NameDiagnosticRootId::new(0), range(source_id, 10, 12)),
            (NameDiagnosticRootId::new(1), range(source_id, 40, 42)),
        ]
    );
    for cascade in first_report.cascades() {
        assert_eq!(cascade.root_range(), cascade.range());
    }
}

#[test]
fn name_diagnostics_order_same_range_by_class_spelling_and_candidate_key() {
    let source_id = source_id();
    let current = module_id("app", "main");
    let mut table = NameRefTable::new();
    insert_unresolved_name_entry(&mut table, source_id, current.clone(), 0, 50, "Z");
    insert_ambiguous_name_entry(
        &mut table,
        source_id,
        current.clone(),
        1,
        50,
        "Q",
        &[("B", "app::main::B", 20)],
    );
    insert_unresolved_name_entry(&mut table, source_id, current.clone(), 2, 50, "A");
    insert_ambiguous_name_entry(
        &mut table,
        source_id,
        current,
        3,
        50,
        "Q",
        &[("A", "app::main::A", 10)],
    );

    let report = NameDiagnosticCollector::new().collect(&table);

    let order = report
        .primary()
        .map(|diagnostic| {
            (
                diagnostic.kind(),
                diagnostic.attempted_spelling().to_owned(),
                diagnostic
                    .symbol_candidates()
                    .first()
                    .map(|candidate| candidate.symbol().fqn().as_str().to_owned()),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        order,
        vec![
            (
                NameDiagnosticKind::AmbiguousName,
                "Q".to_owned(),
                Some("app::main::A".to_owned()),
            ),
            (
                NameDiagnosticKind::AmbiguousName,
                "Q".to_owned(),
                Some("app::main::B".to_owned()),
            ),
            (
                NameDiagnosticKind::UnresolvedName {
                    lookup: NameLookupClass::Symbol
                },
                "A".to_owned(),
                None,
            ),
            (
                NameDiagnosticKind::UnresolvedName {
                    lookup: NameLookupClass::Symbol
                },
                "Z".to_owned(),
                None,
            ),
        ]
    );
}

#[test]
fn dot_chain_local_binding_defers_selector_without_namespace_lookup() {
    let source_id = source_id();
    let provider = fixture_provider();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let local_scope = LocalTermScope::new(vec![1, 2]);
    let chain = dot_chain_candidate(
        source_id,
        current.clone(),
        10,
        40,
        &["dep", "logic", "P"],
        local_scope.clone(),
    );
    let base_node = chain.base_node();
    let local_terms = vec![LocalTermBinding::new(
        "dep",
        LocalTermScope::new(vec![1]),
        range(source_id, 0, 3),
        0,
    )];

    let resolved = DotChainFinalizer::new(
        NamespaceResolver::new(ModuleIndexInput::new(&provider)),
        SymbolNameResolver::new(&[], &[]),
        &local_terms,
    )
    .finalize(&current, &namespace, &[], &[], &[chain]);

    assert!(resolved.namespaces().resolved().is_empty());
    assert!(resolved.namespaces().unresolved().is_empty());
    let entry = resolved.table().get(resolved.ids()[0]).unwrap();
    let NameResolution::DeferredSelector(selector) = entry.resolution() else {
        panic!("expected deferred selector");
    };
    assert_eq!(selector.base(), base_node);
    assert_eq!(selector.member(), "logic.P");
    assert_eq!(selector.range(), range(source_id, 40, 51));
}

#[test]
fn dot_chain_without_visible_local_resolves_namespace_symbol() {
    let source_id = source_id();
    let provider = fixture_provider();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let dep = module_id("dep", "logic");
    let projections = vec![imported_projection(
        source_id,
        dep,
        NamespacePath::new("logic"),
        "P",
        "dep::logic::P",
        Visibility::Public,
        0,
    )];
    let chain = dot_chain_candidate(
        source_id,
        current.clone(),
        10,
        40,
        &["dep", "logic", "P"],
        LocalTermScope::new(vec![1, 2]),
    );
    let out_of_scope_locals = vec![LocalTermBinding::new(
        "dep",
        LocalTermScope::new(vec![9]),
        range(source_id, 0, 3),
        0,
    )];

    let resolved = DotChainFinalizer::new(
        NamespaceResolver::new(ModuleIndexInput::new(&provider)),
        SymbolNameResolver::new(&projections, &[]),
        &out_of_scope_locals,
    )
    .finalize(&current, &namespace, &[], &[], &[chain]);

    assert_eq!(resolved.namespaces().resolved().len(), 1);
    let entry = resolved.table().get(resolved.ids()[0]).unwrap();
    let NameResolution::Resolved(symbol) = entry.resolution() else {
        panic!("expected resolved qualified symbol");
    };
    assert_eq!(symbol.symbol().fqn().as_str(), "dep::logic::P");
    assert_eq!(symbol.range(), range(source_id, 50, 51));
}

#[test]
fn dot_chain_uses_innermost_visible_local_binding() {
    let source_id = source_id();
    let provider = fixture_provider();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let chain = dot_chain_candidate(
        source_id,
        current.clone(),
        10,
        40,
        &["x", "field"],
        LocalTermScope::new(vec![1, 2, 3]),
    );
    let local_terms = vec![
        LocalTermBinding::new("x", LocalTermScope::new(vec![1]), range(source_id, 0, 1), 0),
        LocalTermBinding::new(
            "x",
            LocalTermScope::new(vec![1, 2]),
            range(source_id, 10, 11),
            0,
        ),
        LocalTermBinding::new(
            "x",
            LocalTermScope::new(vec![1, 2]),
            range(source_id, 12, 13),
            5,
        ),
        LocalTermBinding::new(
            "x",
            LocalTermScope::new(vec![1, 2]),
            range(source_id, 14, 15),
            5,
        ),
        LocalTermBinding::new(
            "x",
            LocalTermScope::new(vec![1, 2, 3]),
            range(source_id, 20, 21),
            20,
        ),
    ];
    let finalizer = DotChainFinalizer::new(
        NamespaceResolver::new(ModuleIndexInput::new(&provider)),
        SymbolNameResolver::new(&[], &[]),
        &local_terms,
    );

    let selected = finalizer.local_term_binding(&chain).unwrap();
    assert_eq!(selected.declaration_range(), range(source_id, 14, 15));

    let resolved = finalizer.finalize(&current, &namespace, &[], &[], &[chain]);
    let entry = resolved.table().get(resolved.ids()[0]).unwrap();
    assert!(matches!(
        entry.resolution(),
        NameResolution::DeferredSelector(selector) if selector.member() == "field"
    ));
}

#[test]
fn dot_chain_unresolved_namespace_uses_earliest_failed_segment() {
    let source_id = source_id();
    let provider = fixture_provider();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let chain = dot_chain_candidate(
        source_id,
        current.clone(),
        0,
        20,
        &["pub", "math", "missing", "P"],
        LocalTermScope::new(vec![1]),
    );

    let resolved = DotChainFinalizer::new(
        NamespaceResolver::new(ModuleIndexInput::new(&provider)),
        SymbolNameResolver::new(&[], &[]),
        &[],
    )
    .finalize(&current, &namespace, &[], &[], &[chain]);

    assert_eq!(resolved.namespaces().unresolved().len(), 1);
    let entry = resolved.table().get(resolved.ids()[0]).unwrap();
    let NameResolution::Unresolved(unresolved) = entry.resolution() else {
        panic!("expected unresolved namespace");
    };
    assert_eq!(unresolved.lookup(), NameLookupClass::Namespace);
    assert_eq!(unresolved.range(), range(source_id, 29, 36));

    let report = NameDiagnosticCollector::new()
        .with_namespace_roots(resolved.namespaces().unresolved())
        .collect(resolved.table());
    assert_eq!(report.primary().count(), 1);
    let cascades = report.cascades().collect::<Vec<_>>();
    assert_eq!(cascades.len(), 1);
    assert_eq!(cascades[0].root_range(), range(source_id, 20, 36));
    assert_eq!(cascades[0].range(), range(source_id, 29, 36));
}

#[test]
fn recovered_inputs_do_not_emit_name_diagnostic_roots() {
    let source_id = source_id();
    let provider = fixture_provider();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let recovered_reference = recovered_name_candidate(source_id, current.clone(), 0, 10, "R");

    let resolved_names =
        SymbolNameResolver::new(&[], &[]).resolve(&current, &namespace, &[recovered_reference]);

    assert!(resolved_names.has_unresolved());
    let name_report = NameDiagnosticCollector::new().collect_resolution(&resolved_names);
    assert!(name_report.is_empty());

    let recovered_namespace = candidate(source_id, 1, 20, &["RecoveredNs"]).with_recovered();
    let namespace_resolution = NamespaceResolver::new(ModuleIndexInput::new(&provider)).resolve(
        &current,
        &[],
        &[],
        &[recovered_namespace],
    );

    assert_eq!(namespace_resolution.unresolved().len(), 1);
    assert_eq!(
        namespace_resolution.unresolved()[0].class(),
        NamespaceFailureClass::RecoveredSyntax
    );
    let namespace_report = NameDiagnosticCollector::new()
        .with_namespace_roots(namespace_resolution.unresolved())
        .collect(&NameRefTable::new());
    assert!(namespace_report.is_empty());

    let recovered_chain = dot_chain_candidate(
        source_id,
        current.clone(),
        2,
        40,
        &["x", "field"],
        LocalTermScope::new(vec![1]),
    )
    .with_recovered();
    let dot_resolution = DotChainFinalizer::new(
        NamespaceResolver::new(ModuleIndexInput::new(&provider)),
        SymbolNameResolver::new(&[], &[]),
        &[],
    )
    .finalize(&current, &namespace, &[], &[], &[recovered_chain]);
    let entry = dot_resolution.table().get(dot_resolution.ids()[0]).unwrap();

    assert!(entry.origin().is_recovered());
    assert!(matches!(
        entry.resolution(),
        NameResolution::Unresolved(unresolved)
            if unresolved.lookup() == NameLookupClass::Selector
    ));
    let dot_report = NameDiagnosticCollector::new()
        .with_namespace_roots(dot_resolution.namespaces().unresolved())
        .collect(dot_resolution.table());
    assert!(dot_report.is_empty());
}

#[test]
fn dot_chain_finalizer_orders_out_of_order_inputs() {
    let source_id = source_id();
    let provider = fixture_provider();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let local_terms = vec![LocalTermBinding::new(
        "x",
        LocalTermScope::new(vec![1]),
        range(source_id, 0, 1),
        0,
    )];
    let late = dot_chain_candidate(
        source_id,
        current.clone(),
        2,
        40,
        &["x", "late"],
        LocalTermScope::new(vec![1]),
    );
    let early = dot_chain_candidate(
        source_id,
        current.clone(),
        1,
        20,
        &["x", "early"],
        LocalTermScope::new(vec![1]),
    );

    let resolved = DotChainFinalizer::new(
        NamespaceResolver::new(ModuleIndexInput::new(&provider)),
        SymbolNameResolver::new(&[], &[]),
        &local_terms,
    )
    .finalize(&current, &namespace, &[], &[], &[late, early]);

    let spellings = resolved
        .ids()
        .iter()
        .map(|id| resolved.table().get(*id).unwrap().site().spelling())
        .collect::<Vec<_>>();
    assert_eq!(spellings, vec!["x.early", "x.late"]);
}

#[test]
fn dot_chain_malformed_or_recovered_inputs_stay_unresolved() {
    let source_id = source_id();
    let provider = fixture_provider();
    let current = module_id("app", "main");
    let namespace = NamespacePath::new("main");
    let malformed = dot_chain_candidate(
        source_id,
        current.clone(),
        0,
        20,
        &["x", ""],
        LocalTermScope::new(vec![1]),
    );
    let recovered = dot_chain_candidate(
        source_id,
        current.clone(),
        1,
        40,
        &["x", "field"],
        LocalTermScope::new(vec![1]),
    )
    .with_recovered();
    let single = dot_chain_candidate(
        source_id,
        current.clone(),
        2,
        60,
        &["x"],
        LocalTermScope::new(vec![1]),
    );

    let resolved = DotChainFinalizer::new(
        NamespaceResolver::new(ModuleIndexInput::new(&provider)),
        SymbolNameResolver::new(&[], &[]),
        &[],
    )
    .finalize(
        &current,
        &namespace,
        &[],
        &[],
        &[malformed, recovered, single],
    );

    assert_unresolved_entry(
        resolved.table(),
        resolved.ids()[0],
        NameLookupClass::Selector,
        range(source_id, 22, 22),
    );
    assert_unresolved_entry(
        resolved.table(),
        resolved.ids()[1],
        NameLookupClass::Selector,
        range(source_id, 40, 47),
    );
    assert_unresolved_entry(
        resolved.table(),
        resolved.ids()[2],
        NameLookupClass::Selector,
        range(source_id, 60, 61),
    );
}

#[test]
fn resolver_resolves_alias_roots_and_package_names_deterministically() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let import_resolution = ImportPathResolver::new(input).resolve(
        &current,
        &[ImportPathCandidate::new(
            vec!["dep".to_owned(), "logic".to_owned()],
            ImportPathPrefix::Unprefixed,
            Some("Logic".to_owned()),
            range(source_id, 0, 16),
            0,
        )
        .with_alias_range(range(source_id, 14, 19))],
    );
    let candidates = vec![
        candidate(source_id, 0, 0, &["Logic"]),
        candidate(source_id, 1, 20, &["pub", "math", "algebra", "group"]),
        candidate(source_id, 2, 50, &["dep", "logic"]),
        candidate(source_id, 3, 70, &["util"]),
        candidate(source_id, 4, 90, &["std", "core"]),
        candidate(source_id, 5, 105, &["pkg", "vendor", "lib"]),
        candidate(source_id, 6, 125, &["dev", "sandbox", "tools"]),
        candidate(source_id, 7, 150, &["ext", "mirror", "logic"]),
    ];

    let resolved = NamespaceResolver::new(input).resolve(
        &current,
        import_resolution.resolved(),
        import_resolution.unresolved(),
        &candidates,
    );

    assert!(resolved.unresolved().is_empty());
    let targets = resolved
        .resolved()
        .iter()
        .map(|path| {
            (
                path.spelling().to_owned(),
                path.target().package().as_str().to_owned(),
                path.target().path().as_str().to_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        targets,
        vec![
            ("Logic".to_owned(), "dep".to_owned(), "logic".to_owned()),
            (
                "pub.math.algebra.group".to_owned(),
                "dep".to_owned(),
                "algebra.group".to_owned()
            ),
            ("dep.logic".to_owned(), "dep".to_owned(), "logic".to_owned()),
            ("util".to_owned(), "app".to_owned(), "util".to_owned()),
            (
                "std.core".to_owned(),
                "stdpkg".to_owned(),
                "core".to_owned()
            ),
            (
                "pkg.vendor.lib".to_owned(),
                "pkgdep".to_owned(),
                "lib".to_owned()
            ),
            (
                "dev.sandbox.tools".to_owned(),
                "devdep".to_owned(),
                "tools".to_owned()
            ),
            (
                "ext.mirror.logic".to_owned(),
                "extdep".to_owned(),
                "logic".to_owned()
            ),
        ]
    );
    assert!(matches!(
        resolved.resolved()[0].origin(),
        NamespaceResolutionOrigin::ImportAlias { alias, .. } if alias == "Logic"
    ));
    assert!(matches!(
        resolved.resolved()[1].origin(),
        NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Pub, matched_prefix, .. }
            if matched_prefix == &vec!["math".to_owned()]
    ));
    assert!(matches!(
        resolved.resolved()[2].origin(),
        NamespaceResolutionOrigin::PackageNameBinding { matched_prefix, .. }
            if matched_prefix == &vec!["dep".to_owned()]
    ));
    assert!(matches!(
        resolved.resolved()[3].origin(),
        NamespaceResolutionOrigin::CurrentPackage { package } if package.as_str() == "app"
    ));
    assert!(matches!(
        resolved.resolved()[4].origin(),
        NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Std, matched_prefix, .. }
            if matched_prefix.is_empty()
    ));
    assert!(matches!(
        resolved.resolved()[5].origin(),
        NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Pkg, matched_prefix, .. }
            if matched_prefix == &vec!["vendor".to_owned()]
    ));
    assert!(matches!(
        resolved.resolved()[6].origin(),
        NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Dev, matched_prefix, .. }
            if matched_prefix == &vec!["sandbox".to_owned()]
    ));
    assert!(matches!(
        resolved.resolved()[7].origin(),
        NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Ext, matched_prefix, .. }
            if matched_prefix == &vec!["mirror".to_owned()]
    ));
}

#[test]
fn missing_namespace_records_the_earliest_failing_segment_range() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let candidates = vec![
        candidate(source_id, 0, 10, &["pub", "unknown", "logic"]),
        candidate(source_id, 1, 40, &["pub", "math", "missing"]),
        candidate(source_id, 2, 70, &["dep", "algebra", "missing"]),
    ];

    let resolved = NamespaceResolver::new(input).resolve(&current, &[], &[], &candidates);

    assert!(resolved.resolved().is_empty());
    let failures = resolved
        .unresolved()
        .iter()
        .map(|path| {
            (
                path.spelling().to_owned(),
                path.class(),
                path.failed_segment().map(NamespacePathSegment::spelling),
                path.failed_segment().map(NamespacePathSegment::range),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        failures,
        vec![
            (
                "pub.unknown.logic".to_owned(),
                NamespaceFailureClass::UnknownNamespaceSegment,
                Some("unknown"),
                Some(range(source_id, 14, 21)),
            ),
            (
                "pub.math.missing".to_owned(),
                NamespaceFailureClass::UnknownModule,
                Some("missing"),
                Some(range(source_id, 49, 56)),
            ),
            (
                "dep.algebra.missing".to_owned(),
                NamespaceFailureClass::UnknownModule,
                Some("missing"),
                Some(range(source_id, 82, 89)),
            ),
        ]
    );
}

#[test]
fn longest_namespace_bindings_win_over_shorter_prefixes() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let candidates = vec![
        candidate(source_id, 0, 0, &["dep", "nested", "logic"]),
        candidate(source_id, 1, 30, &["pub", "math", "logic", "core"]),
    ];

    let resolved = NamespaceResolver::new(input).resolve(&current, &[], &[], &candidates);

    assert!(resolved.unresolved().is_empty());
    let targets = resolved
        .resolved()
        .iter()
        .map(|path| {
            (
                path.target().package().as_str().to_owned(),
                path.target().path().as_str().to_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        targets,
        vec![
            ("altdep".to_owned(), "logic".to_owned()),
            ("altdep".to_owned(), "core".to_owned()),
        ]
    );
    assert!(matches!(
        resolved.resolved()[0].origin(),
        NamespaceResolutionOrigin::PackageNameBinding { matched_prefix, .. }
            if matched_prefix == &vec!["dep".to_owned(), "nested".to_owned()]
    ));
    assert!(matches!(
        resolved.resolved()[1].origin(),
        NamespaceResolutionOrigin::ReservedRoot { root: NamespaceRoot::Pub, matched_prefix, .. }
            if matched_prefix == &vec!["math".to_owned(), "logic".to_owned()]
    ));
}

#[test]
fn malformed_namespace_paths_are_unresolved_in_deterministic_order() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let candidates = vec![
        candidate(source_id, 3, 50, &["Recovered"]).with_recovered(),
        candidate(source_id, 1, 20, &[""]),
        NamespacePathCandidate::new(Vec::new(), range(source_id, 0, 0), 0),
        candidate(source_id, 2, 30, &["pub", "unknown"]),
    ];

    let resolved = NamespaceResolver::new(input).resolve(&current, &[], &[], &candidates);

    assert!(resolved.resolved().is_empty());
    let failures = resolved
        .unresolved()
        .iter()
        .map(|path| {
            (
                path.ordinal(),
                path.class(),
                path.failed_segment().map(NamespacePathSegment::spelling),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        failures,
        vec![
            (0, NamespaceFailureClass::EmptyPath, None),
            (1, NamespaceFailureClass::IllegalCandidateState, Some("")),
            (
                2,
                NamespaceFailureClass::UnknownNamespaceSegment,
                Some("unknown")
            ),
            (3, NamespaceFailureClass::RecoveredSyntax, Some("Recovered")),
        ]
    );
}

#[test]
fn recovered_and_ambiguous_alias_paths_remain_explicit() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let import_resolution = ImportPathResolver::new(input).resolve(
        &current,
        &[
            ImportPathCandidate::new(
                vec!["dep".to_owned(), "logic".to_owned()],
                ImportPathPrefix::Unprefixed,
                Some("Shared".to_owned()),
                range(source_id, 0, 16),
                0,
            ),
            ImportPathCandidate::new(
                vec!["dep".to_owned(), "algebra".to_owned(), "group".to_owned()],
                ImportPathPrefix::Unprefixed,
                Some("Group".to_owned()),
                range(source_id, 18, 40),
                1,
            ),
        ],
    );
    let mut ambiguous_imports = import_resolution.resolved().to_vec();
    let duplicate_target = ImportPathResolver::new(input)
        .resolve(
            &current,
            &[ImportPathCandidate::new(
                vec!["app".to_owned(), "util".to_owned()],
                ImportPathPrefix::Unprefixed,
                Some("Shared".to_owned()),
                range(source_id, 42, 58),
                2,
            )],
        )
        .resolved()[0]
        .clone();
    ambiguous_imports.push(duplicate_target);
    let candidates = vec![
        candidate(source_id, 0, 60, &["Shared"]),
        candidate(source_id, 1, 70, &["Group", "extra"]),
        candidate(source_id, 2, 85, &["Recovered"]).with_recovered(),
    ];

    let resolved = NamespaceResolver::new(input).resolve(
        &current,
        &ambiguous_imports,
        import_resolution.unresolved(),
        &candidates,
    );

    assert!(resolved.resolved().is_empty());
    let failures = resolved
        .unresolved()
        .iter()
        .map(|path| {
            (
                path.spelling().to_owned(),
                path.class(),
                path.failed_segment().map(NamespacePathSegment::spelling),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        failures,
        vec![
            (
                "Shared".to_owned(),
                NamespaceFailureClass::AmbiguousImportAlias,
                Some("Shared"),
            ),
            (
                "Group.extra".to_owned(),
                NamespaceFailureClass::UnknownNamespaceSegment,
                Some("extra"),
            ),
            (
                "Recovered".to_owned(),
                NamespaceFailureClass::RecoveredSyntax,
                Some("Recovered"),
            ),
        ]
    );
    let ambiguous = &resolved.unresolved()[0];
    assert!(ambiguous.partial().unwrap().package().is_none());
    let candidate_targets = ambiguous
        .candidate_targets()
        .iter()
        .map(|target| {
            (
                target.target().package().as_str().to_owned(),
                target.target().path().as_str().to_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        candidate_targets,
        vec![
            ("app".to_owned(), "util".to_owned()),
            ("dep".to_owned(), "logic".to_owned()),
        ]
    );
    assert!(resolved.unresolved()[2].recovered());
}

#[test]
fn duplicate_import_aliases_drive_ambiguous_namespace_payloads_deterministically() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let import_resolution = ImportPathResolver::new(input).resolve(
        &current,
        &[
            ImportPathCandidate::new(
                vec!["app".to_owned(), "util".to_owned()],
                ImportPathPrefix::Unprefixed,
                Some("Shared".to_owned()),
                range(source_id, 30, 46),
                9,
            ),
            ImportPathCandidate::new(
                vec!["dep".to_owned(), "logic".to_owned()],
                ImportPathPrefix::Unprefixed,
                Some("Shared".to_owned()),
                range(source_id, 0, 16),
                3,
            ),
        ],
    );
    assert!(import_resolution.resolved().is_empty());

    let candidates = vec![
        candidate(source_id, 2, 60, &["Shared"]),
        candidate(source_id, 1, 80, &["pub", "math", "algebra", "group"]),
        candidate(source_id, 0, 110, &["util"]),
    ];

    let resolved = NamespaceResolver::new(input).resolve(
        &current,
        import_resolution.resolved(),
        import_resolution.unresolved(),
        &candidates,
    );

    let resolved_spellings = resolved
        .resolved()
        .iter()
        .map(ResolvedNamespacePath::spelling)
        .collect::<Vec<_>>();
    assert_eq!(resolved_spellings, vec!["util", "pub.math.algebra.group"]);
    let ambiguous = &resolved.unresolved()[0];
    assert_eq!(ambiguous.spelling(), "Shared");
    assert_eq!(
        ambiguous.class(),
        NamespaceFailureClass::AmbiguousImportAlias
    );
    let dependency_ordinals = ambiguous
        .import_dependencies()
        .iter()
        .map(|dependency| (dependency.ordinal(), dependency.class()))
        .collect::<Vec<_>>();
    assert_eq!(
        dependency_ordinals,
        vec![
            (3, ImportPathFailureClass::DuplicateAlias),
            (9, ImportPathFailureClass::DuplicateAlias),
        ]
    );
    let candidate_targets = ambiguous
        .candidate_targets()
        .iter()
        .map(|target| {
            (
                target.target().package().as_str().to_owned(),
                target.target().path().as_str().to_owned(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        candidate_targets,
        vec![
            ("app".to_owned(), "util".to_owned()),
            ("dep".to_owned(), "logic".to_owned()),
        ]
    );
}

#[test]
fn unresolved_import_alias_blocks_namespace_fallback_with_dependency_payload() {
    let source_id = source_id();
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let import_resolution = ImportPathResolver::new(input).resolve(
        &current,
        &[ImportPathCandidate::new(
            vec!["missing".to_owned(), "thing".to_owned()],
            ImportPathPrefix::Unprefixed,
            Some("util".to_owned()),
            range(source_id, 0, 18),
            0,
        )
        .with_alias_range(range(source_id, 14, 18))],
    );
    assert!(import_resolution.resolved().is_empty());

    let candidates = vec![candidate(source_id, 0, 25, &["util"])];
    let resolved = NamespaceResolver::new(input).resolve(
        &current,
        import_resolution.resolved(),
        import_resolution.unresolved(),
        &candidates,
    );

    assert!(resolved.resolved().is_empty());
    let unresolved = &resolved.unresolved()[0];
    assert_eq!(
        unresolved.class(),
        NamespaceFailureClass::UnresolvedImportAlias
    );
    assert_eq!(
        unresolved
            .failed_segment()
            .map(NamespacePathSegment::spelling),
        Some("util")
    );
    assert!(unresolved.candidate_targets().is_empty());
    let dependencies = unresolved.import_dependencies();
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].alias(), "util");
    assert_eq!(
        dependencies[0].class(),
        ImportPathFailureClass::UnknownModule
    );
    assert_eq!(
        dependencies[0].alias_range(),
        Some(range(source_id, 14, 18))
    );
}

#[test]
fn stale_namespace_bindings_are_provider_errors() {
    let source_id = source_id();
    let provider = WorkspaceStubModuleIndexProvider::new(
        vec![package("app")],
        vec![namespace(NamespaceRoot::PackageName, &["ghost"], "ghost")],
        vec![workspace_module("app", "main")],
        Vec::new(),
    );
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let candidates = vec![candidate(source_id, 0, 0, &["ghost", "logic"])];

    let resolved = NamespaceResolver::new(input).resolve(&current, &[], &[], &candidates);

    assert!(resolved.resolved().is_empty());
    let unresolved = &resolved.unresolved()[0];
    assert_eq!(unresolved.class(), NamespaceFailureClass::ProviderError);
    assert_eq!(
        unresolved
            .failed_segment()
            .map(NamespacePathSegment::spelling),
        Some("ghost")
    );
}

#[test]
fn stale_empty_prefix_reserved_root_bindings_report_the_root_segment() {
    let source_id = source_id();
    let provider = WorkspaceStubModuleIndexProvider::new(
        vec![package("app")],
        vec![namespace(NamespaceRoot::Std, &[], "missingstd")],
        vec![workspace_module("app", "main")],
        Vec::new(),
    );
    let input = ModuleIndexInput::new(&provider);
    let current = module_id("app", "main");
    let candidates = vec![candidate(source_id, 0, 0, &["std", "core"])];

    let resolved = NamespaceResolver::new(input).resolve(&current, &[], &[], &candidates);

    assert!(resolved.resolved().is_empty());
    let unresolved = &resolved.unresolved()[0];
    assert_eq!(unresolved.class(), NamespaceFailureClass::ProviderError);
    assert_eq!(
        unresolved
            .failed_segment()
            .map(NamespacePathSegment::spelling),
        Some("std")
    );
}

fn name_candidate(
    source_id: SourceId,
    module: ModuleId,
    ordinal: usize,
    start: usize,
    spelling: &str,
) -> NameReferenceCandidate {
    let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
    NameReferenceCandidate::unqualified(site, origin, ordinal)
}

fn qualified_name_candidate(
    source_id: SourceId,
    module: ModuleId,
    ordinal: usize,
    start: usize,
    spelling: &str,
    target: ModuleId,
    namespace: NamespacePath,
) -> NameReferenceCandidate {
    let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
    NameReferenceCandidate::qualified(site, origin, ordinal, target, namespace)
}

fn failed_namespace_candidate(
    source_id: SourceId,
    module: ModuleId,
    ordinal: usize,
    start: usize,
    spelling: &str,
) -> NameReferenceCandidate {
    let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
    NameReferenceCandidate::failed_namespace(site, origin, ordinal)
}

fn recovered_name_candidate(
    source_id: SourceId,
    module: ModuleId,
    ordinal: usize,
    start: usize,
    spelling: &str,
) -> NameReferenceCandidate {
    let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
    NameReferenceCandidate::unqualified(site, origin.recovered(), ordinal)
}

fn empty_name_candidate(
    source_id: SourceId,
    module: ModuleId,
    ordinal: usize,
    start: usize,
) -> NameReferenceCandidate {
    name_candidate(source_id, module, ordinal, start, "")
}

fn reference_site(
    source_id: SourceId,
    module: ModuleId,
    start: usize,
    spelling: &str,
    ordinal: usize,
) -> (ReferenceSite, SemanticOrigin) {
    let range = range(source_id, start, start + spelling.len());
    let origin = SemanticOrigin::new(
        source_id,
        module,
        SourceAnchor::Range(range),
        vec![ordinal as u32],
    );
    let mut arena = ResolvedArenaBuilder::new();
    let node = arena
        .push(ResolvedNode::new(
            SurfaceNodeKind::Reference,
            Vec::new(),
            origin.clone(),
        ))
        .unwrap();
    (ReferenceSite::new(node, range, spelling), origin)
}

fn current_public_projection(
    source_id: SourceId,
    module: ModuleId,
    namespace: NamespacePath,
    local: &str,
    fqn: &str,
    declaration_start: usize,
    visible_after_ordinal: usize,
) -> NameSymbolProjection {
    NameSymbolProjection::current_module(
        symbol_id(module, local, fqn),
        namespace,
        primary_spelling(local),
        SymbolKind::Predicate,
        Visibility::Public,
        range(
            source_id,
            declaration_start,
            declaration_start + local.len(),
        ),
        visible_after_ordinal,
    )
}

fn current_private_projection(
    source_id: SourceId,
    module: ModuleId,
    namespace: NamespacePath,
    local: &str,
    fqn: &str,
    declaration_start: usize,
    visible_after_ordinal: usize,
) -> NameSymbolProjection {
    NameSymbolProjection::current_module(
        symbol_id(module, local, fqn),
        namespace,
        primary_spelling(local),
        SymbolKind::Predicate,
        Visibility::Private,
        range(
            source_id,
            declaration_start,
            declaration_start + local.len(),
        ),
        visible_after_ordinal,
    )
}

fn imported_projection(
    source_id: SourceId,
    module: ModuleId,
    namespace: NamespacePath,
    local: &str,
    fqn: &str,
    visibility: Visibility,
    declaration_start: usize,
) -> NameSymbolProjection {
    NameSymbolProjection::imported(
        symbol_id(module, local, fqn),
        namespace,
        primary_spelling(local),
        SymbolKind::Predicate,
        visibility,
        range(
            source_id,
            declaration_start,
            declaration_start + local.len(),
        ),
    )
}

fn primary_spelling(local: &str) -> String {
    local.split('/').next().unwrap_or(local).to_owned()
}

fn assert_resolved_symbol(resolution: &NameReferenceResolution, index: usize, expected_fqn: &str) {
    let entry = resolution.table().get(resolution.ids()[index]).unwrap();
    let NameResolution::Resolved(symbol) = entry.resolution() else {
        panic!("expected resolved symbol at index {index}");
    };
    assert_eq!(symbol.symbol().fqn().as_str(), expected_fqn);
}

fn assert_resolved_builtin(
    resolution: &NameReferenceResolution,
    index: usize,
    expected_builtin: &str,
) {
    let entry = resolution.table().get(resolution.ids()[index]).unwrap();
    let NameResolution::ResolvedBuiltin(builtin) = entry.resolution() else {
        panic!("expected resolved builtin at index {index}");
    };
    assert_eq!(builtin.builtin().as_str(), expected_builtin);
}

fn assert_unresolved(
    resolution: &NameReferenceResolution,
    index: usize,
    expected_lookup: NameLookupClass,
    expected_spelling: &str,
) {
    let entry = resolution.table().get(resolution.ids()[index]).unwrap();
    let NameResolution::Unresolved(unresolved) = entry.resolution() else {
        panic!("expected unresolved name at index {index}");
    };
    assert_eq!(unresolved.lookup(), expected_lookup);
    assert_eq!(unresolved.spelling(), expected_spelling);
}

fn assert_unresolved_entry(
    table: &NameRefTable,
    id: NameRefId,
    expected_lookup: NameLookupClass,
    expected_range: SourceRange,
) {
    let entry = table.get(id).unwrap();
    let NameResolution::Unresolved(unresolved) = entry.resolution() else {
        panic!("expected unresolved name");
    };
    assert_eq!(unresolved.lookup(), expected_lookup);
    assert_eq!(unresolved.range(), expected_range);
}

fn dot_chain_candidate(
    source_id: SourceId,
    module: ModuleId,
    ordinal: usize,
    start: usize,
    spellings: &[&str],
    scope: LocalTermScope,
) -> DotChainCandidate {
    let mut cursor = start;
    let mut segments = Vec::new();
    for spelling in spellings {
        segments.push(DotChainSegment::new(
            *spelling,
            range(source_id, cursor, cursor + spelling.len()),
        ));
        cursor += spelling.len() + 1;
    }
    let chain_range = range(source_id, start, cursor.saturating_sub(1));
    let origin = SemanticOrigin::new(
        source_id,
        module,
        SourceAnchor::Range(chain_range),
        vec![ordinal as u32],
    );
    let mut arena = ResolvedArenaBuilder::new();
    let base_node = arena
        .push(ResolvedNode::new(
            SurfaceNodeKind::TermReference,
            Vec::new(),
            origin.clone(),
        ))
        .unwrap();
    let chain_node = arena
        .push(ResolvedNode::new(
            SurfaceNodeKind::SelectorAccess,
            Vec::new(),
            origin.clone(),
        ))
        .unwrap();
    DotChainCandidate::new(
        segments,
        ReferenceSite::new(chain_node, chain_range, spellings.join(".")),
        origin,
        base_node,
        scope,
        ordinal,
    )
}

fn unresolved_namespace_fixture(
    source_id: SourceId,
    ordinal: usize,
    start: usize,
    spelling: &str,
    class: NamespaceFailureClass,
    matched_prefix: &[&str],
) -> UnresolvedNamespacePath {
    let candidate = candidate(source_id, ordinal, start, &[spelling]);
    let partial = NamespacePartialCandidate::new(
        NamespacePartialOrigin::ImportAlias,
        matched_prefix
            .iter()
            .map(|part| (*part).to_owned())
            .collect(),
        None,
        Vec::new(),
    );
    UnresolvedNamespacePath::from_candidate(
        &candidate,
        class,
        candidate.segments().first().cloned(),
        Some(partial),
        Vec::new(),
        Vec::new(),
    )
}

fn primary_root_ranges(report: &NameDiagnosticReport) -> Vec<(NameDiagnosticRootId, SourceRange)> {
    report
        .primary()
        .map(|diagnostic| (diagnostic.root(), diagnostic.root_range()))
        .collect()
}

fn insert_unresolved_name_entry(
    table: &mut NameRefTable,
    source_id: SourceId,
    module: ModuleId,
    ordinal: usize,
    start: usize,
    spelling: &str,
) {
    let candidate = name_candidate(source_id, module, ordinal, start, spelling);
    table.insert(NameRefEntry::new(
        candidate.site().clone(),
        unresolved_name(&candidate, NameLookupClass::Symbol),
        candidate.origin().clone(),
    ));
}

fn insert_ambiguous_name_entry(
    table: &mut NameRefTable,
    source_id: SourceId,
    module: ModuleId,
    ordinal: usize,
    start: usize,
    spelling: &str,
    candidates: &[(&str, &str, usize)],
) {
    let (site, origin) = reference_site(source_id, module.clone(), start, spelling, ordinal);
    let candidates = candidates
        .iter()
        .map(|(local, fqn, declaration_start)| {
            NameResolutionCandidate::new(
                symbol_id(module.clone(), local, fqn),
                range(
                    source_id,
                    *declaration_start,
                    *declaration_start + local.len(),
                ),
            )
        })
        .collect();
    table.insert(NameRefEntry::new(
        site.clone(),
        NameResolution::Ambiguous(AmbiguousNameRef::new(spelling, site.range(), candidates)),
        origin,
    ));
}

fn candidate(
    source_id: SourceId,
    ordinal: usize,
    start: usize,
    spellings: &[&str],
) -> NamespacePathCandidate {
    let mut cursor = start;
    let mut segments = Vec::new();
    for spelling in spellings {
        segments.push(NamespacePathSegment::new(
            *spelling,
            range(source_id, cursor, cursor + spelling.len()),
        ));
        cursor += spelling.len() + 1;
    }
    NamespacePathCandidate::new(
        segments,
        range(source_id, start, cursor.saturating_sub(1)),
        ordinal,
    )
}

fn fixture_provider() -> WorkspaceStubModuleIndexProvider {
    WorkspaceStubModuleIndexProvider::new(
        vec![
            package("dep"),
            package("app"),
            package("stdpkg"),
            package("pkgdep"),
            package("devdep"),
            package("extdep"),
            package("altdep"),
        ],
        vec![
            namespace(NamespaceRoot::PackageName, &["dep"], "dep"),
            namespace(NamespaceRoot::PackageName, &["dep", "nested"], "altdep"),
            namespace(NamespaceRoot::PackageName, &["app"], "app"),
            namespace(NamespaceRoot::Std, &[], "stdpkg"),
            namespace(NamespaceRoot::Pub, &["math"], "dep"),
            namespace(NamespaceRoot::Pub, &["math", "logic"], "altdep"),
            namespace(NamespaceRoot::Pkg, &["vendor"], "pkgdep"),
            namespace(NamespaceRoot::Dev, &["sandbox"], "devdep"),
            namespace(NamespaceRoot::Ext, &["mirror"], "extdep"),
        ],
        vec![
            workspace_module("app", "main"),
            workspace_module("app", "util"),
            dependency_module("dep", "logic"),
            dependency_module("dep", "algebra.group"),
            dependency_module("stdpkg", "core"),
            dependency_module("pkgdep", "lib"),
            dependency_module("devdep", "tools"),
            dependency_module("extdep", "logic"),
            dependency_module("altdep", "logic"),
            dependency_module("altdep", "core"),
        ],
        vec![
            dependency_summary("dep", "logic", 3),
            dependency_summary("dep", "algebra.group", 4),
            dependency_summary("stdpkg", "core", 5),
            dependency_summary("pkgdep", "lib", 6),
            dependency_summary("devdep", "tools", 7),
            dependency_summary("extdep", "logic", 8),
            dependency_summary("altdep", "logic", 9),
            dependency_summary("altdep", "core", 10),
        ],
    )
}

fn package(package_id: &str) -> PackageIndexEntry {
    PackageIndexEntry {
        package_id: PackageId::new(package_id),
        version: Version::new(0, 1, 0),
        edition: Edition::new("2026"),
        source: PackageIndexSource::Workspace {
            package_root: format!("/workspace/{package_id}"),
            source_root: format!("/workspace/{package_id}/src"),
            manifest_path: format!("/workspace/{package_id}/mizar.toml"),
        },
        dependencies: Vec::new(),
    }
}

fn namespace(root: NamespaceRoot, prefix: &[&str], package_id: &str) -> NamespaceIndexEntry {
    NamespaceIndexEntry {
        root,
        prefix: prefix
            .iter()
            .map(|component| (*component).to_owned())
            .collect(),
        package_id: PackageId::new(package_id),
    }
}

fn workspace_module(package_id: &str, path: &str) -> ModuleIndexEntry {
    ModuleIndexEntry {
        package_id: PackageId::new(package_id),
        module_path: ModulePath::new(path),
        module: IndexedModuleId::new(PackageId::new(package_id), ModulePath::new(path)),
        location: ModuleIndexLocation::WorkspaceFile {
            source_root: format!("/workspace/{package_id}/src"),
            normalized_path: format!("/workspace/{package_id}/src/{}.miz", path.replace('.', "/")),
            source_relative_path: format!("{}.miz", path.replace('.', "/")),
        },
        edition: Edition::new("2026"),
    }
}

fn dependency_module(package_id: &str, path: &str) -> ModuleIndexEntry {
    ModuleIndexEntry {
        package_id: PackageId::new(package_id),
        module_path: ModulePath::new(path),
        module: IndexedModuleId::new(PackageId::new(package_id), ModulePath::new(path)),
        location: ModuleIndexLocation::DependencySummary {
            artifact: format!("{package_id}.{path}.summary"),
            content_hash: Hash::from_bytes([1; Hash::BYTE_LEN]),
        },
        edition: Edition::new("2026"),
    }
}

fn dependency_summary(package_id: &str, path: &str, byte: u8) -> DependencyModuleSummaryRef {
    DependencyModuleSummaryRef {
        module: IndexedModuleId::new(PackageId::new(package_id), ModulePath::new(path)),
        artifact: format!("{package_id}.{path}.summary"),
        content_hash: Hash::from_bytes([byte; Hash::BYTE_LEN]),
    }
}

fn module_id(package_id: &str, path: &str) -> ModuleId {
    ModuleId::new(PackageId::new(package_id), ModulePath::new(path))
}

fn symbol_id(module: ModuleId, local: &str, fqn: &str) -> SymbolId {
    SymbolId::new(
        module,
        LocalSymbolId::new(local),
        FullyQualifiedName::new(fqn),
    )
}

fn task277r1_identity_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let definition = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "definition",
        range(source_id, 0, 10),
    );
    let parameter = task277r1_direct_type_parameter(&mut builder, source_id, 11, "T");
    let generator = task277r1_generator(&mut builder, source_id, 30, 48, "T");
    let functor = builder.add_node(
        SurfaceNodeKind::FunctorDefinition,
        range(source_id, 30, 60),
        vec![generator],
    );
    let definition_block = builder.add_node(
        SurfaceNodeKind::DefinitionBlockItem,
        range(source_id, 0, 62),
        vec![definition, parameter, functor],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 62),
        vec![definition_block],
    );
    builder.finish(Some(root), None)
}

fn task277r1_scoped_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let definition = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "definition",
        range(source_id, 0, 10),
    );
    let outer_parameter = task277r1_direct_type_parameter(&mut builder, source_id, 11, "T");
    let duplicate_outer_parameter =
        task277r1_direct_type_parameter(&mut builder, source_id, 26, "T");
    let unique_outer_parameter = task277r1_direct_type_parameter(&mut builder, source_id, 41, "V");
    let non_generator_parameter = task277r1_direct_type_parameter(&mut builder, source_id, 56, "N");
    let cross_owner_parameter = task277r1_direct_type_parameter(&mut builder, source_id, 71, "X");
    let non_generator_identifier =
        builder.add_token(SurfaceTokenKind::Identifier, "N", range(source_id, 87, 88));
    let non_generator_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, 87, 88),
        vec![non_generator_identifier],
    );
    let non_generator_type = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, 87, 88),
        vec![non_generator_head],
    );
    let ambiguous_generator = task277r1_generator(&mut builder, source_id, 90, 108, "T");
    let first_unique_generator = task277r1_generator(&mut builder, source_id, 120, 138, "V");

    let inner_definition = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "definition",
        range(source_id, 150, 160),
    );
    let inner_parameter = task277r1_direct_type_parameter(&mut builder, source_id, 161, "U");
    let inner_generator = task277r1_generator(&mut builder, source_id, 180, 198, "U");
    let cross_owner_generator = task277r1_generator(&mut builder, source_id, 205, 223, "X");
    let inner_functor = builder.add_node(
        SurfaceNodeKind::FunctorDefinition,
        range(source_id, 180, 235),
        vec![inner_generator, cross_owner_generator],
    );
    let nested_definition = builder.add_node(
        SurfaceNodeKind::DefinitionBlockItem,
        range(source_id, 150, 240),
        vec![inner_definition, inner_parameter, inner_functor],
    );
    let second_unique_generator = task277r1_generator(&mut builder, source_id, 250, 268, "V");
    let outer_functor = builder.add_node(
        SurfaceNodeKind::FunctorDefinition,
        range(source_id, 86, 280),
        vec![
            non_generator_type,
            ambiguous_generator,
            first_unique_generator,
            nested_definition,
            second_unique_generator,
        ],
    );
    let outer_definition = builder.add_node(
        SurfaceNodeKind::DefinitionBlockItem,
        range(source_id, 0, 290),
        vec![
            definition,
            outer_parameter,
            duplicate_outer_parameter,
            unique_outer_parameter,
            non_generator_parameter,
            cross_owner_parameter,
            outer_functor,
        ],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 290),
        vec![outer_definition],
    );
    builder.finish(Some(root), None)
}

fn task277r1_rejected_shapes_ast(source_id: SourceId) -> SurfaceAst {
    let mut builder = SurfaceAstBuilder::new(source_id);
    let definition = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "definition",
        range(source_id, 0, 10),
    );

    let bound_let = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, 11, 14),
    );
    let bound_name = builder.add_token(SurfaceTokenKind::Identifier, "T", range(source_id, 15, 16));
    let bound_be = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "be",
        range(source_id, 17, 19),
    );
    let bound_type = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "type",
        range(source_id, 20, 24),
    );
    let extends = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "extends",
        range(source_id, 25, 32),
    );
    let bound = builder.add_token(SurfaceTokenKind::Identifier, "M", range(source_id, 33, 34));
    let bound_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 34, 35),
    );
    let bounded_parameter = builder.add_node(
        SurfaceNodeKind::TemplateParameter,
        range(source_id, 11, 35),
        vec![
            bound_let,
            bound_name,
            bound_be,
            bound_type,
            extends,
            bound,
            bound_semicolon,
        ],
    );

    let multi_let = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, 36, 39),
    );
    let first_binder =
        builder.add_token(SurfaceTokenKind::Identifier, "T", range(source_id, 40, 41));
    let comma = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ",",
        range(source_id, 41, 42),
    );
    let second_binder =
        builder.add_token(SurfaceTokenKind::Identifier, "U", range(source_id, 43, 44));
    let multi_be = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "be",
        range(source_id, 45, 47),
    );
    let multi_type = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "type",
        range(source_id, 48, 52),
    );
    let multi_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 52, 53),
    );
    let multiple_binders = builder.add_node(
        SurfaceNodeKind::TemplateParameter,
        range(source_id, 36, 53),
        vec![
            multi_let,
            first_binder,
            comma,
            second_binder,
            multi_be,
            multi_type,
            multi_semicolon,
        ],
    );

    let pred_let = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, 54, 57),
    );
    let pred_name = builder.add_token(SurfaceTokenKind::Identifier, "P", range(source_id, 58, 59));
    let pred_be = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "be",
        range(source_id, 60, 62),
    );
    let pred = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "pred",
        range(source_id, 63, 67),
    );
    let pred_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 67, 68),
    );
    let predicate_parameter = builder.add_node(
        SurfaceNodeKind::TemplateParameter,
        range(source_id, 54, 68),
        vec![pred_let, pred_name, pred_be, pred, pred_semicolon],
    );

    let recovery_let = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, 69, 72),
    );
    let recovery_name =
        builder.add_recovered_token(SurfaceTokenKind::Identifier, "R", range(source_id, 73, 74));
    let recovery_be = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "be",
        range(source_id, 75, 77),
    );
    let recovery_type = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "type",
        range(source_id, 78, 82),
    );
    let recovery_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 82, 83),
    );
    let recovered_parameter = builder.add_node(
        SurfaceNodeKind::TemplateParameter,
        range(source_id, 69, 83),
        vec![
            recovery_let,
            recovery_name,
            recovery_be,
            recovery_type,
            recovery_semicolon,
        ],
    );

    let nested_parameter = task277r1_direct_type_parameter(&mut builder, source_id, 84, "T");
    let wrapper = builder.add_node(
        SurfaceNodeKind::DefinitionParameter,
        range(source_id, 84, 98),
        vec![nested_parameter],
    );
    let ordinary_parameter = builder.add_node(
        SurfaceNodeKind::DefinitionParameter,
        range(source_id, 99, 100),
        Vec::new(),
    );
    let generator = task277r1_generator(&mut builder, source_id, 110, 128, "T");
    let functor = builder.add_node(
        SurfaceNodeKind::FunctorDefinition,
        range(source_id, 110, 145),
        vec![generator],
    );

    let constraint_let = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, 150, 153),
    );
    let constraint_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "C",
        range(source_id, 154, 155),
    );
    let constraint_be = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "be",
        range(source_id, 156, 158),
    );
    let constraint_type = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "type",
        range(source_id, 159, 163),
    );
    let such = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "such",
        range(source_id, 164, 168),
    );
    let condition = builder.add_token(
        SurfaceTokenKind::Identifier,
        "Q",
        range(source_id, 169, 170),
    );
    let constraint_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 170, 171),
    );
    let constrained_parameter = builder.add_node(
        SurfaceNodeKind::TemplateParameter,
        range(source_id, 150, 171),
        vec![
            constraint_let,
            constraint_name,
            constraint_be,
            constraint_type,
            such,
            condition,
            constraint_semicolon,
        ],
    );

    let func_let = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, 172, 175),
    );
    let func_name = builder.add_token(
        SurfaceTokenKind::Identifier,
        "F",
        range(source_id, 176, 177),
    );
    let func_be = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "be",
        range(source_id, 178, 180),
    );
    let func = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "func",
        range(source_id, 181, 185),
    );
    let func_semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, 185, 186),
    );
    let functor_parameter = builder.add_node(
        SurfaceNodeKind::TemplateParameter,
        range(source_id, 172, 186),
        vec![func_let, func_name, func_be, func, func_semicolon],
    );
    let valid_parameter = task277r1_direct_type_parameter(&mut builder, source_id, 190, "Z");
    let recovered_sibling_generator = task277r1_generator_with_shape(
        &mut builder,
        source_id,
        210,
        228,
        "Z",
        Task277r1GeneratorShape::RecoveredSibling,
    );
    let malformed_generator = task277r1_generator_with_shape(
        &mut builder,
        source_id,
        240,
        258,
        "Z",
        Task277r1GeneratorShape::MalformedTypeHead,
    );
    let rejected_generators = builder.add_node(
        SurfaceNodeKind::FunctorDefinition,
        range(source_id, 210, 275),
        vec![recovered_sibling_generator, malformed_generator],
    );
    let definition_block = builder.add_node(
        SurfaceNodeKind::DefinitionBlockItem,
        range(source_id, 0, 280),
        vec![
            definition,
            bounded_parameter,
            multiple_binders,
            predicate_parameter,
            recovered_parameter,
            wrapper,
            ordinary_parameter,
            functor,
            constrained_parameter,
            functor_parameter,
            valid_parameter,
            rejected_generators,
        ],
    );
    let root = builder.add_node(
        SurfaceNodeKind::Root,
        range(source_id, 0, 280),
        vec![definition_block],
    );
    builder.finish(Some(root), None)
}

fn task277r1_direct_type_parameter(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    spelling: &str,
) -> mizar_syntax::SurfaceBuilderNodeId {
    let let_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "let",
        range(source_id, start, start + 3),
    );
    let binder_start = start + 4;
    let binder = builder.add_token(
        SurfaceTokenKind::Identifier,
        spelling,
        range(source_id, binder_start, binder_start + spelling.len()),
    );
    let be_start = binder_start + spelling.len() + 1;
    let be_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "be",
        range(source_id, be_start, be_start + 2),
    );
    let type_start = be_start + 3;
    let type_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "type",
        range(source_id, type_start, type_start + 4),
    );
    let semicolon = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        ";",
        range(source_id, type_start + 4, type_start + 5),
    );
    builder.add_node(
        SurfaceNodeKind::TemplateParameter,
        range(source_id, start, type_start + 5),
        vec![let_keyword, binder, be_keyword, type_keyword, semicolon],
    )
}

fn task277r1_generator(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    type_start: usize,
    spelling: &str,
) -> mizar_syntax::SurfaceBuilderNodeId {
    task277r1_generator_with_shape(
        builder,
        source_id,
        start,
        type_start,
        spelling,
        Task277r1GeneratorShape::Exact,
    )
}

#[derive(Clone, Copy)]
enum Task277r1GeneratorShape {
    Exact,
    RecoveredSibling,
    MalformedTypeHead,
}

fn task277r1_generator_with_shape(
    builder: &mut SurfaceAstBuilder,
    source_id: SourceId,
    start: usize,
    type_start: usize,
    spelling: &str,
    shape: Task277r1GeneratorShape,
) -> mizar_syntax::SurfaceBuilderNodeId {
    let opener = if matches!(shape, Task277r1GeneratorShape::RecoveredSibling) {
        builder.add_recovered_token(
            SurfaceTokenKind::ReservedSymbol,
            "{",
            range(source_id, start, start + 1),
        )
    } else {
        builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "{",
            range(source_id, start, start + 1),
        )
    };
    let mapper = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, start + 2, start + 3),
    );
    let where_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "where",
        range(source_id, start + 4, start + 9),
    );
    let generator = builder.add_token(
        SurfaceTokenKind::Identifier,
        "x",
        range(source_id, type_start - 8, type_start - 7),
    );
    let is_keyword = builder.add_token(
        SurfaceTokenKind::ReservedWord,
        "is",
        range(source_id, type_start - 6, type_start - 4),
    );
    let identifier = builder.add_token(
        SurfaceTokenKind::Identifier,
        spelling,
        range(source_id, type_start, type_start + spelling.len()),
    );
    let mut type_head_children = vec![identifier];
    if matches!(shape, Task277r1GeneratorShape::MalformedTypeHead) {
        type_head_children.push(builder.add_token(
            SurfaceTokenKind::ReservedSymbol,
            "[",
            range(
                source_id,
                type_start + spelling.len(),
                type_start + spelling.len() + 1,
            ),
        ));
    }
    let type_head = builder.add_node(
        SurfaceNodeKind::TypeHead,
        range(source_id, type_start, type_start + spelling.len()),
        type_head_children,
    );
    let type_expression = builder.add_node(
        SurfaceNodeKind::TypeExpression,
        range(source_id, type_start, type_start + spelling.len()),
        vec![type_head],
    );
    let segment = builder.add_node(
        SurfaceNodeKind::ComprehensionVariableSegment,
        range(source_id, type_start - 8, type_start + spelling.len()),
        vec![generator, is_keyword, type_expression],
    );
    let closer = builder.add_token(
        SurfaceTokenKind::ReservedSymbol,
        "}",
        range(
            source_id,
            type_start + spelling.len() + 1,
            type_start + spelling.len() + 2,
        ),
    );
    builder.add_node(
        SurfaceNodeKind::SetComprehension,
        range(source_id, start, type_start + spelling.len() + 2),
        vec![opener, mapper, where_keyword, segment, closer],
    )
}

fn resolved_node(
    ast: &SurfaceAst,
    resolved: &SurfaceResolvedArena,
    start: usize,
    end: usize,
    matches_kind: impl Fn(&SurfaceNodeKind) -> bool,
) -> ResolvedNodeId {
    let source = ast
        .node_views()
        .find(|node| node.range() == range(ast.source_id, start, end) && matches_kind(node.kind()))
        .map(|node| node.id())
        .unwrap();
    resolved.resolved_node_for(source).unwrap()
}

fn source_id() -> SourceId {
    let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "05".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot_id)
        .unwrap()
}

const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}
