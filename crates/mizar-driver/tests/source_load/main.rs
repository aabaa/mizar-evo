use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use mizar_artifact::{
    module_summary::ModuleSummaryIdentity,
    store::{CanonicalJson, canonical_json_string},
};
use mizar_build::{
    cache_seam::{
        CacheOutputRef, CacheSchedulingOutcome, CacheSchedulingPlan, CacheTaskDecision,
        ValidatedCacheHit,
    },
    cancel::{CancellationGeneration, CancellationReason, CancellationToken},
    module_index::{
        DependencyArtifactIndex, DependencyModuleSummaryRef, ModuleIndex, ModuleIndexLocation,
        StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage, build_module_index,
    },
    planner::{
        BuildPlan, DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile,
        parse_package_manifest, produce_build_plan,
    },
    scheduler::{CacheSchedulingPolicy, CompletionOrder, TaskState},
    task_graph::{
        BuildTask, DependencyCoverage, ModuleDependencyEdge, ModuleDependencyKind,
        ModuleDependencyOverlay, PipelinePhase, TaskGraphDiagnosticKind, TaskKind, WorkUnit,
    },
};
use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetailValue, DiagnosticPrimaryLocation, DiagnosticSpan, FailureCategory,
        PipelinePhase as DiagnosticPhase,
    },
    sink::{DiagnosticProducerScope, DiagnosticSink},
};
use mizar_driver::{
    driver::{
        CompilerDriver, DriverSubmissionStatus, DriverSubmitError, DriverSubmitInput,
        PhaseDispatchInputProvider,
    },
    events::BuildEventKind,
    registry::{
        PhaseCacheIntent, PhaseExecutionResources, PhaseInput, PhaseRegistryBuilder, PhaseStatus,
        SourceLoadInputs,
    },
    request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildSessionOutcome, BuildSessionState,
        BuildTargets, DependencyInputSet, SourceInputSet, VerifierConfigInput,
    },
};
use mizar_frontend::{
    cache_key::{SOURCE_UNIT_CACHE_KEY_VERSION, SourceUnitCacheKey},
    lexical_env::{
        FrontendLexicalEnvironmentError, LexicalEnvironmentRequest, LexicalSummaryProvider,
        build_active_lexical_environment,
    },
    lexing::TokenKind,
    orchestration::FrontendOutput,
    parsing::{MizarParserSeam, ParserSeam},
    preprocess::preprocess,
    source::SourceUnit,
    source::register_source_unit,
    span_bridge::SpanBridge,
};
use mizar_ir::{
    dispatch_input::{
        DispatchInputError, PhaseDispatchInputBundle, PhaseDispatchInputRequest,
        SealedParentOutputHandle,
    },
    identity::{
        OutputKind, PipelinePhase as IrPipelinePhase, SnapshotHandleRegistry,
        WorkUnit as IrWorkUnit,
    },
    publisher::{AllowedWorkUnit, PhaseOutputPublisher},
    storage::{AnyPhaseOutputRef, IrStorageService, StoragePlacement, StoragePolicy},
};
use mizar_resolve::{
    declarations::DeclarationShellCollector,
    env::NamespacePath,
    imports::{ImportPathCandidate, ImportPathResolver},
    module_index::ModuleIndexInput,
    resolved_ast::ModuleId as ResolverModuleId,
    symbols::SignatureProjectionExtractor,
};
use mizar_session::{
    BuildRequestId, BuildSessionId, BuildSnapshot, BuildSnapshotId, DependencyArtifactRef,
    DiskSourceLoader, Edition, Hash, IdError, InMemorySessionIdAllocator, LineMap, LoadingMap,
    ModulePath, PackageId, SessionIdAllocator, SnapshotLeaseId, SnapshotRegistry, SourceAnchor,
    SourceId, SourceInput, SourceMapId, SourceOrigin, SourceOriginInput, SourceVersion,
    ToolchainInfo, WorkspaceRoot, normalize_path,
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    bytes: Vec<u8>,
    version: SourceVersion,
    line_map: LineMap,
    loading_map: Option<LoadingMap>,
}

impl Fixture {
    fn new(bytes: &[u8]) -> Self {
        let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "mizar-driver-source-load-{}-{id}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        fs::create_dir_all(root.join("alpha/src")).unwrap();
        fs::write(root.join("alpha/src/main.miz"), bytes).unwrap();
        let package_root = root.join("alpha");
        let input = SourceInput {
            package_id: PackageId::new("alpha"),
            module_path: ModulePath::new("main"),
            normalized_path: normalize_path(&package_root, Path::new("src/main.miz")).unwrap(),
            edition: Edition::new("2025"),
            origin: SourceOriginInput::Disk {
                path: PathBuf::from("src/main.miz"),
            },
        };
        let provisional = snapshot_id(0x51);
        let loaded = mizar_session::SourceLoader::load(
            &DiskSourceLoader::new(&package_root),
            provisional,
            input,
            &InMemorySessionIdAllocator::new(),
        )
        .unwrap();
        let line_map = loaded.line_map.clone();
        let loading_map = loaded.loading_map.clone();
        let version = SourceVersion {
            source_id: loaded.source_id,
            package_id: loaded.package_id,
            module_path: loaded.module_path,
            normalized_path: loaded.normalized_path,
            source_hash: loaded.source_hash,
            edition: loaded.edition,
            origin: loaded.origin,
        };
        Self {
            root,
            bytes: bytes.to_vec(),
            version,
            line_map,
            loading_map,
        }
    }

    fn write(&self, bytes: &[u8]) {
        fs::write(self.root.join("alpha/src/main.miz"), bytes).unwrap();
    }

    fn submit(
        &self,
        ids: &InMemorySessionIdAllocator,
        snapshots: &SnapshotRegistry<InMemorySessionIdAllocator>,
    ) -> mizar_driver::driver::BuildSubmission {
        self.submit_with_dependencies(ids, snapshots, Vec::new(), Vec::new())
    }

    fn submit_with_dependencies(
        &self,
        ids: &InMemorySessionIdAllocator,
        snapshots: &SnapshotRegistry<InMemorySessionIdAllocator>,
        captured_artifacts: Vec<DependencyArtifactRef>,
        indexed_artifacts: Vec<DependencyArtifactIndex>,
    ) -> mizar_driver::driver::BuildSubmission {
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        let registry = builder.build().unwrap();
        let mut driver = CompilerDriver::new(registry);
        let (request, input) = self.submit_request(captured_artifacts, indexed_artifacts);
        let submission = driver.submit(request, ids, snapshots, input).unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        submission
    }

    fn submit_request(
        &self,
        captured_artifacts: Vec<DependencyArtifactRef>,
        indexed_artifacts: Vec<DependencyArtifactIndex>,
    ) -> (BuildRequestDraft, DriverSubmitInput<StaticSourceLayout>) {
        let has_dependency = !indexed_artifacts.is_empty();
        let mut input = DriverSubmitInput::new(
            PlanRequest {
                workspace_root: WorkspaceRoot::new(self.root.to_string_lossy().into_owned()),
                dependency_selection: DependencySelection::Normal,
                toolchain: ToolchainInfo::new("mizar-evo-test"),
            },
            vec![WorkspacePackage {
        member_path: "alpha".to_owned(),
        manifest: parse_package_manifest(if has_dependency {
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[dependencies]\ndep = \"1.0.0\"\n"
        } else {
            "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n"
        }).unwrap(),
    }],
            parse_lockfile(if has_dependency {
                "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = [{ name = \"dep\", version = \"1.0.0\" }]\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:dep\" }\ndependencies = []\n"
            } else {
                "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n"
            }).unwrap(),
            StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
            }]),
        );
        input.dependency_overlay = ModuleDependencyOverlay::complete(Vec::new());
        input.dependency_artifacts = indexed_artifacts;
        let request = BuildRequestDraft {
            lane: BuildLaneId::new(1),
            origin: BuildRequestOrigin::Batch(BatchRequest {
                invocation: BatchInvocation::default(),
            }),
            generation: BuildRequestGeneration::new(0),
            workspace_root: WorkspaceRoot::new(self.root.to_string_lossy().into_owned()),
            profile: BuildProfile::new("check"),
            targets: BuildTargets::default(),
            source_inputs: SourceInputSet {
                versions: vec![self.version.clone()],
            },
            dependency_inputs: DependencyInputSet::new(
                captured_artifacts,
                hash(1),
                ToolchainInfo::new("mizar-evo-test"),
            ),
            verifier_config: VerifierConfigInput::new(hash(2)),
        };
        (request, input)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct WorkspaceLeafFixture {
    fixture: Fixture,
    ids: InMemorySessionIdAllocator,
    submission: mizar_driver::driver::BuildSubmission,
    publisher: std::sync::Arc<PhaseOutputPublisher>,
}

impl WorkspaceLeafFixture {
    fn new(importer: &[u8], leaf: &[u8], base: Option<&[u8]>, spill_threshold: usize) -> Self {
        let fixture = Fixture::new(importer);
        let ids = InMemorySessionIdAllocator::new();
        ids.next_source_id(snapshot_id(0x52)).unwrap();
        fs::write(fixture.root.join("alpha/src/leaf.miz"), leaf).unwrap();
        let package_root = fixture.root.join("alpha");
        let loaded = mizar_session::SourceLoader::load(
            &DiskSourceLoader::new(&package_root),
            snapshot_id(0x52),
            SourceInput {
                package_id: PackageId::new("alpha"),
                module_path: ModulePath::new("leaf"),
                normalized_path: normalize_path(&package_root, Path::new("src/leaf.miz")).unwrap(),
                edition: Edition::new("2025"),
                origin: SourceOriginInput::Disk {
                    path: PathBuf::from("src/leaf.miz"),
                },
            },
            &ids,
        )
        .unwrap();
        let leaf_version = SourceVersion {
            source_id: loaded.source_id,
            package_id: loaded.package_id,
            module_path: loaded.module_path,
            normalized_path: loaded.normalized_path,
            source_hash: loaded.source_hash,
            edition: loaded.edition,
            origin: loaded.origin,
        };
        let base_version = base.map(|bytes| {
            fs::write(fixture.root.join("alpha/src/base.miz"), bytes).unwrap();
            let loaded = mizar_session::SourceLoader::load(
                &DiskSourceLoader::new(&package_root),
                snapshot_id(0x52),
                SourceInput {
                    package_id: PackageId::new("alpha"),
                    module_path: ModulePath::new("base"),
                    normalized_path: normalize_path(&package_root, Path::new("src/base.miz"))
                        .unwrap(),
                    edition: Edition::new("2025"),
                    origin: SourceOriginInput::Disk {
                        path: PathBuf::from("src/base.miz"),
                    },
                },
                &ids,
            )
            .unwrap();
            SourceVersion {
                source_id: loaded.source_id,
                package_id: loaded.package_id,
                module_path: loaded.module_path,
                normalized_path: loaded.normalized_path,
                source_hash: loaded.source_hash,
                edition: loaded.edition,
                origin: loaded.origin,
            }
        });
        let (mut request, mut input) = fixture.submit_request(Vec::new(), Vec::new());
        request.source_inputs.versions.push(leaf_version);
        if let Some(version) = base_version {
            request.source_inputs.versions.push(version);
        }
        let mut files = vec![
            WorkspaceSourceFile::new("src/main.miz", "main.miz"),
            WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
        ];
        if base.is_some() {
            files.push(WorkspaceSourceFile::new("src/base.miz", "base.miz"));
        }
        input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files,
        }]);
        let submission = CompilerDriver::new(source_registry())
            .submit(request, &ids, &SnapshotRegistry::new(), input)
            .unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let snapshot = submission.session.captured().snapshot().id;
        let tasks = submission.task_graph.as_ref().unwrap().tasks();
        let first = tasks
            .iter()
            .find(|task| task.kind == TaskKind::SourceLoad)
            .unwrap();
        let publisher = publisher(snapshot, spill_threshold, &first.unit);
        for task in tasks
            .iter()
            .filter(|task| task.kind == TaskKind::SourceLoad)
        {
            let WorkUnit::Module { module } = &task.unit else {
                unreachable!()
            };
            publisher.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SourceLoad"),
                OutputKind::new("SourceUnit"),
                IrWorkUnit::new(format!(
                    "{:?}:{:?}",
                    module.package.as_str(),
                    module.path.as_str()
                )),
            ));
            allow_frontend(&publisher, &task.unit);
        }
        Self {
            fixture,
            ids,
            submission,
            publisher,
        }
    }

    fn version(&self, module: &str) -> &SourceVersion {
        self.submission
            .session
            .request()
            .source_inputs
            .versions
            .iter()
            .find(|version| version.module_path.as_str() == module)
            .unwrap()
    }

    fn task(&self, kind: TaskKind, module: &str) -> &BuildTask {
        self.submission
            .task_graph
            .as_ref()
            .unwrap()
            .tasks()
            .iter()
            .find(|task| {
                task.kind == kind
                    && matches!(&task.unit, WorkUnit::Module { module: id }
                if id.path.as_str() == module)
            })
            .unwrap()
    }

    fn source(&self, module: &str) -> mizar_driver::registry::PhaseResult {
        let snapshot = self.submission.session.captured().snapshot().id;
        source_registry()
            .execute_phase_with_resources(
                PipelinePhase::SourceLoad,
                PhaseInput::new(
                    self.task(TaskKind::SourceLoad, module).unit.clone(),
                    PhaseDispatchInputBundle::without_parent_outputs(
                        snapshot,
                        source_key(self.version(module)),
                        Vec::new(),
                    ),
                ),
                PhaseExecutionResources {
                    diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                        DiagnosticPhase::SourceLoad,
                        snapshot,
                        "frontend.source_load",
                    ))),
                    output_publisher: Some(self.publisher.clone()),
                    source_load: Some(SourceLoadInputs {
                        captured: None,
                        snapshot: self.submission.session.captured().snapshot(),
                        build_plan: self.submission.build_plan.as_ref().unwrap(),
                        module_index: self.submission.module_index.as_ref().unwrap(),
                        allocator: &self.ids,
                    }),
                    ..PhaseExecutionResources::default()
                },
            )
            .unwrap()
            .result
    }

    fn frontend(
        &self,
        module: &str,
        parents: Vec<AnyPhaseOutputRef>,
        source_snapshot: Option<&BuildSnapshot>,
    ) -> mizar_driver::registry::PhaseResult {
        let snapshot = self.submission.session.captured().snapshot().id;
        let sealed = parents
            .into_iter()
            .map(|parent| {
                mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
                    &self.publisher,
                    snapshot,
                    parent,
                )
                .unwrap()
            })
            .collect();
        self.frontend_bundle(
            module,
            PhaseDispatchInputBundle::new(
                snapshot,
                source_key(self.version(module)),
                Vec::new(),
                sealed,
            )
            .unwrap(),
            source_snapshot,
        )
    }

    fn frontend_bundle(
        &self,
        module: &str,
        bundle: PhaseDispatchInputBundle,
        source_snapshot: Option<&BuildSnapshot>,
    ) -> mizar_driver::registry::PhaseResult {
        let snapshot = self.submission.session.captured().snapshot().id;
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(Vec::new());
        builder
            .build()
            .unwrap()
            .execute_phase_with_resources(
                PipelinePhase::Frontend,
                PhaseInput::new(self.task(TaskKind::Frontend, module).unit.clone(), bundle),
                PhaseExecutionResources {
                    diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                        DiagnosticPhase::Frontend,
                        snapshot,
                        "frontend.run_loaded",
                    ))),
                    output_publisher: Some(self.publisher.clone()),
                    source_load: Some(SourceLoadInputs {
                        captured: None,
                        snapshot: source_snapshot
                            .unwrap_or(self.submission.session.captured().snapshot()),
                        build_plan: self.submission.build_plan.as_ref().unwrap(),
                        module_index: self.submission.module_index.as_ref().unwrap(),
                        allocator: &self.ids,
                    }),
                    ..PhaseExecutionResources::default()
                },
            )
            .unwrap()
            .result
    }

    fn clean_parents(&self) -> (AnyPhaseOutputRef, AnyPhaseOutputRef) {
        let leaf_source = self.source("leaf");
        assert_eq!(leaf_source.status, PhaseStatus::Complete);
        let leaf_frontend = self.frontend("leaf", leaf_source.output_refs, None);
        assert_eq!(leaf_frontend.status, PhaseStatus::Complete);
        assert!(leaf_frontend.diagnostics.is_empty());
        let importer_source = self.source("main");
        assert_eq!(importer_source.status, PhaseStatus::Complete);
        (
            importer_source.output_refs[0].clone(),
            leaf_frontend.output_refs[0].clone(),
        )
    }

    fn scheduled_request(
        &self,
        overlay: ModuleDependencyOverlay,
        captured: Vec<DependencyArtifactRef>,
        indexed: Vec<DependencyArtifactIndex>,
    ) -> (BuildRequestDraft, DriverSubmitInput<StaticSourceLayout>) {
        let (mut request, mut input) = self.fixture.submit_request(captured, indexed);
        request.source_inputs.versions = self
            .submission
            .session
            .request()
            .source_inputs
            .versions
            .clone();
        assert!(matches!(request.source_inputs.versions.len(), 2 | 3));
        let mut files = vec![
            WorkspaceSourceFile::new("src/main.miz", "main.miz"),
            WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
        ];
        if request
            .source_inputs
            .versions
            .iter()
            .any(|version| version.module_path.as_str() == "base")
        {
            files.push(WorkspaceSourceFile::new("src/base.miz", "base.miz"));
        }
        input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files,
        }]);
        input.dependency_overlay = overlay;
        (request, input)
    }

    fn scheduled_driver(&self) -> CompilerDriver {
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(Vec::new());
        CompilerDriver::new(builder.build().unwrap()).with_output_publisher(self.publisher.clone())
    }
}

fn complete_leaf_import_overlay() -> ModuleDependencyOverlay {
    ModuleDependencyOverlay::complete(vec![ModuleDependencyEdge::new(
        mizar_build::module_index::ModuleId::new(PackageId::new("alpha"), ModulePath::new("main")),
        mizar_build::module_index::ModuleId::new(PackageId::new("alpha"), ModulePath::new("leaf")),
        ModuleDependencyKind::ImportSummary,
    )])
}

