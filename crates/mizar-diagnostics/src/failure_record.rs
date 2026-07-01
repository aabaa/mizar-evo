//! Structured diagnostic drafts and records.

use std::{cmp::Ordering, collections::BTreeMap, error::Error, fmt};

use mizar_session::{BuildSnapshotId, SourceRange};

use crate::fix::FixSuggestion;
use crate::registry::{DiagnosticCode, DiagnosticRegistry, DiagnosticSeverity, DiagnosticStatus};

/// Pipeline phase that produced a diagnostic.
///
/// This is provenance and ordering metadata only; it does not decide phase
/// success, proof acceptance, or scheduler state.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum PipelinePhase {
    /// Lexical preprocessing or tokenization.
    Lexer,
    /// Concrete syntax parsing and recovery.
    Parser,
    /// Frontend lowering before shared resolution.
    Frontend,
    /// Name, import, namespace, or symbol resolution.
    Resolver,
    /// Type, mode, attribute, and registration checking.
    Type,
    /// Overload or template resolution.
    Overload,
    /// Proof search, obligation discharge, or ATP interaction.
    Proof,
    /// Kernel replay or evidence checking.
    Kernel,
    /// Verification-condition generation or checking.
    VerificationCondition,
    /// Algorithm and contract checking.
    Algorithm,
    /// Compatibility and package diagnostics.
    Compatibility,
    /// Build/session infrastructure diagnostics.
    Build,
    /// Developer-only internal diagnostics.
    Internal,
}

impl PipelinePhase {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lexer => "lexer",
            Self::Parser => "parser",
            Self::Frontend => "frontend",
            Self::Resolver => "resolver",
            Self::Type => "type",
            Self::Overload => "overload",
            Self::Proof => "proof",
            Self::Kernel => "kernel",
            Self::VerificationCondition => "verification_condition",
            Self::Algorithm => "algorithm",
            Self::Compatibility => "compatibility",
            Self::Build => "build",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for PipelinePhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Stable machine-readable failure classification.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum FailureCategory {
    /// Lexical, syntactic, or parse-recovery failure.
    ParseError,
    /// Name, import, namespace, or symbol resolution failure.
    ResolveError,
    /// Type, attribute, mode, or registration mismatch.
    TypeError,
    /// Overload/template ambiguity or no viable overload.
    OverloadAmbiguity,
    /// Cluster, attribute, or registration cycle.
    ClusterLoop,
    /// ATP timeout or resource exhaustion.
    AtpTimeout,
    /// Malformed, unsupported, or policy-rejected evidence envelope.
    CertificateRejection,
    /// Kernel-level evidence or replay rejection.
    KernelRejection,
    /// Logical inconsistency or VC failure not classified above.
    LogicFailure,
    /// Compatibility or packaging warning.
    CompatibilityWarning,
    /// Informational display diagnostic after `I` codes are allocated.
    Informational,
    /// Developer-mode internal diagnostic that is not normal user output.
    InternalInvariant,
}

impl FailureCategory {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ParseError => "parse_error",
            Self::ResolveError => "resolve_error",
            Self::TypeError => "type_error",
            Self::OverloadAmbiguity => "overload_ambiguity",
            Self::ClusterLoop => "cluster_loop",
            Self::AtpTimeout => "atp_timeout",
            Self::CertificateRejection => "certificate_rejection",
            Self::KernelRejection => "kernel_rejection",
            Self::LogicFailure => "logic_failure",
            Self::CompatibilityWarning => "compatibility_warning",
            Self::Informational => "informational",
            Self::InternalInvariant => "internal_invariant",
        }
    }
}

impl fmt::Display for FailureCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Snapshot-scoped diagnostic id assigned by aggregation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DiagnosticId(u64);

impl DiagnosticId {
    /// Creates a snapshot-scoped diagnostic id from an aggregation ordinal.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw snapshot-local ordinal.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Diagnostic handle scoped to one build snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticHandle {
    /// Build snapshot that owns this handle.
    pub snapshot: BuildSnapshotId,
    /// Snapshot-local diagnostic id.
    pub id: DiagnosticId,
}

impl DiagnosticHandle {
    /// Creates a diagnostic handle.
    pub const fn new(snapshot: BuildSnapshotId, id: DiagnosticId) -> Self {
        Self { snapshot, id }
    }
}

/// Reason a diagnostic or span is stale relative to another snapshot.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum StaleDiagnosticReason {
    /// The source text changed after the diagnostic was produced.
    SourceEdited,
    /// The source disappeared after the diagnostic was produced.
    SourceRemoved,
    /// A newer build snapshot superseded the producer snapshot.
    SnapshotSuperseded,
    /// The producer cache entry is obsolete.
    ProducerCacheObsolete,
    /// The diagnostic was replayed from historical data.
    HistoricalReplay,
}

impl StaleDiagnosticReason {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceEdited => "source_edited",
            Self::SourceRemoved => "source_removed",
            Self::SnapshotSuperseded => "snapshot_superseded",
            Self::ProducerCacheObsolete => "producer_cache_obsolete",
            Self::HistoricalReplay => "historical_replay",
        }
    }
}

