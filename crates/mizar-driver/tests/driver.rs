use std::sync::{Arc, Mutex};

use mizar_build::{
    cache_seam::{
        CacheOutputRef, CacheSchedulingOutcome, CacheSchedulingPlan, CacheTaskDecision,
        ValidatedCacheHit,
    },
    cancel::CancellationPolicy,
    module_index::{StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage},
    planner::{
        DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile, parse_package_manifest,
    },
    scheduler::{CacheSchedulingPolicy, CompletionOrder, SchedulerInput, TaskState, run_scheduler},
    task_graph::{BuildTask, ModuleDependencyOverlay, PipelinePhase, TaskId},
};
use mizar_driver::{
    driver::{
        BuildSubmission, CompilerDriver, DriverSubmissionStatus, DriverSubmitInput,
        PhaseDispatchInputProvider,
    },
    events::BuildEventKind,
    registry::{
        PhaseCacheContext, PhaseCacheIntent, PhaseDescriptor, PhaseExecutionContext, PhaseInput,
        PhaseRegistry, PhaseRegistryBuilder, PhaseResult, PhaseService, PhaseServiceAvailability,
        PhaseStatus, required_phase_services,
    },
    request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildSessionOutcome, BuildSessionState,
        BuildTargets, DependencyInputSet, PublicationDecision, SourceInputSet, VerifierConfigInput,
    },
};
use mizar_ir::{
    dispatch_input::{
        PhaseDispatchInputBundle, PhaseDispatchInputRequest, SealedParentOutputHandle,
    },
    identity::{
        NamedInputHash, OutputKind, PipelinePhase as IrPipelinePhase, SnapshotHandleRegistry,
        WorkUnit as IrWorkUnit,
    },
    publisher::{
        AllowedWorkUnit, OutputOrigin, PhaseOutputPublisher, PublicationTarget, PublishOutputInput,
    },
    storage::{BlobDecodeError, BlobDecoder, IrSideTables, IrStorageService, SchemaVersion},
};
use mizar_session::{
    BuildSnapshotId, DependencyArtifactRef, Hash, InMemorySessionIdAllocator, PackageId,
    SnapshotRegistry, ToolchainInfo, WorkspaceRoot,
};

#[test]
fn submit_bootstraps_phase_zero_and_consumes_modeled_scheduler() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();

    let submission = driver
        .submit(request(1), &ids, &snapshots, submit_input(Vec::new()))
        .expect("root-only workspace submits");

    assert_eq!(
        submission.status,
        DriverSubmissionStatus::SchedulerValidated
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
    );
    assert_eq!(submission.build_plan.as_ref().unwrap().packages.len(), 1);
    assert!(submission.module_index.as_ref().unwrap().modules.is_empty());
    assert_eq!(submission.task_graph.as_ref().unwrap().tasks().len(), 1);
    assert!(submission.missing_services.is_empty());
    assert!(submission.dispatch_gap_phases.is_empty());
    assert!(submission.scheduler_run.is_some());
    assert_eq!(
        submission.publication_decision,
        PublicationDecision::Current
    );
    let mut expected_input = SchedulerInput::new(submission.task_graph.as_ref().unwrap().clone());
    expected_input.cache = CacheSchedulingPolicy::Unavailable;
    expected_input.completion_order = CompletionOrder::Reverse;
    let expected_run =
        run_scheduler(expected_input).expect("direct mizar-build scheduler run succeeds");
    let driver_run = submission.scheduler_run.as_ref().unwrap();
    assert!(driver_run.phase_results.is_empty());
    assert_eq!(driver_run.task_states, expected_run.task_states);
    assert_eq!(driver_run.events, expected_run.events);
    assert_eq!(driver_run.diagnostics, expected_run.diagnostics);
    assert_eq!(
        driver
            .session(submission.session.id())
            .map(|session| session.state()),
        Some(BuildSessionState::Finished(BuildSessionOutcome::Succeeded))
    );
    let stream = driver.events(submission.session.id());
    let task_progress: Vec<_> = stream
        .events()
        .iter()
        .filter_map(|event| match &event.kind {
            BuildEventKind::TaskProgress { task } => Some(task),
            _ => None,
        })
        .collect();
    assert!(!task_progress.is_empty());
    assert!(
        task_progress
            .iter()
            .all(|task| !task.task_id.is_empty() && task.state.is_some()),
        "all task-progress events must carry scheduler task identity and state"
    );
    assert_event(&driver, submission.session.id(), |kind| {
        matches!(kind, BuildEventKind::SessionFinished { .. })
    });
    assert_no_event(&driver, submission.session.id(), |kind| {
        matches!(
            kind,
            BuildEventKind::DiagnosticsReady { .. } | BuildEventKind::ArtifactBoundary { .. }
        )
    });
}

#[test]
fn non_phase_zero_descriptor_fixtures_block_on_dispatch_gap() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::new(scheduler_fixture_registry());

    let submission = driver
        .submit(
            request(3),
            &ids,
            &snapshots,
            submit_input(vec![
                WorkspaceSourceFile::new("src/main.miz", "main.miz"),
                WorkspaceSourceFile::new("src/util.miz", "util.miz"),
            ]),
        )
        .expect("registered descriptor fixtures classify dispatch readiness");

    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByPhaseDispatchGap
    );
    assert!(submission.missing_services.is_empty());
    assert!(submission.scheduler_run.is_some());
    assert!(
        submission.task_graph.as_ref().unwrap().tasks().len() > 1,
        "fixture must exercise more than a package root task"
    );
    assert_eq!(
        submission.dispatch_gap_phases,
        vec![PipelinePhase::SourceLoad]
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    assert_event(&driver, submission.session.id(), |kind| {
        matches!(kind, BuildEventKind::DispatchGap { .. })
    });
    assert_event(&driver, submission.session.id(), |kind| {
        matches!(kind, BuildEventKind::TaskProgress { .. })
    });
    assert_no_event(&driver, submission.session.id(), |kind| {
        matches!(
            kind,
            BuildEventKind::DiagnosticsReady { .. } | BuildEventKind::ArtifactBoundary { .. }
        )
    });
}

#[test]
fn registered_services_execute_from_scheduler_selected_dispatch() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut driver = CompilerDriver::new(executable_scheduler_fixture_registry(calls.clone()));
    let mut input = submit_input(vec![
        WorkspaceSourceFile::new("src/main.miz", "main.miz"),
        WorkspaceSourceFile::new("src/util.miz", "util.miz"),
    ]);
    input.phase_dispatch_inputs = Some(Box::new(FixturePhaseInputs));

    let submission = driver
        .submit(request(4), &ids, &snapshots, input)
        .expect("registered services dispatch from scheduler callback");

    assert_eq!(
        submission.status,
        DriverSubmissionStatus::SchedulerValidated
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
    );
    assert!(submission.missing_services.is_empty());
    assert!(submission.dispatch_gap_phases.is_empty());
    assert!(submission.scheduler_run.is_some());
    assert!(
        submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .task_states
            .iter()
            .all(|record| record.state == mizar_build::scheduler::TaskState::Completed)
    );
    let calls = calls.lock().expect("calls lock is not poisoned");
    assert!(
        calls.iter().any(|service| service == "SourceLoad")
            && calls.iter().any(|service| service == "Frontend"),
        "source/frontend service should execute from scheduler-selected callback"
    );
    assert!(
        calls.iter().any(|service| service == "ArtifactService"),
        "artifact service should execute from scheduler-selected callback"
    );
    let module_count = submission
        .module_index
        .as_ref()
        .expect("submission has module index")
        .modules
        .len();
    assert_eq!(
        calls
            .iter()
            .filter(|service| service.as_str() == "SemanticChecker")
            .count(),
        module_count,
        "check-and-elaborate task must dispatch the semantic service span"
    );
    assert_eq!(
        calls
            .iter()
            .filter(|service| service.as_str() == "Elaborator")
            .count(),
        module_count,
        "check-and-elaborate task must dispatch the elaboration service span"
    );
    assert_event(&driver, submission.session.id(), |kind| {
        matches!(kind, BuildEventKind::TaskProgress { .. })
    });
    assert_no_event(&driver, submission.session.id(), |kind| {
        matches!(kind, BuildEventKind::DispatchGap { .. })
    });
}

#[test]
fn invalid_phase_dispatch_bundle_fails_without_dispatch_gap() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut driver = CompilerDriver::new(executable_scheduler_fixture_registry(calls.clone()));
    let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
    input.phase_dispatch_inputs = Some(Box::new(WrongSnapshotPhaseInputs));

    let submission = driver
        .submit(request(5), &ids, &snapshots, input)
        .expect("invalid dispatch input is reported through scheduler outcome");

    assert_eq!(
        submission.status,
        DriverSubmissionStatus::SchedulerValidated
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Failed)
    );
    assert!(submission.dispatch_gap_phases.is_empty());
    assert!(
        calls.lock().expect("calls lock is not poisoned").is_empty(),
        "invalid dispatch input must fail before phase execution"
    );
    assert_no_event(&driver, submission.session.id(), |kind| {
        matches!(
            kind,
            BuildEventKind::DiagnosticsReady { .. } | BuildEventKind::ArtifactBoundary { .. }
        )
    });
}

#[test]
fn phase_dispatch_provider_error_fails_without_dispatch_gap() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut driver = CompilerDriver::new(executable_scheduler_fixture_registry(calls.clone()));
    let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
    input.phase_dispatch_inputs = Some(Box::new(ErrorPhaseInputs));

    let submission = driver
        .submit(request(6), &ids, &snapshots, input)
        .expect("provider dispatch error is reported through scheduler outcome");

    assert_eq!(
        submission.status,
        DriverSubmissionStatus::SchedulerValidated
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Failed)
    );
    assert!(submission.dispatch_gap_phases.is_empty());
    assert!(
        calls.lock().expect("calls lock is not poisoned").is_empty(),
        "invalid provider result must fail before phase execution"
    );
    assert_no_event(&driver, submission.session.id(), |kind| {
        matches!(
            kind,
            BuildEventKind::DiagnosticsReady { .. } | BuildEventKind::ArtifactBoundary { .. }
        )
    });
}

#[test]
fn phase_dispatch_provider_none_blocks_as_dispatch_gap() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut driver = CompilerDriver::new(executable_scheduler_fixture_registry(calls.clone()));
    let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
    input.phase_dispatch_inputs = Some(Box::new(NonePhaseInputs));

    let submission = driver
        .submit(request(7), &ids, &snapshots, input)
        .expect("missing provider bundle is reported as dispatch gap");

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
            .dispatch_gap_phases
            .contains(&PipelinePhase::SourceLoad)
    );
    assert!(
        calls.lock().expect("calls lock is not poisoned").is_empty(),
        "missing provider bundle must block before phase execution"
    );
    assert_event(&driver, submission.session.id(), |kind| {
        matches!(kind, BuildEventKind::DispatchGap { .. })
    });
}

