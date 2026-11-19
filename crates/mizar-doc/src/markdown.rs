//! CommonMark and raw extension views over verified documentation attachments.

use std::{borrow::Cow, ops::Range};

use mizar_artifact::store::CanonicalJson;
use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetailValue as Value, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticPrimaryLocation, DiagnosticSpan, FailureCategory, PipelinePhase,
    },
    registry::{DiagnosticCode, DiagnosticRegistry, DiagnosticSeverity},
};
use mizar_session::{SourceRange, TextRange};
use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};

use crate::comments::{CommentAttachments, CommentBlock, CommentTarget};

/// Caller-supplied authoritative metadata for exactly the supplied target.
#[derive(Debug)]
pub struct TargetMetadata<'a> {
    /// Original verified target within the attachments' publication and module.
    pub target: CommentTarget<'a>,
    /// Raw parameter names, if available.
    pub parameters: Option<&'a [&'a str]>,
    /// Whether the supplied list can establish parameter absence.
    pub parameters_complete: bool,
    /// Whether a return value exists, when authoritatively known.
    pub has_return: Option<bool>,
}

/// Separate raw fields; ranges use the owning block's Q3 byte coordinates.
#[derive(Debug)]
pub struct TagField {
    /// Exact known lowercase marker including `@`.
    pub marker: Range<usize>,
    /// Opaque raw required slot, if present.
    pub slot: Option<Range<usize>>,
    /// Body segments after separator whitespace and container removal.
    pub body: Vec<Range<usize>>,
    /// Entire post-marker spelling, including continuation container prefixes.
    pub payload: Range<usize>,
}

/// Accepted opaque math, retaining delimiters and discontiguous interior spelling.
#[derive(Debug)]
pub struct MathRange {
    /// Raw outer delimiter range.
    pub range: Range<usize>,
    /// Interior segments after container removal.
    pub content: Vec<Range<usize>>,
    /// Whether the accepted candidate is a display block.
    pub display: bool,
}

/// A presentation view; the original Doc7 block always remains available.
#[derive(Debug, Default)]
pub struct MarkdownBlock {
    /// Q3 text, without secondary newline or Unicode normalization.
    pub text: String,
    /// Exact Q3-to-loaded segments; removed prefixes never acquire bridge spans.
    pub source_map: Vec<(Range<usize>, TextRange)>,
    /// Unmodified pinned CommonMark events with Q3 source offsets.
    pub events: Vec<(Event<'static>, Range<usize>)>,
    /// Genuine destination event indices and exact authored destination ranges.
    pub destinations: Vec<(usize, Range<usize>)>,
    /// Known fields in logical paragraph order.
    pub fields: Vec<TagField>,
    /// Accepted math in logical source order.
    pub math: Vec<MathRange>,
    /// A conversion failure requires retaining the original raw block.
    pub fallback: bool,
}

impl MarkdownBlock {
    /// Exact CommonMark destination, including the implicit email autolink scheme.
    pub fn interpreted_destination(&self, ordinal: usize) -> Option<Cow<'_, str>> {
        let (index, _) = self.destinations.get(ordinal)?;
        let Event::Start(
            Tag::Link {
                link_type,
                dest_url,
                ..
            }
            | Tag::Image {
                link_type,
                dest_url,
                ..
            },
        ) = &self.events.get(*index)?.0
        else {
            return None;
        };
        Some(if *link_type == LinkType::Email {
            Cow::Owned(format!("mailto:{dest_url}"))
        } else {
            Cow::Borrowed(dest_url.as_ref())
        })
    }
}

/// Parsing results preserve unavailable/empty input and the original attachments.
#[derive(Debug)]
pub struct ParsedComments<'a, 'input> {
    /// Original verified attachment products and raw projection/maps.
    pub attachments: &'a CommentAttachments<'input>,
    /// One presentation view for each original block, in the same order.
    pub blocks: Vec<MarkdownBlock>,
    /// Attachment and committed Markdown/tag diagnostic drafts.
    pub warnings: Vec<DiagnosticDraft>,
}

impl ParsedComments<'_, '_> {
    /// Distinguishes unavailable input from an available empty collection.
    pub fn available(&self) -> bool {
        self.attachments.available()
    }

    /// Applies only the independent documentation consumer's warning policy.
    pub fn documentation_failed(&self) -> bool {
        self.attachments.plan.profile.fail_on_doc_warnings && !self.warnings.is_empty()
    }
}

