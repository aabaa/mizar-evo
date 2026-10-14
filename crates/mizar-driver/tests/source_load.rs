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

#[test]
fn discovered_real_frontend_prefix_is_stable_across_replay_capture_order_and_worker_controls() {
    for mode in ["chain", "independent"] {
        let (main, leaf, base) = if mode == "chain" {
            (
                b"import alpha.leaf;\ntheorem T: a arrange b = a;\n".as_slice(),
                b"import alpha.base;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: a combine b = a;\n".as_slice(),
                b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n".as_slice(),
            )
        } else {
            (
                b"import alpha.leaf;\ntheorem T: a combine b = a;\n".as_slice(),
                b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n".as_slice(),
                b"definition\nend;\n".as_slice(),
            )
        };
        let fixture = WorkspaceLeafFixture::new(main, leaf, Some(base), usize::MAX);
        let mut snapshot = fixture.submission.session.captured().snapshot().id;
        let mut versions = fixture
            .submission
            .session
            .request()
            .source_inputs
            .versions
            .clone();
        let mut baselines = Vec::new();
        let mut retained: Option<(
            CompilerDriver,
            std::sync::Arc<PhaseOutputPublisher>,
            BuildSnapshotId,
            AnyPhaseOutputRef,
        )> = None;
        for (stage, workers, order, reverse_capture) in [
            (0, 1, CompletionOrder::Canonical, false),
            (0, 1, CompletionOrder::Canonical, false),
            (0, 4, CompletionOrder::Reverse, false),
            (0, 4, CompletionOrder::Canonical, true),
            (1, 1, CompletionOrder::Canonical, false),
            (1, 1, CompletionOrder::Canonical, false),
            (1, 4, CompletionOrder::Reverse, false),
            (1, 4, CompletionOrder::Canonical, true),
            (2, 1, CompletionOrder::Canonical, false),
            (2, 1, CompletionOrder::Canonical, false),
            (2, 4, CompletionOrder::Reverse, false),
            (2, 4, CompletionOrder::Canonical, true),
        ] {
            let (mut request, mut input) = fixture.scheduled_request(
                ModuleDependencyOverlay::unavailable(),
                Vec::new(),
                Vec::new(),
            );
            let first = stage == baselines.len();
            if stage > 0 && first {
                fixture.fixture.write(if stage == 2 {
                    b"import alpha.leaf;\ntheorem T: ;\n"
                } else if mode == "chain" {
                    b"import alpha.leaf;\ntheorem EditedT: a arrange b = a;\n"
                } else {
                    b"import alpha.leaf;\ntheorem EditedT: a combine b = a;\n"
                });
                let root = fixture.fixture.root.join("alpha");
                let version = versions
                    .iter_mut()
                    .find(|version| version.module_path.as_str() == "main")
                    .unwrap();
                let loaded = mizar_session::SourceLoader::load(
                    &DiskSourceLoader::new(&root),
                    snapshot,
                    SourceInput {
                        package_id: version.package_id.clone(),
                        module_path: version.module_path.clone(),
                        normalized_path: version.normalized_path.clone(),
                        edition: version.edition.clone(),
                        origin: SourceOriginInput::Disk {
                            path: PathBuf::from("src/main.miz"),
                        },
                    },
                    &fixture.ids,
                )
                .unwrap();
                *version = SourceVersion {
                    source_id: loaded.source_id,
                    package_id: loaded.package_id,
                    module_path: loaded.module_path,
                    normalized_path: loaded.normalized_path,
                    source_hash: loaded.source_hash,
                    edition: loaded.edition,
                    origin: loaded.origin,
                };
            }
            request.source_inputs.versions = versions.clone();
            request.generation = BuildRequestGeneration::new(stage as u64);
            let captured = request
                .clone()
                .allocate(&fixture.ids)
                .unwrap()
                .capture_snapshot(&SnapshotRegistry::new())
                .unwrap();
            if stage > 0 && first {
                assert_ne!(snapshot, captured.captured().snapshot().id);
            }
            snapshot = captured.captured().snapshot().id;
            if reverse_capture {
                request.source_inputs.versions.reverse();
            }
            input.worker_count = workers;
            input.completion_order = order;

            let (mut driver, publisher, obsolete_output) = if stage > 0 && first {
                let (driver, publisher, old_snapshot, output) = retained.take().unwrap();
                publisher
                    .replace_current_snapshot(old_snapshot, snapshot)
                    .unwrap();
                (driver, publisher, Some((old_snapshot, output)))
            } else {
                let publisher = publisher(
                    snapshot,
                    usize::MAX,
                    &fixture.task(TaskKind::SourceLoad, "base").unit,
                );
                (
                    fixture
                        .scheduled_driver()
                        .with_output_publisher(publisher.clone()),
                    publisher,
                    None,
                )
            };
            for module in ["leaf", "main"] {
                let WorkUnit::Module { module: id } =
                    &fixture.task(TaskKind::SourceLoad, module).unit
                else {
                    unreachable!()
                };
                publisher.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("SourceLoad"),
                    OutputKind::new("SourceUnit"),
                    IrWorkUnit::new(format!("{:?}:{:?}", id.package.as_str(), id.path.as_str())),
                ));
            }
            for module in ["base", "leaf", "main"] {
                allow_frontend(&publisher, &fixture.task(TaskKind::SourceLoad, module).unit);
            }
            let submission = driver
                .submit_with_import_discovery(
                    request,
                    &fixture.ids,
                    &SnapshotRegistry::new(),
                    input,
                )
                .unwrap();
            assert_eq!(
                submission.session.captured().snapshot().id,
                snapshot,
                "{mode}"
            );
            if let Some((old_snapshot, output)) = obsolete_output {
                assert!(publisher.validate_current_snapshot(old_snapshot).is_err());
                for target in [old_snapshot, snapshot] {
                    assert!(publisher.validate_current_output(target, &output).is_err());
                    assert!(
                        SealedParentOutputHandle::from_current_output(
                            &publisher,
                            target,
                            output.clone()
                        )
                        .is_err()
                    );
                }
            }
            assert_eq!(
                submission.status,
                if stage == 2 {
                    DriverSubmissionStatus::SchedulerValidated
                } else {
                    DriverSubmissionStatus::BlockedByMissingPhaseServices
                },
                "{mode}"
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if stage == 2 {
                    BuildSessionOutcome::Failed
                } else {
                    BuildSessionOutcome::Blocked
                }),
                "{mode}"
            );
            let graph = submission.task_graph.as_ref().unwrap();
            let run = submission.scheduler_run.as_ref().unwrap();
            for task in graph.tasks().iter().filter(|task| {
                task.phases.iter().any(|phase| {
                    !matches!(
                        phase,
                        PipelinePhase::PackageResolve
                            | PipelinePhase::SourceLoad
                            | PipelinePhase::Frontend
                    )
                })
            }) {
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == task.id)
                        .unwrap()
                        .state,
                    TaskState::Blocked
                );
            }
            let task = |kind: TaskKind, module: &str| {
                graph
                    .tasks()
                    .iter()
                    .find(|task| {
                        task.kind == kind
                            && matches!(&task.unit, WorkUnit::Module { module: id }
                            if id.path.as_str() == module)
                    })
                    .unwrap()
            };
            let mut projection = Vec::new();
            for module in ["base", "leaf", "main"] {
                let source_task = task(TaskKind::SourceLoad, module);
                let frontend_task = task(TaskKind::Frontend, module);
                let semantic = task(TaskKind::ModuleResolve, module);
                assert_eq!(
                    semantic.dependency_coverage,
                    DependencyCoverage::MissingModuleDependencyOverlay,
                    "{mode}:{module}"
                );
                let source_result = &run.phase_results[&source_task.id][0];
                let frontend_result = &run.phase_results[&frontend_task.id][0];
                assert_eq!(
                    source_result.status,
                    PhaseStatus::Complete,
                    "{mode}:{module}"
                );
                let diagnostic_case = stage == 2 && module == "main";
                assert_eq!(
                    frontend_result.status,
                    if diagnostic_case {
                        PhaseStatus::Recoverable
                    } else {
                        PhaseStatus::Complete
                    },
                    "{mode}:{module}"
                );
                assert_eq!(frontend_result.diagnostics.is_empty(), !diagnostic_case);
                let version = versions
                    .iter()
                    .find(|version| version.module_path.as_str() == module)
                    .unwrap();
                assert!(
                    submission
                        .module_index
                        .as_ref()
                        .unwrap()
                        .dependency_summaries
                        .is_empty()
                );
                let cache_intents = [PipelinePhase::SourceLoad, PipelinePhase::Frontend].map(|phase| {
                    let parents = if phase == PipelinePhase::Frontend {
                        frontend_task.dependencies.iter()
                            .flat_map(|id| run.phase_results.get(id).into_iter().flatten())
                            .filter(|result| result.status == PhaseStatus::Complete)
                            .flat_map(|result| result.output_refs.iter())
                            .map(|parent| SealedParentOutputHandle::from_current_output(
                                &publisher, snapshot, parent.clone()).unwrap()).collect()
                    } else { Vec::new() };
                    let bundle = PhaseDispatchInputBundle::new(snapshot, source_key(version),
                        Vec::new(), parents).unwrap();
                    let input = PhaseInput::new(source_task.unit.clone(), bundle);
                    let intent = driver.registry().cache_key_for_phase(phase, &input).unwrap().intent;
                    assert!(matches!(&intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty()));
                    intent
                });
                let source_ref = &source_result.output_refs[0];

                assert!(
                    publisher
                        .registry()
                        .output_lineage(source_ref.output())
                        .unwrap()
                        .parents
                        .is_empty()
                );
                let source = publisher
                    .storage()
                    .get(
                        &publisher
                            .storage()
                            .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                            .unwrap(),
                    )
                    .unwrap();
                let frontend = frontend_result.output_refs.first().map(|frontend_ref| {
                    let parents = publisher
                        .registry()
                        .output_lineage(frontend_ref.output())
                        .unwrap()
                        .parents
                        .iter()
                        .map(|id| {
                            let lineage = publisher.registry().output_lineage(*id).unwrap();
                            (
                                lineage.phase.as_str().to_owned(),
                                lineage.work_unit.as_str().to_owned(),
                            )
                        })
                        .collect::<Vec<_>>();
                    let frontend = publisher
                    .storage()
                    .get(
                        &publisher
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                frontend_ref,
                                &OutputKind::new("FrontendOutput"),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                    (
                        frontend.canonical_disk_bytes().unwrap(),
                        frontend.cache_keys.canonical_bytes().unwrap(),
                        parents,
                    )
                });
                assert_eq!(frontend.is_none(), diagnostic_case);
                let diagnostics = source_result
                    .diagnostics
                    .iter()
                    .chain(&frontend_result.diagnostics)
                    .flat_map(|batch| batch.drafts())
                    .map(|draft| {
                        let DiagnosticPrimaryLocation::Span(span) = draft.primary_location() else {
                            panic!("expected scheduled parser source span");
                        };
                        let SourceAnchor::Range(mut range) = *span.anchor() else {
                            panic!("expected parser range anchor");
                        };
                        source.line_map.validate_range(range).unwrap();
                        range.source_id = version.source_id;
                        assert!(draft.secondary_spans().is_empty());
                        assert!(draft.notes().iter().all(|note| note.span().is_none()));
                        assert!(
                            draft
                                .details()
                                .entries()
                                .values()
                                .all(|value| matches!(value, DiagnosticDetailValue::String(_)))
                        );
                        assert!(draft.fixes().is_empty());
                        assert!(draft.explanation().is_none());
                        (
                            draft.source_snapshot(),
                            draft.code(),
                            draft.phase(),
                            draft.category(),
                            draft.stable_detail_key().to_owned(),
                            draft.message().to_owned(),
                            DiagnosticSpan::from_anchor(
                                SourceAnchor::Range(range),
                                span.role(),
                                span.label().map(str::to_owned),
                                span.freshness(),
                                span.zero_width(),
                            )
                            .unwrap(),
                            draft.notes().to_vec(),
                            draft.details().clone(),
                        )
                    })
                    .collect::<Vec<_>>();
                assert_eq!(diagnostics.is_empty(), !diagnostic_case);
                projection.push((
                    module.to_owned(),
                    source.canonical_disk_bytes().unwrap(),
                    SourceUnitCacheKey::from_source(&source).stable_hash(),
                    frontend,
                    (source_result.status, frontend_result.status),
                    diagnostics,
                    cache_intents,
                ));
            }
            if let Some(expected) = baselines.get(stage) {
                assert_eq!(
                    &projection, expected,
                    "{mode}: workers={workers}, order={order:?}, reverse_capture={reverse_capture}"
                );
            } else {
                baselines.push(projection);
                if stage > 0 {
                    assert_ne!(baselines[stage][2].1, baselines[stage - 1][2].1);
                    if stage == 1 {
                        assert_ne!(baselines[stage][2].3, baselines[stage - 1][2].3);
                    }
                    let (old_request, old_input) = fixture.scheduled_request(
                        ModuleDependencyOverlay::unavailable(),
                        Vec::new(),
                        Vec::new(),
                    );
                    let obsolete = driver
                        .submit(
                            old_request,
                            &fixture.ids,
                            &SnapshotRegistry::new(),
                            old_input,
                        )
                        .unwrap();
                    assert_eq!(
                        obsolete.status,
                        DriverSubmissionStatus::SupersededBeforeSubmission
                    );
                    assert_eq!(
                        obsolete.session.state(),
                        BuildSessionState::Finished(BuildSessionOutcome::Superseded)
                    );
                    assert!(obsolete.scheduler_run.is_none());
                    assert!(matches!(
                        obsolete.publication_decision,
                        mizar_driver::request::PublicationDecision::Suppressed(_)
                    ));
                }
                retained = Some((
                    driver,
                    publisher,
                    snapshot,
                    run.phase_results[&task(TaskKind::SourceLoad, "main").id][0].output_refs[0]
                        .clone(),
                ));
            }
        }
    }
}

#[test]
fn discovered_source_import_cycles_fail_graph_validation_without_overlay_fallback() {
    for (mode, main, leaf, expected) in [
        (
            "self",
            b"import alpha.main;\ndefinition\nend;\n".as_slice(),
            b"definition\nend;\n".as_slice(),
            TaskGraphDiagnosticKind::SelfDependency,
        ),
        (
            "pair",
            b"import alpha.leaf;\ndefinition\nend;\n".as_slice(),
            b"import alpha.main;\ndefinition\nend;\n".as_slice(),
            TaskGraphDiagnosticKind::DependencyCycle,
        ),
    ] {
        let fixture = WorkspaceLeafFixture::new(main, leaf, None, usize::MAX);
        let (request, input) = fixture.scheduled_request(
            ModuleDependencyOverlay::unavailable(),
            Vec::new(),
            Vec::new(),
        );
        let mut driver = fixture.scheduled_driver();
        let error = driver
            .submit_with_import_discovery(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap_err();
        let DriverSubmitError::TaskGraph {
            session,
            diagnostics,
        } = error
        else {
            panic!("{mode}: expected structural import-graph rejection");
        };
        assert_eq!(
            session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Failed),
            "{mode}"
        );
        assert!(
            diagnostics
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.kind == expected),
            "{mode}"
        );
        assert!(
            diagnostics.diagnostics().iter().all(|diagnostic| {
                diagnostic.kind != TaskGraphDiagnosticKind::MissingModuleDependencyOverlay
            }),
            "{mode}: import discovery must not fall back to Unavailable"
        );
    }
}

#[test]
fn discovered_workspace_leaf_orders_frontend_without_semantic_coverage() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ntheorem T: a combine b = a;\n",
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
        None,
        usize::MAX,
    );
    let (request, mut input) = fixture.scheduled_request(
        ModuleDependencyOverlay::unavailable(),
        Vec::new(),
        Vec::new(),
    );
    input.worker_count = 4;
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit_with_import_discovery(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByMissingPhaseServices
    );
    let graph = submission.task_graph.as_ref().unwrap();
    let run = submission.scheduler_run.as_ref().unwrap();
    let main = fixture.task(TaskKind::Frontend, "main");
    let leaf = fixture.task(TaskKind::Frontend, "leaf");
    assert!(
        graph
            .edges()
            .iter()
            .any(|edge| edge.dependent == main.id && edge.dependency == leaf.id)
    );
    let semantic = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::ModuleResolve && task.unit == main.unit)
        .unwrap();
    assert_eq!(
        semantic.dependency_coverage,
        DependencyCoverage::MissingModuleDependencyOverlay
    );
    assert!(graph.diagnostics().is_empty());
    let leaf_commit = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::ArtifactCommit && task.unit == leaf.unit)
        .unwrap();
    assert!(
        !graph
            .edges()
            .iter()
            .any(|edge| { edge.dependent == semantic.id && edge.dependency == leaf_commit.id })
    );
    for task in [leaf, main] {
        assert_eq!(run.phase_results[&task.id][0].status, PhaseStatus::Complete);
    }
    let leaf_output = &run.phase_results[&leaf.id][0].output_refs[0];
    let main_output = &run.phase_results[&main.id][0].output_refs[0];
    let own = &run.phase_results[&fixture.task(TaskKind::SourceLoad, "main").id][0].output_refs[0];
    let lineage = fixture
        .publisher
        .registry()
        .output_lineage(main_output.output())
        .unwrap();
    assert_eq!(lineage.parents.len(), 2);
    assert!(lineage.parents.contains(&own.output()));
    assert!(lineage.parents.contains(&leaf_output.output()));
    let typed = fixture
        .publisher
        .storage()
        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
            main_output,
            &OutputKind::new("FrontendOutput"),
        )
        .unwrap();
    let loaded = fixture.publisher.storage().get(&typed).unwrap();
    assert!(
        loaded.tokens.tokens().iter().any(|token| {
            token.text.as_ref() == "combine" && token.kind == TokenKind::UserSymbol
        })
    );
}

#[test]
fn discovered_imports_order_intermediates_branches_and_leave_independent_modules_free() {
    for mode in ["chain", "branch_alias", "independent"] {
        let (main, leaf, base, main_symbol) = match mode {
            "chain" => (
                b"import alpha.leaf;\ntheorem T: a arrange b = a;\n".as_slice(),
                b"import alpha.base;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: a combine b = a;\n".as_slice(),
                b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n".as_slice(),
                "arrange",
            ),
            "branch_alias" => (
                b"import alpha.{leaf, base};\nimport alpha.leaf as L;\ntheorem T: a combine b = a;\ntheorem U: a arrange b = a;\n".as_slice(),
                b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n".as_slice(),
                b"definition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\n".as_slice(),
                "arrange",
            ),
            "independent" => (
                b"import alpha.leaf;\ntheorem T: a combine b = a;\n".as_slice(),
                b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n".as_slice(),
                b"definition\nend;\n".as_slice(),
                "combine",
            ),
            _ => unreachable!(),
        };
        let fixture = WorkspaceLeafFixture::new(main, leaf, Some(base), usize::MAX);
        let (request, mut input) = fixture.scheduled_request(
            ModuleDependencyOverlay::unavailable(),
            Vec::new(),
            Vec::new(),
        );
        input.worker_count = 4;
        let mut driver = fixture.scheduled_driver();
        let submission = driver
            .submit_with_import_discovery(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices,
            "{mode}"
        );
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        let main_task = fixture.task(TaskKind::Frontend, "main");
        let leaf_task = fixture.task(TaskKind::Frontend, "leaf");
        let base_task = fixture.task(TaskKind::Frontend, "base");
        let edge_count = |dependent: &BuildTask, dependency: &BuildTask| {
            graph
                .edges()
                .iter()
                .filter(|edge| edge.dependent == dependent.id && edge.dependency == dependency.id)
                .count()
        };
        assert_eq!(edge_count(main_task, leaf_task), 1, "{mode}");
        assert_eq!(
            edge_count(main_task, base_task),
            usize::from(mode == "branch_alias"),
            "{mode}"
        );
        assert_eq!(
            edge_count(leaf_task, base_task),
            usize::from(mode == "chain"),
            "{mode}"
        );
        for task in [base_task, leaf_task, main_task] {
            assert_eq!(
                run.phase_results[&task.id][0].status,
                PhaseStatus::Complete,
                "{mode}"
            );
        }
        let output = &run.phase_results[&main_task.id][0].output_refs[0];
        let typed = fixture
            .publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                output,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let loaded = fixture.publisher.storage().get(&typed).unwrap();
        assert!(
            loaded.tokens.tokens().iter().any(|token| {
                token.text.as_ref() == main_symbol && token.kind == TokenKind::UserSymbol
            }),
            "{mode}"
        );
    }
}

#[test]
fn discovered_imports_discard_all_edges_on_untrusted_source_input() {
    for mode in [
        "drift",
        "malformed",
        "unresolved",
        "captured_edition",
        "missing_capture",
        "unrelated_bad",
    ] {
        let main = match mode {
            "malformed" => b"import alpha.;\ndefinition\nend;\n".as_slice(),
            "unresolved" => b"import alpha.absent;\ndefinition\nend;\n".as_slice(),
            _ => b"import alpha.leaf;\ndefinition\nend;\n".as_slice(),
        };
        let base =
            (mode == "unrelated_bad").then_some(b"import alpha.;\ndefinition\nend;\n".as_slice());
        let fixture = WorkspaceLeafFixture::new(main, b"definition\nend;\n", base, usize::MAX);
        if mode == "drift" {
            fixture.fixture.write(b"definition\nend;\n");
        }
        let (mut request, input) = fixture.scheduled_request(
            ModuleDependencyOverlay::unavailable(),
            Vec::new(),
            Vec::new(),
        );
        if mode == "captured_edition" {
            request
                .source_inputs
                .versions
                .iter_mut()
                .find(|version| version.module_path.as_str() == "leaf")
                .unwrap()
                .edition = Edition::new("2026");
        }
        if mode == "missing_capture" {
            request
                .source_inputs
                .versions
                .retain(|version| version.module_path.as_str() != "leaf");
        }
        let mut driver = fixture.scheduled_driver();
        let error = driver
            .submit_with_import_discovery(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap_err();
        let DriverSubmitError::TaskGraph {
            session,
            diagnostics,
        } = error
        else {
            panic!("{mode}: discovery must leave unavailable graph coverage");
        };
        assert_eq!(
            session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Failed),
            "{mode}"
        );
        assert!(
            diagnostics.diagnostics().iter().any(|diagnostic| {
                diagnostic.kind == TaskGraphDiagnosticKind::MissingModuleDependencyOverlay
            }),
            "{mode}"
        );
    }
}

#[test]
fn import_discovery_preserves_legacy_submit_and_supplied_overlay_precedence() {
    for mode in ["legacy", "complete", "package_only"] {
        let fixture = WorkspaceLeafFixture::new(
            b"import alpha.leaf;\ntheorem T: a combine b = a;\n",
            b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
            None,
            usize::MAX,
        );
        let overlay = match mode {
            "complete" => complete_leaf_import_overlay(),
            "package_only" => ModuleDependencyOverlay::package_only(Vec::new()),
            _ => ModuleDependencyOverlay::unavailable(),
        };
        let (request, input) = fixture.scheduled_request(overlay, Vec::new(), Vec::new());
        let mut driver = fixture.scheduled_driver();
        if mode == "legacy" {
            let error = driver
                .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
                .unwrap_err();
            assert!(matches!(error, DriverSubmitError::TaskGraph { .. }));
            continue;
        }
        let submission = driver
            .submit_with_import_discovery(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap();
        let graph = submission.task_graph.as_ref().unwrap();
        let main = fixture.task(TaskKind::Frontend, "main");
        let leaf = fixture.task(TaskKind::Frontend, "leaf");
        let semantic = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::ModuleResolve && task.unit == main.unit)
            .unwrap();
        assert_eq!(
            semantic.dependency_coverage,
            if mode == "complete" {
                DependencyCoverage::Complete
            } else {
                DependencyCoverage::PackageConservative
            }
        );
        assert_eq!(
            graph
                .edges()
                .iter()
                .filter(|edge| { edge.dependent == main.id && edge.dependency == leaf.id })
                .count(),
            usize::from(mode == "complete")
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        let result = &run.phase_results[&main.id][0];
        assert_eq!(
            result.status,
            if mode == "complete" {
                PhaseStatus::Complete
            } else {
                PhaseStatus::Blocking
            }
        );
    }
}

#[test]
fn supplied_workspace_leaf_public_symbol_lexes_importer_and_seals_both_parents() {
    let importer = b"import alpha.leaf;\ntheorem T: a combine b = a;\n";
    let leaf =
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
    for threshold in [usize::MAX, 3] {
        for reverse in [false, true] {
            let fixture = WorkspaceLeafFixture::new(importer, leaf, None, threshold);
            let (own_output, leaf_output) = fixture.clean_parents();
            assert_eq!(
                matches!(leaf_output.placement(), StoragePlacement::Resident),
                threshold == usize::MAX
            );
            fs::remove_file(fixture.fixture.root.join("alpha/src/leaf.miz")).unwrap();
            let parents = if reverse {
                vec![leaf_output.clone(), own_output.clone()]
            } else {
                vec![own_output.clone(), leaf_output.clone()]
            };
            let importer_frontend = fixture.frontend("main", parents, None);
            assert_eq!(importer_frontend.status, PhaseStatus::Complete);
            assert!(importer_frontend.diagnostics.is_empty());
            let output = &importer_frontend.output_refs[0];
            let lineage = fixture
                .publisher
                .registry()
                .output_lineage(output.output())
                .unwrap();
            assert_eq!(lineage.parents.len(), 2);
            assert!(lineage.parents.contains(&own_output.output()));
            assert!(lineage.parents.contains(&leaf_output.output()));
            let typed = fixture
                .publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    output,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let loaded = fixture.publisher.storage().get(&typed).unwrap();
            assert_eq!(loaded.source.source_id, fixture.fixture.version.source_id);
            assert!(loaded.ast.is_some());
            let candidates =
                ImportPathCandidate::from_surface_ast(loaded.ast.as_ref().unwrap()).unwrap();
            let resolution = ImportPathResolver::new(ModuleIndexInput::new(
                fixture.submission.module_index.as_ref().unwrap(),
            ))
            .resolve(
                &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("main")),
                &candidates,
            );
            assert_eq!(resolution.resolved().len(), 1);
            assert_eq!(
                resolution.resolved()[0].target(),
                &ResolverModuleId::new(PackageId::new("alpha"), ModulePath::new("leaf"))
            );
            assert!(loaded.tokens.tokens().iter().any(|token| {
                token.text.as_ref() == "combine" && token.kind == TokenKind::UserSymbol
            }));
        }
    }
}

#[test]
fn supplied_workspace_leaf_rejects_absent_unused_or_wrong_source_parent() {
    let leaf =
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
    for mode in ["absent", "unused", "wrong_source"] {
        let importer = if mode == "unused" {
            b"definition\nend;\n".as_slice()
        } else {
            b"import alpha.leaf;\ndefinition\nend;\n".as_slice()
        };
        let fixture = WorkspaceLeafFixture::new(importer, leaf, None, usize::MAX);
        if mode == "wrong_source" {
            let leaf_source = fixture.source("leaf");
            assert_eq!(leaf_source.status, PhaseStatus::Complete);
            let leaf_frontend = fixture.frontend("leaf", leaf_source.output_refs.clone(), None);
            assert_eq!(leaf_frontend.status, PhaseStatus::Complete);
            let result = fixture.frontend(
                "main",
                vec![
                    leaf_source.output_refs[0].clone(),
                    leaf_frontend.output_refs[0].clone(),
                ],
                None,
            );
            assert_eq!(result.status, PhaseStatus::Blocking);
            assert!(result.output_refs.is_empty());
            continue;
        }
        let (own_source, leaf_output) = fixture.clean_parents();
        let parents = match mode {
            "absent" => vec![own_source],
            "unused" => vec![own_source, leaf_output],
            _ => unreachable!(),
        };
        let result = fixture.frontend("main", parents, None);
        assert_eq!(result.status, PhaseStatus::Blocking, "{mode}");
        assert!(result.output_refs.is_empty(), "{mode}");
    }
}

#[test]
fn supplied_workspace_leaf_duplicate_handle_is_rejected_by_ir_bundle() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ndefinition\nend;\n",
        b"definition\nend;\n",
        None,
        usize::MAX,
    );
    let (own_source, leaf_output) = fixture.clean_parents();
    let snapshot = fixture.submission.session.captured().snapshot().id;
    let seal = |output| {
        mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
            &fixture.publisher,
            snapshot,
            output,
        )
        .unwrap()
    };
    assert!(
        PhaseDispatchInputBundle::new(
            snapshot,
            source_key(fixture.version("main")),
            Vec::new(),
            vec![
                seal(own_source),
                seal(leaf_output.clone()),
                seal(leaf_output)
            ],
        )
        .is_err()
    );
}

#[test]
fn supplied_workspace_leaf_rejects_tampered_captured_source_and_index_identity() {
    for mode in [
        "source_id",
        "source_hash",
        "source_path",
        "edition",
        "index_edition",
        "index_version",
    ] {
        let mut fixture = WorkspaceLeafFixture::new(
            b"import alpha.leaf;\ndefinition\nend;\n",
            b"definition\nend;\n",
            None,
            usize::MAX,
        );
        let (own_source, leaf_output) = fixture.clean_parents();
        let mut forged_snapshot = fixture.submission.session.captured().snapshot().clone();
        match mode {
            "source_id" => {
                forged_snapshot
                    .source_versions
                    .iter_mut()
                    .find(|version| version.module_path.as_str() == "leaf")
                    .unwrap()
                    .source_id = fixture.ids.next_source_id(snapshot_id(0x53)).unwrap()
            }
            "source_hash" => {
                forged_snapshot
                    .source_versions
                    .iter_mut()
                    .find(|version| version.module_path.as_str() == "leaf")
                    .unwrap()
                    .source_hash = hash(0xee)
            }
            "source_path" => {
                forged_snapshot
                    .source_versions
                    .iter_mut()
                    .find(|version| version.module_path.as_str() == "leaf")
                    .unwrap()
                    .normalized_path = fixture.fixture.version.normalized_path.clone()
            }
            "edition" => {
                forged_snapshot
                    .source_versions
                    .iter_mut()
                    .find(|version| version.module_path.as_str() == "leaf")
                    .unwrap()
                    .edition = Edition::new("2026")
            }
            "index_edition" => {
                fixture
                    .submission
                    .module_index
                    .as_mut()
                    .unwrap()
                    .modules
                    .iter_mut()
                    .find(|entry| entry.module.path.as_str() == "leaf")
                    .unwrap()
                    .edition = Edition::new("2026")
            }
            "index_version" => {
                fixture
                    .submission
                    .module_index
                    .as_mut()
                    .unwrap()
                    .packages
                    .iter_mut()
                    .find(|entry| entry.package_id.as_str() == "alpha")
                    .unwrap()
                    .version = "9.9.9".parse().unwrap()
            }
            _ => unreachable!(),
        }
        let result = fixture.frontend(
            "main",
            vec![own_source, leaf_output],
            Some(&forged_snapshot),
        );
        assert_eq!(result.status, PhaseStatus::Blocking, "{mode}");
        assert!(result.output_refs.is_empty(), "{mode}");
    }
}

