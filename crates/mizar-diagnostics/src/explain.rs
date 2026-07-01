//! Lazy diagnostic explanation handles and bounded payload resolution.

use std::{collections::BTreeMap, error::Error, fmt};

use mizar_session::{BuildSnapshotId, Hash, SourceRange};

use crate::{failure_record::PipelinePhase, registry::DiagnosticCode};

/// Default maximum bytes stored in an explanation preview.
pub const DEFAULT_PREVIEW_MAX_BYTES: usize = 4096;
/// Default maximum lines stored in an explanation preview.
pub const DEFAULT_PREVIEW_MAX_LINES: usize = 80;
/// Deterministic marker appended when preview text is truncated.
pub const TRUNCATION_MARKER: &str = " [truncated]";

/// Stable producer-side identity for a lazy explanation handle.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExplanationHandleId {
    identity: String,
}

impl ExplanationHandleId {
    /// Creates an explanation-handle identity.
    pub fn new(identity: impl Into<String>) -> Result<Self, ExplanationError> {
        let identity = identity.into();
        validate_identity(&identity).map_err(|_| ExplanationError::InvalidHandleId {
            identity: identity.clone(),
        })?;
        Ok(Self { identity })
    }

    /// Returns the stable identity string.
    pub fn identity(&self) -> &str {
        &self.identity
    }
}

/// Explanation payload classification.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ExplanationKind {
    /// Type, mode, attribute, or registration reasoning.
    TypeInference,
    /// Attribute or cluster search and loop explanation.
    ClusterResolution,
    /// Candidate set, selected policy, or overload ambiguity detail.
    OverloadResolution,
    /// Proof obligation, ATP, or evidence rejection context.
    ProofFailure,
    /// Verification-condition generation or checking context.
    VerificationCondition,
    /// Algorithm-contract checking context.
    AlgorithmTrace,
    /// General diagnostic context.
    DiagnosticContext,
    /// Developer-only internal explanation data.
    Internal,
}

impl ExplanationKind {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TypeInference => "type_inference",
            Self::ClusterResolution => "cluster_resolution",
            Self::OverloadResolution => "overload_resolution",
            Self::ProofFailure => "proof_failure",
            Self::VerificationCondition => "verification_condition",
            Self::AlgorithmTrace => "algorithm_trace",
            Self::DiagnosticContext => "diagnostic_context",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for ExplanationKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable subject described by an explanation handle.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExplanationSubject {
    /// Diagnostic-level explanation keyed by pre-publication stable fields.
    Diagnostic {
        /// Stable diagnostic code.
        code: DiagnosticCode,
        /// Stable diagnostic detail key.
        stable_detail_key: String,
    },
    /// Phase-owned expression key.
    Expression(String),
    /// Phase-owned verification-condition key.
    VerificationCondition(String),
    /// Source range attached to the explanation.
    SourceRange(SourceRange),
    /// Opaque phase-local subject key.
    PhaseLocal {
        /// Producing pipeline phase.
        phase: PipelinePhase,
        /// Stable phase-local key.
        key: String,
    },
}

/// Backing data reference for lazy explanation resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExplanationSourceRef {
    /// The bounded preview is the only explanation payload known to this crate.
    PreviewOnly,
    /// Explanation data stored in an artifact-owned file.
    Artifact {
        /// Artifact-relative or owner-provided path string.
        path: String,
        /// Expected content hash.
        content_hash: Hash,
    },
    /// Cache-backed query data.
    CacheRecord {
        /// Stable cache key supplied by the owning cache/query layer.
        cache_key: String,
        /// Optional expected content hash.
        content_hash: Option<Hash>,
    },
    /// Later query-service request key.
    QueryService {
        /// Stable service key.
        service_key: String,
        /// Stable query key.
        query_key: String,
    },
}

/// Preview text format.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ExplanationPreviewFormat {
    /// Plain text.
    PlainText,
    /// Markdown-compatible text.
    Markdown,
    /// Deterministic structured text.
    StructuredText,
}

impl ExplanationPreviewFormat {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlainText => "plain_text",
            Self::Markdown => "markdown",
            Self::StructuredText => "structured_text",
        }
    }
}

impl fmt::Display for ExplanationPreviewFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Bounded preview carried by an explanation handle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExplanationPreview {
    format: ExplanationPreviewFormat,
    text: String,
    truncated: bool,
    byte_len: usize,
    line_count: usize,
}

