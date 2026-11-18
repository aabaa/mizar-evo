//! Read-only resolution of explicitly classified documentation occurrences.

use std::{cmp::Ordering, collections::BTreeSet, ops::Range};

use mizar_artifact::{
    direct_by::{DirectByEndpoint, DirectByOccurrence, DirectByTarget},
    module_summary::{ExportedLabelSummary, ModuleSummaryIdentity},
    verified_artifact::VerifiedExport,
};
use mizar_diagnostics::{
    failure_record::{
        DiagnosticDetailValue as Value, DiagnosticDetails, DiagnosticDraft, DiagnosticDraftInput,
        DiagnosticPrimaryLocation, DiagnosticSpan, FailureCategory, PipelinePhase,
    },
    registry::{DiagnosticCode, DiagnosticRegistry, DiagnosticSeverity},
};
use mizar_session::{Hash, SourceRange};

use crate::{
    allocations::{ValidatedDocAllocations, encode_doc_href},
    comments::CommentTarget,
    doc_build::{
        AuthenticatedDocDirectBy, AuthenticatedDocImports, DocBuildError, DocImportCoverage,
    },
    markdown::ParsedComments,
};

/// The original reference syntax supplied by the parsing consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DocReferenceForm {
    /// One complete inline-code label, never an expression fragment.
    Code,
    /// An explicit Markdown link destination.
    Explicit,
    /// The present reference slot of an `@see` field.
    See,
}

/// Producer classification; the resolver introduces no destination grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DocReferenceClassification {
    /// Exact unqualified exported label.
    Label,
    /// Exact fully qualified exported name.
    Fqn,
    /// Opaque logical id in the bound specification index.
    Section,
    /// An external URL passed through unchanged.
    External,
    /// Ordinary code, expression or other intentionally unlinked content.
    Ordinary,
    /// Missing or unsupported destination classification.
    Unavailable,
}

/// A producer handoff in one parsed block's unmodified logical coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifiedDocOccurrence {
    /// True ordinal in the original attachment collection.
    pub block: usize,
    /// Raw destination spelling and logical subject identity.
    pub range: Range<usize>,
    /// Logical destination segments, in encounter order, without bridge spans.
    pub destination_ranges: Vec<Range<usize>>,
    /// Original syntax form.
    pub form: DocReferenceForm,
    /// Already interpreted exact destination, when available.
    pub interpreted: Option<String>,
    /// Authoritative classification, without fallback inference.
    pub classification: DocReferenceClassification,
}

/// A complete authenticated target, including original label and visibility data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocLinkTarget {
    /// Complete owning module identity.
    pub module: ModuleSummaryIdentity,
    /// Publication against which the export was authenticated.
    pub publication_hash: Hash,
    /// Exact stored export, including origin, FQN and visibility.
    pub export: VerifiedExport,
    /// Original authoritative labels for this export, without normalization.
    pub labels: Vec<ExportedLabelSummary>,
}

/// One distinct route; different semantic import positions remain distinct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocLinkRoute {
    /// Authenticated target reached through this route.
    pub candidate: DocLinkTarget,
    /// `current`, `import` or `closure`.
    pub kind: &'static str,
    /// Original semantic-order import ordinal, otherwise absent.
    pub position: Option<usize>,
    /// Exact module/publication queried directly.
    pub endpoint: (ModuleSummaryIdentity, Hash),
}

/// Presentation result; incomplete inputs never become an empty complete answer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DocLinkOutcome {
    /// Complete unique target with a validated encoded href, or an external URL.
    Linked {
        /// Encoded relative href or unchanged external URL.
        href: String,
    },
    /// Complete search found no target, or classification was unavailable.
    Unresolved,
    /// Complete lookup found multiple distinct targets.
    Ambiguous,
    /// Required coverage or a unique target's destination is unavailable.
    Incomplete,
    /// Ordinary content, absent `@see` slots or failed presentation blocks.
    Excluded,
}

