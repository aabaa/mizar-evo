use super::*;
use crate::module_index::{
    DependencyModuleSummaryRef, ModuleIndexLocation, NamespaceIndexEntry, PackageIndexEntry,
    WorkspaceStubModuleIndexProvider,
};
use mizar_build::module_index::PackageIndexSource;
use mizar_session::{
    BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
    SessionIdAllocator, SourceId,
};
use semver::Version;

#[test]
fn acyclic_fixture_builds_expected_graph_and_dependency_first_order() {
    let provider = fixture_provider();
    let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
        .build(&[
            module_candidates(
                module_id("app", "main"),
                vec![
                    candidate("dep", "logic", 20, 24, 2),
                    candidate("app", "util", 10, 14, 0),
                    candidate("dep", "logic", 30, 34, 1),
                    candidate("dep", "logic", 25, 29, 1),
                ],
            ),
            module_candidates(
                module_id("app", "util"),
                vec![candidate("dep", "logic", 40, 44, 0)],
            ),
            module_candidates(module_id("dep", "logic"), Vec::new()),
        ])
        .unwrap();

    assert!(!graph.has_cycles());
    assert_eq!(
        module_paths(graph.graph().nodes()),
        vec!["app:main", "app:util", "dep:logic"]
    );
    assert_eq!(
        edge_paths(graph.graph().edges()),
        vec![
            ("app:main", "app:util", 0, (10, 14)),
            ("app:main", "dep:logic", 1, (25, 29)),
            ("app:util", "dep:logic", 0, (40, 44)),
        ]
    );
    assert_eq!(
        module_paths(graph.graph().topological_order()),
        vec!["dep:logic", "app:util", "app:main"]
    );
}

#[test]
fn independent_acyclic_components_use_canonical_ready_ties() {
    let provider = fixture_provider();
    let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
        .build(&[
            module_candidates(
                module_id("app", "main"),
                vec![candidate("app", "util", 10, 14, 0)],
            ),
            module_candidates(
                module_id("app", "alpha"),
                vec![candidate("dep", "logic", 20, 24, 0)],
            ),
            module_candidates(module_id("app", "util"), Vec::new()),
            module_candidates(module_id("app", "beta"), Vec::new()),
            module_candidates(module_id("dep", "logic"), Vec::new()),
        ])
        .unwrap();

    assert_eq!(
        module_paths(graph.graph().topological_order()),
        vec!["app:beta", "app:util", "app:main", "dep:logic", "app:alpha",]
    );
}

#[test]
fn cycle_fixture_is_rejected_deterministically() {
    let provider = fixture_provider();
    let first = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
        .build(&cycle_fixture_one())
        .unwrap();
    let second = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
        .build(&cycle_fixture_two())
        .unwrap();

    assert_eq!(first, second);
    assert!(first.has_cycles());
    assert_eq!(first.cycles().len(), 1);
    assert_eq!(
        module_paths(first.cycles()[0].modules()),
        vec!["app:main", "app:util"]
    );
    assert_eq!(
        edge_paths(first.cycles()[0].edges()),
        vec![
            ("app:main", "app:util", 0, (10, 14)),
            ("app:util", "app:main", 0, (20, 24)),
        ]
    );
    assert_eq!(
        module_paths(first.graph().topological_order()),
        vec!["dep:logic", "app:facade"]
    );
    assert_eq!(
        edge_paths(first.graph().edges()),
        vec![("app:facade", "dep:logic", 0, (30, 34))]
    );
}

#[test]
fn self_cycle_is_rejected_deterministically() {
    let provider = fixture_provider();
    let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
        .build(&[module_candidates(
            module_id("app", "main"),
            vec![candidate("app", "main", 50, 54, 0)],
        )])
        .unwrap();

    assert!(graph.has_cycles());
    assert_eq!(module_paths(graph.cycles()[0].modules()), vec!["app:main"]);
    assert!(graph.graph().nodes().is_empty());
    assert!(graph.graph().edges().is_empty());
    assert!(graph.graph().topological_order().is_empty());
}

