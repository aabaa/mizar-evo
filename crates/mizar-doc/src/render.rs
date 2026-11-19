//! Isolated documentation fragments and local asset readiness.

use std::{collections::BTreeMap, fmt::Write, ops::Range, path::Path};

use mizar_artifact::{
    module_summary::ModuleSummaryIdentity, store::CanonicalJson, verified_artifact::VerifiedExport,
};

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
        ClassifiedDocOccurrence, DocLinkOutcome, DocReferenceClassification, DocReferenceForm,
        DocReferenceResolution, ResolvedDocLinks, resolve_doc_links,
    },
    markdown::{MarkdownBlock, ParsedComments, occurrence, parse_comments, same_target},
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
                    || if parsed.blocks[ordinal].fallback {
                        // Failed maps may differ, but raw recovery must retain publication text.
                        raw.lines.len() != original.lines.len()
                            || raw
                                .lines
                                .iter()
                                .zip(&original.lines)
                                .any(|(raw, original)| match (raw, original) {
                                    (CanonicalJson::Array(raw), CanonicalJson::Array(original)) => {
                                        raw.len() != original.len()
                                            || raw.first() != original.first()
                                            || raw.get(2..) != original.get(2..)
                                    }
                                    _ => true,
                                })
                    } else {
                        raw.lines != original.lines
                    }
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

/// Emits one safe flow fragment per Doc9 theorem, in the returned UsedBy order.
/// The fresh resolver result retains coverage, repeated occurrences and warnings.
/// No comment reference occurrences are resolved by this isolated presentation.
pub fn used_by_fragments(
    parsed: &ParsedComments<'_, '_>,
    inputs: &AuthenticatedDocImports,
    direct_by: &AuthenticatedDocDirectBy,
    allocations: &ValidatedDocAllocations,
    specification_version: &str,
) -> Result<(Vec<String>, ResolvedDocLinks), DocBuildError> {
    let resolved = resolve_doc_links(
        parsed,
        inputs,
        direct_by,
        allocations,
        specification_version,
        &[],
    )?;
    let mut fragments = Vec::new();
    for entry in &resolved.used_by {
        let mut html = String::new();
        if !entry.available {
            html.push_str("<p>Used by: unavailable (coverage is incomplete or unavailable).</p>\n");
        } else if entry.citing.is_empty() {
            html.push_str("<p>Used by: none.</p>\n");
        }
        if !entry.citing.is_empty() {
            html.push_str("<ul>\n");
            for citing in &entry.citing {
                html.push_str("<li>");
                let label = escape(&citing.export.fully_qualified_name, false);
                if let Some((path, fragment, _)) =
                    allocations.item(&citing.module, citing.publication_hash, &citing.export)
                {
                    let href = encode_doc_href(path, Some(fragment));
                    write!(html, "<a href=\"{}\">{label}</a>", escape(&href, false))
                        .expect("writing to String cannot fail");
                } else {
                    html.push_str(&label);
                }
                html.push_str("</li>\n");
            }
            html.push_str("</ul>\n");
        }
        fragments.push(html);
    }
    Ok((fragments, resolved))
}

/// Safe fragments for the three comment owner slots, retaining each tag occurrence.
#[derive(Debug)]
pub struct DocCommentFragments {
    /// Known absence is `None`; an attached empty comment is `Some("")`.
    pub complete: Option<String>,
    /// Source-ordered param/returns labels and flow bodies; missing slots are empty.
    pub parameters: Vec<(String, String)>,
    /// One phrasing entry for every See field, including missing reference slots.
    pub see: Vec<String>,
}

