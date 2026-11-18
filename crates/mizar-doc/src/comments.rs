//! Read-only grouping and verified attachment of published documentation comments.

use mizar_artifact::{
    module_summary::ModuleSummaryIdentity,
    store::CanonicalJson,
    verified_artifact::{ExportProofStatus, ExportVisibility, VerifiedExport},
};
use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput, DiagnosticPrimaryLocation,
        DiagnosticSpan, FailureCategory, PipelinePhase,
    },
    registry::{DiagnosticCode, DiagnosticRegistry, DiagnosticSeverity},
};
use mizar_session::{LoadedSource, SourceRange, TextRange};

use crate::doc_build::{DocBuildPlan, DocModuleInput};

/// Borrowed inputs and source-ordered attachment results for one complete module.
#[derive(Debug)]
pub struct CommentAttachments<'a> {
    /// Original publication, binding and independent consumer profile.
    pub plan: &'a DocBuildPlan,
    /// Complete owning module identity.
    pub module: &'a ModuleSummaryIdentity,
    /// Original exports, optional summary and unmodified projection/maps.
    pub input: &'a DocModuleInput,
    /// Separate maximal documentation blocks in loaded-source order.
    pub blocks: Vec<CommentBlock<'a>>,
    /// Shared Documentation drafts; promotion never changes proof acceptance.
    pub warnings: Vec<DiagnosticDraft>,
}

impl CommentAttachments<'_> {
    /// Distinguishes unavailable metadata from an available empty collection.
    pub fn available(&self) -> bool {
        self.input.comment_projection != CanonicalJson::Null
    }

    /// Applies only the documentation consumer's warning policy.
    pub fn documentation_failed(&self) -> bool {
        self.plan.profile.fail_on_doc_warnings && !self.warnings.is_empty()
    }
}

/// Raw borrowed lines and candidate evidence, including unsuccessful associations.
#[derive(Debug)]
pub struct CommentBlock<'a> {
    /// Original `[kind, loaded range, raw lexeme]` rows, without body parsing.
    pub lines: Vec<&'a CanonicalJson>,
    /// Half-open loaded-source range, including indentation between lines.
    pub range: TextRange,
    /// Genuine source binding when a matching loaded source was supplied.
    pub source_range: Option<SourceRange>,
    /// Original candidate landmark rows, retaining target origin/FQN pairs.
    pub candidates: Vec<&'a CanonicalJson>,
    /// Verified association; unresolved blocks retain their lines and evidence.
    pub target: Option<CommentTarget<'a>>,
}

/// Association within the enclosing result's complete publication/module identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommentTarget<'a> {
    /// File-prefix documentation, never simultaneously item documentation.
    Module,
    /// The original verified export, never a reconstructed semantic item.
    Item(&'a VerifiedExport),
}

