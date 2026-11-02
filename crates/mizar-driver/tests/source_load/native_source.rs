use super::*;

#[test]
fn native_source_layout_feeds_protected_prefix_like_explicit_clean_layout() {
    for (threshold, workers, order) in [
        (usize::MAX, 1, CompletionOrder::Canonical),
        (0, 4, CompletionOrder::Reverse),
    ] {
        let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut request, input) = fixture.submit_request(vec![], vec![]);
        let plan = produce_build_plan(
            input.plan_request.clone(),
            input.workspace_packages.clone(),
            input.lockfile.clone(),
        )
        .unwrap();
        let native = StaticSourceLayout::from_native_disk(&plan).unwrap();
        assert_eq!(native, input.source_layout);
        request.source_inputs.versions.clear();
        let session = request
            .capture_sources(
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new("main"),
                        normalized_path: normalize_path(
                            &fixture.root.join("alpha"),
                            Path::new("src/main.miz"),
                        )
                        .unwrap(),
                        edition: Edition::new("2025"),
                        origin: SourceOriginInput::Disk {
                            path: "src/main.miz".into(),
                        },
                    },
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
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        let mut clean = None;
        for layout in [input.source_layout.clone(), native] {
            let (_, mut current) = fixture.submit_request(vec![], vec![]);
            current.source_layout = layout;
            current.worker_count = workers;
            current.completion_order = order;
            let owner = publisher(snapshot, threshold, &unit);
            allow_frontend(&owner, &unit);
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            let submission = driver
                .submit_captured(session.clone(), &ids, &snapshots, current)
                .unwrap();
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
            let results = submission
                .scheduler_run
                .as_ref()
                .unwrap()
                .phase_results
                .values()
                .flatten()
                .collect::<Vec<_>>();
            let refs = results
                .iter()
                .flat_map(|result| &result.output_refs)
                .collect::<Vec<_>>();
            assert_eq!(refs.len(), 2);
            assert!(results.iter().all(|result| result.diagnostics.is_empty()));
            let source_ref = *refs
                .iter()
                .find(|output| output.output_kind() == &OutputKind::new("SourceUnit"))
                .unwrap();
            let source = owner
                .storage()
                .get(
                    &owner
                        .storage()
                        .typed_handle::<SourceUnit>(source_ref, &OutputKind::new("SourceUnit"))
                        .unwrap(),
                )
                .unwrap();
            let frontend_ref = *refs
                .iter()
                .find(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
                .unwrap();
            let frontend = owner
                .storage()
                .get(
                    &owner
                        .storage()
                        .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                            frontend_ref,
                            &OutputKind::new("FrontendOutput"),
                        )
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(source.source_text.as_ref(), "definition\nend;\n");
            assert!(source.loading_map.is_some());
            assert_eq!(
                source.source_id,
                session.captured().snapshot().source_versions[0].source_id
            );
            for output in &refs {
                assert_eq!(
                    matches!(output.placement(), StoragePlacement::Blob { .. }),
                    threshold == 0
                );
            }
            let phase_input = PhaseInput::new(
                unit.clone(),
                PhaseDispatchInputBundle::without_parent_outputs(
                    snapshot,
                    SourceUnitCacheKey::from_source(&source).stable_hash(),
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
                        .cache_key_for_phase(phase, &phase_input)
                        .unwrap()
                        .intent,
                    PhaseCacheIntent::NoKey {
                        reason: reason.into()
                    }
                );
            }
            let actual = (
                submission.module_index.unwrap(),
                source.as_ref().clone(),
                frontend.as_ref().clone(),
            );
            if let Some(expected) = &clean {
                assert_eq!(&actual, expected);
            } else {
                clean = Some(actual);
            }
        }
    }
}