/// Lookup retains authenticated candidates and all distinct provenance routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocReferenceResolution {
    /// Original producer handoff.
    pub occurrence: ClassifiedDocOccurrence,
    /// Safe presentation outcome.
    pub outcome: DocLinkOutcome,
    /// Distinct complete target identities in display order.
    pub candidates: Vec<DocLinkTarget>,
    /// Known routes, independently of lookup completeness.
    pub routes: Vec<DocLinkRoute>,
}

/// Direct syntactic citation reversal for one theorem in the attached module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocUsedBy {
    /// The exact cited theorem.
    pub target: DocLinkTarget,
    /// Unique sorted citing theorem identities.
    pub citing: Vec<DocLinkTarget>,
    /// Original mapped occurrences, retaining repeated and self-citations.
    pub occurrences: Vec<DirectByOccurrence>,
    /// Completeness of the explicit finite dependency closure.
    pub closure_coverage: DocImportCoverage,
    /// Independent completeness of citation examinations.
    pub citation_coverage: DocImportCoverage,
    /// Whether the display can claim completeness over that finite closure.
    pub available: bool,
}

/// Gate-5 results; no rendering, publication or proof state is changed.
#[derive(Debug)]
pub struct ResolvedDocLinks {
    /// Reference outcomes in supplied occurrence order.
    pub references: Vec<DocReferenceResolution>,
    /// Local theorem back-references in canonical identity order.
    pub used_by: Vec<DocUsedBy>,
    /// Existing parsed warnings followed by reference and input-scope drafts.
    pub warnings: Vec<DiagnosticDraft>,
    fail_on_doc_warnings: bool,
}

impl ResolvedDocLinks {
    /// Applies only the existing independent documentation quality profile.
    pub fn documentation_failed(&self) -> bool {
        self.fail_on_doc_warnings && !self.warnings.is_empty()
    }
}

/// Resolves explicit classified handoffs against the same authenticated products.
///
/// Binding refusals precede all presentation drafts. This function performs no
/// source discovery, package selection, lexical classification or filesystem IO.
pub fn resolve_doc_links(
    parsed: &ParsedComments<'_, '_>,
    inputs: &AuthenticatedDocImports,
    direct_by: &AuthenticatedDocDirectBy,
    allocations: &ValidatedDocAllocations,
    specification_version: &str,
    occurrences: &[ClassifiedDocOccurrence],
) -> Result<ResolvedDocLinks, DocBuildError> {
    let attachments = parsed.attachments;
    if !inputs.matches_plan(attachments.plan)
        || !direct_by.matches_inputs(inputs)
        || !allocations.matches_inputs(inputs, specification_version)
        || allocations.output_root() != attachments.plan.request.output_dir
        || attachments.plan.index.modules.get(attachments.module) != Some(attachments.input)
        || parsed.blocks.len() != attachments.blocks.len()
    {
        return Err(DocBuildError::Binding(
            "documentation resolver product binding mismatch",
        ));
    }
    if attachments.blocks.iter().any(|block| {
        matches!(block.target, Some(CommentTarget::Item(export)) if !attachments.input.exports.contains(export))
    }) {
        return Err(DocBuildError::Binding("documentation attached target mismatch"));
    }
    for occurrence in occurrences {
        let Some(block) = parsed.blocks.get(occurrence.block) else {
            return Err(DocBuildError::Binding(
                "documentation occurrence block mismatch",
            ));
        };
        if block.fallback {
            continue;
        }
        if block.text.get(occurrence.range.clone()).is_none()
            || occurrence.destination_ranges.iter().any(|range| {
                range.start < occurrence.range.start
                    || range.end > occurrence.range.end
                    || block.text.get(range.clone()).is_none()
            })
        {
            return Err(DocBuildError::Binding(
                "documentation occurrence range mismatch",
            ));
        }
    }
    let mut result = ResolvedDocLinks {
        references: Vec::new(),
        used_by: Vec::new(),
        warnings: parsed.warnings.clone(),
        fail_on_doc_warnings: attachments.plan.profile.fail_on_doc_warnings,
    };
    for occurrence in occurrences {
        let (reference, causes) = resolve_reference(parsed, inputs, allocations, occurrence);
        let number = match reference.outcome {
            DocLinkOutcome::Unresolved => Some(412),
            DocLinkOutcome::Ambiguous => Some(413),
            DocLinkOutcome::Incomplete => Some(414),
            _ => None,
        };
        if let Some(number) = number {
            result.warnings.push(link_draft(
                parsed,
                specification_version,
                Some(&reference),
                ("reference", None),
                number,
                causes,
                Value::List(Vec::new()),
            ));
        }
        result.references.push(reference);
    }
    let publication = attachments.plan.manifest.artifact_hash;
    let module = attachments.module;
    let page_causes = allocations
        .page_causes(module, publication, None)
        .into_iter()
        .map(|reason| cause(reason, affected(module, publication, None)))
        .collect::<Vec<_>>();
    if !page_causes.is_empty() {
        result.warnings.push(link_draft(
            parsed,
            specification_version,
            None,
            ("module_page", None),
            414,
            page_causes,
            Value::List(Vec::new()),
        ));
    }
    let section_causes = allocations
        .section_causes()
        .into_iter()
        .map(|reason| cause(reason, empty()))
        .collect::<Vec<_>>();
    if !section_causes.is_empty() {
        result.warnings.push(link_draft(
            parsed,
            specification_version,
            None,
            ("section_index", None),
            414,
            section_causes,
            Value::List(Vec::new()),
        ));
    }
    let (used_by, causes, coverage) = reverse_citations(parsed, inputs, direct_by);
    for entry in &used_by {
        if !causes.is_empty() {
            result.warnings.push(link_draft(
                parsed,
                specification_version,
                None,
                ("used_by", Some(&entry.target)),
                414,
                causes.clone(),
                coverage.clone(),
            ));
        }
    }
    result.used_by = used_by;
    Ok(result)
}