impl fmt::Display for StaleDiagnosticReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Freshness of a published diagnostic record.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticFreshness {
    /// Diagnostic was produced for the publication snapshot.
    Current {
        /// Snapshot observed by the producer.
        source_snapshot: BuildSnapshotId,
    },
    /// Diagnostic was produced for an older snapshot and is not current output.
    Stale {
        /// Snapshot observed by the producer.
        source_snapshot: BuildSnapshotId,
        /// Current publication snapshot.
        current_snapshot: BuildSnapshotId,
        /// Why the diagnostic is stale.
        reason: StaleDiagnosticReason,
    },
    /// Diagnostic was loaded from historical artifact/cache/log data.
    Historical {
        /// Snapshot associated with the historical record.
        source_snapshot: BuildSnapshotId,
        /// Optional artifact hash that carried the record.
        artifact_hash: Option<String>,
    },
}

impl DiagnosticFreshness {
    /// Returns the source snapshot carried by this freshness state.
    pub const fn source_snapshot(&self) -> BuildSnapshotId {
        match self {
            Self::Current { source_snapshot }
            | Self::Stale {
                source_snapshot, ..
            }
            | Self::Historical {
                source_snapshot, ..
            } => *source_snapshot,
        }
    }

    /// Returns whether this freshness state is current publication output.
    pub const fn is_current(&self) -> bool {
        matches!(self, Self::Current { .. })
    }
}

/// Role attached to a diagnostic span.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum DiagnosticSpanRole {
    /// Main diagnostic location.
    Primary,
    /// Supporting diagnostic location.
    Secondary,
    /// Definition-site location.
    DefinitionSite,
    /// Related location.
    Related,
}

impl DiagnosticSpanRole {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::DefinitionSite => "definition_site",
            Self::Related => "related",
        }
    }
}

impl fmt::Display for DiagnosticSpanRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Freshness of an individual span relative to the source snapshot it names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SpanFreshness {
    /// Span is current for its source snapshot.
    Current,
    /// Span was copied from stale context.
    Stale {
        /// Why the span is stale.
        reason: StaleDiagnosticReason,
    },
    /// Span came from historical context.
    Historical,
}

/// Intent for a zero-width diagnostic span.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ZeroWidthSpanIntent {
    /// Span anchors an end-of-file diagnostic.
    Eof,
    /// Span anchors an insertion-point diagnostic.
    InsertionPoint,
}

impl ZeroWidthSpanIntent {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eof => "eof",
            Self::InsertionPoint => "insertion_point",
        }
    }
}

/// Source span attached to a diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticSpan {
    range: SourceRange,
    role: DiagnosticSpanRole,
    label: Option<String>,
    freshness: SpanFreshness,
    zero_width: Option<ZeroWidthSpanIntent>,
}

impl DiagnosticSpan {
    /// Creates a validated diagnostic span.
    pub fn new(
        range: SourceRange,
        role: DiagnosticSpanRole,
        label: Option<String>,
        freshness: SpanFreshness,
        zero_width: Option<ZeroWidthSpanIntent>,
    ) -> Result<Self, DiagnosticRecordError> {
        validate_source_range(range)?;
        match (range.start == range.end, zero_width) {
            (true, None) => {
                return Err(DiagnosticRecordError::ZeroWidthIntentRequired {
                    offset: range.start,
                });
            }
            (false, Some(intent)) => {
                return Err(DiagnosticRecordError::ZeroWidthIntentOnNonZeroRange {
                    start: range.start,
                    end: range.end,
                    intent,
                });
            }
            _ => {}
        }

        Ok(Self {
            range,
            role,
            label,
            freshness,
            zero_width,
        })
    }

    /// Creates a current non-zero primary span.
    pub fn primary(
        range: SourceRange,
        label: Option<String>,
    ) -> Result<Self, DiagnosticRecordError> {
        Self::new(
            range,
            DiagnosticSpanRole::Primary,
            label,
            SpanFreshness::Current,
            None,
        )
    }

    /// Creates a current non-zero secondary span.
    pub fn secondary(
        range: SourceRange,
        label: Option<String>,
    ) -> Result<Self, DiagnosticRecordError> {
        Self::new(
            range,
            DiagnosticSpanRole::Secondary,
            label,
            SpanFreshness::Current,
            None,
        )
    }

    /// Returns the source range.
    pub const fn range(&self) -> SourceRange {
        self.range
    }

    /// Returns the span role.
    pub const fn role(&self) -> DiagnosticSpanRole {
        self.role
    }

    /// Returns the optional human-facing label.
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns span freshness.
    pub const fn freshness(&self) -> SpanFreshness {
        self.freshness
    }

    /// Returns zero-width intent, if any.
    pub const fn zero_width(&self) -> Option<ZeroWidthSpanIntent> {
        self.zero_width
    }
}

/// Human-facing note attached to a diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticNote {
    kind: DiagnosticNoteKind,
    message: String,
    span: Option<DiagnosticSpan>,
}

impl DiagnosticNote {
    /// Creates a diagnostic note.
    pub fn new(
        kind: DiagnosticNoteKind,
        message: impl Into<String>,
        span: Option<DiagnosticSpan>,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            span,
        }
    }

    /// Returns the note kind.
    pub const fn kind(&self) -> DiagnosticNoteKind {
        self.kind
    }

    /// Returns the human-facing note message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the optional source span.
    pub const fn span(&self) -> Option<&DiagnosticSpan> {
        self.span.as_ref()
    }
}

/// Diagnostic note kind.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum DiagnosticNoteKind {
    /// Additional context.
    Note,
    /// Human-facing help text.
    Help,
    /// Cause of the diagnostic.
    Cause,
    /// Related context.
    Related,
}

