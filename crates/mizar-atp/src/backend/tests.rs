use super::*;
use crate::problem::{
    AtpAtom, AtpDeclaration, AtpDeclarationId, AtpDeclarationKind, AtpFingerprint, AtpFormula,
    AtpFormulaId, AtpFormulaTree, AtpProblem, AtpProblemParts, AtpProvenance, AtpProvenanceId,
    AtpSourceBinding, AtpSourceRef, AtpSymbolMapEntry, AtpSymbolSource, AtpTypeContext,
    EqualitySupport, LogicFragment, LogicProfile, NativePropertySupport, QuantifierPolicy,
    SoftTypeStrategy,
};
use mizar_vc::vc_ir::VcId;
use std::{
    collections::BTreeSet,
    os::unix::fs::PermissionsExt,
    sync::{Mutex, MutexGuard},
};

static PRIVATE_TEMP_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn runs_mock_backend_with_byte_exact_stdin_input() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let script = dir.executable(
        "stdin-echo",
        &format!(
            "#!/bin/sh\nif [ -e /proc/self/fd/0 ]; then {} /proc/self/fd/0 >&2; fi\n/bin/cat\n",
            readlink_command_for_test().unwrap_or("/bin/true")
        ),
        true,
    );
    let input_bytes = b"p; echo hacked\n(get-proof)\n(get-unsat-core)\n".to_vec();
    let result = run_backend(run_input(
        script,
        Vec::new(),
        BackendIoMode::Stdin,
        input_bytes.clone(),
        BackendResourceLimits::new(),
    ));

    assert_eq!(result.status(), BackendRunStatus::Unknown);
    assert_eq!(result.stdout().retained_bytes(), input_bytes.as_slice());
    assert!(result.child_reaped());
    assert!(!has_diag(&result, "stdin_spool_file_failed"));
    if readlink_command_for_test().is_some() && Path::new("/proc/self/fd/0").exists() {
        assert_private_problem_path_cleaned(&result);
    } else {
        assert!(result.stderr().retained_bytes().is_empty());
    }
}

#[test]
fn runs_mock_backend_with_private_file_input_and_cleans_it() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let script = dir.executable(
        "file-echo",
        "#!/bin/sh\nprintf '%s\\n' \"$1\" >&2\n/bin/cat \"$1\"\n",
        true,
    );
    let input_bytes = b"fof(ax,axiom,p).\n# not a shell command\n".to_vec();
    let result = run_backend(run_input(
        script,
        Vec::new(),
        BackendIoMode::PrivateProblemFile,
        input_bytes.clone(),
        BackendResourceLimits::new(),
    ));

    assert_eq!(result.status(), BackendRunStatus::Unknown);
    assert_eq!(result.stdout().retained_bytes(), input_bytes.as_slice());
    assert_private_problem_path_cleaned(&result);
}

#[test]
fn direct_arguments_are_not_shell_interpreted() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let script = dir.executable(
        "argv",
        "#!/bin/sh\nfor arg in \"$@\"; do printf '[%s]\\n' \"$arg\"; done\n/bin/cat >/dev/null\n",
        true,
    );
    let marker = dir.path().join("shell-injection-marker");
    let args = vec![
        "literal;semicolon".to_owned(),
        format!("; touch {}", marker.display()),
        "$(printf hacked)".to_owned(),
        "*.miz".to_owned(),
        "space value".to_owned(),
    ];
    let result = run_backend(run_input(
        script,
        args.clone(),
        BackendIoMode::Stdin,
        b"input".to_vec(),
        BackendResourceLimits::new(),
    ));
    let expected = args
        .iter()
        .map(|arg| format!("[{arg}]\n"))
        .collect::<String>();

    assert_eq!(result.status(), BackendRunStatus::Unknown);
    assert_eq!(
        String::from_utf8(result.stdout().retained_bytes().to_vec()).expect("utf8 argv"),
        expected
    );
    assert!(
        !marker.exists(),
        "shell metacharacter argument must not be interpreted"
    );
}

#[test]
fn command_fingerprint_excludes_local_paths_and_sorts_environment() {
    let dir_a = TestDir::new();
    let dir_b = TestDir::new();
    let script_a = dir_a.executable("backend", "#!/bin/sh\n/bin/cat\n", true);
    let script_b = dir_b.executable("backend", "#!/bin/sh\n/bin/cat\n", true);
    let env_a = BackendEnvironmentPolicy::new(vec![
        ("B".to_owned(), "2".to_owned()),
        ("A".to_owned(), "1".to_owned()),
    ])
    .expect("env a");
    let env_b = BackendEnvironmentPolicy::new(vec![
        ("A".to_owned(), "1".to_owned()),
        ("B".to_owned(), "2".to_owned()),
    ])
    .expect("env b");
    let command_a = BackendCommand::new(script_a, vec!["--mode".to_owned()])
        .expect("command a")
        .with_environment(env_a)
        .with_working_directory(BackendWorkingDirectoryPolicy::Directory(
            dir_a.path().join("work"),
        ));
    let command_b = BackendCommand::new(script_b, vec!["--mode".to_owned()])
        .expect("command b")
        .with_environment(env_b)
        .with_working_directory(BackendWorkingDirectoryPolicy::Directory(
            dir_b.path().join("other-work"),
        ));
    let input_a = run_input_from_command(
        command_a,
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    );
    let input_b = run_input_from_command(
        command_b,
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    );

    assert_eq!(command_fingerprint(&input_a), command_fingerprint(&input_b));
}

