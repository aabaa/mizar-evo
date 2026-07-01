use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use mizar_build::module_index::WorkspaceSourceFile;
use mizar_driver::{
    cli::{
        CliBatchInput, CliBuildProfile, CliExitCode, CliInvocation, CliMessageFormat,
        CliSnapshotInputs, run_batch, run_batch_with_driver,
    },
    driver::CompilerDriver,
    request::SourceInputSet,
};
use mizar_session::{
    BuildRequestId, BuildSessionId, BuildSnapshotId, DependencyArtifactRef, Edition, Hash, IdError,
    InMemorySessionIdAllocator, ModulePath, NormalizedPath, PackageId, SessionIdAllocator,
    SnapshotLeaseId, SnapshotRegistry, SourceId, SourceMapId, SourceOrigin, SourceVersion,
    ToolchainInfo, WorkspaceRoot, normalize_source_path,
};

static NEXT_FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

#[test]
fn parse_build_arguments_into_batch_request_controls() {
    let invocation = CliInvocation::parse([
        "mizar",
        "build",
        "--workspace",
        "workspace",
        "--package",
        "alpha",
        "--profile",
        "release",
        "--target",
        "alpha.main",
        "--jobs",
        "4",
        "--locked",
        "--no-incremental",
        "--message-format",
        "json",
        "--quiet",
    ])
    .expect("fixture arguments parse");

    assert_eq!(invocation.workspace, "workspace");
    assert_eq!(invocation.packages, vec!["alpha"]);
    assert_eq!(invocation.profile, CliBuildProfile::Release);
    assert_eq!(invocation.targets, vec!["alpha.main"]);
    assert_eq!(invocation.jobs, 4);
    assert!(invocation.locked);
    assert!(!invocation.incremental);
    assert_eq!(invocation.message_format, CliMessageFormat::Json);
    assert!(invocation.quiet);

    let draft = invocation.request_draft(&snapshot_inputs(SourceInputSet::default(), 10));
    assert_eq!(draft.workspace_root, WorkspaceRoot::new("workspace"));
    assert_eq!(draft.profile.name, "release");
    assert_eq!(
        draft
            .targets
            .packages
            .iter()
            .map(PackageId::as_str)
            .collect::<Vec<_>>(),
        vec!["alpha"]
    );
    assert_eq!(
        draft
            .targets
            .modules
            .iter()
            .map(ModulePath::as_str)
            .collect::<Vec<_>>(),
        vec!["alpha.main"]
    );
}

#[test]
fn batch_success_renders_human_progress_from_event_stream() {
    let output = run_batch(
        ["build", "--workspace", "workspace"],
        batch_input(Vec::new(), SourceInputSet::default(), 20),
    );

    assert_eq!(output.exit_code, CliExitCode::Success);
    assert_eq!(output.process_code(), 0);
    assert!(output.stdout.is_empty());
    assert_eq!(
        human_event_kinds(&output.stderr),
        [
            "session_accepted",
            "snapshot_captured",
            "planning_ready",
            "task_progress",
            "session_finished"
        ]
    );
    assert_eq!(human_scheduler_events(&output.stderr), ["task_finished"]);
    assert!(output.stderr.contains("session_finished outcome=succeeded"));
}

#[test]
fn missing_phase_services_exit_unavailable_without_fake_publications() {
    let source_inputs = SourceInputSet {
        versions: vec![source_version("src/main.miz", "main", 30)],
    };
    let output = run_batch(
        ["build", "--workspace", "workspace"],
        batch_input(
            vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
            source_inputs,
            31,
        ),
    );

    assert_eq!(output.exit_code, CliExitCode::UnavailableOwner);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.contains("phase_service_gap"));
    assert!(
        output
            .stderr
            .contains("availability=external_dependency_gap")
    );
    assert!(output.stderr.contains("session_finished outcome=blocked"));
    assert!(!output.stderr.contains("artifact_boundary"));
    assert!(!output.stderr.contains("diagnostics_ready"));
}