impl DiagnosticNoteKind {
    /// Stable lowercase rendering for debug/test snapshots.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Help => "help",
            Self::Cause => "cause",
            Self::Related => "related",
        }
    }
}

/// Structured machine-readable detail map.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiagnosticDetails {
    entries: BTreeMap<String, DiagnosticDetailValue>,
}

impl DiagnosticDetails {
    /// Creates an empty detail map.
    pub const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Creates a detail map from validated entries.
    pub fn from_entries<K, I>(entries: I) -> Result<Self, DiagnosticRecordError>
    where
        K: Into<String>,
        I: IntoIterator<Item = (K, DiagnosticDetailValue)>,
    {
        let mut details = Self::new();
        for (key, value) in entries {
            details.insert(key, value)?;
        }
        Ok(details)
    }

    /// Inserts a validated detail entry.
    pub fn insert(
        &mut self,
        key: impl Into<String>,
        value: DiagnosticDetailValue,
    ) -> Result<Option<DiagnosticDetailValue>, DiagnosticRecordError> {
        let key = key.into();
        validate_detail_key(&key)
            .map_err(|_| DiagnosticRecordError::InvalidDetailKey { key: key.clone() })?;
        Ok(self.entries.insert(key, value))
    }

    /// Returns the detail entries in deterministic key order.
    pub const fn entries(&self) -> &BTreeMap<String, DiagnosticDetailValue> {
        &self.entries
    }

    /// Returns whether the detail map is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Structured diagnostic detail value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticDetailValue {
    /// String payload.
    String(String),
    /// Integer payload.
    Integer(i64),
    /// Boolean payload.
    Boolean(bool),
    /// Diagnostic-code payload.
    Code(DiagnosticCode),
    /// Source-range payload.
    Source(SourceRange),
    /// Nested list payload.
    List(Vec<DiagnosticDetailValue>),
}

impl Ord for DiagnosticDetailValue {
    fn cmp(&self, other: &Self) -> Ordering {
        variant_order(self)
            .cmp(&variant_order(other))
            .then_with(|| match (self, other) {
                (Self::Boolean(left), Self::Boolean(right)) => left.cmp(right),
                (Self::Integer(left), Self::Integer(right)) => left.cmp(right),
                (Self::String(left), Self::String(right)) => left.cmp(right),
                (Self::Code(left), Self::Code(right)) => left.cmp(right),
                (Self::Source(left), Self::Source(right)) => compare_source_ranges(*left, *right),
                (Self::List(left), Self::List(right)) => left.cmp(right),
                _ => Ordering::Equal,
            })
    }
}

impl PartialOrd for DiagnosticDetailValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Opaque identity of a lazy explanation.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExplanationRef {
    identity: String,
}

impl ExplanationRef {
    /// Creates an explanation reference identity.
    pub fn new(identity: impl Into<String>) -> Result<Self, DiagnosticRecordError> {
        let identity = identity.into();
        validate_detail_key(&identity).map_err(|_| {
            DiagnosticRecordError::InvalidExplanationIdentity {
                identity: identity.clone(),
            }
        })?;
        Ok(Self { identity })
    }

    /// Returns the stable explanation identity.
    pub fn identity(&self) -> &str {
        &self.identity
    }
}

/// Input used to create a validated diagnostic draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticDraftInput {
    /// Source snapshot observed by the producer.
    pub source_snapshot: BuildSnapshotId,
    /// Stable diagnostic code.
    pub code: DiagnosticCode,
    /// Producing pipeline phase.
    pub phase: PipelinePhase,
    /// Stable failure category.
    pub category: FailureCategory,
    /// Stable detail key, independent of message text.
    pub stable_detail_key: String,
    /// Human-facing message.
    pub message: String,
    /// Primary source span.
    pub primary_span: DiagnosticSpan,
    /// Secondary source spans.
    pub secondary_spans: Vec<DiagnosticSpan>,
    /// Human-facing notes.
    pub notes: Vec<DiagnosticNote>,
    /// Structured details.
    pub details: DiagnosticDetails,
    /// Structured fix suggestions.
    pub fixes: Vec<FixSuggestion>,
    /// Optional lazy explanation reference.
    pub explanation: Option<ExplanationRef>,
}

/// Producer-owned diagnostic draft before aggregation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticDraft {
    source_snapshot: BuildSnapshotId,
    code: DiagnosticCode,
    phase: PipelinePhase,
    category: FailureCategory,
    stable_detail_key: String,
    message: String,
    primary_span: DiagnosticSpan,
    secondary_spans: Vec<DiagnosticSpan>,
    notes: Vec<DiagnosticNote>,
    details: DiagnosticDetails,
    fixes: Vec<FixSuggestion>,
    explanation: Option<ExplanationRef>,
}

impl DiagnosticDraft {
    /// Creates a validated diagnostic draft against the built-in registry.
    pub fn new(input: DiagnosticDraftInput) -> Result<Self, DiagnosticRecordError> {
        Self::new_with_registry(input, DiagnosticRegistry::builtin())
    }