#[test]
fn command_fingerprint_records_reproducibility_configuration() {
    let dir = TestDir::new();
    let script = dir.executable("backend", "#!/bin/sh\n/bin/cat\n", true);
    let command = BackendCommand::new(script, vec!["--mode".to_owned()])
        .expect("command")
        .with_semantic_executable_id("stable-backend")
        .expect("semantic id");
    let base = run_input_from_command(
        command.clone(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    );
    let seeded = run_input_from_command(
        command.clone(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    )
    .with_random_seed(7);
    let timed = run_input_from_command(
        command.clone(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new().with_wall_timeout(Duration::from_secs(9)),
    );
    let captured = run_input_from_command(
        command.clone(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new().with_stdout_limit(3),
    );
    let limited = run_input_from_command(
        command,
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new()
            .with_unsupported_platform_limit("memory", BackendLimitRequirement::BestEffort)
            .expect("limit"),
    );

    assert_ne!(command_fingerprint(&base), command_fingerprint(&seeded));
    assert_ne!(command_fingerprint(&base), command_fingerprint(&timed));
    assert_ne!(command_fingerprint(&base), command_fingerprint(&captured));
    assert_ne!(command_fingerprint(&base), command_fingerprint(&limited));
}

#[test]
fn version_probe_records_success_and_failure_without_running_proof_policy() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let version = dir.executable(
        "version",
        "#!/bin/sh\nprintf 'MockBackend 1.2'\nprintf 'warn' >&2\n",
        true,
    );
    let backend = dir.executable("backend", "#!/bin/sh\n/bin/cat\n", true);
    let profile_with_probe = profile().with_version_probe(
        BackendVersionProbe::new(version, Vec::new(), Duration::from_secs(1))
            .expect("version probe"),
    );
    let result = run_backend(BackendRunInput::new(
        BackendRunId::new("run-version").expect("run id"),
        encoded_problem(b"p".to_vec()),
        profile_with_probe,
        BackendCommand::new(backend, Vec::new()).expect("command"),
        BackendResourceLimits::new(),
        BackendIoMode::Stdin,
        BackendCancellationToken::new(),
    ));

    let version = result.version_record().expect("version record");
    assert_eq!(result.status(), BackendRunStatus::Unknown);
    assert!(version.success());
    assert_eq!(version.parsed_version(), Some("MockBackend 1.2"));
    assert_eq!(version.stderr().retained_bytes(), b"warn");

    let failing_probe = dir.executable("bad-version", "#!/bin/sh\nexit 9\n", true);
    let backend = dir.executable("backend2", "#!/bin/sh\n/bin/cat\n", true);
    let profile_with_probe = profile().with_version_probe(
        BackendVersionProbe::new(failing_probe, Vec::new(), Duration::from_secs(1))
            .expect("version probe"),
    );
    let result = run_backend(BackendRunInput::new(
        BackendRunId::new("run-version-fail").expect("run id"),
        encoded_problem(b"p".to_vec()),
        profile_with_probe,
        BackendCommand::new(backend, Vec::new()).expect("command"),
        BackendResourceLimits::new(),
        BackendIoMode::Stdin,
        BackendCancellationToken::new(),
    ));
    assert_eq!(result.status(), BackendRunStatus::Error);
    assert_eq!(result.termination(), BackendTermination::NotStarted);
    assert!(has_diag(&result, "version_probe_failed"));
}

#[test]
fn run_metadata_projection_records_reproducibility_inputs_and_observations() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let backend = dir.executable(
        "metadata-backend",
        "#!/bin/sh\nprintf 'stdout-payload'\nprintf 'stderr-payload' >&2\n",
        true,
    );
    let version = dir.executable(
        "metadata-version",
        "#!/bin/sh\nprintf 'MockBackend 9.0'\nprintf 'version-warn' >&2\n",
        true,
    );
    let environment = BackendEnvironmentPolicy::new(vec![
        ("B".to_owned(), "2".to_owned()),
        ("A".to_owned(), "1".to_owned()),
    ])
    .expect("environment");
    let command = BackendCommand::new(backend, vec!["--mode=proof".to_owned()])
        .expect("command")
        .with_semantic_executable_id("stable-backend")
        .expect("semantic id")
        .with_environment(environment)
        .with_working_directory(BackendWorkingDirectoryPolicy::Directory(
            dir.path().to_path_buf(),
        ));
    let profile = profile().with_version_probe(
        BackendVersionProbe::new(version, Vec::new(), Duration::from_secs(1))
            .expect("version probe"),
    );
    let limits = BackendResourceLimits::new()
        .with_wall_timeout(Duration::from_secs(2))
        .with_kill_grace(Duration::from_millis(33))
        .with_stdout_limit(64)
        .with_stderr_limit(64)
        .with_unsupported_platform_limit("memory", BackendLimitRequirement::BestEffort)
        .expect("limit");
    let result = run_backend(
        BackendRunInput::new(
            BackendRunId::new("run-metadata").expect("run id"),
            encoded_problem(b"p".to_vec()),
            profile,
            command,
            limits,
            BackendIoMode::Stdin,
            BackendCancellationToken::new(),
        )
        .with_random_seed(42),
    );

    let metadata = result.metadata();

    assert_eq!(metadata.run_id().as_str(), "run-metadata");
    assert_eq!(metadata.problem_id(), result.encoded_problem().problem_id());
    assert_eq!(metadata.backend_kind().as_str(), "mock-backend");
    assert_eq!(metadata.profile_id().as_str(), "mock-profile");
    assert_eq!(metadata.concrete_format(), ConcreteFormat::Tptp);
    assert_eq!(
        metadata.encoded_input_hash(),
        result.encoded_problem().input_hash()
    );
    assert_eq!(
        metadata.encoded_metadata_hash(),
        result.encoded_problem().metadata_hash()
    );
    assert_eq!(metadata.command_fingerprint(), result.command_fingerprint());
    assert_eq!(metadata.semantic_executable_id(), "stable-backend");
    assert_eq!(metadata.args(), &["--mode=proof".to_owned()]);
    assert_eq!(
        metadata.environment(),
        &[
            ("A".to_owned(), "1".to_owned()),
            ("B".to_owned(), "2".to_owned())
        ]
    );
    assert_eq!(
        metadata.working_directory_policy_kind(),
        "explicit-directory"
    );
    assert_eq!(metadata.io_mode(), BackendIoMode::Stdin);
    assert_eq!(metadata.random_seed(), Some(42));
    assert_eq!(
        metadata.resource_limits().wall_timeout(),
        Duration::from_secs(2)
    );
    assert_eq!(
        metadata.resource_limits().kill_grace(),
        Duration::from_millis(33)
    );
    assert_eq!(metadata.resource_limits().stdout_bytes(), 64);
    assert_eq!(metadata.resource_limits().stderr_bytes(), 64);
    assert_eq!(
        metadata.resource_limits().platform_limits(),
        &[("memory".to_owned(), BackendLimitRequirement::BestEffort)]
    );
    let version = metadata.version_record().expect("version record");
    let result_version = result.version_record().expect("result version record");
    assert!(version.success());
    assert_eq!(version.parsed_version(), Some("MockBackend 9.0"));
    assert_eq!(version.stdout().retained_bytes(), b"MockBackend 9.0");
    assert_eq!(
        version.stdout().total_bytes(),
        b"MockBackend 9.0".len() as u64
    );
    assert_eq!(version.stdout().hash(), result_version.stdout().hash());
    assert!(!version.stdout().truncated());
    assert!(!version.stdout().incomplete());
    assert_eq!(version.stderr().retained_bytes(), b"version-warn");
    assert_eq!(version.stderr().total_bytes(), b"version-warn".len() as u64);
    assert_eq!(version.stderr().hash(), result_version.stderr().hash());
    assert!(!version.stderr().truncated());
    assert!(!version.stderr().incomplete());
    assert!(
        version
            .exit_status()
            .expect("version exit status")
            .success()
    );
    assert!(version.diagnostics().is_empty());
    assert_eq!(metadata.status(), BackendRunStatus::Unknown);
    assert_eq!(metadata.observed_result(), None);
    assert_eq!(metadata.termination(), BackendTermination::Exited);
    assert!(metadata.exit_status().expect("exit status").success());
    assert!(metadata.child_reaped());
    assert_eq!(metadata.elapsed(), result.elapsed());
    assert_eq!(metadata.stdout().retained_bytes(), b"stdout-payload");
    assert_eq!(metadata.stderr().retained_bytes(), b"stderr-payload");
    assert_eq!(metadata.stdout().hash(), result.stdout().hash());
    assert_eq!(metadata.stderr().hash(), result.stderr().hash());
    assert_eq!(
        metadata.stdout().total_bytes(),
        b"stdout-payload".len() as u64
    );
    assert_eq!(
        metadata.stderr().total_bytes(),
        b"stderr-payload".len() as u64
    );
    assert!(!metadata.stdout().truncated());
    assert!(!metadata.stdout().incomplete());
    assert!(!metadata.stderr().truncated());
    assert!(!metadata.stderr().incomplete());
    assert_eq!(metadata, result.metadata());
    assert!(
        metadata
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.key() == "unsupported_resource_limit")
    );
}

#[test]
fn runtime_observations_do_not_change_command_identity() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let backend_a = dir.executable("runtime-a", "#!/bin/sh\nprintf 'first'\n", true);
    let backend_b = dir.executable("runtime-b", "#!/bin/sh\nprintf 'second'\n", true);
    let command_a = BackendCommand::new(backend_a, Vec::new())
        .expect("command")
        .with_semantic_executable_id("stable-runtime-backend")
        .expect("semantic id");
    let command_b = BackendCommand::new(backend_b, Vec::new())
        .expect("command")
        .with_semantic_executable_id("stable-runtime-backend")
        .expect("semantic id");

    let result_a = run_backend(run_input_from_command(
        command_a,
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    let result_b = run_backend(run_input_from_command(
        command_b,
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    let metadata_a = result_a.metadata();
    let metadata_b = result_b.metadata();

    assert_eq!(
        metadata_a.command_fingerprint(),
        metadata_b.command_fingerprint()
    );
    assert_ne!(metadata_a.stdout().hash(), metadata_b.stdout().hash());
    assert_eq!(
        metadata_a.semantic_executable_id(),
        "stable-runtime-backend"
    );
    assert_eq!(
        metadata_b.semantic_executable_id(),
        "stable-runtime-backend"
    );
}

#[test]
fn timeout_cancellation_crash_missing_and_permission_fail_closed() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let sleep = dir.executable("sleep", "#!/bin/sh\n/bin/sleep 5\n", true);
    let limits = BackendResourceLimits::new()
        .with_wall_timeout(Duration::from_millis(30))
        .with_kill_grace(Duration::from_millis(30));
    let result = run_backend(run_input(
        sleep,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        limits,
    ));
    assert_eq!(result.status(), BackendRunStatus::Timeout);
    assert_eq!(result.termination(), BackendTermination::Timeout);
    assert!(result.child_reaped());

    let sleep = dir.executable("sleep-cancel", "#!/bin/sh\n/bin/sleep 5\n", true);
    let token = BackendCancellationToken::new();
    let canceller = token.clone();
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(30));
        canceller.cancel();
    });
    let result = run_backend(BackendRunInput::new(
        BackendRunId::new("run-cancel").expect("run id"),
        encoded_problem(b"p".to_vec()),
        profile(),
        BackendCommand::new(sleep, Vec::new()).expect("command"),
        BackendResourceLimits::new()
            .with_wall_timeout(Duration::from_secs(5))
            .with_kill_grace(Duration::from_millis(30)),
        BackendIoMode::Stdin,
        token,
    ));
    handle.join().expect("canceller joins");
    assert_eq!(result.status(), BackendRunStatus::Cancelled);
    assert_eq!(result.termination(), BackendTermination::Cancelled);
    assert!(result.child_reaped());

    let crash = dir.executable("crash", "#!/bin/sh\nexit 42\n", true);
    let result = run_backend(run_input(
        crash,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    assert_eq!(result.status(), BackendRunStatus::Error);
    assert!(result.child_reaped());
    assert!(has_diag(&result, "process_crash"));

    let missing = dir.path().join("missing-backend");
    let result = run_backend(run_input(
        missing,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    assert_eq!(result.status(), BackendRunStatus::Error);
    assert_eq!(result.termination(), BackendTermination::SpawnFailure);
    assert!(has_diag(&result, "spawn_failed"));

    let denied = dir.executable("not-executable", "#!/bin/sh\n/bin/cat\n", false);
    let result = run_backend(run_input(
        denied,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    assert_eq!(result.status(), BackendRunStatus::Error);
    assert_eq!(result.termination(), BackendTermination::SpawnFailure);
    assert!(has_diag(&result, "spawn_failed"));
}

#[test]
fn private_problem_file_is_cleaned_after_timeout_cancellation_crash_and_spawn_failure() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let print_path_then_loop = "#!/bin/sh\nprintf '%s\\n' \"$1\" >&2\nwhile :; do :; done\n";

    let timeout_script = dir.executable("file-timeout", print_path_then_loop, true);
    let timed_out = run_backend(run_input(
        timeout_script,
        Vec::new(),
        BackendIoMode::PrivateProblemFile,
        b"p".to_vec(),
        BackendResourceLimits::new()
            .with_wall_timeout(Duration::from_millis(30))
            .with_kill_grace(Duration::from_millis(30)),
    ));
    assert_eq!(timed_out.status(), BackendRunStatus::Timeout);
    assert_private_problem_path_cleaned(&timed_out);

    let cancel_script = dir.executable("file-cancel", print_path_then_loop, true);
    let token = BackendCancellationToken::new();
    let canceller = token.clone();
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(30));
        canceller.cancel();
    });
    let cancelled = run_backend(BackendRunInput::new(
        BackendRunId::new("run-file-cancel").expect("run id"),
        encoded_problem(b"p".to_vec()),
        profile(),
        BackendCommand::new(cancel_script, Vec::new()).expect("command"),
        BackendResourceLimits::new()
            .with_wall_timeout(Duration::from_secs(5))
            .with_kill_grace(Duration::from_millis(30)),
        BackendIoMode::PrivateProblemFile,
        token,
    ));
    handle.join().expect("canceller joins");
    assert_eq!(cancelled.status(), BackendRunStatus::Cancelled);
    assert_private_problem_path_cleaned(&cancelled);

    let crash_script = dir.executable(
        "file-crash",
        "#!/bin/sh\nprintf '%s\\n' \"$1\" >&2\nexit 42\n",
        true,
    );
    let crashed = run_backend(run_input(
        crash_script,
        Vec::new(),
        BackendIoMode::PrivateProblemFile,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    assert_eq!(crashed.status(), BackendRunStatus::Error);
    assert_private_problem_path_cleaned(&crashed);

    let before = backend_temp_dirs();
    let missing = dir.path().join("missing-file-backend");
    let missing_result = run_backend(run_input(
        missing,
        Vec::new(),
        BackendIoMode::PrivateProblemFile,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    let after = backend_temp_dirs();
    assert_eq!(missing_result.status(), BackendRunStatus::Error);
    assert_eq!(
        missing_result.termination(),
        BackendTermination::SpawnFailure
    );
    assert_eq!(
        after, before,
        "spawn failure must not leave private backend temp directories"
    );
}

#[test]
fn stdout_and_stderr_are_drained_after_retained_limit_and_hash_complete_streams() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let script = dir.executable(
        "big-output",
        "#!/bin/sh\nprintf 'abcdefghijklmnopqrstuvwxyz'\nprintf 'stderr-output' >&2\n",
        true,
    );
    let limited = run_backend(run_input(
        script.clone(),
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new()
            .with_stdout_limit(5)
            .with_stderr_limit(6),
    ));
    let full = run_backend(run_input(
        script,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new()
            .with_stdout_limit(64)
            .with_stderr_limit(64),
    ));

    assert_eq!(limited.stdout().retained_bytes(), b"abcde");
    assert_eq!(limited.stdout().total_bytes(), 26);
    assert!(limited.stdout().truncated());
    assert_eq!(limited.stdout().hash(), full.stdout().hash());
    assert_eq!(limited.stderr().retained_bytes(), b"stderr");
    assert_eq!(limited.stderr().total_bytes(), 13);
    assert!(limited.stderr().truncated());
    assert_eq!(limited.stderr().hash(), full.stderr().hash());
}

#[test]
fn descendant_held_output_pipe_returns_bounded_incomplete_and_never_proves() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let script = dir.executable(
        "held-output",
        "#!/bin/sh\ntrap '' HUP\n/bin/sleep 1 &\nprintf 'done'\n/bin/cat >/dev/null\nexit 0\n",
        true,
    );
    let start = Instant::now();
    let result = run_backend(run_input(
        script,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new().with_kill_grace(Duration::from_millis(20)),
    ));

    assert!(
        start.elapsed() < Duration::from_millis(500),
        "capture join should be bounded when descendants keep a pipe open"
    );
    assert_eq!(result.status(), BackendRunStatus::Unknown);
    assert!(result.stdout().incomplete());
    assert!(has_diag(&result, "capture_reader_incomplete"));
    assert!(has_diag(&result, "stdout_incomplete"));

    let classified = classify_backend_observation(
        result.clone(),
        BackendObservation::new(BackendObservedResult::Unsat)
            .with_candidate_evidence(candidate(result.encoded_problem()))
            .without_complete_output_requirement(),
    );
    assert_eq!(classified.status(), BackendRunStatus::Error);
    assert!(has_diag(&classified, "proved_rejected_incomplete_output"));
}

#[test]
fn descendant_held_stdin_file_descriptor_does_not_block_runner() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let script = dir.executable(
        "held-stdin",
        "#!/bin/sh\nexec 3<&0\n/bin/sleep 1 <&3 &\nexit 0\n",
        true,
    );
    let start = Instant::now();
    let result = run_backend(run_input(
        script,
        Vec::new(),
        BackendIoMode::Stdin,
        vec![b'x'; 2 * 1024 * 1024],
        BackendResourceLimits::new().with_kill_grace(Duration::from_millis(20)),
    ));

    assert!(
        start.elapsed() < Duration::from_millis(500),
        "stdin mode should not depend on a blocking writer thread"
    );
    assert_eq!(result.status(), BackendRunStatus::Unknown);
    assert!(result.child_reaped());
    assert!(!has_diag(&result, "stdin_spool_file_failed"));
}

#[test]
fn unsupported_required_resource_limit_is_error_before_spawn() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let script = dir.executable("backend", "#!/bin/sh\n/bin/cat\n", true);
    let limits = BackendResourceLimits::new()
        .with_unsupported_platform_limit("memory", BackendLimitRequirement::Required)
        .expect("limit");
    let result = run_backend(run_input(
        script,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        limits,
    ));

    assert_eq!(result.status(), BackendRunStatus::Error);
    assert_eq!(result.termination(), BackendTermination::NotStarted);
    assert!(has_diag(&result, "required_resource_limit_unsupported"));
}

#[test]
fn mock_proved_requires_unsat_and_matching_formula_substitution_candidate() {
    let _guard = private_temp_guard();
    let result = successful_result();
    let candidate = candidate(result.encoded_problem());
    let classified = classify_backend_observation(
        result,
        BackendObservation::new(BackendObservedResult::Unsat).with_candidate_evidence(candidate),
    );

    assert_eq!(classified.status(), BackendRunStatus::Proved);
    assert_eq!(
        classified.observed_result(),
        Some(BackendObservedResult::Unsat)
    );
    assert!(classified.candidate_evidence().is_some());

    let ref_candidate = BackendCandidateEvidence::new(
        "candidate-ref",
        BackendCandidatePayload::FormulaSubstitutionRef("artifact://candidate".to_owned()),
        classified.encoded_problem().target_binding().clone(),
        classified.encoded_problem().input_hash(),
        classified.encoded_problem().provenance_hash(),
        classified.encoded_problem().formula_labels().to_vec(),
        classified.encoded_problem().symbol_bindings().to_vec(),
    )
    .expect("ref candidate");
    let classified = classify_backend_observation(
        successful_result(),
        BackendObservation::new(BackendObservedResult::Unsat)
            .with_candidate_evidence(ref_candidate),
    );
    assert_eq!(classified.status(), BackendRunStatus::Proved);
}

#[test]
fn proved_rejects_missing_or_prohibited_payload_and_metadata_mismatches() {
    let _guard = private_temp_guard();
    let result = successful_result();
    let no_candidate = classify_backend_observation(
        result.clone(),
        BackendObservation::new(BackendObservedResult::Unsat),
    );
    assert_eq!(no_candidate.status(), BackendRunStatus::Unknown);
    assert!(has_diag(&no_candidate, "proved_rejected_missing_candidate"));

    for (name, payload) in [
        (
            "backend-proof-method",
            BackendCandidatePayload::BackendProofMethod(vec![1]),
        ),
        ("backend-log", BackendCandidatePayload::BackendLog(vec![1])),
        ("unsat-core", BackendCandidatePayload::UnsatCore(vec![1])),
        (
            "smt-proof-object",
            BackendCandidatePayload::SmtProofObject(vec![1]),
        ),
        ("tstp-trace", BackendCandidatePayload::TstpTrace(vec![1])),
        (
            "resolution-trace",
            BackendCandidatePayload::ResolutionTrace(vec![1]),
        ),
        ("used-axioms", BackendCandidatePayload::UsedAxioms(vec![1])),
    ] {
        let prohibited = BackendCandidateEvidence::new(
            name,
            payload,
            result.encoded_problem().target_binding().clone(),
            result.encoded_problem().input_hash(),
            result.encoded_problem().provenance_hash(),
            result.encoded_problem().formula_labels().to_vec(),
            result.encoded_problem().symbol_bindings().to_vec(),
        )
        .expect("prohibited candidate");
        let classified = classify_backend_observation(
            result.clone(),
            BackendObservation::new(BackendObservedResult::Unsat)
                .with_candidate_evidence(prohibited),
        );
        assert_eq!(classified.status(), BackendRunStatus::Unknown, "{name}");
        assert!(
            has_diag(&classified, "unsupported_candidate_payload"),
            "{name}"
        );
    }

    for (candidate, key) in [
        (
            candidate_with_target_mismatch(result.encoded_problem()),
            "candidate_target_mismatch",
        ),
        (
            candidate_with_input_mismatch(result.encoded_problem()),
            "candidate_input_hash_mismatch",
        ),
        (
            candidate_with_provenance_mismatch(result.encoded_problem()),
            "candidate_provenance_mismatch",
        ),
        (
            candidate_with_label_mismatch(result.encoded_problem()),
            "candidate_label_mismatch",
        ),
        (
            candidate_with_symbol_mismatch(result.encoded_problem()),
            "candidate_symbol_mismatch",
        ),
    ] {
        let classified = classify_backend_observation(
            result.clone(),
            BackendObservation::new(BackendObservedResult::Unsat)
                .with_candidate_evidence(candidate),
        );
        assert_eq!(classified.status(), BackendRunStatus::Unknown, "{key}");
        assert!(has_diag(&classified, key), "{key}");
    }
}

#[test]
fn proved_rejects_otherwise_matching_evidence_after_bad_process_status_or_truncation() {
    let _guard = private_temp_guard();
    let dir = TestDir::new();
    let sleep = dir.executable("sleep-proof", "#!/bin/sh\n/bin/sleep 5\n", true);
    let timed_out = run_backend(run_input(
        sleep,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new()
            .with_wall_timeout(Duration::from_millis(20))
            .with_kill_grace(Duration::from_millis(20)),
    ));
    let classified = classify_backend_observation(
        timed_out.clone(),
        BackendObservation::new(BackendObservedResult::Unsat)
            .with_candidate_evidence(candidate(timed_out.encoded_problem())),
    );
    assert_eq!(classified.status(), BackendRunStatus::Timeout);
    assert!(has_diag(&classified, "proved_rejected_process_status"));

    let sleep = dir.executable("cancel-proof", "#!/bin/sh\n/bin/sleep 5\n", true);
    let token = BackendCancellationToken::new();
    let canceller = token.clone();
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        canceller.cancel();
    });
    let cancelled = run_backend(BackendRunInput::new(
        BackendRunId::new("run-cancel-proof").expect("run id"),
        encoded_problem(b"p".to_vec()),
        profile(),
        BackendCommand::new(sleep, Vec::new()).expect("command"),
        BackendResourceLimits::new()
            .with_wall_timeout(Duration::from_secs(5))
            .with_kill_grace(Duration::from_millis(20)),
        BackendIoMode::Stdin,
        token,
    ));
    handle.join().expect("canceller joins");
    let classified = classify_with_matching_unsat_candidate(cancelled.clone());
    assert_eq!(classified.status(), BackendRunStatus::Cancelled);
    assert!(has_diag(&classified, "proved_rejected_process_status"));

    let crash = dir.executable("crash-proof", "#!/bin/sh\nexit 42\n", true);
    let crashed = run_backend(run_input(
        crash,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    let classified = classify_with_matching_unsat_candidate(crashed.clone());
    assert_eq!(classified.status(), BackendRunStatus::Error);
    assert!(has_diag(&classified, "proved_rejected_process_status"));

    let missing = dir.path().join("missing-proof-backend");
    let missing_result = run_backend(run_input(
        missing,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    let classified = classify_with_matching_unsat_candidate(missing_result.clone());
    assert_eq!(classified.status(), BackendRunStatus::Error);
    assert!(has_diag(&classified, "proved_rejected_process_status"));

    let denied = dir.executable("denied-proof", "#!/bin/sh\n/bin/cat\n", false);
    let denied_result = run_backend(run_input(
        denied,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    let classified = classify_with_matching_unsat_candidate(denied_result.clone());
    assert_eq!(classified.status(), BackendRunStatus::Error);
    assert!(has_diag(&classified, "proved_rejected_process_status"));

    let unsupported_required = run_backend(run_input(
        dir.executable("unsupported-proof", "#!/bin/sh\n/bin/cat\n", true),
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new()
            .with_unsupported_platform_limit("memory", BackendLimitRequirement::Required)
            .expect("limit"),
    ));
    let classified = classify_with_matching_unsat_candidate(unsupported_required.clone());
    assert_eq!(classified.status(), BackendRunStatus::Error);
    assert!(has_diag(&classified, "proved_rejected_process_status"));

    let script = dir.executable("truncated-proof", "#!/bin/sh\nprintf 'abcdef'\n", true);
    let truncated = run_backend(run_input(
        script,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new().with_stdout_limit(1),
    ));
    assert!(truncated.stdout().truncated());
    let classified = classify_backend_observation(
        truncated.clone(),
        BackendObservation::new(BackendObservedResult::Unsat)
            .with_candidate_evidence(candidate(truncated.encoded_problem())),
    );
    assert_eq!(classified.status(), BackendRunStatus::Error);
    assert!(has_diag(&classified, "proved_rejected_incomplete_output"));
}

#[test]
fn counterexample_unknown_cancelled_and_error_never_accept_proof_status() {
    let _guard = private_temp_guard();
    let result = successful_result();
    let counterexample = BackendCounterexample::new(b"model".to_vec(), true).expect("model");
    let classified = classify_backend_observation(
        result.clone(),
        BackendObservation::new(BackendObservedResult::Sat).with_counterexample(counterexample),
    );
    assert_eq!(classified.status(), BackendRunStatus::Counterexample);
    assert!(classified.candidate_evidence().is_none());

    let classified = classify_backend_observation(
        result.clone(),
        BackendObservation::new(BackendObservedResult::Unknown),
    );
    assert_eq!(classified.status(), BackendRunStatus::Unknown);

    let classified = classify_backend_observation(
        result,
        BackendObservation::new(BackendObservedResult::Malformed),
    );
    assert_eq!(classified.status(), BackendRunStatus::Error);
}

#[test]
fn constructor_validation_rejects_empty_and_duplicate_trusted_surface_inputs() {
    assert_eq!(
        BackendRunId::new("").unwrap_err(),
        BackendConfigError::EmptyField { field: "run_id" }
    );
    assert_eq!(
        BackendEnvironmentPolicy::new(vec![
            ("A".to_owned(), "1".to_owned()),
            ("A".to_owned(), "2".to_owned()),
        ])
        .unwrap_err(),
        BackendConfigError::DuplicateEnvironmentKey {
            key: "A".to_owned()
        }
    );
    assert!(matches!(
        BackendCandidateEvidence::new(
            "empty",
            BackendCandidatePayload::FormulaSubstitutionBytes(Vec::new()),
            target_binding("empty"),
            hash_with_seed(1),
            hash_with_seed(2),
            Vec::new(),
            Vec::new(),
        ),
        Err(BackendConfigError::EmptyField {
            field: "candidate_payload"
        })
    ));
}

fn successful_result() -> BackendRunResult {
    let dir = TestDir::new();
    let script = dir.executable("ok", "#!/bin/sh\n/bin/cat\n", true);
    let result = run_backend(run_input(
        script,
        Vec::new(),
        BackendIoMode::Stdin,
        b"p".to_vec(),
        BackendResourceLimits::new(),
    ));
    assert_eq!(result.status(), BackendRunStatus::Unknown);
    result
}

fn run_input(
    executable: PathBuf,
    args: Vec<String>,
    io_mode: BackendIoMode,
    input_text: Vec<u8>,
    resource_limits: BackendResourceLimits,
) -> BackendRunInput {
    run_input_from_command(
        BackendCommand::new(executable, args).expect("command"),
        io_mode,
        input_text,
        resource_limits,
    )
}

fn run_input_from_command(
    command: BackendCommand,
    io_mode: BackendIoMode,
    input_text: Vec<u8>,
    resource_limits: BackendResourceLimits,
) -> BackendRunInput {
    BackendRunInput::new(
        BackendRunId::new("run").expect("run id"),
        encoded_problem(input_text),
        profile(),
        command,
        resource_limits,
        io_mode,
        BackendCancellationToken::new(),
    )
}

fn profile() -> BackendProfile {
    BackendProfile::new(
        BackendProfileId::new("mock-profile").expect("profile id"),
        BackendKind::new("mock-backend").expect("backend kind"),
        ConcreteFormat::Tptp,
    )
}

fn encoded_problem(input_text: Vec<u8>) -> EncodedBackendProblem {
    let problem = minimal_problem();
    EncodedBackendProblem::new(EncodedBackendProblemParts {
        problem_id: problem.problem_id(),
        target_binding: problem.target_binding().clone(),
        expected_result: problem.expected_result(),
        concrete_format: ConcreteFormat::Tptp,
        logic_profile_name: problem.logic_profile().name().as_str().to_owned(),
        logic_fragment: "Fof".to_owned(),
        input_text,
        formula_labels: vec!["ax_1".to_owned(), "conj_1".to_owned()],
        symbol_bindings: vec!["P".to_owned()],
        provenance_hash: hash_with_seed(9),
    })
    .expect("encoded problem")
}

fn minimal_problem() -> AtpProblem {
    AtpProblem::try_new(AtpProblemParts {
        vc_id: VcId::new(7),
        target_binding: target_binding("vc:7"),
        logic_profile: LogicProfile::try_new(
            "fof-fixture",
            LogicFragment::Fof,
            EqualitySupport::Unsupported,
            QuantifierPolicy::PropositionalOnly,
            SoftTypeStrategy::BackendSorts,
            NativePropertySupport::Unsupported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("logic profile"),
        expected_result: ExpectedBackendResult::Unsat,
        declarations: vec![AtpDeclaration::new(
            AtpDeclarationId::new(1),
            AtpDeclarationKind::Predicate,
            "P",
            0,
            AtpProvenanceId::new(1),
        )],
        axioms: vec![AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Atom(AtpAtom::new("P", Vec::new())),
            AtpProvenanceId::new(2),
        )],
        conjecture: AtpFormula::new(
            AtpFormulaId::new(2),
            AtpFormulaTree::Atom(AtpAtom::new("P", Vec::new())),
            AtpProvenanceId::new(3),
        ),
        type_context: AtpTypeContext::new(Vec::new()),
        properties: Vec::new(),
        symbol_map: vec![AtpSymbolMapEntry::new(
            "P",
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:P")),
        )],
        provenance: vec![
            AtpProvenance::new(
                AtpProvenanceId::new(1),
                AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("decl:P")),
                "decl",
            ),
            AtpProvenance::new(
                AtpProvenanceId::new(2),
                AtpSourceRef::CitedPremise(AtpSourceBinding::new("premise:1")),
                "premise",
            ),
            AtpProvenance::new(
                AtpProvenanceId::new(3),
                AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")),
                "goal",
            ),
        ],
        diagnostics: Vec::new(),
    })
    .expect("minimal problem")
}

fn candidate(problem: &EncodedBackendProblem) -> BackendCandidateEvidence {
    BackendCandidateEvidence::new(
        "candidate",
        BackendCandidatePayload::FormulaSubstitutionBytes(b"formula-substitution".to_vec()),
        problem.target_binding().clone(),
        problem.input_hash(),
        problem.provenance_hash(),
        problem.formula_labels().to_vec(),
        problem.symbol_bindings().to_vec(),
    )
    .expect("candidate")
}

fn candidate_with_target_mismatch(problem: &EncodedBackendProblem) -> BackendCandidateEvidence {
    BackendCandidateEvidence::new(
        "target-mismatch",
        BackendCandidatePayload::FormulaSubstitutionBytes(vec![1]),
        target_binding("other-vc"),
        problem.input_hash(),
        problem.provenance_hash(),
        problem.formula_labels().to_vec(),
        problem.symbol_bindings().to_vec(),
    )
    .expect("candidate")
}

fn candidate_with_input_mismatch(problem: &EncodedBackendProblem) -> BackendCandidateEvidence {
    BackendCandidateEvidence::new(
        "input-mismatch",
        BackendCandidatePayload::FormulaSubstitutionBytes(vec![1]),
        problem.target_binding().clone(),
        hash_with_seed(44),
        problem.provenance_hash(),
        problem.formula_labels().to_vec(),
        problem.symbol_bindings().to_vec(),
    )
    .expect("candidate")
}

fn candidate_with_provenance_mismatch(problem: &EncodedBackendProblem) -> BackendCandidateEvidence {
    BackendCandidateEvidence::new(
        "provenance-mismatch",
        BackendCandidatePayload::FormulaSubstitutionBytes(vec![1]),
        problem.target_binding().clone(),
        problem.input_hash(),
        hash_with_seed(45),
        problem.formula_labels().to_vec(),
        problem.symbol_bindings().to_vec(),
    )
    .expect("candidate")
}

fn candidate_with_label_mismatch(problem: &EncodedBackendProblem) -> BackendCandidateEvidence {
    BackendCandidateEvidence::new(
        "label-mismatch",
        BackendCandidatePayload::FormulaSubstitutionBytes(vec![1]),
        problem.target_binding().clone(),
        problem.input_hash(),
        problem.provenance_hash(),
        vec!["ax_1".to_owned(), "different".to_owned()],
        problem.symbol_bindings().to_vec(),
    )
    .expect("candidate")
}

fn candidate_with_symbol_mismatch(problem: &EncodedBackendProblem) -> BackendCandidateEvidence {
    BackendCandidateEvidence::new(
        "symbol-mismatch",
        BackendCandidatePayload::FormulaSubstitutionBytes(vec![1]),
        problem.target_binding().clone(),
        problem.input_hash(),
        problem.provenance_hash(),
        problem.formula_labels().to_vec(),
        vec!["Q".to_owned()],
    )
    .expect("candidate")
}

fn target_binding(source: &str) -> AtpTargetBinding {
    AtpTargetBinding::new(
        AtpFingerprint::new(18, source.as_bytes().to_vec()).expect("fingerprint"),
        AtpSourceBinding::new(source),
    )
    .expect("target binding")
}

fn hash_with_seed(seed: u8) -> Hash {
    Hash::from_bytes([seed; Hash::BYTE_LEN])
}

fn has_diag(result: &BackendRunResult, key: &str) -> bool {
    result
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.key() == key)
}

fn classify_with_matching_unsat_candidate(result: BackendRunResult) -> BackendRunResult {
    let candidate = candidate(result.encoded_problem());
    classify_backend_observation(
        result,
        BackendObservation::new(BackendObservedResult::Unsat).with_candidate_evidence(candidate),
    )
}

fn private_temp_guard() -> MutexGuard<'static, ()> {
    match PRIVATE_TEMP_TEST_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn private_problem_path_from_stderr(result: &BackendRunResult) -> PathBuf {
    let path = String::from_utf8(result.stderr().retained_bytes().to_vec())
        .expect("private problem path is utf8")
        .trim()
        .to_owned();
    assert!(
        !path.is_empty(),
        "mock backend should echo the private problem path"
    );
    PathBuf::from(path)
}

fn assert_private_problem_path_cleaned(result: &BackendRunResult) {
    let problem_path = private_problem_path_from_stderr(result);
    assert!(
        !problem_path.exists(),
        "private problem file should be removed after the run"
    );
    assert!(
        !problem_path
            .parent()
            .expect("problem file has parent")
            .exists(),
        "private problem directory should be removed after the run"
    );
}

fn backend_temp_dirs() -> BTreeSet<PathBuf> {
    fs::read_dir(std::env::temp_dir())
        .expect("read temp dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .is_some_and(|name| name.starts_with("mizar-atp-backend-"))
        })
        .collect()
}

fn readlink_command_for_test() -> Option<&'static str> {
    ["/usr/bin/readlink", "/bin/readlink"]
        .into_iter()
        .find(|path| Path::new(path).exists())
}

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "mizar-atp-backend-test-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&path).expect("create test dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn executable(&self, name: &str, source: &str, executable: bool) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, source).expect("write script");
        let mode = if executable { 0o700 } else { 0o600 };
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).expect("chmod script");
        path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