fn target(
    inputs: &AuthenticatedDocImports,
    module: &ModuleSummaryIdentity,
    hash: Hash,
    export: &VerifiedExport,
) -> DocLinkTarget {
    DocLinkTarget {
        module: module.clone(),
        publication_hash: hash,
        export: export.clone(),
        labels: inputs
            .summary(module, hash)
            .into_iter()
            .flat_map(|summary| &summary.exported_labels)
            .filter(|label| label.owner_fully_qualified_name == export.fully_qualified_name)
            .cloned()
            .collect(),
    }
}

fn target_order(left: &DocLinkTarget, right: &DocLinkTarget) -> Ordering {
    left.module
        .cmp(&right.module)
        .then_with(|| {
            left.export
                .fully_qualified_name
                .cmp(&right.export.fully_qualified_name)
        })
        .then_with(|| left.export.origin_id.cmp(&right.export.origin_id))
        .then_with(|| {
            left.publication_hash
                .to_lower_hex()
                .cmp(&right.publication_hash.to_lower_hex())
        })
}

fn matches(
    inputs: &AuthenticatedDocImports,
    module: &ModuleSummaryIdentity,
    hash: Hash,
    text: &str,
    fqn: bool,
) -> (Vec<DocLinkTarget>, bool) {
    let mut pending = vec![(module, hash)];
    let mut visited = BTreeSet::new();
    let mut candidates = Vec::new();
    let mut complete = true;
    while let Some((module, hash)) = pending.pop() {
        if !visited.insert(module) {
            continue;
        }
        let summary = inputs.summary(module, hash);
        candidates.extend(
            inputs
                .exports(module, hash)
                .into_iter()
                .flatten()
                .filter(|export| {
                    if fqn {
                        export.fully_qualified_name == text
                    } else {
                        summary.is_some_and(|summary| {
                            summary.exported_labels.iter().any(|label| {
                                label.label == text
                                    && label.owner_fully_qualified_name
                                        == export.fully_qualified_name
                                    && (label.visibility == "public"
                                        || label.visibility == "reexported")
                            })
                        })
                    }
                })
                .map(|export| target(inputs, module, hash, export)),
        );
        if !fqn {
            let Some(summary) = summary else {
                complete = false;
                continue;
            };
            for edge in &summary.reexports {
                if let Some(endpoint) = inputs
                    .inventory()
                    .find(|(module, _)| *module == &edge.target_module)
                {
                    pending.push(endpoint);
                } else {
                    complete = false;
                }
            }
        }
    }
    (candidates, complete)
}

