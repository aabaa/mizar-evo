use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use mizar_build::{
    module_index::{StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage},
    planner::{
        DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile, parse_package_manifest,
    },
    task_graph::ModuleDependencyOverlay,
};
use mizar_driver::{
    driver::{
        CompilerDriver, DriverSubmitError, DriverSubmitInput, WatchModeGapOwner, WatchOwnerSeam,
        WatchSnapshotReplacementStatus, WatchSubmitControl, WatchSubmitError,
    },
    events::{BuildEventKind, OwnerGapClassification},
    request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildSessionOutcome, BuildSessionState,
        BuildTargets, DependencyInputSet, PublicationDecision, SourceInputSet, VerifierConfigInput,
        WatchRequest,
    },
};
use mizar_ir::{
    identity::{
        NamedInputHash, OutputKind, PipelinePhase as IrPipelinePhase, SnapshotHandleRegistry,
        WorkUnit,
    },
    publisher::{
        AllowedWorkUnit, OutputOrigin, PhaseOutputPublisher, PublicationTarget, PublishError,
        PublishOutputInput,
    },
    storage::{BlobDecodeError, BlobDecoder, IrSideTables, IrStorageService, SchemaVersion},
};
use mizar_session::{
    BuildSnapshotId, DependencyArtifactRef, Edition, Hash, InMemorySessionIdAllocator, ModulePath,
    NormalizedPath, PackageId, SessionIdAllocator, SnapshotRegistry, SourceOrigin, SourceVersion,
    ToolchainInfo, WorkspaceRoot, normalize_source_path,
};

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn watch_change_rebuilds_replaces_snapshot_and_suppresses_previous_replay() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let publisher = publisher();
    let lane = BuildLaneId::new(400);

    let first = driver
        .submit_watch_change(
            watch_request(1, lane, BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, Some(&publisher)),
        )
        .expect("initial watch submission succeeds");
    let first_snapshot = first.submission.session.captured.snapshot.id;
    let retained = publish_text(&publisher, first_snapshot, "old-output");

    assert_eq!(
        first.snapshot_replacement.status,
        WatchSnapshotReplacementStatus::RegisteredInitialSnapshot
    );
    publisher
        .validate_current_snapshot(first_snapshot)
        .expect("first watch snapshot is registered current");

    let second = driver
        .submit_watch_change(
            watch_request(2, lane, BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(first.submission.session.id), Some(&publisher)),
        )
        .expect("changed watch snapshot resubmits");
    let second_snapshot = second.submission.session.captured.snapshot.id;

    assert_ne!(first_snapshot, second_snapshot);
    assert_eq!(
        second.snapshot_replacement.status,
        WatchSnapshotReplacementStatus::Replaced
    );
    assert_eq!(
        second
            .superseded
            .as_ref()
            .expect("previous session is superseded")
            .previous_state,
        BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
    );
    assert_eq!(
        driver
            .session(first.submission.session.id)
            .map(|session| session.state),
        Some(BuildSessionState::Finished(BuildSessionOutcome::Superseded))
    );
    assert_previous_replay_is_superseded(&driver, first.submission.session.id);
    assert_eq!(
        publisher.storage().get(&retained).unwrap().as_ref(),
        "old-output",
        "driver consumes mizar-ir replacement without collecting owner-retained output"
    );
    assert!(matches!(
        publisher.validate_current_snapshot(first_snapshot),
        Err(PublishError::ObsoleteSnapshot { snapshot }) if snapshot == first_snapshot
    ));
    publisher
        .validate_current_snapshot(second_snapshot)
        .expect("replacement snapshot is current");
}

