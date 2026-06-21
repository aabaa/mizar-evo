//! Label declaration and citation resolution.
//!
//! This module implements the R-018 label-resolution slice. It resolves
//! theorem/lemma and proof-step label projections, keeps proof labels in a
//! separate lexical scope family, populates `LabelRefTable` outcomes, and
//! records crate-local/internal label conflict diagnostics. It does not prove
//! statements, instantiate templates, generate obligations, select ATP
//! premises, or assign public resolver diagnostic codes.

use crate::env::{
    ExportStatus, LabelEntry, LabelIndex, NamespacePath, SourceContributionId, Visibility,
};
use crate::recovery::suppress_dependent_diagnostic_for_recovered_origin;
use crate::resolved_ast::{
    AmbiguousLabelRef, LabelCandidate, LabelExpectation, LabelKind, LabelOriginPath, LabelRef,
    LabelRefEntry, LabelRefId, LabelRefTable, LabelResolution, ModuleId, ReferenceSite,
    SemanticOrigin, UnresolvedLabelRef,
};
use mizar_session::SourceRange;
use std::cmp::Ordering;

/// Stable proof-label scope path.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LabelScopePath {
    path: Vec<u32>,
}

impl LabelScopePath {
    /// Creates a label scope path from stable proof-block components.
    #[must_use]
    pub fn new(path: impl Into<Vec<u32>>) -> Self {
        Self { path: path.into() }
    }

    /// Returns the proof-block path components.
    #[must_use]
    pub fn path(&self) -> &[u32] {
        &self.path
    }

    fn contains(&self, other: &Self) -> bool {
        other.path.starts_with(&self.path)
    }
}

/// Source of a label projection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LabelProjectionSource {
    /// Current-module label that becomes visible after the source-order ordinal.
    CurrentModule {
        /// Source-order ordinal after which this declaration is visible.
        visible_after_ordinal: usize,
        /// Proof scope for proof-step labels.
        proof_scope: Option<LabelScopePath>,
    },
    /// Imported public or dependency-summary label projection.
    Imported,
}

/// Common data for a resolver-visible label projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelProjectionData {
    /// Normalized label origin path.
    pub origin_path: LabelOriginPath,
    /// Declaring module.
    pub module: ModuleId,
    /// Namespace projection used for label lookup.
    pub namespace: NamespacePath,
    /// Primary source spelling.
    pub primary_spelling: String,
    /// Label kind.
    pub kind: LabelKind,
    /// Declaration source range.
    pub declaration_range: SourceRange,
    /// Normalized declaration provenance.
    pub origin: SemanticOrigin,
    /// Source contribution id.
    pub contribution: SourceContributionId,
}

/// Resolver-visible label projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelProjection {
    origin_path: LabelOriginPath,
    module: ModuleId,
    namespace: NamespacePath,
    primary_spelling: String,
    kind: LabelKind,
    visibility: Visibility,
    export_status: ExportStatus,
    declaration_range: SourceRange,
    origin: SemanticOrigin,
    contribution: SourceContributionId,
    source: LabelProjectionSource,
}

impl LabelProjection {
    /// Creates a current-module label projection.
    #[must_use]
    pub fn current_module(data: LabelProjectionData, visible_after_ordinal: usize) -> Self {
        Self {
            origin_path: data.origin_path,
            module: data.module,
            namespace: data.namespace,
            primary_spelling: data.primary_spelling,
            kind: data.kind,
            visibility: Visibility::Private,
            export_status: ExportStatus::LocalOnly,
            declaration_range: data.declaration_range,
            origin: data.origin,
            contribution: data.contribution,
            source: LabelProjectionSource::CurrentModule {
                visible_after_ordinal,
                proof_scope: None,
            },
        }
    }

    /// Creates a current-module proof-step label projection.
    #[must_use]
    pub fn proof_step(
        mut data: LabelProjectionData,
        visible_after_ordinal: usize,
        proof_scope: LabelScopePath,
    ) -> Self {
        data.kind = LabelKind::ProofStep;
        Self {
            origin_path: data.origin_path,
            module: data.module,
            namespace: data.namespace,
            primary_spelling: data.primary_spelling,
            kind: LabelKind::ProofStep,
            visibility: Visibility::Private,
            export_status: ExportStatus::LocalOnly,
            declaration_range: data.declaration_range,
            origin: data.origin,
            contribution: data.contribution,
            source: LabelProjectionSource::CurrentModule {
                visible_after_ordinal,
                proof_scope: Some(proof_scope),
            },
        }
    }

