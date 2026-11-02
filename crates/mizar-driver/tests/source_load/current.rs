use super::*;

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
fn current_disk_diagnostic_index_preserves_original_native_runs_and_retained_sources() {
    use mizar_diagnostics::{
        aggregator::BuildDiagnosticIndex, failure_record::DiagnosticFreshness,
    };
    for text in [
        "definition\nend;\n",
        "end",
        "import mml.no_such;\ndefinition\nend;\n",
    ] {
        let fixture = Fixture::new(text.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, _) = fixture.submit_request(vec![], vec![]);
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
        let snapshot = session.captured().snapshot().id;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        fs::write(
            fixture.root.join("alpha/src/main.miz"),
            b"changed after capture",
        )
        .unwrap();
        fs::remove_file(fixture.root.join("alpha/src/main.miz")).unwrap();
        let mut replay = None;
        for workers in [1, 4] {
            let mut original = None;
            for indexed in [false, true] {
                let owner = publisher(snapshot, 0, &unit);
                allow_frontend(&owner, &unit);
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_load();
                builder.register_frontend(vec![]);
                let mut driver = CompilerDriver::new(builder.build().unwrap())
                    .with_output_publisher(owner.clone());
                let (_, mut input) = fixture.submit_request(vec![], vec![]);
                input.worker_count = workers;
                input.completion_order = if workers == 1 {
                    CompletionOrder::Canonical
                } else {
                    CompletionOrder::Reverse
                };
                let (submission, index) = if indexed {
                    driver
                        .submit_captured_with_diagnostic_index(
                            session.clone(),
                            &ids,
                            &snapshots,
                            input,
                        )
                        .unwrap()
                } else {
                    (
                        driver
                            .submit_captured(session.clone(), &ids, &snapshots, input)
                            .unwrap(),
                        None,
                    )
                };
                let run = submission.scheduler_run.as_ref().unwrap();
                let source = source_task(&submission);
                let source_result = &run.phase_results[&source.id][0];
                assert_eq!(source_result.status, PhaseStatus::Complete);
                let handle = owner
                    .storage()
                    .typed_handle::<SourceUnit>(
                        &source_result.output_refs[0],
                        &OutputKind::new("SourceUnit"),
                    )
                    .unwrap();
                let loaded = owner.storage().get(&handle).unwrap();
                assert_eq!(loaded.source_text.as_ref(), text);
                assert_eq!(
                    loaded.source_id,
                    session.captured().snapshot().source_versions[0].source_id
                );
                let (_, phase_input) = native_disk_lookup_context(&submission, &ids);
                for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
                    assert!(matches!(
                        driver
                            .registry()
                            .cache_key_for_phase(phase, &phase_input)
                            .unwrap()
                            .intent,
                        PhaseCacheIntent::NoKey { .. }
                    ));
                }
                assert!(!submission.missing_services.is_empty());
                let frontend = submission
                    .task_graph
                    .as_ref()
                    .unwrap()
                    .tasks()
                    .iter()
                    .find(|task| task.kind == TaskKind::Frontend)
                    .unwrap();
                assert_eq!(
                    run.phase_results[&frontend.id][0].status,
                    if text == "end" {
                        PhaseStatus::Fatal
                    } else if text.starts_with("import") {
                        PhaseStatus::Recoverable
                    } else {
                        PhaseStatus::Complete
                    }
                );
                assert_eq!(
                    submission.session.state(),
                    BuildSessionState::Finished(if text == "definition\nend;\n" {
                        BuildSessionOutcome::Blocked
                    } else {
                        BuildSessionOutcome::Failed
                    })
                );
                if indexed {
                    assert_eq!(&submission, original.as_ref().unwrap());
                    assert_eq!(
                        driver.validate_current_disk_diagnostics(session.id(), &snapshots),
                        Ok(())
                    );
                    let index = index.unwrap();
                    let batches = run
                        .phase_results
                        .values()
                        .flatten()
                        .flat_map(|result| result.diagnostics.clone())
                        .collect();
                    assert_eq!(
                        index,
                        BuildDiagnosticIndex::from_batches(snapshot, batches).unwrap()
                    );
                    assert_eq!(index.snapshot(), snapshot);
                    assert!(index.obsolete_drafts().is_empty());
                    assert_eq!(index.is_empty(), text == "definition\nend;\n");
                    let codes = index
                        .records()
                        .iter()
                        .map(|record| record.code().number())
                        .collect::<Vec<_>>();
                    if text == "end" {
                        assert_eq!(codes, vec![33, 52]);
                    }
                    if text.starts_with("import") {
                        assert!(codes.contains(&22));
                        assert!(codes.contains(&220));
                    }
                    for record in index.records() {
                        assert_eq!(record.handle().snapshot, snapshot);
                        assert_eq!(index.record_by_id(record.handle().id), Some(record));
                        assert_eq!(
                            record.freshness(),
                            &DiagnosticFreshness::Current {
                                source_snapshot: snapshot
                            }
                        );
                        assert_eq!(
                            record.phase(),
                            if record.code().number() >= 220 {
                                DiagnosticPhase::Resolver
                            } else {
                                DiagnosticPhase::Frontend
                            }
                        );
                        let DiagnosticPrimaryLocation::Span(span) = record.primary_location()
                        else {
                            panic!("real loaded-source span")
                        };
                        assert_eq!(span.range().source_id, loaded.source_id);
                        assert!(span.range().end <= text.len());
                        assert!(
                            index
                                .handles_for_source(loaded.source_id)
                                .unwrap()
                                .contains(&record.handle())
                        );
                    }
                    if let Some(prior) = &replay {
                        assert_eq!(&index, prior);
                    } else {
                        replay = Some(index);
                    }
                } else {
                    original = Some(submission);
                }
            }
        }
    }
}