#[test]
fn supplied_workspace_leaf_rejects_stale_parent() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ndefinition\nend;\n",
        b"definition\nend;\n",
        None,
        usize::MAX,
    );
    let (own_source, leaf_output) = fixture.clean_parents();
    let snapshot = fixture.submission.session.captured().snapshot().id;
    let sealed = [own_source, leaf_output]
        .into_iter()
        .map(|output| {
            mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
                &fixture.publisher,
                snapshot,
                output,
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let bundle = PhaseDispatchInputBundle::new(
        snapshot,
        source_key(fixture.version("main")),
        Vec::new(),
        sealed,
    )
    .unwrap();
    fixture.publisher.mark_obsolete(snapshot).unwrap();
    let result = fixture.frontend_bundle("main", bundle, None);
    assert_eq!(result.status, PhaseStatus::Blocking);
    assert!(result.output_refs.is_empty());
}

#[test]
fn supplied_workspace_leaf_private_only_exports_match_empty_lexical_environment() {
    let importer = b"import alpha.leaf;\ndefinition\nend;\n";
    let mut fingerprints = Vec::new();
    for leaf in [
        b"definition\nend;\n".as_slice(),
        b"definition\nlet x, y be set;\nprivate pred Hidden: x hidden y means thesis;\nend;\n"
            .as_slice(),
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n"
            .as_slice(),
    ] {
        let fixture = WorkspaceLeafFixture::new(importer, leaf, None, usize::MAX);
        let (own_source, leaf_output) = fixture.clean_parents();
        let result = fixture.frontend("main", vec![own_source, leaf_output], None);
        assert_eq!(result.status, PhaseStatus::Complete);
        assert!(result.diagnostics.is_empty());
        let typed = fixture
            .publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                &result.output_refs[0],
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let loaded = fixture.publisher.storage().get(&typed).unwrap();
        fingerprints.push(loaded.cache_keys.active_lexical_environment.stable_hash());
    }
    assert_eq!(fingerprints[0], fingerprints[1]);
    assert_ne!(fingerprints[0], fingerprints[2]);
}

#[test]
fn supplied_workspace_leaf_preserves_importer_diagnostics() {
    for (importer, expected) in [
        (
            b"import alpha.leaf, mml.no_such;\ndefinition\nend;\n".as_slice(),
            &[22, 220][..],
        ),
        (b"import alpha.leaf;\ndefinition\n".as_slice(), &[10][..]),
    ] {
        let fixture = WorkspaceLeafFixture::new(importer, b"definition\nend;\n", None, usize::MAX);
        let (own_source, leaf_output) = fixture.clean_parents();
        let result = fixture.frontend("main", vec![own_source, leaf_output], None);
        assert_eq!(result.status, PhaseStatus::Recoverable);
        assert!(result.output_refs.is_empty());
        let codes = result
            .diagnostics
            .iter()
            .flat_map(|batch| batch.drafts())
            .map(|draft| draft.code().number())
            .collect::<Vec<_>>();
        for code in expected {
            assert!(codes.contains(code), "missing E{code:04}: {codes:?}");
        }
    }
}

#[test]
fn supplied_workspace_leaf_rejects_reexports_and_unsupported_lexical_shapes() {
    for (case, leaf) in [
        ("reexport", b"export Demo;\ndefinition\nend;\n".as_slice()),
        ("alias", b"definition\nlet X, Y be set;\nfunc Base: X \\+\\ Y -> set equals X;\nsynonym X <+> Y for X \\+\\ Y;\nend;\n".as_slice()),
        ("operator", b"definition\ninfix_operator(\"+\", left, 80);\nend;\n".as_slice()),
    ] {
        let fixture = WorkspaceLeafFixture::new(
            b"import alpha.leaf;\ndefinition\nend;\n", leaf, None, usize::MAX);
        let (own_source, leaf_output) = fixture.clean_parents();
        let result = fixture.frontend("main", vec![own_source, leaf_output], None);
        assert_eq!(result.status, PhaseStatus::Blocking, "{case}");
        assert!(result.output_refs.is_empty(), "{case}");
    }
}

#[test]
fn scheduled_workspace_intermediate_consumes_only_its_own_public_symbols() {
    let importer = b"import alpha.leaf;\ntheorem T: a arrange b = a;\n";
    let intermediate = b"import alpha.base;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: a combine b = a;\n";
    let mut lexical_hashes = Vec::new();
    for ancestor in [
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n".as_slice(),
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\npublic func Spare: x spare y -> set equals x;\nend;\n".as_slice(),
    ] {
        let fixture = WorkspaceLeafFixture::new(importer, intermediate, Some(ancestor), usize::MAX);
        let mut overlay = complete_leaf_import_overlay();
        overlay.edges.push(ModuleDependencyEdge::new(
            mizar_build::module_index::ModuleId::new(PackageId::new("alpha"), ModulePath::new("leaf")),
            mizar_build::module_index::ModuleId::new(PackageId::new("alpha"), ModulePath::new("base")),
            ModuleDependencyKind::ImportSummary,
        ));
        let (request, mut input) = fixture.scheduled_request(overlay, Vec::new(), Vec::new());
        input.worker_count = 4;
        let mut driver = fixture.scheduled_driver();
        let submission = driver
            .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap();
        assert_eq!(
            submission.session.captured().snapshot().id,
            fixture.submission.session.captured().snapshot().id
        );
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        for module in ["base", "leaf", "main"] {
            let task = fixture.task(TaskKind::Frontend, module);
            assert_eq!(run.phase_results.get(&task.id).unwrap()[0].status, PhaseStatus::Complete, "{module}");
        }
        let base = &run.phase_results[&fixture.task(TaskKind::Frontend, "base").id][0].output_refs[0];
        let leaf = &run.phase_results[&fixture.task(TaskKind::Frontend, "leaf").id][0].output_refs[0];
        let main = &run.phase_results[&fixture.task(TaskKind::Frontend, "main").id][0].output_refs[0];
        let leaf_source = &run.phase_results[&fixture.task(TaskKind::SourceLoad, "leaf").id][0].output_refs[0];
        let main_source = &run.phase_results[&fixture.task(TaskKind::SourceLoad, "main").id][0].output_refs[0];
        let leaf_lineage = fixture.publisher.registry().output_lineage(leaf.output()).unwrap();
        assert_eq!(leaf_lineage.parents.len(), 2);
        assert!(leaf_lineage.parents.contains(&leaf_source.output()));
        assert!(leaf_lineage.parents.contains(&base.output()));
        let main_lineage = fixture.publisher.registry().output_lineage(main.output()).unwrap();
        assert_eq!(main_lineage.parents.len(), 2);
        assert!(main_lineage.parents.contains(&main_source.output()));
        assert!(main_lineage.parents.contains(&leaf.output()));
        assert!(!main_lineage.parents.contains(&base.output()));
        let mut active_hashes = Vec::new();
        for (output, spelling) in [(leaf, "combine"), (main, "arrange")] {
            let typed = fixture.publisher.storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    output, &OutputKind::new("FrontendOutput")).unwrap();
            let loaded = fixture.publisher.storage().get(&typed).unwrap();
            assert!(loaded.tokens.tokens().iter().any(|token| {
                token.text.as_ref() == spelling && token.kind == TokenKind::UserSymbol
            }));
            active_hashes.push(loaded.cache_keys.active_lexical_environment.stable_hash());
        }
        lexical_hashes.push((active_hashes[0], active_hashes[1]));
    }
    assert_ne!(lexical_hashes[0].0, lexical_hashes[1].0);
    assert_eq!(lexical_hashes[0].1, lexical_hashes[1].1);
}

#[test]
fn supplied_workspace_intermediate_rejects_reexport_and_changed_captured_import_index() {
    for mode in ["reexport", "missing_import_index"] {
        let intermediate = if mode == "reexport" {
            b"import alpha.base;\nexport alpha.base;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: a combine b = a;\n".as_slice()
        } else {
            b"import alpha.base;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: a combine b = a;\n".as_slice()
        };
        let mut fixture = WorkspaceLeafFixture::new(
            b"import alpha.leaf;\ntheorem T: a arrange b = a;\n",
            intermediate,
            Some(b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n"),
            usize::MAX,
        );
        let base_source = fixture.source("base");
        assert_eq!(base_source.status, PhaseStatus::Complete);
        let base_frontend = fixture.frontend("base", base_source.output_refs, None);
        assert_eq!(base_frontend.status, PhaseStatus::Complete);
        let leaf_source = fixture.source("leaf");
        assert_eq!(leaf_source.status, PhaseStatus::Complete);
        let leaf_frontend = fixture.frontend(
            "leaf",
            vec![
                leaf_source.output_refs[0].clone(),
                base_frontend.output_refs[0].clone(),
            ],
            None,
        );
        assert_eq!(leaf_frontend.status, PhaseStatus::Complete, "{mode}");
        assert!(leaf_frontend.diagnostics.is_empty(), "{mode}");
        let main_source = fixture.source("main");
        assert_eq!(main_source.status, PhaseStatus::Complete);
        if mode == "missing_import_index" {
            fixture
                .submission
                .module_index
                .as_mut()
                .unwrap()
                .modules
                .retain(|entry| entry.module.path.as_str() != "base");
        }
        let result = fixture.frontend(
            "main",
            vec![
                main_source.output_refs[0].clone(),
                leaf_frontend.output_refs[0].clone(),
            ],
            None,
        );
        assert_eq!(result.status, PhaseStatus::Blocking, "{mode}");
        assert!(result.output_refs.is_empty(), "{mode}");
    }
}

#[test]
fn scheduled_cross_package_workspace_parent_uses_target_owned_lexical_identity() {
    let mut active_hashes = Vec::new();
    for (core_version, chain, mutation) in [
        ("1.2.0", false, "none"),
        ("2.0.0", false, "none"),
        ("1.2.0", true, "none"),
        ("1.2.0", false, "plan_version"),
        ("1.2.0", false, "index_version"),
        ("1.2.0", false, "index_root"),
        ("1.2.0", false, "source_version"),
    ] {
        let fixture = Fixture::new(b"definition\nend;\n");
        fs::create_dir_all(fixture.root.join("app/src")).unwrap();
        fs::create_dir_all(fixture.root.join("core/src")).unwrap();
        fs::write(
            fixture.root.join("app/src/main.miz"),
            if chain {
                b"import app.mid;\ntheorem T: a arrange b = a;\n".as_slice()
            } else {
                b"import core.base;\ntheorem T: a combine b = a;\n".as_slice()
            },
        )
        .unwrap();
        if chain {
            fs::write(
                fixture.root.join("app/src/mid.miz"),
                b"import core.base;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: a combine b = a;\n",
            )
            .unwrap();
        }
        fs::write(
            fixture.root.join("core/src/base.miz"),
            b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
        )
        .unwrap();
        let ids = InMemorySessionIdAllocator::new();
        let version = |package: &str, module: &str| {
            let root = fixture.root.join(package);
            let relative = format!("src/{module}.miz");
            let loaded = mizar_session::SourceLoader::load(
                &DiskSourceLoader::new(&root),
                snapshot_id(0x52),
                SourceInput {
                    package_id: PackageId::new(package),
                    module_path: ModulePath::new(module),
                    normalized_path: normalize_path(&root, Path::new(&relative)).unwrap(),
                    edition: Edition::new("2025"),
                    origin: SourceOriginInput::Disk {
                        path: PathBuf::from(&relative),
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
        };
        let mut versions = vec![version("app", "main"), version("core", "base")];
        if chain {
            versions.push(version("app", "mid"));
        }
        let make_request = || {
            let (mut request, mut input) = fixture.submit_request(Vec::new(), Vec::new());
            request.source_inputs.versions = versions.clone();
            input.workspace_packages = vec![
                WorkspacePackage {
                    member_path: "app".to_owned(),
                    manifest: parse_package_manifest(&format!(
                        "[package]\nname = \"app\"\nversion = \"0.1.0\"\n[dependencies]\ncore = \"{core_version}\"\n"
                    )).unwrap(),
                },
                WorkspacePackage {
                    member_path: "core".to_owned(),
                    manifest: parse_package_manifest(&format!(
                        "[package]\nname = \"core\"\nversion = \"{core_version}\"\n"
                    )).unwrap(),
                },
            ];
            input.lockfile = parse_lockfile(&format!(
                "schema_version = 1\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = \"app\" }}\ndependencies = [{{ name = \"core\", version = \"{core_version}\" }}]\n[[package]]\nname = \"core\"\nversion = \"{core_version}\"\nsource = {{ kind = \"workspace\", path = \"core\" }}\ndependencies = []\n"
            )).unwrap();
            let mut app_files = vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")];
            if chain {
                app_files.push(WorkspaceSourceFile::new("src/mid.miz", "mid.miz"));
            }
            input.source_layout = StaticSourceLayout::new(vec![
                WorkspaceSourcePackage {
                    package_id: PackageId::new("app"),
                    files: app_files,
                },
                WorkspaceSourcePackage {
                    package_id: PackageId::new("core"),
                    files: vec![WorkspaceSourceFile::new("src/base.miz", "base.miz")],
                },
            ]);
            let app_main = mizar_build::module_index::ModuleId::new(
                PackageId::new("app"),
                ModulePath::new("main"),
            );
            let core_base = mizar_build::module_index::ModuleId::new(
                PackageId::new("core"),
                ModulePath::new("base"),
            );
            let mut edges = Vec::new();
            if chain {
                let app_mid = mizar_build::module_index::ModuleId::new(
                    PackageId::new("app"),
                    ModulePath::new("mid"),
                );
                edges.push(ModuleDependencyEdge::new(
                    app_main,
                    app_mid.clone(),
                    ModuleDependencyKind::ImportSummary,
                ));
                edges.push(ModuleDependencyEdge::new(
                    app_mid,
                    core_base,
                    ModuleDependencyKind::ImportSummary,
                ));
            } else {
                edges.push(ModuleDependencyEdge::new(
                    app_main,
                    core_base,
                    ModuleDependencyKind::ImportSummary,
                ));
            }
            input.dependency_overlay = ModuleDependencyOverlay::complete(edges);
            (request, input)
        };
        let (request, input) = make_request();
        let baseline = CompilerDriver::new(source_registry())
            .submit(request, &ids, &SnapshotRegistry::new(), input)
            .unwrap();
        assert_eq!(
            baseline.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let tasks = baseline.task_graph.as_ref().unwrap().tasks();
        let first = tasks
            .iter()
            .find(|task| task.kind == TaskKind::SourceLoad)
            .unwrap();
        let publisher = publisher(
            baseline.session.captured().snapshot().id,
            usize::MAX,
            &first.unit,
        );
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
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(Vec::new());
        let registry = builder.build().unwrap();
        if mutation != "none" {
            let mut direct = WorkspaceLeafFixture {
                fixture,
                ids,
                submission: baseline,
                publisher,
            };
            let core_source = direct.source("base");
            assert_eq!(core_source.status, PhaseStatus::Complete);
            let core_frontend = direct.frontend("base", core_source.output_refs, None);
            let mut forged_snapshot = direct.submission.session.captured().snapshot().clone();
            assert_eq!(core_frontend.status, PhaseStatus::Complete);
            let app_source = direct.source("main");
            assert_eq!(app_source.status, PhaseStatus::Complete);
            match mutation {
                "plan_version" => {
                    direct
                        .submission
                        .build_plan
                        .as_mut()
                        .unwrap()
                        .packages
                        .iter_mut()
                        .find(|entry| entry.package_id.as_str() == "core")
                        .unwrap()
                        .version = "9.9.9".parse().unwrap();
                }
                "index_version" => {
                    direct
                        .submission
                        .module_index
                        .as_mut()
                        .unwrap()
                        .packages
                        .iter_mut()
                        .find(|entry| entry.package_id.as_str() == "core")
                        .unwrap()
                        .version = "9.9.9".parse().unwrap();
                }
                "index_root" => {
                    let entry = direct
                        .submission
                        .module_index
                        .as_mut()
                        .unwrap()
                        .packages
                        .iter_mut()
                        .find(|entry| entry.package_id.as_str() == "core")
                        .unwrap();
                    let mizar_build::module_index::PackageIndexSource::Workspace {
                        package_root,
                        ..
                    } = &mut entry.source
                    else {
                        unreachable!()
                    };
                    *package_root = "app".to_owned();
                }
                "source_version" => {
                    forged_snapshot
                        .source_versions
                        .iter_mut()
                        .find(|version| version.package_id.as_str() == "core")
                        .unwrap()
                        .source_hash = hash(0xee);
                }
                _ => unreachable!(),
            }
            let result = direct.frontend(
                "main",
                vec![
                    app_source.output_refs[0].clone(),
                    core_frontend.output_refs[0].clone(),
                ],
                Some(&forged_snapshot),
            );
            assert_eq!(result.status, PhaseStatus::Blocking, "{mutation}");
            assert!(result.output_refs.is_empty(), "{mutation}");
            continue;
        }
        let mut driver = CompilerDriver::new(registry).with_output_publisher(publisher.clone());
        let (request, mut input) = make_request();
        input.worker_count = 4;
        let submission = driver
            .submit(request, &ids, &SnapshotRegistry::new(), input)
            .unwrap();
        assert_eq!(
            submission.session.captured().snapshot().id,
            baseline.session.captured().snapshot().id
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        let graph = submission.task_graph.as_ref().unwrap();
        let task = |kind, package: &str, module: &str| {
            graph
                .tasks()
                .iter()
                .find(|task| {
                    task.kind == kind
                        && matches!(&task.unit,
                WorkUnit::Module { module: id }
                if id.package.as_str() == package && id.path.as_str() == module)
                })
                .unwrap()
        };
        let core = task(TaskKind::Frontend, "core", "base");
        let app = task(TaskKind::Frontend, "app", "main");
        assert_eq!(run.phase_results[&core.id][0].status, PhaseStatus::Complete);
        if chain {
            assert_eq!(
                run.phase_results[&task(TaskKind::Frontend, "app", "mid").id][0].status,
                PhaseStatus::Complete
            );
        }
        assert_eq!(run.phase_results[&app.id][0].status, PhaseStatus::Complete);
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let core_output = &run.phase_results[&core.id][0].output_refs[0];
        let app_output = &run.phase_results[&app.id][0].output_refs[0];
        let own_source =
            &run.phase_results[&task(TaskKind::SourceLoad, "app", "main").id][0].output_refs[0];
        let lineage = publisher
            .registry()
            .output_lineage(app_output.output())
            .unwrap();
        assert_eq!(lineage.parents.len(), 2);
        assert!(lineage.parents.contains(&own_source.output()));
        if chain {
            let mid =
                &run.phase_results[&task(TaskKind::Frontend, "app", "mid").id][0].output_refs[0];
            let mid_source =
                &run.phase_results[&task(TaskKind::SourceLoad, "app", "mid").id][0].output_refs[0];
            assert!(lineage.parents.contains(&mid.output()));
            assert!(!lineage.parents.contains(&core_output.output()));
            let mid_lineage = publisher.registry().output_lineage(mid.output()).unwrap();
            assert_eq!(mid_lineage.parents.len(), 2);
            assert!(mid_lineage.parents.contains(&mid_source.output()));
            assert!(mid_lineage.parents.contains(&core_output.output()));
            let typed_mid = publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    mid,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let loaded_mid = publisher.storage().get(&typed_mid).unwrap();
            assert!(
                loaded_mid
                    .tokens
                    .tokens()
                    .iter()
                    .any(|token| token.text.as_ref() == "combine"
                        && token.kind == TokenKind::UserSymbol)
            );
        } else {
            assert!(lineage.parents.contains(&core_output.output()));
        }
        let typed = publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                app_output,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let loaded = publisher.storage().get(&typed).unwrap();
        let spelling = if chain { "arrange" } else { "combine" };
        assert!(loaded.tokens.tokens().iter().any(|token| token.text.as_ref() == spelling
            && token.kind == TokenKind::UserSymbol));
        if !chain {
            active_hashes.push(loaded.cache_keys.active_lexical_environment.stable_hash());
        }
    }
    assert_ne!(active_hashes[0], active_hashes[1]);
}

#[test]
fn scheduled_artifact_importing_intermediate_exports_only_its_own_symbol() {
    use mizar_artifact::{
        module_summary::{
            ExportedSymbolSummary, LexicalContributionSummary, ModuleLexicalSummary, ModuleSummary,
            SourceRangeSummary, current_schema_version, module_summary_json,
        },
        store::{PublishedArtifactPath, artifact_hash_domain, write_published_artifact},
    };
    use mizar_frontend::lexical_env::{
        ExportRank, ExportedSymbolShape, ModuleId as LexerModuleId, SymbolId, UserSymbolArity,
        UserSymbolKind,
    };

    let mut lexical_hashes = Vec::new();
    for mode in ["base", "unused_public_symbol", "missing_core_artifact"] {
        let mut fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ntheorem T: a arrange b = a;\n",
        b"import dep.core;\ndefinition\nlet x, y be set;\npublic func Infix: x arrange y -> set equals x;\nend;\ntheorem U: combine(a,b) = a;\n",
        None,
        usize::MAX,
    );
        let root = fixture.fixture.root.join("dep-artifacts");
        fs::create_dir_all(&root).unwrap();
        let identity = ModuleSummaryIdentity {
            package_id: "dep".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("dependency-lock".to_owned()),
            module_path: "core".to_owned(),
            language_edition: "2025".to_owned(),
        };
        let origin = "symbol:combine";
        let shape = ExportedSymbolShape {
            spelling: "combine".to_owned(),
            symbol_id: SymbolId::new(canonical_json_string(&CanonicalJson::array([
                identity.canonical_json().unwrap(),
                CanonicalJson::string(origin),
            ]))),
            source_module: LexerModuleId::new(canonical_json_string(
                &identity.canonical_json().unwrap(),
            )),
            export_rank: ExportRank::new(0),
            kind: UserSymbolKind::Functor,
            arity: UserSymbolArity::exact(2),
            operator: None,
        };
        let mut core = ModuleSummary {
            schema_version: current_schema_version(),
            module: identity,
            source_hash: hash(9),
            interface_hash: hash(0),
            exported_symbols: vec![ExportedSymbolSummary {
                origin_id: origin.to_owned(),
                fully_qualified_name: "core.combine".to_owned(),
                namespace_path: vec!["core".to_owned()],
                visibility: "public".to_owned(),
                declaration_kind: "functor".to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: 0,
                    end_byte: 1,
                },
                rendered_signature: "functor core.combine".to_owned(),
                interface_fingerprint: hash(8),
                proof_status: None,
            }],
            exported_labels: Vec::new(),
            lexical_summary: ModuleLexicalSummary {
                schema_version: "mizar-resolve/exported-lexical/v1".to_owned(),
                fingerprint: None,
                contributions: vec![LexicalContributionSummary {
                    kind: "exported-symbol".to_owned(),
                    key: origin.to_owned(),
                    payload: String::from_utf8(shape.canonical_bytes().unwrap()).unwrap(),
                }],
            },
            reexports: Vec::new(),
            dependency_interfaces: Vec::new(),
        };
        if mode == "unused_public_symbol" {
            let origin = "symbol:spare";
            let shape = ExportedSymbolShape {
                spelling: "spare".to_owned(),
                symbol_id: SymbolId::new(canonical_json_string(&CanonicalJson::array([
                    core.module.canonical_json().unwrap(),
                    CanonicalJson::string(origin),
                ]))),
                source_module: LexerModuleId::new(canonical_json_string(
                    &core.module.canonical_json().unwrap(),
                )),
                export_rank: ExportRank::new(1),
                kind: UserSymbolKind::Functor,
                arity: UserSymbolArity::exact(2),
                operator: None,
            };
            core.exported_symbols.push(ExportedSymbolSummary {
                origin_id: origin.to_owned(),
                fully_qualified_name: "core.spare".to_owned(),
                namespace_path: vec!["core".to_owned()],
                visibility: "public".to_owned(),
                declaration_kind: "functor".to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: 2,
                    end_byte: 3,
                },
                rendered_signature: "functor core.spare".to_owned(),
                interface_fingerprint: hash(7),
                proof_status: None,
            });
            core.lexical_summary
                .contributions
                .push(LexicalContributionSummary {
                    kind: "exported-symbol".to_owned(),
                    key: origin.to_owned(),
                    payload: String::from_utf8(shape.canonical_bytes().unwrap()).unwrap(),
                });
        }
        core.refresh_interface_hash().unwrap();
        let mut other = core.clone();
        other.module.module_path = "other".to_owned();
        other.exported_symbols.clear();
        other.lexical_summary.contributions.clear();
        other.refresh_interface_hash().unwrap();
        let mut references = Vec::new();
        let mut captured = Vec::new();
        for summary in [core, other] {
            let module = summary.module.module_path.as_str();
            let artifact = format!("dep/{module}.summary.json");
            let content_hash = write_published_artifact(
                &root,
                &PublishedArtifactPath::new(&artifact).unwrap(),
                &module_summary_json(&summary).unwrap(),
                &artifact_hash_domain(
                    mizar_artifact::module_summary::MODULE_SUMMARY_SCHEMA_FAMILY,
                    summary.schema_version,
                ),
                &[],
            )
            .unwrap()
            .artifact_hash;
            references.push(DependencyModuleSummaryRef {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("dep"),
                    ModulePath::new(module),
                ),
                artifact: artifact.clone(),
                content_hash,
            });
            captured.push(DependencyArtifactRef::new(artifact, content_hash));
        }
        references.sort_by(|left, right| {
            right
                .content_hash
                .as_bytes()
                .cmp(left.content_hash.as_bytes())
        });
        let indexed = vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            references,
        )];
        let (request, input) = fixture.scheduled_request(
            complete_leaf_import_overlay(),
            captured.clone(),
            indexed.clone(),
        );
        let baseline = CompilerDriver::new(source_registry())
            .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap();
        fixture
            .publisher
            .register_current_snapshot(baseline.session.captured().snapshot().id);
        fixture.submission = baseline;
        if mode == "missing_core_artifact" {
            fs::remove_file(root.join("dep/core.summary.json")).unwrap();
        }
        let (request, mut input) =
            fixture.scheduled_request(complete_leaf_import_overlay(), captured, indexed);
        input.worker_count = 4;
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![(PackageId::new("dep"), root)]);
        let mut driver = CompilerDriver::new(builder.build().unwrap())
            .with_output_publisher(fixture.publisher.clone());
        let submission = driver
            .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
            .unwrap();
        assert_eq!(
            submission.session.captured().snapshot().id,
            fixture.submission.session.captured().snapshot().id
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        if mode == "missing_core_artifact" {
            let leaf_result = &run.phase_results[&fixture.task(TaskKind::Frontend, "leaf").id][0];
            assert_ne!(leaf_result.status, PhaseStatus::Complete);
            assert!(leaf_result.output_refs.is_empty());
            assert!(
                run.phase_results
                    .get(&fixture.task(TaskKind::Frontend, "main").id)
                    .is_none_or(|results| results
                        .iter()
                        .all(|result| result.output_refs.is_empty()))
            );
            continue;
        }
        for module in ["leaf", "main"] {
            let result = &run.phase_results[&fixture.task(TaskKind::Frontend, module).id][0];
            assert_eq!(
                result.status,
                PhaseStatus::Complete,
                "{module}: {:?}",
                result.diagnostics
            );
        }
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let leaf =
            &run.phase_results[&fixture.task(TaskKind::Frontend, "leaf").id][0].output_refs[0];
        let main =
            &run.phase_results[&fixture.task(TaskKind::Frontend, "main").id][0].output_refs[0];
        let leaf_source =
            &run.phase_results[&fixture.task(TaskKind::SourceLoad, "leaf").id][0].output_refs[0];
        let main_source =
            &run.phase_results[&fixture.task(TaskKind::SourceLoad, "main").id][0].output_refs[0];
        let leaf_lineage = fixture
            .publisher
            .registry()
            .output_lineage(leaf.output())
            .unwrap();
        assert_eq!(leaf_lineage.parents, vec![leaf_source.output()]);
        let main_lineage = fixture
            .publisher
            .registry()
            .output_lineage(main.output())
            .unwrap();
        assert_eq!(main_lineage.parents.len(), 2);
        assert!(main_lineage.parents.contains(&main_source.output()));
        assert!(main_lineage.parents.contains(&leaf.output()));
        let hashes = fixture
            .submission
            .module_index
            .as_ref()
            .unwrap()
            .dependency_summaries
            .iter()
            .map(|reference| reference.content_hash)
            .collect::<Vec<_>>();
        assert_eq!(hashes.len(), 2);
        let mut sorted = hashes.clone();
        sorted.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        assert_ne!(hashes, sorted);
        assert_eq!(
            leaf_lineage
                .named_input_hashes
                .iter()
                .map(|input| input.name.as_str())
                .collect::<Vec<_>>(),
            [
                "active-lexical-environment",
                "dependency.0",
                "dependency.1",
                "tokens"
            ]
        );
        assert_eq!(leaf_lineage.named_input_hashes[1].digest, sorted[0]);
        assert_eq!(leaf_lineage.named_input_hashes[2].digest, sorted[1]);
        let mut active_hashes = Vec::new();
        for (output, spelling) in [(leaf, "combine"), (main, "arrange")] {
            let typed = fixture
                .publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    output,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let loaded = fixture.publisher.storage().get(&typed).unwrap();
            assert!(loaded.tokens.tokens().iter().any(|token| {
                token.text.as_ref() == spelling && token.kind == TokenKind::UserSymbol
            }));
            active_hashes.push(loaded.cache_keys.active_lexical_environment.stable_hash());
        }
        lexical_hashes.push((active_hashes[0], active_hashes[1]));
    }
    assert_ne!(lexical_hashes[0].0, lexical_hashes[1].0);
    assert_eq!(lexical_hashes[0].1, lexical_hashes[1].1);
}

#[test]
fn workspace_leaf_is_not_injected_by_ordinary_no_provider_submit() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ndefinition\nend;\n",
        b"definition\nend;\n",
        None,
        usize::MAX,
    );
    let (request, input) = fixture.scheduled_request(
        ModuleDependencyOverlay::complete(Vec::new()),
        Vec::new(),
        Vec::new(),
    );
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    let frontend = submission
        .task_graph
        .as_ref()
        .unwrap()
        .tasks()
        .iter()
        .find(|task| {
            task.kind == TaskKind::Frontend
                && matches!(&task.unit,
            WorkUnit::Module { module } if module.path.as_str() == "main")
        })
        .unwrap();
    let result = &submission
        .scheduler_run
        .as_ref()
        .unwrap()
        .phase_results
        .get(&frontend.id)
        .unwrap()[0];
    assert_eq!(result.status, PhaseStatus::Blocking);
    assert!(result.output_refs.is_empty());
}

#[test]
fn scheduled_complete_import_overlay_hands_off_real_leaf_output() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ntheorem T: a combine b = a;\n",
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
        None,
        usize::MAX,
    );
    let (request, mut input) =
        fixture.scheduled_request(complete_leaf_import_overlay(), Vec::new(), Vec::new());
    input.worker_count = 4;
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByMissingPhaseServices
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    let graph = submission.task_graph.as_ref().unwrap();
    let run = submission.scheduler_run.as_ref().unwrap();
    let leaf_frontend = graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend
        && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == "leaf")).unwrap();
    let importer_frontend = graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend
        && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == "main")).unwrap();
    let importer_source = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::SourceLoad && task.unit == importer_frontend.unit)
        .unwrap();
    assert!(importer_frontend.dependencies.contains(&leaf_frontend.id));
    for task in [leaf_frontend, importer_frontend] {
        assert_eq!(
            run.task_states
                .iter()
                .find(|state| state.task_id == task.id)
                .unwrap()
                .state,
            TaskState::Completed
        );
        assert_eq!(
            run.phase_results.get(&task.id).unwrap()[0].status,
            PhaseStatus::Complete
        );
        assert_eq!(
            run.phase_results.get(&task.id).unwrap()[0]
                .output_refs
                .len(),
            1
        );
    }
    let own = &run.phase_results.get(&importer_source.id).unwrap()[0].output_refs[0];
    let leaf = &run.phase_results.get(&leaf_frontend.id).unwrap()[0].output_refs[0];
    let output = &run.phase_results.get(&importer_frontend.id).unwrap()[0].output_refs[0];
    let lineage = fixture
        .publisher
        .registry()
        .output_lineage(output.output())
        .unwrap();
    assert_eq!(lineage.parents.len(), 2);
    assert!(lineage.parents.contains(&own.output()));
    assert!(lineage.parents.contains(&leaf.output()));
    let typed = fixture
        .publisher
        .storage()
        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
            output,
            &OutputKind::new("FrontendOutput"),
        )
        .unwrap();
    let loaded = fixture.publisher.storage().get(&typed).unwrap();
    assert!(
        loaded.tokens.tokens().iter().any(|token| {
            token.text.as_ref() == "combine" && token.kind == TokenKind::UserSymbol
        })
    );
}