#[test]
fn independent_cycles_sort_by_source_provenance() {
    let provider = fixture_provider();
    let graph = ImportGraphBuilder::new(ModuleIndexInput::new(&provider))
        .build(&[
            module_candidates(
                module_id("app", "alpha"),
                vec![candidate("app", "beta", 100, 104, 0)],
            ),
            module_candidates(
                module_id("app", "beta"),
                vec![candidate("app", "alpha", 110, 114, 0)],
            ),
            module_candidates(
                module_id("app", "yankee"),
                vec![candidate("app", "zulu", 10, 14, 0)],
            ),
            module_candidates(
                module_id("app", "zulu"),
                vec![candidate("app", "yankee", 20, 24, 0)],
            ),
        ])
        .unwrap();

    assert_eq!(graph.cycles().len(), 2);
    assert_eq!(
        module_paths(graph.cycles()[0].modules()),
        vec!["app:yankee", "app:zulu"]
    );
    assert_eq!(
        module_paths(graph.cycles()[1].modules()),
        vec!["app:alpha", "app:beta"]
    );
}

#[test]
fn unknown_modules_are_rejected_before_graph_publication() {
    let provider = fixture_provider();
    let builder = ImportGraphBuilder::new(ModuleIndexInput::new(&provider));

    let unknown_source = builder
        .build(&[module_candidates(module_id("missing", "main"), Vec::new())])
        .unwrap_err();
    match unknown_source {
        ImportGraphBuildError::UnknownSourceModule { module, lookup } => {
            assert_eq!(module_key(&module), "missing:main");
            match *lookup {
                ModuleIndexProviderError::UnknownModule { module } => {
                    assert_eq!(module.package.as_str(), "missing");
                    assert_eq!(module.path.as_str(), "main");
                }
                other => panic!("expected unknown source module lookup, got {other:?}"),
            }
        }
        other => panic!("expected unknown source module, got {other:?}"),
    }

    let unknown_target = builder
        .build(&[module_candidates(
            module_id("app", "main"),
            vec![candidate("missing", "dep", 60, 64, 0)],
        )])
        .unwrap_err();
    match unknown_target {
        ImportGraphBuildError::UnknownTargetModule {
            source,
            target,
            range: actual_range,
            ordinal,
            lookup,
        } => {
            assert_eq!(module_key(&source), "app:main");
            assert_eq!(module_key(&target), "missing:dep");
            assert_eq!(actual_range, range(60, 64));
            assert_eq!(ordinal, 0);
            match *lookup {
                ModuleIndexProviderError::UnknownModule { module } => {
                    assert_eq!(module.package.as_str(), "missing");
                    assert_eq!(module.path.as_str(), "dep");
                }
                other => panic!("expected unknown target module lookup, got {other:?}"),
            }
        }
        other => panic!("expected unknown target module, got {other:?}"),
    }
}

#[test]
fn aliases_do_not_change_canonical_targets_or_graph_candidates() {
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let resolution = ImportPathResolver::new(input).resolve(
        &module_id("app", "main"),
        &[
            path_candidate(
                &["dep", "logic"],
                ImportPathPrefix::Unprefixed,
                None,
                10,
                14,
                0,
            ),
            path_candidate(
                &["dep", "logic"],
                ImportPathPrefix::Unprefixed,
                Some("LogicAlias"),
                20,
                24,
                1,
            )
            .with_alias_range(range(25, 35)),
        ],
    );

    assert!(resolution.unresolved().is_empty());
    assert_eq!(
        resolved_imports(&resolution),
        vec![
            ("dep:logic".to_owned(), "logic".to_owned(), None),
            (
                "dep:logic".to_owned(),
                "LogicAlias".to_owned(),
                Some("LogicAlias".to_owned())
            ),
        ]
    );

    let graph_candidates = resolution.module_candidates(module_id("app", "main"));
    assert_eq!(
        graph_candidates
            .imports()
            .iter()
            .map(|candidate| module_key(candidate.target()).to_owned())
            .collect::<Vec<_>>(),
        vec!["dep:logic", "dep:logic"]
    );
}

#[test]
fn relative_prefixes_use_dot_separated_module_directories() {
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let nested = ImportPathResolver::new(input).resolve(
        &module_id("app", "dir.main"),
        &[
            path_candidate(&["sibling"], ImportPathPrefix::Current, None, 10, 14, 0),
            path_candidate(&["common"], ImportPathPrefix::Parent, None, 20, 24, 1),
            path_candidate(&["missing"], ImportPathPrefix::Current, None, 25, 29, 2),
        ],
    );

    assert_eq!(
        resolved_targets(&nested),
        vec!["app:dir.sibling", "app:common"]
    );
    assert_eq!(
        unresolved_classes(&nested),
        vec![ImportPathFailureClass::UnknownModule]
    );
    assert_eq!(
        nested.unresolved()[0]
            .partial()
            .and_then(ImportPathPartialCandidate::package)
            .map(PackageId::as_str),
        Some("app")
    );
    assert_eq!(
        nested.unresolved()[0]
            .partial()
            .map(ImportPathPartialCandidate::remaining_components),
        Some(&["dir".to_owned(), "missing".to_owned()][..])
    );

    let recovered = ImportPathResolver::new(input).resolve(
        &module_id("app", "main"),
        &[
            path_candidate(&["util"], ImportPathPrefix::Parent, None, 30, 34, 0),
            path_candidate(&["util"], ImportPathPrefix::Unprefixed, None, 40, 44, 1),
        ],
    );

    assert_eq!(resolved_targets(&recovered), vec!["app:util"]);
    assert_eq!(
        unresolved_classes(&recovered),
        vec![ImportPathFailureClass::RelativePathEscapesPackage]
    );
}

