use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use mizar_build::{
    module_index::{StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage},
    planner::{
        DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile, parse_package_manifest,
    },
    scheduler::CompletionOrder,
    task_graph::ModuleDependencyOverlay,
};
use mizar_driver::{
    cli::{CliBatchInput, CliExitCode, CliOutput, CliSnapshotInputs, run_batch},
    driver::{
        BuildSubmission, CompilerDriver, DriverSubmitInput, WatchModeGapOwner, WatchOwnerSeam,
        WatchSnapshotReplacementStatus, WatchSubmitControl,
    },
    events::{BuildEventKind, BuildEventOrderKey, BuildEventStream},
    request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet,
        PublicationDecision, SourceInputSet, VerifierConfigInput, WatchRequest,
    },
};
use mizar_session::{
    BuildSnapshotId, DependencyArtifactRef, Edition, Hash, InMemorySessionIdAllocator, ModulePath,
    NormalizedPath, PackageId, SessionIdAllocator, SnapshotRegistry, SourceOrigin, SourceVersion,
    ToolchainInfo, WorkspaceRoot, normalize_source_path,
};

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn phase_zero_driver_projection_is_invariant_to_worker_controls() {
    let canonical = direct_driver_projection(1, CompletionOrder::Canonical);
    let repeat = direct_driver_projection(1, CompletionOrder::Canonical);
    let parallel_reverse = direct_driver_projection(4, CompletionOrder::Reverse);

    assert_eq!(canonical, repeat);
    assert_eq!(canonical, parallel_reverse);
    assert_eq!(canonical.status, "SchedulerValidated");
    assert_eq!(
        canonical.session_state, "Finished(Succeeded)",
        "phase-zero work should finish without real downstream services"
    );
    assert_eq!(canonical.publication, "Current");
    assert_eq!(
        canonical.task_count, 1,
        "phase-zero driver work has one scheduler root; multi-task work is covered by the deferred dispatch guard"
    );
    assert!(canonical.missing_services.is_empty());
    assert!(canonical.dispatch_gap_phases.is_empty());
    assert!(canonical.scheduler_diagnostics.is_empty());
    assert!(
        canonical
            .events
            .iter()
            .any(|event| event.contains("task_progress:"))
    );
    assert!(
        canonical
            .events
            .iter()
            .all(|event| !event.contains("diagnostics_ready"))
    );
    assert!(
        canonical
            .events
            .iter()
            .all(|event| !event.contains("artifact_boundary"))
    );
    assert_no_unknown_projection(&canonical.events);
}

#[test]
fn multi_task_worker_equivalence_is_deferred_until_real_phase_dispatch_exists() {
    let canonical = deferred_multi_task_projection(1, CompletionOrder::Canonical);
    let repeat = deferred_multi_task_projection(1, CompletionOrder::Canonical);
    let parallel_reverse = deferred_multi_task_projection(4, CompletionOrder::Reverse);

    assert_eq!(canonical, repeat);
    assert_eq!(canonical, parallel_reverse);
    assert_eq!(canonical.status, "BlockedByMissingPhaseServices");
    assert!(
        canonical.task_count > 1,
        "fixture must build source/module tasks before owner gaps block scheduling"
    );
    assert!(canonical.scheduler_events.is_empty());
    assert!(
        canonical
            .missing_services
            .iter()
            .any(|gap| gap.ends_with(":ExternalDependencyGap"))
    );
    assert!(
        canonical
            .events
            .iter()
            .any(|event| event.contains("phase_service_gap:"))
    );
    assert!(
        canonical
            .events
            .iter()
            .all(|event| !event.contains("task_progress:"))
    );
    assert!(
        canonical
            .events
            .iter()
            .all(|event| !event.contains("diagnostics_ready"))
    );
    assert!(
        canonical
            .events
            .iter()
            .all(|event| !event.contains("artifact_boundary"))
    );
    assert_no_unknown_projection(&canonical.events);
}

