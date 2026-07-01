use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use mizar_build::{
    cache_seam::CacheSchedulingPlan,
    cancel::CancellationPolicy,
    module_index::{
        DependencyArtifactIndex, StaticSourceLayout, WorkspaceSourceFile, WorkspaceSourcePackage,
    },
    planner::{
        DependencySelection, ManifestDiagnostics, PlanRequest, WorkspacePackage, parse_lockfile,
        parse_package_manifest,
    },
    resource::ResourceBudget,
    scheduler::{
        CacheSchedulingPolicy, CompletionOrder, PriorityHints, SchedulerEventKind, SchedulerMode,
        TaskState,
    },
    task_graph::{ModuleDependencyOverlay, PipelinePhase, VcTaskDescriptor},
};
use mizar_session::{
    InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator, SnapshotRegistry,
    WorkspaceRoot,
};

use crate::{
    driver::{
        BuildSubmission, CompilerDriver, DriverSubmissionStatus, DriverSubmitError,
        DriverSubmitInput,
    },
    events::{BuildEvent, BuildEventKind, BuildEventStream, EventOwner, OwnerGapClassification},
    registry::{PhaseServiceAvailability, PhaseStatus},
    request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildSession, BuildSessionOutcome,
        BuildSessionState, BuildTargets, DependencyInputSet, PublicationDecision, SourceInputSet,
        VerifierConfigInput,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliInvocation {
    pub raw_args: Vec<String>,
    pub command: CliCommand,
    pub workspace: String,
    pub manifest_path: Option<String>,
    pub packages: Vec<String>,
    pub profile: CliBuildProfile,
    pub targets: Vec<String>,
    pub jobs: usize,
    pub locked: bool,
    pub incremental: bool,
    pub message_format: CliMessageFormat,
    pub quiet: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliSnapshotInputs {
    pub source_inputs: SourceInputSet,
    pub dependency_inputs: DependencyInputSet,
    pub verifier_config: VerifierConfigInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliBatchInput {
    pub package_manifest: String,
    pub lockfile: String,
    pub lockfile_existing: bool,
    pub member_path: String,
    pub source_files: Vec<WorkspaceSourceFile>,
    pub snapshot_inputs: CliSnapshotInputs,
    pub dependency_artifacts: Vec<DependencyArtifactIndex>,
    pub dependency_overlay: ModuleDependencyOverlay,
    pub vc_descriptors: Vec<VcTaskDescriptor>,
    pub completion_order: CompletionOrder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOutput {
    pub exit_code: CliExitCode,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliUsageError {
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CliCommand {
    Build,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CliBuildProfile {
    Check,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CliMessageFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CliExitCode {
    Success,
    BuildFailed,
    Usage,
    UnavailableOwner,
    Cancelled,
    InternalError,
}

impl CliInvocation {
    pub fn parse<I, S>(args: I) -> Result<Self, CliUsageError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        let command_index = command_index(&args)?;
        let mut invocation = Self {
            raw_args: args.clone(),
            command: CliCommand::Build,
            workspace: ".".to_owned(),
            manifest_path: None,
            packages: Vec::new(),
            profile: CliBuildProfile::Check,
            targets: Vec::new(),
            jobs: 1,
            locked: false,
            incremental: true,
            message_format: CliMessageFormat::Human,
            quiet: false,
        };

        let mut index = command_index + 1;
        while index < args.len() {
            match args[index].as_str() {
                "--workspace" => {
                    let value = required_value(&args, index, "--workspace")?;
                    invocation.workspace = value.to_owned();
                    index += 2;
                }
                "--manifest-path" => {
                    let value = required_value(&args, index, "--manifest-path")?;
                    invocation.manifest_path = Some(value.to_owned());
                    index += 2;
                }
                "--package" => {
                    let value = required_value(&args, index, "--package")?;
                    invocation.packages.push(value.to_owned());
                    index += 2;
                }
                "--profile" => {
                    let value = required_value(&args, index, "--profile")?;
                    invocation.profile = CliBuildProfile::parse(value)?;
                    index += 2;
                }
                "--target" => {
                    let value = required_value(&args, index, "--target")?;
                    invocation.targets.push(value.to_owned());
                    index += 2;
                }
                "--jobs" => {
                    let value = required_value(&args, index, "--jobs")?;
                    invocation.jobs = parse_jobs(value)?;
                    index += 2;
                }
                "--locked" => {
                    invocation.locked = true;
                    index += 1;
                }
                "--no-incremental" => {
                    invocation.incremental = false;
                    index += 1;
                }
                "--message-format" => {
                    let value = required_value(&args, index, "--message-format")?;
                    invocation.message_format = CliMessageFormat::parse(value)?;
                    index += 2;
                }
                "--quiet" => {
                    invocation.quiet = true;
                    index += 1;
                }
                value if value.starts_with('-') => {
                    return Err(CliUsageError::new(format!("unknown flag {value}")));
                }
                value => {
                    return Err(CliUsageError::new(format!("unexpected argument {value}")));
                }
            }
        }

        if invocation.workspace.trim().is_empty() {
            return Err(CliUsageError::new("workspace path is empty"));
        }
        Ok(invocation)
    }

    pub fn request_draft(&self, snapshot_inputs: &CliSnapshotInputs) -> BuildRequestDraft {
        BuildRequestDraft {
            lane: BuildLaneId::new(0),
            origin: BuildRequestOrigin::Batch(BatchRequest {
                invocation: BatchInvocation {
                    args: self.raw_args.clone(),
                },
            }),
            generation: BuildRequestGeneration::new(0),
            workspace_root: WorkspaceRoot::new(self.workspace.clone()),
            profile: BuildProfile::new(self.profile.as_str()),
            targets: BuildTargets {
                packages: self.packages.iter().cloned().map(PackageId::new).collect(),
                modules: self.targets.iter().cloned().map(ModulePath::new).collect(),
            },
            source_inputs: snapshot_inputs.source_inputs.clone(),
            dependency_inputs: snapshot_inputs.dependency_inputs.clone(),
            verifier_config: snapshot_inputs.verifier_config,
        }
    }
}

impl CliSnapshotInputs {
    pub fn new(
        source_inputs: SourceInputSet,
        dependency_inputs: DependencyInputSet,
        verifier_config: VerifierConfigInput,
    ) -> Self {
        Self {
            source_inputs,
            dependency_inputs,
            verifier_config,
        }
    }
}

impl CliBatchInput {
    pub fn new(
        package_manifest: impl Into<String>,
        lockfile: impl Into<String>,
        member_path: impl Into<String>,
        source_files: Vec<WorkspaceSourceFile>,
        snapshot_inputs: CliSnapshotInputs,
    ) -> Self {
        Self {
            package_manifest: package_manifest.into(),
            lockfile: lockfile.into(),
            lockfile_existing: true,
            member_path: member_path.into(),
            source_files,
            snapshot_inputs,
            dependency_artifacts: Vec::new(),
            dependency_overlay: ModuleDependencyOverlay::complete(Vec::new()),
            vc_descriptors: Vec::new(),
            completion_order: CompletionOrder::Canonical,
        }
    }
}

impl CliOutput {
    pub fn process_code(&self) -> i32 {
        self.exit_code.process_code()
    }
}

impl CliUsageError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

impl CliBuildProfile {
    fn parse(value: &str) -> Result<Self, CliUsageError> {
        match value {
            "check" => Ok(Self::Check),
            "release" => Ok(Self::Release),
            _ => Err(CliUsageError::new(format!(
                "unsupported profile {value}; expected check or release"
            ))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Check => "check",
            Self::Release => "release",
        }
    }
}

impl CliMessageFormat {
    fn parse(value: &str) -> Result<Self, CliUsageError> {
        match value {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            _ => Err(CliUsageError::new(format!(
                "unsupported message format {value}; expected human or json"
            ))),
        }
    }
}

impl CliExitCode {
    pub const fn process_code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::BuildFailed => 1,
            Self::Usage => 2,
            Self::UnavailableOwner => 3,
            Self::Cancelled => 4,
            Self::InternalError => 101,
        }
    }
}

pub fn run_batch<I, S>(args: I, input: CliBatchInput) -> CliOutput
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let ids = InMemorySessionIdAllocator::new();
    let snapshots = SnapshotRegistry::new();
    let mut driver = CompilerDriver::default();
    run_batch_with_driver(args, input, &mut driver, &ids, &snapshots)
}

pub fn run_batch_with_driver<I, S, A>(
    args: I,
    input: CliBatchInput,
    driver: &mut CompilerDriver,
    ids: &A,
    snapshots: &SnapshotRegistry<A>,
) -> CliOutput
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
    A: SessionIdAllocator,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    let usage_format = requested_message_format(&args);
    let invocation = match CliInvocation::parse(args) {
        Ok(invocation) => invocation,
        Err(error) => return usage_output(error, usage_format),
    };
    run_invocation_with_driver(invocation, input, driver, ids, snapshots)
}

pub fn run_invocation_with_driver<A>(
    invocation: CliInvocation,
    input: CliBatchInput,
    driver: &mut CompilerDriver,
    ids: &A,
    snapshots: &SnapshotRegistry<A>,
) -> CliOutput
where
    A: SessionIdAllocator,
{
    let prepared = match prepare_driver_submission(&invocation, input) {
        Ok(prepared) => prepared,
        Err(output) => return output,
    };

    match driver.submit(prepared.request, ids, snapshots, prepared.submit_input) {
        Ok(submission) => output_from_submission(&invocation, driver, &submission),
        Err(error) => output_from_submit_error(&invocation, driver, error),
    }
}

struct PreparedCliSubmission {
    request: BuildRequestDraft,
    submit_input: DriverSubmitInput<StaticSourceLayout>,
}

fn prepare_driver_submission(
    invocation: &CliInvocation,
    input: CliBatchInput,
) -> Result<PreparedCliSubmission, CliOutput> {
    if invocation.manifest_path.is_some() {
        return Err(gap_output(
            invocation,
            CliExitCode::UnavailableOwner,
            "manifest_path_resolution",
            "external_dependency_gap",
        ));
    }

    if invocation.locked && !input.lockfile_existing {
        return Err(usage_output(
            CliUsageError::new("locked build requires an existing lockfile"),
            invocation.message_format,
        ));
    }

    let manifest = match parse_package_manifest(&input.package_manifest) {
        Ok(manifest) => manifest,
        Err(diagnostics) => {
            return Err(owner_diagnostics_gap_output(
                invocation,
                CliExitCode::BuildFailed,
                "manifest",
                diagnostics,
            ));
        }
    };

    if let Err(owner) = validate_target_selection(invocation, &manifest.name) {
        return Err(gap_output(
            invocation,
            CliExitCode::UnavailableOwner,
            owner,
            "external_dependency_gap",
        ));
    }
    if !source_layout_matches_snapshot(
        &manifest.package_id(),
        &input.source_files,
        &input.snapshot_inputs.source_inputs,
    ) {
        return Err(gap_output(
            invocation,
            CliExitCode::UnavailableOwner,
            "source_snapshot_inputs",
            "external_dependency_gap",
        ));
    }

    let lockfile = match parse_lockfile(&input.lockfile) {
        Ok(lockfile) => lockfile,
        Err(diagnostics) => {
            return Err(owner_diagnostics_gap_output(
                invocation,
                CliExitCode::BuildFailed,
                "lockfile",
                diagnostics,
            ));
        }
    };

    let package_id = manifest.package_id();
    let layout = StaticSourceLayout::new(vec![WorkspaceSourcePackage {
        package_id,
        files: input.source_files.clone(),
    }]);
    let mut submit_input = DriverSubmitInput::new(
        PlanRequest {
            workspace_root: WorkspaceRoot::new(invocation.workspace.clone()),
            dependency_selection: DependencySelection::Normal,
            toolchain: input.snapshot_inputs.dependency_inputs.toolchain.clone(),
        },
        vec![WorkspacePackage {
            member_path: input.member_path.clone(),
            manifest,
        }],
        lockfile,
        layout,
    );
    submit_input.dependency_artifacts = input.dependency_artifacts.clone();
    submit_input.dependency_overlay = input.dependency_overlay.clone();
    submit_input.vc_descriptors = input.vc_descriptors.clone();
    submit_input.scheduler_mode = SchedulerMode::Batch;
    submit_input.priority_hints = PriorityHints::default();
    submit_input.cache_policy = if invocation.incremental {
        CacheSchedulingPolicy::Unavailable
    } else {
        CacheSchedulingPolicy::Disabled
    };
    submit_input.cache_decisions = CacheSchedulingPlan::empty();
    submit_input.resource_budget = ResourceBudget::default();
    submit_input.cancellation = CancellationPolicy::default();
    submit_input.worker_count = invocation.jobs;
    submit_input.completion_order = input.completion_order;

    let request = invocation.request_draft(&input.snapshot_inputs);
    Ok(PreparedCliSubmission {
        request,
        submit_input,
    })
}

fn command_index(args: &[String]) -> Result<usize, CliUsageError> {
    match args {
        [] => Err(CliUsageError::new("missing command")),
        [command, ..] if command == "build" => Ok(0),
        [program, command, ..] if is_mizar_program(program) && command == "build" => Ok(1),
        [program, ..] if is_mizar_program(program) => Err(CliUsageError::new("missing command")),
        [command, ..] => Err(CliUsageError::new(format!(
            "unsupported command {command}; expected build"
        ))),
    }
}

fn requested_message_format(args: &[String]) -> CliMessageFormat {
    args.windows(2)
        .find_map(|window| {
            (window[0] == "--message-format" && window[1] == "json")
                .then_some(CliMessageFormat::Json)
        })
        .unwrap_or(CliMessageFormat::Human)
}

fn is_mizar_program(program: &str) -> bool {
    Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "mizar")
}

fn validate_target_selection<'a>(
    invocation: &CliInvocation,
    manifest_package: &'a str,
) -> Result<(), &'a str> {
    if !invocation.targets.is_empty() {
        return Err("target_resolution");
    }
    match invocation.packages.as_slice() {
        [] => Ok(()),
        [package] if package == manifest_package => Ok(()),
        _ => Err("package_selection"),
    }
}

fn source_layout_matches_snapshot(
    package: &PackageId,
    files: &[WorkspaceSourceFile],
    source_inputs: &SourceInputSet,
) -> bool {
    let layout_paths = files
        .iter()
        .map(|file| file.normalized_path.as_str())
        .collect::<BTreeSet<_>>();
    let snapshot_paths = source_inputs
        .versions
        .iter()
        .filter(|version| version.package_id == *package)
        .map(|version| version.normalized_path.as_str())
        .collect::<BTreeSet<_>>();
    let snapshot_modules_by_path = source_inputs
        .versions
        .iter()
        .filter(|version| version.package_id == *package)
        .map(|version| {
            (
                version.normalized_path.as_str(),
                version.module_path.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let snapshot_packages = source_inputs
        .versions
        .iter()
        .map(|version| version.package_id.as_str())
        .collect::<BTreeSet<_>>();

    if layout_paths != snapshot_paths
        || !snapshot_packages
            .iter()
            .all(|snapshot_package| *snapshot_package == package.as_str())
    {
        return false;
    }

    files.iter().all(|file| {
        let Some(layout_module) = module_path_from_source_relative(file) else {
            return true;
        };
        snapshot_modules_by_path
            .get(file.normalized_path.as_str())
            .is_some_and(|snapshot_module| *snapshot_module == layout_module.as_str())
    })
}

fn module_path_from_source_relative(file: &WorkspaceSourceFile) -> Option<String> {
    if file.normalized_path != format!("src/{}", file.source_relative_path) {
        return None;
    }
    let stem = file.source_relative_path.strip_suffix(".miz")?;
    if stem.is_empty() {
        return None;
    }
    Some(stem.split('/').collect::<Vec<_>>().join("."))
}

fn required_value<'a>(
    args: &'a [String],
    index: usize,
    flag: &'static str,
) -> Result<&'a str, CliUsageError> {
    let Some(value) = args.get(index + 1) else {
        return Err(CliUsageError::new(format!("missing value for {flag}")));
    };
    if value.starts_with('-') {
        return Err(CliUsageError::new(format!("missing value for {flag}")));
    }
    Ok(value)
}

fn parse_jobs(value: &str) -> Result<usize, CliUsageError> {
    let jobs = value
        .parse::<usize>()
        .map_err(|_| CliUsageError::new(format!("invalid jobs value {value}")))?;
    if jobs == 0 {
        return Err(CliUsageError::new("jobs must be at least 1"));
    }
    Ok(jobs)
}

fn usage_output(error: CliUsageError, format: CliMessageFormat) -> CliOutput {
    match format {
        CliMessageFormat::Human => CliOutput {
            exit_code: CliExitCode::Usage,
            stdout: String::new(),
            stderr: format!("mizar build: usage_error reason={}\n", error.reason),
        },
        CliMessageFormat::Json => CliOutput {
            exit_code: CliExitCode::Usage,
            stdout: format!(
                "{{\"schema_version\":1,\"kind\":\"usage_error\",\"reason\":\"{}\"}}\n",
                json_escape(&error.reason)
            ),
            stderr: String::new(),
        },
    }
}

fn owner_diagnostics_gap_output(
    invocation: &CliInvocation,
    exit_code: CliExitCode,
    owner: &'static str,
    diagnostics: ManifestDiagnostics,
) -> CliOutput {
    let count = diagnostics.diagnostics().len();
    gap_output_with_diagnostics(
        invocation,
        exit_code,
        owner,
        "external_dependency_gap",
        Some(count),
    )
}

fn gap_output(
    invocation: &CliInvocation,
    exit_code: CliExitCode,
    owner: &str,
    classification: &str,
) -> CliOutput {
    gap_output_with_diagnostics(invocation, exit_code, owner, classification, None)
}

fn gap_output_with_diagnostics(
    invocation: &CliInvocation,
    exit_code: CliExitCode,
    owner: &str,
    classification: &str,
    diagnostics: Option<usize>,
) -> CliOutput {
    match invocation.message_format {
        CliMessageFormat::Human => CliOutput {
            exit_code,
            stdout: String::new(),
            stderr: match diagnostics {
                Some(count) => format!(
                    "mizar build: owner_gap owner={owner} classification={classification} diagnostics={count}\n"
                ),
                None => format!(
                    "mizar build: owner_gap owner={owner} classification={classification}\n"
                ),
            },
        },
        CliMessageFormat::Json => CliOutput {
            exit_code,
            stdout: match diagnostics {
                Some(count) => format!(
                    "{{\"schema_version\":1,\"kind\":\"owner_gap\",\"owner\":\"{}\",\"classification\":\"{}\",\"diagnostics\":{count}}}\n",
                    json_escape(owner),
                    json_escape(classification)
                ),
                None => format!(
                    "{{\"schema_version\":1,\"kind\":\"owner_gap\",\"owner\":\"{}\",\"classification\":\"{}\"}}\n",
                    json_escape(owner),
                    json_escape(classification)
                ),
            },
            stderr: String::new(),
        },
    }
}

fn output_from_submission(
    invocation: &CliInvocation,
    driver: &CompilerDriver,
    submission: &BuildSubmission,
) -> CliOutput {
    let exit_code = exit_code_for_submission(submission);
    render_driver_output(
        invocation,
        exit_code,
        driver.events(submission.session.id),
        None,
    )
}

fn output_from_submit_error(
    invocation: &CliInvocation,
    driver: &CompilerDriver,
    error: DriverSubmitError,
) -> CliOutput {
    match error {
        DriverSubmitError::RequestIdAllocation { .. } => {
            internal_error_output(invocation, "request_id_allocation")
        }
        DriverSubmitError::SnapshotCapture { .. } => gap_output(
            invocation,
            CliExitCode::BuildFailed,
            "snapshot_capture",
            "external_dependency_gap",
        ),
        DriverSubmitError::Planning {
            session,
            diagnostics,
        } => render_driver_error_output(
            invocation,
            driver,
            &session,
            CliExitCode::BuildFailed,
            "planning",
            diagnostics.diagnostics().len(),
        ),
        DriverSubmitError::ModuleIndex {
            session,
            diagnostics,
        } => render_driver_error_output(
            invocation,
            driver,
            &session,
            CliExitCode::BuildFailed,
            "module_index",
            diagnostics.diagnostics().len(),
        ),
        DriverSubmitError::TaskGraph {
            session,
            diagnostics,
        } => render_driver_error_output(
            invocation,
            driver,
            &session,
            CliExitCode::BuildFailed,
            "task_graph",
            diagnostics.diagnostics().len(),
        ),
        DriverSubmitError::PhaseRegistry { session, .. } => render_driver_error_output(
            invocation,
            driver,
            &session,
            CliExitCode::InternalError,
            "phase_registry",
            1,
        ),
        DriverSubmitError::Scheduler {
            session,
            diagnostics,
        } => render_driver_error_output(
            invocation,
            driver,
            &session,
            CliExitCode::BuildFailed,
            "scheduler",
            diagnostics.diagnostics().len(),
        ),
    }
}

fn render_driver_error_output(
    invocation: &CliInvocation,
    driver: &CompilerDriver,
    session: &BuildSession,
    exit_code: CliExitCode,
    owner: &'static str,
    diagnostics_count: usize,
) -> CliOutput {
    render_driver_output(
        invocation,
        exit_code,
        driver.events(session.id),
        Some((owner, diagnostics_count)),
    )
}

fn internal_error_output(invocation: &CliInvocation, kind: &'static str) -> CliOutput {
    match invocation.message_format {
        CliMessageFormat::Human => CliOutput {
            exit_code: CliExitCode::InternalError,
            stdout: String::new(),
            stderr: format!("mizar build: internal_error kind={kind}\n"),
        },
        CliMessageFormat::Json => CliOutput {
            exit_code: CliExitCode::InternalError,
            stdout: format!(
                "{{\"schema_version\":1,\"kind\":\"internal_error\",\"error\":\"{kind}\"}}\n"
            ),
            stderr: String::new(),
        },
    }
}

fn render_driver_output(
    invocation: &CliInvocation,
    exit_code: CliExitCode,
    stream: BuildEventStream,
    diagnostics_gap: Option<(&'static str, usize)>,
) -> CliOutput {
    let mut stdout = String::new();
    let mut stderr = String::new();
    match invocation.message_format {
        CliMessageFormat::Human => {
            render_human_events(invocation, &stream, &mut stderr);
            if let Some((owner, count)) = diagnostics_gap {
                stderr.push_str(&format!(
                    "mizar build: owner_gap owner={owner} classification=external_dependency_gap diagnostics={count}\n"
                ));
            }
        }
        CliMessageFormat::Json => {
            render_json_events(invocation, &stream, &mut stdout);
            if let Some((owner, count)) = diagnostics_gap {
                stdout.push_str(&format!(
                    "{{\"schema_version\":1,\"kind\":\"owner_gap\",\"owner\":\"{owner}\",\"classification\":\"external_dependency_gap\",\"diagnostics\":{count}}}\n"
                ));
            }
        }
    }
    CliOutput {
        exit_code,
        stdout,
        stderr,
    }
}

fn render_human_events(invocation: &CliInvocation, stream: &BuildEventStream, output: &mut String) {
    for event in stream.events() {
        if invocation.quiet && !matches!(event.kind, BuildEventKind::SessionFinished { .. }) {
            continue;
        }
        output.push_str("mizar build: ");
        output.push_str(&human_event(event));
        output.push('\n');
    }
}

fn render_json_events(invocation: &CliInvocation, stream: &BuildEventStream, output: &mut String) {
    for event in stream.events() {
        if invocation.quiet && !matches!(event.kind, BuildEventKind::SessionFinished { .. }) {
            continue;
        }
        output.push_str(&json_event(event));
        output.push('\n');
    }
}

fn exit_code_for_submission(submission: &BuildSubmission) -> CliExitCode {
    match submission.status {
        DriverSubmissionStatus::BlockedByMissingPhaseServices
        | DriverSubmissionStatus::BlockedByPhaseDispatchGap => CliExitCode::UnavailableOwner,
        DriverSubmissionStatus::SupersededBeforeSubmission => CliExitCode::Cancelled,
        DriverSubmissionStatus::SchedulerValidated => match submission.session.state {
            BuildSessionState::Finished(BuildSessionOutcome::Succeeded) => CliExitCode::Success,
            BuildSessionState::Finished(
                BuildSessionOutcome::Cancelled | BuildSessionOutcome::Superseded,
            ) => CliExitCode::Cancelled,
            BuildSessionState::Finished(
                BuildSessionOutcome::Failed | BuildSessionOutcome::Blocked,
            ) => CliExitCode::BuildFailed,
            BuildSessionState::SnapshotCaptured
            | BuildSessionState::Submitted
            | BuildSessionState::Running
            | BuildSessionState::Cancelling => CliExitCode::InternalError,
        },
    }
}

fn human_event(event: &BuildEvent) -> String {
    match &event.kind {
        BuildEventKind::SessionAccepted => "session_accepted".to_owned(),
        BuildEventKind::SnapshotCaptured => "snapshot_captured".to_owned(),
        BuildEventKind::PlanningReady { status } => {
            format!("planning_ready status={}", planning_status_name(*status))
        }
        BuildEventKind::TaskProgress { task } => {
            let state = task.state.map(task_state_name).unwrap_or("none");
            format!(
                "task_progress task={} scheduler_event={} state={state}",
                task.task_id,
                scheduler_event_name(task.scheduler_event)
            )
        }
        BuildEventKind::PhaseServiceGap {
            phase,
            availability,
        } => format!(
            "phase_service_gap phase={} availability={}",
            phase_name(*phase),
            availability_name(*availability)
        ),
        BuildEventKind::DispatchGap { phases } => {
            format!("dispatch_gap phases={}", phase_list(phases))
        }
        BuildEventKind::OwnerReadinessGap {
            owner,
            classification,
        } => format!(
            "owner_gap owner={} classification={}",
            owner_name(*owner),
            owner_gap_name(*classification)
        ),
        BuildEventKind::PhaseReady { phase, status } => format!(
            "phase_ready phase={} status={}",
            phase_name(*phase),
            phase_status_name(*status)
        ),
        BuildEventKind::DiagnosticsReady { records } => format!(
            "diagnostics_ready owner={} identity={}",
            owner_name(records.owner),
            records.identity
        ),
        BuildEventKind::ArtifactBoundary { committed } => format!(
            "artifact_boundary owner={} identity={}",
            owner_name(committed.owner),
            committed.identity
        ),
        BuildEventKind::PublicationSuppressed => "publication_suppressed".to_owned(),
        BuildEventKind::SessionFinished { outcome } => {
            format!("session_finished outcome={}", outcome_name(*outcome))
        }
    }
}

fn json_event(event: &BuildEvent) -> String {
    let publication = publication_name(event.identity.publication);
    match &event.kind {
        BuildEventKind::SessionAccepted => {
            json_base_event("session_accepted", publication, String::new())
        }
        BuildEventKind::SnapshotCaptured => {
            json_base_event("snapshot_captured", publication, String::new())
        }
        BuildEventKind::PlanningReady { status } => json_base_event(
            "planning_ready",
            publication,
            format!(",\"status\":\"{}\"", planning_status_name(*status)),
        ),
        BuildEventKind::TaskProgress { task } => {
            let state = task.state.map(task_state_name).unwrap_or("none");
            json_base_event(
                "task_progress",
                publication,
                format!(
                    ",\"task\":\"{}\",\"scheduler_event\":\"{}\",\"state\":\"{}\"",
                    json_escape(&task.task_id),
                    scheduler_event_name(task.scheduler_event),
                    state
                ),
            )
        }
        BuildEventKind::PhaseServiceGap {
            phase,
            availability,
        } => json_base_event(
            "phase_service_gap",
            publication,
            format!(
                ",\"phase\":\"{}\",\"availability\":\"{}\"",
                phase_name(*phase),
                availability_name(*availability)
            ),
        ),
        BuildEventKind::DispatchGap { phases } => json_base_event(
            "dispatch_gap",
            publication,
            format!(",\"phases\":[{}]", json_phase_array(phases)),
        ),
        BuildEventKind::OwnerReadinessGap {
            owner,
            classification,
        } => json_base_event(
            "owner_gap",
            publication,
            format!(
                ",\"owner\":\"{}\",\"classification\":\"{}\"",
                owner_name(*owner),
                owner_gap_name(*classification)
            ),
        ),
        BuildEventKind::PhaseReady { phase, status } => json_base_event(
            "phase_ready",
            publication,
            format!(
                ",\"phase\":\"{}\",\"status\":\"{}\"",
                phase_name(*phase),
                phase_status_name(*status)
            ),
        ),
        BuildEventKind::DiagnosticsReady { records } => json_base_event(
            "diagnostics_ready",
            publication,
            format!(
                ",\"owner\":\"{}\",\"identity\":\"{}\"",
                owner_name(records.owner),
                json_escape(&records.identity)
            ),
        ),
        BuildEventKind::ArtifactBoundary { committed } => json_base_event(
            "artifact_boundary",
            publication,
            format!(
                ",\"owner\":\"{}\",\"identity\":\"{}\"",
                owner_name(committed.owner),
                json_escape(&committed.identity)
            ),
        ),
        BuildEventKind::PublicationSuppressed => {
            json_base_event("publication_suppressed", publication, String::new())
        }
        BuildEventKind::SessionFinished { outcome } => json_base_event(
            "session_finished",
            publication,
            format!(",\"outcome\":\"{}\"", outcome_name(*outcome)),
        ),
    }
}

fn json_base_event(kind: &'static str, publication: &'static str, fields: String) -> String {
    format!(
        "{{\"schema_version\":1,\"kind\":\"{kind}\",\"publication\":\"{publication}\"{fields}}}"
    )
}

fn phase_list(phases: &[PipelinePhase]) -> String {
    phases
        .iter()
        .copied()
        .map(phase_name)
        .collect::<Vec<_>>()
        .join(",")
}

fn json_phase_array(phases: &[PipelinePhase]) -> String {
    phases
        .iter()
        .copied()
        .map(|phase| format!("\"{}\"", phase_name(phase)))
        .collect::<Vec<_>>()
        .join(",")
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            other if other <= '\u{1f}' => push_json_control_escape(&mut escaped, other),
            other => escaped.push(other),
        }
    }
    escaped
}

fn push_json_control_escape(output: &mut String, ch: char) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let value = ch as usize;
    output.push_str("\\u00");
    output.push(HEX[(value >> 4) & 0x0f] as char);
    output.push(HEX[value & 0x0f] as char);
}