#[test]
fn namespace_bindings_win_over_package_local_fallback() {
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let resolution = ImportPathResolver::new(input).resolve(
        &module_id("app", "main"),
        &[
            path_candidate(
                &["dep", "logic"],
                ImportPathPrefix::Unprefixed,
                None,
                10,
                14,
                0,
            ),
            path_candidate(&["util"], ImportPathPrefix::Unprefixed, None, 20, 24, 1),
            path_candidate(
                &["pub", "math", "logic"],
                ImportPathPrefix::Unprefixed,
                None,
                30,
                34,
                2,
            ),
            path_candidate(
                &["dep", "missing"],
                ImportPathPrefix::Unprefixed,
                None,
                40,
                44,
                3,
            ),
            path_candidate(
                &["std", "missing"],
                ImportPathPrefix::Unprefixed,
                None,
                50,
                54,
                4,
            ),
        ],
    );

    assert_eq!(
        resolved_targets(&resolution),
        vec!["dep:logic", "app:util", "dep:logic"]
    );
    assert_eq!(
        unresolved_classes(&resolution),
        vec![
            ImportPathFailureClass::UnknownModule,
            ImportPathFailureClass::UnknownNamespaceOrPackage,
        ]
    );
    assert_eq!(
        resolution.unresolved()[0]
            .partial()
            .and_then(ImportPathPartialCandidate::package)
            .map(PackageId::as_str),
        Some("dep")
    );
    assert_eq!(
        resolution.unresolved()[1]
            .partial()
            .and_then(ImportPathPartialCandidate::namespace_root),
        Some(NamespaceRoot::Std)
    );
}

#[test]
fn unresolved_imports_do_not_abort_later_candidates() {
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let resolution = ImportPathResolver::new(input).resolve(
        &module_id("app", "main"),
        &[
            path_candidate(&["missing"], ImportPathPrefix::Unprefixed, None, 10, 14, 0),
            path_candidate(&["util"], ImportPathPrefix::Unprefixed, None, 20, 24, 1),
            path_candidate(&["logic"], ImportPathPrefix::Unprefixed, None, 30, 34, 2)
                .with_recovered(),
        ],
    );

    assert_eq!(resolved_targets(&resolution), vec!["app:util"]);
    assert_eq!(
        unresolved_classes(&resolution),
        vec![
            ImportPathFailureClass::UnknownModule,
            ImportPathFailureClass::RecoveredSyntax,
        ]
    );
    assert!(resolution.unresolved()[1].recovered());
    assert_eq!(
        resolution.unresolved()[1].components(),
        &["logic".to_owned()]
    );
}

