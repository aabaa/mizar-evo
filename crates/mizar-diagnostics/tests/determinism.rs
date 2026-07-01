use std::{collections::HashMap, str::FromStr};

use mizar_diagnostics::{
    aggregator::BuildDiagnosticIndex,
    explain::{
        ExplanationHandle, ExplanationHandleId, ExplanationHandleInput, ExplanationKind,
        ExplanationPreview, ExplanationPreviewFormat, ExplanationSourceRef, ExplanationSubject,
    },
    failure_record::{
        DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticNote, DiagnosticNoteKind, DiagnosticSpan, FailureCategory, PipelinePhase,
    },
    fix::{
        FixApplicability, FixEdit, FixSafety, FixSuggestion, FixSuggestionId, FixSuggestionInput,
    },
    registry::DiagnosticCode,
    render::{DiagnosticRenderInput, DiagnosticSourceContext, RenderOptions, render_diagnostics},
    sink::{DiagnosticBatch, DiagnosticProducerScope, DiagnosticSink},
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, LineColumn, LineColumnRange,
    SessionIdAllocator, SourceId, SourceRange,
};

#[test]
fn semantic_inputs_have_deterministic_records_indexes_rendering_and_explanation_previews() {
    let forward = scenario(ScenarioOrder::Forward);
    let reversed = scenario(ScenarioOrder::Reversed);

    assert_eq!(
        forward.index.debug_snapshot(),
        reversed.index.debug_snapshot()
    );
    assert_eq!(
        record_snapshots(&forward.index),
        record_snapshots(&reversed.index)
    );
    assert_eq!(forward.rendered, reversed.rendered);
    assert_eq!(forward.explanation_previews, reversed.explanation_previews);

    assert_eq!(forward.index.len(), 2);
    assert_eq!(
        forward
            .index
            .records()
            .iter()
            .map(|record| (record.handle().id.get(), record.code().to_string()))
            .collect::<Vec<_>>(),
        vec![(0, "E0001".to_owned()), (1, "E0201".to_owned())]
    );
    assert_eq!(
        forward.explanation_previews,
        vec![PreviewSnapshot {
            id: "det.explain.unexpected".to_owned(),
            text: "first line\nsecond [truncated]".to_owned(),
            truncated: true,
            byte_len: 29,
            line_count: 2,
        }]
    );
    assert_eq!(
        forward.rendered,
        concat!(
            "error[E0001]: unexpected token (syntax.unexpected_token)\n",
            "  --> src/a.miz:1:1\n",
            "   |\n",
            " 1 | alpha\n",
            "   | ^^^^^ unexpected token\n",
            "  --> src/a.miz:2:1\n",
            "   |\n",
            " 2 | beta\n",
            "   | ---- recovery anchor\n",
            "   = note: recovered before next block\n",
            "   = help: fix suggestion `det.fix.insert`: insert missing token (machine_applicable local_text_edit, 1 edit)\n",
            "   = help: fix suggestion `det.fix.reference`: see grammar reference\n",
            "   = explain: `det.explain.unexpected`: first line\n",
            "second [truncated]\n",
            "\n",
            "error[E0201]: ambiguous symbol (resolve.ambiguous_symbol)\n",
            "  --> src/b.miz:1:5\n",
            "   |\n",
            " 1 | let gamma\n",
            "   |     ^^^^^ ambiguous symbol",
        )
    );
}

#[derive(Clone, Copy)]
enum ScenarioOrder {
    Forward,
    Reversed,
}

struct Scenario {
    index: BuildDiagnosticIndex,
    rendered: String,
    explanation_previews: Vec<PreviewSnapshot>,
}

#[derive(Debug, Eq, PartialEq)]
struct PreviewSnapshot {
    id: String,
    text: String,
    truncated: bool,
    byte_len: usize,
    line_count: usize,
}

