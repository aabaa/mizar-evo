use super::full_snapshot_registry_a::RegistryFixture;
use crate::LspPosition;
use serde_json::{Value, json};

fn hover_request(id: u64, uri: &str, position: LspPosition) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/hover",
        "params": {
            "textDocument": {"uri": uri},
            "position": {"line": position.line, "character": position.character}
        }
    })
}

fn rendered(value: &Value) -> &str {
    value
        .pointer("/contents/value")
        .and_then(Value::as_str)
        .unwrap()
}

fn typed_variable_oracle(fixture: &RegistryFixture, uri: &str) -> (LspPosition, String, String) {
    let (snapshot, _) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .expect("published snapshot");
    let (loaded, source) = snapshot
        .native_open_buffers()
        .captured()
        .native_sources()
        .unwrap()
        .iter()
        .find_map(|(_, loaded, source)| {
            matches!(&source.origin, mizar_session::SourceOriginInput::OpenBuffer { uri: key, .. } if key == uri)
                .then_some((loaded, source))
        })
        .expect("fixture source association");
    let text = match &source.origin {
        mizar_session::SourceOriginInput::OpenBuffer { text, .. } => text.as_ref(),
        _ => unreachable!("selected source is an open buffer"),
    };
    let artifact = snapshot
        .publication()
        .artifacts()
        .iter()
        .find(|artifact| artifact.source_file == loaded.normalized_path.as_str())
        .expect("published artifact for original source");
    let row = artifact
        .expressions
        .iter()
        .filter(|row| row.rendered_surface == "X" && row.inferred_type.is_some())
        .filter(|row| {
            usize::try_from(row.source_range.end_byte)
                .ok()
                .and_then(|end| {
                    usize::try_from(row.source_range.start_byte)
                        .ok()
                        .map(|start| (start, end))
                })
                .and_then(|(start, end)| text.get(start..end))
                == Some("X")
        })
        .min_by_key(|row| row.source_range.start_byte)
        .expect("fixture publication includes a typed source variable expression");
    let start = usize::try_from(row.source_range.start_byte).unwrap();
    assert!(
        text.is_ascii(),
        "fixture is an ASCII source for this raw UTF-16 oracle"
    );
    (
        LspPosition {
            line: 0,
            character: start as u32,
        },
        row.expression_id.clone(),
        row.inferred_type.clone().unwrap(),
    )
}