#[test]
fn duplicate_aliases_and_reserved_aliases_are_unresolved_deterministically() {
    let provider = fixture_provider();
    let input = ModuleIndexInput::new(&provider);
    let resolution = ImportPathResolver::new(input).resolve(
        &module_id("app", "main"),
        &[
            path_candidate(
                &["dep", "logic"],
                ImportPathPrefix::Unprefixed,
                Some("Shared"),
                10,
                14,
                0,
            )
            .with_alias_range(range(15, 21)),
            path_candidate(
                &["util"],
                ImportPathPrefix::Unprefixed,
                Some("Shared"),
                20,
                24,
                1,
            )
            .with_alias_range(range(25, 31))
            .with_branch_provenance(range(22, 24), range(25, 31)),
            path_candidate(
                &["dep", "logic"],
                ImportPathPrefix::Unprefixed,
                Some("Logic"),
                30,
                34,
                2,
            )
            .with_branch_provenance(range(32, 33), range(33, 34)),
            path_candidate(
                &["dep", "logic"],
                ImportPathPrefix::Unprefixed,
                Some("Logic"),
                40,
                44,
                3,
            ),
            path_candidate(
                &["dep", "logic"],
                ImportPathPrefix::Unprefixed,
                Some("std"),
                50,
                54,
                4,
            )
            .with_alias_range(range(55, 58)),
        ],
    );

    assert_eq!(
        resolved_imports(&resolution),
        vec![
            (
                "dep:logic".to_owned(),
                "Logic".to_owned(),
                Some("Logic".to_owned())
            ),
            (
                "dep:logic".to_owned(),
                "Logic".to_owned(),
                Some("Logic".to_owned())
            ),
        ]
    );
    assert_eq!(
        resolution
            .resolved()
            .iter()
            .map(|candidate| {
                (
                    candidate.branch_base_range().map(range_key),
                    candidate.branch_member_range().map(range_key),
                )
            })
            .collect::<Vec<_>>(),
        vec![(Some((32, 33)), Some((33, 34))), (None, None)]
    );
    assert_eq!(
        unresolved_classes(&resolution),
        vec![
            ImportPathFailureClass::DuplicateAlias,
            ImportPathFailureClass::DuplicateAlias,
            ImportPathFailureClass::AliasRootConflict,
        ]
    );
    assert_eq!(
        resolution
            .unresolved()
            .iter()
            .map(|candidate| (
                candidate.alias().map(str::to_owned),
                candidate.alias_range().map(range_key),
                candidate.candidate_target().map(module_key),
            ))
            .collect::<Vec<_>>(),
        vec![
            (Some("Shared".to_owned()), Some((15, 21)), Some("dep:logic")),
            (Some("Shared".to_owned()), Some((25, 31)), Some("app:util")),
            (Some("std".to_owned()), Some((55, 58)), None),
        ]
    );
    assert_eq!(
        resolution.unresolved()[1]
            .branch_base_range()
            .map(range_key),
        Some((22, 24))
    );
    assert_eq!(
        resolution.unresolved()[1]
            .branch_member_range()
            .map(range_key),
        Some((25, 31))
    );
}

fn cycle_fixture_one() -> Vec<ModuleImportCandidates> {
    vec![
        module_candidates(
            module_id("app", "main"),
            vec![candidate("app", "util", 10, 14, 0)],
        ),
        module_candidates(
            module_id("app", "util"),
            vec![candidate("app", "main", 20, 24, 0)],
        ),
        module_candidates(
            module_id("app", "facade"),
            vec![candidate("dep", "logic", 30, 34, 0)],
        ),
        module_candidates(module_id("dep", "logic"), Vec::new()),
    ]
}

fn cycle_fixture_two() -> Vec<ModuleImportCandidates> {
    vec![
        module_candidates(module_id("dep", "logic"), Vec::new()),
        module_candidates(
            module_id("app", "facade"),
            vec![candidate("dep", "logic", 30, 34, 0)],
        ),
        module_candidates(
            module_id("app", "util"),
            vec![candidate("app", "main", 20, 24, 0)],
        ),
        module_candidates(
            module_id("app", "main"),
            vec![candidate("app", "util", 10, 14, 0)],
        ),
    ]
}

fn module_candidates(
    module: ModuleId,
    imports: Vec<ImportEdgeCandidate>,
) -> ModuleImportCandidates {
    ModuleImportCandidates::new(module, imports)
}

fn candidate(
    package: &str,
    path: &str,
    start: usize,
    end: usize,
    ordinal: usize,
) -> ImportEdgeCandidate {
    ImportEdgeCandidate::new(module_id(package, path), range(start, end), ordinal)
}

fn path_candidate(
    components: &[&str],
    prefix: ImportPathPrefix,
    alias: Option<&str>,
    start: usize,
    end: usize,
    ordinal: usize,
) -> ImportPathCandidate {
    ImportPathCandidate::new(
        components
            .iter()
            .map(|component| (*component).to_owned())
            .collect(),
        prefix,
        alias.map(str::to_owned),
        range(start, end),
        ordinal,
    )
}

