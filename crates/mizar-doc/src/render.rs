//! Isolated documentation fragments and local asset readiness.

use std::{collections::BTreeMap, fmt::Write, path::Path};

use mizar_artifact::{module_summary::ModuleSummaryIdentity, verified_artifact::VerifiedExport};

use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetailValue as Value, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticPrimaryLocation, DiagnosticSpan, FailureCategory, PipelinePhase,
    },
    registry::{DiagnosticCode, DiagnosticRegistry, DiagnosticSeverity},
};
use mizar_session::{Hash, SourceRange, TextRange};
use pulldown_cmark::{Event, Tag};

use crate::{
    allocations::{
        DocAllocationTable, DocDestinationAllocations, DocPageAllocation, ValidatedDocAllocations,
        allocate_doc_destinations, encode_doc_href,
    },
    comments::{CommentTarget, attach_comments},
    doc_build::{
        AuthenticatedDocDirectBy, AuthenticatedDocIdentities, AuthenticatedDocImports,
        AuthenticatedDocSources, DocBuildError, DocImportCoverage, validate_coordinate_range,
    },
    links::{
        ClassifiedDocOccurrence, DocReferenceClassification, DocReferenceForm, ResolvedDocLinks,
        resolve_doc_links,
    },
    markdown::{ParsedComments, occurrence, parse_comments, same_target},
};

/// Projects the first eligible successful paragraph for this exact attached target.
/// Known absence is `None`; available comments without a paragraph are empty text.
pub fn summary_text(
    parsed: &ParsedComments<'_, '_>,
    target: CommentTarget<'_>,
) -> Result<Option<String>, DocBuildError> {
    let refuse = || DocBuildError::Binding("documentation summary binding mismatch");
    let attachments = parsed.attachments;
    if !parsed.available()
        || !attachments
            .plan
            .index
            .modules
            .get(attachments.module)
            .is_some_and(|input| std::ptr::eq(input, attachments.input))
        || matches!(target, CommentTarget::Item(export) if !attachments.input.exports.iter().any(|item| std::ptr::eq(item, export)))
    {
        return Err(refuse());
    }
    let original =
        attach_comments(attachments.plan, attachments.module, None).ok_or_else(refuse)?;
    if parsed.blocks.len() != attachments.blocks.len()
        || original.blocks.len() != attachments.blocks.len()
        || attachments
            .blocks
            .iter()
            .zip(&original.blocks)
            .enumerate()
            .any(|(ordinal, (raw, original))| {
                raw.range != original.range
                    || (!parsed.blocks[ordinal].fallback && raw.lines != original.lines)
                    || match (raw.target, original.target) {
                        (Some(left), Some(right)) => !same_target(left, right),
                        (None, None) => false,
                        _ => true,
                    }
            })
        || parsed
            .blocks
            .iter()
            .zip(&attachments.blocks)
            .enumerate()
            .any(|(ordinal, (block, raw))| {
                if block.fallback {
                    let expected = occurrence(attachments, ordinal, raw, 411, None);
                    !parsed.warnings.iter().any(|warning| {
                        warning.code().to_string() == "W0411"
                            && warning.details().entries().get("documentation.occurrence")
                                == Some(&expected)
                    })
                } else {
                    !block.summary_view_complete(raw, attachments)
                }
            })
    {
        return Err(refuse());
    }
    let mut attached = false;
    for (raw, block) in attachments.blocks.iter().zip(&parsed.blocks) {
        if !raw
            .target
            .is_some_and(|candidate| same_target(candidate, target))
        {
            continue;
        }
        attached = true;
        if !block.fallback
            && let Some(text) = block.first_summary()?
        {
            return Ok(Some(text));
        }
    }
    Ok(attached.then(String::new))
}

