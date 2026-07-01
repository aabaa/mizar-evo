use std::str::FromStr;

use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticNote, DiagnosticNoteKind, DiagnosticSpan, ExplanationRef, FailureCategory,
        PipelinePhase,
    },
    fix::{FixSuggestion, FixSuggestionId},
    registry::DiagnosticCode,
    sink::{DiagnosticProducerScope, DiagnosticSink, DiagnosticSinkError},
};
use mizar_session::{
    BuildSnapshotId, InMemorySessionIdAllocator, SessionIdAllocator, SourceId, SourceRange,
};

#[test]
fn sink_preserves_drafts_in_local_order_until_sealed() {
    let snapshot = snapshot_id(1);
    let source_id = source_id(snapshot);
    let first = draft(snapshot, source_id, PipelinePhase::Parser, 0, 1, "first");
    let second = draft(snapshot, source_id, PipelinePhase::Parser, 2, 3, "second");
    let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::Parser,
        snapshot,
        "parser.recovery",
    ));

    sink.emit(first.clone()).expect("first draft is accepted");
    sink.emit(second.clone()).expect("second draft is accepted");
    assert_eq!(sink.drafts(), &[first.clone(), second.clone()]);
    assert!(!sink.is_sealed());

    let batch = sink.seal();
    assert!(sink.is_sealed());
    assert_eq!(batch.scope().phase(), PipelinePhase::Parser);
    assert_eq!(batch.scope().source_snapshot(), snapshot);
    assert_eq!(batch.scope().producer_name(), "parser.recovery");
    assert_eq!(batch.drafts(), &[first, second]);
    assert_eq!(batch.len(), 2);
    assert!(!batch.is_empty());
}

#[test]
fn sink_preserves_non_empty_draft_payload_fields() {
    let snapshot = snapshot_id(8);
    let source_id = source_id(snapshot);
    let rich = rich_draft(snapshot, source_id);
    let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::Parser,
        snapshot,
        "parser.recovery",
    ));

    sink.emit(rich.clone()).expect("rich draft is accepted");
    let batch = sink.seal();
    assert_eq!(batch.drafts(), std::slice::from_ref(&rich));

    let preserved = &batch.drafts()[0];
    assert_eq!(preserved.secondary_spans(), rich.secondary_spans());
    assert_eq!(preserved.notes(), rich.notes());
    assert_eq!(preserved.details(), rich.details());
    assert_eq!(preserved.fixes(), rich.fixes());
    assert_eq!(preserved.explanation(), rich.explanation());
}

#[test]
fn failed_emits_are_non_mutating_and_sink_remains_open() {
    let snapshot = snapshot_id(2);
    let other_snapshot = snapshot_id(3);
    let primary_source_id = source_id(snapshot);
    let valid = draft(
        snapshot,
        primary_source_id,
        PipelinePhase::Parser,
        0,
        1,
        "valid",
    );
    let wrong_phase = draft(
        snapshot,
        primary_source_id,
        PipelinePhase::Resolver,
        1,
        2,
        "wrong-phase",
    );
    let wrong_snapshot_source = source_id(other_snapshot);
    let wrong_snapshot = draft(
        other_snapshot,
        wrong_snapshot_source,
        PipelinePhase::Parser,
        0,
        1,
        "wrong-snapshot",
    );
    let later_valid = draft(
        snapshot,
        primary_source_id,
        PipelinePhase::Parser,
        2,
        3,
        "later-valid",
    );
    let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::Parser,
        snapshot,
        "parser.recovery",
    ));

    sink.emit(valid.clone()).expect("valid draft is accepted");
    assert!(matches!(
        sink.emit(wrong_phase),
        Err(DiagnosticSinkError::PhaseMismatch {
            expected: PipelinePhase::Parser,
            actual: PipelinePhase::Resolver,
        })
    ));
    assert_eq!(sink.drafts(), std::slice::from_ref(&valid));
    assert!(!sink.is_sealed());

    assert!(matches!(
        sink.emit(wrong_snapshot),
        Err(DiagnosticSinkError::SnapshotMismatch {
            expected,
            actual,
        }) if expected == snapshot && actual == other_snapshot
    ));
    assert_eq!(sink.drafts(), std::slice::from_ref(&valid));
    assert!(!sink.is_sealed());

    sink.emit(later_valid.clone())
        .expect("sink remains usable after recoverable errors");
    assert_eq!(sink.drafts(), &[valid, later_valid]);
}