impl ExplanationPreview {
    /// Creates a preview using the default bounds.
    pub fn new(format: ExplanationPreviewFormat, text: impl Into<String>) -> Self {
        Self::with_bounds(
            format,
            text,
            DEFAULT_PREVIEW_MAX_BYTES,
            DEFAULT_PREVIEW_MAX_LINES,
        )
        .expect("default preview bounds are valid")
    }

    /// Creates a preview using explicit bounds.
    pub fn with_bounds(
        format: ExplanationPreviewFormat,
        text: impl Into<String>,
        max_bytes: usize,
        max_lines: usize,
    ) -> Result<Self, ExplanationError> {
        validate_preview_bounds(max_bytes, max_lines)?;
        let (text, truncated) = bounded_text(text.into(), max_bytes, max_lines);
        Ok(Self {
            format,
            byte_len: text.len(),
            line_count: line_count(&text),
            text,
            truncated,
        })
    }

    /// Returns the preview format.
    pub const fn format(&self) -> ExplanationPreviewFormat {
        self.format
    }

    /// Returns bounded preview text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns whether the preview was truncated.
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    /// Returns stored preview bytes.
    pub const fn byte_len(&self) -> usize {
        self.byte_len
    }

    /// Returns stored preview line count.
    pub const fn line_count(&self) -> usize {
        self.line_count
    }
}

/// Input used to create an explanation handle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExplanationHandleInput {
    /// Stable explanation handle id.
    pub id: ExplanationHandleId,
    /// Explanation kind.
    pub kind: ExplanationKind,
    /// Explanation subject.
    pub subject: ExplanationSubject,
    /// Backing source reference.
    pub source: ExplanationSourceRef,
    /// Snapshot precondition.
    pub required_snapshot: Option<BuildSnapshotId>,
    /// Artifact hash precondition.
    pub required_artifact_hash: Option<Hash>,
    /// Canonical summary or backing descriptor hash.
    pub summary_hash: Option<Hash>,
    /// Optional bounded preview.
    pub preview: Option<ExplanationPreview>,
}

/// Compact lazy explanation handle attached to diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExplanationHandle {
    id: ExplanationHandleId,
    kind: ExplanationKind,
    subject: ExplanationSubject,
    source: ExplanationSourceRef,
    required_snapshot: Option<BuildSnapshotId>,
    required_artifact_hash: Option<Hash>,
    summary_hash: Option<Hash>,
    preview: Option<ExplanationPreview>,
}

impl ExplanationHandle {
    /// Creates a validated explanation handle.
    pub fn new(input: ExplanationHandleInput) -> Result<Self, ExplanationError> {
        validate_subject(&input.subject)?;
        validate_source_ref(&input.source)?;
        Ok(Self {
            id: input.id,
            kind: input.kind,
            subject: input.subject,
            source: input.source,
            required_snapshot: input.required_snapshot,
            required_artifact_hash: input.required_artifact_hash,
            summary_hash: input.summary_hash,
            preview: input.preview,
        })
    }

    /// Creates a preview-only diagnostic explanation.
    pub fn preview_only(
        id: ExplanationHandleId,
        code: DiagnosticCode,
        stable_detail_key: impl Into<String>,
        preview: Option<ExplanationPreview>,
    ) -> Result<Self, ExplanationError> {
        Self::new(ExplanationHandleInput {
            id,
            kind: ExplanationKind::DiagnosticContext,
            subject: ExplanationSubject::Diagnostic {
                code,
                stable_detail_key: stable_detail_key.into(),
            },
            source: ExplanationSourceRef::PreviewOnly,
            required_snapshot: None,
            required_artifact_hash: None,
            summary_hash: None,
            preview,
        })
    }

    /// Returns stable handle id.
    pub const fn id(&self) -> &ExplanationHandleId {
        &self.id
    }

    /// Returns explanation kind.
    pub const fn kind(&self) -> ExplanationKind {
        self.kind
    }

    /// Returns explanation subject.
    pub const fn subject(&self) -> &ExplanationSubject {
        &self.subject
    }

    /// Returns source reference.
    pub const fn source(&self) -> &ExplanationSourceRef {
        &self.source
    }

    /// Returns required snapshot.
    pub const fn required_snapshot(&self) -> Option<BuildSnapshotId> {
        self.required_snapshot
    }

    /// Returns required artifact hash.
    pub const fn required_artifact_hash(&self) -> Option<Hash> {
        self.required_artifact_hash
    }

    /// Returns summary hash.
    pub const fn summary_hash(&self) -> Option<Hash> {
        self.summary_hash
    }