#[test]
fn native_source_layout_drives_nested_cross_package_import_discovery() {
    let fixture = Fixture::new(b"definition\nend;\n");
    for (path, bytes) in [
        ("app/src/main.miz", b"import core.groups.base;\ntheorem T: a combine b = a;\n".as_slice()),
        ("core/src/groups/base.miz", b"definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n".as_slice()),
    ] {
        fs::create_dir_all(fixture.root.join(path).parent().unwrap()).unwrap();
        fs::write(fixture.root.join(path), bytes).unwrap();
    }
    fs::write(fixture.root.join("core/src/ignored.MIZ"), [0xff]).unwrap();
    fs::write(fixture.root.join("core/src/notes.txt"), [0xff]).unwrap();
    let (mut request, mut input) = fixture.submit_request(vec![], vec![]);
    input.workspace_packages = vec![
        WorkspacePackage { member_path: "app".into(), manifest: parse_package_manifest("[package]\nname = \"app\"\nversion = \"0.1.0\"\n[dependencies]\ncore = \"1.0.0\"\n").unwrap() },
        WorkspacePackage { member_path: "core".into(), manifest: parse_package_manifest("[package]\nname = \"core\"\nversion = \"1.0.0\"\n").unwrap() },
    ];
    input.lockfile = parse_lockfile("schema_version = 1\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"app\" }\ndependencies = [{ name = \"core\", version = \"1.0.0\" }]\n[[package]]\nname = \"core\"\nversion = \"1.0.0\"\nsource = { kind = \"workspace\", path = \"core\" }\ndependencies = []\n").unwrap();
    let plan = produce_build_plan(
        input.plan_request.clone(),
        input.workspace_packages.clone(),
        input.lockfile.clone(),
    )
    .unwrap();
    let native = StaticSourceLayout::from_native_disk(&plan).unwrap();
    let explicit = StaticSourceLayout::new(vec![
        WorkspaceSourcePackage {
            package_id: PackageId::new("app"),
            files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
        },
        WorkspaceSourcePackage {
            package_id: PackageId::new("core"),
            files: vec![WorkspaceSourceFile::new(
                "src/groups/base.miz",
                "groups/base.miz",
            )],
        },
    ]);
    assert_eq!(native, explicit);
    assert_eq!(
        build_module_index(&plan, &native, &[]).unwrap(),
        build_module_index(&plan, &explicit, &[]).unwrap()
    );
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    request.source_inputs.versions.clear();
    let descriptors = [
        ("app", "main", "src/main.miz"),
        ("core", "groups.base", "src/groups/base.miz"),
    ]
    .into_iter()
    .map(|(package, module, path)| {
        let root = fixture.root.join(package);
        (
            DiskSourceLoader::new(&root),
            SourceInput {
                package_id: PackageId::new(package),
                module_path: ModulePath::new(module),
                normalized_path: normalize_path(&root, Path::new(path)).unwrap(),
                edition: Edition::new("2025"),
                origin: SourceOriginInput::Disk { path: path.into() },
            },
        )
    })
    .collect();
    let session = request
        .capture_sources(&ids, &snapshots, descriptors)
        .unwrap();
    let snapshot = session.captured().snapshot().id;
    let mut clean = None;
    for (layout, workers, order) in [
        (explicit, 1, CompletionOrder::Canonical),
        (native, 4, CompletionOrder::Reverse),
    ] {
        let mut current = DriverSubmitInput::new(
            input.plan_request.clone(),
            input.workspace_packages.clone(),
            input.lockfile.clone(),
            layout,
        );
        current.worker_count = workers;
        current.completion_order = order;
        let main = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("app"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(snapshot, 0, &main);
        for (package, module) in [("app", "main"), ("core", "groups.base")] {
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new(package),
                    ModulePath::new(module),
                ),
            };
            owner.allow_work_unit(AllowedWorkUnit::new(
                IrPipelinePhase::new("SourceLoad"),
                OutputKind::new("SourceUnit"),
                IrWorkUnit::new(format!("{package:?}:{module:?}")),
            ));
            allow_frontend(&owner, &unit);
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
        let submission = driver
            .submit_captured_with_import_discovery(session.clone(), &ids, &snapshots, current)
            .unwrap();
        let graph = submission.task_graph.as_ref().unwrap();
        let run = submission.scheduler_run.as_ref().unwrap();
        let main_task = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::Frontend && task.unit == main)
            .unwrap();
        let leaf_task = graph
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::Frontend && task.unit != main)
            .unwrap();
        assert!(
            graph
                .edges()
                .iter()
                .any(|edge| edge.dependent == main_task.id && edge.dependency == leaf_task.id)
        );
        let result = &run.phase_results[&main_task.id][0];
        assert_eq!(result.status, PhaseStatus::Complete);
        assert!(result.diagnostics.is_empty());
        let output = owner
            .storage()
            .get(
                &owner
                    .storage()
                    .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                        &result.output_refs[0],
                        &OutputKind::new("FrontendOutput"),
                    )
                    .unwrap(),
            )
            .unwrap();
        assert!(
            output.tokens.tokens().iter().any(
                |token| token.text.as_ref() == "combine" && token.kind == TokenKind::UserSymbol
            )
        );
        assert_eq!(result.output_refs[0].lineage().parents.len(), 2);
        let actual = (submission.module_index.unwrap(), output.as_ref().clone());
        if let Some(expected) = &clean {
            assert_eq!(&actual, expected);
        } else {
            clean = Some(actual);
        }
    }
}

