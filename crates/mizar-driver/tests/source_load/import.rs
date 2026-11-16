use super::*;

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
                .submit(
                    request,
                    &fixture.ids,
                    &SnapshotRegistry::new(),
                    input,
                    Default::default(),
                )
                .unwrap_err();
            assert!(matches!(error, DriverSubmitError::TaskGraph { .. }));
            continue;
        }
        let submission = driver
            .submit(
                request,
                &fixture.ids,
                &SnapshotRegistry::new(),
                input,
                SubmitOptions {
                    discover_imports: true,
                    ..Default::default()
                },
            )
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

fn captured_native_execution_fixture(
    text: &str,
) -> (
    Fixture,
    InMemorySessionIdAllocator,
    SnapshotRegistry<InMemorySessionIdAllocator>,
    mizar_driver::request::BuildSession,
    DriverSubmitInput<StaticSourceLayout>,
    BuildSnapshotId,
) {
    let fixture = Fixture::new(text.as_bytes());
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let (mut draft, _) = fixture.submit_request(vec![], vec![]);
    draft.source_inputs.versions.clear();
    let captured_input = SourceInput {
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
            &ids,
            &snapshots,
            vec![(
                DiskSourceLoader::new(fixture.root.join("alpha")),
                captured_input,
            )],
        )
        .unwrap();
    let snapshot = session.captured().snapshot().id;
    let input = fixture.submit_request(vec![], vec![]).1;
    (fixture, ids, snapshots, session, input, snapshot)
}

#[test]
fn captured_native_execution_yields_and_matches_synchronous_completion() {
    let (fixture, ids, snapshots, session, input, snapshot) =
        captured_native_execution_fixture("theorem T: a = a;\n");
    let (mut sync_driver, _) = import_discovered_native_index_driver(snapshot, &["main"]);
    let expected = sync_driver
        .submit_captured(
            session.clone(),
            &ids,
            &snapshots,
            fixture.submit_request(vec![], vec![]).1,
            SubmitOptions::default(),
        )
        .unwrap();

    let (mut driver, _) = import_discovered_native_index_driver(snapshot, &["main"]);
    let accepted = driver
        .start_captured(
            session.clone(),
            &ids,
            &snapshots,
            input,
            SubmitOptions::default(),
        )
        .unwrap();
    let session_id = accepted.session.id();
    assert_eq!(accepted.session.state(), BuildSessionState::Running);
    assert!(accepted.scheduler_run.is_none());
    assert_eq!(accepted.session.captured().snapshot().id, snapshot);
    assert_eq!(
        accepted.session.captured().native_sources().unwrap()[0]
            .1
            .source_hash,
        session.captured().native_sources().unwrap()[0]
            .1
            .source_hash
    );

    let task_count = accepted.task_graph.as_ref().unwrap().tasks().len();
    let mut steps = 0;
    while let Some(task) = driver.pending_task(session_id).cloned() {
        if matches!(task.kind, TaskKind::SourceLoad | TaskKind::Frontend) {
            break;
        }
        steps += 1;
        assert!(
            steps <= task_count + 2,
            "progress exceeded the graph boundary allowance"
        );
        assert!(
            driver
                .progress(session_id, &ids, &snapshots)
                .unwrap()
                .is_none()
        );
    }
    assert!(matches!(
        driver.pending_task(session_id).map(|task| task.kind),
        Some(TaskKind::SourceLoad | TaskKind::Frontend)
    ));

    let mut terminal = None;
    while terminal.is_none() {
        steps += 1;
        assert!(
            steps <= task_count + 2,
            "progress exceeded the graph boundary allowance"
        );
        terminal = driver.progress(session_id, &ids, &snapshots).unwrap();
    }
    let terminal = terminal.unwrap();
    assert_eq!(terminal, expected);
    assert_eq!(
        driver.terminal_submission(session_id, &snapshots),
        Some(expected.clone())
    );
    assert_eq!(
        driver.progress(session_id, &ids, &snapshots).unwrap(),
        Some(expected)
    );
    assert_eq!(
        driver.events(session_id).events(),
        sync_driver.events(session_id).events()
    );
    assert_eq!(
        driver
            .events(session_id)
            .events()
            .iter()
            .filter(|event| matches!(event.kind, BuildEventKind::SessionFinished { .. }))
            .count(),
        1
    );
    let finished = driver.cancel(
        session_id,
        mizar_driver::driver::DriverCancelReason::ExplicitRequest,
        &snapshots,
    );
    assert!(!finished.changed);
    assert_eq!(finished.state, Some(terminal.session.state()));
    let unknown_id = ids.next_session_id().unwrap();
    let unknown = driver.cancel(
        unknown_id,
        mizar_driver::driver::DriverCancelReason::Shutdown,
        &snapshots,
    );
    assert!(!unknown.changed);
    assert_eq!(unknown.state, None);
    assert!(!driver.events(unknown_id).known_session);

    let (
        _failed_fixture,
        failed_ids,
        failed_snapshots,
        failed_session,
        mut failed_input,
        failed_snapshot,
    ) = captured_native_execution_fixture("theorem T: a = a;\n");
    failed_input.workspace_packages.clear();
    let failed_session_id = failed_session.id();
    let (mut failed_driver, _) = import_discovered_native_index_driver(failed_snapshot, &["main"]);
    let error = failed_driver
        .start_captured(
            failed_session,
            &failed_ids,
            &failed_snapshots,
            failed_input,
            SubmitOptions::default(),
        )
        .unwrap_err();
    let DriverSubmitError::Planning {
        session: stored_error_session,
        diagnostics,
    } = error
    else {
        panic!("expected a structured planning failure, got {error:?}");
    };
    assert!(!diagnostics.diagnostics().is_empty());
    assert_eq!(stored_error_session.id(), failed_session_id);
    assert_eq!(
        stored_error_session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Failed)
    );
    assert_eq!(
        failed_driver
            .session(failed_session_id)
            .map(|stored| stored.state()),
        Some(BuildSessionState::Finished(BuildSessionOutcome::Failed))
    );
    assert!(failed_driver.pending_task(failed_session_id).is_none());
    assert!(
        failed_driver
            .terminal_submission(failed_session_id, &failed_snapshots)
            .is_none()
    );
    assert!(failed_driver.events(failed_session_id).known_session);
    drop(fixture);
}

