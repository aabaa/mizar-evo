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
use mizar_artifact::{
    manifest::{
        ArtifactManifest, ManifestProvenance, ModuleArtifactEntry, PackageIdentity,
        artifact_manifest_json, artifact_manifest_path, current_schema_version,
        write_manifest_file,
    },
    module_summary::{MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummaryIdentity},
    registration_summary::{ArtifactHashClass, ArtifactHashRef},
    store::{CanonicalJson, PublishedArtifactReadOptions, SchemaVersion, read_published_artifact},
    verified_artifact::VERIFIED_ARTIFACT_SCHEMA_FAMILY,
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

#[test]
fn indexed_summary_read_validates_real_files_hashes_and_known_identity() {
    use mizar_artifact::{
        module_summary::{
            ModuleLexicalSummary, ModuleSummary, current_schema_version as summary_schema,
            module_summary_json,
        },
        store::{
            PublishedArtifactPath, artifact_hash_domain, canonical_json_bytes,
            write_published_artifact,
        },
    };

    let root = std::env::temp_dir().join(format!(
        "mizar-build-indexed-summary-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    // Storage fixture only; no source export or proof producer is substituted.
    let mut summary = ModuleSummary {
        schema_version: summary_schema(),
        module: ModuleSummaryIdentity {
            package_id: "dep".into(),
            package_version: Some("1.0.0".into()),
            lockfile_identity: Some("dependency-lock".into()),
            module_path: "core".into(),
            language_edition: "2026".into(),
        },
        source_hash: Hash::from_bytes([1; Hash::BYTE_LEN]),
        interface_hash: Hash::from_bytes([0; Hash::BYTE_LEN]),
        exported_symbols: Vec::new(),
        exported_labels: Vec::new(),
        lexical_summary: ModuleLexicalSummary {
            schema_version: "mizar-resolve/exported-lexical/v1".into(),
            fingerprint: None,
            contributions: Vec::new(),
        },
        reexports: Vec::new(),
        dependency_interfaces: Vec::new(),
    };
    summary.refresh_interface_hash().unwrap();
    let value = module_summary_json(&summary).unwrap();
    let path = PublishedArtifactPath::new("summary.json").unwrap();
    let domain = artifact_hash_domain(MODULE_SUMMARY_SCHEMA_FAMILY, summary_schema());
    let publish = |value: &CanonicalJson| {
        write_published_artifact(&root, &path, value, &domain, &[])
            .unwrap()
            .artifact_hash
    };
    let reference = DependencyModuleSummaryRef {
        module: ModuleId::new(PackageId::new("dep"), ModulePath::new("core")),
        artifact: path.as_str().into(),
        content_hash: publish(&value),
    };
    let read = reference.read_current_summary(&root).unwrap();
    assert_eq!(read, value);
    assert_eq!(
        canonical_json_bytes(&read),
        std::fs::read(root.join(path.as_str())).unwrap()
    );
    assert_ne!(reference.content_hash, summary.interface_hash);
    let mut invalid = reference.clone();
    invalid.content_hash = summary.interface_hash;
    assert!(invalid.read_current_summary(&root).is_none());
    for module in [
        ModuleId::new(PackageId::new("other"), ModulePath::new("core")),
        ModuleId::new(PackageId::new("dep"), ModulePath::new("other")),
    ] {
        let mut invalid = reference.clone();
        invalid.module = module;
        assert!(invalid.read_current_summary(&root).is_none());
    }
    let mut invalid = reference.clone();
    invalid.artifact = "../summary.json".into();
    assert!(invalid.read_current_summary(&root).is_none());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join(path.as_str()), root.join("link.json")).unwrap();
        invalid.artifact = "link.json".into();
        assert!(invalid.read_current_summary(&root).is_none());
    }
    for schema_failure in [false, true] {
        let mut corrupt = value.clone();
        let CanonicalJson::Object(fields) = &mut corrupt else {
            panic!("summary object")
        };
        if schema_failure {
            fields.insert("schema_version".into(), CanonicalJson::string("2.0"));
        } else {
            let CanonicalJson::String(hash) = fields.get_mut("interface_hash").unwrap() else {
                panic!("interface hash string")
            };
            let last = hash.pop().unwrap();
            hash.push(if last == '0' { '1' } else { '0' });
        }
        // The store hash matches: rejection must come from the summary reader.
        let mut invalid = reference.clone();
        invalid.content_hash = publish(&corrupt);
        assert!(invalid.read_current_summary(&root).is_none());
    }
    assert_eq!(publish(&value), reference.content_hash);
    assert_eq!(reference.read_current_summary(&root), Some(value));
    std::fs::remove_file(root.join(path.as_str())).unwrap();
    assert!(reference.read_current_summary(&root).is_none());
    std::fs::write(root.join(path.as_str()), b"not JSON").unwrap();
    assert!(reference.read_current_summary(&root).is_none());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn manifest_dependency_index_projects_stored_metadata_and_rejects_invalid_shapes() {
    let plan = build_plan(vec![registry_package("registry_dep", "1.0.0")]);
    let package = &plan.packages[0];
    let manifest_schema = current_schema_version();
    let summary_schema = mizar_artifact::module_summary::current_schema_version();
    let hash_ref = |class, family: &str, seed, schema| {
        ArtifactHashRef::new(
            class,
            family,
            schema,
            Hash::from_bytes([seed; Hash::BYTE_LEN]),
        )
    };
    let entry = |module_path: &str, seed: u8, with_summary: bool| ModuleArtifactEntry {
        module: ModuleSummaryIdentity {
            package_id: "registry_dep".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("lock".to_owned()),
            module_path: module_path.to_owned(),
            language_edition: "2025".to_owned(),
        },
        source_file: format!("src/{module_path}.miz"),
        source_hash: Hash::from_bytes([seed; Hash::BYTE_LEN]),
        artifact_file: format!("artifacts/{module_path}.mizir.json"),
        artifact_hash: hash_ref(
            ArtifactHashClass::Artifact,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            seed,
            manifest_schema,
        ),
        interface_hash: hash_ref(
            ArtifactHashClass::Interface,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            seed + 1,
            manifest_schema,
        ),
        implementation_hash: hash_ref(
            ArtifactHashClass::Implementation,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            seed + 2,
            manifest_schema,
        ),
        module_summary_file: with_summary.then(|| format!("summaries/{module_path}.json")),
        module_summary_hash: with_summary.then(|| {
            hash_ref(
                ArtifactHashClass::Artifact,
                MODULE_SUMMARY_SCHEMA_FAMILY,
                seed + 3,
                summary_schema,
            )
        }),
        module_summary_interface_hash: with_summary.then(|| {
            hash_ref(
                ArtifactHashClass::Interface,
                MODULE_SUMMARY_SCHEMA_FAMILY,
                seed + 4,
                summary_schema,
            )
        }),
        registration_summary_file: None,
        registration_summary_hash: None,
        registration_interface_hash: None,
        proof_witnesses: Vec::new(),
        diagnostics_hash: None,
    };
    let manifest = ArtifactManifest {
        schema_version: manifest_schema,
        package: PackageIdentity {
            package_id: "registry_dep".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("lock".to_owned()),
        },
        artifact_root: "build".to_owned(),
        lockfile_hash: hash_ref(
            ArtifactHashClass::Artifact,
            "mizar-build/lockfile",
            30,
            manifest_schema,
        ),
        toolchain: "mizar-evo-test".to_owned(),
        language_edition: "2025".to_owned(),
        verifier_config_hash: hash_ref(
            ArtifactHashClass::Interface,
            "mizar-build/verifier-config",
            31,
            manifest_schema,
        ),
        modules: vec![entry("alpha", 1, true), entry("beta", 10, false)],
        development_artifacts: Vec::new(),
        provenance: ManifestProvenance {
            generated_by: "mizar-build-test".to_owned(),
            manifest_policy: "test-policy".to_owned(),
            transaction_format: "manifest-transaction-v1".to_owned(),
        },
    };
    let namespace_bindings = vec![ArtifactNamespaceBinding::new(
        NamespaceRoot::Pkg,
        vec!["dep_alias".to_owned()],
    )];
    let root = std::env::temp_dir().join(format!(
        "mizar-build-module-index-manifest-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("manifest test root");
    let raw_index = |value: CanonicalJson| {
        DependencyArtifactIndex::from_manifest(package, &value, namespace_bindings.clone())
    };

    write_manifest_file(&root, &manifest).expect("manifest write");
    let path = artifact_manifest_path();
    let stored = read_published_artifact(&root, &path, PublishedArtifactReadOptions::default())
        .expect("stored manifest read");
    let projected =
        DependencyArtifactIndex::from_manifest(package, &stored.value, namespace_bindings.clone())
            .expect("known package manifest");
    assert_eq!(projected.namespace_bindings, namespace_bindings);
    assert_eq!(projected.summaries.len(), 1, "all-null sidecar is omitted");
    assert_eq!(projected.summaries[0].artifact, "summaries/alpha.json");
    assert_eq!(
        projected.summaries[0].content_hash,
        Hash::from_bytes([4; Hash::BYTE_LEN])
    );
    let index = build_module_index(
        &plan,
        &StaticSourceLayout::default(),
        std::slice::from_ref(&projected),
    )
    .expect("manifest projection enters the existing index");
    let alpha = ModuleId::new(PackageId::new("registry_dep"), ModulePath::new("alpha"));
    assert!(matches!(
        &index.module(&alpha).expect("alpha module").location,
        ModuleIndexLocation::DependencySummary { artifact, content_hash }
            if artifact == "summaries/alpha.json"
                && *content_hash == Hash::from_bytes([4; Hash::BYTE_LEN])
    ));
    assert!(index.namespace_bindings().iter().any(|binding| {
        binding.root == NamespaceRoot::Pkg && binding.prefix == vec!["dep_alias".to_owned()]
    }));

    let mut empty = manifest.clone();
    empty.modules.remove(0);
    assert!(
        raw_index(artifact_manifest_json(&empty).expect("empty manifest JSON"))
            .expect("empty sidecar projection")
            .summaries
            .is_empty()
    );

    let reject_manifest = |mut value: ArtifactManifest, update: fn(&mut ArtifactManifest)| {
        update(&mut value);
        raw_index(artifact_manifest_json(&value).expect("mutated manifest JSON")).is_none()
    };
    let identity_mutations: [fn(&mut ArtifactManifest); 9] = [
        |value| value.package.package_id = "other_dep".to_owned(),
        |value| value.package.package_version = None,
        |value| value.package.package_version = Some("9.0.0".to_owned()),
        |value| value.language_edition = "2026".to_owned(),
        |value| value.modules[1].module.package_id = "other_dep".to_owned(),
        |value| value.modules[1].module.package_version = None,
        |value| value.modules[1].module.package_version = Some("9.0.0".to_owned()),
        |value| value.modules[1].module.language_edition = "2026".to_owned(),
        |value| value.modules[1].module.lockfile_identity = Some("other-lock".to_owned()),
    ];
    for update in identity_mutations {
        assert!(reject_manifest(manifest.clone(), update));
    }
    assert!(reject_manifest(manifest.clone(), |value| {
        value.modules[0]
            .module_summary_hash
            .as_mut()
            .expect("summary hash")
            .schema_version = SchemaVersion::new(2, 0);
        value.modules[0]
            .module_summary_interface_hash
            .as_mut()
            .expect("summary interface hash")
            .schema_version = SchemaVersion::new(2, 0);
    }));

    let mutate_first = |mut value: CanonicalJson, field: &str, replacement: CanonicalJson| {
        let CanonicalJson::Object(fields) = &mut value else {
            panic!("manifest JSON object")
        };
        let CanonicalJson::Array(modules) = fields.get_mut("modules").expect("modules") else {
            panic!("manifest modules array")
        };
        let CanonicalJson::Object(entry) = modules.first_mut().expect("first module") else {
            panic!("manifest module object")
        };
        entry.insert(field.to_owned(), replacement);
        value
    };
    let value = artifact_manifest_json(&manifest).expect("path JSON");
    assert!(
        raw_index(mutate_first(
            value,
            "module_summary_file",
            CanonicalJson::string("../summary.json")
        ))
        .is_none()
    );
    let value = artifact_manifest_json(&manifest).expect("hash JSON");
    let wrong_class = hash_ref(
        ArtifactHashClass::Interface,
        MODULE_SUMMARY_SCHEMA_FAMILY,
        4,
        summary_schema,
    )
    .to_artifact_hash_string();
    assert!(
        raw_index(mutate_first(
            value,
            "module_summary_hash",
            CanonicalJson::string(wrong_class)
        ))
        .is_none()
    );
    let value = artifact_manifest_json(&manifest).expect("group JSON");
    assert!(
        raw_index(mutate_first(
            value,
            "module_summary_hash",
            CanonicalJson::null()
        ))
        .is_none()
    );
    let mut value = artifact_manifest_json(&manifest).expect("order JSON");
    let CanonicalJson::Object(fields) = &mut value else {
        panic!("manifest JSON object")
    };
    let CanonicalJson::Array(modules) = fields.get_mut("modules").expect("modules") else {
        panic!("manifest modules array")
    };
    modules.reverse();
    assert!(raw_index(value).is_none());

    let bad_namespace = vec![ArtifactNamespaceBinding::new(
        NamespaceRoot::Pkg,
        vec!["bad-name".to_owned()],
    )];
    let bad = DependencyArtifactIndex::from_manifest(package, &stored.value, bad_namespace.clone())
        .expect("namespace metadata is caller-owned");
    assert_eq!(bad.namespace_bindings, bad_namespace);
    let diagnostics = build_module_index(
        &plan,
        &StaticSourceLayout::default(),
        std::slice::from_ref(&bad),
    )
    .expect_err("existing namespace validator rejects bad prefix");
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        matches!(
            diagnostic.kind,
            ModuleIndexDiagnosticKind::InvalidNamespacePrefix
        )
    }));
    let _ = std::fs::remove_dir_all(root);
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