#[test]
fn json_progress_uses_protocol_agnostic_event_records() {
    let output = run_batch(
        [
            "mizar",
            "build",
            "--workspace",
            "workspace",
            "--message-format",
            "json",
        ],
        batch_input(Vec::new(), SourceInputSet::default(), 40),
    );

    assert_eq!(output.exit_code, CliExitCode::Success);
    assert!(output.stderr.is_empty());
    assert_eq!(
        json_event_kinds(&output.stdout),
        [
            "session_accepted",
            "snapshot_captured",
            "planning_ready",
            "task_progress",
            "session_finished"
        ]
    );
    assert_eq!(json_scheduler_events(&output.stdout), ["task_finished"]);
    assert!(!output.stdout.contains("jsonrpc"));
}

#[test]
fn manifest_errors_exit_build_failed_with_diagnostics_bridge_gap() {
    let mut input = batch_input(Vec::new(), SourceInputSet::default(), 45);
    input.package_manifest = "not = [valid".to_owned();

    let output = run_batch(["build", "--workspace", "workspace"], input);

    assert_eq!(output.exit_code, CliExitCode::BuildFailed);
    assert_eq!(output.process_code(), 1);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.contains("owner_gap owner=manifest"));
    assert!(
        output
            .stderr
            .contains("classification=external_dependency_gap diagnostics=")
    );
}

#[test]
fn manifest_errors_render_structured_json_diagnostics_bridge_gap() {
    let mut input = batch_input(Vec::new(), SourceInputSet::default(), 46);
    input.package_manifest = "not = [valid".to_owned();

    let output = run_batch(
        [
            "build",
            "--workspace",
            "workspace",
            "--message-format",
            "json",
        ],
        input,
    );

    assert_eq!(output.exit_code, CliExitCode::BuildFailed);
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        "{\"schema_version\":1,\"kind\":\"owner_gap\",\"owner\":\"manifest\",\"classification\":\"external_dependency_gap\",\"diagnostics\":1}\n"
    );
}

#[test]
fn driver_diagnostics_errors_render_explicit_bridge_gap_after_events() {
    let source_inputs = SourceInputSet {
        versions: vec![source_version("src/main.miz", "main", 47)],
    };
    let output = run_batch(
        ["build", "--workspace", "workspace"],
        batch_input(
            vec![WorkspaceSourceFile::new("src/main.miz", "../bad.miz")],
            source_inputs,
            48,
        ),
    );

    assert_eq!(output.exit_code, CliExitCode::BuildFailed);
    assert_eq!(output.process_code(), 1);
    assert!(output.stdout.is_empty());
    assert_eq!(
        human_event_kinds(&output.stderr),
        [
            "session_accepted",
            "snapshot_captured",
            "planning_ready",
            "session_finished"
        ]
    );
    assert!(output.stderr.contains("planning_ready status=failed"));
    assert!(output.stderr.contains("session_finished outcome=failed"));
    assert!(output.stderr.contains("owner_gap owner=module_index"));
    assert!(
        output
            .stderr
            .contains("classification=external_dependency_gap diagnostics=")
    );
}

#[test]
fn driver_diagnostics_errors_render_structured_json_bridge_gap() {
    let source_inputs = SourceInputSet {
        versions: vec![source_version("src/main.miz", "main", 49)],
    };
    let output = run_batch(
        [
            "build",
            "--workspace",
            "workspace",
            "--message-format",
            "json",
        ],
        batch_input(
            vec![WorkspaceSourceFile::new("src/main.miz", "../bad.miz")],
            source_inputs,
            50,
        ),
    );

    assert_eq!(output.exit_code, CliExitCode::BuildFailed);
    assert!(output.stderr.is_empty());
    assert_eq!(
        json_event_kinds(&output.stdout),
        [
            "session_accepted",
            "snapshot_captured",
            "planning_ready",
            "session_finished",
            "owner_gap"
        ]
    );
    assert!(output.stdout.contains(
        "{\"schema_version\":1,\"kind\":\"owner_gap\",\"owner\":\"module_index\",\"classification\":\"external_dependency_gap\",\"diagnostics\":1}"
    ));
}

