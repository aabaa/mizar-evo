use super::*;

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
