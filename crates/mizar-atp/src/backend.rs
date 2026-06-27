//! Generic backend child-process runner and mock classification boundary.
//!
//! This module implements the task-14 runner specified in
//! [backend.md](../../../doc/design/mizar-atp/en/backend.md). It launches
//! untrusted backend processes, records deterministic run metadata, and offers
//! a mock classification seam for candidate evidence. It does not parse real
//! backend proof languages, extract real candidate evidence, call the kernel,
//! check SAT, accept proofs, publish witnesses, or trust backend proof
//! objects, logs, unsat cores, traces, or used-axiom reports.

use crate::problem::{AtpProblemId, AtpTargetBinding, ConcreteFormat, ExpectedBackendResult};
use mizar_session::Hash;
use std::{
    error::Error,
    ffi::OsStr,
    fmt::{self, Write as _},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};

const INPUT_HASH_DOMAIN: &str = "mizar-atp/backend-input/v1";
const COMMAND_HASH_DOMAIN: &str = "mizar-atp/backend-command/v1";
const STREAM_HASH_DOMAIN: &str = "mizar-atp/backend-stream/v1";
const METADATA_HASH_DOMAIN: &str = "mizar-atp/backend-metadata/v1";
const TEMP_COUNTER_DOMAIN: &str = "mizar-atp/backend-temp/v1";
const DEFAULT_CAPTURE_LIMIT: usize = 64 * 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_KILL_GRACE: Duration = Duration::from_millis(500);

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRunId(String);

