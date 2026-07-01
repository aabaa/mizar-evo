//! Deterministic build-snapshot diagnostic aggregation.

use std::{collections::BTreeMap, error::Error, fmt};

use mizar_session::{BuildSnapshotId, SourceId};

use crate::{
    failure_record::{
        DiagnosticDetailValue, DiagnosticDraft, DiagnosticFreshness, DiagnosticHandle,
        DiagnosticId, DiagnosticRecord, DiagnosticRecordError, DiagnosticSpan, FailureCategory,
        PipelinePhase, SpanFreshness,
    },
    registry::{DiagnosticCode, DiagnosticRegistry, DiagnosticSeverity},
    sink::DiagnosticBatch,
};

/// Input consumed by build-snapshot diagnostic aggregation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticAggregationInput {
    publication_snapshot: BuildSnapshotId,
    batches: Vec<DiagnosticBatch>,
}

impl DiagnosticAggregationInput {
    /// Creates aggregation input for one publication snapshot.
    pub fn new(publication_snapshot: BuildSnapshotId, batches: Vec<DiagnosticBatch>) -> Self {
        Self {
            publication_snapshot,
            batches,
        }
    }

    /// Returns the publication snapshot.
    pub const fn publication_snapshot(&self) -> BuildSnapshotId {
        self.publication_snapshot
    }

    /// Returns producer batches in caller-provided order.
    pub fn batches(&self) -> &[DiagnosticBatch] {
        &self.batches
    }
}

/// Deterministic source key used inside a diagnostic index.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DiagnosticSourceKey(String);

impl DiagnosticSourceKey {
    /// Creates a deterministic source key for a compiler-native source id.
    pub fn from_source_id(source_id: SourceId) -> Self {
        Self(
            source_id
                .to_published_schema_string()
                .unwrap_or_else(|_| format!("{source_id:?}")),
        )
    }

    /// Returns the rendered source key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A producer draft withheld because it came from an obsolete source snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObsoleteDiagnosticDraft {
    producer_name: &'static str,
    local_ordinal: usize,
    draft: DiagnosticDraft,
}

impl ObsoleteDiagnosticDraft {
    /// Returns the source snapshot observed by the producer.
    pub const fn source_snapshot(&self) -> BuildSnapshotId {
        self.draft.source_snapshot()
    }

    /// Returns the producer debug name.
    pub const fn producer_name(&self) -> &'static str {
        self.producer_name
    }

    /// Returns the draft ordinal within its producer batch.
    pub const fn local_ordinal(&self) -> usize {
        self.local_ordinal
    }

    /// Returns the withheld draft.
    pub const fn draft(&self) -> &DiagnosticDraft {
        &self.draft
    }
}

/// Immutable current diagnostic index for one build snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildDiagnosticIndex {
    snapshot: BuildSnapshotId,
    records: Vec<DiagnosticRecord>,
    by_source: BTreeMap<DiagnosticSourceKey, Vec<DiagnosticHandle>>,
    by_id: BTreeMap<DiagnosticId, usize>,
    obsolete_drafts: Vec<ObsoleteDiagnosticDraft>,
}

impl BuildDiagnosticIndex {
    /// Aggregates producer batches for a publication snapshot.
    pub fn aggregate(
        input: DiagnosticAggregationInput,
    ) -> Result<Self, DiagnosticAggregationError> {
        Self::from_batches(input.publication_snapshot, input.batches)
    }