    /// Returns bounded preview.
    pub const fn preview(&self) -> Option<&ExplanationPreview> {
        self.preview.as_ref()
    }

    /// Returns a deterministic debug/test snapshot without a published handle.
    pub fn debug_snapshot(&self) -> String {
        self.debug_snapshot_with_diagnostic("unpublished")
    }

    /// Returns a deterministic debug/test snapshot with a caller-supplied
    /// diagnostic identity projection.
    pub fn debug_snapshot_with_diagnostic(&self, diagnostic: &str) -> String {
        let lines = vec![
            "kind=explanation".to_owned(),
            format!("id={:?}", self.id.identity),
            format!("diagnostic={diagnostic}"),
            format!("explanation_kind={}", self.kind),
            format!("subject={}", render_subject(&self.subject)),
            format!("source={}", render_source_ref(&self.source)),
            format!(
                "required_snapshot={}",
                self.required_snapshot
                    .map_or_else(|| "none".to_owned(), render_snapshot)
            ),
            format!(
                "required_artifact_hash={}",
                self.required_artifact_hash
                    .map_or_else(|| "none".to_owned(), render_hash)
            ),
            format!(
                "summary_hash={}",
                self.summary_hash
                    .map_or_else(|| "none".to_owned(), render_hash)
            ),
            format!("preview={}", render_preview(self.preview.as_ref())),
        ];
        let mut rendered = lines.join("\n");
        rendered.push('\n');
        rendered
    }

    pub(crate) fn canonical_key(&self) -> ExplanationHandleKey {
        ExplanationHandleKey {
            id: self.id.identity.clone(),
            kind: self.kind,
            subject: subject_key(&self.subject),
            source: source_ref_key(&self.source),
            required_snapshot: self.required_snapshot.map(render_snapshot),
            required_artifact_hash: self.required_artifact_hash.map(render_hash),
            summary_hash: self.summary_hash.map(render_hash),
        }
    }
}

/// Bounded resolved explanation payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExplanationPayload {
    format: ExplanationPreviewFormat,
    text: String,
    truncated: bool,
    summary_hash: Option<Hash>,
}

impl ExplanationPayload {
    /// Creates a payload from bounded preview text.
    pub fn from_preview(preview: ExplanationPreview, summary_hash: Option<Hash>) -> Self {
        Self {
            format: preview.format,
            text: preview.text,
            truncated: preview.truncated,
            summary_hash,
        }
    }

    /// Creates a payload with explicit bounds.
    pub fn with_bounds(
        format: ExplanationPreviewFormat,
        text: impl Into<String>,
        summary_hash: Option<Hash>,
        max_bytes: usize,
        max_lines: usize,
    ) -> Result<Self, ExplanationError> {
        let preview = ExplanationPreview::with_bounds(format, text, max_bytes, max_lines)?;
        Ok(Self::from_preview(preview, summary_hash))
    }

    /// Returns payload format.
    pub const fn format(&self) -> ExplanationPreviewFormat {
        self.format
    }

    /// Returns bounded payload text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns whether the payload was truncated.
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    /// Returns payload summary hash.
    pub const fn summary_hash(&self) -> Option<Hash> {
        self.summary_hash
    }
}

/// Lazy resolution result.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExplanationResolution {
    /// Bounded explanation payload is available.
    Available(ExplanationPayload),
    /// Backing data is missing.
    Missing {
        /// Missing reason.
        reason: ExplanationMissingReason,
    },
    /// Handle belongs to a stale snapshot.
    Stale {
        /// Snapshot that owns the handle.
        source_snapshot: BuildSnapshotId,
        /// Current snapshot supplied by the caller.
        current_snapshot: BuildSnapshotId,
    },
    /// Backing data exists but cannot be safely used.
    Unavailable {
        /// Unavailability reason.
        reason: String,
    },
}

/// Missing explanation reason.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ExplanationMissingReason {
    /// No preview exists for a preview-only handle.
    PreviewUnavailable,
    /// Backing data was not registered in this store.
    BackingDataMissing,
}

impl ExplanationMissingReason {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreviewUnavailable => "preview_unavailable",
            Self::BackingDataMissing => "backing_data_missing",
        }
    }
}

/// In-memory bounded explanation store.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExplanationStore {
    payloads: BTreeMap<ExplanationHandleKey, ExplanationPayload>,
}