#[test]
fn same_snapshot_watch_generation_supersedes_replay_without_ir_replacement() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let publisher = publisher();
    let lane = BuildLaneId::new(401);

    let first = driver
        .submit_watch_change(
            watch_request(3, lane, BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, Some(&publisher)),
        )
        .expect("initial watch submission succeeds");
    let second = driver
        .submit_watch_change(
            watch_request(3, lane, BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(first.submission.session.id), Some(&publisher)),
        )
        .expect("same snapshot generation resubmits");

    assert_eq!(
        first.submission.session.captured.snapshot.id,
        second.submission.session.captured.snapshot.id
    );
    assert_eq!(
        second.snapshot_replacement.status,
        WatchSnapshotReplacementStatus::SameSnapshot
    );
    assert_previous_replay_is_superseded(&driver, first.submission.session.id);
    publisher
        .validate_current_snapshot(second.submission.session.captured.snapshot.id)
        .expect("same snapshot remains publisher-current");
}

#[test]
fn omitted_previous_session_uses_lane_current_watch_session() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let publisher = publisher();
    let lane = BuildLaneId::new(411);

    let first = driver
        .submit_watch_change(
            watch_request(18, lane, BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, Some(&publisher)),
        )
        .expect("initial watch submission succeeds");
    let second = driver
        .submit_watch_change(
            watch_request(19, lane, BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, Some(&publisher)),
        )
        .expect("omitted previous session is derived from the lane current session");

    assert_eq!(
        second
            .superseded
            .as_ref()
            .expect("lane-current previous session is superseded")
            .session,
        first.submission.session.id
    );
    assert_eq!(
        second.snapshot_replacement.status,
        WatchSnapshotReplacementStatus::Replaced
    );
    assert_previous_replay_is_superseded(&driver, first.submission.session.id);
}

#[test]
fn same_snapshot_replacement_does_not_register_or_revive_publisher_state() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let publisher = publisher();
    let lane = BuildLaneId::new(412);

    let first = driver
        .submit_watch_change(
            watch_request(20, lane, BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, None),
        )
        .expect("initial watch submission succeeds without publisher");
    let second = driver
        .submit_watch_change(
            watch_request(20, lane, BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(first.submission.session.id), Some(&publisher)),
        )
        .expect("same-snapshot watch generation resubmits");
    let snapshot = second.submission.session.captured.snapshot.id;

    assert_eq!(
        second.snapshot_replacement.status,
        WatchSnapshotReplacementStatus::SameSnapshot
    );
    assert!(matches!(
        publisher.validate_current_snapshot(snapshot),
        Err(PublishError::UnknownSnapshot { snapshot: unknown }) if unknown == snapshot
    ));
}

#[test]
fn missing_watch_owner_seams_are_classified_without_fake_watcher_or_publisher() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();

    let submission = driver
        .submit_watch_change(
            watch_request(4, BuildLaneId::new(402), BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            WatchSubmitControl {
                previous_session: None,
                output_publisher: None,
                file_watcher: WatchOwnerSeam::Deferred,
                lsp_bridge: WatchOwnerSeam::Unavailable,
            },
        )
        .expect("missing non-critical owner seams are classified on result");

    assert_eq!(
        submission.snapshot_replacement.status,
        WatchSnapshotReplacementStatus::ExternalDependencyGap
    );
    assert_gap(
        &submission.gaps,
        WatchModeGapOwner::FileWatcher,
        OwnerGapClassification::Deferred,
    );
    assert_gap(
        &submission.gaps,
        WatchModeGapOwner::LspBridge,
        OwnerGapClassification::Unavailable,
    );
    assert_gap(
        &submission.gaps,
        WatchModeGapOwner::IrSnapshotReplacement,
        OwnerGapClassification::ExternalDependencyGap,
    );
}