#[test]
fn scheduled_import_overlay_cannot_invent_a_source_import() {
    let fixture = WorkspaceLeafFixture::new(
        b"definition\nend;\n",
        b"definition\nend;\n",
        None,
        usize::MAX,
    );
    let (request, input) =
        fixture.scheduled_request(complete_leaf_import_overlay(), Vec::new(), Vec::new());
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    let run = submission.scheduler_run.as_ref().unwrap();
    let leaf = fixture.task(TaskKind::Frontend, "leaf");
    let importer = fixture.task(TaskKind::Frontend, "main");
    assert_eq!(
        run.phase_results.get(&leaf.id).unwrap()[0].status,
        PhaseStatus::Complete
    );
    let result = &run.phase_results.get(&importer.id).unwrap()[0];
    assert_eq!(result.status, PhaseStatus::Blocking);
    assert!(result.output_refs.is_empty());
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
}

#[test]
fn scheduled_import_overlay_does_not_synthesize_a_cached_leaf_output() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ntheorem T: a combine b = a;\n",
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
        None,
        usize::MAX,
    );
    let (request, mut input) =
        fixture.scheduled_request(complete_leaf_import_overlay(), Vec::new(), Vec::new());
    input.cache_policy = CacheSchedulingPolicy::Enabled;
    input.cache_decisions = CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
        fixture.task(TaskKind::Frontend, "leaf").id.clone(),
        CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
            vec![CacheOutputRef::new("leaf", "cached")],
            Vec::new(),
        )),
    )]);
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    let run = submission.scheduler_run.as_ref().unwrap();
    let leaf = fixture.task(TaskKind::Frontend, "leaf");
    let importer = fixture.task(TaskKind::Frontend, "main");
    assert_eq!(
        run.task_states
            .iter()
            .find(|state| state.task_id == leaf.id)
            .unwrap()
            .state,
        TaskState::CacheHit
    );
    assert!(!run.phase_results.contains_key(&leaf.id));
    assert_ne!(
        run.task_states
            .iter()
            .find(|state| state.task_id == importer.id)
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert!(
        run.phase_results
            .get(&importer.id)
            .is_none_or(|results| { results.iter().all(|result| result.output_refs.is_empty()) })
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
}

#[test]
fn scheduled_import_overlay_keeps_supplied_dispatch_provider_authoritative() {
    let fixture = WorkspaceLeafFixture::new(
        b"import alpha.leaf;\ntheorem T: a combine b = a;\n",
        b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n",
        None,
        usize::MAX,
    );
    let (request, mut input) =
        fixture.scheduled_request(complete_leaf_import_overlay(), Vec::new(), Vec::new());
    input.phase_dispatch_inputs = Some(Box::new(SuppliedInput::None));
    let mut driver = fixture.scheduled_driver();
    let submission = driver
        .submit(request, &fixture.ids, &SnapshotRegistry::new(), input)
        .unwrap();
    assert_eq!(
        submission.session.captured().snapshot().id,
        fixture.submission.session.captured().snapshot().id
    );
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByPhaseDispatchGap
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    assert!(
        submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .is_empty()
    );
}

#[test]
fn scheduled_source_frontend_prefix_publishes_real_parent_and_blocks_later_services() {
    for (workers, cached_later) in [(1, false), (4, false), (1, true)] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let second = if workers == 4 {
            fs::write(
                fixture.root.join("alpha/src/other.miz"),
                b"definition\nend;\n",
            )
            .unwrap();
            let package_root = fixture.root.join("alpha");
            ids.next_source_id(snapshot_id(0x52)).unwrap();
            let loaded = mizar_session::SourceLoader::load(
                &DiskSourceLoader::new(&package_root),
                snapshot_id(0x52),
                SourceInput {
                    package_id: PackageId::new("alpha"),
                    module_path: ModulePath::new("other"),
                    normalized_path: normalize_path(&package_root, Path::new("src/other.miz"))
                        .unwrap(),
                    edition: Edition::new("2025"),
                    origin: SourceOriginInput::Disk {
                        path: PathBuf::from("src/other.miz"),
                    },
                },
                &ids,
            )
            .unwrap();
            Some(SourceVersion {
                source_id: loaded.source_id,
                package_id: loaded.package_id,
                module_path: loaded.module_path,
                normalized_path: loaded.normalized_path,
                source_hash: loaded.source_hash,
                edition: loaded.edition,
                origin: loaded.origin,
            })
        } else {
            None
        };
        let make_request = || {
            let (mut request, mut input) = fixture.submit_request(Vec::new(), Vec::new());
            if let Some(version) = &second {
                request.source_inputs.versions.push(version.clone());
                input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                    package_id: PackageId::new("alpha"),
                    files: vec![
                        WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                        WorkspaceSourceFile::new("src/other.miz", "other.miz"),
                    ],
                }]);
            }
            (request, input)
        };
        let (request, input) = make_request();
        let baseline = CompilerDriver::new(source_registry())
            .submit(request, &ids, &snapshots, input)
            .unwrap();
        assert_eq!(
            baseline.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let snapshot = baseline.session.captured().snapshot().id;
        let source_tasks = baseline
            .task_graph
            .as_ref()
            .unwrap()
            .tasks()
            .iter()
            .filter(|task| task.kind == TaskKind::SourceLoad)
            .collect::<Vec<_>>();
        assert_eq!(source_tasks.len(), if workers == 4 { 2 } else { 1 });
        let publisher = publisher(snapshot, usize::MAX, &source_tasks[0].unit);
        for source in &source_tasks {
            let WorkUnit::Module { module } = &source.unit else {
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
            allow_frontend(&publisher, &source.unit);
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(Vec::new());
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher.clone());
        let (request, mut input) = make_request();
        input.worker_count = workers;
        if cached_later {
            input.cache_policy = CacheSchedulingPolicy::Enabled;
            input.cache_decisions = CacheSchedulingPlan::new(
                baseline
                    .task_graph
                    .as_ref()
                    .unwrap()
                    .tasks()
                    .iter()
                    .filter(|task| !matches!(task.kind, TaskKind::SourceLoad | TaskKind::Frontend))
                    .map(|task| {
                        CacheTaskDecision::new(
                            task.id.clone(),
                            CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
                                vec![CacheOutputRef::new(task.id.as_str(), "cached")],
                                Vec::new(),
                            )),
                        )
                    })
                    .collect(),
            );
        }
        let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
        assert_eq!(submission.session.captured().snapshot().id, snapshot);
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Blocked)
        );
        assert!(
            submission
                .missing_services
                .iter()
                .any(|missing| { missing.phase == PipelinePhase::ModuleResolve })
        );
        assert!(
            driver
                .events(submission.session.id())
                .events()
                .iter()
                .any(|event| {
                    matches!(
                        event.kind,
                        BuildEventKind::PhaseServiceGap {
                            phase: PipelinePhase::ModuleResolve,
                            ..
                        }
                    )
                })
        );
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        if cached_later {
            for task in graph.tasks().iter().filter(|task| {
                submission
                    .missing_services
                    .iter()
                    .any(|missing| task.phases.contains(&missing.phase))
            }) {
                assert_ne!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == task.id)
                        .unwrap()
                        .state,
                    TaskState::CacheHit
                );
            }
        }
        for source in graph
            .tasks()
            .iter()
            .filter(|task| task.kind == TaskKind::SourceLoad)
        {
            let frontend = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::Frontend && task.unit == source.unit)
                .unwrap();
            let resolve = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::ModuleResolve && task.unit == source.unit)
                .unwrap();
            for task in [source, frontend] {
                assert_eq!(
                    run.task_states
                        .iter()
                        .find(|state| state.task_id == task.id)
                        .unwrap()
                        .state,
                    TaskState::Completed
                );
                assert_eq!(
                    run.phase_results.get(&task.id).unwrap()[0].status,
                    PhaseStatus::Complete
                );
                assert_eq!(
                    run.phase_results.get(&task.id).unwrap()[0]
                        .output_refs
                        .len(),
                    1
                );
            }
            assert_eq!(
                run.task_states
                    .iter()
                    .find(|state| state.task_id == resolve.id)
                    .unwrap()
                    .state,
                TaskState::Blocked
            );
            let source_output = &run.phase_results.get(&source.id).unwrap()[0].output_refs[0];
            let frontend_output = &run.phase_results.get(&frontend.id).unwrap()[0].output_refs[0];
            assert_eq!(source_output.output_kind(), &OutputKind::new("SourceUnit"));
            assert_eq!(
                frontend_output.output_kind(),
                &OutputKind::new("FrontendOutput")
            );
            assert_eq!(source_output.snapshot(), snapshot);
            assert_eq!(frontend_output.snapshot(), snapshot);
            assert_eq!(
                publisher
                    .registry()
                    .output_lineage(frontend_output.output())
                    .unwrap()
                    .parents,
                vec![source_output.output()]
            );
            assert!(publisher.storage().validate_handle(source_output).is_ok());
            assert!(publisher.storage().validate_handle(frontend_output).is_ok());
        }
    }
}

#[test]
fn scheduled_frontend_import_failure_retains_diagnostics_and_blocks_dependents() {
    let fixture = Fixture::new(b"import mml.no_such;\ndefinition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let baseline = fixture.submit(&ids, &snapshots);
    let snapshot = baseline.session.captured().snapshot().id;
    let source_unit = &source_task(&baseline).unit;
    let publisher = publisher(snapshot, usize::MAX, source_unit);
    allow_frontend(&publisher, source_unit);
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(Vec::new());
    let mut driver = CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
    let (request, input) = fixture.submit_request(Vec::new(), Vec::new());
    let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
    assert_eq!(submission.session.captured().snapshot().id, snapshot);
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::SchedulerValidated
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Failed)
    );
    let graph = submission.task_graph.as_ref().unwrap();
    let run = submission.scheduler_run.as_ref().unwrap();
    let source = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::SourceLoad)
        .unwrap();
    let frontend = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    let resolve = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::ModuleResolve)
        .unwrap();
    assert_eq!(
        run.phase_results.get(&source.id).unwrap()[0].status,
        PhaseStatus::Complete
    );
    assert_eq!(
        run.phase_results.get(&source.id).unwrap()[0]
            .output_refs
            .len(),
        1
    );
    assert_eq!(
        run.task_states
            .iter()
            .find(|state| state.task_id == frontend.id)
            .unwrap()
            .state,
        TaskState::Failed
    );
    assert_eq!(
        run.task_states
            .iter()
            .find(|state| state.task_id == resolve.id)
            .unwrap()
            .state,
        TaskState::Blocked
    );
    let result = &run.phase_results.get(&frontend.id).unwrap()[0];
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    let codes = result
        .diagnostics
        .iter()
        .flat_map(|batch| batch.drafts())
        .map(|draft| draft.code().number())
        .collect::<Vec<_>>();
    assert!(codes.contains(&22), "missing E0022: {codes:?}");
    assert!(codes.contains(&220), "missing E0220: {codes:?}");
}

#[test]
fn scheduled_prefix_requires_current_publisher_and_both_builtin_services() {
    for mode in [
        "missing_publisher",
        "obsolete_publisher",
        "missing_frontend",
    ] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let baseline = fixture.submit(&ids, &snapshots);
        let snapshot = baseline.session.captured().snapshot().id;
        let publisher = publisher(snapshot, usize::MAX, &source_task(&baseline).unit);
        allow_frontend(&publisher, &source_task(&baseline).unit);
        if mode == "obsolete_publisher" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        if mode != "missing_frontend" {
            builder.register_frontend(Vec::new());
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if mode != "missing_publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let (request, input) = fixture.submit_request(Vec::new(), Vec::new());
        let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
        assert_eq!(
            submission.session.captured().snapshot().id,
            snapshot,
            "{mode}"
        );
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices,
            "{mode}"
        );
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Blocked),
            "{mode}"
        );
        assert!(submission.scheduler_run.is_none(), "{mode}");
    }
}

#[test]
fn scheduled_prefix_respects_source_and_frontend_publication_rights() {
    for deny_source in [true, false] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let baseline = fixture.submit(&ids, &snapshots);
        let snapshot = baseline.session.captured().snapshot().id;
        let publisher = if deny_source {
            let publisher = std::sync::Arc::new(PhaseOutputPublisher::new(
                std::sync::Arc::new(IrStorageService::new()),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            ));
            publisher.register_current_snapshot(snapshot);
            publisher
        } else {
            publisher(snapshot, usize::MAX, &source_task(&baseline).unit)
        };
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(Vec::new());
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
        let (request, input) = fixture.submit_request(Vec::new(), Vec::new());
        let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
        assert_eq!(submission.session.captured().snapshot().id, snapshot);
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Blocked)
        );
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        let source = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::SourceLoad)
            .unwrap();
        let frontend = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::Frontend)
            .unwrap();
        let blocked = if deny_source { source } else { frontend };
        assert_eq!(
            run.task_states
                .iter()
                .find(|state| state.task_id == blocked.id)
                .unwrap()
                .state,
            TaskState::Blocked
        );
        let blocked_result = &run.phase_results.get(&blocked.id).unwrap()[0];
        assert_eq!(blocked_result.status, PhaseStatus::Blocking);
        assert!(blocked_result.output_refs.is_empty());
        if deny_source {
            assert!(!run.phase_results.contains_key(&frontend.id));
        } else {
            assert_eq!(
                run.phase_results.get(&source.id).unwrap()[0]
                    .output_refs
                    .len(),
                1
            );
        }
    }
}

#[test]
fn scheduled_prefix_never_replaces_supplied_dispatch_inputs() {
    for mode in [
        SuppliedInput::None,
        SuppliedInput::Error,
        SuppliedInput::ForeignSnapshot,
    ] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let baseline = fixture.submit(&ids, &snapshots);
        let snapshot = baseline.session.captured().snapshot().id;
        let publisher = publisher(snapshot, usize::MAX, &source_task(&baseline).unit);
        allow_frontend(&publisher, &source_task(&baseline).unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(Vec::new());
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
        let (request, mut input) = fixture.submit_request(Vec::new(), Vec::new());
        input.phase_dispatch_inputs = Some(Box::new(mode));
        let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
        assert_eq!(submission.session.captured().snapshot().id, snapshot);
        let (expected_state, expected_status, expected_task_state) = match mode {
            SuppliedInput::None => (
                BuildSessionOutcome::Blocked,
                DriverSubmissionStatus::BlockedByPhaseDispatchGap,
                TaskState::Blocked,
            ),
            SuppliedInput::Error | SuppliedInput::ForeignSnapshot => (
                BuildSessionOutcome::Failed,
                DriverSubmissionStatus::SchedulerValidated,
                TaskState::Failed,
            ),
        };
        assert_eq!(submission.status, expected_status, "{mode:?}");
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(expected_state),
            "{mode:?}"
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        let source = source_task(&submission);
        assert_eq!(
            run.task_states
                .iter()
                .find(|state| state.task_id == source.id)
                .unwrap()
                .state,
            expected_task_state,
            "{mode:?}"
        );
        assert!(
            run.phase_results.is_empty(),
            "{mode:?} must not fall back to captured source"
        );
    }
}

#[test]
fn scheduled_frontend_rejects_cache_hit_without_retained_source_parent() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let baseline = fixture.submit(&ids, &snapshots);
    let snapshot = baseline.session.captured().snapshot().id;
    let source = source_task(&baseline);
    let publisher = publisher(snapshot, usize::MAX, &source.unit);
    allow_frontend(&publisher, &source.unit);
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(Vec::new());
    let mut driver = CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
    let (request, mut input) = fixture.submit_request(Vec::new(), Vec::new());
    input.cache_policy = CacheSchedulingPolicy::Enabled;
    input.cache_decisions = CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
        source.id.clone(),
        CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
            vec![CacheOutputRef::new("source", "cached")],
            Vec::new(),
        )),
    )]);
    let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
    assert_eq!(submission.session.captured().snapshot().id, snapshot);
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByPhaseDispatchGap
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    let graph = submission.task_graph.as_ref().unwrap();
    let run = submission.scheduler_run.as_ref().unwrap();
    let frontend = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    assert_eq!(
        run.task_states
            .iter()
            .find(|state| state.task_id == source.id)
            .unwrap()
            .state,
        TaskState::CacheHit
    );
    assert_eq!(
        run.task_states
            .iter()
            .find(|state| state.task_id == frontend.id)
            .unwrap()
            .state,
        TaskState::Blocked
    );
    assert!(!run.phase_results.contains_key(&source.id));
    assert!(!run.phase_results.contains_key(&frontend.id));
    assert!(
        submission
            .dispatch_gap_phases
            .contains(&PipelinePhase::Frontend)
    );
}

#[test]
fn scheduled_frontend_uses_sorted_captured_dependency_summary_hashes() {
    let fixture = Fixture::new(b"import dep.core;\ndefinition\nend;\n");
    let references = [("core", hash(0xf0)), ("other", hash(0x10))];
    let captured = references
        .iter()
        .map(|(path, digest)| {
            DependencyArtifactRef::new(format!("dep/{path}.summary.json"), *digest)
        })
        .collect::<Vec<_>>();
    let indexed = vec![DependencyArtifactIndex::new(
        PackageId::new("dep"),
        Vec::new(),
        references
            .iter()
            .map(|(path, digest)| DependencyModuleSummaryRef {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("dep"),
                    ModulePath::new(*path),
                ),
                artifact: format!("dep/{path}.summary.json"),
                content_hash: *digest,
            })
            .collect(),
    )];
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let baseline =
        fixture.submit_with_dependencies(&ids, &snapshots, captured.clone(), indexed.clone());
    assert_eq!(
        baseline
            .module_index
            .as_ref()
            .unwrap()
            .dependency_summaries
            .iter()
            .map(|summary| summary.content_hash)
            .collect::<Vec<_>>(),
        vec![hash(0xf0), hash(0x10)]
    );
    let snapshot = baseline.session.captured().snapshot().id;
    let publisher = publisher(snapshot, usize::MAX, &source_task(&baseline).unit);
    allow_frontend(&publisher, &source_task(&baseline).unit);
    let artifact_root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&artifact_root).unwrap();
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(vec![(PackageId::new("dep"), artifact_root)]);
    let mut driver = CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
    let (request, input) = fixture.submit_request(captured, indexed);
    let submission = driver.submit(request, &ids, &snapshots, input).unwrap();
    assert_eq!(submission.session.captured().snapshot().id, snapshot);
    let graph = submission.task_graph.as_ref().unwrap();
    let frontend = graph
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    let run = submission.scheduler_run.as_ref().unwrap();
    let result = &run.phase_results.get(&frontend.id).unwrap()[0];
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert!(
        result
            .diagnostics
            .iter()
            .flat_map(|batch| batch.drafts())
            .any(|draft| draft.code().number() == 23)
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Failed)
    );
}

#[test]
fn source_load_publishes_resident_and_blob_with_captured_identity_and_maps() {
    for threshold in [usize::MAX, 3] {
        let fixture = Fixture::new(b"\xef\xbb\xbfalpha\r\nbeta\r\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (result, publisher) = execute(&submission, &ids, threshold);
        assert_eq!(result.status, PhaseStatus::Complete);
        assert_eq!(result.output_refs.len(), 1);
        let output = &result.output_refs[0];
        if threshold == usize::MAX {
            assert!(matches!(output.placement(), StoragePlacement::Resident));
        } else {
            assert!(matches!(output.placement(), StoragePlacement::Blob { .. }));
        }
        let typed = publisher
            .storage()
            .typed_handle::<SourceUnit>(output, &OutputKind::new("SourceUnit"))
            .unwrap();
        let source = publisher.storage().get(&typed).unwrap();
        assert_eq!(source.package_id, fixture.version.package_id);
        assert_eq!(source.module_path, fixture.version.module_path);
        assert_eq!(source.normalized_path, fixture.version.normalized_path);
        assert_eq!(source.edition, fixture.version.edition);
        assert_eq!(source.origin, fixture.version.origin);
        assert_eq!(source.file_path, PathBuf::from("src/main.miz"));
        assert_eq!(source.source_id, fixture.version.source_id);
        assert_eq!(source.source_hash, fixture.version.source_hash);
        assert_eq!(source.line_map, fixture.line_map);
        assert_eq!(source.loading_map, fixture.loading_map);
        assert_eq!(source.source_text.as_ref(), "alpha\nbeta\n");
    }
}

#[test]
fn frontend_publishes_only_clean_loaded_source_with_parent_and_storage_maps() {
    for threshold in [usize::MAX, 3] {
        let fixture = Fixture::new(b"\xef\xbb\xbfdefinition\r\nend;\r\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, threshold);
        assert_eq!(source.status, PhaseStatus::Complete);
        fixture.write(b"replaced after source publication");

        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Complete);
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.output_refs.len(), 1);
        let output = &result.output_refs[0];
        assert_eq!(output.phase(), &IrPipelinePhase::new("Frontend"));
        assert_eq!(output.output_kind(), &OutputKind::new("FrontendOutput"));
        assert_eq!(
            output.snapshot(),
            submission.session.captured().snapshot().id
        );
        assert!(
            matches!(
                output.placement(),
                StoragePlacement::Resident if threshold == usize::MAX
            ) || matches!(
                output.placement(),
                StoragePlacement::Blob { .. } if threshold != usize::MAX
            )
        );
        let lineage = publisher
            .registry()
            .output_lineage(output.output())
            .unwrap();
        assert_eq!(lineage.parents, vec![source.output_refs[0].output()]);
        assert!(
            lineage
                .named_input_hashes
                .iter()
                .any(|input| input.name == "active-lexical-environment")
        );
        assert!(
            lineage
                .named_input_hashes
                .iter()
                .any(|input| input.name == "tokens")
        );
        let tables = publisher.storage().side_tables_by_ref(output).unwrap();
        assert_eq!(tables.source_maps.len(), 1);
        assert_eq!(tables.source_maps[0].domain, "frontend-storage");
        let typed = publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                output,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let loaded = publisher.storage().get(&typed).unwrap();
        assert_eq!(loaded.source.source_id, fixture.version.source_id);
        assert_eq!(loaded.source.source_text.as_ref(), "definition\nend;\n");
        assert!(loaded.ast.is_some());
        assert!(loaded.diagnostics.is_empty());
        let wrong_parent = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![output.clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(wrong_parent.status, PhaseStatus::Blocking);
        assert!(wrong_parent.output_refs.is_empty());
    }
}

#[test]
fn sealed_frontend_output_feeds_source_lexical_contributions() {
    let text = b"definition\n\
        let x, y be set;\n\
        private pred Hidden: x hidden y means thesis;\n\
        public func Pair: |. x .| -> set equals x;\n\
        func Infix: x combine y -> set equals x;\n\
        mode Visible: Carrier is set;\nend;\n";
    let mut previous = None;
    for threshold in [usize::MAX, 3] {
        let fixture = Fixture::new(text);
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, threshold);
        assert_eq!(source.status, PhaseStatus::Complete);
        let frontend = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(frontend.status, PhaseStatus::Complete);
        assert!(frontend.diagnostics.is_empty());
        let output = frontend
            .output_refs
            .first()
            .expect("sealed frontend output");
        assert_eq!(frontend.output_refs.len(), 1);
        assert_eq!(
            matches!(output.placement(), StoragePlacement::Resident),
            threshold == usize::MAX
        );
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        let typed = publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                output,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let loaded = publisher.storage().get(&typed).unwrap();
        assert_eq!(loaded.source.source_id, fixture.version.source_id);
        assert_eq!(loaded.source.source_hash, fixture.version.source_hash);
        assert!(loaded.diagnostics.is_empty());
        let ast = loaded.ast.as_ref().expect("clean frontend AST");
        let module = ResolverModuleId::new(
            loaded.source.package_id.clone(),
            loaded.source.module_path.clone(),
        );
        let package = submission
            .build_plan
            .as_ref()
            .unwrap()
            .packages
            .iter()
            .find(|package| package.package_id == *module.package())
            .expect("planned source package");
        let identity = ModuleSummaryIdentity {
            package_id: loaded.source.package_id.as_str().to_owned(),
            package_version: Some(package.version.to_string()),
            lockfile_identity: None,
            module_path: loaded.source.module_path.as_str().to_owned(),
            language_edition: loaded.source.edition.as_str().to_owned(),
        };
        let shells = DeclarationShellCollector::new(ast, &module).collect();
        let collection = SignatureProjectionExtractor::new(
            ast,
            &shells,
            NamespacePath::new(module.path().as_str()),
        )
        .collect(&module);
        assert!(collection.diagnostics().is_empty());
        let pairs = collection
            .pair_frontend_lexical_declarations(&loaded)
            .expect("sealed source lexical pairs");
        assert_eq!(
            pairs
                .iter()
                .map(|(_, local)| (local.spelling.as_str(), local.export_rank.get()))
                .collect::<Vec<_>>(),
            [("|.", 1), (".|", 2), ("combine", 3), ("Carrier", 4)]
        );
        assert_eq!(loaded.tokens.local_declarations().user_symbols.len(), 5);
        let contributions = collection
            .export_frontend_lexical_contributions(&loaded, &identity)
            .expect("sealed source lexical contributions");
        assert_eq!(contributions.len(), pairs.len());
        let identity_json = identity.canonical_json().unwrap();
        let source_module = canonical_json_string(&identity_json);
        for ((entry, _), contribution) in pairs.iter().zip(&contributions) {
            assert_eq!(contribution.key, entry.symbol().local().as_str());
            let SourceAnchor::Range(origin) = entry.origin().anchor() else {
                panic!("source lexical origin must be a range");
            };
            assert_eq!(origin.source_id, loaded.source.source_id);
            let shape = mizar_frontend::lexical_env::ExportedSymbolShape::from_canonical_bytes(
                contribution.payload.as_bytes(),
            )
            .unwrap();
            assert_eq!(shape.source_module.as_str(), source_module);
            assert_eq!(
                shape.symbol_id.as_str(),
                canonical_json_string(&CanonicalJson::array([
                    identity_json.clone(),
                    CanonicalJson::string(entry.symbol().local().as_str()),
                ]))
            );
        }
        if let Some(previous) = &previous {
            assert_eq!(&contributions, previous);
        }
        previous = Some(contributions);
    }
}

#[test]
fn frontend_recovery_and_unrecoverable_input_emit_diagnostics_without_output() {
    for (text, status, code) in [
        (b"definition\n".as_slice(), PhaseStatus::Recoverable, 10),
        (b"end".as_slice(), PhaseStatus::Fatal, 52),
    ] {
        let fixture = Fixture::new(text);
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, status);
        assert!(result.output_refs.is_empty());
        assert_eq!(result.diagnostics.len(), 1);
        assert!(
            result.diagnostics[0]
                .drafts()
                .iter()
                .any(|draft| draft.code().number() == code)
        );
    }
}

#[test]
fn frontend_diagnostic_only_import_reports_five_path_failures_from_live_source() {
    for (text, code, dependency_modules, needs_dependency, paths, primary_text) in [
        (
            "import mml.no_such;\ndefinition\nend;\n",
            220,
            &[][..],
            false,
            &["mml.no_such"][..],
            "mml.no_such",
        ),
        (
            "import dep.missing;\ndefinition\nend;\n",
            221,
            &["present"][..],
            true,
            &["dep.missing"][..],
            "dep.missing",
        ),
        (
            "import ..common;\ndefinition\nend;\n",
            222,
            &[][..],
            false,
            &["..common"][..],
            "..common",
        ),
        (
            "import dep.core as shared, dep.core as shared, dep.other as shared;\ndefinition\nend;\n",
            223,
            &["core", "other"][..],
            true,
            &["dep.core", "dep.core", "dep.other"][..],
            "shared",
        ),
        (
            "import dep.core as mml;\ndefinition\nend;\n",
            224,
            &["core"][..],
            true,
            &["dep.core"][..],
            "mml",
        ),
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let references = dependency_modules
            .iter()
            .enumerate()
            .map(|(ordinal, path)| {
                let artifact = format!("dep/{path}.summary.json");
                let digest = hash(0x60 + ordinal as u8);
                (
                    DependencyModuleSummaryRef {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("dep"),
                            ModulePath::new(*path),
                        ),
                        artifact: artifact.clone(),
                        content_hash: digest,
                    },
                    DependencyArtifactRef::new(artifact, digest),
                )
            })
            .collect::<Vec<_>>();
        let mut dependency_hashes = references
            .iter()
            .map(|(indexed, _)| indexed.content_hash)
            .collect::<Vec<_>>();
        dependency_hashes.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        let submission = fixture.submit_with_dependencies(
            &ids,
            &snapshots,
            references
                .iter()
                .map(|(_, captured)| captured.clone())
                .collect(),
            if needs_dependency {
                vec![DependencyArtifactIndex::new(
                    PackageId::new("dep"),
                    Vec::new(),
                    references.into_iter().map(|(indexed, _)| indexed).collect(),
                )]
            } else {
                Vec::new()
            },
        );
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        assert_eq!(source.status, PhaseStatus::Complete, "{text}");
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), dependency_hashes),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Recoverable, "{text}");
        assert!(result.output_refs.is_empty(), "{text}");
        assert_eq!(result.diagnostics.len(), 2, "{text}");
        let frontend = &result.diagnostics[0];
        assert_eq!(frontend.scope().phase(), DiagnosticPhase::Frontend);
        assert_eq!(frontend.drafts().len(), paths.len());
        assert!(
            frontend
                .drafts()
                .iter()
                .all(|draft| draft.code().number() == 22)
        );
        let resolver = &result.diagnostics[1];
        assert_eq!(resolver.scope().phase(), DiagnosticPhase::Resolver);
        assert_eq!(resolver.drafts().len(), paths.len(), "{text}");
        assert!(resolver.drafts().iter().all(|draft| {
            draft.code().number() == code
                && draft.phase() == DiagnosticPhase::Resolver
                && draft.category() == FailureCategory::ResolveError
        }));
        for (ordinal, draft) in resolver.drafts().iter().enumerate() {
            assert_eq!(
                draft.stable_detail_key(),
                mizar_diagnostics::registry::DiagnosticRegistry::builtin()
                    .lookup(draft.code())
                    .unwrap()
                    .semantic_name
            );
            let details = draft.details().entries();
            assert_eq!(
                details.get("import.source_module"),
                Some(&DiagnosticDetailValue::List(vec![
                    DiagnosticDetailValue::String("alpha".to_owned()),
                    DiagnosticDetailValue::String("main".to_owned()),
                ])),
                "{text}"
            );
            assert_eq!(
                details.get("import.ordinal"),
                Some(&DiagnosticDetailValue::Integer(ordinal as i64)),
                "{text}"
            );
            assert_eq!(
                details.get("import.path"),
                Some(&DiagnosticDetailValue::String(paths[ordinal].to_owned()))
            );
            let DiagnosticPrimaryLocation::Span(primary) = draft.primary_location() else {
                panic!("semantic failure has a loaded-source primary span");
            };
            let SourceAnchor::Range(primary_range) = primary.anchor() else {
                panic!("semantic failure retains Range shape");
            };
            assert_eq!(primary_range.source_id, fixture.version.source_id);
            assert_eq!(&text[primary_range.start..primary_range.end], primary_text);
            if code == 223 {
                assert_eq!(
                    details.get("import.alias"),
                    Some(&DiagnosticDetailValue::String("shared".to_owned()))
                );
                assert_eq!(
                    details.get("import.target"),
                    Some(&DiagnosticDetailValue::List(vec![
                        DiagnosticDetailValue::String("dep".to_owned()),
                        DiagnosticDetailValue::String(
                            if ordinal == 2 { "other" } else { "core" }.to_owned(),
                        ),
                    ]))
                );
                let alias_offsets = text
                    .match_indices("shared")
                    .map(|(start, _)| start)
                    .collect::<Vec<_>>();
                assert_eq!(primary.range().start, alias_offsets[ordinal]);
                assert_eq!(
                    draft
                        .secondary_spans()
                        .iter()
                        .map(|span| match span.anchor() {
                            SourceAnchor::Range(range) => *range,
                            _ => panic!("peer retains Range shape"),
                        })
                        .collect::<Vec<_>>(),
                    alias_offsets
                        .into_iter()
                        .enumerate()
                        .filter_map(|(peer, start)| {
                            (peer != ordinal).then_some(mizar_session::SourceRange {
                                source_id: fixture.version.source_id,
                                start,
                                end: start + "shared".len(),
                            })
                        })
                        .collect::<Vec<_>>()
                );
            }
        }
    }
}