    /// Creates an imported theorem/lemma label projection.
    #[must_use]
    pub fn imported(data: LabelProjectionData) -> Self {
        Self {
            origin_path: data.origin_path,
            module: data.module,
            namespace: data.namespace,
            primary_spelling: data.primary_spelling,
            kind: data.kind,
            visibility: Visibility::Public,
            export_status: ExportStatus::ReExported,
            declaration_range: data.declaration_range,
            origin: data.origin,
            contribution: data.contribution,
            source: LabelProjectionSource::Imported,
        }
    }

    /// Sets visibility.
    #[must_use]
    pub const fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Sets export status.
    #[must_use]
    pub const fn with_export_status(mut self, export_status: ExportStatus) -> Self {
        self.export_status = export_status;
        self
    }

    /// Returns the normalized label origin path.
    #[must_use]
    pub const fn origin_path(&self) -> &LabelOriginPath {
        &self.origin_path
    }

    /// Returns the declaring module.
    #[must_use]
    pub const fn module(&self) -> &ModuleId {
        &self.module
    }

    /// Returns the namespace projection.
    #[must_use]
    pub const fn namespace(&self) -> &NamespacePath {
        &self.namespace
    }

    /// Returns the primary source spelling.
    #[must_use]
    pub fn primary_spelling(&self) -> &str {
        &self.primary_spelling
    }

    /// Returns the label kind.
    #[must_use]
    pub const fn kind(&self) -> LabelKind {
        self.kind
    }

    /// Returns visibility.
    #[must_use]
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns export status.
    #[must_use]
    pub const fn export_status(&self) -> ExportStatus {
        self.export_status
    }

    /// Returns the declaration range.
    #[must_use]
    pub const fn declaration_range(&self) -> SourceRange {
        self.declaration_range
    }

    /// Returns normalized provenance.
    #[must_use]
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns the source contribution id.
    #[must_use]
    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    /// Returns projection source metadata.
    #[must_use]
    pub const fn source(&self) -> &LabelProjectionSource {
        &self.source
    }

    fn visible_after_ordinal(&self) -> Option<usize> {
        match &self.source {
            LabelProjectionSource::CurrentModule {
                visible_after_ordinal,
                ..
            } => Some(*visible_after_ordinal),
            LabelProjectionSource::Imported => None,
        }
    }

    fn proof_scope(&self) -> Option<&LabelScopePath> {
        match &self.source {
            LabelProjectionSource::CurrentModule { proof_scope, .. } => proof_scope.as_ref(),
            LabelProjectionSource::Imported => None,
        }
    }

    fn label_entry(&self) -> LabelEntry {
        LabelEntry::new(
            self.origin_path.clone(),
            self.kind,
            self.namespace.clone(),
            self.primary_spelling.clone(),
            self.origin.clone(),
            self.contribution,
        )
        .with_visibility(self.visibility)
        .with_export_status(self.export_status)
    }
}

/// Label-reference lookup scope.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LabelReferenceScope {
    /// Unqualified citation in an optional proof scope.
    Unqualified {
        /// Proof scope at the use site.
        proof_scope: Option<LabelScopePath>,
    },
    /// Qualified citation whose namespace prefix was already resolved.
    Qualified {
        /// Canonical target module.
        module: ModuleId,
        /// Namespace projection used for exported label lookup.
        namespace: NamespacePath,
    },
    /// Citation blocked by an unresolved namespace/module prefix.
    FailedNamespace,
}

/// Source-shaped label reference collected before table insertion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelReferenceCandidate {
    site: ReferenceSite,
    origin: SemanticOrigin,
    ordinal: usize,
    expectation: LabelExpectation,
    scope: LabelReferenceScope,
}

impl LabelReferenceCandidate {
    /// Creates an unqualified proof/theorem citation candidate.
    #[must_use]
    pub const fn unqualified_citation(
        site: ReferenceSite,
        origin: SemanticOrigin,
        ordinal: usize,
        proof_scope: Option<LabelScopePath>,
    ) -> Self {
        Self {
            site,
            origin,
            ordinal,
            expectation: LabelExpectation::ProofOrTheorem,
            scope: LabelReferenceScope::Unqualified { proof_scope },
        }
    }