    /// Aggregates producer batches for a publication snapshot.
    pub fn from_batches(
        publication_snapshot: BuildSnapshotId,
        batches: Vec<DiagnosticBatch>,
    ) -> Result<Self, DiagnosticAggregationError> {
        let registry = DiagnosticRegistry::builtin();
        let mut representatives = BTreeMap::<DedupKey, DraftCandidate>::new();
        let mut obsolete_drafts = Vec::new();

        for batch in batches {
            let scope = batch.scope();
            for (local_ordinal, draft) in batch.drafts().iter().cloned().enumerate() {
                if draft.phase() != scope.phase() {
                    return Err(DiagnosticAggregationError::BatchPhaseMismatch {
                        producer_name: scope.producer_name(),
                        local_ordinal,
                        expected: scope.phase(),
                        actual: draft.phase(),
                    });
                }
                if draft.source_snapshot() != scope.source_snapshot() {
                    return Err(DiagnosticAggregationError::BatchSnapshotMismatch {
                        producer_name: scope.producer_name(),
                        local_ordinal,
                        expected: scope.source_snapshot(),
                        actual: draft.source_snapshot(),
                    });
                }

                if draft.source_snapshot() != publication_snapshot {
                    obsolete_drafts.push(ObsoleteDiagnosticDraft {
                        producer_name: scope.producer_name(),
                        local_ordinal,
                        draft,
                    });
                    continue;
                }

                let candidate = DraftCandidate::new(draft, registry);
                representatives
                    .entry(candidate.dedup_key.clone())
                    .and_modify(|existing| {
                        if candidate.presentation_key < existing.presentation_key {
                            *existing = candidate.clone();
                        }
                    })
                    .or_insert(candidate);
            }
        }

        let mut candidates = representatives.into_values().collect::<Vec<_>>();
        candidates.sort_by(|left, right| left.sort_key.cmp(&right.sort_key));
        obsolete_drafts.sort_by_key(obsolete_sort_key);

        let mut records = Vec::with_capacity(candidates.len());
        let mut by_source = BTreeMap::<DiagnosticSourceKey, Vec<DiagnosticHandle>>::new();
        let mut by_id = BTreeMap::new();

        for (ordinal, candidate) in candidates.into_iter().enumerate() {
            let id = DiagnosticId::new(
                u64::try_from(ordinal).expect("diagnostic ordinal fits into u64"),
            );
            let handle = DiagnosticHandle::new(publication_snapshot, id);
            let source_key = DiagnosticSourceKey::from_source_id(
                candidate.draft.primary_span().range().source_id,
            );
            let code = candidate.draft.code();
            let record = DiagnosticRecord::from_draft_with_registry(
                candidate.draft,
                registry,
                handle,
                DiagnosticFreshness::Current {
                    source_snapshot: publication_snapshot,
                },
                Vec::new(),
            )
            .map_err(|error| DiagnosticAggregationError::RecordProjection { code, error })?;
            by_id.insert(id, records.len());
            by_source.entry(source_key).or_default().push(handle);
            records.push(record);
        }

        Ok(Self {
            snapshot: publication_snapshot,
            records,
            by_source,
            by_id,
            obsolete_drafts,
        })
    }

    /// Returns the publication snapshot.
    pub const fn snapshot(&self) -> BuildSnapshotId {
        self.snapshot
    }

    /// Returns records in canonical publication order.
    pub fn records(&self) -> &[DiagnosticRecord] {
        &self.records
    }

    /// Returns whether the current index contains no current records.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns the number of current records.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns source-indexed handles.
    pub const fn by_source(&self) -> &BTreeMap<DiagnosticSourceKey, Vec<DiagnosticHandle>> {
        &self.by_source
    }

    /// Looks up handles for one source id.
    pub fn handles_for_source(&self, source_id: SourceId) -> Option<&[DiagnosticHandle]> {
        self.by_source
            .get(&DiagnosticSourceKey::from_source_id(source_id))
            .map(Vec::as_slice)
    }

    /// Looks up a record by snapshot-local diagnostic id.
    pub fn record_by_id(&self, id: DiagnosticId) -> Option<&DiagnosticRecord> {
        self.by_id.get(&id).map(|index| &self.records[*index])
    }

    /// Returns obsolete drafts withheld from current publication.
    pub fn obsolete_drafts(&self) -> &[ObsoleteDiagnosticDraft] {
        &self.obsolete_drafts
    }