#[test]
fn native_source_layout_is_metadata_only_and_rejects_incomplete_trees() {
    use mizar_build::module_index::{ModuleIndexDiagnosticKind, SourceLayoutProvider};
    use mizar_build::planner::PackagePlanSource;
    let fixture = Fixture::new(b"definition\nend;\n");
    let (mut request, input) = fixture.submit_request(vec![], vec![]);
    let plan = produce_build_plan(
        input.plan_request.clone(),
        input.workspace_packages.clone(),
        input.lockfile.clone(),
    )
    .unwrap();
    fs::create_dir(fixture.root.join("alpha/src/a")).unwrap();
    for path in ["alpha/src/z.miz", "alpha/src/a/leaf.miz"] {
        fs::write(fixture.root.join(path), []).unwrap();
    }
    fixture.write(&[0xff]);
    let layout = StaticSourceLayout::from_native_disk(&plan).unwrap();
    assert_eq!(
        layout,
        StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![
                WorkspaceSourceFile::new("src/a/leaf.miz", "a/leaf.miz"),
                WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                WorkspaceSourceFile::new("src/z.miz", "z.miz"),
            ],
        }])
    );
    assert!(build_module_index(&plan, &layout, &[]).is_ok());
    let single = produce_build_plan(
        PlanRequest {
            workspace_root: WorkspaceRoot::new(fixture.root.join("alpha").to_str().unwrap()),
            ..input.plan_request.clone()
        },
        vec![WorkspacePackage {
            member_path: ".".into(),
            manifest: parse_package_manifest("[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n[dependencies]\ndep = \"1.0.0\"\n").unwrap(),
        }],
        parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \".\" }\ndependencies = [{ name = \"dep\", version = \"1.0.0\" }]\n[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:dep\" }\ndependencies = []\n").unwrap(),
    ).unwrap();
    assert_eq!(
        StaticSourceLayout::from_native_disk(&single).unwrap(),
        layout
    );
    let registry = single
        .packages
        .iter()
        .find(|package| matches!(package.source, PackagePlanSource::Registry { .. }))
        .unwrap();
    assert!(
        StaticSourceLayout::from_native_disk(&single)
            .unwrap()
            .workspace_source_files(registry)
            .is_err()
    );
    request.source_inputs.versions.clear();
    assert!(
        request
            .capture_sources(
                &InMemorySessionIdAllocator::new(),
                &SnapshotRegistry::new(),
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    SourceInput {
                        package_id: PackageId::new("alpha"),
                        module_path: ModulePath::new("main"),
                        normalized_path: normalize_path(
                            &fixture.root.join("alpha"),
                            Path::new("src/main.miz")
                        )
                        .unwrap(),
                        edition: Edition::new("2025"),
                        origin: SourceOriginInput::Disk {
                            path: "src/main.miz".into()
                        },
                    },
                )]
            )
            .is_err()
    );
    fs::remove_file(fixture.root.join("alpha/src/z.miz")).unwrap();
    fs::remove_dir_all(fixture.root.join("alpha/src/a")).unwrap();
    for (root, source_root) in [
        ("../alpha", "../alpha/src"),
        ("alpha/.", "alpha/./src"),
        ("/alpha", "/alpha/src"),
        ("alpha", "wrong/src"),
    ] {
        let mut invalid = plan.clone();
        let PackagePlanSource::Workspace {
            root: actual_root,
            source_root: actual_source,
            ..
        } = &mut invalid.packages[0].source
        else {
            unreachable!()
        };
        *actual_root = root.into();
        *actual_source = source_root.into();
        assert!(
            StaticSourceLayout::from_native_disk(&invalid).is_err(),
            "{root}"
        );
    }
    let mut invalid = plan.clone();
    invalid.packages.push(plan.packages[0].clone());
    assert!(StaticSourceLayout::from_native_disk(&invalid).is_err());
    invalid = plan.clone();
    let mut later = plan.packages[0].clone();
    later.package_id = PackageId::new("later");
    later.source = PackagePlanSource::Workspace {
        root: "later".into(),
        source_root: "later/src".into(),
        manifest_path: "later/mizar.pkg".into(),
    };
    invalid.packages.push(later);
    fs::create_dir_all(fixture.root.join("later/src")).unwrap();
    fs::write(fixture.root.join("later/src/bad-name.miz"), []).unwrap();
    assert!(StaticSourceLayout::from_native_disk(&invalid).is_err());
    invalid = plan.clone();
    invalid.workspace_root = WorkspaceRoot::new(fixture.root.join("absent").to_str().unwrap());
    assert!(StaticSourceLayout::from_native_disk(&invalid).is_err());
    invalid.workspace_root =
        WorkspaceRoot::new(fixture.root.join("alpha/src/main.miz").to_str().unwrap());
    assert!(StaticSourceLayout::from_native_disk(&invalid).is_err());
    fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
    assert!(
        StaticSourceLayout::from_native_disk(&plan)
            .unwrap()
            .workspace_source_files(&plan.packages[0])
            .unwrap()
            .is_empty()
    );
    fs::remove_dir(fixture.root.join("alpha/src")).unwrap();
    assert!(StaticSourceLayout::from_native_disk(&plan).is_err());
    fs::create_dir(fixture.root.join("alpha/src")).unwrap();
    fixture.write(b"definition\nend;\n");
    for filename in ["bad-name.miz", "definition.miz", "nested.name.miz", "é.miz"] {
        fs::write(fixture.root.join("alpha/src").join(filename), []).unwrap();
        assert!(
            StaticSourceLayout::from_native_disk(&plan).is_err(),
            "{filename}"
        );
        fs::remove_file(fixture.root.join("alpha/src").join(filename)).unwrap();
    }
    let malformed = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
        package_id: PackageId::new("alpha"),
        files: vec![WorkspaceSourceFile::new("src/bad-name.miz", "bad-name.miz")],
    }]);
    assert!(
        build_module_index(&plan, &malformed, &[])
            .unwrap_err()
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.kind == ModuleIndexDiagnosticKind::InvalidModuleComponent)
    );
    #[cfg(unix)]
    {
        use std::os::unix::{ffi::OsStringExt, fs::symlink};
        for target in ["main.miz", "absent.miz", "."] {
            let link = fixture.root.join("alpha/src/link");
            symlink(target, &link).unwrap();
            assert!(
                StaticSourceLayout::from_native_disk(&plan).is_err(),
                "{target}"
            );
            fs::remove_file(link).unwrap();
        }
        for directory in [false, true] {
            let path = fixture
                .root
                .join("alpha/src")
                .join(std::ffi::OsString::from_vec(if directory {
                    vec![0xff]
                } else {
                    vec![0xff, b'.', b'm', b'i', b'z']
                }));
            if directory {
                fs::create_dir(&path).unwrap();
            } else {
                fs::write(&path, []).unwrap();
            }
            assert!(StaticSourceLayout::from_native_disk(&plan).is_err());
            if directory {
                fs::remove_dir(path).unwrap();
            } else {
                fs::remove_file(path).unwrap();
            }
        }
        let fifo_path = fixture.root.join("alpha/src/fifo");
        assert!(
            std::process::Command::new("mkfifo")
                .arg(&fifo_path)
                .status()
                .unwrap()
                .success()
        );
        assert!(StaticSourceLayout::from_native_disk(&plan).is_err());
        fs::remove_file(fifo_path).unwrap();
        fs::rename(
            fixture.root.join("alpha/src"),
            fixture.root.join("real_src"),
        )
        .unwrap();
        symlink(
            fixture.root.join("real_src"),
            fixture.root.join("alpha/src"),
        )
        .unwrap();
        assert!(StaticSourceLayout::from_native_disk(&plan).is_err());
        fs::remove_file(fixture.root.join("alpha/src")).unwrap();
        fs::rename(
            fixture.root.join("real_src"),
            fixture.root.join("alpha/src"),
        )
        .unwrap();
        fs::rename(
            fixture.root.join("alpha"),
            fixture.root.join("real_package"),
        )
        .unwrap();
        symlink(
            fixture.root.join("real_package"),
            fixture.root.join("alpha"),
        )
        .unwrap();
        assert!(StaticSourceLayout::from_native_disk(&plan).is_err());
        fs::remove_file(fixture.root.join("alpha")).unwrap();
        fs::rename(
            fixture.root.join("real_package"),
            fixture.root.join("alpha"),
        )
        .unwrap();
        let link = fixture.root.join("workspace_link");
        symlink(&fixture.root, &link).unwrap();
        let mut invalid = plan.clone();
        for suffix in ["", "/", "/."] {
            invalid.workspace_root = WorkspaceRoot::new(format!("{}{suffix}", link.display()));
            assert!(
                StaticSourceLayout::from_native_disk(&invalid).is_err(),
                "{suffix}"
            );
        }
        fs::remove_file(link).unwrap();
    }
    assert!(StaticSourceLayout::from_native_disk(&plan).is_ok());
}