#[test]
fn current_disk_diagnostic_guard_reads_live_driver_lane_request_and_publisher() {
    for mode in [
        "lane",
        "request",
        "publisher",
        "missing",
        "cancel",
        "open",
        "empty",
    ] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, mut input) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let descriptor = SourceInput {
            package_id: fixture.version.package_id.clone(),
            module_path: fixture.version.module_path.clone(),
            normalized_path: fixture.version.normalized_path.clone(),
            edition: fixture.version.edition.clone(),
            origin: if mode == "open" {
                SourceOriginInput::OpenBuffer {
                    uri: format!(
                        "file://{}",
                        fixture.root.join("alpha/src/main.miz").display()
                    ),
                    expected_version: 1,
                    actual_version: 1,
                    text: std::sync::Arc::from("definition\nend;\n"),
                }
            } else {
                SourceOriginInput::Disk {
                    path: "src/main.miz".into(),
                }
            },
        };
        let sources = if mode == "empty" {
            vec![]
        } else {
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                descriptor,
            )]
        };
        let session = draft
            .clone()
            .capture_sources(&ids, &snapshots, sources.clone())
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let unit = WorkUnit::Module {
            module: mizar_build::module_index::ModuleId::new(
                PackageId::new("alpha"),
                ModulePath::new("main"),
            ),
        };
        let owner = publisher(snapshot, usize::MAX, &unit);
        allow_frontend(&owner, &unit);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load();
        builder.register_frontend(vec![]);
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if mode != "missing" {
            driver = driver.with_output_publisher(owner.clone());
        }
        assert!(
            driver
                .validate_current_disk_diagnostics(session.id(), &snapshots)
                .is_err()
        );
        if mode == "cancel" {
            input.cancellation.supersede_snapshot(snapshot);
        }
        let (mut submission, index) = driver
            .submit_captured_with_diagnostic_index(session.clone(), &ids, &snapshots, input)
            .unwrap();
        let supported = matches!(mode, "lane" | "request" | "publisher");
        assert_eq!(index.is_some(), supported, "{mode}");
        assert_eq!(
            driver
                .validate_current_disk_diagnostics(session.id(), &snapshots)
                .is_ok(),
            supported,
            "{mode}"
        );
        assert!(
            CompilerDriver::default()
                .validate_current_disk_diagnostics(session.id(), &snapshots)
                .is_err()
        );
        if mode == "cancel" {
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Cancelled)
            );
        }
        if !supported {
            continue;
        }
        // Mutating returned carriers cannot alter the live private session guard.
        submission.publication_decision = mizar_driver::request::PublicationDecision::Suppressed(
            mizar_driver::request::ObsoletePublication {
                lane_current: false,
                request_snapshot_current: false,
            },
        );
        submission.scheduler_run = None;
        assert!(
            driver
                .validate_current_disk_diagnostics(session.id(), &snapshots)
                .is_ok()
        );
        assert!(owner.validate_current_snapshot(snapshot).is_ok());
        assert!(snapshots.is_current_for_request(snapshot, session.request().id));
        if mode == "lane" {
            draft.generation = BuildRequestGeneration::new(1);
            let next = draft.capture_sources(&ids, &snapshots, sources).unwrap();
            assert_eq!(next.captured().snapshot().id, snapshot);
            // Fresh physical owner avoids same-snapshot storage slot conflicts.
            let fresh = publisher(snapshot, usize::MAX, &unit);
            allow_frontend(&fresh, &unit);
            driver = driver.with_output_publisher(fresh.clone());
            let (_, input) = fixture.submit_request(vec![], vec![]);
            let (_, next_index) = driver
                .submit_captured_with_diagnostic_index(next.clone(), &ids, &snapshots, input)
                .unwrap();
            assert!(next_index.is_some());
            assert!(
                driver
                    .validate_current_disk_diagnostics(next.id(), &snapshots)
                    .is_ok()
            );
            assert!(fresh.validate_current_snapshot(snapshot).is_ok());
            assert!(snapshots.is_current_for_request(snapshot, session.request().id));
            assert!(
                driver
                    .validate_current_disk_diagnostics(session.id(), &snapshots)
                    .is_err()
            );
        } else if mode == "request" {
            fs::write(fixture.root.join("alpha/src/main.miz"), b"end").unwrap();
            let request = session.request();
            let (replacement, _, _) = snapshots
                .load_snapshot(
                    request.id,
                    (
                        request.workspace_root.clone(),
                        request.dependency_inputs.artifacts.clone(),
                        request.dependency_inputs.lockfile_hash,
                        request.dependency_inputs.toolchain.clone(),
                        request.verifier_config.hash,
                    ),
                    sources,
                )
                .unwrap();
            assert_ne!(replacement.id, snapshot);
            assert!(!snapshots.is_current_for_request(snapshot, request.id));
            assert!(
                driver
                    .validate_current_disk_diagnostics(session.id(), &snapshots)
                    .is_err()
            );
            assert!(owner.validate_current_snapshot(snapshot).is_ok());
        } else {
            owner.mark_obsolete(snapshot).unwrap();
            assert!(snapshots.is_current_for_request(snapshot, session.request().id));
            assert!(
                driver
                    .validate_current_disk_diagnostics(session.id(), &snapshots)
                    .is_err()
            );
        }
        // The immutable point-in-time index does not turn into a publication token.
        assert!(index.unwrap().is_empty());
    }

    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let baseline = fixture.submit(&ids, &snapshots);
    let snapshot = baseline.session.captured().snapshot().id;
    let owner = publisher(snapshot, usize::MAX, &source_task(&baseline).unit);
    allow_frontend(&owner, &source_task(&baseline).unit);
    let mut builder = PhaseRegistryBuilder::new();
    builder.register_source_load();
    builder.register_frontend(vec![]);
    let mut driver =
        CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
    let (request, input) = fixture.submit_request(vec![], vec![]);
    let legacy = driver.submit(request, &ids, &snapshots, input).unwrap();
    let stored = driver.session(legacy.session.id()).unwrap();
    assert_eq!(stored, &legacy.session);
    assert_eq!(stored.captured().snapshot().id, snapshot);
    let versions = &stored.captured().snapshot().source_versions;
    assert!(!versions.is_empty());
    assert!(
        versions
            .iter()
            .all(|version| version.origin == SourceOrigin::Disk)
    );
    assert_eq!(
        stored.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    assert_eq!(
        legacy.publication_decision,
        mizar_driver::request::PublicationDecision::Current
    );
    assert!(snapshots.is_current_for_request(snapshot, stored.request().id));
    assert!(owner.validate_current_snapshot(snapshot).is_ok());
    let results = &legacy.scheduler_run.as_ref().unwrap().phase_results;
    assert_eq!(results.len(), 2);
    assert!(
        results
            .values()
            .flatten()
            .all(|result| result.status == PhaseStatus::Complete)
    );
    assert!(
        driver
            .validate_current_disk_diagnostics(stored.id(), &snapshots)
            .is_err()
    );
}