    /// Creates a qualified theorem citation candidate.
    #[must_use]
    pub const fn qualified_citation(
        site: ReferenceSite,
        origin: SemanticOrigin,
        ordinal: usize,
        module: ModuleId,
        namespace: NamespacePath,
    ) -> Self {
        Self {
            site,
            origin,
            ordinal,
            expectation: LabelExpectation::Theorem,
            scope: LabelReferenceScope::Qualified { module, namespace },
        }
    }

    /// Creates a candidate blocked by an unresolved namespace.
    #[must_use]
    pub const fn failed_namespace(
        site: ReferenceSite,
        origin: SemanticOrigin,
        ordinal: usize,
        expectation: LabelExpectation,
    ) -> Self {
        Self {
            site,
            origin,
            ordinal,
            expectation,
            scope: LabelReferenceScope::FailedNamespace,
        }
    }

    /// Sets the expected label family.
    #[must_use]
    pub const fn with_expectation(mut self, expectation: LabelExpectation) -> Self {
        self.expectation = expectation;
        self
    }

    /// Returns the reference site.
    #[must_use]
    pub const fn site(&self) -> &ReferenceSite {
        &self.site
    }

    /// Returns normalized provenance.
    #[must_use]
    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    /// Returns source-order ordinal.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    /// Returns expected label family.
    #[must_use]
    pub const fn expectation(&self) -> LabelExpectation {
        self.expectation
    }

    /// Returns lookup scope.
    #[must_use]
    pub const fn scope(&self) -> &LabelReferenceScope {
        &self.scope
    }
}

/// Crate-local/internal label diagnostic kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LabelDiagnosticKind {
    /// Duplicate declaration in the same label scope.
    DuplicateLabel,
    /// Inner declaration conflicts with an already visible outer label.
    ConflictingVisibleLabel,
}

/// Crate-local/internal label diagnostic record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelDiagnostic {
    kind: LabelDiagnosticKind,
    spelling: String,
    label_kind: LabelKind,
    origin_path: LabelOriginPath,
    primary_range: SourceRange,
    related_ranges: Vec<SourceRange>,
}

impl LabelDiagnostic {
    fn new(
        kind: LabelDiagnosticKind,
        spelling: String,
        label_kind: LabelKind,
        origin_path: LabelOriginPath,
        primary_range: SourceRange,
        mut related_ranges: Vec<SourceRange>,
    ) -> Self {
        related_ranges.sort_by_key(|range| range_key(*range));
        related_ranges.dedup();
        Self {
            kind,
            spelling,
            label_kind,
            origin_path,
            primary_range,
            related_ranges,
        }
    }

    /// Returns the internal diagnostic kind.
    #[must_use]
    pub const fn kind(&self) -> LabelDiagnosticKind {
        self.kind
    }

    /// Returns the conflicting spelling.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// Returns the label family involved.
    #[must_use]
    pub const fn label_kind(&self) -> LabelKind {
        self.label_kind
    }

    /// Returns the primary label origin path.
    #[must_use]
    pub const fn origin_path(&self) -> &LabelOriginPath {
        &self.origin_path
    }

    /// Returns the primary declaration range.
    #[must_use]
    pub const fn primary_range(&self) -> SourceRange {
        self.primary_range
    }

    /// Returns related declaration ranges.
    #[must_use]
    pub fn related_ranges(&self) -> &[SourceRange] {
        &self.related_ranges
    }
}

/// Deterministic label-resolution output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LabelResolutionResult {
    index: LabelIndex,
    table: LabelRefTable,
    ids: Vec<LabelRefId>,
    diagnostics: Vec<LabelDiagnostic>,
}

impl LabelResolutionResult {
    fn new(
        index: LabelIndex,
        table: LabelRefTable,
        ids: Vec<LabelRefId>,
        mut diagnostics: Vec<LabelDiagnostic>,
    ) -> Self {
        diagnostics.sort_by(label_diagnostic_cmp);
        Self {
            index,
            table,
            ids,
            diagnostics,
        }
    }

    /// Returns the populated label index.
    #[must_use]
    pub const fn index(&self) -> &LabelIndex {
        &self.index
    }

