use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use mizar_artifact::{
    manifest::{ArtifactManifest, ManifestProvenance, ModuleArtifactEntry, PackageIdentity},
    module_summary::ModuleSummaryIdentity,
    registration_summary::{ArtifactHashClass, ArtifactHashRef},
    store::{PublishedArtifactPath, SchemaVersion, artifact_hash_domain, write_published_artifact},
    verified_artifact::{
        BuildProvenance, VERIFIED_ARTIFACT_SCHEMA_FAMILY, VerifiedArtifact,
        artifact_hash_excluded_paths, current_schema_version as verified_schema_version,
        verified_artifact_json,
    },
};
use mizar_build::{
    artifact_commit::{
        ManifestCommitRequest, ManifestCommitSummary, ScheduledManifestUpdate,
        commit_manifest_updates,
    },
    cache_seam::{
        CacheFallbackReason, CacheOutputRef, CacheSchedulingOutcome, CacheSchedulingPlan,
        CacheTaskDecision, ValidatedCacheHit,
    },
    module_index::{
        ModuleId, ModuleIndex, StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage,
        build_module_index,
    },
    planner::{
        BuildPlan, DependencySelection, Lockfile, PlanRequest, WorkspacePackage, parse_lockfile,
        parse_package_manifest, produce_build_plan,
    },
    scheduler::{
        CacheSchedulingPolicy, CompletionOrder, PriorityHints, SchedulerInput, SchedulerRun,
        SyntheticOutputRef, SyntheticTaskOutcome, TaskState, run_scheduler,
    },
    task_graph::{
        BuildTask, DocumentationProfile, ModuleDependencyEdge, ModuleDependencyKind,
        ModuleDependencyOverlay, TaskGraph, TaskGraphInput, TaskGraphProfile, TaskGraphVersion,
        TaskId, TaskKind, VcDescriptorPolicy, WorkUnit, build_task_graph,
    },
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    SnapshotInput, SnapshotRegistry, ToolchainInfo, WorkspaceRoot,
};

static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(1);

#[test]
fn shuffled_inputs_preserve_plan_index_and_graph_projection() {
    let canonical = planning_projection(workspace_packages_canonical(), source_layout_canonical());
    let shuffled = planning_projection(workspace_packages_shuffled(), source_layout_shuffled());

    assert_eq!(canonical.plan, shuffled.plan);
    assert_eq!(canonical.index, shuffled.index);
    assert_eq!(canonical.graph, shuffled.graph);
}

#[test]
fn scheduler_projection_is_identical_across_workers_priorities_and_completion_order() {
    let fixture = build_fixture(workspace_packages_canonical(), source_layout_canonical());
    let source_load_ids = task_ids(&fixture.graph, TaskKind::SourceLoad);
    assert_eq!(source_load_ids.len(), 3);

    let reference = scheduler_projection(schedule_clean(
        fixture.graph.clone(),
        PriorityHints::default(),
        1,
        CompletionOrder::Canonical,
    ));
    let hinted = scheduler_projection(schedule_clean(
        fixture.graph.clone(),
        PriorityHints {
            preferred_tasks: source_load_ids.into_iter().rev().collect(),
        },
        4,
        CompletionOrder::Reverse,
    ));

    assert_eq!(reference, hinted);
}

