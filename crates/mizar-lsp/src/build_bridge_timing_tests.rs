use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use mizar_build::module_index::StaticSourceLayout;
use mizar_driver::request::{BuildLaneId, BuildRequestGeneration};
use serde_json::{Value, json};

use super::full_snapshot_registry_a::RegistryFixture;

fn notification(method: &str, params: Value) -> Value {
    json!({"jsonrpc":"2.0", "method":method, "params":params})
}

fn fixture_with_clock() -> (RegistryFixture, Arc<Mutex<Instant>>, Vec<(String, String)>) {
    let mut fixture = RegistryFixture::new();
    let documents = fixture
        .sources
        .iter()
        .map(|(_, _, uri)| {
            (
                uri.clone(),
                fixture.server.document(uri).unwrap().1.to_owned(),
            )
        })
        .collect::<Vec<_>>();
    for (uri, _) in &documents {
        fixture.server.handle_message(notification(
            "textDocument/didClose",
            json!({"textDocument":{"uri":uri}}),
        ));
    }

    let now = Arc::new(Mutex::new(Instant::now()));
    let clock = now.clone();
    fixture.server = std::mem::take(&mut fixture.server)
        .with_build_bridge(BuildLaneId::new(71), move || *clock.lock().unwrap());
    for (uri, text) in &documents {
        fixture.server.handle_message(notification(
            "textDocument/didOpen",
            json!({"textDocument":{"uri":uri,"languageId":"mizar","version":1,"text":text}}),
        ));
    }
    (fixture, now, documents)
}

fn set_time(now: &Mutex<Instant>, elapsed: Duration) {
    let mut value = now.lock().unwrap();
    *value += elapsed;
}

#[test]
fn admission_ignores_non_state_changes_and_empty_change_list_advances_slot() {
    let (mut fixture, now, documents) = fixture_with_clock();
    let first = fixture.server.build_bridge.as_ref().unwrap().slot.unwrap();
    assert_eq!(first.0, first.1);

    set_time(&now, Duration::from_millis(25));
    let uri = &documents[0].0;
    fixture.server.handle_message(notification(
        "textDocument/didOpen",
        json!({"textDocument":{"uri":uri,"languageId":"mizar","version":9,"text":"duplicate"}}),
    ));
    fixture.server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":1},"contentChanges":[]}),
    ));
    fixture.server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":7}]}),
    ));
    fixture.server.handle_message(notification(
        "textDocument/didSave",
        json!({"textDocument":{"uri":uri}}),
    ));
    assert_eq!(
        fixture.server.build_bridge.as_ref().unwrap().slot,
        Some(first)
    );

    set_time(&now, Duration::from_millis(10));
    fixture.server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[]}),
    ));
    let slot = fixture.server.build_bridge.as_ref().unwrap().slot.unwrap();
    assert_eq!(slot.0, first.0);
    assert_eq!(slot.1, first.1 + Duration::from_millis(35));
    assert!(fixture.server.native_open_buffer_snapshot().is_none());
}

#[test]
fn quiet_deadline_tracks_last_admission_and_flush_prepares_the_latest_open_cohort() {
    let (mut fixture, now, mut documents) = fixture_with_clock();
    let first = fixture
        .server
        .build_bridge
        .as_ref()
        .unwrap()
        .slot
        .unwrap()
        .0;
    set_time(&now, Duration::from_millis(100));
    fixture.server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":documents[0].0,"version":2},"contentChanges":[]}),
    ));
    set_time(&now, Duration::from_millis(100));
    documents[1].1.push_str("\r\n");
    fixture.server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":documents[1].0,"version":2},"contentChanges":[{"text":documents[1].1}]}),
    ));
    let expected = first + Duration::from_millis(500);
    assert_eq!(
        fixture.server.next_build_deadline(&fixture.driver),
        Some(expected)
    );

    set_time(&now, Duration::from_millis(299));
    let mut prepared = false;
    let publisher = fixture.publisher.clone();
    let draft = fixture.draft.clone();
    let index = fixture.index.clone();
    let sources = fixture.sources.clone();
    let input = fixture.driver_input();
    let (_, result) = fixture
        .server
        .drive_build_bridge::<_, StaticSourceLayout, _>(
            None,
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &publisher,
            |server, _| {
                prepared = true;
                assert_eq!(
                    server.document(&documents[0].0),
                    Some((2, documents[0].1.as_str()))
                );
                assert_eq!(
                    server.document(&documents[1].0),
                    Some((2, documents[1].1.as_str()))
                );
                Ok((draft, index, sources, input))
            },
        );
    assert!(result.unwrap().is_none());
    assert!(!prepared);

    set_time(&now, Duration::from_millis(1));
    let publisher = fixture.publisher.clone();
    let draft = fixture.draft.clone();
    let index = fixture.index.clone();
    let sources = fixture.sources.clone();
    let input = fixture.driver_input();
    let (_, result) = fixture
        .server
        .drive_build_bridge::<_, StaticSourceLayout, _>(
            None,
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &publisher,
            |server, _| {
                assert_eq!(
                    server.document(&documents[0].0),
                    Some((2, documents[0].1.as_str()))
                );
                assert_eq!(
                    server.document(&documents[1].0),
                    Some((2, documents[1].1.as_str()))
                );
                Ok((draft, index, sources, input))
            },
        );
    let submission = result.unwrap().unwrap();
    assert_eq!(
        submission.session.request().generation,
        BuildRequestGeneration::new(1)
    );
    let captured = submission.session.captured().native_sources().unwrap();
    assert_eq!(captured.len(), 2);
    for (_, _, source) in captured {
        let mizar_session::SourceOriginInput::OpenBuffer {
            uri,
            actual_version,
            text,
            ..
        } = &source.origin
        else {
            panic!("LSP capture must retain each open buffer");
        };
        assert!(
            documents
                .iter()
                .any(|(expected, raw)| expected == uri && raw.as_str() == text.as_ref())
        );
        assert_eq!(*actual_version, 2);
    }
}