fn scenario(order: ScenarioOrder) -> Scenario {
    let snapshot = snapshot_id(1);
    let allocator = InMemorySessionIdAllocator::new();
    let source_a = allocator
        .next_source_id(snapshot)
        .expect("first source id allocation succeeds");
    let source_b = allocator
        .next_source_id(snapshot)
        .expect("second source id allocation succeeds");
    let parser = parser_draft(snapshot, source_a);
    let resolver = resolver_draft(snapshot, source_b);
    let batches = match order {
        ScenarioOrder::Forward => vec![
            batch(snapshot, PipelinePhase::Parser, "parser.det", vec![parser]),
            batch(
                snapshot,
                PipelinePhase::Resolver,
                "resolver.det",
                vec![resolver],
            ),
        ],
        ScenarioOrder::Reversed => vec![
            batch(
                snapshot,
                PipelinePhase::Resolver,
                "resolver.det",
                vec![resolver],
            ),
            batch(snapshot, PipelinePhase::Parser, "parser.det", vec![parser]),
        ],
    };
    let index = BuildDiagnosticIndex::from_batches(snapshot, batches).expect("index builds");
    let context = TestSourceContext::new()
        .with_source(source_a, "src/a.miz", "Src(A)", "alpha\nbeta\n")
        .with_source(source_b, "src/b.miz", "Src(B)", "let gamma\n");
    let rendered = render_diagnostics(DiagnosticRenderInput::new(
        index.records(),
        &context,
        RenderOptions::plain(),
    ));
    let explanation_previews = index
        .records()
        .iter()
        .filter_map(|record| {
            let explanation = record.explanation()?;
            let preview = explanation.preview()?;
            Some(PreviewSnapshot {
                id: explanation.id().identity().to_owned(),
                text: preview.text().to_owned(),
                truncated: preview.truncated(),
                byte_len: preview.byte_len(),
                line_count: preview.line_count(),
            })
        })
        .collect();

    Scenario {
        index,
        rendered,
        explanation_previews,
    }
}

fn parser_draft(snapshot: BuildSnapshotId, source_id: SourceId) -> DiagnosticDraft {
    let primary_span = DiagnosticSpan::primary(
        SourceRange {
            source_id,
            start: 0,
            end: 5,
        },
        None,
    )
    .expect("valid primary span");
    let secondary_span = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 6,
            end: 10,
        },
        Some("recovery anchor".to_owned()),
    )
    .expect("valid secondary span");
    DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_str("E0001").expect("allocated code"),
        phase: PipelinePhase::Parser,
        category: FailureCategory::ParseError,
        stable_detail_key: "syntax.unexpected_token".to_owned(),
        message: "unexpected token".to_owned(),
        primary_span,
        secondary_spans: vec![secondary_span],
        notes: vec![DiagnosticNote::new(
            DiagnosticNoteKind::Note,
            "recovered before next block",
            None,
        )],
        details: DiagnosticDetails::from_entries([
            ("det.expected_count", DiagnosticDetailValue::Integer(2)),
            ("det.recovered", DiagnosticDetailValue::Boolean(true)),
        ])
        .expect("valid details"),
        fixes: vec![
            informational_fix("det.fix.reference", "see grammar reference"),
            text_fix(snapshot, source_id),
        ],
        explanation: Some(explanation(snapshot)),
    })
    .expect("valid parser draft")
}

fn resolver_draft(snapshot: BuildSnapshotId, source_id: SourceId) -> DiagnosticDraft {
    let primary_span = DiagnosticSpan::primary(
        SourceRange {
            source_id,
            start: 4,
            end: 9,
        },
        None,
    )
    .expect("valid primary span");
    DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: snapshot,
        code: DiagnosticCode::from_str("E0201").expect("allocated code"),
        phase: PipelinePhase::Resolver,
        category: FailureCategory::ResolveError,
        stable_detail_key: "resolve.ambiguous_symbol".to_owned(),
        message: "ambiguous symbol".to_owned(),
        primary_span,
        secondary_spans: vec![],
        notes: vec![],
        details: DiagnosticDetails::from_entries([
            ("det.candidates", DiagnosticDetailValue::Integer(2)),
            (
                "det.scope",
                DiagnosticDetailValue::String("local".to_owned()),
            ),
        ])
        .expect("valid details"),
        fixes: vec![],
        explanation: None,
    })
    .expect("valid resolver draft")
}

fn batch(
    snapshot: BuildSnapshotId,
    phase: PipelinePhase,
    producer_name: &'static str,
    drafts: Vec<DiagnosticDraft>,
) -> DiagnosticBatch {
    let mut sink =
        DiagnosticSink::new(DiagnosticProducerScope::new(phase, snapshot, producer_name));
    for draft in drafts {
        sink.emit(draft).expect("fixture draft matches sink scope");
    }
    sink.into_batch()
}

fn informational_fix(identity: &str, title: &str) -> FixSuggestion {
    FixSuggestion::informational(
        FixSuggestionId::new(identity).expect("valid fix identity"),
        title,
    )
    .expect("valid informational fix")
}