#[test]
fn captured_native_frontend_cancellation_reaches_the_real_service_checkpoint() {
    for (reason, replace_snapshot) in [
        (
            mizar_driver::driver::DriverCancelReason::ExplicitRequest,
            false,
        ),
        (mizar_driver::driver::DriverCancelReason::Shutdown, false),
        (mizar_driver::driver::DriverCancelReason::Superseded, false),
        (mizar_driver::driver::DriverCancelReason::Superseded, true),
    ] {
        let (fixture, ids, snapshots, session, input, snapshot) =
            captured_native_execution_fixture("theorem T: a = a;\n");
        let (mut driver, owner) = import_discovered_native_index_driver(snapshot, &["main"]);
        let accepted = driver
            .start_captured(session, &ids, &snapshots, input, SubmitOptions::default())
            .unwrap();
        let session_id = accepted.session.id();
        let task_count = accepted.task_graph.as_ref().unwrap().tasks().len();
        let frontend = accepted
            .task_graph
            .as_ref()
            .unwrap()
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::Frontend)
            .unwrap()
            .id
            .clone();
        let mut steps = 0;
        while driver.pending_task(session_id).map(|task| task.id.clone()) != Some(frontend.clone())
        {
            steps += 1;
            assert!(
                steps <= task_count + 2,
                "progress exceeded the graph boundary allowance"
            );
            assert!(
                driver
                    .progress(session_id, &ids, &snapshots)
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(driver.pending_task(session_id).unwrap().id, frontend);

        let superseding = if replace_snapshot {
            fixture.write(b"theorem Other: b = b;\n");
            let (mut draft, _) = fixture.submit_request(vec![], vec![]);
            draft.generation = BuildRequestGeneration::new(1);
            draft.source_inputs.versions.clear();
            let replacement_source = SourceInput {
                package_id: fixture.version.package_id.clone(),
                module_path: fixture.version.module_path.clone(),
                normalized_path: fixture.version.normalized_path.clone(),
                edition: fixture.version.edition.clone(),
                origin: SourceOriginInput::Disk {
                    path: "src/main.miz".into(),
                },
            };
            let replacement = draft
                .capture_sources(
                    &ids,
                    &snapshots,
                    vec![(
                        DiskSourceLoader::new(fixture.root.join("alpha")),
                        replacement_source,
                    )],
                )
                .unwrap();
            let replacement_snapshot = replacement.captured().snapshot().id;
            assert_ne!(snapshot, replacement_snapshot);
            owner
                .replace_current_snapshot(snapshot, replacement_snapshot)
                .unwrap();
            let replacement = driver
                .start_captured(
                    replacement,
                    &ids,
                    &snapshots,
                    fixture.submit_request(vec![], vec![]).1,
                    SubmitOptions::default(),
                )
                .unwrap();
            Some((
                replacement.session.id(),
                replacement.task_graph.unwrap().tasks().len(),
            ))
        } else {
            None
        };

        let outcome = driver.cancel(session_id, reason, &snapshots);
        assert!(outcome.changed);
        assert_eq!(outcome.state, Some(BuildSessionState::Cancelling));
        let mut terminal = None;
        while terminal.is_none() {
            steps += 1;
            assert!(
                steps <= task_count + 2,
                "progress exceeded the graph boundary allowance"
            );
            terminal = driver.progress(session_id, &ids, &snapshots).unwrap();
        }
        let terminal = terminal.unwrap();
        assert_eq!(
            terminal.session.state(),
            BuildSessionState::Finished(
                if reason == mizar_driver::driver::DriverCancelReason::Superseded {
                    BuildSessionOutcome::Superseded
                } else {
                    BuildSessionOutcome::Cancelled
                }
            )
        );
        let run = terminal.scheduler_run.as_ref().unwrap();
        if replace_snapshot {
            assert!(run.phase_results.is_empty());
            assert!(run.task_states.iter().any(|task| {
                task.task_id == frontend
                    && task.state == mizar_build::scheduler::TaskState::Cancelled
            }));
        } else {
            let cancelled = run.phase_results[&frontend].first().unwrap();
            assert_eq!(cancelled.status, PhaseStatus::Cancelled);
            assert!(cancelled.output_refs.is_empty());
        }
        let events = driver.events(session_id);
        assert_eq!(
            events
                .events()
                .iter()
                .filter(|event| matches!(event.kind, BuildEventKind::SessionFinished { .. }))
                .count(),
            1
        );
        assert!(
            !events
                .events()
                .iter()
                .any(|event| matches!(event.kind, BuildEventKind::SnapshotPublished { .. }))
        );
        if replace_snapshot {
            assert!(
                events
                    .events()
                    .iter()
                    .any(|event| { matches!(event.kind, BuildEventKind::PublicationSuppressed) })
            );
            let (replacement_id, replacement_graph_size) = superseding.unwrap();
            let mut replacement_steps = 0;
            let mut replacement_terminal = None;
            while replacement_terminal.is_none() {
                replacement_steps += 1;
                assert!(replacement_steps <= replacement_graph_size + 2);
                replacement_terminal = driver.progress(replacement_id, &ids, &snapshots).unwrap();
            }
            let replacement_terminal = replacement_terminal.unwrap();
            assert!(matches!(
                replacement_terminal.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Blocked)
                    | BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
                    | BuildSessionState::Finished(BuildSessionOutcome::Failed)
            ));
        }
        let repeated = driver.cancel(session_id, reason, &snapshots);
        assert!(!repeated.changed);
        assert_eq!(repeated.state, Some(terminal.session.state()));
        assert_eq!(driver.events(session_id).events(), events.events());
        drop(fixture);
    }
}

