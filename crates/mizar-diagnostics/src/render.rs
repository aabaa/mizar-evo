//! Deterministic CLI diagnostic rendering.

use mizar_session::{LineColumnRange, SourceId, SourceRange};

use crate::{
    failure_record::{
        DiagnosticRecord, DiagnosticSpan, DiagnosticSpanRole, SpanFreshness, ZeroWidthSpanIntent,
    },
    registry::DiagnosticSeverity,
};

/// Source context supplied to CLI rendering.
pub trait DiagnosticSourceContext {
    /// Returns a display path for a source id.
    fn path_for(&self, source: SourceId) -> Option<&str>;

    /// Returns a deterministic fallback/debug source key.
    fn source_key_for(&self, source: SourceId) -> String;

    /// Returns one-based line text.
    fn line_text(&self, source: SourceId, line: u32) -> Option<&str>;

    /// Converts a compiler-native source range to one-based display coordinates.
    fn line_column(&self, range: SourceRange) -> Option<LineColumnRange>;
}

/// CLI rendering options.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderOptions {
    style: RenderStyle,
    multiline_context_limit: usize,
}

impl RenderOptions {
    /// Creates byte-stable plain rendering options.
    pub const fn plain() -> Self {
        Self {
            style: RenderStyle::Plain,
            multiline_context_limit: 2,
        }
    }

    /// Creates ANSI-styled rendering options.
    pub const fn ansi() -> Self {
        Self {
            style: RenderStyle::Ansi,
            multiline_context_limit: 2,
        }
    }

    /// Sets the maximum number of context lines before multiline spans elide.
    pub const fn with_multiline_context_limit(mut self, limit: usize) -> Self {
        self.multiline_context_limit = limit;
        self
    }

    /// Returns the selected render style.
    pub const fn style(self) -> RenderStyle {
        self.style
    }

    /// Returns the multiline context limit.
    pub const fn multiline_context_limit(self) -> usize {
        self.multiline_context_limit
    }
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self::plain()
    }
}

/// Output style for CLI rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderStyle {
    /// Byte-stable plain text with no ANSI escapes.
    Plain,
    /// ANSI terminal styling.
    Ansi,
}

/// Input consumed by CLI diagnostic rendering.
pub struct DiagnosticRenderInput<'a> {
    records: &'a [DiagnosticRecord],
    source_context: &'a dyn DiagnosticSourceContext,
    options: RenderOptions,
}

impl<'a> DiagnosticRenderInput<'a> {
    /// Creates render input.
    pub const fn new(
        records: &'a [DiagnosticRecord],
        source_context: &'a dyn DiagnosticSourceContext,
        options: RenderOptions,
    ) -> Self {
        Self {
            records,
            source_context,
            options,
        }
    }

    /// Returns the records in render order.
    pub const fn records(&self) -> &'a [DiagnosticRecord] {
        self.records
    }

    /// Returns render options.
    pub const fn options(&self) -> RenderOptions {
        self.options
    }
}

/// Renders diagnostics for CLI output.
pub fn render_diagnostics(input: DiagnosticRenderInput<'_>) -> String {
    let blocks = input
        .records
        .iter()
        .map(|record| render_record(record, input.source_context, input.options))
        .collect::<Vec<_>>();
    blocks.join("\n\n")
}

fn render_record(
    record: &DiagnosticRecord,
    context: &dyn DiagnosticSourceContext,
    options: RenderOptions,
) -> String {
    let mut lines = Vec::new();
    lines.push(render_header(record, options.style));
    lines.extend(render_span_block(
        record.primary_span(),
        context,
        '^',
        label_or_message(record.primary_span(), record.message()),
        options,
    ));

    for span in record.secondary_spans() {
        lines.extend(render_span_block(
            span,
            context,
            '-',
            span.label().unwrap_or(""),
            options,
        ));
    }

    for note in record.notes() {
        if let Some(span) = note.span() {
            lines.extend(render_span_block(
                span,
                context,
                '-',
                label_or_message(span, note.message()),
                options,
            ));
        }
        lines.push(format!("   = {}: {}", note.kind().as_str(), note.message()));
    }

    for fix in record.fixes() {
        lines.push(render_fix_help(fix));
    }
    if let Some(explanation) = record.explanation() {
        lines.push(render_explanation(explanation));
    }

    lines.join("\n")
}

fn render_header(record: &DiagnosticRecord, style: RenderStyle) -> String {
    let header = format!(
        "{}[{}]: {} ({})",
        render_severity(record.severity()),
        record.code(),
        record.message(),
        record.semantic_name()
    );
    match style {
        RenderStyle::Plain => header,
        RenderStyle::Ansi => match record.severity() {
            DiagnosticSeverity::Error => format!("\x1b[31m{header}\x1b[0m"),
            DiagnosticSeverity::Warning => format!("\x1b[33m{header}\x1b[0m"),
            DiagnosticSeverity::Info => format!("\x1b[36m{header}\x1b[0m"),
        },
    }
}

fn render_fix_help(fix: &crate::fix::FixSuggestion) -> String {
    let title = if fix.title().is_empty() {
        fix.id().identity()
    } else {
        fix.title()
    };
    let mut help = format!(
        "   = help: fix suggestion `{}`: {title}",
        fix.id().identity()
    );
    if !fix.edits().is_empty() {
        help.push_str(&format!(
            " ({} {}, {} edit{})",
            fix.applicability(),
            fix.safety(),
            fix.edits().len(),
            if fix.edits().len() == 1 { "" } else { "s" }
        ));
    } else if let Some(command) = fix.command() {
        help.push_str(&format!(
            " ({} command `{}`)",
            fix.safety(),
            command.identity()
        ));
    }
    help
}