/// Composes authored Q3 segments through the authenticated loaded/original frame.
pub fn original_ranges(
    sources: &crate::doc_build::AuthenticatedDocSources,
    parsed: &ParsedComments<'_, '_>,
    block_ordinal: usize,
    ranges: &[Range<usize>],
) -> Result<(Vec<TextRange>, crate::doc_build::DocImportCoverage), crate::doc_build::DocBuildError>
{
    use crate::doc_build::{DocBuildError, DocImportCoverage, validate_coordinate_range};
    let refuse = || DocBuildError::Binding("Q3 source coordinate binding mismatch");
    let attachments = parsed.attachments;
    if !sources.matches_comments(attachments) {
        return Err(refuse());
    }
    if !attachments.available() {
        return Ok((Vec::new(), DocImportCoverage::Unavailable));
    }
    let canonical = crate::comments::attach_comments(attachments.plan, attachments.module, None)
        .ok_or_else(refuse)?;
    let raw = canonical.blocks.get(block_ordinal).ok_or_else(refuse)?;
    let expected = extract(raw);
    let block = parsed.blocks.get(block_ordinal).ok_or_else(refuse)?;
    if expected.fallback || block.text != expected.text {
        return Err(refuse());
    }
    let mut cursor = 0;
    for (logical, source) in &block.source_map {
        validate_coordinate_range(
            &block.text,
            TextRange {
                start: logical.start,
                end: logical.end,
            },
        )?;
        if logical.start < cursor
            || logical.start == logical.end
            || !expected.source_map.iter().any(|(span, original)| {
                span.start <= logical.start
                    && logical.end <= span.end
                    && source.start == original.start + logical.start - span.start
                    && source.end == original.start + logical.end - span.start
            })
        {
            return Err(refuse());
        }
        cursor = logical.end;
    }
    let mut loaded = Vec::new();
    let mut complete = true;
    for range in ranges {
        validate_coordinate_range(
            &block.text,
            TextRange {
                start: range.start,
                end: range.end,
            },
        )?;
        let mut covered = 0;
        for (logical, source) in &block.source_map {
            let start = logical.start.max(range.start);
            let end = logical.end.min(range.end);
            if start < end {
                loaded.push(TextRange {
                    start: source.start + start - logical.start,
                    end: source.start + end - logical.start,
                });
                covered += end - start;
            }
        }
        complete &= covered == range.end - range.start;
    }
    let module = attachments.module;
    let publication = attachments.plan.manifest.artifact_hash;
    let source_file = &attachments.input.artifact.source_file;
    let digest = sources
        .original_digest(module, publication, source_file)
        .ok_or_else(refuse)?;
    let (mapped, coverage) =
        sources.loaded_ranges(module, publication, source_file, digest, &loaded)?;
    Ok((
        mapped,
        if complete {
            coverage
        } else if block.source_map.is_empty() {
            DocImportCoverage::Unavailable
        } else {
            DocImportCoverage::Incomplete
        },
    ))
}

