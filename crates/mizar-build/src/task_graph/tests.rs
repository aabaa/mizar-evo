use super::*;
use crate::{
    module_index::{ModuleIndexLocation, PackageIndexEntry, PackageIndexSource},
    planner::{
        BuildConfig, DependencyGraph, DependencyKind, Lockfile, PackagePlan, PackagePlanSource,
        VerifierConfig, WorkspaceBuildConfig, WorkspaceVerifierConfig,
    },
};
use mizar_session::{Edition, Hash, ModulePath, ToolchainInfo, WorkspaceRoot};
use semver::Version;

#[test]
fn task_ids_are_deterministic_and_snapshot_scoped() {
    let graph = graph_for_modules(vec![workspace_module("app", "main")], Vec::new());
    let second_graph = graph_for_modules(vec![workspace_module("app", "main")], Vec::new());
    let different_snapshot = snapshot(15);
    let different_snapshot_graph = graph_for_modules_with_snapshot(
        different_snapshot,
        vec![workspace_module("app", "main")],
        Vec::new(),
    );
    let different_snapshot_identity = different_snapshot
        .to_published_schema_string()
        .expect("snapshot identity serializes");

    let ids = task_ids(&graph);
    assert_eq!(ids, task_ids(&second_graph));
    assert_ne!(ids, task_ids(&different_snapshot_graph));
    assert!(
        task_ids(&different_snapshot_graph)
            .iter()
            .all(|id| id.contains(&escape_component(&different_snapshot_identity)))
    );
    assert_eq!(graph.tasks[0].kind, TaskKind::PackageResolve);
    assert!(
        ids.iter()
            .all(|id| id.contains("mizar-session-build-snapshot-v1"))
    );
    assert!(
        ids.iter()
            .all(|id| !contains_forbidden_authority_term(id.as_str()))
    );
}

#[test]
fn workspace_modules_create_phase_tasks_and_summary_modules_remain_inputs() {
    let graph = graph_for_modules(
        vec![
            workspace_module("app", "main"),
            dependency_summary_module("registry_dep", "core"),
        ],
        Vec::new(),
    );

    let workspace_kinds = module_task_kinds(&graph, "app", "main");
    assert_eq!(
        workspace_kinds,
        vec![
            TaskKind::SourceLoad,
            TaskKind::Frontend,
            TaskKind::ModuleResolve,
            TaskKind::CheckAndElaborate,
            TaskKind::VcGenerate,
            TaskKind::ArtifactCommit,
        ]
    );
    assert!(module_task_kinds(&graph, "registry_dep", "core").is_empty());

    let check_task = module_task(&graph, TaskKind::CheckAndElaborate, "app", "main");
    assert_eq!(
        check_task.phases,
        vec![
            PipelinePhase::SignatureCollection,
            PipelinePhase::TypeChecking,
            PipelinePhase::RegistrationResolution,
            PipelinePhase::OverloadResolution,
            PipelinePhase::Elaboration,
            PipelinePhase::AlgorithmPreparation,
        ]
    );
    assert_eq!(check_task.resource_class, ResourceClass::CpuLocal);
    assert_eq!(check_task.priority_class, PriorityClass::Semantic);
}

