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
pub struct BackendRunResult {
    run_id: BackendRunId,
    encoded_problem: EncodedBackendProblem,
    backend_kind: BackendKind,
    profile_id: BackendProfileId,
    command_fingerprint: Hash,
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

    fn with_diagnostic(&mut self, key: &'static str, message: impl Into<String>) {
        self.diagnostics.push(BackendDiagnostic::new(key, message));
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
mod tests {
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
            BackendObservation::new(BackendObservedResult::Unsat)
                .with_candidate_evidence(candidate),
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

    fn candidate_with_provenance_mismatch(
        problem: &EncodedBackendProblem,
    ) -> BackendCandidateEvidence {
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
            BackendObservation::new(BackendObservedResult::Unsat)
                .with_candidate_evidence(candidate),
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
}
