use std::{
    ffi::OsString,
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Child, ChildStdin, Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use serde_json::{Value, json};

static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(0);
const TIMEOUT: Duration = Duration::from_secs(5);

struct Process {
    root: PathBuf,
    child: Child,
    input: Option<ChildStdin>,
    responses: Receiver<io::Result<Value>>,
    reader: Option<JoinHandle<()>>,
    errors: Option<JoinHandle<Vec<u8>>>,
}

impl Process {
    fn new(args: &[OsString], broken_io: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "mizar-lsp-stdio-{}-{}",
            std::process::id(),
            NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("mizar.toml"),
            "[package]\nname = \"editor\"\nversion = \"0.1.0\"\nedition = \"2025\"\n",
        )
        .unwrap();
        fs::write(
            root.join("src/Unicode.miz"),
            "\u{feff}:: 雪😀\r\ndefinition\r\nend;\r\n",
        )
        .unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_mizar"));
        command.args(args).current_dir(&root).stdin(Stdio::piped());
        if broken_io {
            command.stdout(
                fs::OpenOptions::new()
                    .write(true)
                    .open("/dev/full")
                    .unwrap(),
            );
            command.stderr(
                fs::OpenOptions::new()
                    .write(true)
                    .open("/dev/full")
                    .unwrap(),
            );
        } else {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        }
        let mut child = command.spawn().unwrap();
        let input = child.stdin.take();
        let output = child.stdout.take();
        let error = child.stderr.take();
        let (sender, responses) = mpsc::channel();
        let reader = thread::spawn(move || {
            if let Some(output) = output {
                let mut output = BufReader::new(output);
                loop {
                    let mut header = String::new();
                    let response = (|| {
                        if output.read_line(&mut header)? == 0 {
                            return Ok(None);
                        }
                        let length = header
                            .strip_prefix("Content-Length: ")
                            .and_then(|value| value.strip_suffix("\r\n"))
                            .and_then(|value| value.parse::<usize>().ok())
                            .ok_or_else(|| io::Error::other("non-protocol stdout"))?;
                        let mut separator = String::new();
                        output.read_line(&mut separator)?;
                        if separator != "\r\n" {
                            return Err(io::Error::other("invalid output framing"));
                        }
                        let mut body = vec![0; length];
                        output.read_exact(&mut body)?;
                        serde_json::from_slice::<Value>(&body)
                            .map(Some)
                            .map_err(io::Error::other)
                    })();
                    match response {
                        Ok(None) => break,
                        Ok(Some(value)) => {
                            if sender.send(Ok(value)).is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            let _ = sender.send(Err(error));
                            break;
                        }
                    }
                }
            }
        });
        let errors = thread::spawn(move || {
            let mut bytes = Vec::new();
            if let Some(mut error) = error {
                error.read_to_end(&mut bytes).unwrap();
            }
            bytes
        });
        Self {
            root,
            child,
            input,
            responses,
            reader: Some(reader),
            errors: Some(errors),
        }
    }

    fn send(&mut self, bytes: &[u8]) {
        self.input.as_mut().unwrap().write_all(bytes).unwrap();
        self.input.as_mut().unwrap().flush().unwrap();
    }

    fn response(&self) -> Value {
        self.responses
            .recv_timeout(TIMEOUT)
            .expect("reply while stdin remains open")
            .unwrap()
    }

    #[cfg(target_os = "linux")]
    fn native() -> Self {
        let process = Self::new(&["lsp".into()], false);
        fs::write(
            process.root.join("mizar.pkg"),
            "[package]\nname = \"editor\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        fs::write(process.root.join("mizar.lock"), "schema_version = 1\n[[package]]\nname = \"editor\"\nversion = \"0.1.0\"\nsource = { kind = \"workspace\", path = \".\" }\ndependencies = []\n").unwrap();
        process
    }

    #[cfg(target_os = "linux")]
    fn initialize(&mut self, root: Value) {
        self.send(&frame(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":root,"capabilities":{}}})));
        assert_eq!(self.response()["id"], 1);
    }

    #[cfg(target_os = "linux")]
    fn assert_silent(&self) {
        let response = self.responses.recv_timeout(Duration::from_millis(800));
        assert!(
            matches!(response, Err(mpsc::RecvTimeoutError::Timeout)),
            "unexpected response: {response:?}"
        );
    }

    #[cfg(target_os = "linux")]
    fn shutdown(mut self) {
        self.send(&frame(
            json!({"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}),
        ));
        assert_eq!(
            self.response(),
            json!({"jsonrpc":"2.0","id":99,"result":null})
        );
        self.send(&frame(json!({"jsonrpc":"2.0","method":"exit"})));
        let (status, responses, errors) = self.finish();
        assert_eq!(status.code(), Some(0));
        assert!(responses.is_empty());
        assert!(errors.is_empty());
    }

    fn finish(mut self) -> (ExitStatus, Vec<Value>, Vec<u8>) {
        self.input.take();
        let deadline = Instant::now() + TIMEOUT;
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(Instant::now() < deadline, "process did not exit");
            thread::sleep(Duration::from_millis(10));
        };
        self.reader.take().unwrap().join().unwrap();
        let responses = self.responses.try_iter().map(Result::unwrap).collect();
        let errors = self.errors.take().unwrap().join().unwrap();
        (status, responses, errors)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        if let Some(errors) = self.errors.take() {
            let _ = errors.join();
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn frame(message: Value) -> Vec<u8> {
    let body = serde_json::to_vec(&message).unwrap();
    let mut bytes = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    bytes.extend(body);
    bytes
}

#[cfg(target_os = "linux")]
#[test]
fn native_stdio_publishes_editor_frontend_errors_and_clean_clearing_while_idle() {
    let mut process = Process::native();
    let uri = format!("file://{}", process.root.join("src/Unicode.miz").display());
    let clean = fs::read_to_string(process.root.join("src/Unicode.miz")).unwrap();
    process.initialize(json!(format!("file://{}", process.root.display())));
    process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":7,"text":"\u{feff}:: 雪😀\r\n::=🦀=::end\r\n"}}})));
    // The block comment before `end` occupies eight UTF-16 units, ten bytes.
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":7,"diagnostics":[
            {"range":{"start":{"line":1,"character":8},"end":{"line":1,"character":11}},"code":"E0033","message":"unmatched `end` in lexical scope skeleton","severity":1},
            {"range":{"start":{"line":1,"character":8},"end":{"line":1,"character":11}},"code":"E0052","message":"`end` has no matching block opener","severity":1}
        ]}})
    );
    assert!(process.child.try_wait().unwrap().is_none());
    process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":8},"contentChanges":[{"text":clean}]}})));
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":8,"diagnostics":[]}})
    );
    process.shutdown();
}