#[test]
fn cli_success_output_is_byte_stable_across_runs_and_jobs() {
    let single = success_cli_output(120, "1", false);
    let repeat = success_cli_output(120, "1", false);
    let parallel = success_cli_output(120, "4", false);
    let json_single = success_cli_output(121, "1", true);
    let json_parallel = success_cli_output(121, "4", true);

    assert_eq!(single, repeat);
    assert_eq!(single, parallel);
    assert_eq!(json_single, json_parallel);
    assert_eq!(single.exit_code, CliExitCode::Success);
    assert_eq!(json_single.exit_code, CliExitCode::Success);
    assert!(single.stdout.is_empty());
    assert!(json_single.stderr.is_empty());
    assert!(!single.stderr.contains("diagnostics_ready"));
    assert!(!single.stderr.contains("artifact_boundary"));
    assert!(!json_single.stdout.contains("jsonrpc"));
}

#[test]
fn diagnostics_and_unavailable_owner_outputs_are_stable_across_runs_and_jobs() {
    let manifest_single = manifest_diagnostic_output(130, "1");
    let manifest_repeat = manifest_diagnostic_output(130, "1");
    let manifest_parallel = manifest_diagnostic_output(130, "4");
    let manifest_json_single = manifest_diagnostic_json_output(131, "1");
    let manifest_json_parallel = manifest_diagnostic_json_output(131, "4");

    assert_eq!(manifest_single, manifest_repeat);
    assert_eq!(manifest_single, manifest_parallel);
    assert_eq!(manifest_json_single, manifest_json_parallel);
    assert_eq!(manifest_single.exit_code, CliExitCode::BuildFailed);
    assert_eq!(manifest_json_single.exit_code, CliExitCode::BuildFailed);
    assert!(
        manifest_single
            .stderr
            .contains("classification=external_dependency_gap diagnostics=")
    );
    assert_eq!(
        manifest_json_single.stdout,
        "{\"schema_version\":1,\"kind\":\"owner_gap\",\"owner\":\"manifest\",\"classification\":\"external_dependency_gap\",\"diagnostics\":1}\n"
    );

    let missing_services_single = missing_phase_services_output(132, "1");
    let missing_services_repeat = missing_phase_services_output(132, "1");
    let missing_services_parallel = missing_phase_services_output(132, "4");

    assert_eq!(missing_services_single, missing_services_repeat);
    assert_eq!(missing_services_single, missing_services_parallel);
    assert_eq!(
        missing_services_single.exit_code,
        CliExitCode::UnavailableOwner
    );
    assert!(missing_services_single.stderr.contains("phase_service_gap"));
    assert!(
        missing_services_single
            .stderr
            .contains("availability=external_dependency_gap")
    );
    assert!(!missing_services_single.stderr.contains("diagnostics_ready"));
    assert!(!missing_services_single.stderr.contains("artifact_boundary"));

    let missing_services_json_single = missing_phase_services_json_output(133, "1");
    let missing_services_json_repeat = missing_phase_services_json_output(133, "1");
    let missing_services_json_parallel = missing_phase_services_json_output(133, "4");

    assert_eq!(missing_services_json_single, missing_services_json_repeat);
    assert_eq!(missing_services_json_single, missing_services_json_parallel);
    assert_eq!(
        missing_services_json_single.exit_code,
        CliExitCode::UnavailableOwner
    );
    assert!(missing_services_json_single.stderr.is_empty());
    assert!(
        missing_services_json_single
            .stdout
            .contains("\"kind\":\"phase_service_gap\"")
    );
    assert!(
        missing_services_json_single
            .stdout
            .contains("\"availability\":\"external_dependency_gap\"")
    );
    assert!(
        !missing_services_json_single
            .stdout
            .contains("diagnostics_ready")
    );
    assert!(
        !missing_services_json_single
            .stdout
            .contains("artifact_boundary")
    );
}

