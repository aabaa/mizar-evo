use std::{collections::HashMap, str::FromStr};

use mizar_diagnostics::{
    explain::{
        ExplanationHandle, ExplanationHandleId, ExplanationHandleInput, ExplanationKind,
        ExplanationPreview, ExplanationPreviewFormat, ExplanationSourceRef, ExplanationSubject,
    },
    failure_record::{
        DiagnosticDetailValue, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticFreshness, DiagnosticHandle, DiagnosticId, DiagnosticNote, DiagnosticNoteKind,
        DiagnosticSpan, DiagnosticSpanRole, FailureCategory, PipelinePhase, SpanFreshness,
        ZeroWidthSpanIntent,
    },
    fix::{
        FixApplicability, FixCommandRef, FixEdit, FixSafety, FixSuggestion, FixSuggestionId,
        FixSuggestionInput,
    },
    registry::DiagnosticCode,
    render::{
        DiagnosticRenderInput, DiagnosticSourceContext, RenderOptions, RenderStyle,
        render_diagnostics,
    },
};
use mizar_session::{
    BuildSnapshotId, Hash, InMemorySessionIdAllocator, LineColumn, LineColumnRange,
    SessionIdAllocator, SourceId, SourceRange,
};

#[test]
fn plain_rendering_is_byte_stable_for_primary_secondary_notes_and_refs() {
    let snapshot = snapshot_id(1);
    let source_id = source_id(snapshot);
    let context = TestSourceContext::new().with_source(
        source_id,
        "src/lib.miz",
        "Src(1)",
        "theorem Main:\n  a + b\nend\n",
    );
    let secondary = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 0,
            end: 7,
        },
        Some("declaration starts here".to_owned()),
    )
    .expect("valid secondary span");
    let note_span = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 22,
            end: 25,
        },
        Some("closing token".to_owned()),
    )
    .expect("valid note span");
    let record = record(
        RecordFixture::new(snapshot, source_id, 16, 21, "ambiguous addition")
            .secondary_spans(vec![secondary])
            .notes(vec![
                DiagnosticNote::new(DiagnosticNoteKind::Note, "two candidates remain", None),
                DiagnosticNote::new(
                    DiagnosticNoteKind::Help,
                    "qualify one candidate",
                    Some(note_span),
                ),
            ])
            .fixes(vec![informational_fix(
                "render.qualify",
                "qualify one candidate",
            )])
            .explanation(explanation("render.explain", "show overload candidates")),
    );

    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&record),
            &context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: ambiguous addition (syntax.unexpected_token)\n",
            "  --> src/lib.miz:2:3\n",
            "   |\n",
            " 2 |   a + b\n",
            "   |   ^^^^^ ambiguous addition\n",
            "  --> src/lib.miz:1:1\n",
            "   |\n",
            " 1 | theorem Main:\n",
            "   | ------- declaration starts here\n",
            "   = note: two candidates remain\n",
            "  --> src/lib.miz:3:1\n",
            "   |\n",
            " 3 | end\n",
            "   | --- closing token\n",
            "   = help: qualify one candidate\n",
            "   = help: fix suggestion `render.qualify`: qualify one candidate\n",
            "   = explain: `render.explain`: show overload candidates",
        )
    );
}

#[test]
fn fix_rendering_projects_edit_and_command_metadata_without_application_payloads() {
    let snapshot = snapshot_id(7);
    let source_id = source_id(snapshot);
    let context = TestSourceContext::new().with_source(source_id, "fix.miz", "Src(7)", "abc\n");
    let record = record(
        RecordFixture::new(snapshot, source_id, 0, 1, "fixes")
            .fixes(vec![command_fix(), text_edit_fix(source_id)]),
    );
    let rendered = render_diagnostics(DiagnosticRenderInput::new(
        std::slice::from_ref(&record),
        &context,
        RenderOptions::plain(),
    ));

    assert_eq!(
        rendered,
        concat!(
            "error[E0001]: fixes (syntax.unexpected_token)\n",
            "  --> fix.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ fixes\n",
            "   = help: fix suggestion `render.command`: open command (command_only command `render.open_command`)\n",
            "   = help: fix suggestion `render.insert`: insert token (machine_applicable local_text_edit, 1 edit)",
        )
    );
    assert!(!rendered.contains("WorkspaceEdit"));
    assert!(!rendered.contains("replacement"));
    assert!(!rendered.contains("expected_text"));
}