#[cfg(target_os = "linux")]
#[test]
fn native_stdio_publishes_each_open_member_with_transitive_normal_dependencies() {
    let mut process = Process::native();
    fs::write(
        process.root.join("mizar.workspace"),
        "[workspace]\nmembers = [\"alpha\", \"beta\", \"gamma\", \"devonly\"]\n",
    )
    .unwrap();
    let mut lock = String::from("schema_version = 1\n");
    for (name, dependency) in [
        ("alpha", Some("beta")),
        ("beta", Some("gamma")),
        ("gamma", None),
        ("devonly", None),
    ] {
        let member = process.root.join(name);
        fs::create_dir_all(member.join("src")).unwrap();
        fs::write(member.join("src/main.miz"), "definition\nend;\n").unwrap();
        let mut manifest = format!("[package]\nname = {name:?}\nversion = \"0.1.0\"\n");
        let mut dependencies = Vec::new();
        if let Some(dependency) = dependency {
            manifest.push_str(&format!("[dependencies]\n{dependency} = \"0.1.0\"\n"));
            dependencies.push(format!("{{ name = {dependency:?}, version = \"0.1.0\" }}"));
        }
        if name == "alpha" {
            manifest.push_str("[dev-dependencies]\ndevonly = \"0.1.0\"\n");
            dependencies.push("{ name = \"devonly\", version = \"0.1.0\" }".into());
        }
        fs::write(member.join("mizar.pkg"), manifest).unwrap();
        lock.push_str(&format!("[[package]]\nname = {name:?}\nversion = \"0.1.0\"\nsource = {{ kind = \"workspace\", path = {name:?} }}\ndependencies = [{}]\n", dependencies.join(", ")));
    }
    // Preserve unused registry lock metadata without making it a manifest dependency.
    lock.push_str("[[package]]\nname = \"unused\"\nversion = \"1.0.0\"\nsource = { kind = \"registry\", registry = \"default\", checksum = \"sha256:unused\" }\ndependencies = []\n");
    fs::write(process.root.join("mizar.lock"), lock).unwrap();
    process.initialize(json!(format!("file://{}", process.root.display())));
    let alpha = format!(
        "file://{}",
        process.root.join("alpha/src/main.miz").display()
    );
    let gamma = format!(
        "file://{}",
        process.root.join("gamma/src/main.miz").display()
    );
    fs::write(process.root.join("gamma/src/main.miz"), "end;\n").unwrap();
    for (uri, version, text) in [(&alpha, 3, "end;\n"), (&gamma, 9, "definition\nend;\n")] {
        process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":version,"text":text}}})));
    }
    // Uncaptured closed members cannot authorize clean clearing for this all-Open profile.
    process.assert_silent();
    let beta = format!(
        "file://{}",
        process.root.join("beta/src/main.miz").display()
    );
    let devonly = format!(
        "file://{}",
        process.root.join("devonly/src/main.miz").display()
    );
    for (uri, version) in [(&beta, 4), (&devonly, 5)] {
        process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":version,"text":"definition\nend;\n"}}})));
    }
    for (uri, version, codes) in [
        (&alpha, 3, vec!["E0033", "E0052"]),
        (&beta, 4, vec![]),
        (&devonly, 5, vec![]),
        (&gamma, 9, vec![]),
    ] {
        let diagnostics = process.response();
        assert_eq!(diagnostics["method"], "textDocument/publishDiagnostics");
        assert_eq!(diagnostics["params"]["uri"], *uri);
        assert_eq!(diagnostics["params"]["version"], version);
        assert_eq!(
            diagnostics["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(|diagnostic| diagnostic["code"].as_str().unwrap())
                .collect::<Vec<_>>(),
            codes
        );
    }
    process.shutdown();
}

#[cfg(target_os = "linux")]
#[test]
fn native_stdio_coalesces_edits_and_reacquires_metadata_only_on_accepted_events() {
    let mut process = Process::native();
    let uri = format!("file://{}", process.root.join("src/Unicode.miz").display());
    process.initialize(json!(format!("file://{}", process.root.display())));
    process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":1,"text":"end;\n"}}})));
    for version in 2..=8 {
        process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":"end;\n"}]}})));
    }
    let latest = process.response();
    assert_eq!(latest["method"], "textDocument/publishDiagnostics");
    assert_eq!(latest["params"]["version"], 8);
    assert_eq!(latest["params"]["diagnostics"].as_array().unwrap().len(), 2);
    for message in [
        json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":7},"contentChanges":[{"text":"definition\nend;\n"}]}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":8},"contentChanges":[{"text":"definition\nend;\n"}]}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":9},"contentChanges":[{"text":12}]}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didSave","params":{"textDocument":{"uri":uri}}}),
        json!({"jsonrpc":"2.0","method":"workspace/didChangeConfiguration","params":{"settings":{"mizar":{"quietPeriod":0}}}}),
    ] {
        process.send(&frame(message));
    }
    process.assert_silent();
    // Reopen at the same version; the new incarnation gets its own clean cohort.
    process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}})));
    process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":8,"text":"definition\nend;\n"}}})));
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":8,"diagnostics":[]}})
    );
    let manifest = fs::read(process.root.join("mizar.pkg")).unwrap();
    fs::write(process.root.join("mizar.pkg"), "[package").unwrap();
    process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":9},"contentChanges":[{"text":"end;\n"}]}})));
    process.assert_silent();
    fs::write(process.root.join("mizar.pkg"), manifest).unwrap();
    for method in ["textDocument/didSave", "workspace/didChangeConfiguration"] {
        process.send(&frame(json!({"jsonrpc":"2.0","method":method,"params":{"textDocument":{"uri":uri},"settings":{}}})));
    }
    process.assert_silent();
    process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":10},"contentChanges":[{"text":"definition\nend;\n"}]}})));
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":10,"diagnostics":[]}})
    );
    // Queued ignored traffic still permits actual native task progress, with no wall-clock timing oracle.
    let mut input = frame(
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":format!("file://{}", process.root.display()),"capabilities":{}}}),
    );
    input.extend(frame(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":11,"text":"definition\nend;\n"}}})));
    for _ in 0..128 {
        input.extend(frame(json!({"jsonrpc":"2.0","method":"workspace/didChangeConfiguration","params":{"settings":{}}})));
    }
    input.extend(frame(
        json!({"jsonrpc":"2.0","id":99,"method":"shutdown","params":null}),
    ));
    input.extend(frame(json!({"jsonrpc":"2.0","method":"exit"})));
    let tick = AtomicU64::new(0);
    let start = Instant::now();
    let mut server = mizar_lsp::Server::default()
        .with_nonterminal_build_bridge(mizar_driver::request::BuildLaneId::new(1), move || {
            start + Duration::from_millis(tick.fetch_add(301, Ordering::Relaxed))
        });
    let mut output = Vec::new();
    assert_eq!(
        server.serve(io::Cursor::new(input), &mut output).unwrap(),
        0
    );
    let expected = frame(
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":11,"diagnostics":[]}}),
    );
    assert!(
        output
            .windows(expected.len())
            .any(|bytes| bytes == expected)
    );
    process.shutdown();
}

#[cfg(target_os = "linux")]
#[test]
fn native_stdio_acquisition_failures_stay_responsive_and_viable_edits_recover() {
    for failure in [
        "null_root",
        "invalid_root",
        "non_file_root",
        "missing_manifest",
        "malformed_manifest",
        "missing_lock",
        "malformed_lock",
        "missing_source",
        "registry",
        "unmappable_open",
    ] {
        let mut process = Process::native();
        let manifest = fs::read(process.root.join("mizar.pkg")).unwrap();
        let lock = fs::read_to_string(process.root.join("mizar.lock")).unwrap();
        let source = fs::read(process.root.join("src/Unicode.miz")).unwrap();
        let uri = format!("file://{}", process.root.join("src/Unicode.miz").display());
        let root = match failure {
            "null_root" => Value::Null,
            "invalid_root" => json!("file:///%GG"),
            "non_file_root" => json!("untitled:workspace"),
            _ => json!(format!("file://{}", process.root.display())),
        };
        match failure {
            "missing_manifest" => fs::remove_file(process.root.join("mizar.pkg")).unwrap(),
            "malformed_manifest" => fs::write(process.root.join("mizar.pkg"), "[package").unwrap(),
            "missing_lock" => fs::remove_file(process.root.join("mizar.lock")).unwrap(),
            "malformed_lock" => {
                fs::write(process.root.join("mizar.lock"), "schema_version = [").unwrap()
            }
            "missing_source" => fs::remove_file(process.root.join("src/Unicode.miz")).unwrap(),
            "registry" => {
                fs::write(process.root.join("mizar.pkg"), "[package]\nname = \"editor\"\nversion = \"0.1.0\"\n[dependencies]\ndep = \"1.0.0\"\n").unwrap();
                fs::write(process.root.join("mizar.lock"), format!("{}[[package]]\nname = \"dep\"\nversion = \"1.0.0\"\nsource = {{ kind = \"registry\", registry = \"default\", checksum = \"sha256:dep\" }}\ndependencies = []\n", lock.replace("dependencies = []", "dependencies = [{ name = \"dep\", version = \"1.0.0\" }]"))).unwrap();
            }
            _ => {}
        }
        process.initialize(root);
        process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":1,"text":"end;\n"}}})));
        if failure == "unmappable_open" {
            process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"untitled:other","languageId":"mizar","version":1,"text":"end;\n"}}})));
        }
        process.send(&frame(json!({"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":0}}})));
        assert_eq!(
            process.response(),
            json!({"jsonrpc":"2.0","id":2,"result":null}),
            "{failure}"
        );
        process.assert_silent();
        if !matches!(failure, "null_root" | "invalid_root" | "non_file_root") {
            fs::write(process.root.join("mizar.pkg"), manifest).unwrap();
            fs::write(process.root.join("mizar.lock"), lock).unwrap();
            fs::write(process.root.join("src/Unicode.miz"), source).unwrap();
            if failure == "unmappable_open" {
                process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":"untitled:other"}}})));
            }
            process.send(&frame(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":"definition\nend;\n"}]}})));
            assert_eq!(
                process.response(),
                json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":uri,"version":2,"diagnostics":[]}}),
                "{failure}"
            );
        }
        process.shutdown();
    }
}