fn coverage_reason(
    coverage: DocImportCoverage,
    unavailable: &'static str,
    partial: &'static str,
) -> Option<&'static str> {
    match coverage {
        DocImportCoverage::Complete => None,
        DocImportCoverage::Unavailable => Some(unavailable),
        _ => Some(partial),
    }
}

fn route(
    reference: &mut DocReferenceResolution,
    candidates: Vec<DocLinkTarget>,
    kind: &'static str,
    position: Option<usize>,
    module: &ModuleSummaryIdentity,
    publication: Hash,
) {
    for candidate in candidates {
        reference.routes.push(DocLinkRoute {
            candidate: candidate.clone(),
            kind,
            position,
            endpoint: (module.clone(), publication),
        });
        if !reference.candidates.contains(&candidate) {
            reference.candidates.push(candidate);
        }
    }
}

fn resolve_reference(
    parsed: &ParsedComments<'_, '_>,
    inputs: &AuthenticatedDocImports,
    allocations: &ValidatedDocAllocations,
    occurrence: &ClassifiedDocOccurrence,
) -> (DocReferenceResolution, Vec<Value>) {
    let mut reference = DocReferenceResolution {
        occurrence: occurrence.clone(),
        outcome: DocLinkOutcome::Excluded,
        candidates: Vec::new(),
        routes: Vec::new(),
    };
    let block = &parsed.blocks[occurrence.block];
    if block.fallback
        || occurrence.classification == DocReferenceClassification::Ordinary
        || (occurrence.form == DocReferenceForm::See
            && !block.fields.iter().any(|field| {
                field
                    .slot
                    .as_ref()
                    .is_some_and(|slot| slot == &occurrence.range)
                    && block.text.get(field.marker.clone()) == Some("@see")
            }))
    {
        return (reference, Vec::new());
    }
    let Some(text) = occurrence.interpreted.as_deref() else {
        reference.outcome = DocLinkOutcome::Unresolved;
        return (
            reference,
            vec![cause("classification_unavailable", empty())],
        );
    };
    if occurrence.classification == DocReferenceClassification::External {
        reference.outcome = DocLinkOutcome::Linked {
            href: text.to_owned(),
        };
        return (reference, Vec::new());
    }
    if occurrence.classification == DocReferenceClassification::Section {
        let causes = allocations
            .section_causes()
            .into_iter()
            .map(|reason| cause(reason, empty()))
            .collect::<Vec<_>>();
        if !causes.is_empty() {
            reference.outcome = DocLinkOutcome::Incomplete;
            return (reference, causes);
        }
        if let Some((path, fragment)) = allocations.section(text) {
            reference.outcome = DocLinkOutcome::Linked {
                href: encode_doc_href(path, Some(fragment)),
            };
            return (reference, Vec::new());
        }
        reference.outcome = DocLinkOutcome::Unresolved;
        return (reference, vec![cause("section_absent", empty())]);
    }
    let module = parsed.attachments.module;
    let publication = parsed.attachments.plan.manifest.artifact_hash;
    let mut causes = Vec::new();
    match occurrence.classification {
        DocReferenceClassification::Label => {
            let (candidates, complete) = matches(inputs, module, publication, text, false);
            route(
                &mut reference,
                candidates,
                "current",
                None,
                module,
                publication,
            );
            if !complete && !reference.candidates.is_empty() {
                causes.push(cause(
                    "imports_partial",
                    affected(module, publication, None),
                ));
            }
            if reference.candidates.is_empty() {
                if let Some(reason) = coverage_reason(
                    inputs.coverage(module, publication),
                    "imports_unavailable",
                    "imports_partial",
                ) {
                    causes.push(cause(reason, affected(module, publication, None)));
                }
                if inputs.summary(module, publication).is_none() && causes.is_empty() {
                    causes.push(cause(
                        "imports_unavailable",
                        affected(module, publication, None),
                    ));
                }
                if let Some(bindings) = inputs.bound_imports(module, publication) {
                    for (binding, endpoint) in bindings {
                        if let Some((module, hash)) = endpoint {
                            route(
                                &mut reference,
                                matches(inputs, module, hash, text, false).0,
                                "import",
                                Some(binding.ordinal()),
                                module,
                                hash,
                            );
                            if inputs.summary(module, hash).is_none() {
                                causes.push(cause("imports_partial", affected(module, hash, None)));
                            }
                        }
                    }
                }
            }
        }
        DocReferenceClassification::Fqn => {
            for (module, hash) in inputs.inventory() {
                route(
                    &mut reference,
                    matches(inputs, module, hash, text, true).0,
                    "closure",
                    None,
                    module,
                    hash,
                );
            }
            if let Some(reason) = coverage_reason(
                inputs.closure_coverage(),
                "closure_unavailable",
                "closure_partial",
            ) {
                causes.push(cause(reason, empty()));
            }
        }
        _ => {
            reference.outcome = DocLinkOutcome::Unresolved;
            return (
                reference,
                vec![cause("classification_unavailable", empty())],
            );
        }
    }
    reference.candidates.sort_by(target_order);
    reference.routes.sort_by(route_order);
    reference.routes.dedup();
    if !causes.is_empty() {
        reference.outcome = DocLinkOutcome::Incomplete;
    } else if reference.candidates.len() > 1 {
        reference.outcome = DocLinkOutcome::Ambiguous;
        causes.push(cause("multiple_targets", empty()));
    } else if let Some(candidate) = reference.candidates.first() {
        causes.extend(
            allocations
                .page_causes(
                    &candidate.module,
                    candidate.publication_hash,
                    Some(&candidate.export),
                )
                .into_iter()
                .map(|reason| {
                    cause(
                        reason,
                        affected(
                            &candidate.module,
                            candidate.publication_hash,
                            Some(&candidate.export),
                        ),
                    )
                }),
        );
        if causes.is_empty() {
            if let Some((path, fragment, _)) = allocations.item(
                &candidate.module,
                candidate.publication_hash,
                &candidate.export,
            ) {
                reference.outcome = DocLinkOutcome::Linked {
                    href: encode_doc_href(path, Some(fragment)),
                };
            } else {
                reference.outcome = DocLinkOutcome::Incomplete;
                causes.push(cause(
                    "label_unavailable",
                    affected(
                        &candidate.module,
                        candidate.publication_hash,
                        Some(&candidate.export),
                    ),
                ));
            }
        } else {
            reference.outcome = DocLinkOutcome::Incomplete;
        }
    } else {
        reference.outcome = DocLinkOutcome::Unresolved;
        causes.push(cause("target_absent", empty()));
    }
    (reference, causes)
}