#[test]
fn explanation_rendering_projects_only_handle_id_and_bounded_preview_without_resolution_payloads() {
    let snapshot = snapshot_id(8);
    let source_id = source_id(snapshot);
    let context =
        TestSourceContext::new().with_source(source_id, "explain.miz", "Src(8)", "abc\ndef\n");
    let artifact_record = record(
        RecordFixture::new(snapshot, source_id, 0, 1, "artifact explanation")
            .explanation(backing_explanation(snapshot, true)),
    );
    let query_record = record(
        RecordFixture::new(snapshot, source_id, 4, 7, "query explanation")
            .explanation(query_explanation()),
    );
    let rendered = render_diagnostics(DiagnosticRenderInput::new(
        &[artifact_record, query_record],
        &context,
        RenderOptions::plain(),
    ));

    assert_eq!(
        rendered,
        concat!(
            "error[E0001]: artifact explanation (syntax.unexpected_token)\n",
            "  --> explain.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ artifact explanation\n",
            "   = explain: `render.artifact_explain`: bounded preview\n",
            "\n",
            "error[E0001]: query explanation (syntax.unexpected_token)\n",
            "  --> explain.miz:2:1\n",
            "   |\n",
            " 2 | def\n",
            "   | ^^^ query explanation\n",
            "   = explain: `render.query_explain`",
        )
    );
    for forbidden in [
        "explain/proof.json",
        "0101010101010101010101010101010101010101010101010101010101010101",
        "0202020202020202020202020202020202020202020202020202020202020202",
        "required_snapshot",
        "artifact payload",
        "WorkspaceEdit",
        "jsonrpc",
        "mizar/explain",
        "replacement",
        "expected_text",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "rendered explanation must not leak `{forbidden}`"
        );
    }
}

#[test]
fn missing_source_context_uses_deterministic_fallback() {
    let snapshot = snapshot_id(2);
    let source_id = source_id(snapshot);
    let context = TestSourceContext::new().with_source_key(source_id, "source-key");
    let record = record(RecordFixture::new(snapshot, source_id, 1, 4, "fallback"));

    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&record),
            &context,
            RenderOptions::plain(),
        )),
        "error[E0001]: fallback (syntax.unexpected_token)\n  --> source source-key:1..4 \
         (primary, current): fallback"
    );
}

#[test]
fn multiple_diagnostics_use_one_blank_line_separator_and_input_order() {
    let snapshot = snapshot_id(3);
    let source_id = source_id(snapshot);
    let context = TestSourceContext::new().with_source(source_id, "a.miz", "Src(3)", "abc\ndef\n");
    let first = record(RecordFixture::new(snapshot, source_id, 0, 1, "first"));
    let second = record(RecordFixture::new(snapshot, source_id, 4, 7, "second"));

    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            &[first, second],
            &context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: first (syntax.unexpected_token)\n",
            "  --> a.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ first\n",
            "\n",
            "error[E0001]: second (syntax.unexpected_token)\n",
            "  --> a.miz:2:1\n",
            "   |\n",
            " 2 | def\n",
            "   | ^^^ second",
        )
    );
}