#[test]
fn superseded_batch_submission_maps_to_cancelled_exit_code() {
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    let first = run_batch_with_driver(
        ["build", "--workspace", "workspace"],
        batch_input(Vec::new(), SourceInputSet::default(), 70),
        &mut driver,
        &ids,
        &snapshots,
    );
    assert_eq!(first.exit_code, CliExitCode::Success);

    let second = run_batch_with_driver(
        ["build", "--workspace", "workspace"],
        batch_input(Vec::new(), SourceInputSet::default(), 71),
        &mut driver,
        &ids,
        &snapshots,
    );

    assert_eq!(second.exit_code, CliExitCode::Cancelled);
    assert_eq!(second.process_code(), 4);
    assert_eq!(
        human_event_kinds(&second.stderr),
        [
            "session_accepted",
            "snapshot_captured",
            "publication_suppressed",
            "session_finished"
        ]
    );
    assert!(
        second
            .stderr
            .contains("session_finished outcome=superseded")
    );
}

#[test]
fn usage_errors_have_stable_exit_code_before_driver_submission() {
    let output = run_batch(
        ["build", "--jobs", "0"],
        batch_input(Vec::new(), SourceInputSet::default(), 50),
    );

    assert_eq!(output.exit_code, CliExitCode::Usage);
    assert_eq!(output.process_code(), 2);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.contains("usage_error"));
    assert!(output.stderr.contains("jobs must be at least 1"));
}

#[test]
fn usage_errors_honor_json_message_format() {
    let output = run_batch(
        ["build", "--message-format", "json", "--jobs", "0"],
        batch_input(Vec::new(), SourceInputSet::default(), 51),
    );

    assert_eq!(output.exit_code, CliExitCode::Usage);
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        "{\"schema_version\":1,\"kind\":\"usage_error\",\"reason\":\"jobs must be at least 1\"}\n"
    );
}

#[test]
fn quiet_output_still_uses_session_finished_event_only() {
    let output = run_batch(
        [
            "build",
            "--workspace",
            "workspace",
            "--quiet",
            "--message-format",
            "json",
        ],
        batch_input(Vec::new(), SourceInputSet::default(), 52),
    );

    assert_eq!(output.exit_code, CliExitCode::Success);
    assert_eq!(json_event_kinds(&output.stdout), ["session_finished"]);
    assert!(!output.stdout.contains("final_state"));
}

#[test]
fn real_argv0_path_is_accepted() {
    let output = run_batch(
        ["/usr/local/bin/mizar", "build", "--workspace", "workspace"],
        batch_input(Vec::new(), SourceInputSet::default(), 53),
    );

    assert_eq!(output.exit_code, CliExitCode::Success);
}

#[test]
fn manifest_path_requires_real_resolution_owner() {
    let output = run_batch(
        [
            "build",
            "--workspace",
            "workspace",
            "--manifest-path",
            "alpha/Mizar.toml",
        ],
        batch_input(Vec::new(), SourceInputSet::default(), 54),
    );

    assert_eq!(output.exit_code, CliExitCode::UnavailableOwner);
    assert!(output.stderr.contains("owner=manifest_path_resolution"));
}

#[test]
fn locked_requires_existing_lockfile_input() {
    let mut input = batch_input(Vec::new(), SourceInputSet::default(), 55);
    input.lockfile_existing = false;

    let output = run_batch(["build", "--locked"], input);

    assert_eq!(output.exit_code, CliExitCode::Usage);
    assert!(
        output
            .stderr
            .contains("locked build requires an existing lockfile")
    );
}

#[test]
fn unresolved_target_selection_exits_unavailable_before_submit() {
    let output = run_batch(
        [
            "build",
            "--workspace",
            "workspace",
            "--target",
            "alpha.main",
        ],
        batch_input(Vec::new(), SourceInputSet::default(), 56),
    );

    assert_eq!(output.exit_code, CliExitCode::UnavailableOwner);
    assert!(output.stderr.contains("owner=target_resolution"));
}

#[test]
fn nonmatching_package_selection_exits_unavailable_before_submit() {
    let output = run_batch(
        ["build", "--workspace", "workspace", "--package", "beta"],
        batch_input(Vec::new(), SourceInputSet::default(), 57),
    );

    assert_eq!(output.exit_code, CliExitCode::UnavailableOwner);
    assert!(output.stderr.contains("owner=package_selection"));
}