#[test]
fn current_disk_diagnostic_submission_preserves_captured_precondition_errors() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut draft, _) = fixture.submit_request(vec![], vec![]);
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
    for mode in ["request", "state", "duplicate", "legacy"] {
        let selected = if mode == "legacy" {
            fixture
                .submit_request(vec![], vec![])
                .0
                .allocate(&ids)
                .unwrap()
                .capture_snapshot(&snapshots)
                .unwrap()
        } else {
            let mut selected = session.clone();
            if mode == "state" {
                selected.cancel();
            }
            selected
        };
        let invalid_registry = SnapshotRegistry::new();
        let registry = if mode == "request" {
            &invalid_registry
        } else {
            &snapshots
        };
        let mut original = CompilerDriver::new(source_registry());
        let mut indexed = CompilerDriver::new(source_registry());
        if mode == "duplicate" {
            let (_, input) = fixture.submit_request(vec![], vec![]);
            original
                .submit_captured(selected.clone(), &ids, registry, input)
                .unwrap();
            let (_, input) = fixture.submit_request(vec![], vec![]);
            indexed
                .submit_captured(selected.clone(), &ids, registry, input)
                .unwrap();
        }
        let (_, input) = fixture.submit_request(vec![], vec![]);
        let expected = original
            .submit_captured(selected.clone(), &ids, registry, input)
            .unwrap_err();
        let (_, input) = fixture.submit_request(vec![], vec![]);
        let actual = indexed
            .submit_captured_with_diagnostic_index(selected, &ids, registry, input)
            .unwrap_err();
        assert_eq!(actual, expected);
        assert!(matches!(
            actual,
            DriverSubmitError::InvalidCapturedSession { .. }
        ));
    }
}