#[test]
fn tasks_preserve_package_module_descriptor_backend_and_evidence_ordering() {
    let app_main = module_id("app", "main");
    let graph = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(1),
        build_plan: build_plan(
            vec![workspace_package("lib"), workspace_package("app")],
            Vec::new(),
        ),
        module_index: module_index(vec![
            workspace_module("lib", "core"),
            workspace_module("app", "main"),
        ]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: vec![
            vc_descriptor(
                "vc-late",
                app_main.clone(),
                "002",
                vec!["z3", "cvc5"],
                vec!["candidate-b", "candidate-a"],
            ),
            vc_descriptor("vc-early", app_main, "001", vec!["vampire"], Vec::new()),
        ],
        profile: TaskGraphProfile {
            documentation: DocumentationProfile::Enabled,
            ..TaskGraphProfile::default()
        },
    })
    .expect("task graph builds");

    assert_eq!(
        graph
            .tasks
            .iter()
            .filter(|task| task.kind == TaskKind::SourceLoad)
            .map(module_label)
            .collect::<Vec<_>>(),
        vec!["lib:core", "app:main"]
    );
    assert_eq!(
        graph
            .tasks
            .iter()
            .filter(|task| task.kind == TaskKind::VcDischarge)
            .map(descriptor_label)
            .collect::<Vec<_>>(),
        vec!["vc-early", "vc-late"]
    );
    assert_eq!(
        graph
            .tasks
            .iter()
            .filter(|task| task.kind == TaskKind::BackendRun)
            .map(backend_label)
            .collect::<Vec<_>>(),
        vec!["vampire", "z3", "cvc5"]
    );
    assert_eq!(
        graph
            .tasks
            .iter()
            .filter(|task| task.kind == TaskKind::KernelCheck)
            .map(evidence_label)
            .collect::<Vec<_>>(),
        vec!["candidate-a", "candidate-b"]
    );
    assert_eq!(
        graph
            .tasks
            .iter()
            .filter(|task| task.kind == TaskKind::DocumentationExtract)
            .map(package_label)
            .collect::<Vec<_>>(),
        vec!["lib", "app"]
    );
}

#[test]
fn required_edge_rules_cover_root_pipeline_proof_commit_and_docs() {
    let graph = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(10),
        build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
        module_index: module_index(vec![workspace_module("app", "main")]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: vec![vc_descriptor(
            "vc-main",
            module_id("app", "main"),
            "001",
            vec!["vampire"],
            vec!["kernel-candidate"],
        )],
        profile: TaskGraphProfile {
            documentation: DocumentationProfile::Enabled,
            ..TaskGraphProfile::default()
        },
    })
    .expect("task graph builds");

    let root = single_task(&graph, TaskKind::PackageResolve);
    let source = module_task(&graph, TaskKind::SourceLoad, "app", "main");
    let frontend = module_task(&graph, TaskKind::Frontend, "app", "main");
    let module_resolve = module_task(&graph, TaskKind::ModuleResolve, "app", "main");
    let check = module_task(&graph, TaskKind::CheckAndElaborate, "app", "main");
    let vc_generate = module_task(&graph, TaskKind::VcGenerate, "app", "main");
    let vc_discharge = single_task(&graph, TaskKind::VcDischarge);
    let atp = single_task(&graph, TaskKind::AtpSolve);
    let backend = single_task(&graph, TaskKind::BackendRun);
    let kernel = single_task(&graph, TaskKind::KernelCheck);
    let artifact = module_task(&graph, TaskKind::ArtifactCommit, "app", "main");
    let documentation = single_task(&graph, TaskKind::DocumentationExtract);

    for task in graph
        .tasks
        .iter()
        .filter(|task| task.kind != TaskKind::PackageResolve)
    {
        assert_has_edge(&graph, task, root);
    }
    assert_has_edge(&graph, frontend, source);
    assert_has_edge(&graph, module_resolve, frontend);
    assert_has_edge(&graph, check, module_resolve);
    assert_has_edge(&graph, vc_generate, check);
    assert_has_edge(&graph, vc_discharge, vc_generate);
    assert_has_edge(&graph, atp, vc_discharge);
    assert_has_edge(&graph, backend, atp);
    assert_has_edge(&graph, kernel, backend);
    assert_has_edge(&graph, artifact, vc_generate);
    assert_has_edge(&graph, artifact, vc_discharge);
    assert_has_edge(&graph, artifact, atp);
    assert_has_edge(&graph, artifact, backend);
    assert_has_edge(&graph, artifact, kernel);
    assert_has_edge(&graph, documentation, artifact);
    assert_dependencies_match_edges(&graph);
    assert_edges_are_sorted(&graph);

    assert_task_metadata(
        root,
        DependencyCoverage::Complete,
        ResourceClass::Coordinator,
        PriorityClass::Root,
    );
    assert_task_metadata(
        source,
        DependencyCoverage::Complete,
        ResourceClass::SourceIo,
        PriorityClass::Source,
    );
    assert_task_metadata(
        vc_discharge,
        DependencyCoverage::Complete,
        ResourceClass::ProofLocal,
        PriorityClass::Proof,
    );
    assert_task_metadata(
        atp,
        DependencyCoverage::Complete,
        ResourceClass::AtpProcess,
        PriorityClass::Proof,
    );
    assert_task_metadata(
        backend,
        DependencyCoverage::Complete,
        ResourceClass::AtpProcess,
        PriorityClass::Proof,
    );
    assert_task_metadata(
        kernel,
        DependencyCoverage::Complete,
        ResourceClass::Kernel,
        PriorityClass::Proof,
    );
    assert_task_metadata(
        artifact,
        DependencyCoverage::Complete,
        ResourceClass::ArtifactIo,
        PriorityClass::Commit,
    );
    assert_task_metadata(
        documentation,
        DependencyCoverage::Complete,
        ResourceClass::Documentation,
        PriorityClass::Documentation,
    );
}