#[test]
fn equal_snapshot_native_executions_on_distinct_lanes_keep_signals_and_outputs_isolated() {
    let fixture = Fixture::new(b"theorem T: a = a;\n");
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let capture = |lane, generation| {
        let (mut draft, _) = fixture.submit_request(vec![], vec![]);
        draft.lane = BuildLaneId::new(lane);
        draft.generation = BuildRequestGeneration::new(generation);
        draft.source_inputs.versions.clear();
        let captured_input = SourceInput {
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
                &ids,
                &snapshots,
                vec![(
                    DiskSourceLoader::new(fixture.root.join("alpha")),
                    captured_input,
                )],
            )
            .unwrap();
        (session, fixture.submit_request(vec![], vec![]).1)
    };
    for (replacement_lane, replacement_generation) in [(1, 1), (2, 0)] {
        let (old, old_input) = capture(1, 0);
        let (replacement, replacement_input) = capture(replacement_lane, replacement_generation);
        let snapshot = old.captured().snapshot().id;
        assert_eq!(replacement.captured().snapshot().id, snapshot);
        let (mut driver, owner) = import_discovered_native_index_driver(snapshot, &["main"]);
        let old_accepted = driver
            .start_captured(old, &ids, &snapshots, old_input, SubmitOptions::default())
            .unwrap();
        let old_id = old_accepted.session.id();
        let old_graph_size = old_accepted.task_graph.as_ref().unwrap().tasks().len();
        let old_source_load = old_accepted
            .task_graph
            .as_ref()
            .unwrap()
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::SourceLoad)
            .unwrap()
            .id
            .clone();
        let mut old_steps = 0;
        while driver.pending_task(old_id).map(|task| task.id.clone())
            != Some(old_source_load.clone())
        {
            old_steps += 1;
            assert!(old_steps <= old_graph_size + 2);
            assert!(driver.progress(old_id, &ids, &snapshots).unwrap().is_none());
        }

        let new_accepted = driver
            .start_captured(
                replacement,
                &ids,
                &snapshots,
                replacement_input,
                SubmitOptions::default(),
            )
            .unwrap();
        let new_id = new_accepted.session.id();
        assert_ne!(old_id, new_id);
        let new_frontend = new_accepted
            .task_graph
            .as_ref()
            .unwrap()
            .tasks()
            .iter()
            .find(|task| task.kind == TaskKind::Frontend)
            .unwrap()
            .id
            .clone();
        let new_graph_size = new_accepted.task_graph.as_ref().unwrap().tasks().len();

        let cancelled = driver.cancel(
            old_id,
            mizar_driver::driver::DriverCancelReason::Superseded,
            &snapshots,
        );
        assert!(cancelled.changed);
        assert_eq!(cancelled.state, Some(BuildSessionState::Cancelling));
        let mut old_terminal = None;
        while old_terminal.is_none() {
            old_steps += 1;
            assert!(old_steps <= old_graph_size + 2);
            old_terminal = driver.progress(old_id, &ids, &snapshots).unwrap();
        }
        let old_terminal = old_terminal.unwrap();
        assert_eq!(
            old_terminal.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Superseded)
        );
        let old_run = old_terminal.scheduler_run.as_ref().unwrap();
        assert!(old_run.task_states.iter().any(|task| {
            task.task_id == old_source_load
                && task.state == mizar_build::scheduler::TaskState::Cancelled
        }));
        assert!(
            old_run
                .phase_results
                .values()
                .flatten()
                .all(|result| { result.output_refs.is_empty() })
        );
        let old_events = driver.events(old_id);
        assert_eq!(
            old_events
                .events()
                .iter()
                .filter(|event| matches!(event.kind, BuildEventKind::SessionFinished { .. }))
                .count(),
            1
        );
        if replacement_lane == 1 {
            assert!(matches!(
                old_terminal.publication_decision,
                mizar_driver::request::PublicationDecision::Suppressed(_)
            ));
            assert!(
                old_events
                    .events()
                    .iter()
                    .any(|event| { matches!(event.kind, BuildEventKind::PublicationSuppressed) })
            );
        } else {
            assert_eq!(
                old_terminal.publication_decision,
                mizar_driver::request::PublicationDecision::Current
            );
            assert!(
                !old_events
                    .events()
                    .iter()
                    .any(|event| { matches!(event.kind, BuildEventKind::PublicationSuppressed) })
            );
        }

        let mut new_steps = 0;
        let mut new_terminal = None;
        while new_terminal.is_none() {
            new_steps += 1;
            assert!(new_steps <= new_graph_size + 2);
            new_terminal = driver.progress(new_id, &ids, &snapshots).unwrap();
        }
        let new_terminal = new_terminal.unwrap();
        let new_run = new_terminal.scheduler_run.as_ref().unwrap();
        let new_frontend_result = &new_run.phase_results[&new_frontend][0];
        assert_eq!(new_frontend_result.status, PhaseStatus::Complete);
        for output in new_run
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| &result.output_refs)
        {
            owner.storage().validate_handle(output).unwrap();
        }
        assert_eq!(
            driver
                .events(new_id)
                .events()
                .iter()
                .filter(|event| matches!(event.kind, BuildEventKind::SessionFinished { .. }))
                .count(),
            1
        );
    }
}

