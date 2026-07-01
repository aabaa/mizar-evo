//! Producer-side diagnostic draft collection.

use std::{error::Error, fmt};

use mizar_session::BuildSnapshotId;

use crate::failure_record::{DiagnosticDraft, PipelinePhase};

/// Producer scope bound to a diagnostic sink.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticProducerScope {
    phase: PipelinePhase,
    source_snapshot: BuildSnapshotId,
    producer_name: &'static str,
}

impl DiagnosticProducerScope {
    /// Creates a producer scope for one phase invocation.
    pub const fn new(
        phase: PipelinePhase,
        source_snapshot: BuildSnapshotId,
        producer_name: &'static str,
    ) -> Self {
        Self {
            phase,
            source_snapshot,
            producer_name,
        }
    }

    /// Returns the producer phase.
    pub const fn phase(self) -> PipelinePhase {
        self.phase
    }

    /// Returns the source snapshot observed by the producer.
    pub const fn source_snapshot(self) -> BuildSnapshotId {
        self.source_snapshot
    }

    /// Returns stable debug metadata for this producer.
    pub const fn producer_name(self) -> &'static str {
        self.producer_name
    }
}

/// Mutable producer-side sink for diagnostic drafts.
#[derive(Debug)]
pub struct DiagnosticSink {
    scope: DiagnosticProducerScope,
    drafts: Vec<DiagnosticDraft>,
    sealed: bool,
}

impl DiagnosticSink {
    /// Creates an empty sink for one producer scope.
    pub const fn new(scope: DiagnosticProducerScope) -> Self {
        Self {
            scope,
            drafts: Vec::new(),
            sealed: false,
        }
    }

    /// Returns the producer scope.
    pub const fn scope(&self) -> DiagnosticProducerScope {
        self.scope
    }

    /// Emits a draft into this sink.
    ///
    /// Failed emissions are non-mutating unless the sink was already sealed.
    pub fn emit(&mut self, draft: DiagnosticDraft) -> Result<(), DiagnosticSinkError> {
        if self.sealed {
            return Err(DiagnosticSinkError::SinkSealed);
        }
        if draft.phase() != self.scope.phase {
            return Err(DiagnosticSinkError::PhaseMismatch {
                expected: self.scope.phase,
                actual: draft.phase(),
            });
        }
        if draft.source_snapshot() != self.scope.source_snapshot {
            return Err(DiagnosticSinkError::SnapshotMismatch {
                expected: self.scope.source_snapshot,
                actual: draft.source_snapshot(),
            });
        }

        self.drafts.push(draft);
        Ok(())
    }

    /// Returns the currently collected drafts without sealing the sink.
    pub fn drafts(&self) -> &[DiagnosticDraft] {
        &self.drafts
    }

    /// Returns whether this sink has been sealed.
    pub const fn is_sealed(&self) -> bool {
        self.sealed
    }

    /// Seals this sink and returns an immutable diagnostic batch.
    pub fn seal(&mut self) -> DiagnosticBatch {
        self.sealed = true;
        DiagnosticBatch {
            scope: self.scope,
            drafts: self.drafts.clone(),
        }
    }

    /// Consumes this sink and returns an immutable diagnostic batch.
    pub fn into_batch(self) -> DiagnosticBatch {
        DiagnosticBatch {
            scope: self.scope,
            drafts: self.drafts,
        }
    }
}

/// Immutable batch of drafts from one producer scope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticBatch {
    scope: DiagnosticProducerScope,
    drafts: Vec<DiagnosticDraft>,
}

impl DiagnosticBatch {
    /// Returns the producer scope.
    pub const fn scope(&self) -> DiagnosticProducerScope {
        self.scope
    }

    /// Returns drafts in local emission order.
    pub fn drafts(&self) -> &[DiagnosticDraft] {
        &self.drafts
    }

    /// Returns whether this batch is empty.
    pub fn is_empty(&self) -> bool {
        self.drafts.is_empty()
    }

    /// Returns the number of drafts in this batch.
    pub fn len(&self) -> usize {
        self.drafts.len()
    }

    /// Returns a deterministic debug/test snapshot.
    pub fn debug_snapshot(&self) -> String {
        let mut lines = vec![
            "kind=batch".to_owned(),
            format!("phase={}", self.scope.phase),
            format!(
                "source_snapshot={}",
                render_snapshot(self.scope.source_snapshot)
            ),
            format!("producer_name={:?}", self.scope.producer_name),
            format!("draft_count={}", self.drafts.len()),
        ];

        for (index, draft) in self.drafts.iter().enumerate() {
            lines.push(format!(
                "draft[{index}]={}",
                escape_embedded_snapshot(&draft.debug_snapshot())
            ));
        }

        let mut rendered = lines.join("\n");
        rendered.push('\n');
        rendered
    }
}

/// Error returned by producer-side sink emission.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiagnosticSinkError {
    /// Emission was attempted after sealing.
    SinkSealed,
    /// Draft phase differed from the sink scope phase.
    PhaseMismatch {
        /// Expected phase from the sink scope.
        expected: PipelinePhase,
        /// Actual draft phase.
        actual: PipelinePhase,
    },
    /// Draft source snapshot differed from the sink scope snapshot.
    SnapshotMismatch {
        /// Expected source snapshot from the sink scope.
        expected: BuildSnapshotId,
        /// Actual draft source snapshot.
        actual: BuildSnapshotId,
    },
}

impl fmt::Display for DiagnosticSinkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SinkSealed => formatter.write_str("diagnostic sink is sealed"),
            Self::PhaseMismatch { expected, actual } => write!(
                formatter,
                "diagnostic draft phase {actual} does not match sink phase {expected}"
            ),
            Self::SnapshotMismatch { expected, actual } => write!(
                formatter,
                "diagnostic draft source snapshot {actual:?} does not match sink source \
                 snapshot {expected:?}"
            ),
        }
    }
}

impl Error for DiagnosticSinkError {}

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