    /// Returns the populated label-reference table.
    #[must_use]
    pub const fn table(&self) -> &LabelRefTable {
        &self.table
    }

    /// Returns table ids in deterministic reference order.
    #[must_use]
    pub fn ids(&self) -> &[LabelRefId] {
        &self.ids
    }

    /// Returns crate-local/internal label diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[LabelDiagnostic] {
        &self.diagnostics
    }

    /// Returns whether any label reference is unresolved.
    #[must_use]
    pub fn has_unresolved(&self) -> bool {
        self.ids.iter().any(|id| {
            self.table
                .get(*id)
                .is_some_and(|entry| matches!(entry.resolution(), LabelResolution::Unresolved(_)))
        })
    }
}

/// Label resolver over preliminary label projections.
pub struct LabelResolver<'a> {
    projections: &'a [LabelProjection],
}

impl<'a> LabelResolver<'a> {
    /// Creates a label resolver.
    #[must_use]
    pub const fn new(projections: &'a [LabelProjection]) -> Self {
        Self { projections }
    }

    /// Resolves label references for the current module.
    #[must_use]
    pub fn resolve(
        &self,
        current_module: &ModuleId,
        current_namespace: &NamespacePath,
        references: &[LabelReferenceCandidate],
    ) -> LabelResolutionResult {
        let mut ordered = references.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| label_reference_candidate_cmp(left, right));

        let mut table = LabelRefTable::new();
        let mut ids = Vec::with_capacity(ordered.len());
        for reference in ordered {
            let resolution = self.resolve_one(current_module, current_namespace, reference);
            let id = table.insert(LabelRefEntry::new(
                reference.site().clone(),
                resolution,
                reference.origin().clone(),
            ));
            ids.push(id);
        }