impl BackendRunId {
    pub fn new(value: impl Into<String>) -> Result<Self, BackendConfigError> {
        let value = value.into();
        reject_empty("run_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BackendProfileId(String);

impl BackendProfileId {
    pub fn new(value: impl Into<String>) -> Result<Self, BackendConfigError> {
        let value = value.into();
        reject_empty("profile_id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BackendKind(String);

impl BackendKind {
    pub fn new(value: impl Into<String>) -> Result<Self, BackendConfigError> {
        let value = value.into();
        reject_empty("backend_kind", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedBackendProblem {
    problem_id: AtpProblemId,
    target_binding: AtpTargetBinding,
    expected_result: ExpectedBackendResult,
    concrete_format: ConcreteFormat,
    logic_profile_name: String,
    logic_fragment: String,
    input_text: Vec<u8>,
    input_hash: Hash,
    formula_labels: Vec<String>,
    symbol_bindings: Vec<String>,
    provenance_hash: Hash,
    metadata_hash: Hash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedBackendProblemParts {
    pub problem_id: AtpProblemId,
    pub target_binding: AtpTargetBinding,
    pub expected_result: ExpectedBackendResult,
    pub concrete_format: ConcreteFormat,
    pub logic_profile_name: String,
    pub logic_fragment: String,
    pub input_text: Vec<u8>,
    pub formula_labels: Vec<String>,
    pub symbol_bindings: Vec<String>,
    pub provenance_hash: Hash,
}

impl EncodedBackendProblem {
    pub fn new(parts: EncodedBackendProblemParts) -> Result<Self, BackendConfigError> {
        reject_empty("logic_profile_name", &parts.logic_profile_name)?;
        reject_empty("logic_fragment", &parts.logic_fragment)?;
        if parts.input_text.is_empty() {
            return Err(BackendConfigError::EmptyField {
                field: "input_text",
            });
        }
        let formula_labels = sorted_unique("formula_labels", parts.formula_labels)?;
        let symbol_bindings = sorted_unique("symbol_bindings", parts.symbol_bindings)?;
        let input_hash = hash_input(
            parts.concrete_format,
            &parts.logic_profile_name,
            &parts.logic_fragment,
            &parts.input_text,
        );
        let metadata_hash = hash_metadata(
            &parts.target_binding,
            &formula_labels,
            &symbol_bindings,
            parts.provenance_hash,
        );

        Ok(Self {
            problem_id: parts.problem_id,
            target_binding: parts.target_binding,
            expected_result: parts.expected_result,
            concrete_format: parts.concrete_format,
            logic_profile_name: parts.logic_profile_name,
            logic_fragment: parts.logic_fragment,
            input_text: parts.input_text,
            input_hash,
            formula_labels,
            symbol_bindings,
            provenance_hash: parts.provenance_hash,
            metadata_hash,
        })
    }

    pub const fn problem_id(&self) -> AtpProblemId {
        self.problem_id
    }

    pub const fn target_binding(&self) -> &AtpTargetBinding {
        &self.target_binding
    }

    pub const fn expected_result(&self) -> ExpectedBackendResult {
        self.expected_result
    }

    pub const fn concrete_format(&self) -> ConcreteFormat {
        self.concrete_format
    }

    pub fn logic_profile_name(&self) -> &str {
        &self.logic_profile_name
    }

    pub fn logic_fragment(&self) -> &str {
        &self.logic_fragment
    }

    pub fn input_text(&self) -> &[u8] {
        &self.input_text
    }

    pub const fn input_hash(&self) -> Hash {
        self.input_hash
    }

    pub fn formula_labels(&self) -> &[String] {
        &self.formula_labels
    }

    pub fn symbol_bindings(&self) -> &[String] {
        &self.symbol_bindings
    }

    pub const fn provenance_hash(&self) -> Hash {
        self.provenance_hash
    }

    pub const fn metadata_hash(&self) -> Hash {
        self.metadata_hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendProfile {
    profile_id: BackendProfileId,
    backend_kind: BackendKind,
    concrete_format: ConcreteFormat,
    version_probe: Option<BackendVersionProbe>,
    requires_candidate_evidence: bool,
    deterministic_priority: u32,
}

impl BackendProfile {
    pub fn new(
        profile_id: BackendProfileId,
        backend_kind: BackendKind,
        concrete_format: ConcreteFormat,
    ) -> Self {
        Self {
            profile_id,
            backend_kind,
            concrete_format,
            version_probe: None,
            requires_candidate_evidence: true,
            deterministic_priority: 0,
        }
    }

    pub fn with_version_probe(mut self, version_probe: BackendVersionProbe) -> Self {
        self.version_probe = Some(version_probe);
        self
    }

    pub const fn with_deterministic_priority(mut self, priority: u32) -> Self {
        self.deterministic_priority = priority;
        self
    }

    pub const fn profile_id(&self) -> &BackendProfileId {
        &self.profile_id
    }

    pub const fn backend_kind(&self) -> &BackendKind {
        &self.backend_kind
    }

    pub const fn concrete_format(&self) -> ConcreteFormat {
        self.concrete_format
    }

    pub const fn version_probe(&self) -> Option<&BackendVersionProbe> {
        self.version_probe.as_ref()
    }

    pub const fn requires_candidate_evidence(&self) -> bool {
        self.requires_candidate_evidence
    }

    pub const fn deterministic_priority(&self) -> u32 {
        self.deterministic_priority
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendVersionProbe {
    executable: PathBuf,
    args: Vec<String>,
    timeout: Duration,
}

impl BackendVersionProbe {
    pub fn new(
        executable: impl Into<PathBuf>,
        args: Vec<String>,
        timeout: Duration,
    ) -> Result<Self, BackendConfigError> {
        let executable = executable.into();
        reject_empty_path("version_probe.executable", &executable)?;
        Ok(Self {
            executable,
            args,
            timeout,
        })
    }

    pub fn executable(&self) -> &Path {
        self.executable.as_path()
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub const fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendCommand {
    executable: PathBuf,
    semantic_executable_id: Option<String>,
    args: Vec<String>,
    environment: BackendEnvironmentPolicy,
    working_directory: BackendWorkingDirectoryPolicy,
}

impl BackendCommand {
    pub fn new(
        executable: impl Into<PathBuf>,
        args: Vec<String>,
    ) -> Result<Self, BackendConfigError> {
        let executable = executable.into();
        reject_empty_path("command.executable", &executable)?;
        Ok(Self {
            executable,
            semantic_executable_id: None,
            args,
            environment: BackendEnvironmentPolicy::new(Vec::new())?,
            working_directory: BackendWorkingDirectoryPolicy::Inherit,
        })
    }

    pub fn with_semantic_executable_id(
        mut self,
        semantic_executable_id: impl Into<String>,
    ) -> Result<Self, BackendConfigError> {
        let semantic_executable_id = semantic_executable_id.into();
        reject_empty("semantic_executable_id", &semantic_executable_id)?;
        self.semantic_executable_id = Some(semantic_executable_id);
        Ok(self)
    }

    pub fn with_environment(mut self, environment: BackendEnvironmentPolicy) -> Self {
        self.environment = environment;
        self
    }

    pub fn with_working_directory(
        mut self,
        working_directory: BackendWorkingDirectoryPolicy,
    ) -> Self {
        self.working_directory = working_directory;
        self
    }

    pub fn executable(&self) -> &Path {
        self.executable.as_path()
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub const fn environment(&self) -> &BackendEnvironmentPolicy {
        &self.environment
    }

    pub const fn working_directory(&self) -> &BackendWorkingDirectoryPolicy {
        &self.working_directory
    }

    pub fn semantic_executable_id(&self) -> String {
        self.semantic_executable_id
            .clone()
            .unwrap_or_else(|| semantic_path_id(&self.executable))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendEnvironmentPolicy {
    vars: Vec<(String, String)>,
}

impl BackendEnvironmentPolicy {
    pub fn new(vars: Vec<(String, String)>) -> Result<Self, BackendConfigError> {
        let mut vars = vars;
        for (key, _) in &vars {
            reject_empty("environment.key", key)?;
        }
        vars.sort_by(|left, right| left.0.cmp(&right.0));
        for pair in vars.windows(2) {
            if pair[0].0 == pair[1].0 {
                return Err(BackendConfigError::DuplicateEnvironmentKey {
                    key: pair[0].0.clone(),
                });
            }
        }
        Ok(Self { vars })
    }

    pub fn vars(&self) -> &[(String, String)] {
        &self.vars
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendWorkingDirectoryPolicy {
    Inherit,
    Directory(PathBuf),
}

impl BackendWorkingDirectoryPolicy {
    fn hash_kind(&self) -> &'static str {
        match self {
            Self::Inherit => "inherit",
            Self::Directory(_) => "explicit-directory",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendIoMode {
    Stdin,
    PrivateProblemFile,
}

impl BackendIoMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Stdin => "stdin",
            Self::PrivateProblemFile => "private-problem-file",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendLimitRequirement {
    BestEffort,
    Required,
}

impl BackendLimitRequirement {
    const fn as_str(self) -> &'static str {
        match self {
            Self::BestEffort => "best-effort",
            Self::Required => "required",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendResourceLimits {
    wall_timeout: Duration,
    kill_grace: Duration,
    stdout_bytes: usize,
    stderr_bytes: usize,
    platform_limits: Vec<(String, BackendLimitRequirement)>,
}

impl BackendResourceLimits {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn with_wall_timeout(mut self, timeout: Duration) -> Self {
        self.wall_timeout = timeout;
        self
    }

    pub const fn with_kill_grace(mut self, kill_grace: Duration) -> Self {
        self.kill_grace = kill_grace;
        self
    }

    pub const fn with_stdout_limit(mut self, bytes: usize) -> Self {
        self.stdout_bytes = bytes;
        self
    }

    pub const fn with_stderr_limit(mut self, bytes: usize) -> Self {
        self.stderr_bytes = bytes;
        self
    }

    pub fn with_unsupported_platform_limit(
        mut self,
        name: impl Into<String>,
        requirement: BackendLimitRequirement,
    ) -> Result<Self, BackendConfigError> {
        let name = name.into();
        reject_empty("platform_limit.name", &name)?;
        self.platform_limits.push((name, requirement));
        self.platform_limits
            .sort_by(|left, right| left.0.cmp(&right.0));
        Ok(self)
    }

    pub const fn wall_timeout(&self) -> Duration {
        self.wall_timeout
    }

    pub const fn kill_grace(&self) -> Duration {
        self.kill_grace
    }

    pub const fn stdout_bytes(&self) -> usize {
        self.stdout_bytes
    }

    pub const fn stderr_bytes(&self) -> usize {
        self.stderr_bytes
    }

    pub fn platform_limits(&self) -> &[(String, BackendLimitRequirement)] {
        &self.platform_limits
    }

    fn has_required_unsupported_limits(&self) -> bool {
        self.platform_limits
            .iter()
            .any(|(_, requirement)| *requirement == BackendLimitRequirement::Required)
    }

    fn unsupported_limit_diagnostics(&self) -> Vec<BackendDiagnostic> {
        self.platform_limits
            .iter()
            .map(|(name, requirement)| {
                BackendDiagnostic::new(
                    "unsupported_resource_limit",
                    format!(
                        "platform resource limit `{name}` is unsupported ({})",
                        requirement.as_str()
                    ),
                )
            })
            .collect()
    }
}

impl Default for BackendResourceLimits {
    fn default() -> Self {
        Self {
            wall_timeout: DEFAULT_TIMEOUT,
            kill_grace: DEFAULT_KILL_GRACE,
            stdout_bytes: DEFAULT_CAPTURE_LIMIT,
            stderr_bytes: DEFAULT_CAPTURE_LIMIT,
            platform_limits: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl BackendCancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for BackendCancellationToken {
    fn default() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackendRunInput {
    run_id: BackendRunId,
    encoded_problem: EncodedBackendProblem,
    profile: BackendProfile,
    command: BackendCommand,
    resource_limits: BackendResourceLimits,
    io_mode: BackendIoMode,
    cancellation: BackendCancellationToken,
    random_seed: Option<u64>,
}

impl BackendRunInput {
    pub fn new(
        run_id: BackendRunId,
        encoded_problem: EncodedBackendProblem,
        profile: BackendProfile,
        command: BackendCommand,
        resource_limits: BackendResourceLimits,
        io_mode: BackendIoMode,
        cancellation: BackendCancellationToken,
    ) -> Self {
        Self {
            run_id,
            encoded_problem,
            profile,
            command,
            resource_limits,
            io_mode,
            cancellation,
            random_seed: None,
        }
    }

    pub const fn with_random_seed(mut self, random_seed: u64) -> Self {
        self.random_seed = Some(random_seed);
        self
    }

    pub const fn run_id(&self) -> &BackendRunId {
        &self.run_id
    }

    pub const fn encoded_problem(&self) -> &EncodedBackendProblem {
        &self.encoded_problem
    }

    pub const fn profile(&self) -> &BackendProfile {
        &self.profile
    }

    pub const fn command(&self) -> &BackendCommand {
        &self.command
    }

    pub const fn resource_limits(&self) -> &BackendResourceLimits {
        &self.resource_limits
    }

    pub const fn io_mode(&self) -> BackendIoMode {
        self.io_mode
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BackendRunMetadataConfig {
    semantic_executable_id: String,
    args: Vec<String>,
    environment: Vec<(String, String)>,
    working_directory_policy_kind: &'static str,
    io_mode: BackendIoMode,
    random_seed: Option<u64>,
    resource_limits: BackendResourceLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRunResult {
    run_id: BackendRunId,
    encoded_problem: EncodedBackendProblem,
    backend_kind: BackendKind,
    profile_id: BackendProfileId,
    command_fingerprint: Hash,
    metadata_config: BackendRunMetadataConfig,
    version_record: Option<BackendVersionRecord>,
    status: BackendRunStatus,
    observed_result: Option<BackendObservedResult>,
    candidate_evidence: Option<BackendCandidateEvidence>,
    counterexample: Option<BackendCounterexample>,
    stdout: BackendStreamCapture,
    stderr: BackendStreamCapture,
    exit_status: Option<BackendExitStatus>,
    termination: BackendTermination,
    elapsed: Duration,
    child_reaped: bool,
    diagnostics: Vec<BackendDiagnostic>,
}

impl BackendRunResult {
    pub const fn run_id(&self) -> &BackendRunId {
        &self.run_id
    }

    pub const fn encoded_problem(&self) -> &EncodedBackendProblem {
        &self.encoded_problem
    }

    pub const fn backend_kind(&self) -> &BackendKind {
        &self.backend_kind
    }

    pub const fn profile_id(&self) -> &BackendProfileId {
        &self.profile_id
    }

    pub const fn command_fingerprint(&self) -> Hash {
        self.command_fingerprint
    }

    pub const fn version_record(&self) -> Option<&BackendVersionRecord> {
        self.version_record.as_ref()
    }

    pub const fn status(&self) -> BackendRunStatus {
        self.status
    }

    pub const fn observed_result(&self) -> Option<BackendObservedResult> {
        self.observed_result
    }

    pub const fn candidate_evidence(&self) -> Option<&BackendCandidateEvidence> {
        self.candidate_evidence.as_ref()
    }

    pub const fn counterexample(&self) -> Option<&BackendCounterexample> {
        self.counterexample.as_ref()
    }

    pub const fn stdout(&self) -> &BackendStreamCapture {
        &self.stdout
    }

    pub const fn stderr(&self) -> &BackendStreamCapture {
        &self.stderr
    }

    pub const fn exit_status(&self) -> Option<BackendExitStatus> {
        self.exit_status
    }

    pub const fn termination(&self) -> BackendTermination {
        self.termination
    }

    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub const fn child_reaped(&self) -> bool {
        self.child_reaped
    }

    pub fn diagnostics(&self) -> &[BackendDiagnostic] {
        &self.diagnostics
    }

    pub fn metadata(&self) -> BackendRunMetadata {
        BackendRunMetadata {
            run_id: self.run_id.clone(),
            problem_id: self.encoded_problem.problem_id(),
            backend_kind: self.backend_kind.clone(),
            profile_id: self.profile_id.clone(),
            concrete_format: self.encoded_problem.concrete_format(),
            encoded_input_hash: self.encoded_problem.input_hash(),
            encoded_metadata_hash: self.encoded_problem.metadata_hash(),
            command_fingerprint: self.command_fingerprint,
            semantic_executable_id: self.metadata_config.semantic_executable_id.clone(),
            args: self.metadata_config.args.clone(),
            environment: self.metadata_config.environment.clone(),
            working_directory_policy_kind: self.metadata_config.working_directory_policy_kind,
            io_mode: self.metadata_config.io_mode,
            random_seed: self.metadata_config.random_seed,
            resource_limits: self.metadata_config.resource_limits.clone(),
            version_record: self.version_record.clone(),
            status: self.status,
            observed_result: self.observed_result,
            termination: self.termination,
            exit_status: self.exit_status,
            child_reaped: self.child_reaped,
            elapsed: self.elapsed,
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
            diagnostics: self.diagnostics.clone(),
        }
    }

    fn with_diagnostic(&mut self, key: &'static str, message: impl Into<String>) {
        self.diagnostics.push(BackendDiagnostic::new(key, message));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRunMetadata {
    run_id: BackendRunId,
    problem_id: AtpProblemId,
    backend_kind: BackendKind,
    profile_id: BackendProfileId,
    concrete_format: ConcreteFormat,
    encoded_input_hash: Hash,
    encoded_metadata_hash: Hash,
    command_fingerprint: Hash,
    semantic_executable_id: String,
    args: Vec<String>,
    environment: Vec<(String, String)>,
    working_directory_policy_kind: &'static str,
    io_mode: BackendIoMode,
    random_seed: Option<u64>,
    resource_limits: BackendResourceLimits,
    version_record: Option<BackendVersionRecord>,
    status: BackendRunStatus,
    observed_result: Option<BackendObservedResult>,
    termination: BackendTermination,
    exit_status: Option<BackendExitStatus>,
    child_reaped: bool,
    elapsed: Duration,
    stdout: BackendStreamCapture,
    stderr: BackendStreamCapture,
    diagnostics: Vec<BackendDiagnostic>,
}

impl BackendRunMetadata {
    pub const fn run_id(&self) -> &BackendRunId {
        &self.run_id
    }

    pub const fn problem_id(&self) -> AtpProblemId {
        self.problem_id
    }

    pub const fn backend_kind(&self) -> &BackendKind {
        &self.backend_kind
    }

    pub const fn profile_id(&self) -> &BackendProfileId {
        &self.profile_id
    }

    pub const fn concrete_format(&self) -> ConcreteFormat {
        self.concrete_format
    }

    pub const fn encoded_input_hash(&self) -> Hash {
        self.encoded_input_hash
    }

    pub const fn encoded_metadata_hash(&self) -> Hash {
        self.encoded_metadata_hash
    }

    pub const fn command_fingerprint(&self) -> Hash {
        self.command_fingerprint
    }

    pub fn semantic_executable_id(&self) -> &str {
        &self.semantic_executable_id
    }

    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn environment(&self) -> &[(String, String)] {
        &self.environment
    }

    pub const fn working_directory_policy_kind(&self) -> &'static str {
        self.working_directory_policy_kind
    }

    pub const fn io_mode(&self) -> BackendIoMode {
        self.io_mode
    }

    pub const fn random_seed(&self) -> Option<u64> {
        self.random_seed
    }

    pub const fn resource_limits(&self) -> &BackendResourceLimits {
        &self.resource_limits
    }

    pub const fn version_record(&self) -> Option<&BackendVersionRecord> {
        self.version_record.as_ref()
    }

    pub const fn status(&self) -> BackendRunStatus {
        self.status
    }

    pub const fn observed_result(&self) -> Option<BackendObservedResult> {
        self.observed_result
    }

    pub const fn termination(&self) -> BackendTermination {
        self.termination
    }

    pub const fn exit_status(&self) -> Option<BackendExitStatus> {
        self.exit_status
    }

    pub const fn child_reaped(&self) -> bool {
        self.child_reaped
    }

    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub const fn stdout(&self) -> &BackendStreamCapture {
        &self.stdout
    }

    pub const fn stderr(&self) -> &BackendStreamCapture {
        &self.stderr
    }

    pub fn diagnostics(&self) -> &[BackendDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendRunStatus {
    Proved,
    Counterexample,
    Timeout,
    Unknown,
    Error,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendObservedResult {
    Unsat,
    Sat,
    CounterSatisfiable,
    Unknown,
    Malformed,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendTermination {
    NotStarted,
    Exited,
    Timeout,
    Cancelled,
    SpawnFailure,
    ProcessError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendExitStatus {
    code: Option<i32>,
    success: bool,
}

impl BackendExitStatus {
    pub const fn code(self) -> Option<i32> {
        self.code
    }

    pub const fn success(self) -> bool {
        self.success
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendStreamCapture {
    retained: Vec<u8>,
    hash: Hash,
    total_bytes: u64,
    truncated: bool,
    incomplete: bool,
}

impl BackendStreamCapture {
    pub fn retained_bytes(&self) -> &[u8] {
        &self.retained
    }

    pub const fn hash(&self) -> Hash {
        self.hash
    }

    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    pub const fn incomplete(&self) -> bool {
        self.incomplete
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendVersionRecord {
    success: bool,
    stdout: BackendStreamCapture,
    stderr: BackendStreamCapture,
    exit_status: Option<BackendExitStatus>,
    parsed_version: Option<String>,
    diagnostics: Vec<BackendDiagnostic>,
}

impl BackendVersionRecord {
    pub const fn success(&self) -> bool {
        self.success
    }

    pub const fn stdout(&self) -> &BackendStreamCapture {
        &self.stdout
    }

    pub const fn stderr(&self) -> &BackendStreamCapture {
        &self.stderr
    }

    pub const fn exit_status(&self) -> Option<BackendExitStatus> {
        self.exit_status
    }

    pub fn parsed_version(&self) -> Option<&str> {
        self.parsed_version.as_deref()
    }

    pub fn diagnostics(&self) -> &[BackendDiagnostic] {
        &self.diagnostics
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendCandidateEvidence {
    candidate_id: String,
    payload: BackendCandidatePayload,
    target_binding: AtpTargetBinding,
    encoded_problem_hash: Hash,
    provenance_hash: Hash,
    formula_label_refs: Vec<String>,
    symbol_binding_refs: Vec<String>,
}

impl BackendCandidateEvidence {
    pub fn new(
        candidate_id: impl Into<String>,
        payload: BackendCandidatePayload,
        target_binding: AtpTargetBinding,
        encoded_problem_hash: Hash,
        provenance_hash: Hash,
        formula_label_refs: Vec<String>,
        symbol_binding_refs: Vec<String>,
    ) -> Result<Self, BackendConfigError> {
        let candidate_id = candidate_id.into();
        reject_empty("candidate_id", &candidate_id)?;
        if payload.is_empty_formula_substitution_payload() {
            return Err(BackendConfigError::EmptyField {
                field: "candidate_payload",
            });
        }
        Ok(Self {
            candidate_id,
            payload,
            target_binding,
            encoded_problem_hash,
            provenance_hash,
            formula_label_refs: sorted_unique("candidate.formula_label_refs", formula_label_refs)?,
            symbol_binding_refs: sorted_unique(
                "candidate.symbol_binding_refs",
                symbol_binding_refs,
            )?,
        })
    }

    pub fn candidate_id(&self) -> &str {
        &self.candidate_id
    }

    pub const fn payload(&self) -> &BackendCandidatePayload {
        &self.payload
    }

    pub const fn target_binding(&self) -> &AtpTargetBinding {
        &self.target_binding
    }

    pub const fn encoded_problem_hash(&self) -> Hash {
        self.encoded_problem_hash
    }

    pub const fn provenance_hash(&self) -> Hash {
        self.provenance_hash
    }

    pub fn formula_label_refs(&self) -> &[String] {
        &self.formula_label_refs
    }

    pub fn symbol_binding_refs(&self) -> &[String] {
        &self.symbol_binding_refs
    }

    fn mismatch(&self, problem: &EncodedBackendProblem) -> Option<&'static str> {
        if !self.payload.is_formula_substitution_candidate() {
            return Some("unsupported_candidate_payload");
        }
        if self.target_binding != *problem.target_binding() {
            return Some("candidate_target_mismatch");
        }
        if self.encoded_problem_hash != problem.input_hash() {
            return Some("candidate_input_hash_mismatch");
        }
        if self.provenance_hash != problem.provenance_hash() {
            return Some("candidate_provenance_mismatch");
        }
        if self.formula_label_refs != problem.formula_labels() {
            return Some("candidate_label_mismatch");
        }
        if self.symbol_binding_refs != problem.symbol_bindings() {
            return Some("candidate_symbol_mismatch");
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendCandidatePayload {
    FormulaSubstitutionBytes(Vec<u8>),
    FormulaSubstitutionRef(String),
    BackendProofMethod(Vec<u8>),
    BackendLog(Vec<u8>),
    UnsatCore(Vec<u8>),
    SmtProofObject(Vec<u8>),
    TstpTrace(Vec<u8>),
    ResolutionTrace(Vec<u8>),
    UsedAxioms(Vec<u8>),
}

impl BackendCandidatePayload {
    pub fn is_formula_substitution_candidate(&self) -> bool {
        matches!(
            self,
            Self::FormulaSubstitutionBytes(bytes) if !bytes.is_empty()
        ) || matches!(self, Self::FormulaSubstitutionRef(reference) if !reference.trim().is_empty())
    }

    fn is_empty_formula_substitution_payload(&self) -> bool {
        matches!(self, Self::FormulaSubstitutionBytes(bytes) if bytes.is_empty())
            || matches!(self, Self::FormulaSubstitutionRef(reference) if reference.trim().is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendCounterexample {
    payload: Vec<u8>,
    provenance_mapped: bool,
}

impl BackendCounterexample {
    pub fn new(
        payload: impl Into<Vec<u8>>,
        provenance_mapped: bool,
    ) -> Result<Self, BackendConfigError> {
        let payload = payload.into();
        if payload.is_empty() {
            return Err(BackendConfigError::EmptyField {
                field: "counterexample.payload",
            });
        }
        Ok(Self {
            payload,
            provenance_mapped,
        })
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub const fn provenance_mapped(&self) -> bool {
        self.provenance_mapped
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendObservation {
    observed_result: BackendObservedResult,
    candidate_evidence: Option<BackendCandidateEvidence>,
    counterexample: Option<BackendCounterexample>,
    requires_complete_output: bool,
}

impl BackendObservation {
    pub const fn new(observed_result: BackendObservedResult) -> Self {
        Self {
            observed_result,
            candidate_evidence: None,
            counterexample: None,
            requires_complete_output: true,
        }
    }

    pub fn with_candidate_evidence(mut self, candidate_evidence: BackendCandidateEvidence) -> Self {
        self.candidate_evidence = Some(candidate_evidence);
        self
    }

    pub fn with_counterexample(mut self, counterexample: BackendCounterexample) -> Self {
        self.counterexample = Some(counterexample);
        self
    }

    pub const fn without_complete_output_requirement(mut self) -> Self {
        self.requires_complete_output = false;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendDiagnostic {
    key: String,
    message: String,
}

impl BackendDiagnostic {
    pub fn new(key: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            message: message.into(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BackendConfigError {
    EmptyField { field: &'static str },
    DuplicateField { field: &'static str, value: String },
    DuplicateEnvironmentKey { key: String },
}

impl fmt::Display for BackendConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "empty backend field `{field}`"),
            Self::DuplicateField { field, value } => {
                write!(
                    formatter,
                    "duplicate backend field `{field}` value `{value}`"
                )
            }
            Self::DuplicateEnvironmentKey { key } => {
                write!(formatter, "duplicate backend environment key `{key}`")
            }
        }
    }
}

impl Error for BackendConfigError {}

pub fn run_backend(input: BackendRunInput) -> BackendRunResult {
    let command_fingerprint = command_fingerprint(&input);
    let mut diagnostics = input.resource_limits.unsupported_limit_diagnostics();
    if input.resource_limits.has_required_unsupported_limits() {
        diagnostics.push(BackendDiagnostic::new(
            "required_resource_limit_unsupported",
            "one or more required platform resource limits are unsupported",
        ));
        return base_result(
            &input,
            BaseResultParts {
                command_fingerprint,
                version_record: None,
                status: BackendRunStatus::Error,
                termination: BackendTermination::NotStarted,
                exit_status: None,
                stdout: empty_capture("stdout"),
                stderr: empty_capture("stderr"),
                elapsed: Duration::ZERO,
                child_reaped: false,
                diagnostics,
            },
        );
    }

    if input.cancellation.is_cancelled() {
        diagnostics.push(BackendDiagnostic::new(
            "cancelled_before_spawn",
            "backend run was cancelled before process spawn",
        ));
        return base_result(
            &input,
            BaseResultParts {
                command_fingerprint,
                version_record: None,
                status: BackendRunStatus::Cancelled,
                termination: BackendTermination::Cancelled,
                exit_status: None,
                stdout: empty_capture("stdout"),
                stderr: empty_capture("stderr"),
                elapsed: Duration::ZERO,
                child_reaped: false,
                diagnostics,
            },
        );
    }

    let version_record = match input.profile.version_probe() {
        Some(probe) => {
            let record = run_version_probe(probe, &input);
            if !record.success() {
                diagnostics.extend(record.diagnostics().iter().cloned());
                diagnostics.push(BackendDiagnostic::new(
                    "version_probe_failed",
                    "backend version probe did not complete successfully",
                ));
                let (status, termination) = if input.cancellation.is_cancelled() {
                    diagnostics.push(BackendDiagnostic::new(
                        "version_probe_cancelled",
                        "backend version probe was cancelled before the main run",
                    ));
                    (BackendRunStatus::Cancelled, BackendTermination::Cancelled)
                } else {
                    (BackendRunStatus::Error, BackendTermination::NotStarted)
                };
                return base_result(
                    &input,
                    BaseResultParts {
                        command_fingerprint,
                        version_record: Some(record),
                        status,
                        termination,
                        exit_status: None,
                        stdout: empty_capture("stdout"),
                        stderr: empty_capture("stderr"),
                        elapsed: Duration::ZERO,
                        child_reaped: false,
                        diagnostics,
                    },
                );
            }
            Some(record)
        }
        None => None,
    };

    let temp_problem = if input.io_mode == BackendIoMode::PrivateProblemFile {
        match PrivateProblemFile::create(input.encoded_problem.input_text()) {
            Ok(file) => Some(file),
            Err(error) => {
                diagnostics.push(BackendDiagnostic::new(
                    "private_problem_file_failed",
                    format!("private problem-file creation failed: {error}"),
                ));
                return base_result(
                    &input,
                    BaseResultParts {
                        command_fingerprint,
                        version_record,
                        status: BackendRunStatus::Error,
                        termination: BackendTermination::NotStarted,
                        exit_status: None,
                        stdout: empty_capture("stdout"),
                        stderr: empty_capture("stderr"),
                        elapsed: Duration::ZERO,
                        child_reaped: false,
                        diagnostics,
                    },
                );
            }
        }
    } else {
        None
    };

    let mut args = input.command.args.clone();
    if let Some(problem_file) = &temp_problem {
        args.push(problem_file.path().display().to_string());
    }
    let stdin_file = if input.io_mode == BackendIoMode::Stdin {
        match PrivateProblemFile::create(input.encoded_problem.input_text()) {
            Ok(file) => Some(file),
            Err(error) => {
                diagnostics.push(BackendDiagnostic::new(
                    "stdin_spool_file_failed",
                    format!("private stdin spool creation failed: {error}"),
                ));
                return base_result(
                    &input,
                    BaseResultParts {
                        command_fingerprint,
                        version_record,
                        status: BackendRunStatus::Error,
                        termination: BackendTermination::NotStarted,
                        exit_status: None,
                        stdout: empty_capture("stdout"),
                        stderr: empty_capture("stderr"),
                        elapsed: Duration::ZERO,
                        child_reaped: false,
                        diagnostics,
                    },
                );
            }
        }
    } else {
        None
    };

    let stdin_source = match stdin_file.as_ref().map(PrivateProblemFile::open_read) {
        Some(Ok(file)) => Some(file),
        Some(Err(error)) => {
            diagnostics.push(BackendDiagnostic::new(
                "stdin_spool_file_failed",
                format!("private stdin spool could not be opened for reading: {error}"),
            ));
            if let Some(file) = stdin_file {
                diagnostics.extend(file.cleanup_diagnostics());
            }
            return base_result(
                &input,
                BaseResultParts {
                    command_fingerprint,
                    version_record,
                    status: BackendRunStatus::Error,
                    termination: BackendTermination::NotStarted,
                    exit_status: None,
                    stdout: empty_capture("stdout"),
                    stderr: empty_capture("stderr"),
                    elapsed: Duration::ZERO,
                    child_reaped: false,
                    diagnostics,
                },
            );
        }
        None => None,
    };

    let execution = execute_process(ProcessRequest {
        executable: input.command.executable.clone(),
        args,
        environment: input.command.environment.clone(),
        working_directory: input.command.working_directory.clone(),
        stdin_source,
        timeout: input.resource_limits.wall_timeout,
        kill_grace: input.resource_limits.kill_grace,
        stdout_limit: input.resource_limits.stdout_bytes,
        stderr_limit: input.resource_limits.stderr_bytes,
        cancellation: input.cancellation.clone(),
    });

    if let Some(problem_file) = temp_problem {
        diagnostics.extend(problem_file.cleanup_diagnostics());
    }
    if let Some(file) = stdin_file {
        diagnostics.extend(file.cleanup_diagnostics());
    }

    let status = process_status(&execution);
    diagnostics.extend(execution.diagnostics);
    base_result(
        &input,
        BaseResultParts {
            command_fingerprint,
            version_record,
            status,
            termination: execution.termination,
            exit_status: execution.exit_status,
            stdout: execution.stdout,
            stderr: execution.stderr,
            elapsed: execution.elapsed,
            child_reaped: execution.child_reaped,
            diagnostics,
        },
    )
}

pub fn classify_backend_observation(
    mut result: BackendRunResult,
    observation: BackendObservation,
) -> BackendRunResult {
    result.observed_result = Some(observation.observed_result);
    result.candidate_evidence = observation.candidate_evidence;
    result.counterexample = observation.counterexample;

    if result.status != BackendRunStatus::Unknown {
        result.with_diagnostic(
            "proved_rejected_process_status",
            "process status prevents backend candidate proof classification",
        );
        return result;
    }

    if result.stdout.incomplete() || result.stderr.incomplete() {
        result.status = BackendRunStatus::Error;
        result.with_diagnostic(
            "proved_rejected_incomplete_output",
            "incomplete backend output cannot be used for proof classification",
        );
        return result;
    }

    if observation.requires_complete_output
        && (result.stdout.truncated() || result.stderr.truncated())
    {
        result.status = BackendRunStatus::Error;
        result.with_diagnostic(
            "proved_rejected_incomplete_output",
            "complete retained backend output is required for mock classification",
        );
        return result;
    }

    match observation.observed_result {
        BackendObservedResult::Unsat => {
            if result.encoded_problem.expected_result() != ExpectedBackendResult::Unsat {
                result.with_diagnostic(
                    "proved_rejected_expected_result",
                    "observed result does not match expected backend polarity",
                );
                return result;
            }
            let Some(candidate) = result.candidate_evidence.as_ref() else {
                result.with_diagnostic(
                    "proved_rejected_missing_candidate",
                    "formula/substitution candidate evidence is required",
                );
                return result;
            };
            if let Some(key) = candidate.mismatch(&result.encoded_problem) {
                result.with_diagnostic(
                    key,
                    "candidate evidence metadata or payload does not match encoded problem",
                );
                return result;
            }
            result.status = BackendRunStatus::Proved;
        }
        BackendObservedResult::Sat | BackendObservedResult::CounterSatisfiable => {
            if result
                .counterexample
                .as_ref()
                .is_some_and(BackendCounterexample::provenance_mapped)
            {
                result.status = BackendRunStatus::Counterexample;
            } else {
                result.with_diagnostic(
                    "counterexample_unmapped",
                    "counterexample status is diagnostic-only and requires provenance mapping",
                );
            }
        }
        BackendObservedResult::Unknown | BackendObservedResult::Unsupported => {
            result.with_diagnostic(
                "backend_result_unknown",
                "backend output did not produce a supported proof candidate",
            );
        }
        BackendObservedResult::Malformed => {
            result.status = BackendRunStatus::Error;
            result.with_diagnostic(
                "backend_result_malformed",
                "backend output was malformed for mock classification",
            );
        }
    }

    result
}

struct BaseResultParts {
    command_fingerprint: Hash,
    version_record: Option<BackendVersionRecord>,
    status: BackendRunStatus,
    termination: BackendTermination,
    exit_status: Option<BackendExitStatus>,
    stdout: BackendStreamCapture,
    stderr: BackendStreamCapture,
    elapsed: Duration,
    child_reaped: bool,
    diagnostics: Vec<BackendDiagnostic>,
}

fn base_result(input: &BackendRunInput, parts: BaseResultParts) -> BackendRunResult {
    BackendRunResult {
        run_id: input.run_id.clone(),
        encoded_problem: input.encoded_problem.clone(),
        backend_kind: input.profile.backend_kind.clone(),
        profile_id: input.profile.profile_id.clone(),
        command_fingerprint: parts.command_fingerprint,
        metadata_config: metadata_config(input),
        version_record: parts.version_record,
        status: parts.status,
        observed_result: None,
        candidate_evidence: None,
        counterexample: None,
        stdout: parts.stdout,
        stderr: parts.stderr,
        exit_status: parts.exit_status,
        termination: parts.termination,
        elapsed: parts.elapsed,
        child_reaped: parts.child_reaped,
        diagnostics: parts.diagnostics,
    }
}

fn metadata_config(input: &BackendRunInput) -> BackendRunMetadataConfig {
    BackendRunMetadataConfig {
        semantic_executable_id: input.command.semantic_executable_id(),
        args: input.command.args.clone(),
        environment: input.command.environment.vars.clone(),
        working_directory_policy_kind: input.command.working_directory.hash_kind(),
        io_mode: input.io_mode,
        random_seed: input.random_seed,
        resource_limits: input.resource_limits.clone(),
    }
}

pub(crate) fn backend_run_command_fingerprint(input: &BackendRunInput) -> Hash {
    command_fingerprint(input)
}

#[cfg(test)]
pub(crate) fn synthetic_backend_result(
    input: &BackendRunInput,
    status: BackendRunStatus,
) -> BackendRunResult {
    synthetic_backend_result_with_diagnostics(input, status, Vec::new())
}

#[cfg(test)]
pub(crate) fn synthetic_backend_result_with_diagnostics(
    input: &BackendRunInput,
    status: BackendRunStatus,
    diagnostics: Vec<BackendDiagnostic>,
) -> BackendRunResult {
    let (termination, exit_status, child_reaped) = match status {
        BackendRunStatus::Proved | BackendRunStatus::Counterexample | BackendRunStatus::Unknown => {
            (
                BackendTermination::Exited,
                Some(BackendExitStatus {
                    code: Some(0),
                    success: true,
                }),
                true,
            )
        }
        BackendRunStatus::Timeout => (BackendTermination::Timeout, None, true),
        BackendRunStatus::Error => (BackendTermination::ProcessError, None, true),
        BackendRunStatus::Cancelled => (BackendTermination::Cancelled, None, true),
    };

    base_result(
        input,
        BaseResultParts {
            command_fingerprint: command_fingerprint(input),
            version_record: None,
            status,
            termination,
            exit_status,
            stdout: empty_capture("stdout"),
            stderr: empty_capture("stderr"),
            elapsed: Duration::ZERO,
            child_reaped,
            diagnostics,
        },
    )
}

fn process_status(execution: &ProcessExecution) -> BackendRunStatus {
    match execution.termination {
        BackendTermination::Timeout => BackendRunStatus::Timeout,
        BackendTermination::Cancelled => BackendRunStatus::Cancelled,
        BackendTermination::SpawnFailure | BackendTermination::ProcessError => {
            BackendRunStatus::Error
        }
        BackendTermination::NotStarted => BackendRunStatus::Error,
        BackendTermination::Exited => {
            if execution
                .exit_status
                .is_some_and(BackendExitStatus::success)
            {
                BackendRunStatus::Unknown
            } else {
                BackendRunStatus::Error
            }
        }
    }
}

fn run_version_probe(probe: &BackendVersionProbe, input: &BackendRunInput) -> BackendVersionRecord {
    let execution = execute_process(ProcessRequest {
        executable: probe.executable.clone(),
        args: probe.args.clone(),
        environment: input.command.environment.clone(),
        working_directory: input.command.working_directory.clone(),
        stdin_source: None,
        timeout: probe.timeout,
        kill_grace: input.resource_limits.kill_grace,
        stdout_limit: input
            .resource_limits
            .stdout_bytes
            .min(DEFAULT_CAPTURE_LIMIT),
        stderr_limit: input
            .resource_limits
            .stderr_bytes
            .min(DEFAULT_CAPTURE_LIMIT),
        cancellation: input.cancellation.clone(),
    });
    let success = execution.termination == BackendTermination::Exited
        && execution
            .exit_status
            .is_some_and(BackendExitStatus::success)
        && !execution.stdout.incomplete()
        && !execution.stderr.incomplete();
    let parsed_version = success
        .then(|| {
            String::from_utf8_lossy(execution.stdout.retained_bytes())
                .trim()
                .to_owned()
        })
        .filter(|version| !version.is_empty());

    BackendVersionRecord {
        success,
        stdout: execution.stdout,
        stderr: execution.stderr,
        exit_status: execution.exit_status,
        parsed_version,
        diagnostics: execution.diagnostics,
    }
}

struct ProcessRequest {
    executable: PathBuf,
    args: Vec<String>,
    environment: BackendEnvironmentPolicy,
    working_directory: BackendWorkingDirectoryPolicy,
    stdin_source: Option<File>,
    timeout: Duration,
    kill_grace: Duration,
    stdout_limit: usize,
    stderr_limit: usize,
    cancellation: BackendCancellationToken,
}

struct ProcessExecution {
    stdout: BackendStreamCapture,
    stderr: BackendStreamCapture,
    exit_status: Option<BackendExitStatus>,
    termination: BackendTermination,
    elapsed: Duration,
    child_reaped: bool,
    diagnostics: Vec<BackendDiagnostic>,
}

fn execute_process(request: ProcessRequest) -> ProcessExecution {
    let start = Instant::now();
    let mut command = Command::new(&request.executable);
    command.args(&request.args);
    command.env_clear();
    for (key, value) in request.environment.vars() {
        command.env(key, value);
    }
    if let BackendWorkingDirectoryPolicy::Directory(directory) = &request.working_directory {
        command.current_dir(directory);
    }
    if let Some(stdin_source) = request.stdin_source {
        command.stdin(Stdio::from(stdin_source));
    } else {
        command.stdin(Stdio::null());
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut diagnostics = Vec::new();
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            diagnostics.push(BackendDiagnostic::new(
                "spawn_failed",
                format!(
                    "failed to spawn backend executable `{}`: {error}",
                    request.executable.display()
                ),
            ));
            return ProcessExecution {
                stdout: empty_capture("stdout"),
                stderr: empty_capture("stderr"),
                exit_status: None,
                termination: BackendTermination::SpawnFailure,
                elapsed: start.elapsed(),
                child_reaped: false,
                diagnostics,
            };
        }
    };

    let stdout_handle = child
        .stdout
        .take()
        .map(|stdout| spawn_capture_reader("stdout", stdout, request.stdout_limit));
    let stderr_handle = child
        .stderr
        .take()
        .map(|stderr| spawn_capture_reader("stderr", stderr, request.stderr_limit));

    let mut termination = BackendTermination::ProcessError;
    let mut exit_status = None;
    let mut child_reaped = false;

    loop {
        if request.cancellation.is_cancelled() {
            let _ = child.kill();
            termination = BackendTermination::Cancelled;
            match child.wait() {
                Ok(status) => {
                    exit_status = Some(exit_status_from(status));
                    child_reaped = true;
                }
                Err(error) => diagnostics.push(BackendDiagnostic::new(
                    "wait_failed",
                    format!("failed to wait for cancelled backend: {error}"),
                )),
            }
            break;
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                exit_status = Some(exit_status_from(status));
                termination = BackendTermination::Exited;
                child_reaped = true;
                break;
            }
            Ok(None) => {}
            Err(error) => {
                diagnostics.push(BackendDiagnostic::new(
                    "wait_failed",
                    format!("failed to poll backend process: {error}"),
                ));
                let _ = child.kill();
                let _ = child.wait();
                child_reaped = true;
                break;
            }
        }

        if start.elapsed() >= request.timeout {
            let _ = child.kill();
            termination = BackendTermination::Timeout;
            match wait_after_kill(&mut child, request.kill_grace) {
                Ok(reaped) => {
                    child_reaped = reaped;
                }
                Err(error) => diagnostics.push(BackendDiagnostic::new(
                    "wait_failed",
                    format!("failed to wait for timed out backend: {error}"),
                )),
            }
            break;
        }

        thread::sleep(poll_sleep_duration(start, request.timeout));
    }

    let stdout = join_capture_reader(
        stdout_handle,
        "stdout",
        request.kill_grace,
        &mut diagnostics,
    );
    let stderr = join_capture_reader(
        stderr_handle,
        "stderr",
        request.kill_grace,
        &mut diagnostics,
    );
    if stdout.incomplete() {
        diagnostics.push(BackendDiagnostic::new(
            "stdout_incomplete",
            "stdout stream was not completely drained",
        ));
    }
    if stderr.incomplete() {
        diagnostics.push(BackendDiagnostic::new(
            "stderr_incomplete",
            "stderr stream was not completely drained",
        ));
    }
    if exit_status.is_some_and(|status| !status.success())
        && termination == BackendTermination::Exited
    {
        diagnostics.push(BackendDiagnostic::new(
            "process_crash",
            "backend process exited with a non-zero status",
        ));
    }

    ProcessExecution {
        stdout,
        stderr,
        exit_status,
        termination,
        elapsed: start.elapsed(),
        child_reaped,
        diagnostics,
    }
}

fn wait_after_kill(child: &mut std::process::Child, kill_grace: Duration) -> io::Result<bool> {
    let start = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            return Ok(true);
        }
        if start.elapsed() >= kill_grace {
            let _ = child.kill();
            child.wait()?;
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(5));
    }
}

fn poll_sleep_duration(start: Instant, timeout: Duration) -> Duration {
    let elapsed = start.elapsed();
    let remaining = timeout.saturating_sub(elapsed);
    remaining
        .min(Duration::from_millis(10))
        .max(Duration::from_millis(1))
}

fn spawn_capture_reader(
    stream_name: &'static str,
    mut reader: impl Read + Send + 'static,
    limit: usize,
) -> thread::JoinHandle<BackendStreamCapture> {
    thread::spawn(move || {
        let mut retained = Vec::with_capacity(limit.min(8192));
        let mut hash = StableHasher::new(STREAM_HASH_DOMAIN);
        hash.field_str("stream", stream_name);
        let mut total_bytes = 0_u64;
        let mut truncated = false;
        let mut incomplete = false;
        let mut buffer = [0_u8; 8192];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => {
                    let bytes = &buffer[..read];
                    hash.feed_bytes(bytes);
                    total_bytes = total_bytes.saturating_add(read as u64);
                    let remaining = limit.saturating_sub(retained.len());
                    if remaining > 0 {
                        let keep = remaining.min(read);
                        retained.extend_from_slice(&bytes[..keep]);
                        if keep < read {
                            truncated = true;
                        }
                    } else {
                        truncated = true;
                    }
                }
                Err(_) => {
                    incomplete = true;
                    break;
                }
            }
        }
        hash.field_u64("total-bytes", total_bytes);
        BackendStreamCapture {
            retained,
            hash: hash.finalize(),
            total_bytes,
            truncated,
            incomplete,
        }
    })
}

fn join_capture_reader(
    handle: Option<thread::JoinHandle<BackendStreamCapture>>,
    stream_name: &'static str,
    join_grace: Duration,
    diagnostics: &mut Vec<BackendDiagnostic>,
) -> BackendStreamCapture {
    let Some(handle) = handle else {
        return empty_capture(stream_name);
    };
    let start = Instant::now();
    while !handle.is_finished() {
        if start.elapsed() >= join_grace {
            diagnostics.push(BackendDiagnostic::new(
                "capture_reader_incomplete",
                format!("{stream_name} capture reader did not finish before join grace"),
            ));
            return BackendStreamCapture {
                incomplete: true,
                ..empty_capture(stream_name)
            };
        }
        thread::sleep(poll_sleep_duration(start, join_grace));
    }
    match handle.join() {
        Ok(capture) => capture,
        Err(_) => {
            diagnostics.push(BackendDiagnostic::new(
                "capture_thread_panicked",
                format!("{stream_name} capture thread panicked"),
            ));
            BackendStreamCapture {
                incomplete: true,
                ..empty_capture(stream_name)
            }
        }
    }
}

fn empty_capture(stream_name: &'static str) -> BackendStreamCapture {
    let mut hash = StableHasher::new(STREAM_HASH_DOMAIN);
    hash.field_str("stream", stream_name);
    hash.field_u64("total-bytes", 0);
    BackendStreamCapture {
        retained: Vec::new(),
        hash: hash.finalize(),
        total_bytes: 0,
        truncated: false,
        incomplete: false,
    }
}

struct PrivateProblemFile {
    directory: PathBuf,
    path: PathBuf,
}

impl PrivateProblemFile {
    fn create(input: &[u8]) -> io::Result<Self> {
        let root = std::env::temp_dir();
        for _ in 0..128 {
            let directory = root.join(temp_name());
            match create_private_directory(&directory) {
                Ok(()) => {
                    let path = directory.join("problem.input");
                    let write_result = (|| {
                        let mut file = create_private_file(&path)?;
                        file.write_all(input)?;
                        file.flush()
                    })();
                    match write_result {
                        Ok(()) => return Ok(Self { directory, path }),
                        Err(error) => {
                            let _ = fs::remove_file(&path);
                            let _ = fs::remove_dir(&directory);
                            return Err(error);
                        }
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not allocate a unique private backend directory",
        ))
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn open_read(&self) -> io::Result<File> {
        File::open(&self.path)
    }

    fn cleanup_diagnostics(self) -> Vec<BackendDiagnostic> {
        let mut diagnostics = Vec::new();
        if let Err(error) = fs::remove_file(&self.path)
            && error.kind() != io::ErrorKind::NotFound
        {
            diagnostics.push(BackendDiagnostic::new(
                "temp_problem_file_cleanup_failed",
                format!("failed to remove private problem file: {error}"),
            ));
        }
        if let Err(error) = fs::remove_dir(&self.directory)
            && error.kind() != io::ErrorKind::NotFound
        {
            diagnostics.push(BackendDiagnostic::new(
                "temp_problem_dir_cleanup_failed",
                format!("failed to remove private backend directory: {error}"),
            ));
        }
        diagnostics
    }
}

fn temp_name() -> String {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos();
    let mut hash = StableHasher::new(TEMP_COUNTER_DOMAIN);
    hash.field_u64("pid", u64::from(std::process::id()));
    hash.field_u64("counter", counter);
    hash.field_bytes("time", &now.to_le_bytes());
    format!("mizar-atp-backend-{}", hex(hash.finalize().as_bytes()))
}

#[cfg(unix)]
fn create_private_directory(path: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder.create(path)?;
    if let Err(error) = verify_private_directory(path) {
        let _ = fs::remove_dir(path);
        return Err(error);
    }
    Ok(())
}

#[cfg(not(unix))]
fn create_private_directory(path: &Path) -> io::Result<()> {
    fs::create_dir(path)
}

#[cfg(unix)]
fn create_private_file(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    verify_private_file(&file)?;
    Ok(file)
}

#[cfg(not(unix))]
fn create_private_file(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

#[cfg(unix)]
fn verify_private_directory(path: &Path) -> io::Result<()> {
    let mode = fs::metadata(path)?.permissions().mode();
    if mode & 0o077 == 0 {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "private backend directory has group or world permissions",
        ))
    }
}

#[cfg(unix)]
fn verify_private_file(file: &File) -> io::Result<()> {
    let mode = file.metadata()?.permissions().mode();
    if mode & 0o077 == 0 {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "private backend problem file has group or world permissions",
        ))
    }
}

fn exit_status_from(status: ExitStatus) -> BackendExitStatus {
    BackendExitStatus {
        code: status.code(),
        success: status.success(),
    }
}

fn command_fingerprint(input: &BackendRunInput) -> Hash {
    let mut hash = StableHasher::new(COMMAND_HASH_DOMAIN);
    hash.field_str("backend-kind", input.profile.backend_kind().as_str());
    hash.field_str("profile-id", input.profile.profile_id().as_str());
    hash.field_str(
        "format",
        concrete_format_name(input.profile.concrete_format()),
    );
    hash.field_str("executable", &input.command.semantic_executable_id());
    for arg in input.command.args() {
        hash.field_str("arg", arg);
    }
    for (key, value) in input.command.environment().vars() {
        hash.field_str("env-key", key);
        hash.field_str("env-value", value);
    }
    hash.field_str(
        "working-directory-policy",
        input.command.working_directory().hash_kind(),
    );
    hash.field_str("io-mode", input.io_mode.as_str());
    hash.field_u64("random-seed", input.random_seed.unwrap_or(0));
    hash.field_bool("random-seed-present", input.random_seed.is_some());
    hash.field_u64(
        "wall-timeout-ms",
        duration_millis_u64(input.resource_limits.wall_timeout()),
    );
    hash.field_u64(
        "kill-grace-ms",
        duration_millis_u64(input.resource_limits.kill_grace()),
    );
    hash.field_u64("stdout-limit", input.resource_limits.stdout_bytes() as u64);
    hash.field_u64("stderr-limit", input.resource_limits.stderr_bytes() as u64);
    for (name, requirement) in input.resource_limits.platform_limits() {
        hash.field_str("platform-limit", name);
        hash.field_str("platform-limit-requirement", requirement.as_str());
    }
    hash.finalize()
}

fn hash_input(
    concrete_format: ConcreteFormat,
    logic_profile_name: &str,
    logic_fragment: &str,
    input_text: &[u8],
) -> Hash {
    let mut hash = StableHasher::new(INPUT_HASH_DOMAIN);
    hash.field_str("format", concrete_format_name(concrete_format));
    hash.field_str("logic-profile-name", logic_profile_name);
    hash.field_str("logic-fragment", logic_fragment);
    hash.field_bytes("input-text", input_text);
    hash.finalize()
}

fn hash_metadata(
    target_binding: &AtpTargetBinding,
    formula_labels: &[String],
    symbol_bindings: &[String],
    provenance_hash: Hash,
) -> Hash {
    let mut hash = StableHasher::new(METADATA_HASH_DOMAIN);
    hash.field_bytes("target-fingerprint", target_binding.fingerprint().digest());
    hash.field_str(
        "target-producer",
        target_binding.producer_binding().as_str(),
    );
    for label in formula_labels {
        hash.field_str("formula-label", label);
    }
    for symbol in symbol_bindings {
        hash.field_str("symbol-binding", symbol);
    }
    hash.field_bytes("provenance-hash", provenance_hash.as_bytes());
    hash.finalize()
}

struct StableHasher {
    lanes: [u64; 4],
    length: u64,
}

impl StableHasher {
    fn new(domain: &str) -> Self {
        let mut hasher = Self {
            lanes: [
                0x6d_69_7a_61_72_2d_61_74,
                0x70_2d_62_61_63_6b_65_6e,
                0x64_2d_68_61_73_68_2d_76,
                0x31_2d_63_61_6e_6f_6e_69,
            ],
            length: 0,
        };
        hasher.field_str("domain", domain);
        hasher
    }

    fn field_str(&mut self, label: &str, value: &str) {
        self.field_bytes(label, value.as_bytes());
    }

    fn field_u64(&mut self, label: &str, value: u64) {
        self.field_bytes(label, &value.to_le_bytes());
    }

    fn field_bool(&mut self, label: &str, value: bool) {
        self.field_bytes(label, &[u8::from(value)]);
    }

    fn field_bytes(&mut self, label: &str, value: &[u8]) {
        self.feed_bytes(&(label.len() as u64).to_le_bytes());
        self.feed_bytes(label.as_bytes());
        self.feed_bytes(&(value.len() as u64).to_le_bytes());
        self.feed_bytes(value);
    }

    fn feed_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            let lane = self.length as usize % self.lanes.len();
            let mixed = self.length.rotate_left((lane as u32) + 5);
            self.lanes[lane] ^= u64::from(*byte)
                .wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_add(mixed);
            self.lanes[lane] = self.lanes[lane]
                .rotate_left(11 + lane as u32)
                .wrapping_mul(0x1000_0000_01b3);
            self.length = self.length.wrapping_add(1);
        }
    }

    fn finalize(mut self) -> Hash {
        self.lanes[0] ^= self.length;
        self.lanes[1] ^= self.length.rotate_left(17);
        self.lanes[2] ^= self.lanes[0].rotate_left(9);
        self.lanes[3] ^= self.lanes[1].rotate_left(13);

        let mut bytes = [0_u8; Hash::BYTE_LEN];
        for (chunk, lane) in bytes.chunks_exact_mut(8).zip(self.lanes) {
            chunk.copy_from_slice(&lane.to_be_bytes());
        }
        Hash::from_bytes(bytes)
    }
}

fn duration_millis_u64(duration: Duration) -> u64 {
    duration.as_millis().try_into().unwrap_or(u64::MAX)
}

fn concrete_format_name(format: ConcreteFormat) -> &'static str {
    match format {
        ConcreteFormat::Tptp => "tptp",
        ConcreteFormat::SmtLib => "smtlib",
    }
}

fn semantic_path_id(path: &Path) -> String {
    path.file_name()
        .and_then(OsStr::to_str)
        .filter(|name| !name.is_empty())
        .map_or_else(|| path.display().to_string(), ToOwned::to_owned)
}

fn reject_empty(field: &'static str, value: &str) -> Result<(), BackendConfigError> {
    if value.trim().is_empty() {
        Err(BackendConfigError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn reject_empty_path(field: &'static str, path: &Path) -> Result<(), BackendConfigError> {
    if path.as_os_str().is_empty() {
        Err(BackendConfigError::EmptyField { field })
    } else {
        Ok(())
    }
}

fn sorted_unique(
    field: &'static str,
    mut values: Vec<String>,
) -> Result<Vec<String>, BackendConfigError> {
    for value in &values {
        reject_empty(field, value)?;
    }
    values.sort();
    for pair in values.windows(2) {
        if pair[0] == pair[1] {
            return Err(BackendConfigError::DuplicateField {
                field,
                value: pair[0].clone(),
            });
        }
    }
    Ok(values)
}

fn hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").expect("write string");
    }
    encoded
}

#[cfg(all(test, unix))]
mod tests;