#[test]
fn frontend_diagnostic_only_import_keeps_distinct_branch_members() {
    let text = "import dep.{missing, absent};\ndefinition\nend;\n";
    let fixture = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let digest = hash(0x60);
    let artifact = "dep/present.summary.json";
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        vec![DependencyArtifactRef::new(artifact, digest)],
        vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            vec![DependencyModuleSummaryRef {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("dep"),
                    ModulePath::new("present"),
                ),
                artifact: artifact.to_owned(),
                content_hash: digest,
            }],
        )],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(source.status, PhaseStatus::Complete);
    let result = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), vec![digest]),
        Vec::new(),
        None,
    );
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert_eq!(result.diagnostics.len(), 2);
    let [first, second] = result.diagnostics[1].drafts() else {
        panic!("one E0221 draft per branch member");
    };
    for (ordinal, (draft, member)) in [(first, "missing"), (second, "absent")]
        .into_iter()
        .enumerate()
    {
        assert_eq!(draft.code().number(), 221);
        let DiagnosticPrimaryLocation::Span(primary) = draft.primary_location() else {
            panic!("branch member source span");
        };
        let SourceAnchor::Range(member_range) = primary.anchor() else {
            panic!("retained range anchor");
        };
        assert_eq!(&text[member_range.start..member_range.end], member);
        assert_eq!(draft.secondary_spans().len(), 1);
        assert_eq!(
            &text[draft.secondary_spans()[0].range().start..draft.secondary_spans()[0].range().end],
            "dep"
        );
        let details = draft.details().entries();
        assert_eq!(
            details.get("import.branch_member"),
            Some(&DiagnosticDetailValue::Source(*member_range))
        );
        assert_eq!(
            details.get("import.path"),
            Some(&DiagnosticDetailValue::String(format!("dep.{member}")))
        );
        assert_eq!(
            details.get("import.ordinal"),
            Some(&DiagnosticDetailValue::Integer(ordinal as i64))
        );
    }
}

#[test]
fn frontend_mixed_or_malformed_import_diagnostics_do_not_continue_resolution() {
    for text in [
        "import std., pkg.math as ;\ndefinition\nend;\n",
        "import mml.no_such;\ndefinition\n",
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        assert_eq!(source.status, PhaseStatus::Complete);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert!(matches!(
            result.status,
            PhaseStatus::Recoverable | PhaseStatus::Fatal
        ));
        assert!(result.output_refs.is_empty());
        assert_eq!(result.diagnostics.len(), 1, "{text}");
        let drafts = result.diagnostics[0].drafts();
        assert_eq!(
            result.diagnostics[0].scope().phase(),
            DiagnosticPhase::Frontend
        );
        assert!(
            drafts.iter().any(|draft| draft.code().number() == 22),
            "{text}"
        );
        assert!(
            drafts.iter().any(|draft| draft.code().number() != 22),
            "{text}"
        );
    }

    let fixture = Fixture::new(b"import mml.no_such, dep.core;\ndefinition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let artifact = "dep/core.summary.json";
    let digest = hash(0x60);
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        vec![DependencyArtifactRef::new(artifact, digest)],
        vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            vec![DependencyModuleSummaryRef {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("dep"),
                    ModulePath::new("core"),
                ),
                artifact: artifact.to_owned(),
                content_hash: digest,
            }],
        )],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(source.status, PhaseStatus::Complete);
    let artifact_root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&artifact_root).unwrap();
    let result = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), vec![digest]),
        vec![(PackageId::new("dep"), artifact_root)],
        None,
    );
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert_eq!(result.diagnostics.len(), 1);
    let codes = result.diagnostics[0]
        .drafts()
        .iter()
        .map(|draft| draft.code().number())
        .collect::<Vec<_>>();
    assert!(
        codes.contains(&22),
        "unresolved path keeps E0022: {codes:?}"
    );
    assert!(
        codes.contains(&23),
        "missing summary keeps E0023: {codes:?}"
    );
}

#[test]
fn frontend_same_normalized_source_separates_semantic_and_storage_identity() {
    let mut hashes = Vec::new();
    for bytes in [
        b"\xef\xbb\xbfdefinition\r\nend;\r\n".as_slice(),
        b"definition\nend;\n".as_slice(),
    ] {
        let fixture = Fixture::new(bytes);
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Complete);
        hashes.push((
            result.output_refs[0].content_hash(),
            result.output_refs[0].side_table_hash(),
        ));
    }
    assert_eq!(hashes[0].0, hashes[1].0);
    assert_ne!(hashes[0].1, hashes[1].1);
}

#[test]
fn frontend_rejects_missing_wrong_or_mismatched_parent_identities() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    let source_ref = source.output_refs[0].clone();
    for (parents, input_hash, dependencies) in [
        (Vec::new(), source_key(&fixture.version), Vec::new()),
        (vec![source_ref.clone()], hash(7), Vec::new()),
        (
            vec![source_ref.clone()],
            source_key(&fixture.version),
            vec![hash(8)],
        ),
    ] {
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            parents,
            (input_hash, dependencies),
            Vec::new(),
            None,
        );
        assert_eq!(result.status, PhaseStatus::Blocking);
        assert!(result.output_refs.is_empty());
    }
}

#[test]
fn frontend_imports_require_one_captured_summary_artifact_identity() {
    use mizar_artifact::{
        module_summary::{
            ModuleLexicalSummary, ModuleSummary, ModuleSummaryIdentity, current_schema_version,
            module_summary_json,
        },
        store::{PublishedArtifactPath, artifact_hash_domain, write_published_artifact},
    };

    let fixture = Fixture::new(b"import dep.core;\ndefinition\nend;\n");
    let root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&root).unwrap();
    let artifact = "dep/core.summary.json";
    let mut summary = ModuleSummary {
        schema_version: current_schema_version(),
        module: ModuleSummaryIdentity {
            package_id: "dep".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("dependency-lock".to_owned()),
            module_path: "core".to_owned(),
            language_edition: "2025".to_owned(),
        },
        source_hash: hash(9),
        interface_hash: hash(8),
        exported_symbols: Vec::new(),
        exported_labels: Vec::new(),
        lexical_summary: ModuleLexicalSummary {
            schema_version: "mizar-resolve/exported-lexical/v1".to_owned(),
            fingerprint: None,
            contributions: Vec::new(),
        },
        reexports: Vec::new(),
        dependency_interfaces: Vec::new(),
    };
    summary.refresh_interface_hash().unwrap();
    let content_hash = write_published_artifact(
        &root,
        &PublishedArtifactPath::new(artifact).unwrap(),
        &module_summary_json(&summary).unwrap(),
        &artifact_hash_domain(
            mizar_artifact::module_summary::MODULE_SUMMARY_SCHEMA_FAMILY,
            summary.schema_version,
        ),
        &[],
    )
    .unwrap()
    .artifact_hash;
    let indexed = DependencyArtifactIndex::new(
        PackageId::new("dep"),
        Vec::new(),
        vec![DependencyModuleSummaryRef {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("dep"),
                ModulePath::new("core"),
            ),
            artifact: artifact.to_owned(),
            content_hash,
        }],
    );
    let matching = DependencyArtifactRef::new(artifact, content_hash);
    for (captured, expected) in [
        (vec![matching.clone()], PhaseStatus::Complete),
        (Vec::new(), PhaseStatus::Blocking),
        (
            vec![DependencyArtifactRef::new(artifact, hash(0x98))],
            PhaseStatus::Blocking,
        ),
        (
            vec![matching.clone(), matching.clone()],
            PhaseStatus::Blocking,
        ),
        (
            vec![
                matching.clone(),
                DependencyArtifactRef::new(artifact, hash(0x98)),
            ],
            PhaseStatus::Complete,
        ),
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission =
            fixture.submit_with_dependencies(&ids, &snapshots, captured, vec![indexed.clone()]);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        assert_eq!(source.status, PhaseStatus::Complete);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), vec![content_hash]),
            vec![(PackageId::new("dep"), root.clone())],
            None,
        );
        assert_eq!(result.status, expected);
        assert_eq!(
            result.output_refs.len(),
            usize::from(expected == PhaseStatus::Complete)
        );
        if expected == PhaseStatus::Complete {
            assert!(result.diagnostics.is_empty());
            let typed = publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    &result.output_refs[0],
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let loaded = publisher.storage().get(&typed).unwrap();
            let candidates = ImportPathCandidate::from_surface_ast(
                loaded.ast.as_ref().expect("sealed import AST"),
            )
            .expect("parsed import candidates");
            let [candidate] = candidates.as_slice() else {
                panic!("one parsed import candidate");
            };
            assert_eq!(candidate.components(), ["dep", "core"]);
            assert_eq!(candidate.ordinal(), 0);
            assert_eq!(candidate.range().source_id, loaded.source.source_id);
            assert_eq!(
                &loaded.source.source_text[candidate.range().start..candidate.range().end],
                "dep.core"
            );
            let current = ResolverModuleId::new(
                loaded.source.package_id.clone(),
                loaded.source.module_path.clone(),
            );
            let resolution = ImportPathResolver::new(ModuleIndexInput::new(
                submission.module_index.as_ref().unwrap(),
            ))
            .resolve(&current, &candidates);
            assert!(resolution.unresolved().is_empty());
            let [resolved] = resolution.resolved() else {
                panic!("one resolved import");
            };
            assert_eq!(
                resolved.target(),
                &ResolverModuleId::new(PackageId::new("dep"), ModulePath::new("core"))
            );
            assert_eq!(resolved.range(), candidate.range());
            assert_eq!(resolved.ordinal(), candidate.ordinal());
            let lineage = publisher
                .registry()
                .output_lineage(result.output_refs[0].output())
                .unwrap();
            assert!(
                lineage
                    .named_input_hashes
                    .iter()
                    .any(|input| { input.name == "dependency.0" && input.digest == content_hash })
            );
        }
    }
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        vec![matching.clone()],
        vec![indexed.clone()],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    for mutation in 0..4 {
        let mut changed = submission.module_index.as_ref().unwrap().clone();
        let dependency_module = changed
            .modules
            .iter()
            .position(|entry| entry.module.package.as_str() == "dep")
            .unwrap();
        match mutation {
            0 => {
                let ModuleIndexLocation::DependencySummary { artifact, .. } =
                    &mut changed.modules[dependency_module].location
                else {
                    unreachable!()
                };
                *artifact = "dep/other.summary.json".to_owned();
            }
            1 => changed.dependency_summaries[0].artifact = "dep/other.summary.json".to_owned(),
            2 => changed
                .dependency_summaries
                .push(changed.dependency_summaries[0].clone()),
            3 => changed
                .modules
                .push(changed.modules[dependency_module].clone()),
            _ => unreachable!(),
        }
        let dependencies = changed
            .dependency_summaries
            .iter()
            .map(|reference| reference.content_hash)
            .collect();
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), dependencies),
            vec![(PackageId::new("dep"), root.clone())],
            Some(&changed),
        );
        assert_eq!(
            result.status,
            PhaseStatus::Blocking,
            "index mutation {mutation}"
        );
        assert!(result.output_refs.is_empty());
    }
    fs::remove_file(root.join(artifact)).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission =
        fixture.submit_with_dependencies(&ids, &snapshots, vec![matching], vec![indexed]);
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    let missing_summary = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), vec![content_hash]),
        vec![(PackageId::new("dep"), root)],
        None,
    );
    assert_eq!(missing_summary.status, PhaseStatus::Recoverable);
    assert!(missing_summary.output_refs.is_empty());
    assert!(
        missing_summary.diagnostics[0]
            .drafts()
            .iter()
            .any(|draft| draft.code().number() == 23)
    );
}

#[test]
fn frontend_imported_symbol_conflict_uses_real_captured_summaries() {
    use mizar_artifact::{
        module_summary::{
            ExportedSymbolSummary, LexicalContributionSummary, ModuleLexicalSummary, ModuleSummary,
            ModuleSummaryIdentity, SourceRangeSummary, current_schema_version, module_summary_json,
        },
        store::{
            CanonicalJson, PublishedArtifactPath, artifact_hash_domain, canonical_json_string,
            write_published_artifact,
        },
    };
    use mizar_frontend::lexical_env::{
        ExportRank, ExportedSymbolShape, ModuleId as LexerModuleId, SymbolId, UserSymbolArity,
        UserSymbolKind,
    };

    let fixture = Fixture::new(b"import dep.core, dep.other;\ndefinition\nend;\n");
    let root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&root).unwrap();
    let mut indexed = Vec::new();
    let mut captured = Vec::new();
    for module_path in ["core", "other"] {
        let module = ModuleSummaryIdentity {
            package_id: "dep".to_owned(),
            package_version: Some("1.0.0".to_owned()),
            lockfile_identity: Some("dependency-lock".to_owned()),
            module_path: module_path.to_owned(),
            language_edition: "2025".to_owned(),
        };
        let origin_id = "symbol:combine";
        let shape = ExportedSymbolShape {
            spelling: "combine".to_owned(),
            symbol_id: SymbolId::new(canonical_json_string(&CanonicalJson::array([
                module.canonical_json().unwrap(),
                CanonicalJson::string(origin_id),
            ]))),
            source_module: LexerModuleId::new(canonical_json_string(
                &module.canonical_json().unwrap(),
            )),
            export_rank: ExportRank::new(0),
            kind: UserSymbolKind::Functor,
            arity: UserSymbolArity::exact(2),
            operator: None,
        };
        let mut summary = ModuleSummary {
            schema_version: current_schema_version(),
            module,
            source_hash: hash(9),
            interface_hash: hash(8),
            exported_symbols: vec![ExportedSymbolSummary {
                origin_id: origin_id.to_owned(),
                fully_qualified_name: format!("{module_path}.combine"),
                namespace_path: vec![module_path.to_owned()],
                visibility: "public".to_owned(),
                declaration_kind: "functor".to_owned(),
                source_range: SourceRangeSummary {
                    start_byte: 0,
                    end_byte: 1,
                },
                rendered_signature: format!("functor {module_path}.combine"),
                interface_fingerprint: hash(8),
                proof_status: None,
            }],
            exported_labels: Vec::new(),
            lexical_summary: ModuleLexicalSummary {
                schema_version: "mizar-resolve/exported-lexical/v1".to_owned(),
                fingerprint: None,
                contributions: vec![LexicalContributionSummary {
                    kind: "exported-symbol".to_owned(),
                    key: origin_id.to_owned(),
                    payload: String::from_utf8(shape.canonical_bytes().unwrap()).unwrap(),
                }],
            },
            reexports: Vec::new(),
            dependency_interfaces: Vec::new(),
        };
        summary.refresh_interface_hash().unwrap();
        let artifact = format!("dep/{module_path}.summary.json");
        let content_hash = write_published_artifact(
            &root,
            &PublishedArtifactPath::new(&artifact).unwrap(),
            &module_summary_json(&summary).unwrap(),
            &artifact_hash_domain(
                mizar_artifact::module_summary::MODULE_SUMMARY_SCHEMA_FAMILY,
                summary.schema_version,
            ),
            &[],
        )
        .unwrap()
        .artifact_hash;
        indexed.push(DependencyModuleSummaryRef {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("dep"),
                ModulePath::new(module_path),
            ),
            artifact: artifact.clone(),
            content_hash,
        });
        captured.push(DependencyArtifactRef::new(artifact, content_hash));
    }
    let mut dependencies = captured
        .iter()
        .map(|item| item.content_hash)
        .collect::<Vec<_>>();
    dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit_with_dependencies(
        &ids,
        &snapshots,
        captured,
        vec![DependencyArtifactIndex::new(
            PackageId::new("dep"),
            Vec::new(),
            indexed,
        )],
    );
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(source.status, PhaseStatus::Complete);
    let result = execute_frontend(
        &submission,
        &ids,
        &publisher,
        vec![source.output_refs[0].clone()],
        (source_key(&fixture.version), dependencies),
        vec![(PackageId::new("dep"), root)],
        None,
    );
    assert_eq!(result.status, PhaseStatus::Recoverable);
    assert!(result.output_refs.is_empty());
    assert!(
        result.diagnostics[0]
            .drafts()
            .iter()
            .any(|draft| draft.code().number() == 24)
    );
}

#[test]
fn frontend_requires_current_resources_and_scopes_cancellation() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let (source, publisher) = execute(&submission, &ids, usize::MAX);
    let snapshot = submission.session.captured().snapshot().id;
    let task = submission
        .task_graph
        .as_ref()
        .unwrap()
        .tasks()
        .iter()
        .find(|task| task.kind == TaskKind::Frontend)
        .unwrap();
    let parent = mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
        &publisher,
        snapshot,
        source.output_refs[0].clone(),
    )
    .unwrap();
    let source_load = SourceLoadInputs {
        captured: None,
        snapshot: submission.session.captured().snapshot(),
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(Vec::new());
    let registry = builder.build().unwrap();
    let sink = || {
        DiagnosticSink::new(DiagnosticProducerScope::new(
            DiagnosticPhase::Frontend,
            snapshot,
            "frontend.run_loaded",
        ))
    };
    let run = |resources: PhaseExecutionResources<'_>| {
        registry
            .execute_phase_with_resources(
                PipelinePhase::Frontend,
                PhaseInput::new(
                    task.unit.clone(),
                    PhaseDispatchInputBundle::new(
                        snapshot,
                        source_key(&fixture.version),
                        Vec::new(),
                        vec![parent.clone()],
                    )
                    .unwrap(),
                ),
                resources,
            )
            .unwrap()
            .result
    };
    for resources in [
        PhaseExecutionResources {
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        },
        PhaseExecutionResources {
            diagnostics: Some(sink()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        },
        PhaseExecutionResources {
            diagnostics: Some(sink()),
            output_publisher: Some(publisher.clone()),
            ..PhaseExecutionResources::default()
        },
        PhaseExecutionResources {
            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                DiagnosticPhase::SourceLoad,
                snapshot,
                "wrong.phase",
            ))),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        },
    ] {
        let result = run(resources);
        assert_eq!(result.status, PhaseStatus::Blocking);
        assert!(result.output_refs.is_empty());
    }
    let cancelled = run(PhaseExecutionResources {
        cancellation: Some(CancellationToken {
            snapshot,
            generation: CancellationGeneration::new(1),
            reason: CancellationReason::ExplicitRequest,
        }),
        diagnostics: Some(sink()),
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
    });
    assert_eq!(cancelled.status, PhaseStatus::Cancelled);
    assert!(cancelled.output_refs.is_empty());
    let foreign = run(PhaseExecutionResources {
        cancellation: Some(CancellationToken {
            snapshot: snapshot_id(0xaa),
            generation: CancellationGeneration::new(1),
            reason: CancellationReason::ExplicitRequest,
        }),
        diagnostics: Some(sink()),
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
    });
    assert_eq!(foreign.status, PhaseStatus::Blocking);
    assert!(foreign.output_refs.is_empty());
    let denied = run(PhaseExecutionResources {
        diagnostics: Some(sink()),
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
        ..PhaseExecutionResources::default()
    });
    assert_eq!(denied.status, PhaseStatus::Blocking);
    assert!(denied.output_refs.is_empty());
    publisher.mark_obsolete(snapshot).unwrap();
    let stale = run(PhaseExecutionResources {
        diagnostics: Some(sink()),
        output_publisher: Some(publisher),
        source_load: Some(source_load),
        ..PhaseExecutionResources::default()
    });
    assert_eq!(stale.status, PhaseStatus::Blocking);
    assert!(stale.output_refs.is_empty());
}

#[test]
fn frontend_real_diagnostic_producer_matrix() {
    let cases: &[(&str, &[u16])] = &[
        ("definition\rend;", &[13, 21, 29, 38]),
        ("α\n::=\n", &[14, 15]),
        ("import ;\ndefinition\nend;", &[16, 43]),
        (
            "import std., pkg.math as ;\ndefinition\nend;",
            &[17, 18, 22, 41, 51],
        ),
        ("import std.core\ndefinition\nend;", &[19]),
        ("import @;\ndefinition\nend;", &[20]),
        ("let , x be set;\nfor + y holds thesis;", &[30, 31, 34, 47]),
        ("definition\nlet x,x be set;\nend;", &[32]),
        ("end;\ndefinition\nlet x be set;", &[33, 10, 52]),
        ("\"abc\"", &[35]),
        ("infix_operator(\"bad\\n\",left,80);", &[2, 42]),
        ("theorem T: x = y &;\n", &[48]),
        ("private ;", &[45]),
        ("export 123;", &[44]),
        ("theorem T: thesis by ;", &[49]),
        ("@[ label theorem T: thesis;", &[50]),
        ("theorem T: thesis;\nexport foo;", &[51]),
        ("definition\nlet x be ;\nend;", &[46]),
        (
            "func Plus: +(x,y) -> set; infix_operator(\"+\",left,80); theorem T: a + ;",
            &[39],
        ),
        (
            "func Plus: +(x,y) -> set; infix_operator(\"+\",none,80); theorem T: a + b + c;",
            &[40],
        ),
    ];
    for &(text, expected) in cases {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        let (source, publisher) = execute(&submission, &ids, usize::MAX);
        let result = execute_frontend(
            &submission,
            &ids,
            &publisher,
            vec![source.output_refs[0].clone()],
            (source_key(&fixture.version), Vec::new()),
            Vec::new(),
            None,
        );
        assert!(matches!(
            result.status,
            PhaseStatus::Recoverable | PhaseStatus::Fatal
        ));
        assert!(result.output_refs.is_empty());
        let actual = result
            .diagnostics
            .iter()
            .flat_map(|batch| batch.drafts())
            .map(|draft| draft.code().number())
            .collect::<Vec<_>>();
        for &code in expected {
            assert!(
                actual.contains(&code),
                "{text:?} must emit E{code:04}, got {actual:?}"
            );
        }
    }
}

#[test]
fn source_load_same_text_with_distinct_raw_maps_keeps_semantic_key_and_storage_identity() {
    let first = Fixture::new(b"\xef\xbb\xbfalpha\r\nbeta");
    let second = Fixture::new(b"alpha\nbeta");
    assert_eq!(first.version.source_hash, second.version.source_hash);
    let first_key = source_key(&first.version);
    let second_key = source_key(&second.version);
    assert_eq!(first_key, second_key);
    assert_ne!(first.bytes, second.bytes);
    let first_ids = InMemorySessionIdAllocator::new();
    let first_snapshots = SnapshotRegistry::new();
    let first_submission = first.submit(&first_ids, &first_snapshots);
    let second_ids = InMemorySessionIdAllocator::new();
    let second_snapshots = SnapshotRegistry::new();
    let second_submission = second.submit(&second_ids, &second_snapshots);
    let (first_result, _) = execute(&first_submission, &first_ids, usize::MAX);
    let (second_result, _) = execute(&second_submission, &second_ids, usize::MAX);
    assert_eq!(
        first_result.output_refs[0].content_hash(),
        second_result.output_refs[0].content_hash()
    );
    assert_ne!(
        first_result.output_refs[0].side_table_hash(),
        second_result.output_refs[0].side_table_hash()
    );
}

#[test]
fn source_load_changed_disk_bytes_is_blocking_without_publication() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    fixture.write(b"changed");
    let (result, _) = execute(&submission, &ids, usize::MAX);
    assert_eq!(result.status, PhaseStatus::Blocking);
    assert!(result.output_refs.is_empty());
}

#[test]
fn source_load_real_errors_allocate_shared_source_location_diagnostics() {
    let cases: &[(Option<&[u8]>, u16)] = &[(Some(&[0xff]), 601), (None, 602)];
    for (replacement, code) in cases {
        let fixture = Fixture::new(b"alpha");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = fixture.submit(&ids, &snapshots);
        if let Some(bytes) = replacement {
            fixture.write(bytes);
        } else {
            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        }
        let (result, _) = execute(&submission, &ids, usize::MAX);
        assert_source_error(
            &result,
            &fixture.version,
            *code,
            if *code == 601 {
                "source.invalid_utf8"
            } else {
                "source.unreadable_file"
            },
        );
    }
}

#[cfg(unix)]
#[test]
fn source_load_symlink_escape_is_an_allocated_outside_root_diagnostic() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new(b"alpha");
    let outside = fixture.root.join("outside.miz");
    fs::write(&outside, b"outside").unwrap();
    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
    symlink(&outside, fixture.root.join("alpha/src/main.miz")).unwrap();
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let (result, _) = execute(&submission, &ids, usize::MAX);
    assert_source_error(
        &result,
        &fixture.version,
        603,
        "source.outside_package_root",
    );
}

#[test]
fn source_load_allocator_failure_is_generic_source_diagnostic() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let failing = SourceIdFailureAllocator {
        inner: InMemorySessionIdAllocator::new(),
    };
    let task = source_task(&submission);
    let snapshot = submission.session.captured().snapshot().id;
    let publisher = publisher(snapshot, usize::MAX, &task.unit);
    let sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        DiagnosticPhase::SourceLoad,
        snapshot,
        "frontend.source_load",
    ));
    let result = source_registry()
        .execute_phase_with_resources(
            PipelinePhase::SourceLoad,
            PhaseInput::new(
                task.unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    source_key(&fixture.version),
                    Vec::new(),
                ),
            ),
            PhaseExecutionResources {
                diagnostics: Some(sink),
                output_publisher: Some(publisher),
                source_load: Some(SourceLoadInputs {
                    captured: None,
                    snapshot: submission.session.captured().snapshot(),
                    build_plan: submission.build_plan.as_ref().unwrap(),
                    module_index: submission.module_index.as_ref().unwrap(),
                    allocator: &failing,
                }),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap();
    assert_source_error(
        &result.result,
        &fixture.version,
        600,
        "source.source_id_allocation",
    );
}

#[test]
fn source_load_blocks_missing_or_stale_resources_and_scopes_cancellation() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let task = source_task(&submission);
    let snapshot = submission.session.captured().snapshot().id;
    let source_load = SourceLoadInputs {
        captured: None,
        snapshot: submission.session.captured().snapshot(),
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let sink = || {
        DiagnosticSink::new(DiagnosticProducerScope::new(
            DiagnosticPhase::SourceLoad,
            snapshot,
            "frontend.source_load",
        ))
    };
    let input = || {
        PhaseInput::new(
            task.unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                snapshot,
                source_key(&fixture.version),
                Vec::new(),
            ),
        )
    };
    let run = |resources: PhaseExecutionResources<'_>| {
        source_registry()
            .execute_phase_with_resources(PipelinePhase::SourceLoad, input(), resources)
            .unwrap()
            .result
    };
    let publisher = publisher(snapshot, usize::MAX, &task.unit);
    let missing_sink = run(PhaseExecutionResources {
        output_publisher: Some(publisher.clone()),
        source_load: Some(source_load),
        ..PhaseExecutionResources::default()
    });
    assert_eq!(missing_sink.status, PhaseStatus::Blocking);
    assert!(missing_sink.output_refs.is_empty());
    let denied = std::sync::Arc::new(PhaseOutputPublisher::new(
        std::sync::Arc::new(IrStorageService::new()),
        std::sync::Arc::new(SnapshotHandleRegistry::new()),
    ));
    denied.register_current_snapshot(snapshot);
    denied.allow_work_unit(AllowedWorkUnit::new(
        IrPipelinePhase::new("SourceLoad"),
        OutputKind::new("SourceUnit"),
        IrWorkUnit::new("another-module"),
    ));
    let denied = run(PhaseExecutionResources {
        diagnostics: Some(sink()),
        output_publisher: Some(denied),
        source_load: Some(source_load),
        ..PhaseExecutionResources::default()
    });
    assert_eq!(denied.status, PhaseStatus::Blocking);
    assert!(denied.output_refs.is_empty());
    let mut sealed = sink();
    sealed.seal();
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(sealed),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                DiagnosticPhase::Frontend,
                snapshot,
                "frontend.source_load",
            ))),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                DiagnosticPhase::SourceLoad,
                snapshot_id(0xab),
                "frontend.source_load",
            ))),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(sink()),
            output_publisher: None,
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(sink()),
            output_publisher: Some(publisher.clone()),
            source_load: None,
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
    assert_eq!(
        run(PhaseExecutionResources {
            cancellation: Some(CancellationToken {
                snapshot,
                generation: CancellationGeneration::new(1),
                reason: CancellationReason::ExplicitRequest,
            }),
            diagnostics: Some(sink()),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
        })
        .status,
        PhaseStatus::Cancelled
    );
    let foreign = snapshot_id(0xaa);
    assert_eq!(
        run(PhaseExecutionResources {
            cancellation: Some(CancellationToken {
                snapshot: foreign,
                generation: CancellationGeneration::new(1),
                reason: CancellationReason::ExplicitRequest,
            }),
            diagnostics: Some(sink()),
            output_publisher: Some(publisher.clone()),
            source_load: Some(source_load),
        })
        .status,
        PhaseStatus::Blocking
    );
    publisher.mark_obsolete(snapshot).unwrap();
    assert_eq!(
        run(PhaseExecutionResources {
            diagnostics: Some(sink()),
            output_publisher: Some(publisher),
            source_load: Some(source_load),
            ..PhaseExecutionResources::default()
        })
        .status,
        PhaseStatus::Blocking
    );
}