    /// Creates a validated diagnostic draft against a validated registry.
    pub fn new_with_registry(
        input: DiagnosticDraftInput,
        registry: DiagnosticRegistry<'_>,
    ) -> Result<Self, DiagnosticRecordError> {
        let Some(descriptor) = registry.lookup(input.code) else {
            return Err(DiagnosticRecordError::UnknownDiagnosticCode { code: input.code });
        };
        if descriptor.status == DiagnosticStatus::Retired {
            return Err(DiagnosticRecordError::RetiredDescriptorForDraft { code: input.code });
        }
        validate_detail_key(&input.stable_detail_key).map_err(|_| {
            DiagnosticRecordError::InvalidStableDetailKey {
                key: input.stable_detail_key.clone(),
            }
        })?;
        validate_primary_and_secondary_spans(&input.primary_span, &input.secondary_spans)?;

        Ok(Self {
            source_snapshot: input.source_snapshot,
            code: input.code,
            phase: input.phase,
            category: input.category,
            stable_detail_key: input.stable_detail_key,
            message: input.message,
            primary_span: input.primary_span,
            secondary_spans: input.secondary_spans,
            notes: input.notes,
            details: input.details,
            fixes: normalize_fixes(input.fixes),
            explanation: input.explanation,
        })
    }

    /// Returns the source snapshot observed by the producer.
    pub const fn source_snapshot(&self) -> BuildSnapshotId {
        self.source_snapshot
    }

    /// Returns the stable diagnostic code.
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    /// Returns the producing phase.
    pub const fn phase(&self) -> PipelinePhase {
        self.phase
    }

    /// Returns the stable failure category.
    pub const fn category(&self) -> FailureCategory {
        self.category
    }

    /// Returns the stable detail key.
    pub fn stable_detail_key(&self) -> &str {
        &self.stable_detail_key
    }

    /// Returns the human-facing message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the primary source span.
    pub const fn primary_span(&self) -> &DiagnosticSpan {
        &self.primary_span
    }

    /// Returns secondary source spans.
    pub fn secondary_spans(&self) -> &[DiagnosticSpan] {
        &self.secondary_spans
    }

    /// Returns diagnostic notes.
    pub fn notes(&self) -> &[DiagnosticNote] {
        &self.notes
    }

    /// Returns structured details.
    pub const fn details(&self) -> &DiagnosticDetails {
        &self.details
    }

    /// Returns structured fix suggestions.
    pub fn fixes(&self) -> &[FixSuggestion] {
        &self.fixes
    }

    /// Returns the optional explanation reference.
    pub const fn explanation(&self) -> Option<&ExplanationRef> {
        self.explanation.as_ref()
    }

    /// Returns a deterministic debug/test snapshot.
    pub fn debug_snapshot(&self) -> String {
        render_debug_snapshot(DebugSnapshot::Draft(self))
    }
}

/// Immutable diagnostic record after aggregation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticRecord {
    handle: DiagnosticHandle,
    code: DiagnosticCode,
    semantic_name: String,
    severity: DiagnosticSeverity,
    phase: PipelinePhase,
    category: FailureCategory,
    stable_detail_key: String,
    message: String,
    primary_span: DiagnosticSpan,
    secondary_spans: Vec<DiagnosticSpan>,
    notes: Vec<DiagnosticNote>,
    details: DiagnosticDetails,
    fixes: Vec<FixSuggestion>,
    explanation: Option<ExplanationRef>,
    related: Vec<DiagnosticHandle>,
    freshness: DiagnosticFreshness,
}

impl DiagnosticRecord {
    /// Creates a diagnostic record by projecting a draft through the built-in registry.
    pub fn from_draft(
        draft: DiagnosticDraft,
        handle: DiagnosticHandle,
        freshness: DiagnosticFreshness,
        related: Vec<DiagnosticHandle>,
    ) -> Result<Self, DiagnosticRecordError> {
        Self::from_draft_with_registry(
            draft,
            DiagnosticRegistry::builtin(),
            handle,
            freshness,
            related,
        )
    }

    /// Creates a diagnostic record by projecting a draft through a validated registry.
    pub fn from_draft_with_registry(
        draft: DiagnosticDraft,
        registry: DiagnosticRegistry<'_>,
        handle: DiagnosticHandle,
        freshness: DiagnosticFreshness,
        related: Vec<DiagnosticHandle>,
    ) -> Result<Self, DiagnosticRecordError> {
        let Some(descriptor) = registry.lookup(draft.code) else {
            return Err(DiagnosticRecordError::UnknownDiagnosticCode { code: draft.code });
        };
        if descriptor.status == DiagnosticStatus::Retired && freshness.is_current() {
            return Err(DiagnosticRecordError::RetiredDescriptorForCurrentRecord {
                code: descriptor.code,
            });
        }
        validate_draft_freshness(draft.source_snapshot, &freshness)?;
        validate_freshness(handle, &freshness)?;
        validate_related_handles(handle, &related)?;

        Ok(Self {
            handle,
            code: draft.code,
            semantic_name: descriptor.semantic_name.to_owned(),
            severity: descriptor.default_severity,
            phase: draft.phase,
            category: draft.category,
            stable_detail_key: draft.stable_detail_key,
            message: draft.message,
            primary_span: draft.primary_span,
            secondary_spans: draft.secondary_spans,
            notes: draft.notes,
            details: draft.details,
            fixes: draft.fixes,
            explanation: draft.explanation,
            related,
            freshness,
        })
    }

    /// Returns the diagnostic handle.
    pub const fn handle(&self) -> DiagnosticHandle {
        self.handle
    }