#[cfg(target_os = "linux")]
#[test]
fn native_stdio_terminal_traffic_discards_work_and_exits_with_partial_header_open() {
    // Actual running-task cancellation remains covered by checkpoint-controlled bridge tests.
    for shutdown in [false, true] {
        for published in [false, true] {
            let mut process = Process::native();
            let uri = format!("file://{}", process.root.join("src/Unicode.miz").display());
            process.initialize(json!(format!("file://{}", process.root.display())));
            let open = json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":1,"text":"end;\n"}}});
            let mut bytes = if published {
                process.send(&frame(open));
                assert_eq!(process.response()["params"]["version"], 1);
                frame(
                    json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":2},"contentChanges":[{"text":"end;\n"}]}}),
                )
            } else {
                frame(open)
            };
            if shutdown {
                bytes.extend(frame(
                    json!({"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}),
                ));
                process.send(&bytes);
                assert_eq!(
                    process.response(),
                    json!({"jsonrpc":"2.0","id":2,"result":null})
                );
                bytes.clear();
            }
            bytes.extend(frame(json!({"jsonrpc":"2.0","method":"exit"})));
            bytes.extend_from_slice(b"Content-Len");
            process.send(&bytes);
            let deadline = Instant::now() + TIMEOUT;
            while process.child.try_wait().unwrap().is_none() {
                assert!(Instant::now() < deadline, "exit required closing stdin");
                thread::sleep(Duration::from_millis(10));
            }
            assert!(process.input.is_some());
            let (status, responses, errors) = process.finish();
            assert_eq!(status.code(), Some(if shutdown { 0 } else { 1 }));
            assert!(responses.is_empty());
            assert!(errors.is_empty());
        }
    }
}

#[test]
fn interactive_stdio_preserves_lifecycle_and_silent_unicode_document_notifications() {
    let mut process = Process::new(&["lsp".into()], false);
    let uri = format!("file://{}", process.root.join("src/Unicode.miz").display());
    let text = fs::read_to_string(process.root.join("src/Unicode.miz")).unwrap();
    process.send(&frame(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":format!("file://{}", process.root.display()),"capabilities":{}}})));
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":1,"result":{"capabilities":{"positionEncoding":"utf-16","experimental":{"mizarNonRangeDiagnostics":{"version":1}},"hoverProvider":true,"definitionProvider":true,"completionProvider":{"triggerCharacters":["."]},"codeActionProvider":{"codeActionKinds":["quickfix"],"resolveProvider":true},"semanticTokensProvider":{"legend":{"tokenTypes":["keyword","operator","number","comment"],"tokenModifiers":[]},"full":true},"textDocumentSync":{"openClose":true,"change":1}}}})
    );
    for message in [
        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":0,"text":text}}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":1},"contentChanges":[{"text":"\u{feff}:: 雪😀 edited\r\ndefinition\r\nend;\r\n"}]}}),
        json!({"jsonrpc":"2.0","id":4,"method":"textDocument/semanticTokens/full","params":{"textDocument":{"uri":uri}}}),
        json!({"jsonrpc":"2.0","id":3,"method":"textDocument/hover","params":{"textDocument":{"uri":uri},"position":{"line":1,"character":0}}}),
        json!({"jsonrpc":"2.0","method":"textDocument/definition","params":{"textDocument":{"uri":uri},"position":{"line":1,"character":0}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"textDocument/definition","params":{"textDocument":{"uri":uri},"position":{"line":1,"character":0}}}),
        json!({"jsonrpc":"2.0","id":6,"method":"textDocument/definition","params":{"textDocument":{"uri":uri},"position":{"line":1,"character":-1}}}),
        json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":99}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}}),
        json!({"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}),
    ] {
        process.send(&frame(message));
    }
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":4,"result":{"data":[
            0,1,13,3,0,
            1,0,10,0,0
        ]}})
    );
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":3,"result":null})
    );
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":5,"result":null})
    );
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":6,"error":{"code":-32602,"message":"Invalid params"}})
    );
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":2,"result":null})
    );
    process.send(&frame(json!({"jsonrpc":"2.0","method":"exit"})));
    let (status, responses, errors) = process.finish();
    assert_eq!(status.code(), Some(0));
    assert!(responses.is_empty());
    assert!(errors.is_empty());
}