#[test]
fn current_native_diagnostic_index_preserves_open_and_mixed_original_runs() {
    use mizar_diagnostics::{
        aggregator::BuildDiagnosticIndex, failure_record::DiagnosticFreshness,
    };
    for mode in ["clean", "fatal", "mixed"] {
        let raw = if mode == "clean" {
            "\u{feff}definition\r\nend;\r\n"
        } else {
            "\u{feff}end\r\n"
        };
        let fixture = Fixture::new(raw.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, _) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let mut descriptors = vec![SourceInput {
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
                actual_version: 7,
                text: raw.into(),
            },
        }];
        if mode == "mixed" {
            fs::write(
                fixture.root.join("alpha/src/leaf.miz"),
                "\u{feff}import mml.no_such;\r\ndefinition\r\nend;\r\n",
            )
            .unwrap();
            descriptors.push(SourceInput {
                package_id: PackageId::new("alpha"),
                module_path: ModulePath::new("leaf"),
                normalized_path: normalize_path(
                    &fixture.root.join("alpha"),
                    Path::new("src/leaf.miz"),
                )
                .unwrap(),
                edition: Edition::new("2025"),
                origin: SourceOriginInput::Disk {
                    path: "src/leaf.miz".into(),
                },
            });
        }
        let loader = DiskSourceLoader::new(fixture.root.join("alpha"));
        let originals = descriptors
            .iter()
            .map(|descriptor| {
                mizar_session::SourceLoader::load(
                    &loader,
                    snapshot_id(0x51),
                    descriptor.clone(),
                    &InMemorySessionIdAllocator::new(),
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let session = draft
            .capture_sources(
                &ids,
                &snapshots,
                descriptors
                    .iter()
                    .cloned()
                    .map(|descriptor| (loader.clone(), descriptor))
                    .collect(),
            )
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        for descriptor in &descriptors {
            let path = fixture
                .root
                .join("alpha")
                .join(format!("src/{}.miz", descriptor.module_path.as_str()));
            fs::write(&path, [0xff]).unwrap();
            fs::remove_file(path).unwrap();
        }
        let mut ordinary = None;
        for indexed in [false, true] {
            let unit = WorkUnit::Module {
                module: mizar_build::module_index::ModuleId::new(
                    PackageId::new("alpha"),
                    ModulePath::new("main"),
                ),
            };
            let owner = publisher(snapshot, 0, &unit);
            for descriptor in &descriptors {
                let unit = WorkUnit::Module {
                    module: mizar_build::module_index::ModuleId::new(
                        descriptor.package_id.clone(),
                        descriptor.module_path.clone(),
                    ),
                };
                owner.allow_work_unit(AllowedWorkUnit::new(
                    IrPipelinePhase::new("SourceLoad"),
                    OutputKind::new("SourceUnit"),
                    IrWorkUnit::new(format!(
                        "{:?}:{:?}",
                        descriptor.package_id.as_str(),
                        descriptor.module_path.as_str()
                    )),
                ));
                allow_frontend(&owner, &unit);
            }
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut driver =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            let (_, mut input) = fixture.submit_request(vec![], vec![]);
            if mode == "mixed" {
                input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                    package_id: PackageId::new("alpha"),
                    files: vec![
                        WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                        WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
                    ],
                }]);
            }
            let (submission, index) = if indexed {
                driver
                    .submit_captured_with_native_diagnostic_index(
                        session.clone(),
                        &ids,
                        &snapshots,
                        input,
                    )
                    .unwrap()
            } else {
                (
                    driver
                        .submit_captured(session.clone(), &ids, &snapshots, input)
                        .unwrap(),
                    None,
                )
            };
            let run = submission.scheduler_run.as_ref().unwrap();
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if mode == "clean" {
                    BuildSessionOutcome::Blocked
                } else {
                    BuildSessionOutcome::Failed
                })
            );
            let mut sources = Vec::new();
            for (descriptor, expected) in descriptors.iter().zip(&originals) {
                let unit = WorkUnit::Module {
                    module: mizar_build::module_index::ModuleId::new(
                        descriptor.package_id.clone(),
                        descriptor.module_path.clone(),
                    ),
                };
                let tasks = submission.task_graph.as_ref().unwrap().tasks();
                let task = tasks
                    .iter()
                    .find(|task| task.kind == TaskKind::SourceLoad && task.unit == unit)
                    .unwrap();
                let result = &run.phase_results[&task.id][0];
                assert_eq!(result.status, PhaseStatus::Complete);
                assert!(result.diagnostics.is_empty());
                let source = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<SourceUnit>(
                                &result.output_refs[0],
                                &OutputKind::new("SourceUnit"),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                let version = session
                    .captured()
                    .snapshot()
                    .source_versions
                    .iter()
                    .find(|version| version.module_path == descriptor.module_path)
                    .unwrap();
                assert_eq!(source.source_id, version.source_id);
                assert_eq!(source.source_hash, version.source_hash);
                assert_eq!(source.origin, version.origin);
                assert_eq!(source.source_text, expected.text);
                assert_eq!(
                    source.line_map,
                    LineMap::new(source.source_id, &expected.text)
                );
                let map = source.loading_map.as_ref().unwrap();
                assert_eq!(map.source_id, source.source_id);
                assert_eq!(map.loaded_text_hash, source.source_hash);
                assert_eq!(map.loaded_text_len, source.source_text.len());
                assert_eq!(map.origin, expected.loading_map.as_ref().unwrap().origin);
                assert_eq!(
                    map.segments,
                    expected.loading_map.as_ref().unwrap().segments
                );
                assert!(map.segments.iter().any(|segment| matches!(
                    segment,
                    mizar_session::LoadingMapSegment::RemovedLeadingBom { .. }
                )));
                assert!(map.segments.iter().any(|segment| matches!(
                    segment,
                    mizar_session::LoadingMapSegment::NormalizedNewline { .. }
                )));
                let dispatch = PhaseInput::new(
                    unit.clone(),
                    PhaseDispatchInputBundle::without_parent_outputs(
                        snapshot,
                        SourceUnitCacheKey::from_source(&source).stable_hash(),
                        vec![],
                    ),
                );
                for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
                    assert!(matches!(
                        driver
                            .registry()
                            .cache_key_for_phase(phase, &dispatch)
                            .unwrap()
                            .intent,
                        PhaseCacheIntent::NoKey { .. }
                    ));
                }
                let frontend = tasks
                    .iter()
                    .find(|task| task.kind == TaskKind::Frontend && task.unit == unit)
                    .unwrap();
                assert_eq!(
                    run.phase_results[&frontend.id][0].status,
                    if mode == "clean" {
                        PhaseStatus::Complete
                    } else if descriptor.module_path.as_str() == "leaf" {
                        PhaseStatus::Recoverable
                    } else {
                        PhaseStatus::Fatal
                    }
                );
                sources.push(source);
            }
            if indexed {
                assert_eq!(&submission, ordinary.as_ref().unwrap());
                assert_eq!(
                    driver.validate_current_native_diagnostics(session.id(), &snapshots),
                    Ok(())
                );
                assert!(
                    driver
                        .validate_current_disk_diagnostics(session.id(), &snapshots)
                        .is_err()
                );
                let index = index.unwrap();
                assert_eq!(
                    index,
                    BuildDiagnosticIndex::from_batches(
                        snapshot,
                        run.phase_results
                            .values()
                            .flatten()
                            .flat_map(|result| result.diagnostics.clone())
                            .collect()
                    )
                    .unwrap()
                );
                assert_eq!(index.snapshot(), snapshot);
                assert!(index.obsolete_drafts().is_empty());
                assert_eq!(index.is_empty(), mode == "clean");
                for source in sources {
                    let records = index.records().iter().filter(|record| matches!(record.primary_location(), DiagnosticPrimaryLocation::Span(span) if span.range().source_id == source.source_id)).collect::<Vec<_>>();
                    let codes = records
                        .iter()
                        .map(|record| record.code().number())
                        .collect::<Vec<_>>();
                    if mode == "clean" {
                        assert!(codes.is_empty());
                    } else if source.module_path.as_str() == "main" {
                        assert_eq!(codes, vec![33, 52]);
                    } else {
                        assert!(codes.contains(&22));
                        assert!(codes.contains(&220));
                    }
                    for record in records {
                        assert_eq!(record.handle().snapshot, snapshot);
                        assert_eq!(index.record_by_id(record.handle().id), Some(record));
                        assert!(
                            index
                                .handles_for_source(source.source_id)
                                .unwrap()
                                .contains(&record.handle())
                        );
                        assert_eq!(
                            record.freshness(),
                            &DiagnosticFreshness::Current {
                                source_snapshot: snapshot
                            }
                        );
                        assert_eq!(
                            record.phase(),
                            if record.code().number() >= 220 {
                                DiagnosticPhase::Resolver
                            } else {
                                DiagnosticPhase::Frontend
                            }
                        );
                        let DiagnosticPrimaryLocation::Span(span) = record.primary_location()
                        else {
                            unreachable!()
                        };
                        assert!(span.range().start <= span.range().end);
                        assert!(span.range().end <= source.source_text.len());
                        if source.module_path.as_str() == "main" {
                            assert_eq!((span.range().start, span.range().end), (0, 3));
                        }
                    }
                }
            } else {
                ordinary = Some(submission);
            }
        }
    }
}

