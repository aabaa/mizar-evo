use super::{LspSnapshotFreshness, full_snapshot_registry_a::RegistryFixture};
use crate::Server;
use serde_json::{Value, json};

fn request(id: u64, uri: Option<&str>) -> Value {
    let text_document = uri.map(|uri| json!({"uri": uri}));
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": "textDocument/semanticTokens/full",
        "params": {"textDocument": text_document}
    })
}

fn notification(method: &str, params: Value) -> Value {
    json!({"jsonrpc":"2.0","method":method,"params":params})
}

fn initialize(server: &mut Server) {
    server.handle_message(json!({
        "jsonrpc":"2.0", "id":1, "method":"initialize",
        "params":{"processId":null,"rootUri":null,"capabilities":{}}
    }));
}

fn open(server: &mut Server, uri: &str, text: &str) {
    server.handle_message(notification(
        "textDocument/didOpen",
        json!({"textDocument":{"uri":uri,"languageId":"mizar","version":1,"text":text}}),
    ));
}

fn tokens(server: &mut Server, id: u64, uri: Option<&str>) -> Value {
    server
        .handle_message(request(id, uri))
        .unwrap()
        .get("result")
        .unwrap()
        .clone()
}

#[test]
fn raw_shell_classes_have_independent_legend_and_delta_oracle() {
    let mut server = Server::default();
    initialize(&mut server);
    let uri = "file:///tokens.miz";
    open(&mut server, uri, "theorem X : 1 = 2 ;");

    // The oracle is literal: type indexes are keyword=0, operator=1, number=2,
    // comment=3; the identifier X intentionally has no token.
    assert_eq!(
        tokens(&mut server, 2, Some(uri)),
        json!({"data":[
            0,0,7,0,0,
            0,10,1,1,0,
            0,2,1,2,0,
            0,2,1,1,0,
            0,2,1,2,0,
            0,2,1,1,0
        ]})
    );

    // String contents are not source tokens, even when they spell reserved
    // words, numerals, or operators. Both supported delimiters are covered.
    for (id, uri, text) in [
        (
            3,
            "file:///latex-double.miz",
            r#"@latex("x theorem 1 = y")"#,
        ),
        (4, "file:///latex-single.miz", "@latex('x theorem 1 = y')"),
    ] {
        open(&mut server, uri, text);
        assert_eq!(tokens(&mut server, id, Some(uri)), json!({"data":[]}));
    }

    // Keep the declaration keyword, argument words, delimiters, and numeral
    // visible around either form of the same quoted operator spelling.
    for (id, uri, quote) in [
        (5, "file:///operator-double.miz", '"'),
        (6, "file:///operator-single.miz", '\''),
    ] {
        let text = format!("infix_operator ( {quote}x theorem 1 = y{quote} , left , 80 )");
        open(&mut server, uri, &text);
        assert_eq!(
            tokens(&mut server, id, Some(uri)),
            json!({"data":[0,0,14,0,0, 0,15,1,1,0, 0,20,1,1,0, 0,2,4,0,0, 0,5,1,1,0, 0,2,2,2,0, 0,3,1,1,0]})
        );
    }

    // Escaped delimiters and backslashes do not end a string scan early.
    for (id, uri, text) in [
        (
            7,
            "file:///escaped-double.miz",
            r#"@latex("say \" theorem 1 = y \\ theorem 2")
theorem"#,
        ),
        (
            8,
            "file:///escaped-single.miz",
            r#"@latex('say \' theorem 1 = y \\ theorem 2')
theorem"#,
        ),
    ] {
        open(&mut server, uri, text);
        assert_eq!(
            tokens(&mut server, id, Some(uri)),
            json!({"data":[1,0,7,0,0]})
        );
    }

    // A comma remains the argument boundary across comments and layout.
    let uri = "file:///comma-string.miz";
    open(
        &mut server,
        uri,
        "@latex(\"first\", :: comment\n \"x theorem 1 = y\")",
    );
    assert_eq!(
        tokens(&mut server, 9, Some(uri)),
        json!({"data":[0,16,10,3,0]})
    );

    // The surrounding shell tokens use raw UTF-16 columns. The string has a
    // BMP character and a supplementary-plane character before the semicolon;
    // following-line tokens retain ordinary line-relative deltas.
    let uri = "file:///tokens-around-string.miz";
    open(
        &mut server,
        uri,
        "theorem X : 1 = @latex(\"雪😀 theorem 1 = y\") ;\ntheorem Y : 2 = 3 ;",
    );
    assert_eq!(
        tokens(&mut server, 10, Some(uri)),
        json!({"data":[0,0,7,0,0, 0,10,1,1,0, 0,2,1,2,0, 0,2,1,1,0, 0,30,1,1,0, 1,0,7,0,0, 0,10,1,1,0, 0,2,1,2,0, 0,2,1,1,0, 0,2,1,2,0, 0,2,1,1,0]})
    );

    // Quotes that are part of a postfix symbol or identifier do not put the
    // scanner into string mode, so later code tokens remain visible.
    let uri = "file:///quotes-outside-arguments.miz";
    open(&mut server, uri, "f\" theorem X' : 1 = 2 ;");
    assert_eq!(
        tokens(&mut server, 11, Some(uri)),
        json!({"data":[0,3,7,0,0, 0,11,1,1,0, 0,2,1,2,0, 0,2,1,1,0, 0,2,1,2,0, 0,2,1,1,0]})
    );
}