#[test]
fn max_wait_is_exact_under_edits_and_running_sessions_wait_for_quiet() {
    let (mut fixture, now, documents) = fixture_with_clock();
    set_time(&now, Duration::from_millis(1_999));
    fixture.server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":documents[0].0,"version":2},"contentChanges":[]}),
    ));
    assert_eq!(
        fixture.server.next_build_deadline(&fixture.driver),
        Some(*now.lock().unwrap() + Duration::from_millis(1))
    );
    let running_quiet_deadline = fixture
        .server
        .build_bridge
        .as_ref()
        .unwrap()
        .deadline(true)
        .unwrap();
    assert_eq!(
        running_quiet_deadline,
        *now.lock().unwrap() + Duration::from_millis(300)
    );
    set_time(&now, Duration::from_millis(300));
    assert!(*now.lock().unwrap() >= running_quiet_deadline);

    let (mut fixture, now, documents) = fixture_with_clock();
    let first = fixture
        .server
        .build_bridge
        .as_ref()
        .unwrap()
        .slot
        .unwrap()
        .0;
    set_time(&now, Duration::from_millis(1_999));
    fixture.server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":documents[0].0,"version":2},"contentChanges":[]}),
    ));
    set_time(&now, Duration::from_millis(1));
    let publisher = fixture.publisher.clone();
    let mut prepared = false;
    assert_eq!(
        fixture.server.next_build_deadline(&fixture.driver),
        Some(first + Duration::from_millis(2_000))
    );
    let (_, result) = fixture
        .server
        .drive_build_bridge::<_, StaticSourceLayout, _>(
            None,
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &publisher,
            |_, _| {
                prepared = true;
                Err("test observes the exact idle cap without starting a build".into())
            },
        );
    assert!(matches!(result, Err(super::BuildBridgeError::Capture(_))));
    assert!(prepared);
}

#[test]
fn closing_last_document_and_terminal_lifecycle_discard_admission() {
    let (mut fixture, now, documents) = fixture_with_clock();
    for (uri, _) in &documents {
        fixture.server.handle_message(notification(
            "textDocument/didClose",
            json!({"textDocument":{"uri":uri}}),
        ));
    }
    set_time(&now, Duration::from_millis(300));
    let publisher = fixture.publisher.clone();
    let (_, result) = fixture
        .server
        .drive_build_bridge::<_, StaticSourceLayout, _>(
            None,
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &publisher,
            |_, _| panic!("empty open-document set must be discarded before capture"),
        );
    assert!(result.unwrap().is_none());
    assert!(
        fixture
            .server
            .build_bridge_events(&fixture.driver)
            .is_empty()
    );

    let (mut fixture, _, documents) = fixture_with_clock();
    fixture.server.handle_message(json!({
        "jsonrpc":"2.0", "id":2, "method":"shutdown", "params":null
    }));
    assert!(fixture.server.build_bridge.as_ref().unwrap().slot.is_none());
    let (_, result) = fixture
        .server
        .drive_build_bridge::<_, StaticSourceLayout, _>(
            Some(notification(
                "textDocument/didChange",
                json!({"textDocument":{"uri":documents[0].0,"version":2},"contentChanges":[]}),
            )),
            &mut fixture.driver,
            &fixture.ids,
            &fixture.snapshots,
            &fixture.publisher,
            |_, _| panic!("shutdown must prevent capture"),
        );
    assert!(result.unwrap().is_none());
    assert!(fixture.server.build_bridge.as_ref().unwrap().slot.is_none());
}
