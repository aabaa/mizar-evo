use std::{
    collections::BTreeMap,
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
    artifact_commit::{ManifestCommitRequest, ScheduledManifestUpdate, commit_manifest_updates},
    cache_seam::{
        CacheOutputRef, CacheSchedulingOutcome, CacheSchedulingPlan, CacheTaskDecision,
        ValidatedCacheHit,
    },
    module_index::{
        ModuleId, StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage,
        build_module_index,
    },
    planner::{
        DependencySelection, Lockfile, PlanRequest, WorkspacePackage, parse_lockfile,
        parse_package_manifest, produce_build_plan,
    },
    scheduler::{
        CacheSchedulingPolicy, CompletionOrder, SchedulerInput, SchedulerRun, SyntheticOutputRef,
        SyntheticTaskOutcome, TaskState, run_scheduler,
    },
    task_graph::{
        BuildTask, DocumentationProfile, ModuleDependencyEdge, ModuleDependencyKind,
        ModuleDependencyOverlay, PipelinePhase, TaskGraph, TaskGraphInput, TaskGraphProfile,
        TaskGraphVersion, TaskId, TaskKind, VcDescriptorPolicy, WorkUnit, build_task_graph,
    },
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    SnapshotInput, SnapshotRegistry, ToolchainInfo, WorkspaceRoot,
};

static TEST_ROOT_COUNTER: AtomicU64 = AtomicU64::new(1);

#[test]
fn batch_path_runs_plan_graph_schedule_and_commit_without_external_boundaries() {
    let plan = build_plan();
    assert_eq!(
        plan.packages
            .iter()
            .map(|package| package.package_id.as_str())
            .collect::<Vec<_>>(),
        vec!["app"]
    );

    let index = build_source_index(&plan);
    assert_eq!(
        index
            .modules
            .iter()
            .map(|entry| entry.module.path.as_str())
            .collect::<Vec<_>>(),
        vec!["main", "util"]
    );

    let graph = build_batch_graph(plan, index);
    assert!(graph.diagnostics.is_empty());
    assert_eq!(module_task_ids(&graph, TaskKind::ArtifactCommit).len(), 2);
    assert!(
        graph
            .tasks
            .iter()
            .any(|task| task.phases == vec![PipelinePhase::Frontend])
    );

    let frontend_outcomes = frontend_outcomes(&graph);
    let serial = schedule(
        graph.clone(),
        frontend_outcomes.clone(),
        1,
        CompletionOrder::Canonical,
    );
    let parallel = schedule(
        graph.clone(),
        frontend_outcomes,
        4,
        CompletionOrder::Reverse,
    );
    assert!(serial.diagnostics.is_empty());
    assert!(parallel.diagnostics.is_empty());
    assert_eq!(serial.results, parallel.results);
    assert!(serial.failure_records.is_empty());
    assert!(serial.blocked_records.is_empty());
    assert!(serial.results.iter().all(|result| {
        matches!(result.state, TaskState::Completed)
            || result.task_id.as_str().starts_with("task:package-resolve")
    }));
    assert_eq!(
        completed_task_ids(&serial),
        graph
            .tasks
            .iter()
            .map(|task| task.id.as_str().to_owned())
            .collect::<Vec<_>>()
    );

    let root = TestArtifactRoot::new();
    let updates = shuffled_manifest_updates(root.path(), &graph, &serial);
    let commit = commit_manifest_updates(
        ManifestCommitRequest::new(root.path(), seed_manifest(), updates),
        None,
    )
    .expect("batch manifest commit succeeds");

    assert_eq!(
        commit
            .modules
            .iter()
            .map(|module| module.module.module_path.as_str())
            .collect::<Vec<_>>(),
        vec!["main", "util"]
    );
    assert_eq!(
        commit
            .manifest
            .modules
            .iter()
            .map(|module| module.module.module_path.as_str())
            .collect::<Vec<_>>(),
        vec!["main", "util"]
    );
    assert!(commit.modules.iter().all(|module| {
        serial
            .results
            .iter()
            .any(|result| result.task_id == module.task_id && result.state == TaskState::Completed)
    }));
}