impl ExplanationStore {
    /// Creates an empty explanation store.
    pub const fn new() -> Self {
        Self {
            payloads: BTreeMap::new(),
        }
    }

    /// Registers bounded backing data for a handle.
    pub fn insert_payload(
        &mut self,
        handle: &ExplanationHandle,
        payload: ExplanationPayload,
    ) -> Option<ExplanationPayload> {
        self.payloads.insert(handle.canonical_key(), payload)
    }

    /// Resolves a handle against optional current snapshot freshness.
    pub fn resolve(
        &self,
        handle: &ExplanationHandle,
        current_snapshot: Option<BuildSnapshotId>,
    ) -> ExplanationResolution {
        if let (Some(source_snapshot), Some(current_snapshot)) =
            (handle.required_snapshot, current_snapshot)
            && source_snapshot != current_snapshot
        {
            return ExplanationResolution::Stale {
                source_snapshot,
                current_snapshot,
            };
        }

        if matches!(handle.source, ExplanationSourceRef::PreviewOnly) {
            return handle.preview.clone().map_or(
                ExplanationResolution::Missing {
                    reason: ExplanationMissingReason::PreviewUnavailable,
                },
                |preview| {
                    ExplanationResolution::Available(ExplanationPayload::from_preview(
                        preview,
                        handle.summary_hash,
                    ))
                },
            );
        }

        let Some(payload) = self.payloads.get(&handle.canonical_key()) else {
            return ExplanationResolution::Missing {
                reason: ExplanationMissingReason::BackingDataMissing,
            };
        };
        if let Some(expected) = handle.summary_hash
            && payload.summary_hash != Some(expected)
        {
            return ExplanationResolution::Unavailable {
                reason: "summary_hash_mismatch".to_owned(),
            };
        }
        ExplanationResolution::Available(payload.clone())
    }

    /// Returns whether no bounded payloads are registered.
    pub fn is_empty(&self) -> bool {
        self.payloads.is_empty()
    }

    /// Returns registered bounded payload count.
    pub fn len(&self) -> usize {
        self.payloads.len()
    }
}

/// Error returned while constructing explanation data.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ExplanationError {
    /// Explanation handle id was malformed.
    InvalidHandleId {
        /// Rejected identity.
        identity: String,
    },
    /// Stable subject key was malformed.
    InvalidSubjectKey {
        /// Rejected key.
        key: String,
    },
    /// Source range had `start > end`.
    InvalidRange {
        /// Range start.
        start: usize,
        /// Range end.
        end: usize,
    },
    /// Artifact path was empty.
    EmptyArtifactPath,
    /// Cache key was malformed.
    InvalidCacheKey {
        /// Rejected key.
        key: String,
    },
    /// Query service key was malformed.
    InvalidServiceKey {
        /// Rejected key.
        key: String,
    },
    /// Query key was malformed.
    InvalidQueryKey {
        /// Rejected key.
        key: String,
    },
    /// Preview byte bound cannot hold a truncation marker.
    InvalidPreviewByteBound {
        /// Rejected maximum bytes.
        max_bytes: usize,
    },
    /// Preview line bound must be positive.
    InvalidPreviewLineBound,
}

impl fmt::Display for ExplanationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandleId { identity } => {
                write!(formatter, "invalid explanation handle id `{identity}`")
            }
            Self::InvalidSubjectKey { key } => {
                write!(formatter, "invalid explanation subject key `{key}`")
            }
            Self::InvalidRange { start, end } => {
                write!(
                    formatter,
                    "explanation source range start {start} exceeds end {end}"
                )
            }
            Self::EmptyArtifactPath => formatter.write_str("explanation artifact path is empty"),
            Self::InvalidCacheKey { key } => {
                write!(formatter, "invalid explanation cache key `{key}`")
            }
            Self::InvalidServiceKey { key } => {
                write!(formatter, "invalid explanation service key `{key}`")
            }
            Self::InvalidQueryKey { key } => {
                write!(formatter, "invalid explanation query key `{key}`")
            }
            Self::InvalidPreviewByteBound { max_bytes } => write!(
                formatter,
                "explanation preview byte bound {max_bytes} cannot hold truncation marker"
            ),
            Self::InvalidPreviewLineBound => {
                formatter.write_str("explanation preview line bound must be positive")
            }
        }
    }
}