/// Derives presentation views without rereading source or inferring item signatures.
pub fn parse_comments<'a, 'input>(
    attachments: &'a CommentAttachments<'input>,
    metadata: &[TargetMetadata<'_>],
) -> ParsedComments<'a, 'input> {
    let mut result = ParsedComments {
        attachments,
        blocks: Vec::new(),
        warnings: attachments.warnings.clone(),
    };
    let mut seen = Vec::new();
    for (ordinal, raw) in attachments.blocks.iter().enumerate() {
        let mut block = extract(raw);
        let mut conditions = Vec::new();
        if !block.fallback {
            derive(&mut block, &mut conditions);
            let target_metadata = metadata.iter().find(|item| {
                raw.target
                    .is_some_and(|target| same_target(item.target, target))
            });
            validate_fields(&block, raw, target_metadata, &mut seen, &mut conditions);
        }
        if block.fallback || !valid_maps(&block, attachments) {
            block.fallback = true;
            result
                .warnings
                .push(conversion_draft(attachments, ordinal, raw, &conditions));
            block.fields.clear();
            block.math.clear();
            block.destinations.clear();
            conditions.clear();
        }
        conditions.sort_by_key(|condition| {
            (condition.range.start, condition.range.end, condition.number)
        });
        for condition in conditions {
            result
                .warnings
                .push(draft(attachments, ordinal, raw, &block, condition));
        }
        result.blocks.push(block);
    }
    result
}

fn extract(raw: &CommentBlock<'_>) -> MarkdownBlock {
    let mut block = MarkdownBlock::default();
    for row in &raw.lines {
        let CanonicalJson::Array(fields) = row else {
            block.fallback = true;
            break;
        };
        let (Some(CanonicalJson::Array(bounds)), Some(CanonicalJson::String(text))) =
            (fields.get(1), fields.get(2))
        else {
            block.fallback = true;
            break;
        };
        let Some(start) = bounds.first().and_then(number) else {
            block.fallback = true;
            break;
        };
        let Some(end) = bounds.get(1).and_then(number) else {
            block.fallback = true;
            break;
        };
        let Some(body) = text.strip_prefix(":::") else {
            block.fallback = true;
            break;
        };
        let body = body.strip_prefix(' ').unwrap_or(body);
        let removed = text.len() - body.len();
        let offset = block.text.len();
        block.text.push_str(body);
        if end.checked_sub(start) != Some(text.len()) {
            block.fallback = true;
            break;
        }
        if !body.is_empty() {
            block.source_map.push((
                offset..block.text.len(),
                TextRange {
                    start: start + removed,
                    end,
                },
            ));
        }
    }
    block
}

fn number(value: &CanonicalJson) -> Option<usize> {
    match value {
        CanonicalJson::Integer(value) => usize::try_from(*value).ok(),
        _ => None,
    }
}

#[derive(Debug)]
struct Paragraph {
    range: Range<usize>,
    segments: Vec<Range<usize>>,
}

#[derive(Debug)]
struct Line {
    range: Range<usize>,
    content: Range<usize>,
    containers: Vec<usize>,
    protected: bool,
}

#[derive(Debug)]
struct Condition {
    number: u16,
    range: Range<usize>,
    subreason: &'static str,
    raw: Vec<Value>,
}

fn w(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

fn inline_tag(tag: &Tag<'_>) -> bool {
    matches!(
        tag,
        Tag::Emphasis | Tag::Strong | Tag::Link { .. } | Tag::Image { .. }
    )
}

fn derive(block: &mut MarkdownBlock, conditions: &mut Vec<Condition>) {
    let mut parser = Parser::new_ext(&block.text, Options::empty()).into_offset_iter();
    block.events = parser
        .by_ref()
        .map(|(event, range)| (event.into_static(), range))
        .collect();
    let definitions: Vec<_> = block
        .events
        .iter()
        .map(|(event, _)| match event {
            Event::Start(Tag::Link { id, .. } | Tag::Image { id, .. }) => parser
                .reference_definitions()
                .get(id)
                .map(|definition| definition.span.clone()),
            _ => None,
        })
        .collect();
    drop(parser);
    let mut protected = Vec::new();
    let mut containers = Vec::new();
    let mut paragraph = None;
    let mut paragraphs = Vec::new();
    let mut disabled = 0;
    let finish = |paragraph: &mut Option<Range<usize>>, paragraphs: &mut Vec<Range<usize>>| {
        if let Some(range) = paragraph.take() {
            paragraphs.push(range);
        }
    };
    for (event, range) in &block.events {
        match event {
            Event::Start(tag) if inline_tag(tag) => {
                if disabled == 0 {
                    paragraph.get_or_insert(range.clone());
                }
            }
            Event::End(TagEnd::Emphasis | TagEnd::Strong | TagEnd::Link | TagEnd::Image) => {}
            Event::Start(Tag::Paragraph) => {
                finish(&mut paragraph, &mut paragraphs);
                paragraph = Some(range.clone());
            }
            Event::Start(tag) => {
                finish(&mut paragraph, &mut paragraphs);
                match tag {
                    Tag::CodeBlock(_) | Tag::HtmlBlock | Tag::Heading { .. } => {
                        disabled += 1;
                        if !matches!(tag, Tag::Heading { .. }) {
                            protected.push(range.clone());
                        }
                    }
                    Tag::BlockQuote(_) | Tag::Item => {
                        containers.push((range.clone(), matches!(tag, Tag::Item)))
                    }
                    _ => {}
                }
            }
            Event::End(end) => {
                finish(&mut paragraph, &mut paragraphs);
                if matches!(
                    end,
                    TagEnd::CodeBlock | TagEnd::HtmlBlock | TagEnd::Heading(_)
                ) {
                    disabled -= 1;
                }
            }
            Event::Rule => finish(&mut paragraph, &mut paragraphs),
            _ if disabled == 0 => {
                let current = paragraph.get_or_insert(range.clone());
                current.end = current.end.max(range.end);
            }
            _ => {}
        }
    }
    finish(&mut paragraph, &mut paragraphs);
    let mut offset = 0;
    let lines: Vec<_> = block
        .text
        .split_inclusive('\n')
        .map(|text| {
            let range = offset..offset + text.len();
            offset = range.end;
            let body_end = text.trim_end_matches(['\r', '\n']).len() + range.start;
            let stack: Vec<_> = containers
                .iter()
                .enumerate()
                .filter(|(_, (container, _))| {
                    container.start < range.end && range.start < container.end
                })
                .map(|(index, _)| index)
                .collect();
            let content_start =
                container_start(&block.text, range.start..body_end, &stack, &containers);
            let is_protected = protected
                .iter()
                .any(|span| span.start < range.end && range.start < span.end)
                // Reference definitions emit no events, including duplicates
                // omitted from the parser's label map. Container envelopes do
                // not make their suppressed source visible; blank lines do not
                // interrupt display math.
                || (!block.text.as_bytes()[content_start..body_end]
                    .iter()
                    .all(|&byte| w(byte))
                    && !block.events.iter().any(|(event, span)| {
                        !matches!(
                            event,
                            Event::Start(Tag::BlockQuote(_) | Tag::List(_) | Tag::Item)
                                | Event::End(_)
                        ) && span.start < range.end
                            && range.start < span.end
                    }));
            Line {
                range,
                content: content_start..body_end,
                containers: stack,
                protected: is_protected,
            }
        })
        .collect();
    display_math(block, &lines, &containers, conditions);
    let paragraphs: Vec<_> = paragraphs
        .into_iter()
        .map(|mut range| {
            // Tight-list events omit paragraph wrappers and trailing whitespace.
            // Recover only W on the final contributing line from the raw source.
            if let Some(line) = lines
                .iter()
                .find(|line| line.range.start < range.end && range.end <= line.range.end)
                && block.text.as_bytes()[range.end..line.range.end]
                    .iter()
                    .all(|&byte| w(byte))
            {
                range.end = line.range.end;
            }
            range
        })
        .flat_map(|range| {
            let mut chunks = Vec::new();
            let mut start = range.start;
            for math in block.math.iter().filter(|math| {
                math.display && math.range.start < range.end && range.start < math.range.end
            }) {
                if start < math.range.start {
                    chunks.push(start..math.range.start);
                }
                start = math.range.end;
            }
            if start < range.end {
                chunks.push(start..range.end);
            }
            chunks
        })
        .map(|range| {
            let segments = lines
                .iter()
                .filter_map(|line| {
                    let start = range.start.max(line.content.start);
                    let end = range.end.min(line.range.end);
                    (start < end).then_some(start..end)
                })
                .collect();
            Paragraph { range, segments }
        })
        .collect();
    for paragraph in &paragraphs {
        if let Some(field) = field(&block.text, paragraph) {
            block.fields.push(field);
        }
    }
    for paragraph in &paragraphs {
        inline_math(block, paragraph, conditions);
    }
    block
        .math
        .sort_by_key(|math| (math.range.start, math.range.end));
    for (index, (event, range)) in block.events.iter().enumerate() {
        let Event::Start(Tag::Link { link_type, .. } | Tag::Image { link_type, .. }) = event else {
            continue;
        };
        if block.math.iter().any(|math| {
            math.display && math.range.start < range.end && range.start < math.range.end
        }) {
            continue;
        }
        let destination = match link_type {
            LinkType::Autolink | LinkType::Email => Some(range.start + 1..range.end - 1),
            LinkType::Inline => {
                let mut depth = 1;
                let mut label_end = range.start + 1;
                for (event, inner) in &block.events[index + 1..] {
                    match event {
                        Event::Start(_) => depth += 1,
                        Event::End(_) => depth -= 1,
                        _ => {}
                    }
                    if depth == 0 {
                        break;
                    }
                    label_end = label_end.max(inner.end);
                }
                (label_end..range.end.saturating_sub(1))
                    .find(|&position| {
                        block.text.as_bytes()[position..].starts_with(b"](")
                            && !escaped(&block.text, position)
                    })
                    .map(|position| destination_range(&block.text, position + 2, range.end, &lines))
            }
            LinkType::Reference | LinkType::Collapsed | LinkType::Shortcut => {
                definitions[index].as_ref().and_then(|definition| {
                    (definition.start..definition.end.saturating_sub(1))
                        .find(|&position| {
                            block.text.as_bytes()[position..].starts_with(b"]:")
                                && !escaped(&block.text, position)
                        })
                        .map(|position| {
                            destination_range(&block.text, position + 2, definition.end, &lines)
                        })
                })
            }
            _ => None,
        };
        if let Some(destination) = destination {
            block.destinations.push((index, destination));
        } else {
            block.fallback = true;
        }
    }
}

fn destination_range(text: &str, mut start: usize, end: usize, lines: &[Line]) -> Range<usize> {
    let bytes = text.as_bytes();
    while start < end && matches!(bytes[start], b' ' | b'\t'..=b'\r') {
        start += 1;
        if let Some(line) = lines.iter().find(|line| line.range.start == start) {
            start = line.content.start;
        }
    }
    let angle = bytes.get(start) == Some(&b'<');
    start += usize::from(angle);
    let mut cursor = start;
    let mut depth = 0;
    while cursor < end {
        match bytes[cursor] {
            b'\\' if cursor + 1 < end && bytes[cursor + 1].is_ascii_punctuation() => cursor += 2,
            b'>' if angle => break,
            b'(' if !angle => {
                depth += 1;
                cursor += 1;
            }
            b')' if !angle && depth == 0 => break,
            b')' if !angle => {
                depth -= 1;
                cursor += 1;
            }
            byte if !angle && byte <= 0x20 => break,
            _ => cursor += 1,
        }
    }
    start..cursor
}

fn trim_w(text: &str, mut range: Range<usize>) -> Range<usize> {
    while range.start < range.end && w(text.as_bytes()[range.start]) {
        range.start += 1;
    }
    while range.start < range.end && w(text.as_bytes()[range.end - 1]) {
        range.end -= 1;
    }
    range
}

fn escaped(text: &str, position: usize) -> bool {
    text.as_bytes()[..position]
        .iter()
        .rev()
        .take_while(|&&byte| byte == b'\\')
        .count()
        % 2
        == 1
}

fn container_start(
    text: &str,
    line: Range<usize>,
    stack: &[usize],
    containers: &[(Range<usize>, bool)],
) -> usize {
    let bytes = text.as_bytes();
    let mut cursor = line.start;
    for &id in stack {
        let (container, item) = &containers[id];
        let before = cursor;
        if *item && container.start < line.start {
            let marker = container.start;
            let mut end = marker;
            while end < text.len()
                && (text.as_bytes()[end].is_ascii_digit()
                    || matches!(text.as_bytes()[end], b'-' | b'+' | b'*' | b'.' | b')'))
            {
                end += 1;
            }
            let mut width = end - marker;
            while end < text.len() && matches!(text.as_bytes()[end], b' ' | b'\t') {
                width += if text.as_bytes()[end] == b'\t' {
                    4 - width % 4
                } else {
                    1
                };
                end += 1;
            }
            let mut consumed = 0;
            while cursor < line.end && consumed < width && matches!(bytes[cursor], b' ' | b'\t') {
                consumed += if bytes[cursor] == b'\t' {
                    4 - consumed % 4
                } else {
                    1
                };
                cursor += 1;
            }
            continue;
        }
        for _ in 0..3 {
            if cursor < line.end && bytes[cursor] == b' ' {
                cursor += 1;
            } else {
                break;
            }
        }
        if *item {
            let marker = cursor;
            if cursor < line.end && matches!(bytes[cursor], b'-' | b'+' | b'*') {
                cursor += 1;
            } else {
                while cursor < line.end && bytes[cursor].is_ascii_digit() {
                    cursor += 1;
                }
                if cursor < line.end && matches!(bytes[cursor], b'.' | b')') {
                    cursor += 1;
                } else {
                    cursor = marker;
                }
            }
            if cursor == marker {
                cursor = before;
                continue;
            }
            while cursor < line.end && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
        } else if cursor < line.end && bytes[cursor] == b'>' {
            cursor += 1;
            if cursor < line.end && matches!(bytes[cursor], b' ' | b'\t') {
                cursor += 1;
            }
        } else {
            cursor = before;
        }
    }
    cursor
}

fn display_math(
    block: &mut MarkdownBlock,
    lines: &[Line],
    containers: &[(Range<usize>, bool)],
    conditions: &mut Vec<Condition>,
) {
    let mut index = 0;
    while index < lines.len() {
        let line = &lines[index];
        let range = trim_w(&block.text, line.content.clone());
        if line.protected
            || range.is_empty()
            || !block.text[range.clone()].starts_with("$$")
            || block.text[range.clone()].starts_with("$$$")
        {
            index += 1;
            continue;
        }
        // Four spaces remaining after structural prefix stripping are indented code.
        if line.content.start + 3 < range.start {
            index += 1;
            continue;
        }
        let text = &block.text[range.clone()];
        if text == "$$" {
            let close = (index + 1..lines.len())
                .take_while(|&next| {
                    !lines[next].protected && lines[next].containers.starts_with(&line.containers)
                })
                .find(|&next| {
                    let delimiter = trim_w(&block.text, lines[next].content.clone());
                    lines[next].containers == line.containers
                        && delimiter.start <= lines[next].content.start + 3
                        && block.text[delimiter] == *"$$"
                });
            if let Some(close) = close {
                let mut content = Vec::new();
                for inner in &lines[index + 1..close] {
                    let start = container_start(
                        &block.text,
                        inner.range.clone(),
                        &line.containers,
                        containers,
                    );
                    if start < inner.range.end {
                        content.push(start..inner.range.end);
                    }
                }
                let outer = range.start..trim_w(&block.text, lines[close].content.clone()).end;
                if content.iter().any(|segment| {
                    block.text.as_bytes()[segment.clone()]
                        .iter()
                        .any(|&byte| !w(byte))
                }) {
                    block.math.push(MathRange {
                        range: outer,
                        content,
                        display: true,
                    });
                } else {
                    conditions.push(Condition {
                        number: 410,
                        range: range.start..range.start + 2,
                        subreason: "display",
                        raw: Vec::new(),
                    });
                }
                index = close + 1;
                continue;
            }
            conditions.push(Condition {
                number: 409,
                range: range.start..range.start + 2,
                subreason: "display",
                raw: Vec::new(),
            });
        } else if text.len() >= 4
            && text.ends_with("$$")
            && !text.ends_with("$$$")
            && !escaped(&block.text, range.end - 2)
        {
            let inner = range.start + 2..range.end - 2;
            let bad = (inner.start..inner.end.saturating_sub(1)).any(|position| {
                block.text.as_bytes()[position..].starts_with(b"$$")
                    && !escaped(&block.text, position)
            });
            if !bad {
                if block.text.as_bytes()[inner.clone()]
                    .iter()
                    .any(|&byte| !w(byte))
                {
                    block.math.push(MathRange {
                        range,
                        content: vec![inner],
                        display: true,
                    });
                } else {
                    conditions.push(Condition {
                        number: 410,
                        range: range.start..range.start + 2,
                        subreason: "display",
                        raw: Vec::new(),
                    });
                }
            }
        }
        index += 1;
    }
}

fn field(text: &str, paragraph: &Paragraph) -> Option<TagField> {
    let positions: Vec<_> = paragraph
        .segments
        .iter()
        .flat_map(|range| range.clone())
        .collect();
    let first = positions
        .iter()
        .position(|&position| !w(text.as_bytes()[position]))?;
    let mut end = first;
    while end < positions.len() && !w(text.as_bytes()[positions[end]]) {
        end += 1;
    }
    let marker = positions[first]..positions[end - 1] + 1;
    let name = &text[marker.clone()];
    if !matches!(
        name,
        "@param" | "@returns" | "@requires" | "@ensures" | "@see" | "@since" | "@deprecated"
    ) {
        return None;
    }
    let mut cursor = end;
    while cursor < positions.len() && w(text.as_bytes()[positions[cursor]]) {
        cursor += 1;
    }
    let slot = if matches!(name, "@param" | "@see" | "@since" | "@deprecated")
        && cursor < positions.len()
    {
        let start = cursor;
        while cursor < positions.len() && !w(text.as_bytes()[positions[cursor]]) {
            cursor += 1;
        }
        Some(positions[start]..positions[cursor - 1] + 1)
    } else {
        None
    };
    while cursor < positions.len() && w(text.as_bytes()[positions[cursor]]) {
        cursor += 1;
    }
    let body_start = positions
        .get(cursor)
        .copied()
        .unwrap_or(paragraph.range.end);
    let mut payload_end = paragraph.range.end;
    if text[..payload_end].ends_with('\n') {
        payload_end -= 1;
        if text[..payload_end].ends_with('\r') {
            payload_end -= 1;
        }
    }
    let body = paragraph
        .segments
        .iter()
        .filter_map(|range| {
            let start = range.start.max(body_start);
            let end = range.end.min(payload_end);
            (start < end).then_some(start..end)
        })
        .collect();
    Some(TagField {
        payload: marker.end..payload_end,
        marker,
        slot,
        body,
    })
}

fn inline_math(block: &mut MarkdownBlock, paragraph: &Paragraph, conditions: &mut Vec<Condition>) {
    let mut barriers = Vec::new();
    let mut labels = Vec::new();
    let mut links: Vec<(Range<usize>, Option<Range<usize>>, bool)> = Vec::new();
    for (event, range) in &block.events {
        if range.end <= paragraph.range.start || range.start >= paragraph.range.end {
            continue;
        }
        match event {
            Event::Start(Tag::Link { link_type, .. } | Tag::Image { link_type, .. }) => {
                barriers.push(range.clone());
                links.push((
                    range.clone(),
                    None,
                    matches!(
                        link_type,
                        pulldown_cmark::LinkType::Autolink | pulldown_cmark::LinkType::Email
                    ),
                ));
            }
            Event::End(TagEnd::Link | TagEnd::Image) => {
                if let Some((outer, Some(label), false)) = links.pop() {
                    labels.push((outer, label));
                }
            }
            Event::Code(_) | Event::InlineHtml(_) => {
                barriers.push(range.clone());
                if let Some((_, label, _)) = links.last_mut() {
                    if let Some(label) = label {
                        label.end = label.end.max(range.end);
                    } else {
                        *label = Some(range.clone());
                    }
                }
            }
            Event::End(_) => {}
            Event::Start(tag) if !inline_tag(tag) => {}
            _ => {
                if let Some((_, label, _)) = links.last_mut() {
                    if let Some(label) = label {
                        label.end = label.end.max(range.end);
                    } else {
                        *label = Some(range.clone());
                    }
                }
            }
        }
    }
    barriers.extend(block.fields.iter().filter_map(|field| field.slot.clone()));
    let mut contexts = vec![(paragraph.range.clone(), None)];
    contexts.extend(
        labels
            .into_iter()
            .map(|(outer, label)| (label, Some(outer))),
    );
    for (context, own_link) in contexts {
        let mut cursor = context.start;
        let mut pieces = Vec::new();
        let mut sorted: Vec<_> = barriers
            .iter()
            .filter(|barrier| {
                Some(*barrier) != own_link.as_ref()
                    && context.start < barrier.end
                    && barrier.start < context.end
            })
            .cloned()
            .collect();
        sorted.sort_by_key(|range| (range.start, range.end));
        for barrier in sorted {
            if cursor < barrier.start {
                pieces.push(cursor..barrier.start.min(context.end));
            }
            cursor = cursor.max(barrier.end);
        }
        if cursor < context.end {
            pieces.push(cursor..context.end);
        }
        for piece in pieces {
            let segments: Vec<_> = paragraph
                .segments
                .iter()
                .filter_map(|range| {
                    let start = range.start.max(piece.start);
                    let end = range.end.min(piece.end);
                    (start < end).then_some(start..end)
                })
                .collect();
            let mut opener = None;
            for position in segments.iter().flat_map(|segment| segment.clone()) {
                let bytes = block.text.as_bytes();
                if bytes[position] != b'$'
                    || escaped(&block.text, position)
                    || position > 0 && bytes[position - 1] == b'$'
                    || bytes.get(position + 1) == Some(&b'$')
                {
                    continue;
                }
                if let Some(start) = opener.take() {
                    if start + 1 < position {
                        let content = segments
                            .iter()
                            .filter_map(|range| {
                                let left = range.start.max(start + 1);
                                let right = range.end.min(position);
                                (left < right).then_some(left..right)
                            })
                            .collect();
                        block.math.push(MathRange {
                            range: start..position + 1,
                            content,
                            display: false,
                        });
                    } else {
                        opener = Some(position);
                    }
                } else {
                    opener = Some(position);
                }
            }
            if let Some(start) = opener {
                conditions.push(Condition {
                    number: 409,
                    range: start..start + 1,
                    subreason: "inline",
                    raw: Vec::new(),
                });
            }
        }
    }
}

fn same_target(left: CommentTarget<'_>, right: CommentTarget<'_>) -> bool {
    match (left, right) {
        (CommentTarget::Module, CommentTarget::Module) => true,
        (CommentTarget::Item(left), CommentTarget::Item(right)) => std::ptr::eq(left, right),
        _ => false,
    }
}

fn validate_fields<'a>(
    block: &MarkdownBlock,
    raw: &CommentBlock<'a>,
    metadata: Option<&TargetMetadata<'_>>,
    seen: &mut Vec<(CommentTarget<'a>, Vec<Value>)>,
    conditions: &mut Vec<Condition>,
) {
    for field in &block.fields {
        let marker = &block.text[field.marker.clone()];
        let slot = field.slot.as_ref().map(|range| &block.text[range.clone()]);
        let payload = &block.text[field.payload.clone()];
        let key = match marker {
            "@param" => slot
                .map(|slot| vec![value(marker), value(slot)])
                .unwrap_or_default(),
            "@returns" | "@since" | "@deprecated" => vec![value(marker)],
            _ => vec![value(marker), value(payload)],
        };
        let raw_details = vec![value(marker), value(payload), Value::List(key.clone())];
        let mut warn = |number, range, subreason| {
            conditions.push(Condition {
                number,
                range,
                subreason,
                raw: raw_details.clone(),
            })
        };
        if matches!(marker, "@param" | "@see" | "@since" | "@deprecated") && slot.is_none() {
            warn(
                405,
                field.marker.clone(),
                match marker {
                    "@param" => "name",
                    "@see" => "reference",
                    _ => "version",
                },
            );
        }
        if matches!(
            marker,
            "@returns" | "@requires" | "@ensures" | "@deprecated"
        ) && !field.body.iter().any(|range| {
            block.text.as_bytes()[range.clone()]
                .iter()
                .any(|&byte| !w(byte))
        }) {
            warn(
                406,
                field.marker.clone(),
                match marker {
                    "@returns" => "returns",
                    "@requires" => "requires",
                    "@ensures" => "ensures",
                    _ => "deprecated",
                },
            );
        }
        if let Some(target) = raw.target
            && !key.is_empty()
        {
            if seen.iter().any(|(previous, previous_key)| {
                same_target(*previous, target) && *previous_key == key
            }) {
                warn(
                    407,
                    field.marker.clone(),
                    match marker {
                        "@param" => "param",
                        "@returns" | "@since" | "@deprecated" => "singleton",
                        _ => "repeatable",
                    },
                );
            }
            seen.push((target, key));
        }
        if let Some(metadata) = metadata {
            if marker == "@param"
                && metadata.parameters_complete
                && let (Some(slot), Some(names)) = (slot, metadata.parameters)
                && !names.contains(&slot)
            {
                warn(
                    408,
                    field.slot.clone().expect("present slot"),
                    "parameter_absent",
                );
            }
            if marker == "@returns" && metadata.has_return == Some(false) {
                warn(408, field.marker.clone(), "no_return");
            }
        }
    }
}

fn value(text: impl ToString) -> Value {
    Value::String(text.to_string())
}
fn optional_value(text: Option<&str>) -> Value {
    Value::List(text.into_iter().map(value).collect())
}

pub(crate) fn occurrence(
    attachments: &CommentAttachments<'_>,
    ordinal: usize,
    raw: &CommentBlock<'_>,
    number: u16,
    range: Option<&Range<usize>>,
) -> Value {
    let module = attachments.module;
    let target = match raw.target {
        Some(CommentTarget::Module) => vec![value("module")],
        Some(CommentTarget::Item(item)) => vec![
            value("item"),
            value(&item.origin_id),
            value(&item.fully_qualified_name),
        ],
        None => Vec::new(),
    };
    let descriptor = DiagnosticRegistry::builtin()
        .lookup(
            DiagnosticCode::from_parts(DiagnosticSeverity::Warning, number)
                .expect("allocated Documentation code"),
        )
        .expect("Documentation descriptor");
    Value::List(vec![
        value(attachments.plan.manifest.artifact_hash.to_lower_hex()),
        Value::List(vec![
            value(&module.package_id),
            optional_value(module.package_version.as_deref()),
            optional_value(module.lockfile_identity.as_deref()),
            value(&module.module_path),
            value(&module.language_edition),
        ]),
        Value::List(target),
        value(ordinal),
        value(descriptor.meaning_key),
        Value::List(
            range
                .map(|range| vec![value(range.start), value(range.end)])
                .unwrap_or_default(),
        ),
    ])
}

fn mapped(block: &MarkdownBlock, range: &Range<usize>, raw: &CommentBlock<'_>) -> Vec<SourceRange> {
    let Some(source) = raw.source_range else {
        return Vec::new();
    };
    block
        .source_map
        .iter()
        .filter_map(|(logical, loaded)| {
            let start = logical.start.max(range.start);
            let end = logical.end.min(range.end);
            (start < end).then_some(SourceRange {
                source_id: source.source_id,
                start: loaded.start + start - logical.start,
                end: loaded.start + end - logical.start,
            })
        })
        .collect()
}

fn draft(
    attachments: &CommentAttachments<'_>,
    ordinal: usize,
    raw: &CommentBlock<'_>,
    block: &MarkdownBlock,
    condition: Condition,
) -> DiagnosticDraft {
    let code = DiagnosticCode::from_parts(DiagnosticSeverity::Warning, condition.number)
        .expect("allocated Documentation code");
    let descriptor = DiagnosticRegistry::builtin()
        .lookup(code)
        .expect("Documentation descriptor");
    let mut ranges = mapped(block, &condition.range, raw).into_iter();
    let primary_location = ranges.next().map_or_else(
        || provenance(attachments, ordinal),
        |range| {
            DiagnosticPrimaryLocation::Span(
                DiagnosticSpan::primary(range, None).expect("nonempty mapped range"),
            )
        },
    );
    let mut details = DiagnosticDetails::new();
    for (key, value) in [
        (
            "documentation.occurrence",
            occurrence(
                attachments,
                ordinal,
                raw,
                condition.number,
                Some(&condition.range),
            ),
        ),
        ("documentation.raw", Value::List(condition.raw)),
        ("documentation.subreason", value(condition.subreason)),
    ] {
        details.insert(key, value).expect("fixed detail keys");
    }
    DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: attachments.plan.snapshot,
        code,
        phase: PipelinePhase::Documentation,
        category: FailureCategory::DocumentationWarning,
        stable_detail_key: descriptor.meaning_key.to_owned(),
        message: descriptor.summary.to_owned(),
        primary_location,
        secondary_spans: ranges
            .map(|range| DiagnosticSpan::secondary(range, None).expect("nonempty mapped range"))
            .collect(),
        notes: Vec::new(),
        details,
        fixes: Vec::new(),
        explanation: None,
    })
    .expect("complete verified Documentation provenance")
}