#[test]
fn validated_cache_hit_does_not_become_commit_or_proof_authority() {
    let plan = build_plan();
    let index = build_source_index(&plan);
    let graph = build_batch_graph(plan, index);
    let cached_frontend = module_task(&graph, TaskKind::Frontend, "util");
    let cache_decisions = CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
        cached_frontend.id.clone(),
        CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
            vec![CacheOutputRef::new(
                "cache:frontend:util",
                "cached-frontend-output",
            )],
            Vec::new(),
        )),
    )]);

    let cached = run_scheduler(SchedulerInput {
        cache: CacheSchedulingPolicy::Enabled,
        cache_decisions,
        task_outcomes: frontend_outcomes(&graph),
        worker_count: 4,
        completion_order: CompletionOrder::Reverse,
        ..SchedulerInput::new(graph.clone())
    })
    .expect("cached batch schedule succeeds");

    let frontend_result = cached
        .results
        .iter()
        .find(|result| result.task_id == cached_frontend.id)
        .expect("cached frontend result");
    assert_eq!(frontend_result.state, TaskState::CacheHit);
    assert_eq!(
        frontend_result.output_refs[0].identity,
        "cache:frontend:util"
    );
    assert!(cached.results.iter().any(|result| {
        result.state == TaskState::Completed
            && graph
                .tasks
                .iter()
                .any(|task| task.id == result.task_id && task.kind == TaskKind::ArtifactCommit)
    }));

    let root = TestArtifactRoot::new();
    let commit = commit_manifest_updates(
        ManifestCommitRequest::new(
            root.path(),
            seed_manifest(),
            shuffled_manifest_updates(root.path(), &graph, &cached),
        ),
        None,
    )
    .expect("cached batch manifest commit succeeds");

    assert!(
        commit
            .modules
            .iter()
            .all(|module| module.task_id != cached_frontend.id)
    );
    assert!(commit.manifest.modules.iter().all(|module| {
        !module.artifact_file.contains("cache")
            && module.proof_witnesses.is_empty()
            && module.module.module_path.as_str() != "cache:frontend:util"
    }));
}

#[test]
fn batch_integration_boundary_records_external_gaps_without_placeholders() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("mizar-artifact"));
    for forbidden_dependency in [concat!("mizar", "-", "driver"), concat!("mizar", "-", "ir")] {
        assert!(
            !manifest.contains(forbidden_dependency),
            "batch integration must not add `{forbidden_dependency}` dependency"
        );
    }

    let source = include_str!("batch_integration.rs");
    for forbidden in [
        concat!("mizar", "_", "driver"),
        concat!("mizar", "-", "driver"),
        concat!("mizar", "_", "ir"),
        concat!("mizar", "-", "ir"),
        concat!("Driver", "Session"),
        concat!("Driver", "Request"),
        concat!("Ir", "Snapshot", "Handle"),
        concat!("Dependency", "Fingerprint"),
        concat!("dependency", "_", "fingerprint"),
        concat!("Proof", "Reuse"),
        concat!("proof", "_", "reuse"),
        concat!("Producer", "Publication", "Token"),
        concat!("Publication", "Token"),
        concat!("publication", "_", "token"),
        concat!("Proof", "Authority"),
        concat!("Trusted", "Status"),
        concat!("trusted", "_", "status"),
    ] {
        assert!(
            !source.contains(forbidden),
            "batch integration must not contain `{forbidden}` placeholder"
        );
    }
}

fn build_plan() -> mizar_build::planner::BuildPlan {
    produce_build_plan(
        PlanRequest {
            workspace_root: WorkspaceRoot::new("workspace"),
            dependency_selection: DependencySelection::Normal,
            toolchain: ToolchainInfo::new("mizar-evo-test"),
        },
        vec![WorkspacePackage {
            member_path: "app".to_owned(),
            manifest: parse_package_manifest(
                r#"
                [package]
                name = "app"
                version = "1.0.0"
                edition = "2025"
                "#,
            )
            .expect("valid package manifest"),
        }],
        lockfile(),
    )
    .expect("valid build plan")
}

fn lockfile() -> Lockfile {
    parse_lockfile(
        r#"
        schema_version = 1

        [[package]]
        name = "app"
        version = "1.0.0"
        source = { kind = "workspace", path = "app" }
        dependencies = []
        "#,
    )
    .expect("valid lockfile")
}