fn execute(
    submission: &mizar_driver::driver::BuildSubmission,
    ids: &InMemorySessionIdAllocator,
    threshold: usize,
) -> (
    mizar_driver::registry::PhaseResult,
    std::sync::Arc<PhaseOutputPublisher>,
) {
    let task = source_task(submission);
    let snapshot = submission.session.captured().snapshot().id;
    // Force the loader to allocate a different ID from the captured source.
    ids.next_source_id(snapshot).unwrap();
    let publisher = publisher(snapshot, threshold, &task.unit);
    let source_load = SourceLoadInputs {
        captured: None,
        snapshot: submission.session.captured().snapshot(),
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: ids,
    };
    let sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        DiagnosticPhase::SourceLoad,
        snapshot,
        "frontend.source_load",
    ));
    let input = PhaseInput::new(
        task.unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(
            snapshot,
            source_key(&submission.session.request().source_inputs.versions[0]),
            Vec::new(),
        ),
    );
    let result = source_registry()
        .execute_phase_with_resources(
            PipelinePhase::SourceLoad,
            input,
            PhaseExecutionResources {
                diagnostics: Some(sink),
                output_publisher: Some(publisher.clone()),
                source_load: Some(source_load),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result;
    (result, publisher)
}

fn execute_frontend(
    submission: &mizar_driver::driver::BuildSubmission,
    ids: &InMemorySessionIdAllocator,
    publisher: &std::sync::Arc<PhaseOutputPublisher>,
    parents: Vec<AnyPhaseOutputRef>,
    dispatch: (Hash, Vec<Hash>),
    artifact_roots: Vec<(PackageId, PathBuf)>,
    module_index: Option<&ModuleIndex>,
) -> mizar_driver::registry::PhaseResult {
    let snapshot = submission.session.captured().snapshot().id;
    let task = submission
        .task_graph
        .as_ref()
        .unwrap()
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    let ir_unit = IrWorkUnit::new(match &task.unit {
        WorkUnit::Module { module } => {
            format!("{:?}:{:?}", module.package.as_str(), module.path.as_str())
        }
        _ => unreachable!(),
    });
    publisher.allow_work_unit(AllowedWorkUnit::new(
        IrPipelinePhase::new("Frontend"),
        OutputKind::new("FrontendOutput"),
        ir_unit,
    ));
    let parents = parents
        .into_iter()
        .map(|parent| {
            mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
                publisher, snapshot, parent,
            )
            .unwrap()
        })
        .collect();
    let input = PhaseInput::new(
        task.unit.clone(),
        PhaseDispatchInputBundle::new(snapshot, dispatch.0, dispatch.1, parents).unwrap(),
    );
    let source_load = SourceLoadInputs {
        captured: None,
        snapshot: submission.session.captured().snapshot(),
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: module_index.unwrap_or(submission.module_index.as_ref().unwrap()),
        allocator: ids,
    };
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(artifact_roots);
    let registry = builder.build().unwrap();
    let cache = registry
        .cache_key_for_phase(PipelinePhase::Frontend, &input)
        .unwrap();
    assert!(matches!(cache.intent, PhaseCacheIntent::NoKey { .. }));
    registry
        .execute_phase_with_resources(
            PipelinePhase::Frontend,
            input,
            PhaseExecutionResources {
                diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                    DiagnosticPhase::Frontend,
                    snapshot,
                    "frontend.run_loaded",
                ))),
                output_publisher: Some(publisher.clone()),
                source_load: Some(source_load),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result
}

fn binding_result(
    submission: &mizar_driver::driver::BuildSubmission,
    source_load: SourceLoadInputs<'_>,
    unit: WorkUnit,
) -> PhaseStatus {
    let snapshot = submission.session.captured().snapshot().id;
    let module_unit = source_task(submission).unit.clone();
    let publisher = publisher(snapshot, usize::MAX, &module_unit);
    let sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        DiagnosticPhase::SourceLoad,
        snapshot,
        "frontend.source_load",
    ));
    source_registry()
        .execute_phase_with_resources(
            PipelinePhase::SourceLoad,
            PhaseInput::new(
                unit,
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    source_key(&submission.session.request().source_inputs.versions[0]),
                    Vec::new(),
                ),
            ),
            PhaseExecutionResources {
                diagnostics: Some(sink),
                output_publisher: Some(publisher),
                source_load: Some(source_load),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result
        .status
}

fn source_registry() -> mizar_driver::registry::PhaseRegistry {
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.build().unwrap()
}

fn publisher(
    snapshot: BuildSnapshotId,
    threshold: usize,
    unit: &WorkUnit,
) -> std::sync::Arc<PhaseOutputPublisher> {
    let phase = IrPipelinePhase::new("SourceLoad");
    let kind = OutputKind::new("SourceUnit");
    let ir_unit = IrWorkUnit::new(match unit {
        WorkUnit::Module { module } => {
            format!("{:?}:{:?}", module.package.as_str(), module.path.as_str())
        }
        _ => unreachable!(),
    });
    let publisher = std::sync::Arc::new(PhaseOutputPublisher::new(
        std::sync::Arc::new(IrStorageService::with_policy(
            StoragePolicy::with_blob_spill_threshold(threshold),
        )),
        std::sync::Arc::new(SnapshotHandleRegistry::new()),
    ));
    publisher.register_current_snapshot(snapshot);
    publisher.allow_work_unit(AllowedWorkUnit::new(phase, kind, ir_unit));
    publisher
}

fn allow_frontend(publisher: &PhaseOutputPublisher, unit: &WorkUnit) {
    let WorkUnit::Module { module } = unit else {
        unreachable!();
    };
    publisher.allow_work_unit(AllowedWorkUnit::new(
        IrPipelinePhase::new("Frontend"),
        OutputKind::new("FrontendOutput"),
        IrWorkUnit::new(format!(
            "{:?}:{:?}",
            module.package.as_str(),
            module.path.as_str()
        )),
    ));
}

#[derive(Debug, Clone, Copy)]
enum SuppliedInput {
    None,
    Error,
    ForeignSnapshot,
}

impl PhaseDispatchInputProvider<BuildTask> for SuppliedInput {
    fn dispatch_input_for_task(
        &self,
        request: PhaseDispatchInputRequest<'_, BuildTask>,
    ) -> Result<Option<PhaseDispatchInputBundle>, DispatchInputError> {
        match self {
            Self::None => Ok(None),
            Self::Error => Err(DispatchInputError::DispatchSnapshotMismatch {
                expected: request.snapshot(),
                actual: snapshot_id(0xee),
            }),
            Self::ForeignSnapshot => Ok(Some(PhaseDispatchInputBundle::without_parent_outputs(
                snapshot_id(0xef),
                hash(0xef),
                Vec::new(),
            ))),
        }
    }
}

fn source_task(
    submission: &mizar_driver::driver::BuildSubmission,
) -> &mizar_build::task_graph::BuildTask {
    submission
        .task_graph
        .as_ref()
        .unwrap()
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::SourceLoad)
        .unwrap()
}

fn source_key(version: &SourceVersion) -> Hash {
    SourceUnitCacheKey {
        version: std::sync::Arc::from(SOURCE_UNIT_CACHE_KEY_VERSION),
        package_id: version.package_id.clone(),
        module_path: version.module_path.clone(),
        normalized_path: version.normalized_path.clone(),
        source_hash: version.source_hash,
        edition: version.edition.clone(),
    }
    .stable_hash()
}

fn snapshot_id(seed: u8) -> BuildSnapshotId {
    let encoded = format!(
        "mizar-session-build-snapshot-v1:{}",
        format!("{seed:02x}").repeat(Hash::BYTE_LEN)
    );
    BuildSnapshotId::from_published_schema_str(&encoded).unwrap()
}

fn hash(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

#[derive(Debug)]
struct SourceIdFailureAllocator {
    inner: InMemorySessionIdAllocator,
}

impl SessionIdAllocator for SourceIdFailureAllocator {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
        self.inner.next_session_id()
    }

    fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
        self.inner.next_request_id()
    }

    fn next_source_id(&self, _snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
        Err(IdError::AllocatorOverflow)
    }

    fn next_source_map_id(&self, snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
        self.inner.next_source_map_id(snapshot)
    }

    fn next_lease_id(&self, snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
        self.inner.next_lease_id(snapshot)
    }
}

fn assert_source_error(
    result: &mizar_driver::registry::PhaseResult,
    version: &SourceVersion,
    code: u16,
    detail: &str,
) {
    assert_eq!(result.status, PhaseStatus::Fatal);
    assert!(result.output_refs.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].drafts().len(), 1);
    let draft = &result.diagnostics[0].drafts()[0];
    assert_eq!(draft.code().number(), code);
    assert_eq!(draft.stable_detail_key(), detail);
    assert_eq!(
        draft.primary_location(),
        &mizar_diagnostics::failure_record::DiagnosticPrimaryLocation::SourceLoad {
            package_id: version.package_id.clone(),
            path: version.normalized_path.clone(),
        }
    );
}

fn native_disk_record_submission(
    fixture: &Fixture,
    ids: &InMemorySessionIdAllocator,
    snapshots: &SnapshotRegistry<InMemorySessionIdAllocator>,
    threshold: usize,
) -> (
    CompilerDriver,
    mizar_driver::driver::BuildSubmission,
    std::sync::Arc<PhaseOutputPublisher>,
    SourceInput,
    mizar_ir::storage::PhaseOutputRef<SourceUnit>,
) {
    let (mut draft, input) = fixture.submit_request(vec![], vec![]);
    draft.source_inputs.versions.clear();
    let descriptor = SourceInput {
        package_id: fixture.version.package_id.clone(),
        module_path: fixture.version.module_path.clone(),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: fixture.version.edition.clone(),
        origin: SourceOriginInput::Disk {
            path: "src/main.miz".into(),
        },
    };
    let session = draft
        .capture_sources(
            ids,
            snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                descriptor.clone(),
            )],
        )
        .unwrap();
    let version = session.captured().snapshot().source_versions[0].clone();
    let snapshot = session.captured().snapshot().id;
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            version.package_id.clone(),
            version.module_path.clone(),
        ),
    };
    let owner = publisher(snapshot, threshold, &unit);
    allow_frontend(&owner, &unit);
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(vec![]);
    let mut driver =
        CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
    let submission = driver
        .submit_captured(session, ids, snapshots, input)
        .unwrap();
    let result = submission
        .scheduler_run
        .as_ref()
        .unwrap()
        .phase_results
        .values()
        .flatten()
        .find(|result| {
            result
                .output_refs
                .iter()
                .any(|handle| handle.output_kind() == &OutputKind::new("SourceUnit"))
        })
        .unwrap();
    assert_eq!(result.status, PhaseStatus::Complete);
    let handle = owner
        .storage()
        .typed_handle::<SourceUnit>(&result.output_refs[0], &OutputKind::new("SourceUnit"))
        .unwrap();
    (driver, submission, owner, descriptor, handle)
}

fn native_disk_lookup_context<'a>(
    submission: &'a mizar_driver::driver::BuildSubmission,
    ids: &'a InMemorySessionIdAllocator,
) -> (SourceLoadInputs<'a>, PhaseInput) {
    let captured = submission.session.captured();
    (
        SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: ids,
        },
        PhaseInput::new(
            source_task(submission).unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                captured.snapshot().id,
                source_key(&captured.snapshot().source_versions[0]),
                vec![],
            ),
        ),
    )
}

fn native_disk_frontend_record_submission(
    fixture: &Fixture,
    ids: &InMemorySessionIdAllocator,
    snapshots: &SnapshotRegistry<InMemorySessionIdAllocator>,
    threshold: usize,
) -> (
    CompilerDriver,
    mizar_driver::driver::BuildSubmission,
    mizar_ir::cache_adapter::IrCacheAdapter,
    mizar_ir::storage::PhaseOutputRef<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>,
    PhaseInput,
) {
    use mizar_build::planner::{lockfile_content_hash, workspace_config_content_hash};
    let (mut draft, input) = fixture.submit_request(vec![], vec![]);
    let plan = produce_build_plan(
        input.plan_request.clone(),
        input.workspace_packages.clone(),
        input.lockfile.clone(),
    )
    .unwrap();
    draft.source_inputs.versions.clear();
    draft.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
    draft.verifier_config = VerifierConfigInput::new(workspace_config_content_hash(
        &plan.verifier_config,
        &plan.build_config,
    ));
    let descriptor = SourceInput {
        package_id: fixture.version.package_id.clone(),
        module_path: fixture.version.module_path.clone(),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: fixture.version.edition.clone(),
        origin: SourceOriginInput::Disk {
            path: "src/main.miz".into(),
        },
    };
    let session = draft
        .capture_sources(
            ids,
            snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                descriptor,
            )],
        )
        .unwrap();
    let snapshot = session.captured().snapshot().id;
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        ),
    };
    let owner = publisher(snapshot, threshold, &unit);
    allow_frontend(&owner, &unit);
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(vec![]);
    let mut driver =
        CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
    let submission = driver
        .submit_captured(session, ids, snapshots, input)
        .unwrap();
    let (_, source_input) = native_disk_lookup_context(&submission, ids);
    let results = &submission.scheduler_run.as_ref().unwrap().phase_results;
    let source_ref = results
        .values()
        .flatten()
        .flat_map(|result| &result.output_refs)
        .find(|handle| handle.output_kind() == &OutputKind::new("SourceUnit"))
        .unwrap();
    let handle = owner
        .storage()
        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
            results
                .values()
                .flatten()
                .flat_map(|result| &result.output_refs)
                .find(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput"))
                .unwrap(),
            &OutputKind::new("FrontendOutput"),
        )
        .unwrap();
    let parent =
        SealedParentOutputHandle::from_published_output(&owner, snapshot, source_ref.clone())
            .unwrap();
    let phase_input = PhaseInput::new(
        unit,
        PhaseDispatchInputBundle::new(
            snapshot,
            source_input.identities().input_hash(),
            vec![],
            vec![parent],
        )
        .unwrap(),
    );
    (
        driver,
        submission,
        mizar_ir::cache_adapter::IrCacheAdapter::new(owner),
        handle,
        phase_input,
    )
}