#[test]
fn framed_stdio_advertises_completion_and_validates_completion_positions() {
    let mut process = Process::new(&["lsp".into()], false);
    let uri = format!("file://{}", process.root.join("src/Unicode.miz").display());
    process.send(&frame(json!({
        "jsonrpc":"2.0", "id":1, "method":"initialize",
        "params":{"processId":null,"rootUri":format!("file://{}", process.root.display()),"capabilities":{}}
    })));
    let initialized = process.response();
    assert_eq!(initialized["id"], 1);
    assert_eq!(
        initialized.pointer("/result/capabilities/completionProvider/triggerCharacters"),
        Some(&json!(["."]))
    );

    for message in [
        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":1,"text":"be "}}}),
        json!({"jsonrpc":"2.0","id":2,"method":"textDocument/completion","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":3}}}),
        json!({"jsonrpc":"2.0","id":3,"method":"textDocument/completion","params":{"textDocument":{"uri":uri},"position":{"line":-1,"character":3}}}),
        json!({"jsonrpc":"2.0","id":6,"method":"textDocument/completion","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":"3"}}}),
        json!({"jsonrpc":"2.0","id":7,"method":"textDocument/completion","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":3},"context":{"triggerKind":4}}}),
        json!({"jsonrpc":"2.0","id":8,"method":"textDocument/completion","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":4}}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"textDocument/completion","params":{"textDocument":{"uri":uri},"position":{"line":0,"character":3}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"shutdown","params":null}),
    ] {
        process.send(&frame(message));
    }
    let items = process.response()["result"].clone();
    assert!(items.is_array());
    let labels = items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["label"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(labels, ["object", "set"]);
    let invalid = process.response();
    assert_eq!(invalid["id"], 3);
    assert_eq!(invalid["error"]["code"], -32602);
    for expected_id in [6, 7] {
        let invalid = process.response();
        assert_eq!(invalid["id"], expected_id);
        assert_eq!(invalid["error"]["code"], -32602);
    }
    let outside = process.response();
    assert_eq!(outside["id"], 8);
    assert_eq!(outside["result"], json!([]));
    let closed = process.response();
    assert_eq!(closed["id"], 4);
    assert_eq!(closed["result"], json!([]));
    assert_eq!(process.response()["id"], 5);
    process.send(&frame(json!({"jsonrpc":"2.0","method":"exit"})));
    let (status, responses, errors) = process.finish();
    assert_eq!(status.code(), Some(0));
    assert!(responses.is_empty());
    assert!(errors.is_empty());
}