#[test]
fn kernel_checks_with_deterministic_evidence_wait_on_vc_discharge() {
    let graph = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(16),
        build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
        module_index: module_index(vec![workspace_module("app", "main")]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: vec![vc_descriptor(
            "vc-main",
            module_id("app", "main"),
            "001",
            Vec::new(),
            vec!["deterministic-candidate"],
        )],
        profile: TaskGraphProfile::default(),
    })
    .expect("task graph builds");

    let vc_discharge = single_task(&graph, TaskKind::VcDischarge);
    let kernel = single_task(&graph, TaskKind::KernelCheck);
    assert_has_edge(&graph, kernel, vc_discharge);
    assert!(
        graph
            .tasks
            .iter()
            .all(|task| { task.kind != TaskKind::AtpSolve && task.kind != TaskKind::BackendRun })
    );
}

#[test]
fn package_dependency_edges_gate_downstream_semantic_tasks() {
    let graph = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(2),
        build_plan: build_plan(
            vec![workspace_package("lib"), workspace_package("app")],
            vec![DependencyEdge {
                dependent: "app".to_owned(),
                dependency: "lib".to_owned(),
                kind: DependencyKind::Normal,
            }],
        ),
        module_index: module_index(vec![
            workspace_module("lib", "core"),
            workspace_module("app", "main"),
        ]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect("task graph builds");

    assert_has_edge(
        &graph,
        module_task(&graph, TaskKind::ModuleResolve, "app", "main"),
        module_task(&graph, TaskKind::ArtifactCommit, "lib", "core"),
    );
}

#[test]
fn dependency_summary_package_dependencies_are_ready_inputs() {
    let graph = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(11),
        build_plan: build_plan(
            vec![registry_package("dep"), workspace_package("app")],
            vec![DependencyEdge {
                dependent: "app".to_owned(),
                dependency: "dep".to_owned(),
                kind: DependencyKind::Normal,
            }],
        ),
        module_index: module_index(vec![
            dependency_summary_module("dep", "core"),
            workspace_module("app", "main"),
        ]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect("dependency-summary package is a ready input");

    assert!(module_task_kinds(&graph, "dep", "core").is_empty());
    assert_eq!(
        module_task(&graph, TaskKind::ModuleResolve, "app", "main")
            .dependencies
            .iter()
            .filter(|dependency| dependency.as_str().contains("dep%3Acore"))
            .count(),
        0
    );
}

#[test]
fn module_dependency_overlay_edges_gate_dependent_module_resolution() {
    let main = module_id("app", "main");
    let util = module_id("app", "util");
    let graph = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(3),
        build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
        module_index: module_index(vec![
            workspace_module("app", "util"),
            workspace_module("app", "main"),
        ]),
        dependency_overlay: ModuleDependencyOverlay::complete(vec![ModuleDependencyEdge::new(
            main,
            util,
            ModuleDependencyKind::ImportSummary,
        )]),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect("task graph builds");

    assert_has_edge(
        &graph,
        module_task(&graph, TaskKind::ModuleResolve, "app", "main"),
        module_task(&graph, TaskKind::ArtifactCommit, "app", "util"),
    );
}

#[test]
fn missing_module_dependency_coverage_is_diagnostic() {
    let missing = module_id("app", "main");
    let covered = module_id("app", "util");
    let diagnostics = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(4),
        build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
        module_index: module_index(vec![
            workspace_module("app", "main"),
            workspace_module("app", "util"),
        ]),
        dependency_overlay: ModuleDependencyOverlay {
            coverage: ModuleDependencyCoverage::CoveredModules(vec![covered]),
            edges: Vec::new(),
        },
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect_err("missing overlay is reported");

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind == TaskGraphDiagnosticKind::MissingModuleDependencyOverlay
            && diagnostic.module.as_ref() == Some(&missing)
    }));
}