fn planning_status_name(status: crate::events::PlanningEventStatus) -> &'static str {
    match status {
        crate::events::PlanningEventStatus::Ready => "ready",
        crate::events::PlanningEventStatus::Failed => "failed",
    }
}

fn task_state_name(state: TaskState) -> &'static str {
    match state {
        TaskState::Pending => "pending",
        TaskState::Ready => "ready",
        TaskState::Running => "running",
        TaskState::Completed => "completed",
        TaskState::CacheHit => "cache_hit",
        TaskState::Skipped => "skipped",
        TaskState::Failed => "failed",
        TaskState::Blocked => "blocked",
        TaskState::Cancelled => "cancelled",
        _ => "unknown",
    }
}

fn scheduler_event_name(kind: SchedulerEventKind) -> &'static str {
    match kind {
        SchedulerEventKind::TaskBecameReady => "task_became_ready",
        SchedulerEventKind::TaskStarted => "task_started",
        SchedulerEventKind::TaskSkipped => "task_skipped",
        SchedulerEventKind::TaskBlocked => "task_blocked",
        SchedulerEventKind::TaskFinished => "task_finished",
        SchedulerEventKind::RunFinished => "run_finished",
        _ => "unknown",
    }
}

fn phase_name(phase: PipelinePhase) -> &'static str {
    match phase {
        PipelinePhase::PackageResolve => "package_resolve",
        PipelinePhase::SourceLoad => "source_load",
        PipelinePhase::Frontend => "frontend",
        PipelinePhase::ModuleResolve => "module_resolve",
        PipelinePhase::SignatureCollection => "signature_collection",
        PipelinePhase::TypeChecking => "type_checking",
        PipelinePhase::RegistrationResolution => "registration_resolution",
        PipelinePhase::OverloadResolution => "overload_resolution",
        PipelinePhase::Elaboration => "elaboration",
        PipelinePhase::AlgorithmPreparation => "algorithm_preparation",
        PipelinePhase::VcGenerate => "vc_generate",
        PipelinePhase::VcDischarge => "vc_discharge",
        PipelinePhase::AtpSolve => "atp_solve",
        PipelinePhase::BackendRun => "backend_run",
        PipelinePhase::KernelCheck => "kernel_check",
        PipelinePhase::ArtifactCommit => "artifact_commit",
        PipelinePhase::DocumentationExtract => "documentation_extract",
        _ => "unknown",
    }
}