/// Attaches one module from an already validated build plan without source reads.
///
/// `source`, when supplied, must be the consumer's genuine loaded source. Matching
/// source identity, text and maps establish spans, not registry currentness; the
/// shared diagnostic aggregator retains responsibility for freshness. A missing
/// or mismatched binding uses portable ranges and full publication provenance.
pub fn attach_comments<'a>(
    plan: &'a DocBuildPlan,
    module: &ModuleSummaryIdentity,
    source: Option<&LoadedSource>,
) -> Option<CommentAttachments<'a>> {
    let (module, input) = plan.index.modules.get_key_value(module)?;
    let mut result = CommentAttachments {
        plan,
        module,
        input,
        blocks: Vec::new(),
        warnings: Vec::new(),
    };
    if !result.available() {
        return Some(result);
    }
    let projection = &input.comment_projection;
    let loaded = string(field(projection, "loaded"));
    let comments = array(field(projection, "comments"));
    let landmarks = array(field(projection, "landmarks"));
    let source = source.filter(|source| {
        source.package_id.as_str() == module.package_id
            && source.module_path.as_str() == module.module_path
            && source.normalized_path.as_str() == input.artifact.source_file
            && source.source_hash == input.artifact.source_hash
            && source.edition.as_str() == module.language_edition
            && source.text.as_ref() == loaded
            && source.line_map.source_id() == source.source_id
            && source.line_map.source() == loaded
    });
    for row in comments {
        let fields = array(row);
        if string(&fields[0]) != "documentation" {
            continue;
        }
        let range = range(&fields[1]);
        if let Some(previous) = result.blocks.last_mut()
            && adjacent(loaded, previous.range.end, range.start)
            && !comments.iter().any(|comment| {
                let span = range_of(comment);
                previous.range.end <= span.start && span.end <= range.start
            })
        {
            previous.lines.push(row);
            previous.range.end = range.end;
            continue;
        }
        result.blocks.push(CommentBlock {
            lines: vec![row],
            range,
            source_range: None,
            candidates: Vec::new(),
            target: None,
        });
    }
    for (ordinal, block) in result.blocks.iter_mut().enumerate() {
        block.source_range = source.map(|source| SourceRange {
            source_id: source.source_id,
            start: block.range.start,
            end: block.range.end,
        });
        let outcome = if prefix_trivia(loaded, comments, block.range.start) {
            Ok(CommentTarget::Module)
        } else {
            item_target(input, loaded, landmarks, block)
        };
        match outcome {
            Ok(target) => block.target = Some(target),
            Err(number) => {
                let code = DiagnosticCode::from_parts(DiagnosticSeverity::Warning, number)
                    .expect("allocated four-digit warning");
                let descriptor = DiagnosticRegistry::builtin()
                    .lookup(code)
                    .expect("allocated Documentation descriptor");
                let primary_location = block.source_range.map_or_else(
                    || DiagnosticPrimaryLocation::Documentation {
                        publication_hash: plan.manifest.artifact_hash,
                        package_id: module.package_id.clone(),
                        package_version: module.package_version.clone(),
                        lockfile_identity: module.lockfile_identity.clone(),
                        module_path: module.module_path.clone(),
                        language_edition: module.language_edition.clone(),
                        block_ordinal: ordinal as u64,
                    },
                    |range| {
                        DiagnosticPrimaryLocation::Span(
                            DiagnosticSpan::primary(range, None).expect("nonempty raw doc line"),
                        )
                    },
                );
                result.warnings.push(
                    DiagnosticDraft::new(DiagnosticDraftInput {
                        source_snapshot: plan.snapshot,
                        code,
                        phase: PipelinePhase::Documentation,
                        category: FailureCategory::DocumentationWarning,
                        stable_detail_key: descriptor.meaning_key.to_owned(),
                        message: descriptor.summary.to_owned(),
                        primary_location,
                        secondary_spans: Vec::new(),
                        notes: Vec::new(),
                        details: DiagnosticDetails::new(),
                        fixes: Vec::new(),
                        explanation: None,
                    })
                    .expect("allocated Documentation descriptor and complete provenance"),
                );
            }
        }
    }
    Some(result)
}

fn prefix_trivia(loaded: &str, comments: &[CanonicalJson], end: usize) -> bool {
    let mut cursor = 0;
    for comment in comments.iter().filter(|row| range_of(row).end <= end) {
        let span = range_of(comment);
        if !loaded[cursor..span.start].chars().all(char::is_whitespace) {
            return false;
        }
        cursor = span.end;
    }
    loaded[cursor..end].chars().all(char::is_whitespace)
}

fn adjacent(loaded: &str, start: usize, end: usize) -> bool {
    let gap = &loaded[start..end];
    gap.chars().all(|ch| matches!(ch, ' ' | '\t' | '\n'))
        && gap.bytes().filter(|&byte| byte == b'\n').count()
            + usize::from(loaded[..start].ends_with('\n'))
            == 1
}