#[test]
fn source_load_rejects_duplicate_foreign_nondisk_missing_and_summary_bindings() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let base_snapshot = submission.session.captured().snapshot().clone();
    let plan = submission.build_plan.as_ref().unwrap();
    let index = submission.module_index.as_ref().unwrap();
    let module_unit = source_task(&submission).unit.clone();
    let run = |snapshot: &BuildSnapshot, plan: &BuildPlan, index: &ModuleIndex, unit: WorkUnit| {
        binding_result(
            &submission,
            SourceLoadInputs {
                captured: None,
                snapshot,
                build_plan: plan,
                module_index: index,
                allocator: &ids,
            },
            unit,
        )
    };

    let mut duplicate = base_snapshot.clone();
    duplicate.source_versions.push(fixture.version.clone());
    assert_eq!(
        run(&duplicate, plan, index, module_unit.clone()),
        PhaseStatus::Blocking
    );

    let mut foreign = base_snapshot.clone();
    foreign.source_versions[0].package_id = PackageId::new("foreign");
    assert_eq!(
        run(&foreign, plan, index, module_unit.clone()),
        PhaseStatus::Blocking
    );

    let mut nondisk = base_snapshot.clone();
    nondisk.source_versions[0].origin = SourceOrigin::OpenBuffer { version: 1 };
    assert_eq!(
        run(&nondisk, plan, index, module_unit.clone()),
        PhaseStatus::Blocking
    );

    let mut missing = base_snapshot.clone();
    missing.source_versions.clear();
    assert_eq!(
        run(&missing, plan, index, module_unit.clone()),
        PhaseStatus::Blocking
    );

    let mut summary = index.clone();
    summary.modules[0].location = ModuleIndexLocation::DependencySummary {
        artifact: "alpha-summary".to_owned(),
        content_hash: hash(0x91),
    };
    assert_eq!(
        run(&base_snapshot, plan, &summary, module_unit),
        PhaseStatus::Blocking
    );

    let mut mismatched_version = base_snapshot.clone();
    mismatched_version.source_versions[0].edition = Edition::new("2030");
    assert_eq!(
        run(
            &mismatched_version,
            plan,
            index,
            source_task(&submission).unit.clone()
        ),
        PhaseStatus::Blocking
    );

    let mut mismatched_index = index.clone();
    mismatched_index.modules[0].edition = Edition::new("2030");
    assert_eq!(
        run(
            &base_snapshot,
            plan,
            &mismatched_index,
            source_task(&submission).unit.clone()
        ),
        PhaseStatus::Blocking
    );
}

#[test]
fn source_load_rejects_mismatched_plan_and_disallowed_work_unit() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let snapshot = submission.session.captured().snapshot().clone();
    let index = submission.module_index.as_ref().unwrap();
    let mut plan = submission.build_plan.as_ref().unwrap().clone();
    plan.workspace_root = WorkspaceRoot::new("other-workspace");
    let module_unit = source_task(&submission).unit.clone();
    assert_eq!(
        binding_result(
            &submission,
            SourceLoadInputs {
                captured: None,
                snapshot: &snapshot,
                build_plan: &plan,
                module_index: index,
                allocator: &ids,
            },
            module_unit.clone(),
        ),
        PhaseStatus::Blocking
    );
    assert_eq!(
        binding_result(
            &submission,
            SourceLoadInputs {
                captured: None,
                snapshot: &snapshot,
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: index,
                allocator: &ids,
            },
            WorkUnit::Package {
                package_id: PackageId::new("alpha"),
            },
        ),
        PhaseStatus::Blocking
    );
}

#[test]
fn source_load_requires_owner_dispatch_hash_and_no_parents_or_dependencies() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let task = source_task(&submission);
    let snapshot = submission.session.captured().snapshot().id;
    let source_load = SourceLoadInputs {
        captured: None,
        snapshot: submission.session.captured().snapshot(),
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let (parent, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(parent.status, PhaseStatus::Complete);
    let parent = mizar_ir::dispatch_input::SealedParentOutputHandle::from_current_output(
        &publisher,
        snapshot,
        parent.output_refs[0].clone(),
    )
    .unwrap();
    let registry = source_registry();
    for (input_hash, dependencies, parents) in [
        (hash(7), Vec::new(), Vec::new()),
        (source_key(&fixture.version), vec![hash(8)], Vec::new()),
        (source_key(&fixture.version), Vec::new(), vec![parent]),
    ] {
        let input = PhaseInput::new(
            task.unit.clone(),
            PhaseDispatchInputBundle::new(snapshot, input_hash, dependencies, parents).unwrap(),
        );
        let cache = registry
            .cache_key_for_phase(PipelinePhase::SourceLoad, &input)
            .unwrap();
        assert!(matches!(cache.intent, PhaseCacheIntent::NoKey { .. }));
        let result = registry
            .execute_phase_with_resources(
                PipelinePhase::SourceLoad,
                input,
                PhaseExecutionResources {
                    diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                        DiagnosticPhase::SourceLoad,
                        snapshot,
                        "frontend.source_load",
                    ))),
                    output_publisher: Some(publisher.clone()),
                    source_load: Some(source_load),
                    ..PhaseExecutionResources::default()
                },
            )
            .unwrap()
            .result;
        assert_eq!(result.status, PhaseStatus::Blocking);
        assert!(result.output_refs.is_empty());
    }
}

#[test]
fn ordinary_submission_stays_blocked_when_only_source_load_is_registered() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    assert!(!submission.missing_services.is_empty());
    assert!(
        submission
            .task_graph
            .unwrap()
            .tasks()
            .iter()
            .any(|task| task.kind == TaskKind::SourceLoad)
    );
}

#[test]
fn dependency_lexical_provider_reads_real_source_and_summary_boundaries() {
    use mizar_artifact::{
        module_summary::{
            ExportedSymbolSummary, LexicalContributionSummary, ModuleLexicalSummary, ModuleSummary,
            ModuleSummaryIdentity, SourceRangeSummary, current_schema_version, module_summary_json,
        },
        store::{
            PublishedArtifactPath, artifact_hash_domain, canonical_json_string,
            write_published_artifact,
        },
    };
    use mizar_frontend::{
        lexical_env::{
            ExportRank, ExportedSymbolShape, ModuleId as LexerModuleId, SymbolId, UserSymbolArity,
            UserSymbolKind,
        },
        lexing::TokenKind,
        orchestration::{DiagnosticCode, DiagnosticLocation, Frontend, FrontendOutput},
        parsing::MizarParserSeam,
        source::FrontendSourceLoader,
    };
    use std::sync::Arc;

    let fixture = Fixture::new(
        b"\xef\xbb\xbfimport dep.core as A, dep.core as B; import dep.{core, absent};\r\ndefinition\r\nend;\r\ntheorem Combined: combine(x,y) = x;\r\n",
    );
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    let (result, publisher) = execute(&submission, &ids, usize::MAX);
    assert_eq!(result.status, PhaseStatus::Complete);
    let output = &result.output_refs[0];
    let typed = publisher
        .storage()
        .typed_handle::<SourceUnit>(output, &OutputKind::new("SourceUnit"))
        .unwrap();
    let source = publisher.storage().get(&typed).unwrap().clone();
    let mut bridge = SpanBridge::new();
    register_source_unit(&mut bridge, &source).unwrap();
    let preprocessed = preprocess(&source, &mut bridge).unwrap();
    assert_eq!(preprocessed.import_stubs.len(), 4);

    let workspace_root = WorkspaceRoot::new(fixture.root.to_string_lossy().into_owned());
    let plan = produce_build_plan(
        PlanRequest {
            workspace_root: workspace_root.clone(),
            dependency_selection: DependencySelection::Normal,
            toolchain: ToolchainInfo::new("mizar-evo-test"),
        },
        vec![WorkspacePackage {
            member_path: "alpha".to_owned(),
            manifest: parse_package_manifest(
                "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[dependencies]\ndep = \"1.0.0\"\n",
            )
            .unwrap(),
        }],
        parse_lockfile(
            "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = [{ name = \"dep\", version = \"1.0.0\" }]\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:dep\" }\ndependencies = []\n",
        )
        .unwrap(),
    )
    .unwrap();

    let artifact_root = fixture.root.join("dep-artifacts");
    fs::create_dir_all(&artifact_root).unwrap();
    let artifact = "dep/core.summary.json";
    let module = ModuleSummaryIdentity {
        package_id: "dep".to_owned(),
        package_version: Some("1.0.0".to_owned()),
        lockfile_identity: Some("dependency-lock".to_owned()),
        module_path: "core".to_owned(),
        language_edition: "2025".to_owned(),
    };
    let identity = canonical_json_string(&module.canonical_json().unwrap());
    let origin_id = "symbol:combine";
    let shape = ExportedSymbolShape {
        spelling: "combine".to_owned(),
        symbol_id: SymbolId::new(canonical_json_string(
            &mizar_artifact::store::CanonicalJson::array([
                module.canonical_json().unwrap(),
                mizar_artifact::store::CanonicalJson::string(origin_id),
            ]),
        )),
        source_module: LexerModuleId::new(identity),
        export_rank: ExportRank::new(0),
        kind: UserSymbolKind::Functor,
        arity: UserSymbolArity::exact(2),
        operator: None,
    };
    let payload = String::from_utf8(shape.canonical_bytes().unwrap()).unwrap();
    // Published dependency fixture only, not a substitute for a source export producer.
    let mut summary = ModuleSummary {
        schema_version: current_schema_version(),
        module: module.clone(),
        source_hash: hash(9),
        interface_hash: hash(0),
        exported_symbols: vec![ExportedSymbolSummary {
            origin_id: origin_id.to_owned(),
            fully_qualified_name: "core.combine".to_owned(),
            namespace_path: vec!["core".to_owned()],
            visibility: "public".to_owned(),
            declaration_kind: "functor".to_owned(),
            source_range: SourceRangeSummary {
                start_byte: 0,
                end_byte: 1,
            },
            rendered_signature: "functor core.combine".to_owned(),
            interface_fingerprint: hash(8),
            proof_status: None,
        }],
        exported_labels: Vec::new(),
        lexical_summary: ModuleLexicalSummary {
            schema_version: "mizar-resolve/exported-lexical/v1".to_owned(),
            fingerprint: None,
            contributions: vec![LexicalContributionSummary {
                kind: "exported-symbol".to_owned(),
                key: origin_id.to_owned(),
                payload,
            }],
        },
        reexports: Vec::new(),
        dependency_interfaces: Vec::new(),
    };
    summary.refresh_interface_hash().unwrap();
    let publish = |value: &ModuleSummary| {
        let json = module_summary_json(value).unwrap();
        let domain = artifact_hash_domain(
            mizar_artifact::module_summary::MODULE_SUMMARY_SCHEMA_FAMILY,
            value.schema_version,
        );
        write_published_artifact(
            &artifact_root,
            &PublishedArtifactPath::new(artifact).unwrap(),
            &json,
            &domain,
            &[],
        )
        .unwrap()
        .artifact_hash
    };
    let content_hash = publish(&summary);
    let dependency = DependencyArtifactIndex::new(
        PackageId::new("dep"),
        Vec::new(),
        vec![DependencyModuleSummaryRef {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("dep"),
                ModulePath::new("core"),
            ),
            artifact: artifact.to_owned(),
            content_hash,
        }],
    );
    let module_index = build_module_index(
        &plan,
        &StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
        }]),
        &[dependency],
    )
    .unwrap();
    let request = LexicalEnvironmentRequest {
        source_id: source.source_id,
        import_stubs: &preprocessed.import_stubs,
        edition: source.edition.clone(),
    };
    let roots = vec![(PackageId::new("dep"), artifact_root.clone())];
    let inputs = SourceLoadInputs {
        captured: None,
        snapshot: submission.session.captured().snapshot(),
        build_plan: &plan,
        module_index: &module_index,
        allocator: &ids,
    };
    let provider = inputs.dependency_lexical_provider(&roots);
    let resolved = provider.resolve_imports(&request).unwrap();
    assert_eq!(resolved.imports.len(), 3);
    assert!(!resolved.summaries.is_empty());
    assert!(
        resolved
            .summaries
            .iter()
            .all(|summary| summary.module_id == shape.source_module)
    );
    assert!(
        resolved
            .imports
            .iter()
            .all(|entry| entry.import.module_id == resolved.summaries[0].module_id)
    );
    assert_eq!(
        resolved
            .imports
            .iter()
            .map(|entry| entry.stub_ordinal)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    for entry in &resolved.imports {
        assert_eq!(
            entry.stub_span,
            preprocessed.import_stubs[entry.stub_ordinal].span
        );
    }
    let active = build_active_lexical_environment(&request, &provider).unwrap();
    assert!(active.environment.user_symbol("combine").is_some());
    assert_eq!(active.environment.visible_user_symbols().len(), 1);
    assert!(active.diagnostics.iter().any(|diagnostic| {
        diagnostic.code
            == mizar_frontend::lexical_env::LexicalEnvironmentDiagnosticCode::UnresolvedImport
    }));
    let repeat = inputs.dependency_lexical_provider(&roots);
    assert_eq!(resolved, repeat.resolve_imports(&request).unwrap());
    let repeated = build_active_lexical_environment(&request, &repeat).unwrap();
    assert_eq!(active.fingerprint, repeated.fingerprint);

    let frontend_output = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
        inputs.dependency_lexical_provider(&roots),
        MizarParserSeam,
    )
    .run_loaded(source.as_ref().clone())
    .unwrap();
    assert_eq!(frontend_output.source, *source);
    assert_eq!(frontend_output.preprocessed, preprocessed);
    assert_eq!(
        frontend_output
            .cache_keys
            .active_lexical_environment
            .fingerprint,
        active.fingerprint
    );
    let combine = frontend_output
        .tokens
        .tokens
        .iter()
        .find(|token| token.text.as_ref() == "combine")
        .unwrap();
    assert_eq!(combine.kind, TokenKind::UserSymbol);
    let ast = frontend_output.ast.as_ref().unwrap();
    assert!(ast.node_views().any(|view| {
        view.as_theorem_item().is_some()
            && view.range().start <= combine.span.start
            && combine.span.end <= view.range().end
            && !view.is_recovered()
    }));
    assert!(ast.node_views().any(|view| {
        view.as_application_term().is_some()
            && view.range().start <= combine.span.start
            && combine.span.end <= view.range().end
            && !view.is_recovered()
    }));
    assert!(ast.node_views().all(|view| !view.is_recovered()));
    assert_eq!(frontend_output.diagnostics.len(), 1);
    assert_eq!(
        frontend_output.diagnostics[0].class,
        mizar_frontend::orchestration::DiagnosticClass::LexicalEnvironment
    );
    assert_eq!(
        frontend_output.diagnostics[0].code,
        DiagnosticCode::LexicalEnvironment(
            mizar_frontend::lexical_env::LexicalEnvironmentDiagnosticCode::UnresolvedImport
        )
    );
    let DiagnosticLocation::SourceRange(primary) = &frontend_output.diagnostics[0].location else {
        panic!("expected unresolved import source range");
    };
    let input = SourceInput {
        package_id: source.package_id.clone(),
        module_path: source.module_path.clone(),
        normalized_path: source.normalized_path.clone(),
        edition: source.edition.clone(),
        origin: SourceOriginInput::Disk {
            path: source.file_path.clone(),
        },
    };
    let bytes = frontend_output.canonical_disk_bytes().unwrap();
    assert_eq!(
        FrontendOutput::from_canonical_disk_bytes(&bytes, source.source_id, &input).unwrap(),
        frontend_output
    );
    let fresh_id = ids.next_source_id(inputs.snapshot.id).unwrap();
    assert_ne!(fresh_id, source.source_id);
    let rebound = FrontendOutput::from_canonical_disk_bytes(&bytes, fresh_id, &input).unwrap();
    assert_eq!(rebound.source.source_id, fresh_id);
    assert_eq!(rebound.preprocessed.source_id, fresh_id);
    assert_eq!(rebound.tokens.source_id, fresh_id);
    assert_eq!(rebound.ast.as_ref().unwrap().source_id, fresh_id);
    assert_eq!(
        rebound.diagnostics[0].location,
        DiagnosticLocation::SourceRange(mizar_session::SourceRange {
            source_id: fresh_id,
            ..*primary
        })
    );
    assert_eq!(rebound.canonical_disk_bytes().unwrap(), bytes);

    for corrupt in [false, true] {
        if corrupt {
            fs::write(artifact_root.join(artifact), b"not canonical json").unwrap();
        } else {
            fs::remove_file(artifact_root.join(artifact)).unwrap();
        }
        let unavailable = inputs.dependency_lexical_provider(&roots);
        let unavailable_result = unavailable.resolve_imports(&request).unwrap();
        assert_eq!(unavailable_result.imports.len(), 3);
        assert!(unavailable_result.summaries.is_empty());
        let environment = build_active_lexical_environment(&request, &unavailable).unwrap();
        assert!(environment.environment.user_symbol("combine").is_none());
        assert!(environment.diagnostics.iter().any(|diagnostic| {
            diagnostic.code
                == mizar_frontend::lexical_env::LexicalEnvironmentDiagnosticCode::MissingSummary
        }));
        publish(&summary);
    }

    let mut malformed = summary.clone();
    malformed.lexical_summary.contributions[0].payload.push('x');
    malformed.refresh_interface_hash().unwrap();
    let malformed_hash = publish(&malformed);
    let mut malformed_index = module_index.clone();
    for reference in &mut malformed_index.dependency_summaries {
        if reference.module.package.as_str() == "dep" {
            reference.content_hash = malformed_hash;
        }
    }
    for entry in &mut malformed_index.modules {
        if entry.module.package.as_str() == "dep"
            && let ModuleIndexLocation::DependencySummary { content_hash, .. } = &mut entry.location
        {
            *content_hash = malformed_hash;
        }
    }
    let malformed_inputs = SourceLoadInputs {
        captured: None,
        module_index: &malformed_index,
        ..inputs
    };
    assert!(matches!(
        malformed_inputs
            .dependency_lexical_provider(&roots)
            .resolve_imports(&request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));
    publish(&summary);

    for roots in [
        Vec::new(),
        vec![
            (PackageId::new("dep"), artifact_root.clone()),
            (PackageId::new("dep"), artifact_root.clone()),
        ],
        vec![(PackageId::new("other"), artifact_root.clone())],
    ] {
        assert!(matches!(
            inputs
                .dependency_lexical_provider(&roots)
                .resolve_imports(&request),
            Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
        ));
    }

    for invalid_request in [
        LexicalEnvironmentRequest {
            edition: Edition::new("2026"),
            ..request.clone()
        },
        LexicalEnvironmentRequest {
            source_id: ids.next_source_id(inputs.snapshot.id).unwrap(),
            ..request.clone()
        },
    ] {
        assert!(matches!(
            inputs
                .dependency_lexical_provider(&roots)
                .resolve_imports(&invalid_request),
            Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
        ));
    }
    let mut mismatched_index = module_index.clone();
    mismatched_index.dependency_summaries[0].content_hash = hash(71);
    assert!(matches!(
        SourceLoadInputs {
            captured: None,
            module_index: &mismatched_index,
            ..inputs
        }
        .dependency_lexical_provider(&roots)
        .resolve_imports(&request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));
    let mut duplicate_snapshot = submission.session.captured().snapshot().clone();
    duplicate_snapshot
        .source_versions
        .push(fixture.version.clone());
    let duplicate_inputs = SourceLoadInputs {
        captured: None,
        snapshot: &duplicate_snapshot,
        ..inputs
    };
    assert!(matches!(
        duplicate_inputs
            .dependency_lexical_provider(&roots)
            .resolve_imports(&request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));
    let mut bad_index = module_index.clone();
    bad_index
        .modules
        .iter_mut()
        .find(|entry| entry.module.package.as_str() == "alpha")
        .unwrap()
        .edition = Edition::new("2026");
    let bad_index_inputs = SourceLoadInputs {
        captured: None,
        module_index: &bad_index,
        ..inputs
    };
    assert!(matches!(
        bad_index_inputs
            .dependency_lexical_provider(&roots)
            .resolve_imports(&request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));

    let mut source_backed = preprocessed.clone();
    source_backed.import_stubs[0].path.relative =
        Some(mizar_frontend::preprocess::ImportStubRelativePrefix::Current);
    source_backed.import_stubs[0].path.spelling = Arc::from(".main");
    source_backed.import_stubs[0].path.components = vec![Arc::from("main")];
    let source_backed_span = source_backed.import_stubs[0].path.span;
    source_backed.import_stubs[0].path.source_segments = vec![source_backed_span];
    let source_backed_request = LexicalEnvironmentRequest {
        import_stubs: &source_backed.import_stubs,
        ..request
    };
    assert!(matches!(
        inputs
            .dependency_lexical_provider(&roots)
            .resolve_imports(&source_backed_request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));
}

#[test]
fn prepared_native_prefix_uses_frozen_disk_and_open_sources_with_real_storage_and_no_keys() {
    use mizar_frontend::{orchestration::Frontend, source::FrontendSourceLoader};
    use mizar_ir::dispatch_input::SealedParentOutputHandle;
    let raw = "\u{feff}definition\r\nend;\r\n";
    for open in [false, true] {
        for (threshold, workers, order) in [
            (usize::MAX, 1, CompletionOrder::Canonical),
            (0, 4, CompletionOrder::Reverse),
        ] {
            let fixture = Fixture::new(raw.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
            request.source_inputs.versions.clear();
            input.worker_count = workers;
            input.completion_order = order;
            let descriptor = SourceInput {
                package_id: fixture.version.package_id.clone(),
                module_path: fixture.version.module_path.clone(),
                normalized_path: fixture.version.normalized_path.clone(),
                edition: fixture.version.edition.clone(),
                origin: if open {
                    SourceOriginInput::OpenBuffer {
                        uri: format!(
                            "file://{}",
                            fixture.root.join("alpha/src/main.miz").display()
                        ),
                        expected_version: 7,
                        actual_version: 7,
                        text: std::sync::Arc::from(raw),
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
            let session = request
                .capture_sources(
                    &ids,
                    &snapshots,
                    vec![(
                        DiskSourceLoader::new(fixture.root.join("alpha")),
                        descriptor.clone(),
                    )],
                )
                .unwrap();
            let snapshot = session.captured().snapshot().id;
            let captured_id = session.captured().snapshot().source_versions[0].source_id;
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new("main"),
                ),
            };
            let publisher = publisher(snapshot, threshold, &unit);
            allow_frontend(&publisher, &unit);
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver = CompilerDriver::new(builder.build().unwrap())
                .with_output_publisher(publisher.clone());
            fixture.write(b"changed disk content");
            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
            let submission = driver
                .submit_captured(session, &ids, &snapshots, input)
                .unwrap();
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Blocked)
            );
            let run = submission.scheduler_run.as_ref().unwrap();
            let handles = run
                .phase_results
                .values()
                .flatten()
                .flat_map(|result| &result.output_refs)
                .collect::<Vec<_>>();
            assert_eq!(handles.len(), 2);
            let source_ref = *handles
                .iter()
                .find(|handle| handle.output_kind() == &OutputKind::new("SourceUnit"))
                .unwrap();
            let source_handle = publisher
                .storage()
                .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                .unwrap();
            let source = publisher.storage().get(&source_handle).unwrap();
            assert_eq!(source.source_id, captured_id);
            assert_eq!(source.source_text.as_ref(), "definition\nend;\n");
            assert_eq!(
                source.source_hash,
                mizar_session::hash_text("definition\nend;\n")
            );
            assert!(source.loading_map.is_some());
            assert_eq!(
                source.origin,
                if open {
                    SourceOrigin::OpenBuffer { version: 7 }
                } else {
                    SourceOrigin::Disk
                }
            );
            let output_ref = *handles
                .iter()
                .find(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput"))
                .unwrap();
            let output_handle = publisher
                .storage()
                .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                    output_ref,
                    &OutputKind::new("FrontendOutput"),
                )
                .unwrap();
            let output = publisher.storage().get(&output_handle).unwrap();
            let inputs = SourceLoadInputs {
                snapshot: submission.session.captured().snapshot(),
                captured: Some(submission.session.captured()),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            };
            let expected = Frontend::new(
                FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha"))),
                inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded(source.as_ref().clone())
            .unwrap();
            assert_eq!(*output, expected);
            assert_eq!(output.source, *source);
            assert!(output.diagnostics.is_empty());
            assert_eq!(output_ref.lineage().parents, vec![source_ref.output()]);
            for handle in handles {
                assert_eq!(
                    matches!(handle.placement(), StoragePlacement::Blob { .. }),
                    threshold == 0
                );
                assert!(
                    publisher
                        .validate_published_output(snapshot, handle)
                        .is_ok()
                );
                assert_eq!(
                    publisher.validate_current_output(snapshot, handle).is_ok(),
                    !open
                );
            }
            let source_input = PhaseInput::new(
                unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    SourceUnitCacheKey::from_source(&source).stable_hash(),
                    vec![],
                ),
            );
            for source_summary in [false, true] {
                let mut forged = inputs.snapshot.clone();
                if source_summary {
                    forged.source_versions[0].source_hash = hash(91);
                } else {
                    forged.verifier_config_hash = hash(92);
                }
                let rejected = driver
                    .registry()
                    .execute_phase_with_resources(
                        PipelinePhase::SourceLoad,
                        source_input.clone(),
                        PhaseExecutionResources {
                            diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                                DiagnosticPhase::SourceLoad,
                                snapshot,
                                "frontend.source_load",
                            ))),
                            output_publisher: Some(publisher.clone()),
                            source_load: Some(SourceLoadInputs {
                                snapshot: &forged,
                                ..inputs
                            }),
                            ..PhaseExecutionResources::default()
                        },
                    )
                    .unwrap()
                    .result;
                assert_eq!(rejected.status, PhaseStatus::Blocking);
                assert!(rejected.output_refs.is_empty());
                assert!(rejected.diagnostics.is_empty());
            }
            let frontend_input = PhaseInput::new(
                unit,
                PhaseDispatchInputBundle::new(
                    snapshot,
                    SourceUnitCacheKey::from_source(&source).stable_hash(),
                    vec![],
                    vec![
                        SealedParentOutputHandle::from_published_output(
                            &publisher,
                            snapshot,
                            source_ref.clone(),
                        )
                        .unwrap(),
                    ],
                )
                .unwrap(),
            );
            assert_eq!(
                driver
                    .registry()
                    .cache_key_for_phase(PipelinePhase::SourceLoad, &source_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey {
                    reason: "current disk source maps require a real source load".into()
                }
            );
            assert_eq!(
                driver
                    .registry()
                    .cache_key_for_phase(PipelinePhase::Frontend, &frontend_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey {
                    reason: "frontend inputs require a live source and dependency run".into()
                }
            );
        }
    }
}

#[test]
fn native_import_discovery_replays_raw_versions_with_fresh_owners_and_rejects_conflicts() {
    use mizar_ir::dispatch_input::SealedParentOutputHandle;
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let leaf_text =
        "definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
    let main_text = "import alpha.leaf;\ntheorem T: a combine b = a;\n";
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(vec![]);
    let mut driver = CompilerDriver::new(builder.build().unwrap());
    let mut canonical = None;
    let mut previous: Option<(std::sync::Arc<PhaseOutputPublisher>, AnyPhaseOutputRef)> = None;
    let mut oldest = None;
    for stage in 0..3 {
        fixture.write(main_text.as_bytes());
        fs::write(fixture.root.join("alpha/src/leaf.miz"), leaf_text).unwrap();
        let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
        request.source_inputs.versions.clear();
        request.generation = BuildRequestGeneration::new(stage);
        input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![
                WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
            ],
        }]);
        input.dependency_overlay = ModuleDependencyOverlay::unavailable();
        input.worker_count = if stage == 0 { 1 } else { 4 };
        input.completion_order = if stage == 1 {
            CompletionOrder::Reverse
        } else {
            CompletionOrder::Canonical
        };
        let mut descriptors = Vec::new();
        for (module, raw) in [("main", main_text), ("leaf", leaf_text)] {
            let path = format!("src/{module}.miz");
            let normalized_path =
                normalize_path(&fixture.root.join("alpha"), Path::new(&path)).unwrap();
            let origin = if stage == 0 {
                SourceOriginInput::Disk {
                    path: path.clone().into(),
                }
            } else {
                SourceOriginInput::OpenBuffer {
                    uri: format!(
                        "file://{}",
                        fixture.root.join("alpha").join(&path).display()
                    ),
                    expected_version: stage as i64 + 7,
                    actual_version: stage as i64 + 7,
                    text: std::sync::Arc::from(if stage == 0 {
                        raw.to_owned()
                    } else {
                        format!("\u{feff}{}", raw.replace('\n', "\r\n"))
                    }),
                }
            };
            descriptors.push((
                DiskSourceLoader::new(fixture.root.join("alpha")),
                SourceInput {
                    package_id: PackageId::new("alpha"),
                    module_path: ModulePath::new(module),
                    normalized_path,
                    edition: Edition::new("2025"),
                    origin,
                },
            ));
        }
        if stage == 1 {
            descriptors.reverse();
        }
        let session = request
            .clone()
            .capture_sources(&ids, &snapshots, descriptors.clone())
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        assert_eq!(*canonical.get_or_insert(snapshot), snapshot);
        if stage == 0 {
            oldest = Some(
                request
                    .clone()
                    .capture_sources(&ids, &snapshots, descriptors.clone())
                    .unwrap(),
            );
        }
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let publisher = publisher(snapshot, 0, &unit);
        for module in ["main", "leaf"] {
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new(module),
                ),
            };
            let ir_unit = IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module));
            publisher.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SourceLoad"),
                OutputKind::new("SourceUnit"),
                ir_unit,
            ));
            allow_frontend(&publisher, &unit);
        }
        if stage == 0 {
            // Real acquisition keeps canonical metadata/IDs while changing raw Disk provenance.
            fs::write(
                fixture.root.join("alpha/src/leaf.miz"),
                format!("\u{feff}{}", leaf_text.replace('\n', "\r\n")),
            )
            .unwrap();
            let producer_ids = InMemorySessionIdAllocator::new();
            let producer_snapshots = SnapshotRegistry::new();
            let producer_session = request
                .capture_sources(&producer_ids, &producer_snapshots, descriptors)
                .unwrap();
            assert_eq!(
                producer_session.captured().snapshot(),
                session.captured().snapshot()
            );
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut producer = CompilerDriver::new(builder.build().unwrap());
            let mut producer_input = DriverSubmitInput::new(
                input.plan_request.clone(),
                input.workspace_packages.clone(),
                input.lockfile.clone(),
                input.source_layout.clone(),
            );
            producer_input.dependency_overlay = ModuleDependencyOverlay::unavailable();
            let produced = producer
                .submit_captured_with_import_discovery(
                    producer_session.clone(),
                    &producer_ids,
                    &producer_snapshots,
                    producer_input,
                )
                .unwrap();
            for module in ["leaf", "main"] {
                // Each owner starts without the tested Frontend slot, so duplicate storage
                // cannot mask the whole-source equality rejection.
                let owner = crate::publisher(snapshot, 0, &unit);
                for name in ["main", "leaf"] {
                    let unit = WorkUnit::Module {
                        module: mizar_build::module_index::ModuleId::new(
                            PackageId::new("alpha"),
                            ModulePath::new(name),
                        ),
                    };
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        IrPipelinePhase::new("SourceLoad"),
                        OutputKind::new("SourceUnit"),
                        IrWorkUnit::new(format!("{:?}:{:?}", "alpha", name)),
                    ));
                    allow_frontend(&owner, &unit);
                }
                let run =
                    |phase,
                     name: &str,
                     parents: Vec<AnyPhaseOutputRef>,
                     capture: &mizar_driver::request::CapturedSnapshot| {
                        let version = capture
                            .snapshot()
                            .source_versions
                            .iter()
                            .find(|version| version.module_path.as_str() == name)
                            .unwrap();
                        let parents = parents
                            .into_iter()
                            .map(|handle| {
                                SealedParentOutputHandle::from_published_output(
                                    &owner, snapshot, handle,
                                )
                                .unwrap()
                            })
                            .collect();
                        producer
                            .registry()
                            .execute_phase_with_resources(
                                phase,
                                PhaseInput::new(
                                    WorkUnit::Module {
                                        module: mizar_build::module_index::ModuleId::new(
                                            PackageId::new("alpha"),
                                            ModulePath::new(name),
                                        ),
                                    },
                                    PhaseDispatchInputBundle::new(
                                        snapshot,
                                        source_key(version),
                                        vec![],
                                        parents,
                                    )
                                    .unwrap(),
                                ),
                                PhaseExecutionResources {
                                    diagnostics: Some(DiagnosticSink::new(
                                        DiagnosticProducerScope::new(
                                            if phase == PipelinePhase::SourceLoad {
                                                DiagnosticPhase::SourceLoad
                                            } else {
                                                DiagnosticPhase::Frontend
                                            },
                                            snapshot,
                                            "native.parent_binding",
                                        ),
                                    )),
                                    output_publisher: Some(owner.clone()),
                                    source_load: Some(SourceLoadInputs {
                                        snapshot: capture.snapshot(),
                                        captured: Some(capture),
                                        build_plan: produced.build_plan.as_ref().unwrap(),
                                        module_index: produced.module_index.as_ref().unwrap(),
                                        allocator: &producer_ids,
                                    }),
                                    ..PhaseExecutionResources::default()
                                },
                            )
                            .unwrap()
                            .result
                    };
                let leaf = run(
                    PipelinePhase::SourceLoad,
                    "leaf",
                    vec![],
                    producer_session.captured(),
                );
                assert_eq!(leaf.status, PhaseStatus::Complete);
                let leaf_source = leaf.output_refs[0].clone();
                let typed = owner
                    .storage()
                    .typed_handle::<SourceUnit>(&leaf_source, &OutputKind::new("SourceUnit"))
                    .unwrap();
                let actual = owner.storage().get(&typed).unwrap();
                assert!(actual.loading_map.is_some());
                assert_eq!(actual.source_text.as_ref(), leaf_text);
                assert_eq!(actual.origin, SourceOrigin::Disk);
                assert!(actual.canonical_disk_bytes().is_some());
                let parents = if module == "main" {
                    let root = run(
                        PipelinePhase::SourceLoad,
                        "main",
                        vec![],
                        producer_session.captured(),
                    );
                    assert_eq!(root.status, PhaseStatus::Complete);
                    let leaf = run(
                        PipelinePhase::Frontend,
                        "leaf",
                        vec![leaf_source],
                        producer_session.captured(),
                    );
                    assert_eq!(leaf.status, PhaseStatus::Complete);
                    vec![root.output_refs[0].clone(), leaf.output_refs[0].clone()]
                } else {
                    vec![leaf_source]
                };
                let rejected = run(PipelinePhase::Frontend, module, parents, session.captured());
                assert_eq!(
                    rejected.status,
                    PhaseStatus::Blocking,
                    "{module} raw-map binding"
                );
                assert!(rejected.output_refs.is_empty());
                assert!(rejected.diagnostics.is_empty());
            }
        }
        if let Some((owner, old)) = &previous {
            assert!(
                SealedParentOutputHandle::from_published_output(&publisher, snapshot, old.clone())
                    .is_err()
            );
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut conflicting =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            let mut conflicting_input = DriverSubmitInput::new(
                input.plan_request.clone(),
                input.workspace_packages.clone(),
                input.lockfile.clone(),
                input.source_layout.clone(),
            );
            conflicting_input.dependency_overlay = ModuleDependencyOverlay::unavailable();
            let rejected = conflicting
                .submit_captured_with_import_discovery(
                    session.clone(),
                    &ids,
                    &snapshots,
                    conflicting_input,
                )
                .unwrap();
            assert!(
                rejected
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .all(|result| result.output_refs.is_empty())
            );
            assert!(
                rejected
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .values()
                    .flatten()
                    .any(|result| result.status == PhaseStatus::Blocking)
            );
        }
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        fs::remove_file(fixture.root.join("alpha/src/leaf.miz")).unwrap();
        driver = driver.with_output_publisher(publisher.clone());
        let submission = driver
            .submit_captured_with_import_discovery(session, &ids, &snapshots, input)
            .unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        let run = submission.scheduler_run.as_ref().unwrap();
        let handles = run
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
            .collect::<Vec<_>>();
        assert_eq!(handles.len(), 4);
        let main = *handles
            .iter()
            .find(|handle| {
                handle.output_kind() == &OutputKind::new("FrontendOutput")
                    && handle.work_unit() == &IrWorkUnit::new(format!("{:?}:{:?}", "alpha", "main"))
            })
            .unwrap();
        let typed = publisher
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                main,
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        let output = publisher.storage().get(&typed).unwrap();
        assert!(output.diagnostics.is_empty());
        assert!(
            output.tokens.tokens().iter().any(
                |token| token.text.as_ref() == "combine" && token.kind == TokenKind::UserSymbol
            )
        );
        assert_eq!(
            output.source.origin,
            if stage == 0 {
                SourceOrigin::Disk
            } else {
                SourceOrigin::OpenBuffer {
                    version: stage as i64 + 7,
                }
            }
        );
        assert_eq!(output.source.source_text.as_ref(), main_text);
        assert_eq!(main.lineage().parents.len(), 2);
        assert!(matches!(main.placement(), StoragePlacement::Blob { .. }));
        previous = Some((publisher, main.clone()));
    }
    let (_, input) = fixture.submit_request(vec![], vec![]);
    let obsolete = driver
        .submit_captured(oldest.unwrap(), &ids, &snapshots, input)
        .unwrap();
    assert_eq!(
        obsolete.status,
        DriverSubmissionStatus::SupersededBeforeSubmission
    );
    assert_eq!(
        obsolete.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Superseded)
    );
    assert!(obsolete.scheduler_run.is_none());
    assert!(matches!(
        obsolete.publication_decision,
        mizar_driver::request::PublicationDecision::Suppressed(_)
    ));
}