#[test]
fn source_layout_must_match_snapshot_inputs() {
    let source_inputs = SourceInputSet {
        versions: vec![source_version("src/other.miz", "other", 58)],
    };
    let output = run_batch(
        ["build", "--workspace", "workspace"],
        batch_input(
            vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
            source_inputs,
            59,
        ),
    );

    assert_eq!(output.exit_code, CliExitCode::UnavailableOwner);
    assert!(output.stderr.contains("owner=source_snapshot_inputs"));
}

#[test]
fn source_layout_module_must_match_snapshot_inputs() {
    let source_inputs = SourceInputSet {
        versions: vec![source_version("src/main.miz", "other", 60)],
    };
    let output = run_batch(
        ["build", "--workspace", "workspace"],
        batch_input(
            vec![WorkspaceSourceFile::new("src/main.miz", "main.miz")],
            source_inputs,
            61,
        ),
    );

    assert_eq!(output.exit_code, CliExitCode::UnavailableOwner);
    assert!(output.stderr.contains("owner=source_snapshot_inputs"));
}

#[test]
fn matching_package_selection_runs_single_package_input() {
    let output = run_batch(
        ["build", "--workspace", "workspace", "--package", "alpha"],
        batch_input(Vec::new(), SourceInputSet::default(), 62),
    );

    assert_eq!(output.exit_code, CliExitCode::Success);
}

#[test]
fn request_id_allocation_failure_maps_to_internal_error() {
    let snapshots = SnapshotRegistry::with_allocator(FailingAllocator);
    let mut driver = CompilerDriver::default();
    let output = run_batch_with_driver(
        ["build", "--workspace", "workspace"],
        batch_input(Vec::new(), SourceInputSet::default(), 63),
        &mut driver,
        &FailingAllocator,
        &snapshots,
    );

    assert_eq!(output.exit_code, CliExitCode::InternalError);
    assert_eq!(output.process_code(), 101);
    assert_eq!(
        output.stderr,
        "mizar build: internal_error kind=request_id_allocation\n"
    );
}

#[test]
fn request_id_allocation_failure_honors_json_message_format() {
    let snapshots = SnapshotRegistry::with_allocator(FailingAllocator);
    let mut driver = CompilerDriver::default();
    let output = run_batch_with_driver(
        [
            "build",
            "--workspace",
            "workspace",
            "--message-format",
            "json",
        ],
        batch_input(Vec::new(), SourceInputSet::default(), 64),
        &mut driver,
        &FailingAllocator,
        &snapshots,
    );

    assert_eq!(output.exit_code, CliExitCode::InternalError);
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout,
        "{\"schema_version\":1,\"kind\":\"internal_error\",\"error\":\"request_id_allocation\"}\n"
    );
}

#[test]
fn cli_source_keeps_non_owner_boundaries_out() {
    let source = fs::read_to_string(crate_root().join("src/cli.rs")).unwrap();
    let mut violations = Vec::new();

    for forbidden in [
        "PublicationToken",
        "serialize_artifact",
        "commit_manifest",
        "DocumentUri",
        "JsonRpc",
        "DiagnosticId",
        "TrustedStatus",
        "KernelAcceptance",
        "cache_compatibility",
        "SyntheticOutputRef",
        "fake_output",
        "mizar_artifact",
        "output_path",
        "committed_path",
        "ArtifactOwner",
        "artifact_owner",
        "artifact_serial",
        "ProofAcceptance",
        "proof_acceptance",
        "ProofPolicy",
        "CacheCompatibility",
        "phase_semantics",
        "SemanticChecker",
        "TypeChecker",
        "NameResolver",
        "VcGenerator",
    ] {
        if source.contains(forbidden) {
            violations.push(forbidden);
        }
    }

    assert!(
        violations.is_empty(),
        "CLI source must not claim non-owner authority: {violations:?}"
    );
}

fn batch_input(
    files: Vec<WorkspaceSourceFile>,
    source_inputs: SourceInputSet,
    seed: u8,
) -> CliBatchInput {
    CliBatchInput::new(
        r#"
        [package]
        name = "alpha"
        version = "0.1.0"
        "#,
        r#"
        schema_version = 1

        [[package]]
        name = "alpha"
        version = "0.1.0"
        source = { kind = "workspace", path = "alpha" }
        dependencies = []
        "#,
        "alpha",
        files,
        snapshot_inputs(source_inputs, seed),
    )
}