fn configured_native_cache_case(
    settings: &[(&str, &str, bool)],
    typed_cache_path: Option<&str>,
    threshold: usize,
    store_path: &str,
    (cache_policy, insert_records): (CacheSchedulingPolicy, bool),
    workers: usize,
    reverse: bool,
) {
    use mizar_build::{
        planner::{lockfile_content_hash, workspace_config_content_hash},
        scheduler::SchedulerEventKind,
    };
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot};
    use mizar_ir::{
        cache_adapter::{EncodeCacheRecordOutcome, IrCacheAdapter},
        storage::CollectInput,
    };
    let raw = "\u{feff}definition\r\nend;\r\n";
    let fixture = Fixture::new(raw.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let policy = mizar_proof::policy::VerifierPolicy::release();
    let mut warm_versions = Vec::new();
    let mut replay_session = None;
    let mut clean = Vec::new();
    // Warm normally, recapture cleanly, then replay the legacy and configured routes.
    for route in 0..4 {
        let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
        input.workspace_packages = settings
            .iter()
            .map(|(name, build, _)| WorkspacePackage {
                member_path: (*name).into(),
                manifest: parse_package_manifest(&format!(
                    "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n{build}"
                ))
                .unwrap(),
            })
            .collect();
        // Direct typed values exercise eligibility without claiming parser acceptance.
        if let Some(path) = typed_cache_path {
            input.workspace_packages[0].manifest.build.cache_dir = if path == "$absolute" {
                fixture.root.join("absolute-cache").to_str().unwrap().into()
            } else {
                path.into()
            };
        }
        let store = CacheStoreRoot::new(if store_path == "$configured" {
            fixture
                .root
                .join(settings[0].0)
                .join(&input.workspace_packages[0].manifest.build.cache_dir)
        } else {
            fixture.root.join(store_path)
        });
        input.lockfile = parse_lockfile(&format!(
            "schema_version = 1\n{}",
            settings
                .iter()
                .map(|(name, _, _)| format!(
                    "[[package]]\nname = \"{name}\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"{name}\" }}\ndependencies = []\n"
                ))
                .collect::<String>()
        ))
        .unwrap();
        let mut layout = settings
            .iter()
            .map(|(name, _, _)| WorkspaceSourcePackage {
                package_id: PackageId::new(*name),
                files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
            })
            .collect::<Vec<_>>();
        if reverse {
            input.workspace_packages.reverse();
            layout.reverse();
        }
        input.source_layout = StaticSourceLayout::new(layout);
        input.worker_count = workers;
        input.completion_order = if reverse {
            CompletionOrder::Reverse
        } else {
            CompletionOrder::Canonical
        };
        input.cache_policy = if route == 3 {
            cache_policy
        } else {
            CacheSchedulingPolicy::Enabled
        };
        let plan = produce_build_plan(
            input.plan_request.clone(),
            input.workspace_packages.clone(),
            input.lockfile.clone(),
        )
        .unwrap();
        request.source_inputs.versions.clear();
        request.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
        request.verifier_config = VerifierConfigInput::new(workspace_config_content_hash(
            &plan.verifier_config,
            &plan.build_config,
        ));
        let session = if route < 2 {
            let mut descriptors = settings
                .iter()
                .map(|(name, _, _)| {
                    let root = fixture.root.join(name);
                    fs::create_dir_all(root.join("src")).unwrap();
                    fs::write(root.join("src/main.miz"), raw).unwrap();
                    (
                        DiskSourceLoader::new(&root),
                        SourceInput {
                            package_id: PackageId::new(*name),
                            module_path: ModulePath::new("main"),
                            normalized_path: normalize_path(&root, Path::new("src/main.miz"))
                                .unwrap(),
                            edition: Edition::new("2025"),
                            origin: SourceOriginInput::Disk {
                                path: "src/main.miz".into(),
                            },
                        },
                    )
                })
                .collect::<Vec<_>>();
            if reverse {
                descriptors.reverse();
            }
            let session = request
                .capture_sources(&ids, &snapshots, descriptors)
                .unwrap();
            for (name, _, _) in settings {
                fs::remove_file(fixture.root.join(name).join("src/main.miz")).unwrap();
            }
            if route == 1 {
                replay_session = Some(session.clone());
            }
            session
        } else {
            replay_session.as_ref().unwrap().clone()
        };
        let snapshot = session.captured().snapshot().id;
        let first_unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new(settings[0].0),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(snapshot, threshold, &first_unit);
        for (name, _, _) in settings {
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new(*name),
                    ModulePath::new("main"),
                ),
            };
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SourceLoad"),
                OutputKind::new("SourceUnit"),
                IrWorkUnit::new(format!("{name:?}:{:?}", "main")),
            ));
            allow_frontend(&owner, &unit);
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let submission = match route {
            0 | 1 => driver.submit_captured(session, &ids, &snapshots, input),
            2 => driver.submit_captured_with_native_disk_cache(
                session, &ids, &snapshots, input, &policy, &store,
            ),
            _ => driver.submit_captured_with_configured_native_disk_cache(
                session, &ids, &snapshots, input, &policy, &store,
            ),
        }
        .unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        if !insert_records {
            assert!(!store.root().exists());
        }
        let captured = submission.session.captured();
        let inputs = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: &ids,
        };
        assert_eq!(inputs.validate_native_lock_content(), Ok(()));
        assert_eq!(inputs.validate_native_config_content(), Ok(()));
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        for (ordinal, (name, _, configured_hit)) in settings.iter().enumerate() {
            let source_task = graph.tasks().iter().find(|task| task.kind == TaskKind::SourceLoad && matches!(&task.unit, WorkUnit::Module { module } if module.package.as_str() == *name)).unwrap();
            let frontend_task = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::Frontend && task.unit == source_task.unit)
                .unwrap();
            let version = captured
                .snapshot()
                .source_versions
                .iter()
                .find(|version| version.package_id.as_str() == *name)
                .unwrap();
            if route == 0 {
                warm_versions.push(version.source_id);
            } else {
                assert_ne!(version.source_id, warm_versions[ordinal]);
            }
            let hit = insert_records && (route == 2 || (route == 3 && *configured_hit));
            let source_result = &run.phase_results[&source_task.id][0];
            let frontend_result = &run.phase_results[&frontend_task.id][0];
            for (task, expected, started) in [
                (
                    source_task,
                    if hit {
                        TaskState::CacheHit
                    } else {
                        TaskState::Completed
                    },
                    !hit,
                ),
                (frontend_task, TaskState::Completed, true),
            ] {
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == task.id)
                        .unwrap()
                        .state,
                    expected,
                    "{name} route {route}"
                );
                assert_eq!(
                    run.events
                        .iter()
                        .any(|event| event.task_id.as_ref() == Some(&task.id)
                            && event.kind == SchedulerEventKind::TaskStarted),
                    started
                );
            }
            assert_eq!(source_result.cache_observation.is_none(), hit);
            assert!(frontend_result.cache_observation.is_some());
            let source_input = PhaseInput::new(
                source_task.unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    source_key(version),
                    vec![],
                ),
            );
            let source_ref = &source_result.output_refs[0];
            let source_handle = owner
                .storage()
                .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                .unwrap();
            let source = owner.storage().get(&source_handle).unwrap();
            assert_eq!(source.source_id, version.source_id);
            assert_eq!(source.source_text.as_ref(), "definition\nend;\n");
            assert!(source.loading_map.is_some());
            assert!(source_ref.lineage().parents.is_empty());
            let frontend_ref = &frontend_result.output_refs[0];
            let frontend_handle = owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    frontend_ref,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let frontend = owner.storage().get(&frontend_handle).unwrap();
            assert_eq!(frontend.source, *source);
            assert_eq!(frontend_ref.lineage().parents, vec![source_ref.output()]);
            let frontend_input = PhaseInput::new(
                source_task.unit.clone(),
                PhaseDispatchInputBundle::new(
                    snapshot,
                    source_key(version),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            &owner,
                            snapshot,
                            source_ref.clone(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            for (phase, input) in [
                (PipelinePhase::SourceLoad, &source_input),
                (PipelinePhase::Frontend, &frontend_input),
            ] {
                assert!(matches!(
                    driver
                        .registry()
                        .cache_key_for_phase(phase, input)
                        .unwrap()
                        .intent,
                    PhaseCacheIntent::NoKey { .. }
                ));
            }
            for result in [source_result, frontend_result] {
                assert_eq!(result.status, PhaseStatus::Complete);
                assert!(result.diagnostics.is_empty());
                assert_eq!(
                    matches!(
                        result.output_refs[0].placement(),
                        StoragePlacement::Blob { .. }
                    ),
                    threshold == 0
                );
            }
            let projection = (
                source.as_ref().clone(),
                frontend.as_ref().clone(),
                owner.storage().side_tables(&source_handle).unwrap(),
                owner.storage().side_tables(&frontend_handle).unwrap(),
                source_result.diagnostics.clone(),
                frontend_result.diagnostics.clone(),
                source_ref.content_hash(),
                frontend_ref.content_hash(),
            );
            if route == 1 {
                clean.push(projection);
            } else if route > 1 {
                assert_eq!(projection, clean[ordinal]);
            }
            if route == 0 && insert_records {
                let EncodeCacheRecordOutcome::Encoded(record) = inputs
                    .encode_native_disk_cache_record(
                        &source_input,
                        &policy,
                        &IrCacheAdapter::new(owner.clone()),
                        source_handle,
                    )
                    .unwrap()
                else {
                    panic!("real scheduled record");
                };
                assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
                assert!(matches!(
                    store.lookup(&record.header.key),
                    CacheLookupOutcome::Hit(_)
                ));
            }
        }
        let collected = owner.storage().collect(CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(collected.retained_outputs, 0);
        assert_eq!(collected.outputs_dropped, settings.len() * 2);
    }
}

fn persistent_native_cache_case(
    settings: &[(&str, &str, bool)],
    case: &str,
    stages: &[&str],
    threshold: usize,
    store_path: &str,
    workers: usize,
    reverse: bool,
) {
    use mizar_build::{
        planner::{lockfile_content_hash, workspace_config_content_hash},
        scheduler::SchedulerEventKind,
    };
    use mizar_cache::{
        cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
        cache_store::{CacheLookupOutcome, CacheStoreRoot},
    };
    use mizar_ir::storage::CollectInput;
    let fixture = Fixture::new(stages[0].as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let policy = mizar_proof::policy::VerifierPolicy::release();
    let store = CacheStoreRoot::new(fixture.root.join(store_path));
    assert!(!store.root().exists());
    if case == "store-error" {
        fs::write(store.root(), b"existing cache-root file").unwrap();
    }
    let mut prior_versions = Vec::new();
    let mut prior_keys = Vec::new();
    let mut prior_source_keys = Vec::new();
    let route_count = if matches!(case, "hit" | "edits" | "store-error") {
        4
    } else {
        2
    };
    for (stage, raw) in stages.iter().enumerate() {
        let mut replay_session: Option<mizar_driver::request::BuildSession> = None;
        let mut control_tasks: Option<(
            mizar_build::task_graph::TaskId,
            mizar_build::task_graph::TaskId,
        )> = None;
        let mut clean = Vec::new();
        let mut present_before = Vec::new();
        for route in 0..route_count {
            let actual = route % 2 == 1;
            let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
            input.workspace_packages = settings
                .iter()
                .map(|(name, build, _)| WorkspacePackage {
                    member_path: (*name).into(),
                    manifest: parse_package_manifest(&format!(
                        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n{build}"
                    ))
                    .unwrap(),
                })
                .collect();
            input.lockfile = parse_lockfile(&format!(
                "schema_version = 1\n{}",
                settings.iter().map(|(name, _, _)| format!(
                    "[[package]]\nname = \"{name}\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"{name}\" }}\ndependencies = []\n"
                )).collect::<String>()
            )).unwrap();
            let mut layout = settings
                .iter()
                .map(|(name, _, _)| WorkspaceSourcePackage {
                    package_id: PackageId::new(*name),
                    files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
                })
                .collect::<Vec<_>>();
            if reverse {
                input.workspace_packages.reverse();
                layout.reverse();
            }
            input.source_layout = StaticSourceLayout::new(layout);
            input.worker_count = workers;
            input.completion_order = if reverse {
                CompletionOrder::Reverse
            } else {
                CompletionOrder::Canonical
            };
            input.cache_policy = if case == "disabled" {
                CacheSchedulingPolicy::Disabled
            } else {
                CacheSchedulingPolicy::Enabled
            };
            let plan = produce_build_plan(
                input.plan_request.clone(),
                input.workspace_packages.clone(),
                input.lockfile.clone(),
            )
            .unwrap();
            request.source_inputs.versions.clear();
            request.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
            request.verifier_config = VerifierConfigInput::new(if case == "context" {
                hash(2)
            } else {
                workspace_config_content_hash(&plan.verifier_config, &plan.build_config)
            });
            let session = if actual {
                replay_session.as_ref().unwrap().clone()
            } else {
                let mut descriptors = settings
                    .iter()
                    .map(|(name, _, _)| {
                        let root = fixture.root.join(name);
                        fs::create_dir_all(root.join("src")).unwrap();
                        fs::write(
                            root.join("src/main.miz"),
                            if *name == "alpha" {
                                raw.as_bytes()
                            } else {
                                b"definition\nend;\n"
                            },
                        )
                        .unwrap();
                        (
                            DiskSourceLoader::new(&root),
                            SourceInput {
                                package_id: PackageId::new(*name),
                                module_path: ModulePath::new("main"),
                                normalized_path: normalize_path(&root, Path::new("src/main.miz"))
                                    .unwrap(),
                                edition: Edition::new("2025"),
                                origin: SourceOriginInput::Disk {
                                    path: "src/main.miz".into(),
                                },
                            },
                        )
                    })
                    .collect::<Vec<_>>();
                if reverse {
                    descriptors.reverse();
                }
                let session = request
                    .capture_sources(&ids, &snapshots, descriptors)
                    .unwrap();
                for (name, _, _) in settings {
                    fs::remove_file(fixture.root.join(name).join("src/main.miz")).unwrap();
                }
                replay_session = Some(session.clone());
                session
            };
            let snapshot = session.captured().snapshot().id;
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new(settings[0].0),
                    ModulePath::new("main"),
                ),
            };
            let owner = publisher(snapshot, threshold, &unit);
            for (name, _, _) in settings {
                let unit = WorkUnit::Module {
                    module: mizar_build::module_index::ModuleId::new(
                        PackageId::new(*name),
                        ModulePath::new("main"),
                    ),
                };
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("SourceLoad"),
                    OutputKind::new("SourceUnit"),
                    IrWorkUnit::new(format!("{name:?}:{:?}", "main")),
                ));
                allow_frontend(&owner, &unit);
            }
            if case == "publisher" {
                owner.mark_obsolete(snapshot).unwrap();
            }
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            if case == "provider" {
                input.phase_dispatch_inputs = Some(Box::new(SuppliedInput::None));
            }
            if actual && matches!(case, "cancel" | "obsolete" | "cancel-frontend") {
                let (source, frontend) = control_tasks.as_ref().unwrap();
                match case {
                    "cancel" => input.cancellation.cancel_task(source.clone()),
                    "obsolete" => input
                        .cancellation
                        .discard_obsolete_completed_task(source.clone()),
                    "cancel-frontend" => input.cancellation.cancel_task(frontend.clone()),
                    _ => {}
                }
            }
            let submission = if actual {
                driver.submit_captured_with_persistent_native_disk_cache(
                    session, &ids, &snapshots, input, &policy, &store,
                )
            } else {
                driver.submit_captured(session, &ids, &snapshots, input)
            }
            .unwrap();
            if case == "publisher" {
                assert!(submission.scheduler_run.is_none());
                assert!(!store.root().exists());
                continue;
            }
            let captured = submission.session.captured();
            let inputs = SourceLoadInputs {
                snapshot: captured.snapshot(),
                captured: Some(captured),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            };
            let graph = submission.task_graph.as_ref().unwrap();
            let run = submission.scheduler_run.as_ref().unwrap();
            assert_eq!(
                submission.status,
                if case == "provider" {
                    DriverSubmissionStatus::BlockedByPhaseDispatchGap
                } else if raw.contains("theorem T: ;") {
                    DriverSubmissionStatus::SchedulerValidated
                } else {
                    DriverSubmissionStatus::BlockedByMissingPhaseServices
                }
            );
            if !actual {
                clean.clear();
                present_before.clear();
            }
            for (ordinal, (name, _, eligible)) in settings.iter().enumerate() {
                let source_task = graph.tasks().iter().find(|task| task.kind == TaskKind::SourceLoad && matches!(&task.unit, WorkUnit::Module { module } if module.package.as_str() == *name)).unwrap();
                let frontend_task = graph
                    .tasks()
                    .iter()
                    .find(|task| task.kind == TaskKind::Frontend && task.unit == source_task.unit)
                    .unwrap();
                if !actual && ordinal == 0 {
                    control_tasks = Some((source_task.id.clone(), frontend_task.id.clone()));
                }
                let version = captured
                    .snapshot()
                    .source_versions
                    .iter()
                    .find(|version| version.package_id.as_str() == *name)
                    .unwrap();
                if !actual {
                    if route == 0 && stage == 0 {
                        prior_versions.push(version.source_id);
                    } else {
                        assert_ne!(version.source_id, prior_versions[ordinal]);
                        prior_versions[ordinal] = version.source_id;
                    }
                }
                let source_input = PhaseInput::new(
                    source_task.unit.clone(),
                    PhaseDispatchInputBundle::without_parent_outputs(
                        snapshot,
                        source_key(version),
                        vec![],
                    ),
                );
                assert!(matches!(
                    driver
                        .registry()
                        .cache_key_for_phase(PipelinePhase::SourceLoad, &source_input)
                        .unwrap()
                        .intent,
                    PhaseCacheIntent::NoKey { .. }
                ));
                let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(
                    inputs
                        .prepare_native_disk_cache_request(&source_input, &policy)
                        .unwrap(),
                )
                .build() else {
                    panic!("native key");
                };
                let persisted = matches!(store.lookup(&key), CacheLookupOutcome::Hit(_));
                if !actual {
                    present_before.push(persisted);
                    if route == 0 {
                        if stage > 0 && *name == "alpha" {
                            assert_ne!(key, prior_keys[ordinal]);
                            assert_eq!(
                                source_key(version) == prior_source_keys[ordinal],
                                stage == 2
                            );
                        }
                        if stage == 0 {
                            prior_keys.push(key.clone());
                            prior_source_keys.push(source_key(version));
                        } else {
                            prior_keys[ordinal] = key.clone();
                            prior_source_keys[ordinal] = source_key(version);
                        }
                    }
                }
                let writes = *eligible && matches!(case, "hit" | "edits");
                let hit = actual && writes && present_before[ordinal];
                if actual {
                    assert_eq!(
                        persisted, writes,
                        "{case} {name} stage={stage} route={route}"
                    );
                }
                let source_state = run
                    .task_states
                    .iter()
                    .find(|state| state.task_id == source_task.id)
                    .unwrap()
                    .state;
                let source_result = run.phase_results.get(&source_task.id).and_then(|results| {
                    assert_eq!(results.len(), 1);
                    results.first()
                });
                if actual && matches!(case, "cancel" | "obsolete") && ordinal == 0 {
                    assert_eq!(source_state, TaskState::Cancelled);
                    if case == "cancel" {
                        assert!(source_result.is_none());
                        assert!(
                            !run.events
                                .iter()
                                .any(|event| event.task_id.as_ref() == Some(&source_task.id)
                                    && event.kind == SchedulerEventKind::TaskStarted)
                        );
                    }
                    if case == "obsolete" {
                        let cancelled = source_result.unwrap();
                        assert_eq!(cancelled.status, PhaseStatus::Cancelled);
                        assert!(cancelled.output_refs.is_empty());
                        assert!(
                            run.events
                                .iter()
                                .any(|event| event.task_id.as_ref() == Some(&source_task.id)
                                    && event.kind == SchedulerEventKind::TaskStarted)
                        );
                    }
                    continue;
                }
                if case == "provider" {
                    assert_eq!(source_state, TaskState::Blocked);
                    continue;
                }
                assert_eq!(
                    source_state,
                    if hit {
                        TaskState::CacheHit
                    } else {
                        TaskState::Completed
                    }
                );
                assert_eq!(
                    run.events
                        .iter()
                        .any(|event| event.task_id.as_ref() == Some(&source_task.id)
                            && event.kind == SchedulerEventKind::TaskStarted),
                    !hit
                );
                let source_result = source_result.unwrap();
                assert_eq!(source_result.status, PhaseStatus::Complete);
                assert_eq!(source_result.output_refs.len(), 1);
                assert_eq!(source_result.cache_observation.is_some(), !hit);
                let source_ref = &source_result.output_refs[0];
                let source_handle = owner
                    .storage()
                    .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                    .unwrap();
                let source = owner.storage().get(&source_handle).unwrap();
                let expected = &captured
                    .native_sources()
                    .unwrap()
                    .iter()
                    .find(|(_, _, descriptor)| descriptor.package_id.as_str() == *name)
                    .unwrap()
                    .1;
                assert_eq!(source.source_id, version.source_id);
                assert_eq!(source.source_hash, version.source_hash);
                assert_eq!(
                    source.source_text.as_ref(),
                    if *name == "alpha" {
                        raw.trim_start_matches('\u{feff}').replace("\r\n", "\n")
                    } else {
                        "definition\nend;\n".to_owned()
                    }
                );
                assert_eq!(source.line_map, expected.line_map);
                assert_eq!(source.loading_map, expected.loading_map);
                assert!(source_ref.lineage().parents.is_empty());
                let frontend_input = PhaseInput::new(
                    source_task.unit.clone(),
                    PhaseDispatchInputBundle::new(
                        snapshot,
                        source_key(version),
                        vec![],
                        vec![
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                snapshot,
                                source_ref.clone(),
                            )
                            .unwrap(),
                        ],
                    )
                    .unwrap(),
                );
                assert!(matches!(
                    driver
                        .registry()
                        .cache_key_for_phase(PipelinePhase::Frontend, &frontend_input)
                        .unwrap()
                        .intent,
                    PhaseCacheIntent::NoKey { .. }
                ));
                if actual && case == "cancel-frontend" && ordinal == 0 {
                    assert_eq!(
                        run.task_states
                            .iter()
                            .find(|state| state.task_id == frontend_task.id)
                            .unwrap()
                            .state,
                        TaskState::Cancelled
                    );
                    assert_eq!(
                        submission.session.state(),
                        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
                    );
                    assert!(
                        run.task_states
                            .iter()
                            .any(|state| state.state == TaskState::Blocked)
                    );
                    assert!(!store.root().exists());
                    continue;
                }
                let frontend_result = &run.phase_results[&frontend_task.id][0];
                assert!(
                    run.events
                        .iter()
                        .any(|event| event.task_id.as_ref() == Some(&frontend_task.id)
                            && event.kind == SchedulerEventKind::TaskStarted)
                );
                assert!(frontend_result.cache_observation.is_some());
                let frontend = frontend_result.output_refs.first().map(|frontend_ref| {
                    assert_eq!(frontend_ref.lineage().parents, vec![source_ref.output()]);
                    let handle = owner
                        .storage()
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            frontend_ref,
                            &OutputKind::new("FrontendOutput"),
                        )
                        .unwrap();
                    let output = owner.storage().get(&handle).unwrap();
                    assert_eq!(output.source, *source);
                    (
                        output.as_ref().clone(),
                        owner.storage().side_tables(&handle).unwrap(),
                        frontend_ref.content_hash(),
                    )
                });
                let failed = raw.contains("theorem T: ;") && *name == "alpha";
                assert_eq!(
                    frontend_result.status,
                    if failed {
                        PhaseStatus::Recoverable
                    } else {
                        PhaseStatus::Complete
                    }
                );
                assert_eq!(frontend_result.diagnostics.is_empty(), !failed);
                assert_eq!(
                    submission.session.state(),
                    BuildSessionState::Finished(if raw.contains("theorem T: ;") {
                        BuildSessionOutcome::Failed
                    } else {
                        BuildSessionOutcome::Blocked
                    })
                );
                for output in source_result
                    .output_refs
                    .iter()
                    .chain(&frontend_result.output_refs)
                {
                    assert_eq!(
                        matches!(output.placement(), StoragePlacement::Blob { .. }),
                        threshold == 0
                    );
                    owner.storage().validate_handle(output).unwrap();
                }
                let projection = (
                    source_task.id.clone(),
                    frontend_task.id.clone(),
                    source.as_ref().clone(),
                    owner.storage().side_tables(&source_handle).unwrap(),
                    source_ref.content_hash(),
                    frontend,
                    source_result.diagnostics.clone(),
                    frontend_result.diagnostics.clone(),
                );
                if actual {
                    assert_eq!(projection, clean[ordinal]);
                } else {
                    clean.push(projection);
                }
            }
            if actual && !matches!(case, "hit" | "edits" | "store-error") {
                assert!(!store.root().exists());
            }
            if case == "store-error" {
                assert_eq!(fs::read(store.root()).unwrap(), b"existing cache-root file");
            }
            let output_count: usize = run
                .phase_results
                .values()
                .flatten()
                .map(|result| result.output_refs.len())
                .sum();
            let collected = owner.storage().collect(CollectInput {
                snapshot,
                protected_outputs: vec![],
            });
            assert_eq!(collected.retained_outputs, 0);
            assert_eq!(collected.outputs_dropped, output_count);
        }
    }
}