        LabelResolutionResult::new(self.label_index(), table, ids, self.diagnostics())
    }

    fn label_index(&self) -> LabelIndex {
        let mut index = LabelIndex::new();
        let mut ordered = self.projections.iter().collect::<Vec<_>>();
        ordered.sort_by(|left, right| label_projection_cmp(left, right));
        for projection in ordered {
            if projection_index_visible(projection) {
                index.insert(projection.label_entry());
            }
        }
        index
    }

    fn diagnostics(&self) -> Vec<LabelDiagnostic> {
        let mut diagnostics = Vec::new();
        let mut ordered = self
            .projections
            .iter()
            .filter(|projection| {
                matches!(
                    projection.source(),
                    LabelProjectionSource::CurrentModule { .. }
                ) && !suppress_dependent_diagnostic_for_recovered_origin(projection.origin())
            })
            .collect::<Vec<_>>();
        ordered.sort_by(|left, right| label_projection_cmp(left, right));

        for index in 0..ordered.len() {
            let current = ordered[index];
            let mut duplicate_ranges = Vec::new();
            let mut conflict_ranges = Vec::new();
            for previous in &ordered[..index] {
                match label_conflict_kind(previous, current) {
                    Some(LabelDiagnosticKind::DuplicateLabel) => {
                        duplicate_ranges.push(previous.declaration_range());
                    }
                    Some(LabelDiagnosticKind::ConflictingVisibleLabel) => {
                        conflict_ranges.push(previous.declaration_range());
                    }
                    None => {}
                }
            }
            if !duplicate_ranges.is_empty() {
                diagnostics.push(LabelDiagnostic::new(
                    LabelDiagnosticKind::DuplicateLabel,
                    current.primary_spelling().to_owned(),
                    current.kind(),
                    current.origin_path().clone(),
                    current.declaration_range(),
                    duplicate_ranges,
                ));
            }
            if !conflict_ranges.is_empty() {
                diagnostics.push(LabelDiagnostic::new(
                    LabelDiagnosticKind::ConflictingVisibleLabel,
                    current.primary_spelling().to_owned(),
                    current.kind(),
                    current.origin_path().clone(),
                    current.declaration_range(),
                    conflict_ranges,
                ));
            }
        }
        diagnostics
    }

    fn resolve_one(
        &self,
        current_module: &ModuleId,
        current_namespace: &NamespacePath,
        reference: &LabelReferenceCandidate,
    ) -> LabelResolution {
        if reference.origin().is_recovered() || reference.site().spelling().is_empty() {
            return unresolved_label(reference);
        }

        match reference.scope() {
            LabelReferenceScope::Unqualified { proof_scope } => {
                self.resolve_unqualified(current_module, current_namespace, reference, proof_scope)
            }
            LabelReferenceScope::Qualified { module, namespace } => {
                self.resolve_qualified(current_module, module, namespace, reference)
            }
            LabelReferenceScope::FailedNamespace => unresolved_label(reference),
        }
    }

    fn resolve_unqualified(
        &self,
        current_module: &ModuleId,
        current_namespace: &NamespacePath,
        reference: &LabelReferenceCandidate,
        proof_scope: &Option<LabelScopePath>,
    ) -> LabelResolution {
        let proof_steps = self
            .projections
            .iter()
            .filter(|projection| {
                reference.expectation().accepts(projection.kind())
                    && projection.kind() == LabelKind::ProofStep
                    && projection.module() == current_module
                    && projection.primary_spelling() == reference.site().spelling()
                    && current_projection_visible(projection, current_module, reference.ordinal())
                    && reference_projection_visible(projection)
                    && proof_label_visible(projection.proof_scope(), proof_scope.as_ref())
            })
            .collect::<Vec<_>>();

        let current_theorems = self
            .projections
            .iter()
            .filter(|projection| {
                reference.expectation().accepts(projection.kind())
                    && projection.kind() == LabelKind::Theorem
                    && projection.module() == current_module
                    && projection.namespace() == current_namespace
                    && projection.primary_spelling() == reference.site().spelling()
                    && current_projection_visible(projection, current_module, reference.ordinal())
                    && reference_projection_visible(projection)
            })
            .collect::<Vec<_>>();

        let imported_theorems = self
            .projections
            .iter()
            .filter(|projection| {
                reference.expectation().accepts(projection.kind())
                    && projection.kind() == LabelKind::Theorem
                    && projection.namespace() == current_namespace
                    && projection.primary_spelling() == reference.site().spelling()
                    && imported_projection_visible(projection)
                    && reference_projection_visible(projection)
            })
            .collect::<Vec<_>>();

        let mut candidates = Vec::new();
        candidates.extend(proof_steps);
        candidates.extend(current_theorems);
        candidates.extend(imported_theorems);
        resolution_from_label_candidates(reference, candidates)
    }

    fn resolve_qualified(
        &self,
        current_module: &ModuleId,
        module: &ModuleId,
        namespace: &NamespacePath,
        reference: &LabelReferenceCandidate,
    ) -> LabelResolution {
        let candidates = self
            .projections
            .iter()
            .filter(|projection| {
                reference.expectation().accepts(projection.kind())
                    && projection.kind() == LabelKind::Theorem
                    && projection.module() == module
                    && projection.namespace() == namespace
                    && projection.primary_spelling() == reference.site().spelling()
                    && qualified_projection_visible(projection, current_module, reference.ordinal())
                    && reference_projection_visible(projection)
            })
            .collect::<Vec<_>>();
        resolution_from_label_candidates(reference, candidates)
    }
}

fn resolution_from_label_candidates(
    reference: &LabelReferenceCandidate,
    candidates: Vec<&LabelProjection>,
) -> LabelResolution {
    match candidates.len() {
        0 => unresolved_label(reference),
        1 => {
            let projection = candidates[0];
            LabelResolution::Resolved(LabelRef::new(
                projection.origin_path().clone(),
                projection.kind(),
                reference.site().range(),
            ))
        }
        _ => LabelResolution::Ambiguous(AmbiguousLabelRef::new(
            reference.site().spelling(),
            reference.site().range(),
            candidates
                .into_iter()
                .map(|candidate| {
                    LabelCandidate::new(
                        candidate.origin_path().clone(),
                        candidate.kind(),
                        candidate.declaration_range(),
                    )
                })
                .collect(),
        )),
    }
}

fn unresolved_label(reference: &LabelReferenceCandidate) -> LabelResolution {
    LabelResolution::Unresolved(UnresolvedLabelRef::new(
        reference.site().spelling(),
        reference.site().range(),
        reference.expectation(),
    ))
}

fn current_projection_visible(
    projection: &LabelProjection,
    current_module: &ModuleId,
    ordinal: usize,
) -> bool {
    projection.module() == current_module
        && projection
            .visible_after_ordinal()
            .is_some_and(|visible_after| visible_after < ordinal)
}