#[test]
fn cache_hit_tasks_do_not_require_phase_dispatch_inputs() {
    let source = {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut driver = CompilerDriver::new(executable_scheduler_fixture_registry(calls));
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        input.phase_dispatch_inputs = Some(Box::new(FixturePhaseInputs));
        let submission = driver
            .submit(request(41), &ids, &snapshots, input)
            .expect("probe run builds task graph");
        task_id_for_phase(&submission, PipelinePhase::SourceLoad)
    };

    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::new(scheduler_fixture_registry());
    let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
    input.cache_policy = CacheSchedulingPolicy::Enabled;
    input.cache_decisions = CacheSchedulingPlan::new(vec![CacheTaskDecision::new(
        source.clone(),
        CacheSchedulingOutcome::ValidatedHit(ValidatedCacheHit::new(
            vec![CacheOutputRef::new("source", "cached")],
            Vec::new(),
        )),
    )]);

    let submission = driver
        .submit(request(41), &ids, &snapshots, input)
        .expect("cache hit skips dispatch input gap for source task");

    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByPhaseDispatchGap
    );
    assert!(submission.scheduler_run.is_some());
    assert_eq!(state_for_task(&submission, &source), TaskState::CacheHit);
    assert!(
        !submission
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .contains_key(&source)
    );
    assert!(
        !submission
            .dispatch_gap_phases
            .contains(&PipelinePhase::SourceLoad)
    );
    assert!(
        submission
            .dispatch_gap_phases
            .contains(&PipelinePhase::Frontend)
    );
    assert_event(&driver, submission.session.id(), |kind| {
        matches!(kind, BuildEventKind::DispatchGap { .. })
    });
}

#[test]
fn partial_registry_with_extra_or_wrong_service_keeps_missing_service_preflight() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut baseline_driver = CompilerDriver::default();
    let baseline = baseline_driver
        .submit(
            request(83),
            &ids,
            &snapshots,
            submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]),
        )
        .unwrap();
    let snapshot = baseline.session.captured().snapshot().id;
    let publisher = Arc::new(PhaseOutputPublisher::new(
        Arc::new(IrStorageService::new()),
        Arc::new(SnapshotHandleRegistry::new()),
    ));
    publisher.register_current_snapshot(snapshot);
    for wrong_source_descriptor in [false, true] {
        let mut builder = PhaseRegistryBuilder::new();
        if wrong_source_descriptor {
            let source_load = required_phase_services()
                .iter()
                .find(|requirement| requirement.phases.contains(&PipelinePhase::SourceLoad))
                .unwrap();
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    source_load.service_name,
                    source_load.owner,
                    source_load.phases.to_vec(),
                    "wrong-schema-v1",
                    "SourceUnit",
                )
                .unwrap(),
            });
        } else {
            builder.register_source_load();
        }
        builder.register_frontend(Vec::new());
        if !wrong_source_descriptor {
            let module_resolve = required_phase_services()
                .iter()
                .find(|requirement| requirement.phases.contains(&PipelinePhase::ModuleResolve))
                .unwrap();
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    module_resolve.service_name,
                    module_resolve.owner,
                    module_resolve.phases.to_vec(),
                    "driver-fixture-v1",
                    "driver-fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher.clone());
        let submission = driver
            .submit(
                request(83),
                &ids,
                &snapshots,
                submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]),
            )
            .unwrap();
        assert_eq!(submission.session.captured().snapshot().id, snapshot);
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::BlockedByMissingPhaseServices
        );
        assert!(submission.scheduler_run.is_none());
        assert!(
            submission
                .missing_services
                .iter()
                .any(|missing| { missing.phase == PipelinePhase::ArtifactCommit })
        );
    }
}

#[test]
fn registry_dispatch_statuses_map_to_scheduler_outcomes_without_publication() {
    for (status, expected_state, expected_outcome) in [
        (
            PhaseStatus::Recoverable,
            TaskState::Failed,
            BuildSessionOutcome::Failed,
        ),
        (
            PhaseStatus::Blocking,
            TaskState::Blocked,
            BuildSessionOutcome::Blocked,
        ),
        (
            PhaseStatus::Fatal,
            TaskState::Failed,
            BuildSessionOutcome::Failed,
        ),
        (
            PhaseStatus::Cancelled,
            TaskState::Cancelled,
            BuildSessionOutcome::Cancelled,
        ),
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut driver = CompilerDriver::new(executable_scheduler_fixture_registry_with_statuses(
            calls,
            &[(PipelinePhase::ArtifactCommit, status)],
        ));
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        input.phase_dispatch_inputs = Some(Box::new(FixturePhaseInputs));

        let submission = driver
            .submit(request(42), &ids, &snapshots, input)
            .expect("terminal registry status maps through scheduler callback");

        let artifact = task_id_for_phase(&submission, PipelinePhase::ArtifactCommit);
        assert_eq!(state_for_task(&submission, &artifact), expected_state);
        assert_eq!(
            submission.session.state(),
            BuildSessionState::Finished(expected_outcome)
        );
        assert_eq!(
            submission.status,
            DriverSubmissionStatus::SchedulerValidated
        );
        assert!(submission.dispatch_gap_phases.is_empty());
        assert_no_event(&driver, submission.session.id(), |kind| {
            matches!(
                kind,
                BuildEventKind::DiagnosticsReady { .. }
                    | BuildEventKind::ArtifactBoundary { .. }
                    | BuildEventKind::DispatchGap { .. }
            )
        });
    }
}

#[test]
fn recoverable_frontend_diagnostics_do_not_release_semantic_dependents() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut driver = CompilerDriver::new(executable_scheduler_fixture_registry_with_statuses(
        calls.clone(),
        &[(PipelinePhase::Frontend, PhaseStatus::Recoverable)],
    ));
    let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
    input.phase_dispatch_inputs = Some(Box::new(FixturePhaseInputs));
    let submission = driver.submit(request(83), &ids, &snapshots, input).unwrap();

    let frontend = task_id_for_phase(&submission, PipelinePhase::Frontend);
    let module_resolve = task_id_for_phase(&submission, PipelinePhase::ModuleResolve);
    let artifact_commit = task_id_for_phase(&submission, PipelinePhase::ArtifactCommit);
    assert_eq!(state_for_task(&submission, &frontend), TaskState::Failed);
    assert_eq!(
        state_for_task(&submission, &module_resolve),
        TaskState::Blocked
    );
    assert_eq!(
        state_for_task(&submission, &artifact_commit),
        TaskState::Blocked
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Failed)
    );
    let run = submission.scheduler_run.as_ref().unwrap();
    assert!(run.phase_results[&frontend][0].output_refs.is_empty());
    assert!(!run.phase_results.contains_key(&module_resolve));
    assert!(!run.phase_results.contains_key(&artifact_commit));
    let calls = calls.lock().unwrap();
    assert!(calls.iter().any(|service| service == "Frontend"));
    assert!(!calls.iter().any(|service| service == "ModuleResolver"));
    assert!(!calls.iter().any(|service| service == "ArtifactService"));
}

#[test]
fn failed_module_index_submission_is_stored_and_preserves_lane_currentness() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let lane = BuildLaneId::new(60);
    let mut driver = CompilerDriver::default();

    let error = driver
        .submit(
            request_with_lane_generation(6, lane, BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(vec![WorkspaceSourceFile::new("../bad.miz", "../bad.miz")]),
        )
        .expect_err("invalid source layout reports module-index diagnostics");
    let failed_session = match error {
        mizar_driver::driver::DriverSubmitError::ModuleIndex { session, .. } => session,
        other => panic!("expected module-index error, got {other:?}"),
    };
    assert_eq!(
        failed_session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Failed)
    );
    assert_eq!(
        driver
            .session(failed_session.id())
            .map(|session| session.state()),
        Some(BuildSessionState::Finished(BuildSessionOutcome::Failed))
    );

    let older = driver
        .submit(
            request_with_lane_generation(7, lane, BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(Vec::new()),
        )
        .expect("older same-lane request is classified after failed newer bootstrap");

    assert_eq!(
        older.status,
        DriverSubmissionStatus::SupersededBeforeSubmission
    );
    assert_eq!(
        older.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Superseded)
    );
    assert!(older.scheduler_run.is_none());
    match older.publication_decision {
        PublicationDecision::Suppressed(obsolete) => {
            assert!(!obsolete.lane_current);
            assert!(obsolete.request_snapshot_current);
        }
        PublicationDecision::Current => panic!("older request must not become current"),
        _ => panic!("older request must produce a concrete suppression decision"),
    }
    assert_event(&driver, older.session.id(), |kind| {
        matches!(kind, BuildEventKind::PublicationSuppressed)
    });
}

#[test]
fn stale_same_lane_submission_is_suppressed_before_scheduler_submission() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let lane = BuildLaneId::new(40);

    let current = driver
        .submit(
            request_with_lane_generation(4, lane, BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(Vec::new()),
        )
        .expect("newer same-lane request submits");
    let stale = driver
        .submit(
            request_with_lane_generation(5, lane, BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(Vec::new()),
        )
        .expect("stale same-lane request is classified");

    assert_eq!(current.publication_decision, PublicationDecision::Current);
    assert_eq!(
        stale.status,
        DriverSubmissionStatus::SupersededBeforeSubmission
    );
    assert_eq!(
        stale.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Superseded)
    );
    assert!(stale.build_plan.is_none());
    assert!(stale.module_index.is_none());
    assert!(stale.task_graph.is_none());
    assert!(stale.scheduler_run.is_none());
    assert!(stale.dispatch_gap_phases.is_empty());
    match stale.publication_decision {
        PublicationDecision::Suppressed(obsolete) => {
            assert!(!obsolete.lane_current);
            assert!(obsolete.request_snapshot_current);
        }
        PublicationDecision::Current => panic!("stale same-lane request must not publish current"),
        _ => panic!("stale same-lane request must produce a concrete suppression decision"),
    }
    assert_event(&driver, stale.session.id(), |kind| {
        matches!(kind, BuildEventKind::PublicationSuppressed)
    });
}

#[test]
fn missing_phase_services_block_before_scheduler_outputs_are_interpreted() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::new(PhaseRegistry::empty());

    let submission = driver
        .submit(
            request(2),
            &ids,
            &snapshots,
            submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]),
        )
        .expect("missing services are classified, not fatal");

    assert_eq!(
        submission.status,
        DriverSubmissionStatus::BlockedByMissingPhaseServices
    );
    assert!(submission.dispatch_gap_phases.is_empty());
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    assert!(submission.scheduler_run.is_none());
    assert_missing(&submission, PipelinePhase::SourceLoad);
    assert_missing(&submission, PipelinePhase::Frontend);
    assert!(
        submission
            .task_graph
            .as_ref()
            .unwrap()
            .tasks()
            .iter()
            .any(|task| task.phases.contains(&PipelinePhase::SourceLoad))
    );
    assert_event(&driver, submission.session.id(), |kind| {
        matches!(kind, BuildEventKind::PhaseServiceGap { .. })
    });
    assert_no_event(&driver, submission.session.id(), |kind| {
        matches!(
            kind,
            BuildEventKind::DiagnosticsReady { .. } | BuildEventKind::ArtifactBoundary { .. }
        )
    });
}

#[test]
fn scheduler_cancellation_finishes_without_partial_publication() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let mut input = submit_input(Vec::new());
    input.cancellation = CancellationPolicy::default().with_current_snapshot(snapshot(99));

    let submission = driver
        .submit(request(8), &ids, &snapshots, input)
        .expect("phase-0 cancellation submits through mizar-build scheduler");

    assert_eq!(
        submission.status,
        DriverSubmissionStatus::SchedulerValidated
    );
    assert_eq!(
        submission.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Cancelled)
    );
    assert_eq!(
        driver
            .session(submission.session.id())
            .map(|session| session.state()),
        Some(BuildSessionState::Finished(BuildSessionOutcome::Cancelled))
    );
    assert!(submission.scheduler_run.as_ref().is_some_and(|run| {
        run.task_states
            .iter()
            .any(|record| record.state == mizar_build::scheduler::TaskState::Cancelled)
    }));
    assert_event(&driver, submission.session.id(), |kind| {
        matches!(
            kind,
            BuildEventKind::TaskProgress { task }
                if task.state == Some(mizar_build::scheduler::TaskState::Cancelled)
        )
    });
    assert_event(&driver, submission.session.id(), |kind| {
        matches!(
            kind,
            BuildEventKind::SessionFinished {
                outcome: BuildSessionOutcome::Cancelled
            }
        )
    });
    assert_no_event(&driver, submission.session.id(), |kind| {
        matches!(
            kind,
            BuildEventKind::DiagnosticsReady { .. } | BuildEventKind::ArtifactBoundary { .. }
        )
    });
}