#[test]
fn captured_submit_errors_supersede_previous_and_record_replacement() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let publisher = publisher();
    let lane = BuildLaneId::new(404);

    let first = driver
        .submit_watch_change(
            watch_request(6, lane, BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, Some(&publisher)),
        )
        .expect("initial watch submission succeeds");
    let first_snapshot = first.submission.session.captured.snapshot.id;

    let error = driver
        .submit_watch_change(
            watch_request(7, lane, BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            invalid_module_index_input(),
            watch_control(Some(first.submission.session.id), Some(&publisher)),
        )
        .expect_err("module-index diagnostics are returned as a captured submit error");
    let WatchSubmitError::Submit(failure) = error else {
        panic!("expected watch submit failure");
    };

    assert!(matches!(
        failure.error.as_ref(),
        DriverSubmitError::ModuleIndex { .. }
    ));
    assert!(failure.superseded.is_some());
    assert_eq!(
        failure
            .snapshot_replacement
            .as_ref()
            .expect("captured error still records replacement")
            .status,
        WatchSnapshotReplacementStatus::Replaced
    );
    assert_previous_replay_is_superseded(&driver, first.submission.session.id);
    assert!(matches!(
        publisher.validate_current_snapshot(first_snapshot),
        Err(PublishError::ObsoleteSnapshot { snapshot }) if snapshot == first_snapshot
    ));
}

#[test]
fn snapshot_capture_failure_does_not_supersede_previous_session() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let publisher = publisher();
    let lane = BuildLaneId::new(405);

    let first = driver
        .submit_watch_change(
            watch_request(8, lane, BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, Some(&publisher)),
        )
        .expect("initial watch submission succeeds");
    let mut invalid = watch_request(9, lane, BuildRequestGeneration::new(1));
    invalid.workspace_root = WorkspaceRoot::new("");

    let error = driver
        .submit_watch_change(
            invalid,
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(first.submission.session.id), Some(&publisher)),
        )
        .expect_err("snapshot capture fails before supersession");
    let WatchSubmitError::Submit(failure) = error else {
        panic!("expected watch submit failure");
    };

    assert!(matches!(
        failure.error.as_ref(),
        DriverSubmitError::SnapshotCapture { .. }
    ));
    assert!(failure.superseded.is_none());
    assert!(failure.snapshot_replacement.is_none());
    assert_eq!(
        driver
            .session(first.submission.session.id)
            .map(|session| session.state),
        Some(BuildSessionState::Finished(BuildSessionOutcome::Succeeded))
    );
    assert_current_success_replay(&driver, first.submission.session.id);
}

#[test]
fn previous_watch_session_validation_rejects_wrong_session_inputs() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();

    let unknown = ids.next_session_id().unwrap();
    let mut unknown_driver = CompilerDriver::default();
    assert!(matches!(
        unknown_driver.submit_watch_change(
            watch_request(10, BuildLaneId::new(406), BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(unknown), None),
        ),
        Err(WatchSubmitError::PreviousSessionUnknown { session }) if session == unknown
    ));

    let mut lane_driver = CompilerDriver::default();
    let previous = lane_driver
        .submit_watch_change(
            watch_request(11, BuildLaneId::new(407), BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, None),
        )
        .expect("previous watch submission succeeds");
    assert!(matches!(
        lane_driver.submit_watch_change(
            watch_request(
                12,
                BuildLaneId::new(408),
                BuildRequestGeneration::new(1),
            ),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(previous.submission.session.id), None),
        ),
        Err(WatchSubmitError::PreviousSessionLaneMismatch { session })
            if session == previous.submission.session.id
    ));

    let mut batch_driver = CompilerDriver::default();
    let batch = batch_driver
        .submit(batch_request(), &ids, &snapshots, submit_input())
        .expect("batch submission succeeds");
    assert!(matches!(
        batch_driver.submit_watch_change(
            watch_request(13, BuildLaneId::new(403), BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(batch.session.id), None),
        ),
        Err(WatchSubmitError::PreviousSessionNotWatch { session }) if session == batch.session.id
    ));

    let mut generation_driver = CompilerDriver::default();
    let previous = generation_driver
        .submit_watch_change(
            watch_request(14, BuildLaneId::new(409), BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, None),
        )
        .expect("previous watch submission succeeds");
    assert!(matches!(
        generation_driver.submit_watch_change(
            watch_request(
                15,
                BuildLaneId::new(409),
                BuildRequestGeneration::new(1),
            ),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(previous.submission.session.id), None),
        ),
        Err(WatchSubmitError::NonMonotonicGeneration { session })
            if session == previous.submission.session.id
    ));

    let mut not_current_driver = CompilerDriver::default();
    let older = not_current_driver
        .submit_watch_change(
            watch_request(21, BuildLaneId::new(413), BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, None),
        )
        .expect("older watch submission succeeds");
    let current = not_current_driver
        .submit_watch_change(
            watch_request(22, BuildLaneId::new(413), BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(older.submission.session.id), None),
        )
        .expect("newer watch submission succeeds");
    assert!(matches!(
        not_current_driver.submit_watch_change(
            watch_request(
                23,
                BuildLaneId::new(413),
                BuildRequestGeneration::new(2),
            ),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(older.submission.session.id), None),
        ),
        Err(WatchSubmitError::PreviousSessionNotCurrent { session, current: lane_current })
            if session == older.submission.session.id && lane_current == current.submission.session.id
    ));

    let mut stale_driver = CompilerDriver::default();
    let current = stale_driver
        .submit_watch_change(
            watch_request(24, BuildLaneId::new(414), BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, None),
        )
        .expect("current watch submission succeeds");
    assert!(matches!(
        stale_driver.submit_watch_change(
            watch_request(
                25,
                BuildLaneId::new(414),
                BuildRequestGeneration::new(0),
            ),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, None),
        ),
        Err(WatchSubmitError::NonMonotonicGeneration { session })
            if session == current.submission.session.id
    ));
}