#[test]
fn import_discovered_native_index_preserves_captured_import_runs_and_original_batches() {
    use mizar_diagnostics::aggregator::BuildDiagnosticIndex;
    for (origin, diagnostic, workers, order) in [
        ("disk", false, 1, CompletionOrder::Canonical),
        ("open", true, 4, CompletionOrder::Reverse),
        ("mixed", false, 4, CompletionOrder::Canonical),
    ] {
        let main = if diagnostic {
            "import alpha.leaf;\ntheorem T: ;\n"
        } else {
            "import alpha.leaf;\ntheorem T: a combine b = a;\n"
        };
        let leaf =
            "definition\nlet x, y be set;\npublic func Infix: x combine y -> set equals x;\nend;\n";
        let fixture = Fixture::new(main.as_bytes());
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, _) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let descriptors = [("main", main), ("leaf", leaf)]
            .into_iter()
            .map(|(module, text)| {
                let path = format!("src/{module}.miz");
                let raw = format!("\u{feff}{}", text.replace('\n', "\r\n"));
                fs::write(fixture.root.join("alpha").join(&path), &raw).unwrap();
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
                        origin: if origin == "disk" || origin == "mixed" && module == "leaf" {
                            SourceOriginInput::Disk { path: path.into() }
                        } else {
                            SourceOriginInput::OpenBuffer {
                                uri: format!(
                                    "file://{}",
                                    fixture.root.join("alpha").join(&path).display()
                                ),
                                expected_version: 7,
                                actual_version: 7,
                                text: raw.into(),
                            }
                        },
                    },
                )
            })
            .collect();
        let session = draft
            .capture_sources(&ids, &snapshots, descriptors)
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        for module in ["main", "leaf"] {
            let path = fixture.root.join(format!("alpha/src/{module}.miz"));
            fs::remove_file(path).unwrap();
        }
        let mut reference = None;
        for indexed in [false, true] {
            let (mut driver, owner) =
                import_discovered_native_index_driver(snapshot, &["main", "leaf"]);
            let (_, mut input) = fixture.submit_request(vec![], vec![]);
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
            if !indexed {
                reference = Some(
                    driver
                        .submit_captured(
                            session.clone(),
                            &ids,
                            &snapshots,
                            input,
                            SubmitOptions {
                                discover_imports: true,
                                ..Default::default()
                            },
                        )
                        .unwrap(),
                );
                continue;
            }
            let (submission, index) = driver
                .submit_captured(
                    session.clone(),
                    &ids,
                    &snapshots,
                    input,
                    SubmitOptions {
                        discover_imports: true,
                        diagnostic_index: Some(DiagnosticIndexMode::Native),
                        ..Default::default()
                    },
                )
                .map(|mut submission| {
                    let index = submission.diagnostic_index.take();
                    (submission, index)
                })
                .unwrap();
            assert_eq!(submission, reference.take().unwrap());
            let graph = submission.task_graph.as_ref().unwrap();
            let frontend = |name| {
                graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == name)).unwrap()
            };
            assert!(frontend("main").dependencies.contains(&frontend("leaf").id));
            assert!(
                graph
                    .tasks()
                    .iter()
                    .filter(|task| task.kind == TaskKind::ModuleResolve)
                    .all(|task| task.dependency_coverage
                        == DependencyCoverage::MissingModuleDependencyOverlay)
            );
            let run = submission.scheduler_run.as_ref().unwrap();
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if diagnostic {
                    BuildSessionOutcome::Failed
                } else {
                    BuildSessionOutcome::Blocked
                })
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
            assert_eq!(index.is_empty(), !diagnostic);
            for (_, captured, _) in session.captured().native_sources().unwrap() {
                let module = captured.module_path.as_str();
                let task = graph.tasks().iter().find(|task| task.kind == TaskKind::SourceLoad && matches!(&task.unit, WorkUnit::Module { module: id } if id.path.as_str() == module)).unwrap();
                let source_result = &run.phase_results[&task.id][0];
                let frontend_task = frontend(module);
                let frontend_result = &run.phase_results[&frontend_task.id][0];
                assert_eq!(source_result.status, PhaseStatus::Complete);
                assert_eq!(
                    frontend_result.status,
                    if diagnostic && module == "main" {
                        PhaseStatus::Recoverable
                    } else {
                        PhaseStatus::Complete
                    }
                );
                assert!(source_result.cache_observation.is_some());
                assert!(frontend_result.cache_observation.is_some());
                let source = owner
                    .storage()
                    .get(
                        &owner
                            .storage()
                            .typed_handle::<SourceUnit>(
                                &source_result.output_refs[0],
                                &OutputKind::new("SourceUnit"),
                            )
                            .unwrap(),
                    )
                    .unwrap();
                assert_eq!(source.source_id, captured.source_id);
                assert_eq!(source.source_hash, captured.source_hash);
                assert_eq!(source.origin, captured.origin);
                assert_eq!(source.source_text, captured.text);
                assert_eq!(
                    source.source_text.as_ref(),
                    if module == "main" { main } else { leaf }
                );
                assert_eq!(source.line_map, captured.line_map);
                assert_eq!(source.loading_map, captured.loading_map);
                assert!(source.loading_map.is_some());
                for phase in [PipelinePhase::SourceLoad, PipelinePhase::Frontend] {
                    let (dependencies, parents) = if phase == PipelinePhase::Frontend {
                        let mut dependencies = submission
                            .module_index
                            .as_ref()
                            .unwrap()
                            .dependency_summaries
                            .iter()
                            .map(|summary| summary.content_hash)
                            .collect::<Vec<_>>();
                        dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
                        let parents = frontend_task
                            .dependencies
                            .iter()
                            .flat_map(|id| run.phase_results.get(id).into_iter().flatten())
                            .filter(|result| result.status == PhaseStatus::Complete)
                            .flat_map(|result| &result.output_refs)
                            .filter(|parent| {
                                (parent.phase() == &IrPipelinePhase::new("SourceLoad")
                                    && parent.work_unit()
                                        == source_result.output_refs[0].work_unit()
                                    && parent.output_kind() == &OutputKind::new("SourceUnit"))
                                    || (parent.phase() == &IrPipelinePhase::new("Frontend")
                                        && parent.output_kind()
                                            == &OutputKind::new("FrontendOutput"))
                            })
                            .map(|parent| {
                                SealedParentOutputHandle::from_published_output(
                                    &owner,
                                    snapshot,
                                    parent.clone(),
                                )
                                .unwrap()
                            })
                            .collect();
                        (dependencies, parents)
                    } else {
                        (vec![], vec![])
                    };
                    let dispatch = PhaseInput::new(
                        task.unit.clone(),
                        PhaseDispatchInputBundle::new(
                            snapshot,
                            SourceUnitCacheKey::from_source(&source).stable_hash(),
                            dependencies,
                            parents,
                        )
                        .unwrap(),
                    );
                    assert!(
                        matches!(driver.registry().cache_key_for_phase(phase, &dispatch).unwrap().intent, PhaseCacheIntent::NoKey { reason } if !reason.is_empty())
                    );
                }
                let records = index.records().iter().filter(|record| matches!(record.primary_location(), DiagnosticPrimaryLocation::Span(span) if span.range().source_id == captured.source_id)).collect::<Vec<_>>();
                assert_eq!(!records.is_empty(), diagnostic && module == "main");
                for record in records {
                    let DiagnosticPrimaryLocation::Span(span) = record.primary_location() else {
                        unreachable!()
                    };
                    assert!(span.range().start <= span.range().end);
                    assert!(span.range().end <= source.source_text.len());
                    assert_eq!(record.phase(), DiagnosticPhase::Frontend);
                    assert_eq!(record.handle().snapshot, snapshot);
                }
            }
        }
    }
}