#[test]
fn unicode_scalar_columns_and_multiline_context_are_byte_stable() {
    let snapshot = snapshot_id(4);
    let source_id = source_id(snapshot);
    let unicode_context =
        TestSourceContext::new().with_source(source_id, "unicode.miz", "Src(4)", "αβγ\n");
    let unicode_record = record(RecordFixture::new(
        snapshot,
        source_id,
        2,
        6,
        "unicode span",
    ));

    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&unicode_record),
            &unicode_context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: unicode span (syntax.unexpected_token)\n",
            "  --> unicode.miz:1:2\n",
            "   |\n",
            " 1 | αβγ\n",
            "   |  ^^ unicode span",
        )
    );

    let multiline_context = TestSourceContext::new().with_source(
        source_id,
        "multi.miz",
        "Src(4)",
        "one\ntwo\nthree\nfour\n",
    );
    let multiline_record = record(RecordFixture::new(
        snapshot,
        source_id,
        0,
        13,
        "multiline span",
    ));
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&multiline_record),
            &multiline_context,
            RenderOptions::plain().with_multiline_context_limit(4),
        )),
        concat!(
            "error[E0001]: multiline span (syntax.unexpected_token)\n",
            "  --> multi.miz:1:1\n",
            "   |\n",
            " 1 | one\n",
            "   | ^^^ multiline span\n",
            " 2 | two\n",
            "   | ^^^\n",
            " 3 | three\n",
            "   | ^^^^^",
        )
    );
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&multiline_record),
            &multiline_context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: multiline span (syntax.unexpected_token)\n",
            "  --> multi.miz:1:1\n",
            "   |\n",
            " 1 | one\n",
            "   | ^^^ multiline span\n",
            "   | ...\n",
            " 3 | three\n",
            "   | ^^^^^",
        )
    );
}

#[test]
fn source_key_missing_text_cause_related_and_zero_width_rendering_are_stable() {
    let snapshot = snapshot_id(5);
    let source_id = source_id(snapshot);
    let context =
        TestSourceContext::new().with_source_without_path(source_id, "Src(5)", "abc\ndef\n");
    let path_fallback_record = record(
        RecordFixture::new(snapshot, source_id, 0, 1, "path fallback").notes(vec![
            DiagnosticNote::new(DiagnosticNoteKind::Cause, "because of context", None),
            DiagnosticNote::new(DiagnosticNoteKind::Related, "see also", None),
        ]),
    );
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&path_fallback_record),
            &context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: path fallback (syntax.unexpected_token)\n",
            "  --> Src(5):1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ path fallback\n",
            "   = cause: because of context\n",
            "   = related: see also",
        )
    );

    let secondary = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 2,
            end: 4,
        },
        Some("secondary fallback".to_owned()),
    )
    .expect("valid secondary span");
    let note_span = DiagnosticSpan::secondary(
        SourceRange {
            source_id,
            start: 4,
            end: 5,
        },
        Some("note fallback".to_owned()),
    )
    .expect("valid note span");
    let missing_text_context = MissingTextContext { source_id };
    let missing_text_record = record(
        RecordFixture::new(snapshot, source_id, 0, 1, "missing text")
            .secondary_spans(vec![secondary])
            .notes(vec![DiagnosticNote::new(
                DiagnosticNoteKind::Note,
                "anchored note",
                Some(note_span),
            )]),
    );
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&missing_text_record),
            &missing_text_context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: missing text (syntax.unexpected_token)\n",
            "  --> source MissingText:0..1 (primary, current): missing text\n",
            "  --> source MissingText:2..4 (secondary, current): secondary fallback\n",
            "  --> source MissingText:4..5 (secondary, current): note fallback\n",
            "   = note: anchored note",
        )
    );

    let zero_width = DiagnosticSpan::new(
        SourceRange {
            source_id,
            start: 1,
            end: 1,
        },
        DiagnosticSpanRole::Primary,
        Some("insert here".to_owned()),
        SpanFreshness::Current,
        Some(ZeroWidthSpanIntent::InsertionPoint),
    )
    .expect("valid zero-width primary span");
    let zero_width_record =
        record(RecordFixture::new(snapshot, source_id, 0, 1, "unused").primary_span(zero_width));
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&zero_width_record),
            &context,
            RenderOptions::plain(),
        )),
        concat!(
            "error[E0001]: unused (syntax.unexpected_token)\n",
            "  --> Src(5):1:2\n",
            "   |\n",
            " 1 | abc\n",
            "   |  ^ insert here",
        )
    );
}