#[test]
fn sealed_sink_rejects_later_emits_without_changing_batch_source() {
    let snapshot = snapshot_id(4);
    let source_id = source_id(snapshot);
    let first = draft(snapshot, source_id, PipelinePhase::Parser, 0, 1, "first");
    let second = draft(snapshot, source_id, PipelinePhase::Parser, 2, 3, "second");
    let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::Parser,
        snapshot,
        "parser.recovery",
    ));

    sink.emit(first.clone()).expect("valid draft is accepted");
    let batch = sink.seal();
    assert_eq!(batch.drafts(), std::slice::from_ref(&first));
    assert!(matches!(
        sink.emit(second),
        Err(DiagnosticSinkError::SinkSealed)
    ));
    assert_eq!(sink.drafts(), &[first]);
}

#[test]
fn consumed_sink_yields_batch_without_mutating_drafts() {
    let snapshot = snapshot_id(5);
    let source_id = source_id(snapshot);
    let first = draft(snapshot, source_id, PipelinePhase::Parser, 0, 1, "first");
    let second = draft(snapshot, source_id, PipelinePhase::Parser, 2, 3, "second");
    let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::Parser,
        snapshot,
        "parser.recovery",
    ));

    sink.emit(first.clone()).expect("first draft is accepted");
    sink.emit(second.clone()).expect("second draft is accepted");
    let batch = sink.into_batch();

    assert_eq!(batch.drafts(), &[first, second]);
}

#[test]
fn batch_debug_snapshot_is_byte_stable() {
    let snapshot = snapshot_id(6);
    let source_id = source_id(snapshot);
    let first = draft(snapshot, source_id, PipelinePhase::Parser, 0, 1, "first");
    let second = draft(snapshot, source_id, PipelinePhase::Parser, 2, 3, "second");
    let mut sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::Parser,
        snapshot,
        "parser.recovery",
    ));
    sink.emit(first).expect("first draft is accepted");
    sink.emit(second).expect("second draft is accepted");

    assert_eq!(
        sink.seal().debug_snapshot(),
        concat!(
            "kind=batch\n",
            "phase=parser\n",
            "source_snapshot=mizar-session-build-snapshot-v1:",
            "0606060606060606060606060606060606060606060606060606060606060606\n",
            "producer_name=\"parser.recovery\"\n",
            "draft_count=2\n",
            "draft[0]=kind=draft\\nhandle=none\\ncode=E0001\\nsemantic_name=none\\n",
            "severity=none\\nphase=parser\\ncategory=parse_error\\n",
            "stable_detail_key=\"syntax.unexpected_token\"\\nmessage=\"first\"\\n",
            "source_snapshot=mizar-session-build-snapshot-v1:",
            "0606060606060606060606060606060606060606060606060606060606060606\\n",
            "freshness=draft\\nprimary=SourceId(OpaqueId(1)):0..1:primary:current:none:none\\n",
            "secondary=[]\\nnotes=[]\\ndetails={sink.ordinal=int:0}\\nfixes=[]\\n",
            "explanation=none\\nrelated=[]\n",
            "draft[1]=kind=draft\\nhandle=none\\ncode=E0001\\nsemantic_name=none\\n",
            "severity=none\\nphase=parser\\ncategory=parse_error\\n",
            "stable_detail_key=\"syntax.unexpected_token\"\\nmessage=\"second\"\\n",
            "source_snapshot=mizar-session-build-snapshot-v1:",
            "0606060606060606060606060606060606060606060606060606060606060606\\n",
            "freshness=draft\\nprimary=SourceId(OpaqueId(1)):2..3:primary:current:none:none\\n",
            "secondary=[]\\nnotes=[]\\ndetails={sink.ordinal=int:2}\\nfixes=[]\\n",
            "explanation=none\\nrelated=[]\n",
        )
    );
}