fn route_order(left: &DocLinkRoute, right: &DocLinkRoute) -> Ordering {
    let rank = |kind| match kind {
        "current" => 0,
        "import" => 1,
        _ => 2,
    };
    rank(left.kind)
        .cmp(&rank(right.kind))
        .then_with(|| left.position.cmp(&right.position))
        .then_with(|| left.endpoint.0.cmp(&right.endpoint.0))
        .then_with(|| {
            left.endpoint
                .1
                .to_lower_hex()
                .cmp(&right.endpoint.1.to_lower_hex())
        })
        .then_with(|| target_order(&left.candidate, &right.candidate))
}

fn endpoint_target(
    inputs: &AuthenticatedDocImports,
    endpoint: &DirectByEndpoint,
) -> Option<DocLinkTarget> {
    inputs
        .exports(&endpoint.module, endpoint.publication_hash)?
        .contains(&endpoint.export)
        .then(|| {
            target(
                inputs,
                &endpoint.module,
                endpoint.publication_hash,
                &endpoint.export,
            )
        })
}

fn reverse_citations(
    parsed: &ParsedComments<'_, '_>,
    inputs: &AuthenticatedDocImports,
    direct_by: &AuthenticatedDocDirectBy,
) -> (Vec<DocUsedBy>, Vec<Value>, Value) {
    let closure_coverage = inputs.closure_coverage();
    let citation_coverage = direct_by.citation_coverage();
    let mut causes = Vec::new();
    if let Some(reason) =
        coverage_reason(closure_coverage, "closure_unavailable", "closure_partial")
    {
        causes.push(cause(reason, empty()));
    }
    for (module, hash) in inputs.inventory() {
        if let Some(reason) = coverage_reason(
            direct_by.coverage(module, hash),
            "citations_unavailable",
            "citations_partial",
        ) {
            causes.push(cause(reason, affected(module, hash, None)));
        }
    }
    let mut examined = Vec::new();
    for producer in direct_by.producers() {
        for examination in producer.examinations() {
            if let Some(owner) = &examination.owner {
                let mut entry = match candidate_value(&target(
                    inputs,
                    &owner.module,
                    owner.publication_hash,
                    &owner.export,
                )) {
                    Value::List(entry) => entry,
                    _ => unreachable!(),
                };
                let mut parts = vec![value("body")];
                if examination.proof_range.is_some() {
                    parts.push(value("proof"));
                }
                entry.push(Value::List(parts));
                examined.push((
                    target(inputs, &owner.module, owner.publication_hash, &owner.export),
                    Value::List(entry),
                ));
            }
        }
        for occurrence in producer.occurrences() {
            let mapped_target = match &occurrence.target {
                DirectByTarget::Theorem(endpoint) => endpoint_target(inputs, endpoint).is_some(),
                DirectByTarget::NonTheorem => true,
                _ => false,
            };
            if occurrence.citing.is_none() || !mapped_target {
                let affected = occurrence.citing.as_ref().map_or_else(empty, |endpoint| {
                    affected(
                        &endpoint.module,
                        endpoint.publication_hash,
                        Some(&endpoint.export),
                    )
                });
                causes.push(cause("citation_endpoint_unmapped", affected));
            }
        }
    }
    examined.sort_by(|left, right| target_order(&left.0, &right.0));
    examined.dedup_by(|left, right| left.0 == right.0);
    let coverage = Value::List(vec![
        value(coverage_state(closure_coverage)),
        Value::List(
            inputs
                .inventory()
                .map(|(module, hash)| endpoint_value(module, hash))
                .collect(),
        ),
        value(coverage_state(citation_coverage)),
        Value::List(examined.into_iter().map(|entry| entry.1).collect()),
    ]);
    let publication = parsed.attachments.plan.manifest.artifact_hash;
    let module = parsed.attachments.module;
    let mut result = inputs
        .exports(module, publication)
        .into_iter()
        .flatten()
        .filter(|export| export.export_kind == "theorem")
        .map(|export| DocUsedBy {
            target: target(inputs, module, publication, export),
            citing: Vec::new(),
            occurrences: Vec::new(),
            closure_coverage,
            citation_coverage,
            available: causes.is_empty(),
        })
        .collect::<Vec<_>>();
    result.sort_by(|left, right| target_order(&left.target, &right.target));
    let mut producers = direct_by.producers().iter().collect::<Vec<_>>();
    producers.sort_by(|left, right| {
        left.module().cmp(right.module()).then_with(|| {
            left.publication_hash()
                .to_lower_hex()
                .cmp(&right.publication_hash().to_lower_hex())
        })
    });
    for entry in &mut result {
        for producer in &producers {
            for occurrence in producer.occurrences() {
                if let (Some(citing), DirectByTarget::Theorem(cited)) =
                    (&occurrence.citing, &occurrence.target)
                    && cited.module == entry.target.module
                    && cited.publication_hash == entry.target.publication_hash
                    && cited.export == entry.target.export
                    && let Some(citing) = endpoint_target(inputs, citing)
                {
                    if !entry.citing.contains(&citing) {
                        entry.citing.push(citing);
                    }
                    entry.occurrences.push(occurrence.clone());
                }
            }
        }
        entry.citing.sort_by(target_order);
    }
    (result, causes, coverage)
}