#[test]
fn superseded_watch_replay_is_deterministic_and_never_current_payload() {
    let first = watch_suppression_projection(140);
    let repeat = watch_suppression_projection(140);

    assert_eq!(first, repeat);
    assert_eq!(first.replacement_status, "ExternalDependencyGap");
    assert!(
        first.gaps.iter().any(|gap| {
            gap == "FileWatcher:ExternalDependencyGap"
                || gap == "LspBridge:ExternalDependencyGap"
                || gap == "IrSnapshotReplacement:ExternalDependencyGap"
        }),
        "watch run should report missing owner seams without fake bridges"
    );
    assert_eq!(first.superseded_state, "Finished(Superseded)");
    assert!(
        first
            .previous_replay_events
            .iter()
            .any(|event| event.contains("publication_suppressed"))
    );
    assert!(
        first
            .previous_replay_events
            .iter()
            .any(|event| event.contains("session_finished:Superseded"))
    );
    assert!(
        first
            .previous_replay_publications
            .iter()
            .all(|publication| publication.starts_with("Suppressed"))
    );
    assert!(
        first
            .previous_replay_events
            .iter()
            .all(|event| !event.contains("diagnostics_ready"))
    );
    assert!(
        first
            .previous_replay_events
            .iter()
            .all(|event| !event.contains("artifact_boundary"))
    );
    assert_no_unknown_projection(&first.previous_replay_events);
}

fn direct_driver_projection(
    worker_count: usize,
    completion_order: CompletionOrder,
) -> DriverProjection {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let submission = driver
        .submit(
            batch_request(100),
            &ids,
            &snapshots,
            phase_zero_submit_input(worker_count, completion_order),
        )
        .expect("phase-zero submission succeeds");
    let stream = driver.events(submission.session.id);
    submission_projection(&submission, &stream)
}

fn deferred_multi_task_projection(
    worker_count: usize,
    completion_order: CompletionOrder,
) -> DeferredDriverProjection {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let submission = driver
        .submit(
            batch_request(101),
            &ids,
            &snapshots,
            source_module_submit_input(worker_count, completion_order),
        )
        .expect("multi-task submission is classified before scheduler dispatch");
    let stream = driver.events(submission.session.id);
    DeferredDriverProjection {
        status: format!("{:?}", submission.status),
        session_state: format!("{:?}", submission.session.state),
        publication: format!("{:?}", submission.publication_decision),
        snapshot: snapshot_key(submission.session.captured.snapshot.id),
        task_count: submission
            .task_graph
            .as_ref()
            .expect("deferred multi-task fixture builds task graph")
            .tasks()
            .len(),
        events: event_projection(&stream),
        scheduler_events: submission
            .scheduler_run
            .as_ref()
            .map(|run| {
                run.events
                    .iter()
                    .map(|event| format!("{event:?}"))
                    .collect()
            })
            .unwrap_or_default(),
        missing_services: submission
            .missing_services
            .iter()
            .map(|gap| format!("{:?}:{:?}", gap.phase, gap.availability))
            .collect(),
        dispatch_gap_phases: submission
            .dispatch_gap_phases
            .iter()
            .map(|phase| format!("{phase:?}"))
            .collect(),
    }
}

fn submission_projection(
    submission: &BuildSubmission,
    stream: &BuildEventStream,
) -> DriverProjection {
    let scheduler = submission
        .scheduler_run
        .as_ref()
        .expect("determinism fixture reaches scheduler");
    DriverProjection {
        status: format!("{:?}", submission.status),
        session_state: format!("{:?}", submission.session.state),
        publication: format!("{:?}", submission.publication_decision),
        snapshot: snapshot_key(submission.session.captured.snapshot.id),
        task_count: submission
            .task_graph
            .as_ref()
            .expect("determinism fixture builds task graph")
            .tasks()
            .len(),
        events: event_projection(stream),
        scheduler_events: scheduler
            .events
            .iter()
            .map(|event| {
                format!(
                    "{:?}:{}:{}:{}",
                    event.kind,
                    event
                        .task_id
                        .as_ref()
                        .map(|task| task.as_str())
                        .unwrap_or("-"),
                    event.order.graph_index,
                    event.order.lifecycle_rank
                )
            })
            .collect(),
        scheduler_task_states: scheduler
            .task_states
            .iter()
            .map(|state| {
                format!(
                    "{}:{:?}:deps={:?}:blocked={:?}:queue={:?}:coverage={:?}",
                    state.task_id.as_str(),
                    state.state,
                    state
                        .dependencies
                        .iter()
                        .map(|task| task.as_str())
                        .collect::<Vec<_>>(),
                    state
                        .blocked_by
                        .iter()
                        .map(|task| task.as_str())
                        .collect::<Vec<_>>(),
                    state.queue,
                    state.dependency_coverage
                )
            })
            .collect(),
        scheduler_diagnostics: scheduler
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{diagnostic:?}"))
            .collect(),
        missing_services: submission
            .missing_services
            .iter()
            .map(|gap| format!("{:?}:{:?}", gap.phase, gap.availability))
            .collect(),
        dispatch_gap_phases: submission
            .dispatch_gap_phases
            .iter()
            .map(|phase| format!("{phase:?}"))
            .collect(),
    }
}