fn imported_projection_visible(projection: &LabelProjection) -> bool {
    matches!(projection.source(), LabelProjectionSource::Imported)
        && projection.visibility() == Visibility::Public
        && projection.export_status() != ExportStatus::LocalOnly
}

fn projection_index_visible(projection: &LabelProjection) -> bool {
    matches!(
        projection.source(),
        LabelProjectionSource::CurrentModule { .. }
    ) || imported_projection_visible(projection)
}

fn reference_projection_visible(projection: &LabelProjection) -> bool {
    !projection.origin().is_recovered()
}

fn qualified_projection_visible(
    projection: &LabelProjection,
    current_module: &ModuleId,
    ordinal: usize,
) -> bool {
    if projection.module() == current_module {
        current_projection_visible(projection, current_module, ordinal)
    } else {
        imported_projection_visible(projection)
    }
}

fn proof_label_visible(
    declaration_scope: Option<&LabelScopePath>,
    reference_scope: Option<&LabelScopePath>,
) -> bool {
    match (declaration_scope, reference_scope) {
        (Some(declaration_scope), Some(reference_scope)) => {
            declaration_scope.contains(reference_scope)
        }
        _ => false,
    }
}

fn label_conflict_kind(
    previous: &LabelProjection,
    current: &LabelProjection,
) -> Option<LabelDiagnosticKind> {
    if previous.module() != current.module()
        || previous.namespace() != current.namespace()
        || previous.primary_spelling() != current.primary_spelling()
        || previous.kind() != current.kind()
    {
        return None;
    }

    match (
        previous.kind(),
        previous.proof_scope(),
        current.proof_scope(),
    ) {
        (LabelKind::ProofStep, Some(previous_scope), Some(current_scope)) => {
            if previous_scope == current_scope {
                return Some(LabelDiagnosticKind::DuplicateLabel);
            }
            if previous_scope.contains(current_scope)
                && previous
                    .visible_after_ordinal()
                    .zip(current.visible_after_ordinal())
                    .is_some_and(|(previous_ordinal, current_ordinal)| {
                        previous_ordinal < current_ordinal
                    })
            {
                return Some(LabelDiagnosticKind::ConflictingVisibleLabel);
            }
            None
        }
        (LabelKind::Theorem | LabelKind::Definition | LabelKind::Registration, _, _) => {
            Some(LabelDiagnosticKind::DuplicateLabel)
        }
        _ => None,
    }
}

fn label_projection_cmp(left: &LabelProjection, right: &LabelProjection) -> Ordering {
    label_source_order_key(left)
        .cmp(&label_source_order_key(right))
        .then_with(|| {
            range_key(left.declaration_range()).cmp(&range_key(right.declaration_range()))
        })
        .then_with(|| left.origin_path().cmp(right.origin_path()))
        .then_with(|| left.kind().cmp(&right.kind()))
}

fn label_source_order_key(projection: &LabelProjection) -> (u8, usize) {
    match projection.source() {
        LabelProjectionSource::CurrentModule {
            visible_after_ordinal,
            ..
        } => (0, *visible_after_ordinal),
        LabelProjectionSource::Imported => (1, usize::MAX),
    }
}

fn label_reference_candidate_cmp(
    left: &LabelReferenceCandidate,
    right: &LabelReferenceCandidate,
) -> Ordering {
    left.ordinal()
        .cmp(&right.ordinal())
        .then_with(|| range_key(left.site().range()).cmp(&range_key(right.site().range())))
        .then_with(|| left.site().spelling().cmp(right.site().spelling()))
        .then_with(|| left.expectation().cmp(&right.expectation()))
}

fn label_diagnostic_cmp(left: &LabelDiagnostic, right: &LabelDiagnostic) -> Ordering {
    range_key(left.primary_range())
        .cmp(&range_key(right.primary_range()))
        .then_with(|| left.kind().cmp(&right.kind()))
        .then_with(|| left.spelling().cmp(right.spelling()))
        .then_with(|| left.label_kind().cmp(&right.label_kind()))
        .then_with(|| left.origin_path().cmp(right.origin_path()))
}

fn range_key(range: SourceRange) -> (String, usize, usize) {
    (format!("{:?}", range.source_id), range.start, range.end)
}

#[cfg(test)]
mod tests;
