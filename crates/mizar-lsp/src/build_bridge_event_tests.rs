use super::full_snapshot_registry_a::{RegistryFixture, RegistryObservingAllocator};
use super::*;
use mizar_build::task_graph::PipelinePhase;
use mizar_driver::{
    driver::SubmitOptions,
    registry::PhaseStatus,
    request::{BuildSessionOutcome, BuildSessionState},
};
use mizar_driver::{
    driver::{CompilerDriver, DriverSubmitError, WatchOwnerSeam, WatchSubmitControl},
    events::BuildEventKind,
    request::{BuildLaneId, BuildRequestGeneration, BuildRequestOrigin, WatchRequest},
};
use mizar_session::{SnapshotRegistry, SourceOrigin};
use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

fn bridge_clock(fixture: &mut RegistryFixture, lane: u64) -> (Arc<AtomicU64>, Instant) {
    let millis = Arc::new(AtomicU64::new(0));
    let base = Instant::now();
    let clock = millis.clone();
    fixture.server = std::mem::take(&mut fixture.server)
        .with_build_bridge(BuildLaneId::new(lane), move || {
            base + Duration::from_millis(clock.load(Ordering::SeqCst))
        });
    (millis, base)
}

fn change_version(server: &mut Server, uri: &str, version: i32) {
    server.handle_message(serde_json::json!({
        "jsonrpc":"2.0", "method":"textDocument/didChange",
        "params":{"textDocument":{"uri":uri,"version":version},"contentChanges":[]}
    }));
}

fn open_document(server: &mut Server, uri: &str, version: i32, text: String) {
    server.handle_message(serde_json::json!({
        "jsonrpc":"2.0", "method":"textDocument/didOpen",
        "params":{"textDocument":{"uri":uri,"languageId":"mizar","version":version,"text":text}}
    }));
}

#[allow(clippy::type_complexity)] // Reason: mirror the bridge's preparation tuple without another type.
fn prepare_parts(
    fixture: &RegistryFixture,
) -> (
    mizar_driver::request::BuildRequestDraft,
    mizar_build::module_index::ModuleIndex,
    Vec<(
        mizar_session::DiskSourceLoader,
        mizar_session::PackageId,
        String,
    )>,
    mizar_driver::driver::DriverSubmitInput<mizar_build::module_index::StaticSourceLayout>,
) {
    (
        fixture.draft.clone(),
        fixture.index.clone(),
        fixture.sources.clone(),
        fixture.driver_input(),
    )
}

fn configure_nonterminal_bridge(
    fixture: &mut RegistryFixture,
    lane: u64,
) -> (Arc<AtomicU64>, Instant) {
    drop(fixture.capture());
    let millis = Arc::new(AtomicU64::new(0));
    let clock = millis.clone();
    let base = Instant::now();
    fixture.server = std::mem::take(&mut fixture.server)
        .with_nonterminal_build_bridge(BuildLaneId::new(lane), move || {
            base + Duration::from_millis(clock.load(Ordering::SeqCst))
        });
    (millis, base)
}

fn pause_at_frontend(
    fixture: &mut RegistryFixture,
    session: mizar_session::BuildSessionId,
    task_count: usize,
) {
    for _ in 0..=task_count {
        if fixture
            .driver
            .pending_task(session)
            .is_some_and(|task| task.phases == [PipelinePhase::Frontend])
        {
            return;
        }
        assert!(
            fixture
                .server
                .progress_build_bridge(&mut fixture.driver, &fixture.ids, &fixture.snapshots)
                .unwrap()
                .is_empty()
        );
    }
    assert!(
        fixture
            .driver
            .pending_task(session)
            .is_some_and(|task| task.phases == [PipelinePhase::Frontend]),
        "native execution must pause at its admitted Frontend task"
    );
}

fn flush_bridge(
    fixture: &mut RegistryFixture,
    publisher: Arc<mizar_ir::publisher::PhaseOutputPublisher>,
) -> Result<Option<mizar_driver::driver::BuildSubmission>, BuildBridgeError> {
    let (draft, index, sources, input) = prepare_parts(fixture);
    fixture
        .server
        .drive_build_bridge(
            None,
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &publisher,
            move |_, generation| {
                let mut draft = draft;
                draft.generation = generation;
                Ok((draft, index, sources, input))
            },
        )
        .1
}

fn drain_bridge_session(
    fixture: &mut RegistryFixture,
    session: mizar_session::BuildSessionId,
    task_count: usize,
) -> Option<mizar_driver::driver::BuildSubmission> {
    let mut terminal = None;
    for _ in 0..=task_count {
        terminal = fixture
            .server
            .progress_build_bridge(&mut fixture.driver, &fixture.ids, &fixture.snapshots)
            .unwrap()
            .into_iter()
            .find(|submission| submission.session.id() == session)
            .or(terminal);
        if terminal.is_some() {
            break;
        }
    }
    terminal
}