fn fixture_provider() -> WorkspaceStubModuleIndexProvider {
    WorkspaceStubModuleIndexProvider::new(
        vec![package("app"), package("dep")],
        vec![
            namespace(NamespaceRoot::PackageName, &["app"], "app"),
            namespace(NamespaceRoot::PackageName, &["dep"], "dep"),
            namespace(NamespaceRoot::Pub, &["math"], "dep"),
        ],
        vec![
            workspace_module("app", "main"),
            workspace_module("app", "util"),
            workspace_module("app", "common"),
            workspace_module("app", "dir.main"),
            workspace_module("app", "dir.sibling"),
            workspace_module("app", "dep.logic"),
            workspace_module("app", "dep.missing"),
            workspace_module("app", "std.missing"),
            workspace_module("app", "facade"),
            workspace_module("app", "alpha"),
            workspace_module("app", "beta"),
            workspace_module("app", "yankee"),
            workspace_module("app", "zulu"),
            dependency_module("dep", "logic"),
            dependency_module("dep", "algebra.group"),
        ],
        vec![DependencyModuleSummaryRef {
            module: indexed_module("dep", "logic"),
            artifact: "dep.logic.summary".to_owned(),
            content_hash: Hash::from_bytes([9; Hash::BYTE_LEN]),
        }],
    )
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

fn workspace_module(package_id: &str, path: &str) -> crate::module_index::ModuleIndexEntry {
    crate::module_index::ModuleIndexEntry {
        module: indexed_module(package_id, path),
        package_id: PackageId::new(package_id),
        module_path: ModulePath::new(path),
        location: ModuleIndexLocation::WorkspaceFile {
            source_root: format!("/workspace/{package_id}/src"),
            normalized_path: format!("/workspace/{package_id}/src/{path}.miz"),
            source_relative_path: format!("{path}.miz"),
        },
        edition: Edition::new("2026"),
    }
}

fn dependency_module(package_id: &str, path: &str) -> crate::module_index::ModuleIndexEntry {
    crate::module_index::ModuleIndexEntry {
        module: indexed_module(package_id, path),
        package_id: PackageId::new(package_id),
        module_path: ModulePath::new(path),
        location: ModuleIndexLocation::DependencySummary {
            artifact: format!("{package_id}.{path}.summary"),
            content_hash: Hash::from_bytes([9; Hash::BYTE_LEN]),
        },
        edition: Edition::new("2026"),
    }
}

fn indexed_module(package_id: &str, path: &str) -> IndexedModuleId {
    IndexedModuleId::new(PackageId::new(package_id), ModulePath::new(path))
}

fn module_id(package: &str, path: &str) -> ModuleId {
    ModuleId::new(PackageId::new(package), ModulePath::new(path))
}

fn module_paths(modules: &[ModuleId]) -> Vec<String> {
    modules
        .iter()
        .map(|module| format!("{}:{}", module.package().as_str(), module.path().as_str()))
        .collect()
}

fn edge_paths(edges: &[ImportGraphEdge]) -> Vec<(&str, &str, usize, (usize, usize))> {
    edges
        .iter()
        .map(|edge| {
            (
                module_key(edge.source()),
                module_key(edge.target()),
                edge.ordinal(),
                range_key(edge.range()),
            )
        })
        .collect()
}

fn module_key(module: &ModuleId) -> &str {
    match (module.package().as_str(), module.path().as_str()) {
        ("app", "main") => "app:main",
        ("app", "util") => "app:util",
        ("app", "common") => "app:common",
        ("app", "dir.main") => "app:dir.main",
        ("app", "dir.sibling") => "app:dir.sibling",
        ("app", "dep.logic") => "app:dep.logic",
        ("app", "dep.missing") => "app:dep.missing",
        ("app", "std.missing") => "app:std.missing",
        ("app", "facade") => "app:facade",
        ("app", "alpha") => "app:alpha",
        ("app", "beta") => "app:beta",
        ("app", "yankee") => "app:yankee",
        ("app", "zulu") => "app:zulu",
        ("dep", "logic") => "dep:logic",
        ("dep", "algebra.group") => "dep:algebra.group",
        ("missing", "dep") => "missing:dep",
        ("missing", "main") => "missing:main",
        _ => "unknown",
    }
}

fn resolved_targets(resolution: &ImportPathResolution) -> Vec<&str> {
    resolution
        .resolved()
        .iter()
        .map(|candidate| module_key(candidate.target()))
        .collect()
}

fn resolved_imports(resolution: &ImportPathResolution) -> Vec<(String, String, Option<String>)> {
    resolution
        .resolved()
        .iter()
        .map(|candidate| {
            (
                module_key(candidate.target()).to_owned(),
                candidate.alias().to_owned(),
                candidate.explicit_alias().map(str::to_owned),
            )
        })
        .collect()
}

fn unresolved_classes(resolution: &ImportPathResolution) -> Vec<ImportPathFailureClass> {
    resolution
        .unresolved()
        .iter()
        .map(UnresolvedImportCandidate::class)
        .collect()
}

fn source_id() -> SourceId {
    let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        "03".repeat(Hash::BYTE_LEN)
    ))
    .unwrap();
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot_id)
        .unwrap()
}

fn range(start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id: source_id(),
        start,
        end,
    }
}