/// Returns every original export in stored order with the shared plain summary.
pub fn search_summaries<'a>(
    parsed: &ParsedComments<'_, 'a>,
) -> Result<Vec<(&'a VerifiedExport, Option<String>)>, DocBuildError> {
    let exports = &parsed.attachments.input.exports;
    if exports.is_empty() {
        summary_text(parsed, CommentTarget::Module)?;
    }
    exports
        .iter()
        .map(|export| Ok((export, summary_text(parsed, CommentTarget::Item(export))?)))
        .collect()
}

/// Escapes the same shared summaries once into isolated module listing fragments.
pub fn module_summary_fragments<'a>(
    parsed: &ParsedComments<'_, 'a>,
) -> Result<Vec<(&'a VerifiedExport, String)>, DocBuildError> {
    Ok(search_summaries(parsed)?
        .into_iter()
        .map(|(export, text)| {
            (
                export,
                format!(
                    "<p class=\"mizar-summary\">{}</p>\n",
                    escape(text.as_deref().unwrap_or_default(), false)
                ),
            )
        })
        .collect())
}

/// Isolated source views retaining one authenticated source and allocation handoff.
#[derive(Debug)]
pub struct DocSourceViews<'a> {
    inputs: &'a AuthenticatedDocImports,
    sources: &'a AuthenticatedDocSources,
    destinations: DocDestinationAllocations,
    paths: BTreeMap<ModuleSummaryIdentity, String>,
}

impl<'a> DocSourceViews<'a> {
    /// Declares all available source views in the complete G2 auxiliary namespace.
    pub fn allocate(
        inputs: &'a AuthenticatedDocImports,
        identities: &AuthenticatedDocIdentities,
        sources: &'a AuthenticatedDocSources,
        doc_dir: &Path,
        specification_version: &str,
        mut auxiliary: DocAllocationTable,
    ) -> Result<Self, DocBuildError> {
        if !sources.matches_inputs(inputs) {
            return Err(DocBuildError::Binding("source view inventory mismatch"));
        }
        let mut paths = BTreeMap::new();
        for (module, publication) in inputs.inventory() {
            let (source_file, digest) = sources
                .source_binding(module, publication)
                .ok_or(DocBuildError::Binding("source view binding mismatch"))?;
            if digest.is_some() {
                let path = format!("sources/{source_file}.html");
                auxiliary.pages.push(DocPageAllocation {
                    path: path.clone(),
                    module: None,
                    items: Vec::new(),
                    section_ids: Vec::new(),
                });
                paths.insert(module.clone(), path);
            }
        }
        let destinations = allocate_doc_destinations(
            inputs,
            identities,
            doc_dir,
            specification_version,
            auxiliary,
        )?;
        Ok(Self {
            inputs,
            sources,
            destinations,
            paths,
        })
    }

    /// Returns the complete validated destination handoff.
    pub fn destinations(&self) -> &DocDestinationAllocations {
        &self.destinations
    }