#[test]
fn import_discovered_native_index_preserves_discovery_failure_and_supplied_overlays() {
    use mizar_build::{module_index::ModuleId, task_graph::ModuleDependencyCoverage};
    for mode in ["failure", "imports_only", "unavailable_edge"] {
        let main = if mode == "failure" {
            "import alpha.leaf;\nimport alpha.missing;\ndefinition\nend;\n"
        } else {
            "import alpha.leaf;\ndefinition\nend;\n"
        };
        let fixture = Fixture::new(main.as_bytes());
        fs::write(
            fixture.root.join("alpha/src/leaf.miz"),
            "definition\nend;\n",
        )
        .unwrap();
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let (mut draft, _) = fixture.submit_request(vec![], vec![]);
        draft.source_inputs.versions.clear();
        let descriptors = ["main", "leaf"]
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
            .collect();
        let session = draft
            .capture_sources(&ids, &snapshots, descriptors)
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let mut reference = None;
        for indexed in [false, true] {
            let (mut driver, _) =
                import_discovered_native_index_driver(snapshot, &["main", "leaf"]);
            let (_, mut input) = fixture.submit_request(vec![], vec![]);
            input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: vec![
                    WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                    WorkspaceSourceFile::new("src/leaf.miz", "leaf.miz"),
                ],
            }]);
            // Supplied overlays are adversarial caller inputs, never real Complete producers.
            input.dependency_overlay = match mode {
                "imports_only" => ModuleDependencyOverlay {
                    coverage: ModuleDependencyCoverage::ImportsOnly,
                    edges: vec![],
                },
                "unavailable_edge" => ModuleDependencyOverlay {
                    coverage: ModuleDependencyCoverage::Unavailable,
                    edges: vec![ModuleDependencyEdge {
                        dependent: ModuleId::new(PackageId::new("alpha"), ModulePath::new("main")),
                        dependency: ModuleId::new(PackageId::new("alpha"), ModulePath::new("leaf")),
                        kind: ModuleDependencyKind::ImportSummary,
                    }],
                },
                _ => ModuleDependencyOverlay::unavailable(),
            };
            let result = if indexed {
                driver
                    .submit_captured(
                        session.clone(),
                        &ids,
                        &snapshots,
                        input,
                        SubmitOptions {
                            discover_imports: true,
                            diagnostic_index: Some(DiagnosticIndexMode::Native),
                            ..Default::default()
                        },
                    )
                    .map(|mut submission| {
                        let index = submission.diagnostic_index.take();
                        (submission, index)
                    })
            } else {
                driver
                    .submit_captured(
                        session.clone(),
                        &ids,
                        &snapshots,
                        input,
                        SubmitOptions {
                            discover_imports: true,
                            ..Default::default()
                        },
                    )
                    .map(|submission| (submission, None))
            };
            if !indexed {
                reference = Some(result);
                continue;
            }
            match (reference.take().unwrap(), result) {
                (Err(expected), Err(actual)) => {
                    assert!(matches!(mode, "failure" | "unavailable_edge"));
                    assert_eq!(actual, expected);
                    let DriverSubmitError::TaskGraph { diagnostics, .. } = actual else {
                        panic!("discovery must fail closed before scheduling")
                    };
                    assert!(
                        diagnostics
                            .diagnostics()
                            .iter()
                            .any(|diagnostic| diagnostic.kind
                                == TaskGraphDiagnosticKind::MissingModuleDependencyOverlay)
                    );
                }
                (Ok((expected, _)), Ok((submission, index))) => {
                    assert!(!matches!(mode, "failure" | "unavailable_edge"));
                    assert_eq!(submission, expected);
                    let graph = submission.task_graph.as_ref().unwrap();
                    let frontend = |name| {
                        graph.tasks().iter().find(|task| task.kind == TaskKind::Frontend && matches!(&task.unit, WorkUnit::Module { module } if module.path.as_str() == name)).unwrap()
                    };
                    assert!(!frontend("main").dependencies.contains(&frontend("leaf").id));
                    assert!(
                        graph
                            .tasks()
                            .iter()
                            .filter(|task| task.kind == TaskKind::ModuleResolve)
                            .all(|task| task.dependency_coverage
                                == DependencyCoverage::MissingModuleDependencyOverlay)
                    );
                    assert!(index.is_some());
                }
                other => panic!("discovery reference mismatch: {other:?}"),
            }
        }
    }
}

