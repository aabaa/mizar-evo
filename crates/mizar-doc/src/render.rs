//! Isolated documentation fragments and local asset readiness.

use std::fmt::Write;

use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetailValue as Value, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticPrimaryLocation, DiagnosticSpan, FailureCategory, PipelinePhase,
    },
    registry::{DiagnosticCode, DiagnosticRegistry, DiagnosticSeverity},
};
use mizar_session::SourceRange;
use pulldown_cmark::{Event, Tag};

use crate::{
    allocations::ValidatedDocAllocations,
    doc_build::{AuthenticatedDocDirectBy, AuthenticatedDocImports, DocBuildError},
    links::{
        ClassifiedDocOccurrence, DocReferenceClassification, DocReferenceForm, ResolvedDocLinks,
        resolve_doc_links,
    },
    markdown::{ParsedComments, occurrence, parse_comments},
};

/// Emits genuine External destinations and escaped raw HTML after resolver binding.
pub fn url_fragments(
    parsed: &ParsedComments<'_, '_>,
    inputs: &AuthenticatedDocImports,
    direct_by: &AuthenticatedDocDirectBy,
    allocations: &ValidatedDocAllocations,
    specification_version: &str,
    classifications: &[Vec<DocReferenceClassification>],
) -> Result<(Vec<String>, ResolvedDocLinks), DocBuildError> {
    let refuse = || DocBuildError::Binding("documentation URL destination binding mismatch");
    if !parsed.available() || classifications.len() != parsed.blocks.len() {
        return Err(refuse());
    }
    let original = parse_comments(parsed.attachments, &[]);
    if parsed.blocks.len() != original.blocks.len()
        || parsed
            .blocks
            .iter()
            .zip(&original.blocks)
            .any(|(block, original)| {
                block.fallback != original.fallback
                    || block.text != original.text
                    || block.events != original.events
                    || block.destinations != original.destinations
                    || block
                        .source_map
                        .windows(2)
                        .any(|pair| pair[0].0.end > pair[1].0.start)
                    || block.source_map.iter().any(|(logical, loaded)| {
                        logical.start >= logical.end
                            || !original.source_map.iter().any(|(expected, source)| {
                                expected.start <= logical.start
                                    && logical.end <= expected.end
                                    && loaded.start == source.start + logical.start - expected.start
                                    && loaded.end == source.start + logical.end - expected.start
                            })
                    })
            })
    {
        return Err(refuse());
    }
    let mut occurrences = Vec::new();
    for (block_ordinal, (block, classes)) in parsed.blocks.iter().zip(classifications).enumerate() {
        if classes.len() != block.destinations.len() {
            return Err(refuse());
        }
        for (ordinal, ((_, range), classification)) in
            block.destinations.iter().zip(classes).enumerate()
        {
            occurrences.push(ClassifiedDocOccurrence {
                block: block_ordinal,
                range: range.clone(),
                destination_ranges: vec![range.clone()],
                form: DocReferenceForm::Explicit,
                interpreted: Some(
                    block
                        .interpreted_destination(ordinal)
                        .ok_or_else(refuse)?
                        .into_owned(),
                ),
                classification: *classification,
            });
        }
    }
    let mut resolved = resolve_doc_links(
        parsed,
        inputs,
        direct_by,
        allocations,
        specification_version,
        &occurrences,
    )?;
    let mut fragments = Vec::new();
    for (block_ordinal, block) in parsed.blocks.iter().enumerate() {
        if block.fallback {
            continue;
        }
        let mut destinations = block.destinations.iter().enumerate().peekable();
        let mut label_end = 0;
        for (index, (event, range)) in block.events.iter().enumerate() {
            if destinations
                .peek()
                .is_some_and(|(_, (event_index, _))| *event_index == index)
            {
                let (ordinal, (_, destination)) =
                    destinations.next().expect("matching destination");
                if classifications[block_ordinal][ordinal] != DocReferenceClassification::External {
                    continue;
                }
                let Event::Start(tag @ (Tag::Link { .. } | Tag::Image { .. })) = event else {
                    return Err(refuse());
                };
                let image = matches!(tag, Tag::Image { .. });
                let destination_value =
                    block.interpreted_destination(ordinal).ok_or_else(refuse)?;
                let dest_url = destination_value.as_ref();
                let mut label = String::new();
                let mut depth = 1;
                for (offset, (event, _)) in block.events[index + 1..].iter().enumerate() {
                    match event {
                        Event::Start(_) => depth += 1,
                        Event::End(_) => depth -= 1,
                        Event::Text(text) | Event::Code(text) | Event::InlineHtml(text) => {
                            label.push_str(text)
                        }
                        Event::SoftBreak | Event::HardBreak => label.push(' '),
                        _ => {}
                    }
                    if depth == 0 {
                        label_end = label_end.max(index + offset + 2);
                        break;
                    }
                }
                let label = escape(&label, true);
                if accepted_url(dest_url) {
                    let url = escape(dest_url, false);
                    fragments.push(if image {
                        format!("<img src=\"{url}\" alt=\"{label}\">")
                    } else {
                        format!("<a href=\"{url}\">{label}</a>")
                    });
                } else {
                    fragments.push(label);
                    resolved.warnings.push(url_draft(
                        parsed,
                        block_ordinal,
                        ordinal,
                        image,
                        destination,
                        dest_url,
                    ));
                }
            } else if let Event::Html(text) | Event::InlineHtml(text) = event
                && index >= label_end
                && !block.math.iter().any(|math| {
                    math.display && math.range.start < range.end && range.start < math.range.end
                })
            {
                fragments.push(escape(text, true));
            }
        }
    }
    Ok((fragments, resolved))
}

