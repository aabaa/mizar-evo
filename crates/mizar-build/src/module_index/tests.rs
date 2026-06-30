use super::{
    ArtifactNamespaceBinding, DependencyArtifactIndex, DependencyModuleSummaryRef, ModuleId,
    ModuleIndexDiagnosticKind, ModuleIndexLocation, ModuleIndexProvider, ModuleIndexProviderError,
    NamespaceRoot, StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage,
    build_module_index,
};
use crate::planner::{
    BuildConfig, BuildPlan, DependencyGraph, Lockfile, PackagePlan, PackagePlanSource,
    VerifierConfig, WorkspaceBuildConfig, WorkspaceVerifierConfig,
};
use mizar_session::{Edition, Hash, ModulePath, PackageId, ToolchainInfo, WorkspaceRoot};
use semver::Version;

#[test]
fn module_index_builds_multi_package_workspace_modules() {
    let plan = build_plan(vec![
        workspace_package("algebra", "1.0.0"),
        workspace_package("topology", "1.0.0"),
    ]);
    let layout = StaticSourceLayout::new(vec![
        source_package(
            "topology",
            vec![source_file("src/spaces/metric.miz", "spaces/metric.miz")],
        ),
        source_package(
            "algebra",
            vec![
                source_file("src/lib.miz", "lib.miz"),
                source_file("src/groups/basic.miz", "groups/basic.miz"),
            ],
        ),
    ]);

    let index = build_module_index(&plan, &layout, &[]).expect("valid module index");

    assert_eq!(
        index
            .modules
            .iter()
            .map(|module| (module.package_id.as_str(), module.module_path.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("algebra", "groups.basic"),
            ("algebra", "lib"),
            ("topology", "spaces.metric"),
        ]
    );
    assert_eq!(
        index
            .package_for_namespace(&NamespaceRoot::PackageName, &["algebra".to_owned()])
            .expect("package namespace")
            .package_id
            .as_str(),
        "algebra"
    );
    let algebra_modules = index
        .modules_for_package(&PackageId::new("algebra"))
        .expect("known package modules");
    assert_eq!(algebra_modules.len(), 2);
}

#[test]
fn module_identity_is_package_scoped_and_alias_free() {
    let plan = build_plan(vec![
        workspace_package("algebra", "1.0.0"),
        workspace_package("topology", "1.0.0"),
    ]);
    let layout = StaticSourceLayout::new(vec![
        source_package("algebra", vec![source_file("src/lib.miz", "lib.miz")]),
        source_package("topology", vec![source_file("src/lib.miz", "lib.miz")]),
    ]);
    let index = build_module_index(&plan, &layout, &[]).expect("valid module index");
    let algebra_lib = ModuleId::new(PackageId::new("algebra"), ModulePath::new("lib"));
    let topology_lib = ModuleId::new(PackageId::new("topology"), ModulePath::new("lib"));

    let algebra_entry = index.module(&algebra_lib).expect("algebra lib");
    let topology_entry = index.module(&topology_lib).expect("topology lib");

    assert_ne!(algebra_entry.module.package, topology_entry.module.package);
    assert_eq!(
        algebra_entry.module_path.as_str(),
        topology_entry.module_path.as_str()
    );
}

#[test]
fn module_index_is_deterministic_for_shuffled_sources_and_artifacts() {
    let plan = build_plan(vec![
        workspace_package("algebra", "1.0.0"),
        registry_package("registry_dep", "1.0.0"),
        registry_package("second_dep", "1.0.0"),
    ]);
    let first_layout = StaticSourceLayout::new(vec![source_package(
        "algebra",
        vec![
            source_file("src/zeta.miz", "zeta.miz"),
            source_file("src/alpha.miz", "alpha.miz"),
        ],
    )]);
    let second_layout = StaticSourceLayout::new(vec![source_package(
        "algebra",
        vec![
            source_file("src/alpha.miz", "alpha.miz"),
            source_file("src/zeta.miz", "zeta.miz"),
        ],
    )]);
    let first_artifacts = vec![
        dependency_artifact(
            "registry_dep",
            vec![
                ArtifactNamespaceBinding::new(NamespaceRoot::Pkg, vec!["registry_dep".to_owned()]),
                ArtifactNamespaceBinding::new(
                    NamespaceRoot::Dev,
                    vec!["local_registry_dep".to_owned()],
                ),
            ],
            vec![
                summary("registry_dep", "zeta", "build/zeta.mizir.json", 9),
                summary("registry_dep", "core", "build/core.mizir.json", 7),
            ],
        ),
        dependency_artifact(
            "second_dep",
            vec![ArtifactNamespaceBinding::new(
                NamespaceRoot::Pkg,
                vec!["second_dep".to_owned()],
            )],
            vec![summary("second_dep", "lib", "build/lib.mizir.json", 5)],
        ),
    ];
    let second_artifacts = vec![
        dependency_artifact(
            "second_dep",
            vec![ArtifactNamespaceBinding::new(
                NamespaceRoot::Pkg,
                vec!["second_dep".to_owned()],
            )],
            vec![summary("second_dep", "lib", "build/lib.mizir.json", 5)],
        ),
        dependency_artifact(
            "registry_dep",
            vec![
                ArtifactNamespaceBinding::new(
                    NamespaceRoot::Dev,
                    vec!["local_registry_dep".to_owned()],
                ),
                ArtifactNamespaceBinding::new(NamespaceRoot::Pkg, vec!["registry_dep".to_owned()]),
            ],
            vec![
                summary("registry_dep", "core", "build/core.mizir.json", 7),
                summary("registry_dep", "zeta", "build/zeta.mizir.json", 9),
            ],
        ),
    ];

    let first = build_module_index(&plan, &first_layout, &first_artifacts).expect("first index");
    let second =
        build_module_index(&plan, &second_layout, &second_artifacts).expect("second index");

    assert_eq!(first, second);
}

#[test]
fn dependency_summaries_are_module_entries_without_source_paths() {
    let plan = build_plan(vec![registry_package("registry_dep", "1.0.0")]);
    let layout = StaticSourceLayout::default();
    let artifacts = vec![dependency_artifact(
        "registry_dep",
        Vec::new(),
        vec![summary("registry_dep", "core", "build/core.mizir.json", 3)],
    )];

    let index = build_module_index(&plan, &layout, &artifacts).expect("valid index");
    let module = ModuleId::new(PackageId::new("registry_dep"), ModulePath::new("core"));
    let entry = index.module(&module).expect("dependency module");
    let summary = index.dependency_summary(&module).expect("summary");

    assert_eq!(summary.artifact, "build/core.mizir.json");
    assert!(matches!(
        entry.location,
        ModuleIndexLocation::DependencySummary { .. }
    ));
}

#[test]
fn provider_reports_expected_ordering_and_errors() {
    let plan = build_plan(vec![
        workspace_package("empty_pkg", "1.0.0"),
        workspace_package("algebra", "1.0.0"),
    ]);
    let layout = StaticSourceLayout::new(vec![
        source_package("empty_pkg", Vec::new()),
        source_package("algebra", vec![source_file("src/lib.miz", "lib.miz")]),
    ]);
    let index = build_module_index(&plan, &layout, &[]).expect("valid index");

    assert_eq!(
        index
            .packages()
            .iter()
            .map(|package| package.package_id.as_str())
            .collect::<Vec<_>>(),
        vec!["empty_pkg", "algebra"]
    );
    assert_eq!(
        index
            .namespace_bindings()
            .iter()
            .map(|binding| (binding.root, binding.prefix.join(".")))
            .collect::<Vec<_>>(),
        vec![
            (NamespaceRoot::PackageName, "algebra".to_owned()),
            (NamespaceRoot::PackageName, "empty_pkg".to_owned()),
        ]
    );
    assert!(
        index
            .modules_for_package(&PackageId::new("empty_pkg"))
            .expect("known empty package")
            .is_empty()
    );
    assert!(matches!(
        index.package(&PackageId::new("missing")),
        Err(ModuleIndexProviderError::UnknownPackage { .. })
    ));
    assert!(matches!(
        index.package_for_namespace(&NamespaceRoot::Pkg, &["missing".to_owned()]),
        Err(ModuleIndexProviderError::UnknownNamespace { .. })
    ));
    assert!(matches!(
        index.module(&ModuleId::new(
            PackageId::new("algebra"),
            ModulePath::new("missing")
        )),
        Err(ModuleIndexProviderError::UnknownModule { .. })
    ));
    assert!(matches!(
        index.dependency_summary(&ModuleId::new(
            PackageId::new("algebra"),
            ModulePath::new("lib")
        )),
        Err(ModuleIndexProviderError::UnavailableDependencySummary { .. })
    ));
}

#[test]
fn invalid_source_paths_and_empty_module_paths_are_reported() {
    let plan = build_plan(vec![workspace_package("algebra", "1.0.0")]);
    let layout = StaticSourceLayout::new(vec![source_package(
        "algebra",
        vec![
            source_file("lib.miz", "lib.miz"),
            source_file("src/readme.txt", "readme.txt"),
            source_file("src/a.miz", "b.miz"),
            source_file("src/../bad.miz", "../bad.miz"),
            source_file("src/.miz", ".miz"),
        ],
    )]);

    let diagnostics = build_module_index(&plan, &layout, &[]).unwrap_err();

    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                matches!(
                    diagnostic.kind,
                    ModuleIndexDiagnosticKind::InvalidSourcePath
                )
            })
            .count(),
        4
    );
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(diagnostic.kind, ModuleIndexDiagnosticKind::EmptyModulePath)
            && diagnostic.value.as_deref() == Some(".miz")
    }));
}

