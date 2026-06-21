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
                )
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
mod tests {
    use super::*;
    use crate::env::{ContributionKind, SourceContributionIndex};
    use crate::resolved_ast::{ResolvedArenaBuilder, ResolvedNode, SemanticOrigin};
    use mizar_session::{
        BuildSnapshotId, Hash, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator, SourceAnchor, SourceId,
    };
    use mizar_syntax::ast::SurfaceNodeKind;

    #[test]
    fn unqualified_citation_respects_proof_block_visibility_and_confinement() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let dep = module_id("dep", "logic");
        let namespace = NamespacePath::new("main");
        let mut contributions = SourceContributionIndex::new();
        let local_contribution = contribution(&mut contributions, current.clone(), source_id, 0);
        let imported_contribution = contribution(&mut contributions, dep.clone(), source_id, 100);
        let current_fixture = ProjectionFixture::new(
            source_id,
            current.clone(),
            namespace.clone(),
            local_contribution,
        );
        let dep_fixture = ProjectionFixture::new(
            source_id,
            dep.clone(),
            namespace.clone(),
            imported_contribution,
        );
        let outer_scope = LabelScopePath::new(vec![0]);
        let inner_scope = LabelScopePath::new(vec![0, 1]);

        let projections = vec![
            proof_step_projection(&current_fixture, "A", 10, 1, outer_scope.clone()),
            proof_step_projection(&current_fixture, "B", 20, 2, inner_scope.clone()),
            current_theorem_projection(&current_fixture, "T", 30, 3),
            imported_theorem_projection(&dep_fixture, "Lib", 40),
        ];
        let references = vec![
            unqualified_ref(
                source_id,
                current.clone(),
                2,
                80,
                "T",
                Some(inner_scope.clone()),
            ),
            unqualified_ref(
                source_id,
                current.clone(),
                4,
                60,
                "A",
                Some(inner_scope.clone()),
            ),
            unqualified_ref(source_id, current.clone(), 5, 70, "B", Some(outer_scope)),
            unqualified_ref(source_id, current.clone(), 6, 90, "Lib", Some(inner_scope)),
        ];

        let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

        assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "T");
        assert_resolved_label(&resolved, 1, "app::main::proof::A");
        assert_unresolved_label(&resolved, 2, LabelExpectation::ProofOrTheorem, "B");
        assert_resolved_label(&resolved, 3, "dep::logic::theorem::Lib");
        assert!(resolved.has_unresolved());
    }

    #[test]
    fn duplicate_and_visible_nested_labels_are_internal_diagnostics() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let namespace = NamespacePath::new("main");
        let mut contributions = SourceContributionIndex::new();
        let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
        let fixture =
            ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
        let outer_scope = LabelScopePath::new(vec![0]);
        let inner_scope = LabelScopePath::new(vec![0, 1]);
        let sibling_scope = LabelScopePath::new(vec![0, 2]);
        let outer = proof_step_projection(&fixture, "A", 10, 1, outer_scope.clone());
        let duplicate = proof_step_projection(&fixture, "A", 20, 2, outer_scope);
        let inner_conflict = proof_step_projection(&fixture, "A", 30, 3, inner_scope);
        let sibling_conflict = proof_step_projection(&fixture, "A", 40, 4, sibling_scope);
        let projections = vec![sibling_conflict, inner_conflict, duplicate, outer];

        let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &[]);

        let diagnostics = resolved.diagnostics();
        assert_eq!(diagnostics.len(), 3);
        assert_eq!(
            diagnostics
                .iter()
                .map(LabelDiagnostic::kind)
                .collect::<Vec<_>>(),
            vec![
                LabelDiagnosticKind::DuplicateLabel,
                LabelDiagnosticKind::ConflictingVisibleLabel,
                LabelDiagnosticKind::ConflictingVisibleLabel,
            ]
        );
        assert_eq!(
            diagnostics
                .iter()
                .map(LabelDiagnostic::primary_range)
                .collect::<Vec<_>>(),
            vec![
                range(source_id, 20, 21),
                range(source_id, 30, 31),
                range(source_id, 40, 41),
            ]
        );
        let related_ranges = diagnostics
            .iter()
            .map(|diagnostic| diagnostic.related_ranges().to_vec())
            .collect::<Vec<_>>();
        assert_eq!(
            related_ranges,
            vec![
                vec![range(source_id, 10, 11)],
                vec![range(source_id, 10, 11), range(source_id, 20, 21)],
                vec![range(source_id, 10, 11), range(source_id, 20, 21)],
            ]
        );
    }

    #[test]
    fn forward_references_to_later_theorem_labels_are_unresolved() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let namespace = NamespacePath::new("main");
        let mut contributions = SourceContributionIndex::new();
        let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
        let fixture =
            ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
        let projections = vec![current_theorem_projection(&fixture, "Later", 20, 5)];
        let references = vec![
            unqualified_ref(source_id, current.clone(), 4, 10, "Later", None),
            unqualified_ref(source_id, current.clone(), 6, 30, "Later", None),
        ];

        let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

        assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "Later");
        assert_resolved_label(&resolved, 1, "app::main::theorem::Later");
    }

    #[test]
    fn forward_references_to_later_proof_step_labels_are_unresolved() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let namespace = NamespacePath::new("main");
        let mut contributions = SourceContributionIndex::new();
        let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
        let fixture =
            ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
        let scope = LabelScopePath::new(vec![0]);
        let projections = vec![proof_step_projection(
            &fixture,
            "LaterStep",
            20,
            5,
            scope.clone(),
        )];
        let references = vec![
            unqualified_ref(
                source_id,
                current.clone(),
                4,
                10,
                "LaterStep",
                Some(scope.clone()),
            ),
            unqualified_ref(source_id, current.clone(), 6, 40, "LaterStep", Some(scope)),
        ];

        let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

        assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "LaterStep");
        assert_resolved_label(&resolved, 1, "app::main::proof::LaterStep");
    }

    #[test]
    fn qualified_and_lowered_grouped_item_citations_use_module_label_projections() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let dep = module_id("dep", "logic");
        let namespace = NamespacePath::new("logic");
        let current_namespace = NamespacePath::new("main");
        let mut contributions = SourceContributionIndex::new();
        let imported_contribution = contribution(&mut contributions, dep.clone(), source_id, 0);
        let dep_fixture = ProjectionFixture::new(
            source_id,
            dep.clone(),
            namespace.clone(),
            imported_contribution,
        );
        let projections = vec![
            imported_theorem_projection(&dep_fixture, "Th1", 10),
            imported_theorem_projection(&dep_fixture, "G1", 20),
            imported_theorem_projection(&dep_fixture, "G2", 30),
        ];
        let references = vec![
            qualified_ref(
                source_id,
                current.clone(),
                dep.clone(),
                namespace.clone(),
                2,
                50,
                "Th1",
            ),
            qualified_ref(
                source_id,
                current.clone(),
                dep.clone(),
                namespace.clone(),
                3,
                60,
                "G1",
            ),
            qualified_ref(
                source_id,
                current.clone(),
                dep.clone(),
                namespace,
                4,
                70,
                "G2",
            ),
        ];

        let resolved =
            LabelResolver::new(&projections).resolve(&current, &current_namespace, &references);

        assert_resolved_label(&resolved, 0, "dep::logic::theorem::Th1");
        assert_resolved_label(&resolved, 1, "dep::logic::theorem::G1");
        assert_resolved_label(&resolved, 2, "dep::logic::theorem::G2");
    }

    #[test]
    fn imported_local_only_labels_are_not_visible_to_citations() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let dep = module_id("dep", "logic");
        let namespace = NamespacePath::new("logic");
        let current_namespace = NamespacePath::new("main");
        let mut contributions = SourceContributionIndex::new();
        let imported_contribution = contribution(&mut contributions, dep.clone(), source_id, 0);
        let dep_fixture = ProjectionFixture::new(
            source_id,
            dep.clone(),
            namespace.clone(),
            imported_contribution,
        );
        let projections = vec![
            imported_theorem_projection(&dep_fixture, "Hidden", 10)
                .with_export_status(ExportStatus::LocalOnly),
        ];
        let references = vec![
            unqualified_ref(source_id, current.clone(), 2, 30, "Hidden", None),
            qualified_ref(
                source_id,
                current.clone(),
                dep,
                namespace.clone(),
                3,
                40,
                "Hidden",
            ),
        ];

        let resolved =
            LabelResolver::new(&projections).resolve(&current, &current_namespace, &references);

        assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "Hidden");
        assert_unresolved_label(&resolved, 1, LabelExpectation::Theorem, "Hidden");
        assert!(
            resolved
                .index()
                .visible_candidates(&namespace, "Hidden")
                .is_empty()
        );
    }

    #[test]
    fn recovered_empty_and_failed_namespace_references_are_unresolved() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let namespace = NamespacePath::new("main");
        let scope = LabelScopePath::new(vec![0]);
        let (recovered_site, recovered_origin) =
            reference_site(source_id, current.clone(), 10, "A", 1);
        let (empty_site, empty_origin) = reference_site(source_id, current.clone(), 20, "", 2);
        let (failed_site, failed_origin) = reference_site(source_id, current.clone(), 30, "B", 3);
        let references = vec![
            LabelReferenceCandidate::unqualified_citation(
                recovered_site,
                recovered_origin.recovered(),
                1,
                Some(scope),
            ),
            LabelReferenceCandidate::unqualified_citation(empty_site, empty_origin, 2, None),
            LabelReferenceCandidate::failed_namespace(
                failed_site,
                failed_origin,
                3,
                LabelExpectation::Theorem,
            ),
        ];

        let resolved = LabelResolver::new(&[]).resolve(&current, &namespace, &references);

        assert_unresolved_label(&resolved, 0, LabelExpectation::ProofOrTheorem, "A");
        assert_unresolved_label(&resolved, 1, LabelExpectation::ProofOrTheorem, "");
        assert_unresolved_label(&resolved, 2, LabelExpectation::Theorem, "B");
    }

    #[test]
    fn ambiguous_cross_family_citations_keep_sorted_candidates() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let namespace = NamespacePath::new("main");
        let mut contributions = SourceContributionIndex::new();
        let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
        let fixture =
            ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
        let scope = LabelScopePath::new(vec![0]);
        let projections = vec![
            current_theorem_projection(&fixture, "A", 20, 1),
            proof_step_projection(&fixture, "A", 10, 2, scope.clone()),
        ];
        let references = vec![unqualified_ref(
            source_id,
            current.clone(),
            3,
            30,
            "A",
            Some(scope),
        )];

        let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

        let entry = resolved.table().get(resolved.ids()[0]).unwrap();
        let LabelResolution::Ambiguous(ambiguous) = entry.resolution() else {
            panic!("expected ambiguous label reference");
        };
        assert_eq!(
            ambiguous
                .candidates()
                .iter()
                .map(|candidate| candidate.origin().as_str())
                .collect::<Vec<_>>(),
            vec!["app::main::proof::A", "app::main::theorem::A"]
        );
    }

    #[test]
    fn label_index_and_reference_table_order_are_deterministic() {
        let source_id = source_id();
        let current = module_id("app", "main");
        let namespace = NamespacePath::new("main");
        let mut contributions = SourceContributionIndex::new();
        let contribution = contribution(&mut contributions, current.clone(), source_id, 0);
        let fixture =
            ProjectionFixture::new(source_id, current.clone(), namespace.clone(), contribution);
        let projections = vec![
            current_theorem_projection(&fixture, "Z", 30, 3),
            current_theorem_projection(&fixture, "A", 10, 1),
            current_theorem_projection(&fixture, "M", 20, 2),
        ];
        let references = vec![
            unqualified_ref(source_id, current.clone(), 6, 60, "Z", None),
            unqualified_ref(source_id, current.clone(), 4, 40, "A", None),
            unqualified_ref(source_id, current.clone(), 5, 50, "M", None),
        ];

        let resolved = LabelResolver::new(&projections).resolve(&current, &namespace, &references);

        assert_eq!(
            resolved
                .index()
                .iter()
                .map(|entry| entry.origin_path().as_str())
                .collect::<Vec<_>>(),
            vec![
                "app::main::theorem::A",
                "app::main::theorem::M",
                "app::main::theorem::Z",
            ]
        );
        assert_eq!(
            resolved
                .ids()
                .iter()
                .map(|id| resolved.table().get(*id).unwrap().site().spelling())
                .collect::<Vec<_>>(),
            vec!["A", "M", "Z"]
        );
    }

    fn current_theorem_projection(
        fixture: &ProjectionFixture,
        spelling: &str,
        start: usize,
        visible_after: usize,
    ) -> LabelProjection {
        LabelProjection::current_module(
            fixture.data(
                spelling,
                LabelKind::Theorem,
                "theorem",
                start,
                visible_after,
            ),
            visible_after,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
    }

    fn imported_theorem_projection(
        fixture: &ProjectionFixture,
        spelling: &str,
        start: usize,
    ) -> LabelProjection {
        LabelProjection::imported(fixture.data(
            spelling,
            LabelKind::Theorem,
            "theorem",
            start,
            start,
        ))
    }

    fn proof_step_projection(
        fixture: &ProjectionFixture,
        spelling: &str,
        start: usize,
        visible_after: usize,
        scope: LabelScopePath,
    ) -> LabelProjection {
        LabelProjection::proof_step(
            fixture.data(
                spelling,
                LabelKind::ProofStep,
                "proof",
                start,
                visible_after,
            ),
            visible_after,
            scope,
        )
    }

    #[derive(Clone)]
    struct ProjectionFixture {
        source_id: SourceId,
        module: ModuleId,
        namespace: NamespacePath,
        contribution: SourceContributionId,
    }

    impl ProjectionFixture {
        fn new(
            source_id: SourceId,
            module: ModuleId,
            namespace: NamespacePath,
            contribution: SourceContributionId,
        ) -> Self {
            Self {
                source_id,
                module,
                namespace,
                contribution,
            }
        }

        fn data(
            &self,
            spelling: &str,
            kind: LabelKind,
            origin_role: &str,
            start: usize,
            ordinal: usize,
        ) -> LabelProjectionData {
            let range = range(self.source_id, start, start + spelling.len());
            LabelProjectionData {
                origin_path: LabelOriginPath::new(format!(
                    "{}::{}::{origin_role}::{spelling}",
                    self.module.package().as_str(),
                    self.module.path().as_str()
                )),
                module: self.module.clone(),
                namespace: self.namespace.clone(),
                primary_spelling: spelling.to_owned(),
                kind,
                declaration_range: range,
                origin: origin(self.source_id, self.module.clone(), range, ordinal),
                contribution: self.contribution,
            }
        }
    }

    fn unqualified_ref(
        source_id: SourceId,
        module: ModuleId,
        ordinal: usize,
        start: usize,
        spelling: &str,
        scope: Option<LabelScopePath>,
    ) -> LabelReferenceCandidate {
        let (site, origin) = reference_site(source_id, module, start, spelling, ordinal);
        LabelReferenceCandidate::unqualified_citation(site, origin, ordinal, scope)
    }

    fn qualified_ref(
        source_id: SourceId,
        current: ModuleId,
        target: ModuleId,
        namespace: NamespacePath,
        ordinal: usize,
        start: usize,
        spelling: &str,
    ) -> LabelReferenceCandidate {
        let (site, origin) = reference_site(source_id, current, start, spelling, ordinal);
        LabelReferenceCandidate::qualified_citation(site, origin, ordinal, target, namespace)
    }

    fn reference_site(
        source_id: SourceId,
        module: ModuleId,
        start: usize,
        spelling: &str,
        ordinal: usize,
    ) -> (ReferenceSite, SemanticOrigin) {
        let range = range(source_id, start, start + spelling.len());
        let origin = origin(source_id, module, range, ordinal);
        let mut arena = ResolvedArenaBuilder::new();
        let node = arena
            .push(ResolvedNode::new(
                SurfaceNodeKind::Reference,
                Vec::new(),
                origin.clone(),
            ))
            .unwrap();
        (ReferenceSite::new(node, range, spelling), origin)
    }

    fn origin(
        source_id: SourceId,
        module: ModuleId,
        range: SourceRange,
        ordinal: usize,
    ) -> SemanticOrigin {
        SemanticOrigin::new(
            source_id,
            module,
            SourceAnchor::Range(range),
            vec![ordinal as u32],
        )
    }

    fn contribution(
        index: &mut SourceContributionIndex,
        module: ModuleId,
        source_id: SourceId,
        start: usize,
    ) -> SourceContributionId {
        index.insert(
            module,
            ContributionKind::LocalSource { source_id },
            SourceAnchor::Range(range(source_id, start, start + 1)),
        )
    }

    fn assert_resolved_label(
        resolution: &LabelResolutionResult,
        index: usize,
        expected_origin: &str,
    ) {
        let entry = resolution.table().get(resolution.ids()[index]).unwrap();
        let LabelResolution::Resolved(label) = entry.resolution() else {
            panic!("expected resolved label at index {index}");
        };
        assert_eq!(label.origin().as_str(), expected_origin);
    }

    fn assert_unresolved_label(
        resolution: &LabelResolutionResult,
        index: usize,
        expected_expectation: LabelExpectation,
        expected_spelling: &str,
    ) {
        let entry = resolution.table().get(resolution.ids()[index]).unwrap();
        let LabelResolution::Unresolved(unresolved) = entry.resolution() else {
            panic!("expected unresolved label at index {index}");
        };
        assert_eq!(unresolved.expectation(), expected_expectation);
        assert_eq!(unresolved.spelling(), expected_spelling);
    }

    fn module_id(package: &str, path: &str) -> ModuleId {
        ModuleId::new(PackageId::new(package), ModulePath::new(path))
    }

    fn source_id() -> SourceId {
        let snapshot_id = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "34".repeat(Hash::BYTE_LEN)
        ))
        .unwrap();
        let allocator = InMemorySessionIdAllocator::new();
        allocator.next_source_id(snapshot_id).unwrap()
    }

    const fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }
}