fn coverage_state(coverage: DocImportCoverage) -> &'static str {
    match coverage {
        DocImportCoverage::Complete => "complete",
        DocImportCoverage::Unavailable => "unavailable",
        _ => "incomplete",
    }
}

fn value(text: impl ToString) -> Value {
    Value::String(text.to_string())
}
fn empty() -> Value {
    Value::List(Vec::new())
}
fn optional_value(text: Option<&str>) -> Value {
    Value::List(text.into_iter().map(value).collect())
}
fn module_value(module: &ModuleSummaryIdentity) -> Value {
    Value::List(vec![
        value(&module.package_id),
        optional_value(module.package_version.as_deref()),
        optional_value(module.lockfile_identity.as_deref()),
        value(&module.module_path),
        value(&module.language_edition),
    ])
}
fn endpoint_value(module: &ModuleSummaryIdentity, hash: Hash) -> Value {
    Value::List(vec![value(hash.to_lower_hex()), module_value(module)])
}
fn target_value(export: Option<&VerifiedExport>) -> Value {
    Value::List(
        export
            .map(|export| {
                vec![
                    value("item"),
                    value(&export.origin_id),
                    value(&export.fully_qualified_name),
                ]
            })
            .unwrap_or_else(|| vec![value("module")]),
    )
}
fn affected(module: &ModuleSummaryIdentity, hash: Hash, export: Option<&VerifiedExport>) -> Value {
    Value::List(vec![
        value(hash.to_lower_hex()),
        module_value(module),
        target_value(export),
    ])
}
fn candidate_value(candidate: &DocLinkTarget) -> Value {
    Value::List(vec![
        value(candidate.publication_hash.to_lower_hex()),
        module_value(&candidate.module),
        value(&candidate.export.origin_id),
        value(&candidate.export.fully_qualified_name),
    ])
}
fn cause(reason: &str, affected: Value) -> Value {
    Value::List(vec![value(reason), affected])
}

