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
        DependencySelection, PlanRequest, WorkspacePackage, parse_lockfile, parse_package_manifest,
    },
    resource::ResourceBudget,
    scheduler::{CacheSchedulingPolicy, CompletionOrder, PriorityHints, SchedulerMode},
    task_graph::{ModuleDependencyOverlay, VcTaskDescriptor},
};
use mizar_session::{
    InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator, SnapshotRegistry,
    WorkspaceRoot,
};

use crate::{
    driver::{CompilerDriver, DriverSubmitInput},
    request::{
        BatchInvocation, BatchRequest, BuildLaneId, BuildProfile, BuildRequestDraft,
        BuildRequestGeneration, BuildRequestOrigin, BuildTargets, DependencyInputSet,
        SourceInputSet, VerifierConfigInput,
    },
};

mod output;

use output::{
    gap_output, output_from_submission, output_from_submit_error, owner_diagnostics_gap_output,
    usage_output,
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
        assert_eq!(
            output::json_escape("a\u{0001}\u{001f}b"),
            "a\\u0001\\u001fb"
        );
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
