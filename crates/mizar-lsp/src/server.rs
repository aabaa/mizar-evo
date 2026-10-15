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

    /// Submits a native capture and returns current all-OpenBuffer frontend notification data.
    pub fn submit_native_frontend_diagnostics<
        A: mizar_session::SessionIdAllocator,
        L: mizar_build::module_index::SourceLayoutProvider,
    >(
        &self,
        driver: &mut mizar_driver::driver::CompilerDriver,
        session: mizar_driver::request::BuildSession,
        allocator: &A,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        input: mizar_driver::driver::DriverSubmitInput<L>,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, Option<Vec<Value>>),
        mizar_driver::driver::DriverSubmitError,
    > {
        use mizar_build::task_graph::{PipelinePhase as BuildPhase, TaskKind, WorkUnit};
        use mizar_diagnostics::{
            failure_record::{DiagnosticFreshness, DiagnosticPrimaryLocation, PipelinePhase},
            registry::DiagnosticSeverity,
        };
        use mizar_driver::registry::PhaseStatus;
        let (submission, index) = driver
            .submit_captured_with_native_diagnostic_index(session, allocator, snapshots, input)?;
        let notifications = (|| {
            if self.initialized.is_none() || self.shutdown || self.exit_code.is_some() {
                return None;
            }
            let index = index.as_ref()?;
            let snapshot = submission.session.captured().snapshot().id;
            if !index.obsolete_drafts().is_empty() {
                return None;
            }
            driver
                .validate_current_native_diagnostics(submission.session.id(), snapshots)
                .ok()?;
            let sources = driver
                .session(submission.session.id())?
                .captured()
                .native_sources()?;
            if sources.is_empty() {
                return None;
            }
            let graph = submission.task_graph.as_ref()?;
            let run = submission.scheduler_run.as_ref()?;
            let mut by_source = std::collections::HashMap::new();
            let mut by_uri = BTreeMap::new();
            for (_, loaded, input) in sources {
                let mizar_session::SourceOriginInput::OpenBuffer {
                    uri,
                    actual_version,
                    text,
                    ..
                } = &input.origin
                else {
                    return None;
                };
                let version = i32::try_from(*actual_version).ok()?;
                if self.document(uri) != Some((version, text.as_ref()))
                    || by_source
                        .insert(loaded.source_id, (loaded, input, uri))
                        .is_some()
                    || by_uri.insert(uri, (version, Vec::new())).is_some()
                {
                    return None;
                }
                // Clean captures must also have valid original maps and version context.
                crate::lsp_range_from_open_buffer(
                    &loaded.line_map,
                    loaded.loading_map.as_ref(),
                    &input.origin,
                    mizar_session::SourceRange {
                        source_id: loaded.source_id,
                        start: 0,
                        end: 0,
                    },
                )
                .ok()?;
                let mut tasks = graph.tasks().iter().filter(|task| {
                    task.kind == TaskKind::Frontend
                        && matches!(&task.unit, WorkUnit::Module { module }
                            if module.package == input.package_id && module.path == input.module_path)
                });
                let task = tasks.next()?;
                if tasks.next().is_some() || task.phases != [BuildPhase::Frontend] {
                    return None;
                }
                let [result] = run.phase_results.get(&task.id)?.as_slice() else {
                    return None;
                };
                if !matches!(
                    result.status,
                    PhaseStatus::Complete | PhaseStatus::Recoverable | PhaseStatus::Fatal
                ) || (result.status != PhaseStatus::Complete
                    && !index.records().iter().any(|record| {
                        matches!(record.primary_location(), DiagnosticPrimaryLocation::Span(span)
                            if span.range().source_id == loaded.source_id)
                    }))
                {
                    return None;
                }
            }
            for record in index.records() {
                if record.phase() != PipelinePhase::Frontend
                    || record.freshness()
                        != &(DiagnosticFreshness::Current {
                            source_snapshot: snapshot,
                        })
                {
                    return None;
                }
                let DiagnosticPrimaryLocation::Span(span) = record.primary_location() else {
                    return None;
                };
                let (loaded, input, uri) = by_source.get(&span.range().source_id)?;
                let range = crate::lsp_range_from_open_buffer(
                    &loaded.line_map,
                    loaded.loading_map.as_ref(),
                    &input.origin,
                    span.range(),
                )
                .ok()?;
                let severity = match record.severity() {
                    DiagnosticSeverity::Error => 1,
                    DiagnosticSeverity::Warning => 2,
                    DiagnosticSeverity::Info => 3,
                    _ => return None,
                };
                by_uri.get_mut(uri)?.1.push(json!({
                    "range": {"start": {"line": range.start.line, "character": range.start.character},
                        "end": {"line": range.end.line, "character": range.end.character}},
                    "code": record.code().to_string(), "message": record.message(), "severity": severity
                }));
            }
            let notifications = by_uri
                .into_iter()
                .map(|(uri, (version, diagnostics))| {
                    json!({
                        "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                        "params": {"uri": uri, "version": version, "diagnostics": diagnostics}
                    })
                })
                .collect();
            driver
                .validate_current_native_diagnostics(submission.session.id(), snapshots)
                .ok()?;
            Some(notifications)
        })();
        Ok((submission, notifications))
    }

    /// Submits native frontend diagnostics and writes each currently eligible LSP frame.
    pub fn submit_native_frontend_diagnostics_to_writer<
        A: mizar_session::SessionIdAllocator,
        L: mizar_build::module_index::SourceLayoutProvider,
    >(
        &self,
        driver: &mut mizar_driver::driver::CompilerDriver,
        session: mizar_driver::request::BuildSession,
        allocator: &A,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        input: mizar_driver::driver::DriverSubmitInput<L>,
        output: &mut impl Write,
    ) -> Result<
        (mizar_driver::driver::BuildSubmission, io::Result<usize>),
        mizar_driver::driver::DriverSubmitError,
    > {
        let (submission, notifications) =
            self.submit_native_frontend_diagnostics(driver, session, allocator, snapshots, input)?;
        let delivery = (|| {
            let mut count = 0;
            for notification in notifications.into_iter().flatten() {
                let body = serde_json::to_vec(&notification).map_err(io::Error::other)?;
                if driver
                    .validate_current_native_diagnostics(submission.session.id(), snapshots)
                    .is_err()
                {
                    break;
                }
                write!(output, "Content-Length: {}\r\n\r\n", body.len())?;
                output.write_all(&body)?;
                output.flush()?;
                count += 1;
            }
            Ok(count)
        })();
        Ok((submission, delivery))
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

    #[test]
    fn native_frontend_notifications_preserve_real_submissions_and_original_editor_tuples() {
        use mizar_build::{
            module_index::{StaticSourceLayout, build_module_index},
            planner::{
                DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile,
                parse_package_manifest, produce_build_plan,
            },
            task_graph::{ModuleDependencyOverlay, TaskKind},
        };
        use mizar_driver::{
            driver::{CompilerDriver, DriverSubmitError, DriverSubmitInput},
            registry::{PhaseRegistryBuilder, PhaseStatus},
            request::{
                BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
                BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet,
                SourceInputSet, VerifierConfigInput,
            },
        };
        use mizar_ir::{
            identity::{OutputKind, PipelinePhase as IrPhase, SnapshotHandleRegistry, WorkUnit},
            publisher::{AllowedWorkUnit, PhaseOutputPublisher},
            storage::IrStorageService,
        };
        use mizar_session::{
            DiskSourceLoader, Hash, InMemorySessionIdAllocator, PackageId, SnapshotRegistry,
            SourceOriginInput, ToolchainInfo, WorkspaceRoot,
        };
        use std::{fs, sync::Arc, time::SystemTime};

        for mode in [
            "fatal",
            "clean",
            "version",
            "raw",
            "closed",
            "alias",
            "disk",
            "mixed",
            "generated",
            "resolver",
            "uninitialized",
            "shutdown",
            "exit",
            "missing-publisher",
            "invalid-registry",
            "denied-source",
            "denied-clean-frontend",
        ] {
            let root = std::env::temp_dir().join(format!(
                "mizar-lsp-native-notifications-{}-{}-{mode}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(root.join("alpha/src")).unwrap();
            for name in ["main", "clean"] {
                fs::write(root.join(format!("alpha/src/{name}.miz")), [0xff]).unwrap();
            }
            let workspace = WorkspaceRoot::new(root.to_str().unwrap());
            let plan_request = PlanRequest {
                workspace_root: workspace.clone(),
                dependency_selection: DependencySelection::Normal,
                toolchain: ToolchainInfo::new("mizar-evo-test"),
            };
            let packages = vec![WorkspacePackage {
                member_path: "alpha".into(),
                manifest: parse_package_manifest(
                    "[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n",
                )
                .unwrap(),
            }];
            let lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n").unwrap();
            let plan =
                produce_build_plan(plan_request.clone(), packages.clone(), lock.clone()).unwrap();
            let layout = StaticSourceLayout::from_native_disk(&plan).unwrap();
            let index = build_module_index(&plan, &layout, &[]).unwrap();
            let loader = DiskSourceLoader::new(root.join("alpha"));
            let mut server = Server::default();
            initialize(&mut server);
            let mut descriptors = Vec::new();
            let mut uris = Vec::new();
            for name in ["main", "clean"] {
                let uri = format!(
                    "file://{}{}",
                    if cfg!(windows) { "/" } else { "" },
                    root.join(format!("alpha/src/{name}.miz"))
                        .to_str()
                        .unwrap()
                        .replace('\\', "/")
                );
                let raw = if name == "clean" || mode == "clean" {
                    "\u{feff}definition\r\nend;\r\n:: 🦀\r\n"
                } else if mode == "alias" {
                    "end\n"
                } else if mode == "resolver" {
                    "import mml.no_such;\ndefinition\nend;\n"
                } else {
                    "\u{feff}::=🦀=::end\r\n:: suffix\r"
                };
                server.handle_message(notification(
                    "textDocument/didOpen",
                    json!({"textDocument": {
                    "uri": uri, "languageId": "mizar", "version": 7, "text": raw}}),
                ));
                let (version, text) = server.document(&uri).unwrap();
                let mut descriptor = index
                    .native_open_buffer_input(
                        &workspace,
                        &PackageId::new("alpha"),
                        &loader,
                        SourceOriginInput::OpenBuffer {
                            uri: if mode == "alias" && name == "main" {
                                uri.replace("main.miz", "%6dain.miz")
                            } else {
                                uri.clone()
                            },
                            expected_version: i64::from(version),
                            actual_version: i64::from(version),
                            text: text.into(),
                        },
                    )
                    .unwrap();
                if mode == "disk" || (mode == "mixed" && name == "clean") {
                    fs::write(root.join(format!("alpha/src/{name}.miz")), raw).unwrap();
                    descriptor.origin = SourceOriginInput::Disk {
                        path: format!("src/{name}.miz").into(),
                    };
                } else if mode == "generated" {
                    descriptor.origin = SourceOriginInput::Generated {
                        generator: mizar_session::GeneratedSourceKind::new("lsp-native-test"),
                        text: raw.into(),
                        anchor: None,
                    };
                }
                descriptors.push((loader.clone(), descriptor));
                uris.push(uri);
            }
            let ids = InMemorySessionIdAllocator::new();
            let snapshots = SnapshotRegistry::new();
            let draft = BuildRequestDraft {
                lane: BuildLaneId::new(1),
                origin: BuildRequestOrigin::Batch(BatchRequest {
                    invocation: BatchInvocation::default(),
                }),
                generation: BuildRequestGeneration::new(0),
                workspace_root: workspace,
                profile: BuildProfile::new("check"),
                targets: BuildTargets::default(),
                source_inputs: SourceInputSet { versions: vec![] },
                dependency_inputs: DependencyInputSet::new(
                    vec![],
                    Hash::from_bytes([1; 32]),
                    ToolchainInfo::new("mizar-evo-test"),
                ),
                verifier_config: VerifierConfigInput::new(Hash::from_bytes([2; 32])),
            };
            let session = draft
                .capture_sources(&ids, &snapshots, descriptors)
                .unwrap();
            let snapshot = session.captured().snapshot().id;
            // SourceLoad and Frontend must use the protected capture, including Disk inputs.
            for name in ["main", "clean"] {
                fs::remove_file(root.join(format!("alpha/src/{name}.miz"))).unwrap();
            }
            match mode {
                "version" => {
                    server.handle_message(notification(
                        "textDocument/didChange",
                        json!({
                    "textDocument": {"uri": uris[1], "version": 8}, "contentChanges": []}),
                    ));
                }
                "raw" => {
                    server.handle_message(notification(
                        "textDocument/didClose",
                        json!({"textDocument": {"uri": uris[1]}}),
                    ));
                    server.handle_message(notification("textDocument/didOpen", json!({"textDocument": {
                        "uri": uris[1], "languageId": "mizar", "version": 7, "text": "definition\nend;\n:: 🦀\n"}})));
                    let (_, loaded, input) = session
                        .captured()
                        .native_sources()
                        .unwrap()
                        .iter()
                        .find(|(_, _, input)| input.module_path.as_str() == "clean")
                        .unwrap();
                    assert_eq!(server.document(&uris[1]), Some((7, loaded.text.as_ref())));
                    let SourceOriginInput::OpenBuffer { text, .. } = &input.origin else {
                        unreachable!()
                    };
                    assert_ne!(server.document(&uris[1]).unwrap().1, text.as_ref());
                }
                "closed" => {
                    server.handle_message(notification(
                        "textDocument/didClose",
                        json!({
                    "textDocument": {"uri": uris[1]}}),
                    ));
                }
                "uninitialized" => server = Server::default(),
                "shutdown" => {
                    server.handle_message(request("shutdown", Value::Null));
                }
                "exit" => {
                    server.handle_message(notification("exit", Value::Null));
                }
                _ => {}
            }
            let invalid_registry = SnapshotRegistry::new();
            let registry = if mode == "invalid-registry" {
                &invalid_registry
            } else {
                &snapshots
            };
            let mut ordinary = None;
            let mut original_records = Vec::new();
            for route in ["ordinary", "index", "notifications"] {
                let owner = Arc::new(PhaseOutputPublisher::new(
                    Arc::new(IrStorageService::new()),
                    Arc::new(SnapshotHandleRegistry::new()),
                ));
                owner.register_current_snapshot(snapshot);
                for name in ["main", "clean"] {
                    for (phase, kind) in
                        [("SourceLoad", "SourceUnit"), ("Frontend", "FrontendOutput")]
                    {
                        if (mode == "denied-source" && phase == "SourceLoad")
                            || (mode == "denied-clean-frontend"
                                && name == "clean"
                                && phase == "Frontend")
                        {
                            continue;
                        }
                        owner.allow_work_unit(AllowedWorkUnit::new(
                            IrPhase::new(phase),
                            OutputKind::new(kind),
                            WorkUnit::new(format!("{:?}:{:?}", "alpha", name)),
                        ));
                    }
                }
                let mut builder = PhaseRegistryBuilder::new();
                builder.register_source_load();
                builder.register_frontend(vec![]);
                let mut driver = CompilerDriver::new(builder.build().unwrap());
                if mode != "missing-publisher" {
                    driver = driver.with_output_publisher(owner);
                }
                let mut input = DriverSubmitInput::new(
                    plan_request.clone(),
                    packages.clone(),
                    lock.clone(),
                    layout.clone(),
                );
                input.dependency_overlay = ModuleDependencyOverlay::complete(vec![]);
                let result = match route {
                    "ordinary" => driver
                        .submit_captured(session.clone(), &ids, registry, input)
                        .map(|s| (s, None)),
                    "index" => driver
                        .submit_captured_with_native_diagnostic_index(
                            session.clone(),
                            &ids,
                            registry,
                            input,
                        )
                        .map(|(s, index)| {
                            assert_eq!(
                                index.is_some(),
                                !matches!(mode, "generated" | "missing-publisher"),
                                "{mode}"
                            );
                            if let Some(index) = index {
                                original_records = index.records().to_vec();
                                if mode == "denied-source" {
                                    assert!(index.is_empty());
                                } else if mode == "denied-clean-frontend" {
                                    assert_eq!(
                                        original_records
                                            .iter()
                                            .map(|record| record.code().number())
                                            .collect::<Vec<_>>(),
                                        vec![33, 52]
                                    );
                                }
                                if mode == "resolver" {
                                    assert!(original_records.iter().any(|record| {
                                        record.phase() ==
                                        mizar_diagnostics::failure_record::PipelinePhase::Resolver
                                    }));
                                }
                            }
                            (s, None)
                        }),
                    _ => server.submit_native_frontend_diagnostics(
                        &mut driver,
                        session.clone(),
                        &ids,
                        registry,
                        input,
                    ),
                };
                if mode == "invalid-registry" {
                    assert!(
                        matches!(
                            &result,
                            Err(DriverSubmitError::InvalidCapturedSession { .. })
                        ),
                        "{mode}/{route}: {result:?}"
                    );
                } else {
                    assert!(result.is_ok(), "{mode}/{route}: {result:?}");
                }
                if route == "ordinary" {
                    ordinary = Some(result.map(|(submission, _)| submission));
                    continue;
                }
                let (submission, notifications) = match result {
                    Ok(value) => {
                        assert_eq!(
                            Ok(value.0.clone()),
                            *ordinary.as_ref().unwrap(),
                            "{mode}/{route}"
                        );
                        value
                    }
                    Err(error) => {
                        assert_eq!(Err(error), *ordinary.as_ref().unwrap(), "{mode}/{route}");
                        continue;
                    }
                };
                if !matches!(mode, "generated" | "missing-publisher") {
                    assert_eq!(
                        driver.validate_current_native_diagnostics(session.id(), registry),
                        Ok(()),
                        "{mode}"
                    );
                    let run = submission.scheduler_run.as_ref().unwrap();
                    if matches!(mode, "denied-source" | "denied-clean-frontend") {
                        for (_, _, input) in driver
                            .session(session.id())
                            .unwrap()
                            .captured()
                            .native_sources()
                            .unwrap()
                        {
                            let SourceOriginInput::OpenBuffer {
                                uri,
                                actual_version,
                                text,
                                ..
                            } = &input.origin
                            else {
                                unreachable!()
                            };
                            assert_eq!(
                                server.document(uri),
                                Some((i32::try_from(*actual_version).unwrap(), text.as_ref()))
                            );
                        }
                    }
                    for task in submission
                        .task_graph
                        .as_ref()
                        .unwrap()
                        .tasks()
                        .iter()
                        .filter(|task| task.kind == TaskKind::SourceLoad)
                    {
                        let source_results = &run.phase_results[&task.id];
                        assert_eq!(source_results.len(), 1);
                        assert_eq!(
                            source_results[0].status,
                            if mode == "denied-source" {
                                PhaseStatus::Blocking
                            } else {
                                PhaseStatus::Complete
                            }
                        );
                        assert!(source_results[0].diagnostics.is_empty());
                        if matches!(mode, "denied-source" | "denied-clean-frontend") {
                            let frontend = submission
                                .task_graph
                                .as_ref()
                                .unwrap()
                                .tasks()
                                .iter()
                                .find(|frontend| {
                                    frontend.kind == TaskKind::Frontend
                                        && frontend.unit == task.unit
                                })
                                .unwrap();
                            if mode == "denied-source" {
                                assert!(source_results[0].output_refs.is_empty());
                                assert!(!run.phase_results.contains_key(&frontend.id));
                            } else {
                                let frontend_results = &run.phase_results[&frontend.id];
                                assert_eq!(frontend_results.len(), 1);
                                let mizar_build::task_graph::WorkUnit::Module { module } =
                                    &task.unit
                                else {
                                    unreachable!()
                                };
                                assert_eq!(
                                    frontend_results[0].status,
                                    if module.path.as_str() == "clean" {
                                        PhaseStatus::Blocking
                                    } else {
                                        PhaseStatus::Fatal
                                    }
                                );
                                assert_eq!(
                                    frontend_results[0].diagnostics.is_empty(),
                                    module.path.as_str() == "clean"
                                );
                            }
                        }
                    }
                }
                if route == "notifications" {
                    if !matches!(mode, "fatal" | "clean") {
                        assert_eq!(notifications, None, "{mode}");
                        if mode == "alias" {
                            assert!(
                                driver
                                    .session(session.id())
                                    .unwrap()
                                    .captured()
                                    .native_sources()
                                    .unwrap()
                                    .iter()
                                    .find(|(_, _, input)| input.module_path.as_str() == "main")
                                    .unwrap()
                                    .1
                                    .loading_map
                                    .is_none()
                            );
                        }
                        continue;
                    }
                    let notifications = notifications.unwrap();
                    let expected_diagnostics = if mode == "clean" {
                        vec![]
                    } else {
                        vec![
                            json!({"range": {"start": {"line": 0, "character": 9},
                            "end": {"line": 0, "character": 12}}, "code": "E0033",
                            "message": "unmatched `end` in lexical scope skeleton", "severity": 1}),
                            json!({"range": {"start": {"line": 0, "character": 9},
                            "end": {"line": 0, "character": 12}}, "code": "E0052",
                            "message": "`end` has no matching block opener", "severity": 1}),
                        ]
                    };
                    assert_eq!(
                        notifications,
                        vec![
                            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                            "params": {"uri": uris[1], "version": 7, "diagnostics": []}}),
                            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
                            "params": {"uri": uris[0], "version": 7, "diagnostics": expected_diagnostics}})
                        ],
                        "{mode}"
                    );
                    let diagnostics = notifications[1]["params"]["diagnostics"]
                        .as_array()
                        .unwrap();
                    assert_eq!(diagnostics.len(), original_records.len());
                    for (diagnostic, record) in diagnostics.iter().zip(&original_records) {
                        assert_eq!(diagnostic["code"], record.code().to_string());
                        assert_eq!(diagnostic["message"], record.message());
                        assert_eq!(
                            record.severity(),
                            mizar_diagnostics::registry::DiagnosticSeverity::Error
                        );
                    }
                }
            }
            fs::remove_dir_all(root).unwrap();
        }
    }

    fn native_wire_fixture(
        server: &mut Server,
        ids: &mizar_session::InMemorySessionIdAllocator,
        snapshots: &mizar_session::SnapshotRegistry,
    ) -> (
        mizar_driver::request::BuildSession,
        impl Fn() -> mizar_driver::driver::DriverSubmitInput<
            mizar_build::module_index::StaticSourceLayout,
        > + use<>,
        impl Fn() -> (
            mizar_driver::driver::CompilerDriver,
            std::sync::Arc<mizar_ir::publisher::PhaseOutputPublisher>,
        ) + use<>,
    ) {
        use mizar_build::{
            module_index::{StaticSourceLayout, build_module_index},
            planner::{
                DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile,
                parse_package_manifest, produce_build_plan,
            },
            task_graph::ModuleDependencyOverlay,
        };
        use mizar_driver::{
            driver::{CompilerDriver, DriverSubmitInput},
            registry::PhaseRegistryBuilder,
            request::{
                BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
                BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet,
                SourceInputSet, VerifierConfigInput,
            },
        };
        use mizar_ir::{
            identity::{OutputKind, PipelinePhase, SnapshotHandleRegistry, WorkUnit},
            publisher::{AllowedWorkUnit, PhaseOutputPublisher},
            storage::IrStorageService,
        };
        use mizar_session::{
            DiskSourceLoader, Hash, PackageId, SourceOriginInput, ToolchainInfo, WorkspaceRoot,
        };
        use std::{fs, sync::Arc, time::SystemTime};

        let root = std::env::temp_dir().join(format!(
            "mizar-lsp-wire-🦀-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("alpha/src")).unwrap();
        for name in ["main", "clean"] {
            fs::write(root.join(format!("alpha/src/{name}.miz")), [0xff]).unwrap();
        }
        let workspace = WorkspaceRoot::new(root.to_str().unwrap());
        let plan_request = PlanRequest {
            workspace_root: workspace.clone(),
            dependency_selection: DependencySelection::Normal,
            toolchain: ToolchainInfo::new("mizar-evo-test"),
        };
        let packages = vec![WorkspacePackage {
            member_path: "alpha".into(),
            manifest: parse_package_manifest("[package]\nname = \"alpha\"\nversion = \"0.1.0\"\n")
                .unwrap(),
        }];
        let lock = parse_lockfile("schema_version = 1\n[[package]]\nname = \"alpha\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \"alpha\" }\ndependencies = []\n").unwrap();
        let plan =
            produce_build_plan(plan_request.clone(), packages.clone(), lock.clone()).unwrap();
        let layout = StaticSourceLayout::from_native_disk(&plan).unwrap();
        let index = build_module_index(&plan, &layout, &[]).unwrap();
        let loader = DiskSourceLoader::new(root.join("alpha"));
        initialize(server);
        let mut descriptors = Vec::new();
        for (name, raw) in [
            ("main", "\u{feff}::=🦀=::end\r\n:: suffix\r"),
            ("clean", "\u{feff}definition\r\nend;\r\n:: 🦀\r\n"),
        ] {
            let uri = format!(
                "file://{}{}",
                if cfg!(windows) { "/" } else { "" },
                root.join(format!("alpha/src/{name}.miz"))
                    .to_str()
                    .unwrap()
                    .replace('\\', "/")
            );
            server.handle_message(notification(
                "textDocument/didOpen",
                json!({"textDocument": {
                "uri": uri, "languageId": "mizar", "version": 7, "text": raw}}),
            ));
            let descriptor = index
                .native_open_buffer_input(
                    &workspace,
                    &PackageId::new("alpha"),
                    &loader,
                    SourceOriginInput::OpenBuffer {
                        uri,
                        expected_version: 7,
                        actual_version: 7,
                        text: raw.into(),
                    },
                )
                .unwrap();
            descriptors.push((loader.clone(), descriptor));
        }
        let session = BuildRequestDraft {
            lane: BuildLaneId::new(1),
            origin: BuildRequestOrigin::Batch(BatchRequest {
                invocation: BatchInvocation::default(),
            }),
            generation: BuildRequestGeneration::new(0),
            workspace_root: workspace,
            profile: BuildProfile::new("check"),
            targets: BuildTargets::default(),
            source_inputs: SourceInputSet { versions: vec![] },
            dependency_inputs: DependencyInputSet::new(
                vec![],
                Hash::from_bytes([1; 32]),
                ToolchainInfo::new("mizar-evo-test"),
            ),
            verifier_config: VerifierConfigInput::new(Hash::from_bytes([2; 32])),
        }
        .capture_sources(ids, snapshots, descriptors)
        .unwrap();
        for name in ["main", "clean"] {
            fs::remove_file(root.join(format!("alpha/src/{name}.miz"))).unwrap();
        }
        let snapshot = session.captured().snapshot().id;
        let drivers = move || {
            let owner = Arc::new(PhaseOutputPublisher::new(
                Arc::new(IrStorageService::new()),
                Arc::new(SnapshotHandleRegistry::new()),
            ));
            owner.register_current_snapshot(snapshot);
            for name in ["main", "clean"] {
                for (phase, kind) in [("SourceLoad", "SourceUnit"), ("Frontend", "FrontendOutput")]
                {
                    owner.allow_work_unit(AllowedWorkUnit::new(
                        PipelinePhase::new(phase),
                        OutputKind::new(kind),
                        WorkUnit::new(format!("{:?}:{:?}", "alpha", name)),
                    ));
                }
            }
            let mut builder = PhaseRegistryBuilder::new();
            builder.register_source_load();
            builder.register_frontend(vec![]);
            (
                CompilerDriver::new(builder.build().unwrap()).with_output_publisher(owner.clone()),
                owner,
            )
        };
        let input = move || {
            let mut input = DriverSubmitInput::new(
                plan_request.clone(),
                packages.clone(),
                lock.clone(),
                layout.clone(),
            );
            input.dependency_overlay = ModuleDependencyOverlay::complete(vec![]);
            input
        };
        (session, input, drivers)
    }

    #[derive(Default)]
    struct NativeWireWriter {
        bytes: Vec<u8>,
        calls: Vec<&'static str>,
        body: bool,
        fail: Option<&'static str>,
        partial: bool,
        obsolete_on_flush: Option<(
            std::sync::Arc<mizar_ir::publisher::PhaseOutputPublisher>,
            mizar_session::BuildSnapshotId,
        )>,
    }

    impl Write for NativeWireWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let operation = if self.body { "body" } else { "header" };
            self.calls.push(operation);
            if self.fail == Some(operation) && self.partial {
                return Err(io::Error::other(operation));
            }
            let count = if self.fail == Some(operation) {
                self.partial = true;
                bytes.len().min(2)
            } else {
                bytes.len()
            };
            self.bytes.extend_from_slice(&bytes[..count]);
            self.body |= self.bytes.ends_with(b"\r\n\r\n");
            Ok(count)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.calls.push("flush");
            if self.fail == Some("flush") {
                return Err(io::Error::other("flush"));
            }
            if let Some((owner, snapshot)) = self.obsolete_on_flush.take() {
                owner.mark_obsolete(snapshot).unwrap();
            }
            self.body = false;
            Ok(())
        }
    }

    #[test]
    fn native_wire_frames_preserve_original_records_raw_ranges_and_submission() {
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = mizar_session::SnapshotRegistry::new();
        let (session, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (mut ordinary_driver, _) = drivers();
        let ordinary = ordinary_driver
            .submit_captured(session.clone(), &ids, &snapshots, input())
            .unwrap();
        let (mut indexed_driver, _) = drivers();
        let (_, index) = indexed_driver
            .submit_captured_with_native_diagnostic_index(
                session.clone(),
                &ids,
                &snapshots,
                input(),
            )
            .unwrap();
        let index = index.unwrap();
        let (mut driver, _) = drivers();
        let mut writer = NativeWireWriter::default();
        let (submission, delivery) = server
            .submit_native_frontend_diagnostics_to_writer(
                &mut driver,
                session,
                &ids,
                &snapshots,
                input(),
                &mut writer,
            )
            .unwrap();
        assert_eq!(submission, ordinary);
        assert_eq!(delivery.unwrap(), 2);
        assert_eq!(writer.calls.iter().filter(|&&op| op == "flush").count(), 2);
        let mut remaining = writer.bytes.as_slice();
        let mut messages = Vec::new();
        while !remaining.is_empty() {
            let header_end = remaining
                .windows(4)
                .position(|part| part == b"\r\n\r\n")
                .unwrap();
            let header = std::str::from_utf8(&remaining[..header_end]).unwrap();
            let length: usize = header
                .strip_prefix("Content-Length: ")
                .unwrap()
                .parse()
                .unwrap();
            remaining = &remaining[header_end + 4..];
            let body = &remaining[..length];
            assert!(body.len() > std::str::from_utf8(body).unwrap().chars().count());
            messages.push(serde_json::from_slice::<Value>(body).unwrap());
            remaining = &remaining[length..];
        }
        let uris = server.documents.keys().collect::<Vec<_>>();
        assert_eq!(
            messages[0],
            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
            "params": {"uri": uris[0], "version": 7, "diagnostics": []}})
        );
        let diagnostics = vec![
            json!({"range": {"start": {"line": 0, "character": 9}, "end": {"line": 0, "character": 12}},
                "code": "E0033", "message": "unmatched `end` in lexical scope skeleton", "severity": 1}),
            json!({"range": {"start": {"line": 0, "character": 9}, "end": {"line": 0, "character": 12}},
                "code": "E0052", "message": "`end` has no matching block opener", "severity": 1}),
        ];
        assert_eq!(
            messages[1],
            json!({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics",
            "params": {"uri": uris[1], "version": 7, "diagnostics": diagnostics}})
        );
        assert_eq!(diagnostics.len(), index.records().len());
        for (diagnostic, record) in diagnostics.iter().zip(index.records()) {
            assert_eq!(diagnostic["code"], record.code().to_string());
            assert_eq!(diagnostic["message"], record.message());
            assert_eq!(
                record.severity(),
                mizar_diagnostics::registry::DiagnosticSeverity::Error
            );
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn native_wire_withholds_ineligible_handoffs_and_preserves_driver_errors() {
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = mizar_session::SnapshotRegistry::new();
        let (session, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (mut ordinary_driver, _) = drivers();
        let ordinary = ordinary_driver
            .submit_captured(session.clone(), &ids, &snapshots, input())
            .unwrap();
        let uri = server.documents.keys().next().unwrap().clone();
        let original = server.document(&uri).unwrap().1.to_owned();
        for mode in ["control", "version", "raw", "invalid-registry"] {
            let mut current = Server::default();
            initialize(&mut current);
            for (uri, (version, text)) in &server.documents {
                current.handle_message(notification(
                    "textDocument/didOpen",
                    json!({"textDocument": {
                    "uri": uri, "languageId": "mizar", "version": version, "text": text}}),
                ));
            }
            if mode == "version" {
                current.handle_message(notification("textDocument/didChange", json!({
                    "textDocument": {"uri": uri, "version": 8}, "contentChanges": [{"text": original}]})));
                assert_eq!(current.document(&uri), Some((8, original.as_str())));
            } else if mode == "raw" {
                let normalized = original
                    .trim_start_matches('\u{feff}')
                    .replace("\r\n", "\n");
                current.handle_message(notification(
                    "textDocument/didClose",
                    json!({"textDocument": {"uri": uri}}),
                ));
                current.handle_message(notification(
                    "textDocument/didOpen",
                    json!({"textDocument": {
                    "uri": uri, "languageId": "mizar", "version": 7, "text": normalized}}),
                ));
                let loaded = session.captured().native_sources().unwrap().iter()
                    .find(|(_, _, input)| matches!(&input.origin,
                        mizar_session::SourceOriginInput::OpenBuffer { uri: captured, .. } if captured == &uri))
                    .unwrap();
                assert_eq!(loaded.1.text.as_ref(), normalized);
                assert_ne!(original, normalized);
            }
            let invalid = mizar_session::SnapshotRegistry::new();
            let registry = if mode == "invalid-registry" {
                &invalid
            } else {
                &snapshots
            };
            let (mut driver, _) = drivers();
            let mut writer = NativeWireWriter::default();
            let result = current.submit_native_frontend_diagnostics_to_writer(
                &mut driver,
                session.clone(),
                &ids,
                registry,
                input(),
                &mut writer,
            );
            if mode == "invalid-registry" {
                let (mut ordinary_driver, _) = drivers();
                let expected = ordinary_driver
                    .submit_captured(session.clone(), &ids, registry, input())
                    .unwrap_err();
                assert!(matches!(
                    expected,
                    mizar_driver::driver::DriverSubmitError::InvalidCapturedSession { .. }
                ));
                assert_eq!(result.unwrap_err(), expected);
            } else {
                let (submission, delivery) = result.unwrap();
                assert_eq!(submission, ordinary);
                assert_eq!(delivery.unwrap(), if mode == "control" { 2 } else { 0 });
                assert_eq!(
                    driver.validate_current_native_diagnostics(session.id(), registry),
                    Ok(())
                );
            }
            if mode != "control" {
                assert!(writer.bytes.is_empty());
                assert!(writer.calls.is_empty());
            }
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }

    #[test]
    fn native_wire_stops_on_io_errors_and_rechecks_real_publisher_after_each_frame() {
        let mut server = Server::default();
        let ids = mizar_session::InMemorySessionIdAllocator::new();
        let snapshots = mizar_session::SnapshotRegistry::new();
        let (session, input, drivers) = native_wire_fixture(&mut server, &ids, &snapshots);
        let (mut ordinary_driver, _) = drivers();
        let ordinary = ordinary_driver
            .submit_captured(session.clone(), &ids, &snapshots, input())
            .unwrap();
        let mut complete_prefix = None;
        for mode in ["header", "body", "flush", "obsolete"] {
            let (mut driver, owner) = drivers();
            let mut writer = NativeWireWriter {
                fail: (mode != "obsolete").then_some(mode),
                obsolete_on_flush: (mode == "obsolete")
                    .then_some((owner, session.captured().snapshot().id)),
                ..NativeWireWriter::default()
            };
            let (submission, delivery) = server
                .submit_native_frontend_diagnostics_to_writer(
                    &mut driver,
                    session.clone(),
                    &ids,
                    &snapshots,
                    input(),
                    &mut writer,
                )
                .unwrap();
            assert_eq!(submission, ordinary);
            if mode == "obsolete" {
                assert_eq!(delivery.unwrap(), 1);
                assert!(
                    driver
                        .validate_current_native_diagnostics(session.id(), &snapshots)
                        .is_err()
                );
                assert_eq!(writer.calls.iter().filter(|&&op| op == "flush").count(), 1);
                assert_eq!(&writer.bytes, complete_prefix.as_ref().unwrap());
            } else {
                let error = delivery.unwrap_err();
                assert_eq!(error.kind(), io::ErrorKind::Other);
                assert_eq!(error.to_string(), mode);
                assert_eq!(writer.calls.last(), Some(&mode));
                if mode != "flush" {
                    assert_eq!(writer.calls.iter().filter(|&&op| op == mode).count(), 2);
                }
                assert_eq!(
                    writer.calls.iter().filter(|&&op| op == "flush").count(),
                    usize::from(mode == "flush")
                );
                if mode == "header" {
                    assert_eq!(writer.bytes, b"Co");
                } else {
                    let header_end = writer
                        .bytes
                        .windows(4)
                        .position(|part| part == b"\r\n\r\n")
                        .unwrap();
                    let length: usize = std::str::from_utf8(&writer.bytes[..header_end])
                        .unwrap()
                        .strip_prefix("Content-Length: ")
                        .unwrap()
                        .parse()
                        .unwrap();
                    assert_eq!(
                        writer.bytes.len() - header_end - 4,
                        if mode == "body" { 2 } else { length }
                    );
                    if mode == "flush" {
                        let message: Value =
                            serde_json::from_slice(&writer.bytes[header_end + 4..]).unwrap();
                        assert_eq!(message["params"]["diagnostics"], json!([]));
                        complete_prefix = Some(writer.bytes);
                    }
                }
            }
        }
        std::fs::remove_dir_all(input().plan_request.workspace_root.as_str()).unwrap();
    }
}
