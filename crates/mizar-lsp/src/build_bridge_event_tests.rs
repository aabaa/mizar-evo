use super::full_snapshot_registry_a::{RegistryFixture, RegistryObservingAllocator};
use super::*;
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
    let (millis, _) = bridge_clock(&mut fixture, 71);
    let uri = fixture.sources[0].2.clone();
    change_version(&mut fixture.server, &uri, 2);
    millis.store(300, Ordering::SeqCst);
    let (draft, index, sources, input) = prepare_parts(&fixture);
    let (_, first) = fixture.server.drive_build_bridge(
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
    assert!(first.unwrap().is_some());
    let original_events = fixture.server.build_bridge_events(&fixture.driver);
    assert!(!original_events.is_empty());
    let original_association = original_events[0].1.clone();
    let original_event_count = original_events.len();

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
    assert!(next.unwrap().is_some());
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