#[test]
fn import_discovered_native_index_preserves_native_eligibility_currentness_and_errors() {
    for mode in [
        "current",
        "obsolete_publisher",
        "generated",
        "invalid_registry",
    ] {
        let fixture = Fixture::new(b"definition\nend;\n");
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let invalid_registry = SnapshotRegistry::new();
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
                        origin: if mode == "generated" {
                            SourceOriginInput::Generated {
                                generator: mizar_session::GeneratedSourceKind::new(
                                    "import-index-guard",
                                ),
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
                    },
                )],
            )
            .unwrap();
        let snapshot = session.captured().snapshot().id;
        let registry = if mode == "invalid_registry" {
            &invalid_registry
        } else {
            &snapshots
        };
        let mut reference = None;
        for indexed in [false, true] {
            let (mut driver, owner) = import_discovered_native_index_driver(snapshot, &["main"]);
            if mode == "obsolete_publisher" {
                owner.mark_obsolete(snapshot).unwrap();
            }
            let (_, mut input) = fixture.submit_request(vec![], vec![]);
            input.dependency_overlay = if mode == "generated" {
                // Only bypass discovery as caller input to isolate native guard rejection.
                ModuleDependencyOverlay {
                    coverage: mizar_build::task_graph::ModuleDependencyCoverage::ImportsOnly,
                    edges: vec![],
                }
            } else {
                ModuleDependencyOverlay::unavailable()
            };
            let result = if indexed {
                driver
                    .submit_captured(
                        session.clone(),
                        &ids,
                        registry,
                        input,
                        SubmitOptions {
                            discover_imports: true,
                            diagnostic_index: Some(DiagnosticIndexMode::Native),
                            ..Default::default()
                        },
                    )
                    .map(|mut submission| {
                        let index = submission.diagnostic_index.take();
                        (submission, index)
                    })
            } else {
                driver
                    .submit_captured(
                        session.clone(),
                        &ids,
                        registry,
                        input,
                        SubmitOptions {
                            discover_imports: true,
                            ..Default::default()
                        },
                    )
                    .map(|submission| (submission, None))
            };
            if !indexed {
                reference = Some(result);
                continue;
            }
            if mode == "invalid_registry" {
                let actual = result.unwrap_err();
                assert_eq!(actual, reference.take().unwrap().unwrap_err());
                assert!(matches!(
                    actual,
                    DriverSubmitError::InvalidCapturedSession { .. }
                ));
                continue;
            }
            let (submission, index) = result.unwrap();
            assert_eq!(submission, reference.take().unwrap().unwrap().0);
            assert_eq!(index.is_some(), mode == "current");
            assert_eq!(
                driver
                    .validate_current_native_diagnostics(session.id(), &snapshots)
                    .is_ok(),
                mode == "current"
            );
            if mode == "current" {
                assert_eq!(
                    submission.session.state(),
                    BuildSessionState::Finished(BuildSessionOutcome::Blocked)
                );
                assert!(submission.scheduler_run.is_some());
                let index = index.unwrap();
                assert!(index.is_empty());
            }
        }
    }
}
