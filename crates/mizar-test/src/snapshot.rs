use std::collections::BTreeMap;
use std::fmt;

use mizar_session::{Hash, hash_text};

use crate::expectation::TestCaseId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaVersion(pub u32);

impl SchemaVersion {
    pub const CURRENT: Self = Self(1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