fn event_projection(stream: &BuildEventStream) -> Vec<String> {
    stream
        .events()
        .iter()
        .map(|event| {
            format!(
                "lane={}:generation={}:snapshot={}:publication={}:order={}:kind={}",
                event.identity.lane.get(),
                event.identity.generation.get(),
                event
                    .identity
                    .snapshot
                    .map(snapshot_key)
                    .unwrap_or_else(|| "-".to_owned()),
                publication_name(event.identity.publication),
                order_projection(&event.order),
                event_kind_projection(&event.kind)
            )
        })
        .collect()
}

fn order_projection(order: &BuildEventOrderKey) -> String {
    let scheduler_order = order
        .scheduler_order
        .map(|order| format!("{}:{}", order.graph_index, order.lifecycle_rank))
        .unwrap_or_else(|| "-".to_owned());
    format!(
        "{}:{}:{:?}:{}:{}:{}",
        order.lifecycle_rank,
        scheduler_order,
        order.phase,
        order.work_unit_identity,
        order.owner_order,
        order.kind_rank
    )
}

fn event_kind_projection(kind: &BuildEventKind) -> String {
    match kind {
        BuildEventKind::SessionAccepted => "session_accepted".to_owned(),
        BuildEventKind::SnapshotCaptured => "snapshot_captured".to_owned(),
        BuildEventKind::PlanningReady { status } => format!("planning_ready:{status:?}"),
        BuildEventKind::TaskProgress { task } => format!(
            "task_progress:{}:{:?}:{:?}",
            task.task_id, task.scheduler_event, task.state
        ),
        BuildEventKind::PhaseServiceGap {
            phase,
            availability,
        } => format!("phase_service_gap:{phase:?}:{availability:?}"),
        BuildEventKind::DispatchGap { phases } => format!("dispatch_gap:{phases:?}"),
        BuildEventKind::OwnerReadinessGap {
            owner,
            classification,
        } => format!("owner_gap:{owner:?}:{classification:?}"),
        BuildEventKind::PhaseReady { phase, status } => {
            format!("phase_ready:{phase:?}:{status:?}")
        }
        BuildEventKind::DiagnosticsReady { records } => {
            format!("diagnostics_ready:{:?}:{}", records.owner, records.identity)
        }
        BuildEventKind::ArtifactBoundary { committed } => {
            format!(
                "artifact_boundary:{:?}:{}",
                committed.owner, committed.identity
            )
        }
        BuildEventKind::PublicationSuppressed => "publication_suppressed".to_owned(),
        BuildEventKind::SessionFinished { outcome } => format!("session_finished:{outcome:?}"),
        other => format!("unknown_event:{other:?}"),
    }
}

fn publication_name(publication: PublicationDecision) -> String {
    match publication {
        PublicationDecision::Current => "Current".to_owned(),
        PublicationDecision::Suppressed(obsolete) => format!(
            "Suppressed(lane_current={},request_snapshot_current={})",
            obsolete.lane_current, obsolete.request_snapshot_current
        ),
        other => format!("Unknown({other:?})"),
    }
}

fn assert_no_unknown_projection(events: &[String]) {
    assert!(
        events.iter().all(|event| {
            !event.contains("unknown_event:") && !event.contains("publication=Unknown(")
        }),
        "implemented-seam projection must classify all current event/publication variants: {events:#?}"
    );
}