#[test]
fn comments_unicode_bom_and_mixed_raw_newlines_use_utf16_coordinates() {
    let mut server = Server::default();
    initialize(&mut server);
    let uri = "file:///unicode-tokens.miz";
    open(
        &mut server,
        uri,
        "\u{feff}:: 雪😀\r::=\n\nline 😀\n=::definition\r\nend;\nfor X",
    );

    // BOM occupies one raw UTF-16 unit. The comment occupies six units,
    // including the supplementary-plane emoji's two units. CR, CRLF and LF
    // each advance one raw line. The multiline comment includes an empty
    // line; its nonempty slices and the following code stay in raw positions.
    // The shell leaves the attached `end;` run unclassified; it is omitted.
    assert_eq!(
        tokens(&mut server, 2, Some(uri)),
        json!({"data":[
            0,1,6,3,0,
            1,0,3,3,0,
            2,0,7,3,0,
            1,0,3,3,0,
            0,3,10,0,0,
            2,0,3,0,0
        ]})
    );
}

#[test]
fn incomplete_raw_input_keeps_recognized_tokens_and_omits_unknowns() {
    let mut server = Server::default();
    initialize(&mut server);
    let uri = "file:///incomplete.miz";
    open(&mut server, uri, "theorem X : 1 = @ ;");

    assert_eq!(
        tokens(&mut server, 2, Some(uri)),
        json!({"data":[
            0,0,7,0,0,
            0,10,1,1,0,
            0,2,1,2,0,
            0,2,1,1,0,
            0,4,1,1,0
        ]})
    );

    let uri = "file:///unterminated-string-argument.miz";
    open(&mut server, uri, "theorem X : 1 = @latex(\"x theorem 1 = y");
    assert_eq!(
        tokens(&mut server, 3, Some(uri)),
        json!({"data":[0,0,7,0,0, 0,10,1,1,0, 0,2,1,2,0, 0,2,1,1,0]})
    );
}

#[test]
fn document_edits_close_reopen_empty_and_missing_sources_follow_live_text() {
    let mut server = Server::default();
    initialize(&mut server);
    let uri = "file:///lifecycle-tokens.miz";
    open(&mut server, uri, "theorem X : 1 = 2 ;");
    let original = tokens(&mut server, 2, Some(uri));

    // Accepted version-only changes preserve text; malformed and old-version
    // changes are ignored. A full replacement can accept empty text.
    server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[]}),
    ));
    server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":3},"contentChanges":[{"range":{},"text":"definition"}]}),
    ));
    server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":"definition"}]}),
    ));
    assert_eq!(tokens(&mut server, 3, Some(uri)), original);
    server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":4},"contentChanges":[{"text":""}]}),
    ));
    assert_eq!(tokens(&mut server, 4, Some(uri)), json!({"data":[]}));
    server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":5},"contentChanges":[{"text":"theorem  X : 1 = 2 ;\n"}]}),
    ));
    // Whitespace normalization preserves the recognized lexemes but raw
    // columns move by one UTF-16 unit after the repeated space.
    assert_eq!(
        tokens(&mut server, 9, Some(uri)),
        json!({"data":[0,0,7,0,0,0,11,1,1,0,0,2,1,2,0,0,2,1,1,0,0,2,1,2,0,0,2,1,1,0]})
    );
    // A BOM/CRLF edit has equal normalized input, but raw columns still move.
    server.handle_message(notification(
        "textDocument/didChange",
        json!({"textDocument":{"uri":uri,"version":6},"contentChanges":[{"text":"\u{feff}theorem  X : 1 = 2 ;\r\n"}]}),
    ));
    assert_eq!(
        tokens(&mut server, 10, Some(uri)),
        json!({"data":[0,1,7,0,0,0,11,1,1,0,0,2,1,2,0,0,2,1,1,0,0,2,1,2,0,0,2,1,1,0]})
    );

    server.handle_message(notification(
        "textDocument/didClose",
        json!({"textDocument":{"uri":uri}}),
    ));
    assert_eq!(tokens(&mut server, 5, Some(uri)), Value::Null);
    assert_eq!(
        tokens(&mut server, 6, Some("file:///never-opened.miz")),
        Value::Null
    );
    assert_eq!(
        server.handle_message(request(7, None)).unwrap()["error"]["code"],
        -32602
    );

    open(&mut server, uri, "definition X = 1 ;");
    assert_eq!(
        tokens(&mut server, 8, Some(uri)),
        json!({"data":[0,0,10,0,0,0,13,1,1,0,0,2,1,2,0,0,2,1,1,0]})
    );
}