#[test]
fn empty_batch_debug_snapshot_is_byte_stable() {
    let snapshot = snapshot_id(7);
    let sink = DiagnosticSink::new(DiagnosticProducerScope::new(
        PipelinePhase::Parser,
        snapshot,
        "parser.\"escape\"\\line\nnext",
    ));

    assert_eq!(
        sink.into_batch().debug_snapshot(),
        concat!(
            "kind=batch\n",
            "phase=parser\n",
            "source_snapshot=mizar-session-build-snapshot-v1:",
            "0707070707070707070707070707070707070707070707070707070707070707\n",
            "producer_name=\"parser.\\\"escape\\\"\\\\line\\nnext\"\n",
            "draft_count=0\n",
        )
    );
}

fn draft(
    snapshot: BuildSnapshotId,
    source_id: SourceId,
    phase: PipelinePhase,
    start: usize,
    end: usize,
    message: &str,
) -> DiagnosticDraft {
    DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_str("E0001").expect("allocated code"),
        phase,
        category: FailureCategory::ParseError,
        stable_detail_key: "syntax.unexpected_token".to_owned(),
        message: message.to_owned(),
        primary_span: DiagnosticSpan::primary(
            SourceRange {
                source_id,
                start,
                end,
            },
            None,
        )
        .expect("valid primary span"),
        secondary_spans: vec![],
        notes: vec![],
        details: DiagnosticDetails::from_entries([(
            "sink.ordinal",
            DiagnosticDetailValue::Integer(start.try_into().expect("test start fits i64")),
        )])
        .expect("valid details"),
        fixes: vec![],
        explanation: None,
    })
    .expect("valid draft")
}

fn rich_draft(snapshot: BuildSnapshotId, source_id: SourceId) -> DiagnosticDraft {
    let secondary_span = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 2,
            end: 4,
        },
        Some("matching token".to_owned()),
    )
    .expect("valid secondary span");

    DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_str("E0001").expect("allocated code"),
        phase: PipelinePhase::Parser,
        category: FailureCategory::ParseError,
        stable_detail_key: "syntax.unexpected_token".to_owned(),
        message: "rich".to_owned(),
        primary_span: DiagnosticSpan::primary(
            SourceRange {
                source_id,
                start: 0,
                end: 1,
            },
            Some("unexpected token".to_owned()),
        )
        .expect("valid primary span"),
        secondary_spans: vec![secondary_span.clone()],
        notes: vec![DiagnosticNote::new(
            DiagnosticNoteKind::Help,
            "insert the expected token",
            Some(secondary_span),
        )],
        details: DiagnosticDetails::from_entries([
            ("sink.ordinal", DiagnosticDetailValue::Integer(0)),
            ("sink.payload_present", DiagnosticDetailValue::Boolean(true)),
        ])
        .expect("valid details"),
        fixes: vec![
            FixSuggestion::informational(
                FixSuggestionId::new("sink.insert_expected_token").expect("valid fix identity"),
                "insert the expected token",
            )
            .expect("valid informational fix"),
        ],
        explanation: Some(
            ExplanationRef::new("sink.explain_unexpected_token").expect("valid explanation ref"),
        ),
    })
    .expect("valid rich draft")
}

fn snapshot_id(byte: u8) -> BuildSnapshotId {
    let hex = format!("{byte:02x}");
    BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        hex.repeat(32)
    ))
    .expect("test snapshot id is valid")
}

fn source_id(snapshot: BuildSnapshotId) -> SourceId {
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id allocation succeeds")
}