fn accepted_url(url: &str) -> bool {
    if url
        .chars()
        .any(|ch| ch <= '\u{1f}' || ch == '\u{7f}' || ch == '\\')
    {
        return false;
    }
    let checked = url.trim_matches(' ');
    let Some(colon) = checked.find(':') else {
        return true;
    };
    if checked
        .find(['/', '?', '#'])
        .is_some_and(|delimiter| delimiter < colon)
    {
        return true;
    }
    let scheme = &checked[..colon];
    scheme
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphabetic)
        && scheme
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'-'))
        && ["http", "https", "mailto"]
            .iter()
            .any(|allowed| scheme.eq_ignore_ascii_case(allowed))
}

fn url_draft(
    parsed: &ParsedComments<'_, '_>,
    block_ordinal: usize,
    ordinal: usize,
    image: bool,
    range: &std::ops::Range<usize>,
    interpreted: &str,
) -> DiagnosticDraft {
    let attachments = parsed.attachments;
    let raw = &attachments.blocks[block_ordinal];
    let block = &parsed.blocks[block_ordinal];
    let value = |text: &str| Value::String(text.to_owned());
    let mut details = DiagnosticDetails::new();
    details
        .insert(
            "documentation.occurrence",
            occurrence(attachments, block_ordinal, raw, 415, Some(range)),
        )
        .expect("adopted documentation occurrence key");
    details
        .insert(
            "documentation.render.url",
            Value::List(vec![
                value(if image { "image" } else { "link" }),
                value(&ordinal.to_string()),
                value(&block.text[range.clone()]),
                value(interpreted),
            ]),
        )
        .expect("adopted URL detail key");
    let mut spans = Vec::new();
    if let Some(source) = raw.source_range {
        for (logical, loaded) in &block.source_map {
            let start = logical.start.max(range.start);
            let end = logical.end.min(range.end);
            if start < end {
                let span = SourceRange {
                    source_id: source.source_id,
                    start: loaded.start + start - logical.start,
                    end: loaded.start + end - logical.start,
                };
                spans.push(span);
            }
        }
    }
    let module = attachments.module;
    let primary_location = if !spans.is_empty() {
        let span = spans.remove(0);
        DiagnosticPrimaryLocation::Span(
            DiagnosticSpan::primary(span, None).expect("nonempty mapped destination"),
        )
    } else {
        DiagnosticPrimaryLocation::Documentation {
            publication_hash: attachments.plan.manifest.artifact_hash,
            package_id: module.package_id.clone(),
            package_version: module.package_version.clone(),
            lockfile_identity: module.lockfile_identity.clone(),
            module_path: module.module_path.clone(),
            language_edition: module.language_edition.clone(),
            block_ordinal: block_ordinal as u64,
        }
    };
    let code =
        DiagnosticCode::from_parts(DiagnosticSeverity::Warning, 415).expect("allocated URL code");
    let descriptor = DiagnosticRegistry::builtin()
        .lookup(code)
        .expect("adopted URL descriptor");
    DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: attachments.plan.snapshot,
        code,
        phase: PipelinePhase::Documentation,
        category: FailureCategory::DocumentationWarning,
        stable_detail_key: descriptor.meaning_key.to_owned(),
        message: descriptor.summary.to_owned(),
        primary_location,
        secondary_spans: spans
            .into_iter()
            .map(|span| DiagnosticSpan::secondary(span, None).expect("nonempty mapped destination"))
            .collect(),
        notes: Vec::new(),
        details,
        fixes: Vec::new(),
        explanation: None,
    })
    .expect("bound URL diagnostic draft")
}

