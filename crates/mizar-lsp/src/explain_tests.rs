use super::full_snapshot_registry_a::RegistryFixture;
use crate::Server;
use mizar_diagnostics::explain::ExplanationStore;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};

fn explain_request(id: u64, snapshot: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "mizar/explain",
        "params": {"snapshot": snapshot, "diagnosticId": u64::MAX}
    })
}

#[test]
fn host_explanation_route_tracks_publication_freshness_cancellation_and_lifecycle() {
    let mut fixture = RegistryFixture::new();
    let (_, activated) = fixture.build();
    assert!(activated);
    let uri = fixture.sources[0].2.clone();
    let (published, freshness) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    assert_eq!(freshness, super::LspSnapshotFreshness::Current);
    let snapshot = published
        .publication()
        .diagnostics()
        .snapshot()
        .to_published_schema_string()
        .unwrap();
    let store = ExplanationStore::new();
    let document_before = fixture.server.document(&uri).unwrap().1.to_owned();

    let current = fixture
        .server
        .handle_message_with_explanations(
            explain_request(1, &snapshot),
            &fixture.driver,
            &fixture.snapshots,
            &store,
            &|| false,
        )
        .unwrap();
    assert_eq!(current["result"]["status"], "unavailable");
    assert_eq!(current["result"]["reason"], "target_unavailable");
    assert_eq!(current["result"]["freshness"], "current");
    assert_eq!(current["result"]["snapshot"], snapshot);
    assert_eq!(fixture.server.document(&uri).unwrap().1, document_before);

    let mut no_publication = Server::default();
    no_publication.handle_message(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}));
    let unavailable = no_publication
        .handle_message_with_explanations(
            explain_request(2, &snapshot),
            &fixture.driver,
            &fixture.snapshots,
            &store,
            &|| false,
        )
        .unwrap();
    assert_eq!(unavailable["result"]["reason"], "no_snapshot");
    assert_eq!(unavailable["result"]["snapshot"], Value::Null);

    for params in [
        Value::Null,
        json!([]),
        json!({"snapshot":snapshot}),
        json!({"snapshot":"invalid","diagnosticId":0}),
        json!({"snapshot":snapshot,"diagnosticId":-1}),
        json!({"snapshot":snapshot,"diagnosticId":0.5}),
        json!({"snapshot":snapshot,"diagnosticId":"0"}),
        json!({"snapshot":snapshot,"exprId":""}),
        json!({"snapshot":snapshot,"vcId":false}),
        json!({"snapshot":snapshot,"diagnosticId":0,"exprId":"expression"}),
        json!({"snapshot":snapshot,"exprId":"expression","vcId":"condition"}),
        json!({"snapshot":snapshot,"unknown":"expression"}),
    ] {
        let malformed = fixture
            .server
            .handle_message_with_explanations(
                json!({"jsonrpc":"2.0","id":3,"method":"mizar/explain","params":params}),
                &fixture.driver,
                &fixture.snapshots,
                &store,
                &|| panic!("invalid params reached query"),
            )
            .unwrap();
        assert_eq!(malformed["error"]["code"], -32602);
    }

    let before_cancel = fixture
        .server
        .handle_message_with_explanations(
            explain_request(4, &snapshot),
            &fixture.driver,
            &fixture.snapshots,
            &store,
            &|| true,
        )
        .unwrap();
    assert_eq!(before_cancel["error"]["code"], -32800);
    let calls = AtomicUsize::new(0);
    let after_cancel = fixture
        .server
        .handle_message_with_explanations(
            explain_request(5, &snapshot),
            &fixture.driver,
            &fixture.snapshots,
            &store,
            &|| calls.fetch_add(1, Ordering::SeqCst) > 0,
        )
        .unwrap();
    assert_eq!(after_cancel["error"]["code"], -32800);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(fixture.server.document(&uri).unwrap().1, document_before);
    assert!(std::sync::Arc::ptr_eq(
        &published,
        &fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .0
    ));

    fixture.did_change_version_only(&uri, 2);
    let stale = fixture
        .server
        .handle_message_with_explanations(
            explain_request(6, &snapshot),
            &fixture.driver,
            &fixture.snapshots,
            &store,
            &|| false,
        )
        .unwrap();
    assert_eq!(stale["result"]["freshness"], "stale_artifact");
    assert_eq!(stale["result"]["snapshot"], snapshot);

    fixture.did_close_and_reopen(&uri, 2);
    let reopened = fixture
        .server
        .handle_message_with_explanations(
            explain_request(7, &snapshot),
            &fixture.driver,
            &fixture.snapshots,
            &store,
            &|| false,
        )
        .unwrap();
    assert_eq!(reopened["result"]["freshness"], "stale_artifact");

    let replacement_text = format!(" {}", fixture.server.document(&uri).unwrap().1);
    fixture.did_change_text(&uri, 3, replacement_text.clone());
    let (_, activated) = fixture.build();
    assert!(activated);
    let (replacement, replacement_freshness) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    assert_eq!(replacement_freshness, super::LspSnapshotFreshness::Current);
    let replacement_snapshot = replacement
        .publication()
        .diagnostics()
        .snapshot()
        .to_published_schema_string()
        .unwrap();
    assert_ne!(replacement_snapshot, snapshot);
    let obsolete = fixture
        .server
        .handle_message_with_explanations(
            explain_request(8, &snapshot),
            &fixture.driver,
            &fixture.snapshots,
            &store,
            &|| false,
        )
        .unwrap();
    assert_eq!(obsolete["result"]["status"], "stale_handle");
    assert_eq!(obsolete["result"]["locations"], json!([]));
    assert_eq!(obsolete["result"]["payload"], Value::Null);
    let replaced = fixture
        .server
        .handle_message_with_explanations(
            explain_request(8, &replacement_snapshot),
            &fixture.driver,
            &fixture.snapshots,
            &store,
            &|| false,
        )
        .unwrap();
    assert_eq!(replaced["result"]["freshness"], "current");
    assert_eq!(replaced["result"]["snapshot"], replacement_snapshot);
    assert!(std::sync::Arc::ptr_eq(
        &replacement,
        &fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .0
    ));
    assert!(store.is_empty());
    assert_eq!(fixture.server.document(&uri).unwrap().1, replacement_text);

    fixture
        .server
        .handle_message(json!({"jsonrpc":"2.0","id":9,"method":"shutdown","params":null}));
    let after_shutdown = fixture
        .server
        .handle_message_with_explanations(
            explain_request(10, &replacement_snapshot),
            &fixture.driver,
            &fixture.snapshots,
            &store,
            &|| false,
        )
        .unwrap();
    assert_eq!(after_shutdown["error"]["code"], -32600);
}