/// Composes one exact target from authenticated original events, without reparsing bodies.
/// Classifications have one vector per original block, including empty fallback blocks.
/// Within a block, genuine code events, explicit link/image start events and present
/// See slots occur in source encounter order (the owning event, not a reference
/// definition's destination position). Code within link labels, opaque tag slots or
/// math creates no additional occurrence; missing See slots consume no classification.
pub fn comment_fragments(
    parsed: &ParsedComments<'_, '_>,
    target: CommentTarget<'_>,
    inputs: &AuthenticatedDocImports,
    direct_by: &AuthenticatedDocDirectBy,
    allocations: &ValidatedDocAllocations,
    specification_version: &str,
    classifications: &[Vec<DocReferenceClassification>],
) -> Result<(DocCommentFragments, ResolvedDocLinks), DocBuildError> {
    let refuse = || DocBuildError::Binding("documentation comment binding mismatch");
    let attached = summary_text(parsed, target)?.is_some();
    if classifications.len() != parsed.blocks.len() {
        return Err(refuse());
    }
    let mut occurrences = Vec::new();
    let mut bindings = Vec::new();
    for (ordinal, (block, classes)) in parsed.blocks.iter().zip(classifications).enumerate() {
        let mut slots = Vec::new();
        if block.fallback {
            // Recovery views must still be the exact parser product, not caller HTML.
            let expected = parse_comments(parsed.attachments, &[]);
            let original = &expected.blocks[ordinal];
            if !original.fallback
                || block.text != original.text
                || block.source_map != original.source_map
                || block.events != original.events
                || !block.fields.is_empty()
                || !block.math.is_empty()
                || !block.destinations.is_empty()
            {
                return Err(refuse());
            }
        } else {
            for (index, (event, range)) in block.events.iter().enumerate() {
                if matches!(
                    event,
                    Event::Code(_) | Event::Start(Tag::Link { .. } | Tag::Image { .. })
                ) && block.fields.iter().any(|field| {
                    field
                        .slot
                        .as_ref()
                        .is_some_and(|slot| slot.start < range.end && range.start < slot.end)
                }) {
                    continue;
                }
                let candidate =
                    if let Some(destination) = block
                        .destinations
                        .iter()
                        .position(|(event, _)| *event == index)
                    {
                        Some((
                            DocReferenceForm::Explicit,
                            block.destinations[destination].1.clone(),
                            block
                                .interpreted_destination(destination)
                                .ok_or_else(refuse)?
                                .into_owned(),
                        ))
                    } else if let Event::Code(text) = event {
                        (!block.math.iter().any(|math| {
                            math.range.start < range.end && range.start < math.range.end
                        }) && !block.events.iter().any(|(event, outer)| {
                            matches!(event, Event::Start(Tag::Link { .. } | Tag::Image { .. }))
                                && outer.start <= range.start
                                && range.end <= outer.end
                        }))
                        .then(|| (DocReferenceForm::Code, range.clone(), text.to_string()))
                    } else {
                        None
                    };
                if let Some((form, destination, interpreted)) = candidate {
                    slots.push((range.start, Some(index), form, destination, interpreted));
                }
            }
            for field in &block.fields {
                if &block.text[field.marker.clone()] == "@see"
                    && let Some(slot) = &field.slot
                {
                    slots.push((
                        slot.start,
                        None,
                        DocReferenceForm::See,
                        slot.clone(),
                        block.text[slot.clone()].to_owned(),
                    ));
                }
            }
        }
        slots.sort_by_key(|(position, _, _, _, _)| *position);
        if slots.len() != classes.len() {
            return Err(refuse());
        }
        for ((_, event, form, range, interpreted), classification) in slots.into_iter().zip(classes)
        {
            bindings.push(event);
            occurrences.push(ClassifiedDocOccurrence {
                block: ordinal,
                destination_ranges: vec![range.clone()],
                range,
                form,
                interpreted: Some(interpreted),
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
    // Apply the same URL policy once per original occurrence, never per derived view.
    for (ordinal, reference) in resolved.references.iter().enumerate() {
        let occurrence = &reference.occurrence;
        if occurrence.classification == DocReferenceClassification::External
            && let Some(url) = occurrence.interpreted.as_deref()
            && !accepted_url(url)
        {
            let block = &parsed.blocks[occurrence.block];
            let Some(destination) = bindings[ordinal].and_then(|event| {
                block
                    .destinations
                    .iter()
                    .position(|(index, _)| *index == event)
            }) else {
                continue;
            };
            let image = bindings[ordinal].is_some_and(|event| {
                matches!(block.events[event].0, Event::Start(Tag::Image { .. }))
            });
            resolved.warnings.push(url_draft(
                parsed,
                occurrence.block,
                destination,
                image,
                &occurrence.range,
                url,
            ));
        }
    }
    let math = opaque_math_fragments(parsed)?;
    let mut math_offset = 0;
    let mut result = DocCommentFragments {
        complete: attached.then(String::new),
        parameters: Vec::new(),
        see: Vec::new(),
    };
    for (ordinal, (block, raw)) in parsed
        .blocks
        .iter()
        .zip(&parsed.attachments.blocks)
        .enumerate()
    {
        let block_math = &math[math_offset..math_offset + block.math.len()];
        math_offset += block.math.len();
        if !raw
            .target
            .is_some_and(|candidate| same_target(candidate, target))
        {
            continue;
        }
        if block.fallback {
            let complete = result.complete.as_mut().ok_or_else(refuse)?;
            complete.push_str("<pre>");
            for row in &raw.lines {
                if let mizar_artifact::store::CanonicalJson::Array(fields) = row
                    && let Some(mizar_artifact::store::CanonicalJson::String(text)) = fields.get(2)
                {
                    complete.push_str(&escape(text, true));
                }
            }
            complete.push_str("</pre>\n");
            continue;
        }
        let references: Vec<_> = resolved
            .references
            .iter()
            .zip(&bindings)
            .filter(|(reference, _)| reference.occurrence.block == ordinal)
            .collect();
        result
            .complete
            .as_mut()
            .ok_or_else(refuse)?
            .push_str(&comment_view(
                block,
                std::slice::from_ref(&(0..block.text.len())),
                &references,
                block_math,
                false,
            )?);
        for field in &block.fields {
            let marker = &block.text[field.marker.clone()];
            match marker {
                "@param" | "@returns" => {
                    let label = if marker == "@returns" {
                        "Returns".to_owned()
                    } else {
                        field
                            .slot
                            .as_ref()
                            .map(|slot| block.text[slot.clone()].to_owned())
                            .unwrap_or_default()
                    };
                    result.parameters.push((
                        label,
                        comment_view(block, &field.body, &references, block_math, false)?,
                    ));
                }
                "@see" => {
                    let mut entry = String::new();
                    if let Some(slot) = &field.slot {
                        entry.push_str(&comment_view(
                            block,
                            std::slice::from_ref(slot),
                            &references,
                            block_math,
                            true,
                        )?);
                    }
                    let body = comment_view(block, &field.body, &references, block_math, true)?;
                    if !entry.is_empty() && !body.is_empty() {
                        entry.push(' ');
                    }
                    entry.push_str(&body);
                    result.see.push(entry);
                }
                _ => {}
            }
        }
    }
    Ok((result, resolved))
}

fn reference_wrapper(reference: &DocReferenceResolution) -> (String, String) {
    match &reference.outcome {
        DocLinkOutcome::Linked { href }
            if reference.occurrence.classification != DocReferenceClassification::External
                || accepted_url(href) =>
        {
            (
                format!("<a href=\"{}\">", escape(href, false)),
                "</a>".to_owned(),
            )
        }
        DocLinkOutcome::Incomplete => (
            "<span class=\"mizar-unavailable\">".to_owned(),
            " (unavailable)</span>".to_owned(),
        ),
        _ => (String::new(), String::new()),
    }
}

fn comment_view(
    block: &MarkdownBlock,
    selected: &[Range<usize>],
    references: &[(&DocReferenceResolution, &Option<usize>)],
    math: &[String],
    phrasing: bool,
) -> Result<String, DocBuildError> {
    let refuse = || DocBuildError::Binding("documentation comment projection mismatch");
    let overlaps = |range: &Range<usize>| {
        selected
            .iter()
            .any(|part| part.start < range.end && range.start < part.end)
    };
    let mut html = String::new();
    // Original event nesting supplies balanced clipped wrappers, including tight lists.
    let mut stack: Vec<(String, String, bool)> = Vec::new();
    let mut emitted_math = vec![false; math.len()];
    let mut emitted_see = Vec::new();
    let mut skip_image = 0;
    for (index, (event, range)) in block.events.iter().enumerate() {
        if skip_image > 0 {
            match event {
                Event::Start(_) => skip_image += 1,
                Event::End(_) => skip_image -= 1,
                _ => {}
            }
            continue;
        }
        if let Event::End(_) = event {
            let (_, close, _) = stack.pop().ok_or_else(refuse)?;
            html.push_str(&close);
            continue;
        }
        let inside_math = block
            .math
            .iter()
            .any(|math| math.range.start <= range.start && range.end <= math.range.end);
        if let Event::Start(tag) = event {
            let active = overlaps(range) && !inside_math;
            let reference = references
                .iter()
                .find(|(_, event)| **event == Some(index))
                .map(|(reference, _)| *reference);
            if let Tag::Image { .. } = tag {
                let opaque_slot = references.iter().find(|(reference, _)| {
                    reference.occurrence.form == DocReferenceForm::See
                        && reference.occurrence.range.start <= range.start
                        && range.end <= reference.occurrence.range.end
                });
                let image_url = reference.and_then(|reference| match &reference.outcome {
                    DocLinkOutcome::Linked { href }
                        if reference.occurrence.classification
                            != DocReferenceClassification::External
                            || accepted_url(href) =>
                    {
                        Some(href)
                    }
                    _ => None,
                });
                if opaque_slot.is_some() || image_url.is_some() {
                    if active {
                        if let Some((reference, _)) = opaque_slot {
                            let occurrence = &reference.occurrence;
                            if !emitted_see.contains(&occurrence.range) {
                                emitted_see.push(occurrence.range.clone());
                                let (open, close) = reference_wrapper(reference);
                                html.push_str(&open);
                                html.push_str(&escape(&block.text[occurrence.range.clone()], true));
                                html.push_str(&close);
                            }
                        } else if let Some(href) = image_url {
                            let mut label = String::new();
                            let mut depth = 1;
                            for (event, _) in &block.events[index + 1..] {
                                match event {
                                    Event::Start(_) => depth += 1,
                                    Event::End(_) => depth -= 1,
                                    Event::Text(text)
                                    | Event::Code(text)
                                    | Event::InlineHtml(text) => label.push_str(text),
                                    Event::SoftBreak | Event::HardBreak => label.push(' '),
                                    _ => {}
                                }
                                if depth == 0 {
                                    break;
                                }
                            }
                            write!(
                                html,
                                "<img src=\"{}\" alt=\"{}\">",
                                escape(href, false),
                                escape(&label, true)
                            )
                            .expect("String write");
                            // Alt remains the original flattened label. Opaque inline math
                            // needs an actual element, placed immediately after the image.
                            for (ordinal, candidate) in block.math.iter().enumerate() {
                                if range.start <= candidate.range.start
                                    && candidate.range.end <= range.end
                                    && overlaps(&candidate.range)
                                    && !emitted_math[ordinal]
                                {
                                    html.push_str(&math[ordinal]);
                                    emitted_math[ordinal] = true;
                                }
                            }
                        }
                    }
                    skip_image = 1;
                    continue;
                }
                // Rejected/unlinked images retain their original inline label events,
                // including formatting and opaque math, without an image load.
            }
            let phrasing_wrapper = matches!(
                tag,
                Tag::Paragraph
                    | Tag::Heading { .. }
                    | Tag::Emphasis
                    | Tag::Strong
                    | Tag::Link { .. }
                    | Tag::Image { .. }
            );
            let (open, close) = if !active {
                (String::new(), String::new())
            } else {
                match tag {
                    Tag::Emphasis => ("<em>".into(), "</em>".into()),
                    Tag::Strong => ("<strong>".into(), "</strong>".into()),
                    Tag::Link { .. } | Tag::Image { .. } => {
                        reference.map(reference_wrapper).unwrap_or_default()
                    }
                    _ if phrasing => (String::new(), String::new()),
                    Tag::Paragraph => ("<p>".into(), "</p>\n".into()),
                    Tag::Heading { level, .. } => (format!("<{level}>"), format!("</{level}>\n")),
                    Tag::BlockQuote(_) => ("<blockquote>\n".into(), "</blockquote>\n".into()),
                    Tag::List(Some(start)) => {
                        (format!("<ol start=\"{start}\">\n"), "</ol>\n".into())
                    }
                    Tag::List(None) => ("<ul>\n".into(), "</ul>\n".into()),
                    Tag::Item => ("<li>".into(), "</li>\n".into()),
                    Tag::CodeBlock(_) => ("<pre><code>".into(), "</code></pre>\n".into()),
                    Tag::HtmlBlock => (String::new(), String::new()),
                    _ => return Err(refuse()),
                }
            };
            html.push_str(&open);
            stack.push((open, close, phrasing_wrapper));
            continue;
        }
        if !overlaps(range) {
            continue;
        }
        let mut overlays: Vec<(Range<usize>, String, bool)> = block
            .math
            .iter()
            .enumerate()
            .filter(|(_, math)| {
                overlaps(&math.range)
                    && math.range.start < range.end
                    && range.start < math.range.end
            })
            .map(|(ordinal, candidate)| {
                let fragment = if emitted_math[ordinal] {
                    String::new()
                } else {
                    emitted_math[ordinal] = true;
                    math[ordinal].clone()
                };
                (candidate.range.clone(), fragment, candidate.display)
            })
            .collect();
        for (reference, _) in references {
            let occurrence = &reference.occurrence;
            if occurrence.form == DocReferenceForm::See
                && occurrence.range.start < range.end
                && range.start < occurrence.range.end
                && overlaps(&occurrence.range)
            {
                let fragment = if emitted_see.contains(&occurrence.range) {
                    String::new()
                } else {
                    emitted_see.push(occurrence.range.clone());
                    let (open, close) = reference_wrapper(reference);
                    format!(
                        "{open}{}{close}",
                        escape(&block.text[occurrence.range.clone()], true)
                    )
                };
                overlays.push((occurrence.range.clone(), fragment, false));
            }
        }
        overlays.sort_by_key(|(range, _, _)| range.start);
        let text = match event {
            Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
                text.as_ref()
            }
            Event::SoftBreak => "\n",
            Event::HardBreak => "",
            Event::Rule => "",
            _ => return Err(refuse()),
        };
        let code = matches!(event, Event::Code(_));
        let reference = references
            .iter()
            .find(|(_, event)| **event == Some(index))
            .map(|(reference, _)| *reference);
        let (open, close) = if code && overlays.is_empty() {
            reference.map(reference_wrapper).unwrap_or_default()
        } else {
            (String::new(), String::new())
        };
        html.push_str(&open);
        let code_wrapped = code && overlays.is_empty();
        if code_wrapped {
            html.push_str("<code>");
        }
        let mut cursor = range.start;
        for (overlay, fragment, display) in overlays.into_iter().chain(std::iter::once((
            range.end..range.end,
            String::new(),
            false,
        ))) {
            let end = overlay.start.min(range.end);
            for part in selected {
                let start = cursor.max(part.start);
                let stop = end.min(part.end);
                if start < stop {
                    let visible = if start == range.start && stop == range.end {
                        text.to_owned()
                    } else if code {
                        // Project the genuine code payload through CommonMark's
                        // delimiter/newline/edge-space transformation, then verify
                        // it against the authoritative original event text.
                        let source = &block.text[range.clone()];
                        let delimiter = source.bytes().take_while(|byte| *byte == b'`').count();
                        let end = source.len().checked_sub(delimiter).ok_or_else(refuse)?;
                        let payload = source.get(delimiter..end).ok_or_else(refuse)?;
                        let mut characters = payload.char_indices().peekable();
                        let mut mapped = Vec::new();
                        while let Some((offset, character)) = characters.next() {
                            let mut length = character.len_utf8();
                            if character == '\r'
                                && characters.peek().is_some_and(|(_, next)| *next == '\n')
                            {
                                characters.next();
                                length += 1;
                            }
                            let position = range.start + delimiter + offset;
                            mapped.push((
                                position..position + length,
                                if matches!(character, '\r' | '\n') {
                                    ' '
                                } else {
                                    character
                                },
                            ));
                        }
                        // Doc8 already removed container prefixes from these exact
                        // paragraph segments; keep that original coordinate handoff.
                        mapped.retain(|(source, _)| {
                            block.paragraphs.iter().any(|paragraph| {
                                paragraph.segments.iter().any(|segment| {
                                    segment.start <= source.start && source.end <= segment.end
                                })
                            })
                        });
                        if mapped
                            .first()
                            .is_some_and(|(_, character)| *character == ' ')
                            && mapped
                                .last()
                                .is_some_and(|(_, character)| *character == ' ')
                            && mapped.iter().any(|(_, character)| *character != ' ')
                        {
                            mapped.pop();
                            mapped.remove(0);
                        }
                        if mapped
                            .iter()
                            .map(|(_, character)| *character)
                            .collect::<String>()
                            != text
                        {
                            return Err(refuse());
                        }
                        mapped
                            .into_iter()
                            .filter(|(source, _)| source.start < stop && start < source.end)
                            .map(|(_, character)| character)
                            .collect()
                    } else {
                        let prefix = &block.text[range.start..start];
                        let suffix = &block.text[stop..range.end];
                        text.strip_prefix(prefix)
                            .and_then(|text| text.strip_suffix(suffix))
                            .ok_or_else(refuse)?
                            .to_owned()
                    };
                    html.push_str(&escape(&visible, true));
                }
            }
            if !fragment.is_empty() {
                let suspend = display.then(|| {
                    stack
                        .iter()
                        .position(|(open, _, phrasing_wrapper)| {
                            *phrasing_wrapper && !open.is_empty()
                        })
                        .unwrap_or(stack.len())
                });
                if let Some(start) = suspend {
                    for (_, close, _) in stack[start..].iter().rev() {
                        html.push_str(close);
                    }
                }
                html.push_str(&fragment);
                if let Some(start) = suspend {
                    for (open, _, _) in &stack[start..] {
                        html.push_str(open);
                    }
                }
            }
            cursor = cursor.max(overlay.end);
        }
        if code_wrapped {
            html.push_str("</code>");
        }
        html.push_str(&close);
        if matches!(event, Event::HardBreak) {
            html.push_str("<br>\n");
        }
        if matches!(event, Event::Rule) && !phrasing {
            html.push_str("<hr>\n");
        }
    }
    if !stack.is_empty() {
        return Err(refuse());
    }
    Ok(html)
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