#[test]
fn package_only_coverage_conservatively_marks_semantic_tasks() {
    let graph = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(12),
        build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
        module_index: module_index(vec![workspace_module("app", "main")]),
        dependency_overlay: ModuleDependencyOverlay::package_only(Vec::new()),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect("package-only coverage is conservatively schedulable");

    assert_eq!(
        module_task(&graph, TaskKind::SourceLoad, "app", "main").dependency_coverage,
        DependencyCoverage::Complete
    );
    assert_eq!(
        module_task(&graph, TaskKind::Frontend, "app", "main").dependency_coverage,
        DependencyCoverage::Complete
    );
    for kind in [
        TaskKind::ModuleResolve,
        TaskKind::CheckAndElaborate,
        TaskKind::VcGenerate,
        TaskKind::ArtifactCommit,
    ] {
        assert_eq!(
            module_task(&graph, kind, "app", "main").dependency_coverage,
            DependencyCoverage::PackageConservative
        );
    }
}

#[test]
fn unavailable_coverage_reports_missing_overlay_for_each_module() {
    let diagnostics = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(13),
        build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
        module_index: module_index(vec![
            workspace_module("app", "main"),
            workspace_module("app", "util"),
        ]),
        dependency_overlay: ModuleDependencyOverlay::unavailable(),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect_err("unavailable overlay is reported");

    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.kind)
            .collect::<Vec<_>>(),
        vec![
            TaskGraphDiagnosticKind::MissingModuleDependencyOverlay,
            TaskGraphDiagnosticKind::MissingModuleDependencyOverlay,
        ]
    );
    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.module.as_ref().map(module_key))
            .collect::<Vec<_>>(),
        vec![Some("app:main".to_owned()), Some("app:util".to_owned())]
    );
}

#[test]
fn required_vc_descriptors_mark_artifact_commit_coverage() {
    let diagnostics = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(5),
        build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
        module_index: module_index(vec![workspace_module("app", "main")]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile {
            vc_descriptor_policy: VcDescriptorPolicy::RequiredForArtifactCommit,
            ..TaskGraphProfile::default()
        },
    })
    .expect_err("missing VC descriptors are reported");

    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.kind)
            .collect::<Vec<_>>(),
        vec![TaskGraphDiagnosticKind::MissingVcDescriptors]
    );
}

#[test]
fn vc_descriptors_targeting_dependency_summaries_are_rejected() {
    let diagnostics = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(17),
        build_plan: build_plan(vec![registry_package("dep")], Vec::new()),
        module_index: module_index(vec![dependency_summary_module("dep", "core")]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: vec![vc_descriptor(
            "vc-dep",
            module_id("dep", "core"),
            "001",
            Vec::new(),
            Vec::new(),
        )],
        profile: TaskGraphProfile::default(),
    })
    .expect_err("dependency-summary descriptor is rejected");

    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.kind)
            .collect::<Vec<_>>(),
        vec![TaskGraphDiagnosticKind::BoundaryViolation]
    );
}