fn provenance(attachments: &CommentAttachments<'_>, ordinal: usize) -> DiagnosticPrimaryLocation {
    DiagnosticPrimaryLocation::Documentation {
        publication_hash: attachments.plan.manifest.artifact_hash,
        package_id: attachments.module.package_id.clone(),
        package_version: attachments.module.package_version.clone(),
        lockfile_identity: attachments.module.lockfile_identity.clone(),
        module_path: attachments.module.module_path.clone(),
        language_edition: attachments.module.language_edition.clone(),
        block_ordinal: ordinal as u64,
    }
}

fn valid_maps(block: &MarkdownBlock, attachments: &CommentAttachments<'_>) -> bool {
    let CanonicalJson::Object(projection) = &attachments.input.comment_projection else {
        return block.text.is_empty();
    };
    let Some(CanonicalJson::String(loaded)) = projection.get("loaded") else {
        return false;
    };
    let mut cursor = 0;
    for (logical, source) in &block.source_map {
        if logical.start != cursor
            || logical.end <= logical.start
            || source.end.checked_sub(source.start) != Some(logical.end - logical.start)
            || block.text.get(logical.clone()) != loaded.get(source.start..source.end)
        {
            return false;
        }
        cursor = logical.end;
    }
    cursor == block.text.len()
}