fn escape(text: &str, visible_controls: bool) -> String {
    let mut html = String::new();
    for ch in text.chars() {
        match ch {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            '"' => html.push_str("&quot;"),
            '\'' => html.push_str("&#39;"),
            ch if visible_controls && ch.is_control() && ch != '\n' && ch != '\t' => {
                write!(html, "\\u{{{:x}}}", u32::from(ch)).expect("writing to String cannot fail");
            }
            ch => html.push(ch),
        }
    }
    html
}

/// Emits source-ordered fragments; provenance remains in `parsed` at each ordinal.
pub fn opaque_math_fragments(
    parsed: &ParsedComments<'_, '_>,
) -> Result<Vec<String>, DocBuildError> {
    let refuse = || DocBuildError::Binding("opaque math source binding mismatch");
    if !parsed.available() || parsed.blocks.len() != parsed.attachments.blocks.len() {
        return Err(refuse());
    }
    let mut fragments = Vec::new();
    for (block_ordinal, block) in parsed.blocks.iter().enumerate() {
        let mut cursor = 0;
        for (occurrence, math) in block.math.iter().enumerate() {
            if math.range.start < cursor
                || math.range.start >= math.range.end
                || block.text.get(math.range.clone()).is_none()
                || block.fallback
            {
                return Err(refuse());
            }
            cursor = math.range.end;
            let mut payload = String::new();
            let mut end = math.range.start;
            for segment in &math.content {
                if segment.start < end || segment.end > math.range.end {
                    return Err(refuse());
                }
                payload.push_str(block.text.get(segment.clone()).ok_or_else(refuse)?);
                end = segment.end;
            }
            let tag = if math.display { "div" } else { "span" };
            let mut html = format!(
                "<{tag} class=\"mizar-math\" data-mizar-block=\"{block_ordinal}\" data-mizar-occurrence=\"{occurrence}\" data-mizar-display=\"{}\" data-mizar-tex=\"",
                math.display
            );
            for byte in payload.as_bytes() {
                write!(html, "{byte:02x}").expect("writing to String cannot fail");
            }
            html.push_str("\"><span role=\"status\">Mathematics pending</span><code>");
            html.push_str(&escape(&payload, true));
            writeln!(html, "</code></{tag}>").expect("writing to String cannot fail");
            fragments.push(html);
        }
    }
    Ok(fragments)
}

/// Refuses incomplete, duplicated, extra or changed local asset/configuration bytes.
pub fn validate_math_assets(assets: &[(&str, &[u8])]) -> Result<(), DocBuildError> {
    let expected = crate::math::bundled_math_assets();
    if assets.len() != expected.len()
        || expected.iter().any(|(path, bytes)| {
            let mut found = assets.iter().filter(|(candidate, _)| candidate == path);
            !found.next().is_some_and(|(_, supplied)| supplied == bytes) || found.next().is_some()
        })
    {
        return Err(DocBuildError::Binding("opaque math local assets mismatch"));
    }
    Ok(())
}

/// Fixed local search/style assets and their immutable integration manifest.
pub fn bundled_search_assets() -> &'static [(&'static str, &'static [u8])] {
    &[
        ("assets/style.css", include_bytes!("../assets/style.css")),
        ("assets/search.js", include_bytes!("../assets/search.js")),
        (
            "assets/search-manifest.json",
            include_bytes!("../assets/search-manifest.json"),
        ),
    ]
}

/// Refuses missing, extra, duplicated or changed search/style assets and manifest.
pub fn validate_search_assets(assets: &[(&str, &[u8])]) -> Result<(), DocBuildError> {
    let expected = bundled_search_assets();
    if assets.len() != expected.len()
        || expected.iter().any(|(path, bytes)| {
            let mut found = assets.iter().filter(|(candidate, _)| candidate == path);
            !found.next().is_some_and(|(_, supplied)| supplied == bytes) || found.next().is_some()
        })
    {
        return Err(DocBuildError::Binding("search/style local assets mismatch"));
    }
    Ok(())
}