#[test]
fn missing_layout_and_dependency_artifact_diagnostics_are_reported() {
    let plan = build_plan(vec![
        workspace_package("algebra", "1.0.0"),
        registry_package("registry_dep", "1.0.0"),
    ]);
    let layout = StaticSourceLayout::default();

    let diagnostics = build_module_index(&plan, &layout, &[]).unwrap_err();

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ModuleIndexDiagnosticKind::MissingSourceLayout
        ) && diagnostic.package_id.as_deref() == Some("algebra")
    }));
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ModuleIndexDiagnosticKind::MissingDependencySummary
        ) && diagnostic.package_id.as_deref() == Some("registry_dep")
    }));
}

#[test]
fn dependency_artifact_identity_and_namespace_inputs_are_validated() {
    let plan = build_plan(vec![registry_package("registry_dep", "1.0.0")]);
    let layout = StaticSourceLayout::default();
    let artifacts = vec![dependency_artifact(
        "registry_dep",
        vec![
            ArtifactNamespaceBinding::new(NamespaceRoot::PackageName, vec!["alias".to_owned()]),
            ArtifactNamespaceBinding::new(NamespaceRoot::Pkg, vec!["bad-name".to_owned()]),
        ],
        vec![
            summary("other_dep", "core", "build/other.mizir.json", 1),
            summary("registry_dep", "bad-name", "build/bad.mizir.json", 2),
        ],
    )];

    let diagnostics = build_module_index(&plan, &layout, &artifacts).unwrap_err();

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ModuleIndexDiagnosticKind::UnsupportedNamespaceRoot
        )
    }));
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ModuleIndexDiagnosticKind::InvalidNamespacePrefix
        ) && diagnostic.value.as_deref() == Some("bad-name")
    }));
    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                matches!(
                    diagnostic.kind,
                    ModuleIndexDiagnosticKind::MalformedSummaryIdentity
                )
            })
            .count(),
        2
    );
}