#[test]
fn ansi_style_wraps_header_only_and_uses_severity_colors() {
    let snapshot = snapshot_id(6);
    let source_id = source_id(snapshot);
    let context = TestSourceContext::new().with_source(source_id, "a.miz", "Src(6)", "abc\n");
    let error_record = record(RecordFixture::new(
        snapshot,
        source_id,
        0,
        1,
        "error styled",
    ));
    let warning_record = record(
        RecordFixture::new(snapshot, source_id, 0, 1, "warning styled")
            .code("W0001")
            .stable_detail_key("warn.unused_variable")
            .category(FailureCategory::CompatibilityWarning),
    );

    assert!(matches!(RenderOptions::ansi().style(), RenderStyle::Ansi));
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&error_record),
            &context,
            RenderOptions::ansi(),
        )),
        concat!(
            "\u{1b}[31merror[E0001]: error styled (syntax.unexpected_token)\u{1b}[0m\n",
            "  --> a.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ error styled",
        )
    );
    assert_eq!(
        render_diagnostics(DiagnosticRenderInput::new(
            std::slice::from_ref(&warning_record),
            &context,
            RenderOptions::ansi(),
        )),
        concat!(
            "\u{1b}[33mwarning[W0001]: warning styled (warn.unused_variable)\u{1b}[0m\n",
            "  --> a.miz:1:1\n",
            "   |\n",
            " 1 | abc\n",
            "   | ^ warning styled",
        )
    );
}

#[derive(Clone, Copy)]
struct MissingTextContext {
    source_id: SourceId,
}

impl DiagnosticSourceContext for MissingTextContext {
    fn path_for(&self, _source: SourceId) -> Option<&str> {
        None
    }

    fn source_key_for(&self, _source: SourceId) -> String {
        "MissingText".to_owned()
    }

    fn line_text(&self, _source: SourceId, _line: u32) -> Option<&str> {
        None
    }

    fn line_column(&self, range: SourceRange) -> Option<LineColumnRange> {
        if range.source_id != self.source_id {
            return None;
        }
        Some(LineColumnRange {
            start: LineColumn { line: 1, column: 1 },
            end: LineColumn { line: 1, column: 2 },
        })
    }
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
                path: Some(path),
                source_key,
                text: Some(text),
                line_starts: line_starts(text),
            },
        );
        self
    }

    fn with_source_without_path(
        mut self,
        source_id: SourceId,
        source_key: &'static str,
        text: &'static str,
    ) -> Self {
        self.sources.insert(
            source_id,
            TestSource {
                path: None,
                source_key,
                text: Some(text),
                line_starts: line_starts(text),
            },
        );
        self
    }

    fn with_source_key(mut self, source_id: SourceId, source_key: &'static str) -> Self {
        self.sources.insert(
            source_id,
            TestSource {
                path: None,
                source_key,
                text: None,
                line_starts: vec![],
            },
        );
        self
    }
}

impl DiagnosticSourceContext for TestSourceContext {
    fn path_for(&self, source: SourceId) -> Option<&str> {
        self.sources.get(&source).and_then(|source| source.path)
    }

    fn source_key_for(&self, source: SourceId) -> String {
        self.sources.get(&source).map_or_else(
            || format!("{source:?}"),
            |source| source.source_key.to_owned(),
        )
    }

    fn line_text(&self, source: SourceId, line: u32) -> Option<&str> {
        let source = self.sources.get(&source)?;
        let text = source.text?;
        let line_index = usize::try_from(line.checked_sub(1)?).ok()?;
        let start = *source.line_starts.get(line_index)?;
        let end = source
            .line_starts
            .get(line_index + 1)
            .copied()
            .unwrap_or(text.len());
        Some(text[start..end].trim_end_matches('\n'))
    }

