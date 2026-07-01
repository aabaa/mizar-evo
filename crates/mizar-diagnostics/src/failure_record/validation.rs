//! Validation helpers for structured diagnostic records.

use mizar_session::{BuildSnapshotId, SourceRange};

use crate::{
    explain::{ExplanationHandle, ExplanationSubject},
    registry::DiagnosticCode,
};

use super::{
    DiagnosticFreshness, DiagnosticHandle, DiagnosticRecordError, DiagnosticSpan,
    DiagnosticSpanRole,
};

pub(super) fn validate_detail_key(key: &str) -> Result<(), ()> {
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

pub(super) fn validate_source_range(range: SourceRange) -> Result<(), DiagnosticRecordError> {
    if range.start > range.end {
        return Err(DiagnosticRecordError::InvalidRange {
            start: range.start,
            end: range.end,
        });
    }
    Ok(())
}

pub(super) fn validate_primary_and_secondary_spans(
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

pub(super) fn validate_explanation_attachment(
    explanation: Option<&ExplanationHandle>,
    source_snapshot: BuildSnapshotId,
    code: DiagnosticCode,
    stable_detail_key: &str,
) -> Result<(), DiagnosticRecordError> {
    let Some(explanation) = explanation else {
        return Ok(());
    };
    if let Some(required_snapshot) = explanation.required_snapshot()
        && required_snapshot != source_snapshot
    {
        return Err(DiagnosticRecordError::ExplanationSnapshotMismatch {
            source_snapshot,
            required_snapshot,
        });
    }
    if let ExplanationSubject::Diagnostic {
        code: actual_code,
        stable_detail_key: actual_stable_detail_key,
    } = explanation.subject()
        && (*actual_code != code || actual_stable_detail_key != stable_detail_key)
    {
        return Err(
            DiagnosticRecordError::ExplanationDiagnosticSubjectMismatch {
                expected_code: code,
                expected_stable_detail_key: stable_detail_key.to_owned(),
                actual_code: *actual_code,
                actual_stable_detail_key: actual_stable_detail_key.clone(),
            },
        );
    }
    Ok(())
}

pub(super) fn validate_draft_freshness(
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

pub(super) fn validate_freshness(
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

pub(super) fn validate_related_handles(
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