#[test]
fn native_scheduled_diagnostics_cancellation_generated_and_denied_publication_remain_real() {
    use mizar_frontend::{
        orchestration::{DiagnosticCode, DiagnosticLocation, Frontend},
        source::{FrontendSourceLoader, SourceUnitLoader, SourceUnitRequest},
    };
    for mode in ["diagnostic", "cancel", "generated", "denied"] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
        request.source_inputs.versions.clear();
        let descriptor = SourceInput {
            package_id: fixture.version.package_id.clone(),
            module_path: fixture.version.module_path.clone(),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: fixture.version.edition.clone(),
            origin: if mode == "generated" {
                SourceOriginInput::Generated {
                    generator: mizar_session::GeneratedSourceKind::new("native-test"),
                    text: std::sync::Arc::from("definition\nend;\n"),
                    anchor: None,
                }
            } else {
                SourceOriginInput::OpenBuffer {
                    uri: format!(
                        "file://{}",
                        fixture.root.join("alpha/src/main.miz").display()
                    ),
                    expected_version: 7,
                    actual_version: 7,
                    text: std::sync::Arc::from(if mode == "diagnostic" {
                        "end"
                    } else {
                        "definition\nend;\n"
                    }),
                }
            },
        };
        let session = request
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    descriptor.clone(),
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
        let publisher = publisher(snapshot, 0, &unit);
        if mode != "denied" {
            allow_frontend(&publisher, &unit);
        }
        if mode == "cancel" {
            input.cancellation.supersede_snapshot(snapshot);
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher.clone());
        let submission = driver
            .submit_captured(session, &ids, &snapshots, input)
            .unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        let results = run.phase_results.values().flatten().collect::<Vec<_>>();
        if mode == "diagnostic" {
            let drafts = results
                .iter()
                .flat_map(|result| &result.diagnostics)
                .flat_map(|batch| batch.drafts())
                .collect::<Vec<_>>();
            assert!(!drafts.is_empty());
            assert!(
                results
                    .iter()
                    .any(|result| result.status == PhaseStatus::Fatal)
            );
            let inputs = SourceLoadInputs {
                snapshot: submission.session.captured().snapshot(),
                captured: Some(submission.session.captured()),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            };
            let native_loader =
                FrontendSourceLoader::new(DiskSourceLoader::new(fixture.root.join("alpha")));
            let actual = native_loader
                .load_source_unit(
                    SourceUnitRequest {
                        snapshot,
                        input: descriptor,
                    },
                    &InMemorySessionIdAllocator::new(),
                )
                .unwrap();
            assert_eq!(
                actual.source_id,
                inputs.snapshot.source_versions[0].source_id
            );
            let oracle = Frontend::new(
                native_loader,
                inputs.dependency_lexical_provider(&[]),
                MizarParserSeam,
            )
            .run_loaded(actual)
            .unwrap();
            assert_eq!(drafts.len(), oracle.diagnostics.len());
            for (draft, diagnostic) in drafts.iter().zip(&oracle.diagnostics) {
                let (number, class, local_code) = match &diagnostic.code {
                    DiagnosticCode::Lexing(
                        mizar_frontend::lexing::LexingDiagnosticKind::ScopeSkeleton(
                            mizar_frontend::lexing::ScopeSkeletonDiagnosticCode::UnmatchedEnd,
                        ),
                    ) => (33, "scope_skeleton", "lexing.scope_skeleton.unmatched_end"),
                    DiagnosticCode::Syntax(key) if key.as_ref() == "unrecoverable_input" => {
                        (52, "syntax", "syntax.unrecoverable_input")
                    }
                    code => panic!("unexpected producer diagnostic {code:?}"),
                };
                assert_eq!(
                    draft.code(),
                    mizar_diagnostics::registry::DiagnosticCode::from_parts(
                        mizar_diagnostics::registry::DiagnosticSeverity::Error,
                        number
                    )
                    .unwrap()
                );
                assert_eq!(
                    draft.stable_detail_key(),
                    mizar_diagnostics::registry::DiagnosticRegistry::builtin()
                        .lookup(draft.code())
                        .unwrap()
                        .semantic_name
                );
                assert_eq!(draft.source_snapshot(), snapshot);
                assert_eq!(draft.phase(), DiagnosticPhase::Frontend);
                assert_eq!(draft.category(), FailureCategory::ParseError);
                assert_eq!(draft.message(), diagnostic.message.as_ref());
                let DiagnosticLocation::SourceRange(range) = diagnostic.location else {
                    panic!("actual source range");
                };
                let DiagnosticPrimaryLocation::Span(span) = draft.primary_location() else {
                    panic!("converted source range");
                };
                assert_eq!(span.range(), range);
                assert_eq!(range.source_id, oracle.source.source_id);
                assert_eq!(
                    draft
                        .secondary_spans()
                        .iter()
                        .map(|span| span.anchor())
                        .collect::<Vec<_>>(),
                    diagnostic.secondary.iter().collect::<Vec<_>>()
                );
                assert_eq!(
                    draft
                        .notes()
                        .iter()
                        .map(|note| note.message())
                        .collect::<Vec<_>>(),
                    diagnostic
                        .recovery_note
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    draft.details().entries().get("frontend.class"),
                    Some(&DiagnosticDetailValue::String(class.into()))
                );
                assert_eq!(
                    draft.details().entries().get("frontend.code"),
                    Some(&DiagnosticDetailValue::String(local_code.into()))
                );
                assert_eq!(draft.details().entries().len(), 2);
                assert!(draft.notes().iter().all(|note| note.span().is_none()));
                assert!(draft.fixes().is_empty());
                assert!(draft.explanation().is_none());
            }
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Failed)
            );
        } else if mode == "cancel" {
            assert!(
                run.task_states
                    .iter()
                    .any(|record| record.state == TaskState::Cancelled)
            );
            assert!(results.iter().all(|result| result.output_refs.is_empty()));
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Cancelled)
            );
        } else {
            assert!(
                results
                    .iter()
                    .any(|result| result.status == PhaseStatus::Blocking)
            );
            assert!(
                results
                    .iter()
                    .flat_map(|result| &result.output_refs)
                    .all(|output| output.output_kind() != &OutputKind::new("FrontendOutput"))
            );
            if mode == "generated" {
                assert!(results.iter().all(|result| result.output_refs.is_empty()));
            }
        }
    }
}

#[test]
fn native_capture_and_prepared_submission_fail_closed_without_replacing_pending_context() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (request, _) = fixture.submit_request(vec![], vec![]);
    let control = InMemorySessionIdAllocator::new();
    assert_eq!(
        ids.next_request_id().unwrap(),
        control.next_request_id().unwrap()
    );
    let error = request
        .clone()
        .capture_sources(&ids, &snapshots, vec![])
        .unwrap_err();
    assert!(
        matches!(error, DriverSubmitError::NativeSourceVersionsProvided { request: original } if *original == request)
    );
    assert_eq!(
        ids.next_request_id().unwrap(),
        control.next_request_id().unwrap()
    );
    assert_eq!(
        ids.next_session_id().unwrap(),
        control.next_session_id().unwrap()
    );
    let (mut request, input) = fixture.submit_request(vec![], vec![]);
    request.source_inputs.versions.clear();
    let descriptor = SourceInput {
        package_id: fixture.version.package_id.clone(),
        module_path: fixture.version.module_path.clone(),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: fixture.version.edition.clone(),
        origin: SourceOriginInput::OpenBuffer {
            uri: format!(
                "file://{}",
                fixture.root.join("alpha/src/main.miz").display()
            ),
            expected_version: 7,
            actual_version: 6,
            text: std::sync::Arc::from("definition\nend;\n"),
        },
    };
    let mut valid = descriptor.clone();
    let SourceOriginInput::OpenBuffer { actual_version, .. } = &mut valid.origin else {
        unreachable!()
    };
    *actual_version = 7;
    request.clone().allocate(&control).unwrap();
    let seeded = request
        .clone()
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                valid.clone(),
            )],
        )
        .unwrap();
    let expected_pending = request.clone().allocate(&control).unwrap();
    let error = request
        .clone()
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                descriptor.clone(),
            )],
        )
        .unwrap_err();
    let DriverSubmitError::SnapshotCapture { error } = error else {
        panic!("native capture error")
    };
    assert_eq!(error.pending, expected_pending);
    assert!(matches!(
        error.error,
        mizar_session::SnapshotError::SourceLoading {
            error: mizar_session::SourceLoadError::StaleLspDocumentVersion {
                expected: 7,
                actual: 6
            }
        }
    ));
    assert!(snapshots.is_current_for_request(seeded.captured().snapshot().id, seeded.request().id));
    assert!(
        !snapshots
            .is_current_for_request(seeded.captured().snapshot().id, error.pending.request.id)
    );
    let session = request
        .clone()
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                valid.clone(),
            )],
        )
        .unwrap();
    let mut driver = CompilerDriver::new(source_registry());
    let mut newer = request;
    newer.generation = BuildRequestGeneration::new(10);
    let mut cancelled = newer
        .capture_sources(
            &ids,
            &snapshots,
            vec![(DiskSourceLoader::new(fixture.root.join("alpha")), valid)],
        )
        .unwrap();
    cancelled.cancel();
    let cancelled_id = cancelled.id();
    assert!(
        matches!(driver.submit_captured(cancelled, &ids, &snapshots, input), Err(DriverSubmitError::InvalidCapturedSession { session }) if session.id() == cancelled_id)
    );
    assert!(driver.session(cancelled_id).is_none());
    let (_, input) = fixture.submit_request(vec![], vec![]);
    assert!(matches!(
        driver.submit_captured(session.clone(), &ids, &SnapshotRegistry::new(), input),
        Err(DriverSubmitError::InvalidCapturedSession { .. })
    ));
    let (_, input) = fixture.submit_request(vec![], vec![]);
    let submission = driver
        .submit_captured(session.clone(), &ids, &snapshots, input)
        .unwrap();
    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByMissingPhaseServices
    );
    let (_, input) = fixture.submit_request(vec![], vec![]);
    assert!(matches!(
        driver.submit_captured(session, &ids, &snapshots, input),
        Err(DriverSubmitError::InvalidCapturedSession { .. })
    ));
    let (legacy_request, input) = fixture.submit_request(vec![], vec![]);
    let legacy = legacy_request
        .allocate(&ids)
        .unwrap()
        .capture_snapshot(&snapshots)
        .unwrap();
    assert!(matches!(
        driver.submit_captured(legacy, &ids, &snapshots, input),
        Err(DriverSubmitError::InvalidCapturedSession { .. })
    ));
}