    /// Renders the fixed source shell with the owner's authoritative package display name.
    /// Legacy original text remains unavailable; version comes from the bound publication.
    pub fn view(
        &self,
        module: &ModuleSummaryIdentity,
        publication: Hash,
        package_display_name: &str,
    ) -> Result<Option<(&str, String)>, DocBuildError> {
        let (source_file, digest) = self
            .sources
            .source_binding(module, publication)
            .ok_or(DocBuildError::Binding("source view binding mismatch"))?;
        let Some(digest) = digest else {
            return Ok(None);
        };
        let text = self
            .sources
            .original_text(module, publication, source_file, digest)?
            .ok_or(DocBuildError::Binding("source view text mismatch"))?;
        let path = self
            .paths
            .get(module)
            .ok_or(DocBuildError::Binding("source view allocation mismatch"))?;
        let package = &self
            .inputs
            .publications()
            .iter()
            .find(|published| published.artifact_hash == publication)
            .ok_or(DocBuildError::Binding("source view publication mismatch"))?
            .manifest
            .package;
        let mut rows = String::new();
        for (index, line) in text.split('\n').enumerate() {
            let mut visible = String::new();
            for ch in line.chars() {
                if (ch <= '\u{1f}' && ch != '\t') || ch == '\u{7f}' {
                    write!(visible, "\\u{{{:x}}}", u32::from(ch))
                        .expect("writing to String cannot fail");
                } else {
                    visible.push(ch);
                }
            }
            writeln!(
                rows,
                "<li id=\"L{}\"><pre><code>{}</code></pre></li>",
                index + 1,
                escape(&visible, false)
            )
            .expect("writing to String cannot fail");
        }
        let version = package
            .package_version
            .as_ref()
            .map_or_else(String::new, |version| {
                format!(
                    " <span class=\"mizar-version\">{}</span>",
                    escape(version, false)
                )
            });
        let title = escape(source_file, false);
        let package = escape(package_display_name, false);
        let style = relative_source_href(path, "assets/style.css", None);
        let search_script = relative_source_href(path, "assets/search.js", None);
        let math_client = relative_source_href(path, "assets/katex/client.js", None);
        let search_index = relative_source_href(path, "search.json", None);
        Ok(Some((
            path,
            format!(
                "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>{title}</title>\n<link rel=\"stylesheet\" href=\"{style}\">\n<script defer src=\"{search_script}\"></script>\n<script defer src=\"{math_client}\"></script>\n</head>\n<body>\n<header><span class=\"mizar-package\">{package}</span>{version}</header>\n<section class=\"mizar-search\" data-mizar-index=\"{search_index}\"><label>Search <input type=\"search\" class=\"mizar-search-query\"></label><p class=\"mizar-search-status\" role=\"status\"></p><ul class=\"mizar-search-results\"></ul></section>\n<main>\n<h1>{title}</h1>\n<ol class=\"mizar-source-lines\">{rows}</ol>\n</main>\n</body>\n</html>\n"
            ),
        )))
    }

    /// Uses the exact authenticated Artifact export range in the loaded frame.
    pub fn item_source_slot(
        &self,
        module: &ModuleSummaryIdentity,
        publication: Hash,
        export: &VerifiedExport,
        containing_page: &str,
    ) -> Result<String, DocBuildError> {
        if !self
            .inputs
            .exports(module, publication)
            .is_some_and(|exports| exports.contains(export))
        {
            return Err(DocBuildError::Binding("source item binding mismatch"));
        }
        let start = usize::try_from(export.source_range.start_byte)
            .map_err(|_| DocBuildError::Binding("source item range overflow"))?;
        let end = usize::try_from(export.source_range.end_byte)
            .map_err(|_| DocBuildError::Binding("source item range overflow"))?;
        self.loaded_source_slot(
            module,
            publication,
            &[TextRange { start, end }],
            containing_page,
        )
    }

    /// Projects ordered loaded segments through the retained G3 maps without coalescing.
    /// Exact ranges and their provenance remain with the caller.
    pub fn loaded_source_slot(
        &self,
        module: &ModuleSummaryIdentity,
        publication: Hash,
        ranges: &[TextRange],
        containing_page: &str,
    ) -> Result<String, DocBuildError> {
        if !self
            .destinations
            .allocations()
            .contains_page(containing_page)
        {
            return Err(DocBuildError::Binding("source containing page mismatch"));
        }
        let (source_file, digest) = self
            .sources
            .source_binding(module, publication)
            .ok_or(DocBuildError::Binding("source slot binding mismatch"))?;
        let Some(digest) = digest else {
            return Ok("<p class=\"mizar-source-unavailable\">Source unavailable</p>\n".to_owned());
        };
        let text = self
            .sources
            .original_text(module, publication, source_file, digest)?
            .ok_or(DocBuildError::Binding("source slot text mismatch"))?;
        let path = self
            .paths
            .get(module)
            .ok_or(DocBuildError::Binding("source slot allocation mismatch"))?;
        let (segments, coverage) =
            self.sources
                .loaded_ranges(module, publication, source_file, digest, ranges)?;
        source_slot(text, &segments, coverage, containing_page, path)
    }
}

