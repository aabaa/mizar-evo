use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use mizar_build::{
    cancel::{CancellationGeneration, CancellationReason, CancellationToken},
    module_index::{
        DependencyArtifactIndex, DependencyModuleSummaryRef, ModuleIndex, ModuleIndexLocation,
        StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage, build_module_index,
    },
    planner::{
        BuildPlan, DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile,
        parse_package_manifest, produce_build_plan,
    },
    task_graph::{ModuleDependencyOverlay, PipelinePhase, TaskKind, WorkUnit},
};
use mizar_diagnostics::{
    failure_record::PipelinePhase as DiagnosticPhase,
    sink::{DiagnosticProducerScope, DiagnosticSink},
};
use mizar_driver::{
    driver::{CompilerDriver, DriverSubmissionStatus, DriverSubmitInput},
    registry::{
        PhaseCacheIntent, PhaseExecutionResources, PhaseInput, PhaseRegistryBuilder, PhaseStatus,
        SourceLoadInputs,
    },
    request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet,
        SourceInputSet, VerifierConfigInput,
    },
};
use mizar_frontend::{
    cache_key::{SOURCE_UNIT_CACHE_KEY_VERSION, SourceUnitCacheKey},
    lexical_env::{
        FrontendLexicalEnvironmentError, LexicalEnvironmentRequest, LexicalSummaryProvider,
        build_active_lexical_environment,
    },
    preprocess::preprocess,
    source::SourceUnit,
    source::register_source_unit,
    span_bridge::SpanBridge,
};
use mizar_ir::{
    dispatch_input::PhaseDispatchInputBundle,
    identity::{
        OutputKind, PipelinePhase as IrPipelinePhase, SnapshotHandleRegistry,
        WorkUnit as IrWorkUnit,
    },
    publisher::{AllowedWorkUnit, PhaseOutputPublisher},
    storage::{IrStorageService, StoragePlacement, StoragePolicy},
};
use mizar_session::{
    BuildRequestId, BuildSessionId, BuildSnapshot, BuildSnapshotId, DiskSourceLoader, Edition,
    Hash, IdError, InMemorySessionIdAllocator, LineMap, LoadingMap, ModulePath, PackageId,
    SessionIdAllocator, SnapshotLeaseId, SnapshotRegistry, SourceId, SourceInput, SourceMapId,
    SourceOrigin, SourceOriginInput, SourceVersion, ToolchainInfo, WorkspaceRoot, normalize_path,
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
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        let registry = builder.build().unwrap();
        let mut driver = CompilerDriver::new(registry);
        let mut input = DriverSubmitInput::new(
            PlanRequest {
                workspace_root: WorkspaceRoot::new(self.root.to_string_lossy().into_owned()),
                dependency_selection: DependencySelection::Normal,
                toolchain: ToolchainInfo::new("mizar-evo-test"),
            },
            vec![WorkspacePackage {
        member_path: "alpha".to_owned(),
        manifest: parse_package_manifest("[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n")
            .unwrap(),
    }],
            parse_lockfile(
                "schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n",
            )
            .unwrap(),
            StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
            }]),
        );
        input.dependency_overlay = ModuleDependencyOverlay::complete(Vec::new());
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
                Vec::new(),
                hash(1),
                ToolchainInfo::new("mizar-evo-test"),
            ),
            verifier_config: VerifierConfigInput::new(hash(2)),
        };
        let submission = driver.submit(request, ids, snapshots, input).unwrap();
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        submission
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
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
    let snapshot = submission.session.captured.snapshot.id;
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
                    snapshot: &submission.session.captured.snapshot,
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
    let snapshot = submission.session.captured.snapshot.id;
    let source_load = SourceLoadInputs {
        snapshot: &submission.session.captured.snapshot,
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
    let base_snapshot = submission.session.captured.snapshot.clone();
    let plan = submission.build_plan.as_ref().unwrap();
    let index = submission.module_index.as_ref().unwrap();
    let module_unit = source_task(&submission).unit.clone();
    let run = |snapshot: &BuildSnapshot, plan: &BuildPlan, index: &ModuleIndex, unit: WorkUnit| {
        binding_result(
            &submission,
            SourceLoadInputs {
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
    let snapshot = submission.session.captured.snapshot.clone();
    let index = submission.module_index.as_ref().unwrap();
    let mut plan = submission.build_plan.as_ref().unwrap().clone();
    plan.workspace_root = WorkspaceRoot::new("other-workspace");
    let module_unit = source_task(&submission).unit.clone();
    assert_eq!(
        binding_result(
            &submission,
            SourceLoadInputs {
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
    let snapshot = submission.session.captured.snapshot.id;
    let source_load = SourceLoadInputs {
        snapshot: &submission.session.captured.snapshot,
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
        snapshot: &submission.session.captured.snapshot,
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
            module_index: &mismatched_index,
            ..inputs
        }
        .dependency_lexical_provider(&roots)
        .resolve_imports(&request),
        Err(FrontendLexicalEnvironmentError::ProviderUnavailable { .. })
    ));
    let mut duplicate_snapshot = submission.session.captured.snapshot.clone();
    duplicate_snapshot
        .source_versions
        .push(fixture.version.clone());
    let duplicate_inputs = SourceLoadInputs {
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

fn execute(
    submission: &mizar_driver::driver::BuildSubmission,
    ids: &InMemorySessionIdAllocator,
    threshold: usize,
) -> (
    mizar_driver::registry::PhaseResult,
    std::sync::Arc<PhaseOutputPublisher>,
) {
    let task = source_task(submission);
    let snapshot = submission.session.captured.snapshot.id;
    // Force the loader to allocate a different ID from the captured source.
    ids.next_source_id(snapshot).unwrap();
    let publisher = publisher(snapshot, threshold, &task.unit);
    let source_load = SourceLoadInputs {
        snapshot: &submission.session.captured.snapshot,
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
            source_key(&submission.session.request.source_inputs.versions[0]),
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

fn binding_result(
    submission: &mizar_driver::driver::BuildSubmission,
    source_load: SourceLoadInputs<'_>,
    unit: WorkUnit,
) -> PhaseStatus {
    let snapshot = submission.session.captured.snapshot.id;
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
                    source_key(&submission.session.request.source_inputs.versions[0]),
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