fn native_disk_frontend_lookup_target(
    submission: &mizar_driver::driver::BuildSubmission,
    ids: &InMemorySessionIdAllocator,
    threshold: usize,
) -> (mizar_ir::cache_adapter::IrCacheAdapter, PhaseInput) {
    let (inputs, source_input) = native_disk_lookup_context(submission, ids);
    let owner = publisher(source_input.snapshot, threshold, &source_input.work_unit);
    let result = source_registry()
        .execute_phase_with_resources(
            PipelinePhase::SourceLoad,
            source_input.clone(),
            PhaseExecutionResources {
                diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                    DiagnosticPhase::SourceLoad,
                    source_input.snapshot,
                    "frontend.source_load",
                ))),
                output_publisher: Some(owner.clone()),
                source_load: Some(inputs),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result;
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.output_refs.len(), 1);
    let source = &result.output_refs[0];
    assert_eq!(source.output_kind(), &OutputKind::new("SourceUnit"));
    let parent = SealedParentOutputHandle::from_current_output(
        &owner,
        source_input.snapshot,
        source.clone(),
    )
    .unwrap();
    let input = PhaseInput::new(
        source_input.work_unit.clone(),
        PhaseDispatchInputBundle::new(
            source_input.snapshot,
            source_input.identities().input_hash(),
            vec![],
            vec![parent],
        )
        .unwrap(),
    );
    (mizar_ir::cache_adapter::IrCacheAdapter::new(owner), input)
}

fn persistent_native_frontend_case(
    settings: &[(&str, &str)],
    case: &str,
    stages: &[&str],
    threshold: usize,
    store_path: &str,
    reverse: bool,
) {
    use mizar_build::planner::{lockfile_content_hash, workspace_config_content_hash};
    use mizar_cache::{
        cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
        cache_store::{CacheLookupOutcome, CacheStoreRoot},
    };
    use mizar_ir::cache_adapter::{CacheRehydrateOutcome, IrCacheAdapter};
    let fixture = Fixture::new(stages[0].as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let policy = mizar_proof::policy::VerifierPolicy::release();
    let store = CacheStoreRoot::new(fixture.root.join(store_path));
    assert!(!store.root().exists());
    if case == "store-error" {
        fs::write(store.root(), b"cache-root file").unwrap();
    }
    let mut previous_keys = Vec::new();
    for (stage, raw) in stages.iter().enumerate() {
        for replay in 0..2 {
            let mut captured = None;
            let mut clean = None;
            let mut source_hits = Vec::new();
            let mut frontend_keys = Vec::new();
            for actual in [false, true] {
                let (mut draft, mut input) = fixture.submit_request(vec![], vec![]);
                input.workspace_packages = settings
                    .iter()
                    .map(|(name, build)| WorkspacePackage {
                        member_path: (*name).into(),
                        manifest: parse_package_manifest(&format!(
                            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n{build}"
                        ))
                        .unwrap(),
                    })
                    .collect();
                input.lockfile = parse_lockfile(&format!(
                    "schema_version = 1\n{}",
                    settings.iter().map(|(name, _)| format!(
                        "[[package]]\nname = \"{name}\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"{name}\" }}\ndependencies = []\n"
                    )).collect::<String>()
                )).unwrap();
                let mut layout = settings
                    .iter()
                    .map(|(name, _)| WorkspaceSourcePackage {
                        package_id: PackageId::new(*name),
                        files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
                    })
                    .collect::<Vec<_>>();
                if reverse {
                    input.workspace_packages.reverse();
                    layout.reverse();
                    input.worker_count = 4;
                    input.completion_order = CompletionOrder::Reverse;
                }
                if case == "imports" {
                    layout[0]
                        .files
                        .push(WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"));
                    input.dependency_overlay = complete_leaf_import_overlay();
                }
                input.source_layout = StaticSourceLayout::new(layout);
                input.cache_policy = if case == "disabled" {
                    CacheSchedulingPolicy::Disabled
                } else {
                    CacheSchedulingPolicy::Enabled
                };
                let plan = produce_build_plan(
                    input.plan_request.clone(),
                    input.workspace_packages.clone(),
                    input.lockfile.clone(),
                )
                .unwrap();
                draft.source_inputs.versions.clear();
                draft.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
                draft.verifier_config = VerifierConfigInput::new(if case == "context" {
                    hash(2)
                } else {
                    workspace_config_content_hash(&plan.verifier_config, &plan.build_config)
                });
                let session = if actual {
                    captured.take().unwrap()
                } else {
                    let mut descriptors = Vec::new();
                    for (name, _) in settings {
                        let root = fixture.root.join(name);
                        fs::create_dir_all(root.join("src")).unwrap();
                        for module in if case == "imports" {
                            vec!["main", "leaf"]
                        } else {
                            vec!["main"]
                        } {
                            let path = format!("src/{module}.miz");
                            fs::write(
                                root.join(&path),
                                if *name == "alpha" && module == "main" {
                                    raw.as_bytes()
                                } else {
                                    b"definition\nend;\n"
                                },
                            )
                            .unwrap();
                            descriptors.push((
                                DiskSourceLoader::new(&root),
                                SourceInput {
                                    package_id: PackageId::new(*name),
                                    module_path: ModulePath::new(module),
                                    normalized_path: normalize_path(&root, Path::new(&path))
                                        .unwrap(),
                                    edition: Edition::new("2025"),
                                    origin: SourceOriginInput::Disk { path: path.into() },
                                },
                            ));
                        }
                    }
                    if reverse {
                        descriptors.reverse();
                    }
                    let session = draft
                        .capture_sources(&ids, &snapshots, descriptors)
                        .unwrap();
                    for (name, _) in settings {
                        fs::remove_file(fixture.root.join(name).join("src/main.miz")).unwrap();
                        if case == "imports" {
                            fs::remove_file(fixture.root.join(name).join("src/leaf.miz")).unwrap();
                        }
                    }
                    captured = Some(session.clone());
                    session
                };
                let snapshot = session.captured().snapshot().id;
                let unit = WorkUnit::Module {
                    module: mizar_build::module_index::ModuleId::new(
                        PackageId::new(settings[0].0),
                        ModulePath::new("main"),
                    ),
                };
                let owner = publisher(snapshot, threshold, &unit);
                for (name, _) in settings {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("SourceLoad"),
                        OutputKind::new("SourceUnit"),
                        IrWorkUnit::new(format!("{name:?}:{:?}", "main")),
                    ));
                    allow_frontend(
                        &owner,
                        &WorkUnit::Module {
                            module: mizar_build::module_index::ModuleId::new(
                                PackageId::new(*name),
                                ModulePath::new("main"),
                            ),
                        },
                    );
                }
                if case == "imports" {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("SourceLoad"),
                        OutputKind::new("SourceUnit"),
                        IrWorkUnit::new(format!("{:?}:{:?}", "alpha", "leaf")),
                    ));
                    allow_frontend(
                        &owner,
                        &WorkUnit::Module {
                            module: mizar_build::module_index::ModuleId::new(
                                PackageId::new("alpha"),
                                ModulePath::new("leaf"),
                            ),
                        },
                    );
                }
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_load();
                builder.register_frontend(vec![]);
                let mut driver = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone());
                if case == "provider" {
                    input.phase_dispatch_inputs = Some(Box::new(SuppliedInput::None));
                }
                if actual && matches!(case, "cancel" | "obsolete" | "cancel-frontend") {
                    let (submission, _): &(
                        mizar_driver::driver::BuildSubmission,
                        std::sync::Arc<PhaseOutputPublisher>,
                    ) = clean.as_ref().unwrap();
                    let task = submission
                        .task_graph
                        .as_ref()
                        .unwrap()
                        .tasks()
                        .iter()
                        .find(|task| {
                            task.kind
                                == if case == "cancel-frontend" {
                                    TaskKind::Frontend
                                } else {
                                    TaskKind::SourceLoad
                                }
                        })
                        .unwrap();
                    if case == "obsolete" {
                        input
                            .cancellation
                            .discard_obsolete_completed_task(task.id.clone());
                    } else {
                        input.cancellation.cancel_task(task.id.clone());
                    }
                }
                let submission = if !actual {
                    driver.submit_captured(session, &ids, &snapshots, input)
                } else if case == "source-only" {
                    driver.submit_captured_with_persistent_native_disk_cache(
                        session, &ids, &snapshots, input, &policy, &store,
                    )
                } else if case == "read-only" {
                    driver.submit_captured_with_configured_native_disk_cache(
                        session, &ids, &snapshots, input, &policy, &store,
                    )
                } else {
                    driver.submit_captured_with_persistent_native_prefix_cache(
                        session, &ids, &snapshots, input, &policy, &store,
                    )
                }
                .unwrap();
                let run = submission.scheduler_run.as_ref().unwrap();
                let graph = submission.task_graph.as_ref().unwrap();
                let captured = submission.session.captured();
                let inputs = SourceLoadInputs {
                    snapshot: captured.snapshot(),
                    captured: Some(captured),
                    build_plan: submission.build_plan.as_ref().unwrap(),
                    module_index: submission.module_index.as_ref().unwrap(),
                    allocator: &ids,
                };
                let adapter = IrCacheAdapter::new(owner.clone());
                for (ordinal, (name, build)) in settings.iter().enumerate() {
                    let source_task = graph.tasks().iter().find(|task| task.kind == TaskKind::SourceLoad && matches!(&task.unit, WorkUnit::Module { module } if module.package.as_str() == *name && module.path.as_str() == "main")).unwrap();
                    let frontend_task = graph
                        .tasks()
                        .iter()
                        .find(|task| {
                            task.kind == TaskKind::Frontend && task.unit == source_task.unit
                        })
                        .unwrap();
                    let version = captured
                        .snapshot()
                        .source_versions
                        .iter()
                        .find(|version| {
                            version.package_id.as_str() == *name
                                && version.module_path.as_str() == "main"
                        })
                        .unwrap();
                    let source_input = PhaseInput::new(
                        source_task.unit.clone(),
                        PhaseDispatchInputBundle::without_parent_outputs(
                            snapshot,
                            source_key(version),
                            vec![],
                        ),
                    );
                    let CacheKeyBuildOutcome::Cacheable(source_key) = CacheKeyBuilder::new(
                        inputs
                            .prepare_native_disk_cache_request(&source_input, &policy)
                            .unwrap(),
                    )
                    .build() else {
                        panic!("source key");
                    };
                    let eligible = !matches!(
                        case,
                        "disabled"
                            | "foreign"
                            | "provider"
                            | "context"
                            | "cancel"
                            | "obsolete"
                            | "cancel-frontend"
                            | "store-error"
                            | "read-only"
                    ) && !build.contains("incremental = false")
                        && store.root()
                            == fixture.root.join(name).join(if build.contains("cache/") {
                                if *name == "alpha" {
                                    "cache/a"
                                } else {
                                    "cache/b"
                                }
                            } else {
                                ".mizar-cache"
                            });
                    if !actual {
                        source_hits.push(matches!(
                            store.lookup(&source_key),
                            CacheLookupOutcome::Hit(_)
                        ));
                    }
                    if actual && matches!(case, "cancel" | "obsolete" | "cancel-frontend") {
                        assert!(
                            run.task_states
                                .iter()
                                .any(|state| state.state == TaskState::Cancelled)
                        );
                        assert!(!store.root().exists());
                        continue;
                    }
                    let Some(source_result) = run
                        .phase_results
                        .get(&source_task.id)
                        .and_then(|results| results.first())
                    else {
                        assert_eq!(case, "provider");
                        continue;
                    };
                    assert_eq!(
                        run.task_states
                            .iter()
                            .find(|state| state.task_id == source_task.id)
                            .unwrap()
                            .state,
                        if actual && eligible && source_hits[ordinal] {
                            TaskState::CacheHit
                        } else {
                            TaskState::Completed
                        }
                    );
                    let source_ref = &source_result.output_refs[0];
                    let input = PhaseInput::new(
                        source_task.unit.clone(),
                        PhaseDispatchInputBundle::new(
                            snapshot,
                            source_input.identities().input_hash(),
                            vec![],
                            vec![
                                SealedParentOutputHandle::from_published_output(
                                    &owner,
                                    snapshot,
                                    source_ref.clone(),
                                )
                                .unwrap(),
                            ],
                        )
                        .unwrap(),
                    );
                    for (phase, reason) in [
                        (
                            PipelinePhase::SourceLoad,
                            "current disk source maps require a real source load",
                        ),
                        (
                            PipelinePhase::Frontend,
                            "frontend inputs require a live source and dependency run",
                        ),
                    ] {
                        assert_eq!(
                            driver
                                .registry()
                                .cache_key_for_phase(phase, &input)
                                .unwrap()
                                .intent,
                            PhaseCacheIntent::NoKey {
                                reason: reason.into()
                            }
                        );
                    }
                    let result = &run.phase_results[&frontend_task.id][0];
                    let failed = *name == "alpha" && raw.contains("theorem T: ;");
                    assert_eq!(
                        result.status,
                        if failed {
                            PhaseStatus::Recoverable
                        } else {
                            PhaseStatus::Complete
                        }
                    );
                    if result.output_refs.is_empty() {
                        assert!(!result.diagnostics.is_empty());
                        assert!(!store.root().join("records/Frontend").exists());
                        if actual {
                            assert_eq!(
                                matches!(store.lookup(&source_key), CacheLookupOutcome::Hit(_)),
                                eligible
                            );
                            let (clean_submission, _) = clean.as_ref().unwrap();
                            assert_eq!(
                                submission.session.state(),
                                clean_submission.session.state()
                            );
                            assert_eq!(
                                result.diagnostics,
                                clean_submission
                                    .scheduler_run
                                    .as_ref()
                                    .unwrap()
                                    .phase_results[&frontend_task.id][0]
                                    .diagnostics
                            );
                        }
                        continue;
                    }
                    assert_eq!(
                        run.task_states
                            .iter()
                            .find(|state| state.task_id == frontend_task.id)
                            .unwrap()
                            .state,
                        TaskState::Completed
                    );
                    let handle = owner
                        .storage()
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            &result.output_refs[0],
                            &OutputKind::new("FrontendOutput"),
                        )
                        .unwrap();
                    assert_eq!(frontend_task.phases, [PipelinePhase::Frontend]);
                    let output = owner.storage().get(&handle).unwrap();
                    assert_eq!(
                        handle.lineage().parents.len(),
                        if case == "imports" { 2 } else { 1 }
                    );
                    assert!(handle.lineage().parents.contains(&source_ref.output()));
                    assert_eq!(
                        matches!(handle.placement(), StoragePlacement::Blob { .. }),
                        threshold == 0
                    );
                    if actual {
                        let (clean_submission, clean_owner) = clean.as_ref().unwrap();
                        assert_eq!(submission.session.state(), clean_submission.session.state());
                        assert_eq!(submission.status, clean_submission.status);
                        let clean_result = &clean_submission
                            .scheduler_run
                            .as_ref()
                            .unwrap()
                            .phase_results[&frontend_task.id][0];
                        assert_eq!(result.diagnostics, clean_result.diagnostics);
                        assert_eq!(
                            source_result.diagnostics,
                            clean_submission
                                .scheduler_run
                                .as_ref()
                                .unwrap()
                                .phase_results[&source_task.id][0]
                                .diagnostics
                        );
                        let clean_handle = clean_owner
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                &clean_result.output_refs[0],
                                &OutputKind::new("FrontendOutput"),
                            )
                            .unwrap();
                        assert_eq!(*output, *clean_owner.storage().get(&clean_handle).unwrap());
                        assert_eq!(
                            owner.storage().side_tables(&handle).unwrap(),
                            clean_owner.storage().side_tables(&clean_handle).unwrap()
                        );
                    }
                    if case == "imports" {
                        assert_eq!(frontend_task.dependencies.len(), 3);
                        assert!(
                            inputs
                                .prepare_native_disk_frontend_cache_request(
                                    &input, &policy, &adapter, &handle
                                )
                                .is_err()
                        );
                        if actual {
                            assert!(matches!(
                                store.lookup(&source_key),
                                CacheLookupOutcome::Hit(_)
                            ));
                            assert_eq!(
                                fs::read_dir(store.root().join("records/Frontend"))
                                    .unwrap()
                                    .count(),
                                1
                            );
                        }
                        continue;
                    }
                    if case == "context" {
                        assert!(
                            inputs
                                .prepare_native_disk_frontend_cache_request(
                                    &input, &policy, &adapter, &handle
                                )
                                .is_err()
                        );
                        assert!(!store.root().exists());
                        continue;
                    }
                    let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(
                        inputs
                            .prepare_native_disk_frontend_cache_request(
                                &input, &policy, &adapter, &handle,
                            )
                            .unwrap(),
                    )
                    .build() else {
                        panic!("frontend key");
                    };
                    if !actual {
                        assert!(!store.record_path(&key).exists());
                        if replay == 0 && *name == "alpha" {
                            if stage > 0 {
                                assert_ne!(key, previous_keys[ordinal]);
                            }
                            if stage == 0 {
                                previous_keys.push(key.clone());
                            } else {
                                previous_keys[ordinal] = key.clone();
                            }
                        }
                        frontend_keys.push(key);
                        continue;
                    }
                    assert_eq!(key, frontend_keys[ordinal]);
                    assert_eq!(
                        matches!(store.lookup(&key), CacheLookupOutcome::Hit(_)),
                        eligible && case != "source-only"
                    );
                    assert_eq!(
                        matches!(store.lookup(&source_key), CacheLookupOutcome::Hit(_)),
                        eligible
                    );
                    if eligible && case != "source-only" {
                        if settings.len() == 1 {
                            let (target, target_input) =
                                native_disk_frontend_lookup_target(&submission, &ids, threshold);
                            allow_frontend(target.publisher(), &target_input.work_unit);
                            let CacheRehydrateOutcome::Rehydrated(rebuilt) = inputs
                                .rehydrate_native_disk_frontend_cache(
                                    &target_input,
                                    &policy,
                                    &store,
                                    (&input, &adapter, &handle),
                                    &target,
                                )
                                .unwrap()
                            else {
                                panic!("Driver-produced retained record");
                            };
                            assert_eq!(
                                *target.publisher().storage().get(&rebuilt).unwrap(),
                                *output
                            );
                            assert_eq!(
                                target.publisher().storage().side_tables(&rebuilt).unwrap(),
                                owner.storage().side_tables(&handle).unwrap()
                            );
                            assert_eq!(rebuilt.lineage(), handle.lineage());
                            assert_eq!(target.successful_rehydrations(), 1);
                        }
                        // Force the next real SourceLoad hit to produce the Frontend record anew.
                        fs::remove_file(store.record_path(&key)).unwrap();
                        assert!(!store.record_path(&key).exists());
                    }
                }
                if actual
                    && !matches!(
                        case,
                        "hit" | "edits" | "source-only" | "store-error" | "failed" | "imports"
                    )
                {
                    assert!(!store.root().exists());
                }
                if case == "store-error" {
                    assert_eq!(fs::read(store.root()).unwrap(), b"cache-root file");
                }
                if !actual {
                    clean = Some((submission, owner));
                }
            }
        }
    }
}