#[test]
fn native_open_buffer_payloads_publish_internal_blobs_with_genuine_parent_authority() {
    use mizar_frontend::{
        cache_key::{ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION, TOKEN_STREAM_CACHE_KEY_VERSION},
        orchestration::Frontend,
        source::{FrontendSourceLoader, source_unit_from_loaded},
    };
    use mizar_ir::{
        dispatch_input::SealedParentOutputHandle,
        identity::NamedInputHash,
        publisher::{OutputOrigin, PublicationTarget, PublishError, PublishOutputInput},
        storage::{BlobDecodeError, BlobDecoder, IrSideTables, SchemaVersion, SideTableRecord},
    };
    use std::sync::{Arc, atomic::AtomicUsize};
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let disk_submission = fixture.submit(&ids, &snapshots);
    let disk_context = &disk_submission.session.captured().snapshot();
    let package_root = fixture.root.join("alpha");
    let input = SourceInput {
        package_id: fixture.version.package_id.clone(),
        module_path: fixture.version.module_path.clone(),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: fixture.version.edition.clone(),
        origin: SourceOriginInput::OpenBuffer {
            uri: format!("file://{}", package_root.join("src/main.miz").display()),
            expected_version: 7,
            actual_version: 7,
            text: Arc::from("\u{feff}definition\r\nend;\r\n"),
        },
    };
    let (snapshot, _lease, captured) = snapshots
        .load_snapshot(
            ids.next_request_id().unwrap(),
            (
                disk_context.workspace_root.clone(),
                disk_context.dependency_artifacts.clone(),
                disk_context.lockfile_hash,
                disk_context.toolchain.clone(),
                disk_context.verifier_config_hash,
            ),
            vec![(DiskSourceLoader::new(&package_root), input)],
        )
        .unwrap();
    let (loaded, captured_input) = captured.into_iter().next().unwrap();
    let source = source_unit_from_loaded(loaded, package_root.join("src/main.miz"));
    assert_eq!(snapshot.source_versions[0].source_id, source.source_id);
    assert!(matches!(
        source.origin,
        SourceOrigin::OpenBuffer { version: 7 }
    ));
    let inputs = SourceLoadInputs {
        captured: None,
        snapshot: &snapshot,
        build_plan: disk_submission.build_plan.as_ref().unwrap(),
        module_index: disk_submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let output = Frontend::new(
        FrontendSourceLoader::new(DiskSourceLoader::new(&package_root)),
        inputs.dependency_lexical_provider(&[]),
        MizarParserSeam,
    )
    .run_loaded(source.clone())
    .unwrap();
    assert!(output.ast.is_some());
    assert!(output.diagnostics.is_empty());
    let unit = &source_task(&disk_submission).unit;
    let publisher = publisher(snapshot.id, 0, unit);
    allow_frontend(&publisher, unit);
    let WorkUnit::Module { module } = unit else {
        unreachable!()
    };
    let ir_unit = IrWorkUnit::new(format!(
        "{:?}:{:?}",
        module.package.as_str(),
        module.path.as_str()
    ));
    let source_id = source.source_id;
    let expected_hash = source.source_hash;
    let source_key = SourceUnitCacheKey::from_source(&source).stable_hash();
    let source_bytes = source.canonical_open_buffer_bytes(&captured_input).unwrap();
    let source_storage_hash = Hash::from_bytes(blake3::derive_key(
        "mizar-frontend/source-unit-storage-side-table/v1",
        &source_bytes,
    ));
    let source_decodes = Arc::new(AtomicUsize::new(0));
    let mut source_handle = None;
    for target in [
        PublicationTarget::CurrentPackage,
        PublicationTarget::InternalOnly,
    ] {
        let decode_input = captured_input.clone();
        let decodes = source_decodes.clone();
        let phase = IrPipelinePhase::new("SourceLoad");
        let kind = OutputKind::new("SourceUnit");
        let result = publisher.publish(PublishOutputInput {
            slot: publisher.allocate(
                snapshot.id,
                phase.clone(),
                ir_unit.clone(),
                kind.clone(),
                SchemaVersion::new(1),
            ),
            snapshot: snapshot.id,
            phase,
            work_unit: ir_unit.clone(),
            output_kind: kind,
            schema_version: SchemaVersion::new(1),
            payload: source.clone(),
            canonical_payload: Some(source_key.as_bytes().to_vec()),
            storage_payload: Some(source_bytes.clone()),
            decode: BlobDecoder::new(move |bytes| {
                decodes.fetch_add(1, Ordering::SeqCst);
                SourceUnit::from_canonical_storage_bytes(bytes, source_id, &decode_input)
                    .filter(|source| source.source_hash == expected_hash)
                    .ok_or_else(|| BlobDecodeError::new("invalid native OpenBuffer source"))
            }),
            parents: vec![],
            named_input_hashes: vec![NamedInputHash {
                name: "source".into(),
                domain: SOURCE_UNIT_CACHE_KEY_VERSION.into(),
                digest: source_key,
            }],
            side_tables: IrSideTables {
                source_maps: vec![SideTableRecord::new(
                    "source-storage",
                    source.normalized_path.as_str(),
                    source_storage_hash,
                )],
                ..IrSideTables::default()
            },
            origin: OutputOrigin::OpenBuffer,
            target,
        });
        if target == PublicationTarget::CurrentPackage {
            assert!(matches!(result, Err(PublishError::OpenBufferOutput { .. })));
        } else {
            source_handle = Some(result.unwrap());
        }
    }
    let source_handle = source_handle.unwrap();
    let source_parent = SealedParentOutputHandle::from_published_output(
        &publisher,
        snapshot.id,
        source_handle.erase(),
    )
    .unwrap();
    let output_bytes = output.canonical_open_buffer_bytes(&captured_input).unwrap();
    let mut semantic_bytes = b"mizar-frontend/output-publication/v1\0".to_vec();
    semantic_bytes.extend_from_slice(
        output
            .cache_keys
            .ast
            .as_ref()
            .unwrap()
            .stable_hash()
            .as_bytes(),
    );
    let output_storage_hash = Hash::from_bytes(blake3::derive_key(
        "mizar-frontend/output-storage-side-table/v1",
        &output_bytes,
    ));
    let output_decodes = Arc::new(AtomicUsize::new(0));
    let decode_input = captured_input;
    let decodes = output_decodes.clone();
    let phase = IrPipelinePhase::new("Frontend");
    let kind = OutputKind::new("FrontendOutput");
    let output_handle = publisher
        .publish(PublishOutputInput {
            slot: publisher.allocate(
                snapshot.id,
                phase.clone(),
                ir_unit.clone(),
                kind.clone(),
                SchemaVersion::new(1),
            ),
            snapshot: snapshot.id,
            phase,
            work_unit: ir_unit,
            output_kind: kind,
            schema_version: SchemaVersion::new(1),
            payload: output.clone(),
            canonical_payload: Some(semantic_bytes),
            storage_payload: Some(output_bytes),
            decode: BlobDecoder::new(move |bytes| {
                decodes.fetch_add(1, Ordering::SeqCst);
                FrontendOutput::from_canonical_storage_bytes(bytes, source_id, &decode_input)
                    .filter(|output| output.source.source_hash == expected_hash)
                    .ok_or_else(|| BlobDecodeError::new("invalid native OpenBuffer frontend"))
            }),
            parents: vec![source_parent.into_output_ref()],
            named_input_hashes: vec![
                NamedInputHash {
                    name: "active-lexical-environment".into(),
                    domain: ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION.into(),
                    digest: output.cache_keys.active_lexical_environment.stable_hash(),
                },
                NamedInputHash {
                    name: "tokens".into(),
                    domain: TOKEN_STREAM_CACHE_KEY_VERSION.into(),
                    digest: output.cache_keys.tokens.stable_hash(),
                },
            ],
            side_tables: IrSideTables {
                source_maps: vec![SideTableRecord::new(
                    "frontend-storage",
                    source.normalized_path.as_str(),
                    output_storage_hash,
                )],
                ..IrSideTables::default()
            },
            origin: OutputOrigin::OpenBuffer,
            target: PublicationTarget::InternalOnly,
        })
        .unwrap();
    for handle in [source_handle.erase(), output_handle.erase()] {
        assert!(matches!(handle.placement(), StoragePlacement::Blob { .. }));
        assert!(matches!(
            publisher.validate_current_output(snapshot.id, &handle),
            Err(PublishError::OutputNotCurrentPackage { .. })
        ));
        assert!(
            SealedParentOutputHandle::from_current_output(&publisher, snapshot.id, handle.clone())
                .is_err()
        );
        assert!(
            SealedParentOutputHandle::from_published_output(&publisher, snapshot.id, handle)
                .is_ok()
        );
    }
    assert_eq!(source_decodes.load(Ordering::SeqCst), 0);
    assert_eq!(output_decodes.load(Ordering::SeqCst), 0);
    fixture.write(b"changed disk source");
    fs::remove_file(package_root.join("src/main.miz")).unwrap();
    assert_eq!(*publisher.storage().get(&source_handle).unwrap(), source);
    assert_eq!(*publisher.storage().get(&output_handle).unwrap(), output);
    assert_eq!(source_decodes.load(Ordering::SeqCst), 1);
    assert_eq!(output_decodes.load(Ordering::SeqCst), 1);
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

#[cfg(unix)]
#[test]
fn source_load_rejects_package_root_symlink_outside_workspace() {
    let fixture = Fixture::new(b"alpha");
    let outside = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    fs::remove_dir_all(fixture.root.join("alpha")).unwrap();
    std::os::unix::fs::symlink(outside.root.join("alpha"), fixture.root.join("alpha")).unwrap();
    let (result, _) = execute(&submission, &ids, usize::MAX);
    assert_eq!(result.status, PhaseStatus::Blocking);
    assert!(result.output_refs.is_empty());
    assert!(result.diagnostics.is_empty());
}

#[test]
fn missing_package_root_retains_real_loader_diagnostic() {
    let fixture = Fixture::new(b"alpha");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let submission = fixture.submit(&ids, &snapshots);
    fs::remove_dir_all(fixture.root.join("alpha")).unwrap();
    let (result, _) = execute(&submission, &ids, usize::MAX);
    assert_source_error(&result, &fixture.version, 602, "source.unreadable_file");
}

#[test]
fn native_open_edits_match_independent_clean_real_prefixes_across_replay_controls() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let leaf =
        "definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
    fs::write(fixture.root.join("alpha/src/leaf.miz"), leaf).unwrap();
    let mut retained = None;
    let mut baselines = Vec::new();
    let mut canonical_ids: Vec<Vec<SourceId>> = Vec::new();
    for (stage, workers, order, reverse) in [
        (0, 1, CompletionOrder::Canonical, false),
        (0, 1, CompletionOrder::Canonical, false),
        (0, 4, CompletionOrder::Reverse, false),
        (0, 4, CompletionOrder::Canonical, true),
        (1, 1, CompletionOrder::Canonical, false),
        (1, 1, CompletionOrder::Canonical, false),
        (1, 4, CompletionOrder::Reverse, false),
        (1, 4, CompletionOrder::Canonical, true),
        (2, 1, CompletionOrder::Canonical, false),
        (2, 1, CompletionOrder::Canonical, false),
        (2, 4, CompletionOrder::Reverse, false),
        (2, 4, CompletionOrder::Canonical, true),
    ] {
        let first = stage == baselines.len();
        let main = match stage {
            0 => "import alpha.leaf;\ntheorem T: a combine b = a;\n",
            1 => "import alpha.leaf;\ntheorem EditedT: a combine b = b;\n",
            _ => "import alpha.leaf;\ntheorem T: ;\n",
        };
        let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
        request.source_inputs.versions.clear();
        request.generation = BuildRequestGeneration::new(stage as u64);
        input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![
                WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
            ],
        }]);
        input.dependency_overlay = ModuleDependencyOverlay::unavailable();
        input.worker_count = workers;
        input.completion_order = order;
        let mut descriptors = ["leaf", "main"]
            .map(|module| {
                let path = format!("src/{module}.miz");
                let version = if module == "main" {
                    stage as i64 + 7
                } else {
                    7
                };
                (
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new(module),
                        normalized_path: normalize_path(
                            &fixture.root.join("alpha"),
                            Path::new(&path),
                        )
                        .unwrap(),
                        edition: Edition::new("2025"),
                        origin: SourceOriginInput::OpenBuffer {
                            uri: format!(
                                "file://{}",
                                fixture.root.join("alpha").join(&path).display()
                            ),
                            expected_version: version,
                            actual_version: version,
                            text: std::sync::Arc::from(format!(
                                "\u{feff}{}",
                                if module == "main" { main } else { leaf }.replace('\n', "\r\n")
                            )),
                        },
                    },
                )
            })
            .to_vec();
        if reverse {
            descriptors.reverse();
        }
        let (mut driver, ids, snapshots, previous) = if first && stage > 0 {
            let (driver, ids, snapshots, owner, old_snapshot, output, unused) =
                retained.take().unwrap();
            (
                driver,
                ids,
                snapshots,
                Some((owner, old_snapshot, output, unused)),
            )
        } else {
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            (
                CompilerDriver::new(builder.build().unwrap()),
                InMemorySessionIdAllocator::new(),
                SnapshotRegistry::new(),
                None,
            )
        };
        let session = request
            .clone()
            .capture_sources(&ids, &snapshots, descriptors.clone())
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let versions = &session.captured().snapshot().source_versions;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(snapshot, 0, &unit);
        for module in ["leaf", "main"] {
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new(module),
                ),
            };
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SourceLoad"),
                OutputKind::new("SourceUnit"),
                IrWorkUnit::new(format!("{:?}:{:?}", "alpha", module)),
            ));
            allow_frontend(&owner, &unit);
        }
        driver = driver.with_output_publisher(owner.clone());
        let mut obsolete = None;
        if let Some((old_owner, old_snapshot, old_output, unused)) = previous {
            let old_owner: std::sync::Arc<PhaseOutputPublisher> = old_owner;
            let old_output: AnyPhaseOutputRef = old_output;
            let unused: mizar_driver::request::BuildSession = unused;
            assert_ne!(snapshot, old_snapshot);
            old_owner
                .replace_current_snapshot(old_snapshot, snapshot)
                .unwrap();
            assert!(old_owner.validate_current_snapshot(old_snapshot).is_err());
            for target in [old_snapshot, snapshot] {
                assert!(
                    old_owner
                        .validate_published_output(target, &old_output)
                        .is_err()
                );
                assert!(
                    SealedParentOutputHandle::from_published_output(
                        &old_owner,
                        target,
                        old_output.clone()
                    )
                    .is_err()
                );
            }
            assert_eq!(unused.state(), BuildSessionState::SnapshotCaptured);
            assert!(driver.session(unused.id()).is_none());
            assert!(snapshots.is_current_for_request(old_snapshot, unused.request().id));
            let (_, obsolete_input) = fixture.submit_request(vec![], vec![]);
            // Submit below, after the edited generation has become current in the same Driver.
            obsolete = Some((unused, obsolete_input));
        }
        if first {
            canonical_ids.push(
                ["leaf", "main"]
                    .map(|module| {
                        versions
                            .iter()
                            .find(|version| version.module_path.as_str() == module)
                            .unwrap()
                            .source_id
                    })
                    .to_vec(),
            );
        }
        let submission = driver
            .submit_captured_with_import_discovery(session, &ids, &snapshots, input)
            .unwrap();
        assert_eq!(
            submission.status,
            if stage == 2 {
                DriverSubmissionStatus::SchedulerValidated
            } else {
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            }
        );
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(if stage == 2 {
                BuildSessionOutcome::Failed
            } else {
                BuildSessionOutcome::Blocked
            })
        );
        let snapshot = submission.session.captured().snapshot();
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        for task in graph.tasks().iter().filter(|task| {
            task.phases.iter().any(|phase| {
                !matches!(
                    phase,
                    PipelinePhase::PackageResolve
                        | PipelinePhase::SourceLoad
                        | PipelinePhase::Frontend
                )
            })
        }) {
            assert_eq!(
                run.task_states
                    .iter()
                    .find(|state| state.task_id == task.id)
                    .unwrap()
                    .state,
                TaskState::Blocked
            );
        }
        let mut projection = Vec::new();
        for (ordinal, module) in ["leaf", "main"].into_iter().enumerate() {
            let source_task = graph.tasks().iter().find(|task| task.kind == TaskKind::SourceLoad && matches!(&task.unit, WorkUnit::Module { module: id } if id.path.as_str() == module)).unwrap();
            let frontend_task = graph
                .tasks()
                .iter()
                .find(|task| task.kind == TaskKind::Frontend && task.unit == source_task.unit)
                .unwrap();
            assert_eq!(run.phase_results[&source_task.id].len(), 1);
            assert_eq!(run.phase_results[&frontend_task.id].len(), 1);
            let source_result = &run.phase_results[&source_task.id][0];
            let frontend_result = &run.phase_results[&frontend_task.id][0];
            let diagnostic_case = stage == 2 && module == "main";
            assert_eq!(source_result.output_refs.len(), 1);
            assert_eq!(
                frontend_result.output_refs.len(),
                usize::from(!diagnostic_case)
            );
            assert_eq!(source_result.status, PhaseStatus::Complete);
            assert_eq!(
                frontend_result.status,
                if diagnostic_case {
                    PhaseStatus::Recoverable
                } else {
                    PhaseStatus::Complete
                }
            );
            let descriptor = &descriptors
                .iter()
                .find(|(_, input)| input.module_path.as_str() == module)
                .unwrap()
                .1;
            let version = snapshot
                .source_versions
                .iter()
                .find(|version| version.module_path.as_str() == module)
                .unwrap();
            let source_ref = &source_result.output_refs[0];
            assert!(matches!(
                source_ref.placement(),
                StoragePlacement::Blob { .. }
            ));
            assert!(source_ref.lineage().parents.is_empty());
            let source = owner
                .storage()
                .get(
                    &owner
                        .storage()
                        .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(source.source_id, version.source_id);
            assert_eq!(source.source_hash, version.source_hash);
            assert_eq!(source.origin, version.origin);
            assert_eq!(
                source.source_text.as_ref(),
                if module == "main" { main } else { leaf }
            );
            let source_bytes = source.canonical_open_buffer_bytes(descriptor).unwrap();
            let cache_intents = [PipelinePhase::SourceLoad, PipelinePhase::Frontend].map(|phase| {
                let parents = if phase == PipelinePhase::Frontend {
                    frontend_task
                        .dependencies
                        .iter()
                        .flat_map(|id| run.phase_results.get(id).into_iter().flatten())
                        .filter(|result| result.status == PhaseStatus::Complete)
                        .flat_map(|result| &result.output_refs)
                        .map(|parent| {
                            SealedParentOutputHandle::from_published_output(
                                &owner,
                                snapshot.id,
                                parent.clone(),
                            )
                            .unwrap()
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let input = PhaseInput::new(
                    source_task.unit.clone(),
                    PhaseDispatchInputBundle::new(
                        snapshot.id,
                        SourceUnitCacheKey::from_source(&source).stable_hash(),
                        vec![],
                        parents,
                    )
                    .unwrap(),
                );
                let intent = driver
                    .registry()
                    .cache_key_for_phase(phase, &input)
                    .unwrap()
                    .intent;
                assert!(
                    matches!(&intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                );
                intent
            });
            let frontend = frontend_result.output_refs.first().map(|handle| {
                assert!(matches!(handle.placement(), StoragePlacement::Blob { .. }));
                let output = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                handle,
                                &OutputKind::new("FrontendOutput"),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                assert_eq!(output.source, *source);
                let parents = handle
                    .lineage()
                    .parents
                    .iter()
                    .map(|parent| {
                        let parent = owner.registry().output_lineage(*parent).unwrap();
                        (parent.phase, parent.work_unit, parent.output_kind)
                    })
                    .collect::<Vec<_>>();
                (
                    output.canonical_open_buffer_bytes(descriptor).unwrap(),
                    output.cache_keys.canonical_bytes().unwrap(),
                    parents,
                )
            });
            let diagnostics = source_result
                .diagnostics
                .iter()
                .chain(&frontend_result.diagnostics)
                .flat_map(|batch| batch.drafts())
                .map(|draft| {
                    assert_eq!(draft.source_snapshot(), snapshot.id);
                    let DiagnosticPrimaryLocation::Span(span) = draft.primary_location() else {
                        panic!("real parser span");
                    };
                    let SourceAnchor::Range(mut range) = *span.anchor() else {
                        panic!("real parser range");
                    };
                    source.line_map.validate_range(range).unwrap();
                    assert_eq!(range.source_id, version.source_id);
                    range.source_id = canonical_ids[stage][ordinal];
                    assert!(draft.secondary_spans().is_empty());
                    assert!(draft.notes().iter().all(|note| note.span().is_none()));
                    assert!(
                        draft
                            .details()
                            .entries()
                            .values()
                            .all(|value| matches!(value, DiagnosticDetailValue::String(_)))
                    );
                    assert!(draft.fixes().is_empty());
                    assert!(draft.explanation().is_none());
                    (
                        draft.source_snapshot(),
                        draft.code(),
                        draft.phase(),
                        draft.category(),
                        draft.stable_detail_key().to_owned(),
                        draft.message().to_owned(),
                        DiagnosticSpan::from_anchor(
                            SourceAnchor::Range(range),
                            span.role(),
                            span.label().map(str::to_owned),
                            span.freshness(),
                            span.zero_width(),
                        )
                        .unwrap(),
                        draft.notes().to_vec(),
                        draft.details().clone(),
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(diagnostics.is_empty(), !diagnostic_case);
            projection.push((
                module,
                snapshot.id,
                source_bytes,
                SourceUnitCacheKey::from_source(&source).stable_hash(),
                frontend,
                (source_result.status, frontend_result.status),
                diagnostics,
                cache_intents,
            ));
        }
        if let Some(expected) = baselines.get(stage) {
            assert_eq!(
                &projection, expected,
                "stage={stage}, workers={workers}, order={order:?}, reverse={reverse}"
            );
        } else {
            baselines.push(projection);
            if stage > 0 {
                assert_ne!(baselines[stage][1].1, baselines[stage - 1][1].1);
                assert_ne!(baselines[stage][1].2, baselines[stage - 1][1].2);
                if stage == 1 {
                    assert_ne!(baselines[stage][1].4, baselines[stage - 1][1].4);
                }
                let (unused, obsolete_input) = obsolete.take().unwrap();
                let obsolete = driver
                    .submit_captured(unused, &ids, &snapshots, obsolete_input)
                    .unwrap();
                assert_eq!(
                    obsolete.status,
                    DriverSubmissionStatus::SupersededBeforeSubmission
                );
                assert_eq!(
                    obsolete.session.state(),
                    BuildSessionState::Finished(BuildSessionOutcome::Superseded)
                );
                assert!(obsolete.scheduler_run.is_none());
                assert!(matches!(
                    obsolete.publication_decision,
                    mizar_driver::request::PublicationDecision::Suppressed(_)
                ));
            }
            let unused = request
                .capture_sources(&ids, &snapshots, descriptors)
                .unwrap();
            let output = run
                .phase_results
                .values()
                .flatten()
                .flat_map(|result| &result.output_refs)
                .find(|handle| handle.output_kind() == &OutputKind::new("SourceUnit"))
                .unwrap()
                .clone();
            retained = Some((driver, ids, snapshots, owner, snapshot.id, output, unused));
        }
    }
}

#[test]
fn current_storage_comparison_binds_native_disk_blob_readbacks_after_removal() {
    let raw = "\u{feff}definition\r\nend;\r\n";
    let fixture = Fixture::new(raw.as_bytes());
    let descriptor = SourceInput {
        package_id: fixture.version.package_id.clone(),
        module_path: fixture.version.module_path.clone(),
        normalized_path: fixture.version.normalized_path.clone(),
        edition: fixture.version.edition.clone(),
        origin: SourceOriginInput::Disk {
            path: "src/main.miz".into(),
        },
    };
    let mut previous: Option<SourceUnit> = None;
    for text in [raw, "definition\nend;\n"] {
        fixture.write(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut request, input) = fixture.submit_request(vec![], vec![]);
        request.source_inputs.versions.clear();
        let session = request
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    descriptor.clone(),
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
        let owner = publisher(snapshot, 0, &unit);
        allow_frontend(&owner, &unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        let submission = driver
            .submit_captured(session, &ids, &snapshots, input)
            .unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        let mut source = None;
        let mut output = None;
        for result in run.phase_results.values().flatten() {
            for handle in &result.output_refs {
                assert_eq!(result.status, PhaseStatus::Complete);
                assert!(matches!(handle.placement(), StoragePlacement::Blob { .. }));
                if handle.output_kind() == &OutputKind::new("SourceUnit") {
                    source = Some(
                        owner
                            .storage()
                            .get(
                                &owner
                                    .storage()
                                    .typed_handle::<SourceUnit>(
                                        handle,
                                        &OutputKind::new("SourceUnit"),
                                    )
                                    .unwrap(),
                            )
                            .unwrap(),
                    );
                } else if handle.output_kind() == &OutputKind::new("FrontendOutput") {
                    output = Some(
                        owner
                            .storage()
                            .get(
                                &owner
                                    .storage()
                                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                                        handle,
                                        &OutputKind::new("FrontendOutput"),
                                    )
                                    .unwrap(),
                            )
                            .unwrap(),
                    );
                }
            }
        }
        let source = source.unwrap();
        let output = output.unwrap();
        assert!(
            output
                .source
                .matches_canonical_storage(&source, &descriptor)
        );
        assert_eq!(output.source, *source);
        if let Some(old) = &previous {
            assert_eq!(old.source_hash, source.source_hash);
            assert!(!source.matches_canonical_storage(old, &descriptor));
            assert!(!output.source.matches_canonical_storage(old, &descriptor));
            let restored = SourceUnit::from_canonical_storage_bytes(
                &old.canonical_disk_bytes().unwrap(),
                source.source_id,
                &descriptor,
            )
            .unwrap();
            assert!(!restored.matches_canonical_storage(&source, &descriptor));
        }
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
            let phase_input = PhaseInput::new(
                unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    SourceUnitCacheKey::from_source(&source).stable_hash(),
                    vec![],
                ),
            );
            assert_eq!(
                driver
                    .registry()
                    .cache_key_for_phase(phase, &phase_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey {
                    reason: reason.into()
                }
            );
        }
        previous = Some(source.as_ref().clone());
    }
}

#[test]
fn native_disk_cache_request_projects_real_storage_and_replays_without_local_ids() {
    use mizar_cache::cache_key::{
        CACHE_KEY_SCHEMA_VERSION, CacheKeyBuildOutcome, CacheKeyBuilder, FootprintCompleteness,
    };
    use mizar_proof::policy::VerifierPolicy;
    let policy = VerifierPolicy::release();
    let mut previous = Vec::new();
    for raw in ["\u{feff}definition\r\nend;\r\n", "definition\nend;\n"] {
        let mut replays = Vec::new();
        for replay in 0..2 {
            let fixture = Fixture::new(raw.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let source_ids = InMemorySessionIdAllocator::new();
            for _ in 0..replay {
                source_ids.next_source_id(snapshot_id(0x62)).unwrap();
            }
            let snapshots = SnapshotRegistry::with_allocator(source_ids);
            let (mut draft, input) = fixture.submit_request(vec![], vec![]);
            draft.source_inputs.versions.clear();
            let session = draft
                .capture_sources(
                    &ids,
                    &snapshots,
                    vec![(
                        DiskSourceLoader::new(fixture.root.join("alpha")),
                        SourceInput {
                            package_id: fixture.version.package_id.clone(),
                            module_path: fixture.version.module_path.clone(),
                            normalized_path: fixture.version.normalized_path.clone(),
                            edition: fixture.version.edition.clone(),
                            origin: SourceOriginInput::Disk {
                                path: "src/main.miz".into(),
                            },
                        },
                    )],
                )
                .unwrap();
            let version = session.captured().snapshot().source_versions[0].clone();
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    version.package_id.clone(),
                    version.module_path.clone(),
                ),
            };
            let owner = publisher(session.captured().snapshot().id, 0, &unit);
            allow_frontend(&owner, &unit);
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
            let submission = driver
                .submit_captured(session, &ids, &snapshots, input)
                .unwrap();
            let captured = submission.session.captured();
            let phase_input = PhaseInput::new(
                unit,
                PhaseDispatchInputBundle::without_parent_outputs(
                    captured.snapshot().id,
                    source_key(&version),
                    vec![],
                ),
            );
            let request = SourceLoadInputs {
                snapshot: captured.snapshot(),
                captured: Some(captured),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            }
            .prepare_native_disk_cache_request(&phase_input, &policy)
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
            let handle = &result.output_refs[0];
            let source = owner
                .storage()
                .get(
                    &owner
                        .storage()
                        .typed_handle::<SourceUnit>(handle, &OutputKind::new("SourceUnit"))
                        .unwrap(),
                )
                .unwrap();
            let side = owner.storage().side_tables_by_ref(handle).unwrap();
            assert_eq!(request.phase.as_str(), handle.phase().as_str());
            assert_eq!(request.work_unit.as_str(), handle.work_unit().as_str());
            let identity = request.source_identity.as_ref().unwrap();
            assert_eq!(identity.package_id, source.package_id);
            assert_eq!(identity.module_path, source.module_path);
            assert_eq!(identity.normalized_source_path, source.normalized_path);
            assert_eq!(identity.source_hash, source.source_hash);
            assert_eq!(identity.language_edition, source.edition);
            assert_eq!(request.input_hashes.len(), 3);
            assert_eq!(request.input_hashes[0].name, "source");
            assert_eq!(
                request.input_hashes[0].domain,
                SOURCE_UNIT_CACHE_KEY_VERSION
            );
            assert_eq!(
                request.input_hashes[0].digest,
                SourceUnitCacheKey::from_source(&source).stable_hash()
            );
            assert_eq!(request.input_hashes[1].name, "source-storage");
            assert_eq!(
                request.input_hashes[1].domain,
                "mizar-frontend/source-unit-storage-side-table/v1"
            );
            assert_eq!(request.input_hashes[1].digest, side.source_maps[0].digest);
            assert_eq!(request.input_hashes[2].name, "captured-lockfile");
            assert_eq!(
                request.input_hashes[2].domain,
                "mizar-driver/captured-lockfile-identity/v1"
            );
            assert_eq!(
                request.input_hashes[2].digest,
                captured.snapshot().lockfile_hash
            );
            assert_eq!(
                request.config_hash,
                captured.snapshot().verifier_config_hash
            );
            assert_eq!(
                request.policy_fingerprint.hash(),
                policy.policy_fingerprint().hash()
            );
            assert_ne!(request.config_hash, request.policy_fingerprint.hash());
            assert_eq!(
                request.cache_schema_version.as_str(),
                CACHE_KEY_SCHEMA_VERSION
            );
            let descriptor = driver
                .registry()
                .descriptor_for_phase(PipelinePhase::SourceLoad)
                .unwrap();
            assert_eq!(
                request
                    .schema_versions
                    .iter()
                    .map(|entry| (
                        entry.schema_family.as_str(),
                        entry.name.as_str(),
                        entry.version.as_str(),
                    ))
                    .collect::<Vec<_>>(),
                vec![
                    (
                        "phase-output",
                        "SourceUnit",
                        descriptor.schema_version.as_str()
                    ),
                    ("source-key", "SourceUnit", SOURCE_UNIT_CACHE_KEY_VERSION),
                    ("source-storage", "Disk", "mizar-frontend-source-disk-v1"),
                ]
            );
            let validation = &request.validation_inputs;
            assert_eq!(
                validation.cache_schema_compatibility.as_str(),
                CACHE_KEY_SCHEMA_VERSION
            );
            assert_eq!(validation.toolchain_compatibility.len(), 1);
            assert_eq!(
                validation.toolchain_compatibility[0].family,
                "mizar-session/toolchain"
            );
            assert_eq!(validation.toolchain_compatibility[0].field_name, "identity");
            assert_eq!(
                validation.toolchain_compatibility[0].value,
                captured.snapshot().toolchain.identity()
            );
            assert_eq!(
                validation
                    .policy_compatibility
                    .iter()
                    .map(|field| (
                        field.family.as_str(),
                        field.field_name.as_str(),
                        field.value.clone(),
                    ))
                    .collect::<Vec<_>>(),
                vec![
                    (
                        "mizar-proof/verifier-policy",
                        "schema_version",
                        policy.schema_version().to_string()
                    ),
                    (
                        "mizar-proof/verifier-policy",
                        "profile_id",
                        policy.profile_id().to_owned()
                    ),
                    (
                        "mizar-proof/verifier-policy",
                        "checker_schema_version",
                        policy.checker_schema_version().to_string()
                    ),
                    (
                        "mizar-proof/verifier-policy",
                        "fingerprint",
                        policy
                            .policy_fingerprint()
                            .hash()
                            .as_bytes()
                            .iter()
                            .map(|byte| format!("{byte:02x}"))
                            .collect::<String>()
                    ),
                ]
            );
            assert_eq!(
                validation.footprint_completeness,
                FootprintCompleteness::Complete
            );
            assert!(!validation.uncacheable);
            assert!(request.dependency_hashes.is_empty() && request.dependency_slices.is_empty());
            assert!(validation.dependency_artifacts.is_empty());
            assert!(validation.dependency_slice_fingerprints.is_empty());
            assert!(validation.proof_reuse_schema_versions.is_empty());
            assert!(validation.proof_reuse_evidence_identities.is_empty());
            assert!(validation.diagnostic_refs.is_empty());
            assert_eq!(validation.canonical_vc_fingerprint, None);
            assert_eq!(validation.local_context_fingerprint, None);
            assert_eq!(validation.obligation_anchor_fingerprint, None);
            assert_eq!(validation.selected_proof_witness_hash, None);
            assert_eq!(validation.deterministic_discharge_hash, None);
            assert_eq!(validation.proof_reuse_validation_hash, None);
            assert_eq!(
                driver
                    .registry()
                    .cache_key_for_phase(PipelinePhase::SourceLoad, &phase_input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey {
                    reason: "current disk source maps require a real source load".into(),
                }
            );
            let CacheKeyBuildOutcome::Cacheable(key) =
                CacheKeyBuilder::new(request.clone()).build()
            else {
                panic!("bound source-only request must construct a cacheable key");
            };
            replays.push((request, key.final_hash, source.source_id));
        }
        assert_eq!(replays[0].0, replays[1].0);
        assert_eq!(replays[0].1, replays[1].1);
        assert_ne!(replays[0].2, replays[1].2);
        previous.push(replays.remove(0));
    }
    assert_eq!(previous[0].0.source_identity, previous[1].0.source_identity);
    assert_eq!(previous[0].0.input_hashes[0], previous[1].0.input_hashes[0]);
    assert_ne!(previous[0].0.input_hashes[1], previous[1].0.input_hashes[1]);
    assert_ne!(previous[0].1, previous[1].1);
}

#[test]
fn native_disk_cache_request_keeps_captured_and_explicit_policy_identities_distinct() {
    use mizar_cache::cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder};
    use mizar_proof::policy::VerifierPolicy;
    let fixture = Fixture::new(b"definition\nend;\n");
    let mut baseline = None;
    for mode in [
        "baseline",
        "text",
        "config",
        "lock",
        "toolchain",
        "profile",
        "settings",
    ] {
        fixture.write(if mode == "text" {
            b"definition\n\nend;\n"
        } else {
            b"definition\nend;\n"
        });
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, mut input) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        if mode == "config" {
            draft.verifier_config = VerifierConfigInput::new(hash(3));
        }
        if mode == "lock" {
            draft.dependency_inputs.lockfile_hash = hash(4);
        }
        if mode == "toolchain" {
            draft.dependency_inputs.toolchain = ToolchainInfo::new("different-toolchain");
            input.plan_request.toolchain = draft.dependency_inputs.toolchain.clone();
        }
        let session = draft
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: fixture.version.package_id.clone(),
                        module_path: fixture.version.module_path.clone(),
                        normalized_path: fixture.version.normalized_path.clone(),
                        edition: fixture.version.edition.clone(),
                        origin: SourceOriginInput::Disk {
                            path: "src/main.miz".into(),
                        },
                    },
                )],
            )
            .unwrap();
        let mut driver = CompilerDriver::new(source_registry());
        let submission = driver
            .submit_captured(session, &ids, &snapshots, input)
            .unwrap();
        let captured = submission.session.captured();
        let phase_input = PhaseInput::new(
            source_task(&submission).unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                captured.snapshot().id,
                source_key(&captured.snapshot().source_versions[0]),
                vec![],
            ),
        );
        let policy = match mode {
            "profile" => VerifierPolicy::release().with_profile_id("alternate-release"),
            "settings" => VerifierPolicy::release().with_require_kernel_certificates(false),
            _ => VerifierPolicy::release(),
        };
        let request = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            allocator: &ids,
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
        }
        .prepare_native_disk_cache_request(&phase_input, &policy)
        .unwrap();
        let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(request.clone()).build()
        else {
            panic!("bound native Disk request");
        };
        if let Some((old, old_key)) = &baseline {
            assert_ne!(&request, old, "{mode}");
            assert_ne!(&key.final_hash, old_key, "{mode}");
            if mode == "config" {
                assert_ne!(request.config_hash, old.config_hash);
                assert_eq!(request.policy_fingerprint, old.policy_fingerprint);
            } else if mode == "profile" || mode == "settings" {
                assert_eq!(request.config_hash, old.config_hash);
                assert_ne!(request.policy_fingerprint, old.policy_fingerprint);
            } else {
                assert_eq!(request.config_hash, old.config_hash);
                assert_eq!(request.policy_fingerprint, old.policy_fingerprint);
            }
            match mode {
                "text" => assert_ne!(request.source_identity, old.source_identity),
                "lock" => assert_ne!(request.input_hashes[2], old.input_hashes[2]),
                "toolchain" => assert_ne!(
                    request.validation_inputs.toolchain_compatibility,
                    old.validation_inputs.toolchain_compatibility
                ),
                "settings" => assert_eq!(
                    request.validation_inputs.policy_compatibility[1],
                    old.validation_inputs.policy_compatibility[1]
                ),
                _ => {}
            }
        } else {
            baseline = Some((request, key.final_hash));
        }
    }
}

#[test]
fn native_disk_cache_request_rejects_unbound_dispatch_capture_roots_and_policy_metadata() {
    use mizar_proof::policy::{
        DEFAULT_CHECKER_SCHEMA_VERSION, POLICY_SCHEMA_VERSION, VerifierPolicy,
    };
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
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
        .clone()
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                descriptor.clone(),
            )],
        )
        .unwrap();
    let mut foreign_draft = draft.clone();
    foreign_draft.verifier_config = VerifierConfigInput::new(hash(9));
    let foreign = foreign_draft
        .capture_sources(
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                descriptor.clone(),
            )],
        )
        .unwrap();
    let mut driver = CompilerDriver::new(source_registry());
    let submission = driver
        .submit_captured(session, &ids, &snapshots, input)
        .unwrap();
    let captured = submission.session.captured();
    let phase_input = PhaseInput::new(
        source_task(&submission).unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(
            captured.snapshot().id,
            source_key(&captured.snapshot().source_versions[0]),
            vec![],
        ),
    );
    let inputs = SourceLoadInputs {
        snapshot: captured.snapshot(),
        captured: Some(captured),
        allocator: &ids,
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
    };
    let policy = VerifierPolicy::release();
    assert!(
        inputs
            .prepare_native_disk_cache_request(&phase_input, &policy)
            .is_ok()
    );
    for rejected in [
        SourceLoadInputs {
            captured: None,
            ..inputs
        },
        SourceLoadInputs {
            captured: Some(foreign.captured()),
            ..inputs
        },
    ] {
        assert!(
            rejected
                .prepare_native_disk_cache_request(&phase_input, &policy)
                .is_err()
        );
    }
    let legacy = fixture.submit(&ids, &SnapshotRegistry::new());
    let legacy_capture = legacy.session.captured();
    let legacy_input = PhaseInput::new(
        phase_input.work_unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(
            legacy_capture.snapshot().id,
            source_key(&legacy_capture.snapshot().source_versions[0]),
            vec![],
        ),
    );
    assert!(
        SourceLoadInputs {
            snapshot: legacy_capture.snapshot(),
            captured: Some(legacy_capture),
            build_plan: legacy.build_plan.as_ref().unwrap(),
            module_index: legacy.module_index.as_ref().unwrap(),
            allocator: &ids,
        }
        .prepare_native_disk_cache_request(&legacy_input, &policy)
        .is_err()
    );
    for unsupported in [
        policy.clone().with_profile_id(" \t\n"),
        policy
            .clone()
            .with_schema_version(POLICY_SCHEMA_VERSION + 1),
        policy
            .clone()
            .with_checker_schema_version(DEFAULT_CHECKER_SCHEMA_VERSION + 1),
    ] {
        assert!(
            inputs
                .prepare_native_disk_cache_request(&phase_input, &unsupported)
                .is_err()
        );
    }
    for mode in [
        "missing",
        "duplicate",
        "source-id",
        "source-hash",
        "origin",
        "workspace",
        "config",
        "lock",
        "toolchain",
    ] {
        let mut snapshot = captured.snapshot().clone();
        match mode {
            "missing" => snapshot.source_versions.clear(),
            "duplicate" => snapshot
                .source_versions
                .push(snapshot.source_versions[0].clone()),
            "source-id" => {
                ids.next_source_id(snapshot.id).unwrap();
                snapshot.source_versions[0].source_id = ids.next_source_id(snapshot.id).unwrap();
            }
            "source-hash" => snapshot.source_versions[0].source_hash = hash(8),
            "origin" => {
                snapshot.source_versions[0].origin = SourceOrigin::OpenBuffer { version: 1 }
            }
            "workspace" => snapshot.workspace_root = WorkspaceRoot::new("/foreign"),
            "config" => snapshot.verifier_config_hash = hash(8),
            "lock" => snapshot.lockfile_hash = hash(8),
            "toolchain" => snapshot.toolchain = ToolchainInfo::new("foreign"),
            _ => unreachable!(),
        }
        assert!(
            SourceLoadInputs {
                snapshot: &snapshot,
                ..inputs
            }
            .prepare_native_disk_cache_request(&phase_input, &policy)
            .is_err(),
            "{mode}"
        );
    }
    for mode in [
        "duplicate-package",
        "root",
        "source-root",
        "manifest",
        "edition",
        "toolchain",
    ] {
        let mut plan = inputs.build_plan.clone();
        match mode {
            "duplicate-package" => plan.packages.push(plan.packages[0].clone()),
            "root" | "source-root" | "manifest" => {
                let mizar_build::planner::PackagePlanSource::Workspace {
                    root,
                    source_root,
                    manifest_path,
                } = &mut plan.packages[0].source
                else {
                    panic!("workspace package");
                };
                match mode {
                    "root" => *root = "..".into(),
                    "source-root" => *source_root = "foreign/src".into(),
                    "manifest" => *manifest_path = "foreign/mizar.pkg".into(),
                    _ => unreachable!(),
                }
            }
            "edition" => plan.packages[0].edition = Edition::new("foreign"),
            "toolchain" => plan.toolchain = ToolchainInfo::new("foreign"),
            _ => unreachable!(),
        }
        assert!(
            SourceLoadInputs {
                build_plan: &plan,
                ..inputs
            }
            .prepare_native_disk_cache_request(&phase_input, &policy)
            .is_err(),
            "{mode}"
        );
    }
    for mode in [
        "missing-module",
        "duplicate-module",
        "duplicate-index-package",
        "path",
    ] {
        let mut index = inputs.module_index.clone();
        match mode {
            "missing-module" => index.modules.clear(),
            "duplicate-module" => index.modules.push(index.modules[0].clone()),
            "duplicate-index-package" => index.packages.push(index.packages[0].clone()),
            "path" => {
                let ModuleIndexLocation::WorkspaceFile {
                    source_relative_path,
                    ..
                } = &mut index.modules[0].location
                else {
                    panic!("workspace source");
                };
                *source_relative_path = "foreign.miz".into();
            }
            _ => unreachable!(),
        }
        assert!(
            SourceLoadInputs {
                module_index: &index,
                ..inputs
            }
            .prepare_native_disk_cache_request(&phase_input, &policy)
            .is_err(),
            "{mode}"
        );
    }
    for (snapshot, unit, digest, dependencies) in [
        (
            snapshot_id(0x71),
            phase_input.work_unit.clone(),
            source_key(&fixture.version),
            vec![],
        ),
        (
            phase_input.snapshot,
            WorkUnit::Workspace,
            source_key(&fixture.version),
            vec![],
        ),
        (
            phase_input.snapshot,
            phase_input.work_unit.clone(),
            hash(9),
            vec![],
        ),
        (
            phase_input.snapshot,
            phase_input.work_unit.clone(),
            source_key(&fixture.version),
            vec![hash(9)],
        ),
        (
            phase_input.snapshot,
            WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("foreign"),
                    ModulePath::new("main"),
                ),
            },
            source_key(&fixture.version),
            vec![],
        ),
    ] {
        let rejected = PhaseInput::new(
            unit,
            PhaseDispatchInputBundle::without_parent_outputs(snapshot, digest, dependencies),
        );
        assert!(
            inputs
                .prepare_native_disk_cache_request(&rejected, &policy)
                .is_err()
        );
    }
    let owner = publisher(phase_input.snapshot, 0, &phase_input.work_unit);
    let source_result = driver
        .registry()
        .execute_phase_with_resources(
            PipelinePhase::SourceLoad,
            phase_input.clone(),
            PhaseExecutionResources {
                output_publisher: Some(owner.clone()),
                source_load: Some(inputs),
                diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                    DiagnosticPhase::SourceLoad,
                    phase_input.snapshot,
                    "frontend.source_load",
                ))),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result;
    assert_eq!(source_result.status, PhaseStatus::Complete);
    let parent = SealedParentOutputHandle::from_published_output(
        &owner,
        phase_input.snapshot,
        source_result.output_refs[0].clone(),
    )
    .unwrap();
    let rejected = PhaseInput::new(
        phase_input.work_unit.clone(),
        PhaseDispatchInputBundle::new(
            phase_input.snapshot,
            source_key(&captured.snapshot().source_versions[0]),
            vec![],
            vec![parent],
        )
        .unwrap(),
    );
    assert!(
        inputs
            .prepare_native_disk_cache_request(&rejected, &policy)
            .is_err()
    );
    for origin in [
        SourceOriginInput::OpenBuffer {
            uri: format!(
                "file://{}",
                fixture.root.join("alpha/src/main.miz").display()
            ),
            expected_version: 1,
            actual_version: 1,
            text: std::sync::Arc::from("definition\nend;\n"),
        },
        SourceOriginInput::Generated {
            generator: mizar_session::GeneratedSourceKind::new("cache-request-test"),
            text: std::sync::Arc::from("definition\nend;\n"),
            anchor: None,
        },
    ] {
        let native = draft
            .clone()
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        origin,
                        ..descriptor.clone()
                    },
                )],
            )
            .unwrap();
        let snapshot = native.captured().snapshot();
        let rejected = PhaseInput::new(
            phase_input.work_unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                snapshot.id,
                source_key(&snapshot.source_versions[0]),
                vec![],
            ),
        );
        assert!(
            SourceLoadInputs {
                snapshot,
                captured: Some(native.captured()),
                ..inputs
            }
            .prepare_native_disk_cache_request(&rejected, &policy)
            .is_err()
        );
    }
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

