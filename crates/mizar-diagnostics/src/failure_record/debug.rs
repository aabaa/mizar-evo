//! Deterministic debug rendering for diagnostic records.

use mizar_session::BuildSnapshotId;

use crate::{explain::ExplanationHandle, fix::FixSuggestion};

use super::{
    DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticFreshness,
    DiagnosticHandle, DiagnosticNote, DiagnosticRecord, DiagnosticSpan, SpanFreshness,
    ZeroWidthSpanIntent, source_id_debug,
};

pub(super) enum DebugSnapshot<'a> {
    Draft(&'a DiagnosticDraft),
    Record(&'a DiagnosticRecord),
}

pub(super) fn render_debug_snapshot(snapshot: DebugSnapshot<'_>) -> String {
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
                render_explanation(draft.explanation.as_ref(), "unpublished")
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
                render_explanation(record.explanation.as_ref(), &render_handle(record.handle))
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

fn render_explanation(explanation: Option<&ExplanationHandle>, diagnostic: &str) -> String {
    explanation.map_or_else(
        || "none".to_owned(),
        |explanation| {
            escape_embedded_snapshot(&explanation.debug_snapshot_with_diagnostic(diagnostic))
        },
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