#[test]
fn stdio_code_action_without_publication_returns_an_empty_list() {
    let mut process = Process::new(&["lsp".into()], false);
    let uri = format!("file://{}", process.root.join("src/Unicode.miz").display());
    let text = fs::read_to_string(process.root.join("src/Unicode.miz")).unwrap();
    process.send(&frame(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":format!("file://{}", process.root.display()),"capabilities":{"textDocument":{"codeAction":{"dataSupport":true,"resolveSupport":{"properties":["edit"]}}}}}})));
    let initialized = process.response();
    assert_eq!(initialized["id"], 1);
    assert_eq!(
        initialized["result"]["capabilities"]["codeActionProvider"]["resolveProvider"],
        true
    );
    for message in [
        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":1,"text":text}}}),
        json!({"jsonrpc":"2.0","id":2,"method":"textDocument/codeAction","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"context":{"diagnostics":[],"only":["quickfix"]}}}),
        json!({"jsonrpc":"2.0","id":3,"method":"textDocument/codeAction","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"context":{"diagnostics":[],"only":["refactor"]}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"textDocument/codeAction","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":100}},"context":{"diagnostics":[]}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"textDocument/codeAction","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":1}},"context":{}}}),
        json!({"jsonrpc":"2.0","id":6,"method":"codeAction/resolve","params":{"title":"unavailable","data":{"publication":"unavailable","snapshot":"s","id":1,"fix":"f","uri":uri,"version":1}}}),
        json!({"jsonrpc":"2.0","id":7,"method":"shutdown","params":null}),
        json!({"jsonrpc":"2.0","id":8,"method":"textDocument/codeAction","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"context":{"diagnostics":[]}}}),
    ] {
        process.send(&frame(message));
    }
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":2,"result":[]})
    );
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":3,"result":[]})
    );
    assert_eq!(process.response()["error"]["code"], -32602);
    assert_eq!(process.response()["error"]["code"], -32602);
    assert_eq!(process.response()["error"]["code"], -32801);
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":7,"result":null})
    );
    assert_eq!(process.response()["error"]["code"], -32600);
    process.send(&frame(json!({"jsonrpc":"2.0","method":"exit"})));
    let (status, responses, errors) = process.finish();
    assert_eq!(status.code(), Some(0));
    assert!(responses.is_empty());
    assert!(errors.is_empty());

    let mut unsupported = Process::new(&["lsp".into()], false);
    let uri = format!(
        "file://{}",
        unsupported.root.join("src/Unicode.miz").display()
    );
    let text = fs::read_to_string(unsupported.root.join("src/Unicode.miz")).unwrap();
    unsupported.send(&frame(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":format!("file://{}", unsupported.root.display()),"capabilities":{}}})));
    assert_eq!(unsupported.response()["id"], 1);
    for message in [
        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":1,"text":text}}}),
        json!({"jsonrpc":"2.0","id":2,"method":"textDocument/codeAction","params":{"textDocument":{"uri":uri},"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"context":{"diagnostics":[]}}}),
        json!({"jsonrpc":"2.0","id":3,"method":"shutdown","params":null}),
    ] {
        unsupported.send(&frame(message));
    }
    assert_eq!(
        unsupported.response(),
        json!({"jsonrpc":"2.0","id":2,"result":[]})
    );
    assert_eq!(
        unsupported.response(),
        json!({"jsonrpc":"2.0","id":3,"result":null})
    );
    unsupported.send(&frame(json!({"jsonrpc":"2.0","method":"exit"})));
    let (status, responses, errors) = unsupported.finish();
    assert_eq!(status.code(), Some(0));
    assert!(responses.is_empty());
    assert!(errors.is_empty());
}

#[test]
fn stdio_explanation_query_returns_framed_unavailable_without_host_backing() {
    let mut process = Process::new(&["lsp".into()], false);
    let snapshot = format!("mizar-session-build-snapshot-v1:{}", "00".repeat(32));
    for message in [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}),
        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
        json!({"jsonrpc":"2.0","id":2,"method":"mizar/explain","params":{"snapshot":snapshot,"diagnosticId":1}}),
        json!({"jsonrpc":"2.0","id":3,"method":"shutdown","params":null}),
    ] {
        process.send(&frame(message));
    }
    assert_eq!(process.response()["id"], 1);
    let unavailable = process.response();
    assert_eq!(unavailable["id"], 2);
    assert_eq!(unavailable["result"]["status"], "unavailable");
    assert_eq!(unavailable["result"]["freshness"], "unavailable");
    assert_eq!(unavailable["result"]["reason"], "no_backing_host");
    assert_eq!(process.response()["id"], 3);
    process.send(&frame(json!({"jsonrpc":"2.0","method":"exit"})));
    let (status, responses, errors) = process.finish();
    assert_eq!(status.code(), Some(0));
    assert!(responses.is_empty());
    assert!(errors.is_empty());
}

#[test]
fn early_exit_and_fatal_frames_return_one_without_protocol_noise() {
    for bytes in [
        frame(json!({"jsonrpc":"2.0","method":"exit"})),
        Vec::new(),
        b"Content-Length: 2\r\n".to_vec(),
        b"Content-Length: 20\r\n\r\n{}".to_vec(),
        b"Content-Length: 0\n\n".to_vec(),
        b"Content-Length: 16777217\r\n\r\n".to_vec(),
    ] {
        let early_exit = bytes == frame(json!({"jsonrpc":"2.0","method":"exit"}));
        let mut process = Process::new(&["lsp".into()], false);
        process.send(&bytes);
        let (status, responses, errors) = process.finish();
        assert_eq!(status.code(), Some(1));
        assert!(responses.is_empty());
        assert_eq!(errors.is_empty(), early_exit);
    }
}

#[test]
fn complete_bad_json_responds_and_continues_to_exit() {
    let mut process = Process::new(&["lsp".into()], false);
    process.send(b"Content-Length: 1\r\n\r\n{");
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}})
    );
    process.send(&frame(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}})));
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":1,"result":{"capabilities":{"positionEncoding":"utf-16","experimental":{"mizarNonRangeDiagnostics":{"version":1}},"hoverProvider":true,"definitionProvider":true,"completionProvider":{"triggerCharacters":["."]},"codeActionProvider":{"codeActionKinds":["quickfix"],"resolveProvider":true},"semanticTokensProvider":{"legend":{"tokenTypes":["keyword","operator","number","comment"],"tokenModifiers":[]},"full":true},"textDocumentSync":{"openClose":true,"change":1}}}})
    );
    process.send(&frame(
        json!({"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}),
    ));
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":2,"result":null})
    );
    process.send(&frame(json!({"jsonrpc":"2.0","method":"exit"})));
    let (status, responses, errors) = process.finish();
    assert_eq!(status.code(), Some(0));
    assert!(responses.is_empty());
    assert!(errors.is_empty());
}

#[test]
fn argument_errors_and_real_parsed_build_options_keep_stdout_empty() {
    for (args, code) in [
        (vec![], 2),
        (vec!["unknown"], 2),
        (vec!["lsp", "--jobs", "1"], 2),
        (vec!["lsp", "lsp"], 2),
        (vec!["build", "--jobs", "0"], 2),
        (vec!["build", "--unknown"], 2),
        (vec!["build"], 3),
        (
            vec![
                "build",
                "--workspace",
                ".",
                "--manifest-path",
                "mizar.toml",
                "--package",
                "editor",
                "--profile",
                "release",
                "--target",
                "Unicode",
                "--jobs",
                "4",
                "--locked",
                "--no-incremental",
                "--message-format",
                "json",
                "--quiet",
            ],
            3,
        ),
    ] {
        let process = Process::new(
            &args.into_iter().map(OsString::from).collect::<Vec<_>>(),
            false,
        );
        let (status, responses, errors) = process.finish();
        assert_eq!(status.code(), Some(code));
        assert!(responses.is_empty());
        assert!(!errors.is_empty());
        if code == 3 {
            assert!(
                String::from_utf8(errors)
                    .unwrap()
                    .contains("bootstrap is unavailable")
            );
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let process = Process::new(&[OsString::from_vec(vec![0xff])], false);
        let (status, responses, errors) = process.finish();
        assert_eq!(status.code(), Some(2));
        assert!(responses.is_empty());
        assert!(String::from_utf8(errors).unwrap().contains("non-Unicode"));
    }
}

#[cfg(target_os = "linux")]
#[test]
fn actual_output_failure_and_unwritable_stderr_exit_without_panicking() {
    let mut process = Process::new(&["lsp".into()], true);
    let mut bytes = frame(
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}),
    );
    bytes.extend(frame(
        json!({"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}),
    ));
    bytes.extend(frame(json!({"jsonrpc":"2.0","method":"exit"})));
    process.send(&bytes);
    let (status, _, _) = process.finish();
    assert_eq!(status.code(), Some(1));
}