#[test]
fn duplicate_task_ids_are_rejected() {
    let diagnostics = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(6),
        build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
        module_index: module_index(vec![
            workspace_module("app", "main"),
            workspace_module("app", "main"),
        ]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect_err("duplicate task ids are reported");

    assert!(
        diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind == TaskGraphDiagnosticKind::DuplicateTaskId)
    );
}

#[test]
fn cyclic_dependencies_are_rejected() {
    let main = module_id("app", "main");
    let util = module_id("app", "util");
    let diagnostics = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(7),
        build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
        module_index: module_index(vec![
            workspace_module("app", "main"),
            workspace_module("app", "util"),
        ]),
        dependency_overlay: ModuleDependencyOverlay::complete(vec![
            ModuleDependencyEdge::new(
                main.clone(),
                util.clone(),
                ModuleDependencyKind::ImportSummary,
            ),
            ModuleDependencyEdge::new(util, main, ModuleDependencyKind::ImportSummary),
        ]),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect_err("dependency cycle is reported");

    assert!(
        diagnostics
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind == TaskGraphDiagnosticKind::DependencyCycle)
    );
}

#[test]
fn structural_diagnostics_are_stable_and_cover_unknowns_and_self_edges() {
    let diagnostics = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(8),
        build_plan: build_plan(
            vec![workspace_package("app")],
            vec![DependencyEdge {
                dependent: "app".to_owned(),
                dependency: "missing_pkg".to_owned(),
                kind: DependencyKind::Normal,
            }],
        ),
        module_index: module_index(vec![workspace_module("app", "main")]),
        dependency_overlay: ModuleDependencyOverlay::complete(vec![
            ModuleDependencyEdge::new(
                module_id("app", "main"),
                module_id("app", "main"),
                ModuleDependencyKind::ImportSummary,
            ),
            ModuleDependencyEdge::new(
                module_id("app", "main"),
                module_id("app", "missing"),
                ModuleDependencyKind::ImportSummary,
            ),
        ]),
        vc_descriptors: vec![vc_descriptor(
            "vc-missing",
            module_id("missing_pkg", "main"),
            "001",
            Vec::new(),
            Vec::new(),
        )],
        profile: TaskGraphProfile::default(),
    })
    .expect_err("structural diagnostics are reported");

    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.kind)
            .collect::<Vec<_>>(),
        vec![
            TaskGraphDiagnosticKind::SelfDependency,
            TaskGraphDiagnosticKind::UnknownModule,
            TaskGraphDiagnosticKind::UnknownPackage,
            TaskGraphDiagnosticKind::UnknownModule,
        ]
    );
}

#[test]
fn missing_overlay_diagnostics_sort_by_package_and_module() {
    let diagnostics = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot(14),
        build_plan: build_plan(
            vec![workspace_package("zeta"), workspace_package("alpha")],
            Vec::new(),
        ),
        module_index: module_index(vec![
            workspace_module("zeta", "main"),
            workspace_module("alpha", "b"),
            workspace_module("alpha", "a"),
        ]),
        dependency_overlay: ModuleDependencyOverlay::unavailable(),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect_err("missing overlay diagnostics are sorted");

    assert_eq!(
        diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.module.as_ref().map(module_key))
            .collect::<Vec<_>>(),
        vec![
            Some("alpha:a".to_owned()),
            Some("alpha:b".to_owned()),
            Some("zeta:main".to_owned()),
        ]
    );
}