fn success_cli_output(seed: u8, jobs: &str, json: bool) -> CliOutput {
    let mut args = vec!["mizar", "build", "--workspace", "workspace", "--jobs", jobs];
    if json {
        args.extend(["--message-format", "json"]);
    }
    run_batch(
        args,
        batch_input(Vec::new(), SourceInputSet::default(), seed),
    )
}

fn manifest_diagnostic_output(seed: u8, jobs: &str) -> CliOutput {
    let mut input = batch_input(Vec::new(), SourceInputSet::default(), seed);
    input.package_manifest = "not = [valid".to_owned();
    run_batch(
        ["mizar", "build", "--workspace", "workspace", "--jobs", jobs],
        input,
    )
}

fn manifest_diagnostic_json_output(seed: u8, jobs: &str) -> CliOutput {
    let mut input = batch_input(Vec::new(), SourceInputSet::default(), seed);
    input.package_manifest = "not = [valid".to_owned();
    run_batch(
        [
            "mizar",
            "build",
            "--workspace",
            "workspace",
            "--jobs",
            jobs,
            "--message-format",
            "json",
        ],
        input,
    )
}

fn missing_phase_services_output(seed: u8, jobs: &str) -> CliOutput {
    let source_inputs = SourceInputSet {
        versions: vec![source_version("src/main.miz", "main", seed)],
    };
    run_batch(
        ["mizar", "build", "--workspace", "workspace", "--jobs", jobs],
        batch_input(
            vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
            source_inputs,
            seed.wrapping_add(1),
        ),
    )
}

fn missing_phase_services_json_output(seed: u8, jobs: &str) -> CliOutput {
    let source_inputs = SourceInputSet {
        versions: vec![source_version("src/main.miz", "main", seed)],
    };
    run_batch(
        [
            "mizar",
            "build",
            "--workspace",
            "workspace",
            "--jobs",
            jobs,
            "--message-format",
            "json",
        ],
        batch_input(
            vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
            source_inputs,
            seed.wrapping_add(1),
        ),
    )
}