#[test]
fn native_source_ir_lifetime_published_owners_roots_and_obsolete_reads() {
    use mizar_ir::{
        cache_adapter::{EncodeCacheRecordOutcome, IrCacheAdapter},
        storage::{CollectInput, CollectionSummary, RetainOwner, StorageError},
    };
    for threshold in [usize::MAX, 0] {
        let fixture = Fixture::new("\u{feff}definition\r\nend;\r\n".as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (_, submission, owner, _, handle) =
            native_disk_record_submission(&fixture, &ids, &snapshots, threshold);
        let (inputs, input) = native_disk_lookup_context(&submission, &ids);
        let storage = owner.storage();
        let source = storage.get(&handle).unwrap();
        let maps = storage.side_tables(&handle).unwrap();
        let adapter = IrCacheAdapter::new(owner.clone());
        let policy = mizar_proof::policy::VerifierPolicy::release();
        assert_eq!(
            matches!(handle.placement(), StoragePlacement::Blob { .. }),
            threshold == 0
        );
        let readers = [RetainOwner::new("diagnostics"), RetainOwner::new("writer")];
        for reader in [&readers[0], &readers[0], &readers[1]] {
            storage.retain(handle.output(), reader.clone()).unwrap();
        }
        let collect = CollectInput {
            snapshot: input.snapshot,
            protected_outputs: vec![],
        };
        let retained = CollectionSummary {
            retained_outputs: 1,
            ..CollectionSummary::default()
        };
        // The genuine fixture also published one unprotected Frontend output.
        let first = storage.collect(collect.clone());
        assert_eq!(first.outputs_dropped, 1);
        assert_eq!(first.retained_outputs, 1);
        assert_eq!(first.blobs_dropped, usize::from(threshold == 0));
        assert_eq!(*storage.get(&handle).unwrap(), *source);
        assert_eq!(storage.side_tables(&handle).unwrap(), maps);
        assert_eq!(
            storage
                .typed_handle::<SourceUnit>(handle.any(), &OutputKind::new("SourceUnit"))
                .unwrap(),
            handle
        );
        assert!(matches!(
            inputs
                .encode_native_disk_cache_record(&input, &policy, &adapter, handle.clone())
                .unwrap(),
            EncodeCacheRecordOutcome::Encoded(_)
        ));
        assert!(storage.release(handle.output(), &readers[0]).unwrap());
        assert!(!storage.release(handle.output(), &readers[0]).unwrap());
        assert_eq!(storage.collect(collect.clone()), retained);
        owner.mark_obsolete(input.snapshot).unwrap();
        assert_eq!(storage.collect(collect.clone()), retained);
        assert_eq!(*storage.get(&handle).unwrap(), *source);
        assert_eq!(storage.side_tables(&handle).unwrap(), maps);
        assert!(
            inputs
                .encode_native_disk_cache_record(&input, &policy, &adapter, handle.clone())
                .is_err()
        );
        assert!(
            SealedParentOutputHandle::from_current_output(&owner, input.snapshot, handle.erase())
                .is_err()
        );
        assert!(storage.release(handle.output(), &readers[1]).unwrap());
        let protected = storage.collect(CollectInput {
            protected_outputs: vec![handle.output()],
            ..collect.clone()
        });
        assert_eq!(
            protected,
            CollectionSummary {
                protected_outputs: 1,
                ..CollectionSummary::default()
            }
        );
        assert_eq!(*storage.get(&handle).unwrap(), *source);
        assert_eq!(storage.side_tables(&handle).unwrap(), maps);
        assert_eq!(
            storage.collect(collect.clone()),
            CollectionSummary {
                outputs_dropped: 1,
                blobs_dropped: usize::from(threshold == 0),
                ..CollectionSummary::default()
            }
        );
        assert!(matches!(
            storage.get(&handle),
            Err(StorageError::CollectedOutput { output }) if output == handle.output()
        ));
        assert!(matches!(
            storage.side_tables(&handle),
            Err(StorageError::CollectedOutput { output }) if output == handle.output()
        ));
        assert_eq!(storage.collect(collect), CollectionSummary::default());
    }
}

#[test]
fn native_source_ir_lifetime_reconstructed_owners_keep_real_frontend_inputs() {
    use mizar_cache::cache_store::{CacheInsertOutcome, CacheStoreRoot};
    use mizar_ir::{
        cache_adapter::{CacheRehydrateOutcome, EncodeCacheRecordOutcome, IrCacheAdapter},
        storage::{CollectInput, CollectionSummary, RetainOwner, StorageError},
    };
    for threshold in [usize::MAX, 0] {
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
            native_disk_record_submission(&fixtures[0], &ids, &old_snapshots, threshold);
        let (driver, submission, owner, _, clean_source) =
            native_disk_record_submission(&fixtures[1], &ids, &current_snapshots, threshold);
        let (old_inputs, old_input) = native_disk_lookup_context(&old_submission, &ids);
        let (inputs, input) = native_disk_lookup_context(&submission, &ids);
        let policy = mizar_proof::policy::VerifierPolicy::release();
        let EncodeCacheRecordOutcome::Encoded(record) = old_inputs
            .encode_native_disk_cache_record(
                &old_input,
                &policy,
                &IrCacheAdapter::new(old_owner.clone()),
                old_handle,
            )
            .unwrap()
        else {
            panic!("genuine native producer");
        };
        assert_ne!(old_input.snapshot, input.snapshot);
        let store = CacheStoreRoot::new(fixtures[1].root.join("cache"));
        assert_eq!(store.insert(&record).unwrap(), CacheInsertOutcome::Inserted);
        let target = publisher(input.snapshot, threshold, &input.work_unit);
        let adapter = IrCacheAdapter::new(target.clone());
        let CacheRehydrateOutcome::Rehydrated(handle) = inputs
            .rehydrate_native_disk_cache(&input, &policy, &store, &adapter)
            .unwrap()
        else {
            panic!("current native reconstruction");
        };
        let handle = *handle;
        assert_eq!(handle.snapshot(), input.snapshot);
        assert_eq!(handle.placement(), clean_source.placement());
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
                    .any(|output| output.output_kind() == &OutputKind::new("FrontendOutput"))
            })
            .unwrap();
        let clean_handle = owner
            .storage()
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                &clean.output_refs[0],
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        assert_eq!(
            owner.storage().collect(CollectInput {
                snapshot: input.snapshot,
                protected_outputs: vec![clean_source.output(), clean_handle.output()],
            }),
            CollectionSummary {
                protected_outputs: 2,
                ..CollectionSummary::default()
            }
        );
        let storage = target.storage();
        let readers = [RetainOwner::new("frontend"), RetainOwner::new("observer")];
        for reader in [&readers[0], &readers[0], &readers[1]] {
            storage.retain(handle.output(), reader.clone()).unwrap();
        }
        let collect = CollectInput {
            snapshot: input.snapshot,
            protected_outputs: vec![],
        };
        let retained = CollectionSummary {
            retained_outputs: 1,
            ..CollectionSummary::default()
        };
        assert_eq!(storage.collect(collect.clone()), retained);
        assert!(storage.release(handle.output(), &readers[0]).unwrap());
        assert!(!storage.release(handle.output(), &readers[0]).unwrap());
        assert_eq!(storage.collect(collect.clone()), retained);
        assert_eq!(
            *storage.get(&handle).unwrap(),
            *owner.storage().get(&clean_source).unwrap()
        );
        assert_eq!(
            storage.side_tables(&handle).unwrap(),
            owner.storage().side_tables(&clean_source).unwrap()
        );
        assert_eq!(
            storage
                .typed_handle::<SourceUnit>(handle.any(), &OutputKind::new("SourceUnit"))
                .unwrap(),
            handle
        );
        assert!(matches!(
            inputs
                .encode_native_disk_cache_record(&input, &policy, &adapter, handle.clone())
                .unwrap(),
            EncodeCacheRecordOutcome::Encoded(_)
        ));
        let frontend_input = PhaseInput::new(
            input.work_unit.clone(),
            PhaseDispatchInputBundle::new(
                input.snapshot,
                input.identities().input_hash(),
                vec![],
                vec![
                    SealedParentOutputHandle::from_current_output(
                        &target,
                        input.snapshot,
                        handle.erase(),
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        );
        allow_frontend(&target, &input.work_unit);
        let replay = driver
            .registry()
            .execute_phase_with_resources(
                PipelinePhase::Frontend,
                frontend_input,
                PhaseExecutionResources {
                    diagnostics: Some(DiagnosticSink::new(DiagnosticProducerScope::new(
                        DiagnosticPhase::Frontend,
                        input.snapshot,
                        "frontend.run_loaded",
                    ))),
                    output_publisher: Some(target.clone()),
                    source_load: Some(inputs),
                    ..PhaseExecutionResources::default()
                },
            )
            .unwrap()
            .result;
        assert_eq!(replay.status, PhaseStatus::Complete);
        assert_eq!(replay.diagnostics, clean.diagnostics);
        let replay_handle = storage
            .typed_handle::<FrontendOutput<<MizarParserSeam as ParserSeam>::Ast>>(
                &replay.output_refs[0],
                &OutputKind::new("FrontendOutput"),
            )
            .unwrap();
        assert_eq!(
            *storage.get(&replay_handle).unwrap(),
            *owner.storage().get(&clean_handle).unwrap()
        );
        assert_eq!(
            storage.side_tables(&replay_handle).unwrap(),
            owner.storage().side_tables(&clean_handle).unwrap()
        );
        assert_eq!(replay_handle.lineage().parents, vec![handle.output()]);
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
            assert!(matches!(
                driver
                    .registry()
                    .cache_key_for_phase(phase, &input)
                    .unwrap()
                    .intent,
                PhaseCacheIntent::NoKey { .. }
            ));
        }
        assert!(storage.release(handle.output(), &readers[1]).unwrap());
        let rooted = CollectInput {
            protected_outputs: vec![handle.output(), replay_handle.output()],
            ..collect.clone()
        };
        let protected = CollectionSummary {
            protected_outputs: 2,
            ..CollectionSummary::default()
        };
        assert_eq!(storage.collect(rooted.clone()), protected);
        assert_eq!(storage.collect(rooted), protected);
        assert_eq!(
            *storage.get(&handle).unwrap(),
            *owner.storage().get(&clean_source).unwrap()
        );
        assert_eq!(
            storage.side_tables(&handle).unwrap(),
            owner.storage().side_tables(&clean_source).unwrap()
        );
        // A rooted child does not implicitly retain its lineage parent.
        assert_eq!(
            storage.collect(CollectInput {
                protected_outputs: vec![replay_handle.output()],
                ..collect.clone()
            }),
            CollectionSummary {
                outputs_dropped: 1,
                blobs_dropped: usize::from(threshold == 0),
                protected_outputs: 1,
                ..CollectionSummary::default()
            }
        );
        assert!(
            matches!(storage.get(&handle), Err(StorageError::CollectedOutput { output }) if output == handle.output())
        );
        assert!(
            matches!(storage.side_tables(&handle), Err(StorageError::CollectedOutput { output }) if output == handle.output())
        );
        assert!(
            target
                .validate_current_output(input.snapshot, handle.any())
                .is_ok()
        );
        assert!(
            inputs
                .encode_native_disk_cache_record(&input, &policy, &adapter, handle.clone())
                .is_err()
        );
        assert!(
            SealedParentOutputHandle::from_current_output(&target, input.snapshot, handle.erase())
                .is_err()
        );
        assert_eq!(
            storage.collect(collect.clone()),
            CollectionSummary {
                outputs_dropped: 1,
                blobs_dropped: usize::from(threshold == 0),
                ..CollectionSummary::default()
            }
        );
        assert_eq!(storage.collect(collect), CollectionSummary::default());
    }
}
