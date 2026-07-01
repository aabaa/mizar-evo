use mizar_build::{
    cancel::CancellationPolicy,
    module_index::{StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage},
    planner::{
        DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile, parse_package_manifest,
    },
    scheduler::{CacheSchedulingPolicy, CompletionOrder, SchedulerInput, run_scheduler},
    task_graph::{ModuleDependencyOverlay, PipelinePhase},
};
use mizar_driver::{
    driver::{BuildSubmission, CompilerDriver, DriverSubmissionStatus, DriverSubmitInput},
    events::BuildEventKind,
    registry::{
        PhaseCacheContext, PhaseCacheIntent, PhaseDescriptor, PhaseExecutionContext, PhaseInput,
        PhaseRegistry, PhaseRegistryBuilder, PhaseResult, PhaseService, PhaseServiceAvailability,
        required_phase_services,
    },
    request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildSessionOutcome, BuildSessionState,
        BuildTargets, DependencyInputSet, PublicationDecision, SourceInputSet, VerifierConfigInput,
    },
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
        submission.session.state,
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
    assert_eq!(driver_run.task_states, expected_run.task_states);
    assert_eq!(driver_run.events, expected_run.events);
    assert_eq!(driver_run.diagnostics, expected_run.diagnostics);
    assert_eq!(
        driver
            .session(submission.session.id)
            .map(|session| session.state),
        Some(BuildSessionState::Finished(BuildSessionOutcome::Succeeded))
    );
    let stream = driver.events(submission.session.id);
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
    assert_event(&driver, submission.session.id, |kind| {
        matches!(kind, BuildEventKind::SessionFinished { .. })
    });
    assert_no_event(&driver, submission.session.id, |kind| {
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
    assert!(submission.scheduler_run.is_none());
    assert!(
        submission.task_graph.as_ref().unwrap().tasks().len() > 1,
        "fixture must exercise more than a package root task"
    );
    assert!(
        [
            PipelinePhase::SourceLoad,
            PipelinePhase::Frontend,
            PipelinePhase::ArtifactCommit,
        ]
        .into_iter()
        .all(|phase| submission.dispatch_gap_phases.contains(&phase))
    );
    assert_eq!(
        submission.session.state,
        BuildSessionState::Finished(BuildSessionOutcome::Blocked)
    );
    assert_event(&driver, submission.session.id, |kind| {
        matches!(kind, BuildEventKind::DispatchGap { .. })
    });
    assert_no_event(&driver, submission.session.id, |kind| {
        matches!(
            kind,
            BuildEventKind::TaskProgress { .. }
                | BuildEventKind::DiagnosticsReady { .. }
                | BuildEventKind::ArtifactBoundary { .. }
        )
    });
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
        failed_session.state,
        BuildSessionState::Finished(BuildSessionOutcome::Failed)
    );
    assert_eq!(
        driver
            .session(failed_session.id)
            .map(|session| session.state),
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
        older.session.state,
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
    assert_event(&driver, older.session.id, |kind| {
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
        stale.session.state,
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
    assert_event(&driver, stale.session.id, |kind| {
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
        submission.session.state,
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
    assert_event(&driver, submission.session.id, |kind| {
        matches!(kind, BuildEventKind::PhaseServiceGap { .. })
    });
    assert_no_event(&driver, submission.session.id, |kind| {
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
        submission.session.state,
        BuildSessionState::Finished(BuildSessionOutcome::Cancelled)
    );
    assert_eq!(
        driver
            .session(submission.session.id)
            .map(|session| session.state),
        Some(BuildSessionState::Finished(BuildSessionOutcome::Cancelled))
    );
    assert!(submission.scheduler_run.as_ref().is_some_and(|run| {
        run.task_states
            .iter()
            .any(|record| record.state == mizar_build::scheduler::TaskState::Cancelled)
    }));
    assert_event(&driver, submission.session.id, |kind| {
        matches!(
            kind,
            BuildEventKind::TaskProgress { task }
                if task.state == Some(mizar_build::scheduler::TaskState::Cancelled)
        )
    });
    assert_event(&driver, submission.session.id, |kind| {
        matches!(
            kind,
            BuildEventKind::SessionFinished {
                outcome: BuildSessionOutcome::Cancelled
            }
        )
    });
    assert_no_event(&driver, submission.session.id, |kind| {
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
        ".output_refs",
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

fn assert_missing(submission: &BuildSubmission, phase: PipelinePhase) {
    assert!(submission.missing_services.iter().any(|missing| {
        missing.phase == phase
            && missing.availability == PhaseServiceAvailability::ExternalDependencyGap
    }));
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