fn watch_suppression_projection(seed: u8) -> WatchProjection {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let lane = BuildLaneId::new(9100);
    let first = driver
        .submit_watch_change(
            watch_request(seed, lane, BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            phase_zero_submit_input(1, CompletionOrder::Canonical),
            watch_control(None),
        )
        .expect("initial watch submission succeeds");
    let second = driver
        .submit_watch_change(
            watch_request(seed.wrapping_add(1), lane, BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            phase_zero_submit_input(4, CompletionOrder::Reverse),
            watch_control(Some(first.submission.session.id)),
        )
        .expect("superseding watch submission succeeds");

    assert_eq!(
        second.snapshot_replacement.status,
        WatchSnapshotReplacementStatus::ExternalDependencyGap
    );
    assert_gap(&second.gaps, WatchModeGapOwner::FileWatcher);
    assert_gap(&second.gaps, WatchModeGapOwner::LspBridge);
    assert_gap(&second.gaps, WatchModeGapOwner::IrSnapshotReplacement);

    let previous_stream = driver.events(first.submission.session.id);
    assert!(previous_stream.events().iter().all(|event| matches!(
        event.identity.publication,
        PublicationDecision::Suppressed(_)
    )));
    assert!(previous_stream.events().iter().all(|event| !matches!(
        event.kind,
        BuildEventKind::DiagnosticsReady { .. } | BuildEventKind::ArtifactBoundary { .. }
    )));

    WatchProjection {
        replacement_status: format!("{:?}", second.snapshot_replacement.status),
        gaps: second
            .gaps
            .iter()
            .map(|gap| format!("{:?}:{:?}", gap.owner, gap.classification))
            .collect(),
        superseded_state: format!(
            "{:?}",
            second
                .superseded
                .as_ref()
                .expect("previous watch session is superseded")
                .state
        ),
        previous_replay_events: event_projection(&previous_stream),
        previous_replay_publications: previous_stream
            .events()
            .iter()
            .map(|event| publication_name(event.identity.publication))
            .collect(),
    }
}

fn assert_gap(gaps: &[mizar_driver::driver::WatchModeGap], owner: WatchModeGapOwner) {
    assert!(
        gaps.iter().any(|gap| gap.owner == owner),
        "expected watch gap for {owner:?} in {gaps:#?}"
    );
}

fn watch_control(
    previous_session: Option<mizar_session::BuildSessionId>,
) -> WatchSubmitControl<'static> {
    WatchSubmitControl {
        previous_session,
        output_publisher: None,
        file_watcher: WatchOwnerSeam::ExternalDependencyGap,
        lsp_bridge: WatchOwnerSeam::ExternalDependencyGap,
    }
}

fn phase_zero_submit_input(
    worker_count: usize,
    completion_order: CompletionOrder,
) -> DriverSubmitInput<StaticSourceLayout> {
    let mut input = DriverSubmitInput::new(
        PlanRequest {
            workspace_root: WorkspaceRoot::new("workspace"),
            dependency_selection: DependencySelection::Normal,
            toolchain: ToolchainInfo::new("mizar-evo-determinism-test"),
        },
        vec![workspace_package("alpha"), workspace_package("beta")],
        parse_lockfile(
            r#"
            schema_version = 1

            [[package]]
            name = "alpha"
            version = "0.1.0"
            source = { kind = "workspace", path = "alpha" }
            dependencies = []

            [[package]]
            name = "beta"
            version = "0.1.0"
            source = { kind = "workspace", path = "beta" }
            dependencies = []
            "#,
        )
        .expect("fixture lockfile parses"),
        StaticSourceLayout::new(vec![
            WorkspaceSourcePackage {
                package_id: PackageId::new("alpha"),
                files: Vec::new(),
            },
            WorkspaceSourcePackage {
                package_id: PackageId::new("beta"),
                files: Vec::new(),
            },
        ]),
    );
    input.dependency_overlay = ModuleDependencyOverlay::complete(Vec::new());
    input.worker_count = worker_count;
    input.completion_order = completion_order;
    input
}

fn source_module_submit_input(
    worker_count: usize,
    completion_order: CompletionOrder,
) -> DriverSubmitInput<StaticSourceLayout> {
    let mut input = phase_zero_submit_input(worker_count, completion_order);
    input.source_layout = StaticSourceLayout::new(vec![
        WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
        },
        WorkspaceSourcePackage {
            package_id: PackageId::new("beta"),
            files: Vec::new(),
        },
    ]);
    input
}

fn workspace_package(name: &str) -> WorkspacePackage {
    WorkspacePackage {
        member_path: name.to_owned(),
        manifest: parse_package_manifest(&format!(
            r#"
            [package]
            name = "{name}"
            version = "0.1.0"
            "#
        ))
        .expect("fixture manifest parses"),
    }
}

fn batch_request(seed: u8) -> BuildRequestDraft {
    BuildRequestDraft {
        lane: BuildLaneId::new(9000),
        origin: BuildRequestOrigin::Batch(BatchRequest {
            invocation: BatchInvocation::default(),
        }),
        generation: BuildRequestGeneration::new(0),
        workspace_root: WorkspaceRoot::new("workspace"),
        profile: BuildProfile::new("check"),
        targets: BuildTargets::default(),
        source_inputs: SourceInputSet::default(),
        dependency_inputs: dependency_inputs(seed),
        verifier_config: VerifierConfigInput::new(hash(seed.wrapping_add(3))),
    }
}

fn watch_request(
    seed: u8,
    lane: BuildLaneId,
    generation: BuildRequestGeneration,
) -> BuildRequestDraft {
    let source = source_version("src/alpha.miz", "alpha", seed);
    BuildRequestDraft {
        lane,
        origin: BuildRequestOrigin::Watch(WatchRequest {
            watch_root: WorkspaceRoot::new("workspace"),
            changed_paths: vec![source.normalized_path.clone()],
        }),
        generation,
        workspace_root: WorkspaceRoot::new("workspace"),
        profile: BuildProfile::new("check"),
        targets: BuildTargets {
            packages: vec![PackageId::new("alpha")],
            modules: vec![ModulePath::new("alpha")],
        },
        source_inputs: SourceInputSet {
            versions: vec![source],
        },
        dependency_inputs: dependency_inputs(seed),
        verifier_config: VerifierConfigInput::new(hash(seed.wrapping_add(3))),
    }
}

