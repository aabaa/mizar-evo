//! Connection lifecycle, full document buffers and bounded LSP stdio framing.

use std::{
    collections::BTreeMap,
    io::{self, BufRead, Write},
};

use serde_json::{Value, json};

/// One LSP connection and its exact client-owned document text.
#[derive(Default)]
pub struct Server {
    initialized: Option<Value>,
    shutdown: bool,
    exit_code: Option<i32>,
    documents: BTreeMap<String, (i32, String)>,
}

impl Server {
    /// Borrows exact raw params from the first successful initialize request.
    /// These client values are not effective settings or compiler context.
    pub fn initialization_params(&self) -> Option<&Value> {
        self.initialized.as_ref()
    }

    /// Acquires current native metadata from the explicit initialize root URI.
    pub fn native_workspace_inputs(
        &self,
    ) -> Result<
        Option<(
            mizar_session::WorkspaceRoot,
            Vec<mizar_build::planner::WorkspacePackage>,
            mizar_build::planner::Lockfile,
        )>,
        String,
    > {
        let Some(uri) = self
            .initialization_params()
            .and_then(|params| params.get("rootUri"))
            .and_then(Value::as_str)
        else {
            return Ok(None);
        };
        let path = mizar_session::source::file_path_from_document_uri(uri)
            .ok_or_else(|| "workspace root must be a supported file URI".to_owned())?
            .components()
            .collect::<std::path::PathBuf>();
        if !path.is_absolute() {
            return Err("workspace root must be absolute".into());
        }
        if !std::fs::symlink_metadata(&path)
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            return Err("workspace root must be an ordinary directory".into());
        }
        let path = std::fs::canonicalize(path).map_err(|error| error.to_string())?;
        let root =
            mizar_session::WorkspaceRoot::new(path.to_str().ok_or("workspace root must be UTF-8")?);
        let (packages, lockfile) = mizar_build::planner::read_native_workspace_inputs(&root)?;
        Ok(Some((root, packages, lockfile)))
    }

    /// Borrows the current full-sync document version and text.
    pub fn document(&self, uri: &str) -> Option<(i32, &str)> {
        self.documents
            .get(uri)
            .map(|(version, text)| (*version, text.as_str()))
    }

    /// Handles a JSON-RPC message without creating compiler snapshots.
    pub fn handle_message(&mut self, message: Value) -> Option<Value> {
        if self.exit_code.is_some() {
            return None;
        }
        let error = |id: Value, code, text| {
            Some(json!({"jsonrpc": "2.0", "id": id,
                "error": {"code": code, "message": text}}))
        };
        let valid_id =
            |id: &Value| id.is_string() || id.as_i64().is_some_and(|id| i32::try_from(id).is_ok());
        if !message.is_object() || message.get("jsonrpc") != Some(&json!("2.0")) {
            return error(Value::Null, -32600, "Invalid Request");
        }
        if message.get("method").is_none() {
            let result = message.get("result").is_some();
            let response_error = message.get("error");
            if message
                .get("id")
                .is_some_and(|id| id.is_null() || valid_id(id))
                && result != response_error.is_some()
                && response_error.is_none_or(|value| {
                    value.is_object()
                        && value.get("code").is_some_and(&valid_id)
                        && value.get("code").is_some_and(Value::is_number)
                        && value.get("message").is_some_and(Value::is_string)
                })
            {
                return None;
            }
            return error(Value::Null, -32600, "Invalid Request");
        }
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return error(Value::Null, -32600, "Invalid Request");
        };
        if message.get("result").is_some() || message.get("error").is_some() {
            return error(Value::Null, -32600, "Invalid Request");
        }
        let params = message.get("params");
        if let Some(id) = message.get("id") {
            if !valid_id(id) {
                return error(Value::Null, -32600, "Invalid Request");
            }
            let result = if method == "initialize" {
                if self.initialized.is_some() {
                    return error(id.clone(), -32600, "Invalid Request");
                }
                let Some(params) = params.filter(|params| params.is_object()) else {
                    return error(id.clone(), -32602, "Invalid params");
                };
                if !params.get("processId").is_some_and(|id| {
                    id.is_null() || id.as_i64().is_some_and(|id| i32::try_from(id).is_ok())
                }) || !params
                    .get("rootUri")
                    .is_some_and(|uri| uri.is_null() || uri.is_string())
                    || !params.get("capabilities").is_some_and(Value::is_object)
                {
                    return error(id.clone(), -32602, "Invalid params");
                }
                self.initialized = Some(params.clone());
                json!({"capabilities": {"positionEncoding": "utf-16",
                    "textDocumentSync": {"openClose": true, "change": 1}}})
            } else {
                if self.initialized.is_none() {
                    return error(id.clone(), -32002, "Server not initialized");
                }
                if self.shutdown {
                    return error(id.clone(), -32600, "Invalid Request");
                }
                match method {
                    "shutdown" => {
                        if !params.is_none_or(Value::is_null) {
                            return error(id.clone(), -32602, "Invalid params");
                        }
                        self.shutdown = true;
                        Value::Null
                    }
                    "exit"
                    | "initialized"
                    | "textDocument/didOpen"
                    | "textDocument/didChange"
                    | "textDocument/didClose" => {
                        return error(id.clone(), -32600, "Invalid Request");
                    }
                    _ => {
                        if !method.starts_with("$/")
                            && params
                                .is_some_and(|params| !params.is_object() && !params.is_array())
                        {
                            return error(id.clone(), -32602, "Invalid params");
                        }
                        return error(id.clone(), -32601, "Method not found");
                    }
                }
            };
            return Some(json!({"jsonrpc": "2.0", "id": id, "result": result}));
        }
        if method == "exit" {
            if params.is_none_or(Value::is_null) {
                self.exit_code = Some(if self.shutdown { 0 } else { 1 });
            }
            return None;
        }
        if self.initialized.is_none() || self.shutdown {
            return None;
        }
        let params = params?;
        let document = params.get("textDocument")?;
        let uri = document.get("uri")?.as_str()?;
        if uri.is_empty() {
            return None;
        }
        match method {
            "textDocument/didOpen" => {
                let version = i32::try_from(document.get("version")?.as_i64()?).ok()?;
                document.get("languageId")?.as_str()?;
                let text = document.get("text")?.as_str()?;
                if !self.documents.contains_key(uri) {
                    self.documents
                        .insert(uri.to_owned(), (version, text.to_owned()));
                }
            }
            "textDocument/didChange" => {
                let version = i32::try_from(document.get("version")?.as_i64()?).ok()?;
                let changes = params.get("contentChanges")?.as_array()?;
                if changes.iter().any(|change| {
                    !change.get("text").is_some_and(Value::is_string)
                        || change.get("range").is_some()
                        || change.get("rangeLength").is_some()
                }) {
                    return None;
                }
                let buffer = self.documents.get_mut(uri)?;
                if version > buffer.0 {
                    if let Some(text) = changes
                        .last()
                        .and_then(|change| change.get("text"))
                        .and_then(Value::as_str)
                    {
                        buffer.1 = text.to_owned();
                    }
                    buffer.0 = version;
                }
            }
            "textDocument/didClose" => {
                self.documents.remove(uri);
            }
            _ => {}
        }
        None
    }

    /// Serves bounded Content-Length frames until an exit notification.
    pub fn serve(&mut self, input: &mut impl BufRead, output: &mut impl Write) -> io::Result<i32> {
        let invalid = || io::Error::new(io::ErrorKind::InvalidData, "Invalid LSP frame");
        loop {
            if let Some(code) = self.exit_code {
                return Ok(code);
            }
            let mut header = Vec::new();
            loop {
                let available = input.fill_buf()?;
                if available.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Missing LSP exit or truncated header",
                    ));
                }
                let count = available
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(available.len(), |index| index + 1);
                if header.len() + count > 8 * 1024 {
                    return Err(invalid());
                }
                header.extend_from_slice(&available[..count]);
                input.consume(count);
                if header.last() == Some(&b'\n') && !header.ends_with(b"\r\n") {
                    return Err(invalid());
                }
                if header.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            if !header.is_ascii() {
                return Err(invalid());
            }
            let mut length = None;
            let mut content_type = None;
            let header = std::str::from_utf8(&header).map_err(|_| invalid())?;
            for line in header[..header.len() - 4].split("\r\n") {
                let (name, value) = line.split_once(':').ok_or_else(&invalid)?;
                if name.is_empty()
                    || !name.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte)
                    })
                    || !value
                        .bytes()
                        .all(|byte| byte.is_ascii_graphic() || byte == b' ' || byte == b'\t')
                {
                    return Err(invalid());
                }
                let value = value.trim();
                if name.eq_ignore_ascii_case("Content-Length") {
                    if length.is_some()
                        || value.is_empty()
                        || !value.bytes().all(|byte| byte.is_ascii_digit())
                    {
                        return Err(invalid());
                    }
                    let value = value.parse::<usize>().map_err(|_| invalid())?;
                    if value > 16 * 1024 * 1024 {
                        return Err(invalid());
                    }
                    length = Some(value);
                } else if name.eq_ignore_ascii_case("Content-Type")
                    && content_type.replace(value).is_some()
                {
                    return Err(invalid());
                }
            }
            let mut body = vec![0; length.ok_or_else(&invalid)?];
            input.read_exact(&mut body)?;
            let supported = content_type.is_none_or(|value| {
                let mut parts = value.split(';');
                parts.next().is_some_and(|kind| {
                    kind.trim()
                        .eq_ignore_ascii_case("application/vscode-jsonrpc")
                }) && parts.all(|part| {
                    part.trim().split_once('=').is_some_and(|(key, value)| {
                        !key.trim().eq_ignore_ascii_case("charset") || {
                            let value = value.trim();
                            let value = value
                                .strip_prefix('"')
                                .and_then(|value| value.strip_suffix('"'))
                                .unwrap_or(value);
                            value.eq_ignore_ascii_case("utf-8")
                                || value.eq_ignore_ascii_case("utf8")
                        }
                    })
                })
            });
            let response = match serde_json::from_slice::<Value>(&body) {
                Ok(message) if supported => self.handle_message(message),
                _ => Some(json!({"jsonrpc": "2.0", "id": null,
                    "error": {"code": -32700, "message": "Parse error"}})),
            };
            if let Some(response) = response {
                let body = serde_json::to_vec(&response).map_err(io::Error::other)?;
                write!(output, "Content-Length: {}\r\n\r\n", body.len())?;
                output.write_all(&body)?;
                output.flush()?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    fn request(method: &str, params: Value) -> Value {
        json!({"jsonrpc": "2.0", "id": 7, "method": method, "params": params})
    }

    fn notification(method: &str, params: Value) -> Value {
        json!({"jsonrpc": "2.0", "method": method, "params": params})
    }

    fn initialize(server: &mut Server) {
        let response = server
            .handle_message(request(
                "initialize",
                json!({"processId": null, "rootUri": null, "capabilities": {"unknown": true}}),
            ))
            .unwrap();
        assert_eq!(
            response,
            json!({"jsonrpc": "2.0", "id": 7, "result": {
            "capabilities": {"positionEncoding": "utf-16", "textDocumentSync": {"openClose": true, "change": 1}}}})
        );
    }

    fn assert_error(server: &mut Server, message: Value, code: i32) -> Value {
        let response = server.handle_message(message).unwrap();
        assert!(response.get("id").is_some());
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["error"]["code"], code);
        assert!(response["error"]["message"].is_string());
        assert!(response.get("result").is_none());
        response
    }

    fn frame(body: impl AsRef<[u8]>) -> Vec<u8> {
        let body = body.as_ref();
        let mut bytes = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
        bytes.extend_from_slice(body);
        bytes
    }

    fn responses(mut bytes: &[u8]) -> Vec<Value> {
        let mut result = Vec::new();
        while !bytes.is_empty() {
            let end = bytes
                .windows(4)
                .position(|part| part == b"\r\n\r\n")
                .unwrap();
            let header = std::str::from_utf8(&bytes[..end]).unwrap();
            let length: usize = header
                .strip_prefix("Content-Length: ")
                .unwrap()
                .parse()
                .unwrap();
            let body = &bytes[end + 4..end + 4 + length];
            result.push(serde_json::from_slice(body).unwrap());
            bytes = &bytes[end + 4 + length..];
        }
        result
    }

    #[test]
    fn lifecycle_and_message_kinds_preserve_request_ids() {
        let mut server = Server::default();
        assert_error(&mut server, request("unknown", Value::Null), -32002);
        for method in [
            "initialize",
            "shutdown",
            "textDocument/didOpen",
            "$/cancelRequest",
        ] {
            assert!(
                server
                    .handle_message(notification(
                        method,
                        if method == "initialize" {
                            json!({"processId": null, "rootUri": null, "capabilities": {}})
                        } else {
                            Value::Null
                        }
                    ))
                    .is_none()
            );
        }
        initialize(&mut server);
        assert!(
            server
                .handle_message(notification("shutdown", Value::Null))
                .is_none()
        );
        assert!(!server.shutdown);
        assert_error(&mut server, request("initialize", Value::Null), -32600);
        for id in [json!(i32::MIN), json!(i32::MAX), json!("リクエスト😀")] {
            let mut message = request("$/cancelRequest", Value::Null);
            message["id"] = id.clone();
            let response = server.handle_message(message).unwrap();
            assert_eq!(response["id"], id);
            assert_eq!(response["error"]["code"], -32601);
        }
        for method in [
            "exit",
            "initialized",
            "textDocument/didOpen",
            "textDocument/didChange",
            "textDocument/didClose",
        ] {
            assert_error(&mut server, request(method, Value::Null), -32600);
            assert!(
                server
                    .handle_message(notification(method, json!({})))
                    .is_none()
            );
        }
        assert_error(&mut server, request("shutdown", json!({})), -32602);
        for params in [Value::Null, json!(false), json!(1), json!("bad")] {
            assert_error(&mut server, request("unknown", params), -32602);
        }
        for params in [json!({}), json!([])] {
            assert_error(&mut server, request("unknown", params), -32601);
        }
        let mut unknown = request("unknown", Value::Null);
        unknown.as_object_mut().unwrap().remove("params");
        assert_error(&mut server, unknown, -32601);
        let mut shutdown = request("shutdown", Value::Null);
        shutdown.as_object_mut().unwrap().remove("params");
        assert_eq!(
            server.handle_message(shutdown).unwrap(),
            json!({"jsonrpc": "2.0", "id": 7, "result": null})
        );
        for method in ["shutdown", "initialize", "unknown"] {
            assert_error(&mut server, request(method, Value::Null), -32600);
        }
        assert!(
            server
                .handle_message(notification("exit", json!({})))
                .is_none()
        );
        assert_eq!(server.exit_code, None);
        assert!(
            server
                .handle_message(notification("exit", Value::Null))
                .is_none()
        );
        assert_eq!(
            server.serve(&mut Cursor::new([]), &mut Vec::new()).unwrap(),
            0
        );
    }

    #[test]
    fn initialization_validates_every_required_field_without_committing_failure() {
        for params in [
            Value::Null,
            json!([]),
            json!({}),
            json!({"processId": null, "capabilities": {}}),
            json!({"rootUri": null, "capabilities": {}}),
            json!({"processId": null, "rootUri": null}),
            json!({"processId": "1", "rootUri": null, "capabilities": {}}),
            json!({"processId": 2147483648_u64, "rootUri": null, "capabilities": {}}),
            json!({"processId": 1.5, "rootUri": null, "capabilities": {}}),
            json!({"processId": null, "rootUri": false, "capabilities": {}}),
            json!({"processId": null, "rootUri": null, "capabilities": []}),
        ] {
            let mut server = Server::default();
            assert_error(&mut server, request("initialize", params), -32602);
            initialize(&mut server);
        }
        for process in [i32::MIN, i32::MAX] {
            let mut server = Server::default();
            assert!(server.handle_message(request("initialize", json!({
                "processId": process, "rootUri": "file:///workspace", "capabilities": {}, "ignored": true
            }))).unwrap().get("result").is_some());
        }
    }

    #[test]
    fn invalid_envelopes_and_ids_cannot_invoke_methods() {
        let mut server = Server::default();
        initialize(&mut server);
        for message in [
            Value::Null,
            json!([]),
            json!([request("shutdown", Value::Null)]),
            json!({}),
            json!({"jsonrpc": "1.0", "method": "shutdown", "id": 1}),
            json!({"jsonrpc": "2.0", "method": 12, "id": 1}),
            json!({"jsonrpc": "2.0", "method": "shutdown", "id": 1, "result": null}),
            json!({"jsonrpc": "2.0", "method": "exit", "error": {"code": 1, "message": "error"}}),
        ] {
            let response = assert_error(&mut server, message, -32600);
            assert_eq!(response.get("id"), Some(&Value::Null));
        }
        for id in [
            Value::Null,
            json!(1.5),
            json!(2147483648_u64),
            json!(-2147483649_i64),
            json!(true),
            json!({}),
            json!([]),
        ] {
            let mut message = request("shutdown", Value::Null);
            message["id"] = id;
            assert_eq!(
                assert_error(&mut server, message, -32600).get("id"),
                Some(&Value::Null)
            );
        }
        assert!(!server.shutdown);
        assert_eq!(server.exit_code, None);
    }

    #[test]
    fn unmatched_responses_are_silent_but_malformed_responses_are_rejected() {
        let mut server = Server::default();
        for id in [Value::Null, json!(1), json!("unused")] {
            for payload in [
                json!({"result": null}),
                json!({"error": {"code": -32601, "message": "unused"}}),
            ] {
                let mut message = payload;
                message["jsonrpc"] = json!("2.0");
                message["id"] = id.clone();
                assert!(server.handle_message(message).is_none());
            }
        }
        for message in [
            json!({"jsonrpc": "2.0", "result": null}),
            json!({"jsonrpc": "2.0", "id": 1, "result": null, "error": null}),
            json!({"jsonrpc": "2.0", "id": 1, "error": null}),
            json!({"jsonrpc": "2.0", "id": 1, "error": {"code": "1", "message": "bad"}}),
            json!({"jsonrpc": "2.0", "id": 1, "error": {"code": 2147483648_u64, "message": "bad"}}),
            json!({"jsonrpc": "2.0", "id": 1, "error": {"code": 1, "message": null}}),
        ] {
            assert_error(&mut server, message, -32600);
        }
    }

    #[test]
    fn full_sync_keeps_exact_text_and_rejects_invalid_updates_atomically() {
        let mut server = Server::default();
        let original = "\u{feff}begin\r\n😀e\u{301}\r\nend;\n";
        let open = notification(
            "textDocument/didOpen",
            json!({"textDocument": {
            "uri": "file:///a.miz", "languageId": "mizar", "version": -1, "text": original}}),
        );
        assert!(server.handle_message(open.clone()).is_none());
        assert_eq!(server.document("file:///a.miz"), None);
        initialize(&mut server);
        assert!(server.handle_message(open.clone()).is_none());
        assert_eq!(server.document("file:///a.miz"), Some((-1, original)));
        let mut duplicate = open;
        duplicate["params"]["textDocument"]["text"] = json!("duplicate");
        server.handle_message(duplicate);
        for version in [json!(-2), json!(-1), json!(0.5), json!(2147483648_u64)] {
            server.handle_message(notification("textDocument/didChange", json!({
                "textDocument": {"uri": "file:///a.miz", "version": version}, "contentChanges": [{"text": "stale"}]})));
            assert_eq!(server.document("file:///a.miz"), Some((-1, original)));
        }
        for invalid in [
            Value::Null,
            json!("bad"),
            json!({}),
            json!({"text": null}),
            json!({"text": "partial", "range": null}),
            json!({"text": "partial", "rangeLength": 0}),
        ] {
            for invalid_last in [false, true] {
                let mut changes = [invalid.clone(), json!({"text": "must not be committed"})];
                if invalid_last {
                    changes.swap(0, 1);
                }
                assert!(
                    server
                        .handle_message(notification(
                            "textDocument/didChange",
                            json!({
                "textDocument": {"uri": "file:///a.miz", "version": 0},
                "contentChanges": changes})
                        ))
                        .is_none()
                );
                assert_eq!(server.document("file:///a.miz"), Some((-1, original)));
            }
        }
        server.handle_message(notification(
            "textDocument/didChange",
            json!({
            "textDocument": {"uri": "file:///a.miz", "version": 0}, "contentChanges": []}),
        ));
        assert_eq!(server.document("file:///a.miz"), Some((0, original)));
        let changed = "\u{feff}new😀\r\n";
        server.handle_message(notification(
            "textDocument/didChange",
            json!({
            "textDocument": {"uri": "file:///a.miz", "version": i32::MAX},
            "contentChanges": [{"text": "first"}, {"text": changed}]}),
        ));
        assert_eq!(server.document("file:///a.miz"), Some((i32::MAX, changed)));
        server.handle_message(notification("textDocument/didChange", json!({
            "textDocument": {"uri": "file:///unknown", "version": 0}, "contentChanges": [{"text": "unknown"}]})));
        server.handle_message(notification(
            "textDocument/didClose",
            json!({"textDocument": {"uri": "file:///unknown"}}),
        ));
        assert_eq!(server.document("file:///unknown"), None);
        server.handle_message(notification(
            "textDocument/didClose",
            json!({"textDocument": {"uri": "file:///a.miz"}}),
        ));
        assert_eq!(server.document("file:///a.miz"), None);
    }

    #[test]
    fn invalid_open_close_and_wrong_message_kind_leave_buffers_unchanged() {
        let mut server = Server::default();
        initialize(&mut server);
        let document =
            json!({"uri": "file:///a.miz", "version": 1, "languageId": "mizar", "text": "kept"});
        for (field, value) in [
            ("uri", json!("")),
            ("uri", Value::Null),
            ("version", json!(2147483648_u64)),
            ("version", json!(-2147483649_i64)),
            ("version", json!(1.5)),
            ("languageId", Value::Null),
            ("text", Value::Null),
        ] {
            let mut invalid = document.clone();
            invalid[field] = value;
            server.handle_message(notification(
                "textDocument/didOpen",
                json!({"textDocument": invalid}),
            ));
            assert_eq!(server.document("file:///a.miz"), None);
        }
        server.handle_message(notification(
            "textDocument/didOpen",
            json!({"textDocument": document}),
        ));
        for method in [
            "textDocument/didOpen",
            "textDocument/didChange",
            "textDocument/didClose",
        ] {
            assert_error(
                &mut server,
                request(
                    method,
                    json!({"textDocument": {
                "uri": "file:///a.miz", "version": 2, "languageId": "mizar", "text": "wrong kind"},
                "contentChanges": [{"text": "wrong kind"}]}),
                ),
                -32600,
            );
        }
        for params in [
            Value::Null,
            json!([]),
            json!({}),
            json!({"textDocument": {"uri": null}}),
        ] {
            for method in ["textDocument/didChange", "textDocument/didClose"] {
                assert!(
                    server
                        .handle_message(notification(method, params.clone()))
                        .is_none()
                );
            }
        }
        assert_eq!(server.document("file:///a.miz"), Some((1, "kept")));
        server.handle_message(request("shutdown", Value::Null));
        for exit in [false, true] {
            if exit {
                server.handle_message(notification("exit", Value::Null));
            }
            for method in [
                "textDocument/didOpen",
                "textDocument/didChange",
                "textDocument/didClose",
            ] {
                let uri = if method == "textDocument/didOpen" {
                    "file:///after.miz"
                } else {
                    "file:///a.miz"
                };
                assert!(
                    server
                        .handle_message(notification(
                            method,
                            json!({"textDocument": {
                    "uri": uri, "version": 2, "languageId": "mizar", "text": "terminal"},
                    "contentChanges": [{"text": "terminal"}]})
                        ))
                        .is_none()
                );
            }
            assert_eq!(server.document("file:///a.miz"), Some((1, "kept")));
            assert_eq!(server.document("file:///after.miz"), None);
        }
        assert!(
            server
                .handle_message(request("initialize", Value::Null))
                .is_none()
        );
    }

    #[test]
    fn exit_without_shutdown_returns_one_and_invalid_exit_does_not_terminate() {
        for initialized in [false, true] {
            let mut server = Server::default();
            if initialized {
                initialize(&mut server);
            }
            for params in [json!({}), json!([]), json!(1)] {
                server.handle_message(notification("exit", params));
                assert_eq!(server.exit_code, None);
            }
            server.handle_message(json!({"jsonrpc": "2.0", "method": "exit"}));
            assert_eq!(
                server.serve(&mut Cursor::new([]), &mut Vec::new()).unwrap(),
                1
            );
        }
    }

    #[test]
    fn fragmented_unicode_wire_traffic_uses_byte_lengths_and_silent_notifications() {
        let mut initialize = request(
            "initialize",
            json!({"processId": null, "rootUri": null, "capabilities": {}}),
        );
        initialize["id"] = json!("初期化😀");
        let messages = [
            initialize,
            notification(
                "textDocument/didOpen",
                json!({"textDocument": {"uri": "file:///あ.miz",
                "version": 1, "languageId": "mizar", "text": "\u{feff}😀\r\n"}}),
            ),
            notification(
                "textDocument/didChange",
                json!({"textDocument": {"uri": "file:///あ.miz", "version": 2},
                "contentChanges": [{"text": "é\r\n😀"}]}),
            ),
            notification("$/cancelRequest", json!({"id": "absent"})),
            request("shutdown", Value::Null),
            notification("exit", Value::Null),
        ];
        let bytes: Vec<_> = messages
            .iter()
            .flat_map(|message| frame(message.to_string()))
            .collect();
        for capacity in [1, 2, 3, 17] {
            let mut server = Server::default();
            let mut output = Vec::new();
            assert_eq!(
                server
                    .serve(
                        &mut BufReader::with_capacity(capacity, Cursor::new(&bytes)),
                        &mut output
                    )
                    .unwrap(),
                0
            );
            let replies = responses(&output);
            assert_eq!(replies.len(), 2);
            assert_eq!(replies[0]["id"], "初期化😀");
            assert_eq!(
                replies[1],
                json!({"jsonrpc": "2.0", "id": 7, "result": null})
            );
            assert_eq!(server.document("file:///あ.miz"), Some((2, "é\r\n😀")));
        }
    }

    #[test]
    fn complete_bad_frames_report_parse_errors_and_continue() {
        let mut bytes = frame(b"not JSON");
        bytes.extend(frame([0xff]));
        for content_type in [
            "application/json",
            "application/vscode-jsonrpc; charset=latin1",
            "application/vscode-jsonrpc; charset=",
        ] {
            bytes.extend_from_slice(
                format!("Content-Length: 2\r\nContent-Type: {content_type}\r\n\r\n{{}}").as_bytes(),
            );
        }
        bytes.extend(frame(
            request(
                "initialize",
                json!({"processId": null, "rootUri": null, "capabilities": {}}),
            )
            .to_string(),
        ));
        bytes.extend(frame(notification("exit", Value::Null).to_string()));
        let mut output = Vec::new();
        assert_eq!(
            Server::default()
                .serve(
                    &mut BufReader::with_capacity(2, Cursor::new(bytes)),
                    &mut output
                )
                .unwrap(),
            1
        );
        let replies = responses(&output);
        assert_eq!(replies.len(), 6);
        for reply in &replies[..5] {
            assert_eq!(reply.get("id"), Some(&Value::Null));
            assert_eq!(reply["error"]["code"], -32700);
        }
        assert!(replies[5].get("result").is_some());
    }

    #[test]
    fn case_insensitive_headers_unknown_headers_and_utf8_charsets_are_accepted() {
        for content_type in [
            "",
            "Content-Type: application/vscode-jsonrpc\r\n",
            "CONTENT-TYPE: Application/Vscode-Jsonrpc; Charset=UTF8\r\n",
            "content-type: application/vscode-jsonrpc; charset=\"utf-8\"\r\n",
        ] {
            let body = notification("exit", Value::Null).to_string();
            let bytes = format!(
                "cOnTeNt-LeNgTh: {}\r\nX-Unknown: ignored\r\nX9!#$%&'*+-.^_`|~: valid\r\n{content_type}\r\n{body}",
                body.len()
            );
            let mut output = Vec::new();
            assert_eq!(
                Server::default()
                    .serve(
                        &mut BufReader::with_capacity(1, Cursor::new(bytes)),
                        &mut output
                    )
                    .unwrap(),
                1
            );
            assert!(responses(&output).is_empty());
        }
    }

    #[test]
    fn invalid_framing_truncation_and_early_eof_are_io_errors() {
        for bytes in [b"".as_slice(), b"Content-Length: 2\r\n", b"Content-Length: 2\r\n\r\n{",
            b"Content-Length: 2\n\n{}", b"Content-Length: 2\r\ncontent-length: 2\r\n\r\n{}",
            b"Content-Length: -1\r\n\r\n", b"Content-Length: +1\r\n\r\n",
            b"Content-Length: nope\r\n\r\n", b"Content-Length:\r\n\r\n",
            b"Content-Length: 99999999999999999999999\r\n\r\n", b"Unknown: value\r\n\r\n",
            b"bad header\r\n\r\n", b"Content-Length: 0\r\nX: \xff\r\n\r\n",
            b"Content-Length: 0\r\nX: bad\rvalue\r\n\r\n",
            b"Content-Length: 33\r\nX@: ignored\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"exit\"}",
            b"Content-Length: 33\r\nX<: ignored\r\n\r\n{\"jsonrpc\":\"2.0\",\"method\":\"exit\"}",
            b"Content-Length: 0\r\nContent-Type: application/vscode-jsonrpc\r\ncontent-type: application/vscode-jsonrpc\r\n\r\n"] {
            let mut server = Server::default();
            let mut output = Vec::new();
            assert!(server.serve(&mut BufReader::with_capacity(1, Cursor::new(bytes)), &mut output).is_err(), "{bytes:?}");
            assert!(output.is_empty());
            assert_eq!(server.exit_code, None);
        }
        let mut server = Server::default();
        initialize(&mut server);
        let bytes = frame(request("shutdown", Value::Null).to_string());
        assert_eq!(
            server
                .serve(&mut Cursor::new(bytes), &mut Vec::new())
                .unwrap_err()
                .kind(),
            io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn header_and_body_bounds_are_checked_before_body_reads() {
        let body = notification("exit", Value::Null).to_string();
        let base = format!("Content-Length: {}\r\nX: ", body.len());
        let header = format!("{base}{}\r\n\r\n", "a".repeat(8192 - base.len() - 4));
        assert_eq!(header.len(), 8192);
        let bytes = format!("{header}{body}");
        assert_eq!(
            Server::default()
                .serve(&mut Cursor::new(bytes), &mut Vec::new())
                .unwrap(),
            1
        );
        for bytes in [
            header.replace("\r\n\r\n", "x\r\n\r\n"),
            "a".repeat(8193),
            "Content-Length: 16777217\r\n\r\n".to_owned(),
        ] {
            let mut input = Cursor::new(bytes);
            let error = Server::default()
                .serve(&mut input, &mut Vec::new())
                .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        }
        let mut body = vec![b' '; 16 * 1024 * 1024];
        body[..2].copy_from_slice(b"{}");
        let mut bytes = frame(body);
        bytes.extend(frame(notification("exit", Value::Null).to_string()));
        let mut output = Vec::new();
        assert_eq!(
            Server::default()
                .serve(&mut Cursor::new(bytes), &mut output)
                .unwrap(),
            1
        );
        assert_eq!(responses(&output)[0]["error"]["code"], -32600);
    }

    #[test]
    fn output_write_failures_propagate() {
        let bytes = frame(request("unknown", Value::Null).to_string());
        assert_eq!(
            Server::default()
                .serve(&mut Cursor::new(bytes), &mut [].as_mut_slice())
                .unwrap_err()
                .kind(),
            io::ErrorKind::WriteZero
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn input_read_and_output_flush_failures_propagate() {
        let directory = std::fs::File::open(std::env::temp_dir()).unwrap();
        assert!(
            Server::default()
                .serve(&mut BufReader::new(directory), &mut Vec::new())
                .is_err()
        );
        let sink = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let mut output = io::BufWriter::with_capacity(4096, sink);
        let bytes = frame(request("unknown", Value::Null).to_string());
        assert!(
            Server::default()
                .serve(&mut Cursor::new(bytes), &mut output)
                .is_err()
        );
    }

    #[test]
    fn raw_initialize_params_remain_exact_through_document_and_terminal_traffic() {
        for (root, options) in [
            (Value::Null, None),
            (json!("file:///未変換%20😀"), Some(Value::Null)),
            (Value::Null, Some(json!(false))),
            (
                json!("untouched:ルート"),
                Some(json!({"nested": [null, 3, "é😀"]})),
            ),
        ] {
            let mut server = Server::default();
            assert_eq!(server.initialization_params(), None);
            for invalid in [Value::Null, json!({"processId": null, "rootUri": null})] {
                assert_error(&mut server, request("initialize", invalid), -32602);
                assert_eq!(server.initialization_params(), None);
            }
            let mut expected = json!({
                "processId": -1, "rootUri": root,
                "capabilities": {"unknown": {"array": [true, null, "能力😀"]}},
                "unknown": [null, false, 1.5, {"raw": "\u{feff}\r\n"}]
            });
            if let Some(options) = options {
                expected["initializationOptions"] = options;
            }
            let mut original = expected.clone();
            assert_eq!(
                server.handle_message(request("initialize", original.clone())),
                Some(json!({
                    "jsonrpc": "2.0", "id": 7, "result": {"capabilities": {
                        "positionEncoding": "utf-16", "textDocumentSync": {"openClose": true, "change": 1}
                    }}
                }))
            );
            original["rootUri"] = json!("replaced");
            original["capabilities"]["unknown"] = Value::Null;
            assert_eq!(server.initialization_params(), Some(&expected));
            for params in [original.clone(), Value::Null] {
                assert_error(&mut server, request("initialize", params), -32600);
                assert_eq!(server.initialization_params(), Some(&expected));
            }
            let uri = "untouched:文書%20😀";
            let text = "\u{feff}begin\r\n😀\r\n";
            assert!(server.handle_message(notification("textDocument/didOpen", json!({
                "textDocument": {"uri": uri, "languageId": "mizar", "version": -1, "text": text}
            }))).is_none());
            assert_eq!(server.document(uri), Some((-1, text)));
            assert_eq!(server.initialization_params(), Some(&expected));
            let changed = "\u{feff}変更😀\r\n";
            assert!(server.handle_message(notification("textDocument/didChange", json!({
                "textDocument": {"uri": uri, "version": 0}, "contentChanges": [{"text": changed}]
            }))).is_none());
            assert_eq!(server.document(uri), Some((0, changed)));
            assert!(
                server
                    .handle_message(notification("$/cancelRequest", json!({"id": 7})))
                    .is_none()
            );
            assert_eq!(server.document(uri), Some((0, changed)));
            assert_eq!(server.initialization_params(), Some(&expected));
            assert!(
                server
                    .handle_message(notification(
                        "textDocument/didClose",
                        json!({
                            "textDocument": {"uri": uri}
                        })
                    ))
                    .is_none()
            );
            assert_eq!(server.document(uri), None);
            assert_eq!(server.initialization_params(), Some(&expected));
            assert_eq!(
                server.handle_message(request("shutdown", Value::Null)),
                Some(json!({"jsonrpc": "2.0", "id": 7, "result": null}))
            );
            assert_error(&mut server, request("initialize", original.clone()), -32600);
            assert_eq!(server.initialization_params(), Some(&expected));
            assert!(
                server
                    .handle_message(notification("exit", Value::Null))
                    .is_none()
            );
            assert!(
                server
                    .handle_message(request("initialize", original))
                    .is_none()
            );
            assert_eq!(server.initialization_params(), Some(&expected));
        }
    }

    #[test]
    fn raw_initialize_params_survive_framed_rejection_and_silent_notifications() {
        let expected = json!({
            "processId": null, "rootUri": "file:///ルート%20😀", "capabilities": {"future": [null, false]},
            "initializationOptions": {"raw": ["\u{feff}\r\n", 9]}, "unknown": {"nested": true}
        });
        let uri = "file:///文書%20😀.miz";
        let text = "\u{feff}編集😀\r\n";
        let mut bytes = Vec::new();
        for message in [
            request("initialize", Value::Null),
            request("initialize", expected.clone()),
            notification("initialized", json!({"unknown": true})),
            notification(
                "textDocument/didOpen",
                json!({"textDocument": {
                    "uri": uri, "languageId": "mizar", "version": -2, "text": "initial\r\n"
                }}),
            ),
            notification(
                "textDocument/didChange",
                json!({
                    "textDocument": {"uri": uri, "version": -1}, "contentChanges": [{"text": text}]
                }),
            ),
            notification("$/cancelRequest", json!({"id": 7})),
            request(
                "initialize",
                json!({"processId": 42, "rootUri": null, "capabilities": {}}),
            ),
            request("shutdown", Value::Null),
            notification(
                "textDocument/didClose",
                json!({"textDocument": {"uri": uri}}),
            ),
            request("initialize", Value::Null),
            notification("exit", Value::Null),
            request(
                "initialize",
                json!({"processId": 42, "rootUri": null, "capabilities": {}}),
            ),
        ] {
            bytes.extend(frame(message.to_string()));
        }
        let mut server = Server::default();
        let mut output = Vec::new();
        assert_eq!(
            server
                .serve(
                    &mut BufReader::with_capacity(2, Cursor::new(bytes)),
                    &mut output
                )
                .unwrap(),
            0
        );
        assert_eq!(
            responses(&output),
            vec![
                json!({"jsonrpc": "2.0", "id": 7, "error": {"code": -32602, "message": "Invalid params"}}),
                json!({"jsonrpc": "2.0", "id": 7, "result": {"capabilities": {
                    "positionEncoding": "utf-16", "textDocumentSync": {"openClose": true, "change": 1}
                }}}),
                json!({"jsonrpc": "2.0", "id": 7, "error": {"code": -32600, "message": "Invalid Request"}}),
                json!({"jsonrpc": "2.0", "id": 7, "result": null}),
                json!({"jsonrpc": "2.0", "id": 7, "error": {"code": -32600, "message": "Invalid Request"}}),
            ]
        );
        assert_eq!(server.initialization_params(), Some(&expected));
        assert_eq!(server.document(uri), Some((-1, text)));
    }

    #[test]
    fn native_workspace_inputs_follow_initialize_and_reacquire_real_metadata() {
        use std::{fs, time::SystemTime};

        let root = std::env::temp_dir().join(format!(
            "mizar lsp native inputs {} {}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let uri = format!(
            "file://{}{}",
            if cfg!(windows) { "/" } else { "" },
            root.to_str()
                .unwrap()
                .replace('\\', "/")
                .replace(' ', "%20")
        );
        let manifest = "[package]\nname = \"actual_package\"\nversion = \"1.2.3\"\n";
        let lock = "schema_version = 1\n[[package]]\nname = \"actual_package\"\nversion = \"1.2.3\"\nsource = { kind = \"workspace\", path = \".\" }\ndependencies = []\n";
        fs::write(root.join("mizar.pkg"), manifest).unwrap();
        fs::write(root.join("mizar.lock"), lock).unwrap();
        let params = json!({"processId": null, "rootUri": format!("{uri}/."),
            "capabilities": {}, "initializationOptions": {"unknown": [1, null]}});
        let mut server = Server::default();
        assert_eq!(
            server.handle_message(request("initialize", params.clone())),
            Some(
                json!({"jsonrpc": "2.0", "id": 7, "result": {"capabilities": {
                    "positionEncoding": "utf-16", "textDocumentSync": {"openClose": true, "change": 1}
                }}})
            )
        );
        let document_uri = format!("{uri}/unsaved.miz");
        let text = "\u{feff}editor😀\r\n";
        assert!(server.handle_message(notification("textDocument/didOpen", json!({
            "textDocument": {"uri": document_uri, "languageId": "mizar", "version": 3, "text": text}
        }))).is_none());
        let (canonical, packages, parsed_lock) = server.native_workspace_inputs().unwrap().unwrap();
        assert_eq!(
            canonical.as_str(),
            fs::canonicalize(&root).unwrap().to_str().unwrap()
        );
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].member_path, ".");
        assert_eq!(
            packages[0].manifest,
            mizar_build::planner::parse_package_manifest(manifest).unwrap()
        );
        assert_eq!(
            parsed_lock,
            mizar_build::planner::parse_lockfile(lock).unwrap()
        );

        fs::create_dir_all(root.join("members/nested")).unwrap();
        let changed_manifest = manifest.replace("1.2.3", "2.3.4");
        let changed_lock = lock
            .replace("1.2.3", "2.3.4")
            .replace("path = \".\"", "path = \"members/nested\"");
        fs::write(root.join("members/nested/mizar.pkg"), &changed_manifest).unwrap();
        fs::write(
            root.join("mizar.workspace"),
            "[workspace]\nmembers = [\"members/nested\"]\n",
        )
        .unwrap();
        fs::write(root.join("mizar.lock"), &changed_lock).unwrap();
        assert_error(
            &mut server,
            request(
                "initialize",
                json!({
                    "processId": null, "rootUri": null, "capabilities": {}
                }),
            ),
            -32600,
        );
        assert_eq!(
            server.handle_message(request("shutdown", Value::Null)),
            Some(json!({"jsonrpc": "2.0", "id": 7, "result": null}))
        );
        assert!(
            server
                .handle_message(notification("exit", Value::Null))
                .is_none()
        );
        let (current_root, current_packages, current_lock) =
            server.native_workspace_inputs().unwrap().unwrap();
        assert_eq!(current_root, canonical);
        assert_eq!(current_packages.len(), 1);
        assert_eq!(current_packages[0].member_path, "members/nested");
        assert_eq!(
            current_packages[0].manifest,
            mizar_build::planner::parse_package_manifest(&changed_manifest).unwrap()
        );
        assert_eq!(
            current_lock,
            mizar_build::planner::parse_lockfile(&changed_lock).unwrap()
        );
        fs::remove_file(root.join("mizar.lock")).unwrap();
        assert!(server.native_workspace_inputs().is_err());
        assert_eq!(server.initialization_params(), Some(&params));
        assert_eq!(server.document(&document_uri), Some((3, text)));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native_workspace_inputs_refuse_absent_invalid_roots_and_reader_failures() {
        use std::{fs, time::SystemTime};

        let root = std::env::temp_dir().join(format!(
            "mizar-lsp-native-errors-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        fs::write(
            root.join("mizar.pkg"),
            "[package]\nname = \"actual_package\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        fs::write(
            root.join("mizar.lock"),
            "schema_version = 1\npackage = []\n",
        )
        .unwrap();
        let uri = format!(
            "file://{}{}",
            if cfg!(windows) { "/" } else { "" },
            root.to_str()
                .unwrap()
                .replace('\\', "/")
                .replace(' ', "%20")
        );
        let mut server = Server::default();
        assert_eq!(server.native_workspace_inputs(), Ok(None));
        let fallback = json!({"processId": null, "rootUri": null, "capabilities": {},
            "rootPath": root, "workspaceFolders": [{"uri": uri, "name": "actual"}],
            "initializationOptions": {"rootUri": uri}, "settings": {"rootUri": uri}});
        assert!(
            server
                .handle_message(request("initialize", fallback.clone()))
                .unwrap()
                .get("result")
                .is_some()
        );
        assert_eq!(server.native_workspace_inputs(), Ok(None));
        assert_eq!(server.initialization_params(), Some(&fallback));
        for invalid in [
            "https://example.com/workspace".to_owned(),
            "file:///bad%GG".to_owned(),
            format!("{uri}/missing"),
            format!("{uri}/mizar.pkg"),
        ] {
            let mut server = Server::default();
            assert!(server.handle_message(request("initialize", json!({
                "processId": null, "rootUri": invalid, "capabilities": {}, "rootPath": root,
                "workspaceFolders": [{"uri": uri, "name": "actual"}]
            }))).unwrap().get("result").is_some());
            assert!(server.native_workspace_inputs().is_err(), "{invalid}");
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&root, root.join("linked-root")).unwrap();
            for suffix in ["/linked-root", "/linked-root/."] {
                let mut server = Server::default();
                server.handle_message(request("initialize", json!({
                    "processId": null, "rootUri": format!("{uri}{suffix}"), "capabilities": {}
                }))).unwrap();
                assert!(server.native_workspace_inputs().is_err(), "{suffix}");
            }
        }
        let mut server = Server::default();
        server
            .handle_message(request(
                "initialize",
                json!({
                    "processId": null, "rootUri": uri, "capabilities": {}
                }),
            ))
            .unwrap();
        assert!(server.native_workspace_inputs().unwrap().is_some());
        fs::write(
            root.join("mizar.lock"),
            "schema_version = 0\npackage = []\n",
        )
        .unwrap();
        let canonical =
            mizar_session::WorkspaceRoot::new(fs::canonicalize(&root).unwrap().to_str().unwrap());
        assert_eq!(
            server.native_workspace_inputs().unwrap_err(),
            mizar_build::planner::read_native_workspace_inputs(&canonical).unwrap_err()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn native_workspace_inputs_refuse_drive_relative_windows_roots() {
        use std::{
            fs,
            path::{Component, Prefix},
            time::SystemTime,
        };

        let current = std::env::current_dir().unwrap();
        let drive = match current.components().next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => drive,
                _ => {
                    eprintln!("drive-relative workspace fixture requires a drive-prefixed cwd");
                    return;
                }
            },
            _ => unreachable!("Windows current directory has a path prefix"),
        };
        let name = format!(
            "mizar-lsp-drive-relative-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let root = current.join(&name);
        fs::create_dir(&root).unwrap();
        fs::write(
            root.join("mizar.pkg"),
            "[package]\nname = \"actual_package\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        fs::write(
            root.join("mizar.lock"),
            "schema_version = 1\npackage = []\n",
        )
        .unwrap();
        let uri = format!("file:///{}:{name}", char::from(drive));
        let decoded = mizar_session::source::file_path_from_document_uri(&uri).unwrap();
        assert!(!decoded.is_absolute());
        assert!(fs::symlink_metadata(&decoded).unwrap().is_dir());
        let canonical = fs::canonicalize(&root).unwrap();
        assert_eq!(fs::canonicalize(&decoded).unwrap(), canonical);
        let workspace = mizar_session::WorkspaceRoot::new(canonical.to_str().unwrap());
        assert!(mizar_build::planner::read_native_workspace_inputs(&workspace).is_ok());
        let mut server = Server::default();
        assert!(
            server
                .handle_message(request(
                    "initialize",
                    json!({
                        "processId": null, "rootUri": uri, "capabilities": {}
                    })
                ))
                .unwrap()
                .get("result")
                .is_some()
        );
        assert_eq!(
            server.native_workspace_inputs().unwrap_err(),
            "workspace root must be absolute"
        );
        fs::remove_dir_all(root).unwrap();
    }
}