fn availability_name(availability: PhaseServiceAvailability) -> &'static str {
    match availability {
        PhaseServiceAvailability::AvailableOwner => "available_owner",
        PhaseServiceAvailability::ExternalDependencyGap => "external_dependency_gap",
        PhaseServiceAvailability::Deferred => "deferred",
    }
}

fn phase_status_name(status: PhaseStatus) -> &'static str {
    match status {
        PhaseStatus::Complete => "complete",
        PhaseStatus::Recoverable => "recoverable",
        PhaseStatus::Blocking => "blocking",
        PhaseStatus::Fatal => "fatal",
        PhaseStatus::Cancelled => "cancelled",
    }
}

fn owner_name(owner: EventOwner) -> &'static str {
    match owner {
        EventOwner::Diagnostics => "diagnostics",
        EventOwner::Artifact => "artifact",
        EventOwner::ProducerOutput => "producer_output",
        EventOwner::LspBridge => "lsp_bridge",
    }
}

fn owner_gap_name(classification: OwnerGapClassification) -> &'static str {
    match classification {
        OwnerGapClassification::ExternalDependencyGap => "external_dependency_gap",
        OwnerGapClassification::Deferred => "deferred",
        OwnerGapClassification::Unavailable => "unavailable",
    }
}

fn outcome_name(outcome: BuildSessionOutcome) -> &'static str {
    match outcome {
        BuildSessionOutcome::Succeeded => "succeeded",
        BuildSessionOutcome::Failed => "failed",
        BuildSessionOutcome::Blocked => "blocked",
        BuildSessionOutcome::Cancelled => "cancelled",
        BuildSessionOutcome::Superseded => "superseded",
    }
}