#[test]
fn diagnostics_are_sorted_independent_of_input_order() {
    let plan = build_plan(vec![
        workspace_package("algebra", "1.0.0"),
        registry_package("registry_dep", "1.0.0"),
    ]);
    let first_layout = StaticSourceLayout::new(vec![source_package(
        "algebra",
        vec![
            source_file("src/zeta.txt", "zeta.txt"),
            source_file("src/bad-name.miz", "bad-name.miz"),
        ],
    )]);
    let second_layout = StaticSourceLayout::new(vec![source_package(
        "algebra",
        vec![
            source_file("src/bad-name.miz", "bad-name.miz"),
            source_file("src/zeta.txt", "zeta.txt"),
        ],
    )]);
    let first_artifacts = vec![
        dependency_artifact(
            "unknown_dep",
            Vec::new(),
            vec![summary("unknown_dep", "lib", "build/unknown.mizir.json", 1)],
        ),
        dependency_artifact(
            "registry_dep",
            Vec::new(),
            vec![summary(
                "registry_dep",
                "bad-name",
                "build/bad.mizir.json",
                2,
            )],
        ),
    ];
    let second_artifacts = vec![
        dependency_artifact(
            "registry_dep",
            Vec::new(),
            vec![summary(
                "registry_dep",
                "bad-name",
                "build/bad.mizir.json",
                2,
            )],
        ),
        dependency_artifact(
            "unknown_dep",
            Vec::new(),
            vec![summary("unknown_dep", "lib", "build/unknown.mizir.json", 1)],
        ),
    ];

    let first = build_module_index(&plan, &first_layout, &first_artifacts)
        .unwrap_err()
        .into_diagnostics();
    let second = build_module_index(&plan, &second_layout, &second_artifacts)
        .unwrap_err()
        .into_diagnostics();

    assert_eq!(first, second);
}