fn link_draft(
    parsed: &ParsedComments<'_, '_>,
    specification_version: &str,
    reference: Option<&DocReferenceResolution>,
    (scope, subject_target): (&str, Option<&DocLinkTarget>),
    number: u16,
    mut causes: Vec<Value>,
    citation_coverage: Value,
) -> DiagnosticDraft {
    let attachments = parsed.attachments;
    let module = attachments.module;
    let publication = attachments.plan.manifest.artifact_hash;
    let occurrence = reference.map(|reference| &reference.occurrence);
    let attached_target =
        occurrence.and_then(|occurrence| attachments.blocks[occurrence.block].target);
    let subject_target = subject_target.map(|target| &target.export);
    let target = if let Some(export) = subject_target {
        target_value(Some(export))
    } else {
        match attached_target {
            Some(CommentTarget::Module) => target_value(None),
            Some(CommentTarget::Item(export)) => target_value(Some(export)),
            _ => empty(),
        }
    };
    let form = |form| match form {
        DocReferenceForm::Code => "code",
        DocReferenceForm::Explicit => "explicit",
        DocReferenceForm::See => "see",
    };
    let subject = Value::List(vec![
        value(publication.to_lower_hex()),
        module_value(module),
        if occurrence.is_none() && subject_target.is_none() {
            target_value(None)
        } else {
            target
        },
        value(scope),
        occurrence.map_or_else(empty, |occurrence| {
            Value::List(vec![
                value(occurrence.block),
                value(occurrence.range.start),
                value(occurrence.range.end),
                value(form(occurrence.form)),
            ])
        }),
    ]);
    let raw_reference = occurrence.map_or_else(empty, |occurrence| {
        Value::List(vec![
            value(&parsed.blocks[occurrence.block].text[occurrence.range.clone()]),
            optional_value(occurrence.interpreted.as_deref()),
        ])
    });
    let section_version = if scope == "section_index"
        || occurrence.is_some_and(|occurrence| {
            occurrence.classification == DocReferenceClassification::Section
        }) {
        optional_value(Some(specification_version))
    } else {
        empty()
    };
    causes.sort();
    causes.dedup();
    let mut details = DiagnosticDetails::new();
    for (key, entry) in [
        ("documentation.link.subject", subject),
        ("documentation.link.reference", raw_reference),
        ("documentation.link.section_version", section_version),
        (
            "documentation.link.candidates",
            reference.map_or_else(empty, |reference| {
                Value::List(reference.candidates.iter().map(candidate_value).collect())
            }),
        ),
        ("documentation.link.causes", Value::List(causes)),
        (
            "documentation.link.routes",
            reference.map_or_else(empty, |reference| {
                Value::List(
                    reference
                        .routes
                        .iter()
                        .map(|route| {
                            Value::List(vec![
                                candidate_value(&route.candidate),
                                value(route.kind),
                                Value::List(route.position.into_iter().map(value).collect()),
                                endpoint_value(&route.endpoint.0, route.endpoint.1),
                            ])
                        })
                        .collect(),
                )
            }),
        ),
        ("documentation.link.citation_coverage", citation_coverage),
    ] {
        details
            .insert(key, entry)
            .expect("fixed adopted link detail keys");
    }
    let mut spans = Vec::new();
    let mut primary_span = None;
    if let Some(occurrence) = occurrence {
        let block = &parsed.blocks[occurrence.block];
        if let Some(source) = attachments.blocks[occurrence.block].source_range {
            for segment in &occurrence.destination_ranges {
                let first = spans.len();
                let mut covered_end = segment.start;
                let mut contiguous = true;
                for (logical, loaded) in &block.source_map {
                    let start = logical.start.max(segment.start);
                    let end = logical.end.min(segment.end);
                    if start < end {
                        let span = SourceRange {
                            source_id: source.source_id,
                            start: loaded.start + start - logical.start,
                            end: loaded.start + end - logical.start,
                        };
                        contiguous &= start == covered_end
                            && (spans.len() == first
                                || spans.last().is_some_and(|previous: &SourceRange| {
                                    previous.end == span.start
                                }));
                        covered_end = end;
                        spans.push(span);
                    }
                }
                if contiguous && covered_end == segment.end && first < spans.len() {
                    let merged = SourceRange {
                        source_id: source.source_id,
                        start: spans[first].start,
                        end: spans.last().expect("mapped segment").end,
                    };
                    spans.truncate(first);
                    if primary_span.is_none() {
                        primary_span = Some(merged);
                    } else {
                        spans.push(merged);
                    }
                }
            }
        }
    }
    let spans = spans.into_iter();
    let primary_location = primary_span.map_or_else(
        || {
            if let Some(occurrence) = occurrence {
                DiagnosticPrimaryLocation::Documentation {
                    publication_hash: publication,
                    package_id: module.package_id.clone(),
                    package_version: module.package_version.clone(),
                    lockfile_identity: module.lockfile_identity.clone(),
                    module_path: module.module_path.clone(),
                    language_edition: module.language_edition.clone(),
                    block_ordinal: occurrence.block as u64,
                }
            } else {
                DiagnosticPrimaryLocation::DocumentationScope {
                    publication_hash: publication,
                    package_id: module.package_id.clone(),
                    package_version: module.package_version.clone(),
                    lockfile_identity: module.lockfile_identity.clone(),
                    module_path: module.module_path.clone(),
                    language_edition: module.language_edition.clone(),
                }
            }
        },
        |span| {
            DiagnosticPrimaryLocation::Span(
                DiagnosticSpan::primary(span, None).expect("nonempty mapped destination"),
            )
        },
    );
    let code = DiagnosticCode::from_parts(DiagnosticSeverity::Warning, number)
        .expect("allocated documentation link code");
    let descriptor = DiagnosticRegistry::builtin()
        .lookup(code)
        .expect("adopted link descriptor");
    DiagnosticDraft::new(DiagnosticDraftInput {
        source_snapshot: attachments.plan.snapshot,
        code,
        phase: PipelinePhase::Documentation,
        category: FailureCategory::DocumentationWarning,
        stable_detail_key: descriptor.meaning_key.to_owned(),
        message: descriptor.summary.to_owned(),
        primary_location,
        secondary_spans: spans
            .map(|span| DiagnosticSpan::secondary(span, None).expect("nonempty mapped destination"))
            .collect(),
        notes: Vec::new(),
        details,
        fixes: Vec::new(),
        explanation: None,
    })
    .expect("bound documentation link draft")
}