fn item_target<'a>(
    input: &'a DocModuleInput,
    loaded: &str,
    landmarks: &'a [CanonicalJson],
    block: &mut CommentBlock<'a>,
) -> Result<CommentTarget<'a>, u16> {
    let Some(start) = landmarks
        .iter()
        .map(range_of)
        .filter(|span| span.start >= block.range.end)
        .map(|span| span.start)
        .min()
    else {
        return Err(400);
    };
    if !adjacent(loaded, block.range.end, start) {
        return Err(400);
    }
    for (index, row) in landmarks.iter().enumerate() {
        if range_of(row).start == start {
            collect_candidates(landmarks, loaded, index, &mut block.candidates);
        }
    }
    if block.candidates.iter().any(|row| {
        matches!(
            string(&array(row)[0]),
            "StandaloneDiagnosticAnnotation" | "ImportItem" | "ReserveItem"
        )
    }) {
        return Err(400);
    }
    let supported: Vec<_> = block
        .candidates
        .iter()
        .copied()
        .filter(|row| supported_kind(string(&array(row)[0])))
        .collect();
    if supported.is_empty() {
        return Err(401);
    }
    // Documentation inside a named item cannot acquire a nested public target.
    if landmarks.iter().any(|row| {
        let span = range_of(row);
        span.start < block.range.start
            && block.range.end < span.end
            && supported_kind(string(&array(row)[0]))
    }) {
        return Err(401);
    }
    if supported.len() != 1 {
        return Err(402);
    }
    let row = array(supported[0]);
    let boundary = range(&row[1]);
    let mut annotation_end = None;
    for candidate in &block.candidates {
        let candidate = array(candidate);
        if candidate[4] != CanonicalJson::Null && candidate[4] != row[4] {
            return Err(402);
        }
        if matches!(
            string(&candidate[0]),
            "Annotation" | "LibraryAnnotation" | "VisibilityMarker"
        ) {
            let annotation = range(&candidate[1]);
            if let Some(end) = annotation_end
                && !annotation_gap(loaded, end, annotation.start)
            {
                return Err(400);
            }
            annotation_end = Some(annotation.end);
        }
    }
    if let Some(end) = annotation_end {
        // The real theorem/lemma boundary includes its leading annotations.
        // Its direct children prove ownership; inspect only intervening trivia,
        // without interpreting a theorem head or treating a formula as its head.
        let body_start = if boundary.start < end && end <= boundary.end {
            let Some(offset) = loaded[end..boundary.end].find(|ch: char| !ch.is_whitespace())
            else {
                return Err(401);
            };
            let start = end + offset;
            if array(field(&input.comment_projection, "comments"))
                .iter()
                .any(|comment| range_of(comment).start == start)
            {
                return Err(400);
            }
            start
        } else {
            boundary.start
        };
        if !annotation_gap(loaded, end, body_start) {
            return Err(400);
        }
    }
    if block
        .candidates
        .iter()
        .any(|candidate| array(candidate)[3] == CanonicalJson::Bool(true))
    {
        return Err(403);
    }
    let target = array(&row[4]);
    let matches: Vec<_> = input
        .exports
        .iter()
        .filter(|export| {
            target.len() == 2
                && export.origin_id == string(&target[0])
                && export.fully_qualified_name == string(&target[1])
        })
        .collect();
    let [export] = matches.as_slice() else {
        return Err(401);
    };
    let kind = string(&row[0]);
    let expected = if matches!(kind, "TheoremItem" | "LemmaItem") {
        "theorem"
    } else if kind == "AlgorithmDefinition" {
        "algorithm"
    } else if kind.ends_with("Registration") {
        "registration"
    } else {
        "definition"
    };
    let declaration_kind = match kind {
        "AttributeDefinition" => "attribute",
        "PredicateDefinition" => "predicate",
        "FunctorDefinition" => "functor",
        "ModeDefinition" => "mode",
        "StructureDefinition" => "structure",
        "LemmaItem" => "lemma",
        "AttributeRedefinition"
        | "PredicateRedefinition"
        | "FunctorRedefinition"
        | "InheritanceDefinition"
        | "FieldRedefinition"
        | "PropertyRedefinition" => "redefinition",
        "NotationAlias" => "synonym",
        _ => expected,
    };
    if export.export_kind != expected && export.export_kind != declaration_kind {
        return Err(401);
    }
    if !matches!(
        export.proof_status,
        Some(ExportProofStatus::Accepted | ExportProofStatus::NotRequired)
    ) {
        return Err(403);
    }
    let Some(mapped) = mapped_range(
        input,
        export.source_range.start_byte,
        export.source_range.end_byte,
    ) else {
        return Err(404);
    };
    if mapped.start < boundary.start || mapped.end > boundary.end {
        return Err(402);
    }
    let labels: Vec<_> = input
        .summary
        .as_ref()
        .into_iter()
        .flat_map(|summary| &summary.exported_labels)
        .filter(|label| label.owner_fully_qualified_name == export.fully_qualified_name)
        .collect();
    let visibility = match export.visibility {
        ExportVisibility::Public => "public",
        ExportVisibility::Reexported => "reexported",
        _ => return Err(401),
    };
    if (labels.is_empty() && expected != "algorithm")
        || labels.iter().any(|label| {
            label.target_kind != expected
                || label.label.is_empty()
                || label.visibility != visibility
        })
    {
        return Err(401);
    }
    let ranges: Option<Vec<_>> = labels
        .iter()
        .map(|label| {
            mapped_range(
                input,
                label.source_range.start_byte,
                label.source_range.end_byte,
            )
        })
        .collect();
    let Some(ranges) = ranges else {
        return Err(404);
    };
    if ranges
        .iter()
        .any(|mapped| mapped.start < boundary.start || mapped.end > boundary.end)
    {
        return Err(402);
    }
    Ok(CommentTarget::Item(export))
}