    /// Returns the stable diagnostic code.
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    /// Returns the semantic descriptor name copied from the registry.
    pub fn semantic_name(&self) -> &str {
        &self.semantic_name
    }

    /// Returns the descriptor severity.
    pub const fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns the producing phase.
    pub const fn phase(&self) -> PipelinePhase {
        self.phase
    }

    /// Returns the stable failure category.
    pub const fn category(&self) -> FailureCategory {
        self.category
    }

    /// Returns the stable detail key.
    pub fn stable_detail_key(&self) -> &str {
        &self.stable_detail_key
    }

    /// Returns the human-facing message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the primary source span.
    pub const fn primary_span(&self) -> &DiagnosticSpan {
        &self.primary_span
    }

    /// Returns secondary source spans.
    pub fn secondary_spans(&self) -> &[DiagnosticSpan] {
        &self.secondary_spans
    }

    /// Returns diagnostic notes.
    pub fn notes(&self) -> &[DiagnosticNote] {
        &self.notes
    }

    /// Returns structured details.
    pub const fn details(&self) -> &DiagnosticDetails {
        &self.details
    }

    /// Returns structured fix suggestions.
    pub fn fixes(&self) -> &[FixSuggestion] {
        &self.fixes
    }

    /// Returns the optional explanation reference.
    pub const fn explanation(&self) -> Option<&ExplanationRef> {
        self.explanation.as_ref()
    }

    /// Returns related handles.
    pub fn related(&self) -> &[DiagnosticHandle] {
        &self.related
    }

    /// Returns diagnostic freshness.
    pub const fn freshness(&self) -> &DiagnosticFreshness {
        &self.freshness
    }

    /// Returns a deterministic debug/test snapshot.
    pub fn debug_snapshot(&self) -> String {
        render_debug_snapshot(DebugSnapshot::Record(self))
    }
}

/// Validation error for diagnostic drafts and records.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticRecordError {
    /// The stable detail key was malformed.
    InvalidStableDetailKey {
        /// Rejected key.
        key: String,
    },
    /// A detail-map key was malformed.
    InvalidDetailKey {
        /// Rejected key.
        key: String,
    },
    /// An explanation identity was malformed.
    InvalidExplanationIdentity {
        /// Rejected identity.
        identity: String,
    },
    /// The diagnostic code is not allocated in the registry.
    UnknownDiagnosticCode {
        /// Unknown diagnostic code.
        code: DiagnosticCode,
    },
    /// A producer draft tried to use a retired code.
    RetiredDescriptorForDraft {
        /// Retired code.
        code: DiagnosticCode,
    },
    /// Source range had `start > end`.
    InvalidRange {
        /// Range start.
        start: usize,
        /// Range end.
        end: usize,
    },
    /// Zero-width range omitted its intent.
    ZeroWidthIntentRequired {
        /// Zero-width offset.
        offset: usize,
    },
    /// Non-zero range carried zero-width intent.
    ZeroWidthIntentOnNonZeroRange {
        /// Range start.
        start: usize,
        /// Range end.
        end: usize,
        /// Rejected intent.
        intent: ZeroWidthSpanIntent,
    },
    /// Primary span did not use the primary role.
    PrimarySpanMustUsePrimaryRole {
        /// Observed role.
        actual: DiagnosticSpanRole,
    },
    /// Secondary spans must not use the primary role.
    SecondarySpanMustNotUsePrimaryRole {
        /// Rejected secondary span index.
        index: usize,
    },
    /// Retired codes cannot be emitted as current records.
    RetiredDescriptorForCurrentRecord {
        /// Retired code.
        code: DiagnosticCode,
    },
    /// Freshness source snapshot did not match the draft source snapshot.
    DraftFreshnessSnapshotMismatch {
        /// Draft source snapshot.
        draft_snapshot: BuildSnapshotId,
        /// Freshness source snapshot.
        freshness_snapshot: BuildSnapshotId,
    },
    /// Current freshness did not match the handle snapshot.
    CurrentFreshnessSnapshotMismatch {
        /// Handle snapshot.
        handle_snapshot: BuildSnapshotId,
        /// Source snapshot.
        source_snapshot: BuildSnapshotId,
    },
    /// Stale freshness current snapshot did not match the handle snapshot.
    StaleFreshnessSnapshotMismatch {
        /// Handle snapshot.
        handle_snapshot: BuildSnapshotId,
        /// Current snapshot.
        current_snapshot: BuildSnapshotId,
    },
    /// Stale freshness used identical source and current snapshots.
    StaleFreshnessNotStale {
        /// Source snapshot.
        source_snapshot: BuildSnapshotId,
        /// Current snapshot.
        current_snapshot: BuildSnapshotId,
    },
    /// Historical freshness did not match the handle snapshot.
    HistoricalFreshnessSnapshotMismatch {
        /// Handle snapshot.
        handle_snapshot: BuildSnapshotId,
        /// Source snapshot.
        source_snapshot: BuildSnapshotId,
    },
    /// Related handle referenced a different snapshot.
    RelatedHandleSnapshotMismatch {
        /// Rejected related handle index.
        index: usize,
        /// Record handle snapshot.
        handle_snapshot: BuildSnapshotId,
        /// Related handle snapshot.
        related_snapshot: BuildSnapshotId,
    },
}

