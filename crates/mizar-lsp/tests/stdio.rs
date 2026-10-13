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

#[test]
fn interactive_stdio_preserves_lifecycle_and_silent_unicode_document_notifications() {
    let mut process = Process::new(&["lsp".into()], false);
    let uri = format!("file://{}", process.root.join("src/Unicode.miz").display());
    let text = fs::read_to_string(process.root.join("src/Unicode.miz")).unwrap();
    process.send(&frame(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":format!("file://{}", process.root.display()),"capabilities":{}}})));
    assert_eq!(
        process.response(),
        json!({"jsonrpc":"2.0","id":1,"result":{"capabilities":{"positionEncoding":"utf-16","textDocumentSync":{"openClose":true,"change":1}}}})
    );
    for message in [
        json!({"jsonrpc":"2.0","method":"initialized","params":{}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"mizar","version":0,"text":text}}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":uri,"version":1},"contentChanges":[{"text":"\u{feff}:: 雪😀 edited\r\ndefinition\r\nend;\r\n"}]}}),
        json!({"jsonrpc":"2.0","method":"$/cancelRequest","params":{"id":99}}),
        json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{"textDocument":{"uri":uri}}}),
        json!({"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}),
    ] {
        process.send(&frame(message));
    }
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
        json!({"jsonrpc":"2.0","id":1,"result":{"capabilities":{"positionEncoding":"utf-16","textDocumentSync":{"openClose":true,"change":1}}}})
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