#[test]
fn unsupported_schema_is_rejected_before_graph_construction() {
    let diagnostics = build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::new(TASK_GRAPH_SCHEMA_VERSION + 1),
        snapshot: snapshot(9),
        build_plan: build_plan(vec![workspace_package("app")], Vec::new()),
        module_index: module_index(vec![workspace_module("app", "main")]),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile::default(),
    })
    .expect_err("unsupported schema is reported");

    assert_eq!(
        diagnostics.diagnostics()[0].kind,
        TaskGraphDiagnosticKind::UnsupportedTaskGraphSchema
    );
}

#[test]
fn cache_driver_ir_and_trust_placeholders_are_absent_from_boundary() {
    let manifest = include_str!("../../Cargo.toml");
    assert!(!manifest.contains("mizar-cache"));
    assert!(!manifest.contains("mizar-driver"));
    assert!(!manifest.contains("mizar-ir"));

    let graph = graph_for_modules(vec![workspace_module("app", "main")], Vec::new());
    let graph_text = format!("{graph:#?}");
    for forbidden in [
        "cachekey",
        "dependencyfingerprint",
        "proofreuse",
        "proofauthority",
        "proofacceptance",
        "trustedstatus",
    ] {
        assert!(
            !graph_text.to_lowercase().contains(forbidden),
            "{forbidden} must not appear in task graph state"
        );
    }

    // Unsupported descriptor families are not expressible in the typed
    // descriptor input; adding one requires extending this public model,
    // not silently carrying an opaque placeholder.
}

fn graph_for_modules(
    modules: Vec<ModuleIndexEntry>,
    descriptors: Vec<VcTaskDescriptor>,
) -> TaskGraph {
    graph_for_modules_with_snapshot(snapshot(0), modules, descriptors)
}

fn graph_for_modules_with_snapshot(
    snapshot: BuildSnapshotId,
    modules: Vec<ModuleIndexEntry>,
    descriptors: Vec<VcTaskDescriptor>,
) -> TaskGraph {
    let packages = modules
        .iter()
        .map(|entry| entry.module.package.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(workspace_package)
        .collect();
    build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot,
        build_plan: build_plan(packages, Vec::new()),
        module_index: module_index(modules),
        dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
        vc_descriptors: descriptors,
        profile: TaskGraphProfile::default(),
    })
    .expect("task graph builds")
}