impl fmt::Display for DiagnosticRecordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStableDetailKey { key } => {
                write!(formatter, "invalid stable detail key `{key}`")
            }
            Self::InvalidDetailKey { key } => write!(formatter, "invalid detail key `{key}`"),
            Self::InvalidExplanationIdentity { identity } => {
                write!(formatter, "invalid explanation identity `{identity}`")
            }
            Self::UnknownDiagnosticCode { code } => {
                write!(formatter, "unknown diagnostic code {code}")
            }
            Self::RetiredDescriptorForDraft { code } => {
                write!(
                    formatter,
                    "retired diagnostic code {code} cannot be emitted by a draft"
                )
            }
            Self::InvalidRange { start, end } => {
                write!(
                    formatter,
                    "diagnostic range start {start} exceeds end {end}"
                )
            }
            Self::ZeroWidthIntentRequired { offset } => {
                write!(
                    formatter,
                    "zero-width diagnostic span at {offset} requires intent"
                )
            }
            Self::ZeroWidthIntentOnNonZeroRange { start, end, intent } => {
                write!(
                    formatter,
                    "non-zero diagnostic span {start}..{end} cannot carry {:?} intent",
                    intent
                )
            }
            Self::PrimarySpanMustUsePrimaryRole { actual } => {
                write!(formatter, "primary span used {actual} role")
            }
            Self::SecondarySpanMustNotUsePrimaryRole { index } => {
                write!(formatter, "secondary span {index} used primary role")
            }
            Self::RetiredDescriptorForCurrentRecord { code } => {
                write!(
                    formatter,
                    "retired diagnostic code {code} cannot be current"
                )
            }
            Self::DraftFreshnessSnapshotMismatch {
                draft_snapshot,
                freshness_snapshot,
            } => write!(
                formatter,
                "freshness source snapshot {freshness_snapshot:?} does not match draft \
                 snapshot {draft_snapshot:?}"
            ),
            Self::CurrentFreshnessSnapshotMismatch {
                handle_snapshot,
                source_snapshot,
            } => write!(
                formatter,
                "current freshness source snapshot {source_snapshot:?} does not match handle \
                 snapshot {handle_snapshot:?}"
            ),
            Self::StaleFreshnessSnapshotMismatch {
                handle_snapshot,
                current_snapshot,
            } => write!(
                formatter,
                "stale freshness current snapshot {current_snapshot:?} does not match handle \
                 snapshot {handle_snapshot:?}"
            ),
            Self::StaleFreshnessNotStale {
                source_snapshot,
                current_snapshot,
            } => write!(
                formatter,
                "stale freshness used identical source/current snapshot {source_snapshot:?} \
                 and {current_snapshot:?}"
            ),
            Self::HistoricalFreshnessSnapshotMismatch {
                handle_snapshot,
                source_snapshot,
            } => write!(
                formatter,
                "historical freshness source snapshot {source_snapshot:?} does not match handle \
                 snapshot {handle_snapshot:?}"
            ),
            Self::RelatedHandleSnapshotMismatch {
                index,
                handle_snapshot,
                related_snapshot,
            } => write!(
                formatter,
                "related handle {index} snapshot {related_snapshot:?} does not match record \
                 snapshot {handle_snapshot:?}"
            ),
        }
    }
}

impl Error for DiagnosticRecordError {}

/// Returns whether a stable detail key is valid.
pub fn is_valid_detail_key(key: &str) -> bool {
    validate_detail_key(key).is_ok()
}

fn validate_detail_key(key: &str) -> Result<(), ()> {
    if key.is_empty() {
        return Err(());
    }
    for segment in key.split('.') {
        validate_detail_key_segment(segment)?;
    }
    Ok(())
}

