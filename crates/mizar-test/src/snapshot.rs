use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use mizar_session::{Hash, hash_text};

use crate::expectation::TestCaseId;
use crate::path_rules::clean_relative_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaVersion(pub u32);

impl SchemaVersion {
    pub const CURRENT: Self = Self(1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SnapshotKind {
    SurfaceAst,
    TypedAst,
    CoreIr,
    VcIr,
    SatClauses,
    ProofCertificate,
    VerifiedArtifact,
    DependencySlice,
    DependencyFingerprint,
    FailureRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainInfo {
    pub name: String,
    pub version: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParallelismProfile {
    Sequential,
    Parallel { workers: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotProfile {
    pub toolchain: ToolchainInfo,
    pub verifier_config_hash: Hash,
    pub parallelism: ParallelismProfile,
    pub normalize_paths: bool,
    pub allow_local_paths: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBody {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotRecord {
    pub schema_version: SchemaVersion,
    pub test_id: TestCaseId,
    pub kind: SnapshotKind,
    pub profile: SnapshotProfile,
    pub content_hash: Hash,
    pub body: SnapshotBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMismatch {
    pub expected_hash: Hash,
    pub actual_hash: Hash,
    pub first_difference: Option<SnapshotTextDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotTextDiff {
    pub line: usize,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SnapshotUpdateReason {
    SchemaChange,
    DiagnosticContractChange,
    SemanticBehaviorChange,
    FuzzPropertyReproducer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SnapshotUpdateMode {
    VerifyOnly,
    Update { reason: SnapshotUpdateReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SnapshotBaselineStatus {
    Matched,
    Created,
    Updated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBaselineReport {
    pub path: PathBuf,
    pub status: SnapshotBaselineStatus,
    pub update_reason: Option<SnapshotUpdateReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBaselineMismatch {
    pub expected_hash: Option<Hash>,
    pub actual_hash: Hash,
    pub first_difference: Option<SnapshotTextDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SnapshotBaselineError {
    Snapshot(SnapshotError),
    InvalidBaselinePath {
        path: PathBuf,
    },
    Io {
        path: PathBuf,
        message: String,
    },
    MissingBaseline {
        path: PathBuf,
    },
    Mismatch {
        path: PathBuf,
        mismatch: Box<SnapshotBaselineMismatch>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotDeterminismFailure {
    pub baseline_index: usize,
    pub candidate_index: usize,
    pub mismatch: Box<SnapshotMismatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SnapshotError {
    EmptyTestId,
    EmptyToolchainName,
    EmptyToolchainVersion,
    EmptyMetadataKey,
    ParallelWorkerCountZero,
    LocalPath { token: String },
    StaleContentHash { stored: Hash, recomputed: Hash },
}

impl ToolchainInfo {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl SnapshotBody {
    pub fn text(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn canonical_text(&self) -> String {
        normalize_line_endings(&self.text)
    }
}

impl SnapshotRecord {
    pub fn new(
        test_id: TestCaseId,
        kind: SnapshotKind,
        profile: SnapshotProfile,
        body: SnapshotBody,
    ) -> Result<Self, SnapshotError> {
        let schema_version = SchemaVersion::CURRENT;
        validate_record_parts(&test_id, &profile, &body)?;
        let content_hash = hash_text(&canonical_hash_input(
            schema_version,
            &test_id,
            kind,
            &profile,
            &body,
        ));
        Ok(Self {
            schema_version,
            test_id,
            kind,
            profile,
            content_hash,
            body,
        })
    }

    pub fn canonical_hash_input(&self) -> Result<String, SnapshotError> {
        validate_record_parts(&self.test_id, &self.profile, &self.body)?;
        Ok(canonical_hash_input(
            self.schema_version,
            &self.test_id,
            self.kind,
            &self.profile,
            &self.body,
        ))
    }

    pub fn recomputed_content_hash(&self) -> Result<Hash, SnapshotError> {
        Ok(hash_text(&self.canonical_hash_input()?))
    }

    pub fn canonical_text(&self) -> Result<String, SnapshotError> {
        let recomputed = self.recomputed_content_hash()?;
        if recomputed != self.content_hash {
            return Err(SnapshotError::StaleContentHash {
                stored: self.content_hash,
                recomputed,
            });
        }
        let mut output = self.canonical_hash_input()?;
        output.push_str("content_hash = ");
        output.push_str(&hash_hex(recomputed));
        output.push('\n');
        Ok(output)
    }
}

pub fn compare_snapshot_records(
    expected: &SnapshotRecord,
    actual: &SnapshotRecord,
) -> Result<(), SnapshotMismatch> {
    let expected_recomputed = expected.recomputed_content_hash();
    let actual_recomputed = actual.recomputed_content_hash();
    let expected_valid = expected_recomputed.is_ok();
    let actual_valid = actual_recomputed.is_ok();
    let expected_hash = expected_recomputed.unwrap_or(expected.content_hash);
    let actual_hash = actual_recomputed.unwrap_or(actual.content_hash);

    if expected_valid && actual_valid && expected_hash == actual_hash {
        return Ok(());
    }

    let expected_text = expected.canonical_hash_input().unwrap_or_default();
    let actual_text = actual.canonical_hash_input().unwrap_or_default();
    Err(SnapshotMismatch {
        expected_hash,
        actual_hash,
        first_difference: first_text_difference(
            &expected.body.canonical_text(),
            &actual.body.canonical_text(),
        )
        .or_else(|| first_text_difference(&expected_text, &actual_text)),
    })
}

pub fn verify_or_update_snapshot_baseline(
    tests_root: impl AsRef<Path>,
    relative_path: impl AsRef<Path>,
    record: &SnapshotRecord,
    mode: SnapshotUpdateMode,
) -> Result<SnapshotBaselineReport, SnapshotBaselineError> {
    let relative_path = relative_path.as_ref();
    validate_baseline_path(relative_path)?;

    let full_path = tests_root.as_ref().join(relative_path);
    let actual_text = record
        .canonical_text()
        .map_err(SnapshotBaselineError::Snapshot)?;
    match fs::read_to_string(&full_path) {
        Ok(expected_text) if expected_text == actual_text => Ok(SnapshotBaselineReport {
            path: relative_path.to_path_buf(),
            status: SnapshotBaselineStatus::Matched,
            update_reason: None,
        }),
        Ok(expected_text) => match mode {
            SnapshotUpdateMode::VerifyOnly => Err(SnapshotBaselineError::Mismatch {
                path: relative_path.to_path_buf(),
                mismatch: Box::new(baseline_mismatch(&expected_text, record, &actual_text)),
            }),
            SnapshotUpdateMode::Update { reason } => {
                write_snapshot_baseline(&full_path, &actual_text)?;
                Ok(SnapshotBaselineReport {
                    path: relative_path.to_path_buf(),
                    status: SnapshotBaselineStatus::Updated,
                    update_reason: Some(reason),
                })
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => match mode {
            SnapshotUpdateMode::VerifyOnly => Err(SnapshotBaselineError::MissingBaseline {
                path: relative_path.to_path_buf(),
            }),
            SnapshotUpdateMode::Update { reason } => {
                write_snapshot_baseline(&full_path, &actual_text)?;
                Ok(SnapshotBaselineReport {
                    path: relative_path.to_path_buf(),
                    status: SnapshotBaselineStatus::Created,
                    update_reason: Some(reason),
                })
            }
        },
        Err(error) => Err(SnapshotBaselineError::Io {
            path: relative_path.to_path_buf(),
            message: error.to_string(),
        }),
    }
}

pub fn verify_snapshot_determinism(
    records: &[SnapshotRecord],
) -> Result<(), SnapshotDeterminismFailure> {
    let Some(baseline) = records.first() else {
        return Ok(());
    };
    for (index, candidate) in records.iter().enumerate().skip(1) {
        if let Err(mismatch) = compare_snapshot_records(baseline, candidate) {
            return Err(SnapshotDeterminismFailure {
                baseline_index: 0,
                candidate_index: index,
                mismatch: Box::new(mismatch),
            });
        }
    }
    Ok(())
}

pub fn verify_snapshot_parallel_equivalence(
    sequential: &SnapshotRecord,
    parallel: &SnapshotRecord,
) -> Result<(), SnapshotMismatch> {
    if sequential.recomputed_content_hash().is_err() || parallel.recomputed_content_hash().is_err()
    {
        return compare_snapshot_records(sequential, parallel);
    }

    let sequential = record_with_parallelism(sequential, ParallelismProfile::Sequential);
    let parallel = record_with_parallelism(parallel, ParallelismProfile::Sequential);
    compare_snapshot_records(&sequential, &parallel)
}

fn record_with_parallelism(
    record: &SnapshotRecord,
    parallelism: ParallelismProfile,
) -> SnapshotRecord {
    let mut normalized = record.clone();
    normalized.profile.parallelism = parallelism;
    normalized.content_hash = hash_text(&canonical_hash_input(
        normalized.schema_version,
        &normalized.test_id,
        normalized.kind,
        &normalized.profile,
        &normalized.body,
    ));
    normalized
}

fn validate_record_parts(
    test_id: &TestCaseId,
    profile: &SnapshotProfile,
    body: &SnapshotBody,
) -> Result<(), SnapshotError> {
    if test_id.0.is_empty() {
        return Err(SnapshotError::EmptyTestId);
    }
    if profile.toolchain.name.is_empty() {
        return Err(SnapshotError::EmptyToolchainName);
    }
    if profile.toolchain.version.is_empty() {
        return Err(SnapshotError::EmptyToolchainVersion);
    }
    if profile.toolchain.metadata.keys().any(String::is_empty) {
        return Err(SnapshotError::EmptyMetadataKey);
    }
    if matches!(
        profile.parallelism,
        ParallelismProfile::Parallel { workers: 0 }
    ) {
        return Err(SnapshotError::ParallelWorkerCountZero);
    }
    if !profile.allow_local_paths
        && let Some(token) = first_local_absolute_path(&body.canonical_text())
    {
        return Err(SnapshotError::LocalPath { token });
    }
    Ok(())
}

fn validate_baseline_path(path: &Path) -> Result<(), SnapshotBaselineError> {
    if !clean_relative_path(path)
        || !path.starts_with("snapshots")
        || path.extension().and_then(|extension| extension.to_str()) != Some("snap")
    {
        return Err(SnapshotBaselineError::InvalidBaselinePath {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn write_snapshot_baseline(path: &Path, content: &str) -> Result<(), SnapshotBaselineError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| SnapshotBaselineError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    }
    let temp_path = temporary_baseline_path(path);
    fs::write(&temp_path, content).map_err(|error| SnapshotBaselineError::Io {
        path: temp_path.clone(),
        message: error.to_string(),
    })?;
    fs::rename(&temp_path, path).map_err(|error| SnapshotBaselineError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

fn temporary_baseline_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("snapshot");
    path.with_file_name(format!(".{file_name}.tmp.{}", std::process::id()))
}

fn baseline_mismatch(
    expected_text: &str,
    record: &SnapshotRecord,
    actual_text: &str,
) -> SnapshotBaselineMismatch {
    SnapshotBaselineMismatch {
        expected_hash: content_hash_from_snapshot_text(expected_text),
        actual_hash: record
            .recomputed_content_hash()
            .unwrap_or(record.content_hash),
        first_difference: first_text_difference(expected_text, actual_text),
    }
}

fn content_hash_from_snapshot_text(text: &str) -> Option<Hash> {
    text.lines()
        .find_map(|line| line.strip_prefix("content_hash = "))
        .and_then(parse_lower_hex_hash)
}

fn canonical_hash_input(
    schema_version: SchemaVersion,
    test_id: &TestCaseId,
    kind: SnapshotKind,
    profile: &SnapshotProfile,
    body: &SnapshotBody,
) -> String {
    let mut output = String::new();
    output.push_str("snapshot_record = \"mizar-test-snapshot\"\n");
    output.push_str("schema_version = ");
    output.push_str(&schema_version.0.to_string());
    output.push('\n');
    push_framed_scalar(&mut output, "test_id", &test_id.0);
    output.push_str("kind = ");
    output.push_str(kind.as_str());
    output.push('\n');
    push_profile(&mut output, profile);
    let body_text = body.canonical_text();
    output.push_str("body.len = ");
    output.push_str(&body_text.len().to_string());
    output.push('\n');
    output.push_str("body = <<SNAPSHOT\n");
    output.push_str(&body_text);
    output.push('\n');
    output.push_str("SNAPSHOT\n");
    output
}

fn push_profile(output: &mut String, profile: &SnapshotProfile) {
    push_framed_scalar(output, "profile.toolchain.name", &profile.toolchain.name);
    push_framed_scalar(
        output,
        "profile.toolchain.version",
        &profile.toolchain.version,
    );
    output.push_str("profile.toolchain.metadata.count = ");
    output.push_str(&profile.toolchain.metadata.len().to_string());
    output.push('\n');
    for (index, (key, value)) in profile.toolchain.metadata.iter().enumerate() {
        push_framed_scalar(
            output,
            &format!("profile.toolchain.metadata.{index}.key"),
            key,
        );
        push_framed_scalar(
            output,
            &format!("profile.toolchain.metadata.{index}.value"),
            value,
        );
    }
    output.push_str("profile.verifier_config_hash = ");
    output.push_str(&hash_hex(profile.verifier_config_hash));
    output.push('\n');
    output.push_str("profile.parallelism = ");
    match profile.parallelism {
        ParallelismProfile::Sequential => output.push_str("sequential"),
        ParallelismProfile::Parallel { workers } => {
            output.push_str("parallel:");
            output.push_str(&workers.to_string());
        }
    }
    output.push('\n');
    output.push_str("profile.normalize_paths = ");
    output.push_str(if profile.normalize_paths {
        "true"
    } else {
        "false"
    });
    output.push('\n');
    output.push_str("profile.allow_local_paths = ");
    output.push_str(if profile.allow_local_paths {
        "true"
    } else {
        "false"
    });
    output.push('\n');
}

fn push_framed_scalar(output: &mut String, label: &str, value: &str) {
    output.push_str(label);
    output.push_str(".len = ");
    output.push_str(&value.len().to_string());
    output.push('\n');
    output.push_str(label);
    output.push_str(" = ");
    output.push_str(&escape_scalar(value));
    output.push('\n');
}

fn normalize_line_endings(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn escape_scalar(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn first_local_absolute_path(text: &str) -> Option<String> {
    for (index, _) in text.char_indices() {
        if !is_path_start_boundary(text, index) {
            continue;
        }
        let suffix = &text[index..];
        if looks_like_local_absolute_path(suffix) {
            return Some(take_path_token(suffix));
        }
    }
    None
}

fn looks_like_local_absolute_path(token: &str) -> bool {
    if token.starts_with("file:///") {
        return true;
    }
    let second_byte = token.as_bytes().get(1);
    let unix_absolute = token.len() > 2
        && token.starts_with('/')
        && !token.starts_with("//")
        && second_byte
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    let bytes = token.as_bytes();
    let windows_drive = bytes.len() > 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && matches!(bytes[2], b'\\' | b'/');
    let unc = token.starts_with("\\\\");
    unix_absolute || windows_drive || unc
}

fn is_path_start_boundary(text: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }
    text[..index].chars().next_back().is_some_and(|ch| {
        ch.is_whitespace()
            || matches!(
                ch,
                '=' | ':' | '"' | '\'' | '`' | '(' | '[' | '{' | '<' | ',' | ';'
            )
    })
}

fn take_path_token(suffix: &str) -> String {
    suffix
        .split(|ch: char| {
            ch.is_whitespace() || matches!(ch, '"' | '\'' | '`' | ')' | ']' | '}' | '>' | ',' | ';')
        })
        .next()
        .unwrap_or(suffix)
        .trim_end_matches('.')
        .to_owned()
}

fn first_text_difference(expected: &str, actual: &str) -> Option<SnapshotTextDiff> {
    let expected_lines = split_lines_preserving_terminal(expected);
    let actual_lines = split_lines_preserving_terminal(actual);
    let max_len = expected_lines.len().max(actual_lines.len());
    for index in 0..max_len {
        let expected = expected_lines.get(index).cloned();
        let actual = actual_lines.get(index).cloned();
        if expected != actual {
            return Some(SnapshotTextDiff {
                line: index + 1,
                expected,
                actual,
            });
        }
    }
    None
}

fn split_lines_preserving_terminal(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
    if text.ends_with('\n') {
        lines.push(String::new());
    }
    lines
}

fn hash_hex(hash: Hash) -> String {
    let mut output = String::with_capacity(Hash::BYTE_LEN * 2);
    for byte in hash.as_bytes() {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn parse_lower_hex_hash(value: &str) -> Option<Hash> {
    if value.len() != Hash::BYTE_LEN * 2 {
        return None;
    }
    let mut bytes = [0; Hash::BYTE_LEN];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = parse_lower_hex_nibble(pair[0])?;
        let low = parse_lower_hex_nibble(pair[1])?;
        bytes[index] = (high << 4) | low;
    }
    Some(Hash::from_bytes(bytes))
}

fn parse_lower_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

impl SnapshotKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SurfaceAst => "surface_ast",
            Self::TypedAst => "typed_ast",
            Self::CoreIr => "core_ir",
            Self::VcIr => "vc_ir",
            Self::SatClauses => "sat_clauses",
            Self::ProofCertificate => "proof_certificate",
            Self::VerifiedArtifact => "verified_artifact",
            Self::DependencySlice => "dependency_slice",
            Self::DependencyFingerprint => "dependency_fingerprint",
            Self::FailureRecord => "failure_record",
        }
    }
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTestId => f.write_str("snapshot test id must not be empty"),
            Self::EmptyToolchainName => f.write_str("snapshot toolchain name must not be empty"),
            Self::EmptyToolchainVersion => {
                f.write_str("snapshot toolchain version must not be empty")
            }
            Self::EmptyMetadataKey => f.write_str("snapshot metadata keys must not be empty"),
            Self::ParallelWorkerCountZero => {
                f.write_str("parallel snapshot profile must use at least one worker")
            }
            Self::LocalPath { token } => {
                write!(f, "snapshot body contains local absolute path `{token}`")
            }
            Self::StaleContentHash { stored, recomputed } => write!(
                f,
                "snapshot content hash is stale (stored {}, recomputed {})",
                hash_hex(*stored),
                hash_hex(*recomputed)
            ),
        }
    }
}

impl std::error::Error for SnapshotError {}

impl fmt::Display for SnapshotBaselineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(error) => write!(f, "{error}"),
            Self::InvalidBaselinePath { path } => write!(
                f,
                "snapshot baseline path `{}` must be a clean tests-root-relative .snap file under snapshots/",
                path.display()
            ),
            Self::Io { path, message } => {
                write!(
                    f,
                    "snapshot baseline `{}` IO error: {message}",
                    path.display()
                )
            }
            Self::MissingBaseline { path } => {
                write!(f, "snapshot baseline `{}` is missing", path.display())
            }
            Self::Mismatch { path, .. } => {
                write!(f, "snapshot baseline `{}` differs", path.display())
            }
        }
    }
}

impl std::error::Error for SnapshotBaselineError {}