#[test]
fn publisher_replacement_failure_is_reported_without_fake_output() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let publisher = publisher();
    let lane = BuildLaneId::new(410);

    let first = driver
        .submit_watch_change(
            watch_request(16, lane, BuildRequestGeneration::new(0)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, None),
        )
        .expect("initial watch submission can proceed with missing publisher gap");
    let old_snapshot = first.submission.session.captured.snapshot.id;

    let second = driver
        .submit_watch_change(
            watch_request(17, lane, BuildRequestGeneration::new(1)),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(Some(first.submission.session.id), Some(&publisher)),
        )
        .expect("driver records publisher rejection in watch result");

    assert!(matches!(
        second.snapshot_replacement.status,
        WatchSnapshotReplacementStatus::Failed { ref error }
            if matches!(error.as_ref(), PublishError::UnknownSnapshot { snapshot } if *snapshot == old_snapshot)
    ));
    assert_previous_replay_is_superseded(&driver, first.submission.session.id);
}

#[test]
fn non_watch_requests_are_rejected_before_driver_submission() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();

    let error = driver
        .submit_watch_change(
            batch_request(),
            &ids,
            &snapshots,
            submit_input(),
            watch_control(None, None),
        )
        .expect_err("batch request is not a watch change");

    assert!(matches!(
        error,
        WatchSubmitError::NonWatchRequest { origin: "batch" }
    ));
}

#[test]
fn watch_source_keeps_watcher_lsp_artifact_proof_and_cache_authority_out() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/driver.rs"),
    )
    .unwrap();

    for forbidden in [
        "notify::",
        "RecommendedWatcher",
        "FileSystemWatcher",
        "JsonRpc",
        "DocumentUri",
        "DocumentVersion",
        "PublicationToken",
        "VerifiedArtifact",
        "ProofAccepted",
        "CacheCompatibility",
        "cache_key_for_phase",
        "SyntheticOutputRef",
        ".output_refs",
    ] {
        assert!(
            !source.contains(forbidden),
            "watch orchestration must not own forbidden authority term {forbidden}"
        );
    }
}