fn snapshot_inputs(source_inputs: SourceInputSet, seed: u8) -> CliSnapshotInputs {
    CliSnapshotInputs::new(
        source_inputs,
        mizar_driver::request::DependencyInputSet::new(
            vec![DependencyArtifactRef::new(
                "kernel/base.vo",
                hash(seed.wrapping_add(1)),
            )],
            hash(seed.wrapping_add(2)),
            ToolchainInfo::new("mizar-evo-cli-test"),
        ),
        mizar_driver::request::VerifierConfigInput::new(hash(seed.wrapping_add(3))),
    )
}

fn source_version(path: &str, module: &str, seed: u8) -> SourceVersion {
    let fixture = SourcePathFixture::new();
    fixture.write(path, "");
    SourceVersion {
        source_id: InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(seed))
            .unwrap(),
        package_id: PackageId::new("alpha"),
        module_path: ModulePath::new(module),
        normalized_path: normalized_path(fixture.root(), path),
        source_hash: hash(seed),
        edition: Edition::new("2026"),
        origin: SourceOrigin::Disk,
    }
}

fn hash(first_byte: u8) -> Hash {
    let mut bytes = [0; Hash::BYTE_LEN];
    bytes[0] = first_byte;
    Hash::from_bytes(bytes)
}

fn snapshot_id(seed: u8) -> BuildSnapshotId {
    let serialized = format!(
        "mizar-session-build-snapshot-v1:{}",
        format!("{seed:02x}").repeat(Hash::BYTE_LEN)
    );
    BuildSnapshotId::from_published_schema_str(&serialized).unwrap()
}

fn normalized_path(root: &Path, path: &str) -> NormalizedPath {
    normalize_source_path(root, Path::new(path)).unwrap()
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn human_event_kinds(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter_map(|line| line.strip_prefix("mizar build: "))
        .filter(|line| !line.starts_with("owner_gap "))
        .filter(|line| !line.starts_with("usage_error "))
        .filter(|line| !line.starts_with("internal_error "))
        .filter_map(|line| line.split_whitespace().next())
        .collect()
}

fn json_event_kinds(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter_map(|line| line.strip_prefix("{\"schema_version\":1,\"kind\":\""))
        .filter_map(|rest| rest.split_once('"').map(|(kind, _)| kind))
        .collect()
}

fn human_scheduler_events(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter_map(|line| line.split_once(" scheduler_event=").map(|(_, rest)| rest))
        .filter_map(|rest| rest.split_whitespace().next())
        .collect()
}

fn json_scheduler_events(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter_map(|line| {
            line.split_once("\"scheduler_event\":\"")
                .map(|(_, rest)| rest)
        })
        .filter_map(|rest| rest.split_once('"').map(|(event, _)| event))
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct FailingAllocator;

impl SessionIdAllocator for FailingAllocator {
    fn next_session_id(&self) -> Result<BuildSessionId, IdError> {
        Err(IdError::AllocatorOverflow)
    }

    fn next_request_id(&self) -> Result<BuildRequestId, IdError> {
        Err(IdError::AllocatorOverflow)
    }

    fn next_source_id(&self, _snapshot: BuildSnapshotId) -> Result<SourceId, IdError> {
        Err(IdError::AllocatorOverflow)
    }

    fn next_source_map_id(&self, _snapshot: BuildSnapshotId) -> Result<SourceMapId, IdError> {
        Err(IdError::AllocatorOverflow)
    }

    fn next_lease_id(&self, _snapshot: BuildSnapshotId) -> Result<SnapshotLeaseId, IdError> {
        Err(IdError::AllocatorOverflow)
    }
}

struct SourcePathFixture {
    base: PathBuf,
    root: PathBuf,
}

impl SourcePathFixture {
    fn new() -> Self {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "mizar_driver_cli_source_path_{}_{}",
            std::process::id(),
            id
        ));
        let root = base.join("package");
        fs::create_dir_all(root.join("src")).unwrap();
        Self { base, root }
    }

    fn root(&self) -> &Path {
        &self.root
    }

    fn write(&self, path: &str, contents: &str) {
        let path = self.root.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }
}

impl Drop for SourcePathFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}