#[test]
fn nonterminal_host_progress_preserves_native_capture_and_terminal_replay() {
    use mizar_build::task_graph::PipelinePhase;
    use mizar_driver::{registry::PhaseStatus, request::BuildSessionState};

    let mut fixture = RegistryFixture::new();
    drop(fixture.capture()); // Configure the genuine producer's existing publication rights.
    let now = Arc::new(AtomicU64::new(0));
    let clock = now.clone();
    let base = Instant::now();
    fixture.server = std::mem::take(&mut fixture.server)
        .with_nonterminal_build_bridge(BuildLaneId::new(79), move || {
            base + Duration::from_millis(clock.load(Ordering::SeqCst))
        });
    let uri = fixture.sources[0].2.clone();
    change_version(&mut fixture.server, &uri, 2);
    now.store(300, Ordering::SeqCst);
    let (draft, index, sources, input) = prepare_parts(&fixture);
    let (_, started) = fixture.server.drive_build_bridge(
        None,
        &mut fixture.driver,
        &fixture.ids,
        &fixture.snapshots,
        &fixture.publisher,
        move |_, _| Ok((draft, index, sources, input)),
    );
    let started = started.unwrap().unwrap();
    let id = started.session.id();
    let task_count = started.task_graph.as_ref().unwrap().tasks().len();
    assert_eq!(started.session.state(), BuildSessionState::Running);
    let accepted = fixture.server.build_bridge_events(&fixture.driver);
    let original = accepted.first().unwrap().1.clone();
    assert!(
        accepted
            .iter()
            .all(|(event, _)| !matches!(event.kind, BuildEventKind::SessionFinished { .. }))
    );
    assert_eq!(
        fixture.server.next_build_deadline(&fixture.driver),
        Some(base + Duration::from_millis(300))
    );

    // Host scheduling interleaves real task progress with ordinary protocol traffic.
    for _ in 0..=task_count {
        if fixture
            .driver
            .pending_task(id)
            .is_some_and(|task| task.phases == [PipelinePhase::Frontend])
        {
            break;
        }
        assert!(
            fixture
                .server
                .progress_build_bridge(&mut fixture.driver, &fixture.ids, &fixture.snapshots)
                .unwrap()
                .is_empty()
        );
        assert!(!fixture.driver.session(id).unwrap().is_terminal());
    }
    assert!(
        fixture
            .driver
            .pending_task(id)
            .is_some_and(|task| task.phases == [PipelinePhase::Frontend])
    );
    let (response, no_flush) = fixture.server.drive_build_bridge::<_, mizar_build::module_index::StaticSourceLayout, _>(
        Some(serde_json::json!({"jsonrpc":"2.0","id":79,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":0}}})),
        &mut fixture.driver,
        &fixture.ids,
        &fixture.snapshots,
        &fixture.publisher,
        |_, _| panic!("protocol request must not capture"),
    );
    assert_eq!(response.unwrap()["id"], 79);
    assert!(no_flush.unwrap().is_none());
    assert!(!fixture.driver.session(id).unwrap().is_terminal());
    let terminal = (0..=task_count)
        .find_map(|_| {
            let mut finished = fixture
                .server
                .progress_build_bridge(&mut fixture.driver, &fixture.ids, &fixture.snapshots)
                .unwrap();
            finished.pop()
        })
        .expect("native execution must drain within its task count");
    assert!(terminal.session.is_terminal());
    assert!(
        terminal
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .any(|result| result.status == PhaseStatus::Complete && !result.output_refs.is_empty())
    );
    assert!(terminal.diagnostic_index.is_some());
    let replay = fixture.server.build_bridge_events(&fixture.driver);
    assert!(
        replay
            .iter()
            .all(|(_, capture)| Arc::ptr_eq(capture, &original))
    );
    assert_eq!(
        replay
            .iter()
            .filter(|(event, _)| matches!(event.kind, BuildEventKind::SessionFinished { .. }))
            .count(),
        1
    );
    assert!(
        replay
            .iter()
            .any(|(event, _)| matches!(event.kind, BuildEventKind::SnapshotPublished { .. }))
    );
    assert!(
        fixture
            .server
            .progress_build_bridge(&mut fixture.driver, &fixture.ids, &fixture.snapshots)
            .unwrap()
            .is_empty()
    );
    assert!(
        fixture
            .server
            .next_build_deadline(&fixture.driver)
            .is_none()
    );
}

#[test]
fn capture_failure_consumes_its_generation_and_keeps_the_previous_session_and_capture() {
    let mut fixture = RegistryFixture::new();
    let (millis, _) = configure_nonterminal_bridge(&mut fixture, 71);
    let uri = fixture.sources[0].2.clone();
    change_version(&mut fixture.server, &uri, 2);
    millis.store(300, Ordering::SeqCst);
    let publisher = fixture.publisher.clone();
    let first = flush_bridge(&mut fixture, publisher).unwrap().unwrap();
    let first_id = first.session.id();
    let task_count = first.task_graph.as_ref().unwrap().tasks().len();
    assert_eq!(first.session.state(), BuildSessionState::Running);
    pause_at_frontend(&mut fixture, first_id, task_count);
    let original_events = fixture.server.build_bridge_events(&fixture.driver);
    assert!(!original_events.is_empty());
    let original_association = original_events[0].1.clone();
    let original_event_count = original_events.len();
    assert_eq!(
        fixture.driver.session(first_id).unwrap().state(),
        BuildSessionState::Running
    );

    change_version(&mut fixture.server, &uri, 3);
    millis.store(600, Ordering::SeqCst);
    let (draft, index, mut sources, input) = prepare_parts(&fixture);
    sources.pop(); // A partial coordinate set is rejected before native capture.
    let (_, failure) = fixture.server.drive_build_bridge(
        None,
        &mut fixture.driver,
        &fixture.ids,
        &fixture.snapshots,
        &fixture.publisher,
        move |_, generation| {
            let mut draft = draft;
            draft.generation = generation;
            Ok((draft, index, sources, input))
        },
    );
    assert!(matches!(failure, Err(BuildBridgeError::Capture(_))));
    assert_eq!(
        fixture.driver.session(first_id).unwrap().state(),
        BuildSessionState::Running
    );
    let (_, retry) = fixture
        .server
        .drive_build_bridge::<_, mizar_build::module_index::StaticSourceLayout, _>(
            None,
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &fixture.publisher,
            |_, _| panic!("consumed capture slot was retried"),
        );
    assert!(retry.unwrap().is_none());
    let after_failure = fixture.server.build_bridge_events(&fixture.driver);
    assert_eq!(after_failure.len(), original_event_count);
    assert!(Arc::ptr_eq(&original_association, &after_failure[0].1));
    assert_eq!(
        fixture.driver.session(first_id).unwrap().state(),
        BuildSessionState::Running
    );

    change_version(&mut fixture.server, &uri, 4);
    millis.store(900, Ordering::SeqCst);
    let (draft, index, sources, input) = prepare_parts(&fixture);
    let (_, next) = fixture.server.drive_build_bridge(
        None,
        &mut fixture.driver,
        &fixture.ids,
        &fixture.snapshots,
        &fixture.publisher,
        move |_, generation| {
            let mut draft = draft;
            draft.generation = generation;
            Ok((draft, index, sources, input))
        },
    );
    let next = next.unwrap().unwrap();
    assert!(next.session.id() != first_id);
    assert_eq!(next.session.state(), BuildSessionState::Running);
    assert_eq!(
        fixture.driver.session(first_id).unwrap().state(),
        BuildSessionState::Cancelling
    );
    let replay = fixture.server.build_bridge_events(&fixture.driver);
    assert!(
        replay
            .iter()
            .any(|(event, _)| event.identity.generation == BuildRequestGeneration::new(1))
    );
    assert!(
        replay
            .iter()
            .any(|(event, _)| event.identity.generation == BuildRequestGeneration::new(3))
    );
    assert!(
        !replay
            .iter()
            .any(|(event, _)| event.identity.generation == BuildRequestGeneration::new(2))
    );
    let retained = replay
        .iter()
        .find(|(event, _)| event.identity.generation == BuildRequestGeneration::new(1))
        .unwrap();
    assert!(Arc::ptr_eq(&original_association, &retained.1));
    assert!(
        fixture
            .driver
            .session(retained.0.identity.session)
            .is_some()
    );
    assert!(
        drain_bridge_session(&mut fixture, first_id, task_count).is_some(),
        "superseded native work must drain"
    );
    assert_eq!(
        fixture.driver.session(first_id).unwrap().state(),
        BuildSessionState::Finished(BuildSessionOutcome::Superseded)
    );
}

#[test]
fn structured_submit_failure_retains_generation_and_owned_event_history() {
    let mut fixture = RegistryFixture::new();
    let (millis, _) = bridge_clock(&mut fixture, 72);
    let uri = fixture.sources[0].2.clone();
    change_version(&mut fixture.server, &uri, 2);
    millis.store(300, Ordering::SeqCst);
    let (draft, index, sources, mut input) = prepare_parts(&fixture);
    input.workspace_packages.clear();
    let (_, result) = fixture.server.drive_build_bridge(
        None,
        &mut fixture.driver,
        &fixture.ids,
        &fixture.snapshots,
        &fixture.publisher,
        move |_, generation| {
            let mut draft = draft;
            draft.generation = generation;
            Ok((draft, index, sources, input))
        },
    );
    assert!(matches!(
        result,
        Err(BuildBridgeError::Submit(DriverSubmitError::Planning { .. }))
    ));
    let failed_replay = fixture.server.build_bridge_events(&fixture.driver);
    assert!(
        failed_replay
            .iter()
            .any(|(event, _)| event.identity.generation == BuildRequestGeneration::new(1))
    );

    change_version(&mut fixture.server, &uri, 3);
    millis.store(600, Ordering::SeqCst);
    let (draft, index, sources, input) = prepare_parts(&fixture);
    let (_, next) = fixture.server.drive_build_bridge(
        None,
        &mut fixture.driver,
        &fixture.ids,
        &fixture.snapshots,
        &fixture.publisher,
        move |_, generation| {
            let mut draft = draft;
            draft.generation = generation;
            Ok((draft, index, sources, input))
        },
    );
    assert!(next.unwrap().is_some());
    assert!(
        fixture
            .server
            .build_bridge_events(&fixture.driver)
            .iter()
            .any(|(event, _)| event.identity.generation == BuildRequestGeneration::new(2))
    );
}

#[test]
fn replay_keeps_original_version_incarnation_and_publication_without_activating_it() {
    let mut fixture = RegistryFixture::new();
    let seed_capture = fixture.capture();
    drop(seed_capture);
    let (millis, _) = bridge_clock(&mut fixture, 73);
    let uri = fixture.sources[0].2.clone();
    change_version(&mut fixture.server, &uri, 2);
    for (generation, deadline) in [(1, 300), (2, 600)] {
        millis.store(deadline, Ordering::SeqCst);
        let (draft, index, sources, input) = prepare_parts(&fixture);
        let (_, result) = fixture.server.drive_build_bridge(
            None,
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &fixture.publisher,
            move |_, build_generation| {
                let mut draft = draft;
                draft.generation = build_generation;
                Ok((draft, index, sources, input))
            },
        );
        assert!(result.unwrap().is_some(), "flush generation {generation}");
        if generation == 1 {
            change_version(&mut fixture.server, &uri, 3);
        }
    }

    let first_replay = fixture.server.build_bridge_events(&fixture.driver);
    let first_capture = first_replay
        .iter()
        .find(|(event, _)| event.identity.generation == BuildRequestGeneration::new(1))
        .unwrap()
        .1
        .clone();
    let second_capture = first_replay
        .iter()
        .find(|(event, _)| event.identity.generation == BuildRequestGeneration::new(2))
        .unwrap()
        .1
        .clone();
    assert!(!Arc::ptr_eq(&first_capture, &second_capture));
    assert!(first_capture.captured().snapshot().source_versions.iter().any(
        |version| matches!(version.origin, SourceOrigin::OpenBuffer { version } if version == 2)
    ));
    assert!(second_capture.captured().snapshot().source_versions.iter().any(
        |version| matches!(version.origin, SourceOrigin::OpenBuffer { version } if version == 3)
    ));
    assert!(first_replay.iter().any(|(event, _)| {
        event.identity.generation == BuildRequestGeneration::new(1)
            && matches!(event.kind, BuildEventKind::SnapshotPublished { .. })
    }));
    assert!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .is_none()
    );
    for (event, association) in &first_replay {
        assert!(fixture.driver.session(event.identity.session).is_some());
        assert!(
            event
                .identity
                .snapshot
                .is_none_or(|snapshot| snapshot == association.captured().snapshot().id)
        );
        if let BuildEventKind::SnapshotPublished { snapshot } = &event.kind {
            assert_eq!(
                snapshot.captured().snapshot().id,
                association.captured().snapshot().id
            );
        }
    }

    // Replaying through another real Driver with matching numeric request identity must not
    // attach events to the first Server's capture merely because normalized values compare equal.
    let mut foreign_server = Server::default();
    foreign_server.handle_message(serde_json::json!({
        "jsonrpc":"2.0", "id":1, "method":"initialize",
        "params":{"processId":null,"rootUri":null,"capabilities":{}}
    }));
    for (_, _, source_uri) in &fixture.sources {
        let text = fixture.server.document(source_uri).unwrap().1.to_owned();
        open_document(&mut foreign_server, source_uri, 1, text);
    }
    let foreign_ids = RegistryObservingAllocator::default();
    let foreign_snapshots = Arc::new(SnapshotRegistry::with_allocator(foreign_ids.clone()));
    let (seed_draft, seed_index, seed_sources, _) = prepare_parts(&fixture);
    let mut seed_draft = seed_draft;
    seed_draft.generation = BuildRequestGeneration::new(1);
    drop(
        foreign_server
            .capture_native_open_buffers(
                seed_draft,
                &foreign_ids,
                &foreign_snapshots,
                &seed_index,
                seed_sources,
            )
            .unwrap(),
    );
    let foreign_millis = Arc::new(AtomicU64::new(0));
    let foreign_time = foreign_millis.clone();
    let foreign_base = Instant::now();
    foreign_server = foreign_server.with_build_bridge(BuildLaneId::new(73), move || {
        foreign_base + Duration::from_millis(foreign_time.load(Ordering::SeqCst))
    });
    change_version(&mut foreign_server, &uri, 2);
    foreign_millis.store(300, Ordering::SeqCst);
    let foreign_publisher = Arc::new(mizar_ir::publisher::PhaseOutputPublisher::new(
        Arc::new(mizar_ir::storage::IrStorageService::new()),
        Arc::new(mizar_ir::identity::SnapshotHandleRegistry::new()),
    ));
    let mut foreign_driver =
        CompilerDriver::default().with_output_publisher(foreign_publisher.clone());
    let (draft, index, sources, input) = prepare_parts(&fixture);
    let (_, foreign_result) = foreign_server.drive_build_bridge(
        None,
        &mut foreign_driver,
        &foreign_ids,
        &foreign_snapshots,
        &foreign_publisher,
        move |_, generation| {
            let mut draft = draft;
            draft.generation = generation;
            Ok((draft, index, sources, input))
        },
    );
    assert!(foreign_result.unwrap().is_some());
    let foreign_replay = foreign_server.build_bridge_events(&foreign_driver);
    assert!(!foreign_replay.is_empty());
    let foreign_capture = &foreign_replay[0].1;
    assert_eq!(
        first_capture.captured().snapshot(),
        foreign_capture.captured().snapshot()
    );
    assert_ne!(
        first_capture.captured().native_sources().unwrap().as_ptr(),
        foreign_capture
            .captured()
            .native_sources()
            .unwrap()
            .as_ptr(),
    );
    assert!(
        fixture
            .server
            .build_bridge_events(&foreign_driver)
            .is_empty()
    );
}

#[test]
fn watch_and_lsp_lanes_keep_separate_events_and_publishers() {
    let mut fixture = RegistryFixture::new();
    let (millis, _) = bridge_clock(&mut fixture, 89);
    let watch_publisher = mizar_ir::publisher::PhaseOutputPublisher::new(
        Arc::new(mizar_ir::storage::IrStorageService::new()),
        Arc::new(mizar_ir::identity::SnapshotHandleRegistry::new()),
    );
    // No phase services here: both lanes share the real Driver table while their
    // publishers stay independent. Native producer coverage is exercised above.
    fixture.driver = CompilerDriver::default();
    let mut watch_request = fixture.draft.clone();
    watch_request.lane = BuildLaneId::new(90);
    watch_request.generation = BuildRequestGeneration::new(100);
    watch_request.origin = BuildRequestOrigin::Watch(WatchRequest {
        watch_root: watch_request.workspace_root.clone(),
        changed_paths: vec![],
    });
    let watch_input = fixture.driver_input();
    let watch = fixture
        .driver
        .submit_watch_change(
            watch_request,
            &fixture.ids,
            &fixture.snapshots,
            watch_input,
            WatchSubmitControl {
                previous_session: None,
                output_publisher: Some(&watch_publisher),
                file_watcher: WatchOwnerSeam::OwnerProvided,
                lsp_bridge: WatchOwnerSeam::OwnerProvided,
            },
            Default::default(),
        )
        .unwrap();
    let watch_id = watch.submission.session.id();
    let watch_events = fixture.driver.events(watch_id);
    let watch_snapshot = watch.submission.session.captured().snapshot().id;
    assert!(watch_events.known_session);
    watch_publisher
        .validate_current_snapshot(watch_snapshot)
        .unwrap();
    assert!(
        fixture
            .server
            .next_build_deadline(&fixture.driver)
            .is_none()
    );

    let uri = fixture.sources[0].2.clone();
    change_version(&mut fixture.server, &uri, 2);
    let pending_deadline = fixture.server.next_build_deadline(&fixture.driver);
    assert!(pending_deadline.is_some());
    assert_eq!(fixture.driver.events(watch_id), watch_events);
    assert!(!std::ptr::eq(
        Arc::as_ptr(&fixture.publisher),
        &watch_publisher
    ));
    millis.store(300, Ordering::SeqCst);
    let (draft, index, sources, input) = prepare_parts(&fixture);
    let (_, lsp_result) = fixture.server.drive_build_bridge(
        None,
        &mut fixture.driver,
        &fixture.ids,
        &fixture.snapshots,
        &fixture.publisher,
        move |_, generation| {
            let mut draft = draft;
            draft.generation = generation;
            Ok((draft, index, sources, input))
        },
    );
    assert_eq!(
        lsp_result.unwrap().unwrap().publication_decision,
        mizar_driver::request::PublicationDecision::Current
    );
    let lsp_events = fixture.server.build_bridge_events(&fixture.driver);
    assert!(!lsp_events.is_empty());
    assert!(
        lsp_events
            .iter()
            .all(|(event, _)| event.identity.lane == BuildLaneId::new(89))
    );
    assert!(
        watch_events
            .events()
            .iter()
            .all(|event| event.identity.lane == BuildLaneId::new(90))
    );
    assert_eq!(fixture.driver.events(watch_id), watch_events);
    watch_publisher
        .validate_current_snapshot(watch_snapshot)
        .unwrap();
    assert!(
        fixture
            .server
            .next_build_deadline(&fixture.driver)
            .is_none()
    );

    change_version(&mut fixture.server, &uri, 3);
    millis.store(600, Ordering::SeqCst);
    let (draft, index, mut sources, input) = prepare_parts(&fixture);
    sources.pop();
    let (_, capture_failure) = fixture.server.drive_build_bridge(
        None,
        &mut fixture.driver,
        &fixture.ids,
        &fixture.snapshots,
        &fixture.publisher,
        move |_, generation| {
            let mut draft = draft;
            draft.generation = generation;
            Ok((draft, index, sources, input))
        },
    );
    assert!(matches!(capture_failure, Err(BuildBridgeError::Capture(_))));
    assert_eq!(fixture.driver.events(watch_id), watch_events);
    watch_publisher
        .validate_current_snapshot(watch_snapshot)
        .unwrap();
    change_version(&mut fixture.server, &uri, 4);
    let (response, shutdown) = fixture
        .server
        .drive_build_bridge::<_, mizar_build::module_index::StaticSourceLayout, _>(
            Some(serde_json::json!({
                "jsonrpc":"2.0", "id":2, "method":"shutdown", "params":null
            })),
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &fixture.publisher,
            |_, _| panic!("terminal lifecycle traffic must discard the LSP slot"),
        );
    assert!(shutdown.unwrap().is_none());
    assert_eq!(response.unwrap()["result"], serde_json::Value::Null);
    assert!(
        fixture
            .server
            .next_build_deadline(&fixture.driver)
            .is_none()
    );
    assert_eq!(fixture.driver.events(watch_id), watch_events);
    assert_eq!(
        fixture.driver.session(watch_id),
        Some(&watch.submission.session)
    );
    watch_publisher
        .validate_current_snapshot(watch_snapshot)
        .unwrap();
}

#[test]
fn native_bridge_replacement_cancels_only_after_acceptance_and_drains_old_frontend() {
    let mut fixture = RegistryFixture::new();
    let (millis, _) = configure_nonterminal_bridge(&mut fixture, 91);
    let uri = fixture.sources[0].2.clone();
    change_version(&mut fixture.server, &uri, 2);
    millis.store(300, Ordering::SeqCst);
    let publisher = fixture.publisher.clone();
    let first = flush_bridge(&mut fixture, publisher).unwrap().unwrap();
    let first_id = first.session.id();
    let task_count = first.task_graph.as_ref().unwrap().tasks().len();
    let first_snapshot = first.session.captured().snapshot().id;
    assert_eq!(first.session.state(), BuildSessionState::Running);
    let initial_events = fixture.server.build_bridge_events(&fixture.driver);
    let original_capture = initial_events[0].1.clone();
    assert!(
        initial_events
            .iter()
            .all(|(event, _)| event.identity.generation == BuildRequestGeneration::new(1))
    );
    pause_at_frontend(&mut fixture, first_id, task_count);
    let old_frontend = fixture.driver.pending_task(first_id).unwrap().id.clone();

    // Admission records editor activity but leaves the accepted native work Running.
    change_version(&mut fixture.server, &uri, 3);
    millis.store(599, Ordering::SeqCst);
    let (_, not_due) = fixture
        .server
        .drive_build_bridge::<_, mizar_build::module_index::StaticSourceLayout, _>(
            None,
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &fixture.publisher,
            |_, _| panic!("admission alone must not capture"),
        );
    assert!(not_due.unwrap().is_none());
    assert_eq!(
        fixture.driver.session(first_id).unwrap().state(),
        BuildSessionState::Running
    );
    assert_eq!(
        fixture.server.build_bridge_events(&fixture.driver).len(),
        initial_events.len()
    );

    millis.store(600, Ordering::SeqCst);
    let publisher = fixture.publisher.clone();
    let replacement = flush_bridge(&mut fixture, publisher).unwrap().unwrap();
    let replacement_id = replacement.session.id();
    let replacement_count = replacement.task_graph.as_ref().unwrap().tasks().len();
    assert_ne!(first_id, replacement_id);
    assert_eq!(replacement.session.state(), BuildSessionState::Running);
    assert_eq!(replacement.session.captured().snapshot().id, first_snapshot);
    assert_eq!(
        fixture.driver.session(first_id).unwrap().state(),
        BuildSessionState::Cancelling
    );
    let replay = fixture.server.build_bridge_events(&fixture.driver);
    let first_replay = replay
        .iter()
        .filter(|(event, _)| event.identity.session == first_id)
        .collect::<Vec<_>>();
    let replacement_replay = replay
        .iter()
        .filter(|(event, _)| event.identity.session == replacement_id)
        .collect::<Vec<_>>();
    assert!(!first_replay.is_empty());
    assert!(!replacement_replay.is_empty());
    assert!(
        first_replay
            .iter()
            .all(|(event, _)| !matches!(event.kind, BuildEventKind::SessionFinished { .. }))
    );
    assert!(
        first_replay
            .iter()
            .all(|(_, association)| Arc::ptr_eq(association, &original_capture))
    );
    assert!(
        replacement_replay
            .iter()
            .all(|(_, association)| !Arc::ptr_eq(association, &original_capture))
    );
    assert!(
        first_replay
            .iter()
            .all(|(event, _)| event.identity.generation == BuildRequestGeneration::new(1))
    );
    assert!(
        replacement_replay
            .iter()
            .all(|(event, _)| event.identity.generation == BuildRequestGeneration::new(2))
    );

    let old_terminal = drain_bridge_session(&mut fixture, first_id, task_count)
        .expect("superseded old work must drain");
    assert_eq!(
        old_terminal.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Superseded)
    );
    let old_run = old_terminal.scheduler_run.as_ref().unwrap();
    assert!(
        old_run
            .task_states
            .iter()
            .any(|task| task.task_id == old_frontend
                && task.state == mizar_build::scheduler::TaskState::Cancelled)
    );
    assert!(
        old_run
            .phase_results
            .values()
            .flatten()
            .all(|result| result.output_refs.is_empty())
    );
    let old_events = fixture.server.build_bridge_events(&fixture.driver);
    let old_events = old_events
        .iter()
        .filter(|(event, _)| event.identity.session == first_id)
        .collect::<Vec<_>>();
    assert_eq!(
        old_events
            .iter()
            .filter(|(event, _)| matches!(event.kind, BuildEventKind::SessionFinished { .. }))
            .count(),
        1
    );
    assert!(
        !old_events
            .iter()
            .any(|(event, _)| matches!(event.kind, BuildEventKind::SnapshotPublished { .. }))
    );

    pause_at_frontend(&mut fixture, replacement_id, replacement_count);
    let replacement_terminal =
        drain_bridge_session(&mut fixture, replacement_id, replacement_count)
            .expect("replacement native work must complete");
    assert_eq!(
        replacement_terminal.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
    );
    assert!(
        replacement_terminal
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .any(|result| result.status == PhaseStatus::Complete && !result.output_refs.is_empty())
    );
    let replacement_events = fixture.server.build_bridge_events(&fixture.driver);
    let replacement_events = replacement_events
        .iter()
        .filter(|(event, _)| event.identity.session == replacement_id)
        .collect::<Vec<_>>();
    assert_eq!(
        replacement_events
            .iter()
            .filter(|(event, _)| matches!(event.kind, BuildEventKind::SessionFinished { .. }))
            .count(),
        1
    );
    assert!(
        replacement_events
            .iter()
            .any(|(event, _)| matches!(event.kind, BuildEventKind::SnapshotPublished { .. }))
    );
}

#[test]
fn native_bridge_equal_snapshot_preserves_completed_outputs_with_the_configured_publisher() {
    let mut fixture = RegistryFixture::new();
    let (millis, _) = configure_nonterminal_bridge(&mut fixture, 94);
    let uri = fixture.sources[0].2.clone();
    let publisher = fixture.publisher.clone();
    let mut previous_outputs: Option<Vec<mizar_ir::storage::AnyPhaseOutputRef>> = None;
    let mut previous_session = None;
    for (version, now) in [(2, 300), (3, 600)] {
        change_version(&mut fixture.server, &uri, version);
        millis.store(now, Ordering::SeqCst);
        let started = flush_bridge(&mut fixture, publisher.clone())
            .unwrap()
            .unwrap();
        assert_ne!(previous_session, Some(started.session.id()));
        let terminal = drain_bridge_session(
            &mut fixture,
            started.session.id(),
            started.task_graph.as_ref().unwrap().tasks().len(),
        )
        .expect("same-publisher native execution must drain");
        assert_eq!(
            terminal.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
        );
        let outputs = terminal
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .flat_map(|result| result.output_refs.iter().cloned())
            .collect::<Vec<_>>();
        assert!(
            outputs
                .iter()
                .any(|output| output.output_kind().as_str() == "FrontendOutput")
        );
        if let Some(previous) = previous_outputs {
            let unchanged_frontend = previous
                .iter()
                .find(|output| {
                    output.output_kind().as_str() == "FrontendOutput"
                        && output.work_unit().as_str() == "\"alpha\":\"leaf\""
                })
                .expect("the unchanged module produced native Frontend output");
            let replacement_frontend = outputs
                .iter()
                .find(|output| {
                    output.output_kind().as_str() == "FrontendOutput"
                        && output.work_unit() == unchanged_frontend.work_unit()
                })
                .unwrap();
            assert_eq!(replacement_frontend.output(), unchanged_frontend.output());
            assert_ne!(replacement_frontend, unchanged_frontend);
            let [old_frontend, new_frontend] = [unchanged_frontend, replacement_frontend].map(|output| {
                let handle = publisher
                    .storage()
                    .typed_handle::<mizar_frontend::orchestration::FrontendOutput<mizar_syntax::SurfaceAst>>(
                        output,
                        output.output_kind(),
                    )
                    .unwrap();
                publisher.storage().get(&handle).unwrap()
            });
            assert_ne!(old_frontend.source.source_id, new_frontend.source.source_id);
            let captured_leaf = terminal
                .session
                .captured()
                .native_sources()
                .unwrap()
                .iter()
                .find(|(_, source, _)| source.module_path == new_frontend.source.module_path)
                .unwrap();
            assert_eq!(new_frontend.source.source_id, captured_leaf.1.source_id);
            for output in &previous {
                publisher.storage().validate_handle(output).unwrap();
            }
        }
        assert!(
            fixture
                .driver
                .events(started.session.id())
                .events()
                .iter()
                .any(|event| { matches!(event.kind, BuildEventKind::SnapshotPublished { .. }) })
        );
        assert!(Arc::ptr_eq(&fixture.publisher, &publisher));
        previous_outputs = Some(outputs);
        previous_session = Some(started.session.id());
    }
}

#[test]
fn native_bridge_shutdown_and_exit_cancel_pending_frontend_once() {
    for terminal_method in ["shutdown", "exit"] {
        let mut fixture = RegistryFixture::new();
        let (millis, _) = configure_nonterminal_bridge(&mut fixture, 92);
        let uri = fixture.sources[0].2.clone();
        change_version(&mut fixture.server, &uri, 2);
        millis.store(300, Ordering::SeqCst);
        let publisher = fixture.publisher.clone();
        let started = flush_bridge(&mut fixture, publisher).unwrap().unwrap();
        let session = started.session.id();
        let task_count = started.task_graph.as_ref().unwrap().tasks().len();
        let original = fixture.server.build_bridge_events(&fixture.driver)[0]
            .1
            .clone();
        pause_at_frontend(&mut fixture, session, task_count);
        let frontend = fixture.driver.pending_task(session).unwrap().id.clone();
        change_version(&mut fixture.server, &uri, 3); // A slot is admitted and discarded by terminal traffic.
        millis.store(600, Ordering::SeqCst);
        let repeat = if terminal_method == "shutdown" {
            "exit"
        } else {
            "shutdown"
        };
        for (id, method) in [(17, terminal_method), (18, repeat)] {
            let message = if method == "shutdown" {
                serde_json::json!({"jsonrpc":"2.0","id":id,"method":"shutdown","params":null})
            } else {
                serde_json::json!({"jsonrpc":"2.0","method":"exit","params":null})
            };
            let (_, result) = fixture
                .server
                .drive_build_bridge::<_, mizar_build::module_index::StaticSourceLayout, _>(
                    Some(message),
                    &mut fixture.driver,
                    &fixture.ids,
                    &fixture.snapshots,
                    &fixture.publisher,
                    |_, _| panic!("terminal traffic must not capture"),
                );
            assert!(result.unwrap().is_none());
            assert_eq!(
                fixture.driver.session(session).unwrap().state(),
                BuildSessionState::Cancelling
            );
            assert!(fixture.server.build_bridge.as_ref().unwrap().slot.is_none());
        }
        change_version(&mut fixture.server, &uri, 4);
        millis.store(900, Ordering::SeqCst);
        let (_, after_terminal) = fixture
            .server
            .drive_build_bridge::<_, mizar_build::module_index::StaticSourceLayout, _>(
                None,
                &mut fixture.driver,
                &fixture.ids,
                &fixture.snapshots,
                &fixture.publisher,
                |_, _| panic!("terminal lifecycle must prevent later capture and submission"),
            );
        assert!(after_terminal.unwrap().is_none());
        assert!(fixture.server.build_bridge.as_ref().unwrap().slot.is_none());
        assert_eq!(
            fixture.driver.session(session).unwrap().state(),
            BuildSessionState::Cancelling
        );

        assert!(
            fixture
                .server
                .build_bridge_events(&fixture.driver)
                .iter()
                .all(|(event, _)| !matches!(event.kind, BuildEventKind::SessionFinished { .. }))
        );
        let terminal = drain_bridge_session(&mut fixture, session, task_count)
            .expect("shutdown cancellation must drain at the native checkpoint");
        assert_eq!(
            terminal.session.state(),
            BuildSessionState::Finished(BuildSessionOutcome::Cancelled)
        );
        let run = terminal.scheduler_run.as_ref().unwrap();
        assert!(run.task_states.iter().any(|task| task.task_id == frontend
            && task.state == mizar_build::scheduler::TaskState::Cancelled));
        assert!(
            run.phase_results[&frontend]
                .iter()
                .any(|result| result.status == PhaseStatus::Cancelled
                    && result.output_refs.is_empty())
        );
        let replay = fixture.server.build_bridge_events(&fixture.driver);
        assert!(!replay.is_empty());
        assert!(
            replay
                .iter()
                .all(|(_, association)| Arc::ptr_eq(association, &original))
        );
        assert_eq!(
            replay
                .iter()
                .filter(|(event, _)| matches!(event.kind, BuildEventKind::SessionFinished { .. }))
                .count(),
            1
        );
        assert!(
            !replay
                .iter()
                .any(|(event, _)| matches!(event.kind, BuildEventKind::SnapshotPublished { .. }))
        );
        assert!(
            fixture
                .server
                .progress_build_bridge(&mut fixture.driver, &fixture.ids, &fixture.snapshots)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            fixture
                .server
                .build_bridge_events(&fixture.driver)
                .iter()
                .map(|(event, _)| event.clone())
                .collect::<Vec<_>>(),
            replay
                .iter()
                .map(|(event, _)| event.clone())
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn native_other_lane_work_survives_lsp_supersede_and_shutdown_with_equal_snapshot() {
    let mut fixture = RegistryFixture::new();
    drop(fixture.capture()); // Configure native services and the Watch lane's distinct publisher.
    let watch_publisher = fixture.publisher.clone();
    let mut watch_draft = fixture.draft.clone();
    watch_draft.lane = BuildLaneId::new(93);
    watch_draft.generation = BuildRequestGeneration::new(1);
    watch_draft.origin = BuildRequestOrigin::Watch(WatchRequest {
        watch_root: watch_draft.workspace_root.clone(),
        changed_paths: vec![],
    });
    let watch_capture = fixture
        .server
        .capture_native_open_buffers(
            watch_draft,
            &fixture.ids,
            &fixture.snapshots,
            &fixture.index,
            fixture.sources.clone(),
        )
        .unwrap();
    let watch_snapshot = watch_capture.snapshot.captured().snapshot().id;
    watch_publisher.register_current_snapshot(watch_snapshot);
    let mut watch_capture = watch_capture;
    let watch_session = watch_capture
        .session
        .take()
        .expect("native watch capture owns its session");
    watch_capture.release_active = None;
    let watch = fixture
        .driver
        .start_captured(
            watch_session,
            &fixture.ids,
            &fixture.snapshots,
            fixture.driver_input(),
            SubmitOptions {
                discover_imports: true,
                diagnostic_index: Some(mizar_driver::driver::DiagnosticIndexMode::Native),
                ..Default::default()
            },
        )
        .unwrap();
    let watch_id = watch.session.id();
    let watch_count = watch.task_graph.as_ref().unwrap().tasks().len();
    for _ in 0..=watch_count {
        if fixture
            .driver
            .pending_task(watch_id)
            .is_some_and(|task| task.phases == [PipelinePhase::Frontend])
        {
            break;
        }
        assert!(
            fixture
                .driver
                .progress(watch_id, &fixture.ids, &fixture.snapshots)
                .unwrap()
                .is_none()
        );
    }
    assert!(
        fixture
            .driver
            .pending_task(watch_id)
            .is_some_and(|task| task.phases == [PipelinePhase::Frontend])
    );
    let watch_events = fixture.driver.events(watch_id);
    assert!(
        watch_events
            .events()
            .iter()
            .all(|event| event.identity.lane == BuildLaneId::new(93))
    );

    let lsp_publisher = Arc::new(mizar_ir::publisher::PhaseOutputPublisher::new(
        Arc::new(mizar_ir::storage::IrStorageService::new()),
        Arc::new(mizar_ir::identity::SnapshotHandleRegistry::new()),
    ));
    fixture.driver =
        std::mem::take(&mut fixture.driver).with_output_publisher(lsp_publisher.clone());
    let millis = Arc::new(AtomicU64::new(0));
    let clock = millis.clone();
    let base = Instant::now();
    fixture.server = std::mem::take(&mut fixture.server)
        .with_nonterminal_build_bridge(BuildLaneId::new(94), move || {
            base + Duration::from_millis(clock.load(Ordering::SeqCst))
        });
    let uri = fixture.sources[0].2.clone();
    change_version(&mut fixture.server, &uri, 2);
    millis.store(300, Ordering::SeqCst);
    let first = flush_bridge(&mut fixture, lsp_publisher.clone())
        .unwrap()
        .unwrap();
    assert_eq!(first.session.captured().snapshot().id, watch_snapshot);
    let first_lsp = first.session.id();
    change_version(&mut fixture.server, &uri, 3);
    millis.store(600, Ordering::SeqCst);
    let replacement = flush_bridge(&mut fixture, lsp_publisher.clone())
        .unwrap()
        .unwrap();
    assert_eq!(replacement.session.captured().snapshot().id, watch_snapshot);
    assert_eq!(
        fixture.driver.session(first_lsp).unwrap().state(),
        BuildSessionState::Cancelling
    );
    assert_eq!(
        fixture.driver.session(watch_id).unwrap().state(),
        BuildSessionState::Running
    );
    assert_eq!(
        fixture.driver.pending_task(watch_id).unwrap().phases,
        [PipelinePhase::Frontend]
    );
    assert_eq!(fixture.driver.events(watch_id), watch_events);
    watch_publisher
        .validate_current_snapshot(watch_snapshot)
        .unwrap();

    change_version(&mut fixture.server, &uri, 4);
    millis.store(900, Ordering::SeqCst);
    let (_, shutdown) = fixture
        .server
        .drive_build_bridge::<_, mizar_build::module_index::StaticSourceLayout, _>(
            Some(serde_json::json!({"jsonrpc":"2.0","id":31,"method":"shutdown","params":null})),
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &lsp_publisher,
            |_, _| panic!("shutdown must discard the LSP slot"),
        );
    assert!(shutdown.unwrap().is_none());
    assert_eq!(
        fixture
            .driver
            .session(replacement.session.id())
            .unwrap()
            .state(),
        BuildSessionState::Cancelling
    );
    assert_eq!(
        fixture.driver.session(watch_id).unwrap().state(),
        BuildSessionState::Running
    );
    assert_eq!(
        fixture.driver.pending_task(watch_id).unwrap().phases,
        [PipelinePhase::Frontend]
    );
    assert_eq!(fixture.driver.events(watch_id), watch_events);
    watch_publisher
        .validate_current_snapshot(watch_snapshot)
        .unwrap();
    let lsp_replay = fixture.server.build_bridge_events(&fixture.driver);
    assert!(!lsp_replay.is_empty());
    assert!(
        lsp_replay
            .iter()
            .all(|(event, _)| event.identity.lane == BuildLaneId::new(94))
    );
    assert!(
        !lsp_replay
            .iter()
            .any(|(event, _)| event.identity.session == watch_id)
    );

    fixture.driver =
        std::mem::take(&mut fixture.driver).with_output_publisher(watch_publisher.clone());
    let mut watch_terminal = None;
    for _ in 0..=watch_count {
        if fixture.driver.session(watch_id).unwrap().is_terminal() {
            break;
        }
        watch_terminal = fixture
            .driver
            .progress(watch_id, &fixture.ids, &fixture.snapshots)
            .unwrap()
            .or(watch_terminal);
    }
    let watch_terminal = watch_terminal.expect("Watch lane must complete its own native work");
    assert!(matches!(
        watch_terminal.session.state(),
        BuildSessionState::Finished(BuildSessionOutcome::Succeeded)
    ));
    assert!(
        watch_terminal
            .scheduler_run
            .as_ref()
            .unwrap()
            .phase_results
            .values()
            .flatten()
            .any(|result| result.status == PhaseStatus::Complete && !result.output_refs.is_empty())
    );
    let completed_watch_events = fixture.driver.events(watch_id);
    assert!(
        completed_watch_events
            .events()
            .starts_with(watch_events.events())
    );
    assert_eq!(
        completed_watch_events
            .events()
            .iter()
            .filter(|event| matches!(event.kind, BuildEventKind::SessionFinished { .. }))
            .count(),
        1
    );
    watch_publisher
        .validate_current_snapshot(watch_snapshot)
        .unwrap();
}