#[test]
fn driver_source_does_not_claim_diagnostics_artifact_or_lsp_authority() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/driver.rs"),
    )
    .unwrap();

    for forbidden in [
        "DiagnosticRegistry",
        "DiagnosticCode",
        "DiagnosticRecord",
        "DiagnosticAggregator",
        "render_cli",
        "JsonRpc",
        "LspDiagnostic",
        "DiagnosticSeverity",
        "CodeAction",
        "DocumentUri",
        "DocumentVersion",
        "ProgressToken",
        "PublicationToken",
        "commit_manifest_updates",
        "VerifiedArtifact",
        "mizar_frontend",
        "SyntheticOutputRef",
        "scheduler_run.results",
        "AnyPhaseOutputRef",
        "SchedulerResult",
        "cache_key_for_phase",
        "execute_phase",
    ] {
        assert!(
            !source.contains(forbidden),
            "driver core must not own forbidden authority term {forbidden}"
        );
    }
}

#[test]
fn driver_scheduler_helper_does_not_claim_phase_output_or_cache_authority() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/driver/scheduler.rs"),
    )
    .unwrap();

    assert!(
        source.contains("execute_phase_with_resources"),
        "scheduler helper must consume the registry execution boundary"
    );
    assert!(
        source.contains("invalid_phase_dispatch_input"),
        "scheduler helper must classify invalid owner dispatch inputs separately from missing bundles"
    );
    assert!(
        source.contains("missing_phase_input_identities"),
        "scheduler helper must keep missing owner dispatch bundles classified as dispatch gaps"
    );
    for forbidden in [
        "DiagnosticRegistry",
        "DiagnosticCode",
        "DiagnosticRecord",
        "DiagnosticAggregator",
        "JsonRpc",
        "LspDiagnostic",
        "DiagnosticSeverity",
        "CodeAction",
        "DocumentUri",
        "DocumentVersion",
        "ProgressToken",
        "PublicationToken",
        "commit_manifest_updates",
        "VerifiedArtifact",
        "mizar_frontend",
        "SyntheticOutputRef",
        "scheduler_run.results",
        "AnyPhaseOutputRef",
        "cache_key_for_phase",
        "PhaseCacheIntent",
        "Proof",
        "Trusted",
    ] {
        assert!(
            !source.contains(forbidden),
            "driver scheduler helper must not own forbidden authority term {forbidden}"
        );
    }
}

fn assert_missing(submission: &BuildSubmission, phase: PipelinePhase) {
    assert!(submission.missing_services.iter().any(|missing| {
        missing.phase == phase
            && missing.availability
                == if matches!(phase, PipelinePhase::SourceLoad | PipelinePhase::Frontend) {
                    PhaseServiceAvailability::AvailableOwner
                } else {
                    PhaseServiceAvailability::ExternalDependencyGap
                }
    }));
}

fn task_id_for_phase(submission: &BuildSubmission, phase: PipelinePhase) -> TaskId {
    submission
        .task_graph
        .as_ref()
        .expect("submission has task graph")
        .tasks()
        .iter()
        .find(|task| task.phases.contains(&phase))
        .map(|task| task.id.clone())
        .expect("task for phase exists")
}

fn state_for_task(submission: &BuildSubmission, task: &TaskId) -> TaskState {
    submission
        .scheduler_run
        .as_ref()
        .expect("submission has scheduler run")
        .task_states
        .iter()
        .find(|record| record.task_id == *task)
        .map(|record| record.state)
        .expect("task state exists")
}

fn assert_event(
    driver: &CompilerDriver,
    session: mizar_session::BuildSessionId,
    predicate: impl Fn(&BuildEventKind) -> bool,
) {
    let stream = driver.events(session);
    assert!(stream.known_session);
    assert!(
        stream.events().iter().any(|event| predicate(&event.kind)),
        "expected event was not present in {:#?}",
        stream.events()
    );
}

fn assert_no_event(
    driver: &CompilerDriver,
    session: mizar_session::BuildSessionId,
    predicate: impl Fn(&BuildEventKind) -> bool,
) {
    let stream = driver.events(session);
    assert!(stream.known_session);
    assert!(
        stream.events().iter().all(|event| !predicate(&event.kind)),
        "forbidden event was present in {:#?}",
        stream.events()
    );
}

fn scheduler_fixture_registry() -> PhaseRegistry {
    let mut builder = PhaseRegistryBuilder::new();
    for (index, requirement) in required_phase_services().iter().enumerate() {
        builder.register(DescriptorOnlyFixtureService {
            descriptor: PhaseDescriptor::new(
                requirement.service_name,
                requirement.owner,
                requirement.phases.to_vec(),
                "driver-fixture-v1",
                format!("driver-fixture-output-{index}"),
            )
            .unwrap(),
        });
    }
    builder.build().unwrap()
}

fn executable_scheduler_fixture_registry(calls: Arc<Mutex<Vec<String>>>) -> PhaseRegistry {
    executable_scheduler_fixture_registry_with_statuses(calls, &[])
}

fn executable_scheduler_fixture_registry_with_statuses(
    calls: Arc<Mutex<Vec<String>>>,
    statuses: &[(PipelinePhase, PhaseStatus)],
) -> PhaseRegistry {
    let mut builder = PhaseRegistryBuilder::new();
    for (index, requirement) in required_phase_services().iter().enumerate() {
        let status = requirement
            .phases
            .iter()
            .find_map(|phase| {
                statuses.iter().find_map(|(configured_phase, status)| {
                    (*configured_phase == *phase).then_some(*status)
                })
            })
            .unwrap_or(PhaseStatus::Complete);
        builder.register(ExecutableFixtureService {
            descriptor: PhaseDescriptor::new(
                requirement.service_name,
                requirement.owner,
                requirement.phases.to_vec(),
                "driver-fixture-v1",
                format!("driver-fixture-output-{index}"),
            )
            .unwrap(),
            calls: calls.clone(),
            status,
        });
    }
    builder.build().unwrap()
}

fn submit_input(files: Vec<WorkspaceSourceFile>) -> DriverSubmitInput<StaticSourceLayout> {
    let layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
        package_id: PackageId::new("alpha"),
        files,
    }]);
    let mut input = DriverSubmitInput::new(
        PlanRequest {
            workspace_root: WorkspaceRoot::new("workspace"),
            dependency_selection: DependencySelection::Normal,
            toolchain: ToolchainInfo::new("mizar-evo-test"),
        },
        vec![workspace_package()],
        parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "alpha"
            version = "0.1.0"
            source = { kind = "workspace", path = "alpha" }
            dependencies = []
            "#,
        )
        .expect("fixture lockfile parses"),
        layout,
    );
    input.dependency_overlay = ModuleDependencyOverlay::complete(Vec::new());
    input.completion_order = CompletionOrder::Reverse;
    input
}

fn workspace_package() -> WorkspacePackage {
    WorkspacePackage {
        member_path: "alpha".to_owned(),
        manifest: parse_package_manifest(
            r#"
            [package]
            name = "alpha"
            version = "0.1.0"
            "#,
        )
        .expect("fixture manifest parses"),
    }
}

fn request(seed: u8) -> BuildRequestDraft {
    request_with_lane_generation(
        seed,
        BuildLaneId::new(u64::from(seed)),
        BuildRequestGeneration::new(0),
    )
}

fn request_with_lane_generation(
    seed: u8,
    lane: BuildLaneId,
    generation: BuildRequestGeneration,
) -> BuildRequestDraft {
    BuildRequestDraft {
        lane,
        origin: BuildRequestOrigin::Batch(BatchRequest {
            invocation: BatchInvocation::default(),
        }),
        generation,
        workspace_root: WorkspaceRoot::new("workspace"),
        profile: BuildProfile::new("check"),
        targets: BuildTargets::default(),
        source_inputs: SourceInputSet::default(),
        dependency_inputs: DependencyInputSet::new(
            vec![DependencyArtifactRef {
                artifact: format!("dep-{seed}"),
                content_hash: Hash::from_bytes([seed; Hash::BYTE_LEN]),
            }],
            Hash::from_bytes([seed.wrapping_add(1); Hash::BYTE_LEN]),
            ToolchainInfo::new("mizar-evo-test"),
        ),
        verifier_config: VerifierConfigInput::new(Hash::from_bytes(
            [seed.wrapping_add(2); Hash::BYTE_LEN],
        )),
    }
}

fn snapshot(seed: u8) -> BuildSnapshotId {
    let serialized = format!(
        "mizar-session-build-snapshot-v1:{}",
        format!("{seed:02x}").repeat(Hash::BYTE_LEN)
    );
    BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
}

#[derive(Debug)]
struct DescriptorOnlyFixtureService {
    descriptor: PhaseDescriptor,
}

impl PhaseService for DescriptorOnlyFixtureService {
    fn phase(&self) -> PhaseDescriptor {
        self.descriptor.clone()
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        panic!("driver core must not request phase fixture cache keys")
    }

    fn execute(&self, _input: PhaseInput, _context: PhaseExecutionContext) -> PhaseResult {
        panic!("driver core must not execute phase fixtures")
    }
}

struct FixturePhaseInputs;

impl PhaseDispatchInputProvider<BuildTask> for FixturePhaseInputs {
    fn dispatch_input_for_task(
        &self,
        request: PhaseDispatchInputRequest<'_, BuildTask>,
    ) -> Result<Option<PhaseDispatchInputBundle>, mizar_ir::dispatch_input::DispatchInputError>
    {
        let seed = request
            .task()
            .id
            .as_str()
            .bytes()
            .fold(0_u8, |accumulator, byte| accumulator.wrapping_add(byte));
        Ok(Some(
            PhaseDispatchInputBundle::new(
                request.snapshot(),
                Hash::from_bytes([seed; Hash::BYTE_LEN]),
                Vec::new(),
                vec![fixture_parent_handle(request.snapshot(), seed)],
            )
            .expect("fixture parent bundle is valid"),
        ))
    }
}

struct WrongSnapshotPhaseInputs;

impl PhaseDispatchInputProvider<BuildTask> for WrongSnapshotPhaseInputs {
    fn dispatch_input_for_task(
        &self,
        _request: PhaseDispatchInputRequest<'_, BuildTask>,
    ) -> Result<Option<PhaseDispatchInputBundle>, mizar_ir::dispatch_input::DispatchInputError>
    {
        Ok(Some(PhaseDispatchInputBundle::without_parent_outputs(
            snapshot(0xee),
            Hash::from_bytes([0xee; Hash::BYTE_LEN]),
            Vec::new(),
        )))
    }
}

struct ErrorPhaseInputs;

impl PhaseDispatchInputProvider<BuildTask> for ErrorPhaseInputs {
    fn dispatch_input_for_task(
        &self,
        _request: PhaseDispatchInputRequest<'_, BuildTask>,
    ) -> Result<Option<PhaseDispatchInputBundle>, mizar_ir::dispatch_input::DispatchInputError>
    {
        Err(
            mizar_ir::dispatch_input::DispatchInputError::DispatchSnapshotMismatch {
                expected: snapshot(0xaa),
                actual: snapshot(0xbb),
            },
        )
    }
}