fn text_fix(snapshot: BuildSnapshotId, source_id: SourceId) -> FixSuggestion {
    FixSuggestion::new(FixSuggestionInput {
        id: FixSuggestionId::new("det.fix.insert").expect("valid fix identity"),
        producer_key: Some("det.parser".to_owned()),
        title: "insert missing token".to_owned(),
        applicability: FixApplicability::MachineApplicable,
        safety: FixSafety::LocalTextEdit,
        edits: vec![
            FixEdit::new(
                SourceRange {
                    source_id,
                    start: 5,
                    end: 5,
                },
                ";",
                Some(String::new()),
            )
            .expect("valid edit"),
        ],
        command: None,
        required_snapshot: Some(snapshot),
        required_text_hash: Some(Hash::from_bytes([2; Hash::BYTE_LEN])),
    })
    .expect("valid text fix")
}

fn explanation(snapshot: BuildSnapshotId) -> ExplanationHandle {
    ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new("det.explain.unexpected").expect("valid explanation id"),
        kind: ExplanationKind::DiagnosticContext,
        subject: ExplanationSubject::Diagnostic {
            code: DiagnosticCode::from_str("E0001").expect("allocated code"),
            stable_detail_key: "syntax.unexpected_token".to_owned(),
        },
        source: ExplanationSourceRef::QueryService {
            service_key: "det.explain".to_owned(),
            query_key: "det.unexpected".to_owned(),
        },
        required_snapshot: Some(snapshot),
        required_artifact_hash: None,
        summary_hash: Some(Hash::from_bytes([3; Hash::BYTE_LEN])),
        preview: Some(
            ExplanationPreview::with_bounds(
                ExplanationPreviewFormat::PlainText,
                "first line\nsecond line\nthird line",
                29,
                2,
            )
            .expect("valid explanation preview"),
        ),
    })
    .expect("valid explanation handle")
}

fn record_snapshots(index: &BuildDiagnosticIndex) -> Vec<String> {
    index
        .records()
        .iter()
        .map(|record| record.debug_snapshot())
        .collect()
}

#[derive(Default)]
struct TestSourceContext {
    sources: HashMap<SourceId, TestSource>,
}

impl TestSourceContext {
    fn new() -> Self {
        Self::default()
    }

    fn with_source(
        mut self,
        source_id: SourceId,
        path: &'static str,
        source_key: &'static str,
        text: &'static str,
    ) -> Self {
        self.sources.insert(
            source_id,
            TestSource {
                path,
                source_key,
                text,
                line_starts: line_starts(text),
            },
        );
        self
    }
}

impl DiagnosticSourceContext for TestSourceContext {
    fn path_for(&self, source: SourceId) -> Option<&str> {
        self.sources.get(&source).map(|source| source.path)
    }

    fn source_key_for(&self, source: SourceId) -> String {
        self.sources.get(&source).map_or_else(
            || format!("{source:?}"),
            |source| source.source_key.to_owned(),
        )
    }

    fn line_text(&self, source: SourceId, line: u32) -> Option<&str> {
        let source = self.sources.get(&source)?;
        let line_index = usize::try_from(line.checked_sub(1)?).ok()?;
        let start = *source.line_starts.get(line_index)?;
        let end = source
            .line_starts
            .get(line_index + 1)
            .copied()
            .unwrap_or(source.text.len());
        Some(source.text[start..end].trim_end_matches('\n'))
    }

    fn line_column(&self, range: SourceRange) -> Option<LineColumnRange> {
        let source = self.sources.get(&range.source_id)?;
        if range.end > source.text.len() || range.start > range.end {
            return None;
        }
        Some(LineColumnRange {
            start: line_column(source.text, &source.line_starts, range.start)?,
            end: line_column(source.text, &source.line_starts, range.end)?,
        })
    }
}

struct TestSource {
    path: &'static str,
    source_key: &'static str,
    text: &'static str,
    line_starts: Vec<usize>,
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' && index + 1 < text.len() {
            starts.push(index + 1);
        }
    }
    starts
}

fn line_column(text: &str, starts: &[usize], offset: usize) -> Option<LineColumn> {
    let line_index = match starts.binary_search(&offset) {
        Ok(index) => index,
        Err(0) => return None,
        Err(index) => index - 1,
    };
    let line_start = *starts.get(line_index)?;
    Some(LineColumn {
        line: u32::try_from(line_index + 1).ok()?,
        column: u32::try_from(text[line_start..offset].chars().count() + 1).ok()?,
    })
}

fn snapshot_id(byte: u8) -> BuildSnapshotId {
    let hex = format!("{byte:02x}");
    BuildSnapshotId::from_published_schema_str(&format!(
        "mizar-session-build-snapshot-v1:{}",
        hex.repeat(32)
    ))
    .expect("test snapshot id is valid")
}