#[test]
fn duplicate_dependency_summary_modules_are_rejected() {
    let plan = build_plan(vec![registry_package("registry_dep", "1.0.0")]);
    let layout = StaticSourceLayout::default();
    let artifacts = vec![dependency_artifact(
        "registry_dep",
        Vec::new(),
        vec![
            summary("registry_dep", "core", "build/first.mizir.json", 1),
            summary("registry_dep", "core", "build/second.mizir.json", 2),
        ],
    )];

    let diagnostics = build_module_index(&plan, &layout, &artifacts).unwrap_err();

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(diagnostic.kind, ModuleIndexDiagnosticKind::DuplicateModule)
            && diagnostic.value.as_deref() == Some("1:build/second.mizir.json:0202020202020202020202020202020202020202020202020202020202020202")
    }));
}

#[test]
fn duplicate_modules_are_rejected_deterministically() {
    let plan = build_plan(vec![workspace_package("algebra", "1.0.0")]);
    let layout = StaticSourceLayout::new(vec![source_package(
        "algebra",
        vec![
            source_file("src/groups/basic.miz", "groups/basic.miz"),
            source_file("src/groups/basic.miz", "groups/basic.miz"),
        ],
    )]);

    let diagnostics = build_module_index(&plan, &layout, &[]).unwrap_err();

    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .filter(|diagnostic| {
                matches!(diagnostic.kind, ModuleIndexDiagnosticKind::DuplicateModule)
            })
            .count(),
        1
    );
}

#[test]
fn invalid_module_components_are_rejected() {
    let plan = build_plan(vec![workspace_package("algebra", "1.0.0")]);
    let layout = StaticSourceLayout::new(vec![source_package(
        "algebra",
        vec![
            source_file("src/bad-name.miz", "bad-name.miz"),
            source_file("src/foo.bar.miz", "foo.bar.miz"),
            source_file("src/foo/bar.miz", "foo/bar.miz"),
        ],
    )]);

    let diagnostics = build_module_index(&plan, &layout, &[]).unwrap_err();

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ModuleIndexDiagnosticKind::InvalidModuleComponent
        ) && diagnostic.value.as_deref() == Some("bad-name")
    }));
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ModuleIndexDiagnosticKind::InvalidModuleComponent
        ) && diagnostic.value.as_deref() == Some("foo.bar")
    }));
    assert!(
        !diagnostics.diagnostics().iter().any(|diagnostic| {
            matches!(diagnostic.kind, ModuleIndexDiagnosticKind::DuplicateModule)
        }),
        "dotted filenames are invalid before they can collide with directory modules"
    );
}