    fn line_column(&self, range: SourceRange) -> Option<LineColumnRange> {
        let source = self.sources.get(&range.source_id)?;
        let text = source.text?;
        if range.end > text.len() || range.start > range.end {
            return None;
        }
        Some(LineColumnRange {
            start: line_column(text, &source.line_starts, range.start)?,
            end: line_column(text, &source.line_starts, range.end)?,
        })
    }
}

struct TestSource {
    path: Option<&'static str>,
    source_key: &'static str,
    text: Option<&'static str>,
    line_starts: Vec<usize>,
}

struct RecordFixture {
    snapshot: BuildSnapshotId,
    source_id: SourceId,
    start: usize,
    end: usize,
    message: &'static str,
    code: &'static str,
    category: FailureCategory,
    stable_detail_key: &'static str,
    primary_span: Option<DiagnosticSpan>,
    secondary_spans: Vec<DiagnosticSpan>,
    notes: Vec<DiagnosticNote>,
    fixes: Vec<FixSuggestion>,
    explanation: Option<ExplanationHandle>,
}

impl RecordFixture {
    fn new(
        snapshot: BuildSnapshotId,
        source_id: SourceId,
        start: usize,
        end: usize,
        message: &'static str,
    ) -> Self {
        Self {
            snapshot,
            source_id,
            start,
            end,
            message,
            code: "E0001",
            category: FailureCategory::ParseError,
            stable_detail_key: "syntax.unexpected_token",
            primary_span: None,
            secondary_spans: Vec::new(),
            notes: Vec::new(),
            fixes: Vec::new(),
            explanation: None,
        }
    }

    fn code(mut self, code: &'static str) -> Self {
        self.code = code;
        self
    }

    fn category(mut self, category: FailureCategory) -> Self {
        self.category = category;
        self
    }

    fn stable_detail_key(mut self, stable_detail_key: &'static str) -> Self {
        self.stable_detail_key = stable_detail_key;
        self
    }

    fn primary_span(mut self, primary_span: DiagnosticSpan) -> Self {
        self.primary_span = Some(primary_span);
        self
    }

    fn secondary_spans(mut self, secondary_spans: Vec<DiagnosticSpan>) -> Self {
        self.secondary_spans = secondary_spans;
        self
    }

    fn notes(mut self, notes: Vec<DiagnosticNote>) -> Self {
        self.notes = notes;
        self
    }

    fn fixes(mut self, fixes: Vec<FixSuggestion>) -> Self {
        self.fixes = fixes;
        self
    }

    fn explanation(mut self, explanation: ExplanationHandle) -> Self {
        self.explanation = Some(explanation);
        self
    }
}

fn record(fixture: RecordFixture) -> mizar_diagnostics::failure_record::DiagnosticRecord {
    let draft = DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: fixture.snapshot,
        code: DiagnosticCode::from_str(fixture.code).expect("allocated code"),
        phase: PipelinePhase::Parser,
        category: fixture.category,
        stable_detail_key: fixture.stable_detail_key.to_owned(),
        message: fixture.message.to_owned(),
        primary_span: fixture.primary_span.unwrap_or_else(|| {
            DiagnosticSpan::primary(
                SourceRange {
                    source_id: fixture.source_id,
                    start: fixture.start,
                    end: fixture.end,
                },
                None,
            )
            .expect("valid primary span")
        }),
        secondary_spans: fixture.secondary_spans,
        notes: fixture.notes,
        details: DiagnosticDetails::from_entries([(
            "render.case",
            DiagnosticDetailValue::Integer(1),
        )])
        .expect("valid details"),
        fixes: fixture.fixes,
        explanation: fixture.explanation,
    })
    .expect("valid draft");
    mizar_diagnostics::failure_record::DiagnosticRecord::from_draft(
        draft,
        DiagnosticHandle::new(fixture.snapshot, DiagnosticId::new(0)),
        DiagnosticFreshness::Current {
            source_snapshot: fixture.snapshot,
        },
        vec![],
    )
    .expect("valid record")
}