fn validate_detail_key_segment(segment: &str) -> Result<(), ()> {
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

fn validate_source_range(range: SourceRange) -> Result<(), DiagnosticRecordError> {
    if range.start > range.end {
        return Err(DiagnosticRecordError::InvalidRange {
            start: range.start,
            end: range.end,
        });
    }
    Ok(())
}

fn validate_primary_and_secondary_spans(
    primary_span: &DiagnosticSpan,
    secondary_spans: &[DiagnosticSpan],
) -> Result<(), DiagnosticRecordError> {
    if primary_span.role != DiagnosticSpanRole::Primary {
        return Err(DiagnosticRecordError::PrimarySpanMustUsePrimaryRole {
            actual: primary_span.role,
        });
    }
    for (index, span) in secondary_spans.iter().enumerate() {
        if span.role == DiagnosticSpanRole::Primary {
            return Err(DiagnosticRecordError::SecondarySpanMustNotUsePrimaryRole { index });
        }
    }
    Ok(())
}

fn validate_draft_freshness(
    draft_snapshot: BuildSnapshotId,
    freshness: &DiagnosticFreshness,
) -> Result<(), DiagnosticRecordError> {
    let freshness_snapshot = freshness.source_snapshot();
    if freshness_snapshot != draft_snapshot {
        return Err(DiagnosticRecordError::DraftFreshnessSnapshotMismatch {
            draft_snapshot,
            freshness_snapshot,
        });
    }
    Ok(())
}

fn validate_freshness(
    handle: DiagnosticHandle,
    freshness: &DiagnosticFreshness,
) -> Result<(), DiagnosticRecordError> {
    match freshness {
        DiagnosticFreshness::Current { source_snapshot } => {
            if *source_snapshot != handle.snapshot {
                return Err(DiagnosticRecordError::CurrentFreshnessSnapshotMismatch {
                    handle_snapshot: handle.snapshot,
                    source_snapshot: *source_snapshot,
                });
            }
        }
        DiagnosticFreshness::Stale {
            source_snapshot,
            current_snapshot,
            ..
        } => {
            if *current_snapshot != handle.snapshot {
                return Err(DiagnosticRecordError::StaleFreshnessSnapshotMismatch {
                    handle_snapshot: handle.snapshot,
                    current_snapshot: *current_snapshot,
                });
            }
            if source_snapshot == current_snapshot {
                return Err(DiagnosticRecordError::StaleFreshnessNotStale {
                    source_snapshot: *source_snapshot,
                    current_snapshot: *current_snapshot,
                });
            }
        }
        DiagnosticFreshness::Historical {
            source_snapshot, ..
        } => {
            if *source_snapshot != handle.snapshot {
                return Err(DiagnosticRecordError::HistoricalFreshnessSnapshotMismatch {
                    handle_snapshot: handle.snapshot,
                    source_snapshot: *source_snapshot,
                });
            }
        }
    }
    Ok(())
}

fn validate_related_handles(
    handle: DiagnosticHandle,
    related: &[DiagnosticHandle],
) -> Result<(), DiagnosticRecordError> {
    for (index, related_handle) in related.iter().enumerate() {
        if related_handle.snapshot != handle.snapshot {
            return Err(DiagnosticRecordError::RelatedHandleSnapshotMismatch {
                index,
                handle_snapshot: handle.snapshot,
                related_snapshot: related_handle.snapshot,
            });
        }
    }
    Ok(())
}

fn normalize_fixes(mut fixes: Vec<FixSuggestion>) -> Vec<FixSuggestion> {
    fixes.sort_by(|left, right| {
        left.canonical_key()
            .cmp(&right.canonical_key())
            .then_with(|| left.debug_snapshot().cmp(&right.debug_snapshot()))
    });
    fixes.dedup_by(|left, right| left.canonical_key() == right.canonical_key());
    fixes
}

fn variant_order(value: &DiagnosticDetailValue) -> u8 {
    match value {
        DiagnosticDetailValue::Boolean(_) => 0,
        DiagnosticDetailValue::Integer(_) => 1,
        DiagnosticDetailValue::String(_) => 2,
        DiagnosticDetailValue::Code(_) => 3,
        DiagnosticDetailValue::Source(_) => 4,
        DiagnosticDetailValue::List(_) => 5,
    }
}

fn compare_source_ranges(left: SourceRange, right: SourceRange) -> Ordering {
    source_id_debug(left)
        .cmp(&source_id_debug(right))
        .then(left.start.cmp(&right.start))
        .then(left.end.cmp(&right.end))
}

fn source_id_debug(range: SourceRange) -> String {
    format!("{:?}", range.source_id)
}

enum DebugSnapshot<'a> {
    Draft(&'a DiagnosticDraft),
    Record(&'a DiagnosticRecord),
}

fn render_debug_snapshot(snapshot: DebugSnapshot<'_>) -> String {
    let mut lines = Vec::new();
    match snapshot {
        DebugSnapshot::Draft(draft) => {
            lines.push("kind=draft".to_owned());
            lines.push("handle=none".to_owned());
            lines.push(format!("code={}", draft.code));
            lines.push("semantic_name=none".to_owned());
            lines.push("severity=none".to_owned());
            lines.push(format!("phase={}", draft.phase));
            lines.push(format!("category={}", draft.category));
            lines.push(format!("stable_detail_key={:?}", draft.stable_detail_key));
            lines.push(format!("message={:?}", draft.message));
            lines.push(format!(
                "source_snapshot={}",
                render_snapshot(draft.source_snapshot)
            ));
            lines.push("freshness=draft".to_owned());
            lines.push(format!("primary={}", render_span(&draft.primary_span)));
            lines.push(format!(
                "secondary={}",
                render_spans(&draft.secondary_spans)
            ));
            lines.push(format!("notes={}", render_notes(&draft.notes)));
            lines.push(format!("details={}", render_details(&draft.details)));
            lines.push(format!(
                "fixes={}",
                render_fixes(&draft.fixes, "unpublished")
            ));
            lines.push(format!(
                "explanation={}",
                render_explanation(draft.explanation.as_ref())
            ));
            lines.push("related=[]".to_owned());
        }
        DebugSnapshot::Record(record) => {
            lines.push("kind=record".to_owned());
            lines.push(format!("handle={}", render_handle(record.handle)));
            lines.push(format!("code={}", record.code));
            lines.push(format!("semantic_name={:?}", record.semantic_name));
            lines.push(format!("severity={}", record.severity));
            lines.push(format!("phase={}", record.phase));
            lines.push(format!("category={}", record.category));
            lines.push(format!("stable_detail_key={:?}", record.stable_detail_key));
            lines.push(format!("message={:?}", record.message));
            lines.push(format!(
                "source_snapshot={}",
                render_snapshot(record.freshness.source_snapshot())
            ));
            lines.push(format!("freshness={}", render_freshness(&record.freshness)));
            lines.push(format!("primary={}", render_span(&record.primary_span)));
            lines.push(format!(
                "secondary={}",
                render_spans(&record.secondary_spans)
            ));
            lines.push(format!("notes={}", render_notes(&record.notes)));
            lines.push(format!("details={}", render_details(&record.details)));
            lines.push(format!(
                "fixes={}",
                render_fixes(&record.fixes, &render_handle(record.handle))
            ));
            lines.push(format!(
                "explanation={}",
                render_explanation(record.explanation.as_ref())
            ));
            lines.push(format!("related={}", render_handles(&record.related)));
        }
    }

    let mut rendered = lines.join("\n");
    rendered.push('\n');
    rendered
}

fn render_snapshot(snapshot: BuildSnapshotId) -> String {
    snapshot
        .to_published_schema_string()
        .unwrap_or_else(|_| format!("{snapshot:?}"))
}

fn render_handle(handle: DiagnosticHandle) -> String {
    format!("{}#{}", render_snapshot(handle.snapshot), handle.id.get())
}

fn render_handles(handles: &[DiagnosticHandle]) -> String {
    let rendered = handles
        .iter()
        .map(|handle| render_handle(*handle))
        .collect::<Vec<_>>();
    format!("[{}]", rendered.join(", "))
}

fn render_freshness(freshness: &DiagnosticFreshness) -> String {
    match freshness {
        DiagnosticFreshness::Current { source_snapshot } => {
            format!(
                "current(source_snapshot={})",
                render_snapshot(*source_snapshot)
            )
        }
        DiagnosticFreshness::Stale {
            source_snapshot,
            current_snapshot,
            reason,
        } => format!(
            "stale(source_snapshot={},current_snapshot={},reason={})",
            render_snapshot(*source_snapshot),
            render_snapshot(*current_snapshot),
            reason
        ),
        DiagnosticFreshness::Historical {
            source_snapshot,
            artifact_hash,
        } => format!(
            "historical(source_snapshot={},artifact_hash={})",
            render_snapshot(*source_snapshot),
            render_optional_string(artifact_hash.as_deref())
        ),
    }
}

fn render_spans(spans: &[DiagnosticSpan]) -> String {
    let rendered = spans.iter().map(render_span).collect::<Vec<_>>();
    format!("[{}]", rendered.join(", "))
}

fn render_span(span: &DiagnosticSpan) -> String {
    let range = span.range;
    format!(
        "{}:{}..{}:{}:{}:{}:{}",
        source_id_debug(range),
        range.start,
        range.end,
        span.role,
        render_span_freshness(span.freshness),
        render_zero_width(span.zero_width),
        render_optional_string(span.label.as_deref())
    )
}

fn render_span_freshness(freshness: SpanFreshness) -> String {
    match freshness {
        SpanFreshness::Current => "current".to_owned(),
        SpanFreshness::Stale { reason } => format!("stale({reason})"),
        SpanFreshness::Historical => "historical".to_owned(),
    }
}

fn render_zero_width(intent: Option<ZeroWidthSpanIntent>) -> &'static str {
    match intent {
        None => "none",
        Some(ZeroWidthSpanIntent::Eof) => "eof",
        Some(ZeroWidthSpanIntent::InsertionPoint) => "insertion_point",
    }
}

fn render_notes(notes: &[DiagnosticNote]) -> String {
    let rendered = notes
        .iter()
        .map(|note| {
            format!(
                "{{kind={},message={:?},span={}}}",
                note.kind.as_str(),
                note.message,
                note.span
                    .as_ref()
                    .map_or_else(|| "none".to_owned(), render_span)
            )
        })
        .collect::<Vec<_>>();
    format!("[{}]", rendered.join(", "))
}

fn render_details(details: &DiagnosticDetails) -> String {
    let rendered = details
        .entries
        .iter()
        .map(|(key, value)| format!("{key}={}", render_detail_value(value)))
        .collect::<Vec<_>>();
    format!("{{{}}}", rendered.join(", "))
}

fn render_detail_value(value: &DiagnosticDetailValue) -> String {
    match value {
        DiagnosticDetailValue::String(value) => format!("string:{value:?}"),
        DiagnosticDetailValue::Integer(value) => format!("int:{value}"),
        DiagnosticDetailValue::Boolean(value) => format!("bool:{value}"),
        DiagnosticDetailValue::Code(value) => format!("code:{value}"),
        DiagnosticDetailValue::Source(range) => format!(
            "source:{}:{}..{}",
            source_id_debug(*range),
            range.start,
            range.end
        ),
        DiagnosticDetailValue::List(values) => {
            let rendered = values.iter().map(render_detail_value).collect::<Vec<_>>();
            format!("[{}]", rendered.join(", "))
        }
    }
}

fn render_fixes(fixes: &[FixSuggestion], diagnostic: &str) -> String {
    let rendered = fixes
        .iter()
        .map(|fix| escape_embedded_snapshot(&fix.debug_snapshot_with_diagnostic(diagnostic)))
        .collect::<Vec<_>>();
    format!("[{}]", rendered.join(", "))
}

fn render_explanation(explanation: Option<&ExplanationRef>) -> String {
    explanation.map_or_else(
        || "none".to_owned(),
        |explanation| format!("{:?}", explanation.identity),
    )
}

fn render_optional_string(value: Option<&str>) -> String {
    value.map_or_else(|| "none".to_owned(), |value| format!("{value:?}"))
}

fn escape_embedded_snapshot(snapshot: &str) -> String {
    snapshot
        .strip_suffix('\n')
        .unwrap_or(snapshot)
        .replace('\n', "\\n")
}