    /// Returns a deterministic debug/test snapshot.
    pub fn debug_snapshot(&self) -> String {
        let mut lines = vec![
            "kind=index".to_owned(),
            format!("snapshot={}", render_snapshot(self.snapshot)),
            format!("record_count={}", self.records.len()),
            format!("obsolete_count={}", self.obsolete_drafts.len()),
        ];

        for (index, record) in self.records.iter().enumerate() {
            lines.push(format!(
                "record[{index}]={}",
                escape_embedded_snapshot(&record.debug_snapshot())
            ));
        }
        for (index, obsolete) in self.obsolete_drafts.iter().enumerate() {
            lines.push(format!(
                "obsolete[{index}]=source_snapshot={};producer_name={:?};local_ordinal={};draft={}",
                render_snapshot(obsolete.source_snapshot()),
                obsolete.producer_name(),
                obsolete.local_ordinal(),
                escape_embedded_snapshot(&obsolete.draft().debug_snapshot())
            ));
        }

        let mut rendered = lines.join("\n");
        rendered.push('\n');
        rendered
    }
}

/// Error returned by build-snapshot diagnostic aggregation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticAggregationError {
    /// A batch carried a draft whose phase did not match the producer scope.
    BatchPhaseMismatch {
        /// Producer debug name.
        producer_name: &'static str,
        /// Draft ordinal within the batch.
        local_ordinal: usize,
        /// Expected phase from the batch scope.
        expected: PipelinePhase,
        /// Actual draft phase.
        actual: PipelinePhase,
    },
    /// A batch carried a draft whose source snapshot did not match the scope.
    BatchSnapshotMismatch {
        /// Producer debug name.
        producer_name: &'static str,
        /// Draft ordinal within the batch.
        local_ordinal: usize,
        /// Expected snapshot from the batch scope.
        expected: BuildSnapshotId,
        /// Actual draft source snapshot.
        actual: BuildSnapshotId,
    },
    /// A current draft could not be projected to a record.
    RecordProjection {
        /// Diagnostic code being projected.
        code: DiagnosticCode,
        /// Underlying record-construction error.
        error: DiagnosticRecordError,
    },
}

impl fmt::Display for DiagnosticAggregationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BatchPhaseMismatch {
                producer_name,
                local_ordinal,
                expected,
                actual,
            } => write!(
                formatter,
                "diagnostic batch from {producer_name:?} draft {local_ordinal} used phase \
                 {actual}, expected {expected}"
            ),
            Self::BatchSnapshotMismatch {
                producer_name,
                local_ordinal,
                expected,
                actual,
            } => write!(
                formatter,
                "diagnostic batch from {producer_name:?} draft {local_ordinal} used source \
                 snapshot {actual:?}, expected {expected:?}"
            ),
            Self::RecordProjection { code, error } => {
                write!(
                    formatter,
                    "diagnostic {code} could not be projected: {error}"
                )
            }
        }
    }
}