fn informational_fix(identity: &str, title: &str) -> FixSuggestion {
    FixSuggestion::informational(
        FixSuggestionId::new(identity).expect("valid fix identity"),
        title,
    )
    .expect("valid informational fix")
}

fn text_edit_fix(source_id: SourceId) -> FixSuggestion {
    FixSuggestion::local_text_edit(
        FixSuggestionId::new("render.insert").expect("valid fix identity"),
        "insert token",
        FixApplicability::MachineApplicable,
        vec![
            FixEdit::new(
                SourceRange {
                    source_id,
                    start: 1,
                    end: 1,
                },
                "x",
                Some(String::new()),
            )
            .expect("valid edit"),
        ],
    )
    .expect("valid text edit fix")
}

fn command_fix() -> FixSuggestion {
    FixSuggestion::new(FixSuggestionInput {
        id: FixSuggestionId::new("render.command").expect("valid fix identity"),
        producer_key: None,
        title: "open command".to_owned(),
        applicability: FixApplicability::MaybeIncorrect,
        safety: FixSafety::CommandOnly,
        edits: Vec::new(),
        command: Some(FixCommandRef::new("render.open_command").expect("valid command")),
        required_snapshot: None,
        required_text_hash: None,
    })
    .expect("valid command fix")
}

fn explanation(identity: &str, preview: &str) -> ExplanationHandle {
    ExplanationHandle::preview_only(
        ExplanationHandleId::new(identity).expect("valid explanation identity"),
        DiagnosticCode::from_str("E0001").expect("allocated code"),
        "syntax.unexpected_token",
        Some(ExplanationPreview::new(
            ExplanationPreviewFormat::PlainText,
            preview,
        )),
    )
    .expect("valid explanation handle")
}

fn backing_explanation(snapshot: BuildSnapshotId, with_preview: bool) -> ExplanationHandle {
    ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new("render.artifact_explain").expect("valid explanation id"),
        kind: ExplanationKind::ProofFailure,
        subject: ExplanationSubject::Diagnostic {
            code: DiagnosticCode::from_str("E0001").expect("allocated code"),
            stable_detail_key: "syntax.unexpected_token".to_owned(),
        },
        source: ExplanationSourceRef::Artifact {
            path: "explain/proof.json".to_owned(),
            content_hash: Hash::from_bytes([1; Hash::BYTE_LEN]),
        },
        required_snapshot: Some(snapshot),
        required_artifact_hash: Some(Hash::from_bytes([1; Hash::BYTE_LEN])),
        summary_hash: Some(Hash::from_bytes([2; Hash::BYTE_LEN])),
        preview: with_preview.then(|| {
            ExplanationPreview::new(ExplanationPreviewFormat::PlainText, "bounded preview")
        }),
    })
    .expect("valid backing explanation handle")
}

fn query_explanation() -> ExplanationHandle {
    ExplanationHandle::new(ExplanationHandleInput {
        id: ExplanationHandleId::new("render.query_explain").expect("valid explanation id"),
        kind: ExplanationKind::OverloadResolution,
        subject: ExplanationSubject::Expression("expr.render".to_owned()),
        source: ExplanationSourceRef::QueryService {
            service_key: "render.explain_service".to_owned(),
            query_key: "render.query".to_owned(),
        },
        required_snapshot: None,
        required_artifact_hash: None,
        summary_hash: None,
        preview: None,
    })
    .expect("valid query explanation handle")
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

fn source_id(snapshot: BuildSnapshotId) -> SourceId {
    InMemorySessionIdAllocator::new()
        .next_source_id(snapshot)
        .expect("source id allocation succeeds")
}