struct NonePhaseInputs;

impl PhaseDispatchInputProvider<BuildTask> for NonePhaseInputs {
    fn dispatch_input_for_task(
        &self,
        _request: PhaseDispatchInputRequest<'_, BuildTask>,
    ) -> Result<Option<PhaseDispatchInputBundle>, mizar_ir::dispatch_input::DispatchInputError>
    {
        Ok(None)
    }
}

fn fixture_parent_handle(snapshot: BuildSnapshotId, seed: u8) -> SealedParentOutputHandle {
    let publisher = PhaseOutputPublisher::new(
        Arc::new(IrStorageService::new()),
        Arc::new(SnapshotHandleRegistry::new()),
    );
    let phase = IrPipelinePhase::new("driver-fixture-parent");
    let work_unit = IrWorkUnit::new(format!("parent-{seed}"));
    let output_kind = OutputKind::new("driver-fixture-parent-output");
    publisher.register_current_snapshot(snapshot);
    publisher.allow_work_unit(AllowedWorkUnit::new(
        phase.clone(),
        output_kind.clone(),
        work_unit.clone(),
    ));
    let payload = format!("parent-{seed}");
    let handle = publisher
        .publish(PublishOutputInput {
            slot: publisher.allocate(
                snapshot,
                phase.clone(),
                work_unit.clone(),
                output_kind.clone(),
                SchemaVersion::new(1),
            ),
            snapshot,
            phase,
            work_unit,
            output_kind,
            schema_version: SchemaVersion::new(1),
            payload: payload.clone(),
            storage_payload: Some(payload.as_bytes().to_vec()),
            canonical_payload: Some(payload.into_bytes()),
            decode: BlobDecoder::new(|bytes| {
                String::from_utf8(bytes.to_vec())
                    .map_err(|error| BlobDecodeError::new(error.to_string()))
            }),
            parents: Vec::new(),
            named_input_hashes: vec![NamedInputHash {
                name: "source".to_owned(),
                domain: "driver-fixture".to_owned(),
                digest: Hash::from_bytes([seed; Hash::BYTE_LEN]),
            }],
            side_tables: IrSideTables::default(),
            origin: OutputOrigin::PackageSource,
            target: PublicationTarget::CurrentPackage,
        })
        .expect("fixture parent output publishes");
    SealedParentOutputHandle::from_current_output(&publisher, snapshot, handle.erase())
        .expect("fixture parent handle validates")
}

#[derive(Debug)]
struct ExecutableFixtureService {
    descriptor: PhaseDescriptor,
    calls: Arc<Mutex<Vec<String>>>,
    status: PhaseStatus,
}

impl PhaseService for ExecutableFixtureService {
    fn phase(&self) -> PhaseDescriptor {
        self.descriptor.clone()
    }

    fn cache_key(&self, _input: &PhaseInput, _context: &PhaseCacheContext) -> PhaseCacheIntent {
        panic!("driver scheduler dispatch must not request phase fixture cache keys")
    }

    fn execute(&self, _input: PhaseInput, context: PhaseExecutionContext) -> PhaseResult {
        assert!(
            !context.parent_outputs.is_empty(),
            "scheduler-selected dispatch should pass sealed parent handles"
        );
        assert!(
            context
                .parent_outputs
                .iter()
                .all(|parent| parent.snapshot() == context.common.snapshot),
            "scheduler-selected dispatch must pass parent handles for the execution snapshot"
        );
        self.calls
            .lock()
            .expect("calls lock is not poisoned")
            .push(self.descriptor.service_name.clone());
        PhaseResult {
            status: self.status,
            ..PhaseResult::complete()
        }
    }
}

// This service tests transport only; it is not a frontend or semantic producer.
struct PublicationFixtureService {
    descriptor: PhaseDescriptor,
    publisher: Arc<PhaseOutputPublisher>,
    mode: &'static str,
    status: PhaseStatus,
}

impl PhaseService for PublicationFixtureService {
    fn phase(&self) -> PhaseDescriptor {
        self.descriptor.clone()
    }

    fn cache_key(&self, _: &PhaseInput, _: &PhaseCacheContext) -> PhaseCacheIntent {
        panic!("publication transport does not decide cache compatibility")
    }

    fn execute(&self, input: PhaseInput, context: PhaseExecutionContext) -> PhaseResult {
        use mizar_diagnostics::{
            failure_record::PipelinePhase as DiagnosticPhase,
            sink::{DiagnosticProducerScope, DiagnosticSink},
        };
        if self.mode.starts_with("foreign") {
            context
                .output_publisher
                .as_ref()
                .unwrap()
                .register_current_snapshot(input.snapshot);
        }
        let publisher = if self.mode.starts_with("foreign") || self.mode == "missing" {
            self.publisher.clone()
        } else {
            let supplied = context.output_publisher.expect("driver supplies publisher");
            assert!(Arc::ptr_eq(&supplied, &self.publisher));
            supplied
        };
        publisher.register_current_snapshot(input.snapshot);
        let phase = IrPipelinePhase::new(&self.descriptor.service_name);
        let kind = OutputKind::new(&self.descriptor.output_kind);
        let unit = IrWorkUnit::new(format!(
            "{:?}/{:?}",
            input.work_unit,
            input.identities().input_hash()
        ));
        publisher.allow_work_unit(AllowedWorkUnit::new(
            phase.clone(),
            kind.clone(),
            unit.clone(),
        ));
        let handle = publisher
            .publish(PublishOutputInput {
                slot: publisher.allocate(
                    input.snapshot,
                    phase.clone(),
                    unit.clone(),
                    kind.clone(),
                    SchemaVersion::new(1),
                ),
                snapshot: input.snapshot,
                phase,
                work_unit: unit,
                output_kind: kind,
                schema_version: SchemaVersion::new(1),
                payload: "transport fixture".to_owned(),
                storage_payload: Some(b"transport fixture".to_vec()),
                canonical_payload: Some(b"transport fixture".to_vec()),
                decode: BlobDecoder::new(|bytes| {
                    String::from_utf8(bytes.to_vec())
                        .map_err(|error| BlobDecodeError::new(error.to_string()))
                }),
                parents: Vec::new(),
                named_input_hashes: vec![NamedInputHash {
                    name: "dispatch".into(),
                    domain: "transport-fixture".into(),
                    digest: input.identities().input_hash(),
                }],
                side_tables: IrSideTables::default(),
                origin: if self.mode == "open" {
                    OutputOrigin::OpenBuffer
                } else {
                    OutputOrigin::PackageSource
                },
                target: if self.mode == "open" {
                    PublicationTarget::InternalOnly
                } else {
                    PublicationTarget::CurrentPackage
                },
            })
            .unwrap();
        if self.mode == "stale"
            || (self.mode == "late_stale"
                && self
                    .descriptor
                    .phases
                    .contains(&PipelinePhase::ArtifactCommit))
        {
            publisher.mark_obsolete(input.snapshot).unwrap();
        }
        if self.mode == "collected" {
            publisher
                .storage()
                .collect(mizar_ir::storage::CollectInput {
                    snapshot: input.snapshot,
                    protected_outputs: Vec::new(),
                });
        }
        let diagnostic_snapshot = if self.mode == "wrong_batch" {
            snapshot(0xef)
        } else {
            input.snapshot
        };
        PhaseResult {
            status: self.status,
            diagnostics: vec![
                DiagnosticSink::new(DiagnosticProducerScope::new(
                    DiagnosticPhase::Build,
                    diagnostic_snapshot,
                    "transport-fixture",
                ))
                .into_batch(),
            ],
            output_refs: vec![handle.erase()],
            cache_observation: None,
            artifact_commit: None,
        }
    }
}

#[test]
fn scheduled_publication_preserves_real_outputs_and_rejects_invalid_results() {
    let mut canonical_results = None;
    for mode in [
        "valid",
        "valid_forward",
        "missing",
        "foreign",
        "foreign_noncomplete",
        "stale",
        "late_stale",
        "collected",
        "open",
        "wrong_batch",
    ] {
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let mut builder = PhaseRegistryBuilder::new();
        for requirement in required_phase_services() {
            builder.register(PublicationFixtureService {
                descriptor: PhaseDescriptor::new(
                    requirement.service_name,
                    requirement.owner,
                    requirement.phases.to_vec(),
                    "transport-v1",
                    requirement.service_name,
                )
                .unwrap(),
                publisher: publisher.clone(),
                mode,
                status: if mode == "foreign_noncomplete" {
                    PhaseStatus::Fatal
                } else {
                    PhaseStatus::Complete
                },
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if mode != "missing" {
            let supplied = if mode.starts_with("foreign") {
                Arc::new(PhaseOutputPublisher::new(
                    Arc::new(IrStorageService::new()),
                    Arc::new(SnapshotHandleRegistry::new()),
                ))
            } else {
                publisher.clone()
            };
            driver = driver.with_output_publisher(supplied);
        }
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        input.phase_dispatch_inputs = Some(Box::new(FixturePhaseInputs));
        if mode == "valid_forward" {
            input.completion_order = CompletionOrder::Canonical;
        }
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let submission = driver
            .submit(
                request_with_lane_generation(
                    81,
                    BuildLaneId::new(81),
                    BuildRequestGeneration::new(1),
                ),
                &ids,
                &snapshots,
                input,
            )
            .unwrap();
        let run = submission.scheduler_run.unwrap();
        if mode.starts_with("valid") {
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
            );
            assert!(run.phase_results.len() > 1);
            let ordered: Vec<_> = run
                .phase_results
                .iter()
                .map(|(task, results)| {
                    let graph_task = submission
                        .task_graph
                        .as_ref()
                        .unwrap()
                        .tasks()
                        .iter()
                        .find(|candidate| &candidate.id == task)
                        .unwrap();
                    let expected: Vec<_> = required_phase_services()
                        .iter()
                        .filter(|requirement| {
                            requirement
                                .phases
                                .iter()
                                .any(|phase| graph_task.phases.contains(phase))
                        })
                        .map(|requirement| requirement.service_name)
                        .collect();
                    let actual: Vec<_> = results
                        .iter()
                        .map(|result| result.output_refs[0].phase().as_str())
                        .collect();
                    assert_eq!(actual, expected, "service order for {task:?}");
                    (
                        task.clone(),
                        results
                            .iter()
                            .map(|result| {
                                (
                                    result.output_refs[0].phase().clone(),
                                    result.output_refs[0].content_hash(),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect();
            if let Some(expected) = &canonical_results {
                assert_eq!(&ordered, expected);
            } else {
                canonical_results = Some(ordered);
            }
            // submit borrows the driver mutably: same-lane supersession is observed
            // at entry, not concurrently in the synchronous dispatch call.
            let mut stale_input =
                submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
            stale_input.phase_dispatch_inputs = Some(Box::new(FixturePhaseInputs));
            let stale = driver
                .submit(request(81), &ids, &snapshots, stale_input)
                .unwrap();
            assert_eq!(
                stale.session.captured().snapshot().id,
                submission.session.captured().snapshot().id
            );
            assert_eq!(
                stale.status,
                DriverSubmissionStatus::SupersededBeforeSubmission
            );
            assert!(matches!(
                stale.publication_decision,
                PublicationDecision::Suppressed(_)
            ));
            assert!(stale.scheduler_run.is_none());
            publisher
                .validate_current_snapshot(submission.session.captured().snapshot().id)
                .unwrap();
            for result in run.phase_results.values().flatten() {
                assert_eq!(result.diagnostics.len(), 1);
                assert_eq!(result.output_refs.len(), 1);
                let output = &result.output_refs[0];
                publisher
                    .validate_current_output(submission.session.captured().snapshot().id, output)
                    .unwrap();
                let typed = publisher
                    .storage()
                    .typed_handle::<String>(output, output.output_kind())
                    .unwrap();
                assert_eq!(
                    &*publisher.storage().get(&typed).unwrap(),
                    "transport fixture"
                );
            }
        } else {
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(BuildSessionOutcome::Failed),
                "{mode}"
            );
            assert!(run.phase_results.is_empty(), "{mode}");
        }
    }
}

#[test]
fn noncomplete_publication_retains_batches_without_outputs() {
    for status in [
        PhaseStatus::Recoverable,
        PhaseStatus::Blocking,
        PhaseStatus::Fatal,
        PhaseStatus::Cancelled,
    ] {
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let mut builder = PhaseRegistryBuilder::new();
        for requirement in required_phase_services() {
            builder.register(PublicationFixtureService {
                descriptor: PhaseDescriptor::new(
                    requirement.service_name,
                    requirement.owner,
                    requirement.phases.to_vec(),
                    "transport-v1",
                    requirement.service_name,
                )
                .unwrap(),
                publisher: publisher.clone(),
                mode: "valid",
                status,
            });
        }
        let mut driver =
            CompilerDriver::new(builder.build().unwrap()).with_output_publisher(publisher);
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        input.phase_dispatch_inputs = Some(Box::new(FixturePhaseInputs));
        let submission = driver
            .submit(
                request(82),
                &InMemorySessionIdAllocator::new(),
                &SnapshotRegistry::new(),
                input,
            )
            .unwrap();
        let run = submission.scheduler_run.unwrap();
        assert_eq!(run.phase_results.len(), 1);
        let result = &run.phase_results.values().next().unwrap()[0];
        assert_eq!(result.status, status);
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.output_refs.is_empty());
    }
}

#[test]
fn scheduled_theorem_resolver_prefix_requires_exact_real_services() {
    let mut real = PhaseRegistryBuilder::new();
    real.register_source_theorem_module_resolver();
    let real = real.build().unwrap();
    let descriptor = real
        .descriptor_for_phase(PipelinePhase::ModuleResolve)
        .unwrap();
    for case in ["real", "spoof", "extra"] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        publisher.register_current_snapshot(session.captured().snapshot().id);
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        if case == "spoof" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: descriptor.clone(),
            });
        } else {
            builder.register_source_theorem_module_resolver();
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let submission = CompilerDriver::new(builder.build().unwrap())
            .with_output_publisher(publisher)
            .submit(
                request(1),
                &ids,
                &snapshots,
                submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]),
            )
            .unwrap();
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::SignatureCollection)
        );
        assert_eq!(submission.scheduler_run.is_some(), case == "real", "{case}");
        if case != "real" {
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
        }
    }
}