fn build_plan(packages: Vec<PackagePlan>, edges: Vec<DependencyEdge>) -> BuildPlan {
    BuildPlan {
        workspace_root: WorkspaceRoot::new("."),
        packages,
        dependency_graph: DependencyGraph { edges },
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

fn workspace_package(package_id: &str) -> PackagePlan {
    PackagePlan {
        package_id: PackageId::new(package_id),
        version: Version::new(1, 0, 0),
        source: PackagePlanSource::Workspace {
            root: package_id.to_owned(),
            source_root: format!("{package_id}/src"),
            manifest_path: format!("{package_id}/mizar.pkg"),
        },
        edition: Edition::new("2025"),
        dependencies: Vec::new(),
        verifier_config: VerifierConfig::default(),
        build_config: BuildConfig::default(),
    }
}

fn registry_package(package_id: &str) -> PackagePlan {
    PackagePlan {
        package_id: PackageId::new(package_id),
        version: Version::new(1, 0, 0),
        source: PackagePlanSource::Registry {
            registry: "default".to_owned(),
            checksum: format!("sha256:{package_id}"),
        },
        edition: Edition::new("2025"),
        dependencies: Vec::new(),
        verifier_config: VerifierConfig::default(),
        build_config: BuildConfig::default(),
    }
}

fn module_index(modules: Vec<ModuleIndexEntry>) -> ModuleIndex {
    let mut package_has_workspace = BTreeMap::new();
    for entry in &modules {
        let has_workspace = matches!(entry.location, ModuleIndexLocation::WorkspaceFile { .. });
        package_has_workspace
            .entry(entry.module.package.as_str().to_owned())
            .and_modify(|current| *current |= has_workspace)
            .or_insert(has_workspace);
    }

    let packages = package_has_workspace
        .into_iter()
        .map(|(package_id, has_workspace)| PackageIndexEntry {
            package_id: PackageId::new(package_id.clone()),
            version: Version::new(1, 0, 0),
            edition: Edition::new("2025"),
            source: if has_workspace {
                PackageIndexSource::Workspace {
                    package_root: package_id.to_owned(),
                    source_root: format!("{package_id}/src"),
                    manifest_path: format!("{package_id}/mizar.pkg"),
                }
            } else {
                PackageIndexSource::RegistryArtifact {
                    registry: "default".to_owned(),
                    checksum: format!("sha256:{package_id}"),
                }
            },
            dependencies: Vec::new(),
        })
        .collect();
    ModuleIndex {
        packages,
        namespace_bindings: Vec::new(),
        modules,
        dependency_summaries: Vec::new(),
    }
}

fn workspace_module(package_id: &str, module_path: &str) -> ModuleIndexEntry {
    let module = module_id(package_id, module_path);
    ModuleIndexEntry {
        module: module.clone(),
        package_id: module.package.clone(),
        module_path: module.path.clone(),
        location: ModuleIndexLocation::WorkspaceFile {
            source_root: format!("{package_id}/src"),
            normalized_path: format!("{package_id}/src/{module_path}.miz"),
            source_relative_path: format!("{module_path}.miz"),
        },
        edition: Edition::new("2025"),
    }
}

fn dependency_summary_module(package_id: &str, module_path: &str) -> ModuleIndexEntry {
    let module = module_id(package_id, module_path);
    ModuleIndexEntry {
        module: module.clone(),
        package_id: module.package.clone(),
        module_path: module.path.clone(),
        location: ModuleIndexLocation::DependencySummary {
            artifact: format!("{package_id}-{module_path}.summary"),
            content_hash: Hash::from_bytes([1; Hash::BYTE_LEN]),
        },
        edition: Edition::new("2025"),
    }
}

fn module_id(package_id: &str, module_path: &str) -> ModuleId {
    ModuleId::new(PackageId::new(package_id), ModulePath::new(module_path))
}

fn vc_descriptor(
    id: &str,
    module: ModuleId,
    order_key: &str,
    backend_profiles: Vec<&str>,
    evidence_candidates: Vec<&str>,
) -> VcTaskDescriptor {
    VcTaskDescriptor::new(
        VcTaskDescriptorId::new(id),
        module,
        VcOrderKey::new(order_key),
        backend_profiles
            .into_iter()
            .map(BackendProfileId::new)
            .collect(),
        evidence_candidates
            .into_iter()
            .map(EvidenceCandidateId::new)
            .collect(),
    )
}

fn snapshot(seed: u8) -> BuildSnapshotId {
    let hex = format!("{seed:02x}").repeat(Hash::BYTE_LEN);
    BuildSnapshotId::from_published_schema_str(&format!("mizar-session-build-snapshot-v1:{hex}"))
        .expect("valid snapshot id")
}

fn task_ids(graph: &TaskGraph) -> Vec<String> {
    graph
        .tasks
        .iter()
        .map(|task| task.id.as_str().to_owned())
        .collect()
}

fn module_task_kinds(graph: &TaskGraph, package_id: &str, module_path: &str) -> Vec<TaskKind> {
    graph
        .tasks
        .iter()
        .filter(|task| task_module_matches(task, package_id, module_path))
        .map(|task| task.kind)
        .collect()
}

fn module_task<'a>(
    graph: &'a TaskGraph,
    kind: TaskKind,
    package_id: &str,
    module_path: &str,
) -> &'a BuildTask {
    graph
        .tasks
        .iter()
        .find(|task| task.kind == kind && task_module_matches(task, package_id, module_path))
        .expect("module task exists")
}

fn task_module_matches(task: &BuildTask, package_id: &str, module_path: &str) -> bool {
    module_for_unit(&task.unit).is_some_and(|module| {
        module.package.as_str() == package_id && module.path.as_str() == module_path
    })
}