fn retained_native_frontend_cache_case(case: &str, threshold: usize, reverse: bool) {
    use mizar_build::{
        planner::{lockfile_content_hash, workspace_config_content_hash},
        scheduler::SchedulerEventKind,
    };
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheStoreRoot};
    use mizar_ir::{
        cache_adapter::{EncodeCacheRecordOutcome, IrCacheAdapter},
        storage::CollectInput,
    };
    let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
    fs::write(
        fixture.root.join("alpha/src/leaf.miz"),
        b"definition\nend;\n",
    )
    .unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let policy = mizar_proof::policy::VerifierPolicy::release();
    let store = CacheStoreRoot::new(fixture.root.join("retained-schedule"));
    let (mut draft, mut warm_input) = fixture.submit_request(vec![], vec![]);
    let files = vec![
        WorkspaceSourceFile::new("src/main.miz", "main.miz"),
        WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
    ];
    warm_input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
        package_id: PackageId::new("alpha"),
        files: files.clone(),
    }]);
    let plan = produce_build_plan(
        warm_input.plan_request.clone(),
        warm_input.workspace_packages.clone(),
        warm_input.lockfile.clone(),
    )
    .unwrap();
    draft.source_inputs.versions.clear();
    draft.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
    draft.verifier_config = VerifierConfigInput::new(workspace_config_content_hash(
        &plan.verifier_config,
        &plan.build_config,
    ));
    let mut descriptors = ["main", "leaf"]
        .into_iter()
        .map(|module| {
            (
                DiskSourceLoader::new(fixture.root.join("alpha")),
                SourceInput {
                    package_id: PackageId::new("alpha"),
                    module_path: ModulePath::new(module),
                    normalized_path: normalize_path(
                        &fixture.root.join("alpha"),
                        Path::new(&format!("src/{module}.miz")),
                    )
                    .unwrap(),
                    edition: Edition::new("2025"),
                    origin: SourceOriginInput::Disk {
                        path: format!("src/{module}.miz").into(),
                    },
                },
            )
        })
        .collect::<Vec<_>>();
    if reverse {
        descriptors.reverse();
    }
    let saved = draft
        .capture_sources(&ids, &snapshots, descriptors)
        .unwrap();
    assert_eq!(saved.state(), BuildSessionState::SnapshotCaptured);
    let captured = saved.captured().clone();
    let snapshot = captured.snapshot().id;
    let units = ["main", "leaf"].map(|module| WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new(module),
        ),
    });
    let make_driver = || {
        let owner = publisher(snapshot, threshold, &units[0]);
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("SourceLoad"),
            OutputKind::new("SourceUnit"),
            IrWorkUnit::new(format!("{:?}:{:?}", "alpha", "leaf")),
        ));
        for unit in &units {
            allow_frontend(&owner, unit);
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        (
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone()),
            owner,
        )
    };
    let (mut warm_driver, warm_owner) = make_driver();
    // Keep the genuine captured session before the ordinary producer finishes it.
    let warm = warm_driver
        .submit_captured(saved.clone(), &ids, &snapshots, warm_input)
        .unwrap();
    let warm_run = warm.scheduler_run.as_ref().unwrap();
    let graph = warm.task_graph.as_ref().unwrap();
    let main_source = graph
        .tasks()
        .iter()
        .find(|task| task.unit == units[0] && task.phases == [PipelinePhase::SourceLoad])
        .unwrap();
    let main_frontend = graph
        .tasks()
        .iter()
        .find(|task| task.unit == units[0] && task.phases == [PipelinePhase::Frontend])
        .unwrap();
    let root = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::PackageResolve)
        .unwrap();
    let inputs = SourceLoadInputs {
        snapshot: captured.snapshot(),
        captured: Some(&captured),
        build_plan: warm.build_plan.as_ref().unwrap(),
        module_index: warm.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let oracle = IrCacheAdapter::new(warm_owner.clone());
    let mut source_paths = Vec::new();
    for task in graph
        .tasks()
        .iter()
        .filter(|task| task.phases == [PipelinePhase::SourceLoad])
    {
        let WorkUnit::Module { module } = &task.unit else {
            unreachable!()
        };
        let version = captured
            .snapshot()
            .source_versions
            .iter()
            .find(|version| version.module_path == module.path)
            .unwrap();
        let input = PhaseInput::new(
            task.unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(snapshot, source_key(version), vec![]),
        );
        let handle = warm_owner
            .storage()
            .typed_handle::<SourceUnit>(
                &warm_run.phase_results[&task.id][0].output_refs[0],
                &OutputKind::new("SourceUnit"),
            )
            .unwrap();
        let EncodeCacheRecordOutcome::Encoded(record) = inputs
            .encode_native_disk_cache_record(&input, &policy, &oracle, handle)
            .unwrap()
        else {
            panic!("genuine scheduled source record")
        };
        source_paths.push((task.id.clone(), store.record_path(&record.header.key)));
        assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
    }
    let main_version = captured
        .snapshot()
        .source_versions
        .iter()
        .find(|version| version.module_path.as_str() == "main")
        .unwrap();
    let oracle_input = PhaseInput::new(
        units[0].clone(),
        PhaseDispatchInputBundle::new(
            snapshot,
            source_key(main_version),
            vec![],
            vec![
                SealedParentOutputHandle::from_published_output(
                    &warm_owner,
                    snapshot,
                    warm_run.phase_results[&main_source.id][0].output_refs[0].clone(),
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    );
    let oracle_handle = warm_owner
        .storage()
        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
            &warm_run.phase_results[&main_frontend.id][0].output_refs[0],
            &OutputKind::new("FrontendOutput"),
        )
        .unwrap();
    let EncodeCacheRecordOutcome::Encoded(mut frontend_record) = inputs
        .encode_native_disk_frontend_cache_record(
            &oracle_input,
            &policy,
            &oracle,
            oracle_handle.clone(),
        )
        .unwrap()
    else {
        panic!("genuine scheduled Frontend record")
    };
    let frontend_path = store.record_path(&frontend_record.header.key);
    assert_eq!(
        store.insert(&frontend_record).unwrap(),
        CacheInsertOutcome::Inserted
    );
    // Native capture is immutable and no phase can reread these bodies.
    for module in ["main", "leaf"] {
        fs::remove_file(fixture.root.join(format!("alpha/src/{module}.miz"))).unwrap();
    }
    for replay in 0..if case == "hit" { 3 } else { 2 } {
        let negative = replay > 0 && case != "hit";
        if replay == 1 {
            match case {
                "source-miss" => fs::remove_file(
                    &source_paths
                        .iter()
                        .find(|(id, _)| id == &main_source.id)
                        .unwrap()
                        .1,
                )
                .unwrap(),
                "frontend-miss" => fs::remove_file(&frontend_path).unwrap(),
                "source-corrupt" => fs::write(
                    &source_paths
                        .iter()
                        .find(|(id, _)| id == &main_source.id)
                        .unwrap()
                        .1,
                    b"invalid record",
                )
                .unwrap(),
                "frontend-corrupt" => fs::write(&frontend_path, b"invalid record").unwrap(),
                "producer" => {
                    frontend_record.header.produced_by[0].value =
                        "negative-only-other-producer".into();
                    let alternate = CacheStoreRoot::new(fixture.root.join("bad-producer"));
                    alternate.insert(&frontend_record).unwrap();
                    fs::copy(
                        alternate.record_path(&frontend_record.header.key),
                        &frontend_path,
                    )
                    .unwrap();
                }
                "oracle-stale" => {
                    warm_owner.mark_obsolete(snapshot).unwrap();
                }
                "oracle-collected" => {
                    let collected = warm_owner.storage().collect(CollectInput {
                        snapshot,
                        protected_outputs: vec![],
                    });
                    assert_eq!(collected.outputs_dropped, 4);
                    assert!(
                        warm_owner
                            .storage()
                            .validate_handle(oracle_handle.any())
                            .is_err()
                    );
                }
                _ => {}
            }
        }
        let (_, mut input) = fixture.submit_request(vec![], vec![]);
        let mut ordered_files = files.clone();
        if reverse {
            ordered_files.reverse();
        }
        input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: ordered_files,
        }]);
        input.worker_count = if reverse { 4 } else { 1 };
        input.completion_order = if reverse {
            CompletionOrder::Reverse
        } else {
            CompletionOrder::Canonical
        };
        if negative && case == "graph" {
            input.dependency_overlay = complete_leaf_import_overlay();
        }
        if negative && case == "context" {
            input.workspace_packages[0].manifest.verifier.max_axioms += 1;
        }
        let (mut clean_driver, clean_owner) = make_driver();
        let (_, mut clean_input) = fixture.submit_request(vec![], vec![]);
        clean_input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: files.clone(),
        }]);
        clean_input.dependency_overlay = input.dependency_overlay.clone();
        clean_input.workspace_packages = input.workspace_packages.clone();
        let clean = clean_driver
            .submit_captured(saved.clone(), &ids, &snapshots, clean_input)
            .unwrap();
        let (mut driver, mut owner) = make_driver();
        if negative && case == "duplicate" {
            owner = warm_owner.clone();
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            driver =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        }
        input.cache_policy = if negative && case == "disabled" {
            CacheSchedulingPolicy::Disabled
        } else {
            CacheSchedulingPolicy::Enabled
        };
        if negative {
            match case {
                "provider" => input.phase_dispatch_inputs = Some(Box::new(SuppliedInput::None)),
                "cancel-source" => input.cancellation.cancel_task(main_source.id.clone()),
                "cancel-frontend" => input.cancellation.cancel_task(main_frontend.id.clone()),
                "cancel-root" => input.cancellation.cancel_task(root.id.clone()),
                "obsolete-source" => input
                    .cancellation
                    .discard_obsolete_completed_task(main_source.id.clone()),
                "obsolete-frontend" => input
                    .cancellation
                    .discard_obsolete_completed_task(main_frontend.id.clone()),
                "resource" => input.resource_budget.module_workers = 0,
                _ => {}
            }
        }
        let actual_oracle_result = &warm_run.phase_results[&main_frontend.id][0];
        let mut malformed_result = actual_oracle_result.clone();
        if negative {
            match case {
                "oracle-status" => malformed_result.status = PhaseStatus::Recoverable,
                "oracle-empty" => malformed_result.output_refs.clear(),
                "oracle-multiple" => malformed_result.output_refs.push(oracle_handle.erase()),
                "oracle-diagnostics" => malformed_result.diagnostics.push(
                    DiagnosticSink::new(DiagnosticProducerScope::new(
                        DiagnosticPhase::Frontend,
                        snapshot,
                        "frontend.run_loaded",
                    ))
                    .seal(),
                ),
                _ => {}
            }
        }
        let selected_result = if negative
            && matches!(
                case,
                "oracle-status" | "oracle-empty" | "oracle-multiple" | "oracle-diagnostics"
            ) {
            &malformed_result
        } else {
            actual_oracle_result
        };
        let mut selected_input = oracle_input.clone();
        if negative && case == "binding" {
            selected_input.work_unit = units[1].clone();
        }
        let foreign = IrCacheAdapter::new(publisher(snapshot, threshold, &units[0]));
        let selected_oracle = if negative && case == "oracle-foreign" {
            &foreign
        } else {
            &oracle
        };
        let error_store = CacheStoreRoot::new(fixture.root.join("store-error"));
        if negative && case == "store-error" && !error_store.root().exists() {
            fs::write(error_store.root(), b"cache root file").unwrap();
        }
        let selected_store = if negative && case == "store-error" {
            &error_store
        } else {
            &store
        };
        let record_bytes = source_paths
            .iter()
            .map(|(_, path)| fs::read(path).ok())
            .chain(std::iter::once(fs::read(&frontend_path).ok()))
            .collect::<Vec<_>>();
        let submission = driver
            .submit_captured_with_retained_native_frontend_cache(
                saved.clone(),
                &ids,
                &snapshots,
                input,
                (&policy, selected_store),
                (&selected_input, selected_oracle, selected_result),
            )
            .unwrap();
        assert_eq!(submission.session.captured(), &captured);
        assert_eq!(saved.state(), BuildSessionState::SnapshotCaptured);
        assert_eq!(
            record_bytes,
            source_paths
                .iter()
                .map(|(_, path)| fs::read(path).ok())
                .chain(std::iter::once(fs::read(&frontend_path).ok()))
                .collect::<Vec<_>>()
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        let state = |id: &mizar_build::task_graph::TaskId| {
            run.task_states
                .iter()
                .find(|record| &record.task_id == id)
                .unwrap()
                .state
        };
        let started = |id: &mizar_build::task_graph::TaskId| {
            run.events.iter().any(|event| {
                event.task_id.as_ref() == Some(id) && event.kind == SchedulerEventKind::TaskStarted
            })
        };
        let suppress = negative
            && matches!(
                case,
                "provider"
                    | "duplicate"
                    | "cancel-source"
                    | "cancel-frontend"
                    | "cancel-root"
                    | "obsolete-source"
                    | "obsolete-frontend"
            );
        let source_hit = !negative
            || !matches!(
                case,
                "source-miss" | "source-corrupt" | "disabled" | "context" | "store-error"
            );
        let frontend_hit = !negative;
        if negative && case == "resource" {
            assert_eq!(state(&main_source.id), TaskState::CacheHit);
            assert_eq!(state(&main_frontend.id), TaskState::CacheHit);
            assert!(!started(&main_source.id));
            assert!(!started(&main_frontend.id));
            assert!(run.diagnostics.iter().any(|diagnostic| diagnostic.kind
                == mizar_build::scheduler::SchedulerDiagnosticKind::ImpossibleResourceRequest));
            assert!(
                run.task_states
                    .iter()
                    .any(|record| record.state == TaskState::Blocked)
            );
        } else if !suppress {
            assert_eq!(
                state(&main_source.id),
                if source_hit {
                    TaskState::CacheHit
                } else {
                    TaskState::Completed
                },
                "{case}/{replay}"
            );
            assert_eq!(
                state(&main_frontend.id),
                if frontend_hit {
                    TaskState::CacheHit
                } else if case == "graph" {
                    assert_eq!(
                        clean
                            .scheduler_run
                            .as_ref()
                            .unwrap()
                            .task_states
                            .iter()
                            .find(|record| record.task_id == main_frontend.id)
                            .unwrap()
                            .state,
                        TaskState::Blocked
                    );
                    TaskState::Blocked
                } else {
                    TaskState::Completed
                },
                "{case}/{replay}"
            );
            assert_eq!(started(&main_source.id), !source_hit);
            assert_eq!(started(&main_frontend.id), !frontend_hit);
            assert_eq!(submission.status, clean.status);
            assert_eq!(submission.session.state(), clean.session.state());
            assert_eq!(
                run.phase_results.len(),
                clean.scheduler_run.as_ref().unwrap().phase_results.len()
            );
            for (task, results) in &run.phase_results {
                let expected = &clean.scheduler_run.as_ref().unwrap().phase_results[task];
                assert_eq!(results.len(), expected.len());
                for (result, expected) in results.iter().zip(expected) {
                    assert_eq!(result.status, expected.status);
                    assert_eq!(result.diagnostics, expected.diagnostics);
                    assert_eq!(result.output_refs.len(), expected.output_refs.len());
                    if state(task) == TaskState::CacheHit {
                        assert!(result.cache_observation.is_none());
                    }
                    for (output, expected) in result.output_refs.iter().zip(&expected.output_refs) {
                        assert_eq!(output.output(), expected.output());
                        assert_eq!(output.content_hash(), expected.content_hash());
                        assert_eq!(output.side_table_hash(), expected.side_table_hash());
                        assert_eq!(output.lineage(), expected.lineage());
                        assert_eq!(
                            matches!(output.placement(), StoragePlacement::Blob { .. }),
                            threshold == 0
                        );
                        if output.output_kind() == &OutputKind::new("SourceUnit") {
                            let actual = owner
                                .storage()
                                .typed_handle::<SourceUnit>(output, &OutputKind::new("SourceUnit"))
                                .unwrap();
                            let expected = clean_owner
                                .storage()
                                .typed_handle::<SourceUnit>(
                                    expected,
                                    &OutputKind::new("SourceUnit"),
                                )
                                .unwrap();
                            assert_eq!(
                                *owner.storage().get(&actual).unwrap(),
                                *clean_owner.storage().get(&expected).unwrap()
                            );
                            assert_eq!(
                                owner.storage().side_tables(&actual).unwrap(),
                                clean_owner.storage().side_tables(&expected).unwrap()
                            );
                        } else {
                            let actual = owner.storage().typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(output, &OutputKind::new("FrontendOutput")).unwrap();
                            let expected = clean_owner.storage().typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(expected, &OutputKind::new("FrontendOutput")).unwrap();
                            assert_eq!(
                                *owner.storage().get(&actual).unwrap(),
                                *clean_owner.storage().get(&expected).unwrap()
                            );
                            assert_eq!(
                                owner.storage().side_tables(&actual).unwrap(),
                                clean_owner.storage().side_tables(&expected).unwrap()
                            );
                        }
                    }
                }
            }
            let task = submission
                .task_graph
                .as_ref()
                .unwrap()
                .tasks()
                .iter()
                .find(|task| task.id == main_frontend.id)
                .unwrap();
            assert_eq!(
                task.dependencies.len(),
                if negative && case == "graph" { 3 } else { 2 }
            );
            assert!(task.dependencies.contains(&main_source.id));
            assert!(task.dependencies.contains(&root.id));
        } else {
            assert_ne!(state(&main_frontend.id), TaskState::CacheHit, "{case}");
            assert!(!run.phase_results.contains_key(&main_frontend.id));
            if matches!(
                case,
                "cancel-source" | "cancel-root" | "obsolete-source" | "provider"
            ) {
                assert!(!run.phase_results.contains_key(&main_source.id), "{case}");
            }
            if case == "duplicate" {
                assert_eq!(state(&main_source.id), TaskState::Blocked);
                let result = &run.phase_results[&main_source.id][0];
                assert_eq!(result.status, PhaseStatus::Blocking);
                assert!(result.output_refs.is_empty());
                assert!(started(&main_source.id));
            }
            assert!(run.task_states.iter().any(|record| matches!(
                record.state,
                TaskState::Cancelled | TaskState::Failed | TaskState::Blocked
            )));
        }
        // Exactly prepared payloads may remain physically stored after a rejected skip.
        let collected = owner.storage().collect(CollectInput {
            snapshot,
            protected_outputs: vec![],
        });
        assert_eq!(collected.retained_outputs, 0, "{case}/{replay}");
        assert_eq!(
            collected.outputs_dropped,
            if suppress && case == "provider" {
                0
            } else if negative && matches!(case, "cancel-root" | "resource" | "graph") {
                3
            } else {
                4
            },
            "{case}/{replay}"
        );
        for output in run
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
        {
            assert!(owner.storage().validate_handle(output).is_err());
        }
    }
}