fn assert_previous_replay_is_superseded(
    driver: &CompilerDriver,
    session: mizar_session::BuildSessionId,
) {
    let stream = driver.events(session);
    assert!(stream.known_session);
    assert!(
        stream.events().iter().all(|event| matches!(
            event.identity.publication,
            PublicationDecision::Suppressed(_)
        )),
        "superseded watch replay must not keep current publication: {:#?}",
        stream.events()
    );
    assert!(
        stream
            .events()
            .iter()
            .any(|event| { matches!(event.kind, BuildEventKind::PublicationSuppressed) })
    );
    assert!(stream.events().iter().any(|event| {
        matches!(
            event.kind,
            BuildEventKind::SessionFinished {
                outcome: BuildSessionOutcome::Superseded
            }
        )
    }));
    assert!(!stream.events().iter().any(|event| {
        matches!(
            event.kind,
            BuildEventKind::SessionFinished {
                outcome: BuildSessionOutcome::Succeeded
            }
        )
    }));
}

fn assert_current_success_replay(driver: &CompilerDriver, session: mizar_session::BuildSessionId) {
    let stream = driver.events(session);
    assert!(stream.known_session);
    assert!(
        stream
            .events()
            .iter()
            .all(|event| { matches!(event.identity.publication, PublicationDecision::Current) })
    );
    assert!(stream.events().iter().any(|event| {
        matches!(
            event.kind,
            BuildEventKind::SessionFinished {
                outcome: BuildSessionOutcome::Succeeded
            }
        )
    }));
    assert!(
        !stream
            .events()
            .iter()
            .any(|event| matches!(event.kind, BuildEventKind::PublicationSuppressed))
    );
}

fn assert_gap(
    gaps: &[mizar_driver::driver::WatchModeGap],
    owner: WatchModeGapOwner,
    classification: OwnerGapClassification,
) {
    assert!(
        gaps.iter()
            .any(|gap| gap.owner == owner && gap.classification == classification),
        "expected gap {owner:?}/{classification:?} in {gaps:#?}"
    );
}

fn watch_control<'a>(
    previous_session: Option<mizar_session::BuildSessionId>,
    output_publisher: Option<&'a PhaseOutputPublisher>,
) -> WatchSubmitControl<'a> {
    WatchSubmitControl {
        previous_session,
        output_publisher,
        file_watcher: WatchOwnerSeam::OwnerProvided,
        lsp_bridge: WatchOwnerSeam::OwnerProvided,
    }
}

fn submit_input() -> DriverSubmitInput<StaticSourceLayout> {
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
        StaticSourceLayout::new(vec![WorkspaceSourcePackage {
            package_id: PackageId::new("alpha"),
            files: Vec::new(),
        }]),
    );
    input.dependency_overlay = ModuleDependencyOverlay::complete(Vec::new());
    input
}

fn invalid_module_index_input() -> DriverSubmitInput<StaticSourceLayout> {
    let mut input = submit_input();
    input.source_layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
        package_id: PackageId::new("alpha"),
        files: vec![WorkspaceSourceFile::new("../bad.miz", "../bad.miz")],
    }]);
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

fn watch_request(
    seed: u8,
    lane: BuildLaneId,
    generation: BuildRequestGeneration,
) -> BuildRequestDraft {
    BuildRequestDraft {
        lane,
        origin: BuildRequestOrigin::Watch(WatchRequest {
            watch_root: WorkspaceRoot::new("workspace"),
            changed_paths: vec![normalized_path("src/alpha.miz")],
        }),
        generation,
        workspace_root: WorkspaceRoot::new("workspace"),
        profile: BuildProfile::new("check"),
        targets: BuildTargets {
            packages: vec![PackageId::new("alpha")],
            modules: vec![ModulePath::new("alpha")],
        },
        source_inputs: SourceInputSet {
            versions: vec![source_version(seed)],
        },
        dependency_inputs: DependencyInputSet::new(
            vec![DependencyArtifactRef::new("kernel/base.vo", hash(90))],
            hash(91),
            ToolchainInfo::new("mizar-evo-test"),
        ),
        verifier_config: VerifierConfigInput::new(hash(92)),
    }
}