#[test]
fn current_native_diagnostic_guard_rejects_generated_and_superseded_open_sessions() {
    for mode in ["generated", "lane", "request"] {
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
            origin: if mode == "generated" {
                SourceOriginInput::Generated {
                    generator: mizar_session::GeneratedSourceKind::new("native-index-test"),
                    text: "definition\nend;\n".into(),
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
                    text: "definition\nend;\n".into(),
                }
            },
        };
        let loader = DiskSourceLoader::new(fixture.root.join("alpha"));
        let session = draft
            .clone()
            .capture_sources(&ids, &snapshots, vec![(loader.clone(), descriptor.clone())])
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
        assert!(
            driver
                .validate_current_native_diagnostics(session.id(), &snapshots)
                .is_err()
        );
        let (submission, index) = driver
            .submit_captured_with_native_diagnostic_index(session.clone(), &ids, &snapshots, input)
            .unwrap();
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Blocked)
        );
        assert_eq!(
            submission.publication_decision,
            mizar_driver::request::PublicationDecision::Current
        );
        assert!(snapshots.is_current_for_request(snapshot, session.request().id));
        assert!(owner.validate_current_snapshot(snapshot).is_ok());
        assert_eq!(index.is_some(), mode != "generated");
        assert_eq!(
            driver
                .validate_current_native_diagnostics(session.id(), &snapshots)
                .is_ok(),
            mode != "generated"
        );
        assert!(
            driver
                .validate_current_disk_diagnostics(session.id(), &snapshots)
                .is_err()
        );
        if mode == "generated" {
            assert!(matches!(
                submission.session.captured().snapshot().source_versions[0].origin,
                SourceOrigin::Generated { .. }
            ));
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            let mut ordinary =
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone());
            let (_, input) = fixture.submit_request(vec![], vec![]);
            assert_eq!(
                submission,
                ordinary
                    .submit_captured(session.clone(), &ids, &snapshots, input)
                    .unwrap()
            );
            continue;
        }
        let mut replacement = descriptor;
        replacement.origin = SourceOriginInput::OpenBuffer {
            uri: format!(
                "file://{}",
                fixture.root.join("alpha/src/main.miz").display()
            ),
            expected_version: 8,
            actual_version: 8,
            text: if mode == "lane" {
                "definition\nend;\n"
            } else {
                "end"
            }
            .into(),
        };
        if mode == "lane" {
            draft.generation = BuildRequestGeneration::new(1);
            let next = draft
                .capture_sources(&ids, &snapshots, vec![(loader, replacement)])
                .unwrap();
            assert_eq!(next.captured().snapshot().id, snapshot);
            assert_ne!(
                next.captured().snapshot().source_versions,
                session.captured().snapshot().source_versions
            );
            let fresh = publisher(snapshot, 0, &unit);
            allow_frontend(&fresh, &unit);
            driver = driver.with_output_publisher(fresh.clone());
            let (_, input) = fixture.submit_request(vec![], vec![]);
            let (_, next_index) = driver
                .submit_captured_with_native_diagnostic_index(next.clone(), &ids, &snapshots, input)
                .unwrap();
            assert!(next_index.is_some());
            assert_eq!(
                driver.validate_current_native_diagnostics(next.id(), &snapshots),
                Ok(())
            );
            assert!(snapshots.is_current_for_request(snapshot, session.request().id));
            assert!(fresh.validate_current_snapshot(snapshot).is_ok());
        } else {
            let request = session.request();
            let (next, _, _) = snapshots
                .load_snapshot(
                    request.id,
                    (
                        request.workspace_root.clone(),
                        request.dependency_inputs.artifacts.clone(),
                        request.dependency_inputs.lockfile_hash,
                        request.dependency_inputs.toolchain.clone(),
                        request.verifier_config.hash,
                    ),
                    vec![(loader, replacement)],
                )
                .unwrap();
            assert_ne!(next.id, snapshot);
            assert!(!snapshots.is_current_for_request(snapshot, request.id));
            assert!(owner.validate_current_snapshot(snapshot).is_ok());
        }
        assert!(
            driver
                .validate_current_native_diagnostics(session.id(), &snapshots)
                .is_err()
        );
        assert!(index.unwrap().is_empty());
    }
}

#[test]
fn current_native_diagnostic_submission_preserves_captured_precondition_errors() {
    let fixture = Fixture::new(b"definition\nend;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut draft, _) = fixture.submit_request(vec![], vec![]);
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
                    origin: SourceOriginInput::OpenBuffer {
                        uri: format!(
                            "file://{}",
                            fixture.root.join("alpha/src/main.miz").display()
                        ),
                        expected_version: 7,
                        actual_version: 7,
                        text: "definition\nend;\n".into(),
                    },
                },
            )],
        )
        .unwrap();
    let invalid_registry = SnapshotRegistry::new();
    let mut original = CompilerDriver::new(source_registry());
    let mut indexed = CompilerDriver::new(source_registry());
    let (_, input) = fixture.submit_request(vec![], vec![]);
    let expected = original
        .submit_captured(session.clone(), &ids, &invalid_registry, input)
        .unwrap_err();
    let (_, input) = fixture.submit_request(vec![], vec![]);
    let actual = indexed
        .submit_captured_with_native_diagnostic_index(session, &ids, &invalid_registry, input)
        .unwrap_err();
    assert_eq!(actual, expected);
    assert!(matches!(
        actual,
        DriverSubmitError::InvalidCapturedSession { .. }
    ));
}