fn assert_has_edge(graph: &TaskGraph, dependent: &BuildTask, dependency: &BuildTask) {
    assert!(
        graph
            .edges
            .iter()
            .any(|edge| { edge.dependent == dependent.id && edge.dependency == dependency.id }),
        "expected {} to depend on {}",
        dependent.id.as_str(),
        dependency.id.as_str()
    );
}

fn assert_dependencies_match_edges(graph: &TaskGraph) {
    let mut dependencies_by_task: BTreeMap<TaskId, Vec<TaskId>> = BTreeMap::new();
    for edge in &graph.edges {
        dependencies_by_task
            .entry(edge.dependent.clone())
            .or_default()
            .push(edge.dependency.clone());
    }
    for dependencies in dependencies_by_task.values_mut() {
        dependencies.sort();
        dependencies.dedup();
    }
    for task in &graph.tasks {
        let expected = dependencies_by_task.remove(&task.id).unwrap_or_default();
        assert_eq!(task.dependencies, expected, "{}", task.id.as_str());
    }
}

fn assert_edges_are_sorted(graph: &TaskGraph) {
    let edges = graph
        .edges
        .iter()
        .map(|edge| {
            (
                edge.dependent.as_str().to_owned(),
                edge.dependency.as_str().to_owned(),
            )
        })
        .collect::<Vec<_>>();
    let mut sorted_edges = edges.clone();
    sorted_edges.sort();
    assert_eq!(edges, sorted_edges);
}

fn assert_task_metadata(
    task: &BuildTask,
    dependency_coverage: DependencyCoverage,
    resource_class: ResourceClass,
    priority_class: PriorityClass,
) {
    assert_eq!(task.dependency_coverage, dependency_coverage);
    assert_eq!(task.resource_class, resource_class);
    assert_eq!(task.priority_class, priority_class);
    assert!(!task.phases.is_empty());
}

fn single_task(graph: &TaskGraph, kind: TaskKind) -> &BuildTask {
    let mut tasks = graph.tasks.iter().filter(|task| task.kind == kind);
    let task = tasks.next().expect("task exists");
    assert!(tasks.next().is_none(), "expected only one {kind:?} task");
    task
}

fn module_label(task: &BuildTask) -> String {
    let module = module_for_unit(&task.unit).expect("module unit");
    format!("{}:{}", module.package.as_str(), module.path.as_str())
}

fn descriptor_label(task: &BuildTask) -> &str {
    match &task.unit {
        WorkUnit::Vc { descriptor, .. }
        | WorkUnit::BackendAttempt { descriptor, .. }
        | WorkUnit::EvidenceCandidate { descriptor, .. } => descriptor.as_str(),
        WorkUnit::Workspace | WorkUnit::Package { .. } | WorkUnit::Module { .. } => {
            panic!("expected descriptor work unit")
        }
    }
}

fn backend_label(task: &BuildTask) -> &str {
    let WorkUnit::BackendAttempt {
        backend_profile, ..
    } = &task.unit
    else {
        panic!("expected backend work unit");
    };
    backend_profile.as_str()
}

fn evidence_label(task: &BuildTask) -> &str {
    let WorkUnit::EvidenceCandidate {
        evidence_candidate, ..
    } = &task.unit
    else {
        panic!("expected evidence work unit");
    };
    evidence_candidate.as_str()
}

fn package_label(task: &BuildTask) -> &str {
    let WorkUnit::Package { package_id } = &task.unit else {
        panic!("expected package work unit");
    };
    package_id.as_str()
}

fn contains_forbidden_authority_term(value: &str) -> bool {
    let lower = value.to_lowercase();
    [
        "cachekey",
        "dependencyfingerprint",
        "proofreuse",
        "proofauthority",
        "proofacceptance",
        "trustedstatus",
    ]
    .iter()
    .any(|forbidden| lower.contains(forbidden))
}