fn import_discovered_native_index_driver(
    snapshot: BuildSnapshotId,
    modules: &[&str],
) -> (CompilerDriver, std::sync::Arc<PhaseOutputPublisher>) {
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new(modules[0]),
        ),
    };
    let owner = publisher(snapshot, 0, &unit);
    for module in modules {
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new(*module),
            ),
        };
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new("SourceLoad"),
            OutputKind::new("SourceUnit"),
            IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
        ));
        allow_frontend(&owner, &unit);
    }
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(vec![]);
    (
        CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone()),
        owner,
    )
}

fn with_native_summary_parent(
    text: &str,
    open: bool,
    threshold: usize,
    check: impl FnOnce(
        &Fixture,
        SourceLoadInputs<'_>,
        &PhaseInput,
        &std::sync::Arc<PhaseOutputPublisher>,
        &[AnyPhaseOutputRef],
        (
            &mizar_driver::request::BuildSession,
            &InMemorySessionIdAllocator,
            &SnapshotRegistry<InMemorySessionIdAllocator>,
        ),
    ),
) {
    let fixture = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut draft, submit) = fixture.submit_request(vec![], vec![]);
    draft.source_inputs.versions.clear();
    let descriptor = SourceInput {
        package_id: PackageId::new("alpha"),
        module_path: ModulePath::new("main"),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: Edition::new("2025"),
        origin: if open {
            SourceOriginInput::OpenBuffer {
                uri: format!(
                    "file://{}",
                    fixture.root.join("alpha/src/main.miz").display()
                ),
                expected_version: 7,
                actual_version: 7,
                text: std::sync::Arc::from(text),
            }
        } else {
            SourceOriginInput::Disk {
                path: "src/main.miz".into(),
            }
        },
    };
    if open {
        fixture.write(&[0xff]);
    }
    let session = draft
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                descriptor,
            )],
        )
        .unwrap();
    let snapshot = session.captured().snapshot().id;
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        ),
    };
    let owner = publisher(snapshot, threshold, &unit);
    allow_frontend(&owner, &unit);
    owner.allow_work_unit(AllowedWorkUnit::new(
        IrPipelinePhase::new("ModuleResolve"),
        OutputKind::new("SourceTheoremModuleCandidates"),
        IrWorkUnit::new("\"alpha\":\"main\""),
    ));
    let mut builder = PhaseRegistryBuilder::new();
    builder
        .register_source_load()
        .register_frontend(vec![])
        .register_source_theorem_module_resolver();
    let submission = CompilerDriver::new(builder.build().unwrap())
        .with_output_publisher(owner.clone())
        .submit_captured(session.clone(), &ids, &snapshots, submit)
        .unwrap();
    let outputs = submission
        .scheduler_run
        .as_ref()
        .unwrap()
        .phase_results
        .values()
        .flatten()
        .flat_map(|result| result.output_refs.iter().cloned())
        .collect::<Vec<_>>();
    let parent = outputs
        .iter()
        .find(|output| output.output_kind() == &OutputKind::new("SourceTheoremModuleCandidates"))
        .unwrap();
    let (inputs, source_input) = native_disk_lookup_context(&submission, &ids);
    let input = PhaseInput::new(
        unit,
        PhaseDispatchInputBundle::new(
            snapshot,
            source_input.identities().input_hash(),
            vec![],
            vec![
                SealedParentOutputHandle::from_published_output(&owner, snapshot, parent.clone())
                    .unwrap(),
            ],
        )
        .unwrap(),
    );
    let WorkUnit::Module { module } = &input.work_unit else {
        unreachable!()
    };
    assert!(
        inputs
            .native_theorem_module_candidates(module)
            .unwrap()
            .is_some()
    );
    assert!(
        inputs
            .native_theorem_module_summary_candidate(module)
            .unwrap()
            .is_some()
    );
    check(
        &fixture,
        inputs,
        &input,
        &owner,
        &outputs,
        (&session, &ids, &snapshots),
    );
}

fn with_native_theorem_witness_kernel(
    text: &str,
    open: bool,
    threshold: usize,
    check: impl FnOnce(
        &Fixture,
        SourceLoadInputs<'_>,
        &PhaseInput,
        &std::sync::Arc<PhaseOutputPublisher>,
        &[AnyPhaseOutputRef],
        (
            &mizar_driver::request::BuildSession,
            &InMemorySessionIdAllocator,
            &SnapshotRegistry<InMemorySessionIdAllocator>,
        ),
    ),
) {
    use mizar_atp::problem::*;
    use mizar_build::task_graph::{
        EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion},
    };
    with_native_summary_parent(
        text,
        open,
        threshold,
        |fixture, _, old, _, _, (session, ids, snapshots)| {
            let owner = publisher(old.snapshot, threshold, &old.work_unit);
            allow_frontend(&owner, &old.work_unit);
            for (phase, kind) in [
                ("ModuleResolve", "SourceTheoremModuleCandidates"),
                ("SignatureCollection", "SourceTheoremModuleSummary"),
                ("TypeChecking", "SourceTheoremTypedAst"),
                ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                ("OverloadResolution", "SourceTheoremMetadata"),
                ("Elaboration", "SourceTheoremCore"),
                ("AlgorithmPreparation", "SourceTheoremPreparation"),
                ("VcGenerate", "SourceTheoremVC"),
                ("VcDischarge", "SourceTheoremDischarge"),
                ("AtpSolve", "SourceTheoremCandidateInputs"),
                ("KernelCheck", "SourceTheoremCandidateCheck"),
            ] {
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new(phase),
                    OutputKind::new(kind),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                ));
            }
            let policy = DischargePolicy {
                computation_limit: ComputationLimit {
                    policy: PolicyKey::new("explicit"),
                    max_steps: 64,
                },
            };
            let profile = LogicProfile::try_new(
                "witness-data",
                if open {
                    LogicFragment::SmtLibUninterpreted
                } else {
                    LogicFragment::Fof
                },
                EqualitySupport::Supported,
                QuantifierPolicy::FirstOrder,
                SoftTypeStrategy::ScopedGuardPredicates,
                NativePropertySupport::Supported,
                [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
                    .into_iter()
                    .collect(),
            )
            .unwrap();
            let format = if open {
                ConcreteFormat::SmtLib
            } else {
                ConcreteFormat::Tptp
            };
            let mut builder = PhaseRegistryBuilder::new();
            builder
                .register_source_load()
                .register_frontend(vec![])
                .register_source_theorem_module_resolver()
                .register_source_theorem_signature_collection()
                .register_source_theorem_type_checking()
                .register_source_theorem_registration_resolution()
                .register_source_theorem_overload_resolution()
                .register_source_theorem_elaboration()
                .register_source_theorem_algorithm_preparation()
                .register_source_theorem_vc_generation(
                    GenerationSchemaVersion::new("λ\n\0"),
                    VcSchemaVersion::new(""),
                )
                .register_source_theorem_vc_discharge(policy.clone())
                .register_source_theorem_atp_solve(policy.clone(), profile.clone(), format)
                .unwrap()
                .register_source_theorem_candidate_check(
                    policy,
                    profile,
                    format,
                    mizar_proof::policy::VerifierPolicy::default(),
                )
                .unwrap();
            let WorkUnit::Module { module } = &old.work_unit else {
                panic!()
            };
            let (_, mut submit) = fixture.submit_request(vec![], vec![]);
            submit.vc_descriptors = vec![VcTaskDescriptor::new(
                VcTaskDescriptorId::new("opaque-witness-data"),
                module.clone(),
                VcOrderKey::new("order"),
                vec![],
                vec![EvidenceCandidateId::new("opaque-evidence")],
            )];
            let submission = CompilerDriver::new(builder.build().unwrap())
                .with_output_publisher(owner.clone())
                .submit_captured(session.clone(), ids, snapshots, submit)
                .unwrap();
            let (inputs, original) = native_disk_lookup_context(&submission, ids);
            let refs = submission
                .scheduler_run
                .as_ref()
                .unwrap()
                .phase_results
                .values()
                .flatten()
                .flat_map(|row| row.output_refs.iter().cloned())
                .collect::<Vec<_>>();
            assert_eq!(refs.len(), 12);
            let parents = [
                "SourceTheoremCore",
                "SourceTheoremVC",
                "SourceTheoremDischarge",
                "SourceTheoremCandidateCheck",
            ]
            .map(|kind| {
                refs.iter()
                    .find(|row| row.output_kind() == &OutputKind::new(kind))
                    .unwrap()
            });
            let four = PhaseInput::new(
                original.work_unit.clone(),
                PhaseDispatchInputBundle::new(
                    old.snapshot,
                    original.identities().input_hash(),
                    original.identities().dependency_hashes().to_vec(),
                    parents
                        .into_iter()
                        .map(|row| {
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                old.snapshot,
                                row.clone(),
                            )
                            .unwrap()
                        })
                        .collect(),
                )
                .unwrap(),
            );
            check(
                fixture,
                inputs,
                &four,
                &owner,
                &refs,
                (session, ids, snapshots),
            );
        },
    );
}