fn conversion_draft(
    attachments: &CommentAttachments<'_>,
    ordinal: usize,
    raw: &CommentBlock<'_>,
    conditions: &[Condition],
) -> DiagnosticDraft {
    let code = DiagnosticCode::from_parts(DiagnosticSeverity::Warning, 411)
        .expect("allocated Documentation code");
    let descriptor = DiagnosticRegistry::builtin()
        .lookup(code)
        .expect("Documentation descriptor");
    let mut retained: Vec<_> = conditions
        .iter()
        .map(|condition| {
            let reason = DiagnosticRegistry::builtin()
                .lookup(
                    DiagnosticCode::from_parts(DiagnosticSeverity::Warning, condition.number)
                        .expect("allocated Documentation code"),
                )
                .expect("Documentation descriptor")
                .meaning_key;
            (
                condition.range.clone(),
                reason,
                condition.subreason,
                Value::List(vec![
                    occurrence(
                        attachments,
                        ordinal,
                        raw,
                        condition.number,
                        Some(&condition.range),
                    ),
                    Value::List(condition.raw.clone()),
                    value(condition.subreason),
                ]),
            )
        })
        .collect();
    retained.sort_by(|left, right| {
        (left.0.start, left.0.end, left.1, left.2, &left.3).cmp(&(
            right.0.start,
            right.0.end,
            right.1,
            right.2,
            &right.3,
        ))
    });
    let details = DiagnosticDetails::from_entries([
        (
            "documentation.occurrence",
            occurrence(attachments, ordinal, raw, 411, None),
        ),
        ("documentation.raw", Value::List(Vec::new())),
        ("documentation.subreason", value("mapping_failed")),
        ("documentation.conversion_stage", value("source_map")),
        (
            "documentation.retained_conditions",
            Value::List(retained.into_iter().map(|row| row.3).collect()),
        ),
    ])
    .expect("fixed detail keys");
    DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: attachments.plan.snapshot,
        code,
        phase: PipelinePhase::Documentation,
        category: FailureCategory::DocumentationWarning,
        stable_detail_key: descriptor.meaning_key.to_owned(),
        message: descriptor.summary.to_owned(),
        primary_location: raw_primary(attachments, raw).map_or_else(
            || provenance(attachments, ordinal),
            |range| {
                DiagnosticPrimaryLocation::Span(
                    DiagnosticSpan::primary(range, None).expect("retained raw block range"),
                )
            },
        ),
        secondary_spans: Vec::new(),
        notes: Vec::new(),
        details,
        fixes: Vec::new(),
        explanation: None,
    })
    .expect("complete verified Documentation provenance")
}

fn raw_primary(
    attachments: &CommentAttachments<'_>,
    raw: &CommentBlock<'_>,
) -> Option<SourceRange> {
    let source = raw.source_range?;
    let CanonicalJson::Object(projection) = &attachments.input.comment_projection else {
        return None;
    };
    let Some(CanonicalJson::String(loaded)) = projection.get("loaded") else {
        return None;
    };
    (source.start == raw.range.start
        && source.end == raw.range.end
        && source.start < source.end
        && loaded.get(source.start..source.end).is_some())
    .then_some(source)
}