fn publication_name(publication: PublicationDecision) -> &'static str {
    match publication {
        PublicationDecision::Current => "current",
        PublicationDecision::Suppressed(_) => "suppressed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_session::{DependencyArtifactRef, Hash, ToolchainInfo};

    #[test]
    fn invocation_controls_are_forwarded_to_driver_submit_input() {
        let invocation = CliInvocation::parse([
            "build",
            "--workspace",
            "workspace",
            "--jobs",
            "4",
            "--no-incremental",
        ])
        .unwrap();
        let prepared = prepare_driver_submission(&invocation, batch_input(9))
            .expect("fixture input prepares driver submission");

        assert_eq!(prepared.submit_input.scheduler_mode, SchedulerMode::Batch);
        assert_eq!(prepared.submit_input.worker_count, 4);
        assert_eq!(
            prepared.submit_input.cache_policy,
            CacheSchedulingPolicy::Disabled
        );
        assert_eq!(
            prepared.request.workspace_root,
            WorkspaceRoot::new("workspace")
        );
        assert_eq!(prepared.request.profile.name, "check");
    }

    #[test]
    fn json_escape_covers_all_control_characters() {
        assert_eq!(json_escape("a\u{0001}\u{001f}b"), "a\\u0001\\u001fb");
    }

    fn batch_input(seed: u8) -> CliBatchInput {
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
            Vec::new(),
            CliSnapshotInputs::new(
                SourceInputSet::default(),
                DependencyInputSet::new(
                    vec![DependencyArtifactRef::new(
                        "kernel/base.vo",
                        hash(seed.wrapping_add(1)),
                    )],
                    hash(seed.wrapping_add(2)),
                    ToolchainInfo::new("mizar-evo-cli-test"),
                ),
                VerifierConfigInput::new(hash(seed.wrapping_add(3))),
            ),
        )
    }

    fn hash(first_byte: u8) -> Hash {
        let mut bytes = [0; Hash::BYTE_LEN];
        bytes[0] = first_byte;
        Hash::from_bytes(bytes)
    }
}