fn with_native_theorem_artifact_context(
    text: &str,
    open: bool,
    threshold: usize,
    file_fault: &str,
    consume: impl FnOnce(
        &Fixture,
        SourceLoadInputs<'_>,
        &PhaseInput,
        &std::sync::Arc<PhaseOutputPublisher>,
        &[AnyPhaseOutputRef],
        &Option<Result<mizar_ir::projection::VerifiedArtifactDraft, String>>,
        (
            &mizar_checker::type_checker::TermFormulaInferenceOutput,
            &mizar_checker::typed_ast::TypedAst,
            &mizar_checker::resolved_typed_ast::ResolvedTypedAst,
            &str,
        ),
    ),
) {
    use mizar_artifact::{
        module_summary::{MODULE_SUMMARY_SCHEMA_FAMILY, ModuleSummary, SourceRangeSummary},
        registration_summary::{ArtifactHashClass, ArtifactHashRef},
        store::{
            CanonicalHashDomain, CanonicalJson, HashClass, PublishedArtifactPath,
            PublishedArtifactReadOptions, SchemaVersion as ArtifactVersion, artifact_hash_domain,
            read_published_artifact, write_published_artifact,
        },
        verified_artifact::{
            BuildProvenance, ExportVisibility, ExpressionMetadata, ObligationMetadata,
            ObligationStatus, StatementMetadata, VerifiedExport,
        },
    };
    use mizar_atp::problem::*;
    use mizar_build::{
        planner::{lockfile_content_hash, workspace_config_content_hash},
        task_graph::{EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId},
    };
    use mizar_checker::type_checker::{
        NormalizedTypeStatus, SourceVariableSemanticsChecker, TypeHeadRef,
        check_source_theorem_metadata,
    };
    use mizar_ir::projection::{ArtifactProjectionService, ProjectVerifiedArtifactInput};
    use mizar_proof::{
        policy::{KernelEvidenceOrigin, KernelPolicyInput, PolicyDecision, VerifierPolicy},
        selection::{
            CandidateSourceId, ProofEvidenceCandidate, ProofEvidenceSet, VcProofSelection,
            merge_artifact_proof_selections as merge, select_winner,
        },
        status::{
            ArtifactStatusPublication, CurrentArtifactObligationStatus, ProofStatusProjectionInput,
            TrustedUsedAxiomsRef, project_status,
            source_theorem_candidate_witness_file_candidate as fresh,
        },
    };
    use mizar_resolve::{
        names::{SourceVariableScopeInput, SourceVariableScopeResolver},
        symbols::{source_proof_inputs, source_theorem_module_original_inputs},
    };
    use mizar_vc::{
        dependency_slice::DEPENDENCY_SLICE_SCHEMA_VERSION,
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion, VcSet},
    };
    let fixture = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut request, mut submit) = fixture.submit_request(vec![], vec![]);
    if threshold == 0 {
        submit.lockfile = parse_lockfile("schema_version = 1\ntoolchain_version = \"1.2.3\"\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n").unwrap();
        submit.plan_request.toolchain = ToolchainInfo::new("1.2.3");
    }
    let plan = produce_build_plan(
        submit.plan_request.clone(),
        submit.workspace_packages.clone(),
        submit.lockfile.clone(),
    )
    .unwrap();
    request.dependency_inputs.lockfile_hash = lockfile_content_hash(&plan.lockfile);
    request.dependency_inputs.toolchain = plan.toolchain.clone();
    request.verifier_config = VerifierConfigInput::new(workspace_config_content_hash(
        &plan.verifier_config,
        &plan.build_config,
    ));
    request.source_inputs.versions.clear();
    let source_input = SourceInput {
        package_id: PackageId::new("alpha"),
        module_path: ModulePath::new("main"),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: Edition::new("2025"),
        origin: if open {
            SourceOriginInput::OpenBuffer {
                uri: format!(
                    "file://{}",
                    fixture.root.join("alpha/src/main.miz").display()
                ),
                expected_version: 7,
                actual_version: 7,
                text: text.into(),
            }
        } else {
            SourceOriginInput::Disk {
                path: "src/main.miz".into(),
            }
        },
    };
    let session = request
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                source_input,
            )],
        )
        .unwrap();
    let snapshot = session.captured().snapshot();
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            PackageId::new("alpha"),
            ModulePath::new("main"),
        ),
    };
    let owner = publisher(snapshot.id, threshold, &unit);
    allow_frontend(&owner, &unit);
    for (phase, kind) in [
        ("ModuleResolve", "SourceTheoremModuleCandidates"),
        ("SignatureCollection", "SourceTheoremModuleSummary"),
        ("TypeChecking", "SourceTheoremTypedAst"),
        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
        ("OverloadResolution", "SourceTheoremMetadata"),
        ("Elaboration", "SourceTheoremCore"),
        ("AlgorithmPreparation", "SourceTheoremPreparation"),
        ("VcGenerate", "SourceTheoremVC"),
        ("VcDischarge", "SourceTheoremDischarge"),
        ("AtpSolve", "SourceTheoremCandidateInputs"),
        ("KernelCheck", "SourceTheoremCandidateCheck"),
        ("KernelCheck", "SourceTheoremCandidateWitnessFile"),
    ] {
        owner.allow_work_unit(AllowedWorkUnit::new(
            IrPipelinePhase::new(phase),
            OutputKind::new(kind),
            IrWorkUnit::new("\"alpha\":\"main\""),
        ));
    }
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("explicit"),
            max_steps: 64,
        },
    };
    let verifier = if threshold == 0 {
        VerifierPolicy::default()
            .with_profile_id("artifact λ\0")
            .with_checker_schema_version(7)
    } else {
        VerifierPolicy::default()
    };
    let profile = LogicProfile::try_new(
        "artifact-source",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Supported,
        [ConcreteFormat::Tptp, ConcreteFormat::SmtLib]
            .into_iter()
            .collect(),
    )
    .unwrap();
    let generation = GenerationSchemaVersion::new("λ\n\0");
    let schema = VcSchemaVersion::new("");
    let mut builder = PhaseRegistryBuilder::new();
    builder
        .register_source_load()
        .register_frontend(vec![])
        .register_source_theorem_module_resolver()
        .register_source_theorem_signature_collection()
        .register_source_theorem_type_checking()
        .register_source_theorem_registration_resolution()
        .register_source_theorem_overload_resolution()
        .register_source_theorem_elaboration()
        .register_source_theorem_algorithm_preparation()
        .register_source_theorem_vc_generation(generation.clone(), schema.clone())
        .register_source_theorem_vc_discharge(policy.clone())
        .register_source_theorem_atp_solve(policy.clone(), profile.clone(), ConcreteFormat::Tptp)
        .unwrap()
        .register_source_theorem_candidate_check(
            policy.clone(),
            profile.clone(),
            ConcreteFormat::Tptp,
            verifier.clone(),
        )
        .unwrap();
    let WorkUnit::Module { module } = &unit else {
        panic!()
    };
    submit.vc_descriptors = vec![VcTaskDescriptor::new(
        VcTaskDescriptorId::new("source-artifact"),
        module.clone(),
        VcOrderKey::new("order"),
        vec![],
        vec![EvidenceCandidateId::new("actual-evidence")],
    )];
    let submission = CompilerDriver::new(builder.build().unwrap())
        .with_output_publisher(owner.clone())
        .submit_captured(session.clone(), &ids, &snapshots, submit)
        .unwrap();
    let (inputs, original) = native_disk_lookup_context(&submission, &ids);
    assert_eq!(inputs.validate_native_lock_content(), Ok(()));
    assert_eq!(inputs.validate_native_config_content(), Ok(()));
    let mut refs = submission
        .scheduler_run
        .as_ref()
        .unwrap()
        .phase_results
        .values()
        .flatten()
        .flat_map(|row| row.output_refs.iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(refs.len(), 12);
    let parent = |kind: &str| {
        refs.iter()
            .find(|row| row.output_kind() == &OutputKind::new(kind))
            .unwrap()
            .clone()
    };
    let summary_ref = parent("SourceTheoremModuleSummary");
    let core_ref = parent("SourceTheoremCore");
    let vc_ref = parent("SourceTheoremVC");
    let d_ref = parent("SourceTheoremDischarge");
    let kernel_ref = parent("SourceTheoremCandidateCheck");
    let frontend_ref = parent("FrontendOutput");
    let vcs = owner
        .storage()
        .get(
            &owner
                .storage()
                .typed_handle::<VcSet>(&vc_ref, vc_ref.output_kind())
                .unwrap(),
        )
        .unwrap();
    let id = vcs.vcs()[0].id;
    let four = PhaseInput::new(
        original.work_unit.clone(),
        PhaseDispatchInputBundle::new(
            snapshot.id,
            original.identities().input_hash(),
            original.identities().dependency_hashes().to_vec(),
            [&core_ref, &vc_ref, &d_ref, &kernel_ref]
                .into_iter()
                .map(|row| {
                    SealedParentOutputHandle::from_published_output(
                        &owner,
                        snapshot.id,
                        row.clone(),
                    )
                    .unwrap()
                })
                .collect(),
        )
        .unwrap(),
    );
    let selected = inputs
        .native_theorem_candidate_selection_with_core_vc_discharge_check_parents(
            &four,
            &owner,
            id,
            (profile.clone(), ConcreteFormat::Tptp),
            (&policy, &verifier),
        )
        .unwrap()
        .unwrap();
    let frontend = owner
        .storage()
        .get(
            &owner
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    &frontend_ref,
                    frontend_ref.output_kind(),
                )
                .unwrap(),
        )
        .unwrap();
    let ast = frontend.ast.as_ref().unwrap();
    assert_eq!(ast.source_id, snapshot.source_versions[0].source_id);
    let source_text = frontend.source.source_text.as_ref();
    let resolver_module = ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main"));
    let (arena, env) = source_theorem_module_original_inputs(ast, &resolver_module)
        .unwrap()
        .unwrap();
    let variable_scope = SourceVariableScopeResolver::resolve_proof_occurrences(
        SourceVariableScopeInput::new(ast, &resolver_module, &env),
    )
    .unwrap();
    let metadata = check_source_theorem_metadata(&arena, &env, &variable_scope).unwrap();
    let (scope, labels, resolved) = source_proof_inputs(&arena, &env).unwrap();
    let check = SourceVariableSemanticsChecker::check_theorem_skeletons(
        &arena,
        &metadata.2,
        &scope,
        &env,
        &labels,
        &resolved,
        None,
    )
    .unwrap();
    let witness_path = "proof-witnesses/checked.json";
    let actual68 = fresh(
        &arena,
        &check,
        &env,
        snapshot,
        id,
        (&generation, &schema, profile.clone(), ConcreteFormat::Tptp),
        (&policy, &verifier),
        witness_path,
    )
    .unwrap();
    assert_eq!(
        inputs
            .native_theorem_candidate_witness_file_with_core_vc_discharge_check_parents(
                &four,
                &owner,
                id,
                (profile.clone(), ConcreteFormat::Tptp),
                (&policy, &verifier),
                witness_path
            )
            .unwrap(),
        Some((actual68.1.clone(), actual68.2.clone()))
    );
    let file = if file_fault.is_empty() {
        let file = inputs
            .publish_native_theorem_candidate_witness_file(
                &four,
                &owner,
                id,
                (profile.clone(), ConcreteFormat::Tptp),
                (&policy, &verifier),
                witness_path,
            )
            .unwrap()
            .unwrap();
        assert_eq!(*owner.storage().get(&file).unwrap(), actual68.1);
        file
    } else {
        use mizar_ir::{
            publisher::{OutputOrigin, PublicationTarget, PublishOutputInput},
            storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion},
        };
        let value = if file_fault == "bytes" {
            CanonicalJson::Null
        } else {
            actual68.1.clone()
        };
        let bytes = mizar_artifact::store::canonical_json_bytes(&value);
        owner
            .publish(PublishOutputInput {
                slot: owner.allocate(
                    snapshot.id,
                    IrPipelinePhase::new("KernelCheck"),
                    IrWorkUnit::new("\"alpha\":\"main\""),
                    OutputKind::new("SourceTheoremCandidateWitnessFile"),
                    SchemaVersion::new(1),
                ),
                snapshot: snapshot.id,
                phase: IrPipelinePhase::new("KernelCheck"),
                work_unit: IrWorkUnit::new("\"alpha\":\"main\""),
                output_kind: OutputKind::new("SourceTheoremCandidateWitnessFile"),
                schema_version: SchemaVersion::new(1),
                payload: value,
                canonical_payload: Some(bytes.clone()),
                storage_payload: Some(bytes),
                decode: BlobDecoder::new(|bytes| {
                    mizar_artifact::store::parse_canonical_json_bytes(bytes, "negative-file")
                        .map_err(|error| BlobDecodeError::new(error.to_string()))
                }),
                parents: vec![if file_fault == "link" {
                    core_ref.clone()
                } else {
                    kernel_ref.clone()
                }],
                named_input_hashes: kernel_ref.lineage().named_input_hashes.clone(),
                side_tables: IrSideTables::default(),
                origin: OutputOrigin::PackageSource,
                target: PublicationTarget::CurrentPackage,
            })
            .unwrap()
    };
    refs.push(file.erase());
    let roles = vec![
        summary_ref,
        core_ref,
        vc_ref,
        d_ref,
        kernel_ref.clone(),
        file.erase(),
    ];
    let six = PhaseInput::new(
        original.work_unit.clone(),
        PhaseDispatchInputBundle::new(
            snapshot.id,
            original.identities().input_hash(),
            original.identities().dependency_hashes().to_vec(),
            roles
                .iter()
                .map(|row| {
                    SealedParentOutputHandle::from_published_output(
                        &owner,
                        snapshot.id,
                        row.clone(),
                    )
                    .unwrap()
                })
                .collect(),
        )
        .unwrap(),
    );
    let summary = owner
        .storage()
        .get(
            &owner
                .storage()
                .typed_handle::<ModuleSummary>(&roles[0], roles[0].output_kind())
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        Some((*summary).clone()),
        inputs
            .native_theorem_module_summary_candidate(module)
            .unwrap()
    );
    let [(source_owner, visibility)] = check.owners() else {
        panic!()
    };
    let symbol = env.symbols().get(&source_owner.symbol).unwrap();
    let public = match (symbol.visibility(), symbol.export_status()) {
        (mizar_resolve::env::Visibility::Public, mizar_resolve::env::ExportStatus::Exported) => {
            true
        }
        (mizar_resolve::env::Visibility::Private, mizar_resolve::env::ExportStatus::LocalOnly) => {
            false
        }
        other => panic!("unsupported authentic owner {other:?}"),
    };
    assert_eq!(*visibility, if public { "public" } else { "private" });
    assert_eq!(summary.exported_symbols.len(), usize::from(public));
    let export = summary.exported_symbols.first();
    if let Some(export) = export {
        assert_eq!(export.origin_id, source_owner.symbol.fqn().as_str());
        assert_eq!(
            export.fully_qualified_name,
            source_owner.symbol.fqn().as_str()
        );
        assert_eq!(export.proof_status, None);
    }
    let core = owner
        .storage()
        .get(
            &owner
                .storage()
                .typed_handle::<mizar_core::core_ir::CoreIr>(&roles[1], roles[1].output_kind())
                .unwrap(),
        )
        .unwrap();
    let core_items = core.items().iter().collect::<Vec<_>>();
    let [(_, item)] = core_items.as_slice() else {
        panic!("one genuine Core owner")
    };
    assert_eq!(item.symbol, source_owner.symbol);
    assert_eq!(item.visibility.as_str(), *visibility);
    let owner_range = source_owner.source_range;
    let theorem_nodes = ast
        .node_views()
        .filter(|node| {
            node.as_theorem_item().is_some() && node.range() == source_owner.source_range
        })
        .collect::<Vec<_>>();
    let [theorem] = theorem_nodes.as_slice() else {
        panic!("one original checked theorem")
    };
    let owner_node = metadata.2.nodes().node(source_owner.site.node()).unwrap();
    assert_eq!(
        owner_node.resolved_node,
        arena.resolved_node_for(theorem.id())
    );
    assert_eq!(
        owner_node.anchor,
        mizar_session::SourceAnchor::Range(theorem.range())
    );
    let original_formulas = theorem
        .child_views()
        .filter(|node| node.as_formula_expression().is_some())
        .collect::<Vec<_>>();
    let [formula] = original_formulas.as_slice() else {
        panic!("one original owner formula")
    };
    let formulas =
        mizar_resolve::labels::source_public_theorem_formula_candidates(ast, &arena, &env)
            .unwrap()
            .unwrap();
    assert_eq!(formulas.len(), usize::from(public));
    if public {
        assert_eq!(formulas.get(&source_owner.symbol), Some(&formula.id()));
    }
    assert!(formula.as_formula_expression().is_some());
    let formula_children = formula.child_views().collect::<Vec<_>>();
    let [quantified] = formula_children.as_slice() else {
        panic!("one original formula child")
    };
    assert!(quantified.as_quantified_formula().is_some());
    assert_eq!(
        format!("{:?}", quantified.kind()),
        "QuantifiedFormula(Universal)"
    );
    assert_ne!(quantified.id(), formula.id());
    let quantified_range = quantified.range();
    assert_eq!(quantified_range, formula.range());
    let formula_range = formula.range();
    assert_eq!(formula_range.source_id, ast.source_id);
    assert_eq!(owner_range.source_id, ast.source_id);
    assert_eq!(source_owner.symbol.module(), arena.module());
    assert!(owner_range.start <= formula_range.start && formula_range.end <= owner_range.end);
    assert!(formula_range.start < formula_range.end);
    assert_eq!(check.scope().thesis(), None);
    assert!(check.scope().bindings().iter().all(|binding| {
        use mizar_resolve::names::{SourceVariableBindingKind, SourceVariableTypeRadix};
        matches!(
            binding.kind(),
            SourceVariableBindingKind::Quantifier | SourceVariableBindingKind::Let
        ) && binding.declared_type().is_some_and(|ty| {
            ty.radix() == SourceVariableTypeRadix::Set && ty.attributes().is_empty()
        })
    }));
    assert!(env.imports().is_empty() && env.exports().is_empty());
    assert!(
        env.symbols()
            .iter()
            .all(|entry| entry.symbol() == &source_owner.symbol)
    );
    let statement = StatementMetadata {
        item: source_owner.symbol.fqn().as_str().into(),
        source_range: SourceRangeSummary {
            start_byte: u64::try_from(formula_range.start).unwrap(),
            end_byte: u64::try_from(formula_range.end).unwrap(),
        },
        elaborated: source_text
            .get(formula_range.start..formula_range.end)
            .unwrap()
            .into(),
        reserve_binders: vec![],
        resolved_symbols: std::collections::BTreeMap::new(),
    };
    if let Some(export) = export {
        assert_eq!(
            (export.source_range.start_byte, export.source_range.end_byte),
            (
                u64::try_from(owner_range.start).unwrap(),
                u64::try_from(owner_range.end).unwrap()
            )
        );
    }
    let verified_exports = export
        .map(|export| VerifiedExport {
            origin_id: export.origin_id.clone(),
            fully_qualified_name: export.fully_qualified_name.clone(),
            namespace_path: export.namespace_path.clone(),
            visibility: ExportVisibility::Public,
            export_kind: export.declaration_kind.clone(),
            source_range: export.source_range,
            rendered_signature: export.rendered_signature.clone(),
            interface_fingerprint: ArtifactHashRef::new(
                ArtifactHashClass::Interface,
                MODULE_SUMMARY_SCHEMA_FAMILY,
                summary.schema_version,
                export
                    .compute_interface_fingerprint(summary.schema_version, &summary.module)
                    .unwrap(),
            ),
            proof_status: None,
            documentation_ref: None,
        })
        .into_iter()
        .collect();
    let proofs = theorem
        .child_views()
        .filter_map(|node| node.as_proof_block())
        .collect::<Vec<_>>();
    let [proof] = proofs.as_slice() else {
        panic!("one original proof")
    };
    let lets = proof
        .child_views()
        .filter_map(|node| node.as_let_statement())
        .collect::<Vec<_>>();
    let [generalization] = lets.as_slice() else {
        panic!("one actual universal introduction")
    };
    let declarations = generalization
        .child_views()
        .filter_map(|node| node.as_qualified_variable_segment())
        .flat_map(|node| node.child_views())
        .map(|node| node.id())
        .collect::<std::collections::BTreeSet<_>>();
    let locals = check
        .scope()
        .bindings()
        .iter()
        .filter(|binding| {
            binding.kind() == mizar_resolve::names::SourceVariableBindingKind::Let
                && declarations.contains(&binding.node())
        })
        .collect::<Vec<_>>();
    let [local] = locals.as_slice() else {
        panic!("one genuine original let binding")
    };
    let quantified_bindings = check
        .scope()
        .bindings()
        .iter()
        .filter(|binding| {
            binding.kind() == mizar_resolve::names::SourceVariableBindingKind::Quantifier
        })
        .collect::<Vec<_>>();
    let [bound] = quantified_bindings.as_slice() else {
        panic!("one genuine theorem binder")
    };
    assert_ne!(bound.id(), local.id());
    assert_eq!(bound.declared_type(), local.declared_type());
    assert_eq!(ast.node_view(local.node()).unwrap().range(), local.range());
    assert_eq!(
        source_text.get(local.range().start..local.range().end),
        Some(local.spelling())
    );
    let expected_thesis = format!("{} = {}", local.spelling(), local.spelling());
    let conclusions = proof
        .child_views()
        .filter_map(|node| node.as_conclusion_statement())
        .collect::<Vec<_>>();
    let [conclusion] = conclusions.as_slice() else {
        panic!("one actual proof conclusion")
    };
    let propositions = conclusion
        .child_views()
        .filter_map(|node| node.as_proposition())
        .collect::<Vec<_>>();
    let [proposition] = propositions.as_slice() else {
        panic!("one conclusion proposition")
    };
    let written_goals = proposition
        .child_views()
        .filter_map(|node| node.as_formula_expression())
        .collect::<Vec<_>>();
    let [written_goal] = written_goals.as_slice() else {
        panic!("one original conclusion formula")
    };
    let equalities = written_goal
        .child_views()
        .filter_map(|node| node.as_builtin_predicate_application())
        .collect::<Vec<_>>();
    let [equality] = equalities.as_slice() else {
        panic!("one original equality")
    };
    assert_eq!(
        equality
            .child_views()
            .filter(|node| node
                .as_token()
                .is_some_and(|token| token.text.as_ref() == "="))
            .count(),
        1
    );
    let operands = equality
        .child_views()
        .filter_map(|node| node.as_term_expression())
        .collect::<Vec<_>>();
    assert_eq!(operands.len(), 2);
    let mut thesis_ids = std::collections::BTreeSet::new();
    let primary = metadata.2.source_term().unwrap();
    for operand in operands {
        let references = operand
            .child_views()
            .filter_map(|node| node.as_term_reference())
            .collect::<Vec<_>>();
        let [original] = references.as_slice() else {
            panic!("one actual operand reference")
        };
        let resolved = check
            .scope()
            .references()
            .iter()
            .filter(|row| row.node() == original.id())
            .collect::<Vec<_>>();
        let [reference] = resolved.as_slice() else {
            panic!("one genuine resolved reference")
        };
        assert_eq!(
            reference.kind(),
            mizar_resolve::names::SourceVariableReferenceKind::Term
        );
        assert_eq!(reference.range(), original.range());
        assert_eq!(reference.binding(), local.id());
        assert_eq!(reference.spelling(), local.spelling());
        let terms = primary
            .terms()
            .iter()
            .map(|(_, row)| row)
            .filter(|row| row.source_range() == original.range())
            .collect::<Vec<_>>();
        let [term] = terms.as_slice() else {
            panic!("one actual primary handoff for original site")
        };
        assert_eq!(
            term.kind(),
            mizar_checker::source_term::SourcePrimaryTermKind::VariableReference
        );
        assert_eq!(term.spelling(), reference.spelling());
        let row = metadata.3.expr_metadata().get_by_site(term.site()).unwrap();
        assert_eq!(row.source_range, original.range());
        assert!(thesis_ids.insert(row.expr.as_str().to_owned()));
    }
    let goals = metadata
        .2
        .source_atomic_formula()
        .unwrap()
        .formulas()
        .iter()
        .map(|(_, row)| row)
        .filter(|row| row.source_range() == equality.range())
        .collect::<Vec<_>>();
    let [goal] = goals.as_slice() else {
        panic!("one actual equality handoff for original site")
    };
    assert_eq!(
        goal.kind(),
        mizar_checker::source_atomic_formula::SourceAtomicFormulaKind::Equality
    );
    let row = metadata.3.expr_metadata().get_by_site(goal.site()).unwrap();
    assert_eq!(row.source_range, equality.range());
    assert!(thesis_ids.insert(row.expr.as_str().to_owned()));
    assert_eq!(thesis_ids.len(), 3);
    let mut expressions = metadata
        .3
        .expr_metadata()
        .iter()
        .map(|(_, row)| {
            assert!(
                row.overload.is_none()
                    && row.inserted_views.is_empty()
                    && row.diagnostics.is_empty()
            );
            let node = metadata.2.nodes().node(row.typed_site.node()).unwrap();
            let inferred_type = row.final_type.map(|id| {
                let ty = metadata.1.normalized_types().get(id).unwrap();
                assert_eq!(ty.status, NormalizedTypeStatus::Known);
                assert_eq!(ty.head, TypeHeadRef::BuiltinSet);
                assert!(
                    ty.args.is_empty()
                        && ty.attributes.positive().is_empty()
                        && ty.attributes.negative().is_empty()
                );
                "set".to_owned()
            });
            ExpressionMetadata {
                expression_id: row.expr.as_str().into(),
                source_range: SourceRangeSummary {
                    start_byte: u64::try_from(row.source_range.start).unwrap(),
                    end_byte: u64::try_from(row.source_range.end).unwrap(),
                },
                expression_kind: node.kind.as_str().into(),
                rendered_surface: source_text
                    .get(row.source_range.start..row.source_range.end)
                    .unwrap()
                    .into(),
                inferred_type,
                resolved_symbol: None,
                inserted_coercions: vec![],
                active_thesis: thesis_ids
                    .contains(row.expr.as_str())
                    .then(|| expected_thesis.clone()),
                overload_resolution: None,
            }
        })
        .collect::<Vec<_>>();
    // Derive the additional syntax-root union from authentic declaration-owned types.
    // Checker expression metadata deliberately remains the original term/formula table.
    let source_types = metadata.2.source_type().unwrap();
    let context = metadata.2.source_context().unwrap();
    assert_eq!(source_types.source_id(), ast.source_id);
    assert_eq!(source_types.module_id(), metadata.2.module_id());
    assert_eq!(context.source_id(), ast.source_id);
    assert_eq!(context.module_id(), metadata.2.module_id());
    assert_eq!(
        source_types.applications().len(),
        context.declarations().len()
    );
    assert!(source_types.arguments().is_empty());
    assert!(source_types.definition_returns().is_empty());
    assert!(source_types.mode_rhs().is_empty());
    assert!(source_types.structure_members().is_empty());
    let mut covered_applications = std::collections::BTreeSet::new();
    let mut covered_declarations = std::collections::BTreeSet::new();
    let mut raw_sites = std::collections::BTreeSet::new();
    for (root, written) in source_types.expressions().iter() {
        use mizar_checker::{
            source_type::{SourceTypeApplicationForm, SourceTypeHead},
            typed_ast::{
                NodeRecoveryState, TypeEntryActual, TypeStatus, TypedSiteRef, TypingState,
            },
        };
        assert!(raw_sites.insert(written.site().clone()));
        assert_eq!(written.source_id(), ast.source_id);
        assert_eq!(written.module_id(), metadata.2.module_id());
        assert_eq!(written.form(), SourceTypeApplicationForm::Bare);
        assert_eq!(written.head(), &SourceTypeHead::BuiltinSet);
        assert_eq!(written.recovery(), NodeRecoveryState::Normal);
        assert_eq!(written.site(), &TypedSiteRef::Node(written.site().node()));
        assert_eq!(
            written.head_site(),
            &TypedSiteRef::Node(written.head_site().node())
        );
        let raw = metadata.2.nodes().node(written.site().node()).unwrap();
        let head = metadata.2.nodes().node(written.head_site().node()).unwrap();
        assert_eq!(raw.kind.as_str(), "TypeExpression");
        assert_eq!(head.kind.as_str(), "TypeHead");
        assert_eq!(raw.anchor, SourceAnchor::Range(written.source_range()));
        assert_eq!(head.anchor, SourceAnchor::Range(written.head_range()));
        assert_eq!(raw.children, [written.head_site().node()]);
        assert_eq!(raw.recovery, NodeRecoveryState::Normal);
        assert_eq!(head.recovery, NodeRecoveryState::Normal);
        assert_eq!(
            source_text.get(written.source_range().start..written.source_range().end),
            Some(written.spelling())
        );
        assert_eq!(
            source_text.get(written.head_range().start..written.head_range().end),
            Some(written.head_spelling())
        );
        let mut ordinals = Vec::new();
        let mut checked_denotations = std::collections::BTreeSet::new();
        for (application_id, application) in source_types
            .applications()
            .iter()
            .filter(|(_, application)| application.root() == root)
        {
            assert!(covered_applications.insert(application_id));
            let matching = context
                .declarations()
                .iter()
                .filter(|(_, declaration)| {
                    declaration.binding == application.binding()
                        && declaration.source_ordinal == application.source_ordinal()
                })
                .collect::<Vec<_>>();
            let [(declaration_id, declaration)] = matching.as_slice() else {
                panic!("application must have exactly one genuine declaration")
            };
            assert!(covered_declarations.insert(*declaration_id));
            assert_eq!(declaration.written_type_range, written.source_range());
            assert_eq!(raw.links.context, Some(declaration.local_context));
            assert_eq!(head.links.context, Some(declaration.local_context));
            let node = metadata.2.nodes().node(declaration.site.node()).unwrap();
            assert_eq!(node.typing, TypingState::Successful);
            let entry = metadata
                .2
                .types()
                .get(node.links.type_entry.unwrap())
                .unwrap();
            assert_eq!(entry.owner, declaration.site);
            assert_eq!(entry.status, TypeStatus::Known);
            let TypeEntryActual::Known(id) = entry.actual else {
                panic!("declaration must retain a checked type")
            };
            let actual = metadata.1.normalized_types().get(id).unwrap();
            assert_eq!(actual.status, NormalizedTypeStatus::Known);
            assert_eq!(actual.head, TypeHeadRef::BuiltinSet);
            assert!(actual.args.is_empty());
            assert!(actual.attributes.positive().is_empty());
            assert!(actual.attributes.negative().is_empty());
            checked_denotations.insert(id);
            ordinals.push(application.source_ordinal());
        }
        assert_eq!(checked_denotations.len(), 1);
        let ordinal = ordinals.into_iter().min().unwrap();
        let range = written.source_range();
        expressions.push(ExpressionMetadata {
            expression_id: format!("source.theorem.type.{ordinal}"),
            source_range: SourceRangeSummary {
                start_byte: u64::try_from(range.start).unwrap(),
                end_byte: u64::try_from(range.end).unwrap(),
            },
            expression_kind: raw.kind.as_str().into(),
            rendered_surface: source_text.get(range.start..range.end).unwrap().into(),
            inferred_type: Some("set".into()),
            resolved_symbol: None,
            inserted_coercions: vec![],
            active_thesis: None,
            overload_resolution: None,
        });
    }
    assert_eq!(
        covered_applications.len(),
        source_types.applications().len()
    );
    assert_eq!(covered_declarations.len(), context.declarations().len());
    expressions.push(ExpressionMetadata {
        expression_id: format!(
            "source.theorem.quantified.{}",
            source_owner.symbol.fqn().as_str()
        ),
        source_range: SourceRangeSummary {
            start_byte: u64::try_from(quantified_range.start).unwrap(),
            end_byte: u64::try_from(quantified_range.end).unwrap(),
        },
        expression_kind: format!("{:?}", quantified.kind()),
        rendered_surface: source_text
            .get(quantified_range.start..quantified_range.end)
            .unwrap()
            .into(),
        inferred_type: None,
        resolved_symbol: None,
        inserted_coercions: vec![],
        active_thesis: None,
        overload_resolution: None,
    });
    assert_eq!(
        expressions.len(),
        metadata.3.expr_metadata().len() + raw_sites.len() + 1
    );
    assert_eq!(
        expressions
            .iter()
            .map(|row| &row.expression_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        expressions.len()
    );
    assert!(!expressions.is_empty());
    let oracle_root = fixture.root.join("oracle-artifact");
    fs::create_dir(&oracle_root).unwrap();
    let published_path = PublishedArtifactPath::new(witness_path).unwrap();
    let file_domain = artifact_hash_domain(
        "mizar-proof/source-theorem-witness-file",
        ArtifactVersion::new(1, 0),
    );
    let written = write_published_artifact(
        &oracle_root,
        &published_path,
        &actual68.1,
        &file_domain,
        &[],
    )
    .unwrap();
    assert_eq!(
        written.artifact_hash,
        actual68.2.witness_artifact_hash.digest
    );
    let read = read_published_artifact(
        &oracle_root,
        &published_path,
        PublishedArtifactReadOptions {
            artifact_hash_domain: Some(&file_domain),
            expected_artifact_hash: Some(written.artifact_hash),
            ..PublishedArtifactReadOptions::default()
        },
    )
    .unwrap();
    assert_eq!(
        mizar_proof::witness_store::read_source_theorem_witness_file(&read, &actual68.2).unwrap(),
        actual68.0.2.payload_bytes()
    );
    let staged = actual68
        .0
        .2
        .clone()
        .stage_source_theorem_file(&read, &actual68.2)
        .unwrap();
    let physical = owner
        .storage()
        .get(
            &owner
                .storage()
                .typed_handle::<(mizar_kernel::checker::KernelCheckResult, PolicyDecision)>(
                    &kernel_ref,
                    kernel_ref.output_kind(),
                )
                .unwrap(),
        )
        .unwrap();
    let origin = KernelEvidenceOrigin::AtpFormulaSubstitution;
    let kernel_input = KernelPolicyInput::from_kernel_result(&physical.0, origin);
    let axioms = TrustedUsedAxiomsRef::from_kernel_result(&physical.0, origin).unwrap();
    assert_eq!(actual68.0.0.trusted_used_axioms(), Some(&axioms));
    let candidate = ProofEvidenceCandidate::from_trusted_kernel_input(
        CandidateSourceId::new(selected.selected_candidate_id().unwrap().as_str()).unwrap(),
        &kernel_input,
    )
    .unwrap()
    .with_provenance_hash(
        selected
            .reuse_metadata()
            .selected_candidate_provenance_hash()
            .unwrap(),
    )
    .with_selected_proof_witness_hash(staged.staged_payload_hash().digest);
    let witness_selection = select_winner(
        &ProofEvidenceSet::new(
            physical.0.target_vc_fingerprint().sort_bytes(),
            selected.reuse_metadata().encoded_problem_hash(),
            verifier.clone(),
        )
        .with_candidates([candidate]),
    );
    let [merged] =
        <[_; 1]>::try_from(merge([VcProofSelection::new(id, witness_selection)], []).unwrap())
            .unwrap();
    let original_meta = actual68.0.0.reuse_metadata();
    let identity = mizar_proof::status::ProofObligationIdentity::new(
        actual68.0.0.obligation_id(),
        original_meta.obligation_anchor().clone(),
        original_meta.obligation_fingerprint(),
        original_meta.vc_fingerprint(),
        original_meta.local_context_fingerprint(),
        original_meta.dependency_slice_fingerprint(),
    )
    .unwrap();
    let projection = project_status(
        ProofStatusProjectionInput::new(merged, verifier.clone(), identity)
            .with_trusted_used_axioms(axioms),
    )
    .unwrap();
    assert_eq!(
        projection.artifact_publication(),
        ArtifactStatusPublication::Publishable(CurrentArtifactObligationStatus::Accepted)
    );
    assert_eq!(
        projection.accepted_witness_obligation_id(),
        Some(actual68.0.0.obligation_id())
    );
    let raw_inputs = [
        (
            "mizar-vc/source-theorem-canonical-vc/v1",
            original_meta.vc_fingerprint(),
            HashClass::Interface,
        ),
        (
            "mizar-vc/source-theorem-canonical-context/v1",
            original_meta.local_context_fingerprint(),
            HashClass::Interface,
        ),
        (
            DEPENDENCY_SLICE_SCHEMA_VERSION,
            original_meta.dependency_slice_fingerprint(),
            HashClass::Interface,
        ),
        (
            if inputs.build_plan.lockfile.toolchain_version.is_some() {
                "mizar-build/parsed-lock-content/v2"
            } else {
                "mizar-build/parsed-lock-content/v1"
            },
            snapshot.lockfile_hash,
            HashClass::Artifact,
        ),
        (
            "mizar-build/package-config-content/v1",
            snapshot.verifier_config_hash,
            HashClass::Interface,
        ),
    ];
    let mut framed = raw_inputs
        .map(|(producer, digest, class)| {
            let hex = digest
                .as_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            let value = CanonicalJson::object([
                ("producer", CanonicalJson::string(producer)),
                ("digest", CanonicalJson::string(hex)),
            ])
            .unwrap();
            let digest = CanonicalHashDomain::new(
                class,
                "mizar-driver/source-artifact-owner-fingerprint",
                ArtifactVersion::new(1, 0),
            )
            .hash(&value, &[]);
            ArtifactHashRef::new(
                if class == HashClass::Artifact {
                    ArtifactHashClass::Artifact
                } else {
                    ArtifactHashClass::Interface
                },
                "mizar-driver/source-artifact-owner-fingerprint",
                ArtifactVersion::new(1, 0),
                digest,
            )
        })
        .into_iter();
    let vc_fingerprint = framed.next().unwrap();
    let local_context_fingerprint = framed.next().unwrap();
    let dependency_slice_fingerprint = framed.next().unwrap();
    let lockfile_hash = framed.next().unwrap();
    let verifier_config_hash = framed.next().unwrap();
    let vc_range = vcs.vc(id).unwrap().anchor.source_range.unwrap();
    assert_eq!(vc_range.source_id, ast.source_id);
    let obligation = ObligationMetadata {
        obligation_id: projection.obligation_id().into(),
        obligation_anchor: Some(original_meta.obligation_anchor().as_str().into()),
        owner_origin_id: export.map(|export| export.origin_id.clone()),
        source_range: SourceRangeSummary {
            start_byte: u64::try_from(vc_range.start).unwrap(),
            end_byte: u64::try_from(vc_range.end).unwrap(),
        },
        obligation_kind: "theorem-proof-step".into(),
        statement_summary: source_text
            .get(vc_range.start..vc_range.end)
            .unwrap()
            .into(),
        obligation_fingerprint: actual68.2.obligation_fingerprint.clone(),
        vc_fingerprint,
        local_context_fingerprint,
        dependency_slice_fingerprint,
        verifier_policy_fingerprint: actual68
            .2
            .kernel_acceptance
            .verifier_policy_fingerprint
            .clone(),
        status: ObligationStatus::Accepted,
        accepted_witness_obligation_id: projection
            .accepted_witness_obligation_id()
            .map(str::to_owned),
        deterministic_discharge_hash: None,
        diagnostic_ref: None,
    };
    let expected = if file_fault.is_empty() {
        Some(
            ArtifactProjectionService::new(owner.clone())
                .project_module(ProjectVerifiedArtifactInput {
                    snapshot: snapshot.id,
                    required_outputs: roles,
                    module: summary.module.clone(),
                    source_file: snapshot.source_versions[0].normalized_path.as_str().into(),
                    source_hash: snapshot.source_versions[0].source_hash,
                    verified_at: None,
                    exports: verified_exports,
                    expressions,
                    statements: vec![statement],
                    obligations: vec![obligation],
                    proof_witnesses: vec![actual68.2.clone()],
                    diagnostics: vec![],
                    provenance: BuildProvenance {
                        toolchain: snapshot.toolchain.identity().into(),
                        language_edition: snapshot.source_versions[0].edition.as_str().into(),
                        lockfile_hash,
                        verifier_config_hash,
                        dependency_artifact_hashes: vec![],
                        cache_key: None,
                    },
                })
                .map_err(|error| error.to_string()),
        )
    } else {
        None
    };
    assert_eq!(actual68.0.0.accepted_witness_obligation_id(), None);
    assert_eq!(
        actual68.0.0.reuse_metadata().selected_proof_witness_hash(),
        None
    );
    assert!(actual68.0.1.requires_cache_miss());
    consume(
        &fixture,
        inputs,
        &six,
        &owner,
        &refs,
        &expected,
        (&metadata.1, &metadata.2, &metadata.3, source_text),
    );
}