#[test]
fn native_disk_cache_record_resident_round_trips_real_store_with_owner_bytes() {
    use mizar_cache::{
        cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
        cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot},
    };
    use mizar_ir::cache_adapter::{
        CacheAdapterCacheability, EncodeCacheRecordInput, EncodeCacheRecordOutcome,
    };
    let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (driver, submission, owner, descriptor, handle) =
        native_disk_record_submission(&fixture, &ids, &snapshots, usize::MAX);
    let captured = submission.session.captured();
    let version = captured.snapshot().source_versions[0].clone();
    let snapshot = captured.snapshot().id;
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            version.package_id.clone(),
            version.module_path.clone(),
        ),
    };
    let phase_input = PhaseInput::new(
        unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(snapshot, source_key(&version), vec![]),
    );
    let inputs = SourceLoadInputs {
        snapshot: captured.snapshot(),
        captured: Some(captured),
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let source = owner.storage().get(&handle).unwrap();
    let adapter = mizar_ir::cache_adapter::IrCacheAdapter::new(owner.clone());
    let policy = mizar_proof::policy::VerifierPolicy::release();

    assert_eq!(handle.any().placement(), &StoragePlacement::Resident);
    assert_eq!(
        source.line_map,
        LineMap::new(source.source_id, "definition\nend;\n")
    );
    assert_eq!(source.loading_map, fixture.loading_map);
    assert_eq!(source.source_id, version.source_id);
    let request = inputs
        .prepare_native_disk_cache_request(&phase_input, &policy)
        .unwrap();
    let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(request.clone()).build() else {
        panic!("genuine source-only key");
    };
    let EncodeCacheRecordOutcome::Encoded(record) = inputs
        .encode_native_disk_cache_record(&phase_input, &policy, &adapter, handle.clone())
        .unwrap()
    else {
        panic!("bound resident encoding");
    };
    let disk = source.canonical_disk_bytes().unwrap();
    assert_eq!(
        SourceUnit::from_canonical_storage_bytes(&disk, version.source_id, &descriptor).unwrap(),
        *source
    );
    let expected = adapter.encode(EncodeCacheRecordInput {
        handle: handle.clone(),
        cache_key: key.clone(),
        produced_by: request.validation_inputs.toolchain_compatibility.clone(),
        canonical_payload: SourceUnitCacheKey::from_source(&source)
            .stable_hash()
            .as_bytes()
            .to_vec(),
        storage_payload: disk,
        cacheability: CacheAdapterCacheability::Cacheable,
    });
    assert_eq!(EncodeCacheRecordOutcome::Encoded(record.clone()), expected);
    assert_eq!(record.header.key, key);
    assert_eq!(
        record.header.produced_by,
        request.validation_inputs.toolchain_compatibility
    );
    assert!(!record.header.uncacheable);
    let store = CacheStoreRoot::new(fixture.root.join("cache"));
    assert!(matches!(store.lookup(&key), CacheLookupOutcome::Miss(_)));
    assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
    let CacheLookupOutcome::Hit(readback) = store.lookup(&key) else {
        panic!("actual stored hit");
    };
    assert_eq!(readback, record);
    assert_eq!(
        store.insert(&readback).unwrap(),
        CacheInsertOutcome::AlreadyPresent
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
                .cache_key_for_phase(phase, &phase_input)
                .unwrap()
                .intent,
            PhaseCacheIntent::NoKey {
                reason: reason.into()
            }
        );
    }
}

#[test]
fn native_disk_cache_record_blob_replay_relocates_ids_and_separates_raw_maps() {
    use mizar_ir::cache_adapter::EncodeCacheRecordOutcome;
    let mut formats = Vec::new();
    for raw in ["\u{feff}definition\r\nend;\r\n", "definition\nend;\n"] {
        let mut replays = Vec::new();
        for replay in 0..2 {
            let fixture = Fixture::new(raw.as_bytes());
            let ids = InMemorySessionIdAllocator::new();
            let source_ids = InMemorySessionIdAllocator::new();
            for _ in 0..replay {
                source_ids.next_source_id(snapshot_id(0x62)).unwrap();
            }
            let snapshots = SnapshotRegistry::with_allocator(source_ids);
            let (_driver, submission, owner, descriptor, handle) =
                native_disk_record_submission(&fixture, &ids, &snapshots, 0);
            let captured = submission.session.captured();
            let version = captured.snapshot().source_versions[0].clone();
            let snapshot = captured.snapshot().id;
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    version.package_id.clone(),
                    version.module_path.clone(),
                ),
            };
            let phase_input = PhaseInput::new(
                unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    source_key(&version),
                    vec![],
                ),
            );
            let inputs = SourceLoadInputs {
                snapshot: captured.snapshot(),
                captured: Some(captured),
                build_plan: submission.build_plan.as_ref().unwrap(),
                module_index: submission.module_index.as_ref().unwrap(),
                allocator: &ids,
            };
            let source = owner.storage().get(&handle).unwrap();
            let adapter = mizar_ir::cache_adapter::IrCacheAdapter::new(owner.clone());
            let policy = mizar_proof::policy::VerifierPolicy::release();
            assert!(matches!(
                handle.any().placement(),
                StoragePlacement::Blob { .. }
            ));
            assert_eq!(
                source.line_map,
                LineMap::new(source.source_id, "definition\nend;\n")
            );
            assert_eq!(
                source.loading_map,
                fixture
                    .loading_map
                    .as_ref()
                    .map(|expected_map| LoadingMap::new(
                        source.source_id,
                        "definition\nend;\n",
                        expected_map.origin.clone(),
                        expected_map.segments.clone()
                    ))
            );
            let disk = source.canonical_disk_bytes().unwrap();
            let relocated = SourceUnit::from_canonical_storage_bytes(
                &disk,
                fixture.version.source_id,
                &descriptor,
            )
            .unwrap();
            assert!(source.matches_canonical_storage(&relocated, &descriptor));
            let EncodeCacheRecordOutcome::Encoded(record) = inputs
                .encode_native_disk_cache_record(&phase_input, &policy, &adapter, handle.clone())
                .unwrap()
            else {
                panic!("bound blob encoding");
            };
            replays.push((
                record,
                source.source_id,
                source.source_hash,
                disk,
                owner.storage().side_tables(&handle).unwrap(),
            ));
        }
        assert_eq!(replays[0].0, replays[1].0);
        assert_ne!(replays[0].1, replays[1].1);
        assert_eq!(replays[0].3, replays[1].3);
        formats.push(replays.remove(0));
    }
    assert_eq!(formats[0].2, formats[1].2);
    assert_eq!(
        formats[0].0.header.key.source_identity,
        formats[1].0.header.key.source_identity
    );
    assert_ne!(formats[0].3, formats[1].3);
    assert_ne!(formats[0].4.source_maps, formats[1].4.source_maps);
    assert_ne!(
        formats[0].0.header.key.final_hash,
        formats[1].0.header.key.final_hash
    );
    assert_ne!(formats[0].0.output, formats[1].0.output);
}

#[test]
fn native_disk_cache_record_rejects_physical_current_capture_and_output_bindings() {
    use mizar_ir::{
        cache_adapter::{EncodeCacheRecordOutcome, IrCacheAdapter},
        storage::CollectInput,
    };
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (_driver, submission, owner, _descriptor, handle) =
        native_disk_record_submission(&fixture, &ids, &snapshots, usize::MAX);
    let captured = submission.session.captured();
    let version = captured.snapshot().source_versions[0].clone();
    let snapshot = captured.snapshot().id;
    let unit = WorkUnit::Module {
        module: mizar_build::module_index::ModuleId::new(
            version.package_id.clone(),
            version.module_path.clone(),
        ),
    };
    let phase_input = PhaseInput::new(
        unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(snapshot, source_key(&version), vec![]),
    );
    let inputs = SourceLoadInputs {
        snapshot: captured.snapshot(),
        captured: Some(captured),
        build_plan: submission.build_plan.as_ref().unwrap(),
        module_index: submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let source = owner.storage().get(&handle).unwrap();
    let adapter = mizar_ir::cache_adapter::IrCacheAdapter::new(owner.clone());
    let policy = mizar_proof::policy::VerifierPolicy::release();

    assert!(matches!(
        inputs
            .encode_native_disk_cache_record(&phase_input, &policy, &adapter, handle.clone())
            .unwrap(),
        EncodeCacheRecordOutcome::Encoded(_)
    ));
    assert!(
        SourceLoadInputs {
            captured: None,
            ..inputs
        }
        .encode_native_disk_cache_record(&phase_input, &policy, &adapter, handle.clone())
        .is_err()
    );
    let foreign = IrCacheAdapter::new(publisher(snapshot, usize::MAX, &unit));
    assert!(
        inputs
            .encode_native_disk_cache_record(&phase_input, &policy, &foreign, handle.clone())
            .is_err()
    );
    let unpublished = IrCacheAdapter::new(std::sync::Arc::new(PhaseOutputPublisher::new(
        owner.storage().clone(),
        owner.registry().clone(),
    )));
    unpublished.publisher().register_current_snapshot(snapshot);
    assert!(
        inputs
            .encode_native_disk_cache_record(&phase_input, &policy, &unpublished, handle.clone())
            .is_err()
    );
    for (raw, relocate) in [
        ("definition\nend;\n", true),
        ("\u{feff}definition\r\nend;\r\n", false),
    ] {
        fixture.write(raw.as_bytes());
        let other_ids = InMemorySessionIdAllocator::new();
        let source_ids = InMemorySessionIdAllocator::new();
        if relocate {
            source_ids.next_source_id(snapshot_id(0x62)).unwrap();
        }
        let other_snapshots = SnapshotRegistry::with_allocator(source_ids);
        let (_, other_submission, other_owner, _, other_handle) =
            native_disk_record_submission(&fixture, &other_ids, &other_snapshots, usize::MAX);
        let other_capture = other_submission.session.captured();
        assert_eq!(other_capture.snapshot().id, snapshot);
        let other_source = other_owner.storage().get(&other_handle).unwrap();
        assert_eq!(other_source.source_hash, source.source_hash);
        if relocate {
            assert_ne!(other_source.source_id, source.source_id);
            assert_eq!(
                other_source.canonical_disk_bytes(),
                source.canonical_disk_bytes()
            );
        } else {
            assert_eq!(other_source.source_id, source.source_id);
            assert_ne!(
                other_source.canonical_disk_bytes(),
                source.canonical_disk_bytes()
            );
        }
        assert!(
            other_owner
                .validate_current_output(snapshot, other_handle.any())
                .is_ok()
        );
        assert!(
            other_owner
                .storage()
                .validate_handle(other_handle.any())
                .is_ok()
        );
        let other_adapter = IrCacheAdapter::new(other_owner);
        assert!(
            inputs
                .encode_native_disk_cache_record(
                    &phase_input,
                    &policy,
                    &other_adapter,
                    other_handle
                )
                .is_err()
        );
    }
    let wrong_dispatch = PhaseInput::new(
        unit,
        PhaseDispatchInputBundle::without_parent_outputs(snapshot, hash(9), vec![]),
    );
    assert!(
        inputs
            .encode_native_disk_cache_record(&wrong_dispatch, &policy, &adapter, handle.clone())
            .is_err()
    );
    owner.mark_obsolete(snapshot).unwrap();
    assert!(owner.storage().validate_handle(handle.any()).is_ok());
    assert!(
        inputs
            .encode_native_disk_cache_record(&phase_input, &policy, &adapter, handle.clone())
            .is_err()
    );

    fixture.write(b"definition\nend;\n");
    let collected_ids = InMemorySessionIdAllocator::new();
    let collected_snapshots = SnapshotRegistry::new();
    let (_, collected_submission, collected_owner, _, collected_handle) =
        native_disk_record_submission(&fixture, &collected_ids, &collected_snapshots, usize::MAX);
    let collected_capture = collected_submission.session.captured();
    let collected_snapshot = collected_capture.snapshot();
    let collected_input = PhaseInput::new(
        phase_input.work_unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(
            collected_snapshot.id,
            source_key(&collected_snapshot.source_versions[0]),
            vec![],
        ),
    );
    let collected_inputs = SourceLoadInputs {
        snapshot: collected_snapshot,
        captured: Some(collected_capture),
        build_plan: collected_submission.build_plan.as_ref().unwrap(),
        module_index: collected_submission.module_index.as_ref().unwrap(),
        allocator: &collected_ids,
    };
    let collected_adapter = IrCacheAdapter::new(collected_owner.clone());
    assert!(matches!(
        collected_inputs
            .encode_native_disk_cache_record(
                &collected_input,
                &policy,
                &collected_adapter,
                collected_handle.clone()
            )
            .unwrap(),
        EncodeCacheRecordOutcome::Encoded(_)
    ));
    assert!(
        collected_owner
            .validate_current_output(collected_snapshot.id, collected_handle.any())
            .is_ok()
    );
    assert!(
        collected_owner
            .storage()
            .collect(CollectInput {
                snapshot: collected_snapshot.id,
                protected_outputs: vec![]
            })
            .outputs_dropped
            > 0
    );
    assert!(
        collected_owner
            .validate_current_output(collected_snapshot.id, collected_handle.any())
            .is_ok()
    );
    assert!(
        collected_owner
            .storage()
            .validate_handle(collected_handle.any())
            .is_err()
    );
    assert!(
        collected_inputs
            .encode_native_disk_cache_record(
                &collected_input,
                &policy,
                &collected_adapter,
                collected_handle
            )
            .is_err()
    );
}

#[test]
fn native_disk_expected_cache_payload_rebinds_current_resident_source() {
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot};
    use mizar_ir::{
        cache_adapter::{
            CacheAdapterMiss, CacheRehydrateOutcome, EncodeCacheRecordOutcome, IrCacheAdapter,
            RehydrateCacheHitInput,
        },
        publisher::PublishError,
        storage::{BlobDecodeError, BlobDecoder},
    };
    let fixtures = [
        Fixture::new(b"definition\nend;\n"),
        Fixture::new(b"definition\nend;\n"),
    ];
    let ids = InMemorySessionIdAllocator::new();
    let old_snapshots = SnapshotRegistry::new();
    let source_ids = InMemorySessionIdAllocator::new();
    source_ids.next_source_id(snapshot_id(0x62)).unwrap();
    let current_snapshots = SnapshotRegistry::with_allocator(source_ids);
    let (_, old_submission, old_owner, _, old_handle) =
        native_disk_record_submission(&fixtures[0], &ids, &old_snapshots, usize::MAX);
    let (driver, current_submission, current_owner, descriptor, current_handle) =
        native_disk_record_submission(&fixtures[1], &ids, &current_snapshots, usize::MAX);
    let old_capture = old_submission.session.captured();
    let current_capture = current_submission.session.captured();
    let unit = source_task(&current_submission).unit.clone();
    let policy = mizar_proof::policy::VerifierPolicy::release();
    let mut records = Vec::new();
    for (submission, owner, handle) in [
        (&old_submission, &old_owner, &old_handle),
        (&current_submission, &current_owner, &current_handle),
    ] {
        let captured = submission.session.captured();
        let input = PhaseInput::new(
            unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                captured.snapshot().id,
                source_key(&captured.snapshot().source_versions[0]),
                vec![],
            ),
        );
        let inputs = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: &ids,
        };
        let EncodeCacheRecordOutcome::Encoded(record) = inputs
            .encode_native_disk_cache_record(
                &input,
                &policy,
                &IrCacheAdapter::new(owner.clone()),
                handle.clone(),
            )
            .unwrap()
        else {
            panic!("genuine native record");
        };
        records.push(record);
    }
    assert_ne!(old_capture.snapshot().id, current_capture.snapshot().id);
    let old_source = old_owner.storage().get(&old_handle).unwrap();
    let current_source = current_owner.storage().get(&current_handle).unwrap();
    assert_ne!(old_source.source_id, current_source.source_id);
    assert_eq!(records[0], records[1]);
    let key = records[1].header.key.clone();
    let store = CacheStoreRoot::new(fixtures[1].root.join("cache"));
    let expected_bytes = SourceUnitCacheKey::from_source(&current_source)
        .stable_hash()
        .as_bytes()
        .to_vec();
    let expected_tables = current_owner
        .storage()
        .side_tables(&current_handle)
        .unwrap();
    let snapshot = current_capture.snapshot().id;
    for mode in ["miss", "denied", "obsolete", "resident"] {
        let target = if mode == "denied" {
            let target = std::sync::Arc::new(PhaseOutputPublisher::new(
                std::sync::Arc::new(IrStorageService::new()),
                std::sync::Arc::new(SnapshotHandleRegistry::new()),
            ));
            target.register_current_snapshot(snapshot);
            target
        } else {
            publisher(snapshot, usize::MAX, &unit)
        };
        if mode == "obsolete" {
            target.mark_obsolete(snapshot).unwrap();
        }
        if mode == "denied" {
            assert_eq!(
                store.insert(&records[0]).unwrap(),
                CacheInsertOutcome::Inserted
            );
        }
        let lookup = store.lookup(&key);
        if mode == "miss" {
            assert!(matches!(lookup, CacheLookupOutcome::Miss(_)));
        } else {
            assert!(matches!(lookup, CacheLookupOutcome::Hit(_)));
        }
        let source = current_source.clone();
        let descriptor = descriptor.clone();
        let adapter = IrCacheAdapter::new(target.clone());
        let outcome = adapter.rehydrate_with_expected_payload(
            RehydrateCacheHitInput {
                lookup,
                snapshot,
                phase: current_handle.phase().clone(),
                work_unit: current_handle.work_unit().clone(),
                output_kind: current_handle.output_kind().clone(),
                schema_version: current_handle.schema_version(),
                parents: vec![],
                named_input_hashes: current_handle.lineage().named_input_hashes.clone(),
                decode: BlobDecoder::new(move |bytes| {
                    SourceUnit::from_canonical_storage_bytes(bytes, source.source_id, &descriptor)
                        .filter(|decoded| decoded.matches_canonical_storage(&source, &descriptor))
                        .ok_or_else(|| BlobDecodeError::new("current source storage mismatch"))
                }),
            },
            &expected_bytes,
            &expected_tables,
        );
        match mode {
            "miss" => assert!(matches!(
                outcome,
                CacheRehydrateOutcome::Miss(CacheAdapterMiss::Cache(_))
            )),
            "denied" => assert!(
                matches!(outcome, CacheRehydrateOutcome::Miss(CacheAdapterMiss::Publisher { error }) if matches!(*error, PublishError::WorkUnitNotAllowed { .. }))
            ),
            "obsolete" => assert!(
                matches!(outcome, CacheRehydrateOutcome::Miss(CacheAdapterMiss::Publisher { error }) if matches!(*error, PublishError::ObsoleteSnapshot { .. }))
            ),
            _ => {
                let CacheRehydrateOutcome::Rehydrated(handle) = outcome else {
                    panic!("bound resident rehydration");
                };
                assert_eq!(handle.snapshot(), snapshot);
                assert_ne!(handle.output(), old_handle.output());
                assert_eq!(handle.placement(), &StoragePlacement::Resident);
                assert_eq!(*target.storage().get(&handle).unwrap(), *current_source);
                assert_eq!(
                    target.storage().side_tables(&handle).unwrap(),
                    expected_tables
                );
                assert!(
                    target
                        .validate_current_output(snapshot, handle.any())
                        .is_ok()
                );
            }
        }
        assert_eq!(
            adapter.successful_rehydrations(),
            u64::from(mode == "resident")
        );
    }
    let input = PhaseInput::new(
        unit,
        PhaseDispatchInputBundle::without_parent_outputs(
            snapshot,
            source_key(&current_capture.snapshot().source_versions[0]),
            vec![],
        ),
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
}

#[test]
fn native_disk_expected_cache_payload_rejects_current_bytes_and_raw_maps_before_decode() {
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot};
    use mizar_ir::{
        cache_adapter::{
            CacheAdapterMiss, CacheRehydrateOutcome, EncodeCacheRecordOutcome, IrCacheAdapter,
            RehydrateCacheHitInput,
        },
        storage::{BlobDecodeError, BlobDecoder},
    };
    use std::sync::{Arc, atomic::AtomicUsize};
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let old_fixture = Fixture::new(b"definition\nend;\n");
    let (_, old_submission, old_owner, _, old_handle) =
        native_disk_record_submission(&old_fixture, &ids, &snapshots, usize::MAX);
    let old_capture = old_submission.session.captured();
    let unit = source_task(&old_submission).unit.clone();
    let input = PhaseInput::new(
        unit.clone(),
        PhaseDispatchInputBundle::without_parent_outputs(
            old_capture.snapshot().id,
            source_key(&old_capture.snapshot().source_versions[0]),
            vec![],
        ),
    );
    let inputs = SourceLoadInputs {
        snapshot: old_capture.snapshot(),
        captured: Some(old_capture),
        build_plan: old_submission.build_plan.as_ref().unwrap(),
        module_index: old_submission.module_index.as_ref().unwrap(),
        allocator: &ids,
    };
    let EncodeCacheRecordOutcome::Encoded(record) = inputs
        .encode_native_disk_cache_record(
            &input,
            &mizar_proof::policy::VerifierPolicy::release(),
            &IrCacheAdapter::new(old_owner.clone()),
            old_handle.clone(),
        )
        .unwrap()
    else {
        panic!("genuine native record");
    };
    let store = CacheStoreRoot::new(old_fixture.root.join("cache"));
    assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
    let old_source = old_owner.storage().get(&old_handle).unwrap();
    for (raw, expected_miss) in [
        ("definition\nend;\n@", CacheAdapterMiss::PayloadHashMismatch),
        (
            "\u{feff}definition\r\nend;\r\n",
            CacheAdapterMiss::SideTableHashMismatch,
        ),
    ] {
        let fixture = Fixture::new(raw.as_bytes());
        let (_, _submission, owner, _, handle) =
            native_disk_record_submission(&fixture, &ids, &snapshots, usize::MAX);
        let source = owner.storage().get(&handle).unwrap();
        let expected_bytes = SourceUnitCacheKey::from_source(&source)
            .stable_hash()
            .as_bytes()
            .to_vec();
        let expected_tables = owner.storage().side_tables(&handle).unwrap();
        assert_ne!(
            expected_tables,
            old_owner.storage().side_tables(&old_handle).unwrap()
        );
        if expected_miss == CacheAdapterMiss::SideTableHashMismatch {
            assert_eq!(source.source_hash, old_source.source_hash);
            assert_eq!(source.source_text, old_source.source_text);
        } else {
            assert_ne!(source.source_hash, old_source.source_hash);
        }
        let target = publisher(handle.snapshot(), usize::MAX, &unit);
        let adapter = IrCacheAdapter::new(target.clone());
        let decodes = Arc::new(AtomicUsize::new(0));
        let calls = decodes.clone();
        // The old-key hit is intentionally offered against different authentic current expectations.
        let lookup = store.lookup(&record.header.key);
        assert!(matches!(lookup, CacheLookupOutcome::Hit(_)));
        let outcome = adapter.rehydrate_with_expected_payload(
            RehydrateCacheHitInput {
                lookup,
                snapshot: handle.snapshot(),
                phase: handle.phase().clone(),
                work_unit: handle.work_unit().clone(),
                output_kind: handle.output_kind().clone(),
                schema_version: handle.schema_version(),
                parents: vec![],
                named_input_hashes: handle.lineage().named_input_hashes.clone(),
                decode: BlobDecoder::<SourceUnit>::new(move |_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Err(BlobDecodeError::new("must reject before decoder"))
                }),
            },
            &expected_bytes,
            &expected_tables,
        );
        assert_eq!(outcome, CacheRehydrateOutcome::Miss(expected_miss));
        assert_eq!(decodes.load(Ordering::SeqCst), 0);
        assert_eq!(adapter.successful_rehydrations(), 0);
        assert!(
            target
                .validate_current_output(handle.snapshot(), handle.any())
                .is_err()
        );
        // Equal first slot IDs in independent fresh stores detect any hidden allocation.
        let baseline = publisher(handle.snapshot(), usize::MAX, &unit);
        assert_eq!(
            target
                .allocate::<SourceUnit>(
                    handle.snapshot(),
                    handle.phase().clone(),
                    handle.work_unit().clone(),
                    handle.output_kind().clone(),
                    handle.schema_version()
                )
                .slot(),
            baseline
                .allocate::<SourceUnit>(
                    handle.snapshot(),
                    handle.phase().clone(),
                    handle.work_unit().clone(),
                    handle.output_kind().clone(),
                    handle.schema_version()
                )
                .slot(),
        );
    }
}

#[test]
fn native_disk_expected_cache_blob_source_feeds_registered_frontend_like_clean_run() {
    use mizar_cache::{
        cache_key::{CacheKeyBuildOutcome, CacheKeyBuilder},
        cache_store::{CacheInsertOutcome, CacheLookupOutcome, CacheStoreRoot},
    };
    use mizar_ir::{
        cache_adapter::{
            CacheRehydrateOutcome, EncodeCacheRecordOutcome, IrCacheAdapter, RehydrateCacheHitInput,
        },
        storage::{BlobDecodeError, BlobDecoder},
    };
    let fixtures = [
        Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes()),
        Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes()),
    ];
    let ids = InMemorySessionIdAllocator::new();
    let old_snapshots = SnapshotRegistry::new();
    let source_ids = InMemorySessionIdAllocator::new();
    source_ids.next_source_id(snapshot_id(0x62)).unwrap();
    let current_snapshots = SnapshotRegistry::with_allocator(source_ids);
    let (_, old_submission, old_owner, _, old_handle) =
        native_disk_record_submission(&fixtures[0], &ids, &old_snapshots, 0);
    let (driver, submission, owner, descriptor, clean_source_handle) =
        native_disk_record_submission(&fixtures[1], &ids, &current_snapshots, 0);
    let unit = source_task(&submission).unit.clone();
    let policy = mizar_proof::policy::VerifierPolicy::release();
    let mut records = Vec::new();
    let mut keys = Vec::new();
    for (submission, owner, handle) in [
        (&old_submission, &old_owner, &old_handle),
        (&submission, &owner, &clean_source_handle),
    ] {
        let captured = submission.session.captured();
        let input = PhaseInput::new(
            unit.clone(),
            PhaseDispatchInputBundle::without_parent_outputs(
                captured.snapshot().id,
                source_key(&captured.snapshot().source_versions[0]),
                vec![],
            ),
        );
        let inputs = SourceLoadInputs {
            snapshot: captured.snapshot(),
            captured: Some(captured),
            build_plan: submission.build_plan.as_ref().unwrap(),
            module_index: submission.module_index.as_ref().unwrap(),
            allocator: &ids,
        };
        let CacheKeyBuildOutcome::Cacheable(key) = CacheKeyBuilder::new(
            inputs
                .prepare_native_disk_cache_request(&input, &policy)
                .unwrap(),
        )
        .build() else {
            panic!("genuine current source key");
        };
        keys.push(key);
        let EncodeCacheRecordOutcome::Encoded(record) = inputs
            .encode_native_disk_cache_record(
                &input,
                &policy,
                &IrCacheAdapter::new(owner.clone()),
                handle.clone(),
            )
            .unwrap()
        else {
            panic!("genuine native blob record");
        };
        records.push(record);
    }
    assert_eq!(keys[0], keys[1]);
    assert_eq!(records[0], records[1]);
    let source = owner.storage().get(&clean_source_handle).unwrap();
    assert_ne!(
        source.source_id,
        old_owner.storage().get(&old_handle).unwrap().source_id
    );
    assert_ne!(clean_source_handle.snapshot(), old_handle.snapshot());
    let expected_bytes = SourceUnitCacheKey::from_source(&source)
        .stable_hash()
        .as_bytes()
        .to_vec();
    let expected_tables = owner.storage().side_tables(&clean_source_handle).unwrap();
    let store = CacheStoreRoot::new(fixtures[1].root.join("cache"));
    assert_eq!(
        store.insert(&records[0]).unwrap(),
        CacheInsertOutcome::Inserted
    );
    let lookup = store.lookup(&keys[1]);
    assert!(matches!(lookup, CacheLookupOutcome::Hit(_)));
    let target = publisher(clean_source_handle.snapshot(), 0, &unit);
    let adapter = IrCacheAdapter::new(target.clone());
    let decoded_source = source.clone();
    let CacheRehydrateOutcome::Rehydrated(handle) = adapter.rehydrate_with_expected_payload(
        RehydrateCacheHitInput {
            lookup,
            snapshot: clean_source_handle.snapshot(),
            phase: clean_source_handle.phase().clone(),
            work_unit: clean_source_handle.work_unit().clone(),
            output_kind: clean_source_handle.output_kind().clone(),
            schema_version: clean_source_handle.schema_version(),
            parents: vec![],
            named_input_hashes: clean_source_handle.lineage().named_input_hashes.clone(),
            decode: BlobDecoder::new(move |bytes| {
                SourceUnit::from_canonical_storage_bytes(
                    bytes,
                    decoded_source.source_id,
                    &descriptor,
                )
                .filter(|decoded| decoded.matches_canonical_storage(&decoded_source, &descriptor))
                .ok_or_else(|| BlobDecodeError::new("current source storage mismatch"))
            }),
        },
        &expected_bytes,
        &expected_tables,
    ) else {
        panic!("bound blob rehydration");
    };
    assert_eq!(adapter.successful_rehydrations(), 1);
    assert!(matches!(handle.placement(), StoragePlacement::Blob { .. }));
    assert_eq!(*target.storage().get(&handle).unwrap(), *source);
    assert_eq!(
        target.storage().side_tables(&handle).unwrap(),
        expected_tables
    );
    let snapshot = submission.session.captured().snapshot();
    let frontend_input = PhaseInput::new(
        unit,
        PhaseDispatchInputBundle::new(
            snapshot.id,
            source_key(&snapshot.source_versions[0]),
            vec![],
            vec![
                SealedParentOutputHandle::from_current_output(&target, snapshot.id, handle.erase())
                    .unwrap(),
            ],
        )
        .unwrap(),
    );
    allow_frontend(&target, &frontend_input.work_unit);
    assert_eq!(
        driver
            .registry()
            .cache_key_for_phase(PipelinePhase::Frontend, &frontend_input)
            .unwrap()
            .intent,
        PhaseCacheIntent::NoKey {
            reason: "frontend inputs require a live source and dependency run".into()
        }
    );
    let replay = driver
        .registry()
        .execute_phase_with_resources(
            PipelinePhase::Frontend,
            frontend_input,
            PhaseExecutionResources {
                diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                    DiagnosticPhase::Frontend,
                    snapshot.id,
                    "frontend.run_loaded",
                ))),
                output_publisher: Some(target.clone()),
                source_load: Some(SourceLoadInputs {
                    snapshot,
                    captured: Some(submission.session.captured()),
                    build_plan: submission.build_plan.as_ref().unwrap(),
                    module_index: submission.module_index.as_ref().unwrap(),
                    allocator: &ids,
                }),
                ..PhaseExecutionResources::default()
            },
        )
        .unwrap()
        .result;
    let clean = submission
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
                .any(|handle| handle.output_kind() == &OutputKind::new("FrontendOutput"))
        })
        .unwrap();
    assert_eq!(replay.status, PhaseStatus::Complete);
    assert_eq!(replay.diagnostics, clean.diagnostics);
    let replay_handle = target
        .storage()
        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
            &replay.output_refs[0],
            &OutputKind::new("FrontendOutput"),
        )
        .unwrap();
    let clean_handle = owner
        .storage()
        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
            &clean.output_refs[0],
            &OutputKind::new("FrontendOutput"),
        )
        .unwrap();
    assert_eq!(
        *target.storage().get(&replay_handle).unwrap(),
        *owner.storage().get(&clean_handle).unwrap()
    );
    assert_eq!(
        target.storage().side_tables(&replay_handle).unwrap(),
        owner.storage().side_tables(&clean_handle).unwrap()
    );
    assert_eq!(replay_handle.lineage().parents, vec![handle.output()]);
}