#[test]
fn scheduled_theorem_signature_prefix_requires_exact_four_services_and_current_publisher() {
    let mut real = PhaseRegistryBuilder::new();
    real.register_source_theorem_signature_collection();
    let real = real.build().unwrap();
    let descriptor = real
        .descriptor_for_phase(PipelinePhase::SignatureCollection)
        .unwrap();
    for case in [
        "real",
        "spoof",
        "wrong-descriptor",
        "extra",
        "missing-resolver",
        "no-publisher",
        "obsolete",
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let snapshot = session.captured().snapshot().id;
        publisher.register_current_snapshot(snapshot);
        if case == "obsolete" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        if case != "missing-resolver" {
            builder.register_source_theorem_module_resolver();
        }
        if matches!(case, "spoof" | "wrong-descriptor") {
            let mut descriptor = descriptor.clone();
            if case == "wrong-descriptor" {
                descriptor.schema_version = "wrong-schema".into();
            }
            builder.register(DescriptorOnlyFixtureService { descriptor });
        } else {
            builder.register_source_theorem_signature_collection();
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if case != "no-publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let submission = driver
            .submit(
                request(1),
                &ids,
                &snapshots,
                submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]),
            )
            .unwrap();
        assert!(
            !submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::SignatureCollection)
        );
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::TypeChecking)
        );
        assert_eq!(submission.scheduler_run.is_some(), case == "real", "{case}");
        assert_eq!(
            submission.status,
            if case == "real" {
                DriverSubmissionStatus::BlockedByPhaseDispatchGap
            } else {
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            }
        );
    }
}

#[test]
fn scheduled_theorem_type_checking_prefix_requires_real_five_services_and_preserves_provider_priority()
 {
    let mut real = PhaseRegistryBuilder::new();
    real.register_source_theorem_module_resolver()
        .register_source_theorem_signature_collection()
        .register_source_theorem_type_checking();
    let real = real.build().unwrap();
    for case in [
        "real",
        "spoof-resolver",
        "wrong-resolver",
        "missing-resolver",
        "spoof-signature",
        "wrong-signature",
        "missing-signature",
        "spoof-type",
        "wrong-type",
        "missing-type",
        "extra",
        "no-publisher",
        "obsolete",
        "provider-none",
        "provider-error",
        "provider-snapshot",
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let snapshot = session.captured().snapshot().id;
        publisher.register_current_snapshot(snapshot);
        if case == "obsolete" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        for (name, phase) in [
            ("resolver", PipelinePhase::ModuleResolve),
            ("signature", PipelinePhase::SignatureCollection),
            ("type", PipelinePhase::TypeChecking),
        ] {
            if case == format!("missing-{name}") {
                continue;
            }
            if case == format!("spoof-{name}") || case == format!("wrong-{name}") {
                let mut descriptor = real.descriptor_for_phase(phase).unwrap().clone();
                if case.starts_with("wrong-") {
                    descriptor.schema_version = "wrong-schema".into();
                }
                builder.register(DescriptorOnlyFixtureService { descriptor });
            } else {
                match phase {
                    PipelinePhase::ModuleResolve => {
                        builder.register_source_theorem_module_resolver();
                    }
                    PipelinePhase::SignatureCollection => {
                        builder.register_source_theorem_signature_collection();
                    }
                    PipelinePhase::TypeChecking => {
                        builder.register_source_theorem_type_checking();
                    }
                    _ => unreachable!(),
                }
            }
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if case != "no-publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        input.phase_dispatch_inputs = match case {
            "provider-none" => Some(Box::new(NonePhaseInputs)),
            "provider-error" => Some(Box::new(ErrorPhaseInputs)),
            "provider-snapshot" => Some(Box::new(WrongSnapshotPhaseInputs)),
            _ => None,
        };
        let submission = driver.submit(request(1), &ids, &snapshots, input).unwrap();
        let admitted = matches!(
            case,
            "real" | "missing-type" | "provider-none" | "provider-error" | "provider-snapshot"
        );
        assert_eq!(submission.scheduler_run.is_some(), admitted, "{case}");
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::RegistrationResolution)
        );
        if case == "missing-type" {
            assert!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::TypeChecking)
            );
        }
        if !admitted {
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
        } else if case.starts_with("provider-") {
            let run = submission.scheduler_run.as_ref().unwrap();
            assert!(run.phase_results.is_empty());
            assert_eq!(
                submission
                    .dispatch_gap_phases
                    .contains(&PipelinePhase::SourceLoad),
                case == "provider-none"
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if case == "provider-none" {
                    BuildSessionOutcome::Blocked
                } else {
                    BuildSessionOutcome::Failed
                })
            );
        }
    }
}

#[test]
fn scheduled_theorem_registration_resolution_prefix_requires_real_six_services_and_preserves_provider_priority()
 {
    let mut real = PhaseRegistryBuilder::new();
    real.register_source_theorem_module_resolver()
        .register_source_theorem_signature_collection()
        .register_source_theorem_type_checking()
        .register_source_theorem_registration_resolution();
    let real = real.build().unwrap();
    for case in [
        "real",
        "spoof-resolver",
        "wrong-resolver",
        "missing-resolver",
        "spoof-signature",
        "wrong-signature",
        "missing-signature",
        "spoof-type",
        "wrong-type",
        "missing-type",
        "spoof-registration",
        "wrong-registration",
        "missing-registration",
        "extra",
        "no-publisher",
        "obsolete",
        "provider-none",
        "provider-error",
        "provider-snapshot",
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let snapshot = session.captured().snapshot().id;
        publisher.register_current_snapshot(snapshot);
        if case == "obsolete" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        for (name, phase) in [
            ("resolver", PipelinePhase::ModuleResolve),
            ("signature", PipelinePhase::SignatureCollection),
            ("type", PipelinePhase::TypeChecking),
            ("registration", PipelinePhase::RegistrationResolution),
        ] {
            if case == format!("missing-{name}") {
                continue;
            }
            if case == format!("spoof-{name}") || case == format!("wrong-{name}") {
                let mut descriptor = real.descriptor_for_phase(phase).unwrap().clone();
                if case.starts_with("wrong-") {
                    descriptor.schema_version = "wrong-schema".into();
                }
                builder.register(DescriptorOnlyFixtureService { descriptor });
            } else {
                match phase {
                    PipelinePhase::ModuleResolve => {
                        builder.register_source_theorem_module_resolver();
                    }
                    PipelinePhase::SignatureCollection => {
                        builder.register_source_theorem_signature_collection();
                    }
                    PipelinePhase::TypeChecking => {
                        builder.register_source_theorem_type_checking();
                    }
                    PipelinePhase::RegistrationResolution => {
                        builder.register_source_theorem_registration_resolution();
                    }
                    _ => unreachable!(),
                }
            }
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if case != "no-publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        input.phase_dispatch_inputs = match case {
            "provider-none" => Some(Box::new(NonePhaseInputs)),
            "provider-error" => Some(Box::new(ErrorPhaseInputs)),
            "provider-snapshot" => Some(Box::new(WrongSnapshotPhaseInputs)),
            _ => None,
        };
        let submission = driver.submit(request(1), &ids, &snapshots, input).unwrap();
        let admitted = matches!(
            case,
            "real"
                | "missing-registration"
                | "provider-none"
                | "provider-error"
                | "provider-snapshot"
        );
        assert_eq!(submission.scheduler_run.is_some(), admitted, "{case}");
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::OverloadResolution)
        );
        if case == "missing-registration" {
            assert!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::RegistrationResolution)
            );
        }
        if !admitted {
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
        } else if case.starts_with("provider-") {
            let run = submission.scheduler_run.as_ref().unwrap();
            assert!(run.phase_results.is_empty());
            assert_eq!(
                submission
                    .dispatch_gap_phases
                    .contains(&PipelinePhase::SourceLoad),
                case == "provider-none"
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if case == "provider-none" {
                    BuildSessionOutcome::Blocked
                } else {
                    BuildSessionOutcome::Failed
                })
            );
        }
    }
}