fn batch_input(
    files: Vec<WorkspaceSourceFile>,
    source_inputs: SourceInputSet,
    seed: u8,
) -> CliBatchInput {
    CliBatchInput::new(
        r#"
        [package]
        name = "alpha"
        version = "0.1.0"
        "#,
        r#"
        schema_version = 1

        [[package]]
        name = "alpha"
        version = "0.1.0"
        source = { kind = "workspace", path = "alpha" }
        dependencies = []
        "#,
        "alpha",
        files,
        snapshot_inputs(source_inputs, seed),
    )
}

fn snapshot_inputs(source_inputs: SourceInputSet, seed: u8) -> CliSnapshotInputs {
    CliSnapshotInputs::new(
        source_inputs,
        dependency_inputs(seed),
        VerifierConfigInput::new(hash(seed.wrapping_add(3))),
    )
}

fn dependency_inputs(seed: u8) -> DependencyInputSet {
    DependencyInputSet::new(
        vec![DependencyArtifactRef::new(
            "kernel/base.vo",
            hash(seed.wrapping_add(1)),
        )],
        hash(seed.wrapping_add(2)),
        ToolchainInfo::new("mizar-evo-determinism-test"),
    )
}

fn source_version(path: &str, module: &str, seed: u8) -> SourceVersion {
    let fixture = SourcePathFixture::new();
    fixture.write(path, "");
    SourceVersion {
        source_id: InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(seed))
            .unwrap(),
        package_id: PackageId::new("alpha"),
        module_path: ModulePath::new(module),
        normalized_path: normalized_path(fixture.root(), path),
        source_hash: hash(seed),
        edition: Edition::new("2026"),
        origin: SourceOrigin::Disk,
    }
}

fn snapshot_key(snapshot: BuildSnapshotId) -> String {
    snapshot.to_published_schema_string().unwrap()
}

fn snapshot_id(seed: u8) -> BuildSnapshotId {
    let serialized = format!(
        "mizar-session-build-snapshot-v1:{}",
        format!("{seed:02x}").repeat(Hash::BYTE_LEN)
    );
    BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
}

fn hash(first_byte: u8) -> Hash {
    let mut bytes = [0; Hash::BYTE_LEN];
    bytes[0] = first_byte;
    Hash::from_bytes(bytes)
}

fn normalized_path(root: &Path, path: &str) -> NormalizedPath {
    normalize_source_path(root, Path::new(path)).unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DriverProjection {
    status: String,
    session_state: String,
    publication: String,
    snapshot: String,
    task_count: usize,
    events: Vec<String>,
    scheduler_events: Vec<String>,
    scheduler_task_states: Vec<String>,
    scheduler_diagnostics: Vec<String>,
    missing_services: Vec<String>,
    dispatch_gap_phases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeferredDriverProjection {
    status: String,
    session_state: String,
    publication: String,
    snapshot: String,
    task_count: usize,
    events: Vec<String>,
    scheduler_events: Vec<String>,
    missing_services: Vec<String>,
    dispatch_gap_phases: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WatchProjection {
    replacement_status: String,
    gaps: Vec<String>,
    superseded_state: String,
    previous_replay_events: Vec<String>,
    previous_replay_publications: Vec<String>,
}

struct SourcePathFixture {
    base: PathBuf,
    root: PathBuf,
}

impl SourcePathFixture {
    fn new() -> Self {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "mizar_driver_determinism_source_path_{}_{}",
            std::process::id(),
            id
        ));
        let root = base.join("package");
        fs::create_dir_all(root.join("src")).unwrap();
        Self { base, root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write(&self, path: &str, contents: &str) {
        let path = self.root.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }
}

impl Drop for SourcePathFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}