#[test]
fn lifecycle_parameter_and_notification_errors_do_not_create_synthetic_tokens() {
    let mut server = Server::default();
    let before_initialize = server
        .handle_message(request(1, Some("file:///x.miz")))
        .unwrap();
    assert_eq!(before_initialize["error"]["code"], -32002);
    let initialized = server
        .handle_message(json!({
            "jsonrpc":"2.0", "id":8, "method":"initialize",
            "params":{"processId":null,"rootUri":null,"capabilities":{}}
        }))
        .unwrap();
    assert_eq!(
        initialized.pointer("/result/capabilities/semanticTokensProvider"),
        Some(
            &json!({"legend":{"tokenTypes":["keyword","operator","number","comment"],"tokenModifiers":[]},"full":true})
        )
    );

    let invalid = server
        .handle_message(json!({
            "jsonrpc":"2.0", "id":2, "method":"textDocument/semanticTokens/full",
            "params":{"textDocument":{"uri":""}}
        }))
        .unwrap();
    assert_eq!(invalid["error"]["code"], -32602);
    let no_params = server
        .handle_message(json!({
            "jsonrpc":"2.0", "id":3, "method":"textDocument/semanticTokens/full"
        }))
        .unwrap();
    assert_eq!(no_params["error"]["code"], -32602);

    let uri = "file:///empty-token-source.miz";
    open(&mut server, uri, "");
    assert_eq!(tokens(&mut server, 4, Some(uri)), json!({"data":[]}));
    assert!(
        server
            .handle_message(notification(
                "textDocument/didChange",
                json!({"broken":true})
            ))
            .is_none()
    );
    assert_eq!(tokens(&mut server, 5, Some(uri)), json!({"data":[]}));

    let shutdown = server
        .handle_message(json!({
            "jsonrpc":"2.0", "id":6, "method":"shutdown", "params":null
        }))
        .unwrap();
    assert_eq!(shutdown["result"], Value::Null);
    assert_eq!(
        server.handle_message(request(7, Some(uri))).unwrap()["error"]["code"],
        -32600
    );
    assert!(
        server
            .handle_message(notification("exit", Value::Null))
            .is_none()
    );
    assert!(server.handle_message(request(9, Some(uri))).is_none());
    assert!(server.lsp_snapshot.is_none());
    assert!(server.native_open_buffers.is_none());
    assert!(server.build_bridge.is_none());
}

#[test]
fn genuine_publication_staleness_and_replacement_never_enrich_syntax_tokens() {
    let mut fixture = RegistryFixture::new();
    let uri = fixture.sources[0].2.clone();
    let initial = fixture
        .server
        .handle_message_with_metadata(request(40, Some(&uri)), &fixture.driver, &fixture.snapshots)
        .unwrap()["result"]
        .clone();
    let (_, activated) = fixture.build();
    assert!(activated, "the test needs a genuine retained publication");
    assert_eq!(
        fixture
            .server
            .handle_message_with_metadata(
                request(41, Some(&uri)),
                &fixture.driver,
                &fixture.snapshots,
            )
            .unwrap()["result"],
        initial
    );
    assert_eq!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .1,
        LspSnapshotFreshness::Current
    );

    // A version-only edit makes the real artifact stale while retaining source text.
    fixture.did_change_version_only(&uri, 2);
    assert_eq!(tokens(&mut fixture.server, 42, Some(&uri)), initial);
    assert_eq!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .1,
        LspSnapshotFreshness::StaleArtifact
    );

    // A valid comment-only edit preserves the source program while changing the
    // raw syntax stream. The replacement build must still serve editor syntax.
    let current_text = fixture.server.document(&uri).unwrap().1.to_owned();
    fixture.did_change_text(&uri, 3, format!("{current_text}\n:: retained"));
    let changed = tokens(&mut fixture.server, 43, Some(&uri));
    let mut expected = initial["data"].as_array().unwrap().clone();
    expected.extend([json!(1), json!(0), json!(11), json!(3), json!(0)]);
    assert_eq!(changed, json!({"data":expected}));
    assert_eq!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .1,
        LspSnapshotFreshness::StaleArtifact
    );
    let (_, replacement_activated) = fixture.build();
    assert!(replacement_activated);
    assert_eq!(
        fixture
            .server
            .handle_message_with_metadata(
                request(44, Some(&uri)),
                &fixture.driver,
                &fixture.snapshots,
            )
            .unwrap()["result"],
        changed
    );
    assert_eq!(
        fixture
            .server
            .lsp_snapshot(&fixture.driver, &fixture.snapshots)
            .unwrap()
            .1,
        LspSnapshotFreshness::Current
    );
}