#[test]
fn namespace_binding_conflicts_are_rejected() {
    let plan = build_plan(vec![
        registry_package("first_dep", "1.0.0"),
        registry_package("second_dep", "1.0.0"),
    ]);
    let layout = StaticSourceLayout::default();
    let artifacts = vec![
        dependency_artifact(
            "first_dep",
            vec![ArtifactNamespaceBinding::new(
                NamespaceRoot::Std,
                vec!["core".to_owned()],
            )],
            vec![summary("first_dep", "lib", "build/first.mizir.json", 1)],
        ),
        dependency_artifact(
            "second_dep",
            vec![ArtifactNamespaceBinding::new(
                NamespaceRoot::Std,
                vec!["core".to_owned()],
            )],
            vec![summary("second_dep", "lib", "build/second.mizir.json", 2)],
        ),
    ];

    let diagnostics = build_module_index(&plan, &layout, &artifacts).unwrap_err();

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ModuleIndexDiagnosticKind::DuplicateNamespaceBinding
        )
    }));
}

fn build_plan(packages: Vec<PackagePlan>) -> BuildPlan {
    BuildPlan {
        workspace_root: WorkspaceRoot::new("."),
        packages,
        dependency_graph: DependencyGraph { edges: Vec::new() },
        lockfile: Lockfile {
            schema_version: 1,
            packages: Vec::new(),
        },
        toolchain: ToolchainInfo::new("test"),
        verifier_config: WorkspaceVerifierConfig {
            packages: Vec::new(),
        },
        build_config: WorkspaceBuildConfig {
            packages: Vec::new(),
        },
    }
}

fn workspace_package(package_id: &str, version: &str) -> PackagePlan {
    package_plan(
        package_id,
        version,
        PackagePlanSource::Workspace {
            root: package_id.to_owned(),
            source_root: format!("{package_id}/src"),
            manifest_path: format!("{package_id}/mizar.pkg"),
        },
    )
}

fn registry_package(package_id: &str, version: &str) -> PackagePlan {
    package_plan(
        package_id,
        version,
        PackagePlanSource::Registry {
            registry: "default".to_owned(),
            checksum: format!("sha256:{package_id}"),
        },
    )
}

fn package_plan(package_id: &str, version: &str, source: PackagePlanSource) -> PackagePlan {
    PackagePlan {
        package_id: PackageId::new(package_id),
        version: Version::parse(version).expect("valid version"),
        source,
        edition: Edition::new("2025"),
        dependencies: Vec::new(),
        verifier_config: VerifierConfig::default(),
        build_config: BuildConfig::default(),
    }
}

fn source_package(package_id: &str, files: Vec<WorkspaceSourceFile>) -> WorkspaceSourcePackage {
    WorkspaceSourcePackage {
        package_id: PackageId::new(package_id),
        files,
    }
}

fn source_file(normalized_path: &str, source_relative_path: &str) -> WorkspaceSourceFile {
    WorkspaceSourceFile::new(normalized_path, source_relative_path)
}

fn dependency_artifact(
    package_id: &str,
    namespace_bindings: Vec<ArtifactNamespaceBinding>,
    summaries: Vec<DependencyModuleSummaryRef>,
) -> DependencyArtifactIndex {
    DependencyArtifactIndex::new(PackageId::new(package_id), namespace_bindings, summaries)
}

fn summary(
    package_id: &str,
    module_path: &str,
    artifact: &str,
    hash_seed: u8,
) -> DependencyModuleSummaryRef {
    DependencyModuleSummaryRef {
        module: ModuleId::new(PackageId::new(package_id), ModulePath::new(module_path)),
        artifact: artifact.to_owned(),
        content_hash: Hash::from_bytes([hash_seed; Hash::BYTE_LEN]),
    }
}