#[test]
fn cache_hit_and_miss_timing_preserve_commit_projection() {
    let fixture = build_fixture(workspace_packages_canonical(), source_layout_canonical());
    let cached_frontend = module_task(&fixture.graph, TaskKind::Frontend, "util");
    let reference = schedule_clean(
        fixture.graph.clone(),
        PriorityHints::default(),
        1,
        CompletionOrder::Canonical,
    );
    let cache_hit = schedule_with_cache(
        fixture.graph.clone(),
        CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
            cached_frontend.id.clone(),
            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                vec![CacheOutputRef::new(
                    format!(
                        "frontend:{}",
                        module_for_task(cached_frontend).path.as_str()
                    ),
                    "synthetic-frontend-output",
                )],
                Vec::new(),
            )),
        )]),
    );
    let cache_miss = schedule_with_cache(
        fixture.graph.clone(),
        CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
            cached_frontend.id.clone(),
            CacheSchedulingOutcome::Miss(CacheFallbackReason::Miss),
        )]),
    );

    let reference_frontend = reference
        .results
        .iter()
        .find(|result| result.task_id == cached_frontend.id)
        .expect("reference frontend result");
    let cache_hit_frontend = cache_hit
        .results
        .iter()
        .find(|result| result.task_id == cached_frontend.id)
        .expect("cache hit result");
    assert_eq!(cache_hit_frontend.state, TaskState::CacheHit);
    assert_eq!(
        cache_hit_frontend.output_refs,
        reference_frontend.output_refs
    );
    assert_eq!(
        cache_hit_frontend.diagnostics,
        reference_frontend.diagnostics
    );
    assert_eq!(
        committed_projection(&fixture.graph, &reference),
        committed_projection(&fixture.graph, &cache_hit)
    );
    assert_eq!(
        committed_projection(&fixture.graph, &reference),
        committed_projection(&fixture.graph, &cache_miss)
    );
}

#[test]
fn shuffled_manifest_updates_commit_identical_records() {
    let fixture = build_fixture(workspace_packages_canonical(), source_layout_canonical());
    let run = schedule_clean(
        fixture.graph.clone(),
        PriorityHints::default(),
        3,
        CompletionOrder::Reverse,
    );
    let canonical = commit_projection(&fixture.graph, &run, ManifestArrival::Canonical);
    let reversed = commit_projection(&fixture.graph, &run, ManifestArrival::Reverse);

    assert_eq!(canonical.manifest_hash, reversed.manifest_hash);
    assert_eq!(canonical.modules, reversed.modules);
    assert_eq!(canonical.module_paths, reversed.module_paths);
    assert_eq!(canonical.artifact_files, reversed.artifact_files);
}