fn collect_candidates<'a>(
    landmarks: &'a [CanonicalJson],
    loaded: &str,
    index: usize,
    out: &mut Vec<&'a CanonicalJson>,
) {
    let row = &landmarks[index];
    if out.iter().any(|previous| std::ptr::eq(*previous, row)) {
        return;
    }
    out.push(row);
    let kind = string(&array(row)[0]);
    let wrapper = matches!(
        kind,
        "VisibleItem"
            | "ExportItem"
            | "AnnotatedStatement"
            | "AnnotatedDefinitionContent"
            | "AnnotatedRegistrationContent"
    );
    if wrapper || supported_kind(kind) {
        let mut prefix_end = range_of(row).start;
        for (child, row) in landmarks.iter().enumerate().skip(index + 1) {
            if array(row)[2] == CanonicalJson::Integer(index as i64)
                && (wrapper
                    || (matches!(
                        string(&array(row)[0]),
                        "Annotation" | "LibraryAnnotation" | "StandaloneDiagnosticAnnotation"
                    ) && loaded
                        .get(prefix_end..range_of(row).start)
                        .is_some_and(|gap| gap.chars().all(char::is_whitespace))))
            {
                collect_candidates(landmarks, loaded, child, out);
                prefix_end = range_of(row).end;
            }
        }
    }
}

fn supported_kind(kind: &str) -> bool {
    matches!(
        kind,
        "TheoremItem"
            | "LemmaItem"
            | "AttributeDefinition"
            | "PredicateDefinition"
            | "FunctorDefinition"
            | "ModeDefinition"
            | "StructureDefinition"
            | "AttributeRedefinition"
            | "PredicateRedefinition"
            | "FunctorRedefinition"
            | "InheritanceDefinition"
            | "FieldRedefinition"
            | "PropertyRedefinition"
            | "NotationAlias"
            | "AlgorithmDefinition"
            | "ExistentialRegistration"
            | "ConditionalRegistration"
            | "FunctorialRegistration"
            | "ReductionRegistration"
    )
}

fn annotation_gap(loaded: &str, start: usize, end: usize) -> bool {
    loaded.get(start..end).is_some_and(|gap| {
        gap.chars().all(|ch| matches!(ch, ' ' | '\t' | '\n'))
            && gap.bytes().filter(|&byte| byte == b'\n').count()
                + usize::from(loaded[..start].ends_with('\n'))
                <= 1
    })
}

// Export/label offsets are lexical; boundaries and raw comment lines are loaded.
// Only complete authored-copy coverage can establish this association.
fn mapped_range(input: &DocModuleInput, start: u64, end: u64) -> Option<TextRange> {
    let (start, end) = (usize::try_from(start).ok()?, usize::try_from(end).ok()?);
    if start >= end {
        return None;
    }
    let mut cursor = start;
    let mut mapped: Option<TextRange> = None;
    for segment in array(field(&input.comment_projection, "preprocess")) {
        let row = array(segment);
        if row[1] == CanonicalJson::Null {
            continue;
        }
        let lexical = range(&row[1]);
        let left = start.max(lexical.start);
        let right = end.min(lexical.end);
        if left >= right {
            continue;
        }
        if string(&row[0]) != "original" || left != cursor {
            return None;
        }
        let loaded = range(&row[2]);
        let part = TextRange {
            start: loaded.start + left - lexical.start,
            end: loaded.start + right - lexical.start,
        };
        if let Some(mapped) = &mut mapped {
            mapped.end = part.end;
        } else {
            mapped = Some(part);
        }
        cursor = right;
    }
    (cursor == end).then_some(mapped).flatten()
}

fn field<'a>(value: &'a CanonicalJson, key: &str) -> &'a CanonicalJson {
    match value {
        CanonicalJson::Object(fields) => &fields[key],
        _ => unreachable!("validated projection object"),
    }
}
fn array(value: &CanonicalJson) -> &[CanonicalJson] {
    match value {
        CanonicalJson::Array(rows) => rows,
        CanonicalJson::Null => &[],
        _ => unreachable!("validated projection array"),
    }
}
fn string(value: &CanonicalJson) -> &str {
    match value {
        CanonicalJson::String(text) => text,
        _ => unreachable!("validated projection string"),
    }
}
fn range(value: &CanonicalJson) -> TextRange {
    let row = array(value);
    let offset = |value: &CanonicalJson| match value {
        CanonicalJson::Integer(value) => *value as usize,
        _ => unreachable!("validated projection offset"),
    };
    TextRange {
        start: offset(&row[0]),
        end: offset(&row[1]),
    }
}
fn range_of(row: &CanonicalJson) -> TextRange {
    range(&array(row)[1])
}