#[test]
fn genuine_publication_serves_hover_through_host_route() {
    let mut fixture = RegistryFixture::new();
    let (_, activated) = fixture.build();
    assert!(activated);
    let uri = fixture.sources[0].2.clone();
    let (position, expected_id, expected_type) = typed_variable_oracle(&fixture, &uri);
    let direct = fixture
        .server
        .snapshot_hover(&fixture.driver, &fixture.snapshots, &uri, position)
        .unwrap()
        .expect("known published expression should hover");
    let text = rendered(&direct);
    assert!(text.contains(&format!("Source: {uri} (version 1)")));
    assert!(text.contains(&format!("Expression: {expected_id}")));
    assert!(text.contains("Surface: X"));
    assert!(text.contains(&format!("Type: {expected_type}")));
    assert_eq!(direct["range"]["start"]["line"], 0);
    assert_eq!(direct["range"]["start"]["character"], position.character);
    assert_eq!(direct["range"]["end"]["line"], 0);
    assert_eq!(direct["range"]["end"]["character"], position.character + 1);

    let response = fixture
        .server
        .handle_message_with_metadata(
            hover_request(41, &uri, position),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(response["id"], 41);
    assert_eq!(response["result"], direct);
}

#[test]
fn stale_reads_keep_original_source_identity_and_drop_the_live_editor_range() {
    let mut fixture = RegistryFixture::new();
    fixture.build();
    let uri = fixture.sources[0].2.clone();
    let retained = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap()
        .0;
    let original_version = &retained
        .native_open_buffers()
        .captured()
        .native_sources()
        .unwrap()
        .iter()
        .find(|(_, _, source)| matches!(&source.origin, mizar_session::SourceOriginInput::OpenBuffer { uri: source_uri, .. } if source_uri == &uri))
        .unwrap()
        .2;
    let original_version = match &original_version.origin {
        mizar_session::SourceOriginInput::OpenBuffer { actual_version, .. } => {
            i32::try_from(*actual_version).unwrap()
        }
        _ => panic!("fixture source should be the captured open buffer"),
    };

    let (position, _, _) = typed_variable_oracle(&fixture, &uri);
    let original = fixture
        .server
        .snapshot_hover(&fixture.driver, &fixture.snapshots, &uri, position)
        .unwrap()
        .unwrap();
    let original_text = rendered(&original).to_owned();
    fixture.did_change_version_only(&uri, original_version + 1);
    let stale = fixture
        .server
        .snapshot_hover(&fixture.driver, &fixture.snapshots, &uri, position)
        .unwrap()
        .expect("stale artifact remains readable");
    let stale_text = rendered(&stale);
    assert!(stale_text.contains("stale_artifact"));
    assert!(stale_text.contains(&uri));
    assert!(stale_text.contains(&format!("version {original_version}")));
    assert_eq!(stale.get("range"), None);
    for semantic_line in original_text.lines().skip(1) {
        assert!(stale_text.contains(semantic_line));
    }

    let edited = format!("\n   {}", fixture.server.document(&uri).unwrap().1);
    fixture.did_change_text(&uri, original_version + 2, edited);
    let stale_after_edit = fixture
        .server
        .snapshot_hover(&fixture.driver, &fixture.snapshots, &uri, position)
        .unwrap()
        .unwrap();
    assert_eq!(stale_after_edit.get("range"), None);
    assert!(rendered(&stale_after_edit).contains("stale_artifact"));
    assert!(rendered(&stale_after_edit).contains(&original_text));

    fixture.did_close_and_reopen(&uri, original_version + 2);
    let stale_after_reopen = fixture
        .server
        .snapshot_hover(&fixture.driver, &fixture.snapshots, &uri, position)
        .unwrap()
        .unwrap();
    assert_eq!(stale_after_reopen.get("range"), None);
    assert!(rendered(&stale_after_reopen).contains(&uri));
    assert!(rendered(&stale_after_reopen).contains(&format!("version {original_version}")));
}

#[test]
fn publication_replacement_switches_hover_metadata_and_freshness() {
    let mut fixture = RegistryFixture::new();
    fixture.build();
    let uri = fixture.sources[0].2.clone();
    let (position, _, _) = typed_variable_oracle(&fixture, &uri);
    let replacement = format!("   {}", fixture.server.document(&uri).unwrap().1);
    fixture.did_change_text(&uri, 2, replacement);
    let (_, activated) = fixture.build();
    assert!(activated);
    let (new_position, new_expression, _) = typed_variable_oracle(&fixture, &uri);
    assert_eq!(new_position.character, position.character + 3);
    let fresh = fixture
        .server
        .snapshot_hover(&fixture.driver, &fixture.snapshots, &uri, new_position)
        .unwrap()
        .unwrap();
    assert!(rendered(&fresh).contains("version 2"));
    assert!(rendered(&fresh).contains(&new_expression));
    assert_eq!(fresh["range"]["start"]["character"], new_position.character);

    let new_snapshot_id = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap()
        .0
        .publication()
        .captured()
        .snapshot()
        .id;
    fixture.publisher.mark_obsolete(new_snapshot_id).unwrap();
    let obsolete_hover = fixture
        .server
        .snapshot_hover(&fixture.driver, &fixture.snapshots, &uri, new_position)
        .unwrap()
        .unwrap();
    assert!(rendered(&obsolete_hover).contains("stale_artifact"));
    assert!(rendered(&obsolete_hover).contains(rendered(&fresh)));
    assert_eq!(obsolete_hover.get("range"), None);
}

#[test]
fn no_snapshot_unknown_expression_and_invalid_or_closed_lifecycle_requests_are_graceful() {
    let mut fixture = RegistryFixture::new();
    let uri = fixture.sources[0].2.clone();
    let position = LspPosition {
        line: 0,
        character: 0,
    };
    assert!(
        fixture
            .server
            .snapshot_hover(&fixture.driver, &fixture.snapshots, &uri, position)
            .unwrap()
            .is_none()
    );
    fixture.build();
    assert!(
        fixture
            .server
            .snapshot_hover(
                &fixture.driver,
                &fixture.snapshots,
                "file:///unknown.miz",
                position
            )
            .unwrap()
            .is_none()
    );
    let miss = fixture
        .server
        .snapshot_hover(
            &fixture.driver,
            &fixture.snapshots,
            &uri,
            LspPosition {
                line: 0,
                character: u32::MAX,
            },
        )
        .unwrap();
    assert!(miss.is_none());

    let base = fixture
        .server
        .handle_message(hover_request(51, &uri, position))
        .unwrap();
    assert_eq!(base["result"], Value::Null);
    for bad in [
        json!({"jsonrpc":"2.0","id":52,"method":"textDocument/hover","params":{}}),
        json!({"jsonrpc":"2.0","id":53,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":{"line":-1,"character":0}}}),
        json!({"jsonrpc":"2.0","id":54,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":2147483648_u64}}}),
    ] {
        let response = fixture.server.handle_message(bad).unwrap();
        assert_eq!(response["error"]["code"], -32602);
    }
    fixture
        .server
        .handle_message(json!({"jsonrpc":"2.0","id":56,"method":"shutdown","params":null}))
        .expect("shutdown request response");
    assert!(
        fixture
            .server
            .snapshot_hover(&fixture.driver, &fixture.snapshots, &uri, position)
            .unwrap()
            .is_none()
    );
    fixture
        .server
        .handle_message(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(
        fixture
            .server
            .handle_message_with_metadata(
                hover_request(55, &uri, position),
                &fixture.driver,
                &fixture.snapshots
            )
            .is_none()
    );
}

fn code_action_request(id: u64, uri: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/codeAction",
        "params": {
            "textDocument": {"uri": uri},
            "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}},
            "context": {"diagnostics": [], "only": ["quickfix"]}
        }
    })
}

fn code_action_resolve(id: u64, data: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"codeAction/resolve","params":{"title":"old action","kind":"quickfix","data":data}})
}

#[test]
fn code_actions_require_a_current_live_publication_and_reject_old_locators() {
    let mut fixture = RegistryFixture::new();
    fixture.server.initialized = Some(json!({"capabilities":{"textDocument":{"codeAction":{
        "dataSupport":true,"resolveSupport":{"properties":["edit"]}
    }}}}));
    let uri = fixture.sources[0].2.clone();

    let without_publication = fixture
        .server
        .handle_message_with_metadata(
            code_action_request(61, &uri),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(without_publication["result"], json!([]));

    let (_, activated) = fixture.build();
    assert!(activated);
    let (original, freshness) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    assert_eq!(freshness, crate::server::LspSnapshotFreshness::Current);
    let snapshot_id = original.publication().captured().snapshot().id;
    let old_locator = json!({
        "publication": original.publication_identity(),
        "snapshot": snapshot_id.to_published_schema_string().unwrap(),
        "id": 0,
        "fix": "former-fix",
        "uri": uri,
        "version": 1
    });
    let current_empty = fixture
        .server
        .handle_message_with_metadata(
            code_action_request(62, &uri),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(current_empty["result"], json!([]));
    let current_unknown = fixture
        .server
        .handle_message_with_metadata(
            code_action_resolve(63, old_locator.clone()),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(current_unknown["error"]["code"], -32801);

    fixture.did_close_and_reopen(&uri, 1);
    assert!(fixture.build().1);
    let (reopened, freshness) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    assert_eq!(freshness, crate::server::LspSnapshotFreshness::Current);
    assert_eq!(reopened.publication().captured().snapshot().id, snapshot_id);
    assert_eq!(fixture.server.document(&uri).unwrap().0, 1);
    assert_ne!(
        reopened.publication_identity(),
        original.publication_identity()
    );
    assert!(fixture.build().1);
    let (recaptured, freshness) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    assert_eq!(freshness, crate::server::LspSnapshotFreshness::Current);
    assert_eq!(
        recaptured.publication().captured().snapshot().id,
        snapshot_id
    );
    assert_ne!(
        recaptured.publication_identity(),
        reopened.publication_identity()
    );
    let replayed = fixture
        .server
        .handle_message_with_metadata(
            code_action_resolve(68, old_locator.clone()),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(replayed["error"]["code"], -32801);

    fixture.did_change_version_only(&uri, 2);
    assert_eq!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .1,
        crate::server::LspSnapshotFreshness::StaleArtifact
    );
    let stale_actions = fixture
        .server
        .handle_message_with_metadata(
            code_action_request(64, &uri),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(stale_actions["result"], json!([]));
    let stale_resolve = fixture
        .server
        .handle_message_with_metadata(
            code_action_resolve(65, old_locator.clone()),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(stale_resolve["error"]["code"], -32801);

    fixture.did_close_and_reopen(&uri, 2);
    assert_eq!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .1,
        crate::server::LspSnapshotFreshness::StaleArtifact
    );
    let reopened_resolve = fixture
        .server
        .handle_message_with_metadata(
            code_action_resolve(66, old_locator.clone()),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(reopened_resolve["error"]["code"], -32801);

    fixture.did_change_text(
        &uri,
        3,
        format!("   {}", fixture.server.document(&uri).unwrap().1),
    );
    let (_, activated) = fixture.build();
    assert!(activated);
    let (replacement, freshness) = fixture
        .server
        .lsp_snapshot(&fixture.driver, &fixture.snapshots)
        .unwrap();
    assert_eq!(freshness, crate::server::LspSnapshotFreshness::Current);
    assert_ne!(
        replacement.publication().captured().snapshot().id,
        snapshot_id
    );
    let replacement_resolve = fixture
        .server
        .handle_message_with_metadata(
            code_action_resolve(67, old_locator),
            &fixture.driver,
            &fixture.snapshots,
        )
        .unwrap();
    assert_eq!(replacement_resolve["error"]["code"], -32801);
    assert!(replacement_resolve.get("result").is_none());
}