#[test]
fn determinism_suite_records_external_gaps_without_placeholder_boundaries() {
    let manifest = include_str!("../Cargo.toml");
    for forbidden_dependency in [concat!("mizar", "-", "driver"), concat!("mizar", "-", "ir")] {
        assert!(
            !manifest.contains(forbidden_dependency),
            "determinism suite must not add `{forbidden_dependency}` dependency"
        );
    }

    let source = include_str!("determinism_suite.rs");
    for forbidden in [
        concat!("mizar", "_", "driver"),
        concat!("mizar", "-", "driver"),
        concat!("mizar", "_", "ir"),
        concat!("mizar", "-", "ir"),
        concat!("Driver", "Session"),
        concat!("Driver", "Request"),
        concat!("Ir", "Snapshot", "Handle"),
        concat!("Cache", "Key"),
        concat!("Dependency", "Fingerprint"),
        concat!("dependency", "_", "fingerprint"),
        concat!("Proof", "Reuse"),
        concat!("proof", "_", "reuse"),
        concat!("Publication", "Token"),
        concat!("publication", "_", "token"),
        concat!("Proof", "Authority"),
        concat!("Trusted", "Status"),
        concat!("trusted", "_", "status"),
    ] {
        assert!(
            !source.contains(forbidden),
            "determinism suite must not contain `{forbidden}` placeholder"
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlanningProjection {
    plan: BuildPlan,
    index: ModuleIndex,
    graph: TaskGraph,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommitProjection {
    manifest_hash: Hash,
    modules: Vec<(String, String, String)>,
    module_paths: Vec<String>,
    artifact_files: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
enum ManifestArrival {
    Canonical,
    Reverse,
}

fn planning_projection(
    packages: Vec<WorkspacePackage>,
    layout: StaticSourceLayout,
) -> PlanningProjection {
    let plan = build_plan(packages);
    let index = build_source_index(&plan, layout);
    let graph = build_graph(plan.clone(), index.clone());
    PlanningProjection { plan, index, graph }
}

fn build_fixture(
    packages: Vec<WorkspacePackage>,
    layout: StaticSourceLayout,
) -> PlanningProjection {
    planning_projection(packages, layout)
}

fn workspace_packages_canonical() -> Vec<WorkspacePackage> {
    vec![
        workspace_package(
            "core",
            r#"
            [package]
            name = "core"
            version = "1.0.0"
            edition = "2025"
            "#,
        ),
        workspace_package(
            "app",
            r#"
            [package]
            name = "app"
            version = "1.0.0"
            edition = "2025"

            [dependencies]
            core = "1.0.0"
            "#,
        ),
    ]
}

fn workspace_packages_shuffled() -> Vec<WorkspacePackage> {
    vec![
        workspace_package(
            "app",
            r#"
            [dependencies]
            core = "1.0.0"

            [package]
            edition = "2025"
            version = "1.0.0"
            name = "app"
            "#,
        ),
        workspace_package(
            "core",
            r#"
            [package]
            edition = "2025"
            version = "1.0.0"
            name = "core"
            "#,
        ),
    ]
}

fn workspace_package(member_path: &str, manifest: &str) -> WorkspacePackage {
    WorkspacePackage {
        member_path: member_path.to_owned(),
        manifest: parse_package_manifest(manifest).expect("valid package manifest"),
    }
}

fn build_plan(packages: Vec<WorkspacePackage>) -> BuildPlan {
    produce_build_plan(
        PlanRequest {
            workspace_root: WorkspaceRoot::new("workspace"),
            dependency_selection: DependencySelection::Normal,
            toolchain: ToolchainInfo::new("mizar-evo-test"),
        },
        packages,
        lockfile(),
    )
    .expect("valid build plan")
}

fn lockfile() -> Lockfile {
    parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "core"
        version = "1.0.0"
        source = { kind = "workspace", path = "core" }
        dependencies = []

        [[package]]
        name = "app"
        version = "1.0.0"
        source = { kind = "workspace", path = "app" }
        dependencies = [{ name = "core", version = "1.0.0" }]
        "#,
    )
    .expect("valid lockfile")
}

fn source_layout_canonical() -> StaticSourceLayout {
    StaticSourceLayout::new(vec![
        WorkspaceSourcePackage {
            package_id: PackageId::new("core"),
            files: vec![WorkspaceSourceFile::new("src/base.miz", "base.miz")],
        },
        WorkspaceSourcePackage {
            package_id: PackageId::new("app"),
            files: vec![
                WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                WorkspaceSourceFile::new("src/util.miz", "util.miz"),
            ],
        },
    ])
}

fn source_layout_shuffled() -> StaticSourceLayout {
    StaticSourceLayout::new(vec![
        WorkspaceSourcePackage {
            package_id: PackageId::new("app"),
            files: vec![
                WorkspaceSourceFile::new("src/util.miz", "util.miz"),
                WorkspaceSourceFile::new("src/main.miz", "main.miz"),
            ],
        },
        WorkspaceSourcePackage {
            package_id: PackageId::new("core"),
            files: vec![WorkspaceSourceFile::new("src/base.miz", "base.miz")],
        },
    ])
}

fn build_source_index(plan: &BuildPlan, layout: StaticSourceLayout) -> ModuleIndex {
    build_module_index(plan, &layout, &[]).expect("valid module index")
}

fn build_graph(plan: BuildPlan, index: ModuleIndex) -> TaskGraph {
    build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot_id(),
        build_plan: plan,
        module_index: index,
        dependency_overlay: ModuleDependencyOverlay::complete(vec![
            ModuleDependencyEdge::new(
                module_id("app", "main"),
                module_id("app", "util"),
                ModuleDependencyKind::ImportSummary,
            ),
            ModuleDependencyEdge::new(
                module_id("app", "util"),
                module_id("core", "base"),
                ModuleDependencyKind::ImportSummary,
            ),
        ]),
        vc_descriptors: Vec::new(),
        profile: TaskGraphProfile {
            documentation: DocumentationProfile::Disabled,
            vc_descriptor_policy: VcDescriptorPolicy::Optional,
        },
    })
    .expect("valid task graph")
}

fn snapshot_id() -> BuildSnapshotId {
    let registry = SnapshotRegistry::new();
    let allocator = InMemorySessionIdAllocator::new();
    let request = allocator.next_request_id().expect("request id");
    let (snapshot, _lease) = registry
        .create_snapshot(
            request,
            SnapshotInput {
                workspace_root: WorkspaceRoot::new("workspace"),
                source_versions: Vec::new(),
                dependency_artifacts: Vec::new(),
                lockfile_hash: hash(10),
                toolchain: ToolchainInfo::new("mizar-evo-test"),
                verifier_config_hash: hash(11),
            },
        )
        .expect("valid snapshot");
    snapshot.id
}

fn schedule_clean(
    graph: TaskGraph,
    priority_hints: PriorityHints,
    worker_count: usize,
    completion_order: CompletionOrder,
) -> SchedulerRun {
    run_scheduler(SchedulerInput {
        priority_hints,
        task_outcomes: frontend_outcomes(&graph),
        worker_count,
        completion_order,
        ..SchedulerInput::new(graph)
    })
    .expect("clean scheduler run")
}

fn schedule_with_cache(graph: TaskGraph, cache_decisions: CacheSchedulingPlan) -> SchedulerRun {
    run_scheduler(SchedulerInput {
        cache: CacheSchedulingPolicy::Enabled,
        cache_decisions,
        task_outcomes: frontend_outcomes(&graph),
        worker_count: 4,
        completion_order: CompletionOrder::Reverse,
        ..SchedulerInput::new(graph)
    })
    .expect("cached scheduler run")
}

fn scheduler_projection(run: SchedulerRun) -> (Vec<_TaskResult>, Vec<_TaskEvent>) {
    (
        run.results
            .into_iter()
            .map(|result| _TaskResult {
                task_id: result.task_id.as_str().to_owned(),
                state: format!("{:?}", result.state),
                outputs: result
                    .output_refs
                    .into_iter()
                    .map(|output| (output.identity, output.content))
                    .collect(),
                diagnostics: result
                    .diagnostics
                    .into_iter()
                    .map(|diagnostic| (diagnostic.source, diagnostic.code, diagnostic.message))
                    .collect(),
            })
            .collect(),
        run.events
            .into_iter()
            .map(|event| _TaskEvent {
                kind: format!("{:?}", event.kind),
                task_id: event.task_id.map(|task_id| task_id.as_str().to_owned()),
                graph_index: event.order.graph_index,
                lifecycle_rank: event.order.lifecycle_rank,
            })
            .collect(),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct _TaskResult {
    task_id: String,
    state: String,
    outputs: Vec<(String, String)>,
    diagnostics: Vec<(String, String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct _TaskEvent {
    kind: String,
    task_id: Option<String>,
    graph_index: usize,
    lifecycle_rank: usize,
}

fn committed_projection(graph: &TaskGraph, run: &SchedulerRun) -> CommitProjection {
    commit_projection(graph, run, ManifestArrival::Reverse)
}

fn commit_projection(
    graph: &TaskGraph,
    run: &SchedulerRun,
    arrival: ManifestArrival,
) -> CommitProjection {
    let root = TestArtifactRoot::new();
    let commit = commit_manifest_updates(
        ManifestCommitRequest::new(
            root.path(),
            seed_manifest(),
            manifest_updates(root.path(), graph, run, arrival),
        ),
        None,
    )
    .expect("manifest commit succeeds");
    commit_projection_from_summary(commit)
}

fn commit_projection_from_summary(summary: ManifestCommitSummary) -> CommitProjection {
    CommitProjection {
        manifest_hash: summary.manifest_hash,
        modules: summary
            .modules
            .iter()
            .map(|module| {
                (
                    module.task_id.as_str().to_owned(),
                    module.module.module_path.clone(),
                    module.artifact_file.clone(),
                )
            })
            .collect(),
        module_paths: summary
            .manifest
            .modules
            .iter()
            .map(|module| module.module.module_path.clone())
            .collect(),
        artifact_files: summary
            .manifest
            .modules
            .iter()
            .map(|module| module.artifact_file.clone())
            .collect(),
    }
}

fn manifest_updates(
    root: &Path,
    graph: &TaskGraph,
    run: &SchedulerRun,
    arrival: ManifestArrival,
) -> Vec<ScheduledManifestUpdate> {
    let graph_index_by_task = graph
        .tasks
        .iter()
        .enumerate()
        .map(|(index, task)| (task.id.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let completed = run
        .results
        .iter()
        .filter(|result| result.state == TaskState::Completed)
        .map(|result| result.task_id.clone())
        .collect::<BTreeSet<_>>();
    let mut updates = graph
        .tasks
        .iter()
        .filter(|task| task.kind == TaskKind::ArtifactCommit)
        .filter(|task| completed.contains(&task.id))
        .map(|task| {
            let module = module_for_task(task);
            ScheduledManifestUpdate::new(
                task.id.clone(),
                *graph_index_by_task.get(&task.id).expect("task graph index"),
                publish_verified_entry(root, module),
            )
        })
        .collect::<Vec<_>>();
    if matches!(arrival, ManifestArrival::Reverse) {
        updates.reverse();
    }
    updates
}

fn frontend_outcomes(graph: &TaskGraph) -> Vec<SyntheticTaskOutcome> {
    graph
        .tasks
        .iter()
        .filter(|task| task.kind == TaskKind::Frontend)
        .map(|task| {
            let module = module_for_task(task);
            SyntheticTaskOutcome::complete(
                task.id.clone(),
                vec![SyntheticOutputRef::new(
                    format!("frontend:{}", module.path.as_str()),
                    "synthetic-frontend-output",
                )],
            )
        })
        .collect()
}

fn task_ids(graph: &TaskGraph, kind: TaskKind) -> Vec<TaskId> {
    graph
        .tasks
        .iter()
        .filter(|task| task.kind == kind)
        .map(|task| task.id.clone())
        .collect()
}

fn module_task<'a>(graph: &'a TaskGraph, kind: TaskKind, module_path: &str) -> &'a BuildTask {
    graph
        .tasks
        .iter()
        .find(|task| {
            task.kind == kind
                && matches!(
                    &task.unit,
                    WorkUnit::Module { module } if module.path.as_str() == module_path
                )
        })
        .expect("module task exists")
}

fn module_for_task(task: &BuildTask) -> &ModuleId {
    let WorkUnit::Module { module } = &task.unit else {
        panic!("expected module task");
    };
    module
}

fn publish_verified_entry(root: &Path, module: &ModuleId) -> ModuleArtifactEntry {
    let seed = match (module.package.as_str(), module.path.as_str()) {
        ("app", "main") => 21,
        ("app", "util") => 22,
        ("core", "base") => 23,
        _ => 24,
    };
    let identity = ModuleSummaryIdentity {
        package_id: module.package.as_str().to_owned(),
        package_version: Some("1.0.0".to_owned()),
        lockfile_identity: Some("lock".to_owned()),
        module_path: module.path.as_str().to_owned(),
        language_edition: "2025".to_owned(),
    };
    let artifact = sample_verified_artifact(
        identity,
        &format!(
            "{}/src/{}.miz",
            module.package.as_str(),
            module.path.as_str()
        ),
        seed,
    );
    let artifact_path = format!(
        "artifacts/{}/{}.mizir.json",
        module.package.as_str(),
        module.path.as_str()
    );
    publish_verified_artifact(root, &artifact_path, &artifact)
}

fn sample_verified_artifact(
    module: ModuleSummaryIdentity,
    source_file: &str,
    seed: u8,
) -> VerifiedArtifact {
    let mut artifact = VerifiedArtifact {
        schema_version: verified_schema_version(),
        module,
        source_file: source_file.to_owned(),
        source_hash: hash(seed),
        verified_at: None,
        interface_hash: hash(0),
        implementation_hash: hash(0),
        exports: Vec::new(),
        expressions: Vec::new(),
        obligations: Vec::new(),
        proof_witnesses: Vec::new(),
        diagnostics: Vec::new(),
        provenance: BuildProvenance {
            toolchain: "mizar-evo-test".to_owned(),
            language_edition: "2025".to_owned(),
            lockfile_hash: hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 1),
            verifier_config_hash: hash_ref(
                ArtifactHashClass::Interface,
                "mizar-build/verifier-config",
                2,
            ),
            dependency_artifact_hashes: Vec::new(),
            cache_key: None,
        },
    };
    artifact.refresh_hashes().expect("sample hashes");
    artifact
}

fn publish_verified_artifact(
    root: &Path,
    path: &str,
    artifact: &VerifiedArtifact,
) -> ModuleArtifactEntry {
    let json = verified_artifact_json(artifact).expect("verified artifact JSON");
    let domain = artifact_hash_domain(VERIFIED_ARTIFACT_SCHEMA_FAMILY, artifact.schema_version);
    let excluded = artifact_hash_excluded_paths();
    let published_path = PublishedArtifactPath::new(path).expect("published path");
    let write = write_published_artifact(root, &published_path, &json, &domain, &excluded)
        .expect("write verified artifact");
    ModuleArtifactEntry {
        module: artifact.module.clone(),
        source_file: artifact.source_file.clone(),
        source_hash: artifact.source_hash,
        artifact_file: path.to_owned(),
        artifact_hash: ArtifactHashRef::new(
            ArtifactHashClass::Artifact,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            artifact.schema_version,
            write.artifact_hash,
        ),
        interface_hash: ArtifactHashRef::new(
            ArtifactHashClass::Interface,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            artifact.schema_version,
            artifact.interface_hash,
        ),
        implementation_hash: ArtifactHashRef::new(
            ArtifactHashClass::Implementation,
            VERIFIED_ARTIFACT_SCHEMA_FAMILY,
            artifact.schema_version,
            artifact.implementation_hash,
        ),
        module_summary_file: None,
        module_summary_hash: None,
        module_summary_interface_hash: None,
        registration_summary_file: None,
        registration_summary_hash: None,
        registration_interface_hash: None,
        proof_witnesses: Vec::new(),
        diagnostics_hash: None,
    }
}

fn seed_manifest() -> ArtifactManifest {
    ArtifactManifest {
        schema_version: mizar_artifact::manifest::current_schema_version(),
        package: PackageIdentity {
            package_id: "workspace".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("lock".to_owned()),
        },
        artifact_root: "build".to_owned(),
        lockfile_hash: hash_ref(ArtifactHashClass::Artifact, "mizar-build/lockfile", 1),
        toolchain: "mizar-evo-test".to_owned(),
        language_edition: "2025".to_owned(),
        verifier_config_hash: hash_ref(
            ArtifactHashClass::Interface,
            "mizar-build/verifier-config",
            2,
        ),
        modules: Vec::new(),
        development_artifacts: Vec::new(),
        provenance: ManifestProvenance {
            generated_by: "mizar-build-determinism-suite-test".to_owned(),
            manifest_policy: "test-policy".to_owned(),
            transaction_format: "mizar-build-determinism-suite-test-v1".to_owned(),
        },
    }
}

fn module_id(package_id: &str, module_path: &str) -> ModuleId {
    ModuleId::new(PackageId::new(package_id), ModulePath::new(module_path))
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn hash_ref(class: ArtifactHashClass, schema_family: &str, seed: u8) -> ArtifactHashRef {
    ArtifactHashRef::new(class, schema_family, SchemaVersion::new(1, 0), hash(seed))
}

struct TestArtifactRoot {
    path: PathBuf,
}

impl TestArtifactRoot {
    fn new() -> Self {
        let path = Self::fresh_path();
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("test artifact root");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn fresh_path() -> PathBuf {
        let counter = TEST_ROOT_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "mizar-build-determinism-suite-test-{}-{counter}",
            std::process::id()
        ))
    }
}

impl Drop for TestArtifactRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