struct ScheduledArtifactServiceForwarder {
    registry: std::sync::Arc<std::sync::Mutex<mizar_driver::registry::PhaseRegistry>>,
    descriptor: mizar_driver::registry::PhaseDescriptor,
    replay: Option<mizar_driver::registry::PhaseResult>,
}

impl mizar_driver::registry::PhaseService for ScheduledArtifactServiceForwarder {
    fn phase(&self) -> mizar_driver::registry::PhaseDescriptor {
        self.descriptor.clone()
    }

    fn cache_key(
        &self,
        input: &PhaseInput,
        _context: &mizar_driver::registry::PhaseCacheContext,
    ) -> PhaseCacheIntent {
        self.registry
            .lock()
            .unwrap()
            .cache_key_for_phase(self.descriptor.phases[0], input)
            .unwrap()
            .intent
    }

    fn execute(
        &self,
        input: PhaseInput,
        context: mizar_driver::registry::PhaseExecutionContext<'_>,
    ) -> mizar_driver::registry::PhaseResult {
        if let Some(result) = &self.replay {
            return result.clone();
        }
        self.registry
            .lock()
            .unwrap()
            .execute_phase_with_resources(
                self.descriptor.phases[0],
                input,
                PhaseExecutionResources {
                    cancellation: context.cancellation,
                    diagnostics: context.diagnostics,
                    output_publisher: context.output_publisher,
                    source_load: context.source_load,
                },
            )
            .unwrap()
            .result
    }
}

struct RetainedArtifactInputs {
    unit: WorkUnit,
    inputs: Vec<(PipelinePhase, PhaseDispatchInputBundle)>,
}

impl PhaseDispatchInputProvider<BuildTask> for RetainedArtifactInputs {
    fn dispatch_input_for_task(
        &self,
        request: PhaseDispatchInputRequest<'_, BuildTask>,
    ) -> Result<Option<PhaseDispatchInputBundle>, DispatchInputError> {
        if request.task().unit != self.unit {
            return Ok(None);
        }
        Ok(self
            .inputs
            .iter()
            .find(|(phase, _)| request.task().phases.first() == Some(phase))
            .map(|(_, input)| input.clone()))
    }
}

mod committed;
mod current;
mod dependency;
mod discovered;
mod explicit;
mod frontend;
mod import;
mod misc;
mod native_algorithm;
mod native_disk;
mod native_frontend;
mod native_import;
mod native_open;
mod native_source;
mod native_theorem_artifact;
mod native_theorem_candidate;
mod native_theorem_core;
mod native_theorem_discharge;
mod native_theorem_kernel;
mod native_theorem_metadata;
mod native_theorem_module;
mod native_theorem_preparation;
mod native_theorem_reflexivity;
mod native_theorem_registration;
mod native_theorem_summary;
mod native_theorem_typed;
mod native_theorem_vc;
mod native_theorem_vcs;
mod native_toolchain;
mod native_workspace;
mod registered;
mod scheduled_cross;
mod scheduled_theorem_algorithm;
mod scheduled_theorem_atp;
mod scheduled_theorem_candidate;
mod scheduled_theorem_elaboration;
mod scheduled_theorem_overload;
mod scheduled_theorem_registration;
mod scheduled_theorem_resolver;
mod scheduled_theorem_signature;
mod scheduled_theorem_type;
mod scheduled_theorem_vc;
mod source;
mod supplied;