fn source_slot(
    text: &str,
    segments: &[TextRange],
    coverage: DocImportCoverage,
    containing_page: &str,
    path: &str,
) -> Result<String, DocBuildError> {
    if segments.is_empty() {
        return Ok(
            "<p class=\"mizar-source-unavailable\">Source location unavailable</p>\n".to_owned(),
        );
    }
    let mut starts = vec![0];
    starts.extend(text.match_indices('\n').map(|(offset, _)| offset + 1));
    let mut links = Vec::new();
    for segment in segments {
        validate_coordinate_range(text, *segment)?;
        let line = starts.partition_point(|start| *start <= segment.start);
        let href = relative_source_href(containing_page, path, Some(&format!("L{line}")));
        links.push(format!("<a href=\"{href}\">line {line}</a>"));
    }
    let mapping = if coverage == DocImportCoverage::Complete {
        ""
    } else {
        " <span class=\"mizar-source-mapping\">incomplete mapping</span>"
    };
    Ok(format!(
        "<p class=\"mizar-source\">Source: {}{mapping}</p>\n",
        links.join("; ")
    ))
}

fn relative_source_href(from: &str, to: &str, fragment: Option<&str>) -> String {
    let mut from = from.split('/').collect::<Vec<_>>();
    from.pop();
    let to = to.split('/').collect::<Vec<_>>();
    let shared = from
        .iter()
        .zip(&to)
        .take_while(|(left, right)| left == right)
        .count();
    let path = std::iter::repeat_n("..", from.len() - shared)
        .chain(to[shared..].iter().copied())
        .collect::<Vec<_>>()
        .join("/");
    escape(&encode_doc_href(&path, fragment), false)
}

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

#[cfg(test)]
mod source_tests {
    use super::*;

    #[test]
    fn nested_source_hrefs_encode_original_components_once() {
        assert_eq!(
            relative_source_href(
                "深/nested/item.html",
                "sources/非%#&\"'.miz.html",
                Some("L2")
            ),
            "../../sources/%E9%9D%9E%25%23%26%22%27.miz.html#L2"
        );
    }

    #[test]
    fn partial_source_links_keep_order_repeats_points_and_no_end_links() {
        let segments = [
            TextRange { start: 2, end: 5 },
            TextRange { start: 0, end: 1 },
            TextRange { start: 2, end: 2 },
            TextRange { start: 5, end: 5 },
        ];
        assert_eq!(
            source_slot(
                "a\nβ\n",
                &segments,
                DocImportCoverage::Incomplete,
                "items/x.html",
                "sources/a.html"
            )
            .unwrap(),
            "<p class=\"mizar-source\">Source: <a href=\"../sources/a.html#L2\">line 2</a>; <a href=\"../sources/a.html#L1\">line 1</a>; <a href=\"../sources/a.html#L2\">line 2</a>; <a href=\"../sources/a.html#L3\">line 3</a> <span class=\"mizar-source-mapping\">incomplete mapping</span></p>\n"
        );
        assert!(
            source_slot(
                "β",
                &[TextRange { start: 1, end: 1 }],
                DocImportCoverage::Complete,
                "x.html",
                "sources/a.html"
            )
            .is_err()
        );
    }

    #[test]
    fn empty_mapping_never_invents_a_line() {
        for coverage in [
            DocImportCoverage::Complete,
            DocImportCoverage::Incomplete,
            DocImportCoverage::Unavailable,
        ] {
            assert_eq!(
                source_slot("", &[], coverage, "x.html", "sources/a.html").unwrap(),
                "<p class=\"mizar-source-unavailable\">Source location unavailable</p>\n"
            );
        }
    }
}