impl Error for DiagnosticAggregationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RecordProjection { error, .. } => Some(error),
            Self::BatchPhaseMismatch { .. } | Self::BatchSnapshotMismatch { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DraftCandidate {
    dedup_key: DedupKey,
    sort_key: AggregationSortKey,
    presentation_key: String,
    draft: DiagnosticDraft,
}

impl DraftCandidate {
    fn new(draft: DiagnosticDraft, registry: DiagnosticRegistry<'_>) -> Self {
        let dedup_key = DedupKey::from_draft(&draft);
        let presentation_key = draft.debug_snapshot();
        let severity = registry.lookup(draft.code()).map_or_else(
            || draft.code().severity(),
            |descriptor| descriptor.default_severity,
        );
        let sort_key = AggregationSortKey::from_draft(
            &draft,
            severity,
            dedup_key.clone(),
            presentation_key.clone(),
        );

        Self {
            dedup_key,
            sort_key,
            presentation_key,
            draft,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AggregationSortKey {
    primary_source: DiagnosticSourceKey,
    primary_start: usize,
    primary_end: usize,
    phase: PipelinePhase,
    severity: DiagnosticSeverity,
    code: DiagnosticCode,
    category: FailureCategory,
    stable_detail_key: String,
    dedup_key: DedupKey,
    presentation_key: String,
}

impl AggregationSortKey {
    fn from_draft(
        draft: &DiagnosticDraft,
        severity: DiagnosticSeverity,
        dedup_key: DedupKey,
        presentation_key: String,
    ) -> Self {
        let primary_range = draft.primary_span().range();
        Self {
            primary_source: DiagnosticSourceKey::from_source_id(primary_range.source_id),
            primary_start: primary_range.start,
            primary_end: primary_range.end,
            phase: draft.phase(),
            severity,
            code: draft.code(),
            category: draft.category(),
            stable_detail_key: draft.stable_detail_key().to_owned(),
            dedup_key,
            presentation_key,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DedupKey {
    code: DiagnosticCode,
    phase: PipelinePhase,
    category: FailureCategory,
    primary_span: SpanIdentityKey,
    stable_detail_key: String,
    details: Vec<(String, DiagnosticDetailValue)>,
    fixes: Vec<String>,
    explanation: Option<String>,
}

impl DedupKey {
    fn from_draft(draft: &DiagnosticDraft) -> Self {
        Self {
            code: draft.code(),
            phase: draft.phase(),
            category: draft.category(),
            primary_span: SpanIdentityKey::from_span(draft.primary_span()),
            stable_detail_key: draft.stable_detail_key().to_owned(),
            details: draft
                .details()
                .entries()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            fixes: draft
                .fixes()
                .iter()
                .map(|fix| fix.identity().to_owned())
                .collect(),
            explanation: draft
                .explanation()
                .map(|explanation| explanation.identity().to_owned()),
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SpanIdentityKey {
    source: DiagnosticSourceKey,
    start: usize,
    end: usize,
    role: &'static str,
    freshness: SpanFreshnessKey,
    zero_width: &'static str,
}

impl SpanIdentityKey {
    fn from_span(span: &DiagnosticSpan) -> Self {
        let range = span.range();
        Self {
            source: DiagnosticSourceKey::from_source_id(range.source_id),
            start: range.start,
            end: range.end,
            role: span.role().as_str(),
            freshness: SpanFreshnessKey::from_freshness(span.freshness()),
            zero_width: match span.zero_width() {
                Some(intent) => intent.as_str(),
                None => "none",
            },
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SpanFreshnessKey {
    Current,
    Stale(&'static str),
    Historical,
}

impl SpanFreshnessKey {
    fn from_freshness(freshness: SpanFreshness) -> Self {
        match freshness {
            SpanFreshness::Current => Self::Current,
            SpanFreshness::Stale { reason } => Self::Stale(reason.as_str()),
            SpanFreshness::Historical => Self::Historical,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ObsoleteSortKey {
    source_snapshot: String,
    producer_name: &'static str,
    local_ordinal: usize,
    draft_snapshot: String,
}

fn obsolete_sort_key(obsolete: &ObsoleteDiagnosticDraft) -> ObsoleteSortKey {
    ObsoleteSortKey {
        source_snapshot: render_snapshot(obsolete.source_snapshot()),
        producer_name: obsolete.producer_name(),
        local_ordinal: obsolete.local_ordinal(),
        draft_snapshot: obsolete.draft().debug_snapshot(),
    }
}

fn render_snapshot(snapshot: BuildSnapshotId) -> String {
    snapshot
        .to_published_schema_string()
        .unwrap_or_else(|_| format!("{snapshot:?}"))
}

fn escape_embedded_snapshot(snapshot: &str) -> String {
    snapshot
        .strip_suffix('\n')
        .unwrap_or(snapshot)
        .replace('\n', "\\n")
}