#[test]
fn scheduled_theorem_overload_resolution_prefix_requires_real_seven_services_and_preserves_provider_priority()
 {
    let mut real = PhaseRegistryBuilder::new();
    real.register_source_theorem_module_resolver()
        .register_source_theorem_signature_collection()
        .register_source_theorem_type_checking()
        .register_source_theorem_registration_resolution()
        .register_source_theorem_overload_resolution();
    let real = real.build().unwrap();
    for case in [
        "real",
        "spoof-resolver",
        "wrong-resolver",
        "missing-resolver",
        "spoof-signature",
        "wrong-signature",
        "missing-signature",
        "spoof-type",
        "wrong-type",
        "missing-type",
        "spoof-registration",
        "wrong-registration",
        "missing-registration",
        "spoof-overload",
        "wrong-overload",
        "missing-overload",
        "extra",
        "no-publisher",
        "obsolete",
        "provider-none",
        "provider-error",
        "provider-snapshot",
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let snapshot = session.captured().snapshot().id;
        publisher.register_current_snapshot(snapshot);
        if case == "obsolete" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        for (name, phase) in [
            ("resolver", PipelinePhase::ModuleResolve),
            ("signature", PipelinePhase::SignatureCollection),
            ("type", PipelinePhase::TypeChecking),
            ("registration", PipelinePhase::RegistrationResolution),
            ("overload", PipelinePhase::OverloadResolution),
        ] {
            if case == format!("missing-{name}") {
                continue;
            }
            if case == format!("spoof-{name}") || case == format!("wrong-{name}") {
                let mut descriptor = real.descriptor_for_phase(phase).unwrap().clone();
                if case.starts_with("wrong-") {
                    descriptor.schema_version = "wrong-schema".into();
                }
                builder.register(DescriptorOnlyFixtureService { descriptor });
            } else {
                match phase {
                    PipelinePhase::ModuleResolve => {
                        builder.register_source_theorem_module_resolver();
                    }
                    PipelinePhase::SignatureCollection => {
                        builder.register_source_theorem_signature_collection();
                    }
                    PipelinePhase::TypeChecking => {
                        builder.register_source_theorem_type_checking();
                    }
                    PipelinePhase::RegistrationResolution => {
                        builder.register_source_theorem_registration_resolution();
                    }
                    PipelinePhase::OverloadResolution => {
                        builder.register_source_theorem_overload_resolution();
                    }
                    _ => unreachable!(),
                }
            }
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if case != "no-publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        input.phase_dispatch_inputs = match case {
            "provider-none" => Some(Box::new(NonePhaseInputs)),
            "provider-error" => Some(Box::new(ErrorPhaseInputs)),
            "provider-snapshot" => Some(Box::new(WrongSnapshotPhaseInputs)),
            _ => None,
        };
        let submission = driver.submit(request(1), &ids, &snapshots, input).unwrap();
        let admitted = matches!(
            case,
            "real" | "missing-overload" | "provider-none" | "provider-error" | "provider-snapshot"
        );
        assert_eq!(submission.scheduler_run.is_some(), admitted, "{case}");
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::Elaboration)
        );
        if case == "missing-overload" {
            assert!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::OverloadResolution)
            );
        }
        if !admitted {
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
        } else if case.starts_with("provider-") {
            let run = submission.scheduler_run.as_ref().unwrap();
            assert!(run.phase_results.is_empty());
            assert_eq!(
                submission
                    .dispatch_gap_phases
                    .contains(&PipelinePhase::SourceLoad),
                case == "provider-none"
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if case == "provider-none" {
                    BuildSessionOutcome::Blocked
                } else {
                    BuildSessionOutcome::Failed
                })
            );
        }
    }
}

#[test]
fn scheduled_theorem_elaboration_prefix_requires_real_eight_services_and_preserves_provider_priority()
 {
    let mut real = PhaseRegistryBuilder::new();
    real.register_source_theorem_module_resolver()
        .register_source_theorem_signature_collection()
        .register_source_theorem_type_checking()
        .register_source_theorem_registration_resolution()
        .register_source_theorem_overload_resolution()
        .register_source_theorem_elaboration();
    let real = real.build().unwrap();
    for case in [
        "real",
        "spoof-resolver",
        "wrong-resolver",
        "missing-resolver",
        "spoof-signature",
        "wrong-signature",
        "missing-signature",
        "spoof-type",
        "wrong-type",
        "missing-type",
        "spoof-registration",
        "wrong-registration",
        "missing-registration",
        "spoof-overload",
        "wrong-overload",
        "missing-overload",
        "spoof-elaboration",
        "wrong-elaboration",
        "missing-elaboration",
        "extra",
        "no-publisher",
        "obsolete",
        "provider-none",
        "provider-error",
        "provider-snapshot",
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let snapshot = session.captured().snapshot().id;
        publisher.register_current_snapshot(snapshot);
        if case == "obsolete" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        for (name, phase) in [
            ("resolver", PipelinePhase::ModuleResolve),
            ("signature", PipelinePhase::SignatureCollection),
            ("type", PipelinePhase::TypeChecking),
            ("registration", PipelinePhase::RegistrationResolution),
            ("overload", PipelinePhase::OverloadResolution),
            ("elaboration", PipelinePhase::Elaboration),
        ] {
            if case == format!("missing-{name}") {
                continue;
            }
            if case == format!("spoof-{name}") || case == format!("wrong-{name}") {
                let mut descriptor = real.descriptor_for_phase(phase).unwrap().clone();
                if case.starts_with("wrong-") {
                    descriptor.schema_version = "wrong-schema".into();
                }
                builder.register(DescriptorOnlyFixtureService { descriptor });
            } else {
                match phase {
                    PipelinePhase::ModuleResolve => {
                        builder.register_source_theorem_module_resolver();
                    }
                    PipelinePhase::SignatureCollection => {
                        builder.register_source_theorem_signature_collection();
                    }
                    PipelinePhase::TypeChecking => {
                        builder.register_source_theorem_type_checking();
                    }
                    PipelinePhase::RegistrationResolution => {
                        builder.register_source_theorem_registration_resolution();
                    }
                    PipelinePhase::OverloadResolution => {
                        builder.register_source_theorem_overload_resolution();
                    }
                    PipelinePhase::Elaboration => {
                        builder.register_source_theorem_elaboration();
                    }
                    _ => unreachable!(),
                }
            }
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if case != "no-publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        input.phase_dispatch_inputs = match case {
            "provider-none" => Some(Box::new(NonePhaseInputs)),
            "provider-error" => Some(Box::new(ErrorPhaseInputs)),
            "provider-snapshot" => Some(Box::new(WrongSnapshotPhaseInputs)),
            _ => None,
        };
        let submission = driver.submit(request(1), &ids, &snapshots, input).unwrap();
        let admitted = matches!(
            case,
            "real"
                | "missing-elaboration"
                | "provider-none"
                | "provider-error"
                | "provider-snapshot"
        );
        assert_eq!(submission.scheduler_run.is_some(), admitted, "{case}");
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::AlgorithmPreparation)
        );
        if case == "missing-elaboration" {
            assert!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::Elaboration)
            );
        }
        if !admitted {
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
        } else if case.starts_with("provider-") {
            let run = submission.scheduler_run.as_ref().unwrap();
            assert!(run.phase_results.is_empty());
            assert_eq!(
                submission
                    .dispatch_gap_phases
                    .contains(&PipelinePhase::SourceLoad),
                case == "provider-none"
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if case == "provider-none" {
                    BuildSessionOutcome::Blocked
                } else {
                    BuildSessionOutcome::Failed
                })
            );
        }
    }
}

#[test]
fn scheduled_theorem_algorithm_preparation_prefix_requires_real_nine_services_and_preserves_provider_priority()
 {
    let mut real = PhaseRegistryBuilder::new();
    real.register_source_theorem_module_resolver()
        .register_source_theorem_signature_collection()
        .register_source_theorem_type_checking()
        .register_source_theorem_registration_resolution()
        .register_source_theorem_overload_resolution()
        .register_source_theorem_elaboration()
        .register_source_theorem_algorithm_preparation();
    let real = real.build().unwrap();
    for case in [
        "real",
        "spoof-resolver",
        "wrong-resolver",
        "missing-resolver",
        "spoof-signature",
        "wrong-signature",
        "missing-signature",
        "spoof-type",
        "wrong-type",
        "missing-type",
        "spoof-registration",
        "wrong-registration",
        "missing-registration",
        "spoof-overload",
        "wrong-overload",
        "missing-overload",
        "spoof-elaboration",
        "wrong-elaboration",
        "missing-elaboration",
        "spoof-preparation",
        "wrong-preparation",
        "missing-preparation",
        "extra",
        "no-publisher",
        "obsolete",
        "provider-none",
        "provider-error",
        "provider-snapshot",
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let snapshot = session.captured().snapshot().id;
        publisher.register_current_snapshot(snapshot);
        if case == "obsolete" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        for (name, phase) in [
            ("resolver", PipelinePhase::ModuleResolve),
            ("signature", PipelinePhase::SignatureCollection),
            ("type", PipelinePhase::TypeChecking),
            ("registration", PipelinePhase::RegistrationResolution),
            ("overload", PipelinePhase::OverloadResolution),
            ("elaboration", PipelinePhase::Elaboration),
            ("preparation", PipelinePhase::AlgorithmPreparation),
        ] {
            if case == format!("missing-{name}") {
                continue;
            }
            if case == format!("spoof-{name}") || case == format!("wrong-{name}") {
                let mut descriptor = real.descriptor_for_phase(phase).unwrap().clone();
                if case.starts_with("wrong-") {
                    descriptor.schema_version = "wrong-schema".into();
                }
                builder.register(DescriptorOnlyFixtureService { descriptor });
            } else {
                match phase {
                    PipelinePhase::ModuleResolve => {
                        builder.register_source_theorem_module_resolver();
                    }
                    PipelinePhase::SignatureCollection => {
                        builder.register_source_theorem_signature_collection();
                    }
                    PipelinePhase::TypeChecking => {
                        builder.register_source_theorem_type_checking();
                    }
                    PipelinePhase::RegistrationResolution => {
                        builder.register_source_theorem_registration_resolution();
                    }
                    PipelinePhase::OverloadResolution => {
                        builder.register_source_theorem_overload_resolution();
                    }
                    PipelinePhase::Elaboration => {
                        builder.register_source_theorem_elaboration();
                    }
                    PipelinePhase::AlgorithmPreparation => {
                        builder.register_source_theorem_algorithm_preparation();
                    }
                    _ => unreachable!(),
                }
            }
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if case != "no-publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        input.phase_dispatch_inputs = match case {
            "provider-none" => Some(Box::new(NonePhaseInputs)),
            "provider-error" => Some(Box::new(ErrorPhaseInputs)),
            "provider-snapshot" => Some(Box::new(WrongSnapshotPhaseInputs)),
            _ => None,
        };
        let submission = driver.submit(request(1), &ids, &snapshots, input).unwrap();
        let admitted = matches!(
            case,
            "real"
                | "missing-preparation"
                | "provider-none"
                | "provider-error"
                | "provider-snapshot"
        );
        assert_eq!(submission.scheduler_run.is_some(), admitted, "{case}");
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::VcGenerate)
        );
        if case == "missing-preparation" {
            assert!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::AlgorithmPreparation)
            );
        }
        if !admitted {
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
        } else if case.starts_with("provider-") {
            let run = submission.scheduler_run.as_ref().unwrap();
            assert!(run.phase_results.is_empty());
            assert_eq!(
                submission
                    .dispatch_gap_phases
                    .contains(&PipelinePhase::SourceLoad),
                case == "provider-none"
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if case == "provider-none" {
                    BuildSessionOutcome::Blocked
                } else {
                    BuildSessionOutcome::Failed
                })
            );
        }
    }
}