impl Error for ExplanationError {}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ExplanationHandleKey {
    id: String,
    kind: ExplanationKind,
    subject: ExplanationSubjectKey,
    source: ExplanationSourceKey,
    required_snapshot: Option<String>,
    required_artifact_hash: Option<String>,
    summary_hash: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ExplanationSubjectKey {
    Diagnostic {
        code: DiagnosticCode,
        stable_detail_key: String,
    },
    Expression(String),
    VerificationCondition(String),
    SourceRange {
        source: String,
        start: usize,
        end: usize,
    },
    PhaseLocal {
        phase: PipelinePhase,
        key: String,
    },
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ExplanationSourceKey {
    PreviewOnly,
    Artifact {
        path: String,
        content_hash: String,
    },
    CacheRecord {
        cache_key: String,
        content_hash: Option<String>,
    },
    QueryService {
        service_key: String,
        query_key: String,
    },
}

fn validate_subject(subject: &ExplanationSubject) -> Result<(), ExplanationError> {
    match subject {
        ExplanationSubject::Diagnostic {
            stable_detail_key, ..
        }
        | ExplanationSubject::Expression(stable_detail_key)
        | ExplanationSubject::VerificationCondition(stable_detail_key) => {
            validate_identity(stable_detail_key).map_err(|_| ExplanationError::InvalidSubjectKey {
                key: stable_detail_key.clone(),
            })
        }
        ExplanationSubject::SourceRange(range) => validate_range(*range),
        ExplanationSubject::PhaseLocal { key, .. } => validate_identity(key)
            .map_err(|_| ExplanationError::InvalidSubjectKey { key: key.clone() }),
    }
}

fn validate_source_ref(source: &ExplanationSourceRef) -> Result<(), ExplanationError> {
    match source {
        ExplanationSourceRef::PreviewOnly => Ok(()),
        ExplanationSourceRef::Artifact { path, .. } => {
            if path.is_empty() {
                Err(ExplanationError::EmptyArtifactPath)
            } else {
                Ok(())
            }
        }
        ExplanationSourceRef::CacheRecord { cache_key, .. } => validate_identity(cache_key)
            .map_err(|_| ExplanationError::InvalidCacheKey {
                key: cache_key.clone(),
            }),
        ExplanationSourceRef::QueryService {
            service_key,
            query_key,
        } => {
            validate_identity(service_key).map_err(|_| ExplanationError::InvalidServiceKey {
                key: service_key.clone(),
            })?;
            validate_identity(query_key).map_err(|_| ExplanationError::InvalidQueryKey {
                key: query_key.clone(),
            })
        }
    }
}

fn validate_identity(identity: &str) -> Result<(), ()> {
    if identity.is_empty() {
        return Err(());
    }
    for segment in identity.split('.') {
        validate_identity_segment(segment)?;
    }
    Ok(())
}

fn validate_identity_segment(segment: &str) -> Result<(), ()> {
    let bytes = segment.as_bytes();
    let Some((&first, rest)) = bytes.split_first() else {
        return Err(());
    };
    if !first.is_ascii_lowercase() {
        return Err(());
    }

    let mut previous_underscore = false;
    for byte in rest {
        if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            previous_underscore = false;
        } else if *byte == b'_' {
            if previous_underscore {
                return Err(());
            }
            previous_underscore = true;
        } else {
            return Err(());
        }
    }

    if previous_underscore {
        return Err(());
    }
    Ok(())
}

fn validate_range(range: SourceRange) -> Result<(), ExplanationError> {
    if range.start > range.end {
        return Err(ExplanationError::InvalidRange {
            start: range.start,
            end: range.end,
        });
    }
    Ok(())
}

fn validate_preview_bounds(max_bytes: usize, max_lines: usize) -> Result<(), ExplanationError> {
    if max_bytes < TRUNCATION_MARKER.len() {
        return Err(ExplanationError::InvalidPreviewByteBound { max_bytes });
    }
    if max_lines == 0 {
        return Err(ExplanationError::InvalidPreviewLineBound);
    }
    Ok(())
}

fn bounded_text(text: String, max_bytes: usize, max_lines: usize) -> (String, bool) {
    let mut truncated = false;
    let mut bounded = text;
    if line_count(&bounded) > max_lines {
        bounded = bounded
            .split('\n')
            .take(max_lines)
            .collect::<Vec<_>>()
            .join("\n");
        truncated = true;
    }
    if bounded.len() > max_bytes {
        truncated = true;
    }
    if truncated {
        let body_limit = max_bytes - TRUNCATION_MARKER.len();
        bounded = truncate_to_char_boundary(&bounded, body_limit);
        bounded.push_str(TRUNCATION_MARKER);
    }
    (bounded, truncated)
}

fn truncate_to_char_boundary(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}

fn line_count(text: &str) -> usize {
    if text.is_empty() {
        0
    } else {
        text.split('\n').count()
    }
}

fn subject_key(subject: &ExplanationSubject) -> ExplanationSubjectKey {
    match subject {
        ExplanationSubject::Diagnostic {
            code,
            stable_detail_key,
        } => ExplanationSubjectKey::Diagnostic {
            code: *code,
            stable_detail_key: stable_detail_key.clone(),
        },
        ExplanationSubject::Expression(key) => ExplanationSubjectKey::Expression(key.clone()),
        ExplanationSubject::VerificationCondition(key) => {
            ExplanationSubjectKey::VerificationCondition(key.clone())
        }
        ExplanationSubject::SourceRange(range) => ExplanationSubjectKey::SourceRange {
            source: source_key(*range),
            start: range.start,
            end: range.end,
        },
        ExplanationSubject::PhaseLocal { phase, key } => ExplanationSubjectKey::PhaseLocal {
            phase: *phase,
            key: key.clone(),
        },
    }
}

fn source_ref_key(source: &ExplanationSourceRef) -> ExplanationSourceKey {
    match source {
        ExplanationSourceRef::PreviewOnly => ExplanationSourceKey::PreviewOnly,
        ExplanationSourceRef::Artifact { path, content_hash } => ExplanationSourceKey::Artifact {
            path: path.clone(),
            content_hash: render_hash(*content_hash),
        },
        ExplanationSourceRef::CacheRecord {
            cache_key,
            content_hash,
        } => ExplanationSourceKey::CacheRecord {
            cache_key: cache_key.clone(),
            content_hash: content_hash.map(render_hash),
        },
        ExplanationSourceRef::QueryService {
            service_key,
            query_key,
        } => ExplanationSourceKey::QueryService {
            service_key: service_key.clone(),
            query_key: query_key.clone(),
        },
    }
}

fn render_subject(subject: &ExplanationSubject) -> String {
    match subject {
        ExplanationSubject::Diagnostic {
            code,
            stable_detail_key,
        } => format!("diagnostic(code={code},stable_detail_key={stable_detail_key:?})"),
        ExplanationSubject::Expression(key) => format!("expression({key:?})"),
        ExplanationSubject::VerificationCondition(key) => {
            format!("verification_condition({key:?})")
        }
        ExplanationSubject::SourceRange(range) => format!(
            "source_range({}:{}..{})",
            source_key(*range),
            range.start,
            range.end
        ),
        ExplanationSubject::PhaseLocal { phase, key } => {
            format!("phase_local(phase={phase},key={key:?})")
        }
    }
}

fn render_source_ref(source: &ExplanationSourceRef) -> String {
    match source {
        ExplanationSourceRef::PreviewOnly => "preview_only".to_owned(),
        ExplanationSourceRef::Artifact { path, content_hash } => {
            format!(
                "artifact(path={path:?},content_hash={})",
                render_hash(*content_hash)
            )
        }
        ExplanationSourceRef::CacheRecord {
            cache_key,
            content_hash,
        } => format!(
            "cache_record(cache_key={cache_key:?},content_hash={})",
            content_hash.map_or_else(|| "none".to_owned(), render_hash)
        ),
        ExplanationSourceRef::QueryService {
            service_key,
            query_key,
        } => format!("query_service(service_key={service_key:?},query_key={query_key:?})"),
    }
}

fn render_preview(preview: Option<&ExplanationPreview>) -> String {
    preview.map_or_else(
        || "none".to_owned(),
        |preview| {
            format!(
                "{{format={},text={:?},truncated={},byte_len={},line_count={}}}",
                preview.format,
                preview.text,
                preview.truncated,
                preview.byte_len,
                preview.line_count
            )
        },
    )
}

fn source_key(range: SourceRange) -> String {
    range
        .source_id
        .to_published_schema_string()
        .unwrap_or_else(|_| format!("{:?}", range.source_id))
}

fn render_snapshot(snapshot: BuildSnapshotId) -> String {
    snapshot
        .to_published_schema_string()
        .unwrap_or_else(|_| format!("{snapshot:?}"))
}

fn render_hash(hash: Hash) -> String {
    let mut rendered = String::with_capacity(Hash::BYTE_LEN * 2);
    for byte in hash.as_bytes() {
        rendered.push_str(&format!("{byte:02x}"));
    }
    rendered
}