fn render_explanation(explanation: &crate::explain::ExplanationHandle) -> String {
    let mut line = format!("   = explain: `{}`", explanation.id().identity());
    if let Some(preview) = explanation.preview()
        && !preview.text().is_empty()
    {
        line.push_str(": ");
        line.push_str(preview.text());
    }
    line
}

fn render_severity(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Error => "error",
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Info => "info",
    }
}

fn render_span_block(
    span: &DiagnosticSpan,
    context: &dyn DiagnosticSourceContext,
    marker: char,
    label: &str,
    options: RenderOptions,
) -> Vec<String> {
    let range = span.range();
    let Some(line_column) = context.line_column(range) else {
        return vec![render_missing_source(span, context, label)];
    };
    let path = context
        .path_for(range.source_id)
        .map_or_else(|| context.source_key_for(range.source_id), str::to_owned);
    let mut lines = vec![
        format!(
            "  --> {}:{}:{}",
            path, line_column.start.line, line_column.start.column
        ),
        "   |".to_owned(),
    ];

    if line_column.start.line == line_column.end.line {
        let Some(line_text) = context.line_text(range.source_id, line_column.start.line) else {
            return vec![render_missing_source(span, context, label)];
        };
        lines.extend(render_line_with_underline(
            line_column.start.line,
            line_text,
            line_column.start.column,
            line_column.end.column,
            marker,
            label,
            span.zero_width(),
        ));
        return lines;
    }

    let span_lines = line_column
        .end
        .line
        .saturating_sub(line_column.start.line)
        .saturating_add(1);
    let should_elide = usize::try_from(span_lines)
        .map_or(true, |count| count > options.multiline_context_limit.max(2));
    let Some(start_text) = context.line_text(range.source_id, line_column.start.line) else {
        return vec![render_missing_source(span, context, label)];
    };
    let Some(end_text) = context.line_text(range.source_id, line_column.end.line) else {
        return vec![render_missing_source(span, context, label)];
    };
    lines.extend(render_line_with_underline(
        line_column.start.line,
        start_text,
        line_column.start.column,
        display_end_column(start_text),
        marker,
        label,
        None,
    ));
    if should_elide {
        lines.push("   | ...".to_owned());
    } else if line_column.end.line > line_column.start.line + 1 {
        for line in (line_column.start.line + 1)..line_column.end.line {
            let Some(text) = context.line_text(range.source_id, line) else {
                return vec![render_missing_source(span, context, label)];
            };
            lines.extend(render_line_with_underline(
                line,
                text,
                1,
                display_end_column(text),
                marker,
                "",
                None,
            ));
        }
    }
    lines.extend(render_line_with_underline(
        line_column.end.line,
        end_text,
        1,
        line_column.end.column.max(1),
        marker,
        "",
        None,
    ));
    lines
}

fn render_line_with_underline(
    line: u32,
    text: &str,
    start_column: u32,
    end_column: u32,
    marker: char,
    label: &str,
    zero_width: Option<ZeroWidthSpanIntent>,
) -> Vec<String> {
    let width = line.to_string().len().max(2);
    let mut lines = Vec::new();
    lines.push(format!("{line:>width$} | {text}"));
    let underline_start = usize::try_from(start_column.saturating_sub(1)).unwrap_or(usize::MAX);
    let underline_len = underline_len(start_column, end_column, zero_width);
    let underline = marker.to_string().repeat(underline_len);
    let suffix = if label.is_empty() {
        String::new()
    } else {
        format!(" {label}")
    };
    lines.push(format!(
        "{:>width$} | {}{}{}",
        "",
        " ".repeat(underline_start),
        underline,
        suffix
    ));
    lines
}

fn display_end_column(text: &str) -> u32 {
    u32::try_from(text.chars().count() + 1).unwrap_or(u32::MAX)
}

fn underline_len(
    start_column: u32,
    end_column: u32,
    zero_width: Option<ZeroWidthSpanIntent>,
) -> usize {
    if zero_width.is_some() || end_column <= start_column {
        return 1;
    }
    usize::try_from(end_column - start_column)
        .unwrap_or(usize::MAX)
        .max(1)
}

fn render_missing_source(
    span: &DiagnosticSpan,
    context: &dyn DiagnosticSourceContext,
    label: &str,
) -> String {
    let range = span.range();
    let source = context.source_key_for(range.source_id);
    let role = match span.role() {
        DiagnosticSpanRole::Primary => "primary",
        DiagnosticSpanRole::Secondary => "secondary",
        DiagnosticSpanRole::DefinitionSite => "definition_site",
        DiagnosticSpanRole::Related => "related",
    };
    let freshness = match span.freshness() {
        SpanFreshness::Current => "current".to_owned(),
        SpanFreshness::Stale { reason } => format!("stale({reason})"),
        SpanFreshness::Historical => "historical".to_owned(),
    };
    if label.is_empty() {
        format!(
            "  --> source {source}:{}..{} ({role}, {freshness})",
            range.start, range.end
        )
    } else {
        format!(
            "  --> source {source}:{}..{} ({role}, {freshness}): {label}",
            range.start, range.end
        )
    }
}

fn label_or_message<'a>(span: &'a DiagnosticSpan, message: &'a str) -> &'a str {
    span.label().unwrap_or(message)
}