#[test]
fn scheduled_theorem_vc_generation_prefix_requires_real_ten_services_and_preserves_provider_priority()
 {
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcSchemaVersion};
    let generation = GenerationSchemaVersion::new("α\n\0");
    let schema = VcSchemaVersion::new("");
    let mut real = PhaseRegistryBuilder::new();
    real.register_source_theorem_module_resolver()
        .register_source_theorem_signature_collection()
        .register_source_theorem_type_checking()
        .register_source_theorem_registration_resolution()
        .register_source_theorem_overload_resolution()
        .register_source_theorem_elaboration()
        .register_source_theorem_algorithm_preparation()
        .register_source_theorem_vc_generation(generation.clone(), schema.clone());
    let real = real.build().unwrap();
    for case in [
        "real",
        "spoof-resolver",
        "wrong-resolver",
        "missing-resolver",
        "spoof-signature",
        "wrong-signature",
        "missing-signature",
        "spoof-type",
        "wrong-type",
        "missing-type",
        "spoof-registration",
        "wrong-registration",
        "missing-registration",
        "spoof-overload",
        "wrong-overload",
        "missing-overload",
        "spoof-elaboration",
        "wrong-elaboration",
        "missing-elaboration",
        "spoof-preparation",
        "wrong-preparation",
        "missing-preparation",
        "spoof-vc",
        "wrong-vc",
        "missing-vc",
        "extra",
        "no-publisher",
        "obsolete",
        "provider-none",
        "provider-error",
        "provider-snapshot",
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let snapshot = session.captured().snapshot().id;
        publisher.register_current_snapshot(snapshot);
        if case == "obsolete" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        for (name, phase) in [
            ("resolver", PipelinePhase::ModuleResolve),
            ("signature", PipelinePhase::SignatureCollection),
            ("type", PipelinePhase::TypeChecking),
            ("registration", PipelinePhase::RegistrationResolution),
            ("overload", PipelinePhase::OverloadResolution),
            ("elaboration", PipelinePhase::Elaboration),
            ("preparation", PipelinePhase::AlgorithmPreparation),
            ("vc", PipelinePhase::VcGenerate),
        ] {
            if case == format!("missing-{name}") {
                continue;
            }
            if case == format!("spoof-{name}") || case == format!("wrong-{name}") {
                let mut descriptor = real.descriptor_for_phase(phase).unwrap().clone();
                if case.starts_with("wrong-") {
                    descriptor.schema_version = "wrong-schema".into();
                }
                builder.register(DescriptorOnlyFixtureService { descriptor });
            } else {
                match phase {
                    PipelinePhase::ModuleResolve => {
                        builder.register_source_theorem_module_resolver();
                    }
                    PipelinePhase::SignatureCollection => {
                        builder.register_source_theorem_signature_collection();
                    }
                    PipelinePhase::TypeChecking => {
                        builder.register_source_theorem_type_checking();
                    }
                    PipelinePhase::RegistrationResolution => {
                        builder.register_source_theorem_registration_resolution();
                    }
                    PipelinePhase::OverloadResolution => {
                        builder.register_source_theorem_overload_resolution();
                    }
                    PipelinePhase::Elaboration => {
                        builder.register_source_theorem_elaboration();
                    }
                    PipelinePhase::AlgorithmPreparation => {
                        builder.register_source_theorem_algorithm_preparation();
                    }
                    PipelinePhase::VcGenerate => {
                        builder.register_source_theorem_vc_generation(
                            generation.clone(),
                            schema.clone(),
                        );
                    }
                    _ => unreachable!(),
                }
            }
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if case != "no-publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        input.phase_dispatch_inputs = match case {
            "provider-none" => Some(Box::new(NonePhaseInputs)),
            "provider-error" => Some(Box::new(ErrorPhaseInputs)),
            "provider-snapshot" => Some(Box::new(WrongSnapshotPhaseInputs)),
            _ => None,
        };
        let submission = driver.submit(request(1), &ids, &snapshots, input).unwrap();
        let admitted = matches!(
            case,
            "real" | "missing-vc" | "provider-none" | "provider-error" | "provider-snapshot"
        );
        assert_eq!(submission.scheduler_run.is_some(), admitted, "{case}");
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::ArtifactCommit)
        );
        if case == "missing-vc" {
            assert!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::VcGenerate)
            );
        }
        if !admitted {
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
        } else if case.starts_with("provider-") {
            let run = submission.scheduler_run.as_ref().unwrap();
            assert!(run.phase_results.is_empty());
            assert_eq!(
                submission
                    .dispatch_gap_phases
                    .contains(&PipelinePhase::SourceLoad),
                case == "provider-none"
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if case == "provider-none" {
                    BuildSessionOutcome::Blocked
                } else {
                    BuildSessionOutcome::Failed
                })
            );
        }
    }
}

#[test]
fn scheduled_theorem_vc_discharge_prefix_requires_real_eleven_services_and_preserves_provider_priority()
 {
    use mizar_build::{
        module_index::ModuleId,
        task_graph::{TaskKind, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId},
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion},
    };
    let generation = GenerationSchemaVersion::new("α\n\0");
    let schema = VcSchemaVersion::new("");
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("α\n\0"),
            max_steps: u32::MAX,
        },
    };
    let mut real = PhaseRegistryBuilder::new();
    real.register_source_theorem_module_resolver()
        .register_source_theorem_signature_collection()
        .register_source_theorem_type_checking()
        .register_source_theorem_registration_resolution()
        .register_source_theorem_overload_resolution()
        .register_source_theorem_elaboration()
        .register_source_theorem_algorithm_preparation()
        .register_source_theorem_vc_generation(generation.clone(), schema.clone())
        .register_source_theorem_vc_discharge(policy.clone());
    let real = real.build().unwrap();
    for case in [
        "real",
        "spoof-vc",
        "wrong-vc",
        "missing-vc",
        "spoof-discharge",
        "wrong-discharge",
        "missing-discharge",
        "extra",
        "no-publisher",
        "obsolete",
        "provider-none",
        "provider-error",
        "provider-snapshot",
        "duplicate",
        "vacant",
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let snapshot = session.captured().snapshot().id;
        publisher.register_current_snapshot(snapshot);
        if case == "obsolete" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
        let mut builder = PhaseRegistryBuilder::new();
        builder.register_source_load().register_frontend(vec![]);
        for (name, phase) in [
            ("resolver", PipelinePhase::ModuleResolve),
            ("signature", PipelinePhase::SignatureCollection),
            ("type", PipelinePhase::TypeChecking),
            ("registration", PipelinePhase::RegistrationResolution),
            ("overload", PipelinePhase::OverloadResolution),
            ("elaboration", PipelinePhase::Elaboration),
            ("preparation", PipelinePhase::AlgorithmPreparation),
            ("vc", PipelinePhase::VcGenerate),
            ("discharge", PipelinePhase::VcDischarge),
        ] {
            if case == format!("missing-{name}") {
                continue;
            }
            if case == format!("spoof-{name}") || case == format!("wrong-{name}") {
                let mut descriptor = real.descriptor_for_phase(phase).unwrap().clone();
                if case.starts_with("wrong-") {
                    descriptor.schema_version = "wrong-schema".into();
                }
                builder.register(DescriptorOnlyFixtureService { descriptor });
            } else {
                match phase {
                    PipelinePhase::ModuleResolve => {
                        builder.register_source_theorem_module_resolver();
                    }
                    PipelinePhase::SignatureCollection => {
                        builder.register_source_theorem_signature_collection();
                    }
                    PipelinePhase::TypeChecking => {
                        builder.register_source_theorem_type_checking();
                    }
                    PipelinePhase::RegistrationResolution => {
                        builder.register_source_theorem_registration_resolution();
                    }
                    PipelinePhase::OverloadResolution => {
                        builder.register_source_theorem_overload_resolution();
                    }
                    PipelinePhase::Elaboration => {
                        builder.register_source_theorem_elaboration();
                    }
                    PipelinePhase::AlgorithmPreparation => {
                        builder.register_source_theorem_algorithm_preparation();
                    }
                    PipelinePhase::VcGenerate => {
                        builder.register_source_theorem_vc_generation(
                            generation.clone(),
                            schema.clone(),
                        );
                    }
                    PipelinePhase::VcDischarge => {
                        builder.register_source_theorem_vc_discharge(policy.clone());
                    }
                    _ => unreachable!(),
                }
            }
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if case != "no-publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        if case != "vacant" {
            let module = ModuleId::new(
                PackageId::new("alpha"),
                mizar_session::ModulePath::new("main"),
            );
            input.vc_descriptors = (0..if case == "duplicate" { 2 } else { 1 })
                .map(|index| {
                    VcTaskDescriptor::new(
                        VcTaskDescriptorId::new(format!("opaque:{index}")),
                        module.clone(),
                        VcOrderKey::new(format!("{index}")),
                        vec![],
                        vec![],
                    )
                })
                .collect();
        }
        input.phase_dispatch_inputs = match case {
            "provider-none" => Some(Box::new(NonePhaseInputs)),
            "provider-error" => Some(Box::new(ErrorPhaseInputs)),
            "provider-snapshot" => Some(Box::new(WrongSnapshotPhaseInputs)),
            _ => None,
        };
        let submission = driver.submit(request(1), &ids, &snapshots, input).unwrap();
        let admitted = matches!(
            case,
            "real"
                | "missing-discharge"
                | "vacant"
                | "provider-none"
                | "provider-error"
                | "provider-snapshot"
        );
        assert_eq!(submission.scheduler_run.is_some(), admitted, "{case}");
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::ArtifactCommit)
        );
        if case == "missing-discharge" {
            assert!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::VcDischarge)
            );
        }
        if case == "vacant" {
            assert!(
                !submission
                    .task_graph
                    .as_ref()
                    .unwrap()
                    .tasks()
                    .iter()
                    .any(|task| task.kind == TaskKind::VcDischarge)
            );
        }
        if case == "duplicate" {
            assert_eq!(
                submission
                    .task_graph
                    .as_ref()
                    .unwrap()
                    .tasks()
                    .iter()
                    .filter(|task| task.kind == TaskKind::VcDischarge)
                    .count(),
                2
            );
        }
        if !admitted {
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
        } else if case.starts_with("provider-") {
            let run = submission.scheduler_run.as_ref().unwrap();
            assert!(run.phase_results.is_empty());
            assert_eq!(
                submission
                    .dispatch_gap_phases
                    .contains(&PipelinePhase::SourceLoad),
                case == "provider-none"
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if case == "provider-none" {
                    BuildSessionOutcome::Blocked
                } else {
                    BuildSessionOutcome::Failed
                })
            );
        }
    }
}