fn batch_request() -> BuildRequestDraft {
    let mut request = watch_request(5, BuildLaneId::new(403), BuildRequestGeneration::new(0));
    request.origin = BuildRequestOrigin::Batch(BatchRequest {
        invocation: BatchInvocation::default(),
    });
    request
}

fn source_version(seed: u8) -> SourceVersion {
    let ids = InMemorySessionIdAllocator::new();
    SourceVersion {
        source_id: ids.next_source_id(snapshot_id(0)).unwrap(),
        package_id: PackageId::new("alpha"),
        module_path: ModulePath::new("alpha"),
        normalized_path: normalized_path("src/alpha.miz"),
        source_hash: hash(seed),
        edition: Edition::new("2026"),
        origin: SourceOrigin::Disk,
    }
}

fn publisher() -> PhaseOutputPublisher {
    let publisher = PhaseOutputPublisher::new(
        Arc::new(IrStorageService::new()),
        Arc::new(SnapshotHandleRegistry::new()),
    );
    publisher.allow_work_unit(AllowedWorkUnit::new(ir_phase(), output_kind(), work_unit()));
    publisher
}

fn publish_text(
    publisher: &PhaseOutputPublisher,
    snapshot: BuildSnapshotId,
    payload: &str,
) -> mizar_ir::storage::PhaseOutputRef<String> {
    publisher
        .publish(PublishOutputInput {
            slot: publisher.allocate(
                snapshot,
                ir_phase(),
                work_unit(),
                output_kind(),
                SchemaVersion::new(1),
            ),
            snapshot,
            phase: ir_phase(),
            work_unit: work_unit(),
            output_kind: output_kind(),
            schema_version: SchemaVersion::new(1),
            payload: payload.to_owned(),
            canonical_payload: Some(payload.as_bytes().to_vec()),
            decode: BlobDecoder::new(|bytes| {
                String::from_utf8(bytes.to_vec())
                    .map_err(|error| BlobDecodeError::new(error.to_string()))
            }),
            parents: Vec::new(),
            named_input_hashes: vec![NamedInputHash {
                name: "source".to_owned(),
                domain: "watch-test".to_owned(),
                digest: hash(31),
            }],
            side_tables: IrSideTables::default(),
            origin: OutputOrigin::PackageSource,
            target: PublicationTarget::CurrentPackage,
        })
        .expect("fixture output publishes through real mizar-ir seam")
}

fn ir_phase() -> IrPipelinePhase {
    IrPipelinePhase::new("watch-fixture-phase")
}

fn output_kind() -> OutputKind {
    OutputKind::new("watch-fixture-output")
}

fn work_unit() -> WorkUnit {
    WorkUnit::new("alpha")
}

fn hash(first_byte: u8) -> Hash {
    let mut bytes = [0; Hash::BYTE_LEN];
    bytes[0] = first_byte;
    Hash::from_bytes(bytes)
}

fn snapshot_id(seed: u8) -> BuildSnapshotId {
    let serialized = format!(
        "mizar-session-build-snapshot-v1:{}",
        format!("{seed:02x}").repeat(Hash::BYTE_LEN)
    );
    BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
}

fn normalized_path(path: &str) -> NormalizedPath {
    let fixture = SourcePathFixture::new();
    fixture.write(path, "");
    normalize_source_path(fixture.root(), Path::new(path)).unwrap()
}

struct SourcePathFixture {
    base: PathBuf,
    root: PathBuf,
}

impl SourcePathFixture {
    fn new() -> Self {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "mizar_driver_watch_source_path_{}_{}",
            std::process::id(),
            id
        ));
        let root = base.join("package");
        std::fs::create_dir_all(root.join("src")).unwrap();
        Self { base, root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write(&self, path: &str, contents: &str) {
        let target = self.root.join(path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(target, contents).unwrap();
    }
}

impl Drop for SourcePathFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}