fn build_source_index(
    plan: &mizar_build::planner::BuildPlan,
) -> mizar_build::module_index::ModuleIndex {
    let layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
        package_id: PackageId::new("app"),
        files: vec![
            WorkspaceSourceFile::new("src/util.miz", "util.miz"),
            WorkspaceSourceFile::new("src/main.miz", "main.miz"),
        ],
    }]);
    build_module_index(plan, &layout, &[]).expect("valid module index")
}

fn build_batch_graph(
    plan: mizar_build::planner::BuildPlan,
    index: mizar_build::module_index::ModuleIndex,
) -> TaskGraph {
    let main = module_id("main");
    let util = module_id("util");
    build_task_graph(TaskGraphInput {
        graph_version: TaskGraphVersion::current(),
        snapshot: snapshot_id(),
        build_plan: plan,
        module_index: index,
        dependency_overlay: ModuleDependencyOverlay::complete(vec![ModuleDependencyEdge::new(
            main,
            util,
            ModuleDependencyKind::ImportSummary,
        )]),
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

fn schedule(
    graph: TaskGraph,
    task_outcomes: Vec<SyntheticTaskOutcome>,
    worker_count: usize,
    completion_order: CompletionOrder,
) -> SchedulerRun {
    run_scheduler(SchedulerInput {
        task_outcomes,
        worker_count,
        completion_order,
        ..SchedulerInput::new(graph)
    })
    .expect("batch schedule succeeds")
}

fn shuffled_manifest_updates(
    root: &Path,
    graph: &TaskGraph,
    run: &SchedulerRun,
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
        .collect::<Vec<_>>();
    let completed = completed
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let mut updates = graph
        .tasks
        .iter()
        .filter(|task| task.kind == TaskKind::ArtifactCommit)
        .filter(|task| completed.contains(&task.id))
        .map(|task| {
            let module = module_for_task(task);
            let seed = if module.path.as_str() == "main" {
                21
            } else {
                22
            };
            ScheduledManifestUpdate::new(
                task.id.clone(),
                *graph_index_by_task.get(&task.id).expect("task graph index"),
                publish_verified_entry(root, module, seed),
            )
        })
        .collect::<Vec<_>>();
    updates.reverse();
    updates
}

fn module_task<'a>(graph: &'a TaskGraph, kind: TaskKind, module_path: &str) -> &'a BuildTask {
    graph
        .tasks
        .iter()
        .find(|task| {
            task.kind == kind
                && matches!(
                    &task.unit,
                    WorkUnit::Module { module }
                        if module.package.as_str() == "app" && module.path.as_str() == module_path
                )
        })
        .expect("module task exists")
}

fn publish_verified_entry(root: &Path, module: &ModuleId, seed: u8) -> ModuleArtifactEntry {
    let identity = ModuleSummaryIdentity {
        package_id: module.package.as_str().to_owned(),
        package_version: Some("1.0.0".to_owned()),
        lockfile_identity: Some("lock".to_owned()),
        module_path: module.path.as_str().to_owned(),
        language_edition: "2025".to_owned(),
    };
    let artifact =
        sample_verified_artifact(identity, &format!("src/{}.miz", module.path.as_str()), seed);
    let artifact_path = format!("artifacts/{}.mizir.json", module.path.as_str());
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
            package_id: "app".to_owned(),
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
            generated_by: "mizar-build-batch-integration-test".to_owned(),
            manifest_policy: "test-policy".to_owned(),
            transaction_format: "mizar-build-batch-integration-test-v1".to_owned(),
        },
    }
}

fn completed_task_ids(run: &SchedulerRun) -> Vec<String> {
    run.results
        .iter()
        .map(|result| result.task_id.as_str().to_owned())
        .collect()
}

fn module_task_ids(graph: &TaskGraph, kind: TaskKind) -> Vec<TaskId> {
    graph
        .tasks
        .iter()
        .filter(|task| task.kind == kind)
        .map(|task| task.id.clone())
        .collect()
}

fn module_for_task(task: &BuildTask) -> &ModuleId {
    let WorkUnit::Module { module } = &task.unit else {
        panic!("expected module task");
    };
    module
}

fn module_id(module_path: &str) -> ModuleId {
    ModuleId::new(PackageId::new("app"), ModulePath::new(module_path))
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
            "mizar-build-batch-integration-test-{}-{counter}",
            std::process::id()
        ))
    }
}

impl Drop for TestArtifactRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