#[test]
fn scheduled_theorem_atp_solve_prefix_requires_real_twelve_services_and_preserves_provider_priority()
 {
    use mizar_atp::problem::{
        ConcreteFormat, EqualitySupport, LogicFragment, LogicProfile, NativePropertySupport,
        QuantifierPolicy, SoftTypeStrategy,
    };
    use mizar_build::{
        module_index::ModuleId,
        task_graph::{
            BackendProfileId, EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId,
        },
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion},
    };
    let generation = GenerationSchemaVersion::new("α\n\0");
    let schema = VcSchemaVersion::new("");
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("α\n\0"),
            max_steps: u32::MAX,
        },
    };
    let profile = LogicProfile::try_new(
        "α\n\0",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into(),
    )
    .unwrap();
    let mut actual = PhaseRegistryBuilder::new();
    actual
        .register_source_theorem_vc_generation(generation.clone(), schema.clone())
        .register_source_theorem_vc_discharge(policy.clone())
        .register_source_theorem_atp_solve(policy.clone(), profile.clone(), ConcreteFormat::Tptp)
        .unwrap();
    let actual = actual.build().unwrap();
    for case in [
        "real",
        "spoof-atp",
        "wrong-atp",
        "missing-atp",
        "spoof-vc",
        "wrong-discharge",
        "missing-vc",
        "missing-discharge",
        "extra",
        "no-publisher",
        "obsolete",
        "provider-none",
        "provider-error",
        "provider-snapshot",
        "duplicate",
        "vacant",
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let snapshot = session.captured().snapshot().id;
        publisher.register_current_snapshot(snapshot);
        if case == "obsolete" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
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
            .register_source_theorem_algorithm_preparation();
        for (name, phase) in [
            ("vc", PipelinePhase::VcGenerate),
            ("discharge", PipelinePhase::VcDischarge),
            ("atp", PipelinePhase::AtpSolve),
        ] {
            if case == format!("missing-{name}") {
                continue;
            }
            if case == format!("spoof-{name}") || case == format!("wrong-{name}") {
                let mut descriptor = actual.descriptor_for_phase(phase).unwrap().clone();
                if case.starts_with("wrong-") {
                    descriptor.schema_version = "wrong-schema".into();
                }
                builder.register(DescriptorOnlyFixtureService { descriptor });
            } else {
                match phase {
                    PipelinePhase::VcGenerate => {
                        builder.register_source_theorem_vc_generation(
                            generation.clone(),
                            schema.clone(),
                        );
                    }
                    PipelinePhase::VcDischarge => {
                        builder.register_source_theorem_vc_discharge(policy.clone());
                    }
                    PipelinePhase::AtpSolve => {
                        builder
                            .register_source_theorem_atp_solve(
                                policy.clone(),
                                profile.clone(),
                                ConcreteFormat::Tptp,
                            )
                            .unwrap();
                    }
                    _ => unreachable!(),
                }
            }
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if case != "no-publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        if case != "vacant" {
            let module = ModuleId::new(
                PackageId::new("alpha"),
                mizar_session::ModulePath::new("main"),
            );
            input.vc_descriptors = (0..if case == "duplicate" { 2 } else { 1 })
                .map(|index| {
                    VcTaskDescriptor::new(
                        VcTaskDescriptorId::new(format!("opaque:{index}")),
                        module.clone(),
                        VcOrderKey::new(format!("{index}")),
                        vec![BackendProfileId::new("opaque-backend")],
                        vec![EvidenceCandidateId::new("opaque-evidence")],
                    )
                })
                .collect();
        }
        input.phase_dispatch_inputs = match case {
            "provider-none" => Some(Box::new(NonePhaseInputs)),
            "provider-error" => Some(Box::new(ErrorPhaseInputs)),
            "provider-snapshot" => Some(Box::new(WrongSnapshotPhaseInputs)),
            _ => None,
        };
        let submission = driver.submit(request(1), &ids, &snapshots, input).unwrap();
        let admitted = matches!(
            case,
            "real"
                | "missing-atp"
                | "vacant"
                | "provider-none"
                | "provider-error"
                | "provider-snapshot"
        );
        assert_eq!(submission.scheduler_run.is_some(), admitted, "{case}");
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::ArtifactCommit)
        );
        if case == "missing-atp" {
            assert!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::AtpSolve)
            );
        }
        if !admitted {
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
        }
        if case.starts_with("provider-") {
            assert!(
                submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .is_empty()
            );
            assert_eq!(
                submission
                    .dispatch_gap_phases
                    .contains(&PipelinePhase::SourceLoad),
                case == "provider-none"
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if case == "provider-none" {
                    BuildSessionOutcome::Blocked
                } else {
                    BuildSessionOutcome::Failed
                })
            );
        }
    }
}

#[test]
fn scheduled_theorem_candidate_check_prefix_requires_real_thirteen_services_and_preserves_provider_priority()
 {
    use mizar_atp::problem::{
        ConcreteFormat, EqualitySupport, LogicFragment, LogicProfile, NativePropertySupport,
        QuantifierPolicy, SoftTypeStrategy,
    };
    use mizar_build::{
        module_index::ModuleId,
        task_graph::{EvidenceCandidateId, VcOrderKey, VcTaskDescriptor, VcTaskDescriptorId},
    };
    use mizar_vc::{
        discharge::{ComputationLimit, DischargePolicy},
        vc_ir::{GenerationSchemaVersion, PolicyKey, VcSchemaVersion},
    };
    let generation = GenerationSchemaVersion::new("α\n\0");
    let schema = VcSchemaVersion::new("");
    let policy = DischargePolicy {
        computation_limit: ComputationLimit {
            policy: PolicyKey::new("α\n\0"),
            max_steps: u32::MAX,
        },
    };
    let profile = LogicProfile::try_new(
        "α\n\0",
        LogicFragment::Fof,
        EqualitySupport::Supported,
        QuantifierPolicy::FirstOrder,
        SoftTypeStrategy::ScopedGuardPredicates,
        NativePropertySupport::Unsupported,
        [ConcreteFormat::Tptp].into(),
    )
    .unwrap();
    let verifier = mizar_proof::policy::VerifierPolicy::release();
    let mut actual = PhaseRegistryBuilder::new();
    actual
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
    let actual = actual.build().unwrap();
    for case in [
        "real",
        "spoof-kernel",
        "wrong-kernel",
        "missing-kernel",
        "spoof-atp",
        "wrong-atp",
        "missing-atp",
        "spoof-vc",
        "wrong-discharge",
        "missing-vc",
        "missing-discharge",
        "extra",
        "no-publisher",
        "obsolete",
        "provider-none",
        "provider-error",
        "provider-snapshot",
        "duplicate-discharge",
        "duplicate-kernel",
        "vacant",
    ] {
        let ids = InMemorySessionIdAllocator::new();
        let snapshots = SnapshotRegistry::new();
        let session = request(1)
            .allocate(&ids)
            .unwrap()
            .capture_snapshot(&snapshots)
            .unwrap();
        let publisher = Arc::new(PhaseOutputPublisher::new(
            Arc::new(IrStorageService::new()),
            Arc::new(SnapshotHandleRegistry::new()),
        ));
        let snapshot = session.captured().snapshot().id;
        publisher.register_current_snapshot(snapshot);
        if case == "obsolete" {
            publisher.mark_obsolete(snapshot).unwrap();
        }
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
            .register_source_theorem_algorithm_preparation();
        for (name, phase) in [
            ("vc", PipelinePhase::VcGenerate),
            ("discharge", PipelinePhase::VcDischarge),
            ("atp", PipelinePhase::AtpSolve),
            ("kernel", PipelinePhase::KernelCheck),
        ] {
            if case == format!("missing-{name}") {
                continue;
            }
            if case == format!("spoof-{name}") || case == format!("wrong-{name}") {
                let mut descriptor = actual.descriptor_for_phase(phase).unwrap().clone();
                if case.starts_with("wrong-") {
                    descriptor.schema_version = "wrong-schema".into();
                }
                builder.register(DescriptorOnlyFixtureService { descriptor });
            } else {
                match phase {
                    PipelinePhase::VcGenerate => {
                        builder.register_source_theorem_vc_generation(
                            generation.clone(),
                            schema.clone(),
                        );
                    }
                    PipelinePhase::VcDischarge => {
                        builder.register_source_theorem_vc_discharge(policy.clone());
                    }
                    PipelinePhase::AtpSolve => {
                        builder
                            .register_source_theorem_atp_solve(
                                policy.clone(),
                                profile.clone(),
                                ConcreteFormat::Tptp,
                            )
                            .unwrap();
                    }
                    PipelinePhase::KernelCheck => {
                        builder
                            .register_source_theorem_candidate_check(
                                policy.clone(),
                                profile.clone(),
                                ConcreteFormat::Tptp,
                                verifier.clone(),
                            )
                            .unwrap();
                    }
                    _ => unreachable!(),
                }
            }
        }
        if case == "extra" {
            builder.register(DescriptorOnlyFixtureService {
                descriptor: PhaseDescriptor::new(
                    "ExtraPlanner",
                    mizar_driver::registry::PhaseOwner::MizarBuild,
                    vec![PipelinePhase::PackageResolve],
                    "fixture-v1",
                    "fixture-output",
                )
                .unwrap(),
            });
        }
        let mut driver = CompilerDriver::new(builder.build().unwrap());
        if case != "no-publisher" {
            driver = driver.with_output_publisher(publisher);
        }
        let mut input = submit_input(vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")]);
        if case != "vacant" {
            let module = ModuleId::new(
                PackageId::new("alpha"),
                mizar_session::ModulePath::new("main"),
            );
            input.vc_descriptors = (0..if case == "duplicate-discharge" { 2 } else { 1 })
                .map(|index| {
                    VcTaskDescriptor::new(
                        VcTaskDescriptorId::new(format!("opaque:{index}")),
                        module.clone(),
                        VcOrderKey::new(format!("{index}")),
                        vec![],
                        if case == "duplicate-discharge" {
                            vec![]
                        } else if case == "duplicate-kernel" {
                            vec![
                                EvidenceCandidateId::new("opaque-evidence-a"),
                                EvidenceCandidateId::new("opaque-evidence-b"),
                            ]
                        } else {
                            vec![EvidenceCandidateId::new("opaque-evidence")]
                        },
                    )
                })
                .collect();
        }
        input.phase_dispatch_inputs = match case {
            "provider-none" => Some(Box::new(NonePhaseInputs)),
            "provider-error" => Some(Box::new(ErrorPhaseInputs)),
            "provider-snapshot" => Some(Box::new(WrongSnapshotPhaseInputs)),
            _ => None,
        };
        let submission = driver.submit(request(1), &ids, &snapshots, input).unwrap();
        if case.starts_with("duplicate-") {
            let graph = submission.task_graph.as_ref().unwrap();
            for (kind, count) in [
                (
                    mizar_build::task_graph::TaskKind::VcDischarge,
                    if case == "duplicate-discharge" { 2 } else { 1 },
                ),
                (
                    mizar_build::task_graph::TaskKind::KernelCheck,
                    if case == "duplicate-kernel" { 2 } else { 0 },
                ),
            ] {
                assert_eq!(
                    graph
                        .tasks()
                        .iter()
                        .filter(|task| task.kind == kind)
                        .count(),
                    count,
                    "{case}"
                );
            }
        }
        let admitted = matches!(
            case,
            "real"
                | "missing-kernel"
                | "vacant"
                | "provider-none"
                | "provider-error"
                | "provider-snapshot"
        );
        assert_eq!(submission.scheduler_run.is_some(), admitted, "{case}");
        assert!(
            submission
                .missing_services
                .iter()
                .any(|gap| gap.phase == PipelinePhase::ArtifactCommit)
        );
        if case == "missing-kernel" {
            assert!(
                submission
                    .missing_services
                    .iter()
                    .any(|gap| gap.phase == PipelinePhase::KernelCheck)
            );
        }
        if !admitted {
            assert_eq!(
                submission.status,
                DriverSubmissionStatus::BlockedByMissingPhaseServices
            );
        }
        if case.starts_with("provider-") {
            assert!(
                submission
                    .scheduler_run
                    .as_ref()
                    .unwrap()
                    .phase_results
                    .is_empty()
            );
            assert_eq!(
                submission
                    .dispatch_gap_phases
                    .contains(&PipelinePhase::SourceLoad),
                case == "provider-none"
            );
            assert_eq!(
                submission.session.state(),
                BuildSessionState::Finished(if case == "provider-none" {
                    BuildSessionOutcome::Blocked
                } else {
                    BuildSessionOutcome::Failed
                })
            );
        }
    }
}
